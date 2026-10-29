use crate::shared_kernel::{NightingaleError, Result};
use serde_json::Value;

/// Contract version accepted by Nightingale API v1 envelopes.
pub const CONTRACT_VERSION: i64 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationStatus {
    Pending,
    Sent,
    Failed,
    Conflict,
}

impl OperationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Sent => "sent",
            Self::Failed => "failed",
            Self::Conflict => "conflict",
        }
    }
}

/// Input for recording a local operation. It holds exactly what the contract
/// envelope needs to send later; the server assigns actor, state and results.
#[derive(Debug, Clone)]
pub struct NewOperation {
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
        assert_eq!(OperationStatus::Sent.as_str(), "sent");
        assert_eq!(OperationStatus::Failed.as_str(), "failed");
        assert_eq!(OperationStatus::Conflict.as_str(), "conflict");
    }

    fn operation_with(entity_key: Value) -> NewOperation {
        NewOperation {
            entity_key,
            ..operation()
        }
    }
}
