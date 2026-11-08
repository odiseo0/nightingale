use crate::components::appointments::adapters::outbound::SqliteScheduleOccupancyQuery;
use crate::components::appointments::adapters::projector::*;
use crate::components::appointments::application::ScheduleOccupancyQuery;
use crate::components::sync::remote::{ChangeGroup, ChangeKind, SyncChange, apply_groups};
use crate::components::sync::remote::{EntityType, RemoteProjector};
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
        &AppointmentProjector,
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
        &AppointmentProjector,
    )
    .unwrap();
    assert_eq!(occupancy(&c, &s)[0].2, "confirmed".to_string());

    // cancelada libera la ocupación: la fila desaparece.
    apply_groups(
        &mut c,
        &s,
        &[group(vec![appointment(&s, 3, "cancelled")])],
        &AppointmentProjector,
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
        &AppointmentProjector,
    )
    .unwrap();
    apply_groups(
        &mut c,
        &s,
        &[group(vec![appointment(&s, 1, "scheduled")])],
        &AppointmentProjector,
    )
    .unwrap();
    assert_eq!(occupancy(&c, &s)[0].2, "confirmed".to_string());

    let mut removed = appointment(&s, 3, "confirmed");
    removed.change_kind = ChangeKind::Remove;
    removed.entity_version = 3;
    removed.state = serde_json::Value::Null;
    apply_groups(&mut c, &s, &[group(vec![removed])], &AppointmentProjector).unwrap();
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
        &AppointmentProjector,
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
        &AppointmentProjector,
    )
    .unwrap_err();
    assert_eq!(err.code, crate::shared_kernel::ErrorCode::ValidationError);

    let mut invalid = appointment(&s, 2, "scheduled");
    invalid.state["endsAt"] = serde_json::json!("2026-05-06T13:00:00Z");
    let err = apply_groups(&mut c, &s, &[group(vec![invalid])], &AppointmentProjector).unwrap_err();
    assert!(err.message.contains("intervalo"));

    assert_eq!(occupancy(&c, &s).len(), 1);
}

// ---- T14: full accepted appointment view ----

fn accepted_view(
    c: &rusqlite::Connection,
    s: &OperationContext,
) -> Vec<(i64, String, String, String, String)> {
    let mut query = c
        .prepare(
            "SELECT version,status,starts_at,ends_at,service_name_snapshot FROM appointment_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 ORDER BY appointment_id",
        )
        .unwrap();
    query
        .query_map(
            rusqlite::params![
                s.account_id.as_str(),
                s.user_id.as_str(),
                s.device_id.as_str()
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            },
        )
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
}

#[test]
fn full_appointment_view_projects_all_versions_and_snapshots() {
    let mut c = database();
    let s = scope();

    apply_groups(
        &mut c,
        &s,
        &[group(vec![appointment(&s, 1, "scheduled")])],
        &AppointmentProjector,
    )
    .unwrap();
    assert_eq!(
        accepted_view(&c, &s),
        [(
            1,
            "scheduled".to_string(),
            "2026-05-06T13:30:00Z".to_string(),
            "2026-05-06T14:00:00Z".to_string(),
            "Consulta".to_string()
        )]
    );

    // A newer version moves the accepted row and keeps the occupancy in sync.
    apply_groups(
        &mut c,
        &s,
        &[group(vec![appointment(&s, 2, "confirmed")])],
        &AppointmentProjector,
    )
    .unwrap();
    assert_eq!(accepted_view(&c, &s)[0].0, 2);
    assert_eq!(accepted_view(&c, &s)[0].1, "confirmed");
    assert_eq!(occupancy(&c, &s)[0].2, "confirmed");

    // An older version does not regress either view.
    apply_groups(
        &mut c,
        &s,
        &[group(vec![appointment(&s, 1, "scheduled")])],
        &AppointmentProjector,
    )
    .unwrap();
    assert_eq!(accepted_view(&c, &s)[0].0, 2);
    assert_eq!(accepted_view(&c, &s)[0].1, "confirmed");

    // cancelled frees occupancy but the accepted row keeps the history.
    apply_groups(
        &mut c,
        &s,
        &[group(vec![appointment(&s, 3, "cancelled")])],
        &AppointmentProjector,
    )
    .unwrap();
    assert_eq!(accepted_view(&c, &s)[0].1, "cancelled");
    assert!(occupancy(&c, &s).is_empty());
}

#[test]
fn remove_and_reset_retire_appointment_views_but_never_intentions() {
    let mut c = database();
    let s = scope();
    apply_groups(
        &mut c,
        &s,
        &[group(vec![appointment(&s, 1, "scheduled")])],
        &AppointmentProjector,
    )
    .unwrap();

    // A local intention survives remove/reset of accepted views.
    c.execute(
        "INSERT INTO appointment_local (account_id,user_id,device_id,operation_id,appointment_id,action,expected_version,base_operation_id,depends_on,payload,draft,created_at) VALUES (?1,?2,?3,'019a1234-0000-7000-8000-000000000030',?4,'CreateAppointment',0,NULL,'[]','{}','{}',1)",
        rusqlite::params![s.account_id.as_str(), s.user_id.as_str(), s.device_id.as_str(), APPOINTMENT],
    )
    .unwrap();

    // remove retires both accepted views of the appointment.
    let removal = SyncChange {
        entity_type: EntityType::Appointment,
        entity_key: serde_json::json!(APPOINTMENT),
        entity_version: 3,
        change_kind: ChangeKind::Remove,
        state: serde_json::Value::Null,
    };
    apply_groups(&mut c, &s, &[group(vec![removal])], &AppointmentProjector).unwrap();
    assert!(accepted_view(&c, &s).is_empty());
    assert!(occupancy(&c, &s).is_empty());

    // Re-project and reset clears the scope's views.
    apply_groups(
        &mut c,
        &s,
        &[group(vec![appointment(&s, 1, "scheduled")])],
        &AppointmentProjector,
    )
    .unwrap();
    {
        let tx = c.transaction().unwrap();
        AppointmentProjector.reset(&tx, &s).unwrap();
        tx.commit().unwrap();
    }
    assert!(accepted_view(&c, &s).is_empty());
    assert!(occupancy(&c, &s).is_empty());

    let intentions: i64 = c
        .query_row("SELECT count(*) FROM appointment_local", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(intentions, 1);
}

#[test]
fn invalid_appointments_leave_no_appointment_view_effects() {
    let mut c = database();
    let s = scope();

    // Out-of-scope account id rejects the group before any write.
    let mut foreign = appointment(&s, 1, "scheduled");
    foreign.state = serde_json::json!({
        "id":APPOINTMENT,"accountId":"019a1234-0000-7000-8000-0000000000aa","patientId":PATIENT,
        "professionalId":PROFESSIONAL,"serviceId":SERVICE,
        "startsAt":"2026-05-06T13:30:00Z","endsAt":"2026-05-06T14:00:00Z",
        "status":"scheduled","confirmedAt":null,
        "serviceNameSnapshot":"Consulta","offeredDurationMinutesSnapshot":30,
        "professionalNameSnapshot":"Profesional ficticio",
        "locationNameSnapshot":"Consultorio","locationAddressSnapshot":"Calle 0",
        "locationCountrySnapshot":"VE","locationStateSnapshot":"DC",
        "locationCitySnapshot":"Caracas","timeZoneSnapshot":"America/Caracas",
        "responsibleContactId":null,
        "createdAt":null,"updatedAt":null,"version":1
    });
    assert!(apply_groups(&mut c, &s, &[group(vec![foreign])], &AppointmentProjector).is_err());
    assert!(accepted_view(&c, &s).is_empty());

    // Reversed interval rejects the group as well.
    let mut reversed = appointment(&s, 1, "scheduled");
    reversed.state = serde_json::json!({
        "id":APPOINTMENT,"accountId":s.account_id,"patientId":PATIENT,
        "professionalId":PROFESSIONAL,"serviceId":SERVICE,
        "startsAt":"2026-05-06T14:00:00Z","endsAt":"2026-05-06T13:30:00Z",
        "status":"scheduled","confirmedAt":null,
        "serviceNameSnapshot":"Consulta","offeredDurationMinutesSnapshot":30,
        "professionalNameSnapshot":"Profesional ficticio",
        "locationNameSnapshot":"Consultorio","locationAddressSnapshot":"Calle 0",
        "locationCountrySnapshot":"VE","locationStateSnapshot":"DC",
        "locationCitySnapshot":"Caracas","timeZoneSnapshot":"America/Caracas",
        "responsibleContactId":null,
        "createdAt":null,"updatedAt":null,"version":1
    });
    assert!(apply_groups(&mut c, &s, &[group(vec![reversed])], &AppointmentProjector).is_err());
    assert!(accepted_view(&c, &s).is_empty());
    assert!(occupancy(&c, &s).is_empty());
}
