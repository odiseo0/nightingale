use crate::components::sync::adapters::outbound::SqliteOperationStore;
use crate::components::sync::application::OperationStore;
use crate::shared_kernel::{NightingaleError, Result};
use rusqlite::Connection;
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingOperationDto {
    pub operation_id: String,
    pub action: String,
    pub entity_type: String,
    pub entity_key: serde_json::Value,
    pub expected_version: i64,
    pub payload: serde_json::Value,
    pub occurred_at_local: Option<String>,
    pub created_at_millis: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingOperationsResponse {
    pub total: usize,
    pub operations: Vec<PendingOperationDto>,
}

const LIST_LIMIT: i64 = 100;

#[tauri::command]
pub fn sync_list_pending(
    connection: State<Mutex<Connection>>,
) -> Result<PendingOperationsResponse> {
    let connection = connection
        .lock()
        .map_err(|_| NightingaleError::internal("no se pudo acceder a la base local"))?;

    let operations = SqliteOperationStore.list_pending(&connection, LIST_LIMIT)?;

    Ok(PendingOperationsResponse {
        total: operations.len(),
        operations: operations
            .into_iter()
            .map(|operation| PendingOperationDto {
                operation_id: operation.operation_id.to_string(),
                action: operation.action,
                entity_type: operation.entity_type,
                entity_key: operation.entity_key,
                expected_version: operation.expected_version,
                payload: operation.payload,
                occurred_at_local: operation.occurred_at_local,
                created_at_millis: operation.created_at_millis,
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dto_serializes_to_camel_case() {
        let response = PendingOperationsResponse {
            total: 1,
            operations: vec![PendingOperationDto {
                operation_id: "019f0000-0000-7000-8000-000000000001".to_owned(),
                action: "RegisterPatient".to_owned(),
                entity_type: "patient_profile".to_owned(),
                entity_key: serde_json::json!("019f0000-0000-7000-8000-000000000101"),
                expected_version: 0,
                payload: serde_json::json!({ "fullName": "Paciente de prueba" }),
                occurred_at_local: None,
                created_at_millis: 1_800_000_000_000,
            }],
        };

        let value = serde_json::to_value(&response).unwrap();

        assert_eq!(value["total"], 1);
        assert_eq!(
            value["operations"][0]["operationId"],
            "019f0000-0000-7000-8000-000000000001"
        );
        assert_eq!(
            value["operations"][0]["createdAtMillis"],
            1_800_000_000_000i64
        );
        assert_eq!(
            value["operations"][0]["payload"]["fullName"],
            "Paciente de prueba"
        );
    }
}
