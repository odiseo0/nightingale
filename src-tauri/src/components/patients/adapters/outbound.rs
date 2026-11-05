//! Scoped storage only. Use cases own permission checks and the write transaction.
//! Reads bind all three scope fields. Writes use the caller's transaction.
//! Sources: https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Statement.html#method.query_map
//! https://www.sqlite.org/lang_upsert.html (version guards keep newer rows).
use super::super::{
    application as app,
    domain::{PatientId, ProfileData},
};
use crate::shared_kernel::id::OperationContext;
use crate::{
    components::{
        identity::{
            domain::AccountAccess,
            repository::{AccessStore, SqliteAccessStore},
        },
        sync::{
            adapters::outbound::SqliteOperationStore, application::record_operation,
            domain::NewOperation,
        },
    },
    shared_kernel::{
        Clock, IdGenerator,
        id::{OperationId, canonical_uuid_v7},
    },
};
use rusqlite::{Connection, Transaction, params};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileRecord {
    pub patient_id: String,
    pub full_name: Option<String>,
    pub birth_date: Option<String>,
    pub phone_code: Option<String>,
    pub phone: Option<String>,
    pub identification_basis: Option<String>,
    pub completion_status: String,
    pub status: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub version: i64,
}

pub fn read_profiles(
    connection: &Connection,
    scope: &OperationContext,
) -> rusqlite::Result<Vec<ProfileRecord>> {
    let mut query = connection.prepare("SELECT patient_id,full_name,birth_date,phone_code,phone,identification_basis,completion_status,status,created_at,updated_at,version FROM patient_profile_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 ORDER BY patient_id")?;
    query
        .query_map(
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str()
            ],
            |row| {
                Ok(ProfileRecord {
                    patient_id: row.get(0)?,
                    full_name: row.get(1)?,
                    birth_date: row.get(2)?,
                    phone_code: row.get(3)?,
                    phone: row.get(4)?,
                    identification_basis: row.get(5)?,
                    completion_status: row.get(6)?,
                    status: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    version: row.get(10)?,
                })
            },
        )?
        .collect()
}

pub fn save_profiles(
    transaction: &Transaction<'_>,
    scope: &OperationContext,
    record: &ProfileRecord,
) -> rusqlite::Result<()> {
    transaction.execute("INSERT INTO patient_profile_view (account_id,user_id,device_id,patient_id,full_name,birth_date,phone_code,phone,identification_basis,completion_status,status,created_at,updated_at,version) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14) ON CONFLICT(account_id,user_id,device_id,patient_id) DO UPDATE SET full_name=excluded.full_name,birth_date=excluded.birth_date,phone_code=excluded.phone_code,phone=excluded.phone,identification_basis=excluded.identification_basis,completion_status=excluded.completion_status,status=excluded.status,created_at=excluded.created_at,updated_at=excluded.updated_at,version=excluded.version WHERE excluded.version > patient_profile_view.version",
        params![
            scope.account_id.as_str(),
            scope.user_id.as_str(),
            scope.device_id.as_str(),
            record.patient_id,
            record.full_name,
            record.birth_date,
            record.phone_code,
            record.phone, record.identification_basis,
            record.completion_status,
            record.status, record.created_at,
            record.updated_at,
            record.version
        ])?;

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifierRecord {
    pub identifier_id: String,
    pub patient_id: String,
    pub issuing_country: String,
    pub document_type: String,
    pub document_number: String,
    pub verification_status: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub version: i64,
}

pub fn read_identifiers(
    connection: &Connection,
    scope: &OperationContext,
) -> rusqlite::Result<Vec<IdentifierRecord>> {
    let mut query = connection.prepare("SELECT identifier_id,patient_id,issuing_country,document_type,document_number,verification_status,created_at,updated_at,version FROM patient_identifier_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 ORDER BY identifier_id")?;
    query
        .query_map(
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str()
            ],
            |row| {
                Ok(IdentifierRecord {
                    identifier_id: row.get(0)?,
                    patient_id: row.get(1)?,
                    issuing_country: row.get(2)?,
                    document_type: row.get(3)?,
                    document_number: row.get(4)?,
                    verification_status: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                    version: row.get(8)?,
                })
            },
        )?
        .collect()
}

pub fn save_identifiers(
    transaction: &Transaction<'_>,
    scope: &OperationContext,
    record: &IdentifierRecord,
) -> rusqlite::Result<()> {
    transaction.execute("INSERT INTO patient_identifier_view (account_id,user_id,device_id,identifier_id,patient_id,issuing_country,document_type,document_number,verification_status,created_at,updated_at,version) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12) ON CONFLICT(account_id,user_id,device_id,identifier_id) DO UPDATE SET patient_id=excluded.patient_id,issuing_country=excluded.issuing_country,document_type=excluded.document_type,document_number=excluded.document_number,verification_status=excluded.verification_status,created_at=excluded.created_at,updated_at=excluded.updated_at,version=excluded.version WHERE excluded.version > patient_identifier_view.version",
        params![
            scope.account_id.as_str(),
            scope.user_id.as_str(),
            scope.device_id.as_str(),
            record.identifier_id,
            record.patient_id,
            record.issuing_country,
            record.document_type,
            record.document_number,
            record.verification_status,
            record.created_at,
            record.updated_at,
            record.version
        ])?;

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactRecord {
    pub contact_person_id: String,
    pub full_name: String,
    pub phone_code: Option<String>,
    pub phone: Option<String>,
    pub issuing_country: Option<String>,
    pub document_type: Option<String>,
    pub document_number: Option<String>,
    pub status: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub version: i64,
}

pub fn read_contacts(
    connection: &Connection,
    scope: &OperationContext,
) -> rusqlite::Result<Vec<ContactRecord>> {
    let mut query = connection.prepare("SELECT contact_person_id,full_name,phone_code,phone,issuing_country,document_type,document_number,status,created_at,updated_at,version FROM contact_person_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 ORDER BY contact_person_id")?;
    query
        .query_map(
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str()
            ],
            |row| {
                Ok(ContactRecord {
                    contact_person_id: row.get(0)?,
                    full_name: row.get(1)?,
                    phone_code: row.get(2)?,
                    phone: row.get(3)?,
                    issuing_country: row.get(4)?,
                    document_type: row.get(5)?,
                    document_number: row.get(6)?,
                    status: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    version: row.get(10)?,
                })
            },
        )?
        .collect()
}

pub fn save_contacts(
    transaction: &Transaction<'_>,
    scope: &OperationContext,
    record: &ContactRecord,
) -> rusqlite::Result<()> {
    transaction.execute("INSERT INTO contact_person_view (account_id,user_id,device_id,contact_person_id,full_name,phone_code,phone,issuing_country,document_type,document_number,status,created_at,updated_at,version) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14) ON CONFLICT(account_id,user_id,device_id,contact_person_id) DO UPDATE SET full_name=excluded.full_name,phone_code=excluded.phone_code,phone=excluded.phone,issuing_country=excluded.issuing_country,document_type=excluded.document_type,document_number=excluded.document_number,status=excluded.status,created_at=excluded.created_at,updated_at=excluded.updated_at,version=excluded.version WHERE excluded.version > contact_person_view.version",
        params![
            scope.account_id.as_str(),
            scope.user_id.as_str(),
            scope.device_id.as_str(),
            record.contact_person_id,
            record.full_name,
            record.phone_code,
            record.phone,
            record.issuing_country,
            record.document_type,
            record.document_number,
            record.status,
            record.created_at,
            record.updated_at,
            record.version
        ])?;

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationshipRecord {
    pub patient_id: String,
    pub contact_person_id: String,
    pub relationship_kind: String,
    pub is_responsible_contact: bool,
    pub representation_status: String,
    pub verified_at: Option<String>,
    pub verified_by_user_id: Option<String>,
    pub valid_from: String,
    pub valid_until: Option<String>,
    pub status: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub version: i64,
}

pub fn read_relationships(
    connection: &Connection,
    scope: &OperationContext,
) -> rusqlite::Result<Vec<RelationshipRecord>> {
    let mut query = connection.prepare("SELECT patient_id,contact_person_id,relationship_kind,is_responsible_contact,representation_status,verified_at,verified_by_user_id,valid_from,valid_until,status,created_at,updated_at,version FROM patient_contact_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 ORDER BY patient_id,contact_person_id")?;
    query
        .query_map(
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str()
            ],
            |row| {
                Ok(RelationshipRecord {
                    patient_id: row.get(0)?,
                    contact_person_id: row.get(1)?,
                    relationship_kind: row.get(2)?,
                    is_responsible_contact: row.get(3)?,
                    representation_status: row.get(4)?,
                    verified_at: row.get(5)?,
                    verified_by_user_id: row.get(6)?,
                    valid_from: row.get(7)?,
                    valid_until: row.get(8)?,
                    status: row.get(9)?,
                    created_at: row.get(10)?,
                    updated_at: row.get(11)?,
                    version: row.get(12)?,
                })
            },
        )?
        .collect()
}

pub fn save_relationships(
    transaction: &Transaction<'_>,
    scope: &OperationContext,
    record: &RelationshipRecord,
) -> rusqlite::Result<()> {
    transaction.execute("INSERT INTO patient_contact_view (account_id,user_id,device_id,patient_id,contact_person_id,relationship_kind,is_responsible_contact,representation_status,verified_at,verified_by_user_id,valid_from,valid_until,status,created_at,updated_at,version) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16) ON CONFLICT(account_id,user_id,device_id,patient_id,contact_person_id) DO UPDATE SET relationship_kind=excluded.relationship_kind,is_responsible_contact=excluded.is_responsible_contact,representation_status=excluded.representation_status,verified_at=excluded.verified_at,verified_by_user_id=excluded.verified_by_user_id,valid_from=excluded.valid_from,valid_until=excluded.valid_until,status=excluded.status,created_at=excluded.created_at,updated_at=excluded.updated_at,version=excluded.version WHERE excluded.version > patient_contact_view.version",
        params![
            scope.account_id.as_str(),
            scope.user_id.as_str(),
            scope.device_id.as_str(),
            record.patient_id,
            record.contact_person_id,
            record.relationship_kind,
            record.is_responsible_contact,
            record.representation_status,
            record.verified_at,
            record.verified_by_user_id,
            record.valid_from,
            record.valid_until,
            record.status,
            record.created_at,
            record.updated_at,
            record.version
        ])?;

    Ok(())
}

/// JSON strings hold the exact payload, dependencies, and local state.
/// Operation receipts and permission epochs belong to sync, not this record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalChange {
    pub operation_id: String,
    pub entity_type: String,
    pub patient_id: Option<String>,
    pub identifier_id: Option<String>,
    pub contact_person_id: Option<String>,
    pub action: String,
    pub expected_version: i64,
    pub base_operation_id: Option<String>,
    pub depends_on: String,
    pub payload: String,
    pub draft: String,
    pub created_at: i64,
}

pub fn read_local_changes(
    connection: &Connection,
    scope: &OperationContext,
) -> rusqlite::Result<Vec<LocalChange>> {
    let mut query = connection.prepare("SELECT operation_id,entity_type,patient_id,identifier_id,contact_person_id,action,expected_version,base_operation_id,depends_on,payload,draft,created_at FROM patient_local WHERE account_id=?1 AND user_id=?2 AND device_id=?3 ORDER BY created_at,operation_id")?;
    query
        .query_map(
            params![
                scope.account_id.as_str(),
                scope.user_id.as_str(),
                scope.device_id.as_str()
            ],
            |row| {
                Ok(LocalChange {
                    operation_id: row.get(0)?,
                    entity_type: row.get(1)?,
                    patient_id: row.get(2)?,
                    identifier_id: row.get(3)?,
                    contact_person_id: row.get(4)?,
                    action: row.get(5)?,
                    expected_version: row.get(6)?,
                    base_operation_id: row.get(7)?,
                    depends_on: row.get(8)?,
                    payload: row.get(9)?,
                    draft: row.get(10)?,
                    created_at: row.get(11)?,
                })
            },
        )?
        .collect()
}

/// Append inside the use case's transaction, alongside record_operation.
/// Source: https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Transaction.html
pub fn insert_local_change(
    transaction: &Transaction<'_>,
    scope: &OperationContext,
    record: &LocalChange,
) -> rusqlite::Result<()> {
    transaction.execute("INSERT INTO patient_local (account_id,user_id,device_id,operation_id,entity_type,patient_id,identifier_id,contact_person_id,action,expected_version,base_operation_id,depends_on,payload,draft,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
        params![
            scope.account_id.as_str(),
            scope.user_id.as_str(),
            scope.device_id.as_str(),
            record.operation_id,
            record.entity_type,
            record.patient_id,
            record.identifier_id,
            record.contact_person_id,
            record.action,
            record.expected_version,
            record.base_operation_id,
            record.depends_on,
            record.payload,
            record.draft,
            record.created_at
        ])?;

    Ok(())
}

pub struct SqlitePatientRepository<'a> {
    pub connection: &'a mut Connection,
}

fn storage_error(_: rusqlite::Error) -> app::PatientsError {
    app::PatientsError::new(
        "INTERNAL_ERROR",
        "datos locales de pacientes no disponibles",
    )
}

fn missing() -> app::PatientsError {
    app::PatientsError::new("NOT_FOUND", "paciente no disponible en este ámbito")
}

fn current_access(c: &Connection, access: &AccountAccess, action: Option<&str>) -> app::Result<()> {
    let s = &access.context;
    let current = SqliteAccessStore
        .access(c, &s.account_id, &s.user_id, &s.device_id)?
        .ok_or_else(|| app::PatientsError::new("PERMISSION_DENIED", "no hay permiso local"))?;
    app::require_read(&current)?;

    if current.context.permission_epoch != s.permission_epoch {
        return Err(app::PatientsError::new(
            "PERMISSION_DENIED",
            "los permisos locales cambiaron",
        ));
    }

    if let Some(action) = action {
        app::require_write(&current, action)?;
    }

    Ok(())
}

fn new_profile(patient_id: String, data: &ProfileData) -> app::ProfileState {
    app::ProfileState {
        patient_id,
        full_name: data.full_name.clone(),
        birth_date: data.birth_date.clone(),
        phone_code: data.phone.as_ref().map(|p| p.code().into()),
        phone: data.phone.as_ref().map(|p| p.number().into()),
        identification_basis: data.identification_basis.map(|v| v.as_str().into()),
        completion_status: "draft".into(),
        status: "active".into(),
    }
}

fn profile_state(record: ProfileRecord) -> app::ProfileState {
    app::ProfileState {
        patient_id: record.patient_id,
        full_name: record.full_name,
        birth_date: record.birth_date,
        phone_code: record.phone_code,
        phone: record.phone,
        identification_basis: record.identification_basis,
        completion_status: record.completion_status,
        status: record.status,
    }
}

fn operation_view(
    c: &Connection,
    s: &OperationContext,
    local: &LocalChange,
) -> app::Result<app::OperationView> {
    let result = SqliteOperationStore.operation_result(
        c,
        s,
        &OperationId::try_from(local.operation_id.clone())?,
    )?;
    let mut view = app::OperationView {
        operation_id: local.operation_id.clone(),
        depends_on: serde_json::from_str(&local.depends_on)
            .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?,
        status: "review_required".into(),
        error: None,
    };

    if let Some(result) = result {
        view.status = if result.review_code.is_some() {
            "review_required"
        } else {
            result.status.as_str()
        }
        .into();

        view.error = result
            .result
            .as_ref()
            .and_then(|v| v.get("error"))
            .and_then(|e| e.get("code"))
            .and_then(Value::as_str)
            .map(str::to_owned);
    }

    Ok(view)
}

fn profile_rows(
    c: &Connection,
    s: &OperationContext,
) -> app::Result<Vec<app::EntityRow<app::ProfileState>>> {
    let mut rows = BTreeMap::new();

    for record in read_profiles(c, s).map_err(storage_error)? {
        let version = record.version;
        let state = profile_state(record);
        rows.insert(
            state.patient_id.clone(),
            app::EntityRow {
                accepted: Some(app::Accepted { state, version }),
                provisional: None,
                operations: vec![],
            },
        );
    }

    for local in intents::profile_changes(c, s)? {
        let state: app::ProfileState = serde_json::from_str(&local.draft)
            .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?;
        let accepted_version = rows
            .get(&state.patient_id)
            .and_then(|r| r.accepted.as_ref().map(|a| a.version));
        let row = rows
            .entry(state.patient_id.clone())
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

#[path = "intents.rs"]
mod intents;

#[path = "documents.rs"]
mod documents;

#[path = "outbound_contacts.rs"]
mod contacts;

#[path = "outbound_relationships.rs"]
mod relationships;

struct PreparedChange {
    entity_type: &'static str,
    patient_id: Option<String>,
    identifier_id: Option<String>,
    contact_person_id: Option<String>,
    key: Value,
    draft: Value,
    payload: Value,
    base: intents::EditContext,
}

impl PreparedChange {
    fn profile(
        state: app::ProfileState,
        payload: Value,
        base: intents::EditContext,
    ) -> app::Result<Self> {
        Ok(Self {
            entity_type: "patient_profile",
            patient_id: Some(state.patient_id.clone()),
            identifier_id: None,
            contact_person_id: None,
            key: json!(state.patient_id),
            draft: serde_json::to_value(state)
                .map_err(|_| storage_error(rusqlite::Error::InvalidQuery))?,
            payload,
            base,
        })
    }
}

fn profile_data(state: &app::ProfileState) -> app::Result<ProfileData> {
    Ok(ProfileData {
        full_name: state.full_name.clone(),
        birth_date: state.birth_date.clone(),
        phone: super::super::domain::Phone::optional(
            state.phone_code.clone(),
            state.phone.clone(),
        )?,
        identification_basis: state
            .identification_basis
            .as_deref()
            .map(TryInto::try_into)
            .transpose()?,
    })
}

fn patch_payload(patch: &super::super::domain::ProfilePatch) -> Value {
    use super::super::domain::FieldChange;
    let mut fields = serde_json::Map::new();

    for (name, change) in [
        ("fullName", &patch.full_name),
        ("birthDate", &patch.birth_date),
        ("phoneCode", &patch.phone_code),
        ("phone", &patch.phone),
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

    match patch.identification_basis {
        FieldChange::Keep => {}
        FieldChange::Clear => {
            fields.insert("identificationBasis".into(), Value::Null);
        }
        FieldChange::Set(value) => {
            fields.insert("identificationBasis".into(), json!(value.as_str()));
        }
    }

    Value::Object(fields)
}

fn prepare_change(
    c: &Connection,
    s: &OperationContext,
    mutation: app::PatientMutation,
    ids: &impl IdGenerator,
    now: i64,
) -> app::Result<PreparedChange> {
    if matches!(
        mutation,
        app::PatientMutation::CreateContact { .. }
            | app::PatientMutation::UpdateContact { .. }
            | app::PatientMutation::SetContactStatus { .. }
    ) {
        return contacts::prepare(c, s, mutation, ids, now);
    }

    if matches!(
        mutation,
        app::PatientMutation::CreateRelationship { .. }
            | app::PatientMutation::UpdateRelationship { .. }
            | app::PatientMutation::SetRelationshipStatus { .. }
    ) {
        return relationships::prepare(c, s, mutation, now);
    }

    let (patient, base) = match &mutation {
        app::PatientMutation::Register {
            profile,
            identifiers,
        } => {
            let id = ids.new_id().to_string();
            canonical_uuid_v7(&id)?;
            let state = new_profile(id.clone(), profile);
            let mut payload = json!({"fullName":state.full_name,"birthDate":state.birth_date,"phoneCode":state.phone_code,"phone":state.phone,"identificationBasis":state.identification_basis});

            if !identifiers.is_empty() {
                let documents: Vec<_>=identifiers.iter().map(|d| { let id=ids.new_id().to_string(); canonical_uuid_v7(&id)?; Ok(json!({"id":id,"issuingCountry":d.issuing_country(),"documentType":d.document_type().as_str(),"documentNumber":d.number()})) }).collect::<app::Result<_>>()?;
                payload["identifiers"] = json!(documents);
            }

            return PreparedChange::profile(
                state,
                payload,
                intents::EditContext {
                    version: 0,
                    operation_id: None,
                    depends_on: vec![],
                },
            );
        }
        app::PatientMutation::AddIdentifier { patient, document } => {
            return documents::prepare_add(c, s, patient, document, ids);
        }
        app::PatientMutation::Update { patient, base, .. }
        | app::PatientMutation::SetStatus { patient, base, .. }
        | app::PatientMutation::SetCompletion { patient, base, .. } => (patient, base),
        _ => unreachable!(),
    };
    let row = profile_rows(c, s)?
        .into_iter()
        .find(|r| {
            r.visible()
                .is_some_and(|v| v.patient_id == patient.as_str())
        })
        .ok_or_else(missing)?;
    let mut base = intents::edit_base(&row, base)?;
    let original = row.visible().ok_or_else(missing)?;
    let mut state = original.clone();
    let explicit_completion = matches!(&mutation, app::PatientMutation::SetCompletion { .. });
    let payload = match mutation {
        app::PatientMutation::Update { patch, .. } => {
            let data = patch.apply(&profile_data(original)?)?;
            let profile = new_profile(original.patient_id.clone(), &data);
            state.full_name = profile.full_name;
            state.birth_date = profile.birth_date;
            state.phone_code = profile.phone_code;
            state.phone = profile.phone;
            state.identification_basis = profile.identification_basis;
            patch_payload(&patch)
        }
        app::PatientMutation::SetStatus { status, .. } => {
            state.status = status.as_str().into();
            json!({"status":status.as_str()})
        }
        app::PatientMutation::SetCompletion { completion, .. } => {
            state.completion_status = completion.as_str().into();
            json!({"completionStatus":completion.as_str()})
        }
        app::PatientMutation::Register { .. }
        | app::PatientMutation::AddIdentifier { .. }
        | app::PatientMutation::CreateContact { .. }
        | app::PatientMutation::UpdateContact { .. }
        | app::PatientMutation::SetContactStatus { .. }
        | app::PatientMutation::CreateRelationship { .. }
        | app::PatientMutation::UpdateRelationship { .. }
        | app::PatientMutation::SetRelationshipStatus { .. } => unreachable!(),
    };

    if &state == original {
        return Err(app::PatientsError::new(
            "NO_CHANGES",
            "la ficha no tiene cambios",
        ));
    }

    if state.completion_status == "complete" {
        let (valid, dependencies) = documents::completion_support(c, s, &state, now)?;

        if !valid {
            let code = if explicit_completion {
                "VALIDATION_ERROR"
            } else {
                "COMPLETENESS_CONFLICT"
            };
            let mut error = app::PatientsError::new(
                code,
                "faltan requisitos de la ficha completa; use draft para retirarlos",
            );
            error.affected_patient_ids.push(state.patient_id.clone());

            return Err(error);
        }

        for id in dependencies {
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
    }

    PreparedChange::profile(state, payload, base)
}

impl app::PatientRepository for SqlitePatientRepository<'_> {
    fn identifiers(
        &mut self,
        access: &AccountAccess,
        patient: &PatientId,
    ) -> app::Result<Vec<app::EntityRow<app::IdentifierState>>> {
        let tx = self.connection.transaction().map_err(storage_error)?;
        current_access(&tx, access, None)?;

        if !profile_rows(&tx, &access.context)?.iter().any(|r| {
            r.visible()
                .is_some_and(|v| v.patient_id == patient.as_str())
        }) {
            return Err(missing());
        }

        let rows = documents::rows(&tx, &access.context)?
            .into_iter()
            .filter(|r| {
                r.visible()
                    .is_some_and(|d| d.patient_id == patient.as_str())
            })
            .collect();
        tx.commit().map_err(storage_error)?;

        Ok(rows)
    }

    fn profiles(
        &mut self,
        access: &AccountAccess,
        filter: &app::PatientFilter,
    ) -> app::Result<app::PatientPage> {
        let tx = self.connection.transaction().map_err(storage_error)?;
        current_access(&tx, access, None)?;
        // Merge the scope before paging so local drafts take part in search.
        // Large scopes will need indexed search over accepted and local data.
        let rows = profile_rows(&tx, &access.context)?;
        let identifiers = documents::rows(&tx, &access.context)?;
        let q = filter.q.as_deref().unwrap_or("").to_lowercase();
        let rows: Vec<_> = rows
            .into_iter()
            .filter(|row| {
                row.visible().is_some_and(|state| {
                    filter.status.is_none_or(|v| state.status == v.as_str())
                        && filter
                            .completion_status
                            .is_none_or(|v| state.completion_status == v.as_str())
                        && (q.is_empty()
                            || [&state.full_name, &state.phone_code, &state.phone]
                                .into_iter()
                                .flatten()
                                .any(|v| v.to_lowercase().contains(&q))
                            || identifiers.iter().any(|r| {
                                r.visible().is_some_and(|d| {
                                    d.patient_id == state.patient_id
                                        && d.document_number.to_lowercase().contains(&q)
                                })
                            }))
                })
            })
            .collect();
        let total = rows.len();
        let offset = filter
            .page
            .checked_sub(1)
            .and_then(|p| p.checked_mul(filter.shows))
            .ok_or_else(|| app::PatientsError::new("VALIDATION_ERROR", "página inválida"))?;
        let items = rows.into_iter().skip(offset).take(filter.shows).collect();
        tx.commit().map_err(storage_error)?;

        Ok(app::PatientPage { items, total })
    }

    fn contacts(
        &mut self,
        access: &AccountAccess,
        filter: &app::ContactFilter,
    ) -> app::Result<app::ContactPage> {
        let tx = self.connection.transaction().map_err(storage_error)?;
        current_access(&tx, access, None)?;
        let rows = contacts::rows(&tx, &access.context)?;
        let q = filter.q.as_deref().unwrap_or("").to_lowercase();
        let rows: Vec<_> = rows
            .into_iter()
            .filter(|row| {
                row.visible().is_some_and(|state| {
                    filter.status.is_none_or(|v| state.status == v.as_str())
                        && (q.is_empty()
                            || state.full_name.to_lowercase().contains(&q)
                            || [&state.phone_code, &state.phone, &state.document_number]
                                .into_iter()
                                .flatten()
                                .any(|v| v.to_lowercase().contains(&q)))
                })
            })
            .collect();
        let total = rows.len();
        let offset = filter
            .page
            .checked_sub(1)
            .and_then(|p| p.checked_mul(filter.shows))
            .ok_or_else(|| app::PatientsError::new("VALIDATION_ERROR", "página inválida"))?;
        let items = rows.into_iter().skip(offset).take(filter.shows).collect();
        tx.commit().map_err(storage_error)?;

        Ok(app::ContactPage { items, total })
    }

    fn contact(
        &mut self,
        access: &AccountAccess,
        contact: &super::super::domain::ContactPersonId,
    ) -> app::Result<app::EntityRow<app::ContactState>> {
        let tx = self.connection.transaction().map_err(storage_error)?;
        current_access(&tx, access, None)?;
        let row = contacts::rows(&tx, &access.context)?
            .into_iter()
            .find(|r| {
                r.visible()
                    .is_some_and(|v| v.contact_person_id == contact.as_str())
            })
            .ok_or_else(|| {
                app::PatientsError::new("NOT_FOUND", "contacto no disponible en este ámbito")
            })?;
        tx.commit().map_err(storage_error)?;

        Ok(row)
    }

    fn patient_contacts(
        &mut self,
        access: &AccountAccess,
        patient: &PatientId,
        status: Option<super::super::domain::Status>,
    ) -> app::Result<Vec<app::EntityRow<app::RelationshipView>>> {
        let tx = self.connection.transaction().map_err(storage_error)?;
        current_access(&tx, access, None)?;
        let views: Vec<_> = relationships::views(&tx, &access.context)?
            .into_iter()
            .filter(|r| {
                r.visible().is_some_and(|v| {
                    v.relationship.patient_id == patient.as_str()
                        && status.is_none_or(|s| v.relationship.status == s.as_str())
                })
            })
            .collect();
        tx.commit().map_err(storage_error)?;

        Ok(views)
    }

    fn contact_patients(
        &mut self,
        access: &AccountAccess,
        contact: &super::super::domain::ContactPersonId,
        status: Option<super::super::domain::Status>,
    ) -> app::Result<Vec<app::EntityRow<app::RelationshipView>>> {
        let tx = self.connection.transaction().map_err(storage_error)?;
        current_access(&tx, access, None)?;
        let views: Vec<_> = relationships::views(&tx, &access.context)?
            .into_iter()
            .filter(|r| {
                r.visible().is_some_and(|v| {
                    v.relationship.contact_person_id == contact.as_str()
                        && status.is_none_or(|s| v.relationship.status == s.as_str())
                })
            })
            .collect();
        tx.commit().map_err(storage_error)?;

        Ok(views)
    }

    fn profile(
        &mut self,
        access: &AccountAccess,
        patient: &PatientId,
    ) -> app::Result<app::EntityRow<app::ProfileState>> {
        let tx = self.connection.transaction().map_err(storage_error)?;
        current_access(&tx, access, None)?;
        let row = profile_rows(&tx, &access.context)?
            .into_iter()
            .find(|r| {
                r.visible()
                    .is_some_and(|v| v.patient_id == patient.as_str())
            })
            .ok_or_else(missing)?;
        tx.commit().map_err(storage_error)?;

        Ok(row)
    }

    fn persist(
        &mut self,
        access: &AccountAccess,
        mutation: app::PatientMutation,
        ids: &impl IdGenerator,
        clock: &impl Clock,
    ) -> app::Result<app::LocalMutationResult> {
        app::require_write(access, mutation.action())?;
        mutation.validate()?;
        // Reserve the write lock before checking permissions and preparing the change.
        // https://docs.rs/rusqlite/0.40.2/rusqlite/struct.Connection.html#method.transaction_with_behavior
        let tx = self
            .connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            .map_err(storage_error)?;
        current_access(&tx, access, Some(mutation.action()))?;
        let action = mutation.action();
        let now = clock
            .now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .and_then(|v| i64::try_from(v.as_millis()).ok())
            .ok_or_else(|| app::PatientsError::new("INTERNAL_ERROR", "reloj local inválido"))?;
        let PreparedChange {
            entity_type,
            patient_id,
            identifier_id,
            contact_person_id,
            key,
            draft,
            payload,
            base,
        } = prepare_change(&tx, &access.context, mutation, ids, now)?;
        let dependencies: Vec<String> = base
            .depends_on
            .iter()
            .map(|id| id.as_str().into())
            .collect();
        let operation_id = record_operation(
            &tx,
            &SqliteOperationStore,
            ids,
            clock,
            NewOperation {
                context: access.context.clone(),
                depends_on: base.depends_on.clone(),
                action: action.into(),
                entity_type: entity_type.into(),
                entity_key: key.clone(),
                expected_version: base.version,
                payload: payload.clone(),
                occurred_at_local: None,
            },
        )?
        .to_string();
        insert_local_change(
            &tx,
            &access.context,
            &LocalChange {
                operation_id: operation_id.clone(),
                entity_type: entity_type.into(),
                patient_id,
                identifier_id,
                contact_person_id,
                action: action.into(),
                expected_version: base.version,
                base_operation_id: base.operation_id,
                depends_on: json!(dependencies).to_string(),
                payload: payload.to_string(),
                draft: draft.to_string(),
                created_at: now,
            },
        )
        .map_err(storage_error)?;
        tx.commit().map_err(storage_error)?;

        Ok(app::LocalMutationResult {
            operation_id,
            entity_type: entity_type.into(),
            entity_key: key.clone(),
            expected_version: base.version,
            depends_on: dependencies,
            provisional: draft,
        })
    }
}

impl app::PatientQuery for SqlitePatientRepository<'_> {
    fn accepted_patient(
        &mut self,
        access: &AccountAccess,
        patient: &PatientId,
    ) -> app::Result<Option<app::Accepted<app::ProfileState>>> {
        let tx = self.connection.transaction().map_err(storage_error)?;
        current_access(&tx, access, None)?;
        let accepted = read_profiles(&tx, &access.context)
            .map_err(storage_error)?
            .into_iter()
            .find(|p| p.patient_id == patient.as_str())
            .map(|p| {
                let version = p.version;
                app::Accepted {
                    state: profile_state(p),
                    version,
                }
            });
        tx.commit().map_err(storage_error)?;
        Ok(accepted)
    }

    fn accepted_identifiers(
        &mut self,
        access: &AccountAccess,
        patient: &PatientId,
    ) -> app::Result<Vec<app::Accepted<app::IdentifierState>>> {
        let tx = self.connection.transaction().map_err(storage_error)?;
        current_access(&tx, access, None)?;
        let accepted = read_identifiers(&tx, &access.context)
            .map_err(storage_error)?
            .into_iter()
            .filter(|d| d.patient_id == patient.as_str())
            .map(|p| {
                let version = p.version;
                app::Accepted {
                    state: documents::accepted_state(p),
                    version,
                }
            })
            .collect();
        tx.commit().map_err(storage_error)?;

        Ok(accepted)
    }
}
