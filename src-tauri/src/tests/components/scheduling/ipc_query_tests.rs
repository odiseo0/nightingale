use super::test_support::{PROFESSIONAL, database, database_with_professional};
use crate::components::scheduling::adapters::inbound_availability::{
    LocalAvailabilityRequest, get_local_availability,
};
use crate::components::scheduling::adapters::inbound_queries::{
    list_blocks, list_exceptions, list_rules,
};
use crate::components::scheduling::adapters::inbound_rules::create_rule;
use serde_json::json;

const SERVICE: &str = "019a1234-0000-7000-8000-0000000000b1";

fn seed_offer(c: &mut rusqlite::Connection) {
    let tx = c.transaction().unwrap();
    tx.execute(
        "INSERT INTO service_offer_view (account_id,user_id,device_id,professional_id,service_id,name,default_duration_minutes,duration_minutes_override,effective_duration_minutes,service_status,status,version,service_version) VALUES ('019a1234-0000-7000-8000-000000000001','019a1234-0000-7000-8000-000000000002','019a1234-0000-7000-8000-000000000003',?1,?2,'Consulta',30,NULL,30,'active','active',2,7)",
        rusqlite::params![PROFESSIONAL, SERVICE],
    )
    .unwrap();
    tx.commit().unwrap();
}

fn availability_request() -> LocalAvailabilityRequest {
    serde_json::from_value(json!({
        "professionalId": PROFESSIONAL,
        "serviceId": SERVICE,
        "from": "2026-05-06T04:00:00Z",
        "until": "2026-05-07T04:00:00Z"
    }))
    .unwrap()
}

#[test]
fn query_dto_rejects_bad_input_and_scope_fields() {
    // Campos extra (posible inyección de ámbito) rechazados.
    assert!(
        serde_json::from_value::<LocalAvailabilityRequest>(json!({
            "professionalId": PROFESSIONAL,
            "serviceId": SERVICE,
            "from": "2026-05-06T04:00:00Z",
            "until": "2026-05-07T04:00:00Z",
            "accountId": "otra",
            "sql": "DROP TABLE schedule_rule_view"
        }))
        .is_err()
    );
    // from/until mal formados: el DTO admite texto, el caso lo valida.
    let bad: LocalAvailabilityRequest = serde_json::from_value(json!({
        "professionalId": PROFESSIONAL,
        "serviceId": SERVICE,
        "from": "ayer",
        "until": "2026-05-07T04:00:00Z"
    }))
    .unwrap();
    let mut c = database_with_professional();
    seed_offer(&mut c);
    assert_eq!(
        get_local_availability(&mut c, bad).unwrap_err().code,
        "VALIDATION_ERROR"
    );
}

#[test]
fn list_commands_return_fused_rows_without_session_leaks() {
    let mut c = database();
    let created = create_rule(
        &mut c,
        serde_json::from_value(json!({
            "professionalId": PROFESSIONAL,
            "isoWeekday": 3,
            "localStartMinute": 480,
            "localEndMinute": 720,
            "validFrom": "2026-01-01"
        }))
        .unwrap(),
    )
    .unwrap();

    let rules = list_rules(&mut c).unwrap();
    assert_eq!(rules.len(), 1);
    assert!(rules[0].accepted.is_none());
    assert_eq!(
        rules[0].provisional.as_ref().unwrap().operation_id,
        created.operation_id
    );

    assert!(list_exceptions(&mut c).unwrap().is_empty());
    assert!(list_blocks(&mut c).unwrap().is_empty());
}

#[test]
fn local_availability_reports_coverage_from_the_sync_cursor() {
    let mut c = database_with_professional();
    seed_offer(&mut c);
    create_rule(
        &mut c,
        serde_json::from_value(json!({
            "professionalId": PROFESSIONAL,
            "isoWeekday": 3,
            "localStartMinute": 480,
            "localEndMinute": 720,
            "validFrom": "2026-01-01"
        }))
        .unwrap(),
    )
    .unwrap();

    // Sin cursor: solo local.
    let evaluation = get_local_availability(&mut c, availability_request()).unwrap();
    assert_eq!(evaluation.coverage, "no_synced_window");
    assert_eq!(evaluation.intervals.len(), 1);

    // Cursor con ventana que cubre la consulta.
    c.execute(
        "INSERT INTO sync_cursor (account_id,user_id,device_id,permission_epoch,cursor,window) VALUES ('019a1234-0000-7000-8000-000000000001','019a1234-0000-7000-8000-000000000002','019a1234-0000-7000-8000-000000000003',1,'cursor-1', json('{\"appointmentsFrom\":\"2026-05-01T00:00:00Z\",\"appointmentsUntil\":\"2026-05-31T00:00:00Z\"}'))",
        [],
    )
    .unwrap();
    let evaluation = get_local_availability(&mut c, availability_request()).unwrap();
    assert_eq!(evaluation.coverage, "within_synced_window");
    assert!(evaluation.provisional);

    // Ventana que no cubre: insuficiente.
    c.execute(
        "UPDATE sync_cursor SET window = json('{\"appointmentsFrom\":\"2026-05-01T00:00:00Z\",\"appointmentsUntil\":\"2026-05-06T12:00:00Z\"}')",
        [],
    )
    .unwrap();
    let evaluation = get_local_availability(&mut c, availability_request()).unwrap();
    assert_eq!(evaluation.coverage, "insufficient_coverage");
}
