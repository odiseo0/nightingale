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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationRecord {
    pub operation_id: OperationId,
    pub status: OriginalStatus,
    pub entity_version: Option<i64>,
    pub state: Option<Value>,
    pub error: Option<RemoteError>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult {
    pub operation_id: OperationId,
    pub status: ResultStatus,
    pub entity_version: Option<i64>,
    pub state: Option<Value>,
    pub error: Option<RemoteError>,
    pub original_result: Option<OperationRecord>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationBatchResult {
    pub results: Vec<OperationResult>,
    pub correlation_id: OperationId,
}

pub struct SyncApi<'a>(pub &'a APITransport);

impl SyncApi<'_> {
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
