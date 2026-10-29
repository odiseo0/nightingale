use crate::components::sync::application::{OperationStore, PendingOperation};
use crate::components::sync::domain::{CONTRACT_VERSION, OperationStatus};
use crate::shared_kernel::{NightingaleError, Result};
use rusqlite::{Connection, Transaction, params};

pub struct SqliteOperationStore;

impl OperationStore for SqliteOperationStore {
    fn append(&self, transaction: &Transaction, operation: &PendingOperation) -> Result<()> {
        transaction
            .execute(
                "INSERT INTO sync_outbox (
                    operation_id, contract_version, action, entity_type, entity_key,
                    expected_version, payload, occurred_at_local, status, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    operation.operation_id.to_string(),
                    CONTRACT_VERSION,
                    operation.action,
                    operation.entity_type,
                    serde_json::to_string(&operation.entity_key)
                        .map_err(|error| NightingaleError::internal(error.to_string()))?,
                    operation.expected_version,
                    serde_json::to_string(&operation.payload)
                        .map_err(|error| NightingaleError::internal(error.to_string()))?,
                    operation.occurred_at_local,
                    OperationStatus::Pending.as_str(),
                    operation.created_at_millis,
                ],
            )
            .map_err(|error| NightingaleError::internal(error.to_string()))?;

        Ok(())
    }

    fn list_pending(&self, connection: &Connection, limit: i64) -> Result<Vec<PendingOperation>> {
        let mut statement = connection
            .prepare(
                "SELECT operation_id, action, entity_type, entity_key, expected_version,
                        payload, occurred_at_local, created_at
                 FROM sync_outbox
                 WHERE status = 'pending'
                 ORDER BY created_at
                 LIMIT ?1",
            )
            .map_err(|error| NightingaleError::internal(error.to_string()))?;

        let rows = statement
            .query_map(params![limit], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, i64>(7)?,
                ))
            })
            .map_err(|error| NightingaleError::internal(error.to_string()))?;

        let mut operations = Vec::new();

        for row in rows {
            let (
                id,
                action,
                entity_type,
                entity_key,
                expected_version,
                payload,
                occurred_at,
                created_at,
            ) = row.map_err(|error| NightingaleError::internal(error.to_string()))?;
            let operation_id = uuid::Uuid::parse_str(&id)
                .map_err(|error| NightingaleError::internal(error.to_string()))?;

            operations.push(PendingOperation {
                operation_id,
                action,
                entity_type,
                entity_key: serde_json::from_str(&entity_key)
                    .map_err(|error| NightingaleError::internal(error.to_string()))?,
                expected_version,
                payload: serde_json::from_str(&payload)
                    .map_err(|error| NightingaleError::internal(error.to_string()))?,
                occurred_at_local: occurred_at,
                created_at_millis: created_at,
            });
        }

        Ok(operations)
    }
}
