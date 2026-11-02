use super::{config::APIConfig, secrets::Secret};
use crate::shared_kernel::id::{OperationId, canonical_uuid_v7};
use reqwest::{Client, Method, header};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::time::Duration;

pub type APIResult<T> = std::result::Result<T, APIFailure>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum APICode {
    AuthenticationRequired,
    PermissionDenied,
    NotFound,
    VersionConflict,
    SlotUnavailable,
    IdempotencyConflict,
    CompletenessConflict,
    InvalidStateTransition,
    NoChanges,
    DependencyFailed,
    CursorExpired,
    SnapshotExpired,
    SnapshotScopeChanged,
    ResultExpired,
    ConnectionLimit,
    ValidationError,
    PayloadTooLarge,
    TemporaryFailure,
    ServiceUnavailable,
    InvalidResponse,
    ResponseTooLarge,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteError {
    pub code: APICode,
    pub correlation_id: OperationId,
    // The server's message and details never enter local diagnostics.
}

#[derive(Debug)]
pub struct APIFailure {
    pub status: Option<u16>,
    pub code: APICode,
    pub correlation_id: Option<OperationId>,
}

impl APIFailure {
    pub fn local(code: APICode) -> Self {
        Self {
            status: None,
            code,
            correlation_id: None,
        }
    }
}

pub struct APIResponse<T> {
    pub value: T,
    pub correlation_id: Option<OperationId>,
}

/// Shared HTTPS adapter. It owns no SQLite connection or transaction.
pub struct APITransport {
    client: Client,
    stream_client: Client,
    config: APIConfig,
    response_limit: usize,
}

// Keep discovery, JWKS and token requests under the same TLS/body limits.
// https://docs.rs/openidconnect/4.0.1/openidconnect/trait.AsyncHttpClient.html
impl<'c> openidconnect::AsyncHttpClient<'c> for APITransport {
    type Error = crate::shared_kernel::NightingaleError;
    type Future = std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = std::result::Result<openidconnect::HttpResponse, Self::Error>,
                > + Send
                + 'c,
        >,
    >;

    fn call(&'c self, request: openidconnect::HttpRequest) -> Self::Future {
        Box::pin(async move {
            let failure =
                || crate::shared_kernel::NightingaleError::temporary("fallo de transporte OIDC");

            APIConfig::parse(&request.uri().to_string()).map_err(|_| failure())?;

            if request.body().len() > 1024 * 1024 {
                return Err(failure());
            }

            let request = reqwest::Request::try_from(request).map_err(|_| failure())?;
            let mut response = self.client.execute(request).await.map_err(|_| failure())?;
            let mut output = openidconnect::http::Response::builder().status(response.status());

            for (name, value) in response.headers() {
                output = output.header(name, value);
            }

            let mut body = Vec::new();

            if response
                .content_length()
                .is_some_and(|length| length > self.response_limit as u64)
            {
                return Err(failure());
            }

            while let Some(chunk) = response.chunk().await.map_err(|_| failure())? {
                if chunk.len() > self.response_limit.saturating_sub(body.len()) {
                    return Err(failure());
                }

                body.extend_from_slice(&chunk);
            }

            output.body(body).map_err(|_| failure())
        })
    }
}

impl APITransport {
    pub fn new(config: APIConfig) -> APIResult<Self> {
        Self::with_timeout(config, Duration::from_secs(30))
    }

    pub(crate) fn with_timeout(config: APIConfig, timeout: Duration) -> APIResult<Self> {
        let certificate = development_certificate(
            std::env::var_os("NIGHTINGALE_DEV_CA_CERT").as_deref(),
            cfg!(debug_assertions),
        )?;
        // Do not forward credentials through redirects or retry token issuance.
        // https://docs.rs/reqwest/0.12.28/reqwest/struct.ClientBuilder.html
        let builder = || {
            let mut builder = Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .retry(reqwest::retry::never())
                .connect_timeout(Duration::from_secs(10))
                .user_agent(concat!("nightingale/", env!("CARGO_PKG_VERSION")));

            // Trust one explicit development PEM; keep TLS name and date checks.
            // https://docs.rs/reqwest/0.12.28/reqwest/struct.ClientBuilder.html#method.add_root_certificate
            if let Some(certificate) = &certificate {
                builder = builder.add_root_certificate(certificate.clone());
            }

            builder
        };
        let client = builder()
            .timeout(timeout)
            .build()
            .map_err(|_| APIFailure::local(APICode::TemporaryFailure))?;

        Ok(Self {
            client,
            stream_client: builder()
                .read_timeout(Duration::from_secs(45))
                .build()
                .map_err(|_| APIFailure::local(APICode::TemporaryFailure))?,
            config,
            response_limit: 8 * 1024 * 1024,
        })
    }

    pub(crate) async fn events(
        &self,
        token: &Secret,
        account: &crate::shared_kernel::id::AccountId,
        cursor: &str,
    ) -> APIResult<reqwest::Response> {
        let url = self
            .config
            .base_url()
            .join(&format!("v1/accounts/{}/sync/events", account.as_str()))
            .map_err(|_| APIFailure::local(APICode::ValidationError))?;
        let mut bearer = header::HeaderValue::from_str(&format!("Bearer {}", token.expose()))
            .map_err(|_| APIFailure::local(APICode::ValidationError))?;

        bearer.set_sensitive(true);

        // No total timeout: heartbeat bounds the idle read, not stream lifetime.
        // https://docs.rs/reqwest/0.12.28/reqwest/struct.ClientBuilder.html#method.read_timeout
        let response = self
            .stream_client
            .get(url)
            .query(&[("cursor", cursor)])
            .header(header::AUTHORIZATION, bearer)
            .header(header::ACCEPT, "text/event-stream")
            .send()
            .await
            .map_err(|_| APIFailure::local(APICode::TemporaryFailure))?;
        let status = response.status().as_u16();

        if status != 200 {
            return Err(APIFailure {
                status: Some(status),
                code: match status {
                    401 => APICode::AuthenticationRequired,
                    403 => APICode::PermissionDenied,
                    410 => APICode::CursorExpired,
                    429 => APICode::ConnectionLimit,
                    _ => APICode::TemporaryFailure,
                },
                correlation_id: None,
            });
        }

        if response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_none_or(|v| v.split(';').next().unwrap_or("").trim() != "text/event-stream")
        {
            return Err(APIFailure::local(APICode::InvalidResponse));
        }

        Ok(response)
    }

    pub(crate) async fn request<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        token: Option<&Secret>,
        body: Option<Vec<u8>>,
    ) -> APIResult<APIResponse<T>> {
        self.request_with_query(method, path, token, body, &[])
            .await
    }

    pub(crate) async fn request_with_query<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        token: Option<&Secret>,
        body: Option<Vec<u8>>,
        query: &[(&str, &str)],
    ) -> APIResult<APIResponse<T>> {
        let started = std::time::Instant::now();
        let result = self.request_inner(method, path, token, body, query).await;

        use super::observability::{Event, Kind, Outcome, emit};

        let mut event = Event::new(
            Kind::Http,
            if result.is_ok() {
                Outcome::Success
            } else {
                Outcome::Failed
            },
            started.elapsed(),
        );

        match &result {
            Ok(response) => event.correlation_id = response.correlation_id.clone(),
            Err(error) => {
                event.correlation_id = error.correlation_id.clone();
                event.code = Some(error.code);
            }
        }

        emit(&event);

        result
    }

    async fn request_inner<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        token: Option<&Secret>,
        body: Option<Vec<u8>>,
        query: &[(&str, &str)],
    ) -> APIResult<APIResponse<T>> {
        let invalid = || APIFailure::local(APICode::ValidationError);

        if !path.starts_with("v1/") {
            return Err(invalid());
        }

        let url = self.config.base_url().join(path).map_err(|_| invalid())?;

        if url.origin() != self.config.base_url().origin()
            || !url
                .path()
                .starts_with(&format!("{}v1/", self.config.base_url().path()))
        {
            return Err(invalid());
        }

        let mut request = self
            .client
            .request(method, url)
            // Encode opaque cursors as one value, including +, & and =.
            // https://docs.rs/reqwest/0.12.28/reqwest/struct.RequestBuilder.html#method.query
            .query(query)
            .header(header::ACCEPT, "application/json");

        if let Some(token) = token {
            let mut value = header::HeaderValue::from_str(&format!("Bearer {}", token.expose()))
                .map_err(|_| invalid())?;
            value.set_sensitive(true);
            request = request.header(header::AUTHORIZATION, value);
        }

        if let Some(body) = body {
            if body.len() > 1024 * 1024 {
                return Err(APIFailure::local(APICode::PayloadTooLarge));
            }

            request = request
                .header(header::CONTENT_TYPE, "application/json")
                .body(body);
        }

        let mut response = request
            .send()
            .await
            .map_err(|_| APIFailure::local(APICode::TemporaryFailure))?;
        let status = response.status();
        let correlation_id = response
            .headers()
            .get("X-Correlation-ID")
            .and_then(|value| value.to_str().ok())
            .filter(|value| canonical_uuid_v7(value).is_ok())
            .and_then(|value| OperationId::try_from(value.to_owned()).ok());
        let fail = |code| APIFailure {
            status: Some(status.as_u16()),
            code,
            correlation_id: correlation_id.clone(),
        };

        if response
            .content_length()
            .is_some_and(|length| length > self.response_limit as u64)
        {
            return Err(fail(APICode::ResponseTooLarge));
        }

        let mut bytes = Vec::new();

        // Bound the body even when Content-Length is absent or inaccurate.
        // https://docs.rs/reqwest/0.12.28/reqwest/struct.Response.html#method.chunk
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| fail(APICode::TemporaryFailure))?
        {
            if chunk.len() > self.response_limit.saturating_sub(bytes.len()) {
                return Err(fail(APICode::ResponseTooLarge));
            }

            bytes.extend_from_slice(&chunk);
        }

        if !status.is_success() {
            let error = serde_json::from_slice::<RemoteError>(&bytes).ok();
            let fallback = match status.as_u16() {
                401 => APICode::AuthenticationRequired,
                403 => APICode::PermissionDenied,
                404 => APICode::NotFound,
                413 => APICode::PayloadTooLarge,
                422 => APICode::ValidationError,
                429 => APICode::ConnectionLimit,
                503 => APICode::ServiceUnavailable,
                _ => APICode::Unknown,
            };

            return Err(match error {
                Some(error) => APIFailure {
                    status: Some(status.as_u16()),
                    code: error.code,
                    correlation_id: Some(error.correlation_id),
                },
                None => fail(fallback),
            });
        }

        // Snapshot release returns 204. Its empty body represents unit/null.
        let value = serde_json::from_slice(if status == reqwest::StatusCode::NO_CONTENT {
            b"null"
        } else {
            &bytes
        })
        .map_err(|_| fail(APICode::InvalidResponse))?;

        Ok(APIResponse {
            value,
            correlation_id,
        })
    }
}

fn development_certificate(
    path: Option<&std::ffi::OsStr>,
    development: bool,
) -> APIResult<Option<reqwest::Certificate>> {
    let Some(path) = path else { return Ok(None) };
    let invalid = || APIFailure::local(APICode::ValidationError);

    if !development {
        return Err(invalid());
    }

    let pem = std::fs::read(path).map_err(|_| invalid())?;
    let mut certificates = reqwest::Certificate::from_pem_bundle(&pem).map_err(|_| invalid())?;
    if certificates.len() != 1 {
        return Err(invalid());
    }
    Ok(certificates.pop())
}

#[cfg(test)]
pub(crate) mod testing {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
        time::Instant,
    };

    pub struct Server {
        pub config: APIConfig,
        handle: thread::JoinHandle<String>,
    }

    impl Server {
        pub fn new(status: u16, body: String, headers: &str) -> Self {
            Self::delayed(status, body, headers, Duration::ZERO)
        }

        pub fn delayed(status: u16, body: String, headers: &str, delay: Duration) -> Self {
            Self::with_handler(status, headers, delay, move |_| body)
        }

        pub fn respond(
            status: u16,
            headers: &str,
            handler: impl FnOnce(&str) -> String + Send + 'static,
        ) -> Self {
            Self::with_handler(status, headers, Duration::ZERO, handler)
        }

        fn with_handler(
            status: u16,
            headers: &str,
            delay: Duration,
            handler: impl FnOnce(&str) -> String + Send + 'static,
        ) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.set_nonblocking(true).unwrap();
            let config =
                APIConfig::parse(&format!("http://{}/", listener.local_addr().unwrap())).unwrap();
            let headers = headers.to_owned();
            let handle = thread::spawn(move || {
                let deadline = Instant::now() + Duration::from_secs(5);
                let mut stream = loop {
                    match listener.accept() {
                        Ok((stream, _)) => break stream,
                        Err(error)
                            if error.kind() == std::io::ErrorKind::WouldBlock
                                && Instant::now() < deadline =>
                        {
                            thread::sleep(Duration::from_millis(5))
                        }
                        Err(_) => panic!("el cliente no conectó al servidor de prueba"),
                    }
                };
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                stream
                    .set_write_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut bytes = Vec::new();

                loop {
                    let mut buffer = [0; 4096];
                    let length = stream.read(&mut buffer).unwrap();
                    assert!(length > 0, "petición incompleta");
                    bytes.extend_from_slice(&buffer[..length]);
                    assert!(bytes.len() <= 1024 * 1024 + 16384);

                    if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                        let head = String::from_utf8_lossy(&bytes[..index]).to_ascii_lowercase();
                        let content_length: usize = head
                            .lines()
                            .find_map(|line| {
                                line.strip_prefix("content-length:")
                                    .and_then(|value| value.trim().parse().ok())
                            })
                            .unwrap_or(0);

                        if bytes.len() >= index + 4 + content_length {
                            break;
                        }
                    }
                }
                thread::sleep(delay);
                let request = String::from_utf8(bytes).unwrap();
                let body = handler(&request);
                let response = if headers.contains("Transfer-Encoding")
                    || headers.contains("Content-Length")
                {
                    format!("HTTP/1.1 {status} Test\r\nConnection: close\r\n{headers}\r\n{body}")
                } else {
                    format!(
                        "HTTP/1.1 {status} Test\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{headers}\r\n{body}",
                        body.len()
                    )
                };

                // A timeout/size test can close the connection before the server writes.
                let _ = stream.write_all(response.as_bytes());

                request
            });

            Self { config, handle }
        }

        pub fn finish(self) -> String {
            self.handle.join().unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::identity::api::{DeviceRegistration, IdentityAPI};
    use crate::components::sync::adapters::api::{ResultStatus, SyncAPI};
    use crate::components::sync::domain::OperationEnvelope;
    use crate::shared_kernel::id::AccountId;
    use serde_json::{Value, json};
    use testing::Server;

    const CORRELATION: &str = "019a1234-0000-7000-8000-000000000009";

    #[test]
    fn development_certificate_is_explicit_rejects_release_and_hides_paths() {
        assert!(development_certificate(None, false).unwrap().is_none());
        assert!(development_certificate(None, true).unwrap().is_none());

        for development in [false, true] {
            for path in [
                "missing-sensitive-path.pem",
                concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"),
            ] {
                let error = development_certificate(Some(std::ffi::OsStr::new(path)), development)
                    .unwrap_err();
                assert_eq!(error.code, APICode::ValidationError);
                assert!(!format!("{error:?}").contains(path));
            }
        }
    }

    fn transport(server: &Server) -> APITransport {
        APITransport::new(APIConfig::parse(server.config.base_url().as_str()).unwrap()).unwrap()
    }

    #[test]
    fn auth_config_and_device_issuance_use_contract_and_bearer() {
        tauri::async_runtime::block_on(async {
            let server = Server::new(200,json!({"issuer":"https://issuer.example", "publicClientId":"desktop", "audience":"api"}).to_string(), &format!("X-Correlation-ID: {CORRELATION}\r\n"));
            let client = transport(&server);
            let response = IdentityAPI(&client).auth_config().await.unwrap();
            assert_eq!(response.value.public_client_id, "desktop");
            assert_eq!(response.correlation_id.unwrap().as_str(), CORRELATION);
            let request = server.finish();
            assert!(request.starts_with("GET /v1/auth/config "));
            assert!(!request.to_ascii_lowercase().contains("authorization:"));

            let server = Server::new(201,json!({"accessToken":"synthetic-device", "tokenId":CORRELATION, "deviceId":CORRELATION, "userId":CORRELATION,"issuedAt":"2026-10-29T12:00:00Z"}).to_string(),"");
            let client = transport(&server);
            let token = Secret::new("synthetic-oidc".into()).unwrap();
            let registration = DeviceRegistration {
                device_name: "Test device".into(),
                client_version: "0.1.0".into(),
                local_schema_version: 4,
            };
            let response = IdentityAPI(&client)
                .issue_device_token(&token, &registration)
                .await
                .unwrap();
            assert!(response.value.access_token.expose() == "synthetic-device");
            let request = server.finish();
            assert!(request.starts_with("POST /v1/auth/device-tokens "));
            assert!(
                request
                    .to_ascii_lowercase()
                    .contains("authorization: bearer synthetic-oidc")
            );
            let body: Value =
                serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
            assert_eq!(
                body,
                json!({"deviceName":"Test device","clientVersion":"0.1.0","localSchemaVersion":4})
            );
        });
    }

    #[test]
    fn http_errors_use_codes_not_message_and_never_copy_sensitive_data() {
        tauri::async_runtime::block_on(async {
            for (status, code, expected) in [
                (
                    401,
                    "AUTHENTICATION_REQUIRED",
                    APICode::AuthenticationRequired,
                ),
                (403, "PERMISSION_DENIED", APICode::PermissionDenied),
                (410, "CURSOR_EXPIRED", APICode::CursorExpired),
                (413, "PAYLOAD_TOO_LARGE", APICode::PayloadTooLarge),
                (422, "VALIDATION_ERROR", APICode::ValidationError),
                (429, "CONNECTION_LIMIT", APICode::ConnectionLimit),
                (503, "TEMPORARY_FAILURE", APICode::TemporaryFailure),
            ] {
                let server = Server::new(status,json!({"code":code,"message":"https://secret.example/token synthetic-secret", "correlationId":CORRELATION,"details":[{"loc":["synthetic-secret"],"code":"synthetic-secret"}]}).to_string(),"");
                let client = transport(&server);
                let error = IdentityAPI(&client).auth_config().await.err().unwrap();
                assert_eq!(error.status, Some(status));
                assert_eq!(error.code, expected);
                assert_eq!(error.correlation_id.as_ref().unwrap().as_str(), CORRELATION);
                assert!(!format!("{error:?}").contains("secret"));
                server.finish();
            }
        });
    }

    #[test]
    fn invalid_json_html_redirects_and_body_limits_return_safe_errors() {
        tauri::async_runtime::block_on(async {
            for (status, body, headers, expected) in [
                (200, "not json", "", APICode::InvalidResponse),
                (404, "<html>synthetic-secret</html>", "", APICode::NotFound),
                (
                    302,
                    "",
                    "Location: https://secret.example/\r\n",
                    APICode::Unknown,
                ),
                (
                    200,
                    "",
                    "Content-Length: 99999999\r\n",
                    APICode::ResponseTooLarge,
                ),
                (
                    200,
                    "20\r\nxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\r\n0\r\n\r\n",
                    "Transfer-Encoding: chunked\r\n",
                    APICode::ResponseTooLarge,
                ),
            ] {
                let server = Server::new(status, body.into(), headers);
                let mut client = transport(&server);
                client.response_limit = 16;

                // Keep the HTML body inside this case's configured bound.
                if status == 404 {
                    client.response_limit = 128;
                }

                let error = IdentityAPI(&client).auth_config().await.err().unwrap();
                assert_eq!(error.code, expected);
                assert!(!format!("{error:?}").contains("secret"));
                server.finish();
            }
        });
    }

    #[test]
    fn timeout_and_connection_failure_hide_urls_and_tokens() {
        tauri::async_runtime::block_on(async {
            let server = Server::delayed(200, "{}".into(), "", Duration::from_millis(250));
            let client = APITransport::with_timeout(
                APIConfig::parse(server.config.base_url().as_str()).unwrap(),
                Duration::from_millis(100),
            )
            .unwrap();
            let error = IdentityAPI(&client).auth_config().await.err().unwrap();
            assert_eq!(error.code, APICode::TemporaryFailure);
            assert!(!format!("{error:?}").contains("127.0.0.1"));
            server.finish();
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            drop(listener);
            let client =
                APITransport::new(APIConfig::parse(&format!("http://{address}/")).unwrap())
                    .unwrap();
            let error = IdentityAPI(&client)
                .me(&Secret::new("synthetic-secret".into()).unwrap())
                .await
                .err()
                .unwrap();
            assert_eq!(error.code, APICode::TemporaryFailure);
            assert!(!format!("{error:?}").contains("synthetic-secret"));
            assert!(!format!("{error:?}").contains(&address.to_string()));
        });
    }

    fn operation() -> OperationEnvelope {
        serde_json::from_value(json!({"operationId":"019a1234-0000-7000-8000-000000000001","contractVersion":1,"action":"RegisterAccountDevice","entityKey":CORRELATION,"expectedVersion":0,"payload":{},"dependsOn":[]})).unwrap()
    }

    #[test]
    fn batch_200_keeps_individual_results_and_duplicate_original() {
        tauri::async_runtime::block_on(async {
            for status in [
                "accepted",
                "conflict",
                "rejected",
                "retryable_failure",
                "duplicate",
            ] {
                let operation = operation();
                let original = json!({"operationId":operation.operation_id,"status":"rejected","entityVersion":null,"state":null,"error":{"code":"DEPENDENCY_FAILED","message":"changed text","correlationId":CORRELATION}});
                let server = Server::new(200,json!({"results":[{"operationId":operation.operation_id,"status":status,"entityVersion":null,"state":null,"error":null,"originalResult":if status == "duplicate" {original} else {Value::Null}}],"correlationId":CORRELATION}).to_string(),"");
                let client = transport(&server);
                let account = AccountId::try_from(CORRELATION.to_owned()).unwrap();
                let response = SyncAPI(&client)
                    .send_operations(
                        &Secret::new("synthetic-device".into()).unwrap(),
                        &account,
                        &[operation],
                    )
                    .await
                    .unwrap();
                if status == "duplicate" {
                    assert_eq!(response.value.results[0].status, ResultStatus::Duplicate);
                    assert_eq!(
                        response.value.results[0]
                            .original_result
                            .as_ref()
                            .unwrap()
                            .error
                            .as_ref()
                            .unwrap()
                            .code,
                        APICode::DependencyFailed
                    );
                } else if status == "conflict" {
                    assert_eq!(response.value.results[0].status, ResultStatus::Conflict);
                }
                let request = server.finish();
                assert!(
                    request
                        .starts_with(&format!("POST /v1/accounts/{CORRELATION}/sync/operations "))
                );
                assert!(!request.contains("accountId"));
                assert!(request.contains("dependsOn"));
            }
        });
    }

    #[test]
    fn malformed_batches_and_incomplete_results_are_rejected() {
        tauri::async_runtime::block_on(async {
            let client =
                APITransport::new(APIConfig::parse("https://api.example/").unwrap()).unwrap();
            let account = AccountId::try_from(CORRELATION.to_owned()).unwrap();
            let token = Secret::new("synthetic-device".into()).unwrap();
            for operations in [vec![], vec![operation(); 101], vec![operation(); 2]] {
                assert_eq!(
                    SyncAPI(&client)
                        .send_operations(&token, &account, &operations)
                        .await
                        .err()
                        .unwrap()
                        .code,
                    APICode::ValidationError
                );
            }
            let server = Server::new(
                200,
                json!({"results":[],"correlationId":CORRELATION}).to_string(),
                "",
            );
            let client = transport(&server);
            assert_eq!(
                SyncAPI(&client)
                    .send_operations(&token, &account, &[operation()])
                    .await
                    .err()
                    .unwrap()
                    .code,
                APICode::InvalidResponse
            );
            server.finish();
        });
    }

    #[test]
    fn identity_reads_and_operation_lookup_use_typed_routes_and_pagination() {
        tauri::async_runtime::block_on(async {
            let token = Secret::new("synthetic-device".into()).unwrap();
            let account = AccountId::try_from(CORRELATION.to_owned()).unwrap();
            let server = Server::new(200,json!({"userId":CORRELATION,"displayName":"Test user","status":"active","oidcIssuer":"https://issuer.example","oidcSubject":"test-user","accounts":{"items":[],"total":0}}).to_string(),"");
            let client = transport(&server);
            assert_eq!(
                IdentityAPI(&client)
                    .me(&token)
                    .await
                    .unwrap()
                    .value
                    .user_id
                    .as_str(),
                CORRELATION
            );
            assert!(server.finish().starts_with("GET /v1/me "));
            let server = Server::new(200,json!({"items":[{"id":CORRELATION,"displayName":"Test practice","status":"active","role":"owner"}],"total":101}).to_string(),"");
            let client = transport(&server);
            let page = IdentityAPI(&client)
                .accounts(&token, 2, 100)
                .await
                .unwrap()
                .value;
            assert_eq!(page.total, 101);
            assert_eq!(page.items.len(), 1);
            assert!(
                server
                    .finish()
                    .starts_with("GET /v1/accounts?page=2&shows=100 ")
            );
            let server = Server::new(200,json!({"accountId":CORRELATION,"userId":CORRELATION,"deviceId":CORRELATION,"role":"owner","status":"registration_required","permissionEpoch":3,"allowedActions":["RegisterAccountDevice"]}).to_string(),"");
            let client = transport(&server);
            let access = IdentityAPI(&client)
                .access(&token, &account)
                .await
                .unwrap()
                .value;
            assert_eq!(
                access.status,
                crate::components::identity::domain::AccessStatus::RegistrationRequired
            );
            assert_eq!(access.allowed_actions, vec!["RegisterAccountDevice"]);
            assert!(
                server
                    .finish()
                    .starts_with(&format!("GET /v1/accounts/{CORRELATION}/access "))
            );
            let server = Server::new(200,json!({"operationId":CORRELATION,"status":"accepted","entityVersion":1,"state":{},"error":null}).to_string(),"");
            let client = transport(&server);
            let operation = OperationId::try_from(CORRELATION.to_owned()).unwrap();
            let result = SyncAPI(&client)
                .operation_result(&token, &account, &operation)
                .await
                .unwrap()
                .value;
            assert_eq!(result.entity_version, Some(1));
            assert!(server.finish().starts_with(&format!(
                "GET /v1/accounts/{CORRELATION}/sync/operations/{CORRELATION} "
            )));
        });
    }
}
