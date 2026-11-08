//! Intent preparation for appointment writes: edit bases, contract payloads
//! and dependency assembly. Business validation from the booking ports and
//! the clocks layers on top per task without changing this plumbing.
use super::super::{
    application as app,
    domain::{self as dom, AppointmentTransition},
};
use crate::shared_kernel::id::{OperationId, canonical_uuid_v7};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::str::FromStr;

#[derive(Clone)]
pub(super) struct EditContext {
    pub version: i64,
    pub operation_id: Option<String>,
    pub depends_on: Vec<OperationId>,
}

pub(super) struct PreparedChange {
    pub action: &'static str,
    pub appointment_id: String,
    pub key: Value,
    pub draft: Value,
    pub payload: Value,
    pub base: EditContext,
}

impl PreparedChange {
    fn new(
        action: &'static str,
        appointment_id: String,
        draft: Value,
        payload: Value,
        base: EditContext,
    ) -> Self {
        Self {
            action,
            key: json!(appointment_id),
            draft,
            payload,
            base,
            appointment_id,
        }
    }
}

pub(super) fn appointment_changes(
    c: &Connection,
    s: &crate::shared_kernel::id::OperationContext,
) -> app::AppResult<Vec<super::outbound::LocalChange>> {
    let mut changes = super::outbound::read_local_changes(c, s).map_err(storage_error)?;

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

/// Resolves an edit base against the merged appointment row. Accepted bases
/// require a clean accepted row at the same version; local bases consume the
/// provisional and reject edits while any prior operation cannot be depended on.
pub(super) fn edit_base(
    row: &app::EntityRow<app::AppointmentState>,
    base: &app::EditBase,
) -> app::AppResult<EditContext> {
    use dom::AppointmentsError;

    let stale =
        || AppointmentsError::new("VERSION_CONFLICT", "la base local cambió; consulte la cita");

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
                return Err(AppointmentsError::new(
                    "DEPENDENCY_FAILED",
                    "recupere o revise los resultados previos antes de editar",
                ));
            }

            let version = last.expected_version.checked_add(1).ok_or_else(stale)?;

            Ok(EditContext {
                version,
                operation_id: Some(operation_id.clone()),
                depends_on: vec![OperationId::try_from(operation_id.clone()).map_err(|_| stale())?],
            })
        }
    }
}

use dom::AppointmentsError;

fn storage_error(_: rusqlite::Error) -> AppointmentsError {
    AppointmentsError::internal("datos locales de citas no disponibles")
}

fn missing() -> AppointmentsError {
    AppointmentsError::new("NOT_FOUND", "cita no disponible en este ámbito")
}

fn no_changes() -> AppointmentsError {
    AppointmentsError::new("NO_CHANGES", "el cambio no modifica el estado actual")
}

/// Validates declared dependencies: canonical operation ids that exist, can
/// still be depended on, carry no duplicates and fit the eight-slot limit.
/// A new operation cannot reference itself, so the dependency graph stays acyclic.
fn validate_depends_on(
    c: &Connection,
    s: &crate::shared_kernel::id::OperationContext,
    deps: &[String],
) -> app::AppResult<Vec<OperationId>> {
    let store = crate::components::sync::adapters::outbound::SqliteOperationStore;
    let mut out: Vec<OperationId> = Vec::with_capacity(deps.len());

    for dep in deps {
        let id = OperationId::try_from(dep.clone()).map_err(|_| {
            AppointmentsError::validation("dependencia debe ser un UUIDv7 canónico")
        })?;

        if out.contains(&id) {
            return Err(AppointmentsError::validation(
                "las dependencias no admiten duplicados",
            ));
        }

        let stored = store
            .operation_result(c, s, &id)
            .map_err(|_| AppointmentsError::internal("resultado de operación no disponible"))?
            .ok_or_else(|| {
                AppointmentsError::validation("la operación dependencia no existe en este ámbito")
            })?;

        if !can_depend(stored.status.as_str()) {
            return Err(AppointmentsError::new(
                "DEPENDENCY_FAILED",
                "una dependencia fue rechazada o quedó en conflicto",
            ));
        }

        out.push(id);
    }

    if out.len() > 8 {
        return Err(AppointmentsError::validation(
            "se admiten hasta ocho dependencias por operación",
        ));
    }

    Ok(out)
}

fn draft_of(
    state: &app::AppointmentState,
    status: dom::AppointmentStatus,
    confirmed_at: Option<String>,
) -> app::AppResult<Value> {
    let mut state = state.clone();
    state.status = status.as_str().into();
    state.confirmed_at = confirmed_at;
    serde_json::to_value(&state)
        .map_err(|_| AppointmentsError::internal("borrador de cita inválido"))
}

fn reason_payload(reason: &Option<dom::ChangeReason>) -> Value {
    let mut payload = serde_json::Map::new();

    if let Some(reason) = reason {
        payload.insert("changeReason".into(), json!(reason.as_str()));
    }

    Value::Object(payload)
}

pub(super) fn prepare(
    c: &Connection,
    s: &crate::shared_kernel::id::OperationContext,
    mutation: app::AppointmentMutation,
    ids: &impl crate::shared_kernel::IdGenerator,
) -> app::AppResult<PreparedChange> {
    match mutation {
        app::AppointmentMutation::Create { data, depends_on } => {
            let data = *data;
            let id = ids.new_id().to_string();
            canonical_uuid_v7(&id)?;
            let row = super::outbound::appointment_rows(c, s)?
                .into_iter()
                .find(|row| row.visible().is_some_and(|v| v.appointment_id == id));

            if row.is_some() {
                return Err(AppointmentsError::invalid_state(
                    "la cita ya existe; use una acción de ciclo de vida",
                ));
            }

            let dependencies = validate_depends_on(c, s, &depends_on)?;
            let status = dom::apply_transition(AppointmentTransition::Create, None)?;
            let state = app::AppointmentState {
                appointment_id: id.clone(),
                patient_id: data.patient_id,
                professional_id: data.professional_id,
                service_id: data.service_id,
                responsible_contact_id: data.responsible_contact_id,
                starts_at: data.interval.starts_at.timestamp().to_string(),
                ends_at: data.interval.ends_at.timestamp().to_string(),
                status: status.as_str().into(),
                confirmed_at: None,
                service_name: data.snapshots.service_name,
                offered_duration_minutes: data.snapshots.offered_duration_minutes,
                professional_name: data.snapshots.professional_name,
                location_name: data.snapshots.location_name,
                location_address: data.snapshots.location_address,
                location_country: data.snapshots.location_country,
                location_state: data.snapshots.location_state,
                location_city: data.snapshots.location_city,
                time_zone: data.snapshots.time_zone,
            };
            let mut payload = json!({
                "patientId": state.patient_id,
                "professionalId": state.professional_id,
                "serviceId": state.service_id,
                "startsAt": state.starts_at,
                "endsAt": state.ends_at,
            });

            if let Some(contact) = &state.responsible_contact_id {
                payload["responsibleContactId"] = json!(contact);
            }

            let draft = serde_json::to_value(&state)
                .map_err(|_| AppointmentsError::internal("borrador de cita inválido"))?;

            Ok(PreparedChange::new(
                "CreateAppointment",
                id,
                draft,
                payload,
                EditContext {
                    version: 0,
                    operation_id: None,
                    depends_on: dependencies,
                },
            ))
        }

        app::AppointmentMutation::Confirm { appointment, base } => {
            let row = super::outbound::appointment_rows(c, s)?
                .into_iter()
                .find(|row| {
                    row.visible()
                        .is_some_and(|v| v.appointment_id == appointment.as_str())
                })
                .ok_or_else(missing)?;
            let context = edit_base(&row, &base)?;
            let current = row.visible().ok_or_else(missing)?;
            let next =
                dom::apply_transition(AppointmentTransition::Confirm, Some(current.status()?))?;
            let draft = draft_of(current, next, current.confirmed_at.clone())?;

            // EmptyAppointmentPayload: confirmation time comes from the server.
            Ok(PreparedChange::new(
                "ConfirmAppointment",
                appointment.as_str().into(),
                draft,
                json!({}),
                context,
            ))
        }

        app::AppointmentMutation::Reschedule {
            appointment,
            base,
            interval,
        } => {
            let row = super::outbound::appointment_rows(c, s)?
                .into_iter()
                .find(|row| {
                    row.visible()
                        .is_some_and(|v| v.appointment_id == appointment.as_str())
                })
                .ok_or_else(missing)?;
            let context = edit_base(&row, &base)?;
            let current = row.visible().ok_or_else(missing)?;
            let next =
                dom::apply_transition(AppointmentTransition::Reschedule, Some(current.status()?))?;

            let new_start = interval.starts_at.timestamp().to_string();
            let new_end = interval.ends_at.timestamp().to_string();

            if current.starts_at == new_start && current.ends_at == new_end {
                return Err(no_changes());
            }

            // Rescheduling keeps the real duration; only the reference differs.
            let current_duration = jiff::Timestamp::from_str(&current.ends_at)
                .and_then(|ends| {
                    jiff::Timestamp::from_str(&current.starts_at).map(|starts| ends - starts)
                })
                .map_err(|_| AppointmentsError::internal("intervalo local inválido"))?
                .total(jiff::Unit::Minute)
                .map_err(|_| AppointmentsError::internal("duración local inválida"))?;

            if (current_duration - interval.duration_minutes() as f64).abs() > f64::EPSILON {
                return Err(AppointmentsError::validation(
                    "reprogramar conserva la duración real de la cita",
                ));
            }

            let mut state = current.clone();
            state.status = next.as_str().into();
            state.starts_at = new_start;
            state.ends_at = new_end;
            // Rescheduling invalidates the confirmation until acceptance.
            state.confirmed_at = None;
            let draft = serde_json::to_value(&state)
                .map_err(|_| AppointmentsError::internal("borrador de cita inválido"))?;
            let payload = json!({
                "startsAt": state.starts_at,
                "endsAt": state.ends_at,
            });

            Ok(PreparedChange::new(
                "RescheduleAppointment",
                appointment.as_str().into(),
                draft,
                payload,
                context,
            ))
        }

        app::AppointmentMutation::Cancel {
            appointment,
            base,
            reason,
        } => lifecycle(
            c,
            s,
            appointment,
            base,
            reason,
            AppointmentTransition::Cancel,
        ),
        app::AppointmentMutation::Complete {
            appointment,
            base,
            reason,
        } => lifecycle(
            c,
            s,
            appointment,
            base,
            reason,
            AppointmentTransition::Complete,
        ),
        app::AppointmentMutation::NoShow {
            appointment,
            base,
            reason,
        } => lifecycle(
            c,
            s,
            appointment,
            base,
            reason,
            AppointmentTransition::MarkNoShow,
        ),
    }
}

fn lifecycle(
    c: &Connection,
    s: &crate::shared_kernel::id::OperationContext,
    appointment: dom::AppointmentId,
    base: app::EditBase,
    reason: Option<dom::ChangeReason>,
    transition: AppointmentTransition,
) -> app::AppResult<PreparedChange> {
    let row = super::outbound::appointment_rows(c, s)?
        .into_iter()
        .find(|row| {
            row.visible()
                .is_some_and(|v| v.appointment_id == appointment.as_str())
        })
        .ok_or_else(missing)?;
    let context = edit_base(&row, &base)?;
    let current = row.visible().ok_or_else(missing)?;
    let next = dom::apply_transition(transition, Some(current.status()?))?;
    let draft = draft_of(current, next, current.confirmed_at.clone())?;

    let action = match transition {
        AppointmentTransition::Cancel => "CancelAppointment",
        AppointmentTransition::Complete => "CompleteAppointment",
        AppointmentTransition::MarkNoShow => "MarkAppointmentNoShow",
        _ => {
            return Err(AppointmentsError::internal(
                "transición de ciclo de vida inválida",
            ));
        }
    };

    Ok(PreparedChange::new(
        action,
        appointment.as_str().into(),
        draft,
        reason_payload(&reason),
        context,
    ))
}

/// A provisional retires only when its operation was accepted and the accepted
/// view reached the receipt's confirmed version. A receipt alone never replaces
/// the download; conflicts, rejections and unknown results keep the provisional.
pub(super) fn confirmed(
    c: &Connection,
    s: &crate::shared_kernel::id::OperationContext,
    accepted_version: Option<i64>,
    local: &super::outbound::LocalChange,
) -> app::AppResult<bool> {
    use crate::components::sync::domain::OperationStatus;

    let Some(stored) = crate::components::sync::adapters::outbound::SqliteOperationStore
        .operation_result(
            c,
            s,
            &OperationId::try_from(local.operation_id.clone())
                .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?,
        )
        .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?
    else {
        return Ok(false);
    };

    if stored.review_code.is_some() || stored.status != OperationStatus::Accepted {
        return Ok(false);
    }

    let Some(confirmed_version) = stored
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
        .ok_or_else(|| AppointmentsError::internal("versión local inválida"))?;

    // A provisional resolves only when the receipt is accepted and BOTH the
    // accepted appointment and its history reached the confirmed version.
    if accepted_version.is_none_or(|v| v < confirmed_version) {
        return Ok(false);
    }

    let history_version: i64 = super::outbound::history_version(c, s, &local.appointment_id)
        .map_err(|_| AppointmentsError::internal("historia local no disponible"))?;

    Ok(confirmed_version >= target && history_version >= confirmed_version)
}
