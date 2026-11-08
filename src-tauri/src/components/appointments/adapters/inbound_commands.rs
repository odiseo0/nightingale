//! Specific local commands for appointment lifecycle writes.
//! React cannot supply an account scope, an operation envelope or SQL.
use super::super::application::{
    AppointmentMutation, AppointmentRepository, CreateAppointmentInput, EditBase,
    LocalMutationResult,
};
use super::super::domain::{AppointmentId, AppointmentsError, ChangeReason, MinuteInstant};
use super::outbound::SqliteAppointmentRepository;
use crate::components::identity::repository::SqliteAccessStore;
use crate::shared_kernel::{SystemClock, UuidV7Generator};
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::Mutex;
use tauri::State;

pub(crate) fn locked() -> AppointmentsError {
    AppointmentsError::internal("base local no disponible")
}

/// Loads the selected access and hands the connection to one appointments call.
pub(crate) fn run<T>(
    c: &mut Connection,
    work: impl FnOnce(
        &mut SqliteAppointmentRepository<'_>,
        &crate::components::identity::domain::AccountAccess,
    ) -> super::super::application::AppResult<T>,
) -> super::super::application::AppResult<T> {
    let access = SqliteAccessStore
        .selected_access(c)
        .map_err(|_| AppointmentsError::internal("autorización local no disponible"))?
        .ok_or_else(|| {
            AppointmentsError::new(
                "AUTHENTICATION_REQUIRED",
                "inicie sesión y elija una cuenta",
            )
        })?;
    let mut repository = SqliteAppointmentRepository { connection: c };

    work(&mut repository, &access)
}

/// The edit base is exactly one of an accepted version or the last local operation.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EditBaseRequest {
    #[serde(default)]
    pub accepted_version: Option<i64>,
    #[serde(default)]
    pub operation_id: Option<String>,
}

impl TryFrom<EditBaseRequest> for EditBase {
    type Error = AppointmentsError;

    fn try_from(value: EditBaseRequest) -> super::super::application::AppResult<Self> {
        match (value.accepted_version, value.operation_id) {
            (Some(version), None) if version > 0 => Ok(EditBase::Accepted { version }),
            (None, Some(operation_id)) => Ok(EditBase::LocalOperation { operation_id }),
            _ => Err(AppointmentsError::validation(
                "la base debe ser una versión aceptada o una operación local",
            )),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateAppointmentRequest {
    pub patient_id: String,
    pub professional_id: String,
    pub service_id: String,
    #[serde(default)]
    pub responsible_contact_id: Option<String>,
    pub starts_at: String,
    pub ends_at: String,
}

impl TryFrom<CreateAppointmentRequest> for CreateAppointmentInput {
    type Error = AppointmentsError;

    fn try_from(value: CreateAppointmentRequest) -> super::super::application::AppResult<Self> {
        Ok(CreateAppointmentInput {
            patient_id: value.patient_id,
            professional_id: value.professional_id,
            service_id: value.service_id,
            responsible_contact_id: value.responsible_contact_id,
            starts_at: value.starts_at,
            ends_at: value.ends_at,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfirmAppointmentRequest {
    pub appointment_id: String,
    pub base: EditBaseRequest,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RescheduleAppointmentRequest {
    pub appointment_id: String,
    pub base: EditBaseRequest,
    pub starts_at: String,
    pub ends_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LifecycleAppointmentRequest {
    pub appointment_id: String,
    pub base: EditBaseRequest,
    #[serde(default)]
    pub change_reason: Option<String>,
}

fn appointment_id(value: &str) -> super::super::application::AppResult<AppointmentId> {
    value
        .to_owned()
        .try_into()
        .map_err(|_| AppointmentsError::validation("appointmentId debe ser un UUIDv7 canónico"))
}

fn interval_of(
    starts_at: &str,
    ends_at: &str,
) -> super::super::application::AppResult<super::super::domain::AppointmentInterval> {
    let starts = MinuteInstant::parse(starts_at)?;
    let ends = MinuteInstant::parse(ends_at)?;
    super::super::domain::AppointmentInterval::new(starts, ends)
}

fn reason_of(value: Option<String>) -> super::super::application::AppResult<Option<ChangeReason>> {
    ChangeReason::new(value)
}

pub(crate) fn create_appointment(
    connection: &mut Connection,
    request: CreateAppointmentRequest,
) -> super::super::application::AppResult<LocalMutationResult> {
    let input = request.try_into()?;
    run(connection, |repo, access| {
        repo.create_appointment(access, input, &UuidV7Generator, &SystemClock)
    })
}

pub(crate) fn confirm_appointment(
    connection: &mut Connection,
    request: ConfirmAppointmentRequest,
) -> super::super::application::AppResult<LocalMutationResult> {
    let appointment = appointment_id(&request.appointment_id)?;
    let base = request.base.try_into()?;
    run(connection, |repo, access| {
        repo.persist(
            access,
            AppointmentMutation::Confirm { appointment, base },
            &UuidV7Generator,
            &SystemClock,
        )
    })
}

pub(crate) fn reschedule_appointment(
    connection: &mut Connection,
    request: RescheduleAppointmentRequest,
) -> super::super::application::AppResult<LocalMutationResult> {
    let appointment = appointment_id(&request.appointment_id)?;
    let base = request.base.try_into()?;
    let interval = interval_of(&request.starts_at, &request.ends_at)?;
    run(connection, |repo, access| {
        repo.reschedule_appointment(
            access,
            appointment,
            base,
            interval,
            &UuidV7Generator,
            &SystemClock,
        )
    })
}

pub(crate) fn cancel_appointment(
    connection: &mut Connection,
    request: LifecycleAppointmentRequest,
) -> super::super::application::AppResult<LocalMutationResult> {
    let appointment = appointment_id(&request.appointment_id)?;
    let base = request.base.try_into()?;
    let reason = reason_of(request.change_reason)?;
    run(connection, |repo, access| {
        repo.persist(
            access,
            AppointmentMutation::Cancel {
                appointment,
                base,
                reason,
            },
            &UuidV7Generator,
            &SystemClock,
        )
    })
}

pub(crate) fn complete_appointment(
    connection: &mut Connection,
    request: LifecycleAppointmentRequest,
) -> super::super::application::AppResult<LocalMutationResult> {
    let appointment = appointment_id(&request.appointment_id)?;
    let base = request.base.try_into()?;
    let reason = reason_of(request.change_reason)?;
    run(connection, |repo, access| {
        repo.complete_appointment(
            access,
            appointment,
            base,
            reason,
            &UuidV7Generator,
            &SystemClock,
        )
    })
}

pub(crate) fn mark_no_show(
    connection: &mut Connection,
    request: LifecycleAppointmentRequest,
) -> super::super::application::AppResult<LocalMutationResult> {
    let appointment = appointment_id(&request.appointment_id)?;
    let base = request.base.try_into()?;
    let reason = reason_of(request.change_reason)?;
    run(connection, |repo, access| {
        repo.mark_no_show_appointment(
            access,
            appointment,
            base,
            reason,
            &UuidV7Generator,
            &SystemClock,
        )
    })
}

#[tauri::command]
pub fn appointments_create_appointment(
    connection: State<Mutex<Connection>>,
    request: CreateAppointmentRequest,
) -> super::super::application::AppResult<LocalMutationResult> {
    create_appointment(&mut *connection.lock().map_err(|_| locked())?, request)
}

#[tauri::command]
pub fn appointments_confirm_appointment(
    connection: State<Mutex<Connection>>,
    request: ConfirmAppointmentRequest,
) -> super::super::application::AppResult<LocalMutationResult> {
    confirm_appointment(&mut *connection.lock().map_err(|_| locked())?, request)
}

#[tauri::command]
pub fn appointments_reschedule_appointment(
    connection: State<Mutex<Connection>>,
    request: RescheduleAppointmentRequest,
) -> super::super::application::AppResult<LocalMutationResult> {
    reschedule_appointment(&mut *connection.lock().map_err(|_| locked())?, request)
}

#[tauri::command]
pub fn appointments_cancel_appointment(
    connection: State<Mutex<Connection>>,
    request: LifecycleAppointmentRequest,
) -> super::super::application::AppResult<LocalMutationResult> {
    cancel_appointment(&mut *connection.lock().map_err(|_| locked())?, request)
}

#[tauri::command]
pub fn appointments_complete_appointment(
    connection: State<Mutex<Connection>>,
    request: LifecycleAppointmentRequest,
) -> super::super::application::AppResult<LocalMutationResult> {
    complete_appointment(&mut *connection.lock().map_err(|_| locked())?, request)
}

#[tauri::command]
pub fn appointments_mark_no_show(
    connection: State<Mutex<Connection>>,
    request: LifecycleAppointmentRequest,
) -> super::super::application::AppResult<LocalMutationResult> {
    mark_no_show(&mut *connection.lock().map_err(|_| locked())?, request)
}
