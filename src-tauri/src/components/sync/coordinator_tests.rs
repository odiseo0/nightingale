use super::*;
use crate::{
    components::sync::{
        application::{OperationStore, record_operation},
        domain::NewOperation,
        download::tests::{Database, access, credential, cursor, group, scope},
    },
    platform::{config::APIConfig, sqlite::open, test_api::TestApi},
    shared_kernel::{SystemClock, UuidV7Generator},
};
use serde_json::{Value, json};
use std::{
    pin::Pin,
    sync::atomic::{AtomicUsize, Ordering},
    task::Poll,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

struct FixedClock(i64);

impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_millis(self.0 as u64)
    }
}

fn seed(c: &mut Connection, input: OperationContext, status: &str) -> String {
    let tx = c.transaction().unwrap();
    let id = record_operation(
        &tx,
        &SqliteOperationStore,
        &UuidV7Generator,
        &FixedClock(1_000),
        NewOperation {
            context: input,
            depends_on: vec![],
            action: "RegisterPatient".into(),
            entity_type: "patient_profile".into(),
            entity_key: json!("019a1234-0000-7000-8000-000000000010"),
            expected_version: 0,
            payload: json!({"fullName":"Synthetic"}),
            occurred_at_local: None,
        },
    )
    .unwrap();
    tx.commit().unwrap();
    c.execute(
        "UPDATE sync_outbox SET status=?1 WHERE operation_id=?2",
        rusqlite::params![status, id.to_string()],
    )
    .unwrap();
    id.to_string()
}

fn fake_report(coordinator: &SyncCoordinator, data: &Mutex<Connection>) -> CycleReport {
    CycleReport {
        outcome: CycleOutcome::Complete,
        status: coordinator.status(data, &scope()).unwrap(),
        remote_code: None,
        local_code: None,
    }
}

async fn poll_pending(future: Pin<&mut impl Future>) {
    let mut future = future;
    std::future::poll_fn(|cx| {
        assert!(future.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
}

#[test]
fn concurrent_triggers_share_one_flight_and_publish_phase_without_sqlite_lock() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        let coordinator = Arc::new(SyncCoordinator::default());
        let calls = Arc::new(AtomicUsize::new(0));
        let (ready_tx, mut ready) = tauri::async_runtime::channel(1);
        let (release_tx, release) = tauri::async_runtime::channel(1);
        let release = Arc::new(AsyncMutex::new(release));
        let (first_coordinator, first_data, first_calls) =
            (coordinator.clone(), db.data.clone(), calls.clone());
        let first = tauri::async_runtime::spawn(async move {
            first_coordinator
                .run_with(&scope(), CycleTrigger::Startup, |manual| {
                    assert!(!manual);
                    first_calls.fetch_add(1, Ordering::SeqCst);
                    let release = release.clone();
                    let ready = ready_tx.clone();
                    let coordinator = first_coordinator.clone();
                    let data = first_data.clone();
                    async move {
                        coordinator
                            .flight(&scope())
                            .unwrap()
                            .phase(SyncPhase::Recovering)
                            .unwrap();
                        ready.send(()).await.unwrap();
                        release.lock().await.recv().await.unwrap();
                        Ok(fake_report(&coordinator, &data))
                    }
                })
                .await
                .unwrap()
        });
        ready.recv().await.unwrap();
        assert_eq!(
            coordinator.status(&db.data, &scope()).unwrap().phase,
            SyncPhase::Recovering
        );
        assert!(db.data.try_lock().is_ok());
        let second_scope = scope();
        let mut second = Box::pin(coordinator.run_with(
            &second_scope,
            CycleTrigger::Reconnect,
            |_| async { panic!("second cycle must join") },
        ));
        poll_pending(second.as_mut()).await;
        release_tx.send(()).await.unwrap();
        let first = first.await.unwrap();
        assert_eq!(second.await.unwrap(), first);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            coordinator.status(&db.data, &scope()).unwrap().phase,
            SyncPhase::Idle
        );
    });
}

#[test]
fn manual_join_requests_one_serial_pass_and_cancellation_allows_recovery() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        let coordinator = SyncCoordinator::default();
        let s = scope();
        let calls = AtomicUsize::new(0);
        let (release_tx, release) = tauri::async_runtime::channel(1);
        let release = Arc::new(AsyncMutex::new(release));
        let mut first = Box::pin(coordinator.run_with(&s, CycleTrigger::Startup, |manual| {
            let n = calls.fetch_add(1, Ordering::SeqCst);
            assert_eq!(manual, n == 1);
            let release = release.clone();
            let coordinator = &coordinator;
            let data = &db.data;
            async move {
                if n == 0 {
                    release.lock().await.recv().await.unwrap();
                }
                Ok(fake_report(coordinator, data))
            }
        }));
        poll_pending(first.as_mut()).await;
        let mut second = Box::pin(coordinator.run_with(&s, CycleTrigger::Manual, |_| async {
            panic!("must join manual request")
        }));
        poll_pending(second.as_mut()).await;
        release_tx.send(()).await.unwrap();
        let result = first.await.unwrap();
        assert_eq!(second.await.unwrap(), result);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        let mut cancelled = Box::pin(coordinator.run_with(&s, CycleTrigger::Reconnect, |_| {
            std::future::pending::<Result<CycleReport>>()
        }));
        poll_pending(cancelled.as_mut()).await;
        drop(cancelled);
        coordinator
            .run_with(&s, CycleTrigger::Manual, |manual| {
                assert!(manual);
                async { Ok(fake_report(&coordinator, &db.data)) }
            })
            .await
            .unwrap();
        assert_eq!(
            coordinator.status(&db.data, &s).unwrap().phase,
            SyncPhase::Idle
        );
    });
}

#[test]
fn native_cycle_recovers_unknown_then_sends_and_downloads_without_changing_ids() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        let unknown = seed(&mut db.data.lock().unwrap(), scope(), "unknown");
        let pending = seed(&mut db.data.lock().unwrap(), scope(), "pending");
        let data = db.data.clone();
        let expected_pending = pending.clone();
        let expected_unknown = unknown.clone();
        let api = TestApi::new(8, move |request| {
            assert!(data.try_lock().is_ok());
            if request.contains("/access ") {
                (200, access(1, "active"))
            } else if request.starts_with("GET ") && request.contains("/sync/operations/") {
                assert!(request.contains(&expected_unknown));
                (
                    200,
                    json!({"operationId":expected_unknown,"status":"accepted","entityVersion":1,"state":{"version":1},"error":null}),
                )
            } else if request.starts_with("POST ") {
                let body: Value =
                    serde_json::from_str(request.split_once("\r\n\r\n").unwrap().1).unwrap();
                assert_eq!(body["operations"].as_array().unwrap().len(), 1);
                assert_eq!(body["operations"][0]["operationId"], expected_pending);
                (
                    200,
                    json!({"results":[{"operationId":expected_pending,"status":"accepted","entityVersion":1,"state":{"version":1},"error":null}],"correlationId":"019a1234-0000-7000-8000-000000000099"}),
                )
            } else {
                (
                    200,
                    json!({"groups":[group()],"nextCursor":"after-cycle","hasMore":false}),
                )
            }
        });
        let config = APIConfig::parse(&api.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
        let coordinator = SyncCoordinator::default();
        let result = SyncEngine {
            coordinator: &coordinator,
            http: &http,
            database: &db.data,
        }
        .run(
            &credential(&config),
            &scope(),
            CycleTrigger::Startup,
            &FixedClock(2_000),
            &UuidV7Generator,
            &(),
        )
        .await
        .unwrap();
        assert_eq!(result.outcome, CycleOutcome::Complete);
        assert_eq!(result.status.outbox.pending, 0);
        assert_eq!(result.status.outbox.unknown, 0);
        assert_eq!(db.cursor(), cursor("after-cycle"));
        let c = db.data.lock().unwrap();
        assert_eq!(
            c.query_row(
                "SELECT COUNT(*) FROM sync_outbox WHERE status='accepted'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            2
        );
        assert_eq!(
            SqliteOperationStore
                .operation_result(
                    &c,
                    &scope(),
                    &serde_json::from_value(json!(unknown)).unwrap()
                )
                .unwrap()
                .unwrap()
                .status,
            super::super::domain::OperationStatus::Accepted
        );
        drop(c);
        api.finish();
    });
}

#[test]
fn durable_wait_manual_retry_reviews_foreign_scope_and_live_leases_survive_reopen() {
    let db = Database::new();
    let (unknown, retryable, review, foreign, leased) = {
        let mut c = db.data.lock().unwrap();
        let unknown = seed(&mut c, scope(), "unknown");
        let retryable = seed(&mut c, scope(), "retryable");
        let review = seed(&mut c, scope(), "unknown");
        let mut other = scope();
        other.permission_epoch = 0;
        let foreign = seed(&mut c, other, "unknown");
        let leased = seed(&mut c, scope(), "in_flight");
        c.execute("UPDATE sync_outbox SET next_attempt_at=900000", [])
            .unwrap();
        c.execute(
            "UPDATE sync_outbox SET last_error='\"RESULT_EXPIRED\"' WHERE operation_id=?1",
            [&review],
        )
        .unwrap();
        c.execute(
            "UPDATE sync_outbox SET lease_until=900000 WHERE operation_id=?1",
            [&leased],
        )
        .unwrap();
        (unknown, retryable, review, foreign, leased)
    };
    let coordinator = SyncCoordinator::default();
    let before = coordinator.status(&db.data, &scope()).unwrap();
    let access = {
        let c = db.data.lock().unwrap();
        SqliteAccessStore
            .access(
                &c,
                &scope().account_id,
                &scope().user_id,
                &scope().device_id,
            )
            .unwrap()
            .unwrap()
    };
    assert!(
        SqliteOperationStore
            .prepare_cycle(&mut db.data.lock().unwrap(), &access, 2_000, true)
            .unwrap()
            .is_empty()
    );
    assert_eq!(coordinator.status(&db.data, &scope()).unwrap(), before);
    *db.data.lock().unwrap() = open(&db.path).unwrap();
    let restored = SyncCoordinator::default()
        .status(&db.data, &scope())
        .unwrap();
    assert_eq!(restored.outbox.unknown, 4);
    assert_eq!(restored.outbox.in_flight, 0);
    assert_eq!(restored.outbox.review_required, 1);
    assert_eq!(restored.outbox.other_epoch, 1);
    let mut c = db.data.lock().unwrap();
    assert!(
        SqliteOperationStore
            .prepare_cycle(&mut c, &access, 2_000, false)
            .unwrap()
            .is_empty()
    );
    let eligible = SqliteOperationStore
        .prepare_cycle(&mut c, &access, 2_000, true)
        .unwrap();
    assert!(eligible.iter().any(|id| id.as_str() == unknown));
    assert!(eligible.iter().any(|id| id.as_str() == leased));
    assert!(
        !eligible
            .iter()
            .any(|id| id.as_str() == review || id.as_str() == foreign || id.as_str() == retryable)
    );
    assert_eq!(
        c.query_row(
            "SELECT next_attempt_at FROM sync_outbox WHERE operation_id=?1",
            [&retryable],
            |r| r.get::<_, Option<i64>>(0)
        )
        .unwrap(),
        None
    );
    for id in [review, foreign] {
        assert_eq!(
            c.query_row(
                "SELECT next_attempt_at FROM sync_outbox WHERE operation_id=?1",
                [id],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
            900000
        );
    }
}

#[test]
fn api_failure_publishes_safe_status_and_local_work_remains_available_during_http() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        let coordinator = Arc::new(SyncCoordinator::default());
        let data = db.data.clone();
        let state = coordinator.clone();
        let api = TestApi::new(1, move |_| {
            assert_eq!(
                state.status(&data, &scope()).unwrap().phase,
                SyncPhase::Access
            );
            seed(&mut data.lock().unwrap(), scope(), "pending");
            (
                503,
                json!({"code":"SERVICE_UNAVAILABLE","message":"secret server text","details":[],"correlationId":"019a1234-0000-7000-8000-000000000099"}),
            )
        });
        let config = APIConfig::parse(&api.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
        let result = SyncEngine {
            coordinator: &coordinator,
            http: &http,
            database: &db.data,
        }
        .run(
            &credential(&config),
            &scope(),
            CycleTrigger::Reconnect,
            &SystemClock,
            &UuidV7Generator,
            &(),
        )
        .await
        .unwrap();
        assert_eq!(result.outcome, CycleOutcome::RetryWaiting);
        assert_eq!(result.status.outbox.pending, 1);
        assert_eq!(result.remote_code, Some(APICode::ServiceUnavailable));
        let text = serde_json::to_string(&result).unwrap();
        assert!(!text.contains("secret server text"));
        assert!(!text.contains("synthetic-device-token"));
        assert!(!text.contains("fullName"));
        let c = db.data.lock().unwrap();
        assert_eq!(SqliteOperationStore.list_pending(&c, 10).unwrap().len(), 1);
        drop(c);
        api.finish();
    });
}

#[test]
fn manual_native_cycle_bypasses_wait_and_posts_known_retryable_but_queries_unknown() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        let unknown = seed(&mut db.data.lock().unwrap(), scope(), "unknown");
        let retryable = seed(&mut db.data.lock().unwrap(), scope(), "retryable");
        db.data
            .lock()
            .unwrap()
            .execute("UPDATE sync_outbox SET next_attempt_at=900000", [])
            .unwrap();
        let expected_unknown = unknown.clone();
        let expected_retryable = retryable.clone();
        let api = TestApi::new(12, move |request| {
            if request.contains("/access ") {
                (200, access(1, "active"))
            } else if request.contains("/sync/operations/") {
                assert!(request.starts_with("GET "));
                assert!(request.contains(&expected_unknown));
                (
                    200,
                    json!({"operationId":expected_unknown,"status":"accepted","entityVersion":1,"state":{"version":1},"error":null}),
                )
            } else if request.starts_with("POST ") {
                let body: Value =
                    serde_json::from_str(request.split_once("\r\n\r\n").unwrap().1).unwrap();
                assert_eq!(body["operations"][0]["operationId"], expected_retryable);
                assert_eq!(body["operations"].as_array().unwrap().len(), 1);
                (
                    200,
                    json!({"results":[{"operationId":expected_retryable,"status":"accepted","entityVersion":1,"state":{"version":1},"error":null}],"correlationId":"019a1234-0000-7000-8000-000000000099"}),
                )
            } else {
                (
                    200,
                    json!({"groups":[],"nextCursor":"after-wait","hasMore":false}),
                )
            }
        });
        let config = APIConfig::parse(&api.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
        let coordinator = SyncCoordinator::default();
        let credential = credential(&config);
        let engine = SyncEngine {
            coordinator: &coordinator,
            http: &http,
            database: &db.data,
        };
        let automatic = engine
            .run(
                &credential,
                &scope(),
                CycleTrigger::Startup,
                &FixedClock(2_000),
                &UuidV7Generator,
                &(),
            )
            .await
            .unwrap();
        assert_eq!(automatic.status.outbox.unknown, 1);
        assert_eq!(automatic.status.outbox.retryable, 1);
        assert_eq!(automatic.status.outbox.next_attempt_at, Some(900000));
        let result = engine
            .run(
                &credential,
                &scope(),
                CycleTrigger::Manual,
                &FixedClock(2_000),
                &UuidV7Generator,
                &(),
            )
            .await
            .unwrap();
        assert_eq!(result.outcome, CycleOutcome::Complete);
        assert_eq!(result.status.outbox.unknown, 0);
        assert_eq!(result.status.outbox.retryable, 0);
        let requests = api.finish();
        assert_eq!(
            requests.iter().filter(|r| r.starts_with("POST ")).count(),
            1
        );
    });
}

#[test]
fn new_epoch_rebuilds_before_incremental_and_never_relabels_old_intentions() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        let pending = seed(&mut db.data.lock().unwrap(), scope(), "pending");
        let api = TestApi::new(9, |request| {
            if request.contains("/access ") {
                (200, access(2, "active"))
            } else if request.starts_with("DELETE ") {
                (204, Value::Null)
            } else if request.contains("/sync/changes?") {
                (
                    200,
                    json!({"groups":[],"nextCursor":"new-epoch-next","hasMore":false}),
                )
            } else {
                assert!(!request.contains("/sync/operations"));
                (
                    200,
                    json!({"snapshotId":"019a1234-0000-7000-8000-000000000500","accountId":scope().account_id,"contractVersion":1,"modelRevision":"api-v1.1","window":cursor("base").window,"baseCursor":"new-epoch-base","expiresAt":"2026-10-30T23:00:00Z","itemCount":0}),
                )
            }
        });
        let config = APIConfig::parse(&api.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
        let coordinator = SyncCoordinator::default();
        let result = SyncEngine {
            coordinator: &coordinator,
            http: &http,
            database: &db.data,
        }
        .run(
            &credential(&config),
            &scope(),
            CycleTrigger::Reconnect,
            &FixedClock(2_000),
            &UuidV7Generator,
            &(),
        )
        .await
        .unwrap();
        assert_eq!(result.outcome, CycleOutcome::Complete);
        assert_eq!(result.status.context.permission_epoch, 2);
        assert!(result.status.cursor_ready);
        assert_eq!(result.status.outbox.other_epoch, 1);
        let c = db.data.lock().unwrap();
        assert_eq!(
            c.query_row(
                "SELECT permission_epoch,status FROM sync_outbox WHERE operation_id=?1",
                [pending],
                |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
            )
            .unwrap(),
            (1, "pending".into())
        );
        drop(c);
        api.finish();
    });
}

#[test]
fn denial_stops_cycle_and_separate_accounts_do_not_share_results() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        seed(&mut db.data.lock().unwrap(), scope(), "pending");
        let api = TestApi::new(1, |_| {
            (
                403,
                json!({"code":"PERMISSION_DENIED","message":"fixture","details":[],"correlationId":"019a1234-0000-7000-8000-000000000099"}),
            )
        });
        let config = APIConfig::parse(&api.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
        let coordinator = SyncCoordinator::default();
        let result = SyncEngine {
            coordinator: &coordinator,
            http: &http,
            database: &db.data,
        }
        .run(
            &credential(&config),
            &scope(),
            CycleTrigger::Manual,
            &FixedClock(2_000),
            &UuidV7Generator,
            &(),
        )
        .await
        .unwrap();
        assert_eq!(result.outcome, CycleOutcome::AccessDenied);
        assert_eq!(result.status.outbox.pending, 1);
        api.finish();
        let mut other = scope();
        other.account_id =
            serde_json::from_value(json!("019a1234-0000-7000-8000-000000000099")).unwrap();
        let s = scope();
        let mut first = Box::pin(coordinator.run_with(&s, CycleTrigger::Startup, |_| {
            std::future::pending::<Result<CycleReport>>()
        }));
        poll_pending(first.as_mut()).await;
        let next = coordinator
            .run_with(&other, CycleTrigger::Startup, |_| async {
                let mut report = fake_report(&coordinator, &db.data);
                report.status.context = other.clone();
                Ok(report)
            })
            .await
            .unwrap();
        assert_eq!(next.status.context, other);
        drop(first);
    });
}

#[test]
#[ignore = "requires development API/OIDC, native secret store, browser and a synthetic account"]
fn manual_t13_cycle_with_development_api() {
    tauri::async_runtime::block_on(async {
        use crate::{
            components::{
                devices::activation::DeviceCredentials,
                identity::{
                    api::IdentityAPI,
                    application::AccountActivation,
                    oidc::{LoginCancellation, OidcProvider, SystemBrowser},
                },
            },
            platform::secrets::NativeSecretStore,
            shared_kernel::id::AccountId,
        };
        let config = APIConfig::from_environment().unwrap();
        let account = AccountId::try_from(
            std::env::var("NIGHTINGALE_TEST_ACCOUNT_ID")
                .expect("falta ID no secreto de la cuenta sintética"),
        )
        .unwrap();
        let http =
            APITransport::new(APIConfig::parse(config.base_url().as_str()).unwrap()).unwrap();
        let auth = IdentityAPI(&http).auth_config().await.unwrap().value;
        let provider = OidcProvider::discover(&auth, &http).await.unwrap();
        let attempt = provider.begin(&SystemBrowser).unwrap();
        let session = provider
            .complete(attempt, LoginCancellation::default(), &http)
            .await
            .unwrap();
        let db = Database::new();
        db.data
            .lock()
            .unwrap()
            .execute_batch("DELETE FROM identity_account_access; DELETE FROM sync_cursor;")
            .unwrap();
        let store = NativeSecretStore;
        let profile = format!("manual-t13-{}", uuid::Uuid::now_v7());
        let activation = AccountActivation {
            config: &config,
            profile: &profile,
            http: &http,
            secrets: &store,
            database: &db.data,
        };
        let activated = activation
            .activate(&session, &account, &UuidV7Generator, &SystemClock)
            .await;
        let result = match activated {
            Ok(access) => {
                let credential = DeviceCredentials {
                    config: &config,
                    profile: &profile,
                    identity: (session.issuer(), session.subject()),
                    secrets: &store,
                }
                .load()
                .unwrap()
                .unwrap();
                let coordinator = SyncCoordinator::default();
                let result = SyncEngine {
                    coordinator: &coordinator,
                    http: &http,
                    database: &db.data,
                }
                .run(
                    &credential,
                    &access.context,
                    CycleTrigger::Startup,
                    &SystemClock,
                    &UuidV7Generator,
                    &(),
                )
                .await;
                drop(credential);
                result
            }
            Err(_) => Err(NightingaleError::temporary(
                "alta de desarrollo no disponible",
            )),
        };
        activation
            .logout((session.issuer(), session.subject()))
            .unwrap();
        let report = result.unwrap();
        assert_eq!(report.outcome, CycleOutcome::Complete);
        assert!(report.status.cursor_ready);
    });
}

#[test]
fn expired_cursor_after_pending_release_starts_one_fresh_capture_in_same_cycle() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        db.data.lock().unwrap().execute("INSERT INTO sync_snapshot_recovery (account_id,user_id,device_id,permission_epoch,snapshot_id,base_cursor,window,expires_at,item_count,next_page) VALUES (?1,?2,?3,1,?4,?5,?6,?7,0,?8)",rusqlite::params![scope().account_id.as_str(),scope().user_id.as_str(),scope().device_id.as_str(),"019a1234-0000-7000-8000-000000000500","old-base",serde_json::to_string(&cursor("base").window).unwrap(),"2026-10-30T23:00:00Z",i64::MAX]).unwrap();
        let mut created = false;
        let api = TestApi::new(14, move |request| {
            if request.contains("/access ") {
                (200, access(1, "active"))
            } else if request.contains("/sync/changes?") {
                if created {
                    (
                        200,
                        json!({"groups":[],"nextCursor":"fresh-next","hasMore":false}),
                    )
                } else {
                    (
                        410,
                        json!({"code":"CURSOR_EXPIRED","message":"fixture","details":[],"correlationId":"019a1234-0000-7000-8000-000000000099"}),
                    )
                }
            } else if request.starts_with("DELETE ") {
                (204, Value::Null)
            } else {
                if request.starts_with("POST ") {
                    assert!(!created);
                    created = true;
                }
                (
                    200,
                    json!({"snapshotId":"019a1234-0000-7000-8000-000000000501","accountId":scope().account_id,"contractVersion":1,"modelRevision":"api-v1.1","window":cursor("base").window,"baseCursor":"fresh-base","expiresAt":"2026-10-30T23:00:00Z","itemCount":0}),
                )
            }
        });
        let config = APIConfig::parse(&api.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
        let coordinator = SyncCoordinator::default();
        let result = SyncEngine {
            coordinator: &coordinator,
            http: &http,
            database: &db.data,
        }
        .run(
            &credential(&config),
            &scope(),
            CycleTrigger::Startup,
            &FixedClock(2_000),
            &UuidV7Generator,
            &(),
        )
        .await
        .unwrap();
        assert_eq!(result.outcome, CycleOutcome::Complete);
        assert!(!result.status.snapshot_pending);
        assert_eq!(db.cursor().cursor, "fresh-next");
        let requests = api.finish();
        assert_eq!(
            requests.iter().filter(|r| r.starts_with("POST ")).count(),
            1
        );
        assert_eq!(
            requests.iter().filter(|r| r.starts_with("DELETE ")).count(),
            2
        );
    });
}
