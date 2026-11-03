//! Specific local commands. React cannot supply an account scope or an envelope.
use super::super::{application::*, domain::*};
use super::outbound::SqliteCatalogRepository;
use crate::{
    components::identity::repository::SqliteAccessStore,
    shared_kernel::{SystemClock, UuidV7Generator},
};
use rusqlite::Connection;
use serde::{Deserialize, Deserializer};
use std::sync::Mutex;
use tauri::State;

fn locked() -> ServicesError {
    ServicesError::new("INTERNAL_ERROR", "base local no disponible")
}

fn run<T>(
    c: &mut Connection,
    work: impl FnOnce(
        &mut CatalogUseCases<'_, SqliteCatalogRepository<'_>, UuidV7Generator, SystemClock>,
    ) -> Result<T>,
) -> Result<T> {
    let access = SqliteAccessStore.selected_access(c)?.ok_or_else(|| {
        ServicesError::new(
            "AUTHENTICATION_REQUIRED",
            "inicie sesión y elija una cuenta",
        )
    })?;
    let mut repository = SqliteCatalogRepository { connection: c };

    work(&mut CatalogUseCases {
        repository: &mut repository,
        access: &access,
        ids: &UuidV7Generator,
        clock: &SystemClock,
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateServiceRequest {
    pub professional_id: String,
    pub name: String,
    pub default_duration_minutes: i64,
}

// Default handles absence; a present null must not erase a required service field.
// https://serde.rs/field-attrs.html#deserialize_with
fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> std::result::Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServicePatchRequest {
    #[serde(default, deserialize_with = "present")]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub default_duration_minutes: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateServiceRequest {
    pub service_id: String,
    pub expected_version: i64,
    pub patch: ServicePatchRequest,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServiceStatusRequest {
    pub service_id: String,
    pub expected_version: i64,
    pub status: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OfferServiceRequest {
    pub professional_id: String,
    pub service_id: String,
    pub duration_minutes_override: Option<i64>,
}

fn required_nullable<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<i64>, D::Error> {
    Option::<i64>::deserialize(d)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateOfferRequest {
    pub professional_id: String,
    pub service_id: String,
    pub expected_version: i64,
    #[serde(deserialize_with = "required_nullable")]
    pub duration_minutes_override: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OfferStatusRequest {
    pub professional_id: String,
    pub service_id: String,
    pub expected_version: i64,
    pub status: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfessionalRequest {
    pub professional_id: String,
    pub expected_version: i64,
    pub public_name: String,
}

#[tauri::command]
pub fn services_list_catalog(connection: State<Mutex<Connection>>) -> Result<CatalogScreen> {
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.list_catalog()
    })
}

#[tauri::command]
pub fn services_create_service(
    connection: State<Mutex<Connection>>,
    request: CreateServiceRequest,
) -> Result<LocalMutationResult> {
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.create_service(
            request.professional_id.try_into()?,
            request.name,
            DurationMinutes::new(request.default_duration_minutes)?,
        )
    })
}

#[tauri::command]
pub fn services_update_service(
    connection: State<Mutex<Connection>>,
    request: UpdateServiceRequest,
) -> Result<LocalMutationResult> {
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.update_service(
            request.service_id.try_into()?,
            request.expected_version,
            ServicePatch::new(request.patch.name, request.patch.default_duration_minutes)?,
        )
    })
}

#[tauri::command]
pub fn services_set_service_status(
    connection: State<Mutex<Connection>>,
    request: ServiceStatusRequest,
) -> Result<LocalMutationResult> {
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.set_service_status(
            request.service_id.try_into()?,
            request.expected_version,
            Status::try_from(request.status.as_str())?,
        )
    })
}

#[tauri::command]
pub fn services_offer_service(
    connection: State<Mutex<Connection>>,
    request: OfferServiceRequest,
) -> Result<LocalMutationResult> {
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.offer_service(
            OfferKey::new(request.professional_id, request.service_id)?,
            request
                .duration_minutes_override
                .map(DurationMinutes::new)
                .transpose()?,
        )
    })
}

#[tauri::command]
pub fn services_update_service_offer(
    connection: State<Mutex<Connection>>,
    request: UpdateOfferRequest,
) -> Result<LocalMutationResult> {
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.update_service_offer(
            OfferKey::new(request.professional_id, request.service_id)?,
            request.expected_version,
            ServiceOfferPatch::new(Some(request.duration_minutes_override))?,
        )
    })
}

#[tauri::command]
pub fn services_set_service_offer_status(
    connection: State<Mutex<Connection>>,
    request: OfferStatusRequest,
) -> Result<LocalMutationResult> {
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.set_service_offer_status(
            OfferKey::new(request.professional_id, request.service_id)?,
            request.expected_version,
            Status::try_from(request.status.as_str())?,
        )
    })
}

#[tauri::command]
pub fn services_update_professional_profile(
    connection: State<Mutex<Connection>>,
    request: ProfessionalRequest,
) -> Result<LocalMutationResult> {
    run(&mut *connection.lock().map_err(|_| locked())?, |cases| {
        cases.update_professional_profile(
            request.professional_id.try_into()?,
            request.expected_version,
            request.public_name,
        )
    })
}

#[cfg(test)]
#[path = "../ipc_tests.rs"]
mod tests;
