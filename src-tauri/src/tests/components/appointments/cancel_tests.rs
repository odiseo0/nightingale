//! CancelAppointment: optional reason, provisional occupancy release, work
//! preserved across reopen.
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
fn cancel_with_reason_releases_occupation_and_survives_reopen() {
    let database = Database::new();
    let mut connection = crate::platform::sqlite::open(&database.0).unwrap();
    seed_access(&mut connection);
    let ids = Ids::default();
    let created = create(&mut connection, &ids);
    let appointment = appointment_id(&created.entity_key);

    let cancelled = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .persist(
        &access(),
        AppointmentMutation::Cancel {
            appointment: appointment.clone(),
            base: EditBase::local(created.operation_id.clone()),
            reason: ChangeReason::new(Some("paciente solicitó cancelar".into())).unwrap(),
        },
        &ids,
        &FixedClock,
    )
    .unwrap();
    assert_eq!(cancelled.provisional["status"], "cancelled");

    // The reason travels in the payload; the draft keeps every snapshot.
    let payload: String = connection
        .query_row(
            "SELECT payload FROM sync_outbox WHERE operation_id=?1",
            params![cancelled.operation_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&payload).unwrap(),
        serde_json::json!({"changeReason": "paciente solicitó cancelar"})
    );
    assert_eq!(
        cancelled.provisional["serviceName"],
        created.provisional["serviceName"]
    );

    // The provisional cancellation frees the slot locally.
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
    assert!(occupancy.is_empty());

    // Reopening keeps the cancelled provisional and both operations.
    drop(connection);
    let connection = crate::platform::sqlite::open(&database.0).unwrap();
    let draft: String = connection
        .query_row(
            "SELECT draft FROM appointment_local WHERE operation_id=?1",
            params![cancelled.operation_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&draft).unwrap()["status"],
        "cancelled"
    );
}

#[test]
fn cancel_without_reason_keeps_payload_empty() {
    let mut connection = database();
    let ids = Ids::default();
    let created = create(&mut connection, &ids);
    let appointment = appointment_id(&created.entity_key);

    let cancelled = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .persist(
        &access(),
        AppointmentMutation::Cancel {
            appointment: appointment.clone(),
            base: EditBase::local(created.operation_id.clone()),
            reason: None,
        },
        &ids,
        &FixedClock,
    )
    .unwrap();

    let payload: String = connection
        .query_row(
            "SELECT payload FROM sync_outbox WHERE operation_id=?1",
            params![cancelled.operation_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(payload, "{}");
}

#[test]
fn empty_reason_is_rejected_and_cancel_is_terminal() {
    let mut connection = database();
    let ids = Ids::default();
    assert!(ChangeReason::new(Some("   ".into())).is_err());
    let created = create(&mut connection, &ids);
    let appointment = appointment_id(&created.entity_key);

    let cancelled = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .persist(
        &access(),
        AppointmentMutation::Cancel {
            appointment: appointment.clone(),
            base: EditBase::local(created.operation_id.clone()),
            reason: None,
        },
        &ids,
        &FixedClock,
    )
    .unwrap();

    let err = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .persist(
        &access(),
        AppointmentMutation::Reschedule {
            appointment: appointment.clone(),
            base: EditBase::local(cancelled.operation_id.clone()),
            interval: crate::tests::components::appointments::test_support::interval(
                "2026-05-04T14:00:00Z",
                "2026-05-04T14:30:00Z",
            ),
        },
        &ids,
        &FixedClock,
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
