//! Native patient cases. Storage and transport have their own models.
use super::domain::{
    CompletionStatus, ContactData, ContactPatch, ContactPersonId, Document, PatientId, ProfileData,
    ProfilePatch, RelationshipData, RelationshipPatch, Status,
};
use crate::{
    components::identity::domain::{AccessStatus, AccountAccess},
    shared_kernel::{Clock, IdGenerator, NightingaleError},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatientsError {
    pub code: String,
    pub message: String,
    pub affected_patient_ids: Vec<String>,
}

pub type Result<T> = std::result::Result<T, PatientsError>;

impl PatientsError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            affected_patient_ids: vec![],
        }
    }
}

impl From<NightingaleError> for PatientsError {
    fn from(error: NightingaleError) -> Self {
        let code = serde_json::to_value(error.code)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_else(|| "INTERNAL_ERROR".into());
        let message = if code == "INTERNAL_ERROR" {
            "datos locales de pacientes no disponibles"
        } else {
            &error.message
        };

        Self::new(&code, message)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileState {
    pub patient_id: String,
    pub full_name: Option<String>,
    pub birth_date: Option<String>,
    pub phone_code: Option<String>,
    pub phone: Option<String>,
    pub identification_basis: Option<String>,
    pub completion_status: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Accepted<T> {
    pub state: T,
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Provisional<T> {
    pub state: T,
    pub operation_id: String,
    pub expected_version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationView {
    pub operation_id: String,
    pub depends_on: Vec<String>,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityRow<T> {
    pub accepted: Option<Accepted<T>>,
    pub provisional: Option<Provisional<T>>,
    pub operations: Vec<OperationView>,
}

impl<T> EntityRow<T> {
    pub fn visible(&self) -> Option<&T> {
        self.provisional
            .as_ref()
            .map(|v| &v.state)
            .or_else(|| self.accepted.as_ref().map(|v| &v.state))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipState {
    pub patient_id: String,
    pub contact_person_id: String,
    pub relationship_kind: String,
    pub is_responsible_contact: bool,
    pub valid_from: String,
    pub valid_until: Option<String>,
    pub status: String,
}

/// Read-side join of a relationship with current names and phone of both ends.
/// Related data is not part of the accepted relationship state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipView {
    pub relationship: RelationshipState,
    pub patient_full_name: Option<String>,
    pub contact_full_name: String,
    pub contact_phone_code: Option<String>,
    pub contact_phone: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactState {
    pub contact_person_id: String,
    pub full_name: String,
    pub phone_code: Option<String>,
    pub phone: Option<String>,
    pub issuing_country: Option<String>,
    pub document_type: Option<String>,
    pub document_number: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct ContactFilter {
    pub q: Option<String>,
    pub status: Option<Status>,
    pub page: usize,
    pub shows: usize,
}

impl Default for ContactFilter {
    fn default() -> Self {
        Self {
            q: None,
            status: None,
            page: 1,
            shows: 100,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ContactPage {
    pub items: Vec<EntityRow<ContactState>>,
    pub total: usize,
}

#[derive(Debug, Clone)]
pub struct PatientFilter {
    pub q: Option<String>,
    pub status: Option<Status>,
    pub completion_status: Option<CompletionStatus>,
    pub page: usize,
    pub shows: usize,
}

impl Default for PatientFilter {
    fn default() -> Self {
        Self {
            q: None,
            status: None,
            completion_status: None,
            page: 1,
            shows: 100,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PatientPage {
    pub items: Vec<EntityRow<ProfileState>>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalMutationResult {
    pub operation_id: String,
    pub entity_type: String,
    pub entity_key: Value,
    pub expected_version: i64,
    pub depends_on: Vec<String>,
    pub provisional: Value,
}

#[derive(Debug, Clone)]
pub enum EditBase {
    Accepted {
        version: i64,
    },
    LocalOperation {
        operation_id: crate::shared_kernel::id::OperationId,
    },
}

pub enum PatientMutation {
    CreateRelationship {
        patient: PatientId,
        contact: ContactPersonId,
        data: RelationshipData,
    },
    UpdateRelationship {
        patient: PatientId,
        contact: ContactPersonId,
        base: EditBase,
        patch: RelationshipPatch,
    },
    SetRelationshipStatus {
        patient: PatientId,
        contact: ContactPersonId,
        base: EditBase,
        status: Status,
    },
    CreateContact {
        data: ContactData,
    },
    UpdateContact {
        contact: ContactPersonId,
        base: EditBase,
        patch: ContactPatch,
    },
    SetContactStatus {
        contact: ContactPersonId,
        base: EditBase,
        status: Status,
    },
    Register {
        profile: ProfileData,
        identifiers: Vec<Document>,
    },
    Update {
        patient: PatientId,
        base: EditBase,
        patch: ProfilePatch,
    },
    SetCompletion {
        patient: PatientId,
        base: EditBase,
        completion: CompletionStatus,
    },
    AddIdentifier {
        patient: PatientId,
        document: Document,
    },
    SetStatus {
        patient: PatientId,
        base: EditBase,
        status: Status,
    },
}

impl PatientMutation {
    pub fn action(&self) -> &'static str {
        match self {
            Self::CreateRelationship { .. } => "CreatePatientContactRelationship",
            Self::UpdateRelationship { .. } => "UpdatePatientContactRelationship",
            Self::SetRelationshipStatus { .. } => "SetPatientContactRelationshipStatus",
            Self::CreateContact { .. } => "CreateContactPerson",
            Self::UpdateContact { .. } => "UpdateContactPerson",
            Self::SetContactStatus { .. } => "SetContactPersonStatus",
            Self::Register { .. } => "RegisterPatient",
            Self::Update { .. } => "UpdatePatientProfile",
            Self::SetStatus { .. } => "SetPatientProfileStatus",
            Self::SetCompletion { .. } => "SetPatientCompletion",
            Self::AddIdentifier { .. } => "AddPatientIdentifier",
        }
    }
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Register {
                profile,
                identifiers,
            } => {
                if identifiers.len() > 100 {
                    return Err(PatientsError::new(
                        "VALIDATION_ERROR",
                        "máximo cien documentos iniciales",
                    ));
                }
                profile.validate().map_err(Into::into)
            }
            Self::CreateRelationship { .. }
            | Self::UpdateRelationship { .. }
            | Self::SetRelationshipStatus { .. }
            | Self::CreateContact { .. }
            | Self::UpdateContact { .. }
            | Self::SetContactStatus { .. }
            | Self::Update { .. }
            | Self::SetStatus { .. }
            | Self::SetCompletion { .. }
            | Self::AddIdentifier { .. } => Ok(()),
        }
    }
}

pub fn require_read(access: &AccountAccess) -> Result<()> {
    if access.status != AccessStatus::Active {
        return Err(PatientsError::new(
            "PERMISSION_DENIED",
            "la cuenta no tiene acceso activo",
        ));
    }

    Ok(())
}

pub fn require_write(access: &AccountAccess, action: &str) -> Result<()> {
    if !access.allows_offline(action) {
        return Err(PatientsError::new(
            "PERMISSION_DENIED",
            "la acción no tiene permiso local",
        ));
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentifierState {
    pub id: String,
    pub patient_id: String,
    pub issuing_country: String,
    pub document_type: String,
    pub document_number: String,
    pub verification_status: String,
}

/// Internal consumers receive accepted administrative values, never provisional versions.
pub trait PatientQuery {
    fn accepted_patient(
        &mut self,
        access: &AccountAccess,
        patient: &PatientId,
    ) -> Result<Option<Accepted<ProfileState>>>;
    fn accepted_identifiers(
        &mut self,
        access: &AccountAccess,
        patient: &PatientId,
    ) -> Result<Vec<Accepted<IdentifierState>>>;
}

pub trait PatientRepository {
    fn identifiers(
        &mut self,
        access: &AccountAccess,
        patient: &PatientId,
    ) -> Result<Vec<EntityRow<IdentifierState>>>;

    fn profiles(&mut self, access: &AccountAccess, filter: &PatientFilter) -> Result<PatientPage>;
    fn contacts(&mut self, access: &AccountAccess, filter: &ContactFilter) -> Result<ContactPage>;
    fn contact(
        &mut self,
        access: &AccountAccess,
        contact: &ContactPersonId,
    ) -> Result<EntityRow<ContactState>>;
    fn patient_contacts(
        &mut self,
        access: &AccountAccess,
        patient: &PatientId,
        status: Option<Status>,
    ) -> Result<Vec<EntityRow<RelationshipView>>>;
    fn contact_patients(
        &mut self,
        access: &AccountAccess,
        contact: &ContactPersonId,
        status: Option<Status>,
    ) -> Result<Vec<EntityRow<RelationshipView>>>;
    fn profile(
        &mut self,
        access: &AccountAccess,
        patient: &PatientId,
    ) -> Result<EntityRow<ProfileState>>;
    fn persist(
        &mut self,
        access: &AccountAccess,
        mutation: PatientMutation,
        ids: &impl IdGenerator,
        clock: &impl Clock,
    ) -> Result<LocalMutationResult>;
}

pub struct PatientUseCases<'a, R, I, C> {
    pub repository: &'a mut R,
    pub access: &'a AccountAccess,
    pub ids: &'a I,
    pub clock: &'a C,
}

impl<R: PatientRepository, I: IdGenerator, C: Clock> PatientUseCases<'_, R, I, C> {
    pub fn register_patient(&mut self, profile: ProfileData) -> Result<LocalMutationResult> {
        self.register_patient_with_identifiers(profile, vec![])
    }

    pub fn register_patient_with_identifiers(
        &mut self,
        profile: ProfileData,
        identifiers: Vec<Document>,
    ) -> Result<LocalMutationResult> {
        self.write(PatientMutation::Register {
            profile,
            identifiers,
        })
    }

    pub fn add_patient_identifier(
        &mut self,
        patient: PatientId,
        document: Document,
    ) -> Result<LocalMutationResult> {
        self.write(PatientMutation::AddIdentifier { patient, document })
    }

    pub fn set_patient_completion(
        &mut self,
        patient: PatientId,
        base: EditBase,
        completion: CompletionStatus,
    ) -> Result<LocalMutationResult> {
        self.write(PatientMutation::SetCompletion {
            patient,
            base,
            completion,
        })
    }

    pub fn list_patient_identifiers(
        &mut self,
        patient: &PatientId,
    ) -> Result<Vec<EntityRow<IdentifierState>>> {
        require_read(self.access)?;
        self.repository.identifiers(self.access, patient)
    }

    pub fn update_patient_profile(
        &mut self,
        patient: PatientId,
        base: EditBase,
        patch: ProfilePatch,
    ) -> Result<LocalMutationResult> {
        self.write(PatientMutation::Update {
            patient,
            base,
            patch,
        })
    }

    pub fn set_patient_profile_status(
        &mut self,
        patient: PatientId,
        base: EditBase,
        status: Status,
    ) -> Result<LocalMutationResult> {
        self.write(PatientMutation::SetStatus {
            patient,
            base,
            status,
        })
    }

    fn write(&mut self, mutation: PatientMutation) -> Result<LocalMutationResult> {
        require_write(self.access, mutation.action())?;
        mutation.validate()?;
        self.repository
            .persist(self.access, mutation, self.ids, self.clock)
    }

    pub fn list_patients(&mut self, filter: &PatientFilter) -> Result<PatientPage> {
        require_read(self.access)?;

        if filter.page == 0
            || !(1..=500).contains(&filter.shows)
            || (filter.page - 1).checked_mul(filter.shows).is_none()
        {
            return Err(PatientsError::new("VALIDATION_ERROR", "página inválida"));
        }

        self.repository.profiles(self.access, filter)
    }

    pub fn get_patient(&mut self, patient: &PatientId) -> Result<EntityRow<ProfileState>> {
        require_read(self.access)?;
        self.repository.profile(self.access, patient)
    }

    pub fn create_contact_person(&mut self, data: ContactData) -> Result<LocalMutationResult> {
        self.write(PatientMutation::CreateContact { data })
    }

    pub fn update_contact_person(
        &mut self,
        contact: ContactPersonId,
        base: EditBase,
        patch: ContactPatch,
    ) -> Result<LocalMutationResult> {
        self.write(PatientMutation::UpdateContact {
            contact,
            base,
            patch,
        })
    }

    pub fn set_contact_person_status(
        &mut self,
        contact: ContactPersonId,
        base: EditBase,
        status: Status,
    ) -> Result<LocalMutationResult> {
        self.write(PatientMutation::SetContactStatus {
            contact,
            base,
            status,
        })
    }

    pub fn list_contacts(&mut self, filter: &ContactFilter) -> Result<ContactPage> {
        require_read(self.access)?;

        if filter.page == 0
            || !(1..=500).contains(&filter.shows)
            || (filter.page - 1).checked_mul(filter.shows).is_none()
        {
            return Err(PatientsError::new("VALIDATION_ERROR", "página inválida"));
        }

        self.repository.contacts(self.access, filter)
    }

    pub fn get_contact(&mut self, contact: &ContactPersonId) -> Result<EntityRow<ContactState>> {
        require_read(self.access)?;
        self.repository.contact(self.access, contact)
    }

    pub fn create_patient_contact_relationship(
        &mut self,
        patient: PatientId,
        contact: ContactPersonId,
        data: RelationshipData,
    ) -> Result<LocalMutationResult> {
        self.write(PatientMutation::CreateRelationship {
            patient,
            contact,
            data,
        })
    }

    pub fn update_patient_contact_relationship(
        &mut self,
        patient: PatientId,
        contact: ContactPersonId,
        base: EditBase,
        patch: RelationshipPatch,
    ) -> Result<LocalMutationResult> {
        self.write(PatientMutation::UpdateRelationship {
            patient,
            contact,
            base,
            patch,
        })
    }

    pub fn set_patient_contact_relationship_status(
        &mut self,
        patient: PatientId,
        contact: ContactPersonId,
        base: EditBase,
        status: Status,
    ) -> Result<LocalMutationResult> {
        self.write(PatientMutation::SetRelationshipStatus {
            patient,
            contact,
            base,
            status,
        })
    }

    pub fn list_patient_contacts(
        &mut self,
        patient: &PatientId,
        status: Option<Status>,
    ) -> Result<Vec<EntityRow<RelationshipView>>> {
        require_read(self.access)?;
        self.repository
            .patient_contacts(self.access, patient, status)
    }

    pub fn list_contact_patients(
        &mut self,
        contact: &ContactPersonId,
        status: Option<Status>,
    ) -> Result<Vec<EntityRow<RelationshipView>>> {
        require_read(self.access)?;
        self.repository
            .contact_patients(self.access, contact, status)
    }
}
