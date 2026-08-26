//! Pilote natif de `codex app-server`.
//!
//! Le protocole est volontairement porté en `serde_json::Value` : le schéma
//! généré par le binaire Codex installé est l'autorité de compatibilité. La
//! crate publiée `codex-app-server-protocol` suit un autre produit et une
//! autre numérotation ; la lier ici cacherait une incompatibilité possible.

use crate::journal::{JournalFailureSink, JournalLiveFeed, JournalWriter, with_turn_failed_kind};
use crate::managed_session::{
    ManagedEvent, ManagedEventKind, ManagedEventOrigin, ManagedEventSource, ManagedSession,
    ManagedSessionDescriptor, ManagedTerminal,
};
use crate::protocol::PresenceMode;
use crate::transport::{Transport, TransportError};
use bridget_core::BridgetMessage;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const RATE_LIMIT_READ_TIMEOUT: Duration = Duration::from_secs(2);
const TURN_POLL: Duration = Duration::from_millis(25);
const SATURATION_RETRIES: u32 = 4;
const CODEX_SATURATED_REASON: &str = "saturation Codex";
const COMMAND_EXECUTION_APPROVAL_METHOD: &str = "item/commandExecution/requestApproval";

type Writer = Arc<Mutex<Option<ChildStdin>>>;
type Waiters = Arc<Mutex<HashMap<u64, mpsc::Sender<Result<ServerResponse, String>>>>>;
type Journal = Arc<Mutex<Option<JournalWriter>>>;
type PendingRequest = Arc<Mutex<Option<PendingProviderRequest>>>;

#[derive(Debug)]
struct ServerResponse {
    value: Value,
    raw: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct CodexAppServerOptions {
    pub command: String,
    pub args: Vec<String>,
    pub queue_capacity: usize,
    pub notify_timeout_secs: u64,
    pub model: Option<String>,
}

#[derive(Debug)]
struct ActiveTurn {
    message_id: String,
    cancel: mpsc::Sender<String>,
}

#[derive(Debug)]
struct QueueState {
    messages: VecDeque<BridgetMessage>,
    active: Option<ActiveTurn>,
    closed: bool,
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
}

// Les deltas restent provisoires : seul le worker atteste leur présence au terminal.
#[derive(Debug, Default)]
struct CodexTurnDetail {
    message_id: String,
    thread_id: String,
    turn_id: Option<String>,
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
    Command,
    // Producteur réel (`item/fileChange/patchUpdated` → record_active_act), mais
    // 0 occurrence dans les journaux relec* du 2026-08-26 : Codex n'a pas émis
    // cette méthode sur les tours mesurés (seulement commandExecution + approval).
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
        self.as_update_kind().as_str()
    }
}

struct ReaderContext {
    waiters: Waiters,
    observations: Arc<(Mutex<Observations>, Condvar)>,
    alive: Arc<AtomicBool>,
    journal: Journal,
    queue: Arc<(Mutex<QueueState>, Condvar)>,
    pending_request: PendingRequest,
    pinned_model: Option<String>,
    active_detail: ActiveTurnDetail,
}


pub struct CodexAppServerTransport {
    connection_id: String,
    alive: Arc<AtomicBool>,
    shutdown_started: AtomicBool,
    busy: Arc<AtomicBool>,
    queue: Arc<(Mutex<QueueState>, Condvar)>,
    queue_capacity: usize,
    writer: Writer,
    thread_id: String,
    child: Arc<Mutex<Child>>,
    observations: Arc<(Mutex<Observations>, Condvar)>,
    journal: Journal,
    reader_handle: Mutex<Option<thread::JoinHandle<()>>>,
    worker_handle: Mutex<Option<thread::JoinHandle<()>>>,
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
        if options.queue_capacity == 0 {
            return Err(TransportError::DeliveryFailed(
                "queue Codex de capacité nulle".to_string(),
            ));
        }
        let mut command = Command::new(&options.command);
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
        let mut child = command.spawn().map_err(|error| {
            TransportError::Io(format!("impossible de lancer codex app-server: {error}"))
        })?;
        let pid = child.id();
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| TransportError::Io("stdin Codex absent".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| TransportError::Io("stdout Codex absent".to_string()))?;
        let writer = Arc::new(Mutex::new(Some(stdin)));
        let waiters = Arc::new(Mutex::new(HashMap::new()));
        let observations = Arc::new((Mutex::new(Observations::default()), Condvar::new()));
        let active_detail = Arc::new(Mutex::new(None));
        let alive = Arc::new(AtomicBool::new(true));
        let next_id = Arc::new(AtomicU64::new(1));
        let journal = Arc::new(Mutex::new(None));
        let queue = Arc::new((
            Mutex::new(QueueState {
                messages: VecDeque::new(),
                active: None,
                closed: false,
            }),
            Condvar::new(),
        ));
        let pending_request = Arc::new(Mutex::new(None));
        let reader_handle = spawn_reader(
            stdout,
            ReaderContext {
                waiters: waiters.clone(),
                observations: observations.clone(),
                alive: alive.clone(),
                journal: journal.clone(),
                queue: queue.clone(),
                pending_request: pending_request.clone(),
                pinned_model: options.model.clone(),
                active_detail: active_detail.clone(),
            },
        );

        let setup = (|| -> Result<String, TransportError> {
            let initialize = request(
                &writer,
                &waiters,
                &next_id,
                "initialize",
                json!({
                    "clientInfo": {
                        "name": "bridget",
                        "version": env!("CARGO_PKG_VERSION"),
                    },
                }),
            )?;
            for required in ["userAgent", "codexHome", "platformFamily", "platformOs"] {
                if initialize.value.get(required).is_none() {
                    return Err(TransportError::DeliveryFailed(format!(
                        "initialize Codex ne retourne pas {required}"
                    )));
                }
            }
            write_notification(&writer, "initialized", json!({}))?;
            let cwd =
                std::env::current_dir().map_err(|error| TransportError::Io(error.to_string()))?;
            let mut thread_params = json!({ "cwd": cwd });
            if let Some(model) = &options.model {
                thread_params["model"] = Value::String(model.clone());
            }
            let thread = request(&writer, &waiters, &next_id, "thread/start", thread_params)?;
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
            thread
                .value
                .pointer("/thread/id")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| {
                    TransportError::DeliveryFailed(
                        "thread/start Codex ne retourne pas thread.id".to_string(),
                    )
                })
        })();
        let thread_id = match setup {
            Ok(thread_id) => thread_id,
            Err(error) => {
                writer
                    .lock()
                    .unwrap_or_else(|poison| poison.into_inner())
                    .take();
                let mut child = child;
                let _ = child.kill();
                let _ = child.wait();
                let _ = reader_handle.join();
                return Err(error);
            }
        };
        let child = Arc::new(Mutex::new(child));
        let busy = Arc::new(AtomicBool::new(false));
        let worker_handle = spawn_worker(Worker {
            queue: queue.clone(),
            writer: writer.clone(),
            waiters: waiters.clone(),
            next_id,
            observations: observations.clone(),
            alive: alive.clone(),
            busy: busy.clone(),
            thread_id: thread_id.clone(),
            journal: journal.clone(),
            active_detail,
            pending_request,
            notify_timeout: Duration::from_secs(options.notify_timeout_secs),
        });

        Ok(Self {
            connection_id: format!("codex-app-server-{pid}"),
            alive,
            shutdown_started: AtomicBool::new(false),
            busy,
            queue,
            queue_capacity: options.queue_capacity,
            writer,
            thread_id,
            child,
            observations,
            journal,
            reader_handle: Mutex::new(Some(reader_handle)),
            worker_handle: Mutex::new(Some(worker_handle)),
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
        self.writer
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .take();
        let mut child = self
            .child
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let _ = unsafe { libc::kill(-(child.id() as i32), libc::SIGTERM) };
        let _ = child.wait();
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
        if queue.closed || queue.messages.len() >= self.queue_capacity {
            drop(queue);
            self.push_internal(ManagedEventKind::DeliveryRejected {
                message_id: message.id.clone(),
                reason: "file Codex pleine".to_string(),
            });
            return Ok(());
        }
        queue.messages.push_back(message.clone());
        wake.notify_one();
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
    fn descriptor(&self) -> ManagedSessionDescriptor {
        ManagedSessionDescriptor {
            transport: "stdio".to_string(),
            mode: PresenceMode::Cli,
            location: None,
        }
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
    }
}

struct Worker {
    queue: Arc<(Mutex<QueueState>, Condvar)>,
    writer: Writer,
    waiters: Waiters,
    next_id: Arc<AtomicU64>,
    observations: Arc<(Mutex<Observations>, Condvar)>,
    alive: Arc<AtomicBool>,
    busy: Arc<AtomicBool>,
    thread_id: String,
    journal: Journal,
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
                while queue.messages.is_empty() && !queue.closed {
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
                json!({ "body": message.body }),
            );
            *worker
                .active_detail
                .lock()
                .unwrap_or_else(|poison| poison.into_inner()) = Some(CodexTurnDetail {
                message_id: message.id.clone(),
                thread_id: worker.thread_id.clone(),
                ..CodexTurnDetail::default()
            });
            let started = SystemTime::now();
            let result = start_turn_with_retry(&worker, &message);
            let turn_started = result.is_ok();
            let event = match result {
                Ok(turn_id) => {
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
                    wait_for_turn(&worker, &message, &turn_id, &cancel, started)
                }
                Err(reason) => ManagedEventKind::DeliveryRejected {
                    message_id: message.id.clone(),
                    reason,
                },
            };
            if turn_started {
                let reasoning = finish_reasoning(&worker.active_detail, &message.id);
                record_or_terminal(
                    &worker.journal,
                    &worker.observations,
                    "reasoning",
                    Some(&message.id),
                    reasoning,
                );
            } else {
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
            if let ManagedEventKind::TurnFinished { terminal, .. } = &event {
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

fn start_turn_with_retry(worker: &Worker, message: &BridgetMessage) -> Result<String, String> {
    for attempt in 0..=SATURATION_RETRIES {
        match request(
            &worker.writer,
            &worker.waiters,
            &worker.next_id,
            "turn/start",
            json!({
                "threadId": worker.thread_id,
                "clientUserMessageId": message.id,
                "input": [{ "type": "text", "text": message.body }],
            }),
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
    loop {
        if !worker.alive.load(Ordering::SeqCst) {
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
        }
        let (lock, wake) = &*worker.observations;
        let mut observed = lock.lock().unwrap_or_else(|poison| poison.into_inner());
        if let Some(terminal) = observed.terminal_by_turn.remove(turn_id) {
            let response = observed
                .response_by_turn
                .remove(turn_id)
                .unwrap_or_default();
            return ManagedEventKind::TurnFinished {
                message: message.clone(),
                response,
                terminal,
            };
        }
        let now = SystemTime::now();
        if now >= deadline {
            return ManagedEventKind::DeliveryRejected {
                message_id: message.id.clone(),
                reason: "échéance Codex dépassée".to_string(),
            };
        }
        let remaining = deadline.duration_since(now).unwrap_or(TURN_POLL);
        let wait = remaining.min(TURN_POLL);
        let (next, _) = wake
            .wait_timeout(observed, wait)
            .unwrap_or_else(|poison| poison.into_inner());
        drop(next);
    }
}

fn clear_active(queue: &Arc<(Mutex<QueueState>, Condvar)>) {
    queue
        .0
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .active = None;
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
            journal.enqueue(event, message_id, payload)
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
            (
                "kind".to_string(),
                Value::String(kind.as_str().to_string()),
            ),
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
    receiver
        .recv_timeout(timeout)
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

fn spawn_reader(stdout: ChildStdout, context: ReaderContext) -> thread::JoinHandle<()> {
    let ReaderContext {
        waiters,
        observations,
        alive,
        journal,
        queue,
        pending_request,
        pinned_model,
        active_detail,
    } = context;
    thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else { break };
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
            if let (Some(method), Some(request_id)) = (method, value.get("id"))
                && let Err(detail) = record_provider_request(
                    &journal,
                    &queue,
                    &pending_request,
                    method,
                    request_id,
                    &value,
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
            match method {
                // Le schéma produit par `codex app-server` 0.149.0 publie
                // `item/agentMessage/delta`. La forme sans préfixe reste
                // tolérée pour les traces antérieures à v2, sans modifier
                // les octets sources que la frontière commune conserve.
                // (A) Retranscription : chaque delta → journal `update` avec
                // le CONTENU exact. `forward_managed_events` ignore Update.
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
                            CodexActKind::Command,
                            delta,
                            None,
                        );
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
            let _ = waiter.send(Err("stdout Codex fermé".to_string()));
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
) -> Result<(), String> {
    // La réception est linéarisée par ce verrou : dès qu'un lecteur l'a pris,
    // le worker terminal attendra que le fait soit mis en file avant de lire
    // la dernière requête pendante.
    let mut pending = pending_request
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let message_id = queue
        .0
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .active
        .as_ref()
        .map(|active| active.message_id.clone());
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
    match code {
        Some(code) => format!("erreur Codex (code {code}; référence {reference})"),
        None => format!("erreur Codex (référence {reference})"),
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
        let drained: Vec<_> = observations
            .0
            .lock()
            .unwrap()
            .events
            .drain(..)
            .collect();
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
            observations.0.lock().unwrap().events.iter().any(|event| matches!(
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
                                    if [ "${BRIDGET_CODEX_EXIT_AFTER_REQUEST:-0}" = 1 ]; then exit 0; fi
                                elif [ "${BRIDGET_CODEX_HOLD_TURN:-0}" != 1 ]; then
                                    if [ "${BRIDGET_CODEX_ACTIVITY:-0}" = 1 ]; then
                                        printf '%s\n' '{"method":"item/reasoning/summaryPartAdded","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"reasoning-1","summaryIndex":0}}'
                                        printf '%s\n' '{"method":"item/reasoning/summaryTextDelta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"reasoning-1","summaryIndex":0,"delta":"Je compare"}}'
                                        printf '%s\n' '{"method":"item/reasoning/summaryTextDelta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"reasoning-1","summaryIndex":0,"delta":" les options."}}'
                                        printf '%s\n' '{"method":"item/reasoning/textDelta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"reasoning-1","contentIndex":0,"delta":"raison brute"}}'
                                        printf '%s\n' '{"method":"item/commandExecution/outputDelta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"command-1","delta":"313 passés"}}'
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
                        *'"method":"turn/interrupt"'*) printf '%s\n' '{"id":6,"result":{}}'; printf '%s\n' '{"method":"turn/completed","params":{"threadId":"thread-native","turn":{"id":"turn-native","status":"interrupted","items":[]}}}' ;;
                    esac
                done"#.to_string(),
            ],
            queue_capacity: 2,
            notify_timeout_secs: 2,
            model: Some("gpt-5.6-terra".to_string()),
        }
    }

    fn message(id: &str) -> BridgetMessage {
        let mut message = BridgetMessage::new("bridget", "codex-native", format!("mission {id}"));
        message.id = id.to_string();
        message.reply = true;
        message
    }

    fn journal_detail_fixture(label: &str, with_activity: bool) -> (Vec<Value>, Vec<Value>) {
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
            (
                "BRIDGET_CODEX_TEXT_MODE".to_string(),
                text_mode.to_string(),
            ),
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

    /// (A) Deltas → update avec TEXTE EXACT. Mutant content:"" doit tuer A seul.
    #[allow(non_snake_case)]
    #[test]

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
                if detail == "trace durable d'une requête fournisseur impossible: journal ACP saturé"
        ));
        assert!(
            failure
                .raw
                .windows(b"item/commandExecution/requestApproval".len())
                .any(|window| window == b"item/commandExecution/requestApproval")
        );

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
        assert!(!journal.contains("SENTINELLE-SECRETE-019"));
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

    fn TEMOIN_A_codex_app_server_retranscrit_les_deltas_en_update() {
        let events = journal_text_fixture("temoin-a", "deltas");
        let updates: Vec<_> = events
            .iter()
            .filter(|event| {
                event["event"] == "update" && event["payload"]["kind"] == "text"
            })
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
            .filter(|event| {
                event["event"] == "update" && event["payload"]["kind"] == "text"
            })
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
            json!({"kind":"command", "text":"313 passés"}),
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
        assert!(
            outbound.iter().all(|frame| !matches!(
                frame["id"].as_str(),
                Some("approval-7" | "approval-8" | "approval-9")
            )),
            "une attente affichable ne doit produire aucune décision JSON-RPC"
        );

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
        assert_eq!(
            raw.raw,
            b"{  \"method\" : \"item/agentMessage/delta\" , \"params\" : { \"threadId\" : \"thread-native\" , \"turnId\" : \"turn-native\" , \"itemId\":\"i\", \"delta\" : \"r\xC3\xA9ponse native\" } }"
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
}
