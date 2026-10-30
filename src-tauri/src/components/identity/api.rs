use super::domain::{AccessRole, AccessStatus};
use crate::platform::http::{APICode, APIFailure, APIResponse, APIResult, APITransport};
use crate::platform::secrets::Secret;
use crate::shared_kernel::id::{AccountId, DeviceId, OperationId, UserId};
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationFlow {
    #[default]
    AuthorizationCode,
}

#[derive(Default, Deserialize)]
pub enum PkceMethod {
    #[default]
    S256,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthConfig {
    pub issuer: String,
    pub public_client_id: String,
    pub audience: String,
    #[serde(default)]
    pub authorization_flow: AuthorizationFlow,
    #[serde(default)]
    pub pkce_method: PkceMethod,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceRegistration {
    pub device_name: String,
    pub client_version: String,
    pub local_schema_version: i64,
}

#[derive(Default, Deserialize)]
pub enum TokenType {
    #[default]
    Bearer,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceTokenIssued {
    pub access_token: Secret,
    pub token_id: OperationId,
    pub device_id: DeviceId,
    pub user_id: UserId,
    pub issued_at: String,
    #[serde(default)]
    pub token_type: TokenType,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityStatus {
    Active,
    Inactive,
}

#[derive(Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSummary {
    pub id: AccountId,
    pub display_name: String,
    pub status: ActivityStatus,
    pub role: AccessRole,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Me {
    pub user_id: UserId,
    pub display_name: String,
    pub status: ActivityStatus,
    pub oidc_issuer: String,
    pub oidc_subject: String,
    pub accounts: Page<AccountSummary>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountAccessResponse {
    pub account_id: AccountId,
    pub user_id: UserId,
    pub device_id: DeviceId,
    pub role: AccessRole,
    pub status: AccessStatus,
    pub permission_epoch: i64,
    pub allowed_actions: Vec<String>,
}

pub struct IdentityApi<'a>(pub &'a APITransport);

impl IdentityApi<'_> {
    pub async fn auth_config(&self) -> APIResult<APIResponse<AuthConfig>> {
        self.0
            .request(Method::GET, "v1/auth/config", None, None)
            .await
    }

    pub async fn issue_device_token(
        &self,
        oidc_access: &Secret,
        registration: &DeviceRegistration,
    ) -> APIResult<APIResponse<DeviceTokenIssued>> {
        if registration.device_name.is_empty()
            || registration.client_version.is_empty()
            || registration.local_schema_version < 1
        {
            return Err(APIFailure::local(APICode::ValidationError));
        }

        let body = serde_json::to_vec(registration)
            .map_err(|_| APIFailure::local(APICode::ValidationError))?;

        self.0
            .request(
                Method::POST,
                "v1/auth/device-tokens",
                Some(oidc_access),
                Some(body),
            )
            .await
    }

    pub async fn me(&self, token: &Secret) -> APIResult<APIResponse<Me>> {
        self.0
            .request(Method::GET, "v1/me", Some(token), None)
            .await
    }

    pub async fn accounts(
        &self,
        token: &Secret,
        page: u64,
        shows: u16,
    ) -> APIResult<APIResponse<Page<AccountSummary>>> {
        if page == 0 || !(1..=500).contains(&shows) {
            return Err(APIFailure::local(APICode::ValidationError));
        }

        self.0
            .request(
                Method::GET,
                &format!("v1/accounts?page={page}&shows={shows}"),
                Some(token),
                None,
            )
            .await
    }

    pub async fn access(
        &self,
        token: &Secret,
        account: &AccountId,
    ) -> APIResult<APIResponse<AccountAccessResponse>> {
        self.0
            .request(
                Method::GET,
                &format!("v1/accounts/{}/access", account.as_str()),
                Some(token),
                None,
            )
            .await
    }
}
