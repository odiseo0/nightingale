use super::*;
use crate::shared_kernel::id::canonical_uuid_v7;

#[derive(Clone)]
pub(super) struct EditContext {
    pub version: i64,
    pub operation_id: Option<String>,
    pub depends_on: Vec<OperationId>,
}

pub(super) struct PreparedChange {
    pub entity_type: &'static str,
    pub rule_id: Option<String>,
    pub exception_id: Option<String>,
    pub block_id: Option<String>,
    pub key: Value,
    pub draft: Value,
    pub payload: Value,
    pub base: EditContext,
}

impl PreparedChange {
    fn rule(rule_id: String, draft: Value, payload: Value, base: EditContext) -> Self {
        Self {
            entity_type: "schedule_rule",
            rule_id: Some(rule_id.clone()),
            exception_id: None,
            block_id: None,
            key: app::rule_key(&rule_id),
            draft,
            payload,
            base,
        }
    }

    fn exception(exception_id: String, draft: Value, payload: Value, base: EditContext) -> Self {
        Self {
            entity_type: "schedule_exception",
            rule_id: None,
            exception_id: Some(exception_id.clone()),
            block_id: None,
            key: app::rule_key(&exception_id),
            draft,
            payload,
            base,
        }
    }

    fn block(block_id: String, draft: Value, payload: Value, base: EditContext) -> Self {
        Self {
            entity_type: "schedule_block",
            rule_id: None,
            exception_id: None,
            block_id: Some(block_id.clone()),
            key: app::rule_key(&block_id),
            draft,
            payload,
            base,
        }
    }
}

pub(super) fn rule_changes(c: &Connection, s: &OperationContext) -> app::Result<Vec<LocalChange>> {
    changes_of(c, s, "schedule_rule")
}

pub(super) fn exception_changes(
    c: &Connection,
    s: &OperationContext,
) -> app::Result<Vec<LocalChange>> {
    changes_of(c, s, "schedule_exception")
}

pub(super) fn block_changes(c: &Connection, s: &OperationContext) -> app::Result<Vec<LocalChange>> {
    changes_of(c, s, "schedule_block")
}

fn changes_of(
    c: &Connection,
    s: &OperationContext,
    entity_type: &str,
) -> app::Result<Vec<LocalChange>> {
    let mut changes: Vec<_> = read_local_changes(c, s)
        .map_err(storage_error)?
        .into_iter()
        .filter(|v| v.entity_type == entity_type)
        .collect();

    // Each step consumes the prior version; a device clock does not order edits.
    changes.sort_by(|a, b| {
        (a.expected_version, &a.operation_id).cmp(&(b.expected_version, &b.operation_id))
    });

    Ok(changes)
}

// A new intent may depend on an uncertain original. Sync must recover that original ID.
pub(super) fn can_depend(status: &str) -> bool {
    matches!(
        status,
        "pending" | "in_flight" | "unknown" | "retryable" | "accepted"
    )
}

/// Resolves an edit base against the entity row. Accepted bases require a
/// clean accepted row at the same version; local bases consume the provisional
/// and reject edits while any prior operation cannot be depended on.
pub(super) fn edit_base<T>(
    row: &app::EntityRow<T>,
    base: &app::EditBase,
) -> app::Result<EditContext> {
    let stale = || {
        SchedulingError::new(
            "VERSION_CONFLICT",
            "la base local cambió; consulte la agenda",
        )
    };

    match base {
        app::EditBase::Accepted { version } => {
            if *version <= 0
                || row.provisional.is_some()
                || row.accepted.as_ref().is_none_or(|a| a.version != *version)
            {
                return Err(stale());
            }

            Ok(EditContext {
                version: *version,
                operation_id: None,
                depends_on: vec![],
            })
        }

        app::EditBase::LocalOperation { operation_id } => {
            let last = row
                .provisional
                .as_ref()
                .filter(|p| p.operation_id == operation_id.as_str())
                .ok_or_else(stale)?;

            if row.operations.iter().any(|v| !can_depend(&v.status)) {
                return Err(SchedulingError::new(
                    "DEPENDENCY_FAILED",
                    "recupere o revise los resultados previos antes de editar",
                ));
            }

            let version = last.expected_version.checked_add(1).ok_or_else(stale)?;

            Ok(EditContext {
                version,
                operation_id: Some(operation_id.as_str().into()),
                depends_on: vec![operation_id.clone()],
            })
        }
    }
}

pub(super) fn prepare(
    c: &Connection,
    s: &OperationContext,
    access: &crate::components::identity::domain::AccountAccess,
    own_professional: bool,
    mutation: app::SchedulingMutation,
    ids: &impl IdGenerator,
) -> app::Result<PreparedChange> {
    match mutation {
        app::SchedulingMutation::CreateBlock { value } => {
            // Un bloqueo global exige propietario sobre el profesional propio.
            if value.account_id.is_none()
                && (access.role != crate::components::identity::domain::AccessRole::Owner
                    || !own_professional)
            {
                return Err(SchedulingError::new(
                    "PERMISSION_DENIED",
                    "solo el propietario gestiona bloqueos globales del profesional propio",
                ));
            }

            let id = ids.new_id().to_string();
            canonical_uuid_v7(&id)?;
            let state = app::BlockState::from_value(id.clone(), &value, dom::Status::Active);
            let payload = json!({
                "professionalId": state.professional_id,
                "accountId": state.block_account_id,
                "startsAt": state.starts_at,
                "endsAt": state.ends_at,
                "reasonCode": state.reason_code,
            });
            let draft = serde_json::to_value(&state)
                .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?;

            Ok(PreparedChange::block(
                id,
                draft,
                payload,
                EditContext {
                    version: 0,
                    operation_id: None,
                    depends_on: vec![],
                },
            ))
        }

        app::SchedulingMutation::UpdateBlock {
            block_id,
            base,
            patch,
        } => {
            let row = block_rows(c, s)?
                .into_iter()
                .find(|r| r.visible().is_some_and(|v| v.block_id == block_id))
                .ok_or_else(missing)?;
            let context = edit_base(&row, &base)?;
            let original = row.visible().ok_or_else(missing)?;
            let original_value = block_value(original)?;
            let candidate = patch.apply(&original_value)?;
            let state = app::BlockState::from_value(
                block_id.clone(),
                &candidate,
                status_of_block(original),
            );
            let draft = serde_json::to_value(&state)
                .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?;

            let mut payload = serde_json::Map::new();

            if let Some(starts_at) = patch.starts_at {
                payload.insert("startsAt".into(), json!(starts_at.timestamp().to_string()));
            }

            if let Some(ends_at) = patch.ends_at {
                payload.insert("endsAt".into(), json!(ends_at.timestamp().to_string()));
            }

            if let Some(reason) = patch.reason_code {
                payload.insert("reasonCode".into(), json!(reason.as_str()));
            }

            Ok(PreparedChange::block(
                block_id,
                draft,
                Value::Object(payload),
                context,
            ))
        }

        app::SchedulingMutation::SetBlockStatus {
            block_id,
            base,
            status,
        } => {
            let row = block_rows(c, s)?
                .into_iter()
                .find(|r| r.visible().is_some_and(|v| v.block_id == block_id))
                .ok_or_else(missing)?;
            let context = edit_base(&row, &base)?;
            let original = row.visible().ok_or_else(missing)?;

            if original.status == status.as_str() {
                return Err(SchedulingError::no_changes());
            }

            let mut state = original.clone();
            state.status = status.as_str().into();
            let draft = serde_json::to_value(&state)
                .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?;

            Ok(PreparedChange::block(
                block_id,
                draft,
                json!({"status": status.as_str()}),
                context,
            ))
        }

        app::SchedulingMutation::CreateRule { value } => {
            let id = ids.new_id().to_string();
            canonical_uuid_v7(&id)?;
            let state = app::RuleState::from_value(id.clone(), &value, dom::Status::Active);
            let mut payload = json!({
                "professionalId": state.professional_id,
                "isoWeekday": state.iso_weekday,
                "localStartMinute": state.local_start_minute,
                "localEndMinute": state.local_end_minute,
                "validFrom": state.valid_from,
            });

            if let Some(until) = &state.valid_until {
                payload["validUntil"] = json!(until);
            }

            let draft = serde_json::to_value(&state)
                .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?;

            Ok(PreparedChange::rule(
                id,
                draft,
                payload,
                EditContext {
                    version: 0,
                    operation_id: None,
                    depends_on: vec![],
                },
            ))
        }

        app::SchedulingMutation::UpdateRule {
            rule_id,
            base,
            patch,
        } => {
            let row = rule_rows(c, s)?
                .into_iter()
                .find(|r| r.visible().is_some_and(|v| v.rule_id == rule_id))
                .ok_or_else(missing)?;
            let context = edit_base(&row, &base)?;
            let original = row.visible().ok_or_else(missing)?;
            let original_value = rule_value(original)?;
            let candidate = patch.apply(&original_value)?;
            let state =
                app::RuleState::from_value(rule_id.clone(), &candidate, status_of(original));
            let draft = serde_json::to_value(&state)
                .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?;

            let mut payload = serde_json::Map::new();

            if let Some(weekday) = patch.iso_weekday {
                payload.insert("isoWeekday".into(), json!(weekday));
            }

            if let Some(start) = patch.local_start_minute {
                payload.insert("localStartMinute".into(), json!(start));
            }

            if let Some(end) = patch.local_end_minute {
                payload.insert("localEndMinute".into(), json!(end));
            }

            if let Some(from) = patch.valid_from {
                payload.insert("validFrom".into(), json!(from.to_string()));
            }

            if let Some(until) = &patch.valid_until {
                payload.insert(
                    "validUntil".into(),
                    until.map(|d| json!(d.to_string())).unwrap_or(Value::Null),
                );
            }

            Ok(PreparedChange::rule(
                rule_id,
                draft,
                Value::Object(payload),
                context,
            ))
        }

        app::SchedulingMutation::SetRuleStatus {
            rule_id,
            base,
            status,
        } => {
            let row = rule_rows(c, s)?
                .into_iter()
                .find(|r| r.visible().is_some_and(|v| v.rule_id == rule_id))
                .ok_or_else(missing)?;
            let context = edit_base(&row, &base)?;
            let original = row.visible().ok_or_else(missing)?;

            if original.status == status.as_str() {
                return Err(SchedulingError::no_changes());
            }

            let mut state = original.clone();
            state.status = status.as_str().into();
            let draft = serde_json::to_value(&state)
                .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?;

            Ok(PreparedChange::rule(
                rule_id,
                draft,
                json!({"status": status.as_str()}),
                context,
            ))
        }

        app::SchedulingMutation::CreateException { value } => {
            // Una excepción efectiva por profesional y fecha.
            let duplicate = exception_rows(c, s)?
                .into_iter()
                .filter_map(|row| row.visible().cloned())
                .any(|state| {
                    state.professional_id == value.professional_id.as_str()
                        && state.local_date == value.local_date.to_string()
                        && state.status == "active"
                });

            if duplicate {
                return Err(SchedulingError::validation(
                    "ya existe una excepción activa para esa fecha",
                ));
            }

            let id = ids.new_id().to_string();
            canonical_uuid_v7(&id)?;
            let state = app::ExceptionState::from_value(id.clone(), &value, dom::Status::Active);
            let mut payload = json!({
                "professionalId": state.professional_id,
                "localDate": state.local_date,
                "kind": state.kind,
            });

            if let (Some(start), Some(end)) = (state.local_start_minute, state.local_end_minute) {
                payload["localStartMinute"] = json!(start);
                payload["localEndMinute"] = json!(end);
            }

            let draft = serde_json::to_value(&state)
                .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?;

            Ok(PreparedChange::exception(
                id,
                draft,
                payload,
                EditContext {
                    version: 0,
                    operation_id: None,
                    depends_on: vec![],
                },
            ))
        }

        app::SchedulingMutation::UpdateException {
            exception_id,
            base,
            patch,
        } => {
            let row = exception_rows(c, s)?
                .into_iter()
                .find(|r| r.visible().is_some_and(|v| v.exception_id == exception_id))
                .ok_or_else(missing)?;
            let context = edit_base(&row, &base)?;
            let original = row.visible().ok_or_else(missing)?;
            let original_value = exception_value(original)?;
            let candidate = patch.apply(&original_value)?;
            let state = app::ExceptionState::from_value(
                exception_id.clone(),
                &candidate,
                status_of_exception(original),
            );
            let draft = serde_json::to_value(&state)
                .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?;

            let mut payload = serde_json::Map::new();

            if let Some(kind) = patch.kind {
                payload.insert("kind".into(), json!(kind.as_str()));
            }

            if let Some(start) = patch.local_start_minute {
                payload.insert("localStartMinute".into(), json!(start));
            }

            if let Some(end) = patch.local_end_minute {
                payload.insert("localEndMinute".into(), json!(end));
            }

            Ok(PreparedChange::exception(
                exception_id,
                draft,
                Value::Object(payload),
                context,
            ))
        }

        app::SchedulingMutation::SetExceptionStatus {
            exception_id,
            base,
            status,
        } => {
            let row = exception_rows(c, s)?
                .into_iter()
                .find(|r| r.visible().is_some_and(|v| v.exception_id == exception_id))
                .ok_or_else(missing)?;
            let context = edit_base(&row, &base)?;
            let original = row.visible().ok_or_else(missing)?;

            if original.status == status.as_str() {
                return Err(SchedulingError::no_changes());
            }

            let mut state = original.clone();
            state.status = status.as_str().into();
            let draft = serde_json::to_value(&state)
                .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?;

            Ok(PreparedChange::exception(
                exception_id,
                draft,
                json!({"status": status.as_str()}),
                context,
            ))
        }
    }
}

/// A provisional retires only when its operation was accepted and the accepted
/// view reached the receipt's confirmed version. A receipt alone never replaces
/// the download; conflicts, rejections and unknown results keep the provisional.
pub(super) fn confirmed(
    c: &Connection,
    s: &OperationContext,
    accepted_version: Option<i64>,
    local: &LocalChange,
) -> app::Result<bool> {
    use crate::components::sync::domain::OperationStatus;

    let Some(stored) = SqliteOperationStore.operation_result(
        c,
        s,
        &OperationId::try_from(local.operation_id.clone())?,
    )?
    else {
        return Ok(false);
    };

    if stored.review_code.is_some() || stored.status != OperationStatus::Accepted {
        return Ok(false);
    }

    let Some(confirmed) = stored
        .result
        .as_ref()
        .and_then(|v| v.get("entityVersion"))
        .and_then(Value::as_i64)
    else {
        return Ok(false);
    };

    let target = local
        .expected_version
        .checked_add(1)
        .ok_or_else(|| SchedulingError::internal("versión local inválida"))?;

    Ok(confirmed >= target && accepted_version.is_some_and(|v| v >= confirmed))
}

pub(super) fn rule_value(state: &app::RuleState) -> app::Result<dom::RuleValue> {
    dom::RuleValue::new(
        state.professional_id.clone().try_into()?,
        state.iso_weekday,
        state.local_start_minute,
        state.local_end_minute,
        dom::parse_local_date(&state.valid_from)?,
        state
            .valid_until
            .as_deref()
            .map(dom::parse_local_date)
            .transpose()?,
    )
}

pub(super) fn exception_value(state: &app::ExceptionState) -> app::Result<dom::ExceptionValue> {
    dom::ExceptionValue::new(
        state.professional_id.clone().try_into()?,
        dom::parse_local_date(&state.local_date)?,
        dom::ExceptionKind::try_from(state.kind.as_str())?,
        state.local_start_minute,
        state.local_end_minute,
    )
}

pub(super) fn block_value(state: &app::BlockState) -> app::Result<dom::BlockValue> {
    dom::BlockValue::new(
        state.professional_id.clone().try_into()?,
        state
            .block_account_id
            .clone()
            .map(TryInto::try_into)
            .transpose()?,
        dom::Instant::parse(&state.starts_at)?,
        dom::Instant::parse(&state.ends_at)?,
        dom::ReasonCode::try_from(state.reason_code.as_deref().unwrap_or(""))?,
    )
}

fn status_of_block(state: &app::BlockState) -> dom::Status {
    dom::Status::try_from(state.status.as_str()).unwrap_or(dom::Status::Active)
}

fn status_of(state: &app::RuleState) -> dom::Status {
    dom::Status::try_from(state.status.as_str()).unwrap_or(dom::Status::Active)
}

fn status_of_exception(state: &app::ExceptionState) -> dom::Status {
    dom::Status::try_from(state.status.as_str()).unwrap_or(dom::Status::Active)
}
