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
#[derive(Debug, Clone, PartialEq, Eq)]
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
