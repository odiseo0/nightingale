use crate::shared_kernel::id::{AccountId, OperationContext, UserId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessRole {
    Owner,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessStatus {
    Active,
    RegistrationRequired,
    Revoked,
    Inactive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountAccess {
    pub context: OperationContext,
    pub role: AccessRole,
    pub status: AccessStatus,
    pub allowed_actions: Vec<String>,
    pub checked_at_millis: i64,
}

impl AccountAccess {
    pub fn allows_offline(&self, action: &str) -> bool {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Capabilities {
            online_only_actions: Vec<String>,
            operation_catalog: Vec<String>,
        }

        static CAPABILITIES: std::sync::LazyLock<Option<Capabilities>> =
            std::sync::LazyLock::new(|| {
                serde_json::from_str(include_str!(
                    "../../../../docs/contracts/api-v1/capabilities.json"
                ))
                .ok()
            });

        self.status == AccessStatus::Active
            && self.allowed_actions.iter().any(|allowed| allowed == action)
            && CAPABILITIES.as_ref().is_some_and(|caps| {
                caps.operation_catalog.iter().any(|known| known == action)
                    && !caps
                        .online_only_actions
                        .iter()
                        .any(|online| online == action)
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityContext {
    pub issuer: String,
    pub subject: String,
    pub user_id: UserId,
    pub selected_account_id: Option<AccountId>,
}
