    use super::*;
    #[test]
    fn incident171_attestation_preserves_closed_cause_without_body_or_authority_change() {
        for (code, expected) in [("permission_attestation_unavailable", "permission_attestation_unavailable"),
            ("parent_not_active", "parent_not_active"), ("SECRET", "attestation_tool_refused")] {
            let body=json!({"_tag":"OrchestratorMcpFailure","code":code,"message":"SECRET token=PRIVATE"});
            for result in [json!({"isError":true,"structuredContent":body}),
                json!({"isError":true,"content":[{"type":"text","text":body.to_string()}]})] {
                let error=attest(exchange(json!({"id":2,"result":result}))).err().unwrap();
                assert_eq!(error.code(),"t3_session_unavailable");
                assert!(error.remediation().starts_with(expected));
                assert!(!error.remediation().contains("SECRET") && !error.remediation().contains("PRIVATE"));
            }
        }
        let valid=attest(exchange(response(proof("thread-171")))).unwrap();
        assert_eq!(valid.thread_id,"thread-171");
        assert!(attest(exchange(response(proof("thread\ninvalid")))).is_err());
    }

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
    fn spec172_binding_refusal_is_closed_and_never_prints_raw_proof() {
        for reason in [
            "primary_absent",
            "credential_mismatch",
            "instance_mismatch",
            "primary_not_live",
            "connection_already_registered",
        ] {
            let error = binding_failure(crate::communication::client::ClientError::Technical {
                code: "auxiliary_identity_unproven",
                message: format!("auxiliary_identity_unproven : {reason}"),
            });
            assert_eq!(error.code(), "t3_session_unavailable");
            assert!(error.remediation().contains(reason));
        }
        let error = binding_failure(crate::communication::client::ClientError::Technical {
            code: "auxiliary_identity_unproven",
            message: "private-canary172".into(),
        });
        assert_eq!(error.code(), "t3_session_unavailable");
        assert!(!error.remediation().contains("private-canary172"));
    }

    #[test]
    fn spec174_binding_transitoire_guide_sans_assouplir_identite() {
        for (code, reason) in [
            ("auxiliary_identity_unproven", "auxiliary_identity_unproven : primary_absent"),
            ("auxiliary_identity_unproven", "auxiliary_identity_unproven : primary_not_live"),
            ("daemon_unreachable", "private-canary174"),
        ] {
            let error = binding_failure(crate::communication::client::ClientError::Technical {
                code, message: reason.into(),
            });
            assert_eq!(error.code(), "t3_session_unavailable");
            let message = error.remediation();
            assert!(message.contains("bridget_task_status") && message.contains("même task_id"));
            assert!(message.contains("ne recréez pas la mission"));
            assert!(!message.contains("private-canary174"));
        }
        for reason in ["credential_mismatch", "instance_mismatch"] {
            let error = binding_failure(crate::communication::client::ClientError::Technical {
                code: "auxiliary_identity_unproven",
                message: format!("auxiliary_identity_unproven : {reason}"),
            });
            assert!(!error.remediation().contains("Attendez"));
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
    fn spec172_http_attestation_supporte_reponse_apres_trois_secondes() {
        use std::io::{BufRead, BufReader, Write};
        use std::net::TcpListener;
        use std::time::{Duration, Instant};
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/mcp", server.local_addr().unwrap());
        let peer = std::thread::spawn(move || {
            let (mut stream, _) = server.accept().unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut length = 0;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" { break; }
                if let Some((key, value)) = line.split_once(':')
                    && key.eq_ignore_ascii_case("content-length") { length = value.trim().parse::<usize>().unwrap(); }
            }
            reader.read_exact(&mut vec![0; length]).unwrap();
            std::thread::sleep(Duration::from_millis(3500));
            let body = r#"{"jsonrpc":"2.0","id":2,"result":{}}"#;
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
        });
        let start = Instant::now();
        let response = post(&url, &format!("Bearer {}", "a".repeat(43)), &mut None,
            &json!({"jsonrpc":"2.0","id":2,"method":"tools/call"}));
        peer.join().unwrap();
        assert_eq!(response.unwrap()["id"], 2);
        assert!(start.elapsed() >= Duration::from_millis(3500));
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
                if let Some((key, value)) = line.split_once(':')
                    && key.eq_ignore_ascii_case("content-length") {
                        length = value.trim().parse().unwrap();
                    }
            }
            assert!(length < 4096);
            reader.read_exact(&mut vec![0; length]).unwrap();
            write!(stream,"HTTP/1.1 307 Redirect\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        });
        assert!(matches!(
            attest_http(&url, &format!("Bearer {}", "a".repeat(43))),
            Err(IdentityError::T3SessionUnavailableAt(_))
        ));
        server.join().unwrap();
        assert!(
            matches!(destination.accept(),Err(error) if error.kind()==std::io::ErrorKind::WouldBlock)
        );
    }

    #[test]
    fn http_cleanup_failure_preserves_verified_proof_and_next_call_rechecks() {
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
                        _ => (503, String::new()),
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
        assert!(matches!(call(), Err(IdentityError::T3SessionUnavailableAt("attestation_http_unauthorized : T3 refuse l’authentification de cette session"))));
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
