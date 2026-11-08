//! Accepted views for downloaded patient states. Uses the caller's transaction;
//! never writes local intentions or performs network work.
//! https://www.sqlite.org/lang_upsert.html (version guards keep newer rows).
use super::super::domain::{
    Document, DocumentType, Phone, RelationshipKind, parse_date, validate_name, validate_validity,
};
use super::outbound::{
    ContactRecord, IdentifierRecord, ProfileRecord, RelationshipRecord, save_contacts,
    save_identifiers, save_profiles, save_relationships,
};
use crate::{
    components::sync::{
        adapters::remote_models::{
            AccountResponseStatus, PatientIdentifierResponseDocumentType,
            PatientIdentifierResponseVerificationStatus, PatientProfileResponseIdentificationBasis,
            ProjectionId, RelationshipResponseRelationshipKind,
            RelationshipResponseRepresentationStatus, RemoteState,
        },
        remote::{EntityType, RemoteChange, RemoteProjector},
    },
    shared_kernel::{NightingaleError, Result, id::OperationContext},
};
use rusqlite::Transaction;

pub struct PatientsProjector;

fn storage_error(_: rusqlite::Error) -> NightingaleError {
    NightingaleError::internal("proyección local de pacientes no disponible")
}

fn id(value: &ProjectionId) -> String {
    value.clone().into()
}

fn status(value: &AccountResponseStatus) -> &'static str {
    match value {
        AccountResponseStatus::Active => "active",
        AccountResponseStatus::Inactive => "inactive",
    }
}

fn key(value: &serde_json::Value) -> Result<String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| NightingaleError::validation("clave remota inválida"))
}

fn require_account(remote: &ProjectionId, s: &OperationContext) -> Result<()> {
    if id(remote) != s.account_id.as_str() {
        return Err(NightingaleError::validation(
            "estado remoto fuera de ámbito",
        ));
    }

    Ok(())
}

fn profile(
    v: &crate::components::sync::adapters::remote_models::PatientProfileResponse,
) -> Result<ProfileRecord> {
    if let Some(name) = &v.full_name {
        validate_name(name)?;
    }

    if let Some(date) = &v.birth_date {
        parse_date(date)?;
    }

    let phone = Phone::optional(v.phone_code.clone(), v.phone.clone())?;
    let basis = v
        .identification_basis
        .as_ref()
        .map(|v| match v {
            PatientProfileResponseIdentificationBasis::OwnDocument => "own_document",
            PatientProfileResponseIdentificationBasis::MinorWithoutDocument => {
                "minor_without_document"
            }
        })
        .map(str::to_owned);

    Ok(ProfileRecord {
        patient_id: id(&v.patient_id),
        full_name: v.full_name.clone(),
        birth_date: v.birth_date.clone(),
        phone_code: phone.as_ref().map(|p| p.code().into()),
        phone: phone.as_ref().map(|p| p.number().into()),
        identification_basis: basis,
        completion_status: match v.completion_status {
            crate::components::sync::adapters::remote_models::PatientProfileResponseCompletionStatus::Draft => "draft",
            crate::components::sync::adapters::remote_models::PatientProfileResponseCompletionStatus::Complete => "complete",
        }
        .into(),
        status: status(&v.status).into(),
        created_at: v.created_at.clone(),
        updated_at: v.updated_at.clone(),
        version: v.version,
    })
}

fn identifier(
    v: &crate::components::sync::adapters::remote_models::PatientIdentifierResponse,
) -> Result<IdentifierRecord> {
    let kind = match v.document_type {
        PatientIdentifierResponseDocumentType::NationalId => DocumentType::NationalId,
        PatientIdentifierResponseDocumentType::Passport => DocumentType::Passport,
    };
    // Reuse domain rules: country code and non-empty number stay intact.
    Document::new(v.issuing_country.clone(), kind, v.document_number.clone())?;

    Ok(IdentifierRecord {
        identifier_id: id(&v.id),
        patient_id: id(&v.patient_id),
        issuing_country: v.issuing_country.clone(),
        document_type: kind.as_str().into(),
        document_number: v.document_number.clone(),
        verification_status: match v.verification_status {
            PatientIdentifierResponseVerificationStatus::Declared => "declared",
            PatientIdentifierResponseVerificationStatus::Verified => "verified",
            PatientIdentifierResponseVerificationStatus::Invalidated => "invalidated",
        }
        .into(),
        created_at: v.created_at.clone(),
        updated_at: v.updated_at.clone(),
        version: v.version,
    })
}

fn contact(
    v: &crate::components::sync::adapters::remote_models::ContactResponse,
) -> Result<ContactRecord> {
    validate_name(&v.full_name)?;
    let phone = Phone::optional(v.phone_code.clone(), v.phone.clone())?;
    let document = Document::optional(
        v.issuing_country.clone(),
        v.document_type.as_ref().map(|v| match v {
            PatientIdentifierResponseDocumentType::NationalId => DocumentType::NationalId,
            PatientIdentifierResponseDocumentType::Passport => DocumentType::Passport,
        }),
        v.document_number.clone(),
    )?;

    Ok(ContactRecord {
        contact_person_id: id(&v.id),
        full_name: v.full_name.clone(),
        phone_code: phone.as_ref().map(|p| p.code().into()),
        phone: phone.as_ref().map(|p| p.number().into()),
        issuing_country: document.as_ref().map(|d| d.issuing_country().into()),
        document_type: document.as_ref().map(|d| d.document_type().as_str().into()),
        document_number: document.as_ref().map(|d| d.number().into()),
        status: status(&v.status).into(),
        created_at: v.created_at.clone(),
        updated_at: v.updated_at.clone(),
        version: v.version,
    })
}

fn relationship(
    v: &crate::components::sync::adapters::remote_models::RelationshipResponse,
) -> Result<RelationshipRecord> {
    validate_validity(&v.valid_from, v.valid_until.as_deref())?;

    Ok(RelationshipRecord {
        patient_id: id(&v.patient_id),
        contact_person_id: id(&v.contact_person_id),
        relationship_kind: match v.relationship_kind {
            RelationshipResponseRelationshipKind::Mother => RelationshipKind::Mother.as_str(),
            RelationshipResponseRelationshipKind::Father => RelationshipKind::Father.as_str(),
            RelationshipResponseRelationshipKind::Guardian => RelationshipKind::Guardian.as_str(),
            RelationshipResponseRelationshipKind::Other => RelationshipKind::Other.as_str(),
        }
        .into(),
        is_responsible_contact: v.is_responsible_contact,
        representation_status: match v.representation_status {
            RelationshipResponseRepresentationStatus::Unverified => "unverified",
            RelationshipResponseRepresentationStatus::Verified => "verified",
        }
        .into(),
        verified_at: v.verified_at.clone(),
        verified_by_user_id: v.verified_by_user_id.as_ref().map(id),
        valid_from: v.valid_from.clone(),
        valid_until: v.valid_until.clone(),
        status: status(&v.status).into(),
        created_at: v.created_at.clone(),
        updated_at: v.updated_at.clone(),
        version: v.version,
    })
}

impl RemoteProjector for PatientsProjector {
    fn reset(&self, tx: &Transaction, s: &OperationContext) -> Result<()> {
        for table in [
            "patient_profile_view",
            "patient_identifier_view",
            "contact_person_view",
            "patient_contact_view",
        ] {
            tx.execute(
                &format!("DELETE FROM {table} WHERE account_id=?1 AND user_id=?2 AND device_id=?3"),
                rusqlite::params![
                    s.account_id.as_str(),
                    s.user_id.as_str(),
                    s.device_id.as_str()
                ],
            )
            .map_err(storage_error)?;
        }

        Ok(())
    }

    fn project(&self, tx: &Transaction, s: &OperationContext, change: &RemoteChange) -> Result<()> {
        match change {
            RemoteChange::Upsert { state, .. } => match state.as_ref() {
                RemoteState::PatientProfile(v) => {
                    require_account(&v.account_id, s)?;
                    save_profiles(tx, s, &profile(v)?).map_err(storage_error)?;
                }
                RemoteState::PatientIdentifier(v) => {
                    save_identifiers(tx, s, &identifier(v)?).map_err(storage_error)?;
                }
                RemoteState::ContactPerson(v) => {
                    require_account(&v.account_id, s)?;
                    save_contacts(tx, s, &contact(v)?).map_err(storage_error)?;
                }
                RemoteState::PatientContact(v) => {
                    require_account(&v.account_id, s)?;
                    save_relationships(tx, s, &relationship(v)?).map_err(storage_error)?;
                }
                _ => {}
            },
            RemoteChange::Remove {
                entity_type,
                key: entity_key,
            } => match entity_type {
                EntityType::PatientProfile => {
                    let patient = key(entity_key)?;
                    tx.execute("DELETE FROM patient_profile_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND patient_id=?4",rusqlite::params![s.account_id.as_str(),s.user_id.as_str(),s.device_id.as_str(),patient]).map_err(storage_error)?;
                }
                EntityType::PatientIdentifier => {
                    let identifier = key(entity_key)?;
                    tx.execute("DELETE FROM patient_identifier_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND identifier_id=?4",rusqlite::params![s.account_id.as_str(),s.user_id.as_str(),s.device_id.as_str(),identifier]).map_err(storage_error)?;
                }
                EntityType::ContactPerson => {
                    let contact = key(entity_key)?;
                    tx.execute("DELETE FROM contact_person_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND contact_person_id=?4",rusqlite::params![s.account_id.as_str(),s.user_id.as_str(),s.device_id.as_str(),contact]).map_err(storage_error)?;
                }
                EntityType::PatientContact => {
                    if entity_key.as_object().is_none_or(|key| key.len() != 2) {
                        return Err(NightingaleError::validation(
                            "clave compuesta remota inválida",
                        ));
                    }
                    let patient = key(&entity_key["patientId"])?;
                    let contact = key(&entity_key["contactPersonId"])?;
                    tx.execute("DELETE FROM patient_contact_view WHERE account_id=?1 AND user_id=?2 AND device_id=?3 AND patient_id=?4 AND contact_person_id=?5",rusqlite::params![s.account_id.as_str(),s.user_id.as_str(),s.device_id.as_str(),patient,contact]).map_err(storage_error)?;
                }
                _ => {}
            },
        }

        Ok(())
    }
}
