//! Gate manuelle T1006 : exécute les vrais harness Codex et Claude à travers
//! `bridget <type>`, avec les options construites par le wrapper de production.
//!
//! Ce banc est ignoré par défaut : il requiert une session authentifiée de
//! chaque harness et peut consommer du quota. Exécution explicite :
//! `BRIDGET_MCP_REAL_SMOKE=1 cargo test -p bridget-daemon --features
//! test-support --test mcp_injection_smoke_test -- --ignored --test-threads=1`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(feature = "test-support")]
use std::io::{BufRead, BufReader, BufWriter, Write};
#[cfg(feature = "test-support")]
use std::os::unix::fs::PermissionsExt;
#[cfg(feature = "test-support")]
use std::os::unix::net::UnixListener;
#[cfg(feature = "test-support")]
use std::thread;

#[cfg(feature = "test-support")]
use bridget_core::BridgetMessage;
#[cfg(feature = "test-support")]
use bridget_transport::protocol::{decode, encode};
#[cfg(feature = "test-support")]
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};

const PROBE_PROMPT: &str =
    "Appelle exactement une fois l'outil MCP probe puis réponds seulement avec son résultat.";

fn config_files(home: &Path) -> [PathBuf; 4] {
    [
        home.join(".claude/settings.json"),
        home.join(".claude/settings.local.json"),
        home.join(".codex/config.toml"),
        home.join(".gemini/settings.json"),
    ]
}

fn config_snapshot(home: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    config_files(home)
        .into_iter()
        .map(|path| {
            let contents = match fs::read(&path) {
                Ok(contents) => Some(contents),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => panic!("lecture impossible {}: {error}", path.display()),
            };
            (path, contents)
        })
        .collect()
}

#[cfg(feature = "test-support")]
fn write_user_config_sentinels(home: &Path) {
    for (relative, contents) in [
        (".claude/settings.json", b"claude-user-config".as_slice()),
        (".codex/config.toml", b"codex-user-config".as_slice()),
        (".gemini/settings.json", b"gemini-user-config".as_slice()),
    ] {
        let path = home.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }
}

fn fake_server() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/fake-mcp-server.py")
}

fn run_harness(home: &Path, agent: &str, arguments: &[&str], expects_tools_list: bool) {
    let log = std::env::temp_dir().join(format!(
        "bridget-t1006-{agent}-{}-{}.log",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    let before = config_snapshot(home);
    let output = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .arg(agent)
        .args(arguments)
        .env("BRIDGET_TEST_MCP_SERVER_COMMAND", "python3")
        .env(
            "BRIDGET_TEST_MCP_SERVER_ARGS",
            serde_json::to_string(&vec![fake_server().display().to_string()]).unwrap(),
        )
        .env("BRIDGET_MCP_SMOKE_LOG", &log)
        .output()
        .unwrap_or_else(|error| panic!("lancement {agent} impossible: {error}"));
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "{agent} a échoué : {combined}");
    assert!(
        combined.contains("PROBE_OK"),
        "{agent} n'a pas retourné PROBE_OK : {combined}"
    );

    let log_contents = fs::read_to_string(&log)
        .unwrap_or_else(|error| panic!("journal MCP {agent} absent: {error}"));
    if expects_tools_list {
        assert!(
            log_contents.contains("method=tools/list"),
            "{agent} n'a pas listé les outils : {log_contents}"
        );
    }
    assert!(
        log_contents.contains("method=tools/call") && log_contents.contains("probe"),
        "{agent} n'a pas appelé probe : {log_contents}"
    );
    assert_eq!(
        config_snapshot(home),
        before,
        "{agent} a modifié une configuration utilisateur persistante"
    );
    fs::remove_file(log).unwrap();
}

#[cfg(feature = "test-support")]
fn write_acp_probe_adapter(root: &Path) -> PathBuf {
    let adapter = root.join("acp-probe-adapter.py");
    fs::write(
        &adapter,
        r#"#!/usr/bin/env python3
import json
import subprocess
import sys

def send(message):
    print(json.dumps(message), flush=True)

for line in sys.stdin:
    request = json.loads(line)
    method = request["method"]
    if method == "initialize":
        send({"jsonrpc":"2.0", "id":request["id"], "result":{"protocolVersion":1}})
    elif method == "session/new":
        server = request["params"]["mcpServers"][0]
        process = subprocess.Popen(
            [server["command"], *server["args"]],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True,
        )
        def mcp(message, expects_response=True):
            process.stdin.write(json.dumps(message) + "\n")
            process.stdin.flush()
            if expects_response:
                return json.loads(process.stdout.readline())
        mcp({"jsonrpc":"2.0", "id":1, "method":"initialize", "params":{}})
        mcp({"jsonrpc":"2.0", "method":"notifications/initialized"}, False)
        listed = mcp({"jsonrpc":"2.0", "id":2, "method":"tools/list", "params":{}})
        assert listed["result"]["tools"][0]["name"] == "probe"
        probed = mcp({"jsonrpc":"2.0", "id":3, "method":"tools/call", "params":{"name":"probe", "arguments":{}}})
        assert probed["result"]["content"][0]["text"] == "PROBE_OK"
        process.stdin.close()
        process.wait(timeout=3)
        send({"jsonrpc":"2.0", "id":request["id"], "result":{"sessionId":"acp-probe"}})
    elif method == "session/prompt":
        send({"jsonrpc":"2.0", "method":"session/update", "params":{"sessionId":"acp-probe", "update":{"sessionUpdate":"agent_message_chunk", "content":{"type":"text", "text":"PROBE_OK"}}}})
        send({"jsonrpc":"2.0", "id":request["id"], "result":{"stopReason":"end_turn"}})
        break
"#,
    )
    .unwrap();
    fs::set_permissions(&adapter, fs::Permissions::from_mode(0o700)).unwrap();
    adapter
}

#[cfg(feature = "test-support")]
#[test]
fn voie_acp_lance_la_session_wrapper_avec_probe_ephemere() {
    // Une socket Unix macOS est bornée à 104 octets : garder la racine courte
    // pour conserver une socket privée portable.
    let root = PathBuf::from("/tmp").join(format!(
        "b10-{}-{}",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    let home = root.join("home");
    write_user_config_sentinels(&home);
    let before = config_snapshot(&home);
    let namespace = bridget_daemon::environment::Namespace::resolve(
        Some(root.join("state")),
        None,
        Some(home.clone()),
    )
    .unwrap();
    namespace.prepare().unwrap();
    let socket = root.join("state/bridget.sock");
    let adapter = write_acp_probe_adapter(&root);
    let registry_json = serde_json::json!({
        "agents": { "probe-acp": {
            "command": "python3", "args": [adapter], "protocol": "acp",
            "permissions": "allow", "queue_capacity": 1, "notify_timeout_secs": 1,
            "mcp": { "acp_session": true }
        }}
    })
    .to_string();
    let registry_path = root.join("state/agents.json");
    fs::write(&registry_path, registry_json).unwrap();
    fs::set_permissions(&registry_path, fs::Permissions::from_mode(0o600)).unwrap();
    let log = root.join("fake-mcp.log");
    let listener = UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    let server = thread::spawn(move || {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(std::time::Instant::now() < deadline, "wrapper non connecté");
                    thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(error) => panic!("accept : {error}"),
            }
        };
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(10)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = BufWriter::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let WrapperToDaemon::Register { agent_id, .. } =
            decode::<WrapperToDaemon>(line.trim()).unwrap()
        else {
            panic!("enregistrement du wrapper attendu");
        };
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::Registered {
                credential: None,
                agent_id: agent_id.clone()
            })
            .unwrap()
        )
        .unwrap();
        writer.flush().unwrap();
        let message = BridgetMessage::new("human", &agent_id, PROBE_PROMPT);
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::Deliver(message)).unwrap()
        )
        .unwrap();
        writer.flush().unwrap();
        loop {
            line.clear();
            if reader.read_line(&mut line).unwrap() == 0 {
                break;
            }
            if matches!(
                decode::<WrapperToDaemon>(line.trim()).unwrap(),
                WrapperToDaemon::Unregister
            ) {
                break;
            }
        }
    });
    let mut wrapper = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .args([
            "--",
            "python3",
            "--equipier",
            "--agent-id",
            "89000000-0000-4000-8000-000000000a01",
        ])
        .env_clear()
        .env("HOME", &home)
        .env("BRIDGET_HOME", root.join("state"))
        .env("BRIDGET_SOCKET", &socket)
        .env("PATH", "/usr/bin:/bin")
        .env("BRIDGET_TEST_MCP_SERVER_COMMAND", "python3")
        .env(
            "BRIDGET_TEST_MCP_SERVER_ARGS",
            serde_json::to_string(&vec![fake_server().display().to_string()]).unwrap(),
        )
        .env("BRIDGET_MCP_SMOKE_LOG", &log)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    let status = loop {
        if let Some(status) = wrapper.try_wait().unwrap() {
            break status;
        }
        if std::time::Instant::now() >= deadline {
            unsafe {
                libc::kill(wrapper.id() as i32, libc::SIGTERM);
            }
            let stop = std::time::Instant::now() + std::time::Duration::from_secs(3);
            while wrapper.try_wait().ok().flatten().is_none() && std::time::Instant::now() < stop {
                thread::sleep(std::time::Duration::from_millis(10));
            }
            panic!("wrapper MCP non arrêté dans le budget");
        }
        thread::sleep(std::time::Duration::from_millis(10));
    };
    assert!(status.success(), "wrapper MCP : {status}");
    server.join().unwrap();
    let transcript = fs::read_to_string(&log).unwrap();
    assert!(transcript.contains("method=tools/list"));
    assert!(transcript.contains("method=tools/call") && transcript.contains("probe"));
    assert_eq!(config_snapshot(&home), before);
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[ignore = "requiert les profils Codex et Claude authentifiés ; gate manuelle T1006"]
fn wrapper_de_production_injecte_le_serveur_mcp_ephemere_dans_les_harness_disponibles() {
    assert_eq!(
        std::env::var("BRIDGET_MCP_REAL_SMOKE").as_deref(),
        Ok("1"),
        "fixez BRIDGET_MCP_REAL_SMOKE=1 pour confirmer l'exécution volontaire"
    );
    let home = PathBuf::from(std::env::var_os("HOME").expect("HOME requis"));

    run_harness(
        &home,
        "codex",
        &["exec", "--ephemeral", "--skip-git-repo-check", PROBE_PROMPT],
        false,
    );
    run_harness(
        &home,
        "claude",
        &["-p", PROBE_PROMPT, "--output-format", "text"],
        true,
    );

    // Gemini reste volontairement `unsupported`, conformément au constat
    // documenté T708/T1001 ; aucune écriture de configuration ne lui est
    // attribuée par le wrapper.
}
