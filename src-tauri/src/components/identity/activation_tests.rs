use super::*;
use crate::platform::http::APITransport;
use crate::platform::secrets::{CredentialKey, CredentialKind, Secret, testing::MemorySecretStore};
use crate::platform::test_api::TestApi;
use crate::shared_kernel::{SystemClock, UuidV7Generator};
use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

const ACCOUNT: &str = "019a1234-0000-7000-8000-000000000001";
const USER: &str = "019a1234-0000-7000-8000-000000000002";
const DEVICE: &str = "019a1234-0000-7000-8000-000000000003";
const CORRELATION: &str = "019a1234-0000-7000-8000-000000000004";
const OTHER: &str = "019a1234-0000-7000-8000-000000000009";
const ISSUER: &str = "https://issuer.example/";
const SUBJECT: &str = "test-user";
const API_TOKEN: &str = "synthetic-api-device-secret";
const OIDC_TOKEN: &str = "synthetic-oidc-access";

struct Database(PathBuf);

impl Database {
    fn new() -> Self {
        Self(std::env::temp_dir().join(format!(
            "nightingale-activation-{}.sqlite",
            uuid::Uuid::now_v7()
        )))
    }

    fn open(&self) -> Arc<Mutex<Connection>> {
        Arc::new(Mutex::new(crate::platform::sqlite::open(&self.0).unwrap()))
    }
}
impl Drop for Database {
    fn drop(&mut self) {
        if self.0.exists() {
            fs::remove_file(&self.0).unwrap();
        }
    }
}

#[derive(Default)]
struct AuditStore {
    inner: MemorySecretStore,
    writes: Mutex<Vec<CredentialKey>>,
    fail: Mutex<Option<CredentialKey>>,
}

impl SecretStore for AuditStore {
    fn read(&self, key: &CredentialKey) -> crate::shared_kernel::Result<Option<Secret>> {
        self.inner.read(key)
    }

    fn write(&self, key: &CredentialKey, value: &Secret) -> crate::shared_kernel::Result<()> {
        self.writes.lock().unwrap().push(key.clone());

        if self.fail.lock().unwrap().as_ref() == Some(key) {
            return Err(NightingaleError::temporary("fallo de prueba"));
        }

        self.inner.write(key, value)
    }

    fn delete(&self, key: &CredentialKey) -> crate::shared_kernel::Result<()> {
        self.inner.delete(key)
    }
}
impl AuditStore {
    fn token_key(config: &APIConfig) -> CredentialKey {
        let device = crate::shared_kernel::id::DeviceId::try_from(DEVICE.to_owned()).unwrap();

        CredentialKey::new(
            config,
            "test-profile",
            (ISSUER, SUBJECT),
            None,
            Some(&device),
            CredentialKind::APIDevice,
        )
        .unwrap()
    }

    fn session(&self, config: &APIConfig) -> OidcSession {
        let key = CredentialKey::new(
            config,
            "test-profile",
            (ISSUER, SUBJECT),
            None,
            None,
            CredentialKind::OidcAccess,
        )
        .unwrap();
        self.write(&key, &Secret::new(OIDC_TOKEN.into()).unwrap())
            .unwrap();

        OidcSession::load(config, "test-profile", (ISSUER, SUBJECT), self)
            .unwrap()
            .unwrap()
    }

    fn token_writes(&self, config: &APIConfig) -> usize {
        self.writes
            .lock()
            .unwrap()
            .iter()
            .filter(|key| **key == Self::token_key(config))
            .count()
    }
}

fn account() -> AccountId {
    AccountId::try_from(ACCOUNT.to_owned()).unwrap()
}

fn issue() -> Value {
    json!({"accessToken":API_TOKEN,"userId":USER,"deviceId":DEVICE,"tokenId":CORRELATION,"issuedAt":"2026-10-29T12:00:00Z"})
}

fn me(subject: &str) -> Value {
    json!({"userId":USER,"status":"active","displayName":"Test user","oidcIssuer":ISSUER,"oidcSubject":subject,"accounts":{"items":[],"total":0}})
}

fn accounts(id: &str, total: u64) -> Value {
    json!({"items":[{"id":id,"displayName":"Test practice","role":"assistant","status":"active"}],"total":total})
}

fn access(status: &str) -> Value {
    json!({"accountId":ACCOUNT,"userId":USER,"deviceId":DEVICE,"role":"assistant","status":status,"permissionEpoch":3,"allowedActions":["RegisterPatient","UpdateAccountPolicy","RegisterAccountDevice"]})
}

fn error(code: &str) -> Value {
    json!({"code":code,"message":API_TOKEN,"correlationId":CORRELATION})
}

fn path(request: &str) -> &str {
    request
        .lines()
        .next()
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap()
}

fn body(request: &str) -> Value {
    serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap()
}

fn result(request: &str, status: &str, duplicate: bool) -> Value {
    let operation = body(request)["operations"][0]["operationId"].clone();
    let record = json!({"operationId":operation,"status":status,"entityVersion":if status=="accepted"{json!(1)}else{Value::Null},
        "state":if status=="accepted"{json!({"deviceId":DEVICE,"displayName":"Test device","status":"active","clientVersion":"0.1.0","localSchemaVersion":4,"lastSeenAt":null,"version":1})}else{Value::Null},
        "error":if status=="accepted"{Value::Null}else{error("PERMISSION_DENIED")}});
    let outer = if duplicate {
        json!({"operationId":operation,"status":"duplicate","entityVersion":null,"state":null,"error":null,"originalResult":record})
    } else {
        record
    };

    json!({"results":[outer],"correlationId":CORRELATION})
}

fn engine<'a>(
    config: &'a APIConfig,
    http: &'a APITransport,
    secrets: &'a AuditStore,
    database: &'a Mutex<Connection>,
) -> AccountActivation<'a, AuditStore> {
    AccountActivation {
        config,
        profile: "test-profile",
        http,
        secrets,
        database,
    }
}

fn seed_intention(database: &Mutex<Connection>) -> String {
    let connection = database.lock().unwrap();
    connection.execute("INSERT INTO sync_outbox (operation_id,contract_version,action,entity_type,entity_key,expected_version,payload,status,created_at)
        VALUES (?1,1,'RegisterPatient','patient_profile',?2,0,?3,'pending',42)",rusqlite::params![OTHER,json!(OTHER).to_string(),r#"{"fullName":"Saved offline"}"#]).unwrap();

    connection
        .query_row(
            "SELECT payload FROM sync_outbox WHERE operation_id=?1",
            [OTHER],
            |row| row.get(0),
        )
        .unwrap()
}

fn assert_intention(database: &Mutex<Connection>, expected: &str) {
    let value: (String, String, Option<String>) = database
        .lock()
        .unwrap()
        .query_row(
            "SELECT payload,status,account_id FROM sync_outbox WHERE operation_id=?1",
            [OTHER],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(value, (expected.into(), "pending".into(), None));
}

#[test]
fn activation_registers_only_self_reuses_secret_and_survives_reopen() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        let database = db.open();
        let expected = seed_intention(&database);
        let post_count = Arc::new(AtomicUsize::new(0));
        let posts = post_count.clone();
        let data = database.clone();
        let server = TestApi::new(9, move |request| {
            assert!(
                data.try_lock().is_ok(),
                "una petición mantiene el mutex SQLite"
            );
            let endpoint = path(request);

            if endpoint == "/v1/auth/device-tokens" {
                assert!(
                    request
                        .to_ascii_lowercase()
                        .contains(&format!("authorization: bearer {OIDC_TOKEN}"))
                );
                assert_eq!(body(request)["localSchemaVersion"], 10);

                return (201, issue());
            }
            assert!(
                request
                    .to_ascii_lowercase()
                    .contains(&format!("authorization: bearer {API_TOKEN}"))
            );

            if endpoint == "/v1/me" {
                return (200, me(SUBJECT));
            }

            if endpoint.starts_with("/v1/accounts?") {
                return (200, accounts(ACCOUNT, 1));
            }

            if endpoint.ends_with("/access") {
                return (
                    200,
                    access(if posts.load(Ordering::Relaxed) == 0 {
                        "registration_required"
                    } else {
                        "active"
                    }),
                );
            }
            assert!(endpoint.ends_with("/sync/operations"));
            let payload = body(request);
            assert_eq!(payload["operations"].as_array().unwrap().len(), 1);
            let operation = &payload["operations"][0];
            assert_eq!(operation["action"], "RegisterAccountDevice");
            assert_eq!(operation["entityKey"], DEVICE);
            assert_eq!(operation["expectedVersion"], 0);
            assert_eq!(operation["payload"], json!({}));
            assert_eq!(operation["dependsOn"], json!([]));
            assert!(operation.get("accountId").is_none());
            posts.fetch_add(1, Ordering::Relaxed);

            (200, result(request, "accepted", false))
        });
        let config = APIConfig::parse(&server.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
        let store = AuditStore::default();
        let session = store.session(&config);
        let access = engine(&config, &http, &store, &database)
            .activate(&session, &account(), &UuidV7Generator, &SystemClock)
            .await
            .unwrap();
        assert_eq!(access.status, AccessStatus::Active);
        assert!(access.allows_offline("RegisterPatient"));
        assert!(!access.allows_offline("UpdateAccountPolicy"));
        assert!(!access.allows_offline("RegisterAccountDevice"));
        assert!(!access.allows_offline("UnknownAction"));
        assert_intention(&database, &expected);
        drop(database);
        let database = db.open();
        let service = engine(&config, &http, &store, &database);
        assert!(
            service
                .cached_access((ISSUER, SUBJECT), &account())
                .unwrap()
                .unwrap()
                .allows_offline("RegisterPatient")
        );
        let access = service
            .activate(&session, &account(), &UuidV7Generator, &SystemClock)
            .await
            .unwrap();
        assert_eq!(access.status, AccessStatus::Active);
        assert_eq!(store.token_writes(&config), 1);
        assert_eq!(post_count.load(Ordering::Relaxed), 1);
        let requests = server.finish();
        assert_eq!(
            requests
                .iter()
                .filter(|request| path(request) == "/v1/auth/device-tokens")
                .count(),
            1
        );
        service.logout((ISSUER, SUBJECT)).unwrap();
        assert!(
            service
                .cached_access((ISSUER, SUBJECT), &account())
                .unwrap()
                .is_none()
        );
        assert!(
            store
                .read(&AuditStore::token_key(&config))
                .unwrap()
                .is_none()
        );
        assert_intention(&database, &expected);
        drop(database);
        let bytes = fs::read(&db.0).unwrap();

        for secret in [API_TOKEN, OIDC_TOKEN] {
            assert!(
                !bytes
                    .windows(secret.len())
                    .any(|window| window == secret.as_bytes())
            );
        }
    });
}

#[test]
fn lost_registration_response_reuses_same_id_and_fingerprint_after_restart() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        let database = db.open();
        let expected = seed_intention(&database);
        let posts = Arc::new(AtomicUsize::new(0));
        let post_counter = posts.clone();
        let server = TestApi::new(10, move |request| {
            let endpoint = path(request);

            if endpoint == "/v1/auth/device-tokens" {
                return (201, issue());
            }

            if endpoint == "/v1/me" {
                return (200, me(SUBJECT));
            }

            if endpoint.starts_with("/v1/accounts?") {
                return (200, accounts(ACCOUNT, 1));
            }

            if endpoint.ends_with("/access") {
                return (
                    200,
                    access(if post_counter.load(Ordering::Relaxed) < 2 {
                        "registration_required"
                    } else {
                        "active"
                    }),
                );
            }

            let attempt = post_counter.fetch_add(1, Ordering::Relaxed);

            if attempt == 0 {
                (0, Value::Null)
            } else {
                (200, result(request, "accepted", true))
            }
        });

        let config = APIConfig::parse(&server.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
        let store = AuditStore::default();
        let session = store.session(&config);

        assert!(matches!(
            engine(&config, &http, &store, &database)
                .activate(&session, &account(), &UuidV7Generator, &SystemClock)
                .await,
            Err(ActivationFailure::Remote(_))
        ));
        let prior:(String,String,String)=database.lock().unwrap().query_row("SELECT operation_id,semantic_fingerprint,status FROM sync_outbox WHERE action='RegisterAccountDevice'",[],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?))).unwrap();
        assert_eq!(prior.2, "unknown");
        drop(database);
        let database = db.open();
        assert_eq!(
            engine(&config, &http, &store, &database)
                .activate(&session, &account(), &UuidV7Generator, &SystemClock)
                .await
                .unwrap()
                .status,
            AccessStatus::Active
        );
        let final_row:(String,String,String,i64)=database.lock().unwrap().query_row("SELECT operation_id,semantic_fingerprint,status,attempts FROM sync_outbox WHERE action='RegisterAccountDevice'",[],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?))).unwrap();
        assert_eq!(final_row, (prior.0, prior.1, "accepted".into(), 2));
        assert_eq!(store.token_writes(&config), 1);
        assert_intention(&database, &expected);
        let requests = server.finish();
        let bodies: Vec<_> = requests
            .iter()
            .filter(|request| path(request).ends_with("/sync/operations"))
            .map(|request| body(request))
            .collect();
        assert_eq!(bodies.len(), 2);
        assert_eq!(bodies[0], bodies[1]);
    });
}

#[test]
fn revoked_and_inactive_access_stop_registration_and_keep_outbox() {
    tauri::async_runtime::block_on(async {
        for status in ["revoked", "inactive"] {
            let db = Database::new();
            let database = db.open();
            let expected = seed_intention(&database);
            let server = TestApi::new(4, move |request| match path(request) {
                "/v1/auth/device-tokens" => (201, issue()),
                "/v1/me" => (200, me(SUBJECT)),
                endpoint if endpoint.starts_with("/v1/accounts?") => (200, accounts(ACCOUNT, 1)),
                endpoint if endpoint.ends_with("/access") => (200, access(status)),
                _ => panic!("se intentó registrar o restaurar acceso denegado"),
            });
            let config = APIConfig::parse(&server.url).unwrap();
            let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
            let store = AuditStore::default();
            let session = store.session(&config);
            let service = engine(&config, &http, &store, &database);
            let current = service
                .activate(&session, &account(), &UuidV7Generator, &SystemClock)
                .await
                .unwrap();
            assert!(!current.allows_offline("RegisterPatient"));
            assert_eq!(
                service
                    .cached_access((ISSUER, SUBJECT), &account())
                    .unwrap()
                    .unwrap()
                    .status,
                current.status
            );
            assert_intention(&database, &expected);
            assert_eq!(server.finish().len(), 4);
        }
    });
}

#[test]
fn online_denial_invalidates_cached_grants_without_deleting_intentions() {
    tauri::async_runtime::block_on(async {
        for status in [401, 403] {
            let db = Database::new();
            let database = db.open();
            let expected = seed_intention(&database);
            let denied = Arc::new(AtomicBool::new(false));
            let deny = denied.clone();
            let count = if status == 401 { 5 } else { 7 };
            let server = TestApi::new(count, move |request| {
                let endpoint = path(request);

                if endpoint == "/v1/auth/device-tokens" {
                    return (201, issue());
                }

                if endpoint == "/v1/me" {
                    return if deny.load(Ordering::Relaxed) && status == 401 {
                        (401, error("AUTHENTICATION_REQUIRED"))
                    } else {
                        (200, me(SUBJECT))
                    };
                }

                if endpoint.starts_with("/v1/accounts?") {
                    return (200, accounts(ACCOUNT, 1));
                }

                if endpoint.ends_with("/access") {
                    return if deny.load(Ordering::Relaxed) {
                        (403, error("PERMISSION_DENIED"))
                    } else {
                        (200, access("active"))
                    };
                }

                panic!("petición inesperada");
            });

            let config = APIConfig::parse(&server.url).unwrap();
            let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
            let store = AuditStore::default();
            let session = store.session(&config);
            let service = engine(&config, &http, &store, &database);
            assert!(
                service
                    .activate(&session, &account(), &UuidV7Generator, &SystemClock)
                    .await
                    .unwrap()
                    .allows_offline("RegisterPatient")
            );
            denied.store(true, Ordering::Relaxed);
            let failure = service
                .activate(&session, &account(), &UuidV7Generator, &SystemClock)
                .await
                .err()
                .unwrap();
            assert!(!format!("{failure:?}").contains(API_TOKEN));
            assert!(
                !service
                    .cached_access((ISSUER, SUBJECT), &account())
                    .unwrap()
                    .unwrap()
                    .allows_offline("RegisterPatient")
            );
            assert_intention(&database, &expected);
            assert_eq!(store.token_writes(&config), 1);
            server.finish();
        }
    });
}

#[test]
fn one_time_issuance_is_not_repeated_after_lost_response_or_secret_write_failure() {
    tauri::async_runtime::block_on(async {
        for lost in [true, false] {
            let db = Database::new();
            let database = db.open();
            let server = TestApi::new(1, move |request| {
                assert_eq!(path(request), "/v1/auth/device-tokens");

                if lost {
                    (0, Value::Null)
                } else {
                    (201, issue())
                }
            });
            let config = APIConfig::parse(&server.url).unwrap();
            let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
            let store = AuditStore::default();
            let session = store.session(&config);

            if !lost {
                *store.fail.lock().unwrap() = Some(AuditStore::token_key(&config));
            }

            assert!(
                engine(&config, &http, &store, &database)
                    .activate(&session, &account(), &UuidV7Generator, &SystemClock)
                    .await
                    .is_err()
            );

            drop(database);

            let database = db.open();
            let failure = engine(&config, &http, &store, &database)
                .activate(&session, &account(), &UuidV7Generator, &SystemClock)
                .await
                .err()
                .unwrap();
            assert!(matches!(
                failure,
                ActivationFailure::Credential(CredentialFailure::RecoveryRequired)
            ));
            assert_eq!(store.token_writes(&config), usize::from(!lost));
            server.finish();
        }
    });
}

#[test]
fn duplicate_rejection_stays_rejected_and_never_gets_a_new_operation_id() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        let database = db.open();
        let server = TestApi::new(8, move |request| match path(request) {
            "/v1/auth/device-tokens" => (201, issue()),
            "/v1/me" => (200, me(SUBJECT)),
            endpoint if endpoint.starts_with("/v1/accounts?") => (200, accounts(ACCOUNT, 1)),
            endpoint if endpoint.ends_with("/access") => (200, access("registration_required")),
            endpoint if endpoint.ends_with("/sync/operations") => {
                (200, result(request, "rejected", true))
            }
            _ => panic!("petición inesperada"),
        });
        let config = APIConfig::parse(&server.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
        let store = AuditStore::default();
        let session = store.session(&config);
        let service = engine(&config, &http, &store, &database);
        assert!(matches!(
            service
                .activate(&session, &account(), &UuidV7Generator, &SystemClock)
                .await,
            Err(ActivationFailure::RegistrationFailed {
                status: OperationStatus::Rejected,
                ..
            })
        ));
        assert!(matches!(
            service
                .activate(&session, &account(), &UuidV7Generator, &SystemClock)
                .await,
            Err(ActivationFailure::RegistrationReview)
        ));
        assert_eq!(
            database
                .lock()
                .unwrap()
                .query_row(
                    "SELECT count(*) FROM sync_outbox WHERE action='RegisterAccountDevice'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        server.finish();
    });
}

#[test]
fn rejects_identity_and_account_scope_mismatches_before_registration() {
    tauri::async_runtime::block_on(async {
        for wrong_identity in [true, false] {
            let db = Database::new();
            let database = db.open();
            let expected = seed_intention(&database);
            let count = if wrong_identity { 2 } else { 4 };
            let server = TestApi::new(count, move |request| match path(request) {
                "/v1/auth/device-tokens" => (201, issue()),
                "/v1/me" => (
                    200,
                    me(if wrong_identity {
                        "other-user"
                    } else {
                        SUBJECT
                    }),
                ),
                endpoint if endpoint.starts_with("/v1/accounts?") => (200, accounts(ACCOUNT, 1)),
                endpoint if endpoint.ends_with("/access") => {
                    let mut response = access("registration_required");
                    response["accountId"] = json!(OTHER);
                    (200, response)
                }
                _ => panic!("se intentó registrar un alcance ajeno"),
            });
            let config = APIConfig::parse(&server.url).unwrap();
            let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
            let store = AuditStore::default();
            let session = store.session(&config);
            let service = engine(&config, &http, &store, &database);
            assert!(matches!(
                service
                    .activate(&session, &account(), &UuidV7Generator, &SystemClock)
                    .await,
                Err(ActivationFailure::IdentityMismatch)
            ));
            assert!(
                service
                    .cached_access((ISSUER, SUBJECT), &account())
                    .unwrap()
                    .is_none()
            );
            assert_intention(&database, &expected);
            server.finish();
        }
    });
}

#[test]
fn account_selection_reads_later_pages_and_rejects_missing_membership() {
    tauri::async_runtime::block_on(async {
        for visible in [true, false] {
            let db = Database::new();
            let database = db.open();
            let server = TestApi::new(if visible { 5 } else { 3 }, move |request| {
                match path(request) {
                    "/v1/auth/device-tokens" => (201, issue()),
                    "/v1/me" => (200, me(SUBJECT)),
                    "/v1/accounts?page=1&shows=100" => {
                        if visible {
                            (200, accounts(OTHER, 2))
                        } else {
                            (200, json!({"items":[],"total":0}))
                        }
                    }
                    "/v1/accounts?page=2&shows=100" => (200, accounts(ACCOUNT, 2)),
                    endpoint if endpoint.ends_with("/access") => (200, access("active")),
                    _ => panic!("petición fuera de la cuenta visible"),
                }
            });
            let config = APIConfig::parse(&server.url).unwrap();
            let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
            let store = AuditStore::default();
            let session = store.session(&config);
            let service = engine(&config, &http, &store, &database);
            let result = service
                .activate(&session, &account(), &UuidV7Generator, &SystemClock)
                .await;
            if visible {
                assert!(result.unwrap().allows_offline("RegisterPatient"));
            } else {
                assert!(matches!(result, Err(ActivationFailure::AccountUnavailable)));
            }
            server.finish();
        }
    });
}

#[test]
fn logout_during_activation_reports_busy_and_does_not_remove_the_token() {
    tauri::async_runtime::block_on(async {
        let db = Database::new();
        let database = db.open();
        let store = Arc::new(AuditStore::default());
        let secrets = store.clone();
        let server = TestApi::new(4, move |request| match path(request) {
            "/v1/auth/device-tokens" => (201, issue()),
            "/v1/me" => {
                let host = request
                    .lines()
                    .filter_map(|line| line.split_once(':'))
                    .find(|(name, _)| name.eq_ignore_ascii_case("host"))
                    .unwrap()
                    .1
                    .trim();
                let config = APIConfig::parse(&format!("http://{host}/")).unwrap();
                let credentials = DeviceCredentials {
                    config: &config,
                    profile: "test-profile",
                    identity: (ISSUER, SUBJECT),
                    secrets: &*secrets,
                };
                assert!(matches!(credentials.forget(), Err(CredentialFailure::Busy)));

                (200, me(SUBJECT))
            }
            endpoint if endpoint.starts_with("/v1/accounts?") => (200, accounts(ACCOUNT, 1)),
            endpoint if endpoint.ends_with("/access") => (200, access("active")),
            _ => panic!("petición inesperada"),
        });
        let config = APIConfig::parse(&server.url).unwrap();
        let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
        let session = store.session(&config);
        let service = engine(&config, &http, &store, &database);
        assert_eq!(
            service
                .activate(&session, &account(), &UuidV7Generator, &SystemClock)
                .await
                .unwrap()
                .status,
            AccessStatus::Active
        );
        service.logout((ISSUER, SUBJECT)).unwrap();
        assert!(
            store
                .read(&AuditStore::token_key(&config))
                .unwrap()
                .is_none()
        );
        server.finish();
    });
}

#[test]
#[ignore = "requires development API/OIDC, native secret store, browser and an account ID"]
fn manual_c02_activation_and_local_logout() {
    tauri::async_runtime::block_on(async {
        use crate::{
            components::identity::oidc::{LoginCancellation, OidcProvider, SystemBrowser},
            platform::secrets::NativeSecretStore,
        };
        let config = APIConfig::from_environment().unwrap();
        let account = AccountId::try_from(
            std::env::var("NIGHTINGALE_TEST_ACCOUNT_ID")
                .expect("falta el ID no secreto de la cuenta de desarrollo"),
        )
        .unwrap();
        let http =
            APITransport::new(APIConfig::parse(config.base_url().as_str()).unwrap()).unwrap();
        let auth = IdentityAPI(&http).auth_config().await.unwrap().value;
        let provider = OidcProvider::discover(&auth, &http).await.unwrap();
        let attempt = provider.begin(&SystemBrowser).unwrap();
        let session = provider
            .complete(attempt, LoginCancellation::default(), &http)
            .await
            .unwrap();
        let db = Database::new();
        let database = db.open();
        let store = NativeSecretStore;
        let profile = format!("manual-c02-{}", uuid::Uuid::now_v7());
        let service = AccountActivation {
            config: &config,
            profile: &profile,
            http: &http,
            secrets: &store,
            database: &database,
        };
        let result = service
            .activate(&session, &account, &UuidV7Generator, &SystemClock)
            .await;
        let cleanup = service.logout((session.issuer(), session.subject()));
        cleanup.unwrap();
        let access = result.unwrap();
        assert_eq!(access.status, AccessStatus::Active);
        println!(
            "Dispositivo de desarrollo creado: {}",
            access.context.device_id.as_str()
        );
        assert!(
            service
                .cached_access((session.issuer(), session.subject()), &account)
                .unwrap()
                .is_none()
        );
        // Only the device ID is printed. The server device remains for manual review.
    });
}
