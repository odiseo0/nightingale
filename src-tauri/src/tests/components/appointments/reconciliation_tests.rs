//! Provisional reconciliation: a receipt alone retires nothing; the accepted
//! appointment AND its history must reach the confirmed version. Conflicts,
//! rejections and unknown results keep the provisional for review, and an old
//! local intention never overwrites a newer remote state.
use crate::components::appointments::{
    adapters::outbound::{
        AppointmentHistoryRecord, AppointmentRecord, SqliteAppointmentRepository, appointment_rows,
        save_appointment, save_history_entry,
    },
    application::{AppointmentMutation, AppointmentRepository, EditBase},
    domain::AppointmentId,
};
use crate::tests::components::appointments::test_support::{
    Database, FixedClock, Ids, PATIENT, PROFESSIONAL, SERVICE, access, database, new_appointment,
    snapshots,
};
use rusqlite::params;

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

fn receipt(
    connection: &rusqlite::Connection,
    operation_id: &str,
    status: &str,
    version: Option<i64>,
) {
    let result = serde_json::json!({
        "operationId": operation_id,
        "status": if status == "accepted" { "accepted" } else { status },
        "entityVersion": version,
        "state": null,
        "error": null
    });
    connection
        .execute(
            "UPDATE sync_outbox SET status=?2, result=?3 WHERE operation_id=?1",
            params![operation_id, status, result.to_string()],
        )
        .unwrap();
}

fn accepted_record(id: &str, version: i64, status: &str) -> AppointmentRecord {
    AppointmentRecord {
        appointment_id: id.into(),
        patient_id: PATIENT.into(),
        professional_id: PROFESSIONAL.into(),
        service_id: SERVICE.into(),
        responsible_contact_id: None,
        starts_at: "2026-05-04T13:00:00Z".into(),
        ends_at: "2026-05-04T13:30:00Z".into(),
        status: status.into(),
        confirmed_at: None,
        snapshots: snapshots(),
        created_at: None,
        updated_at: None,
        version,
    }
}

fn history_record(id: &str, version: i64, status: &str) -> AppointmentHistoryRecord {
    AppointmentHistoryRecord {
        appointment_id: id.into(),
        version,
        action: "created".into(),
        actor_kind: "staff".into(),
        actor_user_id: Some(access().context.user_id.as_str().into()),
        actor_contact_person_id: None,
        change_reason: None,
        changed_at: "2026-05-01T10:00:00Z".into(),
        starts_at: "2026-05-04T13:00:00Z".into(),
        ends_at: "2026-05-04T13:30:00Z".into(),
        status: status.into(),
        confirmed_at: None,
        snapshots: snapshots(),
    }
}

fn project_accepted(
    connection: &mut rusqlite::Connection,
    id: &str,
    version: i64,
    status: &str,
    with_history: bool,
) {
    let scope = access().context;
    crate::platform::sqlite::in_transaction(connection, |tx| {
        save_appointment(tx, &scope, &accepted_record(id, version, status))?;
        if with_history {
            save_history_entry(tx, &scope, &history_record(id, version, status))?;
        }
        Ok(())
    })
    .unwrap();
}

fn row_of<'a>(
    rows: &'a [crate::components::appointments::application::EntityRow<
        crate::components::appointments::application::AppointmentState,
    >],
    id: &str,
) -> &'a crate::components::appointments::application::EntityRow<
    crate::components::appointments::application::AppointmentState,
> {
    rows.iter()
        .find(|row| row.visible().is_some_and(|v| v.appointment_id == id))
        .unwrap()
}

#[test]
fn receipt_alone_never_retires_the_provisional() {
    let mut connection = database();
    let ids = Ids::default();
    let created = create(&mut connection, &ids);
    let appointment = appointment_id(&created.entity_key);
    let id = appointment.as_str().to_owned();

    // Receipt accepted but nothing downloaded yet.
    receipt(&connection, &created.operation_id, "accepted", Some(1));
    let rows = appointment_rows(&connection, &access().context).unwrap();
    assert!(row_of(&rows, &id).provisional.is_some());

    // The accepted appointment arrives without its history: still unresolved.
    project_accepted(&mut connection, &id, 1, "scheduled", false);
    let rows = appointment_rows(&connection, &access().context).unwrap();
    assert!(row_of(&rows, &id).provisional.is_some());

    // The intention itself is never deleted by reconciliation.
    let count: i64 = connection
        .query_row("SELECT count(*) FROM appointment_local", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn receipt_plus_appointment_and_history_retires_only_the_confirmed_provisional() {
    let mut connection = database();
    let ids = Ids::default();
    let created = create(&mut connection, &ids);
    let appointment = appointment_id(&created.entity_key);
    let id = appointment.as_str().to_owned();

    // Confirm on the local base: a two-step chain.
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

    // Both steps accepted, but only the create has been projected.
    receipt(&connection, &created.operation_id, "accepted", Some(1));
    receipt(&connection, &confirmed.operation_id, "accepted", Some(2));
    project_accepted(&mut connection, &id, 1, "scheduled", true);

    // The create provisional resolves; the confirm provisional stays because
    // the accepted views are still behind its confirmed version 2.
    let rows = appointment_rows(&connection, &access().context).unwrap();
    let row = row_of(&rows, &id);
    assert_eq!(row.provisional.as_ref().unwrap().state.status, "confirmed");
    assert_eq!(row.operations.len(), 2);

    // Views reach version 2 (appointment and history): confirm resolves.
    project_accepted(&mut connection, &id, 2, "confirmed", true);
    let rows = appointment_rows(&connection, &access().context).unwrap();
    let row = row_of(&rows, &id);
    assert!(row.provisional.is_none());
    assert_eq!(row.accepted.as_ref().unwrap().state.status, "confirmed");
    assert_eq!(row.accepted.as_ref().unwrap().version, 2);
    assert_eq!(
        row.operations.len(),
        2,
        "operations stay visible for review"
    );
}

#[test]
fn conflict_rejection_and_unknown_keep_the_provisional() {
    for status in ["conflict", "rejected", "unknown"] {
        let mut connection = database();
        let ids = Ids::default();
        let created = create(&mut connection, &ids);
        let appointment = appointment_id(&created.entity_key);
        let id = appointment.as_str().to_owned();

        // The download already shows the change, but the result is not an
        // acceptance: the provisional remains for review.
        receipt(&connection, &created.operation_id, status, None);
        project_accepted(&mut connection, &id, 1, "scheduled", true);

        let rows = appointment_rows(&connection, &access().context).unwrap();
        let row = row_of(&rows, &id);
        assert!(
            row.provisional.is_some(),
            "{status} must keep the provisional visible"
        );
        assert_eq!(row.operations[0].status, status);
    }
}

#[test]
fn remote_cancel_is_never_overwritten_by_an_old_local_intention() {
    let mut connection = database();
    let ids = Ids::default();
    let created = create(&mut connection, &ids);
    let appointment = appointment_id(&created.entity_key);
    let id = appointment.as_str().to_owned();

    // The server accepted the create, then a remote cancellation arrived at
    // version 2 while the local create operation is still pending.
    project_accepted(&mut connection, &id, 2, "cancelled", true);

    // The accepted view stays cancelled; the old local provisional is still
    // visible (for review) but shows under the newer accepted state.
    let rows = appointment_rows(&connection, &access().context).unwrap();
    let row = row_of(&rows, &id);
    assert_eq!(row.accepted.as_ref().unwrap().state.status, "cancelled");
    assert_eq!(row.accepted.as_ref().unwrap().version, 2);
    assert_eq!(row.provisional.as_ref().unwrap().state.status, "scheduled");

    // Bases tied to the stale provisional still resolve (manual correction is
    // an explicit new operation), but the accepted row is untouched: the
    // provisional is presentation, never a write to the accepted view.
    let count: i64 = connection
        .query_row(
            "SELECT count(*) FROM appointment_view WHERE status='scheduled'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
    let _ = created;
}

#[test]
fn chain_survives_reopen_with_ids_and_dependencies() {
    let database = Database::new();
    let mut connection = crate::platform::sqlite::open(&database.0).unwrap();
    {
        use crate::components::identity::repository::AccessStore as _;
        let access = access();
        let tx = connection.transaction().unwrap();
        crate::components::identity::repository::SqliteAccessStore
            .save_access(&tx, &access)
            .unwrap();
        tx.commit().unwrap();
    }
    let ids = Ids::default();
    let created = create(&mut connection, &ids);
    let appointment = appointment_id(&created.entity_key);
    SqliteAppointmentRepository {
        connection: &mut connection,
    }
    .persist(
        &access(),
        AppointmentMutation::Reschedule {
            appointment: appointment.clone(),
            base: EditBase::local(created.operation_id.clone()),
            interval: crate::tests::components::appointments::test_support::interval(
                "2026-05-04T14:00:00Z",
                "2026-05-04T14:30:00Z",
            ),
        },
        &ids,
        &FixedClock,
    )
    .unwrap();

    // Partial acceptance: only the create is accepted; the reschedule keeps
    // its dependency on the original operation id after reopening.
    receipt(&connection, &created.operation_id, "accepted", Some(1));

    drop(connection);
    let connection = crate::platform::sqlite::open(&database.0).unwrap();
    let rows = appointment_rows(&connection, &access().context).unwrap();
    let row = row_of(&rows, appointment.as_str());
    assert_eq!(row.operations.len(), 2);
    assert_eq!(
        row.operations[1].depends_on,
        [created.operation_id.as_str()]
    );
    assert_eq!(
        row.provisional.as_ref().unwrap().state.starts_at,
        "2026-05-04T14:00:00Z"
    );
}
