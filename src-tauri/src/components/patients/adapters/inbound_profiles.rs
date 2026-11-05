//! Specific local commands for profiles and identifiers.
//! React cannot supply an account scope, an operation envelope or SQL.
use super::super::{application::*, domain::*};
use super::outbound::SqlitePatientRepository;
use crate::{
    components::identity::repository::SqliteAccessStore,
    shared_kernel::{SystemClock, UuidV7Generator},
};
use rusqlite::Connection;
use serde::{Deserialize, Deserializer};
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

/// Absence keeps, an explicit null clears, a value sets.
pub(crate) fn change<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> std::result::Result<Option<Option<T>>, D::Error> {
    Option::<T>::deserialize(d).map(Some)
}

/// The edit base is exactly one of an accepted version or the last local operation.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EditBaseRequest {
    #[serde(default)]
    pub accepted_version: Option<i64>,
    #[serde(default)]
    pub operation_id: Option<String>,
}

impl TryFrom<EditBaseRequest> for EditBase {
    type Error = PatientsError;

    fn try_from(value: EditBaseRequest) -> Result<Self> {
        match (value.accepted_version, value.operation_id) {
            (Some(version), None) if version > 0 => Ok(EditBase::Accepted { version }),
            (None, Some(operation_id)) => Ok(EditBase::LocalOperation {
                operation_id: operation_id.try_into()?,
            }),
            _ => Err(PatientsError::new(
                "VALIDATION_ERROR",
                "la base debe ser una versión aceptada o una operación local",
            )),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IdentifierInputRequest {
    pub id: String,
    #[serde(default)]
    pub issuing_country: Option<String>,
    pub document_type: String,
    pub document_number: String,
}

impl TryFrom<IdentifierInputRequest> for Document {
    type Error = PatientsError;

    fn try_from(value: IdentifierInputRequest) -> Result<Self> {
        Document::new(
            value.issuing_country.unwrap_or_else(|| "VE".into()),
            value.document_type.as_str().try_into()?,
            value.document_number,
        )
        .map_err(Into::into)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegisterPatientRequest {
    #[serde(default, deserialize_with = "change")]
    pub full_name: Option<Option<String>>,
    #[serde(default, deserialize_with = "change")]
    pub birth_date: Option<Option<String>>,
    #[serde(default, deserialize_with = "change")]
    pub phone_code: Option<Option<String>>,
    #[serde(default, deserialize_with = "change")]
    pub phone: Option<Option<String>>,
    #[serde(default, deserialize_with = "change")]
    pub identification_basis: Option<Option<String>>,
    #[serde(default)]
    pub identifiers: Option<Vec<IdentifierInputRequest>>,
}

fn data(request: &RegisterPatientRequest) -> Result<ProfileData> {
    let identification_basis = request
        .identification_basis
        .clone()
        .flatten()
        .map(|v| IdentificationBasis::try_from(v.as_str()))
        .transpose()?;

    Ok(ProfileData {
        full_name: request.full_name.clone().flatten(),
        birth_date: request.birth_date.clone().flatten(),
        phone: Phone::optional(
            request.phone_code.clone().flatten(),
            request.phone.clone().flatten(),
        )?,
        identification_basis,
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PatientPatchRequest {
    #[serde(default, deserialize_with = "change")]
    pub full_name: Option<Option<String>>,
    #[serde(default, deserialize_with = "change")]
    pub birth_date: Option<Option<String>>,
    #[serde(default, deserialize_with = "change")]
    pub phone_code: Option<Option<String>>,
    #[serde(default, deserialize_with = "change")]
    pub phone: Option<Option<String>>,
    #[serde(default, deserialize_with = "change")]
    pub identification_basis: Option<Option<String>>,
}

fn field<T>(value: Option<Option<T>>) -> FieldChange<T> {
    match value {
        None => FieldChange::Keep,
        Some(None) => FieldChange::Clear,
        Some(Some(v)) => FieldChange::Set(v),
    }
}

impl TryFrom<PatientPatchRequest> for ProfilePatch {
    type Error = PatientsError;

    fn try_from(value: PatientPatchRequest) -> Result<Self> {
        let identification_basis = match value.identification_basis {
            None => FieldChange::Keep,
            Some(None) => FieldChange::Clear,
            Some(Some(v)) => FieldChange::Set(IdentificationBasis::try_from(v.as_str())?),
        };

        Ok(ProfilePatch {
            full_name: field(value.full_name),
            birth_date: field(value.birth_date),
            phone_code: field(value.phone_code),
            phone: field(value.phone),
            identification_basis,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdatePatientProfileRequest {
    pub patient_id: String,
    pub base: EditBaseRequest,
    pub patch: PatientPatchRequest,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetPatientCompletionRequest {
    pub patient_id: String,
    pub base: EditBaseRequest,
    pub completion_status: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SetPatientProfileStatusRequest {
    pub patient_id: String,
    pub base: EditBaseRequest,
    pub status: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AddPatientIdentifierRequest {
    pub patient_id: String,
    #[serde(default)]
    pub issuing_country: Option<String>,
    pub document_type: String,
    pub document_number: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ListPatientsRequest {
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub completion_status: Option<String>,
    #[serde(default)]
    pub page: Option<usize>,
    #[serde(default)]
    pub shows: Option<usize>,
}

fn patient_filter(request: ListPatientsRequest) -> Result<PatientFilter> {
    Ok(PatientFilter {
        q: request.q,
        status: request
            .status
            .map(|v| Status::try_from(v.as_str()))
            .transpose()?,
        completion_status: request
            .completion_status
            .map(|v| CompletionStatus::try_from(v.as_str()))
            .transpose()?,
        page: request.page.unwrap_or(1),
        shows: request.shows.unwrap_or(100),
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PatientRequest {
    pub patient_id: String,
}

#[tauri::command]
pub fn patients_register_patient(
    connection: State<Mutex<Connection>>,
    request: RegisterPatientRequest,
) -> Result<LocalMutationResult> {
    let data = data(&request)?;
    let identifiers: Vec<Document> = request
        .identifiers
        .unwrap_or_default()
        .into_iter()
        .map(TryInto::try_into)
        .collect::<Result<_>>()?;
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.register_patient_with_identifiers(data, identifiers)
    })
}

#[tauri::command]
pub fn patients_update_patient_profile(
    connection: State<Mutex<Connection>>,
    request: UpdatePatientProfileRequest,
) -> Result<LocalMutationResult> {
    let base = request.base.try_into()?;
    let patch = request.patch.try_into()?;
    let patient = PatientId::try_from(request.patient_id)?;
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.update_patient_profile(patient, base, patch)
    })
}

#[tauri::command]
pub fn patients_set_patient_completion(
    connection: State<Mutex<Connection>>,
    request: SetPatientCompletionRequest,
) -> Result<LocalMutationResult> {
    let base = request.base.try_into()?;
    let completion = CompletionStatus::try_from(request.completion_status.as_str())?;
    let patient = PatientId::try_from(request.patient_id)?;
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.set_patient_completion(patient, base, completion)
    })
}

#[tauri::command]
pub fn patients_set_patient_profile_status(
    connection: State<Mutex<Connection>>,
    request: SetPatientProfileStatusRequest,
) -> Result<LocalMutationResult> {
    let base = request.base.try_into()?;
    let status = Status::try_from(request.status.as_str())?;
    let patient = PatientId::try_from(request.patient_id)?;
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.set_patient_profile_status(patient, base, status)
    })
}

#[tauri::command]
pub fn patients_add_patient_identifier(
    connection: State<Mutex<Connection>>,
    request: AddPatientIdentifierRequest,
) -> Result<LocalMutationResult> {
    let document = Document::new(
        request.issuing_country.unwrap_or_else(|| "VE".into()),
        request.document_type.as_str().try_into()?,
        request.document_number,
    )?;
    let patient = PatientId::try_from(request.patient_id)?;
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.add_patient_identifier(patient, document)
    })
}

#[tauri::command]
pub fn patients_list_patients(
    connection: State<Mutex<Connection>>,
    request: ListPatientsRequest,
) -> Result<PatientPage> {
    let filter = patient_filter(request)?;
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.list_patients(&filter)
    })
}

#[tauri::command]
pub fn patients_get_patient(
    connection: State<Mutex<Connection>>,
    request: PatientRequest,
) -> Result<EntityRow<ProfileState>> {
    let patient = PatientId::try_from(request.patient_id)?;
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.get_patient(&patient)
    })
}

#[tauri::command]
pub fn patients_list_patient_identifiers(
    connection: State<Mutex<Connection>>,
    request: PatientRequest,
) -> Result<Vec<EntityRow<IdentifierState>>> {
    let patient = PatientId::try_from(request.patient_id)?;
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.list_patient_identifiers(&patient)
    })
}

#[cfg(test)]
#[path = "../ipc_profile_tests.rs"]
mod tests;
