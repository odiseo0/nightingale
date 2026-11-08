//! Specific local commands for schedule blocks.
//! Rust re-authorizes global blocks: only the owner, over the own professional.
use super::super::application::*;
use super::super::domain::{BlockPatch, BlockValue, Instant, ReasonCode, SchedulingError, Status};
use super::inbound_rules::{EditBaseRequest, known_occupancy, locked, run};
use crate::shared_kernel::{SystemClock, UuidV7Generator};
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateBlockRequest {
    pub professional_id: String,
    /// Absent or null means a global block (owner + own professional only).
    #[serde(default)]
    pub account_id: Option<String>,
    pub starts_at: String,
    pub ends_at: String,
    pub reason_code: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BlockPatchRequest {
    #[serde(default)]
    pub starts_at: Option<String>,
    #[serde(default)]
    pub ends_at: Option<String>,
    #[serde(default)]
    pub reason_code: Option<String>,
}

impl TryFrom<BlockPatchRequest> for BlockPatch {
    type Error = SchedulingError;

    fn try_from(value: BlockPatchRequest) -> Result<Self> {
        BlockPatch::new(
            value.starts_at.as_deref().map(Instant::parse).transpose()?,
            value.ends_at.as_deref().map(Instant::parse).transpose()?,
            value
                .reason_code
                .as_deref()
                .map(ReasonCode::try_from)
                .transpose()?,
        )
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateBlockRequest {
    pub block_id: String,
    pub base: EditBaseRequest,
    pub patch: BlockPatchRequest,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetBlockStatusRequest {
    pub block_id: String,
    pub base: EditBaseRequest,
    pub status: String,
}

fn block_value(request: &CreateBlockRequest) -> Result<BlockValue> {
    BlockValue::new(
        request.professional_id.clone().try_into()?,
        request
            .account_id
            .clone()
            .map(TryInto::try_into)
            .transpose()?,
        Instant::parse(&request.starts_at)?,
        Instant::parse(&request.ends_at)?,
        ReasonCode::try_from(request.reason_code.as_str())?,
    )
}

pub(crate) fn create_block(
    connection: &mut Connection,
    request: CreateBlockRequest,
) -> Result<LocalMutationResult> {
    let value = block_value(&request)?;
    let professional = value.professional_id.as_str().to_owned();
    run(connection, |repo, access| {
        let known = known_occupancy(repo.connection, access, &professional)?;
        repo.persist(
            access,
            SchedulingMutation::CreateBlock { value },
            &UuidV7Generator,
            &SystemClock,
            &known,
        )
    })
}

pub(crate) fn update_block(
    connection: &mut Connection,
    request: UpdateBlockRequest,
) -> Result<LocalMutationResult> {
    let base = request.base.try_into()?;
    let patch: BlockPatch = request.patch.try_into()?;
    run(connection, |repo, access| {
        let rows = repo.block_rows_list(access)?;
        let professional = rows
            .iter()
            .find(|row| {
                row.visible()
                    .is_some_and(|state| state.block_id == request.block_id)
            })
            .and_then(|row| row.visible())
            .ok_or_else(|| SchedulingError::new("NOT_FOUND", "bloqueo no disponible"))?
            .professional_id
            .clone();
        let known = known_occupancy(repo.connection, access, &professional)?;
        repo.persist(
            access,
            SchedulingMutation::UpdateBlock {
                block_id: request.block_id.clone(),
                base,
                patch,
            },
            &UuidV7Generator,
            &SystemClock,
            &known,
        )
    })
}

pub(crate) fn set_block_status(
    connection: &mut Connection,
    request: SetBlockStatusRequest,
) -> Result<LocalMutationResult> {
    let base = request.base.try_into()?;
    let status = Status::try_from(request.status.as_str())?;
    run(connection, |repo, access| {
        let rows = repo.block_rows_list(access)?;
        let professional = rows
            .iter()
            .find(|row| {
                row.visible()
                    .is_some_and(|state| state.block_id == request.block_id)
            })
            .and_then(|row| row.visible())
            .ok_or_else(|| SchedulingError::new("NOT_FOUND", "bloqueo no disponible"))?
            .professional_id
            .clone();
        let known = known_occupancy(repo.connection, access, &professional)?;
        repo.persist(
            access,
            SchedulingMutation::SetBlockStatus {
                block_id: request.block_id.clone(),
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
pub fn scheduling_create_block(
    connection: State<Mutex<Connection>>,
    request: CreateBlockRequest,
) -> Result<LocalMutationResult> {
    create_block(&mut *connection.lock().map_err(|_| locked())?, request)
}

#[tauri::command]
pub fn scheduling_update_block(
    connection: State<Mutex<Connection>>,
    request: UpdateBlockRequest,
) -> Result<LocalMutationResult> {
    update_block(&mut *connection.lock().map_err(|_| locked())?, request)
}

#[tauri::command]
pub fn scheduling_set_block_status(
    connection: State<Mutex<Connection>>,
    request: SetBlockStatusRequest,
) -> Result<LocalMutationResult> {
    set_block_status(&mut *connection.lock().map_err(|_| locked())?, request)
}
