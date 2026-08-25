//! Relais local du poste de travail Bridget.
//!
//! Le navigateur ne parle jamais à la socket Unix du daemon. Ce module ouvre
//! des connexions ordinaires vers les projections publiques (`ListAgents`,
//! `LedgerProjection`, Attach) puis les traduit en HTTP/SSE loopback.

use bridget_core::BridgetMessage;
use bridget_transport::protocol::{
    AttachWindow, CLIENT_CONTRACT_VERSION, ClientCapability, ConnectionRole, IdempotencyIssue,
    LedgerMessage, LedgerScope, decode, encode,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use maicie::ui_projection::{UiMissionProjectionV1, read_ui_mission_projection_v1};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const UI_VERSION: u8 = 1;
const MAX_HTTP_HEADER_BYTES: usize = 16 * 1024;
const MAX_HTTP_BODY_BYTES: usize = 64 * 1024;
const MAX_UI_AGENT_NAME_BYTES: usize = 100;
const MAX_UI_SSE_EVENTS: usize = 20_000;
const MAX_UI_RECONNECT_ATTEMPTS: usize = 50;
const UI_RECONNECT_DELAY: Duration = Duration::from_millis(100);
const UI_SENDER: &str = "humain";
const UI_INDEX: &[u8] = include_bytes!("../assets/ui/index.html");
const UI_SCRIPT: &[u8] = include_bytes!("../assets/ui/app.js");
const UI_THEME: &[u8] = include_bytes!("../assets/ui/theme.css");

#[derive(Debug, Clone)]
pub struct UiRelayConfig {
    pub daemon_socket: PathBuf,
    pub maicie_config: PathBuf,
    pub bind: SocketAddr,
    pub token: String,
}

impl UiRelayConfig {
    pub fn loopback(daemon_socket: PathBuf, maicie_config: PathBuf) -> Self {
        Self {
            daemon_socket,
            maicie_config,
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            token: uuid::Uuid::new_v4().simple().to_string(),
        }
    }
}

#[derive(Debug)]
pub enum UiError {
    Io(std::io::Error),
    Protocol(String),
    Configuration(String),
}

impl fmt::Display for UiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O du relais UI: {error}"),
            Self::Protocol(error) => write!(formatter, "protocole Bridget UI: {error}"),
            Self::Configuration(error) => write!(formatter, "configuration UI: {error}"),
        }
    }
}

impl std::error::Error for UiError {}

impl From<std::io::Error> for UiError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

/// Serveur HTTP local sans état métier propre.
pub struct UiRelay {
    listener: TcpListener,
    config: UiRelayConfig,
}

impl UiRelay {
    pub fn bind(config: UiRelayConfig) -> Result<Self, UiError> {
        if !config.bind.ip().is_loopback() {
            return Err(UiError::Configuration(
                "le relais UI doit écouter exclusivement sur la boucle locale".to_string(),
            ));
        }
        if config.token.is_empty() {
            return Err(UiError::Configuration("jeton UI absent".to_string()));
        }
        let listener = TcpListener::bind(config.bind)?;
        Ok(Self { listener, config })
    }

    pub fn local_addr(&self) -> Result<SocketAddr, UiError> {
        self.listener.local_addr().map_err(UiError::Io)
    }

    pub fn url(&self) -> Result<String, UiError> {
        Ok(format!(
            "http://{}/?token={}",
            self.local_addr()?,
            self.config.token
        ))
    }

    pub fn serve(self) -> Result<(), UiError> {
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let config = self.config.clone();
                    thread::spawn(move || {
                        let mut stream = stream;
                        if let Err(error) = serve_connection(&mut stream, &config) {
                            let _ = write_text(&mut stream, 500, &format!("relais UI: {error}"));
                        }
                    });
                }
                Err(error) => return Err(UiError::Io(error)),
            }
        }
        Ok(())
    }

    #[cfg(test)]
    fn serve_one(&self) -> Result<(), UiError> {
        let (stream, _) = self.listener.accept()?;
        let mut stream = stream;
        serve_connection(&mut stream, &self.config)
    }
}

/// Lance `bridget ui`. Le terminal garde le jeton ; aucun secret ne part
/// dans le HTML ou dans une configuration persistée.
pub fn run(daemon_socket: PathBuf, maicie_config: PathBuf) -> Result<(), UiError> {
    let relay = UiRelay::bind(UiRelayConfig::loopback(daemon_socket, maicie_config))?;
    println!("Bridget UI (lecture seule) : {}", relay.url()?);
    relay.serve()
}

#[derive(Serialize)]
struct UiSnapshotV1 {
    version: u8,
    agents: Vec<UiAgentRowV1>,
    open_requests: Vec<bridget_transport::protocol::RequestInfo>,
    missions: UiMissionProjectionV1,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    recovery_losses: Vec<UiRecoveryLossV1>,
}

#[derive(Debug, Serialize)]
struct UiAgentRowV1 {
    name: String,
    #[serde(rename = "type")]
    agent_type: String,
    host: String,
    state: &'static str,
    last_message_at: Option<i64>,
    last_excerpt: Option<String>,
    unread: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UiSendRequestV1 {
    version: u8,
    to: String,
    body: String,
    reply: bool,
}

#[derive(Debug, Serialize)]
struct UiSendAcceptedV1 {
    version: u8,
    delivery_id: String,
    issued_at: i64,
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct UiSendErrorV1 {
    version: u8,
    code: &'static str,
}

#[derive(Debug, Serialize)]
struct UiRelayStateV1 {
    version: u8,
    kind: &'static str,
    state: &'static str,
    since: i64,
}

#[derive(Serialize)]
struct UiRecoveryLossV1 {
    name: String,
    reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Serialize)]
struct UiJournalEventV1<'a> {
    version: u8,
    event: &'a DaemonToWrapper,
}

fn serve_connection(stream: &mut TcpStream, config: &UiRelayConfig) -> Result<(), UiError> {
    let request = read_request(stream)?;
    if request.path == "/v1/send" && request.method != "POST" {
        return write_text(stream, 405, "méthode non autorisée");
    }
    if request.method != "GET" && request.method != "POST" {
        return write_text(stream, 405, "méthode non autorisée");
    }
    if request.query.get("token") != Some(&config.token) {
        return write_text(stream, 403, "jeton UI invalide");
    }
    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/") => write_asset(stream, 200, "text/html; charset=utf-8", UI_INDEX),
        ("GET", "/app.js") => write_asset(
            stream,
            200,
            "application/javascript; charset=utf-8",
            UI_SCRIPT,
        ),
        ("GET", "/theme.css") => write_asset(stream, 200, "text/css; charset=utf-8", UI_THEME),
        ("POST", "/v1/send") => match post_ui_message(config, &request.body) {
            Ok(response) => write_json(stream, 202, &response),
            Err((status, code)) => write_json(
                stream,
                status,
                &UiSendErrorV1 {
                    version: UI_VERSION,
                    code,
                },
            ),
        },
        ("GET", "/v1/snapshot") => {
            let snapshot = read_snapshot(config)?;
            write_json(stream, 200, &snapshot)
        }
        ("GET", "/v1/journal") => {
            let agent = request
                .query
                .get("agent")
                .ok_or_else(|| UiError::Protocol("paramètre agent absent".to_string()))?;
            validate_agent(agent)?;
            let window = match request.query.get("from_seq") {
                Some(value) => AttachWindow::Seq(value.parse().map_err(|_| {
                    UiError::Protocol("from_seq doit être un entier non signé".to_string())
                })?),
                None => AttachWindow::Today,
            };
            stream_sse_journal(stream, &config.daemon_socket, agent, window, None)
        }
        ("GET", "/v1/watch") => {
            let agent = request
                .query
                .get("agent")
                .ok_or_else(|| UiError::Protocol("paramètre agent absent".to_string()))?;
            validate_agent(agent)?;
            let window = match request.query.get("from_seq") {
                Some(value) => AttachWindow::Seq(value.parse().map_err(|_| {
                    UiError::Protocol("from_seq doit être un entier non signé".to_string())
                })?),
                None => AttachWindow::Today,
            };
            // La vue combinée est la porte d'entrée de la future page : elle
            // raccorde Attach avant de capturer l'instantané, donc aucun delta
            // journal ne peut se glisser silencieusement entre les deux.
            stream_sse_journal(stream, &config.daemon_socket, agent, window, Some(config))
        }
        _ => write_text(stream, 404, "ressource UI inconnue"),
    }
}

fn post_ui_message(
    config: &UiRelayConfig,
    body: &[u8],
) -> Result<UiSendAcceptedV1, (u16, &'static str)> {
    let request: UiSendRequestV1 =
        serde_json::from_slice(body).map_err(|_| (400, "invalid_body"))?;
    if request.version != UI_VERSION || request.body.trim().is_empty() {
        return Err((400, "invalid_body"));
    }
    validate_agent(&request.to).map_err(|_| (404, "unknown_recipient"))?;

    let agents = read_agent_list(&config.daemon_socket).map_err(|_| (503, "daemon_unavailable"))?;
    validate_ui_recipient(&agents, &request.to)?;
    send_ui_message(&config.daemon_socket, request).map_err(|_| (503, "daemon_unavailable"))
}

fn validate_ui_recipient(
    agents: &[bridget_transport::protocol::AgentInfo],
    recipient: &str,
) -> Result<(), (u16, &'static str)> {
    let Some(agent) = agents.iter().find(|agent| agent.name == recipient) else {
        return Err((404, "unknown_recipient"));
    };
    if matches!(agent.state.as_str(), "stopped" | "unreachable") {
        return Err((409, "agent_stopped"));
    }
    Ok(())
}

fn read_agent_list(
    socket_path: &Path,
) -> Result<Vec<bridget_transport::protocol::AgentInfo>, UiError> {
    let stream = UnixStream::connect(socket_path)?;
    let read_stream = stream.try_clone()?;
    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(read_stream);
    send_daemon(&mut writer, &WrapperToDaemon::ListAgents)?;
    match read_daemon(&mut reader)? {
        DaemonToWrapper::AgentList { agents } => Ok(agents),
        response => Err(UiError::Protocol(format!(
            "AgentList attendu, reçu {response:?}"
        ))),
    }
}

fn send_ui_message(
    socket_path: &Path,
    request: UiSendRequestV1,
) -> Result<UiSendAcceptedV1, UiError> {
    let stream = UnixStream::connect(socket_path)?;
    let read_stream = stream.try_clone()?;
    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(read_stream);
    send_daemon(
        &mut writer,
        &WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Client,
        },
    )?;
    match read_daemon(&mut reader)? {
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client,
        } => {}
        response => {
            return Err(UiError::Protocol(format!(
                "acceptation client attendue, reçu {response:?}"
            )));
        }
    }
    send_daemon(
        &mut writer,
        &WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: crate::mcp::issuer_scope("bridget-ui"),
            capabilities: vec![ClientCapability::SendIdempotent],
        },
    )?;
    match read_daemon(&mut reader)? {
        DaemonToWrapper::ClientWelcome { capabilities, .. }
            if capabilities.contains(&ClientCapability::SendIdempotent) => {}
        response => {
            return Err(UiError::Protocol(format!(
                "contrat client attendu, reçu {response:?}"
            )));
        }
    }

    let issued_at = now_secs();
    let mut message = BridgetMessage::new(UI_SENDER, request.to, request.body);
    message.reply = request.reply;
    let message_id = message.id.clone();
    send_daemon(
        &mut writer,
        &WrapperToDaemon::SendIdempotent {
            message,
            message_id: message_id.clone(),
            issued_at,
        },
    )?;
    match read_daemon(&mut reader)? {
        DaemonToWrapper::IdempotencyResult {
            issue:
                IdempotencyIssue::OutcomeUnknown {
                    delivery_id: Some(delivery_id),
                    ..
                },
            ..
        } => Ok(UiSendAcceptedV1 {
            version: UI_VERSION,
            delivery_id,
            issued_at,
            status: "in_flight",
        }),
        DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::Accepted { .. },
            ..
        } => Ok(UiSendAcceptedV1 {
            version: UI_VERSION,
            delivery_id: message_id,
            issued_at,
            status: "in_flight",
        }),
        response => Err(UiError::Protocol(format!(
            "issue idempotente en vol attendue, reçu {response:?}"
        ))),
    }
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .min(i64::MAX as u64) as i64
}

fn read_snapshot(config: &UiRelayConfig) -> Result<UiSnapshotV1, UiError> {
    let (agent_infos, messages, open_requests) = read_bridget_snapshot(&config.daemon_socket)?;
    let agents = compose_agent_rows(agent_infos, &messages);
    let missions = read_ui_mission_projection_v1(&config.maicie_config)
        .map_err(|error| UiError::Configuration(error.to_string()))?;
    let recovery_losses = read_recovery_losses(&config.daemon_socket);
    Ok(UiSnapshotV1 {
        version: UI_VERSION,
        agents,
        open_requests,
        missions,
        recovery_losses,
    })
}

fn compose_agent_rows(
    agents: Vec<bridget_transport::protocol::AgentInfo>,
    messages: &[LedgerMessage],
) -> Vec<UiAgentRowV1> {
    agents
        .into_iter()
        .map(|agent| {
            let last = messages
                .iter()
                .filter(|message| message.sender == agent.name || message.target == agent.name)
                .max_by(|left, right| (left.ts, &left.id).cmp(&(right.ts, &right.id)));
            UiAgentRowV1 {
                name: agent.name.clone(),
                agent_type: agent.agent_type,
                host: agent.host,
                state: public_agent_state(&agent.state),
                last_message_at: last.map(|message| message.ts),
                last_excerpt: last.map(|message| excerpt(&message.body)),
                unread: messages
                    .iter()
                    .filter(|message| message.sender == agent.name && message.target == UI_SENDER)
                    .count(),
            }
        })
        .collect()
}

fn public_agent_state(state: &str) -> &'static str {
    match state {
        "busy" => "busy",
        "stopped" | "unreachable" => "stopped",
        _ => "alive",
    }
}

fn excerpt(body: &str) -> String {
    const MAX_EXCERPT_CHARS: usize = 160;
    let mut excerpt = body.chars().take(MAX_EXCERPT_CHARS).collect::<String>();
    if body.chars().count() > MAX_EXCERPT_CHARS {
        excerpt.push('…');
    }
    excerpt
}

fn recovery_losses_path_for_socket(socket_path: &Path) -> PathBuf {
    crate::recovery_trace::report_path(&crate::desired_state::path_for_daemon_db(
        &socket_path.with_extension("db"),
    ))
}

fn read_recovery_losses(socket_path: &Path) -> Vec<UiRecoveryLossV1> {
    match crate::recovery_trace::load_report(&recovery_losses_path_for_socket(socket_path)) {
        Ok(Some(report)) => report
            .absents
            .into_iter()
            .map(|entry| UiRecoveryLossV1 {
                name: entry.name,
                reason: entry.reason,
                detail: entry.detail,
            })
            .collect(),
        Ok(None) | Err(_) => Vec::new(),
    }
}

/// Projection globale déjà détenue par le daemon. La connexion reste dans le
/// rôle wrapper historique, lequel autorise ces deux lectures sans créer une
/// présence temporaire dans l'annuaire.
fn read_bridget_snapshot(
    socket_path: &Path,
) -> Result<
    (
        Vec<bridget_transport::protocol::AgentInfo>,
        Vec<LedgerMessage>,
        Vec<bridget_transport::protocol::RequestInfo>,
    ),
    UiError,
> {
    let stream = UnixStream::connect(socket_path)?;
    let read_stream = stream.try_clone()?;
    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(read_stream);
    send_daemon(&mut writer, &WrapperToDaemon::ListAgents)?;
    let agents = match read_daemon(&mut reader)? {
        DaemonToWrapper::AgentList { agents } => agents,
        response => {
            return Err(UiError::Protocol(format!(
                "AgentList attendu, reçu {response:?}"
            )));
        }
    };
    send_daemon(
        &mut writer,
        &WrapperToDaemon::LedgerProjection {
            scope: LedgerScope::Both,
            limit: 200,
        },
    )?;
    let (messages, open_requests) = match read_daemon(&mut reader)? {
        DaemonToWrapper::LedgerProjection { messages, requests } => (messages, requests),
        response => {
            return Err(UiError::Protocol(format!(
                "LedgerProjection attendu, reçu {response:?}"
            )));
        }
    };
    Ok((agents, messages, open_requests))
}

/// Traduit un unique abonnement Attach existant vers SSE. L'abonnement est
/// ouvert avant le premier événement SSE : le client reçoit donc exactement
/// le rejeu/cursor, puis `SnapshotCaughtUp`, puis le live déjà garanti par
/// Bridget, sans seconde source de journal.
fn stream_sse_journal(
    http: &mut TcpStream,
    socket_path: &Path,
    agent: &str,
    window: AttachWindow,
    snapshot_config: Option<&UiRelayConfig>,
) -> Result<(), UiError> {
    write!(
        http,
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n"
    )?;
    http.flush()?;
    let mut session = match open_attach_session(socket_path, agent, window.clone()) {
        Ok(session) => session,
        Err(error) if snapshot_config.is_some() => {
            write_relay_state(http, "lost", now_secs())?;
            let _ = http.shutdown(Shutdown::Both);
            return Ok(());
        }
        Err(error) => return Err(error),
    };
    let initial_snapshot = snapshot_config.map(read_snapshot).transpose()?;
    if snapshot_config.is_some() {
        write_relay_state(http, "connected", now_secs())?;
    }
    if let Some(snapshot) = initial_snapshot {
        write_sse(http, "snapshot", &snapshot)?;
    }
    write_sse(
        http,
        "journal",
        &UiJournalEventV1 {
            version: UI_VERSION,
            event: &session.subscribed,
        },
    )?;

    let mut last_seq = None;
    let mut events = 0_usize;
    while events < MAX_UI_SSE_EVENTS {
        let event = match read_daemon(&mut session.reader) {
            Ok(event) => event,
            Err(error) if snapshot_config.is_some() => {
                let reconnecting_since = now_secs();
                write_relay_state(http, "reconnecting", reconnecting_since)?;
                let resume_window = last_seq
                    .and_then(|seq: u64| seq.checked_add(1))
                    .map(AttachWindow::Seq)
                    .unwrap_or_else(|| window.clone());
                let mut recovered = None;
                for _ in 0..MAX_UI_RECONNECT_ATTEMPTS {
                    match open_attach_session(socket_path, agent, resume_window.clone()) {
                        Ok(next) => {
                            recovered = Some(next);
                            break;
                        }
                        Err(_) => thread::sleep(UI_RECONNECT_DELAY),
                    }
                }
                let Some(next) = recovered else {
                    write_relay_state(http, "lost", reconnecting_since)?;
                    break;
                };
                session = next;
                let restored_snapshot = snapshot_config.map(read_snapshot).transpose()?;
                write_relay_state(http, "connected", now_secs())?;
                if let Some(snapshot) = restored_snapshot {
                    write_sse(http, "snapshot", &snapshot)?;
                }
                write_sse(
                    http,
                    "journal",
                    &UiJournalEventV1 {
                        version: UI_VERSION,
                        event: &session.subscribed,
                    },
                )?;
                continue;
            }
            Err(error) => return Err(error),
        };
        last_seq = event_resume_seq(&event).or(last_seq);
        write_sse(
            http,
            "journal",
            &UiJournalEventV1 {
                version: UI_VERSION,
                event: &event,
            },
        )?;
        events += 1;
        if matches!(event, DaemonToWrapper::End { .. }) {
            if snapshot_config.is_some() {
                write_relay_state(http, "lost", now_secs())?;
            }
            break;
        }
    }
    let _ = http.shutdown(Shutdown::Both);
    Ok(())
}

struct UiAttachSession {
    _writer: BufWriter<UnixStream>,
    reader: BufReader<UnixStream>,
    subscribed: DaemonToWrapper,
}

fn open_attach_session(
    socket_path: &Path,
    agent: &str,
    window: AttachWindow,
) -> Result<UiAttachSession, UiError> {
    let stream = UnixStream::connect(socket_path)?;
    let read_stream = stream.try_clone()?;
    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(read_stream);
    send_daemon(
        &mut writer,
        &WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Attach,
        },
    )?;
    match read_daemon(&mut reader)? {
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach,
        } => {}
        DaemonToWrapper::AttachRejected { reason, .. } => {
            return Err(UiError::Protocol(format!("rôle attach refusé: {reason:?}")));
        }
        response => {
            return Err(UiError::Protocol(format!(
                "acceptation attach attendue, reçu {response:?}"
            )));
        }
    }
    send_daemon(
        &mut writer,
        &WrapperToDaemon::Subscribe {
            agent: agent.to_string(),
            window,
        },
    )?;
    let subscribed = match read_daemon(&mut reader)? {
        as_message @ DaemonToWrapper::Subscribed { .. } => as_message,
        DaemonToWrapper::AttachRejected { reason, .. } => {
            return Err(UiError::Protocol(format!(
                "abonnement attach refusé: {reason:?}"
            )));
        }
        response => {
            return Err(UiError::Protocol(format!(
                "confirmation Subscribe attendue, reçu {response:?}"
            )));
        }
    };
    Ok(UiAttachSession {
        _writer: writer,
        reader,
        subscribed,
    })
}

fn event_resume_seq(event: &DaemonToWrapper) -> Option<u64> {
    match event {
        DaemonToWrapper::JournalFragment { seq, .. } => Some(*seq),
        DaemonToWrapper::SnapshotCaughtUp { through_seq, .. } => *through_seq,
        DaemonToWrapper::Gap { to_seq, .. } => Some(*to_seq),
        _ => None,
    }
}

fn write_relay_state(http: &mut TcpStream, state: &'static str, since: i64) -> Result<(), UiError> {
    write_sse(
        http,
        "relay_state",
        &UiRelayStateV1 {
            version: UI_VERSION,
            kind: "relay_state",
            state,
            since,
        },
    )
}

fn write_sse<T: Serialize>(
    http: &mut TcpStream,
    event_name: &str,
    value: &T,
) -> Result<(), UiError> {
    let payload = serde_json::to_string(value)
        .map_err(|error| UiError::Protocol(format!("SSE JSON invalide: {error}")))?;
    write!(http, "event: {event_name}\ndata: {payload}\n\n")?;
    http.flush()?;
    Ok(())
}

fn send_daemon(
    writer: &mut BufWriter<UnixStream>,
    message: &WrapperToDaemon,
) -> Result<(), UiError> {
    writeln!(
        writer,
        "{}",
        encode(message).map_err(|error| UiError::Protocol(error.to_string()))?
    )?;
    writer.flush()?;
    Ok(())
}

fn read_daemon(reader: &mut BufReader<UnixStream>) -> Result<DaemonToWrapper, UiError> {
    let mut line = String::new();
    if reader.read_line(&mut line)? == 0 {
        return Err(UiError::Protocol("daemon a fermé la connexion".to_string()));
    }
    decode(line.trim_end()).map_err(|error| UiError::Protocol(error.to_string()))
}

struct HttpRequest {
    method: String,
    path: String,
    query: HashMap<String, String>,
    body: Vec<u8>,
}

fn read_request(stream: &mut TcpStream) -> Result<HttpRequest, UiError> {
    let mut bytes = Vec::new();
    let mut one = [0_u8; 1];
    while bytes.len() < MAX_HTTP_HEADER_BYTES {
        if stream.read(&mut one)? == 0 {
            return Err(UiError::Protocol("requête HTTP incomplète".to_string()));
        }
        bytes.push(one[0]);
        if bytes.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    if !bytes.ends_with(b"\r\n\r\n") {
        return Err(UiError::Protocol(
            "en-têtes HTTP trop volumineux".to_string(),
        ));
    }
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| UiError::Protocol("requête HTTP non UTF-8".to_string()))?;
    let first = text
        .lines()
        .next()
        .ok_or_else(|| UiError::Protocol("ligne HTTP absente".to_string()))?;
    let mut parts = first.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| UiError::Protocol("méthode HTTP absente".to_string()))?
        .to_string();
    let target = parts
        .next()
        .ok_or_else(|| UiError::Protocol("cible HTTP absente".to_string()))?;
    if parts.next().is_none_or(|version| version != "HTTP/1.1") {
        return Err(UiError::Protocol("HTTP/1.1 requis".to_string()));
    }
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    if !path.starts_with('/') || path.contains("..") {
        return Err(UiError::Protocol("chemin HTTP invalide".to_string()));
    }
    let mut headers = HashMap::new();
    for line in text
        .split("\r\n")
        .skip(1)
        .take_while(|line| !line.is_empty())
    {
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| UiError::Protocol("en-tête HTTP invalide".to_string()))?;
        let name = name.trim().to_ascii_lowercase();
        if name.is_empty() || headers.insert(name, value.trim().to_string()).is_some() {
            return Err(UiError::Protocol(
                "en-tête HTTP absent ou dupliqué".to_string(),
            ));
        }
    }
    if headers.contains_key("transfer-encoding") {
        return Err(UiError::Protocol(
            "Transfer-Encoding interdit sur le relais UI".to_string(),
        ));
    }
    let content_length = match headers.get("content-length") {
        Some(value) => value
            .parse::<usize>()
            .map_err(|_| UiError::Protocol("Content-Length invalide".to_string()))?,
        None => 0,
    };
    if content_length > MAX_HTTP_BODY_BYTES {
        return Err(UiError::Protocol("corps HTTP trop volumineux".to_string()));
    }
    let mut body = vec![0_u8; content_length];
    stream.read_exact(&mut body)?;
    Ok(HttpRequest {
        method,
        path: path.to_string(),
        query: parse_query(query)?,
        body,
    })
}

fn parse_query(query: &str) -> Result<HashMap<String, String>, UiError> {
    let mut values = HashMap::new();
    if query.is_empty() {
        return Ok(values);
    }
    for pair in query.split('&') {
        let (key, value) = pair
            .split_once('=')
            .ok_or_else(|| UiError::Protocol("paramètre URL invalide".to_string()))?;
        if key.is_empty()
            || value.is_empty()
            || !key.bytes().all(is_query_byte)
            || !value.bytes().all(is_query_byte)
        {
            return Err(UiError::Protocol("encodage URL interdit".to_string()));
        }
        if values.insert(key.to_string(), value.to_string()).is_some() {
            return Err(UiError::Protocol("paramètre URL dupliqué".to_string()));
        }
    }
    Ok(values)
}

fn is_query_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')
}

fn validate_agent(agent: &str) -> Result<(), UiError> {
    if agent.is_empty()
        || agent.len() > MAX_UI_AGENT_NAME_BYTES
        || !agent
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(UiError::Protocol("nom d'agent UI invalide".to_string()));
    }
    Ok(())
}

fn write_text(stream: &mut TcpStream, status: u16, body: &str) -> Result<(), UiError> {
    write!(
        stream,
        "HTTP/1.1 {status} {}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        status_text(status),
        body.len()
    )?;
    stream.flush()?;
    Ok(())
}

fn write_asset(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> Result<(), UiError> {
    write!(
        stream,
        "HTTP/1.1 {status} {}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status_text(status),
        body.len()
    )?;
    stream.write_all(body)?;
    stream.flush()?;
    Ok(())
}

fn write_json<T: Serialize>(stream: &mut TcpStream, status: u16, value: &T) -> Result<(), UiError> {
    let body = serde_json::to_vec(value)
        .map_err(|error| UiError::Protocol(format!("projection JSON invalide: {error}")))?;
    write!(
        stream,
        "HTTP/1.1 {status} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status_text(status),
        body.len()
    )?;
    stream.write_all(&body)?;
    stream.flush()?;
    Ok(())
}

fn status_text(status: u16) -> &'static str {
    match status {
        200 => "OK",
        202 => "Accepted",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "Error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpStream;
    use std::time::Duration;

    fn agent_info(name: &str, state: &str) -> bridget_transport::protocol::AgentInfo {
        serde_json::from_value(serde_json::json!({
            "name": name,
            "agent_type": "codex",
            "connection_id": "conn-test",
            "host": "test",
            "transport": "unix",
            "os": "linux",
            "state": state,
            "last_seen_secs": 0,
            "reconnect_count": 0
        }))
        .unwrap()
    }

    #[test]
    fn requete_sans_jeton_est_refusee_avant_toute_socket_daemon() {
        let config = UiRelayConfig {
            daemon_socket: PathBuf::from("/tmp/ui-ne-doit-pas-etre-ouvert.sock"),
            maicie_config: PathBuf::from("/tmp/ui-ne-doit-pas-etre-ouvert.json"),
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            token: "jeton-test".to_string(),
        };
        let relay = UiRelay::bind(config).unwrap();
        let address = relay.local_addr().unwrap();
        let worker = thread::spawn(move || relay.serve_one().unwrap());
        let mut client = TcpStream::connect(address).unwrap();
        client
            .write_all(b"GET /v1/snapshot HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        worker.join().unwrap();
        assert!(response.starts_with("HTTP/1.1 403"), "{response}");
    }

    #[test]
    fn page_locale_charge_les_assets_sans_exposer_la_socket_daemon() {
        let config = UiRelayConfig {
            daemon_socket: PathBuf::from("/tmp/ui-page-ne-doit-pas-etre-ouvert.sock"),
            maicie_config: PathBuf::from("/tmp/ui-page-ne-doit-pas-etre-ouvert.json"),
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            token: "jeton-page".to_string(),
        };
        let relay = UiRelay::bind(config).unwrap();
        let address = relay.local_addr().unwrap();
        let worker = thread::spawn(move || relay.serve_one().unwrap());
        let mut client = TcpStream::connect(address).unwrap();
        client
            .write_all(b"GET /?token=jeton-page HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        worker.join().unwrap();
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        assert!(response.contains("/app.js"), "{response}");
        assert!(response.contains("/theme.css"), "{response}");
        assert!(
            !response.contains("bridget.sock"),
            "Mutation : une page qui recevrait la socket Unix contournerait le relais; {response}"
        );
    }

    #[test]
    fn pertes_ui_suivent_le_socket_du_relais_pas_le_home() {
        let path = recovery_losses_path_for_socket(Path::new("/tmp/daemon-test.sock"));
        assert_eq!(path, PathBuf::from("/tmp/recovery-losses.json"));
        assert!(
            !path.components().any(|part| part.as_os_str() == ".config"),
            "un daemon de test ne doit pas lire ~/.config/bridget: {path:?}"
        );
    }

    #[test]
    fn analyse_url_refuse_les_echappements_et_doublons() {
        assert!(parse_query("token=a%2Fb").is_err());
        assert!(parse_query("token=a&token=b").is_err());
        assert_eq!(parse_query("token=abc_123").unwrap()["token"], "abc_123");
    }

    #[test]
    fn garde_destinataire_refuse_absent_et_arrete_sans_refuser_un_agent_vivant() {
        let vivant = agent_info("vivant", "connected");
        let arrete = agent_info("arrete", "stopped");
        let agents = vec![vivant, arrete];
        assert_eq!(
            validate_ui_recipient(&agents, "absent"),
            Err((404, "unknown_recipient"))
        );
        assert_eq!(
            validate_ui_recipient(&agents, "arrete"),
            Err((409, "agent_stopped"))
        );
        assert_eq!(validate_ui_recipient(&agents, "vivant"), Ok(()));
    }
}
