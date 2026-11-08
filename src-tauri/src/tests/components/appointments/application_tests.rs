//! Offline sequences and dependency plumbing for appointment writes.
use crate::components::appointments::{
    adapters::outbound::appointment_rows,
    application::{AppointmentMutation, AppointmentRepository, EditBase},
    domain::{AppointmentId, ChangeReason},
};
use crate::tests::components::appointments::test_support::{
    Database, FixedClock, Ids, access, database, interval, new_appointment, repository,
};
use rusqlite::params;

fn appointment_id(key: &serde_json::Value) -> AppointmentId {
    key.as_str().unwrap().to_owned().try_into().unwrap()
}

fn create(
    c: &mut rusqlite::Connection,
    ids: &Ids,
    depends_on: Vec<String>,
) -> crate::components::appointments::application::AppResult<
    crate::components::appointments::application::LocalMutationResult,
> {
    repository(c).persist(
        &access(),
        AppointmentMutation::Create {
            data: Box::new(new_appointment()),
            depends_on,
        },
        ids,
        &FixedClock,
    )
}

#[test]
fn create_confirm_reschedule_cancel_sequence_survives_reopen() {
    let database = Database::new();
    let mut connection = crate::platform::sqlite::open(&database.0).unwrap();
    seed_access(&mut connection);
    let ids = Ids::default();

    let created = create(&mut connection, &ids, vec![]).unwrap();
    assert_eq!(created.entity_type, "appointment");
    assert_eq!(created.expected_version, 0);
    let appointment = appointment_id(&created.entity_key);

    // Confirm on the local base.
    let confirmed = repository(&mut connection)
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

    // Reschedule on the new local base; the draft goes back to scheduled.
    let rescheduled = repository(&mut connection)
        .persist(
            &access(),
            AppointmentMutation::Reschedule {
                appointment: appointment.clone(),
                base: EditBase::local(confirmed.operation_id.clone()),
                interval: interval("2026-05-04T14:00:00Z", "2026-05-04T14:30:00Z"),
            },
            &ids,
            &FixedClock,
        )
        .unwrap();
    assert_eq!(rescheduled.expected_version, 2);

    // Cancel with a reason on the local base.
    let cancelled = repository(&mut connection)
        .persist(
            &access(),
            AppointmentMutation::Cancel {
                appointment: appointment.clone(),
                base: EditBase::local(rescheduled.operation_id.clone()),
                reason: ChangeReason::new(Some("paciente avisó".into())).unwrap(),
            },
            &ids,
            &FixedClock,
        )
        .unwrap();
    assert_eq!(cancelled.expected_version, 3);

    // Reopen: the provisional row shows the cancelled state and all four operations.
    drop(connection);
    let connection = crate::platform::sqlite::open(&database.0).unwrap();
    let row = appointment_rows(&connection, &access().context)
        .unwrap()
        .into_iter()
        .find(|row| {
            row.visible()
                .is_some_and(|v| v.appointment_id == appointment.as_str())
        })
        .unwrap();
    assert!(row.accepted.is_none());
    let provisional = row.provisional.as_ref().unwrap();
    assert_eq!(provisional.state.status, "cancelled");
    assert_eq!(provisional.expected_version, 3);
    assert_eq!(row.operations.len(), 4);
    assert!(row.operations.iter().all(|op| op.status == "pending"));
}

#[test]
fn stale_or_blocked_bases_reject_without_creating_operations() {
    let mut connection = database();
    let ids = Ids::default();

    let created = create(&mut connection, &ids, vec![]).unwrap();
    let appointment = appointment_id(&created.entity_key);

    // Accepted base while a provisional exists -> stale.
    let err = repository(&mut connection)
        .persist(
            &access(),
            AppointmentMutation::Confirm {
                appointment: appointment.clone(),
                base: EditBase::accepted(1),
            },
            &ids,
            &FixedClock,
        )
        .unwrap_err();
    assert_eq!(err.code, "VERSION_CONFLICT");

    // Local base pointing at another operation -> stale.
    let err = repository(&mut connection)
        .persist(
            &access(),
            AppointmentMutation::Confirm {
                appointment: appointment.clone(),
                base: EditBase::local("019a1234-0000-7000-8000-000000000999".into()),
            },
            &ids,
            &FixedClock,
        )
        .unwrap_err();
    assert_eq!(err.code, "VERSION_CONFLICT");

    // A conflicted prior operation blocks edits.
    connection
        .execute(
            "UPDATE sync_outbox SET status='conflict', last_error='\"SLOT_UNAVAILABLE\"' WHERE operation_id=?1",
            params![created.operation_id],
        )
        .unwrap();
    let err = repository(&mut connection)
        .persist(
            &access(),
            AppointmentMutation::Confirm {
                appointment: appointment.clone(),
                base: EditBase::local(created.operation_id.clone()),
            },
            &ids,
            &FixedClock,
        )
        .unwrap_err();
    assert_eq!(err.code, "DEPENDENCY_FAILED");

    // None of the rejections created an intention or an Outbox record.
    let count: i64 = connection
        .query_row("SELECT count(*) FROM appointment_local", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(count, 1);
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
fn dependencies_are_validated_deduplicated_and_capped() {
    let mut connection = database();
    let ids = Ids::default();

    // A dependency must be an existing operation of this scope.
    let err = create(&mut connection, &ids, vec!["not-a-uuid".into()]).unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");
    let err = create(
        &mut connection,
        &ids,
        vec!["019a1234-0000-7000-8000-000000000777".into()],
    )
    .unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");

    // A real operation from another component can anchor a new appointment.
    let prior = create(&mut connection, &ids, vec![]).unwrap();
    let anchored = create(&mut connection, &ids, vec![prior.operation_id.clone()]).unwrap();
    assert_eq!(anchored.depends_on, [prior.operation_id.as_str()]);

    // Duplicates are rejected.
    let err = create(
        &mut connection,
        &ids,
        vec![prior.operation_id.clone(), prior.operation_id.clone()],
    )
    .unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");

    // More than eight dependencies are rejected.
    let nine: Vec<String> = (0..9)
        .map(|index| format!("019a1234-0000-7000-8000-{:012}", 900 + index))
        .collect();
    // Seed the nine operations so only the cap applies.
    for dep in &nine {
        connection
            .execute(
                "INSERT INTO sync_outbox (operation_id,contract_version,action,entity_type,entity_key,expected_version,payload,occurred_at_local,created_at,account_id,user_id,device_id,permission_epoch,depends_on,semantic_fingerprint,status,attempts,last_error,result,lease_until,next_attempt_at)
                 VALUES (?1,1,'RegisterPatient','patient_profile','\"019a1234-0000-7000-8000-000000000010\"',0,'{}','2026-11-08T10:00:00Z',1,'019a1234-0000-7000-8000-000000000001','019a1234-0000-7000-8000-000000000002','019a1234-0000-7000-8000-000000000003',1,'[]',NULL,'pending',0,NULL,NULL,NULL,NULL)",
                params![dep],
            )
            .unwrap();
    }
    let err = create(&mut connection, &ids, nine.clone()).unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");
    create(&mut connection, &ids, nine[..8].to_vec()).unwrap();

    // A conflicted dependency cannot anchor anything.
    connection
        .execute(
            "UPDATE sync_outbox SET status='conflict' WHERE operation_id=?1",
            params![prior.operation_id],
        )
        .unwrap();
    let err = create(&mut connection, &ids, vec![prior.operation_id.clone()]).unwrap_err();
    assert_eq!(err.code, "DEPENDENCY_FAILED");
}

#[test]
fn no_changes_and_terminal_transitions_create_no_operation() {
    let mut connection = database();
    let ids = Ids::default();

    let created = create(&mut connection, &ids, vec![]).unwrap();
    let appointment = appointment_id(&created.entity_key);

    // Reschedule to the same interval -> NO_CHANGES.
    let err = repository(&mut connection)
        .persist(
            &access(),
            AppointmentMutation::Reschedule {
                appointment: appointment.clone(),
                base: EditBase::local(created.operation_id.clone()),
                interval: interval("2026-05-04T13:00:00Z", "2026-05-04T13:30:00Z"),
            },
            &ids,
            &FixedClock,
        )
        .unwrap_err();
    assert_eq!(err.code, "NO_CHANGES");

    // Confirm -> confirmed, then confirming again is an invalid transition.
    let confirmed = repository(&mut connection)
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
    let err = repository(&mut connection)
        .persist(
            &access(),
            AppointmentMutation::Confirm {
                appointment: appointment.clone(),
                base: EditBase::local(confirmed.operation_id.clone()),
            },
            &ids,
            &FixedClock,
        )
        .unwrap_err();
    assert_eq!(err.code, "INVALID_STATE_TRANSITION");

    // Cancel reaches the terminal state; a second cancel is invalid.
    let cancelled = repository(&mut connection)
        .persist(
            &access(),
            AppointmentMutation::Cancel {
                appointment: appointment.clone(),
                base: EditBase::local(confirmed.operation_id.clone()),
                reason: None,
            },
            &ids,
            &FixedClock,
        )
        .unwrap();
    let err = repository(&mut connection)
        .persist(
            &access(),
            AppointmentMutation::Cancel {
                appointment: appointment.clone(),
                base: EditBase::local(cancelled.operation_id.clone()),
                reason: None,
            },
            &ids,
            &FixedClock,
        )
        .unwrap_err();
    assert_eq!(err.code, "INVALID_STATE_TRANSITION");

    let count: i64 = connection
        .query_row("SELECT count(*) FROM appointment_local", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(count, 3);
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
