//! Public availability/occupancy contracts provided to other components.
//! This module deliberately contains no appointment write use cases: only the
//! accepted occupancy projection and its read port live here for now.
use crate::shared_kernel::{NightingaleError, Result};
use jiff::Timestamp;

/// An accepted occupancy interval of one appointment. Half-open `[startsAt,endsAt)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OccupancyInterval {
    pub appointment_id: String,
    pub starts_at: Timestamp,
    pub ends_at: Timestamp,
    pub status: String,
}

impl OccupancyInterval {
    pub fn new(
        appointment_id: String,
        starts_at: Timestamp,
        ends_at: Timestamp,
        status: &str,
    ) -> Result<Self> {
        if starts_at >= ends_at {
            return Err(NightingaleError::validation(
                "la ocupación exige inicio menor que fin",
            ));
        }

        match status {
            "scheduled" | "confirmed" | "completed" | "no_show" => Ok(Self {
                appointment_id,
                starts_at,
                ends_at,
                status: status.into(),
            }),
            _ => Err(NightingaleError::validation(
                "estado de ocupación inválido; cancelled no ocupa",
            )),
        }
    }
}

/// Read-only occupancy port owned by appointments. Availability subtracts the
/// intervals returned here; the future provisional occupancy of appointments
/// will enter through the same port without changing consumers.
pub trait ScheduleOccupancyQuery {
    /// Accepted occupancy of one professional intersecting `[from,until)`.
    /// Contiguous appointments (end == next start) are both returned and are
    /// not treated as overlapping.
    fn list_occupancy(
        &self,
        professional_id: &str,
        from: Timestamp,
        until: Timestamp,
    ) -> Result<Vec<OccupancyInterval>>;
}
