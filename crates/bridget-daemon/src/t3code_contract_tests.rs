    #[test]
    fn spec226_http_timeout_is_closed_and_other_io_does_not_gain_grace() {
        use std::io::{Error, ErrorKind};
        for kind in [ErrorKind::TimedOut, ErrorKind::WouldBlock] {
            assert_eq!(super::http_read_error(Error::new(kind,"PRIVATE")),super::ContractError::Transport("http_timeout".into()));
            assert_eq!(super::http_request_error(minreq::Error::IoError(Error::new(kind,"PRIVATE"))),super::ContractError::Transport("http_timeout".into()));
        }
        assert_eq!(super::http_read_error(Error::new(ErrorKind::ConnectionReset,"PRIVATE")),super::ContractError::Transport("http_response_interrupted".into()));
    }
    use super::*;

    const RUNTIME: &str = r#"{"version":1,"pid":19619,"host":"127.0.0.1","port":3773,"origin":"http://127.0.0.1:3773","startedAt":"2026-09-14T07:28:30.671Z"}"#;

    #[test]
    fn spec098_runtime_du_mode_web_sans_host_est_lu_par_son_origine() {
        // Forme réelle de `t3 --mode web --no-browser` (0.0.40) : pas de `host`.
        let web = r#"{"version":1,"pid":90922,"port":3773,"origin":"http://127.0.0.1:3773","startedAt":"2026-09-14T08:34:38.449Z"}"#;
        assert_eq!(
            parse_runtime(web, |_| true).unwrap(),
            ServerRuntime {
                port: 3773,
                pid: 90922
            }
        );
        let distant = r#"{"version":1,"pid":1,"port":3773,"origin":"http://192.168.1.9:3773"}"#;
        assert!(matches!(
            parse_runtime(distant, |_| true),
            Err(ContractError::NotLoopback(host)) if host == "192.168.1.9"
        ));
        let sans_repere = r#"{"version":1,"pid":1,"port":3773}"#;
        assert!(matches!(
            parse_runtime(sans_repere, |_| true),
            Err(ContractError::InvalidShape { field, .. }) if field == "host"
        ));
    }

    #[test]
    fn spec221_bounded_response_caps_keep_general_budget_and_utf8() {
        let size = MAX_RESPONSE_BYTES + 1;
        assert!(read_response_body(std::io::repeat(b'x').take(size), MAX_RESPONSE_BYTES).is_err());
        assert!(read_response_body(std::io::repeat(b'x').take(size),
            MAX_BOUNDED_THREAD_RESPONSE_BYTES).is_ok());
        assert!(read_response_body(std::io::repeat(b'x').take(MAX_BOUNDED_THREAD_RESPONSE_BYTES),
            MAX_BOUNDED_THREAD_RESPONSE_BYTES).is_ok());
        assert!(read_response_body(std::io::repeat(b'x').take(MAX_BOUNDED_THREAD_RESPONSE_BYTES + 1),
            MAX_BOUNDED_THREAD_RESPONSE_BYTES).is_err());
        assert!(read_response_body(&[0xff_u8][..], MAX_BOUNDED_THREAD_RESPONSE_BYTES).is_err());
    }

    #[test]
    fn runtime_valide_reconstruit_la_boucle_locale_et_refuse_le_reste() {
        let runtime = parse_runtime(RUNTIME, |_| true).unwrap();
        assert_eq!(
            runtime,
            ServerRuntime {
                port: 3773,
                pid: 19619
            }
        );
        assert_eq!(runtime.base_url(), "http://127.0.0.1:3773");
        assert_eq!(
            parse_runtime(RUNTIME, |_| false),
            Err(ContractError::StaleRuntime(19619))
        );
        let distant = RUNTIME.replace("\"host\":\"127.0.0.1\"", "\"host\":\"0.0.0.0\"");
        assert_eq!(
            parse_runtime(&distant, |_| true),
            Err(ContractError::NotLoopback("0.0.0.0".into()))
        );
        assert!(
            matches!(parse_runtime("{\"host\":\"127.0.0.1\"}", |_| true), Err(ContractError::InvalidShape { field, .. }) if field == "port")
        );
        assert!(matches!(
            parse_runtime("pas du json", |_| true),
            Err(ContractError::InvalidShape { .. })
        ));
    }

    #[test]
    fn snapshot_reel_0_0_40_est_lu_et_les_champs_manquants_sont_nommes() {
        let text = r#"{"snapshotSequence":374,"projects":[{"id":"p1","workspaceRoot":"/tmp/p1"}],"threads":[
          {"id":"t1","projectId":"p1","title":"Sonde","worktreePath":null,"archivedAt":null,"deletedAt":null,"updatedAt":"2026-09-14T07:31:16Z",
           "latestTurn":{"turnId":"turn-1","state":"completed","assistantMessageId":"assistant:a1"},
           "session":{"threadId":"t1","status":"ready","providerName":"claudeAgent","providerInstanceId":"claudeAgent","activeTurnId":null}},
          {"id":"t2","projectId":"p1","title":"Archivé","updatedAt":"2026-09-14T07:00:00Z","archivedAt":"2026-09-14T07:32:00Z","latestTurn":null,"session":null}
        ],"updatedAt":"x"}"#;
        let snapshot = parse_snapshot(text).unwrap();
        assert_eq!(snapshot.snapshot_sequence, 374);
        assert_eq!(snapshot.projects[0].workspace_root, "/tmp/p1");
        let live = &snapshot.threads[0];
        assert_eq!(live.workspace_root.as_deref(), Some("/tmp/p1"));
        assert_eq!(live.relationship_to_parent, None);
        assert!(live.is_live());
        assert_eq!(live.session.as_ref().unwrap().provider_name, "claudeAgent");
        assert_eq!(
            live.latest_turn
                .as_ref()
                .unwrap()
                .assistant_message_id
                .as_deref(),
            Some("assistant:a1")
        );
        assert!(!snapshot.threads[1].is_live());
        assert_ne!(live.change_key(), snapshot.threads[1].change_key());
        let sans_id = text.replace("\"id\":\"t1\",", "");
        assert!(
            matches!(parse_snapshot(&sans_id), Err(ContractError::InvalidShape { field, .. }) if field == "threads[].id")
        );
    }

    #[test]
    fn spec138_snapshot_uses_project_id_not_worktree_or_title() {
        let value = serde_json::json!({
            "snapshotSequence": 1,
            "projects": [
                {"id": "a", "workspaceRoot": "/tmp/project-a"},
                {"id": "b", "workspaceRoot": "/tmp/project-b"}
            ],
            "threads": [{"id": "thread-a", "projectId": "a", "title": "project-b",
                "worktreePath": "/tmp/project-b/foreign-worktree", "updatedAt": "date"}]
        });
        let snapshot = parse_snapshot(&value.to_string()).unwrap();
        assert_eq!(
            snapshot.threads[0].workspace_root.as_deref(),
            Some("/tmp/project-a")
        );
        assert_eq!(
            snapshot.threads[0].worktree_path.as_deref(),
            Some("/tmp/project-b/foreign-worktree")
        );
    }

    #[test]
    fn spec138_snapshot_missing_or_ambiguous_project_stays_unknown() {
        for projects in [
            serde_json::json!([]),
            serde_json::json!([
                {"id": "a", "workspaceRoot": "/tmp/project-a"},
                {"id": "a", "workspaceRoot": "/tmp/project-b"}
            ]),
        ] {
            let value = serde_json::json!({"snapshotSequence": 1, "projects": projects,
                "threads": [{"id": "thread-a", "projectId": "a", "worktreePath": "/tmp/project-a", "updatedAt": "date"}]});
            assert_eq!(
                parse_snapshot(&value.to_string()).unwrap().threads[0].workspace_root,
                None
            );
        }
    }

    #[test]
    fn detail_reel_distingue_utilisateur_et_tours_assistant() {
        let text = r#"{"snapshotSequence":1,"thread":{"messages":[
          {"id":"u1","role":"user","text":"PONG ?","streaming":false,"turnId":null},
          {"id":"assistant:a1","role":"assistant","text":"PONG-098","streaming":false,"turnId":"turn-1"}
        ],"latestTurn":{"turnId":"turn-1","state":"completed","assistantMessageId":"assistant:a1"}},"page":{}}"#;
        let detail = parse_thread_detail(text).unwrap();
        assert_eq!(detail.messages.len(), 2);
        assert_eq!(detail.messages[0].turn_id, None);
        assert_eq!(detail.messages[1].turn_id.as_deref(), Some("turn-1"));
        assert!(
            matches!(parse_thread_detail(r#"{"thread":{"messages":[{"id":"x","role":"user"}]}}"#), Err(ContractError::InvalidShape { field, .. }) if field == "messages[].streaming")
        );
    }

    #[test]
    fn commande_de_tour_porte_les_identifiants_du_pont() {
        let command = turn_start_command(
            "t1",
            "delivery-1",
            "message-1",
            "Bonjour",
            "approval-required",
            "default",
        );
        assert_eq!(command["type"], "thread.turn.start");
        assert_eq!(command["commandId"], "delivery-1");
        assert_eq!(command["message"]["messageId"], "message-1");
        assert_eq!(command["message"]["role"], "user");
        assert!(command["createdAt"].as_str().unwrap().ends_with('Z'));
        // La politique du fil est répétée, jamais choisie par le pont : la
        // route HTTP l'exige et un « full-access » imposé serait une élévation.
        assert_eq!(command["runtimeMode"], "approval-required");
        assert_eq!(command["interactionMode"], "default");
    }

    #[test]
    fn session_emise_et_liste_sont_lues_par_libelle() {
        let issued = parse_issued_session(r#"{"sessionId":"s1","token":"eyJ…","method":"bearer-access-token","scopes":["a"],"subject":"bridget","client":{"label":"bridget-abc","deviceType":"bot"},"expiresAt":"2026-09-14T09:29:28Z"}"#).unwrap();
        assert_eq!(
            (issued.session_id.as_str(), issued.label.as_str()),
            ("s1", "bridget-abc")
        );
        assert!(
            matches!(parse_issued_session(r#"{"sessionId":"s1"}"#), Err(ContractError::InvalidShape { field, .. }) if field == "token")
        );
        let list = parse_session_list(r#"[{"sessionId":"s1","client":{"label":"bridget-abc"}},{"sessionId":"s2","client":{}}]"#).unwrap();
        assert_eq!(
            list,
            vec![
                ("s1".into(), Some("bridget-abc".into())),
                ("s2".into(), None)
            ]
        );
    }

    #[test]
    fn iso_now_est_une_date_utc_plausible() {
        let now = iso_now();
        assert_eq!(now.len(), 24, "{now}");
        assert!(now.starts_with("20") && now.ends_with('Z'));
    }
