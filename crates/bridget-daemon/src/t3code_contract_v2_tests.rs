    use super::*;
    #[test]
    fn tracking230_rpc_ne_confond_pas_erreur_et_refus_certain_correle() {
        let command=json!({"type":"thread.settle","commandId":"tracking-command"});
        let mut exit=json!({"_tag":"Failure","cause":[{"_tag":"Fail","error":{"_tag":"OrchestrationV2DispatchCommandError","commandId":"tracking-command","commandType":"thread.settle","cause":{"name":"OrchestratorCommandPreviouslyRejectedError"}}}]});
        assert!(tracking_rejection(&exit,&command));
        exit["cause"][0]["error"]["cause"]["name"]=json!("OrchestratorDispatchError");
        assert!(!tracking_rejection(&exit,&command),"une erreur peut suivre un commit");
        exit["cause"][0]["error"]["cause"]["name"]=json!("OrchestratorCommandPreviouslyRejectedError");
        exit["cause"][0]["error"]["commandId"]=json!("another-command");
        assert!(!tracking_rejection(&exit,&command));
        exit["cause"][0]["error"]["commandId"]=json!("tracking-command");
        exit["cause"].as_array_mut().unwrap().push(json!({"_tag":"Die"}));
        assert!(!tracking_rejection(&exit,&command),"une cause ambiguë reste inconnue");
    }
    use crate::t3code_contract::Client;
    use std::io::{BufRead, BufReader};
    use std::net::TcpListener;

    /// Serveur privé préparé par l'opérateur. Les runs restent différés : aucun fournisseur.
    #[test]
    #[ignore = "requires a disposable T3 V2 server and scoped fixture token"]
    fn spec147_v2_real_server_interop() {
        let base = std::env::var("BRIDGET_TEST_T3_URL").unwrap();
        let token = std::env::var("BRIDGET_TEST_T3_TOKEN").unwrap();
        let workspace = std::env::var("BRIDGET_TEST_T3_WORKSPACE").unwrap();
        assert!(workspace.starts_with("/private/tmp/t3-bridget-v2."));
        let workspace = format!("{workspace}/fixture-workspace");
        let client = Client::for_base_url(&base, &token);
        let snapshot = client.snapshot().unwrap();
        assert!(client.uses_v2());
        assert!(
            snapshot.threads.iter().all(|t| t.latest_turn.is_none()),
            "refus d'un environnement avec historique"
        );
        let project = uuid::Uuid::new_v4().to_string();
        let thread = uuid::Uuid::new_v4().to_string();
        rpc(&base,&token,"projects.mutate",&json!({"type":"project.create","commandId":uuid::Uuid::new_v4().to_string(),"projectId":project,"title":"Fixture Bridget147","workspaceRoot":workspace,"createWorkspaceRootIfMissing":true})).unwrap();
        rpc(&base,&token,"orchestration.dispatchCommand",&json!({"type":"thread.create","commandId":uuid::Uuid::new_v4().to_string(),"threadId":thread,"projectId":project,"title":"Fixture sans modèle","createdBy":"agent","creationSource":"mcp","modelSelection":{"instanceId":"codex","model":"gpt-5"},"runtimeMode":"approval-required","interactionMode":"default","branch":null,"worktreePath":null})).unwrap();
        let command = json!({"type":"thread.turn.start","commandId":uuid::Uuid::new_v4().to_string(),"threadId":thread,
            "message":{"messageId":uuid::Uuid::new_v4().to_string(),"role":"user","text":"Fixture différée sans fournisseur"}});
        let mut native = dispatch_command(&command, None).unwrap();
        native["dispatchMode"] = json!({"type":"defer_start"});
        let first = rpc(&base, &token, "orchestration.dispatchCommand", &native).unwrap();
        assert_eq!(
            rpc(&base, &token, "orchestration.dispatchCommand", &native).unwrap(),
            first
        );
        let summary = client.snapshot().unwrap();
        assert_eq!(summary.threads.len(), snapshot.threads.len() + 1);
        assert_eq!(
            summary
                .threads
                .iter()
                .find(|t| t.id == thread)
                .unwrap()
                .workspace_root
                .as_deref(),
            Some(workspace.as_str())
        );
        let detail = client.thread_detail(&thread, 20).unwrap();
        assert_eq!(detail.messages.len(), 1);
        assert_eq!(detail.messages[0].text, "Fixture différée sans fournisseur");
        assert!(detail.messages[0].turn_id.is_some());
        assert_eq!(detail.latest_turn.unwrap().state, "preparing");
        let full = client.thread_detail(&thread, 40).unwrap();
        assert_eq!(full.messages, detail.messages);
        let mut missing = command;
        missing["threadId"] = json!(uuid::Uuid::new_v4().to_string());
        missing["commandId"] = json!(uuid::Uuid::new_v4().to_string());
        assert!(matches!(
            client.dispatch(&missing, None),
            Err(ContractError::Transport(_))
        ));
    }

    pub(crate) fn http_reply(listener: &TcpListener, status: u16, body: &str, expected: &str) {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut headers = String::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            headers.push_str(&line);
            if line == "\r\n" || line.is_empty() {
                break;
            }
        }
        assert!(headers.starts_with(expected), "{headers}");
        assert!(
            headers
                .to_lowercase()
                .contains("authorization: bearer fixture")
        );
        write!(
            stream,
            "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
    }

    pub(crate) fn rpc_reply(listener: &TcpListener, tag: &str, value: Option<Value>) -> Value {
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut socket = tungstenite::accept_hdr(
            stream,
            |request: &tungstenite::handshake::server::Request, response| {
                assert_eq!(request.uri().to_string(), "/ws?orchestrationProtocol=2");
                assert_eq!(request.headers()["authorization"], "Bearer fixture");
                Ok(response)
            },
        )
        .unwrap();
        let frame: Value = serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
        assert_eq!(frame["tag"], tag);
        assert_eq!(frame["_tag"], "Request");
        if let Some(value) = value {
            socket.send(tungstenite::Message::Text(json!([
                {"_tag":"Exit","requestId":"other","exit":{"_tag":"Success","value":"ignore"}},
                {"_tag":"Exit","requestId":"bridget","exit":{"_tag":"Success","value":value}}
            ]).to_string().into())).unwrap();
        }
        frame["payload"].clone()
    }

    #[test]
    fn spec147_v2_client_detects_reads_and_replays_stable_rpc_ids() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let client = Client::for_base_url(
            &format!("http://{}", listener.local_addr().unwrap()),
            "fixture",
        );
        let server = std::thread::spawn(move || {
            http_reply(
                &listener,
                200,
                r#"{"environmentId":"fixture","serverVersion":"fixture","orchestrationProtocolVersion":2}"#,
                "GET /.well-known/t3/environment ",
            );
            http_reply(
                &listener,
                200,
                &shell().to_string(),
                "GET /api/orchestration/shell ",
            );
            rpc_reply(
                &listener,
                "server.getConfig",
                Some(json!({"providers":[{"instanceId":"claude_glm","driver":"claude"}]})),
            );
            http_reply(
                &listener,
                200,
                &detail().to_string(),
                "GET /api/orchestration/threads/t/bounded?compactTurnItems=1 ",
            );
            let first = rpc_reply(
                &listener,
                "orchestration.dispatchCommand",
                Some(json!({"sequence":42})),
            );
            let second = rpc_reply(
                &listener,
                "orchestration.dispatchCommand",
                Some(json!({"sequence":42})),
            );
            assert_eq!(first, second);
            assert_eq!(first["commandId"], "cmd");
            assert_eq!(first["messageId"], "m");
            assert_eq!(first["dispatchMode"]["type"], "queue_after_active");
            assert!(first.get("runtimeMode").is_none());
            assert!(first.get("deliveryIntent").is_none());
        });
        assert_eq!(
            client.snapshot().unwrap().threads[0]
                .session
                .as_ref()
                .unwrap()
                .provider_name,
            "claude"
        );
        assert!(client.uses_v2());
        assert_eq!(client.thread_detail("t", 20).unwrap().messages.len(), 2);
        let command = json!({"type":"thread.turn.start","commandId":"cmd","threadId":"t",
            "runtimeMode":"full-access","message":{"messageId":"m","role":"user","text":"exact"}});
        assert_eq!(client.dispatch(&command, None).unwrap().sequence, 42);
        assert_eq!(client.dispatch(&command, None).unwrap().sequence, 42);
        assert_eq!(
            dispatch_command(&command, Some("steer")).unwrap()["deliveryIntent"],
            "steer"
        );
        server.join().unwrap();
    }

    #[test]
    fn spec147_v2_detection_never_masks_auth_or_malformed_response() {
        for (status, body) in [
            (401, "secret"),
            (403, "secret"),
            (500, "secret"),
            (200, "{}"),
            (200, "<html/>"),
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let client = Client::for_base_url(
                &format!("http://{}", listener.local_addr().unwrap()),
                "fixture",
            );
            let server = std::thread::spawn(move || {
                http_reply(&listener, status, body, "GET /.well-known/t3/environment ");
                listener.set_nonblocking(true).unwrap();
                assert!(listener.accept().is_err());
            });
            let error = client.snapshot().unwrap_err();
            assert!(!error.to_string().contains("secret"));
            assert!(!client.uses_v2());
            server.join().unwrap();
        }
    }

    #[test]
    fn spec147_v2_announced_protocol_requires_authenticated_snapshot() {
        for version in [1, 2, 3] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let client = Client::for_base_url(
                &format!("http://{}", listener.local_addr().unwrap()),
                "fixture",
            );
            let server = std::thread::spawn(move || {
                http_reply(&listener,200,&json!({"environmentId":"fixture","serverVersion":"fixture","orchestrationProtocolVersion":version}).to_string(),"GET /.well-known/t3/environment ");
                if version < 3 {
                    http_reply(
                        &listener,
                        401,
                        "secret",
                        if version == 2 {
                            "GET /api/orchestration/shell "
                        } else {
                            "GET /api/orchestration/snapshot "
                        },
                    );
                }
            });
            assert!(client.snapshot().is_err());
            assert!(!client.uses_v2());
            server.join().unwrap();
        }
    }

    #[test]
    fn spec147_v2_lost_rpc_reply_is_uncertain_without_http_fallback() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let client = Client::for_base_url(
            &format!("http://{}", listener.local_addr().unwrap()),
            "fixture",
        );
        client.set_test_protocol(true);
        let server =
            std::thread::spawn(move || rpc_reply(&listener, "orchestration.dispatchCommand", None));
        let command = json!({"type":"thread.turn.start","commandId":"cmd","threadId":"t","message":{"messageId":"m","role":"user","text":"exact"}});
        assert!(matches!(
            client.dispatch(&command, None),
            Err(ContractError::Transport(_))
        ));
        assert_eq!(server.join().unwrap()["messageId"], "m");
    }

    #[test]
    fn spec147_v2_v1_detection_preserves_legacy_detail() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let client = Client::for_base_url(
            &format!("http://{}", listener.local_addr().unwrap()),
            "fixture",
        );
        let server = std::thread::spawn(move || {
            http_reply(
                &listener,
                200,
                r#"{"environmentId":"fixture","serverVersion":"fixture"}"#,
                "GET /.well-known/t3/environment ",
            );
            http_reply(
                &listener,
                200,
                r#"{"snapshotSequence":0,"projects":[],"threads":[]}"#,
                "GET /api/orchestration/snapshot ",
            );
            http_reply(
                &listener,
                200,
                r#"{"thread":{"messages":[]}}"#,
                "GET /api/orchestration/threads/t?turnLimit=20 ",
            );
        });
        assert!(client.snapshot().unwrap().threads.is_empty());
        assert!(!client.uses_v2());
        assert!(client.thread_detail("t", 20).unwrap().messages.is_empty());
        server.join().unwrap();
    }

    fn shell() -> Value {
        json!({"schemaVersion":2,"snapshotSequence":17,"projects":[{"id":"p","workspaceRoot":"/fixture/project"}],
            "threads":[{"id":"t","projectId":"p","title":"test","providerInstanceId":"claude_glm",
                "modelSelection":{"instanceId":"claude_glm"},"runtimeMode":"approval-required","interactionMode":"default",
                "lineage":{"relationshipToParent":"subagent","parentThreadId":"parent"},
                "latestRunId":"r","latestRunRequestedAt":"2026-10-09T00:00:00.000Z","activeRunId":"r","status":"running",
                "updatedAt":"2026-10-09T00:00:01.000Z","archivedAt":null,"deletedAt":null,"settledOverride":null}]})
    }

    #[test]
    fn mobile183_compaction_items_keep_root_lineage_and_status_identity() {
        let mut raw=detail();
        raw["projection"]["turnItems"]=json!([
            {"id":"comp183","type":"compaction","threadId":"t","runId":"r","nodeId":"root","status":"running"},
            {"id":"child-comp","type":"compaction","threadId":"t","runId":"r","nodeId":"child","status":"running"}
        ]);
        let parsed=parse_detail(&raw.to_string(),"t").unwrap();
        let acts:Vec<_>=parsed.activities.iter().filter(|a|a.kind=="context.compaction").collect();
        assert_eq!(acts.len(),1);assert_eq!(acts[0].payload["status"],"running");
        let first=acts[0].id.clone();
        raw["projection"]["turnItems"][0]["status"]=json!("completed");
        let done=parse_detail(&raw.to_string(),"t").unwrap();
        assert_ne!(done.activities.iter().find(|a|a.kind=="context.compaction").unwrap().id,first);
        raw["projection"]["turnItems"][0]["threadId"]=json!("foreign");
        assert!(parse_detail(&raw.to_string(),"t").is_err());
    }

    #[test]
    fn spec221_compact_t3_snapshot_keeps_local_tools_and_rejects_foreign_rows() {
        let mut plain = detail();
        let root=json!({"id":"compact221","type":"compaction","threadId":"t","runId":"r","nodeId":"root","status":"running"});
        let child=json!({"id":"child221","type":"compaction","threadId":"t","runId":"r","nodeId":"child","status":"running"});
        let tail=json!({"id":"tail221","type":"compaction","threadId":"t","runId":"r","nodeId":"root","status":"completed"});
        plain["projection"]["turnItems"]=json!([root.clone(),child.clone(),tail.clone()]);
        let expected=parse_detail(&plain.to_string(),"t").unwrap();
        let mut compact=plain.clone();
        compact["turnItemsOmitLocalVisible"]=json!(true);
        compact["projection"]["turnItems"]=json!([tail]);
        compact["projection"]["visibleTurnItems"]=json!([
            {"visibility":"local","sourceThreadId":"t","item":root.clone()},
            {"visibility":"local","sourceThreadId":"t","item":child},
            {"visibility":"inherited","sourceThreadId":"ancestor","item":{"id":"foreign221","threadId":"ancestor"}}
        ]);
        let actual=parse_detail(&compact.to_string(),"t").unwrap();
        assert_eq!(actual,expected,
            "la forme compacte ne doit perdre ni message, ni outil racine, ni état");
        compact["projection"]["visibleTurnItems"][0]["item"]["threadId"]=json!("foreign");
        assert!(parse_detail(&compact.to_string(),"t").is_err());
        compact["projection"]["visibleTurnItems"][0]["item"]=root;
        compact["projection"]["visibleTurnItems"][0]["sourceThreadId"]=json!("foreign");
        assert!(parse_detail(&compact.to_string(),"t").is_err());
        compact["projection"]["visibleTurnItems"][0]["sourceThreadId"]=json!("t");
        compact["projection"]["turnItems"]=json!([compact["projection"]["visibleTurnItems"][0]["item"].clone()]);
        assert!(parse_detail(&compact.to_string(),"t").is_err(),"un même outil ne doit pas être émis deux fois");
    }

    #[test]
    fn spec221_compact_http_avoids_duplicate_payload_without_raising_limit() {
        let mut plain=detail();
        let item=json!({"id":"large221","threadId":"t","runId":"r","nodeId":"root",
            "type":"dynamic_tool","status":"completed","output":"x".repeat(9*1024*1024)});
        plain["projection"]["turnItems"]=json!([item.clone()]);
        plain["projection"]["visibleTurnItems"]=json!([
            {"visibility":"local","sourceThreadId":"t","item":item}]);
        assert!(plain.to_string().len()>16*1024*1024);
        plain["projection"]["turnItems"]=json!([]);
        plain["turnItemsOmitLocalVisible"]=json!(true);
        let body=plain.to_string();
        assert!(body.len()<16*1024*1024);
        let listener=TcpListener::bind("127.0.0.1:0").unwrap();
        let client=Client::for_base_url(&format!("http://{}",listener.local_addr().unwrap()),"fixture");
        client.set_test_protocol(true);
        let server=std::thread::spawn(move||http_reply(&listener,200,&body,
            "GET /api/orchestration/threads/t/bounded?compactTurnItems=1 "));
        let parsed=client.thread_detail("t",20).unwrap();
        let tool=parsed.activities.iter().find(|a|a.kind=="tool.progress").unwrap();
        assert_eq!(tool.payload["tool_id"],"large221");
        assert_eq!(tool.payload["details_truncated"],true);
        assert!(tool.payload.to_string().len()<16*1024);
        server.join().unwrap();
    }

    #[test]
    fn spec221_bounded_http_reads_large_official_window_without_full_history() {
        let mut source=detail();
        source["projection"]["checkpoints"]=json!([{"payload":"x".repeat(17*1024*1024)}]);
        source["payloadBudgetExceeded"]=json!(true);
        source["turnItemsOmitLocalVisible"]=json!(true);
        let expected=parse_detail(&source.to_string(),"t").unwrap();
        let body=source.to_string();
        let listener=TcpListener::bind("127.0.0.1:0").unwrap();
        let client=Client::for_base_url(&format!("http://{}",listener.local_addr().unwrap()),"fixture");
        client.set_test_protocol(true);
        let server=std::thread::spawn(move||http_reply(&listener,200,&body,
            "GET /api/orchestration/threads/t/bounded?compactTurnItems=1 "));
        let actual=client.thread_detail("t",20).unwrap();
        assert_eq!(actual,expected);
        server.join().unwrap();
    }

    pub(super) fn detail() -> Value {
        json!({"snapshotSequence":18,"projection":{
            "thread":{"id":"t"},
            "runs":[{"id":"r","threadId":"t","ordinal":1,"userMessageId":"u","rootNodeId":"root","status":"completed","requestedAt":"2026-10-09T00:00:00.000Z"}],
            "nodes":[{"id":"root","kind":"root_turn","parentNodeId":null},{"id":"child","kind":"subagent","parentNodeId":"root"}],
            "messages":[
                {"id":"u","threadId":"t","runId":"r","nodeId":null,"role":"user","text":"question","streaming":false,"createdAt":"2026-10-09T00:00:00.000Z"},
                {"id":"child-message","threadId":"t","runId":"r","nodeId":"child","role":"assistant","text":"secret sous-agent","streaming":false,"createdAt":"2026-10-09T00:00:01.000Z"},
                {"id":"a","threadId":"t","runId":"r","nodeId":"root","role":"assistant","text":"réponse exacte","streaming":false,"createdAt":"2026-10-09T00:00:02.000Z"}],
            "turnItems":[],"runtimeRequests":[],"visibleTurnItems":[]}})
    }

    #[test]
    fn user_input181_t3_response_uses_same_command_after_lost_reply() {
        let listener=TcpListener::bind("127.0.0.1:0").unwrap();
        let client=Client::for_base_url(&format!("http://{}",listener.local_addr().unwrap()),"fixture");client.set_test_protocol(true);
        let server=std::thread::spawn(move || {
            let first=rpc_reply(&listener,"orchestration.dispatchCommand",None);
            let second=rpc_reply(&listener,"orchestration.dispatchCommand",Some(json!({"sequence":42})));
            assert_eq!(first,second);first
        });
        let answers=json!({"q":["A","ma précision"]});
        assert!(client.answer_user_input("t","req181","command181",&answers).is_err());
        client.answer_user_input("t","req181","command181",&answers).unwrap();
        assert_eq!(server.join().unwrap(),json!({"type":"runtime-request.respond","commandId":"command181","threadId":"t","requestId":"req181","answers":answers}));
    }

    #[test]
    fn user_input181_t3_preserves_questions_and_external_resolution() {
        let mut value=detail();
        let questions=json!([{"id":"q","header":"Choix","question":"Lequel ?","options":[{"label":"A","description":"Exact"}],"multiSelect":false,"allowCustomAnswer":true,"initialAnswer":"Brouillon","required":true}]);
        value["projection"]["turnItems"]=json!([{"id":"ui","type":"user_input_request","threadId":"t","runId":"r","nodeId":"root","status":"pending","requestId":"req181","questions":questions}]);
        value["projection"]["runtimeRequests"]=json!([{"id":"req181","nodeId":"root","kind":"user_input","status":"pending","responseCapability":{"type":"live","providerSessionId":"sdk181"}}]);
        let parsed=parse_detail(&value.to_string(),"t").unwrap();
        let question=parsed.activities.iter().find(|a|a.kind=="user_input.request").unwrap();
        assert_eq!(question.payload["questions"],questions);
        let mut compact=value.clone();
        compact["turnItemsOmitLocalVisible"]=json!(true);
        compact["projection"]["visibleTurnItems"]=json!([{"visibility":"local","sourceThreadId":"t","item":value["projection"]["turnItems"][0].clone()}]);
        compact["projection"]["turnItems"]=json!([]);
        assert_eq!(parse_detail(&compact.to_string(),"t").unwrap(),parsed);

        value["projection"]["runtimeRequests"][0]["status"]=json!("resolved");
        value["projection"]["runtimeRequests"][0]["answers"]=json!({"q":["A"]});
        let parsed=parse_detail(&value.to_string(),"t").unwrap();
        assert_eq!(parsed.activities.iter().find(|a|a.kind=="user_input.request").unwrap().payload["request"]["status"],"resolved");
        value["projection"]["runtimeRequests"][0]["responseCapability"]=json!({"type":"not_resumable","reason":"gone"});
        assert_eq!(parse_detail(&value.to_string(),"t").unwrap().activities.iter().find(|a|a.kind=="user_input.request").unwrap().payload["request"]["responseCapability"]["type"],"not_resumable");
        value["projection"]["turnItems"][0]["threadId"]=json!("foreign");
        assert!(parse_detail(&value.to_string(),"t").is_err());
    }

    #[test]
    fn mobile177_activity_projects_only_root_details_and_distinct_states() {
        let mut raw=detail();
        raw["projection"]["turnItems"]=json!([
            {"id":"tool","threadId":"t","runId":"r","nodeId":"root","type":"dynamic_tool","status":"running","toolName":"Read","input":{"path":"/fixture/readme.md"},"output":"file contents"},
            {"id":"child-tool","threadId":"t","runId":"r","nodeId":"child","type":"dynamic_tool","status":"running","toolName":"Private child","input":"never-copy"}
        ]);
        let current=parse_detail(&raw.to_string(),"t").unwrap();
        let tools:Vec<_>=current.activities.iter().filter(|a|a.kind=="tool.progress").collect();
        assert_eq!(tools.len(),1);
        assert_eq!(tools[0].payload["tool"],"Read");
        assert_eq!(tools[0].payload["tool_id"],"tool");
        assert!(tools[0].payload["input"].as_str().unwrap().contains("/fixture/readme.md"));
        assert_eq!(tools[0].payload["output"],"file contents");
        assert!(!tools[0].payload.to_string().contains("never-copy"));
        let first=tools[0].id.clone();
        raw["projection"]["turnItems"][0]["status"]=json!("completed");
        let done=parse_detail(&raw.to_string(),"t").unwrap();
        assert_ne!(done.activities.iter().find(|a|a.kind=="tool.progress").unwrap().id,first);
    }

    #[test]
    fn mobile177_tool_detail_is_bound_and_large_unicode_output_is_explicit() {
        let item=json!({"id":"x","threadId":"t","runId":"r","nodeId":"root","type":"command_execution","status":"completed","input":"echo confirmed","output":"é".repeat(9000),"exitCode":0});
        let value=json!({"item":item});
        let fields=tool_detail(value.clone(),"t","x","r",&json!("root")).unwrap();
        assert_eq!(fields["input"],"echo confirmed");assert_eq!(fields["exit_code"],0);
        assert_eq!(fields["details_truncated"],true);assert!(fields.to_string().len()<13*1024);
        for (thread,id,run,node) in [("other","x","r","root"),("t","other","r","root"),("t","x","other","root"),("t","x","r","child")] {
            assert!(tool_detail(value.clone(),thread,id,run,&json!(node)).is_err());
        }
    }

    #[test]
    fn mobile177_tool_metadata_is_bounded_before_detail_reduction() {
        let item=json!({"id":"x".repeat(16000),"nodeId":"root","type":"command_execution","status":"completed"});
        assert!(tool_fields(&item).is_err());
        for field in ["nodeId","type","status"] {
            let mut item=json!({"id":"x","nodeId":"root","type":"command_execution","status":"completed"});
            item[field]=json!("x".repeat(16000));assert!(tool_fields(&item).is_err());
        }
    }

    #[test]
    fn mobile177_tool_detail_fits_the_actual_final_journal_entry() {
        let item=json!({"id":"\"".repeat(256),"nodeId":"root","type":"command_execution","status":"completed","input":"\"".repeat(4000),"output":"\"".repeat(5000),"exitCode":0});
        let mut fields=tool_fields(&item).unwrap();
        let activity=format!("v2-tool:{}:completed",item["id"].as_str().unwrap());
        fields["kind"]=json!("tool");fields["t3_activity_id"]=json!(activity);
        let message=format!("t3-activity:{activity}:0");
        let root=std::env::temp_dir().join(format!("mobile-tool-size-{}",uuid::Uuid::new_v4()));
        let agent=uuid::Uuid::new_v4().to_string();
        let mut journal=bridget_transport::journal::SessionJournal::new(&root,&agent,&uuid::Uuid::new_v4().to_string()).unwrap();
        journal.append("update",Some(&message),fields).unwrap();drop(journal);
        let page=crate::attach::lineage_journal_page(&root.join(&agent),&agent,0,50).unwrap();
        assert!(page["gap"].is_null());assert_eq!(page["events"].as_array().unwrap().len(),1);
        assert!(page["events"][0].to_string().len()<16*1024);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec147_v2_shell_preserves_busy_state_and_project() {
        let parsed = parse_snapshot(&shell().to_string()).unwrap();
        let thread = &parsed.threads[0];
        assert_eq!(thread.workspace_root.as_deref(), Some("/fixture/project"));
        assert_eq!(thread.latest_turn.as_ref().unwrap().turn_id, "r");
        assert_eq!(thread.relationship_to_parent.as_deref(), Some("subagent"));
        assert_eq!(
            thread.session.as_ref().unwrap().active_turn_id.as_deref(),
            Some("r")
        );
        assert_eq!(
            crate::t3code::thread_provider(thread).as_deref(),
            Some("claude")
        );
    }

    #[test]
    fn spec165_v2_relation_reelle_est_portee_et_inconnue_refusee() {
        for (value, expected) in [
            (json!("subagent"), Some("subagent")),
            (json!("fork"), Some("fork")),
            (Value::Null, None),
        ] {
            let mut fixture = shell();
            fixture["threads"][0]["lineage"]["relationshipToParent"] = value;
            assert_eq!(
                parse_snapshot(&fixture.to_string()).unwrap().threads[0]
                    .relationship_to_parent
                    .as_deref(),
                expected
            );
        }
        let mut absent = shell();
        absent["threads"][0]
            .as_object_mut()
            .unwrap()
            .remove("lineage");
        assert_eq!(
            parse_snapshot(&absent.to_string()).unwrap().threads[0].relationship_to_parent,
            None
        );
        let mut null_lineage = shell();
        null_lineage["threads"][0]["lineage"] = Value::Null;
        assert_eq!(
            parse_snapshot(&null_lineage.to_string()).unwrap().threads[0].relationship_to_parent,
            None
        );
        for invalid in [json!("future"), json!(7), json!({})] {
            let mut fixture = shell();
            fixture["threads"][0]["lineage"]["relationshipToParent"] = invalid;
            assert!(matches!(
                parse_snapshot(&fixture.to_string()),
                Err(ContractError::InvalidShape { field, .. })
                    if field == "lineage.relationshipToParent"
            ));
        }
    }

    #[test]
    fn spec147_v2_unknown_status_and_incomplete_shell_refuse() {
        for (field, value) in [("schemaVersion", json!(3)), ("schemaVersion", Value::Null)] {
            let mut shell = shell();
            shell[field] = value;
            assert!(parse_snapshot(&shell.to_string()).is_err());
        }
        let mut mismatch = shell();
        mismatch["threads"][0]["modelSelection"]["instanceId"] = json!("other");
        assert!(parse_snapshot(&mismatch.to_string()).is_err());
        for status in ["new-future-status", ""] {
            let mut value = shell();
            value["threads"][0]["status"] = json!(status);
            assert!(parse_snapshot(&value.to_string()).is_err());
        }
        let mut value = shell();
        value["threads"][0]
            .as_object_mut()
            .unwrap()
            .remove("activeRunId");
        assert!(parse_snapshot(&value.to_string()).is_err());
    }

    #[test]
    fn spec147_v2_detail_preserves_text_and_run_without_child_reply() {
        let parsed = parse_detail(&detail().to_string(), "t").unwrap();
        assert_eq!(
            parsed
                .messages
                .iter()
                .map(|m| m.id.as_str())
                .collect::<Vec<_>>(),
            ["u", "a"]
        );
        assert_eq!(parsed.messages[0].turn_id.as_deref(), Some("r"));
        assert_eq!(parsed.messages[1].text, "réponse exacte");
        assert_eq!(parsed.latest_turn.unwrap().state, "completed");
        assert!(
            parsed
                .activities
                .iter()
                .any(|a| a.kind == "t3v2.run" && a.payload["userMessageId"] == "u")
        );
    }

    #[test]
    fn spec147_v2_detail_refuses_wrong_thread_and_unknown_run_state() {
        assert!(parse_detail(&detail().to_string(), "foreign").is_err());
        let mut value = detail();
        value["projection"]["runs"][0]["status"] = json!("unknown");
        assert!(parse_detail(&value.to_string(), "t").is_err());
    }

    #[test]
    fn spec147_v2_run_end_states_are_explicit() {
        for (status, expected) in [
            ("failed", "error"),
            ("cancelled", "interrupted"),
            ("rolled_back", "interrupted"),
            ("queued", "queued"),
        ] {
            let mut value = detail();
            value["projection"]["runs"][0]["status"] = json!(status);
            assert_eq!(
                parse_detail(&value.to_string(), "t")
                    .unwrap()
                    .latest_turn
                    .unwrap()
                    .state,
                expected
            );
        }
    }
    #[test]
    fn guidance156_t3_steer_pose_delivery_intent() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let client = Client::for_base_url(
            &format!("http://{}", listener.local_addr().unwrap()),
            "fixture",
        );
        client.set_test_protocol(true);
        let server = std::thread::spawn(move || {
            rpc_reply(
                &listener,
                "orchestration.dispatchCommand",
                Some(json!({"sequence":42})),
            )
        });
        let command = json!({"type":"thread.turn.start","commandId":"guide",
            "threadId":"thread-1","message":{"messageId":"message-guide","role":"user","text":"complément"}});
        assert_eq!(
            client.dispatch(&command, Some("steer")).unwrap().sequence,
            42
        );
        let frame = server.join().unwrap();
        assert_eq!(frame["deliveryIntent"], "steer");
        assert_eq!(frame["text"], "complément");
        assert_eq!(frame["commandId"], "guide");
    }

    #[test]
    fn guidance156_t3_interrupt_pose_restart() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let client = Client::for_base_url(
            &format!("http://{}", listener.local_addr().unwrap()),
            "fixture",
        );
        client.set_test_protocol(true);
        let server = std::thread::spawn(move || {
            rpc_reply(
                &listener,
                "orchestration.dispatchCommand",
                Some(json!({"sequence":43})),
            )
        });
        let command = json!({"type":"thread.turn.start","commandId":"restart",
            "threadId":"thread-1","message":{"messageId":"message-restart","role":"user","text":"nouvelle direction"}});
        assert_eq!(
            client.dispatch(&command, Some("restart")).unwrap().sequence,
            43
        );
        let frame = server.join().unwrap();
        assert_eq!(frame["deliveryIntent"], "restart");
        assert_eq!(frame["text"], "nouvelle direction");
    }
