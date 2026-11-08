use super::{
    adapters::{
        api::{ChangePage, SyncAPI},
        control::SqliteSyncControlStore,
    },
    application::CursorStore,
    dispatch::{DispatchFailure, OutboxDispatcher},
    domain::StoredCursor,
    remote::{RemoteProjector, apply_group, storage_error},
};
use crate::{
    components::{
        devices::activation::DeviceCredential,
        identity::{
            domain::AccessStatus,
            repository::{AccessStore, SqliteAccessStore},
        },
    },
    platform::http::{APICode, APIFailure, APITransport},
    shared_kernel::{Clock, NightingaleError, id::OperationContext},
};
use rusqlite::{Connection, TransactionBehavior};
use std::sync::Mutex;

pub enum DownloadOutcome {
    Complete { pages: usize },
    RecoveryRequired,
}

pub struct ChangeDownloader<'a> {
    pub http: &'a APITransport,
    pub database: &'a Mutex<Connection>,
}

impl ChangeDownloader<'_> {
    pub(super) fn local<T>(
        &self,
        work: impl FnOnce(&mut Connection) -> Result<T, DispatchFailure>,
    ) -> Result<T, DispatchFailure> {
        let mut c = self.database.lock().map_err(|_| {
            DispatchFailure::Local(NightingaleError::internal(
                "almacén de entrada no disponible",
            ))
        })?;

        work(&mut c)
    }

    /// Fetches complete pages without holding SQLite during HTTP.
    pub async fn download(
        &self,
        credential: &DeviceCredential,
        scope: &OperationContext,
        clock: &impl Clock,
        projector: &impl RemoteProjector,
    ) -> Result<DownloadOutcome, DispatchFailure> {
        let access = OutboxDispatcher {
            http: self.http,
            database: self.database,
        };
        let mut pages = 0;

        loop {
            match access.authorize(credential, scope, clock).await {
                Err(DispatchFailure::AccessChanged) => {
                    return Ok(DownloadOutcome::RecoveryRequired);
                }
                result => {
                    result?;
                }
            }

            let Some(cursor) = self.local(|c| {
                SqliteSyncControlStore
                    .cursor(c, scope)
                    .map_err(DispatchFailure::Local)
            })?
            else {
                return Ok(DownloadOutcome::RecoveryRequired);
            };

            let page = match SyncAPI(self.http)
                .changes(&credential.token, &scope.account_id, &cursor.cursor)
                .await
            {
                Ok(response) => response.value,
                Err(error) => {
                    access.deny(scope, &error)?;

                    if error.code == APICode::CursorExpired {
                        return Ok(DownloadOutcome::RecoveryRequired);
                    }

                    return Err(DispatchFailure::Remote(error));
                }
            };

            if page.next_cursor.is_empty() || (page.has_more && page.next_cursor == cursor.cursor) {
                return Err(DispatchFailure::Remote(APIFailure::local(
                    APICode::InvalidResponse,
                )));
            }

            self.apply_page(scope, &cursor, &page, projector)?;
            pages += 1;

            if !page.has_more {
                return Ok(DownloadOutcome::Complete { pages });
            }
        }
    }

    pub(crate) fn apply_page(
        &self,
        scope: &OperationContext,
        previous: &StoredCursor,
        page: &ChangePage,
        projector: &impl RemoteProjector,
    ) -> Result<(), DispatchFailure> {
        self.local(|c| {
            let tx = c
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(storage_error)
                .map_err(DispatchFailure::Local)?;
            ensure_access(&tx, scope)?;

            // Another download or snapshot must not let a stale page move the cursor.
            if SqliteSyncControlStore
                .cursor(&tx, scope)
                .map_err(DispatchFailure::Local)?
                .as_ref()
                != Some(previous)
            {
                return Err(DispatchFailure::AccessChanged);
            }

            for group in &page.groups {
                apply_group(&tx, scope, group, projector).map_err(DispatchFailure::Local)?;
            }

            SqliteSyncControlStore
                .save_cursor(
                    &tx,
                    scope,
                    &StoredCursor {
                        cursor: page.next_cursor.clone(),
                        window: previous.window.clone(),
                    },
                )
                .map_err(DispatchFailure::Local)?;

            tx.commit()
                .map_err(storage_error)
                .map_err(DispatchFailure::Local)
        })
    }
}

pub(super) fn ensure_access(
    c: &Connection,
    scope: &OperationContext,
) -> Result<(), DispatchFailure> {
    let access = SqliteAccessStore
        .access(c, &scope.account_id, &scope.user_id, &scope.device_id)
        .map_err(DispatchFailure::Local)?;

    match access {
        Some(access) if access.status == AccessStatus::Active && access.context == *scope => Ok(()),
        _ => Err(DispatchFailure::AccessChanged),
    }
}
