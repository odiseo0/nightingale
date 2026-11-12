use super::*;
use crate::{
    components::sync::{
        adapters::api::{OperationRecord, OperationResult},
        domain::OperationStatus,
    },
    platform::http::APICode,
};
use std::sync::atomic::{AtomicUsize, Ordering};

fn record(number: u64, status: &str) -> OperationRecord {
    serde_json::from_value(record_json(number, status)).unwrap()
}

fn record_json(number: u64, status: &str) -> Value {
    json!({"operationId":id(number),"status":status,"entityVersion":7,
        "state":{"fullName":"Remote state","version":7},
        "error":if status=="accepted" {Value::Null} else {json!({"code":"DEPENDENCY_FAILED","correlationId":OTHER,
            "message":"synthetic-device-token","details":{"secret":"synthetic-device-token"}})}})
}

fn normalized(number: u64, status: &str, duplicate: bool) -> OperationRecord {
    let value = if duplicate {
        json!({"operationId":id(number),"status":"duplicate","entityVersion":null,"state":null,"error":null,"originalResult":record_json(number,status)})
    } else {
        record_json(number, status)
    };

    serde_json::from_value::<OperationResult>(value)
        .unwrap()
        .original()
        .unwrap()
}

#[test]
fn every_original_and_duplicate_result_keeps_intention_and_remote_state_after_reopen() {
    let db = Database::new();
    let mut connection = db.open();
    save_access(&mut connection, &access());

    for number in 1..=8 {
        seed(&mut connection, number, vec![]);
    }

    let batch = lease(&mut connection, 1_000).unwrap();
    let mut records = Vec::new();

    for (index, status) in ["accepted", "conflict", "rejected", "retryable_failure"]
        .iter()
        .enumerate()
    {
        records.push(normalized(index as u64 + 1, status, false));
        records.push(normalized(index as u64 + 5, status, true));
    }

    records.reverse();
    SqliteOperationStore
        .reconcile(&mut connection, &batch, &records, 1_001)
        .unwrap();
    let changes = connection.total_changes();
    let bytes = fs::read(&db.0).unwrap();
    SqliteOperationStore
        .reconcile(&mut connection, &batch, &records, 1_002)
        .unwrap();
    assert_eq!(connection.total_changes(), changes);
    assert_eq!(fs::read(&db.0).unwrap(), bytes);
    drop(connection);
    let connection = db.open();

    for entry in &batch.entries {
        let saved = SqliteOperationStore
            .operation_result(&connection, &access().context, &entry.envelope.operation_id)
            .unwrap()
            .unwrap();
        assert_eq!(saved.operation.envelope().unwrap(), entry.envelope);
        assert_eq!(
            saved.operation.semantic_fingerprint,
            Some(entry.envelope.semantic_fingerprint().unwrap())
        );
        let original = records
            .iter()
            .find(|record| record.operation_id == entry.envelope.operation_id)
            .unwrap();
        assert_eq!(saved.status, original.local_status());
        assert_eq!(
            saved.result.as_ref().unwrap()["state"],
            record_json(1, "accepted")["state"]
        );
        assert!(
            !saved
                .result
                .unwrap()
                .to_string()
                .contains("synthetic-device-token")
        );
        assert!(saved.review_code.is_none());
    }
    let count: i64 = connection
        .query_row("SELECT count(*) FROM sync_remote_items", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(count, 0, "los recibos no sustituyen la proyección remota");
}

#[test]
fn invalid_or_partial_receipts_and_storage_failure_leave_whole_batch_available() {
    let db = Database::new();
    let mut connection = db.open();
    save_access(&mut connection, &access());
    seed(&mut connection, 1, vec![]);
    seed(&mut connection, 2, vec![]);
    let batch = lease(&mut connection, 100).unwrap();

    for records in [
        vec![record(1, "accepted")],
        vec![record(1, "accepted"), record(1, "accepted")],
        vec![record(1, "accepted"), record(3, "accepted")],
    ] {
        assert!(
            SqliteOperationStore
                .reconcile(&mut connection, &batch, &records, 101)
                .is_err()
        );
    }

    connection.execute_batch("CREATE TEMP TRIGGER fail_receipt BEFORE UPDATE OF result ON sync_outbox
        WHEN NEW.operation_id='019a1234-0000-7000-8000-000000000002' BEGIN SELECT RAISE(ABORT,'injected failure'); END;").unwrap();
    let records = vec![record(1, "accepted"), record(2, "conflict")];
    assert!(
        SqliteOperationStore
            .reconcile(&mut connection, &batch, &records, 102)
            .is_err()
    );

    for number in [1, 2] {
        assert_eq!(row(&connection, number).0, "in_flight");
        assert!(
            SqliteOperationStore
                .operation_result(&connection, &access().context, &id(number))
                .unwrap()
                .unwrap()
                .result
                .is_none()
        );
    }

    connection
        .execute_batch("DROP TRIGGER fail_receipt;")
        .unwrap();
    SqliteOperationStore
        .reconcile(&mut connection, &batch, &records, 103)
        .unwrap();
    assert_eq!(row(&connection, 1).0, "accepted");
    assert_eq!(row(&connection, 2).0, "conflict");
}

#[test]
fn stale_receipt_cannot_replace_new_attempt_or_terminal_outcome() {
    let db = Database::new();
    let mut connection = db.open();
    save_access(&mut connection, &access());
    seed(&mut connection, 1, vec![]);
    let old = lease(&mut connection, 100).unwrap();
    let next = row(&connection, 1).3.unwrap();
    let new = lease(&mut connection, next).unwrap();
    assert!(
        SqliteOperationStore
            .reconcile(
                &mut connection,
                &old,
                &[record(1, "retryable_failure")],
                next
            )
            .is_err()
    );
    SqliteOperationStore
        .reconcile(&mut connection, &new, &[record(1, "accepted")], next)
        .unwrap();
    assert!(
        SqliteOperationStore
            .reconcile(&mut connection, &old, &[record(1, "rejected")], next)
            .is_err()
    );
    assert!(
        SqliteOperationStore
            .reconcile(&mut connection, &new, &[record(1, "conflict")], next)
            .is_err()
    );
    assert_eq!(row(&connection, 1).0, "accepted");
}

#[test]
fn repeated_retryable_receipt_finishes_each_attempt_and_keeps_backoff() {
    let db = Database::new();
    let mut connection = db.open();
    save_access(&mut connection, &access());
    seed(&mut connection, 1, vec![]);
    let batch = lease(&mut connection, 100).unwrap();
    SqliteOperationStore
        .reconcile(
            &mut connection,
            &batch,
            &[record(1, "retryable_failure")],
            101,
        )
        .unwrap();
    assert_eq!(row(&connection, 1).0, "retryable");
    let next = row(&connection, 1).3.unwrap();
    assert!(lease(&mut connection, next - 1).is_none());
    let batch = lease(&mut connection, next).unwrap();
    SqliteOperationStore
        .reconcile(
            &mut connection,
            &batch,
            &[record(1, "retryable_failure")],
            next,
        )
        .unwrap();
    assert_eq!(row(&connection, 1).0, "retryable");
    assert_eq!(row(&connection, 1).1, 2);
    assert!(row(&connection, 1).3.unwrap() > next);
}

#[test]
fn duplicate_requires_matching_original_id() {
    for original in [Value::Null, record_json(2, "accepted")] {
        let result: OperationResult = serde_json::from_value(
            json!({"operationId":id(1),"status":"duplicate","entityVersion":null,"state":null,"error":null,"originalResult":original}),
        )
        .unwrap();
        assert_eq!(
            result.original().err().unwrap().code,
            APICode::InvalidResponse
        );
    }
}

#[test]
fn lost_response_after_remote_commit_is_recovered_by_get_after_restart_without_second_effect() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        let mut connection = db.open();
        seed(&mut connection, 1, vec![]);
        let database = Arc::new(Mutex::new(connection));
        let effects = Arc::new(AtomicUsize::new(0));
        let commits = effects.clone();
        let data = database.clone();
        let server = TestApi::new(4, move |request| {
            assert!(data.try_lock().is_ok());
            if request.starts_with("POST ") {
                commits.fetch_add(1, Ordering::Relaxed);
                return (0, Value::Null);
            }
            if request.contains("/access ") {
                return (200, access_json("active", 1));
            }
            assert!(request.contains(&format!("/sync/operations/{} ", id(1).as_str())));
            (200, record_json(1, "accepted"))
        });
        let config = APIConfig::parse(&server.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
        let secrets = MemorySecretStore::default();
        let credential = credential(&config, &secrets);
        let dispatcher = OutboxDispatcher {
            http: &http,
            database: &database,
        };
        assert!(
            dispatcher
                .dispatch(
                    &credential,
                    &access().context,
                    &FixedClock(1_000),
                    &UuidV7Generator
                )
                .await
                .is_err()
        );
        let before = SqliteOperationStore
            .operation_result(&database.lock().unwrap(), &access().context, &id(1))
            .unwrap()
            .unwrap()
            .operation;
        *database.lock().unwrap() = db.open();
        assert!(matches!(
            dispatcher
                .recover(
                    &credential,
                    &access().context,
                    &id(1),
                    &FixedClock(5_000),
                    &UuidV7Generator
                )
                .await
                .unwrap(),
            RecoveryOutcome::Resolved(OperationStatus::Accepted)
        ));
        let guard = database.lock().unwrap();
        let saved = SqliteOperationStore
            .operation_result(&guard, &access().context, &id(1))
            .unwrap()
            .unwrap();
        assert_eq!(
            saved.operation.envelope().unwrap(),
            before.envelope().unwrap()
        );
        assert_eq!(
            saved.operation.semantic_fingerprint,
            before.semantic_fingerprint
        );
        assert_eq!(saved.status, OperationStatus::Accepted);
        assert_eq!(effects.load(Ordering::Relaxed), 1);
        drop(guard);
        let requests = server.finish();
        assert_eq!(
            requests
                .iter()
                .filter(|request| request.starts_with("POST "))
                .count(),
            1
        );
    });
}

#[test]
fn missing_or_expired_receipt_requires_persistent_review_and_never_mints_or_sends_an_id() {
    tauri::async_runtime::block_on(async {
        for (status, code) in [(404, "UNRECOGNIZED_CODE"), (410, "RESULT_EXPIRED")] {
            let db = Database::new();
            let mut connection = db.open();
            seed(&mut connection, 1, vec![]);
            connection
                .execute("UPDATE sync_outbox SET status='unknown',attempts=1", [])
                .unwrap();
            let server = TestApi::new(4, move |request| {
                assert!(
                    request.starts_with("GET "),
                    "se creó una escritura después de perder el recibo"
                );
                if request.contains("/access ") {
                    return (200, access_json("active", 1));
                }
                (
                    status,
                    json!({"code":code,"correlationId":OTHER,"message":"synthetic-device-token"}),
                )
            });
            let config = APIConfig::parse(&server.url).unwrap();
            let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
            let secrets = MemorySecretStore::default();
            let credential = credential(&config, &secrets);
            let database = Mutex::new(connection);
            let dispatcher = OutboxDispatcher {
                http: &http,
                database: &database,
            };
            let expected = if status == 404 {
                APICode::NotFound
            } else {
                APICode::ResultExpired
            };
            assert!(
                matches!(dispatcher.recover(&credential,&access().context,&id(1),&FixedClock(5_000),&UuidV7Generator).await.unwrap(),RecoveryOutcome::ReviewRequired(code) if code==expected)
            );
            *database.lock().unwrap() = db.open();
            assert!(matches!(
                dispatcher
                    .dispatch(
                        &credential,
                        &access().context,
                        &FixedClock(50_000),
                        &UuidV7Generator
                    )
                    .await
                    .unwrap(),
                DispatchOutcome::Idle
            ));
            assert!(matches!(
                dispatcher
                    .recover(
                        &credential,
                        &access().context,
                        &id(1),
                        &FixedClock(50_000),
                        &UuidV7Generator
                    )
                    .await
                    .unwrap(),
                RecoveryOutcome::Idle
            ));
            let guard = database.lock().unwrap();
            let saved = SqliteOperationStore
                .operation_result(&guard, &access().context, &id(1))
                .unwrap()
                .unwrap();
            assert_eq!(saved.review_code, Some(expected));
            assert_eq!(saved.operation.operation_id, FixedId(1).new_id());
            assert_eq!(saved.status, OperationStatus::Unknown);
            let count: i64 = guard
                .query_row("SELECT count(*) FROM sync_outbox", [], |row| row.get(0))
                .unwrap();
            assert_eq!(count, 1);
            drop(guard);
            server.finish();
        }
    });
}

#[test]
fn timeout_then_restart_resolves_duplicate_batch_without_repeating_remote_effect() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        let mut connection = db.open();
        for number in 1..=4 {
            seed(&mut connection, number, vec![]);
        }
        let mut posts = 0;
        let effects = Arc::new(AtomicUsize::new(0));
        let commits = effects.clone();
        let server = TestApi::new(4, move |request| {
            if request.starts_with("GET ") {
                return (200, access_json("active", 1));
            }
            posts += 1;
            if posts == 1 {
                commits.fetch_add(1, Ordering::Relaxed);
                std::thread::sleep(Duration::from_millis(800));
                return (0, Value::Null);
            }
            (
                200,
                json!({"results":(["accepted","conflict","rejected","retryable_failure"].iter().enumerate().map(|(i,status)|json!({
                "operationId":id(i as u64+1),"status":"duplicate","entityVersion":null,"state":null,"error":null,
                "originalResult":record_json(i as u64+1,status)
            })).collect::<Vec<_>>()),"correlationId":OTHER}),
            )
        });
        let config = APIConfig::parse(&server.url).unwrap();
        let http = APITransport::with_timeout(
            APIConfig::parse(&server.url).unwrap(),
            Duration::from_millis(500),
        )
        .unwrap();
        let secrets = MemorySecretStore::default();
        let credential = credential(&config, &secrets);
        let database = Mutex::new(connection);
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
                Err(DispatchFailure::Remote(error)) if error.status.is_none() && error.code==APICode::TemporaryFailure
        ));
        // Let the server finish its first request before the second access check.
        std::thread::sleep(Duration::from_millis(400));
        *database.lock().unwrap() = db.open();
        assert!(matches!(
            dispatcher
                .dispatch(
                    &credential,
                    &access().context,
                    &FixedClock(5_000),
                    &UuidV7Generator
                )
                .await
                .unwrap(),
            DispatchOutcome::Received { .. }
        ));
        for (number, status) in [
            (1, "accepted"),
            (2, "conflict"),
            (3, "rejected"),
            (4, "retryable"),
        ] {
            assert_eq!(row(&database.lock().unwrap(), number).0, status);
        }
        assert_eq!(effects.load(Ordering::Relaxed), 1);
        let requests = server.finish();
        let bodies: Vec<_> = requests
            .iter()
            .filter(|request| request.starts_with("POST "))
            .map(|request| request.split("\r\n\r\n").nth(1).unwrap())
            .collect();
        assert_eq!(bodies.len(), 2);
        assert_eq!(bodies[0], bodies[1]);
    });
}

#[test]
fn required_nullable_receipt_fields_must_exist_and_incomplete_http_reply_stays_unknown() {
    for field in ["entityVersion", "state", "error"] {
        let mut value = record_json(1, "accepted");
        value.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<OperationRecord>(value.clone()).is_err());
        assert!(serde_json::from_value::<OperationResult>(value).is_err());
    }
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        let mut connection = db.open();
        seed(&mut connection, 1, vec![]);
        let server = TestApi::new(2, |request| {
            if request.starts_with("GET ") {
                return (200, access_json("active", 1));
            }
            (
                200,
                json!({"results":[{"operationId":id(1),"status":"accepted"}],"correlationId":OTHER}),
            )
        });
        let config = APIConfig::parse(&server.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
        let secrets = MemorySecretStore::default();
        let credential = credential(&config, &secrets);
        let database = Mutex::new(connection);
        assert!(
            matches!(OutboxDispatcher{http:&http,database:&database}.dispatch(&credential,&access().context,&FixedClock(1_000),&UuidV7Generator).await,
            Err(DispatchFailure::Remote(error)) if error.code==APICode::InvalidResponse)
        );
        assert_eq!(row(&database.lock().unwrap(), 1).0, "unknown");
        server.finish();
    });
}

#[test]
fn get_recovers_all_outcomes_and_transient_failure_keeps_uncertainty() {
    tauri::async_runtime::block_on(async {
        for status in [
            "accepted",
            "conflict",
            "rejected",
            "retryable_failure",
            "unavailable",
        ] {
            let db = Database::new();
            let mut connection = db.open();
            seed(&mut connection, 1, vec![]);
            connection
                .execute("UPDATE sync_outbox SET status='unknown',attempts=1", [])
                .unwrap();
            let server = TestApi::new(2, move |request| {
                assert!(request.starts_with("GET "));
                if request.contains("/access ") {
                    return (200, access_json("active", 1));
                }
                if status == "unavailable" {
                    return (
                        503,
                        json!({"code":"SERVICE_UNAVAILABLE","correlationId":OTHER}),
                    );
                }
                (200, record_json(1, status))
            });
            let config = APIConfig::parse(&server.url).unwrap();
            let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
            let secrets = MemorySecretStore::default();
            let credential = credential(&config, &secrets);
            let database = Mutex::new(connection);
            let outcome = OutboxDispatcher {
                http: &http,
                database: &database,
            }
            .recover(
                &credential,
                &access().context,
                &id(1),
                &FixedClock(5_000),
                &UuidV7Generator,
            )
            .await;
            if status == "unavailable" {
                assert!(matches!(outcome, Err(DispatchFailure::Remote(_))));
                assert_eq!(row(&database.lock().unwrap(), 1).0, "unknown");
                assert!(row(&database.lock().unwrap(), 1).3.unwrap() > 5_000);
            } else {
                let expected = record(1, status).local_status();
                assert!(
                    matches!(outcome.unwrap(),RecoveryOutcome::Resolved(actual) if actual==expected)
                );
                assert_eq!(row(&database.lock().unwrap(), 1).0, expected.as_str());
            }
            server.finish();
        }
    });
}

#[test]
fn reviews_expose_only_scoped_metadata_and_preserve_selection_and_storage() {
    let db = Database::new();
    let mut connection = db.open();
    let scope = access().context;
    assert!(
        SqliteOperationStore
            .reviews(&connection, &scope)
            .unwrap()
            .is_empty()
    );
    for number in 1..=8 {
        seed(&mut connection, number, vec![]);
    }
    connection
        .execute(
            "UPDATE sync_outbox SET status='conflict' WHERE operation_id=?1",
            [id(1).as_str()],
        )
        .unwrap();
    connection.execute("UPDATE sync_outbox SET status='rejected',entity_type='patient_contact',entity_key=?1 WHERE operation_id=?2",
        params![json!({"patientId":id(10_002),"contactPersonId":id(20_002)}).to_string(),id(2).as_str()]).unwrap();
    connection.execute("UPDATE sync_outbox SET status='unknown',last_error='\"NOT_FOUND\"' WHERE operation_id=?1", [id(3).as_str()]).unwrap();
    for (number, field) in [(4, "account_id"), (5, "user_id"), (6, "device_id")] {
        connection
            .execute(
                &format!(
                    "UPDATE sync_outbox SET status='conflict',{field}=?1 WHERE operation_id=?2"
                ),
                params![OTHER, id(number).as_str()],
            )
            .unwrap();
    }
    connection
        .execute(
            "UPDATE sync_outbox SET permission_epoch=2 WHERE operation_id=?1",
            [id(7).as_str()],
        )
        .unwrap();
    connection
        .execute(
            "UPDATE sync_outbox SET status='unknown' WHERE operation_id=?1",
            [id(8).as_str()],
        )
        .unwrap();
    let before = connection.total_changes();
    let reviews = SqliteOperationStore.reviews(&connection, &scope).unwrap();
    assert_eq!(connection.total_changes(), before);
    assert_eq!(
        reviews
            .iter()
            .map(|r| r.operation_id.as_str())
            .collect::<Vec<_>>(),
        [
            id(7).as_str(),
            id(3).as_str(),
            id(2).as_str(),
            id(1).as_str()
        ]
    );
    let one = reviews
        .iter()
        .find(|r| r.operation_id == id(1).as_str())
        .unwrap();
    assert_eq!(one.action, "RegisterPatient");
    assert_eq!(one.entity_type, "patient_profile");
    assert_eq!(one.entity_key, json!(id(10_001)));
    assert_eq!(one.created_at_millis, 1);
    let two = reviews
        .iter()
        .find(|r| r.operation_id == id(2).as_str())
        .unwrap();
    assert_eq!(
        two.entity_key,
        json!({"patientId":id(10_002),"contactPersonId":id(20_002)})
    );
    let value = serde_json::to_value(&reviews).unwrap();
    assert_eq!(value[0]["createdAtMillis"], 7);
    assert_eq!(value[1]["code"], "NOT_FOUND");
    for row in value.as_array().unwrap() {
        assert_eq!(row.as_object().unwrap().len(), 7);
    }
    let text = value.to_string();
    for private in [
        "payload",
        "fullName",
        "result",
        "dependsOn",
        "semanticFingerprint",
    ] {
        assert!(!text.contains(private));
    }
}
