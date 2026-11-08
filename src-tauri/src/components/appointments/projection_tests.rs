use super::super::super::application::ScheduleOccupancyQuery;
use super::super::outbound::SqliteScheduleOccupancyQuery;
use super::*;
use crate::components::sync::remote::{ChangeGroup, ChangeKind, SyncChange, apply_groups};
use crate::shared_kernel::id::OperationContext;

const PROFESSIONAL: &str = "019a1234-0000-7000-8000-000000000008";
const PATIENT: &str = "019a1234-0000-7000-8000-000000000009";
const SERVICE: &str = "019a1234-0000-7000-8000-00000000000a";
const APPOINTMENT: &str = "019a1234-0000-7000-8000-000000000010";

fn scope() -> OperationContext {
    serde_json::from_value(serde_json::json!({
        "accountId":"019a1234-0000-7000-8000-000000000001",
        "userId":"019a1234-0000-7000-8000-000000000002",
        "deviceId":"019a1234-0000-7000-8000-000000000003", "permissionEpoch":1
    }))
    .unwrap()
}

fn database() -> rusqlite::Connection {
    crate::platform::sqlite::open(std::path::Path::new(":memory:")).unwrap()
}

fn appointment(s: &OperationContext, version: i64, status: &str) -> SyncChange {
    SyncChange {
        entity_type: EntityType::Appointment,
        entity_key: serde_json::json!(APPOINTMENT),
        entity_version: version,
        change_kind: ChangeKind::Upsert,
        state: serde_json::json!({
            "id":APPOINTMENT,"accountId":s.account_id,"patientId":PATIENT,
            "professionalId":PROFESSIONAL,"serviceId":SERVICE,
            "startsAt":"2026-05-06T13:30:00Z","endsAt":"2026-05-06T14:00:00Z",
            "status":status,"confirmedAt":null,
            "serviceNameSnapshot":"Consulta","offeredDurationMinutesSnapshot":30,
            "professionalNameSnapshot":"Profesional ficticio",
            "locationNameSnapshot":"Consultorio","locationAddressSnapshot":"Calle 0",
            "locationCountrySnapshot":"VE","locationStateSnapshot":"DC",
            "locationCitySnapshot":"Caracas","timeZoneSnapshot":"America/Caracas",
            "responsibleContactId":null,
            "createdAt":null,"updatedAt":null,"version":version
        }),
    }
}

fn group(changes: Vec<SyncChange>) -> ChangeGroup {
    ChangeGroup {
        group_id: uuid::Uuid::now_v7().to_string().try_into().unwrap(),
        operation_id: uuid::Uuid::now_v7().to_string().try_into().unwrap(),
        accepted_at: "2026-11-08T12:00:00Z".into(),
        changes,
    }
}

fn occupancy(c: &rusqlite::Connection, s: &OperationContext) -> Vec<(String, String, String)> {
    SqliteScheduleOccupancyQuery {
        connection: c,
        scope: s,
    }
    .list_occupancy(
        PROFESSIONAL,
        jiff::Timestamp::from_str("2026-01-01T00:00:00Z").unwrap(),
        jiff::Timestamp::from_str("2027-01-01T00:00:00Z").unwrap(),
    )
    .unwrap()
    .into_iter()
    .map(|interval| {
        (
            interval.appointment_id,
            interval.starts_at.to_string(),
            interval.status,
        )
    })
    .collect()
}

use std::str::FromStr;

#[test]
fn occupying_states_project_and_cancelled_frees_occupancy() {
    let mut c = database();
    let s = scope();

    apply_groups(
        &mut c,
        &s,
        &[group(vec![appointment(&s, 1, "scheduled")])],
        &AppointmentOccupancyProjector,
    )
    .unwrap();
    assert_eq!(
        occupancy(&c, &s),
        [(
            APPOINTMENT.to_string(),
            "2026-05-06T13:30:00Z".to_string(),
            "scheduled".to_string()
        )]
    );

    // Reprogramación (misma versión mayor): la ocupación se mueve.
    apply_groups(
        &mut c,
        &s,
        &[group(vec![appointment(&s, 2, "confirmed")])],
        &AppointmentOccupancyProjector,
    )
    .unwrap();
    assert_eq!(occupancy(&c, &s)[0].2, "confirmed".to_string());

    // cancelada libera la ocupación: la fila desaparece.
    apply_groups(
        &mut c,
        &s,
        &[group(vec![appointment(&s, 3, "cancelled")])],
        &AppointmentOccupancyProjector,
    )
    .unwrap();
    assert!(occupancy(&c, &s).is_empty());
}

#[test]
fn old_versions_do_not_regress_and_remove_retires_only_views() {
    let mut c = database();
    let s = scope();
    apply_groups(
        &mut c,
        &s,
        &[group(vec![appointment(&s, 2, "confirmed")])],
        &AppointmentOccupancyProjector,
    )
    .unwrap();
    apply_groups(
        &mut c,
        &s,
        &[group(vec![appointment(&s, 1, "scheduled")])],
        &AppointmentOccupancyProjector,
    )
    .unwrap();
    assert_eq!(occupancy(&c, &s)[0].2, "confirmed".to_string());

    let mut removed = appointment(&s, 3, "confirmed");
    removed.change_kind = ChangeKind::Remove;
    removed.entity_version = 3;
    removed.state = serde_json::Value::Null;
    apply_groups(
        &mut c,
        &s,
        &[group(vec![removed])],
        &AppointmentOccupancyProjector,
    )
    .unwrap();
    assert!(occupancy(&c, &s).is_empty());
}

#[test]
fn out_of_scope_or_invalid_appointments_are_rejected_without_effects() {
    let mut c = database();
    let s = scope();
    apply_groups(
        &mut c,
        &s,
        &[group(vec![appointment(&s, 1, "scheduled")])],
        &AppointmentOccupancyProjector,
    )
    .unwrap();

    let mut foreign = scope();
    foreign.account_id = "019a1234-0000-7000-8000-0000000000ee"
        .to_owned()
        .try_into()
        .unwrap();
    let err = apply_groups(
        &mut c,
        &foreign,
        &[group(vec![appointment(&s, 1, "scheduled")])],
        &AppointmentOccupancyProjector,
    )
    .unwrap_err();
    assert_eq!(err.code, crate::shared_kernel::ErrorCode::ValidationError);

    let mut invalid = appointment(&s, 2, "scheduled");
    invalid.state["endsAt"] = serde_json::json!("2026-05-06T13:00:00Z");
    let err = apply_groups(
        &mut c,
        &s,
        &[group(vec![invalid])],
        &AppointmentOccupancyProjector,
    )
    .unwrap_err();
    assert!(err.message.contains("intervalo"));

    assert_eq!(occupancy(&c, &s).len(), 1);
}
