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
use crate::protocol::{PresenceMode, ProviderObservation, ProviderOperation};
use crate::transport::{Transport, TransportError};
use bridget_core::{BridgetMessage, MessageIntent, MessageOrigin};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{BufRead, BufReader, Write};
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
const TURN_POLL: Duration = Duration::from_millis(25);
const SATURATION_RETRIES: u32 = 4;
const CODEX_SATURATED_REASON: &str = "saturation Codex";
const COMMAND_EXECUTION_APPROVAL_METHOD: &str = "item/commandExecution/requestApproval";

type Writer = Arc<Mutex<Option<Box<dyn Write + Send>>>>;
type SourceLines = Box<dyn Iterator<Item = std::io::Result<String>> + Send>;
type Waiters = Arc<Mutex<HashMap<u64, mpsc::Sender<Result<ServerResponse, String>>>>>;
type Journal = Arc<Mutex<Option<JournalWriter>>>;
type PendingRequest = Arc<Mutex<Option<PendingProviderRequest>>>;
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
    cwd: String,
    waiters: Waiters,
    observations: Arc<(Mutex<Observations>, Condvar)>,
    alive: Arc<AtomicBool>,
    journal: Journal,
    queue: Arc<(Mutex<QueueState>, Condvar)>,
    pending_request: PendingRequest,
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
    provider_observation: Option<ProviderObservation>,
    reader_handle: Mutex<Option<thread::JoinHandle<()>>>,
    worker_handle: Mutex<Option<thread::JoinHandle<()>>>,
    interactive_socket: Option<std::path::PathBuf>,
}

impl CodexAppServerTransport {
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
        Self::spawn_endpoint(options, environment, inherit_stderr, None, None)
    }

    /// Un app-server privé, partagé avec la TUI native. Aucun octet terminal
    /// n'est injecté ; les deux clients utilisent le protocole fournisseur.
    pub fn spawn_interactive(
        options: CodexAppServerOptions,
        environment: &[(String, String)],
        socket: &Path,
    ) -> Result<Self, TransportError> {
        Self::spawn_endpoint(options, environment, false, Some(socket), None)
    }

    /// La sélection précède thread/resume, Register et la TUI : aucun fil
    /// provisoire et aucun premier tour avant attachement de Bridget.
    pub fn spawn_interactive_selecting(
        options: CodexAppServerOptions,
        environment: &[(String, String)],
        socket: &Path,
        selector: Option<InteractiveThreadSelector<'_>>,
    ) -> Result<Self, TransportError> {
        Self::spawn_endpoint(options, environment, false, Some(socket), selector)
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
    ) -> Result<Self, TransportError> {
        if options.queue_capacity == 0 {
            return Err(TransportError::DeliveryFailed(
                "queue Codex de capacité nulle".to_string(),
            ));
        }
        let mut command = Command::new(&options.command);
        validate_endpoint_bootstrap(&options, interactive_socket.is_some())?;
        command
            .args(&options.args)
            .envs(environment.iter().cloned())
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
        let selected_thread = Arc::new(Mutex::new(None));
        let reader_handle = spawn_reader(
            lines,
            ReaderContext {
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
            let mut initialization = json!({
                "clientInfo": { "name": "bridget", "version": env!("CARGO_PKG_VERSION") }
            });
            // thread/settings/update (091) est explicitement négocié ; aucune
            // permission supplémentaire n'est accordée par cette capacité filaire.
            initialization["capabilities"] = json!({"experimentalApi": true});
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
            if let Ok(rate_limits) = request_with_timeout(
                &writer,
                &waiters,
                &next_id,
                "account/rateLimits/read",
                Value::Null,
                RATE_LIMIT_READ_TIMEOUT,
            ) {
                for (window, status, resets_at, used_percent) in
                    rate_limits_from_snapshot(rate_limits.value.get("rateLimits"))
                {
                    push_source(
                        &observations,
                        rate_limits.raw.clone(),
                        ManagedEventKind::RateLimitObserved {
                            window,
                            status,
                            resets_at,
                            used_percent,
                        },
                    );
                }
            }
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
                writer
                    .lock()
                    .unwrap_or_else(|poison| poison.into_inner())
                    .take();
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
        *selected_thread.lock().unwrap_or_else(|e| e.into_inner()) = Some(thread_id.clone());
        let child = Arc::new(Mutex::new(child));
        let busy = Arc::new(AtomicBool::new(false));
        let worker_handle = spawn_worker(Worker {
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
            pending_request,
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
            .unwrap_or_else(|poison| poison.into_inner())
            .events
            .push_back(ManagedEvent::internal(
                ManagedEventSource::CodexAppServer,
                raw,
                kind,
            ));
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
                .unwrap_or_else(|poison| poison.into_inner())
                .events
                .push_back(ManagedEvent::internal(
                    ManagedEventSource::CodexAppServer,
                    raw,
                    ManagedEventKind::JournalFailed { detail },
                ));
            observations.1.notify_all();
        });
        *self
            .journal
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) =
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
        self.observations.1.notify_all();
        let (queue, wake) = &*self.queue;
        let mut queue = queue.lock().unwrap_or_else(|poison| poison.into_inner());
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
                .unwrap_or_else(|e| e.into_inner())
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
        self.writer
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .take();
        let mut child = self
            .child
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if let Some(socket) = &self.interactive_socket {
            if let Err(error) = stop_interactive_server(&mut child, socket) {
                // Garder le socket de récupération, jamais supprimer la seule
                // référence d'un enfant dont l'arrêt n'est pas confirmé.
                self.push_internal(ManagedEventKind::Error {
                    detail: error.to_string(),
                });
            }
        } else {
            let _ = unsafe { libc::kill(-(child.id() as i32), libc::SIGTERM) };
            let _ = child.wait();
        }
        drop(child);
        if let Some(handle) = self
            .reader_handle
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .take()
        {
            let _ = handle.join();
        }
        if let Some(handle) = self
            .worker_handle
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .take()
        {
            let _ = handle.join();
        }
        if let Some(journal) = self
            .journal
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .take()
        {
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
        let mut queue = queue.lock().unwrap_or_else(|poison| poison.into_inner());
        let human_message = message.origin == Some(MessageOrigin::Human);
        let has_active_turn = queue.active.is_some();
        // Un message humain doit toujours obtenir son propre tour. Le piloter
        // dans un tour en cours mélange sa réponse à celle du message actif :
        // l'utilisateur voit alors son envoi mais jamais sa réponse dédiée.
        // Le pilotage reste réservé au contrôle explicite.
        let steering_requested = message.intent == Some(MessageIntent::SteerCurrent);
        let interrupt_requested = message.intent == Some(MessageIntent::InterruptAndStart)
            || (human_message && has_active_turn);
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
            queue.messages.push_back(message.clone());
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
    fn select_runtime(
        &mut self,
        selection: crate::protocol::RuntimeSelection,
    ) -> crate::protocol::RuntimeSelectionOutcome {
        use crate::protocol::{
            RuntimeSelectionOutcome as Outcome, RuntimeSelectionRefusal as Refusal,
        };
        let refuse = |reason| Outcome::Refused { reason };
        if self.interactive_socket.is_some() {
            return refuse(Refusal::Unsupported); // La TUI native reste propriétaire de ses réglages.
        }
        if !selection.valid() {
            return refuse(Refusal::InvalidSelection);
        }
        if !self.alive.load(Ordering::SeqCst) {
            return refuse(Refusal::TargetUnavailable);
        }
        // O(n), au plus 4 pages de 100 modèles, une échéance ABSOLUE de 2 s.
        // Ce catalogue autorise le choix humain, jamais le bootstrap/rejeu figé.
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut cursor = Value::Null;
        for _ in 0..4 {
            let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                return refuse(Refusal::CatalogueUnavailable);
            };
            let Ok(response) = request_with_timeout(
                &self.writer,
                &self.waiters,
                &self.next_id,
                "model/list",
                json!({"cursor":cursor,"limit":100,"includeHidden":false}),
                remaining,
            ) else {
                return refuse(Refusal::CatalogueUnavailable);
            };
            let Some(models) = response.value.get("data").and_then(Value::as_array) else {
                return refuse(Refusal::CatalogueUnavailable);
            };
            if let Some(model) = models.iter().find(|model| {
                model.get("model").and_then(Value::as_str) == Some(selection.model.as_str())
            }) {
                let supported = model
                    .get("supportedReasoningEfforts")
                    .and_then(Value::as_array)
                    .is_some_and(|efforts| {
                        efforts.iter().any(|effort| {
                            effort.get("reasoningEffort").and_then(Value::as_str)
                                == Some(selection.effort.as_str())
                        })
                    });
                if !supported {
                    return refuse(Refusal::EffortUnavailable);
                }
                let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
                    return refuse(Refusal::CatalogueUnavailable);
                };
                // Le fournisseur détient déjà les réglages du fil. Ne pas en
                // créer un double Bridget, ni un turn/start vide de convenance.
                return match request_with_timeout(
                    &self.writer,
                    &self.waiters,
                    &self.next_id,
                    "thread/settings/update",
                    json!({"threadId":self.thread_id,
                    "model":selection.model,"effort":selection.effort}),
                    remaining,
                ) {
                    Ok(_) => Outcome::Selected { selection },
                    Err(error) if error.to_string().contains("-32601") => {
                        refuse(Refusal::Unsupported)
                    }
                    Err(_) => Outcome::OutcomeUnknown {},
                };
            }
            cursor = response
                .value
                .get("nextCursor")
                .cloned()
                .unwrap_or(Value::Null);
            if cursor.is_null() {
                return refuse(Refusal::ModelUnavailable);
            }
        }
        refuse(Refusal::CatalogueUnavailable)
    }
    fn set_private_profile_instructions(
        &mut self,
        instructions: &str,
    ) -> Result<(), TransportError> {
        *self
            .private_profile_instructions
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = Some(instructions.to_string());
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
        let active = self
            .active_detail
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
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
        self.child
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .id()
    }

    fn activate_journal(
        &self,
        root: &Path,
        agent: &str,
        live_feed: Option<JournalLiveFeed>,
    ) -> std::io::Result<()> {
        self.enable_journal_with_live_feed(root, agent, live_feed)
    }

    fn drain_events(&self) -> Vec<ManagedEvent> {
        self.observations
            .0
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .events
            .drain(..)
            .collect()
    }

    fn cancel_delivery(&self, message_id: &str, reason: &str) -> bool {
        let (queue, wake) = &*self.queue;
        let mut queue = queue.lock().unwrap_or_else(|poison| poison.into_inner());
        if let Some(active) = queue
            .active
            .as_ref()
            .filter(|active| active.message_id == message_id)
        {
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

    fn is_busy(&self) -> bool {
        self.busy.load(Ordering::SeqCst)
            || self
                .queue
                .0
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .external_turn
                .is_some()
    }
}

struct Worker {
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
    notify_timeout: Duration,
}

fn spawn_worker(worker: Worker) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let (message, cancel) = {
                let (queue, wake) = &*worker.queue;
                let mut queue = queue.lock().unwrap_or_else(|poison| poison.into_inner());
                while (queue.messages.is_empty() || queue.external_turn.is_some()) && !queue.closed
                {
                    queue = wake
                        .wait(queue)
                        .unwrap_or_else(|poison| poison.into_inner());
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
                *worker
                    .active_detail
                    .lock()
                    .unwrap_or_else(|poison| poison.into_inner()) = Some(CodexTurnDetail {
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
                worker
                    .active_detail
                    .lock()
                    .unwrap_or_else(|poison| poison.into_inner())
                    .take();
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
        "Aucune réponse inter-agent attendue. N'envoie pas d'accusé de réception, même pour annoncer ton silence. Ta réponse finale ne sera pas relayée."
    } else if interactive {
        "Si reply=true, réponds une seule fois par l'outil bridget_send avec to=from et in_reply_to=id ci-dessus. La réponse finale à l'écran n'est pas envoyée à cet agent."
    } else {
        "Si reply=true, le wrapper relaie automatiquement ta réponse finale à from avec in_reply_to=id ci-dessus. Utilise cette réponse finale pour terminer la demande ; pour un avancement distinct, utilise bridget_send avec to=from et reply=false, sans in_reply_to."
    };
    format!(
        "[Message Bridget : {}]\n{}\n\n[Réponse Bridget]\n{response}\nPour communiquer, utilise les outils MCP Bridget (bridget_send, bridget_who, bridget_ledger). S'ils ne sont pas affichés, cherche-les dans le catalogue d'outils disponible. L'accès à la socket depuis le shell restreint n'est pas requis ; un refus du shell ne prouve pas une panne MCP.",
        json!({"from": message.from, "to": message.to, "id": message.id, "reply": message.reply, "in_reply_to": message.in_reply_to}),
        message.body
    )
}

fn start_turn_with_retry(worker: &Worker, message: &BridgetMessage) -> Result<String, String> {
    let instructions = worker
        .private_profile_instructions
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .clone();
    // Les outils exposent leurs capacités ; le pilote ne promet ni interface
    // de rendu ni disponibilité du sandbox et ne transforme pas un document
    // HTML en ordre de publication ou d'exécution.
    // L'enveloppe est une projection fournisseur ; le corps durable reste intact.
    let body = communication_prompt(message, worker.interactive);
    let prompt = private_prompt(instructions.as_deref(), &body);
    let params = json!({
        "threadId": worker.thread_id,
        "clientUserMessageId": message.id,
        "input": [{ "type": "text", "text": &prompt }],
    });
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
            let mut observed = lock.lock().unwrap_or_else(|poison| poison.into_inner());
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
        let mut observed = lock.lock().unwrap_or_else(|poison| poison.into_inner());
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
        let observed = lock.lock().unwrap_or_else(|poison| poison.into_inner());
        let (next, _) = wake
            .wait_timeout(observed, wait)
            .unwrap_or_else(|poison| poison.into_inner());
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
    let state = lock.lock().unwrap_or_else(|poison| poison.into_inner());
    accepted
        .iter()
        .chain(state.steer.iter())
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
    let mut state = lock.lock().unwrap_or_else(|poison| poison.into_inner());
    state.steering_open = false;
    let steered: Vec<_> = state.steer.drain(..).collect();
    for message in steered.into_iter().rev() {
        state.messages.push_front(message);
    }
    for message in pending.drain(..).rev() {
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
            let mut state = lock.lock().unwrap_or_else(|poison| poison.into_inner());
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
                let mut state = lock.lock().unwrap_or_else(|poison| poison.into_inner());
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
    let mut state = lock.lock().unwrap_or_else(|poison| poison.into_inner());
    state.steering_open = false;
    // `turn/start` peut échouer avant `wait_for_turn`. Dans ce chemin, aucun
    // repli précédent n’a vidé `steer` : le restituer évite une perte humaine.
    let steered: Vec<_> = state.steer.drain(..).collect();
    for message in steered.into_iter().rev() {
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
        .unwrap_or_else(|poison| poison.into_inner())
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
    let mut active = active_detail
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
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
    let mut active = active_detail
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
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
    let active = active_detail
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
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
        .unwrap_or_else(|poison| poison.into_inner())
        .consumed_steers_by_turn
        .entry(key)
        .or_default()
        .insert(message_id.to_string());
    observations.1.notify_all();
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
        .unwrap_or_else(|poison| poison.into_inner())
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
        .unwrap_or_else(|poison| poison.into_inner())
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
        let mut active = active_detail
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
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
        let mut active = active_detail
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
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
    let mut active = active_detail
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
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
    let detail = active_detail
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .take();
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

/// Réponse JSON-RPC au `item/*/requestApproval` Codex, calquée sur
/// `permission_response` ACP : le réglage `permissions` de l'équipier décide
/// `accept` ou `decline`, l'`id` de la demande est repris tel quel.
fn approval_response(value: &Value, permissions: &str) -> Option<(Value, Value)> {
    let id = value.get("id")?;
    let method = value.get("method").and_then(Value::as_str).unwrap_or("");
    let decision = if permissions == "allow" {
        "accept"
    } else {
        "decline"
    };
    let decision = value
        .pointer("/params/availableDecisions")
        .and_then(Value::as_array)
        .and_then(|choices| {
            choices
                .iter()
                .filter_map(Value::as_str)
                .find(|choice| *choice == decision)
                .or_else(|| choices.iter().filter_map(Value::as_str).next())
        })
        .unwrap_or(decision);
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
        .unwrap_or_else(|poison| poison.into_inner())
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
    waiters
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .insert(id, sender);
    if let Err(error) = write_value(
        writer,
        json!({ "id": id, "method": method, "params": params }),
    ) {
        waiters
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .remove(&id);
        return Err(error);
    }
    let result = receiver.recv_timeout(timeout);
    // Un fournisseur muet ne doit pas accumuler un waiter à chaque contrôle.
    // La réponse tardive reste ignorée, jamais réattribuée au contrôle suivant.
    if result.is_err() {
        waiters
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .remove(&id);
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
) -> Vec<(String, String, Option<i64>, Option<u8>)> {
    let Some(snapshot) = snapshot else {
        return Vec::new();
    };
    let status = snapshot
        .get("rateLimitReachedType")
        .and_then(Value::as_str)
        .unwrap_or("available")
        .to_string();
    let mut facts = Vec::new();
    for key in ["primary", "secondary"] {
        if let Some(fact) = window_fact_from_codex(snapshot.get(key), key, &status) {
            facts.push(fact);
        }
    }
    facts
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
    let mut writer = writer
        .lock()
        .map_err(|error| TransportError::Io(error.to_string()))?;
    let writer = writer.as_mut().ok_or(TransportError::AgentDead)?;
    writeln!(writer, "{value}").map_err(|error| TransportError::Io(error.to_string()))?;
    writer
        .flush()
        .map_err(|error| TransportError::Io(error.to_string()))
}

fn spawn_reader(lines: SourceLines, context: ReaderContext) -> thread::JoinHandle<()> {
    let ReaderContext {
        cwd,
        waiters,
        observations,
        alive,
        journal,
        queue,
        pending_request,
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
                && let Some(waiter) = waiters
                    .lock()
                    .unwrap_or_else(|poison| poison.into_inner())
                    .remove(&id)
            {
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
                let selected = selected_thread
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .clone();
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
            match method {
                Some("turn/started") if interactive => {
                    if let Some(turn_id) = value.pointer("/params/turn/id").and_then(Value::as_str)
                    {
                        let mut state = queue.0.lock().unwrap_or_else(|e| e.into_inner());
                        // Le tour OBSERVÉ est indépendant de la réservation
                        // d'un RPC Bridget, y compris pendant sa finalisation.
                        state.external_turn = Some(turn_id.into());
                        *active_detail.lock().unwrap_or_else(|e| e.into_inner()) =
                            Some(CodexTurnDetail {
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
                    let selected = selected_thread
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .clone();
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
                            .unwrap_or_else(|poison| poison.into_inner())
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
                Some("item/completed") => {
                    let item = value.pointer("/params/item");
                    if item.and_then(|i| i.get("type")).and_then(Value::as_str)
                        == Some("fileChange")
                        && item.and_then(|i| i.get("status")).and_then(Value::as_str)
                            == Some("completed")
                    {
                        let message_id = active_detail
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .as_ref()
                            .filter(|active| source_matches_active_turn(active, &value))
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
                                completed_file_items = (turn.into(), HashSet::new());
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
                                log::warn!(
                                    "observation_gap: déduplication Codex saturée pour ce tour"
                                );
                            }
                            for change in changes.iter().take(if accepted { 256 } else { 0 }) {
                                if let Some(path) = change.get("path").and_then(Value::as_str)
                                    && let Some(payload) = crate::journal::confirmed_write_payload(
                                        path,
                                        Path::new(&cwd),
                                    )
                                {
                                    record_or_terminal(
                                        &journal,
                                        &observations,
                                        "update",
                                        Some(&message_id),
                                        payload,
                                    );
                                }
                            }
                            if changes.len() > 256 {
                                log::warn!(
                                    "observation_gap: changements Codex au-delà de 256 fichiers"
                                );
                            }
                        }
                    }
                    if interactive
                        && item.and_then(|i| i.get("type")).and_then(Value::as_str)
                            == Some("userMessage")
                        && let Some(client_id) =
                            item.and_then(|i| i.get("clientId")).and_then(Value::as_str)
                    {
                        let active = queue.0.lock().unwrap_or_else(|e| e.into_inner());
                        let mut detail = active_detail.lock().unwrap_or_else(|e| e.into_inner());
                        if let Some(detail) = detail.as_mut().filter(|d| {
                            source_matches_active_turn(d, &value)
                                && d.message_id == client_id
                                && !d.prompt_dispatched
                        }) && active
                            .active
                            .as_ref()
                            .is_some_and(|a| a.message_id == client_id)
                        {
                            detail.prompt_dispatched = true;
                            push_source(
                                &observations,
                                raw.clone(),
                                ManagedEventKind::PromptDispatched {
                                    message_id: client_id.into(),
                                },
                            );
                            record_or_terminal(
                                &journal,
                                &observations,
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
                            &journal,
                            &observations,
                            &active_detail,
                            &value,
                            sandbox_posture,
                            &mut sandbox_lines_by_item,
                            &mut refusals_recorded,
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
                        if let Some(turn_id) =
                            value.pointer("/params/turnId").and_then(Value::as_str)
                        {
                            let mut observed = observations
                                .0
                                .lock()
                                .unwrap_or_else(|poison| poison.into_inner());
                            let entry = observed
                                .response_by_turn
                                .entry(turn_id.to_string())
                                .or_default();
                            if entry.is_empty() {
                                entry.push_str(text);
                            }
                        }
                        maybe_record_final_agent_message(
                            &journal,
                            &observations,
                            &active_detail,
                            &value,
                            text,
                        );
                    }
                    push_source(
                        &observations,
                        raw,
                        ManagedEventKind::Update {
                            detail: "item/completed Codex".to_string(),
                        },
                    );
                }
                // Mesure 2026-08-27 (codex app-server 0.149) : la COMMANDE réelle
                // est dans item/started type=commandExecution.params.item.command.
                // Les outputDelta ne portent que la sortie — ce n'est pas le nom.
                Some("item/started") => {
                    let item = value.pointer("/params/item").unwrap_or(&Value::Null);
                    if interactive
                        && item.get("type").and_then(Value::as_str) == Some("userMessage")
                    {
                        let state = queue.0.lock().unwrap_or_else(|e| e.into_inner());
                        let client_id = item.get("clientId").and_then(Value::as_str).filter(|id| {
                            state.active.as_ref().is_some_and(|a| a.message_id == *id)
                        });
                        let mut active = active_detail.lock().unwrap_or_else(|e| e.into_inner());
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
                        .unwrap_or_else(|poison| poison.into_inner())
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
                            .unwrap_or_else(|poison| poison.into_inner())
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
                            .unwrap_or_else(|poison| poison.into_inner())
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
                        if interactive {
                            let mut state = queue.0.lock().unwrap_or_else(|e| e.into_inner());
                            if state.external_turn.as_deref() != Some(turn_id) {
                                // Un ancien terminal ou sa répétition ne peut
                                // ni clore le tour courant ni remplir la table.
                                continue;
                            }
                            if state.external_turn.as_deref() == Some(turn_id) {
                                let message_id = active_detail
                                    .lock()
                                    .unwrap_or_else(|e| e.into_inner())
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
                                    json!({"provider_turn_id": turn_id, "stop_reason": status}),
                                );
                                *active_detail.lock().unwrap_or_else(|e| e.into_inner()) = None;
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
                                        .unwrap_or_else(|e| e.into_inner())
                                        .response_by_turn
                                        .remove(turn_id);
                                    continue;
                                }
                            }
                        }
                        let terminal = match status {
                            "completed" => ManagedTerminal::Completed,
                            "interrupted" => ManagedTerminal::Cancelled,
                            other => ManagedTerminal::Failed {
                                detail: format!("terminal Codex: {other}"),
                            },
                        };
                        observations
                            .0
                            .lock()
                            .unwrap_or_else(|poison| poison.into_inner())
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
                            .unwrap_or_else(|poison| poison.into_inner())
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
        let pending =
            std::mem::take(&mut *waiters.lock().unwrap_or_else(|poison| poison.into_inner()));
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
    let mut pending = pending_request
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
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
    let mut pending = pending_request
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let message_id = if let Some(detail) = interactive_detail {
        detail
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
            .filter(|d| source_matches_active_turn(d, frame) && !d.message_id.is_empty())
            .map(|d| d.message_id.clone())
    } else {
        queue
            .0
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
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
        .unwrap_or_else(|poison| poison.into_inner())
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
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
    use std::time::Instant;

    static FIXTURE_SEQ: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn spec091_catalogue_muet_expire_sans_accumuler_de_reponses_en_attente() {
        let writer: Writer = Arc::new(Mutex::new(Some(Box::new(Vec::<u8>::new()))));
        let waiters: Waiters = Arc::new(Mutex::new(HashMap::new()));
        let next_id = Arc::new(AtomicU64::new(1));
        let start = Instant::now();
        for _ in 0..3 {
            assert!(
                request_with_timeout(
                    &writer,
                    &waiters,
                    &next_id,
                    "model/list",
                    json!({"limit":100}),
                    Duration::from_millis(1)
                )
                .is_err()
            );
            assert!(waiters.lock().unwrap().is_empty());
        }
        assert!(start.elapsed() < Duration::from_secs(1));
        assert_eq!(next_id.load(AtomicOrdering::SeqCst), 4);
    }

    #[test]
    fn catalogue_reprise_pagine_sans_ignorer_la_borne_ni_un_curseur_invalide() {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut count = 0;
        let result = collect_thread_pages(deadline, |params, remaining| {
            assert!(remaining <= Duration::from_secs(10));
            assert_eq!(params["limit"], 100);
            assert_eq!(params["cursor"], if count == 0 { Value::Null } else { json!(count.to_string()) });
            count += 1;
            Ok(json!({"data":[{"id":uuid::Uuid::new_v4().to_string(),"name":"N","cwd":"/tmp"}],"nextCursor": if count == 2 { Value::Null } else { json!(count.to_string()) }}))
        }).unwrap();
        assert_eq!(result.len(), 2);
        count = 0;
        // Mutation : retirer la borne permettrait une onzième page. Le
        // catalogue incomplet n'est jamais livré au sélecteur comme complet.
        assert!(collect_thread_pages(deadline, |_, _| {
            count += 1;
            assert!(count <= 10);
            Ok(json!({"data":(0..100).map(|_| json!({"id":uuid::Uuid::new_v4().to_string(),"name":"N","cwd":"/tmp"})).collect::<Vec<_>>(),"nextCursor":count.to_string()}))
        }).is_err());
        assert_eq!(count, 10);
        for invalid in [
            json!({"data":[]}),
            json!({"data":[],"nextCursor":1}),
            json!({"data":[],"nextCursor":"same"}),
        ] {
            assert!(collect_thread_pages(deadline, |_, _| Ok(invalid.clone())).is_err());
        }
        assert!(collect_thread_pages(Instant::now(), |_, _| panic!("RPC après échéance")).is_err());
    }

    #[test]
    fn spec090_catalogue_interactif_utilise_state_db_seul_a_chaque_page() {
        let deadline = Instant::now() + Duration::from_secs(10);
        let expected_ids = [uuid::Uuid::new_v4(), uuid::Uuid::new_v4()];
        let mut page = 0;

        let threads = collect_thread_pages(deadline, |params, remaining| {
            assert!(!remaining.is_zero());
            assert_eq!(
                params.get("useStateDbOnly"),
                Some(&Value::Bool(true)),
                "sans la projection state DB, Codex rescane et répare les rollouts avant de répondre"
            );
            assert_eq!(
                params.get("searchTerm"),
                None,
                "le menu et la détection des noms ambigus exigent toujours le catalogue complet"
            );
            assert_eq!(
                params["cursor"],
                if page == 0 {
                    Value::Null
                } else {
                    json!("page-2")
                }
            );
            let response = json!({
                "data": [{
                    "id": expected_ids[page].to_string(),
                    "name": "fil-exemple",
                    "cwd": "/Users/user/projets/autre-projet"
                }],
                "nextCursor": if page == 0 { json!("page-2") } else { Value::Null }
            });
            page += 1;
            Ok(response)
        })
        .expect("le catalogue public complet doit rester disponible sous la borne existante");

        assert_eq!(page, 2, "le drapeau doit accompagner toutes les pages");
        assert_eq!(
            threads.len(),
            2,
            "les doublons de nom restent visibles au sélecteur"
        );
        assert_eq!(threads[0].name.as_deref(), Some("fil-exemple"));
        assert_eq!(threads[1].name.as_deref(), Some("fil-exemple"));
    }

    #[test]
    fn lecteur_interactif_ne_repond_jamais_pour_humain_et_conserve_raw() {
        struct Capture(Arc<Mutex<Vec<u8>>>);
        impl Write for Capture {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let output = Arc::new(Mutex::new(Vec::new()));
        let observations = Arc::new((Mutex::new(Observations::default()), Condvar::new()));
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
        let alive = Arc::new(AtomicBool::new(true));
        let detail = Arc::new(Mutex::new(None));
        let pending_request = Arc::new(Mutex::new(None));
        let journal_root = root("interactive-reader");
        let journal = Arc::new(Mutex::new(Some(
            JournalWriter::start(
                &journal_root,
                "interactive",
                "thread",
                Arc::new(Mutex::new(crate::acp::AcpEventQueue::default())),
            )
            .unwrap(),
        )));
        let (tx, rx) = mpsc::channel::<std::io::Result<String>>();
        let waiters = Arc::new(Mutex::new(HashMap::new()));
        let reader = spawn_reader(
            Box::new(rx.into_iter()),
            ReaderContext {
                waiters: waiters.clone(),
                cwd: "/tmp".into(),
                observations: observations.clone(),
                alive: alive.clone(),
                journal: journal.clone(),
                queue: queue.clone(),
                pending_request: pending_request.clone(),
                pinned_model: None,
                active_detail: detail.clone(),
                writer: Arc::new(Mutex::new(Some(Box::new(Capture(output.clone()))))),
                permissions: "allow".into(),
                dynamic_tool_handler: None,
                sandbox_posture: "complete",
                interactive: true,
                selected_thread: Arc::new(Mutex::new(Some("thread".into()))),
            },
        );
        let serial = AtomicU64::new(0);
        let barrier = |raw: &str| {
            tx.send(Ok(raw.into())).unwrap();
            let sentinel = json!({"method": "test/barrier", "params": {"n": serial.fetch_add(1, AtomicOrdering::SeqCst)}}).to_string();
            tx.send(Ok(sentinel.clone())).unwrap();
            let deadline = Instant::now() + Duration::from_secs(3);
            let mut state = observations.0.lock().unwrap();
            while !state
                .events
                .iter()
                .any(|event| event.raw == sentinel.as_bytes())
            {
                let remaining = deadline
                    .checked_duration_since(Instant::now())
                    .expect("événement reader attendu");
                state = observations.1.wait_timeout(state, remaining).unwrap().0;
            }
        };
        barrier(
            r#"{ "method":"turn/started", "params":{"threadId":"thread","turn":{"id":"t","status":"inProgress"}} }"#,
        );
        assert_eq!(queue.0.lock().unwrap().external_turn.as_deref(), Some("t"));
        // SPEC100 : le vrai lecteur ne publie qu'une réussite structurée,
        // ignore échec, doublon et ancien tour (aucun processus fournisseur).
        for (turn, id, status) in [
            ("old", "late", "completed"),
            ("t", "failed", "failed"),
            ("t", "write", "completed"),
            ("t", "write", "completed"),
        ] {
            barrier(&json!({"method":"item/completed","params":{"threadId":"thread","turnId":turn,
                "item":{"type":"fileChange","id":id,"status":status,"changes":[{"path":"/project/spec100.rs"}]}}}).to_string());
        }
        for raw in [
            r#"{ "id":7, "method":"item/commandExecution/requestApproval", "params":{"threadId":"thread","turnId":"t","future":42} }"#,
            r#"{ "id":"eight", "method":"mcpServer/elicitation/request", "params":{"serverName":"bridget","mode":"form","requestedSchema":{}} }"#,
            r#"{  "id":9, "method":"future/request", "params": {"unknown": true}  }"#,
        ] {
            barrier(raw);
        }
        // Mutation : appeler les réponses automatiques managed (allow), y
        // compris celle de refus inconnu, rendrait cette assertion rouge.
        assert!(output.lock().unwrap().is_empty());
        barrier(
            r#"{"method":"turn/completed","params":{"threadId":"thread","turn":{"id":"t","status":"completed"}}}"#,
        );
        assert!(queue.0.lock().unwrap().external_turn.is_none());
        let state = observations.0.lock().unwrap();
        assert!(state.events.iter().any(|e| matches!(
            e.kind,
            ManagedEventKind::ActivityObserved { in_progress: true }
        )));
        assert!(state.events.iter().any(|e| matches!(
            e.kind,
            ManagedEventKind::ActivityObserved { in_progress: false }
        )));
        assert!(
            !state
                .events
                .iter()
                .any(|e| matches!(e.kind, ManagedEventKind::PromptDispatched { .. }))
        );
        assert!(state.terminal_by_turn.is_empty());
        drop(state);

        // Mutation : tester queue.active.is_none() au lieu du tour observé
        // perd le tour humain arrivé pendant une réservation Bridget.
        let (cancel, _receiver) = mpsc::channel();
        queue.0.lock().unwrap().active = Some(ActiveTurn {
            message_id: "pending".into(),
            cancel,
        });
        barrier(
            r#"{"method":"turn/started","params":{"threadId":"thread","turn":{"id":"human-race","status":"inProgress"}}}"#,
        );
        barrier(
            r#"{"method":"item/started","params":{"threadId":"thread","turnId":"human-race","item":{"type":"userMessage","clientId":"human","content":[]}}}"#,
        );
        assert_eq!(
            queue.0.lock().unwrap().external_turn.as_deref(),
            Some("human-race")
        );
        assert_eq!(detail.lock().unwrap().as_ref().unwrap().message_id, "");
        barrier(
            r#"{"id":10,"method":"item/commandExecution/requestApproval","params":{"threadId":"thread","turnId":"human-race"}}"#,
        );
        assert!(
            pending_request
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .message_id
                .is_none(),
            "la réservation n'est pas l'auteur de la permission humaine"
        );
        assert!(!source_matches_active_turn(
            detail.lock().unwrap().as_ref().unwrap(),
            &json!({"params":{"threadId":"thread","turnId":"different"}})
        ));
        barrier(
            r#"{"method":"turn/completed","params":{"threadId":"thread","turn":{"id":"human-race","status":"completed"}}}"#,
        );
        barrier(
            r#"{"method":"turn/started","params":{"threadId":"thread","turn":{"id":"owned","status":"inProgress"}}}"#,
        );
        barrier(
            r#"{"method":"item/started","params":{"threadId":"thread","turnId":"owned","item":{"type":"userMessage","clientId":"pending","content":[]}}}"#,
        );
        assert_eq!(
            detail.lock().unwrap().as_ref().unwrap().message_id,
            "pending"
        );
        assert!(
            !observations
                .0
                .lock()
                .unwrap()
                .events
                .iter()
                .any(|e| matches!(e.kind, ManagedEventKind::PromptDispatched { .. })),
            "pas d'ACK avant consommation"
        );
        barrier(
            r#"{"method":"item/completed","params":{"threadId":"thread","turnId":"ancien","item":{"type":"userMessage","clientId":"pending","content":[]}}}"#,
        );
        assert!(
            !observations
                .0
                .lock()
                .unwrap()
                .events
                .iter()
                .any(|e| matches!(e.kind, ManagedEventKind::PromptDispatched { .. })),
            "ancien tour : aucun ACK malgré le bon clientId"
        );
        barrier(
            r#"{"method":"item/completed","params":{"threadId":"thread","turnId":"owned","item":{"type":"userMessage","clientId":"pending","content":[]}}}"#,
        );
        barrier(
            r#"{"method":"item/completed","params":{"threadId":"thread","turnId":"owned","item":{"type":"userMessage","clientId":"pending","content":[]}}}"#,
        );
        assert_eq!(observations.0.lock().unwrap().events.iter().filter(|e| matches!(&e.kind, ManagedEventKind::PromptDispatched { message_id } if message_id == "pending")).count(), 1);
        barrier(
            r#"{"method":"item/started","params":{"threadId":"thread","turnId":"owned","item":{"type":"userMessage","clientId":"human-steer","content":[{"type":"text","text":"AJOUT HUMAIN"}]}}}"#,
        );
        assert_eq!(
            detail.lock().unwrap().as_ref().unwrap().message_id,
            "pending",
            "l'ajout humain ne vole pas la corrélation du tour"
        );
        barrier(
            r#"{"method":"turn/completed","params":{"threadId":"thread","turn":{"id":"owned","status":"completed"}}}"#,
        );
        assert!(
            observations
                .0
                .lock()
                .unwrap()
                .terminal_by_turn
                .contains_key("owned"),
            "le worker doit recevoir son terminal malgré l'ajout humain"
        );
        // Le worker n'a PAS encore libéré sa réservation. Le lecteur doit
        // pourtant attester le tour suivant et clear_active ne peut l'effacer.
        barrier(
            r#"{"method":"turn/started","params":{"threadId":"thread","turn":{"id":"next-human","status":"inProgress"}}}"#,
        );
        clear_active(&queue);
        assert_eq!(
            queue.0.lock().unwrap().external_turn.as_deref(),
            Some("next-human")
        );
        assert_eq!(
            detail.lock().unwrap().as_ref().unwrap().turn_id.as_deref(),
            Some("next-human")
        );
        barrier(
            r#"{"method":"turn/completed","params":{"threadId":"thread","turn":{"id":"ancien","status":"completed"}}}"#,
        );
        assert!(
            !observations
                .0
                .lock()
                .unwrap()
                .terminal_by_turn
                .contains_key("ancien")
        );
        assert_eq!(
            queue.0.lock().unwrap().external_turn.as_deref(),
            Some("next-human")
        );
        let (result_tx, result_rx) = mpsc::channel();
        waiters.lock().unwrap().insert(999, result_tx);
        tx.send(Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Space limit exceeded: message too long",
        )))
        .unwrap();
        // Une erreur de lecture ne doit plus se transformer en un faux EOF.
        let error = result_rx
            .recv_timeout(Duration::from_secs(3))
            .unwrap()
            .unwrap_err();
        assert!(
            error.contains("InvalidData") && error.contains("message too long"),
            "{error}"
        );
        assert!(!error.contains("stdout Codex fermé"));
        drop(tx);
        reader.join().unwrap();
        assert!(!alive.load(Ordering::SeqCst));
        assert!(
            !observations
                .0
                .lock()
                .unwrap()
                .events
                .iter()
                .any(|event| matches!(event.kind, ManagedEventKind::JournalFailed { .. })),
            "une saisie supplémentaire doit traverser la vraie garde du journal"
        );
        journal.lock().unwrap().take().unwrap().stop();
        let logs: String = fs::read_dir(journal_root.join("interactive"))
            .unwrap()
            .map(|entry| fs::read_to_string(entry.unwrap().path()).unwrap())
            .collect();
        assert_eq!(
            logs.lines()
                .map(|line| serde_json::from_str::<Value>(line).unwrap())
                .filter(|entry| entry["payload"]["write_confirmed"] == true)
                .count(),
            1
        );
        assert!(
            logs.lines()
                .map(|line| serde_json::from_str::<Value>(line).unwrap())
                .any(|entry| entry["event"] == "user_message"
                    && entry["payload"]["body"] == "AJOUT HUMAIN"
                    && entry["payload"]["from"] == "human")
        );
    }

    #[test]
    fn spec093_lecteur_interactif_ignore_les_fils_codex_tiers_sans_fermer_le_parent() {
        struct Capture(Arc<Mutex<Vec<u8>>>);
        impl Write for Capture {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let output = Arc::new(Mutex::new(Vec::new()));
        let observations = Arc::new((Mutex::new(Observations::default()), Condvar::new()));
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
        let alive = Arc::new(AtomicBool::new(true));
        let detail = Arc::new(Mutex::new(None));
        let pending_request = Arc::new(Mutex::new(None));
        let journal_root = root("foreign-codex-threads");
        let journal = Arc::new(Mutex::new(Some(
            JournalWriter::start(
                &journal_root,
                "interactive",
                "parent-thread",
                Arc::new(Mutex::new(crate::acp::AcpEventQueue::default())),
            )
            .unwrap(),
        )));
        let (tx, rx) = mpsc::channel::<std::io::Result<String>>();
        let waiters = Arc::new(Mutex::new(HashMap::new()));
        let reader = spawn_reader(
            Box::new(rx.into_iter()),
            ReaderContext {
                waiters: waiters.clone(),
                cwd: "/tmp".into(),
                observations: observations.clone(),
                alive: alive.clone(),
                journal: journal.clone(),
                queue: queue.clone(),
                pending_request: pending_request.clone(),
                pinned_model: None,
                active_detail: detail.clone(),
                writer: Arc::new(Mutex::new(Some(Box::new(Capture(output.clone()))))),
                permissions: "allow".into(),
                dynamic_tool_handler: None,
                sandbox_posture: "complete",
                interactive: true,
                selected_thread: Arc::new(Mutex::new(Some("parent-thread".into()))),
            },
        );
        let send_and_wait = |raw: &str| {
            tx.send(Ok(raw.into())).unwrap();
            let deadline = Instant::now() + Duration::from_secs(3);
            let mut state = observations.0.lock().unwrap();
            while alive.load(Ordering::SeqCst)
                && !state.events.iter().any(|event| event.raw == raw.as_bytes())
            {
                let remaining = deadline
                    .checked_duration_since(Instant::now())
                    .expect("événement parent attendu avant échéance");
                state = observations.1.wait_timeout(state, remaining).unwrap().0;
            }
            assert!(
                state.events.iter().any(|event| event.raw == raw.as_bytes()),
                "le reader doit traiter l'événement parent suivant"
            );
        };

        send_and_wait(
            r#"{"method":"turn/started","params":{"threadId":"parent-thread","turn":{"id":"parent-turn","status":"inProgress"}}}"#,
        );
        send_and_wait(
            r#"{"method":"item/started","params":{"threadId":"parent-thread","turnId":"parent-turn","item":{"type":"userMessage","content":[{"type":"text","text":"MESSAGE PARENT"}]}}}"#,
        );
        for raw in [
            r#"{"method":"thread/started","params":{"thread":{"id":"child-one","parentThreadId":"parent-thread","status":{"type":"active"}}}}"#,
            r#"{"method":"turn/started","params":{"threadId":"child-one","turn":{"id":"child-turn","status":"inProgress"}}}"#,
            r#"{"method":"thread/status/changed","params":{"threadId":"child-one","status":{"type":"idle"}}}"#,
            r#"{"method":"thread/status/changed","params":{"threadId":"resumed-child","status":{"type":"active"}}}"#,
            r#"{"method":"item/agentMessage/delta","params":{"threadId":"child-one","turnId":"child-turn","itemId":"child-answer","delta":"NE PAS ATTRIBUER"}}"#,
            r#"{"id":71,"method":"item/permissions/requestApproval","params":{"threadId":"child-one","turnId":"child-turn","permissions":{}}}"#,
        ] {
            tx.send(Ok(raw.into())).unwrap();
        }
        send_and_wait(
            r#"{"method":"item/agentMessage/delta","params":{"threadId":"parent-thread","turnId":"parent-turn","itemId":"parent-answer","delta":"PARENT TOUJOURS ACTIF"}}"#,
        );

        assert!(alive.load(Ordering::SeqCst));
        assert_eq!(
            queue.0.lock().unwrap().external_turn.as_deref(),
            Some("parent-turn")
        );
        {
            let active = detail.lock().unwrap();
            let active = active.as_ref().expect("tour parent conservé");
            assert_eq!(active.thread_id, "parent-thread");
            assert_eq!(active.turn_id.as_deref(), Some("parent-turn"));
        }
        assert!(pending_request.lock().unwrap().is_none());
        assert!(output.lock().unwrap().is_empty());
        assert_eq!(
            observations
                .0
                .lock()
                .unwrap()
                .response_by_turn
                .get("parent-turn")
                .map(String::as_str),
            Some("PARENT TOUJOURS ACTIF")
        );
        assert!(
            !observations
                .0
                .lock()
                .unwrap()
                .response_by_turn
                .contains_key("child-turn")
        );

        let (result_tx, result_rx) = mpsc::channel();
        waiters.lock().unwrap().insert(993, result_tx);
        drop(tx);
        reader.join().unwrap();
        assert!(!alive.load(Ordering::SeqCst));
        assert_eq!(
            result_rx
                .recv_timeout(Duration::from_secs(3))
                .unwrap()
                .unwrap_err(),
            "flux Codex fermé"
        );

        journal.lock().unwrap().take().unwrap().stop();
        let logs: String = fs::read_dir(journal_root.join("interactive"))
            .unwrap()
            .map(|entry| fs::read_to_string(entry.unwrap().path()).unwrap())
            .collect();
        assert!(logs.contains("parent-turn"));
        assert!(!logs.contains("child-one"));
        assert!(!logs.contains("resumed-child"));
    }

    fn root(label: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "bridget-codex-native-{label}-{}-{}",
            std::process::id(),
            FIXTURE_SEQ.fetch_add(1, AtomicOrdering::Relaxed)
        ));
        fs::create_dir_all(&root).expect("racine temporaire");

        root
    }

    #[test]
    fn requete_fournisseur_inconnue_recoit_un_refus_json_rpc_redacte() {
        let sentinel = "SENTINELLE-METHODE-INCONNUE";
        let (reply, diagnostic, delegated_reference) =
            unsupported_provider_request_response(&json!({
                "id": "dynamic-tool-42",
                "method": sentinel,
                "params": { "secret": "ne-pas-publier" },
            }))
            .expect("réponse aux requêtes avec id");
        assert_eq!(reply["id"], "dynamic-tool-42");
        assert_eq!(reply["error"]["code"], -32601);
        assert_eq!(
            reply["error"]["data"]["code"],
            "unsupported_provider_request"
        );
        assert_eq!(diagnostic["code"], "unsupported_provider_request");
        let serialized =
            serde_json::to_string(&json!({ "reply": reply, "diagnostic": diagnostic }))
                .expect("diagnostic sérialisable");
        assert!(!serialized.contains(sentinel));
        assert!(!serialized.contains("ne-pas-publier"));
        assert!(serialized.contains("sha256:"));
        assert!(delegated_reference.starts_with("sha256:"));
        assert_ne!(delegated_reference, sentinel);
        assert!(!delegated_reference.contains("commandExecution"));
        assert!(!delegated_reference.contains("ne-pas-publier"));
    }

    #[test]
    fn elicitation_mcp_vide_de_bridget_est_acceptee_et_un_formulaire_est_refuse() {
        let (reply, payload) = mcp_elicitation_response(&json!({
            "id": "elicitation-vide",
            "method": "mcpServer/elicitation/request",
            "params": {
                "serverName": "bridget",
                "mode": "form",
                "message": "ne pas enregistrer ce texte",
                "requestedSchema": {"type": "object", "properties": {}, "required": []}
            }
        }))
        .expect("élicitation MCP valide");
        assert_eq!(reply["id"], "elicitation-vide");
        assert_eq!(reply["result"]["action"], "accept");
        assert_eq!(reply["result"]["content"], json!({}));
        assert_eq!(payload["code"], "mcp_elicitation_autoaccepted");
        assert!(
            !serde_json::to_string(&payload)
                .expect("payload sérialisable")
                .contains("ne pas enregistrer ce texte")
        );

        let (reply, payload) = mcp_elicitation_response(&json!({
            "id": "elicitation-secret",
            "method": "mcpServer/elicitation/request",
            "params": {
                "serverName": "bridget",
                "mode": "form",
                "requestedSchema": {
                    "type": "object",
                    "properties": {"secret": {"type": "string"}},
                    "required": ["secret"]
                }
            }
        }))
        .expect("élicitation MCP valide");
        assert_eq!(reply["result"]["action"], "decline");
        assert_eq!(reply["result"].get("content"), None);
        assert_eq!(payload["code"], "mcp_elicitation_declined");
        assert_eq!(payload["required_field_count"], 1);
    }

    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_codex_refus_update_hors_vocabulaire_emet_JournalFailed() {
        let root = root("act-kind-visible");
        let observations = Arc::new((Mutex::new(Observations::default()), Condvar::new()));
        let journal = Arc::new(Mutex::new(Some(
            JournalWriter::start(
                &root,
                "codex-1",
                "session-1",
                Arc::new(Mutex::new(crate::acp::AcpEventQueue::default())),
            )
            .unwrap(),
        )));
        record_or_terminal(
            &journal,
            &observations,
            "update",
            Some("m1"),
            json!({"kind":"command","text":"ls"}),
        );
        assert!(
            observations
                .0
                .lock()
                .unwrap()
                .events
                .iter()
                .all(|event| !matches!(event.kind, ManagedEventKind::JournalFailed { .. })),
            "écriture command valide ne doit pas émettre JournalFailed"
        );
        record_or_terminal(
            &journal,
            &observations,
            "update",
            Some("m1"),
            json!({"kind":"intent","text":"fantôme"}),
        );
        let drained: Vec<_> = observations.0.lock().unwrap().events.drain(..).collect();
        assert!(
            drained.iter().any(|event| matches!(
                &event.kind,
                ManagedEventKind::JournalFailed { detail }
                    if detail.contains("hors vocabulaire")
            )),
            "refus hors vocabulaire doit être visible via JournalFailed, got {drained:?}"
        );
        if let Some(writer) = journal.lock().unwrap().take() {
            writer.stop();
        }
        let _ = fs::remove_dir_all(root);
    }

    /// Dette 3 — oracle bout-en-bout Codex : commandExecution → record_active_act
    /// avec kind interdit → JournalFailed drainé. Preuve d'abord qu'un cas valide passe.
    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_codex_pilote_kind_interdit_emet_JournalFailed_bout_en_bout() {
        let (events_ok, journal_ok) = {
            let _suite = crate::act_kind::pilot_kind_suite_lock();
            codex_activity_turn_managed_events("e2e-ok")
        };
        assert!(
            events_ok
                .iter()
                .all(|event| { !matches!(event.kind, ManagedEventKind::JournalFailed { .. }) }),
            "cas valide ne doit pas émettre JournalFailed, got {events_ok:?}"
        );
        assert!(
            journal_ok.iter().any(|event| {
                event["event"] == "update" && event["payload"]["kind"] == "command"
            }),
            "cas valide doit journaliser kind=command, got {journal_ok:?}"
        );

        let (events_bad, journal_bad) =
            crate::act_kind::with_forced_pilot_update_kind("intent", || {
                codex_activity_turn_managed_events("e2e-bad")
            });
        assert!(
            events_bad.iter().any(|event| matches!(
                &event.kind,
                ManagedEventKind::JournalFailed { detail }
                    if detail.contains("hors vocabulaire")
            )),
            "kind interdit du pilote doit être VISIBLE via JournalFailed, got {events_bad:?}"
        );
        assert!(
            journal_bad
                .iter()
                .all(|event| event["payload"]["kind"] != "intent"),
            "le kind refusé ne doit jamais atteindre le JSONL, got {journal_bad:?}"
        );
    }

    fn codex_activity_turn_managed_events(label: &str) -> (Vec<ManagedEvent>, Vec<Value>) {
        codex_activity_turn_managed_events_with(label, &[], None)
    }

    /// Variante paramétrée : variables d'environnement supplémentaires pour la
    /// fixture, et arguments de lancement (posture SPEC-088).
    fn codex_activity_turn_managed_events_with(
        label: &str,
        extra_environment: &[(&str, &str)],
        args: Option<Vec<String>>,
    ) -> (Vec<ManagedEvent>, Vec<Value>) {
        let root = root(label);
        let trace = root.join("trace.jsonl");
        let mut environment = vec![
            (
                "BRIDGET_CODEX_TRACE".to_string(),
                trace.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_ACTIVITY".to_string(), "1".to_string()),
        ];
        environment.extend(
            extra_environment
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string())),
        );
        let mut options = fake_options(&trace);
        if let Some(args) = args {
            options.args = args;
        }
        let mut transport =
            CodexAppServerTransport::spawn_with_environment(options, &environment, false)
                .expect("session native e2e");
        transport
            .activate_journal(&root, "codex-native", None)
            .expect("journal e2e");
        transport.deliver(&message(label)).expect("livraison e2e");
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events.iter().any(|event| {
                matches!(
                    event.kind,
                    ManagedEventKind::TurnFinished {
                        terminal: ManagedTerminal::Completed,
                        ..
                    }
                )
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        events.extend(transport.drain_events());
        transport.stop();
        let journal_path = fs::read_dir(root.join("codex-native"))
            .expect("répertoire journal")
            .next()
            .expect("fichier journal")
            .expect("entrée journal")
            .path();
        let journal_events = crate::journal::valid_events(&journal_path);
        let _ = fs::remove_dir_all(&root);
        (events, journal_events)
    }

    /// SPEC-088 T006 révisé : ligne de sandbox + fin en ÉCHEC du même item ⇒
    /// UN acte `refusal` (signalement non attesté) ; ligne + fin réussie ⇒
    /// aucun ; fin en échec sans ligne ⇒ aucun refus mais l'acte `command`
    /// porte l'échec. Contrôle positif : la posture est lue des arguments.
    #[test]
    fn spec_088_signalement_de_sandbox_exige_la_ligne_et_l_echec_du_meme_item() {
        let _suite = crate::act_kind::pilot_kind_suite_lock();
        let discovery_args = || {
            let mut args = fake_options(Path::new("/tmp/unused")).args;
            // Après le script `sh -c` : paramètres positionnels inoffensifs,
            // mais visibles pour la lecture de posture.
            args.extend(["--sandbox".to_string(), "read-only".to_string()]);
            args
        };
        let refusals_of = |journal: &[Value]| -> Vec<Value> {
            journal
                .iter()
                .filter(|event| event["event"] == "update" && event["payload"]["kind"] == "refusal")
                .cloned()
                .collect()
        };

        // 1. Ligne bwrap (deux fois) + fin en échec ⇒ un seul acte.
        let (_events, journal) = codex_activity_turn_managed_events_with(
            "spec-088-line-failed",
            &[
                ("BRIDGET_CODEX_SANDBOX_REFUSAL", "1"),
                ("BRIDGET_CODEX_COMMAND_COMPLETION", "failed"),
            ],
            Some(discovery_args()),
        );
        let refusals = refusals_of(&journal);
        assert_eq!(refusals.len(), 1, "un seul signalement, got {journal:?}");
        let payload = &refusals[0]["payload"];
        assert_eq!(payload["layer"], "provider_sandbox");
        assert_eq!(payload["evidence"], "output_and_exit");
        assert_eq!(payload["provider"], "codex");
        assert_eq!(payload["posture"], "discovery");
        assert_eq!(payload["item_id"], "command-1");
        assert_eq!(payload["exit_code"], 1);
        assert_eq!(payload["attributed_to"], "bridget");
        assert_eq!(payload["gesture"]["target"], "shell");
        assert!(
            journal.iter().any(|event| {
                event["payload"]["kind"] == "command"
                    && event["payload"]["state"] == "failed"
                    && event["payload"]["item_id"] == "command-1"
            }),
            "la fin de commande est journalisée en échec, got {journal:?}"
        );

        // 2. Ligne bwrap + fin RÉUSSIE ⇒ aucun signalement (un `echo` ne prouve rien).
        let (_events, journal_ok) = codex_activity_turn_managed_events_with(
            "spec-088-line-ok",
            &[
                ("BRIDGET_CODEX_SANDBOX_REFUSAL", "1"),
                ("BRIDGET_CODEX_COMMAND_COMPLETION", "completed"),
            ],
            Some(discovery_args()),
        );
        assert!(
            refusals_of(&journal_ok).is_empty(),
            "sortie 0 ⇒ aucun refus, got {journal_ok:?}"
        );
        assert!(
            journal_ok.iter().any(|event| {
                event["payload"]["kind"] == "command"
                    && event["payload"]["state"] == "completed"
                    && event["payload"]["exit_code"] == 0
                    && event["payload"]["output_tail"]
                        .as_str()
                        .is_some_and(|tail| tail.contains("BRIDGET-TEST-fixture"))
            }),
            "la fin réussie porte le code 0 et la queue de sortie, got {journal_ok:?}"
        );

        // 3. Fin en échec SANS ligne reconnue ⇒ aucun refus.
        let (_events, journal_failed_only) = codex_activity_turn_managed_events_with(
            "spec-088-failed-only",
            &[("BRIDGET_CODEX_COMMAND_COMPLETION", "failed")],
            None,
        );
        assert!(
            refusals_of(&journal_failed_only).is_empty(),
            "échec sans motif ⇒ aucun refus"
        );

        // 4. Ligne sans aucune fin ⇒ rien (une ligne seule n'est pas une preuve).
        let (_events, journal_line_only) = codex_activity_turn_managed_events_with(
            "spec-088-line-only",
            &[("BRIDGET_CODEX_SANDBOX_REFUSAL", "1")],
            None,
        );
        assert!(
            refusals_of(&journal_line_only).is_empty(),
            "ligne seule ⇒ aucun refus"
        );
        assert_eq!(
            sandbox_posture_from_args(&fake_options(Path::new("/tmp/x")).args),
            "complete"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn mutant_avale_err_record_tue_TEMOIN_codex_JournalFailed() {
        fn broken_swallow(
            journal: &Journal,
            _observations: &Arc<(Mutex<Observations>, Condvar)>,
            event: &str,
            message_id: Option<&str>,
            payload: Value,
        ) {
            let _ = record(journal, event, message_id, payload);
        }
        let root = root("act-kind-mutant");
        let observations = Arc::new((Mutex::new(Observations::default()), Condvar::new()));
        let journal = Arc::new(Mutex::new(Some(
            JournalWriter::start(
                &root,
                "codex-1",
                "session-1",
                Arc::new(Mutex::new(crate::acp::AcpEventQueue::default())),
            )
            .unwrap(),
        )));
        broken_swallow(
            &journal,
            &observations,
            "update",
            Some("m1"),
            json!({"kind":"intent"}),
        );
        assert!(
            observations.0.lock().unwrap().events.is_empty(),
            "le mutant avale l'erreur"
        );
        record_or_terminal(
            &journal,
            &observations,
            "update",
            Some("m1"),
            json!({"kind":"intent"}),
        );
        assert!(
            observations
                .0
                .lock()
                .unwrap()
                .events
                .iter()
                .any(|event| matches!(
                    &event.kind,
                    ManagedEventKind::JournalFailed { detail }
                        if detail.contains("hors vocabulaire")
                )),
            "TEMOIN_codex_JournalFailed doit mourir si Err est avalée"
        );
        if let Some(writer) = journal.lock().unwrap().take() {
            writer.stop();
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn spec091_selection_reelle_du_prochain_tour_sans_prompt_cache_ni_perte_du_fil() {
        use crate::protocol::{
            RuntimeSelection, RuntimeSelectionOutcome as Outcome,
            RuntimeSelectionRefusal as Refusal,
        };
        let root = root("runtime091");
        let trace = root.join("wire.jsonl");
        let mut options = fake_options(&trace);
        options.command = "/usr/bin/python3".into();
        options.args = vec!["-u".into(), "-c".into(), r#"
import json, sys
settings = {'model':'first','effort':'low'}
for line in sys.stdin:
    with open(sys.argv[1], 'a') as trace: trace.write(line)
    value = json.loads(line); method = value.get('method'); params = value.get('params', {})
    result = {}
    if method == 'initialize': result = {'userAgent':'fixture091','codexHome':'/tmp','platformFamily':'unix','platformOs':'macos'}
    if method == 'thread/start': result = {'thread':{'id':'same-thread'},'model':'first','reasoningEffort':'low'}
    if method == 'model/list': result = {'data':[{'model':m,'supportedReasoningEfforts':[{'reasoningEffort':e} for e in ['low','high']]} for m in ['first','second']], 'nextCursor':None}
    if method == 'thread/settings/update': settings = {'model':params['model'],'effort':params['effort']}
    if method == 'turn/start': result = {'turn':{'id':str(value['id'])}}
    if 'id' in value: print(json.dumps({'id':value['id'],'result':result}), flush=True)
    if method == 'thread/settings/update':
        print(json.dumps({'method':'thread/settings/updated','params':{'threadId':params['threadId'],'threadSettings':settings}}), flush=True)
    if method == 'turn/start':
        print(json.dumps({'method':'turn/completed','params':{'threadId':params['threadId'],'turn':{'id':str(value['id']),'status':'completed'}}}), flush=True)
"#.into(), trace.to_string_lossy().into_owned()];
        let mut session = CodexAppServerTransport::spawn(options).unwrap();
        let requested = RuntimeSelection {
            model: "second".into(),
            effort: "high".into(),
        };
        assert_eq!(
            session.select_runtime(requested.clone()),
            Outcome::Selected {
                selection: requested
            }
        );
        let before = fs::read_to_string(&trace).unwrap();
        assert!(
            !before.contains("turn/start"),
            "la sélection ne doit pas lancer de prompt"
        );
        assert_eq!(
            session.select_runtime(RuntimeSelection {
                model: "absent".into(),
                effort: "high".into()
            }),
            Outcome::Refused {
                reason: Refusal::ModelUnavailable
            }
        );
        assert_eq!(
            session.select_runtime(RuntimeSelection {
                model: "first".into(),
                effort: "invented".into()
            }),
            Outcome::Refused {
                reason: Refusal::EffortUnavailable
            }
        );
        for (id, model, effort) in [("change-1", "second", "high"), ("change-2", "first", "low")] {
            if id == "change-2" {
                assert!(matches!(
                    session.select_runtime(RuntimeSelection {
                        model: model.into(),
                        effort: effort.into()
                    }),
                    Outcome::Selected { .. }
                ));
            }
            session.deliver(&message(id)).unwrap();
            let deadline = Instant::now() + Duration::from_secs(3);
            let mut finished = false;
            let mut observed = false;
            while Instant::now() < deadline && !(finished && observed) {
                for event in session.drain_events() {
                    if let ManagedEventKind::RuntimeObserved {
                        model: actual,
                        effort: actual_effort,
                    } = &event.kind
                        && actual == model
                        && actual_effort.as_deref() == Some(effort)
                    {
                        assert_eq!(
                            event.origin,
                            crate::managed_session::ManagedEventOrigin::SourceLine
                        );
                        assert!(
                            String::from_utf8_lossy(&event.raw).contains("thread/settings/updated")
                        );
                        observed = true;
                    }
                    finished |= matches!(event.kind, ManagedEventKind::TurnFinished { .. });
                }
                thread::sleep(Duration::from_millis(5));
            }
            assert!(
                finished && observed,
                "tour et réglages doivent être attestés"
            );
        }
        session.stop();
        let frames: Vec<Value> = fs::read_to_string(trace)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let turns: Vec<_> = frames
            .iter()
            .filter(|frame| frame["method"] == "turn/start")
            .collect();
        assert_eq!(turns.len(), 2);
        let updates: Vec<_> = frames
            .iter()
            .filter(|frame| frame["method"] == "thread/settings/update")
            .collect();
        assert_eq!(updates.len(), 2);
        for (update, model, effort) in
            [(updates[0], "second", "high"), (updates[1], "first", "low")]
        {
            assert_eq!(
                update["params"],
                json!({"threadId":"same-thread","model":model,"effort":effort})
            );
        }
        assert!(
            turns
                .iter()
                .all(|turn| turn["params"]["threadId"] == "same-thread"
                    && turn["params"].get("model").is_none())
        );
        // Mutation : retirer la commande native settings/update casse cet oracle ;
        // remplacer RuntimeObserved par une déclaration synthétique le casse aussi.
    }

    fn fake_options(_trace: &std::path::Path) -> CodexAppServerOptions {
        CodexAppServerOptions {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                r#"started=0; saturated=0; while IFS= read -r line; do
                    printf '%s\n' "$line" >> "$BRIDGET_CODEX_TRACE"
                    case "$line" in
                        *'"method":"initialize"'*) if [ -n "${BRIDGET_CODEX_CHILD_PID:-}" ]; then sleep 60 & printf '%s' "$!" > "$BRIDGET_CODEX_CHILD_PID"; fi; printf '%s\n' '{"id":1,"result":{"userAgent":"fake","codexHome":"/tmp","platformFamily":"unix","platformOs":"macos"}}' ;;
                        *'"method":"initialized"'*) started=1 ;;
                        *'"method":"thread/start"'*) printf '%s\n' '{"id":2,"result":{"thread":{"id":"thread-native"},"model":"gpt-5.6-terra","reasoningEffort":"high"}}' ;;
                        *'"method":"account/rateLimits/read"'*) printf '%s\n' '{"id":3,"result":{"rateLimits":{"primary":{"usedPercent":42,"windowDurationMins":300,"resetsAt":1787572200},"rateLimitReachedType":null}}}' ;;
                        *'"method":"turn/start"'*)
                            if [ "$started" != 1 ]; then printf '%s\n' '{"id":4,"error":{"code":-32099,"message":"initialized absent"}}'
                            elif [ "$saturated" = 0 ]; then saturated=1; printf '%s\n' '{"id":4,"error":{"code":-32001,"message":"saturated"}}'
                            elif [ "${BRIDGET_CODEX_TURN_ERROR:-0}" = 1 ]; then malicious_reason='{"id":5,"error":{"code":-32098,"message":"SENTINELLE-RAISON-019\u001b[2J\r\u202e"}}'; printf '%s\n' "$malicious_reason" >> "$BRIDGET_CODEX_OUTPUT_TRACE"; printf '%s\n' "$malicious_reason"
                            else
                                printf '%s\n' '{"id":5,"result":{"turn":{"id":"turn-native"}}}'
                                if [ "${BRIDGET_CODEX_REQUEST_APPROVAL:-0}" = 1 ]; then
                                    case "${BRIDGET_CODEX_REQUEST_VARIANT:-known}" in
                                        method) printf '%s\n' '{"id":"approval-native","method":"SENTINELLE-METHOD-019\u001b[2J\r\u202e","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"i","command":"SENTINELLE-SECRETE-019"}}' ;;
                                        request_id) printf '%s\n' '{"id":"SENTINELLE-REQUEST-ID-019","method":"item/commandExecution/requestApproval","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"i","command":"SENTINELLE-SECRETE-019"}}' ;;
                                        turn_id) printf '%s\n' '{"id":"approval-native","method":"item/commandExecution/requestApproval","params":{"threadId":"thread-native","turnId":"SENTINELLE-TURN-ID-019","itemId":"i","command":"SENTINELLE-SECRETE-019"}}' ;;
                                        *) printf '%s\n' '{"id":"approval-native","method":"item/commandExecution/requestApproval","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"i","command":"SENTINELLE-SECRETE-019"}}' ;;
                                    esac
                                    # Attend la décision JSON-RPC (présence), puis poursuit
                                    # le tour (effet) — sauf HOLD/EXIT pour les bancs 019.
                                    saw_decision=0
                                    while IFS= read -r reply; do
                                        printf '%s\n' "$reply" >> "$BRIDGET_CODEX_TRACE"
                                        case "$reply" in
                                            *'"decision"'*) saw_decision=1; break ;;
                                        esac
                                    done
                                    if [ "${BRIDGET_CODEX_EXIT_AFTER_REQUEST:-0}" = 1 ]; then exit 0; fi
                                    if [ "${BRIDGET_CODEX_HOLD_TURN:-0}" = 1 ]; then :;
                                    elif [ "$saw_decision" = 1 ]; then
                                        printf '%s\n' '{"method":"item/agentMessage/delta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"i","delta":"tour poursuivi apres autorisation"}}'
                                        printf '%s\n' '{"method":"turn/completed","params":{"threadId":"thread-native","turn":{"id":"turn-native","status":"completed","items":[]}}}'
                                    fi
                                elif [ "${BRIDGET_CODEX_HOLD_TURN:-0}" != 1 ]; then
                                    if [ "${BRIDGET_CODEX_ACTIVITY:-0}" = 1 ]; then
                                        printf '%s\n' '{"method":"item/reasoning/summaryPartAdded","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"reasoning-1","summaryIndex":0}}'
                                        printf '%s\n' '{"method":"item/reasoning/summaryTextDelta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"reasoning-1","summaryIndex":0,"delta":"Je compare"}}'
                                        printf '%s\n' '{"method":"item/reasoning/summaryTextDelta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"reasoning-1","summaryIndex":0,"delta":" les options."}}'
                                        printf '%s\n' '{"method":"item/reasoning/textDelta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"reasoning-1","contentIndex":0,"delta":"raison brute"}}'
                                        printf '%s\n' '{"method":"item/started","params":{"threadId":"thread-native","turnId":"turn-native","startedAtMs":1,"item":{"type":"commandExecution","id":"exec-mesure","command":"echo MESURE_CODEX_CMD_77","cwd":"/tmp/bt","status":"inProgress"}}}'
                                        printf '%s\n' '{"method":"item/commandExecution/outputDelta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"command-1","delta":"313 passés"}}'
                                        if [ "${BRIDGET_CODEX_SANDBOX_REFUSAL:-0}" = 1 ]; then
                                            printf '%s\n' '{"method":"item/commandExecution/outputDelta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"command-1","delta":"bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted\nls: cannot access x\n"}}'
                                            printf '%s\n' '{"method":"item/commandExecution/outputDelta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"command-1","delta":"bwrap: execvp: Permission denied\n"}}'
                                        fi
                                        if [ "${BRIDGET_CODEX_COMMAND_COMPLETION:-}" = failed ]; then
                                            printf '%s\n' '{"method":"item/completed","params":{"threadId":"thread-native","turnId":"turn-native","item":{"type":"commandExecution","id":"command-1","command":"echo MESURE_CODEX_CMD_77","cwd":"/tmp/bt","status":"failed","exitCode":1,"aggregatedOutput":"bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted\n"}}}'
                                        elif [ "${BRIDGET_CODEX_COMMAND_COMPLETION:-}" = completed ]; then
                                            printf '%s\n' '{"method":"item/completed","params":{"threadId":"thread-native","turnId":"turn-native","item":{"type":"commandExecution","id":"command-1","command":"echo MESURE_CODEX_CMD_77","cwd":"/tmp/bt","status":"completed","exitCode":0,"aggregatedOutput":"313 passés\nBRIDGET-TEST-fixture\n"}}}'
                                        fi
                                        printf '%s\n' '{"method":"item/commandExecution/outputDelta","params":{"threadId":"foreign-thread","turnId":"foreign-turn","itemId":"foreign-command","delta":"ne pas attribuer"}}'
                                        printf '%s\n' '{"method":"item/fileChange/patchUpdated","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"file-1","changes":[{"path":"src/main.rs","kind":{"type":"update"},"diff":"@@ -1 +1 @@"}]}}'
                                        printf '%s\n' '{"method":"item/plan/delta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"plan-1","delta":"Tester le flux"}}'
                                        printf '%s\n' '{"id":"approval-7","method":"item/commandExecution/requestApproval","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"approval-1","startedAtMs":1787686800000,"command":"cargo test -p bridget-transport","reason":"sortie réseau"}}'
                                        printf '%s\n' '{"id":"approval-8","method":"item/fileChange/requestApproval","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"approval-2","startedAtMs":1787686800000,"reason":"écrire le fichier"}}'
                                        printf '%s\n' '{"id":"approval-9","method":"item/permissions/requestApproval","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"approval-3","startedAtMs":1787686800000,"cwd":"/tmp","permissions":{},"reason":"accès réseau"}}'
                                        printf '%s\n' '{"method":"item/futureWidget/delta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"future-1","delta":"ne pas inventer"}}'
                                    fi
                                    case "${BRIDGET_CODEX_TEXT_MODE:-default}" in
                                        deltas)
                                            printf '%s\n' '{"method":"item/agentMessage/delta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"i","delta":"BONJOUR "}}'
                                            printf '%s\n' '{"method":"item/agentMessage/delta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"i","delta":"JE SUIS VIVANT"}}'
                                            printf '%s\n' '{"method":"item/completed","params":{"threadId":"thread-native","turnId":"turn-native","completedAtMs":1,"item":{"id":"i","type":"agentMessage","text":"BONJOUR JE SUIS VIVANT"}}}'
                                            ;;
                                        final_only)
                                            printf '%s\n' '{"method":"item/completed","params":{"threadId":"thread-native","turnId":"turn-native","completedAtMs":1,"item":{"id":"i","type":"agentMessage","text":"REPONSE FINALE"}}}'
                                            ;;
                                        *)
                                            printf '%s\n' '{  "method" : "item/agentMessage/delta" , "params" : { "threadId" : "thread-native" , "turnId" : "turn-native" , "itemId":"i", "delta" : "réponse native" } }'
                                            ;;
                                    esac
                                    printf '%s\n' '{"method":"turn/completed","params":{"threadId":"thread-native","turn":{"id":"turn-native","status":"completed","items":[]}}}'
                                fi
                            fi ;;
                        *'"method":"turn/steer"'*)
                            steer_id=$(printf '%s' "$line" | sed 's/.*"id":\([0-9]*\).*/\1/')
                            if [ "${BRIDGET_CODEX_STEER_REFUSE:-0}" = 1 ]; then
                                printf '%s\n' "{\"id\":$steer_id,\"error\":{\"code\":-32600,\"message\":\"cannot steer a review turn\",\"data\":{\"message\":\"cannot steer a review turn\",\"codexErrorInfo\":{\"activeTurnNotSteerable\":{\"turnKind\":\"review\"}}}}}"
                            else
                                printf '%s\n' "{\"id\":$steer_id,\"result\":{}}"
                                if [ "${BRIDGET_CODEX_STEER_CONSUME:-0}" = 1 ]; then
                                    steer_message_id=$(printf '%s' "$line" | sed 's/.*"clientUserMessageId":"\([^"]*\)".*/\1/')
            thread_bootstrap: CodexThreadBootstrap::Start,
                                    provider_item_id="provider-item-$steer_message_id"
                                    printf '%s\n' "{\"method\":\"item/started\",\"params\":{\"threadId\":\"thread-native\",\"turnId\":\"turn-native\",\"startedAtMs\":1,\"item\":{\"id\":\"$provider_item_id\",\"clientId\":\"$steer_message_id\",\"type\":\"userMessage\",\"content\":[]}}}"
                                fi
                            fi ;;
                        *'"method":"turn/interrupt"'*)
                            interrupt_id=$(printf '%s' "$line" | sed 's/.*"id":\([0-9]*\).*/\1/')
                            printf '%s\n' "{\"id\":$interrupt_id,\"result\":{}}"
                            printf '%s\n' '{"method":"turn/completed","params":{"threadId":"thread-native","turn":{"id":"turn-native","status":"interrupted","items":[]}}}' ;;
                    esac
                done"#.to_string(),
            ],
            queue_capacity: 2,
            notify_timeout_secs: 2,
            provider_observation: None,
            thread_bootstrap: Default::default(),
            model: Some("gpt-5.6-terra".to_string()),
            permissions: "allow".to_string(),
            dynamic_tool_handler: None,
        }
    }

    fn message(id: &str) -> BridgetMessage {
        let mut message = BridgetMessage::new("bridget", "codex-native", format!("mission {id}"));
        message.id = id.to_string();
        message.reply = true;
        message
    }

    fn journal_detail_fixture(label: &str, with_activity: bool) -> (Vec<Value>, Vec<Value>) {
        let _suite = if with_activity {
            Some(crate::act_kind::pilot_kind_suite_lock())
        } else {
            None
        };
        let root = root(label);
        let trace = root.join("trace.jsonl");
        let mut environment = vec![(
            "BRIDGET_CODEX_TRACE".to_string(),
            trace.to_string_lossy().into_owned(),
        )];
        if with_activity {
            environment.push(("BRIDGET_CODEX_ACTIVITY".to_string(), "1".to_string()));
        }
        let mut transport = CodexAppServerTransport::spawn_with_environment(
            fake_options(&trace),
            &environment,
            false,
        )
        .expect("session native de détail");
        transport
            .activate_journal(&root, "codex-native", None)
            .expect("journal de détail activé");
        transport
            .deliver(&message(label))
            .expect("livraison de détail");

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut finished = false;
        while Instant::now() < deadline {
            finished |= transport.drain_events().iter().any(|event| {
                matches!(
                    event.kind,
                    ManagedEventKind::TurnFinished {
                        terminal: ManagedTerminal::Completed,
                        ..
                    }
                )
            });
            if finished {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(finished, "le tour de détail n'a pas terminé");
        transport.stop();

        let journal_path = fs::read_dir(root.join("codex-native"))
            .expect("répertoire du journal")
            .next()
            .expect("fichier du journal")
            .expect("entrée du journal")
            .path();
        let events = crate::journal::valid_events(&journal_path);
        let frames = fs::read_to_string(&trace)
            .expect("trace fournisseur")
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .collect();
        fs::remove_dir_all(root).expect("nettoyage de la fixture");
        (events, frames)
    }

    fn journal_text_fixture(label: &str, text_mode: &str) -> Vec<Value> {
        let root = root(label);
        let trace = root.join("trace.jsonl");
        let environment = vec![
            (
                "BRIDGET_CODEX_TRACE".to_string(),
                trace.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_TEXT_MODE".to_string(), text_mode.to_string()),
        ];
        let mut transport = CodexAppServerTransport::spawn_with_environment(
            fake_options(&trace),
            &environment,
            false,
        )
        .expect("session native texte");
        transport
            .activate_journal(&root, "codex-native", None)
            .expect("journal texte activé");
        let mut msg = message(label);
        msg.id = "codex-abc-1".to_string();
        transport.deliver(&msg).expect("livraison texte");

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut finished = false;
        while Instant::now() < deadline {
            finished |= transport.drain_events().iter().any(|event| {
                matches!(
                    event.kind,
                    ManagedEventKind::TurnFinished {
                        terminal: ManagedTerminal::Completed,
                        ..
                    }
                )
            });
            if finished {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(finished, "le tour texte Codex n'a pas terminé");
        transport.stop();

        let journal_path = fs::read_dir(root.join("codex-native"))
            .expect("répertoire du journal")
            .next()
            .expect("fichier du journal")
            .expect("entrée du journal")
            .path();
        let events = crate::journal::valid_events(&journal_path);
        fs::remove_dir_all(root).expect("nettoyage de la fixture texte");
        events
    }

    fn assert_whitelist_events_exclude(text: &str, forbidden: &str) {
        for line in text.lines().filter(|line| !line.is_empty()) {
            let Ok(entry) = serde_json::from_str::<Value>(line) else {
                continue;
            };
            if entry["event"] == "provider_request" {
                let payload =
                    serde_json::to_string(&entry["payload"]).expect("payload sérialisable");
                assert!(
                    !payload.contains(forbidden),
                    "provider_request ne doit pas porter {forbidden:?}: {payload}"
                );
            }
            if entry["event"] == "error"
                && let Some(pending) = entry.pointer("/payload/pending_provider_request")
            {
                let pending = serde_json::to_string(pending).expect("pending sérialisable");
                assert!(
                    !pending.contains(forbidden),
                    "pending_provider_request ne doit pas porter {forbidden:?}: {pending}"
                );
            }
        }
    }

    fn assert_no_provider_controls(text: &str) {
        for forbidden in ["\u{1b}", "\r", "\u{202e}", "\\u001b", "\\r", "\\u202e"] {
            assert!(
                !text.contains(forbidden),
                "contrôle fournisseur durable interdit: {forbidden:?}"
            );
        }
    }

    fn assert_fingerprint(value: &str) {
        assert_eq!(value.len(), "sha256:".len() + 64);
        assert!(value.starts_with("sha256:"));
        assert!(
            value["sha256:".len()..]
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()),
            "empreinte hexadécimale minuscule attendue"
        );
    }

    fn assert_provider_projection_redacts(variant: &str, label: &str, sentinel: &str) {
        let root = root(label);
        let trace = root.join("trace.jsonl");
        let mut options = fake_options(&trace);
        options.notify_timeout_secs = 1;
        let mut transport = CodexAppServerTransport::spawn_with_environment(
            options,
            &[
                (
                    "BRIDGET_CODEX_TRACE".to_string(),
                    trace.to_string_lossy().into_owned(),
                ),
                (
                    "BRIDGET_CODEX_REQUEST_APPROVAL".to_string(),
                    "1".to_string(),
                ),
                (
                    "BRIDGET_CODEX_REQUEST_VARIANT".to_string(),
                    variant.to_string(),
                ),
                ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
            ],
            false,
        )
        .expect("session native");
        transport
            .activate_journal(&root, "codex-native", None)
            .expect("journal activé");
        let message_id = format!("provider-{label}-019");
        transport.deliver(&message(&message_id)).expect("livraison");

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events.iter().any(|event| {
                matches!(
                    event.kind,
                    ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
                        if message_id == &format!("provider-{label}-019")
                            && reason == "échéance Codex dépassée"
                )
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(
            events.iter().any(|event| {
                event
                    .raw
                    .windows(sentinel.len())
                    .any(|window| window == sentinel.as_bytes())
            }),
            "sentinelle absente de la trame fournisseur effectivement lue"
        );
        if variant == "method" {
            let detail = events
                .iter()
                .find_map(|event| match &event.kind {
                    ManagedEventKind::Update { detail }
                        if event
                            .raw
                            .windows(sentinel.len())
                            .any(|window| window == sentinel.as_bytes()) =>
                    {
                        Some(detail.as_str())
                    }
                    _ => None,
                })
                .expect("détail opérateur de la méthode inconnue");
            assert!(!detail.contains(sentinel));
            assert_no_provider_controls(detail);
            assert!(detail.contains("sha256:"));
        }

        transport.stop();
        let journal_path = fs::read_dir(root.join("codex-native"))
            .expect("répertoire journal")
            .next()
            .expect("fichier journal")
            .expect("entrée journal")
            .path();
        let entries = crate::journal::valid_events(&journal_path);
        let request = entries
            .iter()
            .find(|entry| entry["event"] == "provider_request")
            .expect("requête fournisseur durable avant l'échéance");
        let error = entries
            .iter()
            .find(|entry| entry["event"] == "error")
            .expect("échéance durable");
        for field in ["method", "request_id", "turn_id"] {
            assert!(
                request["payload"][field]
                    .as_str()
                    .is_some_and(|value| !value.is_empty()),
                "corrélation {field} non vide"
            );
        }
        assert_fingerprint(
            request["payload"]["request_id"]
                .as_str()
                .expect("empreinte de requête"),
        );
        assert_fingerprint(
            request["payload"]["turn_id"]
                .as_str()
                .expect("empreinte de tour"),
        );
        match variant {
            "method" => assert_fingerprint(
                request["payload"]["method"]
                    .as_str()
                    .expect("empreinte de méthode"),
            ),
            _ => assert_eq!(
                request["payload"]["method"],
                COMMAND_EXECUTION_APPROVAL_METHOD
            ),
        }
        assert_eq!(
            error["payload"]["pending_provider_request"], request["payload"],
            "projection stable entre la requête et sa borne terminale"
        );
        let journal = fs::read_to_string(&journal_path).expect("journal brut");
        assert!(
            !journal.contains(sentinel),
            "chaîne fournisseur libre persistée via {variant}"
        );
        assert_no_provider_controls(&journal);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_019_identifiant_json_rpc_invalide_ne_recopie_pas_son_contenu() {
        let identifier = project_request_id(&json!({
            "secret": "SENTINELLE-ID-019"
        }));
        assert_fingerprint(&identifier);
        assert!(!identifier.contains("SENTINELLE-ID-019"));
    }

    #[test]
    fn test_019_empreintes_sont_stables_et_separees_par_domaine() {
        let method = provider_fingerprint(b"method", b"meme-valeur");
        let request = provider_fingerprint(b"request-id:string", b"meme-valeur");
        let turn = provider_fingerprint(b"turn-id", b"meme-valeur");
        assert_fingerprint(&method);
        assert_fingerprint(&request);
        assert_fingerprint(&turn);
        assert_eq!(method, provider_fingerprint(b"method", b"meme-valeur"));
        assert_ne!(method, request);
        assert_ne!(method, turn);
        assert_ne!(request, turn);
        assert_eq!(
            project_provider_method(COMMAND_EXECUTION_APPROVAL_METHOD),
            COMMAND_EXECUTION_APPROVAL_METHOD
        );
    }

    #[test]
    fn test_019_methode_libre_est_opaque_et_sans_controle() {
        assert_provider_projection_redacts("method", "method-redaction", "SENTINELLE-METHOD-019");
    }

    #[test]
    fn test_019_identifiant_chaine_valide_est_opaque() {
        assert_provider_projection_redacts(
            "request_id",
            "request-id-redaction",
            "SENTINELLE-REQUEST-ID-019",
        );
    }

    #[test]
    fn test_019_identifiant_de_tour_est_opaque() {
        assert_provider_projection_redacts(
            "turn_id",
            "turn-id-redaction",
            "SENTINELLE-TURN-ID-019",
        );
    }

    #[test]
    fn test_019_raison_fournisseur_est_opaque_et_sans_controle() {
        let root = root("provider-reason-redaction");
        let trace = root.join("trace.jsonl");
        let output_trace = root.join("provider-output.jsonl");
        let mut transport = CodexAppServerTransport::spawn_with_environment(
            fake_options(&trace),
            &[
                (
                    "BRIDGET_CODEX_TRACE".to_string(),
                    trace.to_string_lossy().into_owned(),
                ),
                ("BRIDGET_CODEX_TURN_ERROR".to_string(), "1".to_string()),
                (
                    "BRIDGET_CODEX_OUTPUT_TRACE".to_string(),
                    output_trace.to_string_lossy().into_owned(),
                ),
            ],
            false,
        )
        .expect("session native");
        transport
            .activate_journal(&root, "codex-native", None)
            .expect("journal activé");
        transport
            .deliver(&message("provider-reason-019"))
            .expect("livraison");

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events.iter().any(|event| {
                matches!(
                    event.kind,
                    ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
                        if message_id == "provider-reason-019" && reason.contains("-32098")
                )
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let rejected_reason = events
            .iter()
            .find_map(|event| match &event.kind {
                ManagedEventKind::DeliveryRejected { message_id, reason }
                    if message_id == "provider-reason-019" && reason.contains("-32098") =>
                {
                    Some(reason.as_str())
                }
                _ => None,
            })
            .expect("la réponse d'erreur fournisseur n'a pas été lue");
        assert!(!rejected_reason.contains("SENTINELLE-RAISON-019"));
        assert_no_provider_controls(rejected_reason);
        assert!(rejected_reason.contains("sha256:"));
        let emitted = fs::read_to_string(&output_trace).expect("sortie fournisseur attestée");
        assert!(emitted.contains("SENTINELLE-RAISON-019"));

        transport.stop();
        let journal_path = fs::read_dir(root.join("codex-native"))
            .expect("répertoire journal")
            .next()
            .expect("fichier journal")
            .expect("entrée journal")
            .path();
        let entries = crate::journal::valid_events(&journal_path);
        let error = entries
            .iter()
            .find(|entry| entry["event"] == "error")
            .expect("erreur fournisseur durable");
        assert!(
            error["payload"]["reason"]
                .as_str()
                .is_some_and(|reason| !reason.is_empty()),
            "raison durable non vide"
        );
        assert!(
            error["payload"]["reason"]
                .as_str()
                .is_some_and(|reason| reason.contains("sha256:")),
            "référence opaque de l'erreur fournisseur"
        );
        let journal = fs::read_to_string(&journal_path).expect("journal brut");
        assert!(
            !journal.contains("SENTINELLE-RAISON-019"),
            "raison fournisseur libre persistée"
        );
        assert_no_provider_controls(&journal);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_019_saturation_du_journal_devient_un_echec_observable() {
        let root = root("provider-request-journal-full");
        let trace = root.join("trace.jsonl");
        let mut options = fake_options(&trace);
        options.notify_timeout_secs = 1;
        let mut transport = CodexAppServerTransport::spawn_with_environment(
            options,
            &[
                (
                    "BRIDGET_CODEX_TRACE".to_string(),
                    trace.to_string_lossy().into_owned(),
                ),
                (
                    "BRIDGET_CODEX_REQUEST_APPROVAL".to_string(),
                    "1".to_string(),
                ),
                ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
            ],
            false,
        )
        .expect("session native");
        *transport
            .journal
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) =
            Some(JournalWriter::saturated_for_test());
        transport
            .deliver(&message("provider-journal-full-019"))
            .expect("livraison");

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events.iter().any(|event| {
                matches!(
                    event.kind,
                    ManagedEventKind::JournalFailed { ref detail }
                        if detail.contains("requête fournisseur impossible")
                )
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let failure = events
            .iter()
            .find(|event| matches!(event.kind, ManagedEventKind::JournalFailed { .. }))
            .expect("échec de journalisation observable");
        assert!(matches!(
            failure.kind,
            ManagedEventKind::JournalFailed { ref detail }
                if detail.contains("journal ACP saturé")
                    || detail.contains("requête fournisseur impossible")
        ));
        if matches!(
            failure.kind,
            ManagedEventKind::JournalFailed { ref detail }
                if detail.contains("requête fournisseur impossible")
        ) {
            assert!(
                failure
                    .raw
                    .windows(b"item/commandExecution/requestApproval".len())
                    .any(|window| window == b"item/commandExecution/requestApproval")
            );
        }

        transport.stop();
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_019_requete_fournisseur_est_tracee_avant_echeance_sans_payload() {
        let root = root("provider-request");
        let trace = root.join("trace.jsonl");
        let mut options = fake_options(&trace);
        options.notify_timeout_secs = 1;
        let mut transport = CodexAppServerTransport::spawn_with_environment(
            options,
            &[
                (
                    "BRIDGET_CODEX_TRACE".to_string(),
                    trace.to_string_lossy().into_owned(),
                ),
                (
                    "BRIDGET_CODEX_REQUEST_APPROVAL".to_string(),
                    "1".to_string(),
                ),
                ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
            ],
            false,
        )
        .expect("session native");
        transport
            .activate_journal(&root, "codex-native", None)
            .expect("journal activé");
        transport
            .deliver(&message("provider-request-019"))
            .expect("livraison");

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events.iter().any(|event| {
                matches!(
                    event.kind,
                    ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
                        if message_id == "provider-request-019"
                            && reason == "échéance Codex dépassée"
                )
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(events.iter().any(|event| matches!(
            event.kind,
            ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
                if message_id == "provider-request-019"
                    && reason == "échéance Codex dépassée"
        )));

        let provider_frame = events
            .iter()
            .find(|event| {
                event
                    .raw
                    .windows(b"item/commandExecution/requestApproval".len())
                    .any(|window| window == b"item/commandExecution/requestApproval")
            })
            .expect("requête effectivement lue depuis stdout du faux pilote");
        let provider_frame =
            std::str::from_utf8(&provider_frame.raw).expect("trame fournisseur UTF-8");
        assert!(provider_frame.contains("SENTINELLE-SECRETE-019"));

        transport.stop();
        let journal_path = fs::read_dir(root.join("codex-native"))
            .expect("répertoire journal")
            .next()
            .expect("fichier journal")
            .expect("entrée journal")
            .path();
        let entries = crate::journal::valid_events(&journal_path);
        let request = entries
            .iter()
            .find(|entry| entry["event"] == "provider_request")
            .expect("requête fournisseur durable avant l'échéance");
        assert_eq!(request["message_id"], "provider-request-019");
        assert_eq!(
            request["payload"],
            json!({
                "provider": "codex",
                "method": "item/commandExecution/requestApproval",
                "request_id": provider_fingerprint(b"request-id:string", b"approval-native"),
                "turn_id": provider_fingerprint(b"turn-id", b"turn-native"),
                "state": "pending"
            })
        );
        let journal = fs::read_to_string(&journal_path).expect("journal brut");
        // Compromis explicite avec la visibilité GUI (spec 032) : la sentinelle
        // peut apparaître dans un acte `approval` (command/reason), jamais dans
        // provider_request ni pending_provider_request (liste blanche FR-1903).
        assert_whitelist_events_exclude(&journal, "SENTINELLE-SECRETE-019");
        let request_seq = request["seq"].as_u64().expect("séquence requête");
        let error = entries
            .iter()
            .find(|entry| entry["event"] == "error")
            .expect("échéance durable");
        assert!(request_seq < error["seq"].as_u64().expect("séquence erreur"));
        assert_eq!(error["message_id"], request["message_id"]);
        assert_eq!(
            error["payload"]["pending_provider_request"],
            request["payload"]
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn test_019_eof_reste_correle_a_la_derniere_requete_fournisseur() {
        let root = root("provider-request-eof");
        let trace = root.join("trace.jsonl");
        let mut options = fake_options(&trace);
        options.notify_timeout_secs = 60;
        let mut transport = CodexAppServerTransport::spawn_with_environment(
            options,
            &[
                (
                    "BRIDGET_CODEX_TRACE".to_string(),
                    trace.to_string_lossy().into_owned(),
                ),
                (
                    "BRIDGET_CODEX_REQUEST_APPROVAL".to_string(),
                    "1".to_string(),
                ),
                (
                    "BRIDGET_CODEX_EXIT_AFTER_REQUEST".to_string(),
                    "1".to_string(),
                ),
            ],
            false,
        )
        .expect("session native");
        transport
            .activate_journal(&root, "codex-native", None)
            .expect("journal activé");
        transport
            .deliver(&message("provider-eof-019"))
            .expect("livraison");

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events.iter().any(|event| {
                matches!(
                    event.kind,
                    ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
                        if message_id == "provider-eof-019"
                            && reason == "stdout Codex fermé pendant le tour"
                )
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(events.iter().any(|event| matches!(
            event.kind,
            ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
                if message_id == "provider-eof-019"
                    && reason == "stdout Codex fermé pendant le tour"
        )));
        assert!(events.iter().any(|event| {
            event
                .raw
                .windows(b"item/commandExecution/requestApproval".len())
                .any(|window| window == b"item/commandExecution/requestApproval")
        }));

        transport.stop();
        let journal_path = fs::read_dir(root.join("codex-native"))
            .expect("répertoire journal")
            .next()
            .expect("fichier journal")
            .expect("entrée journal")
            .path();
        let entries = crate::journal::valid_events(&journal_path);
        let request = entries
            .iter()
            .find(|entry| entry["event"] == "provider_request")
            .expect("requête fournisseur durable avant EOF");
        let error = entries
            .iter()
            .find(|entry| entry["event"] == "error")
            .expect("EOF durable");
        assert!(
            request["seq"].as_u64().expect("séquence requête")
                < error["seq"].as_u64().expect("séquence erreur")
        );
        assert_eq!(error["message_id"], request["message_id"]);
        assert_eq!(
            error["payload"]["pending_provider_request"],
            request["payload"]
        );

        let _ = fs::remove_dir_all(root);
    }

    /// (A) Deltas → update avec TEXTE EXACT. Mutant content:"" doit tuer A seul.
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_A_codex_app_server_retranscrit_les_deltas_en_update() {
        let events = journal_text_fixture("temoin-a", "deltas");
        let updates: Vec<_> = events
            .iter()
            .filter(|event| event["event"] == "update" && event["payload"]["kind"] == "text")
            .collect();
        let contents: Vec<&str> = updates
            .iter()
            .filter_map(|event| event["payload"]["content"].as_str())
            .collect();
        assert_eq!(
            contents,
            vec!["BONJOUR ", "JE SUIS VIVANT"],
            "deux deltas exacts sans doublon item/completed: {contents:?} via {events:?}"
        );
        assert!(
            updates
                .iter()
                .all(|event| event["message_id"] == "codex-abc-1"),
            "même message_id, reçu {updates:?}"
        );
    }

    /// (B) Tour sans delta : item/completed agentMessage → texte final exact.
    /// Mutant : retirer maybe_record_final_agent_message → left=[].
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_B_codex_app_server_reponse_finale_sans_delta_dans_le_fil() {
        let events = journal_text_fixture("temoin-b", "final_only");
        let updates: Vec<_> = events
            .iter()
            .filter(|event| event["event"] == "update" && event["payload"]["kind"] == "text")
            .collect();
        let contents: Vec<&str> = updates
            .iter()
            .filter_map(|event| event["payload"]["content"].as_str())
            .collect();
        assert_eq!(
            contents,
            vec!["REPONSE FINALE"],
            "repli item/completed → update exact, reçu {contents:?} via {events:?}"
        );
        assert!(
            updates
                .iter()
                .all(|event| event["message_id"] == "codex-abc-1"),
            "même message_id, reçu {updates:?}"
        );
    }

    /// (C) turn_end d'un SUCCÈS porte stop_reason=completed (+ routed_to si reply).
    /// Distinct de l'échéance DeliveryRejected sans turn_end. Mutant turn_end {} → C meurt.
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_C_codex_app_server_turn_end_porte_stop_reason_completed() {
        let events = journal_text_fixture("temoin-c", "final_only");
        let turn_end = events
            .iter()
            .find(|event| event["event"] == "turn_end")
            .expect("turn_end attendu sur succès");
        assert_eq!(
            turn_end["payload"]["stop_reason"].as_str(),
            Some("completed"),
            "stop_reason manquant → attach « inconnu », payload={}",
            turn_end["payload"]
        );
        assert_eq!(
            turn_end["payload"]["routed_to"].as_str(),
            Some("bridget"),
            "reply=true doit porter routed_to, payload={}",
            turn_end["payload"]
        );
    }

    /// Point D du chantier : un message arrivé pendant un tour actif est INJECTÉ
    /// dans ce tour par `turn/steer`, sans l'interrompre et sans attendre sa fin.
    ///
    /// Assertion métier : la requête porte `expectedTurnId` = le tour ACTIF
    /// (`turn-native`), et non l'identifiant du message ni un tour mis en file
    /// (contrainte t3code CodexSessionRuntime.ts:1853).
    ///
    /// Mutant qui le tue : router le second message vers `messages` au lieu de
    /// `steer` dans `deliver` — aucun `turn/steer` n'est alors émis, le message
    /// attend la fin du tour, et c'est exactement le défaut que le chantier
    /// demande de corriger.
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_E_codex_message_arrive_est_pilote_dans_le_tour_en_cours() {
        let root = root("temoin-steer");
        let trace = root.join("trace.jsonl");
        let mut options = fake_options(&trace);
        options.notify_timeout_secs = 6;
        let mut transport = CodexAppServerTransport::spawn_with_environment(
            options,
            &[
                (
                    "BRIDGET_CODEX_TRACE".to_string(),
                    trace.to_string_lossy().into_owned(),
                ),
                ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
            ],
            false,
        )
        .expect("session native pilotage");

        let mut premier = message("temoin-steer");
        premier.id = "codex-steer-tour".to_string();
        transport.deliver(&premier).expect("livraison du premier");

        // Attendre que le tour soit RÉELLEMENT actif : piloter avant que
        // `turn/start` ait rendu son identifiant n'aurait aucun sens.
        let attente = Instant::now() + Duration::from_secs(5);
        while Instant::now() < attente {
            if fs::read_to_string(&trace)
                .unwrap_or_default()
                .contains("\"method\":\"turn/start\"")
            {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        thread::sleep(Duration::from_millis(150));

        let mut second = message("temoin-steer-2");
        second.id = "codex-steer-injecte".to_string();
        second.from = "superviseur".to_string();
        second.intent = Some(MessageIntent::SteerCurrent);
        transport.deliver(&second).expect("livraison du second");

        let deadline = Instant::now() + Duration::from_secs(5);
        let mut ligne_steer = None;
        while Instant::now() < deadline {
            let contenu = fs::read_to_string(&trace).unwrap_or_default();
            if let Some(ligne) = contenu
                .lines()
                .find(|ligne| ligne.contains("\"method\":\"turn/steer\""))
            {
                ligne_steer = Some(ligne.to_string());
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        transport.stop();

        let ligne_steer = ligne_steer.unwrap_or_else(|| {
            panic!(
                "aucun turn/steer émis : le message a attendu la fin du tour; trace={}",
                fs::read_to_string(&trace).unwrap_or_else(|error| error.to_string())
            )
        });
        assert!(
            ligne_steer.contains("\"expectedTurnId\":\"turn-native\""),
            "turn/steer doit viser le tour ACTIF, ligne={ligne_steer}"
        );
        assert!(
            ligne_steer.contains("\"clientUserMessageId\":\"codex-steer-injecte\""),
            "turn/steer doit porter le message arrivé, ligne={ligne_steer}"
        );
        let steer: Value = serde_json::from_str(&ligne_steer).unwrap();
        let prompt = steer["params"]["input"][0]["text"].as_str().unwrap();
        let header = prompt.lines().next().unwrap();
        let envelope: Value = serde_json::from_str(
            header
                .strip_prefix("[Message Bridget : ")
                .unwrap()
                .strip_suffix(']')
                .unwrap(),
        )
        .unwrap();
        assert_eq!(envelope["from"], second.from);
        assert_eq!(envelope["id"], second.id);
        assert_eq!(envelope["reply"], second.reply);
        assert!(prompt.contains(&second.body));
        let _ = fs::remove_dir_all(root);
    }

    /// Un tour NON PILOTABLE (`activeTurnNotSteerable`, mesuré le 28/08 contre
    /// codex-cli 0.150.1) doit rendre le message à la file PRINCIPALE, pour
    /// qu'il soit traité au tour suivant — et non le rejouer en boucle dans la
    /// file de pilotage, ce qui bloquerait tout ce qui suit.
    ///
    /// Assertion métier : après l'échéance du premier tour, un SECOND
    /// `turn/start` est émis — preuve que le message refusé est reparti en mode
    /// « queue ». Mutant qui le tue : remplacer `state.messages.push_front` par
    /// `state.steer.push_front` dans la branche non pilotable.
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_F_codex_tour_non_pilotable_rend_le_message_a_la_file() {
        let root = root("temoin-non-pilotable");
        let trace = root.join("trace.jsonl");
        let mut options = fake_options(&trace);
        options.notify_timeout_secs = 3;
        let mut transport = CodexAppServerTransport::spawn_with_environment(
            options,
            &[
                (
                    "BRIDGET_CODEX_TRACE".to_string(),
                    trace.to_string_lossy().into_owned(),
                ),
                ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
                ("BRIDGET_CODEX_STEER_REFUSE".to_string(), "1".to_string()),
            ],
            false,
        )
        .expect("session native non pilotable");

        let mut premier = message("np-1");
        premier.id = "codex-np-tour".to_string();
        transport.deliver(&premier).expect("livraison du premier");

        let attente = Instant::now() + Duration::from_secs(5);
        while Instant::now() < attente {
            if fs::read_to_string(&trace)
                .unwrap_or_default()
                .contains("\"method\":\"turn/start\"")
            {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        thread::sleep(Duration::from_millis(150));

        let mut second = message("np-2");
        second.id = "codex-np-refuse".to_string();
        second.from = "humain".to_string();
        second.intent = Some(MessageIntent::SteerCurrent);
        transport.deliver(&second).expect("livraison du second");

        // Le premier tour tombe sur son échéance (HOLD_TURN), puis le worker
        // doit reprendre le message rendu à la file principale.
        let deadline = Instant::now() + Duration::from_secs(14);
        let mut deux_turn_start = false;
        while Instant::now() < deadline {
            let contenu = fs::read_to_string(&trace).unwrap_or_default();
            let repris = contenu.lines().any(|ligne| {
                ligne.contains("\"method\":\"turn/start\"") && ligne.contains("codex-np-refuse")
            });
            if repris {
                deux_turn_start = true;
                break;
            }
            thread::sleep(Duration::from_millis(40));
        }
        transport.stop();

        let contenu = fs::read_to_string(&trace).unwrap_or_default();
        assert!(
            contenu.contains("\"method\":\"turn/steer\""),
            "le second message devait d'abord être tenté en pilotage; trace={contenu}"
        );
        assert!(
            deux_turn_start,
            "un tour non pilotable doit rendre le message à la file : \
             aucun second turn/start observé, le message est resté bloqué en \
             pilotage; trace={contenu}"
        );
        let _ = fs::remove_dir_all(root);
    }

    /// Un message humain reçu pendant un tour actif doit interrompre ce tour et
    /// démarrer le sien. Il ne doit jamais être injecté via `turn/steer`, car
    /// Codex rattache alors sa réponse au message précédent.
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_G_codex_message_humain_interrompt_et_recoit_un_tour_dedie() {
        let root = root("temoin-steer-sans-consommation");
        let trace = root.join("trace.jsonl");
        let mut options = fake_options(&trace);
        options.notify_timeout_secs = 60;
        let mut transport = CodexAppServerTransport::spawn_with_environment(
            options,
            &[
                (
                    "BRIDGET_CODEX_TRACE".to_string(),
                    trace.to_string_lossy().into_owned(),
                ),
                ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
            ],
            false,
        )
        .expect("session native interruption humaine");

        let mut ancien = message("ancien-bloque");
        ancien.id = "codex-ancien-bloque".to_string();
        transport
            .deliver(&ancien)
            .expect("livraison du tour ancien");

        let attente = Instant::now() + Duration::from_secs(5);
        while Instant::now() < attente {
            if fs::read_to_string(&trace)
                .unwrap_or_default()
                .contains("\"method\":\"turn/start\"")
            {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }

        let mut humain = message("humain-prioritaire");
        humain.id = "codex-humain-prioritaire".to_string();
        humain.from = "humain".to_string();
        humain.origin = Some(MessageOrigin::Human);
        humain.deadline_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("horloge")
                .as_secs()
                + 1,
        );
        transport.deliver(&humain).expect("livraison humaine");

        let mut systeme = message("systeme-apres-humain");
        systeme.id = "codex-systeme-apres-humain".to_string();
        transport.deliver(&systeme).expect("livraison système");

        let deadline = Instant::now() + Duration::from_secs(6);
        let mut humain_repris = false;
        while Instant::now() < deadline {
            let contenu = fs::read_to_string(&trace).unwrap_or_default();
            humain_repris = contenu.lines().any(|ligne| {
                ligne.contains("\"method\":\"turn/start\"")
                    && ligne.contains("codex-humain-prioritaire")
            });
            if humain_repris {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        transport.stop();

        let contenu = fs::read_to_string(&trace).unwrap_or_default();
        let position_humain = contenu
            .find("codex-humain-prioritaire")
            .expect("turn/start humain attendu");
        let position_systeme = contenu.find("codex-systeme-apres-humain");
        assert!(
            contenu.contains("\"method\":\"turn/interrupt\""),
            "le message humain doit interrompre le tour ancien; trace={contenu}"
        );
        assert!(
            humain_repris,
            "le message humain doit démarrer son propre tour avant le système; trace={contenu}"
        );
        assert!(
            position_systeme.is_none_or(|position| position_humain < position),
            "un message système ne doit pas doubler l'humain restitué; trace={contenu}"
        );
        let _ = fs::remove_dir_all(root);
    }

    /// Un résultat JSON-RPC `turn/steer` ne solde jamais une remise humaine.
    /// Le faux serveur garde le tour ouvert et n'émet pas `userMessage` :
    /// aucun `PromptDispatched` humain ne doit sortir avant preuve contraire.
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_H_codex_steer_ack_sans_consommation_ne_solde_pas_la_remise() {
        let root = root("temoin-steer-ack-sans-consommation");
        let trace = root.join("trace.jsonl");
        let mut options = fake_options(&trace);
        options.notify_timeout_secs = 8;
        let mut transport = CodexAppServerTransport::spawn_with_environment(
            options,
            &[
                (
                    "BRIDGET_CODEX_TRACE".to_string(),
                    trace.to_string_lossy().into_owned(),
                ),
                ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
            ],
            false,
        )
        .expect("session native ack sans consommation");

        let mut ancien = message("ack-sans-consommation-ancien");
        ancien.id = "codex-ack-sans-consommation-ancien".to_string();
        transport
            .deliver(&ancien)
            .expect("livraison du tour ancien");
        let attente = Instant::now() + Duration::from_secs(5);
        while Instant::now() < attente {
            if fs::read_to_string(&trace)
                .unwrap_or_default()
                .contains("\"method\":\"turn/start\"")
            {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }

        let mut humain = message("ack-sans-consommation-humain");
        humain.id = "codex-ack-sans-consommation-humain".to_string();
        humain.from = "humain".to_string();
        humain.intent = Some(MessageIntent::SteerCurrent);
        humain.deadline_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("horloge")
                .as_secs()
                + 30,
        );
        transport.deliver(&humain).expect("livraison humaine");

        let deadline = Instant::now() + Duration::from_secs(2);
        let mut steer_observe = false;
        let mut prompt_humain = false;
        while Instant::now() < deadline {
            let contenu = fs::read_to_string(&trace).unwrap_or_default();
            steer_observe |= contenu.contains("\"method\":\"turn/steer\"");
            for event in transport.drain_events() {
                prompt_humain |= matches!(
                    event.kind,
                    ManagedEventKind::PromptDispatched { ref message_id }
                        if message_id == "codex-ack-sans-consommation-humain"
                );
            }
            if steer_observe {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        thread::sleep(Duration::from_millis(100));
        for event in transport.drain_events() {
            prompt_humain |= matches!(
                event.kind,
                ManagedEventKind::PromptDispatched { ref message_id }
                    if message_id == "codex-ack-sans-consommation-humain"
            );
        }
        transport.stop();

        assert!(steer_observe, "turn/steer attendu");
        assert!(
            !prompt_humain,
            "un accusé turn/steer sans item/completed userMessage ne doit pas acquitter la remise"
        );
        let _ = fs::remove_dir_all(root);
    }

    /// La même remise est acquittée dès que le flux natif atteste exactement le
    /// `clientUserMessageId` dans un `item/completed userMessage` du tour actif.
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_I_codex_user_message_corrige_acquitte_la_remise_humaine() {
        let root = root("temoin-steer-consomme");
        let trace = root.join("trace.jsonl");
        let mut options = fake_options(&trace);
        options.notify_timeout_secs = 8;
        let mut transport = CodexAppServerTransport::spawn_with_environment(
            options,
            &[
                (
                    "BRIDGET_CODEX_TRACE".to_string(),
                    trace.to_string_lossy().into_owned(),
                ),
                ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
                ("BRIDGET_CODEX_STEER_CONSUME".to_string(), "1".to_string()),
            ],
            false,
        )
        .expect("session native steering consommé");

        let mut ancien = message("consomme-ancien");
        ancien.id = "codex-consomme-ancien".to_string();
        transport
            .deliver(&ancien)
            .expect("livraison du tour ancien");
        let attente = Instant::now() + Duration::from_secs(5);
        while Instant::now() < attente {
            if fs::read_to_string(&trace)
                .unwrap_or_default()
                .contains("\"method\":\"turn/start\"")
            {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }

        let mut humain = message("consomme-humain");
        humain.id = "codex-consomme-humain".to_string();
        humain.from = "humain".to_string();
        humain.intent = Some(MessageIntent::SteerCurrent);
        humain.deadline_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("horloge")
                .as_secs()
                + 30,
        );
        transport.deliver(&humain).expect("livraison humaine");

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut prompt_humain = false;
        while Instant::now() < deadline {
            for event in transport.drain_events() {
                prompt_humain |= matches!(
                    event.kind,
                    ManagedEventKind::PromptDispatched { ref message_id }
                        if message_id == "codex-consomme-humain"
                );
            }
            if prompt_humain {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        transport.stop();

        assert!(
            prompt_humain,
            "la preuve userMessage corrélée devait déclencher PromptDispatched"
        );
        let _ = fs::remove_dir_all(root);
    }

    fn active_steer_turn_detail() -> ActiveTurnDetail {
        Arc::new(Mutex::new(Some(CodexTurnDetail {
            message_id: "tour-courant".to_string(),
            thread_id: "thread-native".to_string(),
            turn_id: Some("turn-native".to_string()),
            ..Default::default()
        })))
    }

    fn user_message_started(client_id: Option<&str>, turn_id: &str) -> Value {
        let mut item = json!({
            "id": "provider-item-001",
            "type": "userMessage",
            "content": [],
        });
        if let Some(client_id) = client_id {
            item["clientId"] = Value::String(client_id.to_string());
        }
        json!({
            "params": {
                "threadId": "thread-native",
                "turnId": turn_id,
                "item": item,
            },
        })
    }

    #[test]
    fn provider_item_id_est_observe_uniquement_sur_le_tour_actif() {
        let active_detail = active_steer_turn_detail();
        let event = user_message_started(Some("message-humain-001"), "turn-native");

        observe_provider_item_id(&active_detail, &event);
        let mut late = user_message_started(Some("message-humain-001"), "tour-tardif");
        late["params"]["item"]["id"] = Value::String("provider-item-tardif".to_string());
        observe_provider_item_id(&active_detail, &late);

        let observed = active_detail
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        assert_eq!(
            observed
                .as_ref()
                .expect("tour actif")
                .provider_item_id
                .as_deref(),
            Some("provider-item-001")
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_J_codex_item_started_client_id_distingue_id_fournisseur() {
        let observations = Arc::new((Mutex::new(Observations::default()), Condvar::new()));
        let active_detail = active_steer_turn_detail();
        let event = user_message_started(Some("message-humain-001"), "turn-native");

        observe_steered_user_message_visibility(&observations, &active_detail, &event);

        let mut observed = observations
            .0
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let consumed = observed
            .consumed_steers_by_turn
            .get(&("thread-native".to_string(), "turn-native".to_string()))
            .expect("clientId visible pour le tour actif");
        assert!(
            consumed.contains("message-humain-001"),
            "la corrélation doit utiliser clientId, pas l'identifiant interne"
        );
        assert!(
            !consumed.contains("provider-item-001"),
            "l'identifiant interne fournisseur ne doit jamais acquitter Bridget"
        );
        let mut accepted = vec![message("message-humain-001")];
        let confirmed =
            consume_accepted_steers(&mut observed, "thread-native", "turn-native", &mut accepted);
        assert_eq!(confirmed.len(), 1, "une seule remise est attestée");
        assert!(
            accepted.is_empty(),
            "aucune remise distincte ne doit rester"
        );
        assert!(
            consume_accepted_steers(&mut observed, "thread-native", "turn-native", &mut accepted,)
                .is_empty(),
            "la même preuve ne peut pas produire un second acquittement"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_K_codex_item_started_refuse_mauvais_client_id_et_tour_tardif() {
        let observations = Arc::new((Mutex::new(Observations::default()), Condvar::new()));
        let active_detail = active_steer_turn_detail();
        let wrong_id = user_message_started(Some("client-inconnu"), "turn-native");
        observe_steered_user_message_visibility(&observations, &active_detail, &wrong_id);

        let mut observed = observations
            .0
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let mut accepted = vec![message("message-humain-001")];
        assert!(
            consume_accepted_steers(&mut observed, "thread-native", "turn-native", &mut accepted,)
                .is_empty(),
            "un clientId inconnu ne doit pas produire d'acquittement"
        );
        drop(observed);

        let late = user_message_started(Some("message-humain-001"), "tour-termine");
        let late_observations = Arc::new((Mutex::new(Observations::default()), Condvar::new()));
        observe_steered_user_message_visibility(&late_observations, &active_detail, &late);
        assert!(
            late_observations
                .0
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .consumed_steers_by_turn
                .is_empty(),
            "un événement d'un autre tour ne doit laisser aucune preuve"
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_L_codex_item_started_sans_client_id_ne_consomme_pas() {
        let observations = Arc::new((Mutex::new(Observations::default()), Condvar::new()));
        let active_detail = active_steer_turn_detail();
        let event = user_message_started(None, "turn-native");

        observe_steered_user_message_visibility(&observations, &active_detail, &event);

        assert!(
            observations
                .0
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .consumed_steers_by_turn
                .is_empty(),
            "sans clientId il n'existe aucune preuve de consommation"
        );
    }

    fn journal_echeance_fixture(label: &str) -> Vec<Value> {
        let root = root(label);
        let trace = root.join("trace.jsonl");
        // HOLD_TURN : turn/start OK, aucun turn/completed → wait_for_turn
        // tombe sur « échéance Codex dépassée » (notify_timeout_secs=2).
        let mut options = fake_options(&trace);
        options.notify_timeout_secs = 1;
        let mut transport = CodexAppServerTransport::spawn_with_environment(
            options,
            &[
                (
                    "BRIDGET_CODEX_TRACE".to_string(),
                    trace.to_string_lossy().into_owned(),
                ),
                ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
            ],
            false,
        )
        .expect("session native échéance");
        transport
            .activate_journal(&root, "codex-native", None)
            .expect("journal échéance activé");
        let mut msg = message(label);
        msg.id = "codex-echeance-1".to_string();
        transport.deliver(&msg).expect("livraison échéance");

        let deadline = Instant::now() + Duration::from_secs(4);
        let mut rejected = false;
        while Instant::now() < deadline {
            rejected |= transport.drain_events().iter().any(|event| {
                matches!(
                    event.kind,
                    ManagedEventKind::DeliveryRejected { ref reason, .. }
                        if reason == "échéance Codex dépassée"
                )
            });
            if rejected {
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        assert!(
            rejected,
            "DeliveryRejected échéance attendu; trace={}",
            fs::read_to_string(&trace).unwrap_or_else(|error| error.to_string())
        );
        assert!(
            fs::read_to_string(&trace)
                .unwrap_or_default()
                .contains("\"method\":\"turn/interrupt\""),
            "l'échéance fournisseur doit interrompre le tour avant le terminal"
        );
        transport.stop();

        let journal_path = fs::read_dir(root.join("codex-native"))
            .expect("répertoire du journal")
            .next()
            .expect("fichier du journal")
            .expect("entrée du journal")
            .path();
        let events = crate::journal::valid_events(&journal_path);
        fs::remove_dir_all(root).expect("nettoyage échéance");
        events
    }

    /// (D) Séparation verrouillée : échéance → error, AUCUN turn_end.
    /// Mutant INTERDIT : écrire turn_end completed sur DeliveryRejected → D meurt
    /// (A/B/C restent verts — c'est exactement le trou signalé par cursor6).
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_D_codex_app_server_echeance_ecrit_error_sans_turn_end() {
        let events = journal_echeance_fixture("temoin-d");
        let errors: Vec<_> = events
            .iter()
            .filter(|event| event["event"] == "error")
            .collect();
        assert_eq!(
            errors.len(),
            1,
            "une seule error d'échéance, reçu {errors:?} via {events:?}"
        );
        assert_eq!(
            errors[0]["payload"]["reason"].as_str(),
            Some("échéance Codex dépassée"),
            "raison en dur, reçu {}",
            errors[0]["payload"]
        );
        assert_eq!(
            errors[0]["payload"]["terminal_kind"].as_str(),
            Some(crate::journal::TURN_FAILED_KIND),
            "échéance DOIT porter terminal_kind=turn_failed sinon la ronde classe OCCUPE; reçu {}",
            errors[0]["payload"]
        );
        assert_eq!(
            errors[0]["message_id"].as_str(),
            Some("codex-echeance-1"),
            "même message_id, reçu {errors:?}"
        );
        let turn_ends: Vec<_> = events
            .iter()
            .filter(|event| event["event"] == "turn_end")
            .collect();
        assert!(
            turn_ends.is_empty(),
            "échéance SANS turn_end (≠ succès Claude à payload vide) ; reçu {turn_ends:?}"
        );
    }

    /// (TOOL) Commande Codex → update kind=command avec le NOM réel
    /// (`item.command`), pas la sortie stdout. Source mesurée : item/started
    /// type=commandExecution. Mutant : remettre outputDelta en texte d'acte,
    /// ou vider command → meurt.
    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_TOOL_codex_app_server_retranscrit_command_execution_nom() {
        let (present, _outbound) = journal_detail_fixture("temoin-tool-command", true);
        let commands: Vec<_> = present
            .iter()
            .filter(|event| event["event"] == "update" && event["payload"]["kind"] == "command")
            .collect();
        assert_eq!(
            commands.len(),
            1,
            "un seul acte command attendu (pas les outputDelta), reçu {commands:?}"
        );
        assert_eq!(
            commands[0]["payload"]["text"].as_str(),
            Some("echo MESURE_CODEX_CMD_77"),
            "commande en dur (pas la sortie), payload={}",
            commands[0]["payload"]
        );
        assert_eq!(
            commands[0]["payload"]["detail"].as_str(),
            Some("item/started"),
            "provenance item/started en dur, payload={}",
            commands[0]["payload"]
        );
        assert!(
            commands
                .iter()
                .all(|event| event["payload"]["text"] != "313 passés"),
            "un mutant qui rejoue outputDelta comme nom laisserait 313 passés"
        );
    }

    #[test]
    fn journal_codex_atteste_presence_puis_absence_et_ne_valide_pas_approbation() {
        let (present, outbound) = journal_detail_fixture("detail-present", true);
        let reasoning = present
            .iter()
            .filter(|event| event["event"] == "reasoning")
            .collect::<Vec<_>>();
        assert_eq!(reasoning.len(), 1, "une seule attestation terminale");
        assert_eq!(
            reasoning[0]["payload"],
            json!({
                "available": true,
                "summary": "Je compare les options.",
                "raw": "raison brute"
            })
        );
        let reasoning_index = present
            .iter()
            .position(|event| event["event"] == "reasoning")
            .expect("attestation de présence");
        let turn_end_index = present
            .iter()
            .position(|event| event["event"] == "turn_end")
            .expect("fin de tour présente");
        assert_eq!(
            reasoning_index + 1,
            turn_end_index,
            "le raisonnement n'est attesté qu'au terminal"
        );
        for expected in [
            json!({"kind":"command", "text":"echo MESURE_CODEX_CMD_77", "detail":"item/started"}),
            json!({"kind":"file", "text":"src/main.rs", "detail":"update"}),
            json!({"kind":"plan", "text":"Tester le flux"}),
            json!({
                "kind":"approval",
                "text":"cargo test -p bridget-transport",
                "detail":"item/commandExecution/requestApproval"
            }),
            json!({
                "kind":"approval",
                "text":"écrire le fichier",
                "detail":"item/fileChange/requestApproval"
            }),
            json!({
                "kind":"approval",
                "text":"accès réseau",
                "detail":"item/permissions/requestApproval"
            }),
        ] {
            assert!(
                present
                    .iter()
                    .any(|event| event["event"] == "update" && event["payload"] == expected),
                "acte Codex absent du journal: {expected}"
            );
        }
        assert!(
            present.iter().all(|event| {
                !(event["event"] == "update"
                    && event["payload"]["kind"] == "command"
                    && event["payload"]["text"] == "313 passés")
            }),
            "outputDelta ne doit plus être journalisé comme nom de commande"
        );
        assert_eq!(
            present
                .iter()
                .filter(|event| {
                    event["event"] == "update" && event["payload"]["kind"] != "text"
                })
                .count(),
            6,
            "une notification inconnue doit rester un événement système inerte"
        );
        for approval_id in ["approval-7", "approval-8", "approval-9"] {
            assert!(
                outbound.iter().any(|frame| {
                    frame["id"].as_str() == Some(approval_id)
                        && frame["result"]["decision"].as_str() == Some("accept")
                }),
                "décision JSON-RPC absente pour {approval_id}: {outbound:?}"
            );
        }

        let (absent, _) = journal_detail_fixture("detail-absent", false);
        let reasoning = absent
            .iter()
            .filter(|event| event["event"] == "reasoning")
            .collect::<Vec<_>>();
        assert_eq!(
            reasoning.len(),
            1,
            "l'absence n'est attestée qu'au terminal"
        );
        assert_eq!(reasoning[0]["payload"], json!({"available": false}));
        let reasoning_index = absent
            .iter()
            .position(|event| event["event"] == "reasoning")
            .expect("attestation d'absence");
        let turn_end_index = absent
            .iter()
            .position(|event| event["event"] == "turn_end")
            .expect("fin du tour sans raisonnement");
        assert_eq!(
            reasoning_index + 1,
            turn_end_index,
            "un silence intermédiaire ne doit jamais attester l'absence"
        );
    }

    #[test]
    fn approval_response_choisit_accept_ou_decline_selon_permissions() {
        let request = json!({
            "id": "approval-unit",
            "method": "item/commandExecution/requestApproval",
            "params": { "command": "git worktree add", "turnId": "t1" }
        });
        let (allow_reply, allow_payload) = approval_response(&request, "allow").expect("allow");
        assert_eq!(
            allow_reply,
            json!({ "id": "approval-unit", "result": { "decision": "accept" } }),
            "attente en dur: allow → accept"
        );
        assert_eq!(allow_payload["decision"], "accept");
        let (deny_reply, deny_payload) = approval_response(&request, "deny").expect("deny");
        assert_eq!(
            deny_reply,
            json!({ "id": "approval-unit", "result": { "decision": "decline" } }),
            "attente en dur: deny → decline"
        );
        assert_eq!(deny_payload["decision"], "decline");
        assert!(
            approval_response(
                &json!({ "method": "item/commandExecution/requestApproval" }),
                "allow"
            )
            .is_none(),
            "sans id: aucune réponse inventée"
        );
    }

    /// ORACLE PRESENCE + EFFET dans le même corps.
    /// Présence : une réponse JSON-RPC portant `decision` est émise (trace non vide).
    /// Effet : le tour se poursuit après cette réponse (texte + Completed).
    /// Attentes écrites en dur — jamais un appel à `approval_response`.
    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_codex_approbation_repond_et_le_tour_se_poursuit() {
        let root = root("approval-presence-effet");
        let trace = root.join("trace.jsonl");
        let mut transport = CodexAppServerTransport::spawn_with_environment(
            fake_options(&trace),
            &[
                (
                    "BRIDGET_CODEX_TRACE".to_string(),
                    trace.to_string_lossy().into_owned(),
                ),
                (
                    "BRIDGET_CODEX_REQUEST_APPROVAL".to_string(),
                    "1".to_string(),
                ),
            ],
            false,
        )
        .expect("session native");
        transport
            .activate_journal(&root, "codex-native", None)
            .expect("journal activé");
        transport
            .deliver(&message("approval-presence-effet"))
            .expect("livraison");

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut finished = false;
        let mut saw_continuation = false;
        while Instant::now() < deadline {
            for event in transport.drain_events() {
                if let ManagedEventKind::TurnFinished {
                    terminal: ManagedTerminal::Completed,
                    response,
                    ..
                } = &event.kind
                {
                    finished = true;
                    saw_continuation = response.contains("tour poursuivi apres autorisation");
                }
            }
            if finished {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        transport.stop();

        let frames: Vec<Value> = fs::read_to_string(&trace)
            .expect("trace fournisseur")
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();
        // ORACLE PRESENCE — non-vacuité : au moins une décision émise, id repris.
        let decisions: Vec<_> = frames
            .iter()
            .filter(|frame| {
                frame["id"].as_str() == Some("approval-native")
                    && frame.get("result").is_some()
                    && frame.get("method").is_none()
            })
            .collect();
        assert!(
            !decisions.is_empty(),
            "PRESENCE: aucune réponse d'approbation dans la trace (vide)"
        );
        assert_eq!(
            decisions[0]["result"]["decision"].as_str(),
            Some("accept"),
            "PRESENCE: décision attendue en dur accept, reçu {}",
            decisions[0]
        );

        let journal_path = fs::read_dir(root.join("codex-native"))
            .expect("répertoire journal")
            .next()
            .expect("fichier journal")
            .expect("entrée journal")
            .path();
        let entries = crate::journal::valid_events(&journal_path);
        let permission = entries
            .iter()
            .find(|entry| entry["event"] == "permission")
            .expect("PRESENCE: événement permission journalisé");
        assert_eq!(
            permission["payload"]["decision"].as_str(),
            Some("accept"),
            "PRESENCE journal: décision en dur accept"
        );
        assert_eq!(
            permission["message_id"].as_str(),
            Some("approval-presence-effet"),
            "PRESENCE: message_id non vide"
        );

        // ORACLE EFFET — le tour continue après la réponse (pas une garde morte).
        assert!(
            finished,
            "EFFET: le tour doit atteindre Completed après la décision"
        );
        assert!(
            saw_continuation,
            "EFFET: texte de poursuite absent — le faux fournisseur n'a pas repris après décision"
        );
        let decision_pos = frames
            .iter()
            .position(|frame| {
                frame["id"].as_str() == Some("approval-native")
                    && frame["result"]["decision"].as_str() == Some("accept")
            })
            .expect("position de la décision");
        // La poursuite est côté stdout du faux (pas dans TRACE stdin). On vérifie
        // l'ordre journal : permission avant turn_end.
        let permission_seq = permission["seq"].as_u64().expect("seq permission");
        let turn_end = entries
            .iter()
            .find(|entry| entry["event"] == "turn_end")
            .expect("EFFET: turn_end après poursuite");
        assert!(
            permission_seq < turn_end["seq"].as_u64().expect("seq turn_end"),
            "EFFET: permission doit précéder turn_end (décision puis poursuite)"
        );
        assert!(
            decision_pos < frames.len(),
            "PRESENCE: index de décision dans un univers non vide"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn passerelle_codex_marque_seulement_les_erreurs_terminales() {
        let terminal = codex_journal_payload(
            "error",
            json!({
                "reason": "échéance Codex dépassée",
                "pending_provider_request": {"state": "pending"}
            }),
        );
        assert_eq!(terminal["terminal_kind"], crate::journal::TURN_FAILED_KIND);
        assert_eq!(
            terminal["pending_provider_request"],
            json!({"state": "pending"})
        );

        let update = codex_journal_payload("update", json!({"kind": "text"}));
        assert!(update.get("terminal_kind").is_none());

        let malformed = codex_journal_payload("error", json!("payload invalide"));
        assert_eq!(malformed, json!("payload invalide"));
    }

    #[test]
    fn session_native_respecte_sequence_saturation_et_octets_sources() {
        let root = root("sequence");
        let trace = root.join("trace.jsonl");
        let mut transport = CodexAppServerTransport::spawn_with_environment(
            fake_options(&trace),
            &[(
                "BRIDGET_CODEX_TRACE".to_string(),
                trace.to_string_lossy().into_owned(),
            )],
            false,
        )
        .expect("session native");
        let feed = JournalLiveFeed::default();
        transport
            .activate_journal(&root, "codex-native", Some(feed))
            .expect("journal activé");
        let mut sent = message("native-1");
        sent.reply = true;
        sent.body = "<!doctype html>\n<script>const texte = 'été';</script>\n".to_string();
        transport.deliver(&sent).expect("livraison");

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events.iter().any(|event| matches!(
                event.kind,
                ManagedEventKind::TurnFinished { ref response, .. } if response == "réponse native"
            )) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(events.iter().any(|event| matches!(
            event.kind,
            ManagedEventKind::TurnFinished { ref response, terminal: ManagedTerminal::Completed, .. }
                if response == "réponse native"
        )));
        assert!(events.iter().any(|event| matches!(
            event.kind,
            ManagedEventKind::RuntimeObserved { ref model, effort: Some(ref effort) }
                if model == "gpt-5.6-terra" && effort == "high"
        )));
        assert!(events.iter().any(|event| matches!(
            event.kind,
            ManagedEventKind::RateLimitObserved {
                ref window,
                ref status,
                resets_at: Some(1_787_572_200),
                used_percent: Some(42),
            } if window == "primary/300m" && status == "available"
        )));
        let raw = events.iter().find(|event| {
            matches!(event.kind, ManagedEventKind::Update { .. })
                && event
                    .raw
                    .starts_with(b"{  \"method\" : \"item/agentMessage/delta\"")
        });
        let raw = raw.expect("notification native source");
        assert_eq!(raw.origin, ManagedEventOrigin::SourceLine);
        assert_eq!(raw.source, ManagedEventSource::CodexAppServer);
        // Oracle externe, extrait du faux fournisseur historique (dfa2134).
        // Le lecteur retire seulement le LF JSONL : aucun parse/réencodage
        // ne peut remplacer les espaces ni les octets UTF-8 du fournisseur.
        assert_eq!(
            raw.raw.as_slice(),
            include_bytes!("../../../fixtures/native-codex-delta.jsonl")
                .strip_suffix(b"\n")
                .expect("fixture JSONL terminée par LF")
        );

        let trace = fs::read_to_string(&trace).expect("trace fournisseur");
        let frames = trace.lines().collect::<Vec<_>>();
        assert_eq!(
            frames
                .iter()
                .filter_map(|line| serde_json::from_str::<Value>(line).ok())
                .filter_map(|frame| frame
                    .get("method")
                    .and_then(Value::as_str)
                    .map(str::to_owned))
                .collect::<Vec<_>>(),
            vec![
                "initialize",
                "initialized",
                "thread/start",
                "account/rateLimits/read",
                "turn/start",
                "turn/start"
            ]
        );
        assert!(frames.iter().all(|frame| !frame.contains("jsonrpc")));
        // Oracle au destinataire : enveloppe complète et corps inchangé,
        // sans ordre de rendu/publication ajouté au contenu HTML.
        let turns = frames
            .iter()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .filter(|frame| frame["method"] == "turn/start")
            .collect::<Vec<_>>();
        assert_eq!(turns.len(), 2);
        for turn in turns {
            let prompt = turn["params"]["input"][0]["text"].as_str().unwrap();
            let (header, rest) = prompt.split_once('\n').unwrap();
            let envelope: Value = serde_json::from_str(
                header
                    .strip_prefix("[Message Bridget : ")
                    .unwrap()
                    .strip_suffix(']')
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(
                envelope,
                json!({"from": sent.from, "to": sent.to, "id": sent.id, "reply": true, "in_reply_to": null})
            );
            assert_eq!(
                rest.split_once("\n\n[Réponse Bridget]\n")
                    .unwrap()
                    .0
                    .as_bytes(),
                sent.body.as_bytes()
            );
            assert!(rest.contains("le wrapper relaie automatiquement ta réponse finale"));
            assert!(rest.contains("catalogue d'outils disponible"));
            assert!(!rest.contains("kind:html"));
        }
        transport.stop();
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn absence_de_signaux_codex_ne_cree_aucun_fait() {
        assert_eq!(runtime_from_thread_start(&json!({})), None);
        assert!(rate_limits_from_snapshot(None).is_empty());
        assert!(rate_limits_from_snapshot(Some(&json!({ "primary": null }))).is_empty());
    }

    #[test]
    fn snapshot_codex_emet_primary_et_secondary_sans_ecrasement() {
        let facts = rate_limits_from_snapshot(Some(&json!({
            "primary": {
                "usedPercent": 19,
                "windowDurationMins": 300,
                "resetsAt": 1_787_572_200
            },
            "secondary": {
                "usedPercent": 61,
                "windowDurationMins": 10080,
                "resetsAt": 1_787_700_000
            },
            "rateLimitReachedType": null
        })));
        assert_eq!(facts.len(), 2);
        assert_eq!(
            facts[0],
            (
                "primary/300m".to_string(),
                "available".to_string(),
                Some(1_787_572_200),
                Some(19)
            )
        );
        assert_eq!(
            facts[1],
            (
                "secondary/10080m".to_string(),
                "available".to_string(),
                Some(1_787_700_000),
                Some(61)
            )
        );
    }

    #[test]
    fn percent_hors_bornes_conserve_la_fenetre_sans_pourcent() {
        // Mandat : % omis, fenêtre conservée — pas d'effacement via `?`.
        let facts = rate_limits_from_snapshot(Some(&json!({
            "primary": {
                "usedPercent": 250,
                "windowDurationMins": 300,
                "resetsAt": 1_787_572_200
            },
            "rateLimitReachedType": null
        })));
        assert_eq!(
            facts.len(),
            1,
            "la fenêtre doit survivre à un % hors bornes"
        );
        assert_eq!(facts[0].0, "primary/300m");
        assert_eq!(facts[0].1, "available");
        assert_eq!(facts[0].2, Some(1_787_572_200));
        assert_eq!(facts[0].3, None, "% omis, pas fenêtre effacée");
    }

    #[test]
    fn annulation_interrompt_le_tour_natif() {
        let root = root("cancel");
        let trace = root.join("trace.jsonl");
        let mut transport = CodexAppServerTransport::spawn_with_environment(
            fake_options(&trace),
            &[
                (
                    "BRIDGET_CODEX_TRACE".to_string(),
                    trace.to_string_lossy().into_owned(),
                ),
                ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
            ],
            false,
        )
        .expect("session native");
        let sent = message("cancel-1");
        transport.deliver(&sent).expect("livraison");
        let deadline = Instant::now() + Duration::from_secs(1);
        while Instant::now() < deadline && !transport.is_busy() {
            thread::sleep(Duration::from_millis(5));
        }
        assert!(
            transport.cancel_delivery("cancel-1", "annulation test"),
            "annulation absente; trace={}; events={:?}",
            fs::read_to_string(&trace).unwrap_or_else(|error| error.to_string()),
            transport.drain_events(),
        );
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events.iter().any(|event| {
                matches!(
                    event.kind,
                    ManagedEventKind::TurnFinished {
                        terminal: ManagedTerminal::Cancelled,
                        ..
                    }
                )
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(events.iter().any(|event| matches!(
            event.kind,
            ManagedEventKind::TurnFinished {
                terminal: ManagedTerminal::Cancelled,
                ..
            }
        )));
        transport.stop();
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn eof_pendant_un_tour_reveille_le_worker_et_nettoie_le_groupe() {
        let root = root("eof-turn");
        let trace = root.join("trace.jsonl");
        let mut options = fake_options(&trace);
        // L'ancien worker attendait jusqu'à l'échéance du tour : cette valeur
        // rend le mutant visible sans faire dormir le test pendant une minute.
        options.notify_timeout_secs = 60;
        let mut transport = CodexAppServerTransport::spawn_with_environment(
            options,
            &[
                (
                    "BRIDGET_CODEX_TRACE".to_string(),
                    trace.to_string_lossy().into_owned(),
                ),
                ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
            ],
            false,
        )
        .expect("session native");
        transport.deliver(&message("eof-1")).expect("livraison");

        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(2) {
            let turns_started = fs::read_to_string(&trace)
                .unwrap_or_default()
                .lines()
                .filter(|line| line.contains("\"method\":\"turn/start\""))
                .count();
            if turns_started >= 2 {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        assert!(
            fs::read_to_string(&trace)
                .unwrap_or_default()
                .lines()
                .filter(|line| line.contains("\"method\":\"turn/start\""))
                .count()
                >= 2,
            "le tour suspendu n'a jamais atteint wait_for_turn"
        );
        let adapter_pid = transport.process_id() as i32;

        assert_eq!(unsafe { libc::kill(adapter_pid, libc::SIGTERM) }, 0);
        let deadline = Instant::now() + Duration::from_secs(1);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events.iter().any(|event| {
                matches!(
                    event.kind,
                    ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
                        if message_id == "eof-1" && reason == "stdout Codex fermé pendant le tour"
                )
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        assert!(
            events.iter().any(|event| matches!(
                event.kind,
                ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
                    if message_id == "eof-1" && reason == "stdout Codex fermé pendant le tour"
            )),
            "EOF pendant le tour n'a pas terminé le worker: {events:?}"
        );

        transport.stop();
        assert_eq!(unsafe { libc::kill(adapter_pid, 0) }, -1);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn stop_pendant_un_tour_termine_le_groupe_enfant() {
        let root = root("stop-group");
        let trace = root.join("trace.jsonl");
        let child_pid_path = root.join("child.pid");
        let mut transport = CodexAppServerTransport::spawn_with_environment(
            fake_options(&trace),
            &[
                (
                    "BRIDGET_CODEX_TRACE".to_string(),
                    trace.to_string_lossy().into_owned(),
                ),
                ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
                (
                    "BRIDGET_CODEX_CHILD_PID".to_string(),
                    child_pid_path.to_string_lossy().into_owned(),
                ),
            ],
            false,
        )
        .expect("session native");
        transport.deliver(&message("stop-1")).expect("livraison");
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline && !child_pid_path.is_file() {
            thread::sleep(Duration::from_millis(5));
        }
        let child_pid = fs::read_to_string(&child_pid_path)
            .expect("descendant Codex")
            .trim()
            .parse::<i32>()
            .expect("pid descendant");

        let started = Instant::now();
        transport.stop();
        assert!(
            started.elapsed() < Duration::from_secs(1),
            "stop attend encore l'échéance du tour"
        );
        let reaped_deadline = Instant::now() + Duration::from_secs(1);
        while Instant::now() < reaped_deadline && unsafe { libc::kill(child_pid, 0) } == 0 {
            thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(
            unsafe { libc::kill(child_pid, 0) },
            -1,
            "le groupe enfant Codex survit au shutdown"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn reroutage_codex_atteste_le_modele_sans_inventer_si_muet() {
        let rerouted: Value =
            serde_json::from_str(r#"{"method":"modelRerouted","params":{"to":"gpt-5.4"}}"#)
                .unwrap();
        assert_eq!(
            served_model_from_codex(&rerouted).as_deref(),
            Some("gpt-5.4")
        );
        let mute: Value =
            serde_json::from_str(r#"{"method":"modelRerouted","params":{}}"#).unwrap();
        assert!(served_model_from_codex(&mute).is_none());
    }

    #[test]
    fn reprise_et_bifurcation_codex_emettent_la_requete_attestee_ou_refusent() {
        let compatible = ProviderObservation {
            binary_path: "/fixtures/codex".to_string(),
            binary_version: "0.150.1".to_string(),
            binary_digest: "0".repeat(64),
            contract_version: "codex-app-server-fixture-v1".to_string(),
            operations: vec![ProviderOperation::Resume, ProviderOperation::Fork],
        };
        let cwd = Path::new("/workspace");
        let resume = CodexThreadBootstrap::Resume {
            thread_id: "thread-parent".to_string(),
        };
        let mut interactive = fake_options(Path::new("/tmp/non-execute"));
        interactive.thread_bootstrap = resume.clone();
        assert!(validate_endpoint_bootstrap(&interactive, true).is_ok());
        assert!(validate_endpoint_bootstrap(&interactive, false).is_err());
        assert_eq!(
            thread_bootstrap_request(&resume, cwd, Some("explicite"), true).1,
            json!({"threadId": "thread-parent", "model": "explicite", "excludeTurns": true})
        );
        let fork = CodexThreadBootstrap::Fork {
            thread_id: "thread-parent".to_string(),
        };
        assert!(validate_thread_bootstrap(&Some(compatible.clone()), &resume).is_ok());
        assert!(validate_thread_bootstrap(&Some(compatible.clone()), &fork).is_ok());
        assert_eq!(
            thread_bootstrap_request(&resume, cwd, None, false),
            ("thread/resume", json!({ "threadId": "thread-parent" }))
        );
        assert_eq!(
            thread_bootstrap_request(&fork, cwd, None, false),
            ("thread/fork", json!({ "threadId": "thread-parent" }))
        );
        assert!(matches!(
            validate_thread_bootstrap(&None, &resume),
            Err(TransportError::DeliveryFailed(reason)) if reason.contains("thread/resume")
        ));
        let fork_incompatible = ProviderObservation {
            operations: vec![ProviderOperation::Resume],
            ..compatible
        };
        assert!(matches!(
            validate_thread_bootstrap(&Some(fork_incompatible), &fork),
            Err(TransportError::DeliveryFailed(reason)) if reason.contains("thread/fork")
        ));
    }

    #[test]
    fn reprise_geree_projette_les_restrictions_effectives_ou_refuse_avant_le_fil() {
        for sandbox in ["read-only", "workspace-write", "missing"] {
            for bootstrap in [
                CodexThreadBootstrap::Start,
                CodexThreadBootstrap::Resume {
                    thread_id: "thread-native".into(),
                },
                CodexThreadBootstrap::Fork {
                    thread_id: "thread-native".into(),
                },
            ] {
                let root = root("resume-policy");
                let trace = root.join("trace.jsonl");
                let is_start = matches!(bootstrap, CodexThreadBootstrap::Start);
                let mut options = fake_options(&trace);
                options.permissions = "deny".into();
                options.thread_bootstrap = bootstrap;
                options.provider_observation = Some(ProviderObservation {
                    binary_path: "/fixtures/codex".into(),
                    binary_version: "fixture".into(),
                    binary_digest: "0".repeat(64),
                    contract_version: "codex-app-server-fixture-v1".into(),
                    operations: vec![ProviderOperation::Resume, ProviderOperation::Fork],
                });
                options.args = vec!["-c".into(), r#"
                    while IFS= read -r line; do
                        printf '%s\n' "$line" >> "$BRIDGET_CODEX_TRACE"
                        id=$(printf '%s' "$line" | sed 's/.*"id":\([0-9]*\).*/\1/')
                        case "$line" in
                            *'"method":"initialize"'*) printf '%s\n' '{"id":1,"result":{"userAgent":"fake","codexHome":"/tmp","platformFamily":"unix","platformOs":"macos"}}' ;;
                            *'"method":"config/read"'*)
                                if [ "$POLICY_SANDBOX" = missing ]; then
                                    printf '{"id":%s,"result":{"config":{}}}\n' "$id"
                                else
                                    printf '{"id":%s,"result":{"config":{"approval_policy":"never","sandbox_mode":"%s"}}}\n' "$id" "$POLICY_SANDBOX"
                                fi ;;
                            *'"method":"thread/start"'*|*'"method":"thread/resume"'*|*'"method":"thread/fork"'*) printf '{"id":%s,"result":{"thread":{"id":"thread-native"}}}\n' "$id" ;;
                            *'"method":"account/rateLimits/read"'*) printf '{"id":%s,"result":{}}\n' "$id" ;;
                        esac
                    done
                "#.into()];
                let result = CodexAppServerTransport::spawn_with_environment(
                    options,
                    &[
                        (
                            "BRIDGET_CODEX_TRACE".into(),
                            trace.to_string_lossy().into_owned(),
                        ),
                        ("POLICY_SANDBOX".into(), sandbox.into()),
                    ],
                    false,
                );
                if sandbox == "missing" && !is_start {
                    assert!(
                        matches!(result, Err(TransportError::DeliveryFailed(ref reason)) if reason.contains("configuration effective approval_policy absente"))
                    );
                } else {
                    result.expect("transport restreint").stop();
                }
                let frames: Vec<Value> = fs::read_to_string(&trace)
                    .unwrap()
                    .lines()
                    .map(|line| serde_json::from_str(line).unwrap())
                    .collect();
                let thread = frames.iter().find(|frame| {
                    frame["method"]
                        .as_str()
                        .is_some_and(|method| method.starts_with("thread/"))
                });
                if sandbox == "missing" && !is_start {
                    assert!(
                        thread.is_none(),
                        "aucun fil ne démarre sans politique attestée"
                    );
                } else {
                    let params = &thread.unwrap()["params"];
                    assert_eq!(params["cwd"], json!(std::env::current_dir().unwrap()));
                    if is_start {
                        assert!(!frames.iter().any(|frame| frame["method"] == "config/read"));
                    } else {
                        assert_eq!(params["sandbox"], sandbox);
                        assert_eq!(params["approvalPolicy"], "never");
                    }
                }
                fs::remove_dir_all(root).unwrap();
            }
        }
    }
    #[test]
    fn consigne_privee_preserve_le_corps_visible_du_message() {
        let body = "Demande utilisateur visible.";
        let prompt = private_prompt(Some("Privilégie les sources attestées."), body);

        assert!(prompt.contains("Privilégie les sources attestées."));
        assert!(prompt.ends_with(body));
        assert_eq!(body, "Demande utilisateur visible.");
        assert_eq!(private_prompt(None, body), body);
    }

    #[test]
    fn consigne_reponse_distingue_mcp_interactif_et_relais_gere() {
        let mut message = message("demande-liée");
        message.in_reply_to = Some("parent".into());
        for interactive in [false, true] {
            let prompt = communication_prompt(&message, interactive);
            assert!(prompt.contains("\"in_reply_to\":\"parent\""));
            assert_eq!(
                prompt.contains("réponds une seule fois par l'outil bridget_send"),
                interactive
            );
            assert_eq!(
                prompt.contains("le wrapper relaie automatiquement"),
                !interactive
            );
            assert!(prompt.contains("un refus du shell ne prouve pas une panne MCP"));
        }
    }

    #[test]
    fn spec105_codex_sans_demande_ne_prescrit_aucun_envoi() {
        for interactive in [false, true] {
            let mut message = message("notification");
            message.reply = false;
            message.in_reply_to = Some("question".into());
            let prompt = communication_prompt(&message, interactive);
            assert!(prompt.contains("Aucune réponse inter-agent attendue"));
            assert!(!prompt.contains("réponds une seule fois"));
            assert!(!prompt.contains("relaie automatiquement"));
            assert!(prompt.contains(&message.body));
        }
    }

    #[test]
    fn document_html_reste_un_corps_sans_consigne_ui_implicite() {
        let body = "<html>  <script>window.contenu = 'é';</script>\n</html>";
        assert_eq!(private_prompt(None, body).as_bytes(), body.as_bytes());
        assert_eq!(private_prompt(Some("  "), body).as_bytes(), body.as_bytes());
    }

    #[test]
    fn outil_dynamique_codex_rejoint_lexecuteur_bridget_atteste() {
        let request = json!({
            "id": "dynamic-1",
            "method": "item/tool/call",
            "params": {
                "tool": "bridget_publish_artifact",
                "arguments": { "kind": "html" },
            }
        });
        let handler: DynamicToolHandler = Arc::new(|tool, arguments| {
            assert_eq!(tool, "bridget_publish_artifact");
            assert_eq!(arguments["kind"], "html");
            Ok(json!({ "status": "published", "receipt": "artifact-1" }))
        });

        let (reply, detail) = dynamic_tool_response(&request, Some(&handler)).expect("réponse");
        assert_eq!(detail, "publication d’artefact Bridget");
        assert_eq!(reply["result"]["success"], true);
        assert!(
            reply["result"]["contentItems"][0]["text"]
                .as_str()
                .is_some_and(|text| text.contains("published"))
        );
    }

    #[test]
    fn outil_dynamique_inconnu_est_refuse_explicitement() {
        let request = json!({
            "id": "dynamic-2",
            "method": "item/tool/call",
            "params": { "tool": "arbitrary_tool", "arguments": {} }
        });
        let handler: DynamicToolHandler =
            Arc::new(|_, _| panic!("outil inconnu ne doit jamais atteindre l'exécuteur"));

        let (reply, detail) = dynamic_tool_response(&request, Some(&handler)).expect("réponse");
        assert_eq!(detail, "outil dynamique Bridget refusé");
        assert_eq!(reply["result"]["success"], false);
    }

    #[test]
    fn outil_dynamique_lecture_conserve_reference_et_document_inerte() {
        let arguments = json!({"artifact_id": "artifact-089", "version": 1});
        let document = "<!doctype html>\n<script>alerte('é');</script>\n";
        let expected = json!({"status":"read", "artifact_id":"artifact-089", "content":document});
        let expected_arguments = arguments.clone();
        let result = expected.clone();
        let handler: DynamicToolHandler = Arc::new(move |tool, actual| {
            assert_eq!(tool, "bridget_read_artifact");
            assert_eq!(actual, &expected_arguments);
            Ok(result.clone())
        });
        let request = json!({"id":"read-089", "method":"item/tool/call", "params":{
            "tool":"bridget_read_artifact", "arguments":arguments
        }});
        let (reply, detail) = dynamic_tool_response(&request, Some(&handler)).unwrap();
        assert_eq!(detail, "lecture d’artefact Bridget");
        assert_eq!(reply["id"], "read-089");
        assert_eq!(reply["result"]["success"], true);
        assert_eq!(reply["result"]["contentItems"][0]["type"], "inputText");
        let content = reply["result"]["contentItems"][0]["text"].as_str().unwrap();
        assert_eq!(serde_json::from_str::<Value>(content).unwrap(), expected);
    }

    #[test]
    fn outil_dynamique_lecture_sans_autorite_ou_en_echec_ne_pretend_pas_reussir() {
        let request = json!({"id":89, "method":"item/tool/call", "params":{
            "tool":"bridget_read_artifact", "arguments":{"artifact_id":"absent"}
        }});
        let (reply, _) = dynamic_tool_response(&request, None).unwrap();
        assert_eq!(reply["result"]["success"], false);
        let handler: DynamicToolHandler = Arc::new(|_, _| Err("SECRET_FIXTURE_089".to_string()));
        let (reply, _) = dynamic_tool_response(&request, Some(&handler)).unwrap();
        assert_eq!(reply["result"]["success"], false);
        assert!(!reply.to_string().contains("SECRET_FIXTURE_089"));
    }

    #[test]
    fn elicitation_bridget_formulaire_implicite_est_acceptee() {
        let request = json!({
            "id": "elicitation-1",
            "method": "mcpServer/elicitation/request\n",
            "params": {
                "serverName": "bridget",
                "request": {
                    "method": "elicitation/create",
                    "params": {
                        "message": "Publication prête.",
                        "requestedSchema": { "type": "object", "properties": {} }
                    }
                }
            }
        });

        assert!(is_mcp_elicitation_request(
            request["method"].as_str().expect("méthode")
        ));
        let (reply, payload) = mcp_elicitation_response(&request).expect("réponse MCP");
        assert_eq!(reply["result"]["action"], "accept");
        assert_eq!(reply["result"]["content"], json!({}));
        assert_eq!(payload["code"], "mcp_elicitation_autoaccepted");
    }

    #[test]
    fn elicitation_mcp_externe_ou_avec_champ_requis_est_declinee() {
        let request = json!({
            "id": "elicitation-2",
            "method": "mcpServer/elicitation/request",
            "params": {
                "serverName": "externe",
                "requestedSchema": {
                    "type": "object",
                    "properties": { "secret": { "type": "string" } },
                    "required": ["secret"]
                }
            }
        });

        let (reply, payload) = mcp_elicitation_response(&request).expect("réponse MCP");
        assert_eq!(reply["result"]["action"], "decline");
        assert!(reply["result"].get("content").is_none());
        assert_eq!(payload["code"], "mcp_elicitation_declined");
    }
}
