use crate::components::sync::{
    adapters::remote_models::RemoteState, domain::OperationEnvelope, remote::ChangeGroup,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn contract() -> Value {
    serde_json::from_str(include_str!(
        "../../../../../docs/contracts/api-v1/openapi.json"
    ))
    .unwrap()
}

#[test]
fn account_services_actions_payload_models_and_keys_match_fixed_bundle() {
    let capabilities: Value = serde_json::from_str(include_str!(
        "../../../../../docs/contracts/api-v1/capabilities.json"
    ))
    .unwrap();
    let schema: Value = serde_json::from_str(include_str!(
        "../../../../../docs/contracts/api-v1/accepted-operations.schema.json"
    ))
    .unwrap();
    let fixtures: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/account-services-operations.json")).unwrap();
    assert_eq!(fixtures.len(), 10);
    let mut actions = std::collections::BTreeSet::new();
    for fixture in fixtures {
        let operation = &fixture["operation"];
        let action = operation["action"].as_str().unwrap();
        assert!(actions.insert(action.to_owned()));
        assert!(
            capabilities["operationCatalog"]
                .as_array()
                .unwrap()
                .contains(&json!(action))
        );
        let model = capabilities["payloadModels"][action].as_str().unwrap();
        assert_eq!(model.rsplit('.').next().unwrap(), fixture["payloadModel"]);
        check(&schema, &schema, operation, action).unwrap();
        let envelope: OperationEnvelope = serde_json::from_value(operation.clone()).unwrap();
        assert_eq!(serde_json::to_value(envelope).unwrap(), *operation);
        let online = capabilities["onlineOnlyActions"]
            .as_array()
            .unwrap()
            .contains(&json!(action));
        assert_eq!(online, action.contains("Account"));
        if action.contains("Offer") {
            assert_eq!(operation["entityKey"].as_object().unwrap().len(), 2);
            for field in ["professionalId", "serviceId"] {
                crate::shared_kernel::id::canonical_uuid_v7(
                    operation["entityKey"][field].as_str().unwrap(),
                )
                .unwrap();
            }
            let mut change = crate::tests::components::sync::download_tests::group()
                .changes
                .remove(10);
            assert_eq!(
                change
                    .entity_key
                    .as_object()
                    .unwrap()
                    .keys()
                    .collect::<Vec<_>>(),
                operation["entityKey"]
                    .as_object()
                    .unwrap()
                    .keys()
                    .collect::<Vec<_>>()
            );
            change
                .entity_key
                .as_object_mut()
                .unwrap()
                .remove("serviceId");
            assert!(
                change
                    .translate(&crate::tests::components::sync::download_tests::scope())
                    .is_err()
            );
        }
        let mut invalid = operation.clone();
        invalid["payload"]["unexpected"] = json!(true);
        assert!(check(&schema, &schema, &invalid, action).is_err());
    }
}

#[test]
fn patients_actions_payload_models_and_keys_match_fixed_bundle() {
    let capabilities: Value = serde_json::from_str(include_str!(
        "../../../../../docs/contracts/api-v1/capabilities.json"
    ))
    .unwrap();
    let schema: Value = serde_json::from_str(include_str!(
        "../../../../../docs/contracts/api-v1/accepted-operations.schema.json"
    ))
    .unwrap();
    let fixtures: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/patients-operations.json")).unwrap();
    assert_eq!(fixtures.len(), 11);
    let mut actions = std::collections::BTreeSet::new();
    for fixture in fixtures {
        let operation = &fixture["operation"];
        let action = operation["action"].as_str().unwrap();
        assert!(actions.insert(action.to_owned()));
        assert!(
            capabilities["operationCatalog"]
                .as_array()
                .unwrap()
                .contains(&json!(action))
        );
        let model = capabilities["payloadModels"][action].as_str().unwrap();
        assert_eq!(model.rsplit('.').next().unwrap(), fixture["payloadModel"]);
        check(&schema, &schema, operation, action).unwrap();
        let envelope: OperationEnvelope = serde_json::from_value(operation.clone()).unwrap();
        assert_eq!(serde_json::to_value(envelope).unwrap(), *operation);
        assert!(
            !capabilities["onlineOnlyActions"]
                .as_array()
                .unwrap()
                .contains(&json!(action)),
            "{action} must stay offline-capable"
        );
        if action.contains("Relationship") {
            assert_eq!(operation["entityKey"].as_object().unwrap().len(), 2);
            for field in ["patientId", "contactPersonId"] {
                crate::shared_kernel::id::canonical_uuid_v7(
                    operation["entityKey"][field].as_str().unwrap(),
                )
                .unwrap();
            }
            let mut change = crate::tests::components::sync::download_tests::group()
                .changes
                .remove(8);
            assert!(matches!(
                change.entity_type,
                crate::components::sync::remote::EntityType::PatientContact
            ));
            assert_eq!(
                change
                    .entity_key
                    .as_object()
                    .unwrap()
                    .keys()
                    .collect::<Vec<_>>(),
                operation["entityKey"]
                    .as_object()
                    .unwrap()
                    .keys()
                    .collect::<Vec<_>>()
            );
            change
                .entity_key
                .as_object_mut()
                .unwrap()
                .remove("contactPersonId");
            assert!(
                change
                    .translate(&crate::tests::components::sync::download_tests::scope())
                    .is_err()
            );
        } else {
            crate::shared_kernel::id::canonical_uuid_v7(operation["entityKey"].as_str().unwrap())
                .unwrap();
        }
        let mut invalid = operation.clone();
        invalid["payload"]["unexpected"] = json!(true);
        assert!(check(&schema, &schema, &invalid, action).is_err());
    }
}

// Test-only structural checks for the schema forms used in these fixtures.
// Business validation, IANA zones and full date arithmetic belong to API.
fn check(root: &Value, schema: &Value, value: &Value, path: &str) -> Result<(), String> {
    let fail = || Err(format!("contract mismatch at {path}"));

    if let Some(reference) = schema["$ref"].as_str() {
        return check(
            root,
            root.pointer(reference.trim_start_matches('#'))
                .ok_or_else(|| format!("unresolved {reference}"))?,
            value,
            path,
        );
    }

    if let Some(all) = schema["allOf"].as_array() {
        for part in all {
            check(root, part, value, path)?;
        }
    }

    for keyword in ["anyOf", "oneOf"] {
        if let Some(parts) = schema[keyword].as_array() {
            let matches = parts
                .iter()
                .filter(|part| check(root, part, value, path).is_ok())
                .count();

            if matches == 0 || (keyword == "oneOf" && matches != 1) {
                return fail();
            }
        }
    }

    if schema.get("const").is_some_and(|v| v != value)
        || schema["enum"]
            .as_array()
            .is_some_and(|options| !options.contains(value))
    {
        return fail();
    }

    if let Some(kind) = schema["type"].as_str() {
        let valid = match kind {
            "object" => value.is_object(),
            "array" => value.is_array(),
            "string" => value.is_string(),
            "integer" => value.is_i64() || value.is_u64(),
            "number" => value.is_number(),
            "boolean" => value.is_boolean(),
            "null" => value.is_null(),
            _ => false,
        };

        if !valid {
            return fail();
        }
    }
    if let Some(object) = value.as_object() {
        if let Some(required) = schema["required"].as_array() {
            for field in required {
                if !object.contains_key(field.as_str().unwrap()) {
                    return Err(format!("missing {path}.{}", field.as_str().unwrap()));
                }
            }
        }

        for (key, item) in object {
            if let Some(property) = schema["properties"].get(key) {
                check(root, property, item, &format!("{path}.{key}"))?;
            } else if schema["additionalProperties"] == false {
                return fail();
            } else if schema["additionalProperties"].is_object() {
                check(
                    root,
                    &schema["additionalProperties"],
                    item,
                    &format!("{path}.{key}"),
                )?;
            }
        }
    }

    if let Some(items) = value.as_array() {
        for (index, item) in items.iter().enumerate() {
            if let Some(schema) = schema.get("items") {
                check(root, schema, item, &format!("{path}[{index}]"))?;
            }
        }

        if schema["minItems"]
            .as_u64()
            .is_some_and(|n| (items.len() as u64) < n)
            || schema["maxItems"]
                .as_u64()
                .is_some_and(|n| (items.len() as u64) > n)
        {
            return fail();
        }
    }

    if let Some(text) = value.as_str() {
        if schema["minLength"]
            .as_u64()
            .is_some_and(|n| (text.chars().count() as u64) < n)
            || schema["maxLength"]
                .as_u64()
                .is_some_and(|n| (text.chars().count() as u64) > n)
        {
            return fail();
        }

        if schema["format"] == "uuid7" && crate::shared_kernel::id::canonical_uuid_v7(text).is_err()
        {
            return fail();
        }
    }

    if let Some(n) = value.as_f64()
        && (schema["minimum"].as_f64().is_some_and(|min| n < min)
            || schema["maximum"].as_f64().is_some_and(|max| n > max))
    {
        return fail();
    }

    Ok(())
}

#[test]
fn fixed_bundle_hashes_references_and_consumed_routes_match() {
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../../../docs/contracts/api-v1/manifest.json"
    ))
    .unwrap();
    let directory =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/contracts/api-v1");

    for (name, hash) in manifest["files"].as_object().unwrap() {
        let bytes = std::fs::read(directory.join(name)).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            hash.as_str().unwrap(),
            "contract file {name}"
        );
    }

    let root = contract();
    assert_eq!(
        root["paths"].as_object().unwrap().len() as u64,
        manifest["paths"].as_u64().unwrap()
    );
    assert_eq!(
        root["components"]["schemas"].as_object().unwrap().len() as u64,
        manifest["schemas"].as_u64().unwrap()
    );

    fn refs(root: &Value, value: &Value) {
        match value {
            Value::Object(object) => {
                if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
                    assert!(
                        root.pointer(reference.trim_start_matches('#')).is_some(),
                        "{reference}"
                    );
                }

                for child in object.values() {
                    refs(root, child);
                }
            }
            Value::Array(items) => {
                for child in items {
                    refs(root, child)
                }
            }
            _ => {}
        }
    }
    refs(&root, &root);

    for (method, path) in [
        ("get", "/v1/accounts/{account_id}/sync/changes"),
        ("get", "/v1/accounts/{account_id}/sync/events"),
        ("post", "/v1/accounts/{account_id}/sync/operations"),
        (
            "get",
            "/v1/accounts/{account_id}/sync/operations/{operation_id}",
        ),
        ("post", "/v1/accounts/{account_id}/sync/snapshots"),
        (
            "get",
            "/v1/accounts/{account_id}/sync/snapshots/{snapshot_id}/items",
        ),
        (
            "delete",
            "/v1/accounts/{account_id}/sync/snapshots/{snapshot_id}",
        ),
    ] {
        assert!(root["paths"][path][method].is_object(), "{method} {path}");
    }
}

#[test]
fn all_projection_shapes_required_nullable_fields_and_closed_models_match_fixed_schema() {
    let root = contract();
    let fixture: Value = serde_json::from_str(include_str!("fixtures/remote-group.json")).unwrap();
    check(
        &root,
        &root["components"]["schemas"]["ChangeGroup"],
        &fixture,
        "group",
    )
    .unwrap();
    let group: ChangeGroup = serde_json::from_value(fixture.clone()).unwrap();
    assert_eq!(serde_json::to_value(group).unwrap(), fixture);
    let models = [
        "AccountResponse",
        "AccountDeviceResponse",
        "MemberResponse",
        "InvitationResponse",
        "ProfessionalResponse",
        "PatientProfileResponse",
        "PatientIdentifierResponse",
        "ContactResponse",
        "RelationshipResponse",
        "ServiceResponse",
        "ServiceOfferResponse",
        "ScheduleRuleResponse",
        "ScheduleExceptionResponse",
        "ScheduleBlockResponse",
        "AppointmentResponse",
        "AppointmentHistoryProjection",
    ];

    for (item, name) in fixture["changes"].as_array().unwrap().iter().zip(models) {
        let schema = &root["components"]["schemas"][name];
        let state = &item["state"];
        check(&root, schema, state, name).unwrap();
        let envelope = |state: Value| json!({"entityType":item["entityType"],"state":state});
        assert!(
            serde_json::from_value::<RemoteState>(envelope(state.clone())).is_ok(),
            "{name}"
        );

        for required in schema["required"].as_array().unwrap() {
            let field = required.as_str().unwrap();
            let mut missing = state.clone();
            missing.as_object_mut().unwrap().remove(field);
            assert!(
                serde_json::from_value::<RemoteState>(envelope(missing)).is_err(),
                "{name}.{field} must remain required"
            );
        }

        if schema["additionalProperties"] == false {
            let mut extra = state.clone();
            extra["unexpected"] = json!(true);
            assert!(
                serde_json::from_value::<RemoteState>(envelope(extra)).is_err(),
                "{name} extra field"
            );
        }
    }

    let mut bad = fixture.clone();
    bad["changes"][0]["entityVersion"] = json!(0);
    assert!(
        check(
            &root,
            &root["components"]["schemas"]["ChangeGroup"],
            &bad,
            "bad-version"
        )
        .is_err()
    );
}

#[test]
fn operation_payload_absence_null_extra_and_sse_examples_match_fixed_artifacts() {
    let root = contract();
    let operations: Value = serde_json::from_str(include_str!(
        "../../../../../docs/contracts/api-v1/accepted-operations.schema.json"
    ))
    .unwrap();
    let mut operation = json!({"operationId":"019a1234-0000-7000-8000-000000000100","contractVersion":1,"action":"RegisterPatient","entityKey":"019a1234-0000-7000-8000-000000000010","expectedVersion":0,"payload":{"fullName":"Synthetic"},"dependsOn":[]});
    check(&operations, &operations, &operation, "operation").unwrap();
    let dto: OperationEnvelope = serde_json::from_value(operation.clone()).unwrap();
    assert_eq!(serde_json::to_value(dto).unwrap(), operation);
    operation["cloudSecret"] = json!("forbidden");
    assert!(serde_json::from_value::<OperationEnvelope>(operation.clone()).is_err());
    operation.as_object_mut().unwrap().remove("cloudSecret");
    operation["payload"] = Value::Null;
    assert!(check(&operations, &operations, &operation, "null-payload").is_err());
    let original = json!({"operationId":"019a1234-0000-7000-8000-000000000100","status":"accepted","entityVersion":1,"state":null,"error":null});
    check(
        &root,
        &root["components"]["schemas"]["OperationRecord"],
        &original,
        "receipt",
    )
    .unwrap();

    for field in ["entityVersion", "state", "error"] {
        let mut missing = original.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(
            serde_json::from_value::<crate::components::sync::adapters::api::OperationRecord>(
                missing
            )
            .is_err()
        );
    }

    let sse: Value = serde_json::from_str(include_str!(
        "../../../../../docs/contracts/api-v1/sse.schema.json"
    ))
    .unwrap();

    for event in [
        json!({"event":"changes_available","data":{"accountId":"019a1234-0000-7000-8000-000000000001","cursor":"opaque"}}),
        json!({"event":"heartbeat","data":{"serverTime":"2026-10-30T12:00:00Z"}}),
        json!({"event":"access_revoked"}),
    ] {
        check(&sse, &sse, &event, "sse").unwrap();
    }
}

#[test]
fn scheduling_actions_payload_models_and_keys_match_fixed_bundle() {
    let capabilities: Value = serde_json::from_str(include_str!(
        "../../../../../docs/contracts/api-v1/capabilities.json"
    ))
    .unwrap();
    let schema: Value = serde_json::from_str(include_str!(
        "../../../../../docs/contracts/api-v1/accepted-operations.schema.json"
    ))
    .unwrap();
    let fixtures: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/scheduling-operations.json")).unwrap();
    assert_eq!(fixtures.len(), 9);
    let mut actions = std::collections::BTreeSet::new();
    for fixture in fixtures {
        let operation = &fixture["operation"];
        let action = operation["action"].as_str().unwrap();
        assert!(actions.insert(action.to_owned()));
        assert!(
            capabilities["operationCatalog"]
                .as_array()
                .unwrap()
                .contains(&json!(action))
        );
        let model = capabilities["payloadModels"][action].as_str().unwrap();
        assert_eq!(model.rsplit('.').next().unwrap(), fixture["payloadModel"]);
        check(&schema, &schema, operation, action).unwrap();
        let envelope: OperationEnvelope = serde_json::from_value(operation.clone()).unwrap();
        assert_eq!(serde_json::to_value(envelope).unwrap(), *operation);
        assert!(
            !capabilities["onlineOnlyActions"]
                .as_array()
                .unwrap()
                .contains(&json!(action)),
            "{action} must stay offline-capable"
        );
        crate::shared_kernel::id::canonical_uuid_v7(operation["entityKey"].as_str().unwrap())
            .unwrap();
        // The local domain accepts the same values the contract fixture carries.
        let payload = &operation["payload"];
        match action {
            "CreateScheduleRule" => {
                crate::components::scheduling::domain::RuleValue::new(
                    payload["professionalId"]
                        .as_str()
                        .unwrap()
                        .to_owned()
                        .try_into()
                        .unwrap(),
                    payload["isoWeekday"].as_i64().unwrap(),
                    payload["localStartMinute"].as_i64().unwrap(),
                    payload["localEndMinute"].as_i64().unwrap(),
                    crate::components::scheduling::domain::parse_local_date(
                        payload["validFrom"].as_str().unwrap(),
                    )
                    .unwrap(),
                    payload["validUntil"]
                        .as_str()
                        .map(crate::components::scheduling::domain::parse_local_date)
                        .transpose()
                        .unwrap(),
                )
                .unwrap();
            }
            "CreateScheduleException" => {
                crate::components::scheduling::domain::ExceptionValue::new(
                    payload["professionalId"]
                        .as_str()
                        .unwrap()
                        .to_owned()
                        .try_into()
                        .unwrap(),
                    crate::components::scheduling::domain::parse_local_date(
                        payload["localDate"].as_str().unwrap(),
                    )
                    .unwrap(),
                    crate::components::scheduling::domain::ExceptionKind::try_from(
                        payload["kind"].as_str().unwrap(),
                    )
                    .unwrap(),
                    payload["localStartMinute"].as_i64(),
                    payload["localEndMinute"].as_i64(),
                )
                .unwrap();
            }
            "CreateScheduleBlock" => {
                crate::components::scheduling::domain::BlockValue::new(
                    payload["professionalId"]
                        .as_str()
                        .unwrap()
                        .to_owned()
                        .try_into()
                        .unwrap(),
                    payload["accountId"]
                        .as_str()
                        .map(|value| value.to_owned().try_into().unwrap()),
                    crate::components::scheduling::domain::Instant::parse(
                        payload["startsAt"].as_str().unwrap(),
                    )
                    .unwrap(),
                    crate::components::scheduling::domain::Instant::parse(
                        payload["endsAt"].as_str().unwrap(),
                    )
                    .unwrap(),
                    crate::components::scheduling::domain::ReasonCode::try_from(
                        payload["reasonCode"].as_str().unwrap(),
                    )
                    .unwrap(),
                )
                .unwrap();
            }
            _ => {}
        }
        let mut invalid = operation.clone();
        invalid["payload"]["unexpected"] = json!(true);
        assert!(check(&schema, &schema, &invalid, action).is_err());
    }
}

#[test]
fn scheduling_agenda_occupancy_and_availability_states_match_fixed_bundle() {
    use crate::components::sync::adapters::remote_models::RemoteState;

    // The three agenda states parse with their typed models and versions.
    for (entity, state) in [
        (
            "schedule_rule",
            json!({"accountId":"019a1234-0000-7000-8000-000000000001","id":"019a1234-0000-7000-8000-000000000030","professionalId":"019a1234-0000-7000-8000-000000000008","isoWeekday":3,"localStartMinute":480,"localEndMinute":720,"validFrom":"2026-01-01","validUntil":null,"status":"active","createdAt":null,"updatedAt":null,"version":1}),
        ),
        (
            "schedule_exception",
            json!({"accountId":"019a1234-0000-7000-8000-000000000001","id":"019a1234-0000-7000-8000-000000000031","professionalId":"019a1234-0000-7000-8000-000000000008","localDate":"2026-05-06","kind":"replacement","localStartMinute":540,"localEndMinute":600,"status":"active","createdAt":null,"updatedAt":null,"version":1}),
        ),
        (
            "schedule_block",
            json!({"accountId":null,"id":"019a1234-0000-7000-8000-000000000032","professionalId":"019a1234-0000-7000-8000-000000000008","startsAt":"2026-04-01T13:00:00Z","endsAt":"2026-04-01T17:00:00Z","reasonCode":null,"status":"active","createdAt":null,"updatedAt":null,"version":1}),
        ),
    ] {
        let parsed: RemoteState =
            serde_json::from_value(json!({"entityType": entity, "state": state})).unwrap();
        assert_eq!(parsed.version(), 1);
        assert!(parsed.key().is_string());
    }

    // Occupancy: occupying states are accepted by the public interval, cancelled is not.
    for (status, occupies) in [
        ("scheduled", true),
        ("confirmed", true),
        ("completed", true),
        ("no_show", true),
        ("cancelled", false),
    ] {
        let result = crate::components::appointments::application::OccupancyInterval::new(
            "019a1234-0000-7000-8000-000000000040".into(),
            "2026-05-06T13:30:00Z".parse().unwrap(),
            "2026-05-06T14:00:00Z".parse().unwrap(),
            status,
        );
        assert_eq!(result.is_ok(), occupies, "occupancy for {status}");
    }

    // Availability response: no structural drift against the fixed bundle.
    let public_models: Value = serde_json::from_str(include_str!(
        "../../../../../docs/contracts/api-v1/public-models.schema.json"
    ))
    .unwrap();
    let availability = public_models["$defs"]["AvailabilityResponse"]["required"]
        .as_array()
        .unwrap();
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/scheduling-availability.json")).unwrap();
    for field in availability {
        assert!(fixture["response"].get(field.as_str().unwrap()).is_some());
    }
    let response: crate::components::scheduling::adapters::api::RemoteAvailabilityResponse =
        serde_json::from_value(fixture["response"].clone()).unwrap();
    assert_eq!(response.requested_duration_minutes, 45);
    assert_eq!(response.available_intervals.len(), 2);
    let query = &fixture["query"];
    for field in ["professionalId", "serviceId", "from", "until"] {
        assert!(query.get(field).is_some(), "query {field}");
    }
}

#[test]
fn contract_bundle_fingerprints_stay_intact() {
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../../../docs/contracts/api-v1/manifest.json"
    ))
    .unwrap();
    for (file, fingerprint) in manifest["files"].as_object().unwrap() {
        let bytes = match file.as_str() {
            "accepted-operations.schema.json" => include_bytes!(
                "../../../../../docs/contracts/api-v1/accepted-operations.schema.json"
            )
            .as_slice(),
            "capabilities.json" => {
                include_bytes!("../../../../../docs/contracts/api-v1/capabilities.json").as_slice()
            }
            "openapi.json" => {
                include_bytes!("../../../../../docs/contracts/api-v1/openapi.json").as_slice()
            }
            "public-models.schema.json" => {
                include_bytes!("../../../../../docs/contracts/api-v1/public-models.schema.json")
                    .as_slice()
            }
            "sse.schema.json" => {
                include_bytes!("../../../../../docs/contracts/api-v1/sse.schema.json").as_slice()
            }
            other => panic!("unexpected bundle file {other}"),
        };
        let digest = format!("{:x}", Sha256::digest(bytes));
        assert_eq!(&digest, fingerprint, "fingerprint drift in {file}");
    }
}

#[test]
fn appointments_actions_payload_models_and_keys_match_fixed_bundle() {
    use crate::components::appointments::domain as appointments_domain;

    let capabilities: Value = serde_json::from_str(include_str!(
        "../../../../../docs/contracts/api-v1/capabilities.json"
    ))
    .unwrap();
    let schema: Value = serde_json::from_str(include_str!(
        "../../../../../docs/contracts/api-v1/accepted-operations.schema.json"
    ))
    .unwrap();
    let fixtures: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/appointments-operations.json")).unwrap();
    assert_eq!(fixtures.len(), 6);
    let mut actions = std::collections::BTreeSet::new();
    for fixture in fixtures {
        let operation = &fixture["operation"];
        let action = operation["action"].as_str().unwrap();
        assert!(actions.insert(action.to_owned()));
        assert!(
            capabilities["operationCatalog"]
                .as_array()
                .unwrap()
                .contains(&json!(action))
        );
        let model = capabilities["payloadModels"][action].as_str().unwrap();
        assert_eq!(model.rsplit('.').next().unwrap(), fixture["payloadModel"]);
        check(&schema, &schema, operation, action).unwrap();
        let envelope: OperationEnvelope = serde_json::from_value(operation.clone()).unwrap();
        assert_eq!(serde_json::to_value(envelope).unwrap(), *operation);
        assert!(
            !capabilities["onlineOnlyActions"]
                .as_array()
                .unwrap()
                .contains(&json!(action)),
            "{action} must stay offline-capable"
        );
        crate::shared_kernel::id::canonical_uuid_v7(operation["entityKey"].as_str().unwrap())
            .unwrap();

        // The local domain accepts the same values the contract fixture carries.
        let payload = &operation["payload"];
        match action {
            "CreateAppointment" => {
                appointments_domain::AppointmentInterval::new(
                    appointments_domain::MinuteInstant::parse(
                        payload["startsAt"].as_str().unwrap(),
                    )
                    .unwrap(),
                    appointments_domain::MinuteInstant::parse(payload["endsAt"].as_str().unwrap())
                        .unwrap(),
                )
                .unwrap();
                crate::components::patients::domain::PatientId::try_from(
                    payload["patientId"].as_str().unwrap().to_owned(),
                )
                .unwrap();
            }
            "RescheduleAppointment" => {
                appointments_domain::AppointmentInterval::new(
                    appointments_domain::MinuteInstant::parse(
                        payload["startsAt"].as_str().unwrap(),
                    )
                    .unwrap(),
                    appointments_domain::MinuteInstant::parse(payload["endsAt"].as_str().unwrap())
                        .unwrap(),
                )
                .unwrap();
            }
            "CancelAppointment" | "CompleteAppointment" | "MarkAppointmentNoShow" => {
                appointments_domain::ChangeReason::new(
                    payload["changeReason"].as_str().map(str::to_owned),
                )
                .unwrap();
            }
            "ConfirmAppointment" => {
                assert!(payload.as_object().unwrap().is_empty());
            }
            _ => {}
        }
        let mut invalid = operation.clone();
        invalid["payload"]["unexpected"] = json!(true);
        assert!(check(&schema, &schema, &invalid, action).is_err());
    }
}

#[test]
fn appointment_history_and_states_match_fixed_bundle() {
    use crate::components::sync::adapters::remote_models::RemoteState;

    let public_models: Value = serde_json::from_str(include_str!(
        "../../../../../docs/contracts/api-v1/public-models.schema.json"
    ))
    .unwrap();
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/appointments-history.json")).unwrap();

    // AppointmentResponse and its aggregated history parse with the typed
    // models and expose no structural drift against the fixed bundle.
    for (model, value) in [
        ("AppointmentResponse", &fixture["appointment"]),
        (
            "PaginatedResponse_AppointmentResponse_",
            &json!({"items": [fixture["appointment"].clone()], "page": 1, "size": 1, "total": 1}),
        ),
    ] {
        let required = public_models["$defs"][model]["required"]
            .as_array()
            .unwrap();
        for field in required {
            assert!(
                value.get(field.as_str().unwrap()).is_some(),
                "{model} requires {field}"
            );
        }
    }
    let parsed: RemoteState = serde_json::from_value(json!({
        "entityType": "appointment",
        "state": fixture["appointment"].clone()
    }))
    .unwrap();
    assert_eq!(parsed.version(), 2);
    assert!(parsed.key().is_string());

    let history: RemoteState = serde_json::from_value(json!({
        "entityType": "appointment_history",
        "state": fixture["history"].clone()
    }))
    .unwrap();
    assert_eq!(history.version(), 2);
    let required = public_models["$defs"]["AppointmentHistoryProjection"]["required"]
        .as_array()
        .unwrap();
    for field in required {
        assert!(
            fixture["history"].get(field.as_str().unwrap()).is_some(),
            "AppointmentHistoryProjection requires {field}"
        );
    }

    // The aggregated items stay consecutive from 1 and end at the aggregate.
    if let RemoteState::AppointmentHistory(projection) = history {
        for (index, item) in projection.items.iter().enumerate() {
            assert_eq!(item.version, (index as i64) + 1);
        }
        assert_eq!(projection.items.last().unwrap().version, projection.version);
        assert_eq!(projection.items.len(), 2);
    } else {
        panic!("expected appointment history projection");
    }
}
