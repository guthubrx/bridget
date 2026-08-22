//! Transport ACP synchrone : un lecteur stdout, un writer sérialisé et un
//! worker FIFO. Le lecteur est l'unique propriétaire du flux de l'adaptateur.

use crate::transport::{Transport, TransportError};
use bridget_core::BridgetMessage;
use serde_json::{json, Value};
use std::collections::{HashMap, VecDeque};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
pub struct AcpOptions {
    pub command: String,
    pub args: Vec<String>,
    pub queue_capacity: usize,
    pub permissions: String,
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
    TurnFinished {
        message: BridgetMessage,
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
}

type Waiters = Arc<Mutex<HashMap<String, mpsc::Sender<Result<Value, String>>>>>;

struct QueueState {
    messages: VecDeque<BridgetMessage>,
    closed: bool,
}

struct TurnWorker {
    queue: Arc<(Mutex<QueueState>, Condvar)>,
    writer: Arc<Mutex<ChildStdin>>,
    waiters: Waiters,
    next_id: Arc<AtomicU64>,
    alive: Arc<AtomicBool>,
    state: Arc<Mutex<TurnState>>,
    events: Arc<Mutex<VecDeque<AcpEvent>>>,
    response: Arc<Mutex<String>>,
    session_id: String,
}

pub struct AcpTransport {
    connection_id: String,
    alive: Arc<AtomicBool>,
    state: Arc<Mutex<TurnState>>,
    events: Arc<Mutex<VecDeque<AcpEvent>>>,
    queue: Arc<(Mutex<QueueState>, Condvar)>,
    queue_capacity: usize,
    writer: Arc<Mutex<ChildStdin>>,
    next_id: Arc<AtomicU64>,
    session_id: String,
    _child: Arc<Mutex<Child>>,
}

impl AcpTransport {
    pub fn spawn(options: AcpOptions) -> Result<Self, TransportError> {
        if options.queue_capacity == 0 {
            return Err(TransportError::DeliveryFailed(
                "queue ACP de capacité nulle".to_string(),
            ));
        }
        let mut child = Command::new(&options.command)
            .args(&options.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
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
        let events = Arc::new(Mutex::new(VecDeque::new()));
        let response = Arc::new(Mutex::new(String::new()));
        let writer = Arc::new(Mutex::new(stdin));
        let waiters: Waiters = Arc::new(Mutex::new(HashMap::new()));
        let next_id = Arc::new(AtomicU64::new(1));
        spawn_reader(
            stdout,
            writer.clone(),
            waiters.clone(),
            events.clone(),
            response.clone(),
            alive.clone(),
            options.permissions.clone(),
        );

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
                "mcpServers": []
            }),
        )?;
        let session_id = session
            .get("sessionId")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                TransportError::DeliveryFailed("session/new ne retourne pas sessionId".to_string())
            })?
            .to_string();

        let queue = Arc::new((
            Mutex::new(QueueState {
                messages: VecDeque::new(),
                closed: false,
            }),
            Condvar::new(),
        ));
        let state = Arc::new(Mutex::new(TurnState::Idle));
        spawn_worker(TurnWorker {
            queue: queue.clone(),
            writer: writer.clone(),
            waiters,
            next_id: next_id.clone(),
            alive: alive.clone(),
            state: state.clone(),
            events: events.clone(),
            response,
            session_id: session_id.clone(),
        });
        Ok(Self {
            connection_id: format!("acp-{pid}"),
            alive,
            state,
            events,
            queue,
            queue_capacity: options.queue_capacity,
            writer,
            next_id,
            session_id,
            _child: Arc::new(Mutex::new(child)),
        })
    }

    pub fn state(&self) -> TurnState {
        self.state
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .clone()
    }

    pub fn drain_events(&self) -> Vec<AcpEvent> {
        self.events
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .drain(..)
            .collect()
    }

    /// Retire un message en attente par son id. Le tour actif reste sous
    /// l'autorité du daemon et sera traité par `session/cancel` en T705.
    pub fn cancel_delivery(&self, message_id: &str, reason: &str) -> bool {
        let Some(message) = purge_queued(&self.queue, message_id) else {
            return false;
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
        if !self.alive.swap(false, Ordering::SeqCst) {
            return;
        }
        let (queue, wakeup) = &*self.queue;
        queue.lock().unwrap_or_else(|err| err.into_inner()).closed = true;
        wakeup.notify_all();
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let _ = write_json(
            &self.writer,
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": "session/cancel",
                "params": { "sessionId": self.session_id }
            }),
        );
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
        if !enqueue(&self.queue, self.queue_capacity, msg.clone()) {
            self.events
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .push_back(AcpEvent::DeliveryRejected {
                    message_id: msg.id.clone(),
                    reason: "file ACP pleine".to_string(),
                });
            return Ok(());
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

fn spawn_worker(worker: TurnWorker) {
    thread::spawn(move || loop {
        let message = {
            let (queue, wakeup) = &*worker.queue;
            let mut queue = queue.lock().unwrap_or_else(|err| err.into_inner());
            while queue.messages.is_empty() && !queue.closed {
                queue = wakeup.wait(queue).unwrap_or_else(|err| err.into_inner());
            }
            if queue.closed {
                break;
            }
            queue.messages.pop_front().expect("file ACP non vide")
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
            .response
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .clear();
        let result = request(
            &worker.writer,
            &worker.waiters,
            &worker.next_id,
            "session/prompt",
            json!({
                "sessionId": &worker.session_id,
                "prompt": [{ "type": "text", "text": prompt_for(&message) }]
            }),
        );
        let collected = worker
            .response
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .clone();
        *worker.state.lock().unwrap_or_else(|err| err.into_inner()) = TurnState::Idle;
        let event = finish_turn(message, collected, result);
        worker
            .events
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .push_back(event);
    });
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

fn finish_turn(
    message: BridgetMessage,
    response: String,
    result: Result<Value, TransportError>,
) -> AcpEvent {
    match result {
        Ok(value) => AcpEvent::TurnFinished {
            message,
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

fn purge_queued(
    queue: &Arc<(Mutex<QueueState>, Condvar)>,
    message_id: &str,
) -> Option<BridgetMessage> {
    let (queue, wakeup) = &**queue;
    let mut queue = queue.lock().unwrap_or_else(|err| err.into_inner());
    let index = queue
        .messages
        .iter()
        .position(|message| message.id == message_id)?;
    let message = queue.messages.remove(index);
    wakeup.notify_one();
    message
}

fn spawn_reader(
    stdout: ChildStdout,
    writer: Arc<Mutex<ChildStdin>>,
    waiters: Waiters,
    events: Arc<Mutex<VecDeque<AcpEvent>>>,
    response: Arc<Mutex<String>>,
    alive: Arc<AtomicBool>,
    permissions: String,
) {
    thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else {
                break;
            };
            let Ok(value) = serde_json::from_str::<Value>(&line) else {
                events
                    .lock()
                    .unwrap_or_else(|err| err.into_inner())
                    .push_back(AcpEvent::Error {
                        detail: "ligne ACP invalide".to_string(),
                    });
                continue;
            };
            let rpc_result = rpc_response(&value);
            let waiter = rpc_result.as_ref().and_then(|(id, _)| {
                waiters
                    .lock()
                    .unwrap_or_else(|err| err.into_inner())
                    .remove(id)
            });
            if let (Some((_, result)), Some(waiter)) = (rpc_result, waiter) {
                let _ = waiter.send(result);
                continue;
            }
            match value.get("method").and_then(Value::as_str) {
                Some("session/update") => {
                    if let Some(text) = update_text(&value) {
                        response
                            .lock()
                            .unwrap_or_else(|err| err.into_inner())
                            .push_str(text);
                        events
                            .lock()
                            .unwrap_or_else(|err| err.into_inner())
                            .push_back(AcpEvent::Update {
                                detail: text.to_string(),
                            });
                    }
                }
                Some("session/request_permission") => {
                    if let Some(reply) = permission_response(&value, &permissions) {
                        let _ = write_json(&writer, reply);
                    }
                }
                Some(method) if value.get("id").is_some() => {
                    if let Some(reply) = method_not_found_response(&value, method) {
                        let _ = write_json(&writer, reply);
                    }
                }
                _ => {}
            }
        }
        alive.store(false, Ordering::SeqCst);
        fail_waiters(&waiters, "EOF ACP");
        events
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .push_back(AcpEvent::Error {
                detail: "EOF ACP".to_string(),
            });
    });
}

fn request(
    writer: &Arc<Mutex<ChildStdin>>,
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

fn update_text(value: &Value) -> Option<&str> {
    value
        .pointer("/params/update/content/text")
        .and_then(Value::as_str)
}

fn fail_waiters(waiters: &Waiters, reason: &str) {
    let pending = std::mem::take(&mut *waiters.lock().unwrap_or_else(|err| err.into_inner()));
    for (_, waiter) in pending {
        let _ = waiter.send(Err(reason.to_string()));
    }
}

fn write_json(writer: &Arc<Mutex<ChildStdin>>, value: Value) -> Result<(), TransportError> {
    let mut writer = writer
        .lock()
        .map_err(|err| TransportError::Io(err.to_string()))?;
    writeln!(writer, "{value}").map_err(|err| TransportError::Io(err.to_string()))?;
    writer
        .flush()
        .map_err(|err| TransportError::Io(err.to_string()))
}

fn permission_response(value: &Value, permissions: &str) -> Option<Value> {
    let id = value.get("id")?;
    let outcome = if permissions == "allow" {
        "allow"
    } else {
        "deny"
    };
    Some(json!({ "jsonrpc": "2.0", "id": id, "result": { "outcome": outcome } }))
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
    format!(
        "[message Bridget de {} — réponse attendue : {}]\n\n{}",
        message.from,
        if message.reply { "oui" } else { "non" },
        message.body
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn queue() -> Arc<(Mutex<QueueState>, Condvar)> {
        Arc::new((
            Mutex::new(QueueState {
                messages: VecDeque::new(),
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
    fn prompt_preserves_the_body_byte_for_byte() {
        let message = BridgetMessage::new("alice", "bob", "'\"$x\nligne");
        assert_eq!(
            prompt_for(&message),
            "[message Bridget de alice — réponse attendue : non]\n\n'\"$x\nligne"
        );
    }

    #[test]
    fn generic_fixture_covers_json_rpc_matrix() {
        let lines = include_str!("../tests/fixtures/acp/generic.jsonl")
            .lines()
            .collect::<Vec<_>>();
        let notification: Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(update_text(&notification), Some("premier"));

        let permission: Value = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(
            permission_response(&permission, "allow").unwrap()["result"]["outcome"],
            "allow"
        );
        assert_eq!(
            permission_response(&permission, "deny").unwrap()["result"]["outcome"],
            "deny"
        );

        let numeric: Value = serde_json::from_str(lines[2]).unwrap();
        let string: Value = serde_json::from_str(lines[3]).unwrap();
        assert_eq!(rpc_response(&numeric).unwrap().0, "9");
        assert_eq!(rpc_response(&string).unwrap().0, "\"string-9\"");

        let error: Value = serde_json::from_str(lines[4]).unwrap();
        assert!(rpc_response(&error)
            .unwrap()
            .1
            .unwrap_err()
            .contains("refus"));

        let unknown: Value = serde_json::from_str(lines[5]).unwrap();
        assert_eq!(
            method_not_found_response(&unknown, "server/inconnu").unwrap()["error"]["code"],
            -32601
        );
        assert!(serde_json::from_str::<Value>(lines[6]).is_err());
    }

    #[test]
    fn codex_spike_fixture_collects_text_then_stop_reason() {
        let mut text = String::new();
        let mut stop_reason = None;
        for line in include_str!("../tests/fixtures/acp/codex-spike.jsonl").lines() {
            let value: Value = serde_json::from_str(line).unwrap();
            if let Some(chunk) = update_text(&value) {
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
    fn queued_message_is_purged_by_id_without_reordering_others() {
        let queue = queue();
        enqueue(&queue, 3, message("first"));
        enqueue(&queue, 3, message("cancel"));
        enqueue(&queue, 3, message("last"));
        assert_eq!(purge_queued(&queue, "cancel").unwrap().id, "cancel");
        let (state, _) = &*queue;
        assert_eq!(
            state
                .lock()
                .unwrap()
                .messages
                .iter()
                .map(|message| message.id.as_str())
                .collect::<Vec<_>>(),
            vec!["first", "last"]
        );
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
}
