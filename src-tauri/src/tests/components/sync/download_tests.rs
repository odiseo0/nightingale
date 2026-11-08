use crate::components::sync::download::*;
use crate::components::{
    devices::activation::DeviceCredential,
    identity::{
        domain::AccessStatus,
        repository::{AccessStore, SqliteAccessStore},
    },
    sync::{
        adapters::{api::ChangePage, control::SqliteSyncControlStore},
        application::CursorStore,
        domain::StoredCursor,
    },
};
use crate::platform::http::APITransport;
use crate::shared_kernel::id::OperationContext;
use crate::{
    components::{
        devices::activation::DeviceCredentials,
        identity::domain::{AccessRole, AccountAccess},
        sync::remote::ChangeGroup,
    },
    platform::{
        config::APIConfig,
        secrets::{CredentialKey, CredentialKind, Secret, SecretStore, testing::MemorySecretStore},
        sqlite::open,
        test_api::TestApi,
    },
    shared_kernel::SystemClock,
};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::sync::Mutex;
use std::{path::PathBuf, sync::Arc};

pub(crate) fn scope() -> OperationContext {
    serde_json::from_value(json!({"accountId":"019a1234-0000-7000-8000-000000000001", "userId":"019a1234-0000-7000-8000-000000000002", "deviceId":"019a1234-0000-7000-8000-000000000003", "permissionEpoch":1})).unwrap()
}

pub(crate) fn group() -> ChangeGroup {
    serde_json::from_str(include_str!("fixtures/remote-group.json")).unwrap()
}

pub(crate) fn access(epoch: i64, status: &str) -> Value {
    let s = scope();
    json!({"accountId":s.account_id,"userId":s.user_id,"deviceId":s.device_id,"permissionEpoch":epoch,"status":status,"role":"owner","allowedActions":["RegisterPatient"]})
}

pub(crate) struct Database {
    pub data: Arc<Mutex<Connection>>,
    pub path: PathBuf,
}

impl Database {
    pub fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "nightingale-download-{}.sqlite",
            uuid::Uuid::now_v7()
        ));
        let mut c = open(&path).unwrap();
        let tx = c.transaction().unwrap();
        SqliteAccessStore
            .save_access(
                &tx,
                &AccountAccess {
                    context: scope(),
                    role: AccessRole::Owner,
                    status: AccessStatus::Active,
                    allowed_actions: vec!["RegisterPatient".into()],
                    checked_at_millis: 1,
                },
            )
            .unwrap();
        SqliteSyncControlStore
            .save_cursor(&tx, &scope(), &cursor("c+/?&= x"))
            .unwrap();
        tx.commit().unwrap();
        Self {
            data: Arc::new(Mutex::new(c)),
            path,
        }
    }

    pub fn cursor(&self) -> StoredCursor {
        SqliteSyncControlStore
            .cursor(&self.data.lock().unwrap(), &scope())
            .unwrap()
            .unwrap()
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        *self.data.lock().unwrap_or_else(|error| error.into_inner()) =
            Connection::open_in_memory().unwrap();
        std::fs::remove_file(&self.path).unwrap();
    }
}

pub(crate) fn cursor(value: &str) -> StoredCursor {
    StoredCursor {
        cursor: value.into(),
        window: json!({"appointmentsFrom":"2026-10-01","appointmentsUntil":"2026-11-01","startsAt":"2026-10-01T00:00:00Z","endsAt":"2026-11-01T00:00:00Z","timeZone":"UTC"}),
    }
}

pub(crate) fn credential(config: &APIConfig) -> DeviceCredential {
    let store = MemorySecretStore::default();
    let s = scope();
    for (device, kind, value) in [
        (
            None,
            CredentialKind::APIDeviceState,
            json!({"status":"issued","device_id":s.device_id,"user_id":s.user_id}).to_string(),
        ),
        (
            Some(&s.device_id),
            CredentialKind::APIDevice,
            "synthetic-device-token".into(),
        ),
    ] {
        let key = CredentialKey::new(
            config,
            "download",
            ("https://issuer.example/", "subject"),
            None,
            device,
            kind,
        )
        .unwrap();
        store.write(&key, &Secret::new(value).unwrap()).unwrap();
    }
    DeviceCredentials {
        config,
        profile: "download",
        identity: ("https://issuer.example/", "subject"),
        secrets: &store,
    }
    .load()
    .unwrap()
    .unwrap()
}

#[test]
fn multiple_pages_empty_progress_replay_and_window_remove_commit_without_http_lock() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        let data = db.data.clone();
        let mut page = 0;
        let api = TestApi::new(6, move |request| {
            assert!(data.try_lock().is_ok());
            if request.lines().next().unwrap().contains("/access ") {
                return (200, access(1, "active"));
            }
            assert!(request.contains("authorization: Bearer synthetic-device-token"));
            page += 1;
            let groups = match page {
                1 => {
                    assert!(request.contains("cursor=c%2B%2F%3F%26%3D+x"));
                    vec![group()]
                }
                2 => {
                    assert!(request.contains("cursor=second"));
                    vec![]
                }
                _ => {
                    assert!(request.contains("cursor=third"));
                    let mut removal = group();
                    removal.group_id =
                        serde_json::from_value(json!("019a1234-0000-7000-8000-000000000200"))
                            .unwrap();
                    removal.operation_id = removal.group_id.clone();
                    removal.changes.retain(|x| {
                        matches!(
                            x.entity_type,
                            crate::components::sync::remote::EntityType::Appointment
                                | crate::components::sync::remote::EntityType::AppointmentHistory
                        )
                    });
                    for x in &mut removal.changes {
                        x.change_kind = crate::components::sync::remote::ChangeKind::Remove;
                        x.state = Value::Null;
                    }
                    vec![group(), removal]
                }
            };
            (
                200,
                json!({"groups":groups,"nextCursor":if page==1 {"second"} else if page==2 {"third"} else {"done"},"hasMore":page<3}),
            )
        });
        let config = APIConfig::parse(&api.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
        let outcome = ChangeDownloader {
            http: &http,
            database: &db.data,
        }
        .download(&credential(&config), &scope(), &SystemClock, &())
        .await
        .unwrap();
        assert!(matches!(outcome, DownloadOutcome::Complete { pages: 3 }));
        assert_eq!(db.cursor(), cursor("done"));
        let c = db.data.lock().unwrap();
        assert_eq!(
            c.query_row("SELECT COUNT(*) FROM sync_remote_items", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            14
        );
        drop(c);
        let c = open(&db.path).unwrap();
        assert_eq!(
            SqliteSyncControlStore.cursor(&c, &scope()).unwrap(),
            Some(cursor("done"))
        );
        api.finish();
    });
}

#[test]
fn cursor_write_failure_rolls_back_page_inbox_and_projection_and_retry_works() {
    let db = Database::new();
    let http = APITransport::new(APIConfig::parse("http://127.0.0.1:1/").unwrap()).unwrap();
    let downloader = ChangeDownloader {
        http: &http,
        database: &db.data,
    };
    let previous = db.cursor();
    let page = ChangePage {
        groups: vec![group()],
        next_cursor: "next".into(),
        has_more: false,
    };
    db.data.lock().unwrap().execute_batch("CREATE TRIGGER fail_cursor BEFORE UPDATE ON sync_cursor BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
    assert!(
        downloader
            .apply_page(&scope(), &previous, &page, &())
            .is_err()
    );
    assert_eq!(db.cursor(), previous);
    let c = db.data.lock().unwrap();
    for table in ["sync_inbox", "sync_remote_items"] {
        assert_eq!(
            c.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    c.execute("DROP TRIGGER fail_cursor", []).unwrap();
    drop(c);
    downloader
        .apply_page(&scope(), &previous, &page, &())
        .unwrap();
    assert_eq!(db.cursor(), cursor("next"));
    assert!(
        downloader
            .apply_page(&scope(), &previous, &page, &())
            .is_err()
    );
    assert_eq!(db.cursor(), cursor("next"));
}

#[test]
fn expired_cursor_missing_cursor_and_new_epoch_require_recovery_without_advancing() {
    tauri::async_runtime::block_on(async {
        for case in ["expired", "missing", "epoch"] {
            let db = Database::new();
            if case == "missing" {
                db.data
                    .lock()
                    .unwrap()
                    .execute("DELETE FROM sync_cursor", [])
                    .unwrap();
            }
            let api = TestApi::new(if case == "expired" { 2 } else { 1 }, move |request| {
                if request.contains("/access ") {
                    (200, access(if case == "epoch" { 2 } else { 1 }, "active"))
                } else {
                    (
                        410,
                        json!({"code":"CURSOR_EXPIRED","message":"fixture","correlationId":"019a1234-0000-7000-8000-000000000099","details":[]}),
                    )
                }
            });
            let config = APIConfig::parse(&api.url).unwrap();
            let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
            let result = ChangeDownloader {
                http: &http,
                database: &db.data,
            }
            .download(&credential(&config), &scope(), &SystemClock, &())
            .await
            .unwrap();
            assert!(matches!(result, DownloadOutcome::RecoveryRequired));
            if case != "missing" {
                assert_eq!(db.cursor(), cursor("c+/?&= x"));
            }
            api.finish();
        }
    });
}

#[test]
fn denial_and_permission_change_during_http_prevent_page_commit() {
    tauri::async_runtime::block_on(async {
        for status in [401, 403, 200] {
            let db = Database::new();
            let data = db.data.clone();
            let api = TestApi::new(2, move |request| {
                if request.contains("/access ") {
                    return (200, access(1, "active"));
                }
                if status == 200 {
                    let mut c = data.lock().unwrap();
                    let tx = c.transaction().unwrap();
                    SqliteAccessStore
                        .deny_cached_access(
                            &tx,
                            &scope().user_id,
                            &scope().device_id,
                            Some(&scope().account_id),
                        )
                        .unwrap();
                    tx.commit().unwrap();
                    (
                        200,
                        json!({"groups":[group()],"nextCursor":"bad","hasMore":false}),
                    )
                } else {
                    (
                        status,
                        json!({"code":if status==401 {"AUTHENTICATION_REQUIRED"} else {"PERMISSION_DENIED"},"message":"fixture","correlationId":"019a1234-0000-7000-8000-000000000099","details":[]}),
                    )
                }
            });
            let config = APIConfig::parse(&api.url).unwrap();
            let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
            assert!(
                ChangeDownloader {
                    http: &http,
                    database: &db.data
                }
                .download(&credential(&config), &scope(), &SystemClock, &())
                .await
                .is_err()
            );
            assert_eq!(db.cursor(), cursor("c+/?&= x"));
            let c = db.data.lock().unwrap();
            assert_eq!(
                c.query_row("SELECT COUNT(*) FROM sync_remote_items", [], |r| r
                    .get::<_, i64>(0))
                    .unwrap(),
                0
            );
            assert_eq!(
                SqliteAccessStore
                    .access(
                        &c,
                        &scope().account_id,
                        &scope().user_id,
                        &scope().device_id
                    )
                    .unwrap()
                    .unwrap()
                    .status,
                AccessStatus::Inactive
            );
            api.finish();
        }
    });
}

#[test]
fn malformed_or_nonprogressing_page_keeps_cursor_for_retry() {
    tauri::async_runtime::block_on(async {
        for missing in [false, true] {
            let db = Database::new();
            let api = TestApi::new(2, move |request| {
                if request.contains("/access ") {
                    (200, access(1, "active"))
                } else if missing {
                    (200, json!({"groups":[],"hasMore":false}))
                } else {
                    (
                        200,
                        json!({"groups":[],"nextCursor":"c+/?&= x","hasMore":true}),
                    )
                }
            });
            let config = APIConfig::parse(&api.url).unwrap();
            let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
            assert!(
                ChangeDownloader {
                    http: &http,
                    database: &db.data
                }
                .download(&credential(&config), &scope(), &SystemClock, &())
                .await
                .is_err()
            );
            assert_eq!(db.cursor(), cursor("c+/?&= x"));
            api.finish();
        }
    });
}
