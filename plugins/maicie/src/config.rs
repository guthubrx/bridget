//! Configuration declarative et validee de Maicie.
//!
//! La configuration ne contient ni secret ni politique implicite : chaque
//! delai, profil et reference de lancement est fourni explicitement par
//! l'utilisateur. La validation est effectuee avant toute ouverture de socket
//! ou de base SQLite.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

const CONFIG_VERSION: u32 = 1;
const MAX_CONFIG_BYTES: u64 = 1024 * 1024;
const MAX_SOCKET_PATH_BYTES: usize = 103;
const MAX_DATABASE_PATH_BYTES: usize = 1024;
pub(crate) const MAX_TIMEOUT_SECS: u64 = 7 * 24 * 60 * 60;
const MAX_PROFILES: usize = 128;
const MAX_TAGS_PER_PROFILE: usize = 64;
const MAX_TOOLS_PER_PROFILE: usize = 64;
const MAX_SHORT_TEXT_BYTES: usize = 128;
const MAX_REFERENCE_BYTES: usize = 1024;
/// Une consultation ne doit pas devenir un pseudo-runtime résident ni retenir
/// indéfiniment le CLI. Au-delà, le statut rend explicitement l'observation
/// inconnue plutôt que de conserver un fait périmé.
pub const MAX_STATUS_CAPTURE_BUDGET_MS: u64 = 30_000;

/// Configuration complete de Maicie, chargee avant toute I/O Bridget/SQLite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaicieConfig {
    pub version: u32,
    pub bridget_socket: PathBuf,
    pub database_path: PathBuf,
    pub durations: DurationClasses,
    /// Budget global optionnel de capture Attach effectué par `maicie status`.
    /// Son absence désactive la capture : le statut le dit explicitement et ne
    /// remplace jamais cette absence par une valeur implicite.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_capture_budget_ms: Option<u64>,
    /// Chemin absolu du journal de catalogue v1 déclaré pour le projet hôte.
    /// Absent : les commandes hors greffière restent valides ; `registre`
    /// exige sa présence. Aucune valeur implicite n'est inventée.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalogue_path: Option<PathBuf>,
    pub profiles: Vec<ProfileConfig>,
}

/// Delais passifs transmis tels quels au contrat Bridget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DurationClasses {
    pub short_secs: u64,
    pub normal_secs: u64,
    pub long_secs: u64,
}

impl DurationClasses {
    /// Verifie que les trois classes sont explicites, distinctes et bornees.
    pub fn validate(self) -> Result<(), ConfigError> {
        if self.short_secs == 0
            || self.normal_secs == 0
            || self.long_secs == 0
            || self.long_secs > MAX_TIMEOUT_SECS
        {
            return Err(ConfigError::validation(
                "durations",
                format!("chaque delai doit etre compris entre 1 et {MAX_TIMEOUT_SECS} secondes"),
            ));
        }
        if !(self.short_secs < self.normal_secs && self.normal_secs < self.long_secs) {
            return Err(ConfigError::validation(
                "durations",
                "short_secs, normal_secs et long_secs doivent etre strictement croissants",
            ));
        }
        Ok(())
    }
}

/// Profil declaratif : aucun tag, outil ou lancement n'est deduit de son nom.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileConfig {
    /// Clé stable de gouvernance du profil, distincte du nom runtime Bridget.
    pub id: String,
    /// Identité exacte publiée par l'annuaire Bridget. L'absence conserve la
    /// compatibilité avec les profils historiques dont `id` était ce nom.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_name: Option<String>,
    /// Type Bridget affiché à l'approbation. Son absence reste lisible dans
    /// les configurations historiques, mais interdit une activation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
    /// Modèle opaque déclaré pour le profil ; jamais inféré ni classé.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Niveau d'effort opaque affiché tel quel à l'approbation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    pub display_name: String,
    pub tags: Vec<String>,
    pub personality_ref: String,
    pub tools: Vec<String>,
    pub spawn_order_ref: String,
}

impl MaicieConfig {
    /// Charge puis valide un fichier JSON borne. Aucune valeur par defaut ne
    /// complete silencieusement une configuration incomplete.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let mut file = File::open(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let mut bytes = Vec::new();
        file.by_ref()
            .take(MAX_CONFIG_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|source| ConfigError::Read {
                path: path.to_path_buf(),
                source,
            })?;
        if bytes.len() as u64 > MAX_CONFIG_BYTES {
            return Err(ConfigError::validation(
                path.display().to_string(),
                format!("le fichier depasse {MAX_CONFIG_BYTES} octets"),
            ));
        }

        let config: Self = serde_json::from_slice(&bytes).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        })?;
        // Pas de racine projet déduite du parent du fichier de config : un
        // dépôt peut placer la config dans un sous-dossier (.maicie/) tandis
        // que le journal reste à la racine versionnée. L'appartenance au
        // projet reste une déclaration absolue explicite.
        config.validate()?;
        Ok(config)
    }

    /// Valide une configuration construite en memoire avant tout effet de bord.
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.validate_in_project(None)
    }

    /// Valide la configuration ; `project_root` borne optionnellement le
    /// catalogue (tests et outils qui connaissent déjà la racine hôte).
    pub fn validate_in_project(&self, project_root: Option<&Path>) -> Result<(), ConfigError> {
        if self.version != CONFIG_VERSION {
            return Err(ConfigError::validation(
                "version",
                format!(
                    "version {0} attendue, {1} recue",
                    CONFIG_VERSION, self.version
                ),
            ));
        }
        validate_absolute_path("bridget_socket", &self.bridget_socket)?;
        validate_absolute_path("database_path", &self.database_path)?;
        let socket_len = self.bridget_socket.as_os_str().as_encoded_bytes().len();
        if socket_len > MAX_SOCKET_PATH_BYTES {
            return Err(ConfigError::validation(
                "bridget_socket",
                format!("le chemin Unix fait {socket_len} octets, maximum {MAX_SOCKET_PATH_BYTES}"),
            ));
        }
        let database_len = self.database_path.as_os_str().as_encoded_bytes().len();
        if database_len > MAX_DATABASE_PATH_BYTES {
            return Err(ConfigError::validation(
                "database_path",
                format!(
                    "le chemin SQLite fait {database_len} octets, maximum {MAX_DATABASE_PATH_BYTES}"
                ),
            ));
        }
        if self
            .database_path
            .file_name()
            .and_then(|name| name.to_str())
            == Some("bridget.db")
        {
            return Err(ConfigError::validation(
                "database_path",
                "Maicie doit utiliser une base privee distincte de bridget.db",
            ));
        }
        if self.database_path == self.bridget_socket {
            return Err(ConfigError::validation(
                "database_path",
                "la base SQLite et le socket Bridget doivent etre distincts",
            ));
        }
        self.durations.validate()?;
        match self.status_capture_budget_ms {
            Some(budget_ms) if budget_ms == 0 || budget_ms > MAX_STATUS_CAPTURE_BUDGET_MS => {
                return Err(ConfigError::validation(
                    "status_capture_budget_ms",
                    format!(
                        "le budget de capture doit etre compris entre 1 et {MAX_STATUS_CAPTURE_BUDGET_MS} millisecondes"
                    ),
                ));
            }
            _ => {}
        }
        if let Some(catalogue_path) = &self.catalogue_path {
            validate_catalogue_path_field(catalogue_path, project_root)?;
            if catalogue_path == &self.database_path || catalogue_path == &self.bridget_socket {
                return Err(ConfigError::validation(
                    "catalogue_path",
                    "le catalogue doit etre distinct du socket Bridget et de la base SQLite",
                ));
            }
        }
        validate_profiles(&self.profiles)
    }
}

fn validate_absolute_path(field: &'static str, path: &Path) -> Result<(), ConfigError> {
    if !path.is_absolute() {
        return Err(ConfigError::validation(
            field,
            "un chemin absolu est obligatoire",
        ));
    }
    if path.as_os_str().as_encoded_bytes().contains(&0) {
        return Err(ConfigError::validation(
            field,
            "le chemin contient un octet nul",
        ));
    }
    Ok(())
}

fn validate_catalogue_path_field(
    catalogue_path: &Path,
    project_root: Option<&Path>,
) -> Result<(), ConfigError> {
    validate_absolute_path("catalogue_path", catalogue_path)?;
    crate::catalogue::validate_catalogue_path(catalogue_path, project_root)
        .map_err(|error| ConfigError::validation("catalogue_path", error.to_string()))
}

fn validate_profiles(profiles: &[ProfileConfig]) -> Result<(), ConfigError> {
    if profiles.len() > MAX_PROFILES {
        return Err(ConfigError::validation(
            "profiles",
            format!("{MAX_PROFILES} profils au maximum"),
        ));
    }

    let mut ids = HashSet::with_capacity(profiles.len());
    let mut agent_names = HashSet::with_capacity(profiles.len());
    for profile in profiles {
        let field = format!("profiles.{}", profile.id);
        validate_slug(&format!("{field}.id"), &profile.id)?;
        if !ids.insert(profile.id.as_str()) {
            return Err(ConfigError::validation(
                "profiles",
                format!("identifiant de profil duplique : {}", profile.id),
            ));
        }
        let agent_name = profile.agent_name.as_deref().unwrap_or(&profile.id);
        validate_text(
            &format!("{field}.agent_name"),
            agent_name,
            MAX_SHORT_TEXT_BYTES,
        )?;
        if agent_name.chars().any(char::is_control) {
            return Err(ConfigError::validation(
                format!("{field}.agent_name"),
                "le nom d'agent ne peut contenir de caractere de controle",
            ));
        }
        if !agent_names.insert(agent_name) {
            return Err(ConfigError::validation(
                "profiles",
                format!("nom d'agent duplique : {agent_name}"),
            ));
        }
        validate_text(
            &format!("{field}.display_name"),
            &profile.display_name,
            MAX_SHORT_TEXT_BYTES,
        )?;
        validate_reference(
            &format!("{field}.personality_ref"),
            &profile.personality_ref,
        )?;
        validate_reference(
            &format!("{field}.spawn_order_ref"),
            &profile.spawn_order_ref,
        )?;
        validate_unique_values(
            &format!("{field}.tags"),
            &profile.tags,
            MAX_TAGS_PER_PROFILE,
        )?;
        validate_unique_values(
            &format!("{field}.tools"),
            &profile.tools,
            MAX_TOOLS_PER_PROFILE,
        )?;
    }
    Ok(())
}

fn validate_slug(field: &str, value: &str) -> Result<(), ConfigError> {
    if value.is_empty()
        || value.len() > MAX_SHORT_TEXT_BYTES
        || !value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || (index > 0 && matches!(byte, b'-' | b'_'))
        })
    {
        return Err(ConfigError::validation(
            field,
            "un slug ASCII minuscule est requis",
        ));
    }
    Ok(())
}

fn validate_reference(field: &str, value: &str) -> Result<(), ConfigError> {
    validate_text(field, value, MAX_REFERENCE_BYTES)?;
    if value.chars().any(char::is_control) {
        return Err(ConfigError::validation(
            field,
            "une reference ne peut contenir de caractere de controle",
        ));
    }
    Ok(())
}

fn validate_unique_values(
    field: &str,
    values: &[String],
    maximum: usize,
) -> Result<(), ConfigError> {
    if values.len() > maximum {
        return Err(ConfigError::validation(
            field,
            format!("{maximum} valeurs au maximum"),
        ));
    }
    let mut unique = HashSet::with_capacity(values.len());
    for value in values {
        validate_text(field, value, MAX_SHORT_TEXT_BYTES)?;
        if value.chars().any(char::is_control) {
            return Err(ConfigError::validation(
                field,
                "les caracteres de controle sont interdits",
            ));
        }
        if !unique.insert(value.as_str()) {
            return Err(ConfigError::validation(
                field,
                format!("valeur dupliquee : {value}"),
            ));
        }
    }
    Ok(())
}

fn validate_text(field: &str, value: &str, maximum: usize) -> Result<(), ConfigError> {
    if value.trim().is_empty() || value.len() > maximum {
        return Err(ConfigError::validation(
            field,
            format!("une valeur non vide de {maximum} octets maximum est requise"),
        ));
    }
    Ok(())
}

#[derive(Debug)]
pub enum ConfigError {
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
    Validation {
        field: String,
        reason: String,
    },
}

impl ConfigError {
    fn validation(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            reason: reason.into(),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(
                    formatter,
                    "lecture de {} impossible : {source}",
                    path.display()
                )
            }
            Self::Parse { path, source } => {
                write!(
                    formatter,
                    "configuration JSON {} invalide : {source}",
                    path.display()
                )
            }
            Self::Validation { field, reason } => {
                write!(formatter, "configuration invalide pour {field} : {reason}")
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Parse { source, .. } => Some(source),
            Self::Validation { .. } => None,
        }
    }
}
