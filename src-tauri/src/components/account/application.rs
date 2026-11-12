//! Native account use cases and accepted-data queries.
use super::domain::{
    AccountDetailsPatch, AccountPolicy, AccountStatus, AccountView, Preference, Preferences,
};
use crate::components::identity::domain::{AccessRole, AccessStatus, AccountAccess};
use crate::components::sync::domain::{CONTRACT_VERSION, OperationEnvelope};
use crate::shared_kernel::{IdGenerator, id::OperationId};
use crate::shared_kernel::{NightingaleError, id::OperationContext};
use serde::Serialize;

pub type Result<T> = std::result::Result<T, AccountError>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
}

impl AccountError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_owned(),
            message: message.to_owned(),
            operation_id: None,
            correlation_id: None,
        }
    }
}

impl From<NightingaleError> for AccountError {
    fn from(error: NightingaleError) -> Self {
        let code = match error.code {
            crate::shared_kernel::error::ErrorCode::ValidationError => "VALIDATION_ERROR",
            crate::shared_kernel::error::ErrorCode::NotFound => "NOT_FOUND",
            crate::shared_kernel::error::ErrorCode::VersionConflict => "VERSION_CONFLICT",
            crate::shared_kernel::error::ErrorCode::IdempotencyConflict => "IDEMPOTENCY_CONFLICT",
            crate::shared_kernel::error::ErrorCode::TemporaryFailure => "TEMPORARY_FAILURE",
            crate::shared_kernel::error::ErrorCode::InternalError => "INTERNAL_ERROR",
        };
        Self::new(code, &error.message)
    }
}

/// Accepted values only. Consumers cannot read preferences or provisional data here.
pub trait AccountViewQuery {
    fn account_view(&self, scope: &OperationContext) -> Result<Option<AccountView>>;
    fn account_policy(&self, scope: &OperationContext) -> Result<Option<AccountPolicy>>;
}

pub trait AccountReadRepository: AccountViewQuery {
    fn preferences(&self, scope: &OperationContext) -> Result<Preferences>;

    /// The adapter commits one local transaction and never writes an Outbox item.
    fn set_preference(
        &mut self,
        scope: &OperationContext,
        preference: Preference,
    ) -> Result<Preferences>;
}

pub struct AccountScreen {
    pub viewer_role: AccessRole,
    pub account: Option<AccountView>,
    pub preferences: Preferences,
    pub can_manage: bool,
}

pub fn require_access(access: &AccountAccess) -> Result<()> {
    if access.status != AccessStatus::Active {
        return Err(AccountError::new(
            "PERMISSION_DENIED",
            "la cuenta no tiene acceso activo",
        ));
    }

    Ok(())
}

pub fn get_account_view(
    repository: &impl AccountReadRepository,
    access: &AccountAccess,
) -> Result<AccountScreen> {
    require_access(access)?;

    Ok(AccountScreen {
        viewer_role: access.role.clone(),
        account: repository.account_view(&access.context)?,
        preferences: repository.preferences(&access.context)?,
        can_manage: access.role == AccessRole::Owner
            && [
                "UpdateAccountDetails",
                "UpdateAccountPolicy",
                "SetAccountStatus",
            ]
            .iter()
            .all(|action| {
                access
                    .allowed_actions
                    .iter()
                    .any(|allowed| allowed == action)
            }),
    })
}

pub fn get_preferences(
    repository: &impl AccountReadRepository,
    access: &AccountAccess,
) -> Result<Preferences> {
    require_access(access)?;

    repository.preferences(&access.context)
}

pub fn set_preference(
    repository: &mut impl AccountReadRepository,
    access: &AccountAccess,
    preference: Preference,
) -> Result<Preferences> {
    require_access(access)?;

    repository.set_preference(&access.context, preference)
}

pub enum AccountChange {
    Details(AccountDetailsPatch),
    Policy(AccountPolicy),
    Status(AccountStatus),
}

impl AccountChange {
    pub fn action(&self) -> &'static str {
        match self {
            Self::Details(_) => "UpdateAccountDetails",
            Self::Policy(_) => "UpdateAccountPolicy",
            Self::Status(_) => "SetAccountStatus",
        }
    }

    fn payload(&self) -> serde_json::Value {
        match self {
            Self::Details(patch) => serde_json::json!(patch.fields()),
            Self::Policy(policy) => {
                serde_json::json!({"noShowGraceMinutes": policy.no_show_grace_minutes()})
            }
            Self::Status(status) => serde_json::json!({"status": status.as_str()}),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationStatus {
    Accepted,
    Conflict,
    Rejected,
    RetryableFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RefreshStatus {
    Requested,
    Failed,
    NotRequested,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountMutationResult {
    pub operation_id: OperationId,
    pub status: MutationStatus,
    pub entity_version: Option<i64>,
    pub code: Option<String>,
    pub refresh_status: RefreshStatus,
}

pub trait AccountOnlinePort {
    fn refresh_access(&self) -> impl std::future::Future<Output = Result<AccountAccess>> + Send;
    fn send(
        &self,
        operation: &OperationEnvelope,
    ) -> impl std::future::Future<Output = Result<AccountMutationResult>> + Send;
}

fn require_owner(access: &AccountAccess, action: &str) -> Result<()> {
    require_access(access)?;
    if access.role != AccessRole::Owner
        || !access
            .allowed_actions
            .iter()
            .any(|allowed| allowed == action)
    {
        return Err(AccountError::new(
            "PERMISSION_DENIED",
            "la acción requiere permiso de propietario",
        ));
    }

    Ok(())
}

async fn change_account(
    remote: &impl AccountOnlinePort,
    access: &AccountAccess,
    expected_version: i64,
    change: AccountChange,
    ids: &impl IdGenerator,
) -> Result<AccountMutationResult> {
    require_owner(access, change.action())?;
    if expected_version < 1 {
        return Err(AccountError::new(
            "VALIDATION_ERROR",
            "se requiere la versión aceptada de la cuenta",
        ));
    }

    let fresh = remote.refresh_access().await?;
    require_owner(&fresh, change.action())?;

    if fresh.context.account_id != access.context.account_id
        || fresh.context.user_id != access.context.user_id
        || fresh.context.device_id != access.context.device_id
    {
        return Err(AccountError::new(
            "PERMISSION_DENIED",
            "el acceso remoto no coincide con la sesión",
        ));
    }

    let operation = OperationEnvelope {
        operation_id: OperationId::try_from(ids.new_id().to_string())?,
        contract_version: CONTRACT_VERSION,
        action: change.action().to_owned(),
        entity_key: serde_json::json!(access.context.account_id),
        expected_version,
        payload: change.payload(),
        depends_on: vec![],
        occurred_at_local: None,
    };

    operation.canonical_json()?;

    // Each transport attempt uses the same in-memory envelope. This use case has
    // no storage port and cannot queue an online-only operation.
    for attempt in 0..2 {
        match remote.send(&operation).await {
            Ok(result) => return Ok(result),
            Err(mut error) => {
                error.operation_id = Some(operation.operation_id.as_str().to_owned());

                if attempt == 1
                    || !matches!(
                        error.code.as_str(),
                        "TEMPORARY_FAILURE" | "SERVICE_UNAVAILABLE" | "INVALID_RESPONSE"
                    )
                {
                    return Err(error);
                }
            }
        }
    }

    unreachable!("the second attempt always returns")
}

pub async fn update_account_details(
    remote: &impl AccountOnlinePort,
    access: &AccountAccess,
    expected_version: i64,
    patch: AccountDetailsPatch,
    ids: &impl IdGenerator,
) -> Result<AccountMutationResult> {
    change_account(
        remote,
        access,
        expected_version,
        AccountChange::Details(patch),
        ids,
    )
    .await
}

pub async fn update_account_policy(
    remote: &impl AccountOnlinePort,
    access: &AccountAccess,
    expected_version: i64,
    policy: AccountPolicy,
    ids: &impl IdGenerator,
) -> Result<AccountMutationResult> {
    change_account(
        remote,
        access,
        expected_version,
        AccountChange::Policy(policy),
        ids,
    )
    .await
}

pub async fn set_account_status(
    remote: &impl AccountOnlinePort,
    access: &AccountAccess,
    expected_version: i64,
    status: AccountStatus,
    ids: &impl IdGenerator,
) -> Result<AccountMutationResult> {
    change_account(
        remote,
        access,
        expected_version,
        AccountChange::Status(status),
        ids,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::account::domain::Theme;

    #[derive(Default)]
    struct Fake {
        preferences: Preferences,
        writes: usize,
    }
    impl AccountViewQuery for Fake {
        fn account_view(&self, _: &OperationContext) -> Result<Option<AccountView>> {
            Ok(None)
        }
        fn account_policy(&self, _: &OperationContext) -> Result<Option<AccountPolicy>> {
            Ok(None)
        }
    }
    impl AccountReadRepository for Fake {
        fn preferences(&self, _: &OperationContext) -> Result<Preferences> {
            Ok(self.preferences)
        }
        fn set_preference(
            &mut self,
            _: &OperationContext,
            preference: Preference,
        ) -> Result<Preferences> {
            self.writes += 1;
            self.preferences.apply(preference);
            Ok(self.preferences)
        }
    }

    pub(super) fn access() -> AccountAccess {
        AccountAccess {
            context: OperationContext {
                account_id: "019a1234-0000-7000-8000-000000000001"
                    .to_owned()
                    .try_into()
                    .unwrap(),
                user_id: "019a1234-0000-7000-8000-000000000002"
                    .to_owned()
                    .try_into()
                    .unwrap(),
                device_id: "019a1234-0000-7000-8000-000000000003"
                    .to_owned()
                    .try_into()
                    .unwrap(),
                permission_epoch: 1,
            },
            role: AccessRole::Assistant,
            status: AccessStatus::Active,
            allowed_actions: vec![],
            checked_at_millis: 1,
        }
    }

    #[test]
    fn local_use_cases_need_no_network_and_allow_assistant_preferences() {
        let mut repository = Fake::default();
        let access = access();
        let view = get_account_view(&repository, &access).unwrap();
        assert!(view.account.is_none());
        assert!(!view.can_manage);
        assert_eq!(
            get_preferences(&repository, &access).unwrap(),
            Preferences::default()
        );
        assert_eq!(
            set_preference(&mut repository, &access, Preference::Theme(Theme::Dark))
                .unwrap()
                .theme,
            Theme::Dark
        );
        assert_eq!(repository.writes, 1);
    }

    #[test]
    fn revoked_access_cannot_read_or_write_local_account_data() {
        let mut repository = Fake::default();
        let mut access = access();
        access.status = AccessStatus::Revoked;
        assert_eq!(
            get_preferences(&repository, &access).unwrap_err().code,
            "PERMISSION_DENIED"
        );
        assert!(get_account_view(&repository, &access).is_err());
        assert!(set_preference(&mut repository, &access, Preference::Theme(Theme::Dark)).is_err());
        assert_eq!(repository.writes, 0);
    }
}
