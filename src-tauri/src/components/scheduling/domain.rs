//! Scheduling agenda rules and values, independent of storage and remote transport.
//! Time semantics follow the approved plan: civil dates and minutes from midnight for
//! rules/exceptions, RFC 3339 instants normalized to UTC for blocks, half-open
//! `[start,end)` intervals and strict rejection of ambiguous or nonexistent civil times.
use crate::shared_kernel::{NightingaleError, id::canonical_uuid_v7};
use jiff::{Span, Timestamp, civil::Date, tz::TimeZone};
use std::str::FromStr;

macro_rules! scheduling_id {
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

scheduling_id!(RuleId);
scheduling_id!(ExceptionId);
scheduling_id!(BlockId);
scheduling_id!(ProfessionalId);
scheduling_id!(AccountId);

/// Stable machine-readable boundary codes for the scheduling component.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulingError {
    pub code: String,
    pub message: String,
}

pub type Result<T> = std::result::Result<T, SchedulingError>;

impl SchedulingError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::new("VALIDATION_ERROR", message)
    }

    pub fn no_changes() -> Self {
        Self::new("NO_CHANGES", "el cambio no modifica el estado actual")
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("INTERNAL_ERROR", message)
    }
}

impl From<NightingaleError> for SchedulingError {
    fn from(error: NightingaleError) -> Self {
        let code = serde_json::to_value(error.code)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_else(|| "INTERNAL_ERROR".into());

        if code == "INTERNAL_ERROR" {
            Self::internal("datos locales de agenda no disponibles")
        } else {
            Self::new(&code, error.message)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Active,
    Inactive,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
        }
    }
}

impl TryFrom<&str> for Status {
    type Error = SchedulingError;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            _ => Err(SchedulingError::validation(
                "estado de agenda inválido; se requiere active o inactive",
            )),
        }
    }
}

/// Minutes from local midnight at the start of a half-open interval: 0..=1439.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalStartMinute(i64);

impl LocalStartMinute {
    pub fn new(value: i64) -> Result<Self> {
        if !(0..=1439).contains(&value) {
            return Err(SchedulingError::validation(
                "el minuto de inicio debe ser un entero entre 0 y 1439",
            ));
        }
        Ok(Self(value))
    }

    pub fn get(self) -> i64 {
        self.0
    }
}

/// Minutes from local midnight at the exclusive end: 1..=1440. `1440` is the
/// local midnight of the following civil day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalEndMinute(i64);

impl LocalEndMinute {
    pub fn new(value: i64) -> Result<Self> {
        if !(1..=1440).contains(&value) {
            return Err(SchedulingError::validation(
                "el minuto de fin debe ser un entero entre 1 y 1440",
            ));
        }

        Ok(Self(value))
    }

    pub fn get(self) -> i64 {
        self.0
    }
}

fn ordered(start: LocalStartMinute, end: LocalEndMinute) -> Result<()> {
    if start.get() >= end.get() {
        return Err(SchedulingError::validation(
            "el minuto de inicio debe ser menor que el minuto de fin",
        ));
    }

    Ok(())
}

pub fn parse_local_date(value: &str) -> Result<Date> {
    Date::from_str(value)
        .map_err(|_| SchedulingError::validation("la fecha civil debe ser ISO 8601 YYYY-MM-DD"))
}

/// An instant normalized to UTC. The original RFC 3339 offset is discarded so
/// internal comparisons never mix civil and UTC values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(Timestamp);

impl Instant {
    pub fn parse(value: &str) -> Result<Self> {
        let ts = Timestamp::from_str(value).map_err(|_| {
            SchedulingError::validation("el instante debe ser RFC 3339 con offset o Z")
        })?;

        Ok(Self(ts))
    }

    pub fn timestamp(self) -> Timestamp {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasonCode {
    Vacation,
    Administrative,
    Unavailable,
}

impl ReasonCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vacation => "vacation",
            Self::Administrative => "administrative",
            Self::Unavailable => "unavailable",
        }
    }
}

impl TryFrom<&str> for ReasonCode {
    type Error = SchedulingError;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "vacation" => Ok(Self::Vacation),
            "administrative" => Ok(Self::Administrative),
            "unavailable" => Ok(Self::Unavailable),
            _ => Err(SchedulingError::validation(
                "reasonCode debe ser vacation, administrative o unavailable",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleValue {
    pub professional_id: ProfessionalId,
    pub iso_weekday: u8,
    pub start: LocalStartMinute,
    pub end: LocalEndMinute,
    pub valid_from: Date,
    /// Inclusive validity end; `None` means open-ended.
    pub valid_until: Option<Date>,
}

impl RuleValue {
    pub fn new(
        professional_id: ProfessionalId,
        iso_weekday: i64,
        local_start_minute: i64,
        local_end_minute: i64,
        valid_from: Date,
        valid_until: Option<Date>,
    ) -> Result<Self> {
        if !(1..=7).contains(&iso_weekday) {
            return Err(SchedulingError::validation(
                "isoWeekday debe ser un entero entre 1 y 7",
            ));
        }

        if let Some(until) = valid_until
            && until < valid_from
        {
            return Err(SchedulingError::validation(
                "validUntil debe ser mayor o igual que validFrom",
            ));
        }

        let start = LocalStartMinute::new(local_start_minute)?;
        let end = LocalEndMinute::new(local_end_minute)?;
        ordered(start, end)?;

        Ok(Self {
            professional_id,
            iso_weekday: iso_weekday as u8,
            start,
            end,
            valid_from,
            valid_until,
        })
    }
}

/// Tri-state rule patch: absent fields conserve the base value; `valid_until`
/// admits `Some(None)` to clear the open end. A patch with no field set is invalid.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RulePatch {
    pub iso_weekday: Option<i64>,
    pub local_start_minute: Option<i64>,
    pub local_end_minute: Option<i64>,
    pub valid_from: Option<Date>,
    pub valid_until: Option<Option<Date>>,
}

impl RulePatch {
    pub fn new(
        iso_weekday: Option<i64>,
        local_start_minute: Option<i64>,
        local_end_minute: Option<i64>,
        valid_from: Option<Date>,
        valid_until: Option<Option<Date>>,
    ) -> Result<Self> {
        if iso_weekday.is_none()
            && local_start_minute.is_none()
            && local_end_minute.is_none()
            && valid_from.is_none()
            && valid_until.is_none()
        {
            return Err(SchedulingError::validation("el cambio de regla está vacío"));
        }

        Ok(Self {
            iso_weekday,
            local_start_minute,
            local_end_minute,
            valid_from,
            valid_until,
        })
    }

    pub fn apply(&self, base: &RuleValue) -> Result<RuleValue> {
        let candidate = RuleValue::new(
            base.professional_id.clone(),
            self.iso_weekday.unwrap_or(i64::from(base.iso_weekday)),
            self.local_start_minute.unwrap_or(base.start.get()),
            self.local_end_minute.unwrap_or(base.end.get()),
            self.valid_from.unwrap_or(base.valid_from),
            match &self.valid_until {
                None => base.valid_until,
                Some(value) => *value,
            },
        )?;

        if candidate == *base {
            return Err(SchedulingError::no_changes());
        }

        Ok(candidate)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionKind {
    Closed,
    Replacement,
}

impl ExceptionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::Replacement => "replacement",
        }
    }
}

impl TryFrom<&str> for ExceptionKind {
    type Error = SchedulingError;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "closed" => Ok(Self::Closed),
            "replacement" => Ok(Self::Replacement),
            _ => Err(SchedulingError::validation(
                "kind debe ser closed o replacement",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExceptionMinutes {
    pub start: LocalStartMinute,
    pub end: LocalEndMinute,
}

/// A single effective exception per professional and civil date. `Closed` leaves
/// the day empty and requires null minutes; `Replacement` requires both minutes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExceptionValue {
    pub professional_id: ProfessionalId,
    pub local_date: Date,
    pub kind: ExceptionKind,
    pub minutes: Option<ExceptionMinutes>,
}

impl ExceptionValue {
    pub fn new(
        professional_id: ProfessionalId,
        local_date: Date,
        kind: ExceptionKind,
        local_start_minute: Option<i64>,
        local_end_minute: Option<i64>,
    ) -> Result<Self> {
        let minutes = match kind {
            ExceptionKind::Closed => {
                if local_start_minute.is_some() || local_end_minute.is_some() {
                    return Err(SchedulingError::validation("closed exige minutos nulos"));
                }
                None
            }
            ExceptionKind::Replacement => {
                let start = local_start_minute.ok_or_else(|| {
                    SchedulingError::validation("replacement exige minuto de inicio")
                })?;
                let end = local_end_minute.ok_or_else(|| {
                    SchedulingError::validation("replacement exige minuto de fin")
                })?;
                let start = LocalStartMinute::new(start)?;
                let end = LocalEndMinute::new(end)?;
                ordered(start, end)?;
                Some(ExceptionMinutes { start, end })
            }
        };

        Ok(Self {
            professional_id,
            local_date,
            kind,
            minutes,
        })
    }
}

/// Tri-state exception patch revalidating the resulting state; it never
/// withdraws minutes that the resulting kind requires.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExceptionPatch {
    pub kind: Option<ExceptionKind>,
    pub local_start_minute: Option<i64>,
    pub local_end_minute: Option<i64>,
}

impl ExceptionPatch {
    pub fn new(
        kind: Option<ExceptionKind>,
        local_start_minute: Option<i64>,
        local_end_minute: Option<i64>,
    ) -> Result<Self> {
        if kind.is_none() && local_start_minute.is_none() && local_end_minute.is_none() {
            return Err(SchedulingError::validation(
                "el cambio de excepción está vacío",
            ));
        }

        Ok(Self {
            kind,
            local_start_minute,
            local_end_minute,
        })
    }

    pub fn apply(&self, base: &ExceptionValue) -> Result<ExceptionValue> {
        let candidate = ExceptionValue::new(
            base.professional_id.clone(),
            base.local_date,
            self.kind.unwrap_or(base.kind),
            self.local_start_minute
                .or_else(|| base.minutes.map(|m| m.start.get())),
            self.local_end_minute
                .or_else(|| base.minutes.map(|m| m.end.get())),
        )?;

        if candidate == *base {
            return Err(SchedulingError::no_changes());
        }

        Ok(candidate)
    }
}

/// An absolute-time block. `account_id = None` marks a global block that covers
/// the professional's activity across accounts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockValue {
    pub professional_id: ProfessionalId,
    pub account_id: Option<AccountId>,
    pub starts_at: Instant,
    pub ends_at: Instant,
    pub reason_code: ReasonCode,
}

impl BlockValue {
    pub fn new(
        professional_id: ProfessionalId,
        account_id: Option<AccountId>,
        starts_at: Instant,
        ends_at: Instant,
        reason_code: ReasonCode,
    ) -> Result<Self> {
        if starts_at >= ends_at {
            return Err(SchedulingError::validation(
                "el instante de inicio debe ser menor que el de fin",
            ));
        }

        Ok(Self {
            professional_id,
            account_id,
            starts_at,
            ends_at,
            reason_code,
        })
    }
}

/// Block patch limited to times and reason; scope and professional never change.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockPatch {
    pub starts_at: Option<Instant>,
    pub ends_at: Option<Instant>,
    pub reason_code: Option<ReasonCode>,
}

impl BlockPatch {
    pub fn new(
        starts_at: Option<Instant>,
        ends_at: Option<Instant>,
        reason_code: Option<ReasonCode>,
    ) -> Result<Self> {
        if starts_at.is_none() && ends_at.is_none() && reason_code.is_none() {
            return Err(SchedulingError::validation(
                "el cambio de bloqueo está vacío",
            ));
        }

        Ok(Self {
            starts_at,
            ends_at,
            reason_code,
        })
    }

    pub fn apply(&self, base: &BlockValue) -> Result<BlockValue> {
        let candidate = BlockValue::new(
            base.professional_id.clone(),
            base.account_id.clone(),
            self.starts_at.unwrap_or(base.starts_at),
            self.ends_at.unwrap_or(base.ends_at),
            self.reason_code.unwrap_or(base.reason_code),
        )?;

        if candidate == *base {
            return Err(SchedulingError::no_changes());
        }

        Ok(candidate)
    }
}

/// Versioned base for an edit: creation expects version 0; edits use either a
/// positive accepted version or the last local operation as base.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditBase {
    pub expected_version: i64,
    pub base_operation_id: Option<String>,
}

impl EditBase {
    pub fn create() -> Self {
        Self {
            expected_version: 0,
            base_operation_id: None,
        }
    }

    pub fn edit_accepted(expected_version: i64) -> Result<Self> {
        if expected_version <= 0 {
            return Err(SchedulingError::validation(
                "la base aceptada exige una versión positiva",
            ));
        }

        Ok(Self {
            expected_version,
            base_operation_id: None,
        })
    }

    pub fn edit_local(base_operation_id: String) -> Result<Self> {
        canonical_uuid_v7(&base_operation_id)?;

        Ok(Self {
            expected_version: 0,
            base_operation_id: Some(base_operation_id),
        })
    }
}

/// Converts a civil half-open interval to UTC with strict disambiguation:
/// ambiguous (fold) or nonexistent (gap) local times are rejected instead of
/// silently resolved. A `1440` end minute lands on the following civil day.
// Strict civil-time resolution: TimeZone::to_ambiguous_timestamp + unambiguous
// rejects fold/gap instead of Disambiguation::Compatible's silent choice.
// Source: https://docs.rs/jiff/0.2.38/jiff/tz/enum.Disambiguation.html
pub fn civil_interval_to_utc(
    zone: &TimeZone,
    date: Date,
    start: LocalStartMinute,
    end: LocalEndMinute,
) -> Result<(Timestamp, Timestamp)> {
    let midnight = date
        .at(0, 0, 0, 0)
        .checked_add(Span::new().minutes(start.get()))
        .map_err(|_| SchedulingError::internal("fecha civil fuera de rango"))?;
    let end_dt = date
        .at(0, 0, 0, 0)
        .checked_add(Span::new().minutes(end.get()))
        .map_err(|_| SchedulingError::internal("fecha civil fuera de rango"))?;

    let start_ts = zone
        .to_ambiguous_timestamp(midnight)
        .unambiguous()
        .map_err(|_| {
            SchedulingError::validation("la hora civil es ambigua o inexistente en la zona horaria")
        })?;
    let end_ts = zone
        .to_ambiguous_timestamp(end_dt)
        .unambiguous()
        .map_err(|_| {
            SchedulingError::validation("la hora civil es ambigua o inexistente en la zona horaria")
        })?;

    if start_ts >= end_ts {
        return Err(SchedulingError::validation(
            "el intervalo civil resulta vacío en UTC",
        ));
    }

    Ok((start_ts, end_ts))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn professional() -> ProfessionalId {
        "019a1234-0000-7000-8000-000000000001"
            .to_owned()
            .try_into()
            .unwrap()
    }

    fn date(value: &str) -> Date {
        parse_local_date(value).unwrap()
    }

    #[test]
    fn rule_values_validate_contract_bounds() {
        let p = professional();
        let valid = RuleValue::new(p.clone(), 1, 0, 1440, date("2026-01-01"), None).unwrap();
        assert_eq!(valid.iso_weekday, 1);
        assert_eq!(valid.end.get(), 1440);

        // isoWeekday fuera de 1..=7
        for weekday in [0, 8, -1] {
            assert!(RuleValue::new(p.clone(), weekday, 0, 60, date("2026-01-01"), None).is_err());
        }
        // minutos fuera de rango
        assert!(RuleValue::new(p.clone(), 1, -1, 60, date("2026-01-01"), None).is_err());
        assert!(RuleValue::new(p.clone(), 1, 0, 0, date("2026-01-01"), None).is_err());
        assert!(RuleValue::new(p.clone(), 1, 0, 1441, date("2026-01-01"), None).is_err());
        // inicio >= fin
        assert!(RuleValue::new(p.clone(), 1, 60, 60, date("2026-01-01"), None).is_err());
        assert!(RuleValue::new(p.clone(), 1, 120, 60, date("2026-01-01"), None).is_err());
        // vigencia inclusiva: igual es válido, anterior no
        assert!(
            RuleValue::new(
                p.clone(),
                1,
                0,
                60,
                date("2026-01-01"),
                Some(date("2026-01-01"))
            )
            .is_ok()
        );
        assert!(
            RuleValue::new(
                p.clone(),
                1,
                0,
                60,
                date("2026-02-01"),
                Some(date("2026-01-31"))
            )
            .is_err()
        );
    }

    #[test]
    fn rule_patch_tri_state_and_no_changes() {
        let p = professional();
        let base = RuleValue::new(
            p,
            2,
            540,
            1080,
            date("2026-01-01"),
            Some(date("2026-12-31")),
        )
        .unwrap();

        // vacío se rechaza
        assert!(RulePatch::new(None, None, None, None, None).is_err());
        // ausencia conserva, validUntil null retira
        let patch = RulePatch::new(None, None, None, None, Some(None)).unwrap();
        let changed = patch.apply(&base).unwrap();
        assert_eq!(changed.iso_weekday, 2);
        assert_eq!(changed.valid_until, None);
        // valor concreto
        let patch = RulePatch::new(
            Some(3),
            Some(600),
            None,
            None,
            Some(Some(date("2027-06-30"))),
        )
        .unwrap();
        let changed = patch.apply(&base).unwrap();
        assert_eq!(changed.iso_weekday, 3);
        assert_eq!(changed.start.get(), 600);
        assert_eq!(changed.end.get(), 1080);
        assert_eq!(changed.valid_until, Some(date("2027-06-30")));
        // sin diferencia produce NO_CHANGES
        let same = RulePatch::new(
            Some(2),
            Some(540),
            Some(1080),
            Some(date("2026-01-01")),
            Some(Some(date("2026-12-31"))),
        )
        .unwrap();
        let err = same.apply(&base).unwrap_err();
        assert_eq!(err.code, "NO_CHANGES");
        // patch inválido sobre el estado resultante
        let bad = RulePatch::new(None, Some(1200), Some(600), None, None).unwrap();
        assert!(bad.apply(&base).is_err());
    }

    #[test]
    fn exceptions_enforce_kind_minutes_and_resulting_state() {
        let p = professional();
        // closed exige minutos nulos
        assert!(
            ExceptionValue::new(
                p.clone(),
                date("2026-03-10"),
                ExceptionKind::Closed,
                Some(0),
                Some(60)
            )
            .is_err()
        );
        let closed = ExceptionValue::new(
            p.clone(),
            date("2026-03-10"),
            ExceptionKind::Closed,
            None,
            None,
        )
        .unwrap();
        assert_eq!(closed.minutes, None);
        // replacement exige ambos e inicio<fin
        assert!(
            ExceptionValue::new(
                p.clone(),
                date("2026-03-10"),
                ExceptionKind::Replacement,
                None,
                Some(60)
            )
            .is_err()
        );
        assert!(
            ExceptionValue::new(
                p.clone(),
                date("2026-03-10"),
                ExceptionKind::Replacement,
                Some(60),
                Some(60)
            )
            .is_err()
        );
        let replacement = ExceptionValue::new(
            p.clone(),
            date("2026-03-10"),
            ExceptionKind::Replacement,
            Some(0),
            Some(1440),
        )
        .unwrap();
        assert_eq!(replacement.minutes.unwrap().end.get(), 1440);

        // patch: cambiar a closed sin retirar minutos rechaza el estado resultante
        let to_closed = ExceptionPatch::new(Some(ExceptionKind::Closed), None, None).unwrap();
        assert!(to_closed.apply(&replacement).is_err());
        // cambiar a replacement con minutos válidos funciona y ausencia conserva
        let to_replacement =
            ExceptionPatch::new(Some(ExceptionKind::Replacement), Some(30), Some(90)).unwrap();
        let changed = to_replacement.apply(&closed).unwrap();
        assert_eq!(changed.kind, ExceptionKind::Replacement);
        assert_eq!(changed.minutes.unwrap().start.get(), 30);
        // sin diferencia
        let same = ExceptionPatch::new(Some(ExceptionKind::Closed), None, None).unwrap();
        assert_eq!(same.apply(&closed).unwrap_err().code, "NO_CHANGES");
        assert!(ExceptionPatch::new(None, None, None).is_err());
    }

    #[test]
    fn blocks_validate_instants_scope_and_patch() {
        let p = professional();
        let starts = Instant::parse("2026-04-01T09:00:00-04:00").unwrap();
        let ends = Instant::parse("2026-04-01T14:00:00Z").unwrap();
        // offsets distintos normalizados a UTC: 10:00 -04:00 == 14:00Z
        let block = BlockValue::new(
            p.clone(),
            Some(
                "019a1234-0000-7000-8000-000000000099"
                    .to_owned()
                    .try_into()
                    .unwrap(),
            ),
            starts,
            ends,
            ReasonCode::Vacation,
        )
        .unwrap();
        assert_eq!(
            block.starts_at.timestamp().to_string(),
            "2026-04-01T13:00:00Z"
        );

        // inicio >= fin
        assert!(
            BlockValue::new(p.clone(), None, ends, starts, ReasonCode::Administrative).is_err()
        );
        // account_id no canónico
        let bad_account: std::result::Result<AccountId, NightingaleError> =
            "not-a-uuid".to_owned().try_into();
        assert!(bad_account.is_err());

        // patch solo tiempos y motivo
        let patch = BlockPatch::new(
            None,
            Some(Instant::parse("2026-04-01T18:00:00Z").unwrap()),
            Some(ReasonCode::Unavailable),
        )
        .unwrap();
        let changed = patch.apply(&block).unwrap();
        assert_eq!(changed.reason_code, ReasonCode::Unavailable);
        assert_eq!(changed.account_id, block.account_id);
        // sin diferencia
        let same = BlockPatch::new(Some(starts), Some(ends), Some(ReasonCode::Vacation)).unwrap();
        assert_eq!(same.apply(&block).unwrap_err().code, "NO_CHANGES");
        assert!(BlockPatch::new(None, None, None).is_err());
    }

    #[test]
    fn edit_bases_distinguish_create_accepted_and_local() {
        assert_eq!(EditBase::create().expected_version, 0);
        assert!(EditBase::edit_accepted(1).is_ok());
        assert!(EditBase::edit_accepted(0).is_err());
        assert!(EditBase::edit_accepted(-2).is_err());
        let local = EditBase::edit_local("019a1234-0000-7000-8000-000000000042".into()).unwrap();
        assert_eq!(
            local.base_operation_id.as_deref(),
            Some("019a1234-0000-7000-8000-000000000042")
        );
        assert!(EditBase::edit_local("bad".into()).is_err());
    }

    #[test]
    fn civil_to_utc_rejects_gap_and_fold_and_handles_1440() {
        let new_york = TimeZone::get("America/New_York").unwrap();
        // gap 2026-03-08 02:00-03:00 (inicio en el gap)
        let gap = civil_interval_to_utc(
            &new_york,
            date("2026-03-08"),
            LocalStartMinute::new(150).unwrap(),
            LocalEndMinute::new(180).unwrap(),
        );
        assert!(gap.is_err());
        // fold 2026-11-01 01:00-02:00 (hora repetida)
        let fold = civil_interval_to_utc(
            &new_york,
            date("2026-11-01"),
            LocalStartMinute::new(60).unwrap(),
            LocalEndMinute::new(90).unwrap(),
        );
        assert!(fold.is_err());
        // intervalo sano y 1440 = medianoche del día siguiente
        let (start, end) = civil_interval_to_utc(
            &new_york,
            date("2026-03-08"),
            LocalStartMinute::new(540).unwrap(),
            LocalEndMinute::new(1440).unwrap(),
        )
        .unwrap();
        assert_eq!(start.to_string(), "2026-03-08T13:00:00Z");
        assert_eq!(end.to_string(), "2026-03-09T04:00:00Z");
    }

    #[test]
    fn status_and_reason_codes_map_contract_strings() {
        assert_eq!(Status::try_from("active").unwrap(), Status::Active);
        assert_eq!(Status::try_from("inactive").unwrap(), Status::Inactive);
        assert!(Status::try_from("paused").is_err());
        assert_eq!(
            ReasonCode::try_from("vacation").unwrap().as_str(),
            "vacation"
        );
        assert_eq!(
            ReasonCode::try_from("administrative").unwrap().as_str(),
            "administrative"
        );
        assert_eq!(
            ReasonCode::try_from("unavailable").unwrap().as_str(),
            "unavailable"
        );
        assert!(ReasonCode::try_from("other").is_err());
        assert_eq!(
            ExceptionKind::try_from("closed").unwrap(),
            ExceptionKind::Closed
        );
        assert_eq!(
            ExceptionKind::try_from("replacement").unwrap(),
            ExceptionKind::Replacement
        );
        assert!(ExceptionKind::try_from("open").is_err());
    }
}
