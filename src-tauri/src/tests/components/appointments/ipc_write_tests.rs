//! IPC write path: DTOs reject foreign input and the six commands drive the
//! real local flow end to end.
use super::create_tests::seed_context;
use super::test_support::{FixedClock, Ids, PATIENT, PROFESSIONAL, SERVICE, access};
use crate::components::appointments::adapters::inbound_commands::{
    ConfirmAppointmentRequest, CreateAppointmentRequest, EditBaseRequest,
    LifecycleAppointmentRequest, RescheduleAppointmentRequest, cancel_appointment,
    confirm_appointment, create_appointment, mark_no_show, reschedule_appointment,
};
use std::str::FromStr;

fn base_local(operation_id: &str) -> EditBaseRequest {
    EditBaseRequest {
        accepted_version: None,
        operation_id: Some(operation_id.into()),
    }
}

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

#[test]
fn dtos_reject_extra_fields_and_ambiguous_bases() {
    // deny_unknown_fields on every write DTO.
    assert!(
        serde_json::from_value::<CreateAppointmentRequest>(serde_json::json!({
            "patientId": PATIENT,
            "professionalId": PROFESSIONAL,
            "serviceId": SERVICE,
            "startsAt": "2026-05-04T13:00:00Z",
            "endsAt": "2026-05-04T13:30:00Z",
            "accountId": "injected-scope"
        }))
        .is_err()
    );
    let ambiguous_base: ConfirmAppointmentRequest = serde_json::from_value(serde_json::json!({
        "appointmentId": "019a1234-0000-7000-8000-000000000010",
        "base": {"acceptedVersion": 1, "operationId": "019a1234-0000-7000-8000-000000000011"}
    }))
    .unwrap();
    let base: Result<crate::components::appointments::application::EditBase, _> =
        ambiguous_base.base.try_into();
    assert!(base.is_err());
    assert!(
        serde_json::from_value::<RescheduleAppointmentRequest>(serde_json::json!({
            "appointmentId": "019a1234-0000-7000-8000-000000000010",
            "base": {"operationId": "019a1234-0000-7000-8000-000000000011"},
            "startsAt": "2026-05-04T14:00:00Z",
            "endsAt": "2026-05-04T14:30:00Z",
            "sql": "SELECT 1"
        }))
        .is_err()
    );
    let non_positive_base: LifecycleAppointmentRequest =
        serde_json::from_value(serde_json::json!({
            "appointmentId": "019a1234-0000-7000-8000-000000000010",
            "base": {"acceptedVersion": 0},
            "changeReason": "x"
        }))
        .unwrap();
    let base: Result<crate::components::appointments::application::EditBase, _> =
        non_positive_base.base.try_into();
    assert!(base.is_err());

    // Bases: exactly one of a positive accepted version or a local operation.
    let ambiguous = serde_json::from_value::<EditBaseRequest>(serde_json::json!({
        "acceptedVersion": 1,
        "operationId": "019a1234-0000-7000-8000-000000000011"
    }))
    .unwrap();
    let base: Result<crate::components::appointments::application::EditBase, _> =
        ambiguous.try_into();
    assert!(base.is_err());

    let empty = serde_json::from_value::<EditBaseRequest>(serde_json::json!({})).unwrap();
    let base: Result<crate::components::appointments::application::EditBase, _> = empty.try_into();
    assert!(base.is_err());

    let non_positive = serde_json::from_value::<EditBaseRequest>(serde_json::json!({
        "acceptedVersion": 0
    }))
    .unwrap();
    let base: Result<crate::components::appointments::application::EditBase, _> =
        non_positive.try_into();
    assert!(base.is_err());
}

#[test]
fn create_confirm_reschedule_cancel_flow_via_commands() {
    let mut connection = database();
    let _ids = Ids(std::cell::Cell::new(1000));

    let created = create_appointment(
        &mut connection,
        CreateAppointmentRequest {
            patient_id: PATIENT.into(),
            professional_id: PROFESSIONAL.into(),
            service_id: SERVICE.into(),
            responsible_contact_id: None,
            starts_at: "2026-05-04T13:00:00Z".into(),
            ends_at: "2026-05-04T13:30:00Z".into(),
        },
    )
    .unwrap();
    let appointment_id = created.entity_key.as_str().unwrap().to_owned();

    let confirmed = confirm_appointment(
        &mut connection,
        ConfirmAppointmentRequest {
            appointment_id: appointment_id.clone(),
            base: base_local(&created.operation_id),
        },
    )
    .unwrap();
    assert_eq!(confirmed.provisional["status"], "confirmed");

    let rescheduled = reschedule_appointment(
        &mut connection,
        RescheduleAppointmentRequest {
            appointment_id: appointment_id.clone(),
            base: base_local(&confirmed.operation_id),
            starts_at: "2026-05-04T14:00:00Z".into(),
            ends_at: "2026-05-04T14:30:00Z".into(),
        },
    )
    .unwrap();
    assert_eq!(rescheduled.provisional["status"], "scheduled");
    assert_eq!(
        rescheduled.provisional["confirmedAt"],
        serde_json::Value::Null
    );

    let cancelled = cancel_appointment(
        &mut connection,
        LifecycleAppointmentRequest {
            appointment_id: appointment_id.clone(),
            base: base_local(&rescheduled.operation_id),
            change_reason: Some("vía comando".into()),
        },
    )
    .unwrap();
    assert_eq!(cancelled.provisional["status"], "cancelled");

    let count: i64 = connection
        .query_row(
            "SELECT count(*) FROM sync_outbox WHERE entity_type='appointment'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 4);
}

#[test]
fn invalid_transitions_and_inputs_return_safe_errors_without_effects() {
    let mut connection = database();
    let created = create_appointment(
        &mut connection,
        CreateAppointmentRequest {
            patient_id: PATIENT.into(),
            professional_id: PROFESSIONAL.into(),
            service_id: SERVICE.into(),
            responsible_contact_id: None,
            starts_at: "2026-05-04T13:00:00Z".into(),
            ends_at: "2026-05-04T13:30:00Z".into(),
        },
    )
    .unwrap();
    let appointment_id = created.entity_key.as_str().unwrap().to_owned();

    // Unknown appointment id.
    let err = confirm_appointment(
        &mut connection,
        ConfirmAppointmentRequest {
            appointment_id: "019a1234-0000-7000-8000-000000000099".into(),
            base: base_local(&created.operation_id),
        },
    )
    .unwrap_err();
    assert_eq!(err.code, "NOT_FOUND");

    // Non-canonical appointment id.
    let err = confirm_appointment(
        &mut connection,
        ConfirmAppointmentRequest {
            appointment_id: "not-a-uuid".into(),
            base: base_local(&created.operation_id),
        },
    )
    .unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");

    // Reschedule with a duration change.
    let err = reschedule_appointment(
        &mut connection,
        RescheduleAppointmentRequest {
            appointment_id: appointment_id.clone(),
            base: base_local(&created.operation_id),
            starts_at: "2026-05-04T14:00:00Z".into(),
            ends_at: "2026-05-04T15:00:00Z".into(),
        },
    )
    .unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");

    // Empty reason.
    let err = cancel_appointment(
        &mut connection,
        LifecycleAppointmentRequest {
            appointment_id: appointment_id.clone(),
            base: base_local(&created.operation_id),
            change_reason: Some("  ".into()),
        },
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
    let _ = (
        FixedClock,
        PROFESSIONAL,
        jiff::Timestamp::from_str("2026-05-04T13:00:00Z").unwrap(),
    );
}

#[test]
fn no_show_and_complete_use_the_system_clock_and_account_policy() {
    let mut connection = database();
    let created = create_appointment(
        &mut connection,
        CreateAppointmentRequest {
            patient_id: PATIENT.into(),
            professional_id: PROFESSIONAL.into(),
            service_id: SERVICE.into(),
            responsible_contact_id: None,
            starts_at: "2026-05-04T13:00:00Z".into(),
            ends_at: "2026-05-04T13:30:00Z".into(),
        },
    )
    .unwrap();
    let appointment_id = created.entity_key.as_str().unwrap().to_owned();

    // The seeded policy has a 15-minute grace and the system clock is far
    // past the appointment, so both terminal actions are allowed.
    let no_show = mark_no_show(
        &mut connection,
        LifecycleAppointmentRequest {
            appointment_id: appointment_id.clone(),
            base: base_local(&created.operation_id),
            change_reason: Some("no asistió".into()),
        },
    )
    .unwrap();
    assert_eq!(no_show.provisional["status"], "no_show");
}
