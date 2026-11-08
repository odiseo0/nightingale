//! Opt-in consumer checks against the development API. Synthetic data only.
//!
//! Real-run command (from the repository root, with the development API and a
//! browser login available):
//!
//! ```powershell
//! $env:NIGHTINGALE_API_BASE_URL = "<url del entorno de desarrollo>"
//! cargo test --workspace --locked -- --ignored components::scheduling::development_tests
//! ```
//!
//! A blocking run never counts as success; keep this task pending until the
//! command above passes and record its output in the plan evidence. What this
//! probe cannot force against the real API (for example a lost HTTP response)
//! stays an explicit limit instead of a simulated claim.
use crate::components::services::application::CatalogQuery as _CatalogPort;
use crate::components::{
    appointments::application::ScheduleOccupancyQuery as _OccupancyPort,
    devices::activation::{DeviceCredential, DeviceCredentials},
    identity::{
        api::{DeviceRegistration, IdentityAPI},
        application::AccountActivation,
        domain::AccessRole,
        oidc::{LoginCancellation, OidcProvider, SystemBrowser},
    },
    projector::DesktopProjector,
    scheduling::{
        adapters::online::OnlineScheduling,
        adapters::outbound::{KnownOccupancy, SqliteSchedulingRepository},
        application::{
            AvailabilityRequest, ScheduleAvailabilityQuery, SchedulingMutation,
            SchedulingRepository, check_remote_availability,
        },
        domain::{BlockValue, ExceptionKind, ExceptionValue, Instant, ReasonCode, RuleValue},
    },
    sync::coordinator::{CycleOutcome, CycleTrigger, SyncCoordinator, SyncEngine},
};
use crate::platform::{config::APIConfig, http::APITransport, secrets::testing::MemorySecretStore};
use crate::shared_kernel::{SystemClock, UuidV7Generator, id::AccountId};
use std::str::FromStr;

struct TestDatabase {
    data: std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
    path: std::path::PathBuf,
}

impl TestDatabase {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "nightingale-scheduling-dev-{}.sqlite",
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
#[ignore = "creates synthetic schedule data in development; requires NIGHTINGALE_API_BASE_URL and browser login"]
fn manual_t22_scheduling_availability_with_real_api() {
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
            format!("scheduling-a-{}", uuid::Uuid::now_v7()),
            format!("scheduling-b-{}", uuid::Uuid::now_v7()),
        ];
        let secrets_a = MemorySecretStore::default();
        let registration = DeviceRegistration {
            device_name: "Scheduling synthetic probe".into(),
            client_version: config.client_version().into(),
            local_schema_version: 9,
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

        // The accepted professional comes from the real catalog download.
        let coordinator = SyncCoordinator::default();
        let engine = SyncEngine {
            coordinator: &coordinator,
            http: &http,
            database: &database_a.data,
        };
        cycle(&engine, &cred_a, &access_a.context).await;
        let professional = {
            let mut c = database_a.data.lock().unwrap();
            let rows = crate::components::services::adapters::outbound::SqliteCatalogRepository {
                connection: &mut c,
            }
            .list_professionals(&access_a.context)
            .unwrap();
            assert_eq!(
                rows.len(),
                1,
                "one own professional in the development account"
            );
            rows[0].professional_id.clone()
        };

        // Offline writes: rule, exception and account + global blocks.
        let rule_id = {
            let mut c = database_a.data.lock().unwrap();
            let mut repo = SqliteSchedulingRepository { connection: &mut c };
            let rule = repo
                .persist(
                    &access_a,
                    SchedulingMutation::CreateRule {
                        value: RuleValue::new(
                            professional.as_str().to_owned().try_into().unwrap(),
                            3,
                            480,
                            720,
                            crate::components::scheduling::domain::parse_local_date("2026-01-01")
                                .unwrap(),
                            None,
                        )
                        .unwrap(),
                    },
                    &UuidV7Generator,
                    &SystemClock,
                    &KnownOccupancy(vec![]),
                )
                .unwrap();
            repo.persist(
                &access_a,
                SchedulingMutation::CreateException {
                    value: ExceptionValue::new(
                        professional.as_str().to_owned().try_into().unwrap(),
                        crate::components::scheduling::domain::parse_local_date("2027-01-06")
                            .unwrap(),
                        ExceptionKind::Replacement,
                        Some(600),
                        Some(660),
                    )
                    .unwrap(),
                },
                &UuidV7Generator,
                &SystemClock,
                &KnownOccupancy(vec![]),
            )
            .unwrap();
            let scope = Some(account.as_str().to_owned().try_into().unwrap());
            repo.persist(
                &access_a,
                SchedulingMutation::CreateBlock {
                    value: BlockValue::new(
                        professional.as_str().to_owned().try_into().unwrap(),
                        scope,
                        Instant::parse("2027-02-01T13:00:00Z").unwrap(),
                        Instant::parse("2027-02-01T17:00:00Z").unwrap(),
                        ReasonCode::Administrative,
                    )
                    .unwrap(),
                },
                &UuidV7Generator,
                &SystemClock,
                &KnownOccupancy(vec![]),
            )
            .unwrap();
            repo.persist(
                &access_a,
                SchedulingMutation::CreateBlock {
                    value: BlockValue::new(
                        professional.as_str().to_owned().try_into().unwrap(),
                        None,
                        Instant::parse("2027-03-01T13:00:00Z").unwrap(),
                        Instant::parse("2027-03-01T17:00:00Z").unwrap(),
                        ReasonCode::Vacation,
                    )
                    .unwrap(),
                },
                &UuidV7Generator,
                &SystemClock,
                &KnownOccupancy(vec![]),
            )
            .unwrap();
            rule.entity_key.as_str().unwrap().to_owned()
        };

        // Dispatch + download: acceptance retires provisionals; API revalidates.
        cycle(&engine, &cred_a, &access_a.context).await;
        {
            let mut c = database_a.data.lock().unwrap();
            let row = SqliteSchedulingRepository { connection: &mut c }
                .rule_row(&access_a, &rule_id)
                .unwrap();
            assert!(
                row.provisional.is_none(),
                "accepted and downloaded provisional retires"
            );
        }

        // Local availability with coverage from the real downloaded window.
        let local = {
            let mut c = database_a.data.lock().unwrap();
            let known: Vec<_> =
                crate::components::appointments::adapters::outbound::SqliteScheduleOccupancyQuery {
                    connection: &c,
                    scope: &access_a.context,
                }
                .list_occupancy(
                    &professional,
                    jiff::Timestamp::from_str("2026-12-30T04:00:00Z").unwrap(),
                    jiff::Timestamp::from_str("2027-01-03T04:00:00Z").unwrap(),
                )
                .unwrap()
                .into_iter()
                .map(|interval| (professional.clone(), interval))
                .collect();
            SqliteSchedulingRepository { connection: &mut c }.local_availability(
                &access_a,
                &AvailabilityRequest {
                    professional_id: professional.clone(),
                    service_id: String::new(), // resolved below from the catalog
                    from: jiff::Timestamp::from_str("2026-12-30T04:00:00Z").unwrap(),
                    until: jiff::Timestamp::from_str("2027-01-03T04:00:00Z").unwrap(),
                    duration_minutes: None,
                    exclude_appointment_id: None,
                },
                &KnownOccupancy(known),
                &crate::components::sync::adapters::control::SqliteSyncControlStore,
            )
        };
        let local = match local {
            Ok(evaluation) => evaluation,
            Err(error) if error.code == "NOT_FOUND" => {
                // No service offered yet in a fresh development account: offer
                // one through the catalog and retry in a future real run.
                panic!("ofrezca un servicio en la cuenta de desarrollo antes de T22: {error:?}");
            }
            Err(error) => panic!("local availability failed: {error:?}"),
        };
        assert!(local.provisional);

        // Explicit remote availability against the real API.
        let remote = {
            let service = {
                let mut c = database_a.data.lock().unwrap();
                crate::components::services::adapters::outbound::SqliteCatalogRepository {
                    connection: &mut c,
                }
                .list_services(&access_a.context)
                .unwrap()
                .first()
                .map(|service| service.service_id.clone())
                .expect("al menos un servicio aceptado en la cuenta de desarrollo")
            };
            let remote = OnlineScheduling {
                http: &http,
                token: &cred_a.token,
                scope: &access_a.context,
            };
            let request = AvailabilityRequest {
                professional_id: professional.clone(),
                service_id: service.clone(),
                from: local_from(),
                until: local_until(),
                duration_minutes: None,
                exclude_appointment_id: None,
            };
            let gate = {
                let gate_c = database_a.data.lock().unwrap();
                crate::components::scheduling::adapters::desktop::SessionGate::lock(
                    &gate_c, &access_a,
                )
                .unwrap()
            };
            let result = check_remote_availability(&remote, &access_a, &request).await;
            {
                let gate_c = database_a.data.lock().unwrap();
                gate.verify(&gate_c).unwrap();
            }
            result.unwrap()
        };
        assert_eq!(remote.origin, "remote");
    });
}

fn local_from() -> jiff::Timestamp {
    jiff::Timestamp::from_str("2026-12-30T04:00:00Z").unwrap()
}

fn local_until() -> jiff::Timestamp {
    jiff::Timestamp::from_str("2027-01-03T04:00:00Z").unwrap()
}
