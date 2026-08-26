//! Relais local du poste de travail Bridget.
//!
//! Le navigateur ne parle jamais à la socket Unix du daemon. Ce module ouvre
//! des connexions ordinaires vers les projections publiques (`ListAgents`,
//! `LedgerProjection`, Attach) puis les traduit en HTTP/SSE loopback.

use bridget_core::BridgetMessage;
use bridget_transport::protocol::{
    AttachWindow, CLIENT_CONTRACT_VERSION, ClientCapability, ConnectionRole, IdempotencyIssue,
    LedgerMessage, LedgerScope, PresenceMode, decode, encode,
};
use bridget_transport::journal::valid_events;
use bridget_transport::{ChannelReport, DaemonToWrapper, WrapperToDaemon};
use maicie::ui_projection::{
    UiMissionProjectionV1, read_ui_mission_projection_v1, retain_living_objectives,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io::{BufRead, BufReader, BufWriter, ErrorKind, Read, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const UI_VERSION: u8 = 1;
const MAX_HTTP_HEADER_BYTES: usize = 16 * 1024;
const MAX_HTTP_BODY_BYTES: usize = 64 * 1024;
const MAX_UI_AGENT_NAME_BYTES: usize = 100;
const MAX_UI_SSE_EVENTS: usize = 20_000;
const MAX_UI_RECONNECT_ATTEMPTS: usize = 50;
const UI_RECONNECT_DELAY: Duration = Duration::from_millis(100);
/// Relecture ledger pendant un watch : le sortant référent→humain n'apparaît
/// jamais au journal ; sans ce rythme le fil reste figé après l'ouverture.
const UI_THREAD_LEDGER_POLL: Duration = Duration::from_millis(400);
const UI_SENDER: &str = "humain";
/// Ouverture page : derniers enregistrements complets (fragments d'un même seq à bord).
const UI_JOURNAL_OPEN_COMPLETE_RECORDS: usize = 10;
/// Remontée manuelle : pages d'enregistrements complets via `from_seq`.
const UI_JOURNAL_OLDER_PAGE_RECORDS: usize = 20;
/// Plafond dur de fragments SSE rejoués avant CaughtUp — un enregistrement
/// pathologique ne doit pas renvoyer le mégaoctet.
const UI_JOURNAL_MAX_REPLAY_FRAGMENTS: usize = 150;
/// Port loopback par défaut : stable d'un lancement à l'autre, sans option CLI.
pub const DEFAULT_UI_PORT: u16 = 17888;
const UI_ENDPOINT_STATE_VERSION: u8 = 1;
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct UiEndpointStateFile {
    version: u8,
    port: u16,
    token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiEndpoint {
    pub port: u16,
    pub token: String,
}

/// Chemin de l'état local port+jeton (hors dépôt, permissions restrictives).
pub fn ui_endpoint_state_path() -> PathBuf {
    crate::daemon::DaemonConfig::default()
        .socket_path
        .parent()
        .map(|parent| parent.join("ui-endpoint.json"))
        .unwrap_or_else(|| PathBuf::from("/tmp/bridget-ui-endpoint.json"))
}

fn write_ui_endpoint_state(path: &Path, endpoint: &UiEndpoint) -> Result<(), UiError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700));
        }
    }
    let payload = UiEndpointStateFile {
        version: UI_ENDPOINT_STATE_VERSION,
        port: endpoint.port,
        token: endpoint.token.clone(),
    };
    let body = serde_json::to_vec_pretty(&payload).map_err(|error| {
        UiError::Configuration(format!("sérialisation de l'endpoint UI: {error}"))
    })?;
    // Résidu de plantage : un temporaire jamais renommé n'a jamais été validé.
    // On le retire, puis création EXCLUSIVE en 0o600 (pas create+truncate qui
    // hériterait de droits ouverts si le fichier existait encore).
    let tmp = path.with_extension("json.tmp");
    let _ = std::fs::remove_file(&tmp);
    {
        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            let mut file = match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&tmp)
            {
                Ok(file) => file,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    let absolute = std::fs::canonicalize(&tmp)
                        .unwrap_or_else(|_| tmp.clone());
                    return Err(UiError::Configuration(format!(
                        "fichier temporaire d'endpoint UI déjà présent — supprimez ce fichier puis relancez le relais : {}",
                        absolute.display()
                    )));
                }
                Err(error) => return Err(UiError::Io(error)),
            };
            file.write_all(&body)?;
            file.sync_all()?;
        }
        #[cfg(not(unix))]
        {
            let _ = std::fs::remove_file(&tmp);
            if tmp.exists() {
                let absolute = std::fs::canonicalize(&tmp).unwrap_or_else(|_| tmp.clone());
                return Err(UiError::Configuration(format!(
                    "fichier temporaire d'endpoint UI déjà présent — supprimez ce fichier puis relancez le relais : {}",
                    absolute.display()
                )));
            }
            std::fs::write(&tmp, &body)?;
        }
    }
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Charge l'endpoint persistant, ou le crée une seule fois (port défaut + jeton neuf).
pub fn load_or_create_ui_endpoint(
    path: &Path,
    default_port: u16,
) -> Result<UiEndpoint, UiError> {
    if path.exists() {
        return load_ui_endpoint(path);
    }
    let endpoint = UiEndpoint {
        port: default_port,
        token: uuid::Uuid::new_v4().simple().to_string(),
    };
    write_ui_endpoint_state(path, &endpoint)?;
    Ok(endpoint)
}

/// Lit l'endpoint déjà persisté — ne tire jamais un jeton au hasard.
pub fn load_ui_endpoint(path: &Path) -> Result<UiEndpoint, UiError> {
    let raw = std::fs::read_to_string(path)?;
    let parsed: UiEndpointStateFile = serde_json::from_str(&raw).map_err(|error| {
        UiError::Configuration(format!(
            "état UI illisible ({}) : {error}",
            path.display()
        ))
    })?;
    if parsed.version != UI_ENDPOINT_STATE_VERSION {
        return Err(UiError::Configuration(format!(
            "version d'état UI inconnue ({}) dans {}",
            parsed.version,
            path.display()
        )));
    }
    if parsed.port == 0 {
        return Err(UiError::Configuration(
            "port UI invalide (0) dans l'état local — corrigez ou supprimez le fichier d'endpoint"
                .to_string(),
        ));
    }
    if parsed.token.is_empty() {
        return Err(UiError::Configuration(
            "jeton UI vide dans l'état local".to_string(),
        ));
    }
    Ok(UiEndpoint {
        port: parsed.port,
        token: parsed.token,
    })
}

impl UiRelayConfig {
    pub fn loopback(daemon_socket: PathBuf, maicie_config: PathBuf) -> Result<Self, UiError> {
        Self::loopback_with_endpoint_path(
            daemon_socket,
            maicie_config,
            &ui_endpoint_state_path(),
            DEFAULT_UI_PORT,
        )
    }

    pub fn loopback_with_endpoint_path(
        daemon_socket: PathBuf,
        maicie_config: PathBuf,
        endpoint_path: &Path,
        default_port: u16,
    ) -> Result<Self, UiError> {
        let endpoint = load_or_create_ui_endpoint(endpoint_path, default_port)?;
        Ok(Self {
            daemon_socket,
            maicie_config,
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, endpoint.port)),
            token: endpoint.token,
        })
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

struct UiRelayRuntime {
    human_presence: Mutex<Option<UiHumanPresence>>,
    human_presence_channel: Option<String>,
}

struct UiHumanPresence {
    alive: Arc<AtomicBool>,
}

impl UiRelayRuntime {
    fn new(human_presence_channel: Option<String>) -> Self {
        Self {
            human_presence: Mutex::new(None),
            human_presence_channel,
        }
    }

    fn ensure_human_presence(&self, socket_path: &Path) -> Result<(), UiError> {
        let mut presence = self
            .human_presence
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if presence
            .as_ref()
            .is_some_and(|current| current.alive.load(Ordering::Acquire))
        {
            return Ok(());
        }
        *presence = Some(open_human_presence(
            socket_path,
            self.human_presence_channel.as_deref(),
        )?);
        Ok(())
    }
}

fn open_human_presence(
    socket_path: &Path,
    attested_channel: Option<&str>,
) -> Result<UiHumanPresence, UiError> {
    let stream = UnixStream::connect(socket_path)?;
    let read_stream = stream.try_clone()?;
    let writer = Arc::new(Mutex::new(BufWriter::new(stream)));
    let mut reader = BufReader::new(read_stream);
    {
        let mut writer_guard = writer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        send_daemon(
            &mut writer_guard,
            &WrapperToDaemon::Register {
                agent_type: "ui".to_string(),
                name: Some(UI_SENDER.to_string()),
                host: Some("localhost".to_string()),
                transport: None,
                channel: ChannelReport::reported(attested_channel.map(str::to_owned)),
                mode: Some(PresenceMode::Cli),
                location: None,
                os: Some(std::env::consts::OS.to_string()),
                instance_id: Some(format!("bridget-ui-{}", std::process::id())),
                domain: Some("bridget".to_string()),
                turn_in_progress: false,
                journal_available: Some(false),
            },
        )?;
    }
    match read_daemon(&mut reader)? {
        DaemonToWrapper::Registered { name } if name == UI_SENDER => {}
        response => {
            return Err(UiError::Protocol(format!(
                "présence UI humaine refusée: {response:?}"
            )));
        }
    }
    {
        let mut writer_guard = writer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        send_daemon(&mut writer_guard, &WrapperToDaemon::JournalReady)?;
    }

    let alive = Arc::new(AtomicBool::new(true));
    let thread_alive = Arc::clone(&alive);
    thread::spawn(move || {
        while let Ok(event) = read_daemon(&mut reader) {
            match event {
                DaemonToWrapper::DeliverIdempotent {
                    delivery_id,
                    delivery_generation,
                    ..
                } => {
                    let mut writer = writer
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    if send_daemon(
                        &mut writer,
                        &WrapperToDaemon::DeliverAcked {
                            delivery_id,
                            delivery_generation,
                        },
                    )
                    .is_err()
                    {
                        break;
                    }
                }
                DaemonToWrapper::Disconnect => break,
                _ => {}
            }
        }
        thread_alive.store(false, Ordering::Release);
    });
    Ok(UiHumanPresence { alive })
}

/// Serveur HTTP local sans état métier propre.
pub struct UiRelay {
    listener: TcpListener,
    config: UiRelayConfig,
    runtime: Arc<UiRelayRuntime>,
}

impl UiRelay {
    pub fn bind(config: UiRelayConfig) -> Result<Self, UiError> {
        Self::bind_with_attested_channel(config, None)
    }

    fn bind_with_attested_channel(
        config: UiRelayConfig,
        attested_channel: Option<String>,
    ) -> Result<Self, UiError> {
        if !config.bind.ip().is_loopback() {
            return Err(UiError::Configuration(
                "le relais UI doit écouter exclusivement sur la boucle locale".to_string(),
            ));
        }
        if config.token.is_empty() {
            return Err(UiError::Configuration("jeton UI absent".to_string()));
        }
        let listener = TcpListener::bind(config.bind).map_err(|error| {
            if error.kind() == std::io::ErrorKind::AddrInUse {
                UiError::Configuration(format!(
                    "le port {} est déjà pris — arrêtez l'autre relais UI qui l'occupe ; aucun repli sur un port au hasard",
                    config.bind.port()
                ))
            } else {
                UiError::Io(error)
            }
        })?;
        Ok(Self {
            listener,
            config,
            runtime: Arc::new(UiRelayRuntime::new(attested_channel)),
        })
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
                    let runtime = Arc::clone(&self.runtime);
                    thread::spawn(move || {
                        let mut stream = stream;
                        if let Err(error) = serve_connection(&mut stream, &config, &runtime) {
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
        serve_connection(&mut stream, &self.config, &self.runtime)
    }
}

/// Lance `bridget ui`. Port et jeton sont stables d'un redémarrage à l'autre
/// (état local hors dépôt). Le jeton ne figure jamais dans le HTML servi.
pub fn run(daemon_socket: PathBuf, maicie_config: PathBuf) -> Result<(), UiError> {
    let config = UiRelayConfig::loopback(daemon_socket, maicie_config)?;
    let relay = UiRelay::bind_with_attested_channel(
        config,
        crate::connection_channel::attested_connection_channel(),
    )?;
    // VOLONTAIRE : l'URL complète (avec jeton) est la livraison à l'utilisateur.
    // Ne pas retirer. Le jeton ne doit en revanche jamais apparaître dans un
    // message d'erreur ni une trace de diagnostic — seulement ici, sur stdout.
    println!("Bridget UI (lecture et envoi) : {}", relay.url()?);
    relay.serve()
}

#[derive(Serialize)]
struct UiSnapshotV1 {
    version: u8,
    agents: Vec<UiAgentRowV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    peer_exchanges: Option<Vec<UiPeerExchangeV1>>,
    /// Bulles utilisateur↔agent focal — corps issus du ledger (pas du journal).
    /// Absentes hors focus ; présentes (éventuellement vides) dès qu'un agent est ciblé.
    #[serde(skip_serializing_if = "Option::is_none")]
    thread_messages: Option<Vec<UiThreadMessageV1>>,
    open_requests: Vec<bridget_transport::protocol::RequestInfo>,
    missions: UiMissionProjectionV1,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    recovery_losses: Vec<UiRecoveryLossV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum UiThreadRoleV1 {
    User,
    Agent,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct UiThreadMessageV1 {
    version: u8,
    kind: &'static str,
    at: i64,
    role: UiThreadRoleV1,
    text: String,
    delivery_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum UiPeerDirectionV1 {
    In,
    Out,
    Both,
}

#[derive(Debug, Clone, Serialize)]
struct UiPeerExchangeV1 {
    version: u8,
    kind: &'static str,
    at: i64,
    peer: String,
    direction: UiPeerDirectionV1,
    count: usize,
    delivery_ids: Vec<String>,
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UiSearchRequestV1 {
    version: u8,
    q: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct UiSearchHitV1 {
    id: String,
    ts: i64,
    sender: String,
    target: String,
    body: String,
}

#[derive(Debug, Serialize)]
struct UiSearchResponseV1 {
    version: u8,
    hits: Vec<UiSearchHitV1>,
    truncated: bool,
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

#[derive(Debug, Clone, Copy, Serialize)]
struct UiJournalPageV1 {
    version: u8,
    kind: &'static str,
    has_more: bool,
    from_seq: u64,
}

#[derive(Debug, Clone, Copy)]
struct UiJournalPage {
    has_more: bool,
    from_seq: u64,
}

fn serve_connection(
    stream: &mut TcpStream,
    config: &UiRelayConfig,
    runtime: &UiRelayRuntime,
) -> Result<(), UiError> {
    let request = read_request(stream)?;
    if request.path == "/v1/send" && request.method != "POST" {
        return write_text(stream, 405, "méthode non autorisée");
    }
    if request.method != "GET" && request.method != "POST" {
        return write_text(stream, 405, "méthode non autorisée");
    }
    if request.method == "GET" {
        match request.path.as_str() {
            "/app.js" => {
                return write_asset(
                    stream,
                    200,
                    "application/javascript; charset=utf-8",
                    UI_SCRIPT,
                );
            }
            "/theme.css" => {
                return write_asset(stream, 200, "text/css; charset=utf-8", UI_THEME);
            }
            _ => {}
        }
    }
    if request.query.get("token") != Some(&config.token) {
        return write_text(stream, 403, "jeton UI invalide");
    }
    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/") => write_asset(stream, 200, "text/html; charset=utf-8", UI_INDEX),
        ("POST", "/v1/send") => match post_ui_message(config, runtime, &request.body) {
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
        ("POST", "/v1/search") => match post_ui_search(config, &request.body) {
            Ok(response) => write_json(stream, 200, &response),
            Err((status, message)) => write_text(stream, status, message),
        },
        ("GET", "/v1/snapshot") => {
            let focus_agent = request.query.get("agent").map(String::as_str);
            if let Some(agent) = focus_agent {
                validate_agent(agent)?;
            }
            let snapshot = read_snapshot(config, focus_agent)?;
            write_json(stream, 200, &snapshot)
        }
        ("GET", "/v1/journal") => {
            let agent = request
                .query
                .get("agent")
                .ok_or_else(|| UiError::Protocol("paramètre agent absent".to_string()))?;
            validate_agent(agent)?;
            let (window, page) = resolve_ui_journal_window(agent, &request.query)?;
            stream_sse_journal(stream, &config.daemon_socket, agent, window, page, None)
        }
        ("GET", "/v1/watch") => {
            let agent = request
                .query
                .get("agent")
                .ok_or_else(|| UiError::Protocol("paramètre agent absent".to_string()))?;
            validate_agent(agent)?;
            let (window, page) = resolve_ui_journal_window(agent, &request.query)?;
            // La vue combinée est la porte d'entrée de la future page : elle
            // raccorde Attach avant de capturer l'instantané, donc aucun delta
            // journal ne peut se glisser silencieusement entre les deux.
            stream_sse_journal(stream, &config.daemon_socket, agent, window, page, Some(config))
        }
        _ => write_text(stream, 404, "ressource UI inconnue"),
    }
}

fn post_ui_message(
    config: &UiRelayConfig,
    runtime: &UiRelayRuntime,
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
    if request.reply {
        runtime
            .ensure_human_presence(&config.daemon_socket)
            .map_err(|_| (503, "daemon_unavailable"))?;
    }
    send_ui_message(&config.daemon_socket, request).map_err(|_| (503, "daemon_unavailable"))
}

fn post_ui_search(
    config: &UiRelayConfig,
    body: &[u8],
) -> Result<UiSearchResponseV1, (u16, &'static str)> {
    let request: UiSearchRequestV1 =
        serde_json::from_slice(body).map_err(|_| (400, "requête de recherche invalide"))?;
    if request.version != UI_VERSION {
        return Err((400, "requête de recherche invalide"));
    }
    let db_path = ledger_db_path_for_socket(&config.daemon_socket);
    let outcome =
        search_ui_ledger(&db_path, &request.q).map_err(|_| (503, "ledger indisponible"))?;
    Ok(UiSearchResponseV1 {
        version: UI_VERSION,
        hits: outcome.hits,
        truncated: outcome.truncated,
    })
}

#[derive(Debug)]
struct UiSearchOutcome {
    hits: Vec<UiSearchHitV1>,
    truncated: bool,
}

/// Chemin réel emprunté par la page : lit le store à côté de la socket daemon
/// (pas de nouveau RPC — la flotte tourne sans redémarrage).
fn search_ui_ledger(db_path: &Path, query: &str) -> Result<UiSearchOutcome, UiError> {
    let store = crate::store::Store::open(db_path)
        .map_err(|error| UiError::Configuration(error.to_string()))?;
    let outcome = store
        .search_messages(query, crate::store::MAX_LEDGER_SEARCH_PUBLIC)
        .map_err(|error| UiError::Configuration(error.to_string()))?;
    Ok(UiSearchOutcome {
        hits: outcome
            .hits
            .into_iter()
            .map(|entry| UiSearchHitV1 {
                id: entry.id,
                ts: entry.ts,
                sender: entry.sender,
                target: entry.target,
                body: entry.body,
            })
            .collect(),
        truncated: outcome.truncated,
    })
}

fn ledger_db_path_for_socket(socket_path: &Path) -> PathBuf {
    socket_path.with_extension("db")
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

/// Fenêtre d'ouverture : Seq(from) calculé **dans le relais** sur les jsonl
/// de l'agent — jamais Tail/Today sur le fil Attach (daemon live inchangé).
/// Un mutant qui remet Seq(0) ou Tail ici doit tuer
/// `chemin_productif_ouverture_journal_emprunte_fenetre_relais`.
fn resolve_ui_journal_window(
    agent: &str,
    query: &HashMap<String, String>,
) -> Result<(AttachWindow, UiJournalPage), UiError> {
    resolve_ui_journal_window_in(agent_journal_dir(agent), query)
}

fn agent_journal_dir(agent: &str) -> PathBuf {
    let root = std::env::var("HOME")
        .map(|home| PathBuf::from(home).join(".cache").join("bridget").join("sessions"))
        .unwrap_or_else(|_| PathBuf::from("/tmp/bridget/sessions"));
    root.join(agent)
}

fn complete_record_seqs(journal_dir: &Path) -> Vec<u64> {
    let mut seqs = std::fs::read_dir(journal_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "jsonl"))
        .flat_map(|path| valid_events(&path))
        .filter_map(|value| value.get("seq").and_then(|seq| seq.as_u64()))
        .collect::<Vec<_>>();
    seqs.sort_unstable();
    seqs.dedup();
    seqs
}

fn open_window_from_complete_records(seqs: &[u64], count: usize) -> (u64, bool) {
    if seqs.is_empty() {
        return (0, false);
    }
    if seqs.len() <= count {
        return (seqs[0], false);
    }
    (seqs[seqs.len() - count], true)
}

fn older_page_from_complete_records(seqs: &[u64], current_from: u64, page: usize) -> u64 {
    let older: Vec<u64> = seqs
        .iter()
        .copied()
        .filter(|seq| *seq < current_from)
        .collect();
    open_window_from_complete_records(&older, page).0
}

fn resolve_ui_journal_window_in(
    journal_dir: PathBuf,
    query: &HashMap<String, String>,
) -> Result<(AttachWindow, UiJournalPage), UiError> {
    let seqs = complete_record_seqs(&journal_dir);
    match query.get("from_seq") {
        Some(value) => {
            let from_seq = value.parse().map_err(|_| {
                UiError::Protocol("from_seq doit être un entier non signé".to_string())
            })?;
            let first = seqs.first().copied().unwrap_or(0);
            Ok((
                AttachWindow::Seq(from_seq),
                UiJournalPage {
                    has_more: from_seq > first,
                    from_seq,
                },
            ))
        }
        None => {
            let (from_seq, has_more) =
                open_window_from_complete_records(&seqs, UI_JOURNAL_OPEN_COMPLETE_RECORDS);
            Ok((
                AttachWindow::Seq(from_seq),
                UiJournalPage {
                    has_more,
                    from_seq,
                },
            ))
        }
    }
}

/// Remontée manuelle : page précédente de `UI_JOURNAL_OLDER_PAGE_RECORDS`.
#[cfg_attr(not(test), allow(dead_code))]
fn older_journal_page_from_seq(current_from_seq: u64) -> u64 {
    current_from_seq.saturating_sub(UI_JOURNAL_OLDER_PAGE_RECORDS as u64)
}

fn read_snapshot(
    config: &UiRelayConfig,
    focus_agent: Option<&str>,
) -> Result<UiSnapshotV1, UiError> {
    let facts = read_bridget_snapshot(&config.daemon_socket)?;
    let agents = compose_agent_rows(facts.agents, &facts.messages);
    let peer_exchanges = focus_agent.map(|agent| aggregate_peer_exchanges(agent, &facts.messages));
    // Chemin productif du fil humain↔référent : mêmes messages ledger que
    // peer_exchanges, mais SANS exclure UI_SENDER — le journal d'agent ne porte
    // pas les sorties vers l'utilisateur (DETTE : asymétrie journal, voir
    // human_referent_thread_messages). peer_exchange reste agent↔agent.
    let thread_messages =
        focus_agent.map(|agent| human_referent_thread_messages(agent, &facts.messages));
    // Chemin productif missions page : filtre vivants — un mutant qui retire
    // cet appel dans read_snapshot doit tuer le témoin
    // `chemin_productif_snapshot_emprunte_retain_living_objectives`.
    let missions = retain_living_objectives(
        read_ui_mission_projection_v1(&config.maicie_config)
            .map_err(|error| UiError::Configuration(error.to_string()))?,
    );
    let recovery_losses = read_recovery_losses(&config.daemon_socket);
    Ok(UiSnapshotV1 {
        version: UI_VERSION,
        agents,
        peer_exchanges,
        thread_messages,
        open_requests: facts.open_requests,
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

fn aggregate_peer_exchanges(
    focus_agent: &str,
    messages: &[LedgerMessage],
) -> Vec<UiPeerExchangeV1> {
    let mut ordered = messages
        .iter()
        .filter(|message| message.sender == focus_agent || message.target == focus_agent)
        .collect::<Vec<_>>();
    ordered.sort_by(|left, right| (left.ts, &left.id).cmp(&(right.ts, &right.id)));

    let mut exchanges = Vec::new();
    let mut current: Option<UiPeerExchangeV1> = None;
    for message in ordered {
        let Some((peer, direction)) = peer_direction(focus_agent, message) else {
            if let Some(exchange) = current.take() {
                exchanges.push(exchange);
            }
            continue;
        };
        if current
            .as_ref()
            .is_some_and(|exchange| exchange.peer == peer)
        {
            let exchange = current.as_mut().expect("échange courant vérifié");
            if exchange.direction != direction {
                exchange.direction = UiPeerDirectionV1::Both;
            }
            exchange.count += 1;
            exchange.delivery_ids.push(message.id.clone());
            continue;
        }
        if let Some(exchange) = current.replace(UiPeerExchangeV1 {
            version: UI_VERSION,
            kind: "peer_exchange",
            at: message.ts,
            peer: peer.to_string(),
            direction,
            count: 1,
            delivery_ids: vec![message.id.clone()],
        }) {
            exchanges.push(exchange);
        }
    }
    if let Some(exchange) = current {
        exchanges.push(exchange);
    }
    exchanges
}

fn peer_direction<'a>(
    focus_agent: &str,
    message: &'a LedgerMessage,
) -> Option<(&'a str, UiPeerDirectionV1)> {
    if message.sender == focus_agent && message.target != focus_agent && message.target != UI_SENDER
    {
        return Some((&message.target, UiPeerDirectionV1::Out));
    }
    if message.target == focus_agent && message.sender != focus_agent && message.sender != UI_SENDER
    {
        return Some((&message.sender, UiPeerDirectionV1::In));
    }
    None
}

/// Messages ledger entre l'utilisateur (`humain`) et l'agent focal — dans l'ordre
/// d'émission, avec corps. C'est le chemin que `/v1/snapshot` et le SSE empruntent
/// pour le fil ; un mutant qui coupe cet appel dans `read_snapshot` doit tuer le
/// témoin `chemin_productif_snapshot_emprunte_human_referent_thread_messages`.
///
/// # Dette — journal asymétrique (non corrigée ici)
///
/// Le journal d'agent n'enregistre que les livraisons *entrantes*
/// (`record_interactive_turn` sur `Deliver`). Un message référent→humain apparaît
/// au ledger mais **pas** comme événement propre dans le journal. Ce lot projette
/// donc le fil depuis le ledger (les deux sens). Quiconque s'appuiera demain sur
/// le journal seul pour « la conversation utilisateur » retombera dans le piège
/// d'un fil à sens unique. Corriger la journalisation du sortant est un autre lot.
///
/// `peer_direction` continue d'exclure `UI_SENDER` : `peer_exchange` reste
/// agent↔agent (spec 032) ; ce canal est une projection distincte.
fn human_referent_thread_messages(
    focus_agent: &str,
    messages: &[LedgerMessage],
) -> Vec<UiThreadMessageV1> {
    let mut selected = messages
        .iter()
        .filter(|message| {
            (message.sender == UI_SENDER && message.target == focus_agent)
                || (message.sender == focus_agent && message.target == UI_SENDER)
        })
        .collect::<Vec<_>>();
    selected.sort_by(|left, right| (left.ts, &left.id).cmp(&(right.ts, &right.id)));
    selected
        .into_iter()
        .map(|message| {
            let role = if message.sender == UI_SENDER {
                UiThreadRoleV1::User
            } else {
                UiThreadRoleV1::Agent
            };
            UiThreadMessageV1 {
                version: UI_VERSION,
                kind: "thread_message",
                at: message.ts,
                role,
                text: message.body.clone(),
                delivery_id: message.id.clone(),
            }
        })
        .collect()
}

/// Chemin vivant du fil : relecture ledger pendant un `/v1/watch` ouvert.
/// Émet seulement les `delivery_id` encore inconnus. Un mutant qui vide cette
/// fonction doit tuer `watch_pousse_thread_message_sortant_apres_ouverture`.
fn push_live_thread_messages(
    http: &mut TcpStream,
    socket_path: &Path,
    focus_agent: &str,
    seen: &mut HashSet<String>,
) -> Result<(), UiError> {
    let messages = read_ledger_messages(socket_path)?;
    for message in human_referent_thread_messages(focus_agent, &messages) {
        if seen.insert(message.delivery_id.clone()) {
            write_sse(http, "thread_message", &message)?;
        }
    }
    Ok(())
}

fn seed_thread_message_ids(snapshot: &UiSnapshotV1, seen: &mut HashSet<String>) {
    if let Some(messages) = &snapshot.thread_messages {
        for message in messages {
            seen.insert(message.delivery_id.clone());
        }
    }
}

fn read_ledger_messages(socket_path: &Path) -> Result<Vec<LedgerMessage>, UiError> {
    let stream = UnixStream::connect(socket_path)?;
    let read_stream = stream.try_clone()?;
    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(read_stream);
    send_daemon(
        &mut writer,
        &WrapperToDaemon::LedgerProjection {
            scope: LedgerScope::Both,
            limit: 200,
        },
    )?;
    match read_daemon(&mut reader)? {
        DaemonToWrapper::LedgerProjection { messages, .. } => Ok(messages),
        response => Err(UiError::Protocol(format!(
            "LedgerProjection attendu, reçu {response:?}"
        ))),
    }
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
struct BridgetSnapshotFacts {
    agents: Vec<bridget_transport::protocol::AgentInfo>,
    messages: Vec<LedgerMessage>,
    open_requests: Vec<bridget_transport::protocol::RequestInfo>,
}

fn read_bridget_snapshot(socket_path: &Path) -> Result<BridgetSnapshotFacts, UiError> {
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
    Ok(BridgetSnapshotFacts {
        agents,
        messages,
        open_requests,
    })
}

/// Traduit un unique abonnement Attach existant vers SSE. L'abonnement est
/// ouvert avant le premier événement SSE : le client reçoit donc exactement
/// le rejeu/cursor, puis `SnapshotCaughtUp`, puis le live déjà garanti par
/// Bridget, sans seconde source de journal.
///
/// En parallèle, quand `snapshot_config` est fourni (`/v1/watch`), une relecture
/// périodique du ledger pousse les `thread_message` nouveaux — le journal seul
/// ne porte pas le sortant vers l'utilisateur.
fn stream_sse_journal(
    http: &mut TcpStream,
    socket_path: &Path,
    agent: &str,
    window: AttachWindow,
    page: UiJournalPage,
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
    let initial_snapshot = snapshot_config
        .map(|config| read_snapshot(config, Some(agent)))
        .transpose()?;
    if snapshot_config.is_some() {
        write_relay_state(http, "connected", now_secs())?;
    }
    let mut seen_thread_ids = HashSet::new();
    if let Some(snapshot) = &initial_snapshot {
        seed_thread_message_ids(snapshot, &mut seen_thread_ids);
        write_snapshot_sse(http, snapshot)?;
    }
    write_sse(
        http,
        "journal_page",
        &UiJournalPageV1 {
            version: UI_VERSION,
            kind: "journal_page",
            has_more: page.has_more,
            from_seq: page.from_seq,
        },
    )?;
    write_sse(
        http,
        "journal",
        &UiJournalEventV1 {
            version: UI_VERSION,
            event: &session.subscribed,
        },
    )?;

    if snapshot_config.is_some() {
        let _ = session
            .reader
            .get_ref()
            .set_read_timeout(Some(UI_THREAD_LEDGER_POLL));
    }
    let mut last_thread_poll = Instant::now()
        .checked_sub(UI_THREAD_LEDGER_POLL)
        .unwrap_or_else(Instant::now);
    let mut last_seq = None;
    let mut events = 0_usize;
    let mut replay_caught_up = false;
    let mut replay_fragments = 0_usize;
    let mut page_has_more = page.has_more;
    while events < MAX_UI_SSE_EVENTS {
        let event = match read_daemon(&mut session.reader) {
            Ok(event) => event,
            Err(UiError::Io(error))
                if snapshot_config.is_some()
                    && matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) =>
            {
                if let Some(config) = snapshot_config {
                    // Chemin vivant productif — ne pas retirer sans tuer le témoin live.
                    let _ = push_live_thread_messages(
                        http,
                        &config.daemon_socket,
                        agent,
                        &mut seen_thread_ids,
                    );
                    last_thread_poll = Instant::now();
                }
                continue;
            }
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
                let restored_snapshot = snapshot_config
                    .map(|config| read_snapshot(config, Some(agent)))
                    .transpose()?;
                write_relay_state(http, "connected", now_secs())?;
                if let Some(snapshot) = &restored_snapshot {
                    seed_thread_message_ids(snapshot, &mut seen_thread_ids);
                    write_snapshot_sse(http, snapshot)?;
                }
                write_sse(
                    http,
                    "journal",
                    &UiJournalEventV1 {
                        version: UI_VERSION,
                        event: &session.subscribed,
                    },
                )?;
                let _ = session
                    .reader
                    .get_ref()
                    .set_read_timeout(Some(UI_THREAD_LEDGER_POLL));
                last_thread_poll = Instant::now()
                    .checked_sub(UI_THREAD_LEDGER_POLL)
                    .unwrap_or_else(Instant::now);
                continue;
            }
            Err(error) => return Err(error),
        };
        last_seq = event_resume_seq(&event).or(last_seq);
        let is_replay_fragment =
            !replay_caught_up && matches!(event, DaemonToWrapper::JournalFragment { .. });
        if is_replay_fragment {
            replay_fragments += 1;
            if replay_fragments > UI_JOURNAL_MAX_REPLAY_FRAGMENTS {
                page_has_more = true;
                events += 1;
                continue;
            }
        }
        if matches!(event, DaemonToWrapper::SnapshotCaughtUp { .. }) {
            replay_caught_up = true;
            if page_has_more && !page.has_more {
                write_sse(
                    http,
                    "journal_page",
                    &UiJournalPageV1 {
                        version: UI_VERSION,
                        kind: "journal_page",
                        has_more: true,
                        from_seq: page.from_seq,
                    },
                )?;
            }
        }
        write_sse(
            http,
            "journal",
            &UiJournalEventV1 {
                version: UI_VERSION,
                event: &event,
            },
        )?;
        events += 1;
        if let Some(config) = snapshot_config {
            if last_thread_poll.elapsed() >= UI_THREAD_LEDGER_POLL {
                let _ = push_live_thread_messages(
                    http,
                    &config.daemon_socket,
                    agent,
                    &mut seen_thread_ids,
                );
                last_thread_poll = Instant::now();
            }
        }
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

fn write_snapshot_sse(http: &mut TcpStream, snapshot: &UiSnapshotV1) -> Result<(), UiError> {
    write_sse(http, "snapshot", snapshot)?;
    if let Some(exchanges) = &snapshot.peer_exchanges {
        for exchange in exchanges {
            write_sse(http, "peer_exchange", exchange)?;
        }
    }
    if let Some(messages) = &snapshot.thread_messages {
        for message in messages {
            write_sse(http, "thread_message", message)?;
        }
    }
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
    use std::os::unix::net::UnixListener;
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

    fn ledger_message(id: &str, ts: i64, sender: &str, target: &str) -> LedgerMessage {
        LedgerMessage {
            id: id.to_string(),
            ts,
            sender: sender.to_string(),
            target: target.to_string(),
            body: format!("{sender} vers {target}"),
            delivery_status: None,
        }
    }

    fn capture_ui_registration_channel(
        attested_channel: Option<&str>,
    ) -> (Option<String>, ChannelReport) {
        let socket_path = std::env::temp_dir().join(format!(
            "bridget-ui-register-{}.sock",
            uuid::Uuid::new_v4().simple()
        ));
        let listener = UnixListener::bind(&socket_path).unwrap();
        let (registration_sender, registration_receiver) = std::sync::mpsc::channel();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let registration: WrapperToDaemon = decode(line.trim()).unwrap();
            let registration = match registration {
                WrapperToDaemon::Register {
                    transport, channel, ..
                } => (transport, channel),
                message => panic!("Register UI attendu, reçu {message:?}"),
            };
            registration_sender.send(registration).unwrap();
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::Registered {
                    name: UI_SENDER.to_string(),
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode::<WrapperToDaemon>(line.trim()).unwrap(),
                WrapperToDaemon::JournalReady
            ));
        });

        let presence = open_human_presence(&socket_path, attested_channel).unwrap();
        server.join().unwrap();
        let registration = registration_receiver.recv().unwrap();
        drop(presence);
        std::fs::remove_file(socket_path).unwrap();
        registration
    }

    #[test]
    fn spec_024_presence_ui_locale_annonce_unix_dans_la_trame_reelle() {
        assert_eq!(
            capture_ui_registration_channel(Some("unix")),
            (None, Some("unix".to_string()).into())
        );
    }

    #[test]
    fn spec_024_presence_ui_federee_conserve_ssh_unix_dans_la_trame_reelle() {
        assert_eq!(
            capture_ui_registration_channel(Some("ssh-unix")),
            (None, Some("ssh-unix".to_string()).into())
        );
    }

    #[test]
    fn spec_024_presence_ui_sans_attestation_reste_inconnue_dans_la_trame_reelle() {
        assert_eq!(
            capture_ui_registration_channel(None),
            (None, ChannelReport::Unknown)
        );
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

    #[test]
    fn peer_exchange_distingue_in_out_et_both() {
        let both = aggregate_peer_exchanges(
            "rc1",
            &[
                ledger_message("bulle-avant", 10, "humain", "rc1"),
                ledger_message("sortant", 20, "rc1", "jc6"),
                ledger_message("entrant", 21, "jc6", "rc1"),
                ledger_message("bulle-apres", 30, "rc1", "humain"),
            ],
        );
        assert_eq!(both.len(), 1);
        assert_eq!(both[0].peer, "jc6");
        assert_eq!(both[0].direction, UiPeerDirectionV1::Both);
        assert_eq!(both[0].count, 2);
        assert_eq!(both[0].delivery_ids, ["sortant", "entrant"]);

        let incoming = aggregate_peer_exchanges("rc1", &[ledger_message("in", 40, "jc2", "rc1")]);
        assert_eq!(incoming[0].direction, UiPeerDirectionV1::In);
        let outgoing = aggregate_peer_exchanges("rc1", &[ledger_message("out", 41, "rc1", "jc2")]);
        assert_eq!(outgoing[0].direction, UiPeerDirectionV1::Out);
    }

    #[test]
    fn peer_exchange_garde_count_un_et_la_position_chronologique_reelle() {
        let exchanges = aggregate_peer_exchanges(
            "rc1",
            &[
                ledger_message("second", 200, "rc1", "jc2"),
                ledger_message("premier", 100, "jc6", "rc1"),
            ],
        );
        assert_eq!(exchanges.len(), 2);
        assert_eq!(exchanges[0].at, 100);
        assert_eq!(exchanges[1].at, 200);
        let encoded = serde_json::to_value(&exchanges[0]).unwrap();
        assert_eq!(encoded["count"], 1);
        assert_eq!(encoded["delivery_ids"], serde_json::json!(["premier"]));
    }

    #[test]
    fn deux_demarrages_successifs_rendent_le_meme_port_et_le_meme_jeton() {
        // Attendu écrit en dur — pas recalculé par la fonction sous test.
        // Garde la relecture seule ; le gardien d'écriture est
        // premier_demarrage_persiste_puis_second_relit_sans_retirer.
        const PORT: u16 = 17888;
        const TOKEN: &str = "0123456789abcdef0123456789abcdef";
        let path = std::env::temp_dir().join(format!(
            "bridget-ui-endpoint-stable-{}.json",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::write(
            &path,
            format!(r#"{{"version":1,"port":{PORT},"token":"{TOKEN}"}}"#),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        }

        let first = load_ui_endpoint(&path).unwrap();
        let second = load_ui_endpoint(&path).unwrap();
        let _ = std::fs::remove_file(&path);

        assert_eq!(first.port, PORT);
        assert_eq!(first.token, TOKEN);
        assert_eq!(second.port, PORT);
        assert_eq!(second.token, TOKEN);
        assert_eq!(first.port, second.port);
        assert_eq!(first.token, second.token);
    }

    #[test]
    fn port_deja_pris_echoue_clairement_sans_repli_aleatoire() {
        let holder = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).unwrap();
        let occupied = holder.local_addr().unwrap().port();
        let config = UiRelayConfig {
            daemon_socket: PathBuf::from("/tmp/ui-port-pris.sock"),
            maicie_config: PathBuf::from("/tmp/ui-port-pris.json"),
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, occupied)),
            token: "jeton-port-pris".to_string(),
        };
        let error = match UiRelay::bind(config) {
            Ok(_) => panic!("bind devait échouer sur port déjà pris"),
            Err(err) => err,
        };
        let message = error.to_string();
        assert!(
            message.contains("déjà pris") && message.contains(&occupied.to_string()),
            "message attendu de refus explicite, reçu: {message}"
        );
        assert!(
            message.contains("aucun repli"),
            "{message}"
        );
    }

    /// Gardien d'écriture sur le chemin réel : load_or_create → fichier → relecture.
    /// Un mutant qui n'écrit rien ou réécrit un jeton neuf à chaque fois tue ce témoin.
    #[test]
    fn premier_demarrage_persiste_puis_second_relit_sans_retirer() {
        let path = std::env::temp_dir().join(format!(
            "bridget-ui-endpoint-create-{}.json",
            uuid::Uuid::new_v4().simple()
        ));
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("json.tmp"));
        let created = load_or_create_ui_endpoint(&path, DEFAULT_UI_PORT).unwrap();
        assert_eq!(created.port, DEFAULT_UI_PORT);
        assert_eq!(created.token.len(), 32);
        let reloaded = load_or_create_ui_endpoint(&path, DEFAULT_UI_PORT).unwrap();
        assert_eq!(reloaded.port, created.port);
        assert_eq!(reloaded.token, created.token);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600, "permissions jeton: {mode:#o}");
        }
        let _ = std::fs::remove_file(&path);
    }

    #[cfg(unix)]
    fn seed_stale_open_tmp(path: &Path) -> PathBuf {
        use std::io::Write;
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

        let tmp = path.with_extension("json.tmp");
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(&tmp);
        {
            let mut stale = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o666)
                .open(&tmp)
                .unwrap();
            stale.write_all(b"stale").unwrap();
        }
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o666)).unwrap();
        let stale_mode = std::fs::metadata(&tmp).unwrap().permissions().mode() & 0o777;
        assert_eq!(stale_mode, 0o666, "précondition: temporaire ouvert {stale_mode:#o}");
        tmp
    }

    /// Garde 1 — démarrage malgré un résidu. Meurt si exclusif sans purge.
    #[cfg(unix)]
    #[test]
    fn temporaire_preexistant_ne_bloque_pas_le_demarrage() {
        let path = std::env::temp_dir().join(format!(
            "bridget-ui-endpoint-stale-boot-{}.json",
            uuid::Uuid::new_v4().simple()
        ));
        let tmp = seed_stale_open_tmp(&path);

        let created = load_or_create_ui_endpoint(&path, DEFAULT_UI_PORT)
            .expect("un résidu temporaire ne doit pas bloquer le démarrage");
        assert_eq!(created.port, DEFAULT_UI_PORT);
        assert_eq!(created.token.len(), 32);
        assert!(path.exists(), "le final doit être livré");

        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(&path);
    }

    /// Garde 2 — final protégé. Meurt si create+truncate hérite des droits ouverts.
    /// Si le démarrage échoue : INAPPLICABLE — on échoue explicitement (pas un vert
    /// par abstention). La garde 1 juge la panne ; ici on refuse de se dire satisfait.
    #[cfg(unix)]
    #[test]
    fn temporaire_preexistant_livre_un_final_protege() {
        use std::os::unix::fs::PermissionsExt;

        let path = std::env::temp_dir().join(format!(
            "bridget-ui-endpoint-stale-mode-{}.json",
            uuid::Uuid::new_v4().simple()
        ));
        let tmp = seed_stale_open_tmp(&path);

        let created = match load_or_create_ui_endpoint(&path, DEFAULT_UI_PORT) {
            Ok(endpoint) => endpoint,
            Err(error) => {
                let _ = std::fs::remove_file(&tmp);
                let _ = std::fs::remove_file(&path);
                panic!(
                    "INAPPLICABLE — pas de final à juger ({error}) ; \
                     ce n'est pas un succès du témoin mode (abstention = oracle vacant)"
                );
            }
        };
        assert!(path.exists());
        let final_mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(
            final_mode, 0o600,
            "final ne doit pas hériter des droits ouverts du temporaire: {final_mode:#o}"
        );
        assert!(!tmp.exists(), "le temporaire ne doit plus rester après rename");
        let reloaded = load_or_create_ui_endpoint(&path, DEFAULT_UI_PORT).unwrap();
        assert_eq!(reloaded.token, created.token);

        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(&path);
    }

    /// Propriété entrante : présence + contenu, et absence si seuls des pairs.
    #[test]
    fn fil_humain_referent_entrant_porte_corps() {
        assert!(
            human_referent_thread_messages(
                "bridget",
                &[
                    ledger_message("agent-pair", 5, "rc1", "bridget"),
                    ledger_message("other", 30, "bridget", "rc1"),
                ],
            )
            .is_empty(),
            "sans entrant humain, aucune bulle"
        );
        let messages = human_referent_thread_messages(
            "bridget",
            &[
                ledger_message("agent-pair", 5, "rc1", "bridget"),
                ledger_message("h1", 10, "humain", "bridget"),
                ledger_message("other", 30, "bridget", "rc1"),
            ],
        );
        assert_eq!(messages.len(), 1, "un seul entrant attendu");
        assert_eq!(messages[0].delivery_id, "h1");
        assert_eq!(messages[0].role, UiThreadRoleV1::User);
        assert_eq!(messages[0].at, 10);
        assert_eq!(messages[0].text, "humain vers bridget");
    }

    /// Propriété sortante : présence + contenu, et absence si seuls des pairs.
    /// Distincte de l'entrante — si le sortant retombe, ce témoin meurt sans l'autre.
    #[test]
    fn fil_humain_referent_sortant_porte_corps() {
        assert!(
            human_referent_thread_messages(
                "bridget",
                &[
                    ledger_message("agent-pair", 5, "rc1", "bridget"),
                    ledger_message("other", 30, "bridget", "rc1"),
                ],
            )
            .is_empty(),
            "sans sortant vers humain, aucune bulle"
        );
        let messages = human_referent_thread_messages(
            "bridget",
            &[
                ledger_message("agent-pair", 5, "rc1", "bridget"),
                ledger_message("b1", 20, "bridget", "humain"),
                ledger_message("other", 30, "bridget", "rc1"),
            ],
        );
        assert_eq!(messages.len(), 1, "un seul sortant attendu");
        assert_eq!(messages[0].delivery_id, "b1");
        assert_eq!(messages[0].role, UiThreadRoleV1::Agent);
        assert_eq!(messages[0].at, 20);
        assert_eq!(messages[0].text, "bridget vers humain");
    }

    /// Les deux sens ensemble : ordre chronologique (entrant puis sortant).
    #[test]
    fn fil_humain_referent_ordonne_entrant_puis_sortant() {
        let messages = human_referent_thread_messages(
            "bridget",
            &[
                ledger_message("b1", 20, "bridget", "humain"),
                ledger_message("h1", 10, "humain", "bridget"),
            ],
        );
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].delivery_id, "h1");
        assert_eq!(messages[0].role, UiThreadRoleV1::User);
        assert_eq!(messages[1].delivery_id, "b1");
        assert_eq!(messages[1].role, UiThreadRoleV1::Agent);
    }

    #[test]
    fn chemin_productif_snapshot_emprunte_human_referent_thread_messages() {
        // Garde anti-feuille : snapshot + SSE initial + chemin vivant du watch.
        let source = include_str!("ui.rs");
        let read_body = function_body(source, "fn read_snapshot(");
        assert!(
            read_body.contains("human_referent_thread_messages("),
            "read_snapshot doit appeler human_referent_thread_messages"
        );
        assert!(
            read_body.contains("thread_messages"),
            "le snapshot doit exposer thread_messages au client"
        );
        let sse_body = function_body(source, "fn write_snapshot_sse(");
        assert!(
            sse_body.contains("\"thread_message\""),
            "write_snapshot_sse doit émettre l'événement SSE thread_message"
        );
        let watch_body = function_body(source, "fn stream_sse_journal(");
        assert!(
            watch_body.contains("push_live_thread_messages("),
            "le watch doit appeler push_live_thread_messages (chemin vivant)"
        );
    }

    /// Garde anti-feuille : ouverture sans from_seq = Seq des 10 derniers
    /// enregistrements complets, calculé dans le relais — jamais Seq(0)/Today/Tail filaire.
    #[test]
    fn chemin_productif_ouverture_journal_emprunte_fenetre_relais() {
        let source = include_str!("ui.rs");
        let resolve_body = function_body(source, "fn resolve_ui_journal_window(");
        assert!(
            resolve_body.contains("resolve_ui_journal_window_in("),
            "resolve_ui_journal_window doit déléguer au calcul relais"
        );
        let inner = function_body(source, "fn resolve_ui_journal_window_in(");
        assert!(
            inner.contains("AttachWindow::Seq(from_seq)"),
            "le relais doit envoyer Seq, compris du daemon live"
        );
        assert!(
            !inner.contains("AttachWindow::Tail("),
            "Tail ne doit pas partir sur le fil Attach"
        );
        assert!(
            !inner.contains("AttachWindow::Today"),
            "Today n'est plus le défaut d'ouverture page"
        );
        assert!(
            !inner.contains("AttachWindow::Seq(0)"),
            "Seq(0) ne doit pas être le défaut d'ouverture"
        );
        let serve_body = function_body(source, "fn serve_connection(");
        assert!(
            serve_body.contains("resolve_ui_journal_window("),
            "serve_connection doit appeler resolve_ui_journal_window"
        );
        assert!(
            serve_body.matches("resolve_ui_journal_window(").count() >= 2,
            "/v1/journal et /v1/watch doivent emprunter le calcul relais"
        );
        assert_eq!(UI_JOURNAL_OPEN_COMPLETE_RECORDS, 10);
        assert_eq!(UI_JOURNAL_MAX_REPLAY_FRAGMENTS, 150);
        let stream_body = function_body(source, "fn stream_sse_journal(");
        assert!(
            stream_body.contains("UI_JOURNAL_MAX_REPLAY_FRAGMENTS"),
            "le rejeu SSE doit borner les fragments"
        );
        assert!(
            stream_body.contains("\"journal_page\""),
            "has_more doit être émis sur le chemin SSE réel"
        );
    }

    /// Remontée : 20 enregistrements complets sous la fenêtre d'ouverture.
    #[test]
    fn remontee_from_seq_donne_l_ancien_par_pages_de_vingt() {
        let seqs: Vec<u64> = (1..=100).collect();
        let (open_from, has_more) =
            open_window_from_complete_records(&seqs, UI_JOURNAL_OPEN_COMPLETE_RECORDS);
        assert_eq!(open_from, 91);
        assert!(has_more, "il doit rester de l'ancien sous les 10 derniers");
        let older = older_page_from_complete_records(
            &seqs,
            open_from,
            UI_JOURNAL_OLDER_PAGE_RECORDS,
        );
        assert_eq!(older, 71);
        assert!(older < open_from);
        assert_eq!(UI_JOURNAL_OLDER_PAGE_RECORDS, 20);
        assert_eq!(older_journal_page_from_seq(91), 71);
        let mut query = HashMap::new();
        query.insert("from_seq".to_string(), older.to_string());
        let dir = PathBuf::from("/tmp/bridget-ui-journal-absent");
        let (window, page) = resolve_ui_journal_window_in(dir, &query).unwrap();
        assert_eq!(window, AttachWindow::Seq(71));
        assert!(page.has_more);
    }

    #[test]
    fn ouverture_sans_from_seq_prend_dix_enregistrements_complets() {
        let root = PathBuf::from(format!(
            "/tmp/bridget-ui-journal-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        let mut lines = String::new();
        for seq in 1..=40 {
            lines.push_str(&format!("{{\"v\":1,\"seq\":{seq}}}\n"));
        }
        std::fs::write(root.join("2026-08-26.jsonl"), lines).unwrap();
        let (window, page) =
            resolve_ui_journal_window_in(root.clone(), &HashMap::new()).unwrap();
        assert_eq!(window, AttachWindow::Seq(31));
        assert!(page.has_more);
        assert_eq!(page.from_seq, 31);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Garde anti-feuille : retirer l'appel à `retain_living_objectives` dans
    /// `read_snapshot` (tout en laissant la fonction intacte) doit tuer ce témoin.
    #[test]
    fn chemin_productif_snapshot_emprunte_retain_living_objectives() {
        let source = include_str!("ui.rs");
        let read_body = function_body(source, "fn read_snapshot(");
        assert!(
            read_body.contains("retain_living_objectives("),
            "read_snapshot doit appeler retain_living_objectives — sinon le mégaoctet clos revient et la page se fige"
        );
        assert!(
            read_body.contains("missions"),
            "le snapshot doit exposer missions au client"
        );
    }

    fn function_body<'a>(source: &'a str, signature: &str) -> &'a str {
        let start = source
            .find(signature)
            .unwrap_or_else(|| panic!("signature absente: {signature}"));
        let after = start + signature.len();
        let end = source[after..]
            .find("\nfn ")
            .map(|offset| after + offset)
            .unwrap_or(source.len());
        &source[start..end]
    }

    fn seed_search_ledger(db_path: &Path) {
        let store = crate::store::Store::open(db_path).unwrap();
        drop(store);
        let conn = rusqlite::Connection::open(db_path).unwrap();
        // Décoys `100Xwild` : sans échappement LIKE, `100%_wild` les matche
        // toutes ( % = joker, _ = un caractère ). Avec échappement : 1 seule.
        conn.execute_batch(
            r#"
            DELETE FROM ledger;
            INSERT INTO ledger (id, ts, sender, target, body, conversation_key) VALUES
              ('hit-early', 100, 'bridget', 'cursor4', 'alpha cible premiere', 'bridget:cursor4'),
              ('noise', 150, 'jc2', 'jc6', 'rien a voir', 'jc2:jc6'),
              ('decoy-a', 160, 'jc2', 'jc6', 'prefix 100awild suffix', 'jc2:jc6'),
              ('decoy-b', 161, 'jc2', 'jc6', 'prefix 100bwild suffix', 'jc2:jc6'),
              ('decoy-c', 162, 'jc2', 'jc6', 'prefix 100cwild suffix', 'jc2:jc6'),
              ('decoy-d', 163, 'jc2', 'jc6', 'prefix 100dwild suffix', 'jc2:jc6'),
              ('decoy-e', 164, 'jc2', 'jc6', 'prefix 100ewild suffix', 'jc2:jc6'),
              ('hit-late', 200, 'cursor4', 'bridget', 'seconde cible avec <tag> et 100%_wild', 'cursor4:bridget'),
              ('accent', 220, 'bridget', 'cursor4', 'Le café est prêt', 'bridget:cursor4');
            "#,
        )
        .unwrap();
    }

    fn seed_truncated_ledger(db_path: &Path, count: usize) {
        let store = crate::store::Store::open(db_path).unwrap();
        drop(store);
        let conn = rusqlite::Connection::open(db_path).unwrap();
        conn.execute_batch("DELETE FROM ledger;").unwrap();
        for index in 0..count {
            conn.execute(
                "INSERT INTO ledger (id, ts, sender, target, body, conversation_key)
                 VALUES (?1, ?2, 'bridget', 'cursor4', ?3, 'bridget:cursor4')",
                rusqlite::params![
                    format!("row-{index}"),
                    1_000 + index as i64,
                    format!("motif-troncature numero {index}"),
                ],
            )
            .unwrap();
        }
    }

    fn post_search(address: SocketAddr, token: &str, body: &str) -> (u16, String) {
        let mut client = TcpStream::connect(address).unwrap();
        let payload = body.as_bytes();
        let request = format!(
            "POST /v1/search?token={token} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            payload.len()
        );
        client.write_all(request.as_bytes()).unwrap();
        client.write_all(payload).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut response = Vec::new();
        client.read_to_end(&mut response).unwrap();
        let text = String::from_utf8(response).unwrap();
        let status = text
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        (status, text)
    }

    fn json_body(raw: &str) -> serde_json::Value {
        let json = raw.split("\r\n\r\n").nth(1).unwrap_or("");
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn recherche_depuis_la_page_rend_corps_auteur_horodatage_en_ordre() {
        let root = std::env::temp_dir().join(format!(
            "bridget-ui-search-{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let socket = root.join("bridget.sock");
        let db = root.join("bridget.db");
        seed_search_ledger(&db);

        let config = UiRelayConfig {
            daemon_socket: socket,
            maicie_config: root.join("maicie.json"),
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            token: "jeton-search".to_string(),
        };
        let relay = UiRelay::bind(config).unwrap();
        let address = relay.local_addr().unwrap();

        let worker = thread::spawn(move || relay.serve_one().unwrap());
        let (hit_status, hit_raw) =
            post_search(address, "jeton-search", r#"{"version":1,"q":"cible"}"#);
        worker.join().unwrap();
        assert_eq!(hit_status, 200, "{hit_raw}");
        let hit_value = json_body(&hit_raw);
        assert_eq!(hit_value["hits"].as_array().map(|a| a.len()), Some(2));
        assert_eq!(hit_value["hits"][0]["id"], "hit-early");
        assert_eq!(hit_value["hits"][0]["sender"], "bridget");
        assert_eq!(hit_value["hits"][0]["ts"], 100);
        assert_eq!(hit_value["hits"][0]["body"], "alpha cible premiere");
        assert_eq!(hit_value["hits"][1]["id"], "hit-late");
        assert_eq!(hit_value["hits"][1]["sender"], "cursor4");
        assert_eq!(hit_value["hits"][1]["ts"], 200);
        assert_eq!(
            hit_value["hits"][1]["body"],
            "seconde cible avec <tag> et 100%_wild"
        );
        assert_eq!(hit_value["truncated"], false);
        assert!(
            hit_value["hits"][0]["ts"].as_i64().unwrap()
                < hit_value["hits"][1]["ts"].as_i64().unwrap()
        );

        let config = UiRelayConfig {
            daemon_socket: root.join("bridget.sock"),
            maicie_config: root.join("maicie.json"),
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            token: "jeton-search".to_string(),
        };
        let relay = UiRelay::bind(config).unwrap();
        let address = relay.local_addr().unwrap();
        let worker = thread::spawn(move || relay.serve_one().unwrap());
        let (miss_status, miss_raw) =
            post_search(address, "jeton-search", r#"{"version":1,"q":"motabsentxyz"}"#);
        worker.join().unwrap();
        assert_eq!(miss_status, 200, "{miss_raw}");
        let miss_value = json_body(&miss_raw);
        assert_eq!(miss_value["hits"].as_array().map(|a| a.len()), Some(0));
        assert_eq!(miss_value["truncated"], false);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn echappement_like_exclut_les_leurres_joker() {
        let root = std::env::temp_dir().join(format!(
            "bridget-ui-search-escape-{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let db = root.join("bridget.db");
        seed_search_ledger(&db);

        // Attendu en dur : un seul hit littéral. Sans escape_like_needle,
        // les 5 leurres 100Xwild passent aussi → len != 1.
        let outcome = crate::store::Store::open(&db)
            .unwrap()
            .search_messages("100%_wild", 100)
            .unwrap();
        assert_eq!(
            outcome.hits.len(),
            1,
            "sans échappement les leurres 100Xwild passeraient, ids={:?}",
            outcome
                .hits
                .iter()
                .map(|hit| hit.id.as_str())
                .collect::<Vec<_>>()
        );
        assert_eq!(outcome.hits[0].id, "hit-late");
        assert!(outcome.hits[0].body.contains("100%_wild"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn troncature_est_signalee_quand_le_plafond_est_atteint() {
        let root = std::env::temp_dir().join(format!(
            "bridget-ui-search-trunc-{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let db = root.join("bridget.db");
        seed_truncated_ledger(&db, 120);

        let config = UiRelayConfig {
            daemon_socket: root.join("bridget.sock"),
            maicie_config: root.join("maicie.json"),
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            token: "jeton-trunc".to_string(),
        };
        let relay = UiRelay::bind(config).unwrap();
        let address = relay.local_addr().unwrap();
        let worker = thread::spawn(move || relay.serve_one().unwrap());
        let (status, raw) = post_search(
            address,
            "jeton-trunc",
            r#"{"version":1,"q":"motif-troncature"}"#,
        );
        worker.join().unwrap();
        assert_eq!(status, 200, "{raw}");
        let value = json_body(&raw);
        assert_eq!(value["hits"].as_array().map(|a| a.len()), Some(100));
        assert_eq!(
            value["truncated"], true,
            "120 correspondances doivent signaler truncated=true, reçu {value}"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn cafe_sans_accent_trouve_cafe_avec_accent() {
        let root = std::env::temp_dir().join(format!(
            "bridget-ui-search-accent-{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let db = root.join("bridget.db");
        seed_search_ledger(&db);
        let outcome = crate::store::Store::open(&db)
            .unwrap()
            .search_messages("cafe", 100)
            .unwrap();
        assert_eq!(outcome.hits.len(), 1);
        assert_eq!(outcome.hits[0].id, "accent");
        assert!(outcome.hits[0].body.contains("café"));
        let _ = std::fs::remove_dir_all(&root);
    }
}
