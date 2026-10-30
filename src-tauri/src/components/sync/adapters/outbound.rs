use crate::components::sync::application::{OperationStore, PendingOperation};
use crate::components::sync::domain::OperationStatus;
use crate::shared_kernel::id::{AccountId, DeviceId, OperationContext, UserId};
use crate::shared_kernel::{NightingaleError, Result};
use rusqlite::{Connection, OptionalExtension, Transaction, params};

pub struct SqliteOperationStore;

const OPERATION_COLUMNS: &str = "operation_id, action, entity_type, entity_key, expected_version,
    payload, occurred_at_local, created_at, contract_version, account_id, user_id, device_id,
    permission_epoch, depends_on, semantic_fingerprint";

fn conversion_error(
    index: usize,
    error: impl std::error::Error + Send + Sync + 'static,
) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(index, rusqlite::types::Type::Text, Box::new(error))
}

fn json_column<T: serde::de::DeserializeOwned>(
    row: &rusqlite::Row,
    index: usize,
) -> rusqlite::Result<T> {
    serde_json::from_str(&row.get::<_, String>(index)?)
        .map_err(|error| conversion_error(index, error))
}

fn read_operation(row: &rusqlite::Row) -> rusqlite::Result<PendingOperation> {
    let scope = (
        row.get::<_, Option<String>>(9)?,
        row.get::<_, Option<String>>(10)?,
        row.get::<_, Option<String>>(11)?,
        row.get::<_, Option<i64>>(12)?,
    );
    let context = match scope {
        (Some(account), Some(user), Some(device), Some(permission_epoch)) => {
            Some(OperationContext {
                account_id: AccountId::try_from(account)
                    .map_err(|error| conversion_error(9, error))?,
                user_id: UserId::try_from(user).map_err(|error| conversion_error(10, error))?,
                device_id: DeviceId::try_from(device)
                    .map_err(|error| conversion_error(11, error))?,
                permission_epoch,
            })
        }
        (None, None, None, None) => None,
        _ => {
            return Err(conversion_error(
                9,
                NightingaleError::internal("contexto local incompleto"),
            ));
        }
    };
    let id: String = row.get(0)?;

    Ok(PendingOperation {
        operation_id: uuid::Uuid::parse_str(&id).map_err(|error| conversion_error(0, error))?,
        action: row.get(1)?,
        entity_type: row.get(2)?,
        entity_key: json_column(row, 3)?,
        expected_version: row.get(4)?,
        payload: json_column(row, 5)?,
        occurred_at_local: row.get(6)?,
        created_at_millis: row.get(7)?,
        contract_version: row.get(8)?,
        context,
        depends_on: json_column(row, 13)?,
        semantic_fingerprint: row.get(14)?,
    })
}

impl SqliteOperationStore {
    /// Only the online registration case uses these helpers; they do not dispatch Outbox.
    pub fn find_registration(
        &self,
        connection: &Connection,
        scope: &OperationContext,
    ) -> Result<Option<(PendingOperation, OperationStatus)>> {
        connection
            .query_row(
                &format!(
                    "SELECT {OPERATION_COLUMNS}, status FROM sync_outbox
            WHERE action='RegisterAccountDevice' AND account_id=?1 AND user_id=?2 AND device_id=?3
            ORDER BY created_at DESC,operation_id DESC LIMIT 1"
                ),
                params![
                    scope.account_id.as_str(),
                    scope.user_id.as_str(),
                    scope.device_id.as_str()
                ],
                |row| {
                    let status = serde_json::from_value(serde_json::Value::String(row.get(15)?))
                        .map_err(|error| conversion_error(15, error))?;

                    Ok((read_operation(row)?, status))
                },
            )
            .optional()
            .map_err(|_| NightingaleError::internal("no se pudo leer el registro de dispositivo"))
    }

    pub fn registration_attempt(&self, tx: &Transaction, id: uuid::Uuid) -> Result<()> {
        let changed = tx.execute("UPDATE sync_outbox SET status='unknown',attempts=attempts+1
            WHERE operation_id=?1 AND action='RegisterAccountDevice' AND status IN ('pending','unknown','retryable')",[id.to_string()])
            .map_err(|_|NightingaleError::internal("no se pudo guardar el intento de registro"))?;
        if changed != 1 {
            return Err(NightingaleError::validation(
                "el registro requiere revisión",
            ));
        }

        Ok(())
    }

    pub fn registration_result(
        &self,
        tx: &Transaction,
        id: uuid::Uuid,
        status: OperationStatus,
        result: &serde_json::Value,
    ) -> Result<()> {
        if !matches!(
            status,
            OperationStatus::Accepted
                | OperationStatus::Conflict
                | OperationStatus::Rejected
                | OperationStatus::Retryable
        ) {
            return Err(NightingaleError::validation(
                "resultado de registro inválido",
            ));
        }
        let changed = tx.execute("UPDATE sync_outbox SET status=?2,result=?3 WHERE operation_id=?1 AND action='RegisterAccountDevice'",
            params![id.to_string(),status.as_str(),result.to_string()]).map_err(|_|NightingaleError::internal("no se pudo guardar el resultado del registro"))?;
        if changed != 1 {
            return Err(NightingaleError::not_found(
                "registro de dispositivo no encontrado",
            ));
        }

        Ok(())
    }
}

impl OperationStore for SqliteOperationStore {
    fn append(&self, transaction: &Transaction, operation: &PendingOperation) -> Result<()> {
        let context = operation
            .context
            .as_ref()
            .ok_or_else(|| NightingaleError::validation("una operación nueva requiere contexto"))?;
        let fingerprint = operation.envelope()?.semantic_fingerprint()?;

        if operation.semantic_fingerprint.as_deref() != Some(&fingerprint) {
            return Err(NightingaleError::validation("huella de operación inválida"));
        }

        let existing: Option<(Option<String>, bool)> = transaction
            .query_row("SELECT semantic_fingerprint, coalesce(account_id=?2 AND user_id=?3 AND device_id=?4 AND permission_epoch=?5 AND entity_type=?6, 0) FROM sync_outbox WHERE operation_id = ?1",
                params![operation.operation_id.to_string(),context.account_id.as_str(),context.user_id.as_str(),context.device_id.as_str(),context.permission_epoch,operation.entity_type],
                |row| Ok((row.get(0)?, row.get(1)?)))
            .optional().map_err(|error| NightingaleError::internal(error.to_string()))?;

        if let Some((hash, same_context)) = existing {
            if hash.as_deref() == Some(&fingerprint) && same_context {
                return Ok(());
            }

            return Err(NightingaleError::idempotency_conflict(
                "ID de operación reutilizado con otro contenido",
            ));
        }

        transaction
            .execute(
                "INSERT INTO sync_outbox (
                    operation_id, contract_version, action, entity_type, entity_key,
                    expected_version, payload, occurred_at_local, status, created_at,
                    account_id, user_id, device_id, permission_epoch, depends_on, semantic_fingerprint
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                params![
                    operation.operation_id.to_string(),
                    operation.contract_version,
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
                    context.account_id.as_str(), context.user_id.as_str(), context.device_id.as_str(), context.permission_epoch,
                    serde_json::to_string(&operation.depends_on).map_err(|error| NightingaleError::internal(error.to_string()))?,
                    fingerprint,
                ],
            )
            .map_err(|error| NightingaleError::internal(error.to_string()))?;

        Ok(())
    }

    fn list_pending(&self, connection: &Connection, limit: i64) -> Result<Vec<PendingOperation>> {
        let mut statement = connection
            .prepare(&format!(
                "SELECT {OPERATION_COLUMNS} FROM sync_outbox
            WHERE status='pending' ORDER BY created_at,operation_id LIMIT ?1"
            ))
            .map_err(|_| NightingaleError::internal("no se pudo leer el Outbox"))?;
        statement
            .query_map(params![limit], read_operation)
            .map_err(|_| NightingaleError::internal("no se pudo leer el Outbox"))?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|_| NightingaleError::internal("no se pudo leer el Outbox"))
    }
}
