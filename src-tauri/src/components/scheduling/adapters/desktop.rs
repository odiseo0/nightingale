//! Desktop session gate for the remote availability query. It reuses the
//! identity context so an account switch or logout during the HTTP call
//! invalidates the result instead of silently using the wrong session.
use super::super::application::{self, AvailabilityRequest, RemoteAvailability, Result};
use super::super::domain::SchedulingError;
use crate::components::identity::domain::AccountAccess;
use rusqlite::Connection;

fn current_session(connection: &Connection) -> Result<(String, Option<String>)> {
    connection
        .query_row(
            "SELECT user_id,selected_account_id FROM identity_context WHERE singleton=1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| {
            SchedulingError::new("PERMISSION_DENIED", "no hay sesión local activa")
        })
}

/// Snapshot of the desktop session taken before an online call.
pub struct SessionGate {
    account_id: String,
    user_id: String,
}

impl SessionGate {
    pub fn lock(connection: &Connection, access: &AccountAccess) -> Result<Self> {
        let (user_id, selected_account_id) = current_session(connection)?;

        if selected_account_id.as_deref() != Some(access.context.account_id.as_str())
            || user_id != access.context.user_id.as_str()
        {
            return Err(SchedulingError::new(
                "PERMISSION_DENIED",
                "la sesión local no coincide con el acceso",
            ));
        }

        Ok(Self {
            account_id: access.context.account_id.as_str().into(),
            user_id: access.context.user_id.as_str().into(),
        })
    }

    pub fn verify(&self, connection: &Connection) -> Result<()> {
        let (user_id, selected_account_id) = current_session(connection)?;

        if selected_account_id.as_deref() != Some(self.account_id.as_str())
            || user_id != self.user_id
        {
            return Err(SchedulingError::new(
                "PERMISSION_DENIED",
                "la sesión cambió durante la consulta remota",
            ));
        }

        Ok(())
    }
}

/// Runs the remote availability query behind the desktop session gate. The
/// connection is only read (never locked across the await) and no Outbox or
/// accepted view is written; any error leaves local data untouched.
pub async fn check_remote_availability(
    connection: &Connection,
    remote: &impl application::RemoteAvailabilityPort,
    access: &AccountAccess,
    request: &AvailabilityRequest,
) -> Result<RemoteAvailability> {
    let gate = SessionGate::lock(connection, access)?;

    let result = application::check_remote_availability(remote, access, request).await;

    gate.verify(connection)?;
    result
}
