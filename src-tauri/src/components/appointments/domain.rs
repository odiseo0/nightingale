//! Appointment lifecycle values, independent of storage and remote transport.
//! Time semantics follow the approved plan and contract §5.4: RFC 3339 instants
//! with minute precision, real duration 1–1440 minutes, half-open `[start,end)`
//! intervals, terminal states and no free status assignment.
use crate::shared_kernel::{NightingaleError, id::canonical_uuid_v7};
use jiff::{Timestamp, civil::Date};
use std::str::FromStr;

macro_rules! appointments_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name(String);

        impl TryFrom<String> for $name {
            type Error = NightingaleError;

            fn try_from(value: String) -> std::result::Result<Self, NightingaleError> {
                canonical_uuid_v7(&value)?;

                Ok(Self(value))
            }
        }

        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

appointments_id!(AppointmentId);

/// Stable machine-readable boundary codes for the appointments component.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppointmentsError {
    pub code: String,
    pub message: String,
}

pub type Result<T> = std::result::Result<T, AppointmentsError>;

impl AppointmentsError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new("VALIDATION_ERROR", message)
    }

    pub fn invalid_state(message: impl Into<String>) -> Self {
        Self::new("INVALID_STATE_TRANSITION", message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("INTERNAL_ERROR", message)
    }
}

impl From<NightingaleError> for AppointmentsError {
    fn from(error: NightingaleError) -> Self {
        let code = serde_json::to_value(error.code)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_else(|| "INTERNAL_ERROR".into());

        if code == "INTERNAL_ERROR" {
            Self::internal("datos locales de citas no disponibles")
        } else {
            Self::new(&code, error.message)
        }
    }
}

/// Business state of an appointment, kept separate from sync state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppointmentStatus {
    Scheduled,
    Confirmed,
    Cancelled,
    Completed,
    NoShow,
}

impl AppointmentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Scheduled => "scheduled",
            Self::Confirmed => "confirmed",
            Self::Cancelled => "cancelled",
            Self::Completed => "completed",
            Self::NoShow => "no_show",
        }
    }

    /// Terminal states never accept another transition.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Cancelled | Self::Completed | Self::NoShow)
    }

    /// Occupying states per contract §5.4; only `cancelled` releases the slot.
    pub fn occupies(self) -> bool {
        !matches!(self, Self::Cancelled)
    }
}

impl TryFrom<&str> for AppointmentStatus {
    type Error = AppointmentsError;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "scheduled" => Ok(Self::Scheduled),
            "confirmed" => Ok(Self::Confirmed),
            "cancelled" => Ok(Self::Cancelled),
            "completed" => Ok(Self::Completed),
            "no_show" => Ok(Self::NoShow),
            _ => Err(AppointmentsError::validation(
                "estado de cita inválido; se requiere scheduled, confirmed, cancelled, completed o no_show",
            )),
        }
    }
}

/// History entry action; the desktop only produces `staff` actors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryAction {
    Created,
    Confirmed,
    Rescheduled,
    Cancelled,
    Completed,
    NoShow,
}

impl HistoryAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Confirmed => "confirmed",
            Self::Rescheduled => "rescheduled",
            Self::Cancelled => "cancelled",
            Self::Completed => "completed",
            Self::NoShow => "no_show",
        }
    }
}

impl TryFrom<&str> for HistoryAction {
    type Error = AppointmentsError;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "created" => Ok(Self::Created),
            "confirmed" => Ok(Self::Confirmed),
            "rescheduled" => Ok(Self::Rescheduled),
            "cancelled" => Ok(Self::Cancelled),
            "completed" => Ok(Self::Completed),
            "no_show" => Ok(Self::NoShow),
            _ => Err(AppointmentsError::validation(
                "acción de historia inválida; se requiere created, confirmed, rescheduled, cancelled, completed o no_show",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorKind {
    Staff,
    Patient,
    Contact,
}

impl ActorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Staff => "staff",
            Self::Patient => "patient",
            Self::Contact => "contact",
        }
    }
}

impl TryFrom<&str> for ActorKind {
    type Error = AppointmentsError;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "staff" => Ok(Self::Staff),
            "patient" => Ok(Self::Patient),
            "contact" => Ok(Self::Contact),
            _ => Err(AppointmentsError::validation(
                "actorKind debe ser staff, patient o contact",
            )),
        }
    }
}

/// Optional change reason: present values must be non-empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeReason(String);

impl ChangeReason {
    pub fn new(value: Option<String>) -> Result<Option<Self>> {
        match value {
            None => Ok(None),
            Some(text) if text.trim().is_empty() => Err(AppointmentsError::validation(
                "changeReason no puede estar vacío",
            )),
            Some(text) => Ok(Some(Self(text))),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// An instant normalized to UTC with minute precision: seconds and sub-second
/// parts must be zero, matching the appointment contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MinuteInstant(Timestamp);

impl MinuteInstant {
    pub fn parse(value: &str) -> Result<Self> {
        let ts = Timestamp::from_str(value).map_err(|_| {
            AppointmentsError::validation("el instante debe ser RFC 3339 con offset o Z")
        })?;

        Self::validate(ts)
    }

    fn validate(ts: Timestamp) -> Result<Self> {
        // Minute precision: the contract stores whole minutes only.
        // Source: https://docs.rs/jiff/0.2.38/jiff/struct.Timestamp.html
        if ts.as_second() % 60 != 0 || ts.subsec_nanosecond() != 0 {
            return Err(AppointmentsError::validation(
                "los instantes de cita exigen precisión de minuto",
            ));
        }

        Ok(Self(ts))
    }

    pub fn timestamp(self) -> Timestamp {
        self.0
    }
}

/// A half-open `[start,end)` appointment interval whose real duration is
/// 1–1440 minutes. The service reference duration may differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppointmentInterval {
    pub starts_at: MinuteInstant,
    pub ends_at: MinuteInstant,
}

impl AppointmentInterval {
    pub fn new(starts_at: MinuteInstant, ends_at: MinuteInstant) -> Result<Self> {
        if starts_at >= ends_at {
            return Err(AppointmentsError::validation(
                "el instante de inicio debe ser menor que el de fin",
            ));
        }

        // Timestamp subtraction yields a Span; its total whole minutes give the
        // real duration. Source: https://docs.rs/jiff/0.2.38/jiff/struct.Span.html
        let minutes = (ends_at.0 - starts_at.0)
            .total(jiff::Unit::Minute)
            .map_err(|_| AppointmentsError::internal("duración fuera de rango"))?;

        if !(1.0..=1440.0).contains(&minutes) {
            return Err(AppointmentsError::validation(
                "la duración real debe ser de 1 a 1440 minutos",
            ));
        }

        Ok(Self { starts_at, ends_at })
    }

    pub fn duration_minutes(self) -> i64 {
        (self.ends_at.0 - self.starts_at.0)
            .total(jiff::Unit::Minute)
            .unwrap_or(0.0) as i64
    }
}

/// The nine reference snapshots captured at creation. The server recomputes
/// them on acceptance; the client never recalculates them on reschedule or
/// history projection.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshots {
    pub service_name: String,
    pub offered_duration_minutes: i64,
    pub professional_name: String,
    pub location_name: String,
    pub location_address: String,
    pub location_country: String,
    pub location_state: String,
    pub location_city: String,
    pub time_zone: String,
}

impl Snapshots {
    // Nine contract snapshots are validated together; the arity mirrors the
    // accepted appointment row, so grouping would only add indirection.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        service_name: String,
        offered_duration_minutes: i64,
        professional_name: String,
        location_name: String,
        location_address: String,
        location_country: String,
        location_state: String,
        location_city: String,
        time_zone: String,
    ) -> Result<Self> {
        for (label, value) in [
            ("serviceNameSnapshot", &service_name),
            ("professionalNameSnapshot", &professional_name),
            ("locationNameSnapshot", &location_name),
            ("locationAddressSnapshot", &location_address),
            ("locationStateSnapshot", &location_state),
            ("locationCitySnapshot", &location_city),
            ("timeZoneSnapshot", &time_zone),
        ] {
            if value.trim().is_empty() {
                return Err(AppointmentsError::validation(format!(
                    "{label} no puede estar vacío"
                )));
            }
        }

        if location_country.len() != 2 || !location_country.is_ascii() {
            return Err(AppointmentsError::validation(
                "locationCountrySnapshot debe ser un código ISO de dos letras",
            ));
        }

        if !(1..=1440).contains(&offered_duration_minutes) {
            return Err(AppointmentsError::validation(
                "offeredDurationMinutesSnapshot debe ser un entero entre 1 y 1440",
            ));
        }

        Ok(Self {
            service_name,
            offered_duration_minutes,
            professional_name,
            location_name,
            location_address,
            location_country,
            location_state,
            location_city,
            time_zone,
        })
    }
}

/// The six contract transitions as domain intents, not free status assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppointmentTransition {
    Create,
    Confirm,
    Reschedule,
    Cancel,
    Complete,
    MarkNoShow,
}

/// Applies one transition to a current state. `Create` expects no current
/// state. Repeating an operation on a terminal state is invalid here; the sync
/// layer reconciles a repeated operation with its original receipt instead.
pub fn apply_transition(
    transition: AppointmentTransition,
    current: Option<AppointmentStatus>,
) -> Result<AppointmentStatus> {
    use AppointmentStatus as S;

    let next = match (transition, current) {
        (AppointmentTransition::Create, None) => S::Scheduled,
        (AppointmentTransition::Create, Some(_)) => {
            return Err(AppointmentsError::invalid_state(
                "la cita ya existe; use una acción de ciclo de vida",
            ));
        }
        (AppointmentTransition::Confirm, Some(S::Scheduled)) => S::Confirmed,
        (AppointmentTransition::Reschedule, Some(S::Scheduled | S::Confirmed)) => S::Scheduled,
        (
            AppointmentTransition::Cancel
            | AppointmentTransition::Complete
            | AppointmentTransition::MarkNoShow,
            Some(S::Scheduled | S::Confirmed),
        ) => match transition {
            AppointmentTransition::Cancel => S::Cancelled,
            AppointmentTransition::Complete => S::Completed,
            AppointmentTransition::MarkNoShow => S::NoShow,
            _ => unreachable!(),
        },
        (_, Some(state)) if state.is_terminal() => {
            return Err(AppointmentsError::invalid_state(
                "el estado de la cita es terminal y no admite otra transición",
            ));
        }
        (_, Some(_)) => {
            return Err(AppointmentsError::invalid_state(
                "la transición no está permitida desde el estado actual",
            ));
        }
        (_, None) => {
            return Err(AppointmentsError::invalid_state(
                "la acción exige una cita existente",
            ));
        }
    };

    Ok(next)
}

/// Administrative age of a patient on the local civil date of the appointment.
/// A Feb 29 birthday falls on Mar 1 in non-leap years (contract §5.2).
pub fn age_on(birth_date: Date, on: Date) -> i64 {
    let feb29 = birth_date.month() == 2 && birth_date.day() == 29;
    let leap_year = Date::new(on.year(), 2, 29).is_ok();
    let (birth_month, birth_day) = if feb29 && !leap_year {
        // Administrative birthday falls on Mar 1 in non-leap years.
        (3, 1)
    } else {
        (birth_date.month(), birth_date.day())
    };

    let mut age = i64::from(on.year()) - i64::from(birth_date.year());

    if (on.month(), on.day()) < (birth_month, birth_day) {
        age -= 1;
    }

    age
}

/// A minor without their own document must be under 18 on the local date of
/// the appointment; adults with `own_document` are always age-eligible.
pub fn is_minor_at(birth_date: Date, on: Date) -> bool {
    age_on(birth_date, on) < 18
}

pub fn parse_local_date(value: &str) -> Result<Date> {
    Date::from_str(value)
        .map_err(|_| AppointmentsError::validation("la fecha civil debe ser ISO 8601 YYYY-MM-DD"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(value: &str) -> Date {
        parse_local_date(value).unwrap()
    }

    fn instant(value: &str) -> MinuteInstant {
        MinuteInstant::parse(value).unwrap()
    }

    fn scheduled() -> AppointmentStatus {
        AppointmentStatus::Scheduled
    }

    #[test]
    fn status_contract_strings_terminals_and_occupancy() {
        assert_eq!(
            AppointmentStatus::try_from("scheduled").unwrap(),
            AppointmentStatus::Scheduled
        );
        assert_eq!(
            AppointmentStatus::try_from("no_show").unwrap().as_str(),
            "no_show"
        );
        assert!(AppointmentStatus::try_from("reschedule_requested").is_err());

        for terminal in [
            AppointmentStatus::Cancelled,
            AppointmentStatus::Completed,
            AppointmentStatus::NoShow,
        ] {
            assert!(terminal.is_terminal());
            assert!(apply_transition(AppointmentTransition::Confirm, Some(terminal)).is_err());
        }
        assert!(AppointmentStatus::Scheduled.occupies());
        assert!(AppointmentStatus::Confirmed.occupies());
        assert!(AppointmentStatus::Completed.occupies());
        assert!(AppointmentStatus::NoShow.occupies());
        assert!(!AppointmentStatus::Cancelled.occupies());
    }

    #[test]
    fn transitions_follow_the_contract_table() {
        // create sin cita
        assert_eq!(
            apply_transition(AppointmentTransition::Create, None).unwrap(),
            AppointmentStatus::Scheduled
        );
        // create sobre existente
        assert!(apply_transition(AppointmentTransition::Create, Some(scheduled())).is_err());
        // confirm
        assert_eq!(
            apply_transition(AppointmentTransition::Confirm, Some(scheduled())).unwrap(),
            AppointmentStatus::Confirmed
        );
        let err = apply_transition(
            AppointmentTransition::Confirm,
            Some(AppointmentStatus::Confirmed),
        )
        .unwrap_err();
        assert_eq!(err.code, "INVALID_STATE_TRANSITION");
        // reschedule desde scheduled o confirmed
        for state in [AppointmentStatus::Scheduled, AppointmentStatus::Confirmed] {
            assert_eq!(
                apply_transition(AppointmentTransition::Reschedule, Some(state)).unwrap(),
                AppointmentStatus::Scheduled
            );
        }
        // cancel/complete/no_show desde scheduled o confirmed
        for state in [AppointmentStatus::Scheduled, AppointmentStatus::Confirmed] {
            assert_eq!(
                apply_transition(AppointmentTransition::Cancel, Some(state)).unwrap(),
                AppointmentStatus::Cancelled
            );
            assert_eq!(
                apply_transition(AppointmentTransition::Complete, Some(state)).unwrap(),
                AppointmentStatus::Completed
            );
            assert_eq!(
                apply_transition(AppointmentTransition::MarkNoShow, Some(state)).unwrap(),
                AppointmentStatus::NoShow
            );
        }
        // action without an appointment
        assert!(apply_transition(AppointmentTransition::Cancel, None).is_err());
    }

    #[test]
    fn intervals_require_minute_precision_and_real_duration() {
        let start = instant("2026-04-01T09:00:00-04:00");
        let end = instant("2026-04-01T17:00:00Z");
        let interval = AppointmentInterval::new(start, end).unwrap();
        assert_eq!(interval.duration_minutes(), 240);

        // offsets distintos normalizados a UTC
        assert_eq!(
            interval.starts_at.timestamp().to_string(),
            "2026-04-01T13:00:00Z"
        );

        // non-zero seconds
        assert!(MinuteInstant::parse("2026-04-01T09:00:30Z").is_err());
        assert!(MinuteInstant::parse("2026-04-01T09:00:00.500Z").is_err());
        // inicio >= fin
        assert!(AppointmentInterval::new(end, start).is_err());
        // zero and over-1440 durations
        assert!(AppointmentInterval::new(start, start).is_err());
        let too_long_end = MinuteInstant::parse("2026-04-03T13:01:00Z").unwrap();
        assert!(AppointmentInterval::new(start, too_long_end).is_err());
        // exactly 1440 is valid
        let day_end = MinuteInstant::parse("2026-04-02T13:00:00Z").unwrap();
        assert_eq!(
            AppointmentInterval::new(start, day_end)
                .unwrap()
                .duration_minutes(),
            1440
        );
    }

    #[test]
    fn history_actions_and_actors_map_contract_strings() {
        assert_eq!(
            HistoryAction::try_from("created").unwrap().as_str(),
            "created"
        );
        assert_eq!(
            HistoryAction::try_from("no_show").unwrap().as_str(),
            "no_show"
        );
        assert!(HistoryAction::try_from("deleted").is_err());
        assert_eq!(ActorKind::try_from("staff").unwrap().as_str(), "staff");
        assert_eq!(ActorKind::try_from("patient").unwrap().as_str(), "patient");
        assert_eq!(ActorKind::try_from("contact").unwrap().as_str(), "contact");
        assert!(ActorKind::try_from("admin").is_err());
    }

    #[test]
    fn change_reason_rejects_empty_when_present() {
        assert!(ChangeReason::new(None).unwrap().is_none());
        assert!(ChangeReason::new(Some("  ".into())).is_err());
        assert_eq!(
            ChangeReason::new(Some("paciente avisó".into()))
                .unwrap()
                .unwrap()
                .as_str(),
            "paciente avisó"
        );
    }

    #[test]
    fn snapshots_validate_contract_bounds() {
        let valid = Snapshots::new(
            "Consulta general".into(),
            30,
            "Dra. Prueba".into(),
            "Consultorio Central".into(),
            "Av. Principal 1".into(),
            "VE".into(),
            "Distrito Capital".into(),
            "Caracas".into(),
            "America/Caracas".into(),
        )
        .unwrap();
        assert_eq!(valid.offered_duration_minutes, 30);

        assert!(
            Snapshots::new(
                "".into(),
                30,
                "Dra. Prueba".into(),
                "Consultorio Central".into(),
                "Av. Principal 1".into(),
                "VE".into(),
                "Distrito Capital".into(),
                "Caracas".into(),
                "America/Caracas".into(),
            )
            .is_err()
        );
        assert!(
            Snapshots::new(
                "Consulta general".into(),
                30,
                "Dra. Prueba".into(),
                "Consultorio Central".into(),
                "Av. Principal 1".into(),
                "VEN".into(),
                "Distrito Capital".into(),
                "Caracas".into(),
                "America/Caracas".into(),
            )
            .is_err()
        );
        assert!(
            Snapshots::new(
                "Consulta general".into(),
                1441,
                "Dra. Prueba".into(),
                "Consultorio Central".into(),
                "Av. Principal 1".into(),
                "VE".into(),
                "Distrito Capital".into(),
                "Caracas".into(),
                "America/Caracas".into(),
            )
            .is_err()
        );
    }

    #[test]
    fn age_on_uses_local_date_and_february_29_rule() {
        // simple adult
        assert_eq!(age_on(date("1990-06-15"), date("2026-06-15")), 36);
        assert_eq!(age_on(date("1990-06-15"), date("2026-06-14")), 35);
        // boundary minor: turns 18 on the appointment day
        assert!(!is_minor_at(date("2008-03-10"), date("2026-03-10")));
        assert!(is_minor_at(date("2008-03-11"), date("2026-03-10")));
        // Feb 29 birthday falls on Mar 1 in non-leap years
        let feb29 = date("2008-02-29");
        assert!(is_minor_at(feb29, date("2026-02-28")));
        assert!(!is_minor_at(feb29, date("2026-03-01")));
        assert_eq!(age_on(feb29, date("2026-03-01")), 18);
        // leap year keeps the Feb 29 birthday
        assert_eq!(age_on(feb29, date("2028-02-29")), 20);
    }

    #[test]
    fn appointment_id_requires_canonical_uuid_v7() {
        let id: AppointmentId = "019a1234-0000-7000-8000-000000000001"
            .to_owned()
            .try_into()
            .unwrap();
        assert_eq!(id.as_str(), "019a1234-0000-7000-8000-000000000001");
        let bad: std::result::Result<AppointmentId, NightingaleError> =
            "not-a-uuid".to_owned().try_into();
        assert!(bad.is_err());
    }
}
