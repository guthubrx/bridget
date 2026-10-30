//! Découverte sans thread ni tour de modèle. Une seule sonde par worker,
//! exécutée hors de la file des missions, avec le bootstrap géré existant.
use crate::managed_process::{
    ManagedIdentity, ManagedLaunch, ManagedMarkerStore, ManagedStopResult, RunningManagedChild,
};
use crate::registry::AgentRegistry;
use bridget_transport::provider_catalogue::*;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufReader, Write};
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::{Duration, Instant};

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
    let home = source
        .get("CODEX_HOME")
        .filter(|_| def.pass_env.iter().any(|v| v == "CODEX_HOME"))
        .map(PathBuf::from)
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
    pub fn start(registry_path: PathBuf, state_root: PathBuf, machine: String) -> Self {
        let service = Self {
            generation: Arc::new(AtomicU64::new(0)),
            enabled: Arc::new(AtomicBool::new(false)),
            token: Arc::new(Mutex::new(String::new())),
            stopping: Arc::new(AtomicBool::new(false)),
            latest: Arc::new(Mutex::new(BTreeMap::new())),
        };
        let task = service.clone();
        std::thread::spawn(move || task.collect(registry_path, state_root, machine));
        service
    }
    pub fn activate(&self, token: &str) {
        let mut current = self.token.lock().unwrap_or_else(|e| e.into_inner());
        *current = token.into();
        self.generation.fetch_add(1, Ordering::AcqRel);
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
        *sequence = sequence.saturating_add(1);
        self.latest
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(value.connection_id.clone(), (generation, *sequence, value));
    }
    fn collect(&self, registry_path: PathBuf, state_root: PathBuf, machine: String) {
        let source = crate::lifecycle::source_environment();
        let salt = uuid::Uuid::new_v4().as_bytes().to_vec();
        let directory = state_root.join("catalogue");
        let markers = ManagedMarkerStore::at_directory(directory.join("markers"));
        let mut entries: BTreeMap<String, Entry> = BTreeMap::new();
        let mut retained: Option<RunningManagedChild> = None;
        let mut generation = 0;
        let mut sequence = 0u64;
        let mut recovered = false;
        let mut recovery_attempted = false;
        while !self.stopping.load(Ordering::Acquire) {
            std::thread::sleep(Duration::from_secs(1));
            if let Some(child) = retained.as_mut() {
                if child.reap_terminated_group().unwrap_or(false) {
                    retained = None;
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
                        if value.cause != Some(CatalogueCause::SourceUnavailable) {
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
                self.latest
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .clear();
                continue;
            }
            let keep = connections
                .iter()
                .map(|c| c.connection_id.as_str())
                .collect::<BTreeSet<_>>();
            self.latest
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .retain(|id, _| keep.contains(id.as_str()));
            if !recovered {
                recovered = private_dir(&directory)
                    .and_then(|_| private_dir(&directory.join("markers")))
                    .is_ok()
                    && recover_probe(&markers, !recovery_attempted);
                recovery_attempted = true;
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
                    Err((CatalogueCause::StopUnconfirmed, None))
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
                if config_stamp(def, &resolved.digest, &source) != stamp {
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

struct Rpc {
    reader: BufReader<UnixStream>,
    writer: UnixStream,
    id: u64,
    bytes: usize,
    deadline: Instant,
}
impl Rpc {
    fn write(&mut self, value: &Value) -> Result<(), CatalogueCause> {
        let mut bytes =
            serde_json::to_vec(value).map_err(|_| CatalogueCause::ProtocolUnavailable)?;
        bytes.push(b'\n');
        self.writer
            .set_write_timeout(Some(Duration::from_secs(1)))
            .map_err(|_| CatalogueCause::SourceUnavailable)?;
        self.writer
            .write_all(&bytes)
            .map_err(|_| CatalogueCause::SourceUnavailable)
    }
    fn call(&mut self, method: &str, params: Value) -> Result<Value, CatalogueCause> {
        // Liste fermée : aucun login, thread, tour, outil ou permission.
        if !matches!(method, "initialize" | "account/read" | "model/list") {
            return Err(CatalogueCause::ProtocolUnavailable);
        }
        self.id += 1;
        let id = self.id;
        self.write(&json!({"id":id,"method":method,"params":params}))?;
        loop {
            let line = bridget_transport::jsonl::read_unix_line(
                &mut self.reader,
                MAX_PAGE_BYTES,
                bridget_transport::jsonl::LineDeadline::Absolute(self.deadline),
            )
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::TimedOut {
                    CatalogueCause::TimedOut
                } else if e.kind() == std::io::ErrorKind::InvalidData {
                    CatalogueCause::OutputLimit
                } else {
                    CatalogueCause::ProtocolUnavailable
                }
            })?
            .ok_or(CatalogueCause::SourceUnavailable)?;
            self.bytes += line.len();
            if self.bytes > MAX_TOTAL_BYTES {
                return Err(CatalogueCause::OutputLimit);
            }
            let value: Value =
                serde_json::from_slice(&line).map_err(|_| CatalogueCause::ProtocolUnavailable)?;
            if value.get("id") == Some(&json!(id)) {
                return value
                    .get("result")
                    .cloned()
                    .ok_or(CatalogueCause::ProtocolUnavailable);
            }
            // Refuser toute demande du serveur, sans lui accorder d'autorité.
            if let Some(request_id) = value.get("id") {
                self.write(&json!({"id":request_id,"error":{"code":-32601,"message":"unsupported metadata request"}}))?;
            }
        }
    }
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
    let env = crate::lifecycle::build_environment(def, source)
        .map_err(|_| plain(CatalogueCause::SourceUnavailable))?;
    let socket = directory.join("c.sock");
    match std::fs::symlink_metadata(&socket) {
        Ok(m) if m.file_type().is_socket() && m.uid() == unsafe { libc::geteuid() } => {
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
    let mut bootstrap = crate::managed_process::tests::spawn_catalogue_test_bootstrap(&launch);
    bootstrap
        .set_ready_timeout(Duration::from_secs(2))
        .map_err(|_| plain(CatalogueCause::SourceUnavailable))?;
    let ready = bootstrap
        .wait_ready()
        .map_err(|_| plain(CatalogueCause::SourceUnavailable))?;
    let marked = ready
        .persist_marker(markers, "probe")
        .map_err(|_| plain(CatalogueCause::SourceUnavailable))?;
    let mut child = match marked.release_retained() {
        Ok(child) => child,
        Err((_, child)) => return Err((CatalogueCause::StopUnconfirmed, Some(child))),
    };
    let deadline = Instant::now() + PROBE_BUDGET;
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
        let mut rpc = Rpc {
            reader: BufReader::new(
                stream
                    .try_clone()
                    .map_err(|_| CatalogueCause::SourceUnavailable)?,
            ),
            writer: stream,
            id: 0,
            bytes: 0,
            deadline,
        };
        rpc.call(
            "initialize",
            json!({"clientInfo":{"name":"bridget","version":env!("CARGO_PKG_VERSION")}}),
        )?;
        rpc.write(&json!({"method":"initialized"}))?;
        let before = rpc.call("account/read", json!({"refreshToken":false}))?;
        let before_key = account_comparison(&before, salt);
        if before_key == Ok(None) {
            return Ok((Vec::new(), None));
        }
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
        let after = rpc.call("account/read", json!({"refreshToken":false}))?;
        if before_key != account_comparison(&after, salt) {
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
    if let Ok(m) = std::fs::symlink_metadata(&socket) {
        if m.file_type().is_socket() && m.uid() == unsafe { libc::geteuid() } {
            let _ = std::fs::remove_file(&socket);
        }
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
mod tests {
    use super::*;
    fn root() -> PathBuf {
        let root = std::env::temp_dir().join(format!("b220-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir(&root).unwrap();
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
        root
    }
    fn fixture(
        root: &Path,
        scenario: &str,
    ) -> (
        crate::registry::AgentDefinition,
        crate::lifecycle::SourceEnvironment,
    ) {
        let program = root.join("codex");
        std::fs::write(&program,r#"#!/usr/bin/python3
import socket,sys,json,os
path=sys.argv[sys.argv.index('--listen')+1].removeprefix('unix://')
server=socket.socket(socket.AF_UNIX);server.bind(path);server.listen(1)
conn,_=server.accept();reader=conn.makefile('rb');reads=0
for line in reader:
    v=json.loads(line);method=v.get('method');
    with open(os.environ['BRIDGET220_LOG'],'a') as f:f.write(str(method)+'\n')
    if 'id' not in v:continue
    scenario=os.environ['BRIDGET220_SCENARIO']
    if method=='initialize':result={}
    elif method=='account/read':
        reads+=1
        email='fake-before@example.invalid' if reads==1 or scenario!='changed' else 'fake-after@example.invalid'
        result={'account':None if scenario=='absent' else {'type':'chatgpt','email':email,'planType':'pro'},'requiresOpenaiAuth':True}
    elif method=='model/list':
        result={'data':[{'model':'model-'+str(n),'supportedReasoningEfforts':[{'reasoningEffort':'high'}],'defaultReasoningEffort':'high'} for n in range(257 if scenario=='large' else 1)],'nextCursor':None}
    else:raise Exception('forbidden method')
    conn.sendall((json.dumps({'id':v['id'],'result':result})+'\n').encode())
"#).unwrap();
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o700)).unwrap();
        let def=serde_json::from_value(json!({"command":program,"args":["app-server"],"protocol":"codex_app_server","pass_env":["BRIDGET220_LOG","BRIDGET220_SCENARIO"]})).unwrap();
        let mut source = crate::lifecycle::source_environment();
        source.insert("HOME".into(), root.as_os_str().to_owned());
        source.insert("BRIDGET220_LOG".into(), root.join("trace").into_os_string());
        source.insert("BRIDGET220_SCENARIO".into(), scenario.into());
        (def, source)
    }
    #[test]
    fn spec220_probe_real_bootstrap_reads_account_models_and_stops_without_turn() {
        let root = root();
        let directory = root.join("catalogue");
        private_dir(&directory).unwrap();
        private_dir(&directory.join("markers")).unwrap();
        let markers = ManagedMarkerStore::at_directory(directory.join("markers"));
        let (def, source) = fixture(&root, "present");
        let (models, account) = probe(
            &directory,
            &markers,
            &def,
            &source,
            &[7; 32],
            &AtomicBool::new(false),
        )
        .map_err(|e| e.0)
        .unwrap();
        assert_eq!(models[0].model, "model-0");
        assert!(account.is_some());
        let trace = std::fs::read_to_string(root.join("trace")).unwrap();
        assert_eq!(
            trace.lines().collect::<Vec<_>>(),
            [
                "initialize",
                "initialized",
                "account/read",
                "model/list",
                "model/list",
                "account/read"
            ]
        );
        assert!(recover_probe(&markers, false));
        assert!(!directory.join("c.sock").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn spec220_probe_refuses_changed_account_and_oversized_list_and_reports_signout() {
        let root = root();
        let directory = root.join("catalogue");
        private_dir(&directory).unwrap();
        private_dir(&directory.join("markers")).unwrap();
        let markers = ManagedMarkerStore::at_directory(directory.join("markers"));
        for (scenario, expected) in [
            ("changed", Some(CatalogueCause::AccountChanged)),
            ("large", Some(CatalogueCause::OutputLimit)),
            ("absent", None),
        ] {
            let (def, source) = fixture(&root, scenario);
            let value = probe(
                &directory,
                &markers,
                &def,
                &source,
                &[8; 32],
                &AtomicBool::new(false),
            );
            if let Some(expected) = expected {
                assert_eq!(value.err().map(|e| e.0), Some(expected));
            } else {
                assert_eq!(value.map_err(|e| e.0).unwrap(), (Vec::new(), None));
            }
            assert!(recover_probe(&markers, false));
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn spec220_rpc_timeout_and_unknown_method_are_bounded() {
        let (a, _b) = UnixStream::pair().unwrap();
        let mut rpc = Rpc {
            reader: BufReader::new(a.try_clone().unwrap()),
            writer: a,
            id: 0,
            bytes: 0,
            deadline: Instant::now() + Duration::from_millis(30),
        };
        assert_eq!(
            rpc.call("thread/start", json!({})),
            Err(CatalogueCause::ProtocolUnavailable)
        );
        assert_eq!(
            rpc.call("account/read", json!({})),
            Err(CatalogueCause::TimedOut)
        );
    }
    #[test]
    fn spec220_generation_fences_old_snapshots_and_private_paths_reject_links() {
        let service = CatalogueService {
            generation: Arc::new(AtomicU64::new(0)),
            token: Arc::new(Mutex::new(String::new())),
            enabled: Arc::new(AtomicBool::new(false)),
            stopping: Arc::new(AtomicBool::new(false)),
            latest: Arc::new(Mutex::new(BTreeMap::new())),
        };
        service.activate("first");
        let g = service.generation.load(Ordering::Acquire);
        let mut seq = 0;
        service.publish(
            g,
            &mut seq,
            observation("main", &"a".repeat(64), CatalogueState::Refreshing, None),
        );
        assert_eq!(service.snapshots().len(), 1);
        service.activate("second");
        service.publish(
            g,
            &mut seq,
            observation("main", &"a".repeat(64), CatalogueState::Refreshing, None),
        );
        assert!(service.snapshots().is_empty());
        let g = service.generation.load(Ordering::Acquire);
        service.publish(
            g,
            &mut seq,
            observation("main", &"a".repeat(64), CatalogueState::Refreshing, None),
        );
        assert_eq!(service.snapshots()[0].0, "second");
        service.deactivate();
        assert!(service.snapshots().is_empty());
        let root = root();
        let link = root.join("link");
        std::os::unix::fs::symlink(&root, &link).unwrap();
        assert!(private_dir(&link).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
