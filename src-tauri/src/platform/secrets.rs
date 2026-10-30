use super::config::APIConfig;
use crate::shared_kernel::id::{AccountId, DeviceId};
use crate::shared_kernel::{NightingaleError, Result};
use sha2::{Digest, Sha256};

/// This type has no Debug, Display or Serialize implementation.
///
/// ```compile_fail
/// use nightingale_lib::platform::secrets::Secret;
/// let secret = Secret::new("synthetic".into()).unwrap();
/// println!("{secret:?}");
/// ```
pub struct Secret(String);

impl<'de> serde::Deserialize<'de> for Secret {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let value = <String as serde::Deserialize>::deserialize(deserializer)?;

        Self::new(value).map_err(serde::de::Error::custom)
    }
}

impl Secret {
    pub fn new(value: String) -> Result<Self> {
        if value.is_empty() {
            return Err(NightingaleError::validation("credencial vacía"));
        }

        Ok(Self(value))
    }

    /// Use only at a protocol or secure-store boundary.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy)]
pub enum CredentialKind {
    OidcAccess,
    OidcRefresh,
    APIDevice,
    APIDeviceState,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CredentialKey {
    service: &'static str,
    username: String,
}

impl CredentialKey {
    /// Profile separates local data directories, including before API assigns a device.
    /// Identity is the verified issuer/subject pair; do not pass a token here.
    pub fn new(
        config: &APIConfig,
        profile: &str,
        identity: (&str, &str),
        account: Option<&AccountId>,
        device: Option<&DeviceId>,
        kind: CredentialKind,
    ) -> Result<Self> {
        if profile.is_empty() || identity.0.is_empty() || identity.1.is_empty() {
            return Err(NightingaleError::validation(
                "alcance de credencial incompleto",
            ));
        }

        let scope = serde_json::to_vec(&(
            config.base_url().as_str(),
            profile,
            identity,
            account.map(AccountId::as_str),
            device.map(DeviceId::as_str),
        ))
        .map_err(|_| NightingaleError::internal("no se pudo formar el alcance de credencial"))?;
        let service = match kind {
            CredentialKind::OidcAccess => "org.nightingale.desktop.oidc-access",
            CredentialKind::OidcRefresh => "org.nightingale.desktop.oidc-refresh",
            CredentialKind::APIDevice => "org.nightingale.desktop.api-device",
            CredentialKind::APIDeviceState => "org.nightingale.desktop.api-device-state",
        };

        Ok(Self {
            service,
            username: format!("{:x}", Sha256::digest(scope)),
        })
    }
}

pub trait SecretStore {
    fn read(&self, key: &CredentialKey) -> Result<Option<Secret>>;
    /// Replaces the value at the same key, including during token rotation.
    fn write(&self, key: &CredentialKey, secret: &Secret) -> Result<()>;
    fn delete(&self, key: &CredentialKey) -> Result<()>;
}

pub struct NativeSecretStore;

fn store_error(_: keyring::Error) -> NightingaleError {
    // Platform errors can contain raw secret bytes. Never forward their text.
    // https://docs.rs/keyring/4.2.0/keyring/v1/enum.Error.html
    NightingaleError::temporary("no se pudo acceder al almacén seguro del sistema")
}

impl NativeSecretStore {
    fn entry(key: &CredentialKey) -> Result<keyring::Entry> {
        // v1 selects Credential Manager, Keychain or Secret Service for the host.
        // https://docs.rs/keyring/4.2.0/keyring/v1/index.html
        keyring::Entry::new(key.service, &key.username).map_err(store_error)
    }
}

impl SecretStore for NativeSecretStore {
    fn read(&self, key: &CredentialKey) -> Result<Option<Secret>> {
        match Self::entry(key)?.get_password() {
            Ok(value) => Secret::new(value).map(Some),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(store_error(error)),
        }
    }

    fn write(&self, key: &CredentialKey, secret: &Secret) -> Result<()> {
        Self::entry(key)?
            .set_password(secret.expose())
            .map_err(store_error)
    }

    fn delete(&self, key: &CredentialKey) -> Result<()> {
        match Self::entry(key)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(store_error(error)),
        }
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use super::*;
    use std::{collections::HashMap, sync::Mutex};

    #[derive(Default)]
    pub struct MemorySecretStore(Mutex<HashMap<CredentialKey, Secret>>);

    impl SecretStore for MemorySecretStore {
        fn read(&self, key: &CredentialKey) -> Result<Option<Secret>> {
            let values = self
                .0
                .lock()
                .map_err(|_| NightingaleError::internal("almacén de prueba bloqueado"))?;

            values
                .get(key)
                .map(|secret| Secret::new(secret.expose().to_owned()))
                .transpose()
        }

        fn write(&self, key: &CredentialKey, secret: &Secret) -> Result<()> {
            self.0
                .lock()
                .map_err(|_| NightingaleError::internal("almacén de prueba bloqueado"))?
                .insert(key.clone(), Secret::new(secret.expose().to_owned())?);

            Ok(())
        }

        fn delete(&self, key: &CredentialKey) -> Result<()> {
            self.0
                .lock()
                .map_err(|_| NightingaleError::internal("almacén de prueba bloqueado"))?
                .remove(key);

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(profile: &str, kind: CredentialKind) -> CredentialKey {
        CredentialKey::new(
            &APIConfig::parse("https://api.example/").unwrap(),
            profile,
            ("https://issuer.example", "synthetic-user"),
            None,
            None,
            kind,
        )
        .unwrap()
    }

    fn store_lifecycle(store: &impl SecretStore, key: &CredentialKey) {
        assert!(store.read(key).unwrap().is_none());
        store
            .write(key, &Secret::new("synthetic-first".into()).unwrap())
            .unwrap();
        assert!(store.read(key).unwrap().unwrap().expose() == "synthetic-first");
        store
            .write(key, &Secret::new("synthetic-rotated".into()).unwrap())
            .unwrap();
        assert!(store.read(key).unwrap().unwrap().expose() == "synthetic-rotated");
        store.delete(key).unwrap();
        assert!(store.read(key).unwrap().is_none());
        store.delete(key).unwrap();
    }

    #[test]
    fn fake_supports_read_rotation_and_idempotent_delete() {
        store_lifecycle(
            &testing::MemorySecretStore::default(),
            &key("test", CredentialKind::APIDevice),
        );
    }

    #[test]
    fn scopes_keep_protocols_profiles_environments_identities_accounts_and_devices_apart() {
        let store = testing::MemorySecretStore::default();
        let base = key("test", CredentialKind::APIDevice);
        store
            .write(&base, &Secret::new("synthetic-private".into()).unwrap())
            .unwrap();
        let config = APIConfig::parse("https://api.example/").unwrap();
        let other_environment = APIConfig::parse("https://other.example/").unwrap();
        let account =
            AccountId::try_from("019a1234-0000-7000-8000-000000000001".to_owned()).unwrap();
        let device = DeviceId::try_from("019a1234-0000-7000-8000-000000000002".to_owned()).unwrap();
        for different in [
            key("test", CredentialKind::OidcAccess),
            key("test", CredentialKind::OidcRefresh),
            key("other-profile", CredentialKind::APIDevice),
            CredentialKey::new(
                &other_environment,
                "test",
                ("https://issuer.example", "synthetic-user"),
                None,
                None,
                CredentialKind::APIDevice,
            )
            .unwrap(),
            CredentialKey::new(
                &config,
                "test",
                ("https://other-issuer.example", "synthetic-user"),
                None,
                None,
                CredentialKind::APIDevice,
            )
            .unwrap(),
            CredentialKey::new(
                &config,
                "test",
                ("https://issuer.example", "other-user"),
                None,
                None,
                CredentialKind::APIDevice,
            )
            .unwrap(),
            CredentialKey::new(
                &config,
                "test",
                ("https://issuer.example", "synthetic-user"),
                Some(&account),
                None,
                CredentialKind::APIDevice,
            )
            .unwrap(),
            CredentialKey::new(
                &config,
                "test",
                ("https://issuer.example", "synthetic-user"),
                None,
                Some(&device),
                CredentialKind::APIDevice,
            )
            .unwrap(),
        ] {
            assert!(store.read(&different).unwrap().is_none());
            store.delete(&different).unwrap();
        }
        assert!(store.read(&base).unwrap().is_some());
    }

    #[test]
    fn errors_hide_platform_secret_bytes() {
        let error = store_error(keyring::Error::BadEncoding(b"synthetic-private".to_vec()));
        assert!(!format!("{error:?}").contains("synthetic-private"));
        assert!(
            !serde_json::to_string(&error)
                .unwrap()
                .contains("synthetic-private")
        );
        assert!(Secret::new(String::new()).is_err());
    }

    #[test]
    #[ignore = "uses the real OS store with a unique synthetic entry, then deletes it"]
    fn native_store_round_trip_rotation_and_delete() {
        struct Cleanup(CredentialKey);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = NativeSecretStore.delete(&self.0);
            }
        }
        let key = key(
            &format!("smoke-{}", uuid::Uuid::now_v7()),
            CredentialKind::APIDevice,
        );
        let _cleanup = Cleanup(key.clone());
        store_lifecycle(&NativeSecretStore, &key);
    }
}
