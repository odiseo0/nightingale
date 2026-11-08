//! SQLite read access to the accepted appointment occupancy projection.
use super::super::application::{OccupancyInterval, ScheduleOccupancyQuery};
use super::super::domain::Snapshots;
use crate::shared_kernel::{NightingaleError, Result, id::OperationContext};
use jiff::Timestamp;
use rusqlite::{Connection, Transaction, params};
use std::str::FromStr;

fn storage_error(_: rusqlite::Error) -> NightingaleError {
    NightingaleError::internal("datos locales de ocupación no disponibles")
}

/// Scope-bound read-only occupancy query. It exposes no SQL to consumers and
/// never writes: intentions and accepted views have distinct writers.
pub struct SqliteScheduleOccupancyQuery<'a> {
    pub connection: &'a Connection,
    pub scope: &'a OperationContext,
}

impl ScheduleOccupancyQuery for SqliteScheduleOccupancyQuery<'_> {
    fn list_occupancy(
        &self,
        professional_id: &str,
        from: Timestamp,
        until: Timestamp,
    ) -> Result<Vec<OccupancyInterval>> {
        let mut intervals = self.list_accepted_occupancy(professional_id, from, until)?;
        self.add_provisional_occupancy(professional_id, from, until, &mut intervals)?;

        Ok(intervals)
    }
}

impl SqliteScheduleOccupancyQuery<'_> {
    fn list_accepted_occupancy(
        &self,
        professional_id: &str,
        from: Timestamp,
        until: Timestamp,
    ) -> Result<Vec<OccupancyInterval>> {
        let mut query = self
            .connection
            .prepare(
                "SELECT appointment_id,starts_at,ends_at,status FROM appointment_occupancy_view \
                 WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND professional_id=?4 \
                 AND starts_at < ?5 AND ends_at > ?6 ORDER BY starts_at,appointment_id",
            )
            .map_err(storage_error)?;
        let rows = query
            .query_map(
                params![
                    self.scope.account_id.as_str(),
                    self.scope.user_id.as_str(),
                    self.scope.device_id.as_str(),
                    professional_id,
                    until.to_string(),
                    from.to_string()
                ],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .map_err(storage_error)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(storage_error)?;

        rows.iter()
            .map(|(id, starts, ends, status)| {
                let starts_at = Timestamp::from_str(starts).map_err(|_| {
                    NightingaleError::internal("ocupación local con instante inválido")
                })?;
                let ends_at = Timestamp::from_str(ends).map_err(|_| {
                    NightingaleError::internal("ocupación local con instante inválido")
                })?;

                OccupancyInterval::new(id.clone(), starts_at, ends_at, status)
            })
            .collect()
    }

    /// Provisional occupancy from local appointment intentions: a pending
    /// appointment blocks locally and a provisional cancellation frees the
    /// slot. Intentions whose operations are rejected or in conflict cannot
    /// anchor anything and are excluded; the accepted projection wins when
    /// both exist.
    fn add_provisional_occupancy(
        &self,
        professional_id: &str,
        from: Timestamp,
        until: Timestamp,
        accepted: &mut Vec<OccupancyInterval>,
    ) -> Result<()> {
        use std::collections::BTreeMap;

        let store = crate::components::sync::adapters::outbound::SqliteOperationStore;

        let mut by_appointment: BTreeMap<String, Vec<LocalChange>> = BTreeMap::new();

        for change in read_local_changes(self.connection, self.scope).map_err(storage_error)? {
            by_appointment
                .entry(change.appointment_id.clone())
                .or_default()
                .push(change);
        }

        for (appointment_id, changes) in by_appointment {
            if accepted
                .iter()
                .any(|interval| interval.appointment_id == appointment_id)
            {
                continue;
            }

            let mut blocked = false;

            for change in &changes {
                let result = store
                    .operation_result(
                        self.connection,
                        self.scope,
                        &crate::shared_kernel::id::OperationId::try_from(
                            change.operation_id.clone(),
                        )
                        .map_err(|_| NightingaleError::internal("operación local inválida"))?,
                    )
                    .map_err(|_| {
                        NightingaleError::internal("resultado de operación no disponible")
                    })?;

                if let Some(result) = result {
                    let status = result.status.as_str();
                    let review = result
                        .review_code
                        .map(|code| serde_json::to_string(&code).unwrap_or_default());

                    if status == "conflict" || status == "rejected" || review.is_some() {
                        blocked = true;
                        break;
                    }
                }
            }

            if blocked {
                continue;
            }

            // The latest draft at the local level defines the provisional status.
            let Some(latest) = changes.last() else {
                continue;
            };
            let Ok(draft) = serde_json::from_str::<serde_json::Value>(&latest.draft) else {
                continue;
            };
            let (Some(professional), Some(starts), Some(ends), Some(status)) = (
                draft
                    .get("professionalId")
                    .and_then(serde_json::Value::as_str),
                draft.get("startsAt").and_then(serde_json::Value::as_str),
                draft.get("endsAt").and_then(serde_json::Value::as_str),
                draft.get("status").and_then(serde_json::Value::as_str),
            ) else {
                continue;
            };

            if professional != professional_id {
                continue;
            }

            let Ok(starts_at) = Timestamp::from_str(starts) else {
                continue;
            };
            let Ok(ends_at) = Timestamp::from_str(ends) else {
                continue;
            };

            if starts_at >= until || ends_at <= from {
                continue;
            }

            // OccupancyInterval rejects unoccupied states:
            // A provisional cancellation frees up the unoccupied slot.
            if let Ok(interval) = OccupancyInterval::new(appointment_id, starts_at, ends_at, status)
            {
                accepted.push(interval);
            }
        }

        accepted.sort_by(|a, b| {
            a.starts_at
                .cmp(&b.starts_at)
                .then_with(|| a.appointment_id.cmp(&b.appointment_id))
        });

        Ok(())
    }
}

/// Accepted appointment row of `appointment_view`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppointmentRecord {
    pub appointment_id: String,
    pub patient_id: String,
    pub professional_id: String,
    pub service_id: String,
    pub responsible_contact_id: Option<String>,
    pub starts_at: String,
    pub ends_at: String,
    pub status: String,
    pub confirmed_at: Option<String>,
    pub snapshots: Snapshots,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub version: i64,
}

/// One accepted history entry of `appointment_history_view`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppointmentHistoryRecord {
    pub appointment_id: String,
    pub version: i64,
    pub action: String,
    pub actor_kind: String,
    pub actor_user_id: Option<String>,
    pub actor_contact_person_id: Option<String>,
    pub change_reason: Option<String>,
    pub changed_at: String,
    pub starts_at: String,
    pub ends_at: String,
    pub status: String,
    pub confirmed_at: Option<String>,
    pub snapshots: Snapshots,
}

/// A local appointment intention of `appointment_local`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalChange {
    pub operation_id: String,
    pub appointment_id: String,
    pub action: String,
    pub expected_version: i64,
    pub base_operation_id: Option<String>,
    pub depends_on: String,
    pub payload: String,
    pub draft: String,
    pub created_at: i64,
}

const SNAPSHOT_COLUMNS: &str = "service_name_snapshot,offered_duration_minutes_snapshot,professional_name_snapshot,location_name_snapshot,location_address_snapshot,location_country_snapshot,location_state_snapshot,location_city_snapshot,time_zone_snapshot";

fn snapshots_from(row: &rusqlite::Row<'_>, offset: usize) -> rusqlite::Result<Snapshots> {
    Ok(Snapshots {
        service_name: row.get(offset)?,
        offered_duration_minutes: row.get(offset + 1)?,
        professional_name: row.get(offset + 2)?,
        location_name: row.get(offset + 3)?,
        location_address: row.get(offset + 4)?,
        location_country: row.get(offset + 5)?,
        location_state: row.get(offset + 6)?,
        location_city: row.get(offset + 7)?,
        time_zone: row.get(offset + 8)?,
    })
}

pub fn save_appointment(
    tx: &Transaction,
    scope: &OperationContext,
    record: &AppointmentRecord,
) -> rusqlite::Result<()> {
    tx.execute(
        &format!("INSERT INTO appointment_view (account_id,user_id,device_id,appointment_id,patient_id,professional_id,service_id,responsible_contact_id,starts_at,ends_at,status,confirmed_at,{SNAPSHOT_COLUMNS},created_at,updated_at,version) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24) ON CONFLICT(account_id,user_id,device_id,appointment_id) DO UPDATE SET patient_id=?5,professional_id=?6,service_id=?7,responsible_contact_id=?8,starts_at=?9,ends_at=?10,status=?11,confirmed_at=?12,service_name_snapshot=?13,offered_duration_minutes_snapshot=?14,professional_name_snapshot=?15,location_name_snapshot=?16,location_address_snapshot=?17,location_country_snapshot=?18,location_state_snapshot=?19,location_city_snapshot=?20,time_zone_snapshot=?21,created_at=?22,updated_at=?23,version=?24"),
        rusqlite::params![
            scope.account_id.as_str(),
            scope.user_id.as_str(),
            scope.device_id.as_str(),
            record.appointment_id,
            record.patient_id,
            record.professional_id,
            record.service_id,
            record.responsible_contact_id,
            record.starts_at,
            record.ends_at,
            record.status,
            record.confirmed_at,
            record.snapshots.service_name,
            record.snapshots.offered_duration_minutes,
            record.snapshots.professional_name,
            record.snapshots.location_name,
            record.snapshots.location_address,
            record.snapshots.location_country,
            record.snapshots.location_state,
            record.snapshots.location_city,
            record.snapshots.time_zone,
            record.created_at,
            record.updated_at,
            record.version,
        ],
    )?;

    Ok(())
}

pub fn read_appointments(
    connection: &Connection,
    scope: &OperationContext,
) -> rusqlite::Result<Vec<AppointmentRecord>> {
    let mut query = connection.prepare(&format!(
        "SELECT appointment_id,patient_id,professional_id,service_id,responsible_contact_id,starts_at,ends_at,status,confirmed_at,{SNAPSHOT_COLUMNS},created_at,updated_at,version FROM appointment_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 ORDER BY appointment_id"
    ))?;
    query
        .query_map(
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str()
            ],
            |row| {
                Ok(AppointmentRecord {
                    appointment_id: row.get(0)?,
                    patient_id: row.get(1)?,
                    professional_id: row.get(2)?,
                    service_id: row.get(3)?,
                    responsible_contact_id: row.get(4)?,
                    starts_at: row.get(5)?,
                    ends_at: row.get(6)?,
                    status: row.get(7)?,
                    confirmed_at: row.get(8)?,
                    snapshots: snapshots_from(row, 9)?,
                    created_at: row.get(18)?,
                    updated_at: row.get(19)?,
                    version: row.get(20)?,
                })
            },
        )?
        .collect()
}

pub fn save_history_entry(
    tx: &Transaction,
    scope: &OperationContext,
    record: &AppointmentHistoryRecord,
) -> rusqlite::Result<()> {
    tx.execute(
        &format!("INSERT INTO appointment_history_view (account_id,user_id,device_id,appointment_id,version,action,actor_kind,actor_user_id,actor_contact_person_id,change_reason,changed_at,starts_at,ends_at,status,confirmed_at,{SNAPSHOT_COLUMNS}) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24) ON CONFLICT(account_id,user_id,device_id,appointment_id,version) DO UPDATE SET action=?6,actor_kind=?7,actor_user_id=?8,actor_contact_person_id=?9,change_reason=?10,changed_at=?11,starts_at=?12,ends_at=?13,status=?14,confirmed_at=?15,service_name_snapshot=?16,offered_duration_minutes_snapshot=?17,professional_name_snapshot=?18,location_name_snapshot=?19,location_address_snapshot=?20,location_country_snapshot=?21,location_state_snapshot=?22,location_city_snapshot=?23,time_zone_snapshot=?24"),
        rusqlite::params![
            scope.account_id.as_str(),
            scope.user_id.as_str(),
            scope.device_id.as_str(),
            record.appointment_id,
            record.version,
            record.action,
            record.actor_kind,
            record.actor_user_id,
            record.actor_contact_person_id,
            record.change_reason,
            record.changed_at,
            record.starts_at,
            record.ends_at,
            record.status,
            record.confirmed_at,
            record.snapshots.service_name,
            record.snapshots.offered_duration_minutes,
            record.snapshots.professional_name,
            record.snapshots.location_name,
            record.snapshots.location_address,
            record.snapshots.location_country,
            record.snapshots.location_state,
            record.snapshots.location_city,
            record.snapshots.time_zone,
        ],
    )?;

    Ok(())
}

pub fn read_history(
    connection: &Connection,
    scope: &OperationContext,
) -> rusqlite::Result<Vec<AppointmentHistoryRecord>> {
    let mut query = connection.prepare(&format!(
        "SELECT appointment_id,version,action,actor_kind,actor_user_id,actor_contact_person_id,change_reason,changed_at,starts_at,ends_at,status,confirmed_at,{SNAPSHOT_COLUMNS} FROM appointment_history_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 ORDER BY appointment_id,version"
    ))?;
    query
        .query_map(
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str()
            ],
            |row| {
                Ok(AppointmentHistoryRecord {
                    appointment_id: row.get(0)?,
                    version: row.get(1)?,
                    action: row.get(2)?,
                    actor_kind: row.get(3)?,
                    actor_user_id: row.get(4)?,
                    actor_contact_person_id: row.get(5)?,
                    change_reason: row.get(6)?,
                    changed_at: row.get(7)?,
                    starts_at: row.get(8)?,
                    ends_at: row.get(9)?,
                    status: row.get(10)?,
                    confirmed_at: row.get(11)?,
                    snapshots: snapshots_from(row, 12)?,
                })
            },
        )?
        .collect()
}

pub fn insert_local_change(
    tx: &Transaction,
    scope: &OperationContext,
    change: &LocalChange,
) -> rusqlite::Result<()> {
    tx.execute(
        "INSERT INTO appointment_local (account_id,user_id,device_id,operation_id,appointment_id,action,expected_version,base_operation_id,depends_on,payload,draft,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        params![
            scope.account_id.as_str(),
            scope.user_id.as_str(),
            scope.device_id.as_str(),
            change.operation_id,
            change.appointment_id,
            change.action,
            change.expected_version,
            change.base_operation_id,
            change.depends_on,
            change.payload,
            change.draft,
            change.created_at,
        ],
    )?;

    Ok(())
}

pub fn read_local_changes(
    connection: &Connection,
    scope: &OperationContext,
) -> rusqlite::Result<Vec<LocalChange>> {
    let mut query = connection.prepare(
        "SELECT operation_id,appointment_id,action,expected_version,base_operation_id,depends_on,payload,draft,created_at FROM appointment_local WHERE account_id=?1 AND user_id=?2 AND device_id=?3 ORDER BY created_at,operation_id",
    )?;
    query
        .query_map(
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str()
            ],
            |row| {
                Ok(LocalChange {
                    operation_id: row.get(0)?,
                    appointment_id: row.get(1)?,
                    action: row.get(2)?,
                    expected_version: row.get(3)?,
                    base_operation_id: row.get(4)?,
                    depends_on: row.get(5)?,
                    payload: row.get(6)?,
                    draft: row.get(7)?,
                    created_at: row.get(8)?,
                })
            },
        )?
        .collect()
}

use super::super::application::{
    Accepted, AppointmentMutation, AppointmentRepository, AppointmentState, EntityRow,
    LocalMutationResult, OperationView, Provisional,
};
use super::intents;
use crate::components::account::application::AccountViewQuery as _;
use crate::components::identity::domain::AccountAccess;
use crate::components::identity::repository::{AccessStore, SqliteAccessStore};
use crate::components::patients::application::PatientBookingQuery as _;
use crate::components::scheduling::application::ScheduleAvailabilityQuery as _;
use crate::components::services::application::CatalogBookingQuery as _;
use crate::components::sync::{
    adapters::outbound::SqliteOperationStore, application::record_operation, domain::NewOperation,
};
use crate::shared_kernel::id::OperationId;
use crate::shared_kernel::{Clock, IdGenerator};
use rusqlite::TransactionBehavior;
use serde_json::Value;

fn state_of(record: &AppointmentRecord) -> AppointmentState {
    AppointmentState {
        appointment_id: record.appointment_id.clone(),
        patient_id: record.patient_id.clone(),
        professional_id: record.professional_id.clone(),
        service_id: record.service_id.clone(),
        responsible_contact_id: record.responsible_contact_id.clone(),
        starts_at: record.starts_at.clone(),
        ends_at: record.ends_at.clone(),
        status: record.status.clone(),
        confirmed_at: record.confirmed_at.clone(),
        service_name: record.snapshots.service_name.clone(),
        offered_duration_minutes: record.snapshots.offered_duration_minutes,
        professional_name: record.snapshots.professional_name.clone(),
        location_name: record.snapshots.location_name.clone(),
        location_address: record.snapshots.location_address.clone(),
        location_country: record.snapshots.location_country.clone(),
        location_state: record.snapshots.location_state.clone(),
        location_city: record.snapshots.location_city.clone(),
        time_zone: record.snapshots.time_zone.clone(),
    }
}

/// Operation status view of one local change, for query outputs.
pub fn operation_view_of(
    connection: &Connection,
    scope: &OperationContext,
    local: &LocalChange,
) -> super::super::application::AppResult<OperationView> {
    operation_view(connection, scope, local)
}

fn operation_view(
    connection: &Connection,
    scope: &OperationContext,
    local: &LocalChange,
) -> super::super::application::AppResult<OperationView> {
    let result = SqliteOperationStore
        .operation_result(
            connection,
            scope,
            &OperationId::try_from(local.operation_id.clone())
                .map_err(|_| NightingaleError::internal("operación local inválida"))?,
        )
        .map_err(|_| NightingaleError::internal("resultado de operación no disponible"))?;
    let mut view = OperationView {
        operation_id: local.operation_id.clone(),
        depends_on: serde_json::from_str(&local.depends_on)
            .map_err(|_| NightingaleError::internal("dependencias locales inválidas"))?,
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

/// Merged accepted/provisional appointment rows of one scope. A provisional
/// retires only when its receipt and the accepted view confirm it.
pub fn appointment_rows(
    connection: &Connection,
    scope: &OperationContext,
) -> super::super::application::AppResult<Vec<EntityRow<AppointmentState>>> {
    use std::collections::BTreeMap;

    let mut rows: BTreeMap<String, EntityRow<AppointmentState>> = BTreeMap::new();

    for record in read_appointments(connection, scope).map_err(storage_error)? {
        let version = record.version;
        let state = state_of(&record);
        rows.insert(
            state.appointment_id.clone(),
            EntityRow {
                accepted: Some(Accepted { state, version }),
                provisional: None,
                operations: vec![],
            },
        );
    }

    for local in intents::appointment_changes(connection, scope)? {
        let state: AppointmentState = serde_json::from_str(&local.draft)
            .map_err(|_| NightingaleError::internal("borrador de cita inválido"))?;
        let accepted_version = rows
            .get(&state.appointment_id)
            .and_then(|row| row.accepted.as_ref().map(|accepted| accepted.version));
        let row = rows
            .entry(state.appointment_id.clone())
            .or_insert_with(|| EntityRow {
                accepted: None,
                provisional: None,
                operations: vec![],
            });
        row.operations
            .push(operation_view(connection, scope, &local)?);

        if !intents::confirmed(connection, scope, accepted_version, &local)? {
            row.provisional = Some(Provisional {
                state,
                operation_id: local.operation_id,
                expected_version: local.expected_version,
            });
        }
    }

    Ok(rows.into_values().collect())
}

fn current_access(
    connection: &Connection,
    access: &AccountAccess,
) -> super::super::application::AppResult<()> {
    let scope = &access.context;
    let current = SqliteAccessStore
        .access(
            connection,
            &scope.account_id,
            &scope.user_id,
            &scope.device_id,
        )
        .map_err(|_| NightingaleError::internal("autorización local no disponible"))?
        .ok_or_else(|| {
            super::super::domain::AppointmentsError::new(
                "PERMISSION_DENIED",
                "no hay permiso local",
            )
        })?;

    super::super::application::require_active(&current)?;

    if current.context.permission_epoch != scope.permission_epoch {
        return Err(super::super::domain::AppointmentsError::new(
            "PERMISSION_DENIED",
            "los permisos locales cambieron",
        ));
    }

    Ok(())
}

/// Highest accepted history version of one appointment in this scope
/// (0 when the accepted history is absent).
pub fn history_version(
    connection: &Connection,
    scope: &OperationContext,
    appointment_id: &str,
) -> rusqlite::Result<i64> {
    connection.query_row(
        "SELECT COALESCE(MAX(version),0) FROM appointment_history_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND appointment_id=?4",
        params![
            scope.account_id.as_str(),
            scope.user_id.as_str(),
            scope.device_id.as_str(),
            appointment_id
        ],
        |row| row.get(0),
    )
}

/// Write-side repository: the provisional draft and the Outbox record commit
/// or roll back together.
pub struct SqliteAppointmentRepository<'a> {
    pub connection: &'a mut Connection,
}

impl AppointmentRepository for SqliteAppointmentRepository<'_> {
    fn appointment_row(
        &mut self,
        access: &AccountAccess,
        appointment: &super::super::domain::AppointmentId,
    ) -> super::super::application::AppResult<EntityRow<AppointmentState>> {
        current_access(self.connection, access)?;
        appointment_rows(self.connection, &access.context)?
            .into_iter()
            .find(|row| {
                row.visible()
                    .is_some_and(|v| v.appointment_id == appointment.as_str())
            })
            .ok_or_else(|| {
                super::super::domain::AppointmentsError::new(
                    "NOT_FOUND",
                    "cita no disponible en este ámbito",
                )
            })
    }

    fn persist(
        &mut self,
        access: &AccountAccess,
        mutation: AppointmentMutation,
        ids: &impl IdGenerator,
        clock: &impl Clock,
    ) -> super::super::application::AppResult<LocalMutationResult> {
        super::super::application::require_active(access)?;
        let scope = access.context.clone();

        // Validation reads happen before the IMMEDIATE transaction opens.
        let prepared = intents::prepare(self.connection, &scope, mutation, ids)?;

        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        current_access(&tx, access)?;
        let now = clock
            .now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .and_then(|value| i64::try_from(value.as_millis()).ok())
            .ok_or_else(|| {
                super::super::domain::AppointmentsError::internal("reloj local inválido")
            })?;
        let operation_id = record_operation(
            &tx,
            &SqliteOperationStore,
            ids,
            clock,
            NewOperation {
                context: scope.clone(),
                depends_on: prepared.base.depends_on.clone(),
                action: prepared.action.into(),
                entity_type: "appointment".into(),
                entity_key: prepared.key.clone(),
                expected_version: prepared.base.version,
                payload: prepared.payload.clone(),
                occurred_at_local: None,
            },
        )
        .map_err(|_| {
            super::super::domain::AppointmentsError::internal("no se pudo registrar la operación")
        })?;
        insert_local_change(
            &tx,
            &scope,
            &LocalChange {
                operation_id: operation_id.to_string(),
                appointment_id: prepared.appointment_id.clone(),
                action: prepared.action.into(),
                expected_version: prepared.base.version,
                base_operation_id: prepared
                    .base
                    .operation_id
                    .as_ref()
                    .map(|id| id.as_str().to_owned()),
                depends_on: serde_json::to_string(
                    &prepared
                        .base
                        .depends_on
                        .iter()
                        .map(|id| id.as_str().to_owned())
                        .collect::<Vec<_>>(),
                )
                .map_err(|_| {
                    super::super::domain::AppointmentsError::internal("dependencias inválidas")
                })?,
                payload: prepared.payload.to_string(),
                draft: prepared.draft.to_string(),
                created_at: now,
            },
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;

        Ok(LocalMutationResult {
            operation_id: operation_id.to_string(),
            entity_type: "appointment".into(),
            entity_key: prepared.key,
            expected_version: prepared.base.version,
            depends_on: prepared
                .base
                .depends_on
                .iter()
                .map(|id| id.as_str().to_owned())
                .collect(),
            provisional: prepared.draft,
            provisional_operation_id: operation_id.to_string(),
        })
    }
}

impl SqliteAppointmentRepository<'_> {
    /// Creates an appointment against the local booking context. Every
    /// validation failure returns before any transaction opens, so nothing
    /// partial is ever stored.
    pub fn create_appointment(
        &mut self,
        access: &AccountAccess,
        input: super::super::application::CreateAppointmentInput,
        ids: &impl IdGenerator,
        clock: &impl Clock,
    ) -> super::super::application::AppResult<LocalMutationResult> {
        use super::super::application as app;

        app::require_active(access)?;

        if !access.allows_offline("CreateAppointment") {
            return Err(super::super::domain::AppointmentsError::new(
                "PERMISSION_DENIED",
                "la acción CreateAppointment no está permitida para este acceso",
            ));
        }

        let scope = access.context.clone();
        let interval = input.interval()?;

        // Accepted account view: zone, location snapshots and policy.
        let account = crate::components::account::adapters::outbound::SqliteAccountRepository {
            connection: &mut *self.connection,
            clock: &crate::shared_kernel::SystemClock,
        }
        .account_view(&scope)
        .map_err(|error| super::super::domain::AppointmentsError::new(&error.code, &error.message))?
        .ok_or_else(|| {
            super::super::domain::AppointmentsError::new("NOT_FOUND", "cuenta no disponible")
        })?;

        // Booking references: accepted or chainable provisional rows only.
        let patient_id: crate::components::patients::domain::PatientId =
            input.patient_id.clone().try_into().map_err(|_| {
                super::super::domain::AppointmentsError::validation("patientId inválido")
            })?;
        let patient = crate::components::patients::adapters::outbound::SqlitePatientRepository {
            connection: &mut *self.connection,
        }
        .booking_patient(access, &patient_id)
        .map_err(|error| super::super::domain::AppointmentsError::new(&error.code, &error.message))?
        .ok_or_else(|| {
            super::super::domain::AppointmentsError::new(
                "NOT_FOUND",
                "paciente no disponible para reservar",
            )
        })?;

        let professional_id: crate::components::services::domain::ProfessionalId =
            input.professional_id.clone().try_into().map_err(|_| {
                super::super::domain::AppointmentsError::validation("professionalId inválido")
            })?;
        let service_id: crate::components::services::domain::ServiceId =
            input.service_id.clone().try_into().map_err(|_| {
                super::super::domain::AppointmentsError::validation("serviceId inválido")
            })?;
        let offer = crate::components::services::adapters::outbound::SqliteCatalogRepository {
            connection: &mut *self.connection,
        }
        .booking_offer(access, &professional_id, &service_id)
        .map_err(|error| super::super::domain::AppointmentsError::new(&error.code, &error.message))?
        .ok_or_else(|| {
            super::super::domain::AppointmentsError::new(
                "NOT_FOUND",
                "oferta de servicio no disponible para reservar",
            )
        })?;

        // Local availability with explicit coverage. The occupancy snapshot is
        // read first so the scheduling repository never shares its &mut borrow.
        let request = crate::components::scheduling::application::AvailabilityRequest {
            professional_id: input.professional_id.clone(),
            service_id: input.service_id.clone(),
            from: interval.starts_at.timestamp(),
            until: interval.ends_at.timestamp(),
            duration_minutes: Some(interval.duration_minutes()),
            exclude_appointment_id: None,
        };
        let known = SqliteScheduleOccupancyQuery {
            connection: &*self.connection,
            scope: &scope,
        }
        .list_occupancy(&input.professional_id, request.from, request.until)
        .map_err(super::super::domain::AppointmentsError::from)?;
        let known = crate::components::scheduling::adapters::outbound::KnownOccupancy(
            known
                .into_iter()
                .map(|interval| (input.professional_id.clone(), interval))
                .collect(),
        );
        let evaluation =
            crate::components::scheduling::adapters::outbound::SqliteSchedulingRepository {
                connection: &mut *self.connection,
            }
            .local_availability(
                access,
                &request,
                &known,
                &crate::components::sync::adapters::control::SqliteSyncControlStore,
            )
            .map_err(|error| {
                super::super::domain::AppointmentsError::new(&error.code, &error.message)
            })?;

        let validated =
            app::validate_new_appointment(&account, &patient, &offer, &input, &evaluation)?;

        // Provisional references become causal dependencies of the new operation.
        let mut depends_on: Vec<String> = vec![];

        if let crate::components::patients::application::BookingReference::Provisional {
            operation_id,
            ..
        } = &patient.reference
        {
            depends_on.push(operation_id.clone());
        }

        if let crate::components::services::application::CatalogReference::Provisional {
            operation_id,
        } = &offer.reference
            && !depends_on.contains(operation_id)
        {
            depends_on.push(operation_id.clone());
        }

        self.persist(
            access,
            AppointmentMutation::Create {
                data: Box::new(validated),
                depends_on,
            },
            ids,
            clock,
        )
    }
}

impl SqliteAppointmentRepository<'_> {
    /// Reschedules an appointment to a new interval of the same real
    /// duration. The availability check excludes the appointment itself so it
    /// never blocks its own move, while every other known occupancy still
    /// applies. All validation happens before any transaction opens.
    pub fn reschedule_appointment(
        &mut self,
        access: &AccountAccess,
        appointment: super::super::domain::AppointmentId,
        base: super::super::application::EditBase,
        interval: super::super::domain::AppointmentInterval,
        ids: &impl IdGenerator,
        clock: &impl Clock,
    ) -> super::super::application::AppResult<LocalMutationResult> {
        use super::super::application as app;

        app::require_active(access)?;

        if !access.allows_offline("RescheduleAppointment") {
            return Err(super::super::domain::AppointmentsError::new(
                "PERMISSION_DENIED",
                "la acción RescheduleAppointment no está permitida para este acceso",
            ));
        }

        let scope = access.context.clone();

        // The merged row resolves the base and the transition before any write.
        let row = self.appointment_row(access, &appointment)?;
        let current = row.visible().ok_or_else(|| {
            super::super::domain::AppointmentsError::new("NOT_FOUND", "cita no disponible")
        })?;
        let next = super::super::domain::apply_transition(
            super::super::domain::AppointmentTransition::Reschedule,
            Some(current.status()?),
        )?;
        let _ = next; // the prepared draft recomputes the same transition

        // Local availability excluding the appointment being moved.
        let request = crate::components::scheduling::application::AvailabilityRequest {
            professional_id: current.professional_id.clone(),
            service_id: current.service_id.clone(),
            from: interval.starts_at.timestamp(),
            until: interval.ends_at.timestamp(),
            duration_minutes: Some(interval.duration_minutes()),
            exclude_appointment_id: Some(appointment.as_str().into()),
        };
        let known = SqliteScheduleOccupancyQuery {
            connection: &*self.connection,
            scope: &scope,
        }
        .list_occupancy(&current.professional_id, request.from, request.until)
        .map_err(super::super::domain::AppointmentsError::from)?;
        let known = crate::components::scheduling::adapters::outbound::KnownOccupancy(
            known
                .into_iter()
                .map(|occupancy| (current.professional_id.clone(), occupancy))
                .collect(),
        );
        let evaluation =
            crate::components::scheduling::adapters::outbound::SqliteSchedulingRepository {
                connection: &mut *self.connection,
            }
            .local_availability(
                access,
                &request,
                &known,
                &crate::components::sync::adapters::control::SqliteSyncControlStore,
            )
            .map_err(|error| {
                super::super::domain::AppointmentsError::new(&error.code, &error.message)
            })?;

        if evaluation.coverage == "insufficient_coverage" {
            return Err(super::super::domain::AppointmentsError::new(
                "INSUFFICIENT_COVERAGE",
                "la ventana de ocupación descargada no cubre el intervalo; sincronice antes de reprogramar",
            ));
        }

        let start = interval.starts_at.timestamp().to_string();
        let end = interval.ends_at.timestamp().to_string();
        let fits = evaluation
            .intervals
            .iter()
            .any(|available| available.starts_at <= start && end <= available.ends_at);

        if !fits {
            return Err(super::super::domain::AppointmentsError::new(
                "SLOT_UNAVAILABLE",
                "el intervalo queda fuera del horario disponible o dentro de una ocupación",
            ));
        }

        self.persist(
            access,
            AppointmentMutation::Reschedule {
                appointment,
                base,
                interval,
            },
            ids,
            clock,
        )
    }
}

impl SqliteAppointmentRepository<'_> {
    /// Completes an appointment whose start has been reached. The injected
    /// clock is the only local time source; the server stays authoritative.
    pub fn complete_appointment(
        &mut self,
        access: &AccountAccess,
        appointment: super::super::domain::AppointmentId,
        base: super::super::application::EditBase,
        reason: Option<super::super::domain::ChangeReason>,
        ids: &impl IdGenerator,
        clock: &impl Clock,
    ) -> super::super::application::AppResult<LocalMutationResult> {
        use super::super::application as app;

        app::require_active(access)?;

        if !access.allows_offline("CompleteAppointment") {
            return Err(super::super::domain::AppointmentsError::new(
                "PERMISSION_DENIED",
                "la acción CompleteAppointment no está permitida para este acceso",
            ));
        }

        let row = self.appointment_row(access, &appointment)?;
        let current = row.visible().ok_or_else(|| {
            super::super::domain::AppointmentsError::new("NOT_FOUND", "cita no disponible")
        })?;

        // Completing before the start never creates an operation.
        let starts_millis = jiff::Timestamp::from_str(&current.starts_at)
            .map_err(|_| {
                super::super::domain::AppointmentsError::internal("intervalo local inválido")
            })?
            .as_millisecond();
        let now_millis = clock_millis(clock)?;

        if now_millis < starts_millis {
            return Err(super::super::domain::AppointmentsError::validation(
                "aún no se alcanza el inicio de la cita",
            ));
        }

        self.persist(
            access,
            AppointmentMutation::Complete {
                appointment,
                base,
                reason,
            },
            ids,
            clock,
        )
    }

    /// Marks a no-show after the start plus the accepted account grace
    /// window. A missed response never marks absence by itself: this is an
    /// explicit staff action gated by the injected clock.
    pub fn mark_no_show_appointment(
        &mut self,
        access: &AccountAccess,
        appointment: super::super::domain::AppointmentId,
        base: super::super::application::EditBase,
        reason: Option<super::super::domain::ChangeReason>,
        ids: &impl IdGenerator,
        clock: &impl Clock,
    ) -> super::super::application::AppResult<LocalMutationResult> {
        use super::super::application as app;

        app::require_active(access)?;

        if !access.allows_offline("MarkAppointmentNoShow") {
            return Err(super::super::domain::AppointmentsError::new(
                "PERMISSION_DENIED",
                "la acción MarkAppointmentNoShow no está permitida para este acceso",
            ));
        }

        let scope = access.context.clone();
        let row = self.appointment_row(access, &appointment)?;
        let current = row.visible().ok_or_else(|| {
            super::super::domain::AppointmentsError::new("NOT_FOUND", "cita no disponible")
        })?;

        // The accepted, currently valid account policy owns the grace window.
        let account = crate::components::account::adapters::outbound::SqliteAccountRepository {
            connection: &mut *self.connection,
            clock: &crate::shared_kernel::SystemClock,
        }
        .account_view(&scope)
        .map_err(|error| super::super::domain::AppointmentsError::new(&error.code, &error.message))?
        .ok_or_else(|| {
            super::super::domain::AppointmentsError::new("NOT_FOUND", "cuenta no disponible")
        })?;
        let grace_minutes = account.policy.no_show_grace_minutes();

        let starts_millis = jiff::Timestamp::from_str(&current.starts_at)
            .map_err(|_| {
                super::super::domain::AppointmentsError::internal("intervalo local inválido")
            })?
            .as_millisecond();
        let now_millis = clock_millis(clock)?;
        let grace_millis = i128::from(grace_minutes) * 60_000;

        if i128::from(now_millis) < i128::from(starts_millis) + grace_millis {
            return Err(super::super::domain::AppointmentsError::validation(
                "aún no vence el margen de ausencia de la cita",
            ));
        }

        self.persist(
            access,
            AppointmentMutation::NoShow {
                appointment,
                base,
                reason,
            },
            ids,
            clock,
        )
    }
}

fn clock_millis(clock: &impl Clock) -> super::super::application::AppResult<i64> {
    let millis = clock
        .now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|value| i64::try_from(value.as_millis()).ok())
        .ok_or_else(|| super::super::domain::AppointmentsError::internal("reloj local inválido"))?;
    Ok(millis)
}
