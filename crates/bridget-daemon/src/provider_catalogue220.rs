//! Découverte sans thread ni tour de modèle. Une seule sonde par worker,
//! exécutée hors de la file des missions, avec le bootstrap géré existant.
use crate::managed_process::{
    ManagedIdentity, ManagedLaunch, ManagedMarkerStore, ManagedStopResult, RunningManagedChild,
};
use crate::registry::AgentRegistry;
use bridget_transport::provider_catalogue::*;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Write};
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::{Duration, Instant};

pub(crate) const MODEL_OPTIONS_LAUNCH_ENABLED: bool = true;

const REFRESH: Duration = Duration::from_secs(120);
const RETRY: Duration = Duration::from_secs(30);
const MIN_REFRESH: Duration = Duration::from_secs(5);
const PROBE_BUDGET: Duration = Duration::from_secs(10);
const MAX_PAGE_BYTES: usize = 256 * 1024;
const MAX_TOTAL_BYTES: usize = 1024 * 1024;

#[derive(Clone)]
pub(crate) struct CatalogueService {
    generation: Arc<AtomicU64>,
    token: Arc<Mutex<String>>,
    enabled: Arc<AtomicBool>,
    stopping: Arc<AtomicBool>,
    latest: Arc<Mutex<BTreeMap<String, (u64, u64, ProviderCatalogue)>>>,
}
struct Entry {
    generation: u64,
    stamp: String,
    checked: Instant,
    success: bool,
    account_key: Option<String>,
    account_nonce: Option<String>,
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
fn private_dir(path: &Path) -> std::io::Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(m)
            if !m.file_type().is_symlink()
                && m.is_dir()
                && m.uid() == unsafe { libc::geteuid() }
                && m.mode() & 0o077 == 0 =>
        {
            Ok(())
        }
        Ok(_) => Err(std::io::Error::other("catalogue_private_path_required")),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir(path)?;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))
        }
        Err(e) => Err(e),
    }
}
fn file_stamp(path: &Path) -> String {
    match std::fs::metadata(path) {
        Ok(m) => format!(
            "{}:{}:{}:{}:{}",
            m.dev(),
            m.ino(),
            m.len(),
            m.mtime(),
            m.mtime_nsec()
        ),
        Err(e) => format!("{:?}", e.kind()),
    }
}
fn config_stamp(
    def: &crate::registry::AgentDefinition,
    digest: &str,
    source: &crate::lifecycle::SourceEnvironment,
) -> String {
    let home = def.codex_home.as_ref().map(PathBuf::from).or_else(|| source
        .get("CODEX_HOME")
        .filter(|_| def.pass_env.iter().any(|v| v == "CODEX_HOME"))
        .map(PathBuf::from))
        .unwrap_or_else(|| {
            PathBuf::from(source.get("HOME").cloned().unwrap_or_default()).join(".codex")
        });
    format!(
        "{digest}:{}:{}:{}",
        file_stamp(Path::new(&def.command)),
        file_stamp(&home.join("auth.json")),
        file_stamp(&home.join("config.toml"))
    )
}
fn observation(
    connection: &str,
    digest: &str,
    state: CatalogueState,
    cause: Option<CatalogueCause>,
) -> ProviderCatalogue {
    ProviderCatalogue {
        connection_id: connection.into(),
        definition_digest: digest.into(),
        observed_at: now(),
        state,
        account_scope: None,
        models: Vec::new(),
        cause,
    }
}
impl CatalogueService {
    pub fn start(registry_path: PathBuf, state_root: PathBuf, machine: String, source: crate::lifecycle::SourceEnvironment) -> Self {
        let service = Self {
            generation: Arc::new(AtomicU64::new(0)),
            enabled: Arc::new(AtomicBool::new(false)),
            token: Arc::new(Mutex::new(String::new())),
            stopping: Arc::new(AtomicBool::new(false)),
            latest: Arc::new(Mutex::new(BTreeMap::new())),
        };
        let task = service.clone();
        std::thread::spawn(move || task.collect(registry_path, state_root, machine, source));
        service
    }
    pub fn activate(&self, token: &str) {
        let mut current = self.token.lock().unwrap_or_else(|e| e.into_inner());
        *current = token.into();
        let generation = self.generation.fetch_add(1, Ordering::AcqRel) + 1;
        // A new transport token cannot attest that an owned process stopped.
        for (g, _, value) in self
            .latest
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .values_mut()
        {
            if value.cause == Some(CatalogueCause::StopUnconfirmed) {
                *g = generation;
            }
        }
        self.enabled.store(true, Ordering::Release);
    }
    pub fn deactivate(&self) {
        let _guard = self.token.lock().unwrap_or_else(|e| e.into_inner());
        self.enabled.store(false, Ordering::Release);
        self.generation.fetch_add(1, Ordering::AcqRel);
    }
    pub fn shutdown(&self) {
        self.stopping.store(true, Ordering::Release);
        self.deactivate();
    }
    pub fn snapshots(&self) -> Vec<(String, u64, ProviderCatalogue)> {
        let token = self.token.lock().unwrap_or_else(|e| e.into_inner());
        let current = self.generation.load(Ordering::Acquire);
        self.latest
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .filter(|(g, _, _)| *g == current)
            .map(|(_, seq, value)| (token.clone(), *seq, value.clone()))
            .collect()
    }
    fn publish(&self, generation: u64, sequence: &mut u64, value: ProviderCatalogue) {
        if self.generation.load(Ordering::Acquire) != generation
            || !self.enabled.load(Ordering::Acquire)
        {
            return;
        }
        let mut latest = self.latest.lock().unwrap_or_else(|e| e.into_inner());
        // Seule la récolte du groupe ou la récupération du marqueur constate
        // l'arrêt ; aucune nouvelle observation ne libère cette garde.
        if latest.get(&value.connection_id).is_some_and(|(_, _, old)|
            old.cause == Some(CatalogueCause::StopUnconfirmed))
            && value.cause != Some(CatalogueCause::StopUnconfirmed) {
            return;
        }
        // Les contraintes d'anciennes connexions consomment encore leur créneau.
        // Une nouvelle configuration ne peut pas augmenter ce cache sans borne.
        if !latest.contains_key(&value.connection_id) && latest.len() >= MAX_CATALOGUE_CONNECTIONS {
            return;
        }
        *sequence = sequence.saturating_add(1);
        latest.insert(value.connection_id.clone(), (generation, *sequence, value));
    }
    fn confirm_probe_stopped(&self) {
        self.latest.lock().unwrap_or_else(|e| e.into_inner())
            .retain(|_, (_, _, value)| value.cause != Some(CatalogueCause::StopUnconfirmed));
    }

    // Une suppression de configuration n'atteste jamais l'arrêt d'une sonde.
    // Le plafond historique reste applicable : la maintenance doit résoudre les
    // contraintes retenues avant de récupérer leurs créneaux.
    fn retain_connections(&self, keep: &BTreeSet<&str>) {
        self.latest.lock().unwrap_or_else(|e| e.into_inner())
            .retain(|id, (_, _, value)| keep.contains(id.as_str())
                || value.cause == Some(CatalogueCause::StopUnconfirmed));
    }

    fn collect(&self, registry_path: PathBuf, state_root: PathBuf, machine: String,
        source: crate::lifecycle::SourceEnvironment) {
        // Même source figée que les missions du worker, jamais un HOME global
        // relu plus tard depuis un thread indépendant.

        let salt = uuid::Uuid::new_v4().as_bytes().to_vec();
        let directory = state_root.join("catalogue");
        let markers = ManagedMarkerStore::at_directory(directory.join("markers"));
        let mut entries: BTreeMap<String, Entry> = BTreeMap::new();
        let mut retained: Option<RunningManagedChild> = None;
        let mut generation = 0;
        let mut sequence = 0u64;
        let mut recovered = false;
        let mut recovery_attempted = false;
        let mut recovery_cause = CatalogueCause::SourceUnavailable;
        while !self.stopping.load(Ordering::Acquire) {
            std::thread::sleep(Duration::from_secs(1));
            if let Some(child) = retained.as_mut() {
                if child.reap_terminated_group().unwrap_or(false) {
                    retained = None;
                    self.confirm_probe_stopped();
                } else {
                    continue;
                }
            }
            if !self.enabled.load(Ordering::Acquire) {
                continue;
            }
            let next = self.generation.load(Ordering::Acquire);
            if generation != next {
                generation = next;
            }
            let registry = match AgentRegistry::load_from_path_quiet(registry_path.clone()) {
                Ok(registry) => registry,
                Err(_) => {
                    let previous = self
                        .latest
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .values()
                        .map(|(_, _, v)| v.clone())
                        .collect::<Vec<_>>();
                    for value in previous {
                        if value.cause != Some(CatalogueCause::SourceUnavailable)
                            && value.cause != Some(CatalogueCause::StopUnconfirmed) {
                            self.publish(
                                generation,
                                &mut sequence,
                                observation(
                                    &value.connection_id,
                                    &value.definition_digest,
                                    CatalogueState::Unavailable,
                                    Some(CatalogueCause::SourceUnavailable),
                                ),
                            );
                        }
                    }
                    continue;
                }
            };
            let Some(policy) = registry.routing_policy() else {
                continue;
            };
            if !policy.enabled {
                continue;
            }
            let connections = policy
                .connections
                .iter()
                .filter(|c| c.machine_id == machine)
                .collect::<Vec<_>>();
            if connections.len() > MAX_CATALOGUE_CONNECTIONS {
                self.retain_connections(&BTreeSet::new());
                continue;
            }
            let keep = connections
                .iter()
                .map(|c| c.connection_id.as_str())
                .collect::<BTreeSet<_>>();
            self.retain_connections(&keep);
            if !recovered {
                let private = private_dir(&directory)
                    .and_then(|_| private_dir(&directory.join("markers")))
                    .is_ok();
                recovery_cause = if private {
                    CatalogueCause::StopUnconfirmed
                } else {
                    CatalogueCause::SourceUnavailable
                };
                if private {
                    recovered = recover_probe(&markers, !recovery_attempted);
                    recovery_attempted = true;
                    if recovered { self.confirm_probe_stopped(); }
                }
            }
            for connection in connections {
                if !self.enabled.load(Ordering::Acquire)
                    || self.generation.load(Ordering::Acquire) != generation
                {
                    break;
                }
                let Ok(def) = registry.get(&connection.agent_type) else {
                    continue;
                };
                let Ok(resolved) = registry.resolved_definition(&connection.agent_type) else {
                    continue;
                };
                if !connection.enabled
                    || connection.engine != "codex"
                    || def.protocol != "codex_app_server"
                {
                    let inactive_stamp = format!(
                        "{}:{}:{}",
                        resolved.digest, connection.enabled, connection.engine
                    );
                    if entries
                        .get(&connection.connection_id)
                        .is_none_or(|e| e.stamp != inactive_stamp || e.generation != generation)
                    {
                        let value = observation(
                            &connection.connection_id,
                            &resolved.digest,
                            if !connection.enabled {
                                CatalogueState::Disabled
                            } else {
                                CatalogueState::Unavailable
                            },
                            if connection.enabled {
                                Some(CatalogueCause::UnsupportedProvider)
                            } else {
                                None
                            },
                        );
                        self.publish(generation, &mut sequence, value);
                        entries.insert(
                            connection.connection_id.clone(),
                            Entry {
                                generation,
                                stamp: inactive_stamp,
                                checked: Instant::now(),
                                success: true,
                                account_key: None,
                                account_nonce: None,
                            },
                        );
                    }
                    continue;
                }
                let stamp = config_stamp(def, &resolved.digest, &source);
                let due = entries.get(&connection.connection_id).is_none_or(|e| {
                    e.checked.elapsed() >= if e.success { REFRESH } else { RETRY }
                        || (e.stamp != stamp || e.generation != generation)
                            && e.checked.elapsed() >= MIN_REFRESH
                });
                if !due {
                    continue;
                }
                self.publish(
                    generation,
                    &mut sequence,
                    observation(
                        &connection.connection_id,
                        &resolved.digest,
                        CatalogueState::Refreshing,
                        None,
                    ),
                );
                let outcome = if recovered {
                    probe(
                        &directory,
                        &markers,
                        def,
                        &source,
                        &salt,
                        self.stopping.as_ref(),
                    )
                } else {
                    Err((recovery_cause, None))
                };
                let mut value = observation(
                    &connection.connection_id,
                    &resolved.digest,
                    CatalogueState::Unavailable,
                    None,
                );
                let mut key = None;
                let mut nonce = None;
                match outcome {
                    Ok((models, account)) => {
                        if models.is_empty() && account.is_none() {
                            value.state = CatalogueState::SignInRequired;
                        } else {
                            value.state = CatalogueState::Observed;
                            value.models = models;
                            key = account;
                            if let Some(k) = &key {
                                nonce = entries
                                    .get(&connection.connection_id)
                                    .filter(|e| e.account_key.as_ref() == Some(k))
                                    .and_then(|e| e.account_nonce.clone())
                                    .or_else(|| Some(uuid::Uuid::new_v4().simple().to_string()));
                                value.account_scope = nonce.clone();
                            } else {
                                value.cause = Some(CatalogueCause::AccountUnidentified);
                            }
                        }
                    }
                    Err((cause, child)) => {
                        value.cause = Some(cause);
                        retained = child;
                    }
                }
                // Ni un changement de fichier pendant la sonde, ni un résultat
                // d'une ancienne connexion ne deviennent une observation neuve.
                if config_stamp(def, &resolved.digest, &source) != stamp
                    && value.cause != Some(CatalogueCause::StopUnconfirmed) {
                    value = observation(
                        &connection.connection_id,
                        &resolved.digest,
                        CatalogueState::Unavailable,
                        Some(CatalogueCause::DefinitionChanged),
                    );
                }
                let success = matches!(
                    value.state,
                    CatalogueState::Observed | CatalogueState::SignInRequired
                );
                self.publish(generation, &mut sequence, value);
                entries.insert(
                    connection.connection_id.clone(),
                    Entry {
                        generation,
                        stamp,
                        checked: Instant::now(),
                        success,
                        account_key: key,
                        account_nonce: nonce,
                    },
                );
                if retained.is_some() {
                    break;
                }
            }
        }
        if let Some(mut child) = retained {
            let _ = child.stop_group(
                Duration::ZERO,
                Duration::from_secs(1),
                Duration::from_millis(20),
            );
        }
    }
}

struct MetadataSocket {
    stream: UnixStream,
    deadline: Instant,
    bytes: usize,
}
impl MetadataSocket {
    fn remaining(&self) -> std::io::Result<Duration> {
        self.deadline
            .checked_duration_since(Instant::now())
            .filter(|d| !d.is_zero())
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::TimedOut, "metadata deadline"))
    }
}
impl Read for MetadataSocket {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if self.bytes >= MAX_TOTAL_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "metadata output limit",
            ));
        }
        self.stream.set_read_timeout(Some(self.remaining()?))?;
        let limit = buffer.len().min(MAX_TOTAL_BYTES - self.bytes);
        let n = self.stream.read(&mut buffer[..limit])?;
        self.bytes += n;
        Ok(n)
    }
}
impl Write for MetadataSocket {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.stream
            .set_write_timeout(Some(self.remaining()?.min(Duration::from_secs(1))))?;
        self.stream.write(buffer)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.remaining()?;
        self.stream.flush()
    }
}
fn ws_config() -> tungstenite::protocol::WebSocketConfig {
    tungstenite::protocol::WebSocketConfig::default()
        .read_buffer_size(8192)
        .write_buffer_size(0)
        .max_write_buffer_size(MAX_PAGE_BYTES)
        .max_frame_size(Some(MAX_PAGE_BYTES))
        .max_message_size(Some(MAX_PAGE_BYTES))
}
fn ws_failure(error: tungstenite::Error) -> CatalogueCause {
    match error {
        tungstenite::Error::Io(e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
            ) =>
        {
            CatalogueCause::TimedOut
        }
        tungstenite::Error::Capacity(_) => CatalogueCause::OutputLimit,
        tungstenite::Error::Io(e) if e.kind() == std::io::ErrorKind::InvalidData => {
            CatalogueCause::OutputLimit
        }
        tungstenite::Error::Io(_)
        | tungstenite::Error::ConnectionClosed
        | tungstenite::Error::AlreadyClosed => CatalogueCause::SourceUnavailable,
        _ => CatalogueCause::ProtocolUnavailable,
    }
}
struct Rpc {
    socket: tungstenite::WebSocket<MetadataSocket>,
    id: u64,
    bytes: usize,
}
impl Rpc {
    fn connect(stream: UnixStream, deadline: Instant) -> Result<Self, CatalogueCause> {
        // The Codex Unix listener upgrades to WebSocket; no TCP port or copied identity.
        let stream = MetadataSocket {
            stream,
            deadline,
            bytes: 0,
        };
        let (socket, _) =
            tungstenite::client::client_with_config("ws://localhost/", stream, Some(ws_config()))
                .map_err(|e| match e {
                tungstenite::HandshakeError::Failure(error) => ws_failure(error),
                _ => CatalogueCause::ProtocolUnavailable,
            })?;
        Ok(Self {
            socket,
            id: 0,
            bytes: 0,
        })
    }
    fn write(&mut self, value: &Value) -> Result<(), CatalogueCause> {
        let text = serde_json::to_string(value).map_err(|_| CatalogueCause::ProtocolUnavailable)?;
        self.socket
            .send(tungstenite::Message::Text(text.into()))
            .map_err(ws_failure)
    }
    fn call(&mut self, method: &str, params: Value) -> Result<Value, CatalogueCause> {
        if !matches!(method, "initialize" | "account/read" | "model/list") {
            return Err(CatalogueCause::ProtocolUnavailable);
        }
        self.id += 1;
        let id = self.id;
        #[cfg(test)]
        eprintln!("metadata220 rpc={method}");
        self.write(&json!({"id":id,"method":method,"params":params}))?;
        loop {
            let message = self.socket.read().map_err(ws_failure)?;
            let text = match message {
                tungstenite::Message::Text(text) => text,
                tungstenite::Message::Ping(_) | tungstenite::Message::Pong(_) => continue,
                tungstenite::Message::Close(_) => return Err(CatalogueCause::SourceUnavailable),
                _ => return Err(CatalogueCause::ProtocolUnavailable),
            };
            self.bytes += text.len();
            if self.bytes > MAX_TOTAL_BYTES {
                return Err(CatalogueCause::OutputLimit);
            }
            let value: Value =
                serde_json::from_str(&text).map_err(|_| CatalogueCause::ProtocolUnavailable)?;
            if value.get("id") == Some(&json!(id)) {
                return value
                    .get("result")
                    .cloned()
                    .ok_or(CatalogueCause::ProtocolUnavailable);
            }
            if let Some(request_id) = value.get("id") {
                self.write(&json!({"id":request_id,"error":{"code":-32601,"message":"unsupported metadata request"}}))?;
            }
        }
    }
}

/// Même propriétaire et même marqueur que la sonde métadonnée ; jamais deux
/// processus simultanés, ni libération du créneau après un arrêt incertain.
struct BoundedCheckStderr {
    stop: Arc<AtomicBool>,
    reader: std::thread::JoinHandle<Vec<u8>>,
}
impl BoundedCheckStderr {
    fn open() -> std::io::Result<(Self, std::process::Stdio)> {
        let (mut input, output) = UnixStream::pair()?;
        input.set_read_timeout(Some(Duration::from_millis(20)))?;
        let stop = Arc::new(AtomicBool::new(false));
        let done = stop.clone();
        let reader = std::thread::Builder::new().name("compat-stderr".into()).spawn(move || {
            let mut excerpt = Vec::with_capacity(512);
            let mut buffer = [0u8; 8192];
            loop {
                if done.load(Ordering::Acquire) && excerpt.len() == 512 { break; }
                match input.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        let keep = n.min(512 - excerpt.len());
                        excerpt.extend_from_slice(&buffer[..keep]);
                    }
                    Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {
                        if done.load(Ordering::Acquire) { break; }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => break,
                }
            }
            excerpt
        })?;
        use std::os::fd::OwnedFd;
        Ok((Self { stop, reader }, std::process::Stdio::from(OwnedFd::from(output))))
    }
    fn finish(self) -> Vec<u8> {
        self.stop.store(true, Ordering::Release);
        self.reader.join().unwrap_or_default()
    }
}

#[cfg(test)]
mod bounded_stderr244_tests {
    use super::*;
    #[test]
    fn spec244_stderr_bruyant_est_draine_sans_fichier_et_sans_croissance() {
        let (capture, stderr) = BoundedCheckStderr::open().unwrap();
        // Un processus réel écrit bien plus que la capacité du socket : la
        // capture doit drainer le surplus sans bloquer ce processus.
        assert!(std::process::Command::new("/usr/bin/head")
            .args(["-c", "1048576", "/dev/zero"]).stdout(stderr).status().unwrap().success());
        let excerpt = capture.finish();
        assert_eq!(excerpt.len(), 512);
        assert!(excerpt.iter().all(|byte| *byte == 0));
    }
    #[test]
    fn spec244_stderr_court_reste_lisible_apres_la_fin_du_processus() {
        let (capture, stderr) = BoundedCheckStderr::open().unwrap();
        std::process::Command::new("/usr/bin/printf").arg("cause courte")
            .stdout(stderr).status().unwrap();
        assert_eq!(capture.finish(), b"cause courte");
    }
}

fn check_program_schema(
    directory:&Path,markers:&ManagedMarkerStore,def:&crate::registry::AgentDefinition,
    source:&crate::lifecycle::SourceEnvironment,digest:&str,stopping:&AtomicBool,
)->Result<crate::provider_program_compat::CheckedContract,(CatalogueCause,Option<RunningManagedChild>)> {
    // E3 (244) : la sortie d'erreur du bootstrap est gardée, bornée, et
    // rattachée à une fiche d'échec ; un succès efface cette fiche.
    // Le surplus est drainé, jamais écrit sur disque ni gardé en mémoire.
    // Cela ne coupe pas le fournisseur et ne bloque pas son tube stderr.
    let (capture, stderr) = BoundedCheckStderr::open()
        .map_err(|_| (CatalogueCause::SourceUnavailable, None))?;
    let result=check_program_schema_inner(directory,markers,def,source,digest,stopping,stderr);
    let captured = capture.finish();
    match &result {
        Ok(_)=>crate::provider_program_compat::clear_check_failure(source,&def.command),
        Err((cause,_))=>{
            let cause=serde_json::to_value(cause).ok().and_then(|v|v.as_str().map(str::to_owned))
                .unwrap_or_else(||"unknown".into());
            crate::provider_program_compat::record_check_failure(source,&def.command,digest,&cause,&captured);
        }
    }
    result
}

fn check_program_schema_inner(
    directory:&Path,markers:&ManagedMarkerStore,def:&crate::registry::AgentDefinition,
    source:&crate::lifecycle::SourceEnvironment,digest:&str,stopping:&AtomicBool,
    stderr:std::process::Stdio,
)->Result<crate::provider_program_compat::CheckedContract,(CatalogueCause,Option<RunningManagedChild>)> {
    let plain=|cause|(cause,None);
    let job=directory.join(format!("compat-{}",uuid::Uuid::new_v4().simple()));
    std::fs::create_dir(&job).and_then(|_|std::fs::set_permissions(&job,std::fs::Permissions::from_mode(0o700)))
        .map_err(|_|plain(CatalogueCause::SourceUnavailable))?;
    let output=job.join("schemas");
    let mut env=std::collections::BTreeMap::new();
    env.insert("HOME".into(),job.as_os_str().to_os_string());
    env.insert("PATH".into(),source.get("PATH").cloned().unwrap_or_else(||"/usr/bin:/bin".into()));
    env.insert("LANG".into(),"C.UTF-8".into());
    env.insert("TMPDIR".into(),job.as_os_str().to_os_string());
    let launch=ManagedLaunch {bootstrap_executable:std::env::current_exe().map_err(|_|plain(CatalogueCause::ProgramUnavailable))?,
        identity:ManagedIdentity{instance_id:uuid::Uuid::new_v4().to_string(),command_id:"catalogue220".into(),generation:1},
        wrapper_executable:def.command.clone().into(),wrapper_args:vec!["app-server".into(),"generate-json-schema".into(),"--experimental".into(),"--out".into(),output.to_string_lossy().into_owned()],cwd:job.clone(),env};
    #[cfg(not(test))]
    let mut bootstrap=crate::managed_process::spawn_managed_bootstrap_with_stderr(&launch, stderr)
        .map_err(|_|plain(CatalogueCause::ProgramUnavailable))?;
    #[cfg(test)]
    let mut bootstrap=crate::managed_process::tests::spawn_catalogue_test_bootstrap_with_stderr(&launch,stderr);
    bootstrap.set_ready_timeout(Duration::from_secs(2)).map_err(|_|plain(CatalogueCause::SourceUnavailable))?;
    let ready=bootstrap.wait_ready().map_err(|_|plain(CatalogueCause::ProgramUnavailable))?;
    let marked=ready.persist_marker(markers,"probe").map_err(|_|plain(CatalogueCause::SourceUnavailable))?;
    let mut child=match marked.release_retained(){Ok(child)=>child,Err((_,child))=>return Err((CatalogueCause::StopUnconfirmed,Some(child)))};
    let deadline=Instant::now()+Duration::from_secs(5);
    let result=(|| {
        loop {
            if stopping.load(Ordering::Acquire)||Instant::now()>=deadline {return Err(CatalogueCause::TimedOut);}
            schema_output_bounded(&job)?;
            match child.try_wait().map_err(|_|CatalogueCause::SourceUnavailable)? {
                Some(status) if status.success()=>break,
                Some(_)=>return Err(CatalogueCause::ProtocolUnavailable),
                None=>std::thread::sleep(Duration::from_millis(25)),
            }
        }
        if crate::provider_program_compat::program_digest(Path::new(&def.command)).ok().as_deref()!=Some(digest){return Err(CatalogueCause::DefinitionChanged);}
        crate::provider_program_compat::CheckedContract::from_schemas(&output,&def.command,digest).map_err(|_|CatalogueCause::ProtocolUnavailable)
    })();
    if !matches!(child.stop_group(Duration::from_millis(250),Duration::from_secs(1),Duration::from_millis(20)),Ok(ManagedStopResult::Stopped|ManagedStopResult::StoppedForced{..})) {
        return Err((CatalogueCause::StopUnconfirmed,Some(child)));
    }
    // Racine exclusive créée ici, arrêt confirmé ; remove_dir_all ne suit pas
    // les liens internes. Aucun cache fournisseur ni répertoire de profil.
    if std::fs::symlink_metadata(&job).is_ok_and(|m|m.is_dir()&&!m.file_type().is_symlink()&&m.uid()==unsafe{libc::geteuid()}) {
        let _=std::fs::remove_dir_all(&job);
    }
    result.map_err(plain)
}
fn schema_output_bounded(root:&Path)->Result<(),CatalogueCause> {
    let mut todo=vec![(root.to_path_buf(),0)];let mut count=0usize;let mut bytes=0u64;
    while let Some((directory,depth))=todo.pop() {
        let entries=std::fs::read_dir(directory).map_err(|_|CatalogueCause::SourceUnavailable)?;
        for entry in entries {
            count+=1;if count>1024{return Err(CatalogueCause::OutputLimit);}
            let entry=entry.map_err(|_|CatalogueCause::SourceUnavailable)?;
            let m=match entry.path().symlink_metadata(){Ok(m)=>m,Err(e) if e.kind()==std::io::ErrorKind::NotFound=>continue,Err(_)=>return Err(CatalogueCause::SourceUnavailable)};
            if m.file_type().is_symlink(){return Err(CatalogueCause::SourceUnavailable);}
            if m.is_dir(){if depth>=8{return Err(CatalogueCause::OutputLimit);}todo.push((entry.path(),depth+1));}
            else if m.is_file(){bytes=bytes.saturating_add(m.len());if bytes>32*1024*1024{return Err(CatalogueCause::OutputLimit);}}
            else{return Err(CatalogueCause::SourceUnavailable);}
        }
    }Ok(())
}

fn probe(
    directory: &Path,
    markers: &ManagedMarkerStore,
    def: &crate::registry::AgentDefinition,
    source: &crate::lifecycle::SourceEnvironment,
    salt: &[u8],
    stopping: &AtomicBool,
) -> Result<(Vec<CatalogueModel>, Option<String>), (CatalogueCause, Option<RunningManagedChild>)> {
    let plain = |cause| (cause, None);
    let base = Path::new(&def.command)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if !matches!(base, "codex" | "codex-pro")
        || !def.args.iter().any(|a| a == "app-server")
        || def
            .args
            .iter()
            .any(|a| a == "--listen" || a.starts_with("--listen="))
    {
        return Err(plain(CatalogueCause::ProtocolUnavailable));
    }
    let automatic_contract=match crate::provider_program_compat::CheckedContract::needed(def,source) {
        Ok(Some(digest))=>Some(check_program_schema(directory,markers,def,source,&digest,stopping)?),
        Ok(None)=>None,
        Err(_)=>return Err(plain(CatalogueCause::ProtocolUnavailable)),
    };
    #[cfg(test)]
    eprintln!("metadata220 stage=environment");
    let env = crate::lifecycle::build_environment(def, source)
        .map_err(|_| plain(CatalogueCause::SourceUnavailable))?;
    let socket = directory.join("c.sock");
    match std::fs::symlink_metadata(&socket) {
        Ok(m)
            if (m.file_type().is_socket() || m.file_type().is_symlink())
                && m.uid() == unsafe { libc::geteuid() } =>
        {
            std::fs::remove_file(&socket).map_err(|_| plain(CatalogueCause::SourceUnavailable))?;
        }
        Ok(_) => return Err(plain(CatalogueCause::SourceUnavailable)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => return Err(plain(CatalogueCause::SourceUnavailable)),
    }
    bridget_transport::jsonl::validate_unix_socket_path(&socket)
        .map_err(|_| plain(CatalogueCause::SourceUnavailable))?;
    let mut args = def.args.clone();
    args.extend(["--listen".into(), format!("unix://{}", socket.display())]);
    let launch = ManagedLaunch {
        bootstrap_executable: std::env::current_exe()
            .map_err(|_| plain(CatalogueCause::ProgramUnavailable))?,
        identity: ManagedIdentity {
            instance_id: uuid::Uuid::new_v4().to_string(),
            command_id: "catalogue220".into(),
            generation: 1,
        },
        wrapper_executable: PathBuf::from(&def.command),
        wrapper_args: args,
        cwd: PathBuf::from(env.get("HOME").cloned().unwrap_or_default()),
        env,
    };
    #[cfg(not(test))]
    let mut bootstrap = crate::managed_process::spawn_managed_bootstrap(&launch)
        .map_err(|_| plain(CatalogueCause::ProgramUnavailable))?;
    #[cfg(test)]
    let mut bootstrap = if std::env::var_os("BRIDGET_REAL_CODEX_PROGRAM").is_some() {
        crate::managed_process::tests::spawn_catalogue_test_bootstrap_with_stderr(&launch,std::process::Stdio::inherit())
    } else {crate::managed_process::tests::spawn_catalogue_test_bootstrap(&launch)};
    bootstrap
        .set_ready_timeout(Duration::from_secs(2))
        .map_err(|_| plain(CatalogueCause::SourceUnavailable))?;
    #[cfg(test)]
    eprintln!("metadata220 stage=ready");
    let ready = bootstrap
        .wait_ready()
        .map_err(|_| plain(CatalogueCause::SourceUnavailable))?;
    #[cfg(test)]
    eprintln!("metadata220 stage=marker");
    let marked = ready
        .persist_marker(markers, "probe")
        .map_err(|_| plain(CatalogueCause::SourceUnavailable))?;
    #[cfg(test)]
    eprintln!("metadata220 stage=release");
    let mut child = match marked.release_retained() {
        Ok(child) => child,
        Err((_, child)) => return Err((CatalogueCause::StopUnconfirmed, Some(child))),
    };
    #[cfg(test)]
    eprintln!("metadata220 stage=connecting");
    let deadline = Instant::now() + PROBE_BUDGET;
    let mut contract_initialize=None;
    let result = (|| {
        let stream = loop {
            if stopping.load(Ordering::Acquire) || Instant::now() >= deadline {
                return Err(CatalogueCause::TimedOut);
            }
            if child
                .try_wait()
                .map_err(|_| CatalogueCause::SourceUnavailable)?
                .is_some()
            {
                return Err(CatalogueCause::ProgramUnavailable);
            }
            match bridget_transport::jsonl::connect_nonblocking(&socket, deadline) {
                Ok(stream) => break stream,
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
                    ) =>
                {
                    std::thread::sleep(Duration::from_millis(25))
                }
                Err(_) => return Err(CatalogueCause::SourceUnavailable),
            }
        };
        #[cfg(test)]
        eprintln!("metadata220 stage=connected");
        if crate::daemon::unix_peer_uid(&stream).ok() != Some(unsafe { libc::geteuid() }) {
            return Err(CatalogueCause::SourceUnavailable);
        }
        let mut rpc = Rpc::connect(stream, deadline)?;
        let initialize=rpc.call(
            "initialize",
            bridget_transport::codex_app_server::initialization_parameters(),
        )?;
        rpc.write(&json!({"method":"initialized"}))?;
        if automatic_contract.is_some() {contract_initialize=Some(initialize);}
        #[cfg(test)]
        eprintln!("metadata220 stage=account_before");
        let before = rpc.call("account/read", json!({"refreshToken":false}))?;
        let before_key = account_comparison(&before, salt);
        if before_key == Ok(None) {
            return Ok((Vec::new(), None));
        }
        #[allow(clippy::unnecessary_unwrap)] // le unwrap_err suit la vérification is_err()
        if before_key.is_err() && before_key != Err(CatalogueCause::AccountUnidentified) {
            return Err(before_key.unwrap_err());
        }
        // Un premier appel peut répondre depuis le cache de Codex. Le processus
        // neuf dispose d’une fenêtre bornée pour son actualisation asynchrone.
        rpc.call("model/list", json!({"limit":100,"includeHidden":false}))?;
        let settle = Instant::now() + Duration::from_secs(3);
        while Instant::now() < settle {
            if stopping.load(Ordering::Acquire) || Instant::now() >= deadline {
                return Err(CatalogueCause::TimedOut);
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        #[cfg(test)]
        eprintln!("metadata220 stage=models");
        let mut models = Vec::new();
        let mut cursor = None;
        let mut cursors = BTreeSet::new();
        for _ in 0..4 {
            let page = rpc.call(
                "model/list",
                json!({"cursor":cursor,"limit":100,"includeHidden":false}),
            )?;
            let (mut values, next) = model_page(&page)?;
            models.append(&mut values);
            if models.len() > MAX_CATALOGUE_MODELS {
                return Err(CatalogueCause::OutputLimit);
            }
            if let Some(next) = next {
                if !cursors.insert(next.clone()) {
                    return Err(CatalogueCause::ProtocolUnavailable);
                }
                cursor = Some(next);
            } else {
                cursor = None;
                break;
            }
        }
        if serde_json::to_vec(&models).map_or(true, |v| v.len() > MAX_CATALOGUE_BYTES - 1024) {
            return Err(CatalogueCause::OutputLimit);
        }
        if cursor.is_some() {
            return Err(CatalogueCause::OutputLimit);
        }
        if models.is_empty() {
            return Err(CatalogueCause::ProtocolUnavailable);
        }
        models.sort_by(|a, b| a.model.cmp(&b.model));
        if models.windows(2).any(|p| p[0].model == p[1].model) {
            return Err(CatalogueCause::ProtocolUnavailable);
        }
        #[cfg(test)]
        eprintln!("metadata220 stage=account_after");
        let after = rpc.call("account/read", json!({"refreshToken":false}))?;
        if before.pointer("/account/type") != after.pointer("/account/type")
            || before_key != account_comparison(&after, salt)
        {
            return Err(CatalogueCause::AccountChanged);
        }
        Ok((models, before_key.ok().flatten()))
    })();
    // Une réponse complète sans arrêt confirmé reste un échec. Le propriétaire
    // du Child et son marqueur restent détenus ; aucune nouvelle sonde ne part.
    let stopped = child.stop_group(
        Duration::from_millis(250),
        Duration::from_secs(1),
        Duration::from_millis(20),
    );
    if !matches!(
        stopped,
        Ok(ManagedStopResult::Stopped | ManagedStopResult::StoppedForced { .. })
    ) {
        return Err((CatalogueCause::StopUnconfirmed, Some(child)));
    }
    if let Ok(m) = std::fs::symlink_metadata(&socket)
        && (m.file_type().is_socket() || m.file_type().is_symlink())
            && m.uid() == unsafe { libc::geteuid() }
        {
            let _ = std::fs::remove_file(&socket);
        }
    if let (Some(contract),Some(initialize))=(automatic_contract,contract_initialize) {
        let compatible=contract.is_compatible();
        contract.publish(source,&initialize).map_err(|code|plain(match code {"program_changed_during_check"=>CatalogueCause::DefinitionChanged,"program_protocol_incompatible"=>CatalogueCause::ProtocolUnavailable,_=>CatalogueCause::SourceUnavailable}))?;
        if !compatible {return Err(plain(CatalogueCause::ProtocolUnavailable));}
    }
    result.map_err(plain)
}

fn recover_probe(markers: &ManagedMarkerStore, signal_once: bool) -> bool {
    let marker = match markers.load("probe") {
        Ok(value) => value,
        Err(crate::managed_process::ManagedProcessError::Io(e))
            if e.kind() == std::io::ErrorKind::NotFound =>
        {
            return true;
        }
        Err(_) => return false,
    };
    if matches!(crate::managed_process::group_exists(marker.pgid), Ok(false)) {
        return markers.remove("probe").is_ok();
    }
    if crate::managed_process::process_birth(marker.pgid).ok() != Some(marker.birth) {
        return false;
    }
    if signal_once {
        let _ = markers.stop_current_group(
            "probe",
            Duration::from_millis(500),
            Duration::from_millis(20),
        );
    }
    if matches!(crate::managed_process::group_exists(marker.pgid), Ok(false)) {
        markers.remove("probe").is_ok()
    } else {
        false
    }
}

pub(crate) struct ServiceGuard(pub CatalogueService);
impl Drop for ServiceGuard {
    fn drop(&mut self) {
        self.0.shutdown();
    }
}

#[cfg(test)]
#[path = "provider_catalogue220_tests.rs"]
mod tests;

#[cfg(test)]
mod private_recipe220 {
    use super::*;
    #[test]
    #[ignore = "requires an explicitly authorized local configured Codex metadata read"]
    fn spec220_private_real_codex_metadata_without_thread_or_turn() {
        let path = PathBuf::from(
            std::env::var_os("BRIDGET220_REAL_REGISTRY").expect("private registry required"),
        );
        let machine = std::env::var("BRIDGET220_REAL_MACHINE").expect("machine required");
        let output = PathBuf::from(
            std::env::var_os("BRIDGET220_REAL_OUTPUT").expect("private output required"),
        );
        let registry = AgentRegistry::load_from_path_quiet(path).unwrap();
        let connection = registry
            .routing_policy()
            .unwrap()
            .connections
            .iter()
            .find(|c| c.machine_id == machine && c.engine == "codex" && c.enabled)
            .expect("enabled Codex connection required");
        let definition = registry.get(&connection.agent_type).unwrap();
        let root =
            std::env::temp_dir().join(format!("b220-real-{}", uuid::Uuid::new_v4().simple()));
        private_dir(&root).unwrap();
        private_dir(&root.join("markers")).unwrap();
        let markers = ManagedMarkerStore::at_directory(root.join("markers"));
        let (models, account) = probe(
            &root,
            &markers,
            definition,
            &crate::lifecycle::source_environment(),
            &[9; 32],
            &AtomicBool::new(false),
        )
        .map_err(|e| e.0)
        .unwrap();
        assert!(!models.is_empty());
        assert!(recover_probe(&markers, false));
        let value = json!({"machine_id":machine,"connection_id":connection.connection_id,"model_count":models.len(),"models":models,"account_identity_observable":account.is_some(),"scope":"advertised_by_program","entitlement":"unknown","provider_cache":"not_attested","probe_stopped":true,"observed_at":now(),"thread_or_turn":false});
        std::fs::write(&output, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
        std::fs::set_permissions(&output, std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(test)]
mod program_schema221_tests {
    use super::*;
    /// Génération et initialize réels, HOME neuf sans auth, aucun thread/tour.
    #[test]
    #[ignore = "programme Codex réel fourni explicitement, aucune inférence"]
    fn spec221_program_compat_real_schema_metadata_without_model() {
        let command=std::env::var("BRIDGET_REAL_CODEX_PROGRAM").expect("programme explicite");
        let root=PathBuf::from(std::env::var_os("HOME").unwrap()).join(".cache").join(format!("bpr-{}",&uuid::Uuid::new_v4().simple().to_string()[..8]));
        private_dir(&root).unwrap();private_dir(&root.join(".cache")).unwrap();
        let directory=root.join("catalogue");private_dir(&directory).unwrap();private_dir(&directory.join("markers")).unwrap();
        let registry=crate::registry::AgentRegistry::from_json(&json!({"agents":{"fixture":{"command":command,"protocol":"codex_app_server","args":["app-server"],"capabilities":{"execution_paths":["codex_app_server"],"observed":{"binary_path":command,"binary_version":"historical-fixture","binary_digest":"0".repeat(64),"contract_version":"old-fixture","operations":["interrupt","steer"]}}}}}).to_string(),"/tmp/real-program-compat.json").unwrap();
        let def=registry.get("fixture").unwrap();let original=def.clone();
        let source=crate::lifecycle::SourceEnvironment::from([("HOME".into(),root.as_os_str().to_os_string()),("PATH".into(),std::env::var_os("PATH").unwrap())]);
        let markers=ManagedMarkerStore::at_directory(directory.join("markers"));
        let result=probe(&directory,&markers,def,&source,b"synthetic-account-salt",&AtomicBool::new(false));
        assert!(result.is_ok(),"sonde réelle : {:?}",result.err().map(|(cause,_)|cause));
        let digest=crate::provider_program_compat::program_digest(Path::new(&command)).unwrap();
        assert_eq!(crate::provider_program_compat::validate_upgrade(&command,&source,&digest),Ok(()));
        let runtime=crate::provider_program_compat::runtime_observation(def,&source).unwrap().unwrap();
        assert_eq!(runtime.binary_digest,digest);assert!(runtime.operations.contains(&bridget_transport::protocol::ProviderOperation::Steer));
        assert_eq!(*def,original);assert!(markers.load("probe").is_err());
        std::fs::remove_dir_all(root).unwrap();
    }    #[test]
    fn spec221_program_compat_uncertain_stop_keeps_child_and_marker() {
        let root=std::env::temp_dir().join(format!("bpc-stop-{}",&uuid::Uuid::new_v4().simple().to_string()[..8]));private_dir(&root).unwrap();
        let directory=root.join("catalogue");private_dir(&directory).unwrap();private_dir(&directory.join("markers")).unwrap();
        let command=root.join("codex");let release=root.join("allow-exit");
        std::fs::write(&command,format!("#!/bin/sh\ntrap '' TERM\nwhile [ ! -f '{}' ]; do /bin/sleep 0.05; done\n",release.display())).unwrap();std::fs::set_permissions(&command,std::fs::Permissions::from_mode(0o700)).unwrap();
        let registry=crate::registry::AgentRegistry::from_json(&json!({"agents":{"fixture":{"command":command,"protocol":"codex_app_server","args":["app-server"],"capabilities":{"execution_paths":["codex_app_server"]}}}}).to_string(),"/tmp/retained-compat.json").unwrap();
        let source=crate::lifecycle::SourceEnvironment::from([("HOME".into(),root.as_os_str().to_os_string())]);
        let markers=ManagedMarkerStore::at_directory(directory.join("markers"));let digest=crate::provider_program_compat::program_digest(&command).unwrap();
        let outcome=check_program_schema(&directory,&markers,registry.get("fixture").unwrap(),&source,&digest,&AtomicBool::new(false));
        let (cause,child)=outcome.err().expect("sonde retenue");assert_eq!(cause,CatalogueCause::StopUnconfirmed);let mut child=child.expect("propriétaire conservé");
        assert!(markers.load("probe").is_ok());assert_eq!(crate::provider_program_compat::validate_upgrade(command.to_str().unwrap(),&source,&digest),Err("program_upgrade_check_pending"));
        std::fs::write(release,b"exit").unwrap();
        assert!(matches!(child.stop_group(Duration::from_millis(500),Duration::from_secs(2),Duration::from_millis(20)),Ok(ManagedStopResult::Stopped|ManagedStopResult::StoppedForced{..})));
        assert!(markers.load("probe").is_err());std::fs::remove_dir_all(root).unwrap();
    }

}
