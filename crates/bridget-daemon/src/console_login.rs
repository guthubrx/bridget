//! Flux interactif de connexion OAuth et préparation de comptes dédiés
//! pour la console locale `bridget accounts ui` (sous garde de session durcie).

use crate::accounts_connect::{
    ConnectionTarget, build_login_shell_command, build_ssh_login_args, enable_isolated_account,
    extract_auth_url, extract_logged_in_account, find_target, load_fleet_connections, open_browser,
    prepare_isolated_account, strip_ansi, valid_machine_id,
};
use crate::console_ui::{ConsoleState, check_private_auth, error_response, json_with_headers};
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Json, Response};
use serde::Deserialize;
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const MAX_SESSION_LOG_BYTES: usize = 256 * 1024;

pub(crate) struct LoginSession {
    owner_session_id: String,
    engine: String,
    log: String,
    auth_url: Option<String>,
    browser_opened: bool,
    open_in_browser: bool,
    code_sent: bool,
    code_auto_sent: bool,
    logged_in_as: Option<String>,
    running: bool,
    exit_code: Option<i32>,
    stdin: Option<ChildStdin>,
    child: Arc<Mutex<Option<Child>>>,
    enable_on_success: Option<ConnectionTarget>,
    auto_enabled: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
pub(crate) struct StartLoginRequest {
    connection_id: String,
    machine_id: String,
    #[serde(default)]
    logout_first: bool,
    #[serde(default)]
    enable_on_success: bool,
    #[serde(default = "default_true")]
    open_in_browser: bool,
}

#[derive(Deserialize)]
pub(crate) struct LoginPollQuery {
    session: Option<String>,
    offset: Option<usize>,
}

#[derive(Deserialize)]
pub(crate) struct InputLoginRequest {
    session_id: String,
    text: String,
}

#[derive(Deserialize)]
pub(crate) struct CancelLoginRequest {
    session_id: String,
}

#[derive(Deserialize)]
pub(crate) struct PrepareRequest {
    from_connection_id: String,
    machine_id: String,
    new_id: String,
}

#[cfg(target_os = "macos")]
fn read_local_clipboard() -> Option<String> {
    let output = Command::new("/usr/bin/pbpaste")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

#[cfg(target_os = "macos")]
fn clipboard_fingerprint(text: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

#[cfg(target_os = "macos")]
fn start_clipboard_watcher(session: Arc<Mutex<LoginSession>>, initial_fp: Option<String>) {
    use crate::accounts_connect::looks_like_oauth_clipboard_code;
    std::thread::spawn(move || {
        let mut last_fp = initial_fp;
        for _ in 0..500 {
            std::thread::sleep(Duration::from_millis(600));
            let (running, code_sent, engine) = {
                let guard = session.lock().unwrap_or_else(|e| e.into_inner());
                (guard.running, guard.code_sent, guard.engine.clone())
            };
            if !running || code_sent {
                break;
            }
            let Some(clip) = read_local_clipboard() else {
                continue;
            };
            let fp = clipboard_fingerprint(&clip);
            if last_fp.as_deref() == Some(fp.as_str()) {
                continue;
            }
            last_fp = Some(fp);
            if looks_like_oauth_clipboard_code(&engine, &clip) {
                let payload = format!("{}\n", clip.trim());
                let mut guard = session.lock().unwrap_or_else(|e| e.into_inner());
                if !guard.running || guard.code_sent {
                    break;
                }
                if let Some(stdin) = guard.stdin.as_mut()
                    && stdin
                        .write_all(payload.as_bytes())
                        .and_then(|_| stdin.flush())
                        .is_ok()
                {
                    guard.code_sent = true;
                    guard.code_auto_sent = true;
                    guard.log.push_str(
                        "\n[Bridget] Code OAuth détecté dans le presse-papiers et envoyé automatiquement.\n",
                    );
                    break;
                }
            }
        }
    });
}

#[cfg(not(target_os = "macos"))]
fn start_clipboard_watcher(_session: Arc<Mutex<LoginSession>>, _initial_fp: Option<String>) {}

fn spawn_pty_login(target: &ConnectionTarget, logout_first: bool) -> Result<Child, String> {
    let login_cmd = build_login_shell_command(target, logout_first);
    let mut command = if target.is_local {
        #[cfg(target_os = "macos")]
        {
            let mut c = Command::new("/usr/bin/script");
            c.args(["-q", "/dev/null", "bash", "-lc", &login_cmd]);
            c
        }
        #[cfg(not(target_os = "macos"))]
        {
            let mut c = Command::new("/usr/bin/script");
            c.args(["-q", "-e", "-c", &login_cmd, "/dev/null"]);
            c
        }
    } else {
        if !valid_machine_id(&target.machine_id) {
            return Err("machine cible invalide".into());
        }
        let ssh_args = build_ssh_login_args(target, login_cmd);
        #[cfg(target_os = "macos")]
        {
            let mut c = Command::new("/usr/bin/script");
            c.args(["-q", "/dev/null", "ssh"]);
            c.args(&ssh_args);
            c
        }
        #[cfg(not(target_os = "macos"))]
        {
            let mut c = Command::new("ssh");
            c.args(&ssh_args);
            c
        }
    };
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("impossible de lancer la session de connexion : {e}"))
}

fn pump_stream<R: std::io::Read>(reader: R, session: Arc<Mutex<LoginSession>>) {
    let mut buf = BufReader::new(reader);
    let mut line = Vec::new();
    loop {
        line.clear();
        match buf.read_until(b'\n', &mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let text = strip_ansi(&String::from_utf8_lossy(&line));
                let detected_url = extract_auth_url(&text);
                let detected_account = extract_logged_in_account(&text);
                let mut url_to_open = None;
                let mut open_via_os = false;
                let mut watch_engine = false;
                {
                    let mut guard = session.lock().unwrap_or_else(|e| e.into_inner());
                    if guard.log.len() < MAX_SESSION_LOG_BYTES {
                        guard.log.push_str(&text);
                    }
                    if let Some(acc) = detected_account {
                        guard.logged_in_as = Some(acc);
                    }
                    if let Some(url) = detected_url {
                        guard.auth_url = Some(url.clone());
                        if !guard.browser_opened {
                            guard.browser_opened = true;
                            url_to_open = Some(url);
                            open_via_os = guard.open_in_browser;
                            watch_engine =
                                matches!(guard.engine.as_str(), "antigravity" | "claude");
                        }
                    }
                }
                if let Some(url) = url_to_open {
                    if watch_engine {
                        #[cfg(target_os = "macos")]
                        let initial_fp = read_local_clipboard().map(|c| clipboard_fingerprint(&c));
                        #[cfg(not(target_os = "macos"))]
                        let initial_fp = None;
                        start_clipboard_watcher(session.clone(), initial_fp);
                    }
                    if open_via_os {
                        open_browser(&url);
                    }
                }
            }
        }
    }
}

pub(crate) async fn route_api_prepare(
    State(state): State<ConsoleState>,
    headers: HeaderMap,
    Json(req): Json<PrepareRequest>,
) -> Response {
    if let Err((status, code, msg)) = check_private_auth(&state, &headers, true) {
        return error_response(status, code, msg, &format!("Refusé : {msg}"));
    }
    let registry = state.registry_override_ref();
    let res = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let (_, targets) = load_fleet_connections(registry.as_deref())?;
        let source = find_target(&targets, &req.from_connection_id, Some(&req.machine_id))?;
        let new_conn_id = prepare_isolated_account(&source, &req.new_id)?;
        Ok(json!({
            "status": "prepared",
            "connection_id": new_conn_id,
            "machine_id": source.machine_id,
            "engine": source.engine
        }))
    })
    .await;
    match res {
        Ok(Ok(val)) => json_with_headers(StatusCode::OK, val),
        Ok(Err(e)) => error_response(
            StatusCode::BAD_REQUEST,
            "prepare_failed",
            &e,
            &format!("Refusé : {e}"),
        ),
        Err(e) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "prepare_join_error",
            &e.to_string(),
            "Refusé : préparation interrompue",
        ),
    }
}

pub(crate) async fn route_api_login_start(
    State(state): State<ConsoleState>,
    headers: HeaderMap,
    Json(req): Json<StartLoginRequest>,
) -> Response {
    if let Err((status, code, msg)) = check_private_auth(&state, &headers, true) {
        return error_response(status, code, msg, &format!("Refusé : {msg}"));
    }
    let registry = state.registry_override_ref();
    let target_res = tokio::task::spawn_blocking(move || {
        let (_, targets) = load_fleet_connections(registry.as_deref())?;
        find_target(&targets, &req.connection_id, Some(&req.machine_id))
    })
    .await;
    let target = match target_res {
        Ok(Ok(t)) => t,
        Ok(Err(e)) => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "target_unknown",
                &e,
                &format!("Refusé : {e}"),
            );
        }
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "target_join_error",
                &e.to_string(),
                "Refusé : recherche de compte interrompue",
            );
        }
    };

    let mut child = match spawn_pty_login(&target, req.logout_first) {
        Ok(c) => c,
        Err(e) => {
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "login_spawn_failed",
                &e,
                &format!("Refusé : {e}"),
            );
        }
    };
    let stdin = child.stdin.take();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let child_handle = Arc::new(Mutex::new(Some(child)));

    let session_id = uuid::Uuid::new_v4().to_string();
    let session = Arc::new(Mutex::new(LoginSession {
        owner_session_id: state.session_id_ref().to_string(),
        engine: target.engine.clone(),
        log: format!(
            "Session ouverte pour {} ({}) sur {}...\n",
            target.connection_id, target.engine, target.machine_id
        ),
        auth_url: None,
        browser_opened: false,
        open_in_browser: req.open_in_browser,
        code_sent: false,
        code_auto_sent: false,
        logged_in_as: None,
        running: true,
        exit_code: None,
        stdin,
        child: child_handle.clone(),
        enable_on_success: req.enable_on_success.then_some(target.clone()),
        auto_enabled: false,
    }));
    {
        let mut map = state.login_sessions_lock();
        if map.len() >= 16 {
            map.retain(|_, s| s.lock().unwrap_or_else(|e| e.into_inner()).running);
        }
        map.insert(session_id.clone(), session.clone());
    }

    if let Some(out) = stdout {
        let s = session.clone();
        std::thread::spawn(move || pump_stream(out, s));
    }
    if let Some(err) = stderr {
        let s = session.clone();
        std::thread::spawn(move || pump_stream(err, s));
    }
    {
        let s = session.clone();
        std::thread::spawn(move || {
            let exit = loop {
                let maybe_status = {
                    let mut guard = child_handle.lock().unwrap_or_else(|e| e.into_inner());
                    match guard.as_mut() {
                        Some(c) => match c.try_wait() {
                            Ok(Some(st)) => {
                                *guard = None;
                                Some(st.code().unwrap_or(-1))
                            }
                            Ok(None) => None,
                            Err(_) => {
                                *guard = None;
                                Some(-1)
                            }
                        },
                        None => Some(-1),
                    }
                };
                if let Some(code) = maybe_status {
                    break code;
                }
                std::thread::sleep(Duration::from_millis(80));
            };
            let enable_target = {
                let mut guard = s.lock().unwrap_or_else(|e| e.into_inner());
                guard.running = false;
                guard.exit_code = Some(exit);
                guard.stdin = None;
                if exit == 0 {
                    guard.log.push_str("\nConnexion terminée avec succès.\n");
                    guard.enable_on_success.clone()
                } else {
                    guard
                        .log
                        .push_str(&format!("\nSession terminée (code {exit}).\n"));
                    None
                }
            };
            if let Some(t) = enable_target {
                match enable_isolated_account(&t) {
                    Ok(()) => {
                        let mut guard = s.lock().unwrap_or_else(|e| e.into_inner());
                        guard.auto_enabled = true;
                        guard
                            .log
                            .push_str("Compte isolé activé dans le registre.\n");
                    }
                    Err(e) => {
                        let mut guard = s.lock().unwrap_or_else(|e| e.into_inner());
                        guard
                            .log
                            .push_str(&format!("Activation automatique échouée : {e}\n"));
                    }
                }
            }
        });
    }

    json_with_headers(
        StatusCode::OK,
        json!({
            "session_id": session_id,
            "connection_id": target.connection_id,
            "machine_id": target.machine_id,
            "engine": target.engine
        }),
    )
}

pub(crate) async fn route_api_login_poll(
    State(state): State<ConsoleState>,
    headers: HeaderMap,
    Query(query): Query<LoginPollQuery>,
) -> Response {
    if let Err((status, code, msg)) = check_private_auth(&state, &headers, false) {
        return error_response(status, code, msg, &format!("Refusé : {msg}"));
    }
    let Some(session_id) = query.session.as_deref() else {
        return error_response(
            StatusCode::BAD_REQUEST,
            "session_required",
            "Identifiant de session de connexion requis.",
            "Refusé : session absente",
        );
    };
    let session = state.login_sessions_lock().get(session_id).cloned();
    let Some(session) = session else {
        return error_response(
            StatusCode::NOT_FOUND,
            "session_unknown",
            "Session de connexion inconnue.",
            "Refusé : session inconnue",
        );
    };
    let guard = session.lock().unwrap_or_else(|e| e.into_inner());
    if guard.owner_session_id != state.session_id_ref() {
        return error_response(
            StatusCode::FORBIDDEN,
            "session_forbidden",
            "Cette session appartient à une autre instance de console.",
            "Refusé : session étrangère",
        );
    }
    let mut safe_offset = query.offset.unwrap_or(0).min(guard.log.len());
    while safe_offset > 0 && !guard.log.is_char_boundary(safe_offset) {
        safe_offset -= 1;
    }
    let chunk = &guard.log[safe_offset..];
    json_with_headers(
        StatusCode::OK,
        json!({
            "running": guard.running,
            "exit_code": guard.exit_code,
            "auth_url": guard.auth_url,
            "engine": guard.engine,
            "code_sent": guard.code_sent,
            "code_auto_sent": guard.code_auto_sent,
            "logged_in_as": guard.logged_in_as,
            "auto_enabled": guard.auto_enabled,
            "chunk": chunk,
            "next_offset": guard.log.len()
        }),
    )
}

pub(crate) async fn route_api_login_input(
    State(state): State<ConsoleState>,
    headers: HeaderMap,
    Json(req): Json<InputLoginRequest>,
) -> Response {
    if let Err((status, code, msg)) = check_private_auth(&state, &headers, true) {
        return error_response(status, code, msg, &format!("Refusé : {msg}"));
    }
    let session = state.login_sessions_lock().get(&req.session_id).cloned();
    let Some(session) = session else {
        return error_response(
            StatusCode::NOT_FOUND,
            "session_unknown",
            "Session de connexion inconnue.",
            "Refusé : session inconnue",
        );
    };
    let mut guard = session.lock().unwrap_or_else(|e| e.into_inner());
    if guard.owner_session_id != state.session_id_ref() {
        return error_response(
            StatusCode::FORBIDDEN,
            "session_forbidden",
            "Cette session appartient à une autre instance de console.",
            "Refusé : session étrangère",
        );
    }
    let Some(stdin) = guard.stdin.as_mut() else {
        return error_response(
            StatusCode::CONFLICT,
            "session_closed",
            "La session de connexion est déjà fermée.",
            "Conflit : session fermée",
        );
    };
    let payload = format!("{}\n", req.text.trim_end_matches(&['\r', '\n'][..]));
    if let Err(e) = stdin
        .write_all(payload.as_bytes())
        .and_then(|_| stdin.flush())
    {
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "stdin_write_failed",
            &e.to_string(),
            "Refusé : impossible d'envoyer le code",
        );
    }
    guard.code_sent = true;
    json_with_headers(StatusCode::OK, json!({ "status": "sent" }))
}

pub(crate) async fn route_api_login_cancel(
    State(state): State<ConsoleState>,
    headers: HeaderMap,
    Json(req): Json<CancelLoginRequest>,
) -> Response {
    if let Err((status, code, msg)) = check_private_auth(&state, &headers, true) {
        return error_response(status, code, msg, &format!("Refusé : {msg}"));
    }
    let session = state.login_sessions_lock().get(&req.session_id).cloned();
    let Some(session) = session else {
        return error_response(
            StatusCode::NOT_FOUND,
            "session_unknown",
            "Session de connexion inconnue.",
            "Refusé : session inconnue",
        );
    };
    let child_handle = {
        let guard = session.lock().unwrap_or_else(|e| e.into_inner());
        if guard.owner_session_id != state.session_id_ref() {
            return error_response(
                StatusCode::FORBIDDEN,
                "session_forbidden",
                "Cette session appartient à une autre instance de console.",
                "Refusé : session étrangère",
            );
        }
        guard.child.clone()
    };
    {
        let mut child_guard = child_handle.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(child) = child_guard.as_mut() {
            let pid = child.id();
            let _ = Command::new("kill")
                .args(["-TERM", &pid.to_string()])
                .status();
        }
    }
    json_with_headers(StatusCode::OK, json!({ "status": "cancelled" }))
}
