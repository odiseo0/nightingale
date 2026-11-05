use super::{application::*, domain::*, test_support::*};
use crate::shared_kernel::{Clock, IdGenerator};

const FROM: &str = "2026-01-01T08:00:00-04:00";

fn complete_minor<R: PatientRepository, I: IdGenerator, C: Clock>(
    cases: &mut PatientUseCases<'_, R, I, C>,
    name: &str,
    contact: &ContactPersonId,
) -> (PatientId, String) {
    let registered = cases
        .register_patient(ProfileData {
            full_name: Some(name.into()),
            birth_date: Some("2018-02-28".into()),
            phone: None,
            identification_basis: Some(IdentificationBasis::MinorWithoutDocument),
        })
        .unwrap();
    let patient: PatientId = registered
        .entity_key
        .as_str()
        .unwrap()
        .to_owned()
        .try_into()
        .unwrap();
    let linked = cases
        .create_patient_contact_relationship(
            patient.clone(),
            contact.clone(),
            RelationshipData::new(RelationshipKind::Mother, true, FROM.into(), None).unwrap(),
        )
        .unwrap();
    let profile = cases.get_patient(&patient).unwrap();
    let base = EditBase::LocalOperation {
        operation_id: profile
            .provisional
            .unwrap()
            .operation_id
            .try_into()
            .unwrap(),
    };
    let completed = cases
        .set_patient_completion(patient.clone(), base, CompletionStatus::Complete)
        .unwrap();
    assert!(completed.depends_on.contains(&linked.operation_id));

    (patient, completed.operation_id)
}

fn contact_with_phone<R: PatientRepository, I: IdGenerator, C: Clock>(
    cases: &mut PatientUseCases<'_, R, I, C>,
    phone: Option<&str>,
) -> ContactPersonId {
    cases
        .create_contact_person(
            ContactData::new(
                "Responsable compartido".into(),
                Some("+58".into()),
                Some(phone.unwrap_or("4120000003").into()),
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

fn local_op(operation_id: &str) -> EditBase {
    EditBase::LocalOperation {
        operation_id: operation_id.to_owned().try_into().unwrap(),
    }
}

fn provisional_operation_id<R: PatientRepository, I: IdGenerator, C: Clock>(
    cases: &mut PatientUseCases<'_, R, I, C>,
    patient: &PatientId,
) -> String {
    cases
        .get_patient(patient)
        .unwrap()
        .provisional
        .unwrap()
        .operation_id
}

#[test]
fn removing_the_only_responsible_phone_conflicts_until_profiles_move_to_draft() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    run(&mut c, &a, &ids, |cases| {
        let contact = contact_with_phone(cases, None);
        let (minor, _) = complete_minor(cases, "Menor ficticio", &contact);

        // Removing the only responsible phone breaks the complete profile.
        let row = cases.get_contact(&contact).unwrap();
        let contact_base = local_op(&row.provisional.unwrap().operation_id);
        let error = cases
            .update_contact_person(
                contact.clone(),
                contact_base.clone(),
                ContactPatch {
                    phone: FieldChange::Clear,
                    phone_code: FieldChange::Clear,
                    ..ContactPatch::default()
                },
            )
            .unwrap_err();
        assert_eq!(error.code, "COMPLETENESS_CONFLICT");
        assert_eq!(error.affected_patient_ids, vec![minor.as_str().to_owned()]);
        let row = cases.get_contact(&contact).unwrap();
        assert_eq!(row.operations.len(), 1, "the conflict saved nothing");

        // Explicitly moving the profile to draft unblocks the same edit and
        // the contact operation depends on that completion operation.
        let profile_base = local_op(&provisional_operation_id(cases, &minor));
        let draft = cases
            .set_patient_completion(minor.clone(), profile_base, CompletionStatus::Draft)
            .unwrap();
        let updated = cases
            .update_contact_person(
                contact.clone(),
                contact_base,
                ContactPatch {
                    phone: FieldChange::Clear,
                    phone_code: FieldChange::Clear,
                    ..ContactPatch::default()
                },
            )
            .unwrap();
        assert!(updated.depends_on.contains(&draft.operation_id));
        let row = cases.get_contact(&contact).unwrap();
        assert!(row.visible().unwrap().phone.is_none());
        assert_eq!(row.provisional.unwrap().expected_version, 1);
    });
}

#[test]
fn shared_contact_conflict_lists_every_affected_complete_profile() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    run(&mut c, &a, &ids, |cases| {
        let contact = contact_with_phone(cases, None);
        let (first, _) = complete_minor(cases, "Menor uno", &contact);
        let (second, _) = complete_minor(cases, "Menor dos", &contact);

        let row = cases.get_contact(&contact).unwrap();
        let error = cases
            .set_contact_person_status(
                contact.clone(),
                local_op(&row.provisional.unwrap().operation_id),
                Status::Inactive,
            )
            .unwrap_err();
        assert_eq!(error.code, "COMPLETENESS_CONFLICT");
        let mut affected = error.affected_patient_ids.clone();
        affected.sort();
        let mut expected = vec![first.as_str().to_owned(), second.as_str().to_owned()];
        expected.sort();
        assert_eq!(affected, expected);

        // Deactivating a different, unlinked contact is not blocked.
        let other = contact_with_phone(cases, Some("4120000009"));
        let row = cases.get_contact(&other).unwrap();
        let done = cases
            .set_contact_person_status(
                other,
                local_op(&row.provisional.unwrap().operation_id),
                Status::Inactive,
            )
            .unwrap();
        assert_eq!(done.depends_on.len(), 1);
    });
}

#[test]
fn relationship_changes_protect_complete_profiles_and_drafts_unblock_them() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    run(&mut c, &a, &ids, |cases| {
        let contact = contact_with_phone(cases, None);
        let (minor, _) = complete_minor(cases, "Menor ficticio", &contact);

        // Ending the responsible link breaks the only support of the profile.
        let links = cases.list_patient_contacts(&minor, None).unwrap();
        let link_op = links[0].provisional.as_ref().unwrap().operation_id.clone();
        let error = cases
            .set_patient_contact_relationship_status(
                minor.clone(),
                contact.clone(),
                local_op(&link_op),
                Status::Inactive,
            )
            .unwrap_err();
        assert_eq!(error.code, "COMPLETENESS_CONFLICT");
        assert_eq!(error.affected_patient_ids, vec![minor.as_str().to_owned()]);

        // An adult with its own phone and document is never affected.
        let adult = cases
            .register_patient_with_identifiers(
                ProfileData {
                    full_name: Some("Adulto ficticio".into()),
                    birth_date: Some("1990-05-05".into()),
                    phone: Some(
                        crate::components::patients::domain::Phone::new(
                            "+58".into(),
                            "4140000000".into(),
                        )
                        .unwrap(),
                    ),
                    identification_basis: Some(IdentificationBasis::OwnDocument),
                },
                vec![Document::new("VE".into(), DocumentType::NationalId, "V-1".into()).unwrap()],
            )
            .unwrap();
        let adult: PatientId = adult
            .entity_key
            .as_str()
            .unwrap()
            .to_owned()
            .try_into()
            .unwrap();
        let profile = cases.get_patient(&adult).unwrap();
        let base = local_op(&profile.provisional.unwrap().operation_id);
        cases
            .set_patient_completion(adult.clone(), base, CompletionStatus::Complete)
            .unwrap();

        // Move the minor to draft; the same relationship change now succeeds
        // and carries the completion operation as a dependency.
        let profile_base = local_op(&provisional_operation_id(cases, &minor));
        let draft = cases
            .set_patient_completion(minor.clone(), profile_base, CompletionStatus::Draft)
            .unwrap();
        let done = cases
            .set_patient_contact_relationship_status(
                minor.clone(),
                contact.clone(),
                local_op(&link_op),
                Status::Inactive,
            )
            .unwrap();
        assert!(done.depends_on.contains(&draft.operation_id));
    });
}

#[test]
fn dependency_limit_rejects_overloaded_shared_contact_edits_without_loss() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    run(&mut c, &a, &ids, |cases| {
        let contact = contact_with_phone(cases, None);
        let mut minors = vec![];
        for n in 0..8 {
            let (patient, _) = complete_minor(cases, &format!("Menor {n}"), &contact);
            minors.push(patient);
        }

        // Eight explicit draft moves overload the contact edit: its own base
        // operation plus eight completion dependencies exceed the contract limit.
        for patient in &minors {
            let base = local_op(&provisional_operation_id(cases, patient));
            cases
                .set_patient_completion(patient.clone(), base, CompletionStatus::Draft)
                .unwrap();
        }
        let row = cases.get_contact(&contact).unwrap();
        let before = row.operations.len();
        let error = cases
            .update_contact_person(
                contact.clone(),
                local_op(&row.provisional.unwrap().operation_id),
                ContactPatch {
                    phone: FieldChange::Clear,
                    phone_code: FieldChange::Clear,
                    ..ContactPatch::default()
                },
            )
            .unwrap_err();
        assert_eq!(error.code, "VALIDATION_ERROR");
        let row = cases.get_contact(&contact).unwrap();
        assert_eq!(row.operations.len(), before, "no step was lost");
    });
}

#[test]
fn a_minor_with_two_responsibles_only_conflicts_when_the_last_phone_is_removed() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    run(&mut c, &a, &ids, |cases| {
        let mother = contact_with_phone(cases, Some("4120000001"));
        let father = contact_with_phone(cases, Some("4120000002"));
        let registered = cases
            .register_patient(ProfileData {
                full_name: Some("Menor con dos responsables".into()),
                birth_date: Some("2018-02-28".into()),
                phone: None,
                identification_basis: Some(IdentificationBasis::MinorWithoutDocument),
            })
            .unwrap();
        let minor: PatientId = registered
            .entity_key
            .as_str()
            .unwrap()
            .to_owned()
            .try_into()
            .unwrap();
        cases
            .create_patient_contact_relationship(
                minor.clone(),
                mother.clone(),
                RelationshipData::new(RelationshipKind::Mother, true, FROM.into(), None).unwrap(),
            )
            .unwrap();
        cases
            .create_patient_contact_relationship(
                minor.clone(),
                father.clone(),
                RelationshipData::new(RelationshipKind::Father, true, FROM.into(), None).unwrap(),
            )
            .unwrap();
        let profile = cases.get_patient(&minor).unwrap();
        let base = EditBase::LocalOperation {
            operation_id: profile
                .provisional
                .unwrap()
                .operation_id
                .try_into()
                .unwrap(),
        };
        cases
            .set_patient_completion(minor.clone(), base, CompletionStatus::Complete)
            .unwrap();

        // Losing one responsible phone leaves the other support: no conflict.
        let row = cases.get_contact(&father).unwrap();
        cases
            .update_contact_person(
                father.clone(),
                local_op(&row.provisional.unwrap().operation_id),
                ContactPatch {
                    phone: FieldChange::Clear,
                    phone_code: FieldChange::Clear,
                    ..ContactPatch::default()
                },
            )
            .unwrap();

        // Losing the last responsible phone conflicts and saves nothing.
        let without_phone = ContactPatch {
            phone: FieldChange::Clear,
            phone_code: FieldChange::Clear,
            ..ContactPatch::default()
        };
        let row = cases.get_contact(&mother).unwrap();
        let error = cases
            .update_contact_person(
                mother.clone(),
                local_op(&row.provisional.unwrap().operation_id),
                without_phone,
            )
            .unwrap_err();
        assert_eq!(error.code, "COMPLETENESS_CONFLICT");
        assert_eq!(error.affected_patient_ids, vec![minor.as_str().to_owned()]);
        let row = cases.get_contact(&mother).unwrap();
        assert_eq!(row.operations.len(), 1, "the conflict saved nothing");
    });
}

#[test]
fn a_rejected_responsible_link_neither_holds_nor_triggers_the_impact() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    let (contact, link_op) = run(&mut c, &a, &ids, |cases| {
        let contact = contact_with_phone(cases, None);
        let (minor, _) = complete_minor(cases, "Menor ficticio", &contact);
        let links = cases.list_patient_contacts(&minor, None).unwrap();
        let link_op = links[0].provisional.as_ref().unwrap().operation_id.clone();
        (contact, link_op)
    });
    c.execute(
        "UPDATE sync_outbox SET status='rejected' WHERE operation_id=?1",
        rusqlite::params![link_op],
    )
    .unwrap();

    // A rejected result makes the link unable to sustain completion, so
    // breaking it needs no completeness protection either.
    run(&mut c, &a, &ids, |cases| {
        let row = cases.get_contact(&contact).unwrap();
        let done = cases
            .update_contact_person(
                contact.clone(),
                local_op(&row.provisional.unwrap().operation_id),
                ContactPatch {
                    phone: FieldChange::Clear,
                    phone_code: FieldChange::Clear,
                    ..ContactPatch::default()
                },
            )
            .unwrap();
        assert_eq!(
            done.depends_on.len(),
            1,
            "no draft move exists to depend on"
        );
        let row = cases.get_contact(&contact).unwrap();
        assert!(row.visible().unwrap().phone.is_none());
    });
}

#[test]
fn a_profile_edit_after_the_draft_move_keeps_the_completion_dependency() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    run(&mut c, &a, &ids, |cases| {
        let contact = contact_with_phone(cases, None);
        let (minor, _) = complete_minor(cases, "Menor ficticio", &contact);

        // A draft move followed by an ordinary edit: the latest local
        // operation of the profile is no longer the completion one.
        let profile_base = local_op(&provisional_operation_id(cases, &minor));
        let draft = cases
            .set_patient_completion(minor.clone(), profile_base, CompletionStatus::Draft)
            .unwrap();
        let profile_base = local_op(&provisional_operation_id(cases, &minor));
        let edit = cases
            .update_patient_profile(
                minor.clone(),
                profile_base,
                ProfilePatch {
                    full_name: FieldChange::Set("Menor renombrado".into()),
                    ..ProfilePatch::default()
                },
            )
            .unwrap();
        assert_ne!(edit.operation_id, draft.operation_id);

        // The breaking contact edit still depends on the explicit draft move.
        let without_phone = ContactPatch {
            phone: FieldChange::Clear,
            phone_code: FieldChange::Clear,
            ..ContactPatch::default()
        };
        let row = cases.get_contact(&contact).unwrap();
        let done = cases
            .update_contact_person(
                contact.clone(),
                local_op(&row.provisional.unwrap().operation_id),
                without_phone.clone(),
            )
            .unwrap();
        assert!(done.depends_on.contains(&draft.operation_id));

        // Restoring the phone and completing again brings the conflict back.
        let row = cases.get_contact(&contact).unwrap();
        cases
            .update_contact_person(
                contact.clone(),
                local_op(&row.provisional.unwrap().operation_id),
                ContactPatch {
                    phone_code: FieldChange::Set("+58".into()),
                    phone: FieldChange::Set("4120000003".into()),
                    ..ContactPatch::default()
                },
            )
            .unwrap();
        let profile_base = local_op(&provisional_operation_id(cases, &minor));
        cases
            .set_patient_completion(minor.clone(), profile_base, CompletionStatus::Complete)
            .unwrap();
        let row = cases.get_contact(&contact).unwrap();
        let error = cases
            .update_contact_person(
                contact.clone(),
                local_op(&row.provisional.unwrap().operation_id),
                without_phone,
            )
            .unwrap_err();
        assert_eq!(error.code, "COMPLETENESS_CONFLICT");
        assert_eq!(error.affected_patient_ids, vec![minor.as_str().to_owned()]);
    });
}
