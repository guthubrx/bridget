//! Relais local lecture-seule pour les futures fenêtres Bridget.
//!
//! Le navigateur ne parle jamais à la socket Unix du daemon. Ce module ouvre
//! des connexions ordinaires vers les projections publiques (`ListAgents`,
//! `LedgerProjection`, Attach) puis les traduit en HTTP/SSE loopback.

use bridget_transport::protocol::{AttachWindow, ConnectionRole, LedgerScope, decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use maicie::ui_projection::{UiMissionProjectionV1, read_ui_mission_projection_v1};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::thread;

const UI_VERSION: u8 = 1;
const MAX_HTTP_HEADER_BYTES: usize = 16 * 1024;
const MAX_UI_AGENT_NAME_BYTES: usize = 100;
const MAX_UI_SSE_EVENTS: usize = 20_000;

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
    agents: Vec<bridget_transport::protocol::AgentInfo>,
    open_requests: Vec<bridget_transport::protocol::RequestInfo>,
    missions: UiMissionProjectionV1,
}

#[derive(Serialize)]
struct UiJournalEventV1<'a> {
    version: u8,
    event: &'a DaemonToWrapper,
}

fn serve_connection(stream: &mut TcpStream, config: &UiRelayConfig) -> Result<(), UiError> {
    let request = read_request(stream)?;
    if request.method != "GET" {
        return write_text(stream, 405, "méthode non autorisée");
    }
    if request.query.get("token") != Some(&config.token) {
        return write_text(stream, 403, "jeton UI invalide");
    }
    match request.path.as_str() {
        "/" => write_text(stream, 200, "Bridget UI relay ready\n"),
        "/v1/snapshot" => {
            let snapshot = read_snapshot(config)?;
            write_json(stream, 200, &snapshot)
        }
        "/v1/journal" => {
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
        "/v1/watch" => {
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

fn read_snapshot(config: &UiRelayConfig) -> Result<UiSnapshotV1, UiError> {
    let (agents, open_requests) = read_bridget_snapshot(&config.daemon_socket)?;
    let missions = read_ui_mission_projection_v1(&config.maicie_config)
        .map_err(|error| UiError::Configuration(error.to_string()))?;
    Ok(UiSnapshotV1 {
        version: UI_VERSION,
        agents,
        open_requests,
        missions,
    })
}

/// Projection globale déjà détenue par le daemon. La connexion reste dans le
/// rôle wrapper historique, lequel autorise ces deux lectures sans créer une
/// présence temporaire dans l'annuaire.
fn read_bridget_snapshot(
    socket_path: &Path,
) -> Result<
    (
        Vec<bridget_transport::protocol::AgentInfo>,
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
            scope: LedgerScope::Requests,
            limit: 200,
        },
    )?;
    let open_requests = match read_daemon(&mut reader)? {
        DaemonToWrapper::LedgerProjection { requests, .. } => requests,
        response => {
            return Err(UiError::Protocol(format!(
                "LedgerProjection attendu, reçu {response:?}"
            )));
        }
    };
    Ok((agents, open_requests))
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
    write!(
        http,
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n"
    )?;
    http.flush()?;
    if let Some(config) = snapshot_config {
        let snapshot = read_snapshot(config)?;
        write_sse(http, "snapshot", &snapshot)?;
    }
    write_sse(
        http,
        "journal",
        &UiJournalEventV1 {
            version: UI_VERSION,
            event: &subscribed,
        },
    )?;
    for _ in 0..MAX_UI_SSE_EVENTS {
        let event = read_daemon(&mut reader)?;
        write_sse(
            http,
            "journal",
            &UiJournalEventV1 {
                version: UI_VERSION,
                event: &event,
            },
        )?;
        if matches!(event, DaemonToWrapper::End { .. }) {
            break;
        }
    }
    let _ = http.shutdown(Shutdown::Both);
    Ok(())
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
    Ok(HttpRequest {
        method,
        path: path.to_string(),
        query: parse_query(query)?,
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
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpStream;
    use std::time::Duration;

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
    fn analyse_url_refuse_les_echappements_et_doublons() {
        assert!(parse_query("token=a%2Fb").is_err());
        assert!(parse_query("token=a&token=b").is_err());
        assert_eq!(parse_query("token=abc_123").unwrap()["token"], "abc_123");
    }
}
