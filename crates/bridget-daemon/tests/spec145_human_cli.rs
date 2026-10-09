use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = PathBuf::from("/tmp").join(format!("h145-{}", uuid::Uuid::new_v4().simple()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&root)
            .unwrap();
        Self(root)
    }
    fn command(&self, namespace: &std::path::Path) -> Command {
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
                "list",
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
fn spec145_cold_cli_does_not_prepare_namespace_or_bootstrap_daemon() {
    let fixture = Fixture::new();
    let namespace = fixture.0.join("cold");
    let output = fixture.command(&namespace).output().unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(
        !namespace.exists(),
        "inspect ne crée aucun état en absence du daemon"
    );
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("daemon_unreachable")
    );
}

fn negotiated_cli(capability: bool, version: u16) -> (std::process::Output, Vec<Value>) {
    let fixture = Fixture::new();
    let socket = fixture.0.join("s");
    let listener = UnixListener::bind(&socket).unwrap();
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
    listener.set_nonblocking(true).unwrap();
    let server = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(3);
        let stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline =>
                {
                    std::thread::sleep(Duration::from_millis(10))
                }
                Err(e) => panic!("connexion Client inspect absente: {e}"),
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
            json!({"type":"ClientWelcome","version":version,"build_id":"fixture","horizon_secs":60,"issued_at_tolerance_secs":5,"capabilities":if capability {vec!["human_thread_view_v1"]}else{vec![]}}),
            json!({"type":"HumanThreadViewResult","result":{"version":1,"subject":null,"result":{"status":"error","code":"binding_unavailable","detail":"Liaison T3 indisponible.","retryable":false}}}),
        ];
        for reply in replies
            .into_iter()
            .take(if capability && version == 1 { 3 } else { 2 })
        {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            seen.push(serde_json::from_str::<Value>(&line).unwrap());
            writeln!(writer, "{reply}").unwrap();
            writer.flush().unwrap();
        }
        seen
    });
    let output = fixture.command(&fixture.0).output().unwrap();
    (output, server.join().unwrap())
}

#[test]
fn spec145_cli_negotiates_only_client_capability_and_keeps_exit2_json() {
    let (output, seen) = negotiated_cli(true, 1);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(seen[0], json!({"type":"RoleHandshake","role":"client"}));
    assert_eq!(seen[1]["capabilities"], json!(["human_thread_view_v1"]));
    assert_eq!(seen[2]["type"], "HumanThreadViewV1");
    assert!(
        !seen
            .iter()
            .any(|v| v["type"] == "RegisterAuxiliary" || v["type"] == "Register")
    );
    let output: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output["result"]["code"], "binding_unavailable");
    assert!(output["subject"].is_null());
}

#[test]
fn spec145_old_daemon_without_capability_has_no_agent_fallback() {
    let (output, seen) = negotiated_cli(false, 1);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(seen.len(), 2);
    let output: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output["result"]["code"], "unsupported_version");
}

#[test]
fn spec145_welcome_version_must_be_accepted_before_any_read() {
    let (output, seen) = negotiated_cli(true, 2);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(seen.len(), 2);
    let output: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output["result"]["code"], "unsupported_version");
}
