use crate::components::scheduling::adapters::outbound::{
    read_blocks, read_exceptions, read_local_changes, read_rules,
};
use crate::components::scheduling::adapters::projector::*;
use crate::components::sync::remote::{ChangeGroup, ChangeKind, SyncChange, apply_groups};
use crate::components::sync::remote::{EntityType, RemoteProjector};
use crate::shared_kernel::id::OperationContext;

const PROFESSIONAL: &str = "019a1234-0000-7000-8000-000000000008";
const RULE: &str = "019a1234-0000-7000-8000-000000000010";
const EXCEPTION: &str = "019a1234-0000-7000-8000-000000000011";
const BLOCK: &str = "019a1234-0000-7000-8000-000000000012";

fn scope() -> OperationContext {
    serde_json::from_value(serde_json::json!({
        "accountId":"019a1234-0000-7000-8000-000000000001",
        "userId":"019a1234-0000-7000-8000-000000000002",
        "deviceId":"019a1234-0000-7000-8000-000000000003", "permissionEpoch":1
    }))
    .unwrap()
}

fn database() -> rusqlite::Connection {
    crate::platform::sqlite::open(std::path::Path::new(":memory:")).unwrap()
}

fn rule(s: &OperationContext, version: i64) -> SyncChange {
    SyncChange {
        entity_type: EntityType::ScheduleRule,
        entity_key: serde_json::json!(RULE),
        entity_version: version,
        change_kind: ChangeKind::Upsert,
        state: serde_json::json!({
            "accountId":s.account_id,"id":RULE,"professionalId":PROFESSIONAL,
            "isoWeekday":3,"localStartMinute":480,"localEndMinute":720,
            "validFrom":"2026-01-01","validUntil":null,
            "status":"active","createdAt":null,"updatedAt":null,"version":version
        }),
    }
}

fn exception(s: &OperationContext, version: i64) -> SyncChange {
    SyncChange {
        entity_type: EntityType::ScheduleException,
        entity_key: serde_json::json!(EXCEPTION),
        entity_version: version,
        change_kind: ChangeKind::Upsert,
        state: serde_json::json!({
            "accountId":s.account_id,"id":EXCEPTION,"professionalId":PROFESSIONAL,
            "localDate":"2026-05-06","kind":"replacement",
            "localStartMinute":540,"localEndMinute":600,
            "status":"active","createdAt":null,"updatedAt":null,"version":version
        }),
    }
}

fn block(_s: &OperationContext, version: i64) -> SyncChange {
    SyncChange {
        entity_type: EntityType::ScheduleBlock,
        entity_key: serde_json::json!(BLOCK),
        entity_version: version,
        change_kind: ChangeKind::Upsert,
        state: serde_json::json!({
            "accountId":null,"id":BLOCK,"professionalId":PROFESSIONAL,
            "startsAt":"2026-04-01T13:00:00Z","endsAt":"2026-04-01T17:00:00Z",
            "reasonCode":null,
            "status":"active","createdAt":null,"updatedAt":null,"version":version
        }),
    }
}

fn group(changes: Vec<SyncChange>) -> ChangeGroup {
    ChangeGroup {
        group_id: uuid::Uuid::now_v7().to_string().try_into().unwrap(),
        operation_id: uuid::Uuid::now_v7().to_string().try_into().unwrap(),
        accepted_at: "2026-11-08T12:00:00Z".into(),
        changes,
    }
}

fn full_group(s: &OperationContext, version: i64) -> ChangeGroup {
    group(vec![
        rule(s, version),
        exception(s, version),
        block(s, version),
    ])
}

#[test]
fn repeated_groups_and_old_versions_do_not_regress_views() {
    let mut c = database();
    let s = scope();
    apply_groups(&mut c, &s, &[full_group(&s, 2)], &SchedulingProjector).unwrap();
    apply_groups(&mut c, &s, &[full_group(&s, 2)], &SchedulingProjector).unwrap();
    apply_groups(&mut c, &s, &[full_group(&s, 1)], &SchedulingProjector).unwrap();

    assert_eq!(read_rules(&c, &s).unwrap()[0].version, 2);
    assert_eq!(read_exceptions(&c, &s).unwrap()[0].version, 2);
    assert_eq!(read_blocks(&c, &s).unwrap()[0].version, 2);
    // Global block keeps its null scope and hidden reason by audience.
    assert_eq!(read_blocks(&c, &s).unwrap()[0].block_account_id, None);
    assert_eq!(read_blocks(&c, &s).unwrap()[0].reason_code, None);
}

#[test]
fn remove_and_reset_preserve_intentions_and_other_scopes() {
    let mut c = database();
    let s = scope();
    apply_groups(&mut c, &s, &[full_group(&s, 1)], &SchedulingProjector).unwrap();

    // Una intención local sobre la vista aceptada sobrevive a remove/reset.
    {
        let tx = c.transaction().unwrap();
        crate::components::scheduling::adapters::outbound::insert_local_change(
            &tx,
            &s,
            &crate::components::scheduling::adapters::outbound::LocalChange {
                operation_id: "019a1234-0000-7000-8000-000000000020".into(),
                entity_type: "schedule_rule".into(),
                rule_id: Some(RULE.into()),
                exception_id: None,
                block_id: None,
                action: "UpdateScheduleRule".into(),
                expected_version: 1,
                base_operation_id: None,
                depends_on: "[]".into(),
                payload: "{}".into(),
                draft: "{}".into(),
                created_at: 1,
            },
        )
        .unwrap();
        tx.commit().unwrap();
    }

    let mut other = scope();
    other.account_id = "019a1234-0000-7000-8000-000000000099"
        .to_owned()
        .try_into()
        .unwrap();
    apply_groups(
        &mut c,
        &other,
        &[full_group(&other, 1)],
        &SchedulingProjector,
    )
    .unwrap();

    // remove retira solo la vista de su ámbito.
    let mut removed = full_group(&s, 2);
    for change in &mut removed.changes {
        change.change_kind = ChangeKind::Remove;
        change.entity_version = 2;
        change.state = serde_json::Value::Null;
    }
    apply_groups(&mut c, &s, &[removed], &SchedulingProjector).unwrap();
    assert!(read_rules(&c, &s).unwrap().is_empty());
    assert!(read_exceptions(&c, &s).unwrap().is_empty());
    assert!(read_blocks(&c, &s).unwrap().is_empty());
    assert_eq!(read_rules(&c, &other).unwrap().len(), 1);
    assert_eq!(read_local_changes(&c, &s).unwrap().len(), 1);

    // reset reconstruye una captura: vacía vistas, conserva intenciones.
    apply_groups(
        &mut c,
        &other,
        &[full_group(&other, 1)],
        &SchedulingProjector,
    )
    .unwrap();
    SchedulingProjector
        .reset(&c.transaction().unwrap(), &s)
        .unwrap();
    assert!(read_rules(&c, &s).unwrap().is_empty());
    assert_eq!(read_rules(&c, &other).unwrap().len(), 1);
    assert_eq!(read_local_changes(&c, &s).unwrap().len(), 1);
}

#[test]
fn out_of_scope_or_invalid_changes_are_rejected_and_roll_back() {
    let mut c = database();
    let s = scope();
    apply_groups(&mut c, &s, &[full_group(&s, 1)], &SchedulingProjector).unwrap();

    // Un cambio de la cuenta A aplicado en el ámbito de la cuenta B se revierte.
    let mut foreign = scope();
    foreign.account_id = "019a1234-0000-7000-8000-0000000000ee"
        .to_owned()
        .try_into()
        .unwrap();
    let err =
        apply_groups(&mut c, &foreign, &[full_group(&s, 1)], &SchedulingProjector).unwrap_err();
    assert_eq!(err.code, crate::shared_kernel::ErrorCode::ValidationError);
    assert_eq!(read_rules(&c, &s).unwrap()[0].version, 1);

    // Minutos invertidos: rechazada sin efectos.
    let mut invalid = rule(&s, 2);
    invalid.state["localStartMinute"] = serde_json::json!(720);
    invalid.state["localEndMinute"] = serde_json::json!(480);
    let err = apply_groups(&mut c, &s, &[group(vec![invalid])], &SchedulingProjector).unwrap_err();
    assert!(err.message.contains("regla"));
    assert_eq!(read_rules(&c, &s).unwrap()[0].version, 1);
}
