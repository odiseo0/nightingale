use serde_json::json;

const COMMANDS: [(&str, &str); 14] = [
    ("scheduling_create_rule", "inbound_rules"),
    ("scheduling_update_rule", "inbound_rules"),
    ("scheduling_set_rule_status", "inbound_rules"),
    ("scheduling_create_exception", "inbound_exceptions"),
    ("scheduling_update_exception", "inbound_exceptions"),
    ("scheduling_set_exception_status", "inbound_exceptions"),
    ("scheduling_create_block", "inbound_blocks"),
    ("scheduling_update_block", "inbound_blocks"),
    ("scheduling_set_block_status", "inbound_blocks"),
    ("scheduling_list_rules", "inbound_queries"),
    ("scheduling_list_exceptions", "inbound_queries"),
    ("scheduling_list_blocks", "inbound_queries"),
    ("scheduling_get_local_availability", "inbound_availability"),
    (
        "scheduling_check_remote_availability",
        "inbound_availability",
    ),
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
                "components::scheduling::adapters::{module}::{command}"
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
