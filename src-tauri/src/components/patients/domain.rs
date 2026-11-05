//! Administrative values and format rules. No storage or network access.
use crate::shared_kernel::{NightingaleError, Result, reference_data::is_country_code};
use chrono::{DateTime, FixedOffset, NaiveDate};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatientId(String);

impl TryFrom<String> for PatientId {
    type Error = NightingaleError;

    fn try_from(value: String) -> Result<Self> {
        crate::shared_kernel::id::canonical_uuid_v7(&value)?;

        Ok(Self(value))
    }
}

impl PatientId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

macro_rules! patient_value {
    ($name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name { $($variant),+ }

        impl $name {
            pub fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $text),+ }
            }
        }

        impl TryFrom<&str> for $name {
            type Error = NightingaleError;

            fn try_from(value: &str) -> Result<Self> {
                match value {
                    $($text => Ok(Self::$variant),)+
                    _ => Err(NightingaleError::validation("valor administrativo inválido")),
                }
            }
        }
    };
}

patient_value!(Status { Active => "active", Inactive => "inactive" });
patient_value!(CompletionStatus { Draft => "draft", Complete => "complete" });
patient_value!(IdentificationBasis {
    OwnDocument => "own_document", MinorWithoutDocument => "minor_without_document"
});
patient_value!(DocumentType { NationalId => "national_id", Passport => "passport" });
patient_value!(RelationshipKind {
    Mother => "mother", Father => "father", Guardian => "guardian", Other => "other"
});

/// A patch must retain the difference between an absent field and an explicit null.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FieldChange<T> {
    #[default]
    Keep,
    Clear,
    Set(T),
}

impl<T: Clone> FieldChange<T> {
    pub fn apply(&self, current: &Option<T>) -> Option<T> {
        match self {
            Self::Keep => current.clone(),
            Self::Clear => None,
            Self::Set(value) => Some(value.clone()),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    issuing_country: String,
    document_type: DocumentType,
    number: String,
}

impl Document {
    // Verification is absent from this input value. Only API can verify a document.
    pub fn new(country: String, kind: DocumentType, number: String) -> Result<Self> {
        if !is_country_code(&country) || number.trim().is_empty() {
            return Err(NightingaleError::validation("documento inválido"));
        }

        Ok(Self {
            issuing_country: country,
            document_type: kind,
            number,
        })
    }

    pub fn issuing_country(&self) -> &str {
        &self.issuing_country
    }

    pub fn document_type(&self) -> DocumentType {
        self.document_type
    }

    pub fn number(&self) -> &str {
        &self.number
    }

    /// Contact documents must contain all three fields or none.
    pub fn optional(
        country: Option<String>,
        kind: Option<DocumentType>,
        number: Option<String>,
    ) -> Result<Option<Self>> {
        match (country, kind, number) {
            (None, None, None) => Ok(None),
            (Some(country), Some(kind), Some(number)) => Self::new(country, kind, number).map(Some),
            _ => Err(NightingaleError::validation(
                "documento de contacto incompleto",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phone {
    code: String,
    number: String,
}

impl Phone {
    pub fn new(code: String, number: String) -> Result<Self> {
        let prefix = code.strip_prefix('+').unwrap_or("");

        if !(1..=3).contains(&prefix.len())
            || prefix.starts_with('0')
            || !prefix.bytes().all(|b| b.is_ascii_digit())
            || number.is_empty()
            || !number.bytes().all(|b| b.is_ascii_digit())
            || prefix.len() + number.len() > 15
        {
            return Err(NightingaleError::validation("teléfono inválido"));
        }

        Ok(Self { code, number })
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn number(&self) -> &str {
        &self.number
    }

    pub fn optional(code: Option<String>, number: Option<String>) -> Result<Option<Self>> {
        match (code, number) {
            (None, None) => Ok(None),
            (Some(code), Some(number)) => Self::new(code, number).map(Some),
            _ => Err(NightingaleError::validation(
                "se requiere la pareja telefónica",
            )),
        }
    }
}

/// Parse the contract's exact YYYY-MM-DD shape as well as the calendar date.
/// Source: https://docs.rs/chrono/0.4.45/chrono/naive/struct.NaiveDate.html#method.parse_from_str
pub fn parse_date(value: &str) -> Result<NaiveDate> {
    let bytes = value.as_bytes();

    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
    {
        return Err(NightingaleError::validation("fecha inválida"));
    }

    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| NightingaleError::validation("fecha inválida"))
}

/// An explicit offset makes comparisons independent of the device time zone.
/// Source: https://docs.rs/chrono/0.4.45/chrono/struct.DateTime.html#method.parse_from_rfc3339
pub fn parse_instant(value: &str) -> Result<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(value)
        .map_err(|_| NightingaleError::validation("instante inválido"))
}

pub fn validate_validity(from: &str, until: Option<&str>) -> Result<()> {
    let from = parse_instant(from)?;

    if let Some(until) = until
        && parse_instant(until)? <= from
    {
        return Err(NightingaleError::validation("vigencia inválida"));
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProfileData {
    pub full_name: Option<String>,
    pub birth_date: Option<String>,
    pub phone: Option<Phone>,
    pub identification_basis: Option<IdentificationBasis>,
}

impl ProfileData {
    pub fn validate(&self) -> Result<()> {
        if let Some(name) = &self.full_name {
            validate_name(name)?;
        }

        if let Some(date) = &self.birth_date {
            parse_date(date)?;
        }

        Ok(())
    }

    /// Administrative readiness now. Appointment age rules belong to appointments.
    pub fn meets_completion(&self, has_own_document: bool, has_responsible_phone: bool) -> bool {
        self.full_name
            .as_ref()
            .is_some_and(|v| !v.trim().is_empty())
            && match self.identification_basis {
                Some(IdentificationBasis::OwnDocument) => {
                    has_own_document && (self.phone.is_some() || has_responsible_phone)
                }
                Some(IdentificationBasis::MinorWithoutDocument) => {
                    self.birth_date.is_some() && has_responsible_phone
                }
                None => false,
            }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactPersonId(String);

impl TryFrom<String> for ContactPersonId {
    type Error = NightingaleError;

    fn try_from(value: String) -> Result<Self> {
        crate::shared_kernel::id::canonical_uuid_v7(&value)?;

        Ok(Self(value))
    }
}

impl ContactPersonId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Contract ContactData. A contact belongs to the account and needs no patient profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactData {
    pub full_name: String,
    pub phone: Option<Phone>,
    pub document: Option<Document>,
}

impl ContactData {
    pub fn new(
        full_name: String,
        phone_code: Option<String>,
        phone: Option<String>,
        issuing_country: Option<String>,
        document_type: Option<DocumentType>,
        document_number: Option<String>,
    ) -> Result<Self> {
        let data = Self {
            full_name,
            phone: Phone::optional(phone_code, phone)?,
            document: Document::optional(issuing_country, document_type, document_number)?,
        };
        validate_name(&data.full_name)?;

        Ok(data)
    }
}

/// ContactPatch keeps absence and null apart; null cannot remove the mandatory name.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContactPatch {
    pub full_name: FieldChange<String>,
    pub phone_code: FieldChange<String>,
    pub phone: FieldChange<String>,
    pub issuing_country: FieldChange<String>,
    pub document_type: FieldChange<DocumentType>,
    pub document_number: FieldChange<String>,
}

impl ContactPatch {
    pub fn apply(&self, current: &ContactData) -> Result<ContactData> {
        if self == &Self::default() {
            return Err(NightingaleError::validation(
                "el cambio de contacto está vacío",
            ));
        }
        if matches!(self.full_name, FieldChange::Clear) {
            return Err(NightingaleError::validation(
                "el nombre del contacto es obligatorio",
            ));
        }

        let code = current.phone.as_ref().map(|p| p.code.clone());
        let number = current.phone.as_ref().map(|p| p.number.clone());
        let country = current
            .document
            .as_ref()
            .map(|d| d.issuing_country().to_owned());
        let kind = current.document.as_ref().map(|d| d.document_type());
        let doc_number = current.document.as_ref().map(|d| d.number().to_owned());
        let data = ContactData {
            full_name: match &self.full_name {
                FieldChange::Set(value) => value.clone(),
                FieldChange::Keep | FieldChange::Clear => current.full_name.clone(),
            },
            phone: Phone::optional(self.phone_code.apply(&code), self.phone.apply(&number))?,
            document: Document::optional(
                self.issuing_country.apply(&country),
                self.document_type.apply(&kind),
                self.document_number.apply(&doc_number),
            )?,
        };
        validate_name(&data.full_name)?;

        Ok(data)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationshipData {
    pub relationship_kind: RelationshipKind,
    pub is_responsible_contact: bool,
    pub valid_from: String,
    pub valid_until: Option<String>,
}

impl RelationshipData {
    /// The client declares kinship, responsibility and validity only; representation
    /// verification belongs to accepted state and never enters this input.
    pub fn new(
        relationship_kind: RelationshipKind,
        is_responsible_contact: bool,
        valid_from: String,
        valid_until: Option<String>,
    ) -> Result<Self> {
        validate_validity(&valid_from, valid_until.as_deref())?;

        Ok(Self {
            relationship_kind,
            is_responsible_contact,
            valid_from,
            valid_until,
        })
    }
}

/// RelationshipPatch: null only retires validUntil; the other fields cannot be cleared.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RelationshipPatch {
    pub relationship_kind: FieldChange<RelationshipKind>,
    pub is_responsible_contact: FieldChange<bool>,
    pub valid_from: FieldChange<String>,
    pub valid_until: FieldChange<String>,
}

impl RelationshipPatch {
    pub fn apply(&self, current: &RelationshipData) -> Result<RelationshipData> {
        if self == &Self::default() {
            return Err(NightingaleError::validation(
                "el cambio de vínculo está vacío",
            ));
        }

        if matches!(self.valid_from, FieldChange::Clear) {
            return Err(NightingaleError::validation(
                "la vigencia inicial del vínculo es obligatoria",
            ));
        }

        if matches!(self.relationship_kind, FieldChange::Clear) {
            return Err(NightingaleError::validation(
                "el parentesco del vínculo es obligatorio",
            ));
        }

        if matches!(self.is_responsible_contact, FieldChange::Clear) {
            return Err(NightingaleError::validation(
                "la responsabilidad del vínculo no puede retirarse",
            ));
        }

        let data = RelationshipData {
            relationship_kind: self
                .relationship_kind
                .apply(&Some(current.relationship_kind))
                .ok_or_else(|| NightingaleError::validation("parentesco inválido"))?,
            is_responsible_contact: self
                .is_responsible_contact
                .apply(&Some(current.is_responsible_contact))
                .unwrap_or_default(),
            valid_from: self
                .valid_from
                .apply(&Some(current.valid_from.clone()))
                .ok_or_else(|| NightingaleError::validation("vigencia inicial inválida"))?,
            valid_until: self.valid_until.apply(&current.valid_until),
        };
        validate_validity(&data.valid_from, data.valid_until.as_deref())?;

        Ok(data)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProfilePatch {
    pub full_name: FieldChange<String>,
    pub birth_date: FieldChange<String>,
    pub phone_code: FieldChange<String>,
    pub phone: FieldChange<String>,
    pub identification_basis: FieldChange<IdentificationBasis>,
}

impl ProfilePatch {
    pub fn apply(&self, current: &ProfileData) -> Result<ProfileData> {
        if self == &Self::default() {
            return Err(NightingaleError::validation(
                "el cambio de ficha está vacío",
            ));
        }

        let code = current.phone.as_ref().map(|p| p.code.clone());
        let number = current.phone.as_ref().map(|p| p.number.clone());
        let result = ProfileData {
            full_name: self.full_name.apply(&current.full_name),
            birth_date: self.birth_date.apply(&current.birth_date),
            phone: Phone::optional(self.phone_code.apply(&code), self.phone.apply(&number))?,
            identification_basis: self
                .identification_basis
                .apply(&current.identification_basis),
        };
        result.validate()?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activity_and_completion_are_separate_closed_values() {
        assert_eq!(Status::try_from("inactive").unwrap().as_str(), "inactive");
        assert_eq!(
            CompletionStatus::try_from("complete").unwrap().as_str(),
            "complete"
        );
        assert!(Status::try_from("draft").is_err());
        assert!(CompletionStatus::try_from("active").is_err());
        assert!(IdentificationBasis::try_from("guardian_document").is_err());
        assert!(RelationshipKind::try_from("verified_guardian").is_err());
        assert!(DocumentType::try_from("other").is_err());
    }

    #[test]
    fn documents_keep_prefixes_zeroes_and_punctuation() {
        let document =
            Document::new("VE".into(), DocumentType::NationalId, "V-00.012-3".into()).unwrap();
        assert_eq!(document.number(), "V-00.012-3");
        assert_eq!(document.issuing_country(), "VE");
        assert_eq!(document.document_type(), DocumentType::NationalId);
        for country in ["ZZ", "ve", "VEN", ""] {
            assert!(Document::new(country.into(), DocumentType::Passport, "001".into()).is_err());
        }
        assert!(Document::new("VE".into(), DocumentType::Passport, "  ".into()).is_err());
    }

    #[test]
    fn contact_document_is_complete_or_absent() {
        assert_eq!(Document::optional(None, None, None).unwrap(), None);
        assert!(Document::optional(Some("VE".into()), None, Some("001".into())).is_err());
        assert!(
            Document::optional(
                Some("VE".into()),
                Some(DocumentType::Passport),
                Some("001".into())
            )
            .unwrap()
            .is_some()
        );
    }

    #[test]
    fn phone_requires_both_parts_and_ascii_digits() {
        assert_eq!(Phone::optional(None, None).unwrap(), None);
        assert!(Phone::optional(Some("+58".into()), None).is_err());
        assert!(Phone::optional(None, Some("4120000000".into())).is_err());
        let phone = Phone::new("+58".into(), "04120000000".into()).unwrap();
        assert_eq!((phone.code(), phone.number()), ("+58", "04120000000"));
        for (code, number) in [
            ("58", "123"),
            ("+0", "123"),
            ("+1234", "123"),
            ("+58", ""),
            ("+58", "１２３"),
            ("+58", "1-23"),
            ("+58", "12345678901234"),
        ] {
            assert!(
                Phone::new(code.into(), number.into()).is_err(),
                "{code} {number}"
            );
        }
        assert!(Phone::new("+1".into(), "12345678901234".into()).is_ok());
    }

    #[test]
    fn dates_check_shape_and_leap_years() {
        assert!(parse_date("2024-02-29").is_ok());
        for date in [
            "2023-02-29",
            "2026-04-31",
            "2026-1-01",
            "26-01-01",
            "2026-01-01Z",
            "2026-00-01",
        ] {
            assert!(parse_date(date).is_err(), "{date}");
        }
    }

    #[test]
    fn validity_compares_instants_with_offsets() {
        assert!(
            validate_validity("2026-11-04T10:00:00-04:00", Some("2026-11-04T14:00:01Z")).is_ok()
        );
        assert!(validate_validity("2026-11-04T10:00:00-04:00", None).is_ok());
        assert!(
            validate_validity("2026-11-04T10:00:00-04:00", Some("2026-11-04T14:00:00Z")).is_err()
        );
        assert!(validate_validity("2026-11-04T10:00:00", None).is_err());
        assert!(validate_validity("2026-11-04T10:00:00Z", Some("invalid")).is_err());
    }

    #[test]
    fn draft_accepts_missing_fields_but_not_invalid_present_fields() {
        assert!(ProfileData::default().validate().is_ok());
        let minor = ProfileData {
            full_name: Some("Menor ficticio".into()),
            birth_date: Some("2018-02-28".into()),
            identification_basis: Some(IdentificationBasis::MinorWithoutDocument),
            phone: None,
        };
        assert!(minor.validate().is_ok());
        assert!(
            ProfileData {
                full_name: Some(" ".into()),
                ..minor.clone()
            }
            .validate()
            .is_err()
        );
        assert!(
            ProfileData {
                birth_date: Some("2018-02-29".into()),
                ..minor
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn relationship_data_validates_validity_and_patch_rules() {
        let original = RelationshipData::new(
            RelationshipKind::Mother,
            true,
            "2026-11-04T10:00:00-04:00".into(),
            None,
        )
        .unwrap();
        assert!(
            RelationshipData::new(
                RelationshipKind::Guardian,
                false,
                "2026-11-04T10:00:00-04:00".into(),
                Some("2026-11-04T14:00:00Z".into())
            )
            .is_err()
        );

        assert!(RelationshipPatch::default().apply(&original).is_err());
        assert!(
            RelationshipPatch {
                valid_from: FieldChange::Clear,
                ..RelationshipPatch::default()
            }
            .apply(&original)
            .is_err()
        );
        assert!(
            RelationshipPatch {
                relationship_kind: FieldChange::Clear,
                ..RelationshipPatch::default()
            }
            .apply(&original)
            .is_err()
        );
        assert!(
            RelationshipPatch {
                is_responsible_contact: FieldChange::Clear,
                ..RelationshipPatch::default()
            }
            .apply(&original)
            .is_err()
        );
        // Responsibility changes by value only, never by null.
        let delegated = RelationshipPatch {
            is_responsible_contact: FieldChange::Set(false),
            ..RelationshipPatch::default()
        }
        .apply(&original)
        .unwrap();
        assert!(!delegated.is_responsible_contact);
        // validUntil accepts a new value, keeps the current one or is retired by null.
        let moved = RelationshipPatch {
            valid_until: FieldChange::Set("2027-01-01T00:00:00-04:00".into()),
            ..RelationshipPatch::default()
        }
        .apply(&original)
        .unwrap();
        assert_eq!(
            moved.valid_until.as_deref(),
            Some("2027-01-01T00:00:00-04:00")
        );
        let retired = RelationshipPatch {
            valid_until: FieldChange::Clear,
            ..RelationshipPatch::default()
        }
        .apply(&original)
        .unwrap();
        assert_eq!(retired.valid_until, None);
        assert_eq!(retired.relationship_kind, RelationshipKind::Mother);
        let invalid = RelationshipPatch {
            valid_until: FieldChange::Set("2026-11-04T09:00:00-04:00".into()),
            ..RelationshipPatch::default()
        };
        assert!(invalid.apply(&original).is_err());
    }

    #[test]
    fn contact_data_keeps_pairs_and_triples_or_rejects_them() {
        let contact = ContactData::new(
            "Responsable ficticio".into(),
            Some("+58".into()),
            Some("4120000000".into()),
            Some("VE".into()),
            Some(DocumentType::NationalId),
            Some("V-1".into()),
        )
        .unwrap();
        assert_eq!(contact.phone.as_ref().map(Phone::code), Some("+58"));
        assert!(ContactData::new("N".into(), Some("+58".into()), None, None, None, None).is_err());
        assert!(
            ContactData::new(
                "N".into(),
                None,
                None,
                Some("VE".into()),
                None,
                Some("1".into())
            )
            .is_err()
        );
        assert!(ContactData::new(" ".into(), None, None, None, None, None).is_err());
    }

    #[test]
    fn contact_patch_applies_changes_but_never_clears_the_name() {
        let original = ContactData::new(
            "Responsable ficticio".into(),
            Some("+58".into()),
            Some("4120000000".into()),
            None,
            None,
            None,
        )
        .unwrap();
        assert!(ContactPatch::default().apply(&original).is_err());
        assert!(
            ContactPatch {
                full_name: FieldChange::Clear,
                ..ContactPatch::default()
            }
            .apply(&original)
            .is_err()
        );

        let renamed = ContactPatch {
            full_name: FieldChange::Set("Otro nombre".into()),
            phone: FieldChange::Clear,
            phone_code: FieldChange::Clear,
            ..ContactPatch::default()
        }
        .apply(&original)
        .unwrap();
        assert_eq!(renamed.full_name, "Otro nombre");
        assert!(renamed.phone.is_none());
        assert!(original.phone.is_some());

        // Partial document changes keep the triple rule on the resulting value.
        assert!(
            ContactPatch {
                issuing_country: FieldChange::Set("VE".into()),
                ..ContactPatch::default()
            }
            .apply(&original)
            .is_err()
        );
        let with_document = ContactPatch {
            issuing_country: FieldChange::Set("VE".into()),
            document_type: FieldChange::Set(DocumentType::Passport),
            document_number: FieldChange::Set("P-9".into()),
            ..ContactPatch::default()
        }
        .apply(&original)
        .unwrap();
        let document = with_document.document.unwrap();
        assert_eq!(document.number(), "P-9");
        assert_eq!(document.document_type(), DocumentType::Passport);
    }

    #[test]
    fn patch_keeps_clears_and_sets_without_mutating_on_error() {
        let original = ProfileData {
            full_name: Some("Paciente ficticio".into()),
            phone: Some(Phone::new("+58".into(), "4120000000".into()).unwrap()),
            ..ProfileData::default()
        };
        let patch = ProfilePatch {
            full_name: FieldChange::Clear,
            birth_date: FieldChange::Set("2000-02-29".into()),
            ..ProfilePatch::default()
        };
        let next = patch.apply(&original).unwrap();
        assert_eq!(next.full_name, None);
        assert_eq!(next.phone, original.phone);
        assert_eq!(next.birth_date.as_deref(), Some("2000-02-29"));
        assert!(ProfilePatch::default().apply(&original).is_err());
        assert!(
            ProfilePatch {
                phone: FieldChange::Clear,
                ..ProfilePatch::default()
            }
            .apply(&original)
            .is_err()
        );
        let cleared = ProfilePatch {
            phone: FieldChange::Clear,
            phone_code: FieldChange::Clear,
            ..ProfilePatch::default()
        }
        .apply(&original)
        .unwrap();
        assert!(cleared.phone.is_none());
        assert!(original.phone.is_some());
    }
}
