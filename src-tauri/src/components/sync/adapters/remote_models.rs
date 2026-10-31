// Transport models for the fixed api-v1.1 copy, not business entities.
// The entityType tag selects the state model. Keep nullable fields required.
// https://serde.rs/enum-representations.html#adjacently-tagged
use super::api::required_nullable;
use crate::shared_kernel::{NightingaleError, id::canonical_uuid_v7};
use serde::{Deserialize, Serialize};

/// UUIDv7 for a transport reference, without adding business IDs to the kernel.
#[derive(Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ProjectionId(String);

impl TryFrom<String> for ProjectionId {
    type Error = NightingaleError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        canonical_uuid_v7(&value)?;

        Ok(Self(value))
    }
}

impl From<ProjectionId> for String {
    fn from(value: ProjectionId) -> Self {
        value.0
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AccountResponseStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "inactive")]
    Inactive,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountResponse {
    #[serde(deserialize_with = "required_nullable")]
    pub created_at: Option<String>,
    pub display_name: String,
    pub id: ProjectionId,
    pub kind: String,
    pub location_address: String,
    pub location_city: String,
    pub location_country: String,
    pub location_name: String,
    pub location_state: String,
    pub no_show_grace_minutes: i64,
    pub owner_user_id: ProjectionId,
    pub status: AccountResponseStatus,
    pub time_zone: String,
    #[serde(deserialize_with = "required_nullable")]
    pub updated_at: Option<String>,
    pub version: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AccountDeviceResponseStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "revoked")]
    Revoked,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDeviceResponse {
    pub client_version: String,
    pub device_id: ProjectionId,
    pub display_name: String,
    #[serde(deserialize_with = "required_nullable")]
    pub last_seen_at: Option<String>,
    pub local_schema_version: i64,
    pub status: AccountDeviceResponseStatus,
    pub version: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum MemberResponseRole {
    #[serde(rename = "owner")]
    Owner,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberResponse {
    #[serde(deserialize_with = "required_nullable")]
    pub created_at: Option<String>,
    pub display_name: String,
    pub role: MemberResponseRole,
    pub status: AccountResponseStatus,
    #[serde(deserialize_with = "required_nullable")]
    pub updated_at: Option<String>,
    pub user_id: ProjectionId,
    pub version: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum InvitationResponseStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "accepted")]
    Accepted,
    #[serde(rename = "revoked")]
    Revoked,
    #[serde(rename = "expired")]
    Expired,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitationResponse {
    #[serde(deserialize_with = "required_nullable")]
    pub accepted_at: Option<String>,
    pub account_id: ProjectionId,
    #[serde(deserialize_with = "required_nullable")]
    pub created_at: Option<String>,
    pub created_by_user_id: ProjectionId,
    pub expires_at: String,
    pub id: ProjectionId,
    pub invitee_user_id: ProjectionId,
    pub status: InvitationResponseStatus,
    #[serde(deserialize_with = "required_nullable")]
    pub updated_at: Option<String>,
    pub version: i64,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfessionalResponse {
    #[serde(deserialize_with = "required_nullable")]
    pub created_at: Option<String>,
    pub id: ProjectionId,
    pub public_name: String,
    pub status: AccountResponseStatus,
    #[serde(deserialize_with = "required_nullable")]
    pub updated_at: Option<String>,
    pub user_id: ProjectionId,
    pub version: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum PatientProfileResponseCompletionStatus {
    #[serde(rename = "draft")]
    Draft,
    #[serde(rename = "complete")]
    Complete,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum PatientProfileResponseIdentificationBasis {
    #[serde(rename = "own_document")]
    OwnDocument,
    #[serde(rename = "minor_without_document")]
    MinorWithoutDocument,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatientProfileResponse {
    pub account_id: ProjectionId,
    #[serde(deserialize_with = "required_nullable")]
    pub birth_date: Option<String>,
    pub completion_status: PatientProfileResponseCompletionStatus,
    #[serde(deserialize_with = "required_nullable")]
    pub created_at: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub full_name: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub identification_basis: Option<PatientProfileResponseIdentificationBasis>,
    pub patient_id: ProjectionId,
    #[serde(deserialize_with = "required_nullable")]
    pub phone: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub phone_code: Option<String>,
    pub status: AccountResponseStatus,
    #[serde(deserialize_with = "required_nullable")]
    pub updated_at: Option<String>,
    pub version: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum PatientIdentifierResponseDocumentType {
    #[serde(rename = "national_id")]
    NationalId,
    #[serde(rename = "passport")]
    Passport,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum PatientIdentifierResponseVerificationStatus {
    #[serde(rename = "declared")]
    Declared,
    #[serde(rename = "verified")]
    Verified,
    #[serde(rename = "invalidated")]
    Invalidated,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatientIdentifierResponse {
    #[serde(deserialize_with = "required_nullable")]
    pub created_at: Option<String>,
    pub document_number: String,
    pub document_type: PatientIdentifierResponseDocumentType,
    pub id: ProjectionId,
    pub issuing_country: String,
    pub patient_id: ProjectionId,
    #[serde(deserialize_with = "required_nullable")]
    pub updated_at: Option<String>,
    pub verification_status: PatientIdentifierResponseVerificationStatus,
    pub version: i64,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactResponse {
    pub account_id: ProjectionId,
    #[serde(deserialize_with = "required_nullable")]
    pub created_at: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub document_number: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub document_type: Option<PatientIdentifierResponseDocumentType>,
    pub full_name: String,
    pub id: ProjectionId,
    #[serde(deserialize_with = "required_nullable")]
    pub issuing_country: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub phone: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub phone_code: Option<String>,
    pub status: AccountResponseStatus,
    #[serde(deserialize_with = "required_nullable")]
    pub updated_at: Option<String>,
    pub version: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum RelationshipResponseRelationshipKind {
    #[serde(rename = "mother")]
    Mother,
    #[serde(rename = "father")]
    Father,
    #[serde(rename = "guardian")]
    Guardian,
    #[serde(rename = "other")]
    Other,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum RelationshipResponseRepresentationStatus {
    #[serde(rename = "unverified")]
    Unverified,
    #[serde(rename = "verified")]
    Verified,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipResponse {
    pub account_id: ProjectionId,
    pub contact_person_id: ProjectionId,
    #[serde(deserialize_with = "required_nullable")]
    pub created_at: Option<String>,
    pub is_responsible_contact: bool,
    pub patient_id: ProjectionId,
    pub relationship_kind: RelationshipResponseRelationshipKind,
    pub representation_status: RelationshipResponseRepresentationStatus,
    pub status: AccountResponseStatus,
    #[serde(deserialize_with = "required_nullable")]
    pub updated_at: Option<String>,
    pub valid_from: String,
    #[serde(deserialize_with = "required_nullable")]
    pub valid_until: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub verified_at: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub verified_by_user_id: Option<ProjectionId>,
    pub version: i64,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceResponse {
    #[serde(deserialize_with = "required_nullable")]
    pub created_at: Option<String>,
    pub default_duration_minutes: i64,
    pub id: ProjectionId,
    pub name: String,
    pub professional_id: ProjectionId,
    pub status: AccountResponseStatus,
    #[serde(deserialize_with = "required_nullable")]
    pub updated_at: Option<String>,
    pub version: i64,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceOfferResponse {
    pub account_id: ProjectionId,
    #[serde(deserialize_with = "required_nullable")]
    pub created_at: Option<String>,
    pub default_duration_minutes: i64,
    #[serde(deserialize_with = "required_nullable")]
    pub duration_minutes_override: Option<i64>,
    pub effective_duration_minutes: i64,
    pub name: String,
    pub professional_id: ProjectionId,
    pub service_id: ProjectionId,
    pub service_status: AccountResponseStatus,
    pub service_version: i64,
    pub status: AccountResponseStatus,
    #[serde(deserialize_with = "required_nullable")]
    pub updated_at: Option<String>,
    pub version: i64,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleRuleResponse {
    pub account_id: ProjectionId,
    #[serde(deserialize_with = "required_nullable")]
    pub created_at: Option<String>,
    pub id: ProjectionId,
    pub iso_weekday: i64,
    pub local_end_minute: i64,
    pub local_start_minute: i64,
    pub professional_id: ProjectionId,
    pub status: AccountResponseStatus,
    #[serde(deserialize_with = "required_nullable")]
    pub updated_at: Option<String>,
    pub valid_from: String,
    #[serde(deserialize_with = "required_nullable")]
    pub valid_until: Option<String>,
    pub version: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ScheduleExceptionResponseKind {
    #[serde(rename = "closed")]
    Closed,
    #[serde(rename = "replacement")]
    Replacement,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleExceptionResponse {
    pub account_id: ProjectionId,
    #[serde(deserialize_with = "required_nullable")]
    pub created_at: Option<String>,
    pub id: ProjectionId,
    pub kind: ScheduleExceptionResponseKind,
    pub local_date: String,
    #[serde(deserialize_with = "required_nullable")]
    pub local_end_minute: Option<i64>,
    #[serde(deserialize_with = "required_nullable")]
    pub local_start_minute: Option<i64>,
    pub professional_id: ProjectionId,
    pub status: AccountResponseStatus,
    #[serde(deserialize_with = "required_nullable")]
    pub updated_at: Option<String>,
    pub version: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum ScheduleBlockResponseReasonCode {
    #[serde(rename = "vacation")]
    Vacation,
    #[serde(rename = "administrative")]
    Administrative,
    #[serde(rename = "unavailable")]
    Unavailable,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleBlockResponse {
    #[serde(deserialize_with = "required_nullable")]
    pub account_id: Option<ProjectionId>,
    #[serde(deserialize_with = "required_nullable")]
    pub created_at: Option<String>,
    pub ends_at: String,
    pub id: ProjectionId,
    pub professional_id: ProjectionId,
    #[serde(deserialize_with = "required_nullable")]
    pub reason_code: Option<ScheduleBlockResponseReasonCode>,
    pub starts_at: String,
    pub status: AccountResponseStatus,
    #[serde(deserialize_with = "required_nullable")]
    pub updated_at: Option<String>,
    pub version: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AppointmentResponseStatus {
    #[serde(rename = "scheduled")]
    Scheduled,
    #[serde(rename = "confirmed")]
    Confirmed,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "no_show")]
    NoShow,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppointmentResponse {
    pub account_id: ProjectionId,
    #[serde(deserialize_with = "required_nullable")]
    pub confirmed_at: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    pub created_at: Option<String>,
    pub ends_at: String,
    pub id: ProjectionId,
    pub location_address_snapshot: String,
    pub location_city_snapshot: String,
    pub location_country_snapshot: String,
    pub location_name_snapshot: String,
    pub location_state_snapshot: String,
    pub offered_duration_minutes_snapshot: i64,
    pub patient_id: ProjectionId,
    pub professional_id: ProjectionId,
    pub professional_name_snapshot: String,
    #[serde(deserialize_with = "required_nullable")]
    pub responsible_contact_id: Option<ProjectionId>,
    pub service_id: ProjectionId,
    pub service_name_snapshot: String,
    pub starts_at: String,
    pub status: AppointmentResponseStatus,
    pub time_zone_snapshot: String,
    #[serde(deserialize_with = "required_nullable")]
    pub updated_at: Option<String>,
    pub version: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AppointmentHistoryResponseAction {
    #[serde(rename = "created")]
    Created,
    #[serde(rename = "confirmed")]
    Confirmed,
    #[serde(rename = "rescheduled")]
    Rescheduled,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "no_show")]
    NoShow,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AppointmentHistoryResponseActorKind {
    #[serde(rename = "staff")]
    Staff,
    #[serde(rename = "patient")]
    Patient,
    #[serde(rename = "contact")]
    Contact,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppointmentHistoryResponse {
    pub account_id: ProjectionId,
    pub action: AppointmentHistoryResponseAction,
    #[serde(deserialize_with = "required_nullable")]
    pub actor_contact_person_id: Option<ProjectionId>,
    pub actor_kind: AppointmentHistoryResponseActorKind,
    #[serde(deserialize_with = "required_nullable")]
    pub actor_user_id: Option<ProjectionId>,
    pub appointment_id: ProjectionId,
    #[serde(deserialize_with = "required_nullable")]
    pub change_reason: Option<String>,
    pub changed_at: String,
    #[serde(deserialize_with = "required_nullable")]
    pub confirmed_at: Option<String>,
    pub ends_at: String,
    pub location_address_snapshot: String,
    pub location_city_snapshot: String,
    pub location_country_snapshot: String,
    pub location_name_snapshot: String,
    pub location_state_snapshot: String,
    pub offered_duration_minutes_snapshot: i64,
    pub professional_name_snapshot: String,
    #[serde(deserialize_with = "required_nullable")]
    pub responsible_contact_id: Option<ProjectionId>,
    pub service_name_snapshot: String,
    pub starts_at: String,
    pub status: AppointmentResponseStatus,
    pub time_zone_snapshot: String,
    pub version: i64,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppointmentHistoryProjection {
    pub account_id: ProjectionId,
    pub appointment_id: ProjectionId,
    pub ends_at: String,
    pub items: Vec<AppointmentHistoryResponse>,
    pub starts_at: String,
    pub version: i64,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "entityType", content = "state", rename_all = "snake_case")]
pub enum RemoteState {
    Account(AccountResponse),
    AccountDevice(AccountDeviceResponse),
    Membership(MemberResponse),
    Invitation(InvitationResponse),
    Professional(ProfessionalResponse),
    PatientProfile(PatientProfileResponse),
    PatientIdentifier(PatientIdentifierResponse),
    ContactPerson(ContactResponse),
    PatientContact(RelationshipResponse),
    Service(ServiceResponse),
    ServiceOffer(ServiceOfferResponse),
    ScheduleRule(ScheduleRuleResponse),
    ScheduleException(ScheduleExceptionResponse),
    ScheduleBlock(ScheduleBlockResponse),
    Appointment(AppointmentResponse),
    AppointmentHistory(AppointmentHistoryProjection),
}

impl RemoteState {
    pub fn version(&self) -> i64 {
        match self {
            Self::Account(state) => state.version,
            Self::AccountDevice(state) => state.version,
            Self::Membership(state) => state.version,
            Self::Invitation(state) => state.version,
            Self::Professional(state) => state.version,
            Self::PatientProfile(state) => state.version,
            Self::PatientIdentifier(state) => state.version,
            Self::ContactPerson(state) => state.version,
            Self::PatientContact(state) => state.version,
            Self::Service(state) => state.version,
            Self::ServiceOffer(state) => state.version,
            Self::ScheduleRule(state) => state.version,
            Self::ScheduleException(state) => state.version,
            Self::ScheduleBlock(state) => state.version,
            Self::Appointment(state) => state.version,
            Self::AppointmentHistory(state) => state.version,
        }
    }

    pub fn key(&self) -> serde_json::Value {
        match self {
            Self::Account(state) => serde_json::json!(state.id),
            Self::AccountDevice(state) => serde_json::json!(state.device_id),
            Self::Membership(state) => serde_json::json!(state.user_id),
            Self::Invitation(state) => serde_json::json!(state.id),
            Self::Professional(state) => serde_json::json!(state.id),
            Self::PatientProfile(state) => serde_json::json!(state.patient_id),
            Self::PatientIdentifier(state) => serde_json::json!(state.id),
            Self::ContactPerson(state) => serde_json::json!(state.id),
            Self::PatientContact(state) => {
                serde_json::json!({"patientId": state.patient_id, "contactPersonId": state.contact_person_id})
            }
            Self::Service(state) => serde_json::json!(state.id),
            Self::ServiceOffer(state) => {
                serde_json::json!({"professionalId": state.professional_id, "serviceId": state.service_id})
            }
            Self::ScheduleRule(state) => serde_json::json!(state.id),
            Self::ScheduleException(state) => serde_json::json!(state.id),
            Self::ScheduleBlock(state) => serde_json::json!(state.id),
            Self::Appointment(state) => serde_json::json!(state.id),
            Self::AppointmentHistory(state) => serde_json::json!(state.appointment_id),
        }
    }
}
