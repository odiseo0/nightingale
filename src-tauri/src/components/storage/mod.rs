use crate::shared_kernel::{NightingaleError, Result};
use rusqlite::Connection;
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageStatus {
    schema_version: i64,
    application_id: i64,
}

fn read_status(connection: &Connection) -> Result<StorageStatus> {
    let schema_version = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|error| NightingaleError::internal(error.to_string()))?;
    let application_id = connection
        .pragma_query_value(None, "application_id", |row| row.get(0))
        .map_err(|error| NightingaleError::internal(error.to_string()))?;

    Ok(StorageStatus {
        schema_version,
        application_id,
    })
}

// IPC convention: one command per use case; fallible commands return
// Result<Dto, NightingaleError> so the UI receives a stable {code,message} body.
#[tauri::command]
pub fn storage_get_status(connection: State<Mutex<Connection>>) -> Result<StorageStatus> {
    let connection = connection
        .lock()
        .map_err(|_| NightingaleError::internal("no se pudo acceder a la base local"))?;

    read_status(&connection)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_kernel::ErrorCode;

    #[test]
    fn reports_empty_database_status() {
        let connection = Connection::open_in_memory().unwrap();

        let status = read_status(&connection).unwrap();

        assert_eq!(status.schema_version, 0);
        assert_eq!(status.application_id, 0);
    }

    #[test]
    fn poisoned_lock_maps_to_internal_error() {
        let mutex = Mutex::new(Connection::open_in_memory().unwrap());
        let poison = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = mutex.lock().unwrap();
            panic!("poison");
        }));
        assert!(poison.is_err());

        let result = mutex
            .lock()
            .map_err(|_| NightingaleError::internal("no se pudo acceder a la base local"))
            .and_then(|connection| read_status(&connection));

        assert_eq!(result.unwrap_err().code, ErrorCode::InternalError);
    }
}
