//! Profils approuvables Maicie et preuve publique de leur définition Bridget.
//!
//! Ce module ne lance rien et ne lit jamais `agents.json`. Il reçoit le profil
//! déclaré par Maicie puis la définition déjà résolue par Bridget ; les deux
//! sont affichés côte à côte au moment de l'approbation ultérieure.

use crate::config::ProfileConfig;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

const MAX_PROFILE_TEXT_BYTES: usize = 128;
const SHA256_HEX_BYTES: usize = 64;

/// Profil chargé, prêt à être proposé mais sans effet de bord.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedProfile {
    pub id: String,
    pub agent_name: String,
    pub display_name: String,
    pub agent_type: String,
    pub model: String,
    /// Valeur opaque du fournisseur : elle n'est jamais convertie en score.
    pub effort: String,
    pub tags: Vec<String>,
    pub personality_ref: String,
    pub tools: Vec<String>,
    pub spawn_order_ref: String,
}

/// Définition effective renvoyée par le contrat public Bridget après la
/// résolution du registre. Les arguments restent exactement ceux de Bridget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// Miroir volontairement strict du contrat Bridget v1. Une extension de la
// définition résolue doit être ajoutée ici avant activation : ne jamais faire
// croire qu'un digest vérifié couvre un champ que Maicie ignore.
#[serde(deny_unknown_fields)]
pub struct ResolvedAgentDefinition {
    pub command: String,
    pub args: Vec<String>,
    pub protocol: String,
    pub forbidden_env: Vec<String>,
    pub pass_env: Vec<String>,
    pub permissions: String,
    pub queue_capacity: usize,
    pub notify_timeout_secs: u64,
    pub mcp: ResolvedMcpDefinition,
    pub digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedMcpDefinition {
    pub interactive: String,
    pub acp_session: bool,
}

/// Données exactes de l'écran d'approbation. `args` est volontairement un
/// `Vec<String>` non transformé : aucune reconstruction déclarative ne peut
/// masquer une différence entre le profil et le registre résolu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalProfileView {
    pub profile: LoadedProfile,
    pub command: String,
    pub args: Vec<String>,
    pub forbidden_env: Vec<String>,
    pub definition_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileError {
    MissingField {
        profile_id: String,
        field: &'static str,
    },
    InvalidField {
        field: &'static str,
        reason: &'static str,
    },
    DuplicateProfile(String),
    ResolvedDefinition(&'static str),
}

impl fmt::Display for ProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingField { profile_id, field } => {
                write!(
                    formatter,
                    "profil {profile_id} sans champ obligatoire {field}"
                )
            }
            Self::InvalidField { field, reason } => {
                write!(formatter, "profil invalide pour {field} : {reason}")
            }
            Self::DuplicateProfile(id) => write!(formatter, "profil duplique : {id}"),
            Self::ResolvedDefinition(reason) => {
                write!(formatter, "definition Bridget resolue invalide : {reason}")
            }
        }
    }
}

impl std::error::Error for ProfileError {}

/// Charge les profils sans deviner leur palette. Les trois informations de
/// gouvernance sont obligatoires à partir de cette frontière, même si le
/// décodeur de configuration reste compatible avec les anciens fichiers.
pub fn load_profiles(configs: &[ProfileConfig]) -> Result<Vec<LoadedProfile>, ProfileError> {
    let mut ids = BTreeSet::new();
    configs
        .iter()
        .map(|config| {
            if !ids.insert(config.id.clone()) {
                return Err(ProfileError::DuplicateProfile(config.id.clone()));
            }
            let agent_type = required(config, "agent_type", config.agent_type.as_deref())?;
            let model = required(config, "model", config.model.as_deref())?;
            let effort = required(config, "effort", config.effort.as_deref())?;
            validate_text("agent_type", &agent_type)?;
            validate_text("model", &model)?;
            validate_text("effort", &effort)?;
            if config.spawn_order_ref.trim().is_empty() {
                return Err(ProfileError::MissingField {
                    profile_id: config.id.clone(),
                    field: "spawn_order_ref",
                });
            }
            Ok(LoadedProfile {
                id: config.id.clone(),
                agent_name: config
                    .agent_name
                    .clone()
                    .unwrap_or_else(|| config.id.clone()),
                display_name: config.display_name.clone(),
                agent_type,
                model,
                effort,
                tags: config.tags.clone(),
                personality_ref: config.personality_ref.clone(),
                tools: config.tools.clone(),
                spawn_order_ref: config.spawn_order_ref.clone(),
            })
        })
        .collect()
}

/// Assemble les données qui pourront être épinglées dans le hash de contexte
/// par T022. Aucune valeur n'est classée ou reformattée ici.
pub fn approval_view(
    profile: LoadedProfile,
    definition: ResolvedAgentDefinition,
) -> Result<ApprovalProfileView, ProfileError> {
    validate_resolved_definition(&definition)?;
    Ok(ApprovalProfileView {
        profile,
        command: definition.command,
        args: definition.args,
        forbidden_env: definition.forbidden_env,
        definition_digest: definition.digest,
    })
}

/// Compare la définition effectivement renvoyée par Bridget au digest figé
/// dans l'approbation. Cette comparaison reste binaire : aucun registre local
/// ni aucune politique de fournisseur ne peut modifier l'autorisation déjà
/// donnée par l'humain.
pub fn definition_digest_matches(pinned: &[u8], returned: &str) -> bool {
    if pinned.len() * 2 != SHA256_HEX_BYTES || returned.len() != SHA256_HEX_BYTES {
        return false;
    }
    returned
        .as_bytes()
        .chunks_exact(2)
        .zip(pinned)
        .all(
            |(hex, expected)| match hex_nibble(hex[0]).zip(hex_nibble(hex[1])) {
                Some((high, low)) => high << 4 | low == *expected,
                None => false,
            },
        )
}

fn required(
    config: &ProfileConfig,
    field: &'static str,
    value: Option<&str>,
) -> Result<String, ProfileError> {
    value
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| ProfileError::MissingField {
            profile_id: config.id.clone(),
            field,
        })
}

fn validate_text(field: &'static str, value: &str) -> Result<(), ProfileError> {
    if value.len() > MAX_PROFILE_TEXT_BYTES || value.chars().any(char::is_control) {
        return Err(ProfileError::InvalidField {
            field,
            reason: "texte hors borne ou caractere de controle",
        });
    }
    Ok(())
}

fn hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn validate_resolved_definition(definition: &ResolvedAgentDefinition) -> Result<(), ProfileError> {
    if definition.command.trim().is_empty() {
        return Err(ProfileError::ResolvedDefinition("command vide"));
    }
    if definition.forbidden_env.is_empty() {
        return Err(ProfileError::ResolvedDefinition("forbidden_env vide"));
    }
    let mut forbidden = BTreeSet::new();
    for variable in &definition.forbidden_env {
        if variable.trim().is_empty() || variable.chars().any(char::is_control) {
            return Err(ProfileError::ResolvedDefinition("forbidden_env invalide"));
        }
        if !forbidden.insert(variable) {
            return Err(ProfileError::ResolvedDefinition("forbidden_env duplique"));
        }
    }
    if definition.digest.len() != SHA256_HEX_BYTES
        || !definition
            .digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(ProfileError::ResolvedDefinition(
            "digest SHA-256 hex invalide",
        ));
    }
    Ok(())
}
