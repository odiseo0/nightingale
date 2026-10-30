use crate::shared_kernel::{NightingaleError, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn canonical_uuid_v7(value: &str) -> Result<Uuid> {
    let id = Uuid::try_parse(value).map_err(|_| NightingaleError::validation("UUIDv7 inválido"))?;

    if id.get_version_num() != 7
        || id.get_variant() != uuid::Variant::RFC4122
        || id.to_string() != value
    {
        return Err(NightingaleError::validation(
            "se requiere un UUIDv7 canónico",
        ));
    }

    Ok(id)
}

macro_rules! typed_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl TryFrom<String> for $name {
            type Error = NightingaleError;
            fn try_from(value: String) -> Result<Self> {
                canonical_uuid_v7(&value)?;
                Ok(Self(value))
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

typed_id!(AccountId);
typed_id!(UserId);
typed_id!(DeviceId);
typed_id!(OperationId);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OperationContext {
    pub account_id: AccountId,
    pub user_id: UserId,
    pub device_id: DeviceId,
    pub permission_epoch: i64,
}

// Port so tests and offline paths can control identifier generation
// (Master Plan §11.4).
pub trait IdGenerator {
    fn new_id(&self) -> Uuid;
}

/// UUIDv7 canonical identifiers, as required by the Nightingale API contract.
/// Uuid::now_v7 guarantees process-wide creation order.
/// Source: https://docs.rs/uuid/latest/uuid/struct.Uuid.html#method.now_v7
pub struct UuidV7Generator;

impl IdGenerator for UuidV7Generator {
    fn new_id(&self) -> Uuid {
        Uuid::now_v7()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_distinct_ordered_ids() {
        let generator = UuidV7Generator;
        let first = generator.new_id();
        let second = generator.new_id();

        assert_ne!(first, second);
        assert!(second > first);
        assert_eq!(Some(uuid::Version::SortRand), first.get_version());
    }

    #[test]
    fn context_round_trip_uses_typed_canonical_ids() {
        let value = serde_json::json!({
            "accountId":"019a1234-0000-7000-8000-000000000001",
            "userId":"019a1234-0000-7000-8000-000000000002",
            "deviceId":"019a1234-0000-7000-8000-000000000003",
            "permissionEpoch": 4
        });
        let context: OperationContext = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(context).unwrap(), value);
        let mut bad = value;
        bad["deviceId"] = serde_json::json!("not-a-device");
        assert!(serde_json::from_value::<OperationContext>(bad).is_err());
    }
}
