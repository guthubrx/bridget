//! Couture réelle : un Claude géré n'exécute un outil (écrire + relire un
//! fichier) que si la définition embarquée porte le bypass de permissions.
//!
//! Le faux binaire mime le symptôme de agent-relecteur : sans
//! `--dangerously-skip-permissions` / `bypassPermissions`, le tour reste
//! muet (aucune réponse, aucun fichier). Avec les flags, l'outil aboutit.

use bridget_core::BridgetMessage;
use bridget_transport::protocol::{PresenceMode, decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::collections::BTreeSet;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const AGENT_ID: &str = "89000000-0000-4000-8000-000000000301";

struct WrapperChild(Child);

impl Drop for WrapperChild {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            unsafe {
                libc::kill(self.0.id() as i32, libc::SIGTERM);
            }
            let deadline = Instant::now() + Duration::from_secs(3);
            while self.0.try_wait().ok().flatten().is_none() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

fn test_root(label: &str) -> PathBuf {
    PathBuf::from(format!(
        "/tmp/bgp-{}-{}-{}",
        label,
        std::process::id(),
        uuid::Uuid::new_v4()
    ))
}

fn write_tool_fixture(root: &Path) -> PathBuf {
    let adapter = root.join("claude-tool-fixture.sh");
    fs::write(
        &adapter,
        r#"#!/bin/sh
# Mimique un Claude stream-json : sans bypass, le tour ne produit rien
# (permissions bloquées). Avec bypass, écrit un fichier puis le relit.
root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
printf '%s\n' "$@" > "$root/claude-argv.txt"
printf '%s\n' "$BRIDGET_AGENT_ID" > "$root/claude-name.txt"
printf '%s\n' "$PATH" > "$root/claude-path.txt"
has_skip=0
has_mode=0
for arg in "$@"; do
  case "$arg" in
    --dangerously-skip-permissions) has_skip=1 ;;
    bypassPermissions) has_mode=1 ;;
  esac
done
if [ "$has_skip" -ne 1 ] || [ "$has_mode" -ne 1 ]; then
  IFS= read -r _
  exit 0
fi
mkdir -p "$root/worktree"
marker="$root/worktree/outil.txt"
IFS= read -r _
printf "%s\n" "{\"type\":\"system\",\"subtype\":\"init\",\"model\":\"claude-opus-5\"}"
printf "mission-outil-ok\n" > "$marker"
content=$(cat "$marker")
printf "%s\n" "{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",\"result\":\"$content\"}"
"#,
    )
    .unwrap();
    fs::set_permissions(&adapter, fs::Permissions::from_mode(0o700)).unwrap();
    adapter
}

fn registry_json(adapter: &Path, with_bypass: bool, mcp_interactive: &str) -> String {
    let mut args = vec!["--model".to_string(), "claude-opus-5".to_string()];
    if with_bypass {
        args.extend([
            "--dangerously-skip-permissions".to_string(),
            "--permission-mode".to_string(),
            "bypassPermissions".to_string(),
        ]);
    }
    let permissions = if with_bypass { "allow" } else { "deny" };
    serde_json::json!({
        "agents": {
            "claude": {
                "command": adapter,
                "args": args,
                "permissions": permissions,
                "forbidden_env": [],
                "protocol": "claude_stream_json",
                "queue_capacity": 2,
                "notify_timeout_secs": 2,
                "mcp": { "interactive": mcp_interactive },
                "capabilities": {
                    "execution_paths": ["claude_stream_json"],
                    "models": { "claude-opus-5": { "efforts": [] } }
                }
            }
        }
    })
    .to_string()
}

fn run_mission(with_bypass: bool) -> (PathBuf, Result<(String, bool), String>) {
    run_mission_with(with_bypass, "none")
}

fn run_mission_with(
    with_bypass: bool,
    mcp_interactive: &str,
) -> (PathBuf, Result<(String, bool), String>) {
    let root = test_root(if with_bypass { "bypass" } else { "sans" });
    let namespace = bridget_daemon::environment::Namespace::resolve(
        Some(root.join("state")),
        None,
        Some(root.clone()),
    )
    .unwrap();
    namespace.prepare().unwrap();
    let socket = root.join("state/bridget.sock");
    let registry_path = root.join("state/agents.json");
    let adapter = write_tool_fixture(&root);
    let registry_json = registry_json(&adapter, with_bypass, mcp_interactive);
    fs::write(&registry_path, &registry_json).unwrap();
    fs::set_permissions(&registry_path, fs::Permissions::from_mode(0o600)).unwrap();

    let listener = UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    let (tx, rx) = mpsc::channel();
    let daemon = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        let stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(
                        Instant::now() < deadline,
                        "wrapper non connecté à sa socket privée"
                    );
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("accept : {error}"),
            }
        };
        // Darwin peut transmettre O_NONBLOCK au socket accepté. Le listener
        // est pollé, mais les trames utilisent une lecture bloquante bornée.
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(12)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = BufWriter::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        match decode(line.trim_end()).unwrap() {
            WrapperToDaemon::Register {
                agent_type,
                transport,
                channel,
                mode,
                agent_id,
                ..
            } => {
                assert_eq!(agent_id, AGENT_ID);
                assert_eq!(agent_type, "claude");
                assert_eq!(transport.as_deref(), Some("claude_stream_json"));
                assert_eq!(channel.as_deref(), Some("unix"));
                assert_eq!(mode, Some(PresenceMode::Cli));
            }
            other => panic!("Register Claude attendu, reçu : {other:?}"),
        }
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::Registered {
                credential: None,
                agent_id: AGENT_ID.to_string(),
            })
            .unwrap()
        )
        .unwrap();
        writer.flush().unwrap();

        loop {
            line.clear();
            if reader.read_line(&mut line).is_err() || line.is_empty() {
                let _ = tx.send(Err("EOF avant JournalReady".to_string()));
                return;
            }
            if matches!(
                decode(line.trim_end()).unwrap(),
                WrapperToDaemon::JournalReady
            ) {
                break;
            }
        }

        let mut mission = BridgetMessage::new(
            "89000000-0000-4000-8000-000000000302",
            AGENT_ID,
            "écris mission-outil-ok dans worktree/outil.txt puis relis-le",
        );
        mission.id = "mission-outil".to_string();
        mission.reply = true;
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::Deliver(mission)).unwrap()
        )
        .unwrap();
        writer.flush().unwrap();
        // Le wrapper réel et son sous-processus sont planifiés avec les autres
        // intégrations du workspace : 3 s rendait ce témoin dépendant de la charge.
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        let mut body = None;
        let mut peer_finished = false;
        while std::time::Instant::now() < deadline {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    peer_finished = true;
                    break;
                }
                Ok(_) => match decode(line.trim_end()).unwrap() {
                    WrapperToDaemon::Send(reply) => {
                        body = Some(reply.body);
                        break;
                    }
                    WrapperToDaemon::Unregister => {
                        peer_finished = true;
                        break;
                    }
                    _ => {}
                },
                Err(_) => break,
            }
        }
        if !peer_finished {
            let _ = writeln!(writer, "{}", encode(&DaemonToWrapper::Disconnect).unwrap());
            let _ = writer.flush();
            // Comme le daemon, drainer jusqu'à Unregister/EOF : fermer avec des
            // frames non lues provoque un RST qui peut effacer Disconnect (Darwin).
            reader
                .get_ref()
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_)
                        if matches!(
                            decode::<WrapperToDaemon>(line.trim_end()),
                            Ok(WrapperToDaemon::Unregister)
                        ) =>
                    {
                        break;
                    }
                    Ok(_) => {}
                    Err(error) => panic!("arrêt du pair non attesté : {error}"),
                }
            }
        }
        let _ = tx.send(Ok(body.unwrap_or_default()));
    });

    // Processus réel : aucun HOME global manipulé par les tests parallèles.
    let mut wrapper = WrapperChild(
        Command::new(env!("CARGO_BIN_EXE_bridget"))
            .args(["--", "claude", "--equipier", "--agent-id", AGENT_ID])
            .env_clear()
            .env("HOME", &root)
            .env("BRIDGET_HOME", root.join("state"))
            .env("BRIDGET_SOCKET", &socket)
            .env("BRIDGET_CHANNEL", "unix")
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap(),
    );
    let daemon_result = rx.recv_timeout(Duration::from_secs(20));
    daemon.join().expect("pair terminé sans attente infinie");
    let deadline = Instant::now() + Duration::from_secs(5);
    let status = loop {
        if let Some(status) = wrapper.0.try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "wrapper non arrêté après Disconnect"
        );
        thread::sleep(Duration::from_millis(10));
    };
    assert!(status.success(), "wrapper Claude géré : {status}");
    let outcome = match daemon_result {
        Ok(Ok(body)) => Ok((body, root.join("worktree/outil.txt").is_file())),
        Ok(Err(error)) => Err(error),
        Err(error) => Err(format!("timeout daemon: {error}")),
    };
    (root, outcome)
}

#[test]
fn claude_gere_avec_bypass_ecrit_et_relit_un_fichier() {
    let (root, outcome) = run_mission(true);
    let (body, marker) = outcome.expect("mission avec bypass");
    assert!(
        marker,
        "fichier outil absent sous {}",
        root.join("worktree").display()
    );
    assert_eq!(body.trim(), "mission-outil-ok");
    let argv = fs::read_to_string(root.join("claude-argv.txt")).unwrap();
    assert!(
        argv.contains("--dangerously-skip-permissions"),
        "argv sans skip: {argv}"
    );
    assert!(argv.contains("bypassPermissions"), "argv sans mode: {argv}");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn claude_gere_recoit_mcp_identite_et_path() {
    let (root, outcome) = run_mission_with(true, "claude");
    outcome.expect("mission équipée");
    let argv = fs::read_to_string(root.join("claude-argv.txt")).unwrap();
    assert!(argv.contains("--mcp-config"), "argv sans MCP: {argv}");
    assert!(
        argv.contains("--strict-mcp-config"),
        "argv sans MCP strict: {argv}"
    );
    let name = fs::read_to_string(root.join("claude-name.txt")).unwrap();
    assert_eq!(name.trim(), AGENT_ID);
    let path = fs::read_to_string(root.join("claude-path.txt")).unwrap();
    let directory = Path::new(env!("CARGO_BIN_EXE_bridget"))
        .parent()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    // Intention : premier élément = binaire courant (trim, comme C2).
    assert_eq!(
        path.split(':').map(str::trim).next(),
        Some(directory.as_str()),
        "PATH enfant sans préfixe du binaire courant: {path}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn spec094_claude_recoit_la_liste_fermee_des_outils_bridget_autorises() {
    let (root, outcome) = run_mission_with(true, "claude");
    outcome.expect("mission équipée");
    let argv = fs::read_to_string(root.join("claude-argv.txt")).unwrap();
    let arguments = argv.lines().collect::<Vec<_>>();
    let allowed_at = arguments
        .iter()
        .position(|argument| *argument == "--allowedTools")
        .unwrap_or_else(|| panic!("argv sans allowedTools: {argv}"));
    let allowed = arguments
        .get(allowed_at + 1)
        .expect("valeur allowedTools")
        .split(',')
        .collect::<BTreeSet<_>>();
    let expected = [
        "bridget_who",
        "bridget_send",
        "bridget_ledger",
        "bridget_cancel",
        "bridget_read_artifact",
        "bridget_publish_artifact",
        "bridget_rename",
        "bridget_dnd",
        "bridget_domain",
        "bridget_runtime",
        "bridget_status",
        "bridget_control_status",
        "bridget_events",
        "bridget_journal",
        "bridget_thread",
        "bridget_handoff",
    ]
    .map(|name| format!("mcp__bridget__{name}"))
    .into_iter()
    .collect::<BTreeSet<_>>();
    let expected_refs = expected.iter().map(String::as_str).collect::<BTreeSet<_>>();
    assert_eq!(
        allowed, expected_refs,
        "allowlist Claude non fermée : {argv}"
    );
    assert!(
        !argv.contains("mcp__bridget__*"),
        "bypass global interdit: {argv}"
    );
    assert!(!argv.contains("mcp__bridget__guichet_delegate"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn claude_gere_sans_bypass_reste_sans_outil() {
    let (root, outcome) = run_mission(false);
    let (body, marker) = outcome.expect("observation sans bypass");
    assert!(
        !marker,
        "sans bypass le fichier outil ne doit pas apparaître"
    );
    assert!(
        body.is_empty(),
        "sans bypass le tour doit rester muet, reçu {body:?}"
    );
    let argv = fs::read_to_string(root.join("claude-argv.txt")).unwrap();
    assert!(
        !argv.contains("--dangerously-skip-permissions"),
        "témoin inverse contaminé: {argv}"
    );
    let _ = fs::remove_dir_all(root);
}
