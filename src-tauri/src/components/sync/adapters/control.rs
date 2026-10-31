use crate::components::sync::application::{CursorStore, InboxStore, RemoteProjectionStore};
use crate::components::sync::domain::{RemoteItem, StoredCursor};
use crate::shared_kernel::id::{OperationContext, OperationId};
use crate::shared_kernel::{NightingaleError, Result};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde_json::Value;

pub struct SqliteSyncControlStore;

impl SqliteSyncControlStore {
    pub fn snapshot_pending(&self, c: &Connection, scope: &OperationContext) -> Result<bool> {
        c.query_row("SELECT EXISTS(SELECT 1 FROM sync_snapshot_recovery WHERE account_id=?1 AND user_id=?2 AND device_id=?3)",
            params![scope.account_id.as_str(),scope.user_id.as_str(),scope.device_id.as_str()],|r|r.get(0)).map_err(storage_error)
    }
}

fn storage_error(error: impl std::fmt::Display) -> NightingaleError {
    NightingaleError::internal(error.to_string())
}

impl CursorStore for SqliteSyncControlStore {
    fn save_cursor(
        &self,
        tx: &Transaction,
        scope: &OperationContext,
        cursor: &StoredCursor,
    ) -> Result<()> {
        tx.execute("INSERT INTO sync_cursor (account_id,user_id,device_id,permission_epoch,cursor,window) VALUES (?1,?2,?3,?4,?5,?6)
            ON CONFLICT(account_id,user_id,device_id) DO UPDATE SET permission_epoch=excluded.permission_epoch,cursor=excluded.cursor,window=excluded.window",
            params![scope.account_id.as_str(),scope.user_id.as_str(),scope.device_id.as_str(),scope.permission_epoch,cursor.cursor,serde_json::to_string(&cursor.window).map_err(storage_error)?]).map_err(storage_error)?;

        Ok(())
    }

    fn cursor(
        &self,
        connection: &Connection,
        scope: &OperationContext,
    ) -> Result<Option<StoredCursor>> {
        let record = connection.query_row("SELECT cursor,window FROM sync_cursor WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND permission_epoch=?4",
            params![scope.account_id.as_str(),scope.user_id.as_str(),scope.device_id.as_str(),scope.permission_epoch],
            |row| Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?))).optional().map_err(storage_error)?;

        record
            .map(|(cursor, window)| {
                Ok(StoredCursor {
                    cursor,
                    window: serde_json::from_str(&window).map_err(storage_error)?,
                })
            })
            .transpose()
    }
}

impl InboxStore for SqliteSyncControlStore {
    fn record_group(
        &self,
        tx: &Transaction,
        scope: &OperationContext,
        group: &OperationId,
        operation: &OperationId,
    ) -> Result<bool> {
        let count = tx.execute("INSERT INTO sync_inbox (account_id,user_id,device_id,permission_epoch,group_id,operation_id) VALUES (?1,?2,?3,?4,?5,?6) ON CONFLICT DO NOTHING",
            params![scope.account_id.as_str(),scope.user_id.as_str(),scope.device_id.as_str(),scope.permission_epoch,group.as_str(),operation.as_str()]).map_err(storage_error)?;

        Ok(count == 1)
    }

    fn contains_group(
        &self,
        connection: &Connection,
        scope: &OperationContext,
        group: &OperationId,
        operation: &OperationId,
    ) -> Result<bool> {
        connection.query_row("SELECT EXISTS(SELECT 1 FROM sync_inbox WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND permission_epoch=?4 AND (group_id=?5 OR operation_id=?6))",
            params![scope.account_id.as_str(),scope.user_id.as_str(),scope.device_id.as_str(),scope.permission_epoch,group.as_str(),operation.as_str()], |row| row.get(0)).map_err(storage_error)
    }
}

impl RemoteProjectionStore for SqliteSyncControlStore {
    fn upsert(&self, tx: &Transaction, scope: &OperationContext, item: &RemoteItem) -> Result<()> {
        tx.execute("INSERT INTO sync_remote_items (account_id,user_id,device_id,permission_epoch,entity_type,entity_key,entity_version,state) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
            ON CONFLICT(account_id,user_id,device_id,permission_epoch,entity_type,entity_key) DO UPDATE SET entity_version=excluded.entity_version,state=excluded.state
            WHERE excluded.entity_version >= sync_remote_items.entity_version",
            params![scope.account_id.as_str(),scope.user_id.as_str(),scope.device_id.as_str(),scope.permission_epoch,item.entity_type,serde_json::to_string(&item.entity_key).map_err(storage_error)?,item.entity_version,serde_json::to_string(&item.state).map_err(storage_error)?]).map_err(storage_error)?;

        Ok(())
    }

    fn item(
        &self,
        connection: &Connection,
        scope: &OperationContext,
        entity_type: &str,
        key: &Value,
    ) -> Result<Option<RemoteItem>> {
        let record = connection.query_row("SELECT entity_version,state FROM sync_remote_items WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND permission_epoch=?4 AND entity_type=?5 AND entity_key=?6",
            params![scope.account_id.as_str(),scope.user_id.as_str(),scope.device_id.as_str(),scope.permission_epoch,entity_type,serde_json::to_string(key).map_err(storage_error)?],
            |row| Ok((row.get::<_,i64>(0)?,row.get::<_,String>(1)?))).optional().map_err(storage_error)?;

        record
            .map(|(entity_version, state)| {
                Ok(RemoteItem {
                    entity_type: entity_type.to_owned(),
                    entity_key: key.clone(),
                    entity_version,
                    state: serde_json::from_str(&state).map_err(storage_error)?,
                })
            })
            .transpose()
    }

    fn remove(
        &self,
        tx: &Transaction,
        scope: &OperationContext,
        entity_type: &str,
        key: &Value,
    ) -> Result<()> {
        tx.execute("DELETE FROM sync_remote_items WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND permission_epoch=?4 AND entity_type=?5 AND entity_key=?6",
            params![scope.account_id.as_str(),scope.user_id.as_str(),scope.device_id.as_str(),scope.permission_epoch,entity_type,serde_json::to_string(key).map_err(storage_error)?]).map_err(storage_error)?;

        Ok(())
    }
}
