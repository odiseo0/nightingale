use crate::components::identity::{
    domain::{AccessStatus, AccountAccess},
    repository::{AccessStore, SqliteAccessStore},
};
use crate::components::sync::adapters::api::OperationRecord;
use crate::components::sync::application::{OperationStore, PendingOperation};
use crate::components::sync::dispatch::{
    DispatchLease, LEASE_MILLIS, LeasedOperation, retry_delay, storage_error,
};
use crate::components::sync::domain::OperationStatus;
use crate::platform::http::APICode;
use crate::shared_kernel::id::{AccountId, DeviceId, OperationContext, UserId};
use crate::shared_kernel::{NightingaleError, Result};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use std::collections::HashSet;

pub struct SqliteOperationStore;

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboxSummary {
    pub pending: i64,
    pub in_flight: i64,
    pub unknown: i64,
    pub retryable: i64,
    pub conflicts: i64,
    pub rejected: i64,
    pub review_required: i64,
    pub other_epoch: i64,
    pub next_attempt_at: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewSummary {
    pub operation_id: String,
    pub status: String,
    pub code: Option<APICode>,
}

/// Local intention and its receipt for review. No business projection is changed.
pub struct StoredOperationResult {
    pub operation: PendingOperation,
    pub status: OperationStatus,
    pub result: Option<serde_json::Value>,
    pub review_code: Option<APICode>,
}

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

fn available_scope<'a>(
    connection: &'a mut Connection,
    access: &AccountAccess,
    now: i64,
) -> Result<Option<Transaction<'a>>> {
    let scope = &access.context;
    // Serialize selection and marking across SQLite connections.
    // https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.transaction_with_behavior
    let tx = connection
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .map_err(storage_error)?;
    let cached =
        SqliteAccessStore.access(&tx, &scope.account_id, &scope.user_id, &scope.device_id)?;

    if access.status != AccessStatus::Active || cached.as_ref() != Some(access) {
        return Err(NightingaleError::temporary(
            "el acceso cambió antes del envío",
        ));
    }

    tx.execute(
        "UPDATE sync_outbox SET status='unknown',lease_until=NULL
            WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND status='in_flight'
            AND (lease_until IS NULL OR lease_until<=?4)",
        params![
            scope.account_id.as_str(),
            scope.user_id.as_str(),
            scope.device_id.as_str(),
            now
        ],
    )
    .map_err(storage_error)?;
    let busy: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sync_outbox
            WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND status='in_flight')",
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str()
            ],
            |row| row.get(0),
        )
        .map_err(storage_error)?;

    if busy {
        tx.commit().map_err(storage_error)?;
        return Ok(None);
    }

    Ok(Some(tx))
}

impl SqliteOperationStore {
    pub fn reviews(&self, c: &Connection, scope: &OperationContext) -> Result<Vec<ReviewSummary>> {
        let mut query=c.prepare("SELECT operation_id,status,last_error FROM sync_outbox WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND
            (status IN ('conflict','rejected') OR coalesce(last_error,'') IN ('\"NOT_FOUND\"','\"RESULT_EXPIRED\"') OR (permission_epoch!=?4 AND status IN ('pending','unknown','retryable','in_flight'))) ORDER BY created_at DESC,operation_id LIMIT 100").map_err(storage_error)?;
        query
            .query_map(
                params![
                    scope.account_id.as_str(),
                    scope.user_id.as_str(),
                    scope.device_id.as_str(),
                    scope.permission_epoch
                ],
                |r| {
                    let code = r
                        .get::<_, Option<String>>(2)?
                        .and_then(|v| serde_json::from_str::<APICode>(&v).ok());
                    Ok(ReviewSummary {
                        operation_id: r.get(0)?,
                        status: r.get(1)?,
                        code,
                    })
                },
            )
            .map_err(storage_error)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(storage_error)
    }

    /// Expire old leases, then select unknown results before any new POST.
    /// An explicit manual retry clears only the wait, never identity or review.
    pub fn prepare_cycle(
        &self,
        connection: &mut Connection,
        access: &AccountAccess,
        now: i64,
        manual: bool,
    ) -> Result<Vec<crate::shared_kernel::id::OperationId>> {
        let Some(tx) = available_scope(connection, access, now)? else {
            return Ok(vec![]);
        };
        let scope = &access.context;

        if manual {
            tx.execute("UPDATE sync_outbox SET next_attempt_at=NULL WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND permission_epoch=?4
                AND status IN ('unknown','retryable') AND action!='RegisterAccountDevice'
                AND coalesce(last_error,'') NOT IN ('\"NOT_FOUND\"','\"RESULT_EXPIRED\"')",
                params![scope.account_id.as_str(),scope.user_id.as_str(),scope.device_id.as_str(),scope.permission_epoch]).map_err(storage_error)?;
        }

        // Read the due IDs once. Use pages if queue size requires a memory bound.
        let ids = {
            let mut query=tx.prepare("SELECT operation_id FROM sync_outbox WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND permission_epoch=?4
                AND status='unknown' AND action!='RegisterAccountDevice' AND (next_attempt_at IS NULL OR next_attempt_at<=?5)
                AND coalesce(last_error,'') NOT IN ('\"NOT_FOUND\"','\"RESULT_EXPIRED\"') ORDER BY created_at,operation_id").map_err(storage_error)?;
            let rows = query
                .query_map(
                    params![
                        scope.account_id.as_str(),
                        scope.user_id.as_str(),
                        scope.device_id.as_str(),
                        scope.permission_epoch,
                        now
                    ],
                    |row| row.get::<_, String>(0),
                )
                .map_err(storage_error)?;
            let mut ids = Vec::new();

            for id in rows {
                ids.push(crate::shared_kernel::id::OperationId::try_from(
                    id.map_err(storage_error)?,
                )?);
            }

            ids
        };

        tx.commit().map_err(storage_error)?;

        Ok(ids)
    }

    /// Derives durable status without loading intentions or result payloads.
    pub fn summary(
        &self,
        connection: &Connection,
        scope: &OperationContext,
    ) -> Result<OutboxSummary> {
        connection.query_row("SELECT coalesce(sum(status='pending'),0),coalesce(sum(status='in_flight'),0),
            coalesce(sum(status='unknown'),0),coalesce(sum(status='retryable'),0),coalesce(sum(status='conflict'),0),coalesce(sum(status='rejected'),0),
            coalesce(sum(coalesce(last_error,'') IN ('\"NOT_FOUND\"','\"RESULT_EXPIRED\"')),0),
            coalesce(sum(permission_epoch!=?4 AND status IN ('pending','unknown','retryable','in_flight')),0),
            min(CASE WHEN permission_epoch=?4 AND status IN ('unknown','retryable','in_flight') AND coalesce(last_error,'') NOT IN ('\"NOT_FOUND\"','\"RESULT_EXPIRED\"') THEN next_attempt_at END)
            FROM sync_outbox WHERE account_id=?1 AND user_id=?2 AND device_id=?3",
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str(),
                scope.permission_epoch
            ], |r| Ok(
                OutboxSummary {
                    pending:r.get(0)?,
                    in_flight:r.get(1)?,
                    unknown:r.get(2)?,
                    retryable:r.get(3)?,
                    conflicts:r.get(4)?,
                    rejected:r.get(5)?,
                    review_required:r.get(6)?,
                    other_epoch:r.get(7)?,
                    next_attempt_at:r.get(8)?
                }
            )).map_err(storage_error)
    }

    pub fn operation_result(
        &self,
        connection: &Connection,
        scope: &OperationContext,
        id: &crate::shared_kernel::id::OperationId,
    ) -> Result<Option<StoredOperationResult>> {
        connection.query_row(&format!("SELECT {OPERATION_COLUMNS},status,result,last_error FROM sync_outbox
            WHERE operation_id=?1 AND account_id=?2 AND user_id=?3 AND device_id=?4 AND permission_epoch=?5"),
            params![id.as_str(),scope.account_id.as_str(),scope.user_id.as_str(),scope.device_id.as_str(),scope.permission_epoch], |row| {
                let error: Option<String> = row.get(17)?;
                let code = error.and_then(|value| serde_json::from_str::<APICode>(&value).ok());

                Ok(StoredOperationResult {
                    operation: read_operation(row)?,
                    status: serde_json::from_value(serde_json::Value::String(row.get(15)?)).map_err(|error|conversion_error(15,error))?,
                    result: row.get::<_,Option<String>>(16)?.map(|value|serde_json::from_str(&value)).transpose().map_err(|error|conversion_error(16,error))?,
                    review_code: code.filter(|code|matches!(code,APICode::NotFound|APICode::ResultExpired)),
                })
            }).optional().map_err(storage_error)
    }

    /// Normalize first; all receipts then commit together or roll back together.
    pub fn reconcile(
        &self,
        connection: &mut Connection,
        lease: &DispatchLease,
        records: &[OperationRecord],
        now: i64,
    ) -> Result<()> {
        let mut ids: HashSet<_> = lease
            .entries
            .iter()
            .map(|entry| entry.envelope.operation_id.as_str())
            .collect();

        if records.len() != lease.entries.len()
            || records
                .iter()
                .any(|record| !ids.remove(record.operation_id.as_str()))
            || !ids.is_empty()
        {
            return Err(NightingaleError::validation(
                "resultados incompletos o ajenos",
            ));
        }

        let tx = connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;

        for entry in &lease.entries {
            let record = records
                .iter()
                .find(|record| record.operation_id == entry.envelope.operation_id)
                .ok_or_else(|| NightingaleError::validation("resultado ausente"))?;
            let stored = self
                .operation_result(&tx, &lease.context, &record.operation_id)?
                .ok_or_else(|| NightingaleError::not_found("intención fuera del alcance"))?;
            let receipt = serde_json::to_value(record)
                .map_err(|_| NightingaleError::internal("recibo inválido"))?;
            let (attempt, until): (i64, Option<i64>) = tx
                .query_row(
                    "SELECT attempts,lease_until FROM sync_outbox WHERE operation_id=?1",
                    [record.operation_id.as_str()],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .map_err(storage_error)?;

            if stored.operation.envelope()? != entry.envelope
                || stored.operation.semantic_fingerprint.as_deref()
                    != Some(&entry.envelope.semantic_fingerprint()?)
            {
                return Err(NightingaleError::idempotency_conflict(
                    "la intención cambió antes del recibo",
                ));
            }

            // The first committed receipt wins. Replaying it does not write again.
            if stored.result.as_ref() == Some(&receipt)
                && attempt == entry.attempt
                && stored.status == record.local_status()
                && until.is_none()
            {
                continue;
            }

            if stored.status != OperationStatus::InFlight
                || attempt != entry.attempt
                || until != Some(lease.until)
            {
                return Err(NightingaleError::temporary(
                    "el recibo pertenece a otro intento",
                ));
            }

            let status = record.local_status();
            let next =
                if status == OperationStatus::Retryable {
                    Some(now.checked_add(entry.delay).ok_or_else(|| {
                        NightingaleError::validation("fecha de reintento inválida")
                    })?)
                } else {
                    None
                };
            let error = record
                .error
                .as_ref()
                .map(|error| serde_json::to_string(&error.code))
                .transpose()
                .map_err(|_| NightingaleError::internal("código de recibo inválido"))?;
            tx.execute("UPDATE sync_outbox SET status=?2,result=?3,last_error=?4,lease_until=NULL,next_attempt_at=?5 WHERE operation_id=?1",
                params![record.operation_id.as_str(),status.as_str(),receipt.to_string(),error,next]).map_err(storage_error)?;
        }
        tx.commit().map_err(storage_error)
    }

    /// A GET reserves only an uncertain intention; it never creates a new ID.
    pub fn lease_recovery(
        &self,
        connection: &mut Connection,
        access: &AccountAccess,
        id: &crate::shared_kernel::id::OperationId,
        now: i64,
        jitter: u64,
    ) -> Result<Option<DispatchLease>> {
        let Some(tx) = available_scope(connection, access, now)? else {
            return Ok(None);
        };
        let scope = &access.context;
        let candidate = tx.query_row(&format!("SELECT {OPERATION_COLUMNS},attempts FROM sync_outbox
            WHERE operation_id=?1 AND account_id=?2 AND user_id=?3 AND device_id=?4 AND permission_epoch=?5
            AND action!='RegisterAccountDevice' AND status IN ('unknown','retryable')
            AND (next_attempt_at IS NULL OR next_attempt_at<=?6)
            AND coalesce(last_error,'') NOT IN ('\"NOT_FOUND\"','\"RESULT_EXPIRED\"')"),
            params![
                id.as_str(),
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str(),
                scope.permission_epoch,now
            ], |row| Ok((read_operation(row)?,row.get::<_,i64>(15)?))).optional().map_err(storage_error)?;
        let Some((operation, attempts)) = candidate else {
            tx.commit().map_err(storage_error)?;
            return Ok(None);
        };

        let envelope = operation.envelope()?;

        if operation.semantic_fingerprint.as_deref() != Some(&envelope.semantic_fingerprint()?) {
            return Err(NightingaleError::idempotency_conflict(
                "huella de recuperación inválida",
            ));
        }

        let until = now
            .checked_add(LEASE_MILLIS)
            .ok_or_else(|| NightingaleError::validation("fecha de reserva inválida"))?;
        let attempt = attempts
            .checked_add(1)
            .ok_or_else(|| NightingaleError::validation("contador de intentos agotado"))?;
        let delay = retry_delay(attempt, jitter);
        let next = until
            .checked_add(delay)
            .ok_or_else(|| NightingaleError::validation("fecha de reintento inválida"))?;
        tx.execute("UPDATE sync_outbox SET status='in_flight',attempts=?2,lease_until=?3,next_attempt_at=?4 WHERE operation_id=?1",
            params![id.as_str(),attempt,until,next]).map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;

        Ok(Some(DispatchLease {
            context: scope.clone(),
            until,
            entries: vec![LeasedOperation {
                envelope,
                attempt,
                delay,
            }],
        }))
    }

    /// Reserve at most one causal batch for this installation and account.
    /// Invalid, blocked and unauthorized intentions remain unchanged for review.
    pub fn lease_batch(
        &self,
        connection: &mut Connection,
        access: &AccountAccess,
        now: i64,
        jitter: u64,
    ) -> Result<Option<DispatchLease>> {
        let scope = &access.context;
        let Some(tx) = available_scope(connection, access, now)? else {
            return Ok(None);
        };

        let candidates = {
            let mut query = tx.prepare(&format!("SELECT {OPERATION_COLUMNS}, attempts FROM sync_outbox
                WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND permission_epoch=?4
                AND status IN ('pending','unknown','retryable') AND (next_attempt_at IS NULL OR next_attempt_at<=?5)
                AND coalesce(last_error,'') NOT IN ('\"NOT_FOUND\"','\"RESULT_EXPIRED\"')
                AND action!='RegisterAccountDevice' ORDER BY created_at,operation_id")).map_err(storage_error)?;
            query
                .query_map(
                    params![
                        scope.account_id.as_str(),
                        scope.user_id.as_str(),
                        scope.device_id.as_str(),
                        scope.permission_epoch,
                        now
                    ],
                    |row| Ok((read_operation(row)?, row.get::<_, i64>(15)?)),
                )
                .map_err(storage_error)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(storage_error)?
        };
        let until = now
            .checked_add(LEASE_MILLIS)
            .ok_or_else(|| NightingaleError::validation("fecha de reserva inválida"))?;
        let mut lease = DispatchLease {
            context: scope.clone(),
            until,
            entries: vec![],
        };
        let mut selected = HashSet::new();
        let mut bytes = b"{\"operations\":[]}".len();

        // Scan again when an ancestor appears later in creation order. This is
        // bounded to 100 selections; use a paged graph if queue sizes require it.
        loop {
            let before = selected.len();

            for (operation, attempts) in &candidates {
                if selected.len() == 100 {
                    break;
                }

                if selected.contains(&operation.operation_id)
                    || !access.allowed_actions.contains(&operation.action)
                {
                    continue;
                }

                let Ok(envelope) = operation.envelope() else {
                    continue;
                };
                let Ok(encoded) = envelope.canonical_json() else {
                    continue;
                };

                if operation.semantic_fingerprint.as_deref()
                    != Some(&envelope.semantic_fingerprint()?)
                {
                    continue;
                }

                let mut ready = true;

                for dependency in &envelope.depends_on {
                    let id = uuid::Uuid::parse_str(dependency.as_str())
                        .map_err(|_| NightingaleError::validation("dependencia inválida"))?;

                    if selected.contains(&id) {
                        continue;
                    }

                    let accepted: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM sync_outbox
                        WHERE operation_id=?1 AND account_id=?2 AND user_id=?3 AND status='accepted')",
                        params![dependency.as_str(),scope.account_id.as_str(),scope.user_id.as_str()], |row| row.get(0)).map_err(storage_error)?;

                    if !accepted {
                        ready = false;
                        break;
                    }
                }

                let addition = encoded.len() + usize::from(!selected.is_empty());

                if !ready || bytes + addition > 1024 * 1024 {
                    continue;
                }

                let attempt = attempts
                    .checked_add(1)
                    .ok_or_else(|| NightingaleError::validation("contador de intentos agotado"))?;
                let delay = retry_delay(attempt, jitter ^ operation.operation_id.as_u128() as u64);
                let next = until
                    .checked_add(delay)
                    .ok_or_else(|| NightingaleError::validation("fecha de reintento inválida"))?;
                tx.execute("UPDATE sync_outbox SET status='in_flight',attempts=?2,lease_until=?3,next_attempt_at=?4
                    WHERE operation_id=?1",params![operation.operation_id.to_string(),attempt,until,next]).map_err(storage_error)?;
                bytes += addition;
                selected.insert(operation.operation_id);
                lease.entries.push(LeasedOperation {
                    envelope,
                    attempt,
                    delay,
                });
            }

            if selected.len() == before || selected.len() == 100 {
                break;
            }
        }

        tx.commit().map_err(storage_error)?;

        Ok((!lease.entries.is_empty()).then_some(lease))
    }

    /// A late response cannot release a newer attempt. Never change the intention.
    pub fn release_unknown(
        &self,
        connection: &mut Connection,
        lease: &DispatchLease,
        now: i64,
        code: APICode,
    ) -> Result<()> {
        let tx = connection.transaction().map_err(storage_error)?;
        let error = serde_json::to_string(&code)
            .map_err(|_| NightingaleError::internal("código de envío inválido"))?;

        for entry in &lease.entries {
            let next = now
                .checked_add(entry.delay)
                .ok_or_else(|| NightingaleError::validation("fecha de reintento inválida"))?;
            let next =
                (!matches!(code, APICode::NotFound | APICode::ResultExpired)).then_some(next);

            tx.execute("UPDATE sync_outbox SET status='unknown',lease_until=NULL,next_attempt_at=?4,last_error=?5
                WHERE operation_id=?1 AND status='in_flight' AND attempts=?2 AND lease_until=?3",
                params![entry.envelope.operation_id.as_str(),entry.attempt,lease.until,next,error]).map_err(storage_error)?;
        }

        tx.commit().map_err(storage_error)?;

        Ok(())
    }

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
