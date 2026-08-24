//! Pilote natif de `codex app-server`.
//!
//! Le protocole est volontairement porté en `serde_json::Value` : le schéma
//! généré par le binaire Codex installé est l'autorité de compatibilité. La
//! crate publiée `codex-app-server-protocol` suit un autre produit et une
//! autre numérotation ; la lier ici cacherait une incompatibilité possible.

use crate::journal::{JournalFailureSink, JournalLiveFeed, JournalWriter};
use crate::managed_session::{
    ManagedEvent, ManagedEventKind, ManagedEventOrigin, ManagedEventSource, ManagedSession,
    ManagedSessionDescriptor, ManagedTerminal,
};
use crate::protocol::PresenceMode;
use crate::transport::{Transport, TransportError};
use bridget_core::BridgetMessage;
use serde_json::{Value, json};
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
const TURN_POLL: Duration = Duration::from_millis(25);
const SATURATION_RETRIES: u32 = 4;

type Writer = Arc<Mutex<Option<ChildStdin>>>;
type Waiters = Arc<Mutex<HashMap<u64, mpsc::Sender<Result<Value, String>>>>>;
type Journal = Arc<Mutex<Option<JournalWriter>>>;

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

#[derive(Default)]
struct Observations {
    events: VecDeque<ManagedEvent>,
    response_by_turn: HashMap<String, String>,
    terminal_by_turn: HashMap<String, ManagedTerminal>,
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
        let alive = Arc::new(AtomicBool::new(true));
        let next_id = Arc::new(AtomicU64::new(1));
        let reader_handle =
            spawn_reader(stdout, waiters.clone(), observations.clone(), alive.clone());

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
                if initialize.get(required).is_none() {
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
            thread
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
        let queue = Arc::new((
            Mutex::new(QueueState {
                messages: VecDeque::new(),
                active: None,
                closed: false,
            }),
            Condvar::new(),
        ));
        let busy = Arc::new(AtomicBool::new(false));
        let journal = Arc::new(Mutex::new(None));
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
            let _ = record(
                &worker.journal,
                "turn_start",
                Some(&message.id),
                json!({ "body": message.body }),
            );
            let started = SystemTime::now();
            let result = start_turn_with_retry(&worker, &message);
            let event = match result {
                Ok(turn_id) => {
                    push_internal(
                        &worker.observations,
                        ManagedEventKind::PromptDispatched {
                            message_id: message.id.clone(),
                        },
                    );
                    let _ = record(
                        &worker.journal,
                        "prompt_dispatched",
                        Some(&message.id),
                        json!({}),
                    );
                    wait_for_turn(&worker, &message, &turn_id, &cancel, started)
                }
                Err(reason) => ManagedEventKind::DeliveryRejected {
                    message_id: message.id.clone(),
                    reason,
                },
            };
            let is_finished = matches!(event, ManagedEventKind::TurnFinished { .. });
            if is_finished {
                let _ = record(&worker.journal, "turn_end", Some(&message.id), json!({}));
            } else if let ManagedEventKind::DeliveryRejected { reason, .. } = &event {
                let _ = record(
                    &worker.journal,
                    "error",
                    Some(&message.id),
                    json!({ "reason": reason }),
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
    journal
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .as_ref()
        .map_or(Ok(()), |journal| {
            journal.enqueue(event, message_id, payload)
        })
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
    serde_json::from_str::<Value>(reason)
        .ok()
        .and_then(|value| value.get("code").and_then(Value::as_i64))
        == Some(-32001)
}

fn request(
    writer: &Writer,
    waiters: &Waiters,
    next_id: &Arc<AtomicU64>,
    method: &str,
    params: Value,
) -> Result<Value, TransportError> {
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
        .recv_timeout(REQUEST_TIMEOUT)
        .map_err(|_| TransportError::DeliveryFailed(format!("échéance Codex sur {method}")))?
        .map_err(TransportError::DeliveryFailed)
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

fn spawn_reader(
    stdout: ChildStdout,
    waiters: Waiters,
    observations: Arc<(Mutex<Observations>, Condvar)>,
    alive: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
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
                    .map(|error| Err(error.to_string()))
                    .unwrap_or_else(|| Ok(value.get("result").cloned().unwrap_or(Value::Null)));
                let _ = waiter.send(result);
                continue;
            }
            let method = value.get("method").and_then(Value::as_str);
            match method {
                // Le schéma produit par `codex app-server` 0.149.0 publie
                // `item/agentMessage/delta`. La forme sans préfixe reste
                // tolérée pour les traces antérieures à v2, sans modifier
                // les octets sources que la frontière commune conserve.
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
                    }
                    push_source(
                        &observations,
                        raw,
                        ManagedEventKind::Update {
                            detail: "agentMessage/delta Codex".to_string(),
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
                Some(other) => push_source(
                    &observations,
                    raw,
                    ManagedEventKind::Update {
                        detail: format!("notification Codex: {other}"),
                    },
                ),
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
    use std::time::Instant;

    fn root(label: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "bridget-codex-native-{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("horloge système")
                .as_nanos()
        ));
        fs::create_dir_all(&root).expect("racine temporaire");
        root
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
                        *'"method":"thread/start"'*) printf '%s\n' '{"id":2,"result":{"thread":{"id":"thread-native"}}}' ;;
                        *'"method":"turn/start"'*)
                            if [ "$started" != 1 ]; then printf '%s\n' '{"id":3,"error":{"code":-32099,"message":"initialized absent"}}'
                            elif [ "$saturated" = 0 ]; then saturated=1; printf '%s\n' '{"id":3,"error":{"code":-32001,"message":"saturated"}}'
                            else printf '%s\n' '{"id":4,"result":{"turn":{"id":"turn-native"}}}'; if [ "${BRIDGET_CODEX_HOLD_TURN:-0}" != 1 ]; then printf '%s\n' '{  "method" : "item/agentMessage/delta" , "params" : { "threadId" : "thread-native" , "turnId" : "turn-native" , "itemId":"i", "delta" : "réponse native" } }'; printf '%s\n' '{"method":"turn/completed","params":{"threadId":"thread-native","turn":{"id":"turn-native","status":"completed","items":[]}}}'; fi
                            fi ;;
                        *'"method":"turn/interrupt"'*) printf '%s\n' '{"id":5,"result":{}}'; printf '%s\n' '{"method":"turn/completed","params":{"threadId":"thread-native","turn":{"id":"turn-native","status":"interrupted","items":[]}}}' ;;
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
        message
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
                "turn/start",
                "turn/start"
            ]
        );
        assert!(frames.iter().all(|frame| !frame.contains("jsonrpc")));
        transport.stop();
        let _ = fs::remove_dir_all(root);
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
}
