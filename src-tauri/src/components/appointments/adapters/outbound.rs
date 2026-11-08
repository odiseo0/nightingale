//! SQLite read access to the accepted appointment occupancy projection.
use super::super::application::{OccupancyInterval, ScheduleOccupancyQuery};
use crate::shared_kernel::{NightingaleError, Result, id::OperationContext};
use jiff::Timestamp;
use rusqlite::{Connection, params};
use std::str::FromStr;

fn storage_error(_: rusqlite::Error) -> NightingaleError {
    NightingaleError::internal("datos locales de ocupación no disponibles")
}

/// Scope-bound read-only occupancy query. It exposes no SQL to consumers and
/// never writes: intentions and accepted views have distinct writers.
pub struct SqliteScheduleOccupancyQuery<'a> {
    pub connection: &'a Connection,
    pub scope: &'a OperationContext,
}

impl ScheduleOccupancyQuery for SqliteScheduleOccupancyQuery<'_> {
    fn list_occupancy(
        &self,
        professional_id: &str,
        from: Timestamp,
        until: Timestamp,
    ) -> Result<Vec<OccupancyInterval>> {
        let mut query = self
            .connection
            .prepare(
                "SELECT appointment_id,starts_at,ends_at,status FROM appointment_occupancy_view \
                 WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND professional_id=?4 \
                 AND starts_at < ?5 AND ends_at > ?6 ORDER BY starts_at,appointment_id",
            )
            .map_err(storage_error)?;
        let rows = query
            .query_map(
                params![
                    self.scope.account_id.as_str(),
                    self.scope.user_id.as_str(),
                    self.scope.device_id.as_str(),
                    professional_id,
                    until.to_string(),
                    from.to_string()
                ],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .map_err(storage_error)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(storage_error)?;

        rows.iter()
            .map(|(id, starts, ends, status)| {
                let starts_at = Timestamp::from_str(starts).map_err(|_| {
                    NightingaleError::internal("ocupación local con instante inválido")
                })?;
                let ends_at = Timestamp::from_str(ends).map_err(|_| {
                    NightingaleError::internal("ocupación local con instante inválido")
                })?;

                OccupancyInterval::new(id.clone(), starts_at, ends_at, status)
            })
            .collect()
    }
}
