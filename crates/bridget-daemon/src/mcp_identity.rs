//! Résolution locale de l'identité appelante MCP.

use bridget_core::router::validate_agent_id;
use bridget_transport::greffe_authorization::is_valid_greffe_identity_component;
use bridget_transport::greffe_policy_refresh::{
    LiveMarker, MARKER_INVENTORY_VERSION, MarkerInventory, MarkerSource, StaleMarker,
    StaleMarkerReason,
};
use bridget_transport::protocol::IdentityCredential;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrivateIdentity {
    agent_id: String,
    instance_id: String,
    credential: IdentityCredential,
}

fn credential_path(root: &Path, instance_id: &str) -> PathBuf {
    root.join("agent-names").join(format!(
        "proof-{:x}.json",
        Sha256::digest(instance_id.as_bytes())
    ))
}

/// Écriture côté wrapper, donc également sur l'hôte distant d'un tunnel SSH.
pub(crate) fn save_credential(
    root: &Path,
    agent_id: &str,
    instance_id: &str,
    credential: IdentityCredential,
) -> Result<(), String> {
    let path = credential_path(root, instance_id);
    crate::environment::ensure_private_directory(root)?;
    crate::environment::validate_private_directory_if_present(&root.join("agent-names"))?;
    crate::environment::validate_state_file(&path, false)?;
    let identity = PrivateIdentity {
        agent_id: agent_id.into(),
        instance_id: instance_id.into(),
        credential,
    };
    let bytes = serde_json::to_vec(&identity).map_err(|_| "identité privée non sérialisable")?;
    bridget_transport::fsutil::write_private_file_atomic(&path, &bytes)
        .map_err(|_| "impossible de conserver la preuve privée de rattachement".into())
}

pub(crate) fn auxiliary_registration(
    agent_id: &str,
    instance_id: &str,
    socket: &Path,
) -> Result<bridget_transport::WrapperToDaemon, String> {
    let root = socket.parent().ok_or("socket sans répertoire privé")?;
    let credential = load_credential(root, agent_id, instance_id)?;
    Ok(bridget_transport::WrapperToDaemon::RegisterAuxiliary {
        agent_id: agent_id.into(),
        instance_id: instance_id.into(),
        credential,
    })
}

fn load_credential(
    root: &Path,
    agent_id: &str,
    instance_id: &str,
) -> Result<IdentityCredential, String> {
    let invalid = || {
        "auxiliary_credential_required : relancez le wrapper et son client auxiliaire".to_string()
    };
    let path = credential_path(root, instance_id);
    crate::environment::validate_private_directory_if_present(root).map_err(|_| invalid())?;
    crate::environment::validate_private_directory_if_present(&root.join("agent-names"))
        .map_err(|_| invalid())?;
    crate::environment::validate_state_file(&path, false).map_err(|_| invalid())?;
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
        .open(path)
        .map_err(|_| invalid())?;
    let metadata = file.metadata().map_err(|_| invalid())?;
    if !metadata.is_file() || metadata.len() > MAX_MARKER_BYTES {
        return Err(invalid());
    }
    let mut bytes = Vec::new();
    file.take(MAX_MARKER_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| invalid())?;
    if bytes.len() as u64 > MAX_MARKER_BYTES {
        return Err(invalid());
    }
    let identity: PrivateIdentity = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    if identity.agent_id != agent_id || identity.instance_id != instance_id {
        return Err(invalid());
    }
    Ok(identity.credential)
}

/// Fixture de protocole uniquement : aucun vrai daemon n'est simulé ici.
/// Les tests d'autorisation obtiennent leur preuve par Registered.
#[cfg(test)]
pub(crate) fn mock_private_identity(socket: &Path, agent_id: &str, instance_id: &str) {
    if !socket.exists() {
        return;
    }
    let root = socket.parent().unwrap();
    assert_ne!(root, std::env::temp_dir());
    save_credential(
        root,
        agent_id,
        instance_id,
        IdentityCredential::new(format!("mock-{}", uuid::Uuid::new_v4())),
    )
    .unwrap();
}

#[cfg(test)]
pub(crate) fn mock_socket(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "bm-{label}-{}",
        &uuid::Uuid::new_v4().simple().to_string()[..10]
    ));
    crate::environment::ensure_private_directory(&root).unwrap();
    root.join("b.sock")
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
    let namespace = crate::environment::Namespace::from_environment()
        .map_err(|_| IdentityError::IdentityNotFound)?;
    resolve_identity_scoped(
        name_file.as_deref(),
        &namespace.root.join("agent-pids"),
        expected_instance_id.as_deref(),
        std::process::id(),
        &SystemProcessTree,
        Some(&namespace.root),
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
pub(crate) fn process_parent(pid: u32) -> std::io::Result<u32> {
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
pub(crate) fn process_parent(pid: u32) -> std::io::Result<u32> {
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
    resolve_identity_scoped(
        name_file,
        marker_directory,
        expected_instance_id,
        pid,
        processes,
        None,
    )
}

fn resolve_identity_scoped(
    name_file: Option<&Path>,
    marker_directory: &Path,
    expected_instance_id: Option<&str>,
    pid: u32,
    processes: &impl ProcessTree,
    namespace: Option<&Path>,
) -> Result<ResolvedIdentity, IdentityError> {
    let permitted = |path: &Path| {
        namespace.is_none_or(|root| {
            path.starts_with(root) && crate::environment::validate_state_file(path, false).is_ok()
        })
    };
    if let (Some(instance_id), Some(path)) = (expected_instance_id, name_file)
        && permitted(path)
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
            if !permitted(&marker_path) {
                return Err(IdentityError::IdentityNotFound);
            }
            let raw =
                fs::read_to_string(&marker_path).map_err(|_| IdentityError::IdentityNotFound)?;
            let marker: AgentPidMarker =
                serde_json::from_str(&raw).map_err(|_| IdentityError::LegacyMarker)?;
            if marker.pid == candidate
                && !marker.instance_id.is_empty()
                && expected_instance_id.is_none_or(|expected| expected == marker.instance_id)
                && processes.birth(candidate) == Some(marker.birth)
                && permitted(&marker.name_file)
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
    crate::environment::ensure_private_directory(marker_directory)
        .map_err(std::io::Error::other)?;
    let marker = AgentPidMarker {
        pid,
        birth,
        instance_id: instance_id.to_string(),
        name_file: name_file.to_path_buf(),
    };
    let path = marker_directory.join(pid.to_string());
    crate::environment::validate_state_file(&path, false).map_err(std::io::Error::other)?;
    bridget_transport::fsutil::write_private_file_atomic(&path, &serde_json::to_vec(&marker)?)
}

/// Publication atomique sans remplacement, pour un adaptateur qui ne possède
/// pas encore le PID. Le lien dur évite la course « exists puis rename » entre
/// deux ponts ; un marqueur concurrent, même périmé, reste intact.
pub(crate) fn write_marker_if_absent(
    marker_directory: &Path,
    marker: &AgentPidMarker,
) -> std::io::Result<()> {
    crate::environment::ensure_private_directory(marker_directory)
        .map_err(std::io::Error::other)?;
    let temporary = marker_directory.join(format!(".t3-{}.tmp", uuid::Uuid::new_v4()));
    let path = marker_directory.join(marker.pid.to_string());
    bridget_transport::fsutil::write_private_file_atomic(&temporary, &serde_json::to_vec(marker)?)?;
    let result = fs::hard_link(&temporary, &path);
    let _ = fs::remove_file(temporary);
    result
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

pub(crate) fn read_name(path: &Path) -> Option<String> {
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
    use std::io::Write;
    use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, symlink};

    const AGENT_A: &str = "96389249-07a4-4e29-83f0-9c46bd775021";
    const AGENT_B: &str = "da78fd70-41e8-424c-a88d-e29e2c5babcd";

    #[test]
    fn spec099_preuve_privee_atomique_bornee_sans_symlink() {
        use std::os::unix::fs::PermissionsExt;
        let root = std::env::temp_dir().join(format!("bg-proof-{}", uuid::Uuid::new_v4().simple()));
        let credential = IdentityCredential::new("secret-fixture".into());
        save_credential(&root, AGENT_A, "instance-1", credential.clone()).unwrap();
        let path = credential_path(&root, "instance-1");
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            load_credential(&root, AGENT_A, "instance-1").unwrap(),
            credential
        );
        assert!(load_credential(&root, AGENT_B, "instance-1").is_err());
        assert!(!format!("{credential:?}").contains("secret-fixture"));
        fs::remove_file(&path).unwrap();
        let target = root.join("target");
        fs::write(&target, "ne pas modifier").unwrap();
        symlink(&target, &path).unwrap();
        assert!(load_credential(&root, AGENT_A, "instance-1").is_err());
        assert!(save_credential(&root, AGENT_A, "instance-1", credential.clone()).is_err());
        assert_eq!(fs::read_to_string(&target).unwrap(), "ne pas modifier");
        fs::remove_file(&path).unwrap();
        fs::write(&path, vec![b'x'; MAX_MARKER_BYTES as usize + 1]).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(load_credential(&root, AGENT_A, "instance-1").is_err());
        fs::remove_dir_all(root).unwrap();
    }

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
        // Le scan refuse à juste titre /tmp quand le chemin réel est /private/tmp.
        let root = fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .join(format!(
                "bridget-mcp-identity-{label}-{}",
                uuid::Uuid::new_v4()
            ));
        private_dir(&root);
        root
    }
    fn private_dir(path: &Path) {
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(path)
            .unwrap();
    }
    fn private_write(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
            .unwrap();
        file.write_all(bytes.as_ref()).unwrap();
    }
    fn marker(root: &Path, pid: u32, birth: u64, instance: &str, name: &str) -> PathBuf {
        let names = root.join("names");
        private_dir(&names);
        let path = names.join(format!("{pid}.txt"));
        private_write(&path, name);
        private_dir(&root.join("agent-pids"));
        // Précréer le fichier privé, sans changer le umask global des tests.
        private_write(root.join("agent-pids").join(pid.to_string()), []);
        write_marker(&root.join("agent-pids"), pid, birth, instance, &path).unwrap();
        path
    }

    #[test]
    fn relit_le_fichier_identite_et_les_ancetres_valides() {
        let root = root("rename");
        let name = marker(&root, 12, 120, "instance-1", AGENT_A);
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
            Ok(AGENT_A.into())
        );
        // Changement de fixture d'identité : prouve la relecture, PAS un
        // renommage métier (le nom affiché peut changer sans changer l'UUID).
        private_write(&name, AGENT_B);
        assert_eq!(
            resolve_with(
                None,
                &root.join("agent-pids"),
                Some("instance-1"),
                42,
                &tree
            ),
            Ok(AGENT_B.into())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn premier_agent_du_meme_binaire_gagne_dans_la_filiation() {
        let root = root("chain");
        let markers = root.join("agent-pids");
        private_dir(&markers);
        let _ = marker(&root, 10, 100, "instance-a", AGENT_A);
        let _ = marker(&root, 20, 200, "instance-b", AGENT_B);
        let chain = Fixture(BTreeMap::from([
            (50, (500, 40)),
            (40, (400, 30)),
            (30, (300, 20)),
            (20, (200, 10)),
            (10, (100, 1)),
        ]));
        assert_eq!(
            resolve_with(None, &markers, Some("instance-b"), 50, &chain),
            Ok(AGENT_B.into())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn traverse_une_chaine_npx_de_trois_processus() {
        let root = root("npx-chain");
        let markers = root.join("agent-pids");
        let _ = marker(&root, 20, 200, "instance-1", AGENT_A);
        let chain = Fixture(BTreeMap::from([
            (70, (700, 60)),
            (60, (600, 50)),
            (50, (500, 40)),
            (40, (400, 20)),
            (20, (200, 1)),
        ]));
        assert_eq!(
            resolve_with(None, &markers, Some("instance-1"), 70, &chain),
            Ok(AGENT_A.into())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resout_la_filiation_sans_variables_applicatives() {
        let root = root("sans-env");
        let markers = root.join("agent-pids");
        let _ = marker(&root, 20, 200, "instance-marquee", AGENT_A);
        let chain = Fixture(BTreeMap::from([(42, (420, 20)), (20, (200, 1))]));

        assert_eq!(
            resolve_identity_with(None, &markers, None, 42, &chain),
            Ok(ResolvedIdentity {
                name: AGENT_A.into(),
                instance_id: "instance-marquee".into(),
            })
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn refuse_pid_recycle_hors_agent_legacy_et_instance_divergente() {
        let root = root("negative");
        let markers = root.join("agent-pids");
        private_dir(&markers);
        let _ = marker(&root, 20, 200, "instance-1", AGENT_A);
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
        private_write(markers.join("77"), "ancien-nom");
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
        let _name = marker(&root, 12, 120, "instance-1", AGENT_A);
        let dynamic_name = root.join("dynamic-name");
        private_write(&dynamic_name, "agent invalide");
        let tree = Fixture(BTreeMap::from([(42, (420, 12)), (12, (120, 1))]));
        assert_eq!(
            resolve_with(
                Some(&dynamic_name),
                &root.join("agent-pids"),
                Some("instance-1"),
                42,
                &tree
            ),
            Ok(AGENT_A.into())
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn inventaire_mesure_sur_l_hote_distingue_vivant_et_pid_recycle() {
        let root = root("scan-live-stale");
        let markers = root.join("agent-pids");
        let _ = marker(&root, 20, 200, "instance-nouvelle", AGENT_A);
        let _ = marker(&root, 30, 300, "instance-ancienne", AGENT_B);
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
        assert_eq!(inventory.live[0].principal, AGENT_A);
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
        let _ = marker(&root, 20, 200, "instance-ancienne", AGENT_A);

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
        private_dir(&markers);
        private_write(markers.join("20"), "pas-du-json");
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
        let _ = marker(&root, 20, 200, "instance-vivante", AGENT_A);
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

/// Session 116 : bilan d'un passage de nettoyage de l'état d'identité.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct IdentityPurge {
    pub markers: usize,
    pub names: usize,
    pub proofs: usize,
}

/// Session 116 : retire l'état d'identité des processus et instances disparus.
///
/// - Un marqueur dont le processus est mort, ou dont le PID a été recyclé
///   (naissance différente), est retiré : la résolution l'ignorait déjà, il ne
///   porte plus aucune identité vivante.
/// - Un fichier de nom (`instance-*`, `t3-identity-*`) ou une preuve
///   (`proof-*.json`) n'est retiré que s'il a plus de `min_age`, qu'aucun
///   marqueur restant ne le désigne et, pour une preuve, que son instance n'est
///   pas connectée. Une instance qui se reconnecte reçoit une preuve neuve à
///   l'enregistrement : une preuve d'instance déconnectée ne sert plus.
///
/// Les fichiers d'un autre format sont laissés intacts, et un marqueur
/// illisible aussi : ne rien supprimer qu'on ne sache relire.
/// O(marqueurs + fichiers de noms + instances connectées).
pub(crate) fn purge_stale_identity_files(
    root: &Path,
    connected_instances: &std::collections::HashSet<String>,
    min_age: std::time::Duration,
    now: std::time::SystemTime,
) -> IdentityPurge {
    let mut report = IdentityPurge::default();
    let mut kept_names = std::collections::HashSet::new();
    let mut kept_proofs: std::collections::HashSet<PathBuf> = connected_instances
        .iter()
        .map(|instance| credential_path(root, instance))
        .collect();
    if let Ok(entries) = fs::read_dir(root.join("agent-pids")) {
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(pid) = entry
                .file_name()
                .to_str()
                .and_then(|name| name.parse::<u32>().ok())
            else {
                continue;
            };
            let Ok(marker) = read_marker_file(&path, &pid.to_string()) else {
                continue;
            };
            let alive = marker.pid == pid
                && crate::managed_process::process_birth(pid).ok() == Some(marker.birth);
            if alive {
                kept_names.insert(marker.name_file.clone());
                kept_proofs.insert(credential_path(root, &marker.instance_id));
            } else if fs::remove_file(&path).is_ok() {
                report.markers += 1;
            }
        }
    }
    let old_enough = |path: &Path| {
        fs::symlink_metadata(path)
            .ok()
            .filter(|metadata| metadata.is_file())
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|modified| now.duration_since(modified).ok())
            .is_some_and(|age| age >= min_age)
    };
    if let Ok(entries) = fs::read_dir(root.join("agent-names")) {
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            let proof = name.starts_with("proof-") && name.ends_with(".json");
            let identity_name = name.starts_with("instance-") || name.starts_with("t3-identity-");
            if !(proof || identity_name) || !old_enough(&path) {
                continue;
            }
            let referenced = if proof {
                kept_proofs.contains(&path)
            } else {
                kept_names.contains(&path)
            };
            if !referenced && fs::remove_file(&path).is_ok() {
                if proof {
                    report.proofs += 1;
                } else {
                    report.names += 1;
                }
            }
        }
    }
    report
}

#[cfg(test)]
mod spec116_purge_identite {
    use super::*;
    use std::time::{Duration, SystemTime};

    fn pid_disparu() -> u32 {
        let mut child = std::process::Command::new("/usr/bin/true").spawn().unwrap();
        let pid = child.id();
        child.wait().unwrap();
        pid
    }

    /// Un processus disparu perd son marqueur, puis son nom et sa preuve une
    /// fois assez anciens. Rien de vivant ni de connecté n'est touché, et un
    /// fichier d'un format inconnu non plus.
    #[test]
    fn spec116_seul_l_etat_des_disparus_est_retire() {
        let root = std::env::temp_dir().join(format!("bi116-{}", uuid::Uuid::new_v4()));
        crate::environment::ensure_private_directory(&root).unwrap();
        crate::environment::ensure_private_directory(&root.join("agent-names")).unwrap();
        let names = root.join("agent-names");
        let vivant = std::process::id();
        let naissance = crate::managed_process::process_birth(vivant).unwrap();
        let disparu = pid_disparu();
        let nom_vivant = names.join("instance-vivante");
        let nom_disparu = names.join("instance-disparue");
        for path in [&nom_vivant, &nom_disparu] {
            bridget_transport::fsutil::write_private_file_atomic(path, b"agent").unwrap();
        }
        let pids = root.join("agent-pids");
        write_marker(&pids, vivant, naissance, "i-vivante", &nom_vivant).unwrap();
        write_marker(&pids, disparu, 1, "i-disparue", &nom_disparu).unwrap();
        for instance in ["i-vivante", "i-disparue", "i-connectee"] {
            bridget_transport::fsutil::write_private_file_atomic(
                &credential_path(&root, instance),
                b"preuve",
            )
            .unwrap();
        }
        let inconnu = names.join("active-format-inconnu");
        bridget_transport::fsutil::write_private_file_atomic(&inconnu, b"x").unwrap();
        let connectees = std::collections::HashSet::from(["i-connectee".to_string()]);
        let sept_jours = Duration::from_secs(7 * 24 * 3600);

        // Fichiers récents : seul le marqueur du disparu part, sans délai.
        let frais = purge_stale_identity_files(&root, &connectees, sept_jours, SystemTime::now());
        assert_eq!(
            frais,
            IdentityPurge {
                markers: 1,
                names: 0,
                proofs: 0
            }
        );
        assert!(
            pids.join(vivant.to_string()).exists(),
            "le vivant garde son marqueur"
        );

        // Huit jours plus tard : nom et preuve du disparu partent aussi.
        let plus_tard = SystemTime::now() + Duration::from_secs(8 * 24 * 3600);
        let tardif = purge_stale_identity_files(&root, &connectees, sept_jours, plus_tard);
        assert_eq!(
            tardif,
            IdentityPurge {
                markers: 0,
                names: 1,
                proofs: 1
            }
        );
        assert!(nom_vivant.exists());
        assert!(!nom_disparu.exists());
        assert!(
            credential_path(&root, "i-vivante").exists(),
            "preuve d'un marqueur vivant"
        );
        assert!(
            credential_path(&root, "i-connectee").exists(),
            "preuve d'une instance connectée sans marqueur, comme un fil T3 au repos"
        );
        assert!(!credential_path(&root, "i-disparue").exists());
        assert!(inconnu.exists(), "un format inconnu n'est jamais supprimé");
        fs::remove_dir_all(root).unwrap();
    }
}
