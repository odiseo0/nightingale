//! Specific account commands. IPC never accepts a cloud scope or envelope.
use super::super::{
    application::{self, AccountChange, AccountError, AccountMutationResult, Result},
    domain::{
        AccountDetailsPatch, AccountPolicy, AccountStatus, AccountView, Preference, PreferenceKey,
        Preferences, Theme,
    },
};
use super::{desktop, outbound::SqliteAccountRepository};
use crate::{
    components::identity::{domain::AccountAccess, repository::SqliteAccessStore},
    shared_kernel::SystemClock,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Mutex};
use tauri::{AppHandle, State};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreferencesDto {
    pub theme: String,
    pub services_show_inactive: bool,
}

impl From<Preferences> for PreferencesDto {
    fn from(value: Preferences) -> Self {
        Self {
            theme: value.theme.as_str().to_owned(),
            services_show_inactive: value.services_show_inactive,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountPolicyDto {
    pub no_show_grace_minutes: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountViewDto {
    pub account_id: String,
    pub display_name: String,
    pub time_zone: String,
    pub location_name: String,
    pub location_address: String,
    pub location_city: String,
    pub location_state: String,
    pub location_country: String,
    pub policy: AccountPolicyDto,
    pub status: String,
    pub version: i64,
    pub updated_at: Option<String>,
}

impl From<AccountView> for AccountViewDto {
    fn from(view: AccountView) -> Self {
        Self {
            account_id: view.account_id.as_str().to_owned(),
            display_name: view.display_name,
            time_zone: view.time_zone,
            location_name: view.location_name,
            location_address: view.location_address,
            location_city: view.location_city,
            location_state: view.location_state,
            location_country: view.location_country,
            policy: AccountPolicyDto {
                no_show_grace_minutes: view.policy.no_show_grace_minutes(),
            },
            status: view.status.as_str().to_owned(),
            version: view.version,
            updated_at: view.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountScreenDto {
    pub account: Option<AccountViewDto>,
    pub preferences: PreferencesDto,
    pub can_manage: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreferenceRequest {
    pub key: String,
    pub value: serde_json::Value,
}

impl PreferenceRequest {
    fn preference(self) -> Result<Preference> {
        match PreferenceKey::try_from(self.key.as_str())? {
            PreferenceKey::Theme => Ok(Preference::Theme(Theme::try_from(
                self.value.as_str().ok_or_else(|| {
                    AccountError::new("VALIDATION_ERROR", "el tema debe ser texto")
                })?,
            )?)),
            PreferenceKey::ServicesShowInactive => Ok(Preference::ServicesShowInactive(
                self.value.as_bool().ok_or_else(|| {
                    AccountError::new("VALIDATION_ERROR", "la preferencia debe ser booleana")
                })?,
            )),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DetailsRequest {
    pub expected_version: i64,
    pub patch: BTreeMap<String, String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PolicyRequest {
    pub expected_version: i64,
    pub no_show_grace_minutes: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StatusRequest {
    pub expected_version: i64,
    pub status: String,
}

fn selected_access(connection: &Connection) -> Result<AccountAccess> {
    SqliteAccessStore
        .selected_access(connection)?
        .ok_or_else(|| {
            AccountError::new(
                "AUTHENTICATION_REQUIRED",
                "inicie sesión y elija una cuenta",
            )
        })
}

fn locked() -> AccountError {
    AccountError::new("INTERNAL_ERROR", "base local no disponible")
}

fn read_view(connection: &mut Connection) -> Result<AccountScreenDto> {
    let access = selected_access(connection)?;
    let repository = SqliteAccountRepository {
        connection,
        clock: &SystemClock,
    };
    let screen = application::get_account_view(&repository, &access)?;

    Ok(AccountScreenDto {
        account: screen.account.map(Into::into),
        preferences: screen.preferences.into(),
        can_manage: screen.can_manage,
    })
}

#[tauri::command]
pub fn account_get_view(connection: State<Mutex<Connection>>) -> Result<AccountScreenDto> {
    read_view(&mut *connection.lock().map_err(|_| locked())?)
}

#[tauri::command]
pub fn account_get_preferences(connection: State<Mutex<Connection>>) -> Result<PreferencesDto> {
    let mut connection = connection.lock().map_err(|_| locked())?;
    let access = selected_access(&connection)?;
    let repository = SqliteAccountRepository {
        connection: &mut connection,
        clock: &SystemClock,
    };

    Ok(application::get_preferences(&repository, &access)?.into())
}

#[tauri::command]
pub fn account_set_preference(
    connection: State<Mutex<Connection>>,
    request: PreferenceRequest,
) -> Result<PreferencesDto> {
    let preference = request.preference()?;
    let mut connection = connection.lock().map_err(|_| locked())?;
    let access = selected_access(&connection)?;
    let mut repository = SqliteAccountRepository {
        connection: &mut connection,
        clock: &SystemClock,
    };

    Ok(application::set_preference(&mut repository, &access, preference)?.into())
}

#[tauri::command]
pub async fn account_update_details(
    app: AppHandle,
    request: DetailsRequest,
) -> Result<AccountMutationResult> {
    desktop::execute(
        app,
        request.expected_version,
        AccountChange::Details(AccountDetailsPatch::new(request.patch)?),
    )
    .await
}

#[tauri::command]
pub async fn account_update_policy(
    app: AppHandle,
    request: PolicyRequest,
) -> Result<AccountMutationResult> {
    desktop::execute(
        app,
        request.expected_version,
        AccountChange::Policy(AccountPolicy::new(request.no_show_grace_minutes)?),
    )
    .await
}

#[tauri::command]
pub async fn account_set_status(
    app: AppHandle,
    request: StatusRequest,
) -> Result<AccountMutationResult> {
    desktop::execute(
        app,
        request.expected_version,
        AccountChange::Status(AccountStatus::try_from(request.status.as_str())?),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn requests_reject_scope_injection_fractional_policy_and_bad_preferences() {
        assert!(
            serde_json::from_value::<PolicyRequest>(
                json!({"expectedVersion":1,"noShowGraceMinutes":1.5})
            )
            .is_err()
        );
        assert!(
            serde_json::from_value::<PolicyRequest>(
                json!({"expectedVersion":1,"noShowGraceMinutes":15,"accountId":"other"})
            )
            .is_err()
        );
        assert!(
            serde_json::from_value::<DetailsRequest>(
                json!({"expectedVersion":1,"patch":{"displayName":null}})
            )
            .is_err()
        );
        for (key, value) in [
            ("timeZone", json!("UTC")),
            ("theme", json!(true)),
            ("servicesShowInactive", json!("true")),
        ] {
            assert!(
                PreferenceRequest {
                    key: key.into(),
                    value
                }
                .preference()
                .is_err()
            );
        }
    }

    #[test]
    fn account_commands_have_specific_permissions_and_registration() {
        let build = include_str!("../../../../build.rs");
        let lib = include_str!("../../../lib.rs");
        let capabilities: serde_json::Value =
            serde_json::from_str(include_str!("../../../../capabilities/default.json")).unwrap();
        let permissions = capabilities["permissions"].as_array().unwrap();
        for command in [
            "account_get_view",
            "account_get_preferences",
            "account_set_preference",
            "account_update_details",
            "account_update_policy",
            "account_set_status",
        ] {
            assert!(build.contains(&format!("\"{command}\"")));
            assert!(lib.contains(&format!(
                "components::account::adapters::inbound::{command}"
            )));
            assert!(permissions.contains(&json!(format!("allow-{}", command.replace('_', "-")))));
        }
    }

    #[test]
    fn signed_out_read_returns_no_account_and_error_has_no_transport_state() {
        let mut connection =
            crate::platform::sqlite::open(std::path::Path::new(":memory:")).unwrap();
        let error = read_view(&mut connection).unwrap_err();
        let value = serde_json::to_value(error).unwrap();
        assert_eq!(
            value,
            json!({"code":"AUTHENTICATION_REQUIRED","message":"inicie sesión y elija una cuenta"})
        );
        let preferences =
            serde_json::to_value(PreferencesDto::from(Preferences::default())).unwrap();
        assert_eq!(
            preferences,
            json!({"theme":"system","servicesShowInactive":true})
        );
    }
}
