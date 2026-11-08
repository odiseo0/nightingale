//! Accepted-view projector for appointment occupancy.
//! It translates remote `appointment` changes into the minimal occupancy
//! projection: only occupying states are stored; `cancelled` frees occupancy
//! by removing the row. It never touches local intentions.
use crate::components::sync::adapters::remote_models::{
    AppointmentResponseStatus, ProjectionId, RemoteState,
};
use crate::components::sync::remote::{EntityType, RemoteChange, RemoteProjector};
use crate::shared_kernel::{NightingaleError, Result, id::OperationContext};
use jiff::Timestamp;
use rusqlite::{Transaction, params};
use std::str::FromStr;

pub struct AppointmentOccupancyProjector;

fn storage_error(_: rusqlite::Error) -> NightingaleError {
    NightingaleError::internal("proyección local de ocupación no disponible")
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

impl RemoteProjector for AppointmentOccupancyProjector {
    fn reset(&self, tx: &Transaction, s: &OperationContext) -> Result<()> {
        tx.execute(
            "DELETE FROM appointment_occupancy_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3",
            params![s.account_id.as_str(), s.user_id.as_str(), s.device_id.as_str()],
        )
        .map_err(storage_error)?;

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
            }
            RemoteChange::Remove {
                entity_type,
                key: entity_key,
            } => {
                if *entity_type == EntityType::Appointment {
                    let appointment = key(entity_key)?;
                    tx.execute(
                        "DELETE FROM appointment_occupancy_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND appointment_id=?4",
                        params![s.account_id.as_str(), s.user_id.as_str(), s.device_id.as_str(), appointment],
                    )
                    .map_err(storage_error)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "../projection_tests.rs"]
mod tests;
