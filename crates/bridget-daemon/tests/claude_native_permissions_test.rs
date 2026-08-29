//! Couture réelle : un Claude géré n'exécute un outil (écrire + relire un
//! fichier) que si la définition embarquée porte le bypass de permissions.
//!
//! Le faux binaire mime le symptôme de fable-reviewer : sans
//! `--dangerously-skip-permissions` / `bypassPermissions`, le tour reste
//! muet (aucune réponse, aucun fichier). Avec les flags, l'outil aboutit.

use bridget_core::BridgetMessage;
use bridget_daemon::registry::AgentRegistry;
use bridget_daemon::wrapper::launch_acp_with;
use bridget_transport::protocol::{PresenceMode, decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

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
printf '%s\n' "$BRIDGET_AGENT_NAME" > "$root/claude-name.txt"
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
    let socket = root.join(".cache/bridget/bridget.sock");
    let registry_path = root.join(".config/bridget/agents.json");
    fs::create_dir_all(socket.parent().unwrap()).unwrap();
    fs::create_dir_all(registry_path.parent().unwrap()).unwrap();
    let adapter = write_tool_fixture(&root);
    let registry_json = registry_json(&adapter, with_bypass, mcp_interactive);
    fs::write(&registry_path, &registry_json).unwrap();
    fs::set_permissions(&registry_path, fs::Permissions::from_mode(0o600)).unwrap();

    let listener = UnixListener::bind(&socket).unwrap();
    let (tx, rx) = mpsc::channel();
    let daemon = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
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
                ..
            } => {
                assert_eq!(agent_type, "claude");
                assert_eq!(transport.as_deref(), Some("claude_stream_json"));
                assert!(
                    channel
                        .as_deref()
                        .is_some_and(|value| value != "claude_stream_json")
                );
                assert_eq!(mode, Some(PresenceMode::Cli));
            }
            other => panic!("Register Claude attendu, reçu : {other:?}"),
        }
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::Registered {
                name: "claude-outil-1".to_string(),
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
            "demandeur",
            "claude-outil-1",
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
        while std::time::Instant::now() < deadline {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => match decode(line.trim_end()).unwrap() {
                    WrapperToDaemon::Send(reply) => {
                        body = Some(reply.body);
                        break;
                    }
                    WrapperToDaemon::Unregister => break,
                    _ => {}
                },
                Err(_) => break,
            }
        }
        let _ = writeln!(writer, "{}", encode(&DaemonToWrapper::Disconnect).unwrap());
        let _ = writer.flush();
        let _ = tx.send(Ok(body.unwrap_or_default()));
    });

    let registry = AgentRegistry::from_json(&registry_json, &registry_path).unwrap();
    let wrapper_root = root.clone();
    let wrapper_socket = socket.clone();
    let wrapper = thread::spawn(move || {
        launch_acp_with(
            "claude",
            &[],
            Some("claude-outil-1"),
            &registry,
            &wrapper_socket,
            &wrapper_root,
        )
        .map_err(|error| error.to_string())
    });

    let wrapper_result = wrapper.join().unwrap();
    let daemon_result = rx.recv_timeout(Duration::from_secs(5));
    let _ = daemon.join();
    assert_eq!(wrapper_result, Ok(()), "wrapper Claude géré");
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
    assert_eq!(name.trim(), "claude-outil-1");
    let path = fs::read_to_string(root.join("claude-path.txt")).unwrap();
    let directory = std::env::current_exe()
        .unwrap()
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
