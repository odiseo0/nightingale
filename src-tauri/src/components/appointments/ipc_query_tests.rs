//! IPC query path: listings with range/filters/pagination, merged detail,
//! accepted history separate from local operations, and booking context.
use super::adapters::inbound_commands::create_appointment;
use super::adapters::inbound_commands::{CreateAppointmentRequest, EditBaseRequest};
use super::adapters::inbound_queries::{
    GetAppointmentRequest, GetHistoryRequest, ListAppointmentsRequest, get_appointment,
    get_booking_context, get_history, list_appointments,
};
use super::create_tests::seed_context;
use super::test_support::{PATIENT, PROFESSIONAL, SERVICE, access};
use serde::Deserialize;

fn database() -> rusqlite::Connection {
    let mut connection = crate::platform::sqlite::open(std::path::Path::new(":memory:")).unwrap();
    {
        use crate::components::identity::repository::AccessStore as _;
        let access = access();
        let tx = connection.transaction().unwrap();
        crate::components::identity::repository::SqliteAccessStore
            .save_access(&tx, &access)
            .unwrap();
        crate::components::identity::repository::SqliteAccessStore
            .save_context(
                &tx,
                &crate::components::identity::domain::IdentityContext {
                    issuer: "https://synthetic.example".into(),
                    subject: "synthetic".into(),
                    user_id: access.context.user_id,
                    selected_account_id: Some(access.context.account_id),
                },
            )
            .unwrap();
        tx.commit().unwrap();
    }
    seed_context(&connection);
    connection
}

fn create_at(
    connection: &mut rusqlite::Connection,
    starts: &str,
    ends: &str,
) -> super::application::LocalMutationResult {
    create_appointment(
        connection,
        CreateAppointmentRequest {
            patient_id: PATIENT.into(),
            professional_id: PROFESSIONAL.into(),
            service_id: SERVICE.into(),
            responsible_contact_id: None,
            starts_at: starts.into(),
            ends_at: ends.into(),
        },
    )
    .unwrap()
}

#[test]
fn list_intersects_filters_and_paginates() {
    let mut connection = database();
    let a = create_at(
        &mut connection,
        "2026-05-04T13:00:00Z",
        "2026-05-04T13:30:00Z",
    );
    let b = create_at(
        &mut connection,
        "2026-05-04T14:00:00Z",
        "2026-05-04T14:30:00Z",
    );

    // Both overlap the day window.
    let page = list_appointments(
        &mut connection,
        ListAppointmentsRequest {
            from: "2026-05-04T00:00:00Z".into(),
            until: "2026-05-05T00:00:00Z".into(),
            professional_id: None,
            service_id: None,
            patient_id: None,
            status: None,
            page: None,
            shows: None,
        },
    )
    .unwrap();
    assert_eq!(page.total, 2);
    assert!(page.items.iter().all(|row| row.provisional.is_some()));

    // Intersection: a window inside only the first appointment still lists it.
    let page = list_appointments(
        &mut connection,
        ListAppointmentsRequest {
            from: "2026-05-04T13:10:00Z".into(),
            until: "2026-05-04T13:20:00Z".into(),
            professional_id: None,
            service_id: None,
            patient_id: None,
            status: None,
            page: None,
            shows: None,
        },
    )
    .unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(
        page.items[0]
            .provisional
            .as_ref()
            .unwrap()
            .state
            .appointment_id,
        a.entity_key.as_str().unwrap()
    );

    // Filters.
    let page = list_appointments(
        &mut connection,
        ListAppointmentsRequest {
            from: "2026-05-04T00:00:00Z".into(),
            until: "2026-05-05T00:00:00Z".into(),
            professional_id: Some(PROFESSIONAL.into()),
            service_id: Some(SERVICE.into()),
            patient_id: Some(PATIENT.into()),
            status: Some("scheduled".into()),
            page: None,
            shows: Some(1),
        },
    )
    .unwrap();
    assert_eq!(page.total, 2);
    assert_eq!(page.items.len(), 1, "shows=1 paginates");

    // Status filter excludes non-matching rows.
    let page = list_appointments(
        &mut connection,
        ListAppointmentsRequest {
            from: "2026-05-04T00:00:00Z".into(),
            until: "2026-05-05T00:00:00Z".into(),
            professional_id: None,
            service_id: None,
            patient_id: None,
            status: Some("cancelled".into()),
            page: None,
            shows: None,
        },
    )
    .unwrap();
    assert_eq!(page.total, 0);

    // Range and pagination validation.
    let err = list_appointments(
        &mut connection,
        ListAppointmentsRequest {
            from: "2026-05-04T00:00:00Z".into(),
            until: "2028-05-04T00:00:00Z".into(),
            professional_id: None,
            service_id: None,
            patient_id: None,
            status: None,
            page: None,
            shows: None,
        },
    )
    .unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");

    let err = list_appointments(
        &mut connection,
        ListAppointmentsRequest {
            from: "2026-05-04T00:00:00Z".into(),
            until: "2026-05-05T00:00:00Z".into(),
            professional_id: None,
            service_id: None,
            patient_id: None,
            status: Some("reschedule_requested".into()),
            page: None,
            shows: None,
        },
    )
    .unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");
    let _ = b;
}

#[test]
fn detail_returns_merged_row_and_history_stays_separated() {
    let mut connection = database();
    let created = create_at(
        &mut connection,
        "2026-05-04T13:00:00Z",
        "2026-05-04T13:30:00Z",
    );
    let appointment_id = created.entity_key.as_str().unwrap().to_owned();

    let confirmed = super::adapters::inbound_commands::confirm_appointment(
        &mut connection,
        super::adapters::inbound_commands::ConfirmAppointmentRequest {
            appointment_id: appointment_id.clone(),
            base: EditBaseRequest {
                accepted_version: None,
                operation_id: Some(created.operation_id.clone()),
            },
        },
    )
    .unwrap();

    // Detail: accepted absent, provisional confirmed, both operations listed.
    let row = get_appointment(
        &mut connection,
        GetAppointmentRequest {
            appointment_id: appointment_id.clone(),
        },
    )
    .unwrap();
    assert!(row.accepted.is_none());
    assert_eq!(row.provisional.as_ref().unwrap().state.status, "confirmed");
    assert_eq!(row.operations.len(), 2);
    assert!(row.operations.iter().all(|op| op.status == "pending"));

    // Unknown detail.
    let err = get_appointment(
        &mut connection,
        GetAppointmentRequest {
            appointment_id: "019a1234-0000-7000-8000-000000000099".into(),
        },
    )
    .unwrap_err();
    assert_eq!(err.code, "NOT_FOUND");

    // No accepted history yet: accepted list empty, local operations separate.
    let history = get_history(
        &mut connection,
        GetHistoryRequest {
            appointment_id: appointment_id.clone(),
            after_version: None,
        },
    )
    .unwrap();
    assert!(history.accepted.is_empty());
    assert_eq!(history.operations.len(), 2);

    // Project accepted appointment + history v1..2 via the repository helpers.
    project_accepted(&mut connection, &appointment_id, 2, "confirmed");

    let history = get_history(
        &mut connection,
        GetHistoryRequest {
            appointment_id: appointment_id.clone(),
            after_version: None,
        },
    )
    .unwrap();
    assert_eq!(history.accepted.len(), 2);
    assert_eq!(history.accepted[1].status, "confirmed");

    // afterVersion filters the accepted list only.
    let history = get_history(
        &mut connection,
        GetHistoryRequest {
            appointment_id: appointment_id.clone(),
            after_version: Some(1),
        },
    )
    .unwrap();
    assert_eq!(history.accepted.len(), 1);
    assert_eq!(history.accepted[0].version, 2);
    let _ = confirmed;

    // afterVersion cannot be negative.
    let err = get_history(
        &mut connection,
        GetHistoryRequest {
            appointment_id: appointment_id.clone(),
            after_version: Some(-1),
        },
    )
    .unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");
}

fn project_accepted(connection: &mut rusqlite::Connection, id: &str, version: i64, status: &str) {
    use super::adapters::outbound::{
        AppointmentHistoryRecord, AppointmentRecord, save_appointment, save_history_entry,
    };
    use super::test_support::snapshots;
    let scope = access().context;
    crate::platform::sqlite::in_transaction(connection, |tx| {
        for item_version in 1..=version {
            save_appointment(
                tx,
                &scope,
                &AppointmentRecord {
                    appointment_id: id.into(),
                    patient_id: PATIENT.into(),
                    professional_id: PROFESSIONAL.into(),
                    service_id: SERVICE.into(),
                    responsible_contact_id: None,
                    starts_at: "2026-05-04T13:00:00Z".into(),
                    ends_at: "2026-05-04T13:30:00Z".into(),
                    status: if item_version == version {
                        status.into()
                    } else {
                        "scheduled".into()
                    },
                    confirmed_at: None,
                    snapshots: snapshots(),
                    created_at: None,
                    updated_at: None,
                    version: item_version,
                },
            )?;
            save_history_entry(
                tx,
                &scope,
                &AppointmentHistoryRecord {
                    appointment_id: id.into(),
                    version: item_version,
                    action: if item_version == 1 {
                        "created"
                    } else {
                        "confirmed"
                    }
                    .into(),
                    actor_kind: "staff".into(),
                    actor_user_id: Some(scope.user_id.as_str().into()),
                    actor_contact_person_id: None,
                    change_reason: None,
                    changed_at: "2026-05-01T10:00:00Z".into(),
                    starts_at: "2026-05-04T13:00:00Z".into(),
                    ends_at: "2026-05-04T13:30:00Z".into(),
                    status: if item_version == version {
                        status.into()
                    } else {
                        "scheduled".into()
                    },
                    confirmed_at: None,
                    snapshots: snapshots(),
                },
            )?;
        }
        Ok(())
    })
    .unwrap();
}

#[test]
fn booking_context_marks_provisional_rows() {
    let mut connection = database();
    create_at(
        &mut connection,
        "2026-05-04T13:00:00Z",
        "2026-05-04T13:30:00Z",
    );

    let context = get_booking_context(&mut connection).unwrap();
    assert_eq!(context.time_zone, "America/Caracas");
    assert_eq!(context.no_show_grace_minutes, 15);
    assert_eq!(
        context.patients.len(),
        1,
        "accepted active patient is usable"
    );
    assert!(context.patients[0].provisional.is_none());
    assert_eq!(context.offers.len(), 1);
    assert!(context.offers[0].accepted.is_some());
    assert_eq!(context.professionals.len(), 1);

    // DTO hygiene: outputs carry no epoch, fingerprint, token or raw remote state.
    let value = serde_json::to_value(&context).unwrap();
    let text = value.to_string();
    for forbidden in ["permissionEpoch", "semanticFingerprint", "token", "result"] {
        assert!(!text.contains(forbidden), "{forbidden} leaked");
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct _Probe {
    from: String,
}
