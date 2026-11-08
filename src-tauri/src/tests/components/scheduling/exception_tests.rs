use crate::components::appointments::application::OccupancyInterval;
use crate::components::scheduling::{
    application::{EditBase, SchedulingMutation, SchedulingRepository},
    domain::{ExceptionKind, ExceptionPatch, ExceptionValue, Status},
};
use crate::tests::components::scheduling::test_support::{
    FakeOccupancy, FixedClock, Ids, NoOccupancy, PROFESSIONAL, access, database, repository,
};
use jiff::Timestamp;
use serde_json::{Value, json};
use std::str::FromStr;

fn op(id: &str) -> crate::shared_kernel::id::OperationId {
    crate::shared_kernel::id::OperationId::try_from(id.to_owned()).unwrap()
}

/// Cita conocida el miércoles 2026-05-06 de 09:30 a 10:00 en Caracas.
fn wednesday_appointment() -> (String, OccupancyInterval) {
    (
        PROFESSIONAL.into(),
        OccupancyInterval::new(
            "019a1234-0000-7000-8000-000000000020".into(),
            Timestamp::from_str("2026-05-06T13:30:00Z").unwrap(),
            Timestamp::from_str("2026-05-06T14:00:00Z").unwrap(),
            "scheduled",
        )
        .unwrap(),
    )
}

/// Regla que cubre los miércoles completos.
fn wednesday_rule() -> crate::components::scheduling::domain::RuleValue {
    crate::components::scheduling::domain::RuleValue::new(
        PROFESSIONAL.to_owned().try_into().unwrap(),
        3,
        0,
        1440,
        crate::components::scheduling::domain::parse_local_date("2026-01-01").unwrap(),
        None,
    )
    .unwrap()
}

fn closed_exception(date: &str) -> ExceptionValue {
    ExceptionValue::new(
        PROFESSIONAL.to_owned().try_into().unwrap(),
        crate::components::scheduling::domain::parse_local_date(date).unwrap(),
        ExceptionKind::Closed,
        None,
        None,
    )
    .unwrap()
}

fn replacement_exception(date: &str, start: i64, end: i64) -> ExceptionValue {
    ExceptionValue::new(
        PROFESSIONAL.to_owned().try_into().unwrap(),
        crate::components::scheduling::domain::parse_local_date(date).unwrap(),
        ExceptionKind::Replacement,
        Some(start),
        Some(end),
    )
    .unwrap()
}

#[test]
fn closed_and_replacement_follow_contract_payloads_and_survive_reopen() {
    let dir = std::env::temp_dir().join(format!(
        "nightingale-scheduling-t07-{}.sqlite",
        uuid::Uuid::now_v7()
    ));
    let (closed_id, replacement_id) = {
        let mut c = crate::platform::sqlite::open(&dir).unwrap();
        crate::tests::components::scheduling::test_support::seed_account(&mut c);
        let ids = Ids::default();

        let closed = repository(&mut c)
            .persist(
                &access(),
                SchedulingMutation::CreateException {
                    value: closed_exception("2026-05-07"),
                },
                &ids,
                &FixedClock,
                &NoOccupancy,
            )
            .unwrap();
        let replacement = repository(&mut c)
            .persist(
                &access(),
                SchedulingMutation::CreateException {
                    value: replacement_exception("2026-05-08", 480, 720),
                },
                &ids,
                &FixedClock,
                &NoOccupancy,
            )
            .unwrap();
        (
            closed.entity_key.as_str().unwrap().to_owned(),
            replacement.entity_key.as_str().unwrap().to_owned(),
        )
    };

    assert_eq!(
        outbox_payload_of(&dir, "CreateScheduleException", 0),
        json!({
            "professionalId": PROFESSIONAL,
            "localDate": "2026-05-07",
            "kind": "closed"
        })
    );

    // Reapertura: ambas intenciones y provisionales persisten.
    let mut c = crate::platform::sqlite::open(&dir).unwrap();
    let rows = repository(&mut c).exception_rows_list(&access()).unwrap();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|row| row.operations.len() == 1));
    let visible: Vec<String> = rows
        .iter()
        .map(|row| row.visible().unwrap().exception_id.clone())
        .collect();
    assert!(visible.contains(&closed_id) && visible.contains(&replacement_id));
    drop(c);
    std::fs::remove_file(&dir).unwrap();
}

fn outbox_payload_of(dir: &std::path::Path, action: &str, row: usize) -> Value {
    let c = rusqlite::Connection::open(dir).unwrap();
    let payloads: Vec<String> = c
        .prepare(
            "SELECT payload FROM sync_outbox WHERE action = ?1 ORDER BY created_at,operation_id",
        )
        .unwrap()
        .query_map([action], |r| r.get(0))
        .unwrap()
        .collect::<rusqlite::Result<_>>()
        .unwrap();
    serde_json::from_str(&payloads[row]).unwrap()
}

#[test]
fn duplicate_active_exception_for_date_is_rejected() {
    let mut c = database();
    let ids = Ids::default();

    repository(&mut c)
        .persist(
            &access(),
            SchedulingMutation::CreateException {
                value: replacement_exception("2026-05-08", 480, 720),
            },
            &ids,
            &FixedClock,
            &NoOccupancy,
        )
        .unwrap();

    let duplicate = repository(&mut c).persist(
        &access(),
        SchedulingMutation::CreateException {
            value: closed_exception("2026-05-08"),
        },
        &ids,
        &FixedClock,
        &NoOccupancy,
    );
    assert_eq!(duplicate.unwrap_err().code, "VALIDATION_ERROR");

    // Fecha distinta: permitida.
    let other = repository(&mut c)
        .persist(
            &access(),
            SchedulingMutation::CreateException {
                value: closed_exception("2026-05-09"),
            },
            &ids,
            &FixedClock,
            &NoOccupancy,
        )
        .unwrap();
    assert_eq!(other.entity_type, "schedule_exception");
}

#[test]
fn exception_replaces_daily_schedule_for_known_occupancy_checks() {
    let mut c = database();
    let ids = Ids::default();
    let occupancy = FakeOccupancy(vec![wednesday_appointment()]);

    // Regla que cubre el miércoles: aceptada con la cita conocida.
    repository(&mut c)
        .persist(
            &access(),
            SchedulingMutation::CreateRule {
                value: wednesday_rule(),
            },
            &ids,
            &FixedClock,
            &occupancy,
        )
        .unwrap();

    // Cerrar el miércoles de la cita: la cita queda fuera.
    let closed = repository(&mut c).persist(
        &access(),
        SchedulingMutation::CreateException {
            value: closed_exception("2026-05-06"),
        },
        &ids,
        &FixedClock,
        &occupancy,
    );
    assert_eq!(closed.unwrap_err().code, "SLOT_UNAVAILABLE");

    // Reemplazo que no alcanza la cita (10:00 fin vs cita hasta 10:00): ok si 10:00<=fin? cita 09:30-10:00 → fin 600.
    let replacement = repository(&mut c)
        .persist(
            &access(),
            SchedulingMutation::CreateException {
                value: replacement_exception("2026-05-06", 480, 600),
            },
            &ids,
            &FixedClock,
            &occupancy,
        )
        .unwrap();
    let exception_id = replacement.entity_key.as_str().unwrap().to_owned();

    // Reducir el reemplazo para terminar antes de la cita: rechazada.
    let shrunk = repository(&mut c).persist(
        &access(),
        SchedulingMutation::UpdateException {
            exception_id: exception_id.clone(),
            base: EditBase::LocalOperation {
                operation_id: op(&replacement.operation_id),
            },
            patch: ExceptionPatch::new(None, None, Some(570)).unwrap(),
        },
        &ids,
        &FixedClock,
        &occupancy,
    );
    assert_eq!(shrunk.unwrap_err().code, "SLOT_UNAVAILABLE");

    // Retirar la excepción con la cita conocida: la regla del miércoles sigue cubriendo, permitido.
    repository(&mut c)
        .persist(
            &access(),
            SchedulingMutation::SetExceptionStatus {
                exception_id,
                base: EditBase::LocalOperation {
                    operation_id: op(&replacement.operation_id),
                },
                status: Status::Inactive,
            },
            &ids,
            &FixedClock,
            &occupancy,
        )
        .unwrap();
}

#[test]
fn exception_update_revalidates_resulting_kind_and_no_changes() {
    let mut c = database();
    let ids = Ids::default();

    let created = repository(&mut c)
        .persist(
            &access(),
            SchedulingMutation::CreateException {
                value: replacement_exception("2026-05-08", 480, 720),
            },
            &ids,
            &FixedClock,
            &NoOccupancy,
        )
        .unwrap();
    let exception_id = created.entity_key.as_str().unwrap().to_owned();

    // Pasar a closed conservando minutos: estado resultante inválido.
    let invalid = repository(&mut c).persist(
        &access(),
        SchedulingMutation::UpdateException {
            exception_id: exception_id.clone(),
            base: EditBase::LocalOperation {
                operation_id: op(&created.operation_id),
            },
            patch: ExceptionPatch::new(Some(ExceptionKind::Closed), None, None).unwrap(),
        },
        &ids,
        &FixedClock,
        &NoOccupancy,
    );
    assert_eq!(invalid.unwrap_err().code, "VALIDATION_ERROR");

    // Cambio de estado sin diferencia: NO_CHANGES.
    let same = repository(&mut c).persist(
        &access(),
        SchedulingMutation::SetExceptionStatus {
            exception_id,
            base: EditBase::LocalOperation {
                operation_id: op(&created.operation_id),
            },
            status: Status::Active,
        },
        &ids,
        &FixedClock,
        &NoOccupancy,
    );
    assert_eq!(same.unwrap_err().code, "NO_CHANGES");
}
