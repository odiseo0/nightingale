use nightingale_lib::components::devices::{
    DeviceInstallation, InstallationStore, SqliteInstallationStore,
};
use nightingale_lib::components::identity::domain::{
    AccessRole, AccessStatus, AccountAccess, IdentityContext,
};
use nightingale_lib::components::identity::repository::{AccessStore, SqliteAccessStore};
use nightingale_lib::components::sync::adapters::control::SqliteSyncControlStore;
use nightingale_lib::components::sync::adapters::outbound::SqliteOperationStore;
use nightingale_lib::components::sync::application::{
    CursorStore, InboxStore, OperationStore, RemoteProjectionStore, record_operation,
};
use nightingale_lib::components::sync::domain::{NewOperation, RemoteItem, StoredCursor};
use nightingale_lib::platform::sqlite::open;
use nightingale_lib::shared_kernel::id::{OperationContext, OperationId, UuidV7Generator};
use nightingale_lib::shared_kernel::{ErrorCode, SystemClock};
use serde_json::json;
use std::{fs, path::PathBuf};

struct Database(PathBuf);

impl Database {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!("nightingale-core-{}.sqlite", uuid::Uuid::now_v7())))
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        if self.0.exists() {
            fs::remove_file(&self.0).unwrap();
        }
    }
}

fn scope() -> OperationContext {
    serde_json::from_value(json!({
        "accountId":"019a1234-0000-7000-8000-000000000001",
        "userId":"019a1234-0000-7000-8000-000000000002",
        "deviceId":"019a1234-0000-7000-8000-000000000003",
        "permissionEpoch":3
    }))
    .unwrap()
}

fn id(suffix: u8) -> OperationId {
    OperationId::try_from(format!("019a1234-0000-7000-8000-{suffix:012}")).unwrap()
}

fn operation() -> NewOperation {
    NewOperation {
        context: scope(),
        depends_on: vec![id(5)],
        action: "RegisterPatient".into(),
        entity_type: "patient_profile".into(),
        entity_key: json!("019a1234-0000-7000-8000-000000000004"),
        expected_version: 0,
        payload: json!({"fullName":"Synthetic"}),
        occurred_at_local: Some("2026-10-29T12:00:00-04:00".into()),
    }
}

fn item() -> RemoteItem {
    RemoteItem {
        entity_type: "patient_profile".into(),
        entity_key: operation().entity_key,
        entity_version: 2,
        state: json!({"fullName":"Accepted"}),
    }
}

fn cursor() -> StoredCursor {
    StoredCursor {
        cursor: "opaque-cursor".into(),
        window: json!({"appointmentsFrom":"2026-10-01","appointmentsUntil":"2026-11-01"}),
    }
}

#[test]
fn access_context_and_installation_survive_reopen_and_revocation() {
    let db = Database::new();
    let mut connection = open(&db.0).unwrap();
    let context = scope();
    let access = AccountAccess {
        context: context.clone(),
        role: AccessRole::Assistant,
        status: AccessStatus::Active,
        allowed_actions: vec!["RegisterPatient".into()],
        checked_at_millis: 42,
    };
    let identity = IdentityContext {
        issuer: "https://issuer.example".into(),
        subject: "synthetic-user".into(),
        user_id: context.user_id.clone(),
        selected_account_id: Some(context.account_id.clone()),
    };
    let installation = DeviceInstallation {
        device_id: context.device_id.clone(),
        display_name: "Test device".into(),
        platform: "windows".into(),
        client_version: "0.1.0".into(),
    };
    let tx = connection.transaction().unwrap();
    SqliteAccessStore.save_context(&tx, &identity).unwrap();
    SqliteAccessStore.save_access(&tx, &access).unwrap();
    SqliteInstallationStore.save(&tx, &installation).unwrap();
    tx.commit().unwrap();
    drop(connection);
    let mut connection = open(&db.0).unwrap();
    assert_eq!(
        SqliteAccessStore.context(&connection).unwrap(),
        Some(identity)
    );
    assert_eq!(
        SqliteInstallationStore
            .get(&connection, &context.device_id)
            .unwrap(),
        Some(installation)
    );
    assert_eq!(
        SqliteAccessStore
            .access(
                &connection,
                &context.account_id,
                &context.user_id,
                &context.device_id
            )
            .unwrap(),
        Some(access.clone())
    );
    let mut revoked = access;
    revoked.status = AccessStatus::Revoked;
    revoked.allowed_actions.clear();
    revoked.context.permission_epoch += 1;
    {
        let tx = connection.transaction().unwrap();
        SqliteAccessStore.save_access(&tx, &revoked).unwrap();
    }
    assert_eq!(
        SqliteAccessStore
            .access(
                &connection,
                &context.account_id,
                &context.user_id,
                &context.device_id
            )
            .unwrap()
            .unwrap()
            .status,
        AccessStatus::Active
    );
    let tx = connection.transaction().unwrap();
    SqliteAccessStore.save_access(&tx, &revoked).unwrap();
    tx.commit().unwrap();
    drop(connection);
    let connection = open(&db.0).unwrap();
    assert_eq!(
        SqliteAccessStore
            .access(
                &connection,
                &context.account_id,
                &context.user_id,
                &context.device_id
            )
            .unwrap(),
        Some(revoked)
    );
    let other = serde_json::from_value(json!("019a1234-0000-7000-8000-000000000009")).unwrap();
    assert!(
        SqliteAccessStore
            .access(&connection, &other, &context.user_id, &context.device_id)
            .unwrap()
            .is_none()
    );
}

#[test]
fn outbox_preserves_envelope_context_dependencies_and_immutable_identity() {
    let db = Database::new();
    let mut connection = open(&db.0).unwrap();
    let tx = connection.transaction().unwrap();
    let operation_id = record_operation(
        &tx,
        &SqliteOperationStore,
        &UuidV7Generator,
        &SystemClock,
        operation(),
    )
    .unwrap();
    tx.commit().unwrap();
    let pending = SqliteOperationStore
        .list_pending(&connection, 10)
        .unwrap()
        .remove(0);
    let bytes = pending.envelope().unwrap().canonical_json().unwrap();
    drop(connection);
    let mut connection = open(&db.0).unwrap();
    let reloaded = SqliteOperationStore
        .list_pending(&connection, 10)
        .unwrap()
        .remove(0);
    assert_eq!(reloaded.operation_id, operation_id);
    assert_eq!(reloaded.context, Some(scope()));
    assert_eq!(reloaded.depends_on, operation().depends_on);
    assert_eq!(
        reloaded.envelope().unwrap().canonical_json().unwrap(),
        bytes
    );
    assert_eq!(
        reloaded.semantic_fingerprint,
        Some(reloaded.envelope().unwrap().semantic_fingerprint().unwrap())
    );
    let tx = connection.transaction().unwrap();
    SqliteOperationStore.append(&tx, &reloaded).unwrap();
    let mut changed = reloaded.clone();
    changed.payload = json!({"fullName":"Changed"});
    changed.semantic_fingerprint =
        Some(changed.envelope().unwrap().semantic_fingerprint().unwrap());
    assert_eq!(
        SqliteOperationStore.append(&tx, &changed).unwrap_err().code,
        ErrorCode::IdempotencyConflict
    );
    let mut changed = reloaded;
    changed.context.as_mut().unwrap().permission_epoch += 1;
    assert_eq!(
        SqliteOperationStore.append(&tx, &changed).unwrap_err().code,
        ErrorCode::IdempotencyConflict
    );
    tx.commit().unwrap();
    assert_eq!(
        SqliteOperationStore
            .list_pending(&connection, 10)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SqliteOperationStore.list_pending(&connection, 10).unwrap()[0]
            .envelope()
            .unwrap()
            .canonical_json()
            .unwrap(),
        bytes
    );
}

#[test]
fn cursor_inbox_projection_commit_together_and_survive_replay() {
    let db = Database::new();
    let mut connection = open(&db.0).unwrap();
    let store = SqliteSyncControlStore;
    let scope = scope();
    let tx = connection.transaction().unwrap();
    assert!(store.record_group(&tx, &scope, &id(6), &id(7)).unwrap());
    store.upsert(&tx, &scope, &item()).unwrap();
    store.save_cursor(&tx, &scope, &cursor()).unwrap();
    tx.commit().unwrap();
    drop(connection);
    let mut connection = open(&db.0).unwrap();
    assert_eq!(store.cursor(&connection, &scope).unwrap(), Some(cursor()));
    assert!(
        store
            .contains_group(&connection, &scope, &id(6), &id(7))
            .unwrap()
    );
    assert_eq!(
        store
            .item(&connection, &scope, &item().entity_type, &item().entity_key)
            .unwrap(),
        Some(item())
    );
    let tx = connection.transaction().unwrap();
    assert!(!store.record_group(&tx, &scope, &id(6), &id(7)).unwrap());
    assert!(!store.record_group(&tx, &scope, &id(8), &id(7)).unwrap());
    let mut stale = item();
    stale.entity_version = 1;
    stale.state = json!({"fullName":"Old"});
    store.upsert(&tx, &scope, &stale).unwrap();
    tx.commit().unwrap();
    assert_eq!(
        store
            .item(&connection, &scope, &item().entity_type, &item().entity_key)
            .unwrap(),
        Some(item())
    );
    for different in [
        OperationContext {
            permission_epoch: 4,
            ..scope.clone()
        },
        OperationContext {
            account_id: serde_json::from_value(json!("019a1234-0000-7000-8000-000000000009"))
                .unwrap(),
            ..scope.clone()
        },
        OperationContext {
            device_id: serde_json::from_value(json!("019a1234-0000-7000-8000-000000000009"))
                .unwrap(),
            ..scope.clone()
        },
        OperationContext {
            user_id: serde_json::from_value(json!("019a1234-0000-7000-8000-000000000009")).unwrap(),
            ..scope.clone()
        },
    ] {
        assert!(store.cursor(&connection, &different).unwrap().is_none());
        assert!(
            !store
                .contains_group(&connection, &different, &id(6), &id(7))
                .unwrap()
        );
        assert!(
            store
                .item(
                    &connection,
                    &different,
                    &item().entity_type,
                    &item().entity_key
                )
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn failed_page_rolls_back_cursor_inbox_and_projection_without_changing_file() {
    let db = Database::new();
    let mut connection = open(&db.0).unwrap();
    let scope = scope();
    let store = SqliteSyncControlStore;
    let tx = connection.transaction().unwrap();
    store.upsert(&tx, &scope, &item()).unwrap();
    store.save_cursor(&tx, &scope, &cursor()).unwrap();
    tx.commit().unwrap();
    drop(connection);
    let before = fs::read(&db.0).unwrap();
    let mut connection = open(&db.0).unwrap();
    {
        let tx = connection.transaction().unwrap();
        assert!(store.record_group(&tx, &scope, &id(6), &id(7)).unwrap());
        store
            .remove(&tx, &scope, &item().entity_type, &item().entity_key)
            .unwrap();
        store
            .save_cursor(
                &tx,
                &scope,
                &StoredCursor {
                    cursor: "must-roll-back".into(),
                    ..cursor()
                },
            )
            .unwrap();
        let invalid = RemoteItem {
            entity_version: 0,
            ..item()
        };
        assert!(store.upsert(&tx, &scope, &invalid).is_err());
    }
    assert_eq!(store.cursor(&connection, &scope).unwrap(), Some(cursor()));
    assert!(
        !store
            .contains_group(&connection, &scope, &id(6), &id(7))
            .unwrap()
    );
    assert_eq!(
        store
            .item(&connection, &scope, &item().entity_type, &item().entity_key)
            .unwrap(),
        Some(item())
    );
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM schema_migrations", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        10
    );
    drop(connection);
    assert_eq!(fs::read(&db.0).unwrap(), before);
    let connection = open(&db.0).unwrap();
    assert_eq!(store.cursor(&connection, &scope).unwrap(), Some(cursor()));
}

#[test]
fn remove_keeps_local_intention_and_empty_page_can_advance_cursor() {
    let db = Database::new();
    let mut connection = open(&db.0).unwrap();
    let scope = scope();
    let store = SqliteSyncControlStore;
    let tx = connection.transaction().unwrap();
    record_operation(
        &tx,
        &SqliteOperationStore,
        &UuidV7Generator,
        &SystemClock,
        operation(),
    )
    .unwrap();
    store.upsert(&tx, &scope, &item()).unwrap();
    store.save_cursor(&tx, &scope, &cursor()).unwrap();
    tx.commit().unwrap();
    let tx = connection.transaction().unwrap();
    store
        .remove(&tx, &scope, &item().entity_type, &item().entity_key)
        .unwrap();
    store
        .save_cursor(
            &tx,
            &scope,
            &StoredCursor {
                cursor: "next-empty-page".into(),
                ..cursor()
            },
        )
        .unwrap();
    tx.commit().unwrap();
    drop(connection);
    let connection = open(&db.0).unwrap();
    assert!(
        store
            .item(&connection, &scope, &item().entity_type, &item().entity_key)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        store.cursor(&connection, &scope).unwrap().unwrap().cursor,
        "next-empty-page"
    );
    assert_eq!(
        SqliteOperationStore
            .list_pending(&connection, 10)
            .unwrap()
            .len(),
        1
    );
}
