//! Transport ACP synchrone : un lecteur stdout, un writer sérialisé et un
//! worker FIFO. Le lecteur est l'unique propriétaire du flux de l'adaptateur.

use crate::poison::RecoverPoison;
use crate::journal::{JournalLiveFeed, JournalWriter, with_turn_failed_kind};
use crate::managed_session::{
    ManagedEvent, ManagedEventKind, ManagedEventSource, ManagedProviderIdentity, ManagedSession,
    ManagedSessionDescriptor, ManagedTerminal,
};
use crate::protocol::{PresenceMode, ProviderObservation};
use crate::transport::{Transport, TransportError};
use bridget_core::{BridgetMessage, MessageIntent, MessageOrigin};
use serde_json::{Value, json};
use std::collections::{HashMap, VecDeque};
use std::ffi::OsString;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::thread;
use std::time::{Duration, SystemTime};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const CANCEL_GRACE: Duration = Duration::from_millis(250);
const CANCEL_POLL: Duration = Duration::from_millis(20);

#[derive(Debug, Clone)]
pub struct AcpOptions {
    pub command: String,
    pub args: Vec<String>,
    pub queue_capacity: usize,
    pub permissions: String,
    pub notify_timeout_secs: u64,
    /// Baseline relevée avant lancement, absente pour un registre historique.
    pub provider_observation: Option<ProviderObservation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnState {
    Idle,
    InProgress {
        message_id: String,
        since: SystemTime,
    },
}

#[derive(Debug, Clone)]
pub enum AcpEvent {
    TurnStarted {
        message_id: String,
    },
    /// La frame `session/prompt` a été écrite puis flushée vers l'adaptateur.
    /// Ce n'est pas encore une fin de tour ni un accusé de résultat ACP.
    PromptDispatched {
        message_id: String,
    },
    TurnFinished {
        message: Box<BridgetMessage>,
        response: String,
        stop_reason: String,
    },
    DeliveryRejected {
        message_id: String,
        reason: String,
    },
    Update {
        detail: String,
    },
    Error {
        detail: String,
    },
    /// L'écriture append-only est devenue non fiable : le wrapper doit arrêter
    /// le transport afin de ne jamais continuer avec un journal incomplet.
    JournalFailed {
        detail: String,
    },
}

/// File des événements ACP conservant leur provenance jusqu'à la frontière
/// commune. Les insertions historiques gardent `push_back` et sont donc
/// explicitement internes ; seul le lecteur stdout peut inscrire une ligne
/// source via `push_source`.
#[derive(Default)]
pub struct AcpEventQueue(VecDeque<QueuedAcpEvent>);

struct QueuedAcpEvent {
    event: AcpEvent,
    raw_source_line: Option<Vec<u8>>,
}

impl AcpEventQueue {
    pub(crate) fn push_back(&mut self, event: AcpEvent) {
        self.0.push_back(QueuedAcpEvent {
            event,
            raw_source_line: None,
        });
    }

    fn push_source(&mut self, event: AcpEvent, raw_source_line: Vec<u8>) {
        self.0.push_back(QueuedAcpEvent {
            event,
            raw_source_line: Some(raw_source_line),
        });
    }

    #[cfg(test)]
    fn pop_front(&mut self) -> Option<AcpEvent> {
        self.0.pop_front().map(|queued| queued.event)
    }

    fn drain_acp(&mut self) -> Vec<AcpEvent> {
        self.0.drain(..).map(|queued| queued.event).collect()
    }

    fn drain_managed(&mut self) -> Vec<ManagedEvent> {
        self.0
            .drain(..)
            .map(|queued| match queued.raw_source_line {
                Some(raw) => ManagedEvent::source_line(
                    ManagedEventSource::Acp,
                    raw,
                    managed_kind(queued.event),
                ),
                None => ManagedEvent::internal(
                    ManagedEventSource::Acp,
                    acp_event_bytes(&queued.event),
                    managed_kind(queued.event),
                ),
            })
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn iter(&self) -> impl Iterator<Item = &AcpEvent> {
        self.0.iter().map(|queued| &queued.event)
    }
}

type Waiters = Arc<Mutex<HashMap<String, mpsc::Sender<Result<Value, String>>>>>;
type Writer = Arc<Mutex<Option<ChildStdin>>>;
type Completions = Arc<Mutex<HashMap<String, mpsc::Sender<()>>>>;
type Journal = Arc<Mutex<Option<JournalWriter>>>;
type Clock = Arc<dyn Fn() -> SystemTime + Send + Sync>;

struct SpawnContext<'a> {
    inherit_stderr: bool,
    environment: &'a [(OsString, OsString)],
    mcp_servers: Vec<Value>,
}

struct ActiveTurn {
    message_id: String,
    cancellation: mpsc::Sender<String>,
    cancelled: Arc<AtomicBool>,
}

struct QueueState {
    messages: VecDeque<BridgetMessage>,
    active: Option<ActiveTurn>,
    closed: bool,
}

struct TurnWorker {
    queue: Arc<(Mutex<QueueState>, Condvar)>,
    writer: Writer,
    waiters: Waiters,
    completions: Completions,
    next_id: Arc<AtomicU64>,
    alive: Arc<AtomicBool>,
    state: Arc<Mutex<TurnState>>,
    events: Arc<Mutex<AcpEventQueue>>,
    response: Arc<Mutex<String>>,
    /// Accumulé depuis `agent_thought_chunk` ; vidé en début de tour.
    /// Une seule ligne journal `event=reasoning` est émise en fin de tour
    /// (alignement C3 avec L3 Codex — pas de DTO ManagedEvent).
    reasoning_raw: Arc<Mutex<String>>,
    session_id: String,
    notify_timeout: Duration,
    cancel_grace: Duration,
    poll_interval: Duration,
    child: Arc<Mutex<Child>>,
    journal: Journal,
    private_profile_instructions: Arc<Mutex<Option<String>>>,
    clock: Clock,
    test_observer: Option<mpsc::Sender<AcpEvent>>,
}

pub struct AcpTransport {
    connection_id: String,
    alive: Arc<AtomicBool>,
    shutdown_started: Arc<AtomicBool>,
    state: Arc<Mutex<TurnState>>,
    events: Arc<Mutex<AcpEventQueue>>,
    queue: Arc<(Mutex<QueueState>, Condvar)>,
    provider_observation: Option<ProviderObservation>,
    queue_capacity: usize,
    writer: Writer,
    session_id: String,
    child: Arc<Mutex<Child>>,
    reader_handle: Mutex<Option<thread::JoinHandle<()>>>,
    worker_handle: Mutex<Option<thread::JoinHandle<()>>>,
    journal: Journal,
    private_profile_instructions: Arc<Mutex<Option<String>>>,
    clock: Clock,
    context_published: AtomicBool,
}

impl AcpTransport {
    /// PID de l'adaptateur ACP, utilisé pour un marqueur local de filiation.
    pub fn process_id(&self) -> u32 {
        self.child
            .lock()
            .recover_poison()
            .id()
    }

    pub fn spawn(options: AcpOptions) -> Result<Self, TransportError> {
        Self::spawn_with_clock(options, Arc::new(SystemTime::now))
    }

    /// Variante avec environnement explicite pour les processus enfants.
    pub fn spawn_with_environment(
        options: AcpOptions,
        environment: &[(OsString, OsString)],
    ) -> Result<Self, TransportError> {
        Self::spawn_with_environment_and_mcp(options, environment, Vec::new())
    }

    /// Variante avec environnement et serveurs MCP propres à cette session.
    pub fn spawn_with_environment_and_mcp(
        options: AcpOptions,
        environment: &[(OsString, OsString)],
        mcp_servers: Vec<Value>,
    ) -> Result<Self, TransportError> {
        Self::spawn_with_clock_and_cancel_grace_inner(
            options,
            Arc::new(SystemTime::now),
            CANCEL_GRACE,
            CANCEL_POLL,
            None,
            SpawnContext {
                inherit_stderr: false,
                environment,
                mcp_servers,
            },
        )
    }

    /// Variante supervisée : l'adaptateur conserve le stderr hérité du wrapper.
    pub fn spawn_inheriting_stderr(options: AcpOptions) -> Result<Self, TransportError> {
        Self::spawn_inheriting_stderr_with_environment(options, &[])
    }

    /// Variante supervisée avec environnement explicite du processus enfant.
    pub fn spawn_inheriting_stderr_with_environment(
        options: AcpOptions,
        environment: &[(OsString, OsString)],
    ) -> Result<Self, TransportError> {
        Self::spawn_inheriting_stderr_with_environment_and_mcp(options, environment, Vec::new())
    }

    /// Variante supervisée avec environnement et serveurs MCP éphémères.
    pub fn spawn_inheriting_stderr_with_environment_and_mcp(
        options: AcpOptions,
        environment: &[(OsString, OsString)],
        mcp_servers: Vec<Value>,
    ) -> Result<Self, TransportError> {
        Self::spawn_with_clock_and_cancel_grace_inner(
            options,
            Arc::new(SystemTime::now),
            CANCEL_GRACE,
            CANCEL_POLL,
            None,
            SpawnContext {
                inherit_stderr: true,
                environment,
                mcp_servers,
            },
        )
    }

    fn spawn_with_clock(options: AcpOptions, clock: Clock) -> Result<Self, TransportError> {
        Self::spawn_with_clock_and_cancel_grace(options, clock, CANCEL_GRACE, CANCEL_POLL, None)
    }

    fn spawn_with_clock_and_cancel_grace(
        options: AcpOptions,
        clock: Clock,
        cancel_grace: Duration,
        poll_interval: Duration,
        test_observer: Option<mpsc::Sender<AcpEvent>>,
    ) -> Result<Self, TransportError> {
        Self::spawn_with_clock_and_cancel_grace_inner(
            options,
            clock,
            cancel_grace,
            poll_interval,
            test_observer,
            SpawnContext {
                inherit_stderr: false,
                environment: &[],
                mcp_servers: Vec::new(),
            },
        )
    }

    fn spawn_with_clock_and_cancel_grace_inner(
        options: AcpOptions,
        clock: Clock,
        cancel_grace: Duration,
        poll_interval: Duration,
        test_observer: Option<mpsc::Sender<AcpEvent>>,
        context: SpawnContext<'_>,
    ) -> Result<Self, TransportError> {
        if options.queue_capacity == 0 {
            return Err(TransportError::DeliveryFailed(
                "queue ACP de capacité nulle".to_string(),
            ));
        }
        let stderr = if context.inherit_stderr {
            Stdio::inherit()
        } else {
            Stdio::null()
        };
        let mut child = Command::new(&options.command)
            .args(&options.args)
            .envs(context.environment.iter().cloned())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(stderr)
            .spawn()
            .map_err(|err| {
                TransportError::Io(format!("impossible de lancer l'adaptateur ACP: {err}"))
            })?;
        let pid = child.id();
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| TransportError::Io("stdin ACP absent".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| TransportError::Io("stdout ACP absent".to_string()))?;
        let alive = Arc::new(AtomicBool::new(true));
        let shutdown_started = Arc::new(AtomicBool::new(false));
        let events = Arc::new(Mutex::new(AcpEventQueue::default()));
        let response = Arc::new(Mutex::new(String::new()));
        let reasoning_raw = Arc::new(Mutex::new(String::new()));
        let writer = Arc::new(Mutex::new(Some(stdin)));
        let waiters: Waiters = Arc::new(Mutex::new(HashMap::new()));
        let completions: Completions = Arc::new(Mutex::new(HashMap::new()));
        let journal: Journal = Arc::new(Mutex::new(None));
        let next_id = Arc::new(AtomicU64::new(1));
        let queue = Arc::new((
            Mutex::new(QueueState {
                messages: VecDeque::new(),
                active: None,
                closed: false,
            }),
            Condvar::new(),
        ));
        let active_session = Arc::new(Mutex::new(None));
        let child = Arc::new(Mutex::new(child));
        let reader_handle = spawn_reader(
            stdout,
            writer.clone(),
            waiters.clone(),
            completions.clone(),
            events.clone(),
            response.clone(),
            reasoning_raw.clone(),
            alive.clone(),
            options.permissions.clone(),
            queue.clone(),
            active_session.clone(),
            journal.clone(),
            test_observer.clone(),
        );

        let setup = (|| -> Result<String, TransportError> {
            let initialize = request(
                &writer,
                &waiters,
                &next_id,
                "initialize",
                json!({
                    "protocolVersion": 1,
                    "clientCapabilities": {},
                    "clientInfo": { "name": "bridget", "version": env!("CARGO_PKG_VERSION") }
                }),
            )?;
            ensure_protocol_version(&initialize)?;
            let session = request(
                &writer,
                &waiters,
                &next_id,
                "session/new",
                json!({
                    "cwd": std::env::current_dir().map_err(|err| TransportError::Io(err.to_string()))?,
                    "mcpServers": context.mcp_servers
                }),
            )?;
            let session_id = session
                .get("sessionId")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    TransportError::DeliveryFailed(
                        "session/new ne retourne pas sessionId".to_string(),
                    )
                })?
                .to_string();
            Ok(session_id)
        })();
        let session_id = match setup {
            Ok(session_id) => session_id,
            Err(error) => {
                writer.lock().recover_poison().take();
                let mut child = child.lock().recover_poison();
                let _ = child.kill();
                let _ = child.wait();
                drop(child);
                let _ = reader_handle.join();
                return Err(error);
            }
        };
        *active_session.lock().recover_poison() = Some(session_id.clone());

        let state = Arc::new(Mutex::new(TurnState::Idle));
        let private_profile_instructions = Arc::new(Mutex::new(None));
        let worker_handle = spawn_worker(TurnWorker {
            queue: queue.clone(),
            writer: writer.clone(),
            waiters,
            completions,
            next_id: next_id.clone(),
            alive: alive.clone(),
            state: state.clone(),
            events: events.clone(),
            response,
            reasoning_raw,
            session_id: session_id.clone(),
            notify_timeout: Duration::from_secs(options.notify_timeout_secs),
            cancel_grace,
            poll_interval,
            child: child.clone(),
            journal: journal.clone(),
            private_profile_instructions: private_profile_instructions.clone(),
            clock: clock.clone(),
            test_observer,
        });
        Ok(Self {
            connection_id: format!("acp-{pid}"),
            alive,
            provider_observation: options.provider_observation,
            shutdown_started,
            state,
            events,
            queue,
            queue_capacity: options.queue_capacity,
            writer,
            session_id,
            child,
            reader_handle: Mutex::new(Some(reader_handle)),
            worker_handle: Mutex::new(Some(worker_handle)),
            journal,
            private_profile_instructions,
            clock,
            context_published: AtomicBool::new(false),
        })
    }

    pub fn state(&self) -> TurnState {
        self.state
            .lock()
            .recover_poison()
            .clone()
    }

    pub fn enable_journal(&self, root: impl AsRef<Path>, agent: &str) -> std::io::Result<()> {
        self.enable_journal_with_live_feed(root, agent, None)
    }

    pub fn enable_journal_with_live_feed(
        &self,
        root: impl AsRef<Path>,
        agent: &str,
        live_feed: Option<JournalLiveFeed>,
    ) -> std::io::Result<()> {
        *self.journal.lock().recover_poison() =
            Some(JournalWriter::start_with_live_feed(
                root,
                agent,
                &self.session_id,
                self.events.clone(),
                live_feed,
            )?);
        Ok(())
    }

    pub fn drain_events(&self) -> Vec<AcpEvent> {
        self.events
            .lock()
            .recover_poison()
            .drain_acp()
    }

    /// Retire atomiquement un message en attente, ou signale au worker que le
    /// tour actif doit appliquer le protocole cancel+grâce.
    pub fn cancel_delivery(&self, message_id: &str, reason: &str) -> bool {
        let message = {
            let (queue, wakeup) = &*self.queue;
            let mut queue = queue.lock().recover_poison();
            if let Some(active) = queue
                .active
                .as_ref()
                .filter(|active| active.message_id == message_id)
            {
                active.cancelled.store(true, Ordering::SeqCst);
                let _ = active.cancellation.send(reason.to_string());
                return true;
            }
            let Some(index) = queue
                .messages
                .iter()
                .position(|message| message.id == message_id)
            else {
                return false;
            };
            let message = queue.messages.remove(index).expect("message ACP présent");
            wakeup.notify_one();
            message
        };
        self.events
            .lock()
            .recover_poison()
            .push_back(AcpEvent::DeliveryRejected {
                message_id: message.id,
                reason: reason.to_string(),
            });
        true
    }

    /// Arrêt propre : le client demande l'annulation du tour courant et ferme
    /// sa file, sans bloquer le thread qui détruit le transport.
    pub fn shutdown(&self) {
        if self.shutdown_started.swap(true, Ordering::SeqCst) {
            return;
        }
        self.alive.store(false, Ordering::SeqCst);
        let (queue, wakeup) = &*self.queue;
        let mut queue = queue.lock().recover_poison();
        queue.closed = true;
        drain_queue(&mut queue, &self.events, "équipier arrêté");
        wakeup.notify_all();
        drop(queue);
        let _ = write_json(
            &self.writer,
            json!({
                "jsonrpc": "2.0",
                "method": "session/cancel",
                "params": { "sessionId": self.session_id }
            }),
        );
        self.writer
            .lock()
            .recover_poison()
            .take();
        let mut child = self.child.lock().recover_poison();
        let _ = child.kill();
        let _ = child.wait();
        drop(child);
        if let Some(handle) = self
            .reader_handle
            .lock()
            .recover_poison()
            .take()
        {
            let _ = handle.join();
        }
        if let Some(handle) = self
            .worker_handle
            .lock()
            .recover_poison()
            .take()
        {
            let _ = handle.join();
        }
        if let Some(journal) = self
            .journal
            .lock()
            .recover_poison()
            .take()
        {
            journal.stop();
        }
    }
}

impl Drop for AcpTransport {
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl Transport for AcpTransport {
    fn deliver(&mut self, msg: &BridgetMessage) -> Result<(), TransportError> {
        if !self.is_alive() {
            return Err(TransportError::AgentDead);
        }
        if message_expired_at(msg, (self.clock)()) {
            self.events
                .lock()
                .recover_poison()
                .push_back(AcpEvent::DeliveryRejected {
                    message_id: msg.id.clone(),
                    reason: "échéance de livraison dépassée".to_string(),
                });
            return Ok(());
        }
        let steer_or_queue = msg.intent == Some(MessageIntent::SteerCurrent)
            && msg
                .references
                .iter()
                .any(|reference| reference == "steer-or-queue");
        if msg.intent == Some(MessageIntent::SteerCurrent) && !steer_or_queue {
            self.events
                .lock()
                .recover_poison()
                .push_back(AcpEvent::DeliveryRejected {
                    message_id: msg.id.clone(),
                    reason: "pilotage ACP indisponible".to_string(),
                });
            return Ok(());
        }
        let mut ordinary = msg.clone();
        if steer_or_queue {
            ordinary.intent = None;
        }
        let active_message_id = if !steer_or_queue
            && (msg.intent == Some(MessageIntent::InterruptAndStart)
                || msg.origin == Some(MessageOrigin::Human))
        {
            enqueue_and_capture_active_message_id(&self.queue, self.queue_capacity, ordinary)
        } else if enqueue(&self.queue, self.queue_capacity, ordinary) {
            Some(None)
        } else {
            None
        };
        let Some(active_message_id) = active_message_id else {
            self.events
                .lock()
                .recover_poison()
                .push_back(AcpEvent::DeliveryRejected {
                    message_id: msg.id.clone(),
                    reason: "file ACP pleine".to_string(),
                });
            return Ok(());
        };
        // InterruptAndStart reste FIFO, et vise seulement le tour capturé.
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

impl ManagedSession for AcpTransport {
    fn set_private_profile_instructions(
        &mut self,
        instructions: &str,
    ) -> Result<(), TransportError> {
        *self
            .private_profile_instructions
            .lock()
            .recover_poison() = Some(instructions.to_string());
        Ok(())
    }

    fn descriptor(&self) -> ManagedSessionDescriptor {
        ManagedSessionDescriptor {
            transport: "acp".to_string(),
            mode: PresenceMode::Acp,
            location: None,
        }
    }

    fn provider_identity(&self) -> Option<ManagedProviderIdentity> {
        Some(ManagedProviderIdentity {
            provider_kind: "acp".to_string(),
            provider_session_id: Some(self.session_id.clone()),
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
        AcpTransport::process_id(self)
    }

    fn activate_journal(
        &self,
        root: &Path,
        agent: &str,
        live_feed: Option<JournalLiveFeed>,
    ) -> std::io::Result<()> {
        AcpTransport::enable_journal_with_live_feed(self, root, agent, live_feed)
    }

    fn drain_events(&self) -> Vec<ManagedEvent> {
        let mut events = self
            .events
            .lock()
            .recover_poison()
            .drain_managed();
        if !self.context_published.swap(true, Ordering::SeqCst) {
            events.insert(
                0,
                ManagedEvent::internal(
                    ManagedEventSource::Acp,
                    Vec::new(),
                    ManagedEventKind::ProviderContextObserved {
                        identity: self
                            .provider_identity()
                            .expect("identité ACP disponible après session/new"),
                    },
                ),
            );
        }
        events
    }
    /// O(n), borné par queue_capacity. Le passage au tour actif et le retrait
    /// utilisent le même verrou ; aucune annulation active n'est envoyée ici.
    fn cancel_queued(&self, message_id: &str, reason: &str) -> bool {
        let message = {
            let (queue, wakeup) = &*self.queue;
            let mut queue = queue.lock().recover_poison();
            if queue
                .active
                .as_ref()
                .is_some_and(|active| active.message_id == message_id)
            {
                return false;
            }
            let Some(index) = queue
                .messages
                .iter()
                .position(|message| message.id == message_id)
            else {
                return false;
            };
            let message = queue.messages.remove(index).expect("message ACP présent");
            wakeup.notify_one();
            message
        };
        self.events
            .lock()
            .recover_poison()
            .push_back(AcpEvent::DeliveryRejected {
                message_id: message.id,
                reason: reason.to_string(),
            });
        true
    }

    fn cancel_delivery(&self, message_id: &str, reason: &str) -> bool {
        AcpTransport::cancel_delivery(self, message_id, reason)
    }

    fn stop(&self) {
        AcpTransport::shutdown(self);
    }

    fn is_busy(&self) -> bool {
        matches!(self.state(), TurnState::InProgress { .. })
    }
}

fn managed_kind(event: AcpEvent) -> ManagedEventKind {
    match event {
        AcpEvent::TurnStarted { message_id } => ManagedEventKind::TurnStarted { message_id },
        AcpEvent::PromptDispatched { message_id } => {
            ManagedEventKind::PromptDispatched { message_id }
        }
        AcpEvent::TurnFinished {
            message,
            response,
            stop_reason,
        } => ManagedEventKind::TurnFinished {
            message: *message,
            response,
            terminal: terminal_from_acp_stop_reason(&stop_reason),
        },
        AcpEvent::DeliveryRejected { message_id, reason } => {
            ManagedEventKind::DeliveryRejected { message_id, reason }
        }
        AcpEvent::Update { detail } => ManagedEventKind::Update { detail },
        AcpEvent::Error { detail } => ManagedEventKind::Error { detail },
        AcpEvent::JournalFailed { detail } => ManagedEventKind::JournalFailed { detail },
    }
}

fn terminal_from_acp_stop_reason(stop_reason: &str) -> ManagedTerminal {
    match stop_reason {
        "end_turn" | "completed" | "complete" => ManagedTerminal::Completed,
        "cancelled" | "canceled" => ManagedTerminal::Cancelled,
        other => ManagedTerminal::Failed {
            detail: format!("terminal ACP: {other}"),
        },
    }
}

/// ACP produit la représentation canonique de son propre événement avant la
/// frontière du wrapper. Ces bytes restent attachés à leur source plutôt que
/// réduits à un enum de métier du daemon.
fn acp_event_bytes(event: &AcpEvent) -> Vec<u8> {
    let value = match event {
        AcpEvent::TurnStarted { message_id } => json!({
            "kind": "turn_started",
            "message_id": message_id,
        }),
        AcpEvent::PromptDispatched { message_id } => json!({
            "kind": "prompt_dispatched",
            "message_id": message_id,
        }),
        AcpEvent::TurnFinished {
            message,
            response,
            stop_reason,
        } => json!({
            "kind": "turn_finished",
            "message": message,
            "response": response,
            "stop_reason": stop_reason,
        }),
        AcpEvent::DeliveryRejected { message_id, reason } => json!({
            "kind": "delivery_rejected",
            "message_id": message_id,
            "reason": reason,
        }),
        AcpEvent::Update { detail } => json!({ "kind": "update", "detail": detail }),
        AcpEvent::Error { detail } => json!({ "kind": "error", "detail": detail }),
        AcpEvent::JournalFailed { detail } => {
            json!({ "kind": "journal_failed", "detail": detail })
        }
    };
    serde_json::to_vec(&value).expect("canon interne ACP sérialisable")
}

#[cfg(test)]
fn message_expired(message: &BridgetMessage) -> bool {
    message_expired_at(message, SystemTime::now())
}

fn message_expired_at(message: &BridgetMessage, now: SystemTime) -> bool {
    let Some(deadline_at) = message.deadline_at else {
        return false;
    };
    now.duration_since(SystemTime::UNIX_EPOCH)
        .map(|now| now.as_secs() >= deadline_at)
        .unwrap_or(false)
}

/// Convertit l'échéance absolue portée par le daemon en durée restante pour le
/// prompt déjà actif. L'absence d'échéance conserve le comportement historique.
fn spawn_worker(worker: TurnWorker) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let (message, cancellation, cancelled) = {
                let (queue, wakeup) = &*worker.queue;
                let mut queue = queue.lock().recover_poison();
                while queue.messages.is_empty() && !queue.closed {
                    queue = wakeup.wait(queue).recover_poison();
                }
                if queue.closed {
                    break;
                }
                let message = queue.messages.pop_front().expect("file ACP non vide");
                let (cancellation, cancellation_receiver) = mpsc::channel();
                let cancelled = Arc::new(AtomicBool::new(false));
                queue.active = Some(ActiveTurn {
                    message_id: message.id.clone(),
                    cancellation,
                    cancelled: cancelled.clone(),
                });
                (message, cancellation_receiver, cancelled)
            };
            if !worker.alive.load(Ordering::SeqCst) {
                worker
                    .events
                    .lock()
                    .recover_poison()
                    .push_back(AcpEvent::DeliveryRejected {
                        message_id: message.id,
                        reason: "équipier arrêté".to_string(),
                    });
                clear_active_turn(&worker.queue);
                continue;
            }
            if message_expired_at(&message, (worker.clock)()) {
                worker
                    .events
                    .lock()
                    .recover_poison()
                    .push_back(AcpEvent::DeliveryRejected {
                        message_id: message.id.clone(),
                        reason: "échéance de livraison dépassée".to_string(),
                    });
                clear_active_turn(&worker.queue);
                continue;
            }
            *worker.state.lock().recover_poison() = TurnState::InProgress {
                message_id: message.id.clone(),
                since: SystemTime::now(),
            };
            worker
                .events
                .lock()
                .recover_poison()
                .push_back(AcpEvent::TurnStarted {
                    message_id: message.id.clone(),
                });
            worker
                .reasoning_raw
                .lock()
                .recover_poison()
                .clear();
            record_or_terminal(
                &worker.journal,
                &worker.events,
                "turn_start",
                Some(&message.id),
                json!({
                    "from": &message.from,
                    "reply": message.reply,
                    "body": &message.body,
                }),
            );
            worker
                .response
                .lock()
                .recover_poison()
                .clear();
            let instructions = worker
                .private_profile_instructions
                .lock()
                .recover_poison()
                .clone();
            let result = prompt_request(PromptRequestArgs {
writer: &worker.writer,
waiters: &worker.waiters,
completions: &worker.completions,
next_id: &worker.next_id,
method: "session/prompt",
params: json!({
                    "sessionId": &worker.session_id,
                    "prompt": [{ "type": "text", "text": prompt_for_with_private_instructions(&message, instructions.as_deref()) }]
                }),
timeout: // L'échéance absolue vient du daemon (relue à chaque livraison).
                // Sans elle, repli sur la valeur figée au spawn — qui ne suit
                // PAS un changement de agents.json tant que l'agent vit.
                message
                    .deadline_at
                    .is_none()
                    .then_some(worker.notify_timeout),
deadline_at: message.deadline_at,
clock: &worker.clock,
cancellation: &cancellation,
cancelled: &cancelled,
session_id: &worker.session_id,
child: &worker.child,
queue: &worker.queue,
events: &worker.events,
test_observer: &worker.test_observer,
prompt_message_id: &message.id,
alive: &worker.alive,
cancel_grace: worker.cancel_grace,
poll_interval: worker.poll_interval,
});
            let collected = worker
                .response
                .lock()
                .recover_poison()
                .clone();
            *worker.state.lock().recover_poison() = TurnState::Idle;
            clear_active_turn(&worker.queue);
            let event = finish_turn(message, collected, result);
            match &event {
                AcpEvent::TurnFinished {
                    message,
                    stop_reason,
                    ..
                } => {
                    // Ligne terminale C3 — y compris available:false (Gemini).
                    let reasoning = take_reasoning_journal_payload(&worker.reasoning_raw);
                    record_or_terminal(
                        &worker.journal,
                        &worker.events,
                        "reasoning",
                        Some(&message.id),
                        reasoning,
                    );
                    let mut payload = json!({ "stop_reason": stop_reason });
                    if message.reply {
                        payload["routed_to"] = json!(&message.from);
                    }
                    record_or_terminal(
                        &worker.journal,
                        &worker.events,
                        "turn_end",
                        Some(&message.id),
                        payload,
                    );
                }
                AcpEvent::DeliveryRejected { message_id, reason } => {
                    let reasoning = take_reasoning_journal_payload(&worker.reasoning_raw);
                    record_or_terminal(
                        &worker.journal,
                        &worker.events,
                        "reasoning",
                        Some(message_id),
                        reasoning,
                    );
                    record_or_terminal(
                        &worker.journal,
                        &worker.events,
                        "error",
                        Some(message_id),
                        with_turn_failed_kind(json!({ "reason": reason })),
                    );
                }
                _ => {}
            }
            if let Some(observer) = &worker.test_observer {
                let _ = observer.send(event.clone());
            }
            worker
                .events
                .lock()
                .recover_poison()
                .push_back(event);
        }
    })
}

fn clear_active_turn(queue: &Arc<(Mutex<QueueState>, Condvar)>) {
    queue.0.lock().recover_poison().active = None;
}

fn record_journal(
    journal: &Journal,
    event: &str,
    message_id: Option<&str>,
    payload: Value,
) -> Result<(), String> {
    let writer = journal
        .lock()
        .recover_poison()
        .clone();
    writer.map_or(Ok(()), |writer| writer.enqueue(event, message_id, payload))
}

fn record_or_terminal(
    journal: &Journal,
    events: &Arc<Mutex<AcpEventQueue>>,
    event: &str,
    message_id: Option<&str>,
    payload: Value,
) {
    if let Err(detail) = record_journal(journal, event, message_id, payload) {
        events
            .lock()
            .recover_poison()
            .push_back(AcpEvent::JournalFailed { detail });
    }
}

fn active_message_id(queue: &Arc<(Mutex<QueueState>, Condvar)>) -> Option<String> {
    queue
        .0
        .lock()
        .recover_poison()
        .active
        .as_ref()
        .map(|turn| turn.message_id.clone())
}

fn enqueue(
    queue: &Arc<(Mutex<QueueState>, Condvar)>,
    capacity: usize,
    message: BridgetMessage,
) -> bool {
    let (queue, wakeup) = &**queue;
    let mut queue = queue.lock().recover_poison();
    if queue.messages.len() >= capacity || queue.closed {
        return false;
    }
    queue.messages.push_back(message);
    wakeup.notify_one();
    true
}

fn enqueue_and_capture_active_message_id(
    queue: &Arc<(Mutex<QueueState>, Condvar)>,
    capacity: usize,
    message: BridgetMessage,
) -> Option<Option<String>> {
    let (queue, wakeup) = &**queue;
    let mut queue = queue.lock().recover_poison();
    if queue.messages.len() >= capacity || queue.closed {
        return None;
    }
    let active_message_id = queue.active.as_ref().map(|turn| turn.message_id.clone());
    queue.messages.push_back(message);
    wakeup.notify_one();
    Some(active_message_id)
}

fn finish_turn(
    message: BridgetMessage,
    response: String,
    result: Result<Value, TransportError>,
) -> AcpEvent {
    match result {
        Ok(value) => AcpEvent::TurnFinished {
            message: Box::new(message),
            response,
            stop_reason: value
                .get("stopReason")
                .and_then(Value::as_str)
                .unwrap_or("inconnu")
                .to_string(),
        },
        Err(error) => AcpEvent::DeliveryRejected {
            message_id: message.id,
            reason: error.to_string(),
        },
    }
}

// Les huit ressources sont les extrémités explicites du pipeline stdio ; les
// regrouper masquerait l'unique propriétaire de stdout imposé par D-204.
#[allow(clippy::too_many_arguments)]
fn spawn_reader(
    stdout: ChildStdout,
    writer: Writer,
    waiters: Waiters,
    completions: Completions,
    events: Arc<Mutex<AcpEventQueue>>,
    response: Arc<Mutex<String>>,
    reasoning_raw: Arc<Mutex<String>>,
    alive: Arc<AtomicBool>,
    permissions: String,
    queue: Arc<(Mutex<QueueState>, Condvar)>,
    active_session: Arc<Mutex<Option<String>>>,
    journal: Journal,
    test_observer: Option<mpsc::Sender<AcpEvent>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        // Le lecteur stdout est l'unique propriétaire de cette corrélation :
        // un `tool_call_update` ACP enrichit souvent titre/rawInput après coup
        // (mesuré Cursor : Read File → Read /path + rawInput ; MCP: tool →
        // permission qui porte le vrai nom). Chaque ligne du journal reste
        // autonome pour les consommateurs de replay.
        let mut tool_memory = HashMap::<String, ToolCallMemory>::new();
        let mut tool_memory_message = None;
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else {
                break;
            };
            let raw_line = line.as_bytes().to_vec();
            let Ok(value) = serde_json::from_str::<Value>(&line) else {
                let message_id = active_message_id(&queue);
                record_or_terminal(
                    &journal,
                    &events,
                    "error",
                    message_id.as_deref(),
                    json!({ "reason": "ligne ACP invalide" }),
                );
                events
                    .lock()
                    .recover_poison()
                    .push_source(
                        AcpEvent::Error {
                            detail: "ligne ACP invalide".to_string(),
                        },
                        raw_line,
                    );
                continue;
            };
            let current_message = active_message_id(&queue);
            if current_message != tool_memory_message {
                tool_memory.clear();
                tool_memory_message = current_message;
            }
            let rpc_result = rpc_response(&value);
            let waiter = rpc_result.as_ref().and_then(|(id, _)| {
                waiters
                    .lock()
                    .recover_poison()
                    .remove(id)
            });
            if let (Some((id, result)), Some(waiter)) = (rpc_result.as_ref(), waiter) {
                complete_request(&completions, id);
                let _ = waiter.send(result.clone());
                continue;
            }
            if let Some((id, _)) = rpc_result.as_ref()
                && complete_request(&completions, id)
            {
                continue;
            }
            match value.get("method").and_then(Value::as_str) {
                Some("session/update") => {
                    let session_id = active_session
                        .lock()
                        .recover_poison()
                        .clone();
                    if let (Some(observer), Some(text)) =
                        (&test_observer, update_text(&value, session_id.as_deref()))
                    {
                        let _ = observer.send(AcpEvent::Update {
                            detail: text.to_string(),
                        });
                    }
                    if update_has_foreign_session(&value, session_id.as_deref()) {
                        let message_id = active_message_id(&queue);
                        record_or_terminal(
                            &journal,
                            &events,
                            "error",
                            message_id.as_deref(),
                            json!({ "reason": "update ACP ignorée pour une session étrangère" }),
                        );
                        events
                            .lock()
                            .recover_poison()
                            .push_source(
                                AcpEvent::Error {
                                    detail: "update ACP ignorée pour une session étrangère"
                                        .to_string(),
                                },
                                raw_line.clone(),
                            );
                    } else if active_turn_is_cancelled(&queue) {
                        let message_id = active_message_id(&queue);
                        record_or_terminal(
                            &journal,
                            &events,
                            "error",
                            message_id.as_deref(),
                            json!({ "reason": "update ACP ignorée après annulation du tour" }),
                        );
                        events
                            .lock()
                            .recover_poison()
                            .push_source(
                                AcpEvent::Error {
                                    detail: "update ACP ignorée après annulation du tour"
                                        .to_string(),
                                },
                                raw_line.clone(),
                            );
                    } else if let Some(text) = update_text(&value, session_id.as_deref()) {
                        let message_id = active_message_id(&queue);
                        record_or_terminal(
                            &journal,
                            &events,
                            "update",
                            message_id.as_deref(),
                            json!({ "kind": "text", "content": text }),
                        );
                        response
                            .lock()
                            .recover_poison()
                            .push_str(text);
                        let event = AcpEvent::Update {
                            detail: text.to_string(),
                        };
                        events
                            .lock()
                            .recover_poison()
                            .push_source(event, raw_line.clone());
                    } else if let Some(thought) = update_thought_text(&value, session_id.as_deref())
                    {
                        // Variante standard SessionUpdate (effect-acp) : on
                        // accumule pour la ligne terminale event=reasoning.
                        // Pas de journal par chunk — alignement C3 / L3.
                        reasoning_raw
                            .lock()
                            .recover_poison()
                            .push_str(thought);
                        events
                            .lock()
                            .recover_poison()
                            .push_source(
                                AcpEvent::Update {
                                    detail: format!("reasoning:{thought}"),
                                },
                                raw_line.clone(),
                            );
                    } else if matches!(
                        value
                            .pointer("/params/update/sessionUpdate")
                            .and_then(Value::as_str),
                        Some("tool_call") | Some("tool_call_update")
                    ) {
                        let message_id = active_message_id(&queue);
                        let payload = tool_call_journal_payload(&value, &mut tool_memory);
                        if let Some(path) =
                            payload.get("write_completed_path").and_then(Value::as_str)
                            && let Ok(cwd) = std::env::current_dir()
                            && let Some(file) = crate::journal::confirmed_write_payload(path, &cwd)
                        {
                            record_or_terminal(
                                &journal,
                                &events,
                                "update",
                                message_id.as_deref(),
                                file,
                            );
                        }
                        record_or_terminal(
                            &journal,
                            &events,
                            "update",
                            message_id.as_deref(),
                            payload,
                        );
                    }
                }
                Some("session/request_permission") => {
                    if let Some((reply, payload)) =
                        permission_response(&value, &permissions, active_turn_is_cancelled(&queue))
                    {
                        let message_id = active_message_id(&queue);
                        // Mesure Cursor MCP : le vrai nom arrive ici
                        // (`bridget-bridget_ledger: bridget_ledger`) alors que
                        // tool_call ne porte que `MCP: tool` + rawInput vide.
                        if let Some(tool_payload) =
                            permission_tool_journal_payload(&value, &mut tool_memory)
                        {
                            record_or_terminal(
                                &journal,
                                &events,
                                "update",
                                message_id.as_deref(),
                                tool_payload,
                            );
                        }
                        record_or_terminal(
                            &journal,
                            &events,
                            "permission",
                            message_id.as_deref(),
                            payload,
                        );
                        let _ = write_json(&writer, reply);
                    }
                }
                Some(method) if value.get("id").is_some() => {
                    if let Some(reply) = method_not_found_response(&value, method) {
                        let _ = write_json(&writer, reply);
                    }
                    let message_id = active_message_id(&queue);
                    record_or_terminal(
                        &journal,
                        &events,
                        "error",
                        message_id.as_deref(),
                        json!({ "reason": format!("méthode ACP inconnue: {method}") }),
                    );
                    events
                        .lock()
                        .recover_poison()
                        .push_source(
                            AcpEvent::Error {
                                detail: format!("méthode ACP inconnue: {method}"),
                            },
                            raw_line.clone(),
                        );
                }
                Some(method) => {
                    let message_id = active_message_id(&queue);
                    record_or_terminal(
                        &journal,
                        &events,
                        "error",
                        message_id.as_deref(),
                        json!({ "reason": format!("notification ACP inconnue: {method}") }),
                    );
                    events
                        .lock()
                        .recover_poison()
                        .push_source(
                            AcpEvent::Error {
                                detail: format!("notification ACP inconnue: {method}"),
                            },
                            raw_line.clone(),
                        );
                }
                None => {
                    let message_id = active_message_id(&queue);
                    record_or_terminal(
                        &journal,
                        &events,
                        "error",
                        message_id.as_deref(),
                        json!({ "reason": "message ACP inattendu" }),
                    );
                    events
                        .lock()
                        .recover_poison()
                        .push_source(
                            AcpEvent::Error {
                                detail: "message ACP inattendu".to_string(),
                            },
                            raw_line,
                        );
                }
            }
        }
        alive.store(false, Ordering::SeqCst);
        let (queue_state, wakeup) = &*queue;
        let mut queue_state = queue_state.lock().recover_poison();
        queue_state.closed = true;
        drain_queue(&mut queue_state, &events, "EOF ACP");
        wakeup.notify_all();
        fail_waiters(&waiters, "EOF ACP");
        let message_id = queue_state
            .active
            .as_ref()
            .map(|turn| turn.message_id.clone());
        drop(queue_state);
        record_or_terminal(
            &journal,
            &events,
            "error",
            message_id.as_deref(),
            json!({ "reason": "EOF ACP" }),
        );
        events
            .lock()
            .recover_poison()
            .push_back(AcpEvent::Error {
                detail: "EOF ACP".to_string(),
            });
    })
}

fn drain_queue(queue: &mut QueueState, events: &Arc<Mutex<AcpEventQueue>>, reason: &str) {
    for message in queue.messages.drain(..) {
        events
            .lock()
            .recover_poison()
            .push_back(AcpEvent::DeliveryRejected {
                message_id: message.id,
                reason: reason.to_string(),
            });
    }
}

// Les ressources du tour sont distinctes par conception (lecteur unique,
// writer sérialisé, worker et arrêt forcé) ; les regrouper brouillerait D-204.
#[allow(clippy::too_many_arguments)]
struct PromptRequestArgs<'a> {
    pub writer: &'a Writer,
    pub waiters: &'a Waiters,
    pub completions: &'a Completions,
    pub next_id: &'a AtomicU64,
    pub method: &'a str,
    pub params: Value,
    pub timeout: Option<Duration>,
    pub deadline_at: Option<u64>,
    pub clock: &'a Clock,
    pub cancellation: &'a mpsc::Receiver<String>,
    pub cancelled: &'a AtomicBool,
    pub session_id: &'a str,
    pub child: &'a Arc<Mutex<Child>>,
    pub queue: &'a Arc<(Mutex<QueueState>, Condvar)>,
    pub events: &'a Arc<Mutex<AcpEventQueue>>,
    pub test_observer: &'a Option<mpsc::Sender<AcpEvent>>,
    pub prompt_message_id: &'a str,
    pub alive: &'a AtomicBool,
    pub cancel_grace: Duration,
    pub poll_interval: Duration,
}

fn prompt_request(args: PromptRequestArgs<'_>) -> Result<Value, TransportError> {
    let PromptRequestArgs { writer, waiters, completions, next_id, method, params, timeout, deadline_at, clock, cancellation, cancelled, session_id, child, queue, events, test_observer, prompt_message_id, alive, cancel_grace, poll_interval } = args;
    let id = next_id.fetch_add(1, Ordering::SeqCst);
    let (sender, receiver) = mpsc::channel();
    let (completion_sender, completion_receiver) = mpsc::channel();
    let key = id.to_string();
    waiters
        .lock()
        .recover_poison()
        .insert(key.clone(), sender);
    completions
        .lock()
        .recover_poison()
        .insert(key.clone(), completion_sender);
    if let Err(error) = write_json(
        writer,
        json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }),
    ) {
        waiters
            .lock()
            .recover_poison()
            .remove(&key);
        completions
            .lock()
            .recover_poison()
            .remove(&key);
        return Err(error);
    }
    // Cette frontière est volontairement immédiatement après write+flush :
    // un échec de pipe ou de flush ne produit jamais PromptDispatched.
    let dispatched = AcpEvent::PromptDispatched {
        message_id: prompt_message_id.to_string(),
    };
    if let Some(observer) = test_observer {
        let _ = observer.send(dispatched.clone());
    }
    events
        .lock()
        .recover_poison()
        .push_back(dispatched);
    let started = std::time::Instant::now();
    loop {
        let timed_out = deadline_at.is_some_and(|deadline| {
            (clock)()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|now| now.as_secs() >= deadline)
                .unwrap_or(false)
        }) || timeout.is_some_and(|timeout| started.elapsed() >= timeout);
        let cancellation_reason = cancellation
            .try_recv()
            .ok()
            .or_else(|| timed_out.then(|| format!("timeout ACP pour {method}")));
        if let Some(reason) = cancellation_reason {
            cancelled.store(true, Ordering::SeqCst);
            waiters
                .lock()
                .recover_poison()
                .remove(&key);
            let _ = write_json(
                writer,
                json!({ "jsonrpc": "2.0", "method": "session/cancel", "params": { "sessionId": session_id } }),
            );
            if completion_receiver.recv_timeout(cancel_grace).is_err() {
                force_stop_transport(
                    writer,
                    child,
                    queue,
                    events,
                    waiters,
                    completions,
                    alive,
                    &reason,
                );
            }
            return Err(TransportError::DeliveryFailed(reason));
        }
        let remaining = deadline_at
            .map(|deadline| {
                let now = (clock)()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                Duration::from_secs(deadline.saturating_sub(now))
            })
            .or(timeout)
            .map(|timeout| timeout.saturating_sub(started.elapsed()))
            .unwrap_or(CANCEL_POLL);
        let poll = remaining.min(poll_interval);
        match receiver.recv_timeout(poll) {
            Ok(result) => {
                completions
                    .lock()
                    .recover_poison()
                    .remove(&key);
                return result.map_err(TransportError::DeliveryFailed);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => return Err(TransportError::AgentDead),
        }
    }
}

fn complete_request(completions: &Completions, id: &str) -> bool {
    completions
        .lock()
        .recover_poison()
        .remove(id)
        .is_some_and(|completion| completion.send(()).is_ok())
}

fn active_turn_is_cancelled(queue: &Arc<(Mutex<QueueState>, Condvar)>) -> bool {
    queue
        .0
        .lock()
        .recover_poison()
        .active
        .as_ref()
        .is_some_and(|active| active.cancelled.load(Ordering::SeqCst))
}

// L'arrêt forcé touche explicitement chaque propriétaire de ressource afin de
// fermer stdin, récolter l'enfant et drainer la file sans fuite.
#[allow(clippy::too_many_arguments)]
fn force_stop_transport(
    writer: &Writer,
    child: &Arc<Mutex<Child>>,
    queue: &Arc<(Mutex<QueueState>, Condvar)>,
    events: &Arc<Mutex<AcpEventQueue>>,
    waiters: &Waiters,
    completions: &Completions,
    alive: &AtomicBool,
    reason: &str,
) {
    alive.store(false, Ordering::SeqCst);
    let (queue_state, wakeup) = &**queue;
    let mut queue_state = queue_state.lock().recover_poison();
    queue_state.closed = true;
    drain_queue(&mut queue_state, events, reason);
    wakeup.notify_all();
    drop(queue_state);
    writer.lock().recover_poison().take();
    let mut child = child.lock().recover_poison();
    let _ = child.kill();
    let _ = child.wait();
    fail_waiters(waiters, reason);
    completions
        .lock()
        .recover_poison()
        .clear();
}

fn request(
    writer: &Writer,
    waiters: &Waiters,
    next_id: &AtomicU64,
    method: &str,
    params: Value,
) -> Result<Value, TransportError> {
    let id = next_id.fetch_add(1, Ordering::SeqCst);
    let (sender, receiver) = mpsc::channel();
    let key = id.to_string();
    waiters
        .lock()
        .recover_poison()
        .insert(key.clone(), sender);
    if let Err(error) = write_json(
        writer,
        json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }),
    ) {
        waiters
            .lock()
            .recover_poison()
            .remove(&key);
        return Err(error);
    }
    let result = receiver
        .recv_timeout(REQUEST_TIMEOUT)
        .map_err(|_| TransportError::DeliveryFailed(format!("timeout ACP pour {method}")));
    if result.is_err() {
        waiters
            .lock()
            .recover_poison()
            .remove(&key);
    }
    result?.map_err(TransportError::DeliveryFailed)
}

fn rpc_id_key(value: &Value) -> Option<String> {
    match value {
        Value::Number(_) | Value::String(_) => serde_json::to_string(value).ok(),
        _ => None,
    }
}

fn ensure_protocol_version(initialize: &Value) -> Result<(), TransportError> {
    if initialize.get("protocolVersion").and_then(Value::as_i64) == Some(1) {
        Ok(())
    } else {
        Err(TransportError::DeliveryFailed(
            "version ACP incompatible".to_string(),
        ))
    }
}

fn rpc_response(value: &Value) -> Option<(String, Result<Value, String>)> {
    let id = value.get("id").and_then(rpc_id_key)?;
    if value.get("result").is_none() && value.get("error").is_none() {
        return None;
    }
    let result = value
        .get("error")
        .cloned()
        .map(|error| Err(error.to_string()))
        .unwrap_or_else(|| Ok(value.get("result").cloned().unwrap_or(Value::Null)));
    Some((id, result))
}

fn update_has_foreign_session(value: &Value, session_id: Option<&str>) -> bool {
    value
        .pointer("/params/sessionId")
        .and_then(Value::as_str)
        .zip(session_id)
        .is_some_and(|(received, expected)| received != expected)
}

/// Texte d'un `session/update` ACP dont `sessionUpdate` vaut `expected`
/// et dont le contenu est un bloc texte — forme documentée par le schéma
/// Agent Client Protocol (réf. MIT `effect-acp` / `SessionUpdate`).
fn update_chunk_text<'a>(
    value: &'a Value,
    session_id: Option<&str>,
    expected: &str,
) -> Option<&'a str> {
    if update_has_foreign_session(value, session_id) {
        return None;
    }
    if value
        .pointer("/params/update/sessionUpdate")
        .and_then(Value::as_str)
        != Some(expected)
        || value
            .pointer("/params/update/content/type")
            .and_then(Value::as_str)
            != Some("text")
    {
        return None;
    }
    value
        .pointer("/params/update/content/text")
        .and_then(Value::as_str)
}

fn update_text<'a>(value: &'a Value, session_id: Option<&str>) -> Option<&'a str> {
    update_chunk_text(value, session_id, "agent_message_chunk")
}

/// Variante standard `SessionUpdate` : pensée / raisonnement en flux.
/// Avant L4, `update_text` la rejetait avec tout ce qui n'était pas
/// `agent_message_chunk` — d'où zéro occurrence dans le code.
fn update_thought_text<'a>(value: &'a Value, session_id: Option<&str>) -> Option<&'a str> {
    update_chunk_text(value, session_id, "agent_thought_chunk")
}

/// Payload de la ligne journal terminale `event=reasoning` (contrat C3).
fn reasoning_journal_payload_from_raw(raw: &str) -> Value {
    if raw.is_empty() {
        // Absent ≠ vide : pas de summary/raw qui se liraient « pas réfléchi ».
        json!({ "available": false })
    } else {
        json!({
            "available": true,
            "summary": raw,
            "raw": raw,
        })
    }
}

fn take_reasoning_journal_payload(reasoning_raw: &Arc<Mutex<String>>) -> Value {
    let mut raw = reasoning_raw.lock().recover_poison();
    let payload = reasoning_journal_payload_from_raw(&raw);
    raw.clear();
    payload
}

/// Mémoire locale au lecteur stdout : titre et arguments se complètent au fil
/// des `tool_call` / `tool_call_update` / `request_permission` (mesuré Cursor).
#[derive(Debug, Default, Clone)]
struct ToolCallMemory {
    title: Option<String>,
    detail: String,
    write_path: Option<String>,
    edit_kind: bool,
    write_reported: bool,
}

fn remember_write_path(memory: &mut ToolCallMemory, kind: Option<&str>, input: Option<&Value>) {
    if let Some(kind) = kind {
        memory.edit_kind = kind == "edit";
    }
    let Some(input) = input else { return };
    let named = crate::journal::tool_write_path(memory.title.as_deref().unwrap_or(""), input);
    let typed = memory
        .edit_kind
        .then(|| {
            input
                .get("path")
                .or_else(|| input.get("file_path"))
                .and_then(Value::as_str)
        })
        .flatten();
    if let Some(path) = named.or(typed).filter(|p| !p.is_empty() && p.len() <= 4096) {
        memory.write_path = Some(path.into());
    }
}

/// Plafond du champ `detail` (rawInput sérialisé), miroir Claude.
const ACP_TOOL_INPUT_DETAIL_MAX: usize = 512;

fn compact_acp_raw_input(input: &Value) -> String {
    let raw = match input {
        Value::Null => return String::new(),
        Value::Object(map) if map.is_empty() => return String::new(),
        other => other.to_string(),
    };
    if raw.chars().count() <= ACP_TOOL_INPUT_DETAIL_MAX {
        return raw;
    }
    let truncated: String = raw.chars().take(ACP_TOOL_INPUT_DETAIL_MAX).collect();
    format!("{truncated}…")
}

fn remember_tool_title(memory: &mut ToolCallMemory, incoming: Option<&str>) {
    let Some(new) = incoming.map(str::trim).filter(|value| !value.is_empty()) else {
        return;
    };
    match memory.title.as_deref() {
        None => memory.title = Some(new.to_string()),
        // Ne jamais écraser un vrai nom par le libellé générique Cursor MCP.
        Some(_) if new == "MCP: tool" => {}
        Some("MCP: tool") | Some("inconnu") => memory.title = Some(new.to_string()),
        // Mesure Read : tool_call « Read File » puis update « Read /path ».
        Some(_) => memory.title = Some(new.to_string()),
    }
}

fn remember_tool_detail(memory: &mut ToolCallMemory, raw_input: Option<&Value>) {
    let Some(detail) = raw_input
        .map(compact_acp_raw_input)
        .filter(|detail| !detail.is_empty())
    else {
        return;
    };
    memory.detail = detail;
}

fn tool_call_journal_payload(
    value: &Value,
    tool_memory: &mut HashMap<String, ToolCallMemory>,
) -> Value {
    let update = value.pointer("/params/update").unwrap_or(&Value::Null);
    let content = update.get("content").unwrap_or(&Value::Null);
    let field = |name: &str| {
        update
            .get(name)
            .or_else(|| content.get(name))
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
    };
    let title = field("title");
    let name = field("name");
    let tool_kind = field("kind");
    let mut tool_call_id = field("toolCallId").filter(|id| id.len() <= 256);
    let raw_input = update.get("rawInput").or_else(|| content.get("rawInput"));
    let summary_text = update
        .get("text")
        .or_else(|| content.get("text"))
        .and_then(Value::as_str)
        .unwrap_or("");

    if tool_memory.len() >= 256 && tool_call_id.is_some_and(|id| !tool_memory.contains_key(id)) {
        // Conserver les tombstones : une saturation ne réarme pas un outil
        // déjà signalé. Les nouveaux outils restent visibles au journal.
        log::warn!("observation_gap: cache de corrélation ACP saturé");
        tool_call_id = None;
    }
    let memory = match tool_call_id {
        Some(tool_call_id) => tool_memory.entry(tool_call_id.to_string()).or_default(),
        None => {
            // Sans id : état jetable pour cette seule trame.
            let mut ephemeral = ToolCallMemory::default();
            remember_tool_title(&mut ephemeral, title.or(name).or(tool_kind));
            remember_tool_detail(&mut ephemeral, raw_input);
            let detail = if !ephemeral.detail.is_empty() {
                ephemeral.detail.clone()
            } else {
                summary_text.to_string()
            };
            return build_tool_journal_payload(
                ephemeral.title.as_deref().unwrap_or("inconnu"),
                &detail,
                None,
                title,
                name,
                tool_kind,
            );
        }
    };
    remember_tool_title(memory, title.or(name).or(tool_kind));
    remember_tool_detail(memory, raw_input);
    remember_write_path(memory, tool_kind, raw_input);
    // rawInput (arguments) est sticky ; le text/content de la trame reste
    // local — un tool_call_update peut dire « terminé » sans écraser les args.
    let detail = if !memory.detail.is_empty() {
        memory.detail.clone()
    } else {
        summary_text.to_string()
    };
    let tool = memory
        .title
        .clone()
        .unwrap_or_else(|| "inconnu".to_string());
    let mut payload =
        build_tool_journal_payload(&tool, &detail, tool_call_id, title, name, tool_kind);
    if field("status") == Some("failed") {
        memory.write_reported = true;
    }
    if field("status") == Some("completed")
        && !memory.write_reported
        && let Some(path) = memory.write_path.as_ref()
    {
        payload["write_completed_path"] = Value::String(path.clone());
        memory.write_reported = true;
    }
    payload
}

/// Enrichit le fil quand la permission porte le vrai nom (MCP Cursor).
fn permission_tool_journal_payload(
    value: &Value,
    tool_memory: &mut HashMap<String, ToolCallMemory>,
) -> Option<Value> {
    let tool_call = value.pointer("/params/toolCall")?;
    let tool_call_id = tool_call
        .get("toolCallId")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty() && value.len() <= 256)?;
    let title = tool_call
        .get("title")
        .or_else(|| tool_call.get("name"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())?;
    if title == "MCP: tool" {
        return None;
    }
    let raw_input = tool_call.get("rawInput");
    if tool_memory.len() >= 256 && !tool_memory.contains_key(tool_call_id) {
        log::warn!("observation_gap: cache de corrélation ACP saturé");
        return None;
    }
    let memory = tool_memory.entry(tool_call_id.to_string()).or_default();
    let before = memory.clone();
    remember_tool_title(memory, Some(title));
    remember_tool_detail(memory, raw_input);
    remember_write_path(
        memory,
        tool_call.get("kind").and_then(Value::as_str),
        raw_input,
    );
    // N'émettre que si on gagne un vrai nom ou des arguments.
    let gained_title = before.title.as_deref() != memory.title.as_deref()
        && memory
            .title
            .as_deref()
            .is_some_and(|title| title != "MCP: tool");
    let gained_detail = before.detail != memory.detail && !memory.detail.is_empty();
    if !gained_title && !gained_detail {
        return None;
    }
    let tool = memory.title.clone().unwrap_or_else(|| title.to_string());
    Some(build_tool_journal_payload(
        &tool,
        &memory.detail,
        Some(tool_call_id),
        Some(title),
        tool_call.get("name").and_then(Value::as_str),
        tool_call.get("kind").and_then(Value::as_str),
    ))
}

fn build_tool_journal_payload(
    tool: &str,
    detail: &str,
    tool_call_id: Option<&str>,
    title: Option<&str>,
    name: Option<&str>,
    tool_kind: Option<&str>,
) -> Value {
    // C3 : kind fermé `tool` (JournalUpdateKind::Tool). Champs legacy conservés pour
    // corrélation toolCallId / attach tant que ce dernier n'est pas porté.
    // Ne pas réécrire `tool_call` ici — c'est l'héritage borné (ToolCallLegacy).
    let mut payload = serde_json::Map::from_iter([
        ("kind".to_string(), Value::String("tool".to_string())),
        ("text".to_string(), Value::String(tool.to_string())),
        ("tool".to_string(), Value::String(tool.to_string())),
        ("summary".to_string(), Value::String(detail.to_string())),
    ]);
    if !detail.is_empty() {
        payload.insert("detail".to_string(), Value::String(detail.to_string()));
    }
    for (key, value) in [
        ("tool_call_id", tool_call_id),
        ("title", title.or(Some(tool))),
        ("name", name),
        ("tool_kind", tool_kind),
    ] {
        if let Some(value) = value.filter(|value| !value.is_empty()) {
            payload.insert(key.to_string(), Value::String(value.to_string()));
        }
    }
    Value::Object(payload)
}

fn fail_waiters(waiters: &Waiters, reason: &str) {
    let pending = std::mem::take(&mut *waiters.lock().recover_poison());
    for (_, waiter) in pending {
        let _ = waiter.send(Err(reason.to_string()));
    }
}

fn write_json(writer: &Writer, value: Value) -> Result<(), TransportError> {
    let mut writer = writer
        .lock()
        .map_err(|err| TransportError::Io(err.to_string()))?;
    let writer = writer.as_mut().ok_or(TransportError::AgentDead)?;
    writeln!(writer, "{value}").map_err(|err| TransportError::Io(err.to_string()))?;
    writer
        .flush()
        .map_err(|err| TransportError::Io(err.to_string()))
}

fn permission_response(
    value: &Value,
    permissions: &str,
    cancelled: bool,
) -> Option<(Value, Value)> {
    let id = value.get("id")?;
    let options = value.pointer("/params/options")?.as_array()?;
    let offered_options = options
        .iter()
        .filter_map(|option| {
            Some(json!({
                "optionId": option.get("optionId")?.as_str()?,
                "kind": option.get("kind")?.as_str()?,
            }))
        })
        .collect::<Vec<_>>();
    let tool = value
        .pointer("/params/toolCall/title")
        .or_else(|| value.pointer("/params/toolCall/name"))
        .and_then(Value::as_str)
        .unwrap_or("inconnu");
    if cancelled {
        let outcome = json!({ "outcome": "cancelled" });
        return Some((
            json!({ "jsonrpc": "2.0", "id": id, "result": { "outcome": outcome } }),
            json!({ "tool": tool, "options": offered_options, "decision": outcome }),
        ));
    }
    let accepted_kinds = if permissions == "allow" {
        ["allow_once", "allow_always"]
    } else {
        ["reject_once", "reject_always"]
    };
    let option = options.iter().find(|option| {
        option
            .get("kind")
            .and_then(Value::as_str)
            .is_some_and(|kind| accepted_kinds.contains(&kind))
            && option.get("optionId").and_then(Value::as_str).is_some()
    });
    match option
        .and_then(|option| option.get("optionId"))
        .and_then(Value::as_str)
    {
        Some(option_id) => {
            let outcome = json!({ "outcome": "selected", "optionId": option_id });
            Some((
                json!({ "jsonrpc": "2.0", "id": id, "result": { "outcome": outcome } }),
                json!({ "tool": tool, "options": offered_options, "decision": { "outcome": "selected", "option_id": option_id } }),
            ))
        }
        None => {
            let outcome = json!({ "outcome": "cancelled" });
            Some((
                json!({ "jsonrpc": "2.0", "id": id, "result": { "outcome": outcome } }),
                json!({ "tool": tool, "options": offered_options, "decision": { "outcome": "cancelled" } }),
            ))
        }
    }
}

fn method_not_found_response(value: &Value, method: &str) -> Option<Value> {
    let id = value.get("id")?;
    Some(json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": -32601, "message": format!("méthode ACP inconnue: {method}") }
    }))
}

pub fn prompt_for(message: &BridgetMessage) -> String {
    prompt_for_with_private_instructions(message, None)
}

fn prompt_for_with_private_instructions(
    message: &BridgetMessage,
    instructions: Option<&str>,
) -> String {
    let response = if message.reply {
        "Le wrapper relaie automatiquement ta réponse finale à l'expéditeur avec in_reply_to=id. Réponds avec le résultat utile ; ne double pas cette réponse par un envoi d'outil."
    } else {
        bridget_core::envelope::no_reply_notice(message)
    };
    let mut metadata = json!({"id": message.id, "from": message.from, "to": message.to,
        "reply": message.reply, "in_reply_to": message.in_reply_to});
    if let Some(origin) = &message.delegated_origin {
        metadata["delegated_origin"] = json!(origin);
        metadata["sender_label"] = json!(message.sender_label());
    }
    let prompt = format!(
        "[message Bridget de {} — réponse attendue : {}]\n[Métadonnées : {}]\n{response}\n\n{}",
        message.sender_label(),
        if message.reply { "oui" } else { "non" },
        metadata,
        message.body
    );
    let Some(instructions) = instructions
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return prompt;
    };
    format!("[Instructions individuelles Bridget]\n{instructions}\n\n{prompt}")
}

#[cfg(test)]
#[path = "acp_tests.rs"]
mod tests;
