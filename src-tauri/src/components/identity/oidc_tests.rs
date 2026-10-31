use super::*;
use crate::components::identity::api::AuthConfig;
use crate::platform::{
    config::APIConfig,
    http::{APITransport, testing::Server},
    secrets::{CredentialKey, CredentialKind, Secret, SecretStore, testing::MemorySecretStore},
};
use openidconnect::core::{
    CoreIdToken, CoreIdTokenClaims, CoreJwsSigningAlgorithm, CoreProviderMetadata,
    CoreRsaPrivateSigningKey,
};
use openidconnect::{AccessToken, JsonWebKeyId, PrivateSigningKey};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

// Generated for these tests. This key has no use outside the local fake provider.
const TEST_KEY: &str = include_str!("../../../tests/fixtures/oidc-test-key.pem");

fn signing_key() -> CoreRsaPrivateSigningKey {
    CoreRsaPrivateSigningKey::from_pem(TEST_KEY, Some(JsonWebKeyId::new("test-key".into())))
        .unwrap()
}

type BrowserParameters = Arc<Mutex<HashMap<String, String>>>;

struct FakeBrowser {
    parameters: BrowserParameters,
    bad_state: bool,
    cancelled: bool,
}

impl Browser for FakeBrowser {
    fn open(&self, url: &str) -> OidcResult<()> {
        let url = reqwest::Url::parse(url).unwrap();
        let parameters: HashMap<_, _> = url
            .query_pairs()
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect();
        assert_eq!(parameters["response_type"], "code");
        assert_eq!(parameters["code_challenge_method"], "S256");
        assert!(
            parameters["scope"]
                .split_whitespace()
                .any(|scope| scope == "openid")
        );
        assert!(!parameters.contains_key("client_secret"));
        *self.parameters.lock().unwrap() = parameters.clone();
        let mut redirect = reqwest::Url::parse(&parameters["redirect_uri"]).unwrap();
        let host = format!("127.0.0.1:{}", redirect.port().unwrap());
        let state = if self.bad_state {
            "wrong-state"
        } else {
            &parameters["state"]
        };
        redirect.query_pairs_mut().append_pair("state", state);

        if self.cancelled {
            redirect
                .query_pairs_mut()
                .append_pair("error", "access_denied");
        } else {
            redirect
                .query_pairs_mut()
                .append_pair("code", "synthetic-code");
        }

        let target = format!("{}?{}", redirect.path(), redirect.query().unwrap());
        std::thread::spawn(move || {
            if let Ok(mut stream) = TcpStream::connect(&host) {
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                stream
                    .write_all(format!("GET {target} HTTP/1.1\r\nHost: {host}\r\n\r\n").as_bytes())
                    .unwrap();
                let mut response = Vec::new();
                let _ = stream.read_to_end(&mut response);
            }
        });

        Ok(())
    }
}

fn browser() -> FakeBrowser {
    FakeBrowser {
        parameters: Arc::new(Mutex::new(HashMap::new())),
        bad_state: false,
        cancelled: false,
    }
}

#[derive(Clone, Copy)]
enum TokenMode {
    Valid,
    BadNonce,
    BadAudience,
    BadIssuer,
    Expired,
    BadSignature,
    WrongAccess,
    OmitNonce,
    OtherSubject,
}

fn token_response(issuer: &str, nonce: &str, mode: TokenMode) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut claims = json!({
        "iss":if matches!(mode,TokenMode::BadIssuer) {"https://wrong.example/"} else {issuer},
        "sub":if matches!(mode,TokenMode::OtherSubject) {"other-user"} else {"test-user"},"aud":if matches!(mode,TokenMode::BadAudience) {"wrong-client"} else {"desktop"},
        "iat":now,"exp":if matches!(mode,TokenMode::Expired) {now-60} else {now+300},
        "nonce":if matches!(mode,TokenMode::BadNonce) {"wrong-nonce"} else {nonce}
    });

    if matches!(mode, TokenMode::OmitNonce) {
        claims.as_object_mut().unwrap().remove("nonce");
    }

    let claims: CoreIdTokenClaims = serde_json::from_value(claims).unwrap();
    let access = AccessToken::new("synthetic-access".into());
    let mut jwt = CoreIdToken::new(
        claims,
        &signing_key(),
        CoreJwsSigningAlgorithm::RsaSsaPkcs1V15Sha256,
        Some(&access),
        None,
    )
    .unwrap()
    .to_string();

    if matches!(mode, TokenMode::BadSignature) {
        let index = jwt.rfind('.').unwrap() + 1;
        let replacement = if &jwt[index..index + 1] == "A" {
            "B"
        } else {
            "A"
        };
        jwt.replace_range(index..index + 1, replacement);
    }

    json!({"access_token":if matches!(mode,TokenMode::WrongAccess) {"substituted-access"} else {"synthetic-access"},
        "token_type":"Bearer","refresh_token":"synthetic-refresh","expires_in":300,"id_token":jwt}).to_string()
}

fn metadata(issuer: &str, token_url: &str, jwks_url: &str) -> Value {
    json!({"issuer":issuer,"authorization_endpoint":format!("{issuer}authorize"),"token_endpoint":token_url,"jwks_uri":jwks_url,
        "response_types_supported":["code"],"subject_types_supported":["public"],"id_token_signing_alg_values_supported":["RS256"]})
}

async fn fake_provider(mode: TokenMode) -> (OidcProvider, APITransport, FakeBrowser, Server) {
    let browser = browser();
    let parameters = browser.parameters.clone();
    let issuer_value = Arc::new(Mutex::new(String::new()));
    let issuer_for_token = issuer_value.clone();
    let token = Server::respond(200, "", move |request| {
        assert!(request.starts_with("POST /token "));
        let form = request.split("\r\n\r\n").nth(1).unwrap();
        let form_url = reqwest::Url::parse(&format!("http://localhost/?{form}")).unwrap();
        let form: HashMap<_, _> = form_url
            .query_pairs()
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect();
        assert_eq!(form["grant_type"], "authorization_code");
        assert_eq!(form["client_id"], "desktop");
        assert!(!form.contains_key("client_secret"));
        let parameters = parameters.lock().unwrap();
        let verifier = openidconnect::PkceCodeVerifier::new(form["code_verifier"].clone());
        assert_eq!(
            openidconnect::PkceCodeChallenge::from_code_verifier_sha256(&verifier).as_str(),
            parameters["code_challenge"]
        );
        assert_eq!(form["redirect_uri"], parameters["redirect_uri"]);

        token_response(
            &issuer_for_token.lock().unwrap(),
            &parameters["nonce"],
            mode,
        )
    });
    let jwks = Server::new(
        200,
        json!({"keys":[signing_key().as_verification_key()]}).to_string(),
        "",
    );
    let token_url = format!("{}token", token.config.base_url());
    let jwks_url = format!("{}jwks", jwks.config.base_url());
    let discovery = Server::respond(200, "", move |request| {
        assert!(request.starts_with("GET /.well-known/openid-configuration "));
        let host = request
            .lines()
            .filter_map(|line| line.split_once(':'))
            .find(|(name, _)| name.eq_ignore_ascii_case("host"))
            .unwrap()
            .1
            .trim();

        metadata(&format!("http://{host}/"), &token_url, &jwks_url).to_string()
    });
    let issuer = discovery.config.base_url().as_str().to_owned();
    *issuer_value.lock().unwrap() = issuer.clone();
    let config: AuthConfig = serde_json::from_value(
        json!({"issuer":issuer,"publicClientId":"desktop","audience":"api"}),
    )
    .unwrap();
    let http = APITransport::new(APIConfig::parse("https://api.example/").unwrap()).unwrap();
    let provider = OidcProvider::discover(&config, &http).await.unwrap();
    discovery.finish();
    jwks.finish();

    (provider, http, browser, token)
}

#[test]
fn pkce_signed_login_storage_refresh_and_local_logout() {
    tauri::async_runtime::block_on(async {
        let (mut provider, http, browser, token) = fake_provider(TokenMode::Valid).await;
        let attempt = provider.begin(&browser).unwrap();
        let session = provider
            .complete(attempt, LoginCancellation::default(), &http)
            .await
            .unwrap();
        assert_eq!(session.subject(), "test-user");
        assert!(session.access_token().expose() == "synthetic-access");
        assert!(session.expires_at().unwrap() > SystemTime::now());
        token.finish();
        let response = serde_json::from_str(&token_response(
            session.issuer(),
            session.nonce.secret(),
            TokenMode::OmitNonce,
        ))
        .unwrap();
        assert!(
            provider
                .verify(response, session.nonce.clone(), Some(&session))
                .is_ok()
        );
        let response = serde_json::from_str(&token_response(
            session.issuer(),
            session.nonce.secret(),
            TokenMode::OtherSubject,
        ))
        .unwrap();
        assert_eq!(
            provider
                .verify(response, session.nonce.clone(), Some(&session))
                .err()
                .unwrap(),
            OidcFailure::InvalidToken
        );
        let mut response: Value = serde_json::from_str(&token_response(
            session.issuer(),
            session.nonce.secret(),
            TokenMode::Valid,
        ))
        .unwrap();
        response["token_type"] = json!("mac");
        assert_eq!(
            provider
                .verify(
                    serde_json::from_value(response).unwrap(),
                    session.nonce.clone(),
                    Some(&session)
                )
                .err()
                .unwrap(),
            OidcFailure::InvalidToken
        );
        let config = APIConfig::parse("https://api.example/").unwrap();
        let store = MemorySecretStore::default();
        let identity = (session.issuer(), session.subject());
        let api_key = CredentialKey::new(
            &config,
            "profile",
            identity,
            None,
            None,
            CredentialKind::APIDevice,
        )
        .unwrap();
        store
            .write(
                &api_key,
                &Secret::new("synthetic-api-device".into()).unwrap(),
            )
            .unwrap();
        session.persist(&config, "profile", &store).unwrap();
        let loaded = OidcSession::load(&config, "profile", identity, &store)
            .unwrap()
            .unwrap();
        assert!(loaded.access_token().expose() == "synthetic-access");
        let refresh = Server::new(200,json!({"access_token":"synthetic-refreshed","token_type":"Bearer","refresh_token":"synthetic-refresh-rotated","expires_in":600}).to_string(),"");
        provider.metadata = provider.metadata.clone().set_token_endpoint(Some(
            openidconnect::TokenUrl::new(format!("{}refresh", refresh.config.base_url())).unwrap(),
        ));
        let refreshed = provider.refresh(&loaded, &http).await.unwrap();
        assert!(refreshed.access_token().expose() == "synthetic-refreshed");
        refreshed.persist(&config, "profile", &store).unwrap();
        let request = refresh.finish();
        assert!(request.starts_with("POST /refresh "));
        assert!(request.contains("grant_type=refresh_token"));
        assert!(store.read(&api_key).unwrap().unwrap().expose() == "synthetic-api-device");
        logout(&config, "profile", identity, None, &store).unwrap();
        assert!(
            OidcSession::load(&config, "profile", identity, &store)
                .unwrap()
                .is_none()
        );
        assert!(store.read(&api_key).unwrap().is_some());
        logout(&config, "profile", identity, Some(&api_key), &store).unwrap();
        assert!(store.read(&api_key).unwrap().is_none());
    });
}

#[test]
fn rejects_bad_nonce_audience_issuer_expiry_signature_and_access_hash() {
    tauri::async_runtime::block_on(async {
        for mode in [
            TokenMode::BadNonce,
            TokenMode::BadAudience,
            TokenMode::BadIssuer,
            TokenMode::Expired,
            TokenMode::BadSignature,
            TokenMode::WrongAccess,
            TokenMode::OmitNonce,
        ] {
            let (provider, http, browser, token) = fake_provider(mode).await;
            let attempt = provider.begin(&browser).unwrap();
            assert_eq!(
                provider
                    .complete(attempt, LoginCancellation::default(), &http)
                    .await
                    .err()
                    .unwrap(),
                OidcFailure::InvalidToken
            );
            token.finish();
        }
    });
}

#[test]
fn secure_store_failure_reports_error_and_restores_prior_session() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct FailOnce {
        inner: MemorySecretStore,
        writes: AtomicUsize,
    }
    impl SecretStore for FailOnce {
        fn read(&self, key: &CredentialKey) -> crate::shared_kernel::Result<Option<Secret>> {
            self.inner.read(key)
        }
        fn write(&self, key: &CredentialKey, secret: &Secret) -> crate::shared_kernel::Result<()> {
            if self.writes.fetch_add(1, Ordering::Relaxed) == 1 {
                return Err(crate::shared_kernel::NightingaleError::temporary(
                    "almacén no disponible",
                ));
            }

            self.inner.write(key, secret)
        }
        fn delete(&self, key: &CredentialKey) -> crate::shared_kernel::Result<()> {
            self.inner.delete(key)
        }
    }
    let store = FailOnce {
        inner: MemorySecretStore::default(),
        writes: AtomicUsize::new(0),
    };
    let config = APIConfig::parse("https://api.example/").unwrap();
    let session = OidcSession {
        issuer: "https://issuer.example/".into(),
        subject: "test-user".into(),
        access: Secret::new("prior-access".into()).unwrap(),
        refresh: Some(Secret::new("prior-refresh".into()).unwrap()),
        nonce: Nonce::new("prior-nonce".into()),
        expires_at: None,
    };
    session.persist(&config, "profile", &store.inner).unwrap();
    let changed = OidcSession {
        access: Secret::new("changed-access".into()).unwrap(),
        refresh: Some(Secret::new("changed-refresh".into()).unwrap()),
        ..session
    };
    assert_eq!(
        changed.persist(&config, "profile", &store).unwrap_err(),
        OidcFailure::StorageUnavailable
    );
    let loaded = OidcSession::load(
        &config,
        "profile",
        ("https://issuer.example/", "test-user"),
        &store,
    )
    .unwrap()
    .unwrap();
    assert!(loaded.access_token().expose() == "prior-access");
    assert!(loaded.refresh.as_ref().unwrap().expose() == "prior-refresh");
}

#[test]
fn invalid_state_and_cancel_keep_previous_credentials() {
    tauri::async_runtime::block_on(async {
        let issuer = "https://issuer.example/";
        let provider = OidcProvider {
            metadata: serde_json::from_value::<CoreProviderMetadata>(metadata(
                issuer,
                "https://must-not-call.invalid/token",
                "https://issuer.example/jwks",
            ))
            .unwrap(),
            client_id: "desktop".into(),
        };
        let http = APITransport::new(APIConfig::parse("https://api.example/").unwrap()).unwrap();
        let config = APIConfig::parse("https://api.example/").unwrap();
        let store = MemorySecretStore::default();
        let session = OidcSession {
            issuer: issuer.into(),
            subject: "test-user".into(),
            access: Secret::new("previous-access".into()).unwrap(),
            refresh: Some(Secret::new("previous-refresh".into()).unwrap()),
            nonce: openidconnect::Nonce::new("previous-nonce".into()),
            expires_at: None,
        };
        session.persist(&config, "profile", &store).unwrap();
        for (bad_state, cancelled, expected) in [
            (true, false, OidcFailure::InvalidCallback),
            (false, true, OidcFailure::Cancelled),
        ] {
            let mut browser = browser();
            browser.bad_state = bad_state;
            browser.cancelled = cancelled;
            let attempt = provider.begin(&browser).unwrap();
            assert_eq!(
                provider
                    .complete(attempt, LoginCancellation::default(), &http)
                    .await
                    .err()
                    .unwrap(),
                expected
            );
            assert!(
                OidcSession::load(&config, "profile", (issuer, "test-user"), &store)
                    .unwrap()
                    .unwrap()
                    .access_token()
                    .expose()
                    == "previous-access"
            );
        }
        struct QuietBrowser;
        impl Browser for QuietBrowser {
            fn open(&self, _: &str) -> OidcResult<()> {
                Ok(())
            }
        }
        let attempt = provider.begin(&QuietBrowser).unwrap();
        let cancellation = LoginCancellation::default();
        cancellation.cancel();
        assert_eq!(
            provider
                .complete(attempt, cancellation, &http)
                .await
                .err()
                .unwrap(),
            OidcFailure::Cancelled
        );
    });
}

#[test]
#[ignore = "requires a configured development API/OIDC, browser and test account"]
fn manual_oidc_login_in_external_browser() {
    tauri::async_runtime::block_on(async {
        let config = APIConfig::from_environment().unwrap();
        let http = APITransport::new(config).unwrap();
        let config = crate::components::identity::api::IdentityAPI(&http)
            .auth_config()
            .await
            .unwrap()
            .value;
        let provider = OidcProvider::discover(&config, &http).await.unwrap();
        let attempt = provider.begin(&SystemBrowser).unwrap();
        let session = provider
            .complete(attempt, LoginCancellation::default(), &http)
            .await
            .unwrap();
        assert!(!session.subject().is_empty());
        // Do not print or persist this development account's tokens.
    });
}
