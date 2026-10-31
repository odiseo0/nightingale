use crate::components::sync::domain::OperationEnvelope;
use crate::platform::http::{
    APICode, APIFailure, APIResponse, APIResult, APITransport, RemoteError,
};
use crate::platform::secrets::Secret;
use crate::shared_kernel::id::{AccountId, OperationId};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

// These contract fields are required even when their value is null.
// https://serde.rs/field-attrs.html#deserialize_with
pub(super) fn required_nullable<'de, D, T>(
    deserializer: D,
) -> std::result::Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultStatus {
    Accepted,
    Duplicate,
    Conflict,
    Rejected,
    RetryableFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OriginalStatus {
    Accepted,
    Conflict,
    Rejected,
    RetryableFailure,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationRecord {
    pub operation_id: OperationId,
    pub status: OriginalStatus,
    #[serde(deserialize_with = "required_nullable")]
    pub entity_version: Option<i64>,
    #[serde(deserialize_with = "required_nullable")]
    pub state: Option<Value>,
    #[serde(deserialize_with = "required_nullable")]
    pub error: Option<RemoteError>,
}

impl OperationRecord {
    pub fn local_status(&self) -> crate::components::sync::domain::OperationStatus {
        use crate::components::sync::domain::OperationStatus;

        match self.status {
            OriginalStatus::Accepted => OperationStatus::Accepted,
            OriginalStatus::Conflict => OperationStatus::Conflict,
            OriginalStatus::Rejected => OperationStatus::Rejected,
            OriginalStatus::RetryableFailure => OperationStatus::Retryable,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult {
    pub operation_id: OperationId,
    pub status: ResultStatus,
    #[serde(deserialize_with = "required_nullable")]
    pub entity_version: Option<i64>,
    #[serde(deserialize_with = "required_nullable")]
    pub state: Option<Value>,
    #[serde(deserialize_with = "required_nullable")]
    pub error: Option<RemoteError>,
    pub original_result: Option<OperationRecord>,
}

impl OperationResult {
    /// A duplicate carries the original outcome, including rejection or conflict.
    pub fn original(&self) -> APIResult<OperationRecord> {
        let status = match self.status {
            ResultStatus::Duplicate => {
                return self
                    .original_result
                    .as_ref()
                    .filter(|original| original.operation_id == self.operation_id)
                    .cloned()
                    .ok_or_else(|| APIFailure::local(APICode::InvalidResponse));
            }
            ResultStatus::Accepted => OriginalStatus::Accepted,
            ResultStatus::Conflict => OriginalStatus::Conflict,
            ResultStatus::Rejected => OriginalStatus::Rejected,
            ResultStatus::RetryableFailure => OriginalStatus::RetryableFailure,
        };

        Ok(OperationRecord {
            operation_id: self.operation_id.clone(),
            status,
            entity_version: self.entity_version,
            state: self.state.clone(),
            error: self.error.clone(),
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationBatchResult {
    pub results: Vec<OperationResult>,
    pub correlation_id: OperationId,
}

pub struct SyncAPI<'a>(pub &'a APITransport);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePage {
    pub groups: Vec<super::super::remote::ChangeGroup>,
    pub next_cursor: String,
    pub has_more: bool,
}

impl SyncAPI<'_> {
    pub async fn create_snapshot(
        &self,
        token: &Secret,
        account: &AccountId,
    ) -> APIResult<APIResponse<super::super::snapshot::Snapshot>> {
        self.0
            .request(
                Method::POST,
                &format!("v1/accounts/{}/sync/snapshots", account.as_str()),
                Some(token),
                Some(b"{}".to_vec()),
            )
            .await
    }

    pub async fn snapshot(
        &self,
        token: &Secret,
        account: &AccountId,
        snapshot: &OperationId,
    ) -> APIResult<APIResponse<super::super::snapshot::Snapshot>> {
        self.0
            .request(
                Method::GET,
                &format!(
                    "v1/accounts/{}/sync/snapshots/{}",
                    account.as_str(),
                    snapshot.as_str()
                ),
                Some(token),
                None,
            )
            .await
    }

    pub async fn snapshot_items(
        &self,
        token: &Secret,
        account: &AccountId,
        snapshot: &OperationId,
        page: i64,
    ) -> APIResult<APIResponse<super::super::snapshot::SnapshotPage>> {
        self.0
            .request_with_query(
                Method::GET,
                &format!(
                    "v1/accounts/{}/sync/snapshots/{}/items",
                    account.as_str(),
                    snapshot.as_str()
                ),
                Some(token),
                None,
                &[("page", &page.to_string()), ("shows", "100")],
            )
            .await
    }

    pub async fn release_snapshot(
        &self,
        token: &Secret,
        account: &AccountId,
        snapshot: &OperationId,
    ) -> APIResult<APIResponse<()>> {
        self.0
            .request(
                Method::DELETE,
                &format!(
                    "v1/accounts/{}/sync/snapshots/{}",
                    account.as_str(),
                    snapshot.as_str()
                ),
                Some(token),
                None,
            )
            .await
    }

    pub async fn changes(
        &self,
        token: &Secret,
        account: &AccountId,
        cursor: &str,
    ) -> APIResult<APIResponse<ChangePage>> {
        self.0
            .request_with_query(
                Method::GET,
                &format!("v1/accounts/{}/sync/changes", account.as_str()),
                Some(token),
                None,
                &[("cursor", cursor)],
            )
            .await
    }

    pub async fn send_operations(
        &self,
        token: &Secret,
        account: &AccountId,
        operations: &[OperationEnvelope],
    ) -> APIResult<APIResponse<OperationBatchResult>> {
        if operations.is_empty() || operations.len() > 100 {
            return Err(APIFailure::local(APICode::ValidationError));
        }

        let mut ids = HashSet::new();

        for operation in operations {
            operation
                .canonical_json()
                .map_err(|_| APIFailure::local(APICode::ValidationError))?;

            if !ids.insert(operation.operation_id.as_str()) {
                return Err(APIFailure::local(APICode::ValidationError));
            }
        }

        let body = serde_json::to_vec(&serde_json::json!({"operations": operations}))
            .map_err(|_| APIFailure::local(APICode::ValidationError))?;
        let response: APIResponse<OperationBatchResult> = self
            .0
            .request(
                Method::POST,
                &format!("v1/accounts/{}/sync/operations", account.as_str()),
                Some(token),
                Some(body),
            )
            .await?;

        for result in &response.value.results {
            if !ids.remove(result.operation_id.as_str())
                || (result.status == ResultStatus::Duplicate
                    && result
                        .original_result
                        .as_ref()
                        .is_none_or(|original| original.operation_id != result.operation_id))
            {
                return Err(APIFailure::local(APICode::InvalidResponse));
            }
        }

        if !ids.is_empty() {
            return Err(APIFailure::local(APICode::InvalidResponse));
        }

        Ok(response)
    }

    pub async fn operation_result(
        &self,
        token: &Secret,
        account: &AccountId,
        operation: &OperationId,
    ) -> APIResult<APIResponse<OperationRecord>> {
        let response: APIResponse<OperationRecord> = self
            .0
            .request(
                Method::GET,
                &format!(
                    "v1/accounts/{}/sync/operations/{}",
                    account.as_str(),
                    operation.as_str()
                ),
                Some(token),
                None,
            )
            .await?;

        if response.value.operation_id != *operation {
            return Err(APIFailure::local(APICode::InvalidResponse));
        }

        Ok(response)
    }
}
