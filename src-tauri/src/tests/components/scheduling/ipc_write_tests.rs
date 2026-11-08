use super::test_support::{PROFESSIONAL, database, database_with_professional};
use crate::components::scheduling::adapters::inbound_blocks::{
    CreateBlockRequest, UpdateBlockRequest, create_block, update_block,
};
use crate::components::scheduling::adapters::inbound_exceptions::create_exception;
use crate::components::scheduling::adapters::inbound_rules::{
    CreateRuleRequest, EditBaseRequest, RulePatchRequest, SetRuleStatusRequest, UpdateRuleRequest,
    create_rule, set_rule_status, update_rule,
};
use serde_json::json;

fn base_local(operation: &str) -> EditBaseRequest {
    serde_json::from_value(json!({"operationId": operation})).unwrap()
}

#[test]
fn write_dtos_reject_extra_null_and_ambiguous_fields() {
    // Campos extra y ámbito inyectado se rechazan.
    assert!(
        serde_json::from_value::<CreateRuleRequest>(json!({
            "professionalId": PROFESSIONAL,
            "isoWeekday": 3,
            "localStartMinute": 480,
            "localEndMinute": 720,
            "validFrom": "2026-01-01",
            "accountId": "otra-cuenta"
        }))
        .is_err()
    );
    assert!(
        serde_json::from_value::<CreateRuleRequest>(json!({
            "professionalId": PROFESSIONAL,
            "isoWeekday": 3,
            "localStartMinute": 480,
            "validFrom": "2026-01-01"
        }))
        .is_err()
    );
    // Base ambigua, vacía o no positiva: el DTO admite la forma, el dominio la rechaza.
    for bad in [
        json!({"acceptedVersion": 1, "operationId": "019a1234-0000-7000-8000-000000000101"}),
        json!({}),
        json!({"acceptedVersion": 0}),
    ] {
        let parsed: EditBaseRequest = serde_json::from_value(bad).unwrap();
        assert!(crate::components::scheduling::application::EditBase::try_from(parsed).is_err());
    }
    // Patch: validUntil null explícito retira; ausencia conserva.
    let patch: RulePatchRequest = serde_json::from_value(json!({
        "localEndMinute": 720,
        "validUntil": null
    }))
    .unwrap();
    assert_eq!(patch.valid_until, Some(None));
    let patch: RulePatchRequest = serde_json::from_value(json!({"localEndMinute": 720})).unwrap();
    assert_eq!(patch.valid_until, None);
    // Estado nulo o desconocido se rechaza.
    assert!(
        serde_json::from_value::<SetRuleStatusRequest>(json!({
            "ruleId": "019a1234-0000-7000-8000-000000000101",
            "base": {"acceptedVersion": 1},
            "status": null
        }))
        .is_err()
    );
    // Excepción closed con minutos no se acepta en el DTO de dominio.
    let closed_with_minutes = json!({
        "professionalId": PROFESSIONAL,
        "localDate": "2026-05-06",
        "kind": "closed",
        "localStartMinute": 30
    });
    let mut c = database();
    assert_eq!(
        create_exception(&mut c, serde_json::from_value(closed_with_minutes).unwrap())
            .unwrap_err()
            .code,
        "VALIDATION_ERROR"
    );
}

#[test]
fn authorized_rule_roundtrip_via_commands() {
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

    let rule_id = created.entity_key.as_str().unwrap();
    let edited = update_rule(
        &mut c,
        UpdateRuleRequest {
            rule_id: rule_id.into(),
            base: base_local(&created.operation_id),
            patch: serde_json::from_value(json!({"localStartMinute": 540})).unwrap(),
        },
    )
    .unwrap();
    assert_eq!(edited.expected_version, 1);

    let status = set_rule_status(
        &mut c,
        SetRuleStatusRequest {
            rule_id: rule_id.into(),
            base: base_local(&edited.operation_id),
            status: "inactive".into(),
        },
    )
    .unwrap();
    assert_eq!(status.provisional["status"], json!("inactive"));
}

#[test]
fn global_block_is_denied_for_assistant_and_allowed_for_owner() {
    // Asistente (acceso por defecto de test_support): denegado.
    let mut c = database_with_professional();
    let denied = create_block(
        &mut c,
        serde_json::from_value(json!({
            "professionalId": PROFESSIONAL,
            "accountId": null,
            "startsAt": "2026-04-01T13:00:00Z",
            "endsAt": "2026-04-01T17:00:00Z",
            "reasonCode": "vacation"
        }))
        .unwrap(),
    );
    assert_eq!(denied.unwrap_err().code, "PERMISSION_DENIED");

    // Bloqueo de cuenta con el mismo camino de entrada: permitido al asistente.
    let mut c = database_with_professional();
    let account_block: CreateBlockRequest = serde_json::from_value(json!({
        "professionalId": PROFESSIONAL,
        "accountId": "019a1234-0000-7000-8000-000000000001",
        "startsAt": "2026-04-01T13:00:00Z",
        "endsAt": "2026-04-01T17:00:00Z",
        "reasonCode": "vacation"
    }))
    .unwrap();
    let created = create_block(&mut c, account_block).unwrap();
    assert_eq!(created.entity_type, "schedule_block");

    let block_id = created.entity_key.as_str().unwrap().to_owned();
    let updated = update_block(
        &mut c,
        UpdateBlockRequest {
            block_id,
            base: base_local(&created.operation_id),
            patch: serde_json::from_value(json!({
                "endsAt": "2026-04-01T18:00:00Z",
                "reasonCode": "administrative"
            }))
            .unwrap(),
        },
    )
    .unwrap();
    assert_eq!(updated.expected_version, 1);
}
