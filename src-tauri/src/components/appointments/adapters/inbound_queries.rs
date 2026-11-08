//! Specific local query commands for appointments.
//! Outputs keep accepted rows, provisional drafts and local operations
//! separate and expose no epoch, fingerprint, token, SQL or raw remote state.
use super::super::application::{AppResult, AppointmentState, EntityRow, OperationView};
use super::inbound_commands::run;
use super::outbound::{appointment_rows, read_history, read_local_changes};
use crate::components::appointments::domain::{
    AppointmentId, AppointmentStatus, AppointmentsError,
};
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::Mutex;
use tauri::State;

/// Maximum `[from,until)` span of a listing (contract window: 366 days).
pub(crate) const MAX_LIST_RANGE_DAYS: i64 = 366;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListAppointmentsRequest {
    pub from: String,
    pub until: String,
    #[serde(default)]
    pub professional_id: Option<String>,
    #[serde(default)]
    pub service_id: Option<String>,
    #[serde(default)]
    pub patient_id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(default)]
    pub shows: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppointmentPage {
    pub items: Vec<EntityRow<AppointmentState>>,
    pub total: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GetAppointmentRequest {
    pub appointment_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GetHistoryRequest {
    pub appointment_id: String,
    #[serde(default)]
    pub after_version: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryView {
    pub accepted: Vec<super::outbound::AppointmentHistoryRecord>,
    pub operations: Vec<OperationView>,
}

fn instant(value: &str) -> AppResult<jiff::Timestamp> {
    jiff::Timestamp::from_str(value)
        .map_err(|_| AppointmentsError::validation("el instante debe ser RFC 3339 con offset o Z"))
}

use std::str::FromStr;

fn merged_row(
    connection: &Connection,
    access: &crate::components::identity::domain::AccountAccess,
    appointment: &AppointmentId,
) -> AppResult<EntityRow<AppointmentState>> {
    appointment_rows(connection, &access.context)?
        .into_iter()
        .find(|row| {
            row.visible()
                .is_some_and(|v| v.appointment_id == appointment.as_str())
        })
        .ok_or_else(|| AppointmentsError::new("NOT_FOUND", "cita no disponible en este ámbito"))
}

pub(crate) fn list_appointments(
    connection: &mut Connection,
    request: ListAppointmentsRequest,
) -> AppResult<AppointmentPage> {
    let from = instant(&request.from)?;
    let until = instant(&request.until)?;
    let range = until
        .as_second()
        .checked_sub(from.as_second())
        .ok_or_else(|| AppointmentsError::validation("rango de consulta inválido"))?;

    if from >= until || range > MAX_LIST_RANGE_DAYS * 86_400 {
        return Err(AppointmentsError::validation(
            "el rango debe ser un intervalo de hasta 366 días",
        ));
    }

    if let Some(status) = &request.status {
        AppointmentStatus::try_from(status.as_str())?;
    }

    let page = request.page.unwrap_or(1);
    let shows = request.shows.unwrap_or(100);

    if page < 1 || !(1..=500).contains(&shows) {
        return Err(AppointmentsError::validation(
            "la paginación exige página ≥1 y tamaño 1–500",
        ));
    }

    run(connection, |repo, access| {
        let mut rows: Vec<EntityRow<AppointmentState>> =
            appointment_rows(repo.connection, &access.context)?
                .into_iter()
                .filter(|row| {
                    let Some(state) = row.visible() else {
                        return false;
                    };
                    // Intersection with [from,until): a listing sees appointments
                    // whose interval overlaps the requested window.
                    let starts = instant(&state.starts_at).ok();
                    let ends = instant(&state.ends_at).ok();
                    let Some((starts, ends)) = starts.zip(ends) else {
                        return false;
                    };

                    if starts >= until || ends <= from {
                        return false;
                    }

                    if let Some(professional) = &request.professional_id
                        && &state.professional_id != professional
                    {
                        return false;
                    }

                    if let Some(service) = &request.service_id
                        && &state.service_id != service
                    {
                        return false;
                    }

                    if let Some(patient) = &request.patient_id
                        && &state.patient_id != patient
                    {
                        return false;
                    }

                    if let Some(status) = &request.status
                        && &state.status != status
                    {
                        return false;
                    }

                    true
                })
                .collect();
        rows.sort_by(|a, b| {
            let (Some(x), Some(y)) = (a.visible(), b.visible()) else {
                return std::cmp::Ordering::Equal;
            };
            x.starts_at
                .cmp(&y.starts_at)
                .then_with(|| x.appointment_id.cmp(&y.appointment_id))
        });

        let total = rows.len();
        let start = ((page - 1) as usize) * shows as usize;
        let items = rows.into_iter().skip(start).take(shows as usize).collect();

        Ok(AppointmentPage { items, total })
    })
}

pub(crate) fn get_appointment(
    connection: &mut Connection,
    request: GetAppointmentRequest,
) -> AppResult<EntityRow<AppointmentState>> {
    let appointment: AppointmentId =
        request.appointment_id.clone().try_into().map_err(|_| {
            AppointmentsError::validation("appointmentId debe ser un UUIDv7 canónico")
        })?;
    run(connection, |repo, access| {
        merged_row(repo.connection, access, &appointment)
    })
}

pub(crate) fn get_history(
    connection: &mut Connection,
    request: GetHistoryRequest,
) -> AppResult<HistoryView> {
    let appointment: AppointmentId =
        request.appointment_id.clone().try_into().map_err(|_| {
            AppointmentsError::validation("appointmentId debe ser un UUIDv7 canónico")
        })?;

    if request.after_version.is_some_and(|version| version < 0) {
        return Err(AppointmentsError::validation(
            "afterVersion no puede ser negativo",
        ));
    }

    run(connection, |repo, access| {
        let accepted = read_history(repo.connection, &access.context)
            .map_err(|_| AppointmentsError::internal("historia local no disponible"))?
            .into_iter()
            .filter(|record| {
                record.appointment_id == appointment.as_str()
                    && request
                        .after_version
                        .map(|after| record.version > after)
                        .unwrap_or(true)
            })
            .collect();

        let mut operations = vec![];

        for change in read_local_changes(repo.connection, &access.context)
            .map_err(|_| AppointmentsError::internal("intenciones locales no disponibles"))?
        {
            if change.appointment_id == appointment.as_str() {
                operations.push(super::outbound::operation_view_of(
                    repo.connection,
                    &access.context,
                    &change,
                )?);
            }
        }

        Ok(HistoryView {
            accepted,
            operations,
        })
    })
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookingContext {
    pub time_zone: String,
    pub no_show_grace_minutes: i64,
    pub patients: Vec<
        crate::components::patients::application::EntityRow<
            crate::components::patients::application::ProfileState,
        >,
    >,
    pub offers: Vec<
        crate::components::services::application::CatalogRow<
            crate::components::services::application::ServiceOfferView,
            crate::components::services::application::ServiceOfferDraft,
        >,
    >,
    pub professionals: Vec<
        crate::components::services::application::CatalogRow<
            crate::components::services::application::ProfessionalView,
            crate::components::services::application::ProfessionalDraft,
        >,
    >,
}

pub(crate) fn get_booking_context(connection: &mut Connection) -> AppResult<BookingContext> {
    use crate::components::account::application::AccountViewQuery as _;
    use crate::components::patients::application::PatientRepository as _;
    use crate::components::services::application::CatalogRepository as _;

    run(connection, |repo, access| {
        let account = crate::components::account::adapters::outbound::SqliteAccountRepository {
            connection: &mut *repo.connection,
            clock: &crate::shared_kernel::SystemClock,
        }
        .account_view(&access.context)
        .map_err(|error| AppointmentsError::new(&error.code, &error.message))?
        .ok_or_else(|| AppointmentsError::new("NOT_FOUND", "cuenta no disponible"))?;

        let patients = crate::components::patients::adapters::outbound::SqlitePatientRepository {
            connection: &mut *repo.connection,
        }
        .profiles(
            access,
            &crate::components::patients::application::PatientFilter {
                q: None,
                status: Some(crate::components::patients::domain::Status::Active),
                completion_status: None,
                page: 1,
                shows: 500,
            },
        )
        .map_err(|error| AppointmentsError::new(&error.code, &error.message))?
        .items;

        let screen = crate::components::services::adapters::outbound::SqliteCatalogRepository {
            connection: &mut *repo.connection,
        }
        .catalog(access)
        .map_err(|error| AppointmentsError::new(&error.code, &error.message))?;

        Ok(BookingContext {
            time_zone: account.time_zone,
            no_show_grace_minutes: account.policy.no_show_grace_minutes(),
            patients,
            offers: screen
                .offers
                .into_iter()
                .filter(|row| {
                    row.accepted
                        .as_ref()
                        .map(|view| view.status == "active" && view.service_status == "active")
                        .unwrap_or(false)
                        || row.provisional.is_some()
                })
                .collect(),
            professionals: screen
                .professionals
                .into_iter()
                .filter(|row| {
                    row.accepted
                        .as_ref()
                        .map(|view| view.status == "active")
                        .unwrap_or(false)
                        || row.provisional.is_some()
                })
                .collect(),
        })
    })
}

#[tauri::command]
pub fn appointments_list_appointments(
    connection: State<Mutex<Connection>>,
    request: ListAppointmentsRequest,
) -> AppResult<AppointmentPage> {
    list_appointments(
        &mut *connection
            .lock()
            .map_err(|_| super::inbound_commands::locked())?,
        request,
    )
}

#[tauri::command]
pub fn appointments_get_appointment(
    connection: State<Mutex<Connection>>,
    request: GetAppointmentRequest,
) -> AppResult<EntityRow<AppointmentState>> {
    get_appointment(
        &mut *connection
            .lock()
            .map_err(|_| super::inbound_commands::locked())?,
        request,
    )
}

#[tauri::command]
pub fn appointments_get_history(
    connection: State<Mutex<Connection>>,
    request: GetHistoryRequest,
) -> AppResult<HistoryView> {
    get_history(
        &mut *connection
            .lock()
            .map_err(|_| super::inbound_commands::locked())?,
        request,
    )
}

#[tauri::command]
pub fn appointments_get_booking_context(
    connection: State<Mutex<Connection>>,
) -> AppResult<BookingContext> {
    get_booking_context(
        &mut *connection
            .lock()
            .map_err(|_| super::inbound_commands::locked())?,
    )
}
