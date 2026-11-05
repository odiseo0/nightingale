use super::{
    coordinator::{CycleOutcome, CycleTrigger, SyncCoordinator, SyncEngine},
    download::tests::{Database, credential, cursor, group, scope},
    remote::{ChangeGroup, ChangeKind, EntityType, SyncChange},
    snapshot::{SnapshotMode, SnapshotOutcome, SnapshotRecovery},
};
use crate::{
    components::{
        patients::{
            adapters::outbound::{
                SqlitePatientRepository, read_contacts, read_profiles, read_relationships,
            },
            application::{EditBase, PatientUseCases},
            domain::{
                CompletionStatus, ContactData, ContactPatch, FieldChange, RelationshipData,
                RelationshipKind,
            },
            test_support::{access, seed_access},
        },
        projector::DesktopProjector,
    },
    platform::{config::APIConfig, http::APITransport, sqlite::open, test_api::TestApi},
    shared_kernel::{SystemClock, UuidV7Generator},
};
use rusqlite::Connection;
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

fn count(c: &Connection, table: &str) -> i64 {
    c.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

fn accepted_changes(
    patient: &str,
    contact: &str,
    profile_version: i64,
    contact_version: i64,
) -> Vec<SyncChange> {
    let s = scope();
    let mut changes: Vec<_> = group()
        .changes
        .into_iter()
        .filter(|c| {
            matches!(
                c.entity_type,
                EntityType::Account | EntityType::Professional
            )
        })
        .collect();
    changes.push(SyncChange {
        entity_type: EntityType::PatientProfile,
        entity_key: json!(patient),
        entity_version: profile_version,
        change_kind: ChangeKind::Upsert,
        state: json!({"accountId":s.account_id,"patientId":patient,"fullName":"Menor remoto","birthDate":"2018-02-28","phoneCode":null,"phone":null,"identificationBasis":"minor_without_document","completionStatus":"complete","status":"active","version":profile_version,"createdAt":null,"updatedAt":null}),
    });
    changes.push(SyncChange {
        entity_type: EntityType::ContactPerson,
        entity_key: json!(contact),
        entity_version: contact_version,
        change_kind: ChangeKind::Upsert,
        state: json!({"accountId":s.account_id,"id":contact,"fullName":"Responsable remoto","phoneCode":"+58","phone":"4120000000","issuingCountry":null,"documentType":null,"documentNumber":null,"status":"active","version":contact_version,"createdAt":null,"updatedAt":null}),
    });
    changes.push(SyncChange {
        entity_type: EntityType::PatientContact,
        entity_key: json!({"patientId":patient,"contactPersonId":contact}),
        entity_version: 1,
        change_kind: ChangeKind::Upsert,
        state: json!({"accountId":s.account_id,"patientId":patient,"contactPersonId":contact,"relationshipKind":"mother","isResponsibleContact":true,"representationStatus":"unverified","verifiedAt":null,"verifiedByUserId":null,"validFrom":"2026-01-01T08:00:00-04:00","validUntil":null,"status":"active","version":1,"createdAt":null,"updatedAt":null}),
    });
    changes
}

fn group_of(changes: Vec<SyncChange>) -> ChangeGroup {
    ChangeGroup {
        group_id: uuid::Uuid::now_v7().to_string().try_into().unwrap(),
        operation_id: uuid::Uuid::now_v7().to_string().try_into().unwrap(),
        accepted_at: "2026-11-05T12:00:00Z".into(),
        changes,
    }
}

fn entity_version_for(action: &str) -> i64 {
    match action {
        "RegisterPatient"
        | "CreateContactPerson"
        | "CreatePatientContactRelationship"
        | "AddPatientIdentifier" => 1,
        _ => 2,
    }
}

#[test]
fn offline_register_responsible_cycle_conflict_remove_and_snapshot_over_http() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        let access = access();
        let (patient_id, contact_id) = {
            let mut c = db.data.lock().unwrap();
            seed_access(&mut c, &access);
            let ids = &UuidV7Generator;
            let clock = &SystemClock;
            let mut repo = SqlitePatientRepository { connection: &mut c };
            let cases = &mut PatientUseCases {
                repository: &mut repo,
                access: &access,
                ids,
                clock,
            };
            let registered = cases
                .register_patient(crate::components::patients::domain::ProfileData {
                    full_name: Some("Menor offline".into()),
                    birth_date: Some("2018-02-28".into()),
                    phone: None,
                    identification_basis: Some(
                        crate::components::patients::domain::IdentificationBasis::MinorWithoutDocument,
                    ),
                })
                .unwrap();
            let patient = registered.entity_key.as_str().unwrap().to_owned();
            let created = cases
                .create_contact_person(
                    ContactData::new(
                        "Responsable offline".into(),
                        Some("+58".into()),
                        Some("4120000000".into()),
                        None,
                        None,
                        None,
                    )
                    .unwrap(),
                )
                .unwrap();
            let contact = created.entity_key.as_str().unwrap().to_owned();
            cases
                .create_patient_contact_relationship(
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
                .unwrap();
            let profile_base = EditBase::LocalOperation {
                operation_id: registered.operation_id.clone().try_into().unwrap(),
            };
            cases
                .set_patient_completion(
                    patient.clone().try_into().unwrap(),
                    profile_base,
                    CompletionStatus::Complete,
                )
                .unwrap();
            assert!(read_profiles(&c, &scope()).unwrap().is_empty());
            assert_eq!(count(&c, "patient_local"), 4);
            (patient, contact)
        };

        // Reopen before any HTTP: operations and provisional data are durable.
        {
            let c = open(&db.path).unwrap();
            assert_eq!(count(&c, "sync_outbox"), 4);
            assert_eq!(count(&c, "patient_local"), 4);
        }

        let stage = Arc::new(AtomicUsize::new(0));
        let server_stage = stage.clone();
        let allowed = access.allowed_actions.clone();
        let data = db.data.clone();
        let server_patient = patient_id.clone();
        let server_contact = contact_id.clone();
        let api = TestApi::dynamic(move |request| {
            assert!(data.try_lock().is_ok(), "HTTP must not hold SQLite");
            let stage = server_stage.load(Ordering::SeqCst);
            if request.contains("/access ") {
                return (
                    200,
                    json!({"accountId":scope().account_id,"userId":scope().user_id,"deviceId":scope().device_id,"permissionEpoch":1,"role":"assistant","status":"active","allowedActions":allowed}),
                );
            }
            if request.contains("/sync/operations ") {
                let body: Value =
                    serde_json::from_str(request.split_once("\r\n\r\n").unwrap().1).unwrap();
                let results: Vec<_> = body["operations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|operation| {
                        let accepted = stage == 0;
                        json!({"operationId":operation["operationId"],"status":if accepted {"accepted"} else {"conflict"},"entityVersion":if accepted {json!(entity_version_for(operation["action"].as_str().unwrap()))} else {Value::Null},"state":null,"error":if accepted {Value::Null} else {json!({"code":"VERSION_CONFLICT","correlationId":"019a1234-0000-7000-8000-000000000099"})}})
                    })
                    .collect();
                return (
                    200,
                    json!({"correlationId":"019a1234-0000-7000-8000-000000000099","results":results}),
                );
            }
            if request.contains("/sync/changes?") {
                let changes = match stage {
                    0 => accepted_changes(&server_patient, &server_contact, 2, 1),
                    1 => accepted_changes(&server_patient, &server_contact, 3, 1)
                        .into_iter()
                        .filter(|c| c.entity_type == EntityType::PatientProfile)
                        .collect(),
                    2 => {
                        let mut removal = SyncChange {
                            entity_type: EntityType::ContactPerson,
                            entity_key: json!(server_contact),
                            entity_version: 2,
                            change_kind: ChangeKind::Remove,
                            state: Value::Null,
                        };
                        removal.entity_version = 2;
                        vec![removal]
                    }
                    _ => panic!("unexpected download"),
                };
                return (
                    200,
                    json!({"groups":[group_of(changes)],"nextCursor":format!("stage-{stage}"),"hasMore":false}),
                );
            }
            if request.starts_with("DELETE ") {
                return (204, Value::Null);
            }
            if request.contains("/items?") {
                let items: Vec<_> = accepted_changes(&server_patient, &server_contact, 3, 2)
                    .into_iter()
                    .map(|change| json!({"entityType":change.entity_type,"entityKey":change.entity_key,"entityVersion":change.entity_version,"state":change.state}))
                    .collect();
                return (200, json!({"items":items,"total":items.len()}));
            }
            assert!(request.contains("/sync/snapshots"));
            (
                200,
                json!({"snapshotId":"019a1234-0000-7000-8000-000000000300","accountId":scope().account_id,"contractVersion":1,"modelRevision":"api-v1.1","window":cursor("base").window,"baseCursor":"fresh-base","expiresAt":"2099-11-05T23:00:00Z","itemCount":5}),
            )
        });
        let config = APIConfig::parse(&api.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&api.url).unwrap()).unwrap();
        let credential = credential(&config);
        let coordinator = SyncCoordinator::default();
        let engine = SyncEngine {
            coordinator: &coordinator,
            http: &http,
            database: &db.data,
        };
        let report = engine
            .run(
                &credential,
                &scope(),
                CycleTrigger::Manual,
                &SystemClock,
                &UuidV7Generator,
                &DesktopProjector,
            )
            .await
            .unwrap();
        assert_eq!(report.outcome, CycleOutcome::Complete);
        {
            let mut c = db.data.lock().unwrap();
            let s = scope();
            let profile = &read_profiles(&c, &s).unwrap()[0];
            assert_eq!(
                (profile.version, profile.completion_status.as_str()),
                (2, "complete")
            );
            assert_eq!(profile.full_name.as_deref(), Some("Menor remoto"));
            assert_eq!(read_contacts(&c, &s).unwrap()[0].version, 1);
            assert_eq!(read_relationships(&c, &s).unwrap()[0].version, 1);
            // Projectors must not resolve intentions; receipts stay in the outbox.
            assert_eq!(count(&c, "patient_local"), 4);
            assert_eq!(count(&c, "sync_outbox"), 4);
            let row = {
                let mut repo = SqlitePatientRepository { connection: &mut c };
                PatientUseCases {
                    repository: &mut repo,
                    access: &access,
                    ids: &UuidV7Generator,
                    clock: &SystemClock,
                }
                .get_patient(&patient_id.to_owned().try_into().unwrap())
                .unwrap()
            };
            assert!(
                row.provisional.is_none(),
                "accepted and downloaded provisional retires"
            );
            assert_eq!(row.accepted.as_ref().unwrap().version, 2);
        }

        // A second device accepted a newer profile; the local contact edit conflicts.
        let conflict_id = {
            let mut c = db.data.lock().unwrap();
            let mut repo = SqlitePatientRepository { connection: &mut c };
            PatientUseCases {
                repository: &mut repo,
                access: &access,
                ids: &UuidV7Generator,
                clock: &SystemClock,
            }
            .update_contact_person(
                contact_id.to_owned().try_into().unwrap(),
                EditBase::Accepted { version: 1 },
                ContactPatch {
                    phone: FieldChange::Set("4129999999".into()),
                    ..ContactPatch::default()
                },
            )
            .unwrap()
            .operation_id
        };
        stage.store(1, Ordering::SeqCst);
        assert_eq!(
            engine
                .run(
                    &credential,
                    &scope(),
                    CycleTrigger::Manual,
                    &SystemClock,
                    &UuidV7Generator,
                    &DesktopProjector
                )
                .await
                .unwrap()
                .outcome,
            CycleOutcome::Complete
        );
        {
            let mut c = db.data.lock().unwrap();
            let row = {
                let mut repo = SqlitePatientRepository { connection: &mut c };
                PatientUseCases {
                    repository: &mut repo,
                    access: &access,
                    ids: &UuidV7Generator,
                    clock: &SystemClock,
                }
                .get_contact(&contact_id.to_owned().try_into().unwrap())
                .unwrap()
            };
            let provisional = row.provisional.as_ref().unwrap();
            assert_eq!(provisional.state.phone.as_deref(), Some("4129999999"));
            let conflict = row
                .operations
                .iter()
                .find(|o| o.operation_id == conflict_id)
                .unwrap();
            assert_eq!(conflict.status, "conflict");
            assert_eq!(conflict.error.as_deref(), Some("VERSION_CONFLICT"));
            assert_eq!(read_profiles(&c, &scope()).unwrap()[0].version, 3);
        }

        // A removal from another device clears the accepted view but keeps work.
        stage.store(2, Ordering::SeqCst);
        super::download::ChangeDownloader {
            http: &http,
            database: &db.data,
        }
        .download(&credential, &scope(), &SystemClock, &DesktopProjector)
        .await
        .unwrap();
        {
            let c = open(&db.path).unwrap();
            assert!(read_contacts(&c, &scope()).unwrap().is_empty());
            assert_eq!(
                count(&c, "patient_local"),
                5,
                "removal keeps local intentions"
            );
        }

        // A fresh snapshot rebuilds accepted views without deleting local work.
        stage.store(3, Ordering::SeqCst);
        let result = SnapshotRecovery {
            http: &http,
            database: &db.data,
        }
        .rebuild(
            &credential,
            &scope(),
            &SystemClock,
            SnapshotMode::Restart,
            &DesktopProjector,
        )
        .await
        .unwrap();
        assert!(matches!(
            result,
            SnapshotOutcome::Installed {
                cleanup_pending: false
            }
        ));
        {
            let c = open(&db.path).unwrap();
            let s = scope();
            assert_eq!(read_profiles(&c, &s).unwrap()[0].version, 3);
            assert_eq!(read_contacts(&c, &s).unwrap()[0].version, 2);
            assert_eq!(count(&c, "patient_local"), 5);
            assert_eq!(count(&c, "sync_outbox"), 5);
        }
        assert_eq!(db.cursor().cursor, "fresh-base");
    });
}
