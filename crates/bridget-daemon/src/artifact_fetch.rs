//! Collecte distante explicite, bornée et attestée pour les artefacts.
//!
//! Ce composant n'est jamais appelé par le renderer ni par une iframe. Il est
//! invoqué par Bridget après un geste opérateur et retourne les octets ainsi
//! que les faits nécessaires à la provenance. La collecte est désactivée par
//! défaut et valide chaque destination, y compris après redirection.

use crate::artifact_policy::ArtifactPolicy;
use crate::artifact_types::{
    ArtifactSourceAccess, ArtifactSourceKind, ArtifactSourceV1, sha256_hex,
};
use reqwest::Url;
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use std::io::Read;
use std::net::{IpAddr, ToSocketAddrs};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArtifactFetchPolicy {
    pub enabled: bool,
    pub max_bytes: u64,
    pub max_redirects: u8,
    pub timeout: Duration,
}

impl ArtifactFetchPolicy {
    pub fn from_artifact_policy(policy: ArtifactPolicy) -> Self {
        Self {
            enabled: false,
            max_bytes: policy.binary_blob_max_bytes,
            max_redirects: 3,
            timeout: Duration::from_secs(15),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactFetchResult {
    pub final_url: String,
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
    pub content_digest: String,
    pub fetched_at: i64,
}

impl ArtifactFetchResult {
    pub fn provenance(&self, citation: impl Into<String>) -> ArtifactSourceV1 {
        ArtifactSourceV1 {
            source_kind: ArtifactSourceKind::Remote,
            locator: self.final_url.clone(),
            fetched_at: Some(self.fetched_at),
            content_digest: Some(self.content_digest.clone()),
            citation: citation.into(),
            units: None,
            transformations: Vec::new(),
            access_status: ArtifactSourceAccess::Available,
        }
    }
}

#[derive(Debug)]
pub enum ArtifactFetchError {
    Disabled,
    InvalidUrl(&'static str),
    UnsafeDestination,
    RedirectLimit,
    RedirectMissingLocation,
    HttpStatus(u16),
    TooLarge { max_bytes: u64 },
    Unavailable(String),
}

impl std::fmt::Display for ArtifactFetchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disabled => {
                formatter.write_str("collecte distante désactivée dans les réglages de Bridget")
            }
            Self::InvalidUrl(reason) => write!(formatter, "URL de source invalide: {reason}"),
            Self::UnsafeDestination => {
                formatter.write_str("destination distante refusée par la politique de sécurité")
            }
            Self::RedirectLimit => formatter.write_str("trop de redirections de source"),
            Self::RedirectMissingLocation => formatter.write_str("redirection sans destination"),
            Self::HttpStatus(status) => {
                write!(formatter, "source distante indisponible (HTTP {status})")
            }
            Self::TooLarge { max_bytes } => {
                write!(formatter, "source distante supérieure à {max_bytes} octets")
            }
            Self::Unavailable(detail) => {
                write!(formatter, "source distante indisponible: {detail}")
            }
        }
    }
}

impl std::error::Error for ArtifactFetchError {}

pub struct ArtifactFetcher {
    policy: ArtifactFetchPolicy,
    client: Client,
}

impl ArtifactFetcher {
    pub fn new(policy: ArtifactFetchPolicy) -> Result<Self, ArtifactFetchError> {
        let client = Client::builder()
            .redirect(Policy::none())
            .timeout(policy.timeout)
            .build()
            .map_err(|error| ArtifactFetchError::Unavailable(error.to_string()))?;
        Ok(Self { policy, client })
    }

    pub fn validate_destination(raw_url: &str) -> Result<Url, ArtifactFetchError> {
        let url = Url::parse(raw_url).map_err(|_| ArtifactFetchError::InvalidUrl("syntaxe"))?;
        validate_url(&url)?;
        Ok(url)
    }

    pub fn resolve_redirect(current: &Url, location: &str) -> Result<Url, ArtifactFetchError> {
        let next = current
            .join(location)
            .map_err(|_| ArtifactFetchError::RedirectMissingLocation)?;
        validate_url(&next)?;
        Ok(next)
    }

    pub fn fetch(&self, raw_url: &str) -> Result<ArtifactFetchResult, ArtifactFetchError> {
        if !self.policy.enabled {
            return Err(ArtifactFetchError::Disabled);
        }
        let mut current = Self::validate_destination(raw_url)?;
        for redirect_count in 0..=self.policy.max_redirects {
            let response = self
                .client
                .get(current.clone())
                .send()
                .map_err(|error| ArtifactFetchError::Unavailable(error.to_string()))?;
            if response.status().is_redirection() {
                if redirect_count == self.policy.max_redirects {
                    return Err(ArtifactFetchError::RedirectLimit);
                }
                let location = response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|value| value.to_str().ok())
                    .ok_or(ArtifactFetchError::RedirectMissingLocation)?;
                current = Self::resolve_redirect(&current, location)?;
                continue;
            }
            if !response.status().is_success() {
                return Err(ArtifactFetchError::HttpStatus(response.status().as_u16()));
            }
            if let Some(content_length) = response.content_length()
                && content_length > self.policy.max_bytes
            {
                return Err(ArtifactFetchError::TooLarge {
                    max_bytes: self.policy.max_bytes,
                });
            }
            let content_type = response
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .map(str::to_string);
            let bytes = read_bounded(
                response.take(self.policy.max_bytes.saturating_add(1)),
                self.policy.max_bytes,
            )?;
            let fetched_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| i64::try_from(duration.as_secs()).unwrap_or(i64::MAX))
                .unwrap_or(0);
            return Ok(ArtifactFetchResult {
                final_url: current.to_string(),
                content_type,
                content_digest: sha256_hex(&bytes),
                bytes,
                fetched_at,
            });
        }
        Err(ArtifactFetchError::RedirectLimit)
    }
}

fn read_bounded(mut reader: impl Read, max_bytes: u64) -> Result<Vec<u8>, ArtifactFetchError> {
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|error| ArtifactFetchError::Unavailable(error.to_string()))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > max_bytes {
        return Err(ArtifactFetchError::TooLarge { max_bytes });
    }
    Ok(bytes)
}

fn validate_url(url: &Url) -> Result<(), ArtifactFetchError> {
    if url.scheme() != "https" {
        return Err(ArtifactFetchError::InvalidUrl("HTTPS obligatoire"));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(ArtifactFetchError::InvalidUrl("identifiants URL interdits"));
    }
    if !matches!(url.port(), None | Some(443)) {
        return Err(ArtifactFetchError::InvalidUrl("port HTTPS non standard"));
    }
    let host = url
        .host_str()
        .ok_or(ArtifactFetchError::InvalidUrl("hôte absent"))?;
    if host.eq_ignore_ascii_case("localhost")
        || host.ends_with(".localhost")
        || host.ends_with(".local")
    {
        return Err(ArtifactFetchError::UnsafeDestination);
    }
    if let Ok(address) = host.parse::<IpAddr>() {
        return is_public_ip(address)
            .then_some(())
            .ok_or(ArtifactFetchError::UnsafeDestination);
    }
    let addresses = (host, 443)
        .to_socket_addrs()
        .map_err(|error| ArtifactFetchError::Unavailable(error.to_string()))?
        .map(|address| address.ip())
        .collect::<Vec<_>>();
    if addresses.is_empty() || addresses.iter().any(|address| !is_public_ip(*address)) {
        return Err(ArtifactFetchError::UnsafeDestination);
    }
    Ok(())
}

fn is_public_ip(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(value) => {
            let octets = value.octets();
            let documentation = matches!(
                octets,
                [192, 0, 2, _] | [198, 51, 100, _] | [203, 0, 113, _]
            );
            !value.is_private()
                && !value.is_loopback()
                && !value.is_link_local()
                && !value.is_broadcast()
                && !value.is_unspecified()
                && !value.is_multicast()
                && !documentation
                && octets[0] != 0
                && octets[0] < 224
        }
        IpAddr::V6(value) => {
            let segments = value.segments();
            let documentation = segments[0] == 0x2001 && segments[1] == 0x0db8;
            !value.is_loopback()
                && !value.is_unspecified()
                && !value.is_multicast()
                && !value.is_unique_local()
                && !value.is_unicast_link_local()
                && !documentation
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn politique_refuse_file_http_et_reseau_prive() {
        for url in [
            "file:///tmp/secret",
            "http://example.com/data",
            "https://127.0.0.1/private",
            "https://[::1]/private",
            "https://localhost/private",
            "https://service.local/private",
        ] {
            assert!(ArtifactFetcher::validate_destination(url).is_err(), "{url}");
        }
    }

    #[test]
    fn redirection_est_revalidee_et_collecte_desactivee_par_defaut() {
        let current = Url::parse("https://example.com/start").unwrap();
        assert!(ArtifactFetcher::resolve_redirect(&current, "https://127.0.0.1/").is_err());
        let fetcher = ArtifactFetcher::new(ArtifactFetchPolicy::from_artifact_policy(
            ArtifactPolicy::default(),
        ))
        .unwrap();
        assert!(matches!(
            fetcher.fetch("https://example.com/"),
            Err(ArtifactFetchError::Disabled)
        ));
    }

    #[test]
    fn lecture_bornee_refuse_un_contenu_qui_depasse_le_plafond() {
        assert_eq!(read_bounded(&b"abcd"[..], 4).unwrap(), b"abcd");
        assert!(matches!(
            read_bounded(&b"abcde"[..], 4),
            Err(ArtifactFetchError::TooLarge { max_bytes: 4 })
        ));
    }
}
