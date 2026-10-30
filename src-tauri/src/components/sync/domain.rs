use crate::shared_kernel::id::{OperationId, canonical_uuid_v7};
use crate::shared_kernel::{NightingaleError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// Wire body only. Route and credential context stay outside this envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OperationEnvelope {
    pub operation_id: OperationId,
    pub contract_version: i64,
    pub action: String,
    pub entity_key: Value,
    pub expected_version: i64,
    pub payload: Value,
    #[serde(default)]
    pub depends_on: Vec<OperationId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occurred_at_local: Option<String>,
}

impl OperationEnvelope {
    pub fn validate(&self) -> Result<()> {
        if self.contract_version != CONTRACT_VERSION
            || self.action.is_empty()
            || self.expected_version < 0
            || !self.payload.is_object()
        {
            return Err(NightingaleError::validation(
                "envoltorio de operación inválido",
            ));
        }

        match &self.entity_key {
            Value::String(id) => {
                canonical_uuid_v7(id)?;
            }
            Value::Object(key) => {
                for value in key.values() {
                    canonical_uuid_v7(value.as_str().ok_or_else(|| {
                        NightingaleError::validation("clave compuesta inválida")
                    })?)?;
                }
            }
            _ => return Err(NightingaleError::validation("entityKey inválido")),
        }

        if self.depends_on.len() > 8
            || self
                .depends_on
                .iter()
                .enumerate()
                .any(|(i, id)| id == &self.operation_id || self.depends_on[..i].contains(id))
        {
            return Err(NightingaleError::validation("dependsOn inválido"));
        }

        Ok(())
    }

    pub fn canonical_json(&self) -> Result<Vec<u8>> {
        self.validate()?;
        // Struct order fixes envelope fields; Value objects use sorted keys.
        // https://docs.rs/serde_json/1.0.154/serde_json/
        let bytes = serde_json::to_vec(self)
            .map_err(|_| NightingaleError::internal("no se pudo serializar la operación"))?;

        if bytes.len() > 64 * 1024 {
            return Err(NightingaleError::validation("operación mayor de 64 KiB"));
        }

        Ok(bytes)
    }

    pub fn semantic_fingerprint(&self) -> Result<String> {
        // https://docs.rs/sha2/0.10.9/sha2/#one-shot-api
        Ok(format!("{:x}", Sha256::digest(self.canonical_json()?)))
    }
}

/// Contract version accepted by Nightingale API v1 envelopes.
pub const CONTRACT_VERSION: i64 = 1;

#[derive(Debug, Clone, PartialEq)]
pub struct StoredCursor {
    pub cursor: String,
    pub window: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RemoteItem {
    pub entity_type: String,
    pub entity_key: Value,
    pub entity_version: i64,
    pub state: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationStatus {
    Pending,
    InFlight,
    Unknown,
    Retryable,
    Accepted,
    Conflict,
    Rejected,
}

impl OperationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InFlight => "in_flight",
            Self::Unknown => "unknown",
            Self::Retryable => "retryable",
            Self::Accepted => "accepted",
            Self::Conflict => "conflict",
            Self::Rejected => "rejected",
        }
    }
}

/// Input for recording a local operation. It holds exactly what the contract
/// envelope needs to send later; the server assigns actor, state and results.
#[derive(Debug, Clone)]
pub struct NewOperation {
    pub context: crate::shared_kernel::id::OperationContext,
    pub depends_on: Vec<OperationId>,
    pub action: String,
    pub entity_type: String,
    pub entity_key: Value,
    pub expected_version: i64,
    pub payload: Value,
    pub occurred_at_local: Option<String>,
}

impl NewOperation {
    pub fn validate(&self) -> Result<()> {
        if self.action.is_empty() {
            return Err(NightingaleError::validation(
                "la acción de la operación no puede estar vacía",
            ));
        }

        if self.entity_type.is_empty() {
            return Err(NightingaleError::validation(
                "el tipo de entidad no puede estar vacío",
            ));
        }

        if self.expected_version < 0 {
            return Err(NightingaleError::validation(
                "expected_version no puede ser negativo",
            ));
        }

        // Contract: entityKey is a UUID string or a composite object, payload is an object.
        if !self.entity_key.is_string() && !self.entity_key.is_object() {
            return Err(NightingaleError::validation(
                "entity_key debe ser un UUID o un objeto de clave compuesta",
            ));
        }

        if !self.payload.is_object() {
            return Err(NightingaleError::validation(
                "el payload debe ser un objeto JSON",
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn operation() -> NewOperation {
        NewOperation {
            context: serde_json::from_value(json!({
                "accountId":"019a1234-0000-7000-8000-000000000001",
                "userId":"019a1234-0000-7000-8000-000000000002",
                "deviceId":"019a1234-0000-7000-8000-000000000003",
                "permissionEpoch":1
            }))
            .unwrap(),
            depends_on: vec![],
            action: "RegisterPatient".to_owned(),
            entity_type: "patient_profile".to_owned(),
            entity_key: json!("019f0000-0000-7000-8000-000000000101"),
            expected_version: 0,
            payload: json!({ "fullName": "Paciente de prueba" }),
            occurred_at_local: None,
        }
    }

    #[test]
    fn accepts_valid_operation() {
        assert!(operation().validate().is_ok());
        assert!(
            operation_with(json!({"patientId": "a", "contactPersonId": "b"}))
                .validate()
                .is_ok()
        );
    }

    #[test]
    fn rejects_empty_action_and_entity_type() {
        let mut invalid = operation();
        invalid.action = String::new();
        assert!(invalid.validate().is_err());

        let mut invalid = operation();
        invalid.entity_type = String::new();
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn rejects_negative_expected_version() {
        let mut invalid = operation();
        invalid.expected_version = -1;

        assert!(invalid.validate().is_err());
    }

    #[test]
    fn rejects_non_object_payload_and_scalar_entity_key() {
        let mut invalid = operation();
        invalid.payload = json!("not-an-object");
        assert!(invalid.validate().is_err());

        let mut invalid = operation();
        invalid.entity_key = json!(42);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn status_serializes_to_stable_names() {
        assert_eq!(OperationStatus::Pending.as_str(), "pending");
        assert_eq!(OperationStatus::InFlight.as_str(), "in_flight");
        assert_eq!(OperationStatus::Unknown.as_str(), "unknown");
        assert_eq!(OperationStatus::Retryable.as_str(), "retryable");
        assert_eq!(OperationStatus::Accepted.as_str(), "accepted");
        assert_eq!(OperationStatus::Conflict.as_str(), "conflict");
        assert_eq!(OperationStatus::Rejected.as_str(), "rejected");
    }

    fn envelope() -> OperationEnvelope {
        serde_json::from_value(json!({
            "operationId": "019a1234-0000-7000-8000-000000000001",
            "contractVersion": 1,
            "action": "RegisterPatient",
            "entityKey": "019a1234-0000-7000-8000-000000000002",
            "expectedVersion": 0,
            "payload": {"fullName": "Paciente de prueba"},
            "dependsOn": []
        }))
        .unwrap()
    }

    #[test]
    fn contract_example_round_trip_and_no_local_fields() {
        let operation = envelope();
        let bytes = operation.canonical_json().unwrap();
        assert_eq!(
            operation.semantic_fingerprint().unwrap(),
            "9b1dd4d7615adf6b983740641cf12abee5d7d12e329d3b1053d6a9b194d87ce2"
        );
        assert_eq!(
            serde_json::from_slice::<OperationEnvelope>(&bytes).unwrap(),
            operation
        );
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body.as_object().unwrap().len(), 7);
        assert!(body.get("entityType").is_none());
        assert!(body.get("accountId").is_none());
        let schema: Value = serde_json::from_str(include_str!(
            "../../../../docs/contracts/api-v1/openapi.json"
        ))
        .unwrap();
        let properties = &schema["components"]["schemas"]["OperationEnvelope"]["properties"];
        for field in body.as_object().unwrap().keys() {
            assert!(properties.get(field).is_some(), "{field}");
        }
        let mut extra = body;
        extra["accountId"] = json!("extra");
        assert!(serde_json::from_value::<OperationEnvelope>(extra).is_err());
    }

    #[test]
    fn fingerprint_sorts_nested_objects_and_keeps_array_order_and_null() {
        let mut first = envelope();
        first.payload = serde_json::from_str(r#"{"z":[{"b":2,"a":1}],"a":null}"#).unwrap();
        let mut second = first.clone();
        second.payload = serde_json::from_str(r#"{"a":null,"z":[{"a":1,"b":2}]}"#).unwrap();
        assert_eq!(
            first.semantic_fingerprint().unwrap(),
            second.semantic_fingerprint().unwrap()
        );
        assert_eq!(first.semantic_fingerprint().unwrap().len(), 64);
        second.payload = json!({"z": [{"a":1,"b":2}]});
        assert_ne!(
            first.semantic_fingerprint().unwrap(),
            second.semantic_fingerprint().unwrap()
        );
        first.payload = json!({"a":[1,2]});
        second.payload = json!({"a":[2,1]});
        assert_ne!(
            first.semantic_fingerprint().unwrap(),
            second.semantic_fingerprint().unwrap()
        );
    }

    #[test]
    fn fingerprint_covers_each_semantic_field() {
        let base = envelope();
        let fingerprint = base.semantic_fingerprint().unwrap();
        for field in [
            "operationId",
            "action",
            "entityKey",
            "expectedVersion",
            "payload",
            "dependsOn",
            "occurredAtLocal",
        ] {
            let mut changed = serde_json::to_value(&base).unwrap();
            changed[field] = match field {
                "operationId" | "entityKey" => json!("019a1234-0000-7000-8000-000000000003"),
                "action" => json!("UpdatePatientProfile"),
                "expectedVersion" => json!(1),
                "payload" => json!({"fullName":"Otro nombre"}),
                "dependsOn" => json!(["019a1234-0000-7000-8000-000000000003"]),
                _ => json!("2026-10-29T12:00:00-04:00"),
            };
            let changed: OperationEnvelope = serde_json::from_value(changed).unwrap();
            assert_ne!(
                fingerprint,
                changed.semantic_fingerprint().unwrap(),
                "{field}"
            );
        }
        let mut changed = base;
        changed.contract_version = 2;
        assert!(changed.canonical_json().is_err());
    }

    #[test]
    fn rejects_bad_ids_dependencies_versions_keys_and_size() {
        use crate::shared_kernel::{ErrorCode, id::AccountId};
        for id in [
            "bad",
            "019A1234-0000-7000-8000-000000000001",
            "019a1234000070008000000000000001",
            "019a1234-0000-4000-8000-000000000001",
            "019a1234-0000-7000-0000-000000000001",
        ] {
            assert_eq!(
                AccountId::try_from(id.to_owned()).unwrap_err().code,
                ErrorCode::ValidationError
            );
            assert!(serde_json::from_value::<OperationId>(json!(id)).is_err());
        }
        let mut operation = envelope();
        operation.depends_on = vec![operation.operation_id.clone()];
        assert!(operation.validate().is_err());
        let dependency: OperationId =
            serde_json::from_value(json!("019a1234-0000-7000-8000-000000000003")).unwrap();
        operation.depends_on = vec![dependency; 9];
        assert!(operation.validate().is_err());
        operation.depends_on.truncate(2);
        assert!(operation.validate().is_err());
        operation.depends_on.clear();
        operation.entity_key = json!({"patientId":"invalid"});
        assert!(operation.validate().is_err());
        operation.entity_key = json!({"patientId":"019a1234-0000-7000-8000-000000000003"});
        assert!(operation.validate().is_ok());
        operation.expected_version = -1;
        assert!(operation.validate().is_err());
        operation.expected_version = 0;
        operation.payload = json!({"text":"x".repeat(65536)});
        assert!(operation.canonical_json().is_err());
    }

    fn operation_with(entity_key: Value) -> NewOperation {
        NewOperation {
            entity_key,
            ..operation()
        }
    }
}
