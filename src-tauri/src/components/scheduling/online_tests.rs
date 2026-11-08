use super::{
    adapters::{desktop, online::OnlineScheduling},
    application::{self, AvailabilityRequest, RemoteAvailabilityPort},
    test_support::{PROFESSIONAL, access},
};
use crate::components::identity::domain::AccountAccess;
use crate::platform::{config::APIConfig, http::APITransport, secrets::Secret, test_api::TestApi};
use serde_json::{Value, json};
use std::{
    str::FromStr,
    sync::{Arc, Mutex},
};

const SERVICE: &str = "019a1234-0000-7000-8000-0000000000b1";

fn request(duration: Option<i64>) -> AvailabilityRequest {
    AvailabilityRequest {
        professional_id: PROFESSIONAL.into(),
        service_id: SERVICE.into(),
        from: jiff::Timestamp::from_str("2026-05-06T04:00:00Z").unwrap(),
        until: jiff::Timestamp::from_str("2026-05-07T04:00:00Z").unwrap(),
        duration_minutes: duration,
        exclude_appointment_id: None,
    }
}

fn access_response() -> Value {
    json!({
        "accountId":"019a1234-0000-7000-8000-000000000001",
        "userId":"019a1234-0000-7000-8000-000000000002",
        "deviceId":"019a1234-0000-7000-8000-000000000003",
        "permissionEpoch":1,
        "role":"assistant",
        "status":"active",
        "allowedActions":["CreateScheduleRule"]
    })
}

fn availability_response() -> Value {
    json!({
        "accountId":"019a1234-0000-7000-8000-000000000001",
        "professionalId":PROFESSIONAL,
        "serviceId":SERVICE,
        "referenceDurationMinutes":30,
        "requestedDurationMinutes":45,
        "availableIntervals":[
            {"startsAt":"2026-05-06T12:00:00Z","endsAt":"2026-05-06T13:00:00Z"}
        ]
    })
}

#[test]
fn loopback_success_returns_remote_view_without_local_writes() {
    tauri::async_runtime::block_on(async {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let seen = requests.clone();
        let server = TestApi::new(2, move |request| {
            seen.lock().unwrap().push(request.to_owned());
            if request.starts_with("GET ") && request.contains("/availability") {
                (200, availability_response())
            } else {
                (200, access_response())
            }
        });
        let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
        let token = Secret::new("synthetic-secret".into()).unwrap();
        let access = access();
        let remote = OnlineScheduling {
            http: &http,
            token: &token,
            scope: &access.context,
        };

        let result = application::check_remote_availability(&remote, &access, &request(Some(45)))
            .await
            .unwrap();

        assert_eq!(result.origin, "remote");
        assert_eq!(result.account_id, "019a1234-0000-7000-8000-000000000001");
        assert_eq!(result.reference_duration_minutes, 30);
        assert_eq!(result.requested_duration_minutes, 45);
        assert_eq!(result.intervals.len(), 1);
        assert_eq!(result.intervals[0].starts_at, "2026-05-06T12:00:00Z");

        let requests = requests.lock().unwrap();
        let query = requests
            .iter()
            .find(|request| request.contains("/availability"))
            .unwrap();
        assert!(query.contains("professionalId="));
        assert!(query.contains("durationMinutes=45"));
    });
}

#[test]
fn invalid_response_and_server_error_map_to_safe_codes() {
    tauri::async_runtime::block_on(async {
        // Respuesta con serviceId distinto: rechazo de validación.
        let server = TestApi::new(2, |request| {
            if request.contains("/availability") {
                let mut response = availability_response();
                response["serviceId"] = json!("019a1234-0000-7000-8000-0000000000ff");
                (200, response)
            } else {
                (200, access_response())
            }
        });
        let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
        let token = Secret::new("synthetic-secret".into()).unwrap();
        let access = access();
        let remote = OnlineScheduling {
            http: &http,
            token: &token,
            scope: &access.context,
        };
        let err = application::check_remote_availability(&remote, &access, &request(None))
            .await
            .unwrap_err();
        assert_eq!(err.code, "INVALID_RESPONSE");

        // 503: fallo temporal seguro.
        let server = TestApi::new(2, |request| {
            if request.contains("/availability") {
                (
                    503,
                    json!({"code":"retryable_failure","correlationId":null}),
                )
            } else {
                (200, access_response())
            }
        });
        let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
        let remote = OnlineScheduling {
            http: &http,
            token: &token,
            scope: &access.context,
        };
        let err = application::check_remote_availability(&remote, &access, &request(None))
            .await
            .unwrap_err();
        assert_eq!(err.code, "SERVICE_UNAVAILABLE");
    });
}

#[test]
fn oversized_range_is_rejected_before_any_http() {
    tauri::async_runtime::block_on(async {
        let calls = Arc::new(Mutex::new(0));
        let counted = calls.clone();
        let server = TestApi::new(0, move |_request| {
            *counted.lock().unwrap() += 1;
            (200, json!({}))
        });
        let http = APITransport::new(APIConfig::parse(&server.url).unwrap()).unwrap();
        let token = Secret::new("synthetic-secret".into()).unwrap();
        let access = access();
        let remote = OnlineScheduling {
            http: &http,
            token: &token,
            scope: &access.context,
        };

        let mut oversized = request(None);
        oversized.until = jiff::Timestamp::from_str("2026-06-20T04:00:00Z").unwrap();
        let err = application::check_remote_availability(&remote, &access, &oversized)
            .await
            .unwrap_err();
        assert_eq!(err.code, "VALIDATION_ERROR");
        assert_eq!(*calls.lock().unwrap(), 0, "sin peticiones HTTP");
    });
}

struct Stable;

impl RemoteAvailabilityPort for Stable {
    async fn refresh_access(&self) -> application::Result<AccountAccess> {
        Ok(access())
    }

    async fn availability(
        &self,
        _access: &AccountAccess,
        _request: &AvailabilityRequest,
    ) -> application::Result<application::RemoteAvailability> {
        Ok(application::RemoteAvailability {
            account_id: "019a1234-0000-7000-8000-000000000001".into(),
            professional_id: PROFESSIONAL.into(),
            service_id: SERVICE.into(),
            reference_duration_minutes: 30,
            requested_duration_minutes: 30,
            intervals: vec![],
            origin: "remote".into(),
        })
    }
}

/// Puerto falso que cambia la sesión local durante la petición.
struct SessionSwappingRemote {
    path: std::path::PathBuf,
}

impl RemoteAvailabilityPort for SessionSwappingRemote {
    async fn refresh_access(&self) -> application::Result<AccountAccess> {
        Ok(access())
    }

    async fn availability(
        &self,
        _access: &AccountAccess,
        _request: &AvailabilityRequest,
    ) -> application::Result<application::RemoteAvailability> {
        // Cambio de cuenta mientras "viaja" la petición (segunda conexión al
        // mismo archivo: la puerta no retiene ningún candado durante el HTTP).
        rusqlite::Connection::open(&self.path)
            .unwrap()
            .execute(
                "UPDATE identity_context SET selected_account_id = '019a1234-0000-7000-8000-0000000000ee'",
                [],
            )
            .unwrap();
        Ok(application::RemoteAvailability {
            account_id: "019a1234-0000-7000-8000-000000000001".into(),
            professional_id: PROFESSIONAL.into(),
            service_id: SERVICE.into(),
            reference_duration_minutes: 30,
            requested_duration_minutes: 30,
            intervals: vec![],
            origin: "remote".into(),
        })
    }
}

struct SessionDatabase {
    path: std::path::PathBuf,
    connection: Option<rusqlite::Connection>,
}

impl Drop for SessionDatabase {
    fn drop(&mut self) {
        // Windows no deja borrar el archivo mientras haya una conexión abierta.
        if let Some(connection) = self.connection.take() {
            let _ = connection.close();
        }
        if self.path.exists() {
            std::fs::remove_file(&self.path).unwrap();
        }
    }
}

/// Base temporal en archivo: la puerta lee por una conexión y el remoto falso
/// muta la sesión por otra, sin compartir candados.
fn session_database() -> SessionDatabase {
    let path = std::env::temp_dir().join(format!(
        "nightingale-scheduling-t11-{}.sqlite",
        uuid::Uuid::now_v7()
    ));
    let connection = crate::platform::sqlite::open(&path).unwrap();
    connection
        .execute(
            "INSERT INTO identity_context (singleton,issuer,subject,user_id,selected_account_id) VALUES (1,'issuer','subject','019a1234-0000-7000-8000-000000000002','019a1234-0000-7000-8000-000000000001')",
            [],
        )
        .unwrap();
    SessionDatabase {
        path,
        connection: Some(connection),
    }
}

#[test]
fn session_gate_rejects_account_switch_mid_request() {
    tauri::async_runtime::block_on(async {
        let database = session_database();
        let remote = SessionSwappingRemote {
            path: database.path.clone(),
        };
        let access = access();

        let err = desktop::check_remote_availability(
            database.connection.as_ref().unwrap(),
            &remote,
            &access,
            &request(None),
        )
        .await
        .unwrap_err();
        assert_eq!(err.code, "PERMISSION_DENIED");
        assert!(err.message.contains("sesión cambió"));
    });
}

#[test]
fn session_gate_allows_unchanged_session() {
    tauri::async_runtime::block_on(async {
        let database = session_database();
        let access = access();

        let result = desktop::check_remote_availability(
            database.connection.as_ref().unwrap(),
            &Stable,
            &access,
            &request(None),
        )
        .await
        .unwrap();
        assert_eq!(result.origin, "remote");
    });
}
