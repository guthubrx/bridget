//! Transport ACP synchrone : un lecteur stdout, un writer sérialisé et un
//! worker FIFO. Le lecteur est l'unique propriétaire du flux de l'adaptateur.

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
            .unwrap_or_else(|error| error.into_inner())
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
                writer.lock().unwrap_or_else(|err| err.into_inner()).take();
                let mut child = child.lock().unwrap_or_else(|err| err.into_inner());
                let _ = child.kill();
                let _ = child.wait();
                drop(child);
                let _ = reader_handle.join();
                return Err(error);
            }
        };
        *active_session.lock().unwrap_or_else(|err| err.into_inner()) = Some(session_id.clone());

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
            .unwrap_or_else(|err| err.into_inner())
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
        *self.journal.lock().unwrap_or_else(|err| err.into_inner()) =
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
            .unwrap_or_else(|err| err.into_inner())
            .drain_acp()
    }

    /// Retire atomiquement un message en attente, ou signale au worker que le
    /// tour actif doit appliquer le protocole cancel+grâce.
    pub fn cancel_delivery(&self, message_id: &str, reason: &str) -> bool {
        let message = {
            let (queue, wakeup) = &*self.queue;
            let mut queue = queue.lock().unwrap_or_else(|err| err.into_inner());
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
            .unwrap_or_else(|err| err.into_inner())
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
        let mut queue = queue.lock().unwrap_or_else(|err| err.into_inner());
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
            .unwrap_or_else(|err| err.into_inner())
            .take();
        let mut child = self.child.lock().unwrap_or_else(|err| err.into_inner());
        let _ = child.kill();
        let _ = child.wait();
        drop(child);
        if let Some(handle) = self
            .reader_handle
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .take()
        {
            let _ = handle.join();
        }
        if let Some(handle) = self
            .worker_handle
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .take()
        {
            let _ = handle.join();
        }
        if let Some(journal) = self
            .journal
            .lock()
            .unwrap_or_else(|err| err.into_inner())
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
                .unwrap_or_else(|poison| poison.into_inner())
                .push_back(AcpEvent::DeliveryRejected {
                    message_id: msg.id.clone(),
                    reason: "échéance de livraison dépassée".to_string(),
                });
            return Ok(());
        }
        if msg.intent == Some(MessageIntent::SteerCurrent) {
            self.events
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .push_back(AcpEvent::DeliveryRejected {
                    message_id: msg.id.clone(),
                    reason: "pilotage ACP indisponible".to_string(),
                });
            return Ok(());
        }
        let active_message_id = if msg.intent == Some(MessageIntent::InterruptAndStart)
            || msg.origin == Some(MessageOrigin::Human)
        {
            enqueue_and_capture_active_message_id(&self.queue, self.queue_capacity, msg.clone())
        } else if enqueue(&self.queue, self.queue_capacity, msg.clone()) {
            Some(None)
        } else {
            None
        };
        let Some(active_message_id) = active_message_id else {
            self.events
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
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
            .unwrap_or_else(|poison| poison.into_inner()) = Some(instructions.to_string());
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
            .unwrap_or_else(|err| err.into_inner())
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
                let mut queue = queue.lock().unwrap_or_else(|err| err.into_inner());
                while queue.messages.is_empty() && !queue.closed {
                    queue = wakeup.wait(queue).unwrap_or_else(|err| err.into_inner());
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
                    .unwrap_or_else(|err| err.into_inner())
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
                    .unwrap_or_else(|err| err.into_inner())
                    .push_back(AcpEvent::DeliveryRejected {
                        message_id: message.id.clone(),
                        reason: "échéance de livraison dépassée".to_string(),
                    });
                clear_active_turn(&worker.queue);
                continue;
            }
            *worker.state.lock().unwrap_or_else(|err| err.into_inner()) = TurnState::InProgress {
                message_id: message.id.clone(),
                since: SystemTime::now(),
            };
            worker
                .events
                .lock()
                .unwrap_or_else(|err| err.into_inner())
                .push_back(AcpEvent::TurnStarted {
                    message_id: message.id.clone(),
                });
            worker
                .reasoning_raw
                .lock()
                .unwrap_or_else(|err| err.into_inner())
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
                .unwrap_or_else(|err| err.into_inner())
                .clear();
            let instructions = worker
                .private_profile_instructions
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .clone();
            let result = prompt_request(
                &worker.writer,
                &worker.waiters,
                &worker.completions,
                &worker.next_id,
                "session/prompt",
                json!({
                    "sessionId": &worker.session_id,
                    "prompt": [{ "type": "text", "text": prompt_for_with_private_instructions(&message, instructions.as_deref()) }]
                }),
                // L'échéance absolue vient du daemon (relue à chaque livraison).
                // Sans elle, repli sur la valeur figée au spawn — qui ne suit
                // PAS un changement de agents.json tant que l'agent vit.
                message
                    .deadline_at
                    .is_none()
                    .then_some(worker.notify_timeout),
                message.deadline_at,
                &worker.clock,
                &cancellation,
                &cancelled,
                &worker.session_id,
                &worker.child,
                &worker.queue,
                &worker.events,
                &worker.test_observer,
                &message.id,
                &worker.alive,
                worker.cancel_grace,
                worker.poll_interval,
            );
            let collected = worker
                .response
                .lock()
                .unwrap_or_else(|err| err.into_inner())
                .clone();
            *worker.state.lock().unwrap_or_else(|err| err.into_inner()) = TurnState::Idle;
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
                .unwrap_or_else(|err| err.into_inner())
                .push_back(event);
        }
    })
}

fn clear_active_turn(queue: &Arc<(Mutex<QueueState>, Condvar)>) {
    queue.0.lock().unwrap_or_else(|err| err.into_inner()).active = None;
}

fn record_journal(
    journal: &Journal,
    event: &str,
    message_id: Option<&str>,
    payload: Value,
) -> Result<(), String> {
    let writer = journal
        .lock()
        .unwrap_or_else(|err| err.into_inner())
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
            .unwrap_or_else(|err| err.into_inner())
            .push_back(AcpEvent::JournalFailed { detail });
    }
}

fn active_message_id(queue: &Arc<(Mutex<QueueState>, Condvar)>) -> Option<String> {
    queue
        .0
        .lock()
        .unwrap_or_else(|err| err.into_inner())
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
    let mut queue = queue.lock().unwrap_or_else(|err| err.into_inner());
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
    let mut queue = queue.lock().unwrap_or_else(|err| err.into_inner());
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
                    .unwrap_or_else(|err| err.into_inner())
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
                    .unwrap_or_else(|err| err.into_inner())
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
                        .unwrap_or_else(|err| err.into_inner())
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
                            .unwrap_or_else(|err| err.into_inner())
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
                            .unwrap_or_else(|err| err.into_inner())
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
                            .unwrap_or_else(|err| err.into_inner())
                            .push_str(text);
                        let event = AcpEvent::Update {
                            detail: text.to_string(),
                        };
                        events
                            .lock()
                            .unwrap_or_else(|err| err.into_inner())
                            .push_source(event, raw_line.clone());
                    } else if let Some(thought) = update_thought_text(&value, session_id.as_deref())
                    {
                        // Variante standard SessionUpdate (effect-acp) : on
                        // accumule pour la ligne terminale event=reasoning.
                        // Pas de journal par chunk — alignement C3 / L3.
                        reasoning_raw
                            .lock()
                            .unwrap_or_else(|err| err.into_inner())
                            .push_str(thought);
                        events
                            .lock()
                            .unwrap_or_else(|err| err.into_inner())
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
                        .unwrap_or_else(|err| err.into_inner())
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
                        .unwrap_or_else(|err| err.into_inner())
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
                        .unwrap_or_else(|err| err.into_inner())
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
        let mut queue_state = queue_state.lock().unwrap_or_else(|err| err.into_inner());
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
            .unwrap_or_else(|err| err.into_inner())
            .push_back(AcpEvent::Error {
                detail: "EOF ACP".to_string(),
            });
    })
}

fn drain_queue(queue: &mut QueueState, events: &Arc<Mutex<AcpEventQueue>>, reason: &str) {
    for message in queue.messages.drain(..) {
        events
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .push_back(AcpEvent::DeliveryRejected {
                message_id: message.id,
                reason: reason.to_string(),
            });
    }
}

// Les ressources du tour sont distinctes par conception (lecteur unique,
// writer sérialisé, worker et arrêt forcé) ; les regrouper brouillerait D-204.
#[allow(clippy::too_many_arguments)]
fn prompt_request(
    writer: &Writer,
    waiters: &Waiters,
    completions: &Completions,
    next_id: &AtomicU64,
    method: &str,
    params: Value,
    timeout: Option<Duration>,
    deadline_at: Option<u64>,
    clock: &Clock,
    cancellation: &mpsc::Receiver<String>,
    cancelled: &AtomicBool,
    session_id: &str,
    child: &Arc<Mutex<Child>>,
    queue: &Arc<(Mutex<QueueState>, Condvar)>,
    events: &Arc<Mutex<AcpEventQueue>>,
    test_observer: &Option<mpsc::Sender<AcpEvent>>,
    prompt_message_id: &str,
    alive: &AtomicBool,
    cancel_grace: Duration,
    poll_interval: Duration,
) -> Result<Value, TransportError> {
    let id = next_id.fetch_add(1, Ordering::SeqCst);
    let (sender, receiver) = mpsc::channel();
    let (completion_sender, completion_receiver) = mpsc::channel();
    let key = id.to_string();
    waiters
        .lock()
        .unwrap_or_else(|err| err.into_inner())
        .insert(key.clone(), sender);
    completions
        .lock()
        .unwrap_or_else(|err| err.into_inner())
        .insert(key.clone(), completion_sender);
    if let Err(error) = write_json(
        writer,
        json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }),
    ) {
        waiters
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .remove(&key);
        completions
            .lock()
            .unwrap_or_else(|err| err.into_inner())
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
        .unwrap_or_else(|err| err.into_inner())
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
                .unwrap_or_else(|err| err.into_inner())
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
                    .unwrap_or_else(|err| err.into_inner())
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
        .unwrap_or_else(|err| err.into_inner())
        .remove(id)
        .is_some_and(|completion| completion.send(()).is_ok())
}

fn active_turn_is_cancelled(queue: &Arc<(Mutex<QueueState>, Condvar)>) -> bool {
    queue
        .0
        .lock()
        .unwrap_or_else(|err| err.into_inner())
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
    let mut queue_state = queue_state.lock().unwrap_or_else(|err| err.into_inner());
    queue_state.closed = true;
    drain_queue(&mut queue_state, events, reason);
    wakeup.notify_all();
    drop(queue_state);
    writer.lock().unwrap_or_else(|err| err.into_inner()).take();
    let mut child = child.lock().unwrap_or_else(|err| err.into_inner());
    let _ = child.kill();
    let _ = child.wait();
    fail_waiters(waiters, reason);
    completions
        .lock()
        .unwrap_or_else(|err| err.into_inner())
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
        .unwrap_or_else(|err| err.into_inner())
        .insert(key.clone(), sender);
    if let Err(error) = write_json(
        writer,
        json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }),
    ) {
        waiters
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .remove(&key);
        return Err(error);
    }
    let result = receiver
        .recv_timeout(REQUEST_TIMEOUT)
        .map_err(|_| TransportError::DeliveryFailed(format!("timeout ACP pour {method}")));
    if result.is_err() {
        waiters
            .lock()
            .unwrap_or_else(|err| err.into_inner())
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
    let mut raw = reasoning_raw.lock().unwrap_or_else(|err| err.into_inner());
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
    let pending = std::mem::take(&mut *waiters.lock().unwrap_or_else(|err| err.into_inner()));
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
        "Aucune réponse inter-agent attendue. N'envoie pas d'accusé de réception, même pour annoncer ton silence. Ta réponse finale ne sera pas relayée."
    };
    let prompt = format!(
        "[message Bridget de {} — réponse attendue : {}]\n[Métadonnées : {}]\n{response}\n\n{}",
        message.from,
        if message.reply { "oui" } else { "non" },
        json!({"id": message.id, "from": message.from, "to": message.to,
            "reply": message.reply, "in_reply_to": message.in_reply_to}),
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
mod tests {
    #[test]
    fn spec100_acp_correlates_permission_path_and_separate_edit_kind() {
        let mut memory = std::collections::HashMap::new();
        for id in ["permission", "separate"] {
            let start = serde_json::json!({"params":{"update":{"toolCallId":id,"title":"Write File","kind":"edit","status":"in_progress"}}});
            super::tool_call_journal_payload(&start, &mut memory);
            if id == "permission" {
                let permission = serde_json::json!({"params":{"toolCall":{"toolCallId":id,"title":"Write","rawInput":{"path":"/p/x"}}}});
                super::permission_tool_journal_payload(&permission, &mut memory);
            } else {
                let args = serde_json::json!({"params":{"update":{"toolCallId":id,"rawInput":{"path":"/p/x"}}}});
                super::tool_call_journal_payload(&args, &mut memory);
            }
            let end =
                serde_json::json!({"params":{"update":{"toolCallId":id,"status":"completed"}}});
            assert_eq!(
                super::tool_call_journal_payload(&end, &mut memory)["write_completed_path"],
                "/p/x"
            );
            assert!(
                super::tool_call_journal_payload(&end, &mut memory)
                    .get("write_completed_path")
                    .is_none()
            );
        }
    }

    #[test]
    fn spec100_acp_correlates_completed_write_not_read_or_failure() {
        let mut memory = std::collections::HashMap::new();
        let start = serde_json::json!({"params":{"update":{"sessionUpdate":"tool_call","toolCallId":"w","title":"Write","kind":"edit","rawInput":{"content":"x".repeat(2000),"path":"/project/x"},"status":"in_progress"}}});
        assert!(
            super::tool_call_journal_payload(&start, &mut memory)
                .get("write_completed_path")
                .is_none()
        );
        let end = serde_json::json!({"params":{"update":{"sessionUpdate":"tool_call_update","toolCallId":"w","status":"completed"}}});
        assert_eq!(
            super::tool_call_journal_payload(&end, &mut memory)["write_completed_path"],
            "/project/x"
        );
        assert!(
            super::tool_call_journal_payload(&end, &mut memory)
                .get("write_completed_path")
                .is_none()
        );
        let read = serde_json::json!({"params":{"update":{"toolCallId":"r","title":"Read","kind":"read","rawInput":{"path":"/project/x"},"status":"completed"}}});
        assert!(
            super::tool_call_journal_payload(&read, &mut memory)
                .get("write_completed_path")
                .is_none()
        );
        let mut failed = start.clone();
        failed["params"]["update"]["toolCallId"] = serde_json::json!("failure");
        failed["params"]["update"]["status"] = serde_json::json!("failed");
        assert!(
            super::tool_call_journal_payload(&failed, &mut memory)
                .get("write_completed_path")
                .is_none()
        );
        failed["params"]["update"]["status"] = serde_json::json!("completed");
        assert!(
            super::tool_call_journal_payload(&failed, &mut memory)
                .get("write_completed_path")
                .is_none()
        );
    }

    #[test]
    fn spec100_acp_saturation_does_not_rearm_completed_writes() {
        let mut memory = std::collections::HashMap::new();
        let write = serde_json::json!({"params":{"update":{"toolCallId":"original","title":"Write","rawInput":{"path":"/p/x"},"status":"completed"}}});
        assert!(
            super::tool_call_journal_payload(&write, &mut memory)
                .get("write_completed_path")
                .is_some()
        );
        for index in 0..300 {
            let other = serde_json::json!({"params":{"update":{"toolCallId":format!("other-{index}"),"title":"Read","rawInput":{"path":"/p/x"},"status":"completed"}}});
            super::tool_call_journal_payload(&other, &mut memory);
        }
        assert_eq!(memory.len(), 256);
        assert!(
            super::tool_call_journal_payload(&write, &mut memory)
                .get("write_completed_path")
                .is_none()
        );
        let permission = serde_json::json!({"params":{"toolCall":{"toolCallId":"new","title":"Write","rawInput":{"path":"/p/x"}}}});
        assert!(super::permission_tool_journal_payload(&permission, &mut memory).is_none());
        assert_eq!(memory.len(), 256);
    }
    use super::*;
    use std::time::Instant;

    fn queue() -> Arc<(Mutex<QueueState>, Condvar)> {
        Arc::new((
            Mutex::new(QueueState {
                messages: VecDeque::new(),
                active: None,
                closed: false,
            }),
            Condvar::new(),
        ))
    }

    fn message(id: &str) -> BridgetMessage {
        let mut message = BridgetMessage::new("alice", "bob", id);
        message.id = id.to_string();
        message
    }

    #[test]
    fn evenement_interne_est_explicitement_distingue_de_la_ligne_source() {
        let event = ManagedEvent::internal(
            ManagedEventSource::Acp,
            acp_event_bytes(&AcpEvent::Update {
                detail: "octets source".to_string(),
            }),
            ManagedEventKind::Update {
                detail: "octets source".to_string(),
            },
        );
        assert_eq!(event.source, ManagedEventSource::Acp);
        assert_eq!(
            event.origin,
            crate::managed_session::ManagedEventOrigin::Internal
        );
        assert_eq!(
            event.raw,
            br#"{"detail":"octets source","kind":"update"}"#.to_vec()
        );
        assert!(matches!(
            event.kind,
            ManagedEventKind::Update { detail } if detail == "octets source"
        ));
    }

    #[test]
    fn terminal_acp_est_normalise_avant_la_frontiere_commune() {
        assert_eq!(
            terminal_from_acp_stop_reason("end_turn"),
            ManagedTerminal::Completed
        );
        assert_eq!(
            terminal_from_acp_stop_reason("cancelled"),
            ManagedTerminal::Cancelled
        );
        assert_eq!(
            terminal_from_acp_stop_reason("provider-overloaded"),
            ManagedTerminal::Failed {
                detail: "terminal ACP: provider-overloaded".to_string(),
            }
        );
    }

    #[test]
    fn prompt_preserves_the_body_byte_for_byte() {
        let mut message = BridgetMessage::new("alice", "bob", "'\"$x\nligne");
        message.id = "mcp-38210-6a8a7fc7-1".to_string();
        let prompt = prompt_for(&message);
        assert!(prompt.starts_with("[message Bridget de alice — réponse attendue : non]"));
        assert!(prompt.ends_with(&message.body));
    }

    #[test]
    fn spec105_acp_contrat_explicite_et_corps_intact() {
        for requested in [false, true] {
            let mut msg = BridgetMessage::new("alice", "bob", "Texte ' et \" et\nseconde ligne");
            msg.reply = requested;
            msg.in_reply_to = Some("parent".into());
            let prompt = prompt_for(&msg);
            assert!(prompt.contains(&msg.id));
            assert!(prompt.contains("parent"));
            assert!(prompt.ends_with(&msg.body));
            assert_eq!(
                prompt.contains("Aucune réponse inter-agent attendue"),
                !requested
            );
            assert_eq!(prompt.contains("relaie automatiquement"), requested);
        }
    }

    #[test]
    fn generic_fixture_covers_json_rpc_matrix() {
        let lines = include_str!("../tests/fixtures/acp/generic.jsonl")
            .lines()
            .collect::<Vec<_>>();
        let notification: Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(
            update_text(&notification, Some("fixture-session")),
            Some("premier")
        );

        let tool_update: Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(update_text(&tool_update, Some("fixture-session")), None);

        let permission: Value = serde_json::from_str(lines[2]).unwrap();
        assert_eq!(
            permission_response(&permission, "allow", false).unwrap().0["result"],
            json!({"outcome":{"outcome":"selected","optionId":"allow-1"}})
        );
        assert_eq!(
            permission_response(&permission, "allow", false).unwrap().1,
            json!({
                "tool":"inconnu",
                "options":[
                    {"optionId":"allow-1","kind":"allow_once"},
                    {"optionId":"reject-1","kind":"reject_once"}
                ],
                "decision":{"outcome":"selected","option_id":"allow-1"}
            })
        );
        assert_eq!(
            permission_response(&permission, "deny", false).unwrap().0["result"]["outcome"]["optionId"],
            "reject-1"
        );
        let invalid_permission = json!({
            "id": "permission-invalid",
            "params": { "options": [
                {"optionId": "bogus", "kind": "allow_bogus"},
                {"optionId": 7, "kind": "allow_once"}
            ]}
        });
        assert_eq!(
            permission_response(&invalid_permission, "allow", false)
                .unwrap()
                .0["result"],
            json!({"outcome":{"outcome":"cancelled"}})
        );
        assert_eq!(
            permission_response(&invalid_permission, "allow", false)
                .unwrap()
                .1["decision"],
            json!({"outcome":"cancelled"})
        );

        let numeric: Value = serde_json::from_str(lines[3]).unwrap();
        let string: Value = serde_json::from_str(lines[4]).unwrap();
        assert_eq!(rpc_response(&numeric).unwrap().0, "9");
        assert_eq!(rpc_response(&string).unwrap().0, "\"string-9\"");

        let error: Value = serde_json::from_str(lines[5]).unwrap();
        assert!(
            rpc_response(&error)
                .unwrap()
                .1
                .unwrap_err()
                .contains("refus")
        );

        let unknown: Value = serde_json::from_str(lines[6]).unwrap();
        assert_eq!(
            method_not_found_response(&unknown, "server/inconnu").unwrap()["error"]["code"],
            -32601
        );
        assert!(serde_json::from_str::<Value>(lines[7]).is_err());
    }

    #[test]
    fn tool_call_journal_prefers_title_then_name_then_kind() {
        let update = |fields: Value| json!({"params":{"update":fields}});
        let mut tool_memory = HashMap::new();
        let titled = tool_call_journal_payload(
            &update(json!({
                "toolCallId":"tool-1", "title":"Read src/main.rs", "name":"read_file", "kind":"read",
                "content":{"text":"lecture"}
            })),
            &mut tool_memory,
        );
        assert_eq!(titled["tool"], "Read src/main.rs");
        assert_eq!(titled["title"], "Read src/main.rs");
        assert_eq!(titled["tool_call_id"], "tool-1");
        assert_eq!(titled["name"], "read_file");
        assert_eq!(titled["tool_kind"], "read");
        assert_eq!(titled["summary"], "lecture");

        let titled_update = tool_call_journal_payload(
            &update(json!({"toolCallId":"tool-1", "content":{"text":"terminé"}})),
            &mut tool_memory,
        );
        assert_eq!(titled_update["tool"], "Read src/main.rs");
        assert_eq!(titled_update["title"], "Read src/main.rs");
        assert_eq!(titled_update["tool_call_id"], "tool-1");
        assert_eq!(titled_update["summary"], "terminé");

        let named = tool_call_journal_payload(
            &update(json!({"name":"Bash cargo test", "kind":"execute"})),
            &mut tool_memory,
        );
        assert_eq!(named["tool"], "Bash cargo test");
        assert!(named.get("title").is_some());

        let unknown_kind =
            tool_call_journal_payload(&update(json!({"kind":"quantum_wrench"})), &mut tool_memory);
        assert_eq!(unknown_kind["tool"], "quantum_wrench");
        assert_eq!(unknown_kind["tool_kind"], "quantum_wrench");
    }

    /// Mesure Cursor 2026-08-27 : tool_call initial « Read File » + rawInput {},
    /// puis tool_call_update titre enrichi + rawInput.path. Mutant qui ignore
    /// rawInput ou fige le premier titre → meurt.
    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_TOOL_acp_retranscrit_rawInput_et_titre_enrichi() {
        let update = |fields: Value| {
            json!({
                "jsonrpc": "2.0",
                "method": "session/update",
                "params": {
                    "sessionId": "fixture-session",
                    "update": fields
                }
            })
        };
        let mut tool_memory = HashMap::new();
        let first = tool_call_journal_payload(
            &update(json!({
                "sessionUpdate": "tool_call",
                "toolCallId": "call-mesure-read",
                "title": "Read File",
                "kind": "read",
                "status": "pending",
                "rawInput": {}
            })),
            &mut tool_memory,
        );
        assert_eq!(first["tool"], "Read File");
        assert!(first.get("detail").is_none() || first["detail"] == "");

        let enriched = tool_call_journal_payload(
            &update(json!({
                "sessionUpdate": "tool_call_update",
                "toolCallId": "call-mesure-read",
                "title": "Read /tmp/bt/mesure-sentinel.txt",
                "rawInput": {"path": "/tmp/bt/mesure-sentinel.txt"},
                "locations": [{"path": "/tmp/bt/mesure-sentinel.txt"}]
            })),
            &mut tool_memory,
        );
        assert_eq!(
            enriched["text"].as_str(),
            Some("Read /tmp/bt/mesure-sentinel.txt"),
            "titre enrichi en dur, payload={}",
            enriched
        );
        assert_eq!(
            enriched["tool"].as_str(),
            Some("Read /tmp/bt/mesure-sentinel.txt"),
            "champ tool en dur, payload={}",
            enriched
        );
        assert_eq!(
            enriched["detail"].as_str(),
            Some(r#"{"path":"/tmp/bt/mesure-sentinel.txt"}"#),
            "rawInput en dur (pas une présence vide), payload={}",
            enriched
        );
    }

    /// Mesure Cursor MCP : tool_call titre « MCP: tool », puis permission
    /// `bridget-bridget_ledger: bridget_ledger`. Mutant qui laisse MCP: tool → meurt.
    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_TOOL_acp_permission_remplace_MCP_tool_par_vrai_nom() {
        let mut tool_memory = HashMap::new();
        let generic = tool_call_journal_payload(
            &json!({
                "params": {"update": {
                    "sessionUpdate": "tool_call",
                    "toolCallId": "call-mesure-mcp",
                    "title": "MCP: tool",
                    "kind": "other",
                    "rawInput": {}
                }}
            }),
            &mut tool_memory,
        );
        assert_eq!(generic["tool"], "MCP: tool");

        let from_permission = permission_tool_journal_payload(
            &json!({
                "params": {
                    "toolCall": {
                        "toolCallId": "call-mesure-mcp",
                        "title": "bridget-bridget_ledger: bridget_ledger",
                        "kind": "other",
                        "status": "pending"
                    }
                }
            }),
            &mut tool_memory,
        )
        .expect("la permission doit produire un acte tool nommé");
        assert_eq!(
            from_permission["text"].as_str(),
            Some("bridget-bridget_ledger: bridget_ledger"),
            "nom MCP en dur, payload={}",
            from_permission
        );
        assert_eq!(
            from_permission["tool"].as_str(),
            Some("bridget-bridget_ledger: bridget_ledger"),
            "champ tool en dur, payload={}",
            from_permission
        );

        let later = tool_call_journal_payload(
            &json!({
                "params": {"update": {
                    "sessionUpdate": "tool_call_update",
                    "toolCallId": "call-mesure-mcp",
                    "status": "completed"
                }}
            }),
            &mut tool_memory,
        );
        assert_eq!(
            later["tool"].as_str(),
            Some("bridget-bridget_ledger: bridget_ledger"),
            "le cache ne doit pas retomber sur MCP: tool, payload={}",
            later
        );
    }

    fn session_update(session_update: &str, text: &str) -> Value {
        json!({
            "jsonrpc": "2.0",
            "method": "session/update",
            "params": {
                "sessionId": "fixture-session",
                "update": {
                    "sessionUpdate": session_update,
                    "content": { "type": "text", "text": text }
                }
            }
        })
    }

    fn session_tool_call(title: &str, summary: &str) -> Value {
        json!({
            "jsonrpc": "2.0",
            "method": "session/update",
            "params": {
                "sessionId": "fixture-session",
                "update": {
                    "sessionUpdate": "tool_call",
                    "toolCallId": "tool-1",
                    "title": title,
                    "kind": "read",
                    "content": { "type": "text", "text": summary }
                }
            }
        })
    }

    /// Simule le chemin productif : chunks → reasoning_raw → take terminal.
    fn accumulate_thought_then_take(chunks: &[&str]) -> Value {
        let reasoning_raw = Arc::new(Mutex::new(String::new()));
        for chunk in chunks {
            let update = session_update("agent_thought_chunk", chunk);
            let thought = update_thought_text(&update, Some("fixture-session"))
                .expect("thought_chunk productif");
            reasoning_raw
                .lock()
                .unwrap_or_else(|err| err.into_inner())
                .push_str(thought);
        }
        take_reasoning_journal_payload(&reasoning_raw)
    }

    /// T4.4 — présence de pensée → ligne terminale available true (pipeline réel).
    #[test]
    fn agent_thought_chunk_rend_reasoning_available_true() {
        // Ordre documenté : PENSÉE puis MESSAGES puis OUTILS — seul le raw compte.
        let terminal = accumulate_thought_then_take(&["je vais lire ", "le fichier"]);
        assert_eq!(terminal["available"], true);
        assert_eq!(terminal["raw"], "je vais lire le fichier");
        assert_eq!(terminal["summary"], "je vais lire le fichier");

        let mut tool_memory = HashMap::new();
        let act = tool_call_journal_payload(
            &session_tool_call("Read src/main.rs", "lecture"),
            &mut tool_memory,
        );
        assert_eq!(act["kind"], "tool");
        assert_eq!(act["text"], "Read src/main.rs");
        assert_eq!(act["detail"], "lecture");
        // Un second take sur le même accumulateur vide → available:false (unicité).
        let second = accumulate_thought_then_take(&[]);
        assert_eq!(second, json!({ "available": false }));
    }

    /// T4.5 — aucun chunk de pensée → ligne terminale available false
    /// (cas Gemini : chunk jamais émis). Pas de summary/raw vides.
    #[test]
    fn flux_sans_thought_chunk_rend_reasoning_available_false() {
        let terminal = accumulate_thought_then_take(&[]);
        assert_eq!(terminal, json!({ "available": false }));
        assert!(terminal.get("summary").is_none());
        assert!(terminal.get("raw").is_none());

        let mut tool_memory = HashMap::new();
        let act = tool_call_journal_payload(
            &session_tool_call("Bash cargo test", "ok"),
            &mut tool_memory,
        );
        assert_eq!(act["kind"], "tool");
        assert_eq!(act["text"], "Bash cargo test");
    }

    /// Unicité du terminal : take vide le buffer — un second take ne rejoue pas.
    #[test]
    fn reasoning_terminal_est_unique_par_prise() {
        let reasoning_raw = Arc::new(Mutex::new(String::from("une seule fois")));
        let first = take_reasoning_journal_payload(&reasoning_raw);
        let second = take_reasoning_journal_payload(&reasoning_raw);
        assert_eq!(first["available"], true);
        assert_eq!(first["raw"], "une seule fois");
        assert_eq!(second, json!({ "available": false }));
        assert!(
            reasoning_raw
                .lock()
                .unwrap_or_else(|err| err.into_inner())
                .is_empty()
        );
    }

    /// T4.6 — mutant : si l'on rétablissait le filtre qui n'accepte que
    /// `agent_message_chunk`, ce test meurt. Nom du test mort sous mutant :
    /// `agent_thought_chunk_rend_reasoning_available_true`.
    #[test]
    fn thought_chunk_n_est_plus_rejete_par_le_filtre_de_session_update() {
        let thought = session_update("agent_thought_chunk", "pensée");
        assert!(
            update_text(&thought, Some("fixture-session")).is_none(),
            "la pensée ne doit pas alimenter le message"
        );
        assert_eq!(
            update_thought_text(&thought, Some("fixture-session")),
            Some("pensée")
        );
        assert_eq!(
            update_chunk_text(&thought, Some("fixture-session"), "agent_thought_chunk"),
            Some("pensée")
        );
    }

    #[test]
    fn codex_spike_fixture_collects_text_then_stop_reason() {
        let mut text = String::new();
        let mut stop_reason = None;
        for line in include_str!("../tests/fixtures/acp/codex-spike.jsonl").lines() {
            let value: Value = serde_json::from_str(line).unwrap();
            if let Some(chunk) = update_text(&value, Some("spike-session")) {
                text.push_str(chunk);
            }
            if let Some((_, Ok(result))) = rpc_response(&value) {
                stop_reason = result
                    .get("stopReason")
                    .and_then(Value::as_str)
                    .map(str::to_string);
            }
        }
        assert_eq!(text, "SPIKE_ACP_OK");
        assert_eq!(stop_reason.as_deref(), Some("end_turn"));
    }

    #[test]
    fn claude_spike_fixture_collects_text_then_stop_reason() {
        let mut text = String::new();
        let mut stop_reason = None;
        for line in include_str!("../tests/fixtures/acp/claude-spike.jsonl").lines() {
            let value: Value = serde_json::from_str(line).unwrap();
            if let Some(chunk) = update_text(&value, Some("claude-spike-session")) {
                text.push_str(chunk);
            }
            if let Some((_, Ok(result))) = rpc_response(&value) {
                stop_reason = result
                    .get("stopReason")
                    .and_then(Value::as_str)
                    .map(str::to_string);
            }
        }
        assert_eq!(text, "CLAUDE_ACP_OK");
        assert_eq!(stop_reason.as_deref(), Some("end_turn"));
    }

    #[test]
    fn responses_are_correlated_by_exact_id_even_out_of_order() {
        let waiters: Waiters = Arc::new(Mutex::new(HashMap::new()));
        let (numeric_sender, numeric_receiver) = mpsc::channel();
        let (string_sender, string_receiver) = mpsc::channel();
        waiters
            .lock()
            .unwrap()
            .insert("2".to_string(), numeric_sender);
        waiters
            .lock()
            .unwrap()
            .insert("\"two\"".to_string(), string_sender);
        for value in [
            json!({"id":"two","result":"second"}),
            json!({"id":2,"result":"first"}),
        ] {
            let (id, result) = rpc_response(&value).unwrap();
            waiters
                .lock()
                .unwrap()
                .remove(&id)
                .unwrap()
                .send(result)
                .unwrap();
        }
        assert_eq!(numeric_receiver.recv().unwrap().unwrap(), "first");
        assert_eq!(string_receiver.recv().unwrap().unwrap(), "second");
    }

    #[test]
    fn eof_fails_every_pending_waiter() {
        let waiters: Waiters = Arc::new(Mutex::new(HashMap::new()));
        let (sender, receiver) = mpsc::channel();
        waiters.lock().unwrap().insert("1".to_string(), sender);
        fail_waiters(&waiters, "EOF ACP");
        assert_eq!(receiver.recv().unwrap().unwrap_err(), "EOF ACP");
        assert!(waiters.lock().unwrap().is_empty());
    }

    #[test]
    fn incompatible_protocol_version_is_explicit() {
        assert!(ensure_protocol_version(&json!({"protocolVersion": 1})).is_ok());
        assert_eq!(
            ensure_protocol_version(&json!({"protocolVersion": 2}))
                .unwrap_err()
                .to_string(),
            "livraison échouée: version ACP incompatible"
        );
    }

    #[test]
    fn session_acp_recoit_le_serveur_mcp_ephemere() {
        let script = r#"
read initialize
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read session
case "$session" in
  *'"name":"bridget"'*) ;;
  *) exit 31 ;;
esac
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
while read line; do :; done
"#;
        let transport = AcpTransport::spawn_with_environment_and_mcp(
            AcpOptions {
                command: "sh".to_string(),
                args: vec!["-c".to_string(), script.to_string()],
                queue_capacity: 1,
                permissions: "allow".to_string(),
                notify_timeout_secs: 1,
                provider_observation: None,
            },
            &[],
            vec![json!({ "name": "bridget", "command": "bridget", "args": ["mcp"], "env": [] })],
        )
        .unwrap();
        transport.shutdown();
    }

    #[test]
    fn queue_is_fifo_and_capacity_rejection_is_terminal() {
        let queue = queue();
        assert!(enqueue(&queue, 2, message("first")));
        assert!(enqueue(&queue, 2, message("second")));
        assert!(!enqueue(&queue, 2, message("third")));
        let (state, _) = &*queue;
        let state = state.lock().unwrap();
        assert_eq!(
            state
                .messages
                .iter()
                .map(|message| message.id.as_str())
                .collect::<Vec<_>>(),
            vec!["first", "second"]
        );
    }

    #[test]
    fn saturated_journal_becomes_a_terminal_event() {
        let journal = Arc::new(Mutex::new(Some(
            crate::journal::JournalWriter::saturated_for_test(),
        )));
        let events = Arc::new(Mutex::new(AcpEventQueue::default()));
        record_or_terminal(
            &journal,
            &events,
            "update",
            Some("m1"),
            json!({"kind":"text","content":"x"}),
        );
        assert!(
            matches!(events.lock().unwrap().pop_front(), Some(AcpEvent::JournalFailed { detail }) if detail == "journal ACP saturé")
        );
    }

    #[test]
    fn expired_delivery_is_rejected_before_it_reaches_the_prompt_queue() {
        let mut message = message("expired");
        message.deadline_at = Some(0);
        assert!(message_expired(&message));
    }

    #[test]
    fn queued_message_is_purged_by_id_without_reordering_others() {
        let queue = queue();
        enqueue(&queue, 3, message("first"));
        enqueue(&queue, 3, message("cancel"));
        enqueue(&queue, 3, message("last"));
        let (state, _) = &*queue;
        let mut state = state.lock().unwrap();
        let index = state
            .messages
            .iter()
            .position(|message| message.id == "cancel")
            .unwrap();
        assert_eq!(state.messages.remove(index).unwrap().id, "cancel");
        assert_eq!(
            state
                .messages
                .iter()
                .map(|message| message.id.as_str())
                .collect::<Vec<_>>(),
            vec!["first", "last"]
        );
    }

    #[test]
    fn taille_evenement_bornee_et_message_terminal_preserve() {
        // La file ne réserve pas un BridgetMessage complet pour chaque delta.
        // Remettre le message inline fait dépasser cette borne structurelle.
        assert!(std::mem::size_of::<AcpEvent>() <= 8 * std::mem::size_of::<usize>());
        let original = message("terminal-boxed");
        let expected = serde_json::to_vec(&original).unwrap();
        let event = finish_turn(
            original,
            "réponse".to_string(),
            Ok(json!({"stopReason":"end_turn"})),
        );
        let ManagedEventKind::TurnFinished {
            message,
            response,
            terminal,
        } = managed_kind(event)
        else {
            panic!("fin de tour attendue");
        };
        assert_eq!(serde_json::to_vec(&message).unwrap(), expected);
        assert_eq!(response, "réponse");
        assert!(matches!(terminal, ManagedTerminal::Completed));
    }

    #[test]
    fn turn_normal_empty_and_error_have_distinct_events() {
        match finish_turn(
            message("normal"),
            "réponse".to_string(),
            Ok(json!({"stopReason":"end_turn"})),
        ) {
            AcpEvent::TurnFinished {
                response,
                stop_reason,
                ..
            } => {
                assert_eq!(response, "réponse");
                assert_eq!(stop_reason, "end_turn");
            }
            _ => panic!("tour normal attendu"),
        }
        match finish_turn(
            message("empty"),
            String::new(),
            Ok(json!({"stopReason":"end_turn"})),
        ) {
            AcpEvent::TurnFinished { response, .. } => assert!(response.is_empty()),
            _ => panic!("tour vide attendu"),
        }
        match finish_turn(
            message("error"),
            String::new(),
            Err(TransportError::DeliveryFailed("refus".to_string())),
        ) {
            AcpEvent::DeliveryRejected { message_id, reason } => {
                assert_eq!(message_id, "error");
                assert!(reason.contains("refus"));
            }
            _ => panic!("refus attendu"),
        }
    }

    #[test]
    fn shutdown_is_idempotent_before_a_process_is_spawned() {
        let alive = AtomicBool::new(true);
        assert!(alive.swap(false, Ordering::SeqCst));
        assert!(!alive.swap(false, Ordering::SeqCst));
    }

    #[test]
    fn false_adapter_exercises_stdio_reader_writer_and_prompt() {
        let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read request
echo '{"jsonrpc":"2.0","id":"permission-1","method":"session/request_permission","params":{"sessionId":"fixture-session","options":[{"optionId":"allow-1","kind":"allow_once"},{"optionId":"reject-1","kind":"reject_once"}]}}'
read permission
echo '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"fixture-session","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"réponse"}}}}'
echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"end_turn"}}'
"#;
        let mut transport = AcpTransport::spawn(AcpOptions {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), script.to_string()],
            queue_capacity: 2,
            permissions: "allow".to_string(),
            notify_timeout_secs: 1,
            provider_observation: None,
        })
        .unwrap();
        let message = message("fixture-message");
        transport.deliver(&message).unwrap();
        for _ in 0..20 {
            thread::sleep(Duration::from_millis(10));
            let events = transport.drain_events();
            if events.iter().any(|event| matches!(event, AcpEvent::TurnFinished { response, stop_reason, .. } if response == "réponse" && stop_reason == "end_turn")) {
                return;
            }
        }
        panic!("le faux adaptateur n'a pas terminé le tour");
    }

    #[test]
    fn anomalie_acp_ne_ferme_le_tour_qu_apres_un_terminal_atteste() {
        let root = std::env::temp_dir().join(format!(
            "bridget-acp-terminalite-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let release = root.join("release-terminal");
        std::fs::create_dir_all(&root).unwrap();
        let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read request
echo '{"jsonrpc":"2.0","method":"vendor/future","params":{"future":true}}'
echo '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"fixture-session","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"le tour continue"}}}}'
while [ ! -f "__RELEASE_TERMINAL__" ]; do sleep 0.01; done
echo '{"jsonrpc":"2.0","id":3,"error":{"code":-32000,"message":"refus fournisseur"}}'
while read request; do :; done
"#
        .replace(
            "__RELEASE_TERMINAL__",
            &release.display().to_string(),
        );
        let mut transport = AcpTransport::spawn(AcpOptions {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), script],
            queue_capacity: 1,
            permissions: "allow".to_string(),
            notify_timeout_secs: 5,
            provider_observation: None,
        })
        .unwrap();
        transport.enable_journal(&root, "acp-live").unwrap();
        transport.deliver(&message("message-live")).unwrap();

        let mut saw_protocol_error = false;
        let mut saw_continuation = false;
        for _ in 0..100 {
            thread::sleep(Duration::from_millis(10));
            for event in transport.drain_events() {
                saw_protocol_error |= matches!(
                    event,
                    AcpEvent::Error { ref detail }
                        if detail == "notification ACP inconnue: vendor/future"
                );
                saw_continuation |= matches!(
                    event,
                    AcpEvent::Update { ref detail } if detail == "le tour continue"
                );
            }
            if saw_protocol_error && saw_continuation {
                break;
            }
        }
        assert!(saw_protocol_error, "le faux ACP n'a pas émis son anomalie");
        assert!(saw_continuation, "le faux ACP n'a pas poursuivi le prompt");
        assert!(
            matches!(transport.state(), TurnState::InProgress { ref message_id, .. } if message_id == "message-live"),
            "l'anomalie protocole ne doit pas terminer le tour"
        );

        let read_events = || {
            let mut events = std::fs::read_dir(root.join("acp-live"))
                .unwrap()
                .filter_map(Result::ok)
                .flat_map(|entry| crate::journal::valid_events(&entry.path()))
                .collect::<Vec<_>>();
            events.sort_by_key(|event| event["seq"].as_u64());
            events
        };
        let before_terminal = (0..100)
            .find_map(|_| {
                let events = read_events();
                if events.iter().any(|event| event["event"] == "update") {
                    Some(events)
                } else {
                    thread::sleep(Duration::from_millis(10));
                    None
                }
            })
            .expect("journal ACP non terminal non écrit");
        let protocol_error = before_terminal
            .iter()
            .find(|event| event["event"] == "error")
            .expect("anomalie ACP absente du journal");
        assert!(protocol_error["payload"].get("terminal_kind").is_none());
        assert!(
            before_terminal
                .iter()
                .any(|event| event["event"] == "update")
        );
        assert!(!before_terminal.iter().any(|event| {
            event["event"] == "turn_end"
                || event["payload"].get("terminal_kind") == Some(&json!("turn_failed"))
        }));

        std::fs::write(&release, b"go").unwrap();
        let mut rejected = false;
        for _ in 0..100 {
            thread::sleep(Duration::from_millis(10));
            rejected |= transport.drain_events().iter().any(
                |event| matches!(event, AcpEvent::DeliveryRejected { message_id, .. } if message_id == "message-live"),
            );
            if rejected {
                break;
            }
        }
        assert!(rejected, "le vrai terminal du faux ACP n'a pas été observé");
        transport.shutdown();
        let after_terminal = read_events();
        let correlated_errors = after_terminal
            .iter()
            .filter(|event| event["event"] == "error" && event["message_id"] == "message-live")
            .collect::<Vec<_>>();
        let terminal_kind = correlated_errors
            .last()
            .and_then(|event| event["payload"].get("terminal_kind"))
            .cloned();
        let _ = std::fs::remove_dir_all(&root);
        assert_eq!(correlated_errors.len(), 2, "anomalie et terminal attendus");
        assert_eq!(terminal_kind, Some(json!("turn_failed")));
    }

    #[test]
    fn raw_source_line_survives_unusual_spacing_and_unknown_notification() {
        let raw = r#" { "method" : "vendor/future" , "params" : { "future" : true } }"#;
        let script = format!(
            r#"
read request
echo '{{"jsonrpc":"2.0","id":1,"result":{{"protocolVersion":1}}}}'
read request
echo '{{"jsonrpc":"2.0","id":2,"result":{{"sessionId":"fixture-session"}}}}'
printf '%s\n' '{raw}'
while read request; do :; done
"#
        );
        let transport = AcpTransport::spawn(AcpOptions {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), script],
            queue_capacity: 1,
            permissions: "allow".to_string(),
            notify_timeout_secs: 1,
            provider_observation: None,
        })
        .unwrap();

        for _ in 0..20 {
            thread::sleep(Duration::from_millis(10));
            let events = ManagedSession::drain_events(&transport);
            if let Some(event) = events.into_iter().find(|event| {
                matches!(event.kind, ManagedEventKind::Error { ref detail } if detail == "notification ACP inconnue: vendor/future")
            }) {
                assert_eq!(event.source, ManagedEventSource::Acp);
                assert_eq!(event.origin, crate::managed_session::ManagedEventOrigin::SourceLine);
                assert_eq!(event.raw, raw.as_bytes());
                return;
            }
        }
        panic!("la notification source inconnue n'a pas traversé la frontière");
    }

    fn observation_case(view_count: usize, root: &std::path::Path) -> Vec<Duration> {
        const TURNS: usize = 200;
        let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
id=3
while read request; do
  printf '{"jsonrpc":"2.0","id":%s,"result":{"stopReason":"end_turn"}}\n' "$id"
  id=$((id + 1))
done
"#;
        let mut transport = AcpTransport::spawn(AcpOptions {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), script.to_string()],
            queue_capacity: 1,
            permissions: "allow".to_string(),
            notify_timeout_secs: 1,
            provider_observation: None,
        })
        .unwrap();
        transport.enable_journal(root, "codex-bench").unwrap();
        let views = (0..view_count)
            .map(|_| Arc::new(Mutex::new(Vec::<Vec<u8>>::new())))
            .collect::<Vec<_>>();
        for turn in 0..TURNS {
            let message = BridgetMessage::new("humain", "codex-bench", format!("tour-{turn}"));
            transport.deliver(&message).unwrap();
            let deadline = Instant::now() + Duration::from_secs(2);
            loop {
                let finished = transport.drain_events().into_iter().any(|event| {
                    matches!(event, AcpEvent::TurnFinished { message: done, .. } if done.id == message.id)
                });
                if finished {
                    // Deux vues attach simulées consomment la même projection
                    // sans jamais entrer dans le thread d'append JSONL.
                    let projection = format!("{}:{}", message.id, message.body).into_bytes();
                    for view in &views {
                        view.lock().unwrap().push(projection.clone());
                    }
                    break;
                }
                assert!(Instant::now() < deadline, "tour déterministe non terminé");
                thread::sleep(Duration::from_millis(1));
            }
        }
        let journal = transport
            .journal
            .lock()
            .unwrap()
            .clone()
            .expect("journal activé");
        journal.stop();
        let latencies = journal.take_append_latencies();
        transport.shutdown();
        assert!(views.iter().all(|view| view.lock().unwrap().len() == TURNS));
        latencies
    }

    fn p95(samples: &[Duration]) -> Duration {
        let mut ordered = samples.to_vec();
        ordered.sort_unstable();
        ordered[(ordered.len() * 95).div_ceil(100).saturating_sub(1)]
    }

    #[test]
    #[ignore = "remplacé par le banc SC-005 à vues attach réelles"]
    fn sc005_deux_vues_ne_degradent_pas_le_p95_d_append_de_plus_de_cinq_pourcent() {
        const TURNS: usize = 200;
        let root = std::env::temp_dir().join(format!(
            "bridget-sc005-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let baseline = observation_case(0, &root.join("zero"));
        let observed = observation_case(2, &root.join("two"));
        assert!(
            baseline.len() >= TURNS && observed.len() >= TURNS,
            "append instrumentés insuffisants : 0 vue={}, 2 vues={}",
            baseline.len(),
            observed.len()
        );
        let baseline_p95 = p95(&baseline);
        let observed_p95 = p95(&observed);
        eprintln!(
            "SC-005 append p95: 0 vue={baseline_p95:?} ({} échantillons), 2 vues={observed_p95:?} ({} échantillons)",
            baseline.len(),
            observed.len(),
        );
        assert!(
            observed_p95.as_nanos() * 100 < baseline_p95.as_nanos() * 105,
            "p95 append 2 vues={observed_p95:?}, 0 vue={baseline_p95:?}"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn prompt_dispatched_suit_le_flush_et_le_faux_adaptateur_lit_la_frame() {
        let script = r#"
read initialize
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read session
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read prompt
echo '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"fixture-session","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"prompt-lu"}}}}'
echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"end_turn"}}'
"#;
        let (observer, observed_events) = mpsc::channel();
        let mut transport = AcpTransport::spawn_with_clock_and_cancel_grace(
            AcpOptions {
                command: "sh".to_string(),
                args: vec!["-c".to_string(), script.to_string()],
                queue_capacity: 1,
                permissions: "allow".to_string(),
                notify_timeout_secs: 1,
                provider_observation: None,
            },
            Arc::new(SystemTime::now),
            CANCEL_GRACE,
            CANCEL_POLL,
            Some(observer),
        )
        .unwrap();
        transport.deliver(&message("prompt-dispatched")).unwrap();
        assert!(matches!(
            observed_events.recv_timeout(Duration::from_secs(2)),
            Ok(AcpEvent::PromptDispatched { message_id }) if message_id == "prompt-dispatched"
        ));
        assert!(
            matches!(
                observed_events.recv_timeout(Duration::from_secs(2)),
                Ok(AcpEvent::Update { detail }) if detail == "prompt-lu"
            ),
            "le faux adaptateur n'a pas lu la frame session/prompt"
        );
    }

    #[test]
    fn echec_avant_flush_n_emet_pas_prompt_dispatched() {
        let root = std::env::temp_dir().join(format!(
            "bridget-acp-flush-before-dispatch-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let stdin_closed = root.join("stdin-closed");
        let script = r#"
read initialize
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read session
exec 0<&-
touch "__BRIDGET_STDIN_CLOSED__"
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
sleep 30
"#
        .replace(
            "__BRIDGET_STDIN_CLOSED__",
            &stdin_closed.display().to_string(),
        );
        let (observer, observed_events) = mpsc::channel();
        let mut transport = AcpTransport::spawn_with_clock_and_cancel_grace(
            AcpOptions {
                command: "sh".to_string(),
                args: vec!["-c".to_string(), script],
                queue_capacity: 1,
                permissions: "allow".to_string(),
                notify_timeout_secs: 1,
                provider_observation: None,
            },
            Arc::new(SystemTime::now),
            CANCEL_GRACE,
            CANCEL_POLL,
            Some(observer),
        )
        .unwrap();
        // La réponse session/new n'est écrite par l'adaptateur qu'après la
        // fermeture de stdin et ce jalon. spawn() est donc une barrière : le
        // prompt ne peut pas être tenté contre un lecteur encore ouvert.
        assert!(
            stdin_closed.exists(),
            "le jalon de fermeture stdin est absent"
        );
        transport.deliver(&message("flush-failed")).unwrap();
        match observed_events.recv_timeout(Duration::from_secs(2)) {
            Ok(AcpEvent::DeliveryRejected { message_id, .. }) => {
                assert_eq!(message_id, "flush-failed")
            }
            Ok(AcpEvent::PromptDispatched { .. }) => {
                panic!("un échec avant flush ne peut pas être acquitté")
            }
            Ok(other) => panic!("issue ACP inattendue: {other:?}"),
            Err(error) => panic!("échec de flush non observé: {error}"),
        }
        drop(transport);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn enabled_journal_records_a_complete_transport_turn() {
        let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read request
echo '{"jsonrpc":"2.0","id":"permission-1","method":"session/request_permission","params":{"sessionId":"fixture-session","options":[{"optionId":"allow-1","kind":"allow_once"},{"optionId":"reject-1","kind":"reject_once"}]}}'
read permission
echo '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"fixture-session","update":{"sessionUpdate":"tool_call","toolCallId":"tool-1","title":"Read src/main.rs","name":"read_file","kind":"read","content":{"type":"text","text":"lecture"}}}}'
echo '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"fixture-session","update":{"sessionUpdate":"tool_call_update","toolCallId":"tool-1","content":{"type":"text","text":"analyse"}}}}'
echo '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"fixture-session","update":{"sessionUpdate":"tool_call_update","toolCallId":"tool-1","content":{"type":"text","text":"terminé"}}}}'
echo '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"fixture-session","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"réponse"}}}}'
echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"end_turn"}}'
"#;
        let root = std::env::temp_dir().join(format!("bridget-acp-journal-{}", std::process::id()));
        let mut transport = AcpTransport::spawn(AcpOptions {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), script.to_string()],
            queue_capacity: 2,
            permissions: "allow".to_string(),
            notify_timeout_secs: 1,
            provider_observation: None,
        })
        .unwrap();
        transport.enable_journal(&root, "codex-1").unwrap();
        transport.deliver(&message("journal-message")).unwrap();
        for _ in 0..30 {
            thread::sleep(Duration::from_millis(10));
            if transport
                .drain_events()
                .iter()
                .any(|event| matches!(event, AcpEvent::TurnFinished { .. }))
            {
                transport.shutdown();
                let path = std::fs::read_dir(root.join("codex-1"))
                    .unwrap()
                    .next()
                    .unwrap()
                    .unwrap()
                    .path();
                let events = crate::journal::valid_events(&path)
                    .into_iter()
                    .map(|mut event| {
                        event.as_object_mut().unwrap().remove("seq");
                        event.as_object_mut().unwrap().remove("ts");
                        event
                    })
                    .collect::<Vec<_>>();
                assert!(events.contains(&json!({
                    "v": 1, "session_id": "fixture-session", "event": "turn_start",
                    "message_id": "journal-message",
                    "payload": {"from":"alice", "reply":false, "body":"journal-message"}
                })));
                assert!(events.contains(&json!({
                    "v": 1, "session_id": "fixture-session", "event": "update",
                    "message_id": "journal-message",
                    "payload": {"kind":"text", "content":"réponse"}
                })));
                assert!(events.contains(&json!({
                    "v": 1, "session_id": "fixture-session", "event": "update",
                    "message_id": "journal-message",
                    "payload": {
                        "kind":"tool", "text":"Read src/main.rs", "detail":"lecture",
                        "tool_call_id":"tool-1", "tool":"Read src/main.rs",
                        "title":"Read src/main.rs", "name":"read_file",
                        "tool_kind":"read", "summary":"lecture"
                    }
                })));
                for summary in ["analyse", "terminé"] {
                    assert!(
                        events.contains(&json!({
                            "v": 1, "session_id": "fixture-session", "event": "update",
                            "message_id": "journal-message",
                            "payload": {
                                "kind":"tool", "text":"Read src/main.rs", "detail":summary,
                                "tool_call_id":"tool-1", "tool":"Read src/main.rs",
                                "title":"Read src/main.rs", "summary":summary
                            }
                        })),
                        "la mise à jour ACP {summary} doit porter le titre corrélé"
                    );
                }
                assert!(events.contains(&json!({
                    "v": 1, "session_id": "fixture-session", "event": "permission",
                    "message_id": "journal-message",
                    "payload": {
                        "tool":"inconnu",
                        "options":[
                            {"optionId":"allow-1","kind":"allow_once"},
                            {"optionId":"reject-1","kind":"reject_once"}
                        ],
                        "decision":{"outcome":"selected","option_id":"allow-1"}
                    }
                })));
                assert!(
                    events.contains(&json!({
                        "v": 1, "session_id": "fixture-session", "event": "reasoning",
                        "message_id": "journal-message",
                        "payload": {"available": false}
                    })),
                    "sans thought_chunk : ligne terminale available:false"
                );
                let reasoning_lines: Vec<_> = events
                    .iter()
                    .filter(|event| event["event"] == "reasoning")
                    .collect();
                assert_eq!(
                    reasoning_lines.len(),
                    1,
                    "exactement un event=reasoning terminal par tour, got {reasoning_lines:?}"
                );
                assert!(events.contains(&json!({
                    "v": 1, "session_id": "fixture-session", "event": "turn_end",
                    "message_id": "journal-message",
                    "payload": {"stop_reason":"end_turn"}
                })));
                // L'EOF peut être lu juste avant ou juste après le résultat
                // terminal du tour : le journal conserve alors, de façon
                // légitime, le message courant ou aucun message corrélé.
                assert!(events.iter().any(|event| {
                    event["event"] == "error"
                        && match event.get("message_id") {
                            None => true,
                            Some(serde_json::Value::String(id)) => id == "journal-message",
                            _ => false,
                        }
                }));
                std::fs::remove_dir_all(root).unwrap();
                return;
            }
        }
        panic!("le tour journalisé n'a pas terminé");
    }

    #[test]
    fn false_adapter_handles_permission_during_a_prompt() {
        let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read request
echo '{"jsonrpc":"2.0","id":"permission-1","method":"session/request_permission","params":{"sessionId":"fixture-session","options":[{"optionId":"allow-1","kind":"allow_once"}]}}'
read permission
echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"end_turn"}}'
"#;
        let mut transport = AcpTransport::spawn(AcpOptions {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), script.to_string()],
            queue_capacity: 2,
            permissions: "allow".to_string(),
            notify_timeout_secs: 1,
            provider_observation: None,
        })
        .unwrap();
        transport.deliver(&message("permission-message")).unwrap();
        for _ in 0..20 {
            thread::sleep(Duration::from_millis(10));
            if transport
                .drain_events()
                .iter()
                .any(|event| matches!(event, AcpEvent::TurnFinished { .. }))
            {
                return;
            }
        }
        panic!("la permission pendant le prompt n'a pas été traitée");
    }

    #[test]
    fn false_adapter_asserts_the_raw_official_permission_envelope() {
        let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read request
echo '{"jsonrpc":"2.0","id":"permission-1","method":"session/request_permission","params":{"sessionId":"fixture-session","options":[{"optionId":"allow-1","kind":"allow_once"}]}}'
read permission
case "$permission" in
  *'"result":{"outcome":{"optionId":"allow-1","outcome":"selected"}}'*)
    echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"end_turn"}}' ;;
  *) exit 12 ;;
esac
"#;
        let mut transport = AcpTransport::spawn(AcpOptions {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), script.to_string()],
            queue_capacity: 2,
            permissions: "allow".to_string(),
            notify_timeout_secs: 1,
            provider_observation: None,
        })
        .unwrap();
        transport.deliver(&message("permission-raw")).unwrap();
        for _ in 0..30 {
            thread::sleep(Duration::from_millis(10));
            if transport
                .drain_events()
                .iter()
                .any(|event| matches!(event, AcpEvent::TurnFinished { .. }))
            {
                return;
            }
        }
        panic!("l'enveloppe brute de permission officielle n'a pas été acceptée");
    }

    #[test]
    fn foreign_session_update_is_ignored_and_diagnosed() {
        let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read request
echo '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"foreign-session","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"interdit"}}}}'
echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"end_turn"}}'
"#;
        let mut transport = AcpTransport::spawn(AcpOptions {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), script.to_string()],
            queue_capacity: 2,
            permissions: "allow".to_string(),
            notify_timeout_secs: 1,
            provider_observation: None,
        })
        .unwrap();
        transport.deliver(&message("foreign-update")).unwrap();
        for _ in 0..30 {
            thread::sleep(Duration::from_millis(10));
            let events = transport.drain_events();
            if events.iter().any(|event| matches!(event, AcpEvent::TurnFinished { response, .. } if response.is_empty())) {
                assert!(events.iter().any(|event| matches!(event, AcpEvent::Error { detail } if detail.contains("session étrangère"))));
                return;
            }
        }
        panic!("l'update de session étrangère n'a pas été diagnostiquée");
    }

    #[test]
    fn cancelled_turn_waits_for_adapter_before_the_next_prompt() {
        let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read first_prompt
read cancel
echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"cancelled"}}'
read second_prompt
echo '{"jsonrpc":"2.0","id":4,"result":{"stopReason":"end_turn"}}'
"#;
        let mut transport = AcpTransport::spawn(AcpOptions {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), script.to_string()],
            queue_capacity: 2,
            permissions: "allow".to_string(),
            notify_timeout_secs: 1,
            provider_observation: None,
        })
        .unwrap();
        transport.deliver(&message("cancelled")).unwrap();
        transport.deliver(&message("next")).unwrap();
        for _ in 0..30 {
            thread::sleep(Duration::from_millis(10));
            if matches!(transport.state(), TurnState::InProgress { ref message_id, .. } if message_id == "cancelled")
            {
                break;
            }
        }
        assert!(transport.cancel_delivery("cancelled", "annulation daemon"));
        let mut rejected = false;
        let mut next_finished = false;
        for _ in 0..60 {
            thread::sleep(Duration::from_millis(10));
            for event in transport.drain_events() {
                rejected |= matches!(event, AcpEvent::DeliveryRejected { ref message_id, .. } if message_id == "cancelled");
                next_finished |=
                    matches!(event, AcpEvent::TurnFinished { message, .. } if message.id == "next");
            }
            if rejected && next_finished {
                return;
            }
        }
        panic!("le tour annulé a empêché ou contaminé le prompt suivant");
    }

    #[test]
    fn message_humain_actif_interrompt_le_tour_acp_et_garde_fifo() {
        let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read first_prompt
read cancel
echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"cancelled"}}'
read human_prompt
echo '{"jsonrpc":"2.0","id":4,"result":{"stopReason":"end_turn"}}'
read system_prompt
echo '{"jsonrpc":"2.0","id":5,"result":{"stopReason":"end_turn"}}'
"#;
        let mut transport = AcpTransport::spawn(AcpOptions {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), script.to_string()],
            queue_capacity: 3,
            permissions: "allow".to_string(),
            notify_timeout_secs: 1,
            provider_observation: None,
        })
        .unwrap();
        transport.deliver(&message("acp-actif")).unwrap();
        let start_deadline = Instant::now() + Duration::from_secs(2);
        while !transport.is_busy() && Instant::now() < start_deadline {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(transport.is_busy(), "le tour ACP initial doit être actif");

        let mut human = message("acp-humain");
        human.from = "superviseur".to_string();
        human.origin = Some(MessageOrigin::Human);
        transport.deliver(&human).unwrap();
        transport.deliver(&message("acp-systeme")).unwrap();

        let deadline = Instant::now() + Duration::from_secs(3);
        let mut active_rejected = false;
        let mut completed = Vec::new();
        while Instant::now() < deadline {
            for event in transport.drain_events() {
                match event {
                    AcpEvent::DeliveryRejected { message_id, .. } if message_id == "acp-actif" => {
                        active_rejected = true;
                    }
                    AcpEvent::TurnFinished { message, .. } => completed.push(message.id),
                    _ => {}
                }
            }
            if active_rejected
                && completed == vec!["acp-humain".to_string(), "acp-systeme".to_string()]
            {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!(
            "le message humain devait interrompre seulement le tour actif et rester avant le système, annulé={active_rejected}, terminés={completed:?}"
        );
    }

    #[test]
    fn active_reply_deadline_uses_cancel_grace_then_drains_queue() {
        let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read prompt
echo '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"fixture-session","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"prompt-observe"}}}}'
read cancel
echo '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"fixture-session","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"cancel-observe"}}}}'
while :; do :; done
"#;
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let observed_now = Arc::new(AtomicU64::new(now));
        let clock_now = observed_now.clone();
        let clock: Clock = Arc::new(move || {
            SystemTime::UNIX_EPOCH + Duration::from_secs(clock_now.load(Ordering::SeqCst))
        });
        let (observer, observed_events) = mpsc::channel();
        let mut transport = AcpTransport::spawn_with_clock_and_cancel_grace(
            AcpOptions {
                command: "sh".to_string(),
                args: vec!["-c".to_string(), script.to_string()],
                queue_capacity: 2,
                permissions: "allow".to_string(),
                notify_timeout_secs: 1,
                provider_observation: None,
            },
            clock,
            Duration::from_millis(100),
            Duration::from_millis(10),
            Some(observer),
        )
        .unwrap();
        let mut active = message("deadline-active");
        active.reply = true;
        active.deadline_at = Some(now + 1);
        transport.deliver(&active).unwrap();
        transport.deliver(&message("deadline-queued")).unwrap();
        assert!(
            matches!(
                observed_events.recv_timeout(Duration::from_secs(2)),
                Ok(AcpEvent::PromptDispatched { message_id }) if message_id == "deadline-active"
            ),
            "la frontière write+flush n'a pas été publiée"
        );
        assert!(
            matches!(
                observed_events.recv_timeout(Duration::from_secs(2)),
                Ok(AcpEvent::Update { detail }) if detail == "prompt-observe"
            ),
            "le faux adaptateur n'a pas observé le prompt"
        );
        observed_now.store(now + 1, Ordering::SeqCst);
        assert!(
            matches!(
                observed_events.recv_timeout(Duration::from_secs(2)),
                Ok(AcpEvent::Update { detail }) if detail == "cancel-observe"
            ),
            "le faux adaptateur n'a pas observé session/cancel avant la grâce"
        );
        assert!(
            matches!(
                observed_events.recv_timeout(Duration::from_secs(2)),
                Ok(AcpEvent::DeliveryRejected { message_id, reason })
                    if message_id == "deadline-active" && reason.contains("timeout ACP")
            ),
            "l'échéance injectée n'a pas interrompu le prompt ACP"
        );
        let terminal_events = transport.drain_events();
        assert!(
            terminal_events.iter().any(|event| matches!(
                event,
                AcpEvent::DeliveryRejected { message_id, .. } if message_id == "deadline-queued"
            )),
            "la file n'a pas été drainée après l'annulation forcée"
        );
    }

    #[test]
    fn ignored_cancel_kills_transport_and_drains_the_queue() {
        let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read prompt
read cancel
sleep 2
"#;
        let mut transport = AcpTransport::spawn(AcpOptions {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), script.to_string()],
            queue_capacity: 2,
            permissions: "allow".to_string(),
            notify_timeout_secs: 1,
            provider_observation: None,
        })
        .unwrap();
        transport.deliver(&message("active")).unwrap();
        transport.deliver(&message("queued")).unwrap();
        for _ in 0..30 {
            thread::sleep(Duration::from_millis(10));
            if matches!(transport.state(), TurnState::InProgress { ref message_id, .. } if message_id == "active")
            {
                break;
            }
        }
        assert!(transport.cancel_delivery("active", "annulation daemon"));
        let deadline = Instant::now() + CANCEL_GRACE + Duration::from_secs(1);
        let mut active_rejected = false;
        let mut queued_rejected = false;
        loop {
            for event in transport.drain_events() {
                active_rejected |= matches!(
                    event,
                    AcpEvent::DeliveryRejected { ref message_id, .. } if message_id == "active"
                );
                queued_rejected |= matches!(
                    event,
                    AcpEvent::DeliveryRejected { ref message_id, .. } if message_id == "queued"
                );
            }
            if active_rejected && queued_rejected {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "l'arrêt ACP n'a pas publié tous ses rejets (active={active_rejected}, queued={queued_rejected})"
            );
            thread::sleep(Duration::from_millis(10));
        }
        assert!(!transport.is_alive());
    }

    #[test]
    fn notification_timeout_uses_cancel_grace_then_forced_drain() {
        let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read prompt
read cancel
sleep 2
"#;
        let mut transport = AcpTransport::spawn(AcpOptions {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), script.to_string()],
            queue_capacity: 2,
            permissions: "allow".to_string(),
            notify_timeout_secs: 0,
            provider_observation: None,
        })
        .unwrap();
        transport.deliver(&message("timeout-active")).unwrap();
        transport.deliver(&message("timeout-queued")).unwrap();
        thread::sleep(CANCEL_GRACE + Duration::from_millis(100));
        assert!(!transport.is_alive());
        let events = transport.drain_events();
        assert!(events.iter().any(|event| matches!(event, AcpEvent::DeliveryRejected { message_id, reason } if message_id == "timeout-active" && reason.contains("timeout ACP"))));
        assert!(events.iter().any(|event| matches!(event, AcpEvent::DeliveryRejected { message_id, .. } if message_id == "timeout-queued")));
    }

    /// ORACLE — un mandat reply=false (tous les mandats le service compagnon) honore
    /// `deadline_at` absolu du daemon, PAS le notify_timeout figé au spawn.
    /// Mutant : remettre `(!reply).then_some(notify_timeout)` exclusif → meurt.
    /// Preuve d'abord qu'un tour normal aboutit (sinon projection vide).
    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_deadline_daemon_sauve_un_tour_reply_false_malgre_notify_fige() {
        let root = std::env::temp_dir().join(format!(
            "bridget-acp-deadline-hot-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();

        // Tour normal : aboutit malgré notify_timeout=1 (complète avant 1 s).
        let script_ok = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read prompt
echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"end_turn"}}'
"#;
        let mut transport_ok = AcpTransport::spawn(AcpOptions {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), script_ok.to_string()],
            queue_capacity: 2,
            permissions: "allow".to_string(),
            notify_timeout_secs: 1,
            provider_observation: None,
        })
        .unwrap();
        transport_ok.enable_journal(&root, "ok").unwrap();
        let mut ok_msg = message("tour-normal");
        ok_msg.reply = false;
        transport_ok.deliver(&ok_msg).unwrap();
        let mut saw_ok = false;
        for _ in 0..50 {
            thread::sleep(Duration::from_millis(20));
            if transport_ok.drain_events().iter().any(|event| {
                matches!(
                    event,
                    AcpEvent::TurnFinished { message, .. } if message.id == "tour-normal"
                )
            }) {
                saw_ok = true;
                break;
            }
        }
        assert!(
            saw_ok,
            "contrôle positif : un tour court doit aboutir d'abord"
        );

        // Tour long : notify figé à 1 s, mais deadline_at (daemon) à +30 s.
        // Le faux ACP répond après 2 s → survit grâce au deadline daemon.
        let script_long = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read prompt
sleep 2
echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"end_turn"}}'
"#;
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut transport_long = AcpTransport::spawn(AcpOptions {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), script_long.to_string()],
            queue_capacity: 2,
            permissions: "allow".to_string(),
            notify_timeout_secs: 1,
            provider_observation: None,
        })
        .unwrap();
        transport_long.enable_journal(&root, "long").unwrap();
        let mut long_msg = message("tour-deadline-hot");
        long_msg.reply = false;
        long_msg.deadline_at = Some(now + 30);
        transport_long.deliver(&long_msg).unwrap();
        let mut saw_long = false;
        let mut saw_timeout = false;
        for _ in 0..200 {
            thread::sleep(Duration::from_millis(50));
            for event in transport_long.drain_events() {
                match event {
                    AcpEvent::TurnFinished { message, .. } if message.id == "tour-deadline-hot" => {
                        saw_long = true;
                    }
                    AcpEvent::DeliveryRejected { message_id, reason }
                        if message_id == "tour-deadline-hot" && reason.contains("timeout ACP") =>
                    {
                        saw_timeout = true;
                    }
                    _ => {}
                }
            }
            if saw_long || saw_timeout {
                break;
            }
        }
        assert!(
            saw_long && !saw_timeout,
            "deadline_at daemon doit sauver reply=false malgré notify_timeout=1 ; \
             finished={saw_long} timeout={saw_timeout}"
        );

        // Sans deadline_at, le même script tombe à 1 s — le signal est DIT.
        let mut transport_kill = AcpTransport::spawn(AcpOptions {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), script_long.to_string()],
            queue_capacity: 2,
            permissions: "allow".to_string(),
            notify_timeout_secs: 1,
            provider_observation: None,
        })
        .unwrap();
        transport_kill.enable_journal(&root, "kill").unwrap();
        let mut kill_msg = message("tour-tue");
        kill_msg.reply = false;
        // pas de deadline_at → notify_timeout figé gagne
        transport_kill.deliver(&kill_msg).unwrap();
        let mut kill_reason = None;
        for _ in 0..80 {
            thread::sleep(Duration::from_millis(50));
            for event in transport_kill.drain_events() {
                if let AcpEvent::DeliveryRejected { message_id, reason } = event
                    && message_id == "tour-tue"
                {
                    kill_reason = Some(reason);
                }
            }
            if kill_reason.is_some() {
                break;
            }
        }
        let kill_reason = kill_reason.expect("sans deadline, le tour long doit être tué");
        assert!(
            kill_reason.contains("timeout ACP"),
            "signal d'échéance attendu, reçu {kill_reason}"
        );
        let kill_events = std::fs::read_dir(root.join("kill"))
            .unwrap()
            .filter_map(Result::ok)
            .flat_map(|entry| crate::journal::valid_events(&entry.path()))
            .collect::<Vec<_>>();
        let terminal = kill_events
            .iter()
            .find(|event| event["event"] == "error" && event["message_id"] == "tour-tue");
        assert!(
            terminal
                .is_some_and(|event| { event["payload"]["terminal_kind"] == json!("turn_failed") }),
            "un tour tué DOIT porter terminal_kind=turn_failed ; reçu {kill_events:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn false_adapter_eof_rejects_an_active_turn() {
        let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read request
exit 0
"#;
        let mut transport = AcpTransport::spawn(AcpOptions {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), script.to_string()],
            queue_capacity: 2,
            permissions: "allow".to_string(),
            notify_timeout_secs: 1,
            provider_observation: None,
        })
        .unwrap();
        transport.deliver(&message("eof-message")).unwrap();
        transport.deliver(&message("eof-queued")).unwrap();
        for _ in 0..20 {
            thread::sleep(Duration::from_millis(10));
            let events = transport.drain_events();
            if events.iter().any(|event| matches!(event, AcpEvent::DeliveryRejected { message_id, reason } if message_id == "eof-message" && reason.contains("EOF ACP")))
                && events.iter().any(|event| matches!(event, AcpEvent::DeliveryRejected { message_id, reason } if message_id == "eof-queued" && reason.contains("EOF ACP"))) {
                return;
            }
        }
        panic!("EOF actif non propagé au tour");
    }
    #[test]
    fn consigne_privee_preserve_le_corps_visible_du_message() {
        let mut message = message("profile-body");
        message.body = "Demande utilisateur visible.".to_string();
        let prompt = prompt_for_with_private_instructions(
            &message,
            Some("Privilégie les sources attestées."),
        );

        assert!(prompt.contains("Privilégie les sources attestées."));
        assert!(prompt.ends_with(&message.body));
        assert!(prompt_for(&message).ends_with(&message.body));
        assert!(prompt_for(&message).contains(&message.id));
    }
}
