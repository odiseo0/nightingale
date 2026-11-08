//! SQLite scheduling storage. Accepted views and local intentions have distinct writers.
use super::super::{
    application as app,
    domain::{self as dom, SchedulingError},
};
use crate::{
    components::{
        account::application::AccountViewQuery,
        appointments::application::ScheduleOccupancyQuery,
        identity::{
            domain::AccountAccess,
            repository::{AccessStore, SqliteAccessStore},
        },
        services::application::CatalogQuery,
        sync::{
            adapters::outbound::SqliteOperationStore, application::record_operation,
            domain::NewOperation,
        },
    },
    shared_kernel::{
        Clock, IdGenerator,
        id::{OperationContext, OperationId},
    },
};
use rusqlite::{Connection, Transaction, params};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[path = "intents.rs"]
mod intents;

pub(super) fn storage_error(_: rusqlite::Error) -> SchedulingError {
    SchedulingError::internal("datos locales de agenda no disponibles")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleRecord {
    pub rule_id: String,
    pub professional_id: String,
    pub iso_weekday: i64,
    pub local_start_minute: i64,
    pub local_end_minute: i64,
    pub valid_from: String,
    pub valid_until: Option<String>,
    pub status: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExceptionRecord {
    pub exception_id: String,
    pub professional_id: String,
    pub local_date: String,
    pub kind: String,
    pub local_start_minute: Option<i64>,
    pub local_end_minute: Option<i64>,
    pub status: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockRecord {
    pub block_id: String,
    pub professional_id: String,
    pub block_account_id: Option<String>,
    pub starts_at: String,
    pub ends_at: String,
    pub reason_code: Option<String>,
    pub status: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalChange {
    pub operation_id: String,
    pub entity_type: String,
    pub rule_id: Option<String>,
    pub exception_id: Option<String>,
    pub block_id: Option<String>,
    pub action: String,
    pub expected_version: i64,
    pub base_operation_id: Option<String>,
    pub depends_on: String,
    pub payload: String,
    pub draft: String,
    pub created_at: i64,
}

pub fn read_rules(c: &Connection, s: &OperationContext) -> rusqlite::Result<Vec<RuleRecord>> {
    let mut q = c.prepare("SELECT rule_id,professional_id,iso_weekday,local_start_minute,local_end_minute,valid_from,valid_until,status,created_at,updated_at,version FROM schedule_rule_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 ORDER BY rule_id")?;
    q.query_map(
        params![
            s.account_id.as_str(),
            s.user_id.as_str(),
            s.device_id.as_str()
        ],
        |r| {
            Ok(RuleRecord {
                rule_id: r.get(0)?,
                professional_id: r.get(1)?,
                iso_weekday: r.get(2)?,
                local_start_minute: r.get(3)?,
                local_end_minute: r.get(4)?,
                valid_from: r.get(5)?,
                valid_until: r.get(6)?,
                status: r.get(7)?,
                created_at: r.get(8)?,
                updated_at: r.get(9)?,
                version: r.get(10)?,
            })
        },
    )?
    .collect()
}

pub fn save_rule(
    tx: &Transaction,
    s: &OperationContext,
    record: &RuleRecord,
) -> rusqlite::Result<()> {
    tx.execute(
        "INSERT INTO schedule_rule_view (account_id,user_id,device_id,rule_id,professional_id,iso_weekday,local_start_minute,local_end_minute,valid_from,valid_until,status,created_at,updated_at,version) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)
         ON CONFLICT(account_id,user_id,device_id,rule_id) DO UPDATE SET professional_id=excluded.professional_id,iso_weekday=excluded.iso_weekday,local_start_minute=excluded.local_start_minute,local_end_minute=excluded.local_end_minute,valid_from=excluded.valid_from,valid_until=excluded.valid_until,status=excluded.status,created_at=excluded.created_at,updated_at=excluded.updated_at,version=excluded.version",
        params![
            s.account_id.as_str(), s.user_id.as_str(), s.device_id.as_str(),
            record.rule_id, record.professional_id, record.iso_weekday,
            record.local_start_minute, record.local_end_minute, record.valid_from,
            record.valid_until, record.status, record.created_at, record.updated_at,
            record.version
        ],
    )?;

    Ok(())
}

pub fn read_exceptions(
    c: &Connection,
    s: &OperationContext,
) -> rusqlite::Result<Vec<ExceptionRecord>> {
    let mut q = c.prepare("SELECT exception_id,professional_id,local_date,kind,local_start_minute,local_end_minute,status,created_at,updated_at,version FROM schedule_exception_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 ORDER BY exception_id")?;
    q.query_map(
        params![
            s.account_id.as_str(),
            s.user_id.as_str(),
            s.device_id.as_str()
        ],
        |r| {
            Ok(ExceptionRecord {
                exception_id: r.get(0)?,
                professional_id: r.get(1)?,
                local_date: r.get(2)?,
                kind: r.get(3)?,
                local_start_minute: r.get(4)?,
                local_end_minute: r.get(5)?,
                status: r.get(6)?,
                created_at: r.get(7)?,
                updated_at: r.get(8)?,
                version: r.get(9)?,
            })
        },
    )?
    .collect()
}

pub fn save_exception(
    tx: &Transaction,
    s: &OperationContext,
    record: &ExceptionRecord,
) -> rusqlite::Result<()> {
    tx.execute(
        "INSERT INTO schedule_exception_view (account_id,user_id,device_id,exception_id,professional_id,local_date,kind,local_start_minute,local_end_minute,status,created_at,updated_at,version) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
         ON CONFLICT(account_id,user_id,device_id,exception_id) DO UPDATE SET professional_id=excluded.professional_id,local_date=excluded.local_date,kind=excluded.kind,local_start_minute=excluded.local_start_minute,local_end_minute=excluded.local_end_minute,status=excluded.status,created_at=excluded.created_at,updated_at=excluded.updated_at,version=excluded.version",
        params![
            s.account_id.as_str(), s.user_id.as_str(), s.device_id.as_str(),
            record.exception_id, record.professional_id, record.local_date, record.kind,
            record.local_start_minute, record.local_end_minute, record.status,
            record.created_at, record.updated_at, record.version
        ],
    )?;

    Ok(())
}

pub fn read_blocks(c: &Connection, s: &OperationContext) -> rusqlite::Result<Vec<BlockRecord>> {
    let mut q = c.prepare("SELECT block_id,professional_id,block_account_id,starts_at,ends_at,reason_code,status,created_at,updated_at,version FROM schedule_block_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 ORDER BY block_id")?;
    q.query_map(
        params![
            s.account_id.as_str(),
            s.user_id.as_str(),
            s.device_id.as_str()
        ],
        |r| {
            Ok(BlockRecord {
                block_id: r.get(0)?,
                professional_id: r.get(1)?,
                block_account_id: r.get(2)?,
                starts_at: r.get(3)?,
                ends_at: r.get(4)?,
                reason_code: r.get(5)?,
                status: r.get(6)?,
                created_at: r.get(7)?,
                updated_at: r.get(8)?,
                version: r.get(9)?,
            })
        },
    )?
    .collect()
}

pub fn save_block(
    tx: &Transaction,
    s: &OperationContext,
    record: &BlockRecord,
) -> rusqlite::Result<()> {
    tx.execute(
        "INSERT INTO schedule_block_view (account_id,user_id,device_id,block_id,professional_id,block_account_id,starts_at,ends_at,reason_code,status,created_at,updated_at,version) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)
         ON CONFLICT(account_id,user_id,device_id,block_id) DO UPDATE SET professional_id=excluded.professional_id,block_account_id=excluded.block_account_id,starts_at=excluded.starts_at,ends_at=excluded.ends_at,reason_code=excluded.reason_code,status=excluded.status,created_at=excluded.created_at,updated_at=excluded.updated_at,version=excluded.version",
        params![
            s.account_id.as_str(), s.user_id.as_str(), s.device_id.as_str(),
            record.block_id, record.professional_id, record.block_account_id,
            record.starts_at, record.ends_at, record.reason_code, record.status,
            record.created_at, record.updated_at, record.version
        ],
    )?;

    Ok(())
}

pub fn read_local_changes(
    c: &Connection,
    s: &OperationContext,
) -> rusqlite::Result<Vec<LocalChange>> {
    let mut q = c.prepare("SELECT operation_id,entity_type,rule_id,exception_id,block_id,action,expected_version,base_operation_id,depends_on,payload,draft,created_at FROM schedule_local WHERE account_id=?1 AND user_id=?2 AND device_id=?3 ORDER BY created_at,operation_id")?;
    q.query_map(
        params![
            s.account_id.as_str(),
            s.user_id.as_str(),
            s.device_id.as_str()
        ],
        |r| {
            Ok(LocalChange {
                operation_id: r.get(0)?,
                entity_type: r.get(1)?,
                rule_id: r.get(2)?,
                exception_id: r.get(3)?,
                block_id: r.get(4)?,
                action: r.get(5)?,
                expected_version: r.get(6)?,
                base_operation_id: r.get(7)?,
                depends_on: r.get(8)?,
                payload: r.get(9)?,
                draft: r.get(10)?,
                created_at: r.get(11)?,
            })
        },
    )?
    .collect()
}

pub fn insert_local_change(
    tx: &Transaction,
    s: &OperationContext,
    change: &LocalChange,
) -> rusqlite::Result<()> {
    tx.execute(
        "INSERT INTO schedule_local (account_id,user_id,device_id,operation_id,entity_type,rule_id,exception_id,block_id,action,expected_version,base_operation_id,depends_on,payload,draft,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
        params![
            s.account_id.as_str(), s.user_id.as_str(), s.device_id.as_str(),
            change.operation_id, change.entity_type, change.rule_id, change.exception_id,
            change.block_id, change.action, change.expected_version, change.base_operation_id,
            change.depends_on, change.payload, change.draft, change.created_at
        ],
    )?;

    Ok(())
}

fn missing() -> SchedulingError {
    SchedulingError::new("NOT_FOUND", "regla no disponible en este ámbito")
}

fn current_access(c: &Connection, access: &AccountAccess, action: Option<&str>) -> app::Result<()> {
    let s = &access.context;
    let current = SqliteAccessStore
        .access(c, &s.account_id, &s.user_id, &s.device_id)?
        .ok_or_else(|| SchedulingError::new("PERMISSION_DENIED", "no hay permiso local"))?;
    app::require_read(&current)?;

    if current.context.permission_epoch != s.permission_epoch {
        return Err(SchedulingError::new(
            "PERMISSION_DENIED",
            "los permisos locales cambiaron",
        ));
    }

    if let Some(action) = action {
        app::require_write(&current, action)?;
    }

    Ok(())
}

fn operation_view(
    c: &Connection,
    s: &OperationContext,
    local: &LocalChange,
) -> app::Result<app::OperationView> {
    let result = SqliteOperationStore.operation_result(
        c,
        s,
        &OperationId::try_from(local.operation_id.clone())?,
    )?;
    let mut view = app::OperationView {
        operation_id: local.operation_id.clone(),
        depends_on: serde_json::from_str(&local.depends_on)
            .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?,
        status: "review_required".into(),
        error: None,
    };

    if let Some(result) = result {
        view.status = if result.review_code.is_some() {
            "review_required"
        } else {
            result.status.as_str()
        }
        .into();

        view.error = result
            .result
            .as_ref()
            .and_then(|v| v.get("error"))
            .and_then(|e| e.get("code"))
            .and_then(Value::as_str)
            .map(str::to_owned);
    }

    Ok(view)
}

fn rule_rows(
    c: &Connection,
    s: &OperationContext,
) -> app::Result<Vec<app::EntityRow<app::RuleState>>> {
    let mut rows: BTreeMap<String, app::EntityRow<app::RuleState>> = BTreeMap::new();

    for record in read_rules(c, s).map_err(storage_error)? {
        let rule_id = record.rule_id.clone();
        let version = record.version;
        let state = app::RuleState {
            rule_id: record.rule_id,
            professional_id: record.professional_id,
            iso_weekday: record.iso_weekday,
            local_start_minute: record.local_start_minute,
            local_end_minute: record.local_end_minute,
            valid_from: record.valid_from,
            valid_until: record.valid_until,
            status: record.status,
        };
        rows.entry(rule_id)
            .or_insert_with(|| app::EntityRow {
                accepted: None,
                provisional: None,
                operations: vec![],
            })
            .accepted = Some(app::Accepted { state, version });
    }

    for change in intents::rule_changes(c, s)? {
        let rule_id = change.rule_id.clone().ok_or_else(|| {
            SchedulingError::new("INTERNAL_ERROR", "intención de regla sin identificador")
        })?;
        let view = operation_view(c, s, &change)?;
        let row = rows
            .entry(rule_id.clone())
            .or_insert_with(|| app::EntityRow {
                accepted: None,
                provisional: None,
                operations: vec![],
            });
        row.operations.push(view);

        let state: app::RuleState = serde_json::from_str(&change.draft)
            .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?;
        let candidate = app::Provisional {
            state,
            operation_id: change.operation_id.clone(),
            expected_version: change.expected_version,
        };
        let replace = row.provisional.as_ref().is_none_or(|p| {
            (candidate.expected_version, &candidate.operation_id)
                >= (p.expected_version, &p.operation_id)
        });

        if replace {
            row.provisional = Some(candidate);
        }
    }

    Ok(rows.into_values().collect())
}

pub struct SqliteSchedulingRepository<'a> {
    pub connection: &'a mut Connection,
}

fn exception_rows(
    c: &Connection,
    s: &OperationContext,
) -> app::Result<Vec<app::EntityRow<app::ExceptionState>>> {
    let mut rows: BTreeMap<String, app::EntityRow<app::ExceptionState>> = BTreeMap::new();

    for record in read_exceptions(c, s).map_err(storage_error)? {
        let exception_id = record.exception_id.clone();
        let version = record.version;
        let state = app::ExceptionState {
            exception_id: record.exception_id,
            professional_id: record.professional_id,
            local_date: record.local_date,
            kind: record.kind,
            local_start_minute: record.local_start_minute,
            local_end_minute: record.local_end_minute,
            status: record.status,
        };
        rows.entry(exception_id)
            .or_insert_with(|| app::EntityRow {
                accepted: None,
                provisional: None,
                operations: vec![],
            })
            .accepted = Some(app::Accepted { state, version });
    }

    for change in intents::exception_changes(c, s)? {
        let exception_id = change.exception_id.clone().ok_or_else(|| {
            SchedulingError::new("INTERNAL_ERROR", "intención de excepción sin identificador")
        })?;
        let view = operation_view(c, s, &change)?;
        let row = rows
            .entry(exception_id.clone())
            .or_insert_with(|| app::EntityRow {
                accepted: None,
                provisional: None,
                operations: vec![],
            });
        row.operations.push(view);

        let state: app::ExceptionState = serde_json::from_str(&change.draft)
            .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?;
        let candidate = app::Provisional {
            state,
            operation_id: change.operation_id.clone(),
            expected_version: change.expected_version,
        };
        let replace = row.provisional.as_ref().is_none_or(|p| {
            (candidate.expected_version, &candidate.operation_id)
                >= (p.expected_version, &p.operation_id)
        });

        if replace {
            row.provisional = Some(candidate);
        }
    }

    Ok(rows.into_values().collect())
}

impl app::SchedulingRepository for SqliteSchedulingRepository<'_> {
    fn persist(
        &mut self,
        access: &AccountAccess,
        mutation: app::SchedulingMutation,
        ids: &impl IdGenerator,
        clock: &impl Clock,
        occupancy: &impl ScheduleOccupancyQuery,
    ) -> app::Result<app::LocalMutationResult> {
        app::require_write(access, mutation.action())?;
        let time_zone = account_time_zone(self.connection, access)?;
        let own_professional = match &mutation {
            app::SchedulingMutation::CreateBlock { value } if value.account_id.is_none() => {
                let professionals =
                    crate::components::services::adapters::outbound::SqliteCatalogRepository {
                        connection: &mut *self.connection,
                    }
                    .list_professionals(&access.context)
                    .map_err(|error| SchedulingError::new(&error.code, &error.message))?;
                professionals.iter().any(|professional| {
                    professional.professional_id == value.professional_id.as_str()
                        && professional.user_id == access.context.user_id.as_str()
                })
            }
            _ => false,
        };
        // Reserve the write lock before checking permissions and preparing the change.
        // https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.transaction_with_behavior
        let tx = self
            .connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        current_access(&tx, access, Some(mutation.action()))?;
        let action = mutation.action();
        let now = clock
            .now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .and_then(|v| i64::try_from(v.as_millis()).ok())
            .ok_or_else(|| SchedulingError::internal("reloj local inválido"))?;
        let prepared = intents::prepare(
            &tx,
            &access.context,
            access,
            own_professional,
            mutation,
            ids,
        )?;
        let candidate_state: CandidateState = serde_json::from_value(prepared.draft.clone())
            .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?;
        ensure_known_occupancy_fits(
            &tx,
            &access.context,
            &time_zone,
            &candidate_state,
            occupancy,
        )?;
        let dependencies: Vec<String> = prepared
            .base
            .depends_on
            .iter()
            .map(|id| id.as_str().into())
            .collect();
        let operation_id = record_operation(
            &tx,
            &SqliteOperationStore,
            ids,
            clock,
            NewOperation {
                context: access.context.clone(),
                depends_on: prepared.base.depends_on.clone(),
                action: action.into(),
                entity_type: prepared.entity_type.into(),
                entity_key: prepared.key.clone(),
                expected_version: prepared.base.version,
                payload: prepared.payload.clone(),
                occurred_at_local: None,
            },
        )
        .map_err(SchedulingError::from)?
        .to_string();
        insert_local_change(
            &tx,
            &access.context,
            &LocalChange {
                operation_id: operation_id.clone(),
                entity_type: prepared.entity_type.into(),
                rule_id: prepared.rule_id,
                exception_id: prepared.exception_id,
                block_id: prepared.block_id,
                action: action.into(),
                expected_version: prepared.base.version,
                base_operation_id: prepared.base.operation_id,
                depends_on: json!(dependencies).to_string(),
                payload: prepared.payload.to_string(),
                draft: prepared.draft.to_string(),
                created_at: now,
            },
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;

        Ok(app::LocalMutationResult {
            operation_id,
            entity_type: prepared.entity_type.into(),
            entity_key: prepared.key,
            expected_version: prepared.base.version,
            depends_on: dependencies,
            provisional: prepared.draft,
        })
    }

    fn rule_row(
        &mut self,
        access: &AccountAccess,
        rule_id: &str,
    ) -> app::Result<app::EntityRow<app::RuleState>> {
        let tx = self.connection.transaction().map_err(storage_error)?;
        current_access(&tx, access, None)?;
        let row = rule_rows(&tx, &access.context)?
            .into_iter()
            .find(|r| r.visible().is_some_and(|v| v.rule_id == rule_id))
            .ok_or_else(missing)?;
        tx.commit().map_err(storage_error)?;

        Ok(row)
    }

    fn rule_rows_list(
        &mut self,
        access: &AccountAccess,
    ) -> app::Result<Vec<app::EntityRow<app::RuleState>>> {
        let tx = self.connection.transaction().map_err(storage_error)?;
        current_access(&tx, access, None)?;
        let rows = rule_rows(&tx, &access.context)?;
        tx.commit().map_err(storage_error)?;

        Ok(rows)
    }

    fn exception_rows_list(
        &mut self,
        access: &AccountAccess,
    ) -> app::Result<Vec<app::EntityRow<app::ExceptionState>>> {
        let tx = self.connection.transaction().map_err(storage_error)?;
        current_access(&tx, access, None)?;
        let rows = exception_rows(&tx, &access.context)?;
        tx.commit().map_err(storage_error)?;

        Ok(rows)
    }

    fn block_rows_list(
        &mut self,
        access: &AccountAccess,
    ) -> app::Result<Vec<app::EntityRow<app::BlockState>>> {
        let tx = self.connection.transaction().map_err(storage_error)?;
        current_access(&tx, access, None)?;
        let rows = block_rows(&tx, &access.context)?;
        tx.commit().map_err(storage_error)?;

        Ok(rows)
    }
}

fn account_time_zone(
    c: &mut Connection,
    access: &AccountAccess,
) -> app::Result<jiff::tz::TimeZone> {
    let view = crate::components::account::adapters::outbound::SqliteAccountRepository {
        connection: c,
        clock: &crate::shared_kernel::SystemClock,
    }
    .account_view(&access.context)
    .map_err(|error| SchedulingError::new(&error.code, &error.message))?
    .ok_or_else(|| SchedulingError::new("NOT_FOUND", "cuenta no disponible en este ámbito"))?;

    jiff::tz::TimeZone::get(&view.time_zone)
        .map_err(|_| SchedulingError::validation("zona horaria de la cuenta inválida"))
}

fn block_rows(
    c: &Connection,
    s: &OperationContext,
) -> app::Result<Vec<app::EntityRow<app::BlockState>>> {
    let mut rows: BTreeMap<String, app::EntityRow<app::BlockState>> = BTreeMap::new();

    for record in read_blocks(c, s).map_err(storage_error)? {
        let block_id = record.block_id.clone();
        let version = record.version;
        let state = app::BlockState {
            block_id: record.block_id,
            professional_id: record.professional_id,
            block_account_id: record.block_account_id,
            starts_at: record.starts_at,
            ends_at: record.ends_at,
            reason_code: record.reason_code,
            status: record.status,
        };
        rows.entry(block_id)
            .or_insert_with(|| app::EntityRow {
                accepted: None,
                provisional: None,
                operations: vec![],
            })
            .accepted = Some(app::Accepted { state, version });
    }

    for change in intents::block_changes(c, s)? {
        let block_id = change.block_id.clone().ok_or_else(|| {
            SchedulingError::new("INTERNAL_ERROR", "intención de bloqueo sin identificador")
        })?;
        let view = operation_view(c, s, &change)?;
        let row = rows
            .entry(block_id.clone())
            .or_insert_with(|| app::EntityRow {
                accepted: None,
                provisional: None,
                operations: vec![],
            });
        row.operations.push(view);

        let state: app::BlockState = serde_json::from_str(&change.draft)
            .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?;
        let candidate = app::Provisional {
            state,
            operation_id: change.operation_id.clone(),
            expected_version: change.expected_version,
        };
        let replace = row.provisional.as_ref().is_none_or(|p| {
            (candidate.expected_version, &candidate.operation_id)
                >= (p.expected_version, &p.operation_id)
        });

        if replace {
            row.provisional = Some(candidate);
        }
    }

    Ok(rows.into_values().collect())
}

/// A write may shrink the schedule; known accepted occupancy must still fit the
/// fused visible agenda with the candidate applied. Unknown (not downloaded)
/// occupancy never proves safety, but API revalidates regardless.
fn ensure_known_occupancy_fits(
    c: &Connection,
    s: &OperationContext,
    zone: &jiff::tz::TimeZone,
    candidate: &CandidateState,
    occupancy: &impl ScheduleOccupancyQuery,
) -> app::Result<()> {
    let (professional, candidate_id) = match candidate {
        CandidateState::Rule(state) => (&state.professional_id, state.rule_id.as_str()),
        CandidateState::Exception(state) => (&state.professional_id, state.exception_id.as_str()),
        CandidateState::Block(state) => (&state.professional_id, state.block_id.as_str()),
    };

    let mut rules: Vec<app::RuleState> = rule_rows(c, s)?
        .into_iter()
        .filter_map(|row| row.visible().cloned())
        .filter(|state| state.rule_id != candidate_id)
        .collect();
    let mut exceptions: Vec<app::ExceptionState> = exception_rows(c, s)?
        .into_iter()
        .filter_map(|row| row.visible().cloned())
        .filter(|state| state.professional_id == *professional)
        .filter(|state| state.exception_id != candidate_id)
        .collect();

    let mut blocks: Vec<app::BlockState> = block_rows(c, s)?
        .into_iter()
        .filter_map(|row| row.visible().cloned())
        .filter(|state| state.professional_id == *professional)
        .filter(|state| state.block_id != candidate_id)
        .collect();

    match candidate {
        CandidateState::Rule(state) => rules.push(state.clone()),
        CandidateState::Exception(state) => exceptions.push(state.clone()),
        CandidateState::Block(state) => blocks.push(state.clone()),
    }

    for interval in
        occupancy.list_occupancy(professional, jiff::Timestamp::MIN, jiff::Timestamp::MAX)?
    {
        if !app::visible_schedule_covers(zone, &rules, &exceptions, &interval)
            || app::intersects_active_block(&blocks, &interval)
        {
            return Err(SchedulingError::new(
                "SLOT_UNAVAILABLE",
                "el cambio deja una cita conocida fuera del horario o dentro de un bloqueo",
            ));
        }
    }

    Ok(())
}

/// The candidate provisional state carried inside a prepared mutation.
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum CandidateState {
    Rule(app::RuleState),
    Exception(app::ExceptionState),
    Block(app::BlockState),
}
