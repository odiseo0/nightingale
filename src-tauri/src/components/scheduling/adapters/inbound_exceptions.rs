//! Specific local commands for schedule exceptions.
use super::super::application::*;
use super::super::domain::{
    ExceptionKind, ExceptionPatch, ExceptionValue, SchedulingError, Status, parse_local_date,
};
use super::inbound_rules::{EditBaseRequest, known_occupancy, locked, run};
use crate::shared_kernel::{SystemClock, UuidV7Generator};
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateExceptionRequest {
    pub professional_id: String,
    pub local_date: String,
    pub kind: String,
    #[serde(default)]
    pub local_start_minute: Option<i64>,
    #[serde(default)]
    pub local_end_minute: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExceptionPatchRequest {
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub local_start_minute: Option<i64>,
    #[serde(default)]
    pub local_end_minute: Option<i64>,
}

impl TryFrom<ExceptionPatchRequest> for ExceptionPatch {
    type Error = SchedulingError;

    fn try_from(value: ExceptionPatchRequest) -> Result<Self> {
        ExceptionPatch::new(
            value
                .kind
                .as_deref()
                .map(ExceptionKind::try_from)
                .transpose()?,
            value.local_start_minute,
            value.local_end_minute,
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateExceptionRequest {
    pub exception_id: String,
    pub base: EditBaseRequest,
    pub patch: ExceptionPatchRequest,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetExceptionStatusRequest {
    pub exception_id: String,
    pub base: EditBaseRequest,
    pub status: String,
}

fn exception_value(request: &CreateExceptionRequest) -> Result<ExceptionValue> {
    ExceptionValue::new(
        request.professional_id.clone().try_into()?,
        parse_local_date(&request.local_date)?,
        ExceptionKind::try_from(request.kind.as_str())?,
        request.local_start_minute,
        request.local_end_minute,
    )
}

pub(crate) fn create_exception(
    connection: &mut Connection,
    request: CreateExceptionRequest,
) -> Result<LocalMutationResult> {
    let value = exception_value(&request)?;
    let professional = value.professional_id.as_str().to_owned();
    run(connection, |repo, access| {
        let known = known_occupancy(repo.connection, access, &professional)?;
        repo.persist(
            access,
            SchedulingMutation::CreateException { value },
            &UuidV7Generator,
            &SystemClock,
            &known,
        )
    })
}

pub(crate) fn update_exception(
    connection: &mut Connection,
    request: UpdateExceptionRequest,
) -> Result<LocalMutationResult> {
    let base = request.base.try_into()?;
    let patch: ExceptionPatch = request.patch.try_into()?;
    run(connection, |repo, access| {
        let rows = repo.exception_rows_list(access)?;
        let professional = rows
            .iter()
            .find(|row| {
                row.visible()
                    .is_some_and(|state| state.exception_id == request.exception_id)
            })
            .and_then(|row| row.visible())
            .ok_or_else(|| SchedulingError::new("NOT_FOUND", "excepción no disponible"))?
            .professional_id
            .clone();
        let known = known_occupancy(repo.connection, access, &professional)?;
        repo.persist(
            access,
            SchedulingMutation::UpdateException {
                exception_id: request.exception_id.clone(),
                base,
                patch,
            },
            &UuidV7Generator,
            &SystemClock,
            &known,
        )
    })
}

pub(crate) fn set_exception_status(
    connection: &mut Connection,
    request: SetExceptionStatusRequest,
) -> Result<LocalMutationResult> {
    let base = request.base.try_into()?;
    let status = Status::try_from(request.status.as_str())?;
    run(connection, |repo, access| {
        let rows = repo.exception_rows_list(access)?;
        let professional = rows
            .iter()
            .find(|row| {
                row.visible()
                    .is_some_and(|state| state.exception_id == request.exception_id)
            })
            .and_then(|row| row.visible())
            .ok_or_else(|| SchedulingError::new("NOT_FOUND", "excepción no disponible"))?
            .professional_id
            .clone();
        let known = known_occupancy(repo.connection, access, &professional)?;
        repo.persist(
            access,
            SchedulingMutation::SetExceptionStatus {
                exception_id: request.exception_id.clone(),
                base,
                status,
            },
            &UuidV7Generator,
            &SystemClock,
            &known,
        )
    })
}

#[tauri::command]
pub fn scheduling_create_exception(
    connection: State<Mutex<Connection>>,
    request: CreateExceptionRequest,
) -> Result<LocalMutationResult> {
    create_exception(&mut *connection.lock().map_err(|_| locked())?, request)
}

#[tauri::command]
pub fn scheduling_update_exception(
    connection: State<Mutex<Connection>>,
    request: UpdateExceptionRequest,
) -> Result<LocalMutationResult> {
    update_exception(&mut *connection.lock().map_err(|_| locked())?, request)
}

#[tauri::command]
pub fn scheduling_set_exception_status(
    connection: State<Mutex<Connection>>,
    request: SetExceptionStatusRequest,
) -> Result<LocalMutationResult> {
    set_exception_status(&mut *connection.lock().map_err(|_| locked())?, request)
}
