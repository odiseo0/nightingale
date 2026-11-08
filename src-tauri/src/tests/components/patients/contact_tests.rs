use super::test_support::*;
use crate::components::patients::{application::*, domain::*};
use crate::{
    components::sync::{adapters::outbound::SqliteOperationStore, application::OperationStore},
    platform::sqlite,
};

fn phone_contact(name: &str) -> ContactData {
    ContactData::new(
        name.into(),
        Some("+58".into()),
        Some("4120000001".into()),
        None,
        None,
        None,
    )
    .unwrap()
}

fn contact(result: &LocalMutationResult) -> ContactPersonId {
    result
        .entity_key
        .as_str()
        .unwrap()
        .to_owned()
        .try_into()
        .unwrap()
}

fn local_base(result: &LocalMutationResult) -> EditBase {
    EditBase::LocalOperation {
        operation_id: result.operation_id.clone().try_into().unwrap(),
    }
}

#[test]
fn create_search_read_and_reopen_contact_without_http() {
    let db = Database::new();
    let a = access();
    let ids = Ids::default();
    let mut c = sqlite::open(&db.0).unwrap();
    seed_access(&mut c, &a);
    let created = run(&mut c, &a, &ids, |cases| {
        cases
            .create_contact_person(phone_contact("Responsable ficticio"))
            .unwrap()
    });
    let pending = SqliteOperationStore.list_pending(&c, 10).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].action, "CreateContactPerson");
    assert_eq!(pending[0].entity_key, created.entity_key);
    assert_eq!(pending[0].expected_version, 0);
    drop(c);

    let mut c = sqlite::open(&db.0).unwrap();
    run(&mut c, &a, &ids, |cases| {
        let page = cases
            .list_contacts(&ContactFilter {
                q: Some("responsable".into()),
                ..ContactFilter::default()
            })
            .unwrap();
        assert_eq!(page.total, 1);
        let row = cases.get_contact(&contact(&created)).unwrap();
        assert!(row.accepted.is_none());
        let provisional = row.provisional.unwrap();
        assert_eq!(provisional.state.full_name, "Responsable ficticio");
        assert_eq!(provisional.expected_version, 0);
        assert_eq!(row.operations[0].status, "pending");
    });
}

#[test]
fn matching_contact_names_create_distinct_shared_identities() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    run(&mut c, &a, &ids, |cases| {
        let one = cases
            .create_contact_person(phone_contact("Nombre compartido"))
            .unwrap();
        let two = cases
            .create_contact_person(phone_contact("Nombre compartido"))
            .unwrap();
        assert_ne!(one.entity_key, two.entity_key);

        let page = cases
            .list_contacts(&ContactFilter {
                q: Some("4120000001".into()),
                ..ContactFilter::default()
            })
            .unwrap();
        assert_eq!(page.total, 2);
        assert_eq!(page.items.len(), 2);
        assert!(
            cases
                .list_contacts(&ContactFilter {
                    status: Some(Status::Inactive),
                    ..ContactFilter::default()
                })
                .unwrap()
                .items
                .is_empty()
        );
        assert!(
            cases
                .list_contacts(&ContactFilter {
                    page: 0,
                    ..ContactFilter::default()
                })
                .is_err()
        );
    });
}

#[test]
fn successive_contact_edits_keep_bases_payloads_and_status() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    run(&mut c, &a, &ids, |cases| {
        let created = cases
            .create_contact_person(phone_contact("Madre ficticia"))
            .unwrap();
        let first = cases
            .update_contact_person(
                contact(&created),
                local_base(&created),
                ContactPatch {
                    full_name: FieldChange::Set("Madre ficticia Pérez".into()),
                    ..ContactPatch::default()
                },
            )
            .unwrap();
        assert_eq!(first.expected_version, 1);
        assert_eq!(first.depends_on, vec![created.operation_id.clone()]);

        let second = cases
            .set_contact_person_status(contact(&created), local_base(&first), Status::Inactive)
            .unwrap();
        assert_eq!(second.expected_version, 2);
        assert_eq!(second.depends_on, vec![first.operation_id]);

        let row = cases.get_contact(&contact(&created)).unwrap();
        assert_eq!(row.provisional.unwrap().state.status, "inactive");
        assert_eq!(row.operations.len(), 3);
        assert!(row.accepted.is_none());
    });
}

#[test]
fn contact_edit_contract_errors_and_stale_bases_are_rejected() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();
    run(&mut c, &a, &ids, |cases| {
        let created = cases
            .create_contact_person(phone_contact("Padre ficticio"))
            .unwrap();
        let id = contact(&created);

        // No difference, empty patch, cleared name and half-set pairs or triples.
        assert_eq!(
            cases
                .update_contact_person(
                    id.clone(),
                    local_base(&created),
                    ContactPatch {
                        full_name: FieldChange::Set("Padre ficticio".into()),
                        ..ContactPatch::default()
                    }
                )
                .unwrap_err()
                .code,
            "NO_CHANGES"
        );
        assert_eq!(
            cases
                .update_contact_person(id.clone(), local_base(&created), ContactPatch::default())
                .unwrap_err()
                .code,
            "VALIDATION_ERROR"
        );
        assert_eq!(
            cases
                .update_contact_person(
                    id.clone(),
                    local_base(&created),
                    ContactPatch {
                        full_name: FieldChange::Clear,
                        ..ContactPatch::default()
                    }
                )
                .unwrap_err()
                .code,
            "VALIDATION_ERROR"
        );
        assert_eq!(
            cases
                .update_contact_person(
                    id.clone(),
                    local_base(&created),
                    ContactPatch {
                        phone: FieldChange::Clear,
                        ..ContactPatch::default()
                    }
                )
                .unwrap_err()
                .code,
            "VALIDATION_ERROR"
        );
        assert_eq!(
            cases
                .update_contact_person(
                    id.clone(),
                    local_base(&created),
                    ContactPatch {
                        issuing_country: FieldChange::Set("VE".into()),
                        ..ContactPatch::default()
                    }
                )
                .unwrap_err()
                .code,
            "VALIDATION_ERROR"
        );

        // An accepted or stale base cannot edit a purely local contact.
        assert_eq!(
            cases
                .update_contact_person(
                    id.clone(),
                    EditBase::Accepted { version: 1 },
                    ContactPatch {
                        full_name: FieldChange::Set("Otro nombre".into()),
                        ..ContactPatch::default()
                    }
                )
                .unwrap_err()
                .code,
            "VERSION_CONFLICT"
        );
        let other = cases
            .create_contact_person(phone_contact("Contacto aparte"))
            .unwrap();
        assert_eq!(
            cases
                .update_contact_person(
                    id.clone(),
                    local_base(&other),
                    ContactPatch {
                        full_name: FieldChange::Set("Otro nombre".into()),
                        ..ContactPatch::default()
                    }
                )
                .unwrap_err()
                .code,
            "VERSION_CONFLICT"
        );

        // The rejected attempt above kept no operation; the valid base still works.
        let updated = cases
            .update_contact_person(
                id.clone(),
                local_base(&created),
                ContactPatch {
                    document_type: FieldChange::Set(DocumentType::Passport),
                    issuing_country: FieldChange::Set("VE".into()),
                    document_number: FieldChange::Set("P-0001".into()),
                    ..ContactPatch::default()
                },
            )
            .unwrap();
        let row = cases.get_contact(&id).unwrap();
        let state = row.provisional.unwrap().state;
        assert_eq!(state.document_number.as_deref(), Some("P-0001"));
        assert_eq!(updated.depends_on, vec![created.operation_id]);
    });
}

#[test]
fn contact_permissions_isolation_and_rollback_cover_intent_and_outbox() {
    let mut c = database();
    let a = access();
    let ids = Ids::default();

    let mut denied = a.clone();
    denied
        .allowed_actions
        .retain(|v| v != "CreateContactPerson");
    seed_access(&mut c, &denied);
    assert_eq!(
        run(&mut c, &a, &ids, |cases| cases
            .create_contact_person(phone_contact("Sin permiso"))
            .unwrap_err())
        .code,
        "PERMISSION_DENIED"
    );
    assert!(
        SqliteOperationStore
            .list_pending(&c, 10)
            .unwrap()
            .is_empty()
    );
    seed_access(&mut c, &a);

    let created = run(&mut c, &a, &ids, |cases| {
        cases
            .create_contact_person(phone_contact("Con permiso"))
            .unwrap()
    });
    assert_eq!(SqliteOperationStore.list_pending(&c, 10).unwrap().len(), 1);

    // Another account scope does not see the contact.
    let mut other = a.clone();
    other.context.account_id = "019a1234-0000-7000-8000-000000000099"
        .to_owned()
        .try_into()
        .unwrap();
    seed_access(&mut c, &other);
    assert_eq!(
        run(&mut c, &other, &ids, |cases| cases
            .get_contact(&contact(&created))
            .unwrap_err())
        .code,
        "NOT_FOUND"
    );

    // A forced failure in the local intent rolls back the Outbox operation too.
    c.execute_batch("CREATE TRIGGER fail_contact_write BEFORE INSERT ON patient_local WHEN NEW.action='UpdateContactPerson' BEGIN SELECT RAISE(ABORT,'synthetic failure'); END;").unwrap();
    assert_eq!(
        run(&mut c, &a, &ids, |cases| cases
            .update_contact_person(
                contact(&created),
                local_base(&created),
                ContactPatch {
                    full_name: FieldChange::Set("No se guarda".into()),
                    ..ContactPatch::default()
                }
            )
            .unwrap_err())
        .code,
        "INTERNAL_ERROR"
    );
    assert_eq!(SqliteOperationStore.list_pending(&c, 10).unwrap().len(), 1);
    run(&mut c, &a, &ids, |cases| {
        let row = cases.get_contact(&contact(&created)).unwrap();
        assert_eq!(row.provisional.unwrap().state.full_name, "Con permiso");
        assert_eq!(row.operations.len(), 1);
    });
}
