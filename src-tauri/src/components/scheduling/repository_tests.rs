use super::adapters::outbound::*;
use crate::{platform::sqlite, shared_kernel::id::OperationContext};
use rusqlite::{Connection, params};
use serde_json::json;
use std::{fs, path::PathBuf};

const RULE: &str = "019a1234-0000-7000-8000-000000000004";
const EXCEPTION: &str = "019a1234-0000-7000-8000-000000000005";
const BLOCK: &str = "019a1234-0000-7000-8000-000000000006";
const OPERATION: &str = "019a1234-0000-7000-8000-000000000007";
const PROFESSIONAL: &str = "019a1234-0000-7000-8000-000000000008";

fn scope() -> OperationContext {
    serde_json::from_value(json!({
        "accountId":"019a1234-0000-7000-8000-000000000001",
        "userId":"019a1234-0000-7000-8000-000000000002",
        "deviceId":"019a1234-0000-7000-8000-000000000003", "permissionEpoch":1
    }))
    .unwrap()
}

struct Database(PathBuf);

impl Database {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!(
            "nightingale-scheduling-{}.sqlite",
            uuid::Uuid::now_v7()
        )))
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        fs::remove_file(&self.0).unwrap();
    }
}

fn rule() -> RuleRecord {
    RuleRecord {
        rule_id: RULE.into(),
        professional_id: PROFESSIONAL.into(),
        iso_weekday: 2,
        local_start_minute: 540,
        local_end_minute: 1080,
        valid_from: "2026-01-01".into(),
        valid_until: Some("2026-12-31".into()),
        status: "active".into(),
        created_at: Some("2026-11-08T14:00:00Z".into()),
        updated_at: None,
        version: 1,
    }
}

fn exception() -> ExceptionRecord {
    ExceptionRecord {
        exception_id: EXCEPTION.into(),
        professional_id: PROFESSIONAL.into(),
        local_date: "2026-03-10".into(),
        kind: "replacement".into(),
        local_start_minute: Some(0),
        local_end_minute: Some(240),
        status: "active".into(),
        created_at: None,
        updated_at: None,
        version: 1,
    }
}

fn block() -> BlockRecord {
    BlockRecord {
        block_id: BLOCK.into(),
        professional_id: PROFESSIONAL.into(),
        block_account_id: Some("019a1234-0000-7000-8000-000000000099".into()),
        starts_at: "2026-04-01T13:00:00Z".into(),
        ends_at: "2026-04-01T17:00:00Z".into(),
        reason_code: Some("vacation".into()),
        status: "active".into(),
        created_at: None,
        updated_at: None,
        version: 1,
    }
}

fn change() -> LocalChange {
    LocalChange {
        operation_id: OPERATION.into(),
        entity_type: "schedule_rule".into(),
        rule_id: Some(RULE.into()),
        exception_id: None,
        block_id: None,
        action: "UpdateScheduleRule".into(),
        expected_version: 1,
        base_operation_id: None,
        depends_on: "[]".into(),
        payload: r#"{"localStartMinute":600}"#.into(),
        draft: r#"{"isoWeekday":2,"localStartMinute":600,"localEndMinute":1080}"#.into(),
        created_at: 1,
    }
}

fn seed(connection: &mut Connection, s: &OperationContext) {
    let tx = connection.transaction().unwrap();
    save_rule(&tx, s, &rule()).unwrap();
    save_exception(&tx, s, &exception()).unwrap();
    save_block(&tx, s, &block()).unwrap();
    insert_local_change(&tx, s, &change()).unwrap();
    tx.commit().unwrap();
}

fn assert_rows(connection: &Connection, s: &OperationContext) {
    assert_eq!(read_rules(connection, s).unwrap(), [rule()]);
    assert_eq!(read_exceptions(connection, s).unwrap(), [exception()]);
    assert_eq!(read_blocks(connection, s).unwrap(), [block()]);
    assert_eq!(read_local_changes(connection, s).unwrap(), [change()]);
}

#[test]
fn all_records_survive_reopen_without_http() {
    let database = Database::new();
    let s = scope();
    {
        let mut connection = sqlite::open(&database.0).unwrap();
        seed(&mut connection, &s);
        assert_rows(&connection, &s);
    }
    let connection = sqlite::open(&database.0).unwrap();
    assert_rows(&connection, &s);
}

#[test]
fn queries_isolate_scope_and_views_keep_intentions() {
    let database = Database::new();
    let s = scope();
    let mut connection = sqlite::open(&database.0).unwrap();
    seed(&mut connection, &s);

    // Otra cuenta con otros datos no contamina el ámbito.
    let other: OperationContext = serde_json::from_value(json!({
        "accountId":"019a1234-0000-7000-8000-0000000000aa",
        "userId":"019a1234-0000-7000-8000-000000000002",
        "deviceId":"019a1234-0000-7000-8000-000000000003", "permissionEpoch":1
    }))
    .unwrap();
    let mut other_rule = rule();
    other_rule.rule_id = "019a1234-0000-7000-8000-0000000000ab".into();
    other_rule.iso_weekday = 4;
    {
        let tx = connection.transaction().unwrap();
        save_rule(&tx, &other, &other_rule).unwrap();
        tx.commit().unwrap();
    }
    let rules = read_rules(&connection, &s).unwrap();
    assert_eq!(rules, [rule()]);
    assert_eq!(read_rules(&connection, &other).unwrap(), [other_rule]);
    assert_eq!(read_exceptions(&connection, &other).unwrap(), []);
    assert_eq!(read_blocks(&connection, &other).unwrap(), []);
    assert_eq!(read_local_changes(&connection, &other).unwrap(), []);

    // Vaciar vistas no toca intenciones (reset conserva trabajo local).
    connection
        .execute("DELETE FROM schedule_rule_view", [])
        .unwrap();
    connection
        .execute("DELETE FROM schedule_exception_view", [])
        .unwrap();
    connection
        .execute("DELETE FROM schedule_block_view", [])
        .unwrap();
    assert_eq!(read_local_changes(&connection, &s).unwrap(), [change()]);
}

#[test]
fn accepted_views_enforce_contract_constraints() {
    let database = Database::new();
    let s = scope();
    let mut connection = sqlite::open(&database.0).unwrap();

    // Regla con minutos invertidos.
    let mut bad = rule();
    bad.local_start_minute = 1200;
    bad.local_end_minute = 600;
    let err = connection
        .transaction()
        .unwrap()
        .execute(
            "INSERT INTO schedule_rule_view VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
            params![
                s.account_id.as_str(), s.user_id.as_str(), s.device_id.as_str(),
                bad.rule_id, bad.professional_id, bad.iso_weekday,
                bad.local_start_minute, bad.local_end_minute, bad.valid_from,
                bad.valid_until, bad.status, bad.created_at, bad.updated_at, bad.version
            ],
        )
        .unwrap_err();
    assert!(err.to_string().contains("CHECK"));

    // Excepción closed con minutos.
    let err = connection
        .execute(
            "INSERT INTO schedule_exception_view (account_id,user_id,device_id,exception_id,professional_id,local_date,kind,local_start_minute,local_end_minute,status,version) VALUES (?1,?2,?3,?4,?5,?6,'closed',30,60,'active',1)",
            params![
                s.account_id.as_str(), s.user_id.as_str(), s.device_id.as_str(),
                EXCEPTION, PROFESSIONAL, "2026-03-10"
            ],
        )
        .unwrap_err();
    assert!(err.to_string().contains("CHECK"));

    // Bloqueo global sin motivo y con fin anterior al inicio.
    let err = connection
        .execute(
            "INSERT INTO schedule_block_view (account_id,user_id,device_id,block_id,professional_id,block_account_id,starts_at,ends_at,reason_code,status,version) VALUES (?1,?2,?3,?4,?5,NULL,'2026-04-01T17:00:00Z','2026-04-01T13:00:00Z',NULL,'active',1)",
            params![
                s.account_id.as_str(), s.user_id.as_str(), s.device_id.as_str(),
                BLOCK, PROFESSIONAL
            ],
        )
        .unwrap_err();
    assert!(err.to_string().contains("CHECK"));

    // Intención con acción ajena a la entidad.
    let err = connection
        .execute(
            "INSERT INTO schedule_local (account_id,user_id,device_id,operation_id,entity_type,rule_id,action,expected_version,depends_on,payload,draft,created_at) VALUES (?1,?2,?3,?4,'schedule_rule',?5,'CreateScheduleBlock',0,'[]','{}','{}',1)",
            params![
                s.account_id.as_str(), s.user_id.as_str(), s.device_id.as_str(),
                OPERATION, RULE
            ],
        )
        .unwrap_err();
    assert!(err.to_string().contains("CHECK"));
}

#[test]
fn upgrade_from_seven_keeps_data_and_failed_eight_rolls_back() {
    let database = Database::new();
    // Sembrar datos de pacientes con el esquema 7.
    let connection = Connection::open(&database.0).unwrap();
    for migration in [
        include_str!("../../../migrations/0001_schema.sql"),
        include_str!("../../../migrations/0002_sync_outbox.sql"),
        include_str!("../../../migrations/0003_identity_access.sql"),
        include_str!("../../../migrations/0004_sync_control.sql"),
        include_str!("../../../migrations/0005_account.sql"),
        include_str!("../../../migrations/0006_services.sql"),
        include_str!("../../../migrations/0007_patients.sql"),
    ] {
        connection.execute_batch(migration).unwrap();
    }
    connection
        .execute(
            "INSERT INTO patient_profile_view (account_id,user_id,device_id,patient_id,completion_status,status,version) VALUES ('a','u','d','p','draft','active',1)",
            [],
        )
        .unwrap();
    drop(connection);

    let before = fs::read(&database.0).unwrap();
    let connection = sqlite::open(&database.0).unwrap();
    assert_eq!(
        connection
            .pragma_query_value::<i64, _>(None, "user_version", |r| r.get(0))
            .unwrap(),
        10
    );
    assert!(connection.table_exists(None, "schedule_rule_view").unwrap());
    assert!(
        connection
            .table_exists(None, "appointment_occupancy_view")
            .unwrap()
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT completion_status FROM patient_profile_view WHERE patient_id='p'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "draft"
    );
    drop(connection);
    let connection = sqlite::open(&database.0).unwrap();
    assert_eq!(connection.total_changes(), 0);
    drop(connection);
    assert_ne!(fs::read(&database.0).unwrap(), before);
}
