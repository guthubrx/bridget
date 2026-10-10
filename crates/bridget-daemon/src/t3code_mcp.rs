//! Adaptation d'identité T3. Aucun lancement ni état de tâche ne dépend de ce module.
use crate::mcp_identity::{IdentityError, ResolvedIdentity};
use crate::t3code_contract::{ServerRuntime, base_dir, read_runtime};
use serde::Deserialize;
use serde_json::{Value, json};
use std::io::Read;

const MAX_RESPONSE: u64 = 64 * 1024;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SessionProof {
    version: u8,
    environment_id: String,
    thread_id: String,
    provider_session_id: String,
    provider_instance_id: String,
}

/// La présence d'une seule variable sélectionne aussi ce chemin : une preuve
/// partielle ou refusée ne doit jamais retomber sur une identité de processus.
pub(crate) fn resolve_current() -> Option<Result<ResolvedIdentity, IdentityError>> {
    let endpoint = std::env::var_os("BRIDGET_T3_MCP_ENDPOINT");
    let authorization = std::env::var_os("BRIDGET_T3_MCP_AUTHORIZATION");
    if endpoint.is_none() && authorization.is_none() {
        return None;
    }
    Some((|| {
        let denied = || IdentityError::T3SessionUnavailable;
        let endpoint = endpoint
            .and_then(|v| v.into_string().ok())
            .ok_or_else(denied)?;
        let authorization = authorization
            .and_then(|v| v.into_string().ok())
            .ok_or_else(denied)?;
        let runtime = read_runtime(&base_dir().map_err(|_| denied())?).map_err(|_| denied())?;
        let url = validate_endpoint(&runtime, &endpoint, &authorization)?;
        let proof = attest_http(&url, &authorization)?;
        let namespace = crate::environment::Namespace::from_environment().map_err(|_| denied())?;
        resolve_binding(&proof.thread_id, &namespace.socket)
    })())
}

fn resolve_binding(
    thread_id: &str,
    socket: &std::path::Path,
) -> Result<ResolvedIdentity, IdentityError> {
    let name = crate::t3code::stable_uuid(thread_id);
    let instance_id = crate::t3code::stable_uuid(&format!("instance:{thread_id}"));
    crate::mcp_identity::auxiliary_registration(&name, &instance_id, socket)
        .map_err(|_| IdentityError::T3SessionUnavailable)?;
    // La preuve locale seule ne suffit pas : le daemon doit reconnaître
    // cette instance maintenant, même pour un outil de diagnostic.
    crate::communication::client::registered_connection(&name, &instance_id, socket)
        .map_err(|_| IdentityError::T3SessionUnavailable)?;
    Ok(ResolvedIdentity {
        name,
        instance_id,
        delegated_origin: None,
    })
}

fn validate_endpoint(
    runtime: &ServerRuntime,
    endpoint: &str,
    authorization: &str,
) -> Result<String, IdentityError> {
    let allowed = [
        format!("http://127.0.0.1:{}/mcp", runtime.port),
        format!("http://localhost:{}/mcp", runtime.port),
        format!("http://[::1]:{}/mcp", runtime.port),
    ];
    let token = authorization.strip_prefix("Bearer ").unwrap_or("");
    if !allowed.iter().any(|value| value == endpoint)
        || !(32..=256).contains(&token.len())
        || !token
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(IdentityError::T3SessionUnavailable);
    }
    Ok(format!("{}/mcp", runtime.base_url()))
}

fn attest_http(url: &str, authorization: &str) -> Result<SessionProof, IdentityError> {
    let mut session_id = None;
    let proof = attest(|body| post(url, authorization, &mut session_id, body));
    // Une session de transport par attestation. Ne jamais conserver le jeton
    // ni accumuler des sessions HTTP lors des appels successifs de la façade.
    if let Some(session_id) = session_id {
        let cleanup = minreq::delete(url)
            .with_header("Authorization", authorization)
            .with_header("MCP-Protocol-Version", "2025-06-18")
            .with_header("Mcp-Session-Id", session_id)
            .with_max_redirects(0)
            .with_timeout(3)
            .send_lazy()
            .map_err(|_| IdentityError::T3SessionUnavailable)
            .and_then(|response| {
                if (200..300).contains(&response.status_code) {
                    Ok(())
                } else {
                    Err(IdentityError::T3SessionUnavailable)
                }
            });
        if proof.is_ok() {
            cleanup?;
        }
    }
    proof
}

fn post(
    url: &str,
    authorization: &str,
    session_id: &mut Option<String>,
    body: &Value,
) -> Result<Value, IdentityError> {
    let denied = || IdentityError::T3SessionUnavailable;
    let mut request = minreq::post(url)
        .with_header("Authorization", authorization)
        .with_header("Accept", "application/json, text/event-stream")
        .with_header("Content-Type", "application/json")
        .with_header("MCP-Protocol-Version", "2025-06-18")
        .with_max_redirects(0)
        .with_timeout(3)
        .with_body(body.to_string());
    if let Some(session_id) = session_id.as_ref() {
        request = request.with_header("Mcp-Session-Id", session_id);
    }
    let response = request.send_lazy().map_err(|_| denied())?;
    if !(200..300).contains(&response.status_code) {
        return Err(denied());
    }
    if body.get("method").and_then(Value::as_str) == Some("initialize") {
        let issued = response.headers.get("mcp-session-id").ok_or_else(denied)?;
        if issued.is_empty()
            || issued.len() > 256
            || !issued
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        {
            return Err(denied());
        }
        *session_id = Some(issued.clone());
    }
    let mut bytes = Vec::new();
    Read::take(response, MAX_RESPONSE + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| denied())?;
    if bytes.len() as u64 > MAX_RESPONSE {
        return Err(denied());
    }
    if body.get("id").is_none() && bytes.is_empty() {
        return Ok(Value::Null);
    }
    decode_response(&bytes)
}

fn decode_response(bytes: &[u8]) -> Result<Value, IdentityError> {
    let denied = || IdentityError::T3SessionUnavailable;
    if let Ok(value) = serde_json::from_slice(bytes) {
        return Ok(value);
    }
    // Streamable HTTP peut rendre l'unique réponse sous forme d'événement SSE.
    let text = std::str::from_utf8(bytes).map_err(|_| denied())?;
    let data: Vec<_> = text
        .lines()
        .filter_map(|line| line.strip_prefix("data: "))
        .collect();
    if data.len() != 1 {
        return Err(denied());
    }
    serde_json::from_str(data[0]).map_err(|_| denied())
}

fn attest(
    mut exchange: impl FnMut(&Value) -> Result<Value, IdentityError>,
) -> Result<SessionProof, IdentityError> {
    let denied = || IdentityError::T3SessionUnavailable;
    let init = exchange(
        &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"bridget-session","version":"1"}
        }}),
    )?;
    if init.get("id") != Some(&json!(1))
        || init
            .pointer("/result/protocolVersion")
            .and_then(Value::as_str)
            != Some("2025-06-18")
        || init.get("error").is_some()
    {
        return Err(denied());
    }
    exchange(&json!({"jsonrpc":"2.0","method":"notifications/initialized"}))?;
    let response = exchange(
        &json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"bridget_session","arguments":{}}}),
    )?;
    if response.get("id") != Some(&json!(2))
        || response.get("error").is_some()
        || response.pointer("/result/isError") == Some(&Value::Bool(true))
    {
        return Err(denied());
    }
    let proof: SessionProof = serde_json::from_value(
        response
            .pointer("/result/structuredContent")
            .cloned()
            .ok_or_else(denied)?,
    )
    .map_err(|_| denied())?;
    if proof.version != 1
        // Les fils ordinaires ont souvent un UUID ; les enfants T3 ont aussi
        // des identifiants opaques `thread:delegated-task:...` attestés.
        || proof.thread_id.is_empty() || proof.thread_id.len() > 2048 || proof.thread_id.chars().any(char::is_control)
        || [&proof.environment_id, &proof.provider_session_id, &proof.provider_instance_id].iter().any(|s| s.is_empty() || s.len() > 256 || s.chars().any(char::is_control))
    {
        return Err(denied());
    }
    Ok(proof)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn endpoint_cannot_redirect_or_choose_another_runtime() {
        let runtime = ServerRuntime {
            port: 43123,
            pid: 1,
        };
        let token = format!("Bearer {}", "a".repeat(43));
        assert!(validate_endpoint(&runtime, "http://localhost:43123/mcp", &token).is_ok());
        for endpoint in [
            "http://127.0.0.1:43124/mcp",
            "http://evil.test:43123/mcp",
            "http://127.0.0.1:43123/mcp?token=a",
            "http://127.0.0.1:43123/mcp/",
            "http://127.0.0.1:43123@evil.test/mcp",
        ] {
            assert!(validate_endpoint(&runtime, endpoint, &token).is_err());
        }
        assert!(
            validate_endpoint(
                &runtime,
                "http://localhost:43123/mcp",
                "Bearer secret\r\nX: a"
            )
            .is_err()
        );
    }
    fn response(proof: Value) -> Value {
        json!({"jsonrpc":"2.0","id":2,"result":{"structuredContent":proof}})
    }
    fn proof(thread: &str) -> Value {
        json!({"version":1,"environmentId":"environment:a","threadId":thread,"providerSessionId":"session:a","providerInstanceId":"codex"})
    }
    fn exchange(result: Value) -> impl FnMut(&Value) -> Result<Value, IdentityError> {
        move |request| {
            Ok(match request.get("id").and_then(Value::as_u64) {
                Some(1) => {
                    json!({"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-06-18"}})
                }
                Some(2) => result.clone(),
                _ => Value::Null,
            })
        }
    }
    #[test]
    fn session_is_revalidated_and_refusals_are_closed() {
        let thread = "11111111-1111-4111-8111-111111111111";
        assert_eq!(
            attest(exchange(response(proof(thread)))).unwrap().thread_id,
            thread
        );
        let child = "thread:delegated-task:command%3Amcp%3Areview";
        assert_eq!(
            attest(exchange(response(proof(child)))).unwrap().thread_id,
            child
        );
        for result in [
            json!({"id":2,"result":{"isError":true,"structuredContent":proof(thread)}}),
            json!({"id":2,"error":{"code":401}}),
            response(proof("")),
            response(proof("thread\nforged")),
            json!({"id":2,"result":{"content":[{"type":"text","text":"identity"}]}}),
        ] {
            assert!(attest(exchange(result)).is_err());
        }
        assert!(attest(|_| Err(IdentityError::T3SessionUnavailable)).is_err());
    }
    #[test]
    fn sse_is_bounded_to_one_json_response() {
        assert_eq!(
            decode_response(b"event: message\ndata: {\"id\":2}\n\n").unwrap(),
            json!({"id":2})
        );
        assert!(decode_response(b"data: {}\ndata: {}\n").is_err());
    }

    #[test]
    fn http_redirect_never_forwards_the_private_credential() {
        use std::io::{BufRead, BufReader, Write};
        use std::net::TcpListener;
        let destination = TcpListener::bind("127.0.0.1:0").unwrap();
        destination.set_nonblocking(true).unwrap();
        let source = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/mcp", source.local_addr().unwrap());
        let location = format!("http://{}/mcp", destination.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (mut stream, _) = source.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(3)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            let mut length = 0usize;
            loop {
                line.clear();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" {
                    break;
                }
                if let Some((key, value)) = line.split_once(':') {
                    if key.eq_ignore_ascii_case("content-length") {
                        length = value.trim().parse().unwrap();
                    }
                }
            }
            assert!(length < 4096);
            reader.read_exact(&mut vec![0; length]).unwrap();
            write!(stream,"HTTP/1.1 307 Redirect\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        });
        assert!(matches!(
            attest_http(&url, &format!("Bearer {}", "a".repeat(43))),
            Err(IdentityError::T3SessionUnavailable)
        ));
        server.join().unwrap();
        assert!(
            matches!(destination.accept(),Err(error) if error.kind()==std::io::ErrorKind::WouldBlock)
        );
    }

    #[test]
    fn http_keeps_sessions_distinct_and_checks_revocation_on_next_call() {
        use std::io::{BufRead, BufReader, Write};
        use std::net::TcpListener;
        use std::time::Duration;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/mcp", listener.local_addr().unwrap());
        let authorization = format!("Bearer {}", "a".repeat(43));
        let server_authorization = authorization.clone();
        let server = std::thread::spawn(move || {
            for index in 0..9 {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut header = String::new();
                reader.read_line(&mut header).unwrap();
                assert_eq!(
                    header,
                    if index % 4 == 3 {
                        "DELETE /mcp HTTP/1.1\r\n"
                    } else {
                        "POST /mcp HTTP/1.1\r\n"
                    }
                );
                let mut content_length = 0usize;
                let mut authenticated = false;
                let mut request_session = None;
                for _ in 0..32 {
                    header.clear();
                    reader.read_line(&mut header).unwrap();
                    if header == "\r\n" {
                        break;
                    }
                    if let Some((name, value)) = header.split_once(':') {
                        if name.eq_ignore_ascii_case("authorization") {
                            authenticated = value.trim() == server_authorization;
                        }
                        if name.eq_ignore_ascii_case("content-length") {
                            content_length = value.trim().parse().unwrap();
                        }
                        if name.eq_ignore_ascii_case("mcp-session-id") {
                            request_session = Some(value.trim().to_string());
                        }
                    }
                }
                assert!(authenticated);
                assert!(content_length < 4096);
                let mut body = vec![0; content_length];
                reader.read_exact(&mut body).unwrap();
                let request: Value = if body.is_empty() {
                    Value::Null
                } else {
                    serde_json::from_slice(&body).unwrap()
                };
                let issued_session = if index < 4 { "session-a" } else { "session-b" };
                if index % 4 == 0 {
                    assert!(request_session.is_none());
                } else {
                    assert_eq!(request_session.as_deref(), Some(issued_session));
                }
                let (status, body) = if index == 8 {
                    (401, String::new())
                } else {
                    match index % 4 {
                        0 => {
                            assert_eq!(request["method"], "initialize");
                            (200, json!({"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2025-06-18"}}).to_string())
                        }
                        1 => {
                            assert_eq!(request["method"], "notifications/initialized");
                            (202, String::new())
                        }
                        2 => {
                            assert_eq!(
                                request["params"],
                                json!({"name":"bridget_session","arguments":{}})
                            );
                            (
                                200,
                                response(proof(if index < 4 { "thread:a" } else { "thread:b" }))
                                    .to_string(),
                            )
                        }
                        _ => (204, String::new()),
                    }
                };
                let session_header = if index % 4 == 0 && index < 8 {
                    format!("Mcp-Session-Id: {issued_session}\r\n")
                } else {
                    String::new()
                };
                write!(stream, "HTTP/1.1 {status} Response\r\n{session_header}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
            }
        });
        let call = || attest_http(&url, &authorization);
        assert_eq!(call().unwrap().thread_id, "thread:a");
        assert_eq!(call().unwrap().thread_id, "thread:b");
        assert!(matches!(call(), Err(IdentityError::T3SessionUnavailable)));
        server.join().unwrap();
    }

    #[test]
    fn local_binding_requires_current_daemon_admission() {
        use bridget_transport::protocol::{decode, encode};
        use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
        use std::io::{BufRead, BufReader, Write};
        use std::os::unix::net::UnixListener;
        use std::time::Duration;
        let thread = "thread:connector-test";
        let agent_id = crate::t3code::stable_uuid(thread);
        let instance_id = crate::t3code::stable_uuid(&format!("instance:{thread}"));
        let socket = crate::mcp_identity::mock_socket("t3-session-binding");
        let listener = UnixListener::bind(&socket).unwrap();
        // A live socket alone does not establish a principal.
        assert!(resolve_binding(thread, &socket).is_err());
        crate::mcp_identity::mock_private_identity(&socket, &agent_id, &instance_id);
        let expected_agent = agent_id.clone();
        let expected_instance = instance_id.clone();
        let server = std::thread::spawn(move || {
            for index in 0..2 {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut line = String::new();
                let count = BufReader::new(stream.try_clone().unwrap())
                    .read_line(&mut line)
                    .unwrap();
                if count == 0 {
                    continue;
                }
                let WrapperToDaemon::RegisterAuxiliary {
                    agent_id,
                    instance_id,
                    ..
                } = decode(line.trim()).unwrap()
                else {
                    panic!("private binding required")
                };
                assert_eq!(agent_id, expected_agent);
                assert_eq!(instance_id, expected_instance);
                let result = if index == 0 {
                    DaemonToWrapper::Registered {
                        agent_id,
                        credential: None,
                    }
                } else {
                    DaemonToWrapper::Nack {
                        id: "".into(),
                        reason: "instance revoked".into(),
                    }
                };
                writeln!(stream, "{}", encode(&result).unwrap()).unwrap();
            }
        });
        assert_eq!(
            resolve_binding(thread, &socket).unwrap(),
            ResolvedIdentity {
                name: agent_id,
                instance_id,
                delegated_origin: None
            }
        );
        assert!(resolve_binding(thread, &socket).is_err());
        server.join().unwrap();
        std::fs::remove_dir_all(socket.parent().unwrap()).unwrap();
    }
}
