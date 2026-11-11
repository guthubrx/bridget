use super::*;

#[test]
fn spec232_lost_admission_reply_then_attestation_refusal_keeps_same_key_and_uncertainty() {
    let mut calls = 0;
    let mut result=recover_admission_with_budget171("same-key",|_:&Value|false,|deadline| {
            calls+=1;
            if calls==1 {
                assert!(deadline.is_none());
                Err(ClientError::Technical{code:"outcome_unknown",message:"reply lost".into()})
            } else {
                assert!(deadline.is_some());
                Ok(json!({"status":"refused","code":"permission_attestation_unavailable",
                    "detail":{"verification":"origin_session_reattest","cause":"origin_worker_unavailable"}}))
            }
        },Duration::from_secs(1)).unwrap();
    admission_refusal_context232(&mut result, Some("same-key"));
    assert_eq!(calls, 2);
    assert_eq!(result["code"], "permission_attestation_unavailable");
    assert_eq!(result["detail"]["cause"], "origin_worker_unavailable");
    assert_eq!(
        result["detail"]["admission"],
        json!({"request_id":"same-key","state":"outcome_unknown"})
    );
    assert!(
        result["detail"]["admission_recovery"]
            .as_str()
            .unwrap()
            .contains("peut déjà exister")
    );
    assert!(result.get("task_id").is_none());
    let mut unrelated = json!({"status":"refused","code":"model_unavailable"});
    admission_refusal_context232(&mut unrelated, Some("same-key"));
    assert!(unrelated.get("detail").is_none());
}

#[test]
#[allow(clippy::assertions_on_constants)] // le drapeau de lancement est une constante : l'assertion garde le test
fn spec221_c05_client_exposes_only_the_explicit_typed_option() {
    use std::os::unix::net::UnixListener;
    assert!(crate::provider_catalogue::MODEL_OPTIONS_LAUNCH_ENABLED);
    let definitions = tools(&json!({}));
    let auto = definitions
        .iter()
        .find(|t| t["name"] == "bridget_auto_delegate")
        .unwrap();
    let schema = auto
        .pointer("/inputSchema/properties/service_tier")
        .unwrap();
    assert_eq!(schema["type"], "string");
    assert!(schema.get("default").is_none());
    let identity = ResolvedIdentity {
        name: "owner".into(),
        instance_id: "instance".into(),
        delegated_origin: None,
    };
    let socket = crate::mcp_identity::mock_socket("c05-option-local-reject");
    let listener = UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    let args = json!({"request_id":"keep-option-key","task":"read","cwd":"/source",
            "service_tier":42});
    let error =
        execute_with_proof(&identity, "bridget_auto_delegate", &args, &socket, None).unwrap_err();
    assert!(matches!(error, ClientError::InvalidParams(_)));
    assert!(error.to_string().contains("service_tier"));
    assert!(matches!(listener.accept(),Err(e) if e.kind()==std::io::ErrorKind::WouldBlock));
    drop(listener);
    std::fs::remove_dir_all(socket.parent().unwrap()).unwrap();
}

#[test]
fn spec254_commit_view_requirement_is_visible_in_description_and_schema() {
    let all = tools(&serde_json::json!({}));
    let auto = all
        .iter()
        .find(|t| t["name"] == "bridget_auto_delegate")
        .unwrap();
    assert!(
        auto["description"]
            .as_str()
            .unwrap()
            .contains("commit_view exige explicitement posture=discovery")
    );
    assert_eq!(
        auto["inputSchema"]["allOf"][1]["then"]["properties"]["posture"]["const"],
        serde_json::json!("discovery")
    );
}

#[test]
#[allow(clippy::assertions_on_constants)] // le drapeau de lancement est une constante : l'assertion garde le test
fn spec221_c19_auto_view_requires_explicit_scope_and_keeps_closed_schema_before_connection() {
    use std::os::unix::net::UnixListener;
    assert!(crate::workspace_git::COMMIT_VIEW_LAUNCH_ENABLED);
    let definitions = tools(&json!({}));
    let auto = definitions
        .iter()
        .find(|t| t["name"] == "bridget_auto_delegate")
        .unwrap();
    let direct = definitions
        .iter()
        .find(|t| t["name"] == "bridget_delegate")
        .unwrap();
    let variants = auto
        .pointer("/inputSchema/properties/workspace/oneOf")
        .unwrap()
        .as_array()
        .unwrap();
    assert!(
        variants
            .iter()
            .any(
                |v| v.pointer("/properties/strategy/const") == Some(&json!("commit_view"))
                    && v["additionalProperties"] == false
            )
    );
    assert!(
        !direct
            .pointer("/inputSchema/properties/workspace/oneOf")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v.pointer("/properties/strategy/const") == Some(&json!("commit_view")))
    );
    assert_eq!(
        auto["inputSchema"]["allOf"][1]["then"]["required"],
        json!(["posture"])
    );
    let identity = ResolvedIdentity {
        name: "owner".into(),
        instance_id: "instance".into(),
        delegated_origin: None,
    };
    let socket = crate::mcp_identity::mock_socket("c19-view-local-reject");
    let listener = UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    let original = json!({"request_id":"same-key","task":"read","cwd":"/source/.worktrees/X",
            "workspace":{"strategy":"commit_view"},"expected_base_commit":"a".repeat(40),"posture":"discovery"});
    let mut missing = original.clone();
    missing
        .as_object_mut()
        .unwrap()
        .remove("expected_base_commit");
    let mut writing = original.clone();
    writing["posture"] = json!("development");
    let mut absent = original.clone();
    absent.as_object_mut().unwrap().remove("posture");
    let mut extra = original.clone();
    extra["workspace"]["branch"] = json!("PRIVATE-VALUE");
    for (tool, args) in [
        ("bridget_auto_delegate", missing),
        ("bridget_auto_delegate", writing),
        ("bridget_auto_delegate", absent),
        ("bridget_auto_delegate", extra),
        (
            "bridget_delegate",
            json!({"request_id":"same-key","task":"read","cwd":"/source",
                "agent_type":"codex","model":"fixture","posture":"discovery","workspace":{"strategy":"commit_view"}}),
        ),
    ] {
        let err = execute_with_proof(&identity, tool, &args, &socket, None).unwrap_err();
        assert!(matches!(err, ClientError::InvalidParams(_)));
        assert!(!err.to_string().contains("PRIVATE-VALUE"));
        assert!(matches!(listener.accept(),Err(e) if e.kind()==std::io::ErrorKind::WouldBlock));
    }
    drop(listener);
    std::fs::remove_dir_all(socket.parent().unwrap()).unwrap();
}

#[test]
#[allow(clippy::assertions_on_constants)] // le drapeau de lancement est une constante : l'assertion garde le test
fn spec221_c19_auto_view_negotiates_creation_before_sending_original_envelope() {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixListener;
    assert!(crate::workspace_git::COMMIT_VIEW_LAUNCH_ENABLED);
    for features in [
        vec!["auto_delegate_v2"],
        vec!["worker_workspace_view_create_v1"],
        vec!["auto_delegate_v2", "worker_workspace_view_create_v1"],
    ] {
        let ready = features.len() == 2;
        let socket = crate::mcp_identity::mock_socket("c19-view-master");
        let listener = UnixListener::bind(&socket).unwrap();
        crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
        let backend = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(&line).unwrap()["type"],
                "RegisterAuxiliary"
            );
            writeln!(
                stream,
                "{}",
                json!({"type":"Registered","agent_id":"owner","credential":null})
            )
            .unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap();
            assert_eq!(
                serde_json::from_str::<Value>(&line).unwrap()["type"],
                "DaemonIdentityRequest"
            );
            writeln!(
                stream,
                "{}",
                json!({"type":"DaemonIdentityReport","host":"fixture",
                    "db_path":"fixture","instance_id":"fixture","features":features})
            )
            .unwrap();
            line.clear();
            let count = reader.read_line(&mut line).unwrap();
            if !ready {
                assert_eq!(
                    count, 0,
                    "pas d'admission vers un maître sans les deux capacités"
                );
                return None;
            }
            assert!(count > 0);
            let request: Value = serde_json::from_str(&line).unwrap();
            writeln!(stream,"{}",json!({"type":"NativeDelegationResult","result":{"status":"accepted","task_id":"fixture-view"}})).unwrap();
            Some(request)
        });
        let identity = ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        let args = json!({"request_id":"exact-key","task":"read","cwd":"/source/.worktrees/X",
                "workspace":{"strategy":"commit_view"},"expected_base_commit":"a".repeat(40),
                "posture":"discovery","child_requests":"deny"});
        let canonical = serde_json::to_vec(&args).unwrap();
        let result = execute_with_proof(&identity, "bridget_auto_delegate", &args, &socket, None);
        let sent = backend.join().unwrap();
        if ready {
            assert_eq!(result.unwrap()["task_id"], "fixture-view");
            let sent = sent.unwrap();
            assert_eq!(sent["request"]["operation"], "auto_delegate_v2");
            for key in [
                "request_id",
                "cwd",
                "workspace",
                "expected_base_commit",
                "posture",
                "child_requests",
            ] {
                assert_eq!(sent["request"][key], args[key]);
            }
        } else {
            assert!(matches!(result,Err(ClientError::Technical {message,..})
                    if message.starts_with("commit_view_master_upgrade_required")));
        }
        assert_eq!(serde_json::to_vec(&args).unwrap(), canonical);
        std::fs::remove_dir_all(socket.parent().unwrap()).unwrap();
    }
}

#[test]
fn spec221_c19_file_tool_parses_only_task_and_exact_relative_path() {
    use std::os::unix::net::UnixListener;
    let identity = ResolvedIdentity {
        name: "owner".into(),
        instance_id: "instance".into(),
        delegated_origin: None,
    };
    let socket = crate::mcp_identity::mock_socket("c19-file-invalid");
    let listener = UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    for args in [
        json!({"path":"../x"}),
        json!({"path":"/x"}),
        json!({"path":"./x"}),
        json!({"path":"x","task_id":null}),
        json!({"path":"x","execution":"PRIVATE"}),
    ] {
        assert!(matches!(
            execute_with_proof(&identity, "bridget_read_commit_file", &args, &socket, None),
            Err(ClientError::InvalidParams(_))
        ));
        assert!(
            matches!(listener.accept(),Err(error) if error.kind()==std::io::ErrorKind::WouldBlock)
        );
    }
    let tools = tools(&json!({}));
    let tool = tools
        .iter()
        .find(|tool| tool["name"] == "bridget_read_commit_file")
        .unwrap();
    assert_eq!(tool["inputSchema"]["additionalProperties"], false);
    assert_eq!(tool["annotations"]["destructiveHint"], false);
    drop(listener);
    std::fs::remove_dir_all(socket.parent().unwrap()).unwrap();
}

#[test]
fn incident171_pending_admission_recovers_same_receipt_with_one_deadline() {
    let pending =
        |value: &Value| value["status"] == "refused" && value["code"] == "admission_pending";
    let mut calls = 0;
    let mut recovery_deadline = None;
    let result = recover_admission_reply171("real-key", pending, |deadline| {
        calls += 1;
        if calls == 1 {
            assert!(deadline.is_none());
            return Err(ClientError::Technical {
                code: "outcome_unknown",
                message: "lost reply".into(),
            });
        }
        let observed = deadline.unwrap();
        if let Some(previous) = recovery_deadline {
            assert_eq!(observed, previous);
        }
        recovery_deadline = Some(observed);
        Ok(if calls < 4 {
            json!({"status":"refused","code":"admission_pending"})
        } else {
            json!({"task_id":"same-task","status":"starting"})
        })
    })
    .unwrap();
    assert_eq!(calls, 4);
    assert_eq!(result["task_id"], "same-task");
    let mut calls = 0;
    let refusal = json!({"status":"refused","code":"permission_not_inherited"});
    assert_eq!(
        recover_admission_reply171("real-key", pending, |_| {
            calls += 1;
            Ok(refusal.clone())
        })
        .unwrap(),
        refusal
    );
    assert_eq!(calls, 1);
    let mut calls = 0;
    let result = recover_admission_with_budget171(
        "real-key",
        pending,
        |_| {
            calls += 1;
            Ok(json!({"status":"refused","code":"admission_pending"}))
        },
        Duration::ZERO,
    );
    assert_eq!(calls, 1);
    assert!(
        matches!(result, Err(ClientError::Technical { code: "outcome_unknown", message })
            if message.contains("real-key") && message.contains("doublon"))
    );
}

#[test]
fn spec222_recovery_transport_gap_returns_observed_final_refusal() {
    let mut calls = 0;
    let mut fixed = None;
    let refusal = json!({"status":"refused","code":"permission_not_inherited",
            "detail":{"admission":{"state":"refused","task_id":null}}});
    let result = recover_admission_with_budget171(
        "same222-key",
        |v: &Value| v["code"] == "admission_pending",
        |deadline| {
            calls += 1;
            if calls == 1 {
                assert!(deadline.is_none());
                return Err(ClientError::Technical {
                    code: "outcome_unknown",
                    message: "lost".into(),
                });
            }
            if let Some(previous) = fixed {
                assert_eq!(deadline, Some(previous));
            }
            fixed = deadline;
            match calls {
                2 => Err(ClientError::Technical {
                    code: "daemon_unreachable",
                    message: "restart".into(),
                }),
                3 => Ok(json!({"status":"refused","code":"admission_pending"})),
                _ => Ok(refusal.clone()),
            }
        },
        Duration::from_secs(2),
    )
    .unwrap();
    assert_eq!(calls, 4);
    assert_eq!(result, refusal);
    let unknown: Result<Value, _> = recover_admission_with_budget171(
        "same222-key",
        |_| false,
        |_| {
            Err(ClientError::Technical {
                code: "outcome_unknown",
                message: "lost".into(),
            })
        },
        Duration::ZERO,
    );
    assert!(
        matches!(unknown, Err(ClientError::Technical {code:"outcome_unknown",message})
            if message.contains("en attente, admise ou refusée")
                && message.contains("same222-key") && !message.contains("toujours en cours"))
    );
}

#[test]
fn delivery171_lost_admission_reply_is_recovered_once_without_masking_refusals() {
    let mut calls = Vec::new();
    let result = recover_admission_reply171(
        "same-key",
        |_| false,
        |deadline| {
            calls.push(deadline.is_some());
            if deadline.is_some() {
                Ok(json!({"task_id":"same-task","status":"starting"}))
            } else {
                Err(ClientError::Technical {
                    code: "outcome_unknown",
                    message: "lecture interrompue".into(),
                })
            }
        },
    )
    .unwrap();
    assert_eq!(calls, vec![false, true]);
    assert_eq!(result["task_id"], "same-task");
    let mut calls = 0;
    let result: Result<(), _> = recover_admission_with_budget171(
        "same-key",
        |_| false,
        |_| {
            calls += 1;
            Err(ClientError::Technical {
                code: "outcome_unknown",
                message: "délai".into(),
            })
        },
        Duration::from_millis(5),
    );
    assert_eq!(calls, 2);
    assert!(
        matches!(result,Err(ClientError::Technical {code:"outcome_unknown",message}) if message.contains("same-key") && message.contains("doublon"))
    );
    let mut calls = 0;
    let result: Result<(), _> = recover_admission_reply171(
        "same-key",
        |_| false,
        |_| {
            calls += 1;
            Err(ClientError::Technical {
                code: "native_delegation_refused",
                message: "permission_not_inherited".into(),
            })
        },
    );
    assert_eq!(calls, 1);
    assert!(matches!(
        result,
        Err(ClientError::Technical {
            code: "native_delegation_refused",
            ..
        })
    ));
}
