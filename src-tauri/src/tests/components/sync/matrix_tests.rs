use crate::components::sync::{
    adapters::{api::SyncAPI, control::SqliteSyncControlStore, outbound::SqliteOperationStore},
    application::{RemoteProjectionStore, record_operation},
    coordinator::{CycleOutcome, CycleTrigger, SyncCoordinator, SyncEngine},
    domain::NewOperation,
    events::StreamOutcome,
};
use crate::tests::components::sync::download_tests::{Database, cursor, group, scope};
use crate::{
    components::{
        devices::activation::{DeviceCredential, DeviceCredentials},
        identity::{application::AccountActivation, oidc::OidcSession},
    },
    platform::{
        config::APIConfig,
        http::APITransport,
        secrets::{CredentialKey, CredentialKind, Secret, SecretStore, testing::MemorySecretStore},
        sqlite::open,
        test_api::TestApi,
    },
    shared_kernel::{SystemClock, UuidV7Generator, id::OperationContext},
};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

#[derive(Default)]
struct Server {
    registered: HashSet<String>,
    receipts: HashMap<String, Value>,
    envelopes: HashMap<String, String>,
    patient: Option<Value>,
    journal: Option<Value>,
    effects: usize,
    lose_next: bool,
    offline: bool,
    revoked: bool,
    expire_cursor: bool,
    expire_capture: bool,
    capture: Option<Value>,
    snapshots: usize,
}

fn error(code: &str) -> Value {
    json!({"code":code,"message":"synthetic error","details":[],"correlationId":"019a1234-0000-7000-8000-000000000099"})
}

fn scope_for(b: bool) -> OperationContext {
    let mut s = scope();

    if b {
        s.device_id =
            serde_json::from_value(json!("019a1234-0000-7000-8000-000000000004")).unwrap();
    }

    s
}

impl Server {
    fn reply(&mut self, request: &str) -> (u16, Value) {
        if self.offline {
            return (503, error("SERVICE_UNAVAILABLE"));
        }

        let b = request.contains("Bearer oidc-B") || request.contains("Bearer device-B");
        let token = if b { "device-B" } else { "device-A" };
        let s = scope_for(b);
        let line = request.lines().next().unwrap();
        let path = line.split_whitespace().nth(1).unwrap();

        if path == "/v1/auth/device-tokens" {
            return (
                200,
                json!({"accessToken":token,"tokenId":"019a1234-0000-7000-8000-000000000050","deviceId":s.device_id,"userId":s.user_id,"issuedAt":"2026-10-30T12:00:00Z"}),
            );
        }

        let account = json!({"id":s.account_id,"displayName":"Synthetic practice","role":"owner","status":"active"});

        if path == "/v1/me" {
            return (
                200,
                json!({"userId":s.user_id,"displayName":"Synthetic user","status":"active","oidcIssuer":"https://issuer.example/","oidcSubject":"subject","accounts":{"items":[account],"total":1}}),
            );
        }

        if path.starts_with("/v1/accounts?") {
            return (200, json!({"items":[account],"total":1}));
        }

        if self.revoked && b {
            return (403, error("PERMISSION_DENIED"));
        }

        if path.ends_with("/access") {
            return (
                200,
                json!({"accountId":s.account_id,"userId":s.user_id,"deviceId":s.device_id,"permissionEpoch":1,"role":"owner","status":if self.registered.contains(token){"active"}else{"registration_required"},"allowedActions":["RegisterAccountDevice","RegisterPatient"]}),
            );
        }

        if path.contains("/sync/operations/") {
            let id = path.rsplit('/').next().unwrap();

            return self
                .receipts
                .get(id)
                .map(|r| (200, r.clone()))
                .unwrap_or((404, error("NOT_FOUND")));
        }

        if path.ends_with("/sync/operations") {
            let body: Value =
                serde_json::from_str(request.split_once("\r\n\r\n").unwrap().1).unwrap();
            let mut results = Vec::new();
            let mut lost = false;

            for op in body["operations"].as_array().unwrap() {
                let id = op["operationId"].as_str().unwrap().to_owned();
                let encoded = op.to_string();

                if let Some(original) = self.receipts.get(&id) {
                    assert_eq!(self.envelopes[&id], encoded);
                    let mut duplicate = original.clone();
                    duplicate["status"] = json!("duplicate");
                    duplicate["originalResult"] = original.clone();
                    results.push(duplicate);

                    continue;
                }

                let record = if op["action"] == "RegisterAccountDevice" {
                    assert_eq!(op["entityKey"], json!(s.device_id));
                    self.registered.insert(token.into());
                    json!({"operationId":id,"status":"accepted","entityVersion":1,"state":{"deviceId":s.device_id,"status":"active","version":1,"displayName":"Matrix device","clientVersion":"0.1.0","localSchemaVersion":4,"lastSeenAt":null},"error":null})
                } else if self.patient.is_none() {
                    let mut change = serde_json::to_value(group().changes.remove(5)).unwrap();
                    change["state"]["fullName"] = op["payload"]["fullName"].clone();
                    change["state"]["patientId"] = op["entityKey"].clone();
                    change["entityKey"] = op["entityKey"].clone();
                    self.patient = Some(change.clone());
                    self.journal = Some(
                        json!({"groupId":id,"operationId":id,"acceptedAt":"2026-10-30T12:00:00Z","changes":[change]}),
                    );
                    self.effects += 1;
                    lost = self.lose_next;
                    self.lose_next = false;
                    json!({"operationId":id,"status":"accepted","entityVersion":1,"state":self.patient.as_ref().unwrap()["state"],"error":null})
                } else {
                    json!({"operationId":id,"status":"conflict","entityVersion":1,"state":self.patient.as_ref().unwrap()["state"],"error":error("VERSION_CONFLICT")})
                };

                self.envelopes.insert(id.clone(), encoded);
                self.receipts.insert(id, record.clone());

                results.push(record);
            }

            return (
                if lost { 0 } else { 200 },
                json!({"results":results,"correlationId":"019a1234-0000-7000-8000-000000000099"}),
            );
        }

        if path.contains("/sync/events?") {
            return (503, error("SERVICE_UNAVAILABLE"));
        }

        if path.contains("/sync/changes?") {
            if self.expire_cursor {
                self.expire_cursor = false;

                return (410, error("CURSOR_EXPIRED"));
            }

            let groups = if path.contains("cursor=cursor-0") {
                self.journal.clone().into_iter().collect::<Vec<_>>()
            } else {
                vec![]
            };

            return (
                200,
                json!({"groups":groups,"nextCursor":if self.patient.is_some(){"cursor-1"}else{"cursor-0"},"hasMore":false}),
            );
        }

        if path.ends_with("/sync/snapshots") {
            self.snapshots += 1;
            let snapshot = json!({
                "snapshotId":uuid::Uuid::now_v7().to_string(),
                "accountId":s.account_id,"contractVersion":1,
                "modelRevision":"api-v1.1",
                "window":cursor("base").window,"baseCursor": if self.patient.is_some(){
                    "cursor-1"
                } else{
                    "cursor-0"
                },
                "expiresAt":"2026-10-30T23:00:00Z",
                "itemCount":usize::from(self.patient.is_some())
            });
            self.capture = Some(snapshot.clone());

            return (200, snapshot);
        }

        if path.contains("/sync/snapshots/") {
            if line.starts_with("DELETE ") {
                return (204, Value::Null);
            }

            if path.contains("/items?") {
                if self.expire_capture {
                    self.expire_capture = false;

                    return (410, error("SNAPSHOT_EXPIRED"));
                }

                let mut item = self.patient.clone().unwrap();
                item.as_object_mut().unwrap().remove("changeKind");

                return (200, json!({"items":[item],"total":1}));
            }

            return (200, self.capture.clone().unwrap());
        }

        panic!("unexpected synthetic route {line}");
    }
}

async fn activate(
    config: &APIConfig,
    http: &APITransport,
    db: &Database,
    b: bool,
) -> (DeviceCredential, OperationContext, MemorySecretStore) {
    let secrets = MemorySecretStore::default();
    let profile = if b { "matrix-B" } else { "matrix-A" };
    let identity = ("https://issuer.example/", "subject");
    let key = CredentialKey::new(
        config,
        profile,
        identity,
        None,
        None,
        CredentialKind::OidcAccess,
    )
    .unwrap();
    secrets
        .write(
            &key,
            &Secret::new(if b { "oidc-B" } else { "oidc-A" }.into()).unwrap(),
        )
        .unwrap();
    let session = OidcSession::load(config, profile, identity, &secrets)
        .unwrap()
        .unwrap();
    let access = AccountActivation {
        config,
        profile,
        http,
        secrets: &secrets,
        database: &db.data,
    }
    .activate(
        &session,
        &scope().account_id,
        &UuidV7Generator,
        &SystemClock,
    )
    .await
    .unwrap();
    let credential = DeviceCredentials {
        config,
        profile,
        identity,
        secrets: &secrets,
    }
    .load()
    .unwrap()
    .unwrap();

    (credential, access.context, secrets)
}

fn record(db: &Database, context: OperationContext, name: &str) -> uuid::Uuid {
    let mut c = db.data.lock().unwrap();
    let tx = c.transaction().unwrap();
    let id = record_operation(
        &tx,
        &SqliteOperationStore,
        &UuidV7Generator,
        &SystemClock,
        NewOperation {
            context,
            depends_on: vec![],
            action: "RegisterPatient".into(),
            entity_type: "patient_profile".into(),
            entity_key: json!("019a1234-0000-7000-8000-000000000010"),
            expected_version: 0,
            payload: json!({"fullName":name}),
            occurred_at_local: None,
        },
    )
    .unwrap();

    tx.commit().unwrap();

    id
}

#[test]
fn two_isolated_profiles_activate_recover_lost_commit_conflict_revoke_and_rebuild_without_sse() {
    tauri::async_runtime::block_on(async {
        let a = Database::new();
        let b = Database::new();
        assert_ne!(a.path, b.path);

        for db in [&a, &b] {
            db.data
                .lock()
                .unwrap()
                .execute_batch("DELETE FROM identity_account_access;DELETE FROM sync_cursor;")
                .unwrap();
        }

        let server = Arc::new(Mutex::new(Server::default()));
        let state = server.clone();
        let api = TestApi::dynamic(move |r| state.lock().unwrap().reply(r));
        let config = APIConfig::parse(&api.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
        let (cred_a, scope_a, _secrets_a) = activate(&config, &http, &a, false).await;
        let (cred_b, scope_b, _secrets_b) = activate(&config, &http, &b, true).await;
        assert_ne!(cred_a.device_id, cred_b.device_id);
        let ca = SyncCoordinator::default();
        let cb = SyncCoordinator::default();
        let ea = SyncEngine {
            http: &http,
            database: &a.data,
            coordinator: &ca,
        };
        let eb = SyncEngine {
            http: &http,
            database: &b.data,
            coordinator: &cb,
        };

        for (engine, cred, scope) in [(&ea, &cred_a, &scope_a), (&eb, &cred_b, &scope_b)] {
            assert_eq!(
                engine
                    .run(
                        cred,
                        scope,
                        CycleTrigger::Startup,
                        &SystemClock,
                        &UuidV7Generator,
                        &()
                    )
                    .await
                    .unwrap()
                    .outcome,
                CycleOutcome::Complete
            );
        }
        let id = record(&a, scope_a.clone(), "Accepted synthetic patient");
        server.lock().unwrap().offline = true;
        assert_eq!(
            ea.run(
                &cred_a,
                &scope_a,
                CycleTrigger::Manual,
                &SystemClock,
                &UuidV7Generator,
                &()
            )
            .await
            .unwrap()
            .outcome,
            CycleOutcome::RetryWaiting
        );
        server.lock().unwrap().offline = false;
        server.lock().unwrap().lose_next = true;
        assert_eq!(
            ea.run(
                &cred_a,
                &scope_a,
                CycleTrigger::Manual,
                &SystemClock,
                &UuidV7Generator,
                &()
            )
            .await
            .unwrap()
            .outcome,
            CycleOutcome::Offline
        );
        assert_eq!(server.lock().unwrap().effects, 1);
        let id = serde_json::from_value(json!(id.to_string())).unwrap();
        let before = SqliteOperationStore
            .operation_result(&a.data.lock().unwrap(), &scope_a, &id)
            .unwrap()
            .unwrap()
            .operation
            .envelope()
            .unwrap();
        *a.data.lock().unwrap() = open(&a.path).unwrap();
        assert_eq!(
            ea.run(
                &cred_a,
                &scope_a,
                CycleTrigger::Manual,
                &SystemClock,
                &UuidV7Generator,
                &()
            )
            .await
            .unwrap()
            .outcome,
            CycleOutcome::Complete
        );
        assert_eq!(
            eb.run(
                &cred_b,
                &scope_b,
                CycleTrigger::Reconnect,
                &SystemClock,
                &UuidV7Generator,
                &()
            )
            .await
            .unwrap()
            .outcome,
            CycleOutcome::Complete
        );
        let duplicate = SyncAPI(&http)
            .send_operations(
                &cred_b.token,
                &scope_b.account_id,
                std::slice::from_ref(&before),
            )
            .await
            .unwrap();
        assert_eq!(
            duplicate.value.results[0].status,
            crate::components::sync::adapters::api::ResultStatus::Duplicate
        );

        for (db, s) in [(&a, &scope_a), (&b, &scope_b)] {
            let item = SqliteSyncControlStore
                .item(
                    &db.data.lock().unwrap(),
                    s,
                    "patient_profile",
                    &before.entity_key,
                )
                .unwrap()
                .unwrap();
            assert_eq!(item.state["fullName"], "Accepted synthetic patient");
        }

        record(&b, scope_b.clone(), "Conflicting offline intention");
        let conflict = eb
            .run(
                &cred_b,
                &scope_b,
                CycleTrigger::Manual,
                &SystemClock,
                &UuidV7Generator,
                &(),
            )
            .await
            .unwrap();
        assert_eq!(conflict.status.outbox.conflicts, 1);
        server.lock().unwrap().revoked = true;
        assert_eq!(
            eb.run(
                &cred_b,
                &scope_b,
                CycleTrigger::Reconnect,
                &SystemClock,
                &UuidV7Generator,
                &()
            )
            .await
            .unwrap()
            .outcome,
            CycleOutcome::AccessDenied
        );
        server.lock().unwrap().expire_cursor = true;
        server.lock().unwrap().expire_capture = true;
        let count = server.lock().unwrap().snapshots;
        assert_eq!(
            ea.run(
                &cred_a,
                &scope_a,
                CycleTrigger::Manual,
                &SystemClock,
                &UuidV7Generator,
                &()
            )
            .await
            .unwrap()
            .outcome,
            CycleOutcome::Complete
        );
        assert_eq!(server.lock().unwrap().snapshots, count + 2);
        assert_eq!(
            ea.listen_once(&cred_a, &scope_a, &()).await.unwrap(),
            StreamOutcome::Retry
        );
        assert_eq!(
            ea.run(
                &cred_a,
                &scope_a,
                CycleTrigger::Reconnect,
                &SystemClock,
                &UuidV7Generator,
                &()
            )
            .await
            .unwrap()
            .outcome,
            CycleOutcome::Complete
        );
        let after = SqliteOperationStore
            .operation_result(&a.data.lock().unwrap(), &scope_a, &id)
            .unwrap()
            .unwrap();
        assert_eq!(after.operation.envelope().unwrap(), before);
        assert_eq!(
            after.status,
            crate::components::sync::domain::OperationStatus::Accepted
        );
        assert_eq!(server.lock().unwrap().effects, 1);
        assert_eq!(cb.status(&b.data, &scope_b).unwrap().outbox.conflicts, 1);
        let requests = api.finish();
        assert!(requests.iter().any(|r| r.contains("/sync/events?")));
    });
}
