    use super::*;
    use serde_json::Value;
    use std::cell::Cell;
    use std::collections::BTreeSet;
    use std::io::Cursor;
    use std::os::unix::net::UnixListener;
    use std::path::PathBuf;
    use std::sync::{Barrier, mpsc};
    use std::thread;

    const FIXTURES: &str = include_str!("../tests/fixtures/mcp/fr009.jsonl");

    #[test]
    fn spec228_catalogue_input_schemas_are_objects_before_identity() {
        let input = format!(
            "{}\n{}\n{}\n",
            json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
            json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
            json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
        );
        let mut output = Vec::new();
        let identity = || -> Result<crate::mcp_identity::ResolvedIdentity, crate::mcp_identity::IdentityError> {
            panic!("le catalogue ne doit pas exiger une identité")
        };
        let execute = |_: &crate::mcp_identity::ResolvedIdentity, _: &str, _: &Value| -> Result<Value, ToolError> {
            panic!("aucune opération métier pendant la découverte")
        };
        serve_with(Cursor::new(input), &mut output, &identity, &execute).unwrap();
        let responses: Vec<Value> = String::from_utf8(output).unwrap().lines()
            .map(|line| serde_json::from_str(line).unwrap()).collect();
        let catalogue = responses.iter().find(|r| r["id"] == 2).unwrap()["result"]["tools"]
            .as_array().unwrap().clone();
        assert!(!catalogue.is_empty());
        for tool in &catalogue {
            assert_eq!(tool["inputSchema"]["type"], "object", "{}", tool["name"]);
            for key in ["allOf", "anyOf", "oneOf"] {
                assert!(tool["inputSchema"].get(key).is_none(), "racine de {} : {key}", tool["name"]);
            }
        }
        let canonical = tools();
        assert_eq!(catalogue.len(), canonical.len());
        let mut referenced = BTreeSet::new();
        for (wire, original) in catalogue.iter().zip(&canonical) {
            assert_eq!(wire["name"], original["name"]);
            let schema = &wire["inputSchema"];
            let restored = if let Some(reference) = schema.get("$ref") {
                let key = reference.as_str().unwrap().strip_prefix("#/$defs/").unwrap();
                referenced.insert(wire["name"].as_str().unwrap());
                &schema["$defs"][key]
            } else { schema };
            assert_eq!(*restored, original["inputSchema"], "contraintes de {}", wire["name"]);
        }
        assert_eq!(referenced, BTreeSet::from(["bridget_auto_delegate", "bridget_domain",
            "bridget_queue", "bridget_send", "bridget_specialists", "bridget_wake"]));
    }

    #[test]
    fn edit159_mcp_schema() {
        let queue = tools()
            .into_iter()
            .find(|tool| tool["name"] == "bridget_queue")
            .unwrap();
        assert_eq!(
            queue["inputSchema"]["properties"]["action"]["enum"],
            json!(["list", "cancel", "reorder", "promote"])
        );
        assert_eq!(
            queue["inputSchema"]["properties"]["action"]["default"],
            "list"
        );
        assert_eq!(queue["inputSchema"]["required"], json!(["target"]));
        assert_eq!(queue["inputSchema"]["additionalProperties"], false);
        assert_eq!(
            queue["inputSchema"]["properties"]["request_id"]["minLength"],
            1
        );
        assert_eq!(
            queue["inputSchema"]["properties"]["before_message_id"]["type"],
            json!(["string", "null"])
        );
        assert_eq!(queue["annotations"]["readOnlyHint"], false);
        assert!(
            queue["inputSchema"]["allOf"]
                .as_array()
                .unwrap()
                .iter()
                .any(|rule| rule["then"]["required"] == json!(["message_id", "request_id"]))
        );
    }

    #[test]
    fn edit159_mcp_request_id_obligatoire() {
        let socket = test_socket("edit159-required-id");
        for action in ["cancel", "reorder", "promote"] {
            for request_id in [None, Some(json!(""))] {
                let mut args = json!({"target":"89000000-0000-4000-8000-000000000159",
                    "action":action,"message_id":"m159"});
                if action == "reorder" {
                    args["before_message_id"] = Value::Null;
                }
                if let Some(value) = request_id {
                    args["request_id"] = value;
                }
                let result = execute_tool_at(
                    "89000000-0000-4000-8000-000000000156",
                    "bridget_queue",
                    args.as_object().unwrap(),
                    &socket,
                );
                assert!(
                    matches!(result, Err(ToolError::InvalidParams(ref reason))
                    if reason.contains("request_id")),
                    "forme refusée avant connexion : {result:?}"
                );
            }
        }
        let _ = std::fs::remove_dir_all(socket.parent().unwrap());
    }

    #[test]
    fn edit159_mcp_preuve_enfant_refusee() {
        let socket = test_socket("edit159-child-proof");
        let listener = UnixListener::bind(&socket).unwrap();
        listener.set_nonblocking(true).unwrap();
        let child = crate::mcp_identity::ResolvedIdentity {
            name: "89000000-0000-4000-8000-000000000156".into(),
            instance_id: "edit159-child-instance".into(),
            delegated_origin: Some(bridget_core::DelegatedOrigin {
                provider: "codex".into(),
                child_ref: "opaque-child-proof159".into(),
            }),
        };
        for action in ["list", "cancel", "reorder", "promote"] {
            let mut args = json!({"target":"89000000-0000-4000-8000-000000000159","action":action});
            if action != "list" {
                args["message_id"] = json!("m159");
                args["request_id"] = json!("r159");
            }
            if action == "reorder" {
                args["before_message_id"] = Value::Null;
            }
            let result = execute_tool_at_with_identity(
                &child,
                "bridget_queue",
                args.as_object().unwrap(),
                &socket,
            );
            assert!(
                matches!(
                    result,
                    Err(ToolError::Technical {
                        code: "delegated_tool_forbidden",
                        ..
                    })
                ),
                "{result:?}"
            );
        }
        assert!(
            matches!(listener.accept(), Err(error) if error.kind() == io::ErrorKind::WouldBlock)
        );
        drop(listener);
        std::fs::remove_dir_all(socket.parent().unwrap()).unwrap();
    }

    #[test]
    fn guidance156_mcp_queue_schema() {
        // ADR044 : origine enfant attestée par le résolveur, pas par le
        // payload QueueList et jamais par un issuer_scope inventé.
        let socket = test_socket("g156-childproof");
        let listener = UnixListener::bind(&socket).unwrap();
        listener.set_nonblocking(true).unwrap();
        let child = crate::mcp_identity::ResolvedIdentity {
            name: "89000000-0000-4000-8000-000000000156".into(),
            instance_id: "guidance156-childproof".into(),
            delegated_origin: Some(bridget_core::DelegatedOrigin {
                provider: "codex".into(),
                child_ref: "opaque-child-proof156".into(),
            }),
        };
        // Appel direct : le seul filtre global du dispatch MCP ne suffit pas.
        let refused = super::execute_tool_at_with_identity(
            &child,
            "bridget_queue",
            json!({"target":"89000000-0000-4000-8000-000000000157"})
                .as_object()
                .unwrap(),
            &socket,
        );
        let no_connection =
            matches!(listener.accept(), Err(error) if error.kind() == io::ErrorKind::WouldBlock);
        drop(listener);
        std::fs::remove_dir_all(socket.parent().unwrap()).unwrap();
        assert!(
            matches!(
                refused,
                Err(ToolError::Technical {
                    code: "delegated_tool_forbidden",
                    ..
                })
            ),
            "preuve enfant refusée : {refused:?}"
        );
        assert!(
            no_connection,
            "refus avant toute connexion, enregistrement ou négociation QueueListV1"
        );

        let queue = tools()
            .into_iter()
            .find(|tool| tool["name"] == "bridget_queue")
            .expect("outil de lecture de file publié en phase V");
        assert_eq!(queue["inputSchema"]["required"], json!(["target"]));
        assert_eq!(queue["inputSchema"]["additionalProperties"], false);
        assert_eq!(
            queue["inputSchema"]["properties"]
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            [
                "action",
                "before_message_id",
                "message_id",
                "request_id",
                "target"
            ]
        );
        assert_eq!(
            queue["inputSchema"]["properties"]["target"]["type"],
            "string"
        );
        assert_eq!(queue["inputSchema"]["properties"]["target"]["minLength"], 1);
        assert_eq!(queue["annotations"]["readOnlyHint"], false);
        assert_eq!(queue["annotations"]["destructiveHint"], true);
    }

    // Trois oracles wire partagent ce réemploi des helpers MCP existants.
    fn guidance156_capture_send(arguments: Value) -> Result<(String, Value), ToolError> {
        guidance156_capture_send_with_information(arguments, true)
    }

    fn guidance156_capture_send_with_information(arguments: Value, supported: bool) -> Result<(String, Value), ToolError> {
        let information=arguments.get("intent").and_then(Value::as_str)==Some("information");
        let socket = test_socket("g156");
        let listener = UnixListener::bind(&socket).unwrap();
        listener.set_nonblocking(true).unwrap();
        let (stop, stopped) = mpsc::channel();
        let server = thread::spawn(move || {
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            let deadline = Instant::now() + Duration::from_secs(3);
            let stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break Some(stream),
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        if stopped.try_recv().is_ok() || Instant::now() >= deadline {
                            break None;
                        }
                        thread::sleep(Duration::from_millis(1));
                    }
                    Err(error) => panic!("socket privée : {error}"),
                }
            };
            let stream = stream?;
            stream.set_nonblocking(false).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Client
                }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client,
                },
            );
            assert!(
                matches!(read_command(&mut reader), WrapperToDaemon::ClientHello { capabilities, .. }
                    if capabilities == if information { vec![ClientCapability::SendIdempotent,ClientCapability::SendInformationV1] } else { vec![ClientCapability::SendIdempotent] })
            );
            write_command(
                &mut writer,
                DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: "test-build".into(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: if information && supported {vec![ClientCapability::SendIdempotent,ClientCapability::SendInformationV1]} else {vec![ClientCapability::SendIdempotent]},
                },
            );
            if information && !supported {
                assert!(reader.read_line(&mut String::new()).unwrap()==0,"aucun Send après capacité absente");
                return None;
            }
            let sent = read_command(&mut reader);
            assert!(matches!(&sent, WrapperToDaemon::SendIdempotent { .. }));
            let bytes = encode(&sent).unwrap();
            write_command(
                &mut writer,
                DaemonToWrapper::IdempotencyResult {
                    operation_kind: "send".into(),
                    idempotency_key: "guidance156".into(),
                    issue: IdempotencyIssue::Accepted { expires_at: 60 },
                    project_warnings: Vec::new(),
                },
            );
            Some(bytes)
        });
        let result = execute_tool_at(
            "89000000-0000-4000-8000-000000000156",
            "bridget_send",
            arguments.as_object().unwrap(),
            &socket,
        );
        let _ = stop.send(());
        let bytes = server.join().unwrap();
        std::fs::remove_dir_all(socket.parent().unwrap()).unwrap();
        result.map(|result| (bytes.expect("vraie trame SendIdempotent reçue"), result))
    }

    #[test]
    fn guidance156_mcp_schema_intent() {
        let send = tools()
            .into_iter()
            .find(|tool| tool["name"] == "bridget_send")
            .unwrap();
        assert_eq!(
            send["inputSchema"]["properties"]["intent"]["enum"],
            json!(["queue", "steer", "interrupt", "information"])
        );
        assert_eq!(
            send["inputSchema"]["properties"]["intent"]["default"],
            "queue"
        );
        assert_eq!(send["inputSchema"]["additionalProperties"], false);
        assert_eq!(send["inputSchema"]["required"], json!(["to", "body"]));

        let socket = test_socket("g156-caps");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            let (stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            let WrapperToDaemon::RegisterAuxiliary { agent_id, .. } = read_command(&mut reader)
            else {
                panic!("identité auxiliaire requise")
            };
            write_command(
                &mut writer,
                DaemonToWrapper::Registered {
                    agent_id,
                    credential: None,
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::NativeDelegation {
                    request: bridget_transport::protocol::NativeDelegationRequest::Catalogue
                } | WrapperToDaemon::NativeDelegationT3 {
                    request: bridget_transport::protocol::NativeDelegationRequest::Catalogue,
                    ..
                }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::NativeDelegationResult {
                    result: json!({"providers":[{"agent_type":"fakeglm","queue":{"held":true}}],"queue":{"held":false}}),
                },
            );
        });
        let capabilities = execute_tool_at(
            "89000000-0000-4000-8000-000000000156",
            "bridget_capabilities",
            json!({}).as_object().unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(
            capabilities["guidance"],
            json!({"intents":["queue","steer","interrupt"], "interrupt_cooldown_secs":30, "interrupt_cap_per_10min":3, "queue_tool":true})
        );
        assert_eq!(
            capabilities["queue"],
            json!({"held":false,"actions":["list","cancel","reorder","promote"]})
        );
        assert_eq!(
            capabilities["providers"][0]["queue"]["held"], true,
            "flag réel par type conservé"
        );
        server.join().unwrap();
        std::fs::remove_dir_all(socket.parent().unwrap()).unwrap();
    }

    #[test]
    fn guidance156_mcp_intent_inconnue_refusee() {
        let socket = test_socket("g156-refus");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            for _ in 0..4 {
                reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            }
            listener.set_nonblocking(true).unwrap();
            assert!(
                matches!(listener.accept(), Err(error) if error.kind() == io::ErrorKind::WouldBlock)
            );
        });
        for (intent, reply) in [
            (json!("auto"), false),
            (json!(null), false),
            (json!(3), false),
            (json!("steer"), true),
        ] {
            let arguments =
                json!({"to":"890000", "body":"guidage", "intent":intent, "reply":reply});
            assert!(matches!(
                execute_tool_at(
                    "owner",
                    "bridget_send",
                    arguments.as_object().unwrap(),
                    &socket
                ),
                Err(ToolError::Technical {
                    code: "invalid_intent",
                    ..
                })
            ));
            // Le plafond précède les arguments ; aucun échange métier ni résolution UUID.
        }
        server.join().unwrap();
        std::fs::remove_dir_all(socket.parent().unwrap()).unwrap();
    }

    #[test]
    fn guidance156_mcp_queue_egal_absent() {
        let absent = json!({"to":"89000000-0000-4000-8000-000000000157", "body":"octets $VAR intacts", "id":"guidance156", "issued_at":1_700_000_000});
        let mut queue = absent.clone();
        queue["intent"] = json!("queue");
        let (old_wire, old_result) = guidance156_capture_send(absent).unwrap();
        let (queue_wire, queue_result) = guidance156_capture_send(queue).unwrap();
        assert_eq!(
            queue_wire, old_wire,
            "aucun intent QueueOnly explicite dans le canon historique"
        );
        assert_eq!(
            old_result,
            json!({"status":"accepted", "id":"guidance156", "issued_at":1_700_000_000i64, "hops":4})
        );
        assert_eq!(queue_result["intent"], "queue");
        assert_eq!(queue_result["effect"], "unconfirmed");
        assert!(
            serde_json::from_str::<Value>(&queue_wire).unwrap()["message"]
                .get("intent")
                .is_none()
        );
    }

    #[test]
    fn guidance156_mcp_steer_pose_steer_current() {
        let (wire, result) = guidance156_capture_send(json!({"to":"89000000-0000-4000-8000-000000000157", "body":"guidage", "id":"guidance156", "issued_at":1_700_000_000, "intent":"steer"})).unwrap();
        let WrapperToDaemon::SendIdempotent { message, .. } = decode(&wire).unwrap() else {
            panic!("SendIdempotent")
        };
        assert_eq!(
            message.intent,
            Some(bridget_core::MessageIntent::SteerCurrent)
        );
        assert_eq!(result["intent"], "steer");
        assert_eq!(result["effect"], "unconfirmed");
    }

    #[test]
    fn guidance156_mcp_interrupt_pose_interrupt_and_start() {
        let (wire, result) = guidance156_capture_send(json!({"to":"89000000-0000-4000-8000-000000000157", "body":"nouveau tour", "id":"guidance156", "issued_at":1_700_000_000, "intent":"interrupt"})).unwrap();
        let WrapperToDaemon::SendIdempotent { message, .. } = decode(&wire).unwrap() else {
            panic!("SendIdempotent")
        };
        assert_eq!(
            message.intent,
            Some(bridget_core::MessageIntent::InterruptAndStart)
        );
        assert_eq!(result["intent"], "interrupt");
        assert_eq!(result["effect"], "unconfirmed");
    }

    #[test]
    fn spec251_mcp_information_receipt_et_ancien_daemon_sans_repli() {
        let args=json!({"to":"89000000-0000-4000-8000-000000000157","body":"avis sans action","intent":"information","id":"guidance156","issued_at":1700000000});
        let (wire,result)=guidance156_capture_send(args.clone()).unwrap();
        let WrapperToDaemon::SendIdempotent{message,..}=decode(&wire).unwrap() else {panic!("send")};
        assert_eq!(message.intent,Some(bridget_core::MessageIntent::StoreOnly));
        assert_eq!(result["status"],"stored");assert_eq!(result["effect"],"stored_without_wakeup");assert_eq!(result["model_woken"],false);
        let error=guidance156_capture_send_with_information(args,false).unwrap_err();
        assert!(matches!(error,ToolError::Technical{code:"information_unsupported",..}));
        for invalid in [json!({"reply":true}),json!({"in_reply_to":"mission"}),json!({"reply_timeout":30})] {
            let mut args=json!({"to":"89000000-0000-4000-8000-000000000157","body":"avis","intent":"information"});
            args.as_object_mut().unwrap().extend(invalid.as_object().unwrap().clone());
            assert!(guidance156_capture_send(args).is_err());
        }
    }

    #[test]
    fn spec256_reference_variants_expose_native_required_fields() {
        let catalogue = discovery_tools();
        let tool = catalogue.iter().find(|t| t["name"] == "bridget_task_continue").unwrap();
        assert!(tool["description"].as_str().unwrap().contains("kind et label"));
        let items = &tool["inputSchema"]["properties"]["brief"]["properties"]["references"]["items"];
        assert!(items.get("allOf").is_none());
        let variants = items["oneOf"].as_array().unwrap();
        assert_eq!(variants.len(), 6);
        let cases = [
            json!({"kind":"file","label":"source","host":"cartae","path":"/data/review"}),
            json!({"kind":"url","label":"source","url":"https://example.org/"}),
            json!({"kind":"message","label":"source","id":"m1","target":"a1","source_label":"contexte"}),
            json!({"kind":"journal","label":"source","agent":"10000000-0000-4000-8000-000000000001","from_seq":0,"to_seq":2,"source_label":"contexte"}),
            json!({"kind":"thread","label":"source","thread_id":"10000000-0000-4000-8000-000000000001","from_seq":1,"to_seq":2,"source_label":"contexte"}),
            json!({"kind":"artifact","label":"source","artifact_id":"artefact","version_id":"v1","source_label":"contexte"}),
        ];
        for value in cases {
            let variant = variants.iter().find(|v| v["properties"]["kind"]["enum"][0] == value["kind"]).unwrap();
            assert_eq!(variant["additionalProperties"], false);
            assert_eq!(variant["properties"]["kind"]["type"], "string");
            let draft = json!({"objective":"o","summary":"s","references":[value.clone()]});
            assert!(crate::handoff::validate_and_render(&draft).is_ok(), "{value}");
            let required = variant["required"].as_array().unwrap();
            assert_eq!(required.len(), value.as_object().unwrap().len());
            for key in required {
                let key = key.as_str().unwrap();
                assert!(value.get(key).is_some());
                let mut missing = draft.clone();
                missing["references"][0].as_object_mut().unwrap().remove(key);
                let error = crate::handoff::validate_and_render(&missing).unwrap_err();
                assert_eq!(error.field, format!("references[0].{key}"));
            }
            let mut extra = draft;
            extra["references"][0]["unexpected"] = json!(true);
            assert!(crate::handoff::validate_and_render(&extra).is_err());
        }
    }

    #[test]
    fn spec252_brief_schema_exposes_native_utf8_limits() {
        let catalogue=tools();
        let schema=&catalogue.iter().find(|t|t["name"]=="bridget_handoff").unwrap()["inputSchema"]["properties"]["draft"];
        assert_eq!(schema["x-maxRenderedUtf8Bytes"],crate::handoff::MAX_BODY_BYTES);
        assert_eq!(schema["properties"]["results"]["items"]["properties"]["text"]["x-maxUtf8Bytes"],crate::handoff::MAX_ITEM_BYTES);
        assert_eq!(schema["properties"]["references"]["items"]["x-maxSerializedUtf8Bytes"],crate::handoff::MAX_REFERENCE_BYTES);
        assert_eq!(schema["properties"]["references"]["items"]["oneOf"][0]["properties"]["host"]["x-maxUtf8Bytes"],crate::handoff::MAX_LABEL_BYTES);
        for name in ["bridget_delegate","bridget_auto_delegate","bridget_task_continue"] {
            assert_eq!(&catalogue.iter().find(|t|t["name"]==name).unwrap()["inputSchema"]["properties"]["brief"],schema);
        }
        // Moins de2048caractères peut dépasser2048octets ; le diagnostic reste exact.
        let draft=json!({"objective":"o","summary":"s","results":[{"text":"é".repeat(crate::handoff::MAX_ITEM_BYTES/2+1)}]});
        let error=crate::handoff::validate_and_render(&draft).unwrap_err();
        assert_eq!(error.field,"results[0].text");
        assert!(error.reason.contains("octets"));
    }

    #[test]
    fn native153_mcp_continue_catalogue_et_champs_fermes() {
        let socket = test_socket("invalid-scope158");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            for _ in 0..2 {
                reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            }
        });
        let catalogue = tools();
        let handoff_schema = &catalogue
            .iter()
            .find(|tool| tool["name"] == "bridget_handoff")
            .unwrap()["inputSchema"]["properties"]["draft"];
        for (name, args) in [
            (
                "bridget_delegate",
                json!({"request_id":"r","agent_type":"a","model":"m","task":"t","cwd":"/c","parent":"forged"}),
            ),
            (
                "bridget_task_continue",
                json!({"task_id":"child","request_id":"r","task":"t","parent":"forged"}),
            ),
        ] {
            let tool = catalogue.iter().find(|tool| tool["name"] == name).unwrap();
            let schema = &tool["inputSchema"];
            assert_eq!(&schema["properties"]["brief"], handoff_schema);
            assert_eq!(schema["additionalProperties"], false);
            assert!(
                tool["description"]
                    .as_str()
                    .unwrap()
                    .contains("sources_not_verified")
            );
            assert!(matches!(
                execute_tool_at(
                    "owner",
                    name,
                    args.as_object().unwrap(),
                    &socket
                ),
                Err(ToolError::InvalidParams(message)) if message == "champ inconnu"
            ));
        }
        let continuation = catalogue
            .iter()
            .find(|tool| tool["name"] == "bridget_task_continue")
            .unwrap();
        assert_eq!(
            continuation["inputSchema"]["required"],
            json!(["task_id", "request_id", "task"])
        );
        assert_eq!(
            continuation["annotations"],
            json!({"readOnlyHint":false,"destructiveHint":false,"idempotentHint":true,"openWorldHint":true})
        );
        let delegate = catalogue
            .iter()
            .find(|tool| tool["name"] == "bridget_delegate")
            .unwrap();
        let duration = &delegate["inputSchema"]["properties"]["keep_alive_secs"];
        assert_eq!(duration["type"], "integer");
        assert_eq!(duration["minimum"], 60);
        assert_eq!(duration["maximum"], 7200);
        server.join().unwrap();
    }

    #[test]
    fn spec259_ack_scope_timeout_is_pre_dispatch_without_ack() {
        use std::os::fd::AsRawFd;
        let socket = test_socket("259-scope-timeout");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let (mut reader, writer) = registered196(&listener);
            assert!(matches!(read_command(&mut reader), WrapperToDaemon::NativeDelegation {
                request: bridget_transport::protocol::NativeDelegationRequest::CallerScope {}
            }));
            reader.get_ref().set_read_timeout(Some(Duration::from_secs(15))).unwrap();
            // Le client doit fermer à son échéance, sans jamais envoyer l'ACK.
            assert_eq!(reader.read_line(&mut String::new()).unwrap(), 0);
            drop(writer);
            let mut fd = libc::pollfd { fd: listener.as_raw_fd(), events: libc::POLLIN, revents: 0 };
            // SAFETY: `fd` est une structure pollfd locale valide ; le nombre d'éléments passé vaut 1.
            assert_eq!(unsafe { libc::poll(&mut fd, 1, 100) }, 0, "aucune nouvelle connexion");
        });
        let args = json!({"task_ids":["task259"]});
        let start = std::time::Instant::now();
        let error = execute_tool_at("owner", "bridget_task_ack", args.as_object().unwrap(), &socket).unwrap_err();
        assert!(start.elapsed() >= crate::communication::client::DAEMON_BUDGET);
        assert!(matches!(error, ToolError::Technical { code:"daemon_unreachable", message }
            if message.contains("budget JSONL dépassé")
                && message.contains("lecture préalable du plafond MCP")
                && message.contains("task259")
                && message.contains("n'a pas été transmise")
                && message.contains("tout reçu antérieur")
                && message.contains("cause précise du retard reste inconnue")));
        server.join().unwrap();
    }

    #[test]
    fn spec259_scope_disconnect_guides_read_and_ack_before_dispatch() {
        for (name, args) in [
            ("bridget_task_ack", json!({"task_ids":["task259"]})),
            ("bridget_task_status", json!({"task_id":"task259"})),
        ] {
            let socket = test_socket(name);
            let listener = UnixListener::bind(&socket).unwrap();
            let server = thread::spawn(move || drop_scope196(&listener));
            let error = execute_tool_at("owner", name, args.as_object().unwrap(), &socket).unwrap_err();
            assert!(matches!(error, ToolError::Technical { code:"daemon_unreachable", message }
                if message.contains("lecture préalable du plafond MCP")
                    && message.contains("task259") && message.contains("n'a pas été transmise")));
            server.join().unwrap();
        }
    }

    #[test]
    fn native151_ack_facade_schema_ferme_bornes_et_annotations() {
        let catalogue = tools();
        let ack = catalogue
            .iter()
            .find(|tool| tool["name"] == "bridget_task_ack")
            .expect("outil ACK informatif publié");
        let schema = &ack["inputSchema"];
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["required"], json!(["task_ids"]));
        assert_eq!(
            schema["properties"]
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["task_ids"]
        );
        assert_eq!(schema["properties"]["task_ids"]["type"], "array");
        assert_eq!(schema["properties"]["task_ids"]["minItems"], 1);
        assert_eq!(schema["properties"]["task_ids"]["maxItems"], 16);
        assert_eq!(schema["properties"]["task_ids"]["uniqueItems"], true);
        assert_eq!(
            schema["properties"]["task_ids"]["items"],
            json!({"type":"string","minLength":1})
        );
        assert_eq!(ack["annotations"]["readOnlyHint"], false);
        assert_eq!(ack["annotations"]["destructiveHint"], false);
        assert_eq!(ack["annotations"]["idempotentHint"], true);
        let identity = crate::mcp_identity::ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        for args in [
            json!(null),
            json!({}),
            json!({"task_ids":null}),
            json!({"task_ids":"task"}),
            json!({"task_ids":[]}),
            json!({"task_ids":["task","task"]}),
            json!({"task_ids":(0..17).map(|index|format!("task-{index}")).collect::<Vec<_>>()}),
            json!({"task_ids":[""]}),
            json!({"task_ids":[1]}),
            json!({"task_ids":[null]}),
            json!({"task_ids":["task"],"parent":"forged"}),
            json!({"task_ids":["task"],"wait_timeout":600}),
        ] {
            assert!(
                matches!(
                    crate::delegation_mcp::execute(
                        &identity,
                        "bridget_task_ack",
                        &args,
                        Path::new("/missing-ack151")
                    ),
                    Err(ToolError::InvalidParams(_))
                ),
                "arguments ACK refusés avant dial : {args}"
            );
        }
        for count in [1, 16] {
            let args = json!({"task_ids":(0..count).map(|index|format!("task-{index}")).collect::<Vec<_>>()});
            assert!(
                matches!(crate::delegation_mcp::execute(&identity,"bridget_task_ack",&args,Path::new("/missing-ack151")),
                Err(ToolError::Technical {code:"daemon_unreachable", ..})),
                "borne {count} valide, socket absent donc erreur de transport"
            );
        }
    }

    #[test]
    fn native151_ack_facade_ancien_daemon_sans_trame_nouvelle() {
        use std::os::fd::AsRawFd;
        let socket = test_socket("ack-old");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            let mut pollfd = libc::pollfd {
                fd: listener.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            let mut frames = Vec::<Value>::new();
            // SAFETY: `pollfd` est une structure pollfd locale valide ; le nombre d'éléments passé vaut 1.
            if unsafe { libc::poll(&mut pollfd, 1, 2000) } <= 0 {
                return frames;
            }
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            frames.push(serde_json::from_str(&line).unwrap());
            writeln!(stream,"{}",json!({"type":"DaemonIdentityReport","host":"fixture","db_path":"fixture","instance_id":"fixture"})).unwrap();
            line.clear();
            if reader.read_line(&mut line).unwrap() > 0 {
                frames.push(serde_json::from_str(&line).unwrap());
            }
            // SAFETY: `pollfd` est une structure pollfd locale valide ; le nombre d'éléments passé vaut 1.
            if unsafe { libc::poll(&mut pollfd, 1, 100) } > 0 {
                let (stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                line.clear();
                BufReader::new(stream).read_line(&mut line).unwrap();
                frames.push(serde_json::from_str(&line).unwrap());
            }
            frames
        });
        let start = Instant::now();
        let result = execute_tool_at(
            "owner",
            "bridget_task_ack",
            json!({"task_ids":["task"]}).as_object().unwrap(),
            &socket,
        );
        let elapsed = start.elapsed();
        let frames = server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
        eprintln!(
            "ACK151 ancien : résultat={result:?}, durée={} ms, trames={frames:?}",
            elapsed.as_millis()
        );
        assert!(
            matches!(result,Err(ToolError::Technical {code:"native_delegation_refused",message}) if message=="wait_unsupported")
        );
        assert_eq!(
            frames,
            json!([{"type":"DaemonIdentityRequest"}])
                .as_array()
                .unwrap()
                .clone()
        );
        assert!(elapsed < Duration::from_secs(5));
    }

    #[test]
    fn native151_ack_facade_trame_ordonne_refus_et_quota_nonwait() {
        use std::os::fd::AsRawFd;
        for (ids, result) in [
            (
                vec![
                    "task-z".to_string(),
                    "task-a".to_string(),
                    "task-m".to_string(),
                ],
                json!({"version":1,"acknowledged":["task-z","task-m"],"already":["task-a"]}),
            ),
            (
                vec!["active".to_string()],
                json!({"status":"refused","code":"task_not_ready",
                    "detail":{"tasks":[{"task_id":"active","round":4,"state":"mission_pending"}],
                        "acknowledged":[],"recovery":"Relisez les états de ces tâches."}}),
            ),
            (
                (0..16)
                    .map(|index| format!("task-{index}"))
                    .collect::<Vec<_>>(),
                json!({"version":1,"acknowledged":(0..16).map(|index|format!("task-{index}")).collect::<Vec<_>>(),"already":[]}),
            ),
        ] {
            let socket = test_socket("ack-supported");
            let listener = UnixListener::bind(&socket).unwrap();
            crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
            let returned = result.clone();
            let backend = thread::spawn(move || {
                let mut frames = Vec::<Value>::new();
                reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
                for stage in 0..2 {
                    let mut pollfd = libc::pollfd {
                        fd: listener.as_raw_fd(),
                        events: libc::POLLIN,
                        revents: 0,
                    };
                    // SAFETY: `pollfd` est une structure pollfd locale valide ; le nombre d'éléments passé vaut 1.
                    if unsafe { libc::poll(&mut pollfd, 1, 2000) } <= 0 {
                        return frames;
                    }
                    let (mut stream, _) = listener.accept().unwrap();
                    stream
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    let first: Value = serde_json::from_str(&line).unwrap();
                    // La preuve locale est synthétique ; ne conserver que le type de l'authentification.
                    frames.push(if stage == 0 {
                        first
                    } else {
                        json!({"type":first["type"]})
                    });
                    if stage == 0 {
                        writeln!(stream,"{}",json!({"type":"DaemonIdentityReport","host":"fixture","db_path":"fixture","instance_id":"fixture","features":["wait_v1"]})).unwrap();
                    } else {
                        writeln!(
                            stream,
                            "{}",
                            json!({"type":"Registered","agent_id":"owner","credential":null})
                        )
                        .unwrap();
                        line.clear();
                        reader.read_line(&mut line).unwrap();
                        frames.push(serde_json::from_str(&line).unwrap());
                        writeln!(
                            stream,
                            "{}",
                            json!({"type":"NativeDelegationResult","result":returned})
                        )
                        .unwrap();
                    }
                }
                frames
            });
            let input=[json!({"jsonrpc":"2.0","id":1,"method":"initialize"}),json!({"jsonrpc":"2.0","method":"notifications/initialized"})]
                    .into_iter().chain((2..=5).map(|id|json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":"bridget_task_wait","arguments":{"task_ids":["pending"]}}})))
                    .chain([json!({"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"bridget_task_ack","arguments":{"task_ids":ids}}})])
                    .map(|frame|format!("{frame}\n")).collect::<String>();
            let resolver = || {
                Ok(crate::mcp_identity::ResolvedIdentity {
                    name: "owner".into(),
                    instance_id: "instance".into(),
                    delegated_origin: None,
                })
            };
            let completed = (Mutex::new(false), Condvar::new());
            let execute =
                |identity: &crate::mcp_identity::ResolvedIdentity, name: &str, args: &Value| {
                    if name == "bridget_task_wait" {
                        // Garder les quatre créneaux jusqu'à la réponse ACK, avec une borne en RED.
                        drop(
                            completed
                                .1
                                .wait_timeout_while(
                                    completed.0.lock().unwrap(),
                                    Duration::from_secs(3),
                                    |finished| !*finished,
                                )
                                .unwrap(),
                        );
                        Ok(json!({"pending":["pending"]}))
                    } else {
                        TOOL_CALL_CONTEXT.with(|slot| {
                            let context = slot.borrow().clone().unwrap();
                            let registry = context.registry.lock().unwrap();
                            assert_eq!(
                                registry.calls.values().filter(|call| call.waiting).count(),
                                4
                            );
                            assert!(
                                !registry.calls[&context.id].waiting,
                                "ACK doit rester ordinaire"
                            );
                        });
                        let result = execute_tool_at_with_identity(
                            identity,
                            name,
                            args.as_object().unwrap(),
                            &socket,
                        );
                        *completed.0.lock().unwrap() = true;
                        completed.1.notify_all();
                        result
                    }
                };
            let mut output = Vec::new();
            serve_with(Cursor::new(input), &mut output, &resolver, &execute).unwrap();
            let frames = backend.join().unwrap();
            std::fs::remove_file(socket).unwrap();
            let replies = String::from_utf8(output)
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str::<Value>(line).unwrap())
                .collect::<Vec<_>>();
            let ack = replies
                .iter()
                .find(|reply| reply["id"] == 6)
                .expect("réponse ACK ordinaire après EOF");
            eprintln!("ACK151 quatre attentes en vol : trames={frames:?}, réponse={ack}");
            assert_eq!(
                frames,
                vec![
                    json!({"type":"DaemonIdentityRequest"}),
                    json!({"type":"RegisterAuxiliary"}),
                    json!({"type":"NativeDelegation","request":{"operation":"ack","task_ids":ids}})
                ]
            );
            if result["status"] == "refused" {
                assert_eq!(ack["result"]["code"], "native_delegation_refused");
                assert!(
                    ack["result"]["content"][0]["text"]
                        .as_str()
                        .unwrap()
                        .contains("task_not_ready")
                );
                let text = ack["result"]["content"][0]["text"].as_str().unwrap();
                for expected in ["active", "mission_pending", "round", "4", "Relisez les états"] {
                    assert!(text.contains(expected), "contexte ACK conservé : {text}");
                }
            } else {
                assert_eq!(ack["result"]["structuredContent"], result);
            }
        }
    }

    #[test]
    fn native151_facade_request_wait_schema_et_bornes() {
        let socket = test_socket("invalid-scope158");
        let listener = UnixListener::bind(&socket).unwrap();
        crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
        let server = thread::spawn(move || {
            for _ in 0..9 {
                reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            }
        });
        let wait = tools()
            .into_iter()
            .find(|tool| tool["name"] == "bridget_request_wait")
            .expect("outil d'attente de réponse présent");
        assert_eq!(wait["inputSchema"]["required"], json!(["request_id"]));
        assert_eq!(wait["inputSchema"]["additionalProperties"], false);
        assert_eq!(
            wait["inputSchema"]["properties"]["wait_timeout"]["default"],
            600
        );
        assert_eq!(wait["annotations"]["readOnlyHint"], true);
        let identity = crate::mcp_identity::ResolvedIdentity {
            name: "owner".into(),
            instance_id: "instance".into(),
            delegated_origin: None,
        };
        for value in [json!(-1), json!(0), json!(3601)] {
            let args = json!({"request_id":"r","wait_timeout":value});
            assert!(
                matches!(execute_tool_at_with_identity(&identity, "bridget_request_wait", args.as_object().unwrap(), &socket), Err(ToolError::Technical { code: "native_delegation_refused", message }) if message == "wait_timeout_out_of_range")
            );
        }
        for args in [
            json!({"request_id":"r","sender":"other"}),
            json!({"request_id":"r","wait_timeout":null}),
            json!({"request_id":"r","wait_timeout":1.5}),
            json!({"request_id":"r","wait_timeout":"1"}),
            json!({"request_id":"r","wait_timeout":true}),
            json!({"request_id":""}),
        ] {
            assert!(matches!(
                execute_tool_at_with_identity(
                    &identity,
                    "bridget_request_wait",
                    args.as_object().unwrap(),
                    &socket
                ),
                Err(ToolError::InvalidParams(_))
            ));
        }
        server.join().unwrap();
    }

    #[test]
    fn native151_facade_quatre_attentes_gardent_les_controles() {
        let input = [
            json!({"jsonrpc":"2.0","id":1,"method":"initialize"}),
            json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        ].into_iter().chain((2..=6).map(|id| json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":"bridget_task_wait","arguments":{"task_ids":["t"],"wait_timeout":3600}}})))
            .chain([json!({"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"bridget_task_status","arguments":{"task_id":"t"}}}), json!({"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"bridget_task_cancel","arguments":{"task_id":"t"}}})])
            .map(|value| value.to_string()).collect::<Vec<_>>().join("\n");
        let resolver = || {
            Ok(crate::mcp_identity::ResolvedIdentity {
                name: "owner".into(),
                instance_id: "instance".into(),
                delegated_origin: None,
            })
        };
        let execute = |_: &crate::mcp_identity::ResolvedIdentity, name: &str, _: &Value| {
            if name == "bridget_task_wait" {
                thread::sleep(Duration::from_millis(100));
            }
            Ok(json!({"tool":name}))
        };
        let mut output = Vec::new();
        serve_with(input.as_bytes(), &mut output, &resolver, &execute).unwrap();
        let responses = String::from_utf8(output)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .collect::<Vec<_>>();
        assert!(
            responses.iter().any(
                |response| response["id"] == 6 && response["result"]["code"] == "wait_capacity"
            )
        );
        for id in [7, 8] {
            assert!(responses.iter().any(|response| {
                response["id"] == id
                    && response["result"]["structuredContent"]["tool"]
                        .as_str()
                        .is_some()
            }));
        }
    }

    #[test]
    fn native151_facade_annulation_avant_enregistrement() {
        use std::io::Read;
        let (input, mut writer) = UnixStream::pair().unwrap();
        let (started_tx, started_rx) = mpsc::channel();
        let (control_tx, control_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let release_rx = Mutex::new(release_rx);
        let (refused_tx, refused_rx) = mpsc::channel();
        thread::scope(|scope| {
            let server = scope.spawn(|| {
                let resolver = || {
                    Ok(crate::mcp_identity::ResolvedIdentity {
                        name: "owner".into(),
                        instance_id: "instance".into(),
                        delegated_origin: None,
                    })
                };
                let execute = |_: &crate::mcp_identity::ResolvedIdentity, name: &str, _: &Value| {
                    if name != "bridget_task_wait" {
                        control_tx.send(()).unwrap();
                        return Ok(json!({"status":"ok"}));
                    }
                    started_tx.send(()).unwrap();
                    release_rx.lock().unwrap().recv().unwrap();
                    let (stream, mut peer) = UnixStream::pair().unwrap();
                    peer.set_read_timeout(Some(Duration::from_millis(300)))
                        .unwrap();
                    let refused = check_tool_call(Some(&stream)).is_err();
                    let mut byte = [0];
                    let closed = matches!(peer.read(&mut byte), Ok(0));
                    refused_tx.send((refused, closed)).unwrap();
                    Ok(json!({"version":1}))
                };
                let mut output = Vec::new();
                serve_with(BufReader::new(input), &mut output, &resolver, &execute).unwrap();
                output
            });
            for request in [
                json!({"jsonrpc":"2.0","id":1,"method":"initialize"}),
                json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
                json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"bridget_task_wait","arguments":{"task_ids":["t"]}}}),
            ] {
                write_response(&mut writer, &request).unwrap();
            }
            started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            write_response(
                &mut writer,
                &json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":2}}),
            )
            .unwrap();
            write_response(&mut writer, &json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"bridget_task_status","arguments":{"task_id":"t"}}})).unwrap();
            control_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            release_tx.send(()).unwrap();
            let observed = refused_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            drop(writer);
            let output = String::from_utf8(server.join().unwrap()).unwrap();
            assert_eq!(
                observed,
                (true, true),
                "le socket tardif doit être refusé et fermé"
            );
            assert!(
                !output
                    .lines()
                    .any(|line| serde_json::from_str::<Value>(line).unwrap()["id"] == 2)
            );
        });
    }

    #[test]
    fn native151_facade_annulation_pendant() {
        use std::io::Read;
        let (input, mut writer) = UnixStream::pair().unwrap();
        let (started_tx, started_rx) = mpsc::channel();
        let (closed_tx, closed_rx) = mpsc::channel();
        thread::scope(|scope| {
            let server = scope.spawn(|| {
                let resolver = || {
                    Ok(crate::mcp_identity::ResolvedIdentity {
                        name: "owner".into(),
                        instance_id: "instance".into(),
                        delegated_origin: None,
                    })
                };
                let execute = |_: &crate::mcp_identity::ResolvedIdentity, _: &str, _: &Value| {
                    let (mut stream, peer) = UnixStream::pair().unwrap();
                    stream
                        .set_read_timeout(Some(Duration::from_millis(300)))
                        .unwrap();
                    check_tool_call(Some(&stream))?;
                    started_tx.send(peer).unwrap();
                    let mut byte = [0];
                    closed_tx
                        .send(matches!(stream.read(&mut byte), Ok(0)))
                        .unwrap();
                    Ok(json!({"version":1}))
                };
                let mut output = Vec::new();
                serve_with(BufReader::new(input), &mut output, &resolver, &execute).unwrap();
                output
            });
            for request in [
                json!({"jsonrpc":"2.0","id":1,"method":"initialize"}),
                json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
                json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"bridget_task_wait","arguments":{"task_ids":["t"]}}}),
            ] {
                write_response(&mut writer, &request).unwrap();
            }
            let _peer = started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            write_response(
                &mut writer,
                &json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":2}}),
            )
            .unwrap();
            let closed = closed_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            drop(writer);
            let output = String::from_utf8(server.join().unwrap()).unwrap();
            assert!(closed, "l'annulation doit interrompre l'échange bloquant");
            assert!(
                !output
                    .lines()
                    .any(|line| serde_json::from_str::<Value>(line).unwrap()["id"] == 2)
            );
        });
    }

    #[test]
    fn native151_facade_eof_ferme_les_attentes() {
        use std::io::Read;
        let (input, mut writer) = UnixStream::pair().unwrap();
        let (started_tx, started_rx) = mpsc::channel();
        let (closed_tx, closed_rx) = mpsc::channel();
        let start = Instant::now();
        thread::scope(|scope| {
            let server = scope.spawn(|| {
                let resolver = || {
                    Ok(crate::mcp_identity::ResolvedIdentity {
                        name: "owner".into(),
                        instance_id: "instance".into(),
                        delegated_origin: None,
                    })
                };
                let execute = |_: &crate::mcp_identity::ResolvedIdentity, _: &str, _: &Value| {
                    let (mut stream, peer) = UnixStream::pair().unwrap();
                    stream
                        .set_read_timeout(Some(Duration::from_millis(300)))
                        .unwrap();
                    check_tool_call(Some(&stream))?;
                    started_tx.send(peer).unwrap();
                    let mut byte = [0];
                    closed_tx
                        .send(matches!(stream.read(&mut byte), Ok(0)))
                        .unwrap();
                    Ok(json!({"version":1}))
                };
                let mut output = Vec::new();
                serve_with(BufReader::new(input), &mut output, &resolver, &execute).unwrap();
                output
            });
            for request in [
                json!({"jsonrpc":"2.0","id":1,"method":"initialize"}),
                json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
            ] {
                write_response(&mut writer, &request).unwrap();
            }
            for id in [2, 3] {
                write_response(&mut writer, &json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":"bridget_task_wait","arguments":{"task_ids":["t"],"wait_timeout":3600}}})).unwrap();
            }
            let _peers = [
                started_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
                started_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
            ];
            drop(writer);
            let output = String::from_utf8(server.join().unwrap()).unwrap();
            assert!(start.elapsed() < Duration::from_secs(6));
            assert!(closed_rx.recv_timeout(Duration::from_secs(2)).unwrap());
            assert!(closed_rx.recv_timeout(Duration::from_secs(2)).unwrap());
            assert!(!output.lines().any(|line| {
                [json!(2), json!(3)].contains(&serde_json::from_str::<Value>(line).unwrap()["id"])
            }));
        });
    }

    #[test]
    fn native151_facade_eof_attente_et_statut_bloque() {
        let socket = test_socket("wait-status-eof");
        let listener = UnixListener::bind(&socket).unwrap();
        crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
        let (ready_tx, ready_rx) = mpsc::channel();
        let daemon = thread::spawn(move || {
            {
                reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
                let (stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(15)))
                    .unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut writer = BufWriter::new(stream);
                assert!(matches!(
                    read_command(&mut reader),
                    WrapperToDaemon::DaemonIdentityRequest
                ));
                write_command(
                    &mut writer,
                    DaemonToWrapper::DaemonIdentityReport {
                        host: "fixture".into(),
                        db_path: "fixture".into(),
                        instance_id: "fixture".into(),
                        features: vec!["wait_v1".into()],
                    },
                );
            }
            let mut blocked = Vec::new();
            for name in ["bridget_task_wait", "bridget_task_status"] {
                if name == "bridget_task_status" {
                    reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
                }
                let (stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(15)))
                    .unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut writer = BufWriter::new(stream);
                assert!(matches!(read_command(&mut reader),
                    WrapperToDaemon::RegisterAuxiliary { agent_id, instance_id, .. }
                        if agent_id == "owner" && instance_id == "instance"));
                write_command(
                    &mut writer,
                    DaemonToWrapper::Registered {
                        agent_id: "owner".into(),
                        credential: None,
                    },
                );
                match (name, read_command(&mut reader)) {
                    (
                        "bridget_task_wait",
                        WrapperToDaemon::NativeDelegation {
                            request:
                                bridget_transport::protocol::NativeDelegationRequest::Wait {
                                    task_ids,
                                    wait_timeout,
                                },
                        },
                    ) => {
                        assert_eq!(task_ids, ["task"]);
                        assert_eq!(wait_timeout, Some(3600));
                    }
                    (
                        "bridget_task_status",
                        WrapperToDaemon::NativeDelegation {
                            request:
                                bridget_transport::protocol::NativeDelegationRequest::Status { task_id },
                        },
                    ) => assert_eq!(task_id, "task"),
                    other => panic!("appel natif attendu avant EOF : {other:?}"),
                }
                blocked.push((name, reader));
                ready_tx.send(name).unwrap();
            }
            blocked
                .into_iter()
                .map(|(name, mut reader)| {
                    let mut line = String::new();
                    (name, reader.read_line(&mut line).unwrap() == 0)
                })
                .collect::<Vec<_>>()
        });
        let (input, mut input_writer) = UnixStream::pair().unwrap();
        let (elapsed, output) = thread::scope(|scope| {
            let server = scope.spawn(|| {
                let resolver = || {
                    Ok(crate::mcp_identity::ResolvedIdentity {
                        name: "owner".into(),
                        instance_id: "instance".into(),
                        delegated_origin: None,
                    })
                };
                let execute = |identity: &crate::mcp_identity::ResolvedIdentity,
                               name: &str,
                               arguments: &Value| {
                    execute_tool_at_with_identity(
                        identity,
                        name,
                        arguments.as_object().unwrap(),
                        &socket,
                    )
                };
                let mut output = Vec::new();
                serve_with(BufReader::new(input), &mut output, &resolver, &execute).unwrap();
                output
            });
            write_response(
                &mut input_writer,
                &json!({"jsonrpc":"2.0","id":1,"method":"initialize"}),
            )
            .unwrap();
            write_response(
                &mut input_writer,
                &json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
            )
            .unwrap();
            write_response(&mut input_writer, &json!({"jsonrpc":"2.0","id":2,"method":"tools/call",
                "params":{"name":"bridget_task_wait","arguments":{"task_ids":["task"],"wait_timeout":3600}}})).unwrap();
            assert_eq!(
                ready_rx.recv_timeout(Duration::from_secs(3)).unwrap(),
                "bridget_task_wait"
            );
            write_response(
                &mut input_writer,
                &json!({"jsonrpc":"2.0","id":3,"method":"tools/call",
                "params":{"name":"bridget_task_status","arguments":{"task_id":"task"}}}),
            )
            .unwrap();
            assert_eq!(
                ready_rx.recv_timeout(Duration::from_secs(3)).unwrap(),
                "bridget_task_status"
            );
            let eof_at = Instant::now();
            drop(input_writer);
            let output = server.join().unwrap();
            (eof_at.elapsed(), String::from_utf8(output).unwrap())
        });
        let closed = daemon.join().unwrap();
        std::fs::remove_file(socket).unwrap();
        eprintln!(
            "MCP151 EOF mixte : Wait3600 et Status démarrés/authentifiés, budgets hérités ; façade terminée en {} ms ; fermetures backend={closed:?}",
            elapsed.as_millis()
        );
        assert!(closed.iter().all(|(_, closed)| *closed));
        assert!(
            elapsed <= Duration::from_secs(6),
            "FR038 : EOF mixte -> fin en {elapsed:?}, limite 6 s"
        );
        assert!(!output.lines().any(|line| {
            [json!(2), json!(3)].contains(&serde_json::from_str::<Value>(line).unwrap()["id"])
        }));
    }

    #[test]
    fn native151_facade_eof_preserve_nonwait_rapide() {
        let socket = test_socket("nonwait-fast-eof");
        let listener = UnixListener::bind(&socket).unwrap();
        crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
        let (ready_tx, ready_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let backend = thread::spawn(move || {
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            let (stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(
                matches!(read_command(&mut reader), WrapperToDaemon::RegisterAuxiliary { agent_id, instance_id, .. }
                if agent_id == "owner" && instance_id == "instance")
            );
            write_command(
                &mut writer,
                DaemonToWrapper::Registered {
                    agent_id: "owner".into(),
                    credential: None,
                },
            );
            assert!(
                matches!(read_command(&mut reader), WrapperToDaemon::NativeDelegation {
                request: bridget_transport::protocol::NativeDelegationRequest::Status { task_id }
            } if task_id == "task")
            );
            ready_tx.send(()).unwrap();
            release_rx.recv_timeout(Duration::from_secs(3)).unwrap();
            // Latence synthétique du backend après EOF, loin du budget ordinaire de 10 s.
            thread::sleep(Duration::from_millis(100));
            let response = encode(&DaemonToWrapper::NativeDelegationResult {
                result: json!({"task_id":"task","status":"working"}),
            })
            .unwrap();
            writeln!(writer, "{response}")
                .and_then(|_| writer.flush())
                .is_ok()
        });
        let (input, mut input_writer) = UnixStream::pair().unwrap();
        let (elapsed, output) = thread::scope(|scope| {
            let server = scope.spawn(|| {
                let resolver = || {
                    Ok(crate::mcp_identity::ResolvedIdentity {
                        name: "owner".into(),
                        instance_id: "instance".into(),
                        delegated_origin: None,
                    })
                };
                let execute = |identity: &crate::mcp_identity::ResolvedIdentity,
                               name: &str,
                               arguments: &Value| {
                    execute_tool_at_with_identity(
                        identity,
                        name,
                        arguments.as_object().unwrap(),
                        &socket,
                    )
                };
                let mut output = Vec::new();
                serve_with(BufReader::new(input), &mut output, &resolver, &execute).unwrap();
                output
            });
            for request in [
                json!({"jsonrpc":"2.0","id":1,"method":"initialize"}),
                json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
                json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"bridget_task_status","arguments":{"task_id":"task"}}}),
            ] {
                write_response(&mut input_writer, &request).unwrap();
            }
            ready_rx.recv_timeout(Duration::from_secs(3)).unwrap();
            let eof_at = Instant::now();
            drop(input_writer);
            release_tx.send(()).unwrap();
            let output = server.join().unwrap();
            (eof_at.elapsed(), String::from_utf8(output).unwrap())
        });
        let response_sent = backend.join().unwrap();
        std::fs::remove_file(socket).unwrap();
        eprintln!(
            "FR034 EOF nonwait : Status authentifié/reçu avant EOF, latence backend100 ms, réponse transmise={response_sent}, sortie={} ms, stdout brut={output:?}",
            elapsed.as_millis()
        );
        let responses = output
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .collect::<Vec<_>>();
        let response = responses.iter().find(|response| response["id"] == 2);
        assert!(
            response_sent && response.is_some(),
            "FR034 : réponse rapide perdue après EOF : {responses:?}"
        );
        assert_eq!(
            response.unwrap()["result"]["structuredContent"]["status"],
            "working"
        );
        assert!(elapsed < Duration::from_secs(2));
    }

    #[test]
    fn spec241_cancelled_registration_never_opens_a_recovery_connection() {
        let socket=test_socket("241-cancel");let listener=UnixListener::bind(&socket).unwrap();
        let owner="24100000-0000-4000-8000-000000000241";
        crate::mcp_identity::mock_private_identity(&socket,owner,"i241");
        let registry=Arc::new(Mutex::new(ToolCallRegistry::default()));
        registry.lock().unwrap().reserve("c241".into(),true).unwrap();
        let mut call=ToolCallGuard::new("c241".into(),registry.clone());
        let remote=registry.clone();let (done_tx,done_rx)=mpsc::channel();
        let server=thread::spawn(move || {
            let (stream,_)=listener.accept().unwrap();let mut reader=BufReader::new(stream);
            let mut line=String::new();reader.read_line(&mut line).unwrap();
            remote.lock().unwrap().cancel("c241"); drop(reader);
            done_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            listener.set_nonblocking(true).unwrap();
            assert!(matches!(listener.accept(),Err(e) if e.kind()==io::ErrorKind::WouldBlock));
        });
        let result=crate::communication::client::registered_connection_until(owner,"i241",&socket,
            std::time::Instant::now()+Duration::from_secs(1));
        assert!(matches!(result,Err(crate::communication::client::ClientError::Technical{code:"wait_cancelled",..})));
        done_tx.send(()).unwrap();server.join().unwrap();call.finish();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn native151_facade_enregistrement_apres_eof_refuse() {
        use std::io::Read;
        for waiting in [true, false] {
            let registry = Arc::new(Mutex::new(ToolCallRegistry::default()));
            registry
                .lock()
                .unwrap()
                .reserve("late".into(), waiting)
                .unwrap();
            let mut call = ToolCallGuard::new("late".into(), Arc::clone(&registry));
            drop(CancelCallsOnExit(Arc::clone(&registry)));
            let (stream, mut peer) = UnixStream::pair().unwrap();
            peer.set_read_timeout(Some(Duration::from_millis(300)))
                .unwrap();
            assert!(check_tool_call(None).is_err());
            assert!(check_tool_call(Some(&stream)).is_err());
            assert_eq!(peer.read(&mut [0]).unwrap(), 0);
            assert!(call.finish());
            assert!(TOOL_CALL_CONTEXT.with(|context| context.borrow().is_none()));
            assert!(registry.lock().unwrap().calls.is_empty());
        }
    }

    #[test]
    fn native151_facade_notification_nonwait_preserve_connexion_auth_et_operation() {
        for phase in ["connect", "auth", "operation"] {
            let socket = test_socket("nonwait-cancel");
            let listener = UnixListener::bind(&socket).unwrap();
            crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
            let (ready_tx, ready_rx) = mpsc::channel();
            let (release_tx, release_rx) = mpsc::channel();
            let release_rx = Arc::new(Mutex::new(release_rx));
            let (control_tx, control_rx) = mpsc::channel();
            let (done_tx, done_rx) = mpsc::channel();
            let backend_ready = ready_tx.clone();
            let backend_release = Arc::clone(&release_rx);
            let backend = thread::spawn(move || {
                reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
                let (stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut writer = BufWriter::new(stream);
                assert!(matches!(
                    read_command(&mut reader),
                    WrapperToDaemon::RegisterAuxiliary { .. }
                ));
                if phase == "auth" {
                    backend_ready.send(()).unwrap();
                    backend_release
                        .lock()
                        .unwrap()
                        .recv_timeout(Duration::from_secs(3))
                        .unwrap();
                }
                write_command(
                    &mut writer,
                    DaemonToWrapper::Registered {
                        agent_id: "owner".into(),
                        credential: None,
                    },
                );
                assert!(
                    matches!(read_command(&mut reader), WrapperToDaemon::NativeDelegation {
                    request: bridget_transport::protocol::NativeDelegationRequest::Status { task_id }
                } if task_id == "task")
                );
                if phase == "operation" {
                    backend_ready.send(()).unwrap();
                    backend_release
                        .lock()
                        .unwrap()
                        .recv_timeout(Duration::from_secs(3))
                        .unwrap();
                }
                write_command(
                    &mut writer,
                    DaemonToWrapper::NativeDelegationResult {
                        result: json!({"status":"working"}),
                    },
                );
            });
            let (input, mut input_writer) = UnixStream::pair().unwrap();
            let output = thread::scope(|scope| {
                let server = scope.spawn(|| {
                    let resolver = || {
                        Ok(crate::mcp_identity::ResolvedIdentity {
                            name: "owner".into(),
                            instance_id: "instance".into(),
                            delegated_origin: None,
                        })
                    };
                    let execute = |identity: &crate::mcp_identity::ResolvedIdentity,
                                   name: &str,
                                   arguments: &Value| {
                        if name == "bridget_who" {
                            control_tx.send(()).unwrap();
                            return Ok(json!({"control":true}));
                        }
                        if phase == "connect" {
                            ready_tx.send(()).unwrap();
                            release_rx
                                .lock()
                                .unwrap()
                                .recv_timeout(Duration::from_secs(3))
                                .unwrap();
                        }
                        let result = execute_tool_at_with_identity(
                            identity,
                            name,
                            arguments.as_object().unwrap(),
                            &socket,
                        );
                        done_tx.send(result.is_ok()).unwrap();
                        result
                    };
                    let mut output = Vec::new();
                    serve_with(BufReader::new(input), &mut output, &resolver, &execute).unwrap();
                    output
                });
                for request in [
                    json!({"jsonrpc":"2.0","id":1,"method":"initialize"}),
                    json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
                    json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"bridget_task_status","arguments":{"task_id":"task"}}}),
                ] {
                    write_response(&mut input_writer, &request).unwrap();
                }
                ready_rx.recv_timeout(Duration::from_secs(3)).unwrap();
                write_response(&mut input_writer, &json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":2}})).unwrap();
                write_response(&mut input_writer, &json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"bridget_who","arguments":{}}})).unwrap();
                control_rx.recv_timeout(Duration::from_secs(3)).unwrap();
                release_tx.send(()).unwrap();
                assert!(
                    done_rx.recv_timeout(Duration::from_secs(3)).unwrap(),
                    "notification nonwait doit préserver {phase}"
                );
                drop(input_writer);
                server.join().unwrap()
            });
            backend.join().unwrap();
            std::fs::remove_file(socket).unwrap();
            let output = String::from_utf8(output).unwrap();
            assert!(
                !output
                    .lines()
                    .any(|line| serde_json::from_str::<Value>(line).unwrap()["id"] == 2)
            );
            eprintln!(
                "MCP151 notification nonwait pendant {phase}: opération réussie, réponse masquée"
            );
        }
    }

    #[test]
    fn native151_facade_annulation_apres() {
        use std::io::Read;
        let (input, mut input_writer) = UnixStream::pair().unwrap();
        let (output, output_reader) = UnixStream::pair().unwrap();
        output_reader
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut output_reader = BufReader::new(output_reader);
        let (peer_tx, peer_rx) = mpsc::channel();
        thread::scope(|scope| {
            let server = scope.spawn(|| {
                let resolver = || {
                    Ok(crate::mcp_identity::ResolvedIdentity {
                        name: "owner".into(),
                        instance_id: "instance".into(),
                        delegated_origin: None,
                    })
                };
                let execute = |_: &crate::mcp_identity::ResolvedIdentity, _: &str, _: &Value| {
                    let (stream, peer) = UnixStream::pair().unwrap();
                    peer.set_read_timeout(Some(Duration::from_millis(300)))
                        .unwrap();
                    check_tool_call(Some(&stream))?;
                    peer_tx.send(peer).unwrap();
                    Ok(json!({"state":"result_available"}))
                };
                serve_with(BufReader::new(input), output, &resolver, &execute).unwrap();
            });
            for request in [
                json!({"jsonrpc":"2.0","id":1,"method":"initialize"}),
                json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
                json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"bridget_task_wait","arguments":{"task_ids":["t"]}}}),
            ] {
                write_response(&mut input_writer, &request).unwrap();
            }
            let mut line = String::new();
            output_reader.read_line(&mut line).unwrap();
            line.clear();
            output_reader.read_line(&mut line).unwrap();
            let completed: Value = serde_json::from_str(&line).unwrap();
            assert_eq!(completed["id"], 2);
            assert_eq!(
                completed["result"]["structuredContent"]["state"],
                "result_available"
            );
            let mut peer = peer_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            assert_eq!(
                peer.read(&mut [0]).unwrap(),
                0,
                "le socket d'un appel fini n'est plus retenu"
            );
            write_response(
                &mut input_writer,
                &json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":2}}),
            )
            .unwrap();
            write_response(
                &mut input_writer,
                &json!({"jsonrpc":"2.0","id":3,"method":"ping"}),
            )
            .unwrap();
            line.clear();
            output_reader.read_line(&mut line).unwrap();
            assert_eq!(serde_json::from_str::<Value>(&line).unwrap()["id"], 3);
            drop(input_writer);
            server.join().unwrap();
        });
    }

    #[test]
    fn native151_facade_attentes_transmettent_et_rendent_les_contrats() {
        for name in ["bridget_task_wait", "bridget_request_wait"] {
            let socket = test_socket("wait-contract");
            let listener = UnixListener::bind(&socket).unwrap();
            crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
            let server = thread::spawn(move || {
                {
                    reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
                    let (stream, _) = listener.accept().unwrap();
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut writer = BufWriter::new(stream);
                    assert!(matches!(
                        read_command(&mut reader),
                        WrapperToDaemon::DaemonIdentityRequest
                    ));
                    write_command(
                        &mut writer,
                        DaemonToWrapper::DaemonIdentityReport {
                            host: "fixture".into(),
                            db_path: "fixture".into(),
                            instance_id: "fixture".into(),
                            features: vec!["wait_v1".into()],
                        },
                    );
                }
                let (stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut writer = BufWriter::new(stream);
                assert!(
                    matches!(read_command(&mut reader), WrapperToDaemon::RegisterAuxiliary { agent_id, instance_id, .. } if agent_id == "owner" && instance_id == "instance")
                );
                write_command(
                    &mut writer,
                    DaemonToWrapper::Registered {
                        agent_id: "owner".into(),
                        credential: None,
                    },
                );
                match (name, read_command(&mut reader)) {
                    (
                        "bridget_task_wait",
                        WrapperToDaemon::NativeDelegation {
                            request:
                                bridget_transport::protocol::NativeDelegationRequest::Wait {
                                    task_ids,
                                    wait_timeout,
                                },
                        },
                    ) => {
                        assert_eq!(task_ids, ["task"]);
                        assert_eq!(wait_timeout, Some(600));
                        write_command(
                            &mut writer,
                            DaemonToWrapper::NativeDelegationResult {
                                result: json!({"version":1,"timed_out":false,"waited_secs":0,"ready":[{"task_id":"task","work_state":"result_available","summary":"résultat"}],"pending":[]}),
                            },
                        );
                    }
                    (
                        "bridget_request_wait",
                        WrapperToDaemon::RequestWait {
                            request_id,
                            wait_timeout,
                        },
                    ) => {
                        assert_eq!(request_id, "request");
                        assert_eq!(wait_timeout, 600);
                        write_command(
                            &mut writer,
                            DaemonToWrapper::RequestWaitResult {
                                state: "answered".into(),
                                timed_out_wait: false,
                                waited_secs: 0,
                                reply: Some(
                                    json!({"id":"reply","from":"child","summary":"réponse","summary_truncated":false}),
                                ),
                                reply_reason: None,
                            },
                        );
                    }
                    other => panic!("trame d'attente attendue : {other:?}"),
                }
            });
            let identity = crate::mcp_identity::ResolvedIdentity {
                name: "owner".into(),
                instance_id: "instance".into(),
                delegated_origin: None,
            };
            let args = match name {
                "bridget_task_wait" => json!({"task_ids":["task"]}),
                _ => json!({"request_id":"request"}),
            };
            let result =
                execute_tool_at_with_identity(&identity, name, args.as_object().unwrap(), &socket)
                    .unwrap();
            server.join().unwrap();
            std::fs::remove_file(socket).unwrap();
            assert_eq!(result["version"], 1);
            match name {
                "bridget_task_wait" => assert_eq!(result["ready"][0]["summary"], "résultat"),
                _ => {
                    assert_eq!(result["state"], "answered");
                    assert_eq!(result["reply"]["summary"], "réponse");
                    assert_eq!(result["reply_reason"], Value::Null);
                }
            }
        }
    }

    #[test]
    fn native151_facade_annulation_sonde_et_authentification_reelles() {
        for during_probe in [true, false] {
            let socket = test_socket("wait-cancel-connection");
            let listener = UnixListener::bind(&socket).unwrap();
            crate::mcp_identity::mock_private_identity(&socket, "owner", "instance");
            let (ready_tx, ready_rx) = mpsc::channel();
            let (closed_tx, closed_rx) = mpsc::channel();
            let daemon = thread::spawn(move || {
                reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
                let (stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                assert!(matches!(
                    read_command(&mut reader),
                    WrapperToDaemon::DaemonIdentityRequest
                ));
                let mut reader = if during_probe {
                    reader
                } else {
                    let mut writer = BufWriter::new(stream);
                    write_command(
                        &mut writer,
                        DaemonToWrapper::DaemonIdentityReport {
                            host: "fixture".into(),
                            db_path: "fixture".into(),
                            instance_id: "fixture".into(),
                            features: vec!["wait_v1".into()],
                        },
                    );
                    let (stream, _) = listener.accept().unwrap();
                    stream
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    let mut reader = BufReader::new(stream);
                    assert!(matches!(
                        read_command(&mut reader),
                        WrapperToDaemon::RegisterAuxiliary { .. }
                    ));
                    reader
                };
                ready_tx.send(()).unwrap();
                let mut line = String::new();
                closed_tx
                    .send(reader.read_line(&mut line).unwrap() == 0)
                    .unwrap();
            });
            let (input, mut writer) = UnixStream::pair().unwrap();
            thread::scope(|scope| {
                let server = scope.spawn(|| {
                    let resolver = || {
                        Ok(crate::mcp_identity::ResolvedIdentity {
                            name: "owner".into(),
                            instance_id: "instance".into(),
                            delegated_origin: None,
                        })
                    };
                    let execute = |identity: &crate::mcp_identity::ResolvedIdentity,
                                   name: &str,
                                   arguments: &Value| {
                        execute_tool_at_with_identity(
                            identity,
                            name,
                            arguments.as_object().unwrap(),
                            &socket,
                        )
                    };
                    let mut output = Vec::new();
                    serve_with(BufReader::new(input), &mut output, &resolver, &execute).unwrap();
                    output
                });
                for request in [
                    json!({"jsonrpc":"2.0","id":1,"method":"initialize"}),
                    json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
                    json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"bridget_task_wait","arguments":{"task_ids":["t"],"wait_timeout":3600}}}),
                ] {
                    write_response(&mut writer, &request).unwrap();
                }
                ready_rx.recv_timeout(Duration::from_secs(2)).unwrap();
                write_response(&mut writer, &json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":2}})).unwrap();
                let closed = closed_rx.recv_timeout(Duration::from_secs(2)).unwrap();
                drop(writer);
                let output = String::from_utf8(server.join().unwrap()).unwrap();
                assert!(closed);
                assert!(
                    !output
                        .lines()
                        .any(|line| serde_json::from_str::<Value>(line).unwrap()["id"] == 2)
                );
            });
            daemon.join().unwrap();
            std::fs::remove_file(socket).unwrap();
        }
    }

    struct EmptyProcessTree;

    impl crate::mcp_identity::ProcessTree for EmptyProcessTree {
        fn birth(&self, _pid: u32) -> Option<u64> {
            None
        }

        fn parent(&self, _pid: u32) -> Option<u32> {
            None
        }
    }

    fn run(lines: &[Value]) -> Vec<Value> {
        let input = lines
            .iter()
            .map(|line| {
                line.as_str()
                    .map_or_else(|| line.to_string(), str::to_string)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let mut stdout = Vec::new();
        let identity = || {
            Ok(crate::mcp_identity::ResolvedIdentity {
                name: "fixture-agent".to_string(),
                instance_id: "fixture-instance".to_string(),
                delegated_origin: None,
            })
        };
        let execute = |_: &crate::mcp_identity::ResolvedIdentity, _: &str, _: &Value| {
            Err(ToolError::Technical {
                code: "daemon_unreachable",
                message: "daemon de fixture absent".to_string(),
            })
        };
        serve_with(input.as_bytes(), &mut stdout, &identity, &execute).unwrap();
        String::from_utf8(stdout)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    #[test]
    fn matrice_fr009_couvre_les_quinze_cas() {
        let cases: Vec<Value> = FIXTURES
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(cases.len(), 15, "la matrice FR-009 doit rester complète");

        for case in cases {
            let responses = run(case["stdin"].as_array().unwrap());
            let expected_count = case["responses"].as_u64().unwrap() as usize;
            assert_eq!(responses.len(), expected_count, "{}", case["name"]);
            match case["expect"].as_str().unwrap() {
                "initialize" => {
                    assert_eq!(responses[0]["id"], case["id"], "{}", case["name"]);
                    assert_eq!(responses[0]["result"]["protocolVersion"], PROTOCOL_VERSION);
                    assert_eq!(
                        responses[0]["result"]["capabilities"],
                        json!({ "tools": {} })
                    );
                }
                "tools" => assert_eq!(
                    responses.last().unwrap()["result"]["tools"]
                        .as_array()
                        .unwrap()
                        .len(),
                    37
                ),
                "tools_twice" => assert_eq!(responses[1]["result"], responses[2]["result"]),
                "ping" => assert_eq!(responses.last().unwrap()["result"], json!({})),
                "tool_unavailable" => {
                    assert_eq!(responses.last().unwrap()["result"]["isError"], true)
                }
                "error" => assert!(responses.last().unwrap().get("error").is_some()),
                "none" => assert!(responses.is_empty()),
                "mixed_ids" => {
                    assert_eq!(responses[1]["id"], json!(9));
                    assert_eq!(responses[2]["id"], json!("neuf"));
                }
                other => panic!("expectation inconnue: {other}"),
            }
        }
    }

    #[test]
    fn spec102_v33_bridget_thread_transmet_le_contrat_et_refuse_les_champs_inconnus() {
        use bridget_transport::protocol::{IdentityCredential, ThreadAction, ThreadResult};
        use sha2::{Digest, Sha256};
        let root = std::env::temp_dir().join(format!("mcp102-{}", uuid::Uuid::new_v4().simple()));
        bridget_transport::fsutil::create_private_dir(&root).unwrap();
        bridget_transport::fsutil::create_private_dir(&root.join("agent-names")).unwrap();
        let socket = root.join("s");
        let proof = root
            .join("agent-names")
            .join(format!("proof-{:x}.json", Sha256::digest(b"instance102")));
        bridget_transport::fsutil::write_private_file_atomic(&proof,&serde_json::to_vec(&json!({"agent_id":"alice","instance_id":"instance102","credential":IdentityCredential::new("fixture-only".into())})).unwrap()).unwrap();
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = std::io::BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            std::io::BufRead::read_line(&mut reader, &mut line).unwrap();
            assert!(matches!(
                bridget_transport::protocol::decode::<WrapperToDaemon>(line.trim()).unwrap(),
                WrapperToDaemon::RegisterAuxiliary { .. }
            ));
            std::io::Write::write_all(
                &mut stream,
                format!(
                    "{}\n",
                    bridget_transport::protocol::encode(&DaemonToWrapper::Registered {
                        agent_id: "alice".into(),
                        credential: None
                    })
                    .unwrap()
                )
                .as_bytes(),
            )
            .unwrap();
            line.clear();
            std::io::BufRead::read_line(&mut reader, &mut line).unwrap();
            let WrapperToDaemon::ThreadRequest { request } =
                bridget_transport::protocol::decode::<WrapperToDaemon>(line.trim()).unwrap()
            else {
                panic!("requête de fil attendue")
            };
            assert_eq!(request.version, 1);
            assert!(
                matches!(request.request, ThreadAction::Post { ref thread_id, kind:Some(bridget_transport::protocol::ThreadEntryKind::Action), supersedes_seq:Some(1), .. } if thread_id == "33333333-3333-4333-8333-333333333333")
            );
            std::io::Write::write_all(
                &mut stream,
                format!(
                    "{}\n",
                    bridget_transport::protocol::encode(&DaemonToWrapper::ThreadResult {
                        result: ThreadResult { version: 1, result: json!({"status":"error","code":"thread_unavailable","detail":"Fil indisponible pour cette identité.","retryable":false}) }
                    })
                    .unwrap()
                )
                .as_bytes(),
            )
            .unwrap();
        });
        let result = execute_tool_at_with_scope(
            "alice",
            "instance102",
            "bridget_thread",
            json!({"action":"post","thread_id":"33333333-3333-4333-8333-333333333333","kind":"action","supersedes_seq":1,"body":"b","notify":[],"operation_id":"bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb"})
                .as_object()
                .unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(result["code"], "thread_unavailable");
        let rendered = tool_result(result);
        assert_eq!(
            rendered["isError"], true,
            "une erreur métier n'est jamais un succès"
        );
        assert_eq!(rendered["structuredContent"]["code"], "thread_unavailable");
        server.join().unwrap();
        std::fs::remove_file(&socket).unwrap();
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            for _ in 0..3 {
                reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            }
        });
        for invalid in [
            json!({"action":"post","thread_id":"t","body":"b","notify":[],"operation_id":"o","actor":"x"}),
            json!({"action":"summary","thread_id":"t"}),
            json!({"action":"post","thread_id":"t","body":"b","operation_id":"o"}),
        ] {
            assert!(matches!(
                execute_tool_at_with_scope(
                    "alice",
                    "instance102",
                    "bridget_thread",
                    invalid.as_object().unwrap(),
                    &socket
                ),
                Err(ToolError::InvalidParams(_))
            ));
        }
        assert_eq!(
            tool_result(json!({"status":"accepted"}))["isError"],
            Value::Null
        );
        server.join().unwrap();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn spec147_uuid_mcp_facade_ipc_and_business_replay_case() {
        use bridget_transport::protocol::IdentityCredential;
        use sha2::{Digest, Sha256};
        const A: &str = "14700000-aaaa-4aaa-8aaa-00000000000a";
        const B: &str = "14700000-bbbb-4bbb-8bbb-00000000000b";
        const CREATE: &str = "14700000-cccc-4ccc-8ccc-00000000000c";
        const POST: &str = "14700000-dddd-4ddd-8ddd-00000000000d";
        let root = std::env::temp_dir().join(format!("mcp147-{}", uuid::Uuid::new_v4().simple()));
        bridget_transport::fsutil::create_private_dir(&root).unwrap();
        bridget_transport::fsutil::create_private_dir(&root.join("agent-names")).unwrap();
        let socket = root.join("s");
        let proof = root
            .join("agent-names")
            .join(format!("proof-{:x}.json", Sha256::digest(b"instance147")));
        bridget_transport::fsutil::write_private_file_atomic(&proof, &serde_json::to_vec(&json!({"agent_id":A,"instance_id":"instance147","credential":IdentityCredential::new("fixture-only".into())})).unwrap()).unwrap();
        let store = crate::store::Store::open(&root.join("db")).unwrap();
        let listener = UnixListener::bind(&socket).unwrap();
        // Réutilise la frontière auxiliaire de v33, pas handle_connection ni
        // ses attestations. Façade, client IPC et backend SQLite sont réels.
        let server = thread::spawn(move || {
            let mut changed = Vec::new();
            let mut writes = Vec::new();
            for _ in 0..4 {
                reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(6)))
                    .unwrap();
                let mut reader = std::io::BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                std::io::BufRead::read_line(&mut reader, &mut line).unwrap();
                assert!(matches!(
                    bridget_transport::protocol::decode::<WrapperToDaemon>(line.trim()).unwrap(),
                    WrapperToDaemon::RegisterAuxiliary { .. }
                ));
                std::io::Write::write_all(
                    &mut stream,
                    format!(
                        "{}\n",
                        bridget_transport::protocol::encode(&DaemonToWrapper::Registered {
                            agent_id: A.into(),
                            credential: None
                        })
                        .unwrap()
                    )
                    .as_bytes(),
                )
                .unwrap();
                line.clear();
                std::io::BufRead::read_line(&mut reader, &mut line).unwrap();
                let WrapperToDaemon::ThreadRequest { request } =
                    bridget_transport::protocol::decode::<WrapperToDaemon>(line.trim()).unwrap()
                else {
                    panic!("ThreadRequest attendu")
                };
                let (result, marker) =
                    crate::threads::handle_with_change(&store, &FixtureDirectory, A, request, 1);
                changed.push(marker.is_some());
                writes.push(store.connection().total_changes());
                std::io::Write::write_all(
                    &mut stream,
                    format!(
                        "{}\n",
                        bridget_transport::protocol::encode(&DaemonToWrapper::ThreadResult {
                            result
                        })
                        .unwrap()
                    )
                    .as_bytes(),
                )
                .unwrap();
            }
            let bodies = store
                .connection()
                .prepare("SELECT body FROM discussion_entries ORDER BY seq")
                .unwrap()
                .query_map([], |row| row.get::<_, String>(0))
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            (changed, writes, bodies)
        });
        let call = |arguments: Value| {
            execute_dynamic_tool_at_with_scope(
                A,
                "instance147",
                "bridget_thread",
                &arguments,
                &socket,
            )
            .unwrap()
        };
        let created = call(
            json!({"action":"create","title":"Titre É exact","members":[B.to_uppercase()],"operation_id":CREATE.to_uppercase()}),
        );
        let replay = call(
            json!({"action":"create","title":"Titre É exact","members":[B],"operation_id":CREATE}),
        );
        let id = created["thread_id"]
            .as_str()
            .unwrap_or("14700000-eeee-4eee-8eee-00000000000e");
        let body = "É 🦀\r\n  Exact\n\n";
        let posted = call(
            json!({"action":"post","thread_id":id.to_uppercase(),"body":body,"notify":[B.to_uppercase()],"operation_id":POST.to_uppercase()}),
        );
        let post_replay = call(
            json!({"action":"post","thread_id":id,"body":body,"notify":[B],"operation_id":POST}),
        );
        let (changed, writes, bodies) = server.join().unwrap();
        assert_eq!(created["status"], "created", "{created}");
        assert_eq!(replay, created);
        assert_eq!(posted["status"], "posted", "{posted}");
        assert_eq!(post_replay, posted);
        assert_eq!(changed, [true, false, true, false]);
        assert_eq!(writes[0], writes[1]);
        assert_eq!(writes[2], writes[3]);
        assert_eq!(bodies, [body]);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn spec147_thread_catalogue_explains_ack_then_same_read() {
        let tool = tools()
            .into_iter()
            .find(|tool| tool["name"] == "bridget_thread")
            .unwrap();
        let description = tool["description"].as_str().unwrap();
        assert!(description.contains("même read sur le même thread_id"));
        assert!(description.contains("history, qui ne fait pas avancer le repère"));
        for bound in ["from_seq", "to_seq"] {
            assert!(
                tool["inputSchema"]["properties"][bound]["description"]
                    .as_str()
                    .unwrap()
                    .contains("history uniquement")
            );
        }
    }

    #[test]
    fn catalogue_mcp_expose_exactement_les_quatre_verbes_du_greffe() {
        let names = tools()
            .into_iter()
            .map(|tool| tool["name"].as_str().unwrap().to_string())
            .collect::<BTreeSet<_>>();
        let expected = [
            "bridget_capabilities",
            "bridget_refresh_models",
            "bridget_read_commit_file",
            "bridget_ask_user",
            "bridget_task_repair_workspace",
            "bridget_delegate",
            "bridget_auto_delegate",
            "bridget_task_continue",
            "bridget_task_respond",
            "bridget_task_status",
            "bridget_task_wait",
            "bridget_task_ack",
            "bridget_request_wait",
            "bridget_task_cancel",
            "bridget_cancel",
            "bridget_queue",
            "bridget_control_status",
            "bridget_dnd",
            "bridget_domain",
            "bridget_events",
            "bridget_handoff",
            "bridget_journal",
            "bridget_ledger",
            "bridget_publish_artifact",
            "bridget_read_artifact",
            "bridget_rename",
            "bridget_runtime",
            "bridget_send",
            "bridget_specialists",
            "bridget_status",
            "bridget_thread",
            "bridget_wake",
            "bridget_who",
            "guichet_delegate",
            "guichet_objective_close",
            "guichet_registre_add",
            "guichet_request_status",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
        assert_eq!(names, expected);
        for forbidden in ["profile_approve", "routine_approve", "command"] {
            assert!(
                names.iter().all(|name| !name.contains(forbidden)),
                "l'action humaine {forbidden} ne doit jamais être un outil MCP"
            );
        }
    }

    #[test]
    fn spec094_catalogue_expose_les_six_outils_sans_cible_libre() {
        let catalogue = tools();
        let expected = [
            "bridget_rename",
            "bridget_dnd",
            "bridget_domain",
            "bridget_runtime",
            "bridget_status",
            "bridget_control_status",
        ];
        for name in expected {
            let tool = catalogue
                .iter()
                .find(|tool| tool["name"] == name)
                .unwrap_or_else(|| panic!("outil 094 absent du catalogue : {name}"));
            let schema = &tool["inputSchema"];
            assert_eq!(
                schema["additionalProperties"], false,
                "schéma ouvert : {name}"
            );
            let properties = schema["properties"]
                .as_object()
                .expect("propriétés de schéma présentes");
            for forbidden in [
                "from",
                "agent",
                "agent_id",
                "instance_id",
                "source",
                "socket",
                "path",
            ] {
                assert!(
                    !properties.contains_key(forbidden),
                    "{name} expose une autorité libre : {forbidden}"
                );
            }
        }
    }

    #[test]
    fn spec094_parametres_invalides_sont_refuses_avant_connexion() {
        let socket = test_socket("spec094-invalid");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            for _ in 0..16 {
                reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            }
        });
        let cases = [
            (
                "bridget_rename",
                json!({"display_name":"Agent", "agent_id":"autre"}),
            ),
            ("bridget_dnd", json!({"mode":"on", "duration":"0s"})),
            ("bridget_dnd", json!({"mode":"on", "duration":"169h"})),
            (
                "bridget_dnd",
                json!({"mode":"on", "duration":"18446744073709551615h"}),
            ),
            ("bridget_dnd", json!({"mode":"off", "duration":"1m"})),
            ("bridget_domain", json!({"domain":"a", "reset":true})),
            ("bridget_domain", json!({"reset":false})),
            ("bridget_domain", json!({"domain":"avec espace"})),
            ("bridget_domain", json!({"domain":"avec/slash"})),
            ("bridget_domain", json!({"domain":"équipe"})),
            ("bridget_runtime", json!({"model":"m", "source":"observed"})),
            ("bridget_runtime", json!({"model":"   "})),
            ("bridget_runtime", json!({"model":"m", "effort":"   "})),
            ("bridget_status", json!({"socket":"/tmp/interdit"})),
            ("bridget_control_status", json!({"history_limit":51})),
            ("bridget_control_status", json!({"paused":true})),
        ];
        for (name, arguments) in cases {
            assert!(
                matches!(
                    execute_tool_at(
                        "89000000-0000-4000-8000-000000000194",
                        name,
                        arguments.as_object().unwrap(),
                        &socket,
                    ),
                    Err(ToolError::InvalidParams(_))
                ),
                "{name} a dépassé sa validation locale : {arguments}"
            );
        }
        server.join().unwrap();
    }

    #[test]
    fn spec094_inventaire_documente_toutes_les_commandes_du_repartiteur() {
        const CLI: &str = include_str!("cli.rs");
        const INVENTORY: &str = include_str!("../../../skills/bridget/references/commandes.md");

        let dispatch = CLI
            .split("// --- Sous-commandes daemon / client ---")
            .nth(1)
            .expect("marqueur du répartiteur CLI")
            .split("        _ => {")
            .next()
            .expect("bras fermés du répartiteur");
        let mut commands = BTreeSet::new();
        for line in dispatch.lines().filter(|line| line.starts_with("        \"") && line.contains("=>")) {
            let left = line.split("=>").next().unwrap();
            for quoted in left.split('"').skip(1).step_by(2) {
                commands.insert(quoted);
            }
        }
        commands.insert("--");
        for alias in ["codex", "claude", "gemini", "gclaude"] {
            commands.insert(alias);
        }

        let documented = INVENTORY
            .lines()
            .filter_map(|line| line.strip_prefix("| `"))
            .filter_map(|line| line.split('`').next())
            .collect::<BTreeSet<_>>();
        let missing = commands
            .difference(&documented)
            .copied()
            .collect::<Vec<_>>();
        assert!(
            missing.is_empty(),
            "commandes sans décision 094 : {missing:?}"
        );
        for retired in [
            "ui",
            "cleanup",
            "managed-runtime-wrapper",
            "managed-runtime-stop",
            "project-runtime",
            "project-round",
        ] {
            assert!(
                INVENTORY.contains(&format!("`{retired}`")),
                "commande retirée non expliquée : {retired}"
            );
        }
    }

    #[test]
    fn publication_html_est_annoncee_comme_un_contenu_inerte() {
        let publication = tools()
            .into_iter()
            .find(|tool| tool["name"] == "bridget_publish_artifact")
            .expect("outil de publication présent");
        let kinds = publication["inputSchema"]["properties"]["kind"]["enum"]
            .as_array()
            .expect("énumération des types présente");
        assert!(kinds.iter().any(|kind| kind == "html"));
        let description = publication["description"]
            .as_str()
            .expect("description présente");
        assert!(description.contains("kind=html"));
        assert!(description.contains("ne jamais recopier le HTML dans Markdown"));
        assert!(description.contains("contenu inerte"));
        assert!(!description.contains("rend inline"));
    }

    #[test]
    fn publication_d_artefact_transporte_un_contrat_canonique_et_un_recu_atteste() {
        let socket = test_socket("artifact-publish");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            let (stream, _) = listener.accept().unwrap();
            let read_stream = stream.try_clone().unwrap();
            let mut reader = BufReader::new(read_stream);
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RegisterAuxiliary { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::Registered {
                    credential: None,
                    agent_id: "agent-fixture".to_string(),
                },
            );
            let WrapperToDaemon::ArtifactPublish {
                contract_version,
                canonical_publication,
            } = read_command(&mut reader)
            else {
                panic!("publication d'artefact attendue");
            };
            assert_eq!(contract_version, ARTIFACT_CONTRACT_VERSION);
            let publication: ArtifactPublicationV1 =
                serde_json::from_slice(&canonical_publication).unwrap();
            assert_eq!(publication.canonical_bytes(), canonical_publication);
            let receipt = ArtifactReceiptV1 {
                artifact_ref: "artifact:fixture".to_string(),
                version_ref: "artifact-version:fixture".to_string(),
                state: crate::artifact_types::ArtifactState::Published,
                content_digest: publication.content_digest(),
                warnings: Vec::new(),
                conversation_reference: "conversation:project:agent".to_string(),
                storage_state: crate::artifact_types::ArtifactStorageState::Canonical,
            };
            write_command(
                &mut writer,
                DaemonToWrapper::ArtifactPublicationResult {
                    contract_version: ARTIFACT_CONTRACT_VERSION,
                    replayed: false,
                    receipt_json: Some(serde_json::to_vec(&receipt).unwrap()),
                    refusal_code: None,
                    refusal_message: None,
                },
            );
        });
        let arguments = serde_json::json!({
            "idempotency_key": "fixture-artifact-v1",
            "kind": "kpi",
            "title": "Latence médiane",
            "payload": {"value": 17.5, "unit": "ms"},
            "sources": [{
                "source_kind": "agent_computed",
                "locator": "calculation:p50",
                "citation": "Calcul attesté",
                "content_digest": "f6c9a81b91f3329113d5b0ab57c2ca9cbbc96cd6b6f4ea8434fb829432533a7a",
                "transformations": ["médiane p50"]
            }],
            "publication_reason": "initial"
        });
        let result = execute_tool_at(
            "agent-fixture",
            "bridget_publish_artifact",
            arguments.as_object().unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(result["status"], "published");
        assert_eq!(result["receipt"]["artifact_ref"], "artifact:fixture");
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn spec174_publication_coupure_apres_ecriture_reste_issue_inconnue() {
        for disconnect in [true, false] {
            let socket = test_socket("artifact-cut");
            let listener = UnixListener::bind(&socket).unwrap();
            crate::mcp_identity::mock_private_identity(&socket, "agent-fixture", "test-instance");
            let server = thread::spawn(move || {
                reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
                let (stream, _) = listener.accept().unwrap();
                let read_stream = stream.try_clone().unwrap();
                let mut reader = BufReader::new(read_stream);
                let mut writer = BufWriter::new(stream);
                assert!(matches!(
                    read_command(&mut reader),
                    WrapperToDaemon::RegisterAuxiliary { .. }
                ));
                write_command(
                    &mut writer,
                    DaemonToWrapper::Registered {
                        credential: None,
                        agent_id: "agent-fixture".to_string(),
                    },
                );
                assert!(matches!(
                    read_command(&mut reader),
                    WrapperToDaemon::ArtifactPublish { .. }
                ));
                if disconnect {
                    write_command(&mut writer, DaemonToWrapper::Disconnect);
                }
            });
            let arguments = serde_json::json!({
                "idempotency_key": "fixture-artifact-cut",
                "kind": "kpi",
                "title": "Latence médiane",
                "payload": {"value": 17.5, "unit": "ms"},
                "sources": [{
                    "source_kind": "agent_computed",
                    "locator": "calculation:p50",
                    "citation": "Calcul attesté",
                    "content_digest": "f6c9a81b91f3329113d5b0ab57c2ca9cbbc96cd6b6f4ea8434fb829432533a7a",
                    "transformations": ["médiane p50"]
                }],
                "publication_reason": "initial"
            });
            let result = execute_dynamic_tool_at_with_scope(
                "agent-fixture",
                "test-instance",
                "bridget_publish_artifact",
                &arguments,
                &socket,
            );
            server.join().unwrap();
            std::fs::remove_file(socket).unwrap();
            let message = result.unwrap_err();
            assert!(message.starts_with("outcome_unknown:"), "{message}");
            assert!(!message.contains("refusée"), "{message}");
            assert!(message.contains("idempotence"), "{message}");
            if disconnect {
                assert!(message.contains("ne prouve ni son échec"), "{message}");
            }
        }
    }

    #[test]
    fn publication_d_artefact_refuse_les_champs_non_contractuels_avant_connexion() {
        let socket = test_socket("artifact-invalid");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            for _ in 0..1 {
                reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            }
        });
        let arguments = serde_json::json!({
            "idempotency_key": "fixture-artifact-v1",
            "kind": "kpi",
            "title": "Latence médiane",
            "payload": {},
            "sources": [],
            "publication_reason": "initial",
            "project_id": "projet-impose-par-l-appelant"
        });
        assert!(matches!(
            execute_tool_at(
                "agent-fixture",
                "bridget_publish_artifact",
                arguments.as_object().unwrap(),
                &socket,
            ),
            Err(ToolError::InvalidParams(_))
        ));
        server.join().unwrap();
    }

    #[test]
    fn lecture_de_contenu_refuse_l_elargissement_de_portee_avant_connexion() {
        let socket = test_socket("invalid-scope158");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            for _ in 0..7 {
                reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            }
        });
        let base = json!({"version":1,"artifact_ref":"artifact:1","version_ref":"artifact-version:1","kind":{"kind":"manifest"},"offset":0,"limit":1024});
        for field in ["project_id", "agent_id", "path", "source"] {
            let mut invalid = base.clone();
            invalid[field] = json!("autre");
            assert!(matches!(
                execute_tool_at(
                    "89000000-0000-4000-8000-000000000001",
                    "bridget_read_artifact",
                    invalid.as_object().unwrap(),
                    &socket
                ),
                Err(ToolError::InvalidParams(_))
            ));
        }
        for limit in [0, 16385] {
            let mut invalid = base.clone();
            invalid["limit"] = json!(limit);
            assert!(matches!(
                execute_tool_at(
                    "89000000-0000-4000-8000-000000000001",
                    "bridget_read_artifact",
                    invalid.as_object().unwrap(),
                    &socket
                ),
                Err(ToolError::InvalidParams(_))
            ));
        }
        let mut unknown_kind = base;
        unknown_kind["kind"] = json!({"kind":"execute"});
        assert!(matches!(
            execute_tool_at(
                "89000000-0000-4000-8000-000000000001",
                "bridget_read_artifact",
                unknown_kind.as_object().unwrap(),
                &socket
            ),
            Err(ToolError::InvalidParams(_))
        ));
        server.join().unwrap();
    }

    #[test]
    fn stdout_ne_contient_que_des_reponses_json_rpc() {
        let fixture: Value = serde_json::from_str(FIXTURES.lines().next().unwrap()).unwrap();
        let mut input = fixture["stdin"]
            .as_array()
            .unwrap()
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        input.push('\n');
        input.push_str("{ JSON invalide\n");
        let mut stdout = Vec::new();
        serve(input.as_bytes(), &mut stdout).unwrap();
        for line in String::from_utf8(stdout).unwrap().lines() {
            let response: Value = serde_json::from_str(line).expect("stdout doit être JSON");
            assert_eq!(response["jsonrpc"], "2.0");
            assert!(response.get("result").is_some() || response.get("error").is_some());
        }
    }

    #[test]
    fn tools_call_resout_l_identite_a_chaque_appel() {
        let request = json!({
            "jsonrpc": "2.0", "id": 3, "method": "tools/call",
            "params": { "name": "bridget_who", "arguments": {} }
        });
        let mut session = Session {
            initialize_seen: true,
            initialized: true,
        };
        let response = dispatch_with_identity(&request, &mut session, &|| {
            Err(crate::mcp_identity::IdentityError::LegacyMarker)
        })
        .unwrap();
        assert_eq!(response["result"]["code"], "legacy_marker");
        assert_eq!(response["result"]["isError"], true);
    }

    #[test]
    fn spec174_identite_absente_guide_montage_t3_sans_execution() {
        let calls = Cell::new(0);
        let execute = |_: &crate::mcp_identity::ResolvedIdentity, _: &str, _: &Value| {
            calls.set(calls.get() + 1);
            Ok(json!({ "status": "accepted" }))
        };
        for name in ["bridget_status", "bridget_auto_delegate"] {
            let request = json!({
                "jsonrpc": "2.0", "id": 174, "method": "tools/call",
                "params": { "name": name, "arguments": {} }
            });
            let mut session = Session {
                initialize_seen: true,
                initialized: true,
            };
            let response = dispatch_with_executor(
                &request,
                &mut session,
                &|| Err(crate::mcp_identity::IdentityError::IdentityNotFound),
                &execute,
            )
            .unwrap();
            assert_eq!(response["result"]["code"], "identity_not_found");
            assert_eq!(response["result"]["isError"], true);
            let text = response["result"]["content"][0]["text"].as_str().unwrap();
            for action in [
                "t3_code.bridget_session",
                "mcp_servers.bridget.command",
                ".local/bin/bridget",
                "preuve",
                "sous-agent interne",
            ] {
                assert!(text.contains(action), "conseil absent : {action}");
            }
        }
        assert_eq!(calls.get(), 0, "aucun outil ni admission sans identité");
    }

    #[test]
    fn identite_mcp_unicode_est_refusee_avant_toute_execution() {
        let root = test_socket("identity-unicode").with_extension("identity");
        std::fs::create_dir_all(&root).unwrap();
        let name_file = root.join("agent-name");
        std::fs::write(&name_file, "分析\n").unwrap();
        let marker_directory = root.join("agent-pids");
        let resolver = || {
            crate::mcp_identity::resolve_identity_with(
                Some(&name_file),
                &marker_directory,
                Some("fixture-instance"),
                42,
                &EmptyProcessTree,
            )
        };
        let calls = Cell::new(0);
        let execute = |_: &crate::mcp_identity::ResolvedIdentity, _: &str, _: &Value| {
            calls.set(calls.get() + 1);
            Ok(json!({ "agents": [] }))
        };
        let request = json!({
            "jsonrpc": "2.0", "id": 39, "method": "tools/call",
            "params": { "name": "bridget_who", "arguments": {} }
        });
        let mut session = Session {
            initialize_seen: true,
            initialized: true,
        };

        let response = dispatch_with_executor(&request, &mut session, &resolver, &execute).unwrap();
        let execution_count = calls.get();
        std::fs::remove_dir_all(root).unwrap();

        assert_eq!(execution_count, 0, "le refus doit précéder tout outil");
        assert_eq!(response["result"]["code"], "identity_not_found");
        assert_eq!(response["result"]["isError"], true);
    }

    #[test]
    fn identite_mcp_ascii_declenche_exactement_une_execution() {
        let root = test_socket("identity-ascii").with_extension("identity");
        std::fs::create_dir_all(&root).unwrap();
        let name_file = root.join("agent-name");
        std::fs::write(&name_file, "a3d27a89-80d5-4e0f-9b84-cf5523ecb026\n").unwrap();
        let marker_directory = root.join("agent-pids");
        let resolver = || {
            crate::mcp_identity::resolve_identity_with(
                Some(&name_file),
                &marker_directory,
                Some("fixture-instance"),
                42,
                &EmptyProcessTree,
            )
        };
        let calls = Cell::new(0);
        let execute =
            |identity: &crate::mcp_identity::ResolvedIdentity, name: &str, arguments: &Value| {
                calls.set(calls.get() + 1);
                assert_eq!(identity.name, "a3d27a89-80d5-4e0f-9b84-cf5523ecb026");
                assert_eq!(identity.instance_id, "fixture-instance");
                assert_eq!(name, "bridget_who");
                assert_eq!(arguments, &json!({}));
                Ok(json!({ "agents": [] }))
            };
        let request = json!({
            "jsonrpc": "2.0", "id": 40, "method": "tools/call",
            "params": { "name": "bridget_who", "arguments": {} }
        });
        let mut session = Session {
            initialize_seen: true,
            initialized: true,
        };

        let response = dispatch_with_executor(&request, &mut session, &resolver, &execute).unwrap();
        let execution_count = calls.get();
        std::fs::remove_dir_all(root).unwrap();

        assert_eq!(
            response["result"]["structuredContent"],
            json!({ "agents": [] })
        );
        assert_eq!(execution_count, 1, "le contrôle sain doit exécuter l'outil");
    }

    #[test]
    fn deux_appels_outil_ne_partagent_pas_l_identite_resolue() {
        let mut session = Session {
            initialize_seen: true,
            initialized: true,
        };
        let calls = Cell::new(0);
        let resolver = || {
            calls.set(calls.get() + 1);
            Ok(crate::mcp_identity::ResolvedIdentity {
                name: "agent".to_string(),
                instance_id: "instance".to_string(),
                delegated_origin: None,
            })
        };
        for id in [3, 4] {
            let request = json!({
                "jsonrpc": "2.0", "id": id, "method": "tools/call",
                "params": { "name": "bridget_who", "arguments": {} }
            });
            let response = dispatch_with_identity(&request, &mut session, &resolver).unwrap();
            assert_eq!(response["id"], id);
        }
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn spec133_identite_deleguee_limite_les_outils_avant_execution() {
        let delegated = || {
            Ok(crate::mcp_identity::ResolvedIdentity {
                name: "parent".into(),
                instance_id: "instance-parent".into(),
                delegated_origin: Some(bridget_core::DelegatedOrigin {
                    provider: "codex".into(),
                    child_ref: "0123456789abcdef".into(),
                }),
            })
        };
        let calls = Cell::new(0);
        let execute = |_: &crate::mcp_identity::ResolvedIdentity, _: &str, _: &Value| {
            calls.set(calls.get() + 1);
            Ok(json!({"agents": []}))
        };
        let mut session = Session {
            initialize_seen: true,
            initialized: true,
        };
        let mut allowed_calls = 0;
        for (index, tool) in tools().into_iter().enumerate() {
            let name = tool["name"].as_str().unwrap();
            let request = json!({
                "jsonrpc":"2.0", "id":index + 1, "method":"tools/call",
                "params":{"name":name,"arguments":{}}
            });
            let before = calls.get();
            let response =
                dispatch_with_executor(&request, &mut session, &delegated, &execute).unwrap();
            if matches!(name, "bridget_send" | "bridget_who") {
                allowed_calls += 1;
                assert_eq!(
                    response["result"]["structuredContent"],
                    json!({"agents": []}),
                    "{name} doit atteindre le handler"
                );
                assert_eq!(calls.get(), before + 1);
            } else {
                assert_eq!(
                    response["result"]["code"], "delegated_tool_forbidden",
                    "{name} doit être refusé"
                );
                assert_eq!(calls.get(), before, "{name} ne doit produire aucun effet");
            }
        }
        assert_eq!(allowed_calls, 2, "la liste blanche reste exactement bornée");
    }

    pub(super) fn test_socket(label: &str) -> PathBuf {
        crate::mcp_identity::mock_socket(label)
    }

    #[test]
    fn spec100_mcp_shares_exact_excerpt_with_reply_and_closed_events_contract() {
        use bridget_transport::protocol::{ObservationKind, ObservationRequest};
        let socket = test_socket("spec100-mcp");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            let (stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Attach
                }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Attach,
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::Subscribe {
                    window: bridget_transport::protocol::AttachWindow::Tail(50),
                    ..
                }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::Subscribed {
                    subscription_id: "s".into(),
                },
            );
            write_command(
                &mut writer,
                DaemonToWrapper::JournalFragment {
                    subscription_id: "s".into(),
                    seq: 7,
                    offset: 0,
                    final_fragment: true,
                    bytes: serde_json::to_vec(&json!({"seq":7,"text":"preuve 🦀"})).unwrap(),
                },
            );
            write_command(
                &mut writer,
                DaemonToWrapper::SnapshotCaughtUp {
                    subscription_id: "s".into(),
                    through_seq: Some(7),
                },
            );
            let (stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Client
                }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client,
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ClientHello { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: crate::build_info::BUILD_ID.into(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: vec![ClientCapability::SendIdempotent],
                },
            );
            let WrapperToDaemon::SendIdempotent {
                message,
                message_id,
                ..
            } = read_command(&mut reader)
            else {
                panic!("envoi existant requis")
            };
            assert!(message.reply);
            assert!(message.body.starts_with("Extrait du journal Bridget"));
            let quoted: Value =
                serde_json::from_str(message.body.split_once('\n').unwrap().1).unwrap();
            assert_eq!(quoted["entries"][0]["text"], "preuve 🦀");
            assert_eq!(quoted["first_seq"], 7);
            write_command(
                &mut writer,
                DaemonToWrapper::IdempotencyResult {
                    operation_kind: "send".into(),
                    idempotency_key: message_id,
                    project_warnings: Vec::new(),
                    issue: IdempotencyIssue::Accepted {
                        expires_at: 9999999999,
                    },
                },
            );
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            let (stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            let WrapperToDaemon::RegisterAuxiliary { agent_id, .. } = read_command(&mut reader)
            else {
                panic!("identité requise")
            };
            write_command(
                &mut writer,
                DaemonToWrapper::Registered {
                    agent_id,
                    credential: None,
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ObservationRequest {
                    request: ObservationRequest::Sub {
                        event: ObservationKind::TurnEnded,
                        once: true,
                        ..
                    }
                }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::ObservationResult {
                    result: json!({"status":"subscribed"}),
                },
            );
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
        });
        let result = execute_tool_at_with_scope(
            "alice",
            "instance100",
            "bridget_journal",
            json!({"agent":"source","to":"bob","reply":true})
                .as_object()
                .unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(result["send"]["status"], "accepted");
        let result = execute_tool_at_with_scope(
            "alice",
            "instance100",
            "bridget_events",
            json!({"action":"sub","event":"turn_ended","once":true})
                .as_object()
                .unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(result["status"], "subscribed");
        assert!(matches!(
            execute_tool_at_with_scope(
                "alice",
                "instance100",
                "bridget_events",
                json!({"action":"list","owner":"victim"})
                    .as_object()
                    .unwrap(),
                &socket
            ),
            Err(ToolError::InvalidParams(_))
        ));
        server.join().unwrap();
    }

    // Fixture privée pour les tests de projection, sans exception d'admission.
    fn execute_tool_at_with_scope(
        identity: &str,
        instance: &str,
        name: &str,
        arguments: &serde_json::Map<String, Value>,
        socket: &Path,
    ) -> Result<Value, ToolError> {
        crate::mcp_identity::mock_private_identity(socket, identity, instance);
        super::execute_tool_at_with_scope(identity, instance, name, arguments, socket)
    }

    struct FixtureDirectory;
    impl crate::threads::Directory for FixtureDirectory {
        fn known_agent(&self, id: &str) -> bool {
            [
                "14700000-aaaa-4aaa-8aaa-00000000000a",
                "14700000-bbbb-4bbb-8bbb-00000000000b",
            ]
            .contains(&id)
        }
        fn member_facts(&self, _: &str) -> crate::threads::MemberFacts {
            crate::threads::MemberFacts::default()
        }
        fn communication_scope(
            &self,
            _: &str,
            _: &[String],
            _: Option<&str>,
        ) -> Result<Vec<bridget_transport::protocol::ProjectWarning>, Value> {
            Ok(vec![])
        }
    }

    // Fixture IPC partagée : inscription credential+instance réelle puis réponse fermée.
    // Plus de vingt consommateurs standard et les oracles de plafond la réemploient.
    pub(super) fn reply_caller_scope(listener: &UnixListener, result: Value) {
        use std::os::fd::AsRawFd;
        let mut pollfd = libc::pollfd {
            fd: listener.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        assert!(
            // SAFETY: `pollfd` est une structure pollfd locale valide ; le nombre d'éléments passé vaut 1.
            unsafe { libc::poll(&mut pollfd, 1, 3000) } > 0,
            "requête de plafond attendue dans le budget de fixture"
        );
        let (stream, _) = listener.accept().unwrap();
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = BufWriter::new(stream);
        let WrapperToDaemon::RegisterAuxiliary {
            agent_id,
            instance_id,
            credential,
        } = read_command(&mut reader)
        else {
            panic!("inscription auxiliaire attendue")
        };
        let expected = crate::mcp_identity::auxiliary_registration(
            &agent_id,
            &instance_id,
            writer
                .get_ref()
                .local_addr()
                .unwrap()
                .as_pathname()
                .unwrap(),
        )
        .unwrap();
        assert!(
            matches!(expected, WrapperToDaemon::RegisterAuxiliary { credential: expected, .. } if expected == credential)
        );
        write_command(
            &mut writer,
            DaemonToWrapper::Registered {
                agent_id,
                credential: None,
            },
        );
        assert!(matches!(
            read_command(&mut reader),
            WrapperToDaemon::NativeDelegation {
                request: bridget_transport::protocol::NativeDelegationRequest::CallerScope {}
            }
        ));
        write_command(
            &mut writer,
            DaemonToWrapper::NativeDelegationResult { result },
        );
    }

    fn registered196(listener: &UnixListener) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
        use std::os::fd::AsRawFd;
        let mut fd = libc::pollfd {
            fd: listener.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        assert!(
            // SAFETY: `fd` est une structure pollfd locale valide ; le nombre d'éléments passé vaut 1.
            unsafe { libc::poll(&mut fd, 1, 3000) } > 0,
            "connexion attendue"
        );
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = BufWriter::new(stream);
        let WrapperToDaemon::RegisterAuxiliary {
            agent_id,
            instance_id,
            credential,
        } = read_command(&mut reader)
        else {
            panic!("inscription auxiliaire")
        };
        let expected = crate::mcp_identity::auxiliary_registration(
            &agent_id,
            &instance_id,
            writer
                .get_ref()
                .local_addr()
                .unwrap()
                .as_pathname()
                .unwrap(),
        )
        .unwrap();
        assert!(
            matches!(expected,WrapperToDaemon::RegisterAuxiliary {credential:expected,..} if expected==credential)
        );
        write_command(
            &mut writer,
            DaemonToWrapper::Registered {
                agent_id,
                credential: None,
            },
        );
        (reader, writer)
    }

    fn drop_scope196(listener: &UnixListener) {
        let (mut reader, _writer) = registered196(listener);
        assert!(matches!(
            read_command(&mut reader),
            WrapperToDaemon::NativeDelegation {
                request: bridget_transport::protocol::NativeDelegationRequest::CallerScope {}
            }
        ));
    }

    fn reply_wait196(listener: &UnixListener, seconds: u64, result: Option<Value>) {
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = BufWriter::new(stream);
        assert!(matches!(
            read_command(&mut reader),
            WrapperToDaemon::DaemonIdentityRequest
        ));
        let response:DaemonToWrapper=serde_json::from_value(json!({"type":"DaemonIdentityReport",
                "host":"fixture196","db_path":"fixture196","instance_id":"fixture196","features":["wait_v1"]})).unwrap();
        write_command(&mut writer, response);
        drop(reader);
        drop(writer);
        let (mut reader, mut writer) = registered196(listener);
        let WrapperToDaemon::NativeDelegation {
            request:
                bridget_transport::protocol::NativeDelegationRequest::Wait {
                    task_ids,
                    wait_timeout,
                },
        } = read_command(&mut reader)
        else {
            panic!("aucune mutation ni remise admise")
        };
        assert_eq!(task_ids, vec!["task-a", "task-b"]);
        assert_eq!(wait_timeout, Some(seconds));
        if let Some(result) = result {
            write_command(
                &mut writer,
                DaemonToWrapper::NativeDelegationResult { result },
            );
        }
    }

    fn wait_recovery196(fail_in_wait: bool) {
        let socket = test_socket("196-read-recovery");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            if fail_in_wait {
                reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
                reply_wait196(&listener, 45, None);
            } else {
                drop_scope196(&listener);
            }
            // Le nouveau plafond peut avoir changé : aucun résultat mémorisé.
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":true}));
            reply_wait196(
                &listener,
                1,
                Some(
                    json!({"version":1,"ready":[{"task_id":"task-a","status":"result_available"}],
                    "pending":["task-b"],"pending_requests":[],"timed_out":false,"waited_secs":0}),
                ),
            );
        });
        let result = execute_tool_at(
            "fixture196",
            "bridget_task_wait",
            json!({"task_ids":["task-a","task-b"],"wait_timeout":45})
                .as_object()
                .unwrap(),
            &socket,
        )
        .expect("une relecture native doit récupérer le constat");
        assert_eq!(result["ready"][0]["task_id"], "task-a");
        assert_eq!(result["pending"], json!(["task-b"]));
        assert_eq!(result["read_recovery"]["recheck_wait_secs"], 1);
        server.join().unwrap();
        std::fs::remove_dir_all(socket.parent().unwrap()).unwrap();
    }

    #[test]
    fn spec196_wait_recupere_coupure_plafond_sans_mutation() {
        wait_recovery196(false);
    }
    #[test]
    fn spec196_wait_recupere_coupure_reponse_sur_memes_taches() {
        wait_recovery196(true);
    }
    #[test]
    fn spec196_wait_recontrole_plafond_et_conserve_refus() {
        let socket = test_socket("196-recheck-refused");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            drop_scope196(&listener);
            reply_caller_scope(
                &listener,
                json!({"status":"refused","code":"task_unavailable"}),
            );
            listener.set_nonblocking(true).unwrap();
            assert!(matches!(listener.accept(),Err(e) if e.kind()==io::ErrorKind::WouldBlock));
        });
        let result = execute_tool_at(
            "fixture196",
            "bridget_task_wait",
            json!({"task_ids":["task-a","task-b"],"wait_timeout":45})
                .as_object()
                .unwrap(),
            &socket,
        );
        assert!(
            matches!(result,Err(ToolError::Technical {code:"native_delegation_refused",message}) if message=="task_unavailable")
        );
        server.join().unwrap();
        std::fs::remove_dir_all(socket.parent().unwrap()).unwrap();
    }

    #[test]
    fn spec196_deux_coupures_rendent_un_guidage_sans_troisieme_lecture() {
        let socket = test_socket("196-double-read-failure");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            drop_scope196(&listener);
            drop_scope196(&listener);
        });
        let result = execute_tool_at(
            "fixture196",
            "bridget_task_wait",
            json!({"task_ids":["task-a"],"wait_timeout":45})
                .as_object()
                .unwrap(),
            &socket,
        );
        let Err(ToolError::Technical { code, message }) = result else {
            panic!("refus technique attendu")
        };
        assert_eq!(code, "task_wait_read_unavailable");
        let detail: Value = serde_json::from_str(&message).unwrap();
        assert_eq!(detail["stage"], "wait_observation");
        assert!(
            detail["recovery"]
                .as_str()
                .unwrap()
                .contains("bridget_task_status")
        );
        server.join().unwrap();
        std::fs::remove_dir_all(socket.parent().unwrap()).unwrap();
    }
    #[test]
    fn spec196_refus_metier_initial_ne_declenche_pas_de_relecture() {
        let socket = test_socket("196-no-denial-retry");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            reply_caller_scope(
                &listener,
                json!({"status":"refused","code":"permission_attestation_unavailable"}),
            );
            listener.set_nonblocking(true).unwrap();
            assert!(matches!(listener.accept(),Err(e) if e.kind()==io::ErrorKind::WouldBlock));
        });
        let result = execute_tool_at(
            "fixture196",
            "bridget_task_wait",
            json!({"task_ids":["task-a"],"wait_timeout":45})
                .as_object()
                .unwrap(),
            &socket,
        );
        assert!(
            matches!(result,Err(ToolError::Technical {code:"native_delegation_refused",message})
                if message.contains("permission_attestation_unavailable"))
        );
        server.join().unwrap();
        std::fs::remove_dir_all(socket.parent().unwrap()).unwrap();
    }

    #[test]
    fn spec196_coupure_ne_corrige_pas_un_delai_invalide() {
        let socket = test_socket("196-invalid-wait");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            drop_scope196(&listener);
        });
        let result = execute_tool_at(
            "fixture196",
            "bridget_task_wait",
            json!({"task_ids":["task-a"],"wait_timeout":3601})
                .as_object()
                .unwrap(),
            &socket,
        );
        assert!(
            matches!(result,Err(ToolError::Technical {code:"native_delegation_refused",message})
                if message=="wait_timeout_out_of_range")
        );
        server.join().unwrap();
        std::fs::remove_dir_all(socket.parent().unwrap()).unwrap();
    }
    #[test]
    fn spec196_coupure_mutation_ne_declenche_pas_de_relecture() {
        let socket = test_socket("196-no-mutation-retry");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            drop_scope196(&listener);
        });
        let result = execute_tool_at(
            "fixture196",
            "bridget_task_cancel",
            json!({"task_id":"task-a"}).as_object().unwrap(),
            &socket,
        );
        assert!(matches!(
            result,
            Err(ToolError::Technical {
                code: "daemon_unreachable",
                ..
            })
        ));
        server.join().unwrap();
        std::fs::remove_dir_all(socket.parent().unwrap()).unwrap();
    }

    #[test]
    fn mcp158_plafond_enfant_discovery_refuse_ecriture() {
        let cases = [
            ("bridget_delegate", json!({})),
            ("bridget_auto_delegate", json!({})),
            ("bridget_send", json!({"to":"parent","body":"écrit"})),
            ("bridget_task_respond", json!({})),
        ];
        let socket = test_socket("158-14bc9332");
        let listener = UnixListener::bind(&socket).unwrap();
        let count = cases.len();
        let server = thread::spawn(move || {
            for _ in 0..count {
                let scope_result = json!({"version":1,"caller_read_only":true});
                reply_caller_scope(&listener, scope_result);
            }
            listener.set_nonblocking(true).unwrap();
            assert!(
                matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
                "aucun échange métier après le refus"
            );
        });
        for (name, arguments) in cases {
            assert!(
                matches!(
                    execute_tool_at_with_scope(
                        "child158",
                        "instance158",
                        name,
                        arguments.as_object().unwrap(),
                        &socket
                    ),
                    Err(ToolError::Technical {
                        code: "capability_denied",
                        ..
                    })
                ),
                "{name}"
            );
        }
        server.join().unwrap();
        // Une réponse absente, non booléenne ou ouverte ne rend aucun droit.
        let socket = test_socket("mcp158-malformed-scope");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            for scope_result in [
                json!({"version":1}),
                json!({"version":1,"caller_read_only":"false"}),
                json!({"version":2,"caller_read_only":false}),
                json!({"version":1,"caller_read_only":false,"grant":true}),
            ] {
                reply_caller_scope(&listener, scope_result);
            }
        });
        for _ in 0..4 {
            assert!(matches!(
                execute_tool_at_with_scope(
                    "child158",
                    "instance158",
                    "bridget_publish_artifact",
                    &serde_json::Map::new(),
                    &socket
                ),
                Err(ToolError::Technical {
                    code: "daemon_protocol",
                    ..
                })
            ));
        }
        server.join().unwrap();
    }

    #[test]
    fn mcp158_plafond_aucun_effet_de_bord_au_refus() {
        let cases = [
            ("bridget_thread", json!({"action":"post"})),
            ("bridget_cancel", json!({"id":"queued"})),
        ];
        let socket = test_socket("158-eae5697c");
        let listener = UnixListener::bind(&socket).unwrap();
        let count = cases.len();
        let server = thread::spawn(move || {
            for _ in 0..count {
                let scope_result = json!({"version":1,"caller_read_only":true});
                reply_caller_scope(&listener, scope_result);
            }
            listener.set_nonblocking(true).unwrap();
            assert!(
                matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
                "aucun échange métier après le refus"
            );
        });
        for (name, arguments) in cases {
            assert!(
                matches!(
                    execute_tool_at_with_scope(
                        "child158",
                        "instance158",
                        name,
                        arguments.as_object().unwrap(),
                        &socket
                    ),
                    Err(ToolError::Technical {
                        code: "capability_denied",
                        ..
                    })
                ),
                "{name}"
            );
        }
        server.join().unwrap();
    }

    #[test]
    fn mcp158_discovery_publication_artefact_refusee_sans_effet() {
        let cases = [("bridget_publish_artifact", json!({}))];
        let socket = test_socket("158-09a02200");
        let listener = UnixListener::bind(&socket).unwrap();
        let count = cases.len();
        let server = thread::spawn(move || {
            for _ in 0..count {
                let scope_result = json!({"version":1,"caller_read_only":true});
                reply_caller_scope(&listener, scope_result);
            }
            listener.set_nonblocking(true).unwrap();
            assert!(
                matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
                "aucun échange métier après le refus"
            );
        });
        for (name, arguments) in cases {
            assert!(
                matches!(
                    execute_tool_at_with_scope(
                        "child158",
                        "instance158",
                        name,
                        arguments.as_object().unwrap(),
                        &socket
                    ),
                    Err(ToolError::Technical {
                        code: "capability_denied",
                        ..
                    })
                ),
                "{name}"
            );
        }
        server.join().unwrap();
    }

    #[test]
    fn mcp158_discovery_partage_journal_refuse_sans_effet() {
        let cases = [("bridget_journal", json!({"agent":"source","to":"parent"}))];
        let socket = test_socket("158-439fab37");
        let listener = UnixListener::bind(&socket).unwrap();
        let count = cases.len();
        let server = thread::spawn(move || {
            for _ in 0..count {
                let scope_result = json!({"version":1,"caller_read_only":true});
                reply_caller_scope(&listener, scope_result);
            }
            listener.set_nonblocking(true).unwrap();
            assert!(
                matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
                "aucun échange métier après le refus"
            );
        });
        for (name, arguments) in cases {
            assert!(
                matches!(
                    execute_tool_at_with_scope(
                        "child158",
                        "instance158",
                        name,
                        arguments.as_object().unwrap(),
                        &socket
                    ),
                    Err(ToolError::Technical {
                        code: "capability_denied",
                        ..
                    })
                ),
                "{name}"
            );
        }
        server.join().unwrap();
    }

    #[test]
    fn mcp158_plafond_parent_garde_tous_les_outils() {
        let names = tools()
            .into_iter()
            .map(|tool| tool["name"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        let socket = test_socket("mcp158-parent");
        let listener = UnixListener::bind(&socket).unwrap();
        // Les paramètres queue invalides sont refusés avant le RPC CallerScope.
        // Garder tous les noms dans l'oracle, compter seulement les vrais échanges.
        let count = names
            .iter()
            .filter(|name| name.as_str() != "bridget_queue")
            .count();
        let server = thread::spawn(move || {
            for _ in 0..count {
                let scope_result = json!({"version":1,"caller_read_only":false});
                reply_caller_scope(&listener, scope_result);
            }
        });
        for name in names {
            let result = execute_tool_at_with_scope(
                "parent158",
                "instance158",
                &name,
                json!({"forged":1}).as_object().unwrap(),
                &socket,
            );
            assert!(
                matches!(result, Err(ToolError::InvalidParams(_))),
                "{name} : {result:?}"
            );
        }
        server.join().unwrap();
        assert!(matches!(
            execute_tool_at_with_scope(
                "parent158",
                "instance158",
                "outil_inconnu",
                &serde_json::Map::new(),
                &socket
            ),
            Err(ToolError::InvalidParams(_))
        ));

        // Appel public : le nom annoncé atteint le transport natif existant.
        std::fs::remove_file(&socket).unwrap();
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            use std::os::fd::AsRawFd;
            let mut ready = libc::pollfd { fd: listener.as_raw_fd(), events: libc::POLLIN, revents: 0 };
            // SAFETY: `ready` est une structure pollfd locale valide ; le nombre d'éléments passé vaut 1.
            assert!(unsafe { libc::poll(&mut ready, 1, 3000) } > 0);
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            // SAFETY: `ready` est une structure pollfd locale valide ; le nombre d'éléments passé vaut 1.
            assert!(unsafe { libc::poll(&mut ready, 1, 3000) } > 0);
            let (mut stream, _) = listener.accept().unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            assert!(matches!(read_command(&mut reader), WrapperToDaemon::RegisterAuxiliary { .. }));
            writeln!(stream,"{}",json!({"type":"Registered","agent_id":"parent158","credential":null})).unwrap();
            assert!(matches!(read_command(&mut reader), WrapperToDaemon::DaemonIdentityRequest));
            writeln!(stream,"{}",json!({"type":"DaemonIdentityReport","host":"fixture","db_path":"fixture","instance_id":"fixture","features":["auto_delegate_v2"]})).unwrap();
            let request = read_command(&mut reader);
            assert!(matches!(&request, WrapperToDaemon::NativeDelegation {
                request: bridget_transport::protocol::NativeDelegationRequest::AutoDelegateV2(config)
            } if config.request_id == "auto-public-163" && config.task == "mission" && config.cwd == "/tmp"));
            writeln!(stream,"{}",json!({"type":"NativeDelegationResult","result":{"status":"refused","code":"auto_candidate_unavailable"}})).unwrap();
            request
        });
        let identity = || Ok(crate::mcp_identity::ResolvedIdentity {
            name:"parent158".into(),instance_id:"instance158".into(),delegated_origin:None
        });
        let execute = |identity: &crate::mcp_identity::ResolvedIdentity, name: &str, args: &Value| {
            execute_tool_at_with_identity(identity,name,args.as_object().unwrap(),&socket)
        };
        let mut session = Session { initialize_seen:true,initialized:true };
        let response = dispatch_with_executor(&json!({"jsonrpc":"2.0","id":163,"method":"tools/call",
            "params":{"name":"bridget_auto_delegate","arguments":{"request_id":"auto-public-163","task":"mission","cwd":"/tmp"}}}),
            &mut session,&identity,&execute).unwrap();
        server.join().unwrap();
        assert!(response.get("error").is_none(),"aucun -32602 outil inconnu : {response}");
        assert_eq!(response["result"]["code"],"native_delegation_refused");
        assert_eq!(response["result"]["content"][0]["text"],"auto_candidate_unavailable");
        std::fs::remove_file(&socket).unwrap();
    }

    #[test]
    fn mcp158_discovery_chaque_action_de_fil() {
        use bridget_transport::protocol::ThreadResult;
        let socket = test_socket("mcp158-fil-actions");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            for action in [
                "list",
                "show",
                "history",
                "read",
                "ack",
                "create",
                "add_members",
                "post",
                "close",
            ] {
                let scope_result = json!({"version":1,"caller_read_only":true});
                reply_caller_scope(&listener, scope_result);
                if ["list", "show", "history", "read", "ack"].contains(&action) {
                    let (stream, _) = listener.accept().unwrap();
                    stream
                        .set_read_timeout(Some(Duration::from_secs(3)))
                        .unwrap();
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut writer = BufWriter::new(stream);
                    let WrapperToDaemon::RegisterAuxiliary {
                        agent_id,
                        instance_id,
                        credential,
                    } = read_command(&mut reader)
                    else {
                        panic!("inscription auxiliaire attendue")
                    };
                    let expected = crate::mcp_identity::auxiliary_registration(
                        &agent_id,
                        &instance_id,
                        writer
                            .get_ref()
                            .local_addr()
                            .unwrap()
                            .as_pathname()
                            .unwrap(),
                    )
                    .unwrap();
                    assert!(
                        matches!(expected, WrapperToDaemon::RegisterAuxiliary { credential: expected, .. } if expected == credential)
                    );
                    write_command(
                        &mut writer,
                        DaemonToWrapper::Registered {
                            agent_id,
                            credential: None,
                        },
                    );
                    let WrapperToDaemon::ThreadRequest { request } = read_command(&mut reader)
                    else {
                        panic!("requête de fil attendue")
                    };
                    assert_eq!(request.request.name(), action);
                    write_command(
                        &mut writer,
                        DaemonToWrapper::ThreadResult {
                            result: ThreadResult {
                                version: 1,
                                result: json!({"status":"preuve","action":action}),
                            },
                        },
                    );
                }
            }
        });
        let id = "15800000-0000-4000-8000-000000000001";
        for action in [
            "list",
            "show",
            "history",
            "read",
            "ack",
            "create",
            "add_members",
            "post",
            "close",
        ] {
            let arguments = match action {
                "list" => json!({"action":action}),
                "ack" => json!({"action":action,"thread_id":id,"receipt":id}),
                _ => json!({"action":action,"thread_id":id}),
            };
            let result = execute_tool_at_with_scope(
                "child158",
                "instance158",
                "bridget_thread",
                arguments.as_object().unwrap(),
                &socket,
            );
            if ["list", "show", "history", "read", "ack"].contains(&action) {
                assert_eq!(result.unwrap()["action"], action);
            } else {
                assert!(
                    matches!(
                        result,
                        Err(ToolError::Technical {
                            code: "capability_denied",
                            ..
                        })
                    ),
                    "{action}"
                );
            }
        }
        server.join().unwrap();
    }

    // 158B : RED de surfaces seulement. Les oracles de magasin/temps suivent le GO.
    #[test]
    fn wake158b_mcp_surface_wake_et_arguments_fermes() {
        let catalogue = tools();
        let wake = catalogue
            .iter()
            .find(|tool| tool["name"] == "bridget_wake")
            .expect("US5/FR104 : bridget_wake doit être présent dans le vrai catalogue MCP");
        let schema = &wake["inputSchema"];
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(
            schema["properties"]["action"]["enum"],
            json!(["schedule", "list", "cancel"])
        );
        let properties = schema["properties"].as_object().expect("champs de wake");
        for forbidden in [
            "identity",
            "owner",
            "from",
            "issued_at",
            "scope",
            "command_id",
        ] {
            assert!(
                !properties.contains_key(forbidden),
                "champ d'autorité/création refusé : {forbidden}"
            );
        }
        assert_eq!(wake["annotations"]["readOnlyHint"], false);
    }

    #[test]
    fn wake158b_mcp_events_missed_passe_par_identite_attestee() {
        use std::os::fd::AsRawFd;
        let socket = test_socket("158b-missed");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":true}));
            // La base158A refuse encore l'action après CallerScope. Le serveur
            // borné permet d'obtenir le vrai InvalidParams sans thread bloqué.
            let mut wait = libc::pollfd {
                fd: listener.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            // SAFETY: `wait` est une structure pollfd locale valide ; le nombre d'éléments passé vaut 1.
            if unsafe { libc::poll(&mut wait, 1, 3000) } == 0 {
                return false;
            }
            assert!(wait.revents & libc::POLLIN != 0, "requête missed attendue");
            let (stream, _) = listener.accept().unwrap();
            stream.set_nonblocking(false).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            let WrapperToDaemon::RegisterAuxiliary {
                agent_id,
                instance_id,
                credential,
            } = read_command(&mut reader)
            else {
                panic!("inscription auxiliaire requise pour missed");
            };
            assert_eq!(agent_id, "15800000-0000-4000-8000-00000000000b");
            assert_eq!(instance_id, "missed-instance158b");
            let expected = crate::mcp_identity::auxiliary_registration(
                &agent_id,
                &instance_id,
                writer
                    .get_ref()
                    .local_addr()
                    .unwrap()
                    .as_pathname()
                    .unwrap(),
            )
            .unwrap();
            assert!(
                matches!(expected, WrapperToDaemon::RegisterAuxiliary { credential: wanted, .. } if wanted == credential)
            );
            write_command(
                &mut writer,
                DaemonToWrapper::Registered {
                    agent_id,
                    credential: None,
                },
            );
            let WrapperToDaemon::ObservationRequest { request } = read_command(&mut reader) else {
                panic!("action missed via l'enveloppe d'observation existante");
            };
            assert_eq!(serde_json::to_value(request).unwrap()["action"], "missed");
            // Projection de protocole, pas un faux Store ni une preuve de faits.
            write_command(
                &mut writer,
                DaemonToWrapper::ObservationResult {
                    result: json!({"fixture_projection":"missed158b"}),
                },
            );
            true
        });
        let result = execute_tool_at_with_scope(
            "15800000-0000-4000-8000-00000000000b",
            "missed-instance158b",
            "bridget_events",
            json!({"action":"missed"}).as_object().unwrap(),
            &socket,
        );
        let reached_missed = server.join().unwrap();
        std::fs::remove_dir_all(socket.parent().unwrap()).unwrap();
        let actual = result
            .expect("US7 : missed doit être une action valide, pas InvalidParams de la base158A");
        assert!(
            reached_missed,
            "lecture par identité auxiliaire, pas résultat inventé localement"
        );
        assert_eq!(actual["fixture_projection"], "missed158b");
        let events = tools()
            .into_iter()
            .find(|tool| tool["name"] == "bridget_events")
            .unwrap();
        assert!(
            events["inputSchema"]["properties"]["action"]["enum"]
                .as_array()
                .unwrap()
                .iter()
                .any(|action| action == "missed")
        );
        assert_eq!(events["inputSchema"]["additionalProperties"], false);
    }

    #[test]
    fn mcp158_discovery_events_sub_unsub_permis() {
        let socket = test_socket("mcp158-events");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            for _ in 0..2 {
                let scope_result = json!({"version":1,"caller_read_only":true});
                reply_caller_scope(&listener, scope_result);
                let (stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut writer = BufWriter::new(stream);
                let WrapperToDaemon::RegisterAuxiliary {
                    agent_id,
                    instance_id,
                    credential,
                } = read_command(&mut reader)
                else {
                    panic!("inscription auxiliaire attendue")
                };
                let expected = crate::mcp_identity::auxiliary_registration(
                    &agent_id,
                    &instance_id,
                    writer
                        .get_ref()
                        .local_addr()
                        .unwrap()
                        .as_pathname()
                        .unwrap(),
                )
                .unwrap();
                assert!(
                    matches!(expected, WrapperToDaemon::RegisterAuxiliary { credential: expected, .. } if expected == credential)
                );
                write_command(
                    &mut writer,
                    DaemonToWrapper::Registered {
                        agent_id,
                        credential: None,
                    },
                );
                assert!(matches!(
                    read_command(&mut reader),
                    WrapperToDaemon::ObservationRequest { .. }
                ));
                write_command(
                    &mut writer,
                    DaemonToWrapper::ObservationResult {
                        result: json!({"status":"ok"}),
                    },
                );
            }
        });
        for arguments in [
            json!({"action":"sub","event":"turn_ended","once":true}),
            json!({"action":"unsub","id":"subscription158"}),
        ] {
            assert_eq!(
                execute_tool_at_with_scope(
                    "child158",
                    "instance158",
                    "bridget_events",
                    arguments.as_object().unwrap(),
                    &socket
                )
                .unwrap()["status"],
                "ok"
            );
        }
        server.join().unwrap();
    }

    #[test]
    fn mcp158_discovery_lecture_artefact_permise() {
        use bridget_transport::protocol::ArtifactReadOutcome;
        let socket = test_socket("mcp158-artifact-read");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let scope_result = json!({"version":1,"caller_read_only":true});
            reply_caller_scope(&listener, scope_result);
            let (stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            let WrapperToDaemon::RegisterAuxiliary {
                agent_id,
                instance_id,
                credential,
            } = read_command(&mut reader)
            else {
                panic!("inscription auxiliaire attendue")
            };
            let expected = crate::mcp_identity::auxiliary_registration(
                &agent_id,
                &instance_id,
                writer
                    .get_ref()
                    .local_addr()
                    .unwrap()
                    .as_pathname()
                    .unwrap(),
            )
            .unwrap();
            assert!(
                matches!(expected, WrapperToDaemon::RegisterAuxiliary { credential: expected, .. } if expected == credential)
            );
            write_command(
                &mut writer,
                DaemonToWrapper::Registered {
                    agent_id,
                    credential: None,
                },
            );
            let WrapperToDaemon::ArtifactRead { request } = read_command(&mut reader) else {
                panic!("lecture d'artefact attendue")
            };
            write_command(
                &mut writer,
                DaemonToWrapper::ArtifactReadResult {
                    version: 1,
                    artifact_ref: request.artifact_ref,
                    version_ref: request.version_ref,
                    outcome: ArtifactReadOutcome::Chunk {
                        bytes: b"contenu158".to_vec(),
                        total_len: 10,
                        digest: "a".repeat(64),
                        offset: 0,
                        next_offset: None,
                    },
                },
            );
        });
        let result = execute_tool_at_with_scope("child158", "instance158", "bridget_read_artifact", json!({"version":1,"artifact_ref":"artifact158","version_ref":"version158","kind":{"kind":"manifest"},"offset":0,"limit":16}).as_object().unwrap(), &socket).unwrap();
        assert_eq!(result["outcome"]["status"], "chunk");
        assert_eq!(result["outcome"]["bytes"], json!(b"contenu158".to_vec()));
        server.join().unwrap();
    }

    #[test]
    fn mcp158_discovery_journal_avec_et_sans_to() {
        let socket = test_socket("mcp158-journal");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            for _ in 0..2 {
                let scope_result = json!({"version":1,"caller_read_only":true});
                reply_caller_scope(&listener, scope_result);
                let (stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut writer = BufWriter::new(stream);
                assert!(matches!(
                    read_command(&mut reader),
                    WrapperToDaemon::RoleHandshake {
                        role: ConnectionRole::Attach
                    }
                ));
                write_command(
                    &mut writer,
                    DaemonToWrapper::RoleAccepted {
                        role: ConnectionRole::Attach,
                    },
                );
                assert!(matches!(
                    read_command(&mut reader),
                    WrapperToDaemon::Subscribe { .. }
                ));
                write_command(
                    &mut writer,
                    DaemonToWrapper::Subscribed {
                        subscription_id: "s158".into(),
                    },
                );
                write_command(
                    &mut writer,
                    DaemonToWrapper::JournalFragment {
                        subscription_id: "s158".into(),
                        seq: 7,
                        offset: 0,
                        final_fragment: true,
                        bytes: serde_json::to_vec(&json!({"seq":7,"text":"preuve158"})).unwrap(),
                    },
                );
                write_command(
                    &mut writer,
                    DaemonToWrapper::SnapshotCaughtUp {
                        subscription_id: "s158".into(),
                        through_seq: Some(7),
                    },
                );
            }
            for _ in 0..2 {
                let scope_result = json!({"version":1,"caller_read_only":true});
                reply_caller_scope(&listener, scope_result);
            }
        });
        for arguments in [
            json!({"agent":"source"}),
            json!({"agent":"source","to":null}),
        ] {
            let result = execute_tool_at_with_scope(
                "child158",
                "instance158",
                "bridget_journal",
                arguments.as_object().unwrap(),
                &socket,
            )
            .unwrap();
            assert_eq!(result["entries"][0]["text"], "preuve158");
        }
        for to in ["", "parent"] {
            assert!(matches!(
                execute_tool_at_with_scope(
                    "child158",
                    "instance158",
                    "bridget_journal",
                    json!({"agent":"source","to":to}).as_object().unwrap(),
                    &socket
                ),
                Err(ToolError::Technical {
                    code: "capability_denied",
                    ..
                })
            ));
        }
        server.join().unwrap();
    }

    #[test]
    fn mcp158_discovery_lecture_ack_page_suivante() {
        use bridget_transport::protocol::{ThreadAction, ThreadNotify, ThreadRequest};
        const A: &str = "14700000-aaaa-4aaa-8aaa-00000000000a";
        const B: &str = "14700000-bbbb-4bbb-8bbb-00000000000b";
        let socket = test_socket("mcp158-three-pages");
        let root = socket.parent().unwrap().to_path_buf();
        let store = crate::store::Store::open(&root.join("db158")).unwrap();
        let create = crate::threads::handle(
            &store,
            &FixtureDirectory,
            A,
            ThreadRequest {
                version: 1,
                cross_project_reason: None,
                request: ThreadAction::Create {
                    title: "Trois pages158".into(),
                    members: vec![B.into()],
                    operation_id: uuid::Uuid::new_v4().to_string(),
                },
            },
            1,
        );
        assert_eq!(create.result["status"], "created");
        let thread_id = create.result["thread_id"].as_str().unwrap().to_owned();
        for number in 1..=6 {
            let posted = crate::threads::handle(
                &store,
                &FixtureDirectory,
                A,
                ThreadRequest {
                    version: 1,
                    cross_project_reason: None,
                    request: ThreadAction::Post {
                        thread_id: thread_id.clone(),
                        body: format!("entrée{number}"),
                        notify: ThreadNotify::Targets(vec![]),
                        operation_id: uuid::Uuid::new_v4().to_string(),
                        reply_to_seq: None,
                        ack_receipt: None,
                        kind: None,
                        supersedes_seq: None,
                    },
                },
                1,
            );
            assert_eq!(posted.result["status"], "posted");
        }
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            for _ in 0..6 {
                let scope_result = json!({"version":1,"caller_read_only":true});
                reply_caller_scope(&listener, scope_result);
                let (stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(3)))
                    .unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut writer = BufWriter::new(stream);
                let WrapperToDaemon::RegisterAuxiliary {
                    agent_id,
                    instance_id,
                    credential,
                } = read_command(&mut reader)
                else {
                    panic!("inscription auxiliaire attendue")
                };
                let expected = crate::mcp_identity::auxiliary_registration(
                    &agent_id,
                    &instance_id,
                    writer
                        .get_ref()
                        .local_addr()
                        .unwrap()
                        .as_pathname()
                        .unwrap(),
                )
                .unwrap();
                assert!(
                    matches!(expected, WrapperToDaemon::RegisterAuxiliary { credential: expected, .. } if expected == credential)
                );
                write_command(
                    &mut writer,
                    DaemonToWrapper::Registered {
                        agent_id,
                        credential: None,
                    },
                );
                let WrapperToDaemon::ThreadRequest { request } = read_command(&mut reader) else {
                    panic!("read/ack existant attendu")
                };
                assert!(matches!(
                    &request.request,
                    ThreadAction::Read { .. } | ThreadAction::Ack { .. }
                ));
                let result = crate::threads::handle(&store, &FixtureDirectory, B, request, 2);
                write_command(&mut writer, DaemonToWrapper::ThreadResult { result });
            }
        });
        let mut seqs = BTreeSet::new();
        let mut receipts = BTreeSet::new();
        for page_number in 0..3 {
            let page = execute_tool_at_with_scope(
                B,
                "instance158",
                "bridget_thread",
                json!({"action":"read","thread_id":thread_id,"limit":2})
                    .as_object()
                    .unwrap(),
                &socket,
            )
            .unwrap();
            assert_eq!(page["status"], "read");
            assert_eq!(page["has_more"], page_number < 2);
            assert_eq!(page["entries"].as_array().unwrap().len(), 2);
            for entry in page["entries"].as_array().unwrap() {
                assert!(
                    seqs.insert(entry["seq"].as_u64().unwrap()),
                    "une page ne doit jamais revenir après ACK"
                );
            }
            let receipt = page["receipt"].as_str().unwrap().to_owned();
            assert!(receipts.insert(receipt.clone()));
            let ack = execute_tool_at_with_scope(
                B,
                "instance158",
                "bridget_thread",
                json!({"action":"ack","thread_id":thread_id,"receipt":receipt})
                    .as_object()
                    .unwrap(),
                &socket,
            )
            .unwrap();
            assert_eq!(ack["status"], "acknowledged");
        }
        assert_eq!(seqs.into_iter().collect::<Vec<_>>(), vec![1, 2, 3, 4, 5, 6]);
        server.join().unwrap();
    }

    fn read_command(reader: &mut BufReader<UnixStream>) -> WrapperToDaemon {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        decode(line.trim_end()).unwrap()
    }

    fn write_command(writer: &mut BufWriter<UnixStream>, response: DaemonToWrapper) {
        writeln!(writer, "{}", encode(&response).unwrap()).unwrap();
        writer.flush().unwrap();
        if matches!(
            response,
            DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Client
            }
        ) {
            let mut reader = BufReader::new(writer.get_ref().try_clone().unwrap());
            let registration = read_command(&mut reader);
            let WrapperToDaemon::RegisterAuxiliary {
                agent_id,
                instance_id,
                credential,
            } = registration
            else {
                panic!("preuve auxiliaire attendue");
            };
            let address = writer.get_ref().local_addr().unwrap();
            let expected = crate::mcp_identity::auxiliary_registration(
                &agent_id,
                &instance_id,
                address.as_pathname().unwrap(),
            )
            .unwrap();
            assert!(
                matches!(expected, WrapperToDaemon::RegisterAuxiliary { credential: expected, .. }
                if expected == credential)
            );
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::Registered {
                    agent_id,
                    credential: None
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();
        }
    }

    #[test]
    fn mutation_mcp_declare_l_identite_une_fois_et_ne_vend_pas_queued_comme_un_effet() {
        let socket = test_socket("greffe-queued");
        let listener = UnixListener::bind(&socket).unwrap();
        let expected_scope = issuer_scope("instance-greffe-1");
        let server_scope = expected_scope.clone();
        let server = thread::spawn(move || {
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RegisterAuxiliary {


                    agent_id: name,
                    instance_id,
                    ..
                } if name == "jc2"
                    && instance_id == "instance-greffe-1"
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::Registered {
                    credential: None,
                    agent_id: "jc2".to_string(),
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ServiceRequest {
                    issuer_scope,
                    request_id,
                    issued_at: 1_787_824_100,
                    from,
                    operation: ServiceRequestOperation::RegistreAdd,
                    payload: ServiceRequestPayload::RegistreAdd { line },
                    ..
                } if issuer_scope == server_scope
                    && request_id == "request-registre-1"
                    && from == "jc2"
                    && line == "kind=add id=constat-1"
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::GuichetResult {
                    version: SERVICE_CONTRACT_VERSION,
                    issuer_scope: server_scope,
                    request_id: "request-registre-1".to_string(),
                    issue: "queued".to_string(),
                    expires_at: 1_787_824_160,
                    payload: None,
                },
            );
        });

        let result = execute_tool_at_with_scope(
            "jc2",
            "instance-greffe-1",
            "guichet_registre_add",
            json!({
                "line": "kind=add id=constat-1",
                "request_id": "request-registre-1",
                "issued_at": 1_787_824_100_i64,
            })
            .as_object()
            .unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(result["status"], "queued");
        assert_eq!(result["terminal"], false);
        assert_eq!(result["applied"], false);
        assert_eq!(result["next"], "guichet_request_status");
        assert_eq!(result["issuer_scope"], expected_scope);
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn delegate_mcp_transporte_atomiquement_la_cible_de_revue_en_v2() {
        let socket = test_socket("delegate-review-target");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RegisterAuxiliary { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::Registered {
                    credential: None,
                    agent_id: "jc2".to_string(),
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ServiceRequest {
                    version: bridget_transport::protocol::REVIEW_DELEGATE_CONTRACT_VERSION,
                    operation: ServiceRequestOperation::Delegate,
                    payload: ServiceRequestPayload::Delegate {
                        review_target: Some(ReviewTarget { target_ref, expected_head }),
                        ..
                    },
                    ..
                } if target_ref == "origin/session-047-verdict-tete-reecrite"
                    && expected_head == "a".repeat(40)
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::GuichetResult {
                    version: SERVICE_CONTRACT_VERSION,
                    issuer_scope: issuer_scope("instance-review-target"),
                    request_id: "request-review-target".to_string(),
                    issue: "queued".to_string(),
                    expires_at: 1_787_824_560,
                    payload: None,
                },
            );
        });

        let result = execute_tool_at_with_scope(
            "jc2",
            "instance-review-target",
            "guichet_delegate",
            json!({
                "goal": "relire le lot",
                "review_ref": "origin/session-047-verdict-tete-reecrite",
                "expected_head": "a".repeat(40),
                "request_id": "request-review-target",
                "issued_at": 1_787_824_500_i64,
            })
            .as_object()
            .unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(result["status"], "queued");
        server.join().unwrap();
        std::fs::remove_file(&socket).unwrap();

        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            for _ in 0..2 {
                reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            }
        });
        for isolated in [
            json!({ "goal": "relire", "review_ref": "origin/main" }),
            json!({ "goal": "relire", "expected_head": "a".repeat(40) }),
        ] {
            assert!(matches!(
                execute_tool_at_with_scope(
                    "jc2",
                    "instance-review-target",
                    "guichet_delegate",
                    isolated.as_object().unwrap(),
                    &socket,
                ),
                Err(ToolError::InvalidParams(reason))
                    if reason.contains("doivent être fournis ensemble")
            ));
        }
        server.join().unwrap();
    }

    #[test]
    fn coupure_apres_depot_mcp_reste_outcome_unknown_sans_succes_invente() {
        let socket = test_socket("greffe-cut");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RegisterAuxiliary { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::Registered {
                    credential: None,
                    agent_id: "jc2".to_string(),
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ServiceRequest {
                    operation: ServiceRequestOperation::ObjectiveClose,
                    ..
                }
            ));
            // Fermeture volontaire après lecture : le dépôt peut avoir eu lieu,
            // mais aucun résultat terminal n'est attesté à l'appelant.
        });
        let result = execute_tool_at_with_scope(
            "jc2",
            "instance-greffe-2",
            "guichet_objective_close",
            json!({
                "objective_id": "objective-1",
                "reason": "objectif atteint",
                "request_id": "request-close-1",
                "issued_at": 1_787_824_200_i64,
            })
            .as_object()
            .unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(result["status"], "outcome_unknown");
        assert_eq!(result["terminal"], false);
        assert_eq!(result["applied"], false);
        assert_eq!(result["next"], "guichet_request_status");
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn request_status_relit_les_identifiants_terminaux_du_maitre() {
        let socket = test_socket("greffe-status");
        let listener = UnixListener::bind(&socket).unwrap();
        let expected_scope = issuer_scope("instance-greffe-3");
        let server_scope = expected_scope.clone();
        let server = thread::spawn(move || {
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Service
                }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Service,
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ServiceHello {
                    service,
                    issuer_scope,
                    capabilities,
                    ..
                } if service == "guichet"
                    && issuer_scope == server_scope
                    && capabilities == vec![ServiceCapability::GuichetV1]
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::ServiceWelcome {
                    version: SERVICE_CONTRACT_VERSION,
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: vec![ServiceCapability::GuichetV1],
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::GuichetLookup {
                    issuer_scope,
                    request_id,
                    ..
                } if issuer_scope == server_scope && request_id == "request-delegate-1"
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::GuichetResult {
                    version: SERVICE_CONTRACT_VERSION,
                    issuer_scope: server_scope,
                    request_id: "request-delegate-1".to_string(),
                    issue: "accepted".to_string(),
                    expires_at: 1_787_824_360,
                    payload: Some(GuichetReplyPayload::Delegate {
                        status: GuichetDelegateMutationStatus::Created,
                        objective_id: Some("objective-1".to_string()),
                        delegation_id: Some("delegation-1".to_string()),
                        message_id: Some("message-1".to_string()),
                        participant: Some("cursor-1".to_string()),
                        candidates: Vec::new(),
                        waiting_on_prerequisites: false,
                        replayed: false,
                    }),
                },
            );
        });
        let result = execute_tool_at_with_scope(
            "jc2",
            "instance-greffe-3",
            "guichet_request_status",
            json!({ "request_id": "request-delegate-1" })
                .as_object()
                .unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(result["status"], "created");
        assert_eq!(result["terminal"], true);
        assert_eq!(result["applied"], true);
        assert_eq!(result["objective_id"], "objective-1");
        assert_eq!(result["delegation_id"], "delegation-1");
        assert_eq!(result["message_id"], "message-1");
        assert_eq!(result["issuer_scope"], expected_scope);
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn mutation_mcp_refuse_toute_seconde_source_de_principal_ou_de_registre() {
        let socket = test_socket("invalid-scope158");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            for _ in 0..4 {
                reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            }
        });
        for forbidden in ["from", "principal", "database_path", "catalogue_path"] {
            let mut arguments = json!({ "line": "kind=add id=constat-1" });
            arguments[forbidden] = json!("forged");
            assert!(matches!(
                execute_tool_at_with_scope(
                    "jc2",
                    "instance-greffe-4",
                    "guichet_registre_add",
                    arguments.as_object().unwrap(),
                    &socket,
                ),
                Err(ToolError::InvalidParams(reason)) if reason.contains(forbidden)
            ));
        }
        server.join().unwrap();
    }

    #[test]
    fn send_transmet_un_corps_riche_octet_pour_octet_et_negocie_le_client() {
        let socket = test_socket("send-rich");
        let listener = UnixListener::bind(&socket).unwrap();
        let expected_body =
            "l'apostrophe, \"guillemets\", $VAR, `backticks`,\net l'emoji é".to_string();
        let server = thread::spawn({
            let expected_body = expected_body.clone();
            move || {
                reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
                let (stream, _) = listener.accept().unwrap();
                let read_stream = stream.try_clone().unwrap();
                let mut reader = BufReader::new(read_stream);
                let mut writer = BufWriter::new(stream);
                assert!(matches!(
                    read_command(&mut reader),
                    WrapperToDaemon::RoleHandshake {
                        role: ConnectionRole::Client
                    }
                ));
                write_command(
                    &mut writer,
                    DaemonToWrapper::RoleAccepted {
                        role: ConnectionRole::Client,
                    },
                );
                assert!(matches!(
                    read_command(&mut reader),
                    WrapperToDaemon::ClientHello { .. }
                ));
                write_command(
                    &mut writer,
                    DaemonToWrapper::ClientWelcome {
                        version: CLIENT_CONTRACT_VERSION,
                        build_id: "test-build".to_string(),
                        horizon_secs: 60,
                        issued_at_tolerance_secs: 5,
                        capabilities: vec![ClientCapability::SendIdempotent],
                    },
                );
                match read_command(&mut reader) {
                    WrapperToDaemon::SendIdempotent {
                        message,
                        message_id,
                        ..
                    } => {
                        assert_eq!(message.body, expected_body);
                        assert_eq!(message.in_reply_to, None);
                        assert_eq!(message_id, "retry-me");
                    }
                    other => panic!("commande inattendue: {other:?}"),
                }
                write_command(
                    &mut writer,
                    DaemonToWrapper::IdempotencyResult {
                        operation_kind: "send".to_string(),
                        idempotency_key: "retry-me".to_string(),
                        project_warnings: Vec::new(),
                        issue: IdempotencyIssue::Accepted { expires_at: 60 },
                    },
                );
            }
        });
        let result = execute_tool_at(
            "codex-1",
            "bridget_send",
            json!({
            "to": "claude-1", "body": expected_body, "id": "retry-me"
            , "issued_at": 1_700_000_000
            })
            .as_object()
            .unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(result["status"], "accepted");
        assert_eq!(result["id"], "retry-me");
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn spec133_send_signe_par_le_parent_et_transporte_la_provenance() {
        let socket = test_socket("send-delegated");
        let listener = UnixListener::bind(&socket).unwrap();
        crate::mcp_identity::mock_private_identity(&socket, "parent", "instance-parent");
        let server = thread::spawn(move || {
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Client
                }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client,
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ClientHello { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: "test-build".into(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: vec![ClientCapability::SendIdempotent],
                },
            );
            match read_command(&mut reader) {
                WrapperToDaemon::SendIdempotent { message, .. } => {
                    assert_eq!(message.from, "parent");
                    assert_eq!(
                        message.delegated_origin,
                        Some(bridget_core::DelegatedOrigin {
                            provider: "codex".into(),
                            child_ref: "0123456789abcdef".into(),
                        })
                    );
                }
                other => panic!("commande inattendue: {other:?}"),
            }
            write_command(
                &mut writer,
                DaemonToWrapper::IdempotencyResult {
                    operation_kind: "send".into(),
                    idempotency_key: "delegated-message".into(),
                    project_warnings: Vec::new(),
                    issue: IdempotencyIssue::Accepted { expires_at: 60 },
                },
            );
        });
        let identity = crate::mcp_identity::ResolvedIdentity {
            name: "parent".into(),
            instance_id: "instance-parent".into(),
            delegated_origin: Some(bridget_core::DelegatedOrigin {
                provider: "codex".into(),
                child_ref: "0123456789abcdef".into(),
            }),
        };
        let result = execute_tool_at_with_identity(
            &identity,
            "bridget_send",
            json!({
                "to": "destinataire",
                "body": "résultat",
                "id": "delegated-message",
                "issued_at": 1_700_000_000
            })
            .as_object()
            .unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(result["status"], "accepted");
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn send_transmet_in_reply_to_dans_l_enveloppe_idempotente() {
        let socket = test_socket("send-reply");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Client
                }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client,
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ClientHello { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: "test-build".to_string(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: vec![ClientCapability::SendIdempotent],
                },
            );
            match read_command(&mut reader) {
                WrapperToDaemon::SendIdempotent { message, .. } => {
                    assert_eq!(message.in_reply_to.as_deref(), Some("request-open"));
                    assert!(!message.reply);
                }
                other => panic!("commande inattendue: {other:?}"),
            }
            write_command(
                &mut writer,
                DaemonToWrapper::IdempotencyResult {
                    operation_kind: "send".to_string(),
                    idempotency_key: "reply-1".to_string(),
                    project_warnings: Vec::new(),
                    issue: IdempotencyIssue::Accepted { expires_at: 60 },
                },
            );
        });
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "bridget_send",
                "arguments": {
                    "to": "bridget",
                    "body": "réponse liée",
                    "in_reply_to": "request-open",
                    "id": "reply-1",
                    "issued_at": 1_700_000_000
                }
            }
        });
        let mut session = Session {
            initialize_seen: true,
            initialized: true,
        };
        let resolver = || {
            Ok(crate::mcp_identity::ResolvedIdentity {
                name: "codex-1".to_string(),
                instance_id: "test-instance".to_string(),
                delegated_origin: None,
            })
        };
        let execute =
            |identity: &crate::mcp_identity::ResolvedIdentity, name: &str, arguments: &Value| {
                execute_tool_at_with_scope(
                    &identity.name,
                    &identity.instance_id,
                    name,
                    arguments.as_object().expect("arguments objet"),
                    &socket,
                )
            };
        let result = dispatch_with_executor(&request, &mut session, &resolver, &execute).unwrap();
        assert_eq!(result["result"]["structuredContent"]["status"], "accepted");
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn schema_send_expose_in_reply_to_non_vide() {
        let send = tools()
            .into_iter()
            .find(|tool| tool["name"] == "bridget_send")
            .unwrap();
        assert_eq!(
            send["inputSchema"]["properties"]["in_reply_to"]["type"],
            "string"
        );
        assert_eq!(
            send["inputSchema"]["properties"]["in_reply_to"]["minLength"],
            1
        );
    }

    #[test]
    fn resultats_metier_conservent_les_categories_de_refus_fermees() {
        for (category, expected) in [
            ("dnd", "dnd"),
            ("circuit_breaker", "circuit_breaker"),
            ("duplicate_content", "duplicate"),
            ("routing", "unknown_recipient"),
        ] {
            let result = send_issue_result(
                "id-1",
                1_700_000_000,
                IdempotencyIssue::Rejected {
                    category: category.to_string(),
                    reason: "motif exact".to_string(),
                    expires_at: 1_700_000_100,
                },
            );
            assert_eq!(result["status"], expected);
            assert_eq!(result["reason"], "motif exact");
        }
    }

    #[test]
    fn retry_rejoue_scope_et_horodatage_malgre_rename() {
        let socket = test_socket("retry-scope");
        let listener = UnixListener::bind(&socket).unwrap();
        let (seen_tx, seen_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            for _ in 0..2 {
                reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
                let (stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut writer = BufWriter::new(stream);
                assert!(matches!(
                    read_command(&mut reader),
                    WrapperToDaemon::RoleHandshake { .. }
                ));
                write_command(
                    &mut writer,
                    DaemonToWrapper::RoleAccepted {
                        role: ConnectionRole::Client,
                    },
                );
                let hello = read_command(&mut reader);
                write_command(
                    &mut writer,
                    DaemonToWrapper::ClientWelcome {
                        version: CLIENT_CONTRACT_VERSION,
                        build_id: "test-build".to_string(),
                        horizon_secs: 60,
                        issued_at_tolerance_secs: 5,
                        capabilities: vec![ClientCapability::SendIdempotent],
                    },
                );
                let send = read_command(&mut reader);
                seen_tx.send((hello, send)).unwrap();
                write_command(
                    &mut writer,
                    DaemonToWrapper::IdempotencyResult {
                        operation_kind: "send".to_string(),
                        idempotency_key: "retry-1".to_string(),
                        project_warnings: Vec::new(),
                        issue: IdempotencyIssue::Accepted {
                            expires_at: 1_700_000_060,
                        },
                    },
                );
            }
        });
        let arguments = json!({
            "to": "claude-1", "body": "même prompt", "id": "retry-1", "issued_at": 1_700_000_000
        });
        for (index, identity) in ["avant-rename", "apres-rename"].into_iter().enumerate() {
            if index == 1 {
                thread::sleep(Duration::from_secs(1));
            }
            let result = execute_tool_at_with_scope(
                identity,
                "instance-stable-1",
                "bridget_send",
                arguments.as_object().unwrap(),
                &socket,
            )
            .unwrap();
            assert_eq!(result["issued_at"], 1_700_000_000);
        }
        let first = seen_rx.recv().unwrap();
        let second = seen_rx.recv().unwrap();
        let scope = |command: WrapperToDaemon| match command {
            WrapperToDaemon::ClientHello { issuer_scope, .. } => issuer_scope,
            other => panic!("hello attendu: {other:?}"),
        };
        assert_eq!(scope(first.0), scope(second.0));
        for command in [first.1, second.1] {
            match command {
                WrapperToDaemon::SendIdempotent {
                    message, issued_at, ..
                } => {
                    assert_eq!(issued_at, 1_700_000_000);
                    assert_eq!(message.body, "même prompt");
                }
                other => panic!("send attendu: {other:?}"),
            }
        }
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    /// Le cas nominal ne doit plus s'annoncer comme un incident : le daemon
    /// répond avant l'accusé du destinataire, donc TOUT premier envoi passe
    /// par là. Annoncer « accusé perdu » sur un succès, c'est inviter au
    /// double envoi — le défaut mesuré sur ~100 envois d'une seule journée.
    ///
    /// Le statut lui-même doit changer : un lecteur branche sur `status`, pas
    /// sur la prose du motif.
    #[test]
    fn une_remise_en_vol_ne_s_annonce_pas_comme_un_accuse_perdu() {
        let issue = IdempotencyIssue::OutcomeUnknown {
            expires_at: 1_700_000_060,
            delivery_id: Some("livraison-7".to_string()),
        };
        let rendered = send_issue_result("msg-1", 1_700_000_000, issue);
        assert_eq!(rendered["status"], "in_flight");
        assert_eq!(rendered["delivery_id"], "livraison-7");
        // Forme CLOSE : diagnostic puis consigne, rien avant, rien après. Un
        // `contains` laissait passer tout préfixe ajouté — dont un préfixe qui
        // affirme un dépôt que personne n'a constaté.
        assert_eq!(
            rendered["reason"].as_str().unwrap(),
            format!("{DIAGNOSTIC_REMISE_EN_VOL} ; {REJEU_A_L_IDENTIQUE}")
        );
    }

    /// Gardien du point de vérité : les autres oracles vérifient que chaque
    /// ancrage porte CETTE chaîne, celui-ci vérifie ce que la chaîne dit. Sans
    /// lui, la consigne pourrait se vider de son sens sans faire rougir un
    /// seul test — c'est exactement ainsi que le troisième invariant avait
    /// disparu de la version précédente.
    #[test]
    fn la_consigne_de_rejeu_nomme_ses_trois_invariants_et_l_absence_de_doublon() {
        // Chercher « id » ne gardait RIEN : « id » est déjà contenu dans
        // « à l'identique ». Amputer la consigne de « même id, » laissait donc
        // ce gardien vert — sur l'invariant précisément qui avait disparu de la
        // version précédente, celui qu'il était censé protéger. Chaque
        // invariant se vérifie sous la forme qui l'énonce, pas sous un fragment
        // que le reste de la phrase fournit déjà.
        for invariant in ["même id", "même issued_at", "même corps"] {
            assert!(
                REJEU_A_L_IDENTIQUE.contains(invariant),
                "la consigne doit nommer l'invariant « {invariant} »"
            );
        }
        assert!(
            REJEU_A_L_IDENTIQUE.contains("dupliquer"),
            "sans la promesse de non-duplication, le rejeu reste redouté et personne ne l'ose"
        );
    }

    /// Gardien des diagnostics : la forme close verrouille la COMPOSITION du
    /// motif, celui-ci verrouille ce que chaque morceau affirme. Sans lui, il
    /// suffirait de réécrire une constante pour qu'un sort inconnu s'annonce
    /// comme un dépôt attesté — les oracles de forme resteraient verts, car ils
    /// compareraient le rendu à la constante mensongère elle-même.
    ///
    /// Une seule des trois formes atteste une remise. Les deux autres portent
    /// sur des cas où rien n'est constaté : elles ne doivent rien promettre.
    #[test]
    fn seul_le_diagnostic_de_remise_en_vol_atteste_quelque_chose() {
        assert!(
            DIAGNOSTIC_REMISE_EN_VOL.contains("remise en vol"),
            "le seul cas où le daemon a pris la remise doit le dire"
        );
        for (diagnostic, nom) in [
            (DIAGNOSTIC_SORT_INDETERMINE, "sort indéterminé"),
            (DIAGNOSTIC_ACCUSE_PERDU, "accusé perdu"),
        ] {
            for promesse in ["remise en vol", "attesté", "atteste", "réussi", "déposé"] {
                assert!(
                    !diagnostic.contains(promesse),
                    "le diagnostic « {nom} » porte sur un sort NON constaté : \
                     il ne doit rien promettre, or il contient « {promesse} »"
                );
            }
        }
    }

    /// Contre-épreuve : sans `delivery_id`, le sort est vraiment inconnu et le
    /// retour ne doit pas rassurer. Si ce test tombe, le correctif a effacé la
    /// distinction qu'il avait pour but d'établir — c'est-à-dire qu'il aurait
    /// remplacé un mensonge pessimiste par un mensonge optimiste, bien pire
    /// dans un système d'attestation.
    #[test]
    fn un_sort_indetermine_ne_promet_pas_une_remise() {
        let issue = IdempotencyIssue::OutcomeUnknown {
            expires_at: 1_700_000_060,
            delivery_id: None,
        };
        let rendered = send_issue_result("msg-2", 1_700_000_000, issue);
        assert_eq!(rendered["status"], "outcome_unknown");
        assert!(rendered.get("delivery_id").is_none());
        assert_eq!(
            rendered["reason"].as_str().unwrap(),
            format!("{DIAGNOSTIC_SORT_INDETERMINE} ; {REJEU_A_L_IDENTIQUE}")
        );
    }

    /// L'invariant qui porte tout le correctif : les deux cas ne partagent pas
    /// leur statut. Les deux tests précédents pourraient rester verts alors que
    /// les statuts auraient reconvergé sur une valeur commune ; celui-ci le
    /// constate directement.
    #[test]
    fn la_remise_en_vol_et_le_sort_inconnu_ne_partagent_pas_leur_statut() {
        let en_vol = send_issue_result(
            "msg-3",
            1_700_000_000,
            IdempotencyIssue::OutcomeUnknown {
                expires_at: 1_700_000_060,
                delivery_id: Some("livraison-8".to_string()),
            },
        );
        let inconnu = send_issue_result(
            "msg-3",
            1_700_000_000,
            IdempotencyIssue::OutcomeUnknown {
                expires_at: 1_700_000_060,
                delivery_id: None,
            },
        );
        assert_ne!(
            en_vol["status"], inconnu["status"],
            "un dépôt réussi et un sort inconnu doivent se lire sur le statut seul"
        );
    }

    /// Le discriminant vit à UN seul endroit, donc les deux surfaces ne peuvent
    /// pas diverger. Sans cet oracle, rien n'empêche le retour MCP et la sortie
    /// du binaire de répondre différemment à la même issue — et un agent qui
    /// lit les deux n'aurait aucun moyen de savoir laquelle croire.
    ///
    /// Le cas `Some("")` est celui qui les faisait déjà diverger : la ligne du
    /// binaire annonçait « en vol » quand son propre code de sortie refusait le
    /// dépôt.
    #[test]
    fn le_statut_mcp_et_le_depot_du_binaire_ne_peuvent_pas_se_contredire() {
        for (delivery_id, statut_attendu, depot_attendu) in [
            (Some("livraison-9"), STATUT_IN_FLIGHT, true),
            (None, STATUT_OUTCOME_UNKNOWN, false),
            (Some(""), STATUT_OUTCOME_UNKNOWN, false),
            (Some("   "), STATUT_OUTCOME_UNKNOWN, false),
        ] {
            let issue = IdempotencyIssue::OutcomeUnknown {
                expires_at: 1_700_000_060,
                delivery_id: delivery_id.map(str::to_string),
            };
            let rendu = send_issue_result("msg-4", 1_700_000_000, issue.clone());
            assert_eq!(
                rendu["status"], statut_attendu,
                "statut MCP pour delivery_id={delivery_id:?}"
            );
            assert_eq!(
                crate::cli::send_deposited(&issue),
                depot_attendu,
                "le binaire doit conclure comme le statut MCP pour delivery_id={delivery_id:?}"
            );
            // La preuve n'est publiée que lorsqu'elle atteste quelque chose.
            assert_eq!(
                rendu.get("delivery_id").is_some(),
                depot_attendu,
                "un delivery_id sans valeur probante ne doit pas être publié"
            );
        }
    }

    /// Oracle : `orphaned` n'est ni `in_flight` ni `outcome_unknown`.
    #[test]
    fn statut_orphelin_distinct_de_in_flight_et_outcome_unknown() {
        let issue = IdempotencyIssue::Orphaned {
            expires_at: 1_700_000_060,
            delivery_id: "delivery-orphelin".to_string(),
            reason: "destinataire purgé".to_string(),
        };
        let rendu = send_issue_result("msg-orphelin", 1_700_000_000, issue.clone());
        assert_eq!(rendu["status"], STATUT_ORPHELIN);
        assert_ne!(rendu["status"], STATUT_IN_FLIGHT);
        assert_ne!(rendu["status"], STATUT_OUTCOME_UNKNOWN);
        assert_eq!(rendu["delivery_id"], "delivery-orphelin");
        assert!(
            !crate::cli::send_deposited(&issue),
            "un orphelin n'est pas un dépôt réussi à conclure en rc=0"
        );
        let reason = rendu["reason"].as_str().unwrap_or("");
        assert!(
            reason.contains(CONDUITE_ORPHELIN),
            "orphaned doit porter la conduite, pas seulement le constat: {reason}"
        );
        assert!(
            !reason.contains(REJEU_A_L_IDENTIQUE),
            "orphaned ne doit PAS enseigner le rejeu à l'identique (absorbant): {reason}"
        );
    }

    #[test]
    fn coupure_apres_transmission_devient_outcome_unknown_et_le_retry_reste_identique() {
        let socket = test_socket("cut-after-send");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RoleHandshake { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client,
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ClientHello { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: "test-build".to_string(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: vec![ClientCapability::SendIdempotent],
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::SendIdempotent { .. }
            ));
            drop(writer);
            drop(reader);

            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));

            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RoleHandshake { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client,
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ClientHello { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: "test-build".to_string(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: vec![ClientCapability::SendIdempotent],
                },
            );
            match read_command(&mut reader) {
                WrapperToDaemon::SendIdempotent {
                    message_id,
                    issued_at,
                    ..
                } => {
                    assert_eq!(message_id, "retry-cut");
                    assert_eq!(issued_at, 1_700_000_000);
                }
                other => panic!("send attendu: {other:?}"),
            }
            write_command(
                &mut writer,
                DaemonToWrapper::IdempotencyResult {
                    operation_kind: "send".to_string(),
                    idempotency_key: "retry-cut".to_string(),
                    project_warnings: Vec::new(),
                    issue: IdempotencyIssue::Accepted {
                        expires_at: 1_700_000_060,
                    },
                },
            );
        });
        let arguments = json!({
            "to":"claude-1", "body":"retry", "id":"retry-cut", "issued_at":1_700_000_000
        });
        let first = execute_tool_at_with_scope(
            "agent",
            "instance",
            "bridget_send",
            arguments.as_object().unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(first["status"], "outcome_unknown");
        let retry = execute_tool_at_with_scope(
            "agent-renamed",
            "instance",
            "bridget_send",
            arguments.as_object().unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(retry["status"], "accepted");
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    /// L'oracle du mandat : un envoi dont l'accusé aval est retardé rend un
    /// statut HONNÊTE (remise en vol, avec sa preuve), et le rejeu de la même
    /// clé lit le sort réel — en portant EXACTEMENT la même enveloppe, jamais
    /// une nouvelle émission. C'est le geste que le libellé d'origine faisait
    /// redouter alors qu'il est le seul chemin correct.
    #[test]
    fn accuse_retarde_rend_un_statut_honnete_puis_le_rejeu_lit_le_sort_sans_dupliquer() {
        let socket = test_socket("ack-differe");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let mut envelopes = Vec::new();
            for issue in [
                IdempotencyIssue::OutcomeUnknown {
                    expires_at: 1_700_000_060,
                    delivery_id: Some("livraison-differee".to_string()),
                },
                IdempotencyIssue::Accepted {
                    expires_at: 1_700_000_060,
                },
            ] {
                reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
                let (stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut writer = BufWriter::new(stream);
                assert!(matches!(
                    read_command(&mut reader),
                    WrapperToDaemon::RoleHandshake { .. }
                ));
                write_command(
                    &mut writer,
                    DaemonToWrapper::RoleAccepted {
                        role: ConnectionRole::Client,
                    },
                );
                assert!(matches!(
                    read_command(&mut reader),
                    WrapperToDaemon::ClientHello { .. }
                ));
                write_command(
                    &mut writer,
                    DaemonToWrapper::ClientWelcome {
                        version: CLIENT_CONTRACT_VERSION,
                        build_id: "test-build".to_string(),
                        horizon_secs: 60,
                        issued_at_tolerance_secs: 5,
                        capabilities: vec![ClientCapability::SendIdempotent],
                    },
                );
                match read_command(&mut reader) {
                    WrapperToDaemon::SendIdempotent {
                        message_id,
                        issued_at,
                        message,
                        ..
                    } => envelopes.push((message_id, issued_at, message.body)),
                    other => panic!("send attendu: {other:?}"),
                }
                write_command(
                    &mut writer,
                    DaemonToWrapper::IdempotencyResult {
                        operation_kind: "send".to_string(),
                        idempotency_key: "ack-differe-1".to_string(),
                        project_warnings: Vec::new(),
                        issue,
                    },
                );
            }
            envelopes
        });
        let arguments = json!({
            "to":"bridget", "body":"livraison", "id":"ack-differe-1", "issued_at":1_700_000_000
        });
        let first = execute_tool_at_with_scope(
            "fable2",
            "instance",
            "bridget_send",
            arguments.as_object().unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(first["status"], "in_flight");
        assert_eq!(first["delivery_id"], "livraison-differee");
        assert!(!first["reason"].as_str().unwrap().contains("perdu"));

        let replay = execute_tool_at_with_scope(
            "fable2",
            "instance",
            "bridget_send",
            arguments.as_object().unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(replay["status"], "accepted");

        let envelopes = server.join().unwrap();
        assert_eq!(
            envelopes[0], envelopes[1],
            "le rejeu doit porter la même enveloppe, sinon il duplique au lieu de consulter"
        );
        std::fs::remove_file(socket).unwrap();
    }

    /// ORACLE C4 — la TROISIÈME forme d'`outcome_unknown`, celle qui naît côté
    /// client sans aucune issue du daemon : la connexion tombe APRÈS l'écriture
    /// de la commande, donc l'outil ne lira jamais la réponse.
    ///
    /// Ce chemin était le seul des trois à n'avoir aucun filet sur son
    /// contenu : le banc de coupure voisin ne vérifie que le `status`, si bien
    /// qu'une mutation du corps du retour y survivait. Or c'est justement le
    /// cas où l'appelant a le plus besoin de la consigne de rejeu — le message
    /// a pu partir, et lui seul l'ignore.
    ///
    /// L'oracle verrouille donc le contrat ENTIER de ce bras : le statut, les
    /// deux clés sans lesquelles aucun rejeu n'est possible, et les trois
    /// invariants du rejeu à l'identique.
    #[test]
    fn la_coupure_apres_ecriture_rend_les_cles_de_rejeu_et_les_trois_invariants() {
        let socket = test_socket("coupure-contrat");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RoleHandshake { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client,
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::ClientHello { .. }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: "test-build".to_string(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: vec![ClientCapability::SendIdempotent],
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::SendIdempotent { .. }
            ));
            // La coupure : la commande est écrite et lue, aucune réponse ne
            // vient. C'est ce qui distingue ce cas des deux autres.
            drop(writer);
            drop(reader);
        });
        let rendered = execute_tool_at_with_scope(
            "fable2",
            "instance",
            "bridget_send",
            json!({
                "to":"bridget", "body":"coupure", "id":"coupure-1", "issued_at":1_700_000_000
            })
            .as_object()
            .unwrap(),
            &socket,
        )
        .unwrap();

        assert_eq!(rendered["status"], "outcome_unknown");
        assert!(
            rendered.get("delivery_id").is_none(),
            "aucune issue n'a été lue : rien n'atteste une remise, et rien ne doit le prétendre"
        );
        // Sans ces deux clés, la consigne de rejeu est irréalisable.
        assert_eq!(rendered["id"], "coupure-1");
        assert_eq!(rendered["issued_at"], 1_700_000_000i64);

        // Forme CLOSE. Porter la consigne ne suffit pas : un motif peut la
        // citer mot pour mot et affirmer juste avant un dépôt que personne n'a
        // constaté. Sur CE chemin le mensonge optimiste est le pire de tous —
        // aucune issue n'a été lue, le message a pu ne jamais partir.
        //
        // Le seul ajout tolérable est le détail technique final entre
        // parenthèses : il vient de l'erreur d'entrée-sortie, il n'est pas
        // rédigé, et il varie d'une plateforme à l'autre. Tout le reste est
        // verrouillé au caractère près.
        let reason = rendered["reason"].as_str().unwrap();
        let attendu = format!("{DIAGNOSTIC_ACCUSE_PERDU} — {REJEU_A_L_IDENTIQUE} (");
        assert!(
            reason.starts_with(&attendu),
            "le motif doit être exactement le diagnostic puis la consigne, sans rien avant ni entre: {reason}"
        );
        assert!(
            reason.ends_with(')'),
            "seul le détail technique entre parenthèses peut suivre la consigne: {reason}"
        );

        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn issues_idempotentes_deterministes_restent_metier_et_inconnues_sont_preservees() {
        for (issue, status) in [
            (IdempotencyIssue::EnvelopeMismatch, "envelope_mismatch"),
            (IdempotencyIssue::IdempotencyExpired, "idempotency_expired"),
            (IdempotencyIssue::InvalidIssuedAt, "invalid_issued_at"),
        ] {
            assert_eq!(send_issue_result("id", 7, issue)["status"], status);
        }
        let unknown = send_issue_result(
            "id",
            7,
            IdempotencyIssue::Rejected {
                category: "future_refusal".to_string(),
                reason: "raison".to_string(),
                expires_at: 8,
            },
        );
        assert_eq!(unknown["status"], "future_refusal");
    }

    #[test]
    fn resultat_mcp_duplique_le_payload_dans_textcontent() {
        let payload = json!({ "status": "accepted", "id": "m-1", "issued_at": 8 });
        let result = tool_result(payload.clone());
        assert_eq!(result["structuredContent"], payload);
        assert_eq!(
            serde_json::from_str::<Value>(result["content"][0]["text"].as_str().unwrap()).unwrap(),
            result["structuredContent"]
        );
    }

    #[test]
    fn huit_connexions_simultanees_gardent_le_principal_resolu_et_la_neuvieme_est_busy() {
        let started = Arc::new(Barrier::new(MAX_IN_FLIGHT_TOOL_CALLS + 1));
        let (started_tx, started_rx) = mpsc::channel();
        let socket = test_socket("eight-registers");
        let listener = UnixListener::bind(&socket).unwrap();
        crate::mcp_identity::mock_private_identity(&socket, "fixture-agent", "fixture-instance");
        let (principals_tx, principals_rx) = mpsc::channel();
        let daemon = thread::spawn(move || {
            for _ in 0..MAX_IN_FLIGHT_TOOL_CALLS {
                let (stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut writer = BufWriter::new(stream);
                let registered_agent = match read_command(&mut reader) {
                    WrapperToDaemon::RegisterAuxiliary {
                        agent_id: name,
                        instance_id,
                        ..
                    } => {
                        principals_tx.send((name.clone(), instance_id)).unwrap();
                        name
                    }
                    other => panic!("Register MCP attendu: {other:?}"),
                };
                write_command(
                    &mut writer,
                    DaemonToWrapper::Registered {
                        credential: None,
                        agent_id: registered_agent,
                    },
                );
            }
        });
        let input = [
            json!({"jsonrpc":"2.0","id":1,"method":"initialize"}),
            json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        ]
        .into_iter()
        .chain((2..=10).map(|id| {
            json!({
                "jsonrpc":"2.0", "id": id, "method":"tools/call",
                "params":{"name":"bridget_who","arguments":{}}
            })
        }))
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join("\n");
        let barrier = Arc::clone(&started);
        let socket_for_calls = socket.clone();
        let server = thread::spawn(move || {
            let resolver = || {
                Ok(crate::mcp_identity::ResolvedIdentity {
                    name: "agent".to_string(),
                    instance_id: "instance".to_string(),
                    delegated_origin: None,
                })
            };
            let execute = move |_: &crate::mcp_identity::ResolvedIdentity, _: &str, _: &Value| {
                let _connection =
                    registered_connection("fixture-agent", "fixture-instance", &socket_for_calls)
                        .unwrap();
                started_tx.send(()).unwrap();
                barrier.wait();
                Ok(json!({ "agents": [] }))
            };
            let mut output = Vec::new();
            serve_with(Cursor::new(input), &mut output, &resolver, &execute).unwrap();
            output
        });
        for _ in 0..MAX_IN_FLIGHT_TOOL_CALLS {
            started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        }
        started.wait();
        let output = String::from_utf8(server.join().unwrap()).unwrap();
        let responses = output
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            responses
                .iter()
                .filter(|response| response["result"]["code"] == "busy")
                .count(),
            1
        );

        let principals = (0..MAX_IN_FLIGHT_TOOL_CALLS)
            .map(|_| principals_rx.recv_timeout(Duration::from_secs(2)).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(principals.len(), MAX_IN_FLIGHT_TOOL_CALLS);
        assert!(principals.iter().all(|(name, instance_id)| {
            name == "fixture-agent" && instance_id == "fixture-instance"
        }));
        daemon.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn connexion_non_bloquante_refuse_un_budget_expire_avant_toute_attente() {
        let socket = test_socket("expired-connect");
        let _listener = UnixListener::bind(&socket).unwrap();
        let error =
            connect_nonblocking(&socket, Instant::now() - Duration::from_millis(1)).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn dto_ledger_respectent_le_contrat_outil() {
        let source = bridget_transport::protocol::LedgerMessage {
            id: "m-1".to_string(),
            ts: 4,
            sender: "alice".to_string(),
            target: "bob".to_string(),
            body: "riche".to_string(),
            delivery_status: Some(bridget_transport::protocol::LedgerDeliveryStatus::EnVol),
        };
        let message = ledger_message_dto(source.clone());
        assert_eq!(
            message,
            json!({"id":"m-1","from":"alice","to":"bob","body":"riche","ts":4,"delivery_status":"en_vol"})
        );
        assert_eq!(
            crate::cli::render_ledger(&[source]),
            "Derniers 1 messages :\n  [4] alice → bob [en vol]: riche\n"
        );
        let request = request_dto(bridget_transport::protocol::RequestInfo {
            id: "r-1".to_string(),
            sender: "alice".to_string(),
            target: "bob".to_string(),
            state: "open".to_string(),
            created_at: 2,
            deadline_at: 9,
            cancel_reason: None,
            deferred_reminder_level: None,
            deferred_reminder_at: None,
        });
        assert_eq!(
            request,
            json!({"id":"r-1","from":"alice","to":"bob","state":"open","deadline":9,"created":2})
        );
    }

    #[test]
    fn ledger_requests_scope_mine_par_defaut_et_all_accepte() {
        let schema = tools()
            .into_iter()
            .find(|tool| tool["name"] == "bridget_ledger")
            .unwrap();
        assert_eq!(
            schema["inputSchema"]["properties"]["requests_scope"]["default"],
            "mine"
        );
        assert_eq!(
            schema["inputSchema"]["properties"]["requests_scope"]["enum"],
            json!(["mine", "all"])
        );

        let mut args = serde_json::Map::new();
        args.insert("view".into(), json!("requests"));
        assert_eq!(parse_requests_scope(&args).unwrap(), RequestsScope::Mine);
        args.insert("requests_scope".into(), json!("all"));
        assert_eq!(parse_requests_scope(&args).unwrap(), RequestsScope::All);
        args.insert("requests_scope".into(), json!("everyone"));
        assert!(parse_requests_scope(&args).is_err());
    }

    #[test]
    fn daemon_coupe_est_un_is_error_technique_immediat() {
        let missing = test_socket("missing");
        let error = execute_tool_at("codex-1", "bridget_who", &serde_json::Map::new(), &missing)
            .unwrap_err();
        assert!(matches!(
            error,
            ToolError::Technical {
                code: "daemon_unreachable",
                ..
            }
        ));
    }

    #[test]
    fn limite_d_appels_simultanes_refuse_avant_toute_connexion() {
        let mut registry = ToolCallRegistry::default();
        for id in 0..MAX_IN_FLIGHT_TOOL_CALLS {
            assert!(registry.reserve(id.to_string(), false).is_ok());
        }
        assert_eq!(registry.reserve("overflow".into(), false), Err("busy"));
        assert_eq!(registry.calls.len(), MAX_IN_FLIGHT_TOOL_CALLS);
    }

    #[test]
    fn spec138_who_inherits_project_and_defaults_to_local_directory() {
        let socket = test_socket("who");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            let (stream, _) = listener.accept().unwrap();
            let read_stream = stream.try_clone().unwrap();
            let mut reader = BufReader::new(read_stream);
            let mut writer = BufWriter::new(stream);
            let WrapperToDaemon::RegisterAuxiliary { agent_id, .. } = read_command(&mut reader)
            else {
                panic!("identité du caller exigée")
            };
            write_command(
                &mut writer,
                DaemonToWrapper::Registered {
                    agent_id,
                    credential: None,
                },
            );
            assert!(
                matches!(read_command(&mut reader), WrapperToDaemon::ClientHello { capabilities, .. } if capabilities == vec![ClientCapability::CommunicationProjectsV1])
            );
            write_command(
                &mut writer,
                DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: "fixture".into(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 60,
                    capabilities: vec![ClientCapability::CommunicationProjectsV1],
                },
            );
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::DirectoryScoped {
                    scope: bridget_transport::protocol::CommunicationDirectoryScope::SameProject
                }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::CommunicationDirectory {
                    agents: Vec::new(),
                    project: None,
                    project_warnings: Vec::new(),
                },
            );
        });
        let result =
            execute_tool_at("codex-1", "bridget_who", &serde_json::Map::new(), &socket).unwrap();
        assert_eq!(
            result,
            json!({ "agents": [], "project":null, "project_warnings":[], "scope":"same_project", "total_matching":0, "live":true, "has_more":false, "next_cursor":null })
        );
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn spec138_valid_send_reason_reaches_transport_without_rewriting_body() {
        let socket = test_socket("reason-missing");
        let arguments = json!({"to":"11111111-1111-4111-8111-111111111111", "body":"exact body", "cross_project_reason":"Comparer le contrat"});
        let error = execute_tool_at(
            "codex-1",
            "bridget_send",
            arguments.as_object().unwrap(),
            &socket,
        )
        .unwrap_err();
        assert!(
            matches!(
                error,
                ToolError::Technical {
                    code: "daemon_unreachable",
                    ..
                }
            ),
            "{error:?}"
        );
    }

    #[test]
    fn spec138_invalid_reasons_and_scope_are_rejected_before_transport() {
        let socket = test_socket("invalid-reason");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            for _ in 0..8 {
                reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            }
        });
        for reason in [
            Value::Null,
            json!(" "),
            json!("x\nsecret"),
            json!("é".repeat(257)),
            json!(12),
        ] {
            let args = json!({"to":"11111111-1111-4111-8111-111111111111", "body":"unchanged", "cross_project_reason":reason});
            assert!(matches!(
                execute_tool_at(
                    "codex-1",
                    "bridget_send",
                    args.as_object().unwrap(),
                    &socket
                ),
                Err(ToolError::InvalidParams(_))
            ));
        }
        for scope in [Value::Null, json!(true), json!("all")] {
            assert!(matches!(
                execute_tool_at(
                    "codex-1",
                    "bridget_who",
                    json!({"scope":scope}).as_object().unwrap(),
                    &socket
                ),
                Err(ToolError::InvalidParams(_))
            ));
        }
        server.join().unwrap();
    }

    #[test]
    fn spec138_old_server_without_capability_receives_no_message() {
        let socket = test_socket("old-server");
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            let (stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Client
                }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client,
                },
            );
            assert!(
                matches!(read_command(&mut reader), WrapperToDaemon::ClientHello {capabilities, ..} if capabilities.contains(&ClientCapability::CommunicationProjectsV1))
            );
            write_command(
                &mut writer,
                DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: "old-server".into(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 60,
                    capabilities: vec![ClientCapability::SendIdempotent],
                },
            );
            let mut line = String::new();
            assert_eq!(
                reader.read_line(&mut line).unwrap(),
                0,
                "aucun message après capacité absente"
            );
        });
        let args = json!({"to":"11111111-1111-4111-8111-111111111111", "body":"PRIVATE_PROMPT unchanged", "cross_project_reason":"Revue volontaire"});
        assert!(matches!(
            execute_tool_at(
                "codex-1",
                "bridget_send",
                args.as_object().unwrap(),
                &socket
            ),
            Err(ToolError::Technical {
                code: "communication_projects_unsupported",
                ..
            })
        ));
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn spec138_send_returns_structured_warning_outside_exact_body() {
        let socket = test_socket("warning-body");
        let listener = UnixListener::bind(&socket).unwrap();
        let body = "PRIVATE_PROMPT exact\nOPAQUE_REASONING";
        let warning = bridget_transport::protocol::ProjectWarning {
            code: "cross_project".into(),
            sender_project: Some(bridget_transport::protocol::CommunicationProject {
                host: "test".into(),
                root: "/a".into(),
            }),
            recipient_project: Some(bridget_transport::protocol::CommunicationProject {
                host: "test".into(),
                root: "/b".into(),
            }),
            recipient: "11111111-1111-4111-8111-111111111111".into(),
            reason: Some("Revue volontaire".into()),
        };
        let expected = warning.clone();
        let server = thread::spawn(move || {
            reply_caller_scope(&listener, json!({"version":1,"caller_read_only":false}));
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut writer = BufWriter::new(stream);
            assert!(matches!(
                read_command(&mut reader),
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Client
                }
            ));
            write_command(
                &mut writer,
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client,
                },
            );
            assert!(
                matches!(read_command(&mut reader), WrapperToDaemon::ClientHello {capabilities,..} if capabilities.contains(&ClientCapability::CommunicationProjectsV1))
            );
            write_command(
                &mut writer,
                DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: "fixture".into(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 60,
                    capabilities: vec![
                        ClientCapability::SendIdempotent,
                        ClientCapability::CommunicationProjectsV1,
                    ],
                },
            );
            let WrapperToDaemon::SendIdempotent {
                message,
                message_id,
                ..
            } = read_command(&mut reader)
            else {
                panic!("send attendu")
            };
            assert_eq!(message.body, body);
            assert_eq!(
                message.cross_project_reason.as_deref(),
                Some("Revue volontaire")
            );
            write_command(
                &mut writer,
                DaemonToWrapper::IdempotencyResult {
                    operation_kind: "send".into(),
                    idempotency_key: message_id,
                    issue: IdempotencyIssue::Accepted { expires_at: 60 },
                    project_warnings: vec![warning],
                },
            );
        });
        let arguments = json!({"to":"11111111-1111-4111-8111-111111111111", "body":body,"cross_project_reason":" Revue volontaire "});
        let result = execute_tool_at(
            "codex-1",
            "bridget_send",
            arguments.as_object().unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(result["project_warnings"], json!([expected]));
        assert_eq!(result["status"], "accepted");
        assert!(
            !result["project_warnings"]
                .to_string()
                .contains("PRIVATE_PROMPT")
        );
        server.join().unwrap();
        std::fs::remove_file(socket).unwrap();
    }

    #[test]
    fn spec138_real_daemon_mcp_directory_and_cross_project_send_share_guard() {
        scoped_facades_real_daemon(None);
    }

    #[test]
    #[ignore = "requiert un script Agent Loop et un binaire local explicites, jamais la production"]
    fn spec138_real_agent_loop_cli_daemon_scoped_replay() {
        let script = std::env::var_os("BRIDGET_SPEC138_AGENT_LOOP_SCRIPT")
            .map(std::path::PathBuf::from)
            .expect("BRIDGET_SPEC138_AGENT_LOOP_SCRIPT explicite requis");
        let binary = std::env::var_os("BRIDGET_SPEC138_BIN")
            .map(std::path::PathBuf::from)
            .expect("BRIDGET_SPEC138_BIN explicite requis");
        assert!(script.is_absolute() && script.is_file());
        assert!(binary.is_absolute() && binary.is_file());
        scoped_facades_real_daemon(Some((script, binary)));
    }

    // Réutilise le même daemon privé que les projections CLI/MCP : aucun second harnais.
    fn scoped_facades_real_daemon(loop_inputs: Option<(std::path::PathBuf, std::path::PathBuf)>) {
        use bridget_transport::protocol::{CommunicationProjectSource, ProjectRelation};
        let (socket, _) = crate::cli::idempotency_projection_tests::start_real_daemon();
        let root = socket.parent().unwrap();
        let project_a = root.join("a");
        let project_b = root.join("b");
        for project in [&project_a, &project_b] {
            std::fs::create_dir(project).unwrap();
            assert!(
                std::process::Command::new("git")
                    .args(["init", "--quiet"])
                    .arg(project)
                    .status()
                    .unwrap()
                    .success()
            );
        }
        let mut owner = crate::cli::idempotency_projection_tests::owner_connection(&socket);
        let host = crate::build_info::local_host();
        assert!(matches!(
            owner
                .exchange(&WrapperToDaemon::CommunicationProjectFact {
                    root: project_a.to_string_lossy().into_owned(),
                    source: CommunicationProjectSource::Git,
                    host: host.clone(),
                    worktree_root: None
                })
                .unwrap(),
            DaemonToWrapper::ProjectContextResult {
                project: Some(_),
                ..
            }
        ));
        let recipient = "89000000-0000-4000-8000-000000000201";
        let mut target = DaemonConnection::connect(&socket).unwrap();
        assert!(matches!(
            target
                .exchange(&WrapperToDaemon::Register {
                    identity_version: 2,
                    agent_type: "fixture".into(),
                    agent_id: recipient.into(),
                    host: Some(host.clone()),
                    transport: Some("test".into()),
                    channel: bridget_transport::ChannelReport::Unknown,
                    mode: Some(bridget_transport::protocol::PresenceMode::Cli),
                    location: None,
                    os: None,
                    instance_id: Some("test-recipient138".into()),
                    domain: Some("same-visible-name".into()),
                    turn_in_progress: false,
                    journal_available: Some(false),
                    turn_attest: false,
                    queue_fallback: false,
                })
                .unwrap(),
            DaemonToWrapper::Registered { .. }
        ));
        assert!(matches!(
            target
                .exchange(&WrapperToDaemon::CommunicationProjectFact {
                    root: project_b.to_string_lossy().into_owned(),
                    source: CommunicationProjectSource::Git,
                    host,
                    worktree_root: None
                })
                .unwrap(),
            DaemonToWrapper::ProjectContextResult {
                project: Some(_),
                ..
            }
        ));
        let identity = crate::cli::idempotency_projection_tests::caller_identity();
        let who = super::execute_tool_at_with_identity(
            &identity,
            "bridget_who",
            json!({}).as_object().unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(who["agents"].as_array().unwrap().len(), 1);
        assert_eq!(who["agents"][0]["agent_id"], identity.name);
        let all = super::execute_tool_at_with_identity(
            &identity,
            "bridget_who",
            json!({"scope":"global"}).as_object().unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(all["agents"].as_array().unwrap().len(), 2);
        let external = all["agents"]
            .as_array()
            .unwrap()
            .iter()
            .find(|agent| agent["agent_id"] == recipient)
            .unwrap();
        assert_eq!(
            external["project_relation"],
            serde_json::to_value(ProjectRelation::Other).unwrap()
        );
        let rejected = super::execute_tool_at_with_identity(
            &identity,
            "bridget_send",
            json!({"to":recipient,"body":"PRIVATE_PROMPT unchanged"})
                .as_object()
                .unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(rejected["status"], "cross_project_reason_required");
        assert!(
            rejected
                .to_string()
                .contains("cross_project_reason_required")
        );
        let sent = super::execute_tool_at_with_identity(&identity,"bridget_send",json!({"to":recipient,"body":"PRIVATE_PROMPT unchanged","cross_project_reason":"Revue volontaire"}).as_object().unwrap(),&socket).unwrap();
        assert!(
            matches!(sent["status"].as_str(), Some("accepted" | "in_flight")),
            "{sent}"
        );
        assert_eq!(sent["project_warnings"][0]["code"], "cross_project");
        let response = target.read_response("test-delivery").unwrap();
        let DaemonToWrapper::DeliverIdempotent { message, .. } = response else {
            panic!("remise réelle attendue, réponse observée : {response:?}")
        };
        assert_eq!(message.body, "PRIVATE_PROMPT unchanged");
        assert_eq!(
            message.cross_project_reason.as_deref(),
            Some("Revue volontaire")
        );
        let connection = rusqlite::Connection::open(root.join("bridget.db")).unwrap();
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM ledger WHERE target=?1",
                [recipient],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "le refus n'a rien déposé");

        // La vraie projection CLI de fond négocie son contexte propre sans Register.
        let mut bare = BridgetMessage::new("human", recipient, "Bare exact body");
        let mut options = crate::cli::IdempotentSendOptions {
            id: bare.id.clone(),
            issued_at: now_secs(),
            issuer_scope: issuer_scope("test138-background-client"),
        };
        let refused = crate::cli::send_idempotent_to_daemon_at_with_project(
            &socket,
            &bare,
            &options,
            None,
            Some(&project_a),
        )
        .unwrap();
        assert!(
            matches!(refused,DaemonToWrapper::IdempotencyResult {issue:IdempotencyIssue::Rejected {reason,..},..} if reason.contains("cross_project_reason_required"))
        );
        bare.cross_project_reason = Some("Relance mandatée".into());
        options.id = uuid::Uuid::new_v4().to_string();
        bare.id = options.id.clone();
        let result = crate::cli::send_idempotent_to_daemon_at_with_project(
            &socket,
            &bare,
            &options,
            None,
            Some(&project_a),
        )
        .unwrap();
        let DaemonToWrapper::IdempotencyResult {
            issue,
            project_warnings,
            ..
        } = result
        else {
            panic!("issue CLI attendue")
        };
        assert!(crate::cli::send_deposited(&issue), "{issue:?}");
        assert_eq!(project_warnings[0].code, "cross_project");
        let DaemonToWrapper::DeliverIdempotent { message, .. } =
            target.read_response("test-cli-delivery").unwrap()
        else {
            panic!("remise CLI réelle attendue")
        };
        assert_eq!(message.from, "human");
        assert_eq!(message.body, "Bare exact body");
        let all = super::execute_tool_at_with_identity(
            &identity,
            "bridget_who",
            json!({"scope":"global"}).as_object().unwrap(),
            &socket,
        )
        .unwrap();
        assert_eq!(
            all["agents"].as_array().unwrap().len(),
            2,
            "le client CLI de fond ne crée aucun faux agent"
        );

        if let Some((script, binary)) = loop_inputs {
            let python = r#"
import argparse, contextlib, importlib.util, io, json, pathlib, subprocess, sys, time, uuid
spec = importlib.util.spec_from_file_location("agent_loop_spec138", sys.argv[1])
loop = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = loop
spec.loader.exec_module(loop)
project, owner, recipient, run_parent = sys.argv[2:6]
local = loop.bridget_agents(project_root=project)
assert {agent["agent_id"] for agent in local} == {owner}, local
global_agents = loop.bridget_agents(project_root=project, global_scope=True)
assert len(global_agents) == 2, global_agents
external = next(agent for agent in global_agents if agent["agent_id"] == recipient)
assert external["project_relation"] == "other", external
run = pathlib.Path(run_parent) / "loop-fixture"
run.mkdir()
(run / "run.json").write_text(json.dumps({"run_id":"loop-fixture", "project_root":project, "policies":{}}))
args = argparse.Namespace(root=run_parent, run_id="loop-fixture", role="root", target=recipient,
                          backend="existing_bridget", session_id="", jsonl_path="", notes="",
                          cross_project_reason="")
before = {str(path):path.read_bytes() for path in run.rglob("*") if path.is_file()}
try:
    loop.cmd_attach_agent(args)
except SystemExit as error:
    assert str(error) == "cross_project_reason_required", error
else:
    raise AssertionError("attribution interprojets sans mandat acceptée")
assert before == {str(path):path.read_bytes() for path in run.rglob("*") if path.is_file()}
args.cross_project_reason = "Mandat réel de recette"
with contextlib.redirect_stdout(io.StringIO()):
    assert loop.cmd_attach_agent(args) == 0
session = json.loads((run / "sessions" / ("root-" + recipient[:12] + ".json")).read_text())
assert session["cross_project_mandate"] == {"agent_id":recipient, "role":"root", "reason":"Mandat réel de recette"}
context = {"id": str(uuid.uuid4()), "issued_at": int(time.time()),
           "issuer_scope": "spec138-loop-background-receipt", "project_root": project}
try:
    loop.send_bridget_message(recipient, "Loop exact PRIVATE_PROMPT body", replay=context)
except subprocess.CalledProcessError:
    pass
else:
    raise AssertionError("envoi inter-projets sans motif accepté")
context.update(id=str(uuid.uuid4()), **loop.bridget_scope_context(json.loads((run / "run.json").read_text()), session, external, "root"))
loop.send_bridget_message(recipient, "Loop exact PRIVATE_PROMPT body", replay=context)
loop.send_bridget_message(recipient, "Loop exact PRIVATE_PROMPT body", replay=context)
assert len(loop.bridget_agents(project_root=project, global_scope=True)) == 2
print(json.dumps({"same_project":len(local),"global":len(global_agents),
                  "replay":"same_receipt","body":"exact","attach":"mandated"}))
"#;
            let output = std::process::Command::new("python3")
                .arg("-c")
                .arg(python)
                .arg(&script)
                .arg(&project_a)
                .arg(&identity.name)
                .arg(recipient)
                .arg(root)
                .env("AGENT_LOOP_BRIDGET_BIN", &binary)
                .env("BRIDGET_HOME", root)
                .env("BRIDGET_SOCKET", &socket)
                .env("TMPDIR", root)
                .env("PYTHONDONTWRITEBYTECODE", "1")
                .env("BRIDGET_AGENT_ID", "borrowed-t3-identity-must-be-removed")
                .env(
                    "BRIDGET_AGENT_INSTANCE_ID",
                    "borrowed-instance-must-be-removed",
                )
                .current_dir(&project_b)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "Agent Loop échoue dans le namespace isolé : {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let stderr = String::from_utf8(output.stderr).unwrap();
            assert!(stderr.contains("AVERTISSEMENT PROJET: "));
            assert!(stderr.contains("cross_project"));
            assert!(!stderr.contains("PRIVATE_PROMPT"));
            let summary: Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(summary["same_project"], 1);
            assert_eq!(summary["global"], 2);
            assert_eq!(summary["attach"], "mandated");
            let DaemonToWrapper::DeliverIdempotent { message, .. } =
                target.read_response("test-loop-delivery").unwrap()
            else {
                panic!("remise Agent Loop → CLI → daemon attendue")
            };
            assert_eq!(message.from, "human");
            assert_eq!(message.body, "Loop exact PRIVATE_PROMPT body");
            assert_eq!(
                message.cross_project_reason.as_deref(),
                Some("Mandat réel de recette")
            );
            let count: i64 = connection
                .query_row(
                    "SELECT COUNT(*) FROM ledger WHERE target=?1",
                    [recipient],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(
                count, 3,
                "deux projections initiales et une remise Loop ; aucun dépôt au refus ni au rejeu"
            );
        }
    }
