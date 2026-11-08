use crate::components::patients::adapters::outbound::{
    LocalChange, insert_local_change, read_contacts, read_identifiers, read_local_changes,
    read_profiles, read_relationships,
};
use crate::components::patients::adapters::projector::*;
use crate::components::sync::remote::{ChangeGroup, ChangeKind, SyncChange, apply_groups};
use crate::components::sync::remote::{EntityType, RemoteChange, RemoteProjector};
use crate::shared_kernel::id::OperationContext;

const PATIENT: &str = "019a1234-0000-7000-8000-000000000010";
const CONTACT: &str = "019a1234-0000-7000-8000-000000000011";
const IDENTIFIER: &str = "019a1234-0000-7000-8000-000000000012";

fn scope() -> OperationContext {
    serde_json::from_value(serde_json::json!({
        "accountId":"019a1234-0000-7000-8000-000000000001",
        "userId":"019a1234-0000-7000-8000-000000000002",
        "deviceId":"019a1234-0000-7000-8000-000000000003", "permissionEpoch":1
    }))
    .unwrap()
}

fn database() -> rusqlite::Connection {
    crate::platform::sqlite::open(std::path::Path::new(":memory:")).unwrap()
}

fn profile(s: &OperationContext, version: i64) -> SyncChange {
    SyncChange {
        entity_type: EntityType::PatientProfile,
        entity_key: serde_json::json!(PATIENT),
        entity_version: version,
        change_kind: ChangeKind::Upsert,
        state: serde_json::json!({"accountId":s.account_id,"patientId":PATIENT,"fullName":format!("Paciente remoto {version}"),"birthDate":"1990-05-05","phoneCode":"+58","phone":"4140000000","identificationBasis":"own_document","completionStatus":"complete","status":"active","version":version,"createdAt":null,"updatedAt":null}),
    }
}

fn identifier(version: i64) -> SyncChange {
    SyncChange {
        entity_type: EntityType::PatientIdentifier,
        entity_key: serde_json::json!(IDENTIFIER),
        entity_version: version,
        change_kind: ChangeKind::Upsert,
        state: serde_json::json!({"id":IDENTIFIER,"patientId":PATIENT,"issuingCountry":"VE","documentType":"national_id","documentNumber":"V-00.012-3","verificationStatus":"verified","version":version,"createdAt":null,"updatedAt":null}),
    }
}

fn contact(s: &OperationContext, version: i64) -> SyncChange {
    SyncChange {
        entity_type: EntityType::ContactPerson,
        entity_key: serde_json::json!(CONTACT),
        entity_version: version,
        change_kind: ChangeKind::Upsert,
        state: serde_json::json!({"accountId":s.account_id,"id":CONTACT,"fullName":"Responsable remoto","phoneCode":"+58","phone":"4120000000","issuingCountry":null,"documentType":null,"documentNumber":null,"status":"active","version":version,"createdAt":null,"updatedAt":null}),
    }
}

fn relationship(s: &OperationContext, version: i64) -> SyncChange {
    SyncChange {
        entity_type: EntityType::PatientContact,
        entity_key: serde_json::json!({"patientId":PATIENT,"contactPersonId":CONTACT}),
        entity_version: version,
        change_kind: ChangeKind::Upsert,
        state: serde_json::json!({"accountId":s.account_id,"patientId":PATIENT,"contactPersonId":CONTACT,"relationshipKind":"mother","isResponsibleContact":true,"representationStatus":"unverified","verifiedAt":null,"verifiedByUserId":null,"validFrom":"2026-01-01T08:00:00-04:00","validUntil":null,"status":"active","version":version,"createdAt":null,"updatedAt":null}),
    }
}

fn group(changes: Vec<SyncChange>) -> ChangeGroup {
    ChangeGroup {
        group_id: uuid::Uuid::now_v7().to_string().try_into().unwrap(),
        operation_id: uuid::Uuid::now_v7().to_string().try_into().unwrap(),
        accepted_at: "2026-11-05T12:00:00Z".into(),
        changes,
    }
}

fn full_group(s: &OperationContext, version: i64) -> ChangeGroup {
    group(vec![
        profile(s, version),
        identifier(version),
        contact(s, version),
        relationship(s, version),
    ])
}

#[test]
fn repeated_groups_and_old_versions_do_not_regress_views() {
    let mut c = database();
    let s = scope();
    apply_groups(&mut c, &s, &[full_group(&s, 2)], &PatientsProjector).unwrap();
    let repeated = full_group(&s, 2);
    apply_groups(&mut c, &s, &[repeated], &PatientsProjector).unwrap();
    apply_groups(&mut c, &s, &[full_group(&s, 1)], &PatientsProjector).unwrap();

    assert_eq!(read_profiles(&c, &s).unwrap()[0].version, 2);
    assert_eq!(
        read_profiles(&c, &s).unwrap()[0].full_name.as_deref(),
        Some("Paciente remoto 2")
    );
    assert_eq!(read_identifiers(&c, &s).unwrap()[0].version, 2);
    assert_eq!(read_contacts(&c, &s).unwrap()[0].version, 2);
    assert_eq!(read_relationships(&c, &s).unwrap()[0].version, 2);
}

#[test]
fn remove_and_reset_preserve_intentions_and_other_scopes() {
    let mut c = database();
    let s = scope();
    apply_groups(&mut c, &s, &[full_group(&s, 1)], &PatientsProjector).unwrap();

    // A local intent on top of the accepted view must survive reset.
    {
        let tx = c.transaction().unwrap();
        insert_local_change(
            &tx,
            &s,
            &LocalChange {
                operation_id: "019a1234-0000-7000-8000-000000000020".into(),
                entity_type: "patient_profile".into(),
                patient_id: Some(PATIENT.into()),
                identifier_id: None,
                contact_person_id: None,
                action: "UpdatePatientProfile".into(),
                expected_version: 1,
                base_operation_id: None,
                depends_on: "[]".into(),
                payload: "{}".into(),
                draft: "{}".into(),
                created_at: 1,
            },
        )
        .unwrap();
        tx.commit().unwrap();
    }

    let mut other = scope();
    other.account_id = "019a1234-0000-7000-8000-000000000099"
        .to_owned()
        .try_into()
        .unwrap();
    apply_groups(&mut c, &other, &[full_group(&other, 1)], &PatientsProjector).unwrap();

    let tx = c.transaction().unwrap();
    PatientsProjector
        .project(
            &tx,
            &s,
            &RemoteChange::Remove {
                entity_type: EntityType::PatientProfile,
                key: serde_json::json!(PATIENT),
            },
        )
        .unwrap();
    PatientsProjector.reset(&tx, &s).unwrap();
    tx.commit().unwrap();

    assert!(read_profiles(&c, &s).unwrap().is_empty());
    assert!(read_identifiers(&c, &s).unwrap().is_empty());
    assert!(read_contacts(&c, &s).unwrap().is_empty());
    assert!(read_relationships(&c, &s).unwrap().is_empty());
    assert_eq!(read_local_changes(&c, &s).unwrap().len(), 1);
    assert_eq!(read_profiles(&c, &other).unwrap().len(), 1);

    apply_groups(&mut c, &s, &[full_group(&s, 1)], &PatientsProjector).unwrap();
    let rows = read_profiles(&c, &s).unwrap();
    assert_eq!(rows[0].version, 1);
}

#[test]
fn invalid_projection_rolls_back_whole_group_and_cursor_inbox_effects() {
    let mut c = database();
    let s = scope();
    let mut invalid = relationship(&s, 1);
    invalid.state["validUntil"] = serde_json::json!("2025-01-01T08:00:00-04:00");
    assert!(
        apply_groups(
            &mut c,
            &s,
            &[group(vec![
                profile(&s, 1),
                identifier(1),
                contact(&s, 1),
                invalid
            ])],
            &PatientsProjector
        )
        .is_err()
    );
    assert!(read_profiles(&c, &s).unwrap().is_empty());
    assert_eq!(
        c.query_row("SELECT count(*) FROM sync_inbox", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert_eq!(
        c.query_row("SELECT count(*) FROM sync_remote_items", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
}

#[test]
fn out_of_scope_account_states_are_rejected() {
    let mut c = database();
    let s = scope();
    let mut foreign = profile(&s, 1);
    foreign.state["accountId"] = serde_json::json!("019a1234-0000-7000-8000-000000000099");
    assert!(apply_groups(&mut c, &s, &[group(vec![foreign])], &PatientsProjector).is_err());
    assert!(read_profiles(&c, &s).unwrap().is_empty());
}
