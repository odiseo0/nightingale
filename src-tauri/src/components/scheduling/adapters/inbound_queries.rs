//! Query commands: fused lists of rules, exceptions and blocks.
//! Outputs separate accepted, provisional and operation state; React never
//! receives SQL, tokens or raw transport state.
use super::super::application::*;
use super::inbound_rules::{locked, run};
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::State;

pub(crate) fn list_rules(connection: &mut Connection) -> Result<Vec<EntityRow<RuleState>>> {
    run(connection, |repo, access| repo.rule_rows_list(access))
}

pub(crate) fn list_exceptions(
    connection: &mut Connection,
) -> Result<Vec<EntityRow<ExceptionState>>> {
    run(connection, |repo, access| repo.exception_rows_list(access))
}

pub(crate) fn list_blocks(connection: &mut Connection) -> Result<Vec<EntityRow<BlockState>>> {
    run(connection, |repo, access| repo.block_rows_list(access))
}

#[tauri::command]
pub fn scheduling_list_rules(
    connection: State<Mutex<Connection>>,
) -> Result<Vec<EntityRow<RuleState>>> {
    list_rules(&mut *connection.lock().map_err(|_| locked())?)
}

#[tauri::command]
pub fn scheduling_list_exceptions(
    connection: State<Mutex<Connection>>,
) -> Result<Vec<EntityRow<ExceptionState>>> {
    list_exceptions(&mut *connection.lock().map_err(|_| locked())?)
}

#[tauri::command]
pub fn scheduling_list_blocks(
    connection: State<Mutex<Connection>>,
) -> Result<Vec<EntityRow<BlockState>>> {
    list_blocks(&mut *connection.lock().map_err(|_| locked())?)
}
