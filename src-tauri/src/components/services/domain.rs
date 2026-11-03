//! Catalog rules and values, independent of storage and remote transport.
use crate::shared_kernel::{NightingaleError, Result, id::canonical_uuid_v7};

macro_rules! catalog_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name(String);
        impl TryFrom<String> for $name {
            type Error = NightingaleError;
            fn try_from(value: String) -> Result<Self> {
                canonical_uuid_v7(&value)?;
                Ok(Self(value))
            }
        }
        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

catalog_id!(ServiceId);
catalog_id!(ProfessionalId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfferKey {
    pub professional_id: ProfessionalId,
    pub service_id: ServiceId,
}

impl OfferKey {
    pub fn new(professional: String, service: String) -> Result<Self> {
        Ok(Self {
            professional_id: professional.try_into()?,
            service_id: service.try_into()?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DurationMinutes(i64);

impl DurationMinutes {
    pub fn new(value: i64) -> Result<Self> {
        if !(1..=1440).contains(&value) {
            return Err(NightingaleError::validation(
                "la duración debe ser un entero entre 1 y 1440",
            ));
        }
        Ok(Self(value))
    }

    pub fn get(self) -> i64 {
        self.0
    }

    pub fn effective(base: Self, adjustment: Option<Self>) -> Self {
        adjustment.unwrap_or(base)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Active,
    Inactive,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
        }
    }
}

impl TryFrom<&str> for Status {
    type Error = NightingaleError;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            _ => Err(NightingaleError::validation("estado de catálogo inválido")),
        }
    }
}

pub fn validate_name(value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(NightingaleError::validation(
            "el nombre no puede estar vacío",
        ));
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct ServicePatch {
    pub name: Option<String>,
    pub default_duration_minutes: Option<DurationMinutes>,
}

impl ServicePatch {
    pub fn new(name: Option<String>, duration: Option<i64>) -> Result<Self> {
        if name.is_none() && duration.is_none() {
            return Err(NightingaleError::validation(
                "el cambio de servicio está vacío",
            ));
        }

        if let Some(name) = &name {
            validate_name(name)?;
        }

        Ok(Self {
            name,
            default_duration_minutes: duration.map(DurationMinutes::new).transpose()?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ServiceOfferPatch {
    pub duration_minutes_override: Option<DurationMinutes>,
}

impl ServiceOfferPatch {
    pub fn new(value: Option<Option<i64>>) -> Result<Self> {
        let value = value.ok_or_else(|| {
            NightingaleError::validation("durationMinutesOverride debe estar presente")
        })?;

        Ok(Self {
            duration_minutes_override: value.map(DurationMinutes::new).transpose()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_and_effective_values_have_exact_bounds() {
        for value in [1, 1440] {
            assert_eq!(DurationMinutes::new(value).unwrap().get(), value);
        }
        for value in [-1, 0, 1441] {
            assert!(DurationMinutes::new(value).is_err());
        }
        let base = DurationMinutes::new(30).unwrap();
        assert_eq!(DurationMinutes::effective(base, None).get(), 30);
        assert_eq!(
            DurationMinutes::effective(base, Some(DurationMinutes::new(15).unwrap())).get(),
            15
        );
    }

    #[test]
    fn patches_distinguish_missing_null_and_value() {
        assert!(ServicePatch::new(None, None).is_err());
        assert!(ServicePatch::new(Some(" ".into()), None).is_err());
        assert!(ServicePatch::new(None, Some(0)).is_err());
        assert!(ServicePatch::new(Some("Synthetic".into()), None).is_ok());
        assert!(ServiceOfferPatch::new(None).is_err());
        assert!(
            ServiceOfferPatch::new(Some(None))
                .unwrap()
                .duration_minutes_override
                .is_none()
        );
        assert_eq!(
            ServiceOfferPatch::new(Some(Some(1)))
                .unwrap()
                .duration_minutes_override
                .unwrap()
                .get(),
            1
        );
    }

    #[test]
    fn both_offer_ids_must_be_canonical_uuid_v7() {
        let id = "019a1234-0000-7000-8000-000000000001";
        assert!(OfferKey::new(id.into(), id.into()).is_ok());
        for invalid in [
            "019A1234-0000-7000-8000-000000000001",
            "019a1234-0000-4000-8000-000000000001",
            "bad",
        ] {
            assert!(OfferKey::new(invalid.into(), id.into()).is_err());
            assert!(OfferKey::new(id.into(), invalid.into()).is_err());
        }
    }
}
