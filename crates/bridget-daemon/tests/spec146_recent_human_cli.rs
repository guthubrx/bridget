use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = PathBuf::from("/tmp").join(format!("h146-{}", uuid::Uuid::new_v4().simple()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&root)
            .unwrap();
        Self(root)
    }
    fn command(&self, namespace: &Path, action: &str) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_bridget"));
        cmd.env("BRIDGET_HOME", namespace)
            .env("BRIDGET_SOCKET", namespace.join("s"))
            .args([
                "thread",
                "inspect",
                "--t3-thread",
                "89000000-0000-4000-8000-000000000145",
                "--project-root",
                "/fixture",
                "--action",
                action,
                "--json",
            ]);
        cmd
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn spec146_recent_cold_cli_does_not_bootstrap_or_create_namespace() {
    let f = Fixture::new();
    let namespace = f.0.join("cold");
    let output = f.command(&namespace, "list_recent").output().unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(!namespace.exists());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("daemon_unreachable")
    );
    let output = f
        .command(&namespace, "history_recent")
        .args([
            "--thread",
            "14600000-0000-4000-8000-000000000001",
            "--before-seq",
            "0",
            "--to-seq",
            "0",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(!namespace.exists());
}

#[test]
fn spec146_recent_invalid_cursor_is_rejected_before_namespace_preparation() {
    let f = Fixture::new();
    let namespace = f.0.join("cold");
    let output = f
        .command(&namespace, "list_recent")
        .args(["--after", "01:14600000-0000-4000-8000-000000000001"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(!namespace.exists());
    let output: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output["result"]["code"], "invalid_request");
}

fn negotiate_recent(supported: bool) -> (std::process::Output, Vec<Value>) {
    let f = Fixture::new();
    let listener = UnixListener::bind(f.0.join("s")).unwrap();
    std::fs::set_permissions(f.0.join("s"), std::fs::Permissions::from_mode(0o600)).unwrap();
    listener.set_nonblocking(true).unwrap();
    let server = std::thread::spawn(move || {
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        let stream = loop {
            match listener.accept() {
                Ok((s, _)) => break s,
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        && std::time::Instant::now() < deadline =>
                {
                    std::thread::sleep(Duration::from_millis(10))
                }
                Err(e) => panic!("{e}"),
            }
        };
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut writer = stream.try_clone().unwrap();
        let mut reader = BufReader::new(stream);
        let mut seen = Vec::new();
        let replies = [
            json!({"type":"RoleAccepted","role":"client"}),
            json!({"type":"ClientWelcome","version":1,"build_id":"fixture","horizon_secs":60,"issued_at_tolerance_secs":5,"capabilities":if supported {vec!["human_thread_view_recent_v1"]} else {vec!["human_thread_view_v1"]}}),
            json!({"type":"HumanThreadViewResult","result":{"version":1,"subject":null,"result":{"status":"error","code":"binding_unavailable","detail":"Liaison T3 indisponible.","retryable":false}}}),
        ];
        for reply in replies.into_iter().take(if supported { 3 } else { 2 }) {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            seen.push(serde_json::from_str(&line).unwrap());
            writeln!(writer, "{reply}").unwrap();
            writer.flush().unwrap();
        }
        seen
    });
    let output = f.command(&f.0, "list_recent").output().unwrap();
    eprintln!(
        "SPEC146 CLI réelle : code={:?}, stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    (output, server.join().unwrap())
}

#[test]
fn spec146_recent_cli_negotiates_only_recent_human_capability() {
    let (output, seen) = negotiate_recent(true);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(seen[0], json!({"type":"RoleHandshake","role":"client"}));
    assert_eq!(
        seen[1]["capabilities"],
        json!(["human_thread_view_recent_v1"])
    );
    assert_eq!(seen[2]["request"]["request"]["action"], "list_recent");
    assert!(
        seen.iter()
            .all(|frame| frame["type"] != "Register" && frame["type"] != "RegisterAuxiliary")
    );
}

#[test]
fn spec146_recent_old_daemon_has_no_ascending_or_agent_fallback() {
    let (output, seen) = negotiate_recent(false);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(seen.len(), 2);
    let output: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output["result"]["code"], "unsupported_version");
}
