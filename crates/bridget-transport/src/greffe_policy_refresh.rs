//! Régénération explicite des instances approuvées par la politique du greffe.
//!
//! Ce module renouvelle une instance ; il n'accorde jamais un principal ni une
//! action. La politique nomme elle-même toutes les sources de marqueurs à
//! observer. Une collecte partielle, vide, ambiguë ou périmée échoue avant la
//! réécriture du fichier privé.

use crate::fsutil::{AtomicWritePhase, write_private_file_atomic_observed};
use crate::greffe_authorization::{
    GreffePolicyFile, InstancePolicyFile, PolicyFileLoadError, is_valid_greffe_identity_component,
    load_policy_file_detailed, verify_private_regular_file,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io;
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Component, Path, PathBuf};

pub const MARKER_INVENTORY_VERSION: u16 = 1;
pub const MAX_MARKER_INVENTORY_AGE_SECONDS: i64 = 300;

/// Source que la politique exige de mesurer avant toute régénération.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkerSource {
    pub host: String,
    pub marker_directory: PathBuf,
}

/// Marqueur dont le processus et la naissance concordent sur l'hôte source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveMarker {
    pub principal: String,
    pub instance_id: String,
    pub pid: u32,
    pub birth: u64,
}

/// Cause fermée expliquant pourquoi un marqueur n'est plus vivant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StaleMarkerReason {
    ProcessNotLive,
    BirthMismatch,
}

/// Marqueur lisible mais qui ne désigne plus le processus observé.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StaleMarker {
    pub marker: String,
    pub pid: u32,
    pub instance_id: String,
    pub reason: StaleMarkerReason,
}

/// Inventaire produit sur l'hôte qui possède réellement les PID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkerInventory {
    pub version: u16,
    pub source: MarkerSource,
    pub observed_at: i64,
    pub complete: bool,
    pub live: Vec<LiveMarker>,
    pub stale: Vec<StaleMarker>,
}

/// Rapport non secret : la clé d'attestation n'en fait jamais partie.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyRegenerationReport {
    pub applied: bool,
    pub previous_generation: u64,
    pub next_generation: u64,
    pub refreshed_principals: Vec<String>,
    pub unchanged_principals: Vec<String>,
    pub dead_principals: Vec<String>,
    pub unapproved_principals: Vec<String>,
}

/// État relu au chemin final lorsqu'une erreur survient après le renommage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostRenamePolicyState {
    PlannedPolicyPresent,
    DifferentValidPolicyPresent,
    UnreadableOrInvalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyRefreshError {
    PolicyUnavailable,
    PolicyInvalid,
    PolicyFileTypeUnsupported,
    PolicyPathNotCanonical,
    PolicyPathNotUnique,
    LockPathNotUnique,
    MissingMarkerSource {
        principal: String,
    },
    InvalidMarkerSource {
        source: MarkerSource,
    },
    MissingInventory {
        source: MarkerSource,
    },
    UnexpectedInventory {
        source: MarkerSource,
    },
    DuplicateInventory {
        source: MarkerSource,
    },
    IncompleteInventory {
        source: MarkerSource,
    },
    EmptyInventory {
        source: MarkerSource,
    },
    InventoryFromFuture {
        source: MarkerSource,
    },
    InventoryTooOld {
        source: MarkerSource,
    },
    InvalidLiveMarker {
        source: MarkerSource,
    },
    PrincipalOnWrongSource {
        principal: String,
        expected: MarkerSource,
        observed: MarkerSource,
    },
    AmbiguousPrincipal {
        principal: String,
    },
    DivergentGrant {
        principal: String,
    },
    GenerationExhausted,
    WriteOutcomeIndeterminate {
        observed: PostRenamePolicyState,
        message: String,
    },
    Io(String),
}

impl fmt::Display for PolicyRefreshError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PolicyUnavailable => write!(formatter, "politique indisponible"),
            Self::PolicyInvalid => write!(formatter, "politique invalide"),
            Self::PolicyFileTypeUnsupported => {
                write!(formatter, "type de fichier de politique non pris en charge")
            }
            Self::PolicyPathNotCanonical => write!(
                formatter,
                "le chemin de politique doit être absolu, canonique et non lié"
            ),
            Self::PolicyPathNotUnique => write!(
                formatter,
                "la politique régénérable doit posséder exactement une entrée de répertoire"
            ),
            Self::LockPathNotUnique => write!(
                formatter,
                "le verrou de régénération doit posséder exactement une entrée de répertoire"
            ),
            Self::MissingMarkerSource { principal } => write!(
                formatter,
                "le principal approuvé {principal:?} ne déclare aucune source de marqueurs"
            ),
            Self::InvalidMarkerSource { source } => {
                write!(formatter, "source de marqueurs invalide : {source:?}")
            }
            Self::MissingInventory { source } => {
                write!(formatter, "inventaire absent pour la source {source:?}")
            }
            Self::UnexpectedInventory { source } => {
                write!(formatter, "inventaire inattendu pour la source {source:?}")
            }
            Self::DuplicateInventory { source } => {
                write!(formatter, "inventaire dupliqué pour la source {source:?}")
            }
            Self::IncompleteInventory { source } => {
                write!(formatter, "inventaire incomplet pour la source {source:?}")
            }
            Self::EmptyInventory { source } => {
                write!(formatter, "aucun marqueur vivant pour la source {source:?}")
            }
            Self::InventoryFromFuture { source } => {
                write!(
                    formatter,
                    "inventaire daté dans le futur pour la source {source:?}"
                )
            }
            Self::InventoryTooOld { source } => {
                write!(formatter, "inventaire périmé pour la source {source:?}")
            }
            Self::InvalidLiveMarker { source } => {
                write!(
                    formatter,
                    "marqueur vivant invalide dans la source {source:?}"
                )
            }
            Self::PrincipalOnWrongSource {
                principal,
                expected,
                observed,
            } => write!(
                formatter,
                "principal {principal:?} observé sur {observed:?}, attendu sur {expected:?}"
            ),
            Self::AmbiguousPrincipal { principal } => {
                write!(formatter, "plusieurs marqueurs vivants pour {principal:?}")
            }
            Self::DivergentGrant { principal } => write!(
                formatter,
                "les instances de {principal:?} ne portent pas le même droit"
            ),
            Self::GenerationExhausted => write!(formatter, "génération de politique épuisée"),
            Self::WriteOutcomeIndeterminate { observed, message } => write!(
                formatter,
                "issue d'écriture indéterminée après renommage ({observed:?}) : {message}"
            ),
            Self::Io(message) => write!(formatter, "stockage de politique impossible : {message}"),
        }
    }
}

impl std::error::Error for PolicyRefreshError {}

/// Calcule une prévisualisation ou applique la nouvelle politique sous verrou.
///
/// `now` est explicite pour rendre la fraîcheur des inventaires vérifiable. Le
/// binaire de production lui fournit l'heure système ; les tests fournissent
/// une valeur littérale.
pub fn refresh_policy(
    policy_path: &Path,
    inventories: &[MarkerInventory],
    now: i64,
    apply: bool,
) -> Result<PolicyRegenerationReport, PolicyRefreshError> {
    validate_policy_path(policy_path)?;
    if apply {
        refresh_policy_observed(policy_path, inventories, now, |_| Ok(()))
    } else {
        let policy = load_refreshable_policy(policy_path)?;
        let (_, report) = plan_regeneration(policy, inventories, now)?;
        Ok(report)
    }
}

fn validate_policy_path(policy_path: &Path) -> Result<(), PolicyRefreshError> {
    if !policy_path.is_absolute() {
        return Err(PolicyRefreshError::PolicyPathNotCanonical);
    }
    let metadata = std::fs::symlink_metadata(policy_path)
        .map_err(|_| PolicyRefreshError::PolicyUnavailable)?;
    let canonical =
        std::fs::canonicalize(policy_path).map_err(|_| PolicyRefreshError::PolicyUnavailable)?;
    if metadata.file_type().is_symlink() || canonical != policy_path {
        return Err(PolicyRefreshError::PolicyPathNotCanonical);
    }
    if metadata.nlink() != 1 {
        return Err(PolicyRefreshError::PolicyPathNotUnique);
    }
    Ok(())
}

fn refresh_policy_observed(
    policy_path: &Path,
    inventories: &[MarkerInventory],
    now: i64,
    mut observer: impl FnMut(AtomicWritePhase) -> io::Result<()>,
) -> Result<PolicyRegenerationReport, PolicyRefreshError> {
    let _lock = PolicyRefreshLock::acquire(policy_path)?;
    let policy = load_refreshable_policy(policy_path)?;
    let (next, mut report) = plan_regeneration(policy, inventories, now)?;
    if report.next_generation == report.previous_generation {
        return Ok(report);
    }
    let mut bytes = serde_json::to_vec_pretty(&next)
        .map_err(|error| PolicyRefreshError::Io(error.to_string()))?;
    bytes.push(b'\n');
    let mut renamed = false;
    if let Err(error) = write_private_file_atomic_observed(policy_path, &bytes, |phase| {
        if phase == AtomicWritePhase::AfterRename {
            renamed = true;
        }
        observer(phase)
    }) {
        if !renamed {
            return Err(PolicyRefreshError::Io(error.to_string()));
        }
        let observed = match load_refreshable_policy(policy_path) {
            Ok(current) if current == next => PostRenamePolicyState::PlannedPolicyPresent,
            Ok(_) => PostRenamePolicyState::DifferentValidPolicyPresent,
            Err(_) => PostRenamePolicyState::UnreadableOrInvalid,
        };
        return Err(PolicyRefreshError::WriteOutcomeIndeterminate {
            observed,
            message: error.to_string(),
        });
    }
    report.applied = true;
    Ok(report)
}

fn load_refreshable_policy(path: &Path) -> Result<GreffePolicyFile, PolicyRefreshError> {
    load_policy_file_detailed(path).map_err(|error| match error {
        PolicyFileLoadError::Unavailable => PolicyRefreshError::PolicyUnavailable,
        PolicyFileLoadError::UnsupportedType => PolicyRefreshError::PolicyFileTypeUnsupported,
        PolicyFileLoadError::Invalid => PolicyRefreshError::PolicyInvalid,
    })
}

fn plan_regeneration(
    mut policy: GreffePolicyFile,
    inventories: &[MarkerInventory],
    now: i64,
) -> Result<(GreffePolicyFile, PolicyRegenerationReport), PolicyRefreshError> {
    let mut expected_by_principal = BTreeMap::new();
    let mut expected_sources = BTreeSet::new();
    for principal in &policy.principals {
        let source = principal.marker_source.clone().ok_or_else(|| {
            PolicyRefreshError::MissingMarkerSource {
                principal: principal.principal.clone(),
            }
        })?;
        validate_source(&source)?;
        expected_sources.insert(source.clone());
        expected_by_principal.insert(principal.principal.clone(), source);
    }
    if expected_sources.is_empty() {
        return Err(PolicyRefreshError::PolicyInvalid);
    }

    let mut inventory_by_source = BTreeMap::new();
    for inventory in inventories {
        validate_inventory(inventory, now)?;
        if !expected_sources.contains(&inventory.source) {
            return Err(PolicyRefreshError::UnexpectedInventory {
                source: inventory.source.clone(),
            });
        }
        if inventory_by_source
            .insert(inventory.source.clone(), inventory)
            .is_some()
        {
            return Err(PolicyRefreshError::DuplicateInventory {
                source: inventory.source.clone(),
            });
        }
    }
    for source in expected_sources {
        if !inventory_by_source.contains_key(&source) {
            return Err(PolicyRefreshError::MissingInventory { source });
        }
    }

    let mut live_by_principal = BTreeMap::new();
    let mut unapproved = BTreeSet::new();
    for inventory in inventory_by_source.values() {
        for marker in &inventory.live {
            if !is_valid_greffe_identity_component(&marker.principal)
                || !is_valid_greffe_identity_component(&marker.instance_id)
                || marker.pid <= 1
                || marker.birth == 0
            {
                return Err(PolicyRefreshError::InvalidLiveMarker {
                    source: inventory.source.clone(),
                });
            }
            let Some(expected) = expected_by_principal.get(&marker.principal) else {
                unapproved.insert(marker.principal.clone());
                continue;
            };
            if expected != &inventory.source {
                return Err(PolicyRefreshError::PrincipalOnWrongSource {
                    principal: marker.principal.clone(),
                    expected: expected.clone(),
                    observed: inventory.source.clone(),
                });
            }
            if live_by_principal
                .insert(marker.principal.clone(), marker.instance_id.clone())
                .is_some()
            {
                return Err(PolicyRefreshError::AmbiguousPrincipal {
                    principal: marker.principal.clone(),
                });
            }
        }
    }

    let previous_generation = policy.generation;
    let mut refreshed = Vec::new();
    let mut unchanged = Vec::new();
    let mut dead = Vec::new();
    for principal in &mut policy.principals {
        let Some(instance_id) = live_by_principal.get(&principal.principal) else {
            dead.push(principal.principal.clone());
            continue;
        };
        let reference = principal
            .instances
            .first()
            .expect("la validation commune interdit une liste d'instances vide");
        if principal.instances.iter().any(|instance| {
            instance.expires_at != reference.expires_at || instance.revoked != reference.revoked
        }) {
            return Err(PolicyRefreshError::DivergentGrant {
                principal: principal.principal.clone(),
            });
        }
        if principal.instances.len() == 1 && principal.instances[0].instance_id == *instance_id {
            unchanged.push(principal.principal.clone());
            continue;
        }
        principal.instances = vec![InstancePolicyFile {
            instance_id: instance_id.clone(),
            expires_at: reference.expires_at,
            revoked: reference.revoked,
        }];
        refreshed.push(principal.principal.clone());
    }

    refreshed.sort();
    unchanged.sort();
    dead.sort();
    let changed = !refreshed.is_empty();
    let next_generation = if changed {
        previous_generation
            .checked_add(1)
            .ok_or(PolicyRefreshError::GenerationExhausted)?
    } else {
        previous_generation
    };
    policy.generation = next_generation;
    let report = PolicyRegenerationReport {
        applied: false,
        previous_generation,
        next_generation,
        refreshed_principals: refreshed,
        unchanged_principals: unchanged,
        dead_principals: dead,
        unapproved_principals: unapproved.into_iter().collect(),
    };
    Ok((policy, report))
}

fn validate_source(source: &MarkerSource) -> Result<(), PolicyRefreshError> {
    let host_is_valid = !source.host.is_empty()
        && source.host == source.host.trim()
        && source.host.len() <= 256
        && !source.host.chars().any(char::is_control);
    let path_is_canonical = source.marker_directory.is_absolute()
        && source.marker_directory.file_name().is_some()
        && source
            .marker_directory
            .components()
            .all(|component| !matches!(component, Component::CurDir | Component::ParentDir));
    if !host_is_valid || !path_is_canonical {
        return Err(PolicyRefreshError::InvalidMarkerSource {
            source: source.clone(),
        });
    }
    Ok(())
}

fn validate_inventory(inventory: &MarkerInventory, now: i64) -> Result<(), PolicyRefreshError> {
    validate_source(&inventory.source)?;
    if inventory.version != MARKER_INVENTORY_VERSION || !inventory.complete {
        return Err(PolicyRefreshError::IncompleteInventory {
            source: inventory.source.clone(),
        });
    }
    if inventory.live.is_empty() {
        return Err(PolicyRefreshError::EmptyInventory {
            source: inventory.source.clone(),
        });
    }
    if inventory.observed_at <= 0 || inventory.observed_at > now {
        return Err(PolicyRefreshError::InventoryFromFuture {
            source: inventory.source.clone(),
        });
    }
    if now - inventory.observed_at > MAX_MARKER_INVENTORY_AGE_SECONDS {
        return Err(PolicyRefreshError::InventoryTooOld {
            source: inventory.source.clone(),
        });
    }
    Ok(())
}

struct PolicyRefreshLock {
    file: File,
}

impl PolicyRefreshLock {
    fn acquire(policy_path: &Path) -> Result<Self, PolicyRefreshError> {
        let parent = policy_path
            .parent()
            .ok_or_else(|| PolicyRefreshError::Io("politique sans parent".to_string()))?;
        let name = policy_path
            .file_name()
            .ok_or_else(|| PolicyRefreshError::Io("politique sans nom".to_string()))?;
        let mut lock_name = OsString::from(".");
        lock_name.push(name);
        lock_name.push(".refresh.lock");
        let lock_path = parent.join(lock_name);
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&lock_path)
            .map_err(|error| PolicyRefreshError::Io(error.to_string()))?;
        verify_private_regular_file(&file)
            .map_err(|error| PolicyRefreshError::Io(error.to_string()))?;
        if file
            .metadata()
            .map_err(|error| PolicyRefreshError::Io(error.to_string()))?
            .nlink()
            != 1
        {
            return Err(PolicyRefreshError::LockPathNotUnique);
        }
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(PolicyRefreshError::Io(
                io::Error::last_os_error().to_string(),
            ));
        }
        Ok(Self { file })
    }
}

impl Drop for PolicyRefreshLock {
    fn drop(&mut self) {
        let _ = unlock(self.file.as_raw_fd());
    }
}

fn unlock(fd: RawFd) -> io::Result<()> {
    if unsafe { libc::flock(fd, libc::LOCK_UN) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(test)]
#[path = "greffe_policy_refresh_tests.rs"]
mod tests;
