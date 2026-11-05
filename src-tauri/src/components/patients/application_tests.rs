use super::{adapters::outbound::*, application::*, domain::*, test_support::*};
use crate::{
    components::sync::{adapters::outbound::SqliteOperationStore, application::OperationStore},
    platform::sqlite,
};

fn data(name: &str) -> ProfileData {
    ProfileData {
        full_name: Some(name.into()),
        ..ProfileData::default()
    }
}
fn patient(result: &LocalMutationResult) -> PatientId {
    result
        .entity_key
        .as_str()
        .unwrap()
        .to_owned()
        .try_into()
        .unwrap()
}

#[test]
fn register_search_read_and_reopen_work_without_http() {
    let db = Database::new();
    let a = access();
    let ids = Ids::default();
    let mut c = sqlite::open(&db.0).unwrap();
    seed_access(&mut c, &a);
    let result = run(&mut c, &a, &ids, |cases| {
        cases.register_patient(data("Paciente ficticio")).unwrap()
    });
    let pending = SqliteOperationStore.list_pending(&c, 10).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].action, "RegisterPatient");
    assert_eq!(pending[0].entity_key, result.entity_key);
    assert_eq!(pending[0].expected_version, 0);
    drop(c);
    let mut c = sqlite::open(&db.0).unwrap();
    run(&mut c, &a, &ids, |cases| {
        let page = cases
            .list_patients(&PatientFilter {
                q: Some("FICTICIO".into()),
                ..PatientFilter::default()
            })
            .unwrap();
        assert_eq!(page.total, 1);
        let row = cases.get_patient(&patient(&result)).unwrap();
        assert!(row.accepted.is_none());
        assert_eq!(
            row.provisional.unwrap().state.full_name.as_deref(),
            Some("Paciente ficticio")
        );
        assert_eq!(row.operations[0].status, "pending");
    });
}

#[test]
fn empty_draft_and_matching_names_create_distinct_identities() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    run(&mut c, &a, &ids, |cases| {
        let empty = cases.register_patient(ProfileData::default()).unwrap();
        assert_eq!(empty.provisional["completionStatus"], "draft");
        assert_eq!(empty.provisional["status"], "active");
        let one = cases
            .register_patient(data("Nombre ficticio repetido"))
            .unwrap();
        let two = cases
            .register_patient(data("Nombre ficticio repetido"))
            .unwrap();
        assert_ne!(one.entity_key, two.entity_key);
        let page = cases
            .list_patients(&PatientFilter {
                q: Some("repetido".into()),
                shows: 1,
                ..PatientFilter::default()
            })
            .unwrap();
        assert_eq!(page.total, 2);
        assert_eq!(page.items.len(), 1);
        assert!(
            cases
                .list_patients(&PatientFilter {
                    page: 9,
                    ..PatientFilter::default()
                })
                .unwrap()
                .items
                .is_empty()
        );
        assert_eq!(
            cases
                .list_patients(&PatientFilter {
                    completion_status: Some(CompletionStatus::Complete),
                    ..PatientFilter::default()
                })
                .unwrap()
                .total,
            0
        );
        assert!(
            cases
                .list_patients(&PatientFilter {
                    page: 0,
                    ..PatientFilter::default()
                })
                .is_err()
        );
    });
}

#[test]
fn accepted_and_provisional_states_remain_separate_and_scoped() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    let registered = run(&mut c, &a, &ids, |cases| {
        cases.register_patient(data("Local ficticio")).unwrap()
    });
    let tx = c.transaction().unwrap();
    save_profiles(
        &tx,
        &a.context,
        &ProfileRecord {
            patient_id: patient(&registered).as_str().into(),
            full_name: Some("Aceptado ficticio".into()),
            birth_date: None,
            phone_code: None,
            phone: None,
            identification_basis: None,
            completion_status: "draft".into(),
            status: "active".into(),
            created_at: None,
            updated_at: None,
            version: 1,
        },
    )
    .unwrap();
    tx.commit().unwrap();
    let row = run(&mut c, &a, &ids, |cases| {
        cases.get_patient(&patient(&registered)).unwrap()
    });
    assert_eq!(
        row.accepted.unwrap().state.full_name.as_deref(),
        Some("Aceptado ficticio")
    );
    assert_eq!(
        row.provisional.unwrap().state.full_name.as_deref(),
        Some("Local ficticio")
    );
    for field in ["accountId", "userId", "deviceId"] {
        let mut other = a.clone();
        let mut value = serde_json::to_value(&a.context).unwrap();
        value[field] = serde_json::json!("019a1234-0000-7000-8000-000000000099");
        other.context = serde_json::from_value(value).unwrap();
        seed_access(&mut c, &other);
        run(&mut c, &other, &ids, |cases| {
            assert_eq!(
                cases
                    .list_patients(&PatientFilter::default())
                    .unwrap()
                    .total,
                0
            );
            assert_eq!(
                cases.get_patient(&patient(&registered)).unwrap_err().code,
                "NOT_FOUND"
            );
        });
    }
}

#[test]
fn cached_permission_changes_and_epochs_block_stale_callers() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    let mut revoked = a.clone();
    revoked.allowed_actions.clear();
    seed_access(&mut c, &revoked);
    assert_eq!(
        run(&mut c, &a, &ids, |cases| cases
            .register_patient(ProfileData::default())
            .unwrap_err())
        .code,
        "PERMISSION_DENIED"
    );
    let mut epoch = a.clone();
    epoch.context.permission_epoch = 2;
    seed_access(&mut c, &epoch);
    assert_eq!(
        run(&mut c, &a, &ids, |cases| cases
            .list_patients(&PatientFilter::default())
            .unwrap_err())
        .code,
        "PERMISSION_DENIED"
    );
    let mut owner = a.clone();
    owner.role = crate::components::identity::domain::AccessRole::Owner;
    seed_access(&mut c, &owner);
    assert!(
        run(&mut c, &owner, &ids, |cases| cases
            .register_patient(ProfileData::default()))
        .is_ok()
    );
}

#[test]
fn failure_in_either_local_or_outbox_write_rolls_back_both() {
    for table in ["patient_local", "sync_outbox"] {
        let mut c = database();
        let a = access();
        let ids = Ids::default();
        c.execute_batch(&format!("CREATE TRIGGER fail_patient_write BEFORE INSERT ON {table} BEGIN SELECT RAISE(ABORT,'synthetic failure'); END;")).unwrap();
        let error = run(&mut c, &a, &ids, |cases| {
            cases.register_patient(data("Ficticio")).unwrap_err()
        });
        assert_eq!(error.code, "INTERNAL_ERROR");
        assert!(!error.message.contains("synthetic"));
        assert!(read_local_changes(&c, &a.context).unwrap().is_empty());
        assert!(
            SqliteOperationStore
                .list_pending(&c, 10)
                .unwrap()
                .is_empty()
        );
    }
}

#[test]
fn invalid_profile_leaves_no_local_effect() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    assert_eq!(
        run(&mut c, &a, &ids, |cases| cases
            .register_patient(data(" "))
            .unwrap_err())
        .code,
        "VALIDATION_ERROR"
    );
    assert!(read_local_changes(&c, &a.context).unwrap().is_empty());
}

fn local_base(result: &LocalMutationResult) -> EditBase {
    EditBase::LocalOperation {
        operation_id: result.operation_id.clone().try_into().unwrap(),
    }
}

#[test]
fn successive_profile_edits_keep_bases_payloads_and_status_after_reopen() {
    let db = Database::new();
    let a = access();
    let ids = Ids::default();
    let mut c = sqlite::open(&db.0).unwrap();
    seed_access(&mut c, &a);
    let first = run(&mut c, &a, &ids, |cases| {
        cases.register_patient(data("Original ficticio")).unwrap()
    });
    let second = run(&mut c, &a, &ids, |cases| {
        cases
            .update_patient_profile(
                patient(&first),
                local_base(&first),
                ProfilePatch {
                    full_name: FieldChange::Set("Editado ficticio".into()),
                    ..ProfilePatch::default()
                },
            )
            .unwrap()
    });
    let third = run(&mut c, &a, &ids, |cases| {
        cases
            .set_patient_profile_status(patient(&first), local_base(&second), Status::Inactive)
            .unwrap()
    });
    assert_eq!(
        (
            first.expected_version,
            second.expected_version,
            third.expected_version
        ),
        (0, 1, 2)
    );
    assert_eq!(
        second.depends_on.as_slice(),
        std::slice::from_ref(&first.operation_id)
    );
    assert_eq!(
        third.depends_on.as_slice(),
        std::slice::from_ref(&second.operation_id)
    );
    let before = SqliteOperationStore
        .list_pending(&c, 20)
        .unwrap()
        .into_iter()
        .map(|v| v.envelope().unwrap())
        .collect::<Vec<_>>();
    // A backward device clock must not make an older draft replace a later edit.
    c.execute(
        "UPDATE patient_local SET created_at=99-expected_version",
        [],
    )
    .unwrap();
    drop(c);
    let mut c = sqlite::open(&db.0).unwrap();
    let row = run(&mut c, &a, &ids, |cases| {
        cases.get_patient(&patient(&first)).unwrap()
    });
    assert!(row.accepted.is_none());
    let p = row.provisional.unwrap();
    assert_eq!(p.state.status, "inactive");
    assert_eq!(p.state.completion_status, "draft");
    assert_eq!(p.state.full_name.as_deref(), Some("Editado ficticio"));
    assert_eq!(p.operation_id, third.operation_id);
    let after = SqliteOperationStore
        .list_pending(&c, 20)
        .unwrap()
        .into_iter()
        .map(|v| v.envelope().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(before, after);
    let locals = read_local_changes(&c, &a.context).unwrap();
    let last = locals
        .iter()
        .find(|v| v.operation_id == third.operation_id)
        .unwrap();
    assert_eq!(
        last.base_operation_id.as_deref(),
        Some(second.operation_id.as_str())
    );
    assert_eq!(
        run(&mut c, &a, &ids, |cases| cases
            .list_patients(&PatientFilter {
                status: Some(Status::Inactive),
                ..PatientFilter::default()
            })
            .unwrap())
        .total,
        1
    );
}

#[test]
fn stale_local_or_accepted_bases_do_not_add_an_operation() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    let first = run(&mut c, &a, &ids, |cases| {
        cases.register_patient(data("Ficticio")).unwrap()
    });
    let second = run(&mut c, &a, &ids, |cases| {
        cases
            .set_patient_profile_status(patient(&first), local_base(&first), Status::Inactive)
            .unwrap()
    });
    for base in [
        local_base(&first),
        EditBase::Accepted { version: 1 },
        EditBase::LocalOperation {
            operation_id: "019a1234-0000-7000-8000-000000000099"
                .to_owned()
                .try_into()
                .unwrap(),
        },
    ] {
        assert_eq!(
            run(&mut c, &a, &ids, |cases| cases
                .set_patient_profile_status(patient(&first), base, Status::Active)
                .unwrap_err())
            .code,
            "VERSION_CONFLICT"
        );
    }
    assert_eq!(read_local_changes(&c, &a.context).unwrap().len(), 2);
    assert_eq!(
        run(&mut c, &a, &ids, |cases| cases
            .get_patient(&patient(&first))
            .unwrap())
        .provisional
        .unwrap()
        .operation_id,
        second.operation_id
    );
}

#[test]
fn patch_omits_absent_fields_and_retains_explicit_null() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    let first = run(&mut c, &a, &ids, |cases| {
        cases
            .register_patient(ProfileData {
                phone: Some(Phone::new("+58".into(), "4120000000".into()).unwrap()),
                ..data("Ficticio")
            })
            .unwrap()
    });
    let second = run(&mut c, &a, &ids, |cases| {
        cases
            .update_patient_profile(
                patient(&first),
                local_base(&first),
                ProfilePatch {
                    phone: FieldChange::Clear,
                    phone_code: FieldChange::Clear,
                    ..ProfilePatch::default()
                },
            )
            .unwrap()
    });
    let op = SqliteOperationStore
        .list_pending(&c, 20)
        .unwrap()
        .into_iter()
        .find(|v| v.operation_id.to_string() == second.operation_id)
        .unwrap();
    assert_eq!(
        op.payload,
        serde_json::json!({"phone":null,"phoneCode":null})
    );
    assert_eq!(second.provisional["fullName"], "Ficticio");
}

#[test]
fn unchanged_or_invalid_edits_do_not_add_versions_or_intentions() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    let first = run(&mut c, &a, &ids, |cases| {
        cases.register_patient(data("Ficticio")).unwrap()
    });
    assert_eq!(
        run(&mut c, &a, &ids, |cases| cases
            .update_patient_profile(
                patient(&first),
                local_base(&first),
                ProfilePatch {
                    full_name: FieldChange::Set("Ficticio".into()),
                    ..ProfilePatch::default()
                }
            )
            .unwrap_err())
        .code,
        "NO_CHANGES"
    );
    assert_eq!(
        run(&mut c, &a, &ids, |cases| cases
            .set_patient_profile_status(patient(&first), local_base(&first), Status::Active)
            .unwrap_err())
        .code,
        "NO_CHANGES"
    );
    assert_eq!(
        run(&mut c, &a, &ids, |cases| cases
            .update_patient_profile(patient(&first), local_base(&first), ProfilePatch::default())
            .unwrap_err())
        .code,
        "VALIDATION_ERROR"
    );
    assert_eq!(read_local_changes(&c, &a.context).unwrap().len(), 1);
}

#[test]
fn known_failures_and_old_epoch_keep_work_and_block_new_edits() {
    for status in ["conflict", "rejected"] {
        let mut c = database();
        let a = access();
        let ids = Ids::default();
        let first = run(&mut c, &a, &ids, |cases| {
            cases.register_patient(data("Ficticio")).unwrap()
        });
        c.execute(
            "UPDATE sync_outbox SET status=?1 WHERE operation_id=?2",
            rusqlite::params![status, first.operation_id],
        )
        .unwrap();
        assert_eq!(
            run(&mut c, &a, &ids, |cases| cases
                .set_patient_profile_status(patient(&first), local_base(&first), Status::Inactive)
                .unwrap_err())
            .code,
            "DEPENDENCY_FAILED"
        );
        assert_eq!(read_local_changes(&c, &a.context).unwrap().len(), 1);
        let mut next = a.clone();
        next.context.permission_epoch = 2;
        seed_access(&mut c, &next);
        let row = run(&mut c, &next, &ids, |cases| {
            cases.get_patient(&patient(&first)).unwrap()
        });
        assert_eq!(row.operations[0].status, "review_required");
        assert!(row.provisional.is_some());
    }
}

#[test]
fn accepted_base_requires_the_exact_version_and_does_not_change_accepted_data() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    let id = "019a1234-0000-7000-8000-000000000099";
    let record = ProfileRecord {
        patient_id: id.into(),
        full_name: Some("Aceptado ficticio".into()),
        birth_date: None,
        phone_code: None,
        phone: None,
        identification_basis: None,
        completion_status: "draft".into(),
        status: "active".into(),
        created_at: None,
        updated_at: None,
        version: 3,
    };
    let tx = c.transaction().unwrap();
    save_profiles(&tx, &a.context, &record).unwrap();
    tx.commit().unwrap();
    assert_eq!(
        run(&mut c, &a, &ids, |cases| cases
            .set_patient_profile_status(
                id.to_owned().try_into().unwrap(),
                EditBase::Accepted { version: 2 },
                Status::Inactive
            )
            .unwrap_err())
        .code,
        "VERSION_CONFLICT"
    );
    let result = run(&mut c, &a, &ids, |cases| {
        cases
            .set_patient_profile_status(
                id.to_owned().try_into().unwrap(),
                EditBase::Accepted { version: 3 },
                Status::Inactive,
            )
            .unwrap()
    });
    assert_eq!(result.expected_version, 3);
    assert!(result.depends_on.is_empty());
    assert_eq!(read_profiles(&c, &a.context).unwrap(), [record]);
}

fn document() -> Document {
    Document::new("VE".into(), DocumentType::NationalId, "V-00.012-3".into()).unwrap()
}
fn adult() -> ProfileData {
    ProfileData {
        phone: Some(Phone::new("+58".into(), "4120000000".into()).unwrap()),
        identification_basis: Some(IdentificationBasis::OwnDocument),
        ..data("Paciente ficticio")
    }
}

#[test]
fn register_edit_document_complete_and_reopen_preserve_atomic_sequence() {
    let db = Database::new();
    let a = access();
    let ids = Ids::default();
    let mut c = sqlite::open(&db.0).unwrap();
    seed_access(&mut c, &a);
    let first = run(&mut c, &a, &ids, |cases| {
        cases.register_patient(adult()).unwrap()
    });
    let edit = run(&mut c, &a, &ids, |cases| {
        cases
            .update_patient_profile(
                patient(&first),
                local_base(&first),
                ProfilePatch {
                    full_name: FieldChange::Set("Editado ficticio".into()),
                    ..ProfilePatch::default()
                },
            )
            .unwrap()
    });
    let doc = run(&mut c, &a, &ids, |cases| {
        cases
            .add_patient_identifier(patient(&first), document())
            .unwrap()
    });
    assert_eq!(doc.provisional["patientId"], first.entity_key);
    assert_eq!(doc.provisional["verificationStatus"], "declared");
    assert_eq!(
        doc.depends_on.as_slice(),
        std::slice::from_ref(&edit.operation_id)
    );
    let complete = run(&mut c, &a, &ids, |cases| {
        cases
            .set_patient_completion(
                patient(&first),
                local_base(&edit),
                CompletionStatus::Complete,
            )
            .unwrap()
    });
    assert_eq!(complete.expected_version, 2);
    assert_eq!(
        complete.depends_on,
        [edit.operation_id.clone(), doc.operation_id.clone()]
    );
    let pending = SqliteOperationStore.list_pending(&c, 20).unwrap();
    assert_eq!(pending.len(), 4);
    let doc_op = pending
        .iter()
        .find(|v| v.action == "AddPatientIdentifier")
        .unwrap();
    assert_eq!(doc_op.entity_type, "patient_identifier");
    assert_eq!(doc_op.expected_version, 0);
    assert!(doc_op.payload.get("verificationStatus").is_none());
    let saved = pending
        .into_iter()
        .map(|v| v.envelope().unwrap())
        .collect::<Vec<_>>();
    drop(c);
    let mut c = sqlite::open(&db.0).unwrap();
    run(&mut c, &a, &ids, |cases| {
        let row = cases.get_patient(&patient(&first)).unwrap();
        assert!(row.accepted.is_none());
        let state = row.provisional.unwrap().state;
        assert_eq!(state.patient_id, patient(&first).as_str());
        assert_eq!(state.completion_status, "complete");
        let docs = cases.list_patient_identifiers(&patient(&first)).unwrap();
        assert_eq!(docs.len(), 1);
        assert!(docs[0].accepted.is_none());
        assert_eq!(docs[0].visible().unwrap().document_number, "V-00.012-3");
        assert_eq!(
            cases
                .list_patients(&PatientFilter {
                    q: Some("00.012".into()),
                    completion_status: Some(CompletionStatus::Complete),
                    ..PatientFilter::default()
                })
                .unwrap()
                .total,
            1
        );
        assert_eq!(
            cases
                .list_patients(&PatientFilter {
                    q: Some("00.012".into()),
                    completion_status: Some(CompletionStatus::Draft),
                    ..PatientFilter::default()
                })
                .unwrap()
                .total,
            0
        );
    });
    let mut repo = SqlitePatientRepository { connection: &mut c };
    assert!(
        repo.accepted_patient(&a, &patient(&first))
            .unwrap()
            .is_none()
    );
    assert!(
        repo.accepted_identifiers(&a, &patient(&first))
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        SqliteOperationStore
            .list_pending(&c, 20)
            .unwrap()
            .into_iter()
            .map(|v| v.envelope().unwrap())
            .collect::<Vec<_>>(),
        saved
    );
}

#[test]
fn initial_documents_share_registration_and_cannot_claim_verification() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    let first = run(&mut c, &a, &ids, |cases| {
        cases
            .register_patient_with_identifiers(adult(), vec![document()])
            .unwrap()
    });
    let op = SqliteOperationStore.list_pending(&c, 10).unwrap().remove(0);
    let input = &op.payload["identifiers"][0];
    assert_eq!(input["documentNumber"], "V-00.012-3");
    assert!(input.get("verificationStatus").is_none());
    let docs = run(&mut c, &a, &ids, |cases| {
        cases.list_patient_identifiers(&patient(&first)).unwrap()
    });
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].visible().unwrap().verification_status, "declared");
    assert_eq!(docs[0].operations[0].operation_id, first.operation_id);
    let complete = run(&mut c, &a, &ids, |cases| {
        cases
            .set_patient_completion(
                patient(&first),
                local_base(&first),
                CompletionStatus::Complete,
            )
            .unwrap()
    });
    assert_eq!(
        complete.depends_on.as_slice(),
        std::slice::from_ref(&first.operation_id)
    );
    assert_eq!(
        run(&mut c, &a, &ids, |cases| cases
            .register_patient_with_identifiers(adult(), vec![document(); 101])
            .unwrap_err())
        .code,
        "VALIDATION_ERROR"
    );
}

#[test]
fn completing_without_requirements_keeps_the_draft_and_prior_operations() {
    for profile in [
        ProfileData::default(),
        adult(),
        ProfileData {
            phone: None,
            ..adult()
        },
    ] {
        let mut c = database();
        let a = access();
        let ids = Ids::default();
        let first = run(&mut c, &a, &ids, |cases| {
            cases.register_patient(profile).unwrap()
        });
        assert_eq!(
            run(&mut c, &a, &ids, |cases| cases
                .set_patient_completion(
                    patient(&first),
                    local_base(&first),
                    CompletionStatus::Complete
                )
                .unwrap_err())
            .code,
            "VALIDATION_ERROR"
        );
        assert_eq!(read_local_changes(&c, &a.context).unwrap().len(), 1);
        assert_eq!(
            run(&mut c, &a, &ids, |cases| cases
                .get_patient(&patient(&first))
                .unwrap())
            .visible()
            .unwrap()
            .completion_status,
            "draft"
        );
    }
}

#[test]
fn complete_profile_requires_explicit_draft_before_removing_required_data() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    let first = run(&mut c, &a, &ids, |cases| {
        cases
            .register_patient_with_identifiers(adult(), vec![document()])
            .unwrap()
    });
    let complete = run(&mut c, &a, &ids, |cases| {
        cases
            .set_patient_completion(
                patient(&first),
                local_base(&first),
                CompletionStatus::Complete,
            )
            .unwrap()
    });
    let error = run(&mut c, &a, &ids, |cases| {
        cases
            .update_patient_profile(
                patient(&first),
                local_base(&complete),
                ProfilePatch {
                    full_name: FieldChange::Clear,
                    ..ProfilePatch::default()
                },
            )
            .unwrap_err()
    });
    assert_eq!(error.code, "COMPLETENESS_CONFLICT");
    assert_eq!(error.affected_patient_ids, [patient(&first).as_str()]);
    assert_eq!(read_local_changes(&c, &a.context).unwrap().len(), 2);
    let draft = run(&mut c, &a, &ids, |cases| {
        cases
            .set_patient_completion(
                patient(&first),
                local_base(&complete),
                CompletionStatus::Draft,
            )
            .unwrap()
    });
    let edit = run(&mut c, &a, &ids, |cases| {
        cases
            .update_patient_profile(
                patient(&first),
                local_base(&draft),
                ProfilePatch {
                    full_name: FieldChange::Clear,
                    ..ProfilePatch::default()
                },
            )
            .unwrap()
    });
    assert!(edit.provisional["fullName"].is_null());
    assert_eq!(edit.provisional["completionStatus"], "draft");
}

fn seed_responsible(
    c: &mut rusqlite::Connection,
    a: &crate::components::identity::domain::AccountAccess,
    id: &PatientId,
    contact_status: &str,
    until: Option<&str>,
) {
    let tx = c.transaction().unwrap();
    let contact = ContactRecord {
        contact_person_id: "019a1234-0000-7000-8000-000000000088".into(),
        full_name: "Responsable ficticio".into(),
        phone_code: Some("+58".into()),
        phone: Some("4120000000".into()),
        issuing_country: None,
        document_type: None,
        document_number: None,
        status: contact_status.into(),
        created_at: None,
        updated_at: None,
        version: 1,
    };
    save_contacts(&tx, &a.context, &contact).unwrap();
    let relation = RelationshipRecord {
        patient_id: id.as_str().into(),
        contact_person_id: contact.contact_person_id,
        relationship_kind: "guardian".into(),
        is_responsible_contact: true,
        representation_status: "unverified".into(),
        verified_at: None,
        verified_by_user_id: None,
        valid_from: "2020-01-01T00:00:00Z".into(),
        valid_until: until.map(str::to_owned),
        status: "active".into(),
        created_at: None,
        updated_at: None,
        version: 1,
    };
    save_relationships(&tx, &a.context, &relation).unwrap();
    tx.commit().unwrap();
}

#[test]
fn minor_requires_current_responsible_phone_and_keeps_identity_after_own_document() {
    for (status, until, valid) in [
        ("inactive", None, false),
        ("active", Some("2021-01-01T00:00:00Z"), false),
        ("active", None, true),
    ] {
        let mut c = database();
        let a = access();
        let ids = Ids::default();
        let first = run(&mut c, &a, &ids, |cases| {
            cases
                .register_patient(ProfileData {
                    birth_date: Some("2018-02-28".into()),
                    identification_basis: Some(IdentificationBasis::MinorWithoutDocument),
                    ..data("Menor ficticio")
                })
                .unwrap()
        });
        seed_responsible(&mut c, &a, &patient(&first), status, until);
        let completion = run(&mut c, &a, &ids, |cases| {
            cases.set_patient_completion(
                patient(&first),
                local_base(&first),
                CompletionStatus::Complete,
            )
        });
        assert_eq!(completion.is_ok(), valid);
        if let Ok(complete) = completion {
            let doc = run(&mut c, &a, &ids, |cases| {
                cases
                    .add_patient_identifier(patient(&first), document())
                    .unwrap()
            });
            assert_eq!(doc.provisional["patientId"], first.entity_key);
            let row = run(&mut c, &a, &ids, |cases| {
                cases.get_patient(&patient(&first)).unwrap()
            });
            assert_eq!(row.provisional.unwrap().operation_id, complete.operation_id);
            let switched = run(&mut c, &a, &ids, |cases| {
                cases
                    .update_patient_profile(
                        patient(&first),
                        local_base(&complete),
                        ProfilePatch {
                            identification_basis: FieldChange::Set(
                                IdentificationBasis::OwnDocument,
                            ),
                            ..ProfilePatch::default()
                        },
                    )
                    .unwrap()
            });
            assert_eq!(switched.entity_key, first.entity_key);
            assert_eq!(switched.provisional["completionStatus"], "complete");
            assert!(switched.depends_on.contains(&doc.operation_id));
        }
    }
}

#[test]
fn rejected_or_invalidated_document_cannot_complete_a_patient() {
    for rejected in [true, false] {
        let mut c = database();
        let a = access();
        let ids = Ids::default();
        let first = run(&mut c, &a, &ids, |cases| {
            cases.register_patient(adult()).unwrap()
        });
        let doc = run(&mut c, &a, &ids, |cases| {
            cases
                .add_patient_identifier(patient(&first), document())
                .unwrap()
        });
        if rejected {
            c.execute(
                "UPDATE sync_outbox SET status='rejected' WHERE operation_id=?1",
                [&doc.operation_id],
            )
            .unwrap();
        } else {
            let tx = c.transaction().unwrap();
            save_identifiers(
                &tx,
                &a.context,
                &IdentifierRecord {
                    identifier_id: doc.entity_key.as_str().unwrap().into(),
                    patient_id: patient(&first).as_str().into(),
                    issuing_country: "VE".into(),
                    document_type: "national_id".into(),
                    document_number: "V-00.012-3".into(),
                    verification_status: "invalidated".into(),
                    created_at: None,
                    updated_at: None,
                    version: 1,
                },
            )
            .unwrap();
            tx.commit().unwrap();
        }
        assert!(
            run(&mut c, &a, &ids, |cases| cases.set_patient_completion(
                patient(&first),
                local_base(&first),
                CompletionStatus::Complete
            ))
            .is_err()
        );
        assert_eq!(read_local_changes(&c, &a.context).unwrap().len(), 2);
    }
}

#[test]
fn document_and_completion_failures_roll_back_only_the_new_operation() {
    for action in ["AddPatientIdentifier", "SetPatientCompletion"] {
        let mut c = database();
        let a = access();
        let ids = Ids::default();
        let first = run(&mut c, &a, &ids, |cases| {
            cases
                .register_patient_with_identifiers(adult(), vec![document()])
                .unwrap()
        });
        c.execute_batch(&format!("CREATE TRIGGER fail_specific_write BEFORE INSERT ON patient_local WHEN NEW.action='{action}' BEGIN SELECT RAISE(ABORT,'synthetic failure'); END;")).unwrap();
        let result = run(&mut c, &a, &ids, |cases| {
            if action == "AddPatientIdentifier" {
                cases.add_patient_identifier(patient(&first), document())
            } else {
                cases.set_patient_completion(
                    patient(&first),
                    local_base(&first),
                    CompletionStatus::Complete,
                )
            }
        });
        assert_eq!(result.unwrap_err().code, "INTERNAL_ERROR");
        assert_eq!(read_local_changes(&c, &a.context).unwrap().len(), 1);
        assert_eq!(SqliteOperationStore.list_pending(&c, 20).unwrap().len(), 1);
    }
}

#[test]
fn document_and_completion_check_current_permissions_and_hide_other_scopes() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    let first = run(&mut c, &a, &ids, |cases| {
        cases
            .register_patient_with_identifiers(adult(), vec![document()])
            .unwrap()
    });
    let mut denied = a.clone();
    denied
        .allowed_actions
        .retain(|v| !matches!(v.as_str(), "AddPatientIdentifier" | "SetPatientCompletion"));
    seed_access(&mut c, &denied);
    assert_eq!(
        run(&mut c, &a, &ids, |cases| cases
            .add_patient_identifier(patient(&first), document())
            .unwrap_err())
        .code,
        "PERMISSION_DENIED"
    );
    assert_eq!(
        run(&mut c, &a, &ids, |cases| cases
            .set_patient_completion(
                patient(&first),
                local_base(&first),
                CompletionStatus::Complete
            )
            .unwrap_err())
        .code,
        "PERMISSION_DENIED"
    );
    let mut other = a.clone();
    other.context.account_id = "019a1234-0000-7000-8000-000000000099"
        .to_owned()
        .try_into()
        .unwrap();
    seed_access(&mut c, &other);
    assert_eq!(
        run(&mut c, &other, &ids, |cases| cases
            .list_patient_identifiers(&patient(&first))
            .unwrap_err())
        .code,
        "NOT_FOUND"
    );
    assert_eq!(
        run(&mut c, &other, &ids, |cases| cases
            .list_patients(&PatientFilter {
                q: Some("00.012".into()),
                ..PatientFilter::default()
            })
            .unwrap())
        .total,
        0
    );
}

#[test]
fn uncertain_original_allows_distinct_dependent_edits_without_replacing_it() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    let first = run(&mut c, &a, &ids, |cases| {
        cases.register_patient(adult()).unwrap()
    });
    let original = SqliteOperationStore
        .list_pending(&c, 20)
        .unwrap()
        .remove(0)
        .envelope()
        .unwrap();
    c.execute(
        "UPDATE sync_outbox SET status='unknown' WHERE operation_id=?1",
        [&first.operation_id],
    )
    .unwrap();
    let edit = run(&mut c, &a, &ids, |cases| {
        cases
            .update_patient_profile(
                patient(&first),
                local_base(&first),
                ProfilePatch {
                    full_name: FieldChange::Set("Nuevo nombre ficticio".into()),
                    ..ProfilePatch::default()
                },
            )
            .unwrap()
    });
    assert_ne!(edit.operation_id, first.operation_id);
    assert_eq!(
        edit.depends_on.as_slice(),
        std::slice::from_ref(&first.operation_id)
    );
    let doc = run(&mut c, &a, &ids, |cases| {
        cases
            .add_patient_identifier(patient(&first), document())
            .unwrap()
    });
    c.execute(
        "UPDATE sync_outbox SET status='unknown' WHERE operation_id=?1",
        [&doc.operation_id],
    )
    .unwrap();
    let completed = run(&mut c, &a, &ids, |cases| {
        cases
            .set_patient_completion(
                patient(&first),
                local_base(&edit),
                CompletionStatus::Complete,
            )
            .unwrap()
    });
    assert!(completed.depends_on.contains(&doc.operation_id));
    let stored = SqliteOperationStore
        .operation_result(
            &c,
            &a.context,
            &first.operation_id.clone().try_into().unwrap(),
        )
        .unwrap()
        .unwrap();
    assert_eq!(stored.operation.envelope().unwrap(), original);
    assert_eq!(
        stored.status,
        crate::components::sync::domain::OperationStatus::Unknown
    );
    assert!(
        run(&mut c, &a, &ids, |cases| cases
            .get_patient(&patient(&first))
            .unwrap())
        .accepted
        .is_none()
    );
}
