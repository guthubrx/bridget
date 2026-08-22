//! Module wrapper — lance un agent CLI et le connecte au daemon bridget.
//!
//! Appelé par le CLI quand l'utilisateur tape : bridget codex, bridget claude, etc.

use bridget_transport::journal::{
    IncrementalJournalReader, JournalReadItem, JournalSourceIdentity, JournalWindowError,
    current_host_date, resolve_window,
};
use bridget_transport::protocol::{decode, encode};
use bridget_transport::{
    AcpEvent, AcpOptions, AcpTransport, AttachRefusal, AttachWindow, DaemonToWrapper,
    MAX_ATTACH_FRAGMENT_BYTES, MAX_ATTACH_SERIALIZED_FRAME_BYTES, TmuxTransport, Transport,
    WrapperToDaemon,
};
use log::{debug, error, info, warn};
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::receipt_store::{ReceiptDecision, ReceiptQuota, ReceiptStore};

// Constantes de reconnexion optimisées pour auto-reconnect transparent
const RECONNECT_INITIAL_DELAY: Duration = Duration::from_secs(1);
const RECONNECT_MAX_DELAY: Duration = Duration::from_secs(30);
const RECONNECT_STABLE_RESET: Duration = Duration::from_secs(60);

// HEARTBEAT amélioré : plus fréquent pour une détection rapide (3s au lieu de 15s)
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(3);

// SOCKET_CHECK_INTERVAL : vérification proactive de disponibilité du socket
const SOCKET_CHECK_INTERVAL: Duration = Duration::from_secs(5);

// Sonde de runtime : le fichier de session n'est relu que si sa date de
// modification a changé, et le chemin n'est re-résolu que rarement — `lsof`
// coûte environ 130 ms, contre quelques microsecondes pour un `stat`.
const RUNTIME_PROBE_INTERVAL: Duration = Duration::from_secs(20);
// Un `codex resume` ouvre un second rollout sans fermer le premier : garder le
// chemin trop longtemps rendrait la nouvelle session invisible d'autant.
// 60 s aligne cette latence sur l'exigence de fraîcheur de la spec, pour un
// surcoût de 134 ms par minute.
const RUNTIME_PATH_REFRESH: Duration = Duration::from_secs(60);
const ATTACH_RELAY_COMMAND_CAPACITY: usize = 8;
const ATTACH_RELAY_READ_BYTES: usize = 128 * 1024;
const ATTACH_RELAY_IDLE_WAIT: Duration = Duration::from_millis(10);

#[derive(Debug, Clone)]
struct PendingIdempotentDelivery {
    delivery_id: String,
    delivery_generation: u64,
}

/// Raccorde l'observable ACP au reçu durable : aucun accusé n'est émis avant
/// `PromptDispatched`, et tout état ambigu reste explicitement indéterminé.
struct IdempotentDeliveryTracker {
    instance_id: String,
    receipts: ReceiptStore,
    pending: BTreeMap<String, VecDeque<PendingIdempotentDelivery>>,
}

enum IdempotentDeliveryAction {
    Inject {
        message: bridget_core::BridgetMessage,
        delivery_id: String,
    },
    Report(WrapperToDaemon),
}

impl IdempotentDeliveryTracker {
    fn open(home: &std::path::Path, instance_id: &str) -> Result<Self, String> {
        let state_home = std::env::var_os("XDG_STATE_HOME")
            .filter(|path| !path.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/state"));
        Self::open_at(&state_home, instance_id)
    }

    fn open_at(state_home: &std::path::Path, instance_id: &str) -> Result<Self, String> {
        let receipts = ReceiptStore::open(state_home, instance_id, ReceiptQuota::default())
            .map_err(|error| format!("ouverture des reçus idempotents: {error}"))?;
        Ok(Self {
            instance_id: instance_id.to_string(),
            receipts,
            pending: BTreeMap::new(),
        })
    }

    fn receive(
        &mut self,
        delivery_id: String,
        recipient_instance_id: String,
        delivery_generation: u64,
        expires_at: i64,
        message: bridget_core::BridgetMessage,
        now: i64,
    ) -> IdempotentDeliveryAction {
        let indeterminate = || WrapperToDaemon::DeliveryIndeterminate {
            delivery_id: delivery_id.clone(),
            delivery_generation,
        };
        let acked = || WrapperToDaemon::DeliverAcked {
            delivery_id: delivery_id.clone(),
            delivery_generation,
        };
        if recipient_instance_id != self.instance_id {
            return IdempotentDeliveryAction::Report(indeterminate());
        }
        match self.receipts.receive(&delivery_id, expires_at, now) {
            Ok(ReceiptDecision::Inject) => {
                let message_id = message.id.clone();
                let pending_delivery_id = delivery_id.clone();
                self.pending
                    .entry(message_id)
                    .or_default()
                    .push_back(PendingIdempotentDelivery {
                        delivery_id,
                        delivery_generation,
                    });
                IdempotentDeliveryAction::Inject {
                    message,
                    delivery_id: pending_delivery_id,
                }
            }
            Ok(ReceiptDecision::Acked) => IdempotentDeliveryAction::Report(acked()),
            Ok(ReceiptDecision::Indeterminate | ReceiptDecision::RejectedQuota) | Err(_) => {
                IdempotentDeliveryAction::Report(indeterminate())
            }
        }
    }

    fn prompt_dispatched(&mut self, message_id: &str, now: i64) -> Option<WrapperToDaemon> {
        let pending = self.take_pending_by_message(message_id)?;
        let acknowledged = matches!(
            self.receipts.acknowledge(&pending.delivery_id, now),
            Ok(ReceiptDecision::Acked)
        );
        Some(if acknowledged {
            WrapperToDaemon::DeliverAcked {
                delivery_id: pending.delivery_id,
                delivery_generation: pending.delivery_generation,
            }
        } else {
            WrapperToDaemon::DeliveryIndeterminate {
                delivery_id: pending.delivery_id,
                delivery_generation: pending.delivery_generation,
            }
        })
    }

    fn injection_failed(&mut self, delivery_id: &str) -> Option<WrapperToDaemon> {
        let message_id = self.pending.iter().find_map(|(message_id, deliveries)| {
            deliveries
                .iter()
                .any(|pending| pending.delivery_id == delivery_id)
                .then(|| message_id.clone())
        })?;
        let pending = {
            let deliveries = self.pending.get_mut(&message_id)?;
            let index = deliveries
                .iter()
                .position(|pending| pending.delivery_id == delivery_id)?;
            deliveries.remove(index)?
        };
        if self
            .pending
            .get(&message_id)
            .is_some_and(VecDeque::is_empty)
        {
            self.pending.remove(&message_id);
        }
        Some(WrapperToDaemon::DeliveryIndeterminate {
            delivery_id: pending.delivery_id,
            delivery_generation: pending.delivery_generation,
        })
    }

    fn injection_rejected(&mut self, message_id: &str) -> Option<WrapperToDaemon> {
        let pending = self.take_pending_by_message(message_id)?;
        Some(WrapperToDaemon::DeliveryIndeterminate {
            delivery_id: pending.delivery_id,
            delivery_generation: pending.delivery_generation,
        })
    }

    fn take_pending_by_message(&mut self, message_id: &str) -> Option<PendingIdempotentDelivery> {
        let pending = self.pending.get_mut(message_id)?.pop_front()?;
        if self.pending.get(message_id).is_some_and(VecDeque::is_empty) {
            self.pending.remove(message_id);
        }
        Some(pending)
    }
}

fn socket_path() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join(".cache")
            .join("bridget")
            .join("bridget.sock")
    } else {
        PathBuf::from("/tmp").join("bridget.sock")
    }
}

fn unix_now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .min(i64::MAX as u64) as i64
}

fn host_name() -> String {
    if let Ok(host) = std::env::var("HOSTNAME")
        && !host.trim().is_empty()
    {
        return host;
    }
    Command::new("hostname")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|host| host.trim().to_string())
        .filter(|host| !host.is_empty())
        .unwrap_or_else(|| "inconnu".to_string())
}

fn transport_name() -> String {
    if let Ok(transport) = std::env::var("BRIDGET_TRANSPORT")
        && !transport.trim().is_empty()
    {
        return transport;
    }
    let config_path = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
        .join(".config/bridget/federation.env");
    std::fs::read_to_string(config_path)
        .ok()
        .and_then(|config| {
            config
                .lines()
                .find_map(|line| line.strip_prefix("transport=").map(str::to_owned))
        })
        .filter(|transport| !transport.trim().is_empty())
        .unwrap_or_else(|| "unix".to_string())
}

/// Domaine de travail dérivé du répertoire courant.
///
/// La racine du dépôt git donne le regroupement le plus naturel : deux agents
/// lancés n'importe où dans le même projet partagent un domaine. Hors dépôt, le
/// nom du répertoire courant fait office de domaine.
///
/// Le nom est rendu brut, sans embellissement : un répertoire
/// `projet-b` donne le domaine `projet-b`. Une règle
/// de nettoyage implicite serait indevinable pour l'utilisateur, qui peut de
/// toute façon surcharger avec `bridget domain`.
fn derive_domain() -> Option<String> {
    let git_root = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|path| path.trim().to_string())
        .filter(|path| !path.is_empty())
        .map(PathBuf::from);

    let base = match git_root {
        Some(root) => root,
        None => std::env::current_dir().ok()?,
    };
    base.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
}

/// Chemin du domaine surchargé d'un agent, en miroir de `agent-names/`.
fn domain_state_path(agent: &str) -> PathBuf {
    socket_path()
        .parent()
        .unwrap()
        .join("agent-domains")
        .join(agent)
}

/// Domaine effectif d'un agent : la surcharge si elle existe, le dérivé sinon.
///
/// Relu à chaque enregistrement, y compris après une reconnexion, pour la même
/// raison que le nom : seule la trace sur disque connaît l'intention de
/// l'utilisateur.
fn effective_domain(agent: &str) -> Option<String> {
    std::fs::read_to_string(domain_state_path(agent))
        .map(|domain| domain.trim().to_string())
        .ok()
        .filter(|domain| !domain.is_empty())
        .or_else(derive_domain)
}

/// Nom d'OS stable et lisible pour l'annuaire Bridget.
fn operating_system() -> String {
    match std::env::consts::OS {
        "macos" => "macOS".to_string(),
        "linux" => "Linux".to_string(),
        os => os.to_string(),
    }
}

fn get_current_pane_id() -> Result<String, String> {
    let output = Command::new("tmux")
        .args(["display-message", "-p", "#{pane_id}"])
        .output()
        .map_err(|e| format!("tmux exec: {}", e))?;
    if !output.status.success() {
        return Err(format!("tmux: {}", String::from_utf8_lossy(&output.stderr)));
    }
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if id.is_empty() {
        Err("pane vide".into())
    } else {
        Ok(id)
    }
}

/// Calcule un hash des args pour identifier une session (resume, etc.).
fn session_hash(agent_args: &[String]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    let mut found_session = false;
    for arg in agent_args {
        // Un vrai session-id de Codex ressemble à un UUID :
        // ex: 019ff375-1f42-7552-b87c-481a5ac14faa (36 chars, 4 tirets)
        if arg.len() == 36 && arg.matches('-').count() == 4 {
            arg.hash(&mut hasher);
            found_session = true;
        }
    }
    if found_session {
        format!("{:016x}", hasher.finish())
    } else {
        // Pas de session-id → pas de persistance → auto-incrément normal
        String::new()
    }
}

/// Charge le nom persistant pour cette session.
fn load_persistent_name(_agent_type: &str, agent_args: &[String]) -> Option<String> {
    let hash = session_hash(agent_args);
    if hash.is_empty() {
        return None; // Pas de session-id → auto-incrément normal
    }
    let name_file = persistent_name_path(&hash);
    if name_file.exists() {
        let name = std::fs::read_to_string(&name_file).ok()?;
        let name = name.trim().to_string();
        if !name.is_empty() {
            eprintln!("[bridget] Nom retrouvé: « {} »", name);
            return Some(name);
        }
    }
    None
}

/// Sauvegarde le nom pour les futurs resume.
fn save_persistent_name(_agent_type: &str, agent_args: &[String], name: &str) {
    let hash = session_hash(agent_args);
    if hash.is_empty() {
        return; // Pas de session-id, rien à sauver
    }
    let name_file = persistent_name_path(&hash);
    if let Some(parent) = name_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&name_file, name);
}

fn persistent_name_path(hash: &str) -> std::path::PathBuf {
    socket_path()
        .parent()
        .unwrap()
        .join("agent-names")
        .join(hash)
}

/// Met un file descriptor en mode close-on-exec (FD_CLOEXEC).
fn set_cloexec(stream: &UnixStream) {
    use std::os::unix::io::AsRawFd;
    let fd = stream.as_raw_fd();
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFD);
        if flags >= 0 {
            libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC);
        }
    }
}

/// Délai exponentiel (1, 2, 4, 8, 16, 30 s) avec un jitter de ±20 %.
/// Le plafond protège le daemon et le serveur SSH pendant une panne longue.
fn reconnect_delay(attempt: u32) -> Duration {
    let multiplier = 1_u64 << attempt.min(5);
    let base_ms = (RECONNECT_INITIAL_DELAY.as_millis() as u64)
        .saturating_mul(multiplier)
        .min(RECONNECT_MAX_DELAY.as_millis() as u64);
    let jitter_span = base_ms / 5;
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64
        ^ std::process::id() as u64
        ^ attempt as u64;
    let jitter = (seed % (jitter_span.saturating_mul(2) + 1)) as i64 - jitter_span as i64;
    Duration::from_millis((base_ms as i64 + jitter).max(1) as u64).min(RECONNECT_MAX_DELAY)
}

/// Nom courant de l'agent, tel que `bridget rename` l'a laissé sur disque.
///
/// Le wrapper ne peut pas se fier au nom qu'il a obtenu à son enregistrement :
/// l'agent a pu être renommé depuis, et seul ce fichier en garde la trace. S'y
/// référer à chaque reconnexion évite qu'un agent renommé ne réapparaisse sous
/// son nom d'origine après une coupure — ce qui se produit à chaque rupture de
/// tunnel dans une installation fédérée.
fn resolve_current_name(name_state_path: &std::path::Path, fallback: &str) -> String {
    std::fs::read_to_string(name_state_path)
        .map(|name| name.trim().to_string())
        .ok()
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

/// Sonde qui suit le modèle et l'effort courants d'un agent Codex en observant
/// son fichier de session.
///
/// Elle n'émet que sur changement effectif : un agent inactif ne produit aucun
/// trafic vers le daemon (FR-007).
struct RuntimeProbe {
    pid: u32,
    path: Option<PathBuf>,
    path_resolved_at: Instant,
    last_check: Instant,
    last_mtime: Option<SystemTime>,
    last_sent: Option<crate::runtime::RuntimeObservation>,
}

impl RuntimeProbe {
    fn new(pid: u32) -> Self {
        RuntimeProbe {
            pid,
            path: None,
            // Forcer une première résolution au tout premier tick.
            path_resolved_at: Instant::now() - RUNTIME_PATH_REFRESH,
            last_check: Instant::now() - RUNTIME_PROBE_INTERVAL,
            last_mtime: None,
            last_sent: None,
        }
    }

    /// Rend une observation à transmettre, ou `None` s'il n'y a rien de neuf.
    fn poll(&mut self) -> Option<crate::runtime::RuntimeObservation> {
        if self.last_check.elapsed() < RUNTIME_PROBE_INTERVAL {
            return None;
        }
        self.last_check = Instant::now();

        let stale_path = self
            .path
            .as_ref()
            .map(|path| !path.exists())
            .unwrap_or(true);
        if stale_path || self.path_resolved_at.elapsed() >= RUNTIME_PATH_REFRESH {
            let resolved = crate::runtime::open_session_file(self.pid);
            // Un changement de fichier invalide la date de modification
            // mémorisée : sans cela, un nouveau rollout dont la mtime coïncide
            // avec celle de l'ancien ne serait jamais lu. Défaut soulevé par
            // la contre-revue « agent-1 ».
            if resolved != self.path {
                debug!("sonde runtime : fichier de session {:?}", resolved);
                self.last_mtime = None;
            }
            self.path = resolved;
            self.path_resolved_at = Instant::now();
            if self.path.is_none() {
                debug!(
                    "sonde runtime : aucun fichier de session pour le pid {}",
                    self.pid
                );
            }
        }

        let path = self.path.as_ref()?;
        let mtime = std::fs::metadata(path).ok()?.modified().ok()?;
        if self.last_mtime == Some(mtime) {
            return None;
        }
        self.last_mtime = Some(mtime);

        let observed = crate::runtime::parse_codex_rollout(path)?;
        if self.last_sent.as_ref() == Some(&observed) {
            return None;
        }
        self.last_sent = Some(observed.clone());
        Some(observed)
    }
}

/// Ouvre une connexion vers le daemon et enregistre le wrapper.
///
/// `name = None` laisse le daemon attribuer le nom initial. Après une
/// reconnexion, le wrapper passe son nom établi afin de reprendre son identité.
/// Les paramètres reflètent directement l'enveloppe Register ; les regrouper
/// serait un refactor hors périmètre de T709.
#[allow(clippy::too_many_arguments)]
fn connect_and_register(
    agent_type: &str,
    name: Option<&str>,
    host: &str,
    transport: &str,
    os: &str,
    instance_id: &str,
    domain: Option<&str>,
    turn_in_progress: bool,
) -> Result<(BufReader<UnixStream>, BufWriter<UnixStream>, String), String> {
    connect_and_register_at(
        &socket_path(),
        agent_type,
        name,
        host,
        transport,
        os,
        instance_id,
        domain,
        turn_in_progress,
    )
}

#[allow(clippy::too_many_arguments)]
fn connect_and_register_at(
    socket: &std::path::Path,
    agent_type: &str,
    name: Option<&str>,
    host: &str,
    transport: &str,
    os: &str,
    instance_id: &str,
    domain: Option<&str>,
    turn_in_progress: bool,
) -> Result<(BufReader<UnixStream>, BufWriter<UnixStream>, String), String> {
    let stream = UnixStream::connect(socket).map_err(|e| e.to_string())?;
    set_cloexec(&stream);
    let read_stream = stream.try_clone().map_err(|e| e.to_string())?;
    set_cloexec(&read_stream);
    read_stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .map_err(|e| e.to_string())?;
    let write_stream = stream.try_clone().map_err(|e| e.to_string())?;
    set_cloexec(&write_stream);
    let mut writer = BufWriter::new(write_stream);
    let mut reader = BufReader::new(read_stream);

    let register = WrapperToDaemon::Register {
        agent_type: agent_type.to_string(),
        name: name.map(str::to_owned),
        host: Some(host.to_string()),
        transport: Some(transport.to_string()),
        os: Some(os.to_string()),
        instance_id: Some(instance_id.to_string()),
        domain: domain.map(str::to_owned),
        turn_in_progress,
    };
    writeln!(writer, "{}", encode(&register).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;

    let mut line = String::new();
    reader.read_line(&mut line).map_err(|e| e.to_string())?;
    match decode(line.trim()).map_err(|e| e.to_string())? {
        DaemonToWrapper::Registered { name } => Ok((reader, writer, name)),
        DaemonToWrapper::Nack { reason, .. } => Err(format!("enregistrement refusé: {}", reason)),
        other => Err(format!("réponse inattendue: {:?}", other)),
    }
}

/// Lance un agent CLI wrapper.
/// `agent_binary` = nom de la commande à lancer ("codex", "claude", etc.)
/// `agent_type` = type pour le daemon ("codex", "claude", "custom")
/// `agent_args` = arguments à passer à l'agent CLI
/// `explicit_name` = nom optionnel (--name)
pub fn launch(
    agent_binary: &str,
    agent_type: &str,
    agent_args: &[String],
    explicit_name: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (equipier, agent_args) = split_equipier_flag(agent_args);
    if equipier {
        return launch_acp(agent_type, &agent_args, explicit_name);
    }
    // 1. Enregistrement initial — avec persistance du nom.
    // Si l'utilisateur a passé --name, on l'utilise.
    // Sinon, si on fait un resume, on essaie de retrouver le nom précédent.
    let effective_name = if let Some(n) = explicit_name {
        Some(n.to_string())
    } else {
        load_persistent_name(agent_type, &agent_args)
    };

    let host = host_name();
    let transport = transport_name();
    let os = operating_system();
    let instance_id = uuid::Uuid::new_v4().to_string();
    // Au premier enregistrement, le nom définitif n'est pas encore connu : si
    // l'utilisateur en a demandé un, sa surcharge de domaine est déjà lisible,
    // sinon on part du domaine dérivé.
    let initial_domain = match effective_name.as_deref() {
        Some(name) => effective_domain(name),
        None => derive_domain(),
    };
    let (reader, initial_writer, my_name) = connect_and_register(
        agent_type,
        effective_name.as_deref(),
        &host,
        &transport,
        &os,
        &instance_id,
        initial_domain.as_deref(),
        false,
    )?;
    let writer = Arc::new(Mutex::new(Some(initial_writer)));

    eprintln!("[bridget] Enregistré en tant que « {} »", my_name);
    info!("enregistré: {}", my_name);

    // Sauvegarder le nom pour les futurs resume
    save_persistent_name(agent_type, &agent_args, &my_name);
    let name_state_path = {
        let hash = session_hash(&agent_args);
        if hash.is_empty() {
            socket_path()
                .parent()
                .unwrap()
                .join("agent-names")
                .join(format!("active-{}", my_name))
        } else {
            persistent_name_path(&hash)
        }
    };
    if let Some(parent) = name_state_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&name_state_path, &my_name)?;

    // 3. Détection du pane tmux
    let pane_id = get_current_pane_id().unwrap_or_else(|e| {
        warn!("pas de pane tmux: {}", e);
        String::new()
    });

    // 4. Lancer l'agent CLI
    // Pour Codex : ajouter automatiquement --dangerously-bypass-approvals-and-sandbox
    // (= --yolo) sinon le sandbox bloque la connexion socket vers le daemon.
    // Pour Claude Code : ajouter --dangerously-skip-permissions --permission-mode bypassPermissions
    // + injecter un prompt initial qui dit à l'agent de répondre via bridget.
    let mut final_args: Vec<String> = Vec::new();
    if agent_type == "codex" {
        // Vérifier si l'utilisateur n'a pas déjà passé --yolo ou le bypass
        let already_bypassed = agent_args
            .iter()
            .any(|a| a == "--yolo" || a == "--dangerously-bypass-approvals-and-sandbox");
        if !already_bypassed {
            final_args.push("--dangerously-bypass-approvals-and-sandbox".to_string());
        }

        // Ne PAS utiliser --cd : ça change le working directory de l'agent.
        // Les instructions bridget viennent du prompt initial ci-dessous.
        // Le prompt initial est persistant dans le transcript et survive
        // le compaction de contexte (contrairement à un message système).
    } else if agent_type == "claude" {
        // Pour Claude Code (claude et gclaude)
        let already_bypassed = agent_args
            .iter()
            .any(|a| a.contains("dangerously-skip-permissions") || a.contains("bypassPermissions"));
        if !already_bypassed {
            final_args.push("--dangerously-skip-permissions".to_string());
            final_args.push("--permission-mode".to_string());
            final_args.push("bypassPermissions".to_string());
        }
    }

    // Si l'utilisateur n'a pas passé de prompt initial (un argument libre
    // qui n'est pas un flag --xxx), injecter le prompt bridget.
    let has_prompt = agent_args.iter().any(|a| !a.starts_with("--"));
    if !has_prompt && (agent_type == "codex" || agent_type == "claude") {
        let bridget_prompt = format!(
            "Tu es l'agent \"{}\" dans une session Bridget. Tu peux recevoir des messages d'autres agents IA. Quand tu vois une ligne qui commence par 💬 dans ton terminal, c'est un message d'un autre agent IA, pas de l'humain. Le format est : 💬 <expéditeur> → <toi> (reply=yes/no, ...)\n<message>\n\nRègles ABSOLUES :\n1. Réponds TOUJOURS avec: bridget send --to <expéditeur> \"ta réponse\"\n2. N'accuse JAMAIS réception (pas de \"bien reçu\", \"OK\", \"compris\")\n3. Ne fais PAS bridget who, bridget ledger ou bridget --help sans y être explicitement invité\n4. reply=no = notification, ne réponds que si utile. reply=yes = réponds avec du contenu.\n5. Ne réponds JAMAIS uniquement dans ton terminal — l'expéditeur ne te verrait pas.\n\nTu es maintenant en attente. Dis \"Bridget ready\" puis attends les messages.",
            my_name
        );
        final_args.push(bridget_prompt);
    }

    final_args.extend(agent_args.iter().cloned());

    // L'autorisation est déclarative : un type absent du registre est refusé
    // avant le spawn, avec les types disponibles et le fichier concerné.
    crate::registry::AgentRegistry::load()?.get(agent_type)?;

    // Validation des arguments pour prévenir injection
    for arg in &final_args {
        // Rejeter les tentatives d'injection de commandes
        if arg.contains(';') || arg.contains('&') || arg.contains('|') || arg.contains('$') {
            return Err(format!(
                "Argument non autorisé contient des caractères shell dangereux: '{}'",
                arg
            )
            .into());
        }
    }

    eprintln!(
        "[bridget] Lancement: {} {}",
        agent_binary,
        final_args.join(" ")
    );

    let mut child = Command::new(agent_binary)
        .args(&final_args)
        .env("BRIDGET_AGENT_NAME", &my_name)
        .env("BRIDGET_AGENT_NAME_FILE", &name_state_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("impossible de lancer '{}': {}", agent_binary, e))?;

    let agent_pid = child.id();

    // 5. Thread d'écoute
    let writer_clone = writer.clone();
    let writer_for_listener = writer_clone.clone();
    let pane_for_thread = pane_id.clone();
    let name_state_for_thread = name_state_path.clone();
    let agent_type_for_thread = agent_type.to_string();
    let mut my_name_for_thread = my_name.clone();
    let host_for_thread = host.clone();
    let transport_for_thread = transport.clone();
    let os_for_thread = os.clone();
    let instance_id_for_thread = instance_id.clone();
    let stopping = Arc::new(AtomicBool::new(false));
    let stopping_for_thread = stopping.clone();

    let listener_handle = thread::spawn(move || {
        let mut listener = reader;
        let mut transport = if !pane_for_thread.is_empty() {
            Some(TmuxTransport::new(pane_for_thread.clone(), agent_pid))
        } else {
            None
        };
        let mut connected_since = Instant::now();
        let mut failed_attempts = 0_u32;
        let mut last_heartbeat = Instant::now();
        // Seul Codex tient son fichier de session ouvert ; pour Claude, c'est
        // le hook `Stop` qui rapporte le runtime (research.md D-002).
        let mut runtime_probe =
            (agent_type_for_thread == "codex").then(|| RuntimeProbe::new(agent_pid));

        'connection: while !stopping_for_thread.load(Ordering::SeqCst) {
            let mut line = String::new();
            match listener.read_line(&mut line) {
                Ok(0) => {
                    // Connexion fermée par le daemon
                    warn!("connexion fermée par le daemon");
                }
                Ok(_) => {
                    // Données reçues, traiter plus bas
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    // Timeout de lecture - vérifier heartbeat et socket
                    if last_heartbeat.elapsed() >= HEARTBEAT_INTERVAL {
                        let heartbeat = encode(&WrapperToDaemon::Heartbeat).unwrap_or_default();
                        if let Some(writer) = writer_for_listener.lock().unwrap().as_mut() {
                            match writeln!(writer, "{}", heartbeat).and_then(|_| writer.flush()) {
                                Ok(_) => {
                                    last_heartbeat = Instant::now();
                                    debug!("heartbeat envoyé avec succès");
                                }
                                Err(e) => {
                                    error!("heartbeat échoué: {}", e);
                                    // Heartbeat échoué = probablement déconnecté
                                    warn!("Détection de déconnexion via heartbeat échoué");
                                }
                            }
                        } else {
                            error!("Impossible d'obtenir le writer pour heartbeat");
                        }
                    }

                    // Sonde de runtime : greffée sur le même réveil que le
                    // heartbeat, elle ne coûte qu'un `stat` la plupart du temps.
                    if let Some(observed) = runtime_probe.as_mut().and_then(RuntimeProbe::poll) {
                        // Le nom est relu à chaque émission : `bridget rename`
                        // met à jour ce fichier, pas la variable capturée au
                        // démarrage. S'adresser au nom initial vaudrait un
                        // « agent introuvable » sur tout agent renommé.
                        let current_name = std::fs::read_to_string(&name_state_for_thread)
                            .map(|name| name.trim().to_string())
                            .ok()
                            .filter(|name| !name.is_empty())
                            .unwrap_or_else(|| my_name_for_thread.clone());
                        let message = WrapperToDaemon::Runtime {
                            agent: current_name,
                            model: observed.model.clone(),
                            effort: observed.effort.clone(),
                            source: bridget_transport::protocol::RuntimeSource::CodexRollout,
                        };
                        match encode(&message) {
                            Ok(json) => {
                                if let Some(writer) = writer_for_listener.lock().unwrap().as_mut() {
                                    if let Err(e) =
                                        writeln!(writer, "{}", json).and_then(|_| writer.flush())
                                    {
                                        debug!("sonde runtime : envoi impossible ({})", e);
                                    } else {
                                        debug!(
                                            "sonde runtime : modèle={} effort={:?}",
                                            observed.model, observed.effort
                                        );
                                    }
                                }
                            }
                            Err(e) => debug!("sonde runtime : encodage impossible ({})", e),
                        }
                    }

                    // Vérification proactive du socket (auto-reconnect)
                    // Vérifier toutes les X secondes si le socket existe toujours
                    if Instant::now()
                        .duration_since(last_heartbeat)
                        .as_secs()
                        .is_multiple_of(SOCKET_CHECK_INTERVAL.as_secs())
                    {
                        if socket_path().exists() {
                            debug!("Socket Bridget détecté - daemon probablement disponible");
                        } else {
                            warn!("⚠️ Socket Bridget absent - daemon probablement arrêté");
                        }
                    }

                    continue;
                }
                Err(e) => {
                    error!("Erreur de lecture: {}", e);
                    // Erreur de lecture = probablement déconnecté
                }
            }

            if line.is_empty() {
                // Détection de déconnexion
                if stopping_for_thread.load(Ordering::SeqCst) {
                    info!("Arrêt demandé, déconnexion propre");
                    break;
                }

                warn!(
                    "🔌 Connexion Bridget perdue pour « {} » - reconnexion automatique...",
                    my_name_for_thread
                );

                // Reset du compteur si la connexion était stable
                if connected_since.elapsed() >= RECONNECT_STABLE_RESET {
                    failed_attempts = 0;
                    info!("Connexion était stable - reset du compteur de tentatives");
                }

                // Boucle de reconnexion avec backoff exponentiel
                loop {
                    if stopping_for_thread.load(Ordering::SeqCst) {
                        info!("Arrêt demandé pendant reconnexion");
                        break 'connection;
                    }

                    let delay = reconnect_delay(failed_attempts);
                    failed_attempts = failed_attempts.saturating_add(1);

                    // Le nom choisi par l'utilisateur prime sur celui obtenu au
                    // démarrage : sans cela, un agent renommé revient sous son
                    // nom d'origine à chaque coupure.
                    let wanted_name =
                        resolve_current_name(&name_state_for_thread, &my_name_for_thread);

                    info!(
                        "🔄 Tentative de reconnexion {} pour « {} » (délai: {:.1}s)",
                        failed_attempts,
                        wanted_name,
                        delay.as_secs_f64()
                    );

                    thread::sleep(delay);
                    match connect_and_register(
                        &agent_type_for_thread,
                        Some(&wanted_name),
                        &host_for_thread,
                        &transport_for_thread,
                        &os_for_thread,
                        &instance_id_for_thread,
                        effective_domain(&wanted_name).as_deref(),
                        false,
                    ) {
                        Ok((new_reader, new_writer, registered_name)) => {
                            if registered_name != wanted_name {
                                warn!(
                                    "⚠️ Reconnexion refusée : nom inattendu « {} » (attendu: « {} »)",
                                    registered_name, wanted_name
                                );
                                continue;
                            }

                            // Reconnexion réussie ! 🎉
                            *writer_for_listener.lock().unwrap() = Some(new_writer);
                            listener = new_reader;
                            connected_since = Instant::now();
                            last_heartbeat = Instant::now();
                            failed_attempts = 0; // Reset du compteur
                            my_name_for_thread = registered_name;

                            info!(
                                "✅ Agent « {} » reconnecté au daemon avec succès !",
                                my_name_for_thread
                            );

                            // Notification visuelle à l'utilisateur (si tmux)
                            if let Some(ref mut t) = transport {
                                let notif = "🔄 Bridget: reconnecté au daemon".to_string();
                                if let Err(e) = t.deliver(&bridget_core::BridgetMessage::new(
                                    "bridget",
                                    &my_name_for_thread,
                                    &notif,
                                )) {
                                    error!(
                                        "Impossible d'afficher la notification de reconnexion: {}",
                                        e
                                    );
                                }
                            }

                            continue 'connection;
                        }
                        Err(error) => {
                            warn!(
                                "❌ Reconnexion échouée pour « {} » : {} (tentative {})",
                                wanted_name, error, failed_attempts
                            );
                        }
                    }
                }
            }
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let msg: DaemonToWrapper = match decode(line) {
                Ok(m) => m,
                Err(e) => {
                    warn!("message illisible: {}", e);
                    continue;
                }
            };

            match msg {
                DaemonToWrapper::Deliver(bm) => {
                    info!(
                        "reçu de « {} »: {}",
                        bm.from,
                        bm.body.chars().take(60).collect::<String>()
                    );
                    // Stocker le dernier expéditeur pour la commande reply
                    let name_for_reply = std::fs::read_to_string(&name_state_for_thread)
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    if bm.reply && !name_for_reply.is_empty() {
                        let reply_file = socket_path()
                            .parent()
                            .unwrap()
                            .join(format!("last-sender-{}", name_for_reply));
                        let value = if bm.reply {
                            format!("{}\t{}", bm.from, bm.id)
                        } else {
                            bm.from.clone()
                        };
                        let _ = std::fs::write(&reply_file, value);
                    }
                    if let Some(ref mut t) = transport {
                        if let Err(e) = t.deliver(&bm) {
                            error!("injection tmux: {}", e);
                        }
                    } else {
                        warn!("livraison ignorée : aucun pane tmux pour {}", bm.id);
                    }
                }
                DaemonToWrapper::Disconnect => {
                    info!("daemon déconnecté");
                    if connected_since.elapsed() >= RECONNECT_STABLE_RESET {
                        failed_attempts = 0;
                    }
                    loop {
                        if stopping_for_thread.load(Ordering::SeqCst) {
                            break 'connection;
                        }
                        let delay = reconnect_delay(failed_attempts);
                        failed_attempts = failed_attempts.saturating_add(1);
                        thread::sleep(delay);
                        let wanted_name =
                            resolve_current_name(&name_state_for_thread, &my_name_for_thread);
                        match connect_and_register(
                            &agent_type_for_thread,
                            Some(&wanted_name),
                            &host_for_thread,
                            &transport_for_thread,
                            &os_for_thread,
                            &instance_id_for_thread,
                            effective_domain(&wanted_name).as_deref(),
                            false,
                        ) {
                            Ok((new_reader, new_writer, registered_name))
                                if registered_name == wanted_name =>
                            {
                                *writer_for_listener.lock().unwrap() = Some(new_writer);
                                listener = new_reader;
                                connected_since = Instant::now();
                                last_heartbeat = Instant::now();
                                my_name_for_thread = registered_name;
                                continue 'connection;
                            }
                            Ok((_, _, registered_name)) => warn!(
                                "reconnexion refusée : nom inattendu « {} »",
                                registered_name
                            ),
                            Err(error) => warn!(
                                "reconnexion Bridget de « {} » impossible : {}",
                                wanted_name, error
                            ),
                        }
                    }
                }
                _ => {}
            }
        }
    });

    // 6. Attendre la fin de l'agent
    let status = child.wait()?;

    // 7. Désenregistrement
    stopping.store(true, Ordering::SeqCst);
    {
        if let Ok(json) = encode(&WrapperToDaemon::Unregister)
            && let Ok(mut writer) = writer_clone.lock()
            && let Some(w) = writer.as_mut()
        {
            let _ = writeln!(w, "{}", json);
            let _ = w.flush();
        }
    }

    *writer.lock().unwrap() = None;
    let _ = listener_handle.join();

    if let Some(code) = status.code() {
        std::process::exit(code);
    }
    std::process::exit(0);
}

fn split_equipier_flag(agent_args: &[String]) -> (bool, Vec<String>) {
    let mut equipier = false;
    let mut remaining = Vec::with_capacity(agent_args.len());
    for argument in agent_args {
        if argument == "--equipier" {
            equipier = true;
        } else {
            remaining.push(argument.clone());
        }
    }
    (equipier, remaining)
}

enum AttachRelayCommand {
    Subscribe {
        subscription_id: String,
        window: AttachWindow,
    },
}

const ATTACH_RELAY_CONTROL_IDS_CAPACITY: usize = ATTACH_RELAY_COMMAND_CAPACITY * 2;

#[derive(Default)]
struct AttachRelayControlState {
    pending: HashSet<String>,
    active: HashSet<String>,
    cancelled: HashSet<String>,
}

struct RelaySubscription {
    subscription_id: String,
    window: AttachWindow,
    from_seq: Option<u64>,
    files: Vec<PathBuf>,
    follow_after: Option<PathBuf>,
    file_index: usize,
    reader: Option<IncrementalJournalReader>,
    reader_source: Option<JournalSourceIdentity>,
    caught_up: bool,
    through_seq: Option<u64>,
    pending_events: VecDeque<(u64, Vec<u8>)>,
    pending_fragment: Option<(u64, Vec<u8>, usize)>,
}

impl RelaySubscription {
    fn new(
        subscription_id: String,
        window: AttachWindow,
        directory: &std::path::Path,
        host_today: &str,
    ) -> Result<Self, JournalWindowError> {
        let resolved = resolve_window(directory, &window, host_today)?;
        let follow_after = resolve_window(directory, &AttachWindow::Seq(0), host_today)?
            .files
            .last()
            .cloned();
        Ok(Self {
            subscription_id,
            window,
            from_seq: resolved.from_seq,
            files: resolved.files,
            follow_after,
            file_index: 0,
            reader: None,
            reader_source: None,
            caught_up: false,
            through_seq: None,
            pending_events: VecDeque::new(),
            pending_fragment: None,
        })
    }

    fn refresh_files(&mut self, directory: &std::path::Path, host_today: &str) {
        let Ok(resolved) = resolve_window(directory, &AttachWindow::Seq(0), host_today) else {
            return;
        };
        if self.caught_up {
            // Une fenêtre Today/Date borne le rejeu initial seulement. Une fois
            // le snapshot atteint, seul un fichier plus récent que le curseur
            // connu entre dans le suivi, jamais l'historique antérieur.
            for path in resolved.files {
                if self
                    .follow_after
                    .as_ref()
                    .is_none_or(|latest| path > *latest)
                    && !self.files.contains(&path)
                {
                    self.files.push(path);
                }
            }
        } else {
            let Ok(initial) = resolve_window(directory, &self.window, host_today) else {
                return;
            };
            for path in initial.files {
                if !self.files.contains(&path) {
                    self.files.push(path);
                }
            }
        }
    }

    fn next_reader(&mut self) -> Option<&mut IncrementalJournalReader> {
        if self.reader.is_none() {
            let path = self.files.get(self.file_index)?.clone();
            self.reader = Some(IncrementalJournalReader::new(path));
        }
        self.reader.as_mut()
    }

    fn has_pending_output(&self) -> bool {
        self.pending_fragment.is_some() || !self.pending_events.is_empty()
    }
}

#[derive(Clone)]
struct AttachRelayHooks {
    before_command: Arc<dyn Fn() + Send + Sync>,
    before_read: Arc<dyn Fn() + Send + Sync>,
    control_observed: Arc<dyn Fn() + Send + Sync>,
}

impl Default for AttachRelayHooks {
    fn default() -> Self {
        Self {
            before_command: Arc::new(|| {}),
            before_read: Arc::new(|| {}),
            control_observed: Arc::new(|| {}),
        }
    }
}

/// Relais dédié du journal ACP : la boucle de lecture du wrapper ne fait que
/// déposer les commandes, tandis que ce worker traite chaque abonnement par
/// petites tranches. Le contrôle coalescé vit dans un état borné partagé afin
/// qu'un désabonnement ne soit jamais coincé derrière un rejeu volumineux.
struct AttachRelayWorker {
    commands: mpsc::SyncSender<AttachRelayCommand>,
    stopped: Arc<AtomicBool>,
    control_state: Arc<Mutex<AttachRelayControlState>>,
    control_observed: Arc<dyn Fn() + Send + Sync>,
    wake: Arc<(Mutex<()>, Condvar)>,
    generation: Arc<AtomicU64>,
    generation_ack: Arc<(Mutex<u64>, Condvar)>,
    worker: Option<thread::JoinHandle<()>>,
}

type RelayEmitter = Arc<dyn Fn(WrapperToDaemon) + Send + Sync>;
#[cfg(test)]
type RelayEvents = Arc<Mutex<Vec<WrapperToDaemon>>>;

impl AttachRelayWorker {
    fn start(directory: PathBuf, emit: RelayEmitter) -> Self {
        Self::start_with_clock(
            directory,
            Arc::new(current_host_date),
            ATTACH_RELAY_COMMAND_CAPACITY,
            emit,
            AttachRelayHooks::default(),
        )
    }

    #[cfg(test)]
    fn start_with(
        directory: PathBuf,
        host_today: String,
        capacity: usize,
        emit: RelayEmitter,
        hooks: AttachRelayHooks,
    ) -> Self {
        let clock: Arc<dyn Fn() -> String + Send + Sync> = Arc::new(move || host_today.clone());
        Self::start_with_clock(directory, clock, capacity, emit, hooks)
    }

    fn start_with_clock(
        directory: PathBuf,
        host_today: Arc<dyn Fn() -> String + Send + Sync>,
        capacity: usize,
        emit: RelayEmitter,
        hooks: AttachRelayHooks,
    ) -> Self {
        let (command_sender, command_receiver) = mpsc::sync_channel(capacity);
        let stopped = Arc::new(AtomicBool::new(false));
        let control_state = Arc::new(Mutex::new(AttachRelayControlState::default()));
        let wake = Arc::new((Mutex::new(()), Condvar::new()));
        let generation = Arc::new(AtomicU64::new(0));
        let generation_ack = Arc::new((Mutex::new(0), Condvar::new()));

        let worker_stopped = stopped.clone();
        let worker_control = control_state.clone();
        let worker_wake = wake.clone();
        let worker_generation = generation.clone();
        let worker_generation_ack = generation_ack.clone();
        let worker_emit = emit.clone();
        let worker = thread::spawn(move || {
            let mut subscriptions = BTreeMap::<String, RelaySubscription>::new();
            let mut seen_generation = 0;
            while !worker_stopped.load(Ordering::SeqCst) {
                let current_generation = worker_generation.load(Ordering::SeqCst);
                if current_generation != seen_generation {
                    subscriptions.clear();
                    worker_control
                        .lock()
                        .unwrap_or_else(|poison| poison.into_inner())
                        .active
                        .clear();
                    seen_generation = current_generation;
                    let (ack, wake) = &*worker_generation_ack;
                    *ack.lock().unwrap_or_else(|poison| poison.into_inner()) = seen_generation;
                    wake.notify_all();
                    continue;
                }
                (hooks.before_command)();
                let command = if subscriptions.is_empty() {
                    match command_receiver.recv_timeout(ATTACH_RELAY_IDLE_WAIT) {
                        Ok(command) => Some(command),
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        Err(mpsc::RecvTimeoutError::Timeout) => None,
                    }
                } else {
                    match command_receiver.try_recv() {
                        Ok(command) => Some(command),
                        Err(mpsc::TryRecvError::Disconnected) => break,
                        Err(mpsc::TryRecvError::Empty) => None,
                    }
                };
                if let Some(AttachRelayCommand::Subscribe {
                    subscription_id,
                    window,
                }) = command
                {
                    let cancelled = {
                        let mut state = worker_control
                            .lock()
                            .unwrap_or_else(|poison| poison.into_inner());
                        state.pending.remove(&subscription_id);
                        let cancelled = state.cancelled.remove(&subscription_id);
                        if !cancelled {
                            state.active.insert(subscription_id.clone());
                        }
                        cancelled
                    };
                    if cancelled {
                        continue;
                    }
                    match RelaySubscription::new(
                        subscription_id.clone(),
                        window,
                        &directory,
                        &host_today(),
                    ) {
                        Ok(subscription) => {
                            worker_emit(WrapperToDaemon::Subscribed {
                                subscription_id: subscription_id.clone(),
                            });
                            subscriptions.insert(subscription_id, subscription);
                        }
                        Err(error) => {
                            worker_control
                                .lock()
                                .unwrap_or_else(|poison| poison.into_inner())
                                .active
                                .remove(&subscription_id);
                            worker_emit(WrapperToDaemon::AttachRejected {
                                subscription_id: Some(subscription_id),
                                reason: refusal_for_window(error),
                            });
                        }
                    }
                }

                // Aucune souscription => aucune lecture de fichier : le relais
                // ne perturbe jamais l'écrivain JSONL d'un équipier inobservé.
                let ids = subscriptions.keys().cloned().collect::<Vec<_>>();
                for subscription_id in ids {
                    if worker_control
                        .lock()
                        .unwrap_or_else(|poison| poison.into_inner())
                        .cancelled
                        .remove(&subscription_id)
                    {
                        worker_control
                            .lock()
                            .unwrap_or_else(|poison| poison.into_inner())
                            .active
                            .remove(&subscription_id);
                        subscriptions.remove(&subscription_id);
                        continue;
                    }
                    let Some(subscription) = subscriptions.get_mut(&subscription_id) else {
                        continue;
                    };
                    if emit_one_fragment(&worker_emit, subscription) {
                        continue;
                    }
                    subscription.refresh_files(&directory, &host_today());
                    if subscription.file_index >= subscription.files.len() {
                        if !subscription.caught_up {
                            worker_emit(WrapperToDaemon::SnapshotCaughtUp {
                                subscription_id: subscription.subscription_id.clone(),
                                through_seq: subscription.through_seq,
                            });
                            subscription.caught_up = true;
                        }
                        continue;
                    }
                    (hooks.before_read)();
                    if worker_control
                        .lock()
                        .unwrap_or_else(|poison| poison.into_inner())
                        .cancelled
                        .contains(&subscription_id)
                    {
                        continue;
                    }
                    let prior_offset = subscription
                        .reader
                        .as_ref()
                        .map_or(0, IncrementalJournalReader::next_offset);
                    let reader = subscription
                        .next_reader()
                        .expect("fichier de relais présent");
                    let (items, source) = match reader.read_chunk_with_source(ATTACH_RELAY_READ_BYTES) {
                        Ok(result) => result,
                        Err(error) => {
                            worker_emit(WrapperToDaemon::End {
                                subscription_id: subscription.subscription_id.clone(),
                                reason: format!("lecture du journal impossible: {error}"),
                            });
                            subscriptions.remove(&subscription_id);
                            continue;
                        }
                    };
                    let next_offset = reader.next_offset();
                    let source_replaced = source.is_some_and(|source| {
                        subscription
                            .reader_source
                            .is_some_and(|previous| !previous.same_file(source))
                    });
                    let source_truncated = source.is_some_and(|source| source.len < prior_offset);
                    let source_missing = source.is_none();
                    if source_replaced || source_truncated || source_missing {
                        let reason = if source_replaced {
                            "source_replaced"
                        } else if source_truncated {
                            "source_truncated"
                        } else {
                            "source de journal indisponible"
                        };
                        worker_emit(WrapperToDaemon::End {
                            subscription_id: subscription.subscription_id.clone(),
                            reason: reason.to_string(),
                        });
                        worker_control
                            .lock()
                            .unwrap_or_else(|poison| poison.into_inner())
                            .active
                            .remove(&subscription_id);
                        subscriptions.remove(&subscription_id);
                        continue;
                    }
                    subscription.reader_source = source;
                    for item in items {
                        match item {
                            JournalReadItem::Event(event)
                                if subscription
                                    .from_seq
                                    .is_none_or(|from_seq| event.seq >= from_seq) =>
                            {
                                subscription
                                    .pending_events
                                    .push_back((event.seq, event.bytes));
                            }
                            JournalReadItem::Oversized { seq: Some(seq), .. } => {
                                worker_emit(WrapperToDaemon::Gap {
                                    subscription_id: subscription.subscription_id.clone(),
                                    from_seq: seq,
                                    to_seq: seq,
                                    reason: Some("event_too_large".to_string()),
                                })
                            }
                            JournalReadItem::Oversized {
                                seq: None,
                                offset,
                                line,
                            } => worker_emit(WrapperToDaemon::JournalReadError {
                                subscription_id: subscription.subscription_id.clone(),
                                line,
                                offset,
                                reason: "event_too_large sans séquence".to_string(),
                            }),
                            JournalReadItem::Unreadable(line) => {
                                worker_emit(WrapperToDaemon::JournalReadError {
                                    subscription_id: subscription.subscription_id.clone(),
                                    line: line.line,
                                    offset: line.offset,
                                    reason: "ligne de journal illisible".to_string(),
                                })
                            }
                            JournalReadItem::Event(_) => {}
                        }
                    }
                    let file_finished = source.is_some_and(|source| next_offset >= source.len);
                    if file_finished
                        && subscription.file_index.saturating_add(1) < subscription.files.len()
                    {
                        subscription.reader = None;
                        subscription.reader_source = None;
                        subscription.file_index = subscription.file_index.saturating_add(1);
                    } else if file_finished
                        && !subscription.caught_up
                        && !subscription.has_pending_output()
                    {
                        worker_emit(WrapperToDaemon::SnapshotCaughtUp {
                            subscription_id: subscription.subscription_id.clone(),
                            through_seq: subscription.through_seq,
                        });
                        subscription.caught_up = true;
                    }
                }
                if subscriptions.values().all(|subscription| {
                    subscription.caught_up && !subscription.has_pending_output()
                }) {
                    let (lock, wake) = &*worker_wake;
                    let guard = lock.lock().unwrap_or_else(|poison| poison.into_inner());
                    let _ = wake
                        .wait_timeout(guard, ATTACH_RELAY_IDLE_WAIT)
                        .unwrap_or_else(|poison| poison.into_inner());
                }
            }
        });

        Self {
            commands: command_sender,
            stopped,
            control_state,
            control_observed: hooks.control_observed.clone(),
            wake,
            generation,
            generation_ack,
            worker: Some(worker),
        }
    }

    fn subscribe(
        &self,
        subscription_id: String,
        window: AttachWindow,
    ) -> Result<(), AttachRefusal> {
        {
            let mut state = self
                .control_state
                .lock()
                .unwrap_or_else(|poison| poison.into_inner());
            if state.pending.len().saturating_add(state.active.len())
                >= ATTACH_RELAY_CONTROL_IDS_CAPACITY
            {
                return Err(AttachRefusal::CommandQueueSaturated);
            }
            state.pending.insert(subscription_id.clone());
        }
        let result = self
            .commands
            .try_send(AttachRelayCommand::Subscribe {
                subscription_id: subscription_id.clone(),
                window,
            })
            .map_err(|error| {
                self.control_state
                    .lock()
                    .unwrap_or_else(|poison| poison.into_inner())
                    .pending
                    .remove(&subscription_id);
                match error {
                    mpsc::TrySendError::Full(_) | mpsc::TrySendError::Disconnected(_) => {
                        AttachRefusal::CommandQueueSaturated
                    }
                }
            });
        self.wake.1.notify_all();
        result
    }

    fn unsubscribe(&self, subscription_id: String) {
        let mut state = self
            .control_state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if state.pending.contains(&subscription_id) || state.active.contains(&subscription_id) {
            state.cancelled.insert(subscription_id);
            drop(state);
            (self.control_observed)();
            self.wake.1.notify_all();
        }
    }

    fn reset_generation(&self) {
        let mut state = self
            .control_state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let pending = state.pending.iter().cloned().collect::<Vec<_>>();
        state.cancelled.extend(pending);
        state.active.clear();
        drop(state);
        self.wake.1.notify_all();
        let generation = self
            .generation
            .fetch_add(1, Ordering::SeqCst)
            .saturating_add(1);
        let (ack, wake) = &*self.generation_ack;
        let mut acknowledged = ack.lock().unwrap_or_else(|poison| poison.into_inner());
        while *acknowledged < generation {
            acknowledged = wake
                .wait(acknowledged)
                .unwrap_or_else(|poison| poison.into_inner());
        }
    }

    fn shutdown(&mut self) {
        if self.stopped.swap(true, Ordering::SeqCst) {
            return;
        }
        // Le Stop n'emprunte jamais une file : le worker observe ce drapeau
        // avant chaque commande, même sous une rafale de souscriptions.
        self.wake.1.notify_all();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Drop for AttachRelayWorker {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn refusal_for_window(error: JournalWindowError) -> AttachRefusal {
    match error {
        JournalWindowError::InvalidDate => AttachRefusal::InvalidDate,
        JournalWindowError::FutureDate => AttachRefusal::FutureDate,
        JournalWindowError::DateOutsideRetention => AttachRefusal::DateOutsideRetention,
    }
}

fn emit_one_fragment(emit: &RelayEmitter, subscription: &mut RelaySubscription) -> bool {
    if subscription.pending_fragment.is_none()
        && let Some((seq, bytes)) = subscription.pending_events.pop_front()
    {
        subscription.pending_fragment = Some((seq, bytes, 0));
    }
    let Some((seq, bytes, offset)) = subscription.pending_fragment.as_ref() else {
        return false;
    };
    let seq = *seq;
    let offset = *offset;
    let remaining = bytes.len().saturating_sub(offset);
    if remaining == 0 {
        subscription.pending_fragment = None;
        subscription.through_seq = Some(seq);
        return false;
    }
    let mut low = 1_usize;
    let mut high = remaining.min(MAX_ATTACH_FRAGMENT_BYTES);
    while low < high {
        let candidate = (low + high).div_ceil(2);
        let frame = WrapperToDaemon::JournalFragment {
            subscription_id: subscription.subscription_id.clone(),
            seq,
            offset: offset as u64,
            final_fragment: candidate == remaining,
            bytes: bytes[offset..offset + candidate].to_vec(),
        };
        if encode(&frame).is_ok_and(|json| json.len() <= MAX_ATTACH_SERIALIZED_FRAME_BYTES) {
            low = candidate;
        } else {
            high = candidate.saturating_sub(1);
        }
    }
    let length = low;
    let final_fragment = length == remaining;
    emit(WrapperToDaemon::JournalFragment {
        subscription_id: subscription.subscription_id.clone(),
        seq,
        offset: offset as u64,
        final_fragment,
        bytes: bytes[offset..offset + length].to_vec(),
    });
    if final_fragment {
        subscription.pending_fragment = None;
        subscription.through_seq = Some(seq);
    } else if let Some((_, _, next_offset)) = subscription.pending_fragment.as_mut() {
        *next_offset = offset.saturating_add(length);
    }
    true
}

fn launch_acp(
    agent_type: &str,
    agent_args: &[String],
    explicit_name: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let registry = crate::registry::AgentRegistry::load()?;
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or("HOME absent pour le journal de session ACP")?;
    launch_acp_with(
        agent_type,
        agent_args,
        explicit_name,
        &registry,
        &socket_path(),
        &home,
    )
}

/// Lance un équipier ACP avec ses dépendances de configuration et de chemins
/// explicites. Le flux de production passe par [`launch_acp`]; cette variante
/// rend le même chemin vérifiable avec un registre et un daemon temporaires.
pub fn launch_acp_with(
    agent_type: &str,
    agent_args: &[String],
    explicit_name: Option<&str>,
    registry: &crate::registry::AgentRegistry,
    socket: &std::path::Path,
    home: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if !agent_args.is_empty() {
        return Err("le mode --equipier n'accepte pas d'arguments d'agent".into());
    }
    let definition = registry.get(agent_type)?;
    if definition.protocol != "acp" {
        return Err(format!("le type '{agent_type}' n'utilise pas le protocole ACP").into());
    }
    if let Some(error) = forbidden_env_error(
        definition,
        allow_api_key_value(std::env::var("BRIDGET_ALLOW_API_KEY").ok().as_deref()),
        |variable| std::env::var_os(variable).is_some(),
    ) {
        return Err(error.into());
    }

    let effective_name = explicit_name.map(str::to_owned);
    let host = host_name();
    let os = operating_system();
    let instance_id = uuid::Uuid::new_v4().to_string();
    let initial_domain = effective_name
        .as_deref()
        .and_then(effective_domain)
        .or_else(derive_domain);
    let (mut reader, initial_writer, mut my_name) = connect_and_register_at(
        socket,
        agent_type,
        effective_name.as_deref(),
        &host,
        "acp",
        &os,
        &instance_id,
        initial_domain.as_deref(),
        false,
    )?;
    let writer = Arc::new(Mutex::new(Some(initial_writer)));
    let mut idempotent_deliveries = IdempotentDeliveryTracker::open(home, &instance_id)?;
    let name_state_path = socket
        .parent()
        .unwrap()
        .join("agent-names")
        .join(format!("active-{my_name}"));
    if let Some(parent) = name_state_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&name_state_path, &my_name)?;
    let mut transport = AcpTransport::spawn(AcpOptions {
        command: definition.command.clone(),
        args: definition.args.clone(),
        queue_capacity: definition.queue_capacity,
        permissions: definition.permissions.clone(),
        notify_timeout_secs: definition.notify_timeout_secs,
    })?;
    transport.enable_journal(home.join(".cache/bridget/sessions"), &my_name)?;
    let journal_directory = home.join(".cache/bridget/sessions").join(&my_name);
    let relay_writer = writer.clone();
    let mut relay = AttachRelayWorker::start(
        journal_directory,
        Arc::new(move |message| send_wrapper_message(&relay_writer, message)),
    );

    loop {
        let events = transport.drain_events();
        let journal_failed =
            forward_acp_events(&writer, &my_name, events, &mut idempotent_deliveries);
        if journal_failed {
            transport.shutdown();
            break;
        }
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => {
                relay.reset_generation();
                let Some((new_reader, registered_name)) = reconnect_acp(
                    socket,
                    &writer,
                    &transport,
                    agent_type,
                    &name_state_path,
                    &host,
                    &os,
                    &instance_id,
                    &my_name,
                ) else {
                    break;
                };
                reader = new_reader;
                my_name = registered_name;
                continue;
            }
            Ok(_) => match decode(line.trim()) {
                Ok(DaemonToWrapper::Deliver(message)) => {
                    if let Err(error) = transport.deliver(&message) {
                        send_wrapper_message(
                            &writer,
                            WrapperToDaemon::DeliveryRejected {
                                id: message.id,
                                reason: error.to_string(),
                            },
                        );
                    }
                }
                Ok(DaemonToWrapper::DeliverIdempotent {
                    delivery_id,
                    recipient_instance_id,
                    delivery_generation,
                    expires_at,
                    message,
                }) => match idempotent_deliveries.receive(
                    delivery_id,
                    recipient_instance_id,
                    delivery_generation,
                    expires_at,
                    message,
                    unix_now_secs(),
                ) {
                    IdempotentDeliveryAction::Report(report) => {
                        send_wrapper_message(&writer, report);
                    }
                    IdempotentDeliveryAction::Inject {
                        message,
                        delivery_id,
                    } => {
                        let message_id = message.id.clone();
                        if let Err(error) = transport.deliver(&message) {
                            send_wrapper_message(
                                &writer,
                                WrapperToDaemon::DeliveryRejected {
                                    id: message_id.clone(),
                                    reason: error.to_string(),
                                },
                            );
                            if let Some(report) =
                                idempotent_deliveries.injection_failed(&delivery_id)
                            {
                                send_wrapper_message(&writer, report);
                            }
                        }
                    }
                },
                Ok(DaemonToWrapper::CancelDelivery { id, reason }) => {
                    transport.cancel_delivery(&id, &reason);
                }
                Ok(DaemonToWrapper::Subscribe {
                    subscription_id,
                    window,
                    ..
                }) => {
                    if let Err(reason) = relay.subscribe(subscription_id.clone(), window) {
                        send_wrapper_message(
                            &writer,
                            WrapperToDaemon::AttachRejected {
                                subscription_id: Some(subscription_id),
                                reason,
                            },
                        );
                    }
                }
                Ok(DaemonToWrapper::Unsubscribe { subscription_id }) => {
                    relay.unsubscribe(subscription_id);
                }
                Ok(DaemonToWrapper::Disconnect) => break,
                Ok(_) => {}
                Err(error) => warn!("message ACP illisible: {}", error),
            },
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(error) => {
                warn!("connexion daemon ACP perdue : {error}");
                relay.reset_generation();
                let Some((new_reader, registered_name)) = reconnect_acp(
                    socket,
                    &writer,
                    &transport,
                    agent_type,
                    &name_state_path,
                    &host,
                    &os,
                    &instance_id,
                    &my_name,
                ) else {
                    break;
                };
                reader = new_reader;
                my_name = registered_name;
                continue;
            }
        }
        if !transport.is_alive() {
            break;
        }
    }
    relay.shutdown();
    transport.shutdown();
    // L'EOF peut fermer le transport entre deux itérations : vider une dernière
    // fois les événements terminaux avant Unregister afin que le daemon voie
    // chaque DeliveryRejected (tour actif comme file restante).
    let _ = forward_acp_events(
        &writer,
        &my_name,
        transport.drain_events(),
        &mut idempotent_deliveries,
    );
    send_wrapper_message(&writer, WrapperToDaemon::TurnState { in_progress: false });
    send_wrapper_message(&writer, WrapperToDaemon::Unregister);
    Ok(())
}

fn allow_api_key_value(value: Option<&str>) -> bool {
    value == Some("1")
}

fn forbidden_env_error(
    definition: &crate::registry::AgentDefinition,
    allow_api_key: bool,
    is_present: impl Fn(&str) -> bool,
) -> Option<String> {
    if allow_api_key {
        return None;
    }
    definition
        .forbidden_env
        .iter()
        .find(|variable| is_present(variable))
        .map(|variable| {
            format!(
                "variable d'environnement refusée pour l'équipier ACP : {variable} \
                 (utilisez BRIDGET_ALLOW_API_KEY=1 uniquement si vous acceptez la facturation API)"
            )
        })
}

fn send_wrapper_message(
    writer: &Arc<Mutex<Option<BufWriter<UnixStream>>>>,
    message: WrapperToDaemon,
) {
    let Ok(json) = encode(&message) else {
        return;
    };
    let write_result = writer
        .lock()
        .unwrap_or_else(|err| err.into_inner())
        .as_mut()
        .map(|writer| writeln!(writer, "{}", json).and_then(|_| writer.flush()));
    if let Some(Err(error)) = write_result {
        warn!("envoi wrapper ACP impossible: {}", error);
    }
}

#[allow(clippy::too_many_arguments)]
fn reconnect_acp(
    socket: &std::path::Path,
    writer: &Arc<Mutex<Option<BufWriter<UnixStream>>>>,
    transport: &AcpTransport,
    agent_type: &str,
    name_state_path: &std::path::Path,
    host: &str,
    os: &str,
    instance_id: &str,
    fallback_name: &str,
) -> Option<(BufReader<UnixStream>, String)> {
    let mut attempts = 0_u32;
    while transport.is_alive() {
        thread::sleep(reconnect_delay(attempts));
        attempts = attempts.saturating_add(1);
        let wanted_name = resolve_current_name(name_state_path, fallback_name);
        let busy = matches!(
            transport.state(),
            bridget_transport::TurnState::InProgress { .. }
        );
        match connect_and_register_at(
            socket,
            agent_type,
            Some(&wanted_name),
            host,
            "acp",
            os,
            instance_id,
            effective_domain(&wanted_name).as_deref(),
            busy,
        ) {
            Ok((reader, new_writer, registered_name)) if registered_name == wanted_name => {
                *writer.lock().unwrap_or_else(|error| error.into_inner()) = Some(new_writer);
                return Some((reader, registered_name));
            }
            Ok((_, _, registered_name)) => warn!(
                "reconnexion ACP refusée : nom inattendu « {} »",
                registered_name
            ),
            Err(error) => warn!(
                "reconnexion ACP de « {} » impossible (tentative {}) : {}",
                wanted_name, attempts, error
            ),
        }
    }
    None
}

fn forward_acp_events(
    writer: &Arc<Mutex<Option<BufWriter<UnixStream>>>>,
    my_name: &str,
    events: Vec<AcpEvent>,
    idempotent_deliveries: &mut IdempotentDeliveryTracker,
) -> bool {
    let mut journal_failed = false;
    for event in events {
        match event {
            AcpEvent::TurnStarted { .. } => {
                send_wrapper_message(writer, WrapperToDaemon::TurnState { in_progress: true })
            }
            AcpEvent::TurnFinished {
                message,
                response,
                stop_reason,
            } => {
                send_wrapper_message(writer, WrapperToDaemon::TurnState { in_progress: false });
                if stop_reason_is_error(&stop_reason) {
                    send_wrapper_message(
                        writer,
                        WrapperToDaemon::DeliveryRejected {
                            id: message.id,
                            reason: format!("stopReason ACP d'erreur : {stop_reason}"),
                        },
                    );
                } else if message.reply && !response.is_empty() {
                    let mut reply =
                        bridget_core::BridgetMessage::new(my_name, &message.from, response);
                    reply.in_reply_to = Some(message.id);
                    send_wrapper_message(writer, WrapperToDaemon::Send(reply));
                } else if message.reply {
                    send_wrapper_message(
                        writer,
                        WrapperToDaemon::DeliveryRejected {
                            id: message.id,
                            reason: "réponse vide".to_string(),
                        },
                    );
                }
            }
            AcpEvent::DeliveryRejected { message_id, reason } => {
                send_wrapper_message(writer, WrapperToDaemon::TurnState { in_progress: false });
                if let Some(report) = idempotent_deliveries.injection_rejected(&message_id) {
                    send_wrapper_message(writer, report);
                }
                send_wrapper_message(
                    writer,
                    WrapperToDaemon::DeliveryRejected {
                        id: message_id,
                        reason,
                    },
                );
            }
            AcpEvent::JournalFailed { detail } => {
                journal_failed = true;
                warn!("arrêt du transport ACP : {detail}");
            }
            AcpEvent::PromptDispatched { message_id } => {
                if let Some(report) =
                    idempotent_deliveries.prompt_dispatched(&message_id, unix_now_secs())
                {
                    send_wrapper_message(writer, report);
                }
            }
            AcpEvent::Update { .. } | AcpEvent::Error { .. } => {}
        }
    }
    journal_failed
}

fn stop_reason_is_error(stop_reason: &str) -> bool {
    matches!(stop_reason, "error" | "failed" | "failure")
}

#[cfg(test)]
fn journal_failure_requires_shutdown(events: &[AcpEvent]) -> bool {
    events
        .iter()
        .any(|event| matches!(event, AcpEvent::JournalFailed { .. }))
}

#[cfg(test)]
mod reconnect_tests {
    use super::*;

    fn relay_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("bridget-relay-{name}-{}", std::process::id()))
    }

    fn receipt_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "bridget-receipt-wrapper-{name}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ))
    }

    fn idempotent_message(id: &str) -> bridget_core::BridgetMessage {
        let mut message = bridget_core::BridgetMessage::new("maicie", "equipier", "tâche");
        message.id = id.to_string();
        message
    }

    #[test]
    fn redelivery_after_seen_never_injects_a_second_prompt() {
        let root = receipt_root("seen");
        let instance_id = "instance_012_aaaaaaaaaaaa";
        let mut tracker = IdempotentDeliveryTracker::open_at(&root, instance_id).unwrap();
        let first_action = tracker.receive(
            "delivery_seen".to_string(),
            instance_id.to_string(),
            17,
            500,
            idempotent_message("prompt-unique"),
            100,
        );
        assert!(matches!(
            first_action,
            IdempotentDeliveryAction::Inject { .. }
        ));

        // Simule le redémarrage du daemon après remise mais avant son issue :
        // le wrapper vivant revoit la même remise et ne peut pas la réinjecter.
        let redelivery = tracker.receive(
            "delivery_seen".to_string(),
            instance_id.to_string(),
            17,
            500,
            idempotent_message("prompt-unique"),
            101,
        );
        assert!(matches!(
            redelivery,
            IdempotentDeliveryAction::Report(WrapperToDaemon::DeliveryIndeterminate {
                delivery_id,
                delivery_generation: 17,
            }) if delivery_id == "delivery_seen"
        ));
        drop(tracker);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn replaced_instance_is_indeterminate_without_injection() {
        let root = receipt_root("replaced");
        let mut tracker =
            IdempotentDeliveryTracker::open_at(&root, "instance_012_newwrapper").unwrap();
        let action = tracker.receive(
            "delivery_replaced".to_string(),
            "instance_012_oldwrapper".to_string(),
            29,
            500,
            idempotent_message("prompt-replaced"),
            100,
        );
        assert!(matches!(
            action,
            IdempotentDeliveryAction::Report(WrapperToDaemon::DeliveryIndeterminate {
                delivery_id,
                delivery_generation: 29,
            }) if delivery_id == "delivery_replaced"
        ));
        drop(tracker);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn prompt_dispatched_persists_the_ack_before_reporting_it() {
        let root = receipt_root("ack");
        let instance_id = "instance_012_aaaaaaaaaaaa";
        let mut tracker = IdempotentDeliveryTracker::open_at(&root, instance_id).unwrap();
        assert!(matches!(
            tracker.receive(
                "delivery_acked".to_string(),
                instance_id.to_string(),
                31,
                500,
                idempotent_message("prompt-acked"),
                100,
            ),
            IdempotentDeliveryAction::Inject { .. }
        ));
        assert!(matches!(
            tracker.prompt_dispatched("prompt-acked", 101),
            Some(WrapperToDaemon::DeliverAcked {
                delivery_id,
                delivery_generation: 31,
            }) if delivery_id == "delivery_acked"
        ));
        assert!(matches!(
            tracker.receive(
                "delivery_acked".to_string(),
                instance_id.to_string(),
                31,
                500,
                idempotent_message("prompt-acked"),
                102,
            ),
            IdempotentDeliveryAction::Report(WrapperToDaemon::DeliverAcked { .. })
        ));
        drop(tracker);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn same_message_id_from_two_scopes_keeps_each_delivery_acknowledgement() {
        let root = receipt_root("same-message-id");
        let instance_id = "instance_012_aaaaaaaaaaaa";
        let mut tracker = IdempotentDeliveryTracker::open_at(&root, instance_id).unwrap();
        for (delivery_id, generation) in [("delivery_scope_a", 41), ("delivery_scope_b", 43)] {
            assert!(matches!(
                tracker.receive(
                    delivery_id.to_string(),
                    instance_id.to_string(),
                    generation,
                    500,
                    idempotent_message("same-client-message-id"),
                    100,
                ),
                IdempotentDeliveryAction::Inject { .. }
            ));
        }
        assert!(matches!(
            tracker.prompt_dispatched("same-client-message-id", 101),
            Some(WrapperToDaemon::DeliverAcked {
                delivery_id,
                delivery_generation: 41,
            }) if delivery_id == "delivery_scope_a"
        ));
        assert!(matches!(
            tracker.prompt_dispatched("same-client-message-id", 102),
            Some(WrapperToDaemon::DeliverAcked {
                delivery_id,
                delivery_generation: 43,
            }) if delivery_id == "delivery_scope_b"
        ));
        drop(tracker);
        std::fs::remove_dir_all(root).unwrap();
    }

    fn relay_emitter() -> (RelayEvents, RelayEmitter) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = events.clone();
        let emitter: Arc<dyn Fn(WrapperToDaemon) + Send + Sync> = Arc::new(move |message| {
            captured
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .push(message);
        });
        (events, emitter)
    }

    fn wait_for(condition: impl Fn() -> bool) {
        for _ in 0..100 {
            if condition() {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("condition du worker de relais non atteinte dans la borne de test");
    }

    fn acp_definition(forbidden_env: &[&str]) -> crate::registry::AgentDefinition {
        crate::registry::AgentDefinition {
            command: "adapter".to_string(),
            args: Vec::new(),
            protocol: "acp".to_string(),
            forbidden_env: forbidden_env.iter().map(ToString::to_string).collect(),
            permissions: "allow".to_string(),
            queue_capacity: 32,
            notify_timeout_secs: 600,
        }
    }

    #[test]
    fn journal_failure_requires_an_immediate_transport_shutdown() {
        assert!(journal_failure_requires_shutdown(&[
            AcpEvent::JournalFailed {
                detail: "journal ACP saturé".to_string(),
            }
        ]));
        assert!(!journal_failure_requires_shutdown(&[AcpEvent::Error {
            detail: "diagnostic non terminal".to_string(),
        }]));
    }

    #[test]
    fn api_key_forbidden_refuse_le_lancement_en_nommant_la_variable() {
        let error = forbidden_env_error(
            &acp_definition(&["OPENAI_API_KEY", "CODEX_API_KEY"]),
            false,
            |variable| matches!(variable, "OPENAI_API_KEY" | "CODEX_API_KEY"),
        )
        .expect("clé API refusée");
        assert!(error.contains("OPENAI_API_KEY"));
        assert!(!error.contains("CODEX_API_KEY"));
        assert!(error.contains("BRIDGET_ALLOW_API_KEY=1"));
    }

    #[test]
    fn api_key_forbidden_utilise_la_seconde_variable_si_elle_est_seule() {
        let error = forbidden_env_error(
            &acp_definition(&["OPENAI_API_KEY", "CODEX_API_KEY"]),
            false,
            |variable| variable == "CODEX_API_KEY",
        )
        .expect("seconde clé API refusée");
        assert!(error.contains("CODEX_API_KEY"));
    }

    #[test]
    fn api_key_forbidden_accepte_le_contournement_explicite() {
        assert!(
            forbidden_env_error(
                &acp_definition(&["ANTHROPIC_API_KEY"]),
                true,
                |variable| variable == "ANTHROPIC_API_KEY",
            )
            .is_none()
        );
    }

    #[test]
    fn seul_le_contournement_egal_a_un_est_accepte() {
        assert!(allow_api_key_value(Some("1")));
        for value in [None, Some("0"), Some("true"), Some("01")] {
            assert!(
                !allow_api_key_value(value),
                "valeur non autorisée: {value:?}"
            );
        }
    }

    #[test]
    fn equipier_flag_is_removed_before_the_agent_is_started() {
        let (equipier, remaining) = split_equipier_flag(&[
            "--equipier".to_string(),
            "resume".to_string(),
            "session".to_string(),
        ]);
        assert!(equipier);
        assert_eq!(remaining, vec!["resume", "session"]);
    }

    #[test]
    fn le_domaine_derive_nomme_le_depot_courant() {
        // La règle vérifiée est « nom de la racine du dépôt », et non « chemin
        // complet » ni « répertoire courant ». On ne peut pas comparer à une
        // chaîne en dur : le dépôt peut être cloné sous n'importe quel nom, et
        // les tests s'exécutent aussi depuis un sous-répertoire.
        let racine = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .expect("git doit être disponible pour ce test");
        let racine = PathBuf::from(String::from_utf8_lossy(&racine.stdout).trim().to_string());
        let attendu = racine
            .file_name()
            .map(|name| name.to_string_lossy().to_string());

        assert_eq!(derive_domain(), attendu);
        // Le domaine est un nom court, jamais un chemin.
        let domaine = derive_domain().unwrap();
        assert!(
            !domaine.contains('/'),
            "le domaine ne doit pas être un chemin"
        );
    }

    #[test]
    fn le_domaine_surcharge_prime_sur_le_derive() {
        let path = domain_state_path("agent-de-test-domaine");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, "revue-croisee\n").unwrap();
        assert_eq!(
            effective_domain("agent-de-test-domaine").as_deref(),
            Some("revue-croisee")
        );

        // Sans trace disque, on retombe sur le domaine dérivé.
        std::fs::remove_file(&path).unwrap();
        assert_eq!(effective_domain("agent-de-test-domaine"), derive_domain());
    }

    #[test]
    fn le_nom_choisi_par_l_utilisateur_survit_a_la_reconnexion() {
        // `bridget rename` n'écrit que dans ce fichier ; le wrapper doit s'y
        // référer, sinon un agent renommé revient sous son nom d'origine à
        // chaque coupure — y compris une rupture de tunnel en fédération SSH.
        let path =
            std::env::temp_dir().join(format!("bridget-nom-{}-{}", std::process::id(), "renomme"));
        std::fs::write(&path, "agent-1\n").unwrap();
        assert_eq!(resolve_current_name(&path, "codex-8"), "agent-1");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn nom_illisible_ou_vide_retombe_sur_le_nom_d_enregistrement() {
        let absent = std::env::temp_dir().join("bridget-nom-inexistant-xyz");
        let _ = std::fs::remove_file(&absent);
        assert_eq!(resolve_current_name(&absent, "codex-8"), "codex-8");

        let vide = std::env::temp_dir().join(format!("bridget-nom-vide-{}", std::process::id()));
        std::fs::write(&vide, "   \n").unwrap();
        assert_eq!(resolve_current_name(&vide, "codex-8"), "codex-8");
        let _ = std::fs::remove_file(&vide);
    }

    #[test]
    fn relais_ne_lit_jamais_le_journal_sans_abonne() {
        let root = relay_root("sans-abonne");
        std::fs::create_dir_all(&root).unwrap();
        let reads = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let probe = reads.clone();
        let hooks = AttachRelayHooks {
            before_command: Arc::new(|| {}),
            before_read: Arc::new(move || {
                probe.fetch_add(1, Ordering::SeqCst);
            }),
            ..AttachRelayHooks::default()
        };
        let (_, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            1,
            emitter,
            hooks,
        );
        thread::sleep(Duration::from_millis(30));
        assert_eq!(reads.load(Ordering::SeqCst), 0);
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn relais_refuse_la_saturation_et_le_controle_passe_pendant_un_rejeu_suspendu() {
        let root = relay_root("controle");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("2026-08-22.jsonl"), b"{\"v\":1,\"seq\":1}\n").unwrap();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let first_read = Arc::new(AtomicBool::new(true));
        let barrier_for_worker = barrier.clone();
        let first_for_worker = first_read.clone();
        let (control_sender, control_receiver) = mpsc::channel();
        let hooks = AttachRelayHooks {
            before_command: Arc::new(|| {}),
            before_read: Arc::new(move || {
                if first_for_worker.swap(false, Ordering::SeqCst) {
                    barrier_for_worker.wait();
                    barrier_for_worker.wait();
                }
            }),
            control_observed: Arc::new(move || {
                let _ = control_sender.send(());
            }),
        };
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            1,
            emitter,
            hooks,
        );
        worker
            .subscribe("sub-1".to_string(), AttachWindow::Today)
            .unwrap();
        barrier.wait();
        worker
            .subscribe("sub-2".to_string(), AttachWindow::Today)
            .unwrap();
        assert_eq!(
            worker.subscribe("sub-3".to_string(), AttachWindow::Today),
            Err(AttachRefusal::CommandQueueSaturated)
        );
        worker.unsubscribe("sub-1".to_string());
        control_receiver
            .recv_timeout(Duration::from_millis(250))
            .expect("désabonnement reçu par le canal de contrôle");
        barrier.wait();
        thread::sleep(Duration::from_millis(30));
        assert!(!events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, .. } if subscription_id == "sub-1")));
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn relais_marque_une_fenetre_vide_sans_through_seq() {
        let root = relay_root("vide");
        std::fs::create_dir_all(&root).unwrap();
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            1,
            emitter,
            AttachRelayHooks::default(),
        );
        worker
            .subscribe("sub-vide".to_string(), AttachWindow::Today)
            .unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::SnapshotCaughtUp { subscription_id, through_seq: None } if subscription_id == "sub-vide"))
        });
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn relais_rejoue_la_rotation_par_tranches_et_conserve_la_continuite() {
        let root = relay_root("rotation");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("2026-08-22.jsonl"), b"{\"v\":1,\"seq\":5}\n").unwrap();
        std::fs::write(root.join("2026-08-23.jsonl"), b"{\"v\":1,\"seq\":6}\n").unwrap();
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-23".to_string(),
            1,
            emitter,
            AttachRelayHooks::default(),
        );
        worker
            .subscribe("sub-rotation".to_string(), AttachWindow::Seq(5))
            .unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::SnapshotCaughtUp { subscription_id, through_seq: Some(6) } if subscription_id == "sub-rotation"))
        });
        std::fs::write(root.join("2026-08-24.jsonl"), b"{\"v\":1,\"seq\":7}\n").unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { seq: 7, subscription_id, .. } if subscription_id == "sub-rotation"))
        });
        let sequences = events
            .lock()
            .unwrap()
            .iter()
            .filter_map(|message| match message {
                WrapperToDaemon::JournalFragment {
                    seq,
                    subscription_id,
                    final_fragment: true,
                    ..
                } if subscription_id == "sub-rotation" => Some(*seq),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(sequences, vec![5, 6, 7]);
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unsubscribe_avant_consommation_annule_le_subscribe_sans_fuite() {
        let root = relay_root("unsubscribe-avant-consommation");
        std::fs::create_dir_all(&root).unwrap();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let first_command = Arc::new(AtomicBool::new(true));
        let barrier_for_worker = barrier.clone();
        let first_for_worker = first_command.clone();
        let hooks = AttachRelayHooks {
            before_command: Arc::new(move || {
                if first_for_worker.swap(false, Ordering::SeqCst) {
                    barrier_for_worker.wait();
                    barrier_for_worker.wait();
                }
            }),
            ..AttachRelayHooks::default()
        };
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            1,
            emitter,
            hooks,
        );
        barrier.wait();
        worker
            .subscribe("sub-annule".to_string(), AttachWindow::Today)
            .unwrap();
        worker.unsubscribe("sub-annule".to_string());
        for index in 0..(ATTACH_RELAY_CONTROL_IDS_CAPACITY * 4) {
            worker.unsubscribe("sub-annule".to_string());
            worker.unsubscribe(format!("inconnu-{index}"));
        }
        let state = worker
            .control_state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        assert_eq!(state.pending.len(), 1);
        assert!(state.active.is_empty());
        assert_eq!(state.cancelled.len(), 1);
        drop(state);
        barrier.wait();
        thread::sleep(Duration::from_millis(30));
        assert!(!events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::Subscribed { subscription_id } if subscription_id == "sub-annule")));
        thread::sleep(Duration::from_millis(30));
        let state = worker
            .control_state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        assert!(state.pending.is_empty());
        assert!(state.active.is_empty());
        assert!(state.cancelled.is_empty());
        drop(state);
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn shutdown_reste_borne_malgre_un_spam_de_commandes() {
        let root = relay_root("stop-prioritaire");
        std::fs::create_dir_all(&root).unwrap();
        let (_, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            1,
            emitter,
            AttachRelayHooks::default(),
        );
        let sender = worker.commands.clone();
        let running = Arc::new(AtomicBool::new(true));
        let sending = running.clone();
        let spam = thread::spawn(move || {
            let mut index = 0_u64;
            while sending.load(Ordering::SeqCst) {
                let _ = sender.try_send(AttachRelayCommand::Subscribe {
                    subscription_id: format!("spam-{index}"),
                    window: AttachWindow::Today,
                });
                index = index.saturating_add(1);
            }
        });
        let started = std::time::Instant::now();
        worker.shutdown();
        running.store(false, Ordering::SeqCst);
        spam.join().unwrap();
        assert!(
            started.elapsed() < Duration::from_millis(250),
            "le Stop ne doit pas attendre la file saturée"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lignes_illisibles_et_trop_grandes_signalent_puis_laissent_progresser() {
        let root = relay_root("diagnostics-journal");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("2026-08-22.jsonl");
        std::fs::write(&path, b"{invalide}\n").unwrap();
        let mut too_large = b"{\"v\":1,\"seq\":8,\"payload\":\"".to_vec();
        too_large.extend(std::iter::repeat_n(b'x', 4 * 1024 * 1024));
        too_large.extend_from_slice(b"\"}\n{\"v\":1,\"seq\":9}\n");
        std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(&too_large)
            .unwrap();
        let (events, emitter) = relay_emitter();
        let reads = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let read_probe = reads.clone();
        let hooks = AttachRelayHooks {
            before_read: Arc::new(move || {
                read_probe.fetch_add(1, Ordering::SeqCst);
            }),
            ..AttachRelayHooks::default()
        };
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            2,
            emitter,
            hooks,
        );
        worker
            .subscribe("sub-diagnostic".to_string(), AttachWindow::Today)
            .unwrap();
        wait_for(|| {
            let messages = events.lock().unwrap();
            messages.iter().any(|message| matches!(message, WrapperToDaemon::JournalReadError { subscription_id, .. } if subscription_id == "sub-diagnostic"))
                && messages.iter().any(|message| matches!(message, WrapperToDaemon::Gap { from_seq: 8, to_seq: 8, reason: Some(reason), .. } if reason == "event_too_large"))
                && messages.iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { seq: 9, final_fragment: true, .. }))
        });
        assert!(reads.load(Ordering::SeqCst) > 1);
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn queue_partielle_corrompue_devient_un_diagnostic_sans_bloquer_le_suivi() {
        let root = relay_root("queue-partielle");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("2026-08-22.jsonl");
        std::fs::write(&path, b"{incomplet").unwrap();
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            1,
            emitter,
            AttachRelayHooks::default(),
        );
        worker
            .subscribe("sub-partiel".to_string(), AttachWindow::Today)
            .unwrap();
        thread::sleep(Duration::from_millis(30));
        assert!(
            !events
                .lock()
                .unwrap()
                .iter()
                .any(|message| matches!(message, WrapperToDaemon::JournalReadError { .. }))
        );
        std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(b"\n{\"v\":1,\"seq\":10}\n")
            .unwrap();
        wait_for(|| {
            let messages = events.lock().unwrap();
            messages.iter().any(|message| matches!(message, WrapperToDaemon::JournalReadError { subscription_id, .. } if subscription_id == "sub-partiel"))
                && messages.iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 10, final_fragment: true, .. } if subscription_id == "sub-partiel"))
        });
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn today_et_date_suivent_la_rotation_apres_le_snapshot() {
        let root = relay_root("selecteurs-rotation");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("2026-08-22.jsonl"), b"{\"v\":1,\"seq\":1}\n").unwrap();
        let date = Arc::new(Mutex::new("2026-08-22".to_string()));
        let clock_date = date.clone();
        let clock: Arc<dyn Fn() -> String + Send + Sync> =
            Arc::new(move || clock_date.lock().unwrap().clone());
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with_clock(
            root.clone(),
            clock,
            4,
            emitter,
            AttachRelayHooks::default(),
        );
        worker
            .subscribe("sub-today".to_string(), AttachWindow::Today)
            .unwrap();
        worker
            .subscribe(
                "sub-date".to_string(),
                AttachWindow::Date("2026-08-22".to_string()),
            )
            .unwrap();
        wait_for(|| {
            events
                .lock()
                .unwrap()
                .iter()
                .filter(|message| matches!(message, WrapperToDaemon::SnapshotCaughtUp { .. }))
                .count()
                == 2
        });
        *date.lock().unwrap() = "2026-08-23".to_string();
        std::fs::write(root.join("2026-08-23.jsonl"), b"{\"v\":1,\"seq\":2}\n").unwrap();
        wait_for(|| {
            let messages = events.lock().unwrap();
            ["sub-today", "sub-date"].iter().all(|subscription_id| messages.iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id: actual, seq: 2, final_fragment: true, .. } if actual == subscription_id)))
        });
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn today_vide_ne_rejoue_pas_l_historique_anterieur() {
        let root = relay_root("today-vide");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("2026-08-21.jsonl"), b"{\"v\":1,\"seq\":1}\n").unwrap();
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            1,
            emitter,
            AttachRelayHooks::default(),
        );
        worker
            .subscribe("sub-today-vide".to_string(), AttachWindow::Today)
            .unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::SnapshotCaughtUp { subscription_id, through_seq: None } if subscription_id == "sub-today-vide"))
        });
        std::fs::write(root.join("2026-08-22.jsonl"), b"{\"v\":1,\"seq\":2}\n").unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 2, final_fragment: true, .. } if subscription_id == "sub-today-vide"))
        });
        assert!(!events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 1, .. } if subscription_id == "sub-today-vide")));
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn troncature_et_remplacement_repartent_du_nouvel_etat() {
        let root = relay_root("source-reinitialisee");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("2026-08-22.jsonl");
        std::fs::write(&path, b"{\"v\":1,\"seq\":1,\"padding\":\"longue\"}\n").unwrap();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let arm = Arc::new(AtomicBool::new(false));
        let barrier_worker = barrier.clone();
        let arm_worker = arm.clone();
        let hooks = AttachRelayHooks {
            before_read: Arc::new(move || {
                if arm_worker.swap(false, Ordering::SeqCst) {
                    barrier_worker.wait();
                    barrier_worker.wait();
                }
            }),
            ..AttachRelayHooks::default()
        };
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            1,
            emitter,
            hooks,
        );
        worker
            .subscribe("sub-reset".to_string(), AttachWindow::Today)
            .unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 1, final_fragment: true, .. } if subscription_id == "sub-reset"))
        });

        arm.store(true, Ordering::SeqCst);
        barrier.wait();
        std::fs::write(&path, b"{\"v\":1,\"seq\":2}\n").unwrap();
        barrier.wait();
        wait_for(|| {
            let messages = events.lock().unwrap();
            messages.iter().any(|message| matches!(message, WrapperToDaemon::End { subscription_id, reason } if subscription_id == "sub-reset" && reason == "source_truncated"))
                && !messages.iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 2, .. } if subscription_id == "sub-reset"))
        });
        worker
            .subscribe("sub-troncature-reprise".to_string(), AttachWindow::Seq(2))
            .unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 2, final_fragment: true, .. } if subscription_id == "sub-troncature-reprise"))
        });

        arm.store(true, Ordering::SeqCst);
        barrier.wait();
        let replaced = root.join("previous.jsonl");
        std::fs::rename(&path, replaced).unwrap();
        std::fs::write(&path, b"{\"v\":1,\"seq\":3}\n").unwrap();
        barrier.wait();
        wait_for(|| {
            let messages = events.lock().unwrap();
            messages.iter().any(|message| matches!(message, WrapperToDaemon::End { subscription_id, reason } if subscription_id == "sub-troncature-reprise" && reason == "source_replaced"))
                && !messages.iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 3, .. } if subscription_id == "sub-troncature-reprise"))
        });
        worker
            .subscribe("sub-remplacement-reprise".to_string(), AttachWindow::Seq(3))
            .unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 3, final_fragment: true, .. } if subscription_id == "sub-remplacement-reprise"))
        });
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn disparition_d_un_fichier_termine_l_abonnement_sans_boucle() {
        let root = relay_root("source-supprimee");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("2026-08-22.jsonl");
        std::fs::write(&path, b"{\"v\":1,\"seq\":1}\n").unwrap();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let once = Arc::new(AtomicBool::new(true));
        let barrier_worker = barrier.clone();
        let once_worker = once.clone();
        let hooks = AttachRelayHooks {
            before_read: Arc::new(move || {
                if once_worker.swap(false, Ordering::SeqCst) {
                    barrier_worker.wait();
                    barrier_worker.wait();
                }
            }),
            ..AttachRelayHooks::default()
        };
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            1,
            emitter,
            hooks,
        );
        worker
            .subscribe("sub-source".to_string(), AttachWindow::Today)
            .unwrap();
        barrier.wait();
        std::fs::remove_file(path).unwrap();
        barrier.wait();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::End { subscription_id, reason } if subscription_id == "sub-source" && reason.contains("source de journal indisponible")))
        });
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn sorties_des_abonnements_restent_equitablement_entrelacees() {
        let root = relay_root("equite-sortie");
        std::fs::create_dir_all(&root).unwrap();
        let mut large = b"{\"v\":1,\"seq\":1,\"payload\":\"".to_vec();
        large.extend(std::iter::repeat_n(b'a', MAX_ATTACH_FRAGMENT_BYTES * 4));
        large.extend_from_slice(b"\"}\n");
        std::fs::write(root.join("2026-08-22.jsonl"), large).unwrap();
        std::fs::write(root.join("2026-08-23.jsonl"), b"").unwrap();
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-23".to_string(),
            4,
            emitter,
            AttachRelayHooks::default(),
        );
        worker
            .subscribe("a-gros".to_string(), AttachWindow::Seq(0))
            .unwrap();
        worker
            .subscribe(
                "b-live".to_string(),
                AttachWindow::Date("2026-08-23".to_string()),
            )
            .unwrap();
        wait_for(|| {
            let messages = events.lock().unwrap();
            messages.iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 1, .. } if subscription_id == "a-gros"))
                && messages.iter().any(|message| matches!(message, WrapperToDaemon::SnapshotCaughtUp { subscription_id, .. } if subscription_id == "b-live"))
        });
        std::fs::write(root.join("2026-08-24.jsonl"), b"{\"v\":1,\"seq\":2}\n").unwrap();
        wait_for(|| {
            let messages = events.lock().unwrap();
            messages.iter().filter(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 1, .. } if subscription_id == "a-gros")).count() >= 2
                && messages.iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 2, .. } if subscription_id == "b-live"))
                && messages.iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 1, final_fragment: true, .. } if subscription_id == "a-gros"))
        });
        let messages = events.lock().unwrap();
        let b = messages.iter().position(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 2, .. } if subscription_id == "b-live")).expect("fragment B");
        let a_final = messages.iter().position(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 1, final_fragment: true, .. } if subscription_id == "a-gros")).expect("final A");
        assert!(b < a_final, "B doit passer avant le dernier fragment de A");
        drop(messages);
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reset_generation_purge_le_rejeu_avant_un_nouvel_abonnement() {
        let root = relay_root("reset-generation");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("2026-08-22.jsonl"), b"{\"v\":1,\"seq\":1}\n").unwrap();
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            2,
            emitter,
            AttachRelayHooks::default(),
        );
        worker
            .subscribe("ancienne-generation".to_string(), AttachWindow::Today)
            .unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 1, .. } if subscription_id == "ancienne-generation"))
        });
        worker.reset_generation();
        std::fs::write(root.join("2026-08-23.jsonl"), b"{\"v\":1,\"seq\":2}\n").unwrap();
        thread::sleep(Duration::from_millis(50));
        assert!(!events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 2, .. } if subscription_id == "ancienne-generation")));
        worker
            .subscribe("nouvelle-generation".to_string(), AttachWindow::Seq(2))
            .unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 2, final_fragment: true, .. } if subscription_id == "nouvelle-generation"))
        });
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reconnect_delay_grows_then_stays_capped() {
        let first = reconnect_delay(0);
        let second = reconnect_delay(1);
        let sixth = reconnect_delay(5);
        let later = reconnect_delay(20);

        assert!(first <= Duration::from_millis(1200));
        assert!(second >= Duration::from_millis(1600));
        assert!(sixth <= RECONNECT_MAX_DELAY);
        assert!(later <= RECONNECT_MAX_DELAY);
    }
}
