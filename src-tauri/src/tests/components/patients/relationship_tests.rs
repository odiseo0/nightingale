use super::test_support::*;
use crate::components::patients::{
    adapters::outbound::SqlitePatientRepository, application::*, domain::*,
};
use crate::components::sync::{
    adapters::outbound::SqliteOperationStore, application::OperationStore,
};

const FROM: &str = "2026-01-01T08:00:00-04:00";

fn minor(
    cases: &mut PatientUseCases<'_, SqlitePatientRepository<'_>, Ids, FixedClock>,
) -> PatientId {
    cases
        .register_patient(ProfileData {
            full_name: Some("Menor ficticio".into()),
            birth_date: Some("2018-02-28".into()),
            phone: None,
            identification_basis: Some(IdentificationBasis::MinorWithoutDocument),
        })
        .unwrap()
        .entity_key
        .as_str()
        .unwrap()
        .to_owned()
        .try_into()
        .unwrap()
}

fn contact(
    cases: &mut PatientUseCases<'_, SqlitePatientRepository<'_>, Ids, FixedClock>,
) -> ContactPersonId {
    cases
        .create_contact_person(
            ContactData::new(
                "Responsable ficticio".into(),
                Some("+58".into()),
                Some("4120000002".into()),
                None,
                None,
                None,
            )
            .unwrap(),
        )
        .unwrap()
        .entity_key
        .as_str()
        .unwrap()
        .to_owned()
        .try_into()
        .unwrap()
}

fn relation_data(kind: RelationshipKind) -> RelationshipData {
    RelationshipData::new(kind, true, FROM.into(), None).unwrap()
}

#[test]
fn shared_contact_links_two_patients_and_queries_work_both_directions() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    run(&mut c, &a, &ids, |cases| {
        let minor = minor(cases);
        let adult = cases
            .register_patient(ProfileData {
                full_name: Some("Paciente ficticio".into()),
                ..ProfileData::default()
            })
            .unwrap();
        let adult = patient_id(&adult);
        let responsible = contact(cases);

        let first = cases
            .create_patient_contact_relationship(
                minor.clone(),
                responsible.clone(),
                relation_data(RelationshipKind::Mother),
            )
            .unwrap();
        assert_eq!(first.expected_version, 0);
        assert_eq!(
            first.depends_on.len(),
            2,
            "local patient and contact creations become dependencies"
        );

        let second = cases
            .create_patient_contact_relationship(
                adult.clone(),
                responsible.clone(),
                relation_data(RelationshipKind::Guardian),
            )
            .unwrap();
        assert_eq!(second.depends_on.len(), 2);

        // Both directions see the same shared contact identity.
        let by_patient = cases.list_patient_contacts(&minor, None).unwrap();
        assert_eq!(by_patient.len(), 1);
        let view = by_patient[0].visible().unwrap();
        assert_eq!(view.contact_full_name, "Responsable ficticio");
        assert_eq!(view.contact_phone.as_deref(), Some("4120000002"));
        assert_eq!(view.patient_full_name.as_deref(), Some("Menor ficticio"));
        assert_eq!(view.relationship.relationship_kind, "mother");

        let by_contact = cases.list_contact_patients(&responsible, None).unwrap();
        assert_eq!(by_contact.len(), 2);
        let patients: Vec<_> = by_contact
            .iter()
            .map(|r| r.visible().unwrap().relationship.patient_id.clone())
            .collect();
        assert!(patients.contains(&minor.as_str().to_owned()));
        assert!(patients.contains(&adult.as_str().to_owned()));

        assert!(
            cases
                .list_patient_contacts(&minor, Some(Status::Inactive))
                .unwrap()
                .is_empty()
        );
        first.entity_key.clone()
    });

    // The join data is read-side: the relationship payload keeps no names.
    let local = SqliteOperationStore.list_pending(&c, 10).unwrap();
    let created = local
        .iter()
        .find(|o| o.action == "CreatePatientContactRelationship")
        .unwrap();
    assert!(!created.payload.to_string().contains("Responsable ficticio"));
}

#[test]
fn relationship_requires_existing_ends_and_rejects_duplicate_links() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    run(&mut c, &a, &ids, |cases| {
        let minor = minor(cases);
        let responsible = contact(cases);
        let ghost: ContactPersonId = "019a1234-0000-7000-8000-0000000000ff"
            .to_owned()
            .try_into()
            .unwrap();

        assert_eq!(
            cases
                .create_patient_contact_relationship(
                    minor.clone(),
                    ghost,
                    relation_data(RelationshipKind::Mother)
                )
                .unwrap_err()
                .code,
            "NOT_FOUND"
        );

        cases
            .create_patient_contact_relationship(
                minor.clone(),
                responsible.clone(),
                relation_data(RelationshipKind::Mother),
            )
            .unwrap();
        assert_eq!(
            cases
                .create_patient_contact_relationship(
                    minor.clone(),
                    responsible.clone(),
                    relation_data(RelationshipKind::Father)
                )
                .unwrap_err()
                .code,
            "VALIDATION_ERROR"
        );
    });
}

#[test]
fn successive_relationship_edits_and_activity_keep_bases_and_validity() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    run(&mut c, &a, &ids, |cases| {
        let minor = minor(cases);
        let responsible = contact(cases);
        let created = cases
            .create_patient_contact_relationship(
                minor.clone(),
                responsible.clone(),
                relation_data(RelationshipKind::Other),
            )
            .unwrap();
        let base = EditBase::LocalOperation {
            operation_id: created.operation_id.clone().try_into().unwrap(),
        };

        let updated = cases
            .update_patient_contact_relationship(
                minor.clone(),
                responsible.clone(),
                base,
                RelationshipPatch {
                    relationship_kind: FieldChange::Set(RelationshipKind::Guardian),
                    valid_until: FieldChange::Set("2030-01-01T00:00:00-04:00".into()),
                    ..RelationshipPatch::default()
                },
            )
            .unwrap();
        assert_eq!(updated.expected_version, 1);
        assert_eq!(updated.depends_on, vec![created.operation_id.clone()]);

        let status_base = EditBase::LocalOperation {
            operation_id: updated.operation_id.clone().try_into().unwrap(),
        };
        let inactive = cases
            .set_patient_contact_relationship_status(
                minor.clone(),
                responsible.clone(),
                status_base,
                Status::Inactive,
            )
            .unwrap();
        assert_eq!(inactive.expected_version, 2);

        let rows = cases.list_patient_contacts(&minor, None).unwrap();
        let relationship = &rows[0];
        assert_eq!(
            relationship
                .provisional
                .as_ref()
                .unwrap()
                .state
                .relationship
                .status,
            "inactive"
        );
        assert_eq!(relationship.operations.len(), 3);
        let visible = &relationship.visible().unwrap().relationship;
        assert_eq!(visible.relationship_kind, "guardian");
        assert_eq!(
            visible.valid_until.as_deref(),
            Some("2030-01-01T00:00:00-04:00")
        );

        // Without changes or with invalid validity the case creates no operation.
        let last_base = EditBase::LocalOperation {
            operation_id: inactive.operation_id.clone().try_into().unwrap(),
        };
        assert_eq!(
            cases
                .update_patient_contact_relationship(
                    minor.clone(),
                    responsible.clone(),
                    last_base.clone(),
                    RelationshipPatch {
                        valid_until: FieldChange::Clear,
                        ..RelationshipPatch::default()
                    }
                )
                .unwrap()
                .expected_version,
            3
        );
    });
}

#[test]
fn minor_completes_offline_with_local_responsible_and_keeps_dependencies() {
    let db = Database::new();
    let a = access();
    let ids = Ids::default();
    let mut c = crate::platform::sqlite::open(&db.0).unwrap();
    seed_access(&mut c, &a);
    run(&mut c, &a, &ids, |cases| {
        let minor = minor(cases);
        let responsible = contact(cases);
        let linked = cases
            .create_patient_contact_relationship(
                minor.clone(),
                responsible.clone(),
                relation_data(RelationshipKind::Mother),
            )
            .unwrap();

        // The profile's last local operation is the relationship creation.
        let profile = cases.get_patient(&minor).unwrap();
        let profile_base = EditBase::LocalOperation {
            operation_id: profile
                .provisional
                .unwrap()
                .operation_id
                .try_into()
                .unwrap(),
        };
        let completed = cases
            .set_patient_completion(minor.clone(), profile_base, CompletionStatus::Complete)
            .unwrap();
        assert!(
            completed.depends_on.contains(&linked.operation_id),
            "completion depends on the local responsible link: {:?}",
            completed.depends_on
        );
        assert_eq!(completed.provisional["completionStatus"], "complete");
    });
    drop(c);

    let mut c = crate::platform::sqlite::open(&db.0).unwrap();
    run(&mut c, &a, &ids, |cases| {
        let minor = cases
            .list_patients(&PatientFilter {
                q: Some("Menor".into()),
                completion_status: Some(CompletionStatus::Complete),
                ..PatientFilter::default()
            })
            .unwrap();
        assert_eq!(minor.total, 1);
    });
}

fn patient_id(result: &LocalMutationResult) -> PatientId {
    result
        .entity_key
        .as_str()
        .unwrap()
        .to_owned()
        .try_into()
        .unwrap()
}
