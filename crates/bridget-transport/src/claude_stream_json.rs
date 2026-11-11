//! Pilote natif Claude Code en JSONL `stream-json`.
//!
//! Les lignes du fournisseur restent brutes jusqu'à la frontière commune. Ce
//! module ne projette ni modèle, ni quota, ni consommation : G5 ne couvre que
//! la session gérée, le journal et son cycle de vie.

use crate::claude_provider_session::{
    ProviderSessionStore, classify_resume_failure, prepare_launch_args,
    session_id_from_system_init, strip_resume_arg,
};
use crate::journal::{JournalFailureSink, JournalLiveFeed, JournalWriter, with_turn_failed_kind};
use crate::managed_session::{
    ManagedEvent, ManagedEventKind, ManagedEventOrigin, ManagedEventSource,
    ManagedProviderIdentity, ManagedSession, ManagedSessionDescriptor, ManagedTerminal,
    stop_owned_child,
};
use crate::poison::RecoverPoison;
use crate::protocol::{
    ChildProviderRequest, ChildProviderRequestId, ChildRequestAnswer, ChildRequestDecision,
    ChildRequestKind, PresenceMode, ProviderObservation,
};
use crate::transport::{Transport, TransportError};
use bridget_core::{BridgetMessage, MessageIntent, MessageOrigin};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{HashSet, VecDeque};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Délai absolu daemon avant Connected (`DEFAULT_SPAWN_TIMEOUT_SECS` dans cli).
pub const DAEMON_SPAWN_TIMEOUT: Duration = Duration::from_secs(10);
pub const MAX_ISOLATED_DECISION_STDOUT_BYTES: usize = 64 * 1024;
pub const MAX_ISOLATED_DECISION_JSON_BYTES: usize = 4096;

/// Invocation Claude168 sans session, journal ni Task Bridget. Le seul outil
/// toléré est le formateur interne `StructuredOutput` imposé par json-schema.
#[derive(Debug, Clone)]
pub struct ClaudeIsolatedDecisionOptions {
    pub command: String,
    /// Arguments de lanceur attestés par le registre (vides pour Claude CLI).
    pub command_args: Vec<String>,
    pub model: String,
    pub environment: Vec<(String, String)>,
    pub request_json: String,
    /// Échéance absolue partagée avec la chaîne, préparation et scheduling inclus.
    pub deadline: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaudeIsolatedDecisionResult {
    pub candidate_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaudeIsolatedDecisionError {
    InvalidInput,
    Unavailable,
    OutputTooLarge,
    Timeout,
    InvalidOutput,
}

/// Contexte de validation `--resume` déplacée hors du chemin spawn (reader).
struct ResumeBootstrap {
    attempted_id: String,
    command: String,
    args: Vec<String>,
    environment: Vec<(String, String)>,
    inherit_stderr: bool,
    writer: Writer,
    child: Arc<Mutex<Child>>,
}

type Writer = Arc<Mutex<Option<ChildStdin>>>;
type Journal = Arc<Mutex<Option<JournalWriter>>>;
type SessionStoreHandle = Arc<Mutex<Option<ProviderSessionStore>>>;

#[derive(Debug, Clone)]
pub struct ClaudeStreamJsonOptions {
    pub command: String,
    /// Arguments de définition, dont l'éventuel `--model` demandé.
    pub args: Vec<String>,
    /// Type déclaré par le registre, conservé dans les événements et les contextes.
    pub provider_kind: String,
    pub queue_capacity: usize,
    pub notify_timeout_secs: u64,
    /// Racine durable `~/.cache/bridget/sessions` (même arbre que le journal).
    /// Survit à la mort du wrapper et au redémarrage du daemon.
    pub session_store_root: Option<PathBuf>,
    /// Baseline relevée avant lancement, absente pour un registre historique.
    pub provider_observation: Option<ProviderObservation>,
    /// Nom d'agent sous lequel lire/écrire `claude_provider_session`.
    pub agent_name: Option<String>,
}

#[derive(Default)]
struct BackgroundTasks {
    live: HashSet<String>,
    unconfirmed: bool,
}

impl BackgroundTasks {
    fn observe(&mut self, value: &Value) {
        if value["type"] != "system" {
            return;
        }
        let subtype = value["subtype"].as_str().unwrap_or_default();
        if !matches!(
            subtype,
            "task_started" | "task_updated" | "task_notification"
        ) {
            return;
        }
        let Some(id) = value["task_id"]
            .as_str()
            .filter(|id| !id.is_empty() && id.len() <= 256 && !id.chars().any(char::is_control))
        else {
            self.unconfirmed = true;
            return;
        };
        let terminal = match subtype {
            "task_notification" => matches!(
                value["status"].as_str(),
                Some("completed" | "failed" | "stopped")
            ),
            "task_updated" => matches!(
                value["patch"]["status"].as_str(),
                Some("completed" | "failed" | "killed")
            ),
            _ => false,
        };
        if terminal {
            self.live.remove(id);
        } else if subtype == "task_started"
            && value["ambient"] != true
            && value["skip_transcript"] != true
        {
            if self.live.len() < 1024 || self.live.contains(id) {
                self.live.insert(id.to_owned());
            } else {
                self.unconfirmed = true;
            }
        }
    }

    fn pending(&self) -> bool {
        self.unconfirmed || !self.live.is_empty()
    }
}

struct ActiveTurn {
    message_id: String,
    completion: mpsc::Sender<ManagedTerminal>,
    response: String,
    /// Nombre d'événements journal `update` texte déjà écrits pour ce tour.
    text_updates: usize,
    /// Issue fournisseur (`end_turn`, …) — absente ⇒ le fil affiche « inconnu ».
    stop_reason: Option<String>,
    permission_denied: bool,
    latest_assistant_rate_limited: bool,
    latest_assistant_api_error: bool,
    structured_rate_limit_observed: bool,
    background_tasks: BackgroundTasks,
    /// Issue déjà signalée au fil de la mission : sa réponse lui appartient.
    /// Une fin de flux ou un arrêt qui suit ne doit plus reprendre ce tour.
    terminal_sent: bool,
}

struct QueueState {
    messages: VecDeque<BridgetMessage>,
    active: Option<ActiveTurn>,
    closed: bool,
    /// `request_id` du `control_request` d'interruption émis et non encore
    /// corrélé par son `control_response`. Porté par l'état de file parce que
    /// l'émetteur (`cancel_delivery`) et le lecteur le partagent déjà.
    pending_interrupt: Option<String>,
}

pub struct ClaudeStreamJsonTransport {
    permission_observation: Arc<Mutex<Option<(String, Value, u64, Option<String>)>>>,
    user_inputs: Arc<Mutex<Vec<(ChildProviderRequest, bool)>>>,
    connection_id: String,
    alive: Arc<AtomicBool>,
    shutdown_started: AtomicBool,
    busy: Arc<AtomicBool>,
    private_profile_instructions: Arc<Mutex<Option<String>>>,
    queue: Arc<(Mutex<QueueState>, Condvar)>,
    queue_capacity: usize,
    writer: Writer,
    child: Arc<Mutex<Child>>,
    provider_kind: String,
    provider_observation: Option<ProviderObservation>,
    events: Arc<Mutex<VecDeque<ManagedEvent>>>,
    journal: Journal,
    session_store: SessionStoreHandle,
    reader_handle: Mutex<Option<thread::JoinHandle<()>>>,
    worker_handle: Mutex<Option<thread::JoinHandle<()>>>,
}

impl ClaudeStreamJsonTransport {
    pub fn spawn(options: ClaudeStreamJsonOptions) -> Result<Self, TransportError> {
        Self::spawn_with_environment(options, &[], false)
    }

    pub fn spawn_inheriting_stderr_with_environment(
        options: ClaudeStreamJsonOptions,
        environment: &[(String, String)],
    ) -> Result<Self, TransportError> {
        Self::spawn_with_environment(options, environment, true)
    }

    pub fn spawn_with_environment(
        mut options: ClaudeStreamJsonOptions,
        environment: &[(String, String)],
        inherit_stderr: bool,
    ) -> Result<Self, TransportError> {
        if options.queue_capacity == 0 {
            return Err(TransportError::DeliveryFailed(
                "queue Claude de capacité nulle".to_string(),
            ));
        }
        let session_store = match (&options.session_store_root, &options.agent_name) {
            (Some(root), Some(agent)) if !agent.trim().is_empty() => {
                Some(ProviderSessionStore::new(root.clone(), agent.clone()))
            }
            _ => None,
        };
        let attempted_resume = prepare_launch_args(&mut options.args, session_store.as_ref());
        ensure_stream_arguments(&mut options.args)?;

        // Ne pas attendre la première trame ici : le daemon accorde 10 s avant
        // Connected et une reprise réelle peut dépasser ce plafond (~10,5 s).
        // La validation `--resume` vit dans le reader, après le spawn.
        let spawned = spawn_claude_child(&options, environment, inherit_stderr)?
            .ok_or_else(|| TransportError::Io("impossible de lancer Claude".to_string()))?;

        let pid = spawned.child.id();
        let writer = Arc::new(Mutex::new(Some(spawned.stdin)));
        let child = Arc::new(Mutex::new(spawned.child));
        let queue = Arc::new((
            Mutex::new(QueueState {
                messages: VecDeque::new(),
                active: None,
                closed: false,
                pending_interrupt: None,
            }),
            Condvar::new(),
        ));
        let alive = Arc::new(AtomicBool::new(true));
        let busy = Arc::new(AtomicBool::new(false));
        let private_profile_instructions = Arc::new(Mutex::new(None));
        let events = Arc::new(Mutex::new(VecDeque::new()));
        let journal = Arc::new(Mutex::new(None));
        let session_store_handle = Arc::new(Mutex::new(session_store));
        let provider_kind = options.provider_kind.clone();
        let pinned_model = pinned_model_from_args(&options.args);
        let resume_bootstrap = attempted_resume.map(|attempted_id| ResumeBootstrap {
            attempted_id,
            command: options.command.clone(),
            args: options.args.clone(),
            environment: environment.to_vec(),
            inherit_stderr,
            writer: writer.clone(),
            child: child.clone(),
        });
        let permission_observation = Arc::new(Mutex::new(None));
        let user_inputs = Arc::new(Mutex::new(Vec::new()));
        let reader_handle = spawn_reader(
            spawned.stdout,
            Vec::new(),
            queue.clone(),
            events.clone(),
            alive.clone(),
            journal.clone(),
            session_store_handle.clone(),
            provider_kind.clone(),
            pinned_model,
            resume_bootstrap,
            writer.clone(),
            permission_observation.clone(),
            options.args.clone(),
            user_inputs.clone(),
        );
        let worker_handle = spawn_worker(
            queue.clone(),
            writer.clone(),
            events.clone(),
            journal.clone(),
            alive.clone(),
            busy.clone(),
            private_profile_instructions.clone(),
            Duration::from_secs(options.notify_timeout_secs),
        );
        push_internal(
            &events,
            ManagedEventKind::ProviderContextObserved {
                identity: ManagedProviderIdentity {
                    provider_kind: provider_kind.clone(),
                    provider_session_id: None,
                    provider_thread_id: None,
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
            user_inputs,
            connection_id: format!("claude-stream-json-{pid}"),
            alive,
            shutdown_started: AtomicBool::new(false),
            busy,
            private_profile_instructions,
            queue,
            queue_capacity: options.queue_capacity,
            writer,
            provider_kind,
            provider_observation: options.provider_observation,
            child,
            events,
            journal,
            session_store: session_store_handle,
            reader_handle: Mutex::new(Some(reader_handle)),
            worker_handle: Mutex::new(Some(worker_handle)),
        })
    }

    fn push_internal(&self, kind: ManagedEventKind) {
        push_internal(&self.events, kind);
    }

    fn shutdown(&self) {
        if self.shutdown_started.swap(true, Ordering::SeqCst) {
            return;
        }
        self.alive.store(false, Ordering::SeqCst);
        let (lock, wake) = &*self.queue;
        let mut queue = lock.lock().recover_poison();
        queue.closed = true;
        while let Some(message) = queue.messages.pop_front() {
            self.push_internal(ManagedEventKind::DeliveryRejected {
                message_id: message.id,
                reason: "équipier Claude arrêté".to_string(),
            });
        }
        if queue
            .active
            .as_ref()
            .is_some_and(|active| !active.terminal_sent)
            && let Some(active) = queue.active.take()
        {
            let _ = active.completion.send(ManagedTerminal::Cancelled);
        }
        wake.notify_all();
        drop(queue);
        self.writer.lock().recover_poison().take();
        let mut child = self.child.lock().recover_poison();
        terminate_group(&mut child);
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

impl Drop for ClaudeStreamJsonTransport {
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl Transport for ClaudeStreamJsonTransport {
    fn deliver(&mut self, message: &BridgetMessage) -> Result<(), TransportError> {
        if !self.alive.load(Ordering::SeqCst) {
            return Err(TransportError::AgentDead);
        }
        let (lock, wake) = &*self.queue;
        let mut queue = lock.lock().recover_poison();
        let steer_or_queue = message.intent == Some(MessageIntent::SteerCurrent)
            && message
                .references
                .iter()
                .any(|reference| reference == "steer-or-queue");
        if message.intent == Some(MessageIntent::SteerCurrent) && !steer_or_queue {
            drop(queue);
            self.push_internal(ManagedEventKind::DeliveryRejected {
                message_id: message.id.clone(),
                reason: "pilotage Claude indisponible".to_string(),
            });
            return Ok(());
        }
        if queue.closed || queue.messages.len() >= self.queue_capacity {
            drop(queue);
            self.push_internal(ManagedEventKind::DeliveryRejected {
                message_id: message.id.clone(),
                reason: "file Claude pleine".to_string(),
            });
            return Ok(());
        }
        let mut ordinary = message.clone();
        if steer_or_queue {
            ordinary.intent = None;
        }
        queue.messages.push_back(ordinary);
        // Le relais UI marque l'origine humaine sans choisir le protocole.
        // Claude conserve la FIFO et ne cible que le tour actif capturé.
        let active_message_id = (!steer_or_queue
            && (message.intent == Some(MessageIntent::InterruptAndStart)
                || message.origin == Some(MessageOrigin::Human))
            && queue.pending_interrupt.is_none())
        .then(|| {
            queue
                .active
                .as_ref()
                .map(|active| active.message_id.clone())
        })
        .flatten();
        if let Some(active_message_id) = active_message_id {
            drop(queue);
            wake.notify_one();
            let _ = self.cancel_delivery(&active_message_id, "interruption explicite");
        } else {
            wake.notify_one();
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

impl ManagedSession for ClaudeStreamJsonTransport {
    fn answer_provider_request(
        &mut self,
        answer: &ChildRequestAnswer,
    ) -> Result<(), TransportError> {
        if !self.alive.load(Ordering::SeqCst) {
            return Err(TransportError::AgentDead);
        }
        let mut pending = self.user_inputs.lock().recover_poison();
        let index = pending
            .iter()
            .position(|(r, _)| {
                r.provider_session == answer.provider_session
                    && r.turn_id == answer.turn_id
                    && r.provider_request_id == answer.provider_request_id
            })
            .ok_or_else(|| TransportError::DeliveryFailed("request_unknown".into()))?;
        let (request, uncertain) = &pending[index];
        if *uncertain {
            return Err(TransportError::Io(
                "child_request_write_uncertain: attente déjà incertaine".into(),
            ));
        }
        if self
            .queue
            .0
            .lock()
            .recover_poison()
            .active
            .as_ref()
            .is_none_or(|a| a.message_id != request.turn_id)
        {
            return Err(TransportError::DeliveryFailed("request_unlinked".into()));
        }
        let frame = claude_user_input_response(request, answer)?;
        pending[index].1 = true;
        let result = (|| -> std::io::Result<()> {
            let mut writer = self
                .writer
                .lock()
                .map_err(|_| std::io::Error::other("writer poisoned"))?;
            let stdin = writer
                .as_mut()
                .ok_or_else(|| std::io::Error::other("writer closed"))?;
            writeln!(stdin, "{frame}")?;
            stdin.flush()
        })();
        result.map_err(|e| TransportError::Io(format!("child_request_write_uncertain: {e}")))?;
        pending.remove(index);
        Ok(())
    }
    fn provider_permissions(&self) -> Option<(String, Value, u64, Option<String>)> {
        self.permission_observation.lock().recover_poison().clone()
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
        let session_id = self
            .session_store
            .lock()
            .recover_poison()
            .as_ref()
            .and_then(ProviderSessionStore::load);
        Some(ManagedProviderIdentity {
            provider_kind: self.provider_kind.clone(),
            provider_session_id: session_id,
            provider_thread_id: None,
            active_turn_id: None,
            provider_item_id: None,
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
        let events = self.events.clone();
        let failure: JournalFailureSink = Arc::new(move |detail| {
            push_internal(&events, ManagedEventKind::JournalFailed { detail });
        });
        *self.journal.lock().recover_poison() =
            Some(JournalWriter::start_with_live_feed_and_failure(
                root,
                agent,
                self.connection_id(),
                failure,
                live_feed,
            )?);
        // Même racine que le journal : le session_id fournisseur survit ainsi
        // à la mort du wrapper et au redémarrage du daemon.
        *self.session_store.lock().recover_poison() = Some(ProviderSessionStore::new(root, agent));
        Ok(())
    }

    fn journal_writer(&self) -> Option<(JournalWriter, String)> {
        self.journal
            .lock()
            .ok()
            .and_then(|journal| journal.clone())
            .map(|writer| (writer, self.connection_id.clone()))
    }

    fn drain_events(&self) -> Vec<ManagedEvent> {
        self.events.lock().recover_poison().drain(..).collect()
    }

    fn cancel_delivery(&self, message_id: &str, reason: &str) -> bool {
        let (lock, wake) = &*self.queue;
        let mut queue = lock.lock().recover_poison();
        if queue
            .active
            .as_ref()
            .is_some_and(|active| active.message_id == message_id)
        {
            // Le contrôle d'interruption Claude est désormais attesté (mesures
            // du 28/08) : on coupe le tour au lieu de tuer l'agent, qui reste
            // vivant et enchaîne. Le terminal `aborted_*` clôt la livraison ;
            // l'échéance du tour (`recv_timeout`) reste le repli si le
            // fournisseur ne rend ni accusé ni terminal.
            let request_id = format!("bridget-interrupt-{message_id}");
            if write_control_interrupt(&self.writer, &request_id).is_ok() {
                queue.pending_interrupt = Some(request_id.clone());
                drop(queue);
                wake.notify_one();
                record_or_terminal(
                    &self.journal,
                    &self.events,
                    "interrupt_requested",
                    Some(message_id),
                    json!({ "request_id": request_id, "reason": reason }),
                );
                return true;
            }
            // Repli historique : sans canal d'écriture, la seule fin honnête
            // reste de terminer la livraison et d'arrêter le groupe.
            let active = queue.active.take().expect("tour Claude actif");
            let _ = active.completion.send(ManagedTerminal::Cancelled);
            wake.notify_one();
            self.alive.store(false, Ordering::SeqCst);
            self.writer.lock().recover_poison().take();
            let mut child = self.child.lock().recover_poison();
            terminate_group(&mut child);
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
            .expect("message Claude présent");
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
    }
}

fn ensure_stream_arguments(args: &mut Vec<String>) -> Result<(), TransportError> {
    if args.iter().any(|argument| {
        matches!(argument.as_str(), "--input-format" | "--output-format")
            || argument.starts_with("--input-format=")
            || argument.starts_with("--output-format=")
    }) {
        return Err(TransportError::DeliveryFailed(
            "le pilote Claude impose ses formats stream-json".to_string(),
        ));
    }
    args.extend(
        [
            "-p",
            "--input-format",
            "stream-json",
            "--output-format",
            "stream-json",
            "--include-partial-messages",
            "--verbose",
        ]
        .into_iter()
        .map(str::to_string),
    );
    Ok(())
}

/// Exécute exactement une décision168 dans un contexte enfant jetable. Ce
/// chemin ne construit ni transport géré, ni journal, ni Task :
/// le processus est uniquement un adaptateur de sélection structuré.
/// O(b) pour b octets d'entrée/sortie, bornés chacun à 64 Kio.
pub fn run_isolated_claude_decision(
    options: &ClaudeIsolatedDecisionOptions,
) -> Result<ClaudeIsolatedDecisionResult, ClaudeIsolatedDecisionError> {
    if options.command.trim().is_empty()
        || options.model.is_empty()
        || options.request_json.is_empty()
        || options.request_json.len() > 64 * 1024
    {
        return Err(ClaudeIsolatedDecisionError::InvalidInput);
    }
    let deadline = options.deadline;
    if Instant::now() >= deadline {
        return Err(ClaudeIsolatedDecisionError::Timeout);
    }
    let schema = r#"{"type":"object","additionalProperties":false,"required":["candidate_id"],"properties":{"candidate_id":{"type":"string"},"effort":{"type":"string"},"reason":{"type":"string","maxLength":256}}}"#;
    let nonce = format!(
        "bridget-routing-decision-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    );
    let cwd = std::env::temp_dir().join(nonce);
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&cwd)
        .map_err(|_| ClaudeIsolatedDecisionError::Unavailable)?;
    let result = (|| {
        let mut command = Command::new(&options.command);
        command
            .args(&options.command_args)
            .arg("--print")
            .args(["--model", &options.model])
            .args([
                "--safe-mode",
                "--restricted",
                "--tools",
                "",
                "--strict-mcp-config",
                "--mcp-config",
                r#"{"mcpServers":{}}"#,
                "--setting-sources",
                "",
                "--disable-slash-commands",
                "--no-session-persistence",
            ])
            .args([
                "--input-format",
                "stream-json",
                "--output-format",
                "stream-json",
                "--verbose",
                "--json-schema",
                schema,
            ])
            .env_clear()
            .envs(options.environment.iter().cloned())
            .current_dir(&cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .process_group(0);
        if Instant::now() >= deadline {
            return Err(ClaudeIsolatedDecisionError::Timeout);
        }
        let mut child = command
            .spawn()
            .map_err(|_| ClaudeIsolatedDecisionError::Unavailable)?;
        // Tout retour après spawn passe par l'arrêt confirmé ci-dessous. Le
        // thread appelant conserve donc aussi son permis pendant le drain,
        // même si son demandeur cesse déjà d'attendre sa réponse.
        let decision = (|| {
            let mut stdin = Some(
                child
                    .stdin
                    .take()
                    .ok_or(ClaudeIsolatedDecisionError::Unavailable)?,
            );
            let mut stdout = child
                .stdout
                .take()
                .ok_or(ClaudeIsolatedDecisionError::Unavailable)?;
            for fd in [stdin.as_ref().unwrap().as_raw_fd(), stdout.as_raw_fd()] {
                // SAFETY: fcntl avec une commande à argument entier ne déréférence aucun pointeur.
                let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
                if flags < 0
                    // SAFETY: fcntl avec une commande à argument entier ne déréférence aucun pointeur.
                    || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0
                {
                    return Err(ClaudeIsolatedDecisionError::Unavailable);
                }
            }
            let mut input = options.request_json.as_bytes().to_vec();
            input.push(b'\n');
            let mut written = 0;
            let mut bytes = Vec::with_capacity(MAX_ISOLATED_DECISION_STDOUT_BYTES + 1);
            let mut chunk = [0_u8; 4096];
            let mut eof = false;
            let mut status = None;
            loop {
                if Instant::now() >= deadline {
                    return Err(ClaudeIsolatedDecisionError::Timeout);
                }
                // Lire en même temps qu'on écrit évite qu'un fournisseur qui
                // remplit stdout avant de lire stdin bloque les deux tubes.
                if !eof {
                    let remaining =
                        (MAX_ISOLATED_DECISION_STDOUT_BYTES + 1 - bytes.len()).min(chunk.len());
                    match stdout.read(&mut chunk[..remaining]) {
                        Ok(0) => eof = true,
                        Ok(count) => {
                            bytes.extend_from_slice(&chunk[..count]);
                            if bytes.len() > MAX_ISOLATED_DECISION_STDOUT_BYTES {
                                return Err(ClaudeIsolatedDecisionError::OutputTooLarge);
                            }
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                        Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(_) => return Err(ClaudeIsolatedDecisionError::Unavailable),
                    }
                }
                if let Some(writer) = stdin.as_mut() {
                    match writer.write(&input[written..]) {
                        Ok(0) => return Err(ClaudeIsolatedDecisionError::Unavailable),
                        Ok(count) => {
                            written += count;
                            if written == input.len() {
                                stdin.take();
                            }
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                        Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(_) => return Err(ClaudeIsolatedDecisionError::Unavailable),
                    }
                }
                if status.is_none() {
                    status = child
                        .try_wait()
                        .map_err(|_| ClaudeIsolatedDecisionError::Unavailable)?;
                }
                if let Some(status) = status.filter(|_| eof) {
                    if !status.success() {
                        return Err(ClaudeIsolatedDecisionError::InvalidOutput);
                    }
                    return parse_isolated_decision_output(&bytes);
                }
                thread::sleep(Duration::from_millis(5));
            }
        })();
        stop_isolated_group_until_confirmed(&mut child);
        decision
    })();
    let _ = std::fs::remove_dir(&cwd);
    result
}

fn stop_isolated_group_until_confirmed(child: &mut Child) {
    while stop_owned_child(child, true, Duration::from_secs(1)).is_err() {
        thread::sleep(Duration::from_millis(20));
    }
}

// RawValue emprunte les objets de décision sans les parser ni les copier.
// Leurs bornes de 4 Kio sont vérifiées avant le décodage de leurs champs.
#[derive(Deserialize)]
struct IsolatedDecisionFrame<'a> {
    #[serde(rename = "type")]
    kind: &'a str,
    #[serde(default)]
    subtype: Option<&'a str>,
    #[serde(default)]
    tools: Option<Vec<String>>,
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    mcp_servers: Vec<Value>,
    #[serde(default)]
    skills: Vec<Value>,
    #[serde(default)]
    slash_commands: Vec<Value>,
    #[serde(default)]
    plugins: Vec<Value>,
    #[serde(default, borrow)]
    message: Option<IsolatedDecisionMessage<'a>>,
    #[serde(default)]
    is_error: Option<bool>,
    #[serde(default, borrow)]
    structured_output: Option<&'a serde_json::value::RawValue>,
    #[serde(default)]
    result: Option<String>,
}

#[derive(Deserialize)]
struct IsolatedDecisionMessage<'a> {
    #[serde(borrow)]
    content: Vec<IsolatedDecisionContent<'a>>,
}

#[derive(Deserialize)]
struct IsolatedDecisionContent<'a> {
    #[serde(rename = "type")]
    kind: &'a str,
    #[serde(default)]
    name: Option<&'a str>,
    #[serde(default, borrow)]
    input: Option<&'a serde_json::value::RawValue>,
}

fn parse_isolated_decision_output(
    bytes: &[u8],
) -> Result<ClaudeIsolatedDecisionResult, ClaudeIsolatedDecisionError> {
    if bytes.len() > MAX_ISOLATED_DECISION_STDOUT_BYTES {
        return Err(ClaudeIsolatedDecisionError::OutputTooLarge);
    }
    let text =
        std::str::from_utf8(bytes).map_err(|_| ClaudeIsolatedDecisionError::InvalidOutput)?;
    let mut initialized = false;
    let mut structured_input = None;
    let mut final_output = None;
    // O(b), flux entier <=64 Kio ; chaque objet détaillé <=4 Kio.
    for line in text.lines() {
        let frame: IsolatedDecisionFrame<'_> =
            serde_json::from_str(line).map_err(|_| ClaudeIsolatedDecisionError::InvalidOutput)?;
        if final_output.is_some()
            || !matches!(
                frame.kind,
                "system" | "rate_limit_event" | "assistant" | "user" | "result"
            )
        {
            return Err(ClaudeIsolatedDecisionError::InvalidOutput);
        }
        if frame.kind == "system" && frame.subtype == Some("init") {
            if initialized
                || !frame
                    .tools
                    .as_ref()
                    .is_some_and(|tools| tools.len() == 1 && tools[0] == "StructuredOutput")
                || !frame.mcp_servers.is_empty()
                || !frame.skills.is_empty()
                || !frame.slash_commands.is_empty()
                || !frame.plugins.is_empty()
                || frame.capabilities.iter().any(|capability| {
                    !matches!(
                        capability.as_str(),
                        "interrupt_receipt_v1" | "interrupt_cancel_queued_v1" | "msg_lifecycle_v1"
                    )
                })
            {
                return Err(ClaudeIsolatedDecisionError::InvalidOutput);
            }
            initialized = true;
        }
        if let Some(message) = frame.message {
            for part in message.content {
                if part.kind == "tool_use" {
                    if !initialized
                        || frame.kind != "assistant"
                        || part.name != Some("StructuredOutput")
                        || structured_input.is_some()
                    {
                        return Err(ClaudeIsolatedDecisionError::InvalidOutput);
                    }
                    let raw = part
                        .input
                        .ok_or(ClaudeIsolatedDecisionError::InvalidOutput)?;
                    if raw.get().len() > MAX_ISOLATED_DECISION_JSON_BYTES {
                        return Err(ClaudeIsolatedDecisionError::OutputTooLarge);
                    }
                    structured_input = Some(
                        serde_json::from_str::<Value>(raw.get())
                            .map_err(|_| ClaudeIsolatedDecisionError::InvalidOutput)?,
                    );
                }
            }
        }
        if frame.kind == "result" {
            if !initialized || frame.is_error != Some(false) || frame.subtype != Some("success") {
                return Err(ClaudeIsolatedDecisionError::InvalidOutput);
            }
            let raw = frame
                .structured_output
                .ok_or(ClaudeIsolatedDecisionError::InvalidOutput)?;
            let echoed = frame
                .result
                .ok_or(ClaudeIsolatedDecisionError::InvalidOutput)?;
            if raw.get().len() > MAX_ISOLATED_DECISION_JSON_BYTES
                || echoed.len() > MAX_ISOLATED_DECISION_JSON_BYTES
            {
                return Err(ClaudeIsolatedDecisionError::OutputTooLarge);
            }
            let output: Value = serde_json::from_str(raw.get())
                .map_err(|_| ClaudeIsolatedDecisionError::InvalidOutput)?;
            let echoed: Value = serde_json::from_str(&echoed)
                .map_err(|_| ClaudeIsolatedDecisionError::InvalidOutput)?;
            if echoed != output || structured_input.as_ref() != Some(&output) {
                return Err(ClaudeIsolatedDecisionError::InvalidOutput);
            }
            final_output = Some(raw);
        }
    }
    // Décoder directement le JSON fermé conserve le refus serde des champs
    // dupliqués, qu'une conversion préalable en Value aurait effacés.
    let result: ClaudeIsolatedDecisionResult = serde_json::from_str(
        final_output
            .ok_or(ClaudeIsolatedDecisionError::InvalidOutput)?
            .get(),
    )
    .map_err(|_| ClaudeIsolatedDecisionError::InvalidOutput)?;
    if result.candidate_id.is_empty()
        || result.candidate_id.len() > 16
        || result
            .effort
            .as_deref()
            .is_some_and(|value| value.is_empty() || value.len() > 64)
        || result
            .reason
            .as_deref()
            .is_some_and(|value| value.is_empty() || value.len() > 256)
    {
        return Err(ClaudeIsolatedDecisionError::InvalidOutput);
    }
    Ok(result)
}

struct SpawnedClaude {
    child: Child,
    stdin: ChildStdin,
    stdout: ChildStdout,
}

/// Lance le binaire Claude. La validation `--resume` n'est pas faite ici :
/// voir `bootstrap_resume_in_reader`.
fn spawn_claude_child(
    options: &ClaudeStreamJsonOptions,
    environment: &[(String, String)],
    inherit_stderr: bool,
) -> Result<Option<SpawnedClaude>, TransportError> {
    crate::protocol::recheck_frozen_inputs(&options.command, environment)
        .map_err(TransportError::DeliveryFailed)?;
    let mut command = Command::new(&options.command);
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
        .process_group(0);
    let mut child = command
        .spawn()
        .map_err(|error| TransportError::Io(format!("impossible de lancer Claude: {error}")))?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| TransportError::Io("stdin Claude absent".to_string()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| TransportError::Io("stdout Claude absent".to_string()))?;
    Ok(Some(SpawnedClaude {
        child,
        stdin,
        stdout,
    }))
}

/// Repli sans `--resume` après échec nommé.
fn relaunch_after_resume_failure(
    mut bootstrap: ResumeBootstrap,
    events: &Arc<Mutex<VecDeque<ManagedEvent>>>,
    session_store: &SessionStoreHandle,
    notice: String,
) -> (ChildStdout, Vec<String>) {
    if let Some(store) = session_store.lock().recover_poison().as_ref() {
        let _ = store.clear();
    }
    push_internal(events, ManagedEventKind::Error { detail: notice });
    {
        let mut child = bootstrap.child.lock().recover_poison();
        terminate_group(&mut child);
    }
    strip_resume_arg(&mut bootstrap.args);
    let Ok(Some(spawned)) = spawn_claude_command(
        &bootstrap.command,
        &bootstrap.args,
        &bootstrap.environment,
        bootstrap.inherit_stderr,
    ) else {
        let fallback = Command::new("/bin/true")
            .stdout(Stdio::piped())
            .spawn()
            .ok()
            .and_then(|mut child| child.stdout.take());
        return (fallback.expect("repli Claude indisponible"), Vec::new());
    };
    {
        let mut writer = bootstrap.writer.lock().recover_poison();
        *writer = Some(spawned.stdin);
    }
    {
        let mut child = bootstrap.child.lock().recover_poison();
        *child = spawned.child;
    }
    (spawned.stdout, Vec::new())
}

/// Lit la première trame `--resume` dans le reader (hors chemin spawn). En
/// cas d'échec : message nommé, effacement de l'id, repli sans `--resume`.
fn bootstrap_resume_in_reader(
    stdout: ChildStdout,
    bootstrap: ResumeBootstrap,
    events: &Arc<Mutex<VecDeque<ManagedEvent>>>,
    session_store: &SessionStoreHandle,
) -> (ChildStdout, Vec<String>) {
    let mut reader = BufReader::new(stdout);
    let mut first_line = String::new();
    let read = reader.read_line(&mut first_line);
    let attempted_id = bootstrap.attempted_id.clone();

    match read {
        Ok(0) => relaunch_after_resume_failure(
            bootstrap,
            events,
            session_store,
            format!(
                "reprise Claude impossible: conversation introuvable pour l'identifiant {attempted_id} — démarrage d'une session neuve (stdout vide après --resume)"
            ),
        ),
        Ok(_) => {
            let trimmed = first_line.trim_end_matches(['\r', '\n']).to_string();
            if let Ok(value) = serde_json::from_str::<Value>(&trimmed) {
                if let Some(failure) = classify_resume_failure(&value, &attempted_id) {
                    return relaunch_after_resume_failure(
                        bootstrap,
                        events,
                        session_store,
                        failure.named_message(),
                    );
                }
                if let Some(session_id) = session_id_from_system_init(&value)
                    && let Some(store) = session_store.lock().recover_poison().as_ref()
                {
                    let _ = store.store(&session_id);
                }
            }
            (reader.into_inner(), vec![trimmed])
        }
        Err(error) => relaunch_after_resume_failure(
            bootstrap,
            events,
            session_store,
            format!(
                "reprise Claude impossible: lecture initiale impossible pour l'identifiant {attempted_id} — démarrage d'une session neuve ({error})"
            ),
        ),
    }
}

fn spawn_claude_command(
    command_path: &str,
    args: &[String],
    environment: &[(String, String)],
    inherit_stderr: bool,
) -> Result<Option<SpawnedClaude>, TransportError> {
    spawn_claude_child(
        &ClaudeStreamJsonOptions {
            command: command_path.to_string(),
            args: args.to_vec(),
            provider_kind: "claude".to_string(),
            queue_capacity: 1,
            notify_timeout_secs: 1,
            provider_observation: None,
            session_store_root: None,
            agent_name: None,
        },
        environment,
        inherit_stderr,
    )
}

fn terminate_group(child: &mut Child) {
    // `spawn_with_environment` crée ce groupe avec l'identifiant du fils :
    // le signal ne peut donc toucher ni le wrapper ni un autre équipier.
    // SAFETY: kill ne déréférence aucun pointeur ; ses arguments sont des entiers.
    let _ = unsafe { libc::kill(-(child.id() as i32), libc::SIGKILL) };
    let _ = child.wait();
}

#[allow(clippy::too_many_arguments)]
fn spawn_worker(
    queue: Arc<(Mutex<QueueState>, Condvar)>,
    writer: Writer,
    events: Arc<Mutex<VecDeque<ManagedEvent>>>,
    journal: Journal,
    alive: Arc<AtomicBool>,
    busy: Arc<AtomicBool>,
    private_profile_instructions: Arc<Mutex<Option<String>>>,
    notify_timeout: Duration,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let message = {
                let (lock, wake) = &*queue;
                let mut state = lock.lock().recover_poison();
                while state.messages.is_empty() && !state.closed {
                    state = wake.wait(state).recover_poison();
                }
                if state.closed {
                    return;
                }
                state.messages.pop_front().expect("file Claude non vide")
            };
            if !alive.load(Ordering::SeqCst) {
                push_internal(
                    &events,
                    ManagedEventKind::DeliveryRejected {
                        message_id: message.id,
                        reason: "équipier Claude arrêté".to_string(),
                    },
                );
                continue;
            }
            let (completion, receiver) = mpsc::channel();
            queue.0.lock().recover_poison().active = Some(ActiveTurn {
                message_id: message.id.clone(),
                completion,
                response: String::new(),
                text_updates: 0,
                stop_reason: None,
                permission_denied: false,
                latest_assistant_rate_limited: false,
                latest_assistant_api_error: false,
                structured_rate_limit_observed: false,
                background_tasks: BackgroundTasks::default(),
                terminal_sent: false,
            });
            busy.store(true, Ordering::SeqCst);
            push_internal(
                &events,
                ManagedEventKind::TurnStarted {
                    message_id: message.id.clone(),
                },
            );
            record_or_terminal(
                &journal,
                &events,
                "turn_start",
                Some(&message.id),
                json!({
                    "from": &message.from,
                    "body": message.body,
                }),
            );
            let instructions = private_profile_instructions.lock().recover_poison().clone();
            let terminal = match write_input(&writer, &message, instructions.as_deref()) {
                Ok(()) => {
                    push_internal(
                        &events,
                        ManagedEventKind::PromptDispatched {
                            message_id: message.id.clone(),
                        },
                    );
                    record_or_terminal(
                        &journal,
                        &events,
                        "prompt_dispatched",
                        Some(&message.id),
                        json!({
                            "from": &message.from,
                            "body": &message.body,
                        }),
                    );
                    let deadline = message
                        .deadline_at
                        .map(|seconds| UNIX_EPOCH + Duration::from_secs(seconds))
                        .unwrap_or_else(|| SystemTime::now() + notify_timeout);
                    let remaining = deadline
                        .duration_since(SystemTime::now())
                        .unwrap_or(Duration::ZERO);
                    receiver
                        .recv_timeout(remaining)
                        .unwrap_or(ManagedTerminal::Failed {
                            detail: "échéance Claude dépassée".to_string(),
                        })
                }
                Err(error) => ManagedTerminal::Failed {
                    detail: error.to_string(),
                },
            };
            let (response, stop_reason) = {
                let mut state = queue.0.lock().recover_poison();
                match state.active.take() {
                    Some(active) => (active.response, active.stop_reason),
                    None => (String::new(), None),
                }
            };
            let event = match &terminal {
                ManagedTerminal::Completed | ManagedTerminal::Cancelled => {
                    ManagedEventKind::TurnFinished {
                        message: message.clone(),
                        response,
                        terminal: terminal.clone(),
                    }
                }
                ManagedTerminal::Failed { detail } => ManagedEventKind::DeliveryRejected {
                    message_id: message.id.clone(),
                    reason: detail.clone(),
                },
            };
            if matches!(event, ManagedEventKind::TurnFinished { .. }) {
                let stop_reason = stop_reason.unwrap_or_else(|| match &terminal {
                    ManagedTerminal::Cancelled => "cancelled".to_string(),
                    ManagedTerminal::Completed => "completed".to_string(),
                    ManagedTerminal::Failed { detail } => detail.clone(),
                });
                let mut payload = json!({ "stop_reason": stop_reason });
                if message.reply {
                    payload["routed_to"] = json!(&message.from);
                }
                record_or_terminal(&journal, &events, "turn_end", Some(&message.id), payload);
            } else if let ManagedEventKind::DeliveryRejected { reason, .. } = &event {
                // Même borne que ACP/Codex : une échéance DOIT écrire error
                // avec terminal_kind=turn_failed, sinon bridget-idle classe
                // encore OCCUPE un mort des mains.
                record_or_terminal(
                    &journal,
                    &events,
                    "error",
                    Some(&message.id),
                    with_turn_failed_kind(json!({ "reason": reason })),
                );
            }
            push_internal(&events, event);
            busy.store(false, Ordering::SeqCst);
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

fn write_input(
    writer: &Writer,
    message: &BridgetMessage,
    instructions: Option<&str>,
) -> Result<(), TransportError> {
    // Même contrat géré qu'ACP : une réponse finale, uniquement si demandée.
    let prompt = private_prompt(instructions, &crate::acp::prompt_for(message));
    let frame = json!({
        "type": "user",
        "message": {
            "role": "user",
            "content": [{ "type": "text", "text": prompt }],
        },
    });
    let mut writer = writer
        .lock()
        .map_err(|error| TransportError::Io(error.to_string()))?;
    let writer = writer.as_mut().ok_or(TransportError::AgentDead)?;
    writeln!(writer, "{frame}").map_err(|error| TransportError::Io(error.to_string()))?;
    writer
        .flush()
        .map_err(|error| TransportError::Io(error.to_string()))
}

/// Trame de contrôle mesurée le 28/08 : `control_request` / `interrupt` sur
/// l'entrée standard. Claude accuse par `control_response` en 3 ms puis rend un
/// terminal `aborted_tools` ou `aborted_streaming` en 25 ms, sous-processus
/// réellement arrêté, sans aucun signal POSIX. Le tour suivant repart.
fn write_control_interrupt(writer: &Writer, request_id: &str) -> Result<(), TransportError> {
    let frame = json!({
        "type": "control_request",
        "request_id": request_id,
        "request": { "subtype": "interrupt" },
    });
    let mut writer = writer
        .lock()
        .map_err(|error| TransportError::Io(error.to_string()))?;
    let writer = writer.as_mut().ok_or(TransportError::AgentDead)?;
    writeln!(writer, "{frame}").map_err(|error| TransportError::Io(error.to_string()))?;
    writer
        .flush()
        .map_err(|error| TransportError::Io(error.to_string()))
}

/// Terminaux d'interruption Claude. Distincts d'un échec : le tour est coupé à
/// la demande, l'agent reste vivant et enchaîne.
fn interrupt_terminal_reason(reason: &str) -> bool {
    matches!(reason, "aborted_tools" | "aborted_streaming")
}

// terminal_reason est facultatif dans SDKResultMessage. Le sous-type
// success est le repli documenté, jamais le texte d'un assistant ou d'un commit.
// Une raison présente mais inconnue ou mal typée ne bénéficie pas de ce repli.
fn known_claude_terminal_reason(reason: &str) -> Option<&str> {
    matches!(
        reason,
        "completed"
            | "max_turns"
            | "tool_deferred"
            | "aborted_streaming"
            | "aborted_tools"
            | "hook_stopped"
            | "stop_hook_prevented"
            | "background_requested"
            | "blocking_limit"
            | "rapid_refill_breaker"
            | "prompt_too_long"
            | "image_error"
            | "model_error"
            | "api_error"
            | "malformed_tool_use_exhausted"
            | "budget_exhausted"
            | "structured_output_retry_exhausted"
            | "tool_deferred_unavailable"
            | "turn_setup_failed"
    )
    .then_some(reason)
}

fn classify_claude_result(
    value: &Value,
    interrupted: bool,
    rate_limited: bool,
    diagnostic: Option<&str>,
) -> ManagedTerminal {
    let reason = value.get("terminal_reason").and_then(Value::as_str);
    let reason_absent = value.get("terminal_reason").is_none_or(Value::is_null);
    let subtype = value.get("subtype").and_then(Value::as_str);
    let success_type =
        value.get("subtype").is_none_or(Value::is_null) || subtype == Some("success");
    if interrupted {
        return ManagedTerminal::Failed {
            detail: "provider_permission_denied".into(),
        };
    }
    if value["is_error"].as_bool() == Some(false)
        && ((reason == Some("completed") && success_type)
            || (reason_absent && subtype == Some("success")))
    {
        return ManagedTerminal::Completed;
    }
    if reason.is_some_and(interrupt_terminal_reason) {
        return ManagedTerminal::Cancelled;
    }
    // A successful assistant quotation is never an authentication fact. Only
    // this exact provider diagnostic, correlated to an api_error terminal in
    // the same turn, establishes an expired session. Keep the raw api_error.
    const EXPIRED: &str =
        "Failed to authenticate: OAuth session expired and could not be refreshed";
    if reason == Some("api_error")
        && value["is_error"].as_bool() == Some(true)
        && (value
            .get("result")
            .and_then(Value::as_str)
            .is_some_and(|s| s.trim() == EXPIRED)
            || diagnostic.is_some_and(|s| s.trim() == EXPIRED))
    {
        return ManagedTerminal::Failed {
            detail: "api_error:oauth_session_expired".into(),
        };
    }
    if rate_limited {
        return ManagedTerminal::Failed {
            detail: "usage_limit".into(),
        };
    }
    let fallback = if reason_absent && value["is_error"].as_bool() == Some(true) {
        match subtype {
            Some("error_max_turns") => Some("max_turns"),
            Some("error_max_budget_usd") => Some("budget_exhausted"),
            Some("error_max_structured_output_retries") => {
                Some("structured_output_retry_exhausted")
            }
            Some("error_during_execution") => Some("error_during_execution"),
            _ => None,
        }
    } else {
        None
    };
    ManagedTerminal::Failed {
        detail: reason
            .filter(|reason| *reason != "completed")
            .and_then(known_claude_terminal_reason)
            .or(fallback)
            .unwrap_or("terminal Claude inconnu")
            .to_owned(),
    }
}

// Vocabulaire fermé : aucun result, errors, chemin ou identifiant de compte.
// L'identifiant du message actif porte la corrélation dans le journal existant.
fn claude_result_metadata(value: &Value, terminal: &ManagedTerminal, interrupted: bool) -> Value {
    let subtype = match value.get("subtype").and_then(Value::as_str) {
        Some(
            s @ ("success"
            | "error_max_turns"
            | "error_during_execution"
            | "error_max_budget_usd"
            | "error_max_structured_output_retries"),
        ) => s,
        None if value.get("subtype").is_none_or(Value::is_null) => "missing",
        _ => "unrecognized",
    };
    let reason = match value.get("terminal_reason") {
        None | Some(Value::Null) => "missing",
        Some(Value::String(reason)) => {
            known_claude_terminal_reason(reason).unwrap_or("unrecognized")
        }
        _ => "unrecognized",
    };
    json!({
        "subtype": subtype,
        "terminal_reason": reason,
        "is_error": value.get("is_error").and_then(Value::as_bool),
        "num_turns": value.get("num_turns").and_then(Value::as_u64).map(|n| n.min(1_000_000)),
        "permission_denial_count": value.get("permission_denials").and_then(Value::as_array)
            .map_or(0, |v| v.len().min(1_000_000)),
        "permission_interrupted": interrupted,
        "diagnostic_code": if matches!(terminal,ManagedTerminal::Failed{detail} if detail=="api_error:oauth_session_expired") {Some("oauth_session_expired")} else {None},
        "outcome": match terminal { ManagedTerminal::Completed => "completed",
            ManagedTerminal::Cancelled => "cancelled", ManagedTerminal::Failed { .. } => "failed" },
    })
}

// Même mécanisme que canUseTool de T3 ; seule AskUserQuestion sort du refus des permissions.
fn claude_user_input(value: &Value, session: &str, turn: &str) -> Option<ChildProviderRequest> {
    if value.pointer("/request/tool_name")?.as_str()? != "AskUserQuestion" {
        return None;
    }
    let id = value["request_id"].as_str()?;
    let input = value.pointer("/request/input")?;
    if serde_json::to_vec(input).ok()?.len() > 16 * 1024 {
        return None;
    }
    let source = input["questions"].as_array()?;
    if source.is_empty() || source.len() > 16 {
        return None;
    }
    let mut ids = Vec::new();
    let mut questions = Vec::new();
    for q in source {
        let id = q["question"].as_str()?;
        if id.is_empty() || ids.iter().any(|v| v == id) {
            return None;
        }
        ids.push(id.to_string());
        let mut question = q.clone();
        question["id"] = json!(id);
        questions.push(question);
    }
    Some(ChildProviderRequest {
        version: 1,
        provider_session: session.into(),
        turn_id: turn.into(),
        provider_request_id: ChildProviderRequestId::String(id.into()),
        kind: ChildRequestKind::UserInput,
        details: json!({"method":"AskUserQuestion","input":input,"questions":questions,"secret":false}),
        offered: vec![
            ChildRequestDecision::Accept,
            ChildRequestDecision::Decline,
            ChildRequestDecision::Cancel,
        ],
        question_ids: ids,
        truncated: false,
    })
}
fn claude_user_input_response(
    request: &ChildProviderRequest,
    answer: &ChildRequestAnswer,
) -> Result<Value, TransportError> {
    let response = if answer.decision == ChildRequestDecision::Accept {
        let raw = answer
            .answer
            .as_ref()
            .and_then(Value::as_object)
            .ok_or_else(|| TransportError::DeliveryFailed("answer_invalid".into()))?;
        if raw.len() != request.question_ids.len()
            || serde_json::to_vec(raw).map_or(true, |v| v.len() > 32 * 1024)
        {
            return Err(TransportError::DeliveryFailed("answer_invalid".into()));
        }
        let mut mapped = serde_json::Map::new();
        for id in &request.question_ids {
            let a = raw
                .get(id)
                .ok_or_else(|| TransportError::DeliveryFailed("answer_invalid".into()))?;
            let values = a["selected"]
                .as_array()
                .ok_or_else(|| TransportError::DeliveryFailed("answer_invalid".into()))?;
            let mut selected = values
                .iter()
                .map(|v| v.as_str().map(str::to_owned))
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| TransportError::DeliveryFailed("answer_invalid".into()))?;
            let text = a["text"]
                .as_str()
                .ok_or_else(|| TransportError::DeliveryFailed("answer_invalid".into()))?;
            if !text.is_empty() {
                selected.push(text.into());
            }
            if selected.is_empty() {
                return Err(TransportError::DeliveryFailed("answer_invalid".into()));
            }
            mapped.insert(id.clone(), json!(selected.join(", ")));
        }
        let mut input = request.details["input"].clone();
        input["answers"] = Value::Object(mapped);
        json!({"behavior":"allow","updatedInput":input})
    } else {
        json!({"behavior":"deny","message":"user_input_cancelled","interrupt":true})
    };
    Ok(
        json!({"type":"control_response","response":{"subtype":"success","request_id":request.provider_request_id,"response":response}}),
    )
}

#[allow(clippy::too_many_arguments)]
fn spawn_reader(
    stdout: ChildStdout,
    prefetch: Vec<String>,
    queue: Arc<(Mutex<QueueState>, Condvar)>,
    events: Arc<Mutex<VecDeque<ManagedEvent>>>,
    alive: Arc<AtomicBool>,
    journal: Journal,
    session_store: SessionStoreHandle,
    provider_kind: String,
    pinned_model: Option<String>,
    resume_bootstrap: Option<ResumeBootstrap>,
    writer: Writer,
    permission_observation: Arc<Mutex<Option<(String, Value, u64, Option<String>)>>>,
    launch_args: Vec<String>,
    user_inputs: Arc<Mutex<Vec<(ChildProviderRequest, bool)>>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let (stdout, prefetch) = if let Some(bootstrap) = resume_bootstrap {
            bootstrap_resume_in_reader(stdout, bootstrap, &events, &session_store)
        } else {
            (stdout, prefetch)
        };
        let lines = prefetch
            .into_iter()
            .map(Ok)
            .chain(BufReader::new(stdout).lines());
        let mut provider_session = None;
        let mut pending_writes = std::collections::HashMap::new();
        let observation_cwd = std::env::current_dir().ok();
        let mut permission_requests = std::collections::HashSet::new();
        for line in lines {
            let Ok(line) = line else { break };
            let raw = line.as_bytes().to_vec();
            let value = match serde_json::from_str::<Value>(&line) {
                Ok(value) => value,
                Err(_) => {
                    push_source(
                        &events,
                        raw,
                        ManagedEventKind::Error {
                            detail: "ligne Claude stream-json invalide".to_string(),
                        },
                    );
                    continue;
                }
            };
            if value.get("type").and_then(Value::as_str) == Some("system") {
                let mode = value
                    .get("permissionMode")
                    .or_else(|| value.get("permission_mode"))
                    .and_then(Value::as_str);
                let session = value.get("session_id").and_then(Value::as_str);
                let mut observed = permission_observation.lock().recover_poison();
                if let Some(mode) = mode {
                    if matches!(
                        mode,
                        "default"
                            | "acceptEdits"
                            | "bypassPermissions"
                            | "plan"
                            | "dontAsk"
                            | "auto"
                    ) {
                        let session = session
                            .map(str::to_owned)
                            .or_else(|| observed.as_ref().map(|o| o.0.clone()));
                        let tools = value
                            .get("tools")
                            .filter(|v| {
                                v.as_array().is_some_and(|a| a.iter().all(Value::is_string))
                            })
                            .cloned()
                            .or_else(|| observed.as_ref().map(|o| o.1["tools"].clone()));
                        if let (Some(session), Some(tools)) = (session, tools) {
                            let mut policy = json!({"kind":"claude","permission_mode":mode,"tools":tools,"permission_callback":{"kind":"native_wrapper","tool_approval":if matches!(mode,"bypassPermissions"|"auto"){"allow"}else{"prompt"},"plan_exit":"deny"},"settings_sources":"provider_default"});
                            if crate::protocol::claude_launch_policy_options(
                                &launch_args,
                                &mut policy,
                            )
                            .is_err()
                            {
                                *observed = None;
                                continue;
                            }
                            let cwd = value
                                .get("cwd")
                                .and_then(Value::as_str)
                                .map(str::to_owned)
                                .or_else(|| observed.as_ref().and_then(|o| o.3.clone()));
                            let revision = observed
                                .as_ref()
                                .map(|o| {
                                    o.2.saturating_add(u64::from(
                                        o.0 != session || o.1 != policy || o.3 != cwd,
                                    ))
                                })
                                .unwrap_or(1);
                            *observed = Some((session, policy, revision, cwd));
                        }
                    } else {
                        *observed = None;
                    }
                }
            }
            if let Some(session_id) = session_id_from_system_init(&value) {
                provider_session = Some(session_id.clone());
                if let Some(store) = session_store.lock().recover_poison().as_ref() {
                    let _ = store.store(&session_id);
                }
                push_source(
                    &events,
                    raw.clone(),
                    ManagedEventKind::ProviderContextObserved {
                        identity: ManagedProviderIdentity {
                            provider_kind: provider_kind.clone(),
                            provider_session_id: Some(session_id),
                            provider_thread_id: None,
                            active_turn_id: None,
                            provider_item_id: None,
                            capabilities_revision: None,
                            provider_observation: None,
                        },
                    },
                );
            }
            let kind = value
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("inconnu");
            if let Some(active) = queue.0.lock().recover_poison().active.as_mut() {
                active.background_tasks.observe(&value);
            }
            for path in confirmed_tool_writes(&value, &mut pending_writes) {
                let message_id = queue
                    .0
                    .lock()
                    .recover_poison()
                    .active
                    .as_ref()
                    .map(|a| a.message_id.clone());
                if let (Some(message_id), Some(cwd)) = (message_id, observation_cwd.as_deref())
                    && let Some(payload) = crate::journal::confirmed_write_payload(&path, cwd)
                {
                    record_or_terminal(&journal, &events, "update", Some(&message_id), payload);
                }
            }
            // (A) Retranscription : chaque delta texte → journal `update`.
            // Sans cela le fil ne voit que « a travaillé Ns ».
            if let Some(delta) = value.pointer("/event/delta/text").and_then(Value::as_str) {
                let message_id = {
                    let mut state = queue.0.lock().recover_poison();
                    if let Some(active) = state.active.as_mut() {
                        active.response.push_str(delta);
                        active.text_updates = active.text_updates.saturating_add(1);
                        Some(active.message_id.clone())
                    } else {
                        None
                    }
                };
                if let Some(message_id) = message_id {
                    record_or_terminal(
                        &journal,
                        &events,
                        "update",
                        Some(&message_id),
                        json!({ "kind": "text", "content": delta }),
                    );
                }
            }
            // (B) Repli : message `assistant` complet si aucun delta n'a été
            // journalisé (stdout sans include-partial, ou forme agrégée seule).
            //
            // Anti-doublon assistant↔deltas — écarté par écrit, pas par garde
            // supplémentaire : `ensure_stream_arguments` impose
            // `--include-partial-messages`. Sous ce contrat CLI/SDK, les
            // `stream_event` (deltas) arrivent pendant la génération, PUIS le
            // message `assistant` agrégé, PUIS `result`. L'ordre inverse
            // (assistant puis deltas) n'est pas produit ; la garde
            // `text_updates == 0` couvre donc le seul ordre réel
            // (deltas→assistant→result, ou assistant→result sans partial).
            // Une fixture assistant-puis-deltas testerait un fantôme de
            // protocole, pas un trou du pilote.
            if kind == "control_request"
                && value.pointer("/request/subtype").and_then(Value::as_str) == Some("can_use_tool")
            {
                if let Some(request_id) =
                    value
                        .get("request_id")
                        .and_then(Value::as_str)
                        .filter(|id| {
                            !id.is_empty() && id.len() <= 256 && !id.chars().any(char::is_control)
                        })
                {
                    if permission_requests.len() >= 4096 {
                        break;
                    }
                    if permission_requests.insert(request_id.to_owned()) {
                        let turn = queue
                            .0
                            .lock()
                            .recover_poison()
                            .active
                            .as_ref()
                            .map(|a| a.message_id.clone());
                        if let (Some(session), Some(turn)) =
                            (provider_session.as_deref(), turn.as_deref())
                            && let Some(request) = claude_user_input(&value, session, turn)
                        {
                            let mut pending = user_inputs.lock().recover_poison();
                            pending.retain(|(r, _)| r.turn_id == turn);
                            if pending.len() < 4 {
                                pending.push((request.clone(), false));
                                drop(pending);
                                push_source(
                                    &events,
                                    raw,
                                    ManagedEventKind::ProviderRequest { request },
                                );
                                continue;
                            }
                        }
                        if let Some(active) = queue.0.lock().recover_poison().active.as_mut() {
                            active.permission_denied = true;
                        }
                        let frame = json!({"type":"control_response","response":{"subtype":"success","request_id":request_id,"response":{"behavior":"deny","message":"provider_permission_denied","interrupt":true}}});
                        let written = writer.lock().map_err(|_| ()).and_then(|mut lock| {
                            let stdin = lock.as_mut().ok_or(())?;
                            writeln!(stdin, "{frame}").map_err(|_| ())?;
                            stdin.flush().map_err(|_| ())
                        });
                        record_or_terminal(
                            &journal,
                            &events,
                            "permission_denied",
                            None,
                            json!({"request_id":request_id,"code":"provider_permission_denied"}),
                        );
                        if written.is_err() {
                            break;
                        }
                    }
                } else if let Some(active) = queue.0.lock().recover_poison().active.as_mut() {
                    active.permission_denied = true;
                }
                continue;
            }
            if kind == "control_response" {
                // Forme mesurée le 28/08 : {"type":"control_response",
                // "response":{"subtype":"success","request_id":…,
                // "response":{"still_queued":[]}}}. Corrélation stricte : seul
                // l'accusé portant le `request_id` émis solde l'interruption.
                // Un accusé étranger est ignoré plutôt que d'éteindre une
                // interruption qui n'est pas la sienne.
                if let Some(acked) = value
                    .pointer("/response/request_id")
                    .and_then(Value::as_str)
                {
                    let matched = {
                        let mut state = queue.0.lock().recover_poison();
                        if state.pending_interrupt.as_deref() == Some(acked) {
                            state.pending_interrupt = None;
                            true
                        } else {
                            false
                        }
                    };
                    if matched {
                        record_or_terminal(
                            &journal,
                            &events,
                            "interrupt_acked",
                            None,
                            json!({
                                "request_id": acked,
                                "subtype": value
                                    .pointer("/response/subtype")
                                    .and_then(Value::as_str),
                            }),
                        );
                    }
                }
                continue;
            }
            let mut assistant_limit_observation = None;
            if kind == "assistant" {
                // T3 conserve ce code structuré même si le résultat dit api_error.
                // Une réponse ultérieure remplace le refus ; aucune date n'est déduite du texte.
                let limited = value.get("error").and_then(Value::as_str) == Some("rate_limit");
                let mut state = queue.0.lock().recover_poison();
                if let Some(active) = state.active.as_mut() {
                    // Une vraie réponse sans erreur lève un refus sans fenêtre
                    // d'une ancienne mission du même compte. Elle ne prouve aucun crédit.
                    let accepted = value.get("error").is_none_or(Value::is_null)
                        && value.get("isApiErrorMessage") != Some(&Value::Bool(true))
                        && value
                            .pointer("/message/content")
                            .and_then(Value::as_array)
                            .is_some_and(|content| !content.is_empty());
                    if (limited && !active.structured_rate_limit_observed) || (!limited && accepted)
                    {
                        assistant_limit_observation = Some(limited);
                    }
                    active.latest_assistant_rate_limited = limited;
                    active.latest_assistant_api_error = value.get("error").and_then(Value::as_str)
                        == Some("authentication_failed")
                        || (value.get("error").is_none_or(Value::is_null)
                            && value.get("isApiErrorMessage") == Some(&Value::Bool(true)));
                }
                drop(state);
                // TOOL : le content_block_start arrive tôt avec name mais
                // input={}. On journalise ici le bloc assistant qui porte
                // name + input complet (mesure /tmp/flux-outil.jsonl L19).
                // Pas de tool_result : un Read de fichier peut saturer le
                // journal ; le fil garde nom + arguments, pas le rendu.
                record_tool_uses_from_assistant(&journal, &events, &queue, &value);
                let assistant_text = value
                    .pointer("/message/content")
                    .and_then(Value::as_array)
                    .map(|blocks| {
                        blocks
                            .iter()
                            .filter_map(|block| {
                                if block.get("type").and_then(Value::as_str) == Some("text") {
                                    block.get("text").and_then(Value::as_str)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("")
                    })
                    .unwrap_or_default();
                if !assistant_text.is_empty() && !limited {
                    let message_id = {
                        let mut state = queue.0.lock().recover_poison();
                        if let Some(active) = state.active.as_mut() {
                            if active.text_updates == 0 {
                                if active.response.is_empty() {
                                    active.response.push_str(&assistant_text);
                                }
                                active.text_updates = active.text_updates.saturating_add(1);
                                Some(active.message_id.clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    };
                    if let Some(message_id) = message_id {
                        record_or_terminal(
                            &journal,
                            &events,
                            "update",
                            Some(&message_id),
                            json!({ "kind": "text", "content": assistant_text }),
                        );
                    }
                }
            }
            if kind == "result" {
                let tool_denials = value
                    .get("permission_denials")
                    .and_then(Value::as_array)
                    .map_or(0, Vec::len);
                let (interrupted_for_permission, terminal) = {
                    let state = queue.0.lock().recover_poison();
                    let active = state.active.as_ref();
                    let interrupted = active.is_some_and(|active| active.permission_denied);
                    let terminal = classify_claude_result(
                        &value,
                        interrupted,
                        active.is_some_and(|active| active.latest_assistant_rate_limited),
                        active
                            .filter(|active| active.latest_assistant_api_error)
                            .map(|active| active.response.as_str()),
                    );
                    (interrupted, terminal)
                };
                let active_id = queue
                    .0
                    .lock()
                    .recover_poison()
                    .active
                    .as_ref()
                    .map(|active| active.message_id.clone());
                if let Some(message_id) = active_id {
                    record_or_terminal(
                        &journal,
                        &events,
                        "provider_result",
                        Some(&message_id),
                        claude_result_metadata(&value, &terminal, interrupted_for_permission),
                    );
                }
                if tool_denials > 0 {
                    let mut state = queue.0.lock().recover_poison();
                    if let Some(active) = state.active.as_mut() {
                        if active.response.is_empty()
                            && let Some(result) = value.get("result").and_then(Value::as_str)
                        {
                            active.response.push_str(result);
                        }
                        if matches!(terminal, ManagedTerminal::Completed) {
                            active.response.insert_str(0, &format!(
                                "Bridget : {tool_denials} appel(s) d'outil refusé(s) par les permissions ou les hooks. Les refus sont conservés.\n\n"
                            ));
                        }
                        record_or_terminal(
                            &journal,
                            &events,
                            "permission_denied",
                            Some(&active.message_id),
                            json!({"code":"provider_permission_denied","count":tool_denials,"turn_completed":matches!(terminal,ManagedTerminal::Completed)}),
                        );
                    }
                }
                // (C) Issue attestée : `terminal_reason` du résultat Claude
                // (ex. "completed"), pas le champ stop_reason fournisseur
                // (ex. "end_turn") — attach lit payload.stop_reason.
                let stop_reason = if matches!(terminal, ManagedTerminal::Completed) {
                    Some("completed".to_owned())
                } else {
                    value
                        .get("terminal_reason")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|reason| !reason.is_empty())
                        .map(str::to_string)
                        .or_else(|| {
                            value
                                .get("stop_reason")
                                .and_then(Value::as_str)
                                .map(str::trim)
                                .filter(|reason| !reason.is_empty())
                                .map(str::to_string)
                        })
                };
                let pending_result_update = {
                    let mut state = queue.0.lock().recover_poison();
                    if let Some(active) = state.active.as_mut() {
                        active.stop_reason = stop_reason;
                        // (B) Repli result-only : texte final dans le fil SSI
                        // aucun update texte n'a déjà été journalisé (pas de
                        // duplication après deltas).
                        if active.text_updates == 0
                            && let Some(result) = value.get("result").and_then(Value::as_str)
                            && !result.is_empty()
                        {
                            if active.response.is_empty() {
                                active.response.push_str(result);
                            }
                            active.text_updates = 1;
                            Some((active.message_id.clone(), result.to_string()))
                        } else {
                            if active.response.is_empty()
                                && let Some(result) = value.get("result").and_then(Value::as_str)
                            {
                                active.response.push_str(result);
                            }
                            None
                        }
                    } else {
                        None
                    }
                };
                if let Some((message_id, result)) = pending_result_update {
                    // (B) Réponse finale absente des deltas → journaliser `result`.
                    record_or_terminal(
                        &journal,
                        &events,
                        "update",
                        Some(&message_id),
                        json!({ "kind": "text", "content": result }),
                    );
                }
                let mut state = queue.0.lock().recover_poison();
                if let Some(active) = state.active.as_mut() {
                    if matches!(terminal, ManagedTerminal::Completed)
                        && active.background_tasks.pending()
                    {
                        // Comme T3, une fin de tour ne clôt pas les tâches du fournisseur.
                        // Le prochain rapport reste dans la même mission et son délai initial.
                        active.response.clear();
                        active.text_updates = 0;
                        active.stop_reason = None;
                    } else {
                        let _ = active.completion.send(terminal);
                        active.terminal_sent = true;
                    }
                }
            }
            let managed_event = if let Some(limited) = assistant_limit_observation {
                ManagedEventKind::RateLimitObserved {
                    window: "unknown".into(),
                    status: if limited { "rejected" } else { "allowed" }.into(),
                    resets_at: None,
                    used_percent: None,
                    credits: None,
                }
            } else if kind == "rate_limit_event" {
                let event = rate_limit_event(&value).unwrap_or_else(|| ManagedEventKind::Update {
                    detail: "événement Claude rate_limit_event incomplet".to_string(),
                });
                if matches!(&event, ManagedEventKind::RateLimitObserved { status, .. } if status == "rejected")
                    && let Some(active) = queue.0.lock().recover_poison().active.as_mut()
                {
                    // Le code assistant ne doit pas ajouter un refus « unknown »
                    // permanent à une fenêtre dont le fournisseur a donné le retour.
                    active.structured_rate_limit_observed = true;
                }
                event
            } else if let Some(served) = served_model_from_claude(&value) {
                maybe_record_mismatch(&journal, &events, pinned_model.as_deref(), &served);
                ManagedEventKind::ModelObserved { model: served }
            } else if let Some(usage) = usage_event(&value) {
                usage
            } else {
                ManagedEventKind::Update {
                    detail: format!("événement Claude: {kind}"),
                }
            };
            push_source(&events, raw, managed_event);
        }
        alive.store(false, Ordering::SeqCst);
        let mut state = queue.0.lock().recover_poison();
        // Un fournisseur qui sort juste après son résultat ferme stdout avant
        // que le fil de la mission ne relève sa réponse : la lui laisser.
        if state
            .active
            .as_ref()
            .is_some_and(|active| !active.terminal_sent)
            && let Some(active) = state.active.take()
        {
            let _ = active.completion.send(ManagedTerminal::Failed {
                detail: "stdout Claude fermé".to_string(),
            });
        }
        queue.1.notify_all();
    })
}

/// Extrait uniquement un fait complet du schéma `rate_limit_event` attesté par
/// Claude. Une date absente reste `None` : aucune heure de retour n'est déduite.
fn pinned_model_from_args(args: &[String]) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == "--model")
        .map(|pair| {
            pair[1]
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string()
        })
        .filter(|model| !model.is_empty())
}

/// Le modèle servi n'est lu que sur `system/init`. Un autre événement, même
/// porteur d'un champ `model`, ne constitue pas un verdict.
fn served_model_from_claude(value: &Value) -> Option<String> {
    if value.get("type").and_then(Value::as_str) != Some("system") {
        return None;
    }
    if value.get("subtype").and_then(Value::as_str) != Some("init") {
        return None;
    }
    value
        .get("model")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|model| !model.is_empty())
        .map(str::to_string)
}

fn maybe_record_mismatch(
    journal: &Journal,
    events: &Arc<Mutex<VecDeque<ManagedEvent>>>,
    pinned: Option<&str>,
    served: &str,
) {
    let Some(pinned) = pinned.filter(|pinned| *pinned != served) else {
        return;
    };
    record_or_terminal(
        journal,
        events,
        "model_mismatch",
        None,
        json!({ "pinned": pinned, "served": served }),
    );
}

fn rate_limit_event(value: &Value) -> Option<ManagedEventKind> {
    let info = value.get("rate_limit_info")?;
    let window = info
        .get("rateLimitType")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .trim();
    let status = info.get("status")?.as_str()?.trim();
    if window.is_empty() || status.is_empty() {
        return None;
    }
    Some(ManagedEventKind::RateLimitObserved {
        credits: None,
        window: window.to_string(),
        status: status.to_string(),
        resets_at: info.get("resetsAt").and_then(Value::as_i64),
        used_percent: percent_0_100(info.get("usedPercent").or_else(|| info.get("utilization"))),
    })
}

fn percent_0_100(value: Option<&Value>) -> Option<u8> {
    let pct = value.and_then(|v| v.as_f64().or_else(|| v.as_i64().map(|n| n as f64)))?;
    if (0.0..=100.0).contains(&pct) {
        Some(pct.round() as u8)
    } else {
        None
    }
}

/// Extrait la consommation d'un tour depuis l'événement terminal `result`.
///
/// Un tour Claude émet souvent `assistant` (usage sous `/message/usage`) puis
/// `result` (usage à la racine) avec les mêmes compteurs. Ne lire que `result`
/// : un seul échantillon par tour, jamais un facturable doublé. Schéma attesté
/// complet requis ; un champ manquant → aucun fait (jamais zéro inventé).
fn usage_event(value: &Value) -> Option<ManagedEventKind> {
    if value.get("type").and_then(Value::as_str) != Some("result") {
        return None;
    }
    let usage = value
        .get("usage")
        .or_else(|| value.pointer("/message/usage"))?;
    let input_tokens = usage.get("input_tokens")?.as_u64()?;
    let output_tokens = usage.get("output_tokens")?.as_u64()?;
    let cache_creation_input_tokens = usage.get("cache_creation_input_tokens")?.as_u64()?;
    let cache_read_input_tokens = usage.get("cache_read_input_tokens")?.as_u64()?;
    Some(ManagedEventKind::UsageObserved {
        input_tokens,
        output_tokens,
        cache_creation_input_tokens,
        cache_read_input_tokens,
    })
}

/// Plafond du champ `detail` (input sérialisé). Au-delà : troncature + «…».
/// Choix étroit : garder le journal lisible si un outil reçoit un gros blob.
const TOOL_INPUT_DETAIL_MAX: usize = 512;

fn confirmed_tool_writes(
    value: &Value,
    pending: &mut std::collections::HashMap<String, Option<String>>,
) -> Vec<String> {
    let kind = value.get("type").and_then(Value::as_str);
    if kind == Some("result") {
        pending.clear();
        return vec![];
    }
    let Some(blocks) = value.pointer("/message/content").and_then(Value::as_array) else {
        return vec![];
    };
    let mut paths = vec![];
    for block in blocks {
        if kind == Some("assistant")
            && block.get("type").and_then(Value::as_str) == Some("tool_use")
        {
            let Some(id) = block
                .get("id")
                .and_then(Value::as_str)
                .filter(|id| id.len() <= 256)
            else {
                continue;
            };
            let name = block.get("name").and_then(Value::as_str).unwrap_or("");
            if let Some(path) = crate::journal::tool_write_path(name, &block["input"]) {
                if pending.len() >= 256 && !pending.contains_key(id) {
                    log::warn!("observation_gap: trop d'écritures Claude dans ce tour");
                    continue;
                }
                pending
                    .entry(id.into())
                    .or_insert_with(|| Some(path.into()));
            }
        } else if kind == Some("user")
            && block.get("type").and_then(Value::as_str) == Some("tool_result")
            && let Some(id) = block.get("tool_use_id").and_then(Value::as_str)
            && let Some(path) = pending.get_mut(id).and_then(Option::take)
            && block.get("is_error").and_then(Value::as_bool) != Some(true)
        {
            paths.push(path);
        }
    }
    paths
}

/// Journalise chaque bloc `tool_use` d'un message assistant.
/// Vocabulaire imposé : `kind=tool` (pas `tool_call`).
fn record_tool_uses_from_assistant(
    journal: &Journal,
    events: &Arc<Mutex<VecDeque<ManagedEvent>>>,
    queue: &Arc<(Mutex<QueueState>, Condvar)>,
    value: &Value,
) {
    let Some(blocks) = value.pointer("/message/content").and_then(Value::as_array) else {
        return;
    };
    let message_id = {
        let state = queue.0.lock().recover_poison();
        state
            .active
            .as_ref()
            .map(|active| active.message_id.clone())
    };
    let Some(message_id) = message_id else {
        return;
    };
    for block in blocks {
        if block.get("type").and_then(Value::as_str) != Some("tool_use") {
            continue;
        }
        let Some(name) = block
            .get("name")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|name| !name.is_empty())
        else {
            continue;
        };
        let input = block.get("input").cloned().unwrap_or(Value::Null);
        let detail = compact_tool_input(&input);
        record_or_terminal(
            journal,
            events,
            "update",
            Some(&message_id),
            json!({
                "kind": crate::act_kind::pilot_kind_str(crate::JournalUpdateKind::Tool),
                "text": name,
                "tool": name,
                "detail": detail,
            }),
        );
    }
}

fn compact_tool_input(input: &Value) -> String {
    let raw = match input {
        Value::Null => String::new(),
        other => other.to_string(),
    };
    if raw.chars().count() <= TOOL_INPUT_DETAIL_MAX {
        return raw;
    }
    let truncated: String = raw.chars().take(TOOL_INPUT_DETAIL_MAX).collect();
    format!("{truncated}…")
}

fn record(
    journal: &Journal,
    event: &str,
    message_id: Option<&str>,
    payload: Value,
) -> Result<(), String> {
    journal
        .lock()
        .recover_poison()
        .as_ref()
        .map_or(Ok(()), |journal| {
            journal.enqueue(event, message_id, payload)
        })
}

/// Miroir ACP `record_or_terminal` : un échec d'enqueue (dont kind hors
/// vocabulaire) devient `JournalFailed` — visible, jamais avalé en silence.
fn record_or_terminal(
    journal: &Journal,
    events: &Arc<Mutex<VecDeque<ManagedEvent>>>,
    event: &str,
    message_id: Option<&str>,
    payload: Value,
) {
    if let Err(detail) = record(journal, event, message_id, payload) {
        push_internal(events, ManagedEventKind::JournalFailed { detail });
    }
}

fn push_internal(events: &Arc<Mutex<VecDeque<ManagedEvent>>>, kind: ManagedEventKind) {
    let raw = serde_json::to_vec(&json!({ "kind": "claude_internal" }))
        .expect("fait interne Claude sérialisable");
    events
        .lock()
        .recover_poison()
        .push_back(ManagedEvent::internal(
            ManagedEventSource::ClaudeStreamJson,
            raw,
            kind,
        ));
}

fn push_source(events: &Arc<Mutex<VecDeque<ManagedEvent>>>, raw: Vec<u8>, kind: ManagedEventKind) {
    events.lock().recover_poison().push_back(ManagedEvent {
        source: ManagedEventSource::ClaudeStreamJson,
        origin: ManagedEventOrigin::SourceLine,
        raw,
        kind,
    });
}

#[cfg(test)]
#[path = "claude_stream_json_tests.rs"]
mod tests;
