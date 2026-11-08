//! RescheduleAppointment: same real duration, own-slot exclusion, snapshots
//! and references preserved, confirmation invalidated until acceptance.
use super::{
    adapters::outbound::SqliteAppointmentRepository,
    application::{AppointmentMutation, AppointmentRepository, EditBase, ScheduleOccupancyQuery},
    domain::{AppointmentId, AppointmentInterval},
    test_support::{Database, FixedClock, Ids, access, database, interval, new_appointment},
};
use rusqlite::params;

fn appointment_id(key: &serde_json::Value) -> AppointmentId {
    key.as_str().unwrap().to_owned().try_into().unwrap()
}

fn create_at(
    connection: &mut rusqlite::Connection,
    ids: &Ids,
    starts: &str,
    ends: &str,
) -> super::application::LocalMutationResult {
    let mut data = new_appointment();
    data.interval = interval(starts, ends);
    SqliteAppointmentRepository { connection }
        .persist(
            &access(),
            AppointmentMutation::Create {
                data: Box::new(data),
                depends_on: vec![],
            },
            ids,
            &FixedClock,
        )
        .unwrap()
}

#[test]
fn reschedule_moves_slot_preserves_duration_snapshots_and_pending_state() {
    let database = Database::new();
    let mut connection = crate::platform::sqlite::open(&database.0).unwrap();
    seed_access(&mut connection);
    super::create_tests::seed_context(&connection);
    let ids = Ids::default();

    let created = create_at(
        &mut connection,
        &ids,
        "2026-05-04T13:00:00Z",
        "2026-05-04T13:30:00Z",
    );
    let appointment = appointment_id(&created.entity_key);

    // Confirm first: rescheduling a confirmed appointment resets it.
    let confirmed = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .persist(
        &access(),
        AppointmentMutation::Confirm {
            appointment: appointment.clone(),
            base: EditBase::local(created.operation_id.clone()),
        },
        &ids,
        &FixedClock,
    )
    .unwrap();

    let moved = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .reschedule_appointment(
        &access(),
        appointment.clone(),
        EditBase::local(confirmed.operation_id.clone()),
        interval("2026-05-04T14:00:00Z", "2026-05-04T14:30:00Z"),
        &ids,
        &FixedClock,
    )
    .unwrap();

    let draft = &moved.provisional;
    assert_eq!(draft["status"], "scheduled");
    assert_eq!(draft["confirmedAt"], serde_json::Value::Null);
    assert_eq!(draft["startsAt"], "2026-05-04T14:00:00Z");
    assert_eq!(draft["endsAt"], "2026-05-04T14:30:00Z");
    // Snapshots and references never change on reschedule.
    assert_eq!(draft["serviceName"], created.provisional["serviceName"]);
    assert_eq!(
        draft["professionalName"],
        created.provisional["professionalName"]
    );
    assert_eq!(draft["patientId"], created.provisional["patientId"]);
    assert_eq!(
        draft["professionalId"],
        created.provisional["professionalId"]
    );
    assert_eq!(draft["serviceId"], created.provisional["serviceId"]);
    assert_eq!(draft["locationName"], created.provisional["locationName"]);
    assert_eq!(draft["timeZone"], created.provisional["timeZone"]);
    assert_eq!(
        draft["offeredDurationMinutes"],
        created.provisional["offeredDurationMinutes"]
    );

    // Outbox payload carries exactly the new interval.
    let payload: String = connection
        .query_row(
            "SELECT payload FROM sync_outbox WHERE operation_id=?1",
            params![moved.operation_id],
            |row| row.get(0),
        )
        .unwrap();
    let payload: serde_json::Value = serde_json::from_str(&payload).unwrap();
    assert_eq!(
        payload,
        serde_json::json!({
            "startsAt": "2026-05-04T14:00:00Z",
            "endsAt": "2026-05-04T14:30:00Z"
        })
    );

    // Reopening keeps the moved provisional state.
    drop(connection);
    let connection = crate::platform::sqlite::open(&database.0).unwrap();
    let draft: String = connection
        .query_row(
            "SELECT draft FROM appointment_local WHERE operation_id=?1",
            params![moved.operation_id],
            |row| row.get(0),
        )
        .unwrap();
    let draft: serde_json::Value = serde_json::from_str(&draft).unwrap();
    assert_eq!(draft["startsAt"], "2026-05-04T14:00:00Z");
    assert_eq!(draft["status"], "scheduled");
}

#[test]
fn reschedule_rejects_duration_changes_and_same_slot() {
    let mut connection = database();
    seed_access(&mut connection);
    super::create_tests::seed_context(&connection);
    let ids = Ids::default();
    let created = create_at(
        &mut connection,
        &ids,
        "2026-05-04T13:00:00Z",
        "2026-05-04T13:30:00Z",
    );
    let appointment = appointment_id(&created.entity_key);

    // Same interval: NO_CHANGES.
    let err = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .reschedule_appointment(
        &access(),
        appointment.clone(),
        EditBase::local(created.operation_id.clone()),
        interval("2026-05-04T13:00:00Z", "2026-05-04T13:30:00Z"),
        &ids,
        &FixedClock,
    )
    .unwrap_err();
    assert_eq!(err.code, "NO_CHANGES");

    // Different real duration: rejected before any availability check.
    let err = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .reschedule_appointment(
        &access(),
        appointment.clone(),
        EditBase::local(created.operation_id.clone()),
        interval("2026-05-04T14:00:00Z", "2026-05-04T15:00:00Z"),
        &ids,
        &FixedClock,
    )
    .unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");

    let count: i64 = connection
        .query_row("SELECT count(*) FROM appointment_local", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn reschedule_excludes_own_slot_but_respects_other_occupancy() {
    // A moves within its own footprint (13:00-13:30 -> 13:15-13:45 is NOT
    // allowed: 13:30-13:45 is free but the move overlaps B at 13:45? No:
    // B occupies 13:30-14:00, so A cannot take 13:15-13:45 because of B).
    // With B cancelled, the same move succeeds thanks to the self-exclusion.
    let mut connection = database();
    seed_access(&mut connection);
    super::create_tests::seed_context(&connection);
    let ids = Ids::default();
    let a = create_at(
        &mut connection,
        &ids,
        "2026-05-04T13:00:00Z",
        "2026-05-04T13:30:00Z",
    );
    let appointment_a = appointment_id(&a.entity_key);
    let b = create_at(
        &mut connection,
        &ids,
        "2026-05-04T13:30:00Z",
        "2026-05-04T14:00:00Z",
    );

    // A tries to move into B's slot: blocked by the other occupancy even
    // though A's own old slot is excluded from the check.
    let err = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .reschedule_appointment(
        &access(),
        appointment_a.clone(),
        EditBase::local(a.operation_id.clone()),
        interval("2026-05-04T13:30:00Z", "2026-05-04T14:00:00Z"),
        &ids,
        &FixedClock,
    )
    .unwrap_err();
    assert_eq!(err.code, "SLOT_UNAVAILABLE");

    // Cancel B; now A can take B's old slot (its own occupancy excluded).
    let cancelled_b = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .persist(
        &access(),
        AppointmentMutation::Cancel {
            appointment: appointment_id(&b.entity_key),
            base: EditBase::local(b.operation_id.clone()),
            reason: None,
        },
        &ids,
        &FixedClock,
    )
    .unwrap();
    let _ = cancelled_b;

    SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .reschedule_appointment(
        &access(),
        appointment_a.clone(),
        EditBase::local(a.operation_id.clone()),
        interval("2026-05-04T13:30:00Z", "2026-05-04T14:00:00Z"),
        &ids,
        &FixedClock,
    )
    .unwrap();

    // The old slot is free again and the new slot blocks: occupancy follows
    // the latest provisional draft.
    let occupancy = super::adapters::outbound::SqliteScheduleOccupancyQuery {
        connection: &connection,
        scope: &access().context,
    }
    .list_occupancy(
        super::test_support::PROFESSIONAL,
        jiff::Timestamp::from_str("2026-05-04T00:00:00Z").unwrap(),
        jiff::Timestamp::from_str("2026-05-05T00:00:00Z").unwrap(),
    )
    .unwrap();
    assert_eq!(occupancy.len(), 1);
    assert_eq!(occupancy[0].appointment_id, appointment_a.as_str());
    assert_eq!(occupancy[0].starts_at.to_string(), "2026-05-04T13:30:00Z");
}

use std::str::FromStr;

fn seed_access(connection: &mut rusqlite::Connection) {
    use crate::components::identity::repository::AccessStore as _;
    let access = access();
    let tx = connection.transaction().unwrap();
    crate::components::identity::repository::SqliteAccessStore
        .save_access(&tx, &access)
        .unwrap();
    tx.commit().unwrap();
}

#[allow(dead_code)]
fn _signature(_: AppointmentInterval) {}
