use crate::components::scheduling::{
    adapters::outbound::read_local_changes,
    application::{EditBase, SchedulingMutation, SchedulingRepository},
};
use crate::components::sync::remote::{ChangeGroup, ChangeKind, SyncChange, apply_groups};
use crate::tests::components::scheduling::test_support::{
    FixedClock, Ids, NoOccupancy, access, database, repository,
};
use rusqlite::params;

fn op(id: &str) -> crate::shared_kernel::id::OperationId {
    crate::shared_kernel::id::OperationId::try_from(id.to_owned()).unwrap()
}

fn create_rule(c: &mut rusqlite::Connection, ids: &Ids) -> (String, String) {
    let created = repository(c)
        .persist(
            &access(),
            SchedulingMutation::CreateRule {
                value: crate::components::scheduling::domain::RuleValue::new(
                    "019a1234-0000-7000-8000-000000000008"
                        .to_owned()
                        .try_into()
                        .unwrap(),
                    3,
                    480,
                    720,
                    crate::components::scheduling::domain::parse_local_date("2026-01-01").unwrap(),
                    None,
                )
                .unwrap(),
            },
            ids,
            &FixedClock,
            &NoOccupancy,
        )
        .unwrap();
    (
        created.entity_key.as_str().unwrap().to_owned(),
        created.operation_id,
    )
}

fn accepted_rule_group(
    s: &crate::shared_kernel::id::OperationContext,
    rule_id: &str,
    version: i64,
    start: i64,
) -> ChangeGroup {
    ChangeGroup {
        group_id: uuid::Uuid::now_v7().to_string().try_into().unwrap(),
        operation_id: uuid::Uuid::now_v7().to_string().try_into().unwrap(),
        accepted_at: "2026-11-08T12:00:00Z".into(),
        changes: vec![SyncChange {
            entity_type: crate::components::sync::remote::EntityType::ScheduleRule,
            entity_key: serde_json::json!(rule_id),
            entity_version: version,
            change_kind: ChangeKind::Upsert,
            state: serde_json::json!({
                "accountId":s.account_id,"id":rule_id,
                "professionalId":"019a1234-0000-7000-8000-000000000008",
                "isoWeekday":3,"localStartMinute":start,"localEndMinute":720,
                "validFrom":"2026-01-01","validUntil":null,
                "status":"active","createdAt":null,"updatedAt":null,"version":version
            }),
        }],
    }
}

fn mark_accepted(c: &rusqlite::Connection, operation: &str, entity_version: i64) {
    c.execute(
        "UPDATE sync_outbox SET status = 'accepted', result = json(?1) WHERE operation_id = ?2",
        params![
            serde_json::json!({"entityVersion": entity_version, "state": null, "error": null})
                .to_string(),
            operation
        ],
    )
    .unwrap();
}

#[test]
fn accepted_receipt_without_download_keeps_the_provisional() {
    let mut c = database();
    let (rule_id, operation) = create_rule(&mut c, &Ids::default());
    mark_accepted(&c, &operation, 1);

    let rows = repository(&mut c).rule_rows_list(&access()).unwrap();
    let row = rows
        .iter()
        .find(|row| row.visible().unwrap().rule_id == rule_id)
        .unwrap();
    assert!(
        row.provisional.is_some(),
        "recibo aceptado sin descarga no retira el provisional"
    );
    assert_eq!(row.operations[0].status, "accepted");
}

#[test]
fn accepted_receipt_with_view_at_confirmed_version_retires_provisional() {
    let mut c = database();
    let (rule_id, operation) = create_rule(&mut c, &Ids::default());
    mark_accepted(&c, &operation, 1);

    // La descarga proyecta la regla aceptada en la versión confirmada (1).
    let scope = access().context;
    apply_groups(
        &mut c,
        &scope,
        &[accepted_rule_group(&scope, &rule_id, 1, 600)],
        &crate::components::scheduling::adapters::projector::SchedulingProjector,
    )
    .unwrap();

    let rows = repository(&mut c).rule_rows_list(&access()).unwrap();
    let row = rows
        .iter()
        .find(|row| row.accepted.as_ref().unwrap().state.rule_id == rule_id)
        .unwrap();
    assert!(
        row.provisional.is_none(),
        "vista en versión confirmada retira el provisional"
    );
    assert_eq!(row.accepted.as_ref().unwrap().version, 1);
    assert_eq!(row.accepted.as_ref().unwrap().state.local_start_minute, 600);
    // Las operaciones quedan para revisión histórica.
    assert_eq!(row.operations.len(), 1);
}

#[test]
fn conflict_rejected_and_unknown_results_keep_the_provisional() {
    for status in ["conflict", "rejected", "unknown"] {
        let mut c = database();
        let (rule_id, operation) = create_rule(&mut c, &Ids::default());
        c.execute(
            "UPDATE sync_outbox SET status = ?1 WHERE operation_id = ?2",
            params![status, operation],
        )
        .unwrap();

        let scope = access().context;
        apply_groups(
            &mut c,
            &scope,
            &[accepted_rule_group(&scope, &rule_id, 1, 600)],
            &crate::components::scheduling::adapters::projector::SchedulingProjector,
        )
        .unwrap();

        let rows = repository(&mut c).rule_rows_list(&access()).unwrap();
        let row = rows
            .iter()
            .find(|row| row.visible().unwrap().rule_id == rule_id)
            .unwrap();
        assert!(
            row.provisional.is_some(),
            "{status} conserva el provisional aunque la vista descargue el cambio"
        );
    }
}

#[test]
fn accepted_partial_receipt_keeps_sequence_ids_and_dependencies() {
    let mut c = database();
    let ids = Ids::default();
    let (rule_id, first) = create_rule(&mut c, &ids);
    let second = repository(&mut c)
        .persist(
            &access(),
            SchedulingMutation::UpdateRule {
                rule_id: rule_id.clone(),
                base: EditBase::LocalOperation {
                    operation_id: op(&first),
                },
                patch: crate::components::scheduling::domain::RulePatch::new(
                    None,
                    Some(660),
                    None,
                    None,
                    None,
                )
                .unwrap(),
            },
            &ids,
            &FixedClock,
            &NoOccupancy,
        )
        .unwrap()
        .operation_id;

    // Solo la primera operación se acepta y descarga; la segunda sigpe local.
    mark_accepted(&c, &first, 1);
    let scope = access().context;
    apply_groups(
        &mut c,
        &scope,
        &[accepted_rule_group(&scope, &rule_id, 1, 600)],
        &crate::components::scheduling::adapters::projector::SchedulingProjector,
    )
    .unwrap();

    let rows = repository(&mut c).rule_rows_list(&access()).unwrap();
    let row = rows
        .iter()
        .find(|row| row.visible().unwrap().rule_id == rule_id)
        .unwrap();
    // El provisional vigente es la segunda operación (sin recibo), no se retira.
    let provisional = row.provisional.as_ref().unwrap();
    assert_eq!(provisional.operation_id, second);
    assert_eq!(row.operations.len(), 2);

    // Las dependencias originales sobreviven intactas tras la reapertura.
    drop(rows);
    let deps: Vec<String> = read_local_changes(&c, &access().context)
        .unwrap()
        .into_iter()
        .map(|change| change.depends_on)
        .collect();
    assert_eq!(deps[0], "[]");
    assert_eq!(deps[1], serde_json::json!([first]).to_string());
}

#[test]
fn local_intentions_survive_reconciliation_and_reopen() {
    let dir = std::env::temp_dir().join(format!(
        "nightingale-scheduling-t13-{}.sqlite",
        uuid::Uuid::now_v7()
    ));
    let operation = {
        let mut c = crate::platform::sqlite::open(&dir).unwrap();
        crate::tests::components::scheduling::test_support::seed_account(&mut c);
        let (rule_id, operation) = create_rule(&mut c, &Ids::default());
        mark_accepted(&c, &operation, 1);
        let scope = access().context;
        apply_groups(
            &mut c,
            &scope,
            &[accepted_rule_group(&scope, &rule_id, 1, 600)],
            &crate::components::scheduling::adapters::projector::SchedulingProjector,
        )
        .unwrap();
        operation
    };

    let c = crate::platform::sqlite::open(&dir).unwrap();
    let changes = read_local_changes(&c, &access().context).unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].operation_id, operation);
    assert_eq!(changes[0].action, "CreateScheduleRule");
    drop(c);
    std::fs::remove_file(&dir).unwrap();
}
