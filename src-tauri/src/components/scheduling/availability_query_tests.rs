use super::{
    application::{
        AvailabilityRequest, EditBase, ScheduleAvailabilityQuery, SchedulingMutation,
        SchedulingRepository,
    },
    domain::Status,
    test_support::{
        FakeOccupancy, FixedClock, Ids, NoOccupancy, PROFESSIONAL, access, database, repository,
    },
};
use crate::components::appointments::application::OccupancyInterval;
use crate::components::sync::application::CursorStore;
use crate::components::sync::domain::StoredCursor;
use crate::shared_kernel::Result as KernelResult;
use crate::shared_kernel::id::OperationContext;
use jiff::Timestamp;
use rusqlite::{Connection, Transaction};
use serde_json::json;
use std::str::FromStr;

const SERVICE: &str = "019a1234-0000-7000-8000-0000000000b1";

fn ts(value: &str) -> Timestamp {
    Timestamp::from_str(value).unwrap()
}

struct FakeCursors(Option<StoredCursor>);

impl CursorStore for FakeCursors {
    fn save_cursor(
        &self,
        _tx: &Transaction,
        _scope: &OperationContext,
        _cursor: &StoredCursor,
    ) -> KernelResult<()> {
        Ok(())
    }

    fn cursor(
        &self,
        _connection: &Connection,
        _scope: &OperationContext,
    ) -> KernelResult<Option<StoredCursor>> {
        Ok(self.0.clone())
    }
}

fn windowed() -> FakeCursors {
    FakeCursors(Some(StoredCursor {
        cursor: "cursor-1".into(),
        window: json!({
            "starts_at": "2026-04-01T00:00:00Z",
            "ends_at": "2026-04-30T00:00:00Z",
            "appointments_from": "2026-05-01T00:00:00Z",
            "appointments_until": "2026-05-31T00:00:00Z",
            "time_zone": "America/Caracas"
        }),
    }))
}

fn seeded() -> Connection {
    let mut c = database();
    let tx = c.transaction().unwrap();
    tx.execute(
        "INSERT INTO professional_view (account_id,user_id,device_id,professional_id,professional_user_id,public_name,status,version) VALUES ('019a1234-0000-7000-8000-000000000001','019a1234-0000-7000-8000-000000000002','019a1234-0000-7000-8000-000000000003',?1,'019a1234-0000-7000-8000-000000000002','Profesional ficticio','active',1)",
        rusqlite::params![PROFESSIONAL],
    )
    .unwrap();
    tx.execute(
        "INSERT INTO service_offer_view (account_id,user_id,device_id,professional_id,service_id,name,default_duration_minutes,duration_minutes_override,effective_duration_minutes,service_status,status,version,service_version) VALUES ('019a1234-0000-7000-8000-000000000001','019a1234-0000-7000-8000-000000000002','019a1234-0000-7000-8000-000000000003',?1,?2,'Consulta',30,NULL,30,'active','active',2,7)",
        rusqlite::params![PROFESSIONAL, SERVICE],
    )
    .unwrap();
    tx.commit().unwrap();
    c
}

fn request(duration: Option<i64>) -> AvailabilityRequest {
    AvailabilityRequest {
        professional_id: PROFESSIONAL.into(),
        service_id: SERVICE.into(),
        from: ts("2026-05-06T04:00:00Z"),
        until: ts("2026-05-07T04:00:00Z"),
        duration_minutes: duration,
        exclude_appointment_id: None,
    }
}

fn add_wednesday_rule(c: &mut Connection) -> String {
    let ids = Ids::default();
    repository(c)
        .persist(
            &access(),
            SchedulingMutation::CreateRule {
                value: super::domain::RuleValue::new(
                    PROFESSIONAL.to_owned().try_into().unwrap(),
                    3,
                    480,
                    720,
                    super::domain::parse_local_date("2026-01-01").unwrap(),
                    None,
                )
                .unwrap(),
            },
            &ids,
            &FixedClock,
            &NoOccupancy,
        )
        .unwrap()
        .entity_key
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn within_window_reports_versions_intervals_and_provisional() {
    let mut c = seeded();
    let rule_id = add_wednesday_rule(&mut c);

    let evaluation = repository(&mut c)
        .local_availability(&access(), &request(None), &NoOccupancy, &windowed())
        .unwrap();

    assert_eq!(evaluation.time_zone, "America/Caracas");
    assert_eq!(evaluation.reference_duration_minutes, 30);
    assert_eq!(evaluation.requested_duration_minutes, 30);
    assert_eq!(evaluation.account_version, 1);
    assert_eq!(evaluation.offer_version, 2);
    assert_eq!(evaluation.catalog_version, 7);
    assert_eq!(evaluation.agenda_versions, [(rule_id, 1)]);
    assert_eq!(evaluation.origin, "local");
    assert!(evaluation.provisional);
    assert_eq!(evaluation.coverage, "within_synced_window");
    // 08:00-12:00 Caracas (UTC-4) = 12:00-16:00Z.
    assert_eq!(evaluation.intervals.len(), 1);
    assert_eq!(evaluation.intervals[0].starts_at, "2026-05-06T12:00:00Z");
    assert_eq!(evaluation.intervals[0].ends_at, "2026-05-06T16:00:00Z");
}

#[test]
fn no_synced_window_is_local_only_but_still_explicit() {
    let mut c = seeded();
    add_wednesday_rule(&mut c);

    let evaluation = repository(&mut c)
        .local_availability(&access(), &request(None), &NoOccupancy, &FakeCursors(None))
        .unwrap();
    assert_eq!(evaluation.coverage, "no_synced_window");
    assert!(evaluation.provisional);
    assert_eq!(evaluation.intervals.len(), 1);
}

#[test]
fn outside_window_reports_insufficient_coverage() {
    let mut c = seeded();
    add_wednesday_rule(&mut c);

    let mut late = request(None);
    late.from = ts("2026-06-10T04:00:00Z");
    late.until = ts("2026-06-11T04:00:00Z");
    let evaluation = repository(&mut c)
        .local_availability(&access(), &late, &NoOccupancy, &windowed())
        .unwrap();
    assert_eq!(evaluation.coverage, "insufficient_coverage");

    // Parcialmente fuera: también insuficiente.
    let mut partial = request(None);
    partial.until = ts("2026-06-01T04:00:00Z");
    let evaluation = repository(&mut c)
        .local_availability(&access(), &partial, &NoOccupancy, &windowed())
        .unwrap();
    assert_eq!(evaluation.coverage, "insufficient_coverage");
}

#[test]
fn occupancy_subtracts_and_duration_override_filters() {
    let mut c = seeded();
    add_wednesday_rule(&mut c);
    let occupancy = FakeOccupancy(vec![(
        PROFESSIONAL.into(),
        OccupancyInterval::new(
            "019a1234-0000-7000-8000-0000000000c1".into(),
            ts("2026-05-06T12:30:00Z"),
            ts("2026-05-06T13:00:00Z"),
            "scheduled",
        )
        .unwrap(),
    )]);

    let evaluation = repository(&mut c)
        .local_availability(&access(), &request(None), &occupancy, &windowed())
        .unwrap();
    assert_eq!(evaluation.intervals.len(), 2);
    assert_eq!(evaluation.intervals[0].starts_at, "2026-05-06T12:00:00Z");
    assert_eq!(evaluation.intervals[0].ends_at, "2026-05-06T12:30:00Z");
    assert_eq!(evaluation.intervals[1].starts_at, "2026-05-06T13:00:00Z");
    assert_eq!(evaluation.intervals[1].ends_at, "2026-05-06T16:00:00Z");

    // Duración 240: solo el hueco de 12:00-12:30 (30 min) queda descartado → vacío.
    let evaluation = repository(&mut c)
        .local_availability(&access(), &request(Some(240)), &occupancy, &windowed())
        .unwrap();
    assert!(evaluation.intervals.is_empty());
    assert_eq!(evaluation.requested_duration_minutes, 240);
}

#[test]
fn invalid_range_duration_and_missing_offer_are_rejected() {
    let mut c = seeded();
    add_wednesday_rule(&mut c);

    let mut too_long = request(None);
    too_long.until = ts("2026-06-10T04:00:00Z");
    let err = repository(&mut c)
        .local_availability(&access(), &too_long, &NoOccupancy, &windowed())
        .unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");

    let err = repository(&mut c)
        .local_availability(&access(), &request(Some(0)), &NoOccupancy, &windowed())
        .unwrap_err();
    assert_eq!(err.code, "VALIDATION_ERROR");

    let mut missing = request(None);
    missing.service_id = "019a1234-0000-7000-8000-0000000000ff".into();
    let err = repository(&mut c)
        .local_availability(&access(), &missing, &NoOccupancy, &windowed())
        .unwrap_err();
    assert_eq!(err.code, "NOT_FOUND");

    // Oferta inactiva: no disponible.
    c.execute(
        "UPDATE service_offer_view SET status = 'inactive' WHERE service_id = ?1",
        rusqlite::params![SERVICE],
    )
    .unwrap();
    let err = repository(&mut c)
        .local_availability(&access(), &request(None), &NoOccupancy, &windowed())
        .unwrap_err();
    assert_eq!(err.code, "NOT_FOUND");
}

#[test]
fn rule_change_updates_the_agenda_versions_used() {
    let mut c = seeded();
    let rule_id = add_wednesday_rule(&mut c);

    let evaluation = repository(&mut c)
        .local_availability(&access(), &request(None), &NoOccupancy, &windowed())
        .unwrap();
    assert_eq!(evaluation.agenda_versions, [(rule_id.clone(), 1)]);

    let rows = repository(&mut c).rule_rows_list(&access()).unwrap();
    let provisional = rows[0].provisional.as_ref().unwrap();
    repository(&mut c)
        .persist(
            &access(),
            SchedulingMutation::SetRuleStatus {
                rule_id,
                base: EditBase::LocalOperation {
                    operation_id: crate::shared_kernel::id::OperationId::try_from(
                        provisional.operation_id.clone(),
                    )
                    .unwrap(),
                },
                status: Status::Inactive,
            },
            &Ids::default(),
            &FixedClock,
            &NoOccupancy,
        )
        .unwrap();

    let evaluation = repository(&mut c)
        .local_availability(&access(), &request(None), &NoOccupancy, &windowed())
        .unwrap();
    assert!(evaluation.intervals.is_empty());
    assert_eq!(evaluation.agenda_versions[0].1, 2);
}

const APPOINTMENT_A: &str = "019a1234-0000-7000-8000-0000000000c1";
const APPOINTMENT_B: &str = "019a1234-0000-7000-8000-0000000000c2";

fn two_appointments() -> FakeOccupancy {
    FakeOccupancy(vec![
        (
            PROFESSIONAL.into(),
            OccupancyInterval::new(
                APPOINTMENT_A.into(),
                ts("2026-05-06T13:00:00Z"),
                ts("2026-05-06T13:30:00Z"),
                "confirmed",
            )
            .unwrap(),
        ),
        (
            PROFESSIONAL.into(),
            OccupancyInterval::new(
                APPOINTMENT_B.into(),
                ts("2026-05-06T14:00:00Z"),
                ts("2026-05-06T14:30:00Z"),
                "scheduled",
            )
            .unwrap(),
        ),
    ])
}

#[test]
fn rescheduling_excludes_own_appointment_but_keeps_other_occupancy() {
    let mut c = seeded();
    add_wednesday_rule(&mut c);

    // Sin exclusión: 12:00-13:00, 13:30-14:00 y 14:30-16:00Z.
    let evaluation = repository(&mut c)
        .local_availability(&access(), &request(None), &two_appointments(), &windowed())
        .unwrap();
    assert_eq!(evaluation.intervals.len(), 3);

    // Excluir la propia cita A devuelve su hueco: 12:00-14:00 y 14:30-16:00Z.
    let mut excluded = request(None);
    excluded.exclude_appointment_id = Some(APPOINTMENT_A.into());
    let evaluation = repository(&mut c)
        .local_availability(&access(), &excluded, &two_appointments(), &windowed())
        .unwrap();
    assert_eq!(evaluation.intervals.len(), 2);
    assert_eq!(evaluation.intervals[0].starts_at, "2026-05-06T12:00:00Z");
    assert_eq!(evaluation.intervals[0].ends_at, "2026-05-06T14:00:00Z");
    // La cita B sigue ocupando: el hueco 14:00-14:30Z no reaparece.
    assert_eq!(evaluation.intervals[1].starts_at, "2026-05-06T14:30:00Z");
    assert_eq!(evaluation.intervals[1].ends_at, "2026-05-06T16:00:00Z");

    // Excluir una cita desconocida no cambia nada.
    let mut unknown = request(None);
    unknown.exclude_appointment_id = Some(APPOINTMENT_B.into());
    unknown.exclude_appointment_id = Some("019a1234-0000-7000-8000-0000000000cf".into());
    let evaluation = repository(&mut c)
        .local_availability(&access(), &unknown, &two_appointments(), &windowed())
        .unwrap();
    assert_eq!(evaluation.intervals.len(), 3);

    // Un id de exclusión no canónico se rechaza.
    let mut invalid = request(None);
    invalid.exclude_appointment_id = Some("not-a-uuid".into());
    assert!(
        repository(&mut c)
            .local_availability(&access(), &invalid, &two_appointments(), &windowed())
            .is_err()
    );
}
