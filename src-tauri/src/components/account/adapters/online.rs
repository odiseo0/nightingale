use super::super::application::{
    AccountError, AccountMutationResult, AccountOnlinePort, MutationStatus, RefreshStatus, Result,
};
use crate::{
    components::{
        identity::{api::IdentityAPI, domain::AccountAccess},
        sync::{
            adapters::api::{OriginalStatus, SyncAPI},
            domain::OperationEnvelope,
        },
    },
    platform::{
        http::{APIFailure, APITransport},
        secrets::Secret,
    },
    shared_kernel::id::OperationContext,
};

pub struct OnlineAccount<'a> {
    pub http: &'a APITransport,
    pub token: &'a Secret,
    pub scope: &'a OperationContext,
}

pub fn remote_error(error: APIFailure) -> AccountError {
    let code = serde_json::to_value(error.code)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_else(|| "TEMPORARY_FAILURE".to_owned());
    let mut result = AccountError::new(
        &code,
        "no se pudo completar la acción; revise acceso y conexión",
    );
    result.correlation_id = error.correlation_id.map(|id| id.as_str().to_owned());

    result
}

impl AccountOnlinePort for OnlineAccount<'_> {
    async fn refresh_access(&self) -> Result<AccountAccess> {
        let access = IdentityAPI(self.http)
            .access(self.token, &self.scope.account_id)
            .await
            .map_err(remote_error)?
            .value;

        Ok(AccountAccess {
            context: OperationContext {
                account_id: access.account_id,
                user_id: access.user_id,
                device_id: access.device_id,
                permission_epoch: access.permission_epoch,
            },
            role: access.role,
            status: access.status,
            allowed_actions: access.allowed_actions,
            checked_at_millis: 0,
        })
    }

    async fn send(&self, operation: &OperationEnvelope) -> Result<AccountMutationResult> {
        let response = SyncAPI(self.http)
            .send_operations(
                self.token,
                &self.scope.account_id,
                std::slice::from_ref(operation),
            )
            .await
            .map_err(remote_error)?;
        let result = response
            .value
            .results
            .into_iter()
            .next()
            .ok_or_else(|| {
                AccountError::new("INVALID_RESPONSE", "falta el resultado de la operación")
            })?
            .original()
            .map_err(remote_error)?;

        Ok(AccountMutationResult {
            operation_id: result.operation_id,
            status: match result.status {
                OriginalStatus::Accepted => MutationStatus::Accepted,
                OriginalStatus::Conflict => MutationStatus::Conflict,
                OriginalStatus::Rejected => MutationStatus::Rejected,
                OriginalStatus::RetryableFailure => MutationStatus::RetryableFailure,
            },
            entity_version: result.entity_version,
            code: result.error.map(|error| {
                serde_json::to_value(error.code)
                    .ok()
                    .and_then(|value| value.as_str().map(str::to_owned))
                    .unwrap_or_else(|| "UNKNOWN".to_owned())
            }),
            refresh_status: RefreshStatus::NotRequested,
        })
    }
}

#[cfg(test)]
#[path = "../online_tests.rs"]
mod tests;
