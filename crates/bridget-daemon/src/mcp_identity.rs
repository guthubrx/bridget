//! Résolution locale de l'identité appelante MCP.

use bridget_core::router::validate_agent_id;
use bridget_transport::greffe_authorization::is_valid_greffe_identity_component;
use bridget_transport::greffe_policy_refresh::{
    LiveMarker, MARKER_INVENTORY_VERSION, MarkerInventory, MarkerSource, StaleMarker,
    StaleMarkerReason,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

const MAX_ANCESTORS: usize = 16;
const MAX_MARKER_BYTES: u64 = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityError {
    LegacyMarker,
    IdentityNotFound,
}

impl IdentityError {
    /// Code stable présenté par la projection MCP.
    pub fn code(&self) -> &'static str {
        match self {
            Self::LegacyMarker => "legacy_marker",
            Self::IdentityNotFound => "identity_not_found",
        }
    }

    /// Action proposée à l'appelant lorsque sa filiation ne peut pas être
    /// établie de manière sûre.
    pub fn remediation(&self) -> &'static str {
        match self {
            Self::LegacyMarker => {
                "relancez l'agent avec une version Bridget qui réécrit le marqueur agent-pids typé"
            }
            Self::IdentityNotFound => "lancez l'appel depuis un agent Bridget enregistré",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentPidMarker {
    pub pid: u32,
    pub birth: u64,
    pub instance_id: String,
    pub name_file: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedIdentity {
    pub name: String,
    pub instance_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkerScanError {
    InvalidObservationTime,
    HostUnavailable,
    DirectoryNotCanonical,
    DirectoryUnreadable,
    InvalidMarker { marker: String },
    UnsupportedMarkerType { marker: String },
    UnsupportedNameFileType { marker: String },
    NoLiveMarker,
}

impl fmt::Display for MarkerScanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidObservationTime => write!(formatter, "instant d'observation invalide"),
            Self::HostUnavailable => write!(formatter, "nom d'hôte local indisponible"),
            Self::DirectoryNotCanonical => write!(
                formatter,
                "le répertoire de marqueurs doit être absolu et canonique"
            ),
            Self::DirectoryUnreadable => {
                write!(formatter, "répertoire de marqueurs illisible")
            }
            Self::InvalidMarker { marker } => {
                write!(formatter, "marqueur invalide ou illisible : {marker}")
            }
            Self::UnsupportedMarkerType { marker } => {
                write!(
                    formatter,
                    "type de fichier de marqueur non pris en charge : {marker}"
                )
            }
            Self::UnsupportedNameFileType { marker } => write!(
                formatter,
                "type de fichier de nom non pris en charge pour le marqueur : {marker}"
            ),
            Self::NoLiveMarker => write!(formatter, "aucun marqueur vivant observé"),
        }
    }
}

impl std::error::Error for MarkerScanError {}

pub trait ProcessTree {
    fn birth(&self, pid: u32) -> Option<u64>;
    fn parent(&self, pid: u32) -> Option<u32>;
}

pub fn resolve_current() -> Result<String, IdentityError> {
    resolve_current_identity().map(|identity| identity.name)
}

/// Résout atomiquement le nom affiché et la portée stable d'instance.
pub fn resolve_current_identity() -> Result<ResolvedIdentity, IdentityError> {
    let name_file = std::env::var_os("BRIDGET_AGENT_ID_FILE").map(PathBuf::from);
    let expected_instance_id = std::env::var("BRIDGET_AGENT_INSTANCE_ID")
        .ok()
        .filter(|value| !value.is_empty());
    let home = std::env::var_os("HOME").ok_or(IdentityError::IdentityNotFound)?;
    resolve_identity_with(
        name_file.as_deref(),
        &PathBuf::from(home).join(".cache/bridget/agent-pids"),
        expected_instance_id.as_deref(),
        std::process::id(),
        &SystemProcessTree,
    )
}

/// Identifiant d'instance stable du wrapper qui héberge la façade MCP.
///
/// Il ne dépend jamais du nom dynamique de l'agent : ce dernier peut changer
/// entre deux appels, alors que l'instance reste la portée du contrat
/// d'idempotence 012 pendant toute la vie du wrapper.
pub fn resolve_current_instance_id() -> Result<String, IdentityError> {
    resolve_current_identity().map(|identity| identity.instance_id)
}

struct SystemProcessTree;
impl ProcessTree for SystemProcessTree {
    fn birth(&self, pid: u32) -> Option<u64> {
        crate::managed_process::process_birth(pid).ok()
    }
    fn parent(&self, pid: u32) -> Option<u32> {
        process_parent(pid).ok()
    }
}

#[cfg(target_os = "macos")]
fn process_parent(pid: u32) -> std::io::Result<u32> {
    let mut info = std::mem::MaybeUninit::<libc::proc_bsdinfo>::zeroed();
    let expected = std::mem::size_of::<libc::proc_bsdinfo>();
    let written = unsafe {
        libc::proc_pidinfo(
            pid as libc::c_int,
            libc::PROC_PIDTBSDINFO,
            0,
            info.as_mut_ptr().cast(),
            expected as libc::c_int,
        )
    };
    if written != expected as libc::c_int {
        return Err(std::io::Error::last_os_error());
    }
    Ok(unsafe { info.assume_init() }.pbi_ppid)
}

#[cfg(target_os = "linux")]
fn process_parent(pid: u32) -> std::io::Result<u32> {
    let status = fs::read_to_string(format!("/proc/{pid}/status"))?;
    status
        .lines()
        .find_map(|line| line.strip_prefix("PPid:"))
        .and_then(|value| value.trim().parse().ok())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "PPid absent"))
}

/// Résout le fichier dynamique d'abord ; sinon, le premier ancêtre dont le
/// marqueur typé correspond encore au processus vivant.
pub fn resolve_with(
    name_file: Option<&Path>,
    marker_directory: &Path,
    expected_instance_id: Option<&str>,
    pid: u32,
    processes: &impl ProcessTree,
) -> Result<String, IdentityError> {
    resolve_identity_with(
        name_file,
        marker_directory,
        expected_instance_id,
        pid,
        processes,
    )
    .map(|identity| identity.name)
}

/// Résout l'identité complète. Si le processus MCP ne reçoit pas les
/// variables applicatives du wrapper, le marqueur typé valide devient la
/// source de vérité du nom et de l'instance.
pub fn resolve_identity_with(
    name_file: Option<&Path>,
    marker_directory: &Path,
    expected_instance_id: Option<&str>,
    pid: u32,
    processes: &impl ProcessTree,
) -> Result<ResolvedIdentity, IdentityError> {
    if let (Some(instance_id), Some(path)) = (expected_instance_id, name_file)
        && let Some(name) = read_name(path)
    {
        return Ok(ResolvedIdentity {
            name,
            instance_id: instance_id.to_string(),
        });
    }
    let mut current = Some(pid);
    for _ in 0..MAX_ANCESTORS {
        let Some(candidate) = current else { break };
        let marker_path = marker_directory.join(candidate.to_string());
        if marker_path.exists() {
            let raw =
                fs::read_to_string(&marker_path).map_err(|_| IdentityError::IdentityNotFound)?;
            let marker: AgentPidMarker =
                serde_json::from_str(&raw).map_err(|_| IdentityError::LegacyMarker)?;
            if marker.pid == candidate
                && !marker.instance_id.is_empty()
                && expected_instance_id.is_none_or(|expected| expected == marker.instance_id)
                && processes.birth(candidate) == Some(marker.birth)
                && let Some(name) = read_name(&marker.name_file)
            {
                return Ok(ResolvedIdentity {
                    name,
                    instance_id: marker.instance_id,
                });
            }
        }
        current = processes
            .parent(candidate)
            .filter(|parent| *parent > 1 && *parent != candidate);
    }
    Err(IdentityError::IdentityNotFound)
}

pub fn write_marker(
    marker_directory: &Path,
    pid: u32,
    birth: u64,
    instance_id: &str,
    name_file: &Path,
) -> std::io::Result<()> {
    fs::create_dir_all(marker_directory)?;
    let marker = AgentPidMarker {
        pid,
        birth,
        instance_id: instance_id.to_string(),
        name_file: name_file.to_path_buf(),
    };
    fs::write(
        marker_directory.join(pid.to_string()),
        serde_json::to_vec(&marker).unwrap(),
    )
}

/// Produit un inventaire complet sur l'hôte qui possède réellement les PID.
pub fn scan_marker_directory(
    marker_directory: &Path,
    observed_at: i64,
) -> Result<MarkerInventory, MarkerScanError> {
    let host = local_hostname()?;
    scan_marker_directory_with(marker_directory, host, observed_at, &SystemProcessTree)
}

fn scan_marker_directory_with(
    marker_directory: &Path,
    host: String,
    observed_at: i64,
    processes: &impl ProcessTree,
) -> Result<MarkerInventory, MarkerScanError> {
    if observed_at <= 0 {
        return Err(MarkerScanError::InvalidObservationTime);
    }
    if host.is_empty() || host != host.trim() || host.chars().any(char::is_control) {
        return Err(MarkerScanError::HostUnavailable);
    }
    let canonical =
        fs::canonicalize(marker_directory).map_err(|_| MarkerScanError::DirectoryUnreadable)?;
    let metadata =
        fs::symlink_metadata(marker_directory).map_err(|_| MarkerScanError::DirectoryUnreadable)?;
    if !marker_directory.is_absolute()
        || canonical != marker_directory
        || !metadata.file_type().is_dir()
        || metadata.file_type().is_symlink()
    {
        return Err(MarkerScanError::DirectoryNotCanonical);
    }

    let mut entries = fs::read_dir(marker_directory)
        .map_err(|_| MarkerScanError::DirectoryUnreadable)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| MarkerScanError::DirectoryUnreadable)?;
    entries.sort_by_key(|entry| entry.file_name());
    let mut live = Vec::new();
    let mut stale = Vec::new();
    for entry in entries {
        let marker_name =
            entry
                .file_name()
                .into_string()
                .map_err(|_| MarkerScanError::InvalidMarker {
                    marker: "<nom non UTF-8>".to_string(),
                })?;
        let pid = marker_name
            .parse::<u32>()
            .ok()
            .filter(|pid| *pid > 1)
            .ok_or_else(|| MarkerScanError::InvalidMarker {
                marker: marker_name.clone(),
            })?;
        let marker = read_marker_file(&entry.path(), &marker_name)?;
        if marker.pid != pid
            || marker.birth == 0
            || !is_valid_greffe_identity_component(&marker.instance_id)
            || !marker.name_file.is_absolute()
        {
            return Err(MarkerScanError::InvalidMarker {
                marker: marker_name,
            });
        }
        match processes.birth(pid) {
            Some(birth) if birth == marker.birth => {
                let principal = read_name_for_scan(&marker.name_file, &marker_name)?;
                live.push(LiveMarker {
                    principal,
                    instance_id: marker.instance_id,
                    pid,
                    birth,
                });
            }
            Some(_) => stale.push(StaleMarker {
                marker: marker_name,
                pid,
                instance_id: marker.instance_id,
                reason: StaleMarkerReason::BirthMismatch,
            }),
            None => stale.push(StaleMarker {
                marker: marker_name,
                pid,
                instance_id: marker.instance_id,
                reason: StaleMarkerReason::ProcessNotLive,
            }),
        }
    }
    if live.is_empty() {
        return Err(MarkerScanError::NoLiveMarker);
    }
    Ok(MarkerInventory {
        version: MARKER_INVENTORY_VERSION,
        source: MarkerSource {
            host,
            marker_directory: canonical,
        },
        observed_at,
        complete: true,
        live,
        stale,
    })
}

fn read_marker_file(path: &Path, marker_name: &str) -> Result<AgentPidMarker, MarkerScanError> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
        .open(path)
        .map_err(|_| MarkerScanError::InvalidMarker {
            marker: marker_name.to_string(),
        })?;
    let metadata = file
        .metadata()
        .map_err(|_| MarkerScanError::InvalidMarker {
            marker: marker_name.to_string(),
        })?;
    if !metadata.is_file() {
        return Err(MarkerScanError::UnsupportedMarkerType {
            marker: marker_name.to_string(),
        });
    }
    if metadata.len() > MAX_MARKER_BYTES {
        return Err(MarkerScanError::InvalidMarker {
            marker: marker_name.to_string(),
        });
    }
    let mut raw = String::new();
    file.read_to_string(&mut raw)
        .map_err(|_| MarkerScanError::InvalidMarker {
            marker: marker_name.to_string(),
        })?;
    serde_json::from_str(&raw).map_err(|_| MarkerScanError::InvalidMarker {
        marker: marker_name.to_string(),
    })
}

fn local_hostname() -> Result<String, MarkerScanError> {
    let mut buffer = [0_u8; 256];
    if unsafe { libc::gethostname(buffer.as_mut_ptr().cast(), buffer.len()) } != 0 {
        return Err(MarkerScanError::HostUnavailable);
    }
    let length = buffer
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(buffer.len());
    let host = std::str::from_utf8(&buffer[..length])
        .map_err(|_| MarkerScanError::HostUnavailable)?
        .to_string();
    if host.is_empty() || host.chars().any(char::is_control) {
        return Err(MarkerScanError::HostUnavailable);
    }
    Ok(host)
}

fn read_name(path: &Path) -> Option<String> {
    read_name_file(path).ok()
}

fn read_name_for_scan(path: &Path, marker_name: &str) -> Result<String, MarkerScanError> {
    read_name_file(path).map_err(|error| match error {
        NameFileReadError::UnsupportedType => MarkerScanError::UnsupportedNameFileType {
            marker: marker_name.to_string(),
        },
        NameFileReadError::Unreadable => MarkerScanError::InvalidMarker {
            marker: marker_name.to_string(),
        },
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NameFileReadError {
    UnsupportedType,
    Unreadable,
}

fn read_name_file(path: &Path) -> Result<String, NameFileReadError> {
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
        .open(path)
        .map_err(|_| NameFileReadError::Unreadable)?;
    let metadata = file.metadata().map_err(|_| NameFileReadError::Unreadable)?;
    if !metadata.is_file() {
        return Err(NameFileReadError::UnsupportedType);
    }
    if metadata.len() > MAX_MARKER_BYTES {
        return Err(NameFileReadError::Unreadable);
    }
    let mut name = String::new();
    file.read_to_string(&mut name)
        .map_err(|_| NameFileReadError::Unreadable)?;
    let name = name.trim();
    validate_agent_id(name).map_err(|_| NameFileReadError::Unreadable)?;
    Ok(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::os::unix::fs::symlink;

    #[derive(Default)]
    struct Fixture(BTreeMap<u32, (u64, u32)>);
    impl ProcessTree for Fixture {
        fn birth(&self, pid: u32) -> Option<u64> {
            self.0.get(&pid).map(|value| value.0)
        }
        fn parent(&self, pid: u32) -> Option<u32> {
            self.0.get(&pid).map(|value| value.1)
        }
    }
    fn root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "bridget-mcp-identity-{label}-{}",
            uuid::Uuid::new_v4()
        ))
    }
    fn marker(root: &Path, pid: u32, birth: u64, instance: &str, name: &str) -> PathBuf {
        let names = root.join("names");
        fs::create_dir_all(&names).unwrap();
        let path = names.join(format!("{pid}.txt"));
        fs::write(&path, name).unwrap();
        write_marker(&root.join("agent-pids"), pid, birth, instance, &path).unwrap();
        path
    }

    #[test]
    fn suit_rename_et_les_ancetres_valides() {
        let root = root("rename");
        let name = marker(&root, 12, 120, "instance-1", "avant");
        let tree = Fixture(BTreeMap::from([
            (42, (420, 30)),
            (30, (300, 12)),
            (12, (120, 1)),
        ]));
        assert_eq!(
            resolve_with(
                Some(&name),
                &root.join("agent-pids"),
                Some("instance-1"),
                42,
                &tree
            ),
            Ok("avant".into())
        );
        fs::write(&name, "apres").unwrap();
        assert_eq!(
            resolve_with(
                None,
                &root.join("agent-pids"),
                Some("instance-1"),
                42,
                &tree
            ),
            Ok("apres".into())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn premier_agent_du_meme_binaire_gagne_dans_la_filiation() {
        let root = root("chain");
        let markers = root.join("agent-pids");
        fs::create_dir_all(&markers).unwrap();
        let _ = marker(&root, 10, 100, "instance-a", "agent-a");
        let _ = marker(&root, 20, 200, "instance-b", "agent-b");
        let chain = Fixture(BTreeMap::from([
            (50, (500, 40)),
            (40, (400, 30)),
            (30, (300, 20)),
            (20, (200, 10)),
            (10, (100, 1)),
        ]));
        assert_eq!(
            resolve_with(None, &markers, Some("instance-b"), 50, &chain),
            Ok("agent-b".into())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn traverse_une_chaine_npx_de_trois_processus() {
        let root = root("npx-chain");
        let markers = root.join("agent-pids");
        let _ = marker(&root, 20, 200, "instance-1", "agent");
        let chain = Fixture(BTreeMap::from([
            (70, (700, 60)),
            (60, (600, 50)),
            (50, (500, 40)),
            (40, (400, 20)),
            (20, (200, 1)),
        ]));
        assert_eq!(
            resolve_with(None, &markers, Some("instance-1"), 70, &chain),
            Ok("agent".into())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resout_la_filiation_sans_variables_applicatives() {
        let root = root("sans-env");
        let markers = root.join("agent-pids");
        let _ = marker(&root, 20, 200, "instance-marquee", "agent");
        let chain = Fixture(BTreeMap::from([(42, (420, 20)), (20, (200, 1))]));

        assert_eq!(
            resolve_identity_with(None, &markers, None, 42, &chain),
            Ok(ResolvedIdentity {
                name: "agent".into(),
                instance_id: "instance-marquee".into(),
            })
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuse_pid_recycle_hors_agent_legacy_et_instance_divergente() {
        let root = root("negative");
        let markers = root.join("agent-pids");
        fs::create_dir_all(&markers).unwrap();
        let _ = marker(&root, 20, 200, "instance-1", "agent");
        let recycled = Fixture(BTreeMap::from([(20, (201, 1))]));
        assert_eq!(
            resolve_with(None, &markers, Some("instance-1"), 20, &recycled),
            Err(IdentityError::IdentityNotFound)
        );
        assert_eq!(
            resolve_with(None, &markers, Some("instance-1"), 99, &Fixture::default()),
            Err(IdentityError::IdentityNotFound)
        );
        let matching_birth = Fixture(BTreeMap::from([(20, (200, 1))]));
        assert_eq!(
            resolve_with(
                None,
                &markers,
                Some("instance-divergente"),
                20,
                &matching_birth
            ),
            Err(IdentityError::IdentityNotFound)
        );
        fs::write(markers.join("77"), "ancien-nom").unwrap();
        let legacy = Fixture(BTreeMap::from([(77, (770, 1))]));
        assert_eq!(
            resolve_with(None, &markers, Some("instance-1"), 77, &legacy),
            Err(IdentityError::LegacyMarker)
        );
        assert_eq!(IdentityError::LegacyMarker.code(), "legacy_marker");
        assert!(
            IdentityError::LegacyMarker
                .remediation()
                .contains("relancez")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ignore_un_nom_invalide_avant_de_consulter_les_ancetres() {
        let root = root("invalid-name");
        let _name = marker(&root, 12, 120, "instance-1", "agent-valide");
        let dynamic_name = root.join("dynamic-name");
        fs::write(&dynamic_name, "agent invalide").unwrap();
        let tree = Fixture(BTreeMap::from([(42, (420, 12)), (12, (120, 1))]));
        assert_eq!(
            resolve_with(
                Some(&dynamic_name),
                &root.join("agent-pids"),
                Some("instance-1"),
                42,
                &tree
            ),
            Ok("agent-valide".into())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn inventaire_mesure_sur_l_hote_distingue_vivant_et_pid_recycle() {
        let root = root("scan-live-stale");
        let markers = root.join("agent-pids");
        let _ = marker(&root, 20, 200, "instance-nouvelle", "agent-vivant");
        let _ = marker(&root, 30, 300, "instance-ancienne", "agent-recycle");
        let processes = Fixture(BTreeMap::from([(20, (200, 1)), (30, (301, 1))]));

        let inventory = scan_marker_directory_with(
            &markers,
            "hote-mesure".to_string(),
            1_788_200_000,
            &processes,
        )
        .unwrap();

        assert_eq!(inventory.source.host, "hote-mesure");
        assert_eq!(inventory.source.marker_directory, markers);
        assert_eq!(inventory.live.len(), 1);
        assert_eq!(inventory.live[0].principal, "agent-vivant");
        assert_eq!(inventory.live[0].instance_id, "instance-nouvelle");
        assert_eq!(inventory.stale.len(), 1);
        assert_eq!(inventory.stale[0].marker, "30");
        assert_eq!(inventory.stale[0].reason, StaleMarkerReason::BirthMismatch);
        assert!(inventory.complete);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn zero_marqueur_vivant_est_une_erreur_et_non_un_inventaire_vide() {
        let root = root("scan-zero-live");
        let markers = root.join("agent-pids");
        let _ = marker(&root, 20, 200, "instance-ancienne", "agent-arrete");

        assert_eq!(
            scan_marker_directory_with(
                &markers,
                "hote-mesure".to_string(),
                1_788_200_000,
                &Fixture::default(),
            )
            .unwrap_err(),
            MarkerScanError::NoLiveMarker
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn entree_malformee_ou_repertoire_lie_interdit_un_succes_partiel() {
        let root = root("scan-invalid");
        let markers = root.join("agent-pids");
        fs::create_dir_all(&markers).unwrap();
        fs::write(markers.join("20"), "pas-du-json").unwrap();
        assert_eq!(
            scan_marker_directory_with(
                &markers,
                "hote-mesure".to_string(),
                1_788_200_000,
                &Fixture(BTreeMap::from([(20, (200, 1))])),
            )
            .unwrap_err(),
            MarkerScanError::InvalidMarker {
                marker: "20".to_string()
            }
        );

        fs::remove_file(markers.join("20")).unwrap();
        let _ = marker(&root, 20, 200, "instance-vivante", "agent-vivant");
        let linked = root.join("linked-agent-pids");
        symlink(&markers, &linked).unwrap();
        assert_eq!(
            scan_marker_directory_with(
                &linked,
                "hote-mesure".to_string(),
                1_788_200_000,
                &Fixture(BTreeMap::from([(20, (200, 1))])),
            )
            .unwrap_err(),
            MarkerScanError::DirectoryNotCanonical
        );

        fs::remove_dir_all(root).unwrap();
    }
}
