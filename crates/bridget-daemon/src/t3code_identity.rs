//! Rattachement T3 : seuls les identifiants fournisseur attestés sont utilisables.

use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use sha2::{Digest, Sha256};

use crate::managed_process::process_birth;
use crate::mcp_identity::{AgentPidMarker, DelegatedPidMarker, process_parent};
use crate::t3code_contract::ServerRuntime;

const MAX_ROWS: usize = 512;
const MAX_PROCESSES: usize = 16384;
const MAX_BYTES: usize = 64 * 1024;
type Binding = (String, String, String);
type Session = (String, String, String);

#[derive(Clone)]
struct ProcessEvidence {
    pid: u32,
    birth: u64,
    lineage: Vec<(u32, u64)>,
    provider: String,
    sessions: Vec<String>,
}

/// Propriétaire des seuls marqueurs écrits par cette instance de pont.
/// Les preuves restent celles délivrées par l'enregistrement primaire 099.
pub(crate) struct IdentityBindings {
    root: PathBuf,
    owner: String,
    owner_birth: u64,
    owned: HashMap<u32, AgentPidMarker>,
    delegated: HashMap<u32, DelegatedPidMarker>,
}

impl IdentityBindings {
    pub(crate) fn new(root: &Path) -> Self {
        let pid = std::process::id();
        let owner_birth = process_birth(pid).unwrap_or(0);
        Self {
            root: root.to_owned(),
            owner: format!("{pid}-{owner_birth}-{}", uuid::Uuid::new_v4()),
            owner_birth,
            owned: HashMap::new(),
            delegated: HashMap::new(),
        }
    }

    /// `live` vient de l'intersection snapshot HTTP actif / liens enregistrés vivants.
    /// Un échec `lsof` transitoire est retenté une fois sans trou de publication.
    /// Deux échecs consécutifs révoquent les marqueurs : aucune identité devinée.
    pub(crate) fn refresh(
        &mut self,
        base: &Path,
        runtime: &ServerRuntime,
        live: &[Binding],
        uses_v2: bool,
    ) -> Result<usize, String> {
        self.refresh_with_retry(Duration::from_millis(50), |state| {
            state.refresh_inner(base, runtime, live, uses_v2)
        })
    }

    fn refresh_with_retry(
        &mut self,
        pause: Duration,
        mut attempt: impl FnMut(&mut Self) -> Result<usize, String>,
    ) -> Result<usize, String> {
        let first_error = match attempt(self) {
            Ok(count) => return Ok(count),
            Err(error) => error,
        };
        if first_error != "inventaire des sessions ouvertes incomplet" {
            self.clear();
            return Err(first_error);
        }
        std::thread::sleep(pause);
        let result = attempt(self);
        if result.is_err() {
            self.clear();
        }
        result
    }

    fn refresh_inner(
        &mut self,
        base: &Path,
        runtime: &ServerRuntime,
        live: &[Binding],
        uses_v2: bool,
    ) -> Result<usize, String> {
        self.reap_orphans()?;
        if live.len() > MAX_ROWS {
            return Err("trop de liens T3 pour attester l'identité".into());
        }
        if live.is_empty() {
            self.clear();
            return Ok(0);
        }
        let rows = if uses_v2 {
            read_sessions_v2(&base.join("userdata/statev2.sqlite"), live)?
        } else {
            read_sessions(&base.join("userdata/state.sqlite"), live)?
        };
        let Ok(birth_before) = process_birth(runtime.pid) else {
            return Err("serveur T3 absent de l'inventaire OS".into());
        };
        let processes = collect_processes(runtime.pid)?;
        // Session 114 : le serveur attesté ne doit pas avoir été remplacé
        // pendant la collecte. C'est sa NAISSANCE qui le prouve, pas le fichier
        // d'état : une seconde application T3 sur le même dossier utilisateur
        // réécrit ce fichier sans que NOTRE serveur change, et exiger l'égalité
        // révoquait alors toutes les identités, pour tous les fils. La
        // naissance couvre en plus le recyclage de PID, invisible au fichier.
        if process_birth(runtime.pid).ok() != Some(birth_before) {
            return Err("serveur T3 remplacé pendant la collecte d'identité".into());
        }
        let principals = primary_provider_processes(processes.clone());
        let selected = select_bindings(&rows, &principals, live);
        let delegated = select_delegated_bindings(&processes, &selected);
        let keep: HashSet<u32> = selected.iter().map(|(process, _)| process.pid).collect();
        let keep_delegated: HashSet<u32> = delegated.iter().map(|item| item.0).collect();
        let stale: Vec<u32> = self
            .owned
            .keys()
            .copied()
            .filter(|pid| !keep.contains(pid))
            .collect();
        for pid in stale {
            self.remove(pid);
        }
        let stale_delegated: Vec<u32> = self
            .delegated
            .keys()
            .copied()
            .filter(|pid| !keep_delegated.contains(pid))
            .collect();
        for pid in stale_delegated {
            self.remove_delegated(pid);
        }
        for (process, binding) in &selected {
            if !lineage_valid(process, |pid| process_birth(pid).ok())
                || process
                    .lineage
                    .windows(2)
                    .any(|pair| process_parent(pair[0].0).ok() != Some(pair[1].0))
            {
                self.remove(process.pid);
                continue;
            }
            // Charge la preuve existante sans jamais la copier ni la journaliser.
            if crate::mcp_identity::auxiliary_registration(
                &binding.1,
                &binding.2,
                &self.root.join("bridget.sock"),
            )
            .is_err()
            {
                self.remove(process.pid);
                continue;
            }
            self.publish(process, binding)?;
        }
        for (pid, parent, origin, parent_pid) in delegated {
            let Some(process) = processes.iter().find(|process| process.pid == pid) else {
                continue;
            };
            if !lineage_valid(process, |candidate| process_birth(candidate).ok())
                || process
                    .lineage
                    .windows(2)
                    .any(|pair| process_parent(pair[0].0).ok() != Some(pair[1].0))
                || !self.owned.contains_key(&parent_pid)
            {
                self.remove_delegated(pid);
                continue;
            }
            self.publish_delegated(process, parent_pid, &parent, &origin)?;
        }
        Ok(self.owned.len() + self.delegated.len())
    }

    fn publish(&mut self, process: &ProcessEvidence, binding: &Binding) -> Result<(), String> {
        if self.owner_birth == 0 {
            return Err("naissance du pont non attestée".into());
        }
        let marker_path = self.root.join("agent-pids").join(process.pid.to_string());
        if let Some(previous) = self.owned.get(&process.pid).cloned() {
            if previous.pid == process.pid
                && previous.birth == process.birth
                && previous.instance_id == binding.2
                && read_marker(&marker_path).as_ref() == Some(&previous)
                && crate::mcp_identity::read_name(&previous.name_file).as_deref()
                    == Some(binding.1.as_str())
            {
                return Ok(());
            }
            if read_marker(&marker_path).as_ref() != Some(&previous) {
                self.owned.remove(&process.pid);
                return Ok(());
            }
            return self.replace_owned(process, binding, previous);
        }
        // Un autre wrapper/pont reste propriétaire de son marqueur, même périmé.
        if fs::symlink_metadata(&marker_path).is_ok() {
            return Ok(());
        }
        let name_file = self
            .root
            .join("agent-names")
            .join(format!("t3-identity-{}-{}", self.owner, process.pid));
        let marker = AgentPidMarker {
            pid: process.pid,
            birth: process.birth,
            instance_id: binding.2.clone(),
            name_file,
        };
        crate::environment::ensure_private_directory(&self.root)?;
        crate::environment::ensure_private_directory(&self.root.join("agent-names"))?;
        crate::environment::validate_state_file(&marker.name_file, false)?;
        bridget_transport::fsutil::write_private_file_atomic(
            &marker.name_file,
            binding.1.as_bytes(),
        )
        .map_err(|e| e.to_string())?;
        if let Err(error) =
            crate::mcp_identity::write_marker_if_absent(&self.root.join("agent-pids"), &marker)
        {
            let _ = fs::remove_file(&marker.name_file);
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                return Ok(());
            }
            return Err(error.to_string());
        }
        self.owned.insert(process.pid, marker);
        Ok(())
    }

    fn replace_owned(
        &mut self,
        process: &ProcessEvidence,
        binding: &Binding,
        previous: AgentPidMarker,
    ) -> Result<(), String> {
        let (owner_pid, suffix) = self
            .owner
            .split_once('-')
            .ok_or("propriétaire du marqueur T3 invalide")?;
        let (owner_birth, _) = suffix
            .split_once('-')
            .ok_or("propriétaire du marqueur T3 invalide")?;
        let name_file = self.root.join("agent-names").join(format!(
            "t3-identity-{owner_pid}-{owner_birth}-{}-{}",
            uuid::Uuid::new_v4(),
            process.pid
        ));
        let marker = AgentPidMarker {
            pid: process.pid,
            birth: process.birth,
            instance_id: binding.2.clone(),
            name_file,
        };
        crate::environment::ensure_private_directory(&self.root)?;
        crate::environment::ensure_private_directory(&self.root.join("agent-names"))?;
        crate::environment::validate_state_file(&marker.name_file, false)?;
        bridget_transport::fsutil::write_private_file_atomic(
            &marker.name_file,
            binding.1.as_bytes(),
        )
        .map_err(|e| e.to_string())?;
        if let Err(error) = crate::mcp_identity::write_marker(
            &self.root.join("agent-pids"),
            marker.pid,
            marker.birth,
            &marker.instance_id,
            &marker.name_file,
        ) {
            let _ = fs::remove_file(&marker.name_file);
            return Err(error.to_string());
        }
        self.owned.insert(process.pid, marker);
        let _ = fs::remove_file(previous.name_file);
        Ok(())
    }

    fn publish_delegated(
        &mut self,
        process: &ProcessEvidence,
        parent_pid: u32,
        parent: &Binding,
        origin: &bridget_core::DelegatedOrigin,
    ) -> Result<(), String> {
        if self.owner_birth == 0 {
            return Err("naissance du pont non attestée".into());
        }
        let directory = self.root.join("delegated-pids");
        let path = directory.join(process.pid.to_string());
        let parent_marker = self
            .owned
            .get(&parent_pid)
            .filter(|marker| marker.instance_id == parent.2)
            .ok_or("marqueur principal du parent absent")?;
        let marker = DelegatedPidMarker {
            pid: process.pid,
            birth: process.birth,
            parent_instance_id: parent.2.clone(),
            parent_name_file: parent_marker.name_file.clone(),
            provider: origin.provider.clone(),
            child_ref: origin.child_ref.clone(),
            owner: self.owner.clone(),
        };
        if let Some(previous) = self.delegated.get(&process.pid).cloned() {
            if previous == marker && read_delegated_marker(&path).as_ref() == Some(&previous) {
                return Ok(());
            }
            if read_delegated_marker(&path).as_ref() != Some(&previous) {
                self.delegated.remove(&process.pid);
                return Ok(());
            }
            bridget_transport::fsutil::write_private_file_atomic(
                &path,
                &serde_json::to_vec(&marker).map_err(|error| error.to_string())?,
            )
            .map_err(|error| error.to_string())?;
            self.delegated.insert(process.pid, marker);
            return Ok(());
        }
        if fs::symlink_metadata(&path).is_ok() {
            return Ok(());
        }
        crate::mcp_identity::write_delegated_marker_if_absent(&directory, &marker)
            .map_err(|error| error.to_string())?;
        self.delegated.insert(process.pid, marker);
        Ok(())
    }

    fn remove(&mut self, pid: u32) {
        if let Some(marker) = self.owned.remove(&pid) {
            let path = self.root.join("agent-pids").join(pid.to_string());
            if read_marker(&path).as_ref() == Some(&marker) {
                let _ = fs::remove_file(path);
            }
            let _ = fs::remove_file(marker.name_file);
        }
    }

    fn remove_delegated(&mut self, pid: u32) {
        if let Some(marker) = self.delegated.remove(&pid) {
            let path = self.root.join("delegated-pids").join(pid.to_string());
            if read_delegated_marker(&path).as_ref() == Some(&marker) {
                let _ = fs::remove_file(path);
            }
        }
    }

    pub(crate) fn clear(&mut self) {
        for pid in self.delegated.keys().copied().collect::<Vec<_>>() {
            self.remove_delegated(pid);
        }
        for pid in self.owned.keys().copied().collect::<Vec<_>>() {
            self.remove(pid);
        }
    }

    /// Un arrêt forcé ne déclenche pas Drop. Au cycle suivant, récupérer
    /// uniquement les marqueurs de ce namespace dont le pont propriétaire
    /// identifié par PID + naissance est attesté mort (ou son PID recyclé).
    fn reap_orphans(&self) -> Result<(), String> {
        for directory in [
            &self.root,
            &self.root.join("agent-names"),
            &self.root.join("agent-pids"),
            &self.root.join("delegated-pids"),
        ] {
            crate::environment::validate_private_directory_if_present(directory)?;
        }
        let directory = self.root.join("agent-pids");
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(_) => return Err("inventaire des marqueurs T3 indisponible".into()),
        };
        let entries: Vec<_> = entries
            .take(MAX_PROCESSES + 1)
            .collect::<Result<_, _>>()
            .map_err(|_| "inventaire marqueurs incomplet")?;
        if entries.len() > MAX_PROCESSES {
            return Err("trop de marqueurs pour une reprise sûre".into());
        }
        for entry in entries {
            let Some(marker) = read_marker(&entry.path()) else {
                continue;
            };
            if entry.file_name().to_str() != Some(marker.pid.to_string().as_str()) {
                continue;
            }
            let Some((owner_pid, owner_birth)) = marker_owner(&self.root, &marker) else {
                continue;
            };
            let dead = match process_birth(owner_pid) {
                Ok(actual) => actual != owner_birth,
                Err(error) => {
                    error.kind() == std::io::ErrorKind::NotFound
                        || error.raw_os_error() == Some(libc::ESRCH)
                }
            };
            if dead && read_marker(&entry.path()).as_ref() == Some(&marker) {
                fs::remove_file(entry.path())
                    .map_err(|_| "retrait du marqueur T3 orphelin impossible")?;
                let _ = fs::remove_file(marker.name_file);
            }
        }
        let delegated_directory = self.root.join("delegated-pids");
        let delegated_entries = match fs::read_dir(&delegated_directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(_) => return Err("inventaire des marqueurs enfant indisponible".into()),
        };
        let delegated_entries: Vec<_> = delegated_entries
            .take(MAX_PROCESSES + 1)
            .collect::<Result<_, _>>()
            .map_err(|_| "inventaire marqueurs enfant incomplet")?;
        if delegated_entries.len() > MAX_PROCESSES {
            return Err("trop de marqueurs enfant pour une reprise sûre".into());
        }
        for entry in delegated_entries {
            let Some(marker) = read_delegated_marker(&entry.path()) else {
                continue;
            };
            if entry.file_name().to_str() != Some(marker.pid.to_string().as_str()) {
                continue;
            }
            let Some((owner_pid, owner_birth)) = delegated_owner(&marker.owner) else {
                continue;
            };
            let dead = match process_birth(owner_pid) {
                Ok(actual) => actual != owner_birth,
                Err(error) => {
                    error.kind() == std::io::ErrorKind::NotFound
                        || error.raw_os_error() == Some(libc::ESRCH)
                }
            };
            if dead && read_delegated_marker(&entry.path()).as_ref() == Some(&marker) {
                fs::remove_file(entry.path())
                    .map_err(|_| "retrait du marqueur enfant orphelin impossible")?;
            }
        }
        Ok(())
    }
}

fn marker_owner(root: &Path, marker: &AgentPidMarker) -> Option<(u32, u64)> {
    let name = marker.name_file.file_name()?.to_str()?;
    if marker.name_file != root.join("agent-names").join(name) {
        return None;
    }
    let (owner_pid, suffix) = name.strip_prefix("t3-identity-")?.split_once('-')?;
    let (owner_birth, suffix) = suffix.split_once('-')?;
    let (nonce, provider_pid) = suffix.rsplit_once('-')?;
    let owner_pid: u32 = owner_pid.parse().ok()?;
    let owner_birth: u64 = owner_birth.parse().ok()?;
    if owner_pid <= 1
        || owner_pid > i32::MAX as u32
        || owner_birth == 0
        || provider_pid.parse::<u32>().ok()? != marker.pid
        || uuid::Uuid::parse_str(nonce).ok()?.to_string() != nonce
    {
        return None;
    }
    Some((owner_pid, owner_birth))
}

fn delegated_owner(owner: &str) -> Option<(u32, u64)> {
    let (owner_pid, suffix) = owner.split_once('-')?;
    let (owner_birth, nonce) = suffix.split_once('-')?;
    let owner_pid: u32 = owner_pid.parse().ok()?;
    let owner_birth: u64 = owner_birth.parse().ok()?;
    if owner_pid <= 1
        || owner_pid > i32::MAX as u32
        || owner_birth == 0
        || uuid::Uuid::parse_str(nonce).ok()?.to_string() != nonce
    {
        return None;
    }
    Some((owner_pid, owner_birth))
}

impl Drop for IdentityBindings {
    fn drop(&mut self) {
        self.clear();
    }
}

fn read_marker(path: &Path) -> Option<AgentPidMarker> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)
        .ok()?;
    if !file.metadata().ok()?.is_file() {
        return None;
    }
    let mut bytes = Vec::new();
    file.take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > MAX_BYTES {
        return None;
    }
    serde_json::from_slice(&bytes).ok()
}

fn read_delegated_marker(path: &Path) -> Option<DelegatedPidMarker> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)
        .ok()?;
    if !file.metadata().ok()?.is_file() {
        return None;
    }
    let mut bytes = Vec::new();
    file.take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > MAX_BYTES {
        return None;
    }
    serde_json::from_slice(&bytes).ok()
}

/// Seul le fil fournisseur principal actif atteste l'identité, jamais un enfant
/// ni une ancienne session conservée après un changement de fournisseur.
fn read_sessions_v2(path: &Path, live: &[Binding]) -> Result<Vec<Session>, String> {
    if live.is_empty() {
        return Ok(Vec::new());
    }
    if live.len() > MAX_ROWS {
        return Err("trop de fils T3 actifs".into());
    }
    let db = rusqlite::Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|_| "état des sessions T3 V2 indisponible (lecture seule)")?;
    db.busy_timeout(Duration::from_millis(100))
        .map_err(|_| "sessions T3 V2 occupées")?;
    let placeholders = std::iter::repeat_n("?", live.len())
        .collect::<Vec<_>>()
        .join(",");
    let query = format!("SELECT substr(t.thread_id,1,1025), substr(p.driver,1,65), substr(p.payload_json,1,65537)
        FROM orchestration_v2_projection_threads t
        JOIN orchestration_v2_projection_provider_threads p ON p.provider_thread_id=t.active_provider_thread_id
        JOIN orchestration_v2_projection_provider_sessions s ON s.provider_session_id=p.provider_session_id
        WHERE t.thread_id IN ({placeholders}) AND p.thread_id=t.thread_id AND p.owner_node_id IS NULL
        AND p.provider_instance_id=t.provider_instance_id AND s.provider_instance_id=p.provider_instance_id
        AND s.driver=p.driver AND p.status IN ('idle','active')
        AND s.status IN ('starting','ready','running','waiting') AND t.deleted_at IS NULL
        LIMIT 513");
    let mut statement = db
        .prepare(&query)
        .map_err(|_| "schéma des sessions T3 V2 incompatible")?;
    let mut rows = statement
        .query(rusqlite::params_from_iter(
            live.iter().map(|binding| &binding.0),
        ))
        .map_err(|_| "sessions T3 V2 illisibles")?;
    let mut result = Vec::new();
    let mut count = 0;
    while let Some(row) = rows
        .next()
        .map_err(|_| "lecture des sessions T3 V2 interrompue")?
    {
        count += 1;
        if count > MAX_ROWS {
            return Err("inventaire des sessions T3 V2 tronqué".into());
        }
        let thread: String = row
            .get(0)
            .map_err(|_| "identifiant de fil T3 V2 invalide")?;
        let provider: String = row.get(1).map_err(|_| "fournisseur T3 V2 invalide")?;
        let payload: String = row.get(2).map_err(|_| "session T3 V2 invalide")?;
        if thread.len() > 1024 || provider.len() > 64 || payload.len() > MAX_BYTES {
            return Err("session T3 V2 au-delà de la borne".into());
        }
        let value: serde_json::Value =
            serde_json::from_str(&payload).map_err(|_| "session T3 V2 invalide")?;
        let native = &value["nativeThreadRef"];
        if !matches!(provider.as_str(), "codex" | "claude" | "cursor")
            || native["strength"] != "strong"
            || native["driver"] != provider
        {
            continue;
        }
        if let Some(id) = native["nativeId"]
            .as_str()
            .filter(|id| !id.is_empty() && id.len() <= 256)
        {
            result.push((thread, provider, id.into()));
        }
    }
    Ok(result)
}

fn read_sessions(path: &Path, live: &[Binding]) -> Result<Vec<Session>, String> {
    if live.is_empty() {
        return Ok(Vec::new());
    }
    if live.len() > MAX_ROWS {
        return Err("trop de fils T3 actifs".into());
    }
    let db = rusqlite::Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|_| "état des sessions T3 indisponible (lecture seule)".to_string())?;
    db.busy_timeout(Duration::from_millis(100))
        .map_err(|e| e.to_string())?;
    // Seules ces trois colonnes sont lues ; substr borne même les valeurs corrompues.
    let placeholders = std::iter::repeat_n("?", live.len())
        .collect::<Vec<_>>()
        .join(",");
    let query = format!(
        "SELECT substr(thread_id,1,1025), substr(provider_name,1,65), substr(resume_cursor_json,1,65537) FROM provider_session_runtime WHERE thread_id IN ({placeholders}) LIMIT 513"
    );
    let mut statement = db
        .prepare(&query)
        .map_err(|_| "schéma des sessions T3 incompatible".to_string())?;
    let mut rows = statement
        .query(rusqlite::params_from_iter(
            live.iter().map(|binding| &binding.0),
        ))
        .map_err(|_| "sessions T3 illisibles".to_string())?;
    let mut result = Vec::new();
    let mut count = 0;
    while let Some(row) = rows
        .next()
        .map_err(|_| "lecture des sessions T3 interrompue".to_string())?
    {
        count += 1;
        if count > MAX_ROWS {
            return Err("inventaire des sessions T3 tronqué".into());
        }
        let thread: String = row.get(0).map_err(|_| "identifiant de fil T3 invalide")?;
        let provider: String = row.get(1).map_err(|_| "fournisseur T3 invalide")?;
        let cursor: Option<String> = row.get(2).map_err(|_| "curseur T3 invalide")?;
        if thread.len() > 1024
            || provider.len() > 64
            || cursor.as_ref().is_some_and(|v| v.len() > MAX_BYTES)
        {
            return Err("session T3 au-delà de la borne".into());
        }
        let provider = crate::t3code::agent_type_for(&provider);
        let Some(value) =
            cursor.and_then(|cursor| serde_json::from_str::<serde_json::Value>(&cursor).ok())
        else {
            continue;
        };
        let native = match provider.as_str() {
            "codex" => value.get("threadId"),
            "claude" => value.get("resume").or_else(|| value.get("sessionId")),
            // Session 114 : Cursor publie son identifiant ACP, que le processus
            // `cursor-agent` tient ouvert sous `acp-sessions/<id>/`.
            "cursor" => value.get("sessionId"),
            _ => continue,
        };
        if let Some(id) = native
            .and_then(|v| v.as_str())
            .filter(|id| !id.is_empty() && id.len() <= 256)
        {
            result.push((thread, provider, id.to_owned()));
        }
    }
    Ok(result)
}

fn select_bindings<'a>(
    rows: &[Session],
    processes: &'a [ProcessEvidence],
    live: &'a [Binding],
) -> Vec<(&'a ProcessEvidence, &'a Binding)> {
    let mut selected = Vec::new();
    let mut counts: HashMap<&str, usize> = HashMap::new();
    let mut by_thread: HashMap<&str, Vec<&Binding>> = HashMap::new();
    for binding in live {
        by_thread.entry(&binding.0).or_default().push(binding);
    }
    let mut by_native: HashMap<(&str, &str), Vec<&Binding>> = HashMap::new();
    for row in rows {
        if let Some(bindings) = by_thread.get(row.0.as_str()) {
            by_native
                .entry((&row.1, &row.2))
                .or_default()
                .extend(bindings.iter().copied());
        }
    }
    for process in processes {
        let candidates: Vec<_> = process
            .sessions
            .iter()
            .filter_map(|id| by_native.get(&(process.provider.as_str(), id.as_str())))
            .flatten()
            .copied()
            .collect();
        // Une correspondance d'un processus ambigu compte aussi : l'exclure
        // d'abord transformerait artificiellement un autre processus en unique.
        for binding in &candidates {
            *counts.entry(&binding.0).or_default() += 1;
        }
        if candidates.len() == 1 {
            selected.push((process, candidates[0]));
        }
    }
    selected.retain(|(_, binding)| counts.get(binding.0.as_str()) == Some(&1));
    selected
}

fn select_delegated_bindings(
    processes: &[ProcessEvidence],
    selected: &[(&ProcessEvidence, &Binding)],
) -> Vec<(u32, Binding, bridget_core::DelegatedOrigin, u32)> {
    let selected_by_pid: HashMap<u32, &Binding> = selected
        .iter()
        .map(|(process, binding)| (process.pid, *binding))
        .collect();
    let provider_pids: HashSet<u32> = processes.iter().map(|process| process.pid).collect();
    let mut delegated = Vec::new();
    for process in processes {
        if selected_by_pid.contains_key(&process.pid) || process.sessions.len() != 1 {
            continue;
        }
        let nested = process
            .lineage
            .iter()
            .skip(1)
            .any(|(pid, _)| provider_pids.contains(pid));
        if !nested {
            continue;
        }
        let Some((parent_pid, parent)) = process
            .lineage
            .iter()
            .skip(1)
            .find_map(|(pid, _)| selected_by_pid.get(pid).map(|binding| (*pid, *binding)))
        else {
            continue;
        };
        let mut digest = Sha256::new();
        digest.update(process.provider.as_bytes());
        digest.update([0]);
        digest.update(process.sessions[0].as_bytes());
        let child_ref = format!("{:x}", digest.finalize())[..16].to_string();
        delegated.push((
            process.pid,
            (*parent).clone(),
            bridget_core::DelegatedOrigin {
                provider: process.provider.clone(),
                child_ref,
            },
            parent_pid,
        ));
    }
    delegated
}

fn lineage_valid(process: &ProcessEvidence, birth: impl Fn(u32) -> Option<u64>) -> bool {
    !process.lineage.is_empty()
        && process
            .lineage
            .iter()
            .all(|(pid, expected)| birth(*pid) == Some(*expected))
}

fn claude_session(args: &[String]) -> Option<String> {
    let mut session = None;
    for (index, arg) in args.iter().enumerate().skip(1) {
        let value = if arg == "--resume" || arg == "--session-id" {
            args.get(index + 1).map(String::as_str)
        } else {
            arg.strip_prefix("--resume=")
                .or_else(|| arg.strip_prefix("--session-id="))
        };
        if let Some(value) = value {
            if session.is_some() || value.is_empty() || value.len() > 256 || value.starts_with('-')
            {
                return None;
            }
            session = Some(value.to_owned());
        }
    }
    session
}

fn rollout_session(path: &Path) -> Option<String> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)
        .ok()?;
    if !file.metadata().ok()?.is_file() {
        return None;
    }
    let mut bytes = Vec::new();
    file.take(MAX_BYTES as u64).read_to_end(&mut bytes).ok()?;
    // session_meta doit être la première ligne complète, jamais un texte ultérieur.
    let end = bytes.iter().position(|byte| *byte == b'\n')?;
    let value: serde_json::Value = serde_json::from_slice(&bytes[..end]).ok()?;
    if value.get("type")?.as_str()? != "session_meta" {
        return None;
    }
    value
        .get("payload")?
        .get("id")?
        .as_str()
        .filter(|id| !id.is_empty() && id.len() <= 256)
        .map(str::to_owned)
}

fn collect_processes(server: u32) -> Result<Vec<ProcessEvidence>, String> {
    let pids = process_ids().map_err(|_| "inventaire OS indisponible".to_string())?;
    let mut snapshot = HashMap::new();
    for pid in pids {
        if let (Ok(parent), Ok(birth)) = (process_parent(pid), process_birth(pid)) {
            snapshot.insert(pid, (parent, birth));
        }
    }
    if !snapshot.contains_key(&server) {
        return Err("serveur T3 absent de l'inventaire OS".into());
    }
    let mut candidates = Vec::new();
    for (&pid, &(_, birth)) in &snapshot {
        if pid == server {
            continue;
        }
        let mut lineage = Vec::new();
        let mut current = pid;
        for _ in 0..16 {
            let Some(&(parent, birth)) = snapshot.get(&current) else {
                break;
            };
            lineage.push((current, birth));
            if current == server {
                break;
            }
            current = parent;
        }
        if lineage.last().map(|v| v.0) != Some(server) {
            continue;
        }
        let Some(args) = process_arguments(pid) else {
            continue;
        };
        let executable = args
            .first()
            .and_then(|arg| Path::new(arg).file_name())
            .and_then(|name| name.to_str());
        let (provider, sessions) = match executable {
            Some("codex") if args.get(1).is_some_and(|arg| arg == "app-server") => {
                ("codex", Vec::new())
            }
            Some("claude") => match claude_session(&args) {
                Some(id) => ("claude", vec![id]),
                None => continue,
            },
            // Session 114 : Cursor ne porte pas son identifiant de session en
            // ligne de commande ; il le révèle par le dossier `acp-sessions`
            // qu'il tient ouvert, exactement comme Codex par son rollout.
            Some("cursor-agent") if args.iter().any(|arg| arg == "acp") => ("cursor", Vec::new()),
            _ => continue,
        };
        candidates.push(ProcessEvidence {
            pid,
            birth,
            lineage,
            provider: provider.into(),
            sessions,
        });
        if candidates.len() > 256 {
            return Err("trop de processus fournisseur T3".into());
        }
    }
    // Lire aussi les sessions des fournisseurs imbriqués. Ils ne deviendront
    // pas des principaux, mais leur empreinte atteste la provenance déléguée.
    let file_backed: Vec<_> = candidates
        .iter()
        .filter(|p| session_file_backed(&p.provider))
        .map(|p| p.pid)
        .collect();
    let files = crate::runtime::open_session_files(&file_backed, is_session_file)
        .map_err(|_| "inventaire des sessions ouvertes incomplet".to_string())?;
    complete_session_inventory(candidates, &files)
}

fn primary_provider_processes(candidates: Vec<ProcessEvidence>) -> Vec<ProcessEvidence> {
    let provider_pids: HashSet<u32> = candidates.iter().map(|process| process.pid).collect();
    candidates
        .into_iter()
        .filter(|process| {
            !process
                .lineage
                .iter()
                .skip(1)
                .any(|(pid, _)| provider_pids.contains(pid))
        })
        .collect()
}

/// Fournisseurs dont l'identifiant de session se lit dans un fichier ouvert.
fn session_file_backed(provider: &str) -> bool {
    matches!(provider, "codex" | "cursor")
}

/// Fichier de session reconnu, tous fournisseurs confondus. Le tri par
/// fournisseur est fait ensuite : un `.jsonl` ouvert par Cursor ne doit pas
/// devenir une session Cursor, ni l'inverse.
fn is_session_file(path: &Path) -> bool {
    crate::runtime::is_codex_rollout(path) || cursor_session(path).is_some()
}

/// Session Cursor portée par le chemin : `…/acp-sessions/<id>/store.db`.
/// Le dossier est la seule preuve disponible — `cursor-agent` ne met pas son
/// identifiant en ligne de commande et l'a déjà négocié par le protocole ACP.
fn cursor_session(path: &Path) -> Option<String> {
    let mut components = path.components();
    while let Some(component) = components.next() {
        if component.as_os_str() != "acp-sessions" {
            continue;
        }
        let id = components.next()?.as_os_str().to_str()?;
        return (!id.is_empty()
            && id.len() <= 256
            && id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_'))
        .then(|| id.to_owned());
    }
    None
}

/// Fichiers de session appartenant à CE fournisseur, parmi ceux qu'il tient
/// ouverts. Un processus ouvre bien d'autres fichiers : sans ce tri, une
/// lecture impossible sur un fichier étranger invaliderait tout le cycle.
fn session_files_of<'a>(provider: &str, paths: &'a [PathBuf]) -> Vec<&'a PathBuf> {
    paths
        .iter()
        .filter(|path| match provider {
            "codex" => crate::runtime::is_codex_rollout(path),
            "cursor" => cursor_session(path).is_some(),
            _ => false,
        })
        .collect()
}

fn session_id_of(provider: &str, path: &Path) -> Option<String> {
    match provider {
        "codex" => rollout_session(path),
        "cursor" => cursor_session(path),
        _ => None,
    }
}

/// Frontière unique de complétude : perdre un candidat peut créer une fausse
/// unicité pour un autre. L'échec invalide tout le cycle, pas seulement ce PID.
fn complete_session_inventory(
    mut candidates: Vec<ProcessEvidence>,
    files: &HashMap<u32, Vec<PathBuf>>,
) -> Result<Vec<ProcessEvidence>, String> {
    for process in &mut candidates {
        if let Some(paths) = files.get(&process.pid) {
            if paths.len() > 256 {
                return Err("trop de fichiers de session ouverts".into());
            }
            // Une lecture manquante peut cacher un second fil : remonter
            // l'échec jusqu'à refresh, qui retire tous les marqueurs possédés.
            process.sessions = session_files_of(&process.provider, paths)
                .into_iter()
                .map(|path| session_id_of(&process.provider, path))
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| {
                    "métadonnée de session illisible : inventaire incomplet".to_string()
                })?;
            process.sessions.sort();
            process.sessions.dedup();
        }
    }
    Ok(candidates)
}

#[cfg(target_os = "macos")]
fn process_ids() -> std::io::Result<Vec<u32>> {
    let mut pids = vec![0_i32; MAX_PROCESSES + 1];
    // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
    let bytes = unsafe {
        libc::proc_listpids(
            1,
            0,
            pids.as_mut_ptr().cast(),
            std::mem::size_of_val(pids.as_slice()) as i32,
        )
    };
    if bytes < 0 || bytes as usize >= std::mem::size_of_val(pids.as_slice()) {
        return Err(std::io::Error::other("inventaire processus incomplet"));
    }
    pids.truncate(bytes as usize / std::mem::size_of::<i32>());
    Ok(pids
        .into_iter()
        .filter(|pid| *pid > 0)
        .map(|pid| pid as u32)
        .collect())
}

#[cfg(target_os = "linux")]
fn process_ids() -> std::io::Result<Vec<u32>> {
    let mut pids = Vec::new();
    for entry in fs::read_dir("/proc")? {
        if let Some(pid) = entry?
            .file_name()
            .to_str()
            .and_then(|v| v.parse::<u32>().ok())
        {
            pids.push(pid);
        }
        if pids.len() > MAX_PROCESSES {
            return Err(std::io::Error::other("inventaire processus incomplet"));
        }
    }
    Ok(pids)
}

#[cfg(target_os = "macos")]
fn process_arguments(pid: u32) -> Option<Vec<String>> {
    let mut mib = [libc::CTL_KERN, libc::KERN_PROCARGS2, pid as libc::c_int];
    let mut bytes = vec![0_u8; MAX_BYTES];
    let mut size = bytes.len();
    // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
    if unsafe {
        libc::sysctl(
            mib.as_mut_ptr(),
            3,
            bytes.as_mut_ptr().cast(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    } != 0
        || size < 4
    {
        return None;
    }
    bytes.truncate(size);
    let argc = i32::from_ne_bytes(bytes[..4].try_into().ok()?);
    if !(1..=1024).contains(&argc) {
        return None;
    }
    let mut offset = 4 + bytes[4..].iter().position(|b| *b == 0)? + 1;
    while bytes.get(offset) == Some(&0) {
        offset += 1;
    }
    let mut args = Vec::new();
    for _ in 0..argc {
        let end = bytes.get(offset..)?.iter().position(|b| *b == 0)? + offset;
        args.push(std::str::from_utf8(&bytes[offset..end]).ok()?.to_owned());
        offset = end + 1;
    }
    Some(args)
}

#[cfg(target_os = "linux")]
fn process_arguments(pid: u32) -> Option<Vec<String>> {
    let mut bytes = Vec::new();
    fs::File::open(format!("/proc/{pid}/cmdline"))
        .ok()?
        .take(MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > MAX_BYTES || !bytes.ends_with(&[0]) {
        return None;
    }
    bytes.pop();
    bytes
        .split(|b| *b == 0)
        .map(|arg| std::str::from_utf8(arg).ok().map(str::to_owned))
        .collect()
}

#[cfg(test)]
#[path = "t3code_identity_spec101_identity.rs"]
mod spec101_identity;
