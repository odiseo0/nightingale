use super::super::super::domain::{Document, IdentificationBasis, parse_instant};
use super::*;

pub(super) fn accepted_state(record: IdentifierRecord) -> app::IdentifierState {
    app::IdentifierState {
        id: record.identifier_id,
        patient_id: record.patient_id,
        issuing_country: record.issuing_country,
        document_type: record.document_type,
        document_number: record.document_number,
        verification_status: record.verification_status,
    }
}

fn declared(id: String, patient: String, document: &Document) -> app::IdentifierState {
    app::IdentifierState {
        id,
        patient_id: patient,
        issuing_country: document.issuing_country().into(),
        document_type: document.document_type().as_str().into(),
        document_number: document.number().into(),
        verification_status: "declared".into(),
    }
}

pub(super) fn rows(
    c: &Connection,
    s: &OperationContext,
) -> app::Result<Vec<app::EntityRow<app::IdentifierState>>> {
    let mut rows = BTreeMap::new();

    for record in read_identifiers(c, s).map_err(storage_error)? {
        let version = record.version;
        let state = accepted_state(record);
        rows.insert(
            state.id.clone(),
            app::EntityRow {
                accepted: Some(app::Accepted { state, version }),
                provisional: None,
                operations: vec![],
            },
        );
    }

    for local in read_local_changes(c, s).map_err(storage_error)? {
        let states = if local.entity_type == "patient_identifier" {
            vec![
                serde_json::from_str::<app::IdentifierState>(&local.draft)
                    .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?,
            ]
        } else if local.action == "RegisterPatient" {
            let payload: Value = serde_json::from_str(&local.payload)
                .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?;
            let patient = local
                .patient_id
                .as_deref()
                .ok_or_else(|| storage_error(rusqlite::Error::InvalidQuery))?;
            let mut states = vec![];

            if let Some(items) = payload.get("identifiers").and_then(Value::as_array) {
                for item in items {
                    let text = |key: &str| {
                        item.get(key)
                            .and_then(Value::as_str)
                            .ok_or_else(|| storage_error(rusqlite::Error::InvalidQuery))
                    };
                    let document = Document::new(
                        text("issuingCountry")?.into(),
                        text("documentType")?.try_into()?,
                        text("documentNumber")?.into(),
                    )?;

                    states.push(declared(text("id")?.into(), patient.into(), &document));
                }
            }
            states
        } else {
            vec![]
        };

        for state in states {
            let accepted_version = rows
                .get(&state.id)
                .and_then(|r| r.accepted.as_ref().map(|a| a.version));
            let row = rows
                .entry(state.id.clone())
                .or_insert_with(|| app::EntityRow {
                    accepted: None,
                    provisional: None,
                    operations: vec![],
                });
            row.operations.push(operation_view(c, s, &local)?);
            if !intents::confirmed(c, s, accepted_version, &local)? {
                row.provisional = Some(app::Provisional {
                    state,
                    operation_id: local.operation_id.clone(),
                    expected_version: local.expected_version,
                });
            }
        }
    }

    Ok(rows.into_values().collect())
}

pub(super) fn prepare_add(
    c: &Connection,
    s: &OperationContext,
    patient: &PatientId,
    document: &Document,
    ids: &impl IdGenerator,
) -> app::Result<PreparedChange> {
    let profile = profile_rows(c, s)?
        .into_iter()
        .find(|r| {
            r.visible()
                .is_some_and(|v| v.patient_id == patient.as_str())
        })
        .ok_or_else(missing)?;
    let depends_on = if let Some(local) = &profile.provisional {
        intents::edit_base(
            &profile,
            &app::EditBase::LocalOperation {
                operation_id: local.operation_id.clone().try_into()?,
            },
        )?
        .depends_on
    } else {
        vec![]
    };
    let id = ids.new_id().to_string();
    canonical_uuid_v7(&id)?;
    let state = declared(id.clone(), patient.as_str().into(), document);

    Ok(PreparedChange {
        entity_type: "patient_identifier",
        patient_id: Some(patient.as_str().into()),
        identifier_id: Some(id.clone()),
        contact_person_id: None,
        key: json!(id),
        draft: serde_json::to_value(state)
            .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?,
        payload: json!({"patientId":patient.as_str(),"issuingCountry":document.issuing_country(),"documentType":document.document_type().as_str(),"documentNumber":document.number()}),
        base: intents::EditContext {
            version: 0,
            operation_id: None,
            depends_on,
        },
    })
}

/// Whether an active, responsible, in-force relationship plus an active contact
/// with a phone still supports a profile that needs a responsible phone.
pub(super) fn supports_at(
    state: &app::RelationshipState,
    at: &chrono::DateTime<chrono::Utc>,
    contact_active: bool,
    contact_has_phone: bool,
) -> bool {
    state.status == "active"
        && state.is_responsible_contact
        && parse_instant(&state.valid_from).is_ok_and(|v| v <= *at)
        && state
            .valid_until
            .as_deref()
            .map(parse_instant)
            .is_none_or(|v| v.is_ok_and(|v| v > *at))
        && contact_active
        && contact_has_phone
}

pub(super) struct ResponsibleImpact {
    pub conflict_patient_ids: Vec<String>,
    pub completion_dependencies: Vec<OperationId>,
}

/// Complete profiles that would lose their last responsible support when a
/// contact or one of its relationships changes. `future_support` receives the
/// current visible relationship of the edited contact and the injected
/// instant, and decides with the future state; a profile stays out of the
/// impact while any other responsible keeps holding. Profiles already moved
/// explicitly to draft contribute their completion operation as a dependency
/// instead of a conflict.
pub(super) fn responsible_impact(
    c: &Connection,
    s: &OperationContext,
    now: i64,
    contact_id: &str,
    future_support: impl Fn(&chrono::DateTime<chrono::Utc>, &app::RelationshipState) -> bool,
) -> app::Result<ResponsibleImpact> {
    let at = chrono::DateTime::from_timestamp_millis(now)
        .ok_or_else(|| app::PatientsError::new("VALIDATION_ERROR", "instante fuera de rango"))?;
    let profiles = profile_rows(c, s)?;
    let contacts_rows = contacts::rows(c, s)?;
    let relationships = relationships::rows(c, s)?;
    let local_changes = read_local_changes(c, s).map_err(storage_error)?;

    // Support rules mirror `completion_support`: a relationship holds only
    // while every operation behind its visible state and behind its contact
    // may still be depended on.
    let supports = |row: &app::EntityRow<app::RelationshipState>, future: bool| {
        let Some(rel) = row.visible() else {
            return false;
        };

        if !row
            .operations
            .iter()
            .all(|v| intents::can_depend(&v.status))
        {
            return false;
        }

        let Some(contact) = contacts_rows.iter().find(|r| {
            r.visible()
                .is_some_and(|v| v.contact_person_id == rel.contact_person_id)
        }) else {
            return false;
        };

        if contact
            .operations
            .iter()
            .any(|v| !intents::can_depend(&v.status))
        {
            return false;
        }

        if future && rel.contact_person_id == contact_id {
            return future_support(&at, rel);
        }

        let Some(visible) = contact.visible() else {
            return false;
        };

        supports_at(
            rel,
            &at,
            visible.status == "active",
            visible.phone_code.is_some() && visible.phone.is_some(),
        )
    };

    // Profiles whose support through this contact breaks with the change.
    let mut affected: Vec<String> = vec![];

    for row in relationships.iter().filter(|r| {
        r.visible()
            .is_some_and(|v| v.contact_person_id == contact_id)
    }) {
        if supports(row, false)
            && !supports(row, true)
            && let Some(rel) = row.visible()
            && !affected.contains(&rel.patient_id)
        {
            affected.push(rel.patient_id.clone());
        }
    }

    let mut impact = ResponsibleImpact {
        conflict_patient_ids: vec![],
        completion_dependencies: vec![],
    };

    for patient_id in affected {
        if relationships.iter().any(|row| {
            row.visible().is_some_and(|v| v.patient_id == patient_id) && supports(row, true)
        }) {
            continue;
        }

        let Some(profile) = profiles
            .iter()
            .find(|p| p.visible().is_some_and(|v| v.patient_id == patient_id))
        else {
            continue;
        };
        let state = profile
            .visible()
            .ok_or_else(|| app::PatientsError::new("INTERNAL_ERROR", "ficha sin estado visible"))?;
        let needs_responsible = state.identification_basis.as_deref()
            == Some("minor_without_document")
            || (state.phone_code.is_none() && state.phone.is_none());

        if !needs_responsible {
            continue;
        }

        if state.completion_status == "complete" {
            impact.conflict_patient_ids.push(state.patient_id.clone());
        } else if let Some(change) = intents::draft_move(&local_changes, &patient_id)
            && !impact
                .completion_dependencies
                .iter()
                .any(|id| id.as_str() == change.operation_id)
        {
            impact
                .completion_dependencies
                .push(change.operation_id.clone().try_into()?);
        }
    }

    Ok(impact)
}

/// Shared tail of the contact and relationship prepares: reject the edit when
/// complete profiles lose their requirements, otherwise fold the explicit
/// draft moves into the dependencies and enforce the contract limit of eight.
pub(super) fn apply_impact(
    impact: ResponsibleImpact,
    base: &intents::EditContext,
) -> app::Result<intents::EditContext> {
    if !impact.conflict_patient_ids.is_empty() {
        let mut error = app::PatientsError::new(
            "COMPLETENESS_CONFLICT",
            "la edición deja fichas completas sin sus requisitos; páselas a draft",
        );
        error.affected_patient_ids = impact.conflict_patient_ids;

        return Err(error);
    }

    let mut base = base.clone();

    for id in impact.completion_dependencies {
        if !base.depends_on.contains(&id) {
            base.depends_on.push(id);
        }
    }

    if base.depends_on.len() > 8 {
        return Err(app::PatientsError::new(
            "VALIDATION_ERROR",
            "la operación excede ocho dependencias",
        ));
    }

    Ok(base)
}

pub(super) fn completion_support(
    c: &Connection,
    s: &OperationContext,
    state: &app::ProfileState,
    now: i64,
) -> app::Result<(bool, Vec<OperationId>)> {
    let profile = profile_data(state)?;
    let mut dependencies = vec![];
    let mut has_document = false;

    if profile.identification_basis == Some(IdentificationBasis::OwnDocument) {
        for row in rows(c, s)?.into_iter().filter(|r| {
            r.visible()
                .is_some_and(|d| d.patient_id == state.patient_id)
        }) {
            if let Some(accepted) = row.accepted {
                if matches!(
                    accepted.state.verification_status.as_str(),
                    "declared" | "verified"
                ) {
                    has_document = true;
                    break;
                }
            } else if let Some(local) = row.provisional
                && row
                    .operations
                    .iter()
                    .all(|v| intents::can_depend(&v.status))
            {
                has_document = true;
                dependencies.push(local.operation_id.try_into()?);

                break;
            }
        }
    }

    let mut has_responsible_phone = false;

    if profile.identification_basis == Some(IdentificationBasis::MinorWithoutDocument)
        || profile.phone.is_none()
    {
        // Compare an injected instant, never the device's current time zone.
        // https://docs.rs/chrono/0.4.45/chrono/struct.DateTime.html#method.from_timestamp_millis
        let at = chrono::DateTime::from_timestamp_millis(now).ok_or_else(|| {
            app::PatientsError::new("VALIDATION_ERROR", "instante fuera de rango")
        })?;
        let contacts = contacts::rows(c, s)?;

        for relationship in relationships::rows(c, s)?.into_iter().filter(|r| {
            r.visible().is_some_and(|v| {
                v.patient_id == state.patient_id && v.status == "active" && v.is_responsible_contact
            })
        }) {
            // Only accepted rows or provisionals whose operations may still depend
            // count as support; review_required work cannot sustain completion.
            if relationship
                .operations
                .iter()
                .any(|v| !intents::can_depend(&v.status))
            {
                continue;
            }

            let visible = relationship.visible().ok_or_else(|| {
                app::PatientsError::new("INTERNAL_ERROR", "vínculo sin estado visible")
            })?;

            if !supports_at(visible, &at, true, true) {
                continue;
            }

            for contact in contacts.iter().filter(|row| {
                row.visible().is_some_and(|p| {
                    p.contact_person_id == visible.contact_person_id
                        && p.status == "active"
                        && p.phone_code.is_some()
                        && p.phone.is_some()
                })
            }) {
                if contact
                    .operations
                    .iter()
                    .any(|v| !intents::can_depend(&v.status))
                {
                    continue;
                }

                has_responsible_phone = true;

                if let Some(local) = &relationship.provisional {
                    dependencies.push(local.operation_id.clone().try_into()?);
                }

                if let Some(local) = &contact.provisional {
                    dependencies.push(local.operation_id.clone().try_into()?);
                }

                break;
            }

            if has_responsible_phone {
                break;
            }
        }
    }

    Ok((
        profile.meets_completion(has_document, has_responsible_phone),
        dependencies,
    ))
}
