//! Daemon bridget — écoute sur socket locale Unix, route les messages
//! entre les wrappers connectés, persiste l'état en SQLite.

use bridget_core::{CircuitBreaker, Deduplicator, EnvelopeGuard, Router, RouterAction};
use bridget_transport::protocol::{decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use log::{error, info, warn};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::store::Store;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

// Métriques du daemon (M-005)
pub struct Metrics {
    pub messages_sent: AtomicU64,
    pub messages_received: AtomicU64,
    pub errors: AtomicU64,
    pub active_connections: AtomicUsize,
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
                warn!("Permissions non sécurisées sur {:?} - autres utilisateurs peuvent lire/écrire", cache_dir);
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
    connections: HashMap<String, Arc<Mutex<BufWriter<UnixStream>>>>,
    conn_names: HashMap<String, String>,
    conn_hosts: HashMap<String, String>,
    conn_operating_systems: HashMap<String, String>,
    conn_instances: HashMap<String, String>,
    presences: HashMap<String, Presence>,
    conn_counter: u64,
    /// Messages --reply en attente de réponse : (msg_id, from, to, expire_at, target_conn)
    pending_replies: Vec<PendingReply>,
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

fn deliver_to_agent(writer: &Arc<Mutex<BufWriter<UnixStream>>>, target_name: &str, body: &str) -> Result<(), DeliveryError> {
    let msg = bridget_core::BridgetMessage::new("bridget", target_name, body);
    let dtw = DaemonToWrapper::Deliver(msg);
    let json = encode(&dtw).map_err(|e| {
        error!("Erreur d'encodage message pour {}: {}", target_name, e);
        DeliveryError::Encoding(e.to_string())
    })?;

    let mut w = writer.lock().map_err(|e| {
        error!("Impossible de verrouiller le writer pour {}: {}", target_name, e);
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

// Fonction pour exposer les métriques publiquement (M-005)
pub fn get_metrics() -> &'static Metrics {
    // Note: ceci est un stub pour l'observabilité
    // Dans une implémentation complète, les métriques seraient partagées globalement
    use std::sync::OnceLock;
    static METRICS: OnceLock<Metrics> = OnceLock::new();
    METRICS.get_or_init(|| Metrics::new())
}

impl DaemonState {
    fn new(config: &DaemonConfig) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(config.socket_path.parent().unwrap())?;
        let store = Store::open(&config.db_path)?;
        Ok(DaemonState {
            router: Router::new(),
            circuit_breaker: CircuitBreaker::new(
                config.circuit_breaker_window,
                config.circuit_breaker_limit,
            ),
            deduplicator: Deduplicator::new(config.dedup_window),
            envelope_guard: EnvelopeGuard::new(Duration::from_secs(config.quarantine_window)),
            store,
            connections: HashMap::new(),
            conn_names: HashMap::new(),
            conn_hosts: HashMap::new(),
            conn_operating_systems: HashMap::new(),
            conn_instances: HashMap::new(),
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
        if let Some(instance_id) = self.conn_instances.remove(conn_id) {
            if let Some(presence) = self.presences.get_mut(&instance_id) {
                presence.state = "unreachable".to_string();
                presence.last_seen = Instant::now();
            }
        }
    }

    fn remove_presence(&mut self, conn_id: &str) {
        if let Some(instance_id) = self.conn_instances.remove(conn_id) {
            self.presences.remove(&instance_id);
        }
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
                        _ => "connected".to_string(),
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
        for presence in self
            .presences
            .values()
            .filter(|presence| presence.state != "connected")
        {
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
        .open(&pid_file)
        .map_err(|e| format!("Impossible de créer PID file {}: {}", pid_file.display(), e))?;

    // Tenter d'obtenir un verrou exclusif (non-bloquant)
    unsafe {
        if libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) != 0 {
            // Le verrou échoue = un autre daemon tourne
            eprintln!("bridget: un daemon tourne déjà (verrou sur {})", pid_file.display());
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
        if let Ok(n) = st.store.purge_older_than_days(config.retention_days) {
            if n > 0 {
                info!(
                    "purge: {} messages supprimés (> {} jours)",
                    n, config.retention_days
                );
            }
        }
    }

    // Thread de surveillance des --reply sans réponse (escalade progressive)
    // Palier 1 (T/3) : rappel discret au destinataire
    // Palier 2 (2T/3) : rappel ferme au destinataire
    // Palier 3 (T) : notification d'échec à l'émetteur
    // Après T + 30s : abandon (retiré de la liste)
    let st_reminder = state.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1)); // Réduit de 3s à 1s pour meilleure réactivité
        let now = std::time::Instant::now();

        // Collecter les actions à faire
        let actions: Vec<ReminderAction> = {
            let mut st = st_reminder.lock().unwrap_or_else(|e| e.into_inner());
            let mut actions = Vec::new();
            let mut state_updates = Vec::new();

            // Les destinataires qui refusent d'être dérangés ne reçoivent aucun
            // rappel : respecter le statut à l'aller et le violer au rappel
            // n'aurait aucun sens. La demande reste ouverte et son échéance
            // court toujours ; seule la relance est retenue.
            let undisturbed: std::collections::HashSet<String> = st
                .presences
                .values()
                .filter(|presence| presence.is_dnd())
                .map(|presence| presence.name.clone())
                .collect();

            for p in st.pending_replies.iter_mut() {
                let elapsed = now.duration_since(p.created_at).as_secs();
                let t = p.timeout_secs;

                if !should_remind(undisturbed.contains(&p.to), p.escalation_level) {
                    continue;
                }

                if p.escalation_level == 0 && elapsed >= t / 3 {
                    p.escalation_level = 1;
                    state_updates.push((p.msg_id.clone(), 1));
                    actions.push(ReminderAction::Gentle {
                        to: p.to.clone(),
                        from: p.from.clone(),
                        msg_id: p.msg_id.clone(),
                        target_conn: p.target_conn.clone(),
                    });
                } else if p.escalation_level == 1 && elapsed >= (t * 2) / 3 {
                    p.escalation_level = 2;
                    state_updates.push((p.msg_id.clone(), 2));
                    actions.push(ReminderAction::Firm {
                        to: p.to.clone(),
                        from: p.from.clone(),
                        msg_id: p.msg_id.clone(),
                        target_conn: p.target_conn.clone(),
                    });
                } else if p.escalation_level == 2 && elapsed >= t {
                    p.escalation_level = 3;
                    state_updates.push((p.msg_id.clone(), 3));
                    actions.push(ReminderAction::Timeout {
                        to: p.to.clone(),
                        from: p.from.clone(),
                        msg_id: p.msg_id.clone(),
                        from_conn: p.from_conn.clone(),
                        timeout_secs: t,
                    });
                }
            }

            // Retirer les entries au palier 3 depuis plus de 30s (abandon)
            st.pending_replies.retain(|p| {
                if p.escalation_level >= 3 {
                    let elapsed = now.duration_since(p.created_at).as_secs();
                    elapsed < p.timeout_secs + TIMEOUT_GRACE_PERIOD
                } else {
                    true
                }
            });

            for (id, level) in state_updates {
                if level == 3 {
                    let _ = st.store.mark_timed_out(&id);
                } else {
                    let _ = st.store.set_escalation_level(&id, level);
                }
            }

            actions
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
                            from, &msg_id[..msg_id.len().min(8)]
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
                            from, &msg_id[..msg_id.len().min(8)]
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
                            to, timeout_secs, &msg_id[..msg_id.len().min(8)]
                        );
                        if let Err(e) = deliver_to_agent(sender_writer, &from, &body) {
                            error!("Impossible de délivrer la notification de timeout à {}: {}", from, e);
                        }
                        info!(
                            "palier 3 (timeout notifié à {} : {} n'a pas répondu)",
                            from, to
                        );
                    }
                }
            }
        }
    });

    // Thread de purge périodique
    let st_purge = state.clone();
    let retention = config.retention_days;
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(3600));
        let st = st_purge.lock().unwrap_or_else(|e| e.into_inner());
        if let Ok(n) = st.store.purge_older_than_days(retention) {
            if n > 0 {
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
            for (_, writer) in &st.connections {
                let msg = DaemonToWrapper::Disconnect;
                if let Ok(json) = encode(&msg) {
                    if let Ok(mut w) = writer.lock() {
                        let _ = writeln!(w, "{}", json);
                        let _ = w.flush();
                    }
                }
            }
            drop(st);
            if let Err(e) = std::fs::remove_file(&config.socket_path) {
                log::warn!("Impossible de supprimer socket {}: {}", config.socket_path.display(), e);
            }
            if let Err(e) = std::fs::remove_file(&config.socket_path.with_extension("pid")) {
                log::warn!("Impossible de supprimer PID file {}: {}", config.socket_path.with_extension("pid").display(), e);
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
    let (writer_opt, removed) = {
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());

        // Récupérer le writer AVANT suppression pour nettoyage explicite
        let writer_opt = st.connections.remove(&conn_id);

        let removed = st.router.unregister_by_conn(&conn_id);
        st.mark_unreachable(&conn_id);
        st.conn_names.remove(&conn_id);
        st.conn_hosts.remove(&conn_id);
        st.conn_operating_systems.remove(&conn_id);
        (writer_opt, removed)
    };

    // Nettoyage explicite du writer pour éviter fuites de ressources
    if let Some(writer_mutex) = writer_opt {
        if let Ok(mut writer) = writer_mutex.lock() {
            use std::io::Write;
            let _ = writer.flush();
            // Le drop explicite fermera le stream proprement
        }
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
fn handle_register(
    conn_id: &str,
    agent_type: String,
    name: Option<String>,
    host: Option<String>,
    transport: Option<String>,
    os: Option<String>,
    instance_id: Option<String>,
    domain: Option<String>,
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

    match state.router.register(name.as_deref(), &parsed_type, conn_id) {
        Ok(final_name) => {
            state.conn_names.insert(conn_id.to_string(), final_name.clone());
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

                state.conn_instances.insert(conn_id.to_string(), instance_id.clone());
                state.presences.insert(
                    instance_id,
                    Presence {
                        name: final_name.clone(),
                        agent_type: parsed_type.to_string(),
                        host: host.unwrap_or_else(|| "inconnu".to_string()),
                        transport: transport.unwrap_or_else(|| "unix".to_string()),
                        os: os.unwrap_or_else(|| "inconnu".to_string()),
                        state: "connected".to_string(),
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

/// Retrouve la présence d'un agent désigné par son nom.
///
/// Même résolution que `handle_runtime` : les commandes de contrôle arrivent par
/// le client CLI, dont la connexion est éphémère et distincte de celle de
/// l'agent visé.
fn presence_of_agent<'a>(
    state: &'a mut DaemonState,
    agent: &str,
) -> Option<&'a mut Presence> {
    let instance_id = state
        .router
        .get_agent(agent)
        .map(|found| found.connection_id.clone())
        .and_then(|conn| state.conn_instances.get(&conn).cloned())?;
    state.presences.get_mut(&instance_id)
}

/// Remplace le domaine d'un agent, ou le ramène à son domaine dérivé.
fn handle_domain(
    agent: &str,
    domain: Option<String>,
    state: &mut DaemonState,
) -> DaemonToWrapper {
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
    log::debug!(
        "domaine de '{}' : {:?}",
        presence.name,
        presence.domain
    );
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

/// Traite un message wrapper et retourne une réponse optionnelle.
fn handle_wrapper_message(
    conn_id: &str,
    msg: WrapperToDaemon,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    match msg {
        WrapperToDaemon::Register {
            agent_type,
            name,
            host,
            transport,
            os,
            instance_id,
            domain,
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
                &mut st,
            );
            Some(response)
        }

        WrapperToDaemon::Unregister => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            st.router.unregister_by_conn(conn_id);
            st.remove_presence(conn_id);
            st.conn_names.remove(conn_id);
            st.conn_hosts.remove(conn_id);
            st.conn_operating_systems.remove(conn_id);
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
                    })
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
            let sender_name = st.conn_names.get(conn_id).cloned().unwrap_or_default();
            // Résolution de l'expéditeur :
            // - Si la connexion est un wrapper (agent enregistré sous son vrai nom),
            //   utiliser ce nom.
            // - Si la connexion est un CLI temporaire (cli-send-XXXXX), vérifier si
            //   le from du message correspond à un agent enregistré (ex: codex-1).
            //   Si oui, utiliser ce from (le CLI a été lancé depuis l'intérieur du wrapper).
            //   Si non, garder le from tel quel (envoi depuis terminal externe).
            if !sender_name.is_empty() && !sender_name.starts_with("cli-send-") {
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

            // Un client CLI temporaire se déconnecte dès qu'il a reçu l'Ack.
            // Il ne peut donc pas recevoir une réponse différée. Une demande
            // `--reply` n'est valide que si l'identité de l'émetteur désigne un
            // wrapper encore connecté ; sinon on refuse l'envoi plutôt que de
            // livrer une tâche dont la réponse sera inévitablement rejetée.
            let reply_sender_conn = st
                .router
                .get_agent(&bridge_msg.from)
                .map(|agent| agent.connection_id.clone());
            let is_ephemeral_cli_sender = sender_name.starts_with("cli-send-")
                && reply_sender_conn.as_deref() == Some(conn_id);
            if bridge_msg.reply && (reply_sender_conn.is_none() || is_ephemeral_cli_sender) {
                return Some(DaemonToWrapper::Nack {
                    id: bridge_msg.id.clone(),
                    reason: "--reply requiert un agent Bridget connecté ; lance la commande depuis un wrapper actif ou envoie sans --reply".to_string(),
                });
            }

            if let Some(request_id) = bridge_msg.in_reply_to.as_deref() {
                if st
                    .store
                    .mark_answered(request_id, &bridge_msg.from, &bridge_msg.to)
                    .unwrap_or(false)
                {
                    st.pending_replies
                        .retain(|pending| pending.msg_id != request_id);
                    info!("demande {} répondue", request_id);
                }
            }

            // 1. Disjoncteur
            if !st.circuit_breaker.check(&bridge_msg.from, &bridge_msg.to) {
                warn!(
                    "DISJONCTEUR: {} ↔ {} — {} échanges dans la fenêtre",
                    bridge_msg.from,
                    bridge_msg.to,
                    st.circuit_breaker.count(&bridge_msg.from, &bridge_msg.to)
                );
                return Some(DaemonToWrapper::Nack {
                    id: bridge_msg.id.clone(),
                    reason: format!(
                        "disjoncteur: limite {} échanges / {}s",
                        st.circuit_breaker.limit(),
                        st.circuit_breaker.window_secs()
                    ),
                });
            }

            // 2. Déduplication par contenu
            let content_key = bridge_msg.content_key();
            if st.deduplicator.is_duplicate(&content_key, &bridge_msg.to) {
                warn!(
                    "DEDUP: doublon vers « {} » (clé {})",
                    bridge_msg.to, content_key
                );
                return Some(DaemonToWrapper::Nack {
                    id: bridge_msg.id.clone(),
                    reason: "doublon de contenu".to_string(),
                });
            }

            // 3. Quarantaine par ID
            if st
                .envelope_guard
                .is_quarantined(&bridge_msg.id, &bridge_msg.to)
            {
                warn!(
                    "QUARANTAINE: id {} déjà relayé vers « {} »",
                    bridge_msg.id, bridge_msg.to
                );
                return Some(DaemonToWrapper::Nack {
                    id: bridge_msg.id.clone(),
                    reason: "message déjà relayé (quarantaine)".to_string(),
                });
            }

            // 4. Décrémenter les hops
            if !bridge_msg.decrement_hops() {
                return Some(DaemonToWrapper::Nack {
                    id: bridge_msg.id.clone(),
                    reason: "budget de sauts épuisé".to_string(),
                });
            }

            // 4 bis. Respecter le refus d'être dérangé du destinataire.
            //
            // Le contrôle est ici, dans le daemon, et non côté client : un
            // client d'une version antérieure le contournerait, et il devrait
            // interroger l'annuaire avant chaque envoi. L'émetteur reçoit la
            // raison et le temps restant afin de décider lui-même s'il attend,
            // insiste plus tard, ou s'adresse à quelqu'un d'autre.
            if let Some(presence) = presence_of_agent(&mut st, &bridge_msg.to) {
                if presence.is_dnd() {
                    let minutes = presence.dnd_minutes_left();
                    let target = presence.name.clone();
                    info!(
                        "refus DND: « {} » ne veut pas être dérangé ({} min)",
                        target, minutes
                    );
                    return Some(DaemonToWrapper::Nack {
                        id: bridge_msg.id.clone(),
                        reason: format!(
                            "« {} » ne souhaite pas être dérangé (encore {} min)",
                            target, minutes
                        ),
                    });
                }
            }

            // 5. Router
            let action =
                st.router
                    .resolve(&bridge_msg.from, &bridge_msg.to, bridge_msg.hops, conn_id);

            match action {
                RouterAction::Deliver { target_conn } => {
                    let conv_key = format!("{}|{}", bridge_msg.from, bridge_msg.to);

                    if let Err(e) = st.store.record_message(&bridge_msg, &conv_key) {
                        error!("store: {}", e);
                    }
                    st.circuit_breaker.record(&bridge_msg.from, &bridge_msg.to);
                    st.deduplicator.mark_sent(&content_key, &bridge_msg.to);
                    st.envelope_guard
                        .mark_relayed(&bridge_msg.id, &bridge_msg.to);

                    // Push vers le destinataire
                    let dtw = DaemonToWrapper::Deliver(bridge_msg.clone());
                    let json = encode(&dtw).unwrap_or_default();
                    eprintln!("[BRIDGET] Push vers {}: {} octets", target_conn, json.len());

                    if let Some(target_writer) = st.connections.get(&target_conn) {
                        log::debug!("push vers {}: écriture sur writer", target_conn);
                        if let Ok(mut w) = target_writer.lock() {
                            eprintln!("[BRIDGET] Writer locked for {}, écriture...", target_conn);
                            match writeln!(w, "{}", json) {
                                Ok(_) => {
                                    let _ = w.flush();
                                    info!(
                                        "livré: {} → « {} » (hops={}, reply={})",
                                        bridge_msg.id,
                                        bridge_msg.to,
                                        bridge_msg.hops,
                                        bridge_msg.reply
                                    );
                                }
                                Err(e) => error!("push {}: {}", target_conn, e),
                            }
                        }
                    } else {
                        warn!("cible {} disparue", target_conn);
                    }

                    // Si reply=yes, enregistrer dans pending_replies pour escalade
                    if bridge_msg.reply {
                        let timeout = bridge_msg.reply_timeout.unwrap_or(60);
                        if let Err(error) = st.store.create_request(
                            &bridge_msg.id,
                            &bridge_msg.from,
                            &bridge_msg.to,
                            timeout,
                        ) {
                            return Some(DaemonToWrapper::Nack {
                                id: bridge_msg.id.clone(),
                                reason: format!("impossible de suivre la demande: {}", error),
                            });
                        }
                        st.pending_replies.push(PendingReply {
                            msg_id: bridge_msg.id.clone(),
                            from: bridge_msg.from.clone(),
                            // Si l'envoi a été déclenché par `bridget send` dans
                            // un agent, conn_id est un client CLI éphémère. Les
                            // relances et le timeout doivent viser le wrapper
                            // durable identifié ci-dessus.
                            from_conn: reply_sender_conn
                                .expect("un --reply a toujours un expéditeur connecté"),
                            to: bridge_msg.to.clone(),
                            target_conn: target_conn.clone(),
                            timeout_secs: timeout,
                            created_at: std::time::Instant::now(),
                            escalation_level: 0,
                        });
                        info!(
                            "reply attendu: {} → {} (timeout={}s, escalade à {}/{}s)",
                            bridge_msg.id,
                            bridge_msg.to,
                            timeout,
                            timeout / 3,
                            timeout * 2 / 3
                        );
                    }

                    Some(DaemonToWrapper::Ack {
                        id: bridge_msg.id.clone(),
                    })
                }
                RouterAction::Reject(err) => {
                    warn!("rejet: {}", err);
                    Some(DaemonToWrapper::Nack {
                        id: bridge_msg.id.clone(),
                        reason: err.to_string(),
                    })
                }
            }
        }

        WrapperToDaemon::Heartbeat => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(instance_id) = st.conn_instances.get(conn_id).cloned() {
                if let Some(presence) = st.presences.get_mut(&instance_id) {
                    presence.last_seen = Instant::now();
                }
            }
            None
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
                    if let Some(agent) = st.router.get_agent(&request.target) {
                        if let Some(writer) = st.connections.get(&agent.connection_id) {
                            if let Err(e) = deliver_to_agent(
                                writer,
                                &request.target,
                                &format!(
                                    "Demande #{} annulée par {}. Aucune réponse n'est requise.{}",
                                    request.id,
                                    sender,
                                    request
                                        .cancel_reason
                                        .as_deref()
                                        .map(|r| format!(" Motif : {}", r))
                                        .unwrap_or_default()
                                ),
                            ) {
                                error!("Impossible de délivrer l'annulation à {}: {}", request.target, e);
                            }
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
                Ok(requests) => Some(DaemonToWrapper::RequestList {
                    requests: requests
                        .into_iter()
                        .map(|request| bridget_transport::protocol::RequestInfo {
                            id: request.id,
                            target: request.target,
                            state: request.state,
                            deadline_at: request.deadline_at,
                            cancel_reason: request.cancel_reason,
                        })
                        .collect(),
                }),
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

impl Default for DaemonStatus {
    fn default() -> Self {
        DaemonStatus {
            running: false,
            agents: vec![],
            message_count: 0,
        }
    }
}

pub struct DaemonStatus {
    pub running: bool,
    pub agents: Vec<bridget_transport::protocol::AgentInfo>,
    pub message_count: usize,
}

#[cfg(test)]
mod presence_tests {
    use super::*;

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
            .register(Some("agent-distant-1"), &bridget_core::AgentType::Codex, "conn-1")
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
        assert_eq!(agents[0].host, "projet-a");
        assert_eq!(agents[0].os, "Linux");
        assert_eq!(agents[0].state, "unreachable");
        // FR-010 : la dernière capacité connue survit à la perte de connexion.
        assert_eq!(agents[0].model.as_deref(), Some("gpt-5.3-codex"));
        assert_eq!(agents[0].effort.as_deref(), Some("xhigh"));
        if let Err(e) = std::fs::remove_file(&config.db_path) {
            log::warn!("Impossible de supprimer la base {}: {}", config.db_path.display(), e);
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
                transport: "unix".to_string(),
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
        assert_eq!(agents[0].model.as_deref(), Some("claude-haiku-4-5-20251001"));
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
            &mut state,
        );
        assert!(matches!(response, DaemonToWrapper::Registered { .. }));

        let agents = state.agent_infos();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].state, "connected");
        assert_eq!(agents[0].model.as_deref(), Some("gpt-5.3-codex"));
        assert_eq!(agents[0].effort.as_deref(), Some("xhigh"));

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
            log::warn!("Impossible de supprimer la base {}: {}", config.db_path.display(), e);
        }
    }
}
