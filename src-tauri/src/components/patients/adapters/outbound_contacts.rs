use super::super::super::domain::{ContactData, ContactPatch, FieldChange};
use super::*;

pub(super) fn contact_state(record: ContactRecord) -> app::ContactState {
    app::ContactState {
        contact_person_id: record.contact_person_id,
        full_name: record.full_name,
        phone_code: record.phone_code,
        phone: record.phone,
        issuing_country: record.issuing_country,
        document_type: record.document_type,
        document_number: record.document_number,
        status: record.status,
    }
}

fn contact_data(state: &app::ContactState) -> app::Result<ContactData> {
    ContactData::new(
        state.full_name.clone(),
        state.phone_code.clone(),
        state.phone.clone(),
        state.issuing_country.clone(),
        state
            .document_type
            .as_deref()
            .map(TryInto::try_into)
            .transpose()?,
        state.document_number.clone(),
    )
    .map_err(Into::into)
}

fn contact_payload(data: &ContactData) -> Value {
    json!({
        "fullName": data.full_name,
        "phoneCode": data.phone.as_ref().map(|p| p.code()),
        "phone": data.phone.as_ref().map(|p| p.number()),
        "issuingCountry": data.document.as_ref().map(|d| d.issuing_country()),
        "documentType": data.document.as_ref().map(|d| d.document_type().as_str()),
        "documentNumber": data.document.as_ref().map(|d| d.number()),
    })
}

fn patch_payload(patch: &ContactPatch) -> Value {
    let mut fields = serde_json::Map::new();

    for (name, change) in [
        ("fullName", &patch.full_name),
        ("phoneCode", &patch.phone_code),
        ("phone", &patch.phone),
        ("issuingCountry", &patch.issuing_country),
        ("documentNumber", &patch.document_number),
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

    match patch.document_type {
        FieldChange::Keep => {}
        FieldChange::Clear => {
            fields.insert("documentType".into(), Value::Null);
        }
        FieldChange::Set(value) => {
            fields.insert("documentType".into(), json!(value.as_str()));
        }
    }

    Value::Object(fields)
}

pub(super) fn rows(
    c: &Connection,
    s: &OperationContext,
) -> app::Result<Vec<app::EntityRow<app::ContactState>>> {
    let mut rows = BTreeMap::new();

    for record in read_contacts(c, s).map_err(storage_error)? {
        let version = record.version;
        let state = contact_state(record);
        rows.insert(
            state.contact_person_id.clone(),
            app::EntityRow {
                accepted: Some(app::Accepted { state, version }),
                provisional: None,
                operations: vec![],
            },
        );
    }

    for local in intents::contact_changes(c, s)? {
        let state: app::ContactState = serde_json::from_str(&local.draft)
            .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?;
        let accepted_version = rows
            .get(&state.contact_person_id)
            .and_then(|r| r.accepted.as_ref().map(|a| a.version));
        let row = rows
            .entry(state.contact_person_id.clone())
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

pub(super) fn prepare(
    c: &Connection,
    s: &OperationContext,
    mutation: app::PatientMutation,
    ids: &impl IdGenerator,
    now: i64,
) -> app::Result<PreparedChange> {
    let (contact, base) = match &mutation {
        app::PatientMutation::CreateContact { data } => {
            let id = ids.new_id().to_string();
            canonical_uuid_v7(&id)?;
            let state = app::ContactState {
                contact_person_id: id.clone(),
                full_name: data.full_name.clone(),
                phone_code: data.phone.as_ref().map(|p| p.code().into()),
                phone: data.phone.as_ref().map(|p| p.number().into()),
                issuing_country: data.document.as_ref().map(|d| d.issuing_country().into()),
                document_type: data
                    .document
                    .as_ref()
                    .map(|d| d.document_type().as_str().into()),
                document_number: data.document.as_ref().map(|d| d.number().into()),
                status: "active".into(),
            };

            return Ok(PreparedChange {
                entity_type: "contact_person",
                patient_id: None,
                identifier_id: None,
                contact_person_id: Some(id.clone()),
                key: json!(id),
                draft: serde_json::to_value(state)
                    .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?,
                payload: contact_payload(data),
                base: intents::EditContext {
                    version: 0,
                    operation_id: None,
                    depends_on: vec![],
                },
            });
        }
        app::PatientMutation::UpdateContact { contact, base, .. }
        | app::PatientMutation::SetContactStatus { contact, base, .. } => (contact, base),
        _ => unreachable!(),
    };

    let row = rows(c, s)?
        .into_iter()
        .find(|r| {
            r.visible()
                .is_some_and(|v| v.contact_person_id == contact.as_str())
        })
        .ok_or_else(|| {
            app::PatientsError::new("NOT_FOUND", "contacto no disponible en este ámbito")
        })?;
    let base = intents::edit_base(&row, base)?;
    let original = row.visible().ok_or_else(|| {
        app::PatientsError::new("NOT_FOUND", "contacto no disponible en este ámbito")
    })?;
    let mut state = original.clone();
    let payload = match mutation {
        app::PatientMutation::UpdateContact { patch, .. } => {
            let data = patch.apply(&contact_data(original)?)?;
            state.full_name = data.full_name.clone();
            state.phone_code = data.phone.as_ref().map(|p| p.code().into());
            state.phone = data.phone.as_ref().map(|p| p.number().into());
            state.issuing_country = data.document.as_ref().map(|d| d.issuing_country().into());
            state.document_type = data
                .document
                .as_ref()
                .map(|d| d.document_type().as_str().into());
            state.document_number = data.document.as_ref().map(|d| d.number().into());
            patch_payload(&patch)
        }
        app::PatientMutation::SetContactStatus { status, .. } => {
            state.status = status.as_str().into();
            json!({"status":status.as_str()})
        }
        _ => unreachable!(),
    };

    if &state == original {
        return Err(app::PatientsError::new(
            "NO_CHANGES",
            "el contacto no tiene cambios",
        ));
    }

    // A shared contact edit cannot silently break complete profiles that rely
    // on it as the responsible phone; explicit drafts become dependencies.
    let future_active = state.status == "active";
    let future_phone = state.phone_code.is_some() && state.phone.is_some();
    let impact = documents::responsible_impact(c, s, now, &state.contact_person_id, |at, rel| {
        documents::supports_at(rel, at, future_active, future_phone)
    })?;
    let base = documents::apply_impact(impact, &base)?;

    Ok(PreparedChange {
        entity_type: "contact_person",
        patient_id: None,
        identifier_id: None,
        contact_person_id: Some(state.contact_person_id.clone()),
        key: json!(state.contact_person_id),
        draft: serde_json::to_value(state)
            .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?,
        payload,
        base,
    })
}
