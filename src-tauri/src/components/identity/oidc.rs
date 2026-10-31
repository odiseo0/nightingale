use super::api::AuthConfig;
use crate::platform::{
    config::APIConfig,
    http::APITransport,
    secrets::{CredentialKey, CredentialKind, Secret, SecretStore},
};
use openidconnect::core::{
    CoreAuthenticationFlow, CoreClient, CoreProviderMetadata, CoreTokenResponse,
};
use openidconnect::{
    AccessTokenHash, AuthType, AuthorizationCode, ClientId, CsrfToken, IssuerUrl, Nonce,
    OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RefreshToken,
    TokenResponse,
};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant, SystemTime},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OidcFailure {
    InvalidConfiguration,
    ProviderUnavailable,
    BrowserUnavailable,
    InvalidCallback,
    Cancelled,
    InvalidToken,
    StorageUnavailable,
}

pub type OidcResult<T> = std::result::Result<T, OidcFailure>;

pub trait Browser {
    fn open(&self, url: &str) -> OidcResult<()>;
}

pub struct SystemBrowser;

impl Browser for SystemBrowser {
    fn open(&self, url: &str) -> OidcResult<()> {
        // The URL comes from the OIDC client, never from a React command argument.
        // https://docs.rs/tauri-plugin-opener/2.7.2/tauri_plugin_opener/fn.open_url.html
        tauri_plugin_opener::open_url(url, None::<&str>)
            .map_err(|_| OidcFailure::BrowserUnavailable)
    }
}

#[derive(Clone, Default)]
pub struct LoginCancellation(Arc<AtomicBool>);

impl LoginCancellation {
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

pub struct OidcProvider {
    metadata: CoreProviderMetadata,
    client_id: String,
}

pub struct LoginAttempt {
    listener: TcpListener,
    redirect: RedirectUrl,
    state: CsrfToken,
    nonce: Nonce,
    verifier: PkceCodeVerifier,
}

pub struct OidcSession {
    issuer: String,
    subject: String,
    access: Secret,
    refresh: Option<Secret>,
    nonce: Nonce,
    expires_at: Option<SystemTime>,
}

impl OidcSession {
    pub fn issuer(&self) -> &str {
        &self.issuer
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn access_token(&self) -> &Secret {
        &self.access
    }

    pub fn expires_at(&self) -> Option<SystemTime> {
        self.expires_at
    }

    pub fn persist(
        &self,
        config: &APIConfig,
        profile: &str,
        store: &impl SecretStore,
    ) -> OidcResult<()> {
        let (access_key, refresh_key) =
            session_keys(config, profile, (&self.issuer, &self.subject))?;
        let prior_access = store
            .read(&access_key)
            .map_err(|_| OidcFailure::StorageUnavailable)?;
        let prior_refresh = store
            .read(&refresh_key)
            .map_err(|_| OidcFailure::StorageUnavailable)?;
        let result = (|| {
            store.write(&access_key, &self.access)?;

            match &self.refresh {
                Some(refresh) => {
                    // Only the secure-store boundary may encode a secret bundle.
                    let bundle = Secret::new(serde_json::json!({"refreshToken":refresh.expose(),"nonce":self.nonce.secret()}).to_string())?;
                    store.write(&refresh_key, &bundle)
                }
                None => store.delete(&refresh_key),
            }
        })();

        if result.is_err() {
            for (key, prior) in [(&access_key, prior_access), (&refresh_key, prior_refresh)] {
                let _ = match prior {
                    Some(secret) => store.write(key, &secret),
                    None => store.delete(key),
                };
            }

            return Err(OidcFailure::StorageUnavailable);
        }

        Ok(())
    }

    pub fn load(
        config: &APIConfig,
        profile: &str,
        identity: (&str, &str),
        store: &impl SecretStore,
    ) -> OidcResult<Option<Self>> {
        let (access_key, refresh_key) = session_keys(config, profile, identity)?;
        let Some(access) = store
            .read(&access_key)
            .map_err(|_| OidcFailure::StorageUnavailable)?
        else {
            return Ok(None);
        };

        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Bundle {
            refresh_token: Secret,
            nonce: Secret,
        }

        let bundle = store
            .read(&refresh_key)
            .map_err(|_| OidcFailure::StorageUnavailable)?
            .map(|secret| {
                serde_json::from_str::<Bundle>(secret.expose())
                    .map_err(|_| OidcFailure::StorageUnavailable)
            })
            .transpose()?;
        let (refresh, nonce) = match bundle {
            Some(bundle) => (
                Some(bundle.refresh_token),
                Nonce::new(bundle.nonce.expose().to_owned()),
            ),
            None => (None, Nonce::new(String::new())),
        };

        Ok(Some(Self {
            issuer: identity.0.into(),
            subject: identity.1.into(),
            access,
            refresh,
            nonce,
            expires_at: None,
        }))
    }
}

fn session_keys(
    config: &APIConfig,
    profile: &str,
    identity: (&str, &str),
) -> OidcResult<(CredentialKey, CredentialKey)> {
    let key = |kind| {
        CredentialKey::new(config, profile, identity, None, None, kind)
            .map_err(|_| OidcFailure::InvalidConfiguration)
    };

    Ok((
        key(CredentialKind::OidcAccess)?,
        key(CredentialKind::OidcRefresh)?,
    ))
}

/// Local logout never calls an OIDC or API revocation endpoint.
pub fn logout(
    config: &APIConfig,
    profile: &str,
    identity: (&str, &str),
    api_key: Option<&CredentialKey>,
    store: &impl SecretStore,
) -> OidcResult<()> {
    let (access, refresh) = session_keys(config, profile, identity)?;
    let mut failed = false;

    for key in [Some(&access), Some(&refresh), api_key]
        .into_iter()
        .flatten()
    {
        failed |= store.delete(key).is_err();
    }

    if failed {
        return Err(OidcFailure::StorageUnavailable);
    }

    Ok(())
}

impl OidcProvider {
    pub async fn discover(config: &AuthConfig, http: &APITransport) -> OidcResult<Self> {
        APIConfig::parse(&config.issuer).map_err(|_| OidcFailure::InvalidConfiguration)?;

        if config.public_client_id.is_empty() || config.audience.is_empty() {
            return Err(OidcFailure::InvalidConfiguration);
        }

        let issuer =
            IssuerUrl::new(config.issuer.clone()).map_err(|_| OidcFailure::InvalidConfiguration)?;
        let metadata = CoreProviderMetadata::discover_async(issuer, http)
            .await
            .map_err(|_| OidcFailure::ProviderUnavailable)?;
        APIConfig::parse(metadata.authorization_endpoint().as_str())
            .map_err(|_| OidcFailure::InvalidConfiguration)?;
        let token_endpoint = metadata
            .token_endpoint()
            .ok_or(OidcFailure::InvalidConfiguration)?;
        APIConfig::parse(token_endpoint.as_str()).map_err(|_| OidcFailure::InvalidConfiguration)?;

        Ok(Self {
            metadata,
            client_id: config.public_client_id.clone(),
        })
    }

    pub fn begin(&self, browser: &impl Browser) -> OidcResult<LoginAttempt> {
        let listener =
            TcpListener::bind("127.0.0.1:0").map_err(|_| OidcFailure::InvalidCallback)?;
        listener
            .set_nonblocking(true)
            .map_err(|_| OidcFailure::InvalidCallback)?;
        let address = listener
            .local_addr()
            .map_err(|_| OidcFailure::InvalidCallback)?;
        let redirect = RedirectUrl::new(format!("http://{address}/oidc/callback"))
            .map_err(|_| OidcFailure::InvalidCallback)?;
        let client = CoreClient::from_provider_metadata(
            self.metadata.clone(),
            ClientId::new(self.client_id.clone()),
            None,
        )
        .set_auth_type(AuthType::RequestBody)
        .set_redirect_uri(redirect.clone());
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, state, nonce) = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .set_pkce_challenge(challenge)
            .url();

        browser.open(url.as_str())?;

        Ok(LoginAttempt {
            listener,
            redirect,
            state,
            nonce,
            verifier,
        })
    }

    pub async fn complete(
        &self,
        attempt: LoginAttempt,
        cancellation: LoginCancellation,
        http: &APITransport,
    ) -> OidcResult<OidcSession> {
        let LoginAttempt {
            listener,
            redirect,
            state,
            nonce,
            verifier,
        } = attempt;
        let callback_redirect = redirect.clone();
        let callback_cancel = cancellation.clone();
        let expected_issuer = self.metadata.issuer().as_str().to_owned();
        let code = tauri::async_runtime::spawn_blocking(move || {
            wait_for_callback(
                listener,
                &callback_redirect,
                &state,
                &expected_issuer,
                &callback_cancel,
                Duration::from_secs(300),
            )
        })
        .await
        .map_err(|_| OidcFailure::InvalidCallback)??;
        let client = CoreClient::from_provider_metadata(
            self.metadata.clone(),
            ClientId::new(self.client_id.clone()),
            None,
        )
        .set_auth_type(AuthType::RequestBody)
        .set_redirect_uri(redirect);
        let response = client
            .exchange_code(code)
            .map_err(|_| OidcFailure::InvalidConfiguration)?
            .set_pkce_verifier(verifier)
            .request_async(http)
            .await
            .map_err(|_| OidcFailure::ProviderUnavailable)?;

        if cancellation.0.load(Ordering::Relaxed) {
            return Err(OidcFailure::Cancelled);
        }

        self.verify(response, nonce, None)
    }

    fn verify(
        &self,
        response: CoreTokenResponse,
        nonce: Nonce,
        previous: Option<&OidcSession>,
    ) -> OidcResult<OidcSession> {
        if response.token_type() != &openidconnect::core::CoreTokenType::Bearer {
            return Err(OidcFailure::InvalidToken);
        }

        let client = CoreClient::from_provider_metadata(
            self.metadata.clone(),
            ClientId::new(self.client_id.clone()),
            None,
        );
        let verifier = client.id_token_verifier();
        let (issuer, subject) = if let Some(id_token) = response.id_token() {
            // Refresh may omit nonce; if present, it must match the original login.
            // https://openid.net/specs/openid-connect-core-1_0.html#RefreshTokenResponse
            let claims = id_token
                .claims(&verifier, |received: Option<&Nonce>| match received {
                    Some(received)
                        if Sha256::digest(received.secret().as_bytes())
                            == Sha256::digest(nonce.secret().as_bytes()) =>
                    {
                        Ok(())
                    }
                    None if previous.is_some() => Ok(()),
                    _ => Err("nonce inválido".to_owned()),
                })
                .map_err(|_| OidcFailure::InvalidToken)?;

            if let Some(expected) = claims.access_token_hash() {
                let actual = AccessTokenHash::from_token(
                    response.access_token(),
                    id_token
                        .signing_alg()
                        .map_err(|_| OidcFailure::InvalidToken)?,
                    id_token
                        .signing_key(&verifier)
                        .map_err(|_| OidcFailure::InvalidToken)?,
                )
                .map_err(|_| OidcFailure::InvalidToken)?;

                if actual != *expected {
                    return Err(OidcFailure::InvalidToken);
                }
            }

            (
                claims.issuer().as_str().to_owned(),
                claims.subject().as_str().to_owned(),
            )
        } else {
            let previous = previous.ok_or(OidcFailure::InvalidToken)?;
            (previous.issuer.clone(), previous.subject.clone())
        };

        if previous.is_some_and(|prior| prior.issuer != issuer || prior.subject != subject) {
            return Err(OidcFailure::InvalidToken);
        }

        let expires_at = response
            .expires_in()
            .map(|duration| {
                SystemTime::now()
                    .checked_add(duration)
                    .ok_or(OidcFailure::InvalidToken)
            })
            .transpose()?;
        let access = Secret::new(response.access_token().secret().to_owned())
            .map_err(|_| OidcFailure::InvalidToken)?;
        let refresh = response
            .refresh_token()
            .map(|token| Secret::new(token.secret().to_owned()))
            .transpose()
            .map_err(|_| OidcFailure::InvalidToken)?
            .or_else(|| {
                previous
                    .and_then(|prior| prior.refresh.as_ref())
                    .and_then(|token| Secret::new(token.expose().to_owned()).ok())
            });

        Ok(OidcSession {
            issuer,
            subject,
            access,
            refresh,
            nonce,
            expires_at,
        })
    }

    pub async fn refresh(
        &self,
        session: &OidcSession,
        http: &APITransport,
    ) -> OidcResult<OidcSession> {
        if session.issuer != self.metadata.issuer().as_str() {
            return Err(OidcFailure::InvalidConfiguration);
        }

        let refresh = session.refresh.as_ref().ok_or(OidcFailure::InvalidToken)?;
        let token = RefreshToken::new(refresh.expose().to_owned());
        let client = CoreClient::from_provider_metadata(
            self.metadata.clone(),
            ClientId::new(self.client_id.clone()),
            None,
        )
        .set_auth_type(AuthType::RequestBody);
        let response = client
            .exchange_refresh_token(&token)
            .map_err(|_| OidcFailure::InvalidConfiguration)?
            .request_async(http)
            .await
            .map_err(|_| OidcFailure::ProviderUnavailable)?;

        self.verify(response, session.nonce.clone(), Some(session))
    }
}

fn wait_for_callback(
    listener: TcpListener,
    redirect: &RedirectUrl,
    state: &CsrfToken,
    issuer: &str,
    cancellation: &LoginCancellation,
    timeout: Duration,
) -> OidcResult<AuthorizationCode> {
    let deadline = Instant::now() + timeout;

    while Instant::now() < deadline {
        if cancellation.0.load(Ordering::Relaxed) {
            return Err(OidcFailure::Cancelled);
        }

        let mut stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(20));
                continue;
            }
            Err(_) => return Err(OidcFailure::InvalidCallback),
        };

        // Windows can retain the listener's nonblocking mode on accepted sockets.
        // https://doc.rust-lang.org/1.99.1/std/net/struct.TcpStream.html#method.set_nonblocking
        stream
            .set_nonblocking(false)
            .map_err(|_| OidcFailure::InvalidCallback)?;
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .map_err(|_| OidcFailure::InvalidCallback)?;
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .map_err(|_| OidcFailure::InvalidCallback)?;
        let mut bytes = Vec::new();

        while !bytes.windows(4).any(|window| window == b"\r\n\r\n") {
            let mut buffer = [0; 1024];
            let count = stream
                .read(&mut buffer)
                .map_err(|_| OidcFailure::InvalidCallback)?;

            if count == 0 || bytes.len() + count > 8192 {
                return Err(OidcFailure::InvalidCallback);
            }

            bytes.extend_from_slice(&buffer[..count]);
        }

        let request = std::str::from_utf8(&bytes).map_err(|_| OidcFailure::InvalidCallback)?;
        let expected_host = listener
            .local_addr()
            .map_err(|_| OidcFailure::InvalidCallback)?
            .to_string();
        let hosts: Vec<_> = request
            .lines()
            .skip(1)
            .filter_map(|line| line.split_once(':'))
            .filter(|(name, _)| name.eq_ignore_ascii_case("host"))
            .collect();

        if hosts.len() != 1 || hosts[0].1.trim() != expected_host {
            return Err(OidcFailure::InvalidCallback);
        }

        let mut line = request
            .lines()
            .next()
            .ok_or(OidcFailure::InvalidCallback)?
            .split_whitespace();
        let method = line.next();
        let target = line.next().ok_or(OidcFailure::InvalidCallback)?;

        if method != Some("GET") || !target.starts_with("/oidc/callback?") {
            let _ = stream.write_all(
                b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            );

            continue;
        }

        let url = reqwest::Url::parse(redirect.as_str())
            .map_err(|_| OidcFailure::InvalidCallback)?
            .join(target)
            .map_err(|_| OidcFailure::InvalidCallback)?;
        let pairs: Vec<_> = url.query_pairs().collect();

        if ["state", "code", "error", "iss"]
            .iter()
            .any(|name| pairs.iter().filter(|(key, _)| key == name).count() > 1)
        {
            return Err(OidcFailure::InvalidCallback);
        }

        let unique = |name: &str| {
            let values: Vec<_> = pairs.iter().filter(|(key, _)| key == name).collect();

            if values.len() == 1 {
                Some(values[0].1.to_string())
            } else {
                None
            }
        };

        let returned_state = unique("state").ok_or(OidcFailure::InvalidCallback)?;

        if unique("iss").is_some_and(|value| value != issuer)
            || (unique("code").is_some() && unique("error").is_some())
        {
            return Err(OidcFailure::InvalidCallback);
        }

        // Hash before comparison so a state prefix cannot affect comparison time.
        if Sha256::digest(returned_state.as_bytes()) != Sha256::digest(state.secret().as_bytes()) {
            return Err(OidcFailure::InvalidCallback);
        }

        let result = if unique("error").is_some() {
            Err(OidcFailure::Cancelled)
        } else {
            unique("code")
                .filter(|value| !value.is_empty())
                .map(AuthorizationCode::new)
                .ok_or(OidcFailure::InvalidCallback)
        };

        let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nCache-Control: no-store\r\nConnection: close\r\n\r\nAcceso recibido. Cierre esta pestana.");

        return result;
    }

    Err(OidcFailure::Cancelled)
}

#[cfg(test)]
#[path = "oidc_tests.rs"]
mod tests;
