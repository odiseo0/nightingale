use crate::components::identity::{
    domain::IdentityContext,
    repository::{AccessStore, SqliteAccessStore},
};
use crate::components::patients::adapters::inbound_contacts::{run as ipc_run, *};
use crate::components::patients::adapters::inbound_profiles::EditBaseRequest;
use crate::components::patients::{
    application::EditBase,
    domain::{
        CompletionStatus, ContactData, ContactPatch, FieldChange, IdentificationBasis, ProfileData,
        RelationshipData, RelationshipKind,
    },
};
use crate::tests::components::patients::test_support::{access, database};
use rusqlite::Connection;
use serde_json::json;

fn select(c: &mut Connection) {
    let access = access();
    let tx = c.transaction().unwrap();
    SqliteAccessStore
        .save_context(
            &tx,
            &IdentityContext {
                issuer: "https://synthetic.example".into(),
                subject: "synthetic".into(),
                user_id: access.context.user_id,
                selected_account_id: Some(access.context.account_id),
            },
        )
        .unwrap();
    tx.commit().unwrap();
}

fn minor(c: &mut Connection) -> String {
    let registered = ipc_run(c, |cases| {
        cases.register_patient(ProfileData {
            full_name: Some("Menor sintético".into()),
            birth_date: Some("2018-02-28".into()),
            phone: None,
            identification_basis: Some(IdentificationBasis::MinorWithoutDocument),
        })
    })
    .unwrap();
    registered.entity_key.as_str().unwrap().to_owned()
}

fn local_base(operation_id: &str) -> EditBaseRequest {
    EditBaseRequest {
        accepted_version: None,
        operation_id: Some(operation_id.into()),
    }
}

#[test]
fn dto_rejects_unknown_fields_bad_values_and_scope_injection() {
    assert!(
        serde_json::from_value::<CreateContactRequest>(json!({
            "fullName":"X","phoneCode":"+58","phone":"412","accountId":"other"
        }))
        .is_err()
    );
    assert!(
        serde_json::from_value::<CreateContactRequest>(json!({"fullName":"X","role":"owner"}))
            .is_err()
    );
    assert!(RelationshipKind::try_from("cousin").is_err());
    assert!(
        serde_json::from_value::<CreateRelationshipRequest>(json!({
            "patientId":"019a1234-0000-7000-8000-000000000001",
            "contactPersonId":"019a1234-0000-7000-8000-000000000002",
            "relationshipKind":"mother",
            "validFrom":"2026-01-01T08:00:00-04:00",
            "verifiedByUserId":"x"
        }))
        .is_err()
    );
    // Representation verification fields never enter through the client.
    assert!(
        serde_json::from_value::<RelationshipPatchRequest>(json!({
            "representationStatus":"verified"
        }))
        .is_err()
    );
    // validUntil accepts null (retire) and a value; the shape is tri-state.
    let patch: RelationshipPatchRequest = serde_json::from_value(json!({
        "validUntil":null,"relationshipKind":"guardian"
    }))
    .unwrap();
    assert!(patch.valid_until.is_some() && patch.valid_until.clone().flatten().is_none());
    assert!(patch.relationship_kind.clone().flatten().is_some());
}

#[test]
fn desktop_entry_runs_shared_contact_and_minor_flows_and_hides_internals() {
    let mut c = database();
    let create = CreateContactRequest {
        full_name: "Responsable compartido".into(),
        phone_code: Some(Some("+58".into())),
        phone: Some(Some("4120000000".into())),
        issuing_country: None,
        document_type: None,
        document_number: None,
    };
    let data: ContactData = create.clone().try_into().unwrap();
    assert_eq!(
        ipc_run(&mut c, |cases| cases.create_contact_person(data))
            .unwrap_err()
            .code,
        "AUTHENTICATION_REQUIRED"
    );
    select(&mut c);

    let created = ipc_run(&mut c, |cases| {
        cases.create_contact_person(create.clone().try_into().unwrap())
    })
    .unwrap();
    let contact = created.entity_key.as_str().unwrap().to_owned();
    let value = serde_json::to_value(&created).unwrap();
    for field in [
        "payload",
        "permissionEpoch",
        "deviceId",
        "accountId",
        "contractVersion",
    ] {
        assert!(value.get(field).is_none(), "{field}");
    }

    let patient = minor(&mut c);
    let linked = ipc_run(&mut c, |cases| {
        cases.create_patient_contact_relationship(
            patient.clone().try_into().unwrap(),
            contact.clone().try_into().unwrap(),
            RelationshipData::new(
                RelationshipKind::Mother,
                true,
                "2026-01-01T08:00:00-04:00".into(),
                None,
            )
            .unwrap(),
        )
    })
    .unwrap();
    assert_eq!(linked.depends_on.len(), 2);

    // Both query directions expose related names without persisting them.
    let by_patient = ipc_run(&mut c, |cases| {
        cases.list_patient_contacts(&patient.clone().try_into().unwrap(), None)
    })
    .unwrap();
    assert_eq!(by_patient.len(), 1);
    let view = by_patient[0].visible().unwrap();
    assert_eq!(view.contact_full_name, "Responsable compartido");
    let by_contact = ipc_run(&mut c, |cases| {
        cases.list_contact_patients(&contact.clone().try_into().unwrap(), None)
    })
    .unwrap();
    assert_eq!(
        by_contact[0]
            .visible()
            .unwrap()
            .patient_full_name
            .as_deref(),
        Some("Menor sintético")
    );
    let serialized = serde_json::to_value(&by_contact[0]).unwrap();
    assert_eq!(
        serialized["provisional"]["state"]["contactFullName"],
        "Responsable compartido"
    );

    // Completeness conflict surfaces through the same command path.
    let completed = ipc_run(&mut c, |cases| {
        cases.set_patient_completion(
            patient.clone().try_into().unwrap(),
            EditBase::LocalOperation {
                operation_id: linked.operation_id.try_into().unwrap(),
            },
            CompletionStatus::Complete,
        )
    });
    assert_eq!(completed.unwrap_err().code, "VERSION_CONFLICT");
    let profile = ipc_run(&mut c, |cases| {
        cases.get_patient(&patient.clone().try_into().unwrap())
    })
    .unwrap();
    let profile_base = EditBase::LocalOperation {
        operation_id: profile
            .provisional
            .unwrap()
            .operation_id
            .try_into()
            .unwrap(),
    };
    ipc_run(&mut c, |cases| {
        cases.set_patient_completion(
            patient.clone().try_into().unwrap(),
            profile_base,
            CompletionStatus::Complete,
        )
    })
    .unwrap();

    let row = ipc_run(&mut c, |cases| {
        cases.get_contact(&contact.clone().try_into().unwrap())
    })
    .unwrap();
    let contact_base = local_base(&row.provisional.unwrap().operation_id);
    let error = ipc_run(&mut c, |cases| {
        cases.update_contact_person(
            contact.clone().try_into().unwrap(),
            contact_base.try_into().unwrap(),
            ContactPatch {
                phone: FieldChange::Clear,
                phone_code: FieldChange::Clear,
                ..ContactPatch::default()
            },
        )
    })
    .unwrap_err();
    assert_eq!(error.code, "COMPLETENESS_CONFLICT");
    assert_eq!(error.affected_patient_ids, vec![patient.clone()]);
}
