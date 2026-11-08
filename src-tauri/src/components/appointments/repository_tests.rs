use super::adapters::outbound::SqliteScheduleOccupancyQuery;
use super::application::ScheduleOccupancyQuery;
use crate::{platform::sqlite, shared_kernel::id::OperationContext};
use jiff::Timestamp;
use rusqlite::{Connection, params};
use serde_json::json;
use std::{fs, path::PathBuf, str::FromStr};

const PROFESSIONAL: &str = "019a1234-0000-7000-8000-000000000004";
const OTHER_PROFESSIONAL: &str = "019a1234-0000-7000-8000-000000000005";

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
            "nightingale-appointments-{}.sqlite",
            uuid::Uuid::now_v7()
        )))
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        fs::remove_file(&self.0).unwrap();
    }
}

fn ts(value: &str) -> Timestamp {
    Timestamp::from_str(value).unwrap()
}

fn insert(
    connection: &Connection,
    appointment_id: &str,
    professional_id: &str,
    starts_at: &str,
    ends_at: &str,
    status: &str,
) {
    connection
        .execute(
            "INSERT INTO appointment_occupancy_view (account_id,user_id,device_id,appointment_id,professional_id,starts_at,ends_at,status,version) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,1)",
            params![
                scope().account_id.as_str(),
                scope().user_id.as_str(),
                scope().device_id.as_str(),
                appointment_id,
                professional_id,
                starts_at,
                ends_at,
                status
            ],
        )
        .unwrap();
}

fn occupancy(
    connection: &Connection,
    professional_id: &str,
    from: &str,
    until: &str,
) -> Vec<(String, String, String, String)> {
    SqliteScheduleOccupancyQuery {
        connection,
        scope: &scope(),
    }
    .list_occupancy(professional_id, ts(from), ts(until))
    .unwrap()
    .into_iter()
    .map(|interval| {
        (
            interval.appointment_id,
            interval.starts_at.to_string(),
            interval.ends_at.to_string(),
            interval.status,
        )
    })
    .collect()
}

#[test]
fn query_returns_only_occupying_states_in_window_for_professional() {
    let database = Database::new();
    let connection = sqlite::open(&database.0).unwrap();

    // Contiguas: a termina 10:00, b empieza 10:00; ambas ocupan y no se solapan.
    insert(
        &connection,
        "019a1234-0000-7000-8000-000000000010",
        PROFESSIONAL,
        "2026-05-04T13:00:00Z",
        "2026-05-04T14:00:00Z",
        "scheduled",
    );
    insert(
        &connection,
        "019a1234-0000-7000-8000-000000000011",
        PROFESSIONAL,
        "2026-05-04T14:00:00Z",
        "2026-05-04T15:00:00Z",
        "confirmed",
    );
    insert(
        &connection,
        "019a1234-0000-7000-8000-000000000012",
        PROFESSIONAL,
        "2026-05-04T15:30:00Z",
        "2026-05-04T16:00:00Z",
        "completed",
    );
    insert(
        &connection,
        "019a1234-0000-7000-8000-000000000013",
        PROFESSIONAL,
        "2026-05-04T16:30:00Z",
        "2026-05-04T17:00:00Z",
        "no_show",
    );
    // Otra profesional dentro de la ventana: no aparece.
    insert(
        &connection,
        "019a1234-0000-7000-8000-000000000014",
        OTHER_PROFESSIONAL,
        "2026-05-04T13:30:00Z",
        "2026-05-04T14:30:00Z",
        "scheduled",
    );

    let rows = occupancy(
        &connection,
        PROFESSIONAL,
        "2026-05-04T13:30:00Z",
        "2026-05-04T15:30:00Z",
    );
    assert_eq!(
        rows,
        [
            (
                "019a1234-0000-7000-8000-000000000010".to_string(),
                "2026-05-04T13:00:00Z".to_string(),
                "2026-05-04T14:00:00Z".to_string(),
                "scheduled".to_string()
            ),
            (
                "019a1234-0000-7000-8000-000000000011".to_string(),
                "2026-05-04T14:00:00Z".to_string(),
                "2026-05-04T15:00:00Z".to_string(),
                "confirmed".to_string()
            ),
        ]
    );

    // Otra cuenta no ve la ocupación.
    let other: OperationContext = serde_json::from_value(json!({
        "accountId":"019a1234-0000-7000-8000-0000000000aa",
        "userId":"019a1234-0000-7000-8000-000000000002",
        "deviceId":"019a1234-0000-7000-8000-000000000003", "permissionEpoch":1
    }))
    .unwrap();
    let rows = SqliteScheduleOccupancyQuery {
        connection: &connection,
        scope: &other,
    }
    .list_occupancy(
        PROFESSIONAL,
        ts("2026-05-04T00:00:00Z"),
        ts("2026-05-05T00:00:00Z"),
    )
    .unwrap();
    assert!(rows.is_empty());

    // Reapertura conserva la vista.
    drop(connection);
    let connection = sqlite::open(&database.0).unwrap();
    assert_eq!(
        occupancy(
            &connection,
            PROFESSIONAL,
            "2026-05-04T00:00:00Z",
            "2026-05-05T00:00:00Z"
        )
        .len(),
        4
    );
}

#[test]
fn view_rejects_cancelled_and_reversed_intervals() {
    let database = Database::new();
    let connection = sqlite::open(&database.0).unwrap();

    // cancelled libera ocupación: la vista no lo representa.
    let err = connection
        .execute(
            "INSERT INTO appointment_occupancy_view (account_id,user_id,device_id,appointment_id,professional_id,starts_at,ends_at,status,version) VALUES (?1,?2,?3,?4,?5,'2026-05-04T13:00:00Z','2026-05-04T14:00:00Z','cancelled',1)",
            params![
                scope().account_id.as_str(),
                scope().user_id.as_str(),
                scope().device_id.as_str(),
                "019a1234-0000-7000-8000-000000000010",
                PROFESSIONAL
            ],
        )
        .unwrap_err();
    assert!(err.to_string().contains("CHECK"));

    // fin anterior al inicio.
    let err = connection
        .execute(
            "INSERT INTO appointment_occupancy_view (account_id,user_id,device_id,appointment_id,professional_id,starts_at,ends_at,status,version) VALUES (?1,?2,?3,?4,?5,'2026-05-04T14:00:00Z','2026-05-04T13:00:00Z','scheduled',1)",
            params![
                scope().account_id.as_str(),
                scope().user_id.as_str(),
                scope().device_id.as_str(),
                "019a1234-0000-7000-8000-000000000011",
                PROFESSIONAL
            ],
        )
        .unwrap_err();
    assert!(err.to_string().contains("CHECK"));
}

#[test]
fn upgrade_from_eight_keeps_scheduling_data() {
    let database = Database::new();
    let connection = Connection::open(&database.0).unwrap();
    for migration in [
        include_str!("../../../migrations/0001_schema.sql"),
        include_str!("../../../migrations/0002_sync_outbox.sql"),
        include_str!("../../../migrations/0003_identity_access.sql"),
        include_str!("../../../migrations/0004_sync_control.sql"),
        include_str!("../../../migrations/0005_account.sql"),
        include_str!("../../../migrations/0006_services.sql"),
        include_str!("../../../migrations/0007_patients.sql"),
        include_str!("../../../migrations/0008_scheduling.sql"),
    ] {
        connection.execute_batch(migration).unwrap();
    }
    connection
        .execute(
            "INSERT INTO schedule_rule_view (account_id,user_id,device_id,rule_id,professional_id,iso_weekday,local_start_minute,local_end_minute,valid_from,status,version) VALUES ('a','u','d','r','p',2,540,1080,'2026-01-01','active',1)",
            [],
        )
        .unwrap();
    drop(connection);

    let connection = sqlite::open(&database.0).unwrap();
    assert_eq!(
        connection
            .pragma_query_value::<i64, _>(None, "user_version", |r| r.get(0))
            .unwrap(),
        10
    );
    assert!(
        connection
            .table_exists(None, "appointment_occupancy_view")
            .unwrap()
    );
    assert_eq!(
        connection
            .query_row(
                "SELECT local_start_minute FROM schedule_rule_view WHERE rule_id='r'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        540
    );
    drop(connection);
    let connection = sqlite::open(&database.0).unwrap();
    assert_eq!(connection.total_changes(), 0);
}

use super::adapters::outbound::{
    AppointmentHistoryRecord, AppointmentRecord, LocalChange, insert_local_change,
    read_appointments, read_history, read_local_changes, save_appointment, save_history_entry,
};
use super::domain::Snapshots;

fn snapshots() -> Snapshots {
    Snapshots::new(
        "Consulta general".into(),
        30,
        "Dra. Prueba".into(),
        "Consultorio Central".into(),
        "Av. Principal 1".into(),
        "VE".into(),
        "Distrito Capital".into(),
        "Caracas".into(),
        "America/Caracas".into(),
    )
    .unwrap()
}

fn appointment_record(id: &str, status: &str) -> AppointmentRecord {
    AppointmentRecord {
        appointment_id: id.into(),
        patient_id: "019a1234-0000-7000-8000-000000000020".into(),
        professional_id: PROFESSIONAL.into(),
        service_id: "019a1234-0000-7000-8000-000000000021".into(),
        responsible_contact_id: None,
        starts_at: "2026-05-04T13:00:00Z".into(),
        ends_at: "2026-05-04T13:30:00Z".into(),
        status: status.into(),
        confirmed_at: None,
        snapshots: snapshots(),
        created_at: Some("2026-05-01T10:00:00Z".into()),
        updated_at: None,
        version: 1,
    }
}

fn history_record(appointment_id: &str, version: i64, action: &str) -> AppointmentHistoryRecord {
    AppointmentHistoryRecord {
        appointment_id: appointment_id.into(),
        version,
        action: action.into(),
        actor_kind: "staff".into(),
        actor_user_id: Some("019a1234-0000-7000-8000-000000000002".into()),
        actor_contact_person_id: None,
        change_reason: None,
        changed_at: "2026-05-01T10:00:00Z".into(),
        starts_at: "2026-05-04T13:00:00Z".into(),
        ends_at: "2026-05-04T13:30:00Z".into(),
        status: "scheduled".into(),
        confirmed_at: None,
        snapshots: snapshots(),
    }
}

#[test]
fn appointments_round_trip_scope_isolation_and_intention_survival() {
    let database = Database::new();
    let mut connection = sqlite::open(&database.0).unwrap();
    let scope = scope();
    let other: OperationContext = serde_json::from_value(json!({
        "accountId":"019a1234-0000-7000-8000-0000000000aa",
        "userId":"019a1234-0000-7000-8000-000000000002",
        "deviceId":"019a1234-0000-7000-8000-000000000003", "permissionEpoch":1
    }))
    .unwrap();

    let appointment = appointment_record("019a1234-0000-7000-8000-000000000010", "scheduled");
    sqlite::in_transaction(&mut connection, |tx| {
        save_appointment(tx, &scope, &appointment)?;
        save_history_entry(
            tx,
            &scope,
            &history_record("019a1234-0000-7000-8000-000000000010", 1, "created"),
        )?;
        insert_local_change(
            tx,
            &scope,
            &LocalChange {
                operation_id: "019a1234-0000-7000-8000-000000000030".into(),
                appointment_id: "019a1234-0000-7000-8000-000000000010".into(),
                action: "CreateAppointment".into(),
                expected_version: 0,
                base_operation_id: None,
                depends_on: "[]".into(),
                payload: "{\"patientId\":\"x\"}".into(),
                draft: "{}".into(),
                created_at: 1,
            },
        )
    })
    .unwrap();

    assert_eq!(
        read_appointments(&connection, &scope).unwrap(),
        [appointment]
    );
    let history = read_history(&connection, &scope).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].action, "created");
    assert_eq!(read_local_changes(&connection, &scope).unwrap().len(), 1);

    // Otro ámbito no ve nada.
    assert!(read_appointments(&connection, &other).unwrap().is_empty());
    assert!(read_history(&connection, &other).unwrap().is_empty());
    assert!(read_local_changes(&connection, &other).unwrap().is_empty());

    // Vaciar vistas (reset) conserva las intenciones.
    connection
        .execute("DELETE FROM appointment_view", [])
        .unwrap();
    connection
        .execute("DELETE FROM appointment_history_view", [])
        .unwrap();
    assert_eq!(read_local_changes(&connection, &scope).unwrap().len(), 1);

    // Reapertura conserva la intención.
    drop(connection);
    let connection = sqlite::open(&database.0).unwrap();
    assert_eq!(read_local_changes(&connection, &scope).unwrap().len(), 1);
}

#[test]
fn migration_ten_check_constraints_reject_contract_violations() {
    let database = Database::new();
    let mut connection = sqlite::open(&database.0).unwrap();
    let scope = scope();

    // estado ajeno al contrato en la vista de cita
    let mut bad = appointment_record(
        "019a1234-0000-7000-8000-000000000010",
        "reschedule_requested",
    );
    let err = sqlite::in_transaction(&mut connection, |tx| save_appointment(tx, &scope, &bad))
        .unwrap_err();
    assert!(err.to_string().contains("CHECK"), "{err}");

    // país de snapshot de tres letras
    bad = appointment_record("019a1234-0000-7000-8000-000000000010", "scheduled");
    bad.snapshots.location_country = "VEN".into();
    let err = sqlite::in_transaction(&mut connection, |tx| save_appointment(tx, &scope, &bad))
        .unwrap_err();
    assert!(err.to_string().contains("CHECK"), "{err}");

    // intervalo invertido
    bad = appointment_record("019a1234-0000-7000-8000-000000000010", "scheduled");
    bad.starts_at = "2026-05-04T14:00:00Z".into();
    bad.ends_at = "2026-05-04T13:00:00Z".into();
    let err = sqlite::in_transaction(&mut connection, |tx| save_appointment(tx, &scope, &bad))
        .unwrap_err();
    assert!(err.to_string().contains("CHECK"), "{err}");

    // acción de historia inválida
    let err = sqlite::in_transaction(&mut connection, |tx| {
        save_history_entry(
            tx,
            &scope,
            &history_record("019a1234-0000-7000-8000-000000000010", 1, "deleted"),
        )
    })
    .unwrap_err();
    assert!(err.to_string().contains("CHECK"), "{err}");

    // acción de intención inválida y dependencias con más de ocho entradas
    let bad_change = LocalChange {
        operation_id: "019a1234-0000-7000-8000-000000000030".into(),
        appointment_id: "019a1234-0000-7000-8000-000000000010".into(),
        action: "DeleteAppointment".into(),
        expected_version: 0,
        base_operation_id: None,
        depends_on: "[]".into(),
        payload: "{}".into(),
        draft: "{}".into(),
        created_at: 1,
    };
    let err = sqlite::in_transaction(&mut connection, |tx| {
        insert_local_change(tx, &scope, &bad_change)
    })
    .unwrap_err();
    assert!(err.to_string().contains("CHECK"), "{err}");

    let many_deps = LocalChange {
        action: "CreateAppointment".into(),
        depends_on: "[\"a\",\"b\",\"c\",\"d\",\"e\",\"f\",\"g\",\"h\",\"i\"]".into(),
        ..bad_change.clone()
    };
    let err = sqlite::in_transaction(&mut connection, |tx| {
        insert_local_change(tx, &scope, &many_deps)
    })
    .unwrap_err();
    assert!(err.to_string().contains("CHECK"), "{err}");

    // Un guardado válido tras los rechazos.
    let ok = appointment_record("019a1234-0000-7000-8000-000000000010", "scheduled");
    sqlite::in_transaction(&mut connection, |tx| save_appointment(tx, &scope, &ok)).unwrap();
    assert_eq!(read_appointments(&connection, &scope).unwrap().len(), 1);
}

#[test]
fn migration_nine_to_ten_keeps_scheduling_and_reopens() {
    let database = Database::new();
    let connection = Connection::open(&database.0).unwrap();
    for migration in [
        include_str!("../../../migrations/0001_schema.sql"),
        include_str!("../../../migrations/0002_sync_outbox.sql"),
        include_str!("../../../migrations/0003_identity_access.sql"),
        include_str!("../../../migrations/0004_sync_control.sql"),
        include_str!("../../../migrations/0005_account.sql"),
        include_str!("../../../migrations/0006_services.sql"),
        include_str!("../../../migrations/0007_patients.sql"),
        include_str!("../../../migrations/0008_scheduling.sql"),
        include_str!("../../../migrations/0009_appointment_occupancy.sql"),
    ] {
        connection.execute_batch(migration).unwrap();
    }
    connection
        .execute(
            "INSERT INTO schedule_rule_view (account_id,user_id,device_id,rule_id,professional_id,iso_weekday,local_start_minute,local_end_minute,valid_from,status,version) VALUES ('a','u','d','r','p',2,540,1080,'2026-01-01','active',1)",
            [],
        )
        .unwrap();
    drop(connection);

    let connection = sqlite::open(&database.0).unwrap();
    assert_eq!(
        connection
            .pragma_query_value::<i64, _>(None, "user_version", |r| r.get(0))
            .unwrap(),
        10
    );
    for table in [
        "appointment_view",
        "appointment_history_view",
        "appointment_local",
        "appointment_occupancy_view",
    ] {
        assert!(connection.table_exists(None, table).unwrap(), "{table}");
    }
    assert_eq!(
        connection
            .query_row(
                "SELECT local_start_minute FROM schedule_rule_view WHERE rule_id='r'",
                [],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        540
    );
    drop(connection);
    let connection = sqlite::open(&database.0).unwrap();
    assert_eq!(connection.total_changes(), 0);
}
