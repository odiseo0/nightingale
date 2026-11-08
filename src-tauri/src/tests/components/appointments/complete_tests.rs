//! CompleteAppointment: the injected clock must reach the start; the terminal
//! state keeps occupying.
use crate::components::appointments::{
    adapters::outbound::{SqliteAppointmentRepository, SqliteScheduleOccupancyQuery},
    application::{AppointmentMutation, AppointmentRepository, EditBase, ScheduleOccupancyQuery},
    domain::{AppointmentId, ChangeReason},
};
use crate::tests::components::appointments::test_support::{
    Database, FixedClock, Ids, PROFESSIONAL, access, database, new_appointment,
};
use jiff::Timestamp;
use rusqlite::params;
use std::str::FromStr;
use std::time::{Duration, SystemTime};

// The appointment starts at 2026-05-04T13:00:00Z.
const START_MILLIS: i64 = 1_777_899_600_000;

struct ClockAt(i64);

impl crate::shared_kernel::Clock for ClockAt {
    fn now(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_millis(self.0 as u64)
    }
}

fn appointment_id(key: &serde_json::Value) -> AppointmentId {
    key.as_str().unwrap().to_owned().try_into().unwrap()
}

fn create(
    connection: &mut rusqlite::Connection,
    ids: &Ids,
) -> crate::components::appointments::application::LocalMutationResult {
    SqliteAppointmentRepository { connection }
        .persist(
            &access(),
            AppointmentMutation::Create {
                data: Box::new(new_appointment()),
                depends_on: vec![],
            },
            ids,
            &FixedClock,
        )
        .unwrap()
}

#[test]
fn completing_before_start_creates_no_operation() {
    let mut connection = database();
    let ids = Ids::default();
    let created = create(&mut connection, &ids);
    let appointment = appointment_id(&created.entity_key);

    let err = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .complete_appointment(
        &access(),
        appointment.clone(),
        EditBase::local(created.operation_id.clone()),
        None,
        &ids,
        &ClockAt(START_MILLIS - 60_000),
    )
    .unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");

    let count: i64 = connection
        .query_row(
            "SELECT count(*) FROM sync_outbox WHERE entity_type='appointment'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
    let count: i64 = connection
        .query_row("SELECT count(*) FROM appointment_local", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn completing_at_start_reaches_terminal_and_keeps_occupancy() {
    let database = Database::new();
    let mut connection = crate::platform::sqlite::open(&database.0).unwrap();
    seed_access(&mut connection);
    let ids = Ids::default();
    let created = create(&mut connection, &ids);
    let appointment = appointment_id(&created.entity_key);

    let completed = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .complete_appointment(
        &access(),
        appointment.clone(),
        EditBase::local(created.operation_id.clone()),
        ChangeReason::new(Some("atención registrada".into())).unwrap(),
        &ids,
        &ClockAt(1_800_000_000_000),
    )
    .unwrap();
    assert_eq!(completed.provisional["status"], "completed");

    let payload: String = connection
        .query_row(
            "SELECT payload FROM sync_outbox WHERE operation_id=?1",
            params![completed.operation_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&payload).unwrap(),
        serde_json::json!({"changeReason": "atención registrada"})
    );

    // completed keeps occupying per contract.
    let occupancy = SqliteScheduleOccupancyQuery {
        connection: &connection,
        scope: &access().context,
    }
    .list_occupancy(
        PROFESSIONAL,
        Timestamp::from_str("2026-05-04T00:00:00Z").unwrap(),
        Timestamp::from_str("2026-05-05T00:00:00Z").unwrap(),
    )
    .unwrap();
    assert_eq!(occupancy.len(), 1);
    assert_eq!(occupancy[0].status, "completed");

    // Reopening keeps the terminal provisional; nothing else can transition.
    drop(connection);
    let mut connection = crate::platform::sqlite::open(&database.0).unwrap();
    let err = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .complete_appointment(
        &access(),
        appointment.clone(),
        EditBase::local(completed.operation_id.clone()),
        None,
        &ids,
        &ClockAt(START_MILLIS + 3_600_000),
    )
    .unwrap_err();
    assert_eq!(err.code, "INVALID_STATE_TRANSITION");
}

fn seed_access(connection: &mut rusqlite::Connection) {
    use crate::components::identity::repository::AccessStore as _;
    let access = access();
    let tx = connection.transaction().unwrap();
    crate::components::identity::repository::SqliteAccessStore
        .save_access(&tx, &access)
        .unwrap();
    tx.commit().unwrap();
}
