use crate::components::sync::adapters::outbound::SqliteOperationStore;
use crate::components::sync::application::OperationStore;
use crate::tests::components::patients::test_support::{access, database, seed_access};
use serde_json::json;

const COMMANDS: [(&str, &str); 18] = [
    ("patients_register_patient", "profiles"),
    ("patients_update_patient_profile", "profiles"),
    ("patients_set_patient_completion", "profiles"),
    ("patients_set_patient_profile_status", "profiles"),
    ("patients_add_patient_identifier", "profiles"),
    ("patients_list_patients", "profiles"),
    ("patients_get_patient", "profiles"),
    ("patients_list_patient_identifiers", "profiles"),
    ("patients_create_contact_person", "contacts"),
    ("patients_update_contact_person", "contacts"),
    ("patients_set_contact_person_status", "contacts"),
    ("patients_create_patient_contact_relationship", "contacts"),
    ("patients_update_patient_contact_relationship", "contacts"),
    (
        "patients_set_patient_contact_relationship_status",
        "contacts",
    ),
    ("patients_list_contacts", "contacts"),
    ("patients_get_contact", "contacts"),
    ("patients_list_patient_contacts", "contacts"),
    ("patients_list_contact_patients", "contacts"),
];

#[test]
fn each_command_has_explicit_permission_manifest_and_handler() {
    let manifest = include_str!("../../../../build.rs");
    let handlers = include_str!("../../../lib.rs");
    let capabilities: serde_json::Value =
        serde_json::from_str(include_str!("../../../../capabilities/default.json")).unwrap();
    for (command, group) in COMMANDS {
        assert!(
            manifest.contains(&format!("\"{command}\"")),
            "manifest {command}"
        );
        let module = if group == "profiles" {
            "inbound_profiles"
        } else {
            "inbound_contacts"
        };
        assert!(
            handlers.contains(&format!(
                "components::patients::adapters::{module}::{command}"
            )),
            "handler {command}"
        );
        assert!(
            capabilities["permissions"]
                .as_array()
                .unwrap()
                .contains(&json!(format!("allow-{}", command.replace('_', "-")))),
            "capability {command}"
        );
    }
}

#[test]
fn queries_stay_local_and_writes_check_session_permissions() {
    use crate::components::identity::{
        domain::IdentityContext,
        repository::{AccessStore, SqliteAccessStore},
    };
    use crate::components::patients::adapters::inbound_contacts::{
        CreateContactRequest, run as contact_run,
    };
    use crate::components::patients::adapters::inbound_profiles::run as profile_run;

    let mut c = database();
    let session = access();
    let tx = c.transaction().unwrap();
    SqliteAccessStore
        .save_context(
            &tx,
            &IdentityContext {
                issuer: "https://synthetic.example".into(),
                subject: "synthetic".into(),
                user_id: session.context.user_id,
                selected_account_id: Some(session.context.account_id),
            },
        )
        .unwrap();
    tx.commit().unwrap();
    // Reads work with the session's last valid permissions, no HTTP involved.
    let page = profile_run(&mut c, |cases| {
        cases.list_patients(&crate::components::patients::application::PatientFilter::default())
    })
    .unwrap();
    assert_eq!(page.total, 0);

    // A session whose allowedActions lack the write is denied without effects.
    let mut denied = access();
    denied
        .allowed_actions
        .retain(|v| v != "CreateContactPerson");
    seed_access(&mut c, &denied);
    let create = CreateContactRequest {
        full_name: "Denegado".into(),
        phone_code: None,
        phone: None,
        issuing_country: None,
        document_type: None,
        document_number: None,
    };
    assert_eq!(
        contact_run(&mut c, |cases| {
            cases.create_contact_person(create.try_into().unwrap())
        })
        .unwrap_err()
        .code,
        "PERMISSION_DENIED"
    );
    assert!(
        SqliteOperationStore
            .list_pending(&c, 10)
            .unwrap()
            .is_empty()
    );
}
