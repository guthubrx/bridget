//! Gate MVP explicite : composants réels, aucune dépendance interne Bridget.
//!
//! Exécution : `BRIDGET_MVP_GATE_BIN=/chemin/absolu/bridget cargo test -p maicie
//! --test mvp_gate -- --ignored --nocapture`.

use maicie::bridget_client::BridgetClient;
use serde_json::{Value, json};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[test]
#[ignore = "gate MVP réel : requiert BRIDGET_MVP_GATE_BIN"]
fn delegation_reelle_est_accusee_et_visible_sans_fausse_correlation_de_reponse() {
    let bridget = PathBuf::from(
        std::env::var_os("BRIDGET_MVP_GATE_BIN")
            .expect("BRIDGET_MVP_GATE_BIN doit désigner le binaire Bridget réel"),
    );
    let fixture = Fixture::new(&bridget);
    let started = Instant::now();
    let mut daemon = fixture.start_daemon();
    fixture.wait_for_agent("mvp-agent");

    let delegated = fixture.maicie(&[
        "delegate",
        "--config",
        fixture.config.to_str().unwrap(),
        "--goal",
        "vérifier une délégation réellement remise",
        "--to",
        "mvp-agent",
        "--duration",
        "courte",
        "--idempotency-key",
        "mvp-gate-1",
        "--json",
    ]);
    assert!(delegated.status.success(), "{:?}", delegated.stderr);
    let delegated: Value = serde_json::from_slice(&delegated.stdout).unwrap();
    let objective_id = delegated["objective_id"].as_str().unwrap();

    let delivered_at = started.elapsed();
    let status = fixture.maicie(&[
        "status",
        "--config",
        fixture.config.to_str().unwrap(),
        "--objective",
        objective_id,
        "--json",
    ]);
    assert!(status.status.success(), "{:?}", status.stderr);
    let status: Value = serde_json::from_slice(&status.stdout).unwrap();
    assert_eq!(status["transport_snapshot"], "unknown");
    assert_eq!(status["coordination"][0]["remises_locales"][0]["state"], "accepted");
    assert_eq!(
        status["coordination"][0]["remises_locales"][0]["issue"]["kind"],
        "accepted"
    );

    let closed = fixture.maicie(&[
        "objective",
        objective_id,
        "close",
        "--reason",
        "gate MVP terminé",
        "--config",
        fixture.config.to_str().unwrap(),
        "--json",
    ]);
    assert!(closed.status.success(), "{:?}", closed.stderr);

    eprintln!(
        "gate MVP: livraison+ACK+status={} ms, clôture={} ms",
        delivered_at.as_millis(),
        started.elapsed().as_millis()
    );
    fixture.stop_agent();
    stop_daemon(&mut daemon);
}

struct Fixture {
    root: PathBuf,
    bridget: PathBuf,
    socket: PathBuf,
    config: PathBuf,
}

impl Fixture {
    fn new(bridget: &Path) -> Self {
        let root = std::env::temp_dir().join(format!("maicie-mvp-gate-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".config/bridget")).unwrap();
        let adapter = root.join("mvp-acp.sh");
        fs::write(
            &adapter,
            "#!/bin/sh\nread initialize\necho '{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"protocolVersion\":1}}'\nread session\necho '{\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"sessionId\":\"mvp\"}}'\nread prompt\necho '{\"jsonrpc\":\"2.0\",\"id\":3,\"result\":{\"stopReason\":\"end_turn\"}}'\n",
        )
        .unwrap();
        fs::set_permissions(&adapter, fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(
            root.join(".config/bridget/agents.json"),
            serde_json::to_vec(&json!({"agents":{"mvp_fixture":{
                "command": adapter,
                "protocol":"acp",
                "permissions":"allow",
                "queue_capacity":1,
                "notify_timeout_secs":5,
                "mcp":{"interactive":"none","acp_session":false}
            }}}))
            .unwrap(),
        )
        .unwrap();
        let socket = root.join(".cache/bridget/bridget.sock");
        let config = root.join("maicie.json");
        fs::write(
            &config,
            serde_json::to_vec(&json!({
                "version":1,
                "bridget_socket":socket,
                "database_path":root.join("maicie.sqlite3"),
                "durations":{"short_secs":30,"normal_secs":60,"long_secs":90},
                "profiles":[{"id":"mvp-agent","display_name":"MVP","tags":["gate"],"personality_ref":"profiles/mvp.md","tools":[],"spawn_order_ref":"agents/mvp"}]
            }))
            .unwrap(),
        )
        .unwrap();
        Self { root, bridget: bridget.to_path_buf(), socket, config }
    }

    fn start_daemon(&self) -> Child {
        let child = Command::new(&self.bridget)
            .arg("daemon")
            .env("HOME", &self.root)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if self.socket.exists() {
                let spawn = Command::new(&self.bridget)
                    .args(["spawn", "mvp_fixture", "--name", "mvp-agent", "--cwd"])
                    .arg(&self.root)
                    .env("HOME", &self.root)
                    .output()
                    .unwrap();
                assert!(spawn.status.success(), "{:?}", spawn.stderr);
                return child;
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!("daemon MVP non prêt");
    }

    fn wait_for_agent(&self, name: &str) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if BridgetClient::list_agents_at(&self.socket)
                .unwrap_or_default()
                .iter()
                .any(|agent| agent.name == name && agent.state == "connected")
            {
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!("équipier MVP non connecté");
    }

    fn maicie(&self, args: &[&str]) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_maicie")).args(args).output().unwrap()
    }

    fn stop_agent(&self) {
        let _ = Command::new(&self.bridget)
            .args(["stop", "mvp-agent"])
            .env("HOME", &self.root)
            .output();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.root); }
}

fn stop_daemon(daemon: &mut Child) {
    unsafe { libc::kill(daemon.id() as i32, libc::SIGTERM); }
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if daemon.try_wait().unwrap().is_some() { return; }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("daemon MVP ne s'arrête pas dans la borne");
}
