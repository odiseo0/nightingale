use crate::components::sync::adapters::control::SqliteSyncControlStore;
use crate::components::sync::remote::*;
use crate::shared_kernel::{
    NightingaleError,
    id::{OperationContext, OperationId},
};
use crate::{
    components::sync::{
        adapters::outbound::SqliteOperationStore,
        application::{OperationStore, RemoteProjectionStore, record_operation},
        domain::NewOperation,
    },
    platform::sqlite::open,
    shared_kernel::{Result, SystemClock, id::UuidV7Generator},
};
use rusqlite::{Connection, Transaction};
use serde_json::Value;
use serde_json::json;
use std::{cell::Cell, path::PathBuf};

pub(super) struct Database(pub PathBuf);

impl Database {
    pub fn new() -> Self {
        Self(std::env::temp_dir().join(format!(
            "nightingale-incoming-{}.sqlite",
            uuid::Uuid::now_v7()
        )))
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0).unwrap();
    }
}

pub(super) fn scope() -> OperationContext {
    serde_json::from_value(json!({
        "accountId":"019a1234-0000-7000-8000-000000000001",
        "userId":"019a1234-0000-7000-8000-000000000002",
        "deviceId":"019a1234-0000-7000-8000-000000000003", "permissionEpoch":1
    }))
    .unwrap()
}

pub(super) fn group() -> ChangeGroup {
    serde_json::from_str(include_str!("fixtures/remote-group.json")).unwrap()
}

struct Projector {
    calls: Cell<usize>,
    fail_at: usize,
}

impl RemoteProjector for Projector {
    fn reset(&self, tx: &Transaction, _: &OperationContext) -> Result<()> {
        tx.execute("DELETE FROM test_projection", [])
            .map_err(storage_error)?;
        Ok(())
    }

    fn project(&self, tx: &Transaction, _: &OperationContext, _: &RemoteChange) -> Result<()> {
        let n = self.calls.get() + 1;
        self.calls.set(n);
        tx.execute("INSERT INTO test_projection VALUES (?1)", [n as i64])
            .map_err(storage_error)?;

        if n == self.fail_at {
            return Err(NightingaleError::internal("fallo del proyector de prueba"));
        }

        Ok(())
    }
}

fn count(c: &Connection, table: &str) -> i64 {
    c.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

#[test]
fn contract_group_round_trip_replay_and_reopen_keep_all_types_and_keys() {
    let db = Database::new();
    let mut c = open(&db.0).unwrap();
    c.execute("CREATE TABLE test_projection (n INTEGER)", [])
        .unwrap();
    let projector = Projector {
        calls: Cell::new(0),
        fail_at: usize::MAX,
    };
    let group = group();
    let json = serde_json::to_value(&group).unwrap();
    let parsed: ChangeGroup = serde_json::from_value(json.clone()).unwrap();
    assert_eq!(serde_json::to_value(parsed).unwrap(), json);
    apply_groups(&mut c, &scope(), std::slice::from_ref(&group), &projector).unwrap();
    assert_eq!(projector.calls.get(), 16);
    assert_eq!(count(&c, "sync_remote_items"), 16);
    let writes = c.total_changes();
    apply_groups(&mut c, &scope(), std::slice::from_ref(&group), &projector).unwrap();
    assert_eq!(c.total_changes(), writes);
    let mut duplicate_operation = group.clone();
    duplicate_operation.group_id = OperationId::try_from(uuid::Uuid::now_v7().to_string()).unwrap();
    apply_groups(&mut c, &scope(), &[duplicate_operation], &projector).unwrap();
    assert_eq!(projector.calls.get(), 16);
    drop(c);
    let mut c = open(&db.0).unwrap();
    apply_groups(&mut c, &scope(), std::slice::from_ref(&group), &projector).unwrap();

    for change in &group.changes {
        let stored = SqliteSyncControlStore
            .item(
                &c,
                &scope(),
                change.entity_type.as_str(),
                &change.entity_key,
            )
            .unwrap()
            .unwrap();
        assert_eq!(stored.state, change.state);
        let mut other = scope();
        other.permission_epoch += 1;
        assert!(
            SqliteSyncControlStore
                .item(&c, &other, change.entity_type.as_str(), &change.entity_key)
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn projector_failure_rolls_back_whole_multientity_group_and_inbox() {
    let db = Database::new();
    let mut c = open(&db.0).unwrap();
    c.execute("CREATE TABLE test_projection (n INTEGER)", [])
        .unwrap();
    let projector = Projector {
        calls: Cell::new(0),
        fail_at: 2,
    };
    assert!(apply_groups(&mut c, &scope(), &[group()], &projector).is_err());
    assert_eq!(count(&c, "sync_remote_items"), 0);
    assert_eq!(count(&c, "sync_inbox"), 0);
    assert_eq!(count(&c, "test_projection"), 0);
    apply_groups(&mut c, &scope(), &[group()], &()).unwrap();
    assert_eq!(count(&c, "sync_remote_items"), 16);
}

#[test]
fn equal_offer_version_applies_new_catalog_version_and_stale_version_does_not_project() {
    let db = Database::new();
    let mut c = open(&db.0).unwrap();
    let mut g = group();
    g.changes
        .retain(|x| x.entity_type == EntityType::ServiceOffer);
    g.changes[0].entity_version = 2;
    g.changes[0].state["version"] = json!(2);
    apply_groups(&mut c, &scope(), &[g.clone()], &()).unwrap();
    g.group_id = OperationId::try_from(uuid::Uuid::now_v7().to_string()).unwrap();
    g.operation_id = g.group_id.clone();
    g.changes[0].state["serviceVersion"] = json!(3);
    g.changes[0].state["name"] = json!("New catalog name");
    apply_groups(&mut c, &scope(), &[g.clone()], &()).unwrap();
    let item = SqliteSyncControlStore
        .item(&c, &scope(), "service_offer", &g.changes[0].entity_key)
        .unwrap()
        .unwrap();
    assert_eq!(item.entity_version, 2);
    assert_eq!(item.state["serviceVersion"], 3);
    g.group_id = OperationId::try_from(uuid::Uuid::now_v7().to_string()).unwrap();
    g.operation_id = g.group_id.clone();
    g.changes[0].entity_version = 1;
    g.changes[0].state["version"] = json!(1);
    g.changes[0].state["name"] = json!("Stale");
    apply_groups(&mut c, &scope(), &[g.clone()], &()).unwrap();
    assert_eq!(
        SqliteSyncControlStore
            .item(&c, &scope(), "service_offer", &g.changes[0].entity_key)
            .unwrap(),
        Some(item)
    );
}

#[test]
fn remove_retires_appointment_and_history_but_keeps_local_intention() {
    let db = Database::new();
    let mut c = open(&db.0).unwrap();
    let tx = c.transaction().unwrap();
    record_operation(
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
            payload: json!({"fullName":"Offline intention"}),
            occurred_at_local: None,
        },
    )
    .unwrap();
    tx.commit().unwrap();
    let before = SqliteOperationStore.list_pending(&c, 10).unwrap()[0]
        .envelope()
        .unwrap()
        .canonical_json()
        .unwrap();
    let mut g = group();
    apply_groups(&mut c, &scope(), &[g.clone()], &()).unwrap();
    g.group_id = OperationId::try_from(uuid::Uuid::now_v7().to_string()).unwrap();
    g.operation_id = g.group_id.clone();
    g.changes.retain(|x| {
        matches!(
            x.entity_type,
            EntityType::Appointment | EntityType::AppointmentHistory
        )
    });
    for x in &mut g.changes {
        x.change_kind = ChangeKind::Remove;
        x.state = Value::Null;
    }
    apply_groups(&mut c, &scope(), &[g.clone()], &()).unwrap();
    for x in &g.changes {
        assert!(
            SqliteSyncControlStore
                .item(&c, &scope(), x.entity_type.as_str(), &x.entity_key)
                .unwrap()
                .is_none()
        );
    }
    assert_eq!(
        SqliteOperationStore.list_pending(&c, 10).unwrap()[0]
            .envelope()
            .unwrap()
            .canonical_json()
            .unwrap(),
        before
    );
}

#[test]
fn invalid_type_key_state_scope_or_version_leaves_no_partial_group() {
    let db = Database::new();
    let mut c = open(&db.0).unwrap();
    for field in [
        "state",
        "entityKey",
        "entityVersion",
        "accountId",
        "missingNullable",
        "wrongType",
    ] {
        let mut v = serde_json::to_value(group()).unwrap();
        let x = &mut v["changes"][5];
        match field {
            "accountId" => x["state"]["accountId"] = json!("019a1234-0000-7000-8000-000000000099"),
            "missingNullable" => {
                x["state"].as_object_mut().unwrap().remove("birthDate");
            }
            "wrongType" => x["entityType"] = json!("appointment"),
            "entityVersion" => x[field] = json!(0),
            _ => x[field] = Value::Null,
        }
        let bad: ChangeGroup = serde_json::from_value(v).unwrap();
        assert!(
            apply_groups(&mut c, &scope(), &[bad], &()).is_err(),
            "{field}"
        );
        assert_eq!(count(&c, "sync_remote_items"), 0);
        assert_eq!(count(&c, "sync_inbox"), 0);
    }
    let mut v = serde_json::to_value(group()).unwrap();
    v["changes"][0]["entityType"] = json!("unrecognized");
    assert!(serde_json::from_value::<ChangeGroup>(v).is_err());
    let mut g = group();
    g.changes
        .retain(|x| x.entity_type == EntityType::PatientContact);
    g.changes[0].entity_key["extra"] = json!("019a1234-0000-7000-8000-000000000010");
    assert!(apply_groups(&mut c, &scope(), &[g], &()).is_err());
}
