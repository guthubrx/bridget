//! Pilote natif Claude Code en JSONL `stream-json`.
//!
//! Les lignes du fournisseur restent brutes jusqu'à la frontière commune. Ce
//! module ne projette ni modèle, ni quota, ni consommation : G5 ne couvre que
//! la session gérée, le journal et son cycle de vie.

use crate::journal::{JournalFailureSink, JournalLiveFeed, JournalWriter};
use crate::managed_session::{
    ManagedEvent, ManagedEventKind, ManagedEventOrigin, ManagedEventSource, ManagedSession,
    ManagedSessionDescriptor, ManagedTerminal,
};
use crate::protocol::PresenceMode;
use crate::transport::{Transport, TransportError};
use bridget_core::BridgetMessage;
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

type Writer = Arc<Mutex<Option<ChildStdin>>>;
type Journal = Arc<Mutex<Option<JournalWriter>>>;

#[derive(Debug, Clone)]
pub struct ClaudeStreamJsonOptions {
    pub command: String,
    /// Arguments de définition, dont l'éventuel `--model` demandé.
    pub args: Vec<String>,
    pub queue_capacity: usize,
    pub notify_timeout_secs: u64,
}

struct ActiveTurn {
    message_id: String,
    completion: mpsc::Sender<ManagedTerminal>,
    response: String,
    /// Nombre d'événements journal `update` texte déjà écrits pour ce tour.
    text_updates: usize,
    /// Issue fournisseur (`end_turn`, …) — absente ⇒ le fil affiche « inconnu ».
    stop_reason: Option<String>,
}

struct QueueState {
    messages: VecDeque<BridgetMessage>,
    active: Option<ActiveTurn>,
    closed: bool,
}

pub struct ClaudeStreamJsonTransport {
    connection_id: String,
    alive: Arc<AtomicBool>,
    shutdown_started: AtomicBool,
    busy: Arc<AtomicBool>,
    queue: Arc<(Mutex<QueueState>, Condvar)>,
    queue_capacity: usize,
    writer: Writer,
    child: Arc<Mutex<Child>>,
    events: Arc<Mutex<VecDeque<ManagedEvent>>>,
    journal: Journal,
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
        ensure_stream_arguments(&mut options.args)?;
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
            // Le pilote et ses enfants forment une seule unité de vie :
            // arrêter seulement le parent laisserait stdout ouvert.
            .process_group(0);
        let mut child = command
            .spawn()
            .map_err(|error| TransportError::Io(format!("impossible de lancer Claude: {error}")))?;
        let pid = child.id();
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| TransportError::Io("stdin Claude absent".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| TransportError::Io("stdout Claude absent".to_string()))?;
        let writer = Arc::new(Mutex::new(Some(stdin)));
        let queue = Arc::new((
            Mutex::new(QueueState {
                messages: VecDeque::new(),
                active: None,
                closed: false,
            }),
            Condvar::new(),
        ));
        let alive = Arc::new(AtomicBool::new(true));
        let busy = Arc::new(AtomicBool::new(false));
        let events = Arc::new(Mutex::new(VecDeque::new()));
        let journal = Arc::new(Mutex::new(None));
        let pinned_model = pinned_model_from_args(&options.args);
        let reader_handle = spawn_reader(
            stdout,
            queue.clone(),
            events.clone(),
            alive.clone(),
            journal.clone(),
            pinned_model,
        );
        let worker_handle = spawn_worker(
            queue.clone(),
            writer.clone(),
            events.clone(),
            journal.clone(),
            alive.clone(),
            busy.clone(),
            Duration::from_secs(options.notify_timeout_secs),
        );
        Ok(Self {
            connection_id: format!("claude-stream-json-{pid}"),
            alive,
            shutdown_started: AtomicBool::new(false),
            busy,
            queue,
            queue_capacity: options.queue_capacity,
            writer,
            child: Arc::new(Mutex::new(child)),
            events,
            journal,
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
        let mut queue = lock.lock().unwrap_or_else(|poison| poison.into_inner());
        queue.closed = true;
        while let Some(message) = queue.messages.pop_front() {
            self.push_internal(ManagedEventKind::DeliveryRejected {
                message_id: message.id,
                reason: "équipier Claude arrêté".to_string(),
            });
        }
        if let Some(active) = queue.active.take() {
            let _ = active.completion.send(ManagedTerminal::Cancelled);
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
        terminate_group(&mut child);
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
        let mut queue = lock.lock().unwrap_or_else(|poison| poison.into_inner());
        if queue.closed || queue.messages.len() >= self.queue_capacity {
            drop(queue);
            self.push_internal(ManagedEventKind::DeliveryRejected {
                message_id: message.id.clone(),
                reason: "file Claude pleine".to_string(),
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

impl ManagedSession for ClaudeStreamJsonTransport {
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
        let events = self.events.clone();
        let failure: JournalFailureSink = Arc::new(move |detail| {
            push_internal(&events, ManagedEventKind::JournalFailed { detail });
        });
        *self
            .journal
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) =
            Some(JournalWriter::start_with_live_feed_and_failure(
                root,
                agent,
                self.connection_id(),
                failure,
                live_feed,
            )?);
        Ok(())
    }

    fn drain_events(&self) -> Vec<ManagedEvent> {
        self.events
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .drain(..)
            .collect()
    }

    fn cancel_delivery(&self, message_id: &str, reason: &str) -> bool {
        let (lock, wake) = &*self.queue;
        let mut queue = lock.lock().unwrap_or_else(|poison| poison.into_inner());
        if queue
            .active
            .as_ref()
            .is_some_and(|active| active.message_id == message_id)
        {
            // Aucun contrôle d'interruption Claude n'est attesté dans le
            // contrat G5. On termine donc honnêtement la livraison en cours,
            // sans inventer une interaction de permission fournisseur.
            let active = queue.active.take().expect("tour Claude actif");
            let _ = active.completion.send(ManagedTerminal::Cancelled);
            wake.notify_one();
            self.alive.store(false, Ordering::SeqCst);
            self.writer
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .take();
            let mut child = self
                .child
                .lock()
                .unwrap_or_else(|poison| poison.into_inner());
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

fn terminate_group(child: &mut Child) {
    // `spawn_with_environment` crée ce groupe avec l'identifiant du fils :
    // le signal ne peut donc toucher ni le wrapper ni un autre équipier.
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
    notify_timeout: Duration,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            let message = {
                let (lock, wake) = &*queue;
                let mut state = lock.lock().unwrap_or_else(|poison| poison.into_inner());
                while state.messages.is_empty() && !state.closed {
                    state = wake
                        .wait(state)
                        .unwrap_or_else(|poison| poison.into_inner());
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
            queue
                .0
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .active = Some(ActiveTurn {
                message_id: message.id.clone(),
                completion,
                response: String::new(),
                text_updates: 0,
                stop_reason: None,
            });
            busy.store(true, Ordering::SeqCst);
            push_internal(
                &events,
                ManagedEventKind::TurnStarted {
                    message_id: message.id.clone(),
                },
            );
            let _ = record(
                &journal,
                "turn_start",
                Some(&message.id),
                json!({ "body": message.body }),
            );
            let terminal = match write_input(&writer, &message) {
                Ok(()) => {
                    push_internal(
                        &events,
                        ManagedEventKind::PromptDispatched {
                            message_id: message.id.clone(),
                        },
                    );
                    let _ = record(
                        &journal,
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
                let mut state = queue.0.lock().unwrap_or_else(|poison| poison.into_inner());
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
                let _ = record(&journal, "turn_end", Some(&message.id), payload);
            }
            push_internal(&events, event);
            busy.store(false, Ordering::SeqCst);
        }
    })
}

fn write_input(writer: &Writer, message: &BridgetMessage) -> Result<(), TransportError> {
    let frame = json!({
        "type": "user",
        "message": {
            "role": "user",
            "content": [{ "type": "text", "text": message.body }],
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

fn spawn_reader(
    stdout: ChildStdout,
    queue: Arc<(Mutex<QueueState>, Condvar)>,
    events: Arc<Mutex<VecDeque<ManagedEvent>>>,
    alive: Arc<AtomicBool>,
    journal: Journal,
    pinned_model: Option<String>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
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
            let kind = value
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or("inconnu");
            // (A) Retranscription : chaque delta texte → journal `update`.
            // Sans cela le fil ne voit que « a travaillé Ns ».
            if let Some(delta) = value.pointer("/event/delta/text").and_then(Value::as_str) {
                let message_id = {
                    let mut state = queue.0.lock().unwrap_or_else(|poison| poison.into_inner());
                    if let Some(active) = state.active.as_mut() {
                        active.response.push_str(delta);
                        active.text_updates = active.text_updates.saturating_add(1);
                        Some(active.message_id.clone())
                    } else {
                        None
                    }
                };
                if let Some(message_id) = message_id {
                    let _ = record(
                        &journal,
                        "update",
                        Some(&message_id),
                        json!({ "kind": "text", "content": delta }),
                    );
                }
            }
            // (B) Repli : message `assistant` complet si aucun delta n'a été
            // journalisé (stdout sans include-partial, ou forme agrégée seule).
            if kind == "assistant" {
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
                if !assistant_text.is_empty() {
                    let message_id = {
                        let mut state =
                            queue.0.lock().unwrap_or_else(|poison| poison.into_inner());
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
                        let _ = record(
                            &journal,
                            "update",
                            Some(&message_id),
                            json!({ "kind": "text", "content": assistant_text }),
                        );
                    }
                }
            }
            if kind == "result" {
                let terminal = if value.get("is_error").and_then(Value::as_bool) == Some(false)
                    && value.get("terminal_reason").and_then(Value::as_str) == Some("completed")
                {
                    ManagedTerminal::Completed
                } else {
                    ManagedTerminal::Failed {
                        detail: value
                            .get("terminal_reason")
                            .and_then(Value::as_str)
                            .unwrap_or("terminal Claude inconnu")
                            .to_string(),
                    }
                };
                // (C) Issue attestée : `terminal_reason` du résultat Claude
                // (ex. "completed"), pas le champ stop_reason fournisseur
                // (ex. "end_turn") — attach lit payload.stop_reason.
                let stop_reason = value
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
                    });
                let pending_result_update = {
                    let mut state = queue.0.lock().unwrap_or_else(|poison| poison.into_inner());
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
                    let _ = record(
                        &journal,
                        "update",
                        Some(&message_id),
                        json!({ "kind": "text", "content": result }),
                    );
                }
                let mut state = queue.0.lock().unwrap_or_else(|poison| poison.into_inner());
                if let Some(active) = state.active.as_mut() {
                    let _ = active.completion.send(terminal);
                }
            }
            let managed_event = if kind == "rate_limit_event" {
                rate_limit_event(&value).unwrap_or_else(|| ManagedEventKind::Update {
                    detail: "événement Claude rate_limit_event incomplet".to_string(),
                })
            } else if let Some(served) = served_model_from_claude(&value) {
                maybe_record_mismatch(&journal, pinned_model.as_deref(), &served);
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
        let mut state = queue.0.lock().unwrap_or_else(|poison| poison.into_inner());
        if let Some(active) = state.active.take() {
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

fn maybe_record_mismatch(journal: &Journal, pinned: Option<&str>, served: &str) {
    let Some(pinned) = pinned.filter(|pinned| *pinned != served) else {
        return;
    };
    let _ = record(
        journal,
        "model_mismatch",
        None,
        json!({ "pinned": pinned, "served": served }),
    );
}

fn rate_limit_event(value: &Value) -> Option<ManagedEventKind> {
    let info = value.get("rate_limit_info")?;
    let window = info.get("rateLimitType")?.as_str()?.trim();
    let status = info.get("status")?.as_str()?.trim();
    if window.is_empty() || status.is_empty() {
        return None;
    }
    Some(ManagedEventKind::RateLimitObserved {
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

fn push_internal(events: &Arc<Mutex<VecDeque<ManagedEvent>>>, kind: ManagedEventKind) {
    let raw = serde_json::to_vec(&json!({ "kind": "claude_internal" }))
        .expect("fait interne Claude sérialisable");
    events
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .push_back(ManagedEvent::internal(
            ManagedEventSource::ClaudeStreamJson,
            raw,
            kind,
        ));
}

fn push_source(events: &Arc<Mutex<VecDeque<ManagedEvent>>>, raw: Vec<u8>, kind: ManagedEventKind) {
    events
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .push_back(ManagedEvent {
            source: ManagedEventSource::ClaudeStreamJson,
            origin: ManagedEventOrigin::SourceLine,
            raw,
            kind,
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
    use std::time::Instant;

    static FIXTURE_SEQ: AtomicU64 = AtomicU64::new(0);

    fn options() -> ClaudeStreamJsonOptions {
        ClaudeStreamJsonOptions {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                concat!(
                    "while IFS= read -r line; do ",
                    "printf '%s\\n' '{\"type\":\"system\",\"subtype\":\"status\",\"status\":\"requesting\"}'; ",
                    "printf '%s\\n' '{\"type\":\"stream_event\",\"event\":{\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"réponse Claude\"}}}'; ",
                    "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",\"result\":\"réponse Claude\"}'; ",
                    "done"
                )
                .to_string(),
            ],
            queue_capacity: 2,
            notify_timeout_secs: 2,
        }
    }

    fn message(id: &str) -> BridgetMessage {
        let mut message = BridgetMessage::new("bridget", "claude", "mission");
        message.id = id.to_string();
        message.reply = true;
        message
    }

    #[test]
    fn session_native_preserve_les_lignes_et_termine_le_tour() {
        let root = std::env::temp_dir().join(format!(
            "bridget-claude-native-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let mut transport = ClaudeStreamJsonTransport::spawn(options()).unwrap();
        transport
            .activate_journal(&root, "claude-native", None)
            .unwrap();
        transport.deliver(&message("claude-1")).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events.iter().any(|event| {
                matches!(
                    event.kind,
                    ManagedEventKind::TurnFinished {
                        ref response,
                        terminal: ManagedTerminal::Completed,
                        ..
                    } if response == "réponse Claude"
                )
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(events.iter().any(|event| {
            matches!(event.kind, ManagedEventKind::PromptDispatched { ref message_id } if message_id == "claude-1")
        }));
        assert!(events.iter().any(|event| {
            event.source == ManagedEventSource::ClaudeStreamJson
                && event.origin == ManagedEventOrigin::SourceLine
                && event.raw.starts_with(br#"{"type":"stream_event""#)
        }));
        transport.stop();
        let _ = fs::remove_dir_all(root);
    }

    /// (A) Témoin retranscription : deltas → `update` journal avec TEXTE EXACT.
    /// Mutant : retirer `record(..., "update", ...)` sur `/event/delta/text`,
    /// ou vider `content` (`content:""`) — A doit mourir (présence ≠ propriété).
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_A_claude_stream_json_retranscrit_les_deltas_en_update() {
        let events = journal_fixture_deltas();
        let updates: Vec<_> = events
            .iter()
            .filter(|event| event["event"] == "update")
            .collect();
        assert!(updates.iter().all(|event| event["payload"]["kind"] == "text"));
        let contents: Vec<&str> = updates
            .iter()
            .filter_map(|event| event["payload"]["content"].as_str())
            .collect();
        // Contenu en dur (comme B) : cardinal+kind seuls laissent passer content:"".
        assert_eq!(
            contents,
            vec!["BONJOUR ", "JE SUIS VIVANT"],
            "deux deltas exacts, pas de troisième update issu du result: {contents:?} via {events:?}"
        );
        assert!(
            updates
                .iter()
                .all(|event| event["message_id"] == "claude-abc-1"),
            "même message_id que le tour, reçu {updates:?}"
        );
    }

    /// (B) Témoin M1 : flux result-only — le TEXTE FINAL EXACT dans le fil,
    /// même message_id. Mutant : retirer le repli result→record(update)
    /// (text_updates==0) → left=[] / right=["REPONSE FINALE"].
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_B_claude_stream_json_reponse_finale_result_only_dans_le_fil() {
        let events = journal_fixture_result_only();
        let updates: Vec<_> = events
            .iter()
            .filter(|event| event["event"] == "update")
            .collect();
        let contents: Vec<&str> = updates
            .iter()
            .filter_map(|event| event["payload"]["content"].as_str())
            .collect();
        assert_eq!(
            contents,
            vec!["REPONSE FINALE"],
            "repli result-only → update exact, reçu {contents:?} via {events:?}"
        );
        assert!(
            updates
                .iter()
                .all(|event| event["message_id"] == "claude-abc-1"),
            "même message_id que le tour, reçu {updates:?}"
        );
    }

    /// (C) Témoin M2 : turn_end.payload.stop_reason = issue attestée
    /// (`completed` = terminal_reason fournisseur). Mutant : turn_end {} →
    /// attach affiche « inconnu » (left=Null / right="completed").
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_C_claude_stream_json_turn_end_porte_stop_reason_completed() {
        let events = journal_fixture_result_only();
        let turn_end = events
            .iter()
            .find(|event| event["event"] == "turn_end")
            .expect("turn_end attendu");
        assert_eq!(
            turn_end["payload"]["stop_reason"].as_str(),
            Some("completed"),
            "stop_reason manquant → attach affiche « inconnu », payload={}",
            turn_end["payload"]
        );
        assert_eq!(
            turn_end["payload"]["routed_to"].as_str(),
            Some("bridget"),
            "reply=true doit porter routed_to, payload={}",
            turn_end["payload"]
        );
    }

    fn journal_fixture_deltas() -> Vec<Value> {
        journal_fixture(concat!(
            "while IFS= read -r line; do ",
            "printf '%s\\n' '{\"type\":\"stream_event\",\"event\":{\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"BONJOUR \"}}}'; ",
            "printf '%s\\n' '{\"type\":\"stream_event\",\"event\":{\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"JE SUIS VIVANT\"}}}'; ",
            "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",\"stop_reason\":\"end_turn\",\"result\":\"BONJOUR JE SUIS VIVANT\"}'; ",
            "done"
        ))
    }

    fn journal_fixture_result_only() -> Vec<Value> {
        journal_fixture(concat!(
            "while IFS= read -r line; do ",
            "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",\"stop_reason\":\"end_turn\",\"result\":\"REPONSE FINALE\"}'; ",
            "done"
        ))
    }

    fn journal_fixture(provider_script: &str) -> Vec<Value> {
        // Compteur atomique : sous parallelisme, SystemTime::nanos peut
        // coincider et un remove_dir_all efface le journal d'un autre témoin.
        let root = std::env::temp_dir().join(format!(
            "bridget-claude-abc-{}-{}",
            std::process::id(),
            FIXTURE_SEQ.fetch_add(1, AtomicOrdering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let mut opts = options();
        opts.args[1] = provider_script.to_string();
        let mut transport = ClaudeStreamJsonTransport::spawn(opts).unwrap();
        transport
            .activate_journal(&root, "claude-abc", None)
            .unwrap();
        transport.deliver(&message("claude-abc-1")).unwrap();
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
        assert!(finished, "le tour Claude n'a pas terminé");
        transport.stop();
        let journal_path = fs::read_dir(root.join("claude-abc"))
            .expect("répertoire journal")
            .next()
            .expect("fichier journal")
            .expect("entrée journal")
            .path();
        let events = crate::journal::valid_events(&journal_path);
        let _ = fs::remove_dir_all(&root);
        events
    }

    #[test]
    fn formats_fournisseur_imposes_ne_peuvent_pas_etre_ecrases() {
        let mut args = vec!["--output-format".to_string(), "json".to_string()];
        assert!(ensure_stream_arguments(&mut args).is_err());
    }

    #[test]
    fn annulation_active_termine_et_recolte_le_processus_claude() {
        let mut slow = options();
        slow.args[1] = "while IFS= read -r line; do sleep 10; done".to_string();
        let mut transport = ClaudeStreamJsonTransport::spawn(slow).unwrap();
        transport.deliver(&message("claude-cancel")).unwrap();
        let deadline = Instant::now() + Duration::from_secs(1);
        while !transport.is_busy() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(transport.is_busy());
        assert!(transport.cancel_delivery("claude-cancel", "annulé par le daemon"));

        let deadline = Instant::now() + Duration::from_secs(1);
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
        assert!(events.iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::TurnFinished {
                    terminal: ManagedTerminal::Cancelled,
                    ..
                }
            )
        }));
        assert!(!transport.is_alive());
        transport.stop();
    }

    #[test]
    fn terminal_fournisseur_en_erreur_refuse_la_livraison() {
        let mut failing = options();
        failing.args[1] = concat!(
            "while IFS= read -r line; do ",
            "printf '%s\\n' '{\"type\":\"result\",\"is_error\":true,",
            "\"terminal_reason\":\"api_error\",\"result\":\"authentification refusée\"}'; ",
            "done"
        )
        .to_string();
        let mut transport = ClaudeStreamJsonTransport::spawn(failing).unwrap();
        transport.deliver(&message("claude-reject")).unwrap();
        let deadline = Instant::now() + Duration::from_secs(1);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events.iter().any(|event| {
                matches!(
                    event.kind,
                    ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
                        if message_id == "claude-reject" && reason == "api_error"
                )
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(events.iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
                    if message_id == "claude-reject" && reason == "api_error"
            )
        }));
        transport.stop();
    }

    #[test]
    fn rate_limit_event_reel_devient_un_fait_sans_inventer_de_retour() {
        let event: Value = serde_json::from_str(
            r#"{"type":"rate_limit_event","rate_limit_info":{"status":"rejected","resetsAt":1787572200,"rateLimitType":"five_hour"}}"#,
        )
        .unwrap();
        assert!(matches!(
            rate_limit_event(&event),
            Some(ManagedEventKind::RateLimitObserved {
                ref window,
                ref status,
                resets_at: Some(1_787_572_200),
                used_percent: None,
            }) if window == "five_hour" && status == "rejected"
        ));

        let no_reset: Value = serde_json::from_str(
            r#"{"type":"rate_limit_event","rate_limit_info":{"status":"allowed","rateLimitType":"five_hour"}}"#,
        )
        .unwrap();
        assert!(matches!(
            rate_limit_event(&no_reset),
            Some(ManagedEventKind::RateLimitObserved {
                resets_at: None,
                used_percent: None,
                ..
            })
        ));
    }

    #[test]
    fn init_annonce_un_modele_distinct_et_le_journalise() {
        let event: Value =
            serde_json::from_str(r#"{"type":"system","subtype":"init","model":"claude-opus-4-6"}"#)
                .unwrap();
        assert_eq!(
            served_model_from_claude(&event).as_deref(),
            Some("claude-opus-4-6")
        );

        let mut mismatch = options();
        mismatch.args = vec![
            "-c".to_string(),
            concat!(
                "while IFS= read -r line; do ",
                "printf '%s\\n' '{\"type\":\"system\",\"subtype\":\"init\",\"model\":\"claude-opus-4-6\"}'; ",
                "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",\"result\":\"ok\"}'; ",
                "done"
            )
            .to_string(),
            "--model".to_string(),
            "claude-opus-5".to_string(),
        ];
        let root = std::env::temp_dir().join(format!(
            "bridget-claude-mismatch-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let mut transport = ClaudeStreamJsonTransport::spawn(mismatch).unwrap();
        transport
            .activate_journal(&root, "claude-mismatch", None)
            .unwrap();
        transport.deliver(&message("claude-gap")).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events.iter().any(|event| {
                matches!(
                    event.kind,
                    ManagedEventKind::ModelObserved { ref model } if model == "claude-opus-4-6"
                )
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(events.iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::ModelObserved { ref model } if model == "claude-opus-4-6"
            )
        }));
        transport.stop();
        let contents = fs::read_dir(root.join("claude-mismatch"))
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "jsonl"))
            .map(|entry| fs::read_to_string(entry.path()).unwrap())
            .collect::<String>();
        assert!(
            contents.contains("\"event\":\"model_mismatch\""),
            "journal sans écart: {contents}"
        );
        assert!(contents.contains("claude-opus-5"));
        assert!(contents.contains("claude-opus-4-6"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn flux_muet_ou_modele_identique_ne_produit_aucun_ecart() {
        assert!(
            served_model_from_claude(
                &serde_json::from_str(
                    r#"{"type":"system","subtype":"status","status":"requesting"}"#
                )
                .unwrap()
            )
            .is_none()
        );
        assert!(
            served_model_from_claude(
                &serde_json::from_str(r#"{"type":"system","subtype":"init"}"#).unwrap()
            )
            .is_none()
        );

        let mut matching = options();
        matching.args = vec![
            "-c".to_string(),
            concat!(
                "while IFS= read -r line; do ",
                "printf '%s\\n' '{\"type\":\"system\",\"subtype\":\"init\",\"model\":\"claude-opus-5\"}'; ",
                "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",\"result\":\"ok\"}'; ",
                "done"
            )
            .to_string(),
            "--model".to_string(),
            "claude-opus-5".to_string(),
        ];
        let root = std::env::temp_dir().join(format!(
            "bridget-claude-match-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let mut transport = ClaudeStreamJsonTransport::spawn(matching).unwrap();
        transport
            .activate_journal(&root, "claude-match", None)
            .unwrap();
        transport.deliver(&message("claude-ok")).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events
                .iter()
                .any(|event| matches!(event.kind, ManagedEventKind::ModelObserved { .. }))
            {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(events.iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::ModelObserved { ref model } if model == "claude-opus-5"
            )
        }));
        transport.stop();
        let contents = fs::read_dir(root.join("claude-match"))
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "jsonl"))
            .map(|entry| fs::read_to_string(entry.path()).unwrap())
            .collect::<String>();
        assert!(
            !contents.contains("model_mismatch"),
            "écart inventé: {contents}"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn usage_result_reel_devient_un_fait_complet() {
        let event: Value = serde_json::from_str(
            r#"{"type":"result","usage":{"input_tokens":2,"output_tokens":175,"cache_creation_input_tokens":40804,"cache_read_input_tokens":13907}}"#,
        )
        .unwrap();
        assert!(matches!(
            usage_event(&event),
            Some(ManagedEventKind::UsageObserved {
                input_tokens: 2,
                output_tokens: 175,
                cache_creation_input_tokens: 40_804,
                cache_read_input_tokens: 13_907,
            })
        ));
    }

    #[test]
    fn usage_incomplet_ne_devient_pas_un_zero_invente() {
        let event: Value = serde_json::from_str(
            r#"{"type":"result","usage":{"input_tokens":2,"output_tokens":10}}"#,
        )
        .unwrap();
        assert!(usage_event(&event).is_none());
    }

    #[test]
    fn usage_assistant_puis_result_ne_compte_qu_un_echantillon() {
        let assistant: Value = serde_json::from_str(
            r#"{"type":"assistant","message":{"usage":{"input_tokens":2,"output_tokens":175,"cache_creation_input_tokens":40804,"cache_read_input_tokens":13907}}}"#,
        )
        .unwrap();
        let result: Value = serde_json::from_str(
            r#"{"type":"result","usage":{"input_tokens":2,"output_tokens":175,"cache_creation_input_tokens":40804,"cache_read_input_tokens":13907}}"#,
        )
        .unwrap();
        let counted = [assistant, result]
            .iter()
            .filter(|event| usage_event(event).is_some())
            .count();
        assert_eq!(
            counted, 1,
            "un tour assistant+result ne doit poser qu'un échantillon"
        );
        assert!(usage_event(&serde_json::from_str(
            r#"{"type":"assistant","message":{"usage":{"input_tokens":2,"output_tokens":175,"cache_creation_input_tokens":40804,"cache_read_input_tokens":13907}}}"#,
        )
        .unwrap())
        .is_none());
    }
}
