use crate::components::identity::api::{DeviceRegistration, IdentityApi};
use crate::platform::{
    config::APIConfig,
    http::{APIFailure, APITransport},
    secrets::{CredentialKey, CredentialKind, Secret, SecretStore},
};
use crate::shared_kernel::id::{DeviceId, UserId};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    sync::{LazyLock, Mutex},
};

#[derive(Debug)]
pub enum CredentialFailure {
    Busy,
    StorageUnavailable,
    RecoveryRequired,
    Remote(APIFailure),
}

pub struct DeviceCredential {
    pub device_id: DeviceId,
    pub user_id: UserId,
    pub(crate) token: Secret,
    // Keep logout from deleting this credential during account activation.
    activation: Option<ActivationGuard>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
enum CredentialState {
    Issuing,
    Issued {
        device_id: DeviceId,
        user_id: UserId,
    },
}

// Only credential bootstrap uses this guard. T13 owns the full sync coordinator.
// It prevents two calls in this process from both issuing a new credential.
static ACTIVATING: LazyLock<Mutex<HashSet<CredentialKey>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

struct ActivationGuard(CredentialKey);

impl Drop for ActivationGuard {
    fn drop(&mut self) {
        if let Ok(mut active) = ACTIVATING.lock() {
            active.remove(&self.0);
        }
    }
}

/// The locator has no token. The token's key includes the API-issued device ID.
pub struct DeviceCredentials<'a, S> {
    pub config: &'a APIConfig,
    pub profile: &'a str,
    pub identity: (&'a str, &'a str),
    pub secrets: &'a S,
}

impl<S: SecretStore> DeviceCredentials<'_, S> {
    fn guard(&self) -> Result<ActivationGuard, CredentialFailure> {
        let key = self.key(None, CredentialKind::ApiDeviceState)?;

        if !ACTIVATING
            .lock()
            .map_err(|_| CredentialFailure::StorageUnavailable)?
            .insert(key.clone())
        {
            return Err(CredentialFailure::Busy);
        }

        Ok(ActivationGuard(key))
    }
    fn key(
        &self,
        device: Option<&DeviceId>,
        kind: CredentialKind,
    ) -> Result<CredentialKey, CredentialFailure> {
        CredentialKey::new(self.config, self.profile, self.identity, None, device, kind)
            .map_err(|_| CredentialFailure::StorageUnavailable)
    }

    fn state(&self) -> Result<Option<CredentialState>, CredentialFailure> {
        let key = self.key(None, CredentialKind::ApiDeviceState)?;
        self.secrets
            .read(&key)
            .map_err(|_| CredentialFailure::StorageUnavailable)?
            .map(|value| {
                serde_json::from_str(value.expose())
                    .map_err(|_| CredentialFailure::RecoveryRequired)
            })
            .transpose()
    }

    fn save_state(&self, state: &CredentialState) -> Result<(), CredentialFailure> {
        let encoded =
            serde_json::to_string(state).map_err(|_| CredentialFailure::StorageUnavailable)?;
        let secret = Secret::new(encoded).map_err(|_| CredentialFailure::StorageUnavailable)?;

        self.secrets
            .write(&self.key(None, CredentialKind::ApiDeviceState)?, &secret)
            .map_err(|_| CredentialFailure::StorageUnavailable)
    }

    pub fn load(&self) -> Result<Option<DeviceCredential>, CredentialFailure> {
        match self.state()? {
            None => Ok(None),
            Some(CredentialState::Issuing) => Err(CredentialFailure::RecoveryRequired),
            Some(CredentialState::Issued { device_id, user_id }) => {
                let token = self
                    .secrets
                    .read(&self.key(Some(&device_id), CredentialKind::ApiDevice)?)
                    .map_err(|_| CredentialFailure::StorageUnavailable)?
                    .ok_or(CredentialFailure::RecoveryRequired)?;

                Ok(Some(DeviceCredential {
                    device_id,
                    user_id,
                    token,
                    activation: None,
                }))
            }
        }
    }

    pub async fn ensure(
        &self,
        http: &APITransport,
        oidc_access: &Secret,
        registration: &DeviceRegistration,
    ) -> Result<DeviceCredential, CredentialFailure> {
        let guard = self.guard()?;

        if let Some(mut credential) = self.load()? {
            credential.activation = Some(guard);
            return Ok(credential);
        }
        // A lost response cannot recover this one-time secret. Never issue again
        // just because a caller repeats the failed attempt or restarts the app.
        self.save_state(&CredentialState::Issuing)?;
        let issued = IdentityApi(http)
            .issue_device_token(oidc_access, registration)
            .await
            .map_err(CredentialFailure::Remote)?
            .value;
        self.save_state(&CredentialState::Issued {
            device_id: issued.device_id.clone(),
            user_id: issued.user_id.clone(),
        })?;
        self.secrets
            .write(
                &self.key(Some(&issued.device_id), CredentialKind::ApiDevice)?,
                &issued.access_token,
            )
            .map_err(|_| CredentialFailure::StorageUnavailable)?;

        Ok(DeviceCredential {
            device_id: issued.device_id,
            user_id: issued.user_id,
            token: issued.access_token,
            activation: Some(guard),
        })
    }

    /// Explicit local logout can clear a failed bootstrap. It never revokes remotely.
    pub fn forget(&self) -> Result<(), CredentialFailure> {
        let _guard = self.guard()?;

        if let Some(CredentialState::Issued { device_id, .. }) = self.state()? {
            self.secrets
                .delete(&self.key(Some(&device_id), CredentialKind::ApiDevice)?)
                .map_err(|_| CredentialFailure::StorageUnavailable)?;
        }

        self.secrets
            .delete(&self.key(None, CredentialKind::ApiDeviceState)?)
            .map_err(|_| CredentialFailure::StorageUnavailable)
    }
}
