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
    ) -> Result<usize, String> {
        self.refresh_with_retry(Duration::from_millis(50), |state| {
            state.refresh_inner(base, runtime, live)
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
    ) -> Result<usize, String> {
        self.reap_orphans()?;
        if live.len() > MAX_ROWS {
            return Err("trop de liens T3 pour attester l'identité".into());
        }
        if live.is_empty() {
            self.clear();
            return Ok(0);
        }
        let rows = read_sessions(&base.join("userdata/state.sqlite"), live)?;
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
mod spec101_identity {
    use super::*;

    fn binding(thread: &str) -> (String, String, String) {
        (
            thread.into(),
            crate::t3code::stable_uuid(thread),
            format!("instance-{thread}"),
        )
    }

    fn process(pid: u32, sessions: &[&str]) -> ProcessEvidence {
        ProcessEvidence {
            pid,
            birth: 10,
            lineage: vec![(pid, 10), (1, 20)],
            provider: "codex".into(),
            sessions: sessions.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn fournisseur_interne_n_est_pas_un_agent_t3_principal() {
        let mut principal = process(10, &["native-principal"]);
        principal.lineage = vec![(10, 10), (1, 20)];
        let mut interne = process(20, &["native-interne"]);
        interne.lineage = vec![(20, 30), (30, 40), (10, 10), (1, 20)];
        let mut voisin = process(40, &["native-voisin"]);
        voisin.lineage = vec![(40, 50), (1, 20)];

        let principaux = primary_provider_processes(vec![interne, voisin, principal]);
        let pids: HashSet<u32> = principaux.into_iter().map(|process| process.pid).collect();

        assert_eq!(pids, HashSet::from([10, 40]));
    }

    #[test]
    fn spec133_fournisseur_interne_recoit_le_parent_unique_et_une_reference_opaque() {
        let mut principal = process(10, &["native-principal"]);
        principal.lineage = vec![(10, 10), (1, 20)];
        let mut interne = process(20, &["native-interne"]);
        interne.lineage = vec![(20, 30), (30, 40), (10, 10), (1, 20)];
        let processes = vec![principal, interne];
        let principals = primary_provider_processes(processes.clone());
        let live = [binding("parent")];
        let rows = [("parent".into(), "codex".into(), "native-principal".into())];
        let selected = select_bindings(&rows, &principals, &live);

        let delegated = select_delegated_bindings(&processes, &selected);

        assert_eq!(delegated.len(), 1);
        assert_eq!(delegated[0].0, 20);
        assert_eq!(delegated[0].1, live[0]);
        assert_eq!(delegated[0].2.provider, "codex");
        assert_eq!(delegated[0].2.child_ref.len(), 16);
        assert!(!delegated[0].2.child_ref.contains("native-interne"));
    }

    #[test]
    fn deux_fils_meme_cwd_sous_agent_plus_recent_ne_change_pas_identite() {
        // Le cwd et le mtime n'entrent volontairement pas dans ce contrat.
        let rows = vec![
            ("a".into(), "codex".into(), "native-a".into()),
            ("b".into(), "codex".into(), "native-b".into()),
        ];
        let processes = [
            process(2, &["child", "native-a"]),
            process(3, &["native-b"]),
        ];
        let live = [binding("a"), binding("b")];
        let selected = select_bindings(&rows, &processes, &live);
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].1.0, "a");
        assert_eq!(selected[1].1.0, "b");
    }

    #[test]
    fn plusieurs_correspondances_ou_processus_refuses() {
        let rows = vec![
            ("a".into(), "codex".into(), "one".into()),
            ("b".into(), "codex".into(), "two".into()),
        ];
        assert!(
            select_bindings(
                &rows,
                &[process(2, &["one", "two"])],
                &[binding("a"), binding("b")]
            )
            .is_empty()
        );
        assert!(
            select_bindings(
                &rows,
                &[process(2, &["one"]), process(3, &["one"])],
                &[binding("a")]
            )
            .is_empty()
        );
        assert!(
            select_bindings(
                &rows,
                &[process(2, &["one", "two"]), process(3, &["one"])],
                &[binding("a"), binding("b")]
            )
            .is_empty()
        );
    }

    #[test]
    fn pid_recycle_ou_parent_recycle_refuse() {
        let evidence = process(2, &["native"]);
        assert!(lineage_valid(&evidence, |pid| Some(if pid == 1 {
            20
        } else {
            10
        })));
        assert!(!lineage_valid(&evidence, |_| Some(99)));
        assert!(!lineage_valid(&evidence, |pid| Some(if pid == 1 {
            21
        } else {
            10
        })));
    }

    #[test]
    fn sqlite_absent_ne_cree_pas_de_base_et_curseur_nul_ignore() {
        let temp = std::env::temp_dir().join(format!("bi101-{}", uuid::Uuid::new_v4()));
        crate::environment::ensure_private_directory(&temp).unwrap();
        let path = temp.join("state.sqlite");
        let live = [binding("a"), binding("b")];
        assert!(read_sessions(&path, &live).is_err());
        assert!(!path.exists());
        let db = rusqlite::Connection::open(&path).unwrap();
        db.execute_batch("CREATE TABLE provider_session_runtime(thread_id TEXT, provider_name TEXT, resume_cursor_json TEXT); INSERT INTO provider_session_runtime VALUES ('a','codex',NULL), ('b','codex','{\"threadId\":\"native-b\"}');").unwrap();
        assert_eq!(
            read_sessions(&path, &live).unwrap(),
            vec![("b".into(), "codex".into(), "native-b".into())]
        );
        db.execute_batch("WITH RECURSIVE n(x) AS (SELECT 1 UNION ALL SELECT x+1 FROM n WHERE x<600) INSERT INTO provider_session_runtime SELECT 'archive-'||x,'codex','{\"threadId\":\"old\"}' FROM n;").unwrap();
        assert_eq!(
            read_sessions(&path, &live).unwrap().len(),
            1,
            "l'historique inactif ne consomme pas la borne"
        );
        db.execute_batch("INSERT INTO provider_session_runtime VALUES ('claude-resume','claude','{\"threadId\":\"not-provider\",\"resume\":\"native-resume\"}'),('claude-session','claude','{\"sessionId\":\"native-session\"}'),('claude-wrong','claude','{\"threadId\":\"not-provider\"}');").unwrap();
        assert_eq!(
            read_sessions(
                &path,
                &[
                    binding("claude-resume"),
                    binding("claude-session"),
                    binding("claude-wrong")
                ]
            )
            .unwrap(),
            vec![
                (
                    "claude-resume".into(),
                    "claude".into(),
                    "native-resume".into()
                ),
                (
                    "claude-session".into(),
                    "claude".into(),
                    "native-session".into()
                )
            ]
        );
        drop(db);
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn sqlite_alias_reel_claude_agent_utilise_la_normalisation_du_pont() {
        let root = std::env::temp_dir().join(format!("bi101-{}", uuid::Uuid::new_v4()));
        crate::environment::ensure_private_directory(&root).unwrap();
        let path = root.join("state.sqlite");
        let db = rusqlite::Connection::open(&path).unwrap();
        db.execute_batch("CREATE TABLE provider_session_runtime(thread_id TEXT, provider_name TEXT, resume_cursor_json TEXT); INSERT INTO provider_session_runtime VALUES ('claude-real','claudeAgent','{\"resume\":\"session-native\"}');").unwrap();
        let rows = read_sessions(&path, &[binding("claude-real")]).unwrap();
        assert_eq!(
            rows,
            vec![(
                "claude-real".into(),
                "claude".into(),
                "session-native".into()
            )]
        );
        drop(db);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn inventaire_partiellement_illisible_ne_rend_pas_autre_processus_unique() {
        let root = std::env::temp_dir().join(format!("bi101-{}", uuid::Uuid::new_v4()));
        crate::environment::ensure_private_directory(&root).unwrap();
        let readable = root.join("a.jsonl");
        let missing = root.join("unreadable.jsonl");
        fs::write(
            &readable,
            "{\"type\":\"session_meta\",\"payload\":{\"id\":\"A\"}}\n",
        )
        .unwrap();
        let candidates = vec![process(2, &[]), process(3, &[])];
        let files = HashMap::from([(2, vec![readable.clone()]), (3, vec![readable, missing])]);
        let complete = complete_session_inventory(vec![process(2, &[])], &files).unwrap();
        assert_eq!(complete[0].sessions, vec!["A"]);
        // P1=A, P2=A+inaccessible : aucun jeu de candidats partiel ne doit
        // atteindre select_bindings et rendre artificiellement P1 unique.
        assert!(complete_session_inventory(candidates, &files).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn claude_exige_flag_exact_et_unique() {
        assert_eq!(
            claude_session(&["claude".into(), "--resume".into(), "session-a".into()]),
            Some("session-a".into())
        );
        assert!(
            claude_session(&[
                "claude".into(),
                "--prompt".into(),
                "--resume session-a".into()
            ])
            .is_none()
        );
        assert!(
            claude_session(&[
                "claude".into(),
                "--resume".into(),
                "a".into(),
                "--session-id".into(),
                "b".into()
            ])
            .is_none()
        );
    }

    #[test]
    fn rollout_lit_metadata_exacte_et_refuse_fragment() {
        let root = std::env::temp_dir().join(format!("bi101-{}", uuid::Uuid::new_v4()));
        crate::environment::ensure_private_directory(&root).unwrap();
        let path = root.join("rollout avec espaces.jsonl");
        fs::write(&path, "{\"type\":\"session_meta\",\"payload\":{\"id\":\"parent\"}}\n{\"type\":\"other\",\"payload\":{\"id\":\"child\"}}\n").unwrap();
        assert_eq!(rollout_session(&path).as_deref(), Some("parent"));
        fs::write(
            &path,
            "{\"type\":\"session_meta\",\"payload\":{\"id\":\"parent\"}}",
        )
        .unwrap();
        assert_eq!(rollout_session(&path), None);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn marqueur_prive_retire_sur_clear_sans_ecraser_autre_proprietaire() {
        use std::os::unix::fs::PermissionsExt;
        let root = std::env::temp_dir().join(format!("bi101-{}", uuid::Uuid::new_v4()));
        let mut owner = IdentityBindings::new(&root);
        let evidence = process(42, &["native"]);
        owner.publish(&evidence, &binding("a")).unwrap();
        let path = root.join("agent-pids/42");
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let existing = read_marker(&path).unwrap();
        assert_eq!(
            crate::mcp_identity::write_marker_if_absent(&root.join("agent-pids"), &existing)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::AlreadyExists
        );
        struct Tree;
        impl crate::mcp_identity::ProcessTree for Tree {
            fn birth(&self, pid: u32) -> Option<u64> {
                (pid == 42).then_some(10)
            }
            fn parent(&self, _: u32) -> Option<u32> {
                None
            }
        }
        let resolved = crate::mcp_identity::resolve_identity_with(
            None,
            &root.join("agent-pids"),
            None,
            42,
            &Tree,
        )
        .unwrap();
        assert_eq!(resolved.name, binding("a").1);
        assert_eq!(resolved.instance_id, "instance-a");
        let mut other = IdentityBindings::new(&root);
        other.publish(&evidence, &binding("b")).unwrap();
        assert!(other.owned.is_empty());
        assert_eq!(read_marker(&path).unwrap().instance_id, "instance-a");
        owner.clear();
        assert!(!path.exists());
        other.publish(&evidence, &binding("b")).unwrap();
        owner.clear();
        assert_eq!(read_marker(&path).unwrap().instance_id, "instance-b");
        other.clear();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec133_clear_retire_seulement_le_marqueur_enfant_possede() {
        use std::os::unix::fs::PermissionsExt;

        let root = std::env::temp_dir().join(format!("bi133-{}", uuid::Uuid::new_v4()));
        let mut owner = IdentityBindings::new(&root);
        let parent = process(42, &["native-parent"]);
        let parent_binding = binding("parent");
        owner.publish(&parent, &parent_binding).unwrap();
        let mut child = process(43, &["native-child"]);
        child.lineage = vec![(43, 10), (42, 10), (1, 20)];
        let origin = bridget_core::DelegatedOrigin {
            provider: "codex".into(),
            child_ref: "0123456789abcdef".into(),
        };
        owner
            .publish_delegated(&child, 42, &parent_binding, &origin)
            .unwrap();
        let owned_path = root.join("delegated-pids/43");
        assert_eq!(
            fs::metadata(&owned_path).unwrap().permissions().mode() & 0o777,
            0o600
        );

        let third_name = root.join("agent-names/third-child-parent");
        bridget_transport::fsutil::write_private_file_atomic(
            &third_name,
            binding("third").1.as_bytes(),
        )
        .unwrap();
        let third = DelegatedPidMarker {
            pid: 44,
            birth: 10,
            parent_instance_id: "instance-third".into(),
            parent_name_file: third_name,
            provider: "codex".into(),
            child_ref: "fedcba9876543210".into(),
            owner: format!(
                "{}-{}-{}",
                std::process::id(),
                process_birth(std::process::id()).unwrap(),
                uuid::Uuid::new_v4()
            ),
        };
        crate::mcp_identity::write_delegated_marker_if_absent(&root.join("delegated-pids"), &third)
            .unwrap();

        owner.clear();
        assert!(!owned_path.exists());
        assert!(root.join("delegated-pids/44").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn inventaire_ouvert_transitoire_est_retente_avant_retrait() {
        let root = std::env::temp_dir().join(format!("bi-hotfix-{}", uuid::Uuid::new_v4()));
        let mut owner = IdentityBindings::new(&root);
        owner
            .publish(&process(42, &["native"]), &binding("a"))
            .unwrap();
        let marker = root.join("agent-pids/42");
        let mut attempts = 0;

        let result = owner.refresh_with_retry(Duration::ZERO, |state| {
            attempts += 1;
            if attempts == 1 {
                Err("inventaire des sessions ouvertes incomplet".into())
            } else {
                Ok(state.owned.len())
            }
        });

        assert_eq!(result, Ok(1));
        assert_eq!(attempts, 2);
        assert!(marker.exists(), "le marqueur reste publié pendant le retry");
        owner.clear();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn double_echec_inventaire_ouvert_revoque_le_marqueur() {
        let root = std::env::temp_dir().join(format!("bi-hotfix-{}", uuid::Uuid::new_v4()));
        let mut owner = IdentityBindings::new(&root);
        owner
            .publish(&process(42, &["native"]), &binding("a"))
            .unwrap();
        let marker = root.join("agent-pids/42");
        let mut attempts = 0;

        let result = owner.refresh_with_retry(Duration::ZERO, |_| {
            attempts += 1;
            Err("inventaire des sessions ouvertes incomplet".into())
        });

        assert_eq!(
            result.unwrap_err(),
            "inventaire des sessions ouvertes incomplet"
        );
        assert_eq!(attempts, 2);
        assert!(
            !marker.exists(),
            "deux échecs consécutifs ferment l'identité"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn remplacement_possede_prepare_la_nouvelle_identite_avant_la_bascule() {
        let root = std::env::temp_dir().join(format!("bi117-{}", uuid::Uuid::new_v4()));
        let mut owner = IdentityBindings::new(&root);
        let evidence = process(42, &["native"]);
        owner.publish(&evidence, &binding("a")).unwrap();
        let marker_path = root.join("agent-pids/42");
        let previous = read_marker(&marker_path).unwrap();

        owner.publish(&evidence, &binding("b")).unwrap();

        let current = read_marker(&marker_path).unwrap();
        assert_ne!(
            current.name_file, previous.name_file,
            "le nouveau nom doit être préparé dans un fichier distinct avant la bascule atomique"
        );
        assert!(!previous.name_file.exists());
        assert_eq!(
            crate::mcp_identity::read_name(&current.name_file),
            Some(binding("b").1)
        );
        assert_eq!(current.instance_id, "instance-b");
        owner.clear();
        fs::remove_dir_all(root).unwrap();
    }

    /// Session 114 : Cursor ne met pas son identifiant de session en ligne de
    /// commande ; il le révèle par le dossier `acp-sessions` qu'il tient ouvert.
    #[test]
    fn spec114_session_cursor_lue_dans_le_dossier_acp() {
        assert_eq!(
            cursor_session(Path::new(
                "/Users/u/.cursor/acp-sessions/df338dbf-96b3-4dbf-90f3-ee27fbf1af83/store.db"
            )),
            Some("df338dbf-96b3-4dbf-90f3-ee27fbf1af83".into())
        );
        // Les fichiers frères du même dossier désignent la même session.
        assert_eq!(
            cursor_session(Path::new("/x/acp-sessions/s-1/store.db-wal")),
            Some("s-1".into())
        );
        for refuse in [
            "/Users/u/.cursor/acp-sessions",
            "/Users/u/.codex/sessions/rollout.jsonl",
            "/x/acp-sessions/../evade/store.db",
        ] {
            assert_eq!(cursor_session(Path::new(refuse)), None, "{refuse}");
        }
    }

    /// Chaque fournisseur ne lit que SES fichiers de session : un rollout
    /// Codex ouvert par Cursor ne devient pas une session Cursor, et un
    /// fichier étranger n'invalide pas l'inventaire.
    #[test]
    fn spec114_inventaire_trie_les_fichiers_par_fournisseur() {
        let paths = vec![
            PathBuf::from("/x/acp-sessions/s-1/store.db"),
            PathBuf::from("/x/rollout.jsonl"),
            PathBuf::from("/x/node_modules/tree-sitter.node"),
        ];
        assert_eq!(
            session_files_of("cursor", &paths),
            vec![&PathBuf::from("/x/acp-sessions/s-1/store.db")]
        );
        assert_eq!(
            session_files_of("codex", &paths),
            vec![&PathBuf::from("/x/rollout.jsonl")]
        );
        assert!(session_files_of("claude", &paths).is_empty());
    }

    /// Session 114 : une seconde application T3 sur le même dossier réécrit
    /// `server-runtime.json`. Notre serveur, lui, n'a pas changé : exiger
    /// l'égalité du fichier révoquait les identités de TOUS les fils.
    #[test]
    fn spec114_fichier_d_etat_etranger_ne_revoque_pas_les_identites() {
        let root = std::env::temp_dir().join(format!("bi114-{}", uuid::Uuid::new_v4()));
        crate::environment::ensure_private_directory(&root).unwrap();
        let base = root.join("t3");
        fs::create_dir_all(base.join("userdata")).unwrap();
        let db = rusqlite::Connection::open(base.join("userdata/state.sqlite")).unwrap();
        db.execute_batch(
            "CREATE TABLE provider_session_runtime (thread_id TEXT, provider_name TEXT, resume_cursor_json TEXT);
             INSERT INTO provider_session_runtime VALUES ('fil-a','cursor','{\"sessionId\":\"s-a\"}');",
        )
        .unwrap();
        drop(db);
        fs::write(
            base.join("userdata/server-runtime.json"),
            r#"{"version":1,"pid":4294967000,"host":"127.0.0.1","port":3774,"origin":"http://127.0.0.1:3774"}"#,
        )
        .unwrap();
        let ours = ServerRuntime {
            pid: std::process::id(),
            port: 3773,
        };
        let mut bindings = IdentityBindings::new(&root);
        let result = bindings.refresh(&base, &ours, &[binding("fil-a")]);
        assert!(
            result.is_ok(),
            "un fichier d'état étranger ne révoque plus rien : {result:?}"
        );
        bindings.clear();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn disparition_et_ambiguite_revoquent_uniquement_marqueurs_possedes() {
        let root = std::env::temp_dir().join(format!("bi101-{}", uuid::Uuid::new_v4()));
        let mut owner = IdentityBindings::new(&root);
        owner
            .publish(&process(42, &["native"]), &binding("a"))
            .unwrap();
        let runtime = ServerRuntime {
            pid: 42,
            port: 1234,
        };
        assert!(
            owner
                .refresh(&root.join("absent"), &runtime, &[binding("a")])
                .is_err()
        );
        assert!(!root.join("agent-pids/42").exists());
        assert!(owner.owned.is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reprise_apres_crash_retire_orphelin_mais_preserve_pont_vivant_et_tiers() {
        let root = std::env::temp_dir().join(format!("bi101-{}", uuid::Uuid::new_v4()));
        let mut crashed = IdentityBindings::new(&root);
        // PID réutilisé : le propriétaire déclaré avait une autre naissance.
        // Aucun sous-processus ni signal réel n'est nécessaire pour cette preuve.
        crashed.owner = format!(
            "{}-{}-{}",
            std::process::id(),
            crashed.owner_birth - 1,
            uuid::Uuid::new_v4()
        );
        crashed
            .publish(&process(42, &["a"]), &binding("a"))
            .unwrap();
        let old_name = crashed.owned[&42].name_file.clone();
        let mut live = IdentityBindings::new(&root);
        live.publish(&process(43, &["b"]), &binding("b")).unwrap();
        let third_name = root.join("agent-names/other-wrapper");
        bridget_transport::fsutil::write_private_file_atomic(
            &third_name,
            binding("third").1.as_bytes(),
        )
        .unwrap();
        crate::mcp_identity::write_marker(&root.join("agent-pids"), 44, 10, "third", &third_name)
            .unwrap();
        let mut restarted = IdentityBindings::new(&root);
        restarted
            .refresh(&root, &ServerRuntime { pid: 1, port: 1 }, &[])
            .unwrap();
        assert!(!root.join("agent-pids/42").exists());
        assert!(!old_name.exists());
        assert!(root.join("agent-pids/43").exists());
        assert!(root.join("agent-pids/44").exists());
        restarted
            .publish(&process(42, &["a"]), &binding("a"))
            .unwrap();
        crashed.clear();
        assert!(
            root.join("agent-pids/42").exists(),
            "l'ancien propriétaire ne retire pas le nouveau marqueur"
        );
        restarted.clear();
        live.clear();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn inventaire_os_borne_conserve_tous_fichiers_ouverts_avec_espaces() {
        let root = std::env::temp_dir().join(format!("bi101-{}", uuid::Uuid::new_v4()));
        crate::environment::ensure_private_directory(&root).unwrap();
        let first = root.join("parent avec espaces.jsonl");
        let second = root.join("child.jsonl");
        let file1 = fs::File::create(&first).unwrap();
        let file2 = fs::File::create(&second).unwrap();
        let pid = std::process::id();
        assert!(process_ids().unwrap().contains(&pid));
        assert!(!process_arguments(pid).unwrap().is_empty());
        let started = std::time::Instant::now();
        let inventory = crate::runtime::open_session_files(&[pid], is_session_file).unwrap();
        let elapsed = started.elapsed();
        // /var est un alias de /private/var sur macOS ; lsof publie le canonique.
        assert!(inventory[&pid].contains(&first.canonicalize().unwrap()));
        assert!(inventory[&pid].contains(&second.canonicalize().unwrap()));
        assert!(elapsed < Duration::from_secs(3));
        eprintln!("inventaire identité local : {} ms", elapsed.as_millis());
        drop((file1, file2));
        fs::remove_dir_all(root).unwrap();
    }
}
