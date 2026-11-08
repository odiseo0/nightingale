use super::{
    adapters::{api::SyncAPI, control::SqliteSyncControlStore},
    application::{CursorStore, RemoteProjectionStore},
    dispatch::{DispatchFailure, OutboxDispatcher},
    domain::StoredCursor,
    download::{ChangeDownloader, ensure_access},
    remote::{ChangeKind, EntityType, RemoteChange, RemoteProjector, SyncChange, storage_error},
};
use crate::{
    components::devices::activation::DeviceCredential,
    platform::http::{APICode, APIFailure, APITransport},
    shared_kernel::{
        Clock, NightingaleError,
        id::{AccountId, OperationContext, OperationId},
    },
};
use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Mutex;

const PAGE_SIZE: i64 = 100;
// This value never goes to HTTP. Save it with the cursor commit so a restart
// after installation retries DELETE without installing the same view again.
pub(crate) const INSTALLED: i64 = i64::MAX;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncWindow {
    pub starts_at: String,
    pub ends_at: String,
    pub appointments_from: String,
    pub appointments_until: String,
    pub time_zone: String,
}

fn contract_version() -> i64 {
    1
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub snapshot_id: OperationId,
    pub account_id: AccountId,
    #[serde(default = "contract_version")]
    pub contract_version: i64,
    pub model_revision: String,
    pub window: SyncWindow,
    pub base_cursor: String,
    pub expires_at: String,
    pub item_count: i64,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotItem {
    pub entity_type: EntityType,
    pub entity_key: Value,
    pub entity_version: i64,
    pub state: Value,
}

impl SnapshotItem {
    fn change(&self) -> SyncChange {
        SyncChange {
            entity_type: self.entity_type,
            entity_key: self.entity_key.clone(),
            entity_version: self.entity_version,
            state: self.state.clone(),
            change_kind: ChangeKind::Upsert,
        }
    }
}

#[derive(Deserialize)]
pub struct SnapshotPage {
    pub items: Vec<SnapshotItem>,
    pub total: i64,
}

pub(crate) struct Recovery {
    pub(crate) snapshot: Snapshot,
    epoch: i64,
    pub(crate) next_page: i64,
}

pub enum SnapshotMode {
    Resume,
    Restart,
}

pub enum SnapshotOutcome {
    Installed { cleanup_pending: bool },
    RecoveryRequired,
}

pub struct SnapshotRecovery<'a> {
    pub http: &'a APITransport,
    pub database: &'a Mutex<Connection>,
}

fn local(error: NightingaleError) -> DispatchFailure {
    DispatchFailure::Local(error)
}
fn sql(error: rusqlite::Error) -> DispatchFailure {
    local(storage_error(error))
}
fn invalid() -> DispatchFailure {
    DispatchFailure::Remote(APIFailure::local(APICode::InvalidResponse))
}

pub(crate) fn encode(value: &impl Serialize) -> Result<String, DispatchFailure> {
    serde_json::to_string(value).map_err(|_| invalid())
}

pub(crate) fn load(
    c: &Connection,
    scope: &OperationContext,
) -> Result<Option<Recovery>, DispatchFailure> {
    let record = c.query_row(
        "SELECT snapshot_id, base_cursor, window, expires_at, item_count, next_page, permission_epoch FROM sync_snapshot_recovery WHERE account_id = ?1 AND user_id = ?2 AND device_id = ?3",
        params![
            scope.account_id.as_str(),
            scope.user_id.as_str(),
            scope.device_id.as_str()
        ], |r| Ok(
            (
                r.get::<_,String>(0)?,
                r.get::<_,String>(1)?,
                r.get::<_,String>(2)?,
                r.get::<_,String>(3)?,
                r.get::<_,i64>(4)?,
                r.get::<_,i64>(5)?,
                r.get::<_,i64>(6)?
            )
        )).optional().map_err(sql)?;

    record
        .map(
            |(id, base, window, expires, item_count, next_page, epoch)| {
                Ok(Recovery {
                    snapshot: Snapshot {
                        snapshot_id: OperationId::try_from(id).map_err(local)?,
                        account_id: scope.account_id.clone(),
                        contract_version: 1,
                        model_revision: "api-v1.1".into(),
                        window: serde_json::from_str(&window).map_err(|_| invalid())?,
                        base_cursor: base,
                        expires_at: expires,
                        item_count,
                    },
                    epoch,
                    next_page,
                })
            },
        )
        .transpose()
}

fn discard(
    tx: &Transaction,
    scope: &OperationContext,
    id: Option<&OperationId>,
) -> Result<(), DispatchFailure> {
    tx.execute(
        "DELETE FROM sync_snapshot_recovery WHERE account_id = ?1 AND user_id = ?2 AND device_id = ?3 AND (?4 IS NULL OR snapshot_id = ?4)",
        params![
            scope.account_id.as_str(),
            scope.user_id.as_str(),
            scope.device_id.as_str(),
            id.map(OperationId::as_str)
        ]).map_err(sql)?;

    Ok(())
}

fn validate(snapshot: &Snapshot, scope: &OperationContext) -> Result<(), DispatchFailure> {
    if snapshot.account_id != scope.account_id
        || snapshot.contract_version != 1
        || snapshot.model_revision != "api-v1.1"
        || snapshot.item_count < 0
        || snapshot.base_cursor.is_empty()
        || snapshot.expires_at.is_empty()
    {
        return Err(invalid());
    }

    Ok(())
}

impl SnapshotRecovery<'_> {
    fn local<T>(
        &self,
        work: impl FnOnce(&mut Connection) -> Result<T, DispatchFailure>,
    ) -> Result<T, DispatchFailure> {
        ChangeDownloader {
            http: self.http,
            database: self.database,
        }
        .local(work)
    }

    fn discard(
        &self,
        scope: &OperationContext,
        id: Option<&OperationId>,
    ) -> Result<(), DispatchFailure> {
        self.local(|c| {
            let tx = c
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(sql)?;
            discard(&tx, scope, id)?;
            tx.commit().map_err(sql)
        })
    }

    /// Resumes one durable snapshot. Restart discards only staging.
    /// API checks expiration on pages and on the final metadata read.
    pub async fn rebuild(
        &self,
        credential: &DeviceCredential,
        scope: &OperationContext,
        clock: &impl Clock,
        mode: SnapshotMode,
        projector: &impl RemoteProjector,
    ) -> Result<SnapshotOutcome, DispatchFailure> {
        let auth = OutboxDispatcher {
            http: self.http,
            database: self.database,
        };

        match auth.authorize(credential, scope, clock).await {
            Err(DispatchFailure::AccessChanged) => return Ok(SnapshotOutcome::RecoveryRequired),
            result => {
                result?;
            }
        }

        if matches!(mode, SnapshotMode::Restart) {
            self.discard(scope, None)?;
        }

        let mut recovery = match self.local(|c| load(c, scope))? {
            Some(record) => record,
            None => {
                let snapshot = match SyncAPI(self.http)
                    .create_snapshot(&credential.token, &scope.account_id)
                    .await
                {
                    Ok(response) => response.value,
                    Err(error) => {
                        auth.deny(scope, &error)?;
                        return Err(DispatchFailure::Remote(error));
                    }
                };

                validate(&snapshot, scope)?;
                self.local(|c| {
                    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate).map_err(sql)?;
                    ensure_access(&tx,scope)?;

                    if load(&tx,scope)?.is_some() {
                        return Err(DispatchFailure::AccessChanged);
                    }

                    tx.execute("INSERT INTO sync_snapshot_recovery (account_id,user_id,device_id,permission_epoch,snapshot_id,base_cursor,window,expires_at,item_count,next_page) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,1)",params![scope.account_id.as_str(),scope.user_id.as_str(),scope.device_id.as_str(),scope.permission_epoch,snapshot.snapshot_id.as_str(),snapshot.base_cursor,encode(&snapshot.window)?,snapshot.expires_at,snapshot.item_count]).map_err(sql)?;
                    tx.commit().map_err(sql)
                })?;

                Recovery {
                    snapshot,
                    epoch: scope.permission_epoch,
                    next_page: 1,
                }
            }
        };

        if recovery.epoch != scope.permission_epoch {
            return Ok(SnapshotOutcome::RecoveryRequired);
        }

        let snapshot = &recovery.snapshot;

        if recovery.next_page != INSTALLED {
            let pages =
                snapshot.item_count / PAGE_SIZE + i64::from(snapshot.item_count % PAGE_SIZE != 0);

            while recovery.next_page <= pages {
                let page = match SyncAPI(self.http)
                    .snapshot_items(
                        &credential.token,
                        &scope.account_id,
                        &snapshot.snapshot_id,
                        recovery.next_page,
                    )
                    .await
                {
                    Ok(response) => response.value,
                    Err(error) => return self.failed(scope, &snapshot.snapshot_id, &auth, error),
                };

                self.stage(scope, &recovery, &page)?;
                recovery.next_page += 1;
            }

            let current = match SyncAPI(self.http)
                .snapshot(&credential.token, &scope.account_id, &snapshot.snapshot_id)
                .await
            {
                Ok(response) => response.value,
                Err(error) => return self.failed(scope, &snapshot.snapshot_id, &auth, error),
            };

            validate(&current, scope)?;

            if current != *snapshot {
                return Err(invalid());
            }

            self.install(scope, &recovery, projector)?;
        }

        // Release errors cannot undo an installed view or install it twice.
        let released = match SyncAPI(self.http)
            .release_snapshot(&credential.token, &scope.account_id, &snapshot.snapshot_id)
            .await
        {
            Ok(_) => true,
            Err(error) => {
                auth.deny(scope, &error)?;
                error.status == Some(404)
                    || matches!(
                        error.code,
                        APICode::SnapshotExpired | APICode::SnapshotScopeChanged
                    )
            }
        };

        if released {
            self.discard(scope, Some(&snapshot.snapshot_id))?;
        }

        Ok(SnapshotOutcome::Installed {
            cleanup_pending: !released,
        })
    }

    fn failed(
        &self,
        scope: &OperationContext,
        id: &OperationId,
        auth: &OutboxDispatcher,
        error: APIFailure,
    ) -> Result<SnapshotOutcome, DispatchFailure> {
        auth.deny(scope, &error)?;

        if error.status == Some(404)
            || matches!(
                error.code,
                APICode::SnapshotExpired | APICode::SnapshotScopeChanged
            )
        {
            self.discard(scope, Some(id))?;

            return Ok(SnapshotOutcome::RecoveryRequired);
        }

        Err(DispatchFailure::Remote(error))
    }

    pub(crate) fn stage(
        &self,
        scope: &OperationContext,
        recovery: &Recovery,
        page: &SnapshotPage,
    ) -> Result<(), DispatchFailure> {
        let snapshot = &recovery.snapshot;
        let offset = (recovery.next_page - 1)
            .checked_mul(PAGE_SIZE)
            .ok_or_else(invalid)?;
        let expected = (snapshot.item_count - offset).clamp(0, PAGE_SIZE);

        if page.total != snapshot.item_count || page.items.len() as i64 != expected {
            return Err(invalid());
        }

        self.local(|c| {
            let tx = c.transaction_with_behavior(TransactionBehavior::Immediate).map_err(sql)?;
            ensure_access(&tx,scope)?;
            let current = load(&tx,scope)?.ok_or(DispatchFailure::AccessChanged)?;

            if current.snapshot
                != *snapshot
                || current.epoch!=recovery.epoch
                || current.next_page!=recovery.next_page
            {
                return Err(DispatchFailure::AccessChanged);
            }

            for item in &page.items {
                item.change().translate(scope).map_err(local)?;
                tx.execute(
                    "INSERT INTO sync_snapshot_items (
                            account_id, user_id, device_id, snapshot_id, entity_type, entity_key, entity_version, state
                        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                    params![
                        scope.account_id.as_str(),
                        scope.user_id.as_str(),
                        scope.device_id.as_str(),
                        snapshot.snapshot_id.as_str(),
                        item.entity_type.as_str(),
                        encode(&item.entity_key)?,
                        item.entity_version,encode(&item.state)?
                    ]).map_err(sql)?;
            }

            tx.execute("UPDATE sync_snapshot_recovery SET next_page=next_page+1 WHERE account_id=?1 AND user_id=?2 AND device_id=?3",params![scope.account_id.as_str(),scope.user_id.as_str(),scope.device_id.as_str()]).map_err(sql)?;
            tx.commit().map_err(sql)
        })
    }

    pub(crate) fn install(
        &self,
        scope: &OperationContext,
        recovery: &Recovery,
        projector: &impl RemoteProjector,
    ) -> Result<(), DispatchFailure> {
        self.local(|c| {
            let tx = c.transaction_with_behavior(TransactionBehavior::Immediate).map_err(sql)?;
            ensure_access(&tx,scope)?;
            let current = load(&tx,scope)?.ok_or(DispatchFailure::AccessChanged)?;

            if current.snapshot != recovery.snapshot || current.epoch != scope.permission_epoch || current.next_page != recovery.next_page {
                return Err(DispatchFailure::AccessChanged);
            }

            let items = {
                let mut query = tx.prepare(
                    "SELECT entity_type, entity_key, entity_version, state FROM sync_snapshot_items WHERE account_id = ?1 AND user_id = ?2 AND device_id = ?3 AND snapshot_id = ?4 ORDER BY rowid"
                ).map_err(sql)?;
                let rows = query.query_map(
                    params![
                        scope.account_id.as_str(),
                        scope.user_id.as_str(),
                        scope.device_id.as_str(),
                        recovery.snapshot.snapshot_id.as_str()
                    ], |r| Ok(
                        (
                            r.get::<_,String>(0)?,
                            r.get::<_,String>(1)?,
                            r.get::<_,i64>(2)?,
                            r.get::<_,String>(3)?
                        )
                    )).map_err(sql)?;
                let mut items = Vec::new();

                for row in rows {
                    let (entity_type, key, entity_version,state) = row.map_err(sql)?;

                    items.push(SnapshotItem {
                        entity_type:serde_json::from_value(Value::String(entity_type)).map_err(|_| invalid())?,
                        entity_key:serde_json::from_str(&key).map_err(|_| invalid())?,
                        entity_version,
                        state:serde_json::from_str(&state).map_err(|_| invalid())?
                    });
                }

                items
            };

            let pages = recovery.snapshot.item_count / PAGE_SIZE + i64::from(recovery.snapshot.item_count % PAGE_SIZE != 0);

            if items.len() as i64
                != recovery.snapshot.item_count
                || recovery.next_page
                != pages + 1
            {
                    return Err(invalid());
            }

            projector.reset(&tx,scope).map_err(local)?;

            for table in ["sync_remote_items","sync_inbox"] {
                tx.execute(&format!("DELETE FROM {table} WHERE account_id =?1 AND user_id =?2 AND device_id =?3"),
                    params![
                        scope.account_id.as_str(),
                        scope.user_id.as_str(),
                        scope.device_id.as_str()
                    ]
                ).map_err(sql)?;
            }

            for item in items {
                let change=item.change().translate(scope).map_err(local)?;

                if let RemoteChange::Upsert { item,.. } = &change { SqliteSyncControlStore.upsert(&tx, scope, item).map_err(local)?; }

                projector.project(&tx,scope,&change).map_err(local)?;
            }

            SqliteSyncControlStore.save_cursor(
                &tx,
                scope,
                &StoredCursor {
                    cursor:recovery.snapshot.base_cursor.clone(),
                    window:serde_json::to_value(&recovery.snapshot.window).map_err(|_| invalid())?
                }
            ).map_err(local)?;

            tx.execute(
                "UPDATE sync_snapshot_recovery SET next_page = ?1 WHERE account_id = ?2 AND user_id = ?3 AND device_id = ?4",
                params![
                    INSTALLED,
                    scope.account_id.as_str(),
                    scope.user_id.as_str(),
                    scope.device_id.as_str()
                ]
            ).map_err(sql)?;

            tx.commit().map_err(sql)
        })
    }
}
