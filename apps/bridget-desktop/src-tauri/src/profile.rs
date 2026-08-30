//! Modèle persistant des destinations Bridget Desktop.
//!
//! Un profil décrit uniquement comment joindre un relais Bridget. Il ne contient
//! jamais un jeton de relais, le contenu d'une clé privée, ni un mot de passe.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ConnectionProfile {
    Ssh {
        id: String,
        label: String,
        host: String,
        port: u16,
        user: String,
        identity: SshIdentityRef,
        host_fingerprint: Option<String>,
        #[serde(default)]
        capabilities: Vec<ProfileCapability>,
    },
    Local {
        id: String,
        label: String,
        relay_port: u16,
        #[serde(default)]
        capabilities: Vec<ProfileCapability>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "source", rename_all = "snake_case", deny_unknown_fields)]
pub enum SshIdentityRef {
    Agent,
    File { path: PathBuf },
}

/// Capacité déclarée d'une origine. `RemoteBrowser` est documentaire dans la
/// SPEC-074 : aucun code ne le démarre ni ne le tunnelise encore.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProfileCapability {
    Ui,
    RemoteBrowser,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    Disconnected,
    ConnectingSsh,
    AwaitingHostApproval,
    OpeningTunnel,
    CheckingRelay,
    Connected,
    Reconnecting,
    Failed,
    Closed,
}

/// Etat mémoire d'une connexion. L'endpoint reste volontairement non
/// sérialisable afin que son jeton ne puisse jamais entrer dans profiles.json.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConnectionSession {
    pub profile_id: String,
    pub state: ConnectionState,
    pub retry_count: u8,
    pub last_error: Option<RedactedConnectionError>,
    endpoint: Option<RelayEndpoint>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RedactedConnectionError {
    pub category: ConnectionErrorCategory,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionErrorCategory {
    SshUnavailable,
    HostIdentity,
    Tunnel,
    RelayUnavailable,
    UnsupportedEndpointVersion,
    InvalidEndpoint,
}

#[derive(Clone, Eq, PartialEq)]
pub struct RelayEndpoint {
    pub port: u16,
    token: String,
}

impl fmt::Debug for RelayEndpoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RelayEndpoint")
            .field("port", &self.port)
            .field("token", &"[masqué]")
            .finish()
    }
}

impl RelayEndpoint {
    pub fn new(port: u16, token: String) -> Result<Self, ProfileValidationError> {
        validate_port(port, "Le port du relais")?;
        if token.trim().is_empty() || token.contains(['\n', '\r', '\0']) {
            return Err(ProfileValidationError::new(
                "Le jeton du relais est invalide.",
            ));
        }
        Ok(Self { port, token })
    }

    pub(crate) fn token(&self) -> &str {
        &self.token
    }
}

impl ConnectionSession {
    pub fn disconnected(profile_id: impl Into<String>) -> Self {
        Self {
            profile_id: profile_id.into(),
            state: ConnectionState::Disconnected,
            retry_count: 0,
            last_error: None,
            endpoint: None,
        }
    }

    pub fn endpoint(&self) -> Option<&RelayEndpoint> {
        self.endpoint.as_ref()
    }

    pub(crate) fn set_endpoint(&mut self, endpoint: RelayEndpoint) {
        self.endpoint = Some(endpoint);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileValidationError {
    message: String,
}

impl ProfileValidationError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ProfileValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ProfileValidationError {}

impl ConnectionProfile {
    pub fn id(&self) -> &str {
        match self {
            Self::Ssh { id, .. } | Self::Local { id, .. } => id,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Ssh { label, .. } | Self::Local { label, .. } => label,
        }
    }

    pub fn is_local(&self) -> bool {
        matches!(self, Self::Local { .. })
    }

    pub fn capabilities(&self) -> &[ProfileCapability] {
        match self {
            Self::Ssh { capabilities, .. } | Self::Local { capabilities, .. } => capabilities,
        }
    }

    pub(crate) fn set_host_fingerprint(
        &mut self,
        fingerprint: String,
    ) -> Result<(), ProfileValidationError> {
        validate_host_fingerprint(&fingerprint)?;
        match self {
            Self::Ssh {
                host_fingerprint, ..
            } => {
                *host_fingerprint = Some(fingerprint);
                Ok(())
            }
            Self::Local { .. } => Err(ProfileValidationError::new(
                "Un relais local n'a pas d'identité SSH à approuver.",
            )),
        }
    }

    /// Retourne un résumé affichable qui ne transporte aucun secret.
    pub fn display_origin(&self) -> String {
        match self {
            Self::Ssh {
                user, host, port, ..
            } => format!("SSH {user}@{host}:{port}"),
            Self::Local { relay_port, .. } => format!("local 127.0.0.1:{relay_port}"),
        }
    }

    pub fn validate(&self) -> Result<(), ProfileValidationError> {
        validate_profile_id(self.id())?;
        validate_label(self.label())?;

        match self {
            Self::Ssh {
                host,
                port,
                user,
                identity,
                host_fingerprint,
                ..
            } => {
                validate_host(host)?;
                validate_port(*port, "Le port SSH")?;
                validate_user(user)?;
                validate_identity(identity)?;
                if let Some(fingerprint) = host_fingerprint {
                    validate_host_fingerprint(fingerprint)?;
                }
            }
            Self::Local { relay_port, .. } => {
                validate_port(*relay_port, "Le port du relais local")?;
            }
        }

        Ok(())
    }
}

/// Entrée fermée de la coque locale. L'identifiant est produit par le backend
/// pour éviter qu'une webview choisisse une identité déjà utilisée.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProfileDraft {
    Ssh {
        label: String,
        host: String,
        port: u16,
        user: String,
        identity: SshIdentityRef,
    },
    Local {
        label: String,
        relay_port: u16,
    },
}

impl ProfileDraft {
    pub fn into_profile(self, id: impl Into<String>) -> ConnectionProfile {
        let id = id.into();
        match self {
            Self::Ssh {
                label,
                host,
                port,
                user,
                identity,
            } => ConnectionProfile::Ssh {
                id,
                label,
                host,
                port,
                user,
                identity,
                host_fingerprint: None,
                capabilities: vec![ProfileCapability::Ui],
            },
            Self::Local { label, relay_port } => ConnectionProfile::Local {
                id,
                label,
                relay_port,
                capabilities: vec![ProfileCapability::Ui],
            },
        }
    }
}

fn validate_profile_id(value: &str) -> Result<(), ProfileValidationError> {
    if value.trim().is_empty() {
        return Err(ProfileValidationError::new(
            "L'identifiant du profil est requis.",
        ));
    }
    if value.len() > 128 || value.contains(char::is_whitespace) {
        return Err(ProfileValidationError::new(
            "L'identifiant du profil doit être court et sans espace.",
        ));
    }
    Ok(())
}

fn validate_label(value: &str) -> Result<(), ProfileValidationError> {
    if value.trim().is_empty() || value.len() > 96 || value.contains(['\n', '\r', '\0']) {
        return Err(ProfileValidationError::new(
            "Le nom du profil doit contenir entre 1 et 96 caractères sur une ligne.",
        ));
    }
    if value.contains("-----BEGIN") || value.contains("PRIVATE KEY") {
        return Err(ProfileValidationError::new(
            "Le nom du profil ne peut pas contenir de matière de clé privée.",
        ));
    }
    Ok(())
}

fn validate_host(value: &str) -> Result<(), ProfileValidationError> {
    if value.is_empty()
        || value.len() > 253
        || value.starts_with('-')
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | ':' | '-')
        })
    {
        return Err(ProfileValidationError::new(
            "L'hôte doit être un nom DNS ou une adresse IP explicite.",
        ));
    }
    Ok(())
}

fn validate_port(value: u16, label: &str) -> Result<(), ProfileValidationError> {
    if value == 0 {
        return Err(ProfileValidationError::new(format!("{label} est requis.")));
    }
    Ok(())
}

fn validate_user(value: &str) -> Result<(), ProfileValidationError> {
    if value.is_empty()
        || value.len() > 64
        || value.starts_with('-')
        || !value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        })
    {
        return Err(ProfileValidationError::new(
            "L'utilisateur SSH doit être un identifiant explicite sans espace.",
        ));
    }
    Ok(())
}

fn validate_identity(identity: &SshIdentityRef) -> Result<(), ProfileValidationError> {
    if let SshIdentityRef::File { path } = identity {
        let value = path.to_string_lossy();
        if !path.is_absolute() || value.contains(['\n', '\r', '\0']) {
            return Err(ProfileValidationError::new(
                "Le chemin de clé SSH doit être absolu et tenir sur une ligne.",
            ));
        }
    }
    Ok(())
}

fn validate_host_fingerprint(value: &str) -> Result<(), ProfileValidationError> {
    if !value.starts_with("SHA256:") || value.len() < 16 || value.contains(char::is_whitespace) {
        return Err(ProfileValidationError::new(
            "L'empreinte SSH doit être au format SHA256:... sans espace.",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ConnectionProfile, ConnectionState, ProfileCapability, SshIdentityRef};
    use std::path::PathBuf;

    fn ssh_profile() -> ConnectionProfile {
        ConnectionProfile::Ssh {
            id: "production".into(),
            label: "Cartae.app".into(),
            host: "cartae.app".into(),
            port: 2222,
            user: "moi".into(),
            identity: SshIdentityRef::File {
                path: PathBuf::from("/Users/moi/.ssh/id_ed25519"),
            },
            host_fingerprint: Some("SHA256:abcdefghijklmnopqrstuvwxyz123456".into()),
            capabilities: vec![ProfileCapability::Ui],
        }
    }

    #[test]
    fn un_profil_ssh_valide_ne_contient_pas_de_secret() {
        let profile = ssh_profile();
        profile.validate().expect("profil valide");

        let json = serde_json::to_string(&profile).expect("sérialisation");
        assert!(json.contains("id_ed25519"));
        assert!(!json.contains("PRIVATE KEY"));
        assert!(!json.contains("BEGIN OPENSSH"));
    }

    #[test]
    fn les_champs_ssh_sont_refuses_dans_un_profil_local_par_le_modele() {
        let local = ConnectionProfile::Local {
            id: "poste".into(),
            label: "Ce Mac".into(),
            relay_port: 17888,
            capabilities: vec![ProfileCapability::Ui],
        };
        local.validate().expect("profil local valide");
        let json = serde_json::to_string(&local).expect("sérialisation");
        assert!(!json.contains("host"));
        assert!(!json.contains("identity"));
        assert!(!json.contains("fingerprint"));
    }

    #[test]
    fn un_hote_ambigu_et_une_etiquette_de_cle_sont_refuses() {
        let mut profile = ssh_profile();
        if let ConnectionProfile::Ssh { host, .. } = &mut profile {
            *host = "-oProxyCommand=evil".into();
        }
        assert!(profile.validate().is_err());

        let dangerous = ConnectionProfile::Local {
            id: "danger".into(),
            label: "-----BEGIN PRIVATE KEY-----".into(),
            relay_port: 17888,
            capabilities: vec![ProfileCapability::Ui],
        };
        assert!(dangerous.validate().is_err());
    }

    #[test]
    fn le_jeton_de_relais_ne_peut_pas_etre_serialise_ni_journalise() {
        let endpoint =
            super::RelayEndpoint::new(17888, "token-de-fixture".into()).expect("endpoint valide");
        let session = super::ConnectionSession::disconnected("production");
        assert!(!format!("{endpoint:?}").contains("token-de-fixture"));
        assert!(!format!("{session:?}").contains("token-de-fixture"));
        assert!(serde_json::to_string(&ConnectionState::CheckingRelay).is_ok());
    }

    #[test]
    fn le_brouillon_rejette_un_champ_ssh_dans_un_profil_local() {
        let result = serde_json::from_str::<super::ProfileDraft>(
            r#"{"kind":"local","label":"Ce Mac","relay_port":17888,"host":"interdit"}"#,
        );
        assert!(result.is_err());
    }
}
