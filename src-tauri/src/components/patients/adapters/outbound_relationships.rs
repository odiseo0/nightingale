use super::super::super::domain::{FieldChange, RelationshipData, RelationshipPatch};
use super::*;

pub(super) fn relationship_state(record: RelationshipRecord) -> app::RelationshipState {
    app::RelationshipState {
        patient_id: record.patient_id,
        contact_person_id: record.contact_person_id,
        relationship_kind: record.relationship_kind,
        is_responsible_contact: record.is_responsible_contact,
        valid_from: record.valid_from,
        valid_until: record.valid_until,
        status: record.status,
    }
}

fn relationship_data(state: &app::RelationshipState) -> app::Result<RelationshipData> {
    RelationshipData::new(
        state.relationship_kind.as_str().try_into()?,
        state.is_responsible_contact,
        state.valid_from.clone(),
        state.valid_until.clone(),
    )
    .map_err(Into::into)
}

fn relationship_payload(data: &RelationshipData) -> Value {
    json!({
        "relationshipKind": data.relationship_kind.as_str(),
        "isResponsibleContact": data.is_responsible_contact,
        "validFrom": data.valid_from,
        "validUntil": data.valid_until,
    })
}

fn patch_payload(patch: &RelationshipPatch) -> Value {
    let mut fields = serde_json::Map::new();

    match patch.is_responsible_contact {
        FieldChange::Keep => {}
        FieldChange::Clear => {
            fields.insert("isResponsibleContact".into(), Value::Null);
        }
        FieldChange::Set(value) => {
            fields.insert("isResponsibleContact".into(), json!(value));
        }
    }

    for (name, change) in [
        ("validFrom", &patch.valid_from),
        ("validUntil", &patch.valid_until),
    ] {
        match change {
            FieldChange::Keep => {}
            FieldChange::Clear => {
                fields.insert(name.into(), Value::Null);
            }
            FieldChange::Set(value) => {
                fields.insert(name.into(), json!(value));
            }
        }
    }

    match patch.relationship_kind {
        FieldChange::Keep => {}
        FieldChange::Clear => {
            fields.insert("relationshipKind".into(), Value::Null);
        }
        FieldChange::Set(value) => {
            fields.insert("relationshipKind".into(), json!(value.as_str()));
        }
    }

    Value::Object(fields)
}

pub(super) fn rows(
    c: &Connection,
    s: &OperationContext,
) -> app::Result<Vec<app::EntityRow<app::RelationshipState>>> {
    let mut rows = BTreeMap::new();

    for record in read_relationships(c, s).map_err(storage_error)? {
        let version = record.version;
        let state = relationship_state(record);
        rows.insert(
            (state.patient_id.clone(), state.contact_person_id.clone()),
            app::EntityRow {
                accepted: Some(app::Accepted { state, version }),
                provisional: None,
                operations: vec![],
            },
        );
    }

    for local in intents::relationship_changes(c, s)? {
        let state: app::RelationshipState = serde_json::from_str(&local.draft)
            .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?;
        let accepted_version = rows
            .get(&(state.patient_id.clone(), state.contact_person_id.clone()))
            .and_then(|r| r.accepted.as_ref().map(|a| a.version));
        let row = rows
            .entry((state.patient_id.clone(), state.contact_person_id.clone()))
            .or_insert_with(|| app::EntityRow {
                accepted: None,
                provisional: None,
                operations: vec![],
            });
        row.operations.push(operation_view(c, s, &local)?);

        if !intents::confirmed(c, s, accepted_version, &local)? {
            row.provisional = Some(app::Provisional {
                state,
                operation_id: local.operation_id,
                expected_version: local.expected_version,
            });
        }
    }

    Ok(rows.into_values().collect())
}

/// Related names and phone are read-side data of this account; they are never
/// persisted into the relationship's accepted or provisional state.
pub(super) fn views(
    c: &Connection,
    s: &OperationContext,
) -> app::Result<Vec<app::EntityRow<app::RelationshipView>>> {
    let profiles = profile_rows(c, s)?;
    let contacts = contacts::rows(c, s)?;

    rows(c, s)?
        .into_iter()
        .map(|row| {
            let state = row.visible().ok_or_else(|| {
                app::PatientsError::new("INTERNAL_ERROR", "vínculo sin estado visible")
            })?;
            let patient_name = profiles.iter().find_map(|r| {
                r.visible()
                    .filter(|p| p.patient_id == state.patient_id)
                    .and_then(|p| p.full_name.clone())
            });
            let contact = contacts
                .iter()
                .find_map(|r| {
                    r.visible()
                        .filter(|v| v.contact_person_id == state.contact_person_id)
                })
                .ok_or_else(|| {
                    app::PatientsError::new("INTERNAL_ERROR", "contacto del vínculo ausente")
                })?;

            Ok(app::EntityRow {
                accepted: row.accepted.map(|a| app::Accepted {
                    state: view(a.state, patient_name.clone(), contact),
                    version: a.version,
                }),
                provisional: row.provisional.map(|p| app::Provisional {
                    state: view(p.state, patient_name.clone(), contact),
                    operation_id: p.operation_id,
                    expected_version: p.expected_version,
                }),
                operations: row.operations,
            })
        })
        .collect()
}

fn view(
    state: app::RelationshipState,
    patient_full_name: Option<String>,
    contact: &app::ContactState,
) -> app::RelationshipView {
    app::RelationshipView {
        relationship: state,
        patient_full_name,
        contact_full_name: contact.full_name.clone(),
        contact_phone_code: contact.phone_code.clone(),
        contact_phone: contact.phone.clone(),
    }
}

fn key(patient: &str, contact: &str) -> Value {
    json!({"patientId": patient, "contactPersonId": contact})
}

pub(super) fn prepare(
    c: &Connection,
    s: &OperationContext,
    mutation: app::PatientMutation,
    now: i64,
) -> app::Result<PreparedChange> {
    let (patient, contact, base) = match &mutation {
        app::PatientMutation::CreateRelationship {
            patient,
            contact,
            data,
        } => {
            // Both ends must exist in this scope; local creations become dependencies.
            let profile = profile_rows(c, s)?
                .into_iter()
                .find(|r| {
                    r.visible()
                        .is_some_and(|v| v.patient_id == patient.as_str())
                })
                .ok_or_else(missing)?;
            let contact_row = contacts::rows(c, s)?
                .into_iter()
                .find(|r| {
                    r.visible()
                        .is_some_and(|v| v.contact_person_id == contact.as_str())
                })
                .ok_or_else(|| {
                    app::PatientsError::new("NOT_FOUND", "contacto no disponible en este ámbito")
                })?;

            if rows(c, s)?.iter().any(|r| {
                r.visible().is_some_and(|v| {
                    v.patient_id == patient.as_str() && v.contact_person_id == contact.as_str()
                })
            }) {
                return Err(app::PatientsError::new(
                    "VALIDATION_ERROR",
                    "el vínculo ya existe",
                ));
            }

            let mut depends_on = vec![];

            if let Some(local) = &profile.provisional {
                depends_on.push(local.operation_id.clone().try_into()?);
            }

            if let Some(local) = &contact_row.provisional {
                depends_on.push(local.operation_id.clone().try_into()?);
            }

            let state = app::RelationshipState {
                patient_id: patient.as_str().into(),
                contact_person_id: contact.as_str().into(),
                relationship_kind: data.relationship_kind.as_str().into(),
                is_responsible_contact: data.is_responsible_contact,
                valid_from: data.valid_from.clone(),
                valid_until: data.valid_until.clone(),
                status: "active".into(),
            };

            return Ok(PreparedChange {
                entity_type: "patient_contact",
                patient_id: Some(patient.as_str().into()),
                identifier_id: None,
                contact_person_id: Some(contact.as_str().into()),
                key: key(patient.as_str(), contact.as_str()),
                draft: serde_json::to_value(state)
                    .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?,
                payload: relationship_payload(data),
                base: intents::EditContext {
                    version: 0,
                    operation_id: None,
                    depends_on,
                },
            });
        }
        app::PatientMutation::UpdateRelationship {
            patient,
            contact,
            base,
            ..
        }
        | app::PatientMutation::SetRelationshipStatus {
            patient,
            contact,
            base,
            ..
        } => (patient, contact, base),
        _ => unreachable!(),
    };

    let row = rows(c, s)?
        .into_iter()
        .find(|r| {
            r.visible().is_some_and(|v| {
                v.patient_id == patient.as_str() && v.contact_person_id == contact.as_str()
            })
        })
        .ok_or_else(|| {
            app::PatientsError::new("NOT_FOUND", "vínculo no disponible en este ámbito")
        })?;
    let base = intents::edit_base(&row, base)?;
    let original = row.visible().ok_or_else(|| {
        app::PatientsError::new("NOT_FOUND", "vínculo no disponible en este ámbito")
    })?;
    let mut state = original.clone();
    let payload = match mutation {
        app::PatientMutation::UpdateRelationship { patch, .. } => {
            let data = patch.apply(&relationship_data(original)?)?;
            state.relationship_kind = data.relationship_kind.as_str().into();
            state.is_responsible_contact = data.is_responsible_contact;
            state.valid_from = data.valid_from;
            state.valid_until = data.valid_until;
            patch_payload(&patch)
        }
        app::PatientMutation::SetRelationshipStatus { status, .. } => {
            state.status = status.as_str().into();
            json!({"status":status.as_str()})
        }
        _ => unreachable!(),
    };

    if &state == original {
        return Err(app::PatientsError::new(
            "NO_CHANGES",
            "el vínculo no tiene cambios",
        ));
    }

    // Relationship edits face the same protection for complete profiles that
    // rely on this responsible link; explicit drafts become dependencies.
    // The contact itself does not change here, so its current flags decide.
    let future = state.clone();
    let (contact_active, contact_has_phone) = contacts::rows(c, s)?
        .into_iter()
        .find_map(|r| {
            r.visible()
                .filter(|v| v.contact_person_id == future.contact_person_id)
                .map(|v| {
                    (
                        v.status == "active",
                        v.phone_code.is_some() && v.phone.is_some(),
                    )
                })
        })
        .unwrap_or((false, false));
    let impact = documents::responsible_impact(c, s, now, &state.contact_person_id, |at, rel| {
        // Only this relationship changes; the contact's other links keep theirs.
        let future_rel = if rel.patient_id == future.patient_id
            && rel.contact_person_id == future.contact_person_id
        {
            &future
        } else {
            rel
        };

        documents::supports_at(future_rel, at, contact_active, contact_has_phone)
    })?;
    let base = documents::apply_impact(impact, &base)?;

    Ok(PreparedChange {
        entity_type: "patient_contact",
        patient_id: Some(state.patient_id.clone()),
        identifier_id: None,
        contact_person_id: Some(state.contact_person_id.clone()),
        key: key(&state.patient_id, &state.contact_person_id),
        draft: serde_json::to_value(state)
            .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?,
        payload,
        base,
    })
}
