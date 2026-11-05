//! Specific local commands for contacts and patient relationships.
use super::super::{application::*, domain::*};
use super::inbound_profiles::{EditBaseRequest, change};
use super::outbound::SqlitePatientRepository;
use crate::{
    components::identity::repository::SqliteAccessStore,
    shared_kernel::{SystemClock, UuidV7Generator},
};
use rusqlite::Connection;
use serde::Deserialize;
use std::sync::Mutex;
use tauri::State;

fn locked() -> PatientsError {
    PatientsError::new("INTERNAL_ERROR", "base local no disponible")
}

pub(crate) fn run<T>(
    c: &mut Connection,
    work: impl FnOnce(
        &mut PatientUseCases<'_, SqlitePatientRepository<'_>, UuidV7Generator, SystemClock>,
    ) -> Result<T>,
) -> Result<T> {
    let access = SqliteAccessStore.selected_access(c)?.ok_or_else(|| {
        PatientsError::new(
            "AUTHENTICATION_REQUIRED",
            "inicie sesión y elija una cuenta",
        )
    })?;
    let mut repository = SqlitePatientRepository { connection: c };

    work(&mut PatientUseCases {
        repository: &mut repository,
        access: &access,
        ids: &UuidV7Generator,
        clock: &SystemClock,
    })
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateContactRequest {
    pub full_name: String,
    #[serde(default, deserialize_with = "change")]
    pub phone_code: Option<Option<String>>,
    #[serde(default, deserialize_with = "change")]
    pub phone: Option<Option<String>>,
    #[serde(default, deserialize_with = "change")]
    pub issuing_country: Option<Option<String>>,
    #[serde(default, deserialize_with = "change")]
    pub document_type: Option<Option<String>>,
    #[serde(default, deserialize_with = "change")]
    pub document_number: Option<Option<String>>,
}

impl TryFrom<CreateContactRequest> for ContactData {
    type Error = PatientsError;

    fn try_from(value: CreateContactRequest) -> Result<Self> {
        let document_type = value
            .document_type
            .flatten()
            .map(|v| DocumentType::try_from(v.as_str()))
            .transpose()?;
        ContactData::new(
            value.full_name,
            value.phone_code.flatten(),
            value.phone.flatten(),
            value.issuing_country.flatten(),
            document_type,
            value.document_number.flatten(),
        )
        .map_err(Into::into)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContactPatchRequest {
    #[serde(default, deserialize_with = "change")]
    pub full_name: Option<Option<String>>,
    #[serde(default, deserialize_with = "change")]
    pub phone_code: Option<Option<String>>,
    #[serde(default, deserialize_with = "change")]
    pub phone: Option<Option<String>>,
    #[serde(default, deserialize_with = "change")]
    pub issuing_country: Option<Option<String>>,
    #[serde(default, deserialize_with = "change")]
    pub document_type: Option<Option<String>>,
    #[serde(default, deserialize_with = "change")]
    pub document_number: Option<Option<String>>,
}

fn field<T>(value: Option<Option<T>>) -> FieldChange<T> {
    match value {
        None => FieldChange::Keep,
        Some(None) => FieldChange::Clear,
        Some(Some(v)) => FieldChange::Set(v),
    }
}

impl TryFrom<ContactPatchRequest> for ContactPatch {
    type Error = PatientsError;

    fn try_from(value: ContactPatchRequest) -> Result<Self> {
        let document_type = match value.document_type {
            None => FieldChange::Keep,
            Some(None) => FieldChange::Clear,
            Some(Some(v)) => FieldChange::Set(DocumentType::try_from(v.as_str())?),
        };

        Ok(ContactPatch {
            full_name: field(value.full_name),
            phone_code: field(value.phone_code),
            phone: field(value.phone),
            issuing_country: field(value.issuing_country),
            document_type,
            document_number: field(value.document_number),
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateContactRequest {
    pub contact_person_id: String,
    pub base: EditBaseRequest,
    pub patch: ContactPatchRequest,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetContactStatusRequest {
    pub contact_person_id: String,
    pub base: EditBaseRequest,
    pub status: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateRelationshipRequest {
    pub patient_id: String,
    pub contact_person_id: String,
    pub relationship_kind: String,
    #[serde(default)]
    pub is_responsible_contact: Option<bool>,
    pub valid_from: String,
    #[serde(default, deserialize_with = "change")]
    pub valid_until: Option<Option<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RelationshipPatchRequest {
    #[serde(default, deserialize_with = "change")]
    pub relationship_kind: Option<Option<String>>,
    #[serde(default, deserialize_with = "change")]
    pub is_responsible_contact: Option<Option<bool>>,
    #[serde(default, deserialize_with = "change")]
    pub valid_from: Option<Option<String>>,
    #[serde(default, deserialize_with = "change")]
    pub valid_until: Option<Option<String>>,
}

impl TryFrom<RelationshipPatchRequest> for RelationshipPatch {
    type Error = PatientsError;

    fn try_from(value: RelationshipPatchRequest) -> Result<Self> {
        let relationship_kind = match value.relationship_kind {
            None => FieldChange::Keep,
            Some(None) => FieldChange::Clear,
            Some(Some(v)) => FieldChange::Set(RelationshipKind::try_from(v.as_str())?),
        };
        Ok(RelationshipPatch {
            relationship_kind,
            is_responsible_contact: field(value.is_responsible_contact),
            valid_from: field(value.valid_from),
            valid_until: field(value.valid_until),
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateRelationshipRequest {
    pub patient_id: String,
    pub contact_person_id: String,
    pub base: EditBaseRequest,
    pub patch: RelationshipPatchRequest,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetRelationshipStatusRequest {
    pub patient_id: String,
    pub contact_person_id: String,
    pub base: EditBaseRequest,
    pub status: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListContactsRequest {
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub page: Option<usize>,
    #[serde(default)]
    pub shows: Option<usize>,
}

fn contact_filter(request: ListContactsRequest) -> Result<ContactFilter> {
    Ok(ContactFilter {
        q: request.q,
        status: request
            .status
            .map(|v| Status::try_from(v.as_str()))
            .transpose()?,
        page: request.page.unwrap_or(1),
        shows: request.shows.unwrap_or(100),
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContactRequest {
    pub contact_person_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PatientContactsRequest {
    pub patient_id: String,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContactPatientsRequest {
    pub contact_person_id: String,
    #[serde(default)]
    pub status: Option<String>,
}

#[tauri::command]
pub fn patients_create_contact_person(
    connection: State<Mutex<Connection>>,
    request: CreateContactRequest,
) -> Result<LocalMutationResult> {
    let data = request.try_into()?;
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.create_contact_person(data)
    })
}

#[tauri::command]
pub fn patients_update_contact_person(
    connection: State<Mutex<Connection>>,
    request: UpdateContactRequest,
) -> Result<LocalMutationResult> {
    let contact = ContactPersonId::try_from(request.contact_person_id)?;
    let base = request.base.try_into()?;
    let patch = request.patch.try_into()?;
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.update_contact_person(contact, base, patch)
    })
}

#[tauri::command]
pub fn patients_set_contact_person_status(
    connection: State<Mutex<Connection>>,
    request: SetContactStatusRequest,
) -> Result<LocalMutationResult> {
    let contact = ContactPersonId::try_from(request.contact_person_id)?;
    let base = request.base.try_into()?;
    let status = Status::try_from(request.status.as_str())?;
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.set_contact_person_status(contact, base, status)
    })
}

#[tauri::command]
pub fn patients_create_patient_contact_relationship(
    connection: State<Mutex<Connection>>,
    request: CreateRelationshipRequest,
) -> Result<LocalMutationResult> {
    let patient = PatientId::try_from(request.patient_id)?;
    let contact = ContactPersonId::try_from(request.contact_person_id)?;
    let data = RelationshipData::new(
        RelationshipKind::try_from(request.relationship_kind.as_str())?,
        request.is_responsible_contact.unwrap_or(false),
        request.valid_from,
        request.valid_until.flatten(),
    )?;
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.create_patient_contact_relationship(patient, contact, data)
    })
}

#[tauri::command]
pub fn patients_update_patient_contact_relationship(
    connection: State<Mutex<Connection>>,
    request: UpdateRelationshipRequest,
) -> Result<LocalMutationResult> {
    let patient = PatientId::try_from(request.patient_id)?;
    let contact = ContactPersonId::try_from(request.contact_person_id)?;
    let base = request.base.try_into()?;
    let patch = request.patch.try_into()?;
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.update_patient_contact_relationship(patient, contact, base, patch)
    })
}

#[tauri::command]
pub fn patients_set_patient_contact_relationship_status(
    connection: State<Mutex<Connection>>,
    request: SetRelationshipStatusRequest,
) -> Result<LocalMutationResult> {
    let patient = PatientId::try_from(request.patient_id)?;
    let contact = ContactPersonId::try_from(request.contact_person_id)?;
    let base = request.base.try_into()?;
    let status = Status::try_from(request.status.as_str())?;
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.set_patient_contact_relationship_status(patient, contact, base, status)
    })
}

#[tauri::command]
pub fn patients_list_contacts(
    connection: State<Mutex<Connection>>,
    request: ListContactsRequest,
) -> Result<ContactPage> {
    let filter = contact_filter(request)?;
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.list_contacts(&filter)
    })
}

#[tauri::command]
pub fn patients_get_contact(
    connection: State<Mutex<Connection>>,
    request: ContactRequest,
) -> Result<EntityRow<ContactState>> {
    let contact = ContactPersonId::try_from(request.contact_person_id)?;
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.get_contact(&contact)
    })
}

#[tauri::command]
pub fn patients_list_patient_contacts(
    connection: State<Mutex<Connection>>,
    request: PatientContactsRequest,
) -> Result<Vec<EntityRow<RelationshipView>>> {
    let patient = PatientId::try_from(request.patient_id)?;
    let status = request
        .status
        .map(|v| Status::try_from(v.as_str()))
        .transpose()?;
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.list_patient_contacts(&patient, status)
    })
}

#[tauri::command]
pub fn patients_list_contact_patients(
    connection: State<Mutex<Connection>>,
    request: ContactPatientsRequest,
) -> Result<Vec<EntityRow<RelationshipView>>> {
    let contact = ContactPersonId::try_from(request.contact_person_id)?;
    let status = request
        .status
        .map(|v| Status::try_from(v.as_str()))
        .transpose()?;
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.list_contact_patients(&contact, status)
    })
}

#[cfg(test)]
#[path = "../ipc_contact_tests.rs"]
mod tests;
