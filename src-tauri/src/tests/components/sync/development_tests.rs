//! Opt-in consumer checks. They create synthetic data only in the selected account.
use crate::components::sync::{
    adapters::{
        api::{ResultStatus, SyncAPI},
        control::SqliteSyncControlStore,
        outbound::SqliteOperationStore,
    },
    application::{CursorStore, RemoteProjectionStore, record_operation},
    coordinator::{CycleOutcome, CycleTrigger, SyncCoordinator, SyncEngine},
    domain::{NewOperation, OperationEnvelope, OperationStatus},
    events::{Notice, Parser, StreamOutcome},
};
use crate::tests::components::sync::download_tests::Database;
use crate::{
    components::{
        devices::activation::{DeviceCredential, DeviceCredentials},
        identity::{
            api::{DeviceRegistration, IdentityAPI},
            application::AccountActivation,
            domain::AccessRole,
            oidc::{LoginCancellation, OidcProvider, SystemBrowser},
            repository::SqliteAccessStore,
        },
    },
    platform::{
        config::APIConfig, http::APITransport, secrets::testing::MemorySecretStore, sqlite::open,
    },
    shared_kernel::{
        Clock, SystemClock, UuidV7Generator,
        id::{AccountId, OperationContext, OperationId},
    },
};
use serde_json::{Value, json};

#[test]
#[ignore = "writes one synthetic offline operation to an explicitly selected desktop test database"]
fn manual_c05_save_offline_probe() {
    let path = std::path::PathBuf::from(
        std::env::var_os("NIGHTINGALE_TEST_DESKTOP_DB")
            .expect("indique la base del escritorio de prueba"),
    );
    assert!(path.is_absolute() && path.is_file());
    let expected_account = AccountId::try_from(
        std::env::var("NIGHTINGALE_TEST_ACCOUNT_ID").expect("indique la cuenta sintética"),
    )
    .unwrap();
    let mut connection = open(&path).unwrap();
    let access = SqliteAccessStore
        .selected_access(&connection)
        .unwrap()
        .unwrap();
    assert_eq!(access.context.account_id, expected_account);
    assert!(access.allows_offline("RegisterPatient"));
    let tx = connection.transaction().unwrap();
    let id = record_operation(
        &tx,
        &SqliteOperationStore,
        &UuidV7Generator,
        &SystemClock,
        NewOperation {
            context: access.context,
            depends_on: vec![],
            action: "RegisterPatient".into(),
            entity_type: "patient_profile".into(),
            entity_key: json!(uuid::Uuid::now_v7().to_string()),
            expected_version: 0,
            payload: json!({"fullName":"Synthetic desktop offline checkpoint patient"}),
            occurred_at_local: None,
        },
    )
    .unwrap();
    tx.commit().unwrap();
    println!("Sonda local pendiente guardada: {id}");
}

fn record(
    db: &Database,
    context: &OperationContext,
    patient: &Value,
    action: &str,
    version: i64,
    payload: Value,
) -> OperationId {
    let mut connection = db.data.lock().unwrap();
    let tx = connection.transaction().unwrap();
    let id = record_operation(
        &tx,
        &SqliteOperationStore,
        &UuidV7Generator,
        &SystemClock,
        NewOperation {
            context: context.clone(),
            depends_on: vec![],
            action: action.into(),
            entity_type: "patient_profile".into(),
            entity_key: patient.clone(),
            expected_version: version,
            payload,
            occurred_at_local: None,
        },
    )
    .unwrap();
    tx.commit().unwrap();
    OperationId::try_from(id.to_string()).unwrap()
}

async fn cycle(engine: &SyncEngine<'_>, credential: &DeviceCredential, scope: &OperationContext) {
    let report = engine
        .run(
            credential,
            scope,
            CycleTrigger::Manual,
            &SystemClock,
            &UuidV7Generator,
            &(),
        )
        .await
        .unwrap();
    assert_eq!(report.outcome, CycleOutcome::Complete);
    assert!(report.status.cursor_ready);
}

#[test]
#[ignore = "creates a synthetic patient and two devices in development; requires browser login"]
fn manual_c06_two_profiles_with_development_api() {
    tauri::async_runtime::block_on(async {
        let config = APIConfig::from_environment().unwrap();
        assert!(
            config
                .base_url()
                .host_str()
                .is_some_and(|host| matches!(host, "127.0.0.1" | "localhost" | "[::1]")),
            "solo entorno local de prueba"
        );
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
        let a = Database::new();
        let b = Database::new();
        for db in [&a, &b] {
            db.data
                .lock()
                .unwrap()
                .execute_batch("DELETE FROM identity_account_access; DELETE FROM sync_cursor;")
                .unwrap();
        }
        let secrets_a = MemorySecretStore::default();
        let secrets_b = MemorySecretStore::default();
        let profiles = [
            format!("development-A-{}", uuid::Uuid::now_v7()),
            format!("development-B-{}", uuid::Uuid::now_v7()),
        ];
        let credentials_a = DeviceCredentials {
            config: &config,
            profile: &profiles[0],
            identity,
            secrets: &secrets_a,
        };
        let credentials_b = DeviceCredentials {
            config: &config,
            profile: &profiles[1],
            identity,
            secrets: &secrets_b,
        };
        let registration = DeviceRegistration {
            device_name: "Core checkpoint synthetic probe".into(),
            client_version: config.client_version().into(),
            local_schema_version: 4,
        };
        drop(
            credentials_a
                .ensure(&http, session.access_token(), &registration)
                .await
                .unwrap(),
        );
        let cred_a = credentials_a.load().unwrap().unwrap();
        let me = IdentityAPI(&http).me(&cred_a.token).await.unwrap().value;
        let account = match std::env::var("NIGHTINGALE_TEST_ACCOUNT_ID") {
            Ok(id) => AccountId::try_from(id).unwrap(),
            Err(_) => {
                assert_eq!(
                    me.accounts.total, 1,
                    "indique NIGHTINGALE_TEST_ACCOUNT_ID si hay varias cuentas"
                );
                let account = me.accounts.items.first().unwrap();
                assert!(matches!(account.role, AccessRole::Owner));
                account.id.clone()
            }
        };
        println!("Cuenta sintética seleccionada: {}", account.as_str());
        let activation_a = AccountActivation {
            config: &config,
            profile: &profiles[0],
            http: &http,
            secrets: &secrets_a,
            database: &a.data,
        };
        let activation_b = AccountActivation {
            config: &config,
            profile: &profiles[1],
            http: &http,
            secrets: &secrets_b,
            database: &b.data,
        };
        let access_a = activation_a
            .activate(&session, &account, &UuidV7Generator, &SystemClock)
            .await
            .unwrap();
        let access_b = activation_b
            .activate(&session, &account, &UuidV7Generator, &SystemClock)
            .await
            .unwrap();
        let cred_b = credentials_b.load().unwrap().unwrap();
        assert_ne!(cred_a.device_id, cred_b.device_id);
        let ca = SyncCoordinator::default();
        let cb = SyncCoordinator::default();
        let ea = SyncEngine {
            http: &http,
            database: &a.data,
            coordinator: &ca,
        };
        let eb = SyncEngine {
            http: &http,
            database: &b.data,
            coordinator: &cb,
        };
        cycle(&ea, &cred_a, &access_a.context).await;
        cycle(&eb, &cred_b, &access_b.context).await;
        // The second registration can change the account's permission epoch.
        let access_a = SqliteAccessStore
            .selected_access(&a.data.lock().unwrap())
            .unwrap()
            .unwrap();
        let access_b = SqliteAccessStore
            .selected_access(&b.data.lock().unwrap())
            .unwrap()
            .unwrap();

        let patient = json!(uuid::Uuid::now_v7().to_string());
        let id = record(
            &a,
            &access_a.context,
            &patient,
            "RegisterPatient",
            0,
            json!({"fullName":"Synthetic Core checkpoint patient"}),
        );
        *a.data.lock().unwrap() = open(&a.path).unwrap();
        assert_eq!(
            SqliteOperationStore
                .operation_result(&a.data.lock().unwrap(), &access_a.context, &id)
                .unwrap()
                .unwrap()
                .status,
            OperationStatus::Pending
        );

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let offline_url = format!("http://{}/", listener.local_addr().unwrap());
        drop(listener);
        let offline_http = APITransport::new(APIConfig::parse(&offline_url).unwrap()).unwrap();
        let offline = SyncEngine {
            http: &offline_http,
            database: &a.data,
            coordinator: &ca,
        };
        assert_eq!(
            offline
                .run(
                    &cred_a,
                    &access_a.context,
                    CycleTrigger::Manual,
                    &SystemClock,
                    &UuidV7Generator,
                    &()
                )
                .await
                .unwrap()
                .outcome,
            CycleOutcome::Offline
        );

        // Deliberately discard the confirmed response before local reconciliation.
        let now = SystemClock
            .now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        let lease = {
            let mut connection = a.data.lock().unwrap();
            let access = SqliteAccessStore
                .selected_access(&connection)
                .unwrap()
                .unwrap();
            SqliteOperationStore.lease_batch(&mut connection, &access, now, 0)
        }
        .unwrap()
        .unwrap();
        let envelopes = lease.operations();
        assert_eq!(envelopes.len(), 1);
        let accepted = SyncAPI(&http)
            .send_operations(&cred_a.token, &account, &envelopes)
            .await
            .unwrap();
        assert_eq!(accepted.value.results[0].status, ResultStatus::Accepted);
        drop(accepted);
        *a.data.lock().unwrap() = open(&a.path).unwrap();
        assert_eq!(
            SqliteOperationStore
                .operation_result(&a.data.lock().unwrap(), &access_a.context, &id)
                .unwrap()
                .unwrap()
                .status,
            OperationStatus::Unknown
        );
        cycle(&ea, &cred_a, &access_a.context).await;
        assert_eq!(
            SqliteOperationStore
                .operation_result(&a.data.lock().unwrap(), &access_a.context, &id)
                .unwrap()
                .unwrap()
                .status,
            OperationStatus::Accepted
        );
        let duplicate = SyncAPI(&http)
            .send_operations(&cred_b.token, &account, &envelopes)
            .await
            .unwrap();
        assert_eq!(duplicate.value.results[0].status, ResultStatus::Duplicate);
        cycle(&eb, &cred_b, &access_b.context).await;
        for (db, scope) in [(&a, &access_a.context), (&b, &access_b.context)] {
            let item = SqliteSyncControlStore
                .item(&db.data.lock().unwrap(), scope, "patient_profile", &patient)
                .unwrap()
                .unwrap();
            assert_eq!(item.entity_version, 1);
            assert_eq!(item.state["fullName"], "Synthetic Core checkpoint patient");
        }

        record(
            &a,
            &access_a.context,
            &patient,
            "UpdatePatientProfile",
            1,
            json!({"fullName":"Synthetic updated patient"}),
        );
        cycle(&ea, &cred_a, &access_a.context).await;
        let conflict_id = record(
            &b,
            &access_b.context,
            &patient,
            "UpdatePatientProfile",
            1,
            json!({"fullName":"Synthetic conflicting intention"}),
        );
        cycle(&eb, &cred_b, &access_b.context).await;
        assert_eq!(
            SqliteOperationStore
                .operation_result(&b.data.lock().unwrap(), &access_b.context, &conflict_id)
                .unwrap()
                .unwrap()
                .status,
            OperationStatus::Conflict
        );

        // An invalid signature exercises the real CURSOR_EXPIRED recovery route.
        {
            let mut c = b.data.lock().unwrap();
            let mut cursor = SqliteSyncControlStore
                .cursor(&c, &access_b.context)
                .unwrap()
                .unwrap();
            cursor.cursor = "synthetic-invalid-cursor".into();
            let tx = c.transaction().unwrap();
            SqliteSyncControlStore
                .save_cursor(&tx, &access_b.context, &cursor)
                .unwrap();
            tx.commit().unwrap();
        }
        cycle(&eb, &cred_b, &access_b.context).await;
        assert_eq!(
            SqliteOperationStore
                .operation_result(&b.data.lock().unwrap(), &access_b.context, &conflict_id)
                .unwrap()
                .unwrap()
                .status,
            OperationStatus::Conflict
        );
        assert_eq!(
            SqliteSyncControlStore
                .item(
                    &b.data.lock().unwrap(),
                    &access_b.context,
                    "patient_profile",
                    &patient
                )
                .unwrap()
                .unwrap()
                .entity_version,
            2
        );

        let cursor = SqliteSyncControlStore
            .cursor(&b.data.lock().unwrap(), &access_b.context)
            .unwrap()
            .unwrap();
        let mut stream = http
            .events(&cred_b.token, &account, &cursor.cursor)
            .await
            .unwrap();
        let revoke = OperationEnvelope {
            operation_id: OperationId::try_from(uuid::Uuid::now_v7().to_string()).unwrap(),
            contract_version: 1,
            action: "RevokeAccountDevice".into(),
            entity_key: json!(cred_b.device_id),
            expected_version: 1,
            payload: json!({}),
            depends_on: vec![],
            occurred_at_local: None,
        };
        let revoked = SyncAPI(&http)
            .send_operations(&cred_a.token, &account, &[revoke])
            .await
            .unwrap();
        assert_eq!(revoked.value.results[0].status, ResultStatus::Accepted);
        let mut parser = Parser::default();
        let mut saw_revocation = false;
        let started = std::time::Instant::now();
        while started.elapsed() < std::time::Duration::from_secs(30) {
            let Some(chunk) = stream.chunk().await.unwrap() else {
                break;
            };
            if parser
                .feed(&chunk, &account)
                .unwrap()
                .contains(&Notice::Revoked)
            {
                saw_revocation = true;
                break;
            }
        }
        assert!(saw_revocation);
        drop(stream);
        assert_eq!(
            eb.listen_once(&cred_b, &access_b.context, &())
                .await
                .unwrap(),
            StreamOutcome::Revoked
        );
        assert_eq!(
            SqliteOperationStore
                .operation_result(&b.data.lock().unwrap(), &access_b.context, &conflict_id)
                .unwrap()
                .unwrap()
                .status,
            OperationStatus::Conflict
        );
        assert_eq!(
            IdentityAPI(&http)
                .access(&cred_b.token, &account)
                .await
                .unwrap()
                .value
                .status,
            crate::components::identity::domain::AccessStatus::Revoked
        );
        activation_a.logout(identity).unwrap();
        activation_b.logout(identity).unwrap();
        assert!(credentials_a.load().unwrap().is_none());
        assert!(credentials_b.load().unwrap().is_none());
        for db in [&a, &b] {
            let bytes = std::fs::read(&db.path).unwrap();
            for token in [&cred_a.token, &cred_b.token, session.access_token()] {
                assert!(
                    !bytes
                        .windows(token.expose().len())
                        .any(|part| part == token.expose().as_bytes())
                );
            }
        }
        println!(
            "C06: dos perfiles, respuesta descartada, recuperación GET, duplicado, conflicto, captura, SSE y revocación comprobados; sin tokens en SQLite."
        );
    });
}
