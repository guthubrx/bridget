//! Daemon bridget — écoute sur socket locale Unix, route les messages
//! entre les wrappers connectés, persiste l'état en SQLite.

use bridget_core::{CircuitBreaker, Deduplicator, EnvelopeGuard, Router, RouterAction};
use bridget_transport::greffe_authorization::{
    GreffeAuthorizationGate, GreffeDepositAuthorization, GreffeMutationAction,
};
use bridget_transport::protocol::{
    AttachRefusal, CLIENT_CONTRACT_VERSION, COORDINATION_EVENTS_VERSION,
    COORDINATION_STREAM_VERSION, ClientCapability, ClientRefusal, ConnectionRole, IdempotencyIssue,
    PresenceMode, SERVICE_CONTRACT_VERSION, ServiceCapability, ServiceRefusal, SpawnRefusal,
    StopOutcome, decode, encode,
};
use bridget_transport::{ChannelReport, DaemonToWrapper, ResolvedAgentDefinition, WrapperToDaemon};
use log::{error, info, warn};
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::idempotency::{
    IdempotencyKey, IdempotencyStore, LookupResult, OperationKind, ReplyTracking, Reservation,
    SendDelivery,
};
use crate::managed_supervisor::ManagedSupervisorGuard;
use crate::store::{
    GuichetCoordinationEvent, GuichetDeposit, GuichetLifecycleEvent, GuichetNext,
    GuichetReplyInput, GuichetResult, MAX_GUICHET_FRAME_BYTES, Store, StoreError,
};
use crate::{
    desired_state::DesiredStateStore,
    fleet::{FleetConfig, FleetSupervisor, SpawnLease, SpawnOrder as FleetSpawnOrder},
    lifecycle::{
        PreparedSpawn, SourceEnvironment, SpawnDecision, prepare_recovery, source_environment,
        submit_spawn, submit_spawn_from_resolved,
    },
    managed_process::{
        ManagedIdentity, ManagedLaunch, ManagedMarkerStore, ManagedStatus, ManagedStderrStore,
        ManagedStopResult, RunningManagedChild, spawn_managed_bootstrap_with_stderr,
    },
    recovery_trace::{
        REASON_ABSENT_FROM_FLEET, REASON_FROZEN_DEFINITION, REASON_NON_PERSISTENT, REASON_QUOTA,
        REASON_RECOVERY_FAILED, RecoveryLossEntry,
    },
    registry::{AgentRegistry, DEFAULT_NOTIFY_TIMEOUT_SECS},
};
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

/// Pas de scrutation des threads de fond pendant leur sieste.
///
/// Une sieste longue n'est pas seulement un délai de réaction : tant qu'un
/// thread de fond dort, il retient son clone de `Arc<Mutex<DaemonState>>`, donc
/// l'état, donc le `Sender` du superviseur — et l'arrêt du daemon reste
/// suspendu à son réveil.
const SHUTDOWN_POLL: Duration = Duration::from_millis(100);

/// Dort au plus `total`, en se réveillant pour voir si l'arrêt est demandé.
///
/// Rend `true` quand l'arrêt est demandé : l'appelant doit alors SORTIR de sa
/// boucle et laisser tomber sa référence à l'état.
fn sleep_until_shutdown(total: Duration) -> bool {
    sleep_until_flag(total, &SHUTDOWN_REQUESTED)
}

/// Le drapeau est un paramètre : l'oracle éprouve la boucle sans toucher au
/// statique global, que d'autres tests du même binaire partageraient.
fn sleep_until_flag(total: Duration, flag: &AtomicBool) -> bool {
    let deadline = Instant::now() + total;
    loop {
        if flag.load(Ordering::SeqCst) {
            return true;
        }
        let reste = deadline.saturating_duration_since(Instant::now());
        if reste.is_zero() {
            return false;
        }
        thread::sleep(reste.min(SHUTDOWN_POLL));
    }
}

#[cfg(test)]
mod arret_tests {
    use super::{SHUTDOWN_POLL, SHUTDOWN_REQUESTED, sleep_until_flag, sleep_until_shutdown};
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, Instant};

    /// Sérialise les tests qui manipulent le drapeau GLOBAL d'arrêt.
    static VERROU_DRAPEAU: Mutex<()> = Mutex::new(());

    /// Un thread de fond en sieste longue doit LÂCHER sa référence à l'état dès
    /// l'ordre d'arrêt. Sans cela, l'état — et le `Sender` qu'il contient —
    /// survit jusqu'au réveil, et l'arrêt du daemon attend d'autant.
    ///
    /// Mutant qui tue ce test : revenir à `thread::sleep(total)` sans scrutation
    /// → l'attente dure la sieste entière (3 s ici) et l'assertion sur le délai
    /// mesuré meurt en affichant la valeur.
    #[test]
    fn une_sieste_longue_est_interrompue_par_l_ordre_d_arret() {
        let flag = Arc::new(AtomicBool::new(false));
        let observe = Arc::clone(&flag);
        let sieste = Duration::from_secs(3);
        let debut = Instant::now();
        let dormeur = std::thread::spawn(move || sleep_until_flag(sieste, &observe));
        std::thread::sleep(SHUTDOWN_POLL * 2);
        flag.store(true, Ordering::SeqCst);
        let interrompu = dormeur.join().expect("le dormeur ne panique pas");
        let ecoule = debut.elapsed();

        assert!(
            interrompu,
            "la sieste doit rendre true quand l'arrêt est demandé"
        );
        assert!(
            ecoule < sieste / 2,
            "l'arrêt doit interrompre la sieste : {} ms écoulées pour une sieste de {} ms",
            ecoule.as_millis(),
            sieste.as_millis()
        );
    }

    /// Contrôle positif : sans ordre d'arrêt, la sieste va bien à son terme et
    /// rend `false`. Sans lui, une fonction qui rendrait TOUJOURS `true`
    /// passerait le test ci-dessus.
    #[test]
    fn sans_ordre_d_arret_la_sieste_va_a_son_terme() {
        let flag = AtomicBool::new(false);
        let sieste = SHUTDOWN_POLL * 3;
        let debut = Instant::now();
        let interrompu = sleep_until_flag(sieste, &flag);
        let ecoule = debut.elapsed();
        assert!(!interrompu, "aucun ordre d'arrêt : la sieste rend false");
        assert!(
            ecoule >= sieste,
            "la sieste ne doit pas être écourtée : {} ms pour {} ms demandées",
            ecoule.as_millis(),
            sieste.as_millis()
        );
    }

    /// LE CHEMIN REEL, celui que les fils de fond empruntent.
    ///
    /// Les deux oracles ci-dessus éprouvent `sleep_until_flag`, le helper
    /// GÉNÉRIQUE. Or les fils rappels et purge appellent `sleep_until_shutdown`,
    /// l'ADAPTATEUR qui le lie au drapeau global. Remplacer le seul adaptateur
    /// par `thread::sleep(total)` les rendait insensibles à l'arrêt sans faire
    /// rougir quoi que ce soit : les unités gardaient une couche VOISINE du
    /// chemin de production, et le raccord n'était gardé par rien.
    ///
    /// Mutant qui tue ce test : `fn sleep_until_shutdown(total) { thread::sleep(total); false }`
    /// → l'appel rend `false` après la sieste entière, les deux assertions
    /// meurent en affichant le délai mesuré.
    #[test]
    fn l_adaptateur_des_fils_de_fond_observe_le_drapeau_d_arret() {
        // Le drapeau est global : un seul test à la fois le manipule. Aucun
        // appel à `daemon::run` n'existe dans ce binaire de test, donc personne
        // d'autre ne l'observe ni ne le remet à zéro sous nos pieds.
        let _ordre = VERROU_DRAPEAU
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        SHUTDOWN_REQUESTED.store(false, Ordering::SeqCst);

        let sieste = Duration::from_secs(3);
        let debut = Instant::now();
        let dormeur = std::thread::spawn(move || sleep_until_shutdown(sieste));
        std::thread::sleep(SHUTDOWN_POLL * 2);
        SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
        let interrompu = dormeur.join().expect("le dormeur ne panique pas");
        let ecoule = debut.elapsed();

        // Rendre le drapeau à son état de repos AVANT d'assertir : un échec ne
        // doit pas laisser le binaire de test avec un arrêt demandé.
        SHUTDOWN_REQUESTED.store(false, Ordering::SeqCst);

        assert!(
            interrompu,
            "l'adaptateur doit rendre true quand l'arrêt est demandé —              il a rendu false après {} ms",
            ecoule.as_millis()
        );
        assert!(
            ecoule < sieste / 2,
            "ARRÊT PRÉCOCE attendu : {} ms écoulées pour une sieste de {} ms",
            ecoule.as_millis(),
            sieste.as_millis()
        );
    }

    /// Contrôle positif de l'adaptateur : sans ordre d'arrêt, il dort jusqu'au
    /// bout et rend `false`. Sans lui, un adaptateur qui rendrait TOUJOURS
    /// `true` passerait le test ci-dessus.
    #[test]
    fn l_adaptateur_sans_ordre_d_arret_va_au_bout_de_sa_sieste() {
        let _ordre = VERROU_DRAPEAU
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        SHUTDOWN_REQUESTED.store(false, Ordering::SeqCst);

        let sieste = SHUTDOWN_POLL * 3;
        let debut = Instant::now();
        let interrompu = sleep_until_shutdown(sieste);
        let ecoule = debut.elapsed();

        assert!(!interrompu, "aucun ordre d'arrêt : l'adaptateur rend false");
        assert!(
            ecoule >= sieste,
            "la sieste ne doit pas être écourtée : {} ms pour {} ms demandées",
            ecoule.as_millis(),
            sieste.as_millis()
        );
    }
}

/// Gestionnaires de signaux du **service** daemon uniquement.
///
/// Sous `cfg(test)`, no-op volontaire : un binaire de test qui hériterait de
/// ces handlers (SIGTERM → drapeau sans `_exit`, SIGINT/SIGHUP ignorés)
/// devient ininterruptible. Les suites interrompues laissent alors des
/// orphelins adoptés par launchd, sourds à tout signal propre.
#[cfg(test)]
fn install_daemon_signal_handlers() {}

#[cfg(not(test))]
fn install_daemon_signal_handlers() {
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
}

/// Retain : horloge **lien** seule (`link_seen`). Aucun état — y compris
/// `connected` — n'est immortel (lot B, composé avec A). La capacité
/// (`capacity_seen`) reste exposée par `who` (`last_seen_secs`) mais ne
/// décide plus du retain.
fn presence_within_retention(presence: &Presence) -> bool {
    presence.link_seen.elapsed() <= PRESENCE_RETENTION
}

/// Mutant lot B : l'ancienne exemption `connected`. Hors chemin de production
/// pour prouver que la restaurer cacherait le défaut (relec1 / relec5).
#[cfg(test)]
fn presence_within_retention_mutant_exempt_connected(presence: &Presence) -> bool {
    presence.state == "connected" || presence.link_seen.elapsed() <= PRESENCE_RETENTION
}
const ATTACH_VIEW_BUFFER_BYTES: usize = 1024 * 1024;
const ATTACH_VIEW_WRITE_TIMEOUT: Duration = Duration::from_secs(1);
/// Plafond global (toutes connexions attach confondues) des accusés tardifs.
/// Chaque entrée expire aussi après `PENDING_ATTACH_SEND_TTL`.
const MAX_PENDING_ATTACH_SENDS: usize = 1024;
const PENDING_ATTACH_SEND_TTL: Duration = Duration::from_secs(300);
const MANAGED_STOP_COOPERATIVE_GRACE: Duration = Duration::from_millis(500);
const MANAGED_STOP_FORCED_GRACE: Duration = Duration::from_secs(1);
const MANAGED_STOP_POLL: Duration = Duration::from_millis(20);
const MANAGED_STOP_REPLY_TIMEOUT: Duration = Duration::from_secs(3);
const MANAGED_RECOVERY_DEADLINE_SECS: i64 = 30;
const CLIENT_IDEMPOTENCY_HORIZON_SECS: i64 = 7 * 24 * 60 * 60;
const CLIENT_ISSUED_AT_TOLERANCE_SECS: i64 = 60;
const MAX_ACTIVE_ISSUER_SCOPES: usize = 4096;

fn unix_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

// Constante pour la période de grâce des timeouts (M-004)
const TIMEOUT_GRACE_PERIOD: u64 = 30; // secondes

/// Vrai si un tour `busy` a dépassé notify_timeout + grâce et doit redevenir
/// mandatable. Sans `busy_since`, on retombe sur `capacity_seen` (Register
/// historique / upgrade à chaud).
fn busy_turn_is_stale(presence: &Presence, ttl_secs: u64) -> bool {
    if presence.state != "busy" {
        return false;
    }
    let elapsed = presence
        .busy_since
        .unwrap_or(presence.capacity_seen)
        .elapsed();
    elapsed >= Duration::from_secs(ttl_secs)
}

#[derive(Clone)]
struct Presence {
    name: String,
    agent_type: String,
    host: String,
    /// Protocole d'agent réellement parlé.
    transport: String,
    /// Canal de connexion au daemon, absent s'il n'a pas été attesté.
    channel: Option<String>,
    /// Mode de présence attesté à l'enregistrement. Les présences historiques
    /// restent `None` : ne jamais le déduire du transport ou du type d'agent.
    mode: Option<PresenceMode>,
    /// Localisation tmux attestée au format `session:window.pane`.
    location: Option<String>,
    /// Journal append-only attesté par le pilote vivant. Il est réinitialisé
    /// à chaque Register : un mode de présence ne prouve jamais un journal.
    journal_available: bool,
    os: String,
    state: String,
    /// Instant où le tour `busy` a commencé (`TurnState true` / Register
    /// `turn_in_progress`). Absent hors busy. Sert à libérer un tour qui
    /// n'aboutit jamais : sans cela Maicie refuse tout mandat (connected|dnd).
    busy_since: Option<Instant>,
    /// Dernière attestation de CAPACITÉ (register, tour, runtime…) — pas le
    /// heartbeat. C'est ce que `last_seen_secs` expose à who / bridget-idle.
    capacity_seen: Instant,
    /// Dernière attestation de LIEN (socket / heartbeat). Le retain s'appuie
    /// dessus pour ne pas jeter un long tour `busy` vivant.
    link_seen: Instant,
    reconnect_count: u32,
    /// Modèle courant, `None` tant qu'aucune observation n'a eu lieu.
    model: Option<String>,
    /// Niveau d'effort courant, `None` si jamais observé ou observé absent.
    effort: Option<String>,
    /// Limites fournisseur attestées, indexées par fenêtre. Une observation
    /// n'écrase que sa propre clé : `five_hour` ne touche pas `seven_day`.
    rate_limits: std::collections::BTreeMap<String, bridget_transport::protocol::RateLimitFact>,
    /// Modèle annoncé par le flux natif. Distinct du modèle épinglé : un flux
    /// muet laisse ce champ vide et n'invente aucun écart.
    served_model: Option<String>,
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

    /// Capacité d'exécution observée : met à jour les deux horloges.
    /// Aussi appelée à l'observation d'une *incapacité* (stopped /
    /// unreachable) : ce n'est pas une attestation de vie, c'est dater
    /// l'entrée pour le retain.
    fn touch_capacity(&mut self) {
        let now = Instant::now();
        self.capacity_seen = now;
        self.link_seen = now;
    }

    /// Lien socket seul (heartbeat) : ne prouve aucune capacité.
    fn touch_link(&mut self) {
        self.link_seen = Instant::now();
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

fn desired_state_path(config: &DaemonConfig) -> PathBuf {
    crate::desired_state::path_for_daemon_db(&config.db_path)
}

/// État partagé du daemon.
struct DaemonState {
    /// Machine et base **de ce daemon**, retenues une fois au démarrage.
    ///
    /// Elles voyagent ensuite dans `ClientWelcome` : un client fédéré ne peut
    /// pas les déduire, et jusqu'ici il affichait les siennes à leur place.
    host: String,
    db_path: PathBuf,
    router: Router,
    circuit_breaker: CircuitBreaker,
    deduplicator: Deduplicator,
    envelope_guard: EnvelopeGuard,
    store: Store,
    idempotency: IdempotencyStore,
    fleet: Arc<FleetSupervisor>,
    registry: AgentRegistry,
    source_env: SourceEnvironment,
    recovering: bool,
    recovery_commands: HashSet<String>,
    managed_tx: Sender<ManagedSupervisorCommand>,
    marker_store: ManagedMarkerStore,
    managed_spawns: HashMap<String, ManagedSpawnRecord>,
    managed_by_instance: HashMap<String, String>,
    managed_terminal_instances: HashSet<String>,
    connections: HashMap<String, Arc<Mutex<BufWriter<UnixStream>>>>,
    conn_names: HashMap<String, String>,
    conn_hosts: HashMap<String, String>,
    conn_operating_systems: HashMap<String, String>,
    conn_instances: HashMap<String, String>,
    /// Connexions MCP filles : elles portent le principal canonique pour les
    /// gardes, sans devenir propriétaires de la présence du wrapper.
    auxiliary_connections: HashSet<String>,
    /// Les clients attach négocient ce rôle explicite ; l'absence d'entrée
    /// reste un wrapper pour préserver les agents 007 déjà connectés.
    connection_roles: HashMap<String, ConnectionRole>,
    /// Une négociation appartient à la connexion, tandis que le scope peut
    /// volontairement être partagé par plusieurs retries coopératifs.
    client_negotiations: HashMap<String, NegotiatedClient>,
    /// La capacité du guichet ne dépend jamais d'un nom déclaré : elle est
    /// attachée à cette négociation de service et disparaît avec la connexion.
    service_negotiations: HashMap<String, NegotiatedService>,
    /// Relevés 016 v2 en cours. Une entrée non fraîche reste volontairement
    /// muette jusqu'à une nouvelle souscription ayant atteint son snapshot.
    coordination_subscriptions: HashMap<String, CoordinationSubscription>,
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
    /// Remises idempotentes à écrire juste après la réponse `Registered`.
    /// L'ordre rend le réenregistrement observable avant toute redélivrance.
    pending_post_response_controls: HashMap<String, Vec<DeferredControl>>,
}

struct ManagedSpawnRecord {
    lease: SpawnLease,
    agent_type: String,
    requester_conns: Vec<String>,
    wrapper_conn: Option<String>,
    stop: Arc<ManagedStopControl>,
}

type ManagedRecovery = (PreparedSpawn, Arc<ManagedStopControl>);

struct ManagedStopControl {
    requested: AtomicBool,
    handshake_complete: AtomicBool,
    waiters: Mutex<Vec<Sender<StopOutcome>>>,
}

impl ManagedStopControl {
    fn new() -> Self {
        Self {
            requested: AtomicBool::new(false),
            handshake_complete: AtomicBool::new(false),
            waiters: Mutex::new(Vec::new()),
        }
    }

    fn request(&self) -> Receiver<StopOutcome> {
        let (sender, receiver) = mpsc::channel();
        self.waiters
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .push(sender);
        self.requested.store(true, Ordering::SeqCst);
        receiver
    }

    fn is_requested(&self) -> bool {
        self.requested.load(Ordering::SeqCst)
    }

    fn mark_handshake_complete(&self) {
        self.handshake_complete.store(true, Ordering::SeqCst);
    }

    fn is_actionable(&self) -> bool {
        self.is_requested() && self.handshake_complete.load(Ordering::SeqCst)
    }

    fn complete(&self, outcome: StopOutcome) {
        let waiters = std::mem::take(
            &mut *self
                .waiters
                .lock()
                .unwrap_or_else(|poison| poison.into_inner()),
        );
        for waiter in waiters {
            let _ = waiter.send(outcome.clone());
        }
    }
}

enum ManagedStopTarget {
    Supervised {
        receiver: Receiver<StopOutcome>,
        completion: Arc<ManagedStopControl>,
        wrapper: Option<Arc<Mutex<BufWriter<UnixStream>>>>,
    },
    Marker {
        store: ManagedMarkerStore,
        fallback: StopOutcome,
    },
    Immediate(StopOutcome),
}

pub(crate) enum ManagedSupervisorCommand {
    Start {
        prepared: PreparedSpawn,
        stop: Arc<ManagedStopControl>,
    },
    Registered {
        instance_id: String,
        conn_id: String,
        name: String,
    },
}

pub(crate) enum ManagedSupervisorEvent {
    Connected {
        lease: SpawnLease,
        conn_id: String,
        definition: Box<bridget_transport::ResolvedAgentDefinition>,
    },
    Failed {
        lease: SpawnLease,
        kind: String,
        reason: String,
        /// Connexion wrapper déjà inscrite (Register) — à retirer de l'annuaire
        /// quand le spawn échoue après naissance du processus.
        conn_id: Option<String>,
    },
    Exited {
        lease: SpawnLease,
        conn_id: Option<String>,
        reason: String,
    },
    Stopped {
        lease: SpawnLease,
        conn_id: Option<String>,
        outcome: StopOutcome,
        completion: Arc<ManagedStopControl>,
    },
    StopTimedOut {
        lease: SpawnLease,
        outcome: StopOutcome,
        completion: Arc<ManagedStopControl>,
    },
}

#[derive(Clone)]
struct AttachSubscription {
    agent: String,
    attach_conn: String,
    wrapper_conn: String,
    caught_up: bool,
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

#[derive(Clone)]
struct NegotiatedService {
    version: u16,
    capabilities: Vec<ServiceCapability>,
}

#[derive(Clone)]
struct CoordinationSubscription {
    fresh: bool,
    next_cursor: u64,
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
) -> Result<String, DeliveryError> {
    let msg = bridget_core::BridgetMessage::new("bridget", target_name, body);
    let message_id = msg.id.clone();
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
    Ok(message_id)
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

#[derive(Debug)]
struct AttachSubscriptionRefusal {
    reason: AttachRefusal,
    mode: Option<PresenceMode>,
    location: Option<String>,
}

impl AttachSubscriptionRefusal {
    fn without_presence(reason: AttachRefusal) -> Self {
        Self {
            reason,
            mode: None,
            location: None,
        }
    }
}

fn attach_refusal_for_subscription(
    state: &DaemonState,
    agent: &str,
) -> Result<String, AttachSubscriptionRefusal> {
    let registered = match state.router.get_agent(agent) {
        Some(registered) => registered,
        None if state
            .presences
            .values()
            .any(|presence| presence.name == agent && presence.state == "stopped") =>
        {
            return Err(AttachSubscriptionRefusal::without_presence(
                AttachRefusal::AgentStopped,
            ));
        }
        None => {
            return Err(AttachSubscriptionRefusal::without_presence(
                AttachRefusal::AgentUnknown,
            ));
        }
    };
    let presence = state
        .conn_instances
        .get(&registered.connection_id)
        .and_then(|instance_id| state.presences.get(instance_id));
    if !presence.is_some_and(|presence| presence.journal_available) {
        return Err(AttachSubscriptionRefusal {
            reason: AttachRefusal::JournalUnavailable,
            mode: presence.and_then(|presence| presence.mode),
            location: presence.and_then(|presence| presence.location.clone()),
        });
    }
    if !state.connections.contains_key(&registered.connection_id) {
        return Err(AttachSubscriptionRefusal::without_presence(
            AttachRefusal::WrapperUnavailable,
        ));
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

struct SupervisedProcess {
    prepared: PreparedSpawn,
    child: RunningManagedChild,
    registered: Option<(String, String)>,
    connected: bool,
    failure_sent: bool,
    stop: Arc<ManagedStopControl>,
    stop_attempted: bool,
}

pub(crate) fn spawn_managed_supervisor_thread(
    fleet: Arc<FleetSupervisor>,
    config: &DaemonConfig,
    commands: Receiver<ManagedSupervisorCommand>,
    events: Sender<ManagedSupervisorEvent>,
    executable_override: Option<PathBuf>,
) -> thread::JoinHandle<()> {
    let marker_store = ManagedMarkerStore::at_directory(
        config
            .db_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("/tmp"))
            .join("managed"),
    );
    let stderr_store = ManagedStderrStore::at_directory(
        config
            .db_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("/tmp"))
            .join("managed-stderr"),
    );
    let retention_days = config.retention_days;
    thread::spawn(move || {
        if let Err(error) = stderr_store.purge_older_than_days(retention_days) {
            warn!("purge stderr des équipiers impossible: {error}");
        }
        let mut last_stderr_purge = Instant::now();
        let mut active = HashMap::<String, SupervisedProcess>::new();
        loop {
            match commands.recv_timeout(Duration::from_millis(50)) {
                Ok(command) => handle_managed_command(
                    command,
                    &fleet,
                    &marker_store,
                    &stderr_store,
                    &events,
                    &mut active,
                    executable_override.as_ref(),
                ),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => return,
            }
            while let Ok(command) = commands.try_recv() {
                handle_managed_command(
                    command,
                    &fleet,
                    &marker_store,
                    &stderr_store,
                    &events,
                    &mut active,
                    executable_override.as_ref(),
                );
            }
            poll_managed_processes(&fleet, &events, &mut active);
            if last_stderr_purge.elapsed() >= Duration::from_secs(60 * 60) {
                if let Err(error) = stderr_store.purge_older_than_days(retention_days) {
                    warn!("purge stderr des équipiers impossible: {error}");
                }
                last_stderr_purge = Instant::now();
            }
        }
    })
}

fn handle_managed_command(
    command: ManagedSupervisorCommand,
    fleet: &FleetSupervisor,
    marker_store: &ManagedMarkerStore,
    stderr_store: &ManagedStderrStore,
    events: &Sender<ManagedSupervisorEvent>,
    active: &mut HashMap<String, SupervisedProcess>,
    executable_override: Option<&PathBuf>,
) {
    match command {
        ManagedSupervisorCommand::Start { prepared, stop } => {
            let identity = ManagedIdentity {
                instance_id: prepared.lease.instance_id.clone(),
                command_id: prepared.lease.command_id.clone(),
                generation: prepared.lease.generation,
            };
            let start = (|| {
                let (stderr, _) = stderr_store.open(&prepared.lease.name, &identity)?;
                let executable = executable_override
                    .cloned()
                    .map(Ok)
                    .unwrap_or_else(std::env::current_exe)?;
                let launch = ManagedLaunch {
                    bootstrap_executable: executable.clone(),
                    identity,
                    wrapper_executable: executable,
                    wrapper_args: vec![
                        "managed-wrapper".to_string(),
                        prepared.agent_type.clone(),
                        prepared.lease.name.clone(),
                        serde_json::to_string(&prepared.resolved_definition)
                            .map_err(std::io::Error::other)?,
                    ],
                    cwd: prepared.cwd.clone(),
                    env: prepared.env.clone(),
                };
                let child = spawn_managed_bootstrap_with_stderr(&launch, Stdio::from(stderr))?;
                let Some(ready) = child.wait_ready_or_cancel(
                    || stop.is_actionable() || unix_timestamp() >= prepared.lease.deadline_at,
                    MANAGED_STOP_POLL,
                )?
                else {
                    return Ok(None);
                };
                if ready.ready().instance_id != prepared.lease.instance_id
                    || ready.ready().command_id != prepared.lease.command_id
                    || ready.ready().generation != prepared.lease.generation
                {
                    return Err(crate::managed_process::ManagedProcessError::InvalidStatus(
                        "BootstrapReady non corrélé".to_string(),
                    ));
                }
                let mut child = ready
                    .persist_marker(marker_store, &prepared.lease.name)?
                    .release()?;
                child.set_status_nonblocking()?;
                Ok(Some(child))
            })();
            match start {
                Ok(Some(child)) => {
                    active.insert(
                        prepared.lease.instance_id.clone(),
                        SupervisedProcess {
                            prepared,
                            child,
                            registered: None,
                            connected: false,
                            failure_sent: false,
                            stop,
                            stop_attempted: false,
                        },
                    );
                }
                Ok(None) if stop.is_actionable() => {
                    let _ = events.send(ManagedSupervisorEvent::Stopped {
                        lease: prepared.lease,
                        conn_id: None,
                        outcome: StopOutcome::Stopped,
                        completion: stop,
                    });
                }
                Ok(None) => {
                    let reason = "délai absolu dépassé avant BootstrapReady".to_string();
                    let _ = fleet.fail(&prepared.lease, "spawn_timeout", &reason);
                    let _ = events.send(ManagedSupervisorEvent::Failed {
                        lease: prepared.lease,
                        kind: "spawn_timeout".to_string(),
                        reason,
                        conn_id: None,
                    });
                }
                Err(error) => {
                    if stop.is_actionable() {
                        let _ = events.send(ManagedSupervisorEvent::Stopped {
                            lease: prepared.lease,
                            conn_id: None,
                            outcome: StopOutcome::Stopped,
                            completion: stop,
                        });
                        return;
                    }
                    let reason = error.to_string();
                    let _ = fleet.fail(&prepared.lease, "negotiation_failed", &reason);
                    let _ = events.send(ManagedSupervisorEvent::Failed {
                        lease: prepared.lease,
                        kind: "negotiation_failed".to_string(),
                        reason,
                        conn_id: None,
                    });
                }
            }
        }
        ManagedSupervisorCommand::Registered {
            instance_id,
            conn_id,
            name,
        } => {
            if let Some(process) = active.get_mut(&instance_id)
                && process.prepared.lease.name == name
            {
                process.registered = Some((conn_id, name));
            }
        }
    }
}

fn poll_managed_processes(
    fleet: &FleetSupervisor,
    events: &Sender<ManagedSupervisorEvent>,
    active: &mut HashMap<String, SupervisedProcess>,
) {
    let mut finished = Vec::new();
    for (instance_id, process) in active.iter_mut() {
        if process.stop.is_actionable() && !process.stop_attempted {
            let outcome = match process.child.stop_group(
                MANAGED_STOP_COOPERATIVE_GRACE,
                MANAGED_STOP_FORCED_GRACE,
                MANAGED_STOP_POLL,
            ) {
                Ok(ManagedStopResult::Stopped) => StopOutcome::Stopped,
                Ok(ManagedStopResult::StoppedForced { survivors_killed }) => {
                    StopOutcome::StoppedForced { survivors_killed }
                }
                Ok(ManagedStopResult::Timeout) => StopOutcome::Timeout {
                    state: "groupe encore vivant".to_string(),
                },
                Err(error) => StopOutcome::Timeout {
                    state: error.to_string(),
                },
            };
            if matches!(outcome, StopOutcome::Timeout { .. }) {
                process.stop_attempted = true;
                let _ = events.send(ManagedSupervisorEvent::StopTimedOut {
                    lease: process.prepared.lease.clone(),
                    outcome,
                    completion: Arc::clone(&process.stop),
                });
            } else {
                let _ = events.send(ManagedSupervisorEvent::Stopped {
                    lease: process.prepared.lease.clone(),
                    conn_id: process.registered.as_ref().map(|value| value.0.clone()),
                    outcome,
                    completion: Arc::clone(&process.stop),
                });
                finished.push(instance_id.clone());
                continue;
            }
        }

        // Délai absolu avant Connected : le client a déjà (ou va) recevoir un
        // refus — le processus enfant ne doit PAS survivre (fantôme hors quota).
        if !process.connected
            && !process.failure_sent
            && unix_timestamp() >= process.prepared.lease.deadline_at
        {
            let reason = "délai absolu dépassé avant Connected".to_string();
            let _ = fleet.fail(&process.prepared.lease, "spawn_timeout", &reason);
            if reap_failed_managed_child(process, events, "spawn_timeout", reason) {
                finished.push(instance_id.clone());
            }
            continue;
        }

        let status = process.child.try_status();
        match status {
            Ok(Some(ManagedStatus::StartupFailed {
                kind,
                reason,
                instance_id: reported_instance,
                command_id,
                generation,
            })) => {
                let correlated = reported_instance == *instance_id
                    && command_id == process.prepared.lease.command_id
                    && generation == process.prepared.lease.generation;
                let (kind, reason) = if correlated {
                    (kind, reason)
                } else {
                    (
                        "negotiation_failed".to_string(),
                        "StartupFailed non corrélé".to_string(),
                    )
                };
                if !process.failure_sent {
                    let _ = fleet.fail(&process.prepared.lease, &kind, &reason);
                    if reap_failed_managed_child(process, events, &kind, reason) {
                        finished.push(instance_id.clone());
                    }
                    continue;
                }
            }
            Ok(Some(ManagedStatus::BootstrapReady(_))) => {
                if !process.failure_sent {
                    let reason = "second BootstrapReady interdit".to_string();
                    let _ = fleet.fail(&process.prepared.lease, "negotiation_failed", &reason);
                    if reap_failed_managed_child(process, events, "negotiation_failed", reason) {
                        finished.push(instance_id.clone());
                    }
                    continue;
                }
            }
            Err(error) if !process.failure_sent => {
                let reason = error.to_string();
                let _ = fleet.fail(&process.prepared.lease, "negotiation_failed", &reason);
                if reap_failed_managed_child(process, events, "negotiation_failed", reason) {
                    finished.push(instance_id.clone());
                }
                continue;
            }
            Ok(None) | Err(_) => {}
        }

        let exited = match process.child.try_wait() {
            Ok(Some(status)) => {
                let reason = format!("équipier terminé avec {status}");
                if process.connected {
                    let _ = events.send(ManagedSupervisorEvent::Exited {
                        lease: process.prepared.lease.clone(),
                        conn_id: process.registered.as_ref().map(|value| value.0.clone()),
                        reason,
                    });
                } else if !process.failure_sent {
                    let _ = fleet.fail(&process.prepared.lease, "negotiation_failed", &reason);
                    let _ = events.send(ManagedSupervisorEvent::Failed {
                        lease: process.prepared.lease.clone(),
                        kind: "negotiation_failed".to_string(),
                        reason,
                        conn_id: process.registered.as_ref().map(|value| value.0.clone()),
                    });
                    process.failure_sent = true;
                }
                let _ = process.child.remove_marker();
                finished.push(instance_id.clone());
                true
            }
            Ok(None) => false,
            Err(error) => {
                warn!("waitpid non bloquant impossible pour {instance_id}: {error}");
                false
            }
        };
        if exited {
            continue;
        }

        if !process.connected
            && !process.failure_sent
            && !process.child.status_is_open()
            && let Some((conn_id, _)) = process.registered.clone()
        {
            match fleet.register_connected(&process.prepared.lease, instance_id, unix_timestamp()) {
                Ok(_) => {
                    process.connected = true;
                    let _ = events.send(ManagedSupervisorEvent::Connected {
                        lease: process.prepared.lease.clone(),
                        conn_id,
                        definition: Box::new((*process.prepared.resolved_definition).clone()),
                    });
                }
                Err(error) => {
                    // DeadlineElapsed a déjà clos la lease (expire_locked) :
                    // ne pas rappeler fail — seulement tuer l'enfant.
                    let (kind, reason, fleet_already_closed) = match &error {
                        crate::fleet::FleetError::DeadlineElapsed => {
                            ("spawn_timeout", error.to_string(), true)
                        }
                        _ => ("negotiation_failed", error.to_string(), false),
                    };
                    if !fleet_already_closed {
                        let _ = fleet.fail(&process.prepared.lease, kind, &reason);
                    }
                    if reap_failed_managed_child(process, events, kind, reason) {
                        finished.push(instance_id.clone());
                    }
                }
            }
        }
    }
    for instance_id in finished {
        active.remove(&instance_id);
    }
}

/// Émet l'échec au client et termine le groupe (SIGTERM, jamais SIGKILL).
/// Retourne true si le groupe a disparu et peut quitter `active`.
fn reap_failed_managed_child(
    process: &mut SupervisedProcess,
    events: &Sender<ManagedSupervisorEvent>,
    kind: &str,
    reason: String,
) -> bool {
    let conn_id = process.registered.as_ref().map(|value| value.0.clone());
    let _ = events.send(ManagedSupervisorEvent::Failed {
        lease: process.prepared.lease.clone(),
        kind: kind.to_string(),
        reason,
        conn_id,
    });
    process.failure_sent = true;
    match process.child.stop_group(
        MANAGED_STOP_COOPERATIVE_GRACE,
        MANAGED_STOP_FORCED_GRACE,
        MANAGED_STOP_POLL,
    ) {
        Ok(ManagedStopResult::Stopped | ManagedStopResult::StoppedForced { .. }) => {
            let _ = process.child.remove_marker();
            true
        }
        Ok(ManagedStopResult::Timeout) => {
            warn!(
                "groupe encore vivant après SIGTERM suite à échec de spawn {} — pas de SIGKILL",
                process.prepared.lease.name
            );
            false
        }
        Err(error) => {
            warn!(
                "arrêt du groupe après échec de spawn {} impossible: {error}",
                process.prepared.lease.name
            );
            false
        }
    }
}

fn drain_managed_events(
    state: &Arc<Mutex<DaemonState>>,
    events: &Receiver<ManagedSupervisorEvent>,
) {
    while let Ok(event) = events.try_recv() {
        let mut controls = Vec::new();
        let mut views = Vec::new();
        let mut stop_completion = None;
        {
            let mut st = state.lock().unwrap_or_else(|poison| poison.into_inner());
            match event {
                ManagedSupervisorEvent::Connected {
                    lease,
                    conn_id,
                    definition,
                } => {
                    if let Some(record) = st.managed_spawns.get_mut(&lease.command_id) {
                        record.wrapper_conn = Some(conn_id);
                        let requesters = std::mem::take(&mut record.requester_conns);
                        for requester in requesters {
                            defer_control(
                                &st,
                                &requester,
                                DaemonToWrapper::SpawnAccepted {
                                    command_id: lease.command_id.clone(),
                                    name: lease.name.clone(),
                                    definition: Some((*definition).clone()),
                                },
                                &mut controls,
                            );
                        }
                    }
                    finish_recovery_command(&mut st, &lease.command_id);
                }
                ManagedSupervisorEvent::Failed {
                    lease,
                    kind,
                    reason,
                    conn_id,
                } => {
                    let refusal = match kind.as_str() {
                        "command_missing" => SpawnRefusal::CommandMissing {
                            command: reason.clone(),
                            registry: "canal managed-status".to_string(),
                        },
                        "spawn_timeout" => SpawnRefusal::SpawnTimeout,
                        _ => SpawnRefusal::NegotiationFailed {
                            detail: reason.clone(),
                        },
                    };
                    if let Some(record) = st.managed_spawns.remove(&lease.command_id) {
                        if record.stop.is_requested() {
                            stop_completion =
                                Some((Arc::clone(&record.stop), StopOutcome::Stopped));
                        }
                        st.managed_by_instance.remove(&lease.instance_id);
                        st.managed_terminal_instances
                            .insert(lease.instance_id.clone());
                        for requester in record.requester_conns {
                            defer_control(
                                &st,
                                &requester,
                                DaemonToWrapper::SpawnRejected {
                                    command_id: lease.command_id.clone(),
                                    reason: refusal.clone(),
                                },
                                &mut controls,
                            );
                        }
                        let wrapper_conn = conn_id.or(record.wrapper_conn);
                        if let Some(conn_id) = wrapper_conn {
                            let (attach_controls, attach_views) =
                                close_attach_subscriptions(&mut st, &conn_id);
                            controls.extend(attach_controls);
                            views.extend(attach_views);
                            st.router.unregister_by_conn(&conn_id);
                            st.mark_stopped(&conn_id);
                        }
                        if let Some(presence) = st.presences.get_mut(&lease.instance_id) {
                            presence.state = "stopped".to_string();
                            presence.touch_capacity();
                        }
                    }
                    finish_recovery_command(&mut st, &lease.command_id);
                }
                ManagedSupervisorEvent::Exited {
                    lease,
                    conn_id,
                    reason,
                } => {
                    let record = st.managed_spawns.remove(&lease.command_id);
                    st.managed_by_instance.remove(&lease.instance_id);
                    st.managed_terminal_instances
                        .insert(lease.instance_id.clone());
                    if let Some(record) = record.as_ref()
                        && record.stop.is_requested()
                    {
                        stop_completion = Some((Arc::clone(&record.stop), StopOutcome::Stopped));
                    }
                    let wrapper_conn = conn_id
                        .or_else(|| record.as_ref().and_then(|value| value.wrapper_conn.clone()));
                    if let Some(wrapper_conn) = wrapper_conn {
                        let (attach_controls, attach_views) =
                            close_attach_subscriptions(&mut st, &wrapper_conn);
                        controls.extend(attach_controls);
                        views.extend(attach_views);
                        let pending = std::mem::take(&mut st.pending_replies);
                        for reply in pending {
                            if reply.target_conn == wrapper_conn {
                                defer_control(
                                    &st,
                                    &reply.from_conn,
                                    DaemonToWrapper::DeliveryRejected {
                                        id: reply.msg_id,
                                        reason: reason.clone(),
                                    },
                                    &mut controls,
                                );
                            } else {
                                st.pending_replies.push(reply);
                            }
                        }
                        st.router.unregister_by_conn(&wrapper_conn);
                        st.mark_stopped(&wrapper_conn);
                    }
                    if let Some(presence) = st.presences.get_mut(&lease.instance_id) {
                        presence.state = "stopped".to_string();
                        presence.touch_capacity();
                    }
                    finish_recovery_command(&mut st, &lease.command_id);
                }
                ManagedSupervisorEvent::Stopped {
                    lease,
                    conn_id,
                    outcome,
                    completion,
                } => {
                    let record = st.managed_spawns.remove(&lease.command_id);
                    st.managed_by_instance.remove(&lease.instance_id);
                    st.managed_terminal_instances
                        .insert(lease.instance_id.clone());
                    let wrapper_conn = conn_id
                        .or_else(|| record.as_ref().and_then(|value| value.wrapper_conn.clone()));
                    if let Some(record) = record {
                        for requester in record.requester_conns {
                            defer_control(
                                &st,
                                &requester,
                                DaemonToWrapper::SpawnRejected {
                                    command_id: lease.command_id.clone(),
                                    reason: SpawnRefusal::NegotiationFailed {
                                        detail: "lancement annulé par stop".to_string(),
                                    },
                                },
                                &mut controls,
                            );
                        }
                    }
                    if let Some(wrapper_conn) = wrapper_conn {
                        let (attach_controls, attach_views) =
                            close_attach_subscriptions(&mut st, &wrapper_conn);
                        controls.extend(attach_controls);
                        views.extend(attach_views);
                        let pending = std::mem::take(&mut st.pending_replies);
                        for reply in pending {
                            if reply.target_conn == wrapper_conn {
                                defer_control(
                                    &st,
                                    &reply.from_conn,
                                    DaemonToWrapper::DeliveryRejected {
                                        id: reply.msg_id,
                                        reason: "équipier arrêté".to_string(),
                                    },
                                    &mut controls,
                                );
                            } else {
                                st.pending_replies.push(reply);
                            }
                        }
                        st.router.unregister_by_conn(&wrapper_conn);
                        st.mark_stopped(&wrapper_conn);
                    }
                    if let Some(presence) = st.presences.get_mut(&lease.instance_id) {
                        presence.state = "stopped".to_string();
                        presence.touch_capacity();
                    }
                    finish_recovery_command(&mut st, &lease.command_id);
                    stop_completion = Some((completion, outcome));
                }
                ManagedSupervisorEvent::StopTimedOut {
                    lease,
                    outcome,
                    completion,
                } => {
                    warn!(
                        "arrêt incomplet de l'équipier géré {} génération {} : groupe toujours supervisé",
                        lease.name, lease.generation
                    );
                    stop_completion = Some((completion, outcome));
                }
            }
        }
        let _ = execute_controls(controls);
        for view in views {
            view.close_and_join();
        }
        if let Some((completion, outcome)) = stop_completion {
            completion.complete(outcome);
        }
    }
}

fn finish_recovery_command(state: &mut DaemonState, command_id: &str) {
    state.recovery_commands.remove(command_id);
    if state.recovery_commands.is_empty() {
        state.recovering = false;
    }
}

impl DaemonState {
    fn new(
        config: &DaemonConfig,
        managed_tx: Sender<ManagedSupervisorCommand>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(config.socket_path.parent().unwrap())?;
        let mut store = Store::open(&config.db_path)?;
        store.recover_guichet_claims_after_restart()?;
        let idempotency = IdempotencyStore::open(&config.db_path)?;
        let desired = DesiredStateStore::at_path(desired_state_path(config));
        let fleet = Arc::new(FleetSupervisor::open(
            &config.db_path,
            desired,
            FleetConfig::from_env(),
        )?);
        let registry = AgentRegistry::load()?;
        let (view_closed_tx, view_closed_rx) = mpsc::channel();
        Ok(DaemonState {
            host: crate::build_info::local_host(),
            db_path: config.db_path.clone(),
            router: Router::new(),
            circuit_breaker: CircuitBreaker::new(
                config.circuit_breaker_window,
                config.circuit_breaker_limit,
            ),
            deduplicator: Deduplicator::new(config.dedup_window),
            envelope_guard: EnvelopeGuard::new(Duration::from_secs(config.quarantine_window)),
            store,
            idempotency,
            fleet,
            registry,
            source_env: source_environment(),
            recovering: false,
            recovery_commands: HashSet::new(),
            managed_tx,
            marker_store: ManagedMarkerStore::at_directory(
                config
                    .db_path
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new("/tmp"))
                    .join("managed"),
            ),
            managed_spawns: HashMap::new(),
            managed_by_instance: HashMap::new(),
            managed_terminal_instances: HashSet::new(),
            connections: HashMap::new(),
            conn_names: HashMap::new(),
            conn_hosts: HashMap::new(),
            conn_operating_systems: HashMap::new(),
            conn_instances: HashMap::new(),
            auxiliary_connections: HashSet::new(),
            connection_roles: HashMap::new(),
            client_negotiations: HashMap::new(),
            service_negotiations: HashMap::new(),
            coordination_subscriptions: HashMap::new(),
            attach_subscriptions: HashMap::new(),
            attach_views: HashMap::new(),
            view_closed_tx,
            view_closed_rx,
            pending_attach_sends: HashMap::new(),
            presences: HashMap::new(),
            conn_counter: 0,
            pending_replies: Vec::new(),
            pending_post_response_controls: HashMap::new(),
        })
    }

    fn next_conn_id(&mut self) -> String {
        self.conn_counter += 1;
        format!("conn-{}", self.conn_counter)
    }

    fn mark_unreachable(&mut self, conn_id: &str) {
        if self.auxiliary_connections.remove(conn_id) {
            self.conn_instances.remove(conn_id);
            return;
        }
        if let Some(instance_id) = self.conn_instances.remove(conn_id) {
            // Une ré-inscription concurrente peut déjà détenir la même
            // instance : la fermeture de l'ancienne connexion ne doit pas
            // écraser busy/connected que le nouveau Register vient d'attester.
            if self.conn_instances.iter().any(|(owner, owned)| {
                owned == &instance_id && !self.auxiliary_connections.contains(owner)
            }) {
                return;
            }
            if let Some(presence) = self.presences.get_mut(&instance_id) {
                if presence.state == "stopped" {
                    return;
                }
                presence.state = "unreachable".to_string();
                presence.touch_capacity();
            }
        }
    }

    fn mark_stopped(&mut self, conn_id: &str) {
        if self.auxiliary_connections.remove(conn_id) {
            self.conn_instances.remove(conn_id);
            return;
        }
        if let Some(instance_id) = self.conn_instances.remove(conn_id) {
            if self.conn_instances.iter().any(|(owner, owned)| {
                owned == &instance_id && !self.auxiliary_connections.contains(owner)
            }) {
                return;
            }
            if let Some(presence) = self.presences.get_mut(&instance_id) {
                presence.state = "stopped".to_string();
                presence.touch_capacity();
            }
        }
    }

    /// Fantôme reclaimable : le routeur pointe une instance absente (retain) ou
    /// une présence `unreachable`/`stopped`. Un Register sans `instance_id`
    /// (MCP éphémère, wrapper terminal) n'est PAS un fantôme — il n'a simplement
    /// pas de présence riche.
    fn connection_is_phantom_holder(&self, conn_id: &str) -> bool {
        match self.conn_instances.get(conn_id) {
            Some(instance_id) => match self.presences.get(instance_id) {
                None => true,
                Some(presence) => {
                    matches!(presence.state.as_str(), "unreachable" | "stopped")
                }
            },
            None => false,
        }
    }

    /// Libère un nom tenu par un fantôme. Retourne `false` si le nom est libre,
    /// live, ou tenu par une connexion sans instance (hors cas fantôme).
    fn reclaim_phantom_name(&mut self, name: &str) -> bool {
        let Some(existing) = self.router.get_agent(name) else {
            return false;
        };
        let old_conn = existing.connection_id.clone();
        if !self.connection_is_phantom_holder(&old_conn) {
            return false;
        }
        self.router.unregister_by_conn(&old_conn);
        self.conn_instances.remove(&old_conn);
        self.conn_names.remove(&old_conn);
        self.conn_hosts.remove(&old_conn);
        self.conn_operating_systems.remove(&old_conn);
        self.auxiliary_connections.remove(&old_conn);
        true
    }

    /// Quand le retain jette une présence, retire aussi le nom du routeur :
    /// sinon `agent_infos` projetait `unix`/`connected` inventés (fantôme).
    fn release_router_for_dangling_instances(&mut self) {
        let dangling: Vec<String> = self
            .conn_instances
            .iter()
            .filter(|(_, instance_id)| !self.presences.contains_key(instance_id.as_str()))
            .map(|(conn_id, _)| conn_id.clone())
            .collect();
        for conn_id in dangling {
            self.router.unregister_by_conn(&conn_id);
            self.conn_instances.remove(&conn_id);
            self.conn_names.remove(&conn_id);
            self.conn_hosts.remove(&conn_id);
            self.conn_operating_systems.remove(&conn_id);
            self.auxiliary_connections.remove(&conn_id);
        }
    }

    /// Après purge de présence(s) : orpheliner les remises `dispatching` et
    /// prévenir l'émetteur. Ne décide PAS de purger — seulement les suites.
    fn orphan_deliveries_after_presence_purge(&mut self, purged_instance_ids: &[String]) {
        const REASON: &str = "destinataire purgé — présence absente ; remise orpheline";
        for instance_id in purged_instance_ids {
            match self
                .idempotency
                .orphan_dispatching_for_instance(instance_id, REASON)
            {
                Ok(notices) => {
                    for notice in &notices {
                        info!(
                            "remise orpheline delivery={} msg={} {}→{} ({})",
                            notice.delivery_id,
                            notice.message_id,
                            notice.sender,
                            notice.target,
                            notice.reason
                        );
                    }
                }
                Err(error) => {
                    error!("orphelinage des remises de {instance_id}: {error}");
                }
            }
        }
        self.flush_pending_orphan_emitter_notices();
    }

    /// Pousse les signaux ORPHELIN encore non notifiés (y compris après crash).
    ///
    /// Décision (écrite — charge jury) : rejouer au **Register de l'émetteur**,
    /// pas seulement au moment de la purge. Les chemins « à aller chercher »
    /// (MCP send / ledger / CLI) sont durables ; le Deliver poussé, lui, écrit
    /// directement sur le socket — ni ledger, ni idempotence, ni accusé. Or la
    /// purge frappe quand la flotte bouge, donc souvent quand l'émetteur est
    /// hors ligne. Sans rejeu au Register, « pas en silence » ne tiendrait que
    /// dans le cas le moins probable. Symétrique de
    /// `schedule_idempotent_delivery_recovery` (côté destinataire, remises
    /// `dispatching`) : ici le destinataire du *signal* est l'émetteur.
    ///
    /// Limite déclarée : `notified_at` atteste l'**émission** (`writeln!`+`flush`
    /// Ok sur la socket), pas la **réception** par le wrapper. Si le wrapper
    /// meurt juste après, le signal est perdu et la base dit qu'il a été
    /// délivré — passage direct de rien à « notifié », sans état « en vol ».
    /// Dimensionnement accepté ; pas corrigé dans ce lot.
    fn flush_pending_orphan_emitter_notices(&mut self) {
        let pending = match self.idempotency.pending_orphan_emitter_notices() {
            Ok(pending) => pending,
            Err(error) => {
                error!("lecture des notices orphelines: {error}");
                return;
            }
        };
        let now = unix_now_secs();
        for (delivery_id, sender, body) in pending {
            if let Some(agent) = self.router.get_agent(&sender)
                && let Some(writer) = self.connections.get(&agent.connection_id)
            {
                match deliver_to_agent(writer, &sender, &body) {
                    Ok(_) => {
                        if let Err(error) = self
                            .idempotency
                            .mark_orphan_emitter_notified(&delivery_id, now)
                        {
                            error!("mark notice orphelin {delivery_id}: {error}");
                        }
                    }
                    Err(error) => warn!("signal orphelin non délivré à {sender}: {error}"),
                }
            } else {
                warn!("émetteur {sender} hors ligne pour signal orphelin {delivery_id}");
            }
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
        if in_progress {
            presence.state = "busy".to_string();
            presence.busy_since = Some(Instant::now());
        } else {
            presence.state = "connected".to_string();
            presence.busy_since = None;
        }
        presence.touch_capacity();
        Ok(())
    }

    /// Libère les tours `busy` plus vieux que notify_timeout + grâce.
    /// Sans TurnState false (tour non abouti), l'agent resterait non mandatable
    /// indéfiniment : Maicie n'envoie qu'aux connected|dnd.
    fn release_stale_busy_turns(&mut self) {
        let busy_ids: Vec<(String, String)> = self
            .presences
            .iter()
            .filter(|(_, presence)| presence.state == "busy")
            .map(|(id, presence)| (id.clone(), presence.agent_type.clone()))
            .collect();
        let mut stale = Vec::new();
        for (id, agent_type) in busy_ids {
            let ttl = live_notify_timeout_secs(&self.registry, &agent_type)
                .saturating_add(TIMEOUT_GRACE_PERIOD);
            if let Some(presence) = self.presences.get(&id) {
                if busy_turn_is_stale(presence, ttl) {
                    stale.push(id);
                }
            }
        }
        for id in stale {
            if let Some(presence) = self.presences.get_mut(&id) {
                presence.state = "connected".to_string();
                presence.busy_since = None;
                presence.touch_capacity();
            }
        }
    }

    fn agent_infos(&mut self) -> Vec<bridget_transport::protocol::AgentInfo> {
        // Avant le retain : un busy périmé redevient connected (mandatable)
        // sans attendre un TurnState false qui peut ne jamais venir.
        self.release_stale_busy_turns();
        // Lot B (déjà sur main) : exemption `connected` levée — retain =
        // horloge lien seule via `presence_within_retention`.
        // Ce lot (purge/orphan) : capturer les IDs purgés pour orpheliner les
        // remises `dispatching` — sans réintroduire l'exemption connected.
        let before: HashSet<String> = self.presences.keys().cloned().collect();
        self.presences
            .retain(|_, presence| presence_within_retention(presence));
        let purged: Vec<String> = before
            .into_iter()
            .filter(|id| !self.presences.contains_key(id))
            .collect();
        // Présence expirée + nom encore au routeur = fantôme. On coupe le lien.
        self.release_router_for_dangling_instances();
        // APRÈS la décision de purger (pas la politique de purge) : rendre
        // visibles les remises qui ne partiront plus.
        self.orphan_deliveries_after_presence_purge(&purged);
        // Rejeu des notices dont le Deliver a échoué / crashé après orphan.
        if purged.is_empty() {
            self.flush_pending_orphan_emitter_notices();
        }
        let mut agents: Vec<_> = self
            .router
            .list_agents()
            .iter()
            .filter_map(|agent| {
                let presence = self
                    .conn_instances
                    .get(&agent.connection_id)
                    .and_then(|id| self.presences.get(id))?;
                // Jamais inventer connected/unix : sans présence attestée, hors
                // annuaire public (connexions MCP éphémères sans instance_id).
                Some(bridget_transport::protocol::AgentInfo {
                    name: agent.name.clone(),
                    agent_type: agent.agent_type.to_string(),
                    connection_id: agent.connection_id.clone(),
                    host: presence.host.clone(),
                    transport: presence.transport.clone(),
                    channel: presence.channel.clone(),
                    mode: presence.mode,
                    location: presence.location.clone(),
                    os: presence.os.clone(),
                    // Un agent qui refuse d'être dérangé est connecté mais non
                    // joignable : du point de vue de l'appelant, la question
                    // « puis-je lui écrire » a la même forme que pour un agent
                    // injoignable, d'où un état unique plutôt qu'une colonne.
                    state: if presence.is_dnd() {
                        "dnd".to_string()
                    } else {
                        presence.state.clone()
                    },
                    // Âge de CAPACITÉ (capacity_seen), pas du lien. Honnête à lire ;
                    // aucune décision maicie/reaper ne s'en sert — elles
                    // regardent `state`. ACP sans événement de contenu → âge
                    // figé (voir regles-chantier, limites de ce lot).
                    last_seen_secs: presence.capacity_seen.elapsed().as_secs(),
                    reconnect_count: presence.reconnect_count,
                    domain: presence.domain.clone(),
                    model: presence.model.clone(),
                    effort: presence.effort.clone(),
                    rate_limits: presence.rate_limits.values().cloned().collect(),
                    model_mismatch: bridget_transport::protocol::ModelMismatchFact::observe(
                        presence.model.as_deref(),
                        presence.served_model.as_deref(),
                    ),
                })
            })
            .collect();
        let live_names: std::collections::HashSet<String> =
            agents.iter().map(|agent| agent.name.clone()).collect();
        for record in self.managed_spawns.values().filter(|record| {
            self.recovery_commands.contains(&record.lease.command_id)
                && !live_names.contains(&record.lease.name)
        }) {
            let managed_definition = self
                .fleet
                .resolved_definition_for_command(&record.lease.command_id);
            let (model, effort) = managed_definition
                .as_ref()
                .and_then(definition_runtime)
                .map(|(model, effort)| (Some(model), effort))
                .unwrap_or((None, None));
            // Même dérivation que handle_register : transport et mode viennent
            // de la définition figée, jamais d'un amalgame unix+acp.
            let (transport, mode) = managed_definition
                .as_ref()
                .map(definition_presence_fields)
                .unwrap_or_else(|| ("unknown".to_string(), None));
            agents.push(bridget_transport::protocol::AgentInfo {
                name: record.lease.name.clone(),
                agent_type: record.agent_type.clone(),
                connection_id: String::new(),
                host: "local".to_string(),
                transport,
                channel: None,
                mode,
                location: None,
                os: std::env::consts::OS.to_string(),
                state: "recovering".to_string(),
                last_seen_secs: 0,
                reconnect_count: 0,
                domain: self.fleet.desired_domain(&record.lease.name),
                model,
                effort,
                rate_limits: Vec::new(),
                model_mismatch: None,
            });
        }
        let listed_names: std::collections::HashSet<String> =
            agents.iter().map(|agent| agent.name.clone()).collect();
        for presence in self.presences.values().filter(|presence| {
            matches!(presence.state.as_str(), "stopped" | "unreachable")
                && !listed_names.contains(&presence.name)
        }) {
            agents.push(bridget_transport::protocol::AgentInfo {
                name: presence.name.clone(),
                agent_type: presence.agent_type.clone(),
                connection_id: String::new(),
                host: presence.host.clone(),
                transport: presence.transport.clone(),
                channel: presence.channel.clone(),
                mode: presence.mode,
                location: presence.location.clone(),
                os: presence.os.clone(),
                state: presence.state.clone(),
                last_seen_secs: presence.capacity_seen.elapsed().as_secs(),
                reconnect_count: presence.reconnect_count,
                domain: presence.domain.clone(),
                // FR-010 : un agent injoignable garde sa dernière capacité connue.
                model: presence.model.clone(),
                effort: presence.effort.clone(),
                rate_limits: presence.rate_limits.values().cloned().collect(),
                model_mismatch: bridget_transport::protocol::ModelMismatchFact::observe(
                    presence.model.as_deref(),
                    presence.served_model.as_deref(),
                ),
            });
        }
        agents.sort_by(|left, right| left.name.cmp(&right.name));
        agents
    }

    fn restore_pending_for_agent(&mut self, name: &str, conn_id: &str) {
        // Une reconnexion conserve le cycle suivi déjà en mémoire, mais son
        // routage doit suivre la nouvelle connexion. Sans ce rattachement,
        // l'état `busy` est restauré alors que les relances continuent de
        // viser l'ancien identifiant et ne sont jamais différées.
        for pending in &mut self.pending_replies {
            if pending.to == name {
                pending.target_conn = conn_id.to_string();
            }
            if pending.from == name {
                pending.from_conn = conn_id.to_string();
            }
        }
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
    }
}

/// Réserve toutes les reprises en ordre lexical avant de libérer la boucle
/// normale. Les `PreparedSpawn` ne sont envoyés au superviseur qu'après la fin
/// de cette fonction, ce qui rend tous les noms visibles à `stop` même si le
/// premier bootstrap bloque.
fn reserve_managed_recoveries(
    state: &mut DaemonState,
    now: i64,
) -> Result<Vec<ManagedRecovery>, Box<dyn std::error::Error>> {
    state.recovering = true;
    let mut absents = Vec::new();
    for (name, _) in state.fleet.drain_non_persistent_named() {
        absents.push(RecoveryLossEntry {
            name,
            reason: REASON_NON_PERSISTENT.to_string(),
            detail: Some("spawn sans --persistent".to_string()),
        });
    }
    let roster_persistents = state.fleet.persistent_named();
    let candidates = state.fleet.recovery_candidates();
    let in_flight_names = candidates
        .iter()
        .map(|candidate| candidate.lease.name.clone())
        .collect::<HashSet<_>>();
    let desired = state.fleet.desired_fleet()?;
    let mut prepared = Vec::new();

    for candidate in candidates {
        if candidate.lease.deadline_at <= now {
            let _ =
                state
                    .fleet
                    .expire(&candidate.lease.command_id, candidate.lease.generation, now);
            absents.push(RecoveryLossEntry {
                name: candidate.lease.name.clone(),
                reason: REASON_RECOVERY_FAILED.to_string(),
                detail: Some("échéance de spawn dépassée avant reprise".to_string()),
            });
            continue;
        }
        let lease = candidate.lease.clone();
        match prepare_recovery(&state.source_env, candidate) {
            Ok(recovery) => prepared.push(recovery),
            Err(reason) => {
                let detail = serde_json::to_string(&reason)
                    .unwrap_or_else(|_| "échec de préparation de reprise".to_string());
                absents.push(RecoveryLossEntry {
                    name: lease.name.clone(),
                    reason: REASON_RECOVERY_FAILED.to_string(),
                    detail: Some(detail.clone()),
                });
                let _ = state.fleet.fail(&lease, "recovery_failed", detail);
            }
        }
    }

    for (name, equipier) in desired.equipiers {
        if in_flight_names.contains(&name) {
            continue;
        }
        let Some(resolved_definition) = equipier.resolved_definition else {
            warn!("reprise de {name} refusée: définition figée absente");
            absents.push(RecoveryLossEntry {
                name: name.clone(),
                reason: REASON_FROZEN_DEFINITION.to_string(),
                detail: Some("définition figée absente".to_string()),
            });
            state.fleet.remove_desired(&name)?;
            continue;
        };
        let order = FleetSpawnOrder {
            agent_type: equipier.agent_type,
            requested_name: Some(name.clone()),
            cwd: equipier.cwd,
            persistent: true,
            command_id: format!("recovery-{}", Uuid::new_v4()),
            issued_at: now,
            deadline_at: now.saturating_add(MANAGED_RECOVERY_DEADLINE_SECS),
        };
        match submit_spawn_from_resolved(
            &state.fleet,
            &state.source_env,
            &order,
            now,
            &resolved_definition,
            // Reprise : le daemon relance ses propres agents chez lui, donc
            // demandeur et exécutant sont la même machine.
            &crate::lifecycle::SpawnHosts::local(),
        )? {
            SpawnDecision::Ready(recovery) => prepared.push(recovery),
            SpawnDecision::Rejected(reason) => {
                if let SpawnRefusal::QuotaExceeded { limit } = &reason {
                    // WARN visible (agent + quota + levier). Trace durable = D20.
                    warn!(
                        "{}",
                        crate::fleet::resume_quota_refusal_message(&name, *limit)
                    );
                    absents.push(RecoveryLossEntry {
                        name: name.clone(),
                        reason: REASON_QUOTA.to_string(),
                        detail: Some(crate::fleet::quota_exceeded_detail(*limit)),
                    });
                } else {
                    warn!("reprise de {name} refusée: {reason:?}");
                    absents.push(RecoveryLossEntry {
                        name: name.clone(),
                        reason: REASON_RECOVERY_FAILED.to_string(),
                        detail: Some(format!("{reason:?}")),
                    });
                }
                state.fleet.remove_desired(&name)?;
            }
            SpawnDecision::EnvelopeMismatch => {
                warn!("reprise de {name} refusée: enveloppe divergente");
                absents.push(RecoveryLossEntry {
                    name: name.clone(),
                    reason: REASON_RECOVERY_FAILED.to_string(),
                    detail: Some("enveloppe divergente".to_string()),
                });
                state.fleet.remove_desired(&name)?;
            }
            SpawnDecision::Await(_) | SpawnDecision::Accepted { .. } => {
                warn!("reprise de {name} rattachée à un état inattendu");
                absents.push(RecoveryLossEntry {
                    name: name.clone(),
                    reason: REASON_RECOVERY_FAILED.to_string(),
                    detail: Some("état de reprise inattendu".to_string()),
                });
                state.fleet.remove_desired(&name)?;
            }
        }
    }

    let remaining = state.fleet.desired_fleet()?;
    let already: HashSet<String> = absents.iter().map(|entry| entry.name.clone()).collect();
    for (name, _) in roster_persistents {
        if remaining.equipiers.contains_key(&name) || already.contains(&name) {
            continue;
        }
        absents.push(RecoveryLossEntry {
            name: name.clone(),
            reason: REASON_ABSENT_FROM_FLEET.to_string(),
            detail: Some("présent au roster persistant, absent de fleet.json".to_string()),
        });
        state.fleet.forget_named(&name);
    }

    if let Err(error) = state.fleet.persist_recovery_losses(now, absents) {
        warn!("trace de reprise non écrite: {error}");
    }

    prepared.sort_by(|left, right| left.lease.name.cmp(&right.lease.name));
    for (recovery, stop) in prepared
        .iter()
        .map(|recovery| (recovery, Arc::new(ManagedStopControl::new())))
    {
        state.managed_by_instance.insert(
            recovery.lease.instance_id.clone(),
            recovery.lease.command_id.clone(),
        );
        state
            .recovery_commands
            .insert(recovery.lease.command_id.clone());
        state.managed_spawns.insert(
            recovery.lease.command_id.clone(),
            ManagedSpawnRecord {
                lease: recovery.lease.clone(),
                agent_type: recovery.agent_type.clone(),
                requester_conns: Vec::new(),
                wrapper_conn: None,
                stop,
            },
        );
    }
    state.recovering = !state.recovery_commands.is_empty();

    Ok(prepared
        .into_iter()
        .map(|recovery| {
            let stop = state
                .managed_spawns
                .get(&recovery.lease.command_id)
                .expect("reprise insérée sous le même verrou")
                .stop
                .clone();
            (recovery, stop)
        })
        .collect())
}

fn defer_idempotent_delivery(
    state: &DaemonState,
    target_conn: &str,
    delivery: SendDelivery,
    controls: &mut Vec<DeferredControl>,
) -> Result<(), String> {
    let mut message: bridget_core::BridgetMessage = serde_json::from_slice(&delivery.message_bytes)
        .map_err(|error| format!("enveloppe de remise idempotente corrompue: {error}"))?;
    // Même autorité que Deliver classique : relire à la poussée. Les octets
    // persistés (souvent sans deadline pour reply=false) ne doivent pas
    // condamner le tour au notify figé du fleet (600 s mesuré sur relec6).
    let agent_type = state
        .conn_instances
        .get(target_conn)
        .and_then(|instance_id| state.presences.get(instance_id))
        .map(|presence| presence.agent_type.as_str())
        .unwrap_or("");
    stamp_turn_deadline_for_delivery(&mut message, &state.registry, agent_type);
    defer_control(
        state,
        target_conn,
        DaemonToWrapper::DeliverIdempotent {
            delivery_id: delivery.delivery_id,
            recipient_instance_id: delivery.recipient_instance_id,
            delivery_generation: delivery.delivery_generation,
            expires_at: delivery.expires_at,
            message,
        },
        controls,
    );
    Ok(())
}

/// La reprise n'est déclenchée qu'au réenregistrement de l'instance ciblée :
/// au démarrage, aucun socket wrapper n'existe encore. Les bytes persistés et
/// le couple instance/génération sont les seules autorités ; aucun nom n'est
/// résolu une seconde fois.
fn schedule_idempotent_delivery_recovery(
    state: &mut DaemonState,
    conn_id: &str,
    instance_id: &str,
) {
    let deliveries = match state
        .idempotency
        .dispatching_deliveries_for_instance(instance_id, unix_now_secs())
    {
        Ok(deliveries) => deliveries,
        Err(error) => {
            error!("reprise des remises idempotentes: {error}");
            return;
        }
    };
    let mut controls = Vec::new();
    for delivery in deliveries {
        if let Err(error) =
            defer_idempotent_delivery(state, conn_id, delivery.clone(), &mut controls)
        {
            error!("reprise idempotente mise en quarantaine: {error}");
            let _ = state.idempotency.mark_delivery_indeterminate(
                &delivery.delivery_id,
                &delivery.recipient_instance_id,
                delivery.delivery_generation,
            );
        }
    }
    if !controls.is_empty() {
        state
            .pending_post_response_controls
            .entry(conn_id.to_string())
            .or_default()
            .extend(controls);
    }
}

/// Lance le daemon.
pub fn run(config: DaemonConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Gestionnaires de signaux AVANT toute trace visible de l'extérieur.
    //
    // Ils étaient installés après la liaison de la socket : entre le moment où
    // le daemon devenait joignable et celui où il devenait interruptible
    // proprement, un SIGTERM tombait sur la disposition PAR DÉFAUT et tuait le
    // processus net — socket et fichier PID abandonnés derrière lui. Un
    // démarrage suivi d'un arrêt immédiat laissait donc des reliques que le
    // démarrage suivant devait déblayer. Le drapeau est remis à zéro d'abord :
    // un SIGTERM arrivé avant la boucle sera vu à sa première itération.
    SHUTDOWN_REQUESTED.store(false, Ordering::SeqCst);
    install_daemon_signal_handlers();

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

    let marker_store = ManagedMarkerStore::at_directory(
        config
            .db_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("/tmp"))
            .join("managed"),
    );
    let reconciled =
        marker_store.reconcile_stale_groups(MANAGED_STOP_FORCED_GRACE, MANAGED_STOP_POLL)?;
    if !reconciled.is_empty() {
        info!(
            "réconciliation: {} ancien(s) groupe(s) terminé(s): {}",
            reconciled.len(),
            reconciled.join(", ")
        );
    }

    if config.socket_path.exists() {
        std::fs::remove_file(&config.socket_path)?;
    }
    std::fs::create_dir_all(config.socket_path.parent().unwrap())?;
    crate::wrapper::purge_orphan_mcp_configs(config.socket_path.parent().unwrap());
    // WARN disque synchrone (statvfs = O(1)). Le ramassage /tmp sort du chemin
    // critique — thread détaché après readiness (bisect fable2).
    crate::disk_hygiene::warn_if_disk_low(std::path::Path::new("/"));

    let listener = UnixListener::bind(&config.socket_path)?;
    info!("bridget daemon écoute sur {}", config.socket_path.display());

    let tmp_dir = std::env::temp_dir();
    if let Err(error) = std::thread::Builder::new()
        .name("ramasse-copies".into())
        .spawn(move || {
            let report = crate::disk_hygiene::purge_orphan_bridget_tmp(&tmp_dir);
            if !report.deleted.is_empty() {
                info!(
                    "ramasse-copies: {} orphelin(s) /tmp/bridget-* retiré(s)",
                    report.deleted.len()
                );
            }
            if report.deferred > 0 {
                info!(
                    "ramasse-copies: {} restant(s) pour un prochain passage",
                    report.deferred
                );
            }
        })
    {
        warn!("ramasse-copies: thread détaché impossible: {error}");
    }

    let (managed_tx, managed_rx) = mpsc::channel();
    let (managed_event_tx, managed_event_rx) = mpsc::channel();
    let state = Arc::new(Mutex::new(DaemonState::new(&config, managed_tx.clone())?));
    let fleet = state
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .fleet
        .clone();
    #[cfg(test)]
    let executable_override =
        std::env::var_os("BRIDGET_T908_MANAGED_EXECUTABLE").map(PathBuf::from);
    #[cfg(not(test))]
    let executable_override = None;
    let supervisor_guard = ManagedSupervisorGuard::from_existing_sender(
        managed_tx,
        managed_rx,
        fleet,
        &config,
        managed_event_tx,
        executable_override,
    );
    let recoveries = {
        let mut st = state.lock().unwrap_or_else(|poison| poison.into_inner());
        reserve_managed_recoveries(&mut st, unix_timestamp())?
    };
    for (prepared, stop) in recoveries {
        let command_id = prepared.lease.command_id.clone();
        let instance_id = prepared.lease.instance_id.clone();
        let sent = {
            let st = state.lock().unwrap_or_else(|poison| poison.into_inner());
            st.managed_tx
                .send(ManagedSupervisorCommand::Start { prepared, stop })
        };
        if sent.is_err() {
            let mut st = state.lock().unwrap_or_else(|poison| poison.into_inner());
            if let Some(record) = st.managed_spawns.remove(&command_id) {
                let _ = st.fleet.fail(
                    &record.lease,
                    "negotiation_failed",
                    "superviseur indisponible pendant la reprise",
                );
            }
            st.managed_by_instance.remove(&instance_id);
            finish_recovery_command(&mut st, &command_id);
        }
    }

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
            // Sortie sur arrêt : ce thread détient un clone de l'Arc d'état, et
            // l'état détient `managed_tx`. Tant qu'il boucle, le canal du
            // superviseur ne peut PAS se fermer et l'arrêt reste suspendu.
            if sleep_until_shutdown(Duration::from_secs(1)) {
                return;
            }
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
                        let target_writer = {
                            let st = st_reminder.lock().unwrap_or_else(|e| e.into_inner());
                            st.connections.get(&target_conn).cloned()
                        };
                        if let Some(target_writer) = target_writer {
                            let body = format!(
                                "Rappel : {} attend ta reponse au message #{}.\nReponds avec: bridget reply \"ta reponse\"",
                                from,
                                &msg_id[..msg_id.len().min(8)]
                            );
                            match deliver_to_agent(&target_writer, &to, &body) {
                                Ok(reminder_message_id) => attest_reminder_sent(
                                    &st_reminder,
                                    &msg_id,
                                    &reminder_message_id,
                                    &to,
                                    1,
                                ),
                                Err(e) => {
                                    error!("Impossible de délivrer le rappel doux à {}: {}", to, e)
                                }
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
                        let target_writer = {
                            let st = st_reminder.lock().unwrap_or_else(|e| e.into_inner());
                            st.connections.get(&target_conn).cloned()
                        };
                        if let Some(target_writer) = target_writer {
                            let body = format!(
                                "URGENT : {} attend toujours ta reponse au message #{}.\nTu DOIS repondre maintenant avec: bridget reply \"ta reponse\"\nSi tu ne peux pas repondre, notifie-le : bridget reply \"impossible de repondre : <raison>\"",
                                from,
                                &msg_id[..msg_id.len().min(8)]
                            );
                            match deliver_to_agent(&target_writer, &to, &body) {
                                Ok(reminder_message_id) => attest_reminder_sent(
                                    &st_reminder,
                                    &msg_id,
                                    &reminder_message_id,
                                    &to,
                                    2,
                                ),
                                Err(e) => {
                                    error!("Impossible de délivrer le rappel ferme à {}: {}", to, e)
                                }
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
            // Même raison qu'au-dessus, en plus aigu : une sieste d'une heure
            // gardait l'état vivant une heure de plus après l'ordre d'arrêt.
            if sleep_until_shutdown(Duration::from_secs(3600)) {
                return;
            }
            let st = st_purge.lock().unwrap_or_else(|e| e.into_inner());
            if let Ok(n) = st.store.purge_older_than_days(retention)
                && n > 0
            {
                info!("purge périodique: {} messages supprimés", n);
            }
        }
    });

    // Boucle d'acceptation avec timeout pour vérifier shutdown
    listener.set_nonblocking(true)?;
    loop {
        drain_managed_events(&state, &managed_event_rx);
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
            drop(state);
            supervisor_guard.shutdown();
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

    // Toute sortie, y compris un échec d'écriture de réponse, traverse le
    // nettoyage commun ci-dessous. Une capacité de service est strictement
    // attachée à la connexion : elle ne doit jamais survivre à son socket.
    let connection_result = (|| -> Result<(), Box<dyn std::error::Error>> {
        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(_) => break,
            };
            if line.is_empty() {
                continue;
            }
            if guichet_frame_exceeds_wire_limit(&line) {
                let json = encode(&DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::FrameTooLarge,
                })?;
                writeln!(my_writer, "{}", json)?;
                my_writer.flush()?;
                continue;
            }

            let msg: WrapperToDaemon = match decode(&line) {
                Ok(m) => m,
                Err(e) => {
                    warn!("message illisible de {}: {}", conn_id, e);
                    if raw_guichet_frame(&line) {
                        let json = encode(&DaemonToWrapper::ServiceRejected {
                            reason: ServiceRefusal::InvalidEnvelope,
                        })?;
                        writeln!(my_writer, "{}", json)?;
                        my_writer.flush()?;
                    }
                    continue;
                }
            };

            // Le guichet compare les octets, non une valeur JSON reparsée :
            // cette vérification rejette aussi champs inconnus, doublons et
            // ordre de clés divergent avant toute écriture SQLite.
            if is_guichet_frame(&msg) && encode(&msg).is_ok_and(|canonical| canonical != line) {
                let json = encode(&DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::CanonicalBytesMismatch,
                })?;
                writeln!(my_writer, "{}", json)?;
                my_writer.flush()?;
                continue;
            }

            let response = handle_wrapper_message(&conn_id, msg, &state);
            let registered_just_now = matches!(response, Some(DaemonToWrapper::Registered { .. }));
            if let Some(dtw) = response {
                let json = encode(&dtw)?;
                writeln!(my_writer, "{}", json)?;
                my_writer.flush()?;
            }
            let post_response_controls = state
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .pending_post_response_controls
                .remove(&conn_id)
                .unwrap_or_default();
            let _ = execute_controls(post_response_controls);
            // Après Registered : rejouer les notices ORPHELIN dont l'émetteur
            // était hors ligne à la purge (voir flush_pending_orphan_emitter_notices).
            if registered_just_now {
                state
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .flush_pending_orphan_emitter_notices();
            }
        }
        Ok(())
    })();

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
        st.service_negotiations.remove(&conn_id);
        st.coordination_subscriptions.remove(&conn_id);
        let _ = st.store.release_guichet_claims(&conn_id);
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

    connection_result
}

fn is_guichet_frame(message: &WrapperToDaemon) -> bool {
    matches!(
        message,
        WrapperToDaemon::ServiceHello { .. }
            | WrapperToDaemon::CoordinationSubscribe { .. }
            | WrapperToDaemon::ServiceRequest { .. }
            | WrapperToDaemon::GuichetClaimNext { .. }
            | WrapperToDaemon::GuichetClaim { .. }
            | WrapperToDaemon::GuichetLookup { .. }
            | WrapperToDaemon::GuichetReply { .. }
    )
}

/// `BufRead::lines` enlève le séparateur : la borne du contrat porte bien sur
/// la trame JSONL entière, donc sur la ligne plus son LF filaire.
fn guichet_frame_exceeds_wire_limit(line: &str) -> bool {
    line.len().saturating_add(1) > MAX_GUICHET_FRAME_BYTES && raw_guichet_frame(line)
}

/// Classe la famille du message à partir du JSON, jamais d'une sous-chaîne :
/// les espaces et l'ordre des clés ne doivent pas contourner la garde avant
/// toute consommation de capacité ou écriture durable.
fn raw_guichet_frame(line: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(line)
        .ok()
        .and_then(|value| {
            value
                .get("type")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .is_some_and(|kind| {
            matches!(
                kind.as_str(),
                "ServiceHello"
                    | "coordination_subscribe"
                    | "service_request"
                    | "guichet_claim_next"
                    | "guichet_claim"
                    | "guichet_lookup"
                    | "guichet_reply"
                    | "coordination_event"
            )
        })
}

fn guichet_request_is_valid(
    request_id: &str,
    operation: bridget_transport::protocol::ServiceRequestOperation,
    payload: &bridget_transport::protocol::ServiceRequestPayload,
) -> bool {
    let identifier = |value: &str| {
        !value.is_empty()
            && value.len() <= 128
            && value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-')
            })
    };
    if !identifier(request_id) {
        return false;
    }
    match (operation, payload) {
        (
            bridget_transport::protocol::ServiceRequestOperation::DeliveryReport,
            bridget_transport::protocol::ServiceRequestPayload::DeliveryReport {
                objective_id,
                delegation_id,
                delivery_hash,
                in_reply_to,
                review_verdict,
            },
        ) => {
            identifier(objective_id)
                && identifier(delegation_id)
                && identifier(in_reply_to)
                && delivery_hash.len() == 64
                && delivery_hash.bytes().all(|byte| byte.is_ascii_hexdigit())
                && review_verdict
                    .as_ref()
                    .is_none_or(|evidence| evidence.is_valid())
        }
        (
            bridget_transport::protocol::ServiceRequestOperation::MissionStatus
            | bridget_transport::protocol::ServiceRequestOperation::DeadlineQuestion,
            bridget_transport::protocol::ServiceRequestPayload::Delegation { delegation_id },
        ) => identifier(delegation_id),
        (
            bridget_transport::protocol::ServiceRequestOperation::Delegate,
            bridget_transport::protocol::ServiceRequestPayload::Delegate {
                goal,
                explicit_target,
                required_tags,
                duration: _,
                suite,
                depends_on,
                references,
            },
        ) => {
            let suite_is_valid = match suite {
                bridget_transport::protocol::ServiceSuiteDeclaration::Aucune => true,
                bridget_transport::protocol::ServiceSuiteDeclaration::Objectif { objective_id } => {
                    identifier(objective_id)
                }
            };
            let mut relations = std::collections::BTreeSet::new();
            let relations_are_valid = depends_on
                .iter()
                .chain(references)
                .all(|value| identifier(value) && relations.insert(value.as_str()));
            !goal.trim().is_empty()
                && goal.len() <= 16 * 1024
                && explicit_target
                    .as_ref()
                    .is_none_or(|target| identifier(target))
                && required_tags.len() <= 32
                && required_tags.iter().all(|tag| identifier(tag))
                && required_tags
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    == required_tags.len()
                && depends_on.len() <= 100
                && references.len() <= 100
                && suite_is_valid
                && relations_are_valid
        }
        (
            bridget_transport::protocol::ServiceRequestOperation::RegistreAdd,
            bridget_transport::protocol::ServiceRequestPayload::RegistreAdd { line },
        ) => !line.trim().is_empty() && line.len() <= 32 * 1024,
        (
            bridget_transport::protocol::ServiceRequestOperation::ObjectiveClose,
            bridget_transport::protocol::ServiceRequestPayload::ObjectiveClose {
                objective_id,
                reason,
            },
        ) => identifier(objective_id) && !reason.trim().is_empty() && reason.len() <= 4096,
        _ => false,
    }
}

fn greffe_mutation_action(
    operation: bridget_transport::protocol::ServiceRequestOperation,
) -> Option<GreffeMutationAction> {
    match operation {
        bridget_transport::protocol::ServiceRequestOperation::Delegate => {
            Some(GreffeMutationAction::Delegate)
        }
        bridget_transport::protocol::ServiceRequestOperation::RegistreAdd => {
            Some(GreffeMutationAction::RegistreAdd)
        }
        bridget_transport::protocol::ServiceRequestOperation::ObjectiveClose => {
            Some(GreffeMutationAction::ObjectiveClose)
        }
        bridget_transport::protocol::ServiceRequestOperation::DeliveryReport
        | bridget_transport::protocol::ServiceRequestOperation::MissionStatus
        | bridget_transport::protocol::ServiceRequestOperation::DeadlineQuestion => None,
    }
}

fn guichet_reply_is_valid(
    request_id: &str,
    response_message_id: &str,
    in_reply_to: &str,
    payload: &bridget_transport::protocol::GuichetReplyPayload,
) -> bool {
    let identifier = |value: &str| {
        !value.is_empty()
            && value.len() <= 128
            && value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-')
            })
    };
    if !identifier(request_id) || !identifier(response_message_id) || !identifier(in_reply_to) {
        return false;
    }
    match payload {
        bridget_transport::protocol::GuichetReplyPayload::DeliveryReport {
            objective_id,
            delegation_id,
            delivery_hash,
            review_verdict,
        } => {
            identifier(objective_id)
                && identifier(delegation_id)
                && delivery_hash.len() == 64
                && delivery_hash.bytes().all(|byte| byte.is_ascii_hexdigit())
                && review_verdict
                    .as_ref()
                    .is_none_or(|evidence| evidence.is_valid())
        }
        bridget_transport::protocol::GuichetReplyPayload::MissionStatus {
            delegation_id,
            objective_id,
            transport_observation,
            ..
        } => {
            identifier(delegation_id)
                && identifier(objective_id)
                && transport_observation.as_ref().is_none_or(|observation| {
                    !observation.source.is_empty()
                        && observation.source.len() <= 128
                        && observation
                            .subscription_id
                            .as_deref()
                            .is_none_or(identifier)
                        && observation.request_state.as_deref().is_none_or(identifier)
                })
        }
        bridget_transport::protocol::GuichetReplyPayload::DeadlineQuestion {
            delegation_id,
            ..
        } => identifier(delegation_id),
        bridget_transport::protocol::GuichetReplyPayload::Delegate {
            status,
            objective_id,
            delegation_id,
            message_id,
            participant,
            candidates,
            waiting_on_prerequisites,
            ..
        } => match status {
            bridget_transport::protocol::GuichetDelegateMutationStatus::Created => {
                objective_id.as_deref().is_some_and(identifier)
                    && delegation_id.as_deref().is_some_and(identifier)
                    && message_id.as_deref().is_none_or(identifier)
                    && participant.as_deref().is_some_and(identifier)
                    && candidates.is_empty()
                    && !(*waiting_on_prerequisites && message_id.is_some())
            }
            bridget_transport::protocol::GuichetDelegateMutationStatus::SelectionRequired => {
                objective_id.is_none()
                    && delegation_id.is_none()
                    && message_id.is_none()
                    && participant.is_none()
                    && !*waiting_on_prerequisites
                    && candidates.len() <= 128
                    && candidates.iter().all(|value| identifier(value))
                    && candidates
                        .iter()
                        .collect::<std::collections::BTreeSet<_>>()
                        .len()
                        == candidates.len()
            }
        },
        bridget_transport::protocol::GuichetReplyPayload::RegistreAdd { constat_id, .. } => {
            identifier(constat_id)
        }
        bridget_transport::protocol::GuichetReplyPayload::ObjectiveClose {
            objective_id,
            decision_id,
            ..
        } => identifier(objective_id) && identifier(decision_id),
        bridget_transport::protocol::GuichetReplyPayload::Refused { .. } => true,
    }
}

fn guichet_result_response(
    issuer_scope: String,
    request_id: String,
    result: GuichetResult,
) -> DaemonToWrapper {
    let (issue, expires_at, payload) = match result {
        GuichetResult::Queued { expires_at } => ("queued".to_string(), expires_at, None),
        GuichetResult::OutcomeUnknown { expires_at } => {
            ("outcome_unknown".to_string(), expires_at, None)
        }
        GuichetResult::Terminal {
            issue,
            expires_at,
            reply_bytes,
            ..
        } => {
            let Ok(reply) = std::str::from_utf8(&reply_bytes)
                .ok()
                .and_then(|line| decode(line).ok())
                .ok_or(())
            else {
                return DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::TransitionInvalid,
                };
            };
            let WrapperToDaemon::GuichetReply { payload, .. } = reply else {
                return DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::TransitionInvalid,
                };
            };
            (issue, expires_at, Some(payload))
        }
        GuichetResult::CanonicalBytesMismatch => ("canonical_bytes_mismatch".to_string(), 0, None),
        GuichetResult::IdempotencyExpired => ("idempotency_expired".to_string(), 0, None),
        GuichetResult::InvalidIssuedAt => ("invalid_issued_at".to_string(), 0, None),
        GuichetResult::ClaimStale => ("claim_stale".to_string(), 0, None),
    };
    DaemonToWrapper::GuichetResult {
        version: SERVICE_CONTRACT_VERSION,
        issuer_scope,
        request_id,
        issue,
        expires_at,
        payload,
    }
}

fn guichet_claim_response(claim: crate::store::GuichetClaim) -> DaemonToWrapper {
    DaemonToWrapper::GuichetClaimed {
        version: SERVICE_CONTRACT_VERSION,
        issuer_scope: claim.issuer_scope,
        request_id: claim.request_id,
        canonical_request: claim.canonical_request,
        authorization_attestation: claim.authorization_attestation,
        claimed_at: claim.claimed_at,
        claim_generation: claim.claim_generation,
        claim_token: claim.claim_token,
        claim_lease_expires_at: claim.claim_lease_expires_at,
        expires_at: claim.expires_at,
    }
}

fn guichet_lifecycle_response(event: GuichetLifecycleEvent) -> DaemonToWrapper {
    DaemonToWrapper::RequestLifecycleEvent {
        version: SERVICE_CONTRACT_VERSION,
        issuer_scope: event.issuer_scope,
        event_id: event.event_id,
        request_id: event.request_id,
        state: event.state,
        observed_at: event.observed_at,
        in_reply_to: event.in_reply_to,
        response_message_id: event.response_message_id,
    }
}

fn guichet_coordination_response(event: GuichetCoordinationEvent, version: u16) -> DaemonToWrapper {
    DaemonToWrapper::CoordinationEvent {
        version,
        event_id: event.event_id,
        request_id: event.request_id,
        kind: event.kind,
        reminder_message_id: event.reminder_message_id,
        recipient: event.recipient,
        generation: event.generation,
        observed_at: event.observed_at,
        cursor: (version == COORDINATION_STREAM_VERSION).then_some(event.cursor),
    }
}

/// Atteste le rappel seulement après l'écriture et le flush effectifs vers son
/// destinataire. B (persistance Maicie) relève ce fait pour l'observation,
/// C (reconcile) le déduplique par `event_id`; aucun consommateur ne calcule
/// l'instant ni la génération à la place du transport.
fn attest_reminder_sent(
    state: &Arc<Mutex<DaemonState>>,
    request_id: &str,
    reminder_message_id: &str,
    recipient: &str,
    generation: u64,
) {
    #[cfg(feature = "test-support")]
    crate::test_sync::checkpoint("before_coordination_persist");
    let controls = {
        let mut st = state.lock().unwrap_or_else(|error| error.into_inner());
        let event = match st.store.record_reminder_sent(
            request_id,
            reminder_message_id,
            recipient,
            generation,
            unix_timestamp(),
        ) {
            Ok(event) => event,
            Err(error) => {
                error!("persistance du rappel de coordination: {error}");
                return;
            }
        };
        #[cfg(feature = "test-support")]
        crate::test_sync::checkpoint("after_coordination_persist");
        let mut controls = Vec::new();
        let negotiations = st
            .service_negotiations
            .iter()
            .map(|(connection_id, negotiated)| (connection_id.clone(), negotiated.clone()))
            .collect::<Vec<_>>();
        for (connection_id, negotiated) in negotiations {
            let version = if negotiated
                .capabilities
                .contains(&ServiceCapability::CoordinationEventsV1)
            {
                Some(COORDINATION_EVENTS_VERSION)
            } else if negotiated
                .capabilities
                .contains(&ServiceCapability::CoordinationEventsV2)
                && st
                    .coordination_subscriptions
                    .get(&connection_id)
                    .is_some_and(|subscription| {
                        subscription.fresh && subscription.next_cursor == event.cursor
                    })
            {
                Some(COORDINATION_STREAM_VERSION)
            } else {
                None
            };
            let Some(version) = version else {
                continue;
            };
            if version == COORDINATION_STREAM_VERSION
                && let Some(subscription) = st.coordination_subscriptions.get_mut(&connection_id)
            {
                subscription.next_cursor = event.cursor.saturating_add(1);
            }
            if let Some(writer) = st.connections.get(&connection_id) {
                controls.push(DeferredControl {
                    writer: writer.clone(),
                    message: guichet_coordination_response(event.clone(), version),
                });
            }
        }
        controls
    };
    let _ = execute_controls(controls);
}

/// Traite l'enregistrement d'un wrapper
///
/// Les champs du message `Register` restent dépliés ici pour refléter le
/// protocole de transport ; les regrouper imposerait un refactor hors scope.
fn definition_runtime(definition: &ResolvedAgentDefinition) -> Option<(String, Option<String>)> {
    crate::registry::runtime_model_and_effort(&definition.args)
}

/// Le mode d'un équipier géré vient de sa définition figée, pas de son type
/// de fournisseur ni de la version du wrapper qui se réenregistre.
fn definition_presence_mode(definition: &ResolvedAgentDefinition) -> Option<PresenceMode> {
    match definition.protocol.as_str() {
        "acp" => Some(PresenceMode::Acp),
        "claude_stream_json" => Some(PresenceMode::Cli),
        "codex_app_server" => Some(PresenceMode::Cli),
        "tmux" => Some(PresenceMode::Tmux),
        _ => None,
    }
}

/// Transport et mode affichés pour un géré : toujours `definition.protocol`,
/// jamais un canal inventé. Unique dérivation, partagée par `handle_register`
/// et la projection `recovering` de `who`.
fn definition_presence_fields(
    definition: &ResolvedAgentDefinition,
) -> (String, Option<PresenceMode>) {
    (
        definition.protocol.clone(),
        definition_presence_mode(definition),
    )
}

/// Valeurs historiques qui décrivent un tuyau plutôt qu'un protocole
/// d'agent. Elles ne doivent jamais réapparaître dans `AgentInfo.transport`.
fn legacy_non_protocol_transport(value: &str) -> bool {
    matches!(value, "unix" | "ssh" | "ssh-unix" | "stdio")
}

fn non_empty_registration_value(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Définition figée d'un géré déjà connu. L'instance courante est la clé
/// nominale ; une ré-inscription (reprise, wrapper antérieur) peut arriver
/// avec un autre identifiant tout en portant le même nom de lease.
fn managed_definition_for_register(
    state: &DaemonState,
    instance_id: &str,
    name: &str,
) -> Option<ResolvedAgentDefinition> {
    state
        .managed_by_instance
        .get(instance_id)
        .and_then(|command_id| state.fleet.resolved_definition_for_command(command_id))
        .or_else(|| {
            state.managed_spawns.values().find_map(|record| {
                (record.lease.name == name)
                    .then(|| {
                        state
                            .fleet
                            .resolved_definition_for_command(&record.lease.command_id)
                    })
                    .flatten()
            })
        })
}

#[allow(clippy::too_many_arguments)]
fn handle_register_with_channel(
    conn_id: &str,
    agent_type: String,
    name: Option<String>,
    host: Option<String>,
    transport: Option<String>,
    channel: ChannelReport,
    mode: Option<PresenceMode>,
    location: Option<String>,
    os: Option<String>,
    instance_id: Option<String>,
    domain: Option<String>,
    turn_in_progress: bool,
    journal_available: Option<bool>,
    state: &mut DaemonState,
) -> DaemonToWrapper {
    log::debug!(
        "Register reçu de {}: type={}, name={:?}, host={:?}",
        conn_id,
        agent_type,
        name,
        host
    );

    let auxiliary_mcp = agent_type == "mcp";
    let requested_name = name.clone();
    let parsed_type = agent_type
        .parse()
        .unwrap_or(bridget_core::AgentType::Custom(agent_type));

    // Takeover sans stop : si le nom est tenu par un fantôme (routeur sans
    // présence live), on libère avant d'enregistrer — c'est ce qui forçait
    // trois interventions manuelles « stop puis spawn » la nuit du constat.
    if let Some(requested) = name.as_deref() {
        let _ = state.reclaim_phantom_name(requested);
    }

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
                let presence_owned_by_live_connection =
                    state
                        .conn_instances
                        .iter()
                        .any(|(existing_conn, existing_instance)| {
                            existing_conn != conn_id
                                && !state.auxiliary_connections.contains(existing_conn)
                                && existing_instance == &instance_id
                                && state.presences.get(&instance_id).is_some_and(|presence| {
                                    matches!(presence.state.as_str(), "connected" | "busy")
                                })
                        });
                if presence_owned_by_live_connection {
                    let canonical_mcp_name = state
                        .presences
                        .get(&instance_id)
                        .filter(|presence| {
                            auxiliary_mcp
                                && requested_name.as_deref() == Some(presence.name.as_str())
                        })
                        .map(|presence| presence.name.clone());
                    if let Some(canonical_name) = canonical_mcp_name {
                        // Le MCP est une filiation du wrapper : conserver son
                        // principal exact pour l'autorisation, mais retirer sa
                        // route auxiliaire afin qu'il n'apparaisse jamais comme
                        // un second équipier dans `who`.
                        state.router.unregister_by_conn(conn_id);
                        state
                            .conn_names
                            .insert(conn_id.to_string(), canonical_name.clone());
                        state
                            .conn_instances
                            .insert(conn_id.to_string(), instance_id.clone());
                        state.auxiliary_connections.insert(conn_id.to_string());
                        return DaemonToWrapper::Registered {
                            name: canonical_name,
                        };
                    }
                    let same_equipier = state
                        .presences
                        .get(&instance_id)
                        .is_some_and(|presence| presence.name == final_name);
                    if !same_equipier {
                        // Une connexion auxiliaire issue de la filiation MCP peut
                        // revendiquer la même instance que le wrapper. Elle garde
                        // son entrée de routage éphémère, mais ne devient jamais
                        // propriétaire de la présence : sa fermeture ne doit pas
                        // rendre le wrapper inaccessible ni effacer ses faits.
                        info!(
                            "présence {} conservée : connexion auxiliaire {} ignorée",
                            instance_id, conn_id
                        );
                        state.restore_pending_for_agent(&final_name, conn_id);
                        return DaemonToWrapper::Registered { name: final_name };
                    }
                    // Réconnexion du même équipier avant l'EOF de l'ancienne
                    // connexion : voler l'instance pour que mark_unreachable
                    // retardé ne puisse plus écraser busy / connected.
                    let auxiliary_connections = &state.auxiliary_connections;
                    state
                        .conn_instances
                        .retain(|existing_conn, existing_instance| {
                            existing_conn == conn_id
                                || auxiliary_connections.contains(existing_conn)
                                || existing_instance.as_str() != instance_id
                        });
                }

                let previous_instance_key = if state.presences.contains_key(&instance_id) {
                    Some(instance_id.clone())
                } else {
                    state
                        .presences
                        .iter()
                        .find(|(_, presence)| presence.name == final_name)
                        .map(|(key, _)| key.clone())
                };
                let previous = previous_instance_key
                    .as_ref()
                    .and_then(|key| state.presences.get(key).cloned());
                let reconnect_count = previous
                    .as_ref()
                    .map(|presence| {
                        presence.reconnect_count + u32::from(presence.state != "connected")
                    })
                    .unwrap_or(0);
                let managed_definition =
                    managed_definition_for_register(state, &instance_id, &final_name);
                let managed_runtime = managed_definition
                    .as_ref()
                    .and_then(definition_runtime)
                    .map(|(model, effort)| (Some(model), effort));
                let (managed_transport, managed_mode) = match managed_definition.as_ref() {
                    Some(definition) => {
                        let (transport, mode) = definition_presence_fields(definition);
                        (Some(transport), mode)
                    }
                    None => (None, None),
                };
                // Un géré tient son runtime de la définition figée. Une
                // reconnexion interactive conserve, elle, la dernière sonde.
                let (model, effort) = managed_runtime.unwrap_or_else(|| {
                    previous
                        .as_ref()
                        .map(|presence| (presence.model.clone(), presence.effort.clone()))
                        .unwrap_or((None, None))
                });
                let derived_domain = previous
                    .as_ref()
                    .and_then(|presence| {
                        presence
                            .derived_domain
                            .clone()
                            .or_else(|| presence.domain.clone())
                    })
                    .or_else(|| domain.clone());
                let dnd_until = previous.as_ref().and_then(|presence| presence.dnd_until);
                // Les faits de limite restent indexés par fenêtre à travers
                // une reconnexion ; une fenêtre absente demeure absente.
                let rate_limits = previous
                    .as_ref()
                    .map(|presence| presence.rate_limits.clone())
                    .unwrap_or_default();
                let served_model = previous
                    .as_ref()
                    .and_then(|presence| presence.served_model.clone());
                // Une reconnexion par un binaire antérieur au champ conserve
                // l'observation déjà attestée ; une présence historique sans
                // valeur reste volontairement inconnue.
                let mode = managed_mode.or_else(|| {
                    previous
                        .as_ref()
                        .and_then(|presence| presence.mode)
                        .or(mode)
                });
                let location = match mode {
                    Some(PresenceMode::Tmux) => previous
                        .as_ref()
                        .and_then(|presence| presence.location.clone())
                        .or(location),
                    _ => None,
                };
                let host = previous
                    .as_ref()
                    .filter(|presence| presence.host != "inconnu")
                    .map(|presence| presence.host.clone())
                    .or(host)
                    .unwrap_or_else(|| "inconnu".to_string());
                let os = previous
                    .as_ref()
                    .filter(|presence| presence.os != "inconnu")
                    .map(|presence| presence.os.clone())
                    .or(os)
                    .unwrap_or_else(|| "inconnu".to_string());
                let reported_transport = non_empty_registration_value(transport);
                let channel = match channel {
                    ChannelReport::Known(value) => non_empty_registration_value(Some(value)),
                    ChannelReport::Unknown => None,
                    ChannelReport::Omitted => match mode {
                        Some(PresenceMode::Tmux) => {
                            reported_transport.clone().filter(|value| value != "tmux")
                        }
                        _ => reported_transport
                            .clone()
                            .filter(|value| legacy_non_protocol_transport(value)),
                    }
                    .or_else(|| {
                        previous
                            .as_ref()
                            .and_then(|presence| presence.channel.clone())
                    }),
                };
                let transport = managed_transport.unwrap_or_else(|| match mode {
                    Some(PresenceMode::Tmux) => "tmux".to_string(),
                    Some(PresenceMode::Acp) => reported_transport
                        .clone()
                        .filter(|value| !legacy_non_protocol_transport(value))
                        .or_else(|| previous.as_ref().map(|presence| presence.transport.clone()))
                        .unwrap_or_else(|| "acp".to_string()),
                    Some(PresenceMode::Cli) => reported_transport
                        .filter(|value| !legacy_non_protocol_transport(value))
                        .or_else(|| previous.as_ref().map(|presence| presence.transport.clone()))
                        .unwrap_or_else(|| "cli".to_string()),
                    _ => reported_transport
                        .filter(|value| !legacy_non_protocol_transport(value))
                        .or_else(|| previous.as_ref().map(|presence| presence.transport.clone()))
                        .unwrap_or_else(|| "unknown".to_string()),
                });
                let agent_type = previous
                    .as_ref()
                    .filter(|presence| !matches!(presence.agent_type.as_str(), "mcp" | "cli"))
                    .map(|presence| presence.agent_type.clone())
                    .unwrap_or_else(|| parsed_type.to_string());

                state
                    .conn_instances
                    .insert(conn_id.to_string(), instance_id.clone());
                if state.managed_by_instance.contains_key(&instance_id) {
                    let _ = state
                        .fleet
                        .set_desired_domain(&final_name, derived_domain.as_deref());
                }
                // Les wrappers récents réinitialisent explicitement ce fait à
                // `false` puis annoncent `JournalReady`. Les binaires
                // historiques ne portent pas ce champ, mais leur chemin ACP
                // n'atteint la boucle qu'après activation du journal : les
                // conserver attachables évite de les casser lors d'un
                // redémarrage progressif du daemon.
                let journal_available = journal_available.unwrap_or_else(|| {
                    previous
                        .as_ref()
                        .is_some_and(|presence| presence.journal_available)
                        || transport == "acp"
                });
                // Migration des remises orphelines : un nouvel instance_id pour
                // le même nom reprend les `dispatching` de l'ancienne instance
                // avant de la retirer, sinon elles restent invisibles jusqu'à
                // expiration (canal latéral muet après respawn).
                if let Some(old_instance_id) = previous_instance_key
                    .as_ref()
                    .filter(|old| old.as_str() != instance_id.as_str())
                {
                    match state.idempotency.reassign_dispatching_deliveries(
                        old_instance_id,
                        &instance_id,
                        unix_now_secs(),
                    ) {
                        Ok(migrated) if migrated > 0 => info!(
                            "reprises idempotentes migrées : {} remise(s) {} → {}",
                            migrated, old_instance_id, instance_id
                        ),
                        Ok(_) => {}
                        Err(error) => error!(
                            "migration des remises {} → {} : {error}",
                            old_instance_id, instance_id
                        ),
                    }
                    state.presences.remove(old_instance_id);
                }
                state.presences.insert(
                    instance_id.clone(),
                    Presence {
                        name: final_name.clone(),
                        agent_type,
                        host,
                        transport,
                        channel,
                        mode,
                        location,
                        journal_available,
                        os,
                        state: if turn_in_progress {
                            "busy"
                        } else {
                            "connected"
                        }
                        .to_string(),
                        busy_since: if turn_in_progress {
                            Some(Instant::now())
                        } else {
                            None
                        },
                        capacity_seen: Instant::now(),
                        link_seen: Instant::now(),
                        reconnect_count,
                        model,
                        effort,
                        rate_limits,
                        served_model,
                        domain: derived_domain.clone(),
                        derived_domain,
                        dnd_until,
                    },
                );
                schedule_idempotent_delivery_recovery(state, conn_id, &instance_id);
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

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
/// Simule les enregistrements antérieurs à FR-2407, donc sans canal explicite.
fn handle_register(
    conn_id: &str,
    agent_type: String,
    name: Option<String>,
    host: Option<String>,
    transport: Option<String>,
    mode: Option<PresenceMode>,
    location: Option<String>,
    os: Option<String>,
    instance_id: Option<String>,
    domain: Option<String>,
    turn_in_progress: bool,
    journal_available: Option<bool>,
    state: &mut DaemonState,
) -> DaemonToWrapper {
    handle_register_with_channel(
        conn_id,
        agent_type,
        name,
        host,
        transport,
        ChannelReport::Omitted,
        mode,
        location,
        os,
        instance_id,
        domain,
        turn_in_progress,
        journal_available,
        state,
    )
}

/// Longueur maximale acceptée pour un identifiant de modèle ou un niveau
/// d'effort, alignée sur la validation des noms d'agent côté CLI.
const MAX_RUNTIME_VALUE_LENGTH: usize = 100;

/// Rejette une valeur trop longue ou porteuse de caractères de contrôle / format
/// (bidi), qui casserait l'alignement de l'annuaire ou mentirait à l'affichage.
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
    if value.chars().any(bridget_core::is_disallowed_control) {
        return Err("valeur contenant des caractères de contrôle".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod validate_runtime_value_tests {
    use super::validate_runtime_value;

    /// Contrôle positif Cc ; puis bidi réel. Aveuglement Cc prouvé sans appeler
    /// le prédicat sous test pour construire l'attente.
    #[test]
    fn oracle_runtime_refuse_une_valeur_bidi_reelle() {
        assert!(
            validate_runtime_value("claude\u{0007}").is_err(),
            "PROMESSE — un Cc (BEL) DOIT être refusé"
        );
        assert!(
            validate_runtime_value("claude-opus").is_ok(),
            "PROMESSE — une valeur légitime DOIT passer"
        );
        let bidi = "claude\u{202e}edualc";
        assert!(
            !bidi.chars().any(char::is_control),
            "preuve d'aveuglement Cc : sans is_format_character cette chaîne passerait"
        );
        let err = validate_runtime_value(bidi).expect_err("oracle — U+202E doit être refusé");
        assert!(
            err.contains("contrôle"),
            "motif de refus attendu, reçu: {err}"
        );
        assert!(
            validate_runtime_value("high\u{2069}").is_err(),
            "oracle — isolat U+2069 DOIT être refusé"
        );
    }
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
    presence.touch_capacity();

    DaemonToWrapper::Ack {
        id: "runtime".to_string(),
    }
}

/// Enregistre le modèle servi par le flux, sans toucher au modèle épinglé ni
/// au routage. Un flux muet n'appelle jamais cette fonction : l'écart reste
/// alors non-verdict.
fn handle_served_model(agent: &str, model: String, state: &mut DaemonState) -> DaemonToWrapper {
    if let Err(reason) = validate_runtime_value(&model) {
        return DaemonToWrapper::Nack {
            id: "served-model".to_string(),
            reason: format!("modèle servi invalide: {reason}"),
        };
    }
    let Some(presence) = presence_of_agent(state, agent) else {
        return DaemonToWrapper::Nack {
            id: "served-model".to_string(),
            reason: format!("agent introuvable: {agent}"),
        };
    };
    presence.served_model = Some(model);
    presence.touch_capacity();
    DaemonToWrapper::Ack {
        id: "served-model".to_string(),
    }
}

/// Enregistre un fait de limite fournisseur sans changer l'état de l'agent.
/// Les champs sont validés comme les valeurs runtime : ils resteront affichés
/// dans `who`, donc aucun contrôle ni valeur démesurée ne traverse la frontière.
fn handle_rate_limit(
    agent: &str,
    window: String,
    status: String,
    resets_at: Option<i64>,
    used_percent: Option<u8>,
    source: bridget_transport::protocol::RateLimitSource,
    state: &mut DaemonState,
) -> DaemonToWrapper {
    if let Err(reason) = validate_runtime_value(&window) {
        return DaemonToWrapper::Nack {
            id: "rate-limit".to_string(),
            reason: format!("fenêtre de limite invalide: {reason}"),
        };
    }
    if let Err(reason) = validate_runtime_value(&status) {
        return DaemonToWrapper::Nack {
            id: "rate-limit".to_string(),
            reason: format!("statut de limite invalide: {reason}"),
        };
    }
    if resets_at.is_some_and(|timestamp| timestamp <= 0) {
        return DaemonToWrapper::Nack {
            id: "rate-limit".to_string(),
            reason: "instant de retour invalide".to_string(),
        };
    }
    if used_percent.is_some_and(|pct| pct > 100) {
        return DaemonToWrapper::Nack {
            id: "rate-limit".to_string(),
            reason: "pourcentage de limite invalide".to_string(),
        };
    }
    let Some(presence) = presence_of_agent(state, agent) else {
        return DaemonToWrapper::Nack {
            id: "rate-limit".to_string(),
            reason: format!("agent introuvable: {agent}"),
        };
    };
    // Upsert par fenêtre : un fait five_hour ne doit pas effacer seven_day.
    presence.rate_limits.insert(
        window.clone(),
        bridget_transport::protocol::RateLimitFact {
            window,
            status,
            resets_at,
            used_percent,
        },
    );
    presence.touch_capacity();
    log::debug!("limite de '{}' mise à jour par {}", presence.name, source);
    DaemonToWrapper::Ack {
        id: "rate-limit".to_string(),
    }
}

/// Enregistre un échantillon de consommation dans le ledger horodaté.
/// L'agent doit être présent (géré vivant) ; sans source côté pilote, aucun
/// message n'arrive ici — le greffe rendra « inconnu », jamais zéro.
fn handle_usage(
    agent: &str,
    tokens: bridget_transport::protocol::UsageTokens,
    source: bridget_transport::protocol::UsageSource,
    state: &mut DaemonState,
) -> DaemonToWrapper {
    let observed_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    if observed_at <= 0 {
        return DaemonToWrapper::Nack {
            id: "usage".to_string(),
            reason: "horloge indisponible".to_string(),
        };
    }
    if presence_of_agent(state, agent).is_none() {
        return DaemonToWrapper::Nack {
            id: "usage".to_string(),
            reason: format!("agent introuvable: {agent}"),
        };
    }
    if let Err(error) =
        state
            .store
            .record_usage_sample(agent, observed_at, tokens, &source.to_string())
    {
        return DaemonToWrapper::Nack {
            id: "usage".to_string(),
            reason: format!("ledger usage: {error}"),
        };
    }
    if let Some(presence) = presence_of_agent(state, agent) {
        presence.touch_capacity();
        log::debug!("usage de '{}' enregistré par {}", presence.name, source);
    }
    DaemonToWrapper::Ack {
        id: "usage".to_string(),
    }
}

fn handle_usage_window(
    agent: &str,
    from_secs: i64,
    to_secs: i64,
    state: &DaemonState,
) -> DaemonToWrapper {
    if agent.trim().is_empty() || from_secs <= 0 || to_secs < from_secs {
        return DaemonToWrapper::Nack {
            id: "usage-window".to_string(),
            reason: "fenêtre d'usage invalide".to_string(),
        };
    }
    match state
        .store
        .aggregate_usage_window(agent, from_secs, to_secs)
    {
        Ok(aggregate) => DaemonToWrapper::UsageWindowResult {
            agent: agent.to_string(),
            aggregate,
        },
        Err(error) => DaemonToWrapper::Nack {
            id: "usage-window".to_string(),
            reason: format!("lecture usage: {error}"),
        },
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
        if claim_timeout(&mut state.store, &msg_id) {
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
fn claim_timeout(store: &mut Store, id: &str) -> bool {
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

/// Échéance de tour lue à CHAUD : `agents.json` courant, sinon registre en
/// mémoire, sinon défaut. Contourne la définition figée au spawn — sans
/// exiger de relancer les agents déjà connectés.
fn live_notify_timeout_secs(fallback: &AgentRegistry, agent_type: &str) -> u64 {
    let from = |registry: &AgentRegistry| -> u64 {
        if agent_type.is_empty() {
            return DEFAULT_NOTIFY_TIMEOUT_SECS;
        }
        registry
            .get(agent_type)
            .map(|definition| definition.notify_timeout_secs)
            .unwrap_or(DEFAULT_NOTIFY_TIMEOUT_SECS)
    };
    match AgentRegistry::load() {
        Ok(live) => from(&live),
        Err(_) => from(fallback),
    }
}

/// Pose `deadline_at` absolue pour un mandat reply=false au moment de la
/// POUSSÉE vers le wrapper. Point unique : couvre Deliver classique ET
/// DeliverIdempotent (Maicie / reprise après redémarrage). Sans cela, le
/// worker retombe sur `notify_timeout` figé dans fleet.json — encore 600 s
/// pour les codex absents de agents.json.
fn stamp_turn_deadline_for_delivery(
    message: &mut bridget_core::BridgetMessage,
    registry: &AgentRegistry,
    agent_type: &str,
) {
    if message.reply {
        if message.deadline_at.is_none() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            message.deadline_at = Some(now.saturating_add(message.reply_timeout.unwrap_or(60)));
        }
        return;
    }
    if message.deadline_at.is_some() {
        return;
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let timeout_secs = live_notify_timeout_secs(registry, agent_type);
    message.deadline_at = Some(now.saturating_add(timeout_secs));
    info!(
        "échéance de tour posée: to={} type={} timeout_secs={} deadline_at={}",
        message.to,
        if agent_type.is_empty() {
            "?"
        } else {
            agent_type
        },
        timeout_secs,
        message.deadline_at.unwrap_or(0)
    );
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
        LookupResult::Rejected {
            category,
            reason,
            expires_at,
        } => Ok(IdempotencyIssue::Rejected {
            category,
            reason,
            expires_at,
        }),
        LookupResult::OutcomeUnknown { expires_at } => Ok(IdempotencyIssue::OutcomeUnknown {
            expires_at,
            delivery_id: st
                .idempotency
                .send_delivery(key)
                .map_err(|error| error.to_string())?
                .map(|delivery| delivery.delivery_id),
        }),
        LookupResult::Orphaned { expires_at, reason } => Ok(IdempotencyIssue::Orphaned {
            expires_at,
            delivery_id: st
                .idempotency
                .orphaned_delivery_id(key)
                .map_err(|error| error.to_string())?
                .unwrap_or_default(),
            reason,
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
        return DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::NegotiationRequired,
        };
    };
    if operation_kind != "send" {
        return DaemonToWrapper::Nack {
            id: idempotency_key,
            reason: "opération idempotente inconnue".to_string(),
        };
    }
    let key = match IdempotencyKey::new(
        negotiated.issuer_scope.clone(),
        OperationKind::Send,
        idempotency_key,
    ) {
        Ok(key) => key,
        Err(error) => {
            return DaemonToWrapper::Nack {
                id: "lookup".to_string(),
                reason: error.to_string(),
            };
        }
    };
    match st.idempotency.lookup(&key, unix_now_secs()) {
        Ok(result) => match replay_issue(st, &key, result) {
            Ok(issue) => issue_response(&key, issue),
            Err(error) => DaemonToWrapper::Nack {
                id: key.idempotency_key,
                reason: error,
            },
        },
        Err(error) => DaemonToWrapper::Nack {
            id: key.idempotency_key,
            reason: error.to_string(),
        },
    }
}

fn handle_delivery_ack(
    conn_id: &str,
    delivery_id: String,
    delivery_generation: u64,
    st: &mut DaemonState,
) -> Option<DaemonToWrapper> {
    let Some(instance_id) = st.conn_instances.get(conn_id).cloned() else {
        return Some(DaemonToWrapper::Nack {
            id: delivery_id,
            reason: "accusé idempotent émis par une instance inconnue".to_string(),
        });
    };
    match st
        .idempotency
        .acknowledge_send_delivery(&delivery_id, &instance_id, delivery_generation)
    {
        Ok(answered_request) => {
            if let Some(request_id) = answered_request {
                st.pending_replies
                    .retain(|pending| pending.msg_id != request_id);
                info!("demande {} répondue après accusé idempotent", request_id);
            }
            #[cfg(feature = "test-support")]
            crate::test_sync::checkpoint("after_delivery_acked");
            None
        }
        Err(error) => Some(DaemonToWrapper::Nack {
            id: delivery_id,
            reason: error.to_string(),
        }),
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
    let expires_at = match st.idempotency.lookup(key, unix_now_secs()) {
        Ok(LookupResult::Rejected { expires_at, .. }) => expires_at,
        Ok(_) => return Err("refus idempotent non terminal".to_string()),
        Err(error) => return Err(error.to_string()),
    };
    Ok(issue_response(
        key,
        IdempotencyIssue::Rejected {
            category,
            reason,
            expires_at,
        },
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
    controls: &mut Vec<DeferredControl>,
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
        message.deadline_at = Some(issued_at.saturating_add(timeout as i64).max(0) as u64);
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
        Reservation::Replayed(LookupResult::OutcomeUnknown { expires_at }) => {
            match st.idempotency.prepared_expiry(&key, now) {
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
            }
        }
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
        Reservation::EnvelopeMismatch => {
            return issue_response(&key, IdempotencyIssue::EnvelopeMismatch);
        }
        Reservation::IdempotencyExpired => {
            return issue_response(&key, IdempotencyIssue::IdempotencyExpired);
        }
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
            return reject_idempotent_send(st, &key, category, reason).unwrap_or_else(|error| {
                DaemonToWrapper::Nack {
                    id: key.idempotency_key.clone(),
                    reason: error,
                }
            });
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
        message_bytes: match serde_json::to_vec(&message) {
            Ok(message_bytes) => message_bytes,
            Err(error) => {
                return DaemonToWrapper::Nack {
                    id: key.idempotency_key.clone(),
                    reason: format!("impossible de sérialiser la remise: {error}"),
                };
            }
        },
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
    if let Err(error) =
        defer_idempotent_delivery(st, &prepared.target_conn, delivery.clone(), controls)
    {
        error!("remise idempotente préparée mais non sérialisable: {error}");
    }
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
    if !state
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .connection_roles
        .contains_key(conn_id)
    {
        match &msg {
            // Un hello client ne doit jamais sélectionner implicitement le rôle
            // wrapper : l'ordre public est strict et sans effet de bord.
            WrapperToDaemon::ClientHello { .. } => {
                return Some(DaemonToWrapper::ClientRejected {
                    reason: ClientRefusal::RoleHandshakeRequired,
                });
            }
            // Le nom réservé `maicie` ne donne aucun droit : seule une
            // connexion explicitement négociée comme service peut le demander.
            WrapperToDaemon::ServiceHello { .. } => {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::RoleHandshakeRequired,
                });
            }
            _ => {}
        }
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
            match role {
                ConnectionRole::Client => {
                    return Some(DaemonToWrapper::ClientRejected {
                        reason: ClientRefusal::ClientRoleRequired,
                    });
                }
                ConnectionRole::Service => {
                    return Some(DaemonToWrapper::ServiceRejected {
                        reason: ServiceRefusal::ServiceRoleRequired,
                    });
                }
                ConnectionRole::Wrapper | ConnectionRole::Attach => {}
            }
            return Some(DaemonToWrapper::AttachRejected {
                subscription_id: None,
                reason: AttachRefusal::MessageOutsideAttachRole,
                mode: None,
                location: None,
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
            mode: None,
            location: None,
        });
    }

    let service_refusal = {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        let is_service_message = matches!(
            &msg,
            WrapperToDaemon::ServiceHello { .. }
                | WrapperToDaemon::CoordinationSubscribe { .. }
                | WrapperToDaemon::GuichetClaimNext { .. }
                | WrapperToDaemon::GuichetClaim { .. }
                | WrapperToDaemon::GuichetLookup { .. }
                | WrapperToDaemon::GuichetReply { .. }
        );
        match st.connection_roles.get(conn_id) {
            Some(ConnectionRole::Service) => match &msg {
                WrapperToDaemon::ServiceHello { .. }
                    if st.service_negotiations.contains_key(conn_id) =>
                {
                    Some(ServiceRefusal::AlreadyNegotiated)
                }
                WrapperToDaemon::GuichetClaimNext { .. }
                | WrapperToDaemon::GuichetClaim { .. }
                | WrapperToDaemon::GuichetLookup { .. }
                | WrapperToDaemon::GuichetReply { .. }
                | WrapperToDaemon::CoordinationSubscribe { .. }
                    if !st.service_negotiations.contains_key(conn_id) =>
                {
                    Some(ServiceRefusal::NegotiationRequired)
                }
                WrapperToDaemon::GuichetClaimNext { .. }
                | WrapperToDaemon::GuichetClaim { .. }
                | WrapperToDaemon::GuichetLookup { .. }
                | WrapperToDaemon::GuichetReply { .. }
                    if !st
                        .service_negotiations
                        .get(conn_id)
                        .is_some_and(|negotiated| {
                            negotiated.version == SERVICE_CONTRACT_VERSION
                                && negotiated
                                    .capabilities
                                    .contains(&ServiceCapability::MaicieGuichet)
                        }) =>
                {
                    Some(ServiceRefusal::CapabilityRequired)
                }
                WrapperToDaemon::CoordinationSubscribe { .. }
                    if !st
                        .service_negotiations
                        .get(conn_id)
                        .is_some_and(|negotiated| {
                            negotiated.version == SERVICE_CONTRACT_VERSION
                                && negotiated
                                    .capabilities
                                    .contains(&ServiceCapability::CoordinationEventsV2)
                        }) =>
                {
                    Some(ServiceRefusal::CapabilityRequired)
                }
                WrapperToDaemon::ServiceHello { .. }
                | WrapperToDaemon::CoordinationSubscribe { .. }
                | WrapperToDaemon::GuichetClaimNext { .. }
                | WrapperToDaemon::GuichetClaim { .. }
                | WrapperToDaemon::GuichetLookup { .. }
                | WrapperToDaemon::GuichetReply { .. }
                | WrapperToDaemon::Heartbeat => None,
                _ => Some(ServiceRefusal::MessageOutsideServiceRole),
            },
            Some(ConnectionRole::Wrapper) | None if is_service_message => {
                Some(ServiceRefusal::ServiceRoleRequired)
            }
            _ => None,
        }
    };
    if let Some(reason) = service_refusal {
        return Some(DaemonToWrapper::ServiceRejected { reason });
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
                // MATRICE EXHAUSTIVE — aucun `_`, et c'est délibéré.
                //
                // Le tiret bas précédent classait trois variantes et renvoyait
                // les quarante-deux autres au refus, EN SILENCE. Ajouter une
                // variante au protocole compilait sans rien dire, et le message
                // neuf était rejeté en production sans qu'aucun test unitaire ne
                // puisse le voir : c'est ainsi que `DaemonIdentityRequest` a été
                // livré inatteignable. Désormais, ajouter une variante NE COMPILE
                // PAS tant qu'elle n'est pas classée ici — le compilateur pose la
                // question à la place du relecteur.
                WrapperToDaemon::ClientHello { .. }
                | WrapperToDaemon::SendIdempotent { .. }
                | WrapperToDaemon::Lookup { .. }
                // Sonde d'identité : lecture seule, aucune écriture durable, et
                // c'est le rôle Client qui l'emprunte (`daemon_identity`).
                | WrapperToDaemon::DaemonIdentityRequest => None,
                WrapperToDaemon::RoleHandshake { .. }
                | WrapperToDaemon::ServiceHello { .. }
                | WrapperToDaemon::CoordinationSubscribe { .. }
                | WrapperToDaemon::ServiceRequest { .. }
                | WrapperToDaemon::GuichetClaimNext { .. }
                | WrapperToDaemon::GuichetClaim { .. }
                | WrapperToDaemon::GuichetLookup { .. }
                | WrapperToDaemon::GuichetReply { .. }
                | WrapperToDaemon::DeliverAcked { .. }
                | WrapperToDaemon::DeliveryIndeterminate { .. }
                | WrapperToDaemon::SpawnOrder { .. }
                | WrapperToDaemon::StopOrder { .. }
                | WrapperToDaemon::Subscribe { .. }
                | WrapperToDaemon::Unsubscribe { .. }
                | WrapperToDaemon::Subscribed { .. }
                | WrapperToDaemon::JournalFragment { .. }
                | WrapperToDaemon::LiveJournalFragment { .. }
                | WrapperToDaemon::SnapshotCaughtUp { .. }
                | WrapperToDaemon::Gap { .. }
                | WrapperToDaemon::JournalReadError { .. }
                | WrapperToDaemon::End { .. }
                | WrapperToDaemon::AttachRejected { .. }
                | WrapperToDaemon::Register { .. }
                | WrapperToDaemon::JournalReady
                | WrapperToDaemon::Unregister
                | WrapperToDaemon::Rename { .. }
                | WrapperToDaemon::Send { .. }
                | WrapperToDaemon::DeliveryRejected { .. }
                | WrapperToDaemon::TurnState { .. }
                | WrapperToDaemon::CancelRequest { .. }
                | WrapperToDaemon::ListRequests { .. }
                | WrapperToDaemon::LedgerProjection { .. }
                | WrapperToDaemon::Heartbeat
                | WrapperToDaemon::ListAgents
                | WrapperToDaemon::Runtime { .. }
                | WrapperToDaemon::ServedModel { .. }
                | WrapperToDaemon::RateLimit { .. }
                | WrapperToDaemon::Usage { .. }
                | WrapperToDaemon::UsageWindow { .. }
                | WrapperToDaemon::Domain { .. }
                | WrapperToDaemon::Availability { .. } => {
                    Some(ClientRefusal::MessageOutsideClientRole)
                }
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
        WrapperToDaemon::ServiceHello {
            version,
            service,
            issuer_scope,
            capabilities,
        } => {
            if version != SERVICE_CONTRACT_VERSION {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::UnsupportedVersion {
                        supported_versions: vec![SERVICE_CONTRACT_VERSION],
                    },
                });
            }
            if service != "maicie" {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::ReservedServiceRequired,
                });
            }
            if crate::idempotency::validate_issuer_scope(&issuer_scope).is_err() {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::InvalidIssuerScope,
                });
            }
            let canonical_capabilities = matches!(
                capabilities.as_slice(),
                [] | [ServiceCapability::MaicieGuichet]
                    | [
                        ServiceCapability::MaicieGuichet,
                        ServiceCapability::CoordinationEventsV1,
                    ]
                    | [
                        ServiceCapability::MaicieGuichet,
                        ServiceCapability::CoordinationEventsV2,
                    ]
            );
            if !canonical_capabilities {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::InvalidEnvelope,
                });
            }
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            st.service_negotiations.insert(
                conn_id.to_string(),
                NegotiatedService {
                    version: SERVICE_CONTRACT_VERSION,
                    capabilities: capabilities.clone(),
                },
            );
            if capabilities.contains(&ServiceCapability::MaicieGuichet) {
                match st.store.guichet_lifecycle_events() {
                    Ok(events) => {
                        let controls = events
                            .into_iter()
                            .filter_map(|event| {
                                st.connections.get(conn_id).map(|writer| DeferredControl {
                                    writer: writer.clone(),
                                    message: guichet_lifecycle_response(event),
                                })
                            })
                            .collect::<Vec<_>>();
                        if !controls.is_empty() {
                            st.pending_post_response_controls
                                .insert(conn_id.to_string(), controls);
                        }
                    }
                    Err(error) => error!("lecture des événements guichet: {error}"),
                }
            }
            // La v1 conserve son rejeu initial historique. La v2 ne pousse
            // rien avant `coordination_subscribe`, sinon un fait live pourrait
            // être confondu avec une observation fraîche avant le snapshot.
            if capabilities.contains(&ServiceCapability::CoordinationEventsV1) {
                match st.store.guichet_coordination_events() {
                    Ok(events) => {
                        let controls = events
                            .into_iter()
                            .filter_map(|event| {
                                st.connections.get(conn_id).map(|writer| DeferredControl {
                                    writer: writer.clone(),
                                    message: guichet_coordination_response(
                                        event,
                                        COORDINATION_EVENTS_VERSION,
                                    ),
                                })
                            })
                            .collect::<Vec<_>>();
                        if !controls.is_empty() {
                            st.pending_post_response_controls
                                .entry(conn_id.to_string())
                                .or_default()
                                .extend(controls);
                        }
                    }
                    Err(error) => error!("lecture des événements de coordination: {error}"),
                }
            }
            Some(DaemonToWrapper::ServiceWelcome {
                version: SERVICE_CONTRACT_VERSION,
                horizon_secs: CLIENT_IDEMPOTENCY_HORIZON_SECS,
                issued_at_tolerance_secs: CLIENT_ISSUED_AT_TOLERANCE_SECS,
                capabilities,
            })
        }
        WrapperToDaemon::CoordinationSubscribe {
            version,
            after_cursor,
        } => {
            if version != COORDINATION_STREAM_VERSION {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::UnsupportedVersion {
                        supported_versions: vec![COORDINATION_STREAM_VERSION],
                    },
                });
            }
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            let replay = match st.store.guichet_coordination_events_after(after_cursor) {
                Ok(replay) => replay,
                Err(error) => {
                    error!("relève de coordination indisponible: {error}");
                    st.coordination_subscriptions.insert(
                        conn_id.to_string(),
                        CoordinationSubscription {
                            fresh: false,
                            next_cursor: after_cursor.unwrap_or(0).saturating_add(1),
                        },
                    );
                    if let Some(writer) = st.connections.get(conn_id).cloned() {
                        st.pending_post_response_controls
                            .entry(conn_id.to_string())
                            .or_default()
                            .push(DeferredControl {
                                writer,
                                message: DaemonToWrapper::CoordinationUnavailable {
                                    version,
                                    reason: "source_unavailable".to_string(),
                                },
                            });
                    }
                    return None;
                }
            };
            let mut controls = Vec::new();
            let next_cursor = replay
                .through_cursor
                .or(after_cursor)
                .unwrap_or(0)
                .saturating_add(1);
            let fresh = replay.gap.is_none();
            if let Some((from_cursor, to_cursor)) = replay.gap {
                defer_control(
                    &st,
                    conn_id,
                    DaemonToWrapper::CoordinationGap {
                        version,
                        from_cursor,
                        to_cursor,
                        reason: "cursor_gap".to_string(),
                    },
                    &mut controls,
                );
            } else {
                for event in replay.events {
                    defer_control(
                        &st,
                        conn_id,
                        guichet_coordination_response(event, version),
                        &mut controls,
                    );
                }
                defer_control(
                    &st,
                    conn_id,
                    DaemonToWrapper::CoordinationSnapshotCaughtUp {
                        version,
                        through_cursor: replay.through_cursor,
                    },
                    &mut controls,
                );
            }
            st.coordination_subscriptions.insert(
                conn_id.to_string(),
                CoordinationSubscription { fresh, next_cursor },
            );
            if !controls.is_empty() {
                st.pending_post_response_controls
                    .entry(conn_id.to_string())
                    .or_default()
                    .extend(controls);
            }
            None
        }
        WrapperToDaemon::ServiceRequest {
            version,
            issuer_scope,
            request_id,
            issued_at,
            from,
            to,
            operation,
            payload,
        } => {
            if version != SERVICE_CONTRACT_VERSION {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::UnsupportedVersion {
                        supported_versions: vec![SERVICE_CONTRACT_VERSION],
                    },
                });
            }
            if to != "maicie" {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::ReservedTargetRequired,
                });
            }
            if !guichet_request_is_valid(&request_id, operation, &payload)
                || crate::idempotency::validate_issuer_scope(&issuer_scope).is_err()
            {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::InvalidEnvelope,
                });
            }
            let (declared_name, declared_instance_id, declared_sender_matches) = {
                let st = state.lock().unwrap_or_else(|e| e.into_inner());
                let name = st.conn_names.get(conn_id).cloned();
                let instance_id = st.conn_instances.get(conn_id).cloned();
                let matches = name.as_ref().is_some_and(|registered| {
                    registered == &from
                        || (registered.starts_with("cli-send-")
                            && st.router.get_agent(&from).is_some())
                });
                (name, instance_id, matches)
            };
            let mutation_action = greffe_mutation_action(operation);
            if !declared_sender_matches {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: if mutation_action.is_some() {
                        ServiceRefusal::GreffeAuthorizationDenied
                    } else {
                        ServiceRefusal::DeclaredSenderMismatch
                    },
                });
            }
            let canonical = match encode(&WrapperToDaemon::ServiceRequest {
                version,
                issuer_scope: issuer_scope.clone(),
                request_id: request_id.clone(),
                issued_at,
                from: from.clone(),
                to,
                operation,
                payload: payload.clone(),
            }) {
                Ok(bytes) => bytes.into_bytes(),
                Err(_) => {
                    return Some(DaemonToWrapper::ServiceRejected {
                        reason: ServiceRefusal::InvalidEnvelope,
                    });
                }
            };
            let now = unix_timestamp();
            let mut deposit = GuichetDeposit {
                issuer_scope: issuer_scope.clone(),
                request_id: request_id.clone(),
                issued_at,
                from,
                operation,
                payload,
                canonical_bytes: canonical,
                authorization_attestation: None,
            };
            let authorization_declared_from = deposit.from.clone();
            let authorization_request_id = deposit.request_id.clone();
            let authorization_canonical_request = deposit.canonical_bytes.clone();
            let deposit_result = match mutation_action {
                Some(action) => {
                    // `declared_name` et `declared_instance_id` viennent du
                    // Register de cette connexion. Ils réduisent les sources
                    // d'identité sans authentifier le processus pair.
                    let gate = GreffeAuthorizationGate::from_environment();
                    match gate.authorize_deposit_then(
                        GreffeDepositAuthorization {
                            canonical_name: declared_name.as_deref(),
                            canonical_instance_id: declared_instance_id.as_deref(),
                            declared_from: Some(authorization_declared_from.as_str()),
                            action,
                            issuer_scope: &issuer_scope,
                            request_id: authorization_request_id.as_str(),
                            request_issued_at: deposit.issued_at,
                            canonical_request: &authorization_canonical_request,
                            observed_at: now,
                        },
                        |attestation| {
                            deposit.authorization_attestation = Some(attestation.clone());
                            state
                                .lock()
                                .unwrap_or_else(|error| error.into_inner())
                                .store
                                .deposit_guichet(
                                    &deposit,
                                    CLIENT_IDEMPOTENCY_HORIZON_SECS,
                                    CLIENT_ISSUED_AT_TOLERANCE_SECS,
                                    now,
                                )
                        },
                    ) {
                        Ok(result) => result,
                        Err(refusal) => {
                            warn!(
                                "mutation du greffe refusée au dépôt: {}",
                                refusal.audit_code()
                            );
                            return Some(DaemonToWrapper::ServiceRejected {
                                reason: ServiceRefusal::GreffeAuthorizationDenied,
                            });
                        }
                    }
                }
                None => state
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .store
                    .deposit_guichet(
                        &deposit,
                        CLIENT_IDEMPOTENCY_HORIZON_SECS,
                        CLIENT_ISSUED_AT_TOLERANCE_SECS,
                        now,
                    ),
            };
            match deposit_result {
                Ok(result) => Some(guichet_result_response(issuer_scope, request_id, result)),
                Err(StoreError::FrameTooLarge { .. }) => Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::FrameTooLarge,
                }),
                Err(error) => {
                    error!("dépôt guichet: {error}");
                    Some(DaemonToWrapper::ServiceRejected {
                        reason: ServiceRefusal::TransitionInvalid,
                    })
                }
            }
        }
        WrapperToDaemon::GuichetClaimNext { version } => {
            if version != SERVICE_CONTRACT_VERSION {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::UnsupportedVersion {
                        supported_versions: vec![SERVICE_CONTRACT_VERSION],
                    },
                });
            }
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            match st.store.claim_next_guichet(conn_id, unix_timestamp()) {
                Ok(GuichetNext::Claimed(claim)) => Some(guichet_claim_response(claim)),
                Ok(GuichetNext::Empty) => Some(DaemonToWrapper::GuichetEmpty {
                    version: SERVICE_CONTRACT_VERSION,
                }),
                Err(error) => {
                    error!("claim guichet: {error}");
                    Some(DaemonToWrapper::ServiceRejected {
                        reason: ServiceRefusal::TransitionInvalid,
                    })
                }
            }
        }
        WrapperToDaemon::GuichetClaim {
            version,
            issuer_scope,
            request_id,
            claim_token,
        } => {
            if version != SERVICE_CONTRACT_VERSION {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::InvalidEnvelope,
                });
            }
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            match st.store.claim_guichet(
                conn_id,
                &issuer_scope,
                &request_id,
                &claim_token,
                unix_timestamp(),
            ) {
                Ok(Ok(claim)) => Some(guichet_claim_response(claim)),
                Ok(Err(result)) => Some(guichet_result_response(issuer_scope, request_id, result)),
                Err(error) => {
                    error!("rejeu claim guichet: {error}");
                    Some(DaemonToWrapper::ServiceRejected {
                        reason: ServiceRefusal::TransitionInvalid,
                    })
                }
            }
        }
        WrapperToDaemon::GuichetLookup {
            version,
            issuer_scope,
            request_id,
        } => {
            if version != SERVICE_CONTRACT_VERSION {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::InvalidEnvelope,
                });
            }
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            match st
                .store
                .lookup_guichet(&issuer_scope, &request_id, unix_timestamp())
            {
                Ok(result) => Some(guichet_result_response(issuer_scope, request_id, result)),
                Err(error) => {
                    error!("lookup guichet: {error}");
                    Some(DaemonToWrapper::ServiceRejected {
                        reason: ServiceRefusal::TransitionInvalid,
                    })
                }
            }
        }
        WrapperToDaemon::GuichetReply {
            version,
            issuer_scope,
            request_id,
            claim_generation,
            claim_token,
            response_message_id,
            in_reply_to,
            outcome,
            payload,
        } => {
            if version != SERVICE_CONTRACT_VERSION
                || !guichet_reply_is_valid(
                    &request_id,
                    &response_message_id,
                    &in_reply_to,
                    &payload,
                )
            {
                return Some(DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::InvalidEnvelope,
                });
            }
            let canonical = match encode(&WrapperToDaemon::GuichetReply {
                version,
                issuer_scope: issuer_scope.clone(),
                request_id: request_id.clone(),
                claim_generation,
                claim_token: claim_token.clone(),
                response_message_id: response_message_id.clone(),
                in_reply_to: in_reply_to.clone(),
                outcome,
                payload,
            }) {
                Ok(bytes) => bytes.into_bytes(),
                Err(_) => {
                    return Some(DaemonToWrapper::ServiceRejected {
                        reason: ServiceRefusal::InvalidEnvelope,
                    });
                }
            };
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            match st.store.reply_guichet(
                conn_id,
                GuichetReplyInput {
                    issuer_scope: &issuer_scope,
                    request_id: &request_id,
                    generation: claim_generation,
                    token: &claim_token,
                    response_message_id: &response_message_id,
                    reply_bytes: &canonical,
                    in_reply_to: &in_reply_to,
                    outcome,
                },
                unix_timestamp(),
            ) {
                Ok(result) => {
                    if matches!(&result, GuichetResult::Terminal { issue, newly_finalized: true, .. } if issue == "accepted")
                    {
                        match st.store.guichet_lifecycle_events() {
                            Ok(events) => {
                                if let Some(event) = events.into_iter().find(|event| {
                                    event.issuer_scope == issuer_scope
                                        && event.request_id == request_id
                                }) && let Some(writer) = st.connections.get(conn_id).cloned()
                                {
                                    st.pending_post_response_controls
                                        .entry(conn_id.to_string())
                                        .or_default()
                                        .push(DeferredControl {
                                            writer,
                                            message: guichet_lifecycle_response(event),
                                        });
                                }
                            }
                            Err(error) => error!("lecture événement guichet: {error}"),
                        }
                    }
                    Some(guichet_result_response(issuer_scope, request_id, result))
                }
                Err(error) => {
                    error!("réponse guichet: {error}");
                    Some(DaemonToWrapper::ServiceRejected {
                        reason: ServiceRefusal::TransitionInvalid,
                    })
                }
            }
        }
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
                build_id: crate::build_info::BUILD_ID.to_string(),
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
            let (response, controls) = {
                let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                let mut controls = Vec::new();
                let response = handle_idempotent_send(
                    conn_id,
                    message,
                    message_id,
                    issued_at,
                    &mut st,
                    &mut controls,
                );
                (response, controls)
            };
            let _ = execute_controls(controls);
            Some(response)
        }
        WrapperToDaemon::Lookup {
            operation_kind,
            idempotency_key,
        } => {
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            Some(handle_idempotency_lookup(
                conn_id,
                operation_kind,
                idempotency_key,
                &st,
            ))
        }
        WrapperToDaemon::DeliverAcked {
            delivery_id,
            delivery_generation,
        } => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            handle_delivery_ack(conn_id, delivery_id, delivery_generation, &mut st)
        }
        WrapperToDaemon::DeliveryIndeterminate {
            delivery_id,
            delivery_generation,
        } => {
            let Some(instance_id) = state
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .conn_instances
                .get(conn_id)
                .cloned()
            else {
                return Some(DaemonToWrapper::Nack {
                    id: delivery_id,
                    reason: "accusé indéterminé émis par une instance inconnue".to_string(),
                });
            };
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            match st.idempotency.mark_delivery_indeterminate(
                &delivery_id,
                &instance_id,
                delivery_generation,
            ) {
                Ok(()) => {
                    #[cfg(feature = "test-support")]
                    crate::test_sync::checkpoint("after_delivery_indeterminate");
                    None
                }
                Err(error) => Some(DaemonToWrapper::Nack {
                    id: delivery_id,
                    reason: error.to_string(),
                }),
            }
        }
        WrapperToDaemon::SpawnOrder {
            agent_type,
            name,
            cwd,
            persistent,
            command_id,
            issued_at,
            deadline_at,
        } => {
            let mut st = state.lock().unwrap_or_else(|error| error.into_inner());
            let order = FleetSpawnOrder {
                agent_type,
                requested_name: name,
                cwd: PathBuf::from(cwd),
                persistent,
                command_id: command_id.clone(),
                issued_at,
                deadline_at,
            };
            // Seul endroit du programme où les deux machines coexistent : le
            // daemon cherche le `cwd` chez LUI, le demandeur peut être au bout
            // du tunnel. `conn_hosts` porte l'hôte attesté à l'enregistrement.
            let hosts = crate::lifecycle::SpawnHosts {
                searched_on: st.host.clone(),
                requested_from: st
                    .conn_hosts
                    .get(conn_id)
                    .cloned()
                    .unwrap_or_else(|| crate::build_info::MACHINE_NON_ATTESTEE.to_string()),
            };
            let decision = submit_spawn(
                &st.fleet,
                &st.registry,
                &st.source_env,
                &order,
                unix_timestamp(),
                st.recovering,
                &hosts,
            );
            match decision {
                Ok(SpawnDecision::Ready(prepared)) => {
                    let stop = Arc::new(ManagedStopControl::new());
                    st.managed_by_instance.insert(
                        prepared.lease.instance_id.clone(),
                        prepared.lease.command_id.clone(),
                    );
                    st.managed_spawns.insert(
                        prepared.lease.command_id.clone(),
                        ManagedSpawnRecord {
                            lease: prepared.lease.clone(),
                            agent_type: prepared.agent_type.clone(),
                            requester_conns: vec![conn_id.to_string()],
                            wrapper_conn: None,
                            stop: Arc::clone(&stop),
                        },
                    );
                    if st
                        .managed_tx
                        .send(ManagedSupervisorCommand::Start {
                            prepared: prepared.clone(),
                            stop,
                        })
                        .is_err()
                    {
                        let _ = st.fleet.fail(
                            &prepared.lease,
                            "negotiation_failed",
                            "superviseur de processus indisponible",
                        );
                        st.managed_spawns.remove(&prepared.lease.command_id);
                        st.managed_by_instance.remove(&prepared.lease.instance_id);
                        st.managed_terminal_instances
                            .insert(prepared.lease.instance_id.clone());
                        return Some(DaemonToWrapper::SpawnRejected {
                            command_id,
                            reason: SpawnRefusal::NegotiationFailed {
                                detail: "superviseur de processus indisponible".to_string(),
                            },
                        });
                    }
                    None
                }
                Ok(SpawnDecision::Await(waiter)) => {
                    if let Some(record) = st.managed_spawns.get_mut(&waiter.command_id)
                        && !record.requester_conns.iter().any(|id| id == conn_id)
                    {
                        record.requester_conns.push(conn_id.to_string());
                    }
                    None
                }
                Ok(SpawnDecision::Accepted { name, definition }) => {
                    Some(DaemonToWrapper::SpawnAccepted {
                        command_id,
                        name,
                        definition,
                    })
                }
                Ok(SpawnDecision::Rejected(reason)) => {
                    Some(DaemonToWrapper::SpawnRejected { command_id, reason })
                }
                Ok(SpawnDecision::EnvelopeMismatch) => Some(DaemonToWrapper::IdempotencyResult {
                    operation_kind: "spawn".to_string(),
                    idempotency_key: command_id,
                    issue: IdempotencyIssue::EnvelopeMismatch,
                }),
                Err(error) => Some(DaemonToWrapper::SpawnRejected {
                    command_id,
                    reason: SpawnRefusal::NegotiationFailed {
                        detail: error.to_string(),
                    },
                }),
            }
        }
        WrapperToDaemon::StopOrder { name, command_id } => {
            let target = {
                let mut st = state.lock().unwrap_or_else(|error| error.into_inner());
                let record = st
                    .managed_spawns
                    .values()
                    .find(|spawn| spawn.lease.name == name);
                match record {
                    Some(record) => {
                        let lease = record.lease.clone();
                        let completion = Arc::clone(&record.stop);
                        let wrapper = record
                            .wrapper_conn
                            .as_ref()
                            .and_then(|conn_id| st.connections.get(conn_id))
                            .cloned();
                        if !completion.is_requested()
                            && let Err(error) = st.fleet.invalidate_for_stop(&lease)
                        {
                            ManagedStopTarget::Immediate(StopOutcome::Timeout {
                                state: error.to_string(),
                            })
                        } else {
                            st.managed_terminal_instances
                                .insert(lease.instance_id.clone());
                            if wrapper.is_none() {
                                completion.mark_handshake_complete();
                            }
                            ManagedStopTarget::Supervised {
                                receiver: completion.request(),
                                completion,
                                wrapper,
                            }
                        }
                    }
                    None => ManagedStopTarget::Marker {
                        store: st.marker_store.clone(),
                        fallback: if st.router.get_agent(&name).is_some() {
                            StopOutcome::NotManaged
                        } else {
                            StopOutcome::NotFound
                        },
                    },
                }
            };
            let outcome = match target {
                ManagedStopTarget::Supervised {
                    receiver,
                    completion,
                    wrapper,
                } => {
                    if let Some(writer) = wrapper {
                        let _ = push_control_message(&writer, &DaemonToWrapper::Disconnect);
                    }
                    completion.mark_handshake_complete();
                    receiver.recv_timeout(MANAGED_STOP_REPLY_TIMEOUT).unwrap_or(
                        StopOutcome::Timeout {
                            state: "superviseur sans issue dans le délai".to_string(),
                        },
                    )
                }
                ManagedStopTarget::Marker { store, fallback } => match store.stop_current_group(
                    &name,
                    MANAGED_STOP_FORCED_GRACE,
                    MANAGED_STOP_POLL,
                ) {
                    Ok(Some(ManagedStopResult::Stopped)) => StopOutcome::Stopped,
                    Ok(Some(ManagedStopResult::StoppedForced { survivors_killed })) => {
                        StopOutcome::StoppedForced { survivors_killed }
                    }
                    Ok(Some(ManagedStopResult::Timeout)) => StopOutcome::Timeout {
                        state: "groupe du marqueur encore vivant".to_string(),
                    },
                    Ok(None) => fallback,
                    Err(error) => StopOutcome::Timeout {
                        state: error.to_string(),
                    },
                },
                ManagedStopTarget::Immediate(outcome) => outcome,
            };
            Some(DaemonToWrapper::StopResult {
                command_id,
                outcome,
            })
        }
        WrapperToDaemon::Subscribe { agent, window } => {
            let (subscription_id, control) = {
                let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                if st.connection_roles.get(conn_id) != Some(&ConnectionRole::Attach) {
                    return Some(DaemonToWrapper::AttachRejected {
                        subscription_id: None,
                        reason: AttachRefusal::MessageOutsideAttachRole,
                        mode: None,
                        location: None,
                    });
                }
                let wrapper_conn = match attach_refusal_for_subscription(&st, &agent) {
                    Ok(connection_id) => connection_id,
                    Err(refusal) => {
                        return Some(DaemonToWrapper::AttachRejected {
                            subscription_id: None,
                            reason: refusal.reason,
                            mode: refusal.mode,
                            location: refusal.location,
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
                        mode: None,
                        location: None,
                    });
                };
                let writer = st.connections.get(&wrapper_conn).cloned();
                st.attach_subscriptions.insert(
                    subscription_id.clone(),
                    AttachSubscription {
                        agent: agent.clone(),
                        attach_conn: conn_id.to_string(),
                        wrapper_conn,
                        caught_up: false,
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
                    mode: None,
                    location: None,
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
                        mode: None,
                        location: None,
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
                                mode: None,
                                location: None,
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
        WrapperToDaemon::LiveJournalFragment {
            seq,
            offset,
            final_fragment,
            bytes,
        } => {
            let views = {
                let st = state.lock().unwrap_or_else(|e| e.into_inner());
                st.attach_subscriptions
                    .iter()
                    .filter(|(_, subscription)| {
                        subscription.wrapper_conn == conn_id && subscription.caught_up
                    })
                    .filter_map(|(subscription_id, _)| {
                        st.attach_views
                            .get(subscription_id)
                            .cloned()
                            .map(|view| (subscription_id.clone(), view))
                    })
                    .collect::<Vec<_>>()
            };
            for (subscription_id, view) in views {
                let _ = view.enqueue(DaemonToWrapper::JournalFragment {
                    subscription_id,
                    seq,
                    offset,
                    final_fragment,
                    bytes: bytes.clone(),
                });
            }
            None
        }
        WrapperToDaemon::SnapshotCaughtUp {
            subscription_id,
            through_seq,
        } => {
            let view = {
                let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                let accepted = st
                    .attach_subscriptions
                    .get_mut(&subscription_id)
                    .filter(|subscription| subscription.wrapper_conn == conn_id)
                    .map(|subscription| subscription.caught_up = true)
                    .is_some();
                accepted
                    .then(|| st.attach_views.get(&subscription_id).cloned())
                    .flatten()
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
                let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
                let accepted = st
                    .attach_subscriptions
                    .get_mut(&subscription_id)
                    .filter(|subscription| subscription.wrapper_conn == conn_id)
                    .map(|subscription| {
                        if reason.as_deref() == Some("live_feed_overrun") {
                            subscription.caught_up = false;
                        }
                    })
                    .is_some();
                accepted
                    .then(|| st.attach_views.get(&subscription_id).cloned())
                    .flatten()
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
            channel,
            mode,
            location,
            os,
            instance_id,
            domain,
            turn_in_progress,
            journal_available,
        } => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            if instance_id
                .as_ref()
                .is_some_and(|id| st.managed_terminal_instances.contains(id))
            {
                return Some(DaemonToWrapper::Nack {
                    id: "register".to_string(),
                    reason: "génération gérée déjà terminale".to_string(),
                });
            }
            let managed_instance = instance_id.clone();
            let response = handle_register_with_channel(
                conn_id,
                agent_type,
                name,
                host,
                transport,
                channel,
                mode,
                location,
                os,
                instance_id,
                domain,
                turn_in_progress,
                journal_available,
                &mut st,
            );
            if let (Some(instance_id), DaemonToWrapper::Registered { name: final_name }) =
                (managed_instance, &response)
                && let Some(command_id) = st.managed_by_instance.get(&instance_id).cloned()
                && let Some(record) = st.managed_spawns.get_mut(&command_id)
            {
                record.wrapper_conn = Some(conn_id.to_string());
                let _ = st.managed_tx.send(ManagedSupervisorCommand::Registered {
                    instance_id,
                    conn_id: conn_id.to_string(),
                    name: final_name.clone(),
                });
            }
            Some(response)
        }

        WrapperToDaemon::JournalReady => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            let Some(instance_id) = st.conn_instances.get(conn_id).cloned() else {
                return Some(DaemonToWrapper::Nack {
                    id: "journal-ready".to_string(),
                    reason: "journal annoncé avant l'enregistrement".to_string(),
                });
            };
            let Some(presence) = st.presences.get_mut(&instance_id) else {
                return Some(DaemonToWrapper::Nack {
                    id: "journal-ready".to_string(),
                    reason: "présence introuvable pour le journal annoncé".to_string(),
                });
            };
            presence.journal_available = true;
            None
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
                st.service_negotiations.remove(conn_id);
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

            // Le daemon est l'autorité de l'échéance de TOUR (pas seulement
            // reply=yes) : relire la valeur courante et la pousser en absolu.
            // Sans cela, worker.notify_timeout figé au spawn ignore agents.json.
            let mut delivered_message = bridge_msg.clone();
            let agent_type = st
                .conn_instances
                .get(&target_conn)
                .and_then(|instance_id| st.presences.get(instance_id))
                .map(|presence| presence.agent_type.as_str())
                .unwrap_or("");
            stamp_turn_deadline_for_delivery(&mut delivered_message, &st.registry, agent_type);
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
                                bridge_msg.id, bridge_msg.to, bridge_msg.hops, bridge_msg.reply
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
                // Lien ≠ capacité : le heartbeat prouve le socket, pas l'exécution.
                presence.touch_link();
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

        // Le daemon atteste SA machine et SA base : le client n'a plus à
        // deviner, et n'affiche plus les siennes à leur place.
        WrapperToDaemon::DaemonIdentityRequest => {
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            Some(DaemonToWrapper::DaemonIdentityReport {
                host: st.host.clone(),
                db_path: st.db_path.display().to_string(),
            })
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

        WrapperToDaemon::ServedModel { agent, model } => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            Some(handle_served_model(&agent, model, &mut st))
        }

        WrapperToDaemon::RateLimit {
            agent,
            window,
            status,
            resets_at,
            used_percent,
            source,
        } => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            Some(handle_rate_limit(
                &agent,
                window,
                status,
                resets_at,
                used_percent,
                source,
                &mut st,
            ))
        }

        WrapperToDaemon::Usage {
            agent,
            input_tokens,
            output_tokens,
            cache_creation_input_tokens,
            cache_read_input_tokens,
            source,
        } => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            Some(handle_usage(
                &agent,
                bridget_transport::protocol::UsageTokens {
                    input_tokens,
                    output_tokens,
                    cache_creation_input_tokens,
                    cache_read_input_tokens,
                },
                source,
                &mut st,
            ))
        }

        WrapperToDaemon::UsageWindow {
            agent,
            from_secs,
            to_secs,
        } => {
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            Some(handle_usage_window(&agent, from_secs, to_secs, &st))
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
                    && !claim_timeout(&mut st.store, &id)
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
                            .is_some_and(|presence| presence.mode == Some(PresenceMode::Acp));
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

        WrapperToDaemon::ListRequests { sender, limit } => {
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            match st
                .store
                .requests_for_participant(&sender, usize::from(limit))
            {
                Ok(requests) => match requests
                    .into_iter()
                    .map(|request| -> Result<_, crate::store::StoreError> {
                        let deferred = st.store.latest_deferred_reminder(&request.id)?;
                        Ok(bridget_transport::protocol::RequestInfo {
                            id: request.id,
                            sender: request.sender,
                            target: request.target,
                            state: request.state,
                            created_at: request.created_at,
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

        WrapperToDaemon::LedgerProjection { scope, limit } => {
            let st = state.lock().unwrap_or_else(|error| error.into_inner());
            match crate::ledger::read_projection(&st.store, scope, usize::from(limit)) {
                Ok(projection) => Some(DaemonToWrapper::LedgerProjection {
                    messages: projection.messages,
                    requests: projection.requests,
                }),
                Err(error) => Some(DaemonToWrapper::Nack {
                    id: "ledger".to_string(),
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

    let identity = daemon_identity(&config.socket_path);

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
        channel: ChannelReport::Unknown,
        mode: Some(PresenceMode::Cli),
        location: None,
        os: None,
        instance_id: None,
        domain: None,
        turn_in_progress: false,
        journal_available: None,
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

    // Compter les messages en base — mais SEULEMENT si la base locale est
    // celle du daemon interrogé. Sur une machine fédérée, `config.db_path`
    // désigne un fichier d'ici, pas celui du daemon qui vient de répondre :
    // le compte était lu dans un orphelin local et présenté sous le chemin
    // d'à côté, comme s'il décrivait le daemon.
    let daemon_host = identity.as_ref().and_then(|identity| identity.host.clone());
    let daemon_db_path = identity
        .as_ref()
        .and_then(|identity| identity.db_path.clone());
    let message_count = daemon_store_is_local(
        daemon_host.as_deref(),
        daemon_db_path.as_deref(),
        &crate::build_info::local_host(),
        &config.db_path,
    )
    .then(|| match Store::open(&config.db_path) {
        Ok(store) => store.recent_messages(1000).map(|v| v.len()).unwrap_or(0),
        Err(_) => 0,
    });

    DaemonStatus {
        running: true,
        agents,
        message_count,
        build_id: identity.map(|identity| identity.build_id),
        daemon_host,
        daemon_db_path,
    }
}

const BUILD_ID_PROBE_IDENTITY: &str = "bridget-status-build-id";

fn build_id_probe_issuer_scope() -> String {
    crate::mcp::issuer_scope(BUILD_ID_PROBE_IDENTITY)
}

/// Le compte de messages n'est mesurable d'ici que si la base locale est
/// EXACTEMENT celle que le daemon atteste. Sans attestation, il ne l'est pas :
/// une absence n'autorise pas à compter dans le fichier qu'on a sous la main.
pub(crate) fn daemon_store_is_local(
    daemon_host: Option<&str>,
    daemon_db_path: Option<&str>,
    local_host: &str,
    local_db_path: &std::path::Path,
) -> bool {
    // MÊME CHEMIN N'EST PAS MÊME MACHINE. Deux hôtes Linux portent couramment
    // le même `/home/moi/.cache/bridget/bridget.db` : comparer les chemins seuls
    // déclarait locale une base qui vit ailleurs, et faisait lire le fichier d'à
    // côté comme s'il décrivait le daemon. L'hôte était disponible et écarté de
    // la décision — le système savait nommer la machine, il ne l'attribuait pas.
    //
    // Source UNIQUE : `get_status` et la carte de reprise passent tous deux ici,
    // pour que deux vues du même système ne puissent pas diverger.
    daemon_host.is_some_and(|host| host == local_host)
        && daemon_db_path.is_some_and(|path| std::path::Path::new(path) == local_db_path)
}

#[cfg(test)]
mod attribution_tests {
    use super::daemon_store_is_local;
    use std::path::Path;

    /// POINT 1 — la garde qui empêche de compter dans la base d'à côté.
    ///
    /// Éprouvée sur la décision elle-même, et non à travers `get_status` : un
    /// premier oracle passait par une socket absente, donc `get_status` sortait
    /// AVANT la garde et le test restait vert même sans elle.
    ///
    /// DEUX CONTRÔLES OPPOSÉS SUR LE MÊME CHEMIN, c'est le cœur du test :
    /// même chemin + hôtes distincts doit rendre FAUX ; même chemin + même hôte
    /// doit rendre VRAI. Un seul des deux ne prouverait rien — une garde qui
    /// refuserait toujours passerait le premier, une garde qui ignorerait l'hôte
    /// passerait le second.
    ///
    /// Mutant qui tue ce test : retirer la comparaison d'hôtes → le premier cas,
    /// deux machines partageant le même chemin, devient vrai et meurt.
    #[test]
    fn compter_exige_la_meme_machine_et_le_meme_chemin() {
        let chemin = Path::new("/home/moi/.cache/bridget/bridget.db");

        // MÊME CHEMIN, HÔTES DISTINCTS — le cas que deux Linux produisent tout
        // seuls. La base nommée existe ici ET là-bas ; elle n'est pas la même.
        assert!(
            !daemon_store_is_local(
                Some("monordinateur"),
                Some("/home/moi/.cache/bridget/bridget.db"),
                "cartae",
                chemin
            ),
            "même chemin sur deux machines distinctes ne doit PAS être local"
        );

        // MÊME CHEMIN, MÊME HÔTE — contrôle positif opposé.
        assert!(
            daemon_store_is_local(
                Some("cartae"),
                Some("/home/moi/.cache/bridget/bridget.db"),
                "cartae",
                chemin
            ),
            "même chemin sur la même machine DOIT être local"
        );

        // MÊME HÔTE, CHEMINS DISTINCTS — l'hôte seul ne suffit pas non plus.
        assert!(
            !daemon_store_is_local(Some("cartae"), Some("/autre/bridget.db"), "cartae", chemin),
            "un autre chemin sur la même machine n'est pas cette base"
        );

        // Daemon antérieur : rien d'attesté, donc rien à compter.
        assert!(!daemon_store_is_local(None, None, "cartae", chemin));
        assert!(!daemon_store_is_local(
            Some("cartae"),
            None,
            "cartae",
            chemin
        ));
        assert!(!daemon_store_is_local(
            None,
            Some("/home/moi/.cache/bridget/bridget.db"),
            "cartae",
            chemin
        ));
    }
}

/// Ce que le daemon atteste de LUI-MÊME au client qui l'interroge.
#[derive(Debug, Clone, Default)]
pub struct DaemonIdentity {
    pub build_id: String,
    pub host: Option<String>,
    pub db_path: Option<String>,
}

fn daemon_identity(socket_path: &std::path::Path) -> Option<DaemonIdentity> {
    use std::io::{BufRead, BufReader, BufWriter, Write};
    use std::os::unix::net::UnixStream;

    let stream = UnixStream::connect(socket_path).ok()?;
    let read_stream = stream.try_clone().ok()?;
    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(read_stream);
    writeln!(
        writer,
        "{}",
        encode(&WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Client,
        })
        .ok()?
    )
    .ok()?;
    writer.flush().ok()?;
    let mut line = String::new();
    if reader.read_line(&mut line).ok()? == 0 {
        return None;
    }
    if !matches!(
        decode(line.trim()).ok()?,
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        }
    ) {
        return None;
    }
    writeln!(
        writer,
        "{}",
        encode(&WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            // Même dérivation que les clients normaux : la sonde reste compatible
            // avec toute évolution de la validation de portée.
            issuer_scope: build_id_probe_issuer_scope(),
            capabilities: Vec::new(),
        })
        .ok()?
    )
    .ok()?;
    writer.flush().ok()?;
    line.clear();
    match reader.read_line(&mut line).ok()? {
        0 => None,
        _ => match decode(line.trim()).ok()? {
            DaemonToWrapper::ClientWelcome { build_id, .. } => {
                // Second aller-retour, sur la MÊME connexion : la machine et la
                // base ne sont pas déductibles côté client.
                let (host, db_path) =
                    match probe_daemon_identity(&mut writer, &mut reader, &mut line) {
                        Some((host, db_path)) => (Some(host), Some(db_path)),
                        // Daemon antérieur au message : non attesté, jamais deviné.
                        None => (None, None),
                    };
                Some(DaemonIdentity {
                    build_id,
                    host,
                    db_path,
                })
            }
            _ => None,
        },
    }
}

/// Demande au daemon ce qu'il atteste de lui-même. `None` = daemon antérieur.
fn probe_daemon_identity<W: std::io::Write, R: std::io::BufRead>(
    writer: &mut W,
    reader: &mut R,
    line: &mut String,
) -> Option<(String, String)> {
    writeln!(
        writer,
        "{}",
        encode(&WrapperToDaemon::DaemonIdentityRequest).ok()?
    )
    .ok()?;
    writer.flush().ok()?;
    line.clear();
    if reader.read_line(line).ok()? == 0 {
        return None;
    }
    match decode(line.trim()).ok()? {
        DaemonToWrapper::DaemonIdentityReport { host, db_path } => Some((host, db_path)),
        _ => None,
    }
}

#[derive(Default, Clone)]
pub struct DaemonStatus {
    pub running: bool,
    pub agents: Vec<bridget_transport::protocol::AgentInfo>,
    /// `None` quand la base locale n'est PAS celle du daemon interrogé : on ne
    /// rend alors aucun chiffre plutôt qu'un chiffre pris ailleurs.
    pub message_count: Option<usize>,
    pub build_id: Option<String>,
    /// Machine et base attestées par le daemon lui-même.
    pub daemon_host: Option<String>,
    pub daemon_db_path: Option<String>,
}

#[cfg(test)]
mod signal_disposition_tests {
    use super::install_daemon_signal_handlers;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    const CHILD_ENV: &str = "BRIDGET_SIGNAL_DISPOSITION_CHILD";
    const TERM_DEADLINE: Duration = Duration::from_secs(2);

    /// Enfant de l'oracle : emprunte le chemin d'installation des signaux du
    /// daemon (no-op sous `cfg(test)`), puis reste vivant jusqu'à un TERM.
    #[test]
    #[ignore]
    fn signal_disposition_child() {
        if std::env::var(CHILD_ENV).ok().as_deref() != Some("1") {
            return;
        }
        install_daemon_signal_handlers();
        loop {
            std::thread::sleep(Duration::from_secs(60));
        }
    }

    fn assert_dies_on_term(mut child: std::process::Child, label: &str) {
        // Prouve d'abord qu'il était vivant — sinon l'oracle passe sur une
        // projection vide (processus déjà mort, TERM jamais évalué).
        std::thread::sleep(Duration::from_millis(150));
        assert!(
            child.try_wait().expect("try_wait").is_none(),
            "{label}: déjà mort avant TERM — projection vide"
        );
        assert_eq!(
            unsafe { libc::kill(child.id() as i32, libc::SIGTERM) },
            0,
            "{label}: kill(SIGTERM) a échoué"
        );
        let deadline = Instant::now() + TERM_DEADLINE;
        loop {
            if child.try_wait().expect("try_wait").is_some() {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "{label}: encore vivant après SIGTERM — le binaire de test \
                 a hérité d'un gestionnaire qui empêche la mort propre"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// Oracle : un binaire de test qui a emprunté `install_daemon_signal_handlers`
    /// DOIT mourir sous TERM. La baseline `sleep` prouve d'abord que la mesure
    /// elle-même fonctionne (sinon projection vide).
    #[test]
    fn binaire_de_test_meurt_sous_term_apres_install_signaux_daemon() {
        let baseline = Command::new("sleep")
            .arg("120")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn sleep baseline");
        assert_dies_on_term(baseline, "baseline sleep (disposition par défaut)");

        let child = Command::new(std::env::current_exe().expect("current_exe"))
            .arg("--exact")
            .arg("daemon::signal_disposition_tests::signal_disposition_child")
            .arg("--ignored")
            .arg("--nocapture")
            .env(CHILD_ENV, "1")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn signal_disposition_child");
        assert_dies_on_term(
            child,
            "binaire de test après install_daemon_signal_handlers",
        );
    }
}

#[cfg(test)]
mod presence_tests {
    use super::*;
    use bridget_core::BridgetMessage;
    use std::collections::BTreeMap;
    use std::ffi::{CString, OsString};
    use std::io::Read;
    use std::net::Shutdown;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::PermissionsExt;
    use std::process::{Child, Command};

    const T908_DAEMON_CHILD_ENV: &str = "BRIDGET_T908_DAEMON_CHILD";
    const T908_ROOT_ENV: &str = "BRIDGET_T908_ROOT";
    const SERVICE_NEGOTIATION_FIXTURE: &str = include_str!(
        "../../../specs/015-guichet-maicie/contracts/fixtures/service-negotiation-v1.jsonl"
    );

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
                caught_up: false,
            },
        );
        state.attach_views.insert(subscription_id.to_string(), view);
    }

    #[test]
    fn guichet_frame_limit_counts_the_wire_newline_and_parses_whitespace() {
        let base = r#"{ "type" : "guichet_claim_next", "version" : 1 }"#;
        assert!(raw_guichet_frame(base));
        let exact = format!(
            "{base}{}",
            " ".repeat(MAX_GUICHET_FRAME_BYTES - 1 - base.len())
        );
        let oversized = format!("{exact} ");
        assert_eq!(exact.len() + 1, MAX_GUICHET_FRAME_BYTES);
        assert_eq!(oversized.len() + 1, MAX_GUICHET_FRAME_BYTES + 1);
        // Mutation discriminante : passer de `>` à `>=`, ou retomber sur une
        // sous-chaîne compacte, rejetterait la première trame à tort ou
        // laisserait passer la seconde malgré ses espaces JSON valides.
        assert!(!guichet_frame_exceeds_wire_limit(&exact));
        assert!(guichet_frame_exceeds_wire_limit(&oversized));
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
        let (managed_tx, _managed_rx) = mpsc::channel();
        let mut state = DaemonState::new(&config, managed_tx).unwrap();
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
                transport: "tmux".to_string(),
                channel: Some("ssh-unix".to_string()),
                mode: Some(PresenceMode::Tmux),
                location: None,
                journal_available: false,
                os: "Linux".to_string(),
                state: "connected".to_string(),
                busy_since: None,
                capacity_seen: Instant::now(),
                link_seen: Instant::now(),
                reconnect_count: 0,
                model: Some("gpt-5.3-codex".to_string()),
                effort: Some("xhigh".to_string()),
                rate_limits: Default::default(),
                served_model: None,
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

    #[test]
    fn sonde_build_id_partage_la_derivation_de_portee_client() {
        let expected = crate::mcp::issuer_scope(BUILD_ID_PROBE_IDENTITY);
        assert_eq!(build_id_probe_issuer_scope(), expected);
        assert!(crate::idempotency::validate_issuer_scope(&expected).is_ok());
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
        let (managed_tx, _managed_rx) = mpsc::channel();
        let mut state = DaemonState::new(&config, managed_tx).unwrap();
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
                channel: Some("unix".to_string()),
                mode: Some(PresenceMode::Acp),
                location: None,
                journal_available: true,
                os: "macOS".to_string(),
                state: "connected".to_string(),
                busy_since: None,
                capacity_seen: Instant::now(),
                link_seen: Instant::now(),
                reconnect_count: 0,
                model: None,
                effort: None,
                rate_limits: Default::default(),
                served_model: None,
                derived_domain: None,
                domain: None,
                dnd_until: None,
            },
        );
        (state, config)
    }

    #[test]
    fn spec_024_trame_tmux_federee_historique_separe_protocole_et_canal() {
        let (mut state, config) = state_with_registered_agent("g11-tmux-federe");
        state.router.unregister_by_conn("conn-1");
        state.conn_instances.clear();
        state.presences.clear();

        let response = handle_register_with_channel(
            "conn-lab",
            "codex".to_string(),
            Some("lab-agent".to_string()),
            Some("lab-host".to_string()),
            Some("ssh-unix".to_string()),
            ChannelReport::Omitted,
            Some(PresenceMode::Tmux),
            Some("bridget:2.1".to_string()),
            Some("Linux".to_string()),
            Some("instance-lab".to_string()),
            Some("bridget".to_string()),
            false,
            Some(true),
            &mut state,
        );
        assert!(matches!(response, DaemonToWrapper::Registered { .. }));

        let info = state.agent_infos().pop().expect("agent fédéré visible");
        assert_eq!(info.transport, "tmux");
        assert_eq!(info.channel.as_deref(), Some("ssh-unix"));
        assert_eq!(info.mode, Some(PresenceMode::Tmux));
        assert_eq!(info.location.as_deref(), Some("bridget:2.1"));

        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn spec_024_protocole_et_canal_explicites_restent_independants() {
        let (mut state, config) = state_with_registered_agent("g11-explicite");
        state.router.unregister_by_conn("conn-1");
        state.conn_instances.clear();
        state.presences.clear();

        let response = handle_register_with_channel(
            "conn-natif",
            "fixture".to_string(),
            Some("natif-distant".to_string()),
            Some("lab-host".to_string()),
            Some("codex_app_server".to_string()),
            Some("ssh-unix".to_string()).into(),
            Some(PresenceMode::Cli),
            None,
            Some("Linux".to_string()),
            Some("instance-native".to_string()),
            Some("bridget".to_string()),
            false,
            Some(true),
            &mut state,
        );
        assert!(matches!(response, DaemonToWrapper::Registered { .. }));

        let info = state.agent_infos().pop().expect("agent natif visible");
        assert_eq!(info.transport, "codex_app_server");
        assert_eq!(info.channel.as_deref(), Some("ssh-unix"));
        assert_eq!(info.agent_type, "fixture");

        let _ = std::fs::remove_file(config.db_path);
    }

    fn recovery_fixture_state(root: &std::path::Path) -> (DaemonState, DaemonConfig) {
        std::fs::create_dir_all(root).unwrap();
        let config = DaemonConfig {
            socket_path: root.join("bridget.sock"),
            db_path: root.join("bridget.db"),
            log_path: root.join("daemon.log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        };
        let (managed_tx, _managed_rx) = mpsc::channel();
        let mut state = DaemonState::new(&config, managed_tx).unwrap();
        state.registry = AgentRegistry::from_json(
            &serde_json::json!({
                "agents": {
                    "fixture": {
                        "command": "/bin/sh",
                        "protocol": "acp",
                        "forbidden_env": [],
                        "pass_env": []
                    }
                }
            })
            .to_string(),
            root.join("agents.json"),
        )
        .unwrap();
        state.source_env = BTreeMap::from([
            ("HOME".to_string(), root.as_os_str().to_owned()),
            ("PATH".to_string(), OsString::from("/bin:/usr/bin")),
            ("USER".to_string(), OsString::from("tester")),
            ("LANG".to_string(), OsString::from("C")),
            ("TMPDIR".to_string(), OsString::from("/tmp")),
        ]);
        (state, config)
    }

    fn recovery_fixture_definition() -> bridget_transport::ResolvedAgentDefinition {
        AgentRegistry::from_json(
            r#"{"agents":{"fixture":{"command":"/bin/sh","protocol":"acp","forbidden_env":[],"pass_env":[]}}}"#,
            "/tmp/recovery-fixture-definition.json",
        )
        .unwrap()
        .resolved_definition("fixture")
        .unwrap()
    }

    fn recovery_native_codex_definition() -> bridget_transport::ResolvedAgentDefinition {
        // Même protocole et même matrice qu'un Codex natif, mais `/bin/sh`
        // pour que la reprise passe les gardes de commande. Sans
        // `execution_paths`, la préparation refuse le protocole et l'oracle
        // ne verrait jamais `who recovering`.
        AgentRegistry::from_json(
            r#"{"agents":{"fixture":{"command":"/bin/sh","args":["-c","model=\"gpt-5.6-terra\"","app-server"],"protocol":"codex_app_server","forbidden_env":[],"pass_env":[],"capabilities":{"execution_paths":["codex_app_server"],"models":{"gpt-5.6-terra":{}}}}}}"#,
            "/tmp/recovery-native-codex-definition.json",
        )
        .unwrap()
        .resolved_definition("fixture")
        .unwrap()
    }

    fn persist_connected_fixture_with_definition(
        state: &DaemonState,
        name: &str,
        command_id: &str,
        now: i64,
        definition: &bridget_transport::ResolvedAgentDefinition,
    ) -> SpawnLease {
        let order = FleetSpawnOrder {
            agent_type: "fixture".to_string(),
            requested_name: Some(name.to_string()),
            cwd: PathBuf::from("/tmp"),
            persistent: true,
            command_id: command_id.to_string(),
            issued_at: now,
            deadline_at: now + 60,
        };
        let lease = match state.fleet.request_spawn(&order, now).unwrap() {
            crate::fleet::SpawnSubmission::Start(lease) => lease,
            other => panic!("réservation persistante attendue: {other:?}"),
        };
        state.fleet.mark_starting(&lease, now, definition).unwrap();
        state
            .fleet
            .register_connected(&lease, &lease.instance_id, now + 1)
            .unwrap();
        lease
    }

    fn recovery_daemon_config(root: &std::path::Path) -> DaemonConfig {
        let cache = root.join(".cache/bridget");
        DaemonConfig {
            socket_path: cache.join("bridget.sock"),
            db_path: cache.join("bridget.db"),
            log_path: cache.join("daemon.log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        }
    }

    fn spawn_recovery_daemon(root: &std::path::Path) -> Child {
        Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("daemon::presence_tests::recovery_daemon_child")
            .arg("--ignored")
            .arg("--nocapture")
            .env(T908_DAEMON_CHILD_ENV, "1")
            .env(T908_ROOT_ENV, root)
            .env("HOME", root)
            .env("BRIDGET_T908_MANAGED_EXECUTABLE", managed_test_binary())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap()
    }

    fn wait_daemon_socket(socket: &std::path::Path) {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            if socket.exists() && UnixStream::connect(socket).is_ok() {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "socket daemon absente après reprise"
            );
            thread::sleep(Duration::from_millis(20));
        }
    }

    fn daemon_request(socket: &std::path::Path, request: WrapperToDaemon) -> DaemonToWrapper {
        let stream = UnixStream::connect(socket).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(8)))
            .unwrap();
        let mut writer = BufWriter::new(stream.try_clone().unwrap());
        writeln!(writer, "{}", encode(&request).unwrap()).unwrap();
        writer.flush().unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        decode(line.trim()).unwrap()
    }

    #[test]
    #[ignore]
    fn recovery_daemon_child() {
        if std::env::var(T908_DAEMON_CHILD_ENV).ok().as_deref() != Some("1") {
            return;
        }
        let root = PathBuf::from(std::env::var_os(T908_ROOT_ENV).unwrap());
        run(recovery_daemon_config(&root)).unwrap();
    }

    fn persist_connected_fixture(
        state: &DaemonState,
        name: &str,
        command_id: &str,
        now: i64,
    ) -> SpawnLease {
        persist_connected_fixture_with_definition(
            state,
            name,
            command_id,
            now,
            &recovery_fixture_definition(),
        )
    }

    #[test]
    fn reprise_reserve_tous_les_noms_avant_ouverture_et_devient_visible() {
        let root = PathBuf::from(format!(
            "/tmp/bg908-reserve-{}-{}",
            std::process::id(),
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        let (state, config) = recovery_fixture_state(&root);
        let now = unix_timestamp();
        let previous = persist_connected_fixture(&state, "alpha", "initial-alpha", now);
        drop(state);

        let (mut reopened, _) = recovery_fixture_state(&root);
        let recoveries = reserve_managed_recoveries(&mut reopened, now + 2).unwrap();
        assert_eq!(recoveries.len(), 1);
        assert_eq!(recoveries[0].0.lease.name, "alpha");
        assert!(recoveries[0].0.lease.generation > previous.generation);
        assert!(reopened.recovering);
        assert_eq!(reopened.recovery_commands.len(), 1);
        let info = &reopened.agent_infos()[0];
        assert_eq!(info.state, "recovering");
        assert_eq!(info.transport, "acp");
        assert_eq!(info.mode, Some(PresenceMode::Acp));
        drop(reopened);
        let _ = std::fs::remove_file(config.socket_path);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reprise_natif_codex_affiche_protocole_fige_pas_unix_acp() {
        let root = PathBuf::from(format!(
            "/tmp/bg-recovering-native-{}-{}",
            std::process::id(),
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        let (state, config) = recovery_fixture_state(&root);
        let now = unix_timestamp();
        persist_connected_fixture_with_definition(
            &state,
            "coder-natif",
            "initial-coder-natif",
            now,
            &recovery_native_codex_definition(),
        );
        drop(state);

        let (mut reopened, _) = recovery_fixture_state(&root);
        let recoveries = reserve_managed_recoveries(&mut reopened, now + 2).unwrap();
        assert_eq!(recoveries.len(), 1);
        let info = &reopened.agent_infos()[0];
        assert_eq!(info.state, "recovering");
        assert_eq!(
            info.transport, "codex_app_server",
            "who recovering doit exposer le protocole figé, pas unix"
        );
        assert_eq!(
            info.mode,
            Some(PresenceMode::Cli),
            "who recovering doit exposer MODE=cli, pas acp"
        );
        drop(reopened);
        let _ = std::fs::remove_file(config.socket_path);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn matrice_reprise_isole_echec_et_quota_sans_retry_infini() {
        let root = PathBuf::from(format!(
            "/tmp/bg908-matrix-{}-{}",
            std::process::id(),
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        let (_, config) = recovery_fixture_state(&root);
        let desired = DesiredStateStore::at_path(desired_state_path(&config));
        let mut fleet = crate::desired_state::DesiredFleet::default();
        let _env = crate::fleet::TestFleetQuotaGuard::set(2);
        for index in 0..5_u64 {
            fleet.equipiers.insert(
                format!("agent-{index:02}"),
                crate::desired_state::DesiredEquipier {
                    agent_type: if index == 0 { "inconnu" } else { "fixture" }.to_string(),
                    cwd: PathBuf::from("/tmp"),
                    command_id: format!("ancien-{index}"),
                    generation: index + 1,
                    created: index.to_string(),
                    resolved_definition: (index != 0).then(recovery_fixture_definition),
                    domain: None,
                },
            );
        }
        desired.persist(&fleet).unwrap();

        let (mut reopened, _) = recovery_fixture_state(&root);
        assert_eq!(reopened.fleet.quota(), 2);
        let recoveries = reserve_managed_recoveries(&mut reopened, unix_timestamp()).unwrap();
        assert_eq!(recoveries.len(), reopened.fleet.quota());
        let remaining = reopened.fleet.desired_fleet().unwrap();
        assert_eq!(remaining.equipiers.len(), reopened.fleet.quota());
        assert!(!remaining.equipiers.contains_key("agent-00"));
        // agent-03 et agent-04 refusés pour quota (message complet oraclé côté fleet).
        assert!(!remaining.equipiers.contains_key("agent-03"));
        assert!(!remaining.equipiers.contains_key("agent-04"));
        let report = crate::recovery_trace::load_report(&reopened.fleet.recovery_losses_path())
            .unwrap()
            .expect("trace des absents attendue");
        let by_name: std::collections::BTreeMap<_, _> = report
            .absents
            .iter()
            .map(|entry| (entry.name.as_str(), entry.reason.as_str()))
            .collect();
        assert_eq!(
            by_name.get("agent-00"),
            Some(&crate::recovery_trace::REASON_FROZEN_DEFINITION)
        );
        assert_eq!(
            by_name.get("agent-03"),
            Some(&crate::recovery_trace::REASON_QUOTA)
        );
        assert_eq!(
            by_name.get("agent-04"),
            Some(&crate::recovery_trace::REASON_QUOTA)
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reprise_trace_les_non_persistants_nommes_absents() {
        let root = PathBuf::from(format!(
            "/tmp/bg-d20-ephemere-{}-{}",
            std::process::id(),
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        let (_, config) = recovery_fixture_state(&root);
        let fleet_path = desired_state_path(&config);
        let roster = crate::recovery_trace::NamedRosterStore::at_path(
            crate::recovery_trace::roster_path(&fleet_path),
        );
        roster.remember(
            "cursor-ephemere".to_string(),
            crate::recovery_trace::NamedRosterEntry {
                agent_type: "cursor".to_string(),
                persistent: false,
                domain: Some("bridget".to_string()),
            },
        );

        let (mut reopened, _) = recovery_fixture_state(&root);
        let recoveries = reserve_managed_recoveries(&mut reopened, unix_timestamp()).unwrap();
        assert!(recoveries.is_empty());
        let report = crate::recovery_trace::load_report(&reopened.fleet.recovery_losses_path())
            .unwrap()
            .expect("trace des non-persistants");
        assert_eq!(report.absents.len(), 1);
        assert_eq!(report.absents[0].name, "cursor-ephemere");
        assert_eq!(
            report.absents[0].reason,
            crate::recovery_trace::REASON_NON_PERSISTENT
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reprise_trace_pertes_mixtes_au_meme_redemarrage() {
        let root = PathBuf::from(format!(
            "/tmp/bg-d20-mixte-{}-{}",
            std::process::id(),
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        let (_, config) = recovery_fixture_state(&root);
        let desired = DesiredStateStore::at_path(desired_state_path(&config));
        let mut fleet = crate::desired_state::DesiredFleet::default();
        let _env = crate::fleet::TestFleetQuotaGuard::set(2);
        for index in 0..5_u64 {
            fleet.equipiers.insert(
                format!("agent-{index:02}"),
                crate::desired_state::DesiredEquipier {
                    agent_type: if index == 0 { "inconnu" } else { "fixture" }.to_string(),
                    cwd: PathBuf::from("/tmp"),
                    command_id: format!("ancien-{index}"),
                    generation: index + 1,
                    created: index.to_string(),
                    resolved_definition: (index != 0).then(recovery_fixture_definition),
                    domain: None,
                },
            );
        }
        desired.persist(&fleet).unwrap();
        crate::recovery_trace::NamedRosterStore::at_path(crate::recovery_trace::roster_path(
            &desired_state_path(&config),
        ))
        .remember(
            "cursor-ephemere".to_string(),
            crate::recovery_trace::NamedRosterEntry {
                agent_type: "cursor".to_string(),
                persistent: false,
                domain: Some("bridget".to_string()),
            },
        );

        let (mut reopened, _) = recovery_fixture_state(&root);
        let _ = reserve_managed_recoveries(&mut reopened, unix_timestamp()).unwrap();
        let report = crate::recovery_trace::load_report(&reopened.fleet.recovery_losses_path())
            .unwrap()
            .expect("trace mixte");
        let by_name: std::collections::BTreeMap<_, _> = report
            .absents
            .iter()
            .map(|entry| (entry.name.as_str(), entry.reason.as_str()))
            .collect();
        assert_eq!(
            by_name.get("agent-00"),
            Some(&crate::recovery_trace::REASON_FROZEN_DEFINITION)
        );
        assert_eq!(
            by_name.get("agent-03"),
            Some(&crate::recovery_trace::REASON_QUOTA)
        );
        assert_eq!(
            by_name.get("agent-04"),
            Some(&crate::recovery_trace::REASON_QUOTA)
        );
        assert_eq!(
            by_name.get("cursor-ephemere"),
            Some(&crate::recovery_trace::REASON_NON_PERSISTENT)
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reprise_retire_une_trace_residuelle_si_zero_perte() {
        let root = PathBuf::from(format!(
            "/tmp/bg-d20-stale-{}-{}",
            std::process::id(),
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        let (_, config) = recovery_fixture_state(&root);
        let desired = DesiredStateStore::at_path(desired_state_path(&config));
        let mut fleet = crate::desired_state::DesiredFleet::default();
        fleet.equipiers.insert(
            "agent-ok".to_string(),
            crate::desired_state::DesiredEquipier {
                agent_type: "fixture".to_string(),
                cwd: PathBuf::from("/tmp"),
                command_id: "ancien-ok".to_string(),
                generation: 1,
                created: "1".to_string(),
                resolved_definition: Some(recovery_fixture_definition()),
                domain: None,
            },
        );
        desired.persist(&fleet).unwrap();
        let losses_path = crate::recovery_trace::report_path(&desired_state_path(&config));
        crate::recovery_trace::persist_report(
            &losses_path,
            1,
            vec![crate::recovery_trace::RecoveryLossEntry {
                name: "fantome".to_string(),
                reason: crate::recovery_trace::REASON_QUOTA.to_string(),
                detail: None,
            }],
        )
        .unwrap();
        assert!(losses_path.exists());

        let (mut reopened, _) = recovery_fixture_state(&root);
        let recoveries = reserve_managed_recoveries(&mut reopened, unix_timestamp()).unwrap();
        assert_eq!(recoveries.len(), 1);
        assert!(
            !reopened.fleet.recovery_losses_path().exists(),
            "zéro perte doit retirer le fichier, pas seulement le vider"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn roster_illisible_n_empeche_pas_la_reprise() {
        let root = PathBuf::from(format!(
            "/tmp/bg-d20-roster-ko-{}-{}",
            std::process::id(),
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        let (_, config) = recovery_fixture_state(&root);
        let desired = DesiredStateStore::at_path(desired_state_path(&config));
        let mut fleet = crate::desired_state::DesiredFleet::default();
        fleet.equipiers.insert(
            "agent-ok".to_string(),
            crate::desired_state::DesiredEquipier {
                agent_type: "fixture".to_string(),
                cwd: PathBuf::from("/tmp"),
                command_id: "ancien-ok".to_string(),
                generation: 1,
                created: "1".to_string(),
                resolved_definition: Some(recovery_fixture_definition()),
                domain: None,
            },
        );
        desired.persist(&fleet).unwrap();
        std::fs::write(
            crate::recovery_trace::roster_path(&desired_state_path(&config)),
            "ce n'est pas du json",
        )
        .unwrap();

        let (mut reopened, _) = recovery_fixture_state(&root);
        let recoveries = reserve_managed_recoveries(&mut reopened, unix_timestamp())
            .expect("un roster illisible ne doit pas avorter la reprise");
        assert_eq!(recoveries.len(), 1);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn persistant_disparu_de_fleet_json_est_trace() {
        let root = PathBuf::from(format!(
            "/tmp/bg-d20-silence-{}-{}",
            std::process::id(),
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        let (_, config) = recovery_fixture_state(&root);
        crate::recovery_trace::NamedRosterStore::at_path(crate::recovery_trace::roster_path(
            &desired_state_path(&config),
        ))
        .remember(
            "agent-z".to_string(),
            crate::recovery_trace::NamedRosterEntry {
                agent_type: "fixture".to_string(),
                persistent: true,
                domain: Some("bridget".to_string()),
            },
        );

        let (mut reopened, _) = recovery_fixture_state(&root);
        let recoveries = reserve_managed_recoveries(&mut reopened, unix_timestamp()).unwrap();
        assert!(recoveries.is_empty());
        let report = crate::recovery_trace::load_report(&reopened.fleet.recovery_losses_path())
            .unwrap()
            .expect("perte roster/fleet");
        assert_eq!(report.absents.len(), 1);
        assert_eq!(report.absents[0].name, "agent-z");
        assert_eq!(
            report.absents[0].reason,
            crate::recovery_trace::REASON_ABSENT_FROM_FLEET
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reprise_passe_quand_bridget_fleet_quota_est_pose() {
        let root = PathBuf::from(format!(
            "/tmp/bg-quota-env-{}-{}",
            std::process::id(),
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        let (_, config) = recovery_fixture_state(&root);
        let desired = DesiredStateStore::at_path(desired_state_path(&config));
        let mut fleet = crate::desired_state::DesiredFleet::default();
        for index in 0..10_u64 {
            fleet.equipiers.insert(
                format!("agent-{index:02}"),
                crate::desired_state::DesiredEquipier {
                    agent_type: "fixture".to_string(),
                    cwd: PathBuf::from("/tmp"),
                    command_id: format!("ancien-{index}"),
                    generation: index + 1,
                    created: index.to_string(),
                    resolved_definition: Some(recovery_fixture_definition()),
                    domain: None,
                },
            );
        }
        desired.persist(&fleet).unwrap();

        let _env = crate::fleet::TestFleetQuotaGuard::set(16);
        let (mut reopened, _) = recovery_fixture_state(&root);
        assert_eq!(reopened.fleet.quota(), 16);
        let recoveries = reserve_managed_recoveries(&mut reopened, unix_timestamp()).unwrap();
        assert_eq!(recoveries.len(), 10);
        let remaining = reopened.fleet.desired_fleet().unwrap();
        assert_eq!(remaining.equipiers.len(), 10);
        assert!(remaining.equipiers.contains_key("agent-09"));
        assert!(
            crate::recovery_trace::load_report(&reopened.fleet.recovery_losses_path())
                .unwrap()
                .is_none(),
            "zéro perte ne doit pas laisser de trace"
        );
        assert!(
            !reopened.fleet.recovery_losses_path().exists(),
            "zéro perte retire le fichier"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reprise_en_conflit_avec_un_wrapper_terminal_echoue_sans_retry() {
        let root = PathBuf::from(format!(
            "/tmp/bg908-conflict-{}-{}",
            std::process::id(),
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        let (_, config) = recovery_fixture_state(&root);
        let desired = DesiredStateStore::at_path(desired_state_path(&config));
        let mut fleet = crate::desired_state::DesiredFleet::default();
        fleet.equipiers.insert(
            "alpha".to_string(),
            crate::desired_state::DesiredEquipier {
                agent_type: "fixture".to_string(),
                cwd: PathBuf::from("/tmp"),
                command_id: "ancien-alpha".to_string(),
                generation: 1,
                created: "initial".to_string(),
                resolved_definition: Some(recovery_fixture_definition()),
                domain: None,
            },
        );
        desired.persist(&fleet).unwrap();

        let (mut state, _) = recovery_fixture_state(&root);
        let recoveries = reserve_managed_recoveries(&mut state, unix_timestamp()).unwrap();
        assert_eq!(recoveries.len(), 1);
        let lease = recoveries[0].0.lease.clone();
        let shared = Arc::new(Mutex::new(state));

        assert!(matches!(
            handle_wrapper_message(
                "terminal-alpha",
                WrapperToDaemon::Register {
                    agent_type: "fixture".to_string(),
                    name: Some("alpha".to_string()),
                    host: Some("local".to_string()),
                    transport: Some("tmux".to_string()),
                    channel: Some("unix".to_string()).into(),
                    mode: Some(PresenceMode::Tmux),
                    location: Some("fixture:0.1".to_string()),
                    os: Some("test".to_string()),
                    instance_id: None,
                    domain: None,
                    turn_in_progress: false,
                    journal_available: None,
                },
                &shared,
            ),
            Some(DaemonToWrapper::Registered { ref name }) if name == "alpha"
        ));
        assert!(matches!(
            handle_wrapper_message(
                "managed-alpha",
                WrapperToDaemon::Register {
                    agent_type: "fixture".to_string(),
                    name: Some("alpha".to_string()),
                    host: Some("local".to_string()),
                    transport: Some("acp".to_string()),
                    channel: Some("unix".to_string()).into(),
                    mode: Some(PresenceMode::Acp),
                    location: None,
                    os: Some("test".to_string()),
                    instance_id: Some(lease.instance_id.clone()),
                    domain: None,
                    turn_in_progress: false,
                    journal_available: None,
                },
                &shared,
            ),
            Some(DaemonToWrapper::Nack { ref reason, .. }) if reason.contains("déjà")
        ));

        {
            let state = shared.lock().unwrap();
            state
                .fleet
                .fail(&lease, "name_active", "nom déjà actif")
                .unwrap();
        }
        let (event_tx, event_rx) = mpsc::channel();
        event_tx
            .send(ManagedSupervisorEvent::Failed {
                lease: lease.clone(),
                kind: "name_active".to_string(),
                reason: "nom déjà actif".to_string(),
                conn_id: None,
            })
            .unwrap();
        drain_managed_events(&shared, &event_rx);

        let state = shared.lock().unwrap();
        assert!(
            !state
                .fleet
                .desired_fleet()
                .unwrap()
                .equipiers
                .contains_key("alpha")
        );
        assert!(!state.recovering);
        assert_eq!(
            state.router.get_agent("alpha").unwrap().connection_id,
            "terminal-alpha"
        );
        drop(state);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn stop_du_second_retablissement_gagne_avant_son_bootstrap() {
        let root = PathBuf::from(format!(
            "/tmp/bg908-stop-{}-{}",
            std::process::id(),
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        let (_, config) = recovery_fixture_state(&root);
        let desired = DesiredStateStore::at_path(desired_state_path(&config));
        let mut fleet = crate::desired_state::DesiredFleet::default();
        for name in ["alpha", "beta"] {
            fleet.equipiers.insert(
                name.to_string(),
                crate::desired_state::DesiredEquipier {
                    agent_type: "fixture".to_string(),
                    cwd: PathBuf::from("/tmp"),
                    command_id: format!("ancien-{name}"),
                    generation: 1,
                    created: "initial".to_string(),
                    resolved_definition: Some(recovery_fixture_definition()),
                    domain: None,
                },
            );
        }
        desired.persist(&fleet).unwrap();
        let (mut state, _) = recovery_fixture_state(&root);
        let (managed_tx, managed_rx) = mpsc::channel();
        state.managed_tx = managed_tx.clone();
        let recoveries = reserve_managed_recoveries(&mut state, unix_timestamp()).unwrap();
        assert_eq!(recoveries.len(), 2);
        let beta = state
            .managed_spawns
            .values()
            .find(|record| record.lease.name == "beta")
            .unwrap();
        let completion = Arc::clone(&beta.stop);
        let beta_command = beta.lease.command_id.clone();
        let alpha_command = state
            .managed_spawns
            .values()
            .find(|record| record.lease.name == "alpha")
            .unwrap()
            .lease
            .command_id
            .clone();

        let gate = root.join("bootstrap-gate");
        let gate_ready = root.join("bootstrap-gate.ready");
        let gate_once = root.join("bootstrap-gate.once");
        let gate_c = CString::new(gate.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(gate_c.as_ptr(), 0o600) }, 0);
        let shim = root.join("managed-shim.sh");
        std::fs::write(
            &shim,
            format!(
                "#!/bin/sh\nif [ \"$1\" = managed-bootstrap ]; then\n  if mkdir '{}' 2>/dev/null; then\n    : > '{}'\n    IFS= read -r release < '{}' || exit 1\n  fi\n  exec '{}' \"$@\"\nfi\ntrap 'exit 0' TERM\nwhile :; do sleep 1; done\n",
                gate_once.display(),
                gate_ready.display(),
                gate.display(),
                managed_test_binary().display(),
            ),
        )
        .unwrap();
        std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o700)).unwrap();

        let shared = Arc::new(Mutex::new(state));
        let (event_tx, event_rx) = mpsc::channel();
        let _supervisor = ManagedSupervisorGuard::from_existing_sender(
            managed_tx,
            managed_rx,
            Arc::clone(&shared.lock().unwrap().fleet),
            &config,
            event_tx,
            Some(shim),
        );
        for (prepared, stop) in recoveries {
            shared
                .lock()
                .unwrap()
                .managed_tx
                .send(ManagedSupervisorCommand::Start { prepared, stop })
                .unwrap();
        }
        let gate_deadline = Instant::now() + Duration::from_secs(5);
        while !gate_ready.exists() {
            assert!(
                Instant::now() < gate_deadline,
                "le premier bootstrap n'a pas atteint la barrière"
            );
            thread::yield_now();
        }

        let caller_state = Arc::clone(&shared);
        let caller = thread::spawn(move || {
            handle_wrapper_message(
                "control",
                WrapperToDaemon::StopOrder {
                    name: "beta".to_string(),
                    command_id: "stop-beta-recovery".to_string(),
                },
                &caller_state,
            )
        });
        let deadline = Instant::now() + Duration::from_secs(1);
        while !completion.is_actionable() {
            assert!(
                Instant::now() < deadline,
                "stop non visible avant bootstrap"
            );
            thread::yield_now();
        }

        std::fs::OpenOptions::new()
            .write(true)
            .open(&gate)
            .unwrap()
            .write_all(b"release\n")
            .unwrap();
        let stop_deadline = Instant::now() + Duration::from_secs(8);
        let result = loop {
            drain_managed_events(&shared, &event_rx);
            if caller.is_finished() {
                break caller.join().unwrap();
            }
            assert!(
                Instant::now() < stop_deadline,
                "le stop du second rétablissement n'a pas abouti"
            );
            thread::sleep(Duration::from_millis(10));
        };
        assert!(matches!(
            result,
            Some(DaemonToWrapper::StopResult {
                outcome: StopOutcome::Stopped | StopOutcome::StoppedForced { .. },
                ..
            })
        ));
        let state = shared.lock().unwrap();
        assert!(
            !state
                .fleet
                .desired_fleet()
                .unwrap()
                .equipiers
                .contains_key("beta")
        );
        assert!(!state.recovery_commands.contains(&beta_command));
        assert!(state.recovering, "alpha reste encore en reprise");
        drop(state);

        let alpha = Arc::clone(&shared.lock().unwrap().managed_spawns[&alpha_command].stop);
        alpha.mark_handshake_complete();
        let alpha_done = alpha.request();
        let cleanup_deadline = Instant::now() + Duration::from_secs(8);
        loop {
            drain_managed_events(&shared, &event_rx);
            if alpha_done.try_recv().is_ok() {
                break;
            }
            assert!(
                Instant::now() < cleanup_deadline,
                "nettoyage du premier rétablissement incomplet"
            );
            thread::sleep(Duration::from_millis(10));
        }
        drop(shared);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn sigkill_daemon_reconcilie_l_ancien_groupe_avant_une_reprise_unique() {
        let root = PathBuf::from(format!(
            "/tmp/bg908-e2e-{}-{}",
            std::process::id(),
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        let config = recovery_daemon_config(&root);
        std::fs::create_dir_all(config.db_path.parent().unwrap()).unwrap();
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert!(
            Command::new("git")
                .args(["init", "-q", root.to_str().unwrap()])
                .status()
                .unwrap()
                .success()
        );
        std::fs::write(root.join("wip.txt"), "base\n").unwrap();
        assert!(
            Command::new("git")
                .args([
                    "-C",
                    root.to_str().unwrap(),
                    "-c",
                    "user.name=Test",
                    "-c",
                    "user.email=test@example.invalid",
                    "add",
                    "wip.txt"
                ])
                .status()
                .unwrap()
                .success()
        );
        assert!(
            Command::new("git")
                .args([
                    "-C",
                    root.to_str().unwrap(),
                    "-c",
                    "user.name=Test",
                    "-c",
                    "user.email=test@example.invalid",
                    "commit",
                    "-qm",
                    "base"
                ])
                .status()
                .unwrap()
                .success()
        );
        std::fs::write(root.join("wip.txt"), "non commité\n").unwrap();
        let maicie_config = root.join(".config/maicie/config.json");
        std::fs::create_dir_all(maicie_config.parent().unwrap()).unwrap();
        let maicie_db = root.join("maicie.sqlite3");
        std::fs::write(
            &maicie_config,
            serde_json::to_vec(&serde_json::json!({
                "version": 1, "bridget_socket": config.socket_path, "database_path": maicie_db,
                "durations": {"short_secs":30,"normal_secs":60,"long_secs":90}, "profiles": []
            }))
            .unwrap(),
        )
        .unwrap();
        let mut maicie = maicie::store::MaicieStore::open(&maicie_db).unwrap();
        let mission = maicie::app::delegate(
            &mut maicie,
            maicie::config::DurationClasses {
                short_secs: 30,
                normal_secs: 60,
                long_secs: 90,
            },
            "maicie",
            &[maicie::app::DelegationCandidate {
                name: "persistent-one".to_string(),
                tags: vec![],
                available: true,
                dnd: false,
            }],
            &maicie::app::DelegateRequest {
                goal: "reprendre la bissection",
                explicit_target: Some("persistent-one"),
                required_tags: &[],
                duration: maicie::domain::ClasseDuree::Normale,
                reply: false,
                constat_id: None,
                review_target: None,
                suite: maicie::domain::SuiteObjective::Aucune,
                depends_on: &[],
                references: &[],
                idempotency_key: "resume-daemon-crash",
                now: 100,
                retry_until: 150,
                dedup_retained_until: 200,
                max_frame_bytes: 256 * 1024,
            },
        )
        .unwrap();
        let maicie::app::DelegateResult::Created(mission) = mission else {
            panic!("mission attendue")
        };
        let registry_path = root.join(".config/bridget/agents.json");
        std::fs::create_dir_all(registry_path.parent().unwrap()).unwrap();
        let adapter = root.join("adapter.sh");
        let changed_adapter = root.join("adapter-changed.sh");
        let old_runs = root.join("old-adapter-runs");
        let resume_prompts = root.join("resume-prompts");
        let changed_runs = root.join("changed-adapter-runs");
        std::fs::write(
            &adapter,
            format!(
                "#!/bin/sh\necho old >> {}\nread initialize || exit 1\necho '{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{{\"protocolVersion\":1}}}}'\nread session || exit 1\necho '{{\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{{\"sessionId\":\"recovery-session\"}}}}'\nwhile read line; do case \"$line\" in *session/prompt*) echo \"$line\" >> {} ;; esac; done\n",
                old_runs.display(), resume_prompts.display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&adapter, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::write(
            &changed_adapter,
            format!(
                "#!/bin/sh\necho changed >> {}\nexit 91\n",
                changed_runs.display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&changed_adapter, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::write(
            &registry_path,
            serde_json::to_vec(&serde_json::json!({
                "agents": {
                    "fixture": {
                        "command": adapter,
                        "protocol": "acp",
                        "permissions": "allow",
                        "forbidden_env": [],
                        "pass_env": [],
                        "queue_capacity": 2,
                        "notify_timeout_secs": 1
                    }
                }
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::set_permissions(&registry_path, std::fs::Permissions::from_mode(0o600)).unwrap();

        let mut first_daemon = spawn_recovery_daemon(&root);
        wait_daemon_socket(&config.socket_path);
        let now = unix_timestamp();
        let accepted = daemon_request(
            &config.socket_path,
            WrapperToDaemon::SpawnOrder {
                agent_type: "fixture".to_string(),
                name: Some("persistent-one".to_string()),
                cwd: root.to_string_lossy().to_string(),
                persistent: true,
                command_id: "initial-persistent".to_string(),
                issued_at: now,
                deadline_at: now + 20,
            },
        );
        assert!(matches!(
            accepted,
            DaemonToWrapper::SpawnAccepted { ref name, definition: Some(ref definition), .. }
                if name == "persistent-one"
                    && definition.command == adapter.to_string_lossy()
                    && definition.args.is_empty()
                    && definition.forbidden_env.is_empty()
                    && definition.digest.len() == 64
        ));
        let marker_store =
            ManagedMarkerStore::at_directory(config.db_path.parent().unwrap().join("managed"));
        let prompt_deadline = Instant::now() + Duration::from_secs(4);
        while std::fs::read_to_string(&resume_prompts)
            .unwrap_or_default()
            .lines()
            .count()
            < 1
        {
            assert!(
                Instant::now() < prompt_deadline,
                "première carte de reprise absente"
            );
            thread::sleep(Duration::from_millis(10));
        }
        let first_marker = marker_store.load("persistent-one").unwrap();
        assert!(crate::managed_process::group_exists(first_marker.pgid).unwrap());

        assert_ne!(first_daemon.id(), 0);
        assert_eq!(
            unsafe { libc::kill(first_daemon.id() as libc::pid_t, libc::SIGKILL) },
            0
        );
        let _ = first_daemon.wait().unwrap();

        // Le registre mutable dérive après le crash. La reprise doit pourtant
        // exécuter l'ancien adaptateur figé et republier la même preuve.
        std::fs::write(
            &registry_path,
            serde_json::to_vec(&serde_json::json!({
                "agents": {
                    "fixture": {
                        "command": changed_adapter,
                        "args": ["new-runtime-meaning"],
                        "protocol": "acp",
                        "permissions": "deny",
                        "forbidden_env": ["CHANGED_KEY"],
                        "pass_env": ["LANG"],
                        "queue_capacity": 7,
                        "notify_timeout_secs": 9,
                        "mcp": {"interactive": "claude", "acp_session": true}
                    }
                }
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::set_permissions(&registry_path, std::fs::Permissions::from_mode(0o600)).unwrap();

        let mut restarted = spawn_recovery_daemon(&root);
        wait_daemon_socket(&config.socket_path);
        let deadline = Instant::now() + Duration::from_secs(12);
        let agents = loop {
            if let DaemonToWrapper::AgentList { agents } =
                daemon_request(&config.socket_path, WrapperToDaemon::ListAgents)
                && agents.len() == 1
                && agents[0].name == "persistent-one"
                && agents[0].state == "connected"
            {
                break agents;
            }
            assert!(
                Instant::now() < deadline,
                "reprise persistante non connectée"
            );
            thread::sleep(Duration::from_millis(25));
        };
        assert_eq!(agents.len(), 1, "une seule génération doit être visible");
        assert!(!crate::managed_process::group_exists(first_marker.pgid).unwrap());
        let second_marker = marker_store.load("persistent-one").unwrap();
        assert_ne!(second_marker.instance_id, first_marker.instance_id);
        assert_ne!(second_marker.pgid, first_marker.pgid);
        let execution_deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let executions = std::fs::read_to_string(&old_runs).unwrap().lines().count();
            if executions == 2 {
                break;
            }
            assert!(
                Instant::now() < execution_deadline,
                "la commande figée doit être exécutée une fois avant et une fois après le crash"
            );
            thread::yield_now();
        }
        let prompt_deadline = Instant::now() + Duration::from_secs(4);
        let prompts = loop {
            let prompts = std::fs::read_to_string(&resume_prompts).unwrap_or_default();
            if prompts.lines().count() >= 2 {
                break prompts;
            }
            assert!(
                Instant::now() < prompt_deadline,
                "carte de reprise post-crash absente"
            );
            thread::sleep(Duration::from_millis(10));
        };
        assert!(prompts.contains(&mission.objective_id.to_string()));
        assert!(prompts.contains(&mission.delegation_id.to_string()));
        assert!(
            prompts.contains(
                &mission
                    .message_id
                    .expect("délégation créée sans prérequis porte un message_id")
                    .to_string()
            )
        );
        assert!(prompts.contains("reprendre la bissection"));
        assert!(prompts.contains("wip.txt"));
        assert!(
            !changed_runs.exists(),
            "le registre modifié ne doit jamais piloter la génération reprise"
        );
        assert!(matches!(
            daemon_request(
                &config.socket_path,
                WrapperToDaemon::SpawnOrder {
                    agent_type: "fixture".to_string(),
                    name: Some("persistent-one".to_string()),
                    cwd: root.to_string_lossy().to_string(),
                    persistent: true,
                    command_id: "initial-persistent".to_string(),
                    issued_at: now,
                    deadline_at: now + 20,
                }
            ),
            DaemonToWrapper::SpawnAccepted { definition: Some(definition), .. }
                if definition.command == adapter.to_string_lossy()
                    && definition.permissions == "allow"
                    && definition.queue_capacity == 2
                    && definition.notify_timeout_secs == 1
        ));

        assert!(matches!(
            daemon_request(
                &config.socket_path,
                WrapperToDaemon::StopOrder {
                    name: "persistent-one".to_string(),
                    command_id: "stop-after-recovery".to_string(),
                }
            ),
            DaemonToWrapper::StopResult {
                outcome: StopOutcome::Stopped | StopOutcome::StoppedForced { .. },
                ..
            }
        ));
        assert_eq!(
            unsafe { libc::kill(restarted.id() as libc::pid_t, libc::SIGTERM) },
            0
        );
        let status = restarted.wait().unwrap();
        assert!(status.success());
        std::fs::remove_dir_all(root).unwrap();
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
    fn service_maicie_negocie_sa_capacite_sans_lier_le_nom_au_droit() {
        let (state, config) = state_with_registered_agent("service-guichet");
        let shared = Arc::new(Mutex::new(state));
        let scope = "015_scope_0123456789abcdef0123456789abcdef";

        // Mutation discriminante : si un simple `from` ou nom de wrapper
        // autorisait le guichet, ce claim serait accepté au lieu du refus.
        assert!(matches!(
            handle_wrapper_message(
                "wrapper-maicie",
                WrapperToDaemon::Register {
                    agent_type: "codex".to_string(),
                    name: Some("maicie".to_string()),
                    host: None,
                    transport: None,
                    channel: None.into(),
                    mode: None,
                    location: None,
                    os: None,
                    instance_id: None,
                    domain: None,
                    turn_in_progress: false,
                    journal_available: None,
                },
                &shared,
            ),
            Some(DaemonToWrapper::Registered { .. })
        ));
        assert!(matches!(
            handle_wrapper_message(
                "wrapper-maicie",
                WrapperToDaemon::GuichetClaimNext {
                    version: SERVICE_CONTRACT_VERSION
                },
                &shared
            ),
            Some(DaemonToWrapper::ServiceRejected {
                reason: ServiceRefusal::ServiceRoleRequired
            })
        ));

        for connection in ["service-without-capability", "service-capable"] {
            assert!(matches!(
                handle_wrapper_message(
                    connection,
                    WrapperToDaemon::RoleHandshake {
                        role: ConnectionRole::Service,
                    },
                    &shared,
                ),
                Some(DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Service
                })
            ));
        }
        assert!(matches!(
            handle_wrapper_message(
                "service-without-capability",
                WrapperToDaemon::ServiceHello {
                    version: SERVICE_CONTRACT_VERSION,
                    service: "maicie".to_string(),
                    issuer_scope: scope.to_string(),
                    capabilities: Vec::new(),
                },
                &shared,
            ),
            Some(DaemonToWrapper::ServiceWelcome { capabilities, .. }) if capabilities.is_empty()
        ));
        assert!(matches!(
            handle_wrapper_message(
                "service-without-capability",
                WrapperToDaemon::GuichetClaimNext {
                    version: SERVICE_CONTRACT_VERSION
                },
                &shared,
            ),
            Some(DaemonToWrapper::ServiceRejected {
                reason: ServiceRefusal::CapabilityRequired
            })
        ));
        assert!(matches!(
            handle_wrapper_message(
                "service-without-capability",
                WrapperToDaemon::GuichetReply {
                    version: SERVICE_CONTRACT_VERSION,
                    issuer_scope: scope.to_string(),
                    request_id: "req-1".to_string(),
                    claim_generation: 1,
                    claim_token: "claim-1".to_string(),
                    response_message_id: "message-1".to_string(),
                    in_reply_to: "message-0".to_string(),
                    outcome: bridget_transport::protocol::GuichetOutcome::Accepted,
                    payload: bridget_transport::protocol::GuichetReplyPayload::DeliveryReport {
                        objective_id: "objective-1".to_string(),
                        delegation_id: "delegation-1".to_string(),
                        delivery_hash: "0".repeat(64),
                        review_verdict: None,
                    },
                },
                &shared,
            ),
            Some(DaemonToWrapper::ServiceRejected {
                reason: ServiceRefusal::CapabilityRequired
            })
        ));
        assert!(matches!(
            handle_wrapper_message(
                "service-capable",
                WrapperToDaemon::ServiceHello {
                    version: SERVICE_CONTRACT_VERSION,
                    service: "maicie".to_string(),
                    issuer_scope: scope.to_string(),
                    capabilities: vec![ServiceCapability::MaicieGuichet],
                },
                &shared,
            ),
            Some(DaemonToWrapper::ServiceWelcome { capabilities, .. })
                if capabilities == vec![ServiceCapability::MaicieGuichet]
        ));
        assert!(matches!(
            handle_wrapper_message(
                "service-capable",
                WrapperToDaemon::GuichetClaimNext {
                    version: SERVICE_CONTRACT_VERSION
                },
                &shared
            ),
            Some(DaemonToWrapper::GuichetEmpty {
                version: SERVICE_CONTRACT_VERSION
            })
        ));

        assert_eq!(shared.lock().unwrap().service_negotiations.len(), 2);
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn service_capability_is_cleaned_after_a_broken_response_socket() {
        let (state, config) = state_with_registered_agent("service-cleanup-real-socket");
        let shared = Arc::new(Mutex::new(state));
        let listener = UnixListener::bind(&config.socket_path).unwrap();
        let (first_closed_tx, first_closed_rx) = mpsc::channel();
        let state_for_server = Arc::clone(&shared);
        let server = thread::spawn(move || {
            let (first, _) = listener.accept().unwrap();
            let first_failed = handle_connection(first, Arc::clone(&state_for_server)).is_err();
            first_closed_tx.send(first_failed).unwrap();

            let (second, _) = listener.accept().unwrap();
            handle_connection(second, state_for_server).is_ok()
        });
        let fixture = SERVICE_NEGOTIATION_FIXTURE.lines().collect::<Vec<_>>();

        let first = UnixStream::connect(&config.socket_path).unwrap();
        first
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut first_writer = BufWriter::new(first.try_clone().unwrap());
        let mut first_reader = BufReader::new(first);
        writeln!(first_writer, "{}", fixture[0]).unwrap();
        first_writer.flush().unwrap();
        let mut accepted = String::new();
        first_reader.read_line(&mut accepted).unwrap();
        assert_eq!(accepted.trim_end(), fixture[1]);

        // Mutation discriminante : sans la sortie commune de
        // `handle_connection`, le BrokenPipe ci-dessous laisserait la capacité
        // dans `service_negotiations` et la connexion suivante l'hériterait.
        writeln!(first_writer, "{}", fixture[2]).unwrap();
        first_writer.flush().unwrap();
        first_writer.get_ref().shutdown(Shutdown::Both).unwrap();
        drop(first_reader);
        drop(first_writer);
        assert!(
            first_closed_rx
                .recv_timeout(Duration::from_secs(3))
                .unwrap(),
            "la réponse ServiceWelcome doit échouer sur le socket fermé"
        );
        assert!(shared.lock().unwrap().service_negotiations.is_empty());

        let second = UnixStream::connect(&config.socket_path).unwrap();
        second
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut second_writer = BufWriter::new(second.try_clone().unwrap());
        let mut second_reader = BufReader::new(second);
        writeln!(second_writer, "{}", fixture[0]).unwrap();
        second_writer.flush().unwrap();
        let mut second_accepted = String::new();
        second_reader.read_line(&mut second_accepted).unwrap();
        assert_eq!(second_accepted.trim_end(), fixture[1]);

        let no_capability = WrapperToDaemon::ServiceHello {
            version: SERVICE_CONTRACT_VERSION,
            service: "maicie".to_string(),
            issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
            capabilities: Vec::new(),
        };
        writeln!(second_writer, "{}", encode(&no_capability).unwrap()).unwrap();
        second_writer.flush().unwrap();
        let mut welcome = String::new();
        second_reader.read_line(&mut welcome).unwrap();
        assert!(matches!(
            decode::<DaemonToWrapper>(welcome.trim_end()).unwrap(),
            DaemonToWrapper::ServiceWelcome { capabilities, .. } if capabilities.is_empty()
        ));
        writeln!(
            second_writer,
            "{}",
            encode(&WrapperToDaemon::GuichetClaimNext {
                version: SERVICE_CONTRACT_VERSION
            })
            .unwrap()
        )
        .unwrap();
        second_writer.flush().unwrap();
        let mut refusal = String::new();
        second_reader.read_line(&mut refusal).unwrap();
        assert_eq!(refusal.trim_end(), fixture[4]);

        drop(second_reader);
        drop(second_writer);
        assert!(server.join().unwrap(), "seconde connexion nettoyée");
        assert!(shared.lock().unwrap().service_negotiations.is_empty());
        let _ = std::fs::remove_file(config.socket_path);
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
                build_id,
                horizon_secs: CLIENT_IDEMPOTENCY_HORIZON_SECS,
                issued_at_tolerance_secs: CLIENT_ISSUED_AT_TOLERANCE_SECS,
                capabilities,
            }) if build_id == crate::build_info::BUILD_ID
                && capabilities == vec![ClientCapability::SendIdempotent, ClientCapability::Lookup]
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
        let (mut state, config) = state_with_registered_agent("client-wrapper-matrix");
        let key = IdempotencyKey::new(
            "012_scope_aaaaaaaaaaaa",
            OperationKind::Send,
            "message-ack-wrapper",
        )
        .unwrap();
        let now = unix_now_secs();
        state
            .idempotency
            .reserve(
                &key,
                b"ack-wrapper",
                now,
                CLIENT_IDEMPOTENCY_HORIZON_SECS,
                now,
                CLIENT_ISSUED_AT_TOLERANCE_SECS,
            )
            .unwrap();
        let mut ack_message =
            bridget_core::BridgetMessage::new("sender-peer", "peer-1", "corps collège");
        ack_message.id = "message-ack-wrapper".to_string();
        let message_bytes = serde_json::to_vec(&ack_message).unwrap();
        state
            .idempotency
            .begin_send_delivery(
                &key,
                &SendDelivery {
                    delivery_id: "delivery-1".to_string(),
                    recipient_instance_id: "instance-1".to_string(),
                    delivery_generation: 1,
                    expires_at: now + CLIENT_IDEMPOTENCY_HORIZON_SECS,
                    message_bytes,
                },
            )
            .unwrap();
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
        let record_count_after_ack = shared.lock().unwrap().idempotency.record_count().unwrap();
        assert!(matches!(
            handle_wrapper_message(
                "historic-register",
                WrapperToDaemon::Register {
                    agent_type: "codex".to_string(),
                    name: Some("historique-012".to_string()),
                    host: None,
                    transport: None,
                    channel: None.into(),
                    mode: None,
                    location: None,
                    os: None,
                    instance_id: None,
                    domain: None,
                    turn_in_progress: false,
                    journal_available: None,
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
        assert_eq!(
            shared.lock().unwrap().idempotency.record_count().unwrap(),
            record_count_after_ack
        );
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

    /// ORACLE — SendIdempotent reply=false (tous les mandats Maicie) doit
    /// poser deadline_at au DEFAULT natif pour un type ABSENT de agents.json
    /// (codex). Mutant : omettre stamp dans defer_idempotent_delivery → None.
    /// Mutant : reposer 600 (fleet figé) → timeout hors 2700.
    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_send_idempotent_reply_false_pose_deadline_codex_sans_agents_json() {
        let empty = AgentRegistry::from_json("{}", "/tmp/agents-temoin-codex-deadline.json")
            .expect("registre vide charge les natifs");
        assert_eq!(
            empty.get("codex").unwrap().notify_timeout_secs,
            DEFAULT_NOTIFY_TIMEOUT_SECS,
            "précondition : natif codex = DEFAULT, pas un 600 ailleurs"
        );
        assert_eq!(DEFAULT_NOTIFY_TIMEOUT_SECS, 2700);

        let (mut state, config) = state_with_registered_agent("idempotent-codex-deadline");
        // Comme relec6 : type codex, présence connectée, sans entrée agents.json.
        state.presences.get_mut("instance-1").unwrap().agent_type = "codex".to_string();
        let (target_writer, mut target_reader) = control_socket("idempotent-codex-deadline");
        state
            .connections
            .insert("conn-1".to_string(), target_writer);

        let shared = Arc::new(Mutex::new(state));
        negotiate_idempotent_client(&shared, "client-codex-deadline", "012_scope_codexdeadline");

        let mut message = idempotent_message("mandat Maicie sans reply");
        message.reply = false;
        assert!(
            message.deadline_at.is_none(),
            "contrôle positif : le client n'apporte pas de deadline"
        );

        let before = unix_now_secs() as u64;
        let result = handle_wrapper_message(
            "client-codex-deadline",
            WrapperToDaemon::SendIdempotent {
                message,
                message_id: "mandat-codex-sans-deadline".to_string(),
                issued_at: unix_now_secs(),
            },
            &shared,
        );
        assert!(
            matches!(
                result,
                Some(DaemonToWrapper::IdempotencyResult {
                    issue: IdempotencyIssue::OutcomeUnknown { .. },
                    ..
                })
            ),
            "envoi idempotent attendu, reçu {result:?}"
        );

        let delivered = match read_control(&mut target_reader) {
            DaemonToWrapper::DeliverIdempotent { message, .. } => message,
            other => panic!("DeliverIdempotent attendu vers le wrapper: {other:?}"),
        };
        let after = unix_now_secs() as u64;
        let deadline = delivered
            .deadline_at
            .expect("deadline_at DOIT être posé pour reply=false sur DeliverIdempotent");
        let timeout = deadline.saturating_sub(before);
        assert!(
            timeout >= DEFAULT_NOTIFY_TIMEOUT_SECS.saturating_sub(2)
                && timeout
                    <= DEFAULT_NOTIFY_TIMEOUT_SECS
                        .saturating_add(after.saturating_sub(before).saturating_add(2)),
            "codex sans agents.json doit recevoir DEFAULT={DEFAULT_NOTIFY_TIMEOUT_SECS}s ; \
             timeout≈{timeout} deadline={deadline} before={before} after={after}"
        );
        assert!(
            timeout > 660,
            "régression : encore le plafond 600 s (fleet/notify figé) ; timeout={timeout}"
        );

        let _ = std::fs::remove_file(config.db_path);
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
        let first_key =
            IdempotencyKey::new(scope_a, OperationKind::Send, "message-identique").unwrap();
        let second_key =
            IdempotencyKey::new(scope_b, OperationKind::Send, "message-identique").unwrap();
        assert_ne!(
            state
                .idempotency
                .send_delivery(&first_key)
                .unwrap()
                .unwrap()
                .delivery_generation,
            state
                .idempotency
                .send_delivery(&second_key)
                .unwrap()
                .unwrap()
                .delivery_generation
        );
        drop(state);
        // Un rejeu est jugé avant tout routage : retirer ou renommer la cible
        // n'autorise jamais une nouvelle résolution pour cette clé connue.
        {
            let mut state = shared.lock().unwrap();
            state.router.rename("conn-1", "agent-renomme").unwrap();
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
        assert_eq!(state.idempotency.record_count().unwrap(), 2);
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
            rejected.as_ref(),
            Some(DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::Rejected { category, .. },
                ..
            }) if category == "routing"
        ));
        let replay = handle_wrapper_message("client-reject", send(), &shared);
        assert!(matches!(
            replay.as_ref(),
            Some(DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::Rejected { category, .. },
                ..
            }) if category == "routing"
        ));
        let rejection_expiry = match rejected {
            Some(DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::Rejected { expires_at, .. },
                ..
            }) => expires_at,
            _ => unreachable!("refus idempotent attendu"),
        };
        let replay_expiry = match replay {
            Some(DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::Rejected { expires_at, .. },
                ..
            }) => expires_at,
            _ => unreachable!("rejeu du refus attendu"),
        };
        assert_eq!(replay_expiry, rejection_expiry);

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
        assert!(
            state
                .pending_replies
                .iter()
                .any(|pending| pending.msg_id == "reply-idempotent")
        );
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn accuse_idempotent_d_une_reponse_liee_resout_la_demande_sans_relance() {
        let (mut state, config) = state_with_registered_agent("idempotent-linked-reply");
        state.router.rename("conn-1", "bridget").unwrap();
        state.presences.get_mut("instance-1").unwrap().name = "bridget".to_string();
        let (target_writer, mut target_reader) = control_socket("idempotent-linked-reply");
        state
            .connections
            .insert("conn-1".to_string(), target_writer);
        state
            .store
            .create_request("request-open", "bridget", "coderBridget", 60)
            .unwrap();
        state.pending_replies.push(PendingReply {
            msg_id: "request-open".to_string(),
            from: "bridget".to_string(),
            from_conn: "conn-1".to_string(),
            to: "coderBridget".to_string(),
            target_conn: "client-reply".to_string(),
            timeout_secs: 60,
            created_at: Instant::now(),
            escalation_level: 0,
            deferred_level: None,
        });
        let shared = Arc::new(Mutex::new(state));
        negotiate_idempotent_client(&shared, "client-reply", "012_scope_replyyyyyyyyy");

        let mut response = BridgetMessage::new("coderBridget", "bridget", "réponse MCP");
        response.in_reply_to = Some("request-open".to_string());
        let issued_at = unix_now_secs();
        assert!(matches!(
            handle_wrapper_message(
                "client-reply",
                WrapperToDaemon::SendIdempotent {
                    message: response,
                    message_id: "mcp-linked-reply".to_string(),
                    issued_at,
                },
                &shared,
            ),
            Some(DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            })
        ));
        let (delivery_id, delivery_generation) = match read_control(&mut target_reader) {
            DaemonToWrapper::DeliverIdempotent {
                delivery_id,
                delivery_generation,
                message,
                ..
            } => {
                assert_eq!(message.in_reply_to.as_deref(), Some("request-open"));
                (delivery_id, delivery_generation)
            }
            other => panic!("remise idempotente attendue: {other:?}"),
        };
        assert!(
            handle_wrapper_message(
                "conn-1",
                WrapperToDaemon::DeliverAcked {
                    delivery_id,
                    delivery_generation,
                },
                &shared,
            )
            .is_none()
        );

        let state = shared.lock().unwrap();
        assert_eq!(
            state
                .store
                .get_request("request-open")
                .unwrap()
                .unwrap()
                .state,
            "answered"
        );
        assert!(state.pending_replies.is_empty());
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn meme_cle_idempotente_avec_reponse_liee_divergente_est_refusee_sans_mutation() {
        let (state, config) = state_with_registered_agent("idempotent-linked-mismatch");
        state
            .store
            .create_request("request-a", "agent-2", "coderBridget", 60)
            .unwrap();
        state
            .store
            .create_request("request-b", "agent-2", "coderBridget", 60)
            .unwrap();
        let shared = Arc::new(Mutex::new(state));
        negotiate_idempotent_client(&shared, "client-reply", "012_scope_replyyyyyyyyy");
        let issued_at = unix_now_secs();
        for request_id in ["request-a", "request-b"] {
            let mut response = BridgetMessage::new("coderBridget", "agent-2", "réponse MCP");
            response.in_reply_to = Some(request_id.to_string());
            let result = handle_wrapper_message(
                "client-reply",
                WrapperToDaemon::SendIdempotent {
                    message: response,
                    message_id: "mcp-linked-mismatch".to_string(),
                    issued_at,
                },
                &shared,
            );
            if request_id == "request-a" {
                assert!(matches!(
                    result,
                    Some(DaemonToWrapper::IdempotencyResult {
                        issue: IdempotencyIssue::OutcomeUnknown { .. },
                        ..
                    })
                ));
            } else {
                assert!(matches!(
                    result,
                    Some(DaemonToWrapper::IdempotencyResult {
                        issue: IdempotencyIssue::EnvelopeMismatch,
                        ..
                    })
                ));
            }
        }
        let state = shared.lock().unwrap();
        assert_eq!(
            state.store.get_request("request-a").unwrap().unwrap().state,
            "open"
        );
        assert_eq!(
            state.store.get_request("request-b").unwrap().unwrap().state,
            "open"
        );
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
    fn abonnement_attach_refuse_un_pilote_sans_journal_malgre_son_mode() {
        let (mut state, config) = state_with_registered_agent("attach-tmux");
        let presence = state.presences.get_mut("instance-1").unwrap();
        presence.transport = "unix".to_string();
        presence.mode = Some(PresenceMode::Tmux);
        presence.location = Some("bridget:1.2".to_string());
        presence.journal_available = false;
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
                reason: AttachRefusal::JournalUnavailable,
                ..
            })
        ));
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn modes_reels_des_trois_enregistrements_ne_dependant_pas_du_transport() {
        let (mut state, config) = state_with_registered_agent("presence-modes");
        state.router.unregister_by_conn("conn-1");
        state.conn_instances.remove("conn-1");
        state.presences.clear();

        let registrations = [
            (
                "acp-conn",
                "acp-agent",
                "acp-instance",
                PresenceMode::Acp,
                None,
            ),
            (
                "tmux-conn",
                "tmux-agent",
                "tmux-instance",
                PresenceMode::Tmux,
                Some("bridget:3.1"),
            ),
            (
                "cli-conn",
                "cli-agent",
                "cli-instance",
                PresenceMode::Cli,
                None,
            ),
        ];
        for (conn_id, name, instance_id, mode, location) in registrations {
            assert!(matches!(
                handle_register(
                    conn_id,
                    "fixture".to_string(),
                    Some(name.to_string()),
                    Some("local".to_string()),
                    // Les trois chemins peuvent emprunter le même socket Unix :
                    // seul le mode attesté autorise ou refuse attach.
                    Some("unix".to_string()),
                    Some(mode),
                    location.map(str::to_string),
                    Some("test".to_string()),
                    Some(instance_id.to_string()),
                    None,
                    false,
                    None,
                    &mut state,
                ),
                DaemonToWrapper::Registered { .. }
            ));
        }

        let infos = state.agent_infos();
        let info = |name: &str| infos.iter().find(|agent| agent.name == name).unwrap();
        assert_eq!(info("acp-agent").mode, Some(PresenceMode::Acp));
        assert_eq!(info("tmux-agent").mode, Some(PresenceMode::Tmux));
        assert_eq!(info("tmux-agent").location.as_deref(), Some("bridget:3.1"));
        assert_eq!(info("cli-agent").mode, Some(PresenceMode::Cli));
        assert!(info("cli-agent").location.is_none());

        let tmux_refusal = attach_refusal_for_subscription(&state, "tmux-agent").unwrap_err();
        assert_eq!(tmux_refusal.reason, AttachRefusal::JournalUnavailable);
        assert_eq!(tmux_refusal.mode, Some(PresenceMode::Tmux));
        assert_eq!(tmux_refusal.location.as_deref(), Some("bridget:3.1"));
        let cli_refusal = attach_refusal_for_subscription(&state, "cli-agent").unwrap_err();
        assert_eq!(cli_refusal.reason, AttachRefusal::JournalUnavailable);
        assert_eq!(cli_refusal.mode, Some(PresenceMode::Cli));
        assert!(cli_refusal.location.is_none());

        // Un wrapper historique ne porte pas JournalReady mais son lancement
        // ACP n'atteint cette boucle qu'après activation du journal. Le
        // daemon neuf le laisse donc franchir le gate, sans exiger un
        // redémarrage atomique de tous les wrappers vivants.
        assert!(matches!(
            handle_register(
                "legacy-conn",
                "fixture".to_string(),
                Some("legacy-agent".to_string()),
                Some("local".to_string()),
                Some("acp".to_string()),
                None,
                None,
                Some("test".to_string()),
                Some("legacy-instance".to_string()),
                None,
                false,
                None,
                &mut state,
            ),
            DaemonToWrapper::Registered { .. }
        ));
        let legacy_refusal = attach_refusal_for_subscription(&state, "legacy-agent").unwrap_err();
        assert_eq!(legacy_refusal.reason, AttachRefusal::WrapperUnavailable);
        assert!(legacy_refusal.mode.is_none());
        assert!(legacy_refusal.location.is_none());

        // Mutation discriminante de la transition : un wrapper nouveau
        // annonce explicitement `false` avant JournalReady et reste refusé.
        assert!(matches!(
            handle_register(
                "modern-conn",
                "fixture".to_string(),
                Some("modern-agent".to_string()),
                Some("local".to_string()),
                Some("acp".to_string()),
                Some(PresenceMode::Acp),
                None,
                Some("test".to_string()),
                Some("modern-instance".to_string()),
                None,
                false,
                Some(false),
                &mut state,
            ),
            DaemonToWrapper::Registered { .. }
        ));
        assert_eq!(
            attach_refusal_for_subscription(&state, "modern-agent")
                .unwrap_err()
                .reason,
            AttachRefusal::JournalUnavailable
        );

        // Le journal, et non le mode, est le gate attach. Une fois attesté
        // sur tmux comme sur un pilote CLI natif, le refus progresse jusqu'à
        // la disponibilité réelle du writer : la barrière de protocole est
        // donc franchie sans inférence.
        state
            .presences
            .get_mut("tmux-instance")
            .unwrap()
            .journal_available = true;
        assert_eq!(
            attach_refusal_for_subscription(&state, "tmux-agent")
                .unwrap_err()
                .reason,
            AttachRefusal::WrapperUnavailable
        );
        state
            .presences
            .get_mut("cli-instance")
            .unwrap()
            .journal_available = true;
        assert_eq!(
            attach_refusal_for_subscription(&state, "cli-agent")
                .unwrap_err()
                .reason,
            AttachRefusal::WrapperUnavailable
        );
        let _ = std::fs::remove_file(config.db_path);
    }

    /// ORACLE PRINCIPAL — `agent_infos` ne ment plus.
    ///
    /// Un `busy` dont `capacity_seen` a dépassé le retain (wrapper ACP sans
    /// heartbeat d'autrefois) ne doit PAS réapparaître en `connected`/`unix`
    /// inventés : zéro présence fantôme dans l'annuaire, nom libéré.
    #[test]
    fn presence_expiree_ne_projette_plus_connected_unix_invente() {
        let (mut state, config) = state_with_registered_agent("fantome-expire");
        state.set_turn_state("conn-1", true).unwrap();
        let stale = Instant::now()
            .checked_sub(PRESENCE_RETENTION + Duration::from_secs(1))
            .expect("horloge");
        let presence = state.presences.get_mut("instance-1").unwrap();
        presence.capacity_seen = stale;
        presence.link_seen = stale;

        let infos = state.agent_infos();
        assert!(
            infos.iter().all(|agent| agent.name != "agent-2"),
            "le fantôme ne doit plus figurer dans l'annuaire: {infos:?}"
        );
        assert!(
            state.router.get_agent("agent-2").is_none(),
            "le nom doit être libéré du routeur après expiration de la présence"
        );
        assert!(!infos.iter().any(|agent| {
            agent.state == "connected" && agent.transport == "unix" && agent.mode.is_none()
        }));
        let _ = std::fs::remove_file(config.db_path);
    }

    /// ORACLE — à la purge de présence, une remise `dispatching` devient
    /// `orphaned` (visible), pas un `outcome_unknown` muet.
    ///
    /// Lot A : le retain lit `link_seen`. Stalifier seulement `capacity_seen`
    /// laisserait le lien frais → présence non purgée → faux vert / faux rouge.
    #[test]
    fn purge_presence_orpheline_les_remises_dispatching() {
        use crate::idempotency::{IdempotencyKey, OperationKind, Reservation, SendDelivery};

        let (mut state, config) = state_with_registered_agent("purge-orphelin");
        // busy + les DEUX horloges périmées (capacité et lien).
        state.set_turn_state("conn-1", true).unwrap();
        let stale = Instant::now()
            .checked_sub(PRESENCE_RETENTION + Duration::from_secs(1))
            .expect("horloge");
        let presence = state.presences.get_mut("instance-1").unwrap();
        presence.capacity_seen = stale;
        presence.link_seen = stale;

        let key = IdempotencyKey::new(
            "012_scope_aaaaaaaaaaaa",
            OperationKind::Send,
            "msg-jury-perdu",
        )
        .unwrap();
        assert!(matches!(
            state
                .idempotency
                .reserve(&key, b"canon-jury", 1_000_000, 3600, 1_000_000, 30),
            Ok(Reservation::Prepared { .. })
        ));
        let mut message = BridgetMessage::new("bridget", "agent-2", "mandat de jury");
        message.id = "msg-jury-perdu".to_string();
        let message_bytes = serde_json::to_vec(&message).unwrap();
        state
            .idempotency
            .begin_send_delivery(
                &key,
                &SendDelivery {
                    delivery_id: "delivery-jury".to_string(),
                    recipient_instance_id: "instance-1".to_string(),
                    delivery_generation: 1,
                    expires_at: 1_000_000 + 3600,
                    message_bytes,
                },
            )
            .unwrap();

        let _ = state.agent_infos();
        assert!(
            !state.presences.contains_key("instance-1"),
            "présence doit être purgée"
        );
        assert_eq!(
            state
                .idempotency
                .dispatching_deliveries_for_instance("instance-1", 1_000_000)
                .unwrap()
                .len(),
            0,
            "plus de dispatching après purge"
        );
        assert_eq!(
            state
                .idempotency
                .orphaned_delivery_id(&key)
                .unwrap()
                .as_deref(),
            Some("delivery-jury")
        );
        assert_eq!(
            state.idempotency.lookup(&key, 1_000_000).unwrap(),
            crate::idempotency::LookupResult::Orphaned {
                expires_at: 1_000_000 + 3600,
                reason: "destinataire purgé — présence absente ; remise orpheline".to_string(),
            }
        );
        let _ = std::fs::remove_file(config.db_path);
    }

    /// ORACLE — moitié « pas en silence » : un Deliver ORPHELIN atteint l'émetteur.
    /// Meurt si l'on retire le push (ou la notice durable) tout en gardant la phase.
    #[test]
    fn purge_presence_delivre_un_orphelin_a_l_emetteur() {
        use crate::idempotency::{IdempotencyKey, OperationKind, Reservation, SendDelivery};
        use std::io::Read;
        use std::os::unix::net::UnixStream;

        let (mut state, config) = state_with_registered_agent("purge-deliver");
        // Émetteur « bridget » avec une vraie connexion lisible.
        state
            .router
            .register(
                Some("bridget"),
                &bridget_core::AgentType::Claude,
                "conn-emitter",
            )
            .unwrap();
        state
            .conn_instances
            .insert("conn-emitter".to_string(), "instance-emitter".to_string());
        state.presences.insert(
            "instance-emitter".to_string(),
            Presence {
                name: "bridget".to_string(),
                agent_type: "claude".to_string(),
                host: "macbook".to_string(),
                transport: "acp".to_string(),
                channel: None,
                mode: Some(PresenceMode::Acp),
                location: None,
                journal_available: true,
                os: "macOS".to_string(),
                state: "connected".to_string(),
                busy_since: None,
                capacity_seen: Instant::now(),
                link_seen: Instant::now(),
                reconnect_count: 0,
                model: None,
                effort: None,
                rate_limits: Default::default(),
                served_model: None,
                derived_domain: None,
                domain: None,
                dnd_until: None,
            },
        );
        let (writer_stream, mut peer) = UnixStream::pair().unwrap();
        peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        state.connections.insert(
            "conn-emitter".to_string(),
            Arc::new(Mutex::new(BufWriter::new(writer_stream))),
        );

        state.set_turn_state("conn-1", true).unwrap();
        let stale = Instant::now()
            .checked_sub(PRESENCE_RETENTION + Duration::from_secs(1))
            .expect("horloge");
        let presence = state.presences.get_mut("instance-1").unwrap();
        presence.capacity_seen = stale;
        presence.link_seen = stale;

        let key = IdempotencyKey::new(
            "012_scope_aaaaaaaaaaaa",
            OperationKind::Send,
            "msg-a-notifier",
        )
        .unwrap();
        assert!(matches!(
            state
                .idempotency
                .reserve(&key, b"canon-notify", 1_000_000, 3600, 1_000_000, 30),
            Ok(Reservation::Prepared { .. })
        ));
        let mut message = BridgetMessage::new("bridget", "agent-2", "corps");
        message.id = "msg-a-notifier".to_string();
        state
            .idempotency
            .begin_send_delivery(
                &key,
                &SendDelivery {
                    delivery_id: "delivery-notify".to_string(),
                    recipient_instance_id: "instance-1".to_string(),
                    delivery_generation: 1,
                    expires_at: 1_000_000 + 3600,
                    message_bytes: serde_json::to_vec(&message).unwrap(),
                },
            )
            .unwrap();

        let _ = state.agent_infos();

        let mut buf = [0u8; 4096];
        let n = peer
            .read(&mut buf)
            .expect("Deliver ORPHELIN attendu sur la socket émetteur");
        let received = String::from_utf8_lossy(&buf[..n]);
        assert!(
            received.contains("ORPHELIN") && received.contains("msg-a-notifier"),
            "la moitié « pas en silence » exige un Deliver lisible: {received}"
        );
        assert!(
            received.contains("clé est close") && received.contains("Change de destinataire"),
            "le signal doit porter la conduite (pas seulement le constat): {received}"
        );
        assert!(
            state
                .idempotency
                .pending_orphan_emitter_notices()
                .unwrap()
                .is_empty(),
            "après Deliver réussi, plus aucune notice en attente"
        );
        let _ = std::fs::remove_file(config.db_path);
    }

    /// ORACLE — prouve que `flush_pending_orphan_emitter_notices` pousse une
    /// notice en attente dès que l'émetteur a une connexion lisible.
    ///
    /// **Ce qu'il ne prouve PAS** : que `handle_connection` appelle ce flush
    /// après `Registered`. Ce branchement vit dans la boucle socket ; aucun
    /// test unitaire ne l'atteint. Retirer `if registered_just_now { … }`
    /// laisse cet oracle vert — trou déclaré, pas maquillé. Un témoin d'intégration
    /// (vrai Register sur socket) fermerait le trou ; hors périmètre immédiat.
    #[test]
    fn register_emetteur_rejoue_les_notices_orphelines_en_attente() {
        use crate::idempotency::{IdempotencyKey, OperationKind, Reservation, SendDelivery};
        use std::io::Read;
        use std::os::unix::net::UnixStream;

        let (mut state, config) = state_with_registered_agent("purge-register-replay");
        // Remise dispatching vers le destinataire qui va être purgé.
        let key = IdempotencyKey::new(
            "012_scope_aaaaaaaaaaaa",
            OperationKind::Send,
            "msg-register-replay",
        )
        .unwrap();
        assert!(matches!(
            state
                .idempotency
                .reserve(&key, b"canon-reg", 1_000_000, 3600, 1_000_000, 30),
            Ok(Reservation::Prepared { .. })
        ));
        let mut message = BridgetMessage::new("bridget", "agent-2", "corps");
        message.id = "msg-register-replay".to_string();
        state
            .idempotency
            .begin_send_delivery(
                &key,
                &SendDelivery {
                    delivery_id: "delivery-register-replay".to_string(),
                    recipient_instance_id: "instance-1".to_string(),
                    delivery_generation: 1,
                    expires_at: 1_000_000 + 3600,
                    message_bytes: serde_json::to_vec(&message).unwrap(),
                },
            )
            .unwrap();

        // Purge sans émetteur en ligne → notice durable, Deliver impossible.
        state.set_turn_state("conn-1", true).unwrap();
        let stale = Instant::now()
            .checked_sub(PRESENCE_RETENTION + Duration::from_secs(1))
            .expect("horloge");
        let presence = state.presences.get_mut("instance-1").unwrap();
        presence.capacity_seen = stale;
        presence.link_seen = stale;
        let _ = state.agent_infos();
        assert_eq!(
            state
                .idempotency
                .pending_orphan_emitter_notices()
                .unwrap()
                .len(),
            1,
            "émetteur absent ⇒ notice reste en attente"
        );

        // L'émetteur revient en ligne (présence + socket). On appelle le flush
        // directement — prouve la fonction, pas le branchement Register
        // (voir doc de l'oracle : trou déclaré).
        state
            .router
            .register(
                Some("bridget"),
                &bridget_core::AgentType::Claude,
                "conn-emitter",
            )
            .unwrap();
        state
            .conn_instances
            .insert("conn-emitter".to_string(), "instance-emitter".to_string());
        state.presences.insert(
            "instance-emitter".to_string(),
            Presence {
                name: "bridget".to_string(),
                agent_type: "claude".to_string(),
                host: "macbook".to_string(),
                transport: "acp".to_string(),
                channel: None,
                mode: Some(PresenceMode::Acp),
                location: None,
                journal_available: true,
                os: "macOS".to_string(),
                state: "connected".to_string(),
                busy_since: None,
                capacity_seen: Instant::now(),
                link_seen: Instant::now(),
                reconnect_count: 0,
                model: None,
                effort: None,
                rate_limits: Default::default(),
                served_model: None,
                derived_domain: None,
                domain: None,
                dnd_until: None,
            },
        );
        let (writer_stream, mut peer) = UnixStream::pair().unwrap();
        peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        state.connections.insert(
            "conn-emitter".to_string(),
            Arc::new(Mutex::new(BufWriter::new(writer_stream))),
        );

        state.flush_pending_orphan_emitter_notices();

        let mut buf = [0u8; 4096];
        let n = peer
            .read(&mut buf)
            .expect("Deliver ORPHELIN attendu au retour de l'émetteur");
        let received = String::from_utf8_lossy(&buf[..n]);
        assert!(
            received.contains("ORPHELIN") && received.contains("msg-register-replay"),
            "Register émetteur doit rejouer la notice: {received}"
        );
        assert!(
            received.contains("clé est close"),
            "rejeu Register porte aussi la conduite: {received}"
        );
        assert!(
            state
                .idempotency
                .pending_orphan_emitter_notices()
                .unwrap()
                .is_empty()
        );
        let _ = std::fs::remove_file(config.db_path);
    }

    /// ORACLE — mort du wrapper (unregister + unreachable) → zéro présence live.
    #[test]
    fn mort_du_wrapper_emporte_la_presence_du_routeur() {
        let (mut state, config) = state_with_registered_agent("mort-wrapper");
        assert!(state.router.get_agent("agent-2").is_some());
        state.router.unregister_by_conn("conn-1");
        state.mark_unreachable("conn-1");

        let infos = state.agent_infos();
        assert!(state.router.get_agent("agent-2").is_none());
        assert!(
            !infos
                .iter()
                .any(|agent| agent.name == "agent-2" && agent.state == "connected"),
            "pas de connected résiduel: {infos:?}"
        );
        let unreachable = infos
            .iter()
            .find(|agent| agent.name == "agent-2")
            .expect("l'état unreachable reste listé distinctement");
        assert_eq!(unreachable.state, "unreachable");
        let _ = std::fs::remove_file(config.db_path);
    }

    /// ORACLE — takeover sur fantôme sans stop préalable.
    #[test]
    fn takeover_sur_fantome_reussit_sans_stop_prealable() {
        let (mut state, config) = state_with_registered_agent("takeover-fantome");
        // Simule le fantôme : présence expirée, nom encore au routeur avant
        // le premier agent_infos (ou un résidu après crash de retain).
        state.presences.clear();
        // conn_instances pointe vers une instance absente → dangling.
        assert!(state.router.get_agent("agent-2").is_some());

        let registered = handle_register(
            "conn-takeover",
            "cursor".to_string(),
            Some("agent-2".to_string()),
            Some("local".to_string()),
            Some("acp".to_string()),
            Some(PresenceMode::Acp),
            None,
            Some("macOS".to_string()),
            Some("instance-takeover".to_string()),
            Some("bridget".to_string()),
            false,
            Some(true),
            &mut state,
        );
        assert!(
            matches!(
                registered,
                DaemonToWrapper::Registered { ref name } if name == "agent-2"
            ),
            "takeover refusé: {registered:?}"
        );
        let infos = state.agent_infos();
        let info = infos
            .iter()
            .find(|agent| agent.name == "agent-2")
            .expect("agent repris");
        assert_eq!(info.state, "connected");
        assert_eq!(info.transport, "acp");
        assert_eq!(info.mode, Some(PresenceMode::Acp));
        let _ = std::fs::remove_file(config.db_path);
    }

    /// Contre-épreuve : un agent vraiment live refuse le reclaim.
    #[test]
    fn takeover_refuse_un_nom_encore_live() {
        let (mut state, config) = state_with_registered_agent("takeover-live");
        let refused = handle_register(
            "conn-intrus",
            "cursor".to_string(),
            Some("agent-2".to_string()),
            Some("local".to_string()),
            Some("acp".to_string()),
            Some(PresenceMode::Acp),
            None,
            Some("macOS".to_string()),
            Some("instance-intrus".to_string()),
            None,
            false,
            Some(true),
            &mut state,
        );
        assert!(
            matches!(refused, DaemonToWrapper::Nack { .. }),
            "un live ne doit pas être spolié: {refused:?}"
        );
        let _ = std::fs::remove_file(config.db_path);
    }

    /// ORACLE — Heartbeat rafraîchit le LIEN, jamais la capacité ni l'état.
    ///
    /// Sans la séparation : un wrapper vivant / shell mort garde `capacity_seen`
    /// frais et who / bridget-idle le comptent LIBRE. busy doit rester busy ;
    /// `capacity_seen` (capacité) ne bouge pas sous heartbeat seul.
    #[test]
    fn heartbeat_preserve_etat_metier() {
        let (mut state, config) = state_with_registered_agent("heartbeat-busy");
        state.set_turn_state("conn-1", true).unwrap();
        assert_eq!(state.presences.get("instance-1").unwrap().state, "busy");
        let before_capacity = state.presences.get("instance-1").unwrap().capacity_seen;
        let before_link = state.presences.get("instance-1").unwrap().link_seen;
        std::thread::sleep(Duration::from_millis(5));
        let shared = Arc::new(Mutex::new(state));
        assert!(handle_wrapper_message("conn-1", WrapperToDaemon::Heartbeat, &shared).is_none());
        let st = shared.lock().unwrap();
        let presence = st.presences.get("instance-1").unwrap();
        assert_eq!(
            presence.state, "busy",
            "Heartbeat ne doit jamais écraser l'état métier (busy→connected serait un mensonge)"
        );
        assert_eq!(
            presence.capacity_seen, before_capacity,
            "Heartbeat ne doit pas rafraîchir la capacité (capacity_seen)"
        );
        assert!(
            presence.link_seen > before_link,
            "Heartbeat doit rafraîchir le lien (link_seen), sinon le retain jette un busy live"
        );
        let _ = std::fs::remove_file(&config.db_path);
    }

    /// ORACLE lot A — zombie : wrapper qui heartbeate sans capacité.
    /// Contrôle positif : une capacité fraîche reste visible.
    /// Mutant : si Heartbeat remettait capacity_seen à jour, last_seen_secs
    /// redeviendrait ~0 et l'assertion « capacité gelée » tomberait.
    #[test]
    fn heartbeat_ne_rajeunit_pas_une_capacite_morte() {
        let (mut state, config) = state_with_registered_agent("heartbeat-zombie");
        let stale = Instant::now()
            .checked_sub(Duration::from_secs(1900))
            .expect("horloge");
        {
            let presence = state.presences.get_mut("instance-1").unwrap();
            presence.state = "connected".to_string();
            presence.capacity_seen = stale;
            // Lien encore frais (socket vivant) — c'est le cas nominal zombie.
            presence.link_seen = Instant::now();
        }
        let shared = Arc::new(Mutex::new(state));
        for _ in 0..3 {
            assert!(
                handle_wrapper_message("conn-1", WrapperToDaemon::Heartbeat, &shared).is_none()
            );
        }
        let mut st = shared.lock().unwrap();
        let presence = st
            .presences
            .get("instance-1")
            .expect("lien garde la présence");
        assert!(
            presence.capacity_seen.elapsed() >= Duration::from_secs(1800),
            "la capacité doit rester gelée sous heartbeat seul: {:?}",
            presence.capacity_seen.elapsed()
        );
        let infos = st.agent_infos();
        let agent = infos
            .iter()
            .find(|agent| agent.name == "agent-2")
            .expect("contrôle positif : présence légitime (lien frais) reste à l'annuaire");
        assert!(
            agent.last_seen_secs >= 1800,
            "who doit exposer l'âge de CAPACITÉ, pas du lien: last_seen_secs={}",
            agent.last_seen_secs
        );
        let _ = std::fs::remove_file(&config.db_path);
    }

    /// Contrôle positif lot A : un TurnState (capacité) rafraîchit capacity_seen.
    #[test]
    fn capacite_vraie_rafraichit_last_seen() {
        let (mut state, config) = state_with_registered_agent("capacite-vraie");
        let stale = Instant::now()
            .checked_sub(Duration::from_secs(1900))
            .expect("horloge");
        state.presences.get_mut("instance-1").unwrap().capacity_seen = stale;
        state.presences.get_mut("instance-1").unwrap().link_seen = stale;
        std::thread::sleep(Duration::from_millis(5));
        state.set_turn_state("conn-1", false).unwrap();
        let presence = state.presences.get("instance-1").unwrap();
        assert!(
            presence.capacity_seen.elapsed() < Duration::from_secs(2),
            "TurnState doit attestier une capacité fraîche"
        );
        assert!(
            presence.link_seen.elapsed() < Duration::from_secs(2),
            "une capacité touche aussi le lien"
        );
        let infos = state.agent_infos();
        let agent = infos
            .iter()
            .find(|agent| agent.name == "agent-2")
            .expect("présence légitime visible");
        assert!(
            agent.last_seen_secs < 2,
            "contrôle positif : capacité fraîche ⇒ last_seen_secs bas ({})",
            agent.last_seen_secs
        );
        let _ = std::fs::remove_file(&config.db_path);
    }

    /// ORACLE lot B — busy jury composé (A+B) : capacité gelée 1 h, heartbeat
    /// seul → survit (retain = horloge lien).
    #[test]
    fn lot_b_busy_jury_survit_sous_heartbeat_malgre_capacite_gelee() {
        let (mut state, config) = state_with_registered_agent("lotb-busy-compose");
        state.set_turn_state("conn-1", true).unwrap();
        let capacity_stale = Instant::now()
            .checked_sub(Duration::from_secs(3600))
            .expect("horloge");
        {
            let p = state.presences.get_mut("instance-1").unwrap();
            p.capacity_seen = capacity_stale;
            p.link_seen = capacity_stale;
        }
        let shared = Arc::new(Mutex::new(state));
        assert!(handle_wrapper_message("conn-1", WrapperToDaemon::Heartbeat, &shared).is_none());
        let mut st = shared.lock().unwrap();
        let p = st.presences.get("instance-1").expect("présence");
        assert!(
            p.capacity_seen.elapsed() >= Duration::from_secs(3500),
            "capacité doit rester gelée sous heartbeat seul"
        );
        assert!(
            presence_within_retention(p),
            "busy + lien frais ⇒ keep_b_compose"
        );
        let infos = st.agent_infos();
        assert!(
            infos
                .iter()
                .any(|a| a.name == "agent-2" && a.state == "busy"),
            "busy jury ne doit pas être purgé: {infos:?}"
        );
        let _ = std::fs::remove_file(&config.db_path);
    }

    /// ORACLE — le vrai CLI Maicie sélectionne le busy libéré via ListAgents,
    /// délègue, envoie idempotemment, puis le mandat arrive au wrapper.
    #[test]
    fn tour_non_abouti_redevient_mandatable_et_le_mandat_parvient() {
        let (mut state, config) = state_with_registered_agent("busy-tour-non-abouti");
        state.set_turn_state("conn-1", true).unwrap();
        let ttl = live_notify_timeout_secs(&state.registry, "claude")
            .saturating_add(TIMEOUT_GRACE_PERIOD);
        let aged = Instant::now()
            .checked_sub(Duration::from_secs(ttl.saturating_add(1)))
            .expect("horloge busy_since");
        state.presences.get_mut("instance-1").unwrap().busy_since = Some(aged);

        let (writer_stream, peer) = UnixStream::pair().unwrap();
        peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        state.connections.insert(
            "conn-1".to_string(),
            Arc::new(Mutex::new(BufWriter::new(writer_stream))),
        );
        assert_eq!(state.next_conn_id(), "conn-1");
        let shared = Arc::new(Mutex::new(state));
        let listener = UnixListener::bind(&config.socket_path).unwrap();
        listener.set_nonblocking(true).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let server_stop = Arc::clone(&stop);
        let server_state = Arc::clone(&shared);
        let server = thread::spawn(move || {
            let mut connections = Vec::new();
            while !server_stop.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let connection_state = Arc::clone(&server_state);
                        connections.push(thread::spawn(move || {
                            let _ = handle_connection(stream, connection_state);
                        }));
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("accept Maicie: {error}"),
                }
            }
            for connection in connections {
                connection.join().unwrap();
            }
        });

        let root = std::env::temp_dir().join(format!(
            "maicie-busy-oracle-{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
        let maicie_config = root.join("maicie.json");
        std::fs::write(
            &maicie_config,
            serde_json::to_vec(&serde_json::json!({
                "version": 1,
                "bridget_socket": config.socket_path,
                "database_path": root.join("maicie.sqlite3"),
                "durations": {"short_secs": 30, "normal_secs": 60, "long_secs": 90},
                "profiles": [{
                    "id": "agent-2",
                    "display_name": "Agent 2",
                    "tags": [],
                    "personality_ref": "profiles/agent-2.md",
                    "tools": ["bridget_send"],
                    "spawn_order_ref": "agents/agent-2"
                }]
            }))
            .unwrap(),
        )
        .unwrap();

        const MANDAT_BODY: &str = "MANDAT-BUSY-RECOVERY-ORACLE-98beefe0";
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let output = Command::new("cargo")
            .current_dir(workspace)
            .args([
                "run", "--quiet", "-p", "maicie", "--", "delegate", "--config",
            ])
            .arg(&maicie_config)
            .args([
                "--goal",
                MANDAT_BODY,
                "--suite",
                "aucune",
                "--to",
                "agent-2",
                "--duration",
                "courte",
                "--idempotency-key",
                "busy-recovery-integrated-oracle",
                "--json",
            ])
            .output()
            .unwrap();
        stop.store(true, Ordering::SeqCst);
        server.join().unwrap();

        let mut line = String::new();
        let arrival = BufReader::new(peer).read_line(&mut line);
        assert!(
            arrival.is_ok() && line.contains(MANDAT_BODY),
            "mandat absent après sélection Maicie réelle; lecture={arrival:?}; stdout={}; stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            output.status.success(),
            "Maicie a échoué après livraison: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let _ = std::fs::remove_file(&config.socket_path);
        let _ = std::fs::remove_file(&config.db_path);
        let _ = std::fs::remove_dir_all(root);
    }

    const MAX_NOTIFY_TIMEOUT_CHILD_ENV: &str = "BRIDGET_MAX_NOTIFY_TIMEOUT_CHILD";

    /// Frontière du registre : `u64::MAX` ne doit jamais faire paniquer
    /// `agent_infos` lors du calcul notify_timeout + grâce.
    #[test]
    fn notify_timeout_max_ne_panique_pas_dans_agent_infos() {
        if std::env::var(MAX_NOTIFY_TIMEOUT_CHILD_ENV).ok().as_deref() == Some("1") {
            let (mut state, config) = state_with_registered_agent("max-notify-timeout-child");
            state.set_turn_state("conn-1", true).unwrap();
            let infos = state.agent_infos();
            assert_eq!(infos[0].state, "busy");
            let _ = std::fs::remove_file(config.db_path);
            return;
        }

        let root = std::env::temp_dir().join(format!(
            "bridget-max-notify-timeout-{}",
            uuid::Uuid::new_v4().simple()
        ));
        let registry_dir = root.join(".config/bridget");
        std::fs::create_dir_all(&registry_dir).unwrap();
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
        let registry_path = registry_dir.join("agents.json");
        std::fs::write(
            &registry_path,
            serde_json::to_vec(&serde_json::json!({
                "agents": {
                    "claude": {
                        "command": "/bin/true",
                        "protocol": "acp",
                        "forbidden_env": [],
                        "pass_env": [],
                        "notify_timeout_secs": u64::MAX
                    }
                }
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::set_permissions(&registry_path, std::fs::Permissions::from_mode(0o600)).unwrap();

        let output = Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("daemon::presence_tests::notify_timeout_max_ne_panique_pas_dans_agent_infos")
            .arg("--nocapture")
            .env("HOME", &root)
            .env(MAX_NOTIFY_TIMEOUT_CHILD_ENV, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "agent_infos a paniqué à u64::MAX: stdout={}; stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let _ = std::fs::remove_dir_all(root);
    }

    /// ORACLE lot B — contrôle positif : capacité/lien frais → survit
    /// (ne pas remplacer l'immortalité par la mortalité universelle).
    #[test]
    fn lot_b_presence_fraiche_survit_au_retain() {
        let (mut state, config) = state_with_registered_agent("lotb-frais");
        state.presences.get_mut("instance-1").unwrap().capacity_seen = Instant::now();
        state.presences.get_mut("instance-1").unwrap().link_seen = Instant::now();
        {
            let p = state.presences.get("instance-1").unwrap();
            assert!(
                presence_within_retention(p),
                "capacité/lien frais ⇒ keep_b_compose"
            );
        }
        let infos = state.agent_infos();
        assert!(
            infos.iter().any(|a| a.name == "agent-2"),
            "présence fraîche doit survivre: {infos:?}"
        );
        let _ = std::fs::remove_file(config.db_path);
    }

    /// ORACLE lot B — connected mort (relec1/relec5) : deux horloges
    /// périmées → purgé sans redémarrage. Mutant : exemption connected.
    #[test]
    fn lot_b_connected_mort_est_purge_sans_redemarrage() {
        let (mut state, config) = state_with_registered_agent("lotb-connected-mort");
        let stale = Instant::now()
            .checked_sub(PRESENCE_RETENTION + Duration::from_secs(3600))
            .expect("horloge");
        {
            let p = state.presences.get_mut("instance-1").unwrap();
            p.state = "connected".to_string();
            p.capacity_seen = stale;
            p.link_seen = stale;
            assert!(
                presence_within_retention_mutant_exempt_connected(p),
                "mutant exemption garderait le mort"
            );
            assert!(
                !presence_within_retention(p),
                "connected mort ⇒ keep_b_compose=false"
            );
        }
        let infos = state.agent_infos();
        assert!(
            infos.iter().all(|a| a.name != "agent-2"),
            "connected mort doit disparaître sans redémarrage: {infos:?}"
        );
        assert!(!state.presences.contains_key("instance-1"));
        assert!(state.router.get_agent("agent-2").is_none());
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn journal_ready_atteste_le_gate_attach_apres_register() {
        let (mut state, config) = state_with_registered_agent("attach-journal-ready");
        state
            .presences
            .get_mut("instance-1")
            .unwrap()
            .journal_available = false;
        let shared = Arc::new(Mutex::new(state));

        assert_eq!(
            attach_refusal_for_subscription(&shared.lock().unwrap(), "agent-2")
                .unwrap_err()
                .reason,
            AttachRefusal::JournalUnavailable
        );
        assert!(handle_wrapper_message("conn-1", WrapperToDaemon::JournalReady, &shared).is_none());
        assert!(
            shared
                .lock()
                .unwrap()
                .presences
                .get("instance-1")
                .unwrap()
                .journal_available
        );
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn wrapper_acp_historique_garde_son_journal_apres_reconnexion_vers_daemon_neuf() {
        let (mut state, config) = state_with_registered_agent("legacy-journal-reconnect");
        assert!(matches!(
            handle_register(
                "legacy-wrapper",
                "codex".to_string(),
                Some("legacy-journal".to_string()),
                Some("local".to_string()),
                Some("acp".to_string()),
                None,
                None,
                Some("test".to_string()),
                Some("legacy-journal-instance".to_string()),
                None,
                false,
                None,
                &mut state,
            ),
            DaemonToWrapper::Registered { .. }
        ));
        assert!(
            state
                .presences
                .get("legacy-journal-instance")
                .expect("présence historique")
                .journal_available,
            "mutation discriminante : sans la compatibilité de version, le gate retombe à faux"
        );
        assert_eq!(
            attach_refusal_for_subscription(&state, "legacy-journal")
                .unwrap_err()
                .reason,
            AttachRefusal::WrapperUnavailable
        );
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn wrapper_recentre_reste_non_attachable_jusqu_a_journal_ready() {
        let (mut state, config) = state_with_registered_agent("modern-journal-register");
        assert!(matches!(
            handle_register(
                "modern-wrapper",
                "codex".to_string(),
                Some("modern-journal".to_string()),
                Some("local".to_string()),
                Some("acp".to_string()),
                Some(PresenceMode::Acp),
                None,
                Some("test".to_string()),
                Some("modern-journal-instance".to_string()),
                None,
                false,
                Some(false),
                &mut state,
            ),
            DaemonToWrapper::Registered { .. }
        ));
        assert_eq!(
            attach_refusal_for_subscription(&state, "modern-journal")
                .unwrap_err()
                .reason,
            AttachRefusal::JournalUnavailable,
            "mutation discriminante : confondre l'absence historique et false réintroduirait le gate ACP"
        );
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn enregistrement_auxiliaire_mcp_ne_revendique_pas_la_presence_du_wrapper_vivant() {
        let (mut state, config) = state_with_registered_agent("presence-mcp-fusion");
        let rich = state.presences.get_mut("instance-1").unwrap();
        rich.domain = Some("coordination".to_string());
        rich.derived_domain = Some("coordination".to_string());
        rich.model = Some("gpt-5.6-terra".to_string());
        rich.effort = Some("high".to_string());

        assert!(matches!(
            handle_register(
                "mcp-child",
                "mcp".to_string(),
                Some("agent-2".to_string()),
                None,
                None,
                Some(PresenceMode::Cli),
                None,
                None,
                Some("instance-1".to_string()),
                None,
                false,
                None,
                &mut state,
            ),
            DaemonToWrapper::Registered { .. }
        ));
        assert_eq!(
            state.conn_names.get("mcp-child").map(String::as_str),
            Some("agent-2")
        );
        assert_eq!(
            state.conn_instances.get("mcp-child").map(String::as_str),
            Some("instance-1")
        );
        assert!(state.auxiliary_connections.contains("mcp-child"));

        // Fermeture de la connexion utilisée par l'outil MCP : elle ne doit
        // ni voler l'instance, ni rendre le wrapper principal inaccessible.
        state.router.unregister_by_conn("mcp-child");
        state.mark_unreachable("mcp-child");
        let agents = state.agent_infos();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "agent-2");
        assert_eq!(agents[0].transport, "acp");
        assert_eq!(agents[0].mode, Some(PresenceMode::Acp));
        assert_eq!(agents[0].domain.as_deref(), Some("coordination"));
        assert_eq!(agents[0].model.as_deref(), Some("gpt-5.6-terra"));
        assert_eq!(agents[0].effort.as_deref(), Some("high"));
        assert_eq!(agents[0].state, "connected");

        assert!(matches!(
            handle_register(
                "mcp-child-2",
                "mcp".to_string(),
                Some("agent-2".to_string()),
                None,
                None,
                Some(PresenceMode::Cli),
                None,
                None,
                Some("instance-1".to_string()),
                None,
                false,
                None,
                &mut state,
            ),
            DaemonToWrapper::Registered { .. }
        ));
        state.mark_unreachable("conn-1");
        assert_eq!(
            state.presences.get("instance-1").unwrap().state,
            "unreachable",
            "une connexion auxiliaire ne masque jamais la perte du wrapper propriétaire"
        );
        state.mark_unreachable("mcp-child-2");
        assert_eq!(
            state.presences.get("instance-1").unwrap().state,
            "unreachable"
        );

        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn gere_acp_de_type_inconnu_tient_mode_et_runtime_de_sa_definition_figee() {
        let (mut state, config) = state_with_registered_agent("managed-runtime-definition");
        state.router.unregister_by_conn("conn-1");
        state.conn_instances.remove("conn-1");
        state.presences.clear();
        state.registry = AgentRegistry::from_json(
            r#"{"agents":{"codex-terra":{"command":"fixture-acp","args":["-c","model=\"gpt-5.6-terra\"","-c","model_reasoning_effort=\"high\""],"protocol":"acp","capabilities":{"execution_paths":["acp"],"models":{"gpt-5.6-terra":{"efforts":["high"]}}}}}}"#,
            "/tmp/agents.json",
        )
        .unwrap();
        let definition = state.registry.resolved_definition("codex-terra").unwrap();
        let now = unix_timestamp();
        let order = FleetSpawnOrder {
            agent_type: "codex-terra".to_string(),
            requested_name: Some("coder-terra".to_string()),
            cwd: PathBuf::from("/tmp"),
            persistent: false,
            command_id: "managed-runtime-definition".to_string(),
            issued_at: now,
            deadline_at: now + 60,
        };
        let lease = match state.fleet.request_spawn(&order, now).unwrap() {
            crate::fleet::SpawnSubmission::Start(lease) => lease,
            other => panic!("réservation inattendue: {other:?}"),
        };
        state.fleet.mark_starting(&lease, now, &definition).unwrap();
        state
            .managed_by_instance
            .insert(lease.instance_id.clone(), lease.command_id.clone());
        let instance_id = lease.instance_id.clone();
        let (wrapper_writer, _wrapper_reader) = control_socket("managed-terra-wrapper");
        state
            .connections
            .insert("managed-terra".to_string(), wrapper_writer);

        assert!(matches!(
            handle_register(
                "managed-terra",
                "codex-terra".to_string(),
                Some("coder-terra".to_string()),
                Some("local".to_string()),
                // Mutation discriminante : un wrapper historique ne connaît
                // pas PresenceMode et annonce seulement son canal Unix. La
                // définition gérée `protocol=acp` doit rester l'autorité.
                Some("unix".to_string()),
                None,
                None,
                Some("macOS".to_string()),
                Some(instance_id.clone()),
                None,
                false,
                None,
                &mut state,
            ),
            DaemonToWrapper::Registered { .. }
        ));
        state
            .fleet
            .register_connected(&lease, &instance_id, now + 1)
            .unwrap();
        let agent = state.agent_infos().pop().expect("géré visible");
        assert_eq!(agent.name, "coder-terra");
        assert_eq!(agent.transport, "acp");
        assert_eq!(agent.channel.as_deref(), Some("unix"));
        assert_eq!(agent.mode, Some(PresenceMode::Acp));
        assert_eq!(agent.model.as_deref(), Some("gpt-5.6-terra"));
        assert_eq!(agent.effort.as_deref(), Some("high"));
        assert_eq!(
            attach_refusal_for_subscription(&state, "coder-terra").unwrap(),
            "managed-terra"
        );

        let _ = std::fs::remove_file(config.db_path);
    }

    fn install_gere_acp_pour_reconnexion(state: &mut DaemonState, label: &str) -> SpawnLease {
        state.registry = AgentRegistry::from_json(
            r#"{"agents":{"cursor":{"command":"cursor-agent","args":["acp"],"protocol":"acp"}}}"#,
            "/tmp/agents-cursor.json",
        )
        .unwrap();
        let definition = state.registry.resolved_definition("cursor").unwrap();
        let now = unix_timestamp();
        let order = FleetSpawnOrder {
            agent_type: "cursor".to_string(),
            requested_name: Some("cursor5".to_string()),
            cwd: PathBuf::from("/tmp"),
            persistent: false,
            command_id: label.to_string(),
            issued_at: now,
            deadline_at: now + 60,
        };
        let lease = match state.fleet.request_spawn(&order, now).unwrap() {
            crate::fleet::SpawnSubmission::Start(lease) => lease,
            other => panic!("réservation inattendue: {other:?}"),
        };
        state.fleet.mark_starting(&lease, now, &definition).unwrap();
        state
            .managed_by_instance
            .insert(lease.instance_id.clone(), lease.command_id.clone());
        state.managed_spawns.insert(
            lease.command_id.clone(),
            ManagedSpawnRecord {
                lease: lease.clone(),
                agent_type: "cursor".to_string(),
                requester_conns: Vec::new(),
                wrapper_conn: None,
                stop: Arc::new(ManagedStopControl::new()),
            },
        );
        let (wrapper_writer, _wrapper_reader) = control_socket(label);
        state.connections.insert(label.to_string(), wrapper_writer);
        lease
    }

    #[test]
    fn gere_acp_reconnecte_sans_champs_wrapper_tient_mode_et_domaine() {
        let (mut state, config) = state_with_registered_agent("managed-reconnect-hostile");
        state.router.unregister_by_conn("conn-1");
        state.conn_instances.remove("conn-1");
        state.presences.clear();
        let lease = install_gere_acp_pour_reconnexion(&mut state, "managed-reconnect-hostile");
        let instance_id = lease.instance_id.clone();

        assert!(matches!(
            handle_register(
                "managed-reconnect-hostile",
                "cursor".to_string(),
                Some("cursor5".to_string()),
                Some("local".to_string()),
                Some("unix".to_string()),
                None,
                None,
                Some("macOS".to_string()),
                Some(instance_id.clone()),
                Some("bridget".to_string()),
                false,
                None,
                &mut state,
            ),
            DaemonToWrapper::Registered { .. }
        ));
        let first = state.agent_infos().pop().expect("inscription initiale");
        assert_eq!(first.mode, Some(PresenceMode::Acp));
        assert_eq!(first.domain.as_deref(), Some("bridget"));
        assert_eq!(first.reconnect_count, 0);

        state.router.unregister_by_conn("managed-reconnect-hostile");
        state.mark_unreachable("managed-reconnect-hostile");
        // Mutation discriminante : la ré-inscription production d'un cursor
        // déjà connu ne retrouve plus l'instance dans managed_by_instance
        // (reprise = nouvel identifiant, ou carte instance périmée).
        state.managed_by_instance.remove(&instance_id);

        let (wrapper_writer, _wrapper_reader) = control_socket("managed-reconnect-hostile-2");
        state
            .connections
            .insert("managed-reconnect-hostile-2".to_string(), wrapper_writer);
        assert!(matches!(
            handle_register(
                "managed-reconnect-hostile-2",
                "cursor".to_string(),
                Some("cursor5".to_string()),
                Some("local".to_string()),
                Some("unix".to_string()),
                None,
                None,
                Some("macOS".to_string()),
                Some(instance_id.clone()),
                None,
                true,
                None,
                &mut state,
            ),
            DaemonToWrapper::Registered { .. }
        ));
        let reconnected = state.agent_infos().pop().expect("réinscrit");
        assert_eq!(reconnected.name, "cursor5");
        assert_eq!(reconnected.transport, "acp");
        assert_eq!(reconnected.mode, Some(PresenceMode::Acp));
        assert_eq!(reconnected.domain.as_deref(), Some("bridget"));
        assert!(
            reconnected.reconnect_count >= 1,
            "la coupure doit incrémenter reconnect_count, reçu {}",
            reconnected.reconnect_count
        );

        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn gere_acp_deja_connu_se_reinscrit_sous_nouvelle_instance_tient_mode_et_domaine() {
        let (mut state, config) = state_with_registered_agent("managed-rebind-instance");
        state.router.unregister_by_conn("conn-1");
        state.conn_instances.remove("conn-1");
        state.presences.clear();
        let lease = install_gere_acp_pour_reconnexion(&mut state, "managed-rebind-instance");
        let old_instance = lease.instance_id.clone();

        assert!(matches!(
            handle_register(
                "managed-rebind-instance",
                "cursor".to_string(),
                Some("cursor5".to_string()),
                Some("local".to_string()),
                Some("unix".to_string()),
                None,
                None,
                Some("macOS".to_string()),
                Some(old_instance.clone()),
                Some("bridget".to_string()),
                false,
                None,
                &mut state,
            ),
            DaemonToWrapper::Registered { .. }
        ));
        state.router.unregister_by_conn("managed-rebind-instance");
        state.mark_unreachable("managed-rebind-instance");
        state.managed_by_instance.remove(&old_instance);

        let new_instance = "cursor5-reprise-instance".to_string();
        let (wrapper_writer, _wrapper_reader) = control_socket("managed-rebind-instance-2");
        state
            .connections
            .insert("managed-rebind-instance-2".to_string(), wrapper_writer);
        assert!(matches!(
            handle_register(
                "managed-rebind-instance-2",
                "cursor".to_string(),
                Some("cursor5".to_string()),
                Some("local".to_string()),
                Some("unix".to_string()),
                None,
                None,
                Some("macOS".to_string()),
                Some(new_instance),
                None,
                true,
                None,
                &mut state,
            ),
            DaemonToWrapper::Registered { .. }
        ));
        let reconnected = state
            .agent_infos()
            .into_iter()
            .find(|agent| agent.name == "cursor5" && agent.state != "unreachable")
            .expect("réinscrit visible");
        assert_eq!(reconnected.transport, "acp");
        assert_eq!(reconnected.mode, Some(PresenceMode::Acp));
        assert_eq!(reconnected.domain.as_deref(), Some("bridget"));
        assert!(reconnected.reconnect_count >= 1);

        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn abonnement_attach_distingue_un_equipier_arrete_d_un_nom_inconnu() {
        let (mut state, config) = state_with_registered_agent("attach-stopped");
        state.router.unregister_by_conn("conn-1");
        state.mark_stopped("conn-1");
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
                reason: AttachRefusal::AgentStopped,
                ..
            })
        ));
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn inscription_claude_geree_expose_acp_domaine_et_accepte_attach() {
        let (mut state, config) = state_with_registered_agent("claude-managed-metadata");
        state.router.unregister_by_conn("conn-1");
        state.conn_instances.remove("conn-1");
        state.presences.clear();

        let (wrapper_writer, mut wrapper_reader) = control_socket("claude-managed-wrapper");
        let (attach_writer, _attach_reader) = control_socket("claude-managed-attach");
        state
            .connections
            .insert("claude-managed".to_string(), wrapper_writer);
        state
            .connections
            .insert("attach-claude-managed".to_string(), attach_writer);

        assert!(matches!(
            handle_register(
                "claude-managed",
                "claude".to_string(),
                Some("claude-managed".to_string()),
                Some("local".to_string()),
                Some("unix".to_string()),
                Some(PresenceMode::Acp),
                None,
                Some("macOS".to_string()),
                Some("instance-claude-managed".to_string()),
                Some("bridget".to_string()),
                false,
                None,
                &mut state,
            ),
            DaemonToWrapper::Registered { ref name } if name == "claude-managed"
        ));

        let agent = state.agent_infos().pop().expect("Claude inscrit");
        assert_eq!(agent.transport, "acp");
        assert_eq!(agent.channel.as_deref(), Some("unix"));
        assert_eq!(agent.mode, Some(PresenceMode::Acp));
        assert_eq!(agent.domain.as_deref(), Some("bridget"));

        let shared = Arc::new(Mutex::new(state));
        assert!(
            handle_wrapper_message("claude-managed", WrapperToDaemon::JournalReady, &shared,)
                .is_none()
        );
        assert!(matches!(
            handle_wrapper_message(
                "attach-claude-managed",
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
                "attach-claude-managed",
                WrapperToDaemon::Subscribe {
                    agent: "claude-managed".to_string(),
                    window: bridget_transport::AttachWindow::Today,
                },
                &shared,
            )
            .is_none(),
            "un Claude géré en ACP doit être attachable"
        );
        assert!(matches!(
            read_control(&mut wrapper_reader),
            DaemonToWrapper::Subscribe { .. }
        ));

        let (controls, views) = {
            let mut state = shared.lock().unwrap();
            close_attach_subscriptions(&mut state, "attach-claude-managed")
        };
        assert!(execute_controls(controls).is_empty());
        for view in views {
            view.close_and_join();
        }
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
                caught_up: false,
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
    fn limite_attestee_est_exposee_sans_decision_automatique() {
        use bridget_transport::protocol::RateLimitSource;
        let (mut state, config) = state_with_registered_agent("limite-attestee");

        let ack = handle_rate_limit(
            "agent-2",
            "five_hour".to_string(),
            "rejected".to_string(),
            Some(1_787_572_200),
            None,
            RateLimitSource::ClaudeStreamJson,
            &mut state,
        );
        assert!(matches!(ack, DaemonToWrapper::Ack { .. }));
        let agent = state.agent_infos().pop().unwrap();
        assert_eq!(agent.state, "connected", "la limite ne change pas l'état");
        assert_eq!(agent.rate_limits.len(), 1);
        assert_eq!(agent.rate_limits[0].window, "five_hour");
        assert_eq!(agent.rate_limits[0].status, "rejected");
        assert_eq!(agent.rate_limits[0].resets_at, Some(1_787_572_200));

        let _ = std::fs::remove_file(&config.db_path);
    }

    #[test]
    fn fait_five_hour_n_efface_pas_fait_seven_day() {
        use bridget_transport::protocol::RateLimitSource;
        let (mut state, config) = state_with_registered_agent("limite-deux-fenetres");

        let first = handle_rate_limit(
            "agent-2",
            "seven_day".to_string(),
            "allowed".to_string(),
            Some(1_787_700_000),
            Some(61),
            RateLimitSource::ClaudeStreamJson,
            &mut state,
        );
        assert!(matches!(first, DaemonToWrapper::Ack { .. }));
        let second = handle_rate_limit(
            "agent-2",
            "five_hour".to_string(),
            "allowed".to_string(),
            Some(1_787_572_200),
            Some(19),
            RateLimitSource::ClaudeStreamJson,
            &mut state,
        );
        assert!(matches!(second, DaemonToWrapper::Ack { .. }));

        let agent = state.agent_infos().pop().unwrap();
        assert_eq!(agent.rate_limits.len(), 2, "les deux fenêtres coexistent");
        let windows: Vec<_> = agent
            .rate_limits
            .iter()
            .map(|fact| fact.window.as_str())
            .collect();
        assert!(windows.contains(&"five_hour"));
        assert!(windows.contains(&"seven_day"));
        let seven = agent
            .rate_limits
            .iter()
            .find(|fact| fact.window == "seven_day")
            .expect("7d conservée après upsert 5h");
        assert_eq!(seven.used_percent, Some(61));
        assert_eq!(seven.resets_at, Some(1_787_700_000));

        let _ = std::fs::remove_file(&config.db_path);
    }

    #[test]
    fn served_model_rend_l_ecart_visible_sans_inventer_si_muet() {
        let (mut state, config) = state_with_registered_agent("served-model-who");
        let presence = state.presences.values_mut().next().unwrap();
        presence.model = Some("claude-opus-5".to_string());
        presence.served_model = None;
        let silent = state.agent_infos().pop().unwrap();
        assert!(
            silent.model_mismatch.is_none(),
            "flux muet = pas de verdict"
        );

        let ack = handle_served_model("agent-2", "claude-opus-4-6".to_string(), &mut state);
        assert!(matches!(ack, DaemonToWrapper::Ack { .. }));
        let agent = state.agent_infos().pop().unwrap();
        assert_eq!(agent.state, "connected", "l'écart ne change pas l'état");
        assert_eq!(agent.model.as_deref(), Some("claude-opus-5"));
        let gap = agent.model_mismatch.expect("écart visible");
        assert_eq!(gap.pinned, "claude-opus-5");
        assert_eq!(gap.served, "claude-opus-4-6");

        let match_ack = handle_served_model("agent-2", "claude-opus-5".to_string(), &mut state);
        assert!(matches!(match_ack, DaemonToWrapper::Ack { .. }));
        let aligned = state.agent_infos().pop().unwrap();
        assert!(aligned.model_mismatch.is_none());

        let _ = std::fs::remove_file(&config.db_path);
    }

    #[test]
    fn usage_atteste_s_agrege_et_sans_source_reste_inconnu() {
        use bridget_transport::protocol::{UsageSource, UsageTokens};
        let (mut state, config) = state_with_registered_agent("usage-atteste");

        let ack = handle_usage(
            "agent-2",
            UsageTokens {
                input_tokens: 2,
                output_tokens: 175,
                cache_creation_input_tokens: 40_804,
                cache_read_input_tokens: 13_907,
            },
            UsageSource::ClaudeStreamJson,
            &mut state,
        );
        assert!(matches!(ack, DaemonToWrapper::Ack { .. }));
        let window = handle_usage_window("agent-2", 1, i64::MAX, &state);
        match window {
            DaemonToWrapper::UsageWindowResult {
                aggregate: Some(aggregate),
                ..
            } => {
                assert_eq!(aggregate.turns, 1);
                assert_eq!(aggregate.facturable_tokens, 40_981);
                assert_eq!(aggregate.cache_read_input_tokens, 13_907);
            }
            other => panic!("agrégat attesté attendu, reçu {other:?}"),
        }
        let unknown = handle_usage_window("tmux-sans-sonde", 1, i64::MAX, &state);
        assert!(matches!(
            unknown,
            DaemonToWrapper::UsageWindowResult {
                aggregate: None,
                ..
            }
        ));

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
            Some(PresenceMode::Acp),
            None,
            Some("macOS".to_string()),
            Some("instance-1".to_string()),
            Some("bridget".to_string()),
            false,
            None,
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
            Some("unix".to_string()),
            Some(PresenceMode::Acp),
            None,
            Some("macOS".to_string()),
            Some("instance-1".to_string()),
            None,
            true,
            None,
            &mut state,
        );
        assert!(matches!(response, DaemonToWrapper::Registered { .. }));
        assert_eq!(state.agent_infos()[0].state, "busy");
        state.router.unregister_by_conn("conn-2");
        state.mark_stopped("conn-2");
        assert_eq!(state.agent_infos()[0].state, "stopped");
        let _ = std::fs::remove_file(&config.db_path);
    }

    /// Oracle (iii) bout-en-bout Register : un nouvel instance_id pour le même
    /// nom reprend les remises `dispatching` de l'instance morte.
    #[test]
    fn register_migre_les_remises_dispatching_de_l_ancienne_instance() {
        let (mut state, config) = state_with_registered_agent("migrate-dispatching");
        let key = IdempotencyKey::new(
            "012_scope_bbbbbbbbbbbb",
            OperationKind::Send,
            "msg-orphan-register",
        )
        .unwrap();
        let now = unix_now_secs();
        let mut message = bridget_core::BridgetMessage::new("peer-a", "agent-2", "collège en vol");
        message.id = "msg-orphan-register".to_string();
        state
            .idempotency
            .reserve(
                &key,
                b"orphan-register",
                now,
                CLIENT_IDEMPOTENCY_HORIZON_SECS,
                now,
                CLIENT_ISSUED_AT_TOLERANCE_SECS,
            )
            .unwrap();
        state
            .idempotency
            .begin_send_delivery(
                &key,
                &SendDelivery {
                    delivery_id: "delivery-orphan-register".to_string(),
                    recipient_instance_id: "instance-1".to_string(),
                    delivery_generation: 11,
                    expires_at: now + CLIENT_IDEMPOTENCY_HORIZON_SECS,
                    message_bytes: serde_json::to_vec(&message).unwrap(),
                },
            )
            .unwrap();
        assert_eq!(
            state
                .idempotency
                .dispatching_deliveries_for_instance("instance-1", now)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            state
                .store
                .recent_messages(20)
                .unwrap()
                .iter()
                .filter(|entry| entry.id == "msg-orphan-register")
                .count(),
            1,
            "visibilité d'émission avant respawn"
        );

        state.router.unregister_by_conn("conn-1");
        state.mark_unreachable("conn-1");
        let response = handle_register(
            "conn-new",
            "claude".to_string(),
            Some("agent-2".to_string()),
            Some("macbook".to_string()),
            Some("acp".to_string()),
            Some(PresenceMode::Acp),
            None,
            Some("macOS".to_string()),
            Some("instance-2".to_string()),
            None,
            false,
            Some(true),
            &mut state,
        );
        assert!(matches!(response, DaemonToWrapper::Registered { .. }));
        assert!(
            !state.presences.contains_key("instance-1"),
            "l'ancienne instance doit être retirée après migration"
        );
        assert!(state.presences.contains_key("instance-2"));
        assert_eq!(
            state
                .idempotency
                .dispatching_deliveries_for_instance("instance-1", now)
                .unwrap()
                .len(),
            0
        );
        let revived = state
            .idempotency
            .dispatching_deliveries_for_instance("instance-2", now)
            .unwrap();
        assert_eq!(revived.len(), 1);
        assert_eq!(revived[0].delivery_id, "delivery-orphan-register");
        assert_eq!(revived[0].recipient_instance_id, "instance-2");
        let _ = std::fs::remove_file(&config.db_path);
    }

    /// Redémarrage daemon simulé : aucune présence préalable, le wrapper
    /// ré-annonce un tour ouvert via Register. Sans `turn_in_progress=true`,
    /// who afficherait `connected` — exactement le mensonge du constat.
    #[test]
    fn redemarrage_daemon_reinscription_tour_en_cours_affiche_busy() {
        let (mut state, config) = state_with_registered_agent("busy-cold-restart");
        state.router.unregister_by_conn("conn-1");
        state.conn_instances.clear();
        state.presences.clear();

        let response = handle_register(
            "conn-fresh",
            "codex".to_string(),
            Some("coder-natif".to_string()),
            Some("local".to_string()),
            Some("unix".to_string()),
            None,
            None,
            Some("macOS".to_string()),
            Some("instance-cold".to_string()),
            None,
            true,
            Some(false),
            &mut state,
        );
        assert!(matches!(response, DaemonToWrapper::Registered { .. }));
        let agent = state.agent_infos().pop().expect("agent réinscrit");
        assert_eq!(agent.name, "coder-natif");
        assert_eq!(
            agent.state, "busy",
            "who doit restaurer busy depuis Register.turn_in_progress, pas attendre un prochain tour"
        );

        // Contrôle négatif du même oracle : sans le fait, connected reste honnête.
        state.router.unregister_by_conn("conn-fresh");
        state.conn_instances.clear();
        state.presences.clear();
        let idle = handle_register(
            "conn-idle",
            "codex".to_string(),
            Some("coder-natif".to_string()),
            Some("local".to_string()),
            Some("unix".to_string()),
            None,
            None,
            Some("macOS".to_string()),
            Some("instance-cold".to_string()),
            None,
            false,
            Some(false),
            &mut state,
        );
        assert!(matches!(idle, DaemonToWrapper::Registered { .. }));
        assert_eq!(state.agent_infos()[0].state, "connected");
        let _ = std::fs::remove_file(&config.db_path);
    }

    /// Course production : le router libère le nom (EOF), le nouveau Register
    /// arrive AVANT mark_unreachable, avec l'ancienne carte instance encore
    /// présente. Sans prise de possession, le Register est traité comme
    /// auxiliaire MCP et busy est perdu — rouge sur cet oracle.
    #[test]
    fn fermeture_ancienne_connexion_n_ecrase_pas_busy_de_la_reinscription() {
        let (mut state, config) = state_with_registered_agent("busy-race-rebind");
        state.set_turn_state("conn-1", true).unwrap();
        assert_eq!(state.agent_infos()[0].state, "busy");

        // Première moitié du cleanup EOF : le nom est libre, la carte instance
        // de l'ancienne connexion est encore là (mark_unreachable pas encore).
        state.router.unregister_by_conn("conn-1");
        assert!(state.conn_instances.contains_key("conn-1"));

        let response = handle_register(
            "conn-2",
            "claude".to_string(),
            Some("agent-2".to_string()),
            Some("macbook".to_string()),
            Some("unix".to_string()),
            Some(PresenceMode::Acp),
            None,
            Some("macOS".to_string()),
            Some("instance-1".to_string()),
            None,
            true,
            Some(false),
            &mut state,
        );
        assert!(matches!(response, DaemonToWrapper::Registered { .. }));
        assert_eq!(state.agent_infos()[0].state, "busy");
        assert!(
            state.conn_instances.contains_key("conn-2"),
            "la reconnexion du même équipier doit prendre l'instance, pas rester auxiliaire"
        );
        assert!(
            !state.conn_instances.contains_key("conn-1"),
            "l'ancienne carte instance doit être retirée au takeover"
        );

        // Seconde moitié du cleanup EOF (retardée) : ne doit plus toucher busy.
        state.mark_unreachable("conn-1");
        assert_eq!(
            state.agent_infos()[0].state,
            "busy",
            "l'EOF retardé de l'ancienne connexion ne doit pas effacer busy attesté"
        );
        let _ = std::fs::remove_file(&config.db_path);
    }

    #[test]
    fn projection_ledger_est_bornee_et_lit_le_store_du_daemon() {
        let (state, config) = state_with_registered_agent("ledger-projection");
        let mut first = bridget_core::BridgetMessage::new("alice", "bob", "bonjour");
        first.id = "m-1".to_string();
        let mut second = bridget_core::BridgetMessage::new("alice", "bob", "salut");
        second.id = "m-2".to_string();
        state.store.record_message(&first, "alice:bob").unwrap();
        state.store.record_message(&second, "alice:bob").unwrap();
        let shared = Arc::new(Mutex::new(state));

        let response = handle_wrapper_message(
            "conn-cli",
            WrapperToDaemon::LedgerProjection {
                scope: bridget_transport::protocol::LedgerScope::Messages,
                limit: u16::MAX,
            },
            &shared,
        );

        assert!(matches!(
            response,
            Some(DaemonToWrapper::LedgerProjection { messages, requests })
                if messages.len() == 2
                    && messages.iter().any(|message| message.id == "m-1")
                    && requests.is_empty()
        ));
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
        let (mut state, config) = state_with_registered_agent("timeout-concurrent");
        state
            .store
            .create_request("request-timeout", "sender", "agent-2", 60)
            .unwrap();
        assert!(claim_timeout(&mut state.store, "request-timeout"));
        assert!(!claim_timeout(&mut state.store, "request-timeout"));
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
                limit: 200,
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
        let (managed_tx, _managed_rx) = mpsc::channel();
        let mut state = DaemonState::new(&config, managed_tx).unwrap();
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

    fn managed_test_lease(command_id: &str) -> SpawnLease {
        SpawnLease {
            command_id: command_id.to_string(),
            name: "agent-2".to_string(),
            instance_id: "instance-1".to_string(),
            generation: 1,
            deadline_at: unix_timestamp() + 60,
            persistent: false,
        }
    }

    fn install_managed_test_spawn(
        state: &mut DaemonState,
        command_id: &str,
        connected: bool,
    ) -> (SpawnLease, Arc<ManagedStopControl>) {
        let now = unix_timestamp();
        let order = FleetSpawnOrder {
            agent_type: "fixture".to_string(),
            requested_name: Some("agent-2".to_string()),
            cwd: PathBuf::from("/tmp"),
            persistent: false,
            command_id: command_id.to_string(),
            issued_at: now,
            deadline_at: now + 60,
        };
        let lease = match state.fleet.request_spawn(&order, now).unwrap() {
            crate::fleet::SpawnSubmission::Start(lease) => lease,
            other => panic!("spawn de test non démarré: {other:?}"),
        };
        let definition = recovery_fixture_definition();
        state.fleet.mark_starting(&lease, now, &definition).unwrap();
        if connected {
            state
                .fleet
                .register_connected(&lease, &lease.instance_id, now + 1)
                .unwrap();
        }
        let stop = Arc::new(ManagedStopControl::new());
        state
            .managed_by_instance
            .insert(lease.instance_id.clone(), lease.command_id.clone());
        state.managed_spawns.insert(
            lease.command_id.clone(),
            ManagedSpawnRecord {
                lease: lease.clone(),
                agent_type: "fixture".to_string(),
                requester_conns: Vec::new(),
                wrapper_conn: connected.then(|| "conn-1".to_string()),
                stop: Arc::clone(&stop),
            },
        );
        (lease, stop)
    }

    fn managed_test_binary() -> PathBuf {
        let executable = std::env::current_exe()
            .unwrap()
            .parent()
            .and_then(std::path::Path::parent)
            .unwrap()
            .join("bridget");
        assert!(executable.exists(), "binaire bridget de test absent");
        executable
    }

    fn managed_test_prepared(lease: &SpawnLease, root: &std::path::Path) -> PreparedSpawn {
        let frozen_registry = AgentRegistry::from_json(
            r#"{"agents":{"fixture":{"command":"/bin/sh","protocol":"acp"}}}"#,
            "/tmp/managed-test-definition.json",
        )
        .unwrap();
        PreparedSpawn {
            lease: lease.clone(),
            agent_type: "fixture".to_string(),
            command: "/bin/sh".to_string(),
            args: Vec::new(),
            resolved_definition: Box::new(frozen_registry.resolved_definition("fixture").unwrap()),
            cwd: root.to_path_buf(),
            env: BTreeMap::from([
                ("HOME".to_string(), root.as_os_str().to_owned()),
                ("PATH".to_string(), OsString::from("/bin:/usr/bin")),
                ("USER".to_string(), OsString::from("tester")),
                ("LANG".to_string(), OsString::from("C")),
                ("TMPDIR".to_string(), OsString::from("/tmp")),
            ]),
        }
    }

    fn running_managed_test_group(
        lease: &SpawnLease,
        root: &std::path::Path,
        shell: &str,
    ) -> RunningManagedChild {
        let launch = ManagedLaunch {
            bootstrap_executable: managed_test_binary(),
            identity: ManagedIdentity {
                instance_id: lease.instance_id.clone(),
                command_id: lease.command_id.clone(),
                generation: lease.generation,
            },
            wrapper_executable: PathBuf::from("/bin/sh"),
            wrapper_args: vec!["-c".to_string(), shell.to_string()],
            cwd: root.to_path_buf(),
            env: managed_test_prepared(lease, root).env,
        };
        let marker_store = ManagedMarkerStore::at_directory(root.join("managed"));
        let mut child = crate::managed_process::spawn_managed_bootstrap(&launch)
            .unwrap()
            .wait_ready()
            .unwrap()
            .persist_marker(&marker_store, &lease.name)
            .unwrap()
            .release()
            .unwrap();
        child.set_status_nonblocking().unwrap();
        child
    }

    #[test]
    fn stop_refuse_structurellement_un_wrapper_terminal() {
        let (state, config) = state_with_registered_agent("stop-not-managed");
        let shared = Arc::new(Mutex::new(state));

        assert!(matches!(
            handle_wrapper_message(
                "control",
                WrapperToDaemon::StopOrder {
                    name: "agent-2".to_string(),
                    command_id: "stop-terminal".to_string(),
                },
                &shared,
            ),
            Some(DaemonToWrapper::StopResult {
                command_id,
                outcome: StopOutcome::NotManaged,
            }) if command_id == "stop-terminal"
        ));
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn stop_avant_marqueur_annule_la_generation_et_repond_apres_nettoyage() {
        let (mut state, config) = state_with_registered_agent("stop-before-marker");
        let (lease, stop) = install_managed_test_spawn(&mut state, "spawn-before-marker", false);
        let process_root = PathBuf::from(format!(
            "/tmp/bg907-before-{}-{}",
            std::process::id(),
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        std::fs::create_dir_all(&process_root).unwrap();
        let prepared = managed_test_prepared(&lease, &process_root);
        let shared = Arc::new(Mutex::new(state));
        let caller_state = Arc::clone(&shared);
        let caller = thread::spawn(move || {
            handle_wrapper_message(
                "control",
                WrapperToDaemon::StopOrder {
                    name: "agent-2".to_string(),
                    command_id: "stop-before-marker".to_string(),
                },
                &caller_state,
            )
        });
        let deadline = Instant::now() + Duration::from_secs(2);
        while !stop.is_requested() {
            assert!(
                Instant::now() < deadline,
                "stop non transmis au superviseur"
            );
            thread::yield_now();
        }
        assert!(
            stop.is_actionable(),
            "un lancement sans wrapper doit être annulable immédiatement"
        );
        let (event_tx, event_rx) = mpsc::channel();
        let marker_store = ManagedMarkerStore::at_directory(process_root.join("managed"));
        let stderr_store = ManagedStderrStore::at_directory(process_root.join("stderr"));
        let fleet = Arc::clone(&shared.lock().unwrap().fleet);
        let mut active = HashMap::new();
        handle_managed_command(
            ManagedSupervisorCommand::Start {
                prepared,
                stop: Arc::clone(&stop),
            },
            &fleet,
            &marker_store,
            &stderr_store,
            &event_tx,
            &mut active,
            Some(&managed_test_binary()),
        );
        assert!(active.is_empty());
        assert!(marker_store.load("agent-2").is_err());
        drain_managed_events(&shared, &event_rx);

        assert!(matches!(
            caller.join().unwrap(),
            Some(DaemonToWrapper::StopResult {
                command_id,
                outcome: StopOutcome::Stopped,
            }) if command_id == "stop-before-marker"
        ));
        assert!(shared.lock().unwrap().managed_spawns.is_empty());
        std::fs::remove_dir_all(process_root).unwrap();
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn stop_pendant_bootstrap_bloque_termine_le_processus_reel_avant_marqueur() {
        let (mut state, config) = state_with_registered_agent("stop-bootstrap-blocked");
        let (lease, stop) =
            install_managed_test_spawn(&mut state, "spawn-bootstrap-blocked", false);
        let process_root = PathBuf::from(format!(
            "/tmp/bg907-blocked-{}-{}",
            std::process::id(),
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        std::fs::create_dir_all(&process_root).unwrap();
        let barrier = process_root.join("bootstrap-entered");
        let bootstrap = process_root.join("blocked-bootstrap.sh");
        std::fs::write(
            &bootstrap,
            "#!/bin/sh\n: > \"$BRIDGET_TEST_BARRIER\"\ntrap 'exit 0' TERM\nwhile :; do sleep 1; done\n",
        )
        .unwrap();
        std::fs::set_permissions(&bootstrap, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut prepared = managed_test_prepared(&lease, &process_root);
        prepared.env.insert(
            "BRIDGET_TEST_BARRIER".to_string(),
            barrier.as_os_str().to_owned(),
        );
        let shared = Arc::new(Mutex::new(state));
        let fleet = Arc::clone(&shared.lock().unwrap().fleet);
        let marker_store = ManagedMarkerStore::at_directory(process_root.join("managed"));
        let marker_store_for_thread = marker_store.clone();
        let stderr_store = ManagedStderrStore::at_directory(process_root.join("stderr"));
        let (event_tx, event_rx) = mpsc::channel();
        let stop_for_handler = Arc::clone(&stop);
        let handler = thread::spawn(move || {
            let mut active = HashMap::new();
            handle_managed_command(
                ManagedSupervisorCommand::Start {
                    prepared,
                    stop: stop_for_handler,
                },
                &fleet,
                &marker_store_for_thread,
                &stderr_store,
                &event_tx,
                &mut active,
                Some(&bootstrap),
            );
            active
        });
        let deadline = Instant::now() + Duration::from_secs(2);
        while !barrier.exists() {
            assert!(
                Instant::now() < deadline,
                "le bootstrap réel n'a pas atteint la barrière"
            );
            thread::sleep(Duration::from_millis(5));
        }
        let caller_state = Arc::clone(&shared);
        let caller = thread::spawn(move || {
            handle_wrapper_message(
                "control",
                WrapperToDaemon::StopOrder {
                    name: "agent-2".to_string(),
                    command_id: "stop-bootstrap-blocked".to_string(),
                },
                &caller_state,
            )
        });
        assert!(handler.join().unwrap().is_empty());
        drain_managed_events(&shared, &event_rx);
        assert!(matches!(
            caller.join().unwrap(),
            Some(DaemonToWrapper::StopResult {
                outcome: StopOutcome::Stopped,
                ..
            })
        ));
        assert!(marker_store.load("agent-2").is_err());
        assert!(shared.lock().unwrap().managed_spawns.is_empty());
        std::fs::remove_dir_all(process_root).unwrap();
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn stop_apres_register_envoie_disconnect_puis_attend_la_fin_du_groupe() {
        let (mut state, config) = state_with_registered_agent("stop-connected");
        let (wrapper_writer, mut wrapper_reader) = control_socket("stop-wrapper");
        state
            .connections
            .insert("conn-1".to_string(), wrapper_writer);
        let (lease, stop) = install_managed_test_spawn(&mut state, "spawn-connected", true);
        let process_root = PathBuf::from(format!(
            "/tmp/bg907-connected-{}-{}",
            std::process::id(),
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        std::fs::create_dir_all(&process_root).unwrap();
        let child = running_managed_test_group(
            &lease,
            &process_root,
            "trap 'exit 0' TERM; while :; do sleep 1; done",
        );
        let pgid = child.marker().marker().pgid;
        let marker_path = child.marker().path().to_path_buf();
        let fleet = Arc::clone(&state.fleet);
        let mut active = HashMap::from([(
            lease.instance_id.clone(),
            SupervisedProcess {
                prepared: managed_test_prepared(&lease, &process_root),
                child,
                registered: Some(("conn-1".to_string(), "agent-2".to_string())),
                connected: true,
                failure_sent: false,
                stop: Arc::clone(&stop),
                stop_attempted: false,
            },
        )]);
        let shared = Arc::new(Mutex::new(state));
        let caller_state = Arc::clone(&shared);
        let caller = thread::spawn(move || {
            handle_wrapper_message(
                "control",
                WrapperToDaemon::StopOrder {
                    name: "agent-2".to_string(),
                    command_id: "stop-connected".to_string(),
                },
                &caller_state,
            )
        });

        assert!(matches!(
            read_control(&mut wrapper_reader),
            DaemonToWrapper::Disconnect
        ));
        let deadline = Instant::now() + Duration::from_secs(2);
        while !stop.is_actionable() {
            assert!(
                Instant::now() < deadline,
                "le handshake écrit n'a pas libéré le superviseur"
            );
            thread::yield_now();
        }
        let (event_tx, event_rx) = mpsc::channel();
        poll_managed_processes(&fleet, &event_tx, &mut active);
        assert!(
            active.is_empty(),
            "le groupe réel reste supervisé après sa disparition"
        );
        drain_managed_events(&shared, &event_rx);

        assert!(matches!(
            caller.join().unwrap(),
            Some(DaemonToWrapper::StopResult {
                command_id,
                outcome: StopOutcome::Stopped | StopOutcome::StoppedForced { .. },
            }) if command_id == "stop-connected"
        ));
        let state = shared.lock().unwrap();
        assert!(state.router.get_agent("agent-2").is_none());
        assert_eq!(state.presences["instance-1"].state, "stopped");
        drop(state);
        assert!(!marker_path.exists());
        assert!(!crate::managed_process::group_exists(pgid).unwrap());
        std::fs::remove_dir_all(process_root).unwrap();
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn timeout_d_arret_conserve_le_groupe_reel_et_sa_supervision_jusqu_a_disparition() {
        let (mut state, config) = state_with_registered_agent("stop-timeout-real");
        let (wrapper_writer, mut wrapper_reader) = control_socket("stop-timeout-wrapper");
        state
            .connections
            .insert("conn-1".to_string(), wrapper_writer);
        let (lease, stop) = install_managed_test_spawn(&mut state, "spawn-timeout", true);
        let process_root = PathBuf::from(format!(
            "/tmp/bg907-timeout-{}-{}",
            std::process::id(),
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        std::fs::create_dir_all(&process_root).unwrap();
        let child = running_managed_test_group(
            &lease,
            &process_root,
            "trap '' TERM; while :; do sleep 1; done",
        );
        let pgid = child.marker().marker().pgid;
        let marker_path = child.marker().path().to_path_buf();
        let fleet = Arc::clone(&state.fleet);
        let mut active = HashMap::from([(
            lease.instance_id.clone(),
            SupervisedProcess {
                prepared: managed_test_prepared(&lease, &process_root),
                child,
                registered: Some(("conn-1".to_string(), "agent-2".to_string())),
                connected: true,
                failure_sent: false,
                stop: Arc::clone(&stop),
                stop_attempted: false,
            },
        )]);
        let shared = Arc::new(Mutex::new(state));
        let caller_state = Arc::clone(&shared);
        let caller = thread::spawn(move || {
            handle_wrapper_message(
                "control",
                WrapperToDaemon::StopOrder {
                    name: "agent-2".to_string(),
                    command_id: "stop-timeout".to_string(),
                },
                &caller_state,
            )
        });

        assert!(matches!(
            read_control(&mut wrapper_reader),
            DaemonToWrapper::Disconnect
        ));
        let deadline = Instant::now() + Duration::from_secs(2);
        while !stop.is_actionable() {
            assert!(
                Instant::now() < deadline,
                "stop non transmis au superviseur"
            );
            thread::yield_now();
        }
        let (event_tx, event_rx) = mpsc::channel();
        poll_managed_processes(&fleet, &event_tx, &mut active);
        assert!(active.contains_key(&lease.instance_id));
        drain_managed_events(&shared, &event_rx);
        assert!(matches!(
            caller.join().unwrap(),
            Some(DaemonToWrapper::StopResult {
                outcome: StopOutcome::Timeout { .. },
                ..
            })
        ));
        {
            let state = shared.lock().unwrap();
            assert!(state.managed_spawns.contains_key(&lease.command_id));
            assert_eq!(
                state.managed_by_instance.get(&lease.instance_id),
                Some(&lease.command_id)
            );
            assert!(state.router.get_agent("agent-2").is_some());
            assert_eq!(state.presences["instance-1"].state, "connected");
        }
        assert!(marker_path.exists());
        assert!(crate::managed_process::group_exists(pgid).unwrap());

        crate::managed_process::signal_group(pgid, libc::SIGKILL).unwrap();
        let cleanup_deadline = Instant::now() + Duration::from_secs(2);
        while active.contains_key(&lease.instance_id) {
            poll_managed_processes(&fleet, &event_tx, &mut active);
            assert!(
                Instant::now() < cleanup_deadline,
                "le superviseur n'a pas observé la disparition réelle"
            );
            thread::sleep(Duration::from_millis(10));
        }
        drain_managed_events(&shared, &event_rx);
        assert!(!marker_path.exists());
        assert!(!crate::managed_process::group_exists(pgid).unwrap());
        assert!(shared.lock().unwrap().managed_spawns.is_empty());
        std::fs::remove_dir_all(process_root).unwrap();
        let _ = std::fs::remove_file(config.db_path);
    }

    /// Oracle anti-fantôme : si la négociation expire après naissance du
    /// processus (BootstrapReady + RELEASE, pas encore Connected), le groupe
    /// réel DOIT mourir sous SIGTERM. Meurt si l'enfant survit au refus.
    #[test]
    fn timeout_avant_connected_termine_le_groupe_reel_sans_fantome() {
        let (mut state, config) = state_with_registered_agent("spawn-timeout-fantome");
        let (requester, mut requester_reader) = control_socket("timeout-fantome-requester");
        state.connections.insert("requester".to_string(), requester);
        let (lease, stop) = install_managed_test_spawn(&mut state, "spawn-timeout-fantome", false);
        {
            let record = state
                .managed_spawns
                .get_mut(&lease.command_id)
                .expect("spawn enregistré");
            record.requester_conns.push("requester".to_string());
        }
        let process_root = PathBuf::from(format!(
            "/tmp/bg-fantome-{}-{}",
            std::process::id(),
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        std::fs::create_dir_all(&process_root).unwrap();
        let child = running_managed_test_group(
            &lease,
            &process_root,
            "trap 'exit 0' TERM; while :; do sleep 1; done",
        );
        let pgid = child.marker().marker().pgid;
        let marker_path = child.marker().path().to_path_buf();
        assert!(
            crate::managed_process::group_exists(pgid).unwrap(),
            "précondition: enfant vivant"
        );

        let mut prepared = managed_test_prepared(&lease, &process_root);
        prepared.lease.deadline_at = unix_timestamp() - 1;
        let fleet = Arc::clone(&state.fleet);
        let mut active = HashMap::from([(
            lease.instance_id.clone(),
            SupervisedProcess {
                prepared,
                child,
                registered: None,
                connected: false,
                failure_sent: false,
                stop: Arc::clone(&stop),
                stop_attempted: false,
            },
        )]);
        let shared = Arc::new(Mutex::new(state));
        let (event_tx, event_rx) = mpsc::channel();
        poll_managed_processes(&fleet, &event_tx, &mut active);
        assert!(
            active.is_empty(),
            "le superviseur doit retirer le spawn expiré de active"
        );
        assert!(
            !crate::managed_process::group_exists(pgid).unwrap(),
            "fantôme: le groupe survit à une négociation expirée"
        );
        assert!(!marker_path.exists(), "marqueur résiduel après timeout");
        drain_managed_events(&shared, &event_rx);
        assert!(matches!(
            read_control(&mut requester_reader),
            DaemonToWrapper::SpawnRejected {
                reason: SpawnRefusal::SpawnTimeout,
                ..
            }
        ));
        assert!(shared.lock().unwrap().managed_spawns.is_empty());
        std::fs::remove_dir_all(process_root).unwrap();
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn stop_apres_register_traverse_le_wrapper_et_le_superviseur_reels() {
        let root = PathBuf::from(format!(
            "/tmp/bg907-e2e-{}-{}",
            std::process::id(),
            &Uuid::new_v4().simple().to_string()[..8]
        ));
        let cache = root.join(".cache/bridget");
        let registry_path = root.join(".config/bridget/agents.json");
        let adapter = root.join("adapter.sh");
        std::fs::create_dir_all(&cache).unwrap();
        std::fs::create_dir_all(registry_path.parent().unwrap()).unwrap();
        std::fs::write(
            &adapter,
            "#!/bin/sh\nread initialize\necho '{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"protocolVersion\":1}}'\nread session\necho '{\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"sessionId\":\"fixture-session\"}}'\nwhile read line; do :; done\n",
        )
        .unwrap();
        std::fs::set_permissions(&adapter, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::write(
            &registry_path,
            serde_json::to_vec(&serde_json::json!({
                "agents": {
                    "fixture": {
                        "command": adapter,
                        "protocol": "acp",
                        "permissions": "allow",
                        "queue_capacity": 2,
                        "notify_timeout_secs": 1
                    }
                }
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::set_permissions(&registry_path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let config = DaemonConfig {
            socket_path: cache.join("bridget.sock"),
            db_path: cache.join("bridget.db"),
            log_path: cache.join("daemon.log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        };
        let (managed_tx, managed_rx) = mpsc::channel();
        let mut state = DaemonState::new(&config, managed_tx.clone()).unwrap();
        let (lease, _stop) = install_managed_test_spawn(&mut state, "spawn-e2e", false);
        let mut prepared = managed_test_prepared(&lease, &root);
        prepared.agent_type = "fixture".to_string();
        prepared.command = adapter.to_string_lossy().into_owned();
        prepared.resolved_definition = Box::new(
            AgentRegistry::from_json(
                &serde_json::json!({
                    "agents": {
                        "fixture": {
                            "command": adapter,
                            "protocol": "acp",
                            "permissions": "allow",
                            "queue_capacity": 2,
                            "notify_timeout_secs": 1
                        }
                    }
                })
                .to_string(),
                "/tmp/managed-wrapper-frozen.json",
            )
            .unwrap()
            .resolved_definition("fixture")
            .unwrap(),
        );
        let shared = Arc::new(Mutex::new(state));
        let listener = UnixListener::bind(&config.socket_path).unwrap();
        let connection_state = Arc::clone(&shared);
        let connection = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle_connection(stream, connection_state).unwrap();
        });
        let (event_tx, event_rx) = mpsc::channel();
        let _supervisor = ManagedSupervisorGuard::from_existing_sender(
            managed_tx,
            managed_rx,
            Arc::clone(&shared.lock().unwrap().fleet),
            &config,
            event_tx,
            Some(managed_test_binary()),
        );
        shared
            .lock()
            .unwrap()
            .managed_tx
            .send(ManagedSupervisorCommand::Start {
                prepared,
                stop: Arc::clone(&shared.lock().unwrap().managed_spawns[&lease.command_id].stop),
            })
            .unwrap();

        let ready_deadline = Instant::now() + Duration::from_secs(5);
        loop {
            drain_managed_events(&shared, &event_rx);
            let registered = shared
                .lock()
                .unwrap()
                .managed_spawns
                .get(&lease.command_id)
                .and_then(|record| record.wrapper_conn.as_ref())
                .is_some();
            if registered {
                break;
            }
            assert!(
                Instant::now() < ready_deadline,
                "le wrapper réel ne s'est pas enregistré"
            );
            thread::sleep(Duration::from_millis(10));
        }
        let marker_store = ManagedMarkerStore::at_directory(cache.join("managed"));
        let marker = marker_store.load("agent-2").unwrap();
        assert!(crate::managed_process::group_exists(marker.pgid).unwrap());

        let stop_state = Arc::clone(&shared);
        let (result_tx, result_rx) = mpsc::channel();
        thread::spawn(move || {
            let result = handle_wrapper_message(
                "control",
                WrapperToDaemon::StopOrder {
                    name: "agent-2".to_string(),
                    command_id: "stop-e2e".to_string(),
                },
                &stop_state,
            );
            result_tx.send(result).unwrap();
        });
        let stop_deadline = Instant::now() + Duration::from_secs(5);
        let result = loop {
            drain_managed_events(&shared, &event_rx);
            if let Ok(result) = result_rx.try_recv() {
                break result;
            }
            assert!(
                Instant::now() < stop_deadline,
                "la chaîne réelle stop n'a pas produit d'issue"
            );
            thread::sleep(Duration::from_millis(10));
        };
        assert!(matches!(
            result,
            Some(DaemonToWrapper::StopResult {
                outcome: StopOutcome::Stopped | StopOutcome::StoppedForced { .. },
                ..
            })
        ));
        assert!(!crate::managed_process::group_exists(marker.pgid).unwrap());
        assert!(marker_store.load("agent-2").is_err());
        {
            let state = shared.lock().unwrap();
            assert!(state.managed_spawns.is_empty());
            assert!(state.router.get_agent("agent-2").is_none());
            assert_eq!(state.presences[&lease.instance_id].state, "stopped");
        }
        connection.join().unwrap();
        drop(shared);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn stop_marqueur_perime_retombe_sur_not_managed_sans_tuer_le_wrapper_terminal() {
        let (state, config) = state_with_registered_agent("stop-stale-fallback");
        state
            .marker_store
            .persist(
                "agent-2",
                &crate::managed_process::BootstrapReady {
                    pid: std::process::id(),
                    pgid: unsafe { libc::getpgrp() } as u32,
                    birth: u64::MAX,
                    instance_id: "ancienne-instance".to_string(),
                    command_id: "ancienne-commande".to_string(),
                    generation: 1,
                },
            )
            .unwrap();
        let marker_store = state.marker_store.clone();
        let shared = Arc::new(Mutex::new(state));

        assert!(matches!(
            handle_wrapper_message(
                "control",
                WrapperToDaemon::StopOrder {
                    name: "agent-2".to_string(),
                    command_id: "stop-stale-fallback".to_string(),
                },
                &shared,
            ),
            Some(DaemonToWrapper::StopResult {
                outcome: StopOutcome::NotManaged,
                ..
            })
        ));
        assert!(marker_store.load("agent-2").is_err());
        assert!(shared.lock().unwrap().router.get_agent("agent-2").is_some());
        let marker_directory = config.db_path.parent().unwrap().join("managed");
        let _ = std::fs::remove_file(config.db_path);
        let _ = std::fs::remove_dir_all(marker_directory);
    }

    #[test]
    fn startup_failed_commande_absente_repond_le_motif_du_canal_sans_residu() {
        let (mut state, config) = state_with_registered_agent("managed-startup-failed");
        state.router.unregister_by_conn("conn-1");
        state.conn_instances.remove("conn-1");
        state.presences.remove("instance-1");
        let (requester, mut requester_reader) = control_socket("managed-requester");
        state.connections.insert("requester".to_string(), requester);
        let lease = managed_test_lease("command-missing");
        state
            .managed_by_instance
            .insert(lease.instance_id.clone(), lease.command_id.clone());
        state.managed_spawns.insert(
            lease.command_id.clone(),
            ManagedSpawnRecord {
                lease: lease.clone(),
                agent_type: "fixture".to_string(),
                requester_conns: vec!["requester".to_string()],
                wrapper_conn: None,
                stop: Arc::new(ManagedStopControl::new()),
            },
        );
        let shared = Arc::new(Mutex::new(state));
        let (event_tx, event_rx) = mpsc::channel();
        event_tx
            .send(ManagedSupervisorEvent::Failed {
                lease: lease.clone(),
                kind: "command_missing".to_string(),
                reason: "/adaptateur/disparu".to_string(),
                conn_id: None,
            })
            .unwrap();
        drain_managed_events(&shared, &event_rx);
        assert!(matches!(
            read_control(&mut requester_reader),
            DaemonToWrapper::SpawnRejected {
                command_id,
                reason: SpawnRefusal::CommandMissing { command, registry }
            } if command_id == lease.command_id
                && command == "/adaptateur/disparu"
                && registry == "canal managed-status"
        ));
        let state = shared.lock().unwrap();
        assert!(state.managed_spawns.is_empty());
        assert!(state.managed_by_instance.is_empty());
        assert!(state.managed_terminal_instances.contains("instance-1"));
        drop(state);
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn mort_spontanee_rejette_les_demandes_marque_stopped_et_termine_les_vues() {
        let (mut state, config) = state_with_registered_agent("managed-exit");
        let (sender_writer, mut sender_reader) = control_socket("managed-sender");
        let (attach_writer, mut attach_reader) = control_socket("managed-attach");
        state
            .connections
            .insert("sender-conn".to_string(), sender_writer);
        state
            .connections
            .insert("attach-conn".to_string(), attach_writer.clone());
        install_attach_view(&mut state, "managed-sub", "attach-conn", &attach_writer);
        state.pending_replies.push(PendingReply {
            msg_id: "managed-request".to_string(),
            from: "sender".to_string(),
            from_conn: "sender-conn".to_string(),
            to: "agent-2".to_string(),
            target_conn: "conn-1".to_string(),
            timeout_secs: 60,
            created_at: Instant::now(),
            escalation_level: 0,
            deferred_level: None,
        });
        let lease = managed_test_lease("command-exit");
        state
            .managed_by_instance
            .insert(lease.instance_id.clone(), lease.command_id.clone());
        state.managed_spawns.insert(
            lease.command_id.clone(),
            ManagedSpawnRecord {
                lease: lease.clone(),
                agent_type: "fixture".to_string(),
                requester_conns: Vec::new(),
                wrapper_conn: Some("conn-1".to_string()),
                stop: Arc::new(ManagedStopControl::new()),
            },
        );
        let executable = std::env::current_exe()
            .unwrap()
            .parent()
            .and_then(std::path::Path::parent)
            .unwrap()
            .join("bridget");
        assert!(executable.exists(), "binaire bridget de test absent");
        let process_root =
            std::env::temp_dir().join(format!("bridget-t906-death-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&process_root).unwrap();
        let launch = ManagedLaunch {
            bootstrap_executable: executable,
            identity: ManagedIdentity {
                instance_id: lease.instance_id.clone(),
                command_id: lease.command_id.clone(),
                generation: lease.generation,
            },
            wrapper_executable: PathBuf::from("/bin/sh"),
            wrapper_args: vec!["-c".to_string(), "exit 7".to_string()],
            cwd: process_root.clone(),
            env: std::collections::BTreeMap::new(),
        };
        let marker_store = ManagedMarkerStore::at_directory(process_root.join("managed"));
        let child = crate::managed_process::spawn_managed_bootstrap(&launch)
            .unwrap()
            .wait_ready()
            .unwrap()
            .persist_marker(&marker_store, "agent-2")
            .unwrap()
            .release()
            .unwrap();
        let prepared = PreparedSpawn {
            lease: lease.clone(),
            agent_type: "fixture".to_string(),
            command: "/bin/sh".to_string(),
            args: vec!["-c".to_string(), "exit 7".to_string()],
            resolved_definition: Box::new(bridget_transport::ResolvedAgentDefinition {
                command: "/bin/sh".to_string(),
                args: vec!["-c".to_string(), "exit 7".to_string()],
                protocol: "acp".to_string(),
                forbidden_env: Vec::new(),
                pass_env: Vec::new(),
                permissions: "allow".to_string(),
                queue_capacity: 32,
                notify_timeout_secs: 600,
                mcp: bridget_transport::ResolvedMcpDefinition {
                    interactive: "none".to_string(),
                    acp_session: false,
                },
                capabilities: bridget_transport::AdapterCapabilities::default(),
                digest: "fixture-digest".to_string(),
            }),
            cwd: process_root.clone(),
            env: std::collections::BTreeMap::new(),
        };
        let mut active = HashMap::from([(
            lease.instance_id.clone(),
            SupervisedProcess {
                prepared,
                child,
                registered: Some(("conn-1".to_string(), "agent-2".to_string())),
                connected: true,
                failure_sent: false,
                stop: Arc::new(ManagedStopControl::new()),
                stop_attempted: false,
            },
        )]);
        let (observed_tx, observed_rx) = mpsc::channel();
        let observed = loop {
            poll_managed_processes(&state.fleet, &observed_tx, &mut active);
            if let Ok(event) = observed_rx.try_recv() {
                break event;
            }
            thread::sleep(Duration::from_millis(10));
        };
        assert!(matches!(
            &observed,
            ManagedSupervisorEvent::Exited { reason, .. }
                if reason.contains("exit status: 7")
        ));
        let shared = Arc::new(Mutex::new(state));
        let (event_tx, event_rx) = mpsc::channel();
        event_tx.send(observed).unwrap();
        drain_managed_events(&shared, &event_rx);
        assert!(matches!(
            read_control(&mut sender_reader),
            DaemonToWrapper::DeliveryRejected { id, reason }
                if id == "managed-request" && reason.contains("exit status: 7")
        ));
        assert!(matches!(
            read_control(&mut attach_reader),
            DaemonToWrapper::End { subscription_id, reason }
                if subscription_id == "managed-sub" && reason == "wrapper indisponible"
        ));
        let state = shared.lock().unwrap();
        assert_eq!(state.presences["instance-1"].state, "stopped");
        assert!(state.pending_replies.is_empty());
        assert!(state.attach_subscriptions.is_empty());
        assert!(state.router.get_agent("agent-2").is_none());
        drop(state);
        let _ = std::fs::remove_dir_all(process_root);
        let _ = std::fs::remove_file(config.db_path);
    }
}
