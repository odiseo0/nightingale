use super::*;

#[derive(Clone)]
pub(super) struct EditContext {
    pub version: i64,
    pub operation_id: Option<String>,
    pub depends_on: Vec<OperationId>,
}

pub(super) fn profile_changes(
    c: &Connection,
    s: &OperationContext,
) -> app::Result<Vec<LocalChange>> {
    let mut changes: Vec<_> = read_local_changes(c, s)
        .map_err(storage_error)?
        .into_iter()
        .filter(|v| v.entity_type == "patient_profile")
        .collect();

    // Each step consumes the prior version; a device clock does not order edits.
    changes.sort_by(|a, b| {
        (a.expected_version, &a.operation_id).cmp(&(b.expected_version, &b.operation_id))
    });

    Ok(changes)
}

pub(super) fn contact_changes(
    c: &Connection,
    s: &OperationContext,
) -> app::Result<Vec<LocalChange>> {
    let mut changes: Vec<_> = read_local_changes(c, s)
        .map_err(storage_error)?
        .into_iter()
        .filter(|v| v.entity_type == "contact_person")
        .collect();

    changes.sort_by(|a, b| {
        (a.expected_version, &a.operation_id).cmp(&(b.expected_version, &b.operation_id))
    });

    Ok(changes)
}

pub(super) fn relationship_changes(
    c: &Connection,
    s: &OperationContext,
) -> app::Result<Vec<LocalChange>> {
    let mut changes: Vec<_> = read_local_changes(c, s)
        .map_err(storage_error)?
        .into_iter()
        .filter(|v| v.entity_type == "patient_contact")
        .collect();

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
        .ok_or_else(|| app::PatientsError::new("INTERNAL_ERROR", "versión local inválida"))?;

    Ok(confirmed >= target && accepted_version.is_some_and(|v| v >= confirmed))
}

/// The explicit completion operation behind a profile's current draft state,
/// even when a later edit is the profile's latest local operation.
pub(super) fn draft_move<'a>(
    changes: &'a [LocalChange],
    patient_id: &str,
) -> Option<&'a LocalChange> {
    changes
        .iter()
        .filter(|v| {
            v.entity_type == "patient_profile"
                && v.patient_id.as_deref() == Some(patient_id)
                && v.action == "SetPatientCompletion"
        })
        .max_by(|a, b| {
            (a.expected_version, &a.operation_id).cmp(&(b.expected_version, &b.operation_id))
        })
}

pub(super) fn edit_base<T>(
    row: &app::EntityRow<T>,
    base: &app::EditBase,
) -> app::Result<EditContext> {
    let stale = || {
        app::PatientsError::new(
            "VERSION_CONFLICT",
            "la base local cambió; consulte la ficha",
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
                return Err(app::PatientsError::new(
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
