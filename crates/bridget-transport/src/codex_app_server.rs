//! Pilote natif de `codex app-server`.
//!
//! Le protocole est volontairement porté en `serde_json::Value` : le schéma
//! généré par le binaire Codex installé est l'autorité de compatibilité. La
//! crate publiée `codex-app-server-protocol` suit un autre produit et une
//! autre numérotation ; la lier ici cacherait une incompatibilité possible.

use crate::journal::{JournalFailureSink, JournalLiveFeed, JournalWriter, with_turn_failed_kind};
use crate::managed_session::{
    ManagedEvent, ManagedEventKind, ManagedEventOrigin, ManagedEventSource,
    ManagedProviderIdentity, ManagedSession, ManagedSessionDescriptor, ManagedTerminal,
};
use crate::poison::RecoverPoison;
use crate::protocol::{
    ChildProviderRequest, ChildProviderRequestId, ChildRequestAnswer, ChildRequestDecision,
    ChildRequestKind, PresenceMode, ProviderObservation, ProviderOperation,
};
use crate::transport::{Transport, TransportError};
use bridget_core::{BridgetMessage, MessageIntent, MessageOrigin};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{BufRead, BufReader, Write};
use std::os::fd::AsRawFd;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

fn stop_interactive_server(child: &mut Child, socket: &Path) -> std::io::Result<()> {
    crate::managed_session::stop_owned_child(child, true, Duration::from_secs(4))?;
    match std::fs::remove_file(socket) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}
/// Un pilotage humain ne doit pas rester suspendu derrière un RPC silencieux.
/// Cette borne ne constitue pas une preuve de remise : elle laisse ensuite la
/// FIFO reprendre le message ou déclenche l'interruption de repli.
const STEER_REQUEST_TIMEOUT: Duration = Duration::from_secs(3);
/// Référence t3code : les interruptions de tours enfants sont bornées à trois
/// secondes. Bridget n'a pas de sous-arbre Codex à parcourir, mais la requête
/// du tour actif ne doit pas, elle non plus, retenir un humain indéfiniment.
const INTERRUPT_REQUEST_TIMEOUT: Duration = Duration::from_secs(3);
/// Après l'accusé (ou le silence) de `turn/interrupt`, le terminal du tour
/// ancien dispose encore d'une courte fenêtre. Au-delà, son état ne doit plus
/// immobiliser la file humaine : la remise devient explicitement rejetée,
/// jamais `dispatching` éternel.
const INTERRUPT_TERMINAL_TIMEOUT: Duration = Duration::from_secs(10);
const RATE_LIMIT_READ_TIMEOUT: Duration = Duration::from_secs(2);
const RATE_LIMIT_REFRESH_INTERVAL: Duration = Duration::from_secs(30);
const TURN_POLL: Duration = Duration::from_millis(25);
const SATURATION_RETRIES: u32 = 4;
const CODEX_SATURATED_REASON: &str = "saturation Codex";
const COMMAND_EXECUTION_APPROVAL_METHOD: &str = "item/commandExecution/requestApproval";

type Writer = Arc<Mutex<Option<Box<dyn Write + Send>>>>;
type SourceLines = Box<dyn Iterator<Item = std::io::Result<String>> + Send>;
type Waiters = Arc<Mutex<HashMap<u64, mpsc::Sender<Result<ServerResponse, String>>>>>;
type Journal = Arc<Mutex<Option<JournalWriter>>>;
type PendingRequest = Arc<Mutex<Option<PendingProviderRequest>>>;
/// Attentes RPC155 répondables, distinctes de la trace149 fingerprintée.
/// O(4) pour lookup/fermeture ; aucun corps secret ni droit dans cette table.
type ChildRequests = Arc<Mutex<Vec<PendingChildRequest>>>;

struct PendingChildRequest {
    request: ChildProviderRequest,
    thread_id: String,
    message_id: String,
    approval: Value,
    uncertain: bool,
}
/// Certains clients Codex exposent des outils comme requêtes app-server
/// `item/tool/call`, y compris lorsqu'un serveur MCP statique est présent.
/// Le daemon injecte ici l'exécuteur attesté correspondant : le pilote ne
/// possède ni identité Bridget ni accès direct à la socket métier.
pub type DynamicToolHandler = Arc<dyn Fn(&str, &Value) -> Result<Value, String> + Send + Sync>;

/// Métadonnées publiques de thread/list, jamais un accès aux fichiers Codex.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CodexThreadSummary {
    pub id: String,
    pub name: Option<String>,
    pub preview: Option<String>,
    pub cwd: String,
}
pub type InteractiveThreadSelector<'a> =
    &'a dyn Fn(&[CodexThreadSummary]) -> Result<String, String>;

#[derive(Debug)]
struct ServerResponse {
    value: Value,
    raw: Vec<u8>,
}

#[derive(Clone)]
pub struct CodexAppServerOptions {
    pub command: String,
    pub args: Vec<String>,
    pub queue_capacity: usize,
    pub notify_timeout_secs: u64,
    pub model: Option<String>,
    /// Même sémantique que le pilote ACP : `allow` accepte, tout autre
    /// réglage refuse. Une demande `item/*/requestApproval` reçoit toujours
    /// une réponse JSON-RPC sans intervention humaine.
    pub permissions: String,
    /// Mode155 figé par le wrapper ; faux conserve le chemin deny historique.
    pub child_requests: bool,
    /// Baseline relevée avant lancement, absente pour un registre historique.
    pub provider_observation: Option<ProviderObservation>,
    /// Démarrage de fil demandé par Bridget après vérification de la baseline.
    pub thread_bootstrap: CodexThreadBootstrap,
    /// Exécuteur des requêtes `item/tool/call` que Codex délègue au client.
    /// Il reste absent hors wrapper Bridget géré.
    pub dynamic_tool_handler: Option<DynamicToolHandler>,
}
/// Choix de fil au démarrage, explicitement versionné dans les options du
/// pilote afin qu'une reprise native ne puisse jamais être confondue avec une
/// session fraîche.
#[derive(Debug, Clone, Default)]
pub enum CodexThreadBootstrap {
    #[default]
    Start,
    Resume {
        thread_id: String,
    },
    Fork {
        thread_id: String,
    },
}

#[derive(Debug)]
struct ActiveTurn {
    message_id: String,
    cancel: mpsc::Sender<String>,
}

#[derive(Debug)]
struct QueueState {
    messages: VecDeque<BridgetMessage>,
    /// Messages à injecter dans le tour EN COURS via `turn/steer`, sans
    /// l'interrompre. Distincte de `messages` : celle-ci est le mode « queue »
    /// (attendre la fin du tour), celle-là le mode « steer ».
    steer: VecDeque<BridgetMessage>,
    active: Option<ActiveTurn>,
    /// Ferme atomiquement le couloir de pilotage dès qu’un tour est annulé
    /// ou non pilotable. Les messages suivants rejoignent alors `messages`,
    /// derrière les messages à restituer, au lieu de pouvoir les doubler.
    steering_open: bool,
    closed: bool,
    /// Tour réellement observé en interactif (humain OU remise Bridget),
    /// indépendant de la réservation RPC du worker.
    external_turn: Option<String>,
}

struct PendingProviderRequest {
    message_id: Option<String>,
    method: String,
    request_id: String,
    turn_id: Option<String>,
}

impl PendingProviderRequest {
    fn payload(&self) -> Value {
        let mut payload = json!({
            "provider": "codex",
            "method": self.method,
            "request_id": self.request_id,
            "state": "pending",
        });
        if let Some(turn_id) = &self.turn_id {
            payload["turn_id"] = Value::String(turn_id.clone());
        }
        payload
    }
}

#[derive(Default)]
struct Observations {
    events: VecDeque<ManagedEvent>,
    response_by_turn: HashMap<String, String>,
    terminal_by_turn: HashMap<String, ManagedTerminal>,
    /// Un `item/started` de type `userMessage` dont le `clientId` est présent
    /// prouve la visibilité d’un `turn/steer`. La clé porte le thread ET le tour
    /// afin qu’une sortie tardive ne solde jamais un autre tour actif.
    consumed_steers_by_turn: HashMap<(String, String), HashSet<String>>,
}

// Les deltas restent provisoires : seul le worker atteste leur présence au terminal.
#[derive(Debug, Default)]
struct CodexTurnDetail {
    message_id: String,
    thread_id: String,
    turn_id: Option<String>,
    provider_item_id: Option<String>,
    input_seen: bool,
    prompt_dispatched: bool,
    /// Nombre d'updates texte déjà journalisés (deltas ou repli
    /// `item/completed` agentMessage). Sert d'anti-doublon pour B.
    text_updates: usize,
    reasoning_seen: bool,
    summary: String,
    raw_reasoning: String,
    summary_index: Option<u64>,
    content_index: Option<u64>,
}

type ActiveTurnDetail = Arc<Mutex<Option<CodexTurnDetail>>>;

#[derive(Debug, Clone, Copy)]
enum CodexActKind {
    /// Producteur : `item/started` type=commandExecution (commande réelle).
    /// Les `outputDelta` ne sont plus journalisés ici (sortie stdout ≠ nom).
    Command,
    // Producteur réel (`item/fileChange/patchUpdated` → record_active_act).
    // Conservé : la vue doit l'afficher le jour où l'événement apparaît.
    File,
    // Idem pour `item/plan/delta` — producteur présent, zéro émission mesurée.
    Plan,
    Approval,
}

impl CodexActKind {
    fn as_update_kind(self) -> crate::JournalUpdateKind {
        match self {
            Self::Command => crate::JournalUpdateKind::Command,
            Self::File => crate::JournalUpdateKind::File,
            Self::Plan => crate::JournalUpdateKind::Plan,
            Self::Approval => crate::JournalUpdateKind::Approval,
        }
    }

    fn as_str(self) -> &'static str {
        crate::act_kind::pilot_kind_str(self.as_update_kind())
    }
}

struct ReaderContext {
    permission_observation: Arc<Mutex<Option<(String, Value, u64, Option<String>)>>>,
    cwd: String,
    waiters: Waiters,
    observations: Arc<(Mutex<Observations>, Condvar)>,
    alive: Arc<AtomicBool>,
    journal: Journal,
    queue: Arc<(Mutex<QueueState>, Condvar)>,
    pending_request: PendingRequest,
    child_requests: ChildRequests,
    forward_child_requests: bool,
    provider_session: String,
    pinned_model: Option<String>,
    active_detail: ActiveTurnDetail,
    writer: Writer,
    permissions: String,
    dynamic_tool_handler: Option<DynamicToolHandler>,
    /// SPEC-088 : posture de lancement, lue une fois dans les arguments.
    sandbox_posture: &'static str,
    interactive: bool,
    selected_thread: Arc<Mutex<Option<String>>>,
}

/// `--sandbox read-only` (définition de découverte du registre) ⇒ découverte ;
/// tout autre lancement géré ⇒ complet. Pas de troisième posture (FR-016).
fn sandbox_posture_from_args(args: &[String]) -> &'static str {
    if args
        .windows(2)
        .any(|pair| pair[0] == "--sandbox" && pair[1] == "read-only")
    {
        "discovery"
    } else {
        "complete"
    }
}

pub struct CodexAppServerTransport {
    permission_observation: Arc<Mutex<Option<(String, Value, u64, Option<String>)>>>,
    connection_id: String,
    alive: Arc<AtomicBool>,
    shutdown_started: AtomicBool,
    busy: Arc<AtomicBool>,
    private_profile_instructions: Arc<Mutex<Option<String>>>,
    queue: Arc<(Mutex<QueueState>, Condvar)>,
    queue_capacity: usize,
    writer: Writer,
    thread_id: String,
    next_id: Arc<AtomicU64>,
    waiters: Waiters,
    child: Arc<Mutex<Child>>,
    observations: Arc<(Mutex<Observations>, Condvar)>,
    active_detail: ActiveTurnDetail,
    journal: Journal,
    child_requests: ChildRequests,
    pending_request: PendingRequest,
    provider_observation: Option<ProviderObservation>,
    reader_handle: Mutex<Option<thread::JoinHandle<()>>>,
    worker_handle: Mutex<Option<thread::JoinHandle<()>>>,
    interactive_socket: Option<std::path::PathBuf>,
    quota_read_at: Mutex<Instant>,
}

impl CodexAppServerTransport {
    fn runtime_catalogue_until(
        &mut self,
        deadline: Instant,
    ) -> Result<
        Vec<crate::provider_catalogue::CatalogueModel>,
        crate::protocol::RuntimeSelectionRefusal,
    > {
        use crate::protocol::RuntimeSelectionRefusal as Refusal;
        if self.interactive_socket.is_some() {
            return Err(Refusal::Unsupported);
        }
        if !self.alive.load(Ordering::SeqCst) {
            return Err(Refusal::TargetUnavailable);
        }
        let mut cursor = Value::Null;
        let mut cursors = std::collections::HashSet::new();
        let mut ids = std::collections::HashSet::new();
        let mut result = Vec::new();
        for _ in 0..4 {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .ok_or(Refusal::CatalogueUnavailable)?;
            let response = request_with_timeout(
                &self.writer,
                &self.waiters,
                &self.next_id,
                "model/list",
                json!({"cursor":cursor,"limit":100,"includeHidden":false}),
                remaining,
            )
            .map_err(|_| Refusal::CatalogueUnavailable)?;
            let (models, next) = crate::provider_catalogue::model_page(&response.value)
                .map_err(|_| Refusal::CatalogueUnavailable)?;
            for model in models {
                if !ids.insert(model.model.clone()) {
                    return Err(Refusal::CatalogueUnavailable);
                }
                result.push(model);
            }
            if result.len() > crate::provider_catalogue::MAX_CATALOGUE_MODELS {
                return Err(Refusal::CatalogueUnavailable);
            }
            match next {
                None => return Ok(result),
                Some(next) if cursors.insert(next.clone()) => cursor = Value::String(next),
                Some(_) => return Err(Refusal::CatalogueUnavailable),
            }
        }
        Err(Refusal::CatalogueUnavailable)
    }

    pub fn spawn(options: CodexAppServerOptions) -> Result<Self, TransportError> {
        Self::spawn_with_environment(options, &[], false)
    }

    pub fn spawn_inheriting_stderr_with_environment(
        options: CodexAppServerOptions,
        environment: &[(String, String)],
    ) -> Result<Self, TransportError> {
        Self::spawn_with_environment(options, environment, true)
    }

    pub fn spawn_with_environment(
        options: CodexAppServerOptions,
        environment: &[(String, String)],
        inherit_stderr: bool,
    ) -> Result<Self, TransportError> {
        Self::spawn_endpoint(options, environment, inherit_stderr, None, None, false)
    }

    /// Réparation native : tous les échecs de négociation gardent un arrêt
    /// borné et rendent le propriétaire du processus si cet arrêt est incertain.
    pub fn spawn_repair_with_environment(
        options: CodexAppServerOptions,
        environment: &[(String, String)],
        inherit_stderr: bool,
    ) -> Result<Self, TransportError> {
        Self::spawn_endpoint(options, environment, inherit_stderr, None, None, true)
    }

    /// Un app-server privé, partagé avec la TUI native. Aucun octet terminal
    /// n'est injecté ; les deux clients utilisent le protocole fournisseur.
    pub fn spawn_interactive(
        options: CodexAppServerOptions,
        environment: &[(String, String)],
        socket: &Path,
    ) -> Result<Self, TransportError> {
        Self::spawn_endpoint(options, environment, false, Some(socket), None, false)
    }

    /// La sélection précède thread/resume, Register et la TUI : aucun fil
    /// provisoire et aucun premier tour avant attachement de Bridget.
    pub fn spawn_interactive_selecting(
        options: CodexAppServerOptions,
        environment: &[(String, String)],
        socket: &Path,
        selector: Option<InteractiveThreadSelector<'_>>,
    ) -> Result<Self, TransportError> {
        Self::spawn_endpoint(options, environment, false, Some(socket), selector, false)
    }

    pub fn thread_id(&self) -> &str {
        &self.thread_id
    }

    /// La durée de vie de la TUI borne celle de ce serveur privé. L'appelant
    /// ne peut que fermer la session ; il ne peut pas inventer une présence.
    pub fn interactive_lifetime(&self) -> Arc<AtomicBool> {
        self.alive.clone()
    }

    fn spawn_endpoint(
        mut options: CodexAppServerOptions,
        environment: &[(String, String)],
        inherit_stderr: bool,
        interactive_socket: Option<&Path>,
        selector: Option<InteractiveThreadSelector<'_>>,
        bounded_failure: bool,
    ) -> Result<Self, TransportError> {
        if options.queue_capacity == 0 {
            return Err(TransportError::DeliveryFailed(
                "queue Codex de capacité nulle".to_string(),
            ));
        }
        let mut command = Command::new(&options.command);
        crate::protocol::recheck_frozen_inputs(&options.command, environment)
            .map_err(TransportError::DeliveryFailed)?;
        validate_endpoint_bootstrap(&options, interactive_socket.is_some())?;
        command
            .args(&options.args)
            .envs(environment.iter().cloned())
            .env_remove("BRIDGET_NATIVE_CHILD_POLICY")
            .env_remove("BRIDGET_NATIVE_PERMISSION_SOURCES")
            .env_remove("BRIDGET_REMOTE_RESULT_CONTEXT")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(if inherit_stderr {
                Stdio::inherit()
            } else {
                Stdio::null()
            })
            // Le pilote et ses enfants sont une seule unité de vie. La
            // fermeture d'un tour ne laisse donc jamais un sous-processus
            // Codex tenir stdout ou le journal ouverts.
            .process_group(0);
        if let Some(socket) = interactive_socket {
            crate::codex_socket::validate_private_path(socket)
                .map_err(|error| TransportError::Io(error.to_string()))?;
            command.args(["--listen", &format!("unix://{}", socket.display())]);
            command.stdout(Stdio::null());
        }
        let mut child = command.spawn().map_err(|error| {
            TransportError::Io(format!("impossible de lancer codex app-server: {error}"))
        })?;
        let pid = child.id();
        let (lines, output): (SourceLines, Box<dyn Write + Send>) = if let Some(socket) =
            interactive_socket
        {
            let deadline = std::time::Instant::now() + Duration::from_secs(10);
            let connected = loop {
                match crate::codex_socket::connect(socket, deadline) {
                    Ok(pair) => break Ok(pair),
                    Err(error)
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
                        ) && std::time::Instant::now() < deadline =>
                    {
                        if let Ok(Some(_)) = child.try_wait() {
                            break Err(error);
                        }
                        thread::sleep(Duration::from_millis(25));
                    }
                    Err(error) => break Err(error),
                }
            };
            match connected {
                Ok((lines, output)) => (Box::new(lines), Box::new(output)),
                Err(error) => {
                    stop_interactive_server(&mut child, socket).map_err(|stop| {
                        TransportError::Io(format!(
                            "connexion Codex interactive : {error} ; {stop}"
                        ))
                    })?;
                    return Err(TransportError::Io(format!(
                        "connexion Codex interactive : {error}"
                    )));
                }
            }
        } else {
            let stdin = child
                .stdin
                .take()
                .ok_or_else(|| TransportError::Io("stdin Codex absent".to_string()))?;
            // Même convention que le SocketWriter Codex : le noyau ne doit
            // jamais retenir write_value sur un fournisseur vivant mais muet.
            let fd = stdin.as_raw_fd();
            // SAFETY: fcntl avec une commande à argument entier ne déréférence aucun pointeur.
            let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
            // SAFETY: fcntl avec une commande à argument entier ne déréférence aucun pointeur.
            if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0
            {
                let error = std::io::Error::last_os_error();
                crate::managed_session::stop_owned_child(&mut child, true, Duration::from_secs(4))
                    .map_err(|stop| {
                        TransportError::Io(format!("stdin Codex non bloquant: {error}; {stop}"))
                    })?;
                return Err(TransportError::Io(format!(
                    "stdin Codex non bloquant: {error}"
                )));
            }
            let stdout = child
                .stdout
                .take()
                .ok_or_else(|| TransportError::Io("stdout Codex absent".to_string()))?;
            (Box::new(BufReader::new(stdout).lines()), Box::new(stdin))
        };
        let writer: Writer = Arc::new(Mutex::new(Some(output)));
        let waiters = Arc::new(Mutex::new(HashMap::new()));
        let observations = Arc::new((Mutex::new(Observations::default()), Condvar::new()));
        let active_detail = Arc::new(Mutex::new(None));
        let alive = Arc::new(AtomicBool::new(true));
        let next_id = Arc::new(AtomicU64::new(1));
        let journal = Arc::new(Mutex::new(None));
        let private_profile_instructions = Arc::new(Mutex::new(None));
        let queue = Arc::new((
            Mutex::new(QueueState {
                messages: VecDeque::new(),
                steer: VecDeque::new(),
                active: None,
                steering_open: false,
                closed: false,
                external_turn: None,
            }),
            Condvar::new(),
        ));
        let pending_request = Arc::new(Mutex::new(None));
        let child_requests: ChildRequests = Arc::new(Mutex::new(Vec::new()));
        let selected_thread = Arc::new(Mutex::new(None));
        let permission_observation = Arc::new(Mutex::new(None));
        let reader_handle = spawn_reader(
            lines,
            ReaderContext {
                permission_observation: permission_observation.clone(),
                cwd: std::env::current_dir()
                    .ok()
                    .and_then(|p| p.into_os_string().into_string().ok())
                    .unwrap_or_default(),
                waiters: waiters.clone(),
                observations: observations.clone(),
                alive: alive.clone(),
                journal: journal.clone(),
                queue: queue.clone(),
                pending_request: pending_request.clone(),
                child_requests: child_requests.clone(),
                forward_child_requests: options.child_requests && interactive_socket.is_none(),
                provider_session: format!("codex-app-server-{pid}"),
                pinned_model: options.model.clone(),
                active_detail: active_detail.clone(),
                writer: writer.clone(),
                permissions: options.permissions.clone(),
                dynamic_tool_handler: options.dynamic_tool_handler.clone(),
                sandbox_posture: sandbox_posture_from_args(&options.args),
                interactive: interactive_socket.is_some(),
                selected_thread: selected_thread.clone(),
            },
        );

        let setup = (|| -> Result<String, TransportError> {
            let initialization = initialization_parameters();
            let initialize = request(&writer, &waiters, &next_id, "initialize", initialization)?;
            for required in ["userAgent", "codexHome", "platformFamily", "platformOs"] {
                if initialize.value.get(required).is_none() {
                    return Err(TransportError::DeliveryFailed(format!(
                        "initialize Codex ne retourne pas {required}"
                    )));
                }
            }
            write_notification(&writer, "initialized", json!({}))?;
            if let Some(select) = selector {
                let threads = list_interactive_threads(&writer, &waiters, &next_id)?;
                let selected = select(&threads).map_err(TransportError::DeliveryFailed)?;
                if !threads.iter().any(|thread| thread.id == selected) {
                    return Err(TransportError::DeliveryFailed(
                        "fil choisi absent du catalogue Codex".into(),
                    ));
                }
                options.thread_bootstrap = CodexThreadBootstrap::Resume {
                    thread_id: selected,
                };
            }
            let cwd =
                std::env::current_dir().map_err(|error| TransportError::Io(error.to_string()))?;
            let (thread_method, mut thread_params) = thread_bootstrap_request(
                &options.thread_bootstrap,
                &cwd,
                options.model.as_deref(),
                interactive_socket.is_some(),
            );
            if !matches!(options.thread_bootstrap, CodexThreadBootstrap::Start) {
                // Le serveur résout lui-même profils et -c. Sans projection
                // de SES réglages effectifs, resume restaure l'ancien choix
                // d'approbation/sandbox, y compris pour un équipier restreint.
                // Aucun parsing TOML ni défaut permissif parallèle à Codex.
                let config = request(
                    &writer,
                    &waiters,
                    &next_id,
                    "config/read",
                    json!({"includeLayers": false, "cwd": cwd}),
                )?;
                for (key, field) in [
                    ("approval_policy", "approvalPolicy"),
                    ("sandbox_mode", "sandbox"),
                ] {
                    if let Some(value) = config
                        .value
                        .get("config")
                        .and_then(|config| config.get(key))
                        .filter(|value| !value.is_null())
                    {
                        thread_params[field] = value.clone();
                    } else if interactive_socket.is_none() {
                        return Err(TransportError::DeliveryFailed(format!(
                            "reprise gérée refusée : configuration effective {key} absente"
                        )));
                    }
                }
                if interactive_socket.is_none() {
                    // Le worktree contrôlé du processus prime sur le cwd historique.
                    thread_params["cwd"] = json!(cwd);
                }
            }
            if interactive_socket.is_some()
                && matches!(options.thread_bootstrap, CodexThreadBootstrap::Start)
            {
                // Contrat natif explicite : la TUI 0.153.4 ne peut reprendre
                // un fil paginated vide (missing source rollout). legacy
                // matérialise le journal sans tour ni prompt artificiel.
                thread_params["historyMode"] = json!("legacy");
            }
            let thread = request(&writer, &waiters, &next_id, thread_method, thread_params)
                .map_err(|error| {
                    TransportError::DeliveryFailed(format!("{thread_method}: {error}"))
                })?;
            let history_mode = thread
                .value
                .pointer("/thread/historyMode")
                .and_then(Value::as_str);
            let compatible_history = history_mode == Some("legacy")
                || (matches!(
                    options.thread_bootstrap,
                    CodexThreadBootstrap::Resume { .. }
                ) && history_mode == Some("paginated"));
            if interactive_socket.is_some() && !compatible_history {
                return Err(TransportError::DeliveryFailed(
                    "Codex incompatible : mode d'historique partagé non attesté".into(),
                ));
            }
            if let Some((model, effort)) = runtime_from_thread_start(&thread.value) {
                push_source(
                    &observations,
                    thread.raw.clone(),
                    ManagedEventKind::RuntimeObserved { model, effort },
                );
            }
            // Le lecteur publie la réponse dans l'ordre STDIO, avant les notifications.
            let _ = request_with_timeout(
                &writer,
                &waiters,
                &next_id,
                "account/rateLimits/read",
                Value::Null,
                RATE_LIMIT_READ_TIMEOUT,
            );
            let thread_id = thread
                .value
                .pointer("/thread/id")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| {
                    TransportError::DeliveryFailed(
                        "thread/start Codex ne retourne pas thread.id".to_string(),
                    )
                })?;
            observe_native_permissions(&permission_observation, &thread_id, &thread.value);
            if matches!(&options.thread_bootstrap,
                CodexThreadBootstrap::Resume { thread_id: requested } if requested != &thread_id)
            {
                return Err(TransportError::DeliveryFailed(
                    "thread/resume Codex retourne un autre fil que celui demandé".into(),
                ));
            }
            if interactive_socket.is_some()
                && matches!(options.thread_bootstrap, CodexThreadBootstrap::Start)
            {
                // 0.153.4 : rend le fil sans premier tour reprenable par la
                // seconde connexion, sans envoyer de prompt d'amorçage.
                request(
                    &writer,
                    &waiters,
                    &next_id,
                    "thread/name/set",
                    json!({
                        "threadId": thread_id, "name": "Bridget interactif"
                    }),
                )?;
            }
            Ok(thread_id)
        })();
        let thread_id = match setup {
            Ok(thread_id) => thread_id,
            Err(error) => {
                if bounded_failure {
                    alive.store(false, Ordering::SeqCst);
                    if let Ok(mut queue) = queue.0.try_lock() {
                        queue.closed = true;
                    }
                    observations.1.notify_all();
                    if let Ok(mut writer) = writer.try_lock() {
                        writer.take();
                    }
                    let mut owner = crate::transport::RetainedStart::new(child, reader_handle);
                    if owner.stop(Duration::from_secs(4)).is_err() {
                        return Err(TransportError::StartUnconfirmed {
                            reason: error.to_string(),
                            owner: Box::new(owner),
                        });
                    }
                    return Err(error);
                }
                writer.lock().recover_poison().take();
                if let Some(socket) = interactive_socket {
                    stop_interactive_server(&mut child, socket)
                        .map_err(|stop| TransportError::Io(format!("{error} ; {stop}")))?;
                } else {
                    let _ = child.kill();
                    let _ = child.wait();
                }
                let _ = reader_handle.join();
                return Err(error);
            }
        };
        *selected_thread.lock().recover_poison() = Some(thread_id.clone());
        let child = Arc::new(Mutex::new(child));
        let busy = Arc::new(AtomicBool::new(false));
        let worker_handle = spawn_worker(Worker {
            native_policy: environment
                .iter()
                .rev()
                .find(|(key, _)| key == "BRIDGET_NATIVE_CHILD_POLICY")
                .map(|(_, value)| value.clone())
                .or_else(|| std::env::var("BRIDGET_NATIVE_CHILD_POLICY").ok())
                .and_then(|raw| serde_json::from_str(&raw).ok()),
            queue: queue.clone(),
            writer: writer.clone(),
            waiters: waiters.clone(),
            next_id: next_id.clone(),
            observations: observations.clone(),
            alive: alive.clone(),
            busy: busy.clone(),
            thread_id: thread_id.clone(),
            journal: journal.clone(),
            private_profile_instructions: private_profile_instructions.clone(),
            active_detail: active_detail.clone(),
            pending_request: pending_request.clone(),
            child_requests: child_requests.clone(),
            notify_timeout: Duration::from_secs(options.notify_timeout_secs),
            interactive: interactive_socket.is_some(),
        });

        push_internal(
            &observations,
            ManagedEventKind::ProviderContextObserved {
                identity: ManagedProviderIdentity {
                    provider_kind: "codex".to_string(),
                    provider_session_id: Some(format!("codex-app-server-{pid}")),
                    provider_thread_id: Some(thread_id.clone()),
                    active_turn_id: None,
                    provider_item_id: None,
                    capabilities_revision: options
                        .provider_observation
                        .as_ref()
                        .map(|observation| observation.contract_version.clone()),
                    provider_observation: options.provider_observation.clone(),
                },
            },
        );
        Ok(Self {
            permission_observation,
            connection_id: format!("codex-app-server-{pid}"),
            alive,
            shutdown_started: AtomicBool::new(false),
            busy,
            private_profile_instructions,
            queue,
            queue_capacity: options.queue_capacity,
            writer,
            thread_id,
            next_id,
            waiters,
            child,
            provider_observation: options.provider_observation,
            observations,
            active_detail,
            journal,
            child_requests,
            pending_request,
            quota_read_at: Mutex::new(Instant::now()),
            reader_handle: Mutex::new(Some(reader_handle)),
            worker_handle: Mutex::new(Some(worker_handle)),
            interactive_socket: interactive_socket.map(Path::to_path_buf),
        })
    }

    fn push_internal(&self, kind: ManagedEventKind) {
        let raw = serde_json::to_vec(&json!({ "kind": "codex_internal" }))
            .expect("fait interne Codex sérialisable");
        self.observations
            .0
            .lock()
            .recover_poison()
            .events
            .push_back(ManagedEvent::internal(
                ManagedEventSource::CodexAppServer,
                raw,
                kind,
            ));
    }

    /// O(1), une lecture au plus par intervalle sur ce même processus.
    /// Le lecteur publie les faits fournisseur avant de solder le RPC.
    /// Un échec conserve les faits précédents et ne crée aucun tour.
    fn refresh_idle_quota(&self) {
        if !self.alive.load(Ordering::SeqCst) || self.busy.load(Ordering::SeqCst) {
            return;
        }
        let Ok(mut read_at) = self.quota_read_at.try_lock() else {
            return;
        };
        if read_at.elapsed() < RATE_LIMIT_REFRESH_INTERVAL {
            return;
        }
        *read_at = Instant::now();
        let _ = request_with_timeout(
            &self.writer,
            &self.waiters,
            &self.next_id,
            "account/rateLimits/read",
            Value::Null,
            RATE_LIMIT_READ_TIMEOUT,
        );
    }

    fn enable_journal_with_live_feed(
        &self,
        root: &Path,
        agent: &str,
        live_feed: Option<JournalLiveFeed>,
    ) -> std::io::Result<()> {
        let observations = self.observations.clone();
        let failure: JournalFailureSink = Arc::new(move |detail| {
            let raw = detail.as_bytes().to_vec();
            observations
                .0
                .lock()
                .recover_poison()
                .events
                .push_back(ManagedEvent::internal(
                    ManagedEventSource::CodexAppServer,
                    raw,
                    ManagedEventKind::JournalFailed { detail },
                ));
            observations.1.notify_all();
        });
        *self.journal.lock().recover_poison() =
            Some(JournalWriter::start_with_live_feed_and_failure(
                root,
                agent,
                &self.thread_id,
                failure,
                live_feed,
            )?);
        Ok(())
    }

    fn shutdown(&self) {
        if self.shutdown_started.swap(true, Ordering::SeqCst) {
            return;
        }
        self.alive.store(false, Ordering::SeqCst);
        self.child_requests.lock().recover_poison().clear();
        self.observations.1.notify_all();
        let (queue, wake) = &*self.queue;
        let mut queue = queue.lock().recover_poison();
        queue.closed = true;
        while let Some(message) = queue.messages.pop_front() {
            self.push_internal(ManagedEventKind::DeliveryRejected {
                message_id: message.id,
                reason: "équipier arrêté".to_string(),
            });
        }
        wake.notify_all();
        drop(queue);
        if self.interactive_socket.is_some() {
            // app-server traite SIGTERM comme un drain gracieux. Une demande
            // de permission encore affichée retiendrait donc Child::wait.
            // Annuler le tour possédé AVANT de fermer son canal de contrôle.
            let turn = self
                .active_detail
                .lock()
                .recover_poison()
                .as_ref()
                .and_then(|detail| {
                    detail
                        .turn_id
                        .clone()
                        .map(|turn| (detail.thread_id.clone(), turn))
                });
            if let Some((thread_id, turn_id)) = turn {
                let _ = request_with_timeout(
                    &self.writer,
                    &self.waiters,
                    &self.next_id,
                    "turn/interrupt",
                    json!({"threadId": thread_id, "turnId": turn_id}),
                    INTERRUPT_REQUEST_TIMEOUT,
                );
            }
        }
        self.writer.lock().recover_poison().take();
        let mut child = self.child.lock().recover_poison();
        if let Some(socket) = &self.interactive_socket {
            if let Err(error) = stop_interactive_server(&mut child, socket) {
                // Garder le socket de récupération, jamais supprimer la seule
                // référence d'un enfant dont l'arrêt n'est pas confirmé.
                self.push_internal(ManagedEventKind::Error {
                    detail: error.to_string(),
                });
            }
        } else {
            // SAFETY: kill ne déréférence aucun pointeur ; ses arguments sont des entiers.
            let _ = unsafe { libc::kill(-(child.id() as i32), libc::SIGTERM) };
            let _ = child.wait();
        }
        drop(child);
        if let Some(handle) = self.reader_handle.lock().recover_poison().take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.worker_handle.lock().recover_poison().take() {
            let _ = handle.join();
        }
        if let Some(journal) = self.journal.lock().recover_poison().take() {
            journal.stop();
        }
    }
}

impl Drop for CodexAppServerTransport {
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl Transport for CodexAppServerTransport {
    fn deliver(&mut self, message: &BridgetMessage) -> Result<(), TransportError> {
        if !self.alive.load(Ordering::SeqCst) {
            return Err(TransportError::AgentDead);
        }
        let (queue, wake) = &*self.queue;
        let mut queue = queue.lock().recover_poison();
        let human_message = message.origin == Some(MessageOrigin::Human);
        let has_active_turn = queue.active.is_some();
        // Un message humain ordinaire doit obtenir son propre tour. Le piloter
        // dans un tour en cours mélange sa réponse à celle du message actif :
        // l'utilisateur voit alors son envoi mais jamais sa réponse dédiée.
        // Le pilotage reste réservé au contrôle explicite.
        let steer_or_queue = message.intent == Some(MessageIntent::SteerCurrent)
            && message
                .references
                .iter()
                .any(|reference| reference == "steer-or-queue");
        // Choisir le repli sous le verrou avant les refus TUI/non-pilotable.
        // Une promotion ne gagne jamais le droit d'interrompre le tour actif.
        let steering_requested = message.intent == Some(MessageIntent::SteerCurrent)
            && (!steer_or_queue
                || (self.interactive_socket.is_none() && has_active_turn && queue.steering_open));
        let interrupt_requested = !steer_or_queue
            && (message.intent == Some(MessageIntent::InterruptAndStart)
                || (human_message && has_active_turn && !steering_requested));
        if self.interactive_socket.is_some() && (steering_requested || interrupt_requested) {
            drop(queue);
            self.push_internal(ManagedEventKind::DeliveryRejected {
                message_id: message.id.clone(),
                reason: "pilotage du tour interactif réservé à la TUI ; envoyer sans intention de contrôle pour la FIFO".into(),
            });
            return Ok(());
        }
        if steering_requested && (queue.active.is_none() || !queue.steering_open) {
            drop(queue);
            self.push_internal(ManagedEventKind::DeliveryRejected {
                message_id: message.id.clone(),
                reason: "pilotage Codex indisponible pour le tour courant".to_string(),
            });
            return Ok(());
        }
        let saturated = if steering_requested {
            queue.steer.len() >= self.queue_capacity
        } else {
            queue.messages.len() >= self.queue_capacity
        };
        if queue.closed || saturated {
            drop(queue);
            self.push_internal(ManagedEventKind::DeliveryRejected {
                message_id: message.id.clone(),
                reason: "file Codex pleine".to_string(),
            });
            return Ok(());
        }
        if steering_requested {
            queue.steer.push_back(message.clone());
        } else {
            let mut ordinary = message.clone();
            if steer_or_queue {
                ordinary.intent = None;
            }
            queue.messages.push_back(ordinary);
        }
        // La nouvelle demande reste FIFO, mais l'interruption vise seulement
        // le tour actif capturé sous le même verrou.
        let active_message_id = interrupt_requested
            .then(|| {
                queue
                    .active
                    .as_ref()
                    .map(|active| active.message_id.clone())
            })
            .flatten();
        wake.notify_one();
        drop(queue);
        if let Some(active_message_id) = active_message_id {
            let _ = self.cancel_delivery(&active_message_id, "interruption explicite");
        }
        Ok(())
    }

    fn is_alive(&self) -> bool {
        self.alive.load(Ordering::SeqCst)
    }

    fn connection_id(&self) -> &str {
        &self.connection_id
    }
}

impl ManagedSession for CodexAppServerTransport {
    fn stop_for_workspace_repair(
        &mut self,
        budget: Duration,
    ) -> Result<ManagedProviderIdentity, TransportError> {
        let deadline = Instant::now() + budget;
        let busy = || TransportError::DeliveryFailed("workspace_repair_busy".into());
        let uncertain =
            || TransportError::DeliveryFailed("workspace_repair_stop_unconfirmed".into());
        if self.interactive_socket.is_some() {
            return Err(TransportError::DeliveryFailed(
                "workspace_repair_unsupported".into(),
            ));
        }
        // Même ordre que record_provider_request. Ne jamais attendre un verrou
        // imbriqué : une arrivée concurrente refuse cette tentative sans effet.
        let pending = self.pending_request.try_lock().map_err(|_| busy())?;
        let (queue, wake) = &*self.queue;
        let mut queue = queue.try_lock().map_err(|_| busy())?;
        if queue.closed
            || queue.active.is_some()
            || queue.external_turn.is_some()
            || !queue.messages.is_empty()
            || !queue.steer.is_empty()
            || pending.is_some()
        {
            return Err(busy());
        }
        let children = self.child_requests.try_lock().map_err(|_| busy())?;
        let active = self.active_detail.try_lock().map_err(|_| busy())?;
        let observations = self.observations.0.try_lock().map_err(|_| busy())?;
        if !children.is_empty() || active.is_some() || !observations.events.is_empty() {
            return Err(busy());
        }
        if self
            .provider_observation
            .as_ref()
            .is_none_or(|v| !v.supports(ProviderOperation::Resume))
            || self.thread_id.is_empty()
        {
            return Err(TransportError::DeliveryFailed(
                "workspace_repair_resume_unattested".into(),
            ));
        }
        // Les champs sont ceux du pilote vivant. active est gardé à None ;
        // provider_identity() reprendrait son verrou sous queue et pending.
        let identity = ManagedProviderIdentity {
            provider_kind: "codex".into(),
            provider_session_id: Some(self.connection_id.clone()),
            provider_thread_id: Some(self.thread_id.clone()),
            active_turn_id: None,
            provider_item_id: None,
            capabilities_revision: self
                .provider_observation
                .as_ref()
                .map(|v| v.contract_version.clone()),
            provider_observation: self.provider_observation.clone(),
        };
        if self.shutdown_started.swap(true, Ordering::SeqCst) {
            return Err(uncertain());
        }
        queue.closed = true;
        self.alive.store(false, Ordering::SeqCst);
        wake.notify_all();
        self.observations.1.notify_all();
        drop(observations);
        drop(active);
        drop(children);
        drop(queue);
        drop(pending);
        self.writer.try_lock().map_err(|_| uncertain())?.take();
        let mut child = self.child.try_lock().map_err(|_| uncertain())?;
        crate::managed_session::stop_owned_child(
            &mut child,
            true,
            deadline.saturating_duration_since(Instant::now()),
        )
        .map_err(|_| uncertain())?;
        drop(child);
        for handle in [&self.reader_handle, &self.worker_handle] {
            loop {
                let finished = handle
                    .try_lock()
                    .map_err(|_| uncertain())?
                    .as_ref()
                    .is_none_or(|thread| thread.is_finished());
                if finished {
                    break;
                }
                if Instant::now() >= deadline {
                    return Err(uncertain());
                }
                thread::sleep(Duration::from_millis(10));
            }
            if let Some(thread) = handle.try_lock().map_err(|_| uncertain())?.take() {
                thread.join().map_err(|_| uncertain())?;
            }
        }
        Ok(identity)
    }

    fn answer_provider_request(
        &mut self,
        answer: &ChildRequestAnswer,
    ) -> Result<(), TransportError> {
        if !self.alive.load(Ordering::SeqCst) {
            return Err(TransportError::AgentDead);
        }
        let mut pending = self.child_requests.lock().recover_poison();
        let index = pending
            .iter()
            .position(|entry| {
                entry.request.provider_session == answer.provider_session
                    && entry.request.turn_id == answer.turn_id
                    && entry.request.provider_request_id == answer.provider_request_id
            })
            .ok_or_else(|| TransportError::DeliveryFailed("request_unknown".into()))?;
        let entry = &pending[index];
        if entry.uncertain {
            return Err(TransportError::Io(
                "child_request_write_uncertain: attente déjà incertaine".into(),
            ));
        }
        let linked = self
            .active_detail
            .lock()
            .recover_poison()
            .as_ref()
            .is_some_and(|detail| {
                detail.message_id == entry.message_id
                    && detail.thread_id == entry.thread_id
                    && detail.turn_id.as_deref() == Some(answer.turn_id.as_str())
            });
        if !linked {
            return Err(TransportError::DeliveryFailed("request_unlinked".into()));
        }
        let id = serde_json::to_value(&answer.provider_request_id)
            .map_err(|e| TransportError::DeliveryFailed(e.to_string()))?;
        let reply = match entry.request.kind {
            ChildRequestKind::ApprovalCommand => {
                if answer.answer.is_some() {
                    return Err(TransportError::DeliveryFailed("answer_invalid".into()));
                }
                let wanted = match answer.decision {
                    ChildRequestDecision::Accept => "accept",
                    ChildRequestDecision::Decline => "decline",
                    ChildRequestDecision::Cancel => "cancel",
                };
                // Réemploi unique du repli T000. Une décision explicite offerte
                // est isolée ; un refus peut seulement retomber sur cancel.
                let mut original = entry.approval.clone();
                if entry.request.offered.contains(&answer.decision) {
                    original["params"]["availableDecisions"] = json!([wanted]);
                } else if answer.decision == ChildRequestDecision::Accept {
                    return Err(TransportError::DeliveryFailed(
                        "decision_not_offered".into(),
                    ));
                }
                let permission = if answer.decision == ChildRequestDecision::Accept {
                    "allow"
                } else {
                    "deny"
                };
                approval_response(&original, permission)
                    .ok_or_else(|| TransportError::DeliveryFailed("request_unknown".into()))?
                    .0
            }
            ChildRequestKind::UserInput => {
                if answer.decision != ChildRequestDecision::Accept {
                    json!({"id":id,"error":{"code":-32000,"message":"child_request_refused"}})
                } else {
                    let input = answer
                        .answer
                        .as_ref()
                        .and_then(Value::as_object)
                        .ok_or_else(|| TransportError::DeliveryFailed("answer_invalid".into()))?;
                    if serde_json::to_vec(input).map_or(true, |bytes| bytes.len() > 32 * 1024)
                        || input.len() != entry.request.question_ids.len()
                        || entry.request.question_ids.iter().any(|qid| {
                            !input.get(qid).is_some_and(|value| {
                                value
                                    .as_array()
                                    .is_some_and(|values| values.iter().all(Value::is_string))
                            })
                        })
                    {
                        return Err(TransportError::DeliveryFailed("answer_invalid".into()));
                    }
                    let nested: serde_json::Map<String, Value> = input
                        .iter()
                        .map(|(qid, values)| (qid.clone(), json!({"answers":values})))
                        .collect();
                    json!({"id":id,"result":{"answers":nested}})
                }
            }
            _ => {
                return Err(TransportError::DeliveryFailed(
                    "approval_method_unsupported".into(),
                ));
            }
        };
        // Le verrou sérialise answer/fin/resolved. Une écriture partielle ne
        // prouve aucun refus : garder l'incertitude et fermer l'admission.
        if let Err(error) = write_value(&self.writer, reply) {
            pending[index].uncertain = true;
            self.alive.store(false, Ordering::SeqCst);
            self.observations.1.notify_all();
            return Err(TransportError::Io(format!(
                "child_request_write_uncertain: {error}"
            )));
        }
        pending.remove(index);
        Ok(())
    }
    fn provider_permissions(&self) -> Option<(String, Value, u64, Option<String>)> {
        self.permission_observation.lock().recover_poison().clone()
    }
    fn runtime_catalogue(&mut self) -> crate::protocol::RuntimeCatalogueOutcome {
        match self.runtime_catalogue_until(Instant::now() + Duration::from_secs(2)) {
            Ok(models) => crate::protocol::RuntimeCatalogueOutcome::Observed { models },
            Err(reason) => crate::protocol::RuntimeCatalogueOutcome::Refused { reason },
        }
    }
    fn select_runtime(
        &mut self,
        selection: crate::protocol::RuntimeSelection,
    ) -> crate::protocol::RuntimeSelectionOutcome {
        use crate::protocol::{
            RuntimeSelectionOutcome as Outcome, RuntimeSelectionRefusal as Refusal,
        };
        let refuse = |reason| Outcome::Refused { reason };
        if !selection.valid() {
            return refuse(Refusal::InvalidSelection);
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        let models = match self.runtime_catalogue_until(deadline) {
            Ok(models) => models,
            Err(reason) => return refuse(reason),
        };
        let Some(model) = models.iter().find(|model| model.model == selection.model) else {
            return refuse(Refusal::ModelUnavailable);
        };
        if !model.supported_efforts.contains(&selection.effort) {
            return refuse(Refusal::EffortUnavailable);
        }
        let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
            return refuse(Refusal::CatalogueUnavailable);
        };
        // Le fil natif confirme le réglage ; aucun tour de convenance n'est créé.
        match request_with_timeout(
            &self.writer,
            &self.waiters,
            &self.next_id,
            "thread/settings/update",
            json!({"threadId":self.thread_id,"model":selection.model,"effort":selection.effort}),
            remaining,
        ) {
            Ok(_) => Outcome::Selected { selection },
            Err(error) if error.to_string().contains("-32601") => refuse(Refusal::Unsupported),
            Err(_) => Outcome::OutcomeUnknown {},
        }
    }
    fn set_private_profile_instructions(
        &mut self,
        instructions: &str,
    ) -> Result<(), TransportError> {
        *self.private_profile_instructions.lock().recover_poison() = Some(instructions.to_string());
        Ok(())
    }

    fn descriptor(&self) -> ManagedSessionDescriptor {
        ManagedSessionDescriptor {
            transport: "stdio".to_string(),
            mode: PresenceMode::Cli,
            location: None,
        }
    }

    fn provider_identity(&self) -> Option<ManagedProviderIdentity> {
        let active = self.active_detail.lock().recover_poison();
        Some(ManagedProviderIdentity {
            provider_kind: "codex".to_string(),
            provider_session_id: Some(self.connection_id.clone()),
            provider_thread_id: Some(self.thread_id.clone()),
            active_turn_id: active.as_ref().and_then(|detail| detail.turn_id.clone()),
            provider_item_id: active
                .as_ref()
                .and_then(|detail| detail.provider_item_id.clone()),
            capabilities_revision: self
                .provider_observation
                .as_ref()
                .map(|observation| observation.contract_version.clone()),
            provider_observation: self.provider_observation.clone(),
        })
    }
    fn process_id(&self) -> u32 {
        self.child.lock().recover_poison().id()
    }

    fn activate_journal(
        &self,
        root: &Path,
        agent: &str,
        live_feed: Option<JournalLiveFeed>,
    ) -> std::io::Result<()> {
        self.enable_journal_with_live_feed(root, agent, live_feed)
    }

    fn journal_writer(&self) -> Option<(JournalWriter, String)> {
        self.journal
            .lock()
            .ok()
            .and_then(|journal| journal.clone())
            .map(|writer| (writer, self.thread_id.clone()))
    }

    fn drain_events(&self) -> Vec<ManagedEvent> {
        self.refresh_idle_quota();
        self.observations
            .0
            .lock()
            .recover_poison()
            .events
            .drain(..)
            .collect()
    }

    fn cancel_delivery(&self, message_id: &str, reason: &str) -> bool {
        let (queue, wake) = &*self.queue;
        let mut queue = queue.lock().recover_poison();
        if let Some(active) = queue
            .active
            .as_ref()
            .filter(|active| active.message_id == message_id)
        {
            self.child_requests
                .lock()
                .recover_poison()
                .retain(|entry| entry.message_id != message_id);
            let _ = active.cancel.send(reason.to_string());
            queue.steering_open = false;
            return true;
        }
        let Some(position) = queue
            .messages
            .iter()
            .position(|message| message.id == message_id)
        else {
            return false;
        };
        let message = queue
            .messages
            .remove(position)
            .expect("message Codex présent");
        wake.notify_one();
        drop(queue);
        self.push_internal(ManagedEventKind::DeliveryRejected {
            message_id: message.id,
            reason: reason.to_string(),
        });
        true
    }

    fn stop(&self) {
        self.shutdown();
    }

    fn stop_native_mission(&self, message_id: &str) {
        // cancel_delivery passe par le worker. shutdown ferme son alive avant
        // qu'il puisse lire ce canal : l'arrêt natif interrompt donc le tour
        // exact directement, tant que le writer fournisseur est encore ouvert.
        let turn = self
            .active_detail
            .lock()
            .recover_poison()
            .as_ref()
            .filter(|detail| detail.message_id == message_id)
            .and_then(|detail| {
                detail
                    .turn_id
                    .clone()
                    .map(|turn| (detail.thread_id.clone(), turn))
            });
        // Fermer l'admission avant le contrôle fournisseur : sa réponse ne
        // doit pas permettre au worker de prendre une nouvelle remise.
        self.alive.store(false, Ordering::SeqCst);
        let (queue, wake) = &*self.queue;
        queue.lock().recover_poison().closed = true;
        wake.notify_all();
        if let Some((thread_id, turn_id)) = turn {
            let _ = request_with_timeout(
                &self.writer,
                &self.waiters,
                &self.next_id,
                "turn/interrupt",
                json!({"threadId": thread_id, "turnId": turn_id}),
                INTERRUPT_REQUEST_TIMEOUT,
            );
        }
        self.shutdown();
    }

    fn is_busy(&self) -> bool {
        self.busy.load(Ordering::SeqCst)
            || self.queue.0.lock().recover_poison().external_turn.is_some()
    }
}

struct Worker {
    native_policy: Option<Value>,
    interactive: bool,
    queue: Arc<(Mutex<QueueState>, Condvar)>,
    writer: Writer,
    waiters: Waiters,
    next_id: Arc<AtomicU64>,
    observations: Arc<(Mutex<Observations>, Condvar)>,
    alive: Arc<AtomicBool>,
    busy: Arc<AtomicBool>,
    thread_id: String,
    journal: Journal,
    private_profile_instructions: Arc<Mutex<Option<String>>>,
    active_detail: ActiveTurnDetail,
    pending_request: PendingRequest,
    child_requests: ChildRequests,
    notify_timeout: Duration,
}

fn spawn_worker(worker: Worker) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let (message, cancel) = {
                let (queue, wake) = &*worker.queue;
                let mut queue = queue.lock().recover_poison();
                while (queue.messages.is_empty() || queue.external_turn.is_some()) && !queue.closed
                {
                    queue = wake.wait(queue).recover_poison();
                }
                if queue.closed {
                    break;
                }
                let message = queue.messages.pop_front().expect("file Codex non vide");
                let (sender, receiver) = mpsc::channel();
                queue.active = Some(ActiveTurn {
                    message_id: message.id.clone(),
                    cancel: sender,
                });
                queue.steering_open = true;
                (message, receiver)
            };
            if !worker.alive.load(Ordering::SeqCst) {
                push_internal(
                    &worker.observations,
                    ManagedEventKind::DeliveryRejected {
                        message_id: message.id,
                        reason: "équipier arrêté".to_string(),
                    },
                );
                clear_active(&worker.queue);
                continue;
            }
            worker.busy.store(true, Ordering::SeqCst);
            if !worker.interactive {
                push_internal(
                    &worker.observations,
                    ManagedEventKind::TurnStarted {
                        message_id: message.id.clone(),
                    },
                );
                record_or_terminal(
                    &worker.journal,
                    &worker.observations,
                    "turn_start",
                    Some(&message.id),
                    json!({
                        "from": &message.from,
                        "body": message.body,
                    }),
                );
                *worker.active_detail.lock().recover_poison() = Some(CodexTurnDetail {
                    message_id: message.id.clone(),
                    thread_id: worker.thread_id.clone(),
                    ..CodexTurnDetail::default()
                });
            }
            let started = SystemTime::now();
            let result = start_turn_with_retry(&worker, &message);
            let turn_started = result.is_ok();
            let event = match result {
                Ok(turn_id) => {
                    if !worker.interactive {
                        set_active_turn_id(&worker.active_detail, &message.id, &turn_id);
                        push_internal(
                            &worker.observations,
                            ManagedEventKind::PromptDispatched {
                                message_id: message.id.clone(),
                            },
                        );
                        record_or_terminal(
                            &worker.journal,
                            &worker.observations,
                            "prompt_dispatched",
                            Some(&message.id),
                            json!({
                                "from": &message.from,
                                "body": &message.body,
                            }),
                        );
                    }
                    wait_for_turn(&worker, &message, &turn_id, &cancel, started)
                }
                Err(reason) => ManagedEventKind::DeliveryRejected {
                    message_id: message.id.clone(),
                    reason,
                },
            };
            if !worker.interactive && turn_started {
                let reasoning = finish_reasoning(&worker.active_detail, &message.id);
                record_or_terminal(
                    &worker.journal,
                    &worker.observations,
                    "reasoning",
                    Some(&message.id),
                    reasoning,
                );
            } else if !worker.interactive {
                worker.active_detail.lock().recover_poison().take();
            }
            // (C) Succès / annulation attestés → turn_end avec issue exacte.
            // L'échéance (DeliveryRejected « échéance Codex dépassée ») n'écrit
            // PAS de turn_end : c'est une cause distincte du « inconnu » Claude
            // (succès + payload vide). Ne pas fusionner les deux.
            if let ManagedEventKind::TurnFinished { terminal, .. } = &event
                && !worker.interactive
            {
                let stop_reason = match terminal {
                    ManagedTerminal::Completed => "completed".to_string(),
                    ManagedTerminal::Cancelled => "cancelled".to_string(),
                    ManagedTerminal::Failed { detail } => detail.clone(),
                };
                let mut payload = json!({ "stop_reason": stop_reason });
                if message.reply {
                    payload["routed_to"] = json!(&message.from);
                }
                record_or_terminal(
                    &worker.journal,
                    &worker.observations,
                    "turn_end",
                    Some(&message.id),
                    payload,
                );
                take_pending_request(&worker.pending_request, &message.id);
            } else if let ManagedEventKind::DeliveryRejected { reason, .. } = &event {
                let mut payload = json!({ "reason": reason });
                if let Some(pending) = take_pending_request(&worker.pending_request, &message.id) {
                    payload["pending_provider_request"] = pending.payload();
                }
                record_or_terminal(
                    &worker.journal,
                    &worker.observations,
                    "error",
                    Some(&message.id),
                    payload,
                );
            }
            worker
                .child_requests
                .lock()
                .recover_poison()
                .retain(|entry| entry.message_id != message.id);
            push_internal(&worker.observations, event);
            worker.busy.store(false, Ordering::SeqCst);
            clear_active(&worker.queue);
        }
    })
}

fn private_prompt(instructions: Option<&str>, body: &str) -> String {
    let Some(instructions) = instructions
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return body.to_string();
    };
    format!("[Instructions individuelles Bridget]\n{instructions}\n\n[Demande]\n{body}")
}

fn communication_prompt(message: &BridgetMessage, interactive: bool) -> String {
    let response = if !message.reply {
        bridget_core::envelope::no_reply_notice(message)
    } else if interactive {
        "Si reply=true, réponds une seule fois par l'outil bridget_send avec to=from et in_reply_to=id ci-dessus. La réponse finale à l'écran n'est pas envoyée à cet agent."
    } else {
        "Si reply=true, le wrapper relaie automatiquement ta réponse finale à from avec in_reply_to=id ci-dessus. Utilise cette réponse finale pour terminer la demande ; pour un avancement distinct, utilise bridget_send avec to=from et reply=false, sans in_reply_to."
    };
    let mut metadata = json!({"from": message.from, "to": message.to, "id": message.id,
        "reply": message.reply, "in_reply_to": message.in_reply_to});
    if let Some(origin) = &message.delegated_origin {
        metadata["delegated_origin"] = json!(origin);
        metadata["sender_label"] = json!(message.sender_label());
    }
    format!(
        "[Message Bridget : {metadata}]\n{}\n\n[Réponse Bridget]\n{response}\nPour communiquer, utilise les outils MCP Bridget (bridget_send, bridget_who, bridget_ledger). S'ils ne sont pas affichés, cherche-les dans le catalogue d'outils disponible. L'accès à la socket depuis le shell restreint n'est pas requis ; un refus du shell ne prouve pas une panne MCP.",
        message.body
    )
}

/// Même initialisation pour l'observation sans tour et le transport natif.
pub fn initialization_parameters() -> Value {
    json!({"clientInfo":{"name":"bridget","version":env!("CARGO_PKG_VERSION")},"capabilities":{"experimentalApi":true}})
}
fn turn_start_parameters(
    thread: &str,
    message: &str,
    prompt: &str,
    policy: Option<&Value>,
    cwd: Option<&Path>,
) -> Value {
    let mut params = json!({"threadId":thread,"clientUserMessageId":message,"input":[{"type":"text","text":prompt}]});
    if let Some(policy) = policy {
        params["approvalPolicy"] = policy["approval_policy"].clone();
        params["approvalsReviewer"] = policy["approvals_reviewer"].clone();
        params["sandboxPolicy"] = policy["sandbox_policy"].clone();
        if let Some(cwd) = cwd {
            params["cwd"] = json!(cwd);
        }
    }
    params
}
/// Témoins des enveloppes émises : aucune requête n'est envoyée par ce helper.
pub fn program_contract_requests() -> Vec<(&'static str, Value)> {
    let id = "00000000-0000-4000-8000-000000000001";
    let cwd = Path::new("/tmp");
    let policy = json!({"approval_policy":"never","approvals_reviewer":"user","sandbox_policy":{"type":"readOnly","networkAccess":false}});
    let start = thread_bootstrap_request(
        &CodexThreadBootstrap::Start,
        cwd,
        Some("gpt-6.1-sol"),
        false,
    );
    let resume = thread_bootstrap_request(
        &CodexThreadBootstrap::Resume {
            thread_id: id.into(),
        },
        cwd,
        Some("gpt-6.1-sol"),
        false,
    );
    let fork = thread_bootstrap_request(
        &CodexThreadBootstrap::Fork {
            thread_id: id.into(),
        },
        cwd,
        Some("gpt-6.1-sol"),
        false,
    );
    vec![
        ("initialize", initialization_parameters()),
        start,
        (
            "turn/start",
            turn_start_parameters(id, id, "bridget-contract", Some(&policy), Some(cwd)),
        ),
        ("turn/interrupt", json!({"threadId":id,"turnId":id})),
        (
            "turn/steer",
            json!({"threadId":id,"expectedTurnId":id,"input":[{"type":"text","text":"bridget-contract"}]}),
        ),
        resume,
        fork,
    ]
}

fn start_turn_with_retry(worker: &Worker, message: &BridgetMessage) -> Result<String, String> {
    let instructions = worker
        .private_profile_instructions
        .lock()
        .recover_poison()
        .clone();
    // Les outils exposent leurs capacités ; le pilote ne promet ni interface
    // de rendu ni disponibilité du sandbox et ne transforme pas un document
    // HTML en ordre de publication ou d'exécution.
    // L'enveloppe est une projection fournisseur ; le corps durable reste intact.
    let body = communication_prompt(message, worker.interactive);
    let prompt = private_prompt(instructions.as_deref(), &body);
    let cwd = if worker.native_policy.is_some() {
        Some(std::env::current_dir().map_err(|_| "cwd_unavailable")?)
    } else {
        None
    };
    let params = turn_start_parameters(
        &worker.thread_id,
        &message.id,
        &prompt,
        worker.native_policy.as_ref(),
        cwd.as_deref(),
    );
    for attempt in 0..=SATURATION_RETRIES {
        match request(
            &worker.writer,
            &worker.waiters,
            &worker.next_id,
            "turn/start",
            params.clone(),
        ) {
            Ok(result) => {
                return result
                    .value
                    .pointer("/turn/id")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .ok_or_else(|| "turn/start Codex ne retourne pas turn.id".to_string());
            }
            Err(TransportError::DeliveryFailed(reason))
                if is_saturated(&reason) && attempt < SATURATION_RETRIES =>
            {
                thread::sleep(saturation_delay(attempt, &message.id));
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Err("saturation Codex persistante".to_string())
}

fn observe_native_permissions(
    observation: &Arc<Mutex<Option<(String, Value, u64, Option<String>)>>>,
    session: &str,
    value: &Value,
) {
    let mut observed = observation.lock().recover_poison();
    let previous = observed.as_ref().filter(|o| o.0 == session);
    let mut policy = previous
        .map(|o| o.1.clone())
        .unwrap_or_else(|| json!({"kind":"codex"}));
    for (field, source) in [
        ("approval_policy", "approvalPolicy"),
        ("approvals_reviewer", "approvalsReviewer"),
        ("sandbox_policy", "sandboxPolicy"),
    ] {
        if let Some(value) = value.get(source).or_else(|| {
            if source == "sandboxPolicy" {
                value.get("sandbox")
            } else {
                None
            }
        }) {
            policy[field] = value.clone();
        }
    }
    let cwd = match value.get("cwd") {
        Some(value) => value.as_str().map(str::to_owned),
        None => previous.and_then(|o| o.3.clone()),
    };
    let complete = ["approval_policy", "approvals_reviewer", "sandbox_policy"]
        .iter()
        .all(|k| policy.get(*k).is_some());
    if complete {
        let revision = previous
            .map(|o| o.2.saturating_add(u64::from(o.1 != policy || o.3 != cwd)))
            .unwrap_or(1);
        *observed = Some((session.into(), policy, revision, cwd));
    }
}

fn wait_for_turn(
    worker: &Worker,
    message: &BridgetMessage,
    turn_id: &str,
    cancel: &mpsc::Receiver<String>,
    started: SystemTime,
) -> ManagedEventKind {
    let deadline = message
        .deadline_at
        .map(|seconds| UNIX_EPOCH + Duration::from_secs(seconds))
        .unwrap_or_else(|| started + worker.notify_timeout);
    let mut interrupted = false;
    let mut interrupt_terminal_deadline = None;
    let mut steering_allowed = true;
    let mut deadline_interrupted = false;
    // Un accusé `turn/steer` ne prouve jamais la remise. Chaque message reste
    // dans `accepted_pending` jusqu’à sa complétion `userMessage` corrélée.
    let mut accepted_pending: Vec<BridgetMessage> = Vec::new();
    loop {
        if !worker.alive.load(Ordering::SeqCst) {
            requeue_unconfirmed(&worker.queue, &mut accepted_pending);
            return ManagedEventKind::DeliveryRejected {
                message_id: message.id.clone(),
                reason: "stdout Codex fermé pendant le tour".to_string(),
            };
        }
        if !interrupted && cancel.try_recv().is_ok() {
            interrupted = true;
            let _ = request(
                &worker.writer,
                &worker.waiters,
                &worker.next_id,
                "turn/interrupt",
                json!({ "threadId": worker.thread_id, "turnId": turn_id }),
            );
            requeue_unconfirmed(&worker.queue, &mut accepted_pending);
        }
        if !interrupted && steering_allowed {
            steering_allowed = steer_into_turn(worker, turn_id, &mut accepted_pending);
        }
        // La notification de consommation peut arriver pendant la réponse
        // JSON-RPC de `turn/steer`. La traiter avant l'échéance évite
        // d'interrompre un tour dont la remise humaine est déjà attestée.
        let consumed = {
            let (lock, _) = &*worker.observations;
            let mut observed = lock.lock().recover_poison();
            consume_accepted_steers(
                &mut observed,
                &worker.thread_id,
                turn_id,
                &mut accepted_pending,
            )
        };
        confirm_consumed_steers(worker, turn_id, consumed);
        let now = SystemTime::now();
        // Un tour ancien peut être long, mais sa propre échéance ne doit pas
        // devenir celle d'un humain arrivé par `turn/steer`. Tant qu'aucune
        // complétion userMessage corrélée ne l'atteste, on bascule sur le
        // repli `turn/interrupt` à l'échéance de CE message humain.
        if !interrupted
            && earliest_unconfirmed_steer_deadline(&worker.queue, &accepted_pending)
                .is_some_and(|steer_deadline| now >= steer_deadline)
        {
            interrupted = true;
            let _ = request_with_timeout(
                &worker.writer,
                &worker.waiters,
                &worker.next_id,
                "turn/interrupt",
                json!({ "threadId": worker.thread_id, "turnId": turn_id }),
                INTERRUPT_REQUEST_TIMEOUT,
            );
            requeue_unconfirmed(&worker.queue, &mut accepted_pending);
            interrupt_terminal_deadline = Some(now + INTERRUPT_TERMINAL_TIMEOUT);
        }
        let (lock, wake) = &*worker.observations;
        let mut observed = lock.lock().recover_poison();
        let consumed = consume_accepted_steers(
            &mut observed,
            &worker.thread_id,
            turn_id,
            &mut accepted_pending,
        );
        let terminal = observed.terminal_by_turn.remove(turn_id);
        let response = terminal.as_ref().map(|_| {
            observed
                .response_by_turn
                .remove(turn_id)
                .unwrap_or_default()
        });
        if terminal.is_some() {
            observed
                .consumed_steers_by_turn
                .remove(&(worker.thread_id.clone(), turn_id.to_string()));
        }
        drop(observed);
        confirm_consumed_steers(worker, turn_id, consumed);
        if let Some(terminal) = terminal {
            if deadline_interrupted {
                requeue_unconfirmed(&worker.queue, &mut accepted_pending);
                return ManagedEventKind::DeliveryRejected {
                    message_id: message.id.clone(),
                    reason: "échéance Codex dépassée".to_string(),
                };
            }
            // Même un terminal `completed` ne solde pas un pilotage sans
            // complétion corrélée : la restitution FIFO est donc obligatoire.
            requeue_unconfirmed(&worker.queue, &mut accepted_pending);
            return ManagedEventKind::TurnFinished {
                message: message.clone(),
                response: response.unwrap_or_default(),
                terminal,
            };
        }
        let now = SystemTime::now();
        // L'échéance du fournisseur ne doit pas détacher le processus Codex.
        // Elle déclenche le même arrêt borné que les autres fins de tour, puis
        // rapporte l'échec au message qui a atteint ce plafond.
        if !interrupted && now >= deadline {
            interrupted = true;
            deadline_interrupted = true;
            let _ = request_with_timeout(
                &worker.writer,
                &worker.waiters,
                &worker.next_id,
                "turn/interrupt",
                json!({ "threadId": worker.thread_id, "turnId": turn_id }),
                INTERRUPT_REQUEST_TIMEOUT,
            );
            requeue_unconfirmed(&worker.queue, &mut accepted_pending);
            interrupt_terminal_deadline = Some(now + INTERRUPT_TERMINAL_TIMEOUT);
            continue;
        }
        let terminal_deadline = interrupt_terminal_deadline
            .or(Some(deadline))
            .unwrap_or(deadline);
        if now >= terminal_deadline {
            requeue_unconfirmed(&worker.queue, &mut accepted_pending);
            return ManagedEventKind::DeliveryRejected {
                message_id: message.id.clone(),
                reason: if interrupt_terminal_deadline
                    .is_some_and(|interrupt_deadline| now >= interrupt_deadline)
                {
                    if deadline_interrupted {
                        "interruption Codex sans terminal après échéance fournisseur".to_string()
                    } else {
                        "interruption Codex sans terminal après pilotage humain non attesté"
                            .to_string()
                    }
                } else {
                    "échéance Codex dépassée".to_string()
                },
            };
        }
        let remaining = terminal_deadline.duration_since(now).unwrap_or(TURN_POLL);
        let wait = remaining.min(TURN_POLL);
        let observed = lock.lock().recover_poison();
        let (next, _) = wake.wait_timeout(observed, wait).recover_poison();
        drop(next);
    }
}

/// Échéance la plus proche des messages humains qui ne sont pas encore
/// attestés. Les messages dans `accepted` ont reçu un accusé `turn/steer`,
/// ceux de `QueueState::steer` ne l'ont même pas reçu : les deux doivent être
/// protégés par la même borne et restituer leur ordre FIFO au repli.
fn earliest_unconfirmed_steer_deadline(
    queue: &Arc<(Mutex<QueueState>, Condvar)>,
    accepted: &[BridgetMessage],
) -> Option<SystemTime> {
    let (lock, _) = &**queue;
    let state = lock.lock().recover_poison();
    accepted
        .iter()
        .chain(state.steer.iter())
        // Human est normalisé par le daemon avant la remise. Le nom de
        // l'émetteur et une deadline fournie ne valent aucune autorité.
        .filter(|message| {
            message.origin == Some(MessageOrigin::Human)
                && message.intent == Some(MessageIntent::SteerCurrent)
        })
        .filter_map(|message| {
            message
                .deadline_at
                .map(|seconds| UNIX_EPOCH + Duration::from_secs(seconds))
        })
        .min()
}

fn consume_accepted_steers(
    observed: &mut Observations,
    thread_id: &str,
    turn_id: &str,
    accepted: &mut Vec<BridgetMessage>,
) -> Vec<BridgetMessage> {
    let key = (thread_id.to_string(), turn_id.to_string());
    let Some(mut consumed) = observed.consumed_steers_by_turn.remove(&key) else {
        return Vec::new();
    };
    let mut confirmed = Vec::new();
    let mut remaining = Vec::with_capacity(accepted.len());
    for message in std::mem::take(accepted) {
        if consumed.remove(&message.id) {
            confirmed.push(message);
        } else {
            remaining.push(message);
        }
    }
    *accepted = remaining;
    if !consumed.is_empty() {
        observed.consumed_steers_by_turn.insert(key, consumed);
    }
    confirmed
}

fn confirm_consumed_steers(worker: &Worker, turn_id: &str, consumed: Vec<BridgetMessage>) {
    for message in consumed {
        push_internal(
            &worker.observations,
            ManagedEventKind::PromptDispatched {
                message_id: message.id.clone(),
            },
        );
        record_or_terminal(
            &worker.journal,
            &worker.observations,
            "prompt_dispatched",
            Some(&message.id),
            json!({
                "from": &message.from,
                "body": &message.body,
                "turn_id": turn_id,
                "via": "turn/steer",
            }),
        );
    }
}

/// Restitue en tête de FIFO les messages humains non attestés. Les accusés
/// précèdent ceux jamais acceptés, et tous précèdent la file déjà en attente.
fn requeue_unconfirmed(
    queue: &Arc<(Mutex<QueueState>, Condvar)>,
    pending: &mut Vec<BridgetMessage>,
) {
    let (lock, wake) = &**queue;
    let mut state = lock.lock().recover_poison();
    state.steering_open = false;
    let steered: Vec<_> = state.steer.drain(..).collect();
    for mut message in steered.into_iter().rev().chain(pending.drain(..).rev()) {
        if message.intent == Some(MessageIntent::SteerCurrent)
            && message
                .references
                .iter()
                .any(|reference| reference == "steer-or-queue")
        {
            message.intent = None;
        }
        state.messages.push_front(message);
    }
    wake.notify_one();
}

/// Injecte dans le tour ACTIF les messages en attente de pilotage.
///
/// Trois contraintes tenues ici, documentées par openclaw :
/// - le délai d’attente est borné (`request`, REQUEST_TIMEOUT) : `turn/steer`
///   n’est qu’un accusé et rien ne garantit une réponse. Sans borne, l’appelant
///   ne se débloquerait qu’à la fermeture du client et bloquerait TOUS les
///   pilotages suivants derrière lui (attempt-steering.ts:184-187) ;
/// - `expectedTurnId` porte le tour actif — jamais un identifiant retourné pour
///   un tour mis en file (t3code, CodexSessionRuntime.ts:1853) ;
/// - en cas de rejet, le message est remis EN TÊTE et le drainage s’arrête pour
///   ce passage : le suivant ne doit pas doubler celui qui vient d’échouer
///   (attempt-steering.ts:218-219).
fn steer_into_turn(worker: &Worker, turn_id: &str, accepted: &mut Vec<BridgetMessage>) -> bool {
    loop {
        let next = {
            let (lock, _) = &*worker.queue;
            let mut state = lock.lock().recover_poison();
            state.steer.pop_front()
        };
        let Some(message) = next else {
            return true;
        };
        let outcome = request_with_timeout(
            &worker.writer,
            &worker.waiters,
            &worker.next_id,
            "turn/steer",
            json!({
                "threadId": worker.thread_id,
                "expectedTurnId": turn_id,
                "clientUserMessageId": message.id,
                "input": [{ "type": "text", "text": communication_prompt(&message, worker.interactive) }],
            }),
            STEER_REQUEST_TIMEOUT,
        );
        match outcome {
            Ok(_) => {
                record_or_terminal(
                    &worker.journal,
                    &worker.observations,
                    "turn_steer",
                    Some(&message.id),
                    json!({ "from": &message.from, "turn_id": turn_id }),
                );
                // Accusé reçu : accepté, PAS encore remis. Cf. piège 1.
                accepted.push(message);
            }
            Err(error) => {
                let message_id = message.id.clone();
                let non_pilotable = error.to_string().contains("tour non pilotable");
                let (lock, wake) = &*worker.queue;
                let mut state = lock.lock().recover_poison();
                // Le message reste ici jusqu’au repli atomique : déplacer un
                // refus non pilotable dans `messages` ferait doubler les
                // messages acceptés antérieurs au prochain tour.
                state.steer.push_front(message);
                if non_pilotable {
                    state.steering_open = false;
                }
                wake.notify_one();
                drop(state);
                // Pas de DeliveryRejected ici : le message reste récupérable
                // et en tête de son ordre d’arrivée.
                record_or_terminal(
                    &worker.journal,
                    &worker.observations,
                    "turn_steer_refuse",
                    Some(&message_id),
                    json!({ "reason": error.to_string(), "turn_id": turn_id }),
                );
                return !non_pilotable;
            }
        }
    }
}

fn clear_active(queue: &Arc<(Mutex<QueueState>, Condvar)>) {
    let (lock, wake) = &**queue;
    let mut state = lock.lock().recover_poison();
    state.steering_open = false;
    // `turn/start` peut échouer avant `wait_for_turn`. Dans ce chemin, aucun
    // repli précédent n’a vidé `steer` : le restituer évite une perte humaine.
    let steered: Vec<_> = state.steer.drain(..).collect();
    for mut message in steered.into_iter().rev() {
        if message.intent == Some(MessageIntent::SteerCurrent)
            && message
                .references
                .iter()
                .any(|reference| reference == "steer-or-queue")
        {
            message.intent = None;
        }
        state.messages.push_front(message);
    }
    state.active = None;
    wake.notify_one();
}

fn record(
    journal: &Journal,
    event: &str,
    message_id: Option<&str>,
    payload: Value,
) -> Result<(), String> {
    let payload = codex_journal_payload(event, payload);
    journal
        .lock()
        .recover_poison()
        .as_ref()
        .map_or(Ok(()), |journal| {
            journal.enqueue(event, message_id.filter(|id| !id.is_empty()), payload)
        })
}

/// Miroir ACP `record_or_terminal` : échec d'enqueue (dont kind hors vocabulaire)
/// → `JournalFailed`, jamais avalé par `let _ = record(...)`.
fn record_or_terminal(
    journal: &Journal,
    observations: &Arc<(Mutex<Observations>, Condvar)>,
    event: &str,
    message_id: Option<&str>,
    payload: Value,
) {
    if let Err(detail) = record(journal, event, message_id, payload) {
        push_internal(observations, ManagedEventKind::JournalFailed { detail });
    }
}

fn source_matches_active_turn(detail: &CodexTurnDetail, value: &Value) -> bool {
    value.pointer("/params/threadId").and_then(Value::as_str) == Some(detail.thread_id.as_str())
        && detail.turn_id.as_deref().is_none_or(|turn_id| {
            value.pointer("/params/turnId").and_then(Value::as_str) == Some(turn_id)
        })
}

fn set_active_turn_id(active_detail: &ActiveTurnDetail, message_id: &str, turn_id: &str) {
    let mut active = active_detail.lock().recover_poison();
    if let Some(detail) = active
        .as_mut()
        .filter(|detail| detail.message_id == message_id)
    {
        detail.turn_id = Some(turn_id.to_string());
    }
}

fn observe_provider_item_id(active_detail: &ActiveTurnDetail, value: &Value) {
    let Some(provider_item_id) = value
        .pointer("/params/item/id")
        .and_then(Value::as_str)
        .filter(|item_id| !item_id.is_empty())
    else {
        return;
    };
    let mut active = active_detail.lock().recover_poison();
    if let Some(detail) = active
        .as_mut()
        .filter(|detail| source_matches_active_turn(detail, value))
    {
        detail.provider_item_id = Some(provider_item_id.to_string());
    }
}

fn observe_steered_user_message_visibility(
    observations: &Arc<(Mutex<Observations>, Condvar)>,
    active_detail: &ActiveTurnDetail,
    value: &Value,
) {
    let Some(message_id) = value
        .pointer("/params/item/clientId")
        .and_then(Value::as_str)
        .filter(|message_id| !message_id.is_empty())
    else {
        return;
    };
    let active = active_detail.lock().recover_poison();
    let Some(detail) = active.as_ref() else {
        return;
    };
    let Some(turn_id) = detail.turn_id.as_deref() else {
        return;
    };
    if value.pointer("/params/threadId").and_then(Value::as_str) != Some(detail.thread_id.as_str())
        || value.pointer("/params/turnId").and_then(Value::as_str) != Some(turn_id)
    {
        return;
    }
    let key = (detail.thread_id.clone(), turn_id.to_string());
    drop(active);
    observations
        .0
        .lock()
        .recover_poison()
        .consumed_steers_by_turn
        .entry(key)
        .or_default()
        .insert(message_id.to_string());
    observations.1.notify_all();
}

/// O(1), champs bornés : seul un item structurel du tour actif atteste le compactage.
fn compaction_payload(value: &Value, thread: &str, turn: &str) -> Option<Value> {
    let status = match value.get("method")?.as_str()? {
        "item/started" => "running",
        "item/completed" => "completed",
        _ => return None,
    };
    if value.pointer("/params/threadId")?.as_str()? != thread
        || value.pointer("/params/turnId")?.as_str()? != turn
        || value.pointer("/params/item/type")?.as_str()? != "contextCompaction"
    {
        return None;
    }
    let item = value.pointer("/params/item/id")?.as_str()?;
    if item.is_empty() || item.len() > 256 || item.chars().any(char::is_control) {
        return None;
    }
    Some(
        json!({"kind":"tool","tool_type":"context_compaction","tool_id":format!("compaction:{thread}:{turn}:{item}"),"text":"Compactage du contexte","status":status,"turn_id":turn,"item_id":item}),
    )
}

fn record_context_compaction(
    journal: &Journal,
    observations: &Arc<(Mutex<Observations>, Condvar)>,
    active_detail: &ActiveTurnDetail,
    value: &Value,
) {
    let observed = {
        let active = active_detail.lock().recover_poison();
        active.as_ref().and_then(|a| {
            compaction_payload(value, &a.thread_id, a.turn_id.as_deref()?)
                .map(|payload| (a.message_id.clone(), payload))
        })
    };
    if let Some((message, payload)) = observed {
        record_or_terminal(journal, observations, "update", Some(&message), payload);
    }
}

fn record_active_act(
    journal: &Journal,
    observations: &Arc<(Mutex<Observations>, Condvar)>,
    active_detail: &ActiveTurnDetail,
    value: &Value,
    kind: CodexActKind,
    text: &str,
    detail: Option<&str>,
) {
    let message_id = active_detail
        .lock()
        .recover_poison()
        .as_ref()
        .filter(|active| source_matches_active_turn(active, value))
        .map(|active| active.message_id.clone());
    if let Some(message_id) = message_id {
        let mut payload = serde_json::Map::from_iter([
            ("kind".to_string(), Value::String(kind.as_str().to_string())),
            ("text".to_string(), Value::String(text.to_string())),
        ]);
        if let Some(detail) = detail {
            payload.insert("detail".to_string(), Value::String(detail.to_string()));
        }
        record_or_terminal(
            journal,
            observations,
            "update",
            Some(&message_id),
            Value::Object(payload),
        );
    }
}

/// SPEC-088 : retient, par item de commande, la première ligne de sortie qui
/// correspond à un motif de sandbox. Aucun acte ici : une ligne seule n'est
/// pas une preuve (contre-revue Jim, 2026-09-03).
fn remember_sandbox_lines(delta: &str, item_id: &str, lines_by_item: &mut HashMap<String, String>) {
    if lines_by_item.contains_key(item_id) {
        return;
    }
    for line in delta.lines() {
        if let Some(refusal) = crate::refusals::recognize_sandbox_refusal(line, 0) {
            lines_by_item.insert(item_id.to_string(), refusal.raw);
            return;
        }
    }
}

/// Fin d'une `commandExecution` : acte `command` mis à jour (état, code de
/// sortie, queue de sortie bornée, `item_id`), puis, si la commande a ÉCHOUÉ
/// et qu'une ligne de sandbox a été vue sur ce même item, UN acte `refusal`
/// « signalement non attesté ». Une commande réussie n'en produit jamais.
fn record_command_completion(
    journal: &Journal,
    observations: &Arc<(Mutex<Observations>, Condvar)>,
    active_detail: &ActiveTurnDetail,
    value: &Value,
    sandbox_posture: &str,
    lines_by_item: &mut HashMap<String, String>,
    recorded: &mut HashSet<(String, String)>,
) {
    let message_id = active_detail
        .lock()
        .recover_poison()
        .as_ref()
        .filter(|active| source_matches_active_turn(active, value))
        .map(|active| active.message_id.clone());
    let Some(message_id) = message_id else {
        return;
    };
    let Some(item) = value.pointer("/params/item") else {
        return;
    };
    let item_id = item
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let command = item
        .get("command")
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("")
        .to_string();
    let status = item.get("status").and_then(Value::as_str).unwrap_or("");
    let exit_code = item
        .get("exitCode")
        .or_else(|| item.get("exit_code"))
        .and_then(Value::as_i64);
    let output = item
        .get("aggregatedOutput")
        .or_else(|| item.get("aggregated_output"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let failed =
        status == "failed" || status == "declined" || exit_code.is_some_and(|code| code != 0);
    let state = if failed { "failed" } else { "completed" };
    let output_tail: String = {
        let chars: Vec<char> = output.chars().collect();
        let start = chars.len().saturating_sub(crate::refusals::MAX_RAW_CHARS);
        chars[start..].iter().collect()
    };
    let now = unix_now_secs_i64();
    let mut payload = json!({
        "kind": crate::act_kind::JournalUpdateKind::Command.as_str(),
        "text": command,
        "detail": "item/completed",
        "state": state,
        "item_id": item_id,
        "output_tail": output_tail,
        "status": status,
    });
    if let Some(code) = exit_code {
        payload["exit_code"] = json!(code);
    }
    record_or_terminal(journal, observations, "update", Some(&message_id), payload);

    let sandbox_line = lines_by_item.remove(&item_id);
    if !failed {
        return;
    }
    let Some(raw) = sandbox_line else {
        return;
    };
    if !recorded.insert((message_id.clone(), item_id.clone())) {
        return;
    }
    let refusal = json!({
        "kind": crate::act_kind::JournalUpdateKind::Refusal.as_str(),
        "text": raw,
        "layer": crate::refusals::RefusalLayer::ProviderSandbox.as_str(),
        "evidence": "output_and_exit",
        "prevented": "shell",
        "provider": "codex",
        "posture": sandbox_posture,
        "raw": raw,
        "item_id": item_id,
        "exit_code": exit_code,
        "gesture": {"kind": "rights_line", "target": "shell"},
        "attributed_to": "bridget",
        "at": now,
    });
    record_or_terminal(journal, observations, "update", Some(&message_id), refusal);
}

fn unix_now_secs_i64() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(0)
}

/// (A) Journalise un fragment texte assistant et incrémente `text_updates`.
fn record_agent_text_delta(
    journal: &Journal,
    observations: &Arc<(Mutex<Observations>, Condvar)>,
    active_detail: &ActiveTurnDetail,
    value: &Value,
    content: &str,
) {
    let message_id = {
        let mut active = active_detail.lock().recover_poison();
        let Some(detail) = active
            .as_mut()
            .filter(|active| source_matches_active_turn(active, value))
        else {
            return;
        };
        detail.text_updates = detail.text_updates.saturating_add(1);
        detail.message_id.clone()
    };
    record_or_terminal(
        journal,
        observations,
        "update",
        Some(&message_id),
        json!({ "kind": "text", "content": content }),
    );
}

/// (B) Repli sans delta : `item/completed` agentMessage → un seul update
/// si aucun delta n'a déjà été journalisé (anti-doublon).
fn maybe_record_final_agent_message(
    journal: &Journal,
    observations: &Arc<(Mutex<Observations>, Condvar)>,
    active_detail: &ActiveTurnDetail,
    value: &Value,
    content: &str,
) {
    if content.is_empty() {
        return;
    }
    let message_id = {
        let mut active = active_detail.lock().recover_poison();
        let Some(detail) = active
            .as_mut()
            .filter(|active| source_matches_active_turn(active, value))
        else {
            return;
        };
        if detail.text_updates > 0 {
            return;
        }
        detail.text_updates = 1;
        detail.message_id.clone()
    };
    record_or_terminal(
        journal,
        observations,
        "update",
        Some(&message_id),
        json!({ "kind": "text", "content": content }),
    );
}

fn append_indexed(
    text: &mut String,
    current_index: &mut Option<u64>,
    next_index: Option<u64>,
    delta: Option<&str>,
) {
    if let Some(next_index) = next_index
        && current_index.is_some_and(|current| current != next_index)
        && !text.is_empty()
    {
        text.push('\n');
    }
    if next_index.is_some() {
        *current_index = next_index;
    }
    if let Some(delta) = delta {
        text.push_str(delta);
    }
}

fn observe_reasoning(active_detail: &ActiveTurnDetail, method: &str, value: &Value) {
    let mut active = active_detail.lock().recover_poison();
    let Some(detail) = active.as_mut() else {
        return;
    };
    if !source_matches_active_turn(detail, value) {
        return;
    }
    detail.reasoning_seen = true;
    match method {
        "item/reasoning/summaryPartAdded" => append_indexed(
            &mut detail.summary,
            &mut detail.summary_index,
            value
                .pointer("/params/summaryIndex")
                .and_then(Value::as_u64),
            None,
        ),
        "item/reasoning/summaryTextDelta" => append_indexed(
            &mut detail.summary,
            &mut detail.summary_index,
            value
                .pointer("/params/summaryIndex")
                .and_then(Value::as_u64),
            value.pointer("/params/delta").and_then(Value::as_str),
        ),
        "item/reasoning/textDelta" => append_indexed(
            &mut detail.raw_reasoning,
            &mut detail.content_index,
            value
                .pointer("/params/contentIndex")
                .and_then(Value::as_u64),
            value.pointer("/params/delta").and_then(Value::as_str),
        ),
        _ => {}
    }
}

fn finish_reasoning(active_detail: &ActiveTurnDetail, message_id: &str) -> Value {
    let detail = active_detail.lock().recover_poison().take();
    let Some(detail) = detail.filter(|detail| detail.message_id == message_id) else {
        return json!({ "available": false });
    };
    let mut payload =
        serde_json::Map::from_iter([("available".to_string(), Value::Bool(detail.reasoning_seen))]);
    if detail.reasoning_seen && !detail.summary.is_empty() {
        payload.insert("summary".to_string(), Value::String(detail.summary));
    }
    if detail.reasoning_seen && !detail.raw_reasoning.is_empty() {
        payload.insert("raw".to_string(), Value::String(detail.raw_reasoning));
    }
    Value::Object(payload)
}

fn is_approval_request(method: &str) -> bool {
    let mut parts = method.split('/');
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some("item"), Some(kind), Some("requestApproval"), None) if !kind.is_empty()
    )
}

fn approval_text<'a>(value: &'a Value, method: &'a str) -> &'a str {
    ["command", "reason", "grantRoot", "cwd"]
        .into_iter()
        .find_map(|key| {
            value
                .pointer(&format!("/params/{key}"))
                .and_then(Value::as_str)
                .filter(|text| !text.is_empty())
        })
        .unwrap_or(method)
}

/// Frontière T002 : O(d+q), sans accès au disque ni interprétation de commande.
/// Une information absente ne devient jamais une absence d'effet prouvée.
fn normalize_child_provider_request(
    value: &Value,
    session: &str,
) -> Result<(ChildProviderRequest, Value), &'static str> {
    let id: ChildProviderRequestId =
        serde_json::from_value(value.get("id").cloned().ok_or("request_unlinked")?)
            .map_err(|_| "request_unlinked")?;
    let method = value
        .get("method")
        .and_then(Value::as_str)
        .ok_or("approval_method_unsupported")?;
    let params = value
        .get("params")
        .and_then(Value::as_object)
        .ok_or("details_incomplete")?;
    for field in ["threadId", "turnId", "itemId"] {
        if !params
            .get(field)
            .and_then(Value::as_str)
            .is_some_and(|value| !value.is_empty() && value.len() <= 1024)
        {
            return Err("details_incomplete");
        }
    }
    let mut request = ChildProviderRequest {
        version: 1,
        provider_session: session.into(),
        turn_id: params["turnId"].as_str().unwrap().into(),
        provider_request_id: id,
        kind: ChildRequestKind::UserInput,
        details: Value::Null,
        offered: Vec::new(),
        question_ids: Vec::new(),
        truncated: false,
    };
    let mut approval = json!({"id":value["id"],"method":method,"params":{}});
    match method {
        COMMAND_EXECUTION_APPROVAL_METHOD => {
            if params.keys().any(|key| {
                !matches!(
                    key.as_str(),
                    "threadId"
                        | "turnId"
                        | "itemId"
                        | "startedAtMs"
                        | "approvalId"
                        | "availableDecisions"
                        | "command"
                        | "commandActions"
                        | "cwd"
                        | "environmentId"
                        | "additionalPermissions"
                        | "networkApprovalContext"
                        | "proposedExecpolicyAmendment"
                        | "proposedNetworkPolicyAmendments"
                        | "reason"
                        | "kind"
                )
            }) || params
                .get("startedAtMs")
                .is_none_or(|value| value.as_i64().is_none())
                || ["approvalId", "cwd", "reason"].iter().any(|key| {
                    params
                        .get(*key)
                        .is_some_and(|value| !value.is_null() && !value.is_string())
                })
                || params
                    .get("commandActions")
                    .is_some_and(|value| !value.is_null() && !value.is_array())
            {
                return Err("details_incomplete");
            }
            // Forward155 passe seulement par le processus STDIO privé attesté,
            // jamais par un endpoint choisi dans ce champ fournisseur. "local"
            // décrit ce contexte ; il n'accorde aucun droit ni confinement.
            if params.get("kind").is_some_and(|value| value != "command")
                || params
                    .get("environmentId")
                    .is_some_and(|value| !value.is_null() && value != "local")
            {
                return Err("approval_method_unsupported");
            }
            if [
                "proposedExecpolicyAmendment",
                "proposedNetworkPolicyAmendments",
            ]
            .iter()
            .any(|key| params.get(*key).is_some_and(|value| !value.is_null()))
            {
                return Err("duration_unsupported");
            }
            let command = params
                .get("command")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .ok_or("details_incomplete")?;
            // commandActions est best-effort : même read/list/search ne prouve
            // jamais qu'une commande n'a ni effets shell ni accès fichiers.
            let choices = params
                .get("availableDecisions")
                .and_then(Value::as_array)
                .ok_or("details_incomplete")?;
            for (name, decision) in [
                ("accept", ChildRequestDecision::Accept),
                ("decline", ChildRequestDecision::Decline),
                ("cancel", ChildRequestDecision::Cancel),
            ] {
                if choices.iter().any(|value| value == name) {
                    request.offered.push(decision);
                }
            }
            if !request.offered.contains(&ChildRequestDecision::Accept)
                || !request.offered.iter().any(|decision| {
                    matches!(
                        decision,
                        ChildRequestDecision::Decline | ChildRequestDecision::Cancel
                    )
                })
            {
                return Err("duration_unsupported");
            }
            let mut network = false;
            let mut file_write = false;
            let mut paths = Vec::new();
            if let Some(context) = params
                .get("networkApprovalContext")
                .filter(|value| !value.is_null())
            {
                let context = context.as_object().ok_or("details_incomplete")?;
                if context.len() != 2
                    || !context.get("host").is_some_and(Value::is_string)
                    || !context
                        .get("protocol")
                        .and_then(Value::as_str)
                        .is_some_and(|value| {
                            matches!(value, "http" | "https" | "socks5Tcp" | "socks5Udp")
                        })
                {
                    return Err("details_incomplete");
                }
                network = true;
            }
            if let Some(extra) = params
                .get("additionalPermissions")
                .filter(|value| !value.is_null())
            {
                let extra = extra.as_object().ok_or("details_incomplete")?;
                if extra
                    .keys()
                    .any(|key| !matches!(key.as_str(), "network" | "fileSystem"))
                {
                    return Err("details_incomplete");
                }
                if let Some(net) = extra.get("network").filter(|value| !value.is_null()) {
                    let net = net.as_object().ok_or("details_incomplete")?;
                    if net.keys().any(|key| key != "enabled")
                        || net
                            .get("enabled")
                            .is_some_and(|value| !value.is_null() && !value.is_boolean())
                    {
                        return Err("details_incomplete");
                    }
                    network |= net.get("enabled") == Some(&Value::Bool(true));
                }
                if let Some(files) = extra.get("fileSystem").filter(|value| !value.is_null()) {
                    let files = files.as_object().ok_or("details_incomplete")?;
                    if files.keys().any(|key| {
                        !matches!(
                            key.as_str(),
                            "read" | "write" | "entries" | "globScanMaxDepth"
                        )
                    }) || files.get("entries").is_some_and(|value| {
                        !value.is_null() && !value.as_array().is_some_and(Vec::is_empty)
                    }) || files
                        .get("globScanMaxDepth")
                        .is_some_and(|value| !value.is_null())
                    {
                        return Err("details_incomplete");
                    }
                    // Listes simples seulement ; les portées glob/special ne
                    // sont pas aplaties en racines locales prétendues sûres.
                    for field in ["read", "write"] {
                        if let Some(list) = files.get(field).filter(|value| !value.is_null()) {
                            let list = list.as_array().ok_or("details_incomplete")?;
                            if list.len() > 64
                                || list
                                    .iter()
                                    .any(|path| path.as_str().is_none_or(|path| path.len() > 1024))
                            {
                                return Err("details_incomplete");
                            }
                            file_write |= field == "write" && !list.is_empty();
                            paths.extend(list.iter().cloned());
                        }
                    }
                }
            }
            request.kind = ChildRequestKind::ApprovalCommand;
            // Dossier cité par le fournisseur, jamais résolu ni remplacé par
            // celui de la session. L'absence reste explicitement null.
            let cwd = params.get("cwd").unwrap_or(&Value::Null);
            if cwd.as_str().is_some_and(|cwd| cwd.len() > 1024) {
                return Err("details_incomplete");
            }
            request.truncated = command.len() > 1024;
            let cited_command: String = command
                .chars()
                .scan(0usize, |bytes, ch| {
                    *bytes += ch.len_utf8();
                    (*bytes <= 1024).then_some(ch)
                })
                .collect();
            request.details = json!({"method":method,"kind":"command","duration":"once","effects":{"command":true,"network":network,"file_write":file_write},
                "complete":true,"secret":false,"command":cited_command,"cwd":cwd,"paths_cited":paths});
            if let Some(reason) = params.get("reason").filter(|value| !value.is_null()) {
                let reason = reason.as_str().ok_or("details_incomplete")?;
                let clean: String = reason
                    .chars()
                    .filter(|ch| {
                        !ch.is_control()
                            && !matches!(*ch,'\u{202a}'..='\u{202e}'|'\u{2066}'..='\u{2069}')
                    })
                    .scan(0usize, |bytes, ch| {
                        *bytes += ch.len_utf8();
                        (*bytes <= 1024).then_some(ch)
                    })
                    .collect();
                request.truncated |= reason.len() > 1024;
                request.details["reason"] = json!(clean);
            }
            approval["params"]["availableDecisions"] =
                serde_json::to_value(&request.offered).map_err(|_| "details_incomplete")?;
        }
        "item/tool/requestUserInput" => {
            if params.keys().any(|key| {
                !matches!(
                    key.as_str(),
                    "threadId"
                        | "turnId"
                        | "itemId"
                        | "isBlocking"
                        | "autoResolutionMs"
                        | "questions"
                )
            }) || !params.get("isBlocking").is_some_and(Value::is_boolean)
                || params
                    .get("autoResolutionMs")
                    .is_some_and(|value| !value.is_null() && value.as_u64().is_none())
            {
                return Err("details_incomplete");
            }
            let questions = params
                .get("questions")
                .and_then(Value::as_array)
                .filter(|questions| !questions.is_empty())
                .ok_or("details_incomplete")?;
            if serde_json::to_vec(questions).map_or(true, |bytes| bytes.len() > 4096) {
                return Err("details_incomplete");
            }
            let mut seen = HashSet::new();
            let mut normalized = Vec::new();
            let mut secret = false;
            for question in questions {
                let question = question.as_object().ok_or("details_incomplete")?;
                if question.keys().any(|key| {
                    !matches!(
                        key.as_str(),
                        "id" | "header" | "question" | "isSecret" | "isOther" | "options"
                    )
                }) || ["id", "header", "question"]
                    .iter()
                    .any(|key| !question.get(*key).is_some_and(Value::is_string))
                    || ["isSecret", "isOther"]
                        .iter()
                        .any(|key| question.get(*key).is_some_and(|value| !value.is_boolean()))
                {
                    return Err("details_incomplete");
                }
                let id = question["id"].as_str().unwrap();
                if id.is_empty() || !seen.insert(id) {
                    return Err("details_incomplete");
                }
                secret |= question.get("isSecret") == Some(&Value::Bool(true));
                if let Some(options) = question.get("options").filter(|value| !value.is_null()) {
                    let options = options.as_array().ok_or("details_incomplete")?;
                    if options.iter().any(|option| {
                        !option.as_object().is_some_and(|option| {
                            option.len() == 2
                                && ["label", "description"]
                                    .iter()
                                    .all(|key| option.get(*key).is_some_and(Value::is_string))
                        })
                    }) {
                        return Err("details_incomplete");
                    }
                }
                request.question_ids.push(id.to_string());
                normalized.push(Value::Object(question.clone()));
            }
            if secret {
                return Err("secret_not_forwardable");
            }
            request.offered = vec![
                ChildRequestDecision::Accept,
                ChildRequestDecision::Decline,
                ChildRequestDecision::Cancel,
            ];
            request.details = json!({"method":method,"duration":"once","effects":{"command":false,"network":false,"file_write":false},"complete":true,"secret":false,"questions":normalized});
        }
        "item/fileChange/requestApproval" => {
            return Err(
                if params
                    .get("grantRoot")
                    .is_some_and(|value| !value.is_null())
                {
                    "duration_unsupported"
                } else {
                    "approval_method_unsupported"
                },
            );
        }
        _ => return Err("approval_method_unsupported"),
    }
    if serde_json::to_vec(&request.details).map_or(true, |bytes| bytes.len() > 4096) {
        return Err("details_incomplete");
    }
    Ok((request, approval))
}

/// O(n), n décisions offertes : accept exige allow ; sinon decline, puis cancel.
/// Aucun accord durable ni amendement. Sans décision fermée proposée, erreur
/// JSON-RPC ; l'identifiant de la demande est repris tel quel.
fn approval_response(value: &Value, permissions: &str) -> Option<(Value, Value)> {
    let id = value.get("id")?;
    let method = value.get("method").and_then(Value::as_str).unwrap_or("");
    let decision = value
        .pointer("/params/availableDecisions")
        .and_then(Value::as_array)
        .and_then(|choices| {
            ["accept", "decline", "cancel"]
                .into_iter()
                .find(|decision| {
                    (*decision != "accept" || permissions == "allow")
                        && choices
                            .iter()
                            .filter_map(Value::as_str)
                            .any(|choice| choice == *decision)
                })
        });
    let Some(decision) = decision else {
        return Some((
            json!({ "id": id, "error": { "code": -32000, "message": "child_request_refused" } }),
            json!({
                "method": method,
                "permissions": permissions,
                "status": "refused",
                "code": "child_request_refused",
            }),
        ));
    };
    Some((
        json!({ "id": id, "result": { "decision": decision } }),
        json!({
            "method": method,
            "decision": decision,
            "permissions": permissions,
        }),
    ))
}

/// Répond à une demande d'outil dynamique de l'app-server. Cette surface est
/// distincte de MCP, mais les deux chemins passent par le même exécuteur
/// Bridget attesté. Un nom inconnu reste refusé explicitement au moteur au
/// lieu de laisser le tour finir sur un faux succès.
fn dynamic_tool_response(
    value: &Value,
    handler: Option<&DynamicToolHandler>,
) -> Option<(Value, &'static str)> {
    let id = value.get("id")?;
    let params = value.get("params")?;
    let tool = params.get("tool").and_then(Value::as_str)?;
    let arguments = params.get("arguments").unwrap_or(&Value::Null);
    let label = match tool {
        "bridget_publish_artifact" => "publication d’artefact Bridget",
        "bridget_read_artifact" => "lecture d’artefact Bridget",
        _ => "outil dynamique Bridget refusé",
    };
    let outcome = match handler {
        Some(handler) if matches!(tool, "bridget_publish_artifact" | "bridget_read_artifact") => {
            handler(tool, arguments)
                .map(|payload| {
                    serde_json::to_string(&payload).unwrap_or_else(|_| {
                        "{\"status\":\"refused\",\"code\":\"serialization\"}".to_string()
                    })
                })
                .map_err(|_| {
                    if tool == "bridget_read_artifact" {
                        "La lecture Bridget a été refusée.".to_string()
                    } else {
                        "La publication Bridget a été refusée.".to_string()
                    }
                })
        }
        Some(_) => Err("Cet outil dynamique Bridget n’est pas autorisé.".to_string()),
        None => Err("Aucun exécuteur Bridget n’est disponible dans cette session.".to_string()),
    };
    let (success, text) = match outcome {
        Ok(text) => (true, text),
        Err(text) => (false, text),
    };
    Some((
        json!({
            "id": id,
            "result": {
                "success": success,
                "contentItems": [{ "type": "inputText", "text": text }],
            }
        }),
        label,
    ))
}

/// Refuse une requête fournisseur inconnue sans reproduire son corps dans les
/// diagnostics. Le fournisseur reçoit une erreur JSON-RPC exploitable et
/// l'interface un code stable avec une référence pseudonymisée.
fn unsupported_provider_request_response(value: &Value) -> Option<(Value, Value, String)> {
    let id = value.get("id")?;
    let method = value.get("method").and_then(Value::as_str)?;
    let reference = project_provider_method(method);
    let delegated_reference = provider_fingerprint(b"delegated-runtime", method.as_bytes());
    Some((
        json!({
            "id": id,
            "error": {
                "code": -32601,
                "message": "unsupported provider request",
                "data": {
                    "code": "unsupported_provider_request",
                    "reference": reference,
                }
            }
        }),
        json!({
            "provider": "codex",
            "code": "unsupported_provider_request",
            "reference": project_provider_method(method),
        }),
        delegated_reference,
    ))
}

/// Répond à une élicitation MCP sans la transformer en permission générale.
/// Bridget ne l'accepte automatiquement que lorsqu'elle vient de son propre
/// serveur et ne demande aucun champ. Les formulaires avec données, les URL et
/// les serveurs tiers restent explicitement déclinés : ils nécessitent une
/// décision humaine dans une surface dédiée.
fn is_mcp_elicitation_request(method: &str) -> bool {
    // Le relais app-server a déjà reçu des variantes avec terminaison blanche.
    // JSON-RPC interdit cette variante, mais la normaliser ici évite de perdre
    // une publication Bridget alors que la requête reste sans ambiguïté.
    method.trim() == "mcpServer/elicitation/request"
}

fn mcp_elicitation_response(value: &Value) -> Option<(Value, Value)> {
    let id = value.get("id")?;
    let params = value.get("params").unwrap_or(&Value::Null);
    // `mode: form` est implicite dans MCP. Certains app-servers placent les
    // paramètres MCP sous `request.params`, d'autres les exposent directement.
    let request_params = params.pointer("/request/params").unwrap_or(params);
    let server_name = params
        .get("serverName")
        .and_then(Value::as_str)
        .unwrap_or("");
    let mode = request_params
        .get("mode")
        .and_then(Value::as_str)
        .unwrap_or("form");
    let required_fields = request_params
        .pointer("/requestedSchema/required")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let auto_accepted = server_name == "bridget" && mode == "form" && required_fields == 0;
    let action = if auto_accepted { "accept" } else { "decline" };
    let mut result = json!({ "action": action });
    if auto_accepted {
        result["content"] = json!({});
    }
    Some((
        json!({ "id": id, "result": result }),
        json!({
            "provider": "codex",
            "code": if auto_accepted {
                "mcp_elicitation_autoaccepted"
            } else {
                "mcp_elicitation_declined"
            },
            "server": project_provider_method(server_name),
            "mode": mode,
            "required_field_count": required_fields,
        }),
    ))
}

fn codex_journal_payload(event: &str, payload: Value) -> Value {
    // Dans ce pilote, `error` est exclusivement l'issue terminale du worker.
    // La passerelle porte le code ici afin que les enrichissements de payload
    // (par exemple une requête fournisseur pendante) ne puissent pas l'omettre.
    if event == "error" {
        with_turn_failed_kind(payload)
    } else {
        payload
    }
}

fn served_model_from_codex(value: &Value) -> Option<String> {
    let params = value.get("params")?;
    ["model", "to", "actual", "served"]
        .into_iter()
        .find_map(|key| params.get(key).and_then(Value::as_str))
        .map(str::trim)
        .filter(|model| !model.is_empty())
        .map(str::to_string)
}

fn maybe_record_mismatch(
    journal: &Journal,
    observations: &Arc<(Mutex<Observations>, Condvar)>,
    pinned: Option<&str>,
    served: &str,
) {
    let Some(pinned) = pinned.filter(|pinned| *pinned != served) else {
        return;
    };
    record_or_terminal(
        journal,
        observations,
        "model_mismatch",
        None,
        json!({ "pinned": pinned, "served": served }),
    );
}

fn push_internal(observations: &Arc<(Mutex<Observations>, Condvar)>, kind: ManagedEventKind) {
    let raw = serde_json::to_vec(&json!({ "kind": "codex_internal" }))
        .expect("fait interne Codex sérialisable");
    observations
        .0
        .lock()
        .recover_poison()
        .events
        .push_back(ManagedEvent::internal(
            ManagedEventSource::CodexAppServer,
            raw,
            kind,
        ));
    observations.1.notify_all();
}

fn saturation_delay(attempt: u32, message_id: &str) -> Duration {
    let base = 20_u64.saturating_mul(1_u64 << attempt.min(6));
    let jitter = message_id
        .bytes()
        .fold(0_u64, |sum, byte| sum + u64::from(byte))
        % 17;
    Duration::from_millis(base + jitter)
}

fn is_saturated(reason: &str) -> bool {
    reason == CODEX_SATURATED_REASON
}

fn list_interactive_threads(
    writer: &Writer,
    waiters: &Waiters,
    next_id: &Arc<AtomicU64>,
) -> Result<Vec<CodexThreadSummary>, TransportError> {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    collect_thread_pages(deadline, |params, remaining| {
        request_with_timeout(writer, waiters, next_id, "thread/list", params, remaining)
            .map(|response| response.value)
    })
}

fn collect_thread_pages(
    deadline: std::time::Instant,
    mut fetch: impl FnMut(Value, Duration) -> Result<Value, TransportError>,
) -> Result<Vec<CodexThreadSummary>, TransportError> {
    let mut cursor = None::<String>;
    let mut cursors = HashSet::new();
    let mut ids = HashSet::new();
    let mut threads = Vec::new();
    for _ in 0..10 {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        let response = fetch(
            json!({
                "cursor": cursor, "limit": 100, "sortKey": "updated_at", "modelProviders": [],
                "sourceKinds": ["cli", "vscode", "appServer"],
                // Évite le scan/réparation des rollouts sur le chemin interactif borné.
                "useStateDbOnly": true
            }),
            remaining,
        )?;
        let data = response
            .get("data")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                TransportError::DeliveryFailed("thread/list : catalogue absent".into())
            })?;
        if data.len() > 100 {
            break;
        }
        for value in data {
            let entry: CodexThreadSummary =
                serde_json::from_value(value.clone()).map_err(|error| {
                    TransportError::DeliveryFailed(format!("thread/list invalide : {error}"))
                })?;
            if uuid::Uuid::parse_str(&entry.id).is_err() || !ids.insert(entry.id.clone()) {
                return Err(TransportError::DeliveryFailed(
                    "thread/list : identifiant invalide ou répété".into(),
                ));
            }
            threads.push(entry);
        }
        match response.get("nextCursor") {
            Some(Value::Null) => return Ok(threads),
            Some(Value::String(next)) if !next.is_empty() && cursors.insert(next.clone()) => {
                cursor = Some(next.clone())
            }
            _ => break,
        }
    }
    Err(TransportError::DeliveryFailed(
        "catalogue Codex incomplet (borne 1 000 fils / 10 s) ; reprendre par UUID explicite".into(),
    ))
}

fn bootstrap_operation(bootstrap: &CodexThreadBootstrap) -> Option<ProviderOperation> {
    match bootstrap {
        CodexThreadBootstrap::Start => None,
        CodexThreadBootstrap::Resume { .. } => Some(ProviderOperation::Resume),
        CodexThreadBootstrap::Fork { .. } => Some(ProviderOperation::Fork),
    }
}

fn validate_thread_bootstrap(
    observation: &Option<ProviderObservation>,
    bootstrap: &CodexThreadBootstrap,
) -> Result<(), TransportError> {
    let Some(operation) = bootstrap_operation(bootstrap) else {
        return Ok(());
    };
    if observation
        .as_ref()
        .is_some_and(|value| value.supports(operation))
    {
        return Ok(());
    }
    let requested = match operation {
        ProviderOperation::Resume => "thread/resume",
        ProviderOperation::Fork => "thread/fork",
        _ => unreachable!("bootstrap limité à resume/fork"),
    };
    Err(TransportError::DeliveryFailed(format!(
        "{requested} Codex refusé: capacité non attestée pour cette version"
    )))
}

fn validate_endpoint_bootstrap(
    options: &CodexAppServerOptions,
    interactive: bool,
) -> Result<(), TransportError> {
    // La reprise HUMAINE explicite est négociée avec le serveur privé avant
    // toute présence : RPC réussie, UUID identique et historique reconnu exigés.
    // Ce n'est pas une reprise automatique gérée : son attestation figée reste
    // obligatoire, et aucune observation fournisseur n'est fabriquée ici.
    if interactive
        && matches!(
            options.thread_bootstrap,
            CodexThreadBootstrap::Resume { .. }
        )
    {
        return Ok(());
    }
    validate_thread_bootstrap(&options.provider_observation, &options.thread_bootstrap)
}

fn thread_bootstrap_request(
    bootstrap: &CodexThreadBootstrap,
    cwd: &Path,
    model: Option<&str>,
    interactive: bool,
) -> (&'static str, Value) {
    match bootstrap {
        CodexThreadBootstrap::Start => {
            let mut params = json!({ "cwd": cwd });
            if let Some(model) = model {
                params["model"] = Value::String(model.to_string());
            }
            ("thread/start", params)
        }
        CodexThreadBootstrap::Resume { thread_id } => {
            let mut params = json!({ "threadId": thread_id });
            if interactive {
                // Le contrôleur ne consomme aucun ancien tour. Codex garde
                // l'historique et la TUI le lit par son propre client natif.
                // Une reprise réelle peut sinon répondre avec > 79 Mo.
                params["excludeTurns"] = json!(true);
            }
            if let Some(model) = model {
                params["model"] = json!(model);
            }
            ("thread/resume", params)
        }
        CodexThreadBootstrap::Fork { thread_id } => {
            ("thread/fork", json!({ "threadId": thread_id }))
        }
    }
}

fn request(
    writer: &Writer,
    waiters: &Waiters,
    next_id: &Arc<AtomicU64>,
    method: &str,
    params: Value,
) -> Result<ServerResponse, TransportError> {
    request_with_timeout(writer, waiters, next_id, method, params, REQUEST_TIMEOUT)
}

fn request_with_timeout(
    writer: &Writer,
    waiters: &Waiters,
    next_id: &Arc<AtomicU64>,
    method: &str,
    params: Value,
    timeout: Duration,
) -> Result<ServerResponse, TransportError> {
    let id = next_id.fetch_add(1, Ordering::SeqCst);
    let (sender, receiver) = mpsc::channel();
    waiters.lock().recover_poison().insert(id, sender);
    if let Err(error) = write_value(
        writer,
        json!({ "id": id, "method": method, "params": params }),
    ) {
        waiters.lock().recover_poison().remove(&id);
        return Err(error);
    }
    let result = receiver.recv_timeout(timeout);
    // Un fournisseur muet ne doit pas accumuler un waiter à chaque contrôle.
    // La réponse tardive reste ignorée, jamais réattribuée au contrôle suivant.
    if result.is_err() {
        waiters.lock().recover_poison().remove(&id);
    }
    result
        .map_err(|_| TransportError::DeliveryFailed(format!("échéance Codex sur {method}")))?
        .map_err(TransportError::DeliveryFailed)
}

fn runtime_from_thread_start(value: &Value) -> Option<(String, Option<String>)> {
    let model = value.get("model")?.as_str()?.to_string();
    let effort = value
        .get("reasoningEffort")
        .and_then(Value::as_str)
        .map(str::to_string);
    Some((model, effort))
}

/// Rend un fait public par fenêtre attestée (primary et/ou secondary).
/// Une lecture réussie où `rateLimitReachedType` vaut `null` signifie
/// explicitement qu'aucun dépassement n'est attesté ; l'étiquette `available`
/// est une projection de ce null, sans effet de routage ou de lancement.
/// Une fenêtre absente du snapshot reste absente — jamais inventée.
///
/// Projection partagée du statut : `rateLimitReachedType` est un champ du
/// *snapshot* (pas par fenêtre). La même valeur est donc recopiée sur primary
/// et secondary — ce n'est pas un statut indépendant par fenêtre, c'est la
/// projection honnête du schéma Codex.
fn rate_limits_from_snapshot(
    snapshot: Option<&Value>,
) -> Vec<(
    String,
    String,
    Option<i64>,
    Option<u8>,
    Option<crate::protocol::CodexCredits>,
)> {
    let Some(snapshot) = snapshot else {
        return Vec::new();
    };
    let status = snapshot
        .get("rateLimitReachedType")
        .and_then(Value::as_str)
        .unwrap_or("available")
        .to_string();
    let credits = codex_credits(snapshot);
    let mut facts = Vec::new();
    for key in ["primary", "secondary"] {
        if let Some(fact) = window_fact_from_codex(snapshot.get(key), key, &status) {
            facts.push((fact.0, fact.1, fact.2, fact.3, credits.clone()));
        }
    }
    if snapshot.get("credits").is_some() || snapshot.get("spendControlReached").is_some() {
        facts.push((
            "account/credits".into(),
            "observed".into(),
            None,
            None,
            credits,
        ));
    }
    facts
}

/// Les mises à jour sparse retirent aussi une ancienne autorisation payante.
/// Ce fait administratif n'invente aucune fenêtre de consommation.
fn rate_limit_update_facts(
    snapshot: Option<&Value>,
) -> Vec<(
    String,
    String,
    Option<i64>,
    Option<u8>,
    Option<crate::protocol::CodexCredits>,
)> {
    let mut facts = rate_limits_from_snapshot(snapshot);
    if let Some(snapshot) = snapshot.filter(|value| value.is_object())
        && !facts.iter().any(|fact| fact.0 == "account/credits")
    {
        facts.push((
            "account/credits".into(),
            "observed".into(),
            None,
            None,
            codex_credits(snapshot),
        ));
    }
    facts
}

fn codex_credits(snapshot: &Value) -> Option<crate::protocol::CodexCredits> {
    let spend_control_reached = snapshot.get("spendControlReached")?.as_bool()?;
    let has_credits = snapshot
        .pointer("/credits/hasCredits")
        .and_then(Value::as_bool);
    let unlimited = snapshot
        .pointer("/credits/unlimited")
        .and_then(Value::as_bool);
    if !spend_control_reached && (has_credits.is_none() || unlimited.is_none()) {
        return None;
    }
    Some(crate::protocol::CodexCredits {
        has_credits,
        unlimited,
        spend_control_reached,
    })
}

fn window_fact_from_codex(
    window: Option<&Value>,
    key: &str,
    status: &str,
) -> Option<(String, String, Option<i64>, Option<u8>)> {
    let window = window?;
    // `usedPercent` est le seul champ obligatoire de la fenêtre dans le
    // schéma app-server : sans lui, une mise à jour sparse ne prouve aucune
    // limite complète et reste donc inconnue à l'annuaire.
    // Hors 0..=100 : la fenêtre reste (mandat « % omis »), le pourcentage non.
    let used_raw = window.get("usedPercent")?.as_i64()?;
    let used_percent = ((0..=100).contains(&used_raw)).then_some(used_raw as u8);
    let minutes = window.get("windowDurationMins").and_then(Value::as_i64);
    let name = minutes
        .map(|minutes| format!("{key}/{minutes}m"))
        .unwrap_or_else(|| key.to_string());
    let resets_at = window.get("resetsAt").and_then(Value::as_i64);
    Some((name, status.to_string(), resets_at, used_percent))
}

fn write_notification(writer: &Writer, method: &str, params: Value) -> Result<(), TransportError> {
    write_value(writer, json!({ "method": method, "params": params }))
}

fn write_value(writer: &Writer, value: Value) -> Result<(), TransportError> {
    // Un budget TOTAL pour le verrou, les écritures STDIO partielles et flush.
    // Le SocketWriter interactif conserve son propre flush borné à3s ; il
    // peut ajouter ces3s après contention, sans prétendre au budget STDIO.
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut bytes =
        serde_json::to_vec(&value).map_err(|error| TransportError::Io(error.to_string()))?;
    bytes.push(b'\n'); // mêmes octets JSON compacts + LF que writeln!("{value}").
    let mut guard = loop {
        if Instant::now() >= deadline {
            return Err(TransportError::Io("budget writer Codex dépassé".into()));
        }
        match writer.try_lock() {
            Ok(guard) => break guard,
            Err(std::sync::TryLockError::Poisoned(error)) => {
                return Err(TransportError::Io(error.to_string()));
            }
            Err(std::sync::TryLockError::WouldBlock) => {
                let millis = deadline
                    .saturating_duration_since(Instant::now())
                    .as_millis()
                    .clamp(1, 25) as i32;
                // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
                if unsafe { libc::poll(std::ptr::null_mut(), 0, millis) } < 0 {
                    let error = std::io::Error::last_os_error();
                    if error.kind() != std::io::ErrorKind::Interrupted {
                        return Err(TransportError::Io(error.to_string()));
                    }
                }
            }
        }
    };
    let result = (|| {
        let output = guard.as_mut().ok_or(TransportError::AgentDead)?;
        let mut offset = 0;
        loop {
            if Instant::now() >= deadline {
                return Err(TransportError::Io("budget writer Codex dépassé".into()));
            }
            let progress = if offset < bytes.len() {
                output.write(&bytes[offset..]).map(Some)
            } else {
                output.flush().map(|()| None)
            };
            match progress {
                Ok(None) => return Ok(()),
                Ok(Some(0)) => {
                    return Err(TransportError::Io("écriture Codex sans progrès".into()));
                }
                Ok(Some(count)) => offset += count,
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    // Box<dyn Write> ne donne aucun fd. poll sans descripteur
                    // attend au plus25ms sous la MÊME échéance, sans busy-loop.
                    let millis = deadline
                        .saturating_duration_since(Instant::now())
                        .as_millis()
                        .clamp(1, 25) as i32;
                    // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
                    if unsafe { libc::poll(std::ptr::null_mut(), 0, millis) } < 0 {
                        let error = std::io::Error::last_os_error();
                        if error.kind() != std::io::ErrorKind::Interrupted {
                            return Err(TransportError::Io(error.to_string()));
                        }
                    }
                }
                Err(error) => return Err(TransportError::Io(error.to_string())),
            }
        }
    })();
    // Une trame partielle n'est pas une réponse négative attestée. Fermer le
    // canal empêche qu'une écriture suivante complète ses octets incertains.
    if result.is_err() {
        let _ = guard.take();
    }
    result
}

/// Branche `item/completed` de la lecture Codex, extraite de `spawn_reader` (programme 260, lot 9).
/// Le corps reprend la branche d'origine sans autre changement.
fn on_item_completed(
    journal: &Journal,
    observations: &Arc<(Mutex<Observations>, Condvar)>,
    active_detail: &ActiveTurnDetail,
    value: &Value,
    raw: Vec<u8>,
    cwd: &str,
    interactive: bool,
    queue: &Arc<(Mutex<QueueState>, Condvar)>,
    sandbox_posture: &str,
    completed_file_items: &mut (String, HashSet<String>),
    sandbox_lines_by_item: &mut HashMap<String, String>,
    refusals_recorded: &mut HashSet<(String, String)>,
) {
    record_context_compaction(journal, observations, active_detail, value);
    let item = value.pointer("/params/item");
    if item.and_then(|i| i.get("type")).and_then(Value::as_str) == Some("fileChange")
        && item.and_then(|i| i.get("status")).and_then(Value::as_str) == Some("completed")
    {
        let message_id = active_detail
            .lock()
            .recover_poison()
            .as_ref()
            .filter(|active| source_matches_active_turn(active, value))
            .map(|active| active.message_id.clone());
        if let Some(message_id) = message_id
            && let Some(changes) = item
                .and_then(|i| i.get("changes"))
                .and_then(Value::as_array)
        {
            let turn = value
                .pointer("/params/turnId")
                .and_then(Value::as_str)
                .unwrap_or("");
            if completed_file_items.0 != turn {
                *completed_file_items = (turn.into(), HashSet::new());
            }
            let item_id = item
                .and_then(|i| i.get("id"))
                .and_then(Value::as_str)
                .unwrap_or("");
            let accepted = !item_id.is_empty()
                && item_id.len() <= 256
                && completed_file_items.1.len() < 4096
                && completed_file_items.1.insert(item_id.into());
            if !accepted && completed_file_items.1.len() >= 4096 {
                log::warn!("observation_gap: déduplication Codex saturée pour ce tour");
            }
            for change in changes.iter().take(if accepted { 256 } else { 0 }) {
                if let Some(path) = change.get("path").and_then(Value::as_str)
                    && let Some(payload) =
                        crate::journal::confirmed_write_payload(path, Path::new(&cwd))
                {
                    record_or_terminal(journal, observations, "update", Some(&message_id), payload);
                }
            }
            if changes.len() > 256 {
                log::warn!("observation_gap: changements Codex au-delà de 256 fichiers");
            }
        }
    }
    if interactive
        && item.and_then(|i| i.get("type")).and_then(Value::as_str) == Some("userMessage")
        && let Some(client_id) = item.and_then(|i| i.get("clientId")).and_then(Value::as_str)
    {
        let active = queue.0.lock().recover_poison();
        let mut detail = active_detail.lock().recover_poison();
        if let Some(detail) = detail.as_mut().filter(|d| {
            source_matches_active_turn(d, value)
                && d.message_id == client_id
                && !d.prompt_dispatched
        }) && active
            .active
            .as_ref()
            .is_some_and(|a| a.message_id == client_id)
        {
            detail.prompt_dispatched = true;
            push_source(
                observations,
                raw.clone(),
                ManagedEventKind::PromptDispatched {
                    message_id: client_id.into(),
                },
            );
            record_or_terminal(
                journal,
                observations,
                "prompt_dispatched",
                Some(client_id),
                json!({"provider_turn_id": value.pointer("/params/turnId")}),
            );
        }
    }
    // SPEC-088 : fin d'une commande. Journalisée avec son état
    // réel ; un signalement de sandbox n'existe qu'ici, et
    // seulement si la commande a échoué.
    if item
        .and_then(|item| item.get("type"))
        .and_then(Value::as_str)
        == Some("commandExecution")
    {
        record_command_completion(
            journal,
            observations,
            active_detail,
            value,
            sandbox_posture,
            sandbox_lines_by_item,
            refusals_recorded,
        );
    }
    let is_agent = item
        .and_then(|item| item.get("type"))
        .and_then(Value::as_str)
        == Some("agentMessage");
    if is_agent
        && let Some(text) = item
            .and_then(|item| item.get("text"))
            .and_then(Value::as_str)
    {
        if let Some(turn_id) = value.pointer("/params/turnId").and_then(Value::as_str) {
            let mut observed = observations.0.lock().recover_poison();
            let entry = observed
                .response_by_turn
                .entry(turn_id.to_string())
                .or_default();
            if entry.is_empty() {
                entry.push_str(text);
            }
        }
        maybe_record_final_agent_message(journal, observations, active_detail, value, text);
    }
    push_source(
        observations,
        raw,
        ManagedEventKind::Update {
            detail: "item/completed Codex".to_string(),
        },
    );
}

fn spawn_reader(lines: SourceLines, context: ReaderContext) -> thread::JoinHandle<()> {
    let ReaderContext {
        permission_observation,
        cwd,
        waiters,
        observations,
        alive,
        journal,
        queue,
        pending_request,
        child_requests,
        forward_child_requests,
        provider_session,
        pinned_model,
        active_detail,
        writer,
        permissions,
        dynamic_tool_handler,
        sandbox_posture,
        interactive,
        selected_thread,
    } = context;
    thread::spawn(move || {
        // SPEC-088 : lignes de sandbox reconnues, par item de commande. Un
        // acte `refusal` n'est écrit qu'à la fin en ÉCHEC du même item.
        let mut sandbox_lines_by_item: HashMap<String, String> = HashMap::new();
        let mut completed_file_items = (String::new(), HashSet::<String>::new());
        let mut refusals_recorded: HashSet<(String, String)> = HashSet::new();
        let mut termination = "flux Codex fermé".to_string();
        for line in lines {
            let line = match line {
                Ok(line) => line,
                Err(error) => {
                    termination = format!(
                        "lecture Codex impossible ({:?}): {}",
                        error.kind(),
                        error.to_string().chars().take(512).collect::<String>()
                    );
                    break;
                }
            };
            let raw = line.as_bytes().to_vec();
            let Ok(value) = serde_json::from_str::<Value>(&line) else {
                push_source(
                    &observations,
                    raw,
                    ManagedEventKind::Error {
                        detail: "ligne Codex invalide".to_string(),
                    },
                );
                continue;
            };
            if let Some(id) = value.get("id").and_then(Value::as_u64)
                && (value.get("result").is_some() || value.get("error").is_some())
                && let Some(waiter) = waiters.lock().recover_poison().remove(&id)
            {
                // Le vrai résultat turn/start établit le tour avant de rendre
                // le waiter : les callbacks qui le suivent ne gagnent pas une
                // course artificielle contre le worker, et aucun id n'est deviné.
                // Hors mode natif aussi (le worker le refixe ensuite à l'identique) :
                // sinon un compactage immédiat perdait son fait « running » (macOS).
                if (forward_child_requests || !interactive)
                    && let Some(turn_id) = value.pointer("/result/turn/id").and_then(Value::as_str)
                {
                    let message_id = queue
                        .0
                        .lock()
                        .recover_poison()
                        .active
                        .as_ref()
                        .map(|active| active.message_id.clone());
                    if let Some(message_id) = message_id {
                        set_active_turn_id(&active_detail, &message_id, turn_id);
                    }
                }
                // Publier le snapshot initial avant de rendre le waiter : aucun
                // ancien crédit positif ne peut dépasser une notification de retrait.
                if let Some(snapshot) = value.pointer("/result/rateLimits") {
                    for (window, status, resets_at, used_percent, credits) in
                        rate_limits_from_snapshot(Some(snapshot))
                    {
                        push_source(
                            &observations,
                            raw.clone(),
                            ManagedEventKind::RateLimitObserved {
                                window,
                                status,
                                resets_at,
                                used_percent,
                                credits,
                            },
                        );
                    }
                }
                let result = value
                    .get("error")
                    .map(|error| Err(provider_error_reason(error)))
                    .unwrap_or_else(|| {
                        Ok(ServerResponse {
                            value: value.get("result").cloned().unwrap_or(Value::Null),
                            raw,
                        })
                    });
                let _ = waiter.send(result);
                continue;
            }
            let method = value.get("method").and_then(Value::as_str);
            if interactive {
                let selected = selected_thread.lock().recover_poison().clone();
                let incoming_thread = value
                    .pointer("/params/threadId")
                    .or_else(|| value.pointer("/params/thread/id"))
                    .and_then(Value::as_str);
                if let (Some(selected), Some(incoming)) = (selected.as_deref(), incoming_thread)
                    && selected != incoming
                {
                    // L'app-server publie globalement les événements de ses
                    // autres fils, notamment les sous-agents internes Codex.
                    // Leur présence ne prouve aucune navigation humaine : ne
                    // rien projeter dans le fil sélectionné, son journal ou
                    // ses réponses de permission.
                    continue;
                }
            }
            if let (Some(method), Some(request_id)) = (method, value.get("id"))
                && let Err(detail) = record_provider_request(
                    &journal,
                    &queue,
                    &pending_request,
                    method,
                    request_id,
                    &value,
                    interactive.then_some(&active_detail),
                )
            {
                push_source(
                    &observations,
                    raw.clone(),
                    ManagedEventKind::JournalFailed {
                        detail: format!(
                            "trace durable d'une requête fournisseur impossible: {detail}"
                        ),
                    },
                );
            }
            // La TUI est l'unique répondante aux requêtes du serveur. Cela
            // couvre aussi les nouvelles méthodes, sans refus automatique ni
            // acceptation concurrente cachée du client Bridget.
            if interactive && value.get("id").is_some() && method.is_some() {
                push_source(
                    &observations,
                    raw,
                    ManagedEventKind::Update {
                        detail: "requête Codex réservée à la TUI native".into(),
                    },
                );
                continue;
            }
            if (forward_child_requests || method == Some("item/tool/requestUserInput"))
                && value.get("id").is_some()
                && method.is_some_and(|method| {
                    is_approval_request(method)
                        || is_mcp_elicitation_request(method)
                        || method == "item/tool/requestUserInput"
                })
            {
                let method = method.unwrap();
                let message_id = {
                    let state = queue.0.lock().recover_poison();
                    state
                        .active
                        .as_ref()
                        .filter(|_| !state.closed && state.steering_open)
                        .map(|active| active.message_id.clone())
                };
                let linked = alive.load(Ordering::SeqCst)
                    && active_detail
                        .lock()
                        .recover_poison()
                        .as_ref()
                        .is_some_and(|detail| {
                            message_id.as_deref() == Some(detail.message_id.as_str())
                                && !detail.message_id.is_empty()
                                && detail.turn_id.is_some()
                                && source_matches_active_turn(detail, &value)
                        });
                let mut refusal = None;
                let mut reply = None;
                if is_mcp_elicitation_request(method) {
                    let params = value.get("params").unwrap_or(&Value::Null);
                    let mode = params.get("mode").and_then(Value::as_str);
                    let empty_bridget = linked
                        && params["serverName"] == "bridget"
                        && mode == Some("form")
                        && params
                            .pointer("/requestedSchema/properties")
                            .and_then(Value::as_object)
                            .is_some_and(serde_json::Map::is_empty)
                        && params
                            .pointer("/requestedSchema/required")
                            .is_none_or(|value| {
                                value.is_null() || value.as_array().is_some_and(Vec::is_empty)
                            })
                        && params.get("_meta").is_none_or(|value| {
                            value.is_null()
                                || value.as_object().is_some_and(serde_json::Map::is_empty)
                        });
                    if empty_bridget {
                        // Exception149, réemploi du producteur de réponse natif.
                        reply = mcp_elicitation_response(&value).map(|response| response.0);
                    } else {
                        refusal = Some(if !linked {
                            "request_unlinked"
                        } else if mode == Some("url") {
                            "url_mode_unsupported"
                        } else {
                            "approval_method_unsupported"
                        });
                        reply = Some(json!({"id":value["id"],"result":{"action":"decline"}}));
                    }
                } else if !linked {
                    refusal = Some("request_unlinked");
                } else {
                    match normalize_child_provider_request(&value, &provider_session) {
                        Ok((request, approval)) => {
                            let mut pending = child_requests.lock().recover_poison();
                            if let Some(index) = pending.iter().position(|entry| {
                                entry.request.provider_session == request.provider_session
                                    && entry.request.turn_id == request.turn_id
                                    && entry.request.provider_request_id
                                        == request.provider_request_id
                            }) {
                                if pending[index].request != request {
                                    // Une collision ne laisse pas l'ancienne
                                    // question autoriser le nouveau corps RPC.
                                    pending.remove(index);
                                    refusal = Some("request_conflict");
                                }
                            } else if pending.len() >= 4 {
                                refusal = Some("request_budget_exhausted");
                            } else {
                                pending.push(PendingChildRequest {
                                    request: request.clone(),
                                    thread_id: value["params"]["threadId"].as_str().unwrap().into(),
                                    message_id: message_id.unwrap_or_default(),
                                    approval,
                                    uncertain: false,
                                });
                                // Aucun verrou table en émission : le wrapper peut
                                // immédiatement répondre sans inversion/backpressure.
                                drop(pending);
                                let projected = serde_json::to_vec(
                                    &json!({"id":value["id"],"method":method,"status":"forwarded"}),
                                )
                                .unwrap_or_default();
                                push_source(
                                    &observations,
                                    projected,
                                    ManagedEventKind::ProviderRequest { request },
                                );
                                continue;
                            }
                        }
                        Err(code) => refusal = Some(code),
                    }
                }
                if let Some(code) = refusal {
                    if reply.is_none() {
                        reply = Some(
                            json!({"id":value["id"],"error":{"code":-32000,"message":"child_request_refused"}}),
                        );
                    }
                    let diagnostic =
                        json!({"id":value["id"],"method":method,"status":"refused","code":code});
                    push_source(
                        &observations,
                        serde_json::to_vec(&diagnostic).unwrap_or_default(),
                        ManagedEventKind::Diagnostic {
                            code: code.into(),
                            reference: provider_fingerprint(
                                b"delegated-runtime",
                                method.as_bytes(),
                            ),
                        },
                    );
                }
                if let Some(reply) = reply
                    && let Err(error) = write_value(&writer, reply)
                {
                    alive.store(false, Ordering::SeqCst);
                    push_source(
                        &observations,
                        serde_json::to_vec(
                            &json!({"id":value["id"],"method":method,"status":"unknown"}),
                        )
                        .unwrap_or_default(),
                        ManagedEventKind::Error {
                            detail: format!("child_request_write_uncertain: {error}"),
                        },
                    );
                }
                continue;
            }
            match method {
                Some("account/rateLimits/updated") => {
                    for (window, status, resets_at, used_percent, credits) in
                        rate_limit_update_facts(value.pointer("/params/rateLimits"))
                    {
                        push_source(
                            &observations,
                            raw.clone(),
                            ManagedEventKind::RateLimitObserved {
                                window,
                                status,
                                resets_at,
                                used_percent,
                                credits,
                            },
                        );
                    }
                }
                Some("serverRequest/resolved") if forward_child_requests => {
                    let thread_id = value.pointer("/params/threadId").and_then(Value::as_str);
                    let request_id = value.pointer("/params/requestId").and_then(|value| {
                        serde_json::from_value::<ChildProviderRequestId>(value.clone()).ok()
                    });
                    let mut pending = child_requests.lock().recover_poison();
                    let index = pending.iter().position(|entry| {
                        Some(entry.thread_id.as_str()) == thread_id
                            && Some(&entry.request.provider_request_id) == request_id.as_ref()
                    });
                    let projected = index.map(|index| {
                        let entry = &pending[index];
                        let raw = json!({"method":"serverRequest/resolved","params":{"threadId":thread_id,"requestId":request_id},
                            "correlation":{"provider_session":entry.request.provider_session,"turn_id":entry.request.turn_id,"provider_request_id":entry.request.provider_request_id}});
                        let code = if entry.uncertain { "child_request_outcome_unknown" } else { "child_request_closed" };
                        if !entry.uncertain { pending.remove(index); }
                        (raw,code)
                    });
                    drop(pending);
                    if let Some((raw, code)) = projected {
                        push_source(
                            &observations,
                            serde_json::to_vec(&raw).unwrap_or_default(),
                            ManagedEventKind::Diagnostic {
                                code: code.into(),
                                reference: provider_fingerprint(
                                    b"delegated-runtime",
                                    b"serverRequest/resolved",
                                ),
                            },
                        );
                    }
                }
                Some("turn/started") if interactive => {
                    if let Some(turn_id) = value.pointer("/params/turn/id").and_then(Value::as_str)
                    {
                        let mut state = queue.0.lock().recover_poison();
                        // Le tour OBSERVÉ est indépendant de la réservation
                        // d'un RPC Bridget, y compris pendant sa finalisation.
                        state.external_turn = Some(turn_id.into());
                        *active_detail.lock().recover_poison() = Some(CodexTurnDetail {
                            thread_id: value
                                .pointer("/params/threadId")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .into(),
                            turn_id: Some(turn_id.into()),
                            ..CodexTurnDetail::default()
                        });
                        push_source(
                            &observations,
                            raw.clone(),
                            ManagedEventKind::ActivityObserved { in_progress: true },
                        );
                    }
                    push_source(
                        &observations,
                        raw,
                        ManagedEventKind::Update {
                            detail: "turn/started Codex".into(),
                        },
                    );
                }
                // Le schéma produit par `codex app-server` 0.149.0 publie
                // `item/agentMessage/delta`. La forme sans préfixe reste
                // tolérée pour les traces antérieures à v2, sans modifier
                // les octets sources que la frontière commune conserve.
                // (A) Retranscription : chaque delta → journal `update` avec
                // le CONTENU exact. `forward_managed_events` ignore Update.
                Some("thread/settings/updated") => {
                    let selected = selected_thread.lock().recover_poison().clone();
                    if let Some(thread_id) = selected.as_deref()
                        && value.pointer("/params/threadId").and_then(Value::as_str)
                            == Some(thread_id)
                        && let Some(settings) = value.pointer("/params/threadSettings")
                    {
                        observe_native_permissions(&permission_observation, thread_id, settings);
                    }
                    if value.pointer("/params/threadId").and_then(Value::as_str)
                        == selected.as_deref()
                        && let Some(model) = value
                            .pointer("/params/threadSettings/model")
                            .and_then(Value::as_str)
                    {
                        let effort = value
                            .pointer("/params/threadSettings/effort")
                            .and_then(Value::as_str)
                            .map(str::to_owned);
                        push_source(
                            &observations,
                            raw,
                            ManagedEventKind::RuntimeObserved {
                                model: model.to_string(),
                                effort,
                            },
                        );
                    }
                }
                Some("item/agentMessage/delta" | "agentMessage/delta") => {
                    if let (Some(turn_id), Some(delta)) = (
                        value.pointer("/params/turnId").and_then(Value::as_str),
                        value.pointer("/params/delta").and_then(Value::as_str),
                    ) {
                        observations
                            .0
                            .lock()
                            .recover_poison()
                            .response_by_turn
                            .entry(turn_id.to_string())
                            .or_default()
                            .push_str(delta);
                        record_agent_text_delta(
                            &journal,
                            &observations,
                            &active_detail,
                            &value,
                            delta,
                        );
                    }
                    push_source(
                        &observations,
                        raw,
                        ManagedEventKind::Update {
                            detail: "agentMessage/delta Codex".to_string(),
                        },
                    );
                }
                // (B) Repli : message agent complet sans aucun delta (stdout
                // sans flux partiel). `item/completed` type agentMessage.
                Some("item/completed") => on_item_completed(
                    &journal,
                    &observations,
                    &active_detail,
                    &value,
                    raw,
                    &cwd,
                    interactive,
                    &queue,
                    sandbox_posture,
                    &mut completed_file_items,
                    &mut sandbox_lines_by_item,
                    &mut refusals_recorded,
                ),
                // Mesure 2026-08-27 (codex app-server 0.149) : la COMMANDE réelle
                // est dans item/started type=commandExecution.params.item.command.
                // Les outputDelta ne portent que la sortie — ce n'est pas le nom.
                Some("item/started") => {
                    record_context_compaction(&journal, &observations, &active_detail, &value);
                    let item = value.pointer("/params/item").unwrap_or(&Value::Null);
                    if interactive
                        && item.get("type").and_then(Value::as_str) == Some("userMessage")
                    {
                        let state = queue.0.lock().recover_poison();
                        let client_id = item.get("clientId").and_then(Value::as_str).filter(|id| {
                            state.active.as_ref().is_some_and(|a| a.message_id == *id)
                        });
                        let mut active = active_detail.lock().recover_poison();
                        if let Some(detail) = active
                            .as_mut()
                            .filter(|a| source_matches_active_turn(a, &value))
                        {
                            // Un ajout humain (steer depuis la TUI) n'efface
                            // jamais le propriétaire du tour déjà démarré.
                            if !detail.input_seen {
                                detail.message_id = client_id.unwrap_or_default().into();
                            }
                            // Projection texte du contenu attesté, sans perdre
                            // les autres éléments natifs conservés dans input.
                            let body: String = item
                                .get("content")
                                .and_then(Value::as_array)
                                .into_iter()
                                .flatten()
                                .filter(|part| {
                                    part.get("type").and_then(Value::as_str) == Some("text")
                                })
                                .filter_map(|part| part.get("text").and_then(Value::as_str))
                                .collect();
                            record_or_terminal(
                                &journal,
                                &observations,
                                if detail.input_seen {
                                    "user_message"
                                } else {
                                    "turn_start"
                                },
                                client_id,
                                json!({"from": if client_id.is_some() { "bridget" } else { "human" }, "body": body, "provider_turn_id": detail.turn_id, "input": item.get("content")}),
                            );
                            detail.input_seen = true;
                        }
                    }
                    observe_provider_item_id(&active_detail, &value);
                    if item.get("type").and_then(Value::as_str) == Some("userMessage") {
                        observe_steered_user_message_visibility(
                            &observations,
                            &active_detail,
                            &value,
                        );
                    }
                    if item.get("type").and_then(Value::as_str) == Some("commandExecution")
                        && let Some(command) = item
                            .get("command")
                            .and_then(Value::as_str)
                            .map(str::trim)
                            .filter(|command| !command.is_empty())
                    {
                        record_active_act(
                            &journal,
                            &observations,
                            &active_detail,
                            &value,
                            CodexActKind::Command,
                            command,
                            Some("item/started"),
                        );
                    }
                    push_source(
                        &observations,
                        raw,
                        ManagedEventKind::Update {
                            detail: "item/started Codex".to_string(),
                        },
                    );
                }
                Some(
                    method @ ("item/reasoning/summaryTextDelta"
                    | "item/reasoning/summaryPartAdded"
                    | "item/reasoning/textDelta"),
                ) => {
                    observe_reasoning(&active_detail, method, &value);
                    push_source(
                        &observations,
                        raw,
                        ManagedEventKind::Update {
                            detail: format!("{method} Codex"),
                        },
                    );
                }
                Some("item/commandExecution/outputDelta") => {
                    // Sortie stdout uniquement — ne pas la journaliser comme
                    // acte « command » (sinon le fil affiche la sortie à la
                    // place du nom, constat relec*). Le nom vient de item/started.
                    // SPEC-088 : un refus de sandbox reconnu sur cette sortie
                    // devient un acte `refusal`, une fois par tour et couche.
                    // Même garde de provenance que la fin de commande : une
                    // sortie d'un autre tour ne contamine pas l'item actif.
                    let from_active_turn = active_detail
                        .lock()
                        .recover_poison()
                        .as_ref()
                        .is_some_and(|active| source_matches_active_turn(active, &value));
                    if from_active_turn
                        && let (Some(delta), Some(item_id)) = (
                            value.pointer("/params/delta").and_then(Value::as_str),
                            value.pointer("/params/itemId").and_then(Value::as_str),
                        )
                    {
                        remember_sandbox_lines(delta, item_id, &mut sandbox_lines_by_item);
                    }
                    push_source(
                        &observations,
                        raw,
                        ManagedEventKind::Update {
                            detail: "item/commandExecution/outputDelta Codex".to_string(),
                        },
                    );
                }
                Some("item/fileChange/patchUpdated") => {
                    if let Some(changes) =
                        value.pointer("/params/changes").and_then(Value::as_array)
                    {
                        for change in changes {
                            let Some(path) = change
                                .get("path")
                                .and_then(Value::as_str)
                                .filter(|path| !path.is_empty())
                            else {
                                continue;
                            };
                            let kind = change
                                .pointer("/kind/type")
                                .and_then(Value::as_str)
                                .filter(|kind| !kind.is_empty());
                            record_active_act(
                                &journal,
                                &observations,
                                &active_detail,
                                &value,
                                CodexActKind::File,
                                path,
                                kind,
                            );
                        }
                    }
                    push_source(
                        &observations,
                        raw,
                        ManagedEventKind::Update {
                            detail: "item/fileChange/patchUpdated Codex".to_string(),
                        },
                    );
                }
                Some("item/plan/delta") => {
                    if let Some(delta) = value
                        .pointer("/params/delta")
                        .and_then(Value::as_str)
                        .filter(|delta| !delta.is_empty())
                    {
                        record_active_act(
                            &journal,
                            &observations,
                            &active_detail,
                            &value,
                            CodexActKind::Plan,
                            delta,
                            None,
                        );
                    }
                    push_source(
                        &observations,
                        raw,
                        ManagedEventKind::Update {
                            detail: "item/plan/delta Codex".to_string(),
                        },
                    );
                }
                Some("item/tool/call") => {
                    if let Some((reply, detail)) =
                        dynamic_tool_response(&value, dynamic_tool_handler.as_ref())
                    {
                        if let Err(error) = write_value(&writer, reply) {
                            push_source(
                                &observations,
                                raw.clone(),
                                ManagedEventKind::Error {
                                    detail: format!(
                                        "réponse à l’outil dynamique Codex impossible: {error}"
                                    ),
                                },
                            );
                        }
                        push_source(
                            &observations,
                            raw,
                            ManagedEventKind::Update {
                                detail: detail.to_string(),
                            },
                        );
                    } else {
                        push_source(
                            &observations,
                            raw,
                            ManagedEventKind::Error {
                                detail: "requête d’outil dynamique Codex invalide".to_string(),
                            },
                        );
                    }
                }
                Some(method) if is_mcp_elicitation_request(method) => {
                    if let Some((reply, payload)) = mcp_elicitation_response(&value) {
                        let message_id = queue
                            .0
                            .lock()
                            .recover_poison()
                            .active
                            .as_ref()
                            .map(|active| active.message_id.clone());
                        record_or_terminal(
                            &journal,
                            &observations,
                            "provider_request",
                            message_id.as_deref(),
                            payload,
                        );
                        if let Err(error) = write_value(&writer, reply) {
                            push_source(
                                &observations,
                                raw.clone(),
                                ManagedEventKind::Error {
                                    detail: format!(
                                        "réponse à l’élicitation MCP Codex impossible: {error}"
                                    ),
                                },
                            );
                        }
                    } else {
                        push_source(
                            &observations,
                            raw.clone(),
                            ManagedEventKind::Error {
                                detail: "élicitation MCP Codex invalide".to_string(),
                            },
                        );
                    }
                    push_source(
                        &observations,
                        raw,
                        ManagedEventKind::Update {
                            detail: "mcpServer/elicitation/request Codex".to_string(),
                        },
                    );
                }
                Some(method) if is_approval_request(method) => {
                    record_active_act(
                        &journal,
                        &observations,
                        &active_detail,
                        &value,
                        CodexActKind::Approval,
                        approval_text(&value, method),
                        Some(method),
                    );
                    if let Some((reply, payload)) = approval_response(&value, &permissions) {
                        let message_id = queue
                            .0
                            .lock()
                            .recover_poison()
                            .active
                            .as_ref()
                            .map(|active| active.message_id.clone());
                        record_or_terminal(
                            &journal,
                            &observations,
                            "permission",
                            message_id.as_deref(),
                            payload,
                        );
                        let _ = write_value(&writer, reply);
                    }
                    push_source(
                        &observations,
                        raw,
                        ManagedEventKind::Update {
                            detail: format!("{method} Codex"),
                        },
                    );
                }
                Some("turn/completed") => {
                    let turn_id = value.pointer("/params/turn/id").and_then(Value::as_str);
                    let status = value.pointer("/params/turn/status").and_then(Value::as_str);
                    if let (Some(turn_id), Some(status)) = (turn_id, status) {
                        if forward_child_requests {
                            let thread_id =
                                value.pointer("/params/threadId").and_then(Value::as_str);
                            child_requests.lock().recover_poison().retain(|entry| {
                                entry.request.turn_id != turn_id
                                    || Some(entry.thread_id.as_str()) != thread_id
                            });
                        }
                        if interactive {
                            let mut state = queue.0.lock().recover_poison();
                            if state.external_turn.as_deref() != Some(turn_id) {
                                // Un ancien terminal ou sa répétition ne peut
                                // ni clore le tour courant ni remplir la table.
                                continue;
                            }
                            if state.external_turn.as_deref() == Some(turn_id) {
                                let message_id = active_detail
                                    .lock()
                                    .recover_poison()
                                    .as_ref()
                                    .map(|a| a.message_id.clone())
                                    .unwrap_or_default();
                                let reasoning = finish_reasoning(&active_detail, &message_id);
                                record_or_terminal(
                                    &journal,
                                    &observations,
                                    "reasoning",
                                    Some(&message_id),
                                    reasoning,
                                );
                                record_or_terminal(
                                    &journal,
                                    &observations,
                                    "turn_end",
                                    Some(&message_id),
                                    json!({"provider_turn_id": turn_id, "stop_reason": codex_turn_stop_reason(&value, status)}),
                                );
                                *active_detail.lock().recover_poison() = None;
                                state.external_turn = None;
                                push_source(
                                    &observations,
                                    raw.clone(),
                                    ManagedEventKind::ActivityObserved { in_progress: false },
                                );
                                queue.1.notify_all();
                                // Aucun worker n'attend ce tour humain : ne pas
                                // conserver un terminal orphelin sans borne.
                                if message_id.is_empty() {
                                    observations
                                        .0
                                        .lock()
                                        .recover_poison()
                                        .response_by_turn
                                        .remove(turn_id);
                                    continue;
                                }
                            }
                        }
                        let terminal = codex_turn_terminal(&value, status);
                        observations
                            .0
                            .lock()
                            .recover_poison()
                            .terminal_by_turn
                            .insert(turn_id.to_string(), terminal);
                        observations.1.notify_all();
                    }
                    push_source(
                        &observations,
                        raw,
                        ManagedEventKind::Update {
                            detail: "turn/completed Codex".to_string(),
                        },
                    );
                }
                Some("modelRerouted" | "model/rerouted" | "ModelReroutedNotification") => {
                    if let Some(served) = served_model_from_codex(&value) {
                        maybe_record_mismatch(
                            &journal,
                            &observations,
                            pinned_model.as_deref(),
                            &served,
                        );
                        push_source(
                            &observations,
                            raw,
                            ManagedEventKind::ModelObserved { model: served },
                        );
                    } else {
                        push_source(
                            &observations,
                            raw,
                            ManagedEventKind::Update {
                                detail: "reroutage Codex sans modèle attesté".to_string(),
                            },
                        );
                    }
                }
                Some(other) => {
                    let method = project_provider_method(other);
                    // Une trame avec `id` est une requête JSON-RPC. La traiter
                    // comme une simple notification laisse Codex attendre pour
                    // toujours. La réponse négative est volontairement sans
                    // corps fournisseur : le journal garde seulement un code
                    // stable et l'empreinte déjà publique de la méthode.
                    if value.get("id").is_some()
                        && let Some((reply, payload, delegated_reference)) =
                            unsupported_provider_request_response(&value)
                    {
                        // Observation volontairement bornée : seule la méthode
                        // publique est journalisée pour identifier un nouveau
                        // contrat app-server sans exposer ses arguments.
                        eprintln!("[BRIDGET] requête Codex non prise en charge: {other}");
                        let message_id = queue
                            .0
                            .lock()
                            .recover_poison()
                            .active
                            .as_ref()
                            .map(|active| active.message_id.clone());
                        record_or_terminal(
                            &journal,
                            &observations,
                            "provider_request_rejected",
                            message_id.as_deref(),
                            payload,
                        );
                        if let Err(error) = write_value(&writer, reply) {
                            push_source(
                                &observations,
                                raw.clone(),
                                ManagedEventKind::Error {
                                    detail: format!(
                                        "réponse à la requête Codex non prise en charge impossible: {error}"
                                    ),
                                },
                            );
                        } else {
                            push_source(
                                &observations,
                                raw.clone(),
                                ManagedEventKind::Diagnostic {
                                    code: "unsupported_provider_request".to_string(),
                                    reference: delegated_reference,
                                },
                            );
                        }
                    }
                    push_source(
                        &observations,
                        raw,
                        ManagedEventKind::Update {
                            detail: format!("notification Codex: {method}"),
                        },
                    )
                }
                None => push_source(
                    &observations,
                    raw,
                    ManagedEventKind::Error {
                        detail: "trame Codex sans méthode".to_string(),
                    },
                ),
            }
        }
        alive.store(false, Ordering::SeqCst);
        child_requests.lock().recover_poison().clear();
        let pending = std::mem::take(&mut *waiters.lock().recover_poison());
        for (_, waiter) in pending {
            let _ = waiter.send(Err(termination.clone()));
        }
        observations.1.notify_all();
    })
}

fn take_pending_request(
    pending_request: &PendingRequest,
    message_id: &str,
) -> Option<PendingProviderRequest> {
    let mut pending = pending_request.lock().recover_poison();
    if pending
        .as_ref()
        .and_then(|request| request.message_id.as_deref())
        != Some(message_id)
    {
        return None;
    }
    pending.take()
}
fn record_provider_request(
    journal: &Journal,
    queue: &Arc<(Mutex<QueueState>, Condvar)>,
    pending_request: &PendingRequest,
    method: &str,
    request_id: &Value,
    frame: &Value,
    interactive_detail: Option<&ActiveTurnDetail>,
) -> Result<(), String> {
    // La réception est linéarisée par ce verrou : dès qu'un lecteur l'a pris,
    // le worker terminal attendra que le fait soit mis en file avant de lire
    // la dernière requête pendante.
    let mut pending = pending_request.lock().recover_poison();
    let message_id = if let Some(detail) = interactive_detail {
        detail
            .lock()
            .recover_poison()
            .as_ref()
            .filter(|d| source_matches_active_turn(d, frame) && !d.message_id.is_empty())
            .map(|d| d.message_id.clone())
    } else {
        queue
            .0
            .lock()
            .recover_poison()
            .active
            .as_ref()
            .map(|active| active.message_id.clone())
    };
    let request = PendingProviderRequest {
        message_id,
        method: project_provider_method(method),
        request_id: project_request_id(request_id),
        turn_id: frame
            .pointer("/params/turnId")
            .and_then(Value::as_str)
            .map(|turn_id| provider_fingerprint(b"turn-id", turn_id.as_bytes())),
    };
    let payload = request.payload();

    let result = record(
        journal,
        "provider_request",
        request.message_id.as_deref(),
        payload,
    );
    *pending = Some(request);
    result
}
fn project_provider_method(method: &str) -> String {
    match method {
        COMMAND_EXECUTION_APPROVAL_METHOD => COMMAND_EXECUTION_APPROVAL_METHOD.to_string(),
        _ => provider_fingerprint(b"method", method.as_bytes()),
    }
}
fn project_request_id(request_id: &Value) -> String {
    match request_id {
        Value::String(value) => provider_fingerprint(b"request-id:string", value.as_bytes()),
        Value::Number(value) => {
            provider_fingerprint(b"request-id:number", value.to_string().as_bytes())
        }
        Value::Null => provider_fingerprint(b"request-id:null", &[]),
        Value::Bool(_) | Value::Array(_) | Value::Object(_) => {
            provider_fingerprint(b"request-id:invalid", &[])
        }
    }
}
// Project only an attested protocol code; keep unknown interactive stop reasons unchanged.
fn codex_turn_stop_reason<'a>(value: &Value, status: &'a str) -> &'a str {
    if status == "failed"
        && value
            .pointer("/params/turn/error/codexErrorInfo")
            .and_then(Value::as_str)
            == Some("serverOverloaded")
    {
        "terminal Codex: failed:cause=server_overloaded"
    } else if status == "failed" && codex_model_not_found217(value) {
        "terminal Codex: failed:cause=model_not_found"
    } else {
        status
    }
}

// Le fournisseur peut envelopper un objet HTTP dans message. On ne publie
// jamais ce texte : seule cette combinaison fermée devient une cause native.
fn codex_model_not_found217(value: &Value) -> bool {
    if value
        .pointer("/params/turn/error/codexErrorInfo")
        .and_then(Value::as_str)
        != Some("other")
    {
        return false;
    }
    let Some(message) = value
        .pointer("/params/turn/error/message")
        .and_then(Value::as_str)
        .filter(|message| message.len() <= 4096)
    else {
        return false;
    };
    let Ok(error) = serde_json::from_str::<Value>(message) else {
        return false;
    };
    error.get("type").and_then(Value::as_str) == Some("error")
        && error.get("status").and_then(Value::as_u64) == Some(400)
        && error.pointer("/error/type").and_then(Value::as_str) == Some("invalid_request_error")
        && error.pointer("/error/code").and_then(Value::as_str) == Some("model_not_found")
}

fn codex_turn_terminal(value: &Value, status: &str) -> ManagedTerminal {
    match status {
        "completed" => ManagedTerminal::Completed,
        "interrupted" => ManagedTerminal::Cancelled,
        _ => {
            let reason = codex_turn_stop_reason(value, status);
            ManagedTerminal::Failed {
                detail: if reason == status {
                    format!("terminal Codex: {status}")
                } else {
                    reason.to_string()
                },
            }
        }
    }
}

fn provider_error_reason(error: &Value) -> String {
    let code = error.get("code").and_then(Value::as_i64);
    if code == Some(-32001) {
        return CODEX_SATURATED_REASON.to_string();
    }
    let reference = provider_fingerprint(b"error", error.to_string().as_bytes());
    // Marqueur de NOTRE cru, pose sur la seule foi d'un champ STRUCTURE du
    // protocole (`codexErrorInfo.activeTurnNotSteerable`). Mesure du 28/08 sur
    // codex-cli 0.150.1 : le message fournisseur est « cannot steer a review
    // turn », mais il n'est jamais remonte — l'empreinte le remplace, et c'est
    // voulu. Sans ce marqueur, aucun appelant ne peut distinguer un tour non
    // pilotable d'un refus quelconque.
    let non_pilotable = error
        .pointer("/data/codexErrorInfo/activeTurnNotSteerable")
        .is_some();
    let suffixe = if non_pilotable {
        "; tour non pilotable"
    } else {
        ""
    };
    // Classer uniquement le refus fournisseur reconnu, sans exposer son contenu.
    let active_writer = code == Some(-32600)
        && error
            .get("message")
            .and_then(Value::as_str)
            .is_some_and(|message| {
                message
                    .strip_prefix("thread ")
                    .and_then(|message| message.strip_suffix(" already has an active writer"))
                    .is_some_and(|id| id.len() == 36 && uuid::Uuid::parse_str(id).is_ok())
            });
    if active_writer {
        return format!(
            "codex_thread_already_active: Codex conserve le droit d’écriture sur cette conversation. Fermez la session d’origine si elle est encore ouverte. Si le refus persiste, ce droit n’a pas été libéré{suffixe} (référence {reference})"
        );
    }
    match code {
        Some(code) => format!("erreur Codex (code {code}{suffixe}; référence {reference})"),
        None => format!("erreur Codex ({suffixe}référence {reference})"),
    }
}

fn provider_fingerprint(domain: &'static [u8], value: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(b"bridget/provider-request/v1\0");
    digest.update(domain);
    digest.update(b"\0");
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
    format!("sha256:{:x}", digest.finalize())
}

fn push_source(
    observations: &Arc<(Mutex<Observations>, Condvar)>,
    raw: Vec<u8>,
    kind: ManagedEventKind,
) {
    observations
        .0
        .lock()
        .recover_poison()
        .events
        .push_back(ManagedEvent {
            source: ManagedEventSource::CodexAppServer,
            origin: ManagedEventOrigin::SourceLine,
            raw,
            kind,
        });
    observations.1.notify_all();
}

#[cfg(test)]
#[path = "codex_app_server_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "codex_quota_refresh_tests.rs"]
mod quota_refresh_tests;

#[cfg(test)]
mod mobile183_compaction_tests {
    use super::*;
    #[test]
    fn mobile183_compaction_is_structural_and_scoped_to_the_active_turn() {
        let mut event = json!({"method":"item/started","params":{"threadId":"t","turnId":"r","item":{"id":"c","type":"contextCompaction"}}});
        let start = compaction_payload(&event, "t", "r").unwrap();
        assert_eq!(start["status"], "running");
        assert_eq!(start["item_id"], "c");
        assert!(crate::act_kind::validate_journal_write("update", &start).is_ok());
        assert!(compaction_payload(&event, "foreign", "r").is_none());
        assert!(compaction_payload(&event, "t", "old").is_none());
        event["method"] = json!("item/completed");
        assert_eq!(
            compaction_payload(&event, "t", "r").unwrap()["status"],
            "completed"
        );
        event["params"]["item"]["type"] = json!("agentMessage");
        assert!(compaction_payload(&event, "t", "r").is_none());
        event["params"]["item"]["type"] = json!("contextCompaction");
        event["params"]["item"]["id"] = json!("bad\nitem");
        assert!(compaction_payload(&event, "t", "r").is_none());
    }
}
