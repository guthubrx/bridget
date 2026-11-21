//! Console web locale durcie de Bridget (`bridget accounts ui`, Lot 1a-0).
//!
//! - Écoute strictement locale (`127.0.0.1:0` par défaut).
//! - Garde serveur commune (`Host`, `Origin`, cookie propre au port + preuve `x-bridget-token`
//!   cloisonnée par origine dans `sessionStorage`, échange unique du jeton de lancement).
//! - Routeur minimal Lot 1a-0 : ferme les anciennes routes différées (`/api/prepare`,
//!   `/api/login/*`, `/api/automanaged/*`).
//! - Thèmes `system | light | dark` persistés dans `ui-prefs.json` (`0600`) sans flash.
//! - Activation/désactivation en deux temps (`preview` + `apply`) via `provider_accounts`
//!   uniquement sur registre local/maître d'autorité attestée.

use crate::accounts_connect::{
    ConnectionStatus, FleetAuthStatus, collect_fleet_status, open_browser, valid_machine_id,
};
use crate::cli_table::table_println as println;
use crate::provider_accounts;
use crate::registry::AgentRegistry;
use axum::Router;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{Html, IntoResponse, Json, Response};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const INDEX_HTML: &str = include_str!("../assets/console/index.html");
const STYLES_CSS: &str = include_str!("../assets/console/styles.css");
const APP_JS: &str = include_str!("../assets/console/app.js");
const API_JS: &str = include_str!("../assets/console/api.js");
const VIEW_ACCOUNTS_JS: &str = include_str!("../assets/console/views/accounts.js");

const SESSION_TTL: Duration = Duration::from_secs(8 * 3600);
const PREVIEW_TTL: Duration = Duration::from_secs(300);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RegistryAuthority {
    pub owner_kind: String,
    pub owner_badge: String,
    pub writable: bool,
}

#[derive(Debug, Clone)]
struct BoundPreview {
    session_id: String,
    registry_path: PathBuf,
    action: String,
    account_id: String,
    expected_sha256: String,
    expires_at: Instant,
}

#[derive(Clone)]
pub(crate) struct ConsoleState {
    expected_host: String,
    expected_origin: String,
    cookie_name: String,
    launch_token: Arc<Mutex<Option<String>>>,
    session_id: String,
    client_token: String,
    expires_at: Instant,
    registry_override: Option<PathBuf>,
    attested_default_registry: Option<PathBuf>,
    prefs_path: PathBuf,
    previews: Arc<Mutex<HashMap<String, BoundPreview>>>,
}

#[derive(Deserialize)]
struct BootstrapQuery {
    token: Option<String>,
}

#[derive(Deserialize)]
struct ThemePrefsPayload {
    theme: String,
}

#[derive(Deserialize)]
struct AccountPreviewRequest {
    connection_id: String,
    machine_id: String,
    action: String,
}

#[derive(Deserialize)]
struct AccountApplyRequest {
    preview_id: String,
}

#[derive(Serialize)]
struct EnrichedConnection {
    #[serde(flatten)]
    base: ConnectionStatus,
    owner_kind: String,
    owner_badge: String,
    writable: bool,
    cli_hint: Option<String>,
}

#[derive(Serialize)]
struct ConsoleStatusResponse {
    local_machine: String,
    checked_at: i64,
    machines: Vec<String>,
    known_machines: Vec<String>,
    connections: Vec<EnrichedConnection>,
}

fn state_dir() -> Result<PathBuf, String> {
    if let Ok(home) = std::env::var("BRIDGET_HOME")
        && !home.trim().is_empty()
    {
        return Ok(PathBuf::from(home));
    }
    let home = std::env::var("HOME").map_err(|_| "variable HOME absente".to_string())?;
    Ok(PathBuf::from(home).join(".cache").join("bridget-core"))
}

pub(crate) fn ui_prefs_path() -> Result<PathBuf, String> {
    Ok(state_dir()?.join("ui-prefs.json"))
}

pub(crate) fn load_theme_pref_from(path: &Path) -> String {
    let raw = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return "system".to_string(),
        Err(err) => {
            log::warn!(
                "ui-prefs illisible ({}), repli sur system : {}",
                path.display(),
                err
            );
            return "system".to_string();
        }
    };
    let parsed: Result<Value, _> = serde_json::from_str(&raw);
    match parsed
        .ok()
        .and_then(|v| v.get("theme").and_then(Value::as_str).map(str::to_string))
        .as_deref()
    {
        Some("light") => "light".to_string(),
        Some("dark") => "dark".to_string(),
        Some("system") => "system".to_string(),
        _ => {
            log::warn!("ui-prefs invalide ({}), repli sur system", path.display());
            "system".to_string()
        }
    }
}

pub(crate) fn save_theme_pref_to(path: &Path, theme: &str) -> Result<(), String> {
    if !matches!(theme, "system" | "light" | "dark") {
        return Err("thème invalide (attendu : system, light ou dark)".to_string());
    }
    let parent = path
        .parent()
        .ok_or_else(|| "dossier parent ui-prefs manquant".to_string())?;
    crate::environment::ensure_private_directory(parent)?;
    let tmp = parent.join(format!(".ui-prefs-{}.tmp", uuid::Uuid::new_v4().simple()));
    let body = serde_json::to_vec_pretty(&json!({ "theme": theme }))
        .map_err(|e| format!("sérialisation ui-prefs impossible : {e}"))?;
    let mut payload = body;
    payload.push(b'\n');
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&tmp)
        .map_err(|e| format!("écriture temporaire ui-prefs impossible : {e}"))?;
    file.write_all(&payload)
        .and_then(|_| file.sync_all())
        .map_err(|e| format!("synchronisation ui-prefs impossible : {e}"))?;
    drop(file);
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("remplacement atomique ui-prefs impossible : {e}")
    })?;
    if let Ok(dir) = File::open(parent) {
        let _ = dir.sync_all();
    }
    Ok(())
}

fn is_worker_registry_path(path: &Path) -> bool {
    let fname = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    if fname.starts_with("worker-") {
        return true;
    }
    let text = path.to_string_lossy();
    text.ends_with("/.config/bridget/agents.json")
}

fn same_path(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(ca), Ok(cb)) => ca == cb,
        _ => a == b,
    }
}

/// Résolution serveur native de l'autorité d'un registre.
///
/// En Lot 1a-0, seul le registre local non-worker correspondant au chemin attesté
/// `attested_default_registry` et dont `machine_scope` correspond à `local_machine`
/// est autorisé en écriture. Tout registre worker (y compris local Mac), maître distant
/// ou chemin `--registry` non attesté demeure strictement en lecture seule.
pub(crate) fn resolve_registry_authority(
    registry_path: &Path,
    machine_id: &str,
    is_local: bool,
    local_machine: &str,
    attested_default_registry: Option<&Path>,
) -> RegistryAuthority {
    if registry_path.as_os_str().is_empty() {
        return RegistryAuthority {
            owner_kind: "indeterminate".to_string(),
            owner_badge: "Propriétaire indéterminé · Lecture seule".to_string(),
            writable: false,
        };
    }
    if !is_local {
        if machine_id == "cartae"
            && registry_path.to_string_lossy() == "/home/moi/.cache/bridget-core/agents.json"
        {
            return RegistryAuthority {
                owner_kind: "master_remote".to_string(),
                owner_badge: "Maître distant · Lecture seule".to_string(),
                writable: false,
            };
        }
        return RegistryAuthority {
            owner_kind: "worker_remote".to_string(),
            owner_badge: format!("Worker {machine_id} · Lecture seule"),
            writable: false,
        };
    }
    if is_worker_registry_path(registry_path) {
        return RegistryAuthority {
            owner_kind: "worker_local".to_string(),
            owner_badge: format!("Worker {machine_id} · Lecture seule"),
            writable: false,
        };
    }
    let Some(default_path) = attested_default_registry else {
        return RegistryAuthority {
            owner_kind: "indeterminate".to_string(),
            owner_badge: "Propriétaire indéterminé · Lecture seule".to_string(),
            writable: false,
        };
    };
    if !same_path(registry_path, default_path) {
        return RegistryAuthority {
            owner_kind: "indeterminate".to_string(),
            owner_badge: "Propriétaire indéterminé · Lecture seule".to_string(),
            writable: false,
        };
    }
    let Ok(raw) = crate::registry::read_private_registry(registry_path) else {
        return RegistryAuthority {
            owner_kind: "indeterminate".to_string(),
            owner_badge: "Propriétaire indéterminé · Lecture seule".to_string(),
            writable: false,
        };
    };
    let Ok(reg) = AgentRegistry::from_json(&raw, registry_path) else {
        return RegistryAuthority {
            owner_kind: "indeterminate".to_string(),
            owner_badge: "Propriétaire indéterminé · Lecture seule".to_string(),
            writable: false,
        };
    };
    let scope_ok = crate::client_compatibility::machine_scope(&reg, registry_path, local_machine)
        .is_ok_and(|scope| scope == local_machine);
    if !scope_ok {
        return RegistryAuthority {
            owner_kind: "indeterminate".to_string(),
            owner_badge: "Propriétaire indéterminé · Lecture seule".to_string(),
            writable: false,
        };
    }
    RegistryAuthority {
        owner_kind: "master_local".to_string(),
        owner_badge: "Maître (registre)".to_string(),
        writable: true,
    }
}

fn build_cli_hint(conn: &ConnectionStatus, authority: &RegistryAuthority) -> Option<String> {
    if conn.registry_path.is_empty() || !valid_machine_id(&conn.machine_id) {
        return None;
    }
    if authority.owner_kind == "indeterminate" {
        return None;
    }
    let reg = crate::accounts_connect::sh_quote(&conn.registry_path);
    if conn.auth_state == "unauthenticated" {
        return Some(format!(
            "bridget accounts login --connection {} --machine {} --registry {}",
            conn.connection_id, conn.machine_id, reg
        ));
    }
    if conn.managed
        && !authority.writable
        && let Some(raw_id) = conn.connection_id.strip_prefix("account-")
    {
        let verb = if conn.enabled { "disable" } else { "enable" };
        return Some(format!(
            "bridget accounts {verb} --id {raw_id} --registry {reg}"
        ));
    }
    Some(format!("bridget accounts list --registry {reg}"))
}

fn security_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    headers
}

fn check_host(headers: &HeaderMap, expected_host: &str) -> bool {
    headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|h| h == expected_host)
}

fn extract_cookie<'a>(headers: &'a HeaderMap, cookie_name: &str) -> Option<&'a str> {
    let prefix = format!("{cookie_name}=");
    headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|raw| {
            raw.split(';')
                .find_map(|part| part.trim().strip_prefix(&prefix))
        })
}

fn check_private_auth(
    state: &ConsoleState,
    headers: &HeaderMap,
    is_mutation: bool,
) -> Result<(), (StatusCode, &'static str, &'static str)> {
    if !check_host(headers, &state.expected_host) {
        return Err((
            StatusCode::FORBIDDEN,
            "invalid_host",
            "En-tête Host non autorisé pour cette console locale.",
        ));
    }
    if is_mutation {
        let origin_ok = headers
            .get(header::ORIGIN)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|o| o == state.expected_origin);
        if !origin_ok {
            return Err((
                StatusCode::FORBIDDEN,
                "invalid_origin",
                "En-tête Origin absent ou étranger pour cette mutation locale.",
            ));
        }
    }
    if Instant::now() >= state.expires_at {
        return Err((
            StatusCode::UNAUTHORIZED,
            "session_expired",
            "Session locale expirée : relancez bridget accounts ui.",
        ));
    }
    let cookie_ok =
        extract_cookie(headers, &state.cookie_name).is_some_and(|val| val == state.session_id);
    if !cookie_ok {
        return Err((
            StatusCode::UNAUTHORIZED,
            "session_required",
            "Session locale absente ou invalide.",
        ));
    }
    let token_ok = headers
        .get("x-bridget-token")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|t| t == state.client_token);
    if !token_ok {
        return Err((
            StatusCode::UNAUTHORIZED,
            "token_required",
            "Jeton d'instance x-bridget-token requis.",
        ));
    }
    Ok(())
}

fn json_with_headers(status: StatusCode, body: Value) -> Response {
    (status, security_headers(), Json(body)).into_response()
}

fn error_response(status: StatusCode, code: &str, message: &str, ui_state: &str) -> Response {
    json_with_headers(
        status,
        json!({
            "error": code,
            "message": message,
            "ui_state": ui_state,
        }),
    )
}

async fn route_index(
    State(state): State<ConsoleState>,
    headers: HeaderMap,
    Query(query): Query<BootstrapQuery>,
) -> Response {
    if !check_host(&headers, &state.expected_host) {
        return (
            StatusCode::FORBIDDEN,
            security_headers(),
            Html("Accès refusé : hôte non autorisé.".to_string()),
        )
            .into_response();
    }
    if let Some(provided) = query.token.as_deref() {
        let matched = {
            let mut guard = state.launch_token.lock().unwrap_or_else(|e| e.into_inner());
            if guard.as_deref() == Some(provided) {
                *guard = None;
                true
            } else {
                false
            }
        };
        if !matched || Instant::now() >= state.expires_at {
            return (
                StatusCode::UNAUTHORIZED,
                security_headers(),
                Html("Jeton de lancement invalide, expiré ou déjà consommé.".to_string()),
            )
                .into_response();
        }
        let mut resp_headers = security_headers();
        if let Ok(cookie) = HeaderValue::from_str(&format!(
            "{}={}; HttpOnly; SameSite=Strict; Path=/",
            state.cookie_name, state.session_id
        )) {
            resp_headers.insert(header::SET_COOKIE, cookie);
        }
        if let Ok(loc) = HeaderValue::from_str(&format!("/#bt={}", state.client_token)) {
            resp_headers.insert(header::LOCATION, loc);
        }
        return (StatusCode::SEE_OTHER, resp_headers, "").into_response();
    }
    let theme = load_theme_pref_from(&state.prefs_path);
    let page = INDEX_HTML.replace("__BRIDGET_THEME__", &theme);
    (StatusCode::OK, security_headers(), Html(page)).into_response()
}

async fn serve_asset(
    state: ConsoleState,
    headers: HeaderMap,
    content_type: &'static str,
    body: &'static str,
) -> Response {
    if !check_host(&headers, &state.expected_host) {
        return (StatusCode::FORBIDDEN, security_headers(), "forbidden").into_response();
    }
    let mut h = security_headers();
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    (StatusCode::OK, h, body).into_response()
}

async fn route_styles(State(state): State<ConsoleState>, headers: HeaderMap) -> Response {
    serve_asset(state, headers, "text/css; charset=utf-8", STYLES_CSS).await
}

async fn route_app_js(State(state): State<ConsoleState>, headers: HeaderMap) -> Response {
    serve_asset(
        state,
        headers,
        "application/javascript; charset=utf-8",
        APP_JS,
    )
    .await
}

async fn route_api_js(State(state): State<ConsoleState>, headers: HeaderMap) -> Response {
    serve_asset(
        state,
        headers,
        "application/javascript; charset=utf-8",
        API_JS,
    )
    .await
}

async fn route_accounts_js(State(state): State<ConsoleState>, headers: HeaderMap) -> Response {
    serve_asset(
        state,
        headers,
        "application/javascript; charset=utf-8",
        VIEW_ACCOUNTS_JS,
    )
    .await
}

async fn route_api_status(State(state): State<ConsoleState>, headers: HeaderMap) -> Response {
    if let Err((status, code, msg)) = check_private_auth(&state, &headers, false) {
        return error_response(status, code, msg, &format!("Refusé : {msg}"));
    }
    let registry = state.registry_override.clone();
    let attested = state.attested_default_registry.clone();
    let res = tokio::task::spawn_blocking(move || -> Result<ConsoleStatusResponse, String> {
        let fleet: FleetAuthStatus = collect_fleet_status(registry.as_deref())?;
        let known_machines = if fleet.known_machines.is_empty() {
            fleet.machines.clone()
        } else {
            fleet.known_machines.clone()
        };
        let connections = fleet
            .connections
            .into_iter()
            .map(|c| {
                let auth = resolve_registry_authority(
                    Path::new(&c.registry_path),
                    &c.machine_id,
                    c.is_local,
                    &fleet.local_machine,
                    attested.as_deref(),
                );
                let cli_hint = build_cli_hint(&c, &auth);
                EnrichedConnection {
                    base: c,
                    owner_kind: auth.owner_kind,
                    owner_badge: auth.owner_badge,
                    writable: auth.writable,
                    cli_hint,
                }
            })
            .collect();
        Ok(ConsoleStatusResponse {
            local_machine: fleet.local_machine,
            checked_at: fleet.checked_at,
            machines: fleet.machines,
            known_machines,
            connections,
        })
    })
    .await;
    match res {
        Ok(Ok(payload)) => json_with_headers(
            StatusCode::OK,
            serde_json::to_value(payload).unwrap_or_default(),
        ),
        Ok(Err(e)) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "status_unavailable",
            &e,
            &format!("Refusé : {e}"),
        ),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "status_join_error",
            &e.to_string(),
            "Refusé : lecture d'état interrompue",
        ),
    }
}

async fn route_api_ui_prefs(
    State(state): State<ConsoleState>,
    headers: HeaderMap,
    Json(req): Json<ThemePrefsPayload>,
) -> Response {
    if let Err((status, code, msg)) = check_private_auth(&state, &headers, true) {
        return error_response(status, code, msg, &format!("Refusé : {msg}"));
    }
    let path = state.prefs_path.clone();
    let theme = req.theme.clone();
    let res = tokio::task::spawn_blocking(move || save_theme_pref_to(&path, &theme)).await;
    match res {
        Ok(Ok(())) => json_with_headers(
            StatusCode::OK,
            json!({
                "theme": req.theme,
                "ui_state": "Enregistré et relu"
            }),
        ),
        Ok(Err(e)) => error_response(
            StatusCode::BAD_REQUEST,
            "invalid_theme",
            &e,
            &format!("Refusé : {e}"),
        ),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "prefs_write_error",
            &e.to_string(),
            "Refusé : écriture des préférences interrompue",
        ),
    }
}

async fn route_api_accounts_preview(
    State(state): State<ConsoleState>,
    headers: HeaderMap,
    Json(req): Json<AccountPreviewRequest>,
) -> Response {
    if let Err((status, code, msg)) = check_private_auth(&state, &headers, true) {
        return error_response(status, code, msg, &format!("Refusé : {msg}"));
    }
    if !matches!(req.action.as_str(), "enable" | "disable") {
        return error_response(
            StatusCode::BAD_REQUEST,
            "action_not_allowed",
            "Seules les actions enable et disable sont ouvertes en Lot 1a-0.",
            "Refusé : action non ouverte dans ce lot",
        );
    }
    let Some(raw_id) = req
        .connection_id
        .strip_prefix("account-")
        .map(str::to_string)
    else {
        return error_response(
            StatusCode::BAD_REQUEST,
            "account_not_managed",
            "Seuls les comptes dédiés (account-*) peuvent être activés ou désactivés.",
            "Refusé : compte principal non géré par cette action",
        );
    };
    let registry_override = state.registry_override.clone();
    let attested = state.attested_default_registry.clone();
    let machine_req = req.machine_id.clone();
    let conn_req = req.connection_id.clone();
    let action = req.action.clone();
    let raw_id_for_worker = raw_id.clone();
    let res = tokio::task::spawn_blocking(
        move || -> Result<(PathBuf, RegistryAuthority, Value), (StatusCode, String, String)> {
            let (local_machine, targets) =
                crate::accounts_connect::load_fleet_connections(registry_override.as_deref())
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "registry_read_failed".into(),
                            e,
                        )
                    })?;
            let target = targets
                .into_iter()
                .find(|t| t.connection_id == conn_req && t.machine_id == machine_req)
                .ok_or_else(|| {
                    (
                        StatusCode::NOT_FOUND,
                        "account_unknown".into(),
                        "Compte introuvable dans le registre courant.".into(),
                    )
                })?;
            let reg_path = PathBuf::from(&target.registry_path);
            let auth = resolve_registry_authority(
                &reg_path,
                &target.machine_id,
                target.is_local,
                &local_machine,
                attested.as_deref(),
            );
            if !auth.writable {
                return Err((
                    StatusCode::FORBIDDEN,
                    "registry_read_only".into(),
                    format!(
                        "Registre en lecture seule ({}) : utiliser la CLI sur l'hôte propriétaire.",
                        auth.owner_badge
                    ),
                ));
            }
            let preview =
                provider_accounts::preview_toggle_from_ui(&reg_path, &action, &raw_id_for_worker)
                    .map_err(|code| {
                    (
                        StatusCode::BAD_REQUEST,
                        code.to_string(),
                        provider_accounts::guidance_message(code).to_string(),
                    )
                })?;
            Ok((reg_path, auth, preview))
        },
    )
    .await;

    match res {
        Ok(Ok((reg_path, auth, preview))) => {
            let expected = preview
                .get("expected")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            let preview_id = uuid::Uuid::new_v4().simple().to_string();
            let bound = BoundPreview {
                session_id: state.session_id.clone(),
                registry_path: reg_path.clone(),
                action: req.action.clone(),
                account_id: raw_id,
                expected_sha256: expected.clone(),
                expires_at: Instant::now() + PREVIEW_TTL,
            };
            {
                let mut map = state.previews.lock().unwrap_or_else(|e| e.into_inner());
                let now = Instant::now();
                map.retain(|_, p| p.expires_at > now);
                map.insert(preview_id.clone(), bound);
            }
            json_with_headers(
                StatusCode::OK,
                json!({
                    "preview_id": preview_id,
                    "action": req.action,
                    "connection_id": req.connection_id,
                    "machine_id": req.machine_id,
                    "registry_path": reg_path.to_string_lossy(),
                    "owner_kind": auth.owner_kind,
                    "owner_badge": auth.owner_badge,
                    "expected_sha256": expected,
                    "fields_touched": ["routing_policy.connections[].enabled", "routing_policy.revision"],
                }),
            )
        }
        Ok(Err((status, code, msg))) => {
            error_response(status, &code, &msg, &format!("Refusé : {msg}"))
        }
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "preview_join_error",
            &e.to_string(),
            "Refusé : calcul d'aperçu interrompu",
        ),
    }
}

async fn route_api_accounts_apply(
    State(state): State<ConsoleState>,
    headers: HeaderMap,
    Json(req): Json<AccountApplyRequest>,
) -> Response {
    if let Err((status, code, msg)) = check_private_auth(&state, &headers, true) {
        return error_response(status, code, msg, &format!("Refusé : {msg}"));
    }
    let bound = {
        let mut map = state.previews.lock().unwrap_or_else(|e| e.into_inner());
        map.remove(&req.preview_id)
    };
    let Some(bound) = bound else {
        return error_response(
            StatusCode::CONFLICT,
            "preview_missing_or_consumed",
            "Aperçu absent ou déjà utilisé : rechargez l'aperçu avant de confirmer.",
            "Conflit : rechargez l'aperçu",
        );
    };
    if bound.session_id != state.session_id || Instant::now() >= bound.expires_at {
        return error_response(
            StatusCode::CONFLICT,
            "preview_expired",
            "Aperçu expiré : rechargez l'aperçu avant de confirmer.",
            "Conflit : rechargez l'aperçu",
        );
    }
    let attested = state.attested_default_registry.clone();
    let res = tokio::task::spawn_blocking(
        move || -> Result<Value, (StatusCode, String, String, String)> {
            let local_machine = crate::build_info::local_host();
            let local_name = if local_machine.is_empty() {
                "local".to_string()
            } else {
                local_machine
            };
            let auth = resolve_registry_authority(
                &bound.registry_path,
                &local_name,
                true,
                &local_name,
                attested.as_deref(),
            );
            if !auth.writable {
                let msg = format!(
                    "Autorité registre perdue ({}) : écriture refusée.",
                    auth.owner_badge
                );
                return Err((
                    StatusCode::FORBIDDEN,
                    "registry_read_only".into(),
                    msg.clone(),
                    format!("Refusé : {msg}"),
                ));
            }
            provider_accounts::apply_toggle_from_ui(
                &bound.registry_path,
                &bound.expected_sha256,
                &bound.action,
                &bound.account_id,
            )
            .map_err(|code| {
                let msg = provider_accounts::guidance_message(code).to_string();
                match code {
                    "account_registry_changed" => (
                        StatusCode::CONFLICT,
                        code.to_string(),
                        msg,
                        "Conflit : rechargez l'aperçu".to_string(),
                    ),
                    "account_registry_publish_unconfirmed" => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        code.to_string(),
                        msg,
                        "Résultat incertain : relisez avant de réessayer".to_string(),
                    ),
                    _ => (
                        StatusCode::BAD_REQUEST,
                        code.to_string(),
                        msg.clone(),
                        format!("Refusé : {msg}"),
                    ),
                }
            })
        },
    )
    .await;

    match res {
        Ok(Ok(mut view)) => {
            view["ui_state"] = json!("Enregistré et relu — prise en compte à confirmer");
            json_with_headers(StatusCode::OK, view)
        }
        Ok(Err((status, code, msg, ui_state))) => error_response(status, &code, &msg, &ui_state),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "apply_join_error",
            &e.to_string(),
            "Résultat incertain : relisez avant de réessayer",
        ),
    }
}

pub(crate) fn build_console_router(state: ConsoleState) -> Router {
    Router::new()
        .route("/", get(route_index))
        .route("/assets/styles.css", get(route_styles))
        .route("/assets/app.js", get(route_app_js))
        .route("/assets/api.js", get(route_api_js))
        .route("/assets/views/accounts.js", get(route_accounts_js))
        .route("/api/status", get(route_api_status))
        .route("/api/ui-prefs", post(route_api_ui_prefs))
        .route("/api/accounts/preview", post(route_api_accounts_preview))
        .route("/api/accounts/apply", post(route_api_accounts_apply))
        .with_state(state)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn new_console_state(
    port: u16,
    launch_token: String,
    registry_override: Option<PathBuf>,
    attested_default_registry: Option<PathBuf>,
    prefs_path: PathBuf,
    ttl: Duration,
) -> ConsoleState {
    ConsoleState {
        expected_host: format!("127.0.0.1:{port}"),
        expected_origin: format!("http://127.0.0.1:{port}"),
        cookie_name: format!("bridget_console_sid_{port}"),
        launch_token: Arc::new(Mutex::new(Some(launch_token))),
        session_id: uuid::Uuid::new_v4().simple().to_string(),
        client_token: uuid::Uuid::new_v4().simple().to_string(),
        expires_at: Instant::now() + ttl,
        registry_override,
        attested_default_registry,
        prefs_path,
        previews: Arc::new(Mutex::new(HashMap::new())),
    }
}

pub(crate) fn cmd_ui(args: &[String]) -> Result<(), String> {
    let mut no_open = false;
    let mut port: u16 = 0;
    let mut registry: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--no-open" => {
                no_open = true;
                i += 1;
            }
            "--port" => {
                port = args
                    .get(i + 1)
                    .ok_or("--port exige un numero")?
                    .parse()
                    .map_err(|_| "--port invalide")?;
                i += 2;
            }
            "--registry" => {
                registry = Some(PathBuf::from(
                    args.get(i + 1).ok_or("--registry exige un chemin")?,
                ));
                i += 2;
            }
            other => return Err(format!("option inconnue : {other}")),
        }
    }

    let prefs_path = ui_prefs_path()?;
    let attested_default = crate::registry::config_path().ok();
    let launch_token = uuid::Uuid::new_v4().simple().to_string();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("runtime tokio indisponible : {e}"))?;

    rt.block_on(async move {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
            .await
            .map_err(|e| format!("ecoute locale impossible : {e}"))?;
        let addr = listener.local_addr().map_err(|e| e.to_string())?;
        let state = new_console_state(
            addr.port(),
            launch_token.clone(),
            registry,
            attested_default,
            prefs_path,
            SESSION_TTL,
        );
        let url = format!("http://127.0.0.1:{}/?token={launch_token}", addr.port());
        println!("Console Bridget Comptes prete sur : {url}");
        println!("Ctrl+C pour fermer cette console locale.");
        if !no_open {
            let open_url = url.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(150)).await;
                open_browser(&open_url);
            });
        }
        axum::serve(listener, build_console_router(state))
            .await
            .map_err(|e| e.to_string())
    })
}

#[cfg(test)]
#[path = "console_ui_tests.rs"]
mod tests;
