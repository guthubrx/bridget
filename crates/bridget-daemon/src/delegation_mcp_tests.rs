use super::*;

#[test]
fn spec246_quota_accord_mcp_explicite_et_sans_null() {
    let catalogue = tools(&json!({}));
    for name in ["bridget_auto_delegate", "bridget_delegate"] {
        let tool = catalogue.iter().find(|t| t["name"] == name).unwrap();
        assert_eq!(
            tool["inputSchema"]["properties"]["quota_fallback_authorized"]["type"],
            "boolean"
        );
        let identity = ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        let args = json!({"request_id":"quota246","task":"lecture","cwd":"/source","agent_type":"a","model":"m","quota_fallback_authorized":null});
        let mut args = args;
        if name == "bridget_auto_delegate" {
            args.as_object_mut().unwrap().remove("agent_type");
        }
        assert!(
            matches!(execute_with_proof(&identity,name,&args,Path::new("/missing/never-connected246.sock"),None),Err(ClientError::InvalidParams(message)) if message.contains("quota_fallback_authorized"))
        );
    }
}

#[test]
fn spec245_specialists_guidage_sans_connexion() {
    let catalogue = tools(&json!({}));
    let tool = catalogue
        .iter()
        .find(|t| t["name"] == "bridget_specialists")
        .unwrap();
    let description = tool["description"].as_str().unwrap();
    assert!(
        description.contains("version:1")
            && description.contains("search")
            && description.contains("specialist_agent_id")
    );
    let identity = ResolvedIdentity {
        name: "owner".into(),
        instance_id: "instance".into(),
        delegated_origin: None,
    };
    let refused = execute_with_proof(
        &identity,
        "bridget_specialists",
        &json!({"action":"catalogue"}),
        Path::new("/missing/never-connected245.sock"),
        None,
    );
    match refused {
        Err(ClientError::InvalidParams(message)) => {
            assert!(
                message
                    .contains(r#"{"version":1,"action":"search","role_id":"mainteneur-bridget"}"#)
            );
            assert!(message.contains("Aucun appel envoyé"));
        }
        other => panic!("guidage avant transport attendu : {other:?}"),
    }
}

#[test]
fn spec200_auto_type_refuse_avant_connexion_et_schema_sans_champ_interdit() {
    let catalogue = tools(&json!({}));
    let auto = catalogue
        .iter()
        .find(|t| t["name"] == "bridget_auto_delegate")
        .unwrap();
    assert!(
        auto["inputSchema"]["properties"]
            .get("agent_type")
            .is_none()
    );
    assert!(
        auto["description"]
            .as_str()
            .unwrap()
            .contains("Omettre agent_type")
    );
    let explicit = catalogue
        .iter()
        .find(|t| t["name"] == "bridget_delegate")
        .unwrap();
    assert!(
        explicit["inputSchema"]["properties"]
            .get("agent_type")
            .is_some()
    );
    let identity = ResolvedIdentity {
        name: "owner".into(),
        instance_id: "instance".into(),
        delegated_origin: None,
    };
    for supplied in [json!("private-profile-never-echo"), Value::Null, json!(3)] {
        let refused = execute_with_proof(
            &identity,
            "bridget_auto_delegate",
            &json!({"request_id":"same200","task":"lecture","cwd":"/source","agent_type":supplied}),
            Path::new("/missing/never-connected200.sock"),
            None,
        );
        match refused {
            Err(ClientError::InvalidParams(message)) => {
                assert!(message.contains("agent_type"));
                assert!(message.contains("engine"));
                assert!(message.contains("avant connexion"));
                assert!(!message.contains("private-profile-never-echo"));
            }
            other => panic!("refus du champ avant transport attendu : {other:?}"),
        }
    }
    let corrected = execute_with_proof(
        &identity,
        "bridget_auto_delegate",
        &json!({"request_id":"same200","task":"lecture","cwd":"/source","machine_id":"sysle2","engine":"codex",
                "expected_base_commit":"a".repeat(40),"workspace":{"strategy":"worktree","base_ref":"a".repeat(40)}}),
        Path::new("/missing/never-connected200.sock"),
        None,
    );
    assert!(matches!(
        corrected,
        Err(ClientError::Technical {
            code: "daemon_unreachable",
            ..
        })
    ));
}

#[test]
fn spec174_source_arguments_expliques_et_sha_valide_avant_connexion() {
    let catalogue = tools(&json!({}));
    for name in ["bridget_delegate", "bridget_auto_delegate"] {
        let tool = catalogue.iter().find(|tool| tool["name"] == name).unwrap();
        let properties = &tool["inputSchema"]["properties"];
        assert!(
            properties["cwd"]["description"]
                .as_str()
                .unwrap()
                .contains("machine du parent")
        );
        assert!(
            properties["machine_id"]["description"]
                .as_str()
                .unwrap()
                .contains("destination")
        );
    }
    let automatic = catalogue
        .iter()
        .find(|tool| tool["name"] == "bridget_auto_delegate")
        .unwrap();
    assert_eq!(
        automatic["inputSchema"]["properties"]["expected_base_commit"]["pattern"],
        "^[0-9a-fA-F]{40}$"
    );
    let identity = ResolvedIdentity {
        name: "owner".into(),
        instance_id: "instance".into(),
        delegated_origin: None,
    };
    for invalid in [
        "09d8fa7e9".into(),
        "a".repeat(41),
        "a".repeat(63),
        "F".repeat(64),
        format!("{}\n", "a".repeat(40)),
        "g".repeat(40),
        "feature/private-ref".into(),
    ] {
        let result = execute_with_proof(
            &identity,
            "bridget_auto_delegate",
            &json!({"request_id":"source-sha","task":"lecture","cwd":"/source",
                    "expected_base_commit":invalid}),
            Path::new("/missing/source-sha.sock"),
            None,
        );
        match result {
            Err(ClientError::InvalidParams(message)) => {
                assert!(message.contains("40 caractères"));
                assert!(message.contains("SHA abrégé"));
                assert!(
                    !message.contains(&invalid),
                    "ne pas répéter la valeur fournie"
                );
            }
            other => panic!("refus de forme attendu avant connexion : {other:?}"),
        }
    }
    for valid in ["a".repeat(40), "F".repeat(40)] {
        let result = execute_with_proof(
            &identity,
            "bridget_auto_delegate",
            &json!({"request_id":"source-sha","task":"lecture","cwd":"/source",
                    "expected_base_commit":valid}),
            Path::new("/missing/source-sha.sock"),
            None,
        );
        assert!(matches!(
            result,
            Err(ClientError::Technical {
                code: "daemon_unreachable",
                ..
            })
        ));
    }
}

#[test]
fn spec221_c19_file_mcp_negotiates_feature_before_exact_native_request() {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixListener;
    for supplied_task in [false, true] {
        let socket = crate::mcp_identity::mock_socket("c19-file-mcp");
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
                json!({"type":"DaemonIdentityReport","host":"fixture","db_path":"fixture",
                    "instance_id":"fixture","features":["commit_view_files_v1"]})
            )
            .unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap();
            let request: WrapperToDaemon = serde_json::from_str(&line).unwrap();
            let WrapperToDaemon::NativeDelegation { request } = request else {
                panic!("requête native attendue");
            };
            assert_eq!(
                request,
                NativeDelegationRequest::ReadCommitFile {
                    task_id: supplied_task.then(|| "own-task".into()),
                    path: "sub/source.rs".into()
                }
            );
            writeln!(
                stream,
                "{}",
                json!({"type":"NativeDelegationResult",
                    "result":{"status":"available","file":"/private/view/sub/source.rs"}})
            )
            .unwrap();
        });
        let identity = ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        let args = if supplied_task {
            json!({"task_id":"own-task","path":"sub/source.rs"})
        } else {
            json!({"path":"sub/source.rs"})
        };
        let result =
            execute_with_proof(&identity, "bridget_read_commit_file", &args, &socket, None)
                .unwrap();
        assert_eq!(result["status"], "available");
        backend.join().unwrap();
    }
}

#[test]
fn spec168_wire163_byte_compatible_et_nouveau_client_refuse_ancien_maitre() {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixListener;
    let legacy: NativeDelegationRequest = serde_json::from_value(json!({
            "operation":"auto_delegate","request_id":"legacy163","task":"mission","cwd":"/tmp"}))
    .unwrap();
    assert_eq!(
        serde_json::to_string(&legacy).unwrap(),
        r#"{"operation":"auto_delegate","request_id":"legacy163","task":"mission","cwd":"/tmp"}"#
    );
    for (tool, args, expected) in [
        (
            "bridget_read_commit_file",
            json!({"path":"sub/file.rs"}),
            "commit_view_file_master_upgrade_required",
        ),
        (
            "bridget_auto_delegate",
            json!({"request_id":"new168","task":"mission","cwd":"/tmp"}),
            "auto_delegate_v2_unsupported",
        ),
        (
            "bridget_task_repair_workspace",
            json!({"task_id":"task185","request_id":"repair185",
                "target_cwd":"/srv/projet/task185","expected_head":"1".repeat(40)}),
            "workspace_repair_master_upgrade_required",
        ),
    ] {
        let socket = crate::mcp_identity::mock_socket("legacy168-refusal");
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
                json!({"type":"DaemonIdentityReport","host":"fixture","db_path":"fixture",
                "instance_id":"fixture","features":["auto_delegate_v1"]})
            )
            .unwrap();
            line.clear();
            assert_eq!(
                reader.read_line(&mut line).unwrap(),
                0,
                "aucun envoi V2 à un ancien maître"
            );
        });
        let identity = ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        let result = execute(&identity, tool, &args, &socket);
        assert!(matches!(result,Err(ClientError::Technical{message,..}) if message==expected));
        backend.join().unwrap();
        let _ = std::fs::remove_dir_all(socket.parent().unwrap());
    }
}

#[test]
fn spec168_mcp_auto_delegate_negocie_v2_et_preserve_le_schema_ferme() {
    use std::io::{BufRead, BufReader, Write};
    use std::os::fd::AsRawFd;
    use std::os::unix::net::UnixListener;
    let tool = tools(&json!({}))
        .into_iter()
        .find(|tool| tool["name"] == "bridget_auto_delegate")
        .unwrap();
    assert_eq!(tool["inputSchema"]["additionalProperties"], false);
    assert_eq!(
        tool["inputSchema"]["required"],
        json!(["request_id", "task", "cwd"])
    );
    assert!(tool["inputSchema"]["properties"].get("fallback").is_none());

    let socket = crate::mcp_identity::mock_socket("auto163-mcp");
    let listener = UnixListener::bind(&socket).unwrap();
    crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
    let backend = std::thread::spawn(move || {
        let mut pollfd = libc::pollfd {
            fd: listener.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: `pollfd` est une structure pollfd locale valide ; le nombre d'éléments passé vaut 1.
        assert!(unsafe { libc::poll(&mut pollfd, 1, 3000) } > 0);
        let (mut stream, _) = listener.accept().unwrap();
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
        writeln!(stream, "{}", json!({"type":"DaemonIdentityReport","host":"fixture","db_path":"fixture","instance_id":"fixture","features":["auto_delegate_v2"]})).unwrap();
        line.clear();
        reader.read_line(&mut line).unwrap();
        let request: Value = serde_json::from_str(&line).unwrap();
        writeln!(stream, "{}", json!({"type":"NativeDelegationResult","result":{"status":"accepted","task_id":"task-163"}})).unwrap();
        request
    });
    let identity = ResolvedIdentity {
        name: "owner".into(),
        instance_id: "instance".into(),
        delegated_origin: None,
    };
    let result = execute(
        &identity,
        "bridget_auto_delegate",
        &json!({"request_id":"auto-163","task":"mission","cwd":"/tmp",
                "brief":{"objective":"Choisir","summary":"Candidats bornés"},"urgent":true}),
        &socket,
    )
    .unwrap();
    assert_eq!(result["task_id"], "task-163");
    let request = backend.join().unwrap();
    assert_eq!(request["type"], "NativeDelegation");
    assert_eq!(request["request"]["operation"], "auto_delegate_v2");
    assert_eq!(request["request"]["urgent"], true);
    assert!(request["request"].get("fallback").is_none());
    let _ = std::fs::remove_dir_all(socket.parent().unwrap());
}

#[test]
fn native157_fallback_mcp_schema_ferme() {
    let delegate = crate::mcp::tools()
        .into_iter()
        .find(|tool| tool["name"] == "bridget_delegate")
        .unwrap();
    let fallback = &delegate["inputSchema"]["properties"]["fallback"];
    assert_eq!(fallback["type"], "array");
    assert_eq!(fallback["minItems"], 1);
    assert_eq!(fallback["maxItems"], 3);
    assert_eq!(fallback["items"]["type"], "object");
    assert_eq!(fallback["items"]["additionalProperties"], false);
    assert_eq!(
        fallback["items"]["required"],
        json!(["agent_type", "model"])
    );
    for field in ["agent_type", "model"] {
        assert_eq!(fallback["items"]["properties"][field]["type"], "string");
        assert_eq!(fallback["items"]["properties"][field]["minLength"], 1);
    }
    assert_eq!(fallback["items"]["properties"]["effort"]["type"], "string");
    assert_eq!(
        fallback["items"]["properties"].as_object().unwrap().len(),
        3
    );
    assert!(
        !delegate["inputSchema"]["required"]
            .as_array()
            .unwrap()
            .iter()
            .any(|key| key == "fallback")
    );
}

#[test]
fn native157_fallback_mcp_formes_et_canonique() {
    use std::io::{BufRead, BufReader, Write};
    use std::os::fd::AsRawFd;
    use std::os::unix::net::UnixListener;
    let identity = ResolvedIdentity {
        name: "owner".into(),
        instance_id: "instance".into(),
        delegated_origin: None,
    };
    let base = json!({"request_id":"k","agent_type":"a","model":"m","task":"t","cwd":"/tmp"});
    let socket = crate::mcp_identity::mock_socket("fallback157-invalid");
    let listener = UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
    for fallback in [
        json!(null),
        json!(false),
        json!("b"),
        json!({"agent_type":"b","model":"n"}),
        json!([]),
        json!(vec![json!({"agent_type":"b","model":"n"}); 4]),
        json!([null]),
        json!([{}]),
        json!([{"agent_type":"b"}]),
        json!([{"model":"n"}]),
        json!([{"agent_type":"","model":"n"}]),
        json!([{"agent_type":"b","model":""}]),
        json!([{"agent_type":2,"model":"n"}]),
        json!([{"agent_type":"b","model":2}]),
        json!([{"agent_type":"b","model":"n","effort":null}]),
        json!([{"agent_type":"b","model":"n","effort":2}]),
        json!([{"agent_type":"b","model":"n","permissions":"allow"}]),
        json!([{"agent_type":"b","model":"n"},{"agent_type":"b","model":"n"}]),
        json!([{"agent_type":"a","model":"m"}]),
    ] {
        let mut args = base.clone();
        args["fallback"] = fallback;
        let result = execute(&identity, "bridget_delegate", &args, &socket);
        assert!(
            matches!(result, Err(ClientError::InvalidParams(_))),
            "{args}: {result:?}"
        );
        assert!(
            matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
            "une forme invalide a ouvert la socket"
        );
    }
    drop(listener);
    let _ = std::fs::remove_dir_all(socket.parent().unwrap());

    // Réemploi du faux daemon MCP : absence byte-identique et liste bornée
    // transmise telle quelle, sans appeler de fournisseur.
    for (fallback, resume_on_limit) in [
        (None, None),
        (Some(json!([{"agent_type":"b","model":"n"}])), None),
        (
            Some(json!([{"agent_type":"b","model":"n"},
                {"agent_type":"c","model":"p","effort":"high"},
                {"agent_type":"d","model":"q"}])),
            None,
        ),
        (None, Some(false)),
        (None, Some(true)),
    ] {
        let socket = crate::mcp_identity::mock_socket("fallback157-valid");
        let listener = UnixListener::bind(&socket).unwrap();
        crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
        let backend = std::thread::spawn(move || {
            let mut pollfd = libc::pollfd {
                fd: listener.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            // SAFETY: `pollfd` est une structure pollfd locale valide ; le nombre d'éléments passé vaut 1.
            if unsafe { libc::poll(&mut pollfd, 1, 3000) } <= 0 {
                return None;
            }
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let registration: Value = serde_json::from_str(&line).unwrap();
            assert_eq!(registration["type"], "RegisterAuxiliary");
            writeln!(
                stream,
                "{}",
                json!({"type":"Registered","agent_id":"owner","credential":null})
            )
            .unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap();
            let frame: WrapperToDaemon = serde_json::from_str(&line).unwrap();
            let WrapperToDaemon::NativeDelegation { request } = frame else {
                panic!("trame native attendue : {frame:?}")
            };
            writeln!(
                stream,
                "{}",
                json!({"type":"NativeDelegationResult","result":{"status":"accepted"}})
            )
            .unwrap();
            Some(request)
        });
        let mut args = base.clone();
        if let Some(value) = &fallback {
            args["fallback"] = value.clone();
        }
        if let Some(value) = resume_on_limit {
            args["resume_on_limit"] = json!(value);
        }
        let result = execute(&identity, "bridget_delegate", &args, &socket);
        let request = backend.join().unwrap();
        let _ = std::fs::remove_dir_all(socket.parent().unwrap());
        assert!(
            result.is_ok(),
            "une liste valide doit atteindre le daemon : {args}: {result:?}"
        );
        let request = request.expect("requête transmise");
        if let Some(value) = fallback {
            assert_eq!(serde_json::to_value(request).unwrap()["fallback"], value);
        } else if let Some(expected) = resume_on_limit {
            let NativeDelegationRequest::Delegate {
                resume_on_limit, ..
            } = request
            else {
                panic!("Delegate attendu")
            };
            assert_eq!(resume_on_limit, Some(expected));
        } else {
            let historical = br#"{"operation":"delegate","request_id":"k","agent_type":"a","model":"m","effort":null,"task":"t","cwd":"/tmp","posture":null}"#;
            assert_eq!(serde_json::to_vec(&request).unwrap(), historical);
        }
    }
}

#[test]
fn spec221_workspace_invalid_explains_shape_before_any_connection() {
    use std::os::unix::net::UnixListener;
    let socket = crate::mcp_identity::mock_socket("workspace221-refusal");
    let listener = UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    let identity = ResolvedIdentity {
        name: "owner".into(),
        instance_id: "instance".into(),
        delegated_origin: None,
    };
    for name in ["bridget_delegate", "bridget_auto_delegate"] {
        for workspace in [
            json!({"strategy":"shared","base_ref":"PRIVATE_VALUE"}),
            json!({"strategy":"shared","branch":"PRIVATE_VALUE"}),
            json!({"strategy":"shared","PRIVATE_FIELD":true}),
            json!({"strategy":"worktree","base_ref":null}),
            json!({"strategy":"worktree","branch":false}),
            json!({"strategy":"worktree","PRIVATE_FIELD":"PRIVATE_VALUE"}),
            json!({"strategy":"PRIVATE_VALUE"}),
            json!({}),
            json!("PRIVATE_VALUE"),
            Value::Null,
        ] {
            let mut args =
                json!({"request_id":"stable-key","task":"read","cwd":"/tmp","workspace":workspace});
            if name == "bridget_delegate" {
                args["agent_type"] = json!("codex");
                args["model"] = json!("model");
            }
            let error = execute(&identity, name, &args, &socket).unwrap_err();
            let ClientError::InvalidParams(message) = error else {
                panic!("refus local attendu")
            };
            assert!(message.contains("strategy"), "{message}");
            assert!(message.contains("expected_base_commit"), "{message}");
            assert!(message.contains("cet appel"), "{message}");
            assert!(message.contains("antérieure"), "{message}");
            assert!(!message.contains("PRIVATE"), "{message}");
            assert!(message.len() < 1800);
            assert!(matches!(listener.accept(),Err(e) if e.kind()==std::io::ErrorKind::WouldBlock));
        }
    }
    drop(listener);
    std::fs::remove_dir_all(socket.parent().unwrap()).unwrap();
}

#[test]
fn native154_mcp_workspace_schema_ferme_et_champs_interdits() {
    let delegate = crate::mcp::tools()
        .into_iter()
        .find(|tool| tool["name"] == "bridget_delegate")
        .unwrap();
    let workspace = &delegate["inputSchema"]["properties"]["workspace"];
    let variants = workspace["oneOf"].as_array().unwrap();
    assert_eq!(variants.len(), 2);
    for variant in variants {
        assert_eq!(variant["type"], "object");
        assert_eq!(variant["additionalProperties"], false);
        assert_eq!(variant["required"], json!(["strategy"]));
        for forbidden in ["start_from_origin", "push", "merge", "cleanup"] {
            assert!(variant["properties"].get(forbidden).is_none());
        }
    }
    assert_eq!(variants[0]["properties"]["strategy"]["const"], "shared");
    assert_eq!(variants[1]["properties"]["strategy"]["const"], "worktree");
    let identity = ResolvedIdentity {
        name: "owner".into(),
        instance_id: "instance".into(),
        delegated_origin: None,
    };
    let base = json!({"request_id":"k","agent_type":"a","model":"m","task":"t","cwd":"/tmp"});
    for forbidden in ["start_from_origin", "push", "merge", "cleanup"] {
        let mut args = base.clone();
        args["workspace"] = json!({"strategy":"worktree"});
        args["workspace"][forbidden] = json!(true);
        assert!(matches!(
            execute(&identity, "bridget_delegate", &args, Path::new("/missing")),
            Err(ClientError::InvalidParams(_))
        ));
        args = base.clone();
        args[forbidden] = json!(true);
        assert!(matches!(
            execute(&identity, "bridget_delegate", &args, Path::new("/missing")),
            Err(ClientError::InvalidParams(_))
        ));
    }
    for valid in [
        json!({"strategy":"shared"}),
        json!({"strategy":"worktree","base_ref":"main","branch":"bridget/task"}),
    ] {
        let mut args = base.clone();
        args["workspace"] = valid;
        assert!(!matches!(
            execute(&identity, "bridget_delegate", &args, Path::new("/missing")),
            Err(ClientError::InvalidParams(_))
        ));
    }
}

#[test]
fn native151_facade_task_wait_schema_ferme_et_lecture_pure() {
    let wait = crate::mcp::tools()
        .into_iter()
        .find(|tool| tool["name"] == "bridget_task_wait")
        .expect("outil d'attente présent");
    assert_eq!(wait["inputSchema"]["additionalProperties"], false);
    assert_eq!(wait["inputSchema"]["required"], json!(["task_ids"]));
    assert_eq!(wait["inputSchema"]["properties"]["task_ids"]["minItems"], 1);
    assert_eq!(
        wait["inputSchema"]["properties"]["task_ids"]["maxItems"],
        16
    );
    assert_eq!(
        wait["inputSchema"]["properties"]["task_ids"]["uniqueItems"],
        true
    );
    assert_eq!(
        wait["inputSchema"]["properties"]["wait_timeout"]["default"],
        600
    );
    assert_eq!(
        wait["inputSchema"]["properties"]["wait_timeout"]["maximum"],
        3600
    );
    assert_eq!(wait["annotations"]["readOnlyHint"], true);
    assert_eq!(wait["annotations"]["destructiveHint"], false);
}

#[test]
fn native151_facade_task_wait_bornes_et_types_avant_connexion() {
    let identity = ResolvedIdentity {
        name: "owner".into(),
        instance_id: "instance".into(),
        delegated_origin: None,
    };
    for value in [json!(-1), json!(0), json!(3601), json!(u64::MAX)] {
        let refusal = execute(
            &identity,
            "bridget_task_wait",
            &json!({"task_ids":["task"],"wait_timeout":value}),
            Path::new("/missing"),
        );
        assert!(
            matches!(refusal, Err(ClientError::Technical { code: "native_delegation_refused", message }) if message == "wait_timeout_out_of_range")
        );
    }
    for value in [json!(null), json!(1.5), json!("1"), json!(true)] {
        assert!(matches!(
            execute(
                &identity,
                "bridget_task_wait",
                &json!({"task_ids":["task"],"wait_timeout":value}),
                Path::new("/missing")
            ),
            Err(ClientError::InvalidParams(_))
        ));
    }
    for args in [
        json!({"task_ids":["task"],"parent":"forged"}),
        json!({"task_ids":[]}),
        json!({"task_ids":["task","task"]}),
        json!({"task_ids":vec!["task";17]}),
        json!({"task_ids":[""]}),
        json!({"task_ids":[1]}),
    ] {
        assert!(matches!(
            execute(&identity, "bridget_task_wait", &args, Path::new("/missing")),
            Err(ClientError::InvalidParams(_))
        ));
    }
    for value in [None, Some(json!(1)), Some(json!(3600))] {
        let mut args = json!({"task_ids":["task"]});
        if let Some(value) = value {
            args["wait_timeout"] = value;
        }
        assert!(matches!(
            execute(&identity, "bridget_task_wait", &args, Path::new("/missing")),
            Err(ClientError::Technical {
                code: "daemon_unreachable",
                ..
            })
        ));
    }
}

#[test]
fn closed_arguments_reject_forged_parent_without_connecting() {
    let identity = ResolvedIdentity {
        name: "owner".into(),
        instance_id: "instance".into(),
        delegated_origin: None,
    };
    assert!(matches!(
        execute(
            &identity,
            "bridget_delegate",
            &json!({"parent":"other"}),
            Path::new("/missing")
        ),
        Err(ClientError::InvalidParams(_))
    ));
    for invalid in [
        json!({"request_id":"auto-task","task":"x".repeat(65_537),"cwd":"/tmp"}),
        json!({"request_id":"auto-canon","task":"t","cwd":"/tmp",
                "brief":{"objective":"x".repeat(512 * 1024)}}),
        json!({"request_id":"auto-parent","task":"t","cwd":"/tmp","parent":"other"}),
    ] {
        assert!(matches!(
            execute(
                &identity,
                "bridget_auto_delegate",
                &invalid,
                Path::new("/missing")
            ),
            Err(ClientError::InvalidParams(_))
        ));
    }
    let catalogue = crate::mcp::tools();
    let brief_schema = &catalogue
        .iter()
        .find(|tool| tool["name"] == "bridget_handoff")
        .unwrap()["inputSchema"]["properties"]["draft"];
    let native_tools = tools(brief_schema);
    assert_eq!(native_tools.len(), 13);
    assert!(
        native_tools
            .iter()
            .any(|tool| tool["name"] == "bridget_specialists")
    );
    assert!(
        native_tools
            .iter()
            .any(|tool| tool["name"] == "bridget_refresh_models")
    );
    assert!(
        native_tools
            .iter()
            .any(|tool| tool["name"] == "bridget_read_commit_file")
    );
}

#[test]
fn native150_delai_schema_liste_blanche_et_forme() {
    let delegate = crate::mcp::tools()
        .into_iter()
        .find(|tool| tool["name"] == "bridget_delegate")
        .unwrap();
    let property = &delegate["inputSchema"]["properties"]["reply_timeout"];
    assert_eq!(property["type"], "integer");
    assert_eq!(
        (property["minimum"].as_u64(), property["maximum"].as_u64()),
        (Some(60), Some(86_400))
    );
    assert!(
        !delegate["inputSchema"]["required"]
            .as_array()
            .unwrap()
            .iter()
            .any(|key| key == "reply_timeout")
    );
    let identity = ResolvedIdentity {
        name: "owner".into(),
        instance_id: "instance".into(),
        delegated_origin: None,
    };
    let base = |extra: Value| {
        let mut args =
            json!({"request_id":"k","agent_type":"a","model":"m","task":"t","cwd":"/tmp"});
        args.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        args
    };
    // Forme invalide : refusée avant toute connexion. Les bornes restent au daemon.
    for invalide in [json!("abc"), json!(-5), json!(60.5), json!(true)] {
        assert!(matches!(
            execute(
                &identity,
                "bridget_delegate",
                &base(json!({"reply_timeout": invalide})),
                Path::new("/missing")
            ),
            Err(ClientError::InvalidParams(_))
        ));
    }
    // Forme valide : le champ passe la liste blanche, l'échec vient de la connexion.
    for valide in [json!(120), json!(null)] {
        assert!(!matches!(
            execute(
                &identity,
                "bridget_delegate",
                &base(json!({"reply_timeout": valide})),
                Path::new("/missing")
            ),
            Err(ClientError::InvalidParams(_))
        ));
    }
}

#[test]
fn native157_quota_detail_visible_cote_mcp() {
    use std::io::{BufRead, BufReader, Write};
    use std::os::fd::AsRawFd;
    use std::os::unix::net::UnixListener;
    for (returned, expected_code, expected_detail) in [
        (
            json!({"status":"refused","code":"project_preparation_source_busy","detail":{
                    "cause":"project_preparation_source_busy","verification":"origin_project_preparation",
                    "message":"Le verrou source est resté occupé pendant30s.",
                    "recovery":"Conserver la clé et vérifier l'admission durable avant tout rejeu strict."}}),
            "project_preparation_source_busy",
            Some(
                json!({"cause":"project_preparation_source_busy","verification":"origin_project_preparation",
                    "message":"Le verrou source est resté occupé pendant30s.",
                    "recovery":"Conserver la clé et vérifier l'admission durable avant tout rejeu strict."}),
            ),
        ),
        (
            json!({"status":"refused","code":"project_preparation_busy","detail":{
                    "cause":"project_preparation_busy","verification":"origin_project_preparation",
                    "message":"Le verrou distant est occupé.","recovery":"Ne pas créer de remplacement."}}),
            "project_preparation_busy",
            Some(
                json!({"cause":"project_preparation_busy","verification":"origin_project_preparation",
                    "message":"Le verrou distant est occupé.","recovery":"Ne pas créer de remplacement."}),
            ),
        ),
        (
            json!({"status":"accepted","features":{"native_delegation":true}}),
            "catalogue160",
            None,
        ),
        (
            json!({"status":"refused","code":"provider_quota_exhausted","detail":{"resets_at":1_900_000_000}}),
            "provider_quota_exhausted",
            Some(json!({"resets_at":1_900_000_000})),
        ),
        (
            json!({"status":"refused","code":"remote_execution_ended","message":"La commande distante est terminée ou son lancement a été refusé.","detail":{"message":"La commande distante est terminée ou son lancement a été refusé."}}),
            "remote_execution_ended",
            Some(
                json!({"message":"La commande distante est terminée ou son lancement a été refusé."}),
            ),
        ),
        (
            json!({"status":"refused","code":"remote_execution_unconfirmed","message":"Vérifiez l’état de la tâche sans créer de doublon.","detail":{"message":"Vérifiez l’état de la tâche sans créer de doublon."}}),
            "remote_execution_unconfirmed",
            Some(json!({"message":"Vérifiez l’état de la tâche sans créer de doublon."})),
        ),
        (
            json!({"status":"refused","code":"workspace_preflight_technical","detail":{"message":"Le contrôle préalable du dossier source a refusé la demande.","recovery":"Le champ cwd doit désigner le dossier source sur la machine du parent qui appelle Bridget, pas le dossier de destination. Vérifiez que ce dossier existe, que Git peut le lire et que la référence demandée est disponible. Avant un nouvel essai, vérifiez si une tâche existe déjà pour cette demande ; ne créez pas de doublon et ne changez pas la base automatiquement."}}),
            "workspace_preflight_technical",
            Some(
                json!({"message":"Le contrôle préalable du dossier source a refusé la demande.","recovery":"Le champ cwd doit désigner le dossier source sur la machine du parent qui appelle Bridget, pas le dossier de destination. Vérifiez que ce dossier existe, que Git peut le lire et que la référence demandée est disponible. Avant un nouvel essai, vérifiez si une tâche existe déjà pour cette demande ; ne créez pas de doublon et ne changez pas la base automatiquement."}),
            ),
        ),
        (
            json!({"status":"refused","code":"workspace_base_preflight_failed","detail":{"message":"Bridget n’a pas pu contrôler le dossier source avant l’admission.","recovery":"Le champ cwd désigne un dossier sur la machine du parent appelant. machine_id choisit seulement la destination : vérifier le chemin par SSH sur cette destination ne valide pas la source. Sur la machine du parent, vérifiez que cwd existe et est lisible, puis exécutez git rev-parse --show-toplevel et git rev-parse HEAD dans ce dossier. Corrigez le chemin ou l’accès Git en cas d’échec. En shared, le HEAD source doit être égal à expected_base_commit. Ne changez pas le SHA attendu et ne déplacez pas un checkout utilisé pour contourner le refus. Si shared est imposé, conservez ce mode ; sinon, pour une base figée en worktree autorisé, donnez le même SHA complet à workspace.base_ref et expected_base_commit. Ce refus technique ne clôt pas une admission déjà en cours et ne prouve pas l’absence de tâche. Suivez toute tâche connue avec bridget_task_status. Un rejeu strict conserve request_id et tous les paramètres : admission_pending signifie que la demande reste en attente. Avant une nouvelle clé pour des paramètres corrigés, confirmez qu’aucune tâche et aucune admission en cours ne subsistent pour l’ancienne demande. Si le sort reste inconnu, ne créez pas de nouvelle demande."}}),
            "workspace_base_preflight_failed",
            Some(
                json!({"message":"Bridget n’a pas pu contrôler le dossier source avant l’admission.","recovery":"Le champ cwd désigne un dossier sur la machine du parent appelant. machine_id choisit seulement la destination : vérifier le chemin par SSH sur cette destination ne valide pas la source. Sur la machine du parent, vérifiez que cwd existe et est lisible, puis exécutez git rev-parse --show-toplevel et git rev-parse HEAD dans ce dossier. Corrigez le chemin ou l’accès Git en cas d’échec. En shared, le HEAD source doit être égal à expected_base_commit. Ne changez pas le SHA attendu et ne déplacez pas un checkout utilisé pour contourner le refus. Si shared est imposé, conservez ce mode ; sinon, pour une base figée en worktree autorisé, donnez le même SHA complet à workspace.base_ref et expected_base_commit. Ce refus technique ne clôt pas une admission déjà en cours et ne prouve pas l’absence de tâche. Suivez toute tâche connue avec bridget_task_status. Un rejeu strict conserve request_id et tous les paramètres : admission_pending signifie que la demande reste en attente. Avant une nouvelle clé pour des paramètres corrigés, confirmez qu’aucune tâche et aucune admission en cours ne subsistent pour l’ancienne demande. Si le sort reste inconnu, ne créez pas de nouvelle demande."}),
            ),
        ),
        (
            json!({"status":"refused","code":"expected_base_commit_mismatch","detail":{"message":"La base résolue ne correspond pas au commit exact demandé.","recovery":"Pour une revue figée, utiliser le même SHA complet dans workspace.base_ref et expected_base_commit."}}),
            "expected_base_commit_mismatch",
            Some(
                json!({"message":"La base résolue ne correspond pas au commit exact demandé.","recovery":"Pour une revue figée, utiliser le même SHA complet dans workspace.base_ref et expected_base_commit."}),
            ),
        ),
        (
            json!({"status":"refused","code":"permission_source_unavailable"}),
            "permission_source_unavailable",
            None,
        ),
        (
            json!({"status":"refused","code":"workspace_base_unavailable","message":"Bridget n’a pas pu déterminer le commit de base dans le dépôt source.","detail":{"message":"Bridget n’a pas pu déterminer le commit de base dans le dépôt source.","recovery":"Dans cwd, sur la machine du parent, vérifiez que Git peut résoudre le commit demandé. Sa présence sur la machine de destination ne suffit pas. Si le commit manque, identifiez le dépôt distant et la branche qui le contiennent, puis récupérez cette branche avec git fetch dans le dépôt source. Vérifiez à nouveau le SHA exact, sans changer expected_base_commit, faire de reset ou avancer la branche de travail. Si une tâche existe, suivez-la avec bridget_task_status. Sinon, après correction, faites au plus un rejeu strict avec la même request_id et les mêmes paramètres. Un refus enregistré reste un refus au rejeu : si ce refus durable et l’absence de tâche sont confirmés, utilisez une nouvelle request_id après correction. Ne multipliez pas les essais si le sort reste inconnu. Ce code ne désigne pas le plafond des worktrees."}}),
            "workspace_base_unavailable",
            Some(
                json!({"message":"Bridget n’a pas pu déterminer le commit de base dans le dépôt source.","recovery":"Dans cwd, sur la machine du parent, vérifiez que Git peut résoudre le commit demandé. Sa présence sur la machine de destination ne suffit pas. Si le commit manque, identifiez le dépôt distant et la branche qui le contiennent, puis récupérez cette branche avec git fetch dans le dépôt source. Vérifiez à nouveau le SHA exact, sans changer expected_base_commit, faire de reset ou avancer la branche de travail. Si une tâche existe, suivez-la avec bridget_task_status. Sinon, après correction, faites au plus un rejeu strict avec la même request_id et les mêmes paramètres. Un refus enregistré reste un refus au rejeu : si ce refus durable et l’absence de tâche sont confirmés, utilisez une nouvelle request_id après correction. Ne multipliez pas les essais si le sort reste inconnu. Ce code ne désigne pas le plafond des worktrees."}),
            ),
        ),
    ] {
        let socket = crate::mcp_identity::mock_socket("quota157");
        let listener = UnixListener::bind(&socket).unwrap();
        crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
        let backend = std::thread::spawn(move || {
            let mut pollfd = libc::pollfd {
                fd: listener.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            assert!(
                // SAFETY: `pollfd` est une structure pollfd locale valide ; le nombre d'éléments passé vaut 1.
                unsafe { libc::poll(&mut pollfd, 1, 3000) } > 0,
                "connexion MCP attendue"
            );
            let (mut stream, _) = listener.accept().unwrap();
            stream.set_nonblocking(false).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let registration: Value = serde_json::from_str(&line).unwrap();
            assert_eq!(registration["type"], "RegisterAuxiliary");
            assert_eq!(registration["agent_id"], "owner");
            writeln!(
                stream,
                "{}",
                json!({"type":"Registered","agent_id":"owner","credential":null})
            )
            .unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap();
            let request: Value = serde_json::from_str(&line).unwrap();
            assert_eq!(request["type"], "NativeDelegation");
            writeln!(
                stream,
                "{}",
                json!({"type":"NativeDelegationResult","result":returned})
            )
            .unwrap();
            request
        });
        let identity = ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        let args = json!({"request_id":"quota157","agent_type":"codex","model":"fixture","task":"fixture","cwd":socket.parent().unwrap()});
        let catalogue = expected_code == "catalogue160";
        let catalogue_args = json!({});
        let result = execute(
            &identity,
            if catalogue {
                "bridget_capabilities"
            } else {
                "bridget_delegate"
            },
            if catalogue { &catalogue_args } else { &args },
            &socket,
        );
        let request = backend.join().unwrap();
        if catalogue {
            assert_eq!(request["request"]["operation"], "catalogue");
            let result = result.unwrap();
            assert_eq!(
                result["features"],
                json!({"native_delegation":true,
                    "resume_on_limit":true,"resume_max_wait_secs":21600,"resume_max_rounds":2})
            );
            let _ = std::fs::remove_dir_all(socket.parent().unwrap());
            continue;
        }
        assert_eq!(request["request"]["agent_type"], "codex");
        let Err(ClientError::Technical { code, message }) = result else {
            panic!("refus technique MCP attendu : {result:?}")
        };
        assert_eq!(code, "native_delegation_refused");
        if let Some(detail) = expected_detail {
            let (actual_code, raw_detail) = message
                .split_once(' ')
                .expect("code suivi du JSON de detail");
            assert_eq!(actual_code, expected_code);
            assert_eq!(serde_json::from_str::<Value>(raw_detail).unwrap(), detail);
        } else {
            assert_eq!(
                message, expected_code,
                "les autres refus gardent le message historique"
            );
        }
        let _ = std::fs::remove_dir_all(socket.parent().unwrap());
    }
}

#[test]
fn native160_optin_sans_champ_octets_identiques() {
    use std::os::unix::net::UnixListener;
    let tool = tools(&json!({}))
        .into_iter()
        .find(|tool| tool["name"] == "bridget_delegate")
        .unwrap();
    let schema = &tool["inputSchema"];
    assert_eq!(schema["properties"]["resume_on_limit"]["type"], "boolean");
    assert!(
        !schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field == "resume_on_limit")
    );
    assert_eq!(
        schema["properties"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        [
            "agent_type",
            "brief",
            "child_requests",
            "cwd",
            "effort",
            "fallback",
            "keep_alive_secs",
            "machine_id",
            "model",
            "posture",
            "project_id",
            "quota_fallback_authorized",
            "reply_timeout",
            "request_id",
            "resume_on_limit",
            "task",
            "workspace"
        ]
    );
    let historical = br#"{"operation":"delegate","request_id":"k","agent_type":"a","model":"m","effort":null,"task":"t","cwd":"/tmp","posture":null}"#;
    let request: NativeDelegationRequest = serde_json::from_slice(historical).unwrap();
    assert_eq!(serde_json::to_vec(&request).unwrap(), historical);
    let socket = crate::mcp_identity::mock_socket("resume160-invalid");
    let listener = UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    let identity = ResolvedIdentity {
        name: "owner".into(),
        instance_id: "instance".into(),
        delegated_origin: None,
    };
    for value in [
        json!(null),
        json!(0),
        json!(1),
        json!(0.5),
        json!("true"),
        json!([]),
        json!({}),
    ] {
        let args = json!({"request_id":"k","agent_type":"a","model":"m","task":"t","cwd":"/tmp","resume_on_limit":value});
        assert!(
            matches!(
                execute(&identity, "bridget_delegate", &args, &socket),
                Err(ClientError::InvalidParams(_))
            ),
            "{args}"
        );
        assert!(
            matches!(listener.accept(),Err(error) if error.kind()==std::io::ErrorKind::WouldBlock)
        );
    }
    drop(listener);
    let _ = std::fs::remove_dir_all(socket.parent().unwrap());
}

#[test]
fn native160_optin_absent_et_faux_deux_enveloppes() {
    let historical = br#"{"operation":"delegate","request_id":"k","agent_type":"a","model":"m","effort":null,"task":"t","cwd":"/tmp","posture":null}"#;
    let absent: NativeDelegationRequest = serde_json::from_slice(historical).unwrap();
    let mut explicit_false = absent.clone();
    let NativeDelegationRequest::Delegate {
        resume_on_limit, ..
    } = &mut explicit_false
    else {
        panic!("Delegate attendu")
    };
    *resume_on_limit = Some(false);
    let absent_canonical = serde_json::to_vec(&absent).unwrap();
    let false_canonical = serde_json::to_vec(&explicit_false).unwrap();
    assert_ne!(absent_canonical, false_canonical);
    let store = crate::delegation::DelegationStore::open(Path::new(":memory:")).unwrap();
    store
        .record_refusal("owner", "k", &absent_canonical, "model_unavailable", 100)
        .unwrap();
    assert_eq!(
        store
            .by_agent_request("owner", "k", &false_canonical)
            .unwrap_err(),
        "envelope_mismatch"
    );
    assert_eq!(
        store
            .by_agent_request("owner", "k", &absent_canonical)
            .unwrap_err(),
        "model_unavailable"
    );
}

#[test]
fn spec285_specialists_version_type_names_real_refusal_without_connection() {
    let identity = ResolvedIdentity {
        name: "owner".into(),
        instance_id: "instance".into(),
        delegated_origin: None,
    };
    for version in [json!("1"), Value::Null, json!(true), json!(1.5), json!(-1)] {
        let error = execute_with_proof(
            &identity,
            "bridget_specialists",
            &json!({"version":version,"action":"search","role_id":"mainteneur-bridget"}),
            Path::new("/missing/never-connected285.sock"),
            None,
        );
        assert!(matches!(error, Err(ClientError::InvalidParams(message))
            if message.contains("nombre JSON") && message.contains("Aucun appel envoyé")
                && !message.contains("L’action catalogue n’existe pas")));
    }
    let valid = execute_with_proof(
        &identity,
        "bridget_specialists",
        &json!({"version":1,"action":"search","role_id":"mainteneur-bridget"}),
        Path::new("/missing/never-connected285.sock"),
        None,
    );
    assert!(valid.is_err());
    assert!(!matches!(valid, Err(ClientError::InvalidParams(_))));
}
