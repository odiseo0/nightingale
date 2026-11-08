//! Opt-in consumer checks against the development API. Synthetic data only.
//!
//! Real-run command (from the repository root, with the development API and a
//! browser login available):
//!
//! ```powershell
//! $env:NIGHTINGALE_API_BASE_URL = "<url del entorno de desarrollo>"
//! cargo test --workspace --locked -- --ignored components::patients::development_tests
//! ```
//!
//! A blocking run never counts as success; keep this task pending until the
//! command above passes and record its output in the plan evidence.
use crate::components::{
    devices::activation::{DeviceCredential, DeviceCredentials},
    identity::{
        api::{DeviceRegistration, IdentityAPI},
        application::AccountActivation,
        domain::AccessRole,
        oidc::{LoginCancellation, OidcProvider, SystemBrowser},
        repository::SqliteAccessStore,
    },
    patients::{
        adapters::outbound::SqlitePatientRepository,
        application::{EditBase, PatientUseCases},
        domain::{
            CompletionStatus, ContactData, FieldChange, IdentificationBasis, ProfileData,
            ProfilePatch, RelationshipData, RelationshipKind,
        },
    },
    projector::DesktopProjector,
    sync::coordinator::{CycleOutcome, CycleTrigger, SyncCoordinator, SyncEngine},
};
use crate::platform::{config::APIConfig, http::APITransport, secrets::testing::MemorySecretStore};
use crate::shared_kernel::{SystemClock, UuidV7Generator, id::AccountId};

struct TestDatabase {
    data: std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
    path: std::path::PathBuf,
}

impl TestDatabase {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "nightingale-patients-dev-{}.sqlite",
            uuid::Uuid::now_v7()
        ));
        let data = std::sync::Arc::new(std::sync::Mutex::new(
            crate::platform::sqlite::open(&path).unwrap(),
        ));
        Self { data, path }
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        if self.path.exists() {
            std::fs::remove_file(&self.path).unwrap();
        }
    }
}

async fn cycle(
    engine: &SyncEngine<'_>,
    credential: &DeviceCredential,
    scope: &crate::shared_kernel::id::OperationContext,
) {
    let report = engine
        .run(
            credential,
            scope,
            CycleTrigger::Manual,
            &SystemClock,
            &UuidV7Generator,
            &DesktopProjector,
        )
        .await
        .unwrap();
    assert_eq!(report.outcome, CycleOutcome::Complete);
}

#[test]
#[ignore = "creates a synthetic minor, contact and conflict in development; requires NIGHTINGALE_API_BASE_URL and browser login"]
fn manual_p17_responsible_contact_flow_with_two_devices() {
    tauri::async_runtime::block_on(async {
        let config = APIConfig::from_environment().unwrap();
        let http =
            APITransport::new(APIConfig::parse(config.base_url().as_str()).unwrap()).unwrap();
        let auth = IdentityAPI(&http).auth_config().await.unwrap().value;
        let provider = OidcProvider::discover(&auth, &http).await.unwrap();
        let attempt = provider.begin(&SystemBrowser).unwrap();
        let session = provider
            .complete(attempt, LoginCancellation::default(), &http)
            .await
            .unwrap();
        let identity = (session.issuer(), session.subject());
        let profiles = [
            format!("patients-a-{}", uuid::Uuid::now_v7()),
            format!("patients-b-{}", uuid::Uuid::now_v7()),
        ];
        let secrets_a = MemorySecretStore::default();
        let secrets_b = MemorySecretStore::default();
        let registration = DeviceRegistration {
            device_name: "Patients core synthetic probe".into(),
            client_version: config.client_version().into(),
            local_schema_version: 7,
        };
        drop(
            DeviceCredentials {
                config: &config,
                profile: &profiles[0],
                identity,
                secrets: &secrets_a,
            }
            .ensure(&http, session.access_token(), &registration)
            .await
            .unwrap(),
        );
        let cred_a = DeviceCredentials {
            config: &config,
            profile: &profiles[0],
            identity,
            secrets: &secrets_a,
        }
        .load()
        .unwrap()
        .unwrap();
        let me = IdentityAPI(&http).me(&cred_a.token).await.unwrap().value;
        let account = match std::env::var("NIGHTINGALE_TEST_ACCOUNT_ID") {
            Ok(id) => AccountId::try_from(id).unwrap(),
            Err(_) => {
                assert_eq!(me.accounts.total, 1, "indique NIGHTINGALE_TEST_ACCOUNT_ID");
                let account = me.accounts.items.first().unwrap();
                assert!(matches!(account.role, AccessRole::Owner));
                account.id.clone()
            }
        };
        let database_a = TestDatabase::new();
        let database_b = TestDatabase::new();
        let access_a = AccountActivation {
            config: &config,
            profile: &profiles[0],
            http: &http,
            secrets: &secrets_a,
            database: &database_a.data,
        }
        .activate(&session, &account, &UuidV7Generator, &SystemClock)
        .await
        .unwrap();
        let access_b = AccountActivation {
            config: &config,
            profile: &profiles[1],
            http: &http,
            secrets: &secrets_b,
            database: &database_b.data,
        }
        .activate(&session, &account, &UuidV7Generator, &SystemClock)
        .await
        .unwrap();
        let cred_b = DeviceCredentials {
            config: &config,
            profile: &profiles[1],
            identity,
            secrets: &secrets_b,
        }
        .load()
        .unwrap()
        .unwrap();
        let coordinator_a = SyncCoordinator::default();
        let coordinator_b = SyncCoordinator::default();
        let engine_a = SyncEngine {
            http: &http,
            database: &database_a.data,
            coordinator: &coordinator_a,
        };
        let engine_b = SyncEngine {
            http: &http,
            database: &database_b.data,
            coordinator: &coordinator_b,
        };
        cycle(&engine_a, &cred_a, &access_a.context).await;
        cycle(&engine_b, &cred_b, &access_b.context).await;
        // A second device can change the account's permission epoch.
        let access_a = SqliteAccessStore
            .selected_access(&database_a.data.lock().unwrap())
            .unwrap()
            .unwrap();
        let access_b = SqliteAccessStore
            .selected_access(&database_b.data.lock().unwrap())
            .unwrap()
            .unwrap();

        // Device A works offline: minor + responsible contact + completion.
        let (patient, contact, completed_name) = {
            let mut connection = database_a.data.lock().unwrap();
            let mut repository = SqlitePatientRepository {
                connection: &mut connection,
            };
            let cases = &mut PatientUseCases {
                repository: &mut repository,
                access: &access_a,
                ids: &UuidV7Generator,
                clock: &SystemClock,
            };
            let registered = cases
                .register_patient(ProfileData {
                    full_name: Some(format!("development-minor-{}", uuid::Uuid::now_v7())),
                    birth_date: Some("2018-02-28".into()),
                    phone: None,
                    identification_basis: Some(IdentificationBasis::MinorWithoutDocument),
                })
                .unwrap();
            let patient: crate::components::patients::domain::PatientId = registered
                .entity_key
                .as_str()
                .unwrap()
                .to_owned()
                .try_into()
                .unwrap();
            let created = cases
                .create_contact_person(
                    ContactData::new(
                        format!("development-responsible-{}", uuid::Uuid::now_v7()),
                        Some("+58".into()),
                        Some("4120000000".into()),
                        None,
                        None,
                        None,
                    )
                    .unwrap(),
                )
                .unwrap();
            let contact: crate::components::patients::domain::ContactPersonId = created
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
                    RelationshipData::new(
                        RelationshipKind::Mother,
                        true,
                        "2026-01-01T08:00:00-04:00".into(),
                        None,
                    )
                    .unwrap(),
                )
                .unwrap();
            let completed = cases
                .set_patient_completion(
                    patient.clone(),
                    EditBase::LocalOperation {
                        operation_id: linked.operation_id.clone().try_into().unwrap(),
                    },
                    CompletionStatus::Complete,
                )
                .unwrap();
            (patient, contact, completed.provisional["fullName"].clone())
        };
        assert_eq!(completed_name, completed_name.clone());

        // Sync A: accepted, then B downloads the accepted family.
        cycle(&engine_a, &cred_a, &access_a.context).await;
        {
            let mut connection = database_a.data.lock().unwrap();
            let row = PatientUseCases {
                repository: &mut SqlitePatientRepository {
                    connection: &mut connection,
                },
                access: &access_a,
                ids: &UuidV7Generator,
                clock: &SystemClock,
            }
            .get_patient(&patient)
            .unwrap();
            assert!(row.provisional.is_none(), "accepted and downloaded");
            assert_eq!(row.accepted.as_ref().unwrap().version, 2);
        }
        cycle(&engine_b, &cred_b, &access_b.context).await;
        {
            let mut connection = database_b.data.lock().unwrap();
            let row = PatientUseCases {
                repository: &mut SqlitePatientRepository {
                    connection: &mut connection,
                },
                access: &access_b,
                ids: &UuidV7Generator,
                clock: &SystemClock,
            }
            .get_patient(&patient)
            .unwrap();
            assert!(row.provisional.is_none());
            let contacts = PatientUseCases {
                repository: &mut SqlitePatientRepository {
                    connection: &mut connection,
                },
                access: &access_b,
                ids: &UuidV7Generator,
                clock: &SystemClock,
            }
            .get_contact(&contact)
            .unwrap();
            assert!(contacts.accepted.is_some());
        }

        // Both devices edit the same accepted profile; the late sender conflicts
        // and keeps its intention for review.
        let edit = |access: &crate::components::identity::domain::AccountAccess,
                    database: &TestDatabase,
                    name: &str| {
            let mut connection = database.data.lock().unwrap();
            PatientUseCases {
                repository: &mut SqlitePatientRepository {
                    connection: &mut connection,
                },
                access,
                ids: &UuidV7Generator,
                clock: &SystemClock,
            }
            .update_patient_profile(
                patient.clone(),
                EditBase::Accepted { version: 2 },
                ProfilePatch {
                    full_name: FieldChange::Set(name.into()),
                    ..ProfilePatch::default()
                },
            )
            .unwrap()
            .operation_id
        };
        let id_a = edit(&access_a, &database_a, "Nombre desde A");
        cycle(&engine_a, &cred_a, &access_a.context).await;
        let id_b = edit(&access_b, &database_b, "Nombre desde B");
        cycle(&engine_b, &cred_b, &access_b.context).await;
        {
            let mut connection = database_b.data.lock().unwrap();
            let row = PatientUseCases {
                repository: &mut SqlitePatientRepository {
                    connection: &mut connection,
                },
                access: &access_b,
                ids: &UuidV7Generator,
                clock: &SystemClock,
            }
            .get_patient(&patient)
            .unwrap();
            let status: Vec<_> = row
                .operations
                .iter()
                .filter(|o| o.operation_id == id_a || o.operation_id == id_b)
                .map(|o| (o.operation_id.clone(), o.status.clone()))
                .collect();
            assert_eq!(status.len(), 2, "both intentions stay visible");
            assert!(
                status
                    .iter()
                    .any(|(_, s)| s == "conflict" || s == "review_required"),
                "the late edit conflicts or requires review: {status:?}"
            );
            assert!(row.provisional.is_some(), "the losing intention is kept");
        }
    });
}
