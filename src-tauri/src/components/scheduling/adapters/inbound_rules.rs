//! Specific local commands for schedule rules.
//! React cannot supply an account scope, an operation envelope or SQL.
use super::super::application::*;
use super::super::domain::{RulePatch, RuleValue, SchedulingError, Status, parse_local_date};
use super::outbound::{KnownOccupancy, SqliteSchedulingRepository};
use crate::{
    components::{
        appointments::{
            adapters::outbound::SqliteScheduleOccupancyQuery, application::ScheduleOccupancyQuery,
        },
        identity::{domain::AccountAccess, repository::SqliteAccessStore},
    },
    shared_kernel::{SystemClock, UuidV7Generator},
};
use jiff::Timestamp;
use rusqlite::Connection;
use serde::{Deserialize, Deserializer};
use std::sync::Mutex;
use tauri::State;

pub(crate) fn locked() -> SchedulingError {
    SchedulingError::internal("base local no disponible")
}

/// Loads the selected access and hands the connection to one scheduling call.
pub(crate) fn run<T>(
    c: &mut Connection,
    work: impl FnOnce(&mut SqliteSchedulingRepository<'_>, &AccountAccess) -> Result<T>,
) -> Result<T> {
    let access = SqliteAccessStore.selected_access(c)?.ok_or_else(|| {
        SchedulingError::new(
            "AUTHENTICATION_REQUIRED",
            "inicie sesión y elija una cuenta",
        )
    })?;
    let mut repository = SqliteSchedulingRepository { connection: c };

    work(&mut repository, &access)
}

/// Occupancy known for the mutation's professional, read under the same lock.
pub(crate) fn known_occupancy(
    c: &Connection,
    access: &AccountAccess,
    professional: &str,
) -> Result<KnownOccupancy> {
    let intervals = SqliteScheduleOccupancyQuery {
        connection: c,
        scope: &access.context,
    }
    .list_occupancy(professional, Timestamp::MIN, Timestamp::MAX)
    .map_err(SchedulingError::from)?
    .into_iter()
    .map(|interval| (professional.to_owned(), interval))
    .collect();

    Ok(KnownOccupancy(intervals))
}

/// Absence keeps, an explicit null clears, a value sets.
// https://serde.rs/field-attrs.html#deserialize_with
pub(crate) fn change<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> std::result::Result<Option<Option<T>>, D::Error> {
    Option::<T>::deserialize(d).map(Some)
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
    type Error = SchedulingError;

    fn try_from(value: EditBaseRequest) -> Result<Self> {
        match (value.accepted_version, value.operation_id) {
            (Some(version), None) if version > 0 => Ok(EditBase::Accepted { version }),
            (None, Some(operation_id)) => Ok(EditBase::LocalOperation {
                operation_id: operation_id.try_into()?,
            }),
            _ => Err(SchedulingError::validation(
                "la base debe ser una versión aceptada o una operación local",
            )),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateRuleRequest {
    pub professional_id: String,
    pub iso_weekday: i64,
    pub local_start_minute: i64,
    pub local_end_minute: i64,
    pub valid_from: String,
    #[serde(default)]
    pub valid_until: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RulePatchRequest {
    #[serde(default)]
    pub iso_weekday: Option<i64>,
    #[serde(default)]
    pub local_start_minute: Option<i64>,
    #[serde(default)]
    pub local_end_minute: Option<i64>,
    #[serde(default)]
    pub valid_from: Option<String>,
    #[serde(default, deserialize_with = "change")]
    pub valid_until: Option<Option<String>>,
}

impl TryFrom<RulePatchRequest> for RulePatch {
    type Error = SchedulingError;

    fn try_from(value: RulePatchRequest) -> Result<Self> {
        RulePatch::new(
            value.iso_weekday,
            value.local_start_minute,
            value.local_end_minute,
            value
                .valid_from
                .as_deref()
                .map(parse_local_date)
                .transpose()?,
            value
                .valid_until
                .map(|until| until.as_deref().map(parse_local_date).transpose())
                .transpose()?,
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateRuleRequest {
    pub rule_id: String,
    pub base: EditBaseRequest,
    pub patch: RulePatchRequest,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetRuleStatusRequest {
    pub rule_id: String,
    pub base: EditBaseRequest,
    pub status: String,
}

pub(crate) fn create_rule(
    connection: &mut Connection,
    request: CreateRuleRequest,
) -> Result<LocalMutationResult> {
    let value = RuleValue::new(
        request.professional_id.try_into()?,
        request.iso_weekday,
        request.local_start_minute,
        request.local_end_minute,
        parse_local_date(&request.valid_from)?,
        request
            .valid_until
            .as_deref()
            .map(parse_local_date)
            .transpose()?,
    )?;
    let professional = value.professional_id.as_str().to_owned();
    run(connection, |repo, access| {
        let known = known_occupancy(repo.connection, access, &professional)?;
        repo.persist(
            access,
            SchedulingMutation::CreateRule { value },
            &UuidV7Generator,
            &SystemClock,
            &known,
        )
    })
}

pub(crate) fn update_rule(
    connection: &mut Connection,
    request: UpdateRuleRequest,
) -> Result<LocalMutationResult> {
    let base = request.base.try_into()?;
    let patch: RulePatch = request.patch.try_into()?;
    run(connection, |repo, access| {
        let row = repo.rule_row(access, &request.rule_id)?;
        let professional = row
            .visible()
            .ok_or_else(|| SchedulingError::new("NOT_FOUND", "regla no disponible"))?
            .professional_id
            .clone();
        let known = known_occupancy(repo.connection, access, &professional)?;
        repo.persist(
            access,
            SchedulingMutation::UpdateRule {
                rule_id: request.rule_id.clone(),
                base,
                patch,
            },
            &UuidV7Generator,
            &SystemClock,
            &known,
        )
    })
}

pub(crate) fn set_rule_status(
    connection: &mut Connection,
    request: SetRuleStatusRequest,
) -> Result<LocalMutationResult> {
    let base = request.base.try_into()?;
    let status = Status::try_from(request.status.as_str())?;
    run(connection, |repo, access| {
        let row = repo.rule_row(access, &request.rule_id)?;
        let professional = row
            .visible()
            .ok_or_else(|| SchedulingError::new("NOT_FOUND", "regla no disponible"))?
            .professional_id
            .clone();
        let known = known_occupancy(repo.connection, access, &professional)?;
        repo.persist(
            access,
            SchedulingMutation::SetRuleStatus {
                rule_id: request.rule_id.clone(),
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
pub fn scheduling_create_rule(
    connection: State<Mutex<Connection>>,
    request: CreateRuleRequest,
) -> Result<LocalMutationResult> {
    create_rule(&mut *connection.lock().map_err(|_| locked())?, request)
}

#[tauri::command]
pub fn scheduling_update_rule(
    connection: State<Mutex<Connection>>,
    request: UpdateRuleRequest,
) -> Result<LocalMutationResult> {
    update_rule(&mut *connection.lock().map_err(|_| locked())?, request)
}

#[tauri::command]
pub fn scheduling_set_rule_status(
    connection: State<Mutex<Connection>>,
    request: SetRuleStatusRequest,
) -> Result<LocalMutationResult> {
    set_rule_status(&mut *connection.lock().map_err(|_| locked())?, request)
}
