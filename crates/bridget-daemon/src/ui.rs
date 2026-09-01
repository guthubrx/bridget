//! Relais local du poste de travail Bridget.
//!
//! Le navigateur ne parle jamais à la socket Unix du daemon. Ce module ouvre
//! des connexions ordinaires vers les projections publiques (`ListAgents`,
//! `LedgerProjection`, Attach) puis les traduit en HTTP/SSE loopback.

use crate::agent_profile::{
    AgentProfileDetail, AgentProfileError, AgentProfileStore, AgentProfileSummary,
    AgentProfileUpdate, AttentionEvent, AttentionEventType, ClientNotificationPreference,
};
use crate::mission_projection::{
    MissionProjectionV1, read_public_mission_projection_v1, retain_living_objectives,
};
use crate::project_policy::ProjectRootPolicy;
use bridget_core::{BridgetMessage, MessageIntent, MessageOrigin};
use bridget_transport::journal::valid_events;
use bridget_transport::protocol::{
    AttachWindow, CLIENT_CONTRACT_VERSION, ClientCapability, ConnectionRole, IdempotencyIssue,
    LedgerMessage, LedgerScope, PROJECT_REGISTRY_CONTRACT_VERSION, PROJECT_ROUND_INTERVAL_SECS,
    PROJECT_ROUND_POLICY_CONTRACT_VERSION, PresenceMode, ProjectAdminOperation,
    ProjectAdminRequest, ProjectBackend, ProjectBindRequest, ProjectBindStatus,
    ProjectBindingProjection, ProjectBindingStatus, ProjectRoundDispatchState,
    ProjectRoundOperation, ProjectRoundOutcome, ProjectRoundProjection, ProjectRoundRefusal,
    ProjectRoundRequest, ProjectRuntimeOperation, ProjectRuntimeRefusal, ProjectRuntimeRequest,
    SERVICE_CONTRACT_VERSION, ServiceCapability, decode, encode,
};
use bridget_transport::{ChannelReport, DaemonToWrapper, WrapperToDaemon};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
const MAX_HTTP_BODY_BYTES: usize = 192 * 1024;
const MAX_UI_AGENT_NAME_BYTES: usize = 100;
const MAX_UI_COMMAND_ID_BYTES: usize = 160;
const MAX_UI_SSE_EVENTS: usize = 20_000;
const MAX_UI_RECONNECT_ATTEMPTS: usize = 50;
const MAX_UI_FILE_PREVIEW_BYTES: u64 = 256 * 1024;
const MAX_UI_ARTIFACT_RESPONSE_BYTES: usize = 512 * 1024;
const MAX_UI_ARTIFACT_EXPORT_BYTES: usize = 2 * 1024 * 1024;
const MAX_UI_ARTIFACT_PAGE_ROWS: usize = 100;
const UI_SANDBOX_TICKET_TTL: Duration = Duration::from_secs(60);
const UI_SANDBOX_CSP: &str = "default-src 'none'; connect-src 'none'; frame-src 'none'; child-src 'none'; form-action 'none'; base-uri 'none'; object-src 'none'; media-src 'none'; img-src data: blob:; style-src 'unsafe-inline'; script-src 'unsafe-inline'";
const UI_RECONNECT_DELAY: Duration = Duration::from_millis(100);
/// La rétention des présences côté daemon s'appuie sur `link_seen`, rafraîchi
/// par le heartbeat. Une présence qui ne bat pas est donc jetée au bout de
/// PRESENCE_RETENTION (300 s), même si sa socket est intacte. L'humain
/// disparaissait ainsi de l'annuaire cinq minutes après son dernier message,
/// et toute réponse à un humain silencieux était rejetée.
const HUMAN_PRESENCE_HEARTBEAT: Duration = Duration::from_secs(3);
/// Le battement maintient une présence vivante ; il ne la ressuscite pas.
/// Quand le daemon redémarre, la socket meurt avec lui et l'humain sort de
/// l'annuaire sans jamais y revenir, puisqu'il ne réémet rien de lui-même.
/// Cette veille rouvre la présence dès qu'elle est tombée.
const HUMAN_PRESENCE_WATCH: Duration = Duration::from_secs(10);
/// Relecture ledger pendant un watch : le sortant référent→humain n'apparaît
/// jamais au journal ; sans ce rythme le fil reste figé après l'ouverture.
const UI_THREAD_LEDGER_POLL: Duration = Duration::from_millis(400);
const UI_SENDER: &str = "humain";
/// Ouverture : derniers **tours projetés** (même `message_id` que `projectTimeline`
/// dans la page). Pas des séquences brutes — sinon on coupe un échange à l'écran
/// (réponse sans question). Chiffres T3 : 10 / 20.
const UI_JOURNAL_OPEN_TURNS: usize = 10;
const UI_JOURNAL_OLDER_PAGE_TURNS: usize = 20;
/// Plafond dur de fragments SSE avant CaughtUp. Mesure 2026-08-26 : p50 ≈ 200
/// records/tour (cursor4/6), un tour max > 700 ; 150 coupait toujours au milieu.
/// 500 ≈ 2 tours médians, cible ~100 Ko utiles sans renvoyer le mégaoctet.
const UI_JOURNAL_MAX_REPLAY_FRAGMENTS: usize = 500;
/// Port loopback par défaut : stable d'un lancement à l'autre, sans option CLI.
pub const DEFAULT_UI_PORT: u16 = 17888;
const UI_ENDPOINT_STATE_VERSION: u8 = 1;
const UI_INDEX: &[u8] = include_bytes!("../assets/ui/index.html");
const UI_SCRIPT: &[u8] = include_bytes!("../assets/ui/app.js");
const UI_ARTIFACT_RENDERER: &[u8] = include_bytes!("../assets/ui/artifact-renderer.js");
const UI_ARTIFACT_SANDBOX_HOST: &[u8] = include_bytes!("../assets/ui/artifact-sandbox-host.js");
const UI_THEME: &[u8] = include_bytes!("../assets/ui/theme.css");
const UI_MARKED: &[u8] = include_bytes!("../assets/ui/vendor/marked.min.js");
const UI_PURIFY: &[u8] = include_bytes!("../assets/ui/vendor/purify.min.js");
const UI_HIGHLIGHT: &[u8] = include_bytes!("../assets/ui/vendor/highlight.min.js");
const UI_ECHARTS: &[u8] = include_bytes!("../assets/ui/vendor/echarts.min.js");
const UI_TABULATOR: &[u8] = include_bytes!("../assets/ui/vendor/tabulator.min.js");
const UI_TABULATOR_CSS: &[u8] = include_bytes!("../assets/ui/vendor/tabulator.min.css");
const UI_HIGHLIGHT_GITHUB_DARK: &[u8] =
    include_bytes!("../assets/ui/vendor/highlight-github-dark.min.css");
const UI_HIGHLIGHT_GITHUB_LIGHT: &[u8] =
    include_bytes!("../assets/ui/vendor/highlight-github-light.min.css");
const UI_PROVIDER_OPENAI: &[u8] = include_bytes!("../assets/ui/providers/openai.svg");
const UI_PROVIDER_CLAUDE: &[u8] = include_bytes!("../assets/ui/providers/claude.svg");
const UI_PROVIDER_CURSOR: &[u8] = include_bytes!("../assets/ui/providers/cursor.svg");
const UI_PROVIDER_GEMINI: &[u8] = include_bytes!("../assets/ui/providers/gemini.svg");
const UI_PROVIDER_GLM: &[u8] = include_bytes!("../assets/ui/providers/glm.svg");
const UI_PROVIDER_DEEPSEEK: &[u8] = include_bytes!("../assets/ui/providers/deepseek.svg");

#[derive(Debug, Clone)]
pub struct UiRelayConfig {
    pub daemon_socket: PathBuf,
    pub maicie_config: PathBuf,
    /// Même document que celui remis au daemon. Son absence laisse la flotte
    /// lisible mais ferme toutes les routes d'administration des projets.
    pub project_root_policy_path: Option<PathBuf>,
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
                    let absolute = std::fs::canonicalize(&tmp).unwrap_or_else(|_| tmp.clone());
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
pub fn load_or_create_ui_endpoint(path: &Path, default_port: u16) -> Result<UiEndpoint, UiError> {
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
        UiError::Configuration(format!("état UI illisible ({}) : {error}", path.display()))
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
            project_root_policy_path: None,
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
    sandbox_tickets: Mutex<HashMap<String, UiArtifactSandboxTicketV1>>,
}

struct UiHumanPresence {
    alive: Arc<AtomicBool>,
}

#[derive(Clone)]
struct UiArtifactSandboxTicketV1 {
    project_id: String,
    agent: String,
    version_ref: String,
    expires_at: Instant,
}

impl UiRelayRuntime {
    fn new(human_presence_channel: Option<String>) -> Self {
        Self {
            human_presence: Mutex::new(None),
            human_presence_channel,
            sandbox_tickets: Mutex::new(HashMap::new()),
        }
    }

    fn issue_sandbox_ticket(
        &self,
        project_id: String,
        agent: String,
        version_ref: String,
    ) -> String {
        let ticket = uuid::Uuid::new_v4().simple().to_string();
        let mut tickets = self
            .sandbox_tickets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        tickets.retain(|_, value| value.expires_at > Instant::now());
        tickets.insert(
            ticket.clone(),
            UiArtifactSandboxTicketV1 {
                project_id,
                agent,
                version_ref,
                expires_at: Instant::now() + UI_SANDBOX_TICKET_TTL,
            },
        );
        ticket
    }

    fn resolve_sandbox_ticket(&self, ticket: &str) -> Option<UiArtifactSandboxTicketV1> {
        let mut tickets = self
            .sandbox_tickets
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        tickets.retain(|_, value| value.expires_at > Instant::now());
        tickets.get(ticket).cloned()
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

    fn invalidate_human_presence(&self) {
        let mut presence = self
            .human_presence
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(current) = presence.take() {
            current.alive.store(false, Ordering::Release);
        }
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
                identity_version: 2,
                agent_id: "550e8400-e29b-41d4-a716-4466554400f0".to_string(),
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
        DaemonToWrapper::Registered { agent_id: _ } => {}
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

    // Battement de la présence humaine. Le thread de lecture ci-dessous est
    // bloqué sur read_daemon et ne peut donc pas émettre lui-même ; l'écrivain
    // est partagé, un second thread suffit. Sans ce battement la présence est
    // valide cinq minutes puis jetée, alors que la socket reste ouverte.
    let heartbeat_alive = Arc::clone(&alive);
    let heartbeat_writer = Arc::clone(&writer);
    thread::spawn(move || {
        while heartbeat_alive.load(Ordering::Acquire) {
            thread::sleep(HUMAN_PRESENCE_HEARTBEAT);
            if !heartbeat_alive.load(Ordering::Acquire) {
                break;
            }
            let mut writer_guard = heartbeat_writer
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if send_daemon(&mut writer_guard, &WrapperToDaemon::Heartbeat).is_err() {
                heartbeat_alive.store(false, Ordering::Release);
                break;
            }
        }
    });

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
        // L'humain doit être joignable dès le démarrage du relais, sans avoir
        // rien envoyé et sans avoir coché « attendre une réponse ». Sinon il
        // n'entre à l'annuaire qu'au premier envoi avec réponse attendue, et
        // toute réponse qui lui est destinée est rejetée « agent introuvable ».
        // L'échec n'empêche pas de servir : le sens humain -> agent doit tenir
        // même si l'inscription échoue.
        if let Err(error) = self
            .runtime
            .ensure_human_presence(&self.config.daemon_socket)
        {
            eprintln!("relais UI: inscription de l'humain à l'annuaire impossible: {error}");
        }

        // Veille de présence. Mesuré le 28/08 : au redémarrage du daemon les
        // agents se réinscrivent seuls, mais pas l'humain — il n'a pas de
        // wrapper qui le reconnecte. Sans cette veille, un seul redémarrage le
        // coupe définitivement et il faut relancer le relais à la main.
        let veille_runtime = Arc::clone(&self.runtime);
        let veille_socket = self.config.daemon_socket.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(HUMAN_PRESENCE_WATCH);
                if let Err(error) = veille_runtime.ensure_human_presence(&veille_socket) {
                    eprintln!("relais UI: réinscription de l'humain impossible: {error}");
                }
            }
        });

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
    run_with_project_root_policy(daemon_socket, maicie_config, None)
}

/// Variante de lancement qui associe explicitement l'UI au même document de
/// racines que le daemon. Sans ce chemin, les routes de projet échoueront
/// fermées et le reste de l'interface demeure disponible.
pub fn run_with_project_root_policy(
    daemon_socket: PathBuf,
    maicie_config: PathBuf,
    project_root_policy_path: Option<PathBuf>,
) -> Result<(), UiError> {
    let mut config = UiRelayConfig::loopback(daemon_socket, maicie_config)?;
    config.project_root_policy_path = project_root_policy_path;
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
    alert_thresholds: UiAlertThresholdsV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    peer_exchanges: Option<Vec<UiPeerExchangeV1>>,
    /// Bulles utilisateur↔agent focal — corps issus du ledger (pas du journal).
    /// Absentes hors focus ; présentes (éventuellement vides) dès qu'un agent est ciblé.
    #[serde(skip_serializing_if = "Option::is_none")]
    thread_messages: Option<Vec<UiThreadMessageV1>>,
    /// Références de versions exactes publiées dans le fil de l'agent focal.
    /// Les données complètes restent derrière une route relayée, afin de ne
    /// pas gonfler le snapshot ni exposer les chemins canoniques.
    #[serde(skip_serializing_if = "Option::is_none")]
    artifact_references: Option<UiArtifactReferenceProjectionV1>,
    open_requests: Vec<bridget_transport::protocol::RequestInfo>,
    missions: MissionProjectionV1,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    recovery_losses: Vec<UiRecoveryLossV1>,
}

#[derive(Debug, Serialize)]
struct UiArtifactReferenceProjectionV1 {
    state: &'static str,
    items: Vec<UiArtifactReferenceV1>,
}

#[derive(Debug, Serialize)]
struct UiArtifactReferenceV1 {
    reference_id: String,
    artifact_ref: String,
    version_ref: String,
    turn_reference: String,
    kind: crate::artifact_types::ArtifactKind,
    title: String,
    state: crate::artifact_types::ArtifactState,
    pinned: bool,
    created_at: i64,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct UiAlertThresholdsV1 {
    pub stale_message_secs: u64,
    pub stalled_turn_secs: u64,
    pub stalled_approval_secs: u64,
    pub saturated_queue_depth: u64,
}

impl Default for UiAlertThresholdsV1 {
    fn default() -> Self {
        Self {
            stale_message_secs: 15 * 60,
            stalled_turn_secs: 5 * 60,
            stalled_approval_secs: 5 * 60,
            saturated_queue_depth: 10,
        }
    }
}

/// Alertes déduites exclusivement de faits de projection bornés. Les noms sont
/// des codes stables, sans identité, corps de message ni cardinalité dynamique.
pub fn execution_alerts(
    turn_state: Option<&str>,
    wait_state: Option<&str>,
    progress_age_secs: Option<u64>,
    queue_depth: u64,
    message_age_secs: Option<u64>,
) -> Vec<String> {
    let thresholds = UiAlertThresholdsV1::default();
    let mut alerts = Vec::new();
    if message_age_secs.is_some_and(|age| age >= thresholds.stale_message_secs) {
        alerts.push("stale_message".to_string());
    }
    if turn_state == Some("running")
        && progress_age_secs.is_some_and(|age| age >= thresholds.stalled_turn_secs)
    {
        alerts.push("stalled_turn".to_string());
    }
    if wait_state == Some("waiting_approval")
        && progress_age_secs.is_some_and(|age| age >= thresholds.stalled_approval_secs)
    {
        alerts.push("stalled_approval".to_string());
    }
    if queue_depth >= thresholds.saturated_queue_depth {
        alerts.push("queue_saturated".to_string());
    }
    alerts
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
    #[serde(skip_serializing_if = "Option::is_none")]
    vigilance_round: Option<UiVigilanceRoundV1>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct UiVigilanceRoundV1 {
    interval: String,
    headline: String,
    signal: String,
    body: String,
}

#[derive(Debug, Serialize)]
struct UiAgentRowV1 {
    agent_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    profile: Option<UiAgentProfileV1>,
    #[serde(rename = "type")]
    agent_type: String,
    host: String,
    transport: String,
    /// Domaine historique de regroupement. Il n'est jamais une identité ni
    /// une frontière de sécurité projet.
    #[serde(skip_serializing_if = "Option::is_none")]
    domain: Option<String>,
    /// Identité opaque de projet issue du lien durable de spawn.
    #[serde(skip_serializing_if = "Option::is_none")]
    project_id: Option<String>,
    /// L'absence de liaison est visible séparément, sans réutiliser le
    /// domaine ni inférer un projet depuis le cwd.
    project_state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<PresenceMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    effort: Option<String>,
    /// Valeur presente si et seulement si le cycle de vie est gere par Bridget.
    persistent: Option<bool>,
    state: &'static str,
    /// État de connexion public, distinct de l'exécution durable.
    connection_state: &'static str,
    /// Âge de la dernière capacité fournisseur attestée, jamais inventée.
    provider_age_secs: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    turn_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    wait_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    progress_age_secs: Option<u64>,
    queue_depth: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    continuation_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent_link: Option<bridget_transport::protocol::AgentLinkUiProjection>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    alerts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<bridget_transport::protocol::ProviderUiProjection>,

    last_message_at: Option<i64>,
    last_excerpt: Option<String>,
    unread: usize,
}

#[derive(Debug, Clone, Serialize)]
struct UiAgentProfileV1 {
    agent_id: String,
    display_name: String,
    labels: Vec<String>,
    avatar: UiAgentAvatarV1,
    instruction_state: UiInstructionStateV1,
}

#[derive(Debug, Clone, Serialize)]
struct UiAgentAvatarV1 {
    shape: String,
    color: String,
}

#[derive(Debug, Clone, Serialize)]
struct UiInstructionStateV1 {
    revision: u64,
    status: &'static str,
    updated_at: i64,
}

#[derive(Debug, Serialize)]
struct UiAgentProfileResponseV1 {
    version: u8,
    profile: UiAgentProfileDetailV1,
}

#[derive(Debug, Serialize)]
struct UiAgentProfileDetailV1 {
    agent_id: String,
    display_name: String,
    labels: Vec<String>,
    avatar: UiAgentAvatarV1,
    instructions: String,
    revision: u64,
    instruction_state: UiInstructionStateV1,
    updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UiAgentProfileUpdateRequestV1 {
    version: u8,
    expected_revision: u64,
    display_name: String,
    labels: Vec<String>,
    avatar: UiAgentAvatarV1Input,
    instructions: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UiAgentAvatarV1Input {
    shape: String,
    color: String,
}

#[derive(Debug, Serialize)]
struct UiAttentionResponseV1 {
    version: u8,
    events: Vec<UiAttentionEventV1>,
    next_after: Option<String>,
}

#[derive(Debug, Serialize)]
struct UiAttentionEventV1 {
    event_id: String,
    agent_id: String,
    display_name: String,
    event_type: &'static str,
    summary: String,
    created_at: i64,
    seen: bool,
    native_notified: bool,
    attention_enabled: bool,
}

#[derive(Debug, Serialize)]
struct UiAttentionPreferencesResponseV1 {
    version: u8,
    preferences: Vec<UiAttentionPreferenceV1>,
}

#[derive(Debug, Serialize)]
struct UiAttentionPreferenceV1 {
    agent_id: String,
    human_input_needed: bool,
    task_completed: bool,
    terminal_failure: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UiAttentionPreferencesRequestV1 {
    version: u8,
    client_id: String,
    preferences: Vec<UiAttentionPreferenceInputV1>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UiAttentionPreferenceInputV1 {
    agent_id: String,
    human_input_needed: bool,
    task_completed: bool,
    terminal_failure: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UiAttentionStateRequestV1 {
    version: u8,
    client_id: String,
    event_ids: Vec<String>,
    action: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UiSendRequestV1 {
    version: u8,
    to: String,
    body: String,
    reply: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UiStopRequestV1 {
    version: u8,
    agent_id: String,
    command_id: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct UiStopAcceptedV1 {
    version: u8,
    agent_id: String,
    command_id: String,
    outcome: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    survivors_killed: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation: Option<u64>,
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
    /// Identité stable de la bulle UI et de l'entrée durable du ledger.
    /// Elle diffère du `delivery_id` lorsque le daemon retourne OutcomeUnknown.
    message_id: String,
    delivery_id: String,
    issued_at: i64,
    status: &'static str,
}

#[derive(Debug, Serialize)]
struct UiSendErrorV1 {
    version: u8,
    code: &'static str,
    message: String,
}

#[derive(Debug, Serialize)]
struct UiArtifactListV1 {
    version: u8,
    scope: &'static str,
    items: Vec<crate::artifact_store::ArtifactListItem>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UiArtifactShareRequestV1 {
    version: u8,
    agent: String,
    version_ref: String,
    target_agent: String,
}

#[derive(Debug, Serialize)]
struct UiArtifactShareAcceptedV1 {
    version: u8,
    reference_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UiArtifactLifecycleRequestV1 {
    version: u8,
    agent: String,
    version_ref: String,
    action: String,
    #[serde(default)]
    pinned: Option<bool>,
}

#[derive(Debug, Serialize)]
struct UiArtifactLifecycleAcceptedV1 {
    version: u8,
    action: String,
    artifact_ref: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UiArtifactSandboxSaveRequestV1 {
    version: u8,
    agent: String,
    version_ref: String,
    ui_state: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct UiArtifactSandboxSaveAcceptedV1 {
    version: u8,
    artifact_ref: String,
    version_ref: String,
    parent_version_ref: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UiArtifactSandboxTicketRequestV1 {
    version: u8,
    agent: String,
    version_ref: String,
}

#[derive(Debug, Serialize)]
struct UiArtifactSandboxTicketAcceptedV1 {
    version: u8,
    ticket: String,
    expires_in_seconds: u64,
}

#[derive(Debug, Serialize)]
struct UiRelayStateV1 {
    version: u8,
    kind: &'static str,
    state: &'static str,
    since: i64,
}

#[derive(Debug, Serialize)]
struct UiProjectRuntimeV1 {
    version: u8,
    project_id: String,
    binding_generation: u64,
    state: String,
    policy_id: String,
    policy_version: u64,
    environment_epoch: u64,
    last_reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct UiProjectSettingsV1 {
    version: u8,
    policy_generation: u64,
    allowed_project_roots: Vec<String>,
    configuration_available: bool,
}

#[derive(Debug, Serialize)]
struct UiProjectListV1 {
    version: u8,
    projects: Vec<UiProjectListEntryV1>,
}

#[derive(Debug, Serialize)]
struct UiProjectListEntryV1 {
    project_id: String,
    display_name: String,
    canonical_path: String,
    state: &'static str,
    binding_generation: u64,
    round: UiProjectRoundV1,
}

#[derive(Debug, Serialize)]
struct UiProjectRoundV1 {
    configured: bool,
    enabled: bool,
    revision: u64,
    updated_at: i64,
    interval_secs: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_occurrence_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_dispatch_state: Option<ProjectRoundDispatchState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_dispatch_observed_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UiProjectRoundRequestV1 {
    version: u8,
    command_id: String,
    project_id: String,
    binding_generation: u64,
    enabled: bool,
}

#[derive(Debug, Serialize)]
struct UiProjectRoundAcceptedV1 {
    version: u8,
    command_id: String,
    project_id: String,
    binding_generation: u64,
    round: UiProjectRoundV1,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UiProjectConfirmRequestV1 {
    version: u8,
    command_id: String,
    mode: String,
    root: String,
    #[serde(default)]
    folder_name: Option<String>,
    initialize_git: bool,
}

#[derive(Debug, Serialize)]
struct UiProjectConfirmedV1 {
    version: u8,
    project_id: String,
    display_name: String,
    canonical_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UiProjectAdminRequestV1 {
    version: u8,
    command_id: String,
    project_id: String,
    #[serde(default)]
    root: Option<String>,
}

#[derive(Debug, Serialize)]
struct UiProjectAdminAcceptedV1 {
    version: u8,
    project: UiProjectListEntryV1,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UiProjectRootsUpdateV1 {
    version: u8,
    command_id: String,
    expected_generation: u64,
    allowed_project_roots: Vec<String>,
}

#[derive(Debug, Serialize)]
struct UiProjectRootsUpdateAcceptedV1 {
    version: u8,
    command_id: String,
    policy_generation: u64,
    allowed_project_roots: Vec<String>,
}

#[derive(Debug, Serialize)]
struct UiControlCategoryV1 {
    key: &'static str,
    scope: &'static str,
    access: &'static str,
    summary: &'static str,
}

#[derive(Debug, Serialize)]
struct UiServerControlSettingsV1 {
    version: u8,
    configuration_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    policy_generation: Option<u64>,
    allowed_project_roots: Vec<String>,
    categories: Vec<UiControlCategoryV1>,
    daemon_version: &'static str,
    update_status: &'static str,
}

#[derive(Debug, Serialize)]
struct UiControlProjectRootsPreviewV1 {
    version: u8,
    command_id: String,
    expected_generation: u64,
    current_roots: Vec<String>,
    requested_roots: Vec<String>,
    resulting_generation: u64,
}

#[derive(Debug, Serialize)]
struct UiControlProjectRootsAppliedV1 {
    version: u8,
    command_id: String,
    expected_generation: u64,
    resulting_generation: u64,
    allowed_project_roots: Vec<String>,
    observed_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UiProjectPreviewRequestV1 {
    version: u8,
    mode: String,
    root: String,
    #[serde(default)]
    folder_name: Option<String>,
}

#[derive(Debug, Serialize)]
struct UiProjectPreviewV1 {
    version: u8,
    mode: &'static str,
    canonical_path: String,
    display_name: String,
    git: &'static str,
    git_initialization_proposed: bool,
}

/// Projection serveur locale au tunnel courant. Ce contrat ne contient ni
/// chemin de configuration brut, ni secret, ni commande hôte générale.
#[derive(Debug, Serialize)]
struct UiUsageDashboardV1 {
    version: u8,
    period: &'static str,
    from_secs: i64,
    to_secs: i64,
    rows: Vec<UiUsageDashboardRowV1>,
    /// Aucun prix n'est injecté sans catalogue versionné et daté. La présence
    /// de cette règle est un garde-fou contre une fausse facture API.
    pricing_status: &'static str,
}

#[derive(Debug, Serialize)]
struct UiUsageDashboardRowV1 {
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    source: String,
    samples: u64,
    input_tokens: u64,
    output_tokens: u64,
    cache_creation_input_tokens: u64,
    cache_read_input_tokens: u64,
    total_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    cost_estimate_microunits: Option<u64>,
}

#[derive(Debug, Serialize)]
struct UiFilePreviewV1 {
    version: u8,
    path: String,
    media_type: &'static str,
    encoding: &'static str,
    content: String,
    truncated: bool,
    sha256: String,
}

#[derive(Debug, Serialize)]
struct UiArtifactDetailV1 {
    version: u8,
    item: crate::artifact_store::ArtifactListItem,
    is_current: bool,
    publication: crate::artifact_types::ArtifactPublicationV1,
    content_digest: String,
    payload_digest: String,
    provenance_digest: String,
    parent_version_ref: Option<String>,
    quality_notices: Vec<String>,
    publication_reason: String,
}

#[derive(Debug, Serialize)]
struct UiArtifactDataPageV1 {
    version: u8,
    version_ref: String,
    offset: usize,
    limit: usize,
    has_more: bool,
    data: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct UiArtifactMetricsV1 {
    version: u8,
    canonical_blob_bytes: u64,
    publication_failures: u64,
    restored_versions: u64,
    evictions: u64,
}

#[derive(Serialize)]
struct UiRecoveryLossV1 {
    display_name: String,
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
    if matches!(
        request.path.as_str(),
        "/v1/send" | "/v1/agents/stop" | "/v1/agents/relaunch" | "/v1/agents/decommission"
    ) && request.method != "POST"
    {
        return write_text(stream, 405, "méthode non autorisée");
    }
    if request.method != "GET" && request.method != "POST" && request.method != "PATCH" {
        return write_text(stream, 405, "méthode non autorisée");
    }
    let if_none_match = request.headers.get("if-none-match").map(String::as_str);
    if request.method == "GET" {
        match request.path.as_str() {
            "/app.js" => {
                return write_asset(
                    stream,
                    "application/javascript; charset=utf-8",
                    UI_SCRIPT,
                    if_none_match,
                );
            }
            "/artifact-renderer.js" => {
                return write_asset(
                    stream,
                    "application/javascript; charset=utf-8",
                    UI_ARTIFACT_RENDERER,
                    if_none_match,
                );
            }
            "/artifact-sandbox-host.js" => {
                return write_asset(
                    stream,
                    "application/javascript; charset=utf-8",
                    UI_ARTIFACT_SANDBOX_HOST,
                    if_none_match,
                );
            }
            "/theme.css" => {
                return write_asset(stream, "text/css; charset=utf-8", UI_THEME, if_none_match);
            }
            "/vendor/marked.min.js" => {
                return write_asset(
                    stream,
                    "application/javascript; charset=utf-8",
                    UI_MARKED,
                    if_none_match,
                );
            }
            "/vendor/purify.min.js" => {
                return write_asset(
                    stream,
                    "application/javascript; charset=utf-8",
                    UI_PURIFY,
                    if_none_match,
                );
            }
            "/vendor/highlight.min.js" => {
                return write_asset(
                    stream,
                    "application/javascript; charset=utf-8",
                    UI_HIGHLIGHT,
                    if_none_match,
                );
            }
            "/vendor/echarts.min.js" => {
                return write_asset(
                    stream,
                    "application/javascript; charset=utf-8",
                    UI_ECHARTS,
                    if_none_match,
                );
            }
            "/vendor/tabulator.min.js" => {
                return write_asset(
                    stream,
                    "application/javascript; charset=utf-8",
                    UI_TABULATOR,
                    if_none_match,
                );
            }
            "/vendor/tabulator.min.css" => {
                return write_asset(
                    stream,
                    "text/css; charset=utf-8",
                    UI_TABULATOR_CSS,
                    if_none_match,
                );
            }
            "/vendor/highlight-github-dark.min.css" => {
                return write_asset(
                    stream,
                    "text/css; charset=utf-8",
                    UI_HIGHLIGHT_GITHUB_DARK,
                    if_none_match,
                );
            }
            "/vendor/highlight-github-light.min.css" => {
                return write_asset(
                    stream,
                    "text/css; charset=utf-8",
                    UI_HIGHLIGHT_GITHUB_LIGHT,
                    if_none_match,
                );
            }
            "/providers/openai.svg" => {
                return write_asset(stream, "image/svg+xml", UI_PROVIDER_OPENAI, if_none_match);
            }
            "/providers/claude.svg" => {
                return write_asset(stream, "image/svg+xml", UI_PROVIDER_CLAUDE, if_none_match);
            }
            "/providers/cursor.svg" => {
                return write_asset(stream, "image/svg+xml", UI_PROVIDER_CURSOR, if_none_match);
            }
            "/providers/gemini.svg" => {
                return write_asset(stream, "image/svg+xml", UI_PROVIDER_GEMINI, if_none_match);
            }
            "/providers/glm.svg" => {
                return write_asset(stream, "image/svg+xml", UI_PROVIDER_GLM, if_none_match);
            }
            "/providers/deepseek.svg" => {
                return write_asset(stream, "image/svg+xml", UI_PROVIDER_DEEPSEEK, if_none_match);
            }
            _ => {}
        }
    }
    if request.query.get("token") != Some(&config.token) {
        return write_text(stream, 403, "jeton UI invalide");
    }
    match (request.method.as_str(), request.path.as_str()) {
        ("GET", "/") | ("GET", "/browser-panel") => {
            write_asset(stream, "text/html; charset=utf-8", UI_INDEX, if_none_match)
        }
        ("GET", "/v1/content/file-preview") => {
            match read_content_file_preview(config, request.query.get("path").map(String::as_str)) {
                Ok(response) => write_json(stream, 200, &response),
                Err((status, code, message)) => write_json(
                    stream,
                    status,
                    &UiSendErrorV1 {
                        version: UI_VERSION,
                        code,
                        message,
                    },
                ),
            }
        }
        ("GET", path) if path.starts_with("/v1/agents/") && path.ends_with("/profile") => {
            match get_agent_profile(config, agent_id_from_profile_path(path)) {
                Ok(response) => write_json(stream, 200, &response),
                Err((status, code, message)) => write_json(
                    stream,
                    status,
                    &UiSendErrorV1 {
                        version: UI_VERSION,
                        code,
                        message,
                    },
                ),
            }
        }
        ("PATCH", path) if path.starts_with("/v1/agents/") && path.ends_with("/profile") => {
            match patch_agent_profile(config, agent_id_from_profile_path(path), &request.body) {
                Ok(response) => write_json(stream, 200, &response),
                Err((status, code, message)) => write_json(
                    stream,
                    status,
                    &UiSendErrorV1 {
                        version: UI_VERSION,
                        code,
                        message,
                    },
                ),
            }
        }
        ("GET", "/v1/attention") => {
            match get_attention(
                config,
                request.query.get("client_id").map(String::as_str),
                request.query.get("after").map(String::as_str),
                request.query.get("limit").map(String::as_str),
            ) {
                Ok(response) => write_json(stream, 200, &response),
                Err((status, code, message)) => write_json(
                    stream,
                    status,
                    &UiSendErrorV1 {
                        version: UI_VERSION,
                        code,
                        message,
                    },
                ),
            }
        }
        ("GET", "/v1/attention/preferences") => {
            match get_attention_preferences(
                config,
                request.query.get("client_id").map(String::as_str),
            ) {
                Ok(response) => write_json(stream, 200, &response),
                Err((status, code, message)) => write_json(
                    stream,
                    status,
                    &UiSendErrorV1 {
                        version: UI_VERSION,
                        code,
                        message,
                    },
                ),
            }
        }
        ("PUT", "/v1/attention/preferences") => {
            match put_attention_preferences(config, &request.body) {
                Ok(response) => write_json(stream, 200, &response),
                Err((status, code, message)) => write_json(
                    stream,
                    status,
                    &UiSendErrorV1 {
                        version: UI_VERSION,
                        code,
                        message,
                    },
                ),
            }
        }
        ("POST", "/v1/attention/state") => match post_attention_state(config, &request.body) {
            Ok(()) => write_json(stream, 204, &serde_json::json!({})),
            Err((status, code, message)) => write_json(
                stream,
                status,
                &UiSendErrorV1 {
                    version: UI_VERSION,
                    code,
                    message,
                },
            ),
        },
        ("POST", "/v1/send") => match post_ui_message(config, runtime, &request.body) {
            Ok(response) => write_json(stream, 202, &response),
            Err((status, code, message)) => write_json(
                stream,
                status,
                &UiSendErrorV1 {
                    version: UI_VERSION,
                    code,
                    message,
                },
            ),
        },
        ("POST", "/v1/agents/stop") => match post_ui_stop(config, &request.body) {
            Ok(response) => write_json(stream, 200, &response),
            Err((status, code, message)) => write_json(
                stream,
                status,
                &UiSendErrorV1 {
                    version: UI_VERSION,
                    code,
                    message,
                },
            ),
        },
        ("POST", "/v1/agents/relaunch") => match post_ui_relaunch(config, &request.body) {
            Ok(response) => write_json(stream, 200, &response),
            Err((status, code, message)) => write_json(
                stream,
                status,
                &UiSendErrorV1 {
                    version: UI_VERSION,
                    code,
                    message,
                },
            ),
        },
        ("POST", "/v1/agents/decommission") => match post_ui_decommission(config, &request.body) {
            Ok(response) => write_json(stream, 200, &response),
            Err((status, code, message)) => write_json(
                stream,
                status,
                &UiSendErrorV1 {
                    version: UI_VERSION,
                    code,
                    message,
                },
            ),
        },
        ("GET", "/v1/projects/runtime") => {
            let project_id = request
                .query
                .get("project")
                .ok_or_else(|| UiError::Protocol("paramètre project absent".to_string()))?;
            match read_project_runtime(&config.daemon_socket, project_id) {
                Ok(response) => write_json(stream, 200, &response),
                Err((status, code, message)) => write_json(
                    stream,
                    status,
                    &UiSendErrorV1 {
                        version: UI_VERSION,
                        code,
                        message,
                    },
                ),
            }
        }
        ("GET", "/v1/projects/settings") => match read_project_settings(config) {
            Ok(response) => write_json(stream, 200, &response),
            Err((status, code, message)) => write_json(
                stream,
                status,
                &UiSendErrorV1 {
                    version: UI_VERSION,
                    code,
                    message,
                },
            ),
        },
        ("GET", "/v1/projects") => match read_projects(&config.daemon_socket) {
            Ok(response) => write_json(stream, 200, &response),
            Err((status, code, message)) => write_json(
                stream,
                status,
                &UiSendErrorV1 {
                    version: UI_VERSION,
                    code,
                    message,
                },
            ),
        },
        ("GET", "/v1/control/settings") => match read_server_control_settings(config) {
            Ok(response) => write_json(stream, 200, &response),
            Err((status, code, message)) => write_json(
                stream,
                status,
                &UiSendErrorV1 {
                    version: UI_VERSION,
                    code,
                    message,
                },
            ),
        },
        ("POST", "/v1/control/settings/preview") => {
            match post_control_project_roots_preview(config, &request.body) {
                Ok(response) => write_json(stream, 200, &response),
                Err((status, code, message)) => write_json(
                    stream,
                    status,
                    &UiSendErrorV1 {
                        version: UI_VERSION,
                        code,
                        message,
                    },
                ),
            }
        }
        ("POST", "/v1/control/settings/apply") => {
            match post_control_project_roots_apply(config, &request.body) {
                Ok(response) => write_json(stream, 200, &response),
                Err((status, code, message)) => write_json(
                    stream,
                    status,
                    &UiSendErrorV1 {
                        version: UI_VERSION,
                        code,
                        message,
                    },
                ),
            }
        }
        ("GET", "/v1/usage") => {
            match read_usage_dashboard(config, request.query.get("period").map(String::as_str)) {
                Ok(response) => write_json(stream, 200, &response),
                Err((status, code, message)) => write_json(
                    stream,
                    status,
                    &UiSendErrorV1 {
                        version: UI_VERSION,
                        code,
                        message,
                    },
                ),
            }
        }
        ("GET", "/v1/control/usage") => {
            match read_usage_dashboard(config, request.query.get("period").map(String::as_str)) {
                Ok(response) => write_json(stream, 200, &response),
                Err((status, code, message)) => write_json(
                    stream,
                    status,
                    &UiSendErrorV1 {
                        version: UI_VERSION,
                        code,
                        message,
                    },
                ),
            }
        }
        ("POST", "/v1/projects/preview") => match post_project_preview(config, &request.body) {
            Ok(response) => write_json(stream, 200, &response),
            Err((status, code, message)) => write_json(
                stream,
                status,
                &UiSendErrorV1 {
                    version: UI_VERSION,
                    code,
                    message,
                },
            ),
        },
        ("POST", "/v1/projects/confirm") => match post_project_confirm(config, &request.body) {
            Ok(response) => write_json(stream, 200, &response),
            Err((status, code, message)) => write_json(
                stream,
                status,
                &UiSendErrorV1 {
                    version: UI_VERSION,
                    code,
                    message,
                },
            ),
        },
        ("POST", "/v1/projects/round") => match post_project_round(config, &request.body) {
            Ok(response) => write_json(stream, 200, &response),
            Err((status, code, message)) => write_json(
                stream,
                status,
                &UiSendErrorV1 {
                    version: UI_VERSION,
                    code,
                    message,
                },
            ),
        },
        ("POST", "/v1/projects/disable") => {
            match post_project_admin(config, &request.body, ProjectAdminOperation::Disable) {
                Ok(response) => write_json(stream, 200, &response),
                Err((status, code, message)) => write_json(
                    stream,
                    status,
                    &UiSendErrorV1 {
                        version: UI_VERSION,
                        code,
                        message,
                    },
                ),
            }
        }
        ("POST", "/v1/projects/activate") => {
            match post_project_admin(config, &request.body, ProjectAdminOperation::Activate) {
                Ok(response) => write_json(stream, 200, &response),
                Err((status, code, message)) => write_json(
                    stream,
                    status,
                    &UiSendErrorV1 {
                        version: UI_VERSION,
                        code,
                        message,
                    },
                ),
            }
        }
        ("POST", "/v1/projects/rebind") => {
            match post_project_admin(config, &request.body, ProjectAdminOperation::Rebind) {
                Ok(response) => write_json(stream, 200, &response),
                Err((status, code, message)) => write_json(
                    stream,
                    status,
                    &UiSendErrorV1 {
                        version: UI_VERSION,
                        code,
                        message,
                    },
                ),
            }
        }
        ("POST", "/v1/projects/settings") => {
            match post_project_roots_update(config, &request.body) {
                Ok(response) => write_json(stream, 200, &response),
                Err((status, code, message)) => write_json(
                    stream,
                    status,
                    &UiSendErrorV1 {
                        version: UI_VERSION,
                        code,
                        message,
                    },
                ),
            }
        }
        ("POST", "/v1/search") => match post_ui_search(config, &request.body) {
            Ok(response) => write_json(stream, 200, &response),
            Err((status, message)) => write_text(stream, status, message),
        },
        ("GET", "/v1/artifacts/list") => match read_artifact_list(config, &request.query, false) {
            Ok(response) => write_json(stream, 200, &response),
            Err((status, code, message)) => write_json(
                stream,
                status,
                &UiSendErrorV1 {
                    version: UI_VERSION,
                    code,
                    message,
                },
            ),
        },
        ("GET", "/v1/artifacts/search") => match read_artifact_list(config, &request.query, true) {
            Ok(response) => write_json(stream, 200, &response),
            Err((status, code, message)) => write_json(
                stream,
                status,
                &UiSendErrorV1 {
                    version: UI_VERSION,
                    code,
                    message,
                },
            ),
        },
        ("GET", "/v1/artifacts/metrics") => match read_artifact_metrics(config, &request.query) {
            Ok(response) => write_json(stream, 200, &response),
            Err((status, code, message)) => write_json(
                stream,
                status,
                &UiSendErrorV1 {
                    version: UI_VERSION,
                    code,
                    message,
                },
            ),
        },
        ("POST", "/v1/artifacts/share") => match post_artifact_share(config, &request.body) {
            Ok(response) => write_json(stream, 200, &response),
            Err((status, code, message)) => write_json(
                stream,
                status,
                &UiSendErrorV1 {
                    version: UI_VERSION,
                    code,
                    message,
                },
            ),
        },
        ("POST", "/v1/artifacts/lifecycle") => {
            match post_artifact_lifecycle(config, &request.body) {
                Ok(response) => write_json(stream, 200, &response),
                Err((status, code, message)) => write_json(
                    stream,
                    status,
                    &UiSendErrorV1 {
                        version: UI_VERSION,
                        code,
                        message,
                    },
                ),
            }
        }
        ("POST", "/v1/artifacts/sandbox/save") => {
            match post_artifact_sandbox_save(config, &request.body) {
                Ok(response) => write_json(stream, 200, &response),
                Err((status, code, message)) => write_json(
                    stream,
                    status,
                    &UiSendErrorV1 {
                        version: UI_VERSION,
                        code,
                        message,
                    },
                ),
            }
        }
        ("POST", "/v1/artifacts/sandbox/ticket") => {
            match post_artifact_sandbox_ticket(config, runtime, &request.body) {
                Ok(response) => write_json(stream, 200, &response),
                Err((status, code, message)) => write_json(
                    stream,
                    status,
                    &UiSendErrorV1 {
                        version: UI_VERSION,
                        code,
                        message,
                    },
                ),
            }
        }
        ("GET", "/v1/artifacts/sandbox/frame") => {
            match read_artifact_sandbox_frame(
                config,
                runtime,
                request.query.get("ticket"),
                request.query.get("frame_instance_id"),
                request.query.get("theme"),
            ) {
                Ok(html) => write_sandbox_html(stream, &html),
                Err((status, code, message)) => write_json(
                    stream,
                    status,
                    &UiSendErrorV1 {
                        version: UI_VERSION,
                        code,
                        message,
                    },
                ),
            }
        }
        ("GET", "/v1/artifacts/detail") => {
            match read_artifact_detail(
                config,
                request.query.get("agent"),
                request.query.get("version"),
            ) {
                Ok(response) => write_json(stream, 200, &response),
                Err((status, code, message)) => write_json(
                    stream,
                    status,
                    &UiSendErrorV1 {
                        version: UI_VERSION,
                        code,
                        message,
                    },
                ),
            }
        }
        ("GET", "/v1/artifacts/data") => match read_artifact_data_page(config, &request.query) {
            Ok(response) => write_json(stream, 200, &response),
            Err((status, code, message)) => write_json(
                stream,
                status,
                &UiSendErrorV1 {
                    version: UI_VERSION,
                    code,
                    message,
                },
            ),
        },
        ("GET", "/v1/artifacts/export") => match artifact_export(config, &request.query) {
            Ok((content_type, bytes, filename)) => {
                write_binary(stream, 200, content_type, &bytes, Some(&filename))
            }
            Err((status, code, message)) => write_json(
                stream,
                status,
                &UiSendErrorV1 {
                    version: UI_VERSION,
                    code,
                    message,
                },
            ),
        },
        ("GET", "/v1/artifacts/blob") => match read_artifact_blob(config, &request.query) {
            Ok((content_type, bytes)) => write_binary(stream, 200, content_type, &bytes, None),
            Err((status, code, message)) => write_json(
                stream,
                status,
                &UiSendErrorV1 {
                    version: UI_VERSION,
                    code,
                    message,
                },
            ),
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
            // Un participant sans journal de tour — l'humain, qui écrit depuis
            // l'interface et n'est piloté par aucun wrapper — n'a pas de
            // répertoire sous ~/.cache/bridget/sessions/. L'attache échouait
            // alors APRÈS l'envoi du 200, laissant un corps vide : mesuré le
            // 28/08, `agent=humain` rendait 0 octet quand `agent=jc2` en
            // rendait 26170. Un corps vide ne dit pas « pas de journal », il
            // ressemble à une panne. On sert ici ses messages, par le même
            // mécanisme que /v1/watch. Le chemin des agents pilotés est
            // inchangé : ils ont des tours, donc la condition est fausse.
            if projected_turns(&agent_journal_dir(agent)).is_empty() {
                return stream_sse_thread_only(stream, config, agent);
            }
            stream_sse_journal(stream, &config.daemon_socket, agent, window, page, None)
        }
        ("GET", "/v1/watch") => {
            let agent = request
                .query
                .get("agent")
                .ok_or_else(|| UiError::Protocol("paramètre agent absent".to_string()))?;
            validate_agent(agent)?;
            if projected_turns(&agent_journal_dir(agent)).is_empty() {
                return stream_sse_thread_watch(stream, config, agent);
            }
            let (window, page) = resolve_ui_journal_window(agent, &request.query)?;
            // La vue combinée est la porte d'entrée de la future page : elle
            // raccorde Attach avant de capturer l'instantané, donc aucun delta
            // journal ne peut se glisser silencieusement entre les deux.
            stream_sse_journal(
                stream,
                &config.daemon_socket,
                agent,
                window,
                page,
                Some(config),
            )
        }
        _ => write_text(stream, 404, "ressource UI inconnue"),
    }
}

fn agent_id_from_profile_path(path: &str) -> Option<&str> {
    let agent_id = path.strip_prefix("/v1/agents/")?.strip_suffix("/profile")?;
    (!agent_id.is_empty() && !agent_id.contains('/')).then_some(agent_id)
}

fn open_agent_profile_store(
    config: &UiRelayConfig,
) -> Result<AgentProfileStore, (u16, &'static str, String)> {
    AgentProfileStore::open(&ledger_db_path_for_socket(&config.daemon_socket)).map_err(|_| {
        (
            503,
            "profile_store_unavailable",
            "profils temporairement indisponibles".to_string(),
        )
    })
}

fn get_agent_profile(
    config: &UiRelayConfig,
    agent_id: Option<&str>,
) -> Result<UiAgentProfileResponseV1, (u16, &'static str, String)> {
    let Some(agent_id) = agent_id else {
        return Err((404, "profile_not_found", "profil indisponible".to_string()));
    };
    let store = open_agent_profile_store(config)?;
    let profile = store
        .profile_detail(agent_id)
        .map_err(profile_error_response)?;
    Ok(UiAgentProfileResponseV1 {
        version: UI_VERSION,
        profile: ui_agent_profile_detail(profile),
    })
}

fn patch_agent_profile(
    config: &UiRelayConfig,
    agent_id: Option<&str>,
    body: &[u8],
) -> Result<UiAgentProfileResponseV1, (u16, &'static str, String)> {
    let Some(agent_id) = agent_id else {
        return Err((404, "profile_not_found", "profil indisponible".to_string()));
    };
    let request: UiAgentProfileUpdateRequestV1 = serde_json::from_slice(body)
        .map_err(|_| (400, "invalid_profile", "profil invalide".to_string()))?;
    if request.version != UI_VERSION || request.expected_revision == 0 {
        return Err((400, "invalid_profile", "profil invalide".to_string()));
    }
    let mut store = open_agent_profile_store(config)?;
    let profile = store
        .update_profile(
            agent_id,
            AgentProfileUpdate {
                expected_revision: request.expected_revision,
                display_name: request.display_name,
                labels: request.labels,
                avatar_shape: request.avatar.shape,
                avatar_color: request.avatar.color,
                instructions: request.instructions,
            },
        )
        .map_err(profile_error_response)?;
    Ok(UiAgentProfileResponseV1 {
        version: UI_VERSION,
        profile: ui_agent_profile_detail(profile),
    })
}

fn profile_error_response(error: AgentProfileError) -> (u16, &'static str, String) {
    match error {
        AgentProfileError::Invalid(_) => (400, "invalid_profile", "profil invalide".to_string()),
        AgentProfileError::NotFound => {
            (404, "profile_not_found", "profil indisponible".to_string())
        }
        AgentProfileError::RevisionConflict => (
            409,
            "profile_revision_conflict",
            "profil modifié entre-temps".to_string(),
        ),
        AgentProfileError::DisplayNameConflict => (
            409,
            "display_name_conflict",
            "nom affiché déjà utilisé".to_string(),
        ),
        AgentProfileError::Sqlite(_) => (
            503,
            "profile_store_unavailable",
            "profils temporairement indisponibles".to_string(),
        ),
    }
}

fn ui_agent_profile_detail(profile: AgentProfileDetail) -> UiAgentProfileDetailV1 {
    let summary = profile.summary;
    UiAgentProfileDetailV1 {
        agent_id: summary.agent_id,
        display_name: summary.display_name,
        labels: summary.labels,
        avatar: UiAgentAvatarV1 {
            shape: summary.avatar_shape,
            color: summary.avatar_color,
        },
        instructions: profile.instructions,
        revision: summary.revision,
        instruction_state: UiInstructionStateV1 {
            revision: summary.instructions_revision,
            status: summary.instruction_status.as_str(),
            updated_at: summary.updated_at,
        },
        updated_at: summary.updated_at,
    }
}

fn get_attention(
    config: &UiRelayConfig,
    client_id: Option<&str>,
    after: Option<&str>,
    limit: Option<&str>,
) -> Result<UiAttentionResponseV1, (u16, &'static str, String)> {
    let client_id = client_id.ok_or_else(|| attention_error("client absent"))?;
    let limit = limit
        .map(str::parse::<usize>)
        .transpose()
        .map_err(|_| attention_error("limite invalide"))?
        .unwrap_or(100);
    let store = open_agent_profile_store(config)?;
    let (events, next_after) = store
        .attention_for_client(client_id, after, limit)
        .map_err(attention_error_response)?;
    Ok(UiAttentionResponseV1 {
        version: UI_VERSION,
        events: events.into_iter().map(ui_attention_event).collect(),
        next_after,
    })
}

fn get_attention_preferences(
    config: &UiRelayConfig,
    client_id: Option<&str>,
) -> Result<UiAttentionPreferencesResponseV1, (u16, &'static str, String)> {
    let client_id = client_id.ok_or_else(|| attention_error("client absent"))?;
    let store = open_agent_profile_store(config)?;
    Ok(UiAttentionPreferencesResponseV1 {
        version: UI_VERSION,
        preferences: store
            .preferences_for_client(client_id)
            .map_err(attention_error_response)?
            .into_iter()
            .map(ui_attention_preference)
            .collect(),
    })
}

fn put_attention_preferences(
    config: &UiRelayConfig,
    body: &[u8],
) -> Result<UiAttentionPreferencesResponseV1, (u16, &'static str, String)> {
    let request: UiAttentionPreferencesRequestV1 =
        serde_json::from_slice(body).map_err(|_| attention_error("préférences invalides"))?;
    if request.version != UI_VERSION {
        return Err(attention_error("préférences invalides"));
    }
    let preferences = request
        .preferences
        .into_iter()
        .map(|preference| ClientNotificationPreference {
            agent_id: preference.agent_id,
            human_input_needed: preference.human_input_needed,
            task_completed: preference.task_completed,
            terminal_failure: preference.terminal_failure,
        })
        .collect::<Vec<_>>();
    let mut store = open_agent_profile_store(config)?;
    Ok(UiAttentionPreferencesResponseV1 {
        version: UI_VERSION,
        preferences: store
            .replace_preferences_for_client(&request.client_id, &preferences)
            .map_err(attention_error_response)?
            .into_iter()
            .map(ui_attention_preference)
            .collect(),
    })
}

fn post_attention_state(
    config: &UiRelayConfig,
    body: &[u8],
) -> Result<(), (u16, &'static str, String)> {
    let request: UiAttentionStateRequestV1 =
        serde_json::from_slice(body).map_err(|_| attention_error("état d'attention invalide"))?;
    if request.version != UI_VERSION {
        return Err(attention_error("état d'attention invalide"));
    }
    open_agent_profile_store(config)?
        .mark_attention_state(&request.client_id, &request.event_ids, &request.action)
        .map_err(attention_error_response)
}

fn ui_attention_event(event: AttentionEvent) -> UiAttentionEventV1 {
    let summary = match event.event_type {
        AttentionEventType::HumanInputNeeded => {
            format!("{} attend votre réponse.", event.display_name)
        }
        AttentionEventType::TaskCompleted => format!("{} a terminé.", event.display_name),
        AttentionEventType::TerminalFailure => {
            format!("{} nécessite une vérification.", event.display_name)
        }
    };
    UiAttentionEventV1 {
        event_id: event.event_id,
        agent_id: event.agent_id,
        display_name: event.display_name,
        event_type: event.event_type.as_str(),
        summary,
        created_at: event.created_at,
        seen: event.seen,
        native_notified: event.native_notified,
        attention_enabled: event.attention_enabled,
    }
}

fn ui_attention_preference(preference: ClientNotificationPreference) -> UiAttentionPreferenceV1 {
    UiAttentionPreferenceV1 {
        agent_id: preference.agent_id,
        human_input_needed: preference.human_input_needed,
        task_completed: preference.task_completed,
        terminal_failure: preference.terminal_failure,
    }
}

fn attention_error(_reason: &'static str) -> (u16, &'static str, String) {
    (400, "invalid_attention", "activité invalide".to_string())
}

fn attention_error_response(error: AgentProfileError) -> (u16, &'static str, String) {
    match error {
        AgentProfileError::NotFound => (
            404,
            "attention_not_found",
            "activité indisponible".to_string(),
        ),
        AgentProfileError::Sqlite(_) => (
            503,
            "attention_store_unavailable",
            "activité temporairement indisponible".to_string(),
        ),
        AgentProfileError::Invalid(_)
        | AgentProfileError::RevisionConflict
        | AgentProfileError::DisplayNameConflict => attention_error("activité invalide"),
    }
}

fn post_ui_message(
    config: &UiRelayConfig,
    runtime: &UiRelayRuntime,
    body: &[u8],
) -> Result<UiSendAcceptedV1, (u16, &'static str, String)> {
    let request: UiSendRequestV1 = serde_json::from_slice(body)
        .map_err(|_| (400, "invalid_body", "corps JSON invalide".to_string()))?;
    if request.version != UI_VERSION || request.body.trim().is_empty() {
        return Err((400, "invalid_body", "corps de message invalide".to_string()));
    }
    validate_agent(&request.to)
        .map_err(|_| (404, "unknown_recipient", "destinataire inconnu".to_string()))?;

    let agents = read_agent_list(&config.daemon_socket)
        .map_err(|error| (503, "daemon_unavailable", error.to_string()))?;
    validate_ui_recipient(&agents, &request.to).map_err(|(status, code)| {
        let message = if code == "human_recipient" {
            "l’identité locale ne peut pas recevoir de message"
        } else {
            "destinataire indisponible"
        };
        (status, code, message.to_string())
    })?;
    // Ré-assurée à chaque envoi, que la réponse soit attendue ou non : c'est
    // ce qui rouvre l'inscription après un redémarrage du daemon. Quand aucune
    // réponse n'est attendue, un échec ne doit pas faire perdre le message —
    // le sens humain -> agent prime sur l'inscription.
    if let Err(error) = runtime.ensure_human_presence(&config.daemon_socket) {
        if request.reply {
            return Err((503, "human_sender_unregistered", error.to_string()));
        }
        eprintln!("relais UI: inscription de l'humain à l'annuaire impossible: {error}");
    }
    match send_ui_message(&config.daemon_socket, request.clone()) {
        Ok(accepted) => Ok(accepted),
        // Le daemon est l'autorité : ce refus prouve que la socket de présence
        // est périmée malgré son marqueur local. La rouvrir, puis rejouer une
        // seule fois exactement la même demande évite de faire choisir entre
        // envoyer le message et attendre une réponse.
        Err(error) if request.reply && is_reply_sender_unavailable(&error) => {
            runtime.invalidate_human_presence();
            runtime
                .ensure_human_presence(&config.daemon_socket)
                .map_err(|error| (503, "human_sender_unregistered", error.to_string()))?;
            send_ui_message(&config.daemon_socket, request)
                .map_err(|error| (503, "send_failed", error.to_string()))
        }
        Err(error) => Err((503, "send_failed", error.to_string())),
    }
}

fn is_reply_sender_unavailable(error: &UiError) -> bool {
    matches!(error, UiError::Protocol(detail) if detail.contains("reply_sender_unavailable"))
}

type UiStopError = (u16, &'static str, String);

fn parse_ui_stop_request(body: &[u8]) -> Result<UiStopRequestV1, UiStopError> {
    parse_ui_lifecycle_request(body, "arrêt")
}

fn parse_ui_lifecycle_request(body: &[u8], action: &str) -> Result<UiStopRequestV1, UiStopError> {
    let request: UiStopRequestV1 = serde_json::from_slice(body).map_err(|_| {
        (
            400,
            "invalid_request",
            format!("Demande de {action} invalide."),
        )
    })?;
    let command_id_valid = !request.command_id.is_empty()
        && request.command_id.len() <= MAX_UI_COMMAND_ID_BYTES
        && request.command_id.bytes().all(is_query_byte);
    if request.version != UI_VERSION
        || validate_agent(&request.agent_id).is_err()
        || !command_id_valid
    {
        return Err((
            400,
            "invalid_request",
            format!("Demande de {action} invalide."),
        ));
    }
    Ok(request)
}

fn validate_ui_stop_target(
    agents: &[bridget_transport::protocol::AgentInfo],
    name: &str,
) -> Result<(), (u16, &'static str)> {
    let Some(agent) = agents.iter().find(|agent| agent.agent_id == name) else {
        return Err((404, "agent_not_found"));
    };
    if agent.persistent.is_none() {
        return Err((409, "agent_not_managed"));
    }
    match agent.state.as_str() {
        "connected" | "busy" | "dnd" | "alive" | "idle" => Ok(()),
        "stopped" => Err((409, "agent_stopped")),
        "recovering" | "relaunching" => Err((409, "lifecycle_in_progress")),
        _ => Err((409, "agent_unavailable")),
    }
}

fn validate_ui_relaunch_target(
    agents: &[bridget_transport::protocol::AgentInfo],
    name: &str,
) -> Result<(), (u16, &'static str)> {
    let Some(agent) = agents.iter().find(|agent| agent.agent_id == name) else {
        return Err((404, "agent_not_found"));
    };
    if agent.persistent.is_none() {
        return Err((409, "agent_not_managed"));
    }
    match agent.state.as_str() {
        "stopped" => Ok(()),
        "connected" | "busy" | "dnd" | "alive" | "idle" => Err((409, "agent_already_running")),
        "recovering" | "relaunching" => Err((409, "lifecycle_in_progress")),
        _ => Err((409, "agent_unavailable")),
    }
}

fn validate_ui_decommission_target(
    agents: &[bridget_transport::protocol::AgentInfo],
    name: &str,
) -> Result<(), (u16, &'static str)> {
    let Some(agent) = agents.iter().find(|agent| agent.agent_id == name) else {
        return Err((404, "agent_not_found"));
    };
    if agent.persistent.is_none() {
        return Err((409, "agent_not_managed"));
    }
    match agent.state.as_str() {
        "connected" | "busy" | "dnd" | "alive" | "idle" | "stopped" => Ok(()),
        "recovering" | "relaunching" => Err((409, "lifecycle_in_progress")),
        _ => Err((409, "agent_unavailable")),
    }
}

fn map_ui_stop_outcome(
    request: &UiStopRequestV1,
    outcome: bridget_transport::protocol::StopOutcome,
) -> Result<UiStopAcceptedV1, UiStopError> {
    use bridget_transport::protocol::StopOutcome;
    match outcome {
        StopOutcome::Stopped => Ok(UiStopAcceptedV1 {
            version: UI_VERSION,
            agent_id: request.agent_id.clone(),
            command_id: request.command_id.clone(),
            outcome: "stopped",
            survivors_killed: None,
            generation: None,
        }),
        StopOutcome::StoppedForced { survivors_killed } => Ok(UiStopAcceptedV1 {
            version: UI_VERSION,
            agent_id: request.agent_id.clone(),
            command_id: request.command_id.clone(),
            outcome: "stopped_forced",
            survivors_killed: Some(survivors_killed),
            generation: None,
        }),
        StopOutcome::NotManaged => Err((
            409,
            "agent_not_managed",
            "Cet agent n'est pas géré par Bridget.".to_string(),
        )),
        StopOutcome::NotFound => Err((
            404,
            "agent_not_found",
            "Cet agent est introuvable.".to_string(),
        )),
        StopOutcome::Timeout { .. } => Err((
            504,
            "stop_timeout",
            "Le daemon n'a pas confirmé l'arrêt dans le délai.".to_string(),
        )),
    }
}

fn read_project_settings(
    config: &UiRelayConfig,
) -> Result<UiProjectSettingsV1, (u16, &'static str, String)> {
    let Some(source) = config.project_root_policy_path.as_deref() else {
        return Err((
            409,
            "project_settings_unavailable",
            "Les racines projet ne sont pas configurées pour ce relais.".to_string(),
        ));
    };
    let policy = ProjectRootPolicy::load(source).map_err(|_| {
        (
            409,
            "project_settings_unavailable",
            "La politique de racines n est pas disponible ou valide.".to_string(),
        )
    })?;
    Ok(UiProjectSettingsV1 {
        version: UI_VERSION,
        policy_generation: policy.generation(),
        allowed_project_roots: policy
            .allowed_roots()
            .iter()
            .map(|root| root.to_string_lossy().into_owned())
            .collect(),
        configuration_available: true,
    })
}

fn content_preview_media_type(path: &Path) -> Option<(&'static str, &'static str)> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    match extension.as_str() {
        "txt" | "log" | "toml" | "yaml" | "yml" | "ini" | "cfg" | "conf" => {
            Some(("text/plain", "utf8"))
        }
        "md" | "markdown" => Some(("text/markdown", "utf8")),
        "rs" => Some(("text/rust", "utf8")),
        "js" | "mjs" | "cjs" | "ts" | "tsx" | "jsx" => Some(("text/javascript", "utf8")),
        "json" => Some(("application/json", "utf8")),
        "html" | "htm" | "css" | "py" | "sh" | "zsh" | "bash" | "sql" | "xml" => {
            Some(("text/plain", "utf8"))
        }
        "png" => Some(("image/png", "base64")),
        "jpg" | "jpeg" => Some(("image/jpeg", "base64")),
        "gif" => Some(("image/gif", "base64")),
        "webp" => Some(("image/webp", "base64")),
        "avif" => Some(("image/avif", "base64")),
        _ => None,
    }
}

fn base64_standard(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = *chunk.get(1).unwrap_or(&0);
        let third = *chunk.get(2).unwrap_or(&0);
        output.push(TABLE[(first >> 2) as usize] as char);
        output.push(TABLE[((first & 0x03) << 4 | (second >> 4)) as usize] as char);
        output.push(if chunk.len() > 1 {
            TABLE[((second & 0x0f) << 2 | (third >> 6)) as usize] as char
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            TABLE[(third & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    output
}

fn read_content_file_preview(
    config: &UiRelayConfig,
    requested_path: Option<&str>,
) -> Result<UiFilePreviewV1, (u16, &'static str, String)> {
    let path = requested_path
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .ok_or_else(|| {
            (
                400,
                "invalid_path",
                "Le chemin d'aperçu doit être absolu.".to_string(),
            )
        })?;
    let policy_path = config.project_root_policy_path.as_deref().ok_or_else(|| {
        (
            403,
            "outside_project_roots",
            "Cet aperçu n'est pas autorisé par la politique de projets.".to_string(),
        )
    })?;
    let policy = ProjectRootPolicy::load(policy_path).map_err(|_| {
        (
            403,
            "outside_project_roots",
            "Cet aperçu n'est pas autorisé par la politique de projets.".to_string(),
        )
    })?;
    let canonical = std::fs::canonicalize(&path).map_err(|error| {
        if error.kind() == ErrorKind::NotFound {
            (
                404,
                "not_found",
                "Le fichier demandé est introuvable.".to_string(),
            )
        } else {
            (
                400,
                "invalid_path",
                "Le chemin demandé est invalide.".to_string(),
            )
        }
    })?;
    let root = policy
        .allowed_roots()
        .iter()
        .filter(|root| canonical.starts_with(root))
        .max_by_key(|root| root.components().count())
        .ok_or_else(|| {
            (
                403,
                "outside_project_roots",
                "Cet aperçu n'est pas autorisé par la politique de projets.".to_string(),
            )
        })?;
    let metadata = std::fs::metadata(&canonical).map_err(|_| {
        (
            404,
            "not_found",
            "Le fichier demandé est introuvable.".to_string(),
        )
    })?;
    if !metadata.is_file() {
        return Err((
            415,
            "unsupported_media_type",
            "Seuls les fichiers réguliers peuvent être prévisualisés.".to_string(),
        ));
    }
    if metadata.len() > MAX_UI_FILE_PREVIEW_BYTES {
        return Err((
            413,
            "too_large",
            "Le fichier dépasse la taille maximale d'aperçu.".to_string(),
        ));
    }
    let (media_type, encoding) = content_preview_media_type(&canonical).ok_or_else(|| {
        (
            415,
            "unsupported_media_type",
            "Ce type de fichier ne peut pas être prévisualisé.".to_string(),
        )
    })?;
    let file = std::fs::File::open(&canonical).map_err(|_| {
        (
            404,
            "not_found",
            "Le fichier demandé est introuvable.".to_string(),
        )
    })?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.take(MAX_UI_FILE_PREVIEW_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| {
            (
                415,
                "unsupported_media_type",
                "Le fichier ne peut pas être lu.".to_string(),
            )
        })?;
    if bytes.len() as u64 > MAX_UI_FILE_PREVIEW_BYTES {
        return Err((
            413,
            "too_large",
            "Le fichier dépasse la taille maximale d'aperçu.".to_string(),
        ));
    }
    let content = if encoding == "utf8" {
        String::from_utf8(bytes.clone()).map_err(|_| {
            (
                415,
                "unsupported_media_type",
                "Le fichier texte n'est pas encodé en UTF-8.".to_string(),
            )
        })?
    } else {
        base64_standard(&bytes)
    };
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let sha256 = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let path = canonical
        .strip_prefix(root)
        .map_err(|_| {
            (
                403,
                "outside_project_roots",
                "Cet aperçu n'est pas autorisé par la politique de projets.".to_string(),
            )
        })?
        .to_string_lossy()
        .trim_start_matches('/')
        .to_string();
    Ok(UiFilePreviewV1 {
        version: UI_VERSION,
        path,
        media_type,
        encoding,
        content,
        truncated: false,
        sha256,
    })
}

fn read_server_control_settings(
    config: &UiRelayConfig,
) -> Result<UiServerControlSettingsV1, (u16, &'static str, String)> {
    let project = read_project_settings(config).ok();
    let configuration_available = project.is_some();
    let categories = crate::control_settings::server_setting_descriptors(configuration_available)
        .into_iter()
        .map(|descriptor| UiControlCategoryV1 {
            key: descriptor.key,
            scope: descriptor.scope,
            access: match descriptor.access {
                crate::control_settings::SettingAccess::Writable => "writable",
                crate::control_settings::SettingAccess::ReadOnly => "read_only",
            },
            summary: descriptor.summary,
        })
        .collect();
    Ok(UiServerControlSettingsV1 {
        version: UI_VERSION,
        configuration_available,
        policy_generation: project.as_ref().map(|settings| settings.policy_generation),
        allowed_project_roots: project
            .map(|settings| settings.allowed_project_roots)
            .unwrap_or_default(),
        categories,
        daemon_version: env!("CARGO_PKG_VERSION"),
        update_status: "not_configured",
    })
}

fn parse_control_project_roots_update(
    body: &[u8],
) -> Result<crate::control_settings::ProjectRootsChange, (u16, &'static str, String)> {
    let request: UiProjectRootsUpdateV1 = serde_json::from_slice(body).map_err(|_| {
        (
            400,
            "invalid_request",
            "Réglages de serveur invalides.".to_string(),
        )
    })?;
    if request.version != UI_VERSION {
        return Err((
            400,
            "invalid_request",
            "Réglages de serveur invalides.".to_string(),
        ));
    }
    Ok(crate::control_settings::ProjectRootsChange {
        command_id: request.command_id,
        expected_generation: request.expected_generation,
        allowed_project_roots: request
            .allowed_project_roots
            .into_iter()
            .map(PathBuf::from)
            .collect(),
    })
}

fn control_settings_error(
    error: crate::control_settings::ControlSettingsRefusal,
) -> (u16, &'static str, String) {
    match error {
        crate::control_settings::ControlSettingsRefusal::Unavailable => (
            409,
            "control_settings_unavailable",
            "Les réglages contrôlés ne sont pas disponibles sur ce serveur.".to_string(),
        ),
        crate::control_settings::ControlSettingsRefusal::InvalidRequest => (
            400,
            "invalid_request",
            "Réglages de serveur invalides.".to_string(),
        ),
        crate::control_settings::ControlSettingsRefusal::ConflictOrRefusal => (
            409,
            "control_settings_refused",
            "Le serveur a refusé les racines ou leur génération.".to_string(),
        ),
    }
}

fn post_control_project_roots_preview(
    config: &UiRelayConfig,
    body: &[u8],
) -> Result<UiControlProjectRootsPreviewV1, (u16, &'static str, String)> {
    let change = parse_control_project_roots_update(body)?;
    let command_id = change.command_id.clone();
    let expected_generation = change.expected_generation;
    let preview = crate::control_settings::preview_project_roots(
        config.project_root_policy_path.as_deref(),
        &change,
    )
    .map_err(control_settings_error)?;
    Ok(UiControlProjectRootsPreviewV1 {
        version: UI_VERSION,
        command_id,
        expected_generation,
        current_roots: preview
            .current_roots
            .iter()
            .map(|root| root.to_string_lossy().into_owned())
            .collect(),
        requested_roots: preview
            .requested_roots
            .iter()
            .map(|root| root.to_string_lossy().into_owned())
            .collect(),
        resulting_generation: expected_generation.saturating_add(1),
    })
}

fn post_control_project_roots_apply(
    config: &UiRelayConfig,
    body: &[u8],
) -> Result<UiControlProjectRootsAppliedV1, (u16, &'static str, String)> {
    let change = parse_control_project_roots_update(body)?;
    let applied = crate::control_settings::apply_project_roots(
        config.project_root_policy_path.as_deref(),
        change,
        now_secs(),
    )
    .map_err(control_settings_error)?;
    Ok(UiControlProjectRootsAppliedV1 {
        version: UI_VERSION,
        command_id: applied.receipt.command_id,
        expected_generation: applied.receipt.expected_generation,
        resulting_generation: applied.receipt.resulting_generation,
        allowed_project_roots: applied.receipt.allowed_project_roots,
        observed_at: applied.receipt.observed_at,
    })
}

fn post_project_roots_update(
    config: &UiRelayConfig,
    body: &[u8],
) -> Result<UiProjectRootsUpdateAcceptedV1, (u16, &'static str, String)> {
    let request: UiProjectRootsUpdateV1 = serde_json::from_slice(body).map_err(|_| {
        (
            400,
            "invalid_request",
            "Réglages projet invalides.".to_string(),
        )
    })?;
    let valid_command_id = !request.command_id.is_empty()
        && request.command_id.len() <= MAX_UI_COMMAND_ID_BYTES
        && request.command_id.bytes().all(is_query_byte);
    if request.version != UI_VERSION || !valid_command_id {
        return Err((
            400,
            "invalid_request",
            "Réglages projet invalides.".to_string(),
        ));
    }
    let Some(source) = config.project_root_policy_path.as_deref() else {
        return Err((
            409,
            "project_settings_unavailable",
            "Les racines projet ne sont pas configurées pour ce relais.".to_string(),
        ));
    };
    let roots = request
        .allowed_project_roots
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let policy = ProjectRootPolicy::replace_atomically(source, request.expected_generation, roots)
        .map_err(|_| {
            (
                409,
                "project_settings_refused",
                "Les racines ou leur génération ont été refusées.".to_string(),
            )
        })?;
    Ok(UiProjectRootsUpdateAcceptedV1 {
        version: UI_VERSION,
        command_id: request.command_id,
        policy_generation: policy.generation(),
        allowed_project_roots: policy
            .allowed_roots()
            .iter()
            .map(|root| root.to_string_lossy().into_owned())
            .collect(),
    })
}

fn post_project_preview(
    config: &UiRelayConfig,
    body: &[u8],
) -> Result<UiProjectPreviewV1, (u16, &'static str, String)> {
    use crate::project_workspace::{GitDiagnostic, ProjectFolderMode};

    let request: UiProjectPreviewRequestV1 = serde_json::from_slice(body).map_err(|_| {
        (
            400,
            "invalid_request",
            "Prévisualisation projet invalide.".to_string(),
        )
    })?;
    if request.version != UI_VERSION {
        return Err((
            400,
            "invalid_request",
            "Prévisualisation projet invalide.".to_string(),
        ));
    }
    let preview = resolve_project_preview(
        config,
        &request.mode,
        &request.root,
        request.folder_name.as_deref(),
    )?;
    let mode = match preview.mode {
        ProjectFolderMode::Create => "create",
        ProjectFolderMode::Import => "import",
    };
    let git = match preview.git {
        GitDiagnostic::Absent => "absent",
        GitDiagnostic::Clean => "clean",
        GitDiagnostic::Modified => "modified",
        GitDiagnostic::Worktree => "worktree",
    };
    Ok(UiProjectPreviewV1 {
        version: UI_VERSION,
        mode,
        canonical_path: preview.canonical_path.to_string_lossy().into_owned(),
        display_name: preview.display_name,
        git,
        git_initialization_proposed: preview.git_initialization_proposed,
    })
}

fn resolve_project_preview(
    config: &UiRelayConfig,
    mode: &str,
    root: &str,
    folder_name: Option<&str>,
) -> Result<crate::project_workspace::ProjectPreview, (u16, &'static str, String)> {
    use crate::project_workspace::ProjectPreview;

    if root.trim().is_empty() {
        return Err((
            400,
            "invalid_request",
            "La racine du projet est obligatoire.".to_string(),
        ));
    }
    let Some(source) = config.project_root_policy_path.as_deref() else {
        return Err((
            409,
            "project_settings_unavailable",
            "Les racines projet ne sont pas configurées pour ce relais.".to_string(),
        ));
    };
    let policy = ProjectRootPolicy::load(source).map_err(|_| {
        (
            409,
            "project_settings_unavailable",
            "La politique de racines est indisponible.".to_string(),
        )
    })?;
    match mode {
        "create" => {
            let Some(folder_name) = folder_name else {
                return Err((
                    400,
                    "invalid_request",
                    "Le nom du dossier projet est obligatoire.".to_string(),
                ));
            };
            ProjectPreview::create(&policy, Path::new(root), folder_name)
        }
        "import" => {
            if folder_name.is_some() {
                return Err((
                    400,
                    "invalid_request",
                    "Un import n accepte pas de nom de dossier.".to_string(),
                ));
            }
            ProjectPreview::import(&policy, Path::new(root))
        }
        _ => {
            return Err((400, "invalid_request", "Mode projet inconnu.".to_string()));
        }
    }
    .map_err(|error| (409, "project_preview_refused", error.to_string()))
}

type UiProjectError = (u16, &'static str, String);

fn project_daemon_unavailable() -> UiProjectError {
    (
        503,
        "daemon_unavailable",
        "Le daemon Bridget est indisponible.".to_string(),
    )
}

fn open_project_registry_service(
    socket_path: &Path,
) -> Result<(BufReader<UnixStream>, BufWriter<UnixStream>), UiProjectError> {
    let stream = UnixStream::connect(socket_path).map_err(|_| project_daemon_unavailable())?;
    let read_stream = stream
        .try_clone()
        .map_err(|_| project_daemon_unavailable())?;
    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(read_stream);
    send_daemon(
        &mut writer,
        &WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Service,
        },
    )
    .map_err(|_| project_daemon_unavailable())?;
    match read_daemon(&mut reader).map_err(|_| project_daemon_unavailable())? {
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Service,
        } => {}
        _ => {
            return Err((
                503,
                "project_service_unavailable",
                "Le registre de projets n’a pas accepté la session locale.".to_string(),
            ));
        }
    }
    send_daemon(
        &mut writer,
        &WrapperToDaemon::ServiceHello {
            version: SERVICE_CONTRACT_VERSION,
            service: "maicie".to_string(),
            issuer_scope: crate::mcp::issuer_scope("bridget-ui-project-registry"),
            capabilities: vec![ServiceCapability::ProjectRegistryV1],
        },
    )
    .map_err(|_| project_daemon_unavailable())?;
    match read_daemon(&mut reader).map_err(|_| project_daemon_unavailable())? {
        DaemonToWrapper::ServiceWelcome { capabilities, .. }
            if capabilities.contains(&ServiceCapability::ProjectRegistryV1) =>
        {
            Ok((reader, writer))
        }
        _ => Err((
            503,
            "project_service_unavailable",
            "Le registre de projets n’est pas disponible sur ce serveur.".to_string(),
        )),
    }
}

fn open_project_round_client(
    socket_path: &Path,
) -> Result<(BufReader<UnixStream>, BufWriter<UnixStream>), UiProjectError> {
    let stream =
        UnixStream::connect(socket_path).map_err(|_| project_round_service_unavailable())?;
    let read_stream = stream
        .try_clone()
        .map_err(|_| project_round_service_unavailable())?;
    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(read_stream);
    send_daemon(
        &mut writer,
        &WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Client,
        },
    )
    .map_err(|_| project_round_service_unavailable())?;
    if !matches!(
        read_daemon(&mut reader).map_err(|_| project_round_service_unavailable())?,
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        }
    ) {
        return Err(project_round_service_unavailable());
    }
    send_daemon(
        &mut writer,
        &WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: crate::mcp::issuer_scope("bridget-ui-project-round"),
            capabilities: vec![ClientCapability::ProjectRoundPolicyV1],
        },
    )
    .map_err(|_| project_round_service_unavailable())?;
    match read_daemon(&mut reader).map_err(|_| project_round_service_unavailable())? {
        DaemonToWrapper::ClientWelcome { capabilities, .. }
            if capabilities.contains(&ClientCapability::ProjectRoundPolicyV1) =>
        {
            Ok((reader, writer))
        }
        _ => Err(project_round_service_unavailable()),
    }
}

fn project_round_service_unavailable() -> UiProjectError {
    (
        503,
        "round_service_unavailable",
        "Le contrôle de ronde n’est pas disponible sur ce serveur.".to_string(),
    )
}

fn project_round_refusal(reason: ProjectRoundRefusal) -> UiProjectError {
    match reason {
        ProjectRoundRefusal::ProjectNotFound => (
            404,
            "project_not_found",
            "Ce projet n’existe plus dans Bridget.".to_string(),
        ),
        ProjectRoundRefusal::ProjectInactive => (
            409,
            "project_inactive",
            "La ronde ne peut être modifiée que pour un projet actif.".to_string(),
        ),
        ProjectRoundRefusal::BindingGenerationMismatch => (
            409,
            "binding_generation_mismatch",
            "Le projet a été reconnecté. Actualisez son état avant de modifier la ronde."
                .to_string(),
        ),
        ProjectRoundRefusal::EnvelopeMismatch => (
            409,
            "round_command_conflict",
            "Cette commande a déjà été utilisée avec une autre demande.".to_string(),
        ),
        ProjectRoundRefusal::PolicyDisabled => (
            409,
            "round_policy_disabled",
            "La politique de ronde n’est pas active pour cette génération.".to_string(),
        ),
        ProjectRoundRefusal::InvalidRequest | ProjectRoundRefusal::IdempotencyExpired => (
            400,
            "invalid_request",
            "La demande de ronde est invalide ou expirée.".to_string(),
        ),
        ProjectRoundRefusal::InvalidContract
        | ProjectRoundRefusal::PeerUidMismatch
        | ProjectRoundRefusal::StoreUnavailable => project_round_service_unavailable(),
    }
}

fn request_project_round(
    socket_path: &Path,
    command_id: String,
    operation: ProjectRoundOperation,
    project_id: Option<String>,
    binding_generation: Option<u64>,
) -> Result<ProjectRoundOutcome, UiProjectError> {
    let (mut reader, mut writer) = open_project_round_client(socket_path)?;
    let now = now_secs();
    let request = ProjectRoundRequest {
        contract_version: PROJECT_ROUND_POLICY_CONTRACT_VERSION,
        command_id,
        issued_at: now,
        deadline_at: now.saturating_add(10),
        operation,
        project_id,
        binding_generation,
    };
    send_daemon(
        &mut writer,
        &WrapperToDaemon::ProjectRoundRequest { request },
    )
    .map_err(|_| project_round_service_unavailable())?;
    let DaemonToWrapper::ProjectRoundOutcome { outcome } =
        read_daemon(&mut reader).map_err(|_| project_round_service_unavailable())?
    else {
        return Err(project_round_service_unavailable());
    };
    if let Some(reason) = outcome.reason {
        return Err(project_round_refusal(reason));
    }
    Ok(outcome)
}

fn read_projects(socket_path: &Path) -> Result<UiProjectListV1, UiProjectError> {
    let (mut reader, mut writer) = open_project_registry_service(socket_path)?;
    let now = now_secs();
    let request = ProjectAdminRequest {
        contract_version: PROJECT_REGISTRY_CONTRACT_VERSION,
        command_id: format!("ui-project-list-{}", uuid::Uuid::new_v4()),
        issued_at: now,
        deadline_at: now.saturating_add(10),
        operation: ProjectAdminOperation::List,
        project_id: None,
        requested_root: None,
    };
    send_daemon(
        &mut writer,
        &WrapperToDaemon::ProjectRegistryAdminRequest { request },
    )
    .map_err(|_| project_daemon_unavailable())?;
    let DaemonToWrapper::ProjectRegistryAdminOutcome { outcome } =
        read_daemon(&mut reader).map_err(|_| project_daemon_unavailable())?
    else {
        return Err((
            503,
            "project_service_unavailable",
            "Le registre de projets a renvoyé une réponse inattendue.".to_string(),
        ));
    };
    if outcome.reason.is_some() {
        return Err((
            409,
            "project_list_refused",
            "La liste des projets a été refusée par le serveur.".to_string(),
        ));
    }
    let rounds = request_project_round(
        socket_path,
        format!("ui-project-round-list-{}", uuid::Uuid::new_v4()),
        ProjectRoundOperation::List,
        None,
        None,
    )?;
    let mut rounds_by_project = rounds
        .policies
        .into_iter()
        .map(|policy| (policy.project_id.clone(), policy))
        .collect::<HashMap<_, _>>();
    // Complexité O(p) : chaque liaison et chaque politique est visitée une fois.
    let mut projects = Vec::new();
    for binding in outcome.bindings {
        let policy = rounds_by_project.remove(&binding.project_id);
        if let Some(project) = ui_project_list_entry(binding, policy)? {
            projects.push(project);
        }
    }
    projects.sort_by(|left, right| left.display_name.cmp(&right.display_name));
    Ok(UiProjectListV1 {
        version: UI_VERSION,
        projects,
    })
}

fn ui_project_list_entry(
    binding: ProjectBindingProjection,
    policy: Option<ProjectRoundProjection>,
) -> Result<Option<UiProjectListEntryV1>, UiProjectError> {
    let Some(canonical_path) = binding.canonical_root else {
        return Ok(None);
    };
    let binding_generation = binding
        .binding_generation
        .ok_or_else(project_round_service_unavailable)?;
    let policy = policy.ok_or_else(project_round_service_unavailable)?;
    if policy.project_id != binding.project_id
        || policy.binding_generation != Some(binding_generation)
        || policy.active != (binding.state == ProjectBindingStatus::Active)
    {
        return Err(project_round_service_unavailable());
    }
    let has_no_dispatch = policy.last_occurrence_at.is_none()
        && policy.last_dispatch_state.is_none()
        && policy.last_dispatch_observed_at.is_none();
    let has_complete_dispatch = policy.last_occurrence_at.is_some()
        && policy.last_dispatch_state.is_some()
        && policy.last_dispatch_observed_at.is_some();
    if !has_no_dispatch && !has_complete_dispatch {
        return Err(project_round_service_unavailable());
    }
    let display_name = Path::new(&canonical_path)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(&binding.project_id)
        .to_string();
    let state = match binding.state {
        ProjectBindingStatus::Active => "active",
        ProjectBindingStatus::Disabled => "disabled",
        ProjectBindingStatus::PathMissing => "path_missing",
        ProjectBindingStatus::PendingBinding => "pending_binding",
        ProjectBindingStatus::BindingFailed => "binding_failed",
        ProjectBindingStatus::Unregistered => "unregistered",
    };
    Ok(Some(UiProjectListEntryV1 {
        project_id: binding.project_id,
        display_name,
        canonical_path,
        state,
        binding_generation,
        round: ui_project_round(policy),
    }))
}

fn ui_project_round(policy: ProjectRoundProjection) -> UiProjectRoundV1 {
    UiProjectRoundV1 {
        configured: policy.configured,
        enabled: policy.enabled,
        revision: policy.revision,
        updated_at: policy.updated_at,
        interval_secs: PROJECT_ROUND_INTERVAL_SECS,
        last_occurrence_at: policy.last_occurrence_at,
        last_dispatch_state: policy.last_dispatch_state,
        last_dispatch_observed_at: policy.last_dispatch_observed_at,
    }
}

fn parse_ui_project_round_request(body: &[u8]) -> Result<UiProjectRoundRequestV1, UiProjectError> {
    let request: UiProjectRoundRequestV1 = serde_json::from_slice(body).map_err(|_| {
        (
            400,
            "invalid_request",
            "Action de ronde invalide.".to_string(),
        )
    })?;
    let valid_command_id = request.version == UI_VERSION
        && !request.command_id.trim().is_empty()
        && request.command_id.len() <= MAX_UI_COMMAND_ID_BYTES
        && request.command_id.bytes().all(is_query_byte);
    let valid_project_id = !request.project_id.trim().is_empty()
        && request.project_id.len() <= 128
        && request
            .project_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
    if !valid_command_id || !valid_project_id || request.binding_generation == 0 {
        return Err((
            400,
            "invalid_request",
            "Action de ronde invalide.".to_string(),
        ));
    }
    Ok(request)
}

fn post_project_round(
    config: &UiRelayConfig,
    body: &[u8],
) -> Result<UiProjectRoundAcceptedV1, UiProjectError> {
    let request = parse_ui_project_round_request(body)?;
    let operation = if request.enabled {
        ProjectRoundOperation::Enable
    } else {
        ProjectRoundOperation::Disable
    };
    let outcome = request_project_round(
        &config.daemon_socket,
        request.command_id.clone(),
        operation,
        Some(request.project_id.clone()),
        Some(request.binding_generation),
    )?;
    let Some(policy) = outcome.policies.into_iter().next() else {
        return Err(project_round_service_unavailable());
    };
    if policy.project_id != request.project_id
        || policy.binding_generation != Some(request.binding_generation)
    {
        return Err(project_round_service_unavailable());
    }
    Ok(UiProjectRoundAcceptedV1 {
        version: UI_VERSION,
        command_id: request.command_id,
        project_id: request.project_id,
        binding_generation: request.binding_generation,
        round: ui_project_round(policy),
    })
}

fn post_project_confirm(
    config: &UiRelayConfig,
    body: &[u8],
) -> Result<UiProjectConfirmedV1, UiProjectError> {
    use crate::project_workspace::ProjectFolderMode;
    use std::process::Command;

    let request: UiProjectConfirmRequestV1 = serde_json::from_slice(body).map_err(|_| {
        (
            400,
            "invalid_request",
            "Confirmation de projet invalide.".to_string(),
        )
    })?;
    if request.version != UI_VERSION
        || request.command_id.trim().is_empty()
        || request.command_id.len() > MAX_UI_COMMAND_ID_BYTES
        || !request.command_id.bytes().all(is_query_byte)
    {
        return Err((
            400,
            "invalid_request",
            "Confirmation de projet invalide.".to_string(),
        ));
    }
    let preview = resolve_project_preview(
        config,
        &request.mode,
        &request.root,
        request.folder_name.as_deref(),
    )?;
    let created_directory = matches!(preview.mode, ProjectFolderMode::Create);
    if created_directory {
        std::fs::create_dir(&preview.canonical_path).map_err(|error| {
            (
                409,
                "project_create_refused",
                format!("Le dossier du projet n’a pas pu être créé : {error}"),
            )
        })?;
    }
    if request.initialize_git && preview.git_initialization_proposed {
        let initialized = Command::new("git")
            .arg("init")
            .arg("--quiet")
            .arg(&preview.canonical_path)
            .status()
            .map_err(|error| {
                (
                    503,
                    "git_unavailable",
                    format!("Git n’a pas pu initialiser le projet : {error}"),
                )
            })?;
        if !initialized.success() {
            return Err((
                409,
                "git_initialization_refused",
                "Git a refusé l’initialisation du projet.".to_string(),
            ));
        }
    }
    let (mut reader, mut writer) = open_project_registry_service(&config.daemon_socket)?;
    let now = now_secs();
    let project_id = format!("project-{}", uuid::Uuid::new_v4().simple());
    let bind = ProjectBindRequest {
        contract_version: PROJECT_REGISTRY_CONTRACT_VERSION,
        command_id: request.command_id,
        issued_at: now,
        deadline_at: now.saturating_add(30),
        project_id: project_id.clone(),
        requested_root: preview.canonical_path.to_string_lossy().into_owned(),
        backend: ProjectBackend::Host,
        policy_id: None,
        policy_version: None,
    };
    send_daemon(
        &mut writer,
        &WrapperToDaemon::ProjectRegistryRequest { request: bind },
    )
    .map_err(|_| project_daemon_unavailable())?;
    let DaemonToWrapper::ProjectRegistryOutcome { outcome } =
        read_daemon(&mut reader).map_err(|_| project_daemon_unavailable())?
    else {
        return Err((
            503,
            "project_service_unavailable",
            "Le registre de projets a renvoyé une réponse inattendue.".to_string(),
        ));
    };
    if outcome.status != ProjectBindStatus::Active || outcome.reason.is_some() {
        let context = if created_directory {
            " Le dossier créé est conservé, sans suppression automatique."
        } else {
            " Aucun contenu du dossier importé n’a été modifié par Bridget."
        };
        return Err((
            409,
            "project_registration_refused",
            format!("Le registre a refusé le projet.{context}"),
        ));
    }
    Ok(UiProjectConfirmedV1 {
        version: UI_VERSION,
        project_id,
        display_name: preview.display_name,
        canonical_path: preview.canonical_path.to_string_lossy().into_owned(),
    })
}

fn post_project_admin(
    config: &UiRelayConfig,
    body: &[u8],
    operation: ProjectAdminOperation,
) -> Result<UiProjectAdminAcceptedV1, UiProjectError> {
    let request: UiProjectAdminRequestV1 = serde_json::from_slice(body).map_err(|_| {
        (
            400,
            "invalid_request",
            "Action de projet invalide.".to_string(),
        )
    })?;
    let valid_command_id = request.version == UI_VERSION
        && !request.command_id.trim().is_empty()
        && request.command_id.len() <= MAX_UI_COMMAND_ID_BYTES
        && request.command_id.bytes().all(is_query_byte);
    let valid_project_id = !request.project_id.trim().is_empty()
        && request.project_id.len() <= 128
        && request
            .project_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
    let needs_root = matches!(
        operation,
        ProjectAdminOperation::Activate | ProjectAdminOperation::Rebind
    );
    if !valid_command_id
        || !valid_project_id
        || (needs_root
            && request
                .root
                .as_deref()
                .is_none_or(|root| root.trim().is_empty()))
        || (!needs_root && request.root.is_some())
    {
        return Err((
            400,
            "invalid_request",
            "Action de projet invalide.".to_string(),
        ));
    }
    let (mut reader, mut writer) = open_project_registry_service(&config.daemon_socket)?;
    let now = now_secs();
    let request = ProjectAdminRequest {
        contract_version: PROJECT_REGISTRY_CONTRACT_VERSION,
        command_id: request.command_id,
        issued_at: now,
        deadline_at: now.saturating_add(30),
        operation,
        project_id: Some(request.project_id),
        requested_root: request.root,
    };
    send_daemon(
        &mut writer,
        &WrapperToDaemon::ProjectRegistryAdminRequest { request },
    )
    .map_err(|_| project_daemon_unavailable())?;
    let DaemonToWrapper::ProjectRegistryAdminOutcome { outcome } =
        read_daemon(&mut reader).map_err(|_| project_daemon_unavailable())?
    else {
        return Err((
            503,
            "project_service_unavailable",
            "Le registre de projets a renvoyé une réponse inattendue.".to_string(),
        ));
    };
    if outcome.reason.is_some() {
        return Err((
            409,
            "project_action_refused",
            "Le registre a refusé cette action sur le projet.".to_string(),
        ));
    }
    let Some(binding) = outcome.bindings.into_iter().next() else {
        return Err((
            409,
            "project_action_refused",
            "Le registre n’a pas confirmé l’état du projet.".to_string(),
        ));
    };
    let round = request_project_round(
        &config.daemon_socket,
        format!("ui-project-round-status-{}", uuid::Uuid::new_v4()),
        ProjectRoundOperation::Status,
        Some(binding.project_id.clone()),
        None,
    )?;
    let Some(project) = ui_project_list_entry(binding, round.policies.into_iter().next())? else {
        return Err((
            409,
            "project_action_refused",
            "Le registre n’a pas confirmé l’état du projet.".to_string(),
        ));
    };
    Ok(UiProjectAdminAcceptedV1 {
        version: UI_VERSION,
        project,
    })
}

fn read_usage_dashboard(
    config: &UiRelayConfig,
    requested_period: Option<&str>,
) -> Result<UiUsageDashboardV1, (u16, &'static str, String)> {
    let (period, duration_secs) = match requested_period.unwrap_or("7d") {
        "7d" => ("7d", 7_i64 * 24 * 60 * 60),
        "30d" => ("30d", 30_i64 * 24 * 60 * 60),
        "90d" => ("90d", 90_i64 * 24 * 60 * 60),
        _ => {
            return Err((
                400,
                "invalid_period",
                "La période d’usage doit être 7d, 30d ou 90d.".to_string(),
            ));
        }
    };
    let to_secs = now_secs();
    if to_secs <= 0 {
        return Err((
            503,
            "usage_unavailable",
            "L’horloge du serveur est indisponible.".to_string(),
        ));
    }
    let from_secs = to_secs.saturating_sub(duration_secs);
    let store = crate::store::Store::open(&ledger_db_path_for_socket(&config.daemon_socket))
        .map_err(|_| {
            (
                503,
                "usage_unavailable",
                "Les données d’usage du serveur sont indisponibles.".to_string(),
            )
        })?;
    let rows = store
        .usage_dashboard_window(from_secs, to_secs)
        .map_err(|_| {
            (
                503,
                "usage_unavailable",
                "Les données d’usage du serveur sont indisponibles.".to_string(),
            )
        })?
        .into_iter()
        .map(|row| UiUsageDashboardRowV1 {
            total_tokens: row.total_tokens(),
            provider_kind: row.provider_kind,
            model: row.model,
            source: row.source,
            samples: row.samples,
            input_tokens: row.input_tokens,
            output_tokens: row.output_tokens,
            cache_creation_input_tokens: row.cache_creation_input_tokens,
            cache_read_input_tokens: row.cache_read_input_tokens,
            cost_estimate_microunits: None,
        })
        .collect();
    Ok(UiUsageDashboardV1 {
        version: UI_VERSION,
        period,
        from_secs,
        to_secs,
        rows,
        pricing_status: "unconfigured",
    })
}

fn read_project_runtime(
    socket_path: &Path,
    project_id: &str,
) -> Result<UiProjectRuntimeV1, (u16, &'static str, String)> {
    let valid_project_id = !project_id.is_empty()
        && project_id.len() <= 128
        && project_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
    if !valid_project_id {
        return Err((
            400,
            "invalid_project",
            "Identifiant de projet invalide.".to_string(),
        ));
    }
    let issued_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    let request = ProjectRuntimeRequest {
        contract_version: crate::project_runtime::PROJECT_RUNTIME_CONTRACT_VERSION,
        command_id: format!("ui-runtime-{}", uuid::Uuid::new_v4()),
        issued_at,
        deadline_at: issued_at.saturating_add(10),
        operation: ProjectRuntimeOperation::Status,
        project_id: project_id.to_string(),
        profile: None,
    };
    let stream = UnixStream::connect(socket_path).map_err(|_| {
        (
            503,
            "daemon_unavailable",
            "Le daemon Bridget est indisponible.".to_string(),
        )
    })?;
    let read_stream = stream.try_clone().map_err(|_| {
        (
            503,
            "daemon_unavailable",
            "Le daemon Bridget est indisponible.".to_string(),
        )
    })?;
    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(read_stream);
    send_daemon(
        &mut writer,
        &WrapperToDaemon::ProjectRuntimeRequest { request },
    )
    .map_err(|_| {
        (
            503,
            "daemon_unavailable",
            "Le daemon Bridget est indisponible.".to_string(),
        )
    })?;
    match read_daemon(&mut reader).map_err(|_| {
        (
            503,
            "daemon_unavailable",
            "Le daemon Bridget est indisponible.".to_string(),
        )
    })? {
        DaemonToWrapper::ProjectRuntimeOutcome { outcome } => {
            if let Some(reason) = outcome.reason {
                let (status, code, message) = match reason {
                    ProjectRuntimeRefusal::ProjectNotFound => {
                        (404, "project_not_found", "Projet introuvable.".to_string())
                    }
                    ProjectRuntimeRefusal::ProjectNotDocker => (
                        409,
                        "project_not_docker",
                        "Ce projet utilise le backend hôte.".to_string(),
                    ),
                    ProjectRuntimeRefusal::PeerUidMismatch => (
                        403,
                        "runtime_not_authorized",
                        "Cette interface ne peut pas administrer cet environnement.".to_string(),
                    ),
                    _ => (
                        503,
                        "runtime_unavailable",
                        "État Docker temporairement indisponible.".to_string(),
                    ),
                };
                return Err((status, code, message));
            }
            let policy = outcome.runtime_policy.ok_or_else(|| {
                (
                    503,
                    "runtime_unavailable",
                    "Projection Docker incomplète.".to_string(),
                )
            })?;
            Ok(UiProjectRuntimeV1 {
                version: UI_VERSION,
                project_id: outcome.project_id,
                binding_generation: outcome.binding_generation.ok_or_else(|| {
                    (
                        503,
                        "runtime_unavailable",
                        "Projection Docker incomplète.".to_string(),
                    )
                })?,
                state: outcome.state.ok_or_else(|| {
                    (
                        503,
                        "runtime_unavailable",
                        "Projection Docker incomplète.".to_string(),
                    )
                })?,
                policy_id: policy.policy_id,
                policy_version: policy.policy_version,
                environment_epoch: policy.environment_epoch,
                last_reason: outcome.last_reason,
            })
        }
        response => Err((
            503,
            "runtime_unavailable",
            format!("Réponse runtime inattendue: {response:?}"),
        )),
    }
}

fn send_ui_stop(
    socket_path: &Path,
    request: &UiStopRequestV1,
) -> Result<bridget_transport::protocol::StopOutcome, UiError> {
    let stream = UnixStream::connect(socket_path)?;
    let read_stream = stream.try_clone()?;
    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(read_stream);
    send_daemon(
        &mut writer,
        &WrapperToDaemon::StopOrder {
            agent_id: request.agent_id.clone(),
            command_id: request.command_id.clone(),
        },
    )?;
    match read_daemon(&mut reader)? {
        DaemonToWrapper::StopResult {
            command_id,
            outcome,
        } if command_id == request.command_id => Ok(outcome),
        response => Err(UiError::Protocol(format!(
            "StopResult corrélé attendu, reçu {response:?}"
        ))),
    }
}

fn post_ui_stop(config: &UiRelayConfig, body: &[u8]) -> Result<UiStopAcceptedV1, UiStopError> {
    let request = parse_ui_stop_request(body)?;
    let agents = read_agent_list(&config.daemon_socket).map_err(|_| {
        (
            503,
            "daemon_unavailable",
            "Le daemon Bridget est indisponible.".to_string(),
        )
    })?;
    validate_ui_stop_target(&agents, &request.agent_id).map_err(|(status, code)| {
        let message = match code {
            "agent_not_managed" => "Cet agent n'est pas géré par Bridget.",
            "agent_stopped" => "Cet agent est déjà arrêté.",
            "lifecycle_in_progress" => "Une opération de cycle de vie est déjà en cours.",
            "agent_unavailable" => "L’état de cet agent ne permet pas une action de cycle de vie.",
            _ => "Cet agent est introuvable.",
        };
        (status, code, message.to_string())
    })?;
    let outcome = send_ui_stop(&config.daemon_socket, &request).map_err(|_| {
        (
            503,
            "daemon_unavailable",
            "Le daemon Bridget est indisponible.".to_string(),
        )
    })?;
    map_ui_stop_outcome(&request, outcome)
}

fn map_ui_relaunch_outcome(
    request: &UiStopRequestV1,
    outcome: bridget_transport::protocol::RelaunchOutcome,
) -> Result<UiStopAcceptedV1, UiStopError> {
    use bridget_transport::protocol::RelaunchOutcome;
    match outcome {
        RelaunchOutcome::Started { generation, .. } => Ok(UiStopAcceptedV1 {
            version: UI_VERSION,
            agent_id: request.agent_id.clone(),
            command_id: request.command_id.clone(),
            outcome: "started",
            survivors_killed: None,
            generation: Some(generation),
        }),
        RelaunchOutcome::AlreadyRunning => Err((
            409,
            "agent_already_running",
            "Cet agent est déjà actif.".to_string(),
        )),
        RelaunchOutcome::NotManaged => Err((
            409,
            "agent_not_managed",
            "Cet agent n'est pas géré par Bridget.".to_string(),
        )),
        RelaunchOutcome::NotFound => Err((
            404,
            "agent_not_found",
            "Cet agent est introuvable.".to_string(),
        )),
        RelaunchOutcome::NotRelaunchable { reason } => Err((
            409,
            "agent_not_relaunchable",
            format!("Cet agent ne peut pas être relancé : {reason}"),
        )),
        RelaunchOutcome::Rejected { reason } => Err((
            409,
            "relaunch_rejected",
            format!("La relance a été refusée : {reason:?}"),
        )),
        RelaunchOutcome::Timeout { .. } => Err((
            504,
            "lifecycle_timeout",
            "Le daemon n'a pas confirmé la relance dans le délai.".to_string(),
        )),
    }
}

fn send_ui_relaunch(
    socket_path: &Path,
    request: &UiStopRequestV1,
) -> Result<bridget_transport::protocol::RelaunchOutcome, UiError> {
    let stream = UnixStream::connect(socket_path)?;
    let read_stream = stream.try_clone()?;
    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(read_stream);
    send_daemon(
        &mut writer,
        &WrapperToDaemon::RelaunchOrder {
            agent_id: request.agent_id.clone(),
            command_id: request.command_id.clone(),
        },
    )?;
    match read_daemon(&mut reader)? {
        DaemonToWrapper::RelaunchResult {
            command_id,
            outcome,
        } if command_id == request.command_id => Ok(outcome),
        response => Err(UiError::Protocol(format!(
            "RelaunchResult corrélé attendu, reçu {response:?}"
        ))),
    }
}

fn post_ui_relaunch(config: &UiRelayConfig, body: &[u8]) -> Result<UiStopAcceptedV1, UiStopError> {
    let request = parse_ui_lifecycle_request(body, "relance")?;
    let agents = read_agent_list(&config.daemon_socket).map_err(|_| {
        (
            503,
            "daemon_unavailable",
            "Le daemon Bridget est indisponible.".to_string(),
        )
    })?;
    validate_ui_relaunch_target(&agents, &request.agent_id).map_err(|(status, code)| {
        let message = match code {
            "agent_not_managed" => "Cet agent n'est pas géré par Bridget.",
            "agent_already_running" => "Cet agent est déjà actif.",
            "lifecycle_in_progress" => "Une opération de cycle de vie est déjà en cours.",
            "agent_unavailable" => "L’état de cet agent ne permet pas une action de cycle de vie.",
            _ => "Cet agent est introuvable.",
        };
        (status, code, message.to_string())
    })?;
    let outcome = send_ui_relaunch(&config.daemon_socket, &request).map_err(|_| {
        (
            503,
            "daemon_unavailable",
            "Le daemon Bridget est indisponible.".to_string(),
        )
    })?;
    map_ui_relaunch_outcome(&request, outcome)
}

fn map_ui_decommission_outcome(
    request: &UiStopRequestV1,
    outcome: bridget_transport::protocol::DecommissionOutcome,
) -> Result<UiStopAcceptedV1, UiStopError> {
    use bridget_transport::protocol::DecommissionOutcome;
    match outcome {
        DecommissionOutcome::Decommissioned => Ok(UiStopAcceptedV1 {
            version: UI_VERSION,
            agent_id: request.agent_id.clone(),
            command_id: request.command_id.clone(),
            outcome: "decommissioned",
            survivors_killed: None,
            generation: None,
        }),
        DecommissionOutcome::DecommissionedForced { survivors_killed } => Ok(UiStopAcceptedV1 {
            version: UI_VERSION,
            agent_id: request.agent_id.clone(),
            command_id: request.command_id.clone(),
            outcome: "decommissioned_forced",
            survivors_killed: Some(survivors_killed),
            generation: None,
        }),
        DecommissionOutcome::AlreadyDecommissioned => Err((
            409,
            "agent_already_decommissioned",
            "Cet agent est déjà décommissionné.".to_string(),
        )),
        DecommissionOutcome::NotManaged => Err((
            409,
            "agent_not_managed",
            "Cet agent n'est pas géré par Bridget.".to_string(),
        )),
        DecommissionOutcome::NotFound => Err((
            404,
            "agent_not_found",
            "Cet agent est introuvable.".to_string(),
        )),
        DecommissionOutcome::Timeout { .. } => Err((
            504,
            "lifecycle_timeout",
            "Le daemon n'a pas confirmé le décommissionnement dans le délai.".to_string(),
        )),
    }
}

fn send_ui_decommission(
    socket_path: &Path,
    request: &UiStopRequestV1,
) -> Result<bridget_transport::protocol::DecommissionOutcome, UiError> {
    let stream = UnixStream::connect(socket_path)?;
    let read_stream = stream.try_clone()?;
    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(read_stream);
    send_daemon(
        &mut writer,
        &WrapperToDaemon::DecommissionOrder {
            agent_id: request.agent_id.clone(),
            command_id: request.command_id.clone(),
        },
    )?;
    match read_daemon(&mut reader)? {
        DaemonToWrapper::DecommissionResult {
            command_id,
            outcome,
        } if command_id == request.command_id => Ok(outcome),
        response => Err(UiError::Protocol(format!(
            "DecommissionResult corrélé attendu, reçu {response:?}"
        ))),
    }
}

fn post_ui_decommission(
    config: &UiRelayConfig,
    body: &[u8],
) -> Result<UiStopAcceptedV1, UiStopError> {
    let request = parse_ui_lifecycle_request(body, "décommissionnement")?;
    let agents = read_agent_list(&config.daemon_socket).map_err(|_| {
        (
            503,
            "daemon_unavailable",
            "Le daemon Bridget est indisponible.".to_string(),
        )
    })?;
    validate_ui_decommission_target(&agents, &request.agent_id).map_err(|(status, code)| {
        let message = match code {
            "agent_not_managed" => "Cet agent n'est pas géré par Bridget.",
            _ => "Cet agent est introuvable.",
        };
        (status, code, message.to_string())
    })?;
    let outcome = send_ui_decommission(&config.daemon_socket, &request).map_err(|_| {
        (
            503,
            "daemon_unavailable",
            "Le daemon Bridget est indisponible.".to_string(),
        )
    })?;
    map_ui_decommission_outcome(&request, outcome)
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
    let outcome = search_ui_ledger(config, &request.q).map_err(|_| (503, "ledger indisponible"))?;
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
fn search_ui_ledger(config: &UiRelayConfig, query: &str) -> Result<UiSearchOutcome, UiError> {
    let db_path = ledger_db_path_for_socket(&config.daemon_socket);
    let store = crate::store::Store::open(&db_path)
        .map_err(|error| UiError::Configuration(error.to_string()))?;
    let outcome = store
        .search_messages(query, crate::store::MAX_LEDGER_SEARCH_PUBLIC)
        .map_err(|error| UiError::Configuration(error.to_string()))?;
    let routing_names = outcome
        .hits
        .iter()
        .flat_map(|entry| [entry.sender.clone(), entry.target.clone()])
        .collect::<Vec<_>>();
    let profiles = profile_summaries_for_agent_ids(config, &routing_names);

    Ok(UiSearchOutcome {
        hits: outcome
            .hits
            .into_iter()
            .map(|entry| {
                let sender = public_search_party(&entry.sender, &profiles);
                let target = public_search_party(&entry.target, &profiles);
                UiSearchHitV1 {
                    id: entry.id,
                    ts: entry.ts,
                    sender,
                    target,
                    body: entry.body,
                }
            })
            .collect(),
        truncated: outcome.truncated,
    })
}

fn public_search_party(
    routing_name: &str,
    profiles: &HashMap<String, AgentProfileSummary>,
) -> String {
    if routing_name == UI_SENDER {
        return "Vous".to_string();
    }
    profiles
        .get(routing_name)
        .map(|profile| profile.display_name.clone())
        .unwrap_or_else(|| "Agent non identifié".to_string())
}

fn ledger_db_path_for_socket(socket_path: &Path) -> PathBuf {
    socket_path.with_extension("db")
}

/// Déduit la seule portée d'artefact lisible dans le contexte d'un agent
/// attesté. Un projet actif l'emporte. Sinon la conversation reçoit une portée
/// privée et stable dérivée de l'agent connu du daemon. Aucune route
/// d'artefact n'accepte de portée fournie par le navigateur.
fn artifact_scope_for_agent(
    agents: &[bridget_transport::protocol::AgentInfo],
    agent: &str,
) -> Option<String> {
    agents
        .iter()
        .find(|candidate| candidate.agent_id == agent)
        .map(|candidate| {
            candidate
                .agent_link
                .as_ref()
                .and_then(|link| link.project.as_ref())
                .map(|project| project.project_id.clone())
                .unwrap_or_else(|| format!("conversation-agent:{agent}"))
        })
}

fn read_artifact_references(
    config: &UiRelayConfig,
    agents: &[bridget_transport::protocol::AgentInfo],
    agent: &str,
) -> UiArtifactReferenceProjectionV1 {
    let Some(project_id) = artifact_scope_for_agent(agents, agent) else {
        return UiArtifactReferenceProjectionV1 {
            state: "not_scoped",
            items: Vec::new(),
        };
    };
    let conversation_reference = format!("conversation:{project_id}:{agent}");
    let Ok(store) = crate::artifact_store::ArtifactStore::open(&ledger_db_path_for_socket(
        &config.daemon_socket,
    )) else {
        return UiArtifactReferenceProjectionV1 {
            state: "unavailable",
            items: Vec::new(),
        };
    };
    let Ok(references) =
        store.list_conversation_references(&project_id, &conversation_reference, 200)
    else {
        return UiArtifactReferenceProjectionV1 {
            state: "unavailable",
            items: Vec::new(),
        };
    };
    UiArtifactReferenceProjectionV1 {
        state: "ready",
        items: references
            .into_iter()
            .map(|reference| UiArtifactReferenceV1 {
                reference_id: reference.reference_id,
                artifact_ref: reference.artifact_ref,
                version_ref: reference.version_ref,
                turn_reference: reference.turn_reference,
                kind: reference.kind,
                title: reference.title,
                state: reference.state,
                pinned: reference.pinned,
                created_at: reference.version_created_at,
            })
            .collect(),
    }
}

type UiArtifactError = (u16, &'static str, String);

fn artifact_version_from_suffix(suffix: &str) -> Result<String, UiArtifactError> {
    if suffix.is_empty()
        || suffix.len() > 128
        || !suffix
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err((
            400,
            "artifact_version_invalid",
            "Version d’artefact invalide.".to_string(),
        ));
    }
    Ok(format!("artifact-version:{suffix}"))
}

fn artifact_version_from_query(value: Option<&str>) -> Result<String, UiArtifactError> {
    let Some(value) = value else {
        return Err((
            400,
            "artifact_version_missing",
            "Version d’artefact absente.".to_string(),
        ));
    };
    // La grammaire volontairement restrictive de l'URL relayée n'accepte pas
    // `:`. Les références canoniques utilisent ce caractère, donc la vue les
    // transporte sous la forme réversible `artifact-version.<uuid>` et ne
    // laisse jamais passer une chaîne arbitraire vers SQLite.
    let Some(suffix) = value.strip_prefix("artifact-version.") else {
        return Err((
            400,
            "artifact_version_invalid",
            "Version d’artefact invalide.".to_string(),
        ));
    };
    artifact_version_from_suffix(suffix)
}

/// Les corps JSON portent la référence canonique conservée dans le registre,
/// distincte de la forme URL sans `:` exposée au navigateur.
fn artifact_version_from_request(value: &str) -> Result<String, UiArtifactError> {
    let Some(suffix) = value.strip_prefix("artifact-version:") else {
        return Err((
            400,
            "artifact_version_invalid",
            "Version d’artefact invalide.".to_string(),
        ));
    };
    artifact_version_from_suffix(suffix)
}

fn artifact_ui_scope(
    config: &UiRelayConfig,
    agent: Option<&String>,
) -> Result<(String, String), UiArtifactError> {
    let Some(agent) = agent else {
        return Err((
            400,
            "artifact_agent_missing",
            "Agent requis pour consulter un artefact.".to_string(),
        ));
    };
    validate_agent(agent).map_err(|_| {
        (
            400,
            "artifact_agent_invalid",
            "Agent d’artefact invalide.".to_string(),
        )
    })?;
    let facts = read_bridget_snapshot(&config.daemon_socket).map_err(|_| {
        (
            503,
            "artifact_scope_unavailable",
            "Le contexte projet Bridget est temporairement indisponible.".to_string(),
        )
    })?;
    let Some(project_id) = artifact_scope_for_agent(&facts.agents, agent) else {
        return Err((
            404,
            "artifact_agent_unavailable",
            "Cet agent n’est pas visible dans Bridget.".to_string(),
        ));
    };
    Ok((project_id, agent.clone()))
}

fn open_artifact_store_for_ui(
    config: &UiRelayConfig,
) -> Result<crate::artifact_store::ArtifactStore, UiArtifactError> {
    crate::artifact_store::ArtifactStore::open(&ledger_db_path_for_socket(&config.daemon_socket))
        .map_err(|_| {
            (
                503,
                "artifact_store_unavailable",
                "Le magasin d’artefacts est temporairement indisponible.".to_string(),
            )
        })
}

fn read_artifact_detail(
    config: &UiRelayConfig,
    agent: Option<&String>,
    version: Option<&String>,
) -> Result<UiArtifactDetailV1, UiArtifactError> {
    let (project_id, _) = artifact_ui_scope(config, agent)?;
    let version_ref = artifact_version_from_query(version.map(String::as_str))?;
    read_artifact_detail_for_version(config, &project_id, &version_ref)
}

fn read_artifact_detail_for_version(
    config: &UiRelayConfig,
    project_id: &str,
    version_ref: &str,
) -> Result<UiArtifactDetailV1, UiArtifactError> {
    let store = open_artifact_store_for_ui(config)?;
    let detail = store
        .version_detail(project_id, version_ref)
        .map_err(|_| {
            (
                503,
                "artifact_detail_unavailable",
                "Le détail de l’artefact est momentanément indisponible.".to_string(),
            )
        })?
        .ok_or_else(|| {
            (
                404,
                "artifact_not_found",
                "Cette version d’artefact est introuvable dans le projet de l’agent.".to_string(),
            )
        })?;
    Ok(UiArtifactDetailV1 {
        version: UI_VERSION,
        item: detail.item,
        is_current: detail.is_current,
        publication: detail.publication,
        content_digest: detail.content_digest,
        payload_digest: detail.payload_digest,
        provenance_digest: detail.provenance_digest,
        parent_version_ref: detail.parent_version_ref,
        quality_notices: detail.quality_notices,
        publication_reason: detail.publication_reason,
    })
}

fn artifact_list_limit(query: &HashMap<String, String>) -> Result<usize, UiArtifactError> {
    query
        .get("limit")
        .map(|value| value.parse::<usize>())
        .transpose()
        .map_err(|_| {
            (
                400,
                "artifact_list_invalid",
                "Limite de liste d’artefacts invalide.".to_string(),
            )
        })
        .map(|value| value.unwrap_or(50).clamp(1, 100))
}

fn read_artifact_list(
    config: &UiRelayConfig,
    query: &HashMap<String, String>,
    is_search: bool,
) -> Result<UiArtifactListV1, UiArtifactError> {
    let agent = query.get("agent");
    let (project_id, _) = artifact_ui_scope(config, agent)?;
    let limit = artifact_list_limit(query)?;
    let requested_scope = query.get("scope").map(String::as_str).unwrap_or("project");
    let projects = match requested_scope {
        "project" => vec![project_id],
        "all" => {
            let facts = read_bridget_snapshot(&config.daemon_socket).map_err(|_| {
                (
                    503,
                    "artifact_scope_unavailable",
                    "La portée globale Bridget est momentanément indisponible.".to_string(),
                )
            })?;
            let mut scoped = facts
                .agents
                .iter()
                .filter_map(|candidate| {
                    candidate
                        .agent_link
                        .as_ref()
                        .and_then(|link| link.project.as_ref())
                        .map(|project| project.project_id.clone())
                })
                .collect::<Vec<_>>();
            scoped.sort();
            scoped.dedup();
            if scoped.is_empty() {
                return Err((
                    403,
                    "artifact_global_scope_empty",
                    "Aucun projet actif n’est disponible pour la recherche globale.".to_string(),
                ));
            }
            scoped
        }
        _ => {
            return Err((
                400,
                "artifact_scope_invalid",
                "Portée d’artefacts invalide.".to_string(),
            ));
        }
    };
    let search_query = query.get("query").map(String::as_str).unwrap_or_default();
    let store = open_artifact_store_for_ui(config)?;
    let mut items = Vec::new();
    for scoped_project in &projects {
        let mut page = if is_search {
            store.search_project(scoped_project, search_query, limit, None)
        } else {
            store.list_project(scoped_project, limit, None)
        }
        .map_err(|_| {
            (
                503,
                "artifact_list_unavailable",
                "La liste des artefacts est momentanément indisponible.".to_string(),
            )
        })?;
        items.append(&mut page);
    }
    items.sort_by(|left, right| {
        right
            .version_created_at
            .cmp(&left.version_created_at)
            .then_with(|| left.version_ref.cmp(&right.version_ref))
    });
    items.truncate(limit);
    Ok(UiArtifactListV1 {
        version: UI_VERSION,
        scope: if requested_scope == "all" {
            "all"
        } else {
            "project"
        },
        items,
    })
}

fn post_artifact_share(
    config: &UiRelayConfig,
    body: &[u8],
) -> Result<UiArtifactShareAcceptedV1, UiArtifactError> {
    let request: UiArtifactShareRequestV1 = serde_json::from_slice(body).map_err(|_| {
        (
            400,
            "artifact_share_invalid",
            "Demande de partage d’artefact invalide.".to_string(),
        )
    })?;
    if request.version != UI_VERSION {
        return Err((
            400,
            "artifact_share_invalid",
            "Version de demande de partage invalide.".to_string(),
        ));
    }
    validate_agent(&request.target_agent).map_err(|_| {
        (
            400,
            "artifact_share_target_invalid",
            "Destinataire de partage invalide.".to_string(),
        )
    })?;
    let (project_id, _) = artifact_ui_scope(config, Some(&request.agent))?;
    let version_ref = artifact_version_from_request(&request.version_ref)?;
    let reference_id = open_artifact_store_for_ui(config)?
        .share_with_agent(
            &project_id,
            &version_ref,
            &request.target_agent,
            "operator:ui",
            now_secs(),
        )
        .map_err(|_| {
            (
                404,
                "artifact_share_not_found",
                "Cette version ne peut pas être partagée depuis le projet actif.".to_string(),
            )
        })?;
    Ok(UiArtifactShareAcceptedV1 {
        version: UI_VERSION,
        reference_id,
    })
}

fn open_artifact_service_for_ui(
    config: &UiRelayConfig,
) -> Result<crate::artifact_service::ArtifactService, UiArtifactError> {
    // Le daemon dérive la racine canonique de sa base (`*.db` ->
    // `*.artifacts`). Le relais doit reprendre exactement cette dérivation
    // depuis sa socket, jamais celle d'un autre profil Desktop par défaut.
    let database_path = ledger_db_path_for_socket(&config.daemon_socket);
    let artifact_root = database_path.with_extension("artifacts");
    crate::artifact_service::ArtifactService::open(
        &database_path,
        &artifact_root,
        crate::artifact_policy::ArtifactPolicy::default(),
    )
    .map_err(|_| {
        (
            503,
            "artifact_service_unavailable",
            "Le service d’artefacts est temporairement indisponible.".to_string(),
        )
    })
}

fn read_artifact_metrics(
    config: &UiRelayConfig,
    query: &HashMap<String, String>,
) -> Result<UiArtifactMetricsV1, UiArtifactError> {
    // La portée agent est vérifiée même si les compteurs ne contiennent aucun
    // contenu sensible : le relais n'est jamais une API de métriques anonyme.
    let _ = artifact_ui_scope(config, query.get("agent"))?;
    let metrics = open_artifact_service_for_ui(config)?
        .persisted_metrics()
        .map_err(artifact_service_error_response)?;
    Ok(UiArtifactMetricsV1 {
        version: UI_VERSION,
        canonical_blob_bytes: metrics.canonical_blob_bytes,
        publication_failures: metrics.publication_failures,
        restored_versions: metrics.restored_versions,
        evictions: metrics.evictions,
    })
}

fn artifact_service_error_response(
    error: crate::artifact_service::ArtifactServiceError,
) -> UiArtifactError {
    match error {
        crate::artifact_service::ArtifactServiceError::BlobUnavailable => (
            409,
            "artifact_blob_missing",
            "Le contenu canonique est absent. Vous pouvez réessayer une restauration par Bridget."
                .to_string(),
        ),
        crate::artifact_service::ArtifactServiceError::Interrupted => (
            409,
            "artifact_operation_interrupted",
            "L’opération a été interrompue avant toute nouvelle version. Vous pouvez demander à l’agent de reprendre."
                .to_string(),
        ),
        crate::artifact_service::ArtifactServiceError::PolicyBlocked => (
            409,
            "artifact_quota_blocked",
            "La conservation canonique est bloquée par son quota.".to_string(),
        ),
        crate::artifact_service::ArtifactServiceError::Validation(_) => (
            400,
            "artifact_validation_refused",
            "L’action d’artefact a été refusée par sa validation.".to_string(),
        ),
        crate::artifact_service::ArtifactServiceError::Store(_) => (
            409,
            "artifact_lifecycle_conflict",
            "L’action ne peut pas être appliquée à cette version.".to_string(),
        ),
        crate::artifact_service::ArtifactServiceError::BlobStore(_) => (
            503,
            "artifact_storage_unavailable",
            "Le stockage canonique est temporairement indisponible.".to_string(),
        ),
    }
}

fn post_artifact_lifecycle(
    config: &UiRelayConfig,
    body: &[u8],
) -> Result<UiArtifactLifecycleAcceptedV1, UiArtifactError> {
    let request: UiArtifactLifecycleRequestV1 = serde_json::from_slice(body).map_err(|_| {
        (
            400,
            "artifact_lifecycle_invalid",
            "Action d’artefact invalide.".to_string(),
        )
    })?;
    if request.version != UI_VERSION {
        return Err((
            400,
            "artifact_lifecycle_invalid",
            "Version d’action d’artefact invalide.".to_string(),
        ));
    }
    let (project_id, _) = artifact_ui_scope(config, Some(&request.agent))?;
    let version_ref = artifact_version_from_request(&request.version_ref)?;
    let store = open_artifact_store_for_ui(config)?;
    let artifact_ref = store
        .artifact_ref_for_version(&project_id, &version_ref)
        .map_err(|_| {
            (
                503,
                "artifact_lifecycle_unavailable",
                "L’action d’artefact est momentanément indisponible.".to_string(),
            )
        })?
        .ok_or_else(|| {
            (
                404,
                "artifact_not_found",
                "Cette version est introuvable dans le projet actif.".to_string(),
            )
        })?;
    drop(store);
    match request.action.as_str() {
        "pin" | "unpin" => {
            let pinned = request.pinned.unwrap_or(request.action.as_str() == "pin");
            open_artifact_service_for_ui(config)?
                .set_pinned(&project_id, &artifact_ref, pinned)
                .map_err(artifact_service_error_response)?;
        }
        "delete" => {
            open_artifact_service_for_ui(config)?
                .soft_delete(&project_id, &artifact_ref, now_secs())
                .map_err(artifact_service_error_response)?;
        }
        "restore" => {
            open_artifact_service_for_ui(config)?
                .restore(&project_id, &artifact_ref)
                .map_err(artifact_service_error_response)?;
        }
        _ => {
            return Err((
                400,
                "artifact_lifecycle_invalid",
                "Action d’artefact inconnue.".to_string(),
            ));
        }
    }
    Ok(UiArtifactLifecycleAcceptedV1 {
        version: UI_VERSION,
        action: request.action,
        artifact_ref,
    })
}

fn sandbox_state_is_safe(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(values) => values.iter().all(|(key, value)| {
            !key.to_ascii_lowercase().contains("url") && sandbox_state_is_safe(value)
        }),
        serde_json::Value::Array(values) => values.iter().all(sandbox_state_is_safe),
        serde_json::Value::String(value) => !["http:", "https:", "file:"]
            .iter()
            .any(|prefix| value.starts_with(prefix)),
        _ => true,
    }
}

fn post_artifact_sandbox_ticket(
    config: &UiRelayConfig,
    runtime: &UiRelayRuntime,
    body: &[u8],
) -> Result<UiArtifactSandboxTicketAcceptedV1, UiArtifactError> {
    let request: UiArtifactSandboxTicketRequestV1 = serde_json::from_slice(body).map_err(|_| {
        (
            400,
            "artifact_sandbox_ticket_invalid",
            "Demande de lecture sandbox invalide.".to_owned(),
        )
    })?;
    if request.version != UI_VERSION {
        return Err((
            400,
            "artifact_sandbox_ticket_invalid",
            "Version de lecture sandbox invalide.".to_owned(),
        ));
    }
    let (project_id, agent) = artifact_ui_scope(config, Some(&request.agent))?;
    let version_ref = artifact_version_from_request(&request.version_ref)?;
    let detail = read_artifact_detail_for_version(config, &project_id, &version_ref)?;
    if detail.publication.kind != crate::artifact_types::ArtifactKind::Html {
        return Err((
            409,
            "artifact_sandbox_kind_invalid",
            "Cette version n’est pas un artefact HTML sandboxé.".to_owned(),
        ));
    }
    Ok(UiArtifactSandboxTicketAcceptedV1 {
        version: UI_VERSION,
        ticket: runtime.issue_sandbox_ticket(project_id, agent, version_ref),
        expires_in_seconds: UI_SANDBOX_TICKET_TTL.as_secs(),
    })
}

fn read_artifact_sandbox_ticket(
    config: &UiRelayConfig,
    runtime: &UiRelayRuntime,
    ticket: Option<&String>,
) -> Result<String, UiArtifactError> {
    let Some(ticket) = ticket else {
        return Err((
            400,
            "artifact_sandbox_ticket_missing",
            "Ticket sandbox absent.".to_owned(),
        ));
    };
    if ticket.len() != 32 || !ticket.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err((
            400,
            "artifact_sandbox_ticket_invalid",
            "Ticket sandbox invalide.".to_owned(),
        ));
    }
    let ticket = runtime.resolve_sandbox_ticket(ticket).ok_or_else(|| {
        (
            410,
            "artifact_sandbox_ticket_expired",
            "Le ticket sandbox a expiré. Rechargez l’artefact via Bridget.".to_owned(),
        )
    })?;
    let (current_project_id, _) = artifact_ui_scope(config, Some(&ticket.agent))?;
    if current_project_id != ticket.project_id {
        return Err((
            403,
            "artifact_sandbox_scope_changed",
            "La portée projet a changé. Rechargez l’artefact via Bridget.".to_owned(),
        ));
    }
    let detail =
        read_artifact_detail_for_version(config, &current_project_id, &ticket.version_ref)?;
    if detail.publication.kind != crate::artifact_types::ArtifactKind::Html {
        return Err((
            409,
            "artifact_sandbox_kind_invalid",
            "Cette version n’est pas un artefact HTML sandboxé.".to_owned(),
        ));
    }
    let html = detail
        .publication
        .payload
        .get("html")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            (
                409,
                "artifact_sandbox_content_missing",
                "Le contenu HTML canonique est indisponible. Demandez une restauration par Bridget."
                    .to_owned(),
            )
        })?;
    Ok(html.to_owned())
}

fn read_artifact_sandbox_frame(
    config: &UiRelayConfig,
    runtime: &UiRelayRuntime,
    ticket: Option<&String>,
    frame_instance_id: Option<&String>,
    theme: Option<&String>,
) -> Result<String, UiArtifactError> {
    let frame_instance_id = frame_instance_id
        .filter(|value| {
            !value.is_empty()
                && value.len() <= 160
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        })
        .ok_or_else(|| {
            (
                400,
                "artifact_sandbox_frame_invalid",
                "Identifiant de frame sandbox invalide.".to_owned(),
            )
        })?;
    let html = read_artifact_sandbox_ticket(config, runtime, ticket)?;
    Ok(sandbox_frame_document(
        &html,
        frame_instance_id,
        sandbox_theme(theme),
    ))
}

fn sandbox_theme(theme: Option<&String>) -> &'static str {
    match theme.map(String::as_str) {
        Some("light") => "light",
        _ => "dark",
    }
}

fn sandbox_frame_document(html: &str, frame_instance_id: &str, theme: &str) -> String {
    let bootstrap = base64_standard(
        serde_json::to_string(&serde_json::json!({ "frame_instance_id": frame_instance_id }))
            .expect("bootstrap sandbox sérialisable")
            .as_bytes(),
    );
    let prelude = r#"<meta charset="utf-8"><script>(()=>{const __b=JSON.parse(decodeURIComponent(escape(atob('__BRIDGET_BOOTSTRAP__'))));const __setTheme=t=>{const next=t==='light'?'light':'dark';document.documentElement.dataset.bridgetTheme=next;document.documentElement.style.colorScheme=next;};const __height=()=>{const body=document.body;const top=body?body.getBoundingClientRect().top:0;const children=body?Array.from(body.children).filter(element=>!/^(SCRIPT|STYLE|TEMPLATE)$/i.test(element.tagName)):[];const height=children.length?Math.max(...children.map(element=>Math.ceil(element.getBoundingClientRect().bottom-top))):body?body.scrollHeight:0;parent.postMessage({type:'sandbox.resize',frame_instance_id:__b.frame_instance_id,height:Math.min(1200,Math.max(0,height))},'*');};let __queued=false;const __schedule=()=>{if(__queued)return;__queued=true;requestAnimationFrame(()=>{__queued=false;__height();});};__setTheme('__BRIDGET_THEME__');parent.postMessage({type:'sandbox.ready',frame_instance_id:__b.frame_instance_id},'*');window.addEventListener('message',e=>{if(!e.data)return;if(e.data.type==='sandbox.bootstrap')window.dispatchEvent(new CustomEvent('bridget-sandbox-bootstrap',{detail:e.data}));if(e.data.type==='sandbox.theme')__setTheme(e.data.theme);if(e.data.type==='sandbox.close')window.close();});window.addEventListener('load',()=>{if(window.ResizeObserver){const observer=new ResizeObserver(__schedule);observer.observe(document.documentElement);if(document.body)observer.observe(document.body);}new MutationObserver(__schedule).observe(document.documentElement,{subtree:true,childList:true,attributes:true,characterData:true});window.addEventListener('resize',__schedule);__schedule();});})();</script>"#
        .replace("__BRIDGET_BOOTSTRAP__", &bootstrap)
        .replace("__BRIDGET_THEME__", theme);
    let theme_style = r#"<style id="bridget-runtime-theme">html:root[data-bridget-theme="dark"]{color-scheme:dark;--bridget-background:#10161f;--bridget-foreground:#e8edf5;--bridget-muted:#a8b4c5;--bridget-border:#39495d;--bridget-panel:#182331;--ink:#e8edf5;--muted:#a8b4c5;--line:#39495d;--panel:#182331;--ground:#10161f;--bg:#10161f;--card:#182331;--soft:#223246;--accent:#76b8ff;--blue:#76b8ff;--blue-soft:#223e5e;--orange:#ffb45f;--green:#7acb9a}html:root[data-bridget-theme="light"]{color-scheme:light;--bridget-background:#f4f7fa;--bridget-foreground:#17212b;--bridget-muted:#637180;--bridget-border:#d9e1e8;--bridget-panel:#fff;--ink:#17212b;--muted:#637180;--line:#d9e1e8;--panel:#fff;--ground:#f4f7fa;--bg:#f4f7fa;--card:#fff;--soft:#e1f2f5;--accent:#126e82;--blue:#126e82;--blue-soft:#e1f2f5;--orange:#df6c00;--green:#16866b}html[data-bridget-theme]{height:auto!important;min-height:0!important}html[data-bridget-theme] body,html[data-bridget-theme] body>*{height:auto!important;min-height:0!important;min-block-size:0!important}html[data-bridget-theme] body{background:var(--bridget-background)!important;color:var(--bridget-foreground)!important}html[data-bridget-theme="dark"] :where(button,input:not([type=range]),select,textarea,[class~="metric"],[class~="result"],[class~="stat"],[class~="kpi"],[class~="tile"]){background-color:var(--bridget-panel)!important;color:var(--bridget-foreground)!important;border-color:var(--bridget-border)!important}html[data-bridget-theme="dark"] :where(.bar-tip,.legend,.note,.hint,.scale,[class$="-tip"],[class$="-note"],[class$="-hint"]){color:var(--bridget-muted)!important}</style>"#;
    let lowercase = html.to_ascii_lowercase();
    if let Some(head_start) = lowercase.find("<head") {
        if let Some(head_end) = lowercase[head_start..].find('>') {
            let insertion = head_start + head_end + 1;
            let mut document = html.to_owned();
            document.insert_str(insertion, &prelude);
            let closing_head = document.to_ascii_lowercase().find("</head>");
            if let Some(closing_head) = closing_head {
                document.insert_str(closing_head, theme_style);
            } else {
                document.insert_str(insertion + prelude.len(), theme_style);
            }
            return document;
        }
    }
    format!("<!doctype html><html><head>{prelude}{theme_style}</head><body>{html}</body></html>")
}

fn post_artifact_sandbox_save(
    config: &UiRelayConfig,
    body: &[u8],
) -> Result<UiArtifactSandboxSaveAcceptedV1, UiArtifactError> {
    let request: UiArtifactSandboxSaveRequestV1 = serde_json::from_slice(body).map_err(|_| {
        (
            400,
            "artifact_sandbox_save_invalid",
            "Sauvegarde sandbox invalide.".to_owned(),
        )
    })?;
    if request.version != UI_VERSION
        || serde_json::to_vec(&request.ui_state).map_or(true, |state| state.len() > 128 * 1024)
        || !sandbox_state_is_safe(&request.ui_state)
    {
        return Err((
            400,
            "artifact_sandbox_save_invalid",
            "L’état sandbox est invalide ou dépasse 128 Kio.".to_owned(),
        ));
    }
    let (project_id, agent) = artifact_ui_scope(config, Some(&request.agent))?;
    let parent_version_ref = artifact_version_from_request(&request.version_ref)?;
    let detail = read_artifact_detail_for_version(config, &project_id, &parent_version_ref)?;
    if detail.publication.kind != crate::artifact_types::ArtifactKind::Html {
        return Err((
            409,
            "artifact_sandbox_kind_invalid",
            "Seul un artefact HTML sandboxé peut enregistrer cet état.".to_owned(),
        ));
    }
    let store = open_artifact_store_for_ui(config)?;
    let artifact_ref = store
        .artifact_ref_for_version(&project_id, &parent_version_ref)
        .map_err(|_| {
            (
                503,
                "artifact_sandbox_unavailable",
                "Le registre d’artefacts est indisponible.".to_owned(),
            )
        })?
        .ok_or_else(|| {
            (
                404,
                "artifact_not_found",
                "Cette version est introuvable.".to_owned(),
            )
        })?;
    drop(store);
    let mut publication = detail.publication;
    publication.idempotency_key = format!("ui-html-save-{}", uuid::Uuid::new_v4().simple());
    publication
        .payload
        .as_object_mut()
        .ok_or_else(|| {
            (
                409,
                "artifact_sandbox_payload_invalid",
                "Le manifeste HTML est invalide.".to_owned(),
            )
        })?
        .insert("ui_state".to_owned(), request.ui_state);
    let persisted = open_artifact_service_for_ui(config)?
        .save_html_interaction(
            crate::artifact_store::ArtifactPublicationContext {
                project_id,
                conversation_reference: format!("conversation:ui:{agent}"),
                turn_reference: format!("turn:ui-html-save:{}", uuid::Uuid::new_v4().simple()),
                created_by: "operator:ui".to_owned(),
                origin_instance: "instance:ui".to_owned(),
                observed_at: now_secs(),
            },
            &artifact_ref,
            publication,
        )
        .map_err(artifact_service_error_response)?;
    let receipt = match persisted {
        crate::artifact_store::ArtifactPersistResult::Created(receipt)
        | crate::artifact_store::ArtifactPersistResult::Replayed(receipt) => receipt,
    };
    Ok(UiArtifactSandboxSaveAcceptedV1 {
        version: UI_VERSION,
        artifact_ref: receipt.artifact_ref,
        version_ref: receipt.version_ref,
        parent_version_ref,
    })
}

fn artifact_page_bounds(
    query: &HashMap<String, String>,
) -> Result<(usize, usize), UiArtifactError> {
    let offset = query
        .get("offset")
        .map(|value| value.parse::<usize>())
        .transpose()
        .map_err(|_| {
            (
                400,
                "artifact_page_invalid",
                "Pagination d’artefact invalide.".to_string(),
            )
        })?
        .unwrap_or(0);
    let limit = query
        .get("limit")
        .map(|value| value.parse::<usize>())
        .transpose()
        .map_err(|_| {
            (
                400,
                "artifact_page_invalid",
                "Pagination d’artefact invalide.".to_string(),
            )
        })?
        .unwrap_or(MAX_UI_ARTIFACT_PAGE_ROWS)
        .clamp(1, MAX_UI_ARTIFACT_PAGE_ROWS);
    Ok((offset, limit))
}

fn read_artifact_data_page(
    config: &UiRelayConfig,
    query: &HashMap<String, String>,
) -> Result<UiArtifactDataPageV1, UiArtifactError> {
    let detail = read_artifact_detail(config, query.get("agent"), query.get("version"))?;
    let (offset, limit) = artifact_page_bounds(query)?;
    let mut data = detail.publication.payload.clone();
    let mut has_more = false;
    if detail.publication.kind == crate::artifact_types::ArtifactKind::Table {
        if let Some(object) = data.as_object_mut() {
            if let Some(rows) = object
                .get("rows")
                .and_then(serde_json::Value::as_array)
                .cloned()
            {
                has_more = offset.saturating_add(limit) < rows.len();
                object.insert(
                    "rows".to_string(),
                    serde_json::Value::Array(rows.into_iter().skip(offset).take(limit).collect()),
                );
            }
        }
    }
    if serde_json::to_vec(&data)
        .map_err(|_| {
            (
                500,
                "artifact_serialization",
                "Données d’artefact illisibles.".to_string(),
            )
        })?
        .len()
        > MAX_UI_ARTIFACT_RESPONSE_BYTES
    {
        return Err((
            413,
            "artifact_data_too_large",
            "Les données dépassent la limite de consultation. Exportez la version ou réduisez la sélection.".to_string(),
        ));
    }
    Ok(UiArtifactDataPageV1 {
        version: UI_VERSION,
        version_ref: detail.item.version_ref,
        offset,
        limit,
        has_more,
        data,
    })
}

fn csv_escape(value: &serde_json::Value) -> String {
    let raw = match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(value) => value.clone(),
        value => value.to_string(),
    };
    format!("\"{}\"", raw.replace('"', "\"\""))
}

fn artifact_export(
    config: &UiRelayConfig,
    query: &HashMap<String, String>,
) -> Result<(&'static str, Vec<u8>, String), UiArtifactError> {
    let detail = read_artifact_detail(config, query.get("agent"), query.get("version"))?;
    let format = query.get("format").map(String::as_str).unwrap_or("json");
    let suffix = detail
        .item
        .version_ref
        .strip_prefix("artifact-version:")
        .unwrap_or("version");
    let (content_type, bytes, filename) = if format == "json" {
        (
            "application/json; charset=utf-8",
            serde_json::to_vec_pretty(&detail).map_err(|_| {
                (
                    500,
                    "artifact_serialization",
                    "Export d’artefact illisible.".to_string(),
                )
            })?,
            format!("bridget-artifact-{suffix}.json"),
        )
    } else if format == "csv"
        && detail.publication.kind == crate::artifact_types::ArtifactKind::Table
    {
        let payload = detail.publication.payload.as_object().ok_or_else(|| {
            (
                400,
                "artifact_csv_invalid",
                "Table d’artefact invalide.".to_string(),
            )
        })?;
        let columns = payload
            .get("columns")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| {
                (
                    400,
                    "artifact_csv_invalid",
                    "Colonnes de table absentes.".to_string(),
                )
            })?;
        let rows = payload
            .get("rows")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| {
                (
                    400,
                    "artifact_csv_invalid",
                    "Lignes de table absentes.".to_string(),
                )
            })?;
        let column_keys = columns
            .iter()
            .enumerate()
            .map(|(index, value)| match value {
                serde_json::Value::String(value) => (value.clone(), value.clone()),
                serde_json::Value::Object(object) => {
                    let key = object
                        .get("key")
                        .or_else(|| object.get("field"))
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    let label = object
                        .get("label")
                        .or_else(|| object.get("title"))
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or(&key)
                        .to_string();
                    (
                        if key.is_empty() {
                            format!("column_{}", index + 1)
                        } else {
                            key
                        },
                        label,
                    )
                }
                _ => (
                    format!("column_{}", index + 1),
                    format!("Colonne {}", index + 1),
                ),
            })
            .collect::<Vec<_>>();
        let mut csv = String::new();
        csv.push_str(
            &column_keys
                .iter()
                .map(|(_, label)| format!("\"{}\"", label.replace('"', "\"\"")))
                .collect::<Vec<_>>()
                .join(","),
        );
        for row in rows {
            csv.push('\n');
            let values = row.as_object();
            csv.push_str(
                &column_keys
                    .iter()
                    .map(|(key, _)| {
                        values
                            .and_then(|value| value.get(key))
                            .map(csv_escape)
                            .unwrap_or_else(|| "\"\"".to_string())
                    })
                    .collect::<Vec<_>>()
                    .join(","),
            );
        }
        (
            "text/csv; charset=utf-8",
            csv.into_bytes(),
            format!("bridget-artifact-{suffix}.csv"),
        )
    } else {
        return Err((
            400,
            "artifact_export_format",
            "Ce format d’export n’est pas disponible pour cet artefact.".to_string(),
        ));
    };
    if bytes.len() > MAX_UI_ARTIFACT_EXPORT_BYTES {
        return Err((
            413,
            "artifact_export_too_large",
            "Cet export dépasse la limite du relais. Utilisez une exportation paginée.".to_string(),
        ));
    }
    Ok((content_type, bytes, filename))
}

fn read_artifact_blob(
    config: &UiRelayConfig,
    query: &HashMap<String, String>,
) -> Result<(&'static str, Vec<u8>), UiArtifactError> {
    let (project_id, _) = artifact_ui_scope(config, query.get("agent"))?;
    let version_ref = artifact_version_from_query(query.get("version").map(String::as_str))?;
    let Some(digest) = query.get("digest") else {
        return Err((
            400,
            "artifact_blob_missing",
            "Blob d’artefact absent.".to_string(),
        ));
    };
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err((
            400,
            "artifact_blob_invalid",
            "Blob d’artefact invalide.".to_string(),
        ));
    }
    let store = open_artifact_store_for_ui(config)?;
    let blob = store
        .version_blob(&project_id, &version_ref, digest)
        .map_err(|_| {
            (
                503,
                "artifact_blob_unavailable",
                "Blob d’artefact indisponible.".to_string(),
            )
        })?
        .ok_or_else(|| {
            (
                404,
                "artifact_blob_not_found",
                "Blob introuvable dans cette version.".to_string(),
            )
        })?;
    let bytes = std::fs::read(&blob.canonical_path).map_err(|_| {
        (
            404,
            "artifact_blob_missing",
            "Le blob canonique est absent. Vous pouvez le restaurer.".to_string(),
        )
    })?;
    if bytes.len() as u64 != blob.byte_length
        || crate::artifact_types::sha256_hex(&bytes) != blob.digest
    {
        return Err((
            409,
            "artifact_blob_corrupt",
            "Le blob canonique est corrompu. Aucun aperçu n’est affiché.".to_string(),
        ));
    }
    let content_type = match blob.media_type.as_str() {
        "image/png" => "image/png",
        "image/jpeg" => "image/jpeg",
        "image/webp" => "image/webp",
        "image/gif" => "image/gif",
        "application/pdf" => "application/pdf",
        "text/plain" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    };
    Ok((content_type, bytes))
}

fn validate_ui_recipient(
    agents: &[bridget_transport::protocol::AgentInfo],
    recipient: &str,
) -> Result<(), (u16, &'static str)> {
    if recipient == UI_SENDER {
        return Err((400, "human_recipient"));
    }
    let Some(agent) = agents.iter().find(|agent| agent.agent_id == recipient) else {
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
    message.origin = Some(MessageOrigin::Human);
    message.intent = Some(MessageIntent::TriggerTurn);
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
            message_id: message_id.clone(),
            delivery_id,
            issued_at,
            status: "in_flight",
        }),
        DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::Accepted { .. },
            ..
        } => Ok(UiSendAcceptedV1 {
            version: UI_VERSION,
            message_id: message_id.clone(),
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
/// Unité = tour projeté (`message_id`), aligné sur `projectTimeline` de la page.
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
        .map(|home| {
            PathBuf::from(home)
                .join(".cache")
                .join("bridget")
                .join("sessions")
        })
        .unwrap_or_else(|_| PathBuf::from("/tmp/bridget/sessions"));
    root.join(agent)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectedTurn {
    first_seq: u64,
    last_seq: u64,
    record_count: usize,
}

/// Tours dans l'ordre d'apparition — clé = `message_id` (comme `recordKey` page).
fn projected_turns(journal_dir: &Path) -> Vec<ProjectedTurn> {
    let mut order: Vec<String> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();
    let mut turns: Vec<ProjectedTurn> = Vec::new();
    let mut files = std::fs::read_dir(journal_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "jsonl"))
        .collect::<Vec<_>>();
    files.sort();
    for path in files {
        for value in valid_events(&path) {
            let Some(seq) = value.get("seq").and_then(|s| s.as_u64()) else {
                continue;
            };
            let key = value
                .get("message_id")
                .and_then(|m| m.as_str())
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    let session = value
                        .get("session_id")
                        .and_then(|s| s.as_str())
                        .unwrap_or("session");
                    format!("{session}:{seq}")
                });
            if let Some(&idx) = index.get(&key) {
                let turn = &mut turns[idx];
                turn.last_seq = turn.last_seq.max(seq);
                turn.first_seq = turn.first_seq.min(seq);
                turn.record_count += 1;
            } else {
                index.insert(key.clone(), turns.len());
                order.push(key);
                turns.push(ProjectedTurn {
                    first_seq: seq,
                    last_seq: seq,
                    record_count: 1,
                });
            }
        }
    }
    let _ = order;
    turns
}

/// Sélectionne les `max_turns` plus récents **entiers**, sans couper un
/// `message_id`, sous le plafond de fragments. Le tour le plus récent est
/// toujours pris même s'il dépasse le plafond seul (has_more alors vrai).
fn select_recent_turns(
    turns: &[ProjectedTurn],
    max_turns: usize,
    max_records: usize,
) -> (u64, bool) {
    if turns.is_empty() {
        return (0, false);
    }
    let mut selected: Vec<&ProjectedTurn> = Vec::new();
    let mut records = 0_usize;
    for turn in turns.iter().rev() {
        if selected.len() >= max_turns {
            break;
        }
        if !selected.is_empty() && records + turn.record_count > max_records {
            break;
        }
        selected.push(turn);
        records += turn.record_count;
    }
    selected.reverse();
    let from_seq = selected.first().map(|t| t.first_seq).unwrap_or(0);
    let has_more = selected.len() < turns.len();
    (from_seq, has_more)
}

#[cfg_attr(not(test), allow(dead_code))]
fn older_page_from_turns(
    turns: &[ProjectedTurn],
    current_from: u64,
    page_turns: usize,
    max_records: usize,
) -> (u64, bool) {
    let older: Vec<ProjectedTurn> = turns
        .iter()
        .filter(|turn| turn.first_seq < current_from)
        .cloned()
        .collect();
    select_recent_turns(&older, page_turns, max_records)
}

fn resolve_ui_journal_window_in(
    journal_dir: PathBuf,
    query: &HashMap<String, String>,
) -> Result<(AttachWindow, UiJournalPage), UiError> {
    let turns = projected_turns(&journal_dir);
    match query.get("from_seq") {
        Some(value) => {
            let from_seq = value.parse().map_err(|_| {
                UiError::Protocol("from_seq doit être un entier non signé".to_string())
            })?;
            let has_more = turns.iter().any(|turn| turn.first_seq < from_seq);
            Ok((
                AttachWindow::Seq(from_seq),
                UiJournalPage { has_more, from_seq },
            ))
        }
        None => {
            let (from_seq, has_more) = select_recent_turns(
                &turns,
                UI_JOURNAL_OPEN_TURNS,
                UI_JOURNAL_MAX_REPLAY_FRAGMENTS,
            );
            Ok((
                AttachWindow::Seq(from_seq),
                UiJournalPage { has_more, from_seq },
            ))
        }
    }
}

/// Remontée manuelle indicative (pages de tours, pas de séquences brutes).
#[allow(dead_code)]
fn older_journal_page_from_seq(current_from_seq: u64) -> u64 {
    current_from_seq.saturating_sub(UI_JOURNAL_OLDER_PAGE_TURNS as u64)
}

fn read_snapshot(
    config: &UiRelayConfig,
    focus_agent: Option<&str>,
) -> Result<UiSnapshotV1, UiError> {
    let facts = read_bridget_snapshot(&config.daemon_socket)?;
    let artifact_references =
        focus_agent.map(|agent| read_artifact_references(config, &facts.agents, agent));
    let mut agent_ids = facts
        .agents
        .iter()
        .map(|agent| agent.agent_id.clone())
        .collect::<Vec<_>>();
    agent_ids.extend(
        facts
            .messages
            .iter()
            .flat_map(|message| [message.sender.clone(), message.target.clone()]),
    );
    let profiles = profile_summaries_for_agent_ids(config, &agent_ids);
    let recovery_display_names = facts
        .agents
        .iter()
        .filter_map(|agent| {
            profiles
                .get(&agent.agent_id)
                .map(|profile| (agent.agent_id.clone(), profile.display_name.clone()))
        })
        .collect::<HashMap<_, _>>();
    let agents = compose_agent_rows(facts.agents, &facts.messages, &profiles);
    let peer_exchanges = focus_agent.map(|agent| aggregate_peer_exchanges(agent, &facts.messages));
    // Chemin productif du fil humain↔référent : lecture ledger filtrée sur le
    // couple AVANT toute borne (pas les 200 globaux de peer_exchanges). Le
    // journal d'agent ne porte pas les sorties vers l'utilisateur (DETTE :
    // asymétrie journal). peer_exchange reste agent↔agent.
    let thread_messages =
        focus_agent.map(|agent| load_human_referent_thread(&config.daemon_socket, agent));
    // Chemin productif missions page : filtre vivants — un mutant qui retire
    // cet appel dans read_snapshot doit tuer le témoin
    // `chemin_productif_snapshot_emprunte_retain_living_objectives`.
    let missions = read_public_mission_projection_v1(&config.maicie_config)
        .map_err(|error| UiError::Configuration(error.to_string()))?
        .map(retain_living_objectives)
        .unwrap_or_else(MissionProjectionV1::empty);
    let recovery_losses = read_recovery_losses(&config.daemon_socket, &recovery_display_names);
    Ok(UiSnapshotV1 {
        version: UI_VERSION,
        agents,
        alert_thresholds: UiAlertThresholdsV1::default(),
        peer_exchanges,
        thread_messages,
        artifact_references,
        open_requests: facts.open_requests,
        missions,
        recovery_losses,
    })
}

/// Un agent arrêté ne peut plus alimenter Attach, mais cela ne rend pas le
/// relais indisponible. Le snapshot du daemon est la source de vérité pour
/// distinguer ce cas d'une vraie déconnexion du daemon.
fn focused_agent_is_stopped(config: &UiRelayConfig, agent: &str) -> Result<bool, UiError> {
    let snapshot = read_snapshot(config, Some(agent))?;
    Ok(snapshot
        .agents
        .iter()
        .any(|candidate| candidate.agent_id == agent && candidate.state == "stopped"))
}

fn profile_summaries_for_agent_ids(
    config: &UiRelayConfig,
    agent_ids: &[String],
) -> HashMap<String, AgentProfileSummary> {
    let mut store = match AgentProfileStore::open(&ledger_db_path_for_socket(&config.daemon_socket))
    {
        Ok(store) => store,
        Err(_) => return HashMap::new(),
    };
    if store.ensure_agent_ids(agent_ids).is_err() {
        return HashMap::new();
    }
    agent_ids
        .iter()
        .filter_map(|agent_id| {
            store
                .profile_for_agent_id(agent_id)
                .ok()
                .flatten()
                .map(|profile| (agent_id.clone(), profile))
        })
        .collect()
}

fn ui_agent_profile(summary: &AgentProfileSummary) -> UiAgentProfileV1 {
    UiAgentProfileV1 {
        agent_id: summary.agent_id.clone(),
        display_name: summary.display_name.clone(),
        labels: summary.labels.clone(),
        avatar: UiAgentAvatarV1 {
            shape: summary.avatar_shape.clone(),
            color: summary.avatar_color.clone(),
        },
        instruction_state: UiInstructionStateV1 {
            revision: summary.instructions_revision,
            status: summary.instruction_status.as_str(),
            updated_at: summary.updated_at,
        },
    }
}

fn compose_agent_rows(
    agents: Vec<bridget_transport::protocol::AgentInfo>,
    messages: &[LedgerMessage],
    profiles: &HashMap<String, AgentProfileSummary>,
) -> Vec<UiAgentRowV1> {
    agents
        .into_iter()
        .map(|agent| {
            let project_id = agent
                .agent_link
                .as_ref()
                .and_then(|link| link.project.as_ref())
                .map(|project| project.project_id.clone());
            let project_state = if project_id.is_some() {
                "registered"
            } else {
                "unregistered"
            };
            let last = messages
                .iter()
                .filter(|message| {
                    message.sender == agent.agent_id || message.target == agent.agent_id
                })
                .max_by(|left, right| (left.ts, &left.id).cmp(&(right.ts, &right.id)));
            let connection_state = public_agent_state(&agent.state);
            let (turn_state, wait_state, progress_age_secs, queue_depth, continuation_mode) = agent
                .execution
                .as_ref()
                .map(|execution| {
                    (
                        execution.state.clone(),
                        execution.wait_state.clone(),
                        execution.progress_age_secs,
                        execution.queue_depth,
                        execution.continuation_mode.clone(),
                    )
                })
                .unwrap_or((None, None, None, 0, None));
            let message_age_secs =
                last.and_then(|message| u64::try_from(now_secs().saturating_sub(message.ts)).ok());
            let alerts = execution_alerts(
                turn_state.as_deref(),
                wait_state.as_deref(),
                progress_age_secs,
                queue_depth,
                message_age_secs,
            );

            UiAgentRowV1 {
                agent_id: agent.agent_id.clone(),
                profile: profiles.get(&agent.agent_id).map(ui_agent_profile),
                agent_type: agent.agent_type,
                host: agent.host,
                transport: agent.transport,
                domain: agent.domain,
                project_id,
                project_state,
                mode: agent.mode,
                model: agent.model,
                effort: agent.effort,
                persistent: agent.persistent,
                state: connection_state,
                connection_state,
                provider_age_secs: agent.last_seen_secs,
                turn_state,
                wait_state,
                agent_link: agent.agent_link,
                provider: agent.provider,
                progress_age_secs,
                queue_depth,
                continuation_mode,
                alerts,
                last_message_at: last.map(|message| message.ts),
                last_excerpt: last.map(|message| excerpt(&message.body)),
                unread: messages
                    .iter()
                    .filter(|message| {
                        message.sender == agent.agent_id && message.target == UI_SENDER
                    })
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

/// Projection UI d'une ronde connue. Le registre est la source qui porte le
/// corps complet, même quand la fenêtre de journal a déjà dépassé son début.
fn vigilance_round_projection(message: &LedgerMessage) -> Option<UiVigilanceRoundV1> {
    let header = message.body.lines().next()?.trim();
    let remainder = header.strip_prefix("RONDE DE VIGILANCE (")?;
    let (interval, headline) = remainder.split_once(')')?;
    let headline = headline
        .trim_start_matches(|character: char| {
            character.is_whitespace() || matches!(character, '-' | '\u{2013}' | '\u{2014}')
        })
        .trim();
    if interval.trim().is_empty() || headline.is_empty() {
        return None;
    }
    let marker = "--- SIGNAL MECANIQUE DE LA RONDE ---";
    let marker_index = message
        .body
        .lines()
        .position(|line| line.trim() == marker)?;
    let signal = message
        .body
        .lines()
        .skip(marker_index + 1)
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(2)
        .collect::<Vec<_>>()
        .join(" ");
    Some(UiVigilanceRoundV1 {
        interval: interval.trim().to_string(),
        headline: headline.to_string(),
        signal,
        body: message.body.clone(),
    })
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
        let round = vigilance_round_projection(message);
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
            if exchange.vigilance_round.is_none() {
                exchange.vigilance_round = round;
            }
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
            vigilance_round: round,
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

/// Plafond du fil humain↔agent : borne CE fil seulement. Le trafic entre agents
/// n'entre pas dans le compte — filtrer avant de borner.
const UI_THREAD_MESSAGE_LIMIT: usize = 500;

/// Charge le fil humain↔agent depuis le ledger en filtrant le couple d'abord.
/// C'est le chemin que `/v1/snapshot` et le watch empruntent ; un mutant qui
/// reviendrait à `recent_messages` global puis filtre doit tuer
/// `TEMOIN_fil_humain_survit_au_trafic_agent`.
fn load_human_referent_thread(socket_path: &Path, focus_agent: &str) -> Vec<UiThreadMessageV1> {
    let db_path = ledger_db_path_for_socket(socket_path);
    let Ok(store) = crate::store::Store::open(&db_path) else {
        return Vec::new();
    };
    // La propre entrée de l'humain n'est pas un couple : la clé de conversation
    // (humain, humain) n'existe pas et rendait un fil vide. Sa boîte se lit donc
    // par participant, tous correspondants confondus.
    let loaded = if focus_agent == UI_SENDER {
        store.participant_messages(UI_SENDER, UI_THREAD_MESSAGE_LIMIT)
    } else {
        store.conversation_messages(UI_SENDER, focus_agent, UI_THREAD_MESSAGE_LIMIT)
    };
    let Ok(entries) = loaded else {
        return Vec::new();
    };
    let messages = entries
        .into_iter()
        .map(|entry| LedgerMessage {
            id: entry.id,
            ts: entry.ts,
            sender: entry.sender,
            target: entry.target,
            body: entry.body,
            delivery_status: None,
        })
        .collect::<Vec<_>>();
    human_referent_thread_messages(focus_agent, &messages)
}

/// Messages ledger entre l'utilisateur (`humain`) et l'agent focal — dans l'ordre
/// d'émission, avec corps. Projection pure sur un jeu déjà filtré (ou de test).
/// Un mutant qui coupe `load_human_referent_thread` dans `read_snapshot` doit
/// tuer `chemin_productif_snapshot_emprunte_human_referent_thread_messages`.
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
            if focus_agent == UI_SENDER {
                // L'humain focalisé sur sa propre entrée : le filtre par paire
                // cherchait alors des messages humain -> humain, qui n'existent
                // pas, et rendait un fil vide. Mesuré le 28/08 : la colonne
                // affichait le titre de chaque réponse et la conversation
                // restait vide, donc l'humain ne pouvait pas lire ce qui lui
                // était adressé sans deviner qu'il fallait sélectionner
                // l'agent. Sa propre entrée montre désormais sa boîte entière,
                // tous correspondants confondus.
                message.sender == UI_SENDER || message.target == UI_SENDER
            } else {
                (message.sender == UI_SENDER && message.target == focus_agent)
                    || (message.sender == focus_agent && message.target == UI_SENDER)
            }
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
    for message in load_human_referent_thread(socket_path, focus_agent) {
        if seen.insert(message.delivery_id.clone()) {
            write_sse(http, "thread_message", &message)?;
        }
    }
    Ok(())
}

/// Les publications ne produisent pas forcément un fragment de journal. Le
/// watch doit donc aussi rafraîchir sa projection attestée : sans cela un
/// artefact nouvellement publié n'apparaît qu'après reconnexion de la vue.
/// Les messages du snapshot sont dédoublonnés côté client par `delivery_id`.
fn push_live_snapshot(
    http: &mut TcpStream,
    config: &UiRelayConfig,
    focus_agent: &str,
    last_artifact_signature: &mut Option<String>,
) -> Result<(), UiError> {
    let snapshot = read_snapshot(config, Some(focus_agent))?;
    let artifact_signature = artifact_reference_signature(&snapshot)?;
    if !should_push_live_snapshot(last_artifact_signature.as_deref(), &artifact_signature) {
        return Ok(());
    }
    *last_artifact_signature = Some(artifact_signature);
    write_sse(http, "snapshot", &snapshot)
}

fn artifact_reference_signature(snapshot: &UiSnapshotV1) -> Result<String, UiError> {
    serde_json::to_string(&snapshot.artifact_references)
        .map_err(|error| UiError::Protocol(format!("signature d'artefact invalide: {error}")))
}

fn should_push_live_snapshot(last_artifact_signature: Option<&str>, current: &str) -> bool {
    last_artifact_signature != Some(current)
}

fn seed_thread_message_ids(snapshot: &UiSnapshotV1, seen: &mut HashSet<String>) {
    if let Some(messages) = &snapshot.thread_messages {
        for message in messages {
            seen.insert(message.delivery_id.clone());
        }
    }
}

fn recovery_losses_path_for_socket(socket_path: &Path) -> PathBuf {
    crate::recovery_trace::report_path(&crate::desired_state::path_for_daemon_db(
        &socket_path.with_extension("db"),
    ))
}

fn read_recovery_losses(
    socket_path: &Path,
    display_names: &HashMap<String, String>,
) -> Vec<UiRecoveryLossV1> {
    match crate::recovery_trace::load_report(&recovery_losses_path_for_socket(socket_path)) {
        Ok(Some(report)) => report
            .absents
            .into_iter()
            .map(|entry| UiRecoveryLossV1 {
                display_name: display_names
                    .get(&entry.name)
                    .cloned()
                    .unwrap_or_else(|| "Agent indisponible".to_string()),
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
        Err(_error) if snapshot_config.is_some() => {
            let config = snapshot_config.expect("snapshot config vérifiée par la garde");
            if focused_agent_is_stopped(config, agent)? {
                return stream_sse_thread_watch_after_headers(http, config, agent);
            }
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
    let mut last_artifact_signature = initial_snapshot
        .as_ref()
        .map(artifact_reference_signature)
        .transpose()?;
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
                    push_live_snapshot(http, config, agent, &mut last_artifact_signature)?;
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
                    last_artifact_signature = Some(artifact_reference_signature(snapshot)?);
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
        if let Some(config) = snapshot_config
            && last_thread_poll.elapsed() >= UI_THREAD_LEDGER_POLL
        {
            let _ =
                push_live_thread_messages(http, &config.daemon_socket, agent, &mut seen_thread_ids);
            push_live_snapshot(http, config, agent, &mut last_artifact_signature)?;
            last_thread_poll = Instant::now();
        }
        if matches!(event, DaemonToWrapper::End { .. }) {
            if let Some(config) = snapshot_config {
                if focused_agent_is_stopped(config, agent)? {
                    return stream_sse_thread_watch_after_headers(http, config, agent);
                }
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

/// Sert les messages d'un participant qui n'a pas de journal de tour.
///
/// Réutilise `read_snapshot` + `write_snapshot_sse`, exactement ce que fait
/// `/v1/watch` ; rien n'est inventé ici. La différence avec `stream_sse_journal`
/// est qu'aucune attache n'est ouverte : il n'y a rien à quoi s'attacher.
fn stream_sse_thread_only(
    http: &mut TcpStream,
    config: &UiRelayConfig,
    agent: &str,
) -> Result<(), UiError> {
    let snapshot = read_snapshot(config, Some(agent))?;
    write!(
        http,
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n"
    )?;
    http.flush()?;
    write_snapshot_sse(http, &snapshot)?;
    let _ = http.shutdown(Shutdown::Both);
    Ok(())
}

/// Garde un watch vivant pour un interlocuteur sans journal de tour, notamment
/// `humain`. Une tentative Attach serait rejetée comme non enregistrée et
/// laisserait le client dans une boucle de reconnexion alors que le relais est
/// disponible.
fn stream_sse_thread_watch(
    http: &mut TcpStream,
    config: &UiRelayConfig,
    agent: &str,
) -> Result<(), UiError> {
    write!(
        http,
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n"
    )?;
    http.flush()?;
    stream_sse_thread_watch_after_headers(http, config, agent)
}

/// Continue un `/v1/watch` dont les en-têtes SSE sont déjà envoyés. Ce chemin
/// sert aussi lorsqu'un agent arrêté ferme Attach : l'état arrêté appartient à
/// l'agent, pas au transport entre Bridget Desktop et le relais.
fn stream_sse_thread_watch_after_headers(
    http: &mut TcpStream,
    config: &UiRelayConfig,
    agent: &str,
) -> Result<(), UiError> {
    let snapshot = read_snapshot(config, Some(agent))?;
    write_relay_state(http, "connected", now_secs())?;
    let mut seen_thread_ids = HashSet::new();
    let mut last_artifact_signature = Some(artifact_reference_signature(&snapshot)?);
    seed_thread_message_ids(&snapshot, &mut seen_thread_ids);
    write_snapshot_sse(http, &snapshot)?;

    loop {
        thread::sleep(UI_THREAD_LEDGER_POLL);
        push_live_thread_messages(http, &config.daemon_socket, agent, &mut seen_thread_ids)?;
        push_live_snapshot(http, config, agent, &mut last_artifact_signature)?;
        write!(http, ": keepalive\n\n")?;
        http.flush()?;
    }
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
    headers: HashMap<String, String>,
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
        headers,
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

/// ETag = empreinte du corps. Change à chaque livraison de binaire ; le navigateur
/// revalide systématiquement (`Cache-Control: no-cache`) et reçoit 304 si inchangé.
fn asset_etag(body: &[u8]) -> String {
    format!("\"{:x}\"", Sha256::digest(body))
}

fn if_none_match_hits(header: &str, etag: &str) -> bool {
    let wanted = etag.trim().trim_matches('"');
    header.split(',').any(|candidate| {
        let candidate = candidate.trim();
        let candidate = candidate
            .strip_prefix("W/")
            .unwrap_or(candidate)
            .trim()
            .trim_matches('"');
        !candidate.is_empty() && candidate == wanted
    })
}

/// Assets UI : fraîcheur d'abord. `no-cache` force la revalidation à chaque
/// ouverture ; ETag évite de renvoyer ~200 Ko quand le fichier n'a pas bougé.
fn write_asset(
    stream: &mut TcpStream,
    content_type: &str,
    body: &[u8],
    if_none_match: Option<&str>,
) -> Result<(), UiError> {
    let etag = asset_etag(body);
    if if_none_match.is_some_and(|value| if_none_match_hits(value, &etag)) {
        write!(
            stream,
            "HTTP/1.1 304 {}\r\nETag: {etag}\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n",
            status_text(304),
        )?;
        stream.flush()?;
        return Ok(());
    }
    write!(
        stream,
        "HTTP/1.1 200 {}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nETag: {etag}\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n",
        status_text(200),
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

fn write_binary(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
    filename: Option<&str>,
) -> Result<(), UiError> {
    let disposition = filename
        .map(|name| format!("Content-Disposition: attachment; filename=\"{name}\"\r\n"))
        .unwrap_or_default();
    write!(
        stream,
        "HTTP/1.1 {status} {}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n{disposition}Cache-Control: no-store\r\nConnection: close\r\n\r\n",
        status_text(status),
        body.len(),
    )?;
    stream.write_all(body)?;
    stream.flush()?;
    Ok(())
}

fn write_sandbox_html(stream: &mut TcpStream, body: &str) -> Result<(), UiError> {
    write!(
        stream,
        "HTTP/1.1 200 {}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Security-Policy: {UI_SANDBOX_CSP}\r\nX-Content-Type-Options: nosniff\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status_text(200),
        body.len(),
    )?;
    stream.write_all(body.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn status_text(status: u16) -> &'static str {
    match status {
        200 => "OK",
        202 => "Accepted",
        304 => "Not Modified",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        413 => "Payload Too Large",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
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
            "agent_id": name,
            "display_name": name,
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

    fn agent_info_in_project(
        name: &str,
        project_id: &str,
    ) -> bridget_transport::protocol::AgentInfo {
        let mut agent = agent_info(name, "connected");
        agent.agent_link = Some(bridget_transport::protocol::AgentLinkUiProjection {
            link_id: "link:test".to_string(),
            parent_instance_id: "instance:parent".to_string(),
            parent_execution_id: None,
            objective_id: None,
            delegation_id: None,
            project: Some(bridget_transport::protocol::ProjectReference {
                project_id: project_id.to_string(),
                binding_generation: 1,
            }),
            role: "worker".to_string(),
            agent_path: name.to_string(),
            state: "active".to_string(),
            direct_descendants: 0,
            descendants: 0,
        });
        agent
    }

    #[test]
    fn relais_artefact_pagine_exporte_et_refuse_la_version_d_un_autre_projet() {
        let test_id = uuid::Uuid::new_v4().simple().to_string();
        let root = std::env::temp_dir().join(format!("bui-artifact-{}", &test_id[..8]));
        std::fs::create_dir_all(&root).unwrap();
        let socket = root.join("bridget.sock");
        let db = ledger_db_path_for_socket(&socket);
        let artifacts = root.join("artifacts");
        let mut service = crate::artifact_service::ArtifactService::open(
            &db,
            &artifacts,
            crate::artifact_policy::ArtifactPolicy::default(),
        )
        .unwrap();
        let mut publication: crate::artifact_types::ArtifactPublicationV1 = serde_json::from_str(
            include_str!("../tests/fixtures/artifacts/chart-external-v1.json"),
        )
        .unwrap();
        publication.kind = crate::artifact_types::ArtifactKind::Table;
        publication.payload = serde_json::json!({
            "columns": ["id", "label"],
            "rows": [
                {"id": 1, "label": "un"}, {"id": 2, "label": "deux"},
                {"id": 3, "label": "trois"}, {"id": 4, "label": "quatre"}
            ]
        });
        let receipt = match service
            .publish(
                crate::artifact_store::ArtifactPublicationContext {
                    project_id: "project:relay-one".to_string(),
                    conversation_reference: "conversation:project:relay-one:agent-relay"
                        .to_string(),
                    turn_reference: "turn:relay-one".to_string(),
                    created_by: "agent:relay".to_string(),
                    origin_instance: "instance:relay".to_string(),
                    observed_at: 1_800_000_000,
                },
                publication,
            )
            .unwrap()
        {
            crate::artifact_store::ArtifactPersistResult::Created(receipt) => receipt,
            crate::artifact_store::ArtifactPersistResult::Replayed(_) => panic!("création relay"),
        };
        let mut other: crate::artifact_types::ArtifactPublicationV1 = serde_json::from_str(
            include_str!("../tests/fixtures/artifacts/chart-external-v1.json"),
        )
        .unwrap();
        other.idempotency_key = "fixture-ui-relay-other".to_string();
        let other_receipt = match service
            .publish(
                crate::artifact_store::ArtifactPublicationContext {
                    project_id: "project:relay-two".to_string(),
                    conversation_reference: "conversation:project:relay-two:agent-other"
                        .to_string(),
                    turn_reference: "turn:relay-two".to_string(),
                    created_by: "agent:other".to_string(),
                    origin_instance: "instance:other".to_string(),
                    observed_at: 1_800_000_001,
                },
                other,
            )
            .unwrap()
        {
            crate::artifact_store::ArtifactPersistResult::Created(receipt) => receipt,
            crate::artifact_store::ArtifactPersistResult::Replayed(_) => panic!("création autre"),
        };
        drop(service);

        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            for _ in 0..4 {
                let (stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut writer = BufWriter::new(stream);
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                assert!(matches!(
                    decode::<WrapperToDaemon>(line.trim()).unwrap(),
                    WrapperToDaemon::ListAgents
                ));
                writeln!(
                    writer,
                    "{}",
                    encode(&DaemonToWrapper::AgentList {
                        agents: vec![agent_info_in_project("agent-relay", "project:relay-one")]
                    })
                    .unwrap()
                )
                .unwrap();
                writer.flush().unwrap();
                line.clear();
                reader.read_line(&mut line).unwrap();
                assert!(matches!(
                    decode::<WrapperToDaemon>(line.trim()).unwrap(),
                    WrapperToDaemon::LedgerProjection { .. }
                ));
                writeln!(
                    writer,
                    "{}",
                    encode(&DaemonToWrapper::LedgerProjection {
                        messages: Vec::new(),
                        requests: Vec::new(),
                    })
                    .unwrap()
                )
                .unwrap();
                writer.flush().unwrap();
            }
        });
        let config = UiRelayConfig {
            daemon_socket: socket.clone(),
            maicie_config: root.join("maicie.json"),
            project_root_policy_path: None,
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            token: "artifact-relay".to_string(),
        };
        let wire_version = receipt
            .version_ref
            .replace("artifact-version:", "artifact-version.");
        let mut query = HashMap::from([
            ("agent".to_string(), "agent-relay".to_string()),
            ("version".to_string(), wire_version),
            ("offset".to_string(), "1".to_string()),
            ("limit".to_string(), "2".to_string()),
        ]);
        let page = read_artifact_data_page(&config, &query).unwrap();
        assert_eq!(page.offset, 1);
        assert!(page.has_more);
        assert_eq!(page.data["rows"].as_array().unwrap().len(), 2);
        query.insert("format".to_string(), "csv".to_string());
        let (content_type, csv, _) = artifact_export(&config, &query).unwrap();
        assert_eq!(content_type, "text/csv; charset=utf-8");
        assert!(String::from_utf8(csv).unwrap().contains("trois"));
        let metrics = read_artifact_metrics(
            &config,
            &HashMap::from([("agent".to_string(), "agent-relay".to_string())]),
        )
        .unwrap();
        assert_eq!(metrics.canonical_blob_bytes, 0);
        assert_eq!(metrics.publication_failures, 0);
        query.insert(
            "version".to_string(),
            other_receipt
                .version_ref
                .replace("artifact-version:", "artifact-version."),
        );
        assert!(matches!(
            read_artifact_detail(&config, query.get("agent"), query.get("version")),
            Err((404, "artifact_not_found", _))
        ));
        server.join().unwrap();
        let _ = std::fs::remove_dir_all(root);
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

    #[test]
    fn acceptation_ui_expose_message_id_distinct_de_la_remise() {
        let accepted = UiSendAcceptedV1 {
            version: UI_VERSION,
            message_id: "message-9afa".to_string(),
            delivery_id: "delivery-ccff".to_string(),
            issued_at: 42,
            status: "in_flight",
        };

        let json = serde_json::to_value(accepted).unwrap();
        assert_eq!(json["message_id"], "message-9afa");
        assert_eq!(json["delivery_id"], "delivery-ccff");
    }

    #[test]
    fn spec_079_message_ui_est_humain_et_declenche_un_tour() {
        let socket_path = std::env::temp_dir().join(format!(
            "bridget-ui-spec-079-{}.sock",
            uuid::Uuid::new_v4().simple()
        ));
        let listener = UnixListener::bind(&socket_path).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            let mut line = String::new();

            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode::<WrapperToDaemon>(line.trim()).unwrap(),
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Client
                }
            ));
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client,
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();

            line.clear();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode::<WrapperToDaemon>(line.trim()).unwrap(),
                WrapperToDaemon::ClientHello { capabilities, .. }
                    if capabilities == vec![ClientCapability::SendIdempotent]
            ));
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: "spec-079".to_string(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: vec![ClientCapability::SendIdempotent],
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();

            line.clear();
            reader.read_line(&mut line).unwrap();
            let message_id = match decode::<WrapperToDaemon>(line.trim()).unwrap() {
                WrapperToDaemon::SendIdempotent {
                    message,
                    message_id,
                    ..
                } => {
                    assert_eq!(message.id, message_id);
                    assert_eq!(message.from, UI_SENDER);
                    assert_eq!(message.to, "coordinateur");
                    assert_eq!(message.body, "continue le travail");
                    assert_eq!(message.origin, Some(MessageOrigin::Human));
                    assert_eq!(message.intent, Some(MessageIntent::TriggerTurn));
                    message_id
                }
                other => panic!("SendIdempotent attendu, reçu {other:?}"),
            };
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::IdempotencyResult {
                    operation_kind: "send".to_string(),
                    idempotency_key: message_id,
                    issue: IdempotencyIssue::OutcomeUnknown {
                        expires_at: now_secs() + 60,
                        delivery_id: Some("delivery-ui-079".to_string()),
                    },
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();
        });

        let accepted = send_ui_message(
            &socket_path,
            UiSendRequestV1 {
                version: UI_VERSION,
                to: "coordinateur".to_string(),
                body: "continue le travail".to_string(),
                reply: false,
            },
        )
        .unwrap();
        server.join().unwrap();
        assert_eq!(accepted.delivery_id, "delivery-ui-079");
        std::fs::remove_file(socket_path).unwrap();
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
                    agent_id: UI_SENDER.to_string(),
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
    fn refus_presence_humaine_n_est_pas_confondue_avec_panne_daemon() {
        // Oracle de contrat : les trois étapes exposent des causes distinctes.
        // Un remappage de la présence vers daemon_unavailable doit donc échouer
        // ici, même si le transport reste sain.
        assert_ne!("human_sender_unregistered", "daemon_unavailable");
        assert_ne!("send_failed", "daemon_unavailable");
        assert_ne!("human_sender_unregistered", "send_failed");
    }

    #[test]
    fn refus_reponse_sans_presence_declenche_la_reinscription() {
        let refusal = UiError::Protocol(
            "issue idempotente en vol attendue, reçu reply_sender_unavailable".to_string(),
        );
        assert!(is_reply_sender_unavailable(&refusal));
        assert!(!is_reply_sender_unavailable(&UiError::Protocol(
            "recipient_unavailable".to_string(),
        )));
    }

    #[test]
    fn refus_presence_humaine_remonte_le_code_reel() {
        let socket_path = std::env::temp_dir().join(format!(
            "bridget-ui-presence-{}.sock",
            uuid::Uuid::new_v4().simple()
        ));
        let listener = UnixListener::bind(&socket_path).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode::<WrapperToDaemon>(line.trim()).unwrap(),
                WrapperToDaemon::ListAgents
            ));
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::AgentList {
                    agents: vec![agent_info("rc1", "idle")]
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();

            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            line.clear();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode::<WrapperToDaemon>(line.trim()).unwrap(),
                WrapperToDaemon::Register { .. }
            ));
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::AttachRejected {
                    subscription_id: None,
                    reason: bridget_transport::protocol::AttachRefusal::JournalUnavailable,
                    mode: None,
                    location: None
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();
        });
        let config = UiRelayConfig {
            daemon_socket: socket_path.clone(),
            maicie_config: PathBuf::new(),
            project_root_policy_path: None,
            bind: "127.0.0.1:0".parse().unwrap(),
            token: "test".to_string(),
        };
        let runtime = UiRelayRuntime::new(None);
        let body = serde_json::to_vec(
            &serde_json::json!({"version": UI_VERSION, "to": "rc1", "body": "ping", "reply": true}),
        )
        .unwrap();
        let error = post_ui_message(&config, &runtime, &body).unwrap_err();
        assert_eq!(error.1, "human_sender_unregistered");
        assert!(error.2.contains("présence UI humaine refusée"));
        server.join().unwrap();
        let _ = std::fs::remove_file(socket_path);
    }

    #[test]
    fn la_propre_entree_de_l_humain_montre_sa_boite_entiere() {
        // Mesure du 28/08 a 11h29 : en selectionnant sa propre entree dans la
        // colonne des agents, l'humain voyait le titre de chaque reponse mais
        // une conversation VIDE. Cause : le filtre par paire cherchait alors
        // des messages humain -> humain, qui n'existent pas. Il ne pouvait donc
        // pas lire ce qui lui etait adresse sans deviner qu'il fallait
        // selectionner l'agent. Restaurer le filtre par paire pour ce cas rend
        // zero bulle et tue ce temoin.
        let messages = vec![
            ledger_message("h1", 10, "humain", "bridget"),
            ledger_message("b1", 20, "bridget", "humain"),
            ledger_message("h2", 30, "humain", "jc6"),
            ledger_message("j1", 40, "jc6", "humain"),
            ledger_message("x1", 50, "jc2", "bridget"),
        ];

        let boite = human_referent_thread_messages(UI_SENDER, &messages);
        let vus: Vec<&str> = boite.iter().map(|m| m.delivery_id.as_str()).collect();
        assert_eq!(
            vus,
            vec!["h1", "b1", "h2", "j1"],
            "la propre entree de l'humain doit montrer tous ses echanges, tous correspondants confondus, et exclure le trafic agent-agent"
        );
        assert_eq!(boite[0].role, UiThreadRoleV1::User);
        assert_eq!(boite[1].role, UiThreadRoleV1::Agent);

        // Le filtre par paire reste intact pour un agent focal ordinaire.
        let fil_bridget = human_referent_thread_messages("bridget", &messages);
        let vus_bridget: Vec<&str> = fil_bridget.iter().map(|m| m.delivery_id.as_str()).collect();
        assert_eq!(vus_bridget, vec!["h1", "b1"]);
    }

    #[test]
    fn presence_humaine_est_rouverte_apres_la_chute_du_daemon() {
        // Mesure du 28/08 a 10h13 : au redemarrage du daemon les dix agents se
        // sont reinscrits seuls, mais pas l'humain — il n'a aucun wrapper qui
        // le reconnecte. Il a fallu relancer le relais a la main pour pouvoir
        // lui repondre. ensure_human_presence doit donc ROUVRIR une presence
        // tombee, et pas seulement en constater l'existence passee.
        let socket_path = std::env::temp_dir().join(format!(
            "bridget-ui-reouverture-{}.sock",
            uuid::Uuid::new_v4().simple()
        ));
        let listener = UnixListener::bind(&socket_path).unwrap();
        let (inscriptions, recues) = std::sync::mpsc::channel();
        let server = thread::spawn(move || {
            for tour in 0..2 {
                let (stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut writer = BufWriter::new(stream);
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let trame = decode::<WrapperToDaemon>(line.trim()).unwrap();
                inscriptions
                    .send(matches!(trame, WrapperToDaemon::Register { .. }))
                    .unwrap();
                writeln!(
                    writer,
                    "{}",
                    encode(&DaemonToWrapper::Registered {
                        agent_id: UI_SENDER.to_string(),
                    })
                    .unwrap()
                )
                .unwrap();
                writer.flush().unwrap();
                line.clear();
                reader.read_line(&mut line).unwrap();
                if tour == 0 {
                    // Chute du daemon : la socket meurt, la presence tombe.
                    drop(writer);
                    drop(reader);
                }
            }
        });

        let runtime = UiRelayRuntime::new(None);
        runtime.ensure_human_presence(&socket_path).unwrap();
        assert!(
            recues.recv_timeout(Duration::from_secs(5)).unwrap(),
            "premiere inscription attendue"
        );

        // La veille rappelle ensure_human_presence : une presence tombee doit
        // etre rouverte, ce que prouve une SECONDE trame Register.
        let mut rouverte = false;
        for _ in 0..40 {
            thread::sleep(Duration::from_millis(100));
            if runtime.ensure_human_presence(&socket_path).is_ok()
                && let Ok(vrai) = recues.try_recv()
            {
                rouverte = vrai;
                break;
            }
        }
        assert!(
            rouverte,
            "apres la chute du daemon, la presence humaine doit etre rouverte par une nouvelle inscription"
        );
        server.join().unwrap();
        let _ = std::fs::remove_file(socket_path);
    }

    #[test]
    fn presence_humaine_bat_pour_survivre_a_la_retention() {
        // Mesure du 28/08 : l'humain disparaissait de l'annuaire cinq minutes
        // apres son dernier message, socket pourtant intacte. Cause : le retain
        // du daemon s'appuie sur link_seen, rafraichi par le heartbeat, et la
        // presence UI n'en emettait aucun. Consequence directe : impossible de
        // repondre a un humain silencieux — le cas d'usage meme du referent.
        // Ce temoin exige un Heartbeat apres l'inscription. Retirer le thread
        // de battement laisse la lecture bloquer jusqu'au delai et le tue.
        let socket_path = std::env::temp_dir().join(format!(
            "bridget-ui-heartbeat-{}.sock",
            uuid::Uuid::new_v4().simple()
        ));
        let listener = UnixListener::bind(&socket_path).unwrap();
        let (battement, recu) = std::sync::mpsc::channel();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode::<WrapperToDaemon>(line.trim()).unwrap(),
                WrapperToDaemon::Register { .. }
            ));
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::Registered {
                    agent_id: UI_SENDER.to_string(),
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
            // La trame suivante doit etre le battement, et non le silence.
            line.clear();
            reader.read_line(&mut line).unwrap();
            battement
                .send(decode::<WrapperToDaemon>(line.trim()).unwrap())
                .unwrap();
        });

        let presence = open_human_presence(&socket_path, None).unwrap();
        let trame = recu
            .recv_timeout(HUMAN_PRESENCE_HEARTBEAT * 4)
            .expect("la presence humaine doit battre avant d'etre jetee par le retain");
        assert!(
            matches!(trame, WrapperToDaemon::Heartbeat),
            "battement attendu pour rafraichir link_seen ; reçu {trame:?}"
        );
        drop(presence);
        server.join().unwrap();
        let _ = std::fs::remove_file(socket_path);
    }

    #[test]
    fn presence_humaine_est_tentee_meme_sans_reponse_attendue() {
        // Le défaut mesuré le 28/08 : l'humain n'entrait à l'annuaire que
        // lorsqu'il cochait « attendre une réponse ». Décoché — son réglage
        // courant — aucune inscription, donc tout retour vers lui était rejeté
        // « agent introuvable: humain ».
        // Ce témoin exerce le chemin réel avec reply=false et exige que le
        // Register parte quand même. Remettre la condition sur reply fait
        // recevoir au serveur autre chose qu'un Register : l'assertion du
        // thread serveur tombe et le témoin meurt.
        let socket_path = std::env::temp_dir().join(format!(
            "bridget-ui-presence-sans-reply-{}.sock",
            uuid::Uuid::new_v4().simple()
        ));
        let listener = UnixListener::bind(&socket_path).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode::<WrapperToDaemon>(line.trim()).unwrap(),
                WrapperToDaemon::ListAgents
            ));
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::AgentList {
                    agents: vec![agent_info("rc1", "idle")]
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();

            // Connexion d'inscription : c'est elle qui n'existait pas.
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            line.clear();
            reader.read_line(&mut line).unwrap();
            let inscription = decode::<WrapperToDaemon>(line.trim()).unwrap();
            assert!(
                matches!(inscription, WrapperToDaemon::Register { .. }),
                "sans réponse attendue, l'inscription de l'humain doit tout de même être tentée ; reçu {inscription:?}"
            );
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::AttachRejected {
                    subscription_id: None,
                    reason: bridget_transport::protocol::AttachRefusal::JournalUnavailable,
                    mode: None,
                    location: None
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();

            // L'envoi doit être tenté malgré le refus d'inscription.
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            line.clear();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode::<WrapperToDaemon>(line.trim()).unwrap(),
                WrapperToDaemon::RoleHandshake { .. }
            ));
        });
        let config = UiRelayConfig {
            daemon_socket: socket_path.clone(),
            maicie_config: PathBuf::new(),
            project_root_policy_path: None,
            bind: "127.0.0.1:0".parse().unwrap(),
            token: "test".to_string(),
        };
        let runtime = UiRelayRuntime::new(None);
        let body = serde_json::to_vec(&serde_json::json!({"version": UI_VERSION, "to": "rc1", "body": "ping", "reply": false})).unwrap();
        let error = post_ui_message(&config, &runtime, &body).unwrap_err();
        // Sans réponse attendue, un refus d'inscription ne doit pas faire
        // perdre le message : la cause remontée est celle de l'envoi.
        assert_eq!(error.1, "send_failed");
        server.join().unwrap();
        let _ = std::fs::remove_file(socket_path);
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
            project_root_policy_path: None,
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
            project_root_policy_path: None,
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
        assert!(response.contains("/vendor/marked.min.js"), "{response}");
        assert!(response.contains("/vendor/purify.min.js"), "{response}");
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
    fn references_d_artefact_separent_les_formats_url_et_json() {
        let suffix = "e213601b-7e99-4a52-b856-42f55f79da24";
        let canonical = format!("artifact-version:{suffix}");
        assert_eq!(
            artifact_version_from_query(Some(&format!("artifact-version.{suffix}"))).unwrap(),
            canonical
        );
        assert!(artifact_version_from_query(Some(&canonical)).is_err());
        assert_eq!(artifact_version_from_request(&canonical).unwrap(), canonical);
        assert!(artifact_version_from_request(&format!("artifact-version.{suffix}")).is_err());
    }

    #[test]
    fn garde_destinataire_refuse_humain_absent_et_arrete_sans_refuser_un_agent_vivant() {
        let vivant = agent_info("vivant", "connected");
        let arrete = agent_info("arrete", "stopped");
        let agents = vec![vivant, arrete];
        assert_eq!(
            validate_ui_recipient(&agents, UI_SENDER),
            Err((400, "human_recipient"))
        );
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
    fn spec_073_contrat_stop_refuse_version_champs_et_identifiants_invalides() {
        let valid = parse_ui_stop_request(
            br#"{"version":1,"agent_id":"agent-1","command_id":"stop-ui-123"}"#,
        )
        .unwrap();
        assert_eq!(valid.agent_id, "agent-1");
        assert_eq!(valid.command_id, "stop-ui-123");

        for body in [
            br#"{"version":2,"agent_id":"agent-1","command_id":"stop-ui-123"}"#.as_slice(),
            br#"{"version":1,"agent_id":"agent/1","command_id":"stop-ui-123"}"#.as_slice(),
            br#"{"version":1,"agent_id":"agent-1","command_id":""}"#.as_slice(),
            br#"{"version":1,"agent_id":"agent-1","command_id":"stop ui","extra":true}"#.as_slice(),
        ] {
            assert_eq!(
                parse_ui_stop_request(body).unwrap_err().1,
                "invalid_request"
            );
        }
    }

    #[test]
    fn spec_073_garde_stop_exige_gestion_attestee_et_agent_actif() {
        let mut managed = agent_info("managed", "connected");
        managed.persistent = Some(false);
        let external = agent_info("external", "connected");
        let mut stopped = agent_info("stopped", "stopped");
        stopped.persistent = Some(true);
        let agents = vec![managed, external, stopped];

        assert_eq!(validate_ui_stop_target(&agents, "managed"), Ok(()));
        assert_eq!(
            validate_ui_stop_target(&agents, "external"),
            Err((409, "agent_not_managed"))
        );
        assert_eq!(
            validate_ui_stop_target(&agents, "stopped"),
            Err((409, "agent_stopped"))
        );
        assert_eq!(
            validate_ui_stop_target(&agents, "absent"),
            Err((404, "agent_not_found"))
        );
    }

    #[test]
    fn spec_073_mapping_stop_conserve_tous_les_verdicts() {
        let request = UiStopRequestV1 {
            version: UI_VERSION,
            agent_id: "managed".to_string(),
            command_id: "stop-ui-123".to_string(),
        };
        let clean =
            map_ui_stop_outcome(&request, bridget_transport::protocol::StopOutcome::Stopped)
                .unwrap();
        assert_eq!(clean.outcome, "stopped");
        assert_eq!(clean.survivors_killed, None);
        let forced = map_ui_stop_outcome(
            &request,
            bridget_transport::protocol::StopOutcome::StoppedForced {
                survivors_killed: 2,
            },
        )
        .unwrap();
        assert_eq!(forced.outcome, "stopped_forced");
        assert_eq!(forced.survivors_killed, Some(2));
        for (outcome, status, code) in [
            (
                bridget_transport::protocol::StopOutcome::NotManaged,
                409,
                "agent_not_managed",
            ),
            (
                bridget_transport::protocol::StopOutcome::NotFound,
                404,
                "agent_not_found",
            ),
            (
                bridget_transport::protocol::StopOutcome::Timeout {
                    state: "encore vivant".to_string(),
                },
                504,
                "stop_timeout",
            ),
        ] {
            let error = map_ui_stop_outcome(&request, outcome).unwrap_err();
            assert_eq!((error.0, error.1), (status, code));
        }
    }

    #[test]
    fn spec_075_gardes_relaunch_et_decommission_suivent_l_etat_durable() {
        let mut active = agent_info("active", "connected");
        active.persistent = Some(true);
        let mut stopped = agent_info("stopped", "stopped");
        stopped.persistent = Some(false);
        let external = agent_info("external", "stopped");
        let mut recovering = agent_info("recovering", "recovering");
        recovering.persistent = Some(true);
        let agents = vec![active, stopped, external, recovering];

        assert_eq!(
            validate_ui_relaunch_target(&agents, "active"),
            Err((409, "agent_already_running"))
        );
        assert_eq!(validate_ui_relaunch_target(&agents, "stopped"), Ok(()));
        assert_eq!(
            validate_ui_relaunch_target(&agents, "external"),
            Err((409, "agent_not_managed"))
        );
        assert_eq!(validate_ui_decommission_target(&agents, "active"), Ok(()));
        assert_eq!(validate_ui_decommission_target(&agents, "stopped"), Ok(()));
        for guard in [
            validate_ui_stop_target(&agents, "recovering"),
            validate_ui_relaunch_target(&agents, "recovering"),
            validate_ui_decommission_target(&agents, "recovering"),
        ] {
            assert_eq!(guard, Err((409, "lifecycle_in_progress")));
        }
    }

    #[test]
    fn spec_075_mapping_relaunch_et_decommission_reste_ferme() {
        let request = UiStopRequestV1 {
            version: UI_VERSION,
            agent_id: "managed".to_string(),
            command_id: "lifecycle-ui-123".to_string(),
        };
        let relaunched = map_ui_relaunch_outcome(
            &request,
            bridget_transport::protocol::RelaunchOutcome::Started {
                agent_id: "managed".to_string(),
                generation: 9,
            },
        )
        .unwrap();
        assert_eq!(relaunched.outcome, "started");
        assert_eq!(relaunched.generation, Some(9));
        assert_eq!(
            map_ui_relaunch_outcome(
                &request,
                bridget_transport::protocol::RelaunchOutcome::AlreadyRunning,
            )
            .unwrap_err()
            .1,
            "agent_already_running"
        );

        let decommissioned = map_ui_decommission_outcome(
            &request,
            bridget_transport::protocol::DecommissionOutcome::DecommissionedForced {
                survivors_killed: 2,
            },
        )
        .unwrap();
        assert_eq!(decommissioned.outcome, "decommissioned_forced");
        assert_eq!(decommissioned.survivors_killed, Some(2));
        assert_eq!(
            map_ui_decommission_outcome(
                &request,
                bridget_transport::protocol::DecommissionOutcome::Timeout {
                    state: "vivant".to_string(),
                },
            )
            .unwrap_err()
            .1,
            "lifecycle_timeout"
        );
    }

    #[test]
    fn spec_073_relais_stop_transmet_un_seul_stop_order() {
        let agent_id = "550e8400-e29b-41d4-a716-446655440010";
        let socket_path = std::env::temp_dir().join(format!(
            "bridget-ui-stop-{}.sock",
            uuid::Uuid::new_v4().simple()
        ));
        let listener = UnixListener::bind(&socket_path).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode::<WrapperToDaemon>(line.trim()).unwrap(),
                WrapperToDaemon::ListAgents
            ));
            let mut managed = agent_info(agent_id, "connected");
            managed.persistent = Some(false);
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::AgentList {
                    agents: vec![managed]
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();

            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            line.clear();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode::<WrapperToDaemon>(line.trim()).unwrap(),
                WrapperToDaemon::StopOrder { ref agent_id, ref command_id }
                    if agent_id == "550e8400-e29b-41d4-a716-446655440010" && command_id == "stop-ui-123"
            ));
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::StopResult {
                    command_id: "stop-ui-123".to_string(),
                    outcome: bridget_transport::protocol::StopOutcome::Stopped,
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();
        });
        let config = UiRelayConfig {
            daemon_socket: socket_path.clone(),
            maicie_config: PathBuf::new(),
            project_root_policy_path: None,
            bind: "127.0.0.1:0".parse().unwrap(),
            token: "test".to_string(),
        };
        let response = post_ui_stop(
            &config,
            br#"{"version":1,"agent_id":"550e8400-e29b-41d4-a716-446655440010","command_id":"stop-ui-123"}"#,
        )
        .unwrap();
        assert_eq!(response.outcome, "stopped");
        server.join().unwrap();
        let _ = std::fs::remove_file(socket_path);
    }

    #[test]
    fn spec_073_relais_refuse_un_agent_non_gere_avant_stop_order() {
        let agent_id = "550e8400-e29b-41d4-a716-446655440011";
        let socket_path = std::env::temp_dir().join(format!(
            "bridget-ui-stop-refus-{}.sock",
            uuid::Uuid::new_v4().simple()
        ));
        let listener = UnixListener::bind(&socket_path).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode::<WrapperToDaemon>(line.trim()).unwrap(),
                WrapperToDaemon::ListAgents
            ));
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::AgentList {
                    agents: vec![agent_info(agent_id, "connected")]
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();
        });
        let config = UiRelayConfig {
            daemon_socket: socket_path.clone(),
            maicie_config: PathBuf::new(),
            project_root_policy_path: None,
            bind: "127.0.0.1:0".parse().unwrap(),
            token: "test".to_string(),
        };
        let error = post_ui_stop(
            &config,
            br#"{"version":1,"agent_id":"550e8400-e29b-41d4-a716-446655440011","command_id":"stop-ui-123"}"#,
        )
        .unwrap_err();
        assert_eq!((error.0, error.1), (409, "agent_not_managed"));
        server.join().unwrap();
        let _ = std::fs::remove_file(socket_path);
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
    fn ronde_du_registre_porte_constats_et_corps_pour_la_page_ui() {
        let message = LedgerMessage {
            id: "ronde-1".to_string(),
            ts: 100,
            sender: "cli-send-1239134".to_string(),
            target: "bridget".to_string(),
            body: [
                "RONDE DE VIGILANCE (7 min) \u{2014} c est ton tour maintenant.",
                "",
                "--- SIGNAL MECANIQUE DE LA RONDE ---",
                "LOT SANS RECLAMANT : aucun volontaire.",
                "FICHIERS DISPUTES : install_publish.rs (7).",
            ]
            .join("\n"),
            delivery_status: None,
        };
        let exchanges = aggregate_peer_exchanges("bridget", &[message]);
        let round = exchanges[0]
            .vigilance_round
            .as_ref()
            .expect("une ronde du registre doit porter sa projection UI");
        assert_eq!(round.interval, "7 min");
        assert_eq!(round.headline, "c est ton tour maintenant.");
        assert_eq!(
            round.signal,
            "LOT SANS RECLAMANT : aucun volontaire. FICHIERS DISPUTES : install_publish.rs (7)."
        );
        assert!(round.body.starts_with("RONDE DE VIGILANCE"));
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
            project_root_policy_path: None,
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
        assert!(message.contains("aucun repli"), "{message}");
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
        assert_eq!(
            stale_mode, 0o666,
            "précondition: temporaire ouvert {stale_mode:#o}"
        );
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
        assert!(
            !tmp.exists(),
            "le temporaire ne doit plus rester après rename"
        );
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
        // Le snapshot doit passer par load_human_referent_thread (filtre-avant-borne),
        // pas par les 200 messages globaux de peer_exchanges.
        let source = include_str!("ui.rs");
        let read_body = function_body(source, "fn read_snapshot(");
        assert!(
            read_body.contains("load_human_referent_thread("),
            "read_snapshot doit appeler load_human_referent_thread (filtre avant borne)"
        );
        assert!(
            read_body.contains("thread_messages"),
            "le snapshot doit exposer thread_messages au client"
        );
        let load_body = function_body(source, "fn load_human_referent_thread(");
        assert!(
            load_body.contains("conversation_messages("),
            "load_human_referent_thread doit lire via conversation_messages"
        );
        assert!(
            load_body.contains("human_referent_thread_messages("),
            "load_human_referent_thread doit projeter via human_referent_thread_messages"
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
        let live_body = function_body(source, "fn push_live_thread_messages(");
        assert!(
            live_body.contains("load_human_referent_thread("),
            "le chemin vivant doit aussi filtrer avant de borner"
        );
        let snapshot_body = function_body(source, "fn push_live_snapshot(");
        assert!(
            snapshot_body.contains("read_snapshot(config, Some(focus_agent))"),
            "le rafraîchissement vivant doit relire la projection attestée"
        );
        assert!(
            watch_body
                .contains("push_live_snapshot(http, config, agent, &mut last_artifact_signature)?"),
            "le watch doit pousser les références d'artefacts publiées en direct"
        );
    }

    #[test]
    fn snapshot_vivant_nest_pousse_que_si_les_artefacts_changent() {
        assert!(should_push_live_snapshot(None, "a"));
        assert!(!should_push_live_snapshot(Some("a"), "a"));
        assert!(should_push_live_snapshot(Some("a"), "b"));
    }

    #[test]
    fn watch_sans_journal_reste_un_flux_vivant() {
        let source = include_str!("ui.rs");
        let serve_body = function_body(source, "fn serve_connection(");
        assert!(
            serve_body.contains("stream_sse_thread_watch(stream, config, agent)"),
            "un interlocuteur sans journal ne doit pas tenter Attach"
        );
        let header_body = function_body(source, "fn stream_sse_thread_watch(");
        assert!(
            header_body.contains("stream_sse_thread_watch_after_headers(http, config, agent)"),
            "le watch humain doit rejoindre le flux vivant après ses en-têtes SSE"
        );
        let watch_body = function_body(source, "fn stream_sse_thread_watch_after_headers(");
        assert!(
            watch_body.contains("push_live_thread_messages("),
            "le watch humain doit diffuser les nouveaux messages"
        );
        assert!(
            watch_body.contains(": keepalive\\n\\n"),
            "le watch humain doit rester ouvert entre deux messages"
        );
    }

    #[test]
    fn watch_agent_arrete_preserve_la_connexion_au_relais() {
        let source = include_str!("ui.rs");
        let journal_body = function_body(source, "fn stream_sse_journal(");
        assert!(
            journal_body.contains("focused_agent_is_stopped(config, agent)?"),
            "un arrêt d'agent doit être distingué d'une perte du relais"
        );
        assert!(
            journal_body
                .matches("stream_sse_thread_watch_after_headers(http, config, agent)")
                .count()
                >= 2,
            "le refus d'Attach et End doivent tous deux garder le flux vivant"
        );
        let fallback_body = function_body(source, "fn stream_sse_thread_watch_after_headers(");
        assert!(
            fallback_body.contains("write_relay_state(http, \"connected\", now_secs())"),
            "le flux passif doit confirmer que le relais reste joignable"
        );
        assert!(
            fallback_body.contains(": keepalive\\n\\n"),
            "le flux passif doit rester ouvert"
        );
    }

    /// Témoin du défaut mesuré : des échanges humains noyés sous >200 messages
    /// agent↔agent plus récents. Meurt si on borne le ledger global puis filtre.
    #[allow(non_snake_case)]
    #[test]
    fn TEMOIN_fil_humain_survit_au_trafic_agent() {
        let dir = std::env::temp_dir().join(format!(
            "bridget-fil-volume-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("bridget.db");
        let socket_path = dir.join("bridget.sock");
        // ledger_db_path_for_socket = socket.with_extension("db") → bridget.db
        let store = crate::store::Store::open(&db_path).unwrap();
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        // Quelques échanges humains, ANCIENS.
        conn.execute_batch(
            "INSERT INTO ledger (id, ts, sender, target, body, conversation_key) VALUES
              ('h1', 10, 'humain', 'bridget', 'salut referent', 'humain|bridget'),
              ('b1', 20, 'bridget', 'humain', 'bonjour utilisateur', 'bridget|humain'),
              ('h2', 30, 'humain', 'bridget', 'tu as vu mon fil', 'humain|bridget');",
        )
        .unwrap();
        // 300 messages agent↔agent PLUS RÉCENTS — saturent une borne globale 200.
        let mut insert = conn
            .prepare(
                "INSERT INTO ledger (id, ts, sender, target, body, conversation_key)
                 VALUES (?1, ?2, 'rc1', 'rc2', 'trafic', 'rc1|rc2')",
            )
            .unwrap();
        for i in 0..300 {
            insert
                .execute(rusqlite::params![format!("pair-{i}"), 100 + i])
                .unwrap();
        }
        drop(insert);
        drop(conn);
        drop(store);

        // Mutant documenté : borne globale 200 puis filtre → fil vide.
        let store = crate::store::Store::open(&db_path).unwrap();
        let global = store.recent_messages(200).unwrap();
        let mutant = human_referent_thread_messages(
            "bridget",
            &global
                .iter()
                .map(|entry| LedgerMessage {
                    id: entry.id.clone(),
                    ts: entry.ts,
                    sender: entry.sender.clone(),
                    target: entry.target.clone(),
                    body: entry.body.clone(),
                    delivery_status: None,
                })
                .collect::<Vec<_>>(),
        );
        assert!(
            mutant.is_empty(),
            "précondition du défaut : borne globale puis filtre vide le fil"
        );

        // Chemin productif : filtre-avant-borne.
        let messages = load_human_referent_thread(&socket_path, "bridget");
        assert_eq!(
            messages.len(),
            3,
            "le fil doit survivre au trafic agent: {:?}",
            messages
                .iter()
                .map(|m| m.delivery_id.as_str())
                .collect::<Vec<_>>()
        );
        assert_eq!(messages[0].delivery_id, "h1");
        assert_eq!(messages[0].role, UiThreadRoleV1::User);
        assert_eq!(messages[1].delivery_id, "b1");
        assert_eq!(messages[1].role, UiThreadRoleV1::Agent);
        assert_eq!(messages[2].delivery_id, "h2");
        assert_eq!(messages[2].text, "tu as vu mon fil");

        let _ = std::fs::remove_dir_all(&dir);
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
        assert_eq!(UI_JOURNAL_OPEN_TURNS, 10);
        assert_eq!(UI_JOURNAL_OLDER_PAGE_TURNS, 20);
        assert_eq!(UI_JOURNAL_MAX_REPLAY_FRAGMENTS, 500);
        let stream_body = function_body(source, "fn stream_sse_journal(");
        assert!(
            stream_body.contains("UI_JOURNAL_MAX_REPLAY_FRAGMENTS"),
            "le rejeu SSE doit borner les fragments"
        );
        assert!(
            stream_body.contains("\"journal_page\""),
            "has_more doit être émis sur le chemin SSE réel"
        );
        let resolve_src = function_body(source, "fn projected_turns(");
        assert!(
            resolve_src.contains("message_id"),
            "le tour projeté doit suivre message_id comme la page"
        );
    }

    /// Remontée : 20 tours sous la fenêtre d'ouverture, sans couper un message_id.
    #[test]
    fn remontee_from_seq_donne_l_ancien_par_pages_de_vingt() {
        let turns: Vec<ProjectedTurn> = (1..=100)
            .map(|n| ProjectedTurn {
                first_seq: n * 10,
                last_seq: n * 10 + 2,
                record_count: 3,
            })
            .collect();
        let (open_from, has_more) = select_recent_turns(
            &turns,
            UI_JOURNAL_OPEN_TURNS,
            UI_JOURNAL_MAX_REPLAY_FRAGMENTS,
        );
        assert_eq!(open_from, 910);
        assert!(has_more);
        let (older, older_more) = older_page_from_turns(
            &turns,
            open_from,
            UI_JOURNAL_OLDER_PAGE_TURNS,
            UI_JOURNAL_MAX_REPLAY_FRAGMENTS,
        );
        assert_eq!(older, 710);
        assert!(older < open_from);
        assert!(older_more);
        assert_eq!(UI_JOURNAL_OLDER_PAGE_TURNS, 20);
    }

    #[test]
    fn ouverture_sans_from_seq_prend_dix_tours_projetes() {
        let root = PathBuf::from(format!(
            "/tmp/bridget-ui-journal-turns-{}",
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
        let mut seq = 1_u64;
        for turn in 1..=15 {
            // deux records par tour (même message_id) — ne pas couper au milieu
            for event in ["turn_start", "turn_end"] {
                lines.push_str(&format!(
                    "{{\"v\":1,\"seq\":{seq},\"message_id\":\"m{turn}\",\"event\":\"{event}\",\"session_id\":\"s\"}}\n"
                ));
                seq += 1;
            }
        }
        std::fs::write(root.join("2026-08-26.jsonl"), lines).unwrap();
        let (window, page) = resolve_ui_journal_window_in(root.clone(), &HashMap::new()).unwrap();
        // 15 tours × 2 = 30 seqs ; 10 derniers tours → from_seq = seq du turn 6 start = 11
        assert_eq!(window, AttachWindow::Seq(11));
        assert!(page.has_more);
        assert_eq!(page.from_seq, 11);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn plafond_ne_coupe_pas_un_tour_en_plein_milieu() {
        // Un tour de 600 records puis 9 petits : sous plafond 500 on prend
        // seulement le tour récent entier (600), has_more vrai.
        let mut turns = Vec::new();
        for n in 1..=9 {
            turns.push(ProjectedTurn {
                first_seq: n,
                last_seq: n,
                record_count: 1,
            });
        }
        turns.push(ProjectedTurn {
            first_seq: 100,
            last_seq: 699,
            record_count: 600,
        });
        let (from_seq, has_more) = select_recent_turns(&turns, 10, 500);
        assert_eq!(from_seq, 100);
        assert!(has_more);
        assert_ne!(from_seq, 400, "ne doit pas démarrer au milieu du gros tour");
    }

    /// Garde anti-feuille : retirer l'appel à `retain_living_objectives` dans
    /// `read_snapshot` (tout en laissant la fonction intacte) doit tuer ce témoin.
    #[test]
    fn chemin_productif_snapshot_emprunte_retain_living_objectives() {
        let source = include_str!("ui.rs");
        let read_body = function_body(source, "fn read_snapshot(");
        assert!(
            read_body.contains("retain_living_objectives"),
            "read_snapshot doit appeler retain_living_objectives sinon le megaoctet clos revient et page se fige"
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
            project_root_policy_path: None,
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
        assert_eq!(hit_value["hits"][0]["sender"], "Agent");
        assert_eq!(hit_value["hits"][0]["ts"], 100);
        assert_eq!(hit_value["hits"][0]["body"], "alpha cible premiere");
        assert_eq!(hit_value["hits"][1]["id"], "hit-late");
        assert_eq!(hit_value["hits"][1]["sender"], "Agent (2)");
        assert_eq!(hit_value["hits"][1]["ts"], 200);
        assert_eq!(
            hit_value["hits"][1]["body"],
            "seconde cible avec <tag> et 100%_wild"
        );
        assert_eq!(hit_value["truncated"], false);
        assert!(!hit_raw.contains("\"sender\":\"bridget\""));
        assert!(!hit_raw.contains("\"sender\":\"cursor4\""));
        assert!(
            hit_value["hits"][0]["ts"].as_i64().unwrap()
                < hit_value["hits"][1]["ts"].as_i64().unwrap()
        );

        let config = UiRelayConfig {
            daemon_socket: root.join("bridget.sock"),
            maicie_config: root.join("maicie.json"),
            project_root_policy_path: None,
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            token: "jeton-search".to_string(),
        };
        let relay = UiRelay::bind(config).unwrap();
        let address = relay.local_addr().unwrap();
        let worker = thread::spawn(move || relay.serve_one().unwrap());
        let (miss_status, miss_raw) = post_search(
            address,
            "jeton-search",
            r#"{"version":1,"q":"motabsentxyz"}"#,
        );
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
            project_root_policy_path: None,
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

    fn get_asset(address: SocketAddr, path: &str, if_none_match: Option<&str>) -> (u16, String) {
        let mut client = TcpStream::connect(address).unwrap();
        let extra = match if_none_match {
            Some(etag) => format!("If-None-Match: {etag}\r\n"),
            None => String::new(),
        };
        let request =
            format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n{extra}Connection: close\r\n\r\n");
        client.write_all(request.as_bytes()).unwrap();
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

    fn header_value(raw: &str, name: &str) -> Option<String> {
        let name = name.to_ascii_lowercase();
        raw.split("\r\n\r\n")
            .next()?
            .lines()
            .skip(1)
            .find_map(|line| {
                let (key, value) = line.split_once(':')?;
                if key.trim().eq_ignore_ascii_case(&name) {
                    Some(value.trim().to_string())
                } else {
                    None
                }
            })
    }

    fn spawn_asset_relay() -> (UiRelay, SocketAddr) {
        let config = UiRelayConfig {
            daemon_socket: PathBuf::from("/tmp/ui-cache-ne-doit-pas-ouvrir.sock"),
            maicie_config: PathBuf::from("/tmp/ui-cache-ne-doit-pas-ouvrir.json"),
            project_root_policy_path: None,
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            token: "jeton-cache".to_string(),
        };
        let relay = UiRelay::bind(config).unwrap();
        let address = relay.local_addr().unwrap();
        (relay, address)
    }

    /// Le panneau latéral est une surface distincte : il doit servir l'UI sans
    /// retomber sur une conversation classique lorsque WebKit recharge sa page.
    #[test]
    fn route_browser_panel_sert_l_index_de_l_interface() {
        let (relay, address) = spawn_asset_relay();
        let worker = thread::spawn(move || relay.serve_one().unwrap());
        let (status, raw) = get_asset(address, "/browser-panel?token=jeton-cache", None);
        worker.join().unwrap();

        assert_eq!(status, 200, "{raw}");
        let body = raw.split("\r\n\r\n").nth(1).unwrap_or("");
        assert!(
            body.contains("browser-panel-toggle"),
            "la route doit livrer l'interface qui initialise le panneau navigateur"
        );
    }

    /// Témoin « en-têtes absents » : un mutant qui retire ETag ou no-cache meurt ici.
    #[test]
    fn assets_statiques_annoncent_etag_et_revalidation() {
        let (relay, address) = spawn_asset_relay();
        let worker = thread::spawn(move || relay.serve_one().unwrap());
        let (status, raw) = get_asset(address, "/app.js", None);
        worker.join().unwrap();
        assert_eq!(status, 200, "{raw}");
        let etag = header_value(&raw, "ETag").expect("ETag obligatoire sur asset statique");
        assert!(
            etag.starts_with('"') && etag.ends_with('"'),
            "ETag fort: {etag}"
        );
        let cache = header_value(&raw, "Cache-Control").expect("Cache-Control obligatoire");
        assert!(
            cache.split(',').any(|d| d.trim() == "no-cache"),
            "fraîcheur d'abord — no-cache requis, reçu {cache}"
        );
        assert!(
            !cache.contains("max-age=31536000") && !cache.contains("immutable"),
            "cache agressif interdit (livraison figée), reçu {cache}"
        );
        assert_eq!(etag, asset_etag(UI_SCRIPT));
    }

    /// Témoin « inchangé → 304 » : second GET avec If-None-Match ne renvoie pas le corps.
    #[test]
    fn etag_inchange_rend_304_sans_corps() {
        let (relay, address) = spawn_asset_relay();
        let worker = thread::spawn(move || {
            relay.serve_one().unwrap();
            relay.serve_one().unwrap();
        });
        let (first_status, first) = get_asset(address, "/theme.css", None);
        assert_eq!(first_status, 200, "{first}");
        let etag = header_value(&first, "ETag").expect("ETag sur premier GET");
        let (second_status, second) = get_asset(address, "/theme.css", Some(&etag));
        worker.join().unwrap();
        assert_eq!(second_status, 304, "{second}");
        assert!(
            header_value(&second, "ETag").as_deref() == Some(etag.as_str()),
            "304 doit rappeler l'ETag, reçu {second}"
        );
        assert!(
            header_value(&second, "Cache-Control")
                .as_deref()
                .is_some_and(|c| c.split(',').any(|d| d.trim() == "no-cache")),
            "304 garde no-cache, reçu {second}"
        );
        let body = second.split("\r\n\r\n").nth(1).unwrap_or("x");
        assert!(
            body.is_empty(),
            "304 ne doit pas renvoyer le CSS, corps={body:?}"
        );
    }

    /// Témoin « modifié servi comme inchangé » : mauvais ETag → 200 corps complet.
    /// Un mutant qui répond 304 dès qu'If-None-Match est présent meurt ici.
    #[test]
    fn contenu_modifie_refuse_le_304() {
        assert_ne!(
            asset_etag(b"v1"),
            asset_etag(b"v2"),
            "ETag doit suivre le contenu — sinon livraison figée après correctif"
        );
        let (relay, address) = spawn_asset_relay();
        let worker = thread::spawn(move || relay.serve_one().unwrap());
        let stale = "\"0000000000000000000000000000000000000000000000000000000000000000\"";
        let (status, raw) = get_asset(address, "/vendor/marked.min.js", Some(stale));
        worker.join().unwrap();
        assert_eq!(
            status, 200,
            "ETag périmé doit forcer le téléchargement: {raw}"
        );
        let body = raw.split("\r\n\r\n").nth(1).unwrap_or("");
        assert_eq!(
            body.as_bytes(),
            UI_MARKED,
            "corps 200 doit être le fichier actuel"
        );
        assert_eq!(
            header_value(&raw, "ETag").as_deref(),
            Some(asset_etag(UI_MARKED).as_str())
        );
    }

    /// Anti-feuille : les routes productives doivent passer par write_asset (ETag).
    #[test]
    fn chemin_productif_assets_emprunte_write_asset() {
        let source = include_str!("ui.rs");
        let serve_body = function_body(source, "fn serve_connection(");
        for path in [
            "\"/app.js\"",
            "\"/theme.css\"",
            "\"/vendor/marked.min.js\"",
            "\"/vendor/purify.min.js\"",
            "\"/providers/openai.svg\"",
            "\"/providers/claude.svg\"",
            "\"/providers/cursor.svg\"",
            "\"/providers/gemini.svg\"",
            "\"/providers/glm.svg\"",
            "\"/providers/deepseek.svg\"",
        ] {
            assert!(
                serve_body.contains(path),
                "route {path} absente de serve_connection"
            );
        }
        let write_body = function_body(source, "fn write_asset(");
        assert!(
            write_body.contains("Cache-Control: no-cache"),
            "write_asset doit annoncer no-cache"
        );
        assert!(
            write_body.contains("ETag:"),
            "write_asset doit annoncer ETag"
        );
        assert!(
            write_body.contains("304"),
            "write_asset doit savoir répondre 304"
        );
    }

    #[test]
    fn projection_ui_distingue_connexion_vitalite_tour_attente_file_et_propriete() {
        let mut active = agent_info("active", "busy");
        active.last_seen_secs = 5;
        active.transport = "codex_app_server".to_string();
        active.mode = Some(PresenceMode::Cli);
        active.model = Some("gpt-5.6-terra".to_string());
        active.effort = Some("xhigh".to_string());
        active.execution = Some(bridget_transport::protocol::ExecutionUiProjection {
            state: Some("running".to_string()),
            wait_state: None,
            progress_age_secs: Some(3),
            queue_depth: 2,
            continuation_mode: Some("reconstructed".to_string()),
        });
        active.agent_link = Some(bridget_transport::protocol::AgentLinkUiProjection {
            link_id: "link-active".to_string(),
            parent_instance_id: "parent-instance".to_string(),
            parent_execution_id: Some("execution-parent".to_string()),
            objective_id: Some("objective-1".to_string()),
            delegation_id: Some("delegation-1".to_string()),
            project: None,
            role: "verification".to_string(),
            agent_path: "parent-instance/active-instance".to_string(),
            state: "open".to_string(),
            direct_descendants: 1,
            descendants: 2,
        });
        let mut waiting = agent_info("waiting", "connected");
        waiting.execution = Some(bridget_transport::protocol::ExecutionUiProjection {
            state: Some("waiting_approval".to_string()),
            wait_state: Some("waiting_approval".to_string()),
            progress_age_secs: Some(9),
            continuation_mode: None,
            queue_depth: 0,
        });
        let rows = compose_agent_rows(vec![active, waiting], &[], &HashMap::new());
        let active = rows.iter().find(|row| row.agent_id == "active").unwrap();
        assert_eq!(active.connection_state, "busy");
        assert_eq!(active.provider_age_secs, 5);
        assert_eq!(active.transport, "codex_app_server");
        assert_eq!(active.mode, Some(PresenceMode::Cli));
        assert_eq!(active.model.as_deref(), Some("gpt-5.6-terra"));
        assert_eq!(active.effort.as_deref(), Some("xhigh"));
        assert_eq!(active.turn_state.as_deref(), Some("running"));
        assert_eq!(active.continuation_mode.as_deref(), Some("reconstructed"));
        assert_eq!(active.progress_age_secs, Some(3));
        assert_eq!(active.queue_depth, 2);
        assert!(matches!(
            active.agent_link.as_ref(),
            Some(link) if link.parent_instance_id == "parent-instance"
                && link.objective_id.as_deref() == Some("objective-1")
                && link.direct_descendants == 1
                && link.descendants == 2
        ));
        let waiting = rows.iter().find(|row| row.agent_id == "waiting").unwrap();
        assert_eq!(waiting.connection_state, "alive");
        assert_eq!(waiting.turn_state.as_deref(), Some("waiting_approval"));
        assert_eq!(waiting.wait_state.as_deref(), Some("waiting_approval"));
        assert_eq!(waiting.transport, "unix");
        assert_eq!(waiting.mode, None);
        assert_eq!(waiting.model, None);
        assert_eq!(waiting.effort, None);
    }

    #[test]
    fn spec_073_projection_ui_conserve_les_trois_etats_de_gestion() {
        let mut persistent = agent_info("persistent", "connected");
        persistent.persistent = Some(true);
        let mut ephemeral = agent_info("ephemeral", "connected");
        ephemeral.persistent = Some(false);
        let external = agent_info("external", "connected");

        let rows = compose_agent_rows(vec![persistent, ephemeral, external], &[], &HashMap::new());

        assert_eq!(rows[0].persistent, Some(true));
        assert_eq!(rows[1].persistent, Some(false));
        assert_eq!(rows[2].persistent, None);
    }

    #[test]
    fn spec_076_reglages_racines_sont_versions_et_atomiques() {
        use std::os::unix::fs::PermissionsExt;

        let root = std::env::temp_dir().join(format!(
            "bridget-spec-076-ui-settings-{}",
            uuid::Uuid::new_v4().simple()
        ));
        let projects = root.join("projects");
        std::fs::create_dir_all(&projects).unwrap();
        let policy_path = root.join("project-root-policy.json");
        std::fs::write(
            &policy_path,
            serde_json::to_vec(&serde_json::json!({
                "contract_version": 1,
                "policy_generation": 1,
                "allowed_project_roots": [projects],
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::set_permissions(&policy_path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let config = UiRelayConfig {
            daemon_socket: root.join("bridget.sock"),
            maicie_config: root.join("maicie.json"),
            project_root_policy_path: Some(policy_path.clone()),
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            token: "test".to_string(),
        };

        let initial = read_project_settings(&config).unwrap();
        assert_eq!(initial.policy_generation, 1);
        let request = serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "command_id": "settings-1",
            "expected_generation": 1,
            "allowed_project_roots": initial.allowed_project_roots,
        }))
        .unwrap();
        let updated = post_project_roots_update(&config, &request).unwrap();
        assert_eq!(updated.policy_generation, 2);
        assert!(post_project_roots_update(&config, &request).is_err());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn spec_080_controle_serveur_previsualise_et_rejoue_sans_double_ecriture() {
        use std::os::unix::fs::PermissionsExt;

        let root = std::env::temp_dir().join(format!(
            "bridget-spec-080-server-control-{}",
            uuid::Uuid::new_v4().simple()
        ));
        let projects = root.join("projects");
        std::fs::create_dir_all(&projects).unwrap();
        let policy_path = root.join("project-root-policy.json");
        std::fs::write(
            &policy_path,
            serde_json::to_vec(&serde_json::json!({
                "contract_version": 1,
                "policy_generation": 1,
                "allowed_project_roots": [projects],
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::set_permissions(&policy_path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let config = UiRelayConfig {
            daemon_socket: root.join("bridget.sock"),
            maicie_config: root.join("maicie.json"),
            project_root_policy_path: Some(policy_path.clone()),
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            token: "test".to_string(),
        };

        let settings = read_server_control_settings(&config).unwrap();
        assert!(settings.configuration_available);
        assert!(settings.categories.iter().any(|category| {
            category.key == "project_roots.allowed_roots" && category.access == "writable"
        }));
        assert!(settings.categories.iter().all(|category| {
            category.key == "project_roots.allowed_roots" || category.access == "read_only"
        }));

        let request = serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "command_id": "server-control-1",
            "expected_generation": 1,
            "allowed_project_roots": settings.allowed_project_roots,
        }))
        .unwrap();
        let preview = post_control_project_roots_preview(&config, &request).unwrap();
        assert_eq!(preview.resulting_generation, 2);
        assert_eq!(
            ProjectRootPolicy::load(&policy_path).unwrap().generation(),
            1
        );

        let applied = post_control_project_roots_apply(&config, &request).unwrap();
        assert_eq!(applied.resulting_generation, 2);
        let replay = post_control_project_roots_apply(&config, &request).unwrap();
        assert_eq!(replay.resulting_generation, 2);
        assert_eq!(
            ProjectRootPolicy::load(&policy_path).unwrap().generation(),
            2
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn spec_081_apercu_fichier_reste_borne_canonique_et_sans_chemin_racine() {
        use std::os::unix::fs::PermissionsExt;

        let root = std::env::temp_dir().join(format!(
            "bridget-spec-081-file-preview-{}",
            uuid::Uuid::new_v4().simple()
        ));
        let projects = root.join("projects");
        let source = projects.join("src");
        std::fs::create_dir_all(&source).unwrap();
        let file = source.join("main.rs");
        std::fs::write(&file, "fn main() {}\n").unwrap();
        let oversized = source.join("too-large.txt");
        std::fs::write(
            &oversized,
            vec![b'x'; MAX_UI_FILE_PREVIEW_BYTES as usize + 1],
        )
        .unwrap();
        let outside = root.join("outside.txt");
        std::fs::write(&outside, "hors projet").unwrap();
        let policy_path = root.join("project-root-policy.json");
        std::fs::write(
            &policy_path,
            serde_json::to_vec(&serde_json::json!({
                "contract_version": 1,
                "policy_generation": 1,
                "allowed_project_roots": [projects],
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::set_permissions(&policy_path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let config = UiRelayConfig {
            daemon_socket: root.join("bridget.sock"),
            maicie_config: root.join("maicie.json"),
            project_root_policy_path: Some(policy_path),
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            token: "test".to_string(),
        };
        let preview = read_content_file_preview(&config, file.to_str()).unwrap();
        assert_eq!(preview.path, "src/main.rs");
        assert_eq!(preview.media_type, "text/rust");
        assert_eq!(preview.encoding, "utf8");
        assert_eq!(preview.content, "fn main() {}\n");
        assert!(!preview.content.contains(root.to_string_lossy().as_ref()));
        assert!(preview.sha256.len() == 64);
        assert_eq!(
            read_content_file_preview(&config, Some("relative.txt"))
                .unwrap_err()
                .1,
            "invalid_path"
        );
        assert_eq!(
            read_content_file_preview(&config, outside.to_str())
                .unwrap_err()
                .1,
            "outside_project_roots"
        );
        assert_eq!(
            read_content_file_preview(&config, oversized.to_str())
                .unwrap_err()
                .1,
            "too_large"
        );
        assert_eq!(base64_standard(&[0x66, 0x6f, 0x6f]), "Zm9v");
        assert_eq!(base64_standard(&[0x66]), "Zg==");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn spec_076_previsualisation_ui_n_ecrit_rien_et_refuse_le_conflit() {
        use std::os::unix::fs::PermissionsExt;

        let root = std::env::temp_dir().join(format!(
            "bridget-spec-076-ui-preview-{}",
            uuid::Uuid::new_v4().simple()
        ));
        let projects = root.join("projects");
        std::fs::create_dir_all(&projects).unwrap();
        let policy_path = root.join("project-root-policy.json");
        std::fs::write(
            &policy_path,
            serde_json::to_vec(&serde_json::json!({
                "contract_version": 1,
                "policy_generation": 1,
                "allowed_project_roots": [projects],
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::set_permissions(&policy_path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let config = UiRelayConfig {
            daemon_socket: root.join("bridget.sock"),
            maicie_config: root.join("maicie.json"),
            project_root_policy_path: Some(policy_path),
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            token: "test".to_string(),
        };
        let preview = post_project_preview(
            &config,
            &serde_json::to_vec(&serde_json::json!({
                "version": 1,
                "mode": "create",
                "root": projects,
                "folder_name": "nouveau",
            }))
            .unwrap(),
        )
        .unwrap();
        assert_eq!(preview.mode, "create");
        assert!(!std::path::Path::new(&preview.canonical_path).exists());
        assert!(preview.git_initialization_proposed);
        assert!(
            post_project_preview(
                &config,
                &serde_json::to_vec(&serde_json::json!({
                    "version": 1,
                    "mode": "create",
                    "root": projects,
                    "folder_name": "../interdit",
                }))
                .unwrap(),
            )
            .is_err()
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn spec_080_confirmation_ui_cree_puis_enregistre_le_projet_dans_le_registre() {
        use std::os::unix::fs::PermissionsExt;

        let root = std::env::temp_dir().join(format!(
            "bridget-spec-080-ui-confirm-{}",
            uuid::Uuid::new_v4().simple()
        ));
        let projects = root.join("projects");
        std::fs::create_dir_all(&projects).unwrap();
        let policy_path = root.join("project-root-policy.json");
        std::fs::write(
            &policy_path,
            serde_json::to_vec(&serde_json::json!({
                "contract_version": 1,
                "policy_generation": 1,
                "allowed_project_roots": [projects],
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::set_permissions(&policy_path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let socket = root.join("bridget.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        let expected_root = projects.join("atelier");
        let expected_root_for_server = expected_root.clone();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode::<WrapperToDaemon>(line.trim()).unwrap(),
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Service
                }
            ));
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Service
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode::<WrapperToDaemon>(line.trim()).unwrap(),
                WrapperToDaemon::ServiceHello { capabilities, .. }
                    if capabilities == vec![ServiceCapability::ProjectRegistryV1]
            ));
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::ServiceWelcome {
                    version: SERVICE_CONTRACT_VERSION,
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 30,
                    capabilities: vec![ServiceCapability::ProjectRegistryV1],
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap();
            let WrapperToDaemon::ProjectRegistryRequest { request } =
                decode::<WrapperToDaemon>(line.trim()).unwrap()
            else {
                panic!("la confirmation doit appeler le registre de projets");
            };
            assert_eq!(
                request.requested_root,
                expected_root_for_server.display().to_string()
            );
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::ProjectRegistryOutcome {
                    outcome: bridget_transport::protocol::ProjectBindOutcome {
                        contract_version: PROJECT_REGISTRY_CONTRACT_VERSION,
                        command_id: request.command_id,
                        project_id: request.project_id,
                        status: ProjectBindStatus::Active,
                        binding_generation: Some(1),
                        backend: Some(ProjectBackend::Host),
                        runtime_policy: None,
                        reason: None,
                        existing_project_id: None,
                        existing_binding_generation: None,
                        observed_at: now_secs(),
                    },
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();
        });
        let config = UiRelayConfig {
            daemon_socket: socket,
            maicie_config: root.join("maicie.json"),
            project_root_policy_path: Some(policy_path),
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            token: "test".to_string(),
        };
        let confirmed = post_project_confirm(
            &config,
            &serde_json::to_vec(&serde_json::json!({
                "version": 1,
                "command_id": "project-ui-confirm-1",
                "mode": "create",
                "root": projects,
                "folder_name": "atelier",
                "initialize_git": false,
            }))
            .unwrap(),
        )
        .unwrap();
        assert_eq!(confirmed.display_name, "atelier");
        assert_eq!(
            confirmed.canonical_path,
            expected_root.display().to_string()
        );
        assert!(expected_root.is_dir(), "le dossier confirmé doit être créé");
        server.join().unwrap();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn spec_081_relais_valide_et_confirme_la_mutation_de_ronde() {
        let root = std::env::temp_dir().join(format!(
            "bridget-spec-081-ui-round-{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let socket = root.join("bridget.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode::<WrapperToDaemon>(line.trim()).unwrap(),
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Client
                }
            ));
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();

            line.clear();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode::<WrapperToDaemon>(line.trim()).unwrap(),
                WrapperToDaemon::ClientHello { capabilities, .. }
                    if capabilities == vec![ClientCapability::ProjectRoundPolicyV1]
            ));
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: "spec-081".to_string(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: vec![ClientCapability::ProjectRoundPolicyV1],
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();

            line.clear();
            reader.read_line(&mut line).unwrap();
            let WrapperToDaemon::ProjectRoundRequest { request } =
                decode::<WrapperToDaemon>(line.trim()).unwrap()
            else {
                panic!("mutation de ronde attendue");
            };
            assert_eq!(request.operation, ProjectRoundOperation::Enable);
            assert_eq!(request.project_id.as_deref(), Some("project-081"));
            assert_eq!(request.binding_generation, Some(3));
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::ProjectRoundOutcome {
                    outcome: ProjectRoundOutcome {
                        contract_version: PROJECT_ROUND_POLICY_CONTRACT_VERSION,
                        command_id: request.command_id,
                        operation: request.operation,
                        policies: vec![ProjectRoundProjection {
                            project_id: "project-081".to_string(),
                            binding_generation: Some(3),
                            active: true,
                            configured: true,
                            enabled: true,
                            revision: 4,
                            updated_at: now_secs(),
                            last_occurrence_at: Some(840),
                            last_dispatch_state: Some(ProjectRoundDispatchState::Deposited),
                            last_dispatch_observed_at: Some(850),
                        }],
                        reason: None,
                        observed_at: now_secs(),
                    },
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();
        });
        let config = UiRelayConfig {
            daemon_socket: socket,
            maicie_config: root.join("maicie.json"),
            project_root_policy_path: None,
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            token: "test".to_string(),
        };
        let accepted = post_project_round(
            &config,
            br#"{"version":1,"command_id":"round-ui-081","project_id":"project-081","binding_generation":3,"enabled":true}"#,
        )
        .unwrap();
        assert_eq!(accepted.project_id, "project-081");
        assert_eq!(accepted.binding_generation, 3);
        assert!(accepted.round.enabled);
        assert_eq!(accepted.round.last_occurrence_at, Some(840));
        server.join().unwrap();

        assert!(parse_ui_project_round_request(
            br#"{"version":1,"command_id":"round-ui-081","project_id":"project-081","binding_generation":3,"enabled":true,"provider":"codex"}"#
        )
        .is_err());
        assert!(parse_ui_project_round_request(
            br#"{"version":1,"command_id":"round-ui-081","project_id":"project-081","binding_generation":0,"enabled":true}"#
        )
        .is_err());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn spec_081_jointure_ronde_exige_projet_generation_et_etat_coherents() {
        let binding = ProjectBindingProjection {
            project_id: "project-081".to_string(),
            canonical_root: Some("/srv/project-081".to_string()),
            state: ProjectBindingStatus::Active,
            binding_generation: Some(3),
            backend: Some(ProjectBackend::Host),
            runtime_policy: None,
            reason: None,
            last_audit: None,
            observed_at: 900,
        };
        let policy = ProjectRoundProjection {
            project_id: "project-081".to_string(),
            binding_generation: Some(3),
            active: true,
            configured: true,
            enabled: true,
            revision: 2,
            updated_at: 850,
            last_occurrence_at: None,
            last_dispatch_state: None,
            last_dispatch_observed_at: None,
        };

        let project = ui_project_list_entry(binding.clone(), Some(policy.clone()))
            .unwrap()
            .unwrap();
        assert_eq!(project.binding_generation, 3);
        assert!(project.round.enabled);
        assert_eq!(project.round.interval_secs, PROJECT_ROUND_INTERVAL_SECS);

        let missing = ui_project_list_entry(binding.clone(), None).unwrap_err();
        assert_eq!(missing.1, "round_service_unavailable");

        let mut stale = policy.clone();
        stale.binding_generation = Some(2);
        let mismatch = ui_project_list_entry(binding.clone(), Some(stale)).unwrap_err();
        assert_eq!(mismatch.1, "round_service_unavailable");

        let mut partial = policy;
        partial.last_occurrence_at = Some(840);
        let incomplete = ui_project_list_entry(binding, Some(partial)).unwrap_err();
        assert_eq!(incomplete.1, "round_service_unavailable");

        assert_eq!(
            project_round_refusal(ProjectRoundRefusal::ProjectInactive).1,
            "project_inactive"
        );
        assert_eq!(
            project_round_refusal(ProjectRoundRefusal::BindingGenerationMismatch).1,
            "binding_generation_mismatch"
        );
    }

    #[test]
    fn spec_081_perte_de_reponse_ronde_reste_une_indisponibilite_fermee() {
        let root = std::env::temp_dir().join(format!(
            "bridget-spec-081-ui-round-loss-{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let socket = root.join("bridget.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();

            line.clear();
            reader.read_line(&mut line).unwrap();
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: "spec-081".to_string(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: vec![ClientCapability::ProjectRoundPolicyV1],
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();

            line.clear();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode::<WrapperToDaemon>(line.trim()).unwrap(),
                WrapperToDaemon::ProjectRoundRequest { .. }
            ));
            // La connexion disparaît avant tout verdict: aucun état n'est confirmé.
        });
        let error = request_project_round(
            &socket,
            "round-ui-loss-081".to_string(),
            ProjectRoundOperation::Enable,
            Some("project-081".to_string()),
            Some(3),
        )
        .unwrap_err();
        assert_eq!(error.0, 503);
        assert_eq!(error.1, "round_service_unavailable");
        server.join().unwrap();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn spec_081_capacite_ronde_absente_ne_fabrique_pas_un_etat() {
        let root = std::env::temp_dir().join(format!(
            "bridget-spec-081-ui-round-capability-{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let socket = root.join("bridget.sock");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap();
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: "spec-081".to_string(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: Vec::new(),
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();
        });
        let error = open_project_round_client(&socket).unwrap_err();
        assert_eq!(error.0, 503);
        assert_eq!(error.1, "round_service_unavailable");
        server.join().unwrap();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn logos_runtime_sont_servis_comme_assets_svg_revalides() {
        for (path, expected) in [
            ("/providers/openai.svg", UI_PROVIDER_OPENAI),
            ("/providers/claude.svg", UI_PROVIDER_CLAUDE),
            ("/providers/cursor.svg", UI_PROVIDER_CURSOR),
            ("/providers/gemini.svg", UI_PROVIDER_GEMINI),
            ("/providers/glm.svg", UI_PROVIDER_GLM),
            ("/providers/deepseek.svg", UI_PROVIDER_DEEPSEEK),
        ] {
            let (relay, address) = spawn_asset_relay();
            let worker = thread::spawn(move || {
                relay.serve_one().unwrap();
                relay.serve_one().unwrap();
            });
            let (status, raw) = get_asset(address, path, None);
            assert_eq!(status, 200, "{path}: {raw}");
            assert_eq!(
                header_value(&raw, "Content-Type").as_deref(),
                Some("image/svg+xml")
            );
            assert_eq!(
                header_value(&raw, "ETag").as_deref(),
                Some(asset_etag(expected).as_str())
            );
            assert!(
                header_value(&raw, "Cache-Control")
                    .as_deref()
                    .is_some_and(|value| value.split(',').any(|item| item.trim() == "no-cache")),
                "{path} doit être revalidé"
            );
            assert_eq!(
                raw.split("\r\n\r\n").nth(1).unwrap_or("").as_bytes(),
                expected
            );
            let etag = header_value(&raw, "ETag").unwrap();
            let (status, revalidated) = get_asset(address, path, Some(&etag));
            worker.join().unwrap();
            assert_eq!(status, 304, "{path}: {revalidated}");
            assert_eq!(revalidated.split("\r\n\r\n").nth(1).unwrap_or("x"), "");
        }
    }

    #[test]
    fn spec_078_profil_exige_jeton_et_ne_projette_pas_le_routage() {
        let root = std::env::temp_dir().join(format!(
            "bridget-spec-078-profile-{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let socket = root.join("bridget.sock");
        let mut store = AgentProfileStore::open(&socket.with_extension("db")).unwrap();
        store.ensure_routing_names(["routing-secret"]).unwrap();
        let profile = store
            .profile_for_routing_name("routing-secret")
            .unwrap()
            .unwrap();
        store
            .update_profile(
                &profile.agent_id,
                AgentProfileUpdate {
                    expected_revision: profile.revision,
                    display_name: "Bibliothécaire".to_string(),
                    labels: vec!["recherche".to_string()],
                    avatar_shape: "round".to_string(),
                    avatar_color: "blue".to_string(),
                    instructions: "consigne interne confidentielle".to_string(),
                },
            )
            .unwrap();

        let config = UiRelayConfig {
            daemon_socket: socket,
            maicie_config: root.join("maicie.json"),
            project_root_policy_path: None,
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            token: "jeton-spec-078".to_string(),
        };
        let relay = UiRelay::bind(config.clone()).unwrap();
        let address = relay.local_addr().unwrap();
        let worker = thread::spawn(move || relay.serve_one().unwrap());
        let (denied, _) = get_asset(
            address,
            &format!("/v1/agents/{}/profile", profile.agent_id),
            None,
        );
        worker.join().unwrap();
        assert_eq!(denied, 403);

        let relay = UiRelay::bind(config).unwrap();
        let address = relay.local_addr().unwrap();
        let worker = thread::spawn(move || relay.serve_one().unwrap());
        let (status, raw) = get_asset(
            address,
            &format!(
                "/v1/agents/{}/profile?token=jeton-spec-078",
                profile.agent_id
            ),
            None,
        );
        worker.join().unwrap();
        assert_eq!(status, 200, "{raw}");
        assert!(raw.contains("Bibliothécaire"));
        assert!(!raw.contains("routing-secret"));
        assert!(raw.contains("consigne interne confidentielle"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn spec_078_attention_projette_un_nom_affiche_et_erreurs_sans_detail() {
        let event = ui_attention_event(AttentionEvent {
            event_id: "event-opaque".to_string(),
            agent_id: "profile-opaque".to_string(),
            display_name: "Bibliothécaire".to_string(),
            event_type: AttentionEventType::HumanInputNeeded,
            created_at: 42,
            seen: false,
            native_notified: false,
            attention_enabled: true,
        });
        let payload = serde_json::to_string(&event).expect("projection JSON");
        assert!(payload.contains("Bibliothécaire attend votre réponse."));
        assert!(!payload.contains("routing-name"));
        assert!(!payload.contains("consigne interne"));
        let (_, code, message) =
            attention_error_response(AgentProfileError::Invalid("consigne interne"));
        assert_eq!(code, "invalid_attention");
        assert!(!message.contains("consigne interne"));
    }

    #[test]
    fn ticket_sandbox_est_court_et_lie_a_sa_portee() {
        let runtime = UiRelayRuntime::new(None);
        let ticket = runtime.issue_sandbox_ticket(
            "project:test".to_owned(),
            "agent-test".to_owned(),
            "artifact-version:test".to_owned(),
        );
        let resolved = runtime
            .resolve_sandbox_ticket(&ticket)
            .expect("ticket présent");
        assert_eq!(resolved.project_id, "project:test");
        assert_eq!(resolved.agent, "agent-test");
        assert_eq!(resolved.version_ref, "artifact-version:test");
        assert!(runtime.resolve_sandbox_ticket("0").is_none());
    }

    #[test]
    fn page_sandbox_injecte_le_protocole_sans_alterer_le_document() {
        let page = sandbox_frame_document(
            "<!doctype html><html><head><title>Test</title></head><body><h1>Visible</h1></body></html>",
            "frame-123",
            "dark",
        );
        assert!(!page.contains("Content-Security-Policy"));
        assert!(page.contains("sandbox.ready"));
        assert!(page.contains("frame_instance_id"));
        assert!(page.contains("<h1>Visible</h1>"));
        assert!(page.contains("<head><meta charset=\"utf-8\">"));
        assert!(page.contains("bridget-runtime-theme"));
        assert!(page.contains("sandbox.resize"));
        assert!(page.contains("body.children"));
        assert!(page.contains("[class~=\"metric\"]"));
    }
}
