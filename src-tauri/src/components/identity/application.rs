use super::{
    api::{AccountAccessResponse, ActivityStatus, DeviceRegistration, IdentityAPI},
    domain::{AccessStatus, AccountAccess, IdentityContext},
    oidc::{OidcSession, logout},
    repository::{AccessStore, SqliteAccessStore},
};
use crate::components::{
    devices::{
        DeviceInstallation, InstallationStore, SqliteInstallationStore,
        activation::{CredentialFailure, DeviceCredential, DeviceCredentials},
    },
    sync::{
        adapters::{
            api::{OriginalStatus, ResultStatus, SyncAPI},
            outbound::SqliteOperationStore,
        },
        application::record_operation,
        domain::{NewOperation, OperationStatus},
    },
};
use crate::platform::{
    config::APIConfig,
    http::{APICode, APIFailure, APITransport},
    secrets::SecretStore,
};
use crate::shared_kernel::{
    Clock, IdGenerator, NightingaleError,
    id::{AccountId, OperationContext},
};
use rusqlite::{Connection, TransactionBehavior};
use serde_json::json;
use std::{collections::HashSet, sync::Mutex};

#[derive(Debug)]
pub enum ActivationFailure {
    Credential(CredentialFailure),
    Remote(APIFailure),
    LocalStorage,
    IdentityMismatch,
    AccountUnavailable,
    RegistrationReview,
    RegistrationFailed {
        status: OperationStatus,
        code: Option<APICode>,
    },
}

pub type ActivationResult<T> = Result<T, ActivationFailure>;

/// Native access use cases. SQLite guards end before each HTTPS request.
pub struct AccountActivation<'a, S> {
    pub config: &'a APIConfig,
    pub profile: &'a str,
    pub http: &'a APITransport,
    pub secrets: &'a S,
    pub database: &'a Mutex<Connection>,
}

impl<S: SecretStore> AccountActivation<'_, S> {
    fn local<T>(
        &self,
        work: impl FnOnce(&mut Connection) -> crate::shared_kernel::Result<T>,
    ) -> ActivationResult<T> {
        let mut connection = self
            .database
            .lock()
            .map_err(|_| ActivationFailure::LocalStorage)?;

        work(&mut connection).map_err(|_| ActivationFailure::LocalStorage)
    }

    fn credentials<'a>(&'a self, identity: (&'a str, &'a str)) -> DeviceCredentials<'a, S> {
        DeviceCredentials {
            config: self.config,
            profile: self.profile,
            identity,
            secrets: self.secrets,
        }
    }

    fn deny(
        &self,
        credential: &DeviceCredential,
        account: Option<&AccountId>,
    ) -> ActivationResult<()> {
        self.local(|connection| {
            let tx = connection
                .transaction()
                .map_err(|_| NightingaleError::internal("transacción de acceso fallida"))?;
            SqliteAccessStore.deny_cached_access(
                &tx,
                &credential.user_id,
                &credential.device_id,
                account,
            )?;
            tx.commit()
                .map_err(|_| NightingaleError::internal("transacción de acceso fallida"))
        })
    }

    fn remote_error(
        &self,
        error: APIFailure,
        credential: &DeviceCredential,
        account: &AccountId,
    ) -> ActivationFailure {
        if matches!(error.status, Some(401 | 403)) {
            let scope = if error.status == Some(401) {
                None
            } else {
                Some(account)
            };

            if self.deny(credential, scope).is_err() {
                return ActivationFailure::LocalStorage;
            }
        }

        ActivationFailure::Remote(error)
    }

    pub async fn activate(
        &self,
        session: &OidcSession,
        account: &AccountId,
        ids: &impl IdGenerator,
        clock: &impl Clock,
    ) -> ActivationResult<AccountAccess> {
        let registration = DeviceRegistration {
            device_name: format!("Nightingale ({})", self.config.platform()),
            client_version: self.config.client_version().into(),
            local_schema_version: self.local(|connection| {
                connection
                    .pragma_query_value(None, "user_version", |row| row.get(0))
                    .map_err(|_| NightingaleError::internal("esquema no disponible"))
            })?,
        };
        let credentials = self.credentials((session.issuer(), session.subject()));
        let credential = credentials
            .ensure(self.http, session.access_token(), &registration)
            .await
            .map_err(ActivationFailure::Credential)?;
        let api = IdentityAPI(self.http);
        let me = api
            .me(&credential.token)
            .await
            .map_err(|error| self.remote_error(error, &credential, account))?
            .value;

        if me.user_id != credential.user_id
            || me.oidc_issuer != session.issuer()
            || me.oidc_subject != session.subject()
        {
            return Err(ActivationFailure::IdentityMismatch);
        }

        if matches!(me.status, ActivityStatus::Inactive) {
            self.deny(&credential, None)?;

            return Err(ActivationFailure::AccountUnavailable);
        }

        let mut page = 1_u64;
        let mut seen = HashSet::new();
        let available = loop {
            let previous_count = seen.len();
            let accounts = api
                .accounts(&credential.token, page, 100)
                .await
                .map_err(|error| self.remote_error(error, &credential, account))?
                .value;
            let mut selected = None;

            for item in accounts.items {
                if !seen.insert(item.id.as_str().to_owned()) {
                    return Err(ActivationFailure::Remote(APIFailure::local(
                        APICode::InvalidResponse,
                    )));
                }

                if item.id == *account {
                    selected = Some(matches!(item.status, ActivityStatus::Active));
                }
            }

            if let Some(active) = selected {
                break active;
            }

            if seen.len() as u64 >= accounts.total {
                break false;
            }

            if seen.len() == previous_count {
                return Err(ActivationFailure::Remote(APIFailure::local(
                    APICode::InvalidResponse,
                )));
            }

            page = page.checked_add(1).ok_or_else(|| {
                ActivationFailure::Remote(APIFailure::local(APICode::InvalidResponse))
            })?;
        };

        if !available {
            self.deny(&credential, Some(account))?;

            return Err(ActivationFailure::AccountUnavailable);
        }

        let identity = IdentityContext {
            issuer: session.issuer().into(),
            subject: session.subject().into(),
            user_id: credential.user_id.clone(),
            selected_account_id: Some(account.clone()),
        };
        self.local(|connection| {
            let tx = connection
                .transaction()
                .map_err(|_| NightingaleError::internal("transacción de identidad fallida"))?;

            SqliteInstallationStore.save(
                &tx,
                &DeviceInstallation {
                    device_id: credential.device_id.clone(),
                    display_name: registration.device_name,
                    platform: self.config.platform().into(),
                    client_version: registration.client_version,
                },
            )?;

            SqliteAccessStore.save_context(&tx, &identity)?;
            tx.commit()
                .map_err(|_| NightingaleError::internal("transacción de identidad fallida"))
        })?;
        let access = self.fetch_access(&credential, account, clock).await?;

        if access.status != AccessStatus::RegistrationRequired {
            return Ok(access);
        }

        let operation = self.local(|connection| {
            // Lock the read/create pair so concurrent calls retain one operation ID.
            // https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.transaction_with_behavior
            let tx = connection
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(|_| NightingaleError::internal("transacción de registro fallida"))?;
            let existing = SqliteOperationStore.find_registration(&tx, &access.context)?;

            if existing.is_none() {
                record_operation(
                    &tx,
                    &SqliteOperationStore,
                    ids,
                    clock,
                    NewOperation {
                        context: access.context.clone(),
                        depends_on: vec![],
                        action: "RegisterAccountDevice".into(),
                        entity_type: "account_device".into(),
                        entity_key: json!(credential.device_id),
                        expected_version: 0,
                        payload: json!({}),
                        occurred_at_local: None,
                    },
                )?;
            }

            let result = SqliteOperationStore
                .find_registration(&tx, &access.context)?
                .ok_or_else(|| NightingaleError::internal("registro no disponible"))?;

            tx.commit()
                .map_err(|_| NightingaleError::internal("transacción de registro fallida"))?;

            Ok(result)
        })?;

        let (operation, status) = operation;

        if !matches!(
            status,
            OperationStatus::Pending | OperationStatus::Unknown | OperationStatus::Retryable
        ) {
            return Err(ActivationFailure::RegistrationReview);
        }

        let envelope = operation
            .envelope()
            .map_err(|_| ActivationFailure::RegistrationReview)?;

        if operation.semantic_fingerprint.as_deref()
            != Some(
                &envelope
                    .semantic_fingerprint()
                    .map_err(|_| ActivationFailure::RegistrationReview)?,
            )
            || operation.entity_type != "account_device"
            || envelope.entity_key != json!(credential.device_id)
            || envelope.expected_version != 0
            || envelope.payload != json!({})
            || !envelope.depends_on.is_empty()
        {
            return Err(ActivationFailure::RegistrationReview);
        }

        self.local(|connection| {
            let tx = connection
                .transaction()
                .map_err(|_| NightingaleError::internal("transacción de registro fallida"))?;
            SqliteOperationStore.registration_attempt(&tx, operation.operation_id)?;
            tx.commit()
                .map_err(|_| NightingaleError::internal("transacción de registro fallida"))
        })?;

        let result = SyncAPI(self.http)
            .send_operations(&credential.token, account, &[envelope])
            .await
            .map_err(|error| self.remote_error(error, &credential, account))?
            .value
            .results
            .remove(0);
        let (status, version, state, error) = if result.status == ResultStatus::Duplicate {
            let original = result
                .original_result
                .as_ref()
                .ok_or(ActivationFailure::RegistrationReview)?;
            (
                original.status,
                original.entity_version,
                &original.state,
                &original.error,
            )
        } else {
            let status = match result.status {
                ResultStatus::Accepted => OriginalStatus::Accepted,
                ResultStatus::Conflict => OriginalStatus::Conflict,
                ResultStatus::Rejected => OriginalStatus::Rejected,
                ResultStatus::RetryableFailure => OriginalStatus::RetryableFailure,
                ResultStatus::Duplicate => unreachable!(),
            };
            (status, result.entity_version, &result.state, &result.error)
        };

        if status == OriginalStatus::Accepted
            && (error.is_some()
                || version.is_none_or(|v| v < 1)
                || state.as_ref().is_none_or(|state| {
                    state["deviceId"] != json!(credential.device_id)
                        || state["status"] != "active"
                        || state["version"] != json!(version)
                }))
        {
            return Err(ActivationFailure::Remote(APIFailure::local(
                APICode::InvalidResponse,
            )));
        }

        let local_status = match status {
            OriginalStatus::Accepted => OperationStatus::Accepted,
            OriginalStatus::Conflict => OperationStatus::Conflict,
            OriginalStatus::Rejected => OperationStatus::Rejected,
            OriginalStatus::RetryableFailure => OperationStatus::Retryable,
        };
        let error_code = error.as_ref().map(|error| error.code);
        let stored = serde_json::to_value(&result).map_err(|_| ActivationFailure::LocalStorage)?;
        self.local(|connection| {
            let tx = connection
                .transaction()
                .map_err(|_| NightingaleError::internal("transacción de registro fallida"))?;
            SqliteOperationStore.registration_result(
                &tx,
                operation.operation_id,
                local_status,
                &stored,
            )?;

            tx.commit()
                .map_err(|_| NightingaleError::internal("transacción de registro fallida"))
        })?;

        if local_status != OperationStatus::Accepted {
            return Err(ActivationFailure::RegistrationFailed {
                status: local_status,
                code: error_code,
            });
        }

        self.fetch_access(&credential, account, clock).await
    }

    async fn fetch_access(
        &self,
        credential: &DeviceCredential,
        account: &AccountId,
        clock: &impl Clock,
    ) -> ActivationResult<AccountAccess> {
        let response: AccountAccessResponse = IdentityAPI(self.http)
            .access(&credential.token, account)
            .await
            .map_err(|error| self.remote_error(error, credential, account))?
            .value;

        if response.account_id != *account
            || response.user_id != credential.user_id
            || response.device_id != credential.device_id
        {
            return Err(ActivationFailure::IdentityMismatch);
        }

        let checked = clock
            .now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| ActivationFailure::LocalStorage)?
            .as_millis();
        let checked_at_millis =
            i64::try_from(checked).map_err(|_| ActivationFailure::LocalStorage)?;
        let access = AccountAccess {
            context: OperationContext {
                account_id: response.account_id,
                user_id: response.user_id,
                device_id: response.device_id,
                permission_epoch: response.permission_epoch,
            },
            role: response.role,
            status: response.status,
            allowed_actions: response.allowed_actions,
            checked_at_millis,
        };

        self.local(|connection| {
            let tx = connection
                .transaction()
                .map_err(|_| NightingaleError::internal("transacción de acceso fallida"))?;
            SqliteAccessStore.save_access(&tx, &access)?;
            tx.commit()
                .map_err(|_| NightingaleError::internal("transacción de acceso fallida"))
        })?;

        Ok(access)
    }

    pub fn cached_access(
        &self,
        identity: (&str, &str),
        account: &AccountId,
    ) -> ActivationResult<Option<AccountAccess>> {
        let Some(credential) = self
            .credentials(identity)
            .load()
            .map_err(ActivationFailure::Credential)?
        else {
            return Ok(None);
        };
        self.local(|connection| {
            let context = SqliteAccessStore.context(connection)?;

            if context.is_none_or(|context| {
                context.issuer != identity.0
                    || context.subject != identity.1
                    || context.user_id != credential.user_id
            }) {
                return Ok(None);
            }

            SqliteAccessStore.access(
                connection,
                account,
                &credential.user_id,
                &credential.device_id,
            )
        })
    }

    pub fn logout(&self, identity: (&str, &str)) -> ActivationResult<()> {
        self.credentials(identity)
            .forget()
            .map_err(ActivationFailure::Credential)?;
        logout(self.config, self.profile, identity, None, self.secrets)
            .map_err(|_| ActivationFailure::LocalStorage)?;
        self.local(|connection| {
            let tx = connection
                .transaction()
                .map_err(|_| NightingaleError::internal("transacción de salida fallida"))?;
            if SqliteAccessStore.context(&tx)?.is_some_and(|context| {
                context.issuer == identity.0 && context.subject == identity.1
            }) {
                SqliteAccessStore.clear_context(&tx)?;
            }
            tx.commit()
                .map_err(|_| NightingaleError::internal("transacción de salida fallida"))
        })
    }
}

#[cfg(test)]
#[path = "activation_tests.rs"]
mod tests;
