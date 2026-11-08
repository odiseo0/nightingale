//! MarkAppointmentNoShow: staff action gated by the injected clock and the
//! accepted account grace window; never automatic.
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

// 2026-05-04T13:00:00Z, the appointment start used by test_support fixtures.
const START_MILLIS: i64 = 1_777_899_600_000;
const GRACE_MINUTES: i64 = 15;

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

fn seed_account(connection: &rusqlite::Connection, grace: i64) {
    let scope = access().context;
    connection
        .execute(
            "INSERT INTO account_view VALUES (?1,?2,?3,'Consultorio Sintético','America/Caracas','Consultorio Central','Av. Principal 1','Caracas','Distrito Capital','VE',?4,'active',1,NULL)",
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str(),
                grace
            ],
        )
        .unwrap();
}

#[test]
fn no_show_requires_start_plus_accepted_grace() {
    let mut connection = database();
    seed_account(&connection, GRACE_MINUTES);
    let ids = Ids::default();
    let created = create(&mut connection, &ids);
    let appointment = appointment_id(&created.entity_key);

    // Before the start.
    let err = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .mark_no_show_appointment(
        &access(),
        appointment.clone(),
        EditBase::local(created.operation_id.clone()),
        None,
        &ids,
        &ClockAt(START_MILLIS - 60_000),
    )
    .unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");

    // After the start but inside the grace window.
    let err = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .mark_no_show_appointment(
        &access(),
        appointment.clone(),
        EditBase::local(created.operation_id.clone()),
        None,
        &ids,
        &ClockAt(START_MILLIS + GRACE_MINUTES * 60_000 - 60_000),
    )
    .unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");

    // No operation was created by the failed attempts.
    let count: i64 = connection
        .query_row(
            "SELECT count(*) FROM sync_outbox WHERE entity_type='appointment'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn no_show_at_grace_keeps_occupation_and_survives_reopen() {
    let database = Database::new();
    let mut connection = crate::platform::sqlite::open(&database.0).unwrap();
    seed_access(&mut connection);
    seed_account(&connection, GRACE_MINUTES);
    let ids = Ids::default();
    let created = create(&mut connection, &ids);
    let appointment = appointment_id(&created.entity_key);

    let no_show = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .mark_no_show_appointment(
        &access(),
        appointment.clone(),
        EditBase::local(created.operation_id.clone()),
        ChangeReason::new(Some("no asistió".into())).unwrap(),
        &ids,
        &ClockAt(1_800_000_000_000),
    )
    .unwrap();
    assert_eq!(no_show.provisional["status"], "no_show");

    // no_show keeps occupying per contract.
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
    assert_eq!(occupancy[0].status, "no_show");

    // Reopening keeps the terminal provisional.
    drop(connection);
    let connection = crate::platform::sqlite::open(&database.0).unwrap();
    let draft: String = connection
        .query_row(
            "SELECT draft FROM appointment_local WHERE operation_id=?1",
            params![no_show.operation_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&draft).unwrap()["status"],
        "no_show"
    );
}

#[test]
fn grace_change_uses_the_accepted_account_policy() {
    // A policy with zero grace allows marking exactly at the start.
    let mut connection = database();
    seed_account(&connection, 0);
    let ids = Ids::default();
    let created = create(&mut connection, &ids);
    let appointment = appointment_id(&created.entity_key);

    SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .mark_no_show_appointment(
        &access(),
        appointment.clone(),
        EditBase::local(created.operation_id.clone()),
        None,
        &ids,
        &ClockAt(1_800_000_000_000),
    )
    .unwrap();
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
