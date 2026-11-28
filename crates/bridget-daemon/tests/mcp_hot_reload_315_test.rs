//! Tests d'intégration bout-en-bout du rechargement à chaud `bridget mcp` (Spec 315).

use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

fn private_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "bridget-mcp-hr-{label}-{}",
        &uuid::Uuid::new_v4().simple().to_string()[..10]
    ));
    fs::create_dir_all(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    dir
}

fn copy_bridget_bin(dest: &Path) {
    let src = env!("CARGO_BIN_EXE_bridget");
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).unwrap();
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700)).unwrap();
    }
    fs::copy(src, dest).unwrap();
    fs::set_permissions(dest, fs::Permissions::from_mode(0o755)).unwrap();
}

fn atomic_swap_symlink(link: &Path, target: &Path) {
    let next = link.with_extension("next");
    let _ = fs::remove_file(&next);
    symlink(target, &next).unwrap();
    fs::rename(&next, link).unwrap();
}

struct LiveMcp {
    child: Child,
    stdin: ChildStdin,
    rx: mpsc::Receiver<String>,
}

impl LiveMcp {
    fn spawn(watch_link: &Path, home: &Path, extra_env: &[(&str, &str)]) -> Self {
        let mut cmd = Command::new(watch_link);
        cmd.arg("mcp")
            .env("HOME", home)
            .env("BRIDGET_HOME", home.join("state"))
            .env("BRIDGET_SOCKET", home.join("state/bridget.sock"))
            .env("BRIDGET_MCP_WATCH_LINK", watch_link)
            .env("BRIDGET_MCP_HOT_RELOAD_POLL_MS", "60")
            .env("BRIDGET_MCP_HOT_RELOAD_MIN_INTERVAL_MS", "100")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (k, v) in extra_env {
            cmd.env(k, v);
        }
        let mut child = cmd.spawn().expect("spawn bridget mcp");
        let stdin = child.stdin.take().unwrap();
        let stdout: ChildStdout = child.stdout.take().unwrap();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        if tx.send(line.clone()).is_err() {
                            break;
                        }
                    }
                }
            }
        });
        Self { child, stdin, rx }
    }

    fn send(&mut self, value: &Value) {
        writeln!(self.stdin, "{value}").unwrap();
        self.stdin.flush().unwrap();
    }

    fn recv(&self, timeout: Duration) -> Value {
        let line = self
            .rx
            .recv_timeout(timeout)
            .expect("message JSON-RPC attendu avant expiration");
        serde_json::from_str(line.trim()).expect("JSON-RPC valide")
    }
}

impl Drop for LiveMcp {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn spec315_bascule_a_chaud_conserve_pid_emet_list_changed_et_repond() {
    let root = private_dir("e2e");
    let home = root.join("home");
    fs::create_dir_all(home.join("state")).unwrap();
    fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
    fs::set_permissions(home.join("state"), fs::Permissions::from_mode(0o700)).unwrap();

    let v1 = root.join("v1/bridget");
    let v2 = root.join("v2/bridget");
    copy_bridget_bin(&v1);
    copy_bridget_bin(&v2);

    let bin_dir = root.join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    fs::set_permissions(&bin_dir, fs::Permissions::from_mode(0o700)).unwrap();
    let active = bin_dir.join("bridget");
    symlink(&v1, &active).unwrap();

    let mut mcp = LiveMcp::spawn(&active, &home, &[]);
    let initial_pid = mcp.child.id();

    mcp.send(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    }));
    let init = mcp.recv(Duration::from_secs(5));
    assert_eq!(init["id"], 1);
    assert_eq!(
        init["result"]["capabilities"]["tools"]["listChanged"],
        json!(true)
    );

    mcp.send(&json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    }));
    mcp.send(&json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "ping"
    }));
    let ping1 = mcp.recv(Duration::from_secs(5));
    assert_eq!(ping1["id"], 2);
    assert_eq!(ping1["result"], json!({}));

    // Bascule atomique vers v2 pendant que le processus MCP est au repos.
    atomic_swap_symlink(&active, &v2);

    // Le processus se remplace en place et émet spontanément `notifications/tools/list_changed`.
    let notif = mcp.recv(Duration::from_secs(5));
    assert_eq!(notif["jsonrpc"], "2.0");
    assert_eq!(notif["method"], "notifications/tools/list_changed");
    assert!(notif.get("id").is_none());

    // Même PID vivant, et session toujours initialisée sans nouveau handshake.
    assert_eq!(mcp.child.id(), initial_pid);
    assert!(mcp.child.try_wait().unwrap().is_none());

    // Envoyer deux requêtes pipelinées immédiatement après rechargement.
    write!(
        mcp.stdin,
        "{}\n{}\n",
        json!({"jsonrpc": "2.0", "id": 3, "method": "tools/list"}),
        json!({"jsonrpc": "2.0", "id": 4, "method": "ping"})
    )
    .unwrap();
    mcp.stdin.flush().unwrap();

    let tools_resp = mcp.recv(Duration::from_secs(5));
    assert_eq!(tools_resp["id"], 3);
    assert!(
        tools_resp["result"]["tools"]
            .as_array()
            .is_some_and(|list| !list.is_empty())
    );

    let ping2 = mcp.recv(Duration::from_secs(5));
    assert_eq!(ping2["id"], 4);
    assert_eq!(ping2["result"], json!({}));

    // Une cible non sûre (world-writable) est refusée sans couper la session MCP.
    let v3_bad = root.join("v3/bridget");
    copy_bridget_bin(&v3_bad);
    fs::set_permissions(&v3_bad, fs::Permissions::from_mode(0o777)).unwrap();
    atomic_swap_symlink(&active, &v3_bad);

    std::thread::sleep(Duration::from_millis(200));
    mcp.send(&json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "ping"
    }));
    let ping3 = mcp.recv(Duration::from_secs(5));
    assert_eq!(ping3["id"], 5);
    assert_eq!(ping3["result"], json!({}));

    drop(mcp);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn spec315_bascule_avant_initialize_nemets_pas_list_changed_et_opt_out_desactive() {
    let root = private_dir("preinit");
    let home = root.join("home");
    fs::create_dir_all(home.join("state")).unwrap();
    fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
    fs::set_permissions(home.join("state"), fs::Permissions::from_mode(0o700)).unwrap();

    let v1 = root.join("v1/bridget");
    let v2 = root.join("v2/bridget");
    copy_bridget_bin(&v1);
    copy_bridget_bin(&v2);

    let bin_dir = root.join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    fs::set_permissions(&bin_dir, fs::Permissions::from_mode(0o700)).unwrap();
    let active = bin_dir.join("bridget");
    symlink(&v1, &active).unwrap();

    // 1. Bascule avant initialize : aucune notification prématurée, puis handshake normal.
    let mut mcp = LiveMcp::spawn(&active, &home, &[]);
    atomic_swap_symlink(&active, &v2);
    std::thread::sleep(Duration::from_millis(250));
    mcp.send(&json!({
        "jsonrpc": "2.0",
        "id": 10,
        "method": "initialize",
        "params": {}
    }));
    let first = mcp.recv(Duration::from_secs(5));
    assert_eq!(
        first["id"], 10,
        "aucune notification list_changed ne doit précéder initialize"
    );
    drop(mcp);

    // 2. Opt-out BRIDGET_MCP_HOT_RELOAD=0 : aucune bascule ni notification.
    atomic_swap_symlink(&active, &v1);
    let mut disabled = LiveMcp::spawn(&active, &home, &[("BRIDGET_MCP_HOT_RELOAD", "0")]);
    disabled.send(&json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    }));
    let _ = disabled.recv(Duration::from_secs(5));
    disabled.send(&json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    }));
    atomic_swap_symlink(&active, &v2);
    std::thread::sleep(Duration::from_millis(250));
    disabled.send(&json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "ping"
    }));
    let resp = disabled.recv(Duration::from_secs(5));
    assert_eq!(
        resp["id"], 2,
        "aucune notification list_changed ne doit arriver avec BRIDGET_MCP_HOT_RELOAD=0"
    );

    drop(disabled);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn spec315_echec_execve_et_boucle_rapide_preservent_la_session() {
    let root = private_dir("fallback");
    let home = root.join("home");
    fs::create_dir_all(home.join("state")).unwrap();
    fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
    fs::set_permissions(home.join("state"), fs::Permissions::from_mode(0o700)).unwrap();

    let v1 = root.join("v1/bridget");
    let v2 = root.join("v2/bridget");
    let broken = root.join("broken/bridget");
    copy_bridget_bin(&v1);
    copy_bridget_bin(&v2);
    fs::create_dir_all(broken.parent().unwrap()).unwrap();
    fs::set_permissions(broken.parent().unwrap(), fs::Permissions::from_mode(0o700)).unwrap();
    // Binaire tronqué portant le magic natif : passe le filtre magic puis échoue
    // dans `libc::execve` (ENOEXEC/EBADARCH) sans jamais retomber sur `/bin/sh`.
    #[cfg(target_os = "macos")]
    let truncated_magic: &[u8] = &[0xCF, 0xFA, 0xED, 0xFE, 0, 0, 0, 0];
    #[cfg(not(target_os = "macos"))]
    let truncated_magic: &[u8] = &[0x7F, b'E', b'L', b'F', 0, 0, 0, 0];
    fs::write(&broken, truncated_magic).unwrap();
    fs::set_permissions(&broken, fs::Permissions::from_mode(0o755)).unwrap();

    let bin_dir = root.join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    fs::set_permissions(&bin_dir, fs::Permissions::from_mode(0o700)).unwrap();
    let active = bin_dir.join("bridget");
    symlink(&v1, &active).unwrap();

    let mut mcp = LiveMcp::spawn(&active, &home, &[]);
    mcp.send(&json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}));
    let _ = mcp.recv(Duration::from_secs(5));
    mcp.send(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));

    // 1. Bascule vers un binaire cassé (ENOEXEC) : la session reste vivante sur v1.
    atomic_swap_symlink(&active, &broken);
    std::thread::sleep(Duration::from_millis(250));
    mcp.send(&json!({"jsonrpc": "2.0", "id": 2, "method": "ping"}));
    let resp = mcp.recv(Duration::from_secs(5));
    assert_eq!(resp["id"], 2);
    assert_eq!(resp["result"], json!({}));

    // 2. Boucle rapide v2 -> v1 -> v2 -> v1 : au plus 3 bascules dans la fenêtre de 60s.
    let mut list_changed_count = 0usize;
    for target in [&v2, &v1, &v2, &v1] {
        atomic_swap_symlink(&active, target);
        if let Ok(line) = mcp.rx.recv_timeout(Duration::from_millis(450)) {
            let msg: Value = serde_json::from_str(line.trim()).unwrap();
            if msg["method"] == "notifications/tools/list_changed" {
                list_changed_count += 1;
            }
        }
    }
    assert_eq!(
        list_changed_count, 3,
        "le plafond anti-boucle doit borner à exactement 3 bascules sur 60s"
    );

    // La session reste parfaitement utilisable malgré le plafond atteint.
    mcp.send(&json!({"jsonrpc": "2.0", "id": 3, "method": "ping"}));
    let final_ping = mcp.recv(Duration::from_secs(5));
    assert_eq!(final_ping["id"], 3);
    assert_eq!(final_ping["result"], json!({}));

    drop(mcp);
    let _ = fs::remove_dir_all(&root);
}
