//! Préférences strictement locales de Bridget Desktop.
//!
//! Elles ne traversent jamais le tunnel SSH : un serveur ne reçoit ni nom
//! d'utilisateur, ni thème, ni fuseau, ni choix de police du Mac.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use uuid::Uuid;

const FORMAT_VERSION: u8 = 3;

/// Autorisations d affichage strictement locales. Elles ne sont ni un jeton,
/// ni une permission de tunnel, ni une capacité qu un serveur peut élargir.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ContentSecurityPreferences {
    pub external_links: bool,
    pub file_references: bool,
    pub remote_images: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FleetSortCriterion {
    pub field: String,
    pub direction: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DesktopPreferences {
    pub display_name: String,
    pub color_scheme: String,
    pub timezone: String,
    pub font_size_px: u8,
    #[serde(default)]
    pub content_security: ContentSecurityPreferences,
    #[serde(default)]
    pub pinned_agent_keys: Vec<String>,
    #[serde(default)]
    pub unpinned_coordinator_keys: Vec<String>,
    #[serde(default)]
    pub sort_criteria: Vec<FleetSortCriterion>,
    #[serde(default)]
    pub collapsed_group_keys: Vec<String>,
}

impl Default for DesktopPreferences {
    fn default() -> Self {
        Self {
            display_name: String::new(),
            color_scheme: "system".to_string(),
            timezone: "system".to_string(),
            font_size_px: 16,
            content_security: ContentSecurityPreferences::default(),
            pinned_agent_keys: Vec::new(),
            unpinned_coordinator_keys: Vec::new(),
            sort_criteria: Vec::new(),
            collapsed_group_keys: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum PreferencesStoreError {
    Io(std::io::Error),
    Json(serde_json::Error),
    InvalidPreferences,
    UnsupportedVersion(u64),
}

impl fmt::Display for PreferencesStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "Préférences locales indisponibles : {error}"),
            Self::Json(error) => write!(formatter, "Préférences locales illisibles : {error}"),
            Self::InvalidPreferences => formatter.write_str("Préférences locales invalides."),
            Self::UnsupportedVersion(version) => {
                write!(
                    formatter,
                    "Version de préférences non prise en charge : {version}."
                )
            }
        }
    }
}

impl std::error::Error for PreferencesStoreError {}

impl From<std::io::Error> for PreferencesStoreError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for PreferencesStoreError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub struct PreferencesStore {
    path: PathBuf,
}

impl PreferencesStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn load(&self) -> Result<DesktopPreferences, PreferencesStoreError> {
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(DesktopPreferences::default());
            }
            Err(error) => return Err(error.into()),
        };
        let value: serde_json::Value = match serde_json::from_slice(&bytes) {
            Ok(value) => value,
            Err(_) => return Ok(DesktopPreferences::default()),
        };
        match value.get("version").and_then(serde_json::Value::as_u64) {
            Some(version) if version == u64::from(FORMAT_VERSION) || version == 2 => {
                let document: PreferencesDocument = match serde_json::from_value(value) {
                    Ok(document) => document,
                    Err(_) => return Ok(DesktopPreferences::default()),
                };
                if validate(&document.preferences).is_err() {
                    return Ok(DesktopPreferences::default());
                }
                Ok(document.preferences)
            }
            Some(1) => {
                let document: PreferencesDocumentV1 = match serde_json::from_value(value) {
                    Ok(document) => document,
                    Err(_) => return Ok(DesktopPreferences::default()),
                };
                let migrated = DesktopPreferences {
                    display_name: document.preferences.display_name,
                    color_scheme: document.preferences.color_scheme,
                    timezone: document.preferences.timezone,
                    font_size_px: document.preferences.font_size_px,
                    content_security: ContentSecurityPreferences {
                        external_links: true,
                        file_references: true,
                        remote_images: true,
                    },
                    ..DesktopPreferences::default()
                };
                if validate(&migrated).is_err() {
                    return Ok(DesktopPreferences::default());
                }
                Ok(migrated)
            }
            _ => Ok(DesktopPreferences::default()),
        }
    }

    pub fn save(&self, preferences: DesktopPreferences) -> Result<(), PreferencesStoreError> {
        validate(&preferences)?;
        let parent = self
            .path
            .parent()
            .ok_or(PreferencesStoreError::InvalidPreferences)?;
        fs::create_dir_all(parent)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
        }
        let document = PreferencesDocument {
            version: FORMAT_VERSION,
            preferences,
        };
        let mut bytes = serde_json::to_vec_pretty(&document)?;
        bytes.push(b'\n');
        let temporary = parent.join(format!(".preferences-{}.tmp", Uuid::new_v4()));
        let result = (|| -> Result<(), PreferencesStoreError> {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                file.set_permissions(fs::Permissions::from_mode(0o600))?;
            }
            file.write_all(&bytes)?;
            file.sync_all()?;
            fs::rename(&temporary, &self.path)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PreferencesDocument {
    version: u8,
    preferences: DesktopPreferences,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PreferencesDocumentV1 {
    #[serde(rename = "version")]
    _version: u8,
    preferences: DesktopPreferencesV1,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DesktopPreferencesV1 {
    display_name: String,
    color_scheme: String,
    timezone: String,
    font_size_px: u8,
}

fn validate(preferences: &DesktopPreferences) -> Result<(), PreferencesStoreError> {
    let display_name = preferences.display_name.trim();
    let valid_name = display_name.len() <= 96 && !display_name.chars().any(char::is_control);
    let valid_scheme = matches!(
        preferences.color_scheme.as_str(),
        "system" | "light" | "dark"
    );
    let timezone = preferences.timezone.trim();
    let valid_timezone = timezone == "system"
        || (timezone.len() <= 64
            && timezone.contains('/')
            && timezone.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'_' | b'-' | b'+')
            }));
    let valid_font_size = (13..=24).contains(&preferences.font_size_px);
    let valid_collection = |values: &[String]| {
        values.len() <= 400
            && values.iter().all(|value| {
                !value.trim().is_empty()
                    && value.len() <= 256
                    && !value.chars().any(char::is_control)
            })
            && values
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
                == values.len()
    };
    let valid_sort_criteria = preferences.sort_criteria.len() <= 5
        && preferences.sort_criteria.iter().all(|criterion| {
            matches!(
                criterion.field.as_str(),
                "source" | "state" | "project" | "activity" | "name"
            ) && matches!(criterion.direction.as_str(), "asc" | "desc")
        });
    if valid_name
        && valid_scheme
        && valid_timezone
        && valid_font_size
        && valid_collection(&preferences.pinned_agent_keys)
        && valid_collection(&preferences.unpinned_coordinator_keys)
        && valid_collection(&preferences.collapsed_group_keys)
        && valid_sort_criteria
    {
        Ok(())
    } else {
        Err(PreferencesStoreError::InvalidPreferences)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferences_locales_sont_versionnees_validees_et_atomiques() {
        let root = std::env::temp_dir().join(format!("bridget-preferences-{}", Uuid::new_v4()));
        let store = PreferencesStore::new(root.join("preferences.json"));
        assert_eq!(store.load().unwrap(), DesktopPreferences::default());
        let preferences = DesktopPreferences {
            display_name: "Camille".to_string(),
            color_scheme: "dark".to_string(),
            timezone: "Europe/Paris".to_string(),
            font_size_px: 18,
            content_security: ContentSecurityPreferences {
                external_links: true,
                file_references: false,
                remote_images: true,
            },
            pinned_agent_keys: vec!["remote:coord".to_string()],
            unpinned_coordinator_keys: vec!["local:coord".to_string()],
            sort_criteria: vec![FleetSortCriterion {
                field: "activity".to_string(),
                direction: "desc".to_string(),
            }],
            collapsed_group_keys: vec!["source:Cartae".to_string()],
        };
        store.save(preferences.clone()).unwrap();
        assert_eq!(store.load().unwrap(), preferences);
        assert!(
            store
                .save(DesktopPreferences {
                    timezone: "invalid timezone".to_string(),
                    ..DesktopPreferences::default()
                })
                .is_err()
        );
        fs::write(root.join("preferences.json"), b"{document corrompu").unwrap();
        assert_eq!(store.load().unwrap(), DesktopPreferences::default());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn preferences_existantes_restent_lisibles_et_epingles_persistantes() {
        let root = std::env::temp_dir().join(format!("bridget-preferences-{}", Uuid::new_v4()));
        let path = root.join("preferences.json");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            &path,
            br#"{ "version": 1, "preferences": { "display_name": "", "color_scheme": "system", "timezone": "system", "font_size_px": 16 } }"#,
        )
        .unwrap();
        let store = PreferencesStore::new(&path);
        let migrated = store.load().unwrap();
        assert_eq!(migrated.pinned_agent_keys, Vec::<String>::new());
        assert_eq!(
            migrated.content_security,
            ContentSecurityPreferences {
                external_links: true,
                file_references: true,
                remote_images: true,
            }
        );
        store
            .save(DesktopPreferences {
                pinned_agent_keys: vec!["local:coordinateur".into()],
                ..DesktopPreferences::default()
            })
            .unwrap();
        assert_eq!(
            store.load().unwrap().pinned_agent_keys,
            vec!["local:coordinateur"]
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn preferences_v2_conservent_les_autorisations_et_initialisent_la_flotte() {
        let root = std::env::temp_dir().join(format!("bridget-preferences-{}", Uuid::new_v4()));
        let path = root.join("preferences.json");
        fs::create_dir_all(&root).unwrap();
        fs::write(
            &path,
            br#"{ "version": 2, "preferences": { "display_name": "", "color_scheme": "system", "timezone": "system", "font_size_px": 16, "content_security": { "external_links": false, "file_references": true, "remote_images": false } } }"#,
        )
        .unwrap();
        let loaded = PreferencesStore::new(&path).load().unwrap();
        assert!(loaded.content_security.file_references);
        assert!(loaded.pinned_agent_keys.is_empty());
        assert!(loaded.sort_criteria.is_empty());
        let _ = fs::remove_dir_all(root);
    }
}
