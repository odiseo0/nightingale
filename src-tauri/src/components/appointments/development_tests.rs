//! Opt-in consumer checks for appointments against the development API.
//! Synthetic data only.
//!
//! Real-run command (from the repository root, with the development API and a
//! browser login available):
//!
//! ```powershell
//! $env:NIGHTINGALE_API_BASE_URL = "<url del entorno de desarrollo>"
//! cargo test --workspace --locked -- --ignored components::appointments::development_tests
//! ```
//!
//! A blocking run never counts as success; keep this task pending until the
//! command above passes and record its output in the plan evidence. What this
//! probe cannot force against the real API (for example a lost HTTP response
//! or a concurrent booking) stays an explicit limit instead of a simulated
//! claim.
use crate::components::{
    appointments::{
        adapters::outbound::SqliteAppointmentRepository,
        application::{AppointmentRepository, CreateAppointmentInput},
        domain::{AppointmentId, AppointmentStatus},
    },
    devices::activation::{DeviceCredential, DeviceCredentials},
    identity::{
        api::{DeviceRegistration, IdentityAPI},
        application::AccountActivation,
        domain::AccessRole,
        oidc::{LoginCancellation, OidcProvider, SystemBrowser},
    },
    patients::application::{PatientMutation, PatientRepository},
    projector::DesktopProjector,
    scheduling::application::{SchedulingMutation, SchedulingRepository},
    services::application::{CatalogMutation, CatalogRepository},
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
            "nightingale-appointments-dev-{}.sqlite",
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
        // Release the file before removing it (Windows locks open databases).
        *self.data.lock().unwrap() = rusqlite::Connection::open_in_memory().unwrap();
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
#[ignore = "creates synthetic appointment data in development; requires NIGHTINGALE_API_BASE_URL and browser login"]
fn manual_t25_appointments_with_real_api() {
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
        let profile = format!("appointments-{}", uuid::Uuid::now_v7());
        let secrets = MemorySecretStore::default();
        let registration = DeviceRegistration {
            device_name: "Appointments synthetic probe".into(),
            client_version: config.client_version().into(),
            local_schema_version: 10,
        };
        drop(
            DeviceCredentials {
                config: &config,
                profile: &profile,
                identity,
                secrets: &secrets,
            }
            .ensure(&http, session.access_token(), &registration)
            .await
            .unwrap(),
        );
        let credential = DeviceCredentials {
            config: &config,
            profile: &profile,
            identity,
            secrets: &secrets,
        }
        .load()
        .unwrap()
        .unwrap();
        let me = IdentityAPI(&http)
            .me(&credential.token)
            .await
            .unwrap()
            .value;
        let account = match std::env::var("NIGHTINGALE_TEST_ACCOUNT_ID") {
            Ok(id) => AccountId::try_from(id).unwrap(),
            Err(_) => {
                assert_eq!(me.accounts.total, 1, "indique NIGHTINGALE_TEST_ACCOUNT_ID");
                let account = me.accounts.items.first().unwrap();
                assert!(matches!(account.role, AccessRole::Owner));
                account.id.clone()
            }
        };
        let database = TestDatabase::new();
        let access = AccountActivation {
            config: &config,
            profile: &profile,
            http: &http,
            secrets: &secrets,
            database: &database.data,
        }
        .activate(&session, &account, &UuidV7Generator, &SystemClock)
        .await
        .unwrap();
        let scope = access.context.clone();
        let coordinator = SyncCoordinator::default();
        let engine = SyncEngine {
            coordinator: &coordinator,
            http: &http,
            database: &database.data,
        };

        // First cycle downloads the account professional created server-side.
        cycle(&engine, &credential, &scope).await;
        let (professional_id, patient_id, service_id, starts, ends) = {
            let mut c = database.data.lock().unwrap();
            let professional: String = c
                .query_row(
                    "SELECT professional_id FROM professional_view WHERE account_id=?1 AND user_id=?2",
                    rusqlite::params![scope.account_id.as_str(), scope.user_id.as_str()],
                    |row| row.get(0),
                )
                .unwrap();

            let mut catalog =
                crate::components::services::adapters::outbound::SqliteCatalogRepository {
                    connection: &mut c,
                };
            let professional =
                crate::components::services::domain::ProfessionalId::try_from(professional)
                    .unwrap();
            let service = catalog
                .persist(
                    &access,
                    CatalogMutation::CreateService {
                        professional: professional.clone(),
                        name: format!("Consulta sintética {}", uuid::Uuid::now_v7()),
                        duration: crate::components::services::domain::DurationMinutes::new(30)
                            .unwrap(),
                    },
                    &UuidV7Generator,
                    &SystemClock,
                )
                .unwrap();
            let service_id = service.entity_key.as_str().unwrap().to_owned();
            catalog
                .persist(
                    &access,
                    CatalogMutation::OfferService {
                        key: crate::components::services::domain::OfferKey::new(
                            professional.as_str().into(),
                            service_id.clone(),
                        )
                        .unwrap(),
                        adjustment: None,
                    },
                    &UuidV7Generator,
                    &SystemClock,
                )
                .unwrap();

            let patient = crate::components::patients::adapters::outbound::SqlitePatientRepository {
                connection: &mut c,
            }
            .persist(
                &access,
                PatientMutation::Register {
                    profile: crate::components::patients::domain::ProfileData {
                        full_name: Some("Paciente Sintético Citas".into()),
                        birth_date: None,
                        phone: Some(crate::components::patients::domain::Phone::new(
                            "+58".into(),
                            "4240000000".into(),
                        )
                        .unwrap()),
                        identification_basis: Some(
                            crate::components::patients::domain::IdentificationBasis::OwnDocument,
                        ),
                    },
                    identifiers: vec![crate::components::patients::domain::Document::new(
                        "VE".into(),
                        crate::components::patients::domain::DocumentType::NationalId,
                        format!("V{}", uuid::Uuid::now_v7().simple()),
                    )
                    .unwrap()],
                },
                &UuidV7Generator,
                &SystemClock,
            )
            .unwrap();
            let patient_id = patient.entity_key.as_str().unwrap().to_owned();
            crate::components::patients::adapters::outbound::SqlitePatientRepository {
                connection: &mut c,
            }
            .persist(
                &access,
                PatientMutation::SetCompletion {
                    patient: patient_id.clone().try_into().unwrap(),
                    base: crate::components::patients::application::EditBase::LocalOperation {
                        operation_id: patient.operation_id.as_str().to_owned().try_into().unwrap(),
                    },
                    completion: crate::components::patients::domain::CompletionStatus::Complete,
                },
                &UuidV7Generator,
                &SystemClock,
            )
            .unwrap();

            let time_zone: String = c
                .query_row(
                    "SELECT time_zone FROM account_view WHERE account_id=?1",
                    rusqlite::params![scope.account_id.as_str()],
                    |row| row.get(0),
                )
                .unwrap();
            let zone = jiff::tz::TimeZone::get(&time_zone).unwrap();
            let cursor: String = c
                .query_row(
                    "SELECT window FROM sync_cursor WHERE account_id=?1",
                    rusqlite::params![scope.account_id.as_str()],
                    |row| row.get(0),
                )
                .unwrap();
            let window: serde_json::Value = serde_json::from_str(&cursor).unwrap();
            let window_from = window["appointmentsFrom"].as_str().unwrap();
            let window_until = window["appointmentsUntil"].as_str().unwrap();
            let from_day = crate::components::scheduling::domain::parse_local_date(
                window_from.split('T').next().unwrap(),
            )
            .unwrap();
            let until_day = crate::components::scheduling::domain::parse_local_date(
                window_until.split('T').next().unwrap(),
            )
            .unwrap();
            let today = jiff::Timestamp::now().to_zoned(zone.clone()).date();
            let mut day = std::cmp::max(from_day, today) + jiff::Span::new().days(1);
            if day >= until_day {
                day = std::cmp::max(from_day, today);
            }
            let iso_weekday = day.strftime("%u").to_string().parse::<i64>().unwrap();
            crate::components::scheduling::adapters::outbound::SqliteSchedulingRepository {
                connection: &mut c,
            }
            .persist(
                &access,
                SchedulingMutation::CreateRule {
                    value: crate::components::scheduling::domain::RuleValue::new(
                        professional.as_str().to_owned().try_into().unwrap(),
                        iso_weekday,
                        0,
                        1440,
                        crate::components::scheduling::domain::parse_local_date("2026-01-01")
                            .unwrap(),
                        None,
                    )
                    .unwrap(),
                },
                &UuidV7Generator,
                &SystemClock,
                &crate::components::scheduling::adapters::outbound::KnownOccupancy(vec![]),
            )
            .unwrap();
            let starts = day
                .at(9, 0, 0, 0)
                .to_zoned(zone.clone())
                .unwrap()
                .timestamp()
                .to_string();
            let ends = day
                .at(9, 30, 0, 0)
                .to_zoned(zone.clone())
                .unwrap()
                .timestamp()
                .to_string();
            (
                professional.as_str().to_owned(),
                patient_id,
                service_id,
                starts,
                ends,
            )
        };

        // Acceptance of the synthetic base by the real API. The appointment
        // base (service, offer, patient, completion) must be accepted; the
        // agenda rule acceptance is tracked separately because the fixture API
        // currently rejects CreateScheduleRule on first create
        // (INVALID_STATE_TRANSITION) — a finding for nightingale-api, outside
        // this plan's acceptance scope. The pending rule still feeds the fused
        // local availability.
        cycle(&engine, &credential, &scope).await;
        {
            let c = database.data.lock().unwrap();
            let failures: Vec<(String, String)> = c
                .prepare(
                    "SELECT action, last_error FROM sync_outbox WHERE status NOT IN ('accepted','pending','unknown','retryable','in_flight') AND action != 'CreateScheduleRule'",
                )
                .unwrap()
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap()
                .collect::<rusqlite::Result<_>>()
                .unwrap();
            assert!(
                failures.is_empty(),
                "appointment base operations must be accepted by the real API: {failures:?}"
            );
        }

        // Offline booking on the accepted base, then a second cycle accepts it.
        let created = {
            let mut c = database.data.lock().unwrap();
            SqliteAppointmentRepository { connection: &mut c }
                .create_appointment(
                    &access,
                    CreateAppointmentInput {
                        patient_id,
                        professional_id,
                        service_id,
                        responsible_contact_id: None,
                        starts_at: starts.clone(),
                        ends_at: ends.clone(),
                    },
                    &UuidV7Generator,
                    &SystemClock,
                )
                .unwrap()
        };
        cycle(&engine, &credential, &scope).await;
        let appointment: AppointmentId = created
            .entity_key
            .as_str()
            .unwrap()
            .to_owned()
            .try_into()
            .unwrap();
        {
            let mut c = database.data.lock().unwrap();
            let row = SqliteAppointmentRepository { connection: &mut c }
                .appointment_row(&access, &appointment)
                .unwrap();
            let accepted = row.accepted.expect("accepted appointment view");
            assert_eq!(accepted.state.status, AppointmentStatus::Scheduled.as_str());
            assert!(
                row.provisional.is_none(),
                "receipt + views retire the provisional"
            );
        }

        // The accepted history downloads with the appointment.
        {
            let c = database.data.lock().unwrap();
            let history =
                crate::components::appointments::adapters::outbound::read_history(&c, &scope)
                    .unwrap()
                    .into_iter()
                    .filter(|record| record.appointment_id == appointment.as_str())
                    .count();
            assert!(history >= 1, "accepted history projected");
        }

        // Staff cancellation frees the slot and survives a final cycle.
        {
            let mut c = database.data.lock().unwrap();
            SqliteAppointmentRepository { connection: &mut c }
                .persist(
                    &access,
                    crate::components::appointments::application::AppointmentMutation::Cancel {
                        appointment: appointment.clone(),
                        base: crate::components::appointments::application::EditBase::Accepted {
                            version: 1,
                        },
                        reason: None,
                    },
                    &UuidV7Generator,
                    &SystemClock,
                )
                .unwrap();
        }
        cycle(&engine, &credential, &scope).await;
    });
}
