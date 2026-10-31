use super::*;

#[test]
fn receipt_log_preserves_ids_and_codes_without_remote_payload_or_error_text() {
    let record:crate::components::sync::adapters::api::OperationRecord=serde_json::from_value(serde_json::json!({
        "operationId":"019a1234-0000-7000-8000-000000000050","status":"conflict","entityVersion":2,
        "state":{"fullName":"PRIVATE_PATIENT","token":"PRIVATE_TOKEN"},
        "error":{"code":"VERSION_CONFLICT","correlationId":"019a1234-0000-7000-8000-000000000051","message":"PRIVATE_TOKEN","details":{"sql":"PRIVATE_SQL"}}
    })).unwrap();
    let event = receipt_event(&record, None, std::time::Duration::from_millis(3));
    let text = serde_json::to_string(&event).unwrap();
    for secret in [
        "PRIVATE_PATIENT",
        "PRIVATE_TOKEN",
        "PRIVATE_SQL",
        "fullName",
        "message",
        "details",
    ] {
        assert!(!text.contains(secret));
    }
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value["operationId"], record.operation_id.as_str());
    assert_eq!(
        value["correlationId"],
        "019a1234-0000-7000-8000-000000000051"
    );
    assert_eq!(value["causationId"], record.operation_id.as_str());
}
use crate::{
    components::{
        devices::activation::DeviceCredentials,
        identity::domain::AccessRole,
        sync::{application::record_operation, domain::NewOperation},
    },
    platform::{
        config::APIConfig,
        secrets::{Secret, testing::MemorySecretStore},
        sqlite::open,
        test_api::TestApi,
    },
    shared_kernel::{UuidV7Generator, id::OperationId},
};
use rusqlite::params;
use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime},
};

const ACCOUNT: &str = "019a1234-0000-7000-8000-000000000001";
const USER: &str = "019a1234-0000-7000-8000-000000000002";
const DEVICE: &str = "019a1234-0000-7000-8000-000000000003";
const OTHER: &str = "019a1234-0000-7000-8000-000000000004";

struct Database(PathBuf);

impl Database {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!(
            "nightingale-dispatch-{}.sqlite",
            uuid::Uuid::now_v7()
        )))
    }

    fn open(&self) -> Connection {
        open(&self.0).unwrap()
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        if self.0.exists() {
            fs::remove_file(&self.0).unwrap();
        }
    }
}

struct FixedClock(i64);

impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_millis(self.0 as u64)
    }
}

struct FixedId(u64);

impl IdGenerator for FixedId {
    fn new_id(&self) -> uuid::Uuid {
        uuid::Uuid::parse_str(&format!("019a1234-0000-7000-8000-{:012x}", self.0)).unwrap()
    }
}

fn id(value: u64) -> OperationId {
    OperationId::try_from(FixedId(value).new_id().to_string()).unwrap()
}

fn access() -> AccountAccess {
    AccountAccess {
        context: serde_json::from_value(
            json!({"accountId":ACCOUNT,"userId":USER,"deviceId":DEVICE,"permissionEpoch":1}),
        )
        .unwrap(),
        role: AccessRole::Owner,
        status: AccessStatus::Active,
        allowed_actions: vec!["RegisterPatient".into(), "UpdatePatientProfile".into()],
        checked_at_millis: 10,
    }
}

fn save_access(connection: &mut Connection, access: &AccountAccess) {
    let tx = connection.transaction().unwrap();

    SqliteAccessStore.save_access(&tx, access).unwrap();

    tx.commit().unwrap();
}

fn seed(connection: &mut Connection, number: u64, deps: Vec<OperationId>) {
    let tx = connection.transaction().unwrap();

    record_operation(
        &tx,
        &SqliteOperationStore,
        &FixedId(number),
        &FixedClock(number as i64),
        NewOperation {
            context: access().context,
            depends_on: deps,
            action: "RegisterPatient".into(),
            entity_type: "patient_profile".into(),
            entity_key: json!(id(number + 10_000)),
            expected_version: 0,
            payload: json!({"fullName":"Synthetic"}),
            occurred_at_local: None,
        },
    )
    .unwrap();

    tx.commit().unwrap();
}

fn lease(connection: &mut Connection, now: i64) -> Option<DispatchLease> {
    SqliteOperationStore
        .lease_batch(connection, &access(), now, 17)
        .unwrap()
}

fn row(connection: &Connection, number: u64) -> (String, i64, Option<i64>, Option<i64>) {
    connection.query_row("SELECT status,attempts,lease_until,next_attempt_at FROM sync_outbox WHERE operation_id=?1",
        [id(number).as_str()], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?))).unwrap()
}

#[test]
fn causal_order_skips_failed_missing_cyclic_and_foreign_dependencies() {
    let db = Database::new();
    let mut connection = db.open();
    save_access(&mut connection, &access());
    seed(&mut connection, 10, vec![id(20)]); // Ancestor has a later local timestamp.
    seed(&mut connection, 20, vec![]);
    seed(&mut connection, 30, vec![id(99)]);
    seed(&mut connection, 40, vec![id(50)]);
    seed(&mut connection, 50, vec![id(40)]);
    seed(&mut connection, 60, vec![]);
    seed(&mut connection, 70, vec![id(60)]);
    connection
        .execute(
            "UPDATE sync_outbox SET status='rejected' WHERE operation_id=?1",
            [id(60).as_str()],
        )
        .unwrap();
    seed(&mut connection, 80, vec![]);
    connection
        .execute(
            "UPDATE sync_outbox SET status='accepted',user_id=?2 WHERE operation_id=?1",
            params![id(80).as_str(), OTHER],
        )
        .unwrap();
    seed(&mut connection, 90, vec![id(80)]);
    seed(&mut connection, 100, vec![]);
    connection
        .execute(
            "UPDATE sync_outbox SET status='accepted' WHERE operation_id=?1",
            [id(100).as_str()],
        )
        .unwrap();
    seed(&mut connection, 110, vec![id(100)]);
    let batch = lease(&mut connection, 1_000).unwrap();
    let ids: Vec<_> = batch
        .operations()
        .into_iter()
        .map(|op| op.operation_id)
        .collect();
    assert_eq!(ids, vec![id(20), id(110), id(10)]);
    for blocked in [30, 40, 50, 70, 90] {
        assert_eq!(row(&connection, blocked).0, "pending");
    }
    assert!(lease(&mut connection, 1_001).is_none());
}

#[test]
fn lease_is_atomic_between_connections_and_expiry_fences_late_failures() {
    let db = Database::new();
    let mut first = db.open();
    save_access(&mut first, &access());
    seed(&mut first, 1, vec![]);
    let mut second = db.open();
    let original = lease(&mut first, 100).unwrap();
    assert!(lease(&mut second, 101).is_none());
    let next = row(&first, 1).3.unwrap();
    assert!(lease(&mut second, 100 + LEASE_MILLIS).is_none());
    let retry = lease(&mut second, next).unwrap();
    assert_eq!(original.operations(), retry.operations());
    assert_eq!(row(&first, 1).1, 2);
    SqliteOperationStore
        .release_unknown(
            &mut first,
            &original,
            next,
            crate::platform::http::APICode::TemporaryFailure,
        )
        .unwrap();
    assert_eq!(row(&second, 1).0, "in_flight");
    assert_eq!(row(&second, 1).2, Some(retry.until));
}

#[test]
fn restart_preserves_envelopes_fingerprints_attempts_and_scheduled_retry() {
    let db = Database::new();
    let mut connection = db.open();
    save_access(&mut connection, &access());
    seed(&mut connection, 1, vec![]);
    seed(&mut connection, 2, vec![id(1)]);
    let before = lease(&mut connection, 100).unwrap();
    let fingerprints: Vec<_> = before
        .operations()
        .iter()
        .map(|op| op.semantic_fingerprint().unwrap())
        .collect();
    let next = row(&connection, 1)
        .3
        .unwrap()
        .max(row(&connection, 2).3.unwrap());
    drop(connection);
    let mut connection = db.open();
    assert_eq!(row(&connection, 1).0, "unknown");
    assert_eq!(row(&connection, 1).1, 1);
    assert!(lease(&mut connection, 101).is_none());
    let after = lease(&mut connection, next).unwrap();
    assert_eq!(before.operations(), after.operations());
    assert_eq!(
        fingerprints,
        after
            .operations()
            .iter()
            .map(|op| op.semantic_fingerprint().unwrap())
            .collect::<Vec<_>>()
    );
    assert_eq!(row(&connection, 1).1, 2);
    assert_eq!(row(&connection, 2).1, 2);
}

#[test]
fn retry_delay_grows_has_jitter_and_is_capped_without_overflow() {
    for (attempt, low, high) in [
        (1, 500, 1000),
        (2, 1000, 2000),
        (9, 128000, 256000),
        (i64::MAX, 150000, 300000),
    ] {
        assert_eq!(retry_delay(attempt, 0), low);
        assert_eq!(retry_delay(attempt, (high - low) as u64), high);
        assert!((low..=high).contains(&retry_delay(attempt, u64::MAX)));
    }
}

#[test]
fn marking_failure_rolls_back_every_attempt_and_maximum_dependencies_keep_order() {
    let db = Database::new();
    let mut connection = db.open();
    save_access(&mut connection, &access());
    for number in 1..=8 {
        seed(&mut connection, number, vec![]);
    }
    seed(&mut connection, 9, (1..=8).map(id).collect());
    connection
        .execute_batch(
            "CREATE TEMP TRIGGER fail_attempt BEFORE UPDATE OF attempts ON sync_outbox
        WHEN NEW.attempts>0 AND NEW.operation_id='019a1234-0000-7000-8000-000000000009'
        BEGIN SELECT RAISE(ABORT,'injected failure'); END;",
        )
        .unwrap();
    assert!(
        SqliteOperationStore
            .lease_batch(&mut connection, &access(), 1_000, 0)
            .is_err()
    );
    for number in 1..=9 {
        assert_eq!(row(&connection, number), ("pending".into(), 0, None, None));
    }
    connection
        .execute_batch("DROP TRIGGER fail_attempt;")
        .unwrap();
    let batch = lease(&mut connection, 1_000).unwrap();
    assert_eq!(
        batch
            .operations()
            .into_iter()
            .map(|op| op.operation_id)
            .collect::<Vec<_>>(),
        (1..=9).map(id).collect::<Vec<_>>()
    );
    let mut invalid = batch.operations().pop().unwrap();
    invalid.depends_on.push(id(10));
    assert!(invalid.canonical_json().is_err());
}

#[test]
fn access_http_denials_and_uncertain_post_errors_keep_ids_and_disable_only_denied_access() {
    tauri::async_runtime::block_on(async {
        for (status, post) in [
            (401, false),
            (403, false),
            (401, true),
            (403, true),
            (503, true),
            (422, true),
        ] {
            let db = Database::new();
            let mut connection = db.open();
            save_access(&mut connection, &access());
            seed(&mut connection, 1, vec![]);
            let server = TestApi::new(if post { 2 } else { 1 }, move |request| {
                if post && request.starts_with("GET ") {
                    return (200, access_json("active", 1));
                }
                (
                    status,
                    json!({"error":{"code":if status==401 {"AUTHENTICATION_REQUIRED"} else if status==403 {"PERMISSION_DENIED"} else {"TEMPORARY_FAILURE"},
                    "message":"synthetic-device-token","details":{"token":"synthetic-device-token"},"correlationId":OTHER}}),
                )
            });
            let config = APIConfig::parse(&server.url).unwrap();
            let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
            let secrets = MemorySecretStore::default();
            let credential = credential(&config, &secrets);
            let database = Mutex::new(connection);
            let result = OutboxDispatcher {
                http: &http,
                database: &database,
            }
            .dispatch(
                &credential,
                &access().context,
                &FixedClock(1_000),
                &UuidV7Generator,
            )
            .await;
            assert!(matches!(result, Err(DispatchFailure::Remote(_))));
            let guard = database.lock().unwrap();
            assert_eq!(row(&guard, 1).0, if post { "unknown" } else { "pending" });
            assert_eq!(row(&guard, 1).1, i64::from(post));
            let cached = SqliteAccessStore
                .access(
                    &guard,
                    &access().context.account_id,
                    &access().context.user_id,
                    &access().context.device_id,
                )
                .unwrap()
                .unwrap();
            assert_eq!(
                cached.status,
                if status == 401 || status == 403 {
                    AccessStatus::Inactive
                } else {
                    AccessStatus::Active
                }
            );
            let error: Option<String> = guard
                .query_row(
                    "SELECT last_error FROM sync_outbox WHERE operation_id=?1",
                    [id(1).as_str()],
                    |row| row.get(0),
                )
                .unwrap();
            assert!(!error.unwrap_or_default().contains("synthetic-device-token"));
            drop(guard);
            server.finish();
        }
    });
}

#[test]
fn batch_limits_and_invalid_intentions_do_not_block_independent_work() {
    let db = Database::new();
    let mut connection = db.open();
    save_access(&mut connection, &access());
    for number in 1..=102 {
        seed(&mut connection, number, vec![]);
    }
    connection
        .execute(
            "UPDATE sync_outbox SET semantic_fingerprint='invalid' WHERE operation_id=?1",
            [id(1).as_str()],
        )
        .unwrap();
    let batch = lease(&mut connection, 1_000).unwrap();
    assert_eq!(batch.entries.len(), 100);
    assert_eq!(batch.entries[0].envelope.operation_id, id(2));
    assert_eq!(row(&connection, 1).0, "pending");
    assert_eq!(row(&connection, 102).0, "pending");

    // Force expiry without accepting any operation; large valid envelopes must split.
    connection.execute("DELETE FROM sync_outbox", []).unwrap();
    for number in 1..=18 {
        let tx = connection.transaction().unwrap();
        record_operation(
            &tx,
            &SqliteOperationStore,
            &FixedId(number),
            &FixedClock(number as i64),
            NewOperation {
                context: access().context,
                depends_on: vec![],
                action: "RegisterPatient".into(),
                entity_type: "patient_profile".into(),
                entity_key: json!(id(number + 10_000)),
                expected_version: 0,
                payload: json!({"fullName":"á".repeat(30_000)}),
                occurred_at_local: None,
            },
        )
        .unwrap();
        tx.commit().unwrap();
    }
    let batch = lease(&mut connection, 1_000).unwrap();
    assert_eq!(batch.entries.len(), 17);
    let body = serde_json::to_vec(&json!({"operations":batch.operations()})).unwrap();
    assert!(body.len() <= 1024 * 1024);
    assert!(
        batch
            .operations()
            .iter()
            .all(|op| op.canonical_json().unwrap().len() <= 64 * 1024)
    );
    assert_eq!(row(&connection, 18).1, 0);
}

#[test]
fn authorization_scope_epoch_schedule_and_bootstrap_are_not_relabelled() {
    let db = Database::new();
    let mut connection = db.open();
    save_access(&mut connection, &access());
    for number in 1..=6 {
        seed(&mut connection, number, vec![]);
    }
    for (number, column, value) in [
        (1, "account_id", OTHER),
        (2, "user_id", OTHER),
        (3, "device_id", OTHER),
    ] {
        connection
            .execute(
                &format!("UPDATE sync_outbox SET {column}=?2 WHERE operation_id=?1"),
                params![id(number).as_str(), value],
            )
            .unwrap();
    }
    connection
        .execute(
            "UPDATE sync_outbox SET permission_epoch=2 WHERE operation_id=?1",
            [id(4).as_str()],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE sync_outbox SET next_attempt_at=2000 WHERE operation_id=?1",
            [id(5).as_str()],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE sync_outbox SET action='RegisterAccountDevice' WHERE operation_id=?1",
            [id(6).as_str()],
        )
        .unwrap();
    assert!(lease(&mut connection, 1_000).is_none());
    let mut revoked = access();
    revoked.status = AccessStatus::Revoked;
    save_access(&mut connection, &revoked);
    assert!(
        SqliteOperationStore
            .lease_batch(&mut connection, &access(), 3_000, 0)
            .is_err()
    );
    for number in 1..=6 {
        assert_eq!(row(&connection, number).1, 0);
    }
}

fn access_json(status: &str, epoch: i64) -> Value {
    json!({"accountId":ACCOUNT,"userId":USER,"deviceId":DEVICE,"permissionEpoch":epoch,
        "role":"owner","status":status,"allowedActions":["RegisterPatient","UpdatePatientProfile"]})
}

fn credential(config: &APIConfig, secrets: &MemorySecretStore) -> DeviceCredential {
    use crate::platform::secrets::{CredentialKey, CredentialKind, SecretStore};
    for (device, kind, value) in [
        (
            None,
            CredentialKind::APIDeviceState,
            json!({"status":"issued","device_id":DEVICE,"user_id":USER}).to_string(),
        ),
        (
            Some(crate::shared_kernel::id::DeviceId::try_from(DEVICE.to_owned()).unwrap()),
            CredentialKind::APIDevice,
            "synthetic-device-token".into(),
        ),
    ] {
        let key = CredentialKey::new(
            config,
            "dispatch",
            ("https://issuer.example/", "subject"),
            None,
            device.as_ref(),
            kind,
        )
        .unwrap();
        secrets.write(&key, &Secret::new(value).unwrap()).unwrap();
    }
    DeviceCredentials {
        config,
        profile: "dispatch",
        identity: ("https://issuer.example/", "subject"),
        secrets,
    }
    .load()
    .unwrap()
    .unwrap()
}

#[test]
fn http_failure_then_retry_sends_same_body_without_sqlite_lock_and_saves_receipts() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        let mut connection = db.open();
        seed(&mut connection, 1, vec![]);
        seed(&mut connection, 2, vec![id(1)]);
        let database = Arc::new(Mutex::new(connection));
        let data = database.clone();
        let mut posts = 0;
        let server = TestApi::new(5, move |request| {
            assert!(data.try_lock().is_ok(), "HTTP mantiene el mutex SQLite");
            assert!(
                request
                    .to_ascii_lowercase()
                    .contains("authorization: bearer synthetic-device-token")
            );
            if request.starts_with("GET ") {
                return (200, access_json("active", 1));
            }
            posts += 1;
            let guard = data.lock().unwrap();
            assert_eq!(row(&guard, 1).0, "in_flight");
            assert_eq!(row(&guard, 1).1, posts);
            drop(guard);
            if posts == 1 {
                return (0, Value::Null);
            }
            let body: Value =
                serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
            (
                200,
                json!({"results":body["operations"].as_array().unwrap().iter().map(|op|json!({
                "operationId":op["operationId"],"status":"accepted","entityVersion":1,"state":{},"error":null
            })).collect::<Vec<_>>(),"correlationId":OTHER}),
            )
        });
        let config = APIConfig::parse(&server.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
        let secrets = MemorySecretStore::default();
        let credential = credential(&config, &secrets);
        let dispatcher = OutboxDispatcher {
            http: &http,
            database: &database,
        };
        assert!(matches!(
            dispatcher
                .dispatch(
                    &credential,
                    &access().context,
                    &FixedClock(1_000),
                    &UuidV7Generator
                )
                .await,
            Err(DispatchFailure::Remote(_))
        ));
        assert_eq!(row(&database.lock().unwrap(), 1).0, "unknown");
        assert!(matches!(
            dispatcher
                .dispatch(
                    &credential,
                    &access().context,
                    &FixedClock(1_001),
                    &UuidV7Generator
                )
                .await
                .unwrap(),
            DispatchOutcome::Idle
        ));
        let outcome = dispatcher
            .dispatch(
                &credential,
                &access().context,
                &FixedClock(5_000),
                &UuidV7Generator,
            )
            .await
            .unwrap();
        let DispatchOutcome::Received { lease, response } = outcome else {
            panic!("falta respuesta");
        };
        assert_eq!(lease.entries.len(), 2);
        assert_eq!(response.results.len(), 2);
        assert_eq!(row(&database.lock().unwrap(), 1).0, "accepted");
        let requests = server.finish();
        let posts: Vec<_> = requests
            .iter()
            .filter(|request| request.starts_with("POST "))
            .map(|request| request.split("\r\n\r\n").nth(1).unwrap())
            .collect();
        assert_eq!(posts.len(), 2);
        assert_eq!(posts[0], posts[1]);
    });
}

#[path = "reconciliation_tests.rs"]
mod reconciliation;

#[test]
fn refreshed_denial_or_epoch_change_stops_dispatch_and_preserves_intentions() {
    tauri::async_runtime::block_on(async {
        for (status, epoch) in [
            ("revoked", 1),
            ("inactive", 1),
            ("registration_required", 1),
            ("active", 2),
        ] {
            let db = Database::new();
            let mut connection = db.open();
            seed(&mut connection, 1, vec![]);
            let server = TestApi::new(1, move |request| {
                assert!(request.starts_with("GET "));
                (200, access_json(status, epoch))
            });
            let config = APIConfig::parse(&server.url).unwrap();
            let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
            let secrets = MemorySecretStore::default();
            let credential = credential(&config, &secrets);
            let database = Mutex::new(connection);
            let result = OutboxDispatcher {
                http: &http,
                database: &database,
            }
            .dispatch(
                &credential,
                &access().context,
                &FixedClock(100),
                &UuidV7Generator,
            )
            .await;
            if epoch == 2 {
                assert!(matches!(result, Err(DispatchFailure::AccessChanged)));
            } else {
                assert!(matches!(result, Err(DispatchFailure::AccessDenied)));
            }
            assert_eq!(row(&database.lock().unwrap(), 1).0, "pending");
            assert_eq!(row(&database.lock().unwrap(), 1).1, 0);
            server.finish();
        }
    });
}
