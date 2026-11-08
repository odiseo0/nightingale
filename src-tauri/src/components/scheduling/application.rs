//! Native scheduling cases. Storage and transport have their own models.
use super::domain::{
    BlockPatch, BlockValue, ExceptionPatch, ExceptionValue, RulePatch, RuleValue, SchedulingError,
    Status,
};
use crate::components::appointments::application::{OccupancyInterval, ScheduleOccupancyQuery};
use crate::components::identity::domain::{AccessStatus, AccountAccess};
use crate::shared_kernel::{Clock, IdGenerator, id::OperationId};
use jiff::tz::TimeZone;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type Result<T> = std::result::Result<T, SchedulingError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleState {
    pub rule_id: String,
    pub professional_id: String,
    pub iso_weekday: i64,
    pub local_start_minute: i64,
    pub local_end_minute: i64,
    pub valid_from: String,
    pub valid_until: Option<String>,
    pub status: String,
}

impl RuleState {
    pub fn from_value(rule_id: String, value: &RuleValue, status: Status) -> Self {
        Self {
            rule_id,
            professional_id: value.professional_id.as_str().into(),
            iso_weekday: i64::from(value.iso_weekday),
            local_start_minute: value.start.get(),
            local_end_minute: value.end.get(),
            valid_from: value.valid_from.to_string(),
            valid_until: value.valid_until.map(|d| d.to_string()),
            status: status.as_str().into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExceptionState {
    pub exception_id: String,
    pub professional_id: String,
    pub local_date: String,
    pub kind: String,
    pub local_start_minute: Option<i64>,
    pub local_end_minute: Option<i64>,
    pub status: String,
}

impl ExceptionState {
    pub fn from_value(
        exception_id: String,
        value: &ExceptionValue,
        status: Status,
    ) -> Self {
        Self {
            exception_id,
            professional_id: value.professional_id.as_str().into(),
            local_date: value.local_date.to_string(),
            kind: value.kind.as_str().into(),
            local_start_minute: value.minutes.map(|m| m.start.get()),
            local_end_minute: value.minutes.map(|m| m.end.get()),
            status: status.as_str().into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Accepted<T> {
    pub state: T,
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Provisional<T> {
    pub state: T,
    pub operation_id: String,
    pub expected_version: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationView {
    pub operation_id: String,
    pub depends_on: Vec<String>,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityRow<T> {
    pub accepted: Option<Accepted<T>>,
    pub provisional: Option<Provisional<T>>,
    pub operations: Vec<OperationView>,
}

impl<T> EntityRow<T> {
    pub fn visible(&self) -> Option<&T> {
        self.provisional
            .as_ref()
            .map(|v| &v.state)
            .or_else(|| self.accepted.as_ref().map(|v| &v.state))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditBase {
    Accepted { version: i64 },
    LocalOperation { operation_id: OperationId },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalMutationResult {
    pub operation_id: String,
    pub entity_type: String,
    pub entity_key: Value,
    pub expected_version: i64,
    pub depends_on: Vec<String>,
    pub provisional: Value,
}

pub enum SchedulingMutation {
    CreateBlock {
        value: BlockValue,
    },
    UpdateBlock {
        block_id: String,
        base: EditBase,
        patch: BlockPatch,
    },
    SetBlockStatus {
        block_id: String,
        base: EditBase,
        status: Status,
    },
    CreateRule {
        value: RuleValue,
    },
    UpdateRule {
        rule_id: String,
        base: EditBase,
        patch: RulePatch,
    },
    SetRuleStatus {
        rule_id: String,
        base: EditBase,
        status: Status,
    },
    CreateException {
        value: ExceptionValue,
    },
    UpdateException {
        exception_id: String,
        base: EditBase,
        patch: ExceptionPatch,
    },
    SetExceptionStatus {
        exception_id: String,
        base: EditBase,
        status: Status,
    },
}

impl SchedulingMutation {
    pub fn action(&self) -> &'static str {
        match self {
            Self::CreateBlock { .. } => "CreateScheduleBlock",
            Self::UpdateBlock { .. } => "UpdateScheduleBlock",
            Self::SetBlockStatus { .. } => "SetScheduleBlockStatus",
            Self::CreateRule { .. } => "CreateScheduleRule",
            Self::UpdateRule { .. } => "UpdateScheduleRule",
            Self::SetRuleStatus { .. } => "SetScheduleRuleStatus",
            Self::CreateException { .. } => "CreateScheduleException",
            Self::UpdateException { .. } => "UpdateScheduleException",
            Self::SetExceptionStatus { .. } => "SetScheduleExceptionStatus",
        }
    }

    pub fn entity_type(&self) -> &'static str {
        match self {
            Self::CreateBlock { .. }
            | Self::UpdateBlock { .. }
            | Self::SetBlockStatus { .. } => "schedule_block",
            Self::CreateRule { .. }
            | Self::UpdateRule { .. }
            | Self::SetRuleStatus { .. } => "schedule_rule",
            Self::CreateException { .. }
            | Self::UpdateException { .. }
            | Self::SetExceptionStatus { .. } => "schedule_exception",
        }
    }
}

pub fn require_read(access: &AccountAccess) -> Result<()> {
    if access.status != AccessStatus::Active {
        return Err(SchedulingError::new(
            "PERMISSION_DENIED",
            "la cuenta no tiene acceso activo",
        ));
    }

    Ok(())
}

pub fn require_write(access: &AccountAccess, action: &str) -> Result<()> {
    if !access.allows_offline(action) {
        return Err(SchedulingError::new(
            "PERMISSION_DENIED",
            "la acción no tiene permiso local",
        ));
    }

    Ok(())
}

/// Write-side port. Implementations must commit the provisional, the local
/// intention and the outbox record in one `IMMEDIATE` transaction, and reject
/// a change that leaves known accepted occupancy outside the resulting schedule.
pub trait SchedulingRepository {
    fn persist(
        &mut self,
        access: &AccountAccess,
        mutation: SchedulingMutation,
        ids: &impl IdGenerator,
        clock: &impl Clock,
        occupancy: &impl ScheduleOccupancyQuery,
    ) -> Result<LocalMutationResult>;

    fn rule_row(&mut self, access: &AccountAccess, rule_id: &str) -> Result<EntityRow<RuleState>>;

    /// Fused accepted + provisional + operation rows for the scope.
    fn rule_rows_list(&mut self, access: &AccountAccess) -> Result<Vec<EntityRow<RuleState>>>;

    fn exception_rows_list(
        &mut self,
        access: &AccountAccess,
    ) -> Result<Vec<EntityRow<ExceptionState>>>;

    fn block_rows_list(&mut self, access: &AccountAccess) -> Result<Vec<EntityRow<BlockState>>>;
}

pub(super) fn rule_key(rule_id: &str) -> Value {
    Value::String(rule_id.into())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockState {
    pub block_id: String,
    pub professional_id: String,
    pub block_account_id: Option<String>,
    pub starts_at: String,
    pub ends_at: String,
    pub reason_code: Option<String>,
    pub status: String,
}

impl BlockState {
    pub fn from_value(block_id: String, value: &BlockValue, status: Status) -> Self {
        Self {
            block_id,
            professional_id: value.professional_id.as_str().into(),
            block_account_id: value.account_id.as_ref().map(|id| id.as_str().into()),
            starts_at: value.starts_at.timestamp().to_string(),
            ends_at: value.ends_at.timestamp().to_string(),
            reason_code: Some(value.reason_code.as_str().into()),
            status: status.as_str().into(),
        }
    }
}

/// Whether one known occupancy interval intersects any active visible block.
/// `blocks` must already be scoped to the interval's professional.
pub fn intersects_active_block(blocks: &[BlockState], interval: &OccupancyInterval) -> bool {
    let starts = interval.starts_at.to_string();
    let ends = interval.ends_at.to_string();
    blocks.iter().any(|block| {
        block.status == "active" && block.starts_at.as_str() < ends.as_str() && block.ends_at.as_str() > starts.as_str()
    })
}

/// Whether the fused visible agenda still covers one known occupancy interval
/// in the account's civil zone. `exceptions` must already be scoped to the
/// interval's professional. An active exception replaces the whole daily
/// schedule for its date: `closed` never covers and `replacement` must contain
/// the appointment minutes. Multi-day appointments are not coverable and fail.
pub fn visible_schedule_covers(
    zone: &TimeZone,
    rules: &[RuleState],
    exceptions: &[ExceptionState],
    interval: &OccupancyInterval,
) -> bool {
    let start = interval.starts_at.to_zoned(zone.clone());
    let end = interval.ends_at.to_zoned(zone.clone());
    let start_date = start.date();
    let end_date = end.date();

    if start_date != end_date {
        return false;
    }

    let minutes = |zoned: &jiff::Zoned| {
        let time = zoned.datetime().time();
        i64::from(time.hour()) * 60 + i64::from(time.minute())
    };
    let start_minute = minutes(&start);
    let end_minute = minutes(&end);
    let date = start_date.to_string();

    if let Some(exception) = exceptions
        .iter()
        .find(|exception| exception.status == "active" && exception.local_date == date)
    {
        return match (exception.local_start_minute, exception.local_end_minute) {
            (Some(start), Some(end)) => start <= start_minute && end >= end_minute,
            _ => false,
        };
    }

    let weekday = i64::from(start_date.weekday().to_monday_one_offset());
    rules.iter().any(|rule| {
        rule.status == "active"
            && rule.iso_weekday == weekday
            && rule.valid_from.as_str() <= date.as_str()
            && rule
                .valid_until
                .as_deref()
                .is_none_or(|until| date.as_str() <= until)
            && rule.local_start_minute <= start_minute
            && rule.local_end_minute >= end_minute
    })
}
