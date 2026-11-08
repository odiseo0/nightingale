use super::{
    application::{EditBase, SchedulingMutation, SchedulingRepository},
    domain::{RulePatch, RuleValue, Status},
    test_support::{
        FakeOccupancy, FixedClock, Ids, NoOccupancy, PROFESSIONAL, access, database, repository,
    },
};
use crate::components::appointments::application::OccupancyInterval;
use jiff::Timestamp;
use serde_json::{Value, json};
use std::str::FromStr;

fn op(id: &str) -> crate::shared_kernel::id::OperationId {
    crate::shared_kernel::id::OperationId::try_from(id.to_owned()).unwrap()
}

/// Una cita conocida un miércoles 2026-05-06 de 09:30 a 10:00 en Caracas
/// (13:30–14:00 UTC, UTC-4 en mayo).
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

/// Regla semanal que cubre el miércoles completo.
fn covering_rule() -> RuleValue {
    RuleValue::new(
        PROFESSIONAL.to_owned().try_into().unwrap(),
        3,
        0,
        1440,
        super::domain::parse_local_date("2026-01-01").unwrap(),
        None,
    )
    .unwrap()
}

fn outbox_payload(c: &rusqlite::Connection, action: &str) -> Value {
    serde_json::from_str(
        &c.query_row(
            "SELECT payload FROM sync_outbox WHERE action = ?1",
            [action],
            |r| r.get::<_, String>(0),
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn create_and_edit_payloads_follow_the_contract() {
    let mut c = database();
    let (created_op, edited_op, _rule_id) = {
        let mut repo = repository(&mut c);
        let ids = Ids::default();

        let created = repo
            .persist(
                &access(),
                SchedulingMutation::CreateRule {
                    value: covering_rule(),
                },
                &ids,
                &FixedClock,
                &NoOccupancy,
            )
            .unwrap();
        let rule_id = created.entity_key.as_str().unwrap().to_owned();
        let edited = repo
            .persist(
                &access(),
                SchedulingMutation::UpdateRule {
                    rule_id: rule_id.clone(),
                    base: EditBase::LocalOperation {
                        operation_id: op(&created.operation_id),
                    },
                    patch: RulePatch::new(None, None, Some(1200), None, Some(None)).unwrap(),
                },
                &ids,
                &FixedClock,
                &NoOccupancy,
            )
            .unwrap();
        repo.persist(
            &access(),
            SchedulingMutation::SetRuleStatus {
                rule_id,
                base: EditBase::LocalOperation {
                    operation_id: op(&edited.operation_id),
                },
                status: Status::Inactive,
            },
            &ids,
            &FixedClock,
            &NoOccupancy,
        )
        .unwrap();
        (created.operation_id, edited.operation_id, ())
    };

    let _ = (created_op, edited_op);
    assert_eq!(
        outbox_payload(&c, "CreateScheduleRule"),
        json!({
            "professionalId": PROFESSIONAL,
            "isoWeekday": 3,
            "localStartMinute": 0,
            "localEndMinute": 1440,
            "validFrom": "2026-01-01"
        })
    );
    // Solo los campos presentes viajan y validUntil null retira la vigencia.
    assert_eq!(
        outbox_payload(&c, "UpdateScheduleRule"),
        json!({"localEndMinute": 1200, "validUntil": Value::Null})
    );
    assert_eq!(
        outbox_payload(&c, "SetScheduleRuleStatus"),
        json!({"status": "inactive"})
    );
}

#[test]
fn known_appointment_outside_resulting_schedule_blocks_the_write() {
    let mut c = database();
    let ids = Ids::default();
    let occupancy = FakeOccupancy(vec![wednesday_appointment()]);

    // Regla de martes sola: la cita del miércoles queda fuera.
    let rejected = repository(&mut c).persist(
        &access(),
        SchedulingMutation::CreateRule {
            value: RuleValue::new(
                PROFESSIONAL.to_owned().try_into().unwrap(),
                2,
                0,
                1440,
                super::domain::parse_local_date("2026-01-01").unwrap(),
                None,
            )
            .unwrap(),
        },
        &ids,
        &FixedClock,
        &occupancy,
    );
    assert_eq!(rejected.unwrap_err().code, "SLOT_UNAVAILABLE");
    let count: i64 = c
        .query_row("SELECT count(*) FROM sync_outbox", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0, "no debe crear intención");

    // Regla que cubre el miércoles completo: aceptada.
    let (rule_id, created_op) = {
        let mut repo = repository(&mut c);
        let created = repo
            .persist(
                &access(),
                SchedulingMutation::CreateRule {
                    value: covering_rule(),
                },
                &ids,
                &FixedClock,
                &occupancy,
            )
            .unwrap();
        (
            created.entity_key.as_str().unwrap().to_owned(),
            created.operation_id,
        )
    };

    // Reducir la regla para terminar antes de la cita: rechazada.
    let shrunk = repository(&mut c).persist(
        &access(),
        SchedulingMutation::UpdateRule {
            rule_id: rule_id.clone(),
            base: EditBase::LocalOperation {
                operation_id: op(&created_op),
            },
            patch: RulePatch::new(None, None, Some(560), None, None).unwrap(),
        },
        &ids,
        &FixedClock,
        &occupancy,
    );
    assert_eq!(shrunk.unwrap_err().code, "SLOT_UNAVAILABLE");

    // Retirar la regla (inactive) con la cita conocida: rechazada.
    let withdrawn = repository(&mut c).persist(
        &access(),
        SchedulingMutation::SetRuleStatus {
            rule_id,
            base: EditBase::LocalOperation {
                operation_id: op(&created_op),
            },
            status: Status::Inactive,
        },
        &ids,
        &FixedClock,
        &occupancy,
    );
    assert_eq!(withdrawn.unwrap_err().code, "SLOT_UNAVAILABLE");
}

#[test]
fn merged_list_shows_accepted_provisional_and_operations() {
    let mut c = database();
    let ids = Ids::default();
    let (rule_id, created_op) = {
        let mut repo = repository(&mut c);
        let created = repo
            .persist(
                &access(),
                SchedulingMutation::CreateRule {
                    value: covering_rule(),
                },
                &ids,
                &FixedClock,
                &NoOccupancy,
            )
            .unwrap();
        (
            created.entity_key.as_str().unwrap().to_owned(),
            created.operation_id,
        )
    };

    // Vista aceptada simula la descarga: versión 1 aceptada.
    c.execute(
        "INSERT INTO schedule_rule_view (account_id,user_id,device_id,rule_id,professional_id,iso_weekday,local_start_minute,local_end_minute,valid_from,status,version) VALUES ('019a1234-0000-7000-8000-000000000001','019a1234-0000-7000-8000-000000000002','019a1234-0000-7000-8000-000000000003',?1,?2,3,0,1440,'2026-01-01','active',1)",
        rusqlite::params![&rule_id, PROFESSIONAL],
    )
    .unwrap();

    let rows = repository(&mut c).rule_rows_list(&access()).unwrap();
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.accepted.as_ref().unwrap().version, 1);
    let provisional = row.provisional.as_ref().unwrap();
    assert_eq!(provisional.operation_id, created_op);
    assert_eq!(provisional.expected_version, 0);
    assert_eq!(row.operations.len(), 1);
    assert_eq!(row.visible().unwrap().rule_id, rule_id);
}

#[test]
fn edit_on_accepted_base_without_provisional_is_allowed() {
    let mut c = database();
    let ids = Ids::default();

    // Vista aceptada sin intenciones locales.
    c.execute(
        "INSERT INTO schedule_rule_view (account_id,user_id,device_id,rule_id,professional_id,iso_weekday,local_start_minute,local_end_minute,valid_from,status,version) VALUES ('019a1234-0000-7000-8000-000000000001','019a1234-0000-7000-8000-000000000002','019a1234-0000-7000-8000-000000000003','019a1234-0000-7000-8000-000000000030',?1,3,540,1080,'2026-01-01','active',4)",
        rusqlite::params![PROFESSIONAL],
    )
    .unwrap();

    let result = repository(&mut c)
        .persist(
            &access(),
            SchedulingMutation::UpdateRule {
                rule_id: "019a1234-0000-7000-8000-000000000030".into(),
                base: EditBase::Accepted { version: 4 },
                patch: RulePatch::new(None, Some(600), None, None, None).unwrap(),
            },
            &ids,
            &FixedClock,
            &NoOccupancy,
        )
        .unwrap();
    assert_eq!(result.expected_version, 4);
    assert!(result.depends_on.is_empty());
}

#[test]
fn occupancy_of_another_professional_does_not_block_the_write() {
    let mut c = database();
    let ids = Ids::default();

    // Ocupación de OTRO profesional: no bloquea la regla del profesional de la mutación.
    let other = (
        "019a1234-0000-7000-8000-000000000009".to_string(),
        OccupancyInterval::new(
            "019a1234-0000-7000-8000-000000000040".into(),
            Timestamp::from_str("2026-05-06T13:30:00Z").unwrap(),
            Timestamp::from_str("2026-05-06T14:00:00Z").unwrap(),
            "scheduled",
        )
        .unwrap(),
    );
    let occupancy = FakeOccupancy(vec![other]);
    let value = RuleValue::new(
        PROFESSIONAL.to_owned().try_into().unwrap(),
        2,
        0,
        1440,
        super::domain::parse_local_date("2026-01-01").unwrap(),
        None,
    )
    .unwrap();
    let created = repository(&mut c)
        .persist(
            &access(),
            SchedulingMutation::CreateRule { value },
            &ids,
            &FixedClock,
            &occupancy,
        )
        .unwrap();
    assert_eq!(created.expected_version, 0);
}
