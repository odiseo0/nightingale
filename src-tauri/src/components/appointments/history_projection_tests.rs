//! Aggregated appointment history projection: consecutive entries from 1,
//! immutable accepted sets, newer aggregates only.
use super::AppointmentProjector;
use crate::components::sync::remote::RemoteProjector as _;
use crate::components::sync::remote::{
    ChangeGroup, ChangeKind, EntityType, SyncChange, apply_groups,
};
use crate::shared_kernel::id::OperationContext;
use rusqlite::params;

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

fn entry(s: &OperationContext, version: i64, action: &str, status: &str) -> serde_json::Value {
    serde_json::json!({
        "appointmentId":APPOINTMENT,"accountId":s.account_id,"version":version,
        "action":action,"actorKind":"staff","actorUserId":s.user_id,"actorContactPersonId":null,
        "changeReason":null,"changedAt":"2026-05-06T13:00:00Z",
        "startsAt":"2026-05-06T13:30:00Z","endsAt":"2026-05-06T14:00:00Z",
        "status":status,"confirmedAt":null,
        "serviceNameSnapshot":"Consulta","offeredDurationMinutesSnapshot":30,
        "professionalNameSnapshot":"Profesional ficticio",
        "locationNameSnapshot":"Consultorio","locationAddressSnapshot":"Calle 0",
        "locationCountrySnapshot":"VE","locationStateSnapshot":"DC",
        "locationCitySnapshot":"Caracas","timeZoneSnapshot":"America/Caracas",
        "responsibleContactId":null
    })
}

fn history(s: &OperationContext, version: i64, actions: &[(&str, &str)]) -> SyncChange {
    SyncChange {
        entity_type: EntityType::AppointmentHistory,
        entity_key: serde_json::json!(APPOINTMENT),
        entity_version: version,
        change_kind: ChangeKind::Upsert,
        state: serde_json::json!({
            "appointmentId":APPOINTMENT,"accountId":s.account_id,
            "version":version,
            "startsAt":"2026-05-06T13:30:00Z","endsAt":"2026-05-06T14:00:00Z",
            "items": actions.iter().enumerate().map(|(index,(action,status))| {
                entry(s, (index as i64)+1, action, status)
            }).collect::<Vec<_>>()
        }),
    }
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

fn history_rows(
    c: &rusqlite::Connection,
    s: &OperationContext,
) -> Vec<(i64, String, String, String)> {
    let mut query = c
        .prepare(
            "SELECT version,action,status,service_name_snapshot FROM appointment_history_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND appointment_id=?4 ORDER BY version",
        )
        .unwrap();
    query
        .query_map(
            params![
                s.account_id.as_str(),
                s.user_id.as_str(),
                s.device_id.as_str(),
                APPOINTMENT
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap()
}

#[test]
fn consecutive_aggregates_project_and_older_ones_do_not_rewrite() {
    let mut c = database();
    let s = scope();

    apply_groups(
        &mut c,
        &s,
        &[group(vec![history(
            &s,
            2,
            &[("created", "scheduled"), ("confirmed", "confirmed")],
        )])],
        &AppointmentProjector,
    )
    .unwrap();
    assert_eq!(
        history_rows(&c, &s),
        [
            (
                1,
                "created".to_string(),
                "scheduled".to_string(),
                "Consulta".to_string()
            ),
            (
                2,
                "confirmed".to_string(),
                "confirmed".to_string(),
                "Consulta".to_string()
            ),
        ]
    );

    // A newer aggregate replaces the accepted set entirely.
    apply_groups(
        &mut c,
        &s,
        &[group(vec![history(
            &s,
            3,
            &[
                ("created", "scheduled"),
                ("confirmed", "confirmed"),
                ("rescheduled", "scheduled"),
            ],
        )])],
        &AppointmentProjector,
    )
    .unwrap();
    let rows = history_rows(&c, &s);
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[2].1, "rescheduled");

    // An older or repeated aggregate never rewrites accepted history.
    apply_groups(
        &mut c,
        &s,
        &[group(vec![history(
            &s,
            2,
            &[("created", "scheduled"), ("confirmed", "confirmed")],
        )])],
        &AppointmentProjector,
    )
    .unwrap();
    assert_eq!(history_rows(&c, &s).len(), 3);
    assert_eq!(history_rows(&c, &s)[2].1, "rescheduled");
}

#[test]
fn gapped_duplicated_or_inconsistent_aggregates_are_rejected() {
    let mut c = database();
    let s = scope();

    // Gap: versions 1 and 3.
    let mut gapped = history(
        &s,
        3,
        &[("created", "scheduled"), ("confirmed", "confirmed")],
    );
    gapped.state = serde_json::json!({
        "appointmentId":APPOINTMENT,"accountId":s.account_id,"version":3,
        "startsAt":"2026-05-06T13:30:00Z","endsAt":"2026-05-06T14:00:00Z",
        "items":[
            entry(&s, 1, "created", "scheduled"),
            entry(&s, 3, "confirmed", "confirmed")
        ]
    });
    assert!(apply_groups(&mut c, &s, &[group(vec![gapped])], &AppointmentProjector).is_err());
    assert!(history_rows(&c, &s).is_empty());

    // Duplicate version.
    let mut duplicated = history(
        &s,
        2,
        &[("created", "scheduled"), ("confirmed", "confirmed")],
    );
    duplicated.state = serde_json::json!({
        "appointmentId":APPOINTMENT,"accountId":s.account_id,"version":2,
        "startsAt":"2026-05-06T13:30:00Z","endsAt":"2026-05-06T14:00:00Z",
        "items":[entry(&s, 1, "created", "scheduled"), entry(&s, 1, "confirmed", "confirmed")]
    });
    assert!(
        apply_groups(
            &mut c,
            &s,
            &[group(vec![duplicated])],
            &AppointmentProjector
        )
        .is_err()
    );

    // Aggregate version not matching its last entry.
    let mut mismatch = history(
        &s,
        2,
        &[("created", "scheduled"), ("confirmed", "confirmed")],
    );
    mismatch.entity_version = 3;
    assert!(apply_groups(&mut c, &s, &[group(vec![mismatch])], &AppointmentProjector).is_err());

    // Out-of-scope account.
    let mut foreign = history(&s, 1, &[("created", "scheduled")]);
    foreign.state = serde_json::json!({
        "appointmentId":APPOINTMENT,"accountId":"019a1234-0000-7000-8000-0000000000aa","version":1,
        "startsAt":"2026-05-06T13:30:00Z","endsAt":"2026-05-06T14:00:00Z",
        "items":[entry(&s, 1, "created", "scheduled")]
    });
    assert!(apply_groups(&mut c, &s, &[group(vec![foreign])], &AppointmentProjector).is_err());
    assert!(history_rows(&c, &s).is_empty());
}

#[test]
fn history_and_appointment_together_stay_coherent_and_separate_arrival_waits() {
    let mut c = database();
    let s = scope();

    // Together in one group: last entry matches the accepted appointment.
    apply_groups(
        &mut c,
        &s,
        &[group(vec![
            appointment(&s, 1, "scheduled"),
            history(&s, 1, &[("created", "scheduled")]),
        ])],
        &AppointmentProjector,
    )
    .unwrap();
    assert_eq!(history_rows(&c, &s).len(), 1);

    // History arriving before the appointment change is legal: nothing is
    // invented and the aggregate is kept as accepted history.
    apply_groups(
        &mut c,
        &s,
        &[group(vec![history(
            &s,
            2,
            &[("created", "scheduled"), ("cancelled", "cancelled")],
        )])],
        &AppointmentProjector,
    )
    .unwrap();
    assert_eq!(history_rows(&c, &s).len(), 2);

    // But in the same group, a last entry that contradicts the appointment
    // state is rejected before any write.
    let mut c2 = database();
    assert!(
        apply_groups(
            &mut c2,
            &s,
            &[group(vec![
                appointment(&s, 2, "scheduled"),
                history(
                    &s,
                    2,
                    &[("created", "scheduled"), ("cancelled", "cancelled")]
                ),
            ])],
            &AppointmentProjector
        )
        .is_err()
    );
    assert!(history_rows(&c2, &s).is_empty());

    // When the appointment then reaches the newer version, the earlier
    // history is already coherent with it.
    apply_groups(
        &mut c,
        &s,
        &[group(vec![appointment(&s, 2, "cancelled")])],
        &AppointmentProjector,
    )
    .unwrap();
    apply_groups(
        &mut c,
        &s,
        &[group(vec![history(
            &s,
            2,
            &[("created", "scheduled"), ("cancelled", "cancelled")],
        )])],
        &AppointmentProjector,
    )
    .unwrap();
    let rows = history_rows(&c, &s);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1].2, "cancelled");

    // remove of the appointment retires its accepted history too.
    let removal = SyncChange {
        entity_type: EntityType::Appointment,
        entity_key: serde_json::json!(APPOINTMENT),
        entity_version: 3,
        change_kind: ChangeKind::Remove,
        state: serde_json::Value::Null,
    };
    apply_groups(&mut c, &s, &[group(vec![removal])], &AppointmentProjector).unwrap();
    assert!(history_rows(&c, &s).is_empty());
}

#[test]
fn reset_clears_accepted_history_and_keeps_local_intentions() {
    let mut c = database();
    let s = scope();
    apply_groups(
        &mut c,
        &s,
        &[group(vec![history(&s, 1, &[("created", "scheduled")])])],
        &AppointmentProjector,
    )
    .unwrap();
    c.execute(
        "INSERT INTO appointment_local (account_id,user_id,device_id,operation_id,appointment_id,action,expected_version,base_operation_id,depends_on,payload,draft,created_at) VALUES (?1,?2,?3,'019a1234-0000-7000-8000-000000000030',?4,'CreateAppointment',0,NULL,'[]','{}','{}',1)",
        params![s.account_id.as_str(), s.user_id.as_str(), s.device_id.as_str(), APPOINTMENT],
    )
    .unwrap();

    {
        let tx = c.transaction().unwrap();
        AppointmentProjector.reset(&tx, &s).unwrap();
        tx.commit().unwrap();
    }
    assert!(history_rows(&c, &s).is_empty());
    let intentions: i64 = c
        .query_row("SELECT count(*) FROM appointment_local", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(intentions, 1);
}
