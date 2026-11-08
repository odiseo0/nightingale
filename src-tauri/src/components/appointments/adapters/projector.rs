//! Accepted-view projector for appointments.
//! It translates remote `appointment` changes into this scope's views: the
//! full accepted appointment row and the minimal occupancy projection. Only
//! occupying states keep occupancy; `cancelled` frees it by removing the row.
//! It never touches local intentions.
use crate::components::sync::adapters::remote_models::{
    AppointmentHistoryProjection, AppointmentResponseStatus, ProjectionId, RemoteState,
};
use crate::components::sync::remote::{EntityType, RemoteChange, RemoteProjector};
use crate::shared_kernel::{NightingaleError, Result, id::OperationContext};
use jiff::Timestamp;
use rusqlite::{OptionalExtension, Transaction, params};
use std::str::FromStr;

pub struct AppointmentProjector;

fn storage_error(_: rusqlite::Error) -> NightingaleError {
    NightingaleError::internal("proyección local de citas no disponible")
}

fn id(value: &ProjectionId) -> String {
    value.clone().into()
}

fn key(value: &serde_json::Value) -> Result<String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| NightingaleError::validation("clave remota inválida"))
}

fn instant(value: &str) -> Result<Timestamp> {
    Timestamp::from_str(value).map_err(|_| NightingaleError::validation("instante remoto inválido"))
}

fn status_str(status: &AppointmentResponseStatus) -> &'static str {
    match status {
        AppointmentResponseStatus::Scheduled => "scheduled",
        AppointmentResponseStatus::Confirmed => "confirmed",
        AppointmentResponseStatus::Cancelled => "cancelled",
        AppointmentResponseStatus::Completed => "completed",
        AppointmentResponseStatus::NoShow => "no_show",
    }
}

/// The only states that keep occupancy; `cancelled` frees it.
fn occupies(status: &AppointmentResponseStatus) -> Option<&'static str> {
    match status {
        AppointmentResponseStatus::Scheduled => Some("scheduled"),
        AppointmentResponseStatus::Confirmed => Some("confirmed"),
        AppointmentResponseStatus::Completed => Some("completed"),
        AppointmentResponseStatus::NoShow => Some("no_show"),
        AppointmentResponseStatus::Cancelled => None,
    }
}

/// Validates and stores an aggregated history projection. The accepted set of
/// one appointment is replaced only by a strictly newer aggregate; entries are
/// immutable and always start at version 1 without gaps or duplicates.
fn project_history(
    tx: &Transaction,
    s: &OperationContext,
    v: &AppointmentHistoryProjection,
) -> Result<()> {
    if id(&v.account_id) != s.account_id.as_str() {
        return Err(NightingaleError::validation(
            "historia remota fuera de ámbito",
        ));
    }

    if v.version < 1 || v.items.is_empty() {
        return Err(NightingaleError::validation(
            "historia remota sin entradas válidas",
        ));
    }

    // Consecutive entries from 1, each tied to this appointment and scope.
    for (index, item) in v.items.iter().enumerate() {
        if item.version != (index as i64) + 1 {
            return Err(NightingaleError::validation(
                "historia remota con huecos o duplicados",
            ));
        }

        if id(&item.appointment_id) != id(&v.appointment_id)
            || id(&item.account_id) != s.account_id.as_str()
        {
            return Err(NightingaleError::validation(
                "historia remota fuera de ámbito",
            ));
        }

        if instant(&item.starts_at)? >= instant(&item.ends_at)? {
            return Err(NightingaleError::validation(
                "intervalo remoto de historia inválido",
            ));
        }
    }

    // The aggregate version is its last entry's version.
    let last = v.items.last().unwrap();

    if last.version != v.version {
        return Err(NightingaleError::validation(
            "historia remota inconsistente con su versión agregada",
        ));
    }

    // When the accepted appointment is already at (or past) this aggregate,
    // its current state must match the last entry.
    let current: Option<(i64, String)> = tx
        .query_row(
            "SELECT version,status FROM appointment_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND appointment_id=?4",
            params![
                s.account_id.as_str(),
                s.user_id.as_str(),
                s.device_id.as_str(),
                id(&v.appointment_id)
            ],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(storage_error)?;

    if let Some((version, status)) = current
        && version >= v.version
        && status != status_str(&last.status)
    {
        return Err(NightingaleError::validation(
            "historia remota inconsistente con la cita aceptada",
        ));
    }

    // An older or repeated aggregate never rewrites accepted history.
    let stored: i64 = tx
        .query_row(
            "SELECT COALESCE(MAX(version),0) FROM appointment_history_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND appointment_id=?4",
            params![
                s.account_id.as_str(),
                s.user_id.as_str(),
                s.device_id.as_str(),
                id(&v.appointment_id)
            ],
            |row| row.get(0),
        )
        .map_err(storage_error)?;

    if stored >= v.version {
        return Ok(());
    }

    tx.execute(
        "DELETE FROM appointment_history_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND appointment_id=?4",
        params![
            s.account_id.as_str(),
            s.user_id.as_str(),
            s.device_id.as_str(),
            id(&v.appointment_id)
        ],
    )
    .map_err(storage_error)?;

    for item in &v.items {
        tx.execute(
            "INSERT INTO appointment_history_view (account_id,user_id,device_id,appointment_id,version,action,actor_kind,actor_user_id,actor_contact_person_id,change_reason,changed_at,starts_at,ends_at,status,confirmed_at,service_name_snapshot,offered_duration_minutes_snapshot,professional_name_snapshot,location_name_snapshot,location_address_snapshot,location_country_snapshot,location_state_snapshot,location_city_snapshot,time_zone_snapshot) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24)",
            params![
                s.account_id.as_str(),
                s.user_id.as_str(),
                s.device_id.as_str(),
                id(&v.appointment_id),
                item.version,
                action_str(&item.action),
                actor_str(&item.actor_kind),
                item.actor_user_id.as_ref().map(id),
                item.actor_contact_person_id.as_ref().map(id),
                item.change_reason,
                item.changed_at,
                item.starts_at,
                item.ends_at,
                status_str(&item.status),
                item.confirmed_at,
                item.service_name_snapshot,
                item.offered_duration_minutes_snapshot,
                item.professional_name_snapshot,
                item.location_name_snapshot,
                item.location_address_snapshot,
                item.location_country_snapshot,
                item.location_state_snapshot,
                item.location_city_snapshot,
                item.time_zone_snapshot,
            ],
        )
        .map_err(storage_error)?;
    }

    Ok(())
}

fn action_str(
    action: &crate::components::sync::adapters::remote_models::AppointmentHistoryResponseAction,
) -> &'static str {
    use crate::components::sync::adapters::remote_models::AppointmentHistoryResponseAction as A;
    match action {
        A::Created => "created",
        A::Confirmed => "confirmed",
        A::Rescheduled => "rescheduled",
        A::Cancelled => "cancelled",
        A::Completed => "completed",
        A::NoShow => "no_show",
    }
}

fn actor_str(
    actor: &crate::components::sync::adapters::remote_models::AppointmentHistoryResponseActorKind,
) -> &'static str {
    use crate::components::sync::adapters::remote_models::AppointmentHistoryResponseActorKind as A;
    match actor {
        A::Staff => "staff",
        A::Patient => "patient",
        A::Contact => "contact",
    }
}

impl RemoteProjector for AppointmentProjector {
    fn reset(&self, tx: &Transaction, s: &OperationContext) -> Result<()> {
        for table in [
            "appointment_view",
            "appointment_history_view",
            "appointment_occupancy_view",
        ] {
            tx.execute(
                &format!("DELETE FROM {table} WHERE account_id=?1 AND user_id=?2 AND device_id=?3"),
                params![
                    s.account_id.as_str(),
                    s.user_id.as_str(),
                    s.device_id.as_str()
                ],
            )
            .map_err(storage_error)?;
        }

        Ok(())
    }

    fn project(&self, tx: &Transaction, s: &OperationContext, change: &RemoteChange) -> Result<()> {
        match change {
            RemoteChange::Upsert { state, .. } => {
                if let RemoteState::Appointment(v) = state.as_ref() {
                    if id(&v.account_id) != s.account_id.as_str() {
                        return Err(NightingaleError::validation("cita remota fuera de ámbito"));
                    }

                    if instant(&v.starts_at)? >= instant(&v.ends_at)? {
                        return Err(NightingaleError::validation(
                            "intervalo remoto de cita inválido",
                        ));
                    }

                    // Full accepted row: snapshots stay exactly as the server
                    // computed them; versions never regress.
                    tx.execute(
                        "INSERT INTO appointment_view (account_id,user_id,device_id,appointment_id,patient_id,professional_id,service_id,responsible_contact_id,starts_at,ends_at,status,confirmed_at,service_name_snapshot,offered_duration_minutes_snapshot,professional_name_snapshot,location_name_snapshot,location_address_snapshot,location_country_snapshot,location_state_snapshot,location_city_snapshot,time_zone_snapshot,created_at,updated_at,version) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24)
                        ON CONFLICT(account_id,user_id,device_id,appointment_id) DO UPDATE SET patient_id=excluded.patient_id,professional_id=excluded.professional_id,service_id=excluded.service_id,responsible_contact_id=excluded.responsible_contact_id,starts_at=excluded.starts_at,ends_at=excluded.ends_at,status=excluded.status,confirmed_at=excluded.confirmed_at,service_name_snapshot=excluded.service_name_snapshot,offered_duration_minutes_snapshot=excluded.offered_duration_minutes_snapshot,professional_name_snapshot=excluded.professional_name_snapshot,location_name_snapshot=excluded.location_name_snapshot,location_address_snapshot=excluded.location_address_snapshot,location_country_snapshot=excluded.location_country_snapshot,location_state_snapshot=excluded.location_state_snapshot,location_city_snapshot=excluded.location_city_snapshot,time_zone_snapshot=excluded.time_zone_snapshot,created_at=excluded.created_at,updated_at=excluded.updated_at,version=excluded.version
                        WHERE excluded.version>appointment_view.version",
                        params![
                            s.account_id.as_str(),
                            s.user_id.as_str(),
                            s.device_id.as_str(),
                            id(&v.id),
                            id(&v.patient_id),
                            id(&v.professional_id),
                            id(&v.service_id),
                            v.responsible_contact_id.as_ref().map(id),
                            v.starts_at,
                            v.ends_at,
                            status_str(&v.status),
                            v.confirmed_at,
                            v.service_name_snapshot,
                            v.offered_duration_minutes_snapshot,
                            v.professional_name_snapshot,
                            v.location_name_snapshot,
                            v.location_address_snapshot,
                            v.location_country_snapshot,
                            v.location_state_snapshot,
                            v.location_city_snapshot,
                            v.time_zone_snapshot,
                            v.created_at,
                            v.updated_at,
                            v.version
                        ],
                    )
                    .map_err(storage_error)?;

                    match occupies(&v.status) {
                        Some(status) => {
                            tx.execute(
                                "INSERT INTO appointment_occupancy_view (account_id,user_id,device_id,appointment_id,professional_id,starts_at,ends_at,status,version) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)
                                ON CONFLICT(account_id,user_id,device_id,appointment_id) DO UPDATE SET professional_id=excluded.professional_id,starts_at=excluded.starts_at,ends_at=excluded.ends_at,status=excluded.status,version=excluded.version
                                WHERE excluded.version>appointment_occupancy_view.version",
                                params![
                                    s.account_id.as_str(),
                                    s.user_id.as_str(),
                                    s.device_id.as_str(),
                                    id(&v.id),
                                    id(&v.professional_id),
                                    v.starts_at,
                                    v.ends_at,
                                    status,
                                    v.version
                                ],
                            )
                            .map_err(storage_error)?;
                        }
                        None => {
                            // cancelled: occupancy is freed and the row is gone.
                            tx.execute(
                                "DELETE FROM appointment_occupancy_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND appointment_id=?4",
                                params![
                                    s.account_id.as_str(),
                                    s.user_id.as_str(),
                                    s.device_id.as_str(),
                                    id(&v.id)
                                ],
                            )
                            .map_err(storage_error)?;
                        }
                    }
                }

                if let RemoteState::AppointmentHistory(v) = state.as_ref() {
                    project_history(tx, s, v)?;
                }
            }
            RemoteChange::Remove {
                entity_type,
                key: entity_key,
            } => {
                if *entity_type == EntityType::AppointmentHistory {
                    let appointment = key(entity_key)?;
                    tx.execute(
                        "DELETE FROM appointment_history_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND appointment_id=?4",
                        params![s.account_id.as_str(), s.user_id.as_str(), s.device_id.as_str(), appointment],
                    )
                    .map_err(storage_error)?;
                }

                if *entity_type == EntityType::Appointment {
                    let appointment = key(entity_key)?;

                    for table in [
                        "appointment_view",
                        "appointment_history_view",
                        "appointment_occupancy_view",
                    ] {
                        tx.execute(
                            &format!("DELETE FROM {table} WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND appointment_id=?4"),
                            params![
                                s.account_id.as_str(),
                                s.user_id.as_str(),
                                s.device_id.as_str(),
                                appointment
                            ],
                        )
                        .map_err(storage_error)?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "../projection_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "../history_projection_tests.rs"]
mod history_tests;
