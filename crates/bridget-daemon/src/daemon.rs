//! Daemon bridget — écoute sur socket locale Unix, route les messages
//! entre les wrappers connectés, persiste l'état en SQLite.

use bridget_core::{CircuitBreaker, Deduplicator, EnvelopeGuard, Router, RouterAction};
use bridget_transport::protocol::{
    AttachRefusal, CLIENT_CONTRACT_VERSION, ClientCapability, ClientRefusal, ConnectionRole,
    IdempotencyIssue, decode, encode,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use log::{error, info, warn};
use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::idempotency::{
    IdempotencyKey, IdempotencyStore, LookupResult, OperationKind, Reservation,
    ReplyTracking, SendDelivery,
};
use crate::store::Store;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use uuid::Uuid;

// Métriques du daemon (M-005)
pub struct Metrics {
    pub messages_sent: AtomicU64,
    pub messages_received: AtomicU64,
    pub errors: AtomicU64,
    pub active_connections: AtomicUsize,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            messages_sent: AtomicU64::new(0),
            messages_received: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            active_connections: AtomicUsize::new(0),
        }
    }

    pub fn increment_sent(&self) {
        self.messages_sent.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_received(&self) {
        self.messages_received.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_errors(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_connections(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_connections(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }
}

static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);
const PRESENCE_RETENTION: Duration = Duration::from_secs(300);
const ATTACH_VIEW_BUFFER_BYTES: usize = 1024 * 1024;
const ATTACH_VIEW_WRITE_TIMEOUT: Duration = Duration::from_secs(1);
/// Plafond global (toutes connexions attach confondues) des accusés tardifs.
/// Chaque entrée expire aussi après `PENDING_ATTACH_SEND_TTL`.
const MAX_PENDING_ATTACH_SENDS: usize = 1024;
const PENDING_ATTACH_SEND_TTL: Duration = Duration::from_secs(300);
const CLIENT_IDEMPOTENCY_HORIZON_SECS: i64 = 7 * 24 * 60 * 60;
const CLIENT_ISSUED_AT_TOLERANCE_SECS: i64 = 60;
const MAX_ACTIVE_ISSUER_SCOPES: usize = 4096;

// Constante pour la période de grâce des timeouts (M-004)
const TIMEOUT_GRACE_PERIOD: u64 = 30; // secondes

#[derive(Clone)]
struct Presence {
    name: String,
    agent_type: String,
    host: String,
    transport: String,
    os: String,
    state: String,
    last_seen: Instant,
    reconnect_count: u32,
    /// Modèle courant, `None` tant qu'aucune observation n'a eu lieu.
    model: Option<String>,
    /// Niveau d'effort courant, `None` si jamais observé ou observé absent.
    effort: Option<String>,
    /// Domaine dérivé annoncé à l'enregistrement, conservé pour pouvoir revenir
    /// dessus après une surcharge.
    derived_domain: Option<String>,
    /// Domaine effectif : la surcharge si elle existe, le domaine dérivé sinon.
    domain: Option<String>,
    /// Échéance jusqu'à laquelle l'agent refuse d'être dérangé.
    ///
    /// Une échéance plutôt qu'un booléen : l'expiration devient une simple
    /// comparaison à la lecture, sans tâche de fond pour balayer les statuts.
    dnd_until: Option<Instant>,
}

impl Presence {
    /// Vrai tant que l'agent refuse d'être dérangé.
    fn is_dnd(&self) -> bool {
        self.dnd_until
            .map(|until| Instant::now() < until)
            .unwrap_or(false)
    }

    /// Minutes restantes de refus, arrondies au supérieur, minimum 1.
    fn dnd_minutes_left(&self) -> u64 {
        self.dnd_until
            .and_then(|until| until.checked_duration_since(Instant::now()))
            .map(|left| left.as_secs().div_ceil(60))
            .unwrap_or(0)
            .max(1)
    }
}

/// Configuration du daemon.
#[derive(Clone)]
pub struct DaemonConfig {
    pub socket_path: PathBuf,
    pub db_path: PathBuf,
    pub log_path: PathBuf,
    pub circuit_breaker_window: u64,
    pub circuit_breaker_limit: usize,
    pub dedup_window: u64,
    pub quarantine_window: u64,
    pub retention_days: u32,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        let cache_dir = dirs_cache();
        DaemonConfig {
            socket_path: cache_dir.join("bridget.sock"),
            db_path: cache_dir.join("bridget.db"),
            log_path: cache_dir.join("daemon.log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        }
    }
}

fn dirs_cache() -> PathBuf {
    let cache_dir = if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".cache").join("bridget")
    } else {
        PathBuf::from("/tmp").join("bridget") // Fallback non sécurisé
    };

    // Créer avec permissions sécurisées (M-003)
    let _ = std::fs::create_dir_all(&cache_dir);

    // Vérifier les permissions (Unix seulement) (M-003)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(&cache_dir) {
            let perms = meta.permissions();
            let mode = perms.mode();
            if mode & 0o077 != 0 {
                warn!(
                    "Permissions non sécurisées sur {:?} - autres utilisateurs peuvent lire/écrire",
                    cache_dir
                );
            }
        }
    }

    cache_dir
}

/// État partagé du daemon.
struct DaemonState {
    router: Router,
    circuit_breaker: CircuitBreaker,
    deduplicator: Deduplicator,
    envelope_guard: EnvelopeGuard,
    store: Store,
    idempotency: IdempotencyStore,
    connections: HashMap<String, Arc<Mutex<BufWriter<UnixStream>>>>,
    conn_names: HashMap<String, String>,
    conn_hosts: HashMap<String, String>,
    conn_operating_systems: HashMap<String, String>,
    conn_instances: HashMap<String, String>,
    /// Les clients attach négocient ce rôle explicite ; l'absence d'entrée
    /// reste un wrapper pour préserver les agents 007 déjà connectés.
    connection_roles: HashMap<String, ConnectionRole>,
    /// Une négociation appartient à la connexion, tandis que le scope peut
    /// volontairement être partagé par plusieurs retries coopératifs.
    client_negotiations: HashMap<String, NegotiatedClient>,
    /// Souscriptions attach actives, distinctes de l'annuaire des équipiers.
    attach_subscriptions: HashMap<String, AttachSubscription>,
    attach_views: HashMap<String, Arc<AttachView>>,
    view_closed_tx: Sender<String>,
    view_closed_rx: Receiver<String>,
    pending_attach_sends: HashMap<String, PendingAttachSend>,
    presences: HashMap<String, Presence>,
    conn_counter: u64,
    /// Messages --reply en attente de réponse : (msg_id, from, to, expire_at, target_conn)
    pending_replies: Vec<PendingReply>,
}

#[derive(Clone)]
struct AttachSubscription {
    agent: String,
    attach_conn: String,
    wrapper_conn: String,
}

struct PendingAttachSend {
    conn_id: String,
    expires_at: Instant,
}

#[derive(Clone)]
struct NegotiatedClient {
    version: u16,
    issuer_scope: String,
    capabilities: Vec<ClientCapability>,
}

struct QueuedAttachMessage {
    encoded: String,
    seq: Option<u64>,
    terminal: bool,
}

struct AttachViewBuffer {
    messages: VecDeque<QueuedAttachMessage>,
    bytes: usize,
    gap: Option<(u64, u64, Option<String>)>,
    /// Séquence évincée dont les fragments suivants doivent encore être
    /// ignorés jusqu'à sa frontière finale.
    dropping_seq: Option<u64>,
    terminal_enqueued: bool,
    closed: bool,
    close_notified: bool,
}

struct AttachView {
    queue: Arc<(Mutex<AttachViewBuffer>, Condvar)>,
    worker: Mutex<Option<thread::JoinHandle<()>>>,
    subscription_id: String,
    closed_tx: Sender<String>,
}

impl AttachView {
    #[cfg(test)]
    fn suspended(_subscription_id: impl Into<String>) -> Arc<Self> {
        Self::suspended_with_notifications(_subscription_id).0
    }

    #[cfg(test)]
    fn suspended_with_notifications(
        subscription_id: impl Into<String>,
    ) -> (Arc<Self>, Receiver<String>) {
        let (closed_tx, closed_rx) = mpsc::channel();
        (
            Self::suspended_with_sender(subscription_id, closed_tx),
            closed_rx,
        )
    }

    #[cfg(test)]
    fn suspended_with_sender(
        subscription_id: impl Into<String>,
        closed_tx: Sender<String>,
    ) -> Arc<Self> {
        Arc::new(Self {
            queue: Arc::new((
                Mutex::new(AttachViewBuffer {
                    messages: VecDeque::new(),
                    bytes: 0,
                    gap: None,
                    dropping_seq: None,
                    terminal_enqueued: false,
                    closed: false,
                    close_notified: false,
                }),
                Condvar::new(),
            )),
            worker: Mutex::new(None),
            subscription_id: subscription_id.into(),
            closed_tx,
        })
    }

    fn start(
        subscription_id: String,
        writer: &Arc<Mutex<BufWriter<UnixStream>>>,
        closed_tx: Sender<String>,
    ) -> Option<Arc<Self>> {
        // Tous les producteurs d'une même connexion (worker de vue et messages
        // de contrôle) partagent ce writer : le mutex sérialise les frames JSONL.
        let _ = writer
            .lock()
            .ok()?
            .get_ref()
            .set_write_timeout(Some(ATTACH_VIEW_WRITE_TIMEOUT));
        let queue = Arc::new((
            Mutex::new(AttachViewBuffer {
                messages: VecDeque::new(),
                bytes: 0,
                gap: None,
                dropping_seq: None,
                terminal_enqueued: false,
                closed: false,
                close_notified: false,
            }),
            Condvar::new(),
        ));
        let view = Arc::new(Self {
            queue: queue.clone(),
            worker: Mutex::new(None),
            subscription_id: subscription_id.clone(),
            closed_tx: closed_tx.clone(),
        });
        let worker_queue = queue.clone();
        let worker_subscription = subscription_id;
        let worker_writer = writer.clone();
        let handle = thread::spawn(move || {
            loop {
                let next = {
                    let (lock, wake) = &*worker_queue;
                    let mut buffer = lock.lock().unwrap_or_else(|e| e.into_inner());
                    while buffer.messages.is_empty() && buffer.gap.is_none() && !buffer.closed {
                        buffer = wake.wait(buffer).unwrap_or_else(|e| e.into_inner());
                    }
                    if let Some((from_seq, to_seq, reason)) = buffer.gap.take() {
                        encode(&DaemonToWrapper::Gap {
                            subscription_id: worker_subscription.clone(),
                            from_seq,
                            to_seq,
                            reason,
                        })
                        .ok()
                        .map(|encoded| (encoded, false))
                    } else if let Some(message) = buffer.messages.pop_front() {
                        buffer.bytes = buffer.bytes.saturating_sub(message.encoded.len());
                        Some((message.encoded, message.terminal))
                    } else {
                        None
                    }
                };
                let Some((next, terminal)) = next else {
                    if worker_queue
                        .0
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .closed
                    {
                        break;
                    }
                    continue;
                };
                let write_failed = worker_writer
                    .lock()
                    .map(|mut writer| {
                        writeln!(writer, "{next}")
                            .and_then(|_| writer.flush())
                            .is_err()
                    })
                    .unwrap_or(true);
                if write_failed {
                    let (lock, wake) = &*worker_queue;
                    let mut buffer = lock.lock().unwrap_or_else(|e| e.into_inner());
                    buffer.closed = true;
                    let notify = !buffer.close_notified;
                    buffer.close_notified = true;
                    drop(buffer);
                    wake.notify_all();
                    if notify {
                        let _ = closed_tx.send(worker_subscription.clone());
                    }
                    break;
                }
                if terminal {
                    let (lock, wake) = &*worker_queue;
                    lock.lock().unwrap_or_else(|e| e.into_inner()).closed = true;
                    wake.notify_all();
                    break;
                }
            }
        });
        *view.worker.lock().unwrap_or_else(|e| e.into_inner()) = Some(handle);
        Some(view)
    }

    fn enqueue(&self, message: DaemonToWrapper) -> bool {
        let (seq, final_fragment, terminal) = match &message {
            DaemonToWrapper::JournalFragment {
                seq,
                final_fragment,
                ..
            } => (Some(*seq), *final_fragment, false),
            DaemonToWrapper::End { .. } => (None, false, true),
            _ => (None, false, false),
        };
        let Ok(encoded) = encode(&message) else {
            self.close_and_notify();
            return false;
        };
        let (lock, wake) = &*self.queue;
        let mut buffer = lock.lock().unwrap_or_else(|e| e.into_inner());
        if buffer.closed || buffer.terminal_enqueued {
            return false;
        }
        if terminal {
            // Le terminal clôt la file : les données restées en attente sont
            // explicitement abandonnées avant End, jamais écrites après lui.
            buffer.messages.clear();
            buffer.bytes = 0;
            buffer.gap = None;
            buffer.dropping_seq = None;
            buffer.terminal_enqueued = true;
            buffer.messages.push_back(QueuedAttachMessage {
                encoded,
                seq: None,
                terminal: true,
            });
            wake.notify_one();
            return true;
        }
        if let Some(seq) = seq {
            if buffer.dropping_seq == Some(seq) {
                if final_fragment {
                    buffer.dropping_seq = None;
                }
                return false;
            }
            while buffer.bytes.saturating_add(encoded.len()) > ATTACH_VIEW_BUFFER_BYTES {
                let Some(dropped_seq) = buffer.messages.iter().find_map(|item| item.seq) else {
                    buffer.dropping_seq = Some(seq);
                    buffer.gap = Some((seq, seq, Some("vue trop lente".to_string())));
                    wake.notify_one();
                    return false;
                };
                let mut retained = VecDeque::new();
                while let Some(item) = buffer.messages.pop_front() {
                    if item.seq == Some(dropped_seq) {
                        buffer.bytes = buffer.bytes.saturating_sub(item.encoded.len());
                    } else {
                        retained.push_back(item);
                    }
                }
                buffer.messages = retained;
                buffer.dropping_seq = Some(dropped_seq);
                buffer.gap = Some(match buffer.gap.take() {
                    Some((from, to, reason)) => {
                        (from.min(dropped_seq), to.max(dropped_seq), reason)
                    }
                    None => (dropped_seq, dropped_seq, Some("vue trop lente".to_string())),
                });
            }
            if buffer.dropping_seq == Some(seq) {
                if final_fragment {
                    buffer.dropping_seq = None;
                }
                return false;
            }
        } else if buffer.bytes.saturating_add(encoded.len()) > ATTACH_VIEW_BUFFER_BYTES {
            buffer.closed = true;
            let notify = !buffer.close_notified;
            buffer.close_notified = true;
            drop(buffer);
            wake.notify_all();
            if notify {
                let _ = self.closed_tx.send(self.subscription_id.clone());
            }
            return false;
        }
        buffer.bytes += encoded.len();
        buffer.messages.push_back(QueuedAttachMessage {
            encoded,
            seq,
            terminal: false,
        });
        wake.notify_one();
        true
    }

    fn close(&self) {
        let (lock, wake) = &*self.queue;
        lock.lock().unwrap_or_else(|e| e.into_inner()).closed = true;
        wake.notify_all();
    }

    fn close_and_notify(&self) {
        let (lock, wake) = &*self.queue;
        let mut buffer = lock.lock().unwrap_or_else(|e| e.into_inner());
        if buffer.closed {
            return;
        }
        buffer.closed = true;
        let notify = !buffer.close_notified;
        buffer.close_notified = true;
        drop(buffer);
        wake.notify_all();
        if notify {
            let _ = self.closed_tx.send(self.subscription_id.clone());
        }
    }

    fn close_and_join(&self) {
        self.close();
        if let Some(worker) = self.worker.lock().unwrap_or_else(|e| e.into_inner()).take() {
            let _ = worker.join();
        }
    }
}

struct PendingReply {
    msg_id: String,
    from: String,
    from_conn: String,
    to: String,
    target_conn: String,
    /// Timeout total en secondes (configurable par l'émetteur)
    timeout_secs: u64,
    created_at: std::time::Instant,
    /// Palier d'escalade atteint : 0 = rien, 1 = rappel discret,
    /// 2 = rappel ferme, 3 = notification échec à l'émetteur
    escalation_level: u8,
    deferred_level: Option<u8>,
}

enum ReminderAction {
    Gentle {
        to: String,
        from: String,
        msg_id: String,
        target_conn: String,
    },
    Firm {
        to: String,
        from: String,
        msg_id: String,
        target_conn: String,
    },
    Timeout {
        to: String,
        from: String,
        msg_id: String,
        from_conn: String,
        timeout_secs: u64,
    },
    Deferred {
        to: String,
        msg_id: String,
        level: u8,
    },
}

// Type d'erreur pour la livraison de messages (H-002)
#[derive(Debug)]
enum DeliveryError {
    Encoding(String),
    Lock(String),
    Write(String),
    Flush(String),
}

impl std::fmt::Display for DeliveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeliveryError::Encoding(msg) => write!(f, "Erreur d'encodage: {}", msg),
            DeliveryError::Lock(msg) => write!(f, "Erreur de verrouillage: {}", msg),
            DeliveryError::Write(msg) => write!(f, "Erreur d'écriture: {}", msg),
            DeliveryError::Flush(msg) => write!(f, "Erreur de flush: {}", msg),
        }
    }
}

fn deliver_to_agent(
    writer: &Arc<Mutex<BufWriter<UnixStream>>>,
    target_name: &str,
    body: &str,
) -> Result<(), DeliveryError> {
    let msg = bridget_core::BridgetMessage::new("bridget", target_name, body);
    let dtw = DaemonToWrapper::Deliver(msg);
    let json = encode(&dtw).map_err(|e| {
        error!("Erreur d'encodage message pour {}: {}", target_name, e);
        DeliveryError::Encoding(e.to_string())
    })?;

    let mut w = writer.lock().map_err(|e| {
        error!(
            "Impossible de verrouiller le writer pour {}: {}",
            target_name, e
        );
        DeliveryError::Lock(e.to_string())
    })?;

    writeln!(w, "{}", json).map_err(|e| {
        error!("Erreur d'écriture pour {}: {}", target_name, e);
        DeliveryError::Write(e.to_string())
    })?;

    w.flush().map_err(|e| {
        error!("Erreur de flush pour {}: {}", target_name, e);
        DeliveryError::Flush(e.to_string())
    })?;

    info!("Message délivré à {}", target_name);
    Ok(())
}

fn push_control_message(
    writer: &Arc<Mutex<BufWriter<UnixStream>>>,
    message: &DaemonToWrapper,
) -> bool {
    let Ok(json) = encode(message) else {
        return false;
    };
    let Ok(mut writer) = writer.lock() else {
        return false;
    };
    let written = writeln!(writer, "{json}")
        .and_then(|_| writer.flush())
        .is_ok();
    if !written && matches!(message, DaemonToWrapper::End { .. }) {
        // Une vue ne doit jamais rester suspendue après l'échec du terminal :
        // l'EOF de la connexion est alors son signal de clôture garanti.
        let _ = writer.get_ref().shutdown(std::net::Shutdown::Both);
    }
    written
}

struct DeferredControl {
    writer: Arc<Mutex<BufWriter<UnixStream>>>,
    message: DaemonToWrapper,
}

fn execute_controls(controls: Vec<DeferredControl>) -> Vec<DeferredControl> {
    controls
        .into_iter()
        .filter(|control| !push_control_message(&control.writer, &control.message))
        .collect()
}

fn defer_control(
    state: &DaemonState,
    conn_id: &str,
    message: DaemonToWrapper,
    controls: &mut Vec<DeferredControl>,
) {
    if let Some(writer) = state.connections.get(conn_id) {
        controls.push(DeferredControl {
            writer: writer.clone(),
            message,
        });
    }
}

fn purge_expired_attach_sends(state: &mut DaemonState) {
    let now = Instant::now();
    state
        .pending_attach_sends
        .retain(|_, pending| pending.expires_at > now);
}

fn agent_uses_acp(state: &DaemonState, agent: &bridget_core::router::RegisteredAgent) -> bool {
    state
        .conn_instances
        .get(&agent.connection_id)
        .and_then(|instance_id| state.presences.get(instance_id))
        .is_some_and(|presence| presence.transport == "acp")
}

fn attach_refusal_for_subscription(
    state: &DaemonState,
    agent: &str,
) -> Result<String, AttachRefusal> {
    let registered = state
        .router
        .get_agent(agent)
        .ok_or(AttachRefusal::AgentUnknown)?;
    if !agent_uses_acp(state, registered) {
        return Err(AttachRefusal::AgentNotAcp);
    }
    if !state.connections.contains_key(&registered.connection_id) {
        return Err(AttachRefusal::WrapperUnavailable);
    }
    Ok(registered.connection_id.clone())
}

/// Retire les souscriptions affectées sous le verrou. Les E/S et les `join`
/// retournés doivent impérativement être exécutés après l'avoir relâché.
fn close_attach_subscriptions(
    state: &mut DaemonState,
    conn_id: &str,
) -> (Vec<DeferredControl>, Vec<Arc<AttachView>>) {
    let mut controls = Vec::new();
    let mut views = Vec::new();
    let affected = state
        .attach_subscriptions
        .iter()
        .filter(|(_, subscription)| {
            subscription.attach_conn == conn_id || subscription.wrapper_conn == conn_id
        })
        .map(|(id, subscription)| (id.clone(), subscription.clone()))
        .collect::<Vec<_>>();
    for (subscription_id, subscription) in affected {
        state.attach_subscriptions.remove(&subscription_id);
        let view = state.attach_views.remove(&subscription_id);
        if subscription.attach_conn == conn_id {
            defer_control(
                state,
                &subscription.wrapper_conn,
                DaemonToWrapper::Unsubscribe { subscription_id },
                &mut controls,
            );
        } else if let Some(view) = &view {
            let _ = view.enqueue(DaemonToWrapper::End {
                subscription_id,
                reason: "wrapper indisponible".to_string(),
            });
        }
        if let Some(view) = view {
            views.push(view);
        }
    }
    state
        .pending_attach_sends
        .retain(|_, pending| pending.conn_id != conn_id);
    (controls, views)
}

fn collect_closed_attach_views(
    state: &mut DaemonState,
) -> (Vec<DeferredControl>, Vec<Arc<AttachView>>) {
    let mut controls = Vec::new();
    let mut views = Vec::new();
    while let Ok(subscription_id) = state.view_closed_rx.try_recv() {
        let Some(subscription) = state.attach_subscriptions.remove(&subscription_id) else {
            continue;
        };
        if let Some(view) = state.attach_views.remove(&subscription_id) {
            views.push(view);
        }
        defer_control(
            state,
            &subscription.wrapper_conn,
            DaemonToWrapper::Unsubscribe {
                subscription_id: subscription_id.clone(),
            },
            &mut controls,
        );
        defer_control(
            state,
            &subscription.attach_conn,
            DaemonToWrapper::End {
                subscription_id,
                reason: "vue trop lente".to_string(),
            },
            &mut controls,
        );
    }
    (controls, views)
}

// Fonction pour exposer les métriques publiquement (M-005)
pub fn get_metrics() -> &'static Metrics {
    // Note: ceci est un stub pour l'observabilité
    // Dans une implémentation complète, les métriques seraient partagées globalement
    use std::sync::OnceLock;
    static METRICS: OnceLock<Metrics> = OnceLock::new();
    METRICS.get_or_init(Metrics::new)
}

impl DaemonState {
    fn new(config: &DaemonConfig) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(config.socket_path.parent().unwrap())?;
        let store = Store::open(&config.db_path)?;
        let idempotency = IdempotencyStore::open(&config.db_path)?;
        let (view_closed_tx, view_closed_rx) = mpsc::channel();
        Ok(DaemonState {
            router: Router::new(),
            circuit_breaker: CircuitBreaker::new(
                config.circuit_breaker_window,
                config.circuit_breaker_limit,
            ),
            deduplicator: Deduplicator::new(config.dedup_window),
            envelope_guard: EnvelopeGuard::new(Duration::from_secs(config.quarantine_window)),
            store,
            idempotency,
            connections: HashMap::new(),
            conn_names: HashMap::new(),
            conn_hosts: HashMap::new(),
            conn_operating_systems: HashMap::new(),
            conn_instances: HashMap::new(),
            connection_roles: HashMap::new(),
            client_negotiations: HashMap::new(),
            attach_subscriptions: HashMap::new(),
            attach_views: HashMap::new(),
            view_closed_tx,
            view_closed_rx,
            pending_attach_sends: HashMap::new(),
            presences: HashMap::new(),
            conn_counter: 0,
            pending_replies: Vec::new(),
        })
    }

    fn next_conn_id(&mut self) -> String {
        self.conn_counter += 1;
        format!("conn-{}", self.conn_counter)
    }

    fn mark_unreachable(&mut self, conn_id: &str) {
        if let Some(instance_id) = self.conn_instances.remove(conn_id)
            && let Some(presence) = self.presences.get_mut(&instance_id)
        {
                presence.state = "unreachable".to_string();
                presence.last_seen = Instant::now();
            }
    }

    fn mark_stopped(&mut self, conn_id: &str) {
        if let Some(instance_id) = self.conn_instances.remove(conn_id)
            && let Some(presence) = self.presences.get_mut(&instance_id)
        {
                presence.state = "stopped".to_string();
                presence.last_seen = Instant::now();
            }
    }

    fn set_turn_state(&mut self, conn_id: &str, in_progress: bool) -> Result<(), String> {
        let instance_id = self
            .conn_instances
            .get(conn_id)
            .ok_or_else(|| "état de tour reçu d'une connexion non enregistrée".to_string())?;
        let presence = self
            .presences
            .get_mut(instance_id)
            .ok_or_else(|| "présence de l'équipier introuvable".to_string())?;
        presence.state = if in_progress { "busy" } else { "connected" }.to_string();
        presence.last_seen = Instant::now();
        Ok(())
    }

    fn agent_infos(&mut self) -> Vec<bridget_transport::protocol::AgentInfo> {
        self.presences.retain(|_, presence| {
            presence.state == "connected" || presence.last_seen.elapsed() <= PRESENCE_RETENTION
        });
        let mut agents: Vec<_> = self
            .router
            .list_agents()
            .iter()
            .map(|agent| {
                let presence = self
                    .conn_instances
                    .get(&agent.connection_id)
                    .and_then(|id| self.presences.get(id));
                bridget_transport::protocol::AgentInfo {
                    name: agent.name.clone(),
                    agent_type: agent.agent_type.to_string(),
                    connection_id: agent.connection_id.clone(),
                    host: presence
                        .map(|p| p.host.clone())
                        .or_else(|| self.conn_hosts.get(&agent.connection_id).cloned())
                        .unwrap_or_else(|| "inconnu".to_string()),
                    transport: presence
                        .map(|p| p.transport.clone())
                        .unwrap_or_else(|| "unix".to_string()),
                    os: presence
                        .map(|p| p.os.clone())
                        .or_else(|| {
                            self.conn_operating_systems
                                .get(&agent.connection_id)
                                .cloned()
                        })
                        .unwrap_or_else(|| "inconnu".to_string()),
                    // Un agent qui refuse d'être dérangé est connecté mais non
                    // joignable : du point de vue de l'appelant, la question
                    // « puis-je lui écrire » a la même forme que pour un agent
                    // injoignable, d'où un état unique plutôt qu'une colonne.
                    state: match presence {
                        Some(presence) if presence.is_dnd() => "dnd".to_string(),
                        Some(presence) => presence.state.clone(),
                        None => "connected".to_string(),
                    },
                    last_seen_secs: presence
                        .map(|p| p.last_seen.elapsed().as_secs())
                        .unwrap_or(0),
                    reconnect_count: presence.map(|p| p.reconnect_count).unwrap_or(0),
                    domain: presence.and_then(|p| p.domain.clone()),
                    model: presence.and_then(|p| p.model.clone()),
                    effort: presence.and_then(|p| p.effort.clone()),
                }
            })
            .collect();
        let live_names: std::collections::HashSet<String> =
            agents.iter().map(|agent| agent.name.clone()).collect();
        for presence in self.presences.values().filter(|presence| {
                matches!(presence.state.as_str(), "stopped" | "unreachable")
                    && !live_names.contains(&presence.name)
        }) {
            agents.push(bridget_transport::protocol::AgentInfo {
                name: presence.name.clone(),
                agent_type: presence.agent_type.clone(),
                connection_id: String::new(),
                host: presence.host.clone(),
                transport: presence.transport.clone(),
                os: presence.os.clone(),
                state: presence.state.clone(),
                last_seen_secs: presence.last_seen.elapsed().as_secs(),
                reconnect_count: presence.reconnect_count,
                domain: presence.domain.clone(),
                // FR-010 : un agent injoignable garde sa dernière capacité connue.
                model: presence.model.clone(),
                effort: presence.effort.clone(),
            });
        }
        agents.sort_by(|left, right| left.name.cmp(&right.name));
        agents
    }

    fn restore_pending_for_agent(&mut self, name: &str, conn_id: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let Ok(requests) = self.store.open_requests() else {
            return;
        };
        for request in requests {
            if request.sender != name && request.target != name
                || self
                    .pending_replies
                    .iter()
                    .any(|pending| pending.msg_id == request.id)
            {
                continue;
            }
            let Some(target_conn) = self
                .router
                .get_agent(&request.target)
                .map(|agent| agent.connection_id.clone())
            else {
                continue;
            };
            let Some(from_conn) = self
                .router
                .get_agent(&request.sender)
                .map(|agent| agent.connection_id.clone())
            else {
                continue;
            };
            let timeout_secs = (request.deadline_at - request.created_at).max(1) as u64;
            let elapsed = (now - request.created_at).max(0) as u64;
            let created_at = std::time::Instant::now()
                .checked_sub(Duration::from_secs(elapsed))
                .unwrap_or_else(std::time::Instant::now);
            self.pending_replies.push(PendingReply {
                msg_id: request.id,
                from: request.sender,
                from_conn,
                to: request.target,
                target_conn,
                timeout_secs,
                created_at,
                escalation_level: request.escalation_level,
                deferred_level: None,
            });
        }
        let _ = conn_id;
    }
}

/// Lance le daemon.
pub fn run(config: DaemonConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Verrouillage exclusif avec flock — empêche deux daemons de démarrer en même temps
    // Évite la race condition TOCTOU du PID file traditionnel
    let pid_file = config.socket_path.with_extension("pid");

    // Créer le fichier et obtenir un verrou exclusif avec flock
    use std::os::unix::io::AsRawFd;

    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        // Ce descripteur ne sert qu'au verrou flock ; l'écriture du PID suit
        // avec std::fs::write et effectue explicitement le remplacement.
        .truncate(false)
        .open(&pid_file)
        .map_err(|e| format!("Impossible de créer PID file {}: {}", pid_file.display(), e))?;

    // Tenter d'obtenir un verrou exclusif (non-bloquant)
    unsafe {
        if libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) != 0 {
            // Le verrou échoue = un autre daemon tourne
            eprintln!(
                "bridget: un daemon tourne déjà (verrou sur {})",
                pid_file.display()
            );
            std::process::exit(0);
        }
    }

    // Écrire notre PID maintenant qu'on a le verrou
    std::fs::write(&pid_file, std::process::id().to_string())?;
    eprintln!(
        "[BRIDGET] PID file écrit avec verrou exclusif: {} (PID {})",
        pid_file.display(),
        std::process::id()
    );

    if config.socket_path.exists() {
        std::fs::remove_file(&config.socket_path)?;
    }
    std::fs::create_dir_all(config.socket_path.parent().unwrap())?;

    let listener = UnixListener::bind(&config.socket_path)?;
    info!("bridget daemon écoute sur {}", config.socket_path.display());

    let state = Arc::new(Mutex::new(DaemonState::new(&config)?));

    // Purge au démarrage
    {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        if let Ok(n) = st.store.purge_older_than_days(config.retention_days)
            && n > 0
        {
                info!(
                    "purge: {} messages supprimés (> {} jours)",
                    n, config.retention_days
                );
            }
    }

    // Thread de surveillance des --reply sans réponse (escalade progressive)
    // Palier 1 (T/3) : rappel discret au destinataire
    // Palier 2 (2T/3) : rappel ferme au destinataire
    // Palier 3 (T) : notification d'échec à l'émetteur
    // Après T + 30s : abandon (retiré de la liste)
    let st_reminder = state.clone();
    thread::spawn(move || {
        loop {
        thread::sleep(Duration::from_secs(1)); // Réduit de 3s à 1s pour meilleure réactivité
        let now = std::time::Instant::now();

        // Collecter les actions à faire
        let actions: Vec<ReminderAction> = {
            let mut st = st_reminder.lock().unwrap_or_else(|e| e.into_inner());
            collect_reminder_actions(&mut st, now)
        };

        // Exécuter les actions hors lock
        for action in actions {
            match action {
                ReminderAction::Gentle {
                    to,
                    from,
                    msg_id,
                    target_conn,
                } => {
                    let st = st_reminder.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(target_writer) = st.connections.get(&target_conn) {
                        let body = format!(
                            "Rappel : {} attend ta reponse au message #{}.\nReponds avec: bridget reply \"ta reponse\"",
                                from,
                                &msg_id[..msg_id.len().min(8)]
                        );
                        if let Err(e) = deliver_to_agent(target_writer, &to, &body) {
                            error!("Impossible de délivrer le rappel doux à {}: {}", to, e);
                        }
                        info!("palier 1 (rappel doux) → {} pour {}", to, from);
                    }
                }
                ReminderAction::Firm {
                    to,
                    from,
                    msg_id,
                    target_conn,
                } => {
                    let st = st_reminder.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(target_writer) = st.connections.get(&target_conn) {
                        let body = format!(
                            "URGENT : {} attend toujours ta reponse au message #{}.\nTu DOIS repondre maintenant avec: bridget reply \"ta reponse\"\nSi tu ne peux pas repondre, notifie-le : bridget reply \"impossible de repondre : <raison>\"",
                                from,
                                &msg_id[..msg_id.len().min(8)]
                        );
                        if let Err(e) = deliver_to_agent(target_writer, &to, &body) {
                            error!("Impossible de délivrer le rappel ferme à {}: {}", to, e);
                        }
                        info!("palier 2 (rappel ferme) → {} pour {}", to, from);
                    }
                }
                ReminderAction::Timeout {
                    to,
                    from,
                    msg_id,
                    from_conn,
                    timeout_secs,
                } => {
                    let st = st_reminder.lock().unwrap_or_else(|e| e.into_inner());
                    // Notifier l'émetteur que le destinataire n'a pas répondu
                    if let Some(sender_writer) = st.connections.get(&from_conn) {
                        let body = format!(
                            "{} n'a pas repondu en {}s au message #{}.\nTu peux reessayer, changer de destinataire ou abandonner.",
                                to,
                                timeout_secs,
                                &msg_id[..msg_id.len().min(8)]
                        );
                        if let Err(e) = deliver_to_agent(sender_writer, &from, &body) {
                                error!(
                                    "Impossible de délivrer la notification de timeout à {}: {}",
                                    from, e
                                );
                        }
                        info!(
                            "palier 3 (timeout notifié à {} : {} n'a pas répondu)",
                            from, to
                        );
                    }
                }
                ReminderAction::Deferred { to, msg_id, level } => {
                        info!(
                            "relance différée (tour en cours) : palier {} pour {} sur demande {}",
                            level, to, msg_id
                        );
                    }
                }
            }
        }
    });

    // Thread de purge périodique
    let st_purge = state.clone();
    let retention = config.retention_days;
    thread::spawn(move || {
        loop {
        thread::sleep(Duration::from_secs(3600));
        let st = st_purge.lock().unwrap_or_else(|e| e.into_inner());
        if let Ok(n) = st.store.purge_older_than_days(retention)
                && n > 0
            {
                info!("purge périodique: {} messages supprimés", n);
            }
        }
    });

    // Setup signal handler — flag atomique global (pas de Mutex dans le handler)
    // On utilise un flag atomique simple. Le shutdown propre (notification
    // des wrappers) est fait dans la boucle principale quand elle détecte le flag.
    SHUTDOWN_REQUESTED.store(false, Ordering::SeqCst);

    // Seul SIGTERM déclenche le shutdown propre (c'est ce que launchd/systemd envoie).
    // SIGINT (Ctrl+C) est ignoré en mode daemon — l'utilisateur doit utiliser
    // launchctl stop ou kill -TERM pour arrêter le daemon.
    unsafe {
        let _ = signal_hook::low_level::register(signal_hook::consts::SIGTERM, || {
            eprintln!(
                "[BRIDGET] *** SIGTERM REÇU *** à {}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            );
            SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
        });
        let _ = signal_hook::low_level::register(signal_hook::consts::SIGINT, || {
            eprintln!(
                "[BRIDGET] *** SIGINT REÇU (ignoré) *** à {}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            );
        });
        // Ignorer SIGHUP (envoyé quand le terminal se ferme)
        let _ = signal_hook::low_level::register(signal_hook::consts::SIGHUP, || {
            eprintln!("[BRIDGET] *** SIGHUP REÇU (ignoré) ***");
        });
    }

    // Boucle d'acceptation avec timeout pour vérifier shutdown
    listener.set_nonblocking(true)?;
    loop {
        // Vérifier si shutdown demandé
        if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
            info!("shutdown demandé — notification des wrappers...");
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            for writer in st.connections.values() {
                let msg = DaemonToWrapper::Disconnect;
                if let Ok(json) = encode(&msg)
                    && let Ok(mut w) = writer.lock()
                {
                        let _ = writeln!(w, "{}", json);
                        let _ = w.flush();
                    }
            }
            drop(st);
            if let Err(e) = std::fs::remove_file(&config.socket_path) {
                log::warn!(
                    "Impossible de supprimer socket {}: {}",
                    config.socket_path.display(),
                    e
                );
            }
            if let Err(e) = std::fs::remove_file(config.socket_path.with_extension("pid")) {
                log::warn!(
                    "Impossible de supprimer PID file {}: {}",
                    config.socket_path.with_extension("pid").display(),
                    e
                );
            }
            info!("daemon arrêté proprement");
            return Ok(());
        }

        match listener.accept() {
            Ok((stream, _)) => {
                if let Err(e) = stream.set_nonblocking(false) {
                    error!("set_nonblocking failed: {} — connexion ignorée", e);
                    continue;
                }
                let st = state.clone();
                thread::spawn(move || {
                    if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        handle_connection(stream, st)
                    })) {
                        error!("PANIC dans thread connexion: {:?}", e);
                    }
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                error!("accept: {}", e);
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

/// Gère une connexion wrapper.
fn handle_connection(
    stream: UnixStream,
    state: Arc<Mutex<DaemonState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn_id = state
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .next_conn_id();
    log::debug!("handle_connection: nouvelle connexion {}", conn_id);
    let reader_stream = stream.try_clone()?;
    let reader = BufReader::new(reader_stream);

    // Enregistrer le writer dans la map pour les push
    let push_stream = stream.try_clone()?;
    {
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        st.connections.insert(
            conn_id.clone(),
            Arc::new(Mutex::new(BufWriter::new(push_stream))),
        );
    }
    info!("connexion {} établie", conn_id);

    let mut my_writer = BufWriter::new(stream);

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.is_empty() {
            continue;
        }

        let msg: WrapperToDaemon = match decode(&line) {
            Ok(m) => m,
            Err(e) => {
                warn!("message illisible de {}: {}", conn_id, e);
                continue;
            }
        };

        let response = handle_wrapper_message(&conn_id, msg, &state);
        if let Some(dtw) = response {
            let json = encode(&dtw)?;
            writeln!(my_writer, "{}", json)?;
            my_writer.flush()?;
        }
    }

    // Connexion fermée : désenregistrer avec nettoyage explicite pour éviter fuites
    let (writer_opt, removed, controls, views) = {
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        let (controls, views) = close_attach_subscriptions(&mut st, &conn_id);

        // Récupérer le writer AVANT suppression pour nettoyage explicite
        let writer_opt = st.connections.remove(&conn_id);

        let removed = st.router.unregister_by_conn(&conn_id);
        st.mark_unreachable(&conn_id);
        st.conn_names.remove(&conn_id);
        st.conn_hosts.remove(&conn_id);
        st.conn_operating_systems.remove(&conn_id);
        st.connection_roles.remove(&conn_id);
        st.client_negotiations.remove(&conn_id);
        (writer_opt, removed, controls, views)
    };
    let _ = execute_controls(controls);
    for view in views {
        view.close_and_join();
    }

    // Nettoyage explicite du writer pour éviter fuites de ressources
    if let Some(writer_mutex) = writer_opt
        && let Ok(mut writer) = writer_mutex.lock()
    {
            use std::io::Write;
            let _ = writer.flush();
            // Le drop explicite fermera le stream proprement
        }

    if let Some(agent) = removed {
        info!("agent '{}' déconnecté ({})", agent.name, conn_id);
    } else {
        info!("connexion {} fermée (non enregistrée)", conn_id);
    }
    log::debug!("handle_connection {} terminée", conn_id);

    Ok(())
}

/// Traite l'enregistrement d'un wrapper
///
/// Les champs du message `Register` restent dépliés ici pour refléter le
/// protocole de transport ; les regrouper imposerait un refactor hors scope.
#[allow(clippy::too_many_arguments)]
fn handle_register(
    conn_id: &str,
    agent_type: String,
    name: Option<String>,
    host: Option<String>,
    transport: Option<String>,
    os: Option<String>,
    instance_id: Option<String>,
    domain: Option<String>,
    turn_in_progress: bool,
    state: &mut DaemonState,
) -> DaemonToWrapper {
    log::debug!(
        "Register reçu de {}: type={}, name={:?}, host={:?}",
        conn_id,
        agent_type,
        name,
        host
    );

    let parsed_type = agent_type
        .parse()
        .unwrap_or(bridget_core::AgentType::Custom(agent_type));

    match state
        .router
        .register(name.as_deref(), &parsed_type, conn_id)
    {
        Ok(final_name) => {
            state
                .conn_names
                .insert(conn_id.to_string(), final_name.clone());
            state.conn_hosts.insert(
                conn_id.to_string(),
                host.clone().unwrap_or_else(|| "inconnu".to_string()),
            );
            state.conn_operating_systems.insert(
                conn_id.to_string(),
                os.clone().unwrap_or_else(|| "inconnu".to_string()),
            );

            if let Some(instance_id) = instance_id.filter(|id| !id.is_empty()) {
                let previous = state.presences.get(&instance_id);
                let reconnect_count = previous
                    .map(|presence| {
                        presence.reconnect_count + u32::from(presence.state != "connected")
                    })
                    .unwrap_or(0);
                // Une reconnexion sous la même instance conserve le runtime déjà
                // observé : l'agent n'a pas changé de modèle en perdant le socket.
                let (model, effort) = previous
                    .map(|presence| (presence.model.clone(), presence.effort.clone()))
                    .unwrap_or((None, None));
                // Le domaine annoncé par le wrapper fait foi : il porte déjà la
                // surcharge s'il en existe une, puisqu'il relit le fichier
                // d'état avant de se réenregistrer.
                let derived_domain = domain
                    .clone()
                    .or_else(|| previous.and_then(|presence| presence.derived_domain.clone()));
                let dnd_until = previous.and_then(|presence| presence.dnd_until);

                state
                    .conn_instances
                    .insert(conn_id.to_string(), instance_id.clone());
                state.presences.insert(
                    instance_id,
                    Presence {
                        name: final_name.clone(),
                        agent_type: parsed_type.to_string(),
                        host: host.unwrap_or_else(|| "inconnu".to_string()),
                        transport: transport.unwrap_or_else(|| "unix".to_string()),
                        os: os.unwrap_or_else(|| "inconnu".to_string()),
                        state: if turn_in_progress {
                            "busy"
                        } else {
                            "connected"
                        }
                        .to_string(),
                        last_seen: Instant::now(),
                        reconnect_count,
                        model,
                        effort,
                        domain: derived_domain.clone(),
                        derived_domain,
                        dnd_until,
                    },
                );
            }

            state.restore_pending_for_agent(&final_name, conn_id);
            info!("agent '{}' enregistré ({})", final_name, conn_id);
            DaemonToWrapper::Registered { name: final_name }
        }
        Err(e) => {
            log::warn!("enregistrement refusé pour {}: {}", conn_id, e);
            DaemonToWrapper::Nack {
                id: "register".to_string(),
                reason: format!("enregistrement refusé: {}", e),
            }
        }
    }
}

/// Longueur maximale acceptée pour un identifiant de modèle ou un niveau
/// d'effort, alignée sur la validation des noms d'agent côté CLI.
const MAX_RUNTIME_VALUE_LENGTH: usize = 100;

/// Rejette une valeur trop longue ou porteuse de caractères de contrôle, qui
/// casserait l'alignement de l'annuaire ou l'affichage du terminal.
fn validate_runtime_value(value: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err("valeur vide".to_string());
    }
    if value.chars().count() > MAX_RUNTIME_VALUE_LENGTH {
        return Err(format!(
            "valeur trop longue (max {} caractères)",
            MAX_RUNTIME_VALUE_LENGTH
        ));
    }
    if value.chars().any(char::is_control) {
        return Err("valeur contenant des caractères de contrôle".to_string());
    }
    Ok(())
}

/// Applique une observation de runtime à la présence d'un agent nommé.
///
/// L'agent est désigné par son nom et non par la connexion émettrice : le hook
/// Claude et `bridget runtime` transitent par le client CLI, dont la connexion
/// est éphémère. Même résolution que `Rename` et `CancelRequest`.
///
/// Le couple `(model, effort)` remplace l'état courant **en bloc** : un effort
/// absent efface l'effort connu, parce qu'il décrit une observation réelle et
/// non une lacune. Sans cela, un agent passant d'un modèle qui expose l'effort
/// à un modèle qui ne l'expose pas conserverait indéfiniment l'ancienne valeur.
///
/// Ce message ne traverse ni le routeur ni le disjoncteur : ce n'est pas du
/// trafic entre agents, il ne peut ni être routé ni boucler.
fn handle_runtime(
    agent: &str,
    model: String,
    effort: Option<String>,
    source: bridget_transport::protocol::RuntimeSource,
    state: &mut DaemonState,
) -> DaemonToWrapper {
    if let Err(reason) = validate_runtime_value(&model) {
        return DaemonToWrapper::Nack {
            id: "runtime".to_string(),
            reason: format!("modèle invalide: {}", reason),
        };
    }
    if let Some(Err(reason)) = effort.as_deref().map(validate_runtime_value) {
        return DaemonToWrapper::Nack {
            id: "runtime".to_string(),
            reason: format!("effort invalide: {}", reason),
        };
    }

    let Some(presence) = presence_of_agent(state, agent) else {
        return DaemonToWrapper::Nack {
            id: "runtime".to_string(),
            reason: format!("agent introuvable: {}", agent),
        };
    };

    let unchanged = presence.model.as_deref() == Some(model.as_str())
        && presence.effort.as_deref() == effort.as_deref();
    if !unchanged {
        log::debug!(
            "runtime de '{}' mis à jour par {} : modèle={} effort={:?}",
            presence.name,
            source,
            model,
            effort
        );
        presence.model = Some(model);
        presence.effort = effort;
    }
    presence.last_seen = Instant::now();

    DaemonToWrapper::Ack {
        id: "runtime".to_string(),
    }
}

/// Décide si un rappel d'escalade doit être délivré.
///
/// Un destinataire qui refuse d'être dérangé ne reçoit ni le rappel discret ni
/// le rappel ferme : respecter le statut à l'aller pour le violer au rappel
/// n'aurait aucun sens. La demande reste ouverte et son échéance court toujours.
///
/// Le palier 2 et au-delà correspond à la notification d'échec adressée à
/// l'**émetteur** : elle ne dérange pas le destinataire et part donc toujours.
fn should_remind(target_is_undisturbed: bool, escalation_level: u8) -> bool {
    !target_is_undisturbed || escalation_level >= 2
}

fn deferred_reminder_level(
    target_is_busy: bool,
    elapsed_secs: u64,
    timeout_secs: u64,
) -> Option<u8> {
    if !target_is_busy {
        return None;
    }
    if elapsed_secs >= (timeout_secs * 2) / 3 {
        Some(2)
    } else if elapsed_secs >= timeout_secs / 3 {
        Some(1)
    } else {
        None
    }
}

/// Applique une itération complète de la surveillance des demandes suivies.
/// Les écritures de socket restent dans la boucle du daemon, hors verrou ; ce
/// facteur ne produit que les actions et persiste les transitions associées.
fn collect_reminder_actions(state: &mut DaemonState, now: Instant) -> Vec<ReminderAction> {
    let mut actions = Vec::new();
    let mut state_updates = Vec::new();
    let mut timeout_candidates = Vec::new();
    let mut deferred_events = Vec::new();
    let undisturbed: std::collections::HashSet<String> = state
        .presences
        .values()
        .filter(|presence| presence.is_dnd())
        .map(|presence| presence.name.clone())
        .collect();
    let busy_connections: std::collections::HashSet<String> = state
        .conn_instances
        .iter()
        .filter(|(_, instance_id)| {
            state
                .presences
                .get(*instance_id)
                .is_some_and(|presence| presence.state == "busy")
        })
        .map(|(connection_id, _)| connection_id.clone())
        .collect();

    for pending in state.pending_replies.iter_mut() {
        let elapsed = now.duration_since(pending.created_at).as_secs();
        let timeout = pending.timeout_secs;
        if elapsed >= timeout {
            pending.escalation_level = 3;
            timeout_candidates.push((
                pending.to.clone(),
                pending.from.clone(),
                pending.msg_id.clone(),
                pending.from_conn.clone(),
                timeout,
            ));
            continue;
        }
        if !should_remind(undisturbed.contains(&pending.to), pending.escalation_level) {
            continue;
        }
        if let Some(level) = deferred_reminder_level(
            busy_connections.contains(&pending.target_conn),
            elapsed,
            timeout,
        ) {
            if pending.deferred_level != Some(level) {
                pending.deferred_level = Some(level);
                deferred_events.push((pending.msg_id.clone(), level));
                actions.push(ReminderAction::Deferred {
                    to: pending.to.clone(),
                    msg_id: pending.msg_id.clone(),
                    level,
                });
            }
            continue;
        }
        if let Some(level) = pending.deferred_level.take() {
            pending.escalation_level = level;
            state_updates.push((pending.msg_id.clone(), level));
            actions.push(if level == 1 {
                ReminderAction::Gentle {
                    to: pending.to.clone(),
                    from: pending.from.clone(),
                    msg_id: pending.msg_id.clone(),
                    target_conn: pending.target_conn.clone(),
                }
            } else {
                ReminderAction::Firm {
                    to: pending.to.clone(),
                    from: pending.from.clone(),
                    msg_id: pending.msg_id.clone(),
                    target_conn: pending.target_conn.clone(),
                }
            });
            continue;
        }
        if pending.escalation_level == 0 && elapsed >= timeout / 3 {
            pending.escalation_level = 1;
            state_updates.push((pending.msg_id.clone(), 1));
            actions.push(ReminderAction::Gentle {
                to: pending.to.clone(),
                from: pending.from.clone(),
                msg_id: pending.msg_id.clone(),
                target_conn: pending.target_conn.clone(),
            });
        } else if pending.escalation_level == 1 && elapsed >= (timeout * 2) / 3 {
            pending.escalation_level = 2;
            state_updates.push((pending.msg_id.clone(), 2));
            actions.push(ReminderAction::Firm {
                to: pending.to.clone(),
                from: pending.from.clone(),
                msg_id: pending.msg_id.clone(),
                target_conn: pending.target_conn.clone(),
            });
        }
    }

    state.pending_replies.retain(|pending| {
        pending.escalation_level < 3
            || now.duration_since(pending.created_at).as_secs()
                < pending.timeout_secs + TIMEOUT_GRACE_PERIOD
    });
    for (id, level) in state_updates {
        let _ = state.store.set_escalation_level(&id, level);
    }
    for (id, level) in deferred_events {
        let _ = state.store.record_deferred_reminder(&id, level);
    }
    for (to, from, msg_id, from_conn, timeout_secs) in timeout_candidates {
        if claim_timeout(&state.store, &msg_id) {
            actions.push(ReminderAction::Timeout {
                to,
                from,
                msg_id,
                from_conn,
                timeout_secs,
            });
        }
    }
    actions
}

/// Retourne vrai pour le seul chemin autorisé à notifier l'émetteur d'une
/// échéance. SQLite arbitre l'intercalage transport ↔ thread de relance.
fn claim_timeout(store: &Store, id: &str) -> bool {
    store.mark_timed_out(id).unwrap_or(false)
}

/// Retrouve la présence d'un agent désigné par son nom.
///
/// Même résolution que `handle_runtime` : les commandes de contrôle arrivent par
/// le client CLI, dont la connexion est éphémère et distincte de celle de
/// l'agent visé.
fn presence_of_agent<'a>(state: &'a mut DaemonState, agent: &str) -> Option<&'a mut Presence> {
    let instance_id = state
        .router
        .get_agent(agent)
        .map(|found| found.connection_id.clone())
        .and_then(|conn| state.conn_instances.get(&conn).cloned())?;
    state.presences.get_mut(&instance_id)
}

/// Remplace le domaine d'un agent, ou le ramène à son domaine dérivé.
fn handle_domain(agent: &str, domain: Option<String>, state: &mut DaemonState) -> DaemonToWrapper {
    if let Some(Err(reason)) = domain.as_deref().map(validate_runtime_value) {
        return DaemonToWrapper::Nack {
            id: "domain".to_string(),
            reason: format!("domaine invalide: {}", reason),
        };
    }
    let Some(presence) = presence_of_agent(state, agent) else {
        return DaemonToWrapper::Nack {
            id: "domain".to_string(),
            reason: format!("agent introuvable: {}", agent),
        };
    };
    presence.domain = match domain {
        Some(domain) => Some(domain),
        // Réinitialisation : on retombe sur ce que le wrapper avait annoncé.
        None => presence.derived_domain.clone(),
    };
    log::debug!("domaine de '{}' : {:?}", presence.name, presence.domain);
    DaemonToWrapper::Ack {
        id: "domain".to_string(),
    }
}

/// Déclare la disponibilité d'un agent.
fn handle_availability(
    agent: &str,
    until_secs: Option<u64>,
    state: &mut DaemonState,
) -> DaemonToWrapper {
    let Some(presence) = presence_of_agent(state, agent) else {
        return DaemonToWrapper::Nack {
            id: "availability".to_string(),
            reason: format!("agent introuvable: {}", agent),
        };
    };
    presence.dnd_until = until_secs.and_then(|until| {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        // Une échéance déjà passée équivaut à une levée du statut.
        until
            .checked_sub(now)
            .filter(|remaining| *remaining > 0)
            .map(|remaining| Instant::now() + Duration::from_secs(remaining))
    });
    log::debug!(
        "disponibilité de '{}' : dnd={}",
        presence.name,
        presence.is_dnd()
    );
    DaemonToWrapper::Ack {
        id: "availability".to_string(),
    }
}

fn unix_now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .min(i64::MAX as u64) as i64
}

fn next_delivery_generation() -> u64 {
    loop {
        let generation = (Uuid::new_v4().as_u128() as u64) & i64::MAX as u64;
        if generation != 0 {
            return generation;
        }
    }
}

fn canonical_field(bytes: &mut Vec<u8>, value: &[u8]) {
    bytes.extend_from_slice(&(value.len() as u64).to_be_bytes());
    bytes.extend_from_slice(value);
}

fn canonical_option<T: ToString>(bytes: &mut Vec<u8>, value: Option<T>) {
    match value {
        Some(value) => {
            bytes.push(1);
            canonical_field(bytes, value.to_string().as_bytes());
        }
        None => bytes.push(0),
    }
}

/// Sérialisation binaire fermée et sans ambiguïté de l'enveloppe publiée.
/// Elle ne dépend ni de l'ordre JSON ni des valeurs mutées lors du routage.
fn canonical_send(
    issuer_scope: &str,
    message_id: &str,
    message: &bridget_core::BridgetMessage,
    issued_at: i64,
) -> Vec<u8> {
    let mut bytes = b"bridget/client-send/v1\0".to_vec();
    canonical_field(&mut bytes, issuer_scope.as_bytes());
    canonical_field(&mut bytes, message_id.as_bytes());
    canonical_field(&mut bytes, message.to.as_bytes());
    canonical_field(&mut bytes, message.body.as_bytes());
    bytes.push(u8::from(message.reply));
    canonical_field(&mut bytes, &message.hops.to_be_bytes());
    canonical_option(&mut bytes, message.deadline_at);
    canonical_option(&mut bytes, message.in_reply_to.as_deref());
    canonical_field(&mut bytes, &issued_at.to_be_bytes());
    bytes
}

fn issue_response(key: &IdempotencyKey, issue: IdempotencyIssue) -> DaemonToWrapper {
    DaemonToWrapper::IdempotencyResult {
        operation_kind: key.operation_kind.as_str().to_string(),
        idempotency_key: key.idempotency_key.clone(),
        issue,
    }
}

fn replay_issue(
    st: &DaemonState,
    key: &IdempotencyKey,
    result: LookupResult,
) -> Result<IdempotencyIssue, String> {
    match result {
        LookupResult::Accepted { expires_at } => Ok(IdempotencyIssue::Accepted { expires_at }),
        LookupResult::Rejected { category, reason } => {
            Ok(IdempotencyIssue::Rejected { category, reason })
        }
        LookupResult::OutcomeUnknown { expires_at } => Ok(IdempotencyIssue::OutcomeUnknown {
            expires_at,
            delivery_id: st
                .idempotency
                .send_delivery(key)
                .map_err(|error| error.to_string())?
                .map(|delivery| delivery.delivery_id),
        }),
        LookupResult::IdempotencyExpired => Ok(IdempotencyIssue::IdempotencyExpired),
    }
}

fn handle_idempotency_lookup(
    conn_id: &str,
    operation_kind: String,
    idempotency_key: String,
    st: &DaemonState,
) -> DaemonToWrapper {
    let Some(negotiated) = st.client_negotiations.get(conn_id) else {
        return DaemonToWrapper::ClientRejected { reason: ClientRefusal::NegotiationRequired };
    };
    if operation_kind != "send" {
        return DaemonToWrapper::Nack { id: idempotency_key, reason: "opération idempotente inconnue".to_string() };
    }
    let key = match IdempotencyKey::new(negotiated.issuer_scope.clone(), OperationKind::Send, idempotency_key) {
        Ok(key) => key,
        Err(error) => return DaemonToWrapper::Nack { id: "lookup".to_string(), reason: error.to_string() },
    };
    match st.idempotency.lookup(&key, unix_now_secs()) {
        Ok(result) => match replay_issue(st, &key, result) {
            Ok(issue) => issue_response(&key, issue),
            Err(error) => DaemonToWrapper::Nack { id: key.idempotency_key, reason: error },
        },
        Err(error) => DaemonToWrapper::Nack { id: key.idempotency_key, reason: error.to_string() },
    }
}

fn reject_idempotent_send(
    st: &mut DaemonState,
    key: &IdempotencyKey,
    category: impl Into<String>,
    reason: impl Into<String>,
) -> Result<DaemonToWrapper, String> {
    let category = category.into();
    let reason = reason.into();
    st.idempotency
        .reject_prepared(key, &category, &reason)
        .map_err(|error| error.to_string())?;
    Ok(issue_response(
        key,
        IdempotencyIssue::Rejected { category, reason },
    ))
}

/// Gardes communes précédant toute remise. Les deux appels publics conservent
/// leurs identités de déduplication propres, mais partagent les invariants de
/// réponse, DND, sauts et routage.
struct PreparedDispatch {
    content_key: String,
    message_guard_id: String,
    target_conn: String,
    logical_sender: String,
    reply_sender_conn: Option<String>,
    valid_tracked_reply: bool,
}

fn prepare_dispatch(
    st: &mut DaemonState,
    message: &mut bridget_core::BridgetMessage,
    conn_id: &str,
    logical_sender: String,
    content_key: String,
    message_guard_id: String,
) -> Result<PreparedDispatch, (&'static str, String)> {
    let reply_sender_conn = st
        .router
        .get_agent(&message.from)
        .map(|agent| agent.connection_id.clone());
    if message.reply && reply_sender_conn.is_none() {
        return Err((
            "reply_sender_unavailable",
            "--reply requiert un agent Bridget connecté ; lance la commande depuis un wrapper actif ou envoie sans --reply".to_string(),
        ));
    }

    let valid_tracked_reply = message.in_reply_to.as_deref().is_some_and(|request_id| {
        st.store
            .get_request(request_id)
            .ok()
            .flatten()
            .is_some_and(|request| {
                request.state == "open"
                    && request.sender == message.to
                    && request.target == message.from
            })
    });

    if !st.circuit_breaker.check(&logical_sender, &message.to) {
        return Err((
            "circuit_breaker",
            format!(
                "disjoncteur: limite {} échanges / {}s",
                st.circuit_breaker.limit(),
                st.circuit_breaker.window_secs()
            ),
        ));
    }
    if st.deduplicator.is_duplicate(&content_key, &message.to) {
        return Err(("duplicate_content", "doublon de contenu".to_string()));
    }
    if st
        .envelope_guard
        .is_quarantined(&message_guard_id, &message.to)
    {
        return Err((
            "quarantined",
            "message déjà relayé (quarantaine)".to_string(),
        ));
    }
    if !message.decrement_hops() {
        return Err(("hops_exhausted", "budget de sauts épuisé".to_string()));
    }
    if !valid_tracked_reply
        && let Some(presence) = presence_of_agent(st, &message.to)
        && presence.is_dnd()
    {
        let minutes = presence.dnd_minutes_left();
        let target = presence.name.clone();
        return Err((
            "dnd",
            format!("« {target} » ne souhaite pas être dérangé (encore {minutes} min)"),
        ));
    }
    let target_conn = match st
        .router
        .resolve(&message.from, &message.to, message.hops, conn_id)
    {
        RouterAction::Deliver { target_conn } => target_conn,
        RouterAction::Reject(error) => return Err(("routing", error.to_string())),
    };
    Ok(PreparedDispatch {
        content_key,
        message_guard_id,
        target_conn,
        logical_sender,
        reply_sender_conn,
        valid_tracked_reply,
    })
}

/// Rend durable l'attente d'une réponse après qu'une remise a été préparée.
/// Le même mécanisme est partagé par les envois historiques et idempotents,
/// afin que la reprise du daemon recharge la demande dans `pending_replies`.
fn track_reply_cycle(
    st: &mut DaemonState,
    message: &bridget_core::BridgetMessage,
    reply_sender_conn: Option<String>,
    target_conn: String,
) -> Result<(), String> {
    if !message.reply {
        return Ok(());
    }
    let timeout = message.reply_timeout.unwrap_or(60);
    st.store
        .create_request(&message.id, &message.from, &message.to, timeout)
        .map_err(|error| format!("impossible de suivre la demande: {error}"))?;
    remember_reply_cycle(st, message, reply_sender_conn, target_conn);
    Ok(())
}

fn remember_reply_cycle(
    st: &mut DaemonState,
    message: &bridget_core::BridgetMessage,
    reply_sender_conn: Option<String>,
    target_conn: String,
) {
    if !message.reply {
        return;
    }
    let timeout = message.reply_timeout.unwrap_or(60);
    st.pending_replies.push(PendingReply {
        msg_id: message.id.clone(),
        from: message.from.clone(),
        from_conn: reply_sender_conn
            .expect("une réponse suivie a été validée avec un expéditeur connecté"),
        to: message.to.clone(),
        target_conn,
        timeout_secs: timeout,
        created_at: std::time::Instant::now(),
        escalation_level: 0,
        deferred_level: None,
    });
}

fn handle_idempotent_send(
    conn_id: &str,
    mut message: bridget_core::BridgetMessage,
    message_id: String,
    issued_at: i64,
    st: &mut DaemonState,
) -> DaemonToWrapper {
    let Some(negotiated) = st.client_negotiations.get(conn_id).cloned() else {
        return DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::NegotiationRequired,
        };
    };
    // L'identifiant métier est l'autorité publique ; l'ancien champ `id` de
    // Bridget est donc normalisé avant toute comparaison ou garde mutable.
    message.id = message_id.clone();
    if message.reply && message.deadline_at.is_none() {
        let timeout = message.reply_timeout.unwrap_or(60);
        message.deadline_at = Some(
            issued_at
                .saturating_add(timeout as i64)
                .max(0) as u64,
        );
    }
    let key = match IdempotencyKey::new(negotiated.issuer_scope, OperationKind::Send, message_id) {
        Ok(key) => key,
        Err(_error) => {
            return DaemonToWrapper::ClientRejected {
                reason: ClientRefusal::MessageOutsideClientRole,
            };
        }
    };
    let canonical = canonical_send(&key.issuer_scope, &key.idempotency_key, &message, issued_at);
    let now = unix_now_secs();
    #[cfg(feature = "test-support")]
    crate::test_sync::checkpoint("before_reservation");
    let reservation = match st.idempotency.reserve(
        &key,
        &canonical,
        issued_at,
        CLIENT_IDEMPOTENCY_HORIZON_SECS,
        now,
        CLIENT_ISSUED_AT_TOLERANCE_SECS,
    ) {
        Ok(reservation) => reservation,
        Err(crate::idempotency::IdempotencyError::InvalidIssuedAt) => {
            return issue_response(&key, IdempotencyIssue::InvalidIssuedAt);
        }
        Err(error) => {
            error!("idempotence send: {error}");
            return DaemonToWrapper::Nack {
                id: key.idempotency_key.clone(),
                reason: "erreur de persistance idempotente".to_string(),
            };
        }
    };
    let expires_at = match reservation {
        Reservation::Prepared { expires_at } => expires_at,
        Reservation::Replayed(LookupResult::OutcomeUnknown { expires_at }) => match st
            .idempotency
            .prepared_expiry(&key, now)
        {
            Ok(Some(expires_at)) => expires_at,
            Ok(None) => {
                return issue_response(
                    &key,
                    IdempotencyIssue::OutcomeUnknown {
                        expires_at,
                        delivery_id: st
                            .idempotency
                            .send_delivery(&key)
                            .ok()
                            .flatten()
                            .map(|delivery| delivery.delivery_id),
                    },
                );
            }
            Err(error) => {
                error!("idempotence reprise préparée: {error}");
                return DaemonToWrapper::Nack {
                    id: key.idempotency_key.clone(),
                    reason: "issue idempotente illisible".to_string(),
                };
            }
        },
        Reservation::Replayed(result) => match replay_issue(st, &key, result) {
            Ok(issue) => return issue_response(&key, issue),
            Err(error) => {
                error!("idempotence replay: {error}");
                return DaemonToWrapper::Nack {
                    id: key.idempotency_key.clone(),
                    reason: "issue idempotente illisible".to_string(),
                };
            }
        },
        Reservation::EnvelopeMismatch => return issue_response(&key, IdempotencyIssue::EnvelopeMismatch),
        Reservation::IdempotencyExpired => return issue_response(&key, IdempotencyIssue::IdempotencyExpired),
    };
            // Les gardes ci-dessous peuvent consulter ou modifier les limites
            // historiques, mais seulement après la réservation d'une clé neuve.
            // Les gardes historiques restent applicables à une clé neuve,
            // mais leur espace est celui de l'émetteur idempotent : deux
            // scopes sont deux émetteurs logiques et ne se contaminent pas.
            let content_key = format!("{}:{}", key.issuer_scope, message.content_key());
            let scoped_message_id = format!("{}:{}", key.issuer_scope, message.id);
            let logical_sender = format!("client:{}", key.issuer_scope);
            let prepared = match prepare_dispatch(
                st,
                &mut message,
                conn_id,
                logical_sender,
                content_key,
                scoped_message_id,
            ) {
                Ok(prepared) => prepared,
                Err((category, reason)) => {
                    return reject_idempotent_send(st, &key, category, reason).unwrap_or_else(
                        |error| DaemonToWrapper::Nack {
                            id: key.idempotency_key.clone(),
                            reason: error,
                        },
                    );
                }
            };
            let Some(recipient_instance_id) = st.conn_instances.get(&prepared.target_conn).cloned() else {
                return reject_idempotent_send(
                    st,
                    &key,
                    "recipient_unavailable",
                    "instance destinataire inconnue",
                )
                .unwrap_or_else(|error| DaemonToWrapper::Nack {
                    id: key.idempotency_key.clone(),
                    reason: error,
                });
            };
            let delivery = SendDelivery {
                delivery_id: Uuid::new_v4().to_string(),
                recipient_instance_id,
                delivery_generation: next_delivery_generation(),
                expires_at,
            };
            let reply_tracking = message.reply.then(|| ReplyTracking {
                request_id: message.id.clone(),
                sender: message.from.clone(),
                target: message.to.clone(),
                created_at: now,
                deadline_at: message
                    .deadline_at
                    .expect("un reply idempotent est normalisé avant réservation")
                    .min(i64::MAX as u64) as i64,
            });
            let delivery_result = match reply_tracking.as_ref() {
                Some(reply) => st
                    .idempotency
                    .begin_send_delivery_with_reply(&key, &delivery, reply),
                None => st.idempotency.begin_send_delivery(&key, &delivery),
            };
            if let Err(error) = delivery_result {
                error!("idempotence dispatch: {error}");
                return DaemonToWrapper::Nack {
                    id: key.idempotency_key.clone(),
                    reason: "impossible de préparer la remise".to_string(),
                };
            }
            remember_reply_cycle(
                st,
                &message,
                prepared.reply_sender_conn.clone(),
                prepared.target_conn.clone(),
            );
            st.circuit_breaker
                .record(&prepared.logical_sender, &message.to);
            st.deduplicator
                .mark_sent(&prepared.content_key, &message.to);
            st.envelope_guard
                .mark_relayed(&prepared.message_guard_id, &message.to);
            #[cfg(feature = "test-support")]
            crate::test_sync::checkpoint("after_delivery_before_issue");
            let response = issue_response(
                &key,
                IdempotencyIssue::OutcomeUnknown {
                    expires_at,
                    delivery_id: Some(delivery.delivery_id),
                },
            );
            #[cfg(feature = "test-support")]
            crate::test_sync::checkpoint("after_issue_before_client_ack");
            response
}

/// Traite un message wrapper et retourne une réponse optionnelle.
fn handle_wrapper_message(
    conn_id: &str,
    msg: WrapperToDaemon,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    let (controls, views) = {
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        collect_closed_attach_views(&mut st)
    };
    let _ = execute_controls(controls);
    for view in views {
        view.close_and_join();
    }
    if matches!(msg, WrapperToDaemon::ClientHello { .. })
        && !state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .connection_roles
            .contains_key(conn_id)
    {
        // Un hello client ne doit jamais sélectionner implicitement le rôle
        // wrapper : l'ordre public est strict et sans effet de bord.
        return Some(DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::RoleHandshakeRequired,
        });
    }

    if !matches!(msg, WrapperToDaemon::RoleHandshake { .. }) {
        // La première commande non négociée choisit définitivement la
        // compatibilité wrapper. Une tentative d'upgrade ultérieure vers
        // attach ne doit jamais élargir une connexion déjà active.
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        st.connection_roles
            .entry(conn_id.to_string())
            .or_insert(ConnectionRole::Wrapper);
    }

    if let WrapperToDaemon::RoleHandshake { role } = &msg {
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        if st.connection_roles.contains_key(conn_id) {
            if *role == ConnectionRole::Client {
                return Some(DaemonToWrapper::ClientRejected {
                    reason: ClientRefusal::ClientRoleRequired,
                });
            }
            return Some(DaemonToWrapper::AttachRejected {
                subscription_id: None,
                reason: AttachRefusal::MessageOutsideAttachRole,
            });
        }
        st.connection_roles.insert(conn_id.to_string(), *role);
        return Some(DaemonToWrapper::RoleAccepted { role: *role });
    }

    let attach_refusal = {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        (st.connection_roles.get(conn_id) == Some(&ConnectionRole::Attach))
            .then(|| msg.attach_refusal())
            .flatten()
    };
    if let Some(reason) = attach_refusal {
        return Some(DaemonToWrapper::AttachRejected {
            subscription_id: None,
            reason,
        });
    }

    let client_refusal = {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        match st.connection_roles.get(conn_id) {
            Some(ConnectionRole::Client) => match &msg {
                WrapperToDaemon::ClientHello { .. }
                    if st.client_negotiations.contains_key(conn_id) =>
                {
                    Some(ClientRefusal::AlreadyNegotiated)
                }
                WrapperToDaemon::SendIdempotent { .. } | WrapperToDaemon::Lookup { .. }
                    if !st.client_negotiations.contains_key(conn_id) =>
                {
                    Some(ClientRefusal::NegotiationRequired)
                }
                WrapperToDaemon::SendIdempotent { .. }
                    if st
                        .client_negotiations
                        .get(conn_id)
                        .is_some_and(|negotiated| {
                            negotiated.version != CLIENT_CONTRACT_VERSION
                                || !negotiated
                                    .capabilities
                                    .contains(&ClientCapability::SendIdempotent)
                        }) =>
                {
                    Some(ClientRefusal::CapabilityNotNegotiated)
                }
                WrapperToDaemon::Lookup { .. }
                    if st
                        .client_negotiations
                        .get(conn_id)
                        .is_some_and(|negotiated| {
                            negotiated.version != CLIENT_CONTRACT_VERSION
                                || !negotiated.capabilities.contains(&ClientCapability::Lookup)
                        }) =>
                {
                    Some(ClientRefusal::CapabilityNotNegotiated)
                }
                WrapperToDaemon::ClientHello { .. }
                | WrapperToDaemon::SendIdempotent { .. }
                | WrapperToDaemon::Lookup { .. } => None,
                _ => Some(ClientRefusal::MessageOutsideClientRole),
            },
            Some(ConnectionRole::Wrapper) | None
                if matches!(
                    msg,
                    WrapperToDaemon::ClientHello { .. }
                        | WrapperToDaemon::SendIdempotent { .. }
                        | WrapperToDaemon::Lookup { .. }
                ) =>
            {
                Some(ClientRefusal::ClientRoleRequired)
            }
            _ => None,
        }
    };
    if let Some(reason) = client_refusal {
        return Some(DaemonToWrapper::ClientRejected { reason });
    }

    match msg {
        WrapperToDaemon::RoleHandshake { .. } => unreachable!("handshake traité avant le dispatch"),
        WrapperToDaemon::ClientHello {
            contract_version,
            issuer_scope,
            capabilities,
        } => {
            if contract_version != CLIENT_CONTRACT_VERSION {
                return Some(DaemonToWrapper::ClientRejected {
                    reason: ClientRefusal::UnsupportedVersion {
                        supported_versions: vec![CLIENT_CONTRACT_VERSION],
                    },
                });
            }
            if crate::idempotency::validate_issuer_scope(&issuer_scope).is_err() {
                return Some(DaemonToWrapper::ClientRejected {
                    reason: ClientRefusal::InvalidIssuerScope,
                });
            }
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            let active_scopes = st
                .client_negotiations
                .values()
                .map(|negotiated| negotiated.issuer_scope.as_str())
                .collect::<std::collections::HashSet<_>>();
            if !active_scopes.contains(issuer_scope.as_str())
                && active_scopes.len() >= MAX_ACTIVE_ISSUER_SCOPES
            {
                return Some(DaemonToWrapper::ClientRejected {
                    reason: ClientRefusal::ActiveScopeLimit,
                });
            }
            let capabilities: Vec<ClientCapability> = capabilities
                .into_iter()
                .filter(|capability| {
                    matches!(
                        capability,
                        ClientCapability::SendIdempotent | ClientCapability::Lookup
                    )
                })
                .collect();
            st.client_negotiations.insert(
                conn_id.to_string(),
                NegotiatedClient {
                    version: CLIENT_CONTRACT_VERSION,
                    issuer_scope,
                    capabilities: capabilities.clone(),
                },
            );
            Some(DaemonToWrapper::ClientWelcome {
                version: CLIENT_CONTRACT_VERSION,
                horizon_secs: CLIENT_IDEMPOTENCY_HORIZON_SECS,
                issued_at_tolerance_secs: CLIENT_ISSUED_AT_TOLERANCE_SECS,
                capabilities,
            })
        }
        WrapperToDaemon::SendIdempotent {
            message,
            message_id,
            issued_at,
        } => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            Some(handle_idempotent_send(
                conn_id, message, message_id, issued_at, &mut st,
            ))
        }
        WrapperToDaemon::Lookup { operation_kind, idempotency_key } => {
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            Some(handle_idempotency_lookup(conn_id, operation_kind, idempotency_key, &st))
        }
        WrapperToDaemon::DeliverAcked { .. } | WrapperToDaemon::DeliveryIndeterminate { .. } => {
            None
        }
        WrapperToDaemon::Subscribe { agent, window } => {
            let (subscription_id, control) = {
                let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                if st.connection_roles.get(conn_id) != Some(&ConnectionRole::Attach) {
                    return Some(DaemonToWrapper::AttachRejected {
                        subscription_id: None,
                        reason: AttachRefusal::MessageOutsideAttachRole,
                    });
                }
                let wrapper_conn = match attach_refusal_for_subscription(&st, &agent) {
                    Ok(connection_id) => connection_id,
                    Err(reason) => {
                        return Some(DaemonToWrapper::AttachRejected {
                            subscription_id: None,
                            reason,
                        });
                    }
                };
                let subscription_id = format!("attach-{}", uuid::Uuid::new_v4());
                let Some(view) = st.connections.get(conn_id).and_then(|writer| {
                    AttachView::start(subscription_id.clone(), writer, st.view_closed_tx.clone())
                }) else {
                    return Some(DaemonToWrapper::AttachRejected {
                        subscription_id: Some(subscription_id),
                        reason: AttachRefusal::WrapperUnavailable,
                    });
                };
                let writer = st.connections.get(&wrapper_conn).cloned();
                st.attach_subscriptions.insert(
                    subscription_id.clone(),
                    AttachSubscription {
                        agent: agent.clone(),
                        attach_conn: conn_id.to_string(),
                        wrapper_conn,
                    },
                );
                st.attach_views.insert(subscription_id.clone(), view);
                let control = writer.map(|writer| DeferredControl {
                    writer,
                    message: DaemonToWrapper::Subscribe {
                        subscription_id: subscription_id.clone(),
                        agent,
                        window,
                    },
                });
                (subscription_id, control)
            };
            let accepted =
                control.is_some_and(|control| execute_controls(vec![control]).is_empty());
            if accepted {
                None
            } else {
                let view = {
                    let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                    st.attach_subscriptions.remove(&subscription_id);
                    st.attach_views.remove(&subscription_id)
                };
                if let Some(view) = view {
                    view.close_and_join();
                }
                Some(DaemonToWrapper::AttachRejected {
                    subscription_id: Some(subscription_id),
                    reason: AttachRefusal::WrapperUnavailable,
                })
            }
        }
        WrapperToDaemon::Unsubscribe { subscription_id } => {
            let (control, view) = {
                let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                let Some(subscription) = st.attach_subscriptions.get(&subscription_id).cloned()
                else {
                    // Une fin d'ancienne génération n'a pas le droit de toucher
                    // une souscription plus récente sur la même connexion.
                    return None;
                };
                if subscription.attach_conn != conn_id {
                    return Some(DaemonToWrapper::AttachRejected {
                        subscription_id: Some(subscription_id),
                        reason: AttachRefusal::MessageOutsideAttachRole,
                    });
                }
                st.attach_subscriptions.remove(&subscription_id);
                let control = st
                    .connections
                    .get(&subscription.wrapper_conn)
                    .map(|writer| DeferredControl {
                    writer: writer.clone(),
                    message: DaemonToWrapper::Unsubscribe {
                        subscription_id: subscription_id.clone(),
                    },
                });
                (control, st.attach_views.remove(&subscription_id))
            };
            if let Some(control) = control {
                let _ = execute_controls(vec![control]);
            }
            if let Some(view) = view {
                let _ = view.enqueue(DaemonToWrapper::End {
                    subscription_id,
                    reason: "désabonnement demandé".to_string(),
                });
                view.close_and_join();
            }
            None
        }
        WrapperToDaemon::Subscribed { subscription_id } => {
            let control = {
                let st = state.lock().unwrap_or_else(|e| e.into_inner());
                let subscription = st.attach_subscriptions.get(&subscription_id).cloned()?;
                if subscription.wrapper_conn != conn_id {
                    return None;
                }
                log::debug!(
                    "abonnement attach {} accepté par {}",
                    subscription_id,
                    subscription.agent
                );
                st.connections
                    .get(&subscription.attach_conn)
                    .map(|writer| DeferredControl {
                    writer: writer.clone(),
                    message: DaemonToWrapper::Subscribed {
                        subscription_id: subscription_id.clone(),
                    },
                })
            };
            let delivered =
                control.is_some_and(|control| execute_controls(vec![control]).is_empty());
            if !delivered {
                let (control, view) = {
                    let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                    let subscription = st.attach_subscriptions.remove(&subscription_id);
                    let control = subscription.and_then(|subscription| {
                        st.connections
                            .get(&subscription.wrapper_conn)
                            .map(|writer| DeferredControl {
                            writer: writer.clone(),
                            message: DaemonToWrapper::Unsubscribe {
                                subscription_id: subscription_id.clone(),
                            },
                        })
                    });
                    (control, st.attach_views.remove(&subscription_id))
                };
                if let Some(control) = control {
                    let _ = execute_controls(vec![control]);
                }
                if let Some(view) = view {
                    view.close_and_join();
                }
            }
            None
        }
        WrapperToDaemon::AttachRejected {
            subscription_id,
            reason,
        } => {
            let (control, view) = {
                let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                let subscription_id = subscription_id?;
                let subscription = st.attach_subscriptions.get(&subscription_id).cloned()?;
                if subscription.wrapper_conn != conn_id {
                    return None;
                }
                let control =
                    st.connections
                        .get(&subscription.attach_conn)
                        .map(|writer| DeferredControl {
                    writer: writer.clone(),
                    message: DaemonToWrapper::AttachRejected {
                        subscription_id: Some(subscription_id.clone()),
                        reason,
                    },
                });
                st.attach_subscriptions.remove(&subscription_id);
                (control, st.attach_views.remove(&subscription_id))
            };
            if let Some(control) = control {
                let _ = execute_controls(vec![control]);
            }
            if let Some(view) = view {
                view.close_and_join();
            }
            None
        }
        WrapperToDaemon::End {
            subscription_id,
            reason,
        } => {
            let view = {
                let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                let subscription = st.attach_subscriptions.get(&subscription_id).cloned()?;
                if subscription.wrapper_conn != conn_id {
                    return None;
                }
                st.attach_subscriptions.remove(&subscription_id);
                st.attach_views.remove(&subscription_id)
            };
            if let Some(view) = view {
                let _ = view.enqueue(DaemonToWrapper::End {
                    subscription_id,
                    reason,
                });
                view.close_and_join();
            }
            None
        }
        WrapperToDaemon::JournalReadError {
            subscription_id,
            line,
            offset,
            reason,
        } => {
            let view = {
                let st = state.lock().unwrap_or_else(|e| e.into_inner());
                let subscription = st.attach_subscriptions.get(&subscription_id).cloned()?;
                if subscription.wrapper_conn != conn_id {
                    return None;
                }
                st.attach_views.get(&subscription_id).cloned()
            };
            if let Some(view) = view {
                let _ = view.enqueue(DaemonToWrapper::JournalReadError {
                    subscription_id,
                    line,
                    offset,
                    reason,
                });
            }
            None
        }
        WrapperToDaemon::JournalFragment {
            subscription_id,
            seq,
            offset,
            final_fragment,
            bytes,
        } => {
            let view = {
                let st = state.lock().unwrap_or_else(|e| e.into_inner());
                st.attach_subscriptions
                    .get(&subscription_id)
                    .filter(|subscription| subscription.wrapper_conn == conn_id)
                    .and_then(|_| st.attach_views.get(&subscription_id).cloned())
            };
            if let Some(view) = view {
                let _ = view.enqueue(DaemonToWrapper::JournalFragment {
                    subscription_id,
                    seq,
                    offset,
                    final_fragment,
                    bytes,
                });
            }
            None
        }
        WrapperToDaemon::SnapshotCaughtUp {
            subscription_id,
            through_seq,
        } => {
            let view = {
                let st = state.lock().unwrap_or_else(|e| e.into_inner());
                st.attach_subscriptions
                    .get(&subscription_id)
                    .filter(|subscription| subscription.wrapper_conn == conn_id)
                    .and_then(|_| st.attach_views.get(&subscription_id).cloned())
            };
            if let Some(view) = view {
                let _ = view.enqueue(DaemonToWrapper::SnapshotCaughtUp {
                    subscription_id,
                    through_seq,
                });
            }
            None
        }
        WrapperToDaemon::Gap {
            subscription_id,
            from_seq,
            to_seq,
            reason,
        } => {
            let view = {
                let st = state.lock().unwrap_or_else(|e| e.into_inner());
                st.attach_subscriptions
                    .get(&subscription_id)
                    .filter(|subscription| subscription.wrapper_conn == conn_id)
                    .and_then(|_| st.attach_views.get(&subscription_id).cloned())
            };
            if let Some(view) = view {
                let _ = view.enqueue(DaemonToWrapper::Gap {
                    subscription_id,
                    from_seq,
                    to_seq,
                    reason,
                });
            }
            None
        }
        WrapperToDaemon::Register {
            agent_type,
            name,
            host,
            transport,
            os,
            instance_id,
            domain,
            turn_in_progress,
        } => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            let response = handle_register(
                conn_id,
                agent_type,
                name,
                host,
                transport,
                os,
                instance_id,
                domain,
                turn_in_progress,
                &mut st,
            );
            Some(response)
        }

        WrapperToDaemon::Unregister => {
            let (controls, views) = {
                let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                let (controls, views) = close_attach_subscriptions(&mut st, conn_id);
                st.router.unregister_by_conn(conn_id);
                st.mark_stopped(conn_id);
                st.conn_names.remove(conn_id);
                st.conn_hosts.remove(conn_id);
                st.conn_operating_systems.remove(conn_id);
                (controls, views)
            };
            let _ = execute_controls(controls);
            for view in views {
                view.close_and_join();
            }
            None
        }

        WrapperToDaemon::Rename { current_name, name } => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            let target_conn = match st.router.get_agent(&current_name) {
                Some(agent) => agent.connection_id.clone(),
                None => {
                    return Some(DaemonToWrapper::Nack {
                        id: "rename".to_string(),
                        reason: format!("agent introuvable: {}", current_name),
                    });
                }
            };
            match st.router.rename(&target_conn, &name) {
                Ok((old_name, new_name)) => {
                    st.conn_names.insert(target_conn, new_name.clone());
                    info!("agent '{}' renommé en '{}'", old_name, new_name);
                    Some(DaemonToWrapper::Renamed {
                        old_name,
                        name: new_name,
                    })
                }
                Err(error) => Some(DaemonToWrapper::Nack {
                    id: "rename".to_string(),
                    reason: error.to_string(),
                }),
            }
        }

        WrapperToDaemon::Send(mut bridge_msg) => {
            eprintln!(
                "[BRIDGET] Send de {}: to={}, body={}",
                conn_id,
                bridge_msg.to,
                bridge_msg.body.chars().take(40).collect::<String>()
            );
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            let is_attach = st.connection_roles.get(conn_id) == Some(&ConnectionRole::Attach);
            purge_expired_attach_sends(&mut st);
            if is_attach && st.pending_attach_sends.len() >= MAX_PENDING_ATTACH_SENDS {
                return Some(DaemonToWrapper::Nack {
                    id: bridge_msg.id.clone(),
                    reason: "trop d envois humains en attente".to_string(),
                });
            }
            let sender_name = st.conn_names.get(conn_id).cloned().unwrap_or_default();
            // Résolution de l'expéditeur :
            // - Si la connexion est un wrapper (agent enregistré sous son vrai nom),
            //   utiliser ce nom.
            // - Si la connexion est un CLI temporaire (cli-send-XXXXX), vérifier si
            //   le from du message correspond à un agent enregistré (ex: codex-1).
            //   Si oui, utiliser ce from (le CLI a été lancé depuis l'intérieur du wrapper).
            //   Si non, garder le from tel quel (envoi depuis terminal externe).
            if is_attach {
                // Le rôle attach ne peut pas emprunter l'identité d'un wrapper
                // ni répondre à une demande suivie. T804a conservera cette
                // règle quand il raccordera les envois aux abonnements.
                bridge_msg.from = "humain".to_string();
                bridge_msg.in_reply_to = None;
            } else if !sender_name.is_empty() && !sender_name.starts_with("cli-send-") {
                // Wrapper : utiliser le nom enregistré
                bridge_msg.from = sender_name.clone();
            } else if st.router.get_agent(&bridge_msg.from).is_some() {
                // CLI temporaire mais le from correspond à un agent enregistré
                // → confiance accordée (le CLI tourne dans le contexte du wrapper)
                // On garde bridge_msg.from tel quel
            } else {
                // CLI temporaire avec from inconnu
                // → utiliser le nom de connexion (cli-send-XXXXX ou human)
                if !sender_name.is_empty() {
                    bridge_msg.from = sender_name.clone();
                }
            }

            let logical_sender = bridge_msg.from.clone();
            let content_key = bridge_msg.content_key();
            let message_guard_id = bridge_msg.id.clone();
            let prepared = match prepare_dispatch(
                &mut st,
                &mut bridge_msg,
                conn_id,
                logical_sender,
                content_key,
                message_guard_id,
            ) {
                Ok(prepared) => prepared,
                Err((_category, reason)) => {
                    return Some(DaemonToWrapper::Nack {
                        id: bridge_msg.id.clone(),
                        reason,
                    });
                }
            };
            let is_ephemeral_cli_sender = sender_name.starts_with("cli-send-")
                && prepared.reply_sender_conn.as_deref() == Some(conn_id);
            if bridge_msg.reply && is_ephemeral_cli_sender {
                return Some(DaemonToWrapper::Nack {
                    id: bridge_msg.id.clone(),
                    reason: "--reply requiert un agent Bridget connecté ; lance la commande depuis un wrapper actif ou envoie sans --reply".to_string(),
                });
            }
            let target_conn = prepared.target_conn;
                    let conv_key = format!("{}|{}", bridge_msg.from, bridge_msg.to);

                    if let Err(e) = st.store.record_message(&bridge_msg, &conv_key) {
                        error!("store: {}", e);
                    }
                    st.circuit_breaker
                        .record(&prepared.logical_sender, &bridge_msg.to);
                    st.deduplicator
                        .mark_sent(&prepared.content_key, &bridge_msg.to);
                    st.envelope_guard
                        .mark_relayed(&prepared.message_guard_id, &bridge_msg.to);

                    // Le daemon est l'autorité de l'échéance : le wrapper ACP
                    // reçoit sa valeur absolue pour purger un tour devenu trop
                    // tardif juste avant `session/prompt`.
                    let mut delivered_message = bridge_msg.clone();
                    if delivered_message.reply {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        delivered_message.deadline_at =
                            Some(now.saturating_add(delivered_message.reply_timeout.unwrap_or(60)));
                    }
                    // Push vers le destinataire
                    let dtw = DaemonToWrapper::Deliver(delivered_message);
                    let json = encode(&dtw).unwrap_or_default();
                    eprintln!("[BRIDGET] Push vers {}: {} octets", target_conn, json.len());

                    let mut delivery_succeeded = false;
                    if let Some(target_writer) = st.connections.get(&target_conn) {
                        log::debug!("push vers {}: écriture sur writer", target_conn);
                        if let Ok(mut w) = target_writer.lock() {
                            eprintln!("[BRIDGET] Writer locked for {}, écriture...", target_conn);
                            match writeln!(w, "{}", json) {
                                Ok(_) if w.flush().is_ok() => {
                                    delivery_succeeded = true;
                                    info!(
                                        "livré: {} → « {} » (hops={}, reply={})",
                                        bridge_msg.id,
                                        bridge_msg.to,
                                        bridge_msg.hops,
                                        bridge_msg.reply
                                    );
                                }
                                Ok(_) => error!("push {}: flush échoué", target_conn),
                                Err(e) => error!("push {}: {}", target_conn, e),
                            }
                        }
                    } else {
                        warn!("cible {} disparue", target_conn);
                    }

                    if delivery_succeeded
                        && prepared.valid_tracked_reply
                        && let Some(request_id) = bridge_msg.in_reply_to.as_deref()
                        && st
                            .store
                            .mark_answered(request_id, &bridge_msg.from, &bridge_msg.to)
                            .unwrap_or(false)
                    {
                        st.pending_replies
                            .retain(|pending| pending.msg_id != request_id);
                        info!("demande {} répondue après livraison", request_id);
                    }

                    if let Err(reason) = track_reply_cycle(
                        &mut st,
                        &bridge_msg,
                        prepared.reply_sender_conn.clone(),
                        target_conn.clone(),
                    ) {
                        return Some(DaemonToWrapper::Nack {
                            id: bridge_msg.id.clone(),
                            reason,
                        });
                    }

                    if is_attach {
                        st.pending_attach_sends.insert(
                            bridge_msg.id.clone(),
                            PendingAttachSend {
                                conn_id: conn_id.to_string(),
                                expires_at: Instant::now() + PENDING_ATTACH_SEND_TTL,
                            },
                        );
                    }
                    Some(DaemonToWrapper::Ack {
                        id: bridge_msg.id.clone(),
                    })
        }

        WrapperToDaemon::Heartbeat => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(instance_id) = st.conn_instances.get(conn_id).cloned()
                && let Some(presence) = st.presences.get_mut(&instance_id)
            {
                    presence.last_seen = Instant::now();
                }
            None
        }

        WrapperToDaemon::TurnState { in_progress } => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            match st.set_turn_state(conn_id, in_progress) {
                Ok(()) => None,
                Err(reason) => Some(DaemonToWrapper::Nack {
                    id: "turn-state".to_string(),
                    reason,
                }),
            }
        }

        WrapperToDaemon::ListAgents => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            let agents = st.agent_infos();
            Some(DaemonToWrapper::AgentList { agents })
        }

        WrapperToDaemon::Runtime {
            agent,
            model,
            effort,
            source,
        } => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            Some(handle_runtime(&agent, model, effort, source, &mut st))
        }

        WrapperToDaemon::Domain { agent, domain } => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            Some(handle_domain(&agent, domain, &mut st))
        }

        WrapperToDaemon::Availability { agent, until_secs } => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            Some(handle_availability(&agent, until_secs, &mut st))
        }

        WrapperToDaemon::DeliveryRejected { id, reason } => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            purge_expired_attach_sends(&mut st);
            if let Some(pending) = st.pending_attach_sends.remove(&id) {
                let writer = st.connections.get(&pending.conn_id).cloned();
                drop(st);
                if let Some(writer) = writer {
                    let _ = push_control_message(
                        &writer,
                        &DaemonToWrapper::DeliveryRejected { id, reason },
                    );
                }
                return None;
            }
            let request = st.store.get_request(&id).ok().flatten();
            if let Some(request) = request {
                st.pending_replies.retain(|pending| pending.msg_id != id);
                if (reason.contains("échéance") || reason.contains("timeout ACP"))
                    && !claim_timeout(&st.store, &id)
                {
                        return None;
                }
                if let Some(agent) = st.router.get_agent(&request.sender)
                    && let Some(writer) = st.connections.get(&agent.connection_id)
                    && let Err(error) = deliver_to_agent(
                        writer,
                        &request.sender,
                        &format!("Échec de livraison de la demande #{id} : {reason}"),
                    )
                {
                    error!(
                        "impossible de notifier l'échec ACP à {}: {}",
                        request.sender, error
                    );
                }
            }
            None
        }

        WrapperToDaemon::CancelRequest { id, sender, reason } => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            if st.router.get_agent(&sender).is_none() {
                return Some(DaemonToWrapper::Nack {
                    id,
                    reason: "annulation réservée à un agent Bridget connecté".to_string(),
                });
            }
            match st.store.cancel_request(&id, &sender, reason.as_deref()) {
                Ok(Some(request)) if request.sender != sender => Some(DaemonToWrapper::Nack {
                    id,
                    reason: "seul l'émetteur peut annuler cette demande".to_string(),
                }),
                Ok(Some(request)) if request.state == "cancelled" => {
                    st.pending_replies
                        .retain(|pending| pending.msg_id != request.id);
                    if let Some(agent) = st.router.get_agent(&request.target)
                        && let Some(writer) = st.connections.get(&agent.connection_id)
                    {
                        let is_acp = st
                            .conn_instances
                            .get(&agent.connection_id)
                            .and_then(|instance| st.presences.get(instance))
                            .is_some_and(|presence| presence.transport == "acp");
                        if is_acp {
                            let cancel = DaemonToWrapper::CancelDelivery {
                                id: request.id.clone(),
                                reason: request
                                    .cancel_reason
                                    .clone()
                                    .unwrap_or_else(|| "demande annulée".to_string()),
                            };
                            match encode(&cancel).map_err(|error| error.to_string()).and_then(
                                |json| {
                                    let mut writer =
                                        writer.lock().map_err(|error| error.to_string())?;
                                    writeln!(writer, "{json}")
                                        .map_err(|error| error.to_string())?;
                                    writer.flush().map_err(|error| error.to_string())
                                },
                            ) {
                                Ok(()) => {}
                                Err(error) => error!(
                                    "Impossible de signaler l'annulation à {}: {}",
                                    request.target, error
                                ),
                            }
                        } else if let Err(error) = deliver_to_agent(
                            writer,
                            &request.target,
                            &format!(
                                "Demande #{} annulée par {}. Aucune réponse n'est requise.{}",
                                request.id,
                                sender,
                                request
                                    .cancel_reason
                                    .as_deref()
                                    .map(|reason| format!(" Motif : {reason}"))
                                    .unwrap_or_default()
                            ),
                        ) {
                            error!(
                                "Impossible de délivrer l'annulation à {}: {}",
                                request.target, error
                            );
                        }
                    }
                    Some(DaemonToWrapper::RequestCancelled {
                        id: request.id,
                        state: request.state,
                    })
                }
                Ok(Some(request)) => Some(DaemonToWrapper::Nack {
                    id,
                    reason: format!("demande déjà terminale: {}", request.state),
                }),
                Ok(None) => Some(DaemonToWrapper::Nack {
                    id,
                    reason: "demande introuvable".to_string(),
                }),
                Err(error) => Some(DaemonToWrapper::Nack {
                    id,
                    reason: error.to_string(),
                }),
            }
        }

        WrapperToDaemon::ListRequests { sender } => {
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            match st.store.requests_for_sender(&sender) {
                Ok(requests) => match requests
                        .into_iter()
                        .map(|request| -> Result<_, crate::store::StoreError> {
                            let deferred = st.store.latest_deferred_reminder(&request.id)?;
                            Ok(bridget_transport::protocol::RequestInfo {
                                id: request.id,
                                target: request.target,
                                state: request.state,
                                deadline_at: request.deadline_at,
                                cancel_reason: request.cancel_reason,
                                deferred_reminder_level: deferred.map(|event| event.0),
                                deferred_reminder_at: deferred.map(|event| event.1),
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()
                {
                    Ok(requests) => Some(DaemonToWrapper::RequestList { requests }),
                    Err(error) => Some(DaemonToWrapper::Nack {
                        id: "requests".to_string(),
                        reason: error.to_string(),
                    }),
                },
                Err(error) => Some(DaemonToWrapper::Nack {
                    id: "requests".to_string(),
                    reason: error.to_string(),
                }),
            }
        }
    }
}

/// Statut du daemon — interroge le daemon via la socket locale.
pub fn get_status(config: &DaemonConfig) -> DaemonStatus {
    use std::io::{BufRead, BufReader, BufWriter, Write};
    use std::os::unix::net::UnixStream;

    if !config.socket_path.exists() {
        return DaemonStatus::default();
    }

    let stream = match UnixStream::connect(&config.socket_path) {
        Ok(s) => s,
        Err(_) => return DaemonStatus::default(),
    };

    let read_stream = match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return DaemonStatus::default(),
    };

    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(read_stream);

    // Register
    let reg = WrapperToDaemon::Register {
        agent_type: "status-probe".to_string(),
        name: Some(format!("status-{}", std::process::id())),
        host: None,
        transport: None,
        os: None,
        instance_id: None,
        domain: None,
        turn_in_progress: false,
    };
    let reg_json = match encode(&reg) {
        Ok(j) => j,
        Err(_) => return DaemonStatus::default(),
    };
    if writeln!(writer, "{}", reg_json).is_err() {
        return DaemonStatus::default();
    }
    if writer.flush().is_err() {
        return DaemonStatus::default();
    }

    // Lire Registered
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return DaemonStatus::default();
    }

    // Demander la liste des agents
    let list_req = WrapperToDaemon::ListAgents;
    let list_json = match encode(&list_req) {
        Ok(j) => j,
        Err(_) => return DaemonStatus::default(),
    };
    if writeln!(writer, "{}", list_json).is_err() {
        return DaemonStatus::default();
    }
    if writer.flush().is_err() {
        return DaemonStatus::default();
    }

    // Lire AgentList
    let mut resp_line = String::new();
    if reader.read_line(&mut resp_line).is_err() {
        return DaemonStatus::default();
    }
    let agents = match decode::<DaemonToWrapper>(resp_line.trim()) {
        Ok(DaemonToWrapper::AgentList { agents }) => agents
            .into_iter()
            .filter(|agent| agent.agent_type != "status-probe")
            .collect(),
        _ => vec![],
    };

    // Compter les messages en base
    let message_count = match Store::open(&config.db_path) {
        Ok(store) => store.recent_messages(1000).map(|v| v.len()).unwrap_or(0),
        Err(_) => 0,
    };

    DaemonStatus {
        running: true,
        agents,
        message_count,
    }
}

#[derive(Default)]
pub struct DaemonStatus {
    pub running: bool,
    pub agents: Vec<bridget_transport::protocol::AgentInfo>,
    pub message_count: usize,
}

#[cfg(test)]
mod presence_tests {
    use super::*;
    use bridget_core::BridgetMessage;
    use std::io::Read;

    fn control_socket(label: &str) -> (Arc<Mutex<BufWriter<UnixStream>>>, BufReader<UnixStream>) {
        let path = std::env::temp_dir().join(format!(
            "bc-{label}-{}.sock",
            &uuid::Uuid::new_v4().simple().to_string()[..12]
        ));
        let listener = UnixListener::bind(&path).unwrap();
        let client = UnixStream::connect(&path).unwrap();
        let (server, _) = listener.accept().unwrap();
        std::fs::remove_file(path).unwrap();
        (
            Arc::new(Mutex::new(BufWriter::new(server))),
            BufReader::new(client),
        )
    }

    fn read_control(reader: &mut BufReader<UnixStream>) -> DaemonToWrapper {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        decode(line.trim()).unwrap()
    }

    fn normalized_attach_frame(mut message: DaemonToWrapper) -> String {
        match &mut message {
            DaemonToWrapper::JournalFragment {
                subscription_id, ..
            }
            | DaemonToWrapper::SnapshotCaughtUp {
                subscription_id, ..
            }
            | DaemonToWrapper::Gap {
                subscription_id, ..
            }
            | DaemonToWrapper::JournalReadError {
                subscription_id, ..
            }
            | DaemonToWrapper::End {
                subscription_id, ..
            }
            | DaemonToWrapper::Subscribed { subscription_id }
            | DaemonToWrapper::Subscribe {
                subscription_id, ..
            }
            | DaemonToWrapper::Unsubscribe { subscription_id } => subscription_id.clear(),
            DaemonToWrapper::AttachRejected {
                subscription_id, ..
            } => *subscription_id = None,
            _ => {}
        }
        encode(&message).unwrap()
    }

    fn install_attach_view(
        state: &mut DaemonState,
        subscription_id: &str,
        attach_conn: &str,
        writer: &Arc<Mutex<BufWriter<UnixStream>>>,
    ) {
        let view = AttachView::start(
            subscription_id.to_string(),
            writer,
            state.view_closed_tx.clone(),
        )
        .unwrap();
        state.attach_subscriptions.insert(
            subscription_id.to_string(),
            AttachSubscription {
                agent: "agent-2".to_string(),
                attach_conn: attach_conn.to_string(),
                wrapper_conn: "conn-1".to_string(),
            },
        );
        state.attach_views.insert(subscription_id.to_string(), view);
    }

    #[test]
    fn unreachable_presence_remains_visible() {
        let base = std::env::temp_dir().join(format!("bridget-presence-{}", std::process::id()));
        let config = DaemonConfig {
            socket_path: base.with_extension("sock"),
            db_path: base.with_extension("db"),
            log_path: base.with_extension("log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        };
        let mut state = DaemonState::new(&config).unwrap();
        state
            .router
            .register(
                Some("agent-distant-1"),
                &bridget_core::AgentType::Codex,
                "conn-1",
            )
            .unwrap();
        state
            .conn_instances
            .insert("conn-1".to_string(), "instance-1".to_string());
        state.presences.insert(
            "instance-1".to_string(),
            Presence {
                name: "agent-distant-1".to_string(),
                agent_type: "codex".to_string(),
                host: "projet-a".to_string(),
                transport: "ssh-unix".to_string(),
                os: "Linux".to_string(),
                state: "connected".to_string(),
                last_seen: Instant::now(),
                reconnect_count: 0,
                model: Some("gpt-5.3-codex".to_string()),
                effort: Some("xhigh".to_string()),
                derived_domain: Some("projet-a".to_string()),
                domain: Some("projet-a".to_string()),
                dnd_until: None,
            },
        );
        state.router.unregister_by_conn("conn-1");
        state.mark_unreachable("conn-1");

        let agents = state.agent_infos();
        assert_eq!(agents.len(), 1);
        assert_eq!(
            agents
                .iter()
                .map(|agent| agent.name.as_str())
                .collect::<Vec<_>>(),
            vec!["agent-distant-1"]
        );
        assert_eq!(agents[0].host, "projet-a");
        assert_eq!(agents[0].os, "Linux");
        assert_eq!(agents[0].state, "unreachable");
        // FR-010 : la dernière capacité connue survit à la perte de connexion.
        assert_eq!(agents[0].model.as_deref(), Some("gpt-5.3-codex"));
        assert_eq!(agents[0].effort.as_deref(), Some("xhigh"));
        if let Err(e) = std::fs::remove_file(&config.db_path) {
            log::warn!(
                "Impossible de supprimer la base {}: {}",
                config.db_path.display(),
                e
            );
        }
    }

    /// Construit un état minimal avec un agent enregistré et sa présence.
    fn state_with_registered_agent(label: &str) -> (DaemonState, DaemonConfig) {
        let base = std::env::temp_dir().join(format!("bridget-{}-{}", label, std::process::id()));
        let config = DaemonConfig {
            socket_path: base.with_extension("sock"),
            db_path: base.with_extension("db"),
            log_path: base.with_extension("log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        };
        let mut state = DaemonState::new(&config).unwrap();
        state
            .router
            .register(Some("agent-2"), &bridget_core::AgentType::Claude, "conn-1")
            .unwrap();
        state
            .conn_instances
            .insert("conn-1".to_string(), "instance-1".to_string());
        state.presences.insert(
            "instance-1".to_string(),
            Presence {
                name: "agent-2".to_string(),
                agent_type: "claude".to_string(),
                host: "macbook".to_string(),
                transport: "acp".to_string(),
                os: "macOS".to_string(),
                state: "connected".to_string(),
                last_seen: Instant::now(),
                reconnect_count: 0,
                model: None,
                effort: None,
                derived_domain: None,
                domain: None,
                dnd_until: None,
            },
        );
        (state, config)
    }

    #[test]
    fn connexion_attach_refuse_un_message_reserve_au_wrapper() {
        let (state, config) = state_with_registered_agent("attach-role");
        let shared = Arc::new(Mutex::new(state));

        let accepted = handle_wrapper_message(
            "attach-1",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach,
            },
            &shared,
        );
        assert!(matches!(
            accepted,
            Some(DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Attach
            })
        ));

        let refusal = handle_wrapper_message(
            "attach-1",
            WrapperToDaemon::Runtime {
                agent: "agent-2".to_string(),
                model: "gpt-5.5".to_string(),
                effort: None,
                source: bridget_transport::protocol::RuntimeSource::Declared,
            },
            &shared,
        );
        assert!(matches!(
            refusal,
            Some(DaemonToWrapper::AttachRejected {
                reason: AttachRefusal::MessageOutsideAttachRole,
                ..
            })
        ));
        assert_eq!(shared.lock().unwrap().router.list_agents().len(), 1);
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn premier_message_wrapper_fige_le_role_avant_un_handshake_tardif() {
        let (state, config) = state_with_registered_agent("role-fige");
        let shared = Arc::new(Mutex::new(state));

        let first = handle_wrapper_message(
            "conn-role",
            WrapperToDaemon::Runtime {
                agent: "inconnu".to_string(),
                model: "modele".to_string(),
                effort: None,
                source: bridget_transport::protocol::RuntimeSource::Declared,
            },
            &shared,
        );
        assert!(matches!(first, Some(DaemonToWrapper::Nack { .. })));
        assert_eq!(
            shared.lock().unwrap().connection_roles.get("conn-role"),
            Some(&ConnectionRole::Wrapper)
        );

        let handshake = handle_wrapper_message(
            "conn-role",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach,
            },
            &shared,
        );
        assert!(matches!(
            handshake,
            Some(DaemonToWrapper::AttachRejected {
                reason: AttachRefusal::MessageOutsideAttachRole,
                ..
            })
        ));
        assert_eq!(
            shared.lock().unwrap().connection_roles.get("conn-role"),
            Some(&ConnectionRole::Wrapper)
        );
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn client_negocie_un_contrat_versionne_apres_son_role() {
        let (state, config) = state_with_registered_agent("client-negotiation");
        let shared = Arc::new(Mutex::new(state));
        assert!(matches!(
            handle_wrapper_message(
                "client-1",
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Client
                },
                &shared,
            ),
            Some(DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Client
            })
        ));
        let response = handle_wrapper_message(
            "client-1",
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: "012_scope_aaaaaaaaaaaa".to_string(),
                capabilities: vec![ClientCapability::SendIdempotent, ClientCapability::Lookup],
            },
            &shared,
        );
        assert!(matches!(
            response,
            Some(DaemonToWrapper::ClientWelcome {
                version: CLIENT_CONTRACT_VERSION,
                horizon_secs: CLIENT_IDEMPOTENCY_HORIZON_SECS,
                issued_at_tolerance_secs: CLIENT_ISSUED_AT_TOLERANCE_SECS,
                capabilities,
            }) if capabilities == vec![ClientCapability::SendIdempotent, ClientCapability::Lookup]
        ));
        let negotiated = shared
            .lock()
            .unwrap()
            .client_negotiations
            .get("client-1")
            .cloned()
            .unwrap();
        assert_eq!(negotiated.version, CLIENT_CONTRACT_VERSION);
        assert_eq!(negotiated.issuer_scope, "012_scope_aaaaaaaaaaaa");
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn client_hello_avant_role_et_depuis_attach_sont_refuses_sans_negociation() {
        let (state, config) = state_with_registered_agent("client-order");
        let shared = Arc::new(Mutex::new(state));
        let hello = || WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: "012_scope_bbbbbbbbbbbb".to_string(),
            capabilities: vec![ClientCapability::Lookup],
        };
        assert!(matches!(
            handle_wrapper_message("client-before-role", hello(), &shared),
            Some(DaemonToWrapper::ClientRejected {
                reason: ClientRefusal::RoleHandshakeRequired
            })
        ));
        assert!(
            !shared
            .lock()
            .unwrap()
            .connection_roles
                .contains_key("client-before-role")
        );
        assert!(matches!(
            handle_wrapper_message(
                "attach-client-hello",
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Attach
                },
                &shared,
            ),
            Some(DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Attach
            })
        ));
        assert!(matches!(
            handle_wrapper_message("attach-client-hello", hello(), &shared),
            Some(DaemonToWrapper::AttachRejected {
                reason: AttachRefusal::MessageOutsideAttachRole,
                ..
            })
        ));
        assert!(shared.lock().unwrap().client_negotiations.is_empty());
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn send_et_lookup_avant_welcome_ne_negocient_ni_ne_reservent() {
        let (state, config) = state_with_registered_agent("client-before-welcome");
        let shared = Arc::new(Mutex::new(state));
        handle_wrapper_message(
            "client-before-welcome",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client,
            },
            &shared,
        );
        let message = BridgetMessage::new("client", "agent-2", "sans welcome");
        assert!(matches!(
            handle_wrapper_message(
                "client-before-welcome",
                WrapperToDaemon::SendIdempotent {
                    message,
                    message_id: "message-1".to_string(),
                    issued_at: 1,
                },
                &shared,
            ),
            Some(DaemonToWrapper::ClientRejected {
                reason: ClientRefusal::NegotiationRequired
            })
        ));
        assert!(matches!(
            handle_wrapper_message(
                "client-before-welcome",
                WrapperToDaemon::Lookup {
                    operation_kind: "send".to_string(),
                    idempotency_key: "message-1".to_string(),
                },
                &shared,
            ),
            Some(DaemonToWrapper::ClientRejected {
                reason: ClientRefusal::NegotiationRequired
            })
        ));
        assert!(shared.lock().unwrap().client_negotiations.is_empty());
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn client_refuse_version_et_scope_invalides_et_capacite_non_negociee() {
        let (state, config) = state_with_registered_agent("client-validation");
        let shared = Arc::new(Mutex::new(state));
        for connection in ["client-version", "client-scope", "client-capability"] {
            handle_wrapper_message(
                connection,
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Client,
                },
                &shared,
            );
        }
        assert!(matches!(
            handle_wrapper_message(
                "client-version",
                WrapperToDaemon::ClientHello {
                    contract_version: CLIENT_CONTRACT_VERSION + 1,
                    issuer_scope: "012_scope_cccccccccccc".to_string(),
                    capabilities: vec![ClientCapability::Lookup],
                },
                &shared,
            ),
            Some(DaemonToWrapper::ClientRejected {
                reason: ClientRefusal::UnsupportedVersion { supported_versions }
            }) if supported_versions == vec![CLIENT_CONTRACT_VERSION]
        ));
        assert!(matches!(
            handle_wrapper_message(
                "client-scope",
                WrapperToDaemon::ClientHello {
                    contract_version: CLIENT_CONTRACT_VERSION,
                    issuer_scope: "invalide!".to_string(),
                    capabilities: vec![ClientCapability::Lookup],
                },
                &shared,
            ),
            Some(DaemonToWrapper::ClientRejected {
                reason: ClientRefusal::InvalidIssuerScope
            })
        ));
        handle_wrapper_message(
            "client-capability",
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: "012_scope_dddddddddddd".to_string(),
                capabilities: vec![ClientCapability::Lookup],
            },
            &shared,
        );
        assert!(matches!(
            handle_wrapper_message(
                "client-capability",
                WrapperToDaemon::SendIdempotent {
                    message: BridgetMessage::new("client", "agent-2", "hors capacite"),
                    message_id: "message-2".to_string(),
                    issued_at: 1,
                },
                &shared,
            ),
            Some(DaemonToWrapper::ClientRejected {
                reason: ClientRefusal::CapabilityNotNegotiated
            })
        ));
        assert!(
            shared
                .lock()
                .unwrap()
                .client_negotiations
                .contains_key("client-capability")
        );
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn client_borne_les_scopes_actifs_sans_rejeter_un_scope_deja_actif() {
        let (mut state, config) = state_with_registered_agent("client-scope-limit");
        for number in 0..MAX_ACTIVE_ISSUER_SCOPES {
            state.client_negotiations.insert(
                format!("existing-{number}"),
                NegotiatedClient {
                    version: CLIENT_CONTRACT_VERSION,
                    issuer_scope: format!("012_scope_{number:016x}"),
                    capabilities: vec![ClientCapability::Lookup],
                },
            );
        }
        state
            .connection_roles
            .insert("client-limit".to_string(), ConnectionRole::Client);
        let shared = Arc::new(Mutex::new(state));
        assert!(matches!(
            handle_wrapper_message(
                "client-limit",
                WrapperToDaemon::ClientHello {
                    contract_version: CLIENT_CONTRACT_VERSION,
                    issuer_scope: "012_scope_ffffffffffffffff".to_string(),
                    capabilities: vec![ClientCapability::Lookup],
                },
                &shared,
            ),
            Some(DaemonToWrapper::ClientRejected {
                reason: ClientRefusal::ActiveScopeLimit
            })
        ));
        shared
            .lock()
            .unwrap()
            .connection_roles
            .insert("client-retry".to_string(), ConnectionRole::Client);
        assert!(matches!(
            handle_wrapper_message(
                "client-retry",
                WrapperToDaemon::ClientHello {
                    contract_version: CLIENT_CONTRACT_VERSION,
                    issuer_scope: "012_scope_0000000000000000".to_string(),
                    capabilities: vec![ClientCapability::Lookup],
                },
                &shared,
            ),
            Some(DaemonToWrapper::ClientWelcome { .. })
        ));
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn wrapper_accepte_les_accuses_idempotents_et_register_historique_reste_wrapper() {
        let (state, config) = state_with_registered_agent("client-wrapper-matrix");
        let shared = Arc::new(Mutex::new(state));
        assert!(
            handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::DeliverAcked {
                delivery_id: "delivery-1".to_string(),
                delivery_generation: 1,
            },
            &shared,
        )
            .is_none()
        );
        assert_eq!(
            shared.lock().unwrap().connection_roles.get("conn-1"),
            Some(&ConnectionRole::Wrapper)
        );
        assert!(matches!(
            handle_wrapper_message(
                "historic-register",
                WrapperToDaemon::Register {
                    agent_type: "codex".to_string(),
                    name: Some("historique-012".to_string()),
                    host: None,
                    transport: None,
                    os: None,
                    instance_id: None,
                    domain: None,
                    turn_in_progress: false,
                },
                &shared,
            ),
            Some(DaemonToWrapper::Registered { .. })
        ));
        assert_eq!(
            shared
                .lock()
                .unwrap()
                .connection_roles
                .get("historic-register"),
            Some(&ConnectionRole::Wrapper)
        );
        assert_eq!(shared.lock().unwrap().idempotency.record_count().unwrap(), 0);
        let _ = std::fs::remove_file(config.db_path);
    }

    fn negotiate_idempotent_client(
        shared: &Arc<Mutex<DaemonState>>,
        connection: &str,
        scope: &str,
    ) {
        assert!(matches!(
            handle_wrapper_message(
                connection,
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Client,
                },
                shared,
            ),
            Some(DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Client
            })
        ));
        assert!(matches!(
            handle_wrapper_message(
                connection,
                WrapperToDaemon::ClientHello {
                    contract_version: CLIENT_CONTRACT_VERSION,
                    issuer_scope: scope.to_string(),
                    capabilities: vec![ClientCapability::SendIdempotent],
                },
                shared,
            ),
            Some(DaemonToWrapper::ClientWelcome { .. })
        ));
    }

    fn idempotent_message(body: &str) -> BridgetMessage {
        let mut message = BridgetMessage::new("maicie", "agent-2", body);
        message.hops = 4;
        message
    }

    #[test]
    fn canonical_send_ignore_le_nom_affiche_et_le_timeout_relatif() {
        let mut original = idempotent_message("enveloppe stable");
        original.reply = true;
        original.reply_timeout = Some(20);
        original.deadline_at = Some(123_456);
        let mut renamed = original.clone();
        renamed.from = "maicie-renommee".to_string();
        renamed.reply_timeout = Some(90);
        assert_eq!(
            canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &original, 123_000),
            canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &renamed, 123_000)
        );
    }

    #[test]
    fn send_idempotent_rejoue_sans_rerouter_et_isole_les_scopes() {
        let (state, config) = state_with_registered_agent("idempotent-send");
        let shared = Arc::new(Mutex::new(state));
        let scope_a = "012_scope_aaaaaaaaaaaa";
        let scope_b = "012_scope_bbbbbbbbbbbb";
        negotiate_idempotent_client(&shared, "client-a", scope_a);
        let issued_at = unix_now_secs();
        let first = handle_wrapper_message(
            "client-a",
            WrapperToDaemon::SendIdempotent {
                message: idempotent_message("tâche durable"),
                message_id: "message-identique".to_string(),
                issued_at,
            },
            &shared,
        );
        let first_delivery = match first {
            Some(DaemonToWrapper::IdempotencyResult {
                issue:
                    IdempotencyIssue::OutcomeUnknown {
                        delivery_id: Some(delivery_id),
                        ..
                    },
                ..
            }) => delivery_id,
            other => panic!("réponse inattendue: {other:?}"),
        };
        negotiate_idempotent_client(&shared, "client-b", scope_b);
        let other_scope = handle_wrapper_message(
            "client-b",
            WrapperToDaemon::SendIdempotent {
                message: idempotent_message("tâche durable"),
                message_id: "message-identique".to_string(),
                issued_at,
            },
            &shared,
        );
        assert!(matches!(
            other_scope,
            Some(DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown {
                    delivery_id: Some(delivery_id),
                    ..
                },
                ..
            }) if delivery_id != first_delivery
        ));
        let state = shared.lock().unwrap();
        let first_key = IdempotencyKey::new(scope_a, OperationKind::Send, "message-identique").unwrap();
        let second_key = IdempotencyKey::new(scope_b, OperationKind::Send, "message-identique").unwrap();
        assert_ne!(
            state.idempotency.send_delivery(&first_key).unwrap().unwrap().delivery_generation,
            state.idempotency.send_delivery(&second_key).unwrap().unwrap().delivery_generation
        );
        drop(state);
        // Un rejeu est jugé avant tout routage : retirer ou renommer la cible
        // n'autorise jamais une nouvelle résolution pour cette clé connue.
        {
            let mut state = shared.lock().unwrap();
            state.router.rename("conn-1", "agent-renommé").unwrap();
        }
        let replay = handle_wrapper_message(
            "client-a",
            WrapperToDaemon::SendIdempotent {
                message: idempotent_message("tâche durable"),
                message_id: "message-identique".to_string(),
                issued_at,
            },
            &shared,
        );
        assert!(matches!(
            replay,
            Some(DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown {
                    delivery_id: Some(delivery_id),
                    ..
                },
                ..
            }) if delivery_id == first_delivery
        ));
        let mismatch = handle_wrapper_message(
            "client-a",
            WrapperToDaemon::SendIdempotent {
                message: idempotent_message("tâche différente"),
                message_id: "message-identique".to_string(),
                issued_at,
            },
            &shared,
        );
        assert!(matches!(
            mismatch,
            Some(DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::EnvelopeMismatch,
                ..
            })
        ));
        let state = shared.lock().unwrap();
        assert_eq!(
            state.idempotency.record_count().unwrap(),
            2
        );
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn send_idempotent_persiste_un_refus_et_la_voie_historique_ne_contamine_pas_le_socle() {
        let (state, config) = state_with_registered_agent("idempotent-rejection");
        let shared = Arc::new(Mutex::new(state));
        negotiate_idempotent_client(&shared, "client-reject", "012_scope_cccccccccccc");
        let issued_at = unix_now_secs();
        let mut unknown_target = BridgetMessage::new("maicie", "inconnu", "à refuser");
        unknown_target.hops = 4;
        let send = || WrapperToDaemon::SendIdempotent {
            message: unknown_target.clone(),
            message_id: "message-refuse".to_string(),
            issued_at,
        };
        let rejected = handle_wrapper_message("client-reject", send(), &shared);
        assert!(matches!(
            rejected,
            Some(DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::Rejected { ref category, .. },
                ..
            }) if category == "routing"
        ));
        let replay = handle_wrapper_message("client-reject", send(), &shared);
        assert!(matches!(
            replay,
            Some(DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::Rejected { ref category, .. },
                ..
            }) if category == "routing"
        ));

        let historic = handle_wrapper_message(
            "historique-012",
            WrapperToDaemon::Send(BridgetMessage::new(
                "historique",
                "agent-2",
                "ancienne voie",
            )),
            &shared,
        );
        assert!(matches!(historic, Some(DaemonToWrapper::Ack { .. })));
        assert_eq!(
            shared.lock().unwrap().idempotency.record_count().unwrap(),
            1
        );
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn send_idempotent_conserve_les_invariants_de_reponse_et_dnd() {
        let (mut state, config) = state_with_registered_agent("idempotent-reply");
        state
            .router
            .register(
                Some("maicie"),
                &bridget_core::AgentType::Codex,
                "sender-wrapper",
            )
            .unwrap();
        state
            .store
            .create_request("request-open", "agent-2", "maicie", 60)
            .unwrap();
        state.presences.get_mut("instance-1").unwrap().dnd_until =
            Some(Instant::now() + Duration::from_secs(60));
        let shared = Arc::new(Mutex::new(state));
        negotiate_idempotent_client(&shared, "client-reply", "012_scope_replyyyyyyyyy");
        let mut reply = idempotent_message("réponse suivie");
        reply.from = "maicie".to_string();
        reply.reply = true;
        reply.in_reply_to = Some("request-open".to_string());
        let result = handle_wrapper_message(
            "client-reply",
            WrapperToDaemon::SendIdempotent {
                message: reply,
                message_id: "reply-idempotent".to_string(),
                issued_at: unix_now_secs(),
            },
            &shared,
        );
        assert!(matches!(
            result,
            Some(DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            })
        ));
        let state = shared.lock().unwrap();
        assert_eq!(
            state
                .store
                .get_request("reply-idempotent")
                .unwrap()
                .unwrap()
                .state,
            "open"
        );
        assert!(state
            .pending_replies
            .iter()
            .any(|pending| pending.msg_id == "reply-idempotent"));
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn envoi_attach_force_humain_et_ne_clot_pas_une_demande_forgee() {
        let (mut state, config) = state_with_registered_agent("attach-humain");
        let listener =
            UnixListener::bind(config.socket_path.with_extension("target.sock")).unwrap();
        let _receiver =
            UnixStream::connect(config.socket_path.with_extension("target.sock")).unwrap();
        let (target_stream, _) = listener.accept().unwrap();
        state.connections.insert(
            "conn-1".to_string(),
            Arc::new(Mutex::new(BufWriter::new(target_stream))),
        );
        state
            .store
            .create_request("request-1", "agent-2", "codex-1", 60)
            .unwrap();
        let shared = Arc::new(Mutex::new(state));
        handle_wrapper_message(
            "attach-1",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach,
            },
            &shared,
        );

        let mut forged = bridget_core::BridgetMessage::new("codex-1", "agent-2", "réponse forgée");
        forged.in_reply_to = Some("request-1".to_string());
        let response = handle_wrapper_message("attach-1", WrapperToDaemon::Send(forged), &shared);
        assert!(matches!(response, Some(DaemonToWrapper::Ack { .. })));

        let state = shared.lock().unwrap();
        assert_eq!(state.store.recent_messages(1).unwrap()[0].sender, "humain");
        assert_eq!(
            state.store.get_request("request-1").unwrap().unwrap().state,
            "open"
        );
        let _ = std::fs::remove_file(config.db_path);
        let _ = std::fs::remove_file(config.socket_path.with_extension("target.sock"));
    }

    #[test]
    fn abonnement_attach_attend_le_wrapper_et_ignore_la_fin_d_une_ancienne_generation() {
        let (mut state, config) = state_with_registered_agent("attach-cycle");
        let (wrapper_writer, mut wrapper_reader) = control_socket("wrapper");
        let (attach_writer, mut attach_reader) = control_socket("attach");
        state
            .connections
            .insert("conn-1".to_string(), wrapper_writer);
        state
            .connections
            .insert("attach-1".to_string(), attach_writer);
        let shared = Arc::new(Mutex::new(state));
        assert!(matches!(
            handle_wrapper_message(
                "attach-1",
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Attach,
                },
                &shared,
            ),
            Some(DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Attach
            })
        ));

        assert!(
            handle_wrapper_message(
            "attach-1",
            WrapperToDaemon::Subscribe {
                agent: "agent-2".to_string(),
                window: bridget_transport::AttachWindow::Today,
            },
            &shared,
        )
            .is_none()
        );
        let first_id = match read_control(&mut wrapper_reader) {
            DaemonToWrapper::Subscribe {
                subscription_id, ..
            } => subscription_id,
            other => panic!("commande wrapper inattendue: {}", encode(&other).unwrap()),
        };
        assert_eq!(
            shared.lock().unwrap().agent_infos().len(),
            1,
            "attach reste hors de who"
        );
        assert!(
            shared
                .lock()
                .unwrap()
                .attach_subscriptions
                .contains_key(&first_id)
        );
        assert!(
            handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::Subscribed {
                subscription_id: first_id.clone(),
            },
            &shared,
        )
            .is_none()
        );
        assert!(matches!(
            read_control(&mut attach_reader),
            DaemonToWrapper::Subscribed { subscription_id } if subscription_id == first_id
        ));
        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::JournalReadError {
                subscription_id: first_id.clone(),
                line: 12,
                offset: 384,
                reason: "ligne corrompue".to_string(),
            },
            &shared,
        );
        assert!(matches!(
            read_control(&mut attach_reader),
            DaemonToWrapper::JournalReadError {
                subscription_id,
                line: 12,
                offset: 384,
                reason,
            } if subscription_id == first_id && reason == "ligne corrompue"
        ));

        handle_wrapper_message(
            "attach-1",
            WrapperToDaemon::Subscribe {
                agent: "agent-2".to_string(),
                window: bridget_transport::AttachWindow::Seq(8),
            },
            &shared,
        );
        let second_id = match read_control(&mut wrapper_reader) {
            DaemonToWrapper::Subscribe {
                subscription_id, ..
            } => subscription_id,
            other => panic!("commande wrapper inattendue: {}", encode(&other).unwrap()),
        };
        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::Subscribed {
                subscription_id: second_id.clone(),
            },
            &shared,
        );
        assert!(matches!(
            read_control(&mut attach_reader),
            DaemonToWrapper::Subscribed { subscription_id } if subscription_id == second_id
        ));

        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::End {
                subscription_id: first_id.clone(),
                reason: "ancienne génération".to_string(),
            },
            &shared,
        );
        assert!(matches!(
            read_control(&mut attach_reader),
            DaemonToWrapper::End { subscription_id, .. } if subscription_id == first_id
        ));
        assert!(
            shared
                .lock()
                .unwrap()
                .attach_subscriptions
                .contains_key(&second_id)
        );

        let (controls, views) = {
            let mut state = shared.lock().unwrap();
            let (controls, views) = close_attach_subscriptions(&mut state, "attach-1");
            assert!(state.attach_subscriptions.is_empty());
            (controls, views)
        };
        assert!(execute_controls(controls).is_empty());
        for view in views {
            view.close_and_join();
        }
        assert!(matches!(
            read_control(&mut wrapper_reader),
            DaemonToWrapper::Unsubscribe { subscription_id } if subscription_id == second_id
        ));
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn abonnement_attach_refuse_un_wrapper_tmux_malgre_son_type() {
        let (mut state, config) = state_with_registered_agent("attach-tmux");
        state.presences.get_mut("instance-1").unwrap().transport = "unix".to_string();
        let shared = Arc::new(Mutex::new(state));
        assert!(matches!(
            handle_wrapper_message(
                "attach-1",
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Attach,
                },
                &shared,
            ),
            Some(DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Attach
            })
        ));
        assert!(matches!(
            handle_wrapper_message(
                "attach-1",
                WrapperToDaemon::Subscribe {
                    agent: "agent-2".to_string(),
                    window: bridget_transport::AttachWindow::Today,
                },
                &shared,
            ),
            Some(DaemonToWrapper::AttachRejected {
                reason: AttachRefusal::AgentNotAcp,
                ..
            })
        ));
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn confirmation_vers_vue_fermee_desabonne_le_wrapper() {
        let (mut state, config) = state_with_registered_agent("attach-closed-view");
        let (wrapper_writer, mut wrapper_reader) = control_socket("wrapper-closed-view");
        let (attach_writer, attach_reader) = control_socket("attach-closed-view");
        state
            .connections
            .insert("conn-1".to_string(), wrapper_writer);
        state
            .connections
            .insert("attach-1".to_string(), attach_writer);
        let shared = Arc::new(Mutex::new(state));
        handle_wrapper_message(
            "attach-1",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach,
            },
            &shared,
        );
        handle_wrapper_message(
            "attach-1",
            WrapperToDaemon::Subscribe {
                agent: "agent-2".to_string(),
                window: bridget_transport::AttachWindow::Today,
            },
            &shared,
        );
        let subscription_id = match read_control(&mut wrapper_reader) {
            DaemonToWrapper::Subscribe {
                subscription_id, ..
            } => subscription_id,
            other => panic!("commande wrapper inattendue: {}", encode(&other).unwrap()),
        };
        drop(attach_reader);
        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::Subscribed {
                subscription_id: subscription_id.clone(),
            },
            &shared,
        );
        assert!(matches!(
            read_control(&mut wrapper_reader),
            DaemonToWrapper::Unsubscribe { subscription_id: id } if id == subscription_id
        ));
        assert!(shared.lock().unwrap().attach_subscriptions.is_empty());
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn vue_suspendue_coalesce_un_gap_et_ne_garde_pas_de_fragment_orphelin() {
        let view = AttachView::suspended("sub-gap");
        let bytes = vec![b'x'; 600 * 1024];
        assert!(view.enqueue(DaemonToWrapper::JournalFragment {
            subscription_id: "sub-gap".to_string(),
            seq: 41,
            offset: 0,
            final_fragment: false,
            bytes: bytes.clone(),
        }));
        assert!(view.enqueue(DaemonToWrapper::JournalFragment {
            subscription_id: "sub-gap".to_string(),
            seq: 42,
            offset: 0,
            final_fragment: true,
            bytes,
        }));
        let queue = view.queue.0.lock().unwrap();
        assert_eq!(queue.gap.as_ref().map(|gap| (gap.0, gap.1)), Some((41, 41)));
        assert_eq!(queue.messages.len(), 1);
        assert_eq!(
            queue.messages.front().and_then(|message| message.seq),
            Some(42)
        );
    }

    #[test]
    fn eviction_ignore_les_fragments_restants_de_la_sequence_lachee() {
        let view = AttachView::suspended("sub-fragments");
        let bytes = vec![b'x'; 600 * 1024];
        assert!(view.enqueue(DaemonToWrapper::JournalFragment {
            subscription_id: "sub-fragments".to_string(),
            seq: 41,
            offset: 0,
            final_fragment: false,
            bytes: bytes.clone(),
        }));
        assert!(!view.enqueue(DaemonToWrapper::JournalFragment {
            subscription_id: "sub-fragments".to_string(),
            seq: 41,
            offset: 1,
            final_fragment: false,
            bytes: bytes.clone(),
        }));
        assert!(!view.enqueue(DaemonToWrapper::JournalFragment {
            subscription_id: "sub-fragments".to_string(),
            seq: 41,
            offset: 2,
            final_fragment: false,
            bytes: b"continuation evincee".to_vec(),
        }));
        assert!(!view.enqueue(DaemonToWrapper::JournalFragment {
            subscription_id: "sub-fragments".to_string(),
            seq: 41,
            offset: 3,
            final_fragment: true,
            bytes: b"frontiere evincee".to_vec(),
        }));
        assert!(view.enqueue(DaemonToWrapper::JournalFragment {
            subscription_id: "sub-fragments".to_string(),
            seq: 42,
            offset: 4,
            final_fragment: true,
            bytes: b"sequence suivante admise".to_vec(),
        }));
        let queue = view.queue.0.lock().unwrap();
        assert_eq!(queue.dropping_seq, None);
        assert!(queue.messages.iter().all(|message| message.seq != Some(41)));
        assert_eq!(queue.gap.as_ref().map(|gap| (gap.0, gap.1)), Some((41, 41)));
        assert_eq!(
            queue
                .messages
                .iter()
                .filter(|message| message.seq == Some(42))
                .count(),
            1
        );
    }

    #[test]
    fn saturation_non_sequencee_notifie_et_recolte_la_vue_exactement_une_fois() {
        let (mut state, config) = state_with_registered_agent("attach-non-seq-close");
        let (wrapper_writer, _wrapper_reader) = control_socket("attach-non-seq-wrapper");
        let (attach_writer, _attach_reader) = control_socket("attach-non-seq-view");
        let view = AttachView::suspended_with_sender("sub-non-seq", state.view_closed_tx.clone());
        state
            .connections
            .insert("conn-1".to_string(), wrapper_writer);
        state
            .connections
            .insert("attach-1".to_string(), attach_writer);
        state.attach_subscriptions.insert(
            "sub-non-seq".to_string(),
            AttachSubscription {
                agent: "agent-2".to_string(),
                attach_conn: "attach-1".to_string(),
            wrapper_conn: "conn-1".to_string(),
            },
        );
        state
            .attach_views
            .insert("sub-non-seq".to_string(), view.clone());
        {
            let mut queue = view.queue.0.lock().unwrap();
            queue.bytes = ATTACH_VIEW_BUFFER_BYTES;
        }
        assert!(!view.enqueue(DaemonToWrapper::SnapshotCaughtUp {
            subscription_id: "sub-non-seq".to_string(),
            through_seq: Some(7),
        }));
        assert!(!view.enqueue(DaemonToWrapper::Gap {
            subscription_id: "sub-non-seq".to_string(),
            from_seq: 8,
            to_seq: 9,
            reason: None,
        }));
        let (controls, views) = collect_closed_attach_views(&mut state);
        let _ = execute_controls(controls);
        for view in views {
            view.close_and_join();
        }
        assert!(state.attach_subscriptions.is_empty());
        assert!(state.attach_views.is_empty());
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn end_abandonne_la_file_et_interdit_toute_frame_ulterieure() {
        let view = AttachView::suspended("sub-terminal");
        assert!(view.enqueue(DaemonToWrapper::JournalFragment {
            subscription_id: "sub-terminal".to_string(),
            seq: 5,
            offset: 0,
            final_fragment: true,
            bytes: b"avant-end".to_vec(),
        }));
        assert!(view.enqueue(DaemonToWrapper::End {
            subscription_id: "sub-terminal".to_string(),
            reason: "fin".to_string(),
        }));
        assert!(!view.enqueue(DaemonToWrapper::JournalReadError {
            subscription_id: "sub-terminal".to_string(),
            line: 7,
            offset: 42,
            reason: "trop tard".to_string(),
        }));
        let queue = view.queue.0.lock().unwrap();
        assert_eq!(queue.messages.len(), 1);
        assert!(
            queue
                .messages
                .front()
                .is_some_and(|message| message.terminal)
        );
        assert!(queue.messages.iter().all(|message| message.seq.is_none()));
    }

    #[test]
    fn producteurs_de_vue_et_controle_partagent_des_lignes_json_decodables() {
        let (writer_stream, reader_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(writer_stream)));
        let (closed_tx, _closed_rx) = mpsc::channel();
        let view = AttachView::start("sub-serialise".to_string(), &writer, closed_tx).unwrap();
        let reader = thread::spawn(move || {
            let mut reader = BufReader::new(reader_stream);
            (0..33)
                .map(|_| {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    decode::<DaemonToWrapper>(line.trim()).is_ok()
                })
                .collect::<Vec<_>>()
        });
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let control_barrier = barrier.clone();
        let control_writer = writer.clone();
        let controls = thread::spawn(move || {
            control_barrier.wait();
            let controls = (0..16)
                .map(|number| DeferredControl {
                    writer: control_writer.clone(),
                    message: DaemonToWrapper::Ack {
                        id: format!("ack-{number}"),
                    },
                })
                .chain(std::iter::once(DeferredControl {
                    writer: control_writer.clone(),
                    message: DaemonToWrapper::End {
                        subscription_id: "sub-serialise".to_string(),
                        reason: "fin".to_string(),
                    },
                }))
                .collect();
            execute_controls(controls)
        });
        for seq in 0..16 {
            assert!(view.enqueue(DaemonToWrapper::JournalFragment {
                subscription_id: "sub-serialise".to_string(),
                seq,
                offset: seq,
                final_fragment: true,
                bytes: vec![b'x'; 2048],
            }));
        }
        barrier.wait();
        assert!(controls.join().unwrap().is_empty());
        assert!(reader.join().unwrap().into_iter().all(|decoded| decoded));
        view.close_and_join();
    }

    #[test]
    fn end_indelivrable_force_l_eof_de_la_vue() {
        let (writer_stream, mut peer) = UnixStream::pair().unwrap();
        writer_stream
            .set_write_timeout(Some(Duration::from_millis(50)))
            .unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(writer_stream)));
        // Remplir le tampon noyau sans lire le pair, puis échouer sur End.
        let payload = "x".repeat(16 * 1024);
        while push_control_message(
            &writer,
            &DaemonToWrapper::Ack {
                id: payload.clone(),
            },
        ) {}
        assert!(!push_control_message(
            &writer,
            &DaemonToWrapper::End {
                subscription_id: "sub-eof".to_string(),
                reason: "vue trop lente".to_string(),
            }
        ));
        peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let mut bytes = [0_u8; 1024];
        loop {
            match peer.read(&mut bytes) {
                Ok(0) => break,
                Ok(_) => continue,
                Err(error)
                    if error.kind() == std::io::ErrorKind::WouldBlock
                        || error.kind() == std::io::ErrorKind::TimedOut =>
                {
                        panic!("End ni EOF recus par la vue")
                    }
                Err(error) => panic!("lecture pair: {error}"),
            }
        }
    }

    #[test]
    fn rejet_tardif_attach_est_reroute_puis_purge() {
        let (mut state, config) = state_with_registered_agent("attach-late-reject");
        let (writer, mut reader) = control_socket("attach-late-reject");
        state.connections.insert("attach-1".to_string(), writer);
        state.pending_attach_sends.insert(
            "message-humain".to_string(),
            PendingAttachSend {
                conn_id: "attach-1".to_string(),
                expires_at: Instant::now() + Duration::from_secs(60),
            },
        );
        let shared = Arc::new(Mutex::new(state));
        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::DeliveryRejected {
                id: "message-humain".to_string(),
                reason: "adaptateur refusé".to_string(),
            },
            &shared,
        );
        assert!(matches!(
            read_control(&mut reader),
            DaemonToWrapper::DeliveryRejected { id, reason }
                if id == "message-humain" && reason == "adaptateur refusé"
        ));
        assert!(shared.lock().unwrap().pending_attach_sends.is_empty());
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn pending_attach_expire_et_sont_nettoyes_a_la_deconnexion() {
        let (mut state, config) = state_with_registered_agent("attach-purge");
        state.pending_attach_sends.insert(
            "expired".to_string(),
            PendingAttachSend {
                conn_id: "attach-1".to_string(),
                expires_at: Instant::now() - Duration::from_secs(1),
            },
        );
        purge_expired_attach_sends(&mut state);
        assert!(state.pending_attach_sends.is_empty());
        state.pending_attach_sends.insert(
            "live".to_string(),
            PendingAttachSend {
                conn_id: "attach-1".to_string(),
                expires_at: Instant::now() + Duration::from_secs(60),
            },
        );
        let (controls, views) = close_attach_subscriptions(&mut state, "attach-1");
        assert!(controls.is_empty());
        for view in views {
            view.close_and_join();
        }
        assert!(state.pending_attach_sends.is_empty());
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn deux_vues_attach_recoivent_le_meme_flux_et_sont_recoltees() {
        let (mut state, config) = state_with_registered_agent("attach-two-views");
        let (wrapper_writer, _wrapper_reader) = control_socket("attach-two-wrapper");
        let (first_writer, mut first_reader) = control_socket("attach-two-first");
        let (second_writer, mut second_reader) = control_socket("attach-two-second");
        state
            .connections
            .insert("conn-1".to_string(), wrapper_writer);
        state
            .connections
            .insert("attach-1".to_string(), first_writer.clone());
        state
            .connections
            .insert("attach-2".to_string(), second_writer.clone());
        install_attach_view(&mut state, "sub-first", "attach-1", &first_writer);
        install_attach_view(&mut state, "sub-second", "attach-2", &second_writer);
        let shared = Arc::new(Mutex::new(state));

        for subscription_id in ["sub-first", "sub-second"] {
            handle_wrapper_message(
                "conn-1",
                WrapperToDaemon::JournalFragment {
                    subscription_id: subscription_id.to_string(),
                    seq: 7,
                    offset: 12,
                    final_fragment: true,
                    bytes: b"ligne complete".to_vec(),
                },
                &shared,
            );
            handle_wrapper_message(
                "conn-1",
                WrapperToDaemon::SnapshotCaughtUp {
                    subscription_id: subscription_id.to_string(),
                    through_seq: Some(7),
                },
                &shared,
            );
            handle_wrapper_message(
                "conn-1",
                WrapperToDaemon::Gap {
                    subscription_id: subscription_id.to_string(),
                    from_seq: 8,
                    to_seq: 9,
                    reason: Some("retard".to_string()),
                },
                &shared,
            );
        }

        let first = (0..3)
            .map(|_| normalized_attach_frame(read_control(&mut first_reader)))
            .collect::<Vec<_>>();
        let second = (0..3)
            .map(|_| normalized_attach_frame(read_control(&mut second_reader)))
            .collect::<Vec<_>>();
        assert_eq!(
            first, second,
            "les deux vues doivent recevoir les memes trames"
        );

        let (controls, views) = {
            let mut state = shared.lock().unwrap();
            let (controls, views) = close_attach_subscriptions(&mut state, "attach-1");
            let (second_controls, second_views) =
                close_attach_subscriptions(&mut state, "attach-2");
            assert!(state.attach_subscriptions.is_empty());
            assert!(state.attach_views.is_empty());
            (
                controls
                    .into_iter()
                    .chain(second_controls)
                    .collect::<Vec<_>>(),
                views.into_iter().chain(second_views).collect::<Vec<_>>(),
            )
        };
        let _ = execute_controls(controls);
        for view in &views {
            view.close_and_join();
            assert!(view.worker.lock().unwrap().is_none());
        }
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn vue_lente_est_recoltee_sans_bloquer_le_daemon_ni_les_autres_vues() {
        let (mut state, config) = state_with_registered_agent("attach-slow-view");
        let (wrapper_stream, _wrapper_peer) = UnixStream::pair().unwrap();
        let (slow_stream, _slow_peer) = UnixStream::pair().unwrap();
        let (fast_stream, fast_peer) = UnixStream::pair().unwrap();
        fast_peer
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let wrapper_writer = Arc::new(Mutex::new(BufWriter::new(wrapper_stream)));
        let slow_writer = Arc::new(Mutex::new(BufWriter::new(slow_stream)));
        let fast_writer = Arc::new(Mutex::new(BufWriter::new(fast_stream)));
        state
            .connections
            .insert("conn-1".to_string(), wrapper_writer);
        state
            .connections
            .insert("attach-slow".to_string(), slow_writer.clone());
        state
            .connections
            .insert("attach-fast".to_string(), fast_writer.clone());
        install_attach_view(&mut state, "sub-slow", "attach-slow", &slow_writer);
        install_attach_view(&mut state, "sub-fast", "attach-fast", &fast_writer);
        let shared = Arc::new(Mutex::new(state));

        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::JournalFragment {
                subscription_id: "sub-fast".to_string(),
                seq: 1,
                offset: 0,
                final_fragment: true,
                bytes: b"temoin".to_vec(),
            },
            &shared,
        );
        let mut fast_reader = BufReader::new(fast_peer);
        assert!(matches!(
            read_control(&mut fast_reader),
            DaemonToWrapper::JournalFragment { subscription_id, seq: 1, .. } if subscription_id == "sub-fast"
        ));

        // Le pair lent n'est jamais lu : le worker doit atteindre son timeout
        // d'écriture, tandis que le chemin daemon garde le verrou disponible.
        for seq in 1..=512 {
            handle_wrapper_message(
                "conn-1",
                WrapperToDaemon::JournalFragment {
                    subscription_id: "sub-slow".to_string(),
                    seq,
                    offset: seq * 4096,
                    final_fragment: true,
                    bytes: vec![b'x'; 4096],
                },
                &shared,
            );
        }
        let (observed_tx, observed_rx) = mpsc::channel();
        let observer_state = shared.clone();
        thread::spawn(move || {
            let count = observer_state.lock().unwrap().agent_infos().len();
            let _ = observed_tx.send(count);
        });
        assert_eq!(
            observed_rx
                .recv_timeout(Duration::from_millis(100))
                .unwrap(),
            1
        );

        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            handle_wrapper_message("conn-1", WrapperToDaemon::Heartbeat, &shared);
            if !shared
                .lock()
                .unwrap()
                .attach_subscriptions
                .contains_key("sub-slow")
            {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "la vue lente doit etre fermee dans la borne"
            );
            thread::sleep(Duration::from_millis(10));
        }
        let state = shared.lock().unwrap();
        assert!(!state.attach_views.contains_key("sub-slow"));
        assert!(state.attach_subscriptions.contains_key("sub-fast"));
        drop(state);

        let (controls, views) = {
            let mut state = shared.lock().unwrap();
            close_attach_subscriptions(&mut state, "attach-fast")
        };
        let _ = execute_controls(controls);
        for view in views {
            view.close_and_join();
        }
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn runtime_observation_remplace_le_couple_en_bloc() {
        use bridget_transport::protocol::RuntimeSource;
        let (mut state, config) = state_with_registered_agent("runtime-atomique");

        // Observation initiale : un modèle qui expose son niveau d'effort.
        let ack = handle_runtime(
            "agent-2",
            "claude-opus-5".to_string(),
            Some("high".to_string()),
            RuntimeSource::ClaudeHook,
            &mut state,
        );
        assert!(matches!(ack, DaemonToWrapper::Ack { .. }));
        let agents = state.agent_infos();
        assert_eq!(agents[0].model.as_deref(), Some("claude-opus-5"));
        assert_eq!(agents[0].effort.as_deref(), Some("high"));

        // Bascule vers un modèle sans niveau d'effort : l'ancien effort DOIT
        // disparaître. Le conserver afficherait « haiku + high », capacité qui
        // n'a jamais existé. Défaut soulevé par la contre-revue « agent-1 ».
        handle_runtime(
            "agent-2",
            "claude-haiku-4-5-20251001".to_string(),
            None,
            RuntimeSource::ClaudeHook,
            &mut state,
        );
        let agents = state.agent_infos();
        assert_eq!(
            agents[0].model.as_deref(),
            Some("claude-haiku-4-5-20251001")
        );
        assert_eq!(agents[0].effort, None);

        let _ = std::fs::remove_file(&config.db_path);
    }

    #[test]
    fn les_rappels_epargnent_un_agent_qui_ne_veut_pas_etre_derange() {
        // Destinataire joignable : tous les paliers passent.
        assert!(should_remind(false, 0));
        assert!(should_remind(false, 1));
        assert!(should_remind(false, 2));

        // Destinataire en « ne pas déranger » : les rappels qui lui sont
        // adressés sont retenus…
        assert!(!should_remind(true, 0));
        assert!(!should_remind(true, 1));
        // …mais pas la notification d'échec, qui part vers l'émetteur.
        assert!(should_remind(true, 2));
        assert!(should_remind(true, 3));
    }

    #[test]
    fn domaine_surcharge_puis_reinitialise() {
        let (mut state, config) = state_with_registered_agent("domaine");
        // Domaine dérivé annoncé à l'enregistrement.
        if let Some(presence) = state.presences.get_mut("instance-1") {
            presence.derived_domain = Some("bridget".to_string());
            presence.domain = Some("bridget".to_string());
        }
        assert_eq!(state.agent_infos()[0].domain.as_deref(), Some("bridget"));

        let ack = handle_domain("agent-2", Some("revue-croisee".to_string()), &mut state);
        assert!(matches!(ack, DaemonToWrapper::Ack { .. }));
        assert_eq!(
            state.agent_infos()[0].domain.as_deref(),
            Some("revue-croisee")
        );

        // La réinitialisation revient sur le domaine dérivé, pas sur rien.
        handle_domain("agent-2", None, &mut state);
        assert_eq!(state.agent_infos()[0].domain.as_deref(), Some("bridget"));

        let nack = handle_domain("inconnu", None, &mut state);
        assert!(matches!(nack, DaemonToWrapper::Nack { .. }));

        let _ = std::fs::remove_file(&config.db_path);
    }

    #[test]
    fn dnd_expire_de_lui_meme_et_se_leve_a_la_demande() {
        let (mut state, config) = state_with_registered_agent("dnd");
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Statut actif : l'état devient « dnd » et le temps restant est annoncé.
        handle_availability("agent-2", Some(now + 1800), &mut state);
        assert_eq!(state.agent_infos()[0].state, "dnd");
        let presence = state.presences.get("instance-1").unwrap();
        assert!(presence.is_dnd());
        assert!(presence.dnd_minutes_left() <= 30);

        // Une échéance déjà passée équivaut à une absence de statut : c'est ce
        // qui rend l'expiration automatique, sans tâche de fond.
        handle_availability("agent-2", Some(now - 10), &mut state);
        assert_eq!(state.agent_infos()[0].state, "connected");

        handle_availability("agent-2", Some(now + 600), &mut state);
        assert_eq!(state.agent_infos()[0].state, "dnd");
        handle_availability("agent-2", None, &mut state);
        assert_eq!(state.agent_infos()[0].state, "connected");
        assert!(!state.presences.get("instance-1").unwrap().is_dnd());

        let _ = std::fs::remove_file(&config.db_path);
    }

    #[test]
    fn runtime_refuse_hors_agent_enregistre_et_valeurs_invalides() {
        use bridget_transport::protocol::RuntimeSource;
        let (mut state, config) = state_with_registered_agent("runtime-refus");

        // Connexion inconnue du daemon.
        let nack = handle_runtime(
            "agent-inexistant",
            "modele".to_string(),
            None,
            RuntimeSource::Declared,
            &mut state,
        );
        assert!(
            matches!(nack, DaemonToWrapper::Nack { ref reason, .. } if reason.contains("introuvable"))
        );

        // Caractère de contrôle : casserait l'alignement de l'annuaire.
        let nack = handle_runtime(
            "agent-2",
            "mod\u{1b}[31mele".to_string(),
            None,
            RuntimeSource::Declared,
            &mut state,
        );
        assert!(matches!(nack, DaemonToWrapper::Nack { .. }));

        // Valeur trop longue.
        let nack = handle_runtime(
            "agent-2",
            "m".repeat(101),
            None,
            RuntimeSource::Declared,
            &mut state,
        );
        assert!(matches!(nack, DaemonToWrapper::Nack { .. }));

        // Aucun refus n'a pollué l'annuaire.
        assert_eq!(state.agent_infos()[0].model, None);

        let _ = std::fs::remove_file(&config.db_path);
    }

    #[test]
    fn runtime_survit_a_une_reconnexion_de_la_meme_instance() {
        use bridget_transport::protocol::RuntimeSource;
        let (mut state, config) = state_with_registered_agent("runtime-reconnexion");
        handle_runtime(
            "agent-2",
            "gpt-5.3-codex".to_string(),
            Some("xhigh".to_string()),
            RuntimeSource::CodexRollout,
            &mut state,
        );

        // Coupure puis réenregistrement sous la même instance.
        state.router.unregister_by_conn("conn-1");
        state.mark_unreachable("conn-1");
        let response = handle_register(
            "conn-2",
            "codex".to_string(),
            Some("agent-2".to_string()),
            Some("macbook".to_string()),
            Some("unix".to_string()),
            Some("macOS".to_string()),
            Some("instance-1".to_string()),
            Some("bridget".to_string()),
            false,
            &mut state,
        );
        assert!(matches!(response, DaemonToWrapper::Registered { .. }));

        let agents = state.agent_infos();
        assert_eq!(agents.len(), 1);
        assert_eq!(
            agents
                .iter()
                .map(|agent| agent.name.as_str())
                .collect::<Vec<_>>(),
            vec!["agent-2"]
        );
        assert_eq!(agents[0].state, "connected");
        assert_eq!(agents[0].model.as_deref(), Some("gpt-5.3-codex"));
        assert_eq!(agents[0].effort.as_deref(), Some("xhigh"));

        let _ = std::fs::remove_file(&config.db_path);
    }

    #[test]
    fn tour_busy_differe_les_rappels_sans_differe_l_echeance() {
        assert_eq!(deferred_reminder_level(true, 20, 60), Some(1));
        assert_eq!(deferred_reminder_level(true, 40, 60), Some(2));
        // La boucle traite elapsed >= T avant cet auxiliaire : busy ne peut
        // donc jamais différer le timeout du daemon.
        assert_eq!(deferred_reminder_level(true, 60, 60), Some(2));
        assert_eq!(deferred_reminder_level(false, 20, 60), None);
    }

    #[test]
    fn boucle_busy_persiste_les_reports_et_expire_une_seule_fois() {
        let (mut state, config) = state_with_registered_agent("busy-boucle");
        state.set_turn_state("conn-1", true).unwrap();
        state
            .store
            .create_request("request-busy", "sender", "agent-2", 60)
            .unwrap();
        let started = Instant::now();
        state.pending_replies.push(PendingReply {
            msg_id: "request-busy".to_string(),
            from: "sender".to_string(),
            from_conn: "conn-sender".to_string(),
            to: "agent-2".to_string(),
            target_conn: "conn-1".to_string(),
            timeout_secs: 60,
            created_at: started,
            escalation_level: 0,
            deferred_level: None,
        });

        let first = collect_reminder_actions(&mut state, started + Duration::from_secs(20));
        assert!(
            first
                .iter()
                .any(|action| matches!(action, ReminderAction::Deferred { level: 1, .. }))
        );
        assert!(!first.iter().any(|action| matches!(
            action,
            ReminderAction::Gentle { .. }
                | ReminderAction::Firm { .. }
                | ReminderAction::Timeout { .. }
        )));
        assert_eq!(
            state
                .store
                .latest_deferred_reminder("request-busy")
                .unwrap()
                .map(|event| event.0),
            Some(1)
        );

        let second = collect_reminder_actions(&mut state, started + Duration::from_secs(40));
        assert!(
            second
                .iter()
                .any(|action| matches!(action, ReminderAction::Deferred { level: 2, .. }))
        );
        assert!(!second.iter().any(|action| matches!(
            action,
            ReminderAction::Gentle { .. }
                | ReminderAction::Firm { .. }
                | ReminderAction::Timeout { .. }
        )));
        assert_eq!(
            state
                .store
                .latest_deferred_reminder("request-busy")
                .unwrap()
                .map(|event| event.0),
            Some(2)
        );

        let timeout = collect_reminder_actions(&mut state, started + Duration::from_secs(60));
        assert_eq!(
            timeout
                .iter()
                .filter(|action| matches!(action, ReminderAction::Timeout { .. }))
                .count(),
            1
        );
        assert_eq!(
            state
                .store
                .get_request("request-busy")
                .unwrap()
                .unwrap()
                .state,
            "timed_out"
        );
        let repeated = collect_reminder_actions(&mut state, started + Duration::from_secs(61));
        assert!(
            !repeated
                .iter()
                .any(|action| matches!(action, ReminderAction::Timeout { .. }))
        );
        let _ = std::fs::remove_file(&config.db_path);
    }

    #[test]
    fn reconnexion_redeclare_busy_et_arret_propre_reste_stopped() {
        let (mut state, config) = state_with_registered_agent("tour-reconnexion");
        state.set_turn_state("conn-1", true).unwrap();
        let agents = state.agent_infos();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "agent-2");
        assert_eq!(agents[0].state, "busy");
        state.router.unregister_by_conn("conn-1");
        state.mark_unreachable("conn-1");
        let response = handle_register(
            "conn-2",
            "claude".to_string(),
            Some("agent-2".to_string()),
            Some("macbook".to_string()),
            Some("acp".to_string()),
            Some("macOS".to_string()),
            Some("instance-1".to_string()),
            None,
            true,
            &mut state,
        );
        assert!(matches!(response, DaemonToWrapper::Registered { .. }));
        assert_eq!(state.agent_infos()[0].state, "busy");
        state.router.unregister_by_conn("conn-2");
        state.mark_stopped("conn-2");
        assert_eq!(state.agent_infos()[0].state, "stopped");
        let _ = std::fs::remove_file(&config.db_path);
    }

    #[test]
    fn expiration_transport_est_idempotente_cote_daemon() {
        let (mut state, config) = state_with_registered_agent("expiration-unique");
        state
            .store
            .create_request("request-timeout", "sender", "agent-2", 60)
            .unwrap();
        state.pending_replies.push(PendingReply {
            msg_id: "request-timeout".to_string(),
            from: "sender".to_string(),
            from_conn: "conn-sender".to_string(),
            to: "agent-2".to_string(),
            target_conn: "conn-1".to_string(),
            timeout_secs: 60,
            created_at: Instant::now(),
            escalation_level: 0,
            deferred_level: None,
        });
        let shared = Arc::new(Mutex::new(state));
        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::DeliveryRejected {
                id: "request-timeout".to_string(),
                reason: "échéance de livraison dépassée".to_string(),
            },
            &shared,
        );
        let state = shared.lock().unwrap();
        assert!(state.pending_replies.is_empty());
        assert_eq!(
            state
                .store
                .get_request("request-timeout")
                .unwrap()
                .unwrap()
                .state,
            "timed_out"
        );
        let _ = std::fs::remove_file(&config.db_path);
    }

    #[test]
    fn intercalage_timeout_et_transport_n_autorise_qu_une_notification() {
        let (state, config) = state_with_registered_agent("timeout-concurrent");
        state
            .store
            .create_request("request-timeout", "sender", "agent-2", 60)
            .unwrap();
        assert!(claim_timeout(&state.store, "request-timeout"));
        assert!(!claim_timeout(&state.store, "request-timeout"));
        let _ = std::fs::remove_file(&config.db_path);
    }

    #[test]
    fn vue_requests_expose_le_dernier_report_differe() {
        let (state, config) = state_with_registered_agent("vue-report");
        state
            .store
            .create_request("request-report", "agent-2", "cible", 60)
            .unwrap();
        state
            .store
            .record_deferred_reminder("request-report", 2)
            .unwrap();
        let shared = Arc::new(Mutex::new(state));
        let response = handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::ListRequests {
                sender: "agent-2".to_string(),
            },
            &shared,
        );
        match response {
            Some(DaemonToWrapper::RequestList { requests }) => {
                assert_eq!(requests.len(), 1);
                assert_eq!(requests[0].deferred_reminder_level, Some(2));
                assert!(requests[0].deferred_reminder_at.is_some());
            }
            other => panic!("réponse requests inattendue: {other:?}"),
        }
        let _ = std::fs::remove_file(&config.db_path);
    }

    #[test]
    fn open_request_is_restored_when_both_agents_reconnect() {
        let base = std::env::temp_dir().join(format!("bridget-restore-{}", std::process::id()));
        let config = DaemonConfig {
            socket_path: base.with_extension("sock"),
            db_path: base.with_extension("db"),
            log_path: base.with_extension("log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        };
        let mut state = DaemonState::new(&config).unwrap();
        state
            .store
            .create_request("request-1", "sender", "target", 60)
            .unwrap();
        state
            .router
            .register(Some("sender"), &bridget_core::AgentType::Codex, "conn-s")
            .unwrap();
        state
            .router
            .register(Some("target"), &bridget_core::AgentType::Claude, "conn-t")
            .unwrap();
        state.restore_pending_for_agent("target", "conn-t");
        assert_eq!(state.pending_replies.len(), 1);
        assert_eq!(state.pending_replies[0].msg_id, "request-1");
        if let Err(e) = std::fs::remove_file(&config.db_path) {
            log::warn!(
                "Impossible de supprimer la base {}: {}",
                config.db_path.display(),
                e
            );
        }
    }
}
