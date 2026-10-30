use crate::shared_kernel::id::DeviceId;
use crate::shared_kernel::{NightingaleError, Result};
use rusqlite::{Connection, OptionalExtension, Transaction, params};

/// API-issued ID and public installation metadata. Credentials have a separate owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInstallation {
    pub device_id: DeviceId,
    pub display_name: String,
    pub platform: String,
    pub client_version: String,
}

pub trait InstallationStore {
    fn save(&self, tx: &Transaction, installation: &DeviceInstallation) -> Result<()>;
    fn get(&self, connection: &Connection, device: &DeviceId)
    -> Result<Option<DeviceInstallation>>;
}

pub struct SqliteInstallationStore;

impl InstallationStore for SqliteInstallationStore {
    fn save(&self, tx: &Transaction, installation: &DeviceInstallation) -> Result<()> {
        tx.execute("INSERT INTO device_installation (device_id,display_name,platform,client_version) VALUES (?1,?2,?3,?4)
            ON CONFLICT(device_id) DO UPDATE SET display_name=excluded.display_name,platform=excluded.platform,client_version=excluded.client_version",
            params![installation.device_id.as_str(),installation.display_name,installation.platform,installation.client_version])
            .map_err(|error| NightingaleError::internal(error.to_string()))?;

        Ok(())
    }

    fn get(
        &self,
        connection: &Connection,
        device: &DeviceId,
    ) -> Result<Option<DeviceInstallation>> {
        connection.query_row("SELECT display_name,platform,client_version FROM device_installation WHERE device_id=?1", [device.as_str()],
            |row| Ok(DeviceInstallation {device_id:device.clone(),display_name:row.get(0)?,platform:row.get(1)?,client_version:row.get(2)?}))
            .optional().map_err(|error| NightingaleError::internal(error.to_string()))
    }
}
pub mod activation;
