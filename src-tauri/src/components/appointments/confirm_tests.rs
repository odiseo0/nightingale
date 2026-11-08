//! ConfirmAppointment: scheduled -> confirmed only, empty payload, no local
//! confirmation timestamp, bases tied to the same appointment.
use super::{
    adapters::outbound::{AppointmentRecord, SqliteAppointmentRepository, save_appointment},
    application::{AppointmentMutation, AppointmentRepository, EditBase},
    domain::AppointmentId,
    test_support::{
        Database, FixedClock, Ids, PATIENT, PROFESSIONAL, SERVICE, access, database, interval,
        new_appointment, repository, snapshots,
    },
};
use rusqlite::params;

fn appointment_id(key: &serde_json::Value) -> AppointmentId {
    key.as_str().unwrap().to_owned().try_into().unwrap()
}

fn create(
    connection: &mut rusqlite::Connection,
    ids: &Ids,
) -> super::application::LocalMutationResult {
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
fn confirm_moves_scheduled_to_confirmed_with_empty_payload() {
    let database = Database::new();
    let mut connection = crate::platform::sqlite::open(&database.0).unwrap();
    seed_access(&mut connection);
    let ids = Ids::default();

    let created = create(&mut connection, &ids);
    let appointment = appointment_id(&created.entity_key);

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
    assert_eq!(confirmed.expected_version, 1);

    // The Outbox payload is exactly the EmptyAppointmentPayload.
    let payload: String = connection
        .query_row(
            "SELECT payload FROM sync_outbox WHERE operation_id=?1",
            params![confirmed.operation_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(payload, "{}");

    // The confirmation depends on the same appointment's create operation.
    assert_eq!(confirmed.depends_on, [created.operation_id.as_str()]);

    // The provisional draft is confirmed but confirmedAt stays null: only the
    // server assigns the confirmation time.
    assert_eq!(confirmed.provisional["status"], "confirmed");
    assert_eq!(
        confirmed.provisional["confirmedAt"],
        serde_json::Value::Null
    );
    assert_eq!(
        confirmed.provisional["startsAt"],
        created.provisional["startsAt"]
    );
    assert_eq!(
        confirmed.provisional["endsAt"],
        created.provisional["endsAt"]
    );

    // Reopening keeps the provisional confirmed state pending sync.
    drop(connection);
    let connection = crate::platform::sqlite::open(&database.0).unwrap();
    let draft: String = connection
        .query_row(
            "SELECT draft FROM appointment_local WHERE operation_id=?1",
            params![confirmed.operation_id],
            |row| row.get(0),
        )
        .unwrap();
    let draft: serde_json::Value = serde_json::from_str(&draft).unwrap();
    assert_eq!(draft["status"], "confirmed");
}

#[test]
fn confirm_accepts_a_clean_accepted_base() {
    let mut connection = database();
    let ids = Ids::default();
    let created = create(&mut connection, &ids);
    let appointment = appointment_id(&created.entity_key);
    let appointment = appointment.as_str().to_owned();

    // Accept the create: receipt accepted and the accepted view at version 1.
    accept_operation(&connection, &created.operation_id, 1);
    let scope = access().context;
    let record = AppointmentRecord {
        appointment_id: appointment.clone(),
        patient_id: PATIENT.into(),
        professional_id: PROFESSIONAL.into(),
        service_id: SERVICE.into(),
        responsible_contact_id: None,
        starts_at: "2026-05-04T13:00:00Z".into(),
        ends_at: "2026-05-04T13:30:00Z".into(),
        status: "scheduled".into(),
        confirmed_at: None,
        snapshots: snapshots(),
        created_at: None,
        updated_at: None,
        version: 1,
    };
    crate::platform::sqlite::in_transaction(&mut connection, |tx| {
        save_appointment(tx, &scope, &record)
    })
    .unwrap();

    let confirmed = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .persist(
        &access(),
        AppointmentMutation::Confirm {
            appointment: appointment.clone().try_into().unwrap(),
            base: EditBase::accepted(1),
        },
        &ids,
        &FixedClock,
    )
    .unwrap();
    assert_eq!(confirmed.expected_version, 1);
    assert!(confirmed.depends_on.is_empty());
}

#[test]
fn confirm_rejects_wrong_states_bases_and_foreign_operations() {
    let mut connection = database();
    let ids = Ids::default();

    let created_a = create(&mut connection, &ids);
    let appointment_a = appointment_id(&created_a.entity_key);
    let created_b = create(&mut connection, &ids);

    // A local base from another appointment is not this row's provisional.
    let err = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .persist(
        &access(),
        AppointmentMutation::Confirm {
            appointment: appointment_a.clone(),
            base: EditBase::local(created_b.operation_id.clone()),
        },
        &ids,
        &FixedClock,
    )
    .unwrap_err();
    assert_eq!(err.code, "VERSION_CONFLICT");

    // Confirm A, then confirming again is an invalid transition.
    let confirmed = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .persist(
        &access(),
        AppointmentMutation::Confirm {
            appointment: appointment_a.clone(),
            base: EditBase::local(created_a.operation_id.clone()),
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
        AppointmentMutation::Confirm {
            appointment: appointment_a.clone(),
            base: EditBase::local(confirmed.operation_id.clone()),
        },
        &ids,
        &FixedClock,
    )
    .unwrap_err();
    assert_eq!(err.code, "INVALID_STATE_TRANSITION");

    // Stale accepted base with a provisional present.
    let err = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .persist(
        &access(),
        AppointmentMutation::Confirm {
            appointment: appointment_a.clone(),
            base: EditBase::accepted(7),
        },
        &ids,
        &FixedClock,
    )
    .unwrap_err();
    assert_eq!(err.code, "VERSION_CONFLICT");

    // None of the rejections created an operation beyond the two creates and
    // the single successful confirm.
    let count: i64 = connection
        .query_row(
            "SELECT count(*) FROM sync_outbox WHERE entity_type='appointment'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 3);
}

#[test]
fn confirm_on_terminal_states_is_invalid() {
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

    let err = SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .persist(
        &access(),
        AppointmentMutation::Confirm {
            appointment: appointment.clone(),
            base: EditBase::local(cancelled.operation_id.clone()),
        },
        &ids,
        &FixedClock,
    )
    .unwrap_err();
    assert_eq!(err.code, "INVALID_STATE_TRANSITION");
    let _ = repository(&mut connection);
    let _ = interval("2026-05-04T13:00:00Z", "2026-05-04T13:30:00Z");
    let _ = new_appointment();
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

fn accept_operation(connection: &rusqlite::Connection, operation_id: &str, entity_version: i64) {
    let result = serde_json::json!({
        "operationId": operation_id,
        "status": "accepted",
        "entityVersion": entity_version,
        "state": null,
        "error": null
    });
    connection
        .execute(
            "UPDATE sync_outbox SET status='accepted', result=?2 WHERE operation_id=?1",
            params![operation_id, result.to_string()],
        )
        .unwrap();
}
