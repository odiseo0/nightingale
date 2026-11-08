use crate::components::identity::{
    domain::IdentityContext,
    repository::{AccessStore, SqliteAccessStore},
};
use crate::components::patients::adapters::inbound_profiles::run as ipc_run;
use crate::components::patients::adapters::inbound_profiles::*;
use crate::components::patients::{
    application::{EditBase, Result},
    domain::{CompletionStatus, Document, DocumentType, ProfilePatch},
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

#[test]
fn dto_rejects_unknown_fields_scope_injection_and_ambiguous_bases() {
    assert!(
        serde_json::from_value::<RegisterPatientRequest>(json!({
            "fullName":"Synthetic","accountId":"other"
        }))
        .is_err()
    );
    assert!(
        serde_json::from_value::<RegisterPatientRequest>(json!({
            "fullName":"Synthetic","sql":"DROP TABLE"
        }))
        .is_err()
    );
    // Present null is a clear, absence is a keep, and both must be expressible.
    let patch: PatientPatchRequest =
        serde_json::from_value(json!({"fullName":null,"birthDate":"1990-05-05"})).unwrap();
    assert!(patch.full_name.is_some() && patch.full_name.clone().flatten().is_none());
    assert!(patch.birth_date.clone().flatten().is_some());
    assert!(patch.phone_code.is_none());
    assert!(serde_json::from_value::<PatientPatchRequest>(json!({"birthDate":1.5})).is_err());

    for base in [
        json!({}),
        json!({"acceptedVersion":1,"operationId":"019a1234-0000-7000-8000-000000000001"}),
        json!({"acceptedVersion":0}),
    ] {
        let request = json!({"patientId":"019a1234-0000-7000-8000-000000000001","base":base,"patch":{"fullName":"X"}});
        let parsed: UpdatePatientProfileRequest =
            serde_json::from_value(request.clone()).unwrap_or_else(|_| panic!("{base}"));
        assert!(
            TryInto::<EditBase>::try_into(parsed.base).is_err(),
            "{base}"
        );
    }

    assert!(
        serde_json::from_value::<IdentifierInputRequest>(json!({
            "documentType":"national_id","documentNumber":"V-1"
        }))
        .is_err(),
        "identifiers require their id"
    );
}

#[test]
fn desktop_entry_requires_selection_and_writes_hide_internal_envelope() {
    let mut c = database();
    let request = RegisterPatientRequest {
        full_name: Some(Some("Paciente sintético".into())),
        birth_date: None,
        phone_code: Some(Some("+58".into())),
        phone: Some(Some("4140000000".into())),
        identification_basis: None,
        identifiers: Some(vec![IdentifierInputRequest {
            id: "019a1234-0000-7000-8000-000000000050".into(),
            issuing_country: None,
            document_type: "national_id".into(),
            document_number: "V-00.012-3".into(),
        }]),
    };
    assert_eq!(
        ipc_run(&mut c, |cases| {
            cases.register_patient_with_identifiers(
                data(&request).unwrap(),
                request
                    .identifiers
                    .clone()
                    .unwrap()
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_>>()
                    .unwrap(),
            )
        })
        .unwrap_err()
        .code,
        "AUTHENTICATION_REQUIRED"
    );
    select(&mut c);

    let registered = ipc_run(&mut c, |cases| {
        cases.register_patient_with_identifiers(
            data(&request).unwrap(),
            request
                .identifiers
                .unwrap()
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_>>()
                .unwrap(),
        )
    })
    .unwrap();
    let value = serde_json::to_value(&registered).unwrap();
    assert_eq!(value["operationId"], registered.operation_id);
    assert_eq!(value["provisional"]["status"], "active");
    for field in [
        "payload",
        "permissionEpoch",
        "contractVersion",
        "semanticFingerprint",
        "deviceId",
        "accountId",
    ] {
        assert!(value.get(field).is_none(), "{field}");
    }
    let patient = registered.entity_key.as_str().unwrap().to_owned();

    let page = ipc_run(&mut c, |cases| {
        cases.list_patients(&patient_filter(ListPatientsRequest {
            q: Some("sintético".into()),
            status: None,
            completion_status: None,
            page: None,
            shows: None,
        })?)
    })
    .unwrap();
    assert_eq!(page.total, 1);
    assert!(page.items[0].accepted.is_none());

    let identifiers = ipc_run(&mut c, |cases| {
        cases.list_patient_identifiers(&patient.clone().try_into().unwrap())
    })
    .unwrap();
    assert_eq!(identifiers.len(), 1);
    assert_eq!(
        identifiers[0].visible().unwrap().document_number,
        "V-00.012-3"
    );

    // A present null clears the phone pair; an absent field keeps it.
    let patch: ProfilePatch = PatientPatchRequest {
        full_name: None,
        birth_date: Some(Some("1990-05-05".into())),
        phone_code: Some(None),
        phone: Some(None),
        identification_basis: None,
    }
    .try_into()
    .unwrap();
    let updated = ipc_run(&mut c, |cases| {
        cases.update_patient_profile(
            patient.clone().try_into().unwrap(),
            EditBase::LocalOperation {
                operation_id: registered.operation_id.clone().try_into().unwrap(),
            },
            patch,
        )
    })
    .unwrap();
    let state = updated.provisional["fullName"].clone();
    assert_eq!(updated.provisional["phone"], json!(null));
    let row = ipc_run(&mut c, |cases| {
        cases.get_patient(&patient.clone().try_into().unwrap())
    })
    .unwrap();
    assert_eq!(
        row.visible().unwrap().full_name.as_deref(),
        Some("Paciente sintético")
    );
    assert!(row.visible().unwrap().phone.is_none());
    let _ = state;

    // Completion from an accepted base is rejected for a purely local profile.
    assert_eq!(
        ipc_run(&mut c, |cases| {
            cases.set_patient_completion(
                patient.clone().try_into().unwrap(),
                EditBase::Accepted { version: 1 },
                CompletionStatus::Complete,
            )
        })
        .unwrap_err()
        .code,
        "VERSION_CONFLICT"
    );

    let added = ipc_run(&mut c, |cases| {
        cases.add_patient_identifier(
            patient.clone().try_into().unwrap(),
            Document::new("VE".into(), DocumentType::Passport, "P-9".into()).unwrap(),
        )
    })
    .unwrap();
    assert_eq!(added.entity_type, "patient_identifier");
}
