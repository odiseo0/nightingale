use super::{
    adapters::outbound::SqliteSchedulingRepository,
    application::{EditBase, LocalMutationResult, SchedulingMutation, SchedulingRepository},
    domain::{RulePatch, RuleValue, Status},
    test_support::{Ids, NoOccupancy, access, database, repository, seed_account, FixedClock},
};
use crate::platform::sqlite;
use serde_json::json;


fn rule_value() -> RuleValue {
    RuleValue::new(
        "019a1234-0000-7000-8000-000000000008".to_owned().try_into().unwrap(),
        2,
        540,
        1080,
        super::domain::parse_local_date("2026-01-01").unwrap(),
        Some(super::domain::parse_local_date("2026-12-31").unwrap()),
    )
    .unwrap()
}

fn create(
    repo: &mut SqliteSchedulingRepository<'_>,
    ids: &Ids,
) -> LocalMutationResult {
    repo.persist(&access(), SchedulingMutation::CreateRule { value: rule_value() }, ids, &FixedClock, &NoOccupancy)
        .unwrap()
}

#[test]
fn create_persists_provisional_and_outbox_together_and_survives_reopen() {
    let dir = std::env::temp_dir().join(format!(
        "nightingale-scheduling-t05-{}.sqlite",
        uuid::Uuid::now_v7()
    ));
    let first = {
        let mut c = sqlite::open(&dir).unwrap();
        seed_account(&mut c);
        let mut repo = repository(&mut c);
        let ids = Ids::default();

        let result = create(&mut repo, &ids);
        assert_eq!(result.entity_type, "schedule_rule");
        assert_eq!(result.expected_version, 0);
        assert!(result.depends_on.is_empty());
        assert_eq!(result.provisional["localStartMinute"], json!(540));

        let outbox: i64 = c
            .query_row("SELECT count(*) FROM sync_outbox", [], |r| r.get(0))
            .unwrap();
        let local: i64 = c
            .query_row("SELECT count(*) FROM schedule_local", [], |r| r.get(0))
            .unwrap();
        assert_eq!((outbox, local), (1, 1));
        result
    };

    // Reapertura: intención, provisional y operación siguen presentes.
    let mut c = sqlite::open(&dir).unwrap();
    let mut repo = repository(&mut c);
    let row = repo
        .rule_row(&access(), first.entity_key.as_str().unwrap())
        .unwrap();
    let provisional = row.provisional.unwrap();
    assert_eq!(provisional.operation_id, first.operation_id);
    assert_eq!(provisional.expected_version, 0);
    assert_eq!(provisional.state.local_start_minute, 540);
    assert_eq!(row.operations.len(), 1);
    assert_eq!(row.operations[0].status, "pending");
    drop(c);
    std::fs::remove_file(&dir).unwrap();
}

#[test]
fn successive_edits_survive_reopen_with_ids_bases_and_dependencies() {
    let dir = std::env::temp_dir().join(format!(
        "nightingale-scheduling-t05-chain-{}.sqlite",
        uuid::Uuid::now_v7()
    ));
    let (rule_id, first_op, second_op) = {
        let mut c = sqlite::open(&dir).unwrap();
        seed_account(&mut c);
        let mut repo = repository(&mut c);
        let ids = Ids::default();

        let created = create(&mut repo, &ids);
        let rule_id = created.entity_key.as_str().unwrap().to_owned();
        let edit1 = repo
            .persist(
                &access(),
                SchedulingMutation::UpdateRule {
                    rule_id: rule_id.clone(),
                    base: EditBase::LocalOperation {
                        operation_id: crate::shared_kernel::id::OperationId::try_from(created.operation_id.clone()).unwrap(),
                    },
                    patch: RulePatch::new(None, Some(600), None, None, None).unwrap(),
                },
                &ids,
                &FixedClock,
                &NoOccupancy,
            )
            .unwrap();
        let edit2 = repo
            .persist(
                &access(),
                SchedulingMutation::UpdateRule {
                    rule_id: rule_id.clone(),
                    base: EditBase::LocalOperation {
                        operation_id: crate::shared_kernel::id::OperationId::try_from(edit1.operation_id.clone()).unwrap(),
                    },
                    patch: RulePatch::new(Some(3), None, None, None, None).unwrap(),
                },
                &ids,
                &FixedClock,
                &NoOccupancy,
            )
            .unwrap();
        assert_eq!(edit2.expected_version, 2);
        (rule_id, created.operation_id, edit1.operation_id)
    };

    let mut c = sqlite::open(&dir).unwrap();
    let mut repo = repository(&mut c);
    let row = repo.rule_row(&access(), &rule_id).unwrap();
    assert_eq!(row.operations.len(), 3);
    assert_eq!(row.provisional.unwrap().state.iso_weekday, 3);
    let deps: Vec<String> = c
        .prepare("SELECT depends_on FROM schedule_local ORDER BY expected_version")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    assert_eq!(deps[0], "[]");
    assert_eq!(deps[1], json!([first_op]).to_string());
    assert_eq!(deps[2], json!([second_op]).to_string());
    drop(c);
    std::fs::remove_file(&dir).unwrap();
}

#[test]
fn stale_bases_blocked_dependencies_and_no_changes_are_rejected() {
    let mut c = database();
    let mut repo = repository(&mut c);
    let ids = Ids::default();

    let created = create(&mut repo, &ids);
    let rule_id = created.entity_key.as_str().unwrap().to_owned();
    drop(repo);

    // Base aceptada con provisional pendiente: rechazada.
    let mut repo = repository(&mut c);
    let stale = repo.persist(
        &access(),
        SchedulingMutation::SetRuleStatus {
            rule_id: rule_id.clone(),
            base: EditBase::Accepted { version: 1 },
            status: Status::Inactive,
        },
        &ids,
        &FixedClock,
        &NoOccupancy,
    );
    assert_eq!(stale.unwrap_err().code, "VERSION_CONFLICT");

    // Operación local desconocida como base: rechazada.
    let unknown = repo.persist(
        &access(),
        SchedulingMutation::SetRuleStatus {
            rule_id: rule_id.clone(),
            base: EditBase::LocalOperation {
                operation_id: crate::shared_kernel::id::OperationId::try_from("019a1234-0000-7000-8000-000000000777".to_owned()).unwrap(),
            },
            status: Status::Inactive,
        },
        &ids,
        &FixedClock,
        &NoOccupancy,
    );
    assert_eq!(unknown.unwrap_err().code, "VERSION_CONFLICT");

    // Sin diferencia: NO_CHANGES sin crear operación.
    let same = repo.persist(
        &access(),
        SchedulingMutation::SetRuleStatus {
            rule_id: rule_id.clone(),
            base: EditBase::LocalOperation {
                operation_id: crate::shared_kernel::id::OperationId::try_from(created.operation_id.clone()).unwrap(),
            },
            status: Status::Active,
        },
        &ids,
        &FixedClock,
        &NoOccupancy,
    );
    assert_eq!(same.unwrap_err().code, "NO_CHANGES");
    drop(repo);
    let count: i64 = c
        .query_row("SELECT count(*) FROM sync_outbox", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);

    // Operación previa en conflicto: DEPENDENCY_FAILED.
    c.execute(
        "UPDATE sync_outbox SET status = 'conflict' WHERE operation_id = ?1",
        [&created.operation_id],
    )
    .unwrap();
    let mut repo = repository(&mut c);
    let blocked = repo.persist(
        &access(),
        SchedulingMutation::SetRuleStatus {
            rule_id,
            base: EditBase::LocalOperation {
                operation_id: crate::shared_kernel::id::OperationId::try_from(created.operation_id.clone()).unwrap(),
            },
            status: Status::Inactive,
        },
        &ids,
        &FixedClock,
        &NoOccupancy,
    );
    assert_eq!(blocked.unwrap_err().code, "DEPENDENCY_FAILED");
}

#[test]
fn write_without_permission_is_denied_without_effects() {
    let mut c = database();
    let mut repo = repository(&mut c);
    let ids = Ids::default();

    let mut denied = access();
    denied.allowed_actions.clear();
    let result = repo.persist(
        &denied,
        SchedulingMutation::CreateRule {
            value: rule_value(),
        },
        &ids,
        &FixedClock,
        &NoOccupancy,
    );
    assert_eq!(result.unwrap_err().code, "PERMISSION_DENIED");
    let count: i64 = c
        .query_row("SELECT count(*) FROM sync_outbox", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
}
