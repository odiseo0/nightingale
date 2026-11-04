use super::{
    coordinator::{CycleOutcome, CycleTrigger, SyncCoordinator, SyncEngine},
    download::{
        ChangeDownloader,
        tests::{Database, credential, cursor, scope},
    },
    remote::{ChangeGroup, ChangeKind, EntityType, SyncChange},
    snapshot::{SnapshotMode, SnapshotOutcome, SnapshotRecovery},
};
use crate::{
    components::{
        projector::DesktopProjector,
        services::{
            adapters::outbound::SqliteCatalogRepository,
            application::{CatalogQuery, CatalogRepository, CatalogUseCases},
            domain::{DurationMinutes, ServicePatch},
            test_support::{PROFESSIONAL, access, seed},
        },
    },
    platform::{config::APIConfig, http::APITransport, sqlite::open, test_api::TestApi},
    shared_kernel::{SystemClock, UuidV7Generator},
};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

fn count(c: &Connection, table: &str) -> i64 {
    c.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

fn group(changes: Vec<SyncChange>) -> ChangeGroup {
    ChangeGroup {
        group_id: uuid::Uuid::now_v7().to_string().try_into().unwrap(),
        operation_id: uuid::Uuid::now_v7().to_string().try_into().unwrap(),
        accepted_at: "2026-11-04T12:00:00Z".into(),
        changes,
    }
}

fn service(id: &str, version: i64) -> SyncChange {
    SyncChange {
        entity_type: EntityType::Service,
        entity_key: json!(id),
        entity_version: version,
        change_kind: ChangeKind::Upsert,
        state: json!({"id":id,"professionalId":PROFESSIONAL,"name":"Accepted","defaultDurationMinutes":30,"status":"active","version":version,"createdAt":null,"updatedAt":null}),
    }
}

fn offer(id: &str, version: i64, service_version: i64) -> SyncChange {
    SyncChange {
        entity_type: EntityType::ServiceOffer,
        entity_key: json!({"professionalId":PROFESSIONAL,"serviceId":id}),
        entity_version: version,
        change_kind: ChangeKind::Upsert,
        state: json!({"accountId":scope().account_id,"professionalId":PROFESSIONAL,"serviceId":id,"name":"Accepted offer","defaultDurationMinutes":30,"durationMinutesOverride":null,"effectiveDurationMinutes":30,"status":"active","serviceStatus":"active","version":version,"serviceVersion":service_version,"createdAt":null,"updatedAt":null}),
    }
}

fn business_changes(id: &str) -> Vec<SyncChange> {
    let mut changes = super::download::tests::group().changes;
    changes.retain(|c| {
        matches!(
            c.entity_type,
            EntityType::Account | EntityType::Professional
        )
    });
    let professional = changes
        .iter_mut()
        .find(|c| c.entity_type == EntityType::Professional)
        .unwrap();
    professional.entity_key = json!(PROFESSIONAL);
    professional.state["id"] = json!(PROFESSIONAL);
    professional.state["userId"] = json!(scope().user_id);
    changes.extend([service(id, 1), offer(id, 2, 2)]);
    changes
}

#[test]
fn offline_dispatch_projection_conflict_remove_reopen_and_fresh_snapshot_over_http() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        let access = access();
        let created = {
            let mut c = db.data.lock().unwrap();
            seed(&mut c, &access);
            let mut other = access.clone();
            other.context.account_id = "019a1234-0000-7000-8000-000000000009"
                .to_owned()
                .try_into()
                .unwrap();
            seed(&mut c, &other);
            let mut repo = SqliteCatalogRepository { connection: &mut c };
            let result = CatalogUseCases {
                repository: &mut repo,
                access: &access,
                ids: &UuidV7Generator,
                clock: &SystemClock,
            }
            .create_service(
                PROFESSIONAL.to_owned().try_into().unwrap(),
                "Offline".into(),
                DurationMinutes::new(30).unwrap(),
            )
            .unwrap();
            assert!(repo.list_services(&scope()).unwrap().is_empty());
            assert_eq!(
                repo.catalog(&access).unwrap().services[0].local_changes[0].sync_status,
                "pending"
            );
            result
        };
        let id = created.entity_key.as_str().unwrap().to_owned();
        // Reopen before any HTTP: the operation and provisional data are durable.
        {
            let mut c = open(&db.path).unwrap();
            assert_eq!(count(&c, "sync_outbox"), 1);
            assert_eq!(
                SqliteCatalogRepository { connection: &mut c }
                    .catalog(&access)
                    .unwrap()
                    .services[0]
                    .provisional
                    .as_ref()
                    .unwrap()
                    .name,
                "Offline"
            );
        }
        let stage = Arc::new(AtomicUsize::new(0));
        let server_stage = stage.clone();
        let service_id = id.clone();
        let data = db.data.clone();
        let allowed = access.allowed_actions.clone();
        let expected_id = created.operation_id.clone();
        let api = TestApi::dynamic(move |request| {
            assert!(data.try_lock().is_ok(), "HTTP must not hold SQLite");
            let stage = server_stage.load(Ordering::SeqCst);
            if request.contains("/access ") {
                return (
                    200,
                    json!({"accountId":scope().account_id,"userId":scope().user_id,"deviceId":scope().device_id,"permissionEpoch":1,"role":"owner","status":"active","allowedActions":allowed}),
                );
            }
            if request.contains("/sync/operations ") {
                let body: Value =
                    serde_json::from_str(request.split_once("\r\n\r\n").unwrap().1).unwrap();
                let operation = &body["operations"][0];
                assert_eq!(body["operations"].as_array().unwrap().len(), 1);
                if stage == 0 {
                    assert_eq!(operation["operationId"], expected_id);
                    assert_eq!(operation["action"], "CreateService");
                    assert_eq!(operation["payload"]["name"], "Offline");
                } else {
                    assert_eq!(operation["action"], "UpdateService");
                }
                return (
                    200,
                    json!({"correlationId":"019a1234-0000-7000-8000-000000000099","results":[{"operationId":operation["operationId"],"status":if stage==0 {"accepted"} else {"conflict"},"entityVersion":if stage==0 {json!(1)} else {Value::Null},"state":null,"error":if stage==0 {Value::Null} else {json!({"code":"VERSION_CONFLICT","correlationId":"019a1234-0000-7000-8000-000000000099"})}}]}),
                );
            }
            if request.contains("/sync/changes?") {
                let changes = match stage {
                    0 => business_changes(&service_id),
                    1 => vec![offer(&service_id, 2, 3), offer(&service_id, 1, 99)],
                    2 => {
                        let mut removal = service(&service_id, 1);
                        removal.change_kind = ChangeKind::Remove;
                        removal.state = Value::Null;
                        vec![removal]
                    }
                    _ => panic!("unexpected download"),
                };
                return (
                    200,
                    json!({"groups":[group(changes)],"nextCursor":format!("stage-{stage}"),"hasMore":false}),
                );
            }
            if request.starts_with("DELETE ") {
                return (204, Value::Null);
            }
            if request.contains("/items?") {
                let items: Vec<_> = business_changes(&service_id).into_iter().map(|change| json!({"entityType":change.entity_type,"entityKey":change.entity_key,"entityVersion":change.entity_version,"state":change.state})).collect();
                return (200, json!({"items":items,"total":4}));
            }
            assert!(request.contains("/sync/snapshots"));
            (
                200,
                json!({"snapshotId":"019a1234-0000-7000-8000-000000000300","accountId":scope().account_id,"contractVersion":1,"modelRevision":"api-v1.1","window":cursor("base").window,"baseCursor":"fresh-base","expiresAt":"2099-11-04T23:00:00Z","itemCount":4}),
            )
        });
        let config = APIConfig::parse(&api.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
        let credential = credential(&config);
        let coordinator = SyncCoordinator::default();
        let engine = SyncEngine {
            coordinator: &coordinator,
            http: &http,
            database: &db.data,
        };
        let report = engine
            .run(
                &credential,
                &scope(),
                CycleTrigger::Manual,
                &SystemClock,
                &UuidV7Generator,
                &DesktopProjector,
            )
            .await
            .unwrap();
        assert_eq!(report.outcome, CycleOutcome::Complete);
        {
            let mut c = db.data.lock().unwrap();
            assert_eq!(count(&c, "account_view"), 1);
            assert_eq!(
                count(&c, "service_local"),
                1,
                "projectors must not resolve intentions"
            );
            let view = SqliteCatalogRepository { connection: &mut c }
                .catalog(&access)
                .unwrap();
            assert!(!view.services[0].is_provisional);
            assert_eq!(view.services[0].accepted.as_ref().unwrap().name, "Accepted");
            assert_eq!(count(&c, "service_local"), 0);
            assert_eq!(count(&c, "sync_outbox"), 1, "keep accepted receipt");
        }
        let conflict_id = {
            let mut c = db.data.lock().unwrap();
            let mut repo = SqliteCatalogRepository { connection: &mut c };
            CatalogUseCases {
                repository: &mut repo,
                access: &access,
                ids: &UuidV7Generator,
                clock: &SystemClock,
            }
            .update_service(
                id.clone().try_into().unwrap(),
                1,
                ServicePatch::new(Some("Keep intent".into()), None).unwrap(),
            )
            .unwrap()
            .operation_id
        };
        stage.store(1, Ordering::SeqCst);
        assert_eq!(
            engine
                .run(
                    &credential,
                    &scope(),
                    CycleTrigger::Manual,
                    &SystemClock,
                    &UuidV7Generator,
                    &DesktopProjector
                )
                .await
                .unwrap()
                .outcome,
            CycleOutcome::Complete
        );
        {
            let mut c = db.data.lock().unwrap();
            let mut repo = SqliteCatalogRepository { connection: &mut c };
            let view = repo.catalog(&access).unwrap();
            assert_eq!(view.services[0].local_changes[0].operation_id, conflict_id);
            assert_eq!(
                view.services[0].local_changes[0].code.as_deref(),
                Some("VERSION_CONFLICT")
            );
            let offer = &repo.list_offers(&scope()).unwrap()[0];
            assert_eq!((offer.version, offer.service_version), (2, 3));
        }
        stage.store(2, Ordering::SeqCst);
        ChangeDownloader {
            http: &http,
            database: &db.data,
        }
        .download(&credential, &scope(), &SystemClock, &DesktopProjector)
        .await
        .unwrap();
        {
            let mut c = open(&db.path).unwrap();
            let mut repo = SqliteCatalogRepository { connection: &mut c };
            assert!(repo.list_services(&scope()).unwrap().is_empty());
            assert_eq!(
                repo.catalog(&access).unwrap().services[0]
                    .provisional
                    .as_ref()
                    .unwrap()
                    .name,
                "Keep intent"
            );
            assert_eq!(count(&c, "sync_outbox"), 2);
        }
        stage.store(3, Ordering::SeqCst);
        let result = SnapshotRecovery {
            http: &http,
            database: &db.data,
        }
        .rebuild(
            &credential,
            &scope(),
            &SystemClock,
            SnapshotMode::Restart,
            &DesktopProjector,
        )
        .await
        .unwrap();
        assert!(matches!(
            result,
            SnapshotOutcome::Installed {
                cleanup_pending: false
            }
        ));
        {
            let mut c = open(&db.path).unwrap();
            assert_eq!(count(&c, "account_view"), 1);
            assert_eq!(
                count(&c, "professional_view"),
                2,
                "other account survives reset"
            );
            assert_eq!(count(&c, "service_local"), 1);
            assert_eq!(count(&c, "sync_outbox"), 2);
            let mut repo = SqliteCatalogRepository { connection: &mut c };
            let view = repo.catalog(&access).unwrap();
            assert!(view.services[0].accepted.is_some());
            assert_eq!(view.services[0].local_changes[0].operation_id, conflict_id);
            assert_eq!(view.services[0].local_changes[0].sync_status, "conflict");
            assert_eq!(
                repo.list_offers(&scope()).unwrap()[0].service_version,
                2,
                "fresh capture replaces prior generation"
            );
        }
        assert_eq!(db.cursor().cursor, "fresh-base");
        let requests = api.finish();
        assert_eq!(
            requests
                .iter()
                .filter(|r| r.contains("/sync/operations "))
                .count(),
            2
        );
        assert!(requests.iter().any(|r| r.contains("/items?")));
    });
}
