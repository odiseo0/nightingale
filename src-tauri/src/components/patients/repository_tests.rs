use super::adapters::outbound::*;
use crate::{platform::sqlite, shared_kernel::id::OperationContext};
use rusqlite::{Connection, params};
use serde_json::json;
use std::{fs, path::PathBuf};

const PATIENT: &str = "019a1234-0000-7000-8000-000000000004";
const CONTACT: &str = "019a1234-0000-7000-8000-000000000005";
const IDENTIFIER: &str = "019a1234-0000-7000-8000-000000000006";
const OPERATION: &str = "019a1234-0000-7000-8000-000000000007";

fn scope() -> OperationContext {
    serde_json::from_value(json!({
        "accountId":"019a1234-0000-7000-8000-000000000001",
        "userId":"019a1234-0000-7000-8000-000000000002",
        "deviceId":"019a1234-0000-7000-8000-000000000003", "permissionEpoch":1
    }))
    .unwrap()
}

struct Database(PathBuf);

impl Database {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!(
            "nightingale-patients-{}.sqlite",
            uuid::Uuid::now_v7()
        )))
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        fs::remove_file(&self.0).unwrap();
    }
}

fn profile() -> ProfileRecord {
    ProfileRecord {
        patient_id: PATIENT.into(),
        full_name: Some("Menor ficticio".into()),
        birth_date: Some("2018-02-28".into()),
        phone_code: None,
        phone: None,
        identification_basis: Some("minor_without_document".into()),
        completion_status: "draft".into(),
        status: "active".into(),
        created_at: Some("2026-11-04T14:00:00Z".into()),
        updated_at: None,
        version: 1,
    }
}

fn identifier() -> IdentifierRecord {
    IdentifierRecord {
        identifier_id: IDENTIFIER.into(),
        patient_id: PATIENT.into(),
        issuing_country: "VE".into(),
        document_type: "national_id".into(),
        document_number: "V-00.012-3".into(),
        verification_status: "declared".into(),
        created_at: None,
        updated_at: None,
        version: 1,
    }
}

fn contact() -> ContactRecord {
    ContactRecord {
        contact_person_id: CONTACT.into(),
        full_name: "Responsable ficticio".into(),
        phone_code: Some("+58".into()),
        phone: Some("4120000000".into()),
        issuing_country: None,
        document_type: None,
        document_number: None,
        status: "active".into(),
        created_at: None,
        updated_at: None,
        version: 1,
    }
}

fn relationship() -> RelationshipRecord {
    RelationshipRecord {
        patient_id: PATIENT.into(),
        contact_person_id: CONTACT.into(),
        relationship_kind: "guardian".into(),
        is_responsible_contact: true,
        representation_status: "unverified".into(),
        verified_at: None,
        verified_by_user_id: None,
        valid_from: "2026-11-04T14:00:00Z".into(),
        valid_until: None,
        status: "active".into(),
        created_at: None,
        updated_at: None,
        version: 1,
    }
}

fn change() -> LocalChange {
    LocalChange {
        operation_id: OPERATION.into(),
        entity_type: "patient_profile".into(),
        patient_id: Some(PATIENT.into()),
        identifier_id: None,
        contact_person_id: None,
        action: "UpdatePatientProfile".into(),
        expected_version: 1,
        base_operation_id: None,
        depends_on: "[]".into(),
        payload: r#"{"fullName":"Nombre ficticio nuevo"}"#.into(),
        draft: r#"{"fullName":"Nombre ficticio nuevo","completionStatus":"draft"}"#.into(),
        created_at: 1,
    }
}

fn seed(connection: &mut Connection, s: &OperationContext) {
    let tx = connection.transaction().unwrap();
    save_profiles(&tx, s, &profile()).unwrap();
    save_identifiers(&tx, s, &identifier()).unwrap();
    save_contacts(&tx, s, &contact()).unwrap();
    save_relationships(&tx, s, &relationship()).unwrap();
    insert_local_change(&tx, s, &change()).unwrap();
    tx.commit().unwrap();
}

fn assert_rows(connection: &Connection, s: &OperationContext) {
    assert_eq!(read_profiles(connection, s).unwrap(), [profile()]);
    assert_eq!(read_identifiers(connection, s).unwrap(), [identifier()]);
    assert_eq!(read_contacts(connection, s).unwrap(), [contact()]);
    assert_eq!(read_relationships(connection, s).unwrap(), [relationship()]);
    assert_eq!(read_local_changes(connection, s).unwrap(), [change()]);
}

#[test]
fn all_records_survive_reopen_without_http() {
    let database = Database::new();
    let s = scope();
    let mut c = sqlite::open(&database.0).unwrap();
    seed(&mut c, &s);
    assert_rows(&c, &s);
    drop(c);
    let c = sqlite::open(&database.0).unwrap();
    assert_rows(&c, &s);
    assert_eq!(
        c.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}

#[test]
fn every_reader_and_writer_uses_account_user_and_device() {
    let mut c = sqlite::open(std::path::Path::new(":memory:")).unwrap();
    let s = scope();
    seed(&mut c, &s);
    let mut others = Vec::new();
    let mut other = s.clone();
    other.account_id = "019a1234-0000-7000-8000-000000000011"
        .to_owned()
        .try_into()
        .unwrap();
    others.push(other);
    let mut other = s.clone();
    other.user_id = "019a1234-0000-7000-8000-000000000012"
        .to_owned()
        .try_into()
        .unwrap();
    others.push(other);
    let mut other = s.clone();
    other.device_id = "019a1234-0000-7000-8000-000000000013"
        .to_owned()
        .try_into()
        .unwrap();
    others.push(other);
    for other in &others {
        assert!(read_profiles(&c, other).unwrap().is_empty());
        assert!(read_identifiers(&c, other).unwrap().is_empty());
        assert!(read_contacts(&c, other).unwrap().is_empty());
        assert!(read_relationships(&c, other).unwrap().is_empty());
        assert!(read_local_changes(&c, other).unwrap().is_empty());
        seed(&mut c, other);
        let tx = c.transaction().unwrap();
        save_profiles(
            &tx,
            other,
            &ProfileRecord {
                full_name: Some("Otra cuenta ficticia".into()),
                version: 2,
                ..profile()
            },
        )
        .unwrap();
        tx.commit().unwrap();
        assert_rows(&c, &s);
    }
    let mut new_epoch = s.clone();
    new_epoch.permission_epoch = 2;
    assert_rows(&c, &new_epoch);
}

#[test]
fn deleting_accepted_views_keeps_intentions_and_other_scopes() {
    let mut c = sqlite::open(std::path::Path::new(":memory:")).unwrap();
    let s = scope();
    let mut other = s.clone();
    other.account_id = "019a1234-0000-7000-8000-000000000011"
        .to_owned()
        .try_into()
        .unwrap();
    seed(&mut c, &s);
    seed(&mut c, &other);
    let tx = c.transaction().unwrap();
    for table in [
        "patient_profile_view",
        "patient_identifier_view",
        "contact_person_view",
        "patient_contact_view",
    ] {
        tx.execute(
            &format!("DELETE FROM {table} WHERE account_id=?1 AND user_id=?2 AND device_id=?3"),
            params![
                s.account_id.as_str(),
                s.user_id.as_str(),
                s.device_id.as_str()
            ],
        )
        .unwrap();
    }
    tx.commit().unwrap();
    assert!(read_profiles(&c, &s).unwrap().is_empty());
    assert!(read_identifiers(&c, &s).unwrap().is_empty());
    assert!(read_contacts(&c, &s).unwrap().is_empty());
    assert!(read_relationships(&c, &s).unwrap().is_empty());
    assert_eq!(read_local_changes(&c, &s).unwrap(), [change()]);
    assert_rows(&c, &other);
}

#[test]
fn compound_relationship_key_allows_shared_contacts_without_overwrite() {
    let mut c = sqlite::open(std::path::Path::new(":memory:")).unwrap();
    let s = scope();
    seed(&mut c, &s);
    let second = RelationshipRecord {
        patient_id: "019a1234-0000-7000-8000-000000000008".into(),
        ..relationship()
    };
    let tx = c.transaction().unwrap();
    save_relationships(&tx, &s, &second).unwrap();
    tx.commit().unwrap();
    assert_eq!(
        read_relationships(&c, &s).unwrap(),
        [relationship(), second]
    );
}

#[test]
fn repeated_or_old_accepted_versions_do_not_replace_current_rows() {
    let mut c = sqlite::open(std::path::Path::new(":memory:")).unwrap();
    let s = scope();
    seed(&mut c, &s);
    let tx = c.transaction().unwrap();
    save_profiles(
        &tx,
        &s,
        &ProfileRecord {
            version: 2,
            ..profile()
        },
    )
    .unwrap();
    save_identifiers(
        &tx,
        &s,
        &IdentifierRecord {
            version: 2,
            ..identifier()
        },
    )
    .unwrap();
    save_contacts(
        &tx,
        &s,
        &ContactRecord {
            version: 2,
            ..contact()
        },
    )
    .unwrap();
    save_relationships(
        &tx,
        &s,
        &RelationshipRecord {
            version: 2,
            ..relationship()
        },
    )
    .unwrap();
    for version in [2, 1] {
        save_profiles(
            &tx,
            &s,
            &ProfileRecord {
                full_name: Some("Obsoleto".into()),
                version,
                ..profile()
            },
        )
        .unwrap();
        save_identifiers(
            &tx,
            &s,
            &IdentifierRecord {
                document_number: "Obsoleto".into(),
                version,
                ..identifier()
            },
        )
        .unwrap();
        save_contacts(
            &tx,
            &s,
            &ContactRecord {
                full_name: "Obsoleto".into(),
                version,
                ..contact()
            },
        )
        .unwrap();
        save_relationships(
            &tx,
            &s,
            &RelationshipRecord {
                relationship_kind: "other".into(),
                version,
                ..relationship()
            },
        )
        .unwrap();
    }
    tx.commit().unwrap();
    assert_eq!(
        read_profiles(&c, &s).unwrap(),
        [ProfileRecord {
            version: 2,
            ..profile()
        }]
    );
    assert_eq!(
        read_identifiers(&c, &s).unwrap(),
        [IdentifierRecord {
            version: 2,
            ..identifier()
        }]
    );
    assert_eq!(
        read_contacts(&c, &s).unwrap(),
        [ContactRecord {
            version: 2,
            ..contact()
        }]
    );
    assert_eq!(
        read_relationships(&c, &s).unwrap(),
        [RelationshipRecord {
            version: 2,
            ..relationship()
        }]
    );
}

#[test]
fn failed_write_rolls_back_all_records_in_the_transaction() {
    let mut c = sqlite::open(std::path::Path::new(":memory:")).unwrap();
    let s = scope();
    {
        let tx = c.transaction().unwrap();
        save_profiles(&tx, &s, &profile()).unwrap();
        save_contacts(&tx, &s, &contact()).unwrap();
        assert!(
            insert_local_change(
                &tx,
                &s,
                &LocalChange {
                    depends_on: "invalid".into(),
                    ..change()
                }
            )
            .is_err()
        );
    }
    assert!(read_profiles(&c, &s).unwrap().is_empty());
    assert!(read_contacts(&c, &s).unwrap().is_empty());
    assert!(read_local_changes(&c, &s).unwrap().is_empty());
}

#[test]
fn local_sequence_keeps_base_dependencies_and_exact_payloads() {
    let mut c = sqlite::open(std::path::Path::new(":memory:")).unwrap();
    let s = scope();
    let next = LocalChange {
        operation_id: "019a1234-0000-7000-8000-000000000008".into(),
        expected_version: 2,
        base_operation_id: Some(OPERATION.into()),
        depends_on: json!([OPERATION]).to_string(),
        payload: r#"{"phone":null,"phoneCode":null}"#.into(),
        created_at: 2,
        ..change()
    };
    let tx = c.transaction().unwrap();
    insert_local_change(&tx, &s, &change()).unwrap();
    insert_local_change(&tx, &s, &next).unwrap();
    assert!(insert_local_change(&tx, &s, &change()).is_err());
    tx.commit().unwrap();
    assert_eq!(read_local_changes(&c, &s).unwrap(), [change(), next]);
}

#[test]
fn constraints_reject_invalid_states_keys_versions_and_json() {
    let mut c = sqlite::open(std::path::Path::new(":memory:")).unwrap();
    let s = scope();
    let tx = c.transaction().unwrap();
    for record in [
        ProfileRecord {
            version: 0,
            ..profile()
        },
        ProfileRecord {
            status: "draft".into(),
            ..profile()
        },
        ProfileRecord {
            completion_status: "active".into(),
            ..profile()
        },
        ProfileRecord {
            phone_code: Some("+58".into()),
            ..profile()
        },
    ] {
        assert!(save_profiles(&tx, &s, &record).is_err());
    }
    assert!(
        save_identifiers(
            &tx,
            &s,
            &IdentifierRecord {
                verification_status: "client_verified".into(),
                ..identifier()
            }
        )
        .is_err()
    );
    assert!(
        save_contacts(
            &tx,
            &s,
            &ContactRecord {
                document_number: Some("001".into()),
                ..contact()
            }
        )
        .is_err()
    );
    assert!(
        save_relationships(
            &tx,
            &s,
            &RelationshipRecord {
                representation_status: "declared".into(),
                ..relationship()
            }
        )
        .is_err()
    );
    for record in [
        LocalChange {
            patient_id: None,
            ..change()
        },
        LocalChange {
            action: "CreateAppointment".into(),
            ..change()
        },
        LocalChange {
            expected_version: -1,
            ..change()
        },
        LocalChange {
            payload: "[]".into(),
            ..change()
        },
        LocalChange {
            draft: "null".into(),
            ..change()
        },
        LocalChange {
            depends_on: json!([1, 2, 3, 4, 5, 6, 7, 8, 9]).to_string(),
            ..change()
        },
    ] {
        assert!(insert_local_change(&tx, &s, &record).is_err());
    }
}
