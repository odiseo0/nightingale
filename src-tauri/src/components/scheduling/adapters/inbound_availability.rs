//! Availability query commands: local evaluation with explicit coverage and
//! the explicit remote check. React never computes availability, versions or
//! permissions.
use super::super::application::{
    self, AvailabilityEvaluation, AvailabilityRequest, RemoteAvailability, Result,
    ScheduleAvailabilityQuery,
};
use super::inbound_rules::locked;
use super::outbound::{KnownOccupancy, SqliteSchedulingRepository};
use crate::{
    components::{
        appointments::{
            adapters::outbound::SqliteScheduleOccupancyQuery, application::ScheduleOccupancyQuery,
        },
        identity::{
            desktop::{DesktopSession, credential},
            repository::SqliteAccessStore,
        },
        sync::adapters::control::SqliteSyncControlStore,
    },
    platform::{config::APIConfig, http::APITransport},
};
use jiff::Timestamp;
use rusqlite::Connection;
use serde::Deserialize;
use std::{str::FromStr, sync::Mutex};
use tauri::{AppHandle, Manager, State};

use super::super::domain::SchedulingError;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalAvailabilityRequest {
    pub professional_id: String,
    pub service_id: String,
    pub from: String,
    pub until: String,
    #[serde(default)]
    pub duration_minutes: Option<i64>,
}

impl TryFrom<LocalAvailabilityRequest> for AvailabilityRequest {
    type Error = SchedulingError;

    fn try_from(value: LocalAvailabilityRequest) -> Result<Self> {
        Ok(AvailabilityRequest {
            exclude_appointment_id: None,
            professional_id: value.professional_id,
            service_id: value.service_id,
            from: Timestamp::from_str(&value.from)
                .map_err(|_| SchedulingError::validation("from debe ser RFC 3339"))?,
            until: Timestamp::from_str(&value.until)
                .map_err(|_| SchedulingError::validation("until debe ser RFC 3339"))?,
            duration_minutes: value.duration_minutes,
        })
    }
}

fn missing_selection() -> SchedulingError {
    SchedulingError::new(
        "AUTHENTICATION_REQUIRED",
        "inicie sesión y elija una cuenta",
    )
}

pub(crate) fn get_local_availability(
    connection: &mut Connection,
    request: LocalAvailabilityRequest,
) -> Result<AvailabilityEvaluation> {
    let request = AvailabilityRequest::try_from(request)?;
    let access = SqliteAccessStore
        .selected_access(connection)?
        .ok_or_else(missing_selection)?;

    // Occupancy is read first (immutable borrow ends) and handed as a snapshot.
    let known: Vec<_> = SqliteScheduleOccupancyQuery {
        connection: &*connection,
        scope: &access.context,
    }
    .list_occupancy(&request.professional_id, request.from, request.until)
    .map_err(SchedulingError::from)?
    .into_iter()
    .map(|interval| (request.professional_id.clone(), interval))
    .collect();

    let mut repository = SqliteSchedulingRepository { connection };
    repository.local_availability(
        &access,
        &request,
        &KnownOccupancy(known),
        &SqliteSyncControlStore,
    )
}

#[tauri::command]
pub fn scheduling_get_local_availability(
    connection: State<Mutex<Connection>>,
    request: LocalAvailabilityRequest,
) -> Result<AvailabilityEvaluation> {
    get_local_availability(&mut *connection.lock().map_err(|_| locked())?, request)
}

#[tauri::command]
pub async fn scheduling_check_remote_availability(
    app: AppHandle,
    request: LocalAvailabilityRequest,
) -> Result<RemoteAvailability> {
    let request = AvailabilityRequest::try_from(request)?;
    // The identity gate keeps logout/account selection from changing scope
    // between the access refresh and the remote availability request.
    let session = app.state::<DesktopSession>();
    let _gate = session.gate.lock().await;
    let database = app.state::<Mutex<Connection>>();
    let config = APIConfig::from_environment().map_err(|_| {
        SchedulingError::new(
            "SERVICE_UNAVAILABLE",
            "configure la conexión de Nightingale API",
        )
    })?;
    let credential = credential(&config, &database).map_err(|_| {
        SchedulingError::new(
            "AUTHENTICATION_REQUIRED",
            "inicie sesión con un dispositivo autorizado",
        )
    })?;
    let access = {
        let connection = database
            .lock()
            .map_err(|_| SchedulingError::internal("base local no disponible"))?;
        SqliteAccessStore
            .selected_access(&connection)?
            .ok_or_else(missing_selection)?
    };
    let http = APITransport::new(config).map_err(super::online::remote_error)?;
    let remote = super::online::OnlineScheduling {
        http: &http,
        token: &credential.token,
        scope: &access.context,
    };
    // The SQLite guard is never held across the HTTP call: the session gate is
    // taken before, and the session is re-verified after re-acquiring it.
    let gate = {
        let connection = database
            .lock()
            .map_err(|_| SchedulingError::internal("base local no disponible"))?;
        super::desktop::SessionGate::lock(&connection, &access)?
    };
    let result = application::check_remote_availability(&remote, &access, &request).await;
    let connection = database
        .lock()
        .map_err(|_| SchedulingError::internal("base local no disponible"))?;
    gate.verify(&connection)?;

    result
}
