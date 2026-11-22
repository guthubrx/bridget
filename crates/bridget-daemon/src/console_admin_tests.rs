use super::*;
use crate::console_ui::{build_console_router, new_console_state};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn raw_http(
    port: u16,
    method: &str,
    path: &str,
    extra_headers: &[(&str, &str)],
    body: Option<&str>,
) -> (u16, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
    let mut req =
        format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n");
    for (k, v) in extra_headers {
        req.push_str(&format!("{k}: {v}\r\n"));
    }
    if let Some(b) = body {
        req.push_str("Content-Type: application/json\r\n");
        req.push_str(&format!("Content-Length: {}\r\n\r\n{b}", b.len()));
    } else {
        req.push_str("\r\n");
    }
    stream.write_all(req.as_bytes()).await.unwrap();
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await.unwrap();
    let text = String::from_utf8_lossy(&buf).into_owned();
    let (head, body_part) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
    let status = head
        .lines()
        .next()
        .unwrap_or("")
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    (status, body_part.to_string())
}

fn temp_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().canonicalize().unwrap().join(format!(
        "bridget-console-admin-{label}-{}",
        uuid::Uuid::new_v4().simple()
    ));
    fs::create_dir_all(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    dir
}

fn write_valid_registry(path: &Path) {
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
                }
            ],
            "model_catalog": {
                "claude-main": [{
                    "model": "claude-sonnet",
                    "supported_efforts": ["low"],
                    "default_effort": "low",
                    "provenance": "declared",
                    "observed_at": null
                }]
            },
            "machine_policies": {
                "local": { "enabled": true, "compilation_capacity": 1 }
            }
        }
    });
    let text = serde_json::to_string_pretty(&reg).unwrap();
    fs::write(path, text.as_bytes()).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    AgentRegistry::from_json(&text, path).expect("registre fixture valide");
}

async fn spawn_admin_server(
    reg: PathBuf,
    attested: Option<PathBuf>,
    prefs: PathBuf,
) -> (u16, String, String) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let launch = uuid::Uuid::new_v4().simple().to_string();
    let state = new_console_state(
        port,
        launch.clone(),
        Some(reg),
        attested,
        prefs,
        Duration::from_secs(3600),
    );
    let cookie = format!("bridget_console_sid_{port}={}", state.session_id_ref());
    let token = state.client_token_ref().to_string();
    let router = build_console_router(state);
    tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });
    (port, cookie, token)
}

#[test]
fn spec311_ssh_config_upsert_is_atomic_0600_and_rejects_injection() {
    let dir = temp_dir("ssh");
    let cfg = dir.join(".ssh/config");
    upsert_ssh_config_file(&cfg, "macbook-m1", "macbook-pro.home", "moi", 22, true).unwrap();
    let mode = fs::metadata(&cfg).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    let parsed = parse_ssh_config_hosts(&fs::read_to_string(&cfg).unwrap());
    assert_eq!(parsed["macbook-m1"]["hostname"], "macbook-pro.home");
    assert_eq!(parsed["macbook-m1"]["addressfamily"], "inet");

    // Mise à jour remplace l'ancien bloc sans doublon.
    upsert_ssh_config_file(&cfg, "macbook-m1", "192.168.1.51", "moi", 2222, false).unwrap();
    let text = fs::read_to_string(&cfg).unwrap();
    assert_eq!(text.matches("Host macbook-m1").count(), 1);
    let parsed2 = parse_ssh_config_hosts(&text);
    assert_eq!(parsed2["macbook-m1"]["hostname"], "192.168.1.51");
    assert_eq!(parsed2["macbook-m1"]["port"], "2222");

    // Rejet d'injection.
    assert!(upsert_ssh_config_file(&cfg, "bad host", "1.2.3.4", "moi", 22, true).is_err());
    assert!(
        upsert_ssh_config_file(&cfg, "ok", "1.2.3.4\nProxyCommand evil", "moi", 22, true).is_err()
    );
    assert!(upsert_ssh_config_file(&cfg, "ok", "1.2.3.4", "moi;rm", 22, true).is_err());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn spec311_cli_catalog_is_closed_and_builds_official_scripts() {
    for cid in ["claude", "codex", "antigravity", "cursor", "gemini"] {
        assert!(catalog_item(cid).is_some());
        let install = build_client_deploy_script(cid, false).unwrap();
        let update = build_client_deploy_script(cid, true).unwrap();
        assert!(!install.is_empty() && !update.is_empty());
    }
    assert!(catalog_item("arbitrary-bin").is_none());
    assert!(build_client_deploy_script("rm -rf /", false).is_err());
}

#[tokio::test]
async fn spec311_admin_routes_require_hardened_auth_and_reject_bad_inputs() {
    let dir = temp_dir("auth");
    let reg = dir.join("agents.json");
    let prefs = dir.join("ui-prefs.json");
    write_valid_registry(&reg);
    let (port, cookie, token) = spawn_admin_server(reg.clone(), Some(reg), prefs).await;
    let origin = format!("http://127.0.0.1:{port}");

    // Sans x-bridget-token -> 401
    let (st, _) = raw_http(port, "GET", "/api/settings", &[("Cookie", &cookie)], None).await;
    assert_eq!(st, 401);
    let (st, _) = raw_http(port, "GET", "/api/machines", &[("Cookie", &cookie)], None).await;
    assert_eq!(st, 401);
    let (st, _) = raw_http(
        port,
        "GET",
        "/api/clients/deploy/poll?job=missing",
        &[("Cookie", &cookie)],
        None,
    )
    .await;
    assert_eq!(st, 401);

    // Job inconnu authentifié -> 404
    let (st, _) = raw_http(
        port,
        "GET",
        "/api/clients/deploy/poll?job=missing",
        &[("Cookie", &cookie), ("x-bridget-token", &token)],
        None,
    )
    .await;
    assert_eq!(st, 404);

    // POST sans Origin -> 403
    let (st, _) = raw_http(
        port,
        "POST",
        "/api/clients/deploy",
        &[("Cookie", &cookie), ("x-bridget-token", &token)],
        Some(r#"{"machine_id":"local","client_id":"claude","mode":"install"}"#),
    )
    .await;
    assert_eq!(st, 403);

    // Client inconnu -> 400
    let (st, _) = raw_http(
        port,
        "POST",
        "/api/clients/deploy",
        &[
            ("Origin", &origin),
            ("Cookie", &cookie),
            ("x-bridget-token", &token),
        ],
        Some(r#"{"machine_id":"local","client_id":"inconnu","mode":"install"}"#),
    )
    .await;
    assert_eq!(st, 400);

    // Intervalle hors bornes -> 400
    let (st, _) = raw_http(
        port,
        "POST",
        "/api/settings/automanaged",
        &[
            ("Origin", &origin),
            ("Cookie", &cookie),
            ("x-bridget-token", &token),
        ],
        Some(r#"{"interval_secs":10,"propagate_remotes":false}"#),
    )
    .await;
    assert_eq!(st, 400);

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn spec311_routing_preview_apply_readback_conflict_and_readonly() {
    let dir = temp_dir("routing");
    let reg = dir.join("agents.json");
    let other_reg = dir.join("other.json");
    let prefs = dir.join("ui-prefs.json");
    write_valid_registry(&reg);
    write_valid_registry(&other_reg);

    // 1. Registre non attesté -> 403
    let (ro_port, ro_cookie, ro_tok) =
        spawn_admin_server(other_reg, Some(reg.clone()), prefs.clone()).await;
    let ro_origin = format!("http://127.0.0.1:{ro_port}");
    let (st, _) = raw_http(
        ro_port,
        "POST",
        "/api/settings/routing/preview",
        &[
            ("Origin", &ro_origin),
            ("Cookie", &ro_cookie),
            ("x-bridget-token", &ro_tok),
        ],
        Some(r#"{"kind":"set_machine_capacity","machine_id":"local","capacity":3}"#),
    )
    .await;
    assert_eq!(st, 403);

    // 2. Registre attesté -> preview + apply + relecture disque
    let (port, cookie, token) = spawn_admin_server(reg.clone(), Some(reg.clone()), prefs).await;
    let origin = format!("http://127.0.0.1:{port}");
    let hdrs = [
        ("Origin", origin.as_str()),
        ("Cookie", cookie.as_str()),
        ("x-bridget-token", token.as_str()),
    ];
    let (st, body) = raw_http(
        port,
        "POST",
        "/api/settings/routing/preview",
        &hdrs,
        Some(r#"{"kind":"set_machine_capacity","machine_id":"local","capacity":4}"#),
    )
    .await;
    assert_eq!(st, 200, "body: {body}");
    let prev: Value = serde_json::from_str(&body).unwrap();
    let pid = prev["preview_id"].as_str().unwrap().to_string();

    let (st, body) = raw_http(
        port,
        "POST",
        "/api/settings/routing/apply",
        &hdrs,
        Some(&json!({ "preview_id": pid }).to_string()),
    )
    .await;
    assert_eq!(st, 200, "body: {body}");
    let saved: Value = serde_json::from_str(&fs::read_to_string(&reg).unwrap()).unwrap();
    assert_eq!(
        saved["routing_policy"]["machine_policies"]["local"]["compilation_capacity"],
        4
    );
    assert_eq!(saved["routing_policy"]["revision"], 2);

    // 3. Conflit 409 si le registre change entre preview et apply
    let (st, body) = raw_http(
        port,
        "POST",
        "/api/settings/routing/preview",
        &hdrs,
        Some(r#"{"kind":"set_global_enabled","enabled":false}"#),
    )
    .await;
    assert_eq!(st, 200);
    let pid2 = serde_json::from_str::<Value>(&body).unwrap()["preview_id"]
        .as_str()
        .unwrap()
        .to_string();
    let mut raw = fs::read_to_string(&reg).unwrap();
    raw.push('\n');
    fs::write(&reg, raw).unwrap();
    fs::set_permissions(&reg, fs::Permissions::from_mode(0o600)).unwrap();

    let (st, body) = raw_http(
        port,
        "POST",
        "/api/settings/routing/apply",
        &hdrs,
        Some(&json!({ "preview_id": pid2 }).to_string()),
    )
    .await;
    assert_eq!(st, 409, "body: {body}");

    let _ = fs::remove_dir_all(&dir);
}
