//! Contrat t3code — seul point de dépendance de Bridget à l'API HTTP locale
//! de t3code et à sa commande `t3` (session 098, contrat vérifié sur
//! T3 Code 0.0.40 le 2026-09-14). Tout ce qui est lu est validé champ par
//! champ ; une forme inattendue est un refus nommé, jamais un état inventé.
//! Bridget n'écrit dans t3code que par `dispatch` ; la base d'authentification
//! n'est touchée que par la commande officielle `t3`.

use serde_json::Value;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// Fichier d'état publié par le serveur t3code (`<base>/userdata/`).
pub const RUNTIME_FILE: &str = "server-runtime.json";
const HTTP_TIMEOUT: Duration = Duration::from_secs(10);
/// Sujet de toutes les sessions Bridget ; le libellé porte l'installation.
pub const TOKEN_SUBJECT: &str = "bridget";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractError {
    /// t3code n'est pas démarré : fichier d'état absent.
    NotRunning(PathBuf),
    /// Forme inattendue : route ou fichier, puis champ.
    InvalidShape {
        source: &'static str,
        field: String,
    },
    /// Le serveur annonce une adresse hors de la boucle locale.
    NotLoopback(String),
    /// Le PID publié ne vit plus : fichier périmé.
    StaleRuntime(u32),
    Unauthorized,
    Http {
        status: i32,
        body: String,
    },
    Transport(String),
    /// La commande `t3` est absente ou a échoué.
    Cli(String),
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRunning(path) => write!(f, "t3code non démarré : {} absent", path.display()),
            Self::InvalidShape { source, field } => {
                write!(f, "forme t3code inattendue ({source}) : champ {field}")
            }
            Self::NotLoopback(host) => write!(f, "t3code hors boucle locale refusé : {host}"),
            Self::StaleRuntime(pid) => write!(f, "fichier d'état t3code périmé : PID {pid} absent"),
            Self::Unauthorized => write!(f, "t3code : jeton refusé (401)"),
            Self::Http { status, body } => write!(f, "t3code HTTP {status} : {body}"),
            Self::Transport(detail) => write!(f, "t3code injoignable : {detail}"),
            Self::Cli(detail) => write!(f, "commande t3 : {detail}"),
        }
    }
}

impl std::error::Error for ContractError {}

fn shape(source: &'static str, field: &str) -> ContractError {
    ContractError::InvalidShape {
        source,
        field: field.to_string(),
    }
}

fn str_field<'a>(
    value: &'a Value,
    source: &'static str,
    field: &str,
) -> Result<&'a str, ContractError> {
    // `field` est le libellé du refus (chemin lisible) ; la clé JSON lue est
    // son dernier segment (`projects[].id` → `id`).
    let key = field.rsplit('.').next().unwrap_or(field);
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| shape(source, field))
}

fn opt_str(value: &Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

/// Répertoire de base de t3code : `T3CODE_HOME` sinon `~/.t3`.
pub fn base_dir() -> Result<PathBuf, ContractError> {
    if let Some(home) = std::env::var_os("T3CODE_HOME").filter(|v| !v.is_empty()) {
        return Ok(PathBuf::from(home));
    }
    std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join(".t3"))
        .ok_or_else(|| ContractError::Cli("HOME absent".into()))
}

pub fn runtime_file(base: &Path) -> PathBuf {
    base.join("userdata").join(RUNTIME_FILE)
}

/// Serveur t3code local attesté : port publié et PID vivant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerRuntime {
    pub port: u16,
    pub pid: u32,
}

impl ServerRuntime {
    /// Adresse reconstruite : jamais l'`origin` publié, toujours la boucle locale.
    pub fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

/// Lit et valide le fichier d'état. `alive` dit si un PID vit (injecté pour les tests).
pub fn parse_runtime(
    text: &str,
    alive: impl Fn(u32) -> bool,
) -> Result<ServerRuntime, ContractError> {
    const SRC: &str = "server-runtime.json";
    let value: Value = serde_json::from_str(text).map_err(|_| shape(SRC, "json"))?;
    // L'application publie `host` ; `t3 --mode web` ne publie que `origin`.
    // Les deux sont acceptés, aucun des deux n'est une adresse de confiance :
    // l'hôte lu ne sert qu'à refuser une écoute non locale.
    let host = match value.get("host").and_then(Value::as_str) {
        Some(host) => host.to_string(),
        None => origin_host(&value).ok_or_else(|| shape(SRC, "host"))?,
    };
    if !matches!(host.as_str(), "127.0.0.1" | "localhost" | "::1") {
        return Err(ContractError::NotLoopback(host));
    }
    let port = value
        .get("port")
        .and_then(Value::as_u64)
        .filter(|p| (1..=65535).contains(p))
        .ok_or_else(|| shape(SRC, "port"))? as u16;
    let pid = value
        .get("pid")
        .and_then(Value::as_u64)
        .filter(|p| *p > 0 && *p <= u32::MAX as u64)
        .ok_or_else(|| shape(SRC, "pid"))? as u32;
    if !alive(pid) {
        return Err(ContractError::StaleRuntime(pid));
    }
    Ok(ServerRuntime { port, pid })
}

/// Hôte d'une origine `http://<hôte>:<port>` publiée par le serveur web.
fn origin_host(value: &Value) -> Option<String> {
    let origin = value.get("origin").and_then(Value::as_str)?;
    let rest = origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"))?;
    let authority = rest.split('/').next()?;
    let host = match authority.rsplit_once(':') {
        Some((host, port)) if port.chars().all(|c| c.is_ascii_digit()) => host,
        _ => authority,
    };
    (!host.is_empty()).then(|| host.trim_matches(['[', ']']).to_string())
}

pub fn read_runtime(base: &Path) -> Result<ServerRuntime, ContractError> {
    let path = runtime_file(base);
    let text =
        std::fs::read_to_string(&path).map_err(|_| ContractError::NotRunning(path.clone()))?;
    parse_runtime(&text, |pid| unsafe { libc::kill(pid as i32, 0) == 0 })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatestTurn {
    pub turn_id: String,
    pub state: String,
    pub assistant_message_id: Option<String>,
    /// Même createdAt de la commande que le message utilisateur (projection T3).
    pub requested_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSummary {
    pub provider_name: String,
    pub provider_instance_id: Option<String>,
    pub status: String,
    pub active_turn_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadSummary {
    pub id: String,
    pub project_id: String,
    pub title: String,
    /// Présent même sur un fil jamais démarré : c'est la seule source du
    /// fournisseur tant qu'aucune session n'existe.
    pub provider_instance_id: Option<String>,
    /// Politique du fil, répétée telle quelle au démarrage d'un tour : le pont
    /// ne décide jamais du niveau d'approbation à la place de l'humain.
    pub runtime_mode: String,
    pub interaction_mode: String,
    pub worktree_path: Option<String>,
    pub archived_at: Option<String>,
    pub deleted_at: Option<String>,
    /// « settled » = conversation rangée par l'humain ou issue d'un import
    /// d'historique. Ce n'est pas un agent joignable.
    pub settled_override: Option<String>,
    pub updated_at: String,
    pub latest_turn: Option<LatestTurn>,
    pub session: Option<SessionSummary>,
}

impl ThreadSummary {
    /// Un fil archivé, supprimé ou rangé n'est plus un agent joignable.
    /// L'import d'historique de t3code range ses fils : sans ce filtre, des
    /// centaines de conversations passées apparaîtraient dans l'annuaire.
    pub fn is_live(&self) -> bool {
        self.archived_at.is_none()
            && self.deleted_at.is_none()
            && self.settled_override.as_deref() != Some("settled")
    }
    /// Empreinte de changement : t3code 0.0.40 n'expose pas de séquence par fil.
    pub fn change_key(&self) -> String {
        format!(
            "{}|{}|{}",
            self.updated_at,
            self.latest_turn
                .as_ref()
                .map(|t| t.turn_id.as_str())
                .unwrap_or("-"),
            self.latest_turn
                .as_ref()
                .map(|t| t.state.as_str())
                .unwrap_or("-")
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub id: String,
    pub workspace_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub snapshot_sequence: u64,
    pub projects: Vec<Project>,
    pub threads: Vec<ThreadSummary>,
}

fn parse_latest_turn(
    value: Option<&Value>,
    source: &'static str,
) -> Result<Option<LatestTurn>, ContractError> {
    let Some(turn) = value.filter(|v| !v.is_null()) else {
        return Ok(None);
    };
    Ok(Some(LatestTurn {
        turn_id: str_field(turn, source, "latestTurn.turnId")?.to_string(),
        state: str_field(turn, source, "latestTurn.state")?.to_string(),
        requested_at: opt_str(turn, "requestedAt"),
        assistant_message_id: opt_str(turn, "assistantMessageId"),
    }))
}

pub fn parse_snapshot(text: &str) -> Result<Snapshot, ContractError> {
    const SRC: &str = "GET /api/orchestration/snapshot";
    let value: Value = serde_json::from_str(text).map_err(|_| shape(SRC, "json"))?;
    let snapshot_sequence = value
        .get("snapshotSequence")
        .and_then(Value::as_u64)
        .ok_or_else(|| shape(SRC, "snapshotSequence"))?;
    let mut projects = Vec::new();
    for project in value
        .get("projects")
        .and_then(Value::as_array)
        .ok_or_else(|| shape(SRC, "projects"))?
    {
        projects.push(Project {
            id: str_field(project, SRC, "projects[].id")?.to_string(),
            workspace_root: str_field(project, SRC, "projects[].workspaceRoot")?.to_string(),
        });
    }
    let mut threads = Vec::new();
    for thread in value
        .get("threads")
        .and_then(Value::as_array)
        .ok_or_else(|| shape(SRC, "threads"))?
    {
        let session = match thread.get("session").filter(|s| !s.is_null()) {
            Some(session) => Some(SessionSummary {
                provider_name: str_field(session, SRC, "threads[].session.providerName")?
                    .to_string(),
                provider_instance_id: opt_str(session, "providerInstanceId"),
                status: str_field(session, SRC, "threads[].session.status")?.to_string(),
                active_turn_id: opt_str(session, "activeTurnId"),
            }),
            None => None,
        };
        threads.push(ThreadSummary {
            id: str_field(thread, SRC, "threads[].id")?.to_string(),
            project_id: str_field(thread, SRC, "threads[].projectId")?.to_string(),
            title: opt_str(thread, "title").unwrap_or_default(),
            provider_instance_id: thread
                .get("modelSelection")
                .and_then(|selection| opt_str(selection, "instanceId")),
            // Défauts volontairement restrictifs si t3code cessait de les publier.
            runtime_mode: opt_str(thread, "runtimeMode")
                .unwrap_or_else(|| "approval-required".to_string()),
            interaction_mode: opt_str(thread, "interactionMode")
                .unwrap_or_else(|| "default".to_string()),
            worktree_path: opt_str(thread, "worktreePath"),
            archived_at: opt_str(thread, "archivedAt"),
            deleted_at: opt_str(thread, "deletedAt"),
            settled_override: opt_str(thread, "settledOverride"),
            updated_at: str_field(thread, SRC, "threads[].updatedAt")?.to_string(),
            latest_turn: parse_latest_turn(thread.get("latestTurn"), SRC)?,
            session,
        });
    }
    Ok(Snapshot {
        snapshot_sequence,
        projects,
        threads,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub id: String,
    pub role: String,
    pub text: String,
    pub streaming: bool,
    pub turn_id: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    pub id: String,
    pub kind: String,
    pub turn_id: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ThreadDetail {
    pub messages: Vec<Message>,
    pub latest_turn: Option<LatestTurn>,
    pub activities: Vec<Activity>,
    pub activities_available: bool,
}

pub fn parse_thread_detail(text: &str) -> Result<ThreadDetail, ContractError> {
    const SRC: &str = "GET /api/orchestration/threads/:id";
    let value: Value = serde_json::from_str(text).map_err(|_| shape(SRC, "json"))?;
    let thread = value.get("thread").ok_or_else(|| shape(SRC, "thread"))?;
    let mut messages = Vec::new();
    for message in thread
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| shape(SRC, "thread.messages"))?
    {
        messages.push(Message {
            id: str_field(message, SRC, "messages[].id")?.to_string(),
            role: str_field(message, SRC, "messages[].role")?.to_string(),
            text: message
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            streaming: message
                .get("streaming")
                .and_then(Value::as_bool)
                .ok_or_else(|| shape(SRC, "messages[].streaming"))?,
            turn_id: opt_str(message, "turnId"),
            created_at: opt_str(message, "createdAt"),
        });
    }
    let mut activities = Vec::new();
    if let Some(items) = thread.get("activities").and_then(Value::as_array) {
        if items.len() > 4096 {
            return Err(shape(SRC, "activities: limite 4096 dépassée"));
        }
        for item in items {
            activities.push(Activity {
                id: str_field(item, SRC, "activities[].id")?.to_string(),
                kind: str_field(item, SRC, "activities[].kind")?.to_string(),
                turn_id: opt_str(item, "turnId"),
                payload: item.get("payload").cloned().unwrap_or(Value::Null),
            });
        }
    }
    Ok(ThreadDetail {
        messages,
        latest_turn: parse_latest_turn(thread.get("latestTurn"), SRC)?,
        activities,
        activities_available: thread.get("activities").is_some_and(Value::is_array),
    })
}

/// Commande `thread.turn.start` : `commandId` = identifiant de remise (t3code
/// déduplique dessus), `messageId` choisi par le pont pour la corrélation.
pub fn turn_start_command(
    thread_id: &str,
    command_id: &str,
    message_id: &str,
    text: &str,
    runtime_mode: &str,
    interaction_mode: &str,
) -> Value {
    // `runtimeMode` et `interactionMode` sont exigés par la route HTTP (le
    // schéma client n'a pas de valeur par défaut, contrairement au schéma
    // interne) ; ils reprennent ceux du fil pour ne pas changer sa politique.
    serde_json::json!({
        "type": "thread.turn.start",
        "commandId": command_id,
        "threadId": thread_id,
        "message": { "messageId": message_id, "role": "user", "text": text, "attachments": [] },
        "runtimeMode": runtime_mode,
        "interactionMode": interaction_mode,
        "createdAt": iso_now(),
    })
}

pub fn iso_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs() as i64;
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    // Conversion civile (algorithme de Howard Hinnant), sans dépendance de date.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}.{:03}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60,
        now.subsec_millis()
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchResult {
    pub sequence: u64,
}

/// Client HTTP loopback minimal, un jeton porteur, délais bornés.
pub struct Client {
    base_url: String,
    token: String,
}

impl Client {
    pub fn new(runtime: &ServerRuntime, token: &str) -> Self {
        Self {
            base_url: runtime.base_url(),
            token: token.to_string(),
        }
    }

    pub fn for_base_url(base_url: &str, token: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
        }
    }

    fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<&Value>,
    ) -> Result<String, ContractError> {
        let url = format!("{}{}", self.base_url, path);
        let mut request = match method {
            "GET" => minreq::get(&url),
            _ => minreq::post(&url),
        }
        .with_header("Authorization", format!("Bearer {}", self.token))
        .with_header("Accept", "application/json")
        .with_timeout(HTTP_TIMEOUT.as_secs());
        if let Some(body) = body {
            request = request
                .with_header("Content-Type", "application/json")
                .with_body(body.to_string());
        }
        let response = request
            .send()
            .map_err(|error| ContractError::Transport(error.to_string()))?;
        let text = response.as_str().unwrap_or_default().to_string();
        match response.status_code {
            200..=299 => Ok(text),
            401 => Err(ContractError::Unauthorized),
            status => Err(ContractError::Http {
                status,
                body: text.chars().take(300).collect(),
            }),
        }
    }

    pub fn snapshot(&self) -> Result<Snapshot, ContractError> {
        parse_snapshot(&self.request("GET", "/api/orchestration/snapshot", None)?)
    }

    pub fn thread_detail(
        &self,
        thread_id: &str,
        turn_limit: u32,
    ) -> Result<ThreadDetail, ContractError> {
        let path = format!("/api/orchestration/threads/{thread_id}?turnLimit={turn_limit}");
        parse_thread_detail(&self.request("GET", &path, None)?)
    }

    pub fn dispatch(&self, command: &Value) -> Result<DispatchResult, ContractError> {
        const SRC: &str = "POST /api/orchestration/dispatch";
        let text = self.request("POST", "/api/orchestration/dispatch", Some(command))?;
        let value: Value = serde_json::from_str(&text).map_err(|_| shape(SRC, "json"))?;
        let sequence = value
            .get("sequence")
            .and_then(Value::as_u64)
            .ok_or_else(|| shape(SRC, "sequence"))?;
        Ok(DispatchResult { sequence })
    }
}

/// Session émise par la commande officielle `t3 auth session issue --json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuedSession {
    pub session_id: String,
    pub token: String,
    pub expires_at: String,
    pub label: String,
}

pub fn parse_issued_session(text: &str) -> Result<IssuedSession, ContractError> {
    const SRC: &str = "t3 auth session issue --json";
    let value: Value = serde_json::from_str(text).map_err(|_| shape(SRC, "json"))?;
    let session_id = str_field(&value, SRC, "sessionId")?.to_string();
    let token = str_field(&value, SRC, "token")?.to_string();
    let expires_at = str_field(&value, SRC, "expiresAt")?.to_string();
    let client = value.get("client").ok_or_else(|| shape(SRC, "client"))?;
    Ok(IssuedSession {
        session_id,
        token,
        expires_at,
        label: str_field(client, SRC, "client.label")?.to_string(),
    })
}

/// Sessions listées par `t3 auth session list --json` : identifiant et libellé.
pub fn parse_session_list(text: &str) -> Result<Vec<(String, Option<String>)>, ContractError> {
    const SRC: &str = "t3 auth session list --json";
    let value: Value = serde_json::from_str(text).map_err(|_| shape(SRC, "json"))?;
    let items = value
        .as_array()
        .or_else(|| value.get("sessions").and_then(Value::as_array))
        .ok_or_else(|| shape(SRC, "sessions"))?;
    items
        .iter()
        .map(|item| {
            Ok((
                str_field(item, SRC, "sessionId")?.to_string(),
                item.get("client").and_then(|c| opt_str(c, "label")),
            ))
        })
        .collect()
}

/// Façade sur la commande `t3` ; le binaire est injectable pour les tests.
pub struct T3Cli {
    pub binary: PathBuf,
}

impl T3Cli {
    pub fn from_env() -> Self {
        Self {
            binary: std::env::var_os("BRIDGET_T3_BIN")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("t3")),
        }
    }

    fn run(&self, args: &[&str]) -> Result<String, ContractError> {
        let output = Command::new(&self.binary)
            .args(args)
            .output()
            .map_err(|error| {
                ContractError::Cli(format!(
                    "{} introuvable ({error}) ; installer le CLI t3 (npm i -g t3)",
                    self.binary.display()
                ))
            })?;
        if !output.status.success() {
            return Err(ContractError::Cli(format!(
                "t3 {} : {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr)
                    .chars()
                    .take(300)
                    .collect::<String>()
            )));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn issue_session(&self, label: &str, ttl: &str) -> Result<IssuedSession, ContractError> {
        let out = self.run(&[
            "auth",
            "session",
            "issue",
            "--subject",
            TOKEN_SUBJECT,
            "--label",
            label,
            "--ttl",
            ttl,
            "--json",
        ])?;
        parse_issued_session(&out)
    }

    pub fn list_sessions(&self) -> Result<Vec<(String, Option<String>)>, ContractError> {
        parse_session_list(&self.run(&["auth", "session", "list", "--json"])?)
    }

    pub fn revoke_session(&self, session_id: &str) -> Result<(), ContractError> {
        self.run(&["auth", "session", "revoke", session_id])
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RUNTIME: &str = r#"{"version":1,"pid":19619,"host":"127.0.0.1","port":3773,"origin":"http://127.0.0.1:3773","startedAt":"2026-09-14T07:28:30.671Z"}"#;

    #[test]
    fn spec098_runtime_du_mode_web_sans_host_est_lu_par_son_origine() {
        // Forme réelle de `t3 --mode web --no-browser` (0.0.40) : pas de `host`.
        let web = r#"{"version":1,"pid":90922,"port":3773,"origin":"http://127.0.0.1:3773","startedAt":"2026-09-14T08:34:38.449Z"}"#;
        assert_eq!(
            parse_runtime(web, |_| true).unwrap(),
            ServerRuntime {
                port: 3773,
                pid: 90922
            }
        );
        let distant = r#"{"version":1,"pid":1,"port":3773,"origin":"http://192.0.2.10:3773"}"#;
        assert!(matches!(
            parse_runtime(distant, |_| true),
            Err(ContractError::NotLoopback(host)) if host == "192.0.2.10"
        ));
        let sans_repere = r#"{"version":1,"pid":1,"port":3773}"#;
        assert!(matches!(
            parse_runtime(sans_repere, |_| true),
            Err(ContractError::InvalidShape { field, .. }) if field == "host"
        ));
    }

    #[test]
    fn runtime_valide_reconstruit_la_boucle_locale_et_refuse_le_reste() {
        let runtime = parse_runtime(RUNTIME, |_| true).unwrap();
        assert_eq!(
            runtime,
            ServerRuntime {
                port: 3773,
                pid: 19619
            }
        );
        assert_eq!(runtime.base_url(), "http://127.0.0.1:3773");
        assert_eq!(
            parse_runtime(RUNTIME, |_| false),
            Err(ContractError::StaleRuntime(19619))
        );
        let distant = RUNTIME.replace("\"host\":\"127.0.0.1\"", "\"host\":\"0.0.0.0\"");
        assert_eq!(
            parse_runtime(&distant, |_| true),
            Err(ContractError::NotLoopback("0.0.0.0".into()))
        );
        assert!(
            matches!(parse_runtime("{\"host\":\"127.0.0.1\"}", |_| true), Err(ContractError::InvalidShape { field, .. }) if field == "port")
        );
        assert!(matches!(
            parse_runtime("pas du json", |_| true),
            Err(ContractError::InvalidShape { .. })
        ));
    }

    #[test]
    fn snapshot_reel_0_0_40_est_lu_et_les_champs_manquants_sont_nommes() {
        let text = r#"{"snapshotSequence":374,"projects":[{"id":"p1","workspaceRoot":"/tmp/p1"}],"threads":[
          {"id":"t1","projectId":"p1","title":"Sonde","worktreePath":null,"archivedAt":null,"deletedAt":null,"updatedAt":"2026-09-14T07:31:16Z",
           "latestTurn":{"turnId":"turn-1","state":"completed","assistantMessageId":"assistant:a1"},
           "session":{"threadId":"t1","status":"ready","providerName":"claudeAgent","providerInstanceId":"claudeAgent","activeTurnId":null}},
          {"id":"t2","projectId":"p1","title":"Archivé","updatedAt":"2026-09-14T07:00:00Z","archivedAt":"2026-09-14T07:32:00Z","latestTurn":null,"session":null}
        ],"updatedAt":"x"}"#;
        let snapshot = parse_snapshot(text).unwrap();
        assert_eq!(snapshot.snapshot_sequence, 374);
        assert_eq!(snapshot.projects[0].workspace_root, "/tmp/p1");
        let live = &snapshot.threads[0];
        assert!(live.is_live());
        assert_eq!(live.session.as_ref().unwrap().provider_name, "claudeAgent");
        assert_eq!(
            live.latest_turn
                .as_ref()
                .unwrap()
                .assistant_message_id
                .as_deref(),
            Some("assistant:a1")
        );
        assert!(!snapshot.threads[1].is_live());
        assert_ne!(live.change_key(), snapshot.threads[1].change_key());
        let sans_id = text.replace("\"id\":\"t1\",", "");
        assert!(
            matches!(parse_snapshot(&sans_id), Err(ContractError::InvalidShape { field, .. }) if field == "threads[].id")
        );
    }

    #[test]
    fn detail_reel_distingue_utilisateur_et_tours_assistant() {
        let text = r#"{"snapshotSequence":1,"thread":{"messages":[
          {"id":"u1","role":"user","text":"PONG ?","streaming":false,"turnId":null},
          {"id":"assistant:a1","role":"assistant","text":"PONG-098","streaming":false,"turnId":"turn-1"}
        ],"latestTurn":{"turnId":"turn-1","state":"completed","assistantMessageId":"assistant:a1"}},"page":{}}"#;
        let detail = parse_thread_detail(text).unwrap();
        assert_eq!(detail.messages.len(), 2);
        assert_eq!(detail.messages[0].turn_id, None);
        assert_eq!(detail.messages[1].turn_id.as_deref(), Some("turn-1"));
        assert!(
            matches!(parse_thread_detail(r#"{"thread":{"messages":[{"id":"x","role":"user"}]}}"#), Err(ContractError::InvalidShape { field, .. }) if field == "messages[].streaming")
        );
    }

    #[test]
    fn commande_de_tour_porte_les_identifiants_du_pont() {
        let command = turn_start_command(
            "t1",
            "delivery-1",
            "message-1",
            "Bonjour",
            "approval-required",
            "default",
        );
        assert_eq!(command["type"], "thread.turn.start");
        assert_eq!(command["commandId"], "delivery-1");
        assert_eq!(command["message"]["messageId"], "message-1");
        assert_eq!(command["message"]["role"], "user");
        assert!(command["createdAt"].as_str().unwrap().ends_with('Z'));
        // La politique du fil est répétée, jamais choisie par le pont : la
        // route HTTP l'exige et un « full-access » imposé serait une élévation.
        assert_eq!(command["runtimeMode"], "approval-required");
        assert_eq!(command["interactionMode"], "default");
    }

    #[test]
    fn session_emise_et_liste_sont_lues_par_libelle() {
        let issued = parse_issued_session(r#"{"sessionId":"s1","token":"eyJ…","method":"bearer-access-token","scopes":["a"],"subject":"bridget","client":{"label":"bridget-abc","deviceType":"bot"},"expiresAt":"2026-09-14T09:29:28Z"}"#).unwrap();
        assert_eq!(
            (issued.session_id.as_str(), issued.label.as_str()),
            ("s1", "bridget-abc")
        );
        assert!(
            matches!(parse_issued_session(r#"{"sessionId":"s1"}"#), Err(ContractError::InvalidShape { field, .. }) if field == "token")
        );
        let list = parse_session_list(r#"[{"sessionId":"s1","client":{"label":"bridget-abc"}},{"sessionId":"s2","client":{}}]"#).unwrap();
        assert_eq!(
            list,
            vec![
                ("s1".into(), Some("bridget-abc".into())),
                ("s2".into(), None)
            ]
        );
    }

    #[test]
    fn iso_now_est_une_date_utc_plausible() {
        let now = iso_now();
        assert_eq!(now.len(), 24, "{now}");
        assert!(now.starts_with("20") && now.ends_with('Z'));
    }
}
