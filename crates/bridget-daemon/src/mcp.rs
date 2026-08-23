//! Façade MCP stdio. Le protocole daemon reste le seul transport métier.

use bridget_core::BridgetMessage;
use bridget_transport::protocol::{
    CLIENT_CONTRACT_VERSION, ClientCapability, ConnectionRole, IdempotencyIssue, LedgerScope,
    PresenceMode, decode, encode,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use serde_json::{Value, json};
use std::collections::HashSet;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::FromRawFd;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const PROTOCOL_VERSION: &str = "2025-06-18";
const DAEMON_BUDGET: Duration = Duration::from_secs(10);
const MAX_IN_FLIGHT_TOOL_CALLS: usize = 8;
static NEXT_MESSAGE_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_CONNECTION_NAME: AtomicU64 = AtomicU64::new(1);

#[derive(Default)]
struct Session {
    initialize_seen: bool,
    initialized: bool,
}

/// Exécute le serveur MCP avec un lecteur unique et un writer unique.
///
/// L'ownership exclusif du `BufWriter` sérialise les sorties JSON-RPC : aucun
/// diagnostic ne peut rejoindre stdout, qui est réservé aux réponses MCP.
pub fn serve<R: BufRead, W: Write + Send>(mut input: R, output: W) -> io::Result<()> {
    serve_with(
        &mut input,
        output,
        &crate::mcp_identity::resolve_current_identity,
        &execute_tool,
    )
}

fn serve_with<R, W, I, E>(
    mut input: R,
    output: W,
    resolve_identity: &I,
    execute: &E,
) -> io::Result<()>
where
    R: BufRead,
    W: Write + Send,
    I: Fn() -> Result<crate::mcp_identity::ResolvedIdentity, crate::mcp_identity::IdentityError>
        + Sync,
    E: Fn(&crate::mcp_identity::ResolvedIdentity, &str, &Value) -> Result<Value, ToolError> + Sync,
{
    let output = Arc::new(Mutex::new(BufWriter::new(output)));
    let mut session = Session::default();
    let mut line = String::new();
    let in_flight = Arc::new(Mutex::new(HashSet::new()));
    let cancelled = Arc::new(Mutex::new(HashSet::new()));
    let active = Arc::new(AtomicUsize::new(0));

    std::thread::scope(|scope| -> io::Result<()> {
        loop {
            line.clear();
            if input.read_line(&mut line)? == 0 {
                break;
            }
            if line.trim().is_empty() {
                continue;
            }
            let request = match serde_json::from_str::<Value>(line.trim_end()) {
                Ok(request) => request,
                Err(_) => {
                    write_shared_response(&output, &error(Value::Null, -32700, "JSON malformé"))?;
                    continue;
                }
            };
            if let Some(cancelled_id) = cancellation_id(&request) {
                let is_active = in_flight
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .contains(&cancelled_id);
                if is_active {
                    cancelled
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .insert(cancelled_id);
                }
                continue;
            }
            let call_id = asynchronous_tool_call_id(&request, &session);
            if let Some(call_id) = call_id {
                if !try_reserve_tool_call(&active) {
                    write_shared_response(
                        &output,
                        &result(
                            request.get("id").cloned().unwrap_or(Value::Null),
                            technical_result("busy", "serveur MCP saturé : réessaie plus tard"),
                        ),
                    )?;
                    continue;
                }
                in_flight
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .insert(call_id.clone());
                let output = Arc::clone(&output);
                let in_flight = Arc::clone(&in_flight);
                let cancelled = Arc::clone(&cancelled);
                let active = Arc::clone(&active);
                scope.spawn(move || {
                    let mut tool_session = Session {
                        initialize_seen: true,
                        initialized: true,
                    };
                    let response = dispatch_with_executor(
                        &request,
                        &mut tool_session,
                        resolve_identity,
                        execute,
                    );
                    let cancelled_call = cancelled
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .remove(&call_id);
                    in_flight
                        .lock()
                        .unwrap_or_else(|error| error.into_inner())
                        .remove(&call_id);
                    active.fetch_sub(1, Ordering::AcqRel);
                    if !cancelled_call && let Some(response) = response {
                        let _ = write_shared_response(&output, &response);
                    }
                });
                continue;
            }
            if let Some(response) =
                dispatch_with_executor(&request, &mut session, resolve_identity, execute)
            {
                write_shared_response(&output, &response)?;
            }
        }
        Ok(())
    })?;
    output
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .flush()
}

/// Lance la façade MCP depuis la sous-commande `bridget mcp`.
pub fn run_stdio() -> io::Result<()> {
    serve(BufReader::new(io::stdin().lock()), io::stdout())
}

fn write_response(output: &mut impl Write, response: &Value) -> io::Result<()> {
    serde_json::to_writer(&mut *output, response).map_err(io::Error::other)?;
    output.write_all(b"\n")?;
    output.flush()
}

fn write_shared_response(
    output: &Arc<Mutex<BufWriter<impl Write>>>,
    response: &Value,
) -> io::Result<()> {
    let mut writer = output.lock().unwrap_or_else(|error| error.into_inner());
    write_response(&mut *writer, response)
}

fn cancellation_id(request: &Value) -> Option<String> {
    (request.get("method").and_then(Value::as_str) == Some("notifications/cancelled"))
        .then(|| {
            request
                .get("params")?
                .get("requestId")
                .map(Value::to_string)
        })
        .flatten()
}

fn asynchronous_tool_call_id(request: &Value, session: &Session) -> Option<String> {
    (session.initialized && request.get("method").and_then(Value::as_str) == Some("tools/call"))
        .then(|| request.get("id").map(Value::to_string))
        .flatten()
}

fn try_reserve_tool_call(active: &AtomicUsize) -> bool {
    let mut observed = active.load(Ordering::Acquire);
    loop {
        if observed >= MAX_IN_FLIGHT_TOOL_CALLS {
            return false;
        }
        match active.compare_exchange_weak(
            observed,
            observed + 1,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => return true,
            Err(current) => observed = current,
        }
    }
}

#[cfg(test)]
fn dispatch_with_identity(
    request: &Value,
    session: &mut Session,
    resolve_identity: &impl Fn() -> Result<
        crate::mcp_identity::ResolvedIdentity,
        crate::mcp_identity::IdentityError,
    >,
) -> Option<Value> {
    dispatch_with_executor(request, session, resolve_identity, &execute_tool)
}

fn dispatch_with_executor(
    request: &Value,
    session: &mut Session,
    resolve_identity: &impl Fn() -> Result<
        crate::mcp_identity::ResolvedIdentity,
        crate::mcp_identity::IdentityError,
    >,
    execute: &impl Fn(&crate::mcp_identity::ResolvedIdentity, &str, &Value) -> Result<Value, ToolError>,
) -> Option<Value> {
    let Some(object) = request.as_object() else {
        return Some(error(Value::Null, -32600, "requête JSON-RPC invalide"));
    };
    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Some(error(Value::Null, -32600, "version JSON-RPC invalide"));
    }
    let Some(method) = object.get("method").and_then(Value::as_str) else {
        return Some(error(Value::Null, -32600, "méthode JSON-RPC absente"));
    };
    let id = match object.get("id") {
        None => None,
        Some(Value::String(_) | Value::Number(_)) => object.get("id").cloned(),
        Some(_) => return Some(error(Value::Null, -32600, "identifiant JSON-RPC invalide")),
    };
    let params = object.get("params");

    match method {
        "initialize" => {
            if !params.is_none_or(|value| value.is_object()) {
                return id.map(|id| error(id, -32602, "paramètres initialize invalides"));
            }
            session.initialize_seen = true;
            id.map(|id| result(id, initialize_result()))
        }
        "notifications/initialized" => {
            if session.initialize_seen {
                session.initialized = true;
            } else {
                eprintln!("bridget mcp: notification initialized reçue avant initialize");
            }
            None
        }
        "notifications/cancelled" => None,
        "tools/list" => {
            if let Some(response) = require_initialized(id.clone(), session) {
                return Some(response);
            }
            if !params.is_none_or(|value| value.is_object()) {
                return id.map(|id| error(id, -32602, "paramètres tools/list invalides"));
            }
            id.map(|id| result(id, json!({ "tools": tools() })))
        }
        "ping" => {
            if let Some(response) = require_initialized(id.clone(), session) {
                return Some(response);
            }
            if !params.is_none_or(|value| value.is_object()) {
                return id.map(|id| error(id, -32602, "paramètres ping invalides"));
            }
            id.map(|id| result(id, json!({})))
        }
        "tools/call" => {
            if let Some(response) = require_initialized(id.clone(), session) {
                return Some(response);
            }
            let Some(params) = params.and_then(Value::as_object) else {
                return id.map(|id| error(id, -32602, "paramètres tools/call invalides"));
            };
            let Some(name) = params.get("name").and_then(Value::as_str) else {
                return id.map(|id| error(id, -32602, "nom d'outil absent"));
            };
            if !tools().iter().any(|tool| tool["name"] == name) {
                return id.map(|id| error(id, -32602, "outil inconnu"));
            }
            let identity = match resolve_identity() {
                Ok(identity) => identity,
                Err(identity_error) => {
                    return id.map(|id| identity_error_result(id, &identity_error));
                }
            };
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            id.map(|id| match execute(&identity, name, &arguments) {
                Ok(payload) => result(id, tool_result(payload)),
                Err(ToolError::InvalidParams(message)) => error(id, -32602, &message),
                Err(ToolError::Technical { code, message }) => {
                    result(id, technical_result(code, &message))
                }
            })
        }
        _ => id.map(|id| error(id, -32601, &format!("méthode inconnue: {method}"))),
    }
}

fn require_initialized(id: Option<Value>, session: &Session) -> Option<Value> {
    (!session.initialized)
        .then(|| id.map(|id| error(id, -32002, "initialisation MCP requise")))
        .flatten()
}

fn result(id: Value, payload: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": payload })
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn identity_error_result(id: Value, identity_error: &crate::mcp_identity::IdentityError) -> Value {
    result(
        id,
        json!({
            "content": [{
                "type": "text",
                "text": format!("{}: {}", identity_error.code(), identity_error.remediation())
            }],
            "isError": true,
            "code": identity_error.code()
        }),
    )
}

#[derive(Debug)]
enum ToolError {
    InvalidParams(String),
    Technical { code: &'static str, message: String },
}

struct DaemonConnection {
    writer: BufWriter<UnixStream>,
    reader: BufReader<UnixStream>,
    deadline: Instant,
}

impl DaemonConnection {
    fn connect(socket: &Path) -> Result<Self, ToolError> {
        let deadline = Instant::now() + DAEMON_BUDGET;
        let stream =
            connect_nonblocking(socket, deadline).map_err(|error| ToolError::Technical {
                code: "daemon_unreachable",
                message: format!("daemon Bridget injoignable : {error}"),
            })?;
        let reader_stream = stream.try_clone().map_err(|error| ToolError::Technical {
            code: "daemon_unreachable",
            message: format!("impossible de dupliquer le socket daemon : {error}"),
        })?;
        Ok(Self {
            writer: BufWriter::new(stream),
            reader: BufReader::new(reader_stream),
            deadline,
        })
    }

    fn exchange(&mut self, command: &WrapperToDaemon) -> Result<DaemonToWrapper, ToolError> {
        self.apply_remaining_timeout("daemon_unreachable")?;
        let json = encode(command).map_err(|error| ToolError::Technical {
            code: "daemon_protocol",
            message: format!("encodage daemon impossible : {error}"),
        })?;
        writeln!(self.writer, "{json}").map_err(|error| ToolError::Technical {
            code: "daemon_unreachable",
            message: format!("écriture daemon impossible : {error}"),
        })?;
        self.writer.flush().map_err(|error| ToolError::Technical {
            code: "daemon_unreachable",
            message: format!("flush daemon impossible : {error}"),
        })?;
        self.read_response("daemon_unreachable")
    }

    fn send_then_wait(&mut self, command: &WrapperToDaemon) -> Result<DaemonToWrapper, ToolError> {
        self.apply_remaining_timeout("daemon_unreachable")?;
        let json = encode(command).map_err(|error| ToolError::Technical {
            code: "daemon_protocol",
            message: format!("encodage daemon impossible : {error}"),
        })?;
        writeln!(self.writer, "{json}").map_err(|error| ToolError::Technical {
            code: "outcome_unknown",
            message: format!("écriture daemon impossible : {error}"),
        })?;
        self.writer.flush().map_err(|error| ToolError::Technical {
            code: "outcome_unknown",
            message: format!("flush daemon impossible : {error}"),
        })?;
        self.read_response("outcome_unknown")
    }

    fn read_response(&mut self, failure_code: &'static str) -> Result<DaemonToWrapper, ToolError> {
        self.apply_remaining_timeout(failure_code)?;
        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .map_err(|error| ToolError::Technical {
                code: failure_code,
                message: format!("réponse daemon indisponible : {error}"),
            })?;
        if line.is_empty() {
            return Err(ToolError::Technical {
                code: failure_code,
                message: "daemon Bridget a fermé la connexion sans réponse".to_string(),
            });
        }
        decode(line.trim_end()).map_err(|error| ToolError::Technical {
            code: failure_code,
            message: format!("réponse daemon invalide : {error}"),
        })
    }

    fn apply_remaining_timeout(&self, code: &'static str) -> Result<(), ToolError> {
        let remaining = self
            .deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| ToolError::Technical {
                code,
                message: "budget total de 10 s dépassé".to_string(),
            })?;
        self.writer
            .get_ref()
            .set_write_timeout(Some(remaining))
            .map_err(|error| ToolError::Technical {
                code,
                message: format!("impossible de borner l'écriture daemon : {error}"),
            })?;
        self.reader
            .get_ref()
            .set_read_timeout(Some(remaining))
            .map_err(|error| ToolError::Technical {
                code,
                message: format!("impossible de borner la lecture daemon : {error}"),
            })
    }
}

fn connect_nonblocking(socket: &Path, deadline: Instant) -> io::Result<UnixStream> {
    if deadline <= Instant::now() {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "budget connexion dépassé",
        ));
    }
    let path = socket.as_os_str().as_bytes();
    let mut address: libc::sockaddr_un = unsafe { std::mem::zeroed() };
    if path.len() >= address.sun_path.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "socket Unix trop long",
        ));
    }
    address.sun_family = libc::AF_UNIX as libc::sa_family_t;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        address.sun_len = (std::mem::size_of::<libc::sa_family_t>() + path.len() + 1) as u8;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(
            path.as_ptr().cast(),
            address.sun_path.as_mut_ptr(),
            path.len(),
        );
    }
    let fd = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_STREAM, 0) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    let close = |fd: libc::c_int| unsafe { libc::close(fd) };
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        close(fd);
        return Err(io::Error::last_os_error());
    }
    let result = unsafe {
        libc::connect(
            fd,
            (&address as *const libc::sockaddr_un).cast(),
            std::mem::size_of::<libc::sockaddr_un>() as libc::socklen_t,
        )
    };
    if result < 0 {
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::EINPROGRESS) {
            close(fd);
            return Err(error);
        }
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "budget connexion dépassé"))?;
        let timeout = remaining.as_millis().min(i32::MAX as u128) as libc::c_int;
        let mut pollfd = libc::pollfd {
            fd,
            events: libc::POLLOUT,
            revents: 0,
        };
        if unsafe { libc::poll(&mut pollfd, 1, timeout) } <= 0 {
            close(fd);
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "connexion daemon expirée",
            ));
        }
        let mut so_error: libc::c_int = 0;
        let mut length = std::mem::size_of::<libc::c_int>() as libc::socklen_t;
        if unsafe {
            libc::getsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_ERROR,
                (&mut so_error as *mut libc::c_int).cast(),
                &mut length,
            )
        } < 0
            || so_error != 0
        {
            close(fd);
            return Err(io::Error::from_raw_os_error(so_error));
        }
    }
    let stream = unsafe { UnixStream::from_raw_fd(fd) };
    stream.set_nonblocking(false)?;
    Ok(stream)
}

fn execute_tool(
    identity: &crate::mcp_identity::ResolvedIdentity,
    name: &str,
    arguments: &Value,
) -> Result<Value, ToolError> {
    let arguments = arguments
        .as_object()
        .ok_or_else(|| ToolError::InvalidParams("arguments d'outil invalides".to_string()))?;
    let socket = crate::daemon::DaemonConfig::default().socket_path;
    execute_tool_at_with_scope(
        &identity.name,
        &identity.instance_id,
        name,
        arguments,
        &socket,
    )
}

#[cfg(test)]
fn execute_tool_at(
    identity: &str,
    name: &str,
    arguments: &serde_json::Map<String, Value>,
    socket: &Path,
) -> Result<Value, ToolError> {
    execute_tool_at_with_scope(identity, "test-instance", name, arguments, socket)
}

fn execute_tool_at_with_scope(
    identity: &str,
    instance_id: &str,
    name: &str,
    arguments: &serde_json::Map<String, Value>,
    socket: &Path,
) -> Result<Value, ToolError> {
    match name {
        "bridget_send" => execute_send(identity, instance_id, arguments, socket),
        "bridget_who" => execute_who(arguments, socket),
        "bridget_ledger" => execute_ledger(identity, arguments, socket),
        _ => Err(ToolError::InvalidParams("outil inconnu".to_string())),
    }
}

fn execute_send(
    identity: &str,
    instance_id: &str,
    arguments: &serde_json::Map<String, Value>,
    socket: &Path,
) -> Result<Value, ToolError> {
    reject_unknown_arguments(
        arguments,
        &[
            "to",
            "body",
            "reply",
            "reply_timeout",
            "in_reply_to",
            "id",
            "issued_at",
        ],
    )?;
    let to = required_non_empty_string(arguments, "to")?;
    let body = required_non_empty_string(arguments, "body")?;
    let reply = optional_bool(arguments, "reply")?.unwrap_or(false);
    let reply_timeout = optional_positive_u64(arguments, "reply_timeout")?;
    let in_reply_to = optional_non_empty_string(arguments, "in_reply_to")?;
    if !reply && reply_timeout.is_some() {
        return Err(ToolError::InvalidParams(
            "reply_timeout est réservé à reply=true".to_string(),
        ));
    }
    let supplied_id = arguments.get("id");
    let supplied_issued_at = arguments.get("issued_at");
    if supplied_id.is_some() != supplied_issued_at.is_some() {
        return Err(ToolError::InvalidParams(
            "id et issued_at doivent être fournis ensemble pour un retry".to_string(),
        ));
    }
    let id = match supplied_id {
        Some(value) => value
            .as_str()
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .ok_or_else(|| {
                ToolError::InvalidParams("id doit être une chaîne non vide".to_string())
            })?,
        None => new_message_id(),
    };
    let issued_at = match supplied_issued_at {
        Some(value) => value.as_i64().filter(|value| *value > 0).ok_or_else(|| {
            ToolError::InvalidParams("issued_at doit être un entier positif".to_string())
        })?,
        None => now_secs(),
    };
    let mut message = BridgetMessage::new(identity, to, body);
    message.id = id.clone();
    message.reply = reply;
    message.reply_timeout = reply_timeout;
    message.in_reply_to = in_reply_to;
    let mut connection = DaemonConnection::connect(socket)?;
    match connection.exchange(&WrapperToDaemon::RoleHandshake {
        role: ConnectionRole::Client,
    })? {
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client,
        } => {}
        other => return unexpected_response(other),
    }
    let issuer_scope = issuer_scope(instance_id);
    match connection.exchange(&WrapperToDaemon::ClientHello {
        contract_version: CLIENT_CONTRACT_VERSION,
        issuer_scope,
        capabilities: vec![ClientCapability::SendIdempotent],
    })? {
        DaemonToWrapper::ClientWelcome { capabilities, .. }
            if capabilities.contains(&ClientCapability::SendIdempotent) => {}
        DaemonToWrapper::ClientRejected { reason } => {
            return Err(ToolError::Technical {
                code: "daemon_protocol",
                message: format!("négociation client refusée : {reason:?}"),
            });
        }
        other => return unexpected_response(other),
    }
    match connection.send_then_wait(&WrapperToDaemon::SendIdempotent {
        message,
        message_id: id.clone(),
        issued_at,
    }) {
        Ok(DaemonToWrapper::IdempotencyResult { issue, .. }) => {
            Ok(send_issue_result(&id, issued_at, issue))
        }
        Ok(other) => unexpected_response(other),
        Err(ToolError::Technical {
            code: "outcome_unknown",
            message,
        }) => Ok(json!({
            "status": "outcome_unknown",
            "id": id,
            "issued_at": issued_at,
            "reason": format!("accusé perdu après transmission — retry possible avec le même id ({message})")
        })),
        Err(error) => Err(error),
    }
}

fn execute_who(
    arguments: &serde_json::Map<String, Value>,
    socket: &Path,
) -> Result<Value, ToolError> {
    reject_unknown_arguments(arguments, &["domain"])?;
    let domain = match arguments.get("domain") {
        Some(value) => Some(
            value
                .as_str()
                .ok_or_else(|| ToolError::InvalidParams("domain doit être une chaîne".to_string()))?
                .to_string(),
        ),
        None => None,
    };
    let mut connection = registered_connection(socket)?;
    match connection.exchange(&WrapperToDaemon::ListAgents)? {
        DaemonToWrapper::AgentList { agents } => Ok(json!({
            "agents": agents.into_iter().filter(|agent| {
                domain.as_deref().is_none_or(|wanted| agent.domain.as_deref() == Some(wanted))
            }).collect::<Vec<_>>()
        })),
        other => unexpected_response(other),
    }
}

fn execute_ledger(
    identity: &str,
    arguments: &serde_json::Map<String, Value>,
    socket: &Path,
) -> Result<Value, ToolError> {
    reject_unknown_arguments(arguments, &["view", "limit"])?;
    let view = arguments
        .get("view")
        .and_then(Value::as_str)
        .unwrap_or("both");
    let scope = match view {
        "messages" => LedgerScope::Messages,
        "requests" => LedgerScope::Requests,
        "both" => LedgerScope::Both,
        _ => {
            return Err(ToolError::InvalidParams(
                "view doit valoir messages, requests ou both".to_string(),
            ));
        }
    };
    let limit = match arguments.get("limit") {
        Some(value) => value
            .as_u64()
            .filter(|value| (1..=200).contains(value))
            .ok_or_else(|| {
                ToolError::InvalidParams("limit doit être compris entre 1 et 200".to_string())
            })? as u16,
        None => 20,
    };
    let mut connection = registered_connection(socket)?;
    let mut messages = Vec::new();
    let mut requests = Vec::new();
    if matches!(scope, LedgerScope::Messages | LedgerScope::Both) {
        match connection.exchange(&WrapperToDaemon::LedgerProjection {
            scope: LedgerScope::Messages,
            limit,
        })? {
            DaemonToWrapper::LedgerProjection {
                messages: result, ..
            } => messages = result,
            other => return unexpected_response(other),
        }
    }
    if matches!(scope, LedgerScope::Requests | LedgerScope::Both) {
        match connection.exchange(&WrapperToDaemon::ListRequests {
            sender: identity.to_string(),
            limit,
        })? {
            DaemonToWrapper::RequestList { requests: result } => requests = result,
            other => return unexpected_response(other),
        }
    }
    Ok(json!({
        "messages": messages.into_iter().map(ledger_message_dto).collect::<Vec<_>>(),
        "requests": requests.into_iter().map(request_dto).collect::<Vec<_>>(),
    }))
}

fn registered_connection(socket: &Path) -> Result<DaemonConnection, ToolError> {
    let mut connection = DaemonConnection::connect(socket)?;
    let registration = WrapperToDaemon::Register {
        agent_type: "mcp".to_string(),
        name: Some(ephemeral_connection_name()),
        host: None,
        transport: None,
        mode: Some(PresenceMode::Cli),
        location: None,
        os: None,
        instance_id: None,
        domain: None,
        turn_in_progress: false,
    };
    match connection.exchange(&registration)? {
        DaemonToWrapper::Registered { .. } => Ok(connection),
        other => unexpected_response(other),
    }
}

fn send_issue_result(id: &str, issued_at: i64, issue: IdempotencyIssue) -> Value {
    match issue {
        IdempotencyIssue::Accepted { .. } => {
            json!({ "status": "accepted", "id": id, "issued_at": issued_at, "hops": 4 })
        }
        IdempotencyIssue::Rejected {
            category, reason, ..
        } => {
            json!({ "status": public_refusal_category(&category), "id": id, "issued_at": issued_at, "reason": reason })
        }
        IdempotencyIssue::OutcomeUnknown { .. } => json!({
            "status": "outcome_unknown",
            "id": id,
            "issued_at": issued_at,
            "reason": "accusé perdu après transmission — retry possible avec le même id"
        }),
        IdempotencyIssue::EnvelopeMismatch => json!({
            "status": "envelope_mismatch",
            "id": id,
            "issued_at": issued_at,
            "reason": "enveloppe différente pour le même id"
        }),
        IdempotencyIssue::IdempotencyExpired => json!({
            "status": "idempotency_expired",
            "id": id,
            "issued_at": issued_at,
            "reason": "clé d'idempotence expirée"
        }),
        IdempotencyIssue::InvalidIssuedAt => json!({
            "status": "invalid_issued_at",
            "id": id,
            "issued_at": issued_at,
            "reason": "horodatage d'émission invalide"
        }),
    }
}

fn public_refusal_category(category: &str) -> &str {
    match category {
        "dnd" | "circuit_breaker" | "hops_exhausted" | "queue_full" => category,
        "duplicate_content" | "quarantined" => "duplicate",
        "routing" | "recipient_unavailable" => "unknown_recipient",
        "reply_sender_unavailable" => "reply_requires_agent",
        _ => category,
    }
}

fn tool_result(payload: Value) -> Value {
    let text = serde_json::to_string(&payload).expect("un résultat MCP est toujours sérialisable");
    json!({
        "content": [{ "type": "text", "text": text }],
        "structuredContent": payload
    })
}

fn technical_result(code: &str, message: &str) -> Value {
    json!({
        "content": [{ "type": "text", "text": message }],
        "isError": true,
        "code": code
    })
}

fn unexpected_response<T>(response: DaemonToWrapper) -> Result<T, ToolError> {
    Err(ToolError::Technical {
        code: "daemon_protocol",
        message: format!("réponse daemon inattendue : {response:?}"),
    })
}

fn reject_unknown_arguments(
    arguments: &serde_json::Map<String, Value>,
    allowed: &[&str],
) -> Result<(), ToolError> {
    if let Some(unknown) = arguments
        .keys()
        .find(|key| !allowed.contains(&key.as_str()))
    {
        return Err(ToolError::InvalidParams(format!(
            "argument inconnu : {unknown}"
        )));
    }
    Ok(())
}

fn required_non_empty_string(
    arguments: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<String, ToolError> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| ToolError::InvalidParams(format!("{key} doit être une chaîne non vide")))
}

fn optional_non_empty_string(
    arguments: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<String>, ToolError> {
    arguments
        .get(key)
        .map(|value| {
            value
                .as_str()
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .ok_or_else(|| {
                    ToolError::InvalidParams(format!("{key} doit être une chaîne non vide"))
                })
        })
        .transpose()
}

fn optional_bool(
    arguments: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<bool>, ToolError> {
    arguments
        .get(key)
        .map(|value| {
            value
                .as_bool()
                .ok_or_else(|| ToolError::InvalidParams(format!("{key} doit être booléen")))
        })
        .transpose()
}

fn optional_positive_u64(
    arguments: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<u64>, ToolError> {
    arguments
        .get(key)
        .map(|value| {
            value.as_u64().filter(|value| *value > 0).ok_or_else(|| {
                ToolError::InvalidParams(format!("{key} doit être un entier positif"))
            })
        })
        .transpose()
}

fn new_message_id() -> String {
    let sequence = NEXT_MESSAGE_ID.fetch_add(1, Ordering::Relaxed);
    format!("mcp-{}-{:x}-{:x}", std::process::id(), now_secs(), sequence)
}

fn ephemeral_connection_name() -> String {
    let sequence = NEXT_CONNECTION_NAME.fetch_add(1, Ordering::Relaxed);
    format!("mcp-{}-{sequence}", std::process::id())
}

fn ledger_message_dto(message: bridget_transport::protocol::LedgerMessage) -> Value {
    json!({
        "id": message.id,
        "from": message.sender,
        "to": message.target,
        "body": message.body,
        "ts": message.ts,
    })
}

fn request_dto(request: bridget_transport::protocol::RequestInfo) -> Value {
    json!({
        "id": request.id,
        "from": request.sender,
        "to": request.target,
        "state": request.state,
        "deadline": request.deadline_at,
        "created": request.created_at,
    })
}

fn issuer_scope(identity: &str) -> String {
    let mut first = 0xcbf29ce484222325_u64;
    let mut second = 0x9e3779b97f4a7c15_u64;
    for byte in identity.bytes() {
        first = (first ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        second = second.rotate_left(5) ^ u64::from(byte);
        second = second.wrapping_mul(0x9e3779b185ebca87);
    }
    format!("012_scope_{first:016x}{second:016x}")
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": { "tools": {} },
        "serverInfo": { "name": "bridget", "version": env!("CARGO_PKG_VERSION") }
    })
}

fn tools() -> Vec<Value> {
    vec![
        json!({
            "name": "bridget_send",
            "description": "Envoyer un message Bridget à un équipier.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "to": { "type": "string", "minLength": 1 },
                    "body": { "type": "string", "minLength": 1 },
                    "reply": { "type": "boolean", "default": false },
                    "reply_timeout": { "type": "integer", "minimum": 1 },
                    "in_reply_to": { "type": "string", "minLength": 1, "description": "Identifiant de la demande Bridget à résoudre par cette réponse." },
                    "id": { "type": "string", "minLength": 1, "description": "Clé métier à réutiliser pour un retry explicite." },
                    "issued_at": { "type": "integer", "minimum": 1, "description": "Horodatage renvoyé par le premier appel ; requis avec id pour rejouer le même contrat." }
                },
                "required": ["to", "body"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": "bridget_who",
            "description": "Lister les équipiers Bridget visibles.",
            "inputSchema": {
                "type": "object",
                "properties": { "domain": { "type": "string" } },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "bridget_ledger",
            "description": "Lire les messages et demandes Bridget récents.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "view": { "enum": ["messages", "requests", "both"] },
                    "limit": { "type": "integer", "minimum": 1, "maximum": 200, "default": 20 }
                },
                "additionalProperties": false
            }
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::cell::Cell;
    use std::collections::BTreeSet;
    use std::io::Cursor;
    use std::os::unix::net::UnixListener;
    use std::path::PathBuf;
    use std::sync::{Barrier, mpsc};
    use std::thread;

    const FIXTURES: &str = include_str!("../tests/fixtures/mcp/fr009.jsonl");

    fn run(lines: &[Value]) -> Vec<Value> {
        let input = lines
            .iter()
            .map(|line| {
                line.as_str()
                    .map_or_else(|| line.to_string(), str::to_string)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let mut stdout = Vec::new();
        let identity = || {
            Ok(crate::mcp_identity::ResolvedIdentity {
                name: "fixture-agent".to_string(),
                instance_id: "fixture-instance".to_string(),
            })
        };
        let execute = |_: &crate::mcp_identity::ResolvedIdentity, _: &str, _: &Value| {
            Err(ToolError::Technical {
                code: "daemon_unreachable",
                message: "daemon de fixture absent".to_string(),
            })
        };
        serve_with(input.as_bytes(), &mut stdout, &identity, &execute).unwrap();
        String::from_utf8(stdout)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    #[test]
    fn matrice_fr009_couvre_les_quinze_cas() {
        let cases: Vec<Value> = FIXTURES
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(cases.len(), 15, "la matrice FR-009 doit rester complète");

        for case in cases {
            let responses = run(case["stdin"].as_array().unwrap());
            let expected_count = case["responses"].as_u64().unwrap() as usize;
            assert_eq!(responses.len(), expected_count, "{}", case["name"]);
            match case["expect"].as_str().unwrap() {
                "initialize" => {
                    assert_eq!(responses[0]["id"], case["id"], "{}", case["name"]);
                    assert_eq!(responses[0]["result"]["protocolVersion"], PROTOCOL_VERSION);
                    assert_eq!(
                        responses[0]["result"]["capabilities"],
                        json!({ "tools": {} })
                    );
                }
                "tools" => assert_eq!(
                    responses.last().unwrap()["result"]["tools"]
                        .as_array()
                        .unwrap()
                        .len(),
                    3
                ),
                "tools_twice" => assert_eq!(responses[1]["result"], responses[2]["result"]),
                "ping" => assert_eq!(responses.last().unwrap()["result"], json!({})),
                "tool_unavailable" => {
                    assert_eq!(responses.last().unwrap()["result"]["isError"], true)
                }
                "error" => assert!(responses.last().unwrap().get("error").is_some()),
                "none" => assert!(responses.is_empty()),
                "mixed_ids" => {
                    assert_eq!(responses[1]["id"], json!(9));
                    assert_eq!(responses[2]["id"], json!("neuf"));
                }
                other => panic!("expectation inconnue: {other}"),
            }
        }
    }

    #[test]
    fn stdout_ne_contient_que_des_reponses_json_rpc() {
        let fixture: Value = serde_json::from_str(FIXTURES.lines().next().unwrap()).unwrap();
        let mut input = fixture["stdin"]
            .as_array()
            .unwrap()
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        input.push('\n');
        input.push_str("{ JSON invalide\n");
        let mut stdout = Vec::new();
        serve(input.as_bytes(), &mut stdout).unwrap();
        for line in String::from_utf8(stdout).unwrap().lines() {
            let response: Value = serde_json::from_str(line).expect("stdout doit être JSON");
            assert_eq!(response["jsonrpc"], "2.0");
            assert!(response.get("result").is_some() || response.get("error").is_some());
        }
    }

    #[test]
    fn tools_call_resout_l_identite_a_chaque_appel() {
        let request = json!({
            "jsonrpc": "2.0", "id": 3, "method": "tools/call",
            "params": { "name": "bridget_who", "arguments": {} }
        });
        let mut session = Session {
            initialize_seen: true,
            initialized: true,
        };
        let response = dispatch_with_identity(&request, &mut session, &|| {
            Err(crate::mcp_identity::IdentityError::LegacyMarker)
        })
        .unwrap();
        assert_eq!(response["result"]["code"], "legacy_marker");
        assert_eq!(response["result"]["isError"], true);
    }

    #[test]
    fn deux_appels_outil_ne_partagent_pas_l_identite_resolue() {
        let mut session = Session {
            initialize_seen: true,
            initialized: true,
        };
        let calls = Cell::new(0);
        let resolver = || {
            calls.set(calls.get() + 1);
            Ok(crate::mcp_identity::ResolvedIdentity {
                name: "agent".to_string(),
                instance_id: "instance".to_string(),
            })
        };
        for id in [3, 4] {
            let request = json!({
                "jsonrpc": "2.0", "id": id, "method": "tools/call",
                "params": { "name": "bridget_who", "arguments": {} }
            });
            let response = dispatch_with_identity(&request, &mut session, &resolver).unwrap();
            assert_eq!(response["id"], id);
        }
        assert_eq!(calls.get(), 2);
    }

    fn test_socket(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "bridget-mcp-{label}-{}-{}.sock",
            std::process::id(),
            NEXT_MESSAGE_ID.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn read_command(reader: &mut BufReader<UnixStream>) -> WrapperToDaemon {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        decode(line.trim_end()).unwrap()
    }

    fn write_command(writer: &mut BufWriter<UnixStream>, response: DaemonToWrapper) {
        writeln!(writer, "{}", encode(&response).unwrap()).unwrap();
        writer.flush().unwrap();
    }

    #[test]
    fn send_transmet_un_corps_riche_octet_pour_octet_et_negocie_le_client() {
        let socket = test_socket("send-rich");
        let listener = UnixListener::bind(&socket).unwrap();
        let expected_body =
            "l'apostrophe, \"guillemets\", $VAR, `backticks`,\net l'emoji é".to_string();
        let server = thread::spawn({
            let expected_body = expected_body.clone();
            move || {
                let (stream, _) = listener.accept().unwrap();
                let read_stream = stream.try_clone().unwrap();
                let mut reader = BufReader::new(read_stream);
                let mut writer = BufWriter::new(stream);
                assert!(matches!(
                    read_command(&mut reader),
                    WrapperToDaemon::RoleHandshake {
                        role: ConnectionRole::Client
                    }
                ));
                write_command(
                    &mut writer,
                    DaemonToWrapper::RoleAccepted {
                        role: ConnectionRole::Client,
                    },
                );
                assert!(matches!(
                    read_command(&mut reader),
                    WrapperToDaemon::ClientHello { .. }
                ));
                write_command(
                    &mut writer,
                    DaemonToWrapper::ClientWelcome {
                        version: CLIENT_CONTRACT_VERSION,
                        horizon_secs: 60,
                        issued_at_tolerance_secs: 5,
                        capabilities: vec![ClientCapability::SendIdempotent],
                    },
                );
                match read_command(&mut reader) {
                    WrapperToDaemon::SendIdempotent {
                        message,
                        message_id,
                        ..
                    } => {
                        assert_eq!(message.body, expected_body);
                        assert_eq!(message.in_reply_to, None);
                        assert_eq!(message_id, "retry-me");
                    }
                    other => panic!("commande inattendue: {other:?}"),
                }
                write_command(
                    &mut writer,
                    DaemonToWrapper::IdempotencyResult {
                        operation_kind: "send".to_string(),
                        idempotency_key: "retry-me".to_string(),
                        issue: IdempotencyIssue::Accepted { expires_at: 60 },
                    },
                );
            }
        });
        let result = execute_tool_at(
            "codex-1",
            "bridget_send",
            json!({
            "to": "claude-1", "body": expected_body, "id": "retry-me"
            , "issued_at": 1_700_000_000
            })
            .as_object()
            .unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(result["status"], "accepted");
        assert_eq!(result["id"], "retry-me");
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn send_transmet_in_reply_to_dans_l_enveloppe_idempotente() {
        let socket = test_socket("send-reply");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Client
                }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client,
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ClientHello { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: vec![ClientCapability::SendIdempotent],
                },
            );
            match read_command(&mut reader) {
                WrapperToDaemon::SendIdempotent { message, .. } => {
                    assert_eq!(message.in_reply_to.as_deref(), Some("request-open"));
                    assert!(!message.reply);
                }
                other => panic!("commande inattendue: {other:?}"),
            }
            write_command(
                &mut writer,
                DaemonToWrapper::IdempotencyResult {
                    operation_kind: "send".to_string(),
                    idempotency_key: "reply-1".to_string(),
                    issue: IdempotencyIssue::Accepted { expires_at: 60 },
                },
            );
        });
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "bridget_send",
                "arguments": {
                    "to": "bridget",
                    "body": "réponse liée",
                    "in_reply_to": "request-open",
                    "id": "reply-1",
                    "issued_at": 1_700_000_000
                }
            }
        });
        let mut session = Session {
            initialize_seen: true,
            initialized: true,
        };
        let resolver = || {
            Ok(crate::mcp_identity::ResolvedIdentity {
                name: "codex-1".to_string(),
                instance_id: "test-instance".to_string(),
            })
        };
        let execute =
            |identity: &crate::mcp_identity::ResolvedIdentity, name: &str, arguments: &Value| {
                execute_tool_at_with_scope(
                    &identity.name,
                    &identity.instance_id,
                    name,
                    arguments.as_object().expect("arguments objet"),
                    &socket,
                )
            };
        let result = dispatch_with_executor(&request, &mut session, &resolver, &execute).unwrap();
        assert_eq!(result["result"]["structuredContent"]["status"], "accepted");
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn schema_send_expose_in_reply_to_non_vide() {
        let send = tools()
            .into_iter()
            .find(|tool| tool["name"] == "bridget_send")
            .unwrap();
        assert_eq!(
            send["inputSchema"]["properties"]["in_reply_to"]["type"],
            "string"
        );
        assert_eq!(
            send["inputSchema"]["properties"]["in_reply_to"]["minLength"],
            1
        );
    }

    #[test]
    fn resultats_metier_conservent_les_categories_de_refus_fermees() {
        for (category, expected) in [
            ("dnd", "dnd"),
            ("circuit_breaker", "circuit_breaker"),
            ("duplicate_content", "duplicate"),
            ("routing", "unknown_recipient"),
        ] {
            let result = send_issue_result(
                "id-1",
                1_700_000_000,
                IdempotencyIssue::Rejected {
                    category: category.to_string(),
                    reason: "motif exact".to_string(),
                    expires_at: 1_700_000_100,
                },
            );
            assert_eq!(result["status"], expected);
            assert_eq!(result["reason"], "motif exact");
        }
    }

    #[test]
    fn retry_rejoue_scope_et_horodatage_malgre_rename() {
        let socket = test_socket("retry-scope");
        let listener = UnixListener::bind(&socket).unwrap();
        let (seen_tx, seen_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            for _ in 0..2 {
                let (stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut writer = BufWriter::new(stream);
                assert!(matches!(
                    read_command(&mut reader),
                    WrapperToDaemon::RoleHandshake { .. }
                ));
                write_command(
                    &mut writer,
                    DaemonToWrapper::RoleAccepted {
                        role: ConnectionRole::Client,
                    },
                );
                let hello = read_command(&mut reader);
                write_command(
                    &mut writer,
                    DaemonToWrapper::ClientWelcome {
                        version: CLIENT_CONTRACT_VERSION,
                        horizon_secs: 60,
                        issued_at_tolerance_secs: 5,
                        capabilities: vec![ClientCapability::SendIdempotent],
                    },
                );
                let send = read_command(&mut reader);
                seen_tx.send((hello, send)).unwrap();
                write_command(
                    &mut writer,
                    DaemonToWrapper::IdempotencyResult {
                        operation_kind: "send".to_string(),
                        idempotency_key: "retry-1".to_string(),
                        issue: IdempotencyIssue::Accepted {
                            expires_at: 1_700_000_060,
                        },
                    },
                );
            }
        });
        let arguments = json!({
            "to": "claude-1", "body": "même prompt", "id": "retry-1", "issued_at": 1_700_000_000
        });
        for (index, identity) in ["avant-rename", "apres-rename"].into_iter().enumerate() {
            if index == 1 {
                thread::sleep(Duration::from_secs(1));
            }
            let result = execute_tool_at_with_scope(
                identity,
                "instance-stable-1",
                "bridget_send",
                arguments.as_object().unwrap(),
                &socket,
            )
            .unwrap();
            assert_eq!(result["issued_at"], 1_700_000_000);
        }
        let first = seen_rx.recv().unwrap();
        let second = seen_rx.recv().unwrap();
        let scope = |command: WrapperToDaemon| match command {
            WrapperToDaemon::ClientHello { issuer_scope, .. } => issuer_scope,
            other => panic!("hello attendu: {other:?}"),
        };
        assert_eq!(scope(first.0), scope(second.0));
        for command in [first.1, second.1] {
            match command {
                WrapperToDaemon::SendIdempotent {
                    message, issued_at, ..
                } => {
                    assert_eq!(issued_at, 1_700_000_000);
                    assert_eq!(message.body, "même prompt");
                }
                other => panic!("send attendu: {other:?}"),
            }
        }
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn coupure_apres_transmission_devient_outcome_unknown_et_le_retry_reste_identique() {
        let socket = test_socket("cut-after-send");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RoleHandshake { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client,
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ClientHello { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: vec![ClientCapability::SendIdempotent],
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::SendIdempotent { .. }
            ));
            drop(writer);
            drop(reader);

            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RoleHandshake { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client,
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ClientHello { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: vec![ClientCapability::SendIdempotent],
                },
            );
            match read_command(&mut reader) {
                WrapperToDaemon::SendIdempotent {
                    message_id,
                    issued_at,
                    ..
                } => {
                    assert_eq!(message_id, "retry-cut");
                    assert_eq!(issued_at, 1_700_000_000);
                }
                other => panic!("send attendu: {other:?}"),
            }
            write_command(
                &mut writer,
                DaemonToWrapper::IdempotencyResult {
                    operation_kind: "send".to_string(),
                    idempotency_key: "retry-cut".to_string(),
                    issue: IdempotencyIssue::Accepted {
                        expires_at: 1_700_000_060,
                    },
                },
            );
        });
        let arguments = json!({
            "to":"claude-1", "body":"retry", "id":"retry-cut", "issued_at":1_700_000_000
        });
        let first = execute_tool_at_with_scope(
            "agent",
            "instance",
            "bridget_send",
            arguments.as_object().unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(first["status"], "outcome_unknown");
        let retry = execute_tool_at_with_scope(
            "agent-renamed",
            "instance",
            "bridget_send",
            arguments.as_object().unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(retry["status"], "accepted");
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn issues_idempotentes_deterministes_restent_metier_et_inconnues_sont_preservees() {
        for (issue, status) in [
            (IdempotencyIssue::EnvelopeMismatch, "envelope_mismatch"),
            (IdempotencyIssue::IdempotencyExpired, "idempotency_expired"),
            (IdempotencyIssue::InvalidIssuedAt, "invalid_issued_at"),
        ] {
            assert_eq!(send_issue_result("id", 7, issue)["status"], status);
        }
        let unknown = send_issue_result(
            "id",
            7,
            IdempotencyIssue::Rejected {
                category: "future_refusal".to_string(),
                reason: "raison".to_string(),
                expires_at: 8,
            },
        );
        assert_eq!(unknown["status"], "future_refusal");
    }

    #[test]
    fn resultat_mcp_duplique_le_payload_dans_textcontent() {
        let payload = json!({ "status": "accepted", "id": "m-1", "issued_at": 8 });
        let result = tool_result(payload.clone());
        assert_eq!(result["structuredContent"], payload);
        assert_eq!(
            serde_json::from_str::<Value>(result["content"][0]["text"].as_str().unwrap()).unwrap(),
            result["structuredContent"]
        );
    }

    #[test]
    fn huit_connexions_simultanees_ont_des_noms_ephemeres_distincts_et_la_neuvieme_est_busy() {
        let started = Arc::new(Barrier::new(MAX_IN_FLIGHT_TOOL_CALLS + 1));
        let (started_tx, started_rx) = mpsc::channel();
        let socket = test_socket("eight-registers");
        let listener = UnixListener::bind(&socket).unwrap();
        let (names_tx, names_rx) = mpsc::channel();
        let daemon = thread::spawn(move || {
            for _ in 0..MAX_IN_FLIGHT_TOOL_CALLS {
                let (stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut writer = BufWriter::new(stream);
                match read_command(&mut reader) {
                    WrapperToDaemon::Register {
                        name: Some(name), ..
                    } => names_tx.send(name).unwrap(),
                    other => panic!("Register MCP attendu: {other:?}"),
                }
                write_command(
                    &mut writer,
                    DaemonToWrapper::Registered {
                        name: "mcp".to_string(),
                    },
                );
            }
        });
        let input = [
            json!({"jsonrpc":"2.0","id":1,"method":"initialize"}),
            json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        ]
        .into_iter()
        .chain((2..=10).map(|id| {
            json!({
                "jsonrpc":"2.0", "id": id, "method":"tools/call",
                "params":{"name":"bridget_who","arguments":{}}
            })
        }))
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join("\n");
        let barrier = Arc::clone(&started);
        let socket_for_calls = socket.clone();
        let server = thread::spawn(move || {
            let resolver = || {
                Ok(crate::mcp_identity::ResolvedIdentity {
                    name: "agent".to_string(),
                    instance_id: "instance".to_string(),
                })
            };
            let execute = move |_: &crate::mcp_identity::ResolvedIdentity, _: &str, _: &Value| {
                let _connection = registered_connection(&socket_for_calls).unwrap();
                started_tx.send(()).unwrap();
                barrier.wait();
                Ok(json!({ "agents": [] }))
            };
            let mut output = Vec::new();
            serve_with(Cursor::new(input), &mut output, &resolver, &execute).unwrap();
            output
        });
        for _ in 0..MAX_IN_FLIGHT_TOOL_CALLS {
            started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        }
        started.wait();
        let output = String::from_utf8(server.join().unwrap()).unwrap();
        let responses = output
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            responses
                .iter()
                .filter(|response| response["result"]["code"] == "busy")
                .count(),
            1
        );

        let names = (0..MAX_IN_FLIGHT_TOOL_CALLS)
            .map(|_| names_rx.recv_timeout(Duration::from_secs(2)).unwrap())
            .collect::<BTreeSet<_>>();
        assert_eq!(names.len(), MAX_IN_FLIGHT_TOOL_CALLS);
        daemon.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn connexion_non_bloquante_refuse_un_budget_expire_avant_toute_attente() {
        let socket = test_socket("expired-connect");
        let _listener = UnixListener::bind(&socket).unwrap();
        let error =
            connect_nonblocking(&socket, Instant::now() - Duration::from_millis(1)).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn dto_ledger_respectent_le_contrat_outil() {
        let source = bridget_transport::protocol::LedgerMessage {
            id: "m-1".to_string(),
            ts: 4,
            sender: "alice".to_string(),
            target: "bob".to_string(),
            body: "riche".to_string(),
        };
        let message = ledger_message_dto(source.clone());
        assert_eq!(
            message,
            json!({"id":"m-1","from":"alice","to":"bob","body":"riche","ts":4})
        );
        assert_eq!(
            crate::cli::render_ledger(&[source]),
            "Derniers 1 messages :\n  [4] alice → bob: riche\n"
        );
        let request = request_dto(bridget_transport::protocol::RequestInfo {
            id: "r-1".to_string(),
            sender: "alice".to_string(),
            target: "bob".to_string(),
            state: "open".to_string(),
            created_at: 2,
            deadline_at: 9,
            cancel_reason: None,
            deferred_reminder_level: None,
            deferred_reminder_at: None,
        });
        assert_eq!(
            request,
            json!({"id":"r-1","from":"alice","to":"bob","state":"open","deadline":9,"created":2})
        );
    }

    #[test]
    fn daemon_coupe_est_un_is_error_technique_immediat() {
        let missing = test_socket("missing");
        let error = execute_tool_at("codex-1", "bridget_who", &serde_json::Map::new(), &missing)
            .unwrap_err();
        assert!(matches!(
            error,
            ToolError::Technical {
                code: "daemon_unreachable",
                ..
            }
        ));
    }

    #[test]
    fn limite_d_appels_simultanes_refuse_avant_toute_connexion() {
        let active = AtomicUsize::new(0);
        for _ in 0..MAX_IN_FLIGHT_TOOL_CALLS {
            assert!(try_reserve_tool_call(&active));
        }
        assert!(!try_reserve_tool_call(&active));
        assert_eq!(active.load(Ordering::Acquire), MAX_IN_FLIGHT_TOOL_CALLS);
    }

    #[test]
    fn who_ouvre_une_connexion_ephemere_register_puis_commande() {
        let socket = test_socket("who");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let read_stream = stream.try_clone().unwrap();
            let mut reader = BufReader::new(read_stream);
            let mut writer = BufWriter::new(stream);
            assert!(
                matches!(read_command(&mut reader), WrapperToDaemon::Register { agent_type, .. } if agent_type == "mcp")
            );
            write_command(
                &mut writer,
                DaemonToWrapper::Registered {
                    name: "mcp-test".to_string(),
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ListAgents
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::AgentList { agents: Vec::new() },
            );
        });
        let result =
            execute_tool_at("codex-1", "bridget_who", &serde_json::Map::new(), &socket).unwrap();
        assert_eq!(result, json!({ "agents": [] }));
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }
}
