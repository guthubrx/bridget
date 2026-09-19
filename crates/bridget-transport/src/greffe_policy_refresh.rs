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
mod tests {
    use super::*;
    use crate::greffe_authorization::{
        GreffeAuthorizationGate, GreffeDepositAuthorization, GreffeMutationAction,
    };
    use serde_json::json;
    use std::fs;
    use std::os::unix::fs::{PermissionsExt, symlink};
    use std::sync::atomic::{AtomicU64, Ordering};

    const NOW: i64 = 1_788_200_000;
    const KEY: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

    struct Fixture {
        root: PathBuf,
        policy: PathBuf,
        audit: PathBuf,
        source: MarkerSource,
    }

    impl Fixture {
        fn new(label: &str) -> Self {
            let root = std::env::temp_dir().join(format!(
                "bridget-greffe-refresh-{label}-{}-{}",
                std::process::id(),
                NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
            let root = fs::canonicalize(root).unwrap();
            let source = MarkerSource {
                host: "poste-beta-test".to_string(),
                marker_directory: root.join("agent-pids"),
            };
            let fixture = Self {
                policy: root.join("policy.json"),
                audit: root.join("audit.jsonl"),
                root,
                source,
            };
            fixture.write_policy(7, true);
            fixture
        }

        fn write_policy(&self, generation: u64, annotated: bool) {
            let source = annotated.then(|| self.source.clone());
            let content = json!({
                "version": 1,
                "generation": generation,
                "attestation_key": KEY,
                "principals": [
                    {
                        "principal": "agent-vivant",
                        "marker_source": source,
                        "actions": ["delegate", "registre_add"],
                        "instances": [
                            {
                                "instance_id": "instance-ancienne-a",
                                "expires_at": NOW + 600,
                                "revoked": false
                            },
                            {
                                "instance_id": "instance-ancienne-b",
                                "expires_at": NOW + 600,
                                "revoked": false
                            }
                        ]
                    },
                    {
                        "principal": "agent-arrete",
                        "marker_source": source,
                        "actions": ["objective_close"],
                        "instances": [{
                            "instance_id": "instance-conservee",
                            "expires_at": NOW + 900,
                            "revoked": true
                        }]
                    }
                ]
            });
            fs::write(&self.policy, serde_json::to_vec_pretty(&content).unwrap()).unwrap();
            fs::set_permissions(&self.policy, fs::Permissions::from_mode(0o600)).unwrap();
        }

        fn inventory(&self, instance_id: &str) -> MarkerInventory {
            MarkerInventory {
                version: MARKER_INVENTORY_VERSION,
                source: self.source.clone(),
                observed_at: NOW - 1,
                complete: true,
                live: vec![
                    LiveMarker {
                        principal: "agent-vivant".to_string(),
                        instance_id: instance_id.to_string(),
                        pid: 42,
                        birth: 420,
                    },
                    LiveMarker {
                        principal: "agent-inconnu".to_string(),
                        instance_id: "instance-inconnue".to_string(),
                        pid: 43,
                        birth: 430,
                    },
                ],
                stale: Vec::new(),
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn regenere_seulement_l_instance_vivante_et_la_garde_accepte_son_effet() {
        let fixture = Fixture::new("nominal");
        let original = fs::read(&fixture.policy).unwrap();
        let inventory = fixture.inventory("instance-nouvelle");

        let preview = refresh_policy(
            &fixture.policy,
            std::slice::from_ref(&inventory),
            NOW,
            false,
        )
        .unwrap();
        assert!(!preview.applied);
        assert_eq!(preview.previous_generation, 7);
        assert_eq!(preview.next_generation, 8);
        assert_eq!(preview.refreshed_principals, ["agent-vivant"]);
        assert_eq!(preview.dead_principals, ["agent-arrete"]);
        assert_eq!(preview.unapproved_principals, ["agent-inconnu"]);
        assert_eq!(fs::read(&fixture.policy).unwrap(), original);

        let applied = refresh_policy(&fixture.policy, &[inventory], NOW, true).unwrap();
        assert!(applied.applied);
        assert_eq!(applied.next_generation, 8);
        assert_eq!(
            fs::metadata(&fixture.policy).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let policy: serde_json::Value =
            serde_json::from_slice(&fs::read(&fixture.policy).unwrap()).unwrap();
        assert_eq!(policy["generation"], 8);
        assert_eq!(
            policy["principals"][0]["instances"],
            json!([{
                "instance_id": "instance-nouvelle",
                "expires_at": NOW + 600,
                "revoked": false
            }])
        );
        assert_eq!(
            policy["principals"][1]["instances"],
            json!([{
                "instance_id": "instance-conservee",
                "expires_at": NOW + 900,
                "revoked": true
            }])
        );
        assert!(
            !fs::read_to_string(&fixture.policy)
                .unwrap()
                .contains("agent-inconnu")
        );

        let gate = GreffeAuthorizationGate::new(&fixture.policy, &fixture.audit);
        let durable = fixture.root.join("durable-effect");
        fs::write(&durable, b"unchanged").unwrap();
        gate.authorize_deposit_then(
            GreffeDepositAuthorization {
                canonical_name: Some("agent-vivant"),
                canonical_instance_id: Some("instance-nouvelle"),
                declared_from: Some("agent-vivant"),
                action: GreffeMutationAction::Delegate,
                issuer_scope: "038_scope_0123456789abcdef0123456789abcdef",
                request_id: "request-refresh-positive",
                request_issued_at: NOW - 1,
                canonical_request: br#"{"operation":"delegate"}"#,
                observed_at: NOW,
            },
            |_| fs::write(&durable, b"mutated").unwrap(),
        )
        .unwrap();
        assert_eq!(fs::read(&durable).unwrap(), b"mutated");
    }

    #[test]
    fn politique_historique_reste_lisible_par_la_garde_mais_pas_regenerable() {
        let fixture = Fixture::new("historical");
        fixture.write_policy(7, false);
        let gate = GreffeAuthorizationGate::new(&fixture.policy, &fixture.audit);
        let accepted = gate.authorize_deposit(GreffeDepositAuthorization {
            canonical_name: Some("agent-vivant"),
            canonical_instance_id: Some("instance-ancienne-a"),
            declared_from: Some("agent-vivant"),
            action: GreffeMutationAction::Delegate,
            issuer_scope: "038_scope_0123456789abcdef0123456789abcdef",
            request_id: "request-historical",
            request_issued_at: NOW - 1,
            canonical_request: br#"{"operation":"delegate"}"#,
            observed_at: NOW,
        });
        assert!(accepted.is_ok());

        assert_eq!(
            refresh_policy(&fixture.policy, &[], NOW, true).unwrap_err(),
            PolicyRefreshError::MissingMarkerSource {
                principal: "agent-vivant".to_string()
            }
        );
    }

    #[test]
    fn chemin_lie_ne_peut_pas_creer_un_second_verrou_sur_la_meme_politique() {
        let fixture = Fixture::new("linked-policy-path");
        let linked = fixture.root.join("linked-policy.json");
        symlink(&fixture.policy, &linked).unwrap();

        assert_eq!(
            refresh_policy(
                &linked,
                &[fixture.inventory("instance-nouvelle")],
                NOW,
                true
            )
            .unwrap_err(),
            PolicyRefreshError::PolicyPathNotCanonical
        );
    }

    #[test]
    fn politique_a_plusieurs_entrees_est_refusee_avant_le_plan() {
        let fixture = Fixture::new("hard-linked-policy");
        let alias = fixture.root.join("policy-alias.json");
        fs::hard_link(&fixture.policy, &alias).unwrap();
        let original = fs::read(&fixture.policy).unwrap();

        assert_eq!(
            refresh_policy(&fixture.policy, &[], NOW, true).unwrap_err(),
            PolicyRefreshError::PolicyPathNotUnique
        );
        assert_eq!(fs::read(&fixture.policy).unwrap(), original);
        assert_eq!(fs::read(alias).unwrap(), original);
    }

    #[test]
    fn verrou_a_plusieurs_entrees_est_refuse_avant_le_plan() {
        let fixture = Fixture::new("hard-linked-lock");
        let lock = fixture.root.join(".policy.json.refresh.lock");
        let alias = fixture.root.join("lock-alias");
        fs::write(&lock, b"").unwrap();
        fs::set_permissions(&lock, fs::Permissions::from_mode(0o600)).unwrap();
        fs::hard_link(&lock, &alias).unwrap();
        let original = fs::read(&fixture.policy).unwrap();

        assert_eq!(
            refresh_policy(&fixture.policy, &[], NOW, true).unwrap_err(),
            PolicyRefreshError::LockPathNotUnique
        );
        assert_eq!(fs::read(&fixture.policy).unwrap(), original);
    }

    #[test]
    fn exemple_038_adapte_est_regenerable_sans_ouvrir_de_nouveau_droit() {
        let fixture = Fixture::new("documented-refresh-policy");
        let example_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/greffe-authorization-refresh.example.json");
        let mut example: serde_json::Value =
            serde_json::from_slice(&fs::read(example_path).unwrap()).unwrap();
        example["generation"] = json!(7);
        example["attestation_key"] = json!(KEY);
        example["principals"][0]["principal"] = json!("agent-vivant");
        example["principals"][0]["marker_source"] = json!(fixture.source);
        example["principals"][0]["instances"][0]["instance_id"] = json!("instance-ancienne");
        example["principals"][0]["instances"][0]["expires_at"] = json!(NOW + 600);
        fs::write(
            &fixture.policy,
            serde_json::to_vec_pretty(&example).unwrap(),
        )
        .unwrap();
        fs::set_permissions(&fixture.policy, fs::Permissions::from_mode(0o600)).unwrap();

        let report = refresh_policy(
            &fixture.policy,
            &[fixture.inventory("instance-nouvelle")],
            NOW,
            true,
        )
        .unwrap();

        assert_eq!(report.refreshed_principals, ["agent-vivant"]);
        assert_eq!(report.unapproved_principals, ["agent-inconnu"]);
        let policy = fs::read_to_string(&fixture.policy).unwrap();
        assert!(policy.contains("instance-nouvelle"));
        assert!(!policy.contains("instance-inconnue"));
    }

    #[test]
    fn inventaire_incomplet_vide_ou_absent_preserve_exactement_la_politique() {
        let fixture = Fixture::new("closed-inputs");
        let original = fs::read(&fixture.policy).unwrap();
        let mut incomplete = fixture.inventory("instance-nouvelle");
        incomplete.complete = false;
        assert_eq!(
            refresh_policy(&fixture.policy, &[incomplete], NOW, true).unwrap_err(),
            PolicyRefreshError::IncompleteInventory {
                source: fixture.source.clone()
            }
        );
        assert_eq!(fs::read(&fixture.policy).unwrap(), original);

        let mut empty = fixture.inventory("instance-nouvelle");
        empty.live.clear();
        assert_eq!(
            refresh_policy(&fixture.policy, &[empty], NOW, true).unwrap_err(),
            PolicyRefreshError::EmptyInventory {
                source: fixture.source.clone()
            }
        );
        assert_eq!(fs::read(&fixture.policy).unwrap(), original);

        assert_eq!(
            refresh_policy(&fixture.policy, &[], NOW, true).unwrap_err(),
            PolicyRefreshError::MissingInventory {
                source: fixture.source.clone()
            }
        );
        assert_eq!(fs::read(&fixture.policy).unwrap(), original);
    }

    #[test]
    fn ensemble_des_sources_est_exact_sans_oubli_ajout_ni_doublon() {
        let fixture = Fixture::new("exact-sources");
        let source_distante = MarkerSource {
            host: "hote-distant".to_string(),
            marker_directory: fixture.root.join("remote-agent-pids"),
        };
        let mut policy: serde_json::Value =
            serde_json::from_slice(&fs::read(&fixture.policy).unwrap()).unwrap();
        policy["principals"][1]["marker_source"] = json!(source_distante);
        fs::write(&fixture.policy, serde_json::to_vec_pretty(&policy).unwrap()).unwrap();
        fs::set_permissions(&fixture.policy, fs::Permissions::from_mode(0o600)).unwrap();
        let local = fixture.inventory("instance-nouvelle");
        let remote = MarkerInventory {
            version: MARKER_INVENTORY_VERSION,
            source: source_distante.clone(),
            observed_at: NOW - 1,
            complete: true,
            live: vec![LiveMarker {
                principal: "agent-inconnu-distant".to_string(),
                instance_id: "instance-inconnue-distante".to_string(),
                pid: 44,
                birth: 440,
            }],
            stale: Vec::new(),
        };

        assert_eq!(
            refresh_policy(&fixture.policy, std::slice::from_ref(&local), NOW, false).unwrap_err(),
            PolicyRefreshError::MissingInventory {
                source: source_distante.clone()
            }
        );

        let unexpected_source = MarkerSource {
            host: "hote-inattendu".to_string(),
            marker_directory: fixture.root.join("unexpected-agent-pids"),
        };
        let mut unexpected = remote.clone();
        unexpected.source = unexpected_source.clone();
        assert_eq!(
            refresh_policy(
                &fixture.policy,
                &[local.clone(), remote.clone(), unexpected],
                NOW,
                false
            )
            .unwrap_err(),
            PolicyRefreshError::UnexpectedInventory {
                source: unexpected_source
            }
        );

        assert_eq!(
            refresh_policy(&fixture.policy, &[local.clone(), local, remote], NOW, false)
                .unwrap_err(),
            PolicyRefreshError::DuplicateInventory {
                source: fixture.source.clone()
            }
        );
    }

    #[test]
    fn principal_sur_mauvaise_source_ou_duplique_est_refuse() {
        let fixture = Fixture::new("wrong-or-duplicate-principal");
        let source_distante = MarkerSource {
            host: "hote-distant".to_string(),
            marker_directory: fixture.root.join("remote-agent-pids"),
        };
        let mut policy: serde_json::Value =
            serde_json::from_slice(&fs::read(&fixture.policy).unwrap()).unwrap();
        policy["principals"][1]["marker_source"] = json!(source_distante);
        fs::write(&fixture.policy, serde_json::to_vec_pretty(&policy).unwrap()).unwrap();
        fs::set_permissions(&fixture.policy, fs::Permissions::from_mode(0o600)).unwrap();
        let mut local = fixture.inventory("instance-nouvelle");
        local.live.remove(0);
        let remote = MarkerInventory {
            version: MARKER_INVENTORY_VERSION,
            source: source_distante.clone(),
            observed_at: NOW - 1,
            complete: true,
            live: vec![LiveMarker {
                principal: "agent-vivant".to_string(),
                instance_id: "instance-nouvelle".to_string(),
                pid: 44,
                birth: 440,
            }],
            stale: Vec::new(),
        };
        assert_eq!(
            refresh_policy(&fixture.policy, &[local, remote], NOW, false).unwrap_err(),
            PolicyRefreshError::PrincipalOnWrongSource {
                principal: "agent-vivant".to_string(),
                expected: fixture.source.clone(),
                observed: source_distante
            }
        );

        let duplicate_fixture = Fixture::new("duplicate-principal");
        let mut duplicate = duplicate_fixture.inventory("instance-nouvelle");
        duplicate.live.push(LiveMarker {
            principal: "agent-vivant".to_string(),
            instance_id: "instance-concurrente".to_string(),
            pid: 45,
            birth: 450,
        });
        assert_eq!(
            refresh_policy(&duplicate_fixture.policy, &[duplicate], NOW, false).unwrap_err(),
            PolicyRefreshError::AmbiguousPrincipal {
                principal: "agent-vivant".to_string()
            }
        );
    }

    #[test]
    fn inventaire_perime_ou_futur_est_refuse_avant_ecriture() {
        let fixture = Fixture::new("inventory-time");
        let original = fs::read(&fixture.policy).unwrap();
        let mut old = fixture.inventory("instance-nouvelle");
        old.observed_at = NOW - MAX_MARKER_INVENTORY_AGE_SECONDS - 1;
        assert_eq!(
            refresh_policy(&fixture.policy, &[old], NOW, true).unwrap_err(),
            PolicyRefreshError::InventoryTooOld {
                source: fixture.source.clone()
            }
        );
        assert_eq!(fs::read(&fixture.policy).unwrap(), original);

        let mut future = fixture.inventory("instance-nouvelle");
        future.observed_at = NOW + 1;
        assert_eq!(
            refresh_policy(&fixture.policy, &[future], NOW, true).unwrap_err(),
            PolicyRefreshError::InventoryFromFuture {
                source: fixture.source.clone()
            }
        );
        assert_eq!(fs::read(&fixture.policy).unwrap(), original);
    }

    #[test]
    fn droits_divergents_et_generation_epuisee_ne_sont_jamais_arbitres() {
        let fixture = Fixture::new("divergent-grant");
        let mut policy: serde_json::Value =
            serde_json::from_slice(&fs::read(&fixture.policy).unwrap()).unwrap();
        policy["principals"][0]["instances"][1]["expires_at"] = json!(NOW + 601);
        fs::write(&fixture.policy, serde_json::to_vec_pretty(&policy).unwrap()).unwrap();
        fs::set_permissions(&fixture.policy, fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(
            refresh_policy(
                &fixture.policy,
                &[fixture.inventory("instance-nouvelle")],
                NOW,
                false
            )
            .unwrap_err(),
            PolicyRefreshError::DivergentGrant {
                principal: "agent-vivant".to_string()
            }
        );

        fixture.write_policy(u64::MAX, true);
        assert_eq!(
            refresh_policy(
                &fixture.policy,
                &[fixture.inventory("instance-nouvelle")],
                NOW,
                true
            )
            .unwrap_err(),
            PolicyRefreshError::GenerationExhausted
        );
    }

    #[test]
    fn generation_croit_et_remplacement_reste_atomique() {
        let fixture = Fixture::new("atomic-generation");
        let original = fs::read(&fixture.policy).unwrap();
        let inventory = fixture.inventory("instance-nouvelle");
        let mut observed = Vec::new();
        let report = refresh_policy_observed(&fixture.policy, &[inventory], NOW, |phase| {
            observed.push(phase);
            if phase == AtomicWritePhase::BeforeRename {
                assert_eq!(
                    fs::read(&fixture.policy).unwrap(),
                    original,
                    "avant le renommage, seul le fichier original est visible"
                );
            }
            Ok(())
        })
        .unwrap();
        assert_eq!(report.previous_generation, 7);
        assert_eq!(report.next_generation, 8);
        assert_eq!(
            observed,
            [
                AtomicWritePhase::BeforeRename,
                AtomicWritePhase::AfterRename
            ],
            "le chemin de production doit traverser les deux frontières atomiques"
        );
        let rewritten: serde_json::Value =
            serde_json::from_slice(&fs::read(&fixture.policy).unwrap()).unwrap();
        assert_eq!(rewritten["generation"], 8);
    }

    #[test]
    fn erreur_avant_renommage_garde_l_original_et_apres_rend_l_issue_indeterminee() {
        let before = Fixture::new("failure-before-rename");
        let original = fs::read(&before.policy).unwrap();
        let before_error = refresh_policy_observed(
            &before.policy,
            &[before.inventory("instance-nouvelle")],
            NOW,
            |phase| {
                if phase == AtomicWritePhase::BeforeRename {
                    return Err(io::Error::other("échec injecté avant renommage"));
                }
                Ok(())
            },
        )
        .unwrap_err();
        assert_eq!(
            before_error,
            PolicyRefreshError::Io("échec injecté avant renommage".to_string())
        );
        assert_eq!(fs::read(&before.policy).unwrap(), original);

        let after = Fixture::new("failure-after-rename");
        let after_error = refresh_policy_observed(
            &after.policy,
            &[after.inventory("instance-nouvelle")],
            NOW,
            |phase| {
                if phase == AtomicWritePhase::AfterRename {
                    return Err(io::Error::other("échec injecté après renommage"));
                }
                Ok(())
            },
        )
        .unwrap_err();
        assert_eq!(
            after_error,
            PolicyRefreshError::WriteOutcomeIndeterminate {
                observed: PostRenamePolicyState::PlannedPolicyPresent,
                message: "échec injecté après renommage".to_string(),
            }
        );
        let rewritten: serde_json::Value =
            serde_json::from_slice(&fs::read(&after.policy).unwrap()).unwrap();
        assert_eq!(rewritten["generation"], 8);
    }

    #[test]
    fn instance_deja_alignee_ne_reecrit_ni_ne_rafraichit_la_generation() {
        let fixture = Fixture::new("already-current");
        refresh_policy(
            &fixture.policy,
            &[fixture.inventory("instance-nouvelle")],
            NOW,
            true,
        )
        .unwrap();
        let current = fs::read(&fixture.policy).unwrap();

        let report = refresh_policy_observed(
            &fixture.policy,
            &[fixture.inventory("instance-nouvelle")],
            NOW,
            |_| panic!("aucune écriture ne doit avoir lieu sans changement"),
        )
        .unwrap();
        assert!(!report.applied);
        assert_eq!(report.previous_generation, 8);
        assert_eq!(report.next_generation, 8);
        assert_eq!(report.unchanged_principals, ["agent-vivant"]);
        assert_eq!(fs::read(&fixture.policy).unwrap(), current);
    }
}
