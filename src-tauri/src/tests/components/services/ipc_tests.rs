use crate::components::services::adapters::inbound::*;
use crate::components::services::domain::DurationMinutes;
use crate::tests::components::services::test_support::*;
use crate::{
    components::identity::{
        domain::{AccessRole, IdentityContext},
        repository::{AccessStore, SqliteAccessStore},
    },
    shared_kernel::id::OperationId,
};
use rusqlite::Connection;
use serde_json::json;

fn select(c: &mut Connection) {
    let access = access();
    let tx = c.transaction().unwrap();
    SqliteAccessStore
        .save_context(
            &tx,
            &IdentityContext {
                issuer: "https://synthetic.example".into(),
                subject: "synthetic".into(),
                user_id: access.context.user_id,
                selected_account_id: Some(access.context.account_id),
            },
        )
        .unwrap();
    tx.commit().unwrap();
}

#[test]
fn dto_rejects_missing_override_null_service_fields_fractions_and_scope_injection() {
    let base = json!({"professionalId":PROFESSIONAL,"serviceId":SERVICE,"expectedVersion":1});
    assert!(serde_json::from_value::<UpdateOfferRequest>(base.clone()).is_err());
    let mut clear = base;
    clear["durationMinutesOverride"] = json!(null);
    assert!(
        serde_json::from_value::<UpdateOfferRequest>(clear)
            .unwrap()
            .duration_minutes_override
            .is_none()
    );
    for patch in [
        json!({"name":null}),
        json!({"defaultDurationMinutes":null}),
        json!({"defaultDurationMinutes":1.5}),
        json!({"sql":"DROP TABLE"}),
    ] {
        assert!(serde_json::from_value::<ServicePatchRequest>(patch).is_err());
    }
    assert!(serde_json::from_value::<CreateServiceRequest>(json!({"professionalId":PROFESSIONAL,"name":"Synthetic","defaultDurationMinutes":30,"accountId":"other"})).is_err());
}

#[test]
fn desktop_entry_checks_selection_and_assistant_writes_and_redacts_transport() {
    let mut c = database();
    assert_eq!(
        run(&mut c, |cases| cases.list_catalog()).unwrap_err().code,
        "AUTHENTICATION_REQUIRED"
    );
    select(&mut c);
    let result = run(&mut c, |cases| {
        cases.create_service(
            PROFESSIONAL.to_owned().try_into().unwrap(),
            "Synthetic".into(),
            DurationMinutes::new(30).unwrap(),
        )
    })
    .unwrap();
    let value = serde_json::to_value(&result).unwrap();
    assert_eq!(value["syncStatus"], "pending");
    assert!(value.get("payload").is_none());
    assert!(value.get("permissionEpoch").is_none());
    let view = serde_json::to_value(run(&mut c, |cases| cases.list_catalog()).unwrap()).unwrap();
    assert!(view["services"][0]["isProvisional"].as_bool().unwrap());
    for field in [
        "payload",
        "contractVersion",
        "semanticFingerprint",
        "deviceId",
        "permissionEpoch",
        "token",
    ] {
        assert!(!view.to_string().contains(&format!("\"{field}\"")));
    }
    let mut assistant = access();
    assistant.role = AccessRole::Assistant;
    let tx = c.transaction().unwrap();
    SqliteAccessStore.save_access(&tx, &assistant).unwrap();
    tx.commit().unwrap();
    assert!(
        !run(&mut c, |cases| cases.list_catalog())
            .unwrap()
            .can_manage
    );
    let error = run(&mut c, |cases| {
        cases.create_service(
            PROFESSIONAL.to_owned().try_into().unwrap(),
            "Denied".into(),
            DurationMinutes::new(30).unwrap(),
        )
    })
    .unwrap_err();
    assert_eq!(error.code, "PERMISSION_DENIED");
    assert_eq!(
        c.query_row("SELECT count(*) FROM sync_outbox", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    OperationId::try_from(result.operation_id).unwrap();
}

#[test]
fn each_command_has_explicit_permission_manifest_and_handler() {
    let manifest = include_str!("../../../../build.rs");
    let handlers = include_str!("../../../lib.rs");
    let capabilities: serde_json::Value =
        serde_json::from_str(include_str!("../../../../capabilities/default.json")).unwrap();
    for command in [
        "services_list_catalog",
        "services_create_service",
        "services_update_service",
        "services_set_service_status",
        "services_offer_service",
        "services_update_service_offer",
        "services_set_service_offer_status",
        "services_update_professional_profile",
    ] {
        assert!(manifest.contains(&format!("\"{command}\"")));
        assert!(handlers.contains(&format!(
            "components::services::adapters::inbound::{command}"
        )));
        assert!(
            capabilities["permissions"]
                .as_array()
                .unwrap()
                .contains(&json!(format!("allow-{}", command.replace('_', "-"))))
        );
    }
}
