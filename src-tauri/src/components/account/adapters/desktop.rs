use super::{
    super::application::{
        self, AccountChange, AccountError, AccountMutationResult, MutationStatus, RefreshStatus,
        Result,
    },
    online::{OnlineAccount, remote_error},
};
use crate::{
    components::identity::{
        desktop::{DesktopSession, credential, sync_now},
        repository::{AccessStore, SqliteAccessStore},
    },
    platform::{config::APIConfig, http::APITransport},
    shared_kernel::UuidV7Generator,
};
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub async fn execute(
    app: AppHandle,
    expected_version: i64,
    change: AccountChange,
) -> Result<AccountMutationResult> {
    let mut result = {
        // Use the identity gate to keep logout/account selection from changing
        // scope between the access refresh and the direct submission.
        let session = app.state::<DesktopSession>();
        let _gate = session.gate.lock().await;
        let database = app.state::<Mutex<Connection>>();
        let config = APIConfig::from_environment().map_err(|_| {
            AccountError::new(
                "SERVICE_UNAVAILABLE",
                "configure la conexión de Nightingale API",
            )
        })?;
        let credential = credential(&config, &database).map_err(|_| {
            AccountError::new(
                "AUTHENTICATION_REQUIRED",
                "inicie sesión con un dispositivo autorizado",
            )
        })?;
        let access = {
            let connection = database
                .lock()
                .map_err(|_| AccountError::new("INTERNAL_ERROR", "base local no disponible"))?;
            let account = SqliteAccessStore
                .context(&connection)?
                .and_then(|context| context.selected_account_id)
                .ok_or_else(|| AccountError::new("AUTHENTICATION_REQUIRED", "elija una cuenta"))?;
            SqliteAccessStore
                .access(
                    &connection,
                    &account,
                    &credential.user_id,
                    &credential.device_id,
                )?
                .ok_or_else(|| {
                    AccountError::new("PERMISSION_DENIED", "acceso local no disponible")
                })?
        };
        let http = APITransport::new(config).map_err(remote_error)?;
        let remote = OnlineAccount {
            http: &http,
            token: &credential.token,
            scope: &access.context,
        };
        match change {
            AccountChange::Details(patch) => {
                application::update_account_details(
                    &remote,
                    &access,
                    expected_version,
                    patch,
                    &UuidV7Generator,
                )
                .await?
            }
            AccountChange::Policy(policy) => {
                application::update_account_policy(
                    &remote,
                    &access,
                    expected_version,
                    policy,
                    &UuidV7Generator,
                )
                .await?
            }
            AccountChange::Status(status) => {
                application::set_account_status(
                    &remote,
                    &access,
                    expected_version,
                    status,
                    &UuidV7Generator,
                )
                .await?
            }
        }
    };

    // Release the gate before sync_now takes it. A failed refresh does not undo
    // acceptance and must not cause a new operation to be submitted.
    if result.status == MutationStatus::Accepted {
        result.refresh_status = match sync_now(app).await {
            Ok(report)
                if report.outcome
                    == crate::components::sync::coordinator::CycleOutcome::Complete =>
            {
                RefreshStatus::Requested
            }
            _ => RefreshStatus::Failed,
        };
    }

    Ok(result)
}
