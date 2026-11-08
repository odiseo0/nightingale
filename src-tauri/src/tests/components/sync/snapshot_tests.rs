use crate::components::sync::snapshot::*;
use crate::components::sync::{
    adapters::control::SqliteSyncControlStore,
    download::{ChangeDownloader, DownloadOutcome},
    remote::{RemoteChange, RemoteProjector, apply_groups, storage_error},
};
use crate::platform::http::APITransport;
use crate::shared_kernel::{NightingaleError, id::OperationContext};
use crate::tests::components::sync::download_tests::{
    Database, access, credential, cursor, group, scope,
};
use crate::{
    components::sync::{
        adapters::outbound::SqliteOperationStore,
        application::{CursorStore, OperationStore, record_operation},
        domain::NewOperation,
    },
    platform::{config::APIConfig, sqlite::open, test_api::TestApi},
    shared_kernel::{Result as LocalResult, SystemClock, UuidV7Generator},
};
use rusqlite::params;
use rusqlite::{Connection, Transaction};
use serde_json::Value;
use serde_json::json;
use std::cell::Cell;

fn snapshot(total: i64) -> Snapshot {
    serde_json::from_value(json!({
        "snapshotId":"019a1234-0000-7000-8000-000000000300","accountId":scope().account_id,"contractVersion":1,"modelRevision":"api-v1.1",
        "window":cursor("base").window,"baseCursor":"snapshot-base","expiresAt":"2026-10-30T23:00:00Z","itemCount":total
    })).unwrap()
}

fn items(count: usize) -> Vec<SnapshotItem> {
    let template = group().changes.remove(5);
    (0..count)
        .map(|index| {
            let id = format!("019a1234-0000-7000-8000-{:012}", index + 1000);
            let mut state = template.state.clone();
            state["patientId"] = json!(id);
            SnapshotItem {
                entity_type: template.entity_type,
                entity_key: json!(id),
                entity_version: 1,
                state,
            }
        })
        .collect()
}

fn save(c: &mut Connection, snapshot: &Snapshot) {
    c.execute("INSERT INTO sync_snapshot_recovery (account_id,user_id,device_id,permission_epoch,snapshot_id,base_cursor,window,expires_at,item_count,next_page) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,1)",params![scope().account_id.as_str(),scope().user_id.as_str(),scope().device_id.as_str(),scope().permission_epoch,snapshot.snapshot_id.as_str(),snapshot.base_cursor,encode(&snapshot.window).unwrap(),snapshot.expires_at,snapshot.item_count]).unwrap();
}

fn count(c: &Connection, table: &str) -> i64 {
    c.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

struct Projector {
    calls: Cell<usize>,
    fail: bool,
}
impl RemoteProjector for Projector {
    fn reset(&self, tx: &Transaction, _: &OperationContext) -> LocalResult<()> {
        tx.execute("DELETE FROM test_projection", [])
            .map_err(storage_error)?;
        Ok(())
    }
    fn project(&self, tx: &Transaction, _: &OperationContext, _: &RemoteChange) -> LocalResult<()> {
        self.calls.set(self.calls.get() + 1);
        tx.execute("INSERT INTO test_projection VALUES (1)", [])
            .map_err(storage_error)?;
        if self.fail {
            return Err(NightingaleError::internal("fixture projector"));
        }
        Ok(())
    }
}

#[test]
fn empty_profile_downloads_multiple_snapshot_pages_installs_and_continues_incrementally() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        db.data
            .lock()
            .unwrap()
            .execute("DELETE FROM sync_cursor", [])
            .unwrap();
        let data = db.data.clone();
        let api = TestApi::new(8, move |request| {
            assert!(data.try_lock().is_ok());
            if request.contains("/access ") {
                (200, access(1, "active"))
            } else if request.starts_with("POST ") {
                assert!(request.ends_with("{}"));
                (200, json!(snapshot(101)))
            } else if request.contains("/items?") {
                let all = items(101);
                let page = if request.contains("page=1&shows=100") {
                    &all[..100]
                } else {
                    assert!(request.contains("page=2&shows=100"));
                    &all[100..]
                };
                let c = data.lock().unwrap();
                assert_eq!(count(&c, "sync_remote_items"), 0);
                assert!(
                    SqliteSyncControlStore
                        .cursor(&c, &scope())
                        .unwrap()
                        .is_none()
                );
                (200, json!({"items":page,"total":101}))
            } else if request.starts_with("DELETE ") {
                (204, Value::Null)
            } else if request.contains("/sync/changes?") {
                assert!(request.contains("cursor=snapshot-base"));
                (
                    200,
                    json!({"groups":[group()],"nextCursor":"after-snapshot","hasMore":false}),
                )
            } else {
                (200, json!(snapshot(101)))
            }
        });
        let config = APIConfig::parse(&api.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
        let result = SnapshotRecovery {
            http: &http,
            database: &db.data,
        }
        .rebuild(
            &credential(&config),
            &scope(),
            &SystemClock,
            SnapshotMode::Resume,
            &(),
        )
        .await
        .unwrap();
        assert!(matches!(
            result,
            SnapshotOutcome::Installed {
                cleanup_pending: false
            }
        ));
        assert_eq!(db.cursor().cursor, "snapshot-base");
        {
            let c = db.data.lock().unwrap();
            assert_eq!(count(&c, "sync_remote_items"), 101);
            assert_eq!(count(&c, "sync_snapshot_items"), 0);
            assert_eq!(count(&c, "sync_snapshot_recovery"), 0);
        }
        assert!(matches!(
            ChangeDownloader {
                http: &http,
                database: &db.data
            }
            .download(&credential(&config), &scope(), &SystemClock, &())
            .await
            .unwrap(),
            DownloadOutcome::Complete { pages: 1 }
        ));
        assert_eq!(db.cursor().cursor, "after-snapshot");
        api.finish();
    });
}

#[test]
fn restart_before_install_resumes_exact_next_page_without_mixing_snapshots() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        let http = APITransport::new(APIConfig::parse("http://127.0.0.1:1/").unwrap()).unwrap();
        save(&mut db.data.lock().unwrap(), &snapshot(101));
        let recovery = SnapshotRecovery {
            http: &http,
            database: &db.data,
        };
        let record = load(&db.data.lock().unwrap(), &scope()).unwrap().unwrap();
        recovery
            .stage(
                &scope(),
                &record,
                &SnapshotPage {
                    items: items(101)[..100].to_vec(),
                    total: 101,
                },
            )
            .unwrap();
        assert_eq!(db.cursor(), cursor("c+/?&= x"));
        *db.data.lock().unwrap() = open(&db.path).unwrap();
        let mut stale = record;
        stale.snapshot.snapshot_id =
            serde_json::from_value(json!("019a1234-0000-7000-8000-000000000301")).unwrap();
        stale.next_page = 2;
        assert!(
            recovery
                .stage(
                    &scope(),
                    &stale,
                    &SnapshotPage {
                        items: items(101)[100..].to_vec(),
                        total: 101
                    }
                )
                .is_err()
        );
        let api = TestApi::new(4, |request| {
            if request.contains("/access ") {
                (200, access(1, "active"))
            } else if request.contains("/items?") {
                assert!(request.contains("page=2&shows=100"));
                (200, json!({"items":&items(101)[100..],"total":101}))
            } else if request.starts_with("DELETE ") {
                (204, Value::Null)
            } else {
                (200, json!(snapshot(101)))
            }
        });
        let config = APIConfig::parse(&api.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
        assert!(matches!(
            SnapshotRecovery {
                http: &http,
                database: &db.data
            }
            .rebuild(
                &credential(&config),
                &scope(),
                &SystemClock,
                SnapshotMode::Resume,
                &()
            )
            .await
            .unwrap(),
            SnapshotOutcome::Installed {
                cleanup_pending: false
            }
        ));
        assert_eq!(count(&db.data.lock().unwrap(), "sync_remote_items"), 101);
        api.finish();
    });
}

#[test]
fn install_failure_rolls_back_view_projector_inbox_cursor_and_preserves_intentions_and_conflicts() {
    let db = Database::new();
    let http = APITransport::new(APIConfig::parse("http://127.0.0.1:1/").unwrap()).unwrap();
    {
        let mut c = db.data.lock().unwrap();
        apply_groups(&mut c, &scope(), &[group()], &()).unwrap();
        c.execute_batch(
            "CREATE TABLE test_projection(n INTEGER); INSERT INTO test_projection VALUES(7);",
        )
        .unwrap();
        let tx = c.transaction().unwrap();
        let id = record_operation(
            &tx,
            &SqliteOperationStore,
            &UuidV7Generator,
            &SystemClock,
            NewOperation {
                context: scope(),
                depends_on: vec![],
                action: "UpdatePatientProfile".into(),
                entity_type: "patient_profile".into(),
                entity_key: json!("019a1234-0000-7000-8000-000000000010"),
                expected_version: 1,
                payload: json!({"fullName":"Offline"}),
                occurred_at_local: None,
            },
        )
        .unwrap();
        record_operation(
            &tx,
            &SqliteOperationStore,
            &UuidV7Generator,
            &SystemClock,
            NewOperation {
                context: scope(),
                depends_on: vec![],
                action: "RegisterPatient".into(),
                entity_type: "patient_profile".into(),
                entity_key: json!("019a1234-0000-7000-8000-000000000011"),
                expected_version: 0,
                payload: json!({"fullName":"Another offline intention"}),
                occurred_at_local: None,
            },
        )
        .unwrap();
        tx.commit().unwrap();
        let pending = SqliteOperationStore.list_pending(&c, 10).unwrap()[0]
            .envelope()
            .unwrap()
            .canonical_json()
            .unwrap();
        c.execute(
            "UPDATE sync_outbox SET status='conflict',result=?1 WHERE operation_id=?2",
            params![
                "{\"status\":\"conflict\",\"state\":{\"version\":2}}",
                id.to_string()
            ],
        )
        .unwrap();
        assert!(!pending.is_empty());
        save(&mut c, &snapshot(1));
    }
    let recovery = SnapshotRecovery {
        http: &http,
        database: &db.data,
    };
    let record = load(&db.data.lock().unwrap(), &scope()).unwrap().unwrap();
    recovery
        .stage(
            &scope(),
            &record,
            &SnapshotPage {
                items: items(1),
                total: 1,
            },
        )
        .unwrap();
    let record = load(&db.data.lock().unwrap(), &scope()).unwrap().unwrap();
    let before = {
        let c = db.data.lock().unwrap();
        c.query_row(
            "SELECT payload,result,status FROM sync_outbox WHERE status='conflict'",
            [],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            },
        )
        .unwrap()
    };
    let failing = Projector {
        calls: Cell::new(0),
        fail: true,
    };
    assert!(recovery.install(&scope(), &record, &failing).is_err());
    assert_eq!(db.cursor(), cursor("c+/?&= x"));
    {
        let c = db.data.lock().unwrap();
        assert_eq!(count(&c, "sync_remote_items"), 16);
        assert_eq!(count(&c, "sync_inbox"), 1);
        assert_eq!(count(&c, "test_projection"), 1);
    }
    db.data.lock().unwrap().execute_batch("CREATE TRIGGER fail_cursor BEFORE UPDATE ON sync_cursor BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
    assert!(recovery.install(&scope(), &record, &()).is_err());
    assert_eq!(db.cursor(), cursor("c+/?&= x"));
    db.data
        .lock()
        .unwrap()
        .execute("DROP TRIGGER fail_cursor", [])
        .unwrap();
    let projector = Projector {
        calls: Cell::new(0),
        fail: false,
    };
    recovery.install(&scope(), &record, &projector).unwrap();
    let c = db.data.lock().unwrap();
    assert_eq!(count(&c, "sync_remote_items"), 1);
    assert_eq!(count(&c, "sync_inbox"), 0);
    assert_eq!(load(&c, &scope()).unwrap().unwrap().next_page, INSTALLED);
    assert_eq!(
        c.query_row(
            "SELECT payload,result,status FROM sync_outbox WHERE status='conflict'",
            [],
            |r| Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?
            ))
        )
        .unwrap(),
        before
    );
    drop(c);
    assert_eq!(db.cursor().cursor, "snapshot-base");
    let c = db.data.lock().unwrap();
    let pending = SqliteOperationStore.list_pending(&c, 10).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(
        pending[0].payload,
        json!({"fullName":"Another offline intention"})
    );
}

#[test]
fn final_metadata_expiry_or_drift_prevents_installation() {
    tauri::async_runtime::block_on(async {
        for expired in [true, false] {
            let db = Database::new();
            save(&mut db.data.lock().unwrap(), &snapshot(0));
            let api = TestApi::new(2, move |request| {
                if request.contains("/access ") {
                    (200, access(1, "active"))
                } else if expired {
                    (
                        410,
                        json!({"code":"SNAPSHOT_EXPIRED","message":"fixture","details":[],"correlationId":"019a1234-0000-7000-8000-000000000099"}),
                    )
                } else {
                    let mut changed = snapshot(0);
                    changed.base_cursor = "other-generation".into();
                    (200, json!(changed))
                }
            });
            let config = APIConfig::parse(&api.url).unwrap();
            let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
            let result = SnapshotRecovery {
                http: &http,
                database: &db.data,
            }
            .rebuild(
                &credential(&config),
                &scope(),
                &SystemClock,
                SnapshotMode::Resume,
                &(),
            )
            .await;
            if expired {
                assert!(matches!(result.unwrap(), SnapshotOutcome::RecoveryRequired));
            } else {
                assert!(result.is_err());
            }
            assert_eq!(db.cursor(), cursor("c+/?&= x"));
            api.finish();
        }
    });
}

#[test]
fn explicit_restart_replaces_only_staging_and_current_epoch_is_required() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        save(&mut db.data.lock().unwrap(), &snapshot(101));
        db.data
            .lock()
            .unwrap()
            .execute("UPDATE sync_snapshot_recovery SET permission_epoch=0", [])
            .unwrap();
        let api = TestApi::new(1, |_| (200, access(1, "active")));
        let config = APIConfig::parse(&api.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
        assert!(matches!(
            SnapshotRecovery {
                http: &http,
                database: &db.data
            }
            .rebuild(
                &credential(&config),
                &scope(),
                &SystemClock,
                SnapshotMode::Resume,
                &()
            )
            .await
            .unwrap(),
            SnapshotOutcome::RecoveryRequired
        ));
        assert_eq!(db.cursor(), cursor("c+/?&= x"));
        api.finish();
        let api = TestApi::new(4, |request| {
            if request.contains("/access ") {
                (200, access(1, "active"))
            } else if request.starts_with("DELETE ") {
                (204, Value::Null)
            } else {
                let mut fresh = snapshot(0);
                fresh.snapshot_id =
                    serde_json::from_value(json!("019a1234-0000-7000-8000-000000000301")).unwrap();
                (200, json!(fresh))
            }
        });
        let config = APIConfig::parse(&api.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
        assert!(matches!(
            SnapshotRecovery {
                http: &http,
                database: &db.data
            }
            .rebuild(
                &credential(&config),
                &scope(),
                &SystemClock,
                SnapshotMode::Restart,
                &()
            )
            .await
            .unwrap(),
            SnapshotOutcome::Installed {
                cleanup_pending: false
            }
        ));
        assert_eq!(count(&db.data.lock().unwrap(), "sync_snapshot_recovery"), 0);
        assert_eq!(db.cursor().cursor, "snapshot-base");
        api.finish();
    });
}

#[test]
fn after_install_restart_only_retries_release_and_keeps_newer_incremental_cursor() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        save(&mut db.data.lock().unwrap(), &snapshot(0));
        let http = APITransport::new(APIConfig::parse("http://127.0.0.1:1/").unwrap()).unwrap();
        let record = load(&db.data.lock().unwrap(), &scope()).unwrap().unwrap();
        SnapshotRecovery {
            http: &http,
            database: &db.data,
        }
        .install(&scope(), &record, &())
        .unwrap();
        {
            let mut c = db.data.lock().unwrap();
            let tx = c.transaction().unwrap();
            SqliteSyncControlStore
                .save_cursor(&tx, &scope(), &cursor("newer-incremental"))
                .unwrap();
            tx.commit().unwrap();
        }
        *db.data.lock().unwrap() = open(&db.path).unwrap();
        for status in [503, 204] {
            let api = TestApi::new(2, move |request| {
                if request.contains("/access ") {
                    (200, access(1, "active"))
                } else {
                    assert!(request.starts_with("DELETE "));
                    (
                        status,
                        if status == 204 {
                            Value::Null
                        } else {
                            json!({"code":"SERVICE_UNAVAILABLE","message":"fixture","details":[],"correlationId":"019a1234-0000-7000-8000-000000000099"})
                        },
                    )
                }
            });
            let config = APIConfig::parse(&api.url).unwrap();
            let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
            let result = SnapshotRecovery {
                http: &http,
                database: &db.data,
            }
            .rebuild(
                &credential(&config),
                &scope(),
                &SystemClock,
                SnapshotMode::Resume,
                &(),
            )
            .await
            .unwrap();
            assert!(
                matches!(result,SnapshotOutcome::Installed{cleanup_pending} if cleanup_pending==(status==503))
            );
            assert_eq!(db.cursor(), cursor("newer-incremental"));
            api.finish();
        }
        assert_eq!(count(&db.data.lock().unwrap(), "sync_snapshot_recovery"), 0);
    });
}

#[test]
fn expired_scope_changed_and_missing_snapshot_discard_only_staging_for_explicit_retry() {
    tauri::async_runtime::block_on(async {
        for code in ["SNAPSHOT_EXPIRED", "SNAPSHOT_SCOPE_CHANGED", "NOT_FOUND"] {
            let db = Database::new();
            save(&mut db.data.lock().unwrap(), &snapshot(1));
            let api = TestApi::new(2, move |request| {
                if request.contains("/access ") {
                    (200, access(1, "active"))
                } else {
                    (
                        if code == "NOT_FOUND" { 404 } else { 410 },
                        json!({"code":code,"message":"fixture","details":[],"correlationId":"019a1234-0000-7000-8000-000000000099"}),
                    )
                }
            });
            let config = APIConfig::parse(&api.url).unwrap();
            let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
            assert!(matches!(
                SnapshotRecovery {
                    http: &http,
                    database: &db.data
                }
                .rebuild(
                    &credential(&config),
                    &scope(),
                    &SystemClock,
                    SnapshotMode::Resume,
                    &()
                )
                .await
                .unwrap(),
                SnapshotOutcome::RecoveryRequired
            ));
            assert_eq!(db.cursor(), cursor("c+/?&= x"));
            assert_eq!(count(&db.data.lock().unwrap(), "sync_snapshot_recovery"), 0);
            api.finish();
        }
    });
}

#[test]
fn invalid_metadata_page_total_duplicates_and_epoch_never_replace_current_view() {
    tauri::async_runtime::block_on(async {
        for field in ["accountId", "modelRevision", "contractVersion"] {
            let db = Database::new();
            let api = TestApi::new(2, move |request| {
                if request.contains("/access ") {
                    (200, access(1, "active"))
                } else {
                    let mut v = json!(snapshot(1));
                    v[field] = if field == "contractVersion" {
                        json!(2)
                    } else if field == "accountId" {
                        json!("019a1234-0000-7000-8000-000000000099")
                    } else {
                        json!("other")
                    };
                    (200, v)
                }
            });
            let config = APIConfig::parse(&api.url).unwrap();
            let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
            assert!(
                SnapshotRecovery {
                    http: &http,
                    database: &db.data
                }
                .rebuild(
                    &credential(&config),
                    &scope(),
                    &SystemClock,
                    SnapshotMode::Resume,
                    &()
                )
                .await
                .is_err()
            );
            assert_eq!(db.cursor(), cursor("c+/?&= x"));
            api.finish();
        }
        let db = Database::new();
        save(&mut db.data.lock().unwrap(), &snapshot(2));
        let http = APITransport::new(APIConfig::parse("http://127.0.0.1:1/").unwrap()).unwrap();
        let recovery = SnapshotRecovery {
            http: &http,
            database: &db.data,
        };
        let record = load(&db.data.lock().unwrap(), &scope()).unwrap().unwrap();
        assert!(
            recovery
                .stage(
                    &scope(),
                    &record,
                    &SnapshotPage {
                        items: items(2),
                        total: 3
                    }
                )
                .is_err()
        );
        let duplicate = items(1).remove(0);
        assert!(
            recovery
                .stage(
                    &scope(),
                    &record,
                    &SnapshotPage {
                        items: vec![duplicate.clone(), duplicate],
                        total: 2
                    }
                )
                .is_err()
        );
        assert_eq!(count(&db.data.lock().unwrap(), "sync_snapshot_items"), 0);
        assert_eq!(
            load(&db.data.lock().unwrap(), &scope())
                .unwrap()
                .unwrap()
                .next_page,
            1
        );
        {
            let c = db.data.lock().unwrap();
            c.execute("UPDATE identity_account_access SET permission_epoch=2", [])
                .unwrap();
        }
        assert!(
            recovery
                .stage(
                    &scope(),
                    &record,
                    &SnapshotPage {
                        items: items(2),
                        total: 2
                    }
                )
                .is_err()
        );
        assert_eq!(db.cursor(), cursor("c+/?&= x"));
    });
}
