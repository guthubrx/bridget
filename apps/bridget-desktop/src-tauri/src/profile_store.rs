//! Persistance atomique et versionnée des profils Bridget Desktop.

use crate::profile::{
    ConnectionProfile, ProfileCapability, ProfileValidationError, SshIdentityRef,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const CURRENT_FORMAT_VERSION: u8 = 2;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedProfiles {
    pub profiles: Vec<ConnectionProfile>,
    pub migrated_from_v0: bool,
}

#[derive(Debug)]
pub enum ProfileStoreError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Validation(ProfileValidationError),
    UnsupportedVersion(u64),
    DuplicateId(String),
    InvalidRoot,
}

impl fmt::Display for ProfileStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "Erreur de stockage local : {error}"),
            Self::Json(error) => write!(formatter, "Profils locaux illisibles : {error}"),
            Self::Validation(error) => error.fmt(formatter),
            Self::UnsupportedVersion(version) => {
                write!(
                    formatter,
                    "Version de profils non prise en charge : {version}."
                )
            }
            Self::DuplicateId(id) => write!(formatter, "Identifiant de profil dupliqué : {id}."),
            Self::InvalidRoot => {
                formatter.write_str("Le fichier de profils doit être un objet JSON.")
            }
        }
    }
}

impl std::error::Error for ProfileStoreError {}

impl From<std::io::Error> for ProfileStoreError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for ProfileStoreError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<ProfileValidationError> for ProfileStoreError {
    fn from(error: ProfileValidationError) -> Self {
        Self::Validation(error)
    }
}

pub struct ProfileStore {
    path: PathBuf,
}

impl ProfileStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<LoadedProfiles, ProfileStoreError> {
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(LoadedProfiles {
                    profiles: Vec::new(),
                    migrated_from_v0: false,
                });
            }
            Err(error) => return Err(error.into()),
        };
        let value: serde_json::Value = serde_json::from_slice(&bytes)?;
        let root = value.as_object().ok_or(ProfileStoreError::InvalidRoot)?;

        let (profiles, migrated_from_v0) = match root.get("version") {
            Some(serde_json::Value::Number(number)) => {
                let version = number.as_u64().ok_or(ProfileStoreError::InvalidRoot)?;
                match version {
                    2 => {
                        let current: StoredProfilesV2 = serde_json::from_value(value)?;
                        (current.profiles, false)
                    }
                    1 => {
                        let legacy: StoredProfilesV1 = serde_json::from_value(value)?;
                        (legacy.into_managed_profiles(), true)
                    }
                    _ => return Err(ProfileStoreError::UnsupportedVersion(version)),
                }
            }
            None => {
                let legacy: StoredProfilesV0 = serde_json::from_value(value)?;
                (legacy.into_managed_profiles(), true)
            }
            _ => return Err(ProfileStoreError::InvalidRoot),
        };

        validate_profiles(&profiles)?;
        Ok(LoadedProfiles {
            profiles,
            migrated_from_v0,
        })
    }

    pub fn save(&self, profiles: &[ConnectionProfile]) -> Result<(), ProfileStoreError> {
        validate_profiles(profiles)?;
        let parent = self.path.parent().ok_or_else(|| {
            ProfileStoreError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Le chemin de profils doit avoir un dossier parent.",
            ))
        })?;
        fs::create_dir_all(parent)?;
        set_private_directory_permissions(parent)?;

        let document = StoredProfilesV2 {
            version: CURRENT_FORMAT_VERSION,
            profiles: profiles.to_vec(),
        };
        let mut bytes = serde_json::to_vec_pretty(&document)?;
        bytes.push(b'\n');

        let temporary = parent.join(format!(".profiles-{}.tmp", Uuid::new_v4()));
        let result = (|| -> Result<(), ProfileStoreError> {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)?;
            set_private_file_permissions(&file)?;
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

    pub fn remove(&self, profile_id: &str) -> Result<bool, ProfileStoreError> {
        let mut loaded = self.load()?;
        let before = loaded.profiles.len();
        loaded.profiles.retain(|profile| profile.id() != profile_id);
        if loaded.profiles.len() == before {
            return Ok(false);
        }
        self.save(&loaded.profiles)?;
        Ok(true)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct StoredProfilesV2 {
    version: u8,
    profiles: Vec<ConnectionProfile>,
}

#[derive(Clone, Debug, Deserialize)]
struct StoredProfilesV1 {
    #[serde(rename = "version")]
    _version: u8,
    profiles: Vec<LegacyConnectionProfile>,
}

impl StoredProfilesV1 {
    fn into_managed_profiles(self) -> Vec<ConnectionProfile> {
        self.profiles
            .into_iter()
            .filter_map(LegacyConnectionProfile::into_managed_profile)
            .collect()
    }
}

#[derive(Clone, Debug, Deserialize)]
struct StoredProfilesV0 {
    profiles: Vec<LegacyConnectionProfile>,
}

impl StoredProfilesV0 {
    fn into_managed_profiles(self) -> Vec<ConnectionProfile> {
        self.profiles
            .into_iter()
            .filter_map(LegacyConnectionProfile::into_managed_profile)
            .collect()
    }
}

/// Les profils `local` de la première itération ne sont jamais réexposés : ils
/// n'ont pas de tunnel possédé par Desktop et demandaient un jeton manuel.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum LegacyConnectionProfile {
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
        #[serde(default)]
        host: String,
        relay_port: u16,
        #[serde(default)]
        capabilities: Vec<ProfileCapability>,
    },
}

impl LegacyConnectionProfile {
    fn into_managed_profile(self) -> Option<ConnectionProfile> {
        match self {
            Self::Ssh {
                id,
                label,
                host,
                port,
                user,
                identity,
                host_fingerprint,
                capabilities,
            } => Some(ConnectionProfile::Ssh {
                id,
                label,
                host,
                port,
                user,
                identity,
                host_fingerprint,
                capabilities,
            }),
            Self::Local {
                id,
                label,
                host,
                relay_port,
                capabilities,
            } => {
                let _ = (id, label, host, relay_port, capabilities);
                None
            }
        }
    }
}

fn validate_profiles(profiles: &[ConnectionProfile]) -> Result<(), ProfileStoreError> {
    let mut ids = HashSet::new();
    for profile in profiles {
        profile.validate()?;
        if !ids.insert(profile.id()) {
            return Err(ProfileStoreError::DuplicateId(profile.id().to_owned()));
        }
    }
    Ok(())
}

fn set_private_file_permissions(file: &File) -> Result<(), std::io::Error> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn set_private_directory_permissions(path: &Path) -> Result<(), std::io::Error> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ProfileStore, ProfileStoreError};
    use crate::profile::{ConnectionProfile, ProfileCapability, SshIdentityRef};
    use std::fs;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn test_directory() -> PathBuf {
        let path = std::env::temp_dir().join(format!("bridget-desktop-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&path).expect("dossier temporaire");
        path
    }

    fn profile() -> ConnectionProfile {
        ConnectionProfile::Ssh {
            id: "production".into(),
            label: "Cartae.app".into(),
            host: "cartae.app".into(),
            port: 2222,
            user: "moi".into(),
            identity: SshIdentityRef::Agent,
            host_fingerprint: Some("SHA256:abcdefghijklmnopqrstuvwxyz123456".into()),
            capabilities: vec![ProfileCapability::Ui],
        }
    }

    #[test]
    fn ecriture_atomique_et_relecture_ne_persistent_pas_de_matiere_secrete() {
        let directory = test_directory();
        let store = ProfileStore::new(directory.join("profiles.json"));
        store.save(&[profile()]).expect("écriture");

        let content = fs::read_to_string(store.path()).expect("lecture brute");
        assert!(content.contains("\"version\": 2"));
        assert!(!content.contains("PRIVATE KEY"));
        assert!(!content.contains("BEGIN OPENSSH"));
        let loaded = store.load().expect("relecture");
        assert_eq!(loaded.profiles, vec![profile()]);
        assert!(!loaded.migrated_from_v0);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(store.path())
                    .expect("métadonnées")
                    .permissions()
                    .mode()
                    & 0o077,
                0
            );
        }
        fs::remove_dir_all(directory).expect("nettoyage exact du test");
    }

    #[test]
    fn les_profils_anciens_sont_migres_et_les_endpoints_manuels_ecartes() {
        let directory = test_directory();
        let path = directory.join("profiles.json");
        fs::write(
            &path,
            format!(
                "{{\"profiles\":[{}]}}",
                serde_json::to_string(&profile()).unwrap()
            ),
        )
        .expect("fixture v0");
        let store = ProfileStore::new(&path);
        assert!(store.load().expect("migration").migrated_from_v0);

        fs::write(
            &path,
            r#"{"version":1,"profiles":[{"kind":"local","id":"ancien","label":"Ancien tunnel","host":"127.0.0.1","relay_port":17893,"capabilities":["ui"]}]}"#,
        )
        .expect("fixture endpoint manuel v1");
        let migrated = store.load().expect("migration v1");
        assert!(migrated.migrated_from_v0);
        assert!(migrated.profiles.is_empty());

        fs::write(&path, "{\"version\": 99, \"profiles\": []}").expect("fixture inconnue");
        assert!(matches!(
            store.load(),
            Err(ProfileStoreError::UnsupportedVersion(99))
        ));
        fs::remove_dir_all(directory).expect("nettoyage exact du test");
    }

    #[test]
    fn suppression_requiert_une_confirmation_cote_interface_et_est_precise_cote_stockage() {
        let directory = test_directory();
        let store = ProfileStore::new(directory.join("profiles.json"));
        store.save(&[profile()]).expect("écriture");
        assert!(!store.remove("absent").expect("suppression absente"));
        assert!(store.remove("production").expect("suppression exacte"));
        assert!(store.load().expect("relecture").profiles.is_empty());
        fs::remove_dir_all(directory).expect("nettoyage exact du test");
    }
}
