//! Vérifie le point d'entrée réel dans un processus à environnement isolé.
use serde_json::{Value, json};
use std::process::Command;

#[test]
fn partial_or_invalid_t3_proof_never_falls_back_to_pid() {
    for case in [
        "endpoint_only",
        "authorization_only",
        "invalid_pair",
        "empty_pair",
    ] {
        let root = std::env::temp_dir().join(format!("b148-env-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let mut child = Command::new(std::env::current_exe().unwrap());
        child
            .args(["--ignored", "--exact", "isolated_identity_entrypoint"])
            .env_remove("BRIDGET_T3_MCP_ENDPOINT")
            .env_remove("BRIDGET_T3_MCP_AUTHORIZATION")
            .env_remove("BRIDGET_AGENT_NAME")
            .env_remove("BRIDGET_INSTANCE_ID")
            .env_remove("BRIDGET_AGENT_ID_FILE")
            .env_remove("BRIDGET_AGENT_INSTANCE_ID")
            .env_remove("BRIDGET_SOCKET")
            .env("HOME", &root)
            .env("BRIDGET_HOME", root.join("bridget"))
            .env("T3CODE_HOME", root.join("t3"))
            .env("BRIDGET148_ENV_PROBE", "1");
        match case {
            "endpoint_only" => {
                child.env("BRIDGET_T3_MCP_ENDPOINT", "http://127.0.0.1:1/mcp");
            }
            "authorization_only" => {
                child.env("BRIDGET_T3_MCP_AUTHORIZATION", "Bearer invalid");
            }
            "invalid_pair" => {
                child
                    .env("BRIDGET_T3_MCP_ENDPOINT", "http://evil.test/mcp")
                    .env("BRIDGET_T3_MCP_AUTHORIZATION", "Bearer invalid");
            }
            "empty_pair" => {
                child
                    .env("BRIDGET_T3_MCP_ENDPOINT", "")
                    .env("BRIDGET_T3_MCP_AUTHORIZATION", "");
            }
            _ => unreachable!(),
        }
        let output = child.output().unwrap();
        std::fs::remove_dir_all(root).unwrap();
        assert!(
            output.status.success(),
            "{case}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
#[ignore = "helper appelé uniquement par un processus isolé"]
fn isolated_identity_entrypoint() {
    assert_eq!(std::env::var("BRIDGET148_ENV_PROBE").as_deref(), Ok("1"));
    // Une identité PID réellement valide existe : le refus ne doit pas l'emprunter.
    let namespace = bridget_daemon::environment::Namespace::from_environment().unwrap();
    namespace.prepare().unwrap();
    let markers = namespace.root.join("agent-pids");
    bridget_daemon::environment::ensure_private_directory(&markers).unwrap();
    let name_file = namespace.root.join("native-name");
    let native_name = "a43b86e5-7cff-4b8f-83a3-62c9ec9852c4";
    bridget_transport::fsutil::write_private_file_atomic(&name_file, native_name.as_bytes())
        .unwrap();
    let pid = std::process::id();
    bridget_daemon::mcp_identity::write_marker(
        &markers,
        pid,
        bridget_daemon::managed_process::process_birth(pid).unwrap(),
        "native-instance",
        &name_file,
    )
    .unwrap();
    assert_eq!(
        bridget_daemon::mcp_identity::resolve_current_identity()
            .unwrap()
            .name,
        native_name
    );
    let requests = [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"env-probe","version":"1"}}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"bridget_who","arguments":{}}}),
    ].iter().map(|value| format!("{value}\n")).collect::<String>();
    let mut output = Vec::new();
    bridget_daemon::mcp::serve(std::io::Cursor::new(requests), &mut output).unwrap();
    let response: Value = String::from_utf8(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .find(|value| value["id"] == 2)
        .unwrap();
    assert_eq!(response["result"]["isError"], true);
    assert!(
        response.to_string().contains("t3_session_unavailable"),
        "{response}"
    );
}
