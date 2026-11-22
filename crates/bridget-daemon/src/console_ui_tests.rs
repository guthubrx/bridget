use super::*;
use std::os::unix::fs::PermissionsExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

struct RawHttpResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: String,
}

async fn spawn_test_server(
    registry_override: Option<PathBuf>,
    attested_default_registry: Option<PathBuf>,
    prefs_path: PathBuf,
    ttl: Duration,
) -> (u16, String, ConsoleState) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let launch_token = uuid::Uuid::new_v4().simple().to_string();
    let state = new_console_state(
        port,
        launch_token.clone(),
        registry_override,
        attested_default_registry,
        prefs_path,
        ttl,
    );
    let router = build_console_router(state.clone());
    tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });
    (port, launch_token, state)
}

async fn raw_http(
    port: u16,
    method: &str,
    path: &str,
    extra_headers: &[(&str, &str)],
    body: Option<&str>,
) -> RawHttpResponse {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    let has_host = extra_headers
        .iter()
        .any(|(k, _)| k.eq_ignore_ascii_case("host"));
    let mut req = format!("{method} {path} HTTP/1.1\r\nConnection: close\r\n");
    if !has_host {
        req.push_str(&format!("Host: 127.0.0.1:{port}\r\n"));
    }
    for (k, v) in extra_headers {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    if let Some(b) = body {
        req.push_str("Content-Type: application/json\r\n");
        req.push_str(&format!("Content-Length: {}\r\n", b.len()));
        req.push_str("\r\n");
        req.push_str(b);
    } else {
        req.push_str("\r\n");
    }
    stream.write_all(req.as_bytes()).await.unwrap();
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await.unwrap();
    let text = String::from_utf8_lossy(&buf).into_owned();
    let (head, body_part) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
    let mut lines = head.lines();
    let status_line = lines.next().unwrap_or("");
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    let mut headers = HashMap::new();
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
        }
    }
    RawHttpResponse {
        status,
        headers,
        body: body_part.to_string(),
    }
}

fn temp_private_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().canonicalize().unwrap().join(format!(
        "bridget-console-ui-{label}-{}",
        uuid::Uuid::new_v4().simple()
    ));
    fs::create_dir_all(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    dir
}

fn write_private_registry(path: &Path, enabled: bool) {
    let parent = path.parent().unwrap();
    let accs = parent.join("provider-accounts");
    let pro = accs.join("pro");
    let profile = pro.join("claude");
    let anth = profile.join("anthropic");
    for d in [&accs, &pro, &profile, &anth] {
        fs::create_dir_all(d).unwrap();
        fs::set_permissions(d, fs::Permissions::from_mode(0o700)).unwrap();
    }
    let reg = json!({
        "agents": {
            "claude-main": {
                "protocol": "claude_stream_json",
                "command": "claude",
                "args": [],
                "pass_env": ["CLAUDE_CONFIG_DIR"],
                "capabilities": {
                    "execution_paths": ["claude_stream_json"],
                    "models": { "claude-sonnet": { "efforts": ["low"] } }
                }
            },
            "account-pro": {
                "protocol": "claude_stream_json",
                "command": "claude",
                "args": [],
                "pass_env": [],
                "claude_config_dir": profile.to_string_lossy(),
                "capabilities": {
                    "execution_paths": ["claude_stream_json"],
                    "models": { "claude-sonnet": { "efforts": ["low"] } }
                }
            }
        },
        "routing_policy": {
            "version": 2,
            "revision": 1,
            "enabled": true,
            "connections": [
                {
                    "connection_id": "claude-main",
                    "machine_id": "local",
                    "engine": "claude",
                    "agent_type": "claude-main",
                    "enabled": true
                },
                {
                    "connection_id": "account-pro",
                    "machine_id": "local",
                    "engine": "claude",
                    "agent_type": "account-pro",
                    "enabled": enabled
                }
            ],
            "model_catalog": {
                "claude-main": [{
                    "model": "claude-sonnet",
                    "supported_efforts": ["low"],
                    "default_effort": "low",
                    "provenance": "declared",
                    "observed_at": null
                }],
                "account-pro": [{
                    "model": "claude-sonnet",
                    "supported_efforts": ["low"],
                    "default_effort": "low",
                    "provenance": "declared",
                    "observed_at": null
                }]
            },
            "machine_policies": {
                "local": { "enabled": true }
            }
        }
    });
    let text = serde_json::to_string_pretty(&reg).unwrap();
    fs::write(path, text.as_bytes()).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    // Vérifie que la fixture respecte strictement le schéma du registre.
    AgentRegistry::from_json(&text, path).expect("fixture registre valide");
}

async fn bootstrap_session(port: u16, launch_token: &str) -> (String, String) {
    let resp = raw_http(port, "GET", &format!("/?token={launch_token}"), &[], None).await;
    assert_eq!(resp.status, 303);
    let set_cookie = resp.headers.get("set-cookie").expect("set-cookie").clone();
    let cookie_pair = set_cookie.split(';').next().unwrap().to_string();
    let loc = resp.headers.get("location").expect("location");
    let client_token = loc
        .strip_prefix("/#bt=")
        .expect("fragment token")
        .to_string();
    (cookie_pair, client_token)
}

#[tokio::test]
async fn spec310_bootstrap_exchange_is_one_time_and_shell_leaks_no_token() {
    let dir = temp_private_dir("bootstrap");
    let prefs = dir.join("ui-prefs.json");
    let reg = dir.join("agents.json");
    write_private_registry(&reg, false);

    let (port, launch_token, state) =
        spawn_test_server(Some(reg.clone()), Some(reg), prefs, SESSION_TTL).await;

    let first = raw_http(port, "GET", &format!("/?token={launch_token}"), &[], None).await;
    assert_eq!(first.status, 303);
    assert_eq!(
        first.headers.get("cache-control").map(String::as_str),
        Some("no-store")
    );
    assert_eq!(
        first.headers.get("referrer-policy").map(String::as_str),
        Some("no-referrer")
    );
    let set_cookie = first.headers.get("set-cookie").unwrap();
    assert!(set_cookie.starts_with(&format!("bridget_console_sid_{port}=")));
    assert!(set_cookie.contains("HttpOnly"));
    assert!(set_cookie.contains("SameSite=Strict"));
    assert!(set_cookie.contains("Path=/"));
    assert_eq!(
        first.headers.get("location").map(String::as_str),
        Some(format!("/#bt={}", state.client_token).as_str())
    );

    // Deuxième utilisation du jeton de lancement : refusée.
    let second = raw_http(port, "GET", &format!("/?token={launch_token}"), &[], None).await;
    assert_eq!(second.status, 401);

    // La coquille GET / ne fuit jamais client_token ni de données privées.
    let shell = raw_http(port, "GET", "/", &[], None).await;
    assert_eq!(shell.status, 200);
    assert_eq!(
        shell.headers.get("cache-control").map(String::as_str),
        Some("no-store")
    );
    assert_eq!(
        shell.headers.get("referrer-policy").map(String::as_str),
        Some("no-referrer")
    );
    assert!(!shell.body.contains(&state.client_token));
    assert!(!shell.body.contains("account-pro"));
    assert!(shell.body.contains("data-theme-pref=\"system\""));

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn spec310_host_origin_and_session_expiry_guards() {
    let dir = temp_private_dir("guards");
    let prefs = dir.join("ui-prefs.json");
    let reg = dir.join("agents.json");
    write_private_registry(&reg, false);

    let (port, launch_token, _) = spawn_test_server(
        Some(reg.clone()),
        Some(reg.clone()),
        prefs.clone(),
        SESSION_TTL,
    )
    .await;
    let (cookie, token) = bootstrap_session(port, &launch_token).await;

    // Host étranger refusé 403.
    let bad_host = raw_http(port, "GET", "/", &[("Host", "evil.local:1234")], None).await;
    assert_eq!(bad_host.status, 403);

    // POST sans Origin -> 403.
    let no_origin = raw_http(
        port,
        "POST",
        "/api/ui-prefs",
        &[("Cookie", &cookie), ("x-bridget-token", &token)],
        Some(r#"{"theme":"dark"}"#),
    )
    .await;
    assert_eq!(no_origin.status, 403);

    // POST avec Origin étranger -> 403.
    let bad_origin = raw_http(
        port,
        "POST",
        "/api/ui-prefs",
        &[
            ("Origin", "http://127.0.0.1:9999"),
            ("Cookie", &cookie),
            ("x-bridget-token", &token),
        ],
        Some(r#"{"theme":"dark"}"#),
    )
    .await;
    assert_eq!(bad_origin.status, 403);

    // Session expirée -> 401.
    let (exp_port, _, exp_state) =
        spawn_test_server(Some(reg.clone()), Some(reg), prefs, Duration::ZERO).await;
    let exp_cookie = format!("{}={}", exp_state.cookie_name, exp_state.session_id);
    let expired = raw_http(
        exp_port,
        "GET",
        "/api/status",
        &[
            ("Cookie", &exp_cookie),
            ("x-bridget-token", &exp_state.client_token),
        ],
        None,
    )
    .await;
    assert_eq!(expired.status, 401);
    assert!(expired.body.contains("session_expired"));

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn spec310_two_instances_reject_cookie_without_token_and_cross_port_reuse() {
    let dir = temp_private_dir("two-ports");
    let prefs = dir.join("ui-prefs.json");
    let reg = dir.join("agents.json");
    write_private_registry(&reg, false);

    let (port_a, launch_a, _) = spawn_test_server(
        Some(reg.clone()),
        Some(reg.clone()),
        prefs.clone(),
        SESSION_TTL,
    )
    .await;
    let (port_b, _launch_b, _) =
        spawn_test_server(Some(reg.clone()), Some(reg), prefs, SESSION_TTL).await;
    assert_ne!(port_a, port_b);

    let (cookie_a, token_a) = bootstrap_session(port_a, &launch_a).await;
    let origin_a = format!("http://127.0.0.1:{port_a}");
    let origin_b = format!("http://127.0.0.1:{port_b}");

    // Cookie A seul (sans x-bridget-token) -> 401 sur GET et POST.
    let get_no_tok = raw_http(port_a, "GET", "/api/status", &[("Cookie", &cookie_a)], None).await;
    assert_eq!(get_no_tok.status, 401);

    let post_no_tok = raw_http(
        port_a,
        "POST",
        "/api/ui-prefs",
        &[("Origin", &origin_a), ("Cookie", &cookie_a)],
        Some(r#"{"theme":"light"}"#),
    )
    .await;
    assert_eq!(post_no_tok.status, 401);

    // Cookie A + token A présentés à l'instance B -> 401.
    let cross_b = raw_http(
        port_b,
        "POST",
        "/api/ui-prefs",
        &[
            ("Origin", &origin_b),
            ("Cookie", &cookie_a),
            ("x-bridget-token", &token_a),
        ],
        Some(r#"{"theme":"light"}"#),
    )
    .await;
    assert_eq!(cross_b.status, 401);

    // Cookie A + token A + Origin A sur A -> 200.
    let ok_a = raw_http(
        port_a,
        "POST",
        "/api/ui-prefs",
        &[
            ("Origin", &origin_a),
            ("Cookie", &cookie_a),
            ("x-bridget-token", &token_a),
        ],
        Some(r#"{"theme":"light"}"#),
    )
    .await;
    assert_eq!(ok_a.status, 200);

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn spec310_automanaged_stays_404_and_login_routes_require_hardened_auth() {
    let dir = temp_private_dir("legacy404");
    let prefs = dir.join("ui-prefs.json");
    let reg = dir.join("agents.json");
    write_private_registry(&reg, false);

    let (port, launch_token, _) =
        spawn_test_server(Some(reg.clone()), Some(reg), prefs, SESSION_TTL).await;
    let (cookie, token) = bootstrap_session(port, &launch_token).await;
    let origin = format!("http://127.0.0.1:{port}");

    for (method, path) in [
        ("GET", "/api/automanaged"),
        ("POST", "/api/automanaged/toggle"),
        ("POST", "/api/automanaged/sync"),
    ] {
        let resp = raw_http(
            port,
            method,
            path,
            &[
                ("Origin", &origin),
                ("Cookie", &cookie),
                ("x-bridget-token", &token),
            ],
            if method == "POST" { Some("{}") } else { None },
        )
        .await;
        assert_eq!(resp.status, 404, "route {method} {path} doit retourner 404");
    }

    // Les routes de connexion/préparation exigent Origin (403) et x-bridget-token (401).
    let no_origin = raw_http(
        port,
        "POST",
        "/api/login/cancel",
        &[("Cookie", &cookie), ("x-bridget-token", &token)],
        Some(r#"{"session_id":"missing"}"#),
    )
    .await;
    assert_eq!(no_origin.status, 403);

    let no_token = raw_http(
        port,
        "POST",
        "/api/login/cancel",
        &[("Origin", &origin), ("Cookie", &cookie)],
        Some(r#"{"session_id":"missing"}"#),
    )
    .await;
    assert_eq!(no_token.status, 401);

    let poll_no_token = raw_http(
        port,
        "GET",
        "/api/login/poll?session=missing",
        &[("Cookie", &cookie)],
        None,
    )
    .await;
    assert_eq!(poll_no_token.status, 401);

    let poll_authed = raw_http(
        port,
        "GET",
        "/api/login/poll?session=missing",
        &[("Cookie", &cookie), ("x-bridget-token", &token)],
        None,
    )
    .await;
    assert_eq!(poll_authed.status, 404);

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn spec310_theme_persists_0600_reopens_without_flash_and_corrupt_falls_back() {
    let dir = temp_private_dir("theme");
    let prefs = dir.join("ui-prefs.json");
    let reg = dir.join("agents.json");
    write_private_registry(&reg, false);

    let (port1, launch1, _) = spawn_test_server(
        Some(reg.clone()),
        Some(reg.clone()),
        prefs.clone(),
        SESSION_TTL,
    )
    .await;
    let (cookie1, token1) = bootstrap_session(port1, &launch1).await;
    let origin1 = format!("http://127.0.0.1:{port1}");

    let save = raw_http(
        port1,
        "POST",
        "/api/ui-prefs",
        &[
            ("Origin", &origin1),
            ("Cookie", &cookie1),
            ("x-bridget-token", &token1),
        ],
        Some(r#"{"theme":"light"}"#),
    )
    .await;
    assert_eq!(save.status, 200);
    let mode = fs::metadata(&prefs).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);

    // Réouverture sur un autre port : le thème est injecté serveur dans <html data-theme-pref="light">.
    let (port2, _, _) =
        spawn_test_server(Some(reg.clone()), Some(reg), prefs.clone(), SESSION_TTL).await;
    let html2 = raw_http(port2, "GET", "/", &[], None).await;
    assert_eq!(html2.status, 200);
    assert!(html2.body.contains("data-theme-pref=\"light\""));

    // Fichier corrompu -> repli propre sur system.
    fs::write(&prefs, "{not-valid-json").unwrap();
    assert_eq!(load_theme_pref_from(&prefs), "system");
    let html_corrupt = raw_http(port2, "GET", "/", &[], None).await;
    assert!(html_corrupt.body.contains("data-theme-pref=\"system\""));

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn spec310_readonly_for_local_worker_remote_and_unattested_registry() {
    let dir = temp_private_dir("readonly");
    let prefs = dir.join("ui-prefs.json");
    let default_reg = dir.join("agents.json");
    let worker_reg = dir.join("worker-monordinateur-agents.json");
    let other_reg = dir.join("custom-agents.json");
    write_private_registry(&default_reg, false);
    write_private_registry(&worker_reg, false);
    write_private_registry(&other_reg, false);

    let host = crate::build_info::local_host();
    let local_m = if host.is_empty() {
        "local"
    } else {
        host.as_str()
    };

    let worker_auth =
        resolve_registry_authority(&worker_reg, local_m, true, local_m, Some(&default_reg));
    assert_eq!(worker_auth.owner_kind, "worker_local");
    assert!(!worker_auth.writable);

    let unattested_auth =
        resolve_registry_authority(&other_reg, local_m, true, local_m, Some(&default_reg));
    assert_eq!(unattested_auth.owner_kind, "indeterminate");
    assert!(!unattested_auth.writable);

    let remote_master = resolve_registry_authority(
        Path::new("/home/moi/.cache/bridget-core/agents.json"),
        "cartae",
        false,
        local_m,
        Some(&default_reg),
    );
    assert_eq!(remote_master.owner_kind, "master_remote");
    assert!(!remote_master.writable);

    let remote_worker = resolve_registry_authority(
        Path::new("/home/moi/.config/bridget/agents.json"),
        "sysle2",
        false,
        local_m,
        Some(&default_reg),
    );
    assert_eq!(remote_worker.owner_kind, "worker_remote");
    assert!(!remote_worker.writable);

    // Preview sur registre worker local -> 403.
    let (w_port, w_launch, _) = spawn_test_server(
        Some(worker_reg),
        Some(default_reg.clone()),
        prefs.clone(),
        SESSION_TTL,
    )
    .await;
    let (w_cookie, w_tok) = bootstrap_session(w_port, &w_launch).await;
    let w_origin = format!("http://127.0.0.1:{w_port}");
    let body = json!({
        "connection_id": "account-pro",
        "machine_id": local_m,
        "action": "enable"
    })
    .to_string();
    let w_resp = raw_http(
        w_port,
        "POST",
        "/api/accounts/preview",
        &[
            ("Origin", &w_origin),
            ("Cookie", &w_cookie),
            ("x-bridget-token", &w_tok),
        ],
        Some(&body),
    )
    .await;
    assert_eq!(w_resp.status, 403);
    assert!(w_resp.body.contains("registry_read_only"));

    // Preview sur --registry non attesté -> 403.
    let (u_port, u_launch, _) =
        spawn_test_server(Some(other_reg), Some(default_reg), prefs, SESSION_TTL).await;
    let (u_cookie, u_tok) = bootstrap_session(u_port, &u_launch).await;
    let u_origin = format!("http://127.0.0.1:{u_port}");
    let u_resp = raw_http(
        u_port,
        "POST",
        "/api/accounts/preview",
        &[
            ("Origin", &u_origin),
            ("Cookie", &u_cookie),
            ("x-bridget-token", &u_tok),
        ],
        Some(&body),
    )
    .await;
    assert_eq!(u_resp.status, 403);
    assert!(u_resp.body.contains("registry_read_only"));

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn spec310_preview_apply_toggle_readback_and_conflict_409() {
    let dir = temp_private_dir("toggle");
    let prefs = dir.join("ui-prefs.json");
    let reg = dir.join("agents.json");
    write_private_registry(&reg, false);

    let host = crate::build_info::local_host();
    let local_m = if host.is_empty() {
        "local"
    } else {
        host.as_str()
    };

    let (port, launch, _) =
        spawn_test_server(Some(reg.clone()), Some(reg.clone()), prefs, SESSION_TTL).await;
    let (cookie, token) = bootstrap_session(port, &launch).await;
    let origin = format!("http://127.0.0.1:{port}");
    let headers = [
        ("Origin", origin.as_str()),
        ("Cookie", cookie.as_str()),
        ("x-bridget-token", token.as_str()),
    ];

    // 1. Preview enable
    let prev_resp = raw_http(
        port,
        "POST",
        "/api/accounts/preview",
        &headers,
        Some(
            &json!({
                "connection_id": "account-pro",
                "machine_id": local_m,
                "action": "enable"
            })
            .to_string(),
        ),
    )
    .await;
    assert_eq!(prev_resp.status, 200, "preview body: {}", prev_resp.body);
    let prev_json: Value = serde_json::from_str(&prev_resp.body).unwrap();
    let preview_id = prev_json["preview_id"].as_str().unwrap().to_string();
    assert_eq!(prev_json["owner_kind"], "master_local");

    // 2. Apply enable + relecture disque
    let apply_resp = raw_http(
        port,
        "POST",
        "/api/accounts/apply",
        &headers,
        Some(&json!({ "preview_id": preview_id }).to_string()),
    )
    .await;
    assert_eq!(apply_resp.status, 200, "apply body: {}", apply_resp.body);
    let apply_json: Value = serde_json::from_str(&apply_resp.body).unwrap();
    assert_eq!(apply_json["readback_enabled"], true);
    assert_eq!(
        apply_json["ui_state"],
        "Enregistré et relu — prise en compte à confirmer"
    );

    // Sauvegarde présente
    let backups_dir = dir.join("account-registry-backups");
    assert!(backups_dir.exists());

    // 3. Rejeu du même preview_id -> 409
    let replay = raw_http(
        port,
        "POST",
        "/api/accounts/apply",
        &headers,
        Some(&json!({ "preview_id": preview_id }).to_string()),
    )
    .await;
    assert_eq!(replay.status, 409);

    // 4. Preview disable puis modification concurrente -> 409 Conflit
    let prev2 = raw_http(
        port,
        "POST",
        "/api/accounts/preview",
        &headers,
        Some(
            &json!({
                "connection_id": "account-pro",
                "machine_id": local_m,
                "action": "disable"
            })
            .to_string(),
        ),
    )
    .await;
    assert_eq!(prev2.status, 200);
    let prev2_id = serde_json::from_str::<Value>(&prev2.body).unwrap()["preview_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Modification externe entre preview et apply (ajout d'un saut de ligne)
    let mut raw = fs::read_to_string(&reg).unwrap();
    raw.push('\n');
    fs::write(&reg, raw).unwrap();
    fs::set_permissions(&reg, fs::Permissions::from_mode(0o600)).unwrap();

    let conflict = raw_http(
        port,
        "POST",
        "/api/accounts/apply",
        &headers,
        Some(&json!({ "preview_id": prev2_id }).to_string()),
    )
    .await;
    assert_eq!(conflict.status, 409, "conflict body: {}", conflict.body);
    let conflict_json: Value = serde_json::from_str(&conflict.body).unwrap();
    assert_eq!(conflict_json["error"], "account_registry_changed");
    assert_eq!(conflict_json["ui_state"], "Conflit : rechargez l'aperçu");

    // 5. Recharger aperçu disable + appliquer -> readback_enabled == false
    let prev3 = raw_http(
        port,
        "POST",
        "/api/accounts/preview",
        &headers,
        Some(
            &json!({
                "connection_id": "account-pro",
                "machine_id": local_m,
                "action": "disable"
            })
            .to_string(),
        ),
    )
    .await;
    assert_eq!(prev3.status, 200);
    let prev3_id = serde_json::from_str::<Value>(&prev3.body).unwrap()["preview_id"]
        .as_str()
        .unwrap()
        .to_string();
    let apply3 = raw_http(
        port,
        "POST",
        "/api/accounts/apply",
        &headers,
        Some(&json!({ "preview_id": prev3_id }).to_string()),
    )
    .await;
    assert_eq!(apply3.status, 200);
    let apply3_json: Value = serde_json::from_str(&apply3.body).unwrap();
    assert_eq!(apply3_json["readback_enabled"], false);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn spec310_ui_line_budget_respected() {
    let ui_lines = INDEX_HTML.lines().count()
        + STYLES_CSS.lines().count()
        + APP_JS.lines().count()
        + API_JS.lines().count()
        + VIEW_ACCOUNTS_JS.lines().count()
        + VIEW_MACHINES_JS.lines().count()
        + VIEW_SETTINGS_JS.lines().count();
    assert!(
        ui_lines <= 1800,
        "budget UI dépassé : {ui_lines} lignes (max 1800 pour 3 onglets)"
    );
}
