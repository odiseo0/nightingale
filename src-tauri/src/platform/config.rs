use crate::shared_kernel::{NightingaleError, Result};
use reqwest::Url;

const API_BASE_URL: &str = "NIGHTINGALE_API_BASE_URL";

/// Read this only when a remote use case starts. Local work needs no API URL.
pub struct APIConfig {
    base_url: Url,
}

impl APIConfig {
    pub fn from_environment() -> Result<Self> {
        Self::from_lookup(|key| std::env::var(key).ok())
    }

    fn from_lookup(lookup: impl FnOnce(&str) -> Option<String>) -> Result<Self> {
        let value = lookup(API_BASE_URL)
            .ok_or_else(|| NightingaleError::validation("falta NIGHTINGALE_API_BASE_URL"))?;

        Self::parse(&value)
    }

    pub fn parse(value: &str) -> Result<Self> {
        Self::parse_for_build(value, cfg!(debug_assertions))
    }

    fn parse_for_build(value: &str, development: bool) -> Result<Self> {
        let invalid = || NightingaleError::validation("URL de Nightingale API inválida");
        let mut url = Url::parse(value).map_err(|_| invalid())?;
        let loopback = url.host_str().is_some_and(|host| {
            host == "localhost"
                || host
                    .trim_matches(['[', ']'])
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| ip.is_loopback())
        });

        if url.host_str().is_none()
            || (!url.username().is_empty() || url.password().is_some())
            || url.query().is_some()
            || url.fragment().is_some()
            || !(url.scheme() == "https" || (development && loopback && url.scheme() == "http"))
        {
            return Err(invalid());
        }

        if !url.path().ends_with('/') {
            url.set_path(&format!("{}/", url.path()));
        }

        Ok(Self { base_url: url })
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub fn client_version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    pub fn platform(&self) -> &'static str {
        std::env::consts::OS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_explicit_environment_without_cloud_or_oidc_defaults() {
        assert!(APIConfig::from_lookup(|_| None).is_err());
        let config = APIConfig::from_lookup(|key| {
            assert_eq!(key, API_BASE_URL);
            Some("https://api.example/base".into())
        })
        .unwrap();
        assert_eq!(config.base_url().as_str(), "https://api.example/base/");
        assert_eq!(config.client_version(), env!("CARGO_PKG_VERSION"));
        assert_eq!(config.platform(), std::env::consts::OS);
    }

    #[test]
    fn permits_http_only_for_development_loopback() {
        for url in [
            "http://127.0.0.1:8000",
            "http://localhost:8000",
            "http://[::1]:8000",
        ] {
            assert!(APIConfig::parse_for_build(url, true).is_ok());
            assert!(APIConfig::parse_for_build(url, false).is_err());
        }
        assert!(APIConfig::parse_for_build("https://api.example", false).is_ok());

        for url in [
            "http://api.example",
            "http://192.168.1.1",
            "file:///tmp/api",
            "",
            "relative",
            "https://user:secret@api.example",
            "https://api.example?token=secret",
            "https://api.example#secret",
        ] {
            let error = APIConfig::parse_for_build(url, true).err().unwrap();
            assert_eq!(error.code, crate::shared_kernel::ErrorCode::ValidationError);
            assert!(!error.message.contains("secret"));
            assert!(!error.message.contains(url) || url.is_empty());
        }
    }
}
