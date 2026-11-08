//! Registration coherence: each appointments command has a handler, a build
//! manifest entry and an explicit capability permission.
use serde_json::json;

const COMMANDS: [(&str, &str); 10] = [
    ("appointments_create_appointment", "inbound_commands"),
    ("appointments_confirm_appointment", "inbound_commands"),
    ("appointments_reschedule_appointment", "inbound_commands"),
    ("appointments_cancel_appointment", "inbound_commands"),
    ("appointments_complete_appointment", "inbound_commands"),
    ("appointments_mark_no_show", "inbound_commands"),
    ("appointments_list_appointments", "inbound_queries"),
    ("appointments_get_appointment", "inbound_queries"),
    ("appointments_get_history", "inbound_queries"),
    ("appointments_get_booking_context", "inbound_queries"),
];

#[test]
fn each_command_has_explicit_permission_manifest_and_handler() {
    let manifest = include_str!("../../../build.rs");
    let handlers = include_str!("../../lib.rs");
    let capabilities: serde_json::Value =
        serde_json::from_str(include_str!("../../../capabilities/default.json")).unwrap();
    for (command, module) in COMMANDS {
        assert!(
            manifest.contains(&format!("\"{command}\"")),
            "manifest {command}"
        );
        assert!(
            handlers.contains(&format!(
                "components::appointments::adapters::{module}::{command}"
            )),
            "handler {command}"
        );
        let permission = format!("allow-{}", command.replace('_', "-"));
        assert!(
            capabilities["permissions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry == &json!(permission)),
            "permission {permission}"
        );
    }
}
