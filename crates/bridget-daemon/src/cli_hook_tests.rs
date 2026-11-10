    use super::*;
    use bridget_transport::protocol::LedgerDeliveryStatus;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::UnixListener;
    use std::path::{Path, PathBuf};
    use std::thread;

    fn argv(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn delegation_grant_requires_explicit_root_and_closed_options() {
        use bridget_transport::protocol::SpawnPosture;
        assert_eq!(
            parse_delegate_grant(&argv(&[
                "Regional",
                "--cwd",
                "/tmp/project",
                "--posture",
                "development",
                "--revoke"
            ]))
            .unwrap(),
            (
                "Regional".into(),
                "/tmp/project".into(),
                SpawnPosture::Development,
                true
            )
        );
        for args in [
            argv(&[]),
            argv(&["agent"]),
            argv(&["agent", "--cwd", "relative", "--posture", "development"]),
            argv(&["agent", "--cwd", "/tmp", "--posture", "full-access"]),
            argv(&[
                "agent",
                "--cwd",
                "/tmp",
                "--posture",
                "discovery",
                "--cwd",
                "/other",
            ]),
            argv(&[
                "agent",
                "--cwd",
                "/tmp",
                "--posture",
                "discovery",
                "--revoke",
                "--revoke",
            ]),
            argv(&[
                "agent",
                "--cwd",
                "/tmp",
                "--posture",
                "discovery",
                "--owner",
                "other",
            ]),
        ] {
            assert!(parse_delegate_grant(&args).is_err(), "args={args:?}");
        }
    }

    #[test]
    fn spec163_auto_delegate_cli_conserve_le_payload_ferme_du_handler_mcp() {
        let payload = parse_auto_delegate_args(&argv(&[
            "--request-id",
            "auto-163",
            "--task",
            "admettre une mission",
            "--cwd",
            "/workspace",
            "--agent-type",
            "codex",
            "--model",
            "gpt-5.6",
            "--effort",
            "high",
            "--posture",
            "development",
            "--reply-timeout",
            "21600",
            "--workspace",
            r#"{"strategy":"shared"}"#,
            "--keep-alive-secs",
            "600",
            "--child-requests",
            "deny",
            "--brief",
            r#"{"objective":"objectif","summary":"résumé"}"#,
            "--urgent",
            "true",
            "--deliverable",
            "rapport",
        ]))
        .unwrap();
        assert_eq!(
            payload,
            serde_json::json!({
                "request_id":"auto-163", "task":"admettre une mission", "cwd":"/workspace",
                "agent_type":"codex", "model":"gpt-5.6", "effort":"high",
                "posture":"development", "reply_timeout":21600,
                "workspace":{"strategy":"shared"}, "keep_alive_secs":600,
                "child_requests":"deny", "brief":{"objective":"objectif","summary":"résumé"},
                "urgent":true, "deliverable":"rapport"
            })
        );
        for args in [
            argv(&["--request-id", "r", "--task", "t"]),
            argv(&[
                "--request-id",
                "r",
                "--task",
                "t",
                "--task",
                "autre",
                "--cwd",
                "/workspace",
            ]),
            argv(&[
                "--request-id",
                "r",
                "--task",
                "t",
                "--cwd",
                "/workspace",
                "--workspace",
                "not-json",
            ]),
            argv(&[
                "--request-id",
                "r",
                "--task",
                "t",
                "--cwd",
                "/workspace",
                "--urgent",
                "yes",
            ]),
            argv(&[
                "--request-id",
                "r",
                "--task",
                "t",
                "--cwd",
                "/workspace",
                "--parent",
                "forged",
            ]),
        ] {
            assert!(parse_auto_delegate_args(&args).is_err(), "args={args:?}");
        }
        let task_trop_long = vec![
            "--request-id".into(),
            "r".into(),
            "--task".into(),
            "x".repeat(65_537),
            "--cwd".into(),
            "/workspace".into(),
        ];
        assert!(parse_auto_delegate_args(&task_trop_long).is_err());
        let canon_trop_long = vec![
            "--request-id".into(),
            "r".into(),
            "--task".into(),
            "t".into(),
            "--cwd".into(),
            "/workspace".into(),
            "--brief".into(),
            format!(r#"{{"objective":"{}"}}"#, "x".repeat(512 * 1024)),
        ];
        assert!(parse_auto_delegate_args(&canon_trop_long).is_err());
        assert!(command_requires_principal("auto-delegate"));
    }

    #[test]
    fn spec167_delegate_reduit_le_parcours_nominal_a_la_mission() {
        let cwd = std::env::temp_dir().join(format!(
            "bridget-cli167-nominal-{}",
            uuid::Uuid::new_v4().simple()
        ));
        bridget_transport::fsutil::create_private_dir(&cwd).unwrap();
        let parsed = parse_delegate_args(&argv(&["analyser le dépôt"]), &cwd, || {
            "request-167".into()
        })
        .unwrap();
        assert_eq!(parsed.tool, "bridget_auto_delegate");
        assert!(!parsed.json);
        assert_eq!(parsed.request_id, "request-167");
        assert_eq!(
            parsed.payload,
            serde_json::json!({
                "request_id":"request-167",
                "task":"analyser le dépôt",
                "cwd":cwd.to_string_lossy()
            })
        );
        std::fs::remove_dir(&cwd).unwrap();
    }

    #[test]
    fn spec167_delegate_conserve_les_choix_explicites_sans_defaults_caches() {
        let cwd = std::env::temp_dir().join(format!(
            "bridget-cli167-explicite-{}",
            uuid::Uuid::new_v4().simple()
        ));
        bridget_transport::fsutil::create_private_dir(&cwd).unwrap();
        let parsed = parse_delegate_args(
            &argv(&[
                "réviser le plan",
                "--request-id",
                "stable-167",
                "--cwd",
                cwd.to_str().unwrap(),
                "--agent-type",
                "claude",
                "--model",
                "sonnet",
                "--effort",
                "high",
                "--posture",
                "development",
                "--machine-id",
                "worker-1",
                "--project-id",
                "project-1",
                "--reply-timeout",
                "900",
                "--keep-alive-secs",
                "600",
                "--child-requests",
                "forward",
                "--workspace",
                r#"{"strategy":"shared"}"#,
                "--brief",
                r#"{"objective":"revoir"}"#,
                "--urgent",
                "--deliverable",
                "rapport",
                "--json",
            ]),
            Path::new("/ignored"),
            || panic!("une clé explicite ne doit pas être remplacée"),
        )
        .unwrap();
        assert!(parsed.json);
        assert_eq!(parsed.request_id, "stable-167");
        assert_eq!(
            parsed.payload,
            serde_json::json!({
                "request_id":"stable-167", "task":"réviser le plan", "cwd":cwd.to_string_lossy(),
                "agent_type":"claude", "model":"sonnet", "effort":"high",
                "posture":"development", "machine_id":"worker-1", "project_id":"project-1",
                "reply_timeout":900, "keep_alive_secs":600, "child_requests":"forward",
                "workspace":{"strategy":"shared"}, "brief":{"objective":"revoir"},
                "urgent":true, "deliverable":"rapport"
            })
        );
        for invalid in [
            argv(&[]),
            argv(&["--json"]),
            argv(&["mission", "surplus"]),
            argv(&["mission", "--urgent", "--urgent"]),
            argv(&["mission", "--json", "--json"]),
            argv(&["mission", "--task", "forgée"]),
        ] {
            assert!(
                parse_delegate_args(&invalid, Path::new("/workspace"), || "id".into()).is_err(),
                "args={invalid:?}"
            );
        }
        std::fs::remove_dir(&cwd).unwrap();
    }

    #[test]
    fn spec167_task_traduit_le_cycle_de_vie_vers_les_outils_natifs() {
        let cases = [
            (argv(&["read-file","task-1","sub/file.rs"]),"bridget_read_commit_file",
                serde_json::json!({"task_id":"task-1","path":"sub/file.rs"}),false),
            (argv(&["read-file","sub/file.rs"]),"bridget_read_commit_file",
                serde_json::json!({"path":"sub/file.rs"}),false),
            (
                argv(&["repair-workspace","task-1","/srv/projet/task-1","--expected-head",
                    "1111111111111111111111111111111111111111","--request-id","repair-1"]),
                "bridget_task_repair_workspace",
                serde_json::json!({"task_id":"task-1","target_cwd":"/srv/projet/task-1",
                    "expected_head":"1".repeat(40),"request_id":"repair-1"}),
                false,
            ),

            (
                argv(&["status", "task-1", "--json"]),
                "bridget_task_status",
                serde_json::json!({"task_id":"task-1"}),
                true,
            ),
            (
                argv(&["wait", "task-1", "task-2", "--timeout", "30"]),
                "bridget_task_wait",
                serde_json::json!({"task_ids":["task-1","task-2"],"wait_timeout":30}),
                false,
            ),
            (
                argv(&["ack", "task-1", "task-2"]),
                "bridget_task_ack",
                serde_json::json!({"task_ids":["task-1","task-2"]}),
                false,
            ),
            (
                argv(&["cancel", "task-1"]),
                "bridget_task_cancel",
                serde_json::json!({"task_id":"task-1"}),
                false,
            ),
            (
                argv(&[
                    "continue",
                    "task-1",
                    "approfondir",
                    "--request-id",
                    "round-2",
                ]),
                "bridget_task_continue",
                serde_json::json!({"task_id":"task-1","request_id":"round-2","task":"approfondir"}),
                false,
            ),
            (
                argv(&[
                    "respond",
                    "task-1",
                    "question-1",
                    "accept",
                    "--answer",
                    r#"{"q1":["oui"]}"#,
                ]),
                "bridget_task_respond",
                serde_json::json!({"task_id":"task-1","request_id":"question-1","decision":"accept","answer":{"q1":["oui"]}}),
                false,
            ),
        ];
        for (args, tool, payload, json) in cases {
            let TaskCliAction::Native(parsed) =
                parse_task_args(&args, || "generated-167".into()).unwrap()
            else {
                panic!("action native attendue pour {args:?}");
            };
            assert_eq!(parsed.tool, tool);
            assert_eq!(parsed.payload, payload);
            assert_eq!(parsed.json, json);
        }
        for invalid in [
            argv(&["repair-workspace","task-1","/srv/projet/task-1"]),
            argv(&["repair-workspace","task-1","/srv/projet/task-1","--force"]),

            argv(&[]),
            argv(&["status"]),
            argv(&["wait", "task-1", "task-1"]),
            argv(&["wait", "task-1", "--timeout"]),
            argv(&["continue", "task-1"]),
            argv(&["respond", "task-1", "question-1", "maybe"]),
            argv(&["ack"]),
            argv(&["cancel", "task-1", "surplus"]),
        ] {
            assert!(
                parse_task_args(&invalid, || "generated-167".into()).is_err(),
                "args={invalid:?}"
            );
        }
    }

    #[test]
    fn spec167_task_list_reutilise_la_projection_humaine_globale() {
        let TaskCliAction::List(args) = parse_task_args(
            &argv(&["list", "--state", "all", "--limit", "12", "--json"]),
            || "unused".into(),
        )
        .unwrap() else {
            panic!("liste humaine attendue");
        };
        assert_eq!(
            args,
            argv(&[
                "delegations",
                "list",
                "--state",
                "all",
                "--limit",
                "12",
                "--json"
            ])
        );
    }

    #[test]
    fn spec167_sorties_et_refus_restent_exploitables() {
        let result = serde_json::json!({
            "version":1, "task_id":"task-167", "status":"working",
            "work_state":"working", "child_agent_id":"agent-167",
            "request_id":"request-167", "continuable":true,
            "pending_requests":[{"request_id":"question-167"}]
        });
        assert_eq!(render_native_cli_result(&result, true), result.to_string());
        let human = render_native_cli_result(&result, false);
        assert!(human.contains("task-167"));
        assert!(human.contains("working"));
        assert!(human.contains("agent-167"));
        assert!(human.contains("request-167"));
        assert!(human.contains("task continue"));
        assert!(human.contains("task respond"));

        let error = coordination_identity_error(
            "delegate",
            &crate::mcp_identity::IdentityError::IdentityNotFound,
        );
        assert!(error.contains("identity_not_found"));
        assert!(error.contains("bridget codex"));
        assert!(error.contains("aucune tâche"));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&render_coordination_error(
                "identity_not_found",
                "aucune tâche créée",
                Some("request-167")
            ))
            .unwrap(),
            serde_json::json!({
                "status":"error", "code":"identity_not_found", "message":"aucune tâche créée",
                "request_id":"request-167"
            })
        );
    }

    #[test]
    fn reaper_exige_une_racine_explicitement_choisie() {
        assert!(
            require_explicit_reaper_tmp(None)
                .unwrap_err()
                .contains("--tmp DIR est obligatoire")
        );
        assert_eq!(
            require_explicit_reaper_tmp(Some(PathBuf::from("/tmp"))).unwrap(),
            PathBuf::from("/tmp")
        );
    }

    #[test]
    fn commandes_sans_arguments_refusent_le_surplus_qu_elles_recevront_en_production() {
        for command in [
            "mcp",
            "discover",
            "status",
            "version",
            "--version",
            "-v",
            "help",
            "--help",
            "-h",
        ] {
            let error = validate_zero_arity_command(command, &argv(&["SURPLUS"]))
                .expect_err("le surplus doit être refusé");
            assert!(error.contains(command), "commande absente de {error}");
            assert!(error.contains("SURPLUS"), "argument absent de {error}");
            assert!(validate_zero_arity_command(command, &[]).is_ok());
        }
        assert!(
            validate_zero_arity_command("daemon", &argv(&["--project-root-policy", "/tmp/policy"]))
                .is_ok()
        );
        // `ledger` a quitté cette grammaire en recevant `--limit` : il ne doit
        // plus être refusé en amont, sinon l'option n'atteindrait jamais son
        // parseur. Le refus du surplus lui reste dû, mais par `parse_ledger_args`.
        assert!(validate_zero_arity_command("ledger", &argv(&["--limit", "5"])).is_ok());
        assert!(parse_ledger_args(&argv(&["SURPLUS"])).is_err());
    }

    #[test]
    fn parseurs_structures_refusent_tous_le_premier_surplus() {
        let errors = [
            parse_cancel_args(&argv(&["--SURPLUS"])).unwrap_err(),
            parse_cancel_args(&argv(&["demande", "SURPLUS"])).unwrap_err(),
            parse_hook_args(&argv(&["claude-runtime", "SURPLUS"])).unwrap_err(),
            parse_install_hooks_args(&argv(&["--remove", "SURPLUS"])).unwrap_err(),
            parse_domain_args(&argv(&["revue", "SURPLUS"])).unwrap_err(),
            parse_dnd_args(&argv(&["off", "SURPLUS"])).unwrap_err(),
            parse_directory_args("agents", &argv(&["--json", "SURPLUS"]), true).unwrap_err(),
            parse_directory_args("who", &argv(&["--domain", "revue", "SURPLUS"]), false)
                .unwrap_err(),
        ];
        for error in errors {
            assert!(error.contains("SURPLUS"), "argument non nommé dans {error}");
        }
    }

    #[test]
    fn parseurs_structures_conservent_les_formes_documentees_et_le_hook_fail_soft() {
        assert_eq!(
            parse_cancel_args(&argv(&["demande", "--reason", "plus utile"])).unwrap(),
            CancelArgs {
                id: "demande".to_string(),
                reason: Some("plus utile".to_string()),
            }
        );
        assert_eq!(
            parse_hook_args(&argv(&["claude-runtime"])).unwrap(),
            HookCommand::ClaudeRuntime
        );
        assert_eq!(
            parse_hook_args(&argv(&["extension-future", "payload-opaque"])).unwrap(),
            HookCommand::Unknown("extension-future".to_string())
        );
        assert!(!parse_install_hooks_args(&[]).unwrap());
        assert!(parse_install_hooks_args(&argv(&["--remove"])).unwrap());
        assert_eq!(
            parse_domain_args(&argv(&["revue-croisee"])).unwrap(),
            Some("revue-croisee".to_string())
        );
        assert_eq!(parse_domain_args(&argv(&["--reset"])).unwrap(), None);
        assert_eq!(parse_dnd_args(&[]).unwrap(), DndArgs::Enable(None));
        assert_eq!(parse_dnd_args(&argv(&["off"])).unwrap(), DndArgs::Disable);
        assert_eq!(
            parse_dnd_args(&argv(&["--duration", "15m"])).unwrap(),
            DndArgs::Enable(Some(Duration::from_secs(15 * 60)))
        );
        assert_eq!(
            parse_directory_args("agents", &argv(&["--domain", "revue", "--json"]), true).unwrap(),
            DirectoryArgs {
                json: true,
                domain: Some("revue".to_string()),
                global: false,
                project_root: None,
            }
        );
    }

    #[test]
    fn spec138_directory_accepts_explicit_global_and_project_root() {
        assert!(
            parse_directory_args(
                "agents",
                &argv(&["--json", "--global", "--project-root", "/tmp/project"]),
                true
            )
            .is_ok()
        );
        assert!(parse_directory_args("who", &argv(&["--global"]), false).is_ok());
        assert!(parse_directory_args("who", &argv(&["--global", "--global"]), false).is_err());
        assert!(parse_directory_args("agents", &argv(&["--project-root"]), true).is_err());
    }

    #[test]
    fn spec116_refus_sans_identite_dit_pourquoi() {
        use crate::mcp_identity::IdentityError;
        let raison =
            "identité expéditeur non attestée : --from ne permet pas d'emprunter une identité";
        let indice = unattested_hint("human", raison, Err(IdentityError::IdentityNotFound))
            .expect("un refus d'identité est expliqué");
        assert!(indice.contains("identity_not_found"));
        assert!(indice.contains("rend son résultat à son parent Bridget"));
        assert!(indice.contains("lancez l'agent avec bridget codex ou bridget claude"));
        assert!(
            unattested_hint(
                "human",
                "agent introuvable: x",
                Err(IdentityError::IdentityNotFound)
            )
            .is_none(),
            "un autre refus n'est pas maquillé en problème d'identité"
        );
        assert!(
            unattested_hint(
                "127bccff-8490-453a-8182-884b749ec41e",
                raison,
                Err(IdentityError::IdentityNotFound)
            )
            .is_none(),
            "un envoi signé n'a pas besoin de cet indice"
        );
    }

    #[test]
    fn spec133_cli_sensible_refuse_le_contexte_enfant_avant_dispatch() {
        use crate::mcp_identity::IdentityError;

        for command in [
            "send",
            "reply",
            "who",
            "journal",
            "events",
            "thread",
            "handoff",
            "artifact",
            "cancel",
            "rename",
            "runtime",
            "domain",
            "control",
            "dnd",
            "guichet",
            "ledger",
            "spawn",
            "stop",
            "relaunch",
            "auto-delegate",
            "delegate",
            "task",
        ] {
            assert!(command_requires_principal(command), "{command}");
            assert_eq!(
                delegated_cli_guard(command, Err(IdentityError::DelegatedMcpOnly)),
                Err(IdentityError::DelegatedMcpOnly),
                "{command} doit être refusé avant son handler"
            );
        }
        for command in ["mcp", "status", "version", "help"] {
            assert!(!command_requires_principal(command), "{command}");
            assert_eq!(
                delegated_cli_guard(command, Err(IdentityError::DelegatedMcpOnly)),
                Ok(()),
                "{command} ne porte pas l'autorité d'un agent"
            );
        }
    }

    #[test]
    fn repli_cli_n_accepte_que_l_identifiant_uuid_dans_l_environnement() {
        let first = "550e8400-e29b-41d4-a716-446655440000";
        let second = "550e8400-e29b-41d4-a716-446655440001";
        assert_eq!(resolve_cli_agent_id(None, Some(first), || None), first);
        assert_eq!(
            resolve_cli_agent_id(Some("  "), Some(second), || None),
            second
        );
        assert_eq!(
            resolve_cli_agent_id(Some(first), Some(second), || None),
            first
        );
        assert_eq!(resolve_cli_agent_id(None, None, || None), "human");
        assert_eq!(
            resolve_cli_agent_id(None, Some("legacy-name"), || None),
            "human"
        );
    }

    /// Session 115 : un fil T3 n'a ni fichier ni variable d'identité, mais sa
    /// filiation est attestée par le pont. Il signe désormais sous son identité
    /// au lieu de « human », que sa propre connexion attestée ne peut usurper.
    #[test]
    fn spec115_fil_t3_signe_par_sa_filiation_attestee() {
        let fil = "127bccff-8490-453a-8182-884b749ec41e";
        let lanceur = "550e8400-e29b-41d4-a716-446655440000";
        assert_eq!(
            resolve_cli_agent_id(None, None, || Some(fil.to_string())),
            fil,
            "la filiation attestée signe le message"
        );
        assert_eq!(
            resolve_cli_agent_id(None, None, || Some(format!("  {fil}\n"))),
            fil,
            "le contenu du fichier de nom est normalisé"
        );
        assert_eq!(
            resolve_cli_agent_id(None, None, || Some("nom-libre".to_string())),
            "human",
            "une filiation qui ne livre pas un UUID n'invente rien"
        );
        // Un agent lancé par Bridget garde sa preuve habituelle, et la
        // filiation n'est même pas consultée.
        assert_eq!(
            resolve_cli_agent_id(None, Some(lanceur), || panic!("filiation consultée")),
            lanceur
        );
    }

    #[test]
    fn refus_type_inconnu_explique_l_instantane_du_daemon() {
        let rendered = display_spawn_refusal(&SpawnRefusal::UnknownType {
            requested_type: "cursor".to_string(),
            known_types: vec!["claude".to_string(), "codex".to_string()],
            registry: "/Users/test/.config/bridget/agents.json".to_string(),
        });

        assert_eq!(
            rendered,
            "type d'agent inconnu 'cursor'. Types connus du daemon : claude, codex. \
             Le registre /Users/test/.config/bridget/agents.json est lu au démarrage du daemon ; après modification, relancez-le."
        );
    }

    #[test]
    fn refus_capacite_nomme_type_modele_et_capacite_absente() {
        assert_eq!(
            display_spawn_refusal(&SpawnRefusal::UnsupportedCapability {
                agent_type: "codex".to_string(),
                model: "gpt-5.6-terra".to_string(),
                capability: "modèle pris en charge par l'adaptateur".to_string(),
            }),
            "lancement refusé pour le type 'codex', modèle 'gpt-5.6-terra': capacité manquante modèle pris en charge par l'adaptateur"
        );
    }

    #[test]
    fn rendu_ledger_cli_reste_octet_pour_octet_stable() {
        let entries = vec![
            LedgerMessage {
                id: "ancien".to_string(),
                ts: 1,
                sender: "alice".to_string(),
                target: "bob".to_string(),
                body: "premier".to_string(),
                delivery_status: None,
            },
            LedgerMessage {
                id: "recent".to_string(),
                ts: 2,
                sender: "bob".to_string(),
                target: "alice".to_string(),
                body: "corps riche $VAR\nintact".to_string(),
                delivery_status: Some(LedgerDeliveryStatus::Recu),
            },
        ];
        assert_eq!(
            render_ledger(&entries),
            "Derniers 2 messages :\n  [2] bob → alice [reçu]: corps riche $VAR\nintact\n  [1] alice → bob: premier\n"
        );
    }

    #[test]
    fn rendu_ledger_compte_une_ligne_par_message_apres_refus_d_un_nom_avec_lf() {
        let mut router = bridget_core::Router::new();
        router
            .register(
                "550e8400-e29b-41d4-a716-446655440000",
                &bridget_core::AgentType::Codex,
                "conn-1",
            )
            .unwrap();
        let _ = router.register(
            "invalid-agent-id",
            &bridget_core::AgentType::Codex,
            "conn-2",
        );
        let entries = router
            .list_agents()
            .into_iter()
            .map(|agent| LedgerMessage {
                id: format!("message-{}", agent.connection_id),
                ts: 42,
                sender: agent.agent_id.clone(),
                target: "victime".to_string(),
                body: "corps".to_string(),
                delivery_status: None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            render_ledger(&entries).lines().count(),
            2,
            "l'en-tête et l'unique message légitime doivent former exactement deux lignes"
        );
    }

    #[test]
    fn rendu_ledger_distingue_en_vol_et_recu() {
        let en_vol = LedgerMessage {
            id: "a".into(),
            ts: 10,
            sender: "peer-a".into(),
            target: "peer-b".into(),
            body: "collège".into(),
            delivery_status: Some(LedgerDeliveryStatus::EnVol),
        };
        let recu = LedgerMessage {
            id: "b".into(),
            ts: 11,
            sender: "peer-a".into(),
            target: "peer-b".into(),
            body: "collège".into(),
            delivery_status: Some(LedgerDeliveryStatus::Recu),
        };
        let rendered_vol = render_ledger(&[en_vol]);
        let rendered_recu = render_ledger(&[recu]);
        assert_ne!(
            rendered_vol, rendered_recu,
            "dispatching et acked ne doivent pas se rendre pareil"
        );
        assert!(rendered_vol.contains("[en vol]"));
        assert!(rendered_recu.contains("[reçu]"));
        assert!(!rendered_vol.contains("[reçu]"));
        assert!(!rendered_recu.contains("[en vol]"));
    }

    #[test]
    fn requests_defaut_reste_participant_et_all_expose_de_vers() {
        assert_eq!(
            parse_requests_args(&[]).unwrap(),
            RequestsOptions {
                all: false,
                json: false
            }
        );
        assert_eq!(
            parse_requests_args(&["--all".into(), "--json".into()]).unwrap(),
            RequestsOptions {
                all: true,
                json: true
            }
        );
        assert!(parse_requests_args(&["--global".into()]).is_err());

        let requests = vec![
            RequestInfo {
                id: "req-a".to_string(),
                sender: "guichet".to_string(),
                target: "coderBridget".to_string(),
                state: "open".to_string(),
                created_at: 10,
                deadline_at: 100,
                cancel_reason: None,
                deferred_reminder_level: None,
                deferred_reminder_at: None,
            },
            RequestInfo {
                id: "req-b".to_string(),
                sender: "prospective".to_string(),
                target: "reviewer2".to_string(),
                state: "open".to_string(),
                created_at: 11,
                deadline_at: 200,
                cancel_reason: None,
                deferred_reminder_level: Some(1),
                deferred_reminder_at: Some(50),
            },
        ];

        let participant = render_requests(&requests, false);
        let participant_header = participant.lines().next().unwrap();
        assert!(participant_header.contains("DESTINATAIRE"), "{participant}");
        assert!(
            !participant_header
                .split_whitespace()
                .any(|cell| cell == "DE")
        );
        assert!(
            !participant_header
                .split_whitespace()
                .any(|cell| cell == "VERS")
        );
        assert!(participant.contains("coderBridget"));
        assert!(!participant.contains("guichet"));
        assert!(!participant.contains("prospective"));

        let global = render_requests(&requests, true);
        let global_header = global.lines().next().unwrap();
        assert!(global_header.split_whitespace().any(|cell| cell == "DE"));
        assert!(global_header.split_whitespace().any(|cell| cell == "VERS"));
        assert!(!global_header.contains("DESTINATAIRE"));
        assert!(global.contains("guichet") && global.contains("coderBridget"));
        assert!(global.contains("prospective") && global.contains("reviewer2"));
        assert!(global.contains("100") && global.contains("open"));
    }

    /// Configuration réaliste : quatre hooks utilisateur déjà en place, dont
    /// un sur `Stop`. L'insertion doit être additive, jamais destructive.
    fn settings_utilisateur() -> serde_json::Value {
        serde_json::json!({
            "model": "opus",
            "hooks": {
                "UserPromptSubmit": [{"hooks": [{"type": "command", "command": "attention.sh working"}]}],
                "Stop": [{"hooks": [{"type": "command", "command": "attention.sh mark"}]}],
                "PostToolUse": [{"matcher": "Edit|Write", "hooks": [{"type": "command", "command": "auto-commit.sh"}]}],
                "SessionEnd": [{"hooks": [{"type": "command", "command": "session-sync.sh"}]}]
            }
        })
    }

    #[test]
    fn installation_additive_puis_retrait_restaure_l_original() {
        let original = settings_utilisateur();
        let mut settings = original.clone();

        assert!(insert_bridget_hook(&mut settings));
        let stop = settings["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop.len(), 2, "le hook utilisateur doit être préservé");
        assert_eq!(
            stop[0]["hooks"][0]["command"].as_str(),
            Some("attention.sh mark")
        );
        // Les autres événements sont intacts.
        assert_eq!(
            settings["hooks"]["SessionEnd"],
            original["hooks"]["SessionEnd"]
        );
        assert_eq!(settings["model"], original["model"]);

        assert!(remove_bridget_hook(&mut settings));
        assert_eq!(
            settings, original,
            "le retrait doit rendre le fichier d'origine"
        );
    }

    #[test]
    fn installation_est_idempotente() {
        let mut settings = settings_utilisateur();
        assert!(insert_bridget_hook(&mut settings));
        assert!(
            !insert_bridget_hook(&mut settings),
            "une seconde installation ne doit rien changer"
        );
        assert_eq!(settings["hooks"]["Stop"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn retrait_sans_installation_prealable_ne_change_rien() {
        let mut settings = settings_utilisateur();
        assert!(!remove_bridget_hook(&mut settings));
        assert_eq!(settings, settings_utilisateur());
    }

    #[test]
    fn installation_cree_la_section_hooks_absente() {
        let mut settings = serde_json::json!({"model": "opus"});
        assert!(insert_bridget_hook(&mut settings));
        assert!(hook_is_present(&settings));
        assert_eq!(settings["hooks"]["Stop"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn hook_exige_l_accord_session_id_et_transcript() {
        use std::path::Path;
        assert!(transcript_matches_hook_session(
            Path::new("/tmp/proj/aaaa-session.jsonl"),
            Some("aaaa-session")
        ));
        assert!(
            !transcript_matches_hook_session(
                Path::new("/tmp/proj/bbbb-voisin.jsonl"),
                Some("aaaa-session")
            ),
            "un transcript voisin ne doit pas passer pour la session du hook"
        );
        // Sans session_id, le chemin fourni par Claude Code reste accepté.
        assert!(transcript_matches_hook_session(
            Path::new("/tmp/proj/bbbb-voisin.jsonl"),
            None
        ));
        assert!(transcript_matches_hook_session(
            Path::new("/tmp/proj/bbbb-voisin.jsonl"),
            Some("")
        ));
    }

    #[test]
    fn structure_inattendue_ne_declenche_aucune_modification() {
        // Ni panique, ni écrasement : on refuse de toucher un fichier dont la
        // structure n'est pas celle attendue.
        let mut racine_non_objet = serde_json::json!(["pas", "un", "objet"]);
        assert!(!insert_bridget_hook(&mut racine_non_objet));

        let mut hooks_non_objet = serde_json::json!({"hooks": "une chaîne"});
        assert!(!insert_bridget_hook(&mut hooks_non_objet));
        assert_eq!(hooks_non_objet["hooks"], serde_json::json!("une chaîne"));

        let mut stop_non_liste = serde_json::json!({"hooks": {"Stop": 42}});
        assert!(!insert_bridget_hook(&mut stop_non_liste));
        assert_eq!(stop_non_liste["hooks"]["Stop"], serde_json::json!(42));
    }

    #[test]
    fn ecriture_atomique_preserve_le_contenu() {
        let path = std::env::temp_dir().join(format!("bridget-atomic-{}.json", std::process::id()));
        std::fs::write(&path, "{\"origine\":true}\n").unwrap();
        write_atomically(&path, "{\"nouveau\":true}\n").unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "{\"nouveau\":true}\n"
        );
        // Aucun fichier temporaire ne subsiste dans le répertoire.
        let restes = std::fs::read_dir(std::env::temp_dir())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains(".bridget-"))
            .count();
        assert_eq!(restes, 0);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn cellule_runtime_marque_une_valeur_absente() {
        assert_eq!(cell(Some("claude-opus-5")), "claude-opus-5");
        assert_eq!(cell(None), "—");
    }

    #[test]
    fn arguments_attach_resolvent_les_trois_fenetres() {
        assert_eq!(
            parse_attach_args(&["89000000-0000-4000-8000-000000000011".to_string()]).unwrap(),
            (
                "89000000-0000-4000-8000-000000000011".to_string(),
                AttachWindow::Today
            )
        );
        assert_eq!(
            parse_attach_args(&[
                "89000000-0000-4000-8000-000000000011".to_string(),
                "--from-seq".to_string(),
                "42".to_string(),
            ])
            .unwrap(),
            (
                "89000000-0000-4000-8000-000000000011".to_string(),
                AttachWindow::Seq(42)
            )
        );
        assert_eq!(
            parse_attach_args(&[
                "89000000-0000-4000-8000-000000000011".to_string(),
                "--date".to_string(),
                "2026-08-22".to_string(),
            ])
            .unwrap(),
            (
                "89000000-0000-4000-8000-000000000011".to_string(),
                AttachWindow::Date("2026-08-22".to_string())
            )
        );
    }

    #[test]
    fn arguments_attach_refusent_les_formes_ambigues() {
        assert!(parse_attach_args(&[]).is_err());
        assert!(
            parse_attach_args(&["nom invalide".to_string()]).is_err(),
            "la validation des noms existante reste appliquée"
        );
        assert!(
            parse_attach_args(&[
                "89000000-0000-4000-8000-000000000011".to_string(),
                "--from-seq".to_string(),
                "pas-un-entier".to_string(),
            ])
            .is_err()
        );
        assert!(
            parse_attach_args(&[
                "89000000-0000-4000-8000-000000000011".to_string(),
                "--date".to_string(),
                "2026-08-22".to_string(),
                "--from-seq".to_string(),
                "1".to_string(),
            ])
            .is_err()
        );
    }

    #[test]
    fn spawn_nomme_sans_persistent_avertit_qu_il_ne_survivra_pas() {
        let parsed = parse_spawn_args(&[
            "cursor".to_string(),
            "--agent-id".to_string(),
            "550e8400-e29b-41d4-a716-446655440002".to_string(),
            "--no-persistent".to_string(),
        ])
        .unwrap();
        assert!(!parsed.persistent);
        assert_eq!(
            parsed.agent_id.as_deref(),
            Some("550e8400-e29b-41d4-a716-446655440002")
        );
        let warning = crate::recovery_trace::non_persistent_spawn_warning("cursor3");
        assert!(warning.contains("cursor3"));
        assert!(warning.contains("ne survivra pas au redémarrage"));
        assert!(warning.contains("sans --persistent"));
    }

    /// ORACLE — un spawn neuf REFUSE de partir tant que le sort de l'équipier
    /// n'est pas décidé, et le refus nomme les deux options.
    ///
    /// L'avertissement précédent était affiché puis ignoré cinq fois de suite
    /// le 28/08 : un avertissement qu'on peut ignorer n'est pas une garde.
    /// L'omission est irrattrapable, le `cwd` d'un équipier non persistant
    /// n'existant plus nulle part une fois connecté.
    #[test]
    fn spawn_neuf_refuse_de_partir_sans_choix_de_survie() {
        let root = PathBuf::from("/tmp").join(format!("bgsurv-{}", uuid::Uuid::new_v4().simple()));
        let cwd = root.join("work");
        std::fs::create_dir_all(&cwd).unwrap();
        crate::environment::ensure_private_directory(
            &crate::environment::root_for_home(&root).unwrap(),
        )
        .unwrap();
        let neuf = |options: &[&str]| {
            let mut args = vec!["codex".to_string()];
            args.extend(options.iter().map(|option| option.to_string()));
            resolve_spawn_order(
                &parse_spawn_args(&args).expect("arguments analysables"),
                100,
                &cwd,
                &root,
            )
        };

        let refus = neuf(&["--agent-id", "89000000-0000-4000-8000-000000000012"])
            .expect_err("un spawn muet doit être refusé");
        assert!(
            refus.contains("--persistent") && refus.contains("--no-persistent"),
            "le refus doit nommer les deux options : {refus}"
        );
        assert!(
            refus.contains("irrattrapable"),
            "le refus doit dire pourquoi le choix ne peut pas être différé : {refus}"
        );

        match neuf(&[
            "--agent-id",
            "89000000-0000-4000-8000-000000000013",
            "--persistent",
        ])
        .expect("choix explicite accepté")
        {
            WrapperToDaemon::SpawnOrder { persistent, .. } => assert!(persistent),
            other => panic!("ordre inattendu: {other:?}"),
        }
        match neuf(&[
            "--agent-id",
            "89000000-0000-4000-8000-000000000014",
            "--no-persistent",
        ])
        .expect("choix explicite accepté")
        {
            WrapperToDaemon::SpawnOrder { persistent, .. } => assert!(!persistent),
            other => panic!("ordre inattendu: {other:?}"),
        }

        // Deux drapeaux opposés ne valent pas décision : les départager par
        // « le dernier gagne » restaurerait le défaut silencieux supprimé.
        let contradiction = parse_spawn_args(&[
            "codex".to_string(),
            "--persistent".to_string(),
            "--no-persistent".to_string(),
        ])
        .expect_err("intention contradictoire refusée");
        assert!(contradiction.contains("contradictoires"), "{contradiction}");

        let _ = std::fs::remove_dir_all(root);
    }

    /// ORACLE — le rejeu d'un `--command-id` mémorisé reste possible SANS
    /// redéclarer le choix : l'enveloppe figée le porte déjà. Une garde qui
    /// bloquerait ce rappel bloquerait un lancement légitime.
    #[test]
    fn rejeu_d_un_ordre_memorise_n_exige_pas_de_redeclarer_la_survie() {
        let root = PathBuf::from("/tmp").join(format!("bgretry-{}", uuid::Uuid::new_v4().simple()));
        let cwd = root.join("work");
        std::fs::create_dir_all(&cwd).unwrap();
        crate::environment::ensure_private_directory(
            &crate::environment::root_for_home(&root).unwrap(),
        )
        .unwrap();
        let premier = resolve_spawn_order(
            &parse_spawn_args(&[
                "codex".to_string(),
                "--agent-id".to_string(),
                "89000000-0000-4000-8000-000000000015".to_string(),
                "--persistent".to_string(),
                "--command-id".to_string(),
                "garde-survie-rejeu".to_string(),
            ])
            .unwrap(),
            100,
            &cwd,
            &root,
        )
        .expect("premier envoi accepté");

        let rejeu = resolve_spawn_order(
            &parse_spawn_args(&[
                "codex".to_string(),
                "--command-id".to_string(),
                "garde-survie-rejeu".to_string(),
            ])
            .unwrap(),
            999,
            &cwd,
            &root,
        )
        .expect("le rejeu ne doit pas exiger de redéclarer le choix");
        assert_eq!(encode(&premier).unwrap(), encode(&rejeu).unwrap());

        // En revanche, rejouer en RENVERSANT le choix reste une divergence.
        let divergent = resolve_spawn_order(
            &parse_spawn_args(&[
                "codex".to_string(),
                "--no-persistent".to_string(),
                "--command-id".to_string(),
                "garde-survie-rejeu".to_string(),
            ])
            .unwrap(),
            999,
            &cwd,
            &root,
        )
        .expect_err("un choix renversé au rejeu doit être refusé");
        assert!(divergent.contains("options divergentes"), "{divergent}");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn spawn_cli_rejoue_l_enveloppe_memorisee_octet_pour_octet() {
        let root = PathBuf::from("/tmp").join(format!("bgcanon-{}", uuid::Uuid::new_v4().simple()));
        let cwd = root.join("work");
        std::fs::create_dir_all(&cwd).unwrap();
        crate::environment::ensure_private_directory(
            &crate::environment::root_for_home(&root).unwrap(),
        )
        .unwrap();
        let args = vec![
            "codex".to_string(),
            "--agent-id".to_string(),
            "89000000-0000-4000-8000-000000000016".to_string(),
            "--persistent".to_string(),
            "--command-id".to_string(),
            "command-retry".to_string(),
        ];
        let first =
            resolve_spawn_order(&parse_spawn_args(&args).unwrap(), 100, &cwd, &root).unwrap();
        let retry_args = vec![
            "codex".to_string(),
            "--command-id".to_string(),
            "command-retry".to_string(),
        ];
        let retry = resolve_spawn_order(
            &parse_spawn_args(&retry_args).unwrap(),
            999,
            Path::new("/autre/cwd"),
            &root,
        )
        .unwrap();
        assert_eq!(encode(&first).unwrap(), encode(&retry).unwrap());
        let state_path = spawn_order_path(&root, "command-retry");
        assert_eq!(
            std::fs::metadata(&state_path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert!(
            resolve_spawn_order(
                &parse_spawn_args(&[
                    "claude".to_string(),
                    "--command-id".to_string(),
                    "command-retry".to_string(),
                ])
                .unwrap(),
                999,
                &cwd,
                &root,
            )
            .unwrap_err()
            .contains("options divergentes")
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn spec_091_posture_replay_is_exact_and_divergence_refused() {
        let root = PathBuf::from("/tmp").join(format!("bg091-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&root).unwrap();
        let private_root = crate::environment::root_for_home(&root).unwrap();
        std::fs::create_dir_all(&private_root).unwrap();
        std::fs::set_permissions(&private_root, std::fs::Permissions::from_mode(0o700)).unwrap();
        let args = [
            "codex",
            "--persistent",
            "--command-id",
            "scoped-091",
            "--posture",
            "development",
        ]
        .map(str::to_string);
        let first =
            resolve_spawn_order(&parse_spawn_args(&args).unwrap(), 100, &root, &root).unwrap();
        let retry = ["codex", "--command-id", "scoped-091"].map(str::to_string);
        let replay = resolve_spawn_order(
            &parse_spawn_args(&retry).unwrap(),
            200,
            Path::new("/different"),
            &root,
        )
        .unwrap();
        assert_eq!(encode(&first).unwrap(), encode(&replay).unwrap());
        let divergent = [
            "codex",
            "--command-id",
            "scoped-091",
            "--posture",
            "discovery",
        ]
        .map(str::to_string);
        assert!(
            resolve_spawn_order(&parse_spawn_args(&divergent).unwrap(), 200, &root, &root)
                .unwrap_err()
                .contains("divergentes")
        );
        for value in ["complete", "allow", "", "workspace-write"] {
            assert!(
                parse_spawn_args(&[
                    "codex".to_string(),
                    "--posture".to_string(),
                    value.to_string()
                ])
                .is_err()
            );
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spawn_roundtrip_rejoue_la_meme_issue_apres_reponse_perdue() {
        let socket = PathBuf::from(format!(
            "/tmp/bg-t905-{}-{}.sock",
            std::process::id(),
            uuid::Uuid::new_v4().simple()
        ));
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let mut observed = Vec::new();
            for _ in 0..2 {
                let (stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut writer = BufWriter::new(stream);
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                assert!(matches!(
                    decode::<WrapperToDaemon>(line.trim()).unwrap(),
                    WrapperToDaemon::Register { .. }
                ));
                writeln!(
                    writer,
                    "{}",
                    encode(&DaemonToWrapper::Registered {
                        credential: None,
                        agent_id: "cli-test".to_string()
                    })
                    .unwrap()
                )
                .unwrap();
                writer.flush().unwrap();
                line.clear();
                reader.read_line(&mut line).unwrap();
                observed.push(line.trim().to_string());
                writeln!(
                    writer,
                    "{}",
                    encode(&DaemonToWrapper::SpawnAccepted {
                        command_id: "command-lost".to_string(),
                        agent_id: "codex-managed".to_string(),
                        definition: Some(bridget_transport::ResolvedAgentDefinition {
                            command: "npx".to_string(),
                            args: vec!["fixture-acp".to_string()],
                            protocol: "acp".to_string(),
                            forbidden_env: vec!["OPENAI_API_KEY".to_string()],
                            pass_env: Vec::new(),
                            claude_config_dir: None,
                            codex_home: None,
                            permissions: "allow".to_string(),
                            queue_capacity: 32,
                            notify_timeout_secs: 600,
                            mcp: bridget_transport::ResolvedMcpDefinition {
                                interactive: "none".to_string(),
                                acp_session: false,
                            },
                            capabilities: bridget_transport::AdapterCapabilities::default(),
                            digest: "fixture-digest".to_string(),
                        }),
                    })
                    .unwrap()
                )
                .unwrap();
                writer.flush().unwrap();
            }
            observed
        });
        let order = WrapperToDaemon::SpawnOrder {
            posture: None,
            agent_type: "codex".to_string(),
            agent_id: Some("codex-managed".to_string()),
            cwd: "/tmp".to_string(),
            persistent: false,
            command_id: "command-lost".to_string(),
            issued_at: 100,
            deadline_at: 110,
            project: None,
            ownership: None,
        };
        for _ in 0..2 {
            assert!(matches!(
                send_control_to_daemon_at(&socket, order.clone()).unwrap(),
                DaemonToWrapper::SpawnAccepted { ref command_id, ref agent_id, .. }
                    if command_id == "command-lost" && agent_id == "codex-managed"
            ));
        }
        let observed = server.join().unwrap();
        assert_eq!(observed[0], observed[1], "le retry doit être canonique");
        let _ = std::fs::remove_file(socket);
    }

    #[test]
    fn stop_cli_genere_ou_reutilise_un_command_id() {
        let generated =
            parse_stop_args(&["89000000-0000-4000-8000-000000000011".to_string()]).unwrap();
        assert_eq!(generated.0, "89000000-0000-4000-8000-000000000011");
        assert!(!generated.1.is_empty());
        assert_eq!(
            parse_stop_args(&[
                "89000000-0000-4000-8000-000000000011".to_string(),
                "--command-id".to_string(),
                "stop-retry".to_string(),
            ])
            .unwrap(),
            (
                "89000000-0000-4000-8000-000000000011".to_string(),
                "stop-retry".to_string()
            )
        );
    }

    /// Payload StatusLine conforme au schéma publié par Claude Code, avec les
    /// deux fenêtres présentes. Le reste du payload est là pour vérifier qu'on
    /// ne confond pas la barre de contexte avec une limite de forfait.
    fn statusline_payload_complet() -> serde_json::Value {
        serde_json::json!({
            "session_id": "abc",
            "model": {"display_name": "Opus"},
            "context_window": {"used_percentage": 42, "remaining_percentage": 58},
            "rate_limits": {
                "five_hour": {"used_percentage": 19, "resets_at": 1787590200_i64},
                "seven_day": {"used_percentage": 61, "resets_at": 1788136905_i64}
            }
        })
    }

    #[test]
    fn statusline_releve_les_deux_fenetres_attestees() {
        let facts = rate_limit_facts_from_statusline(&statusline_payload_complet());
        assert_eq!(
            facts,
            vec![
                StatusLineLimit {
                    window: "five_hour".to_string(),
                    used_percent: Some(19),
                    resets_at: Some(1787590200),
                },
                StatusLineLimit {
                    window: "seven_day".to_string(),
                    used_percent: Some(61),
                    resets_at: Some(1788136905),
                },
            ]
        );
    }

    /// Le cas exigé : sans bloc `rate_limits`, on ne pousse RIEN. Le schéma
    /// Claude Code le donne absent hors abonnement et avant la première
    /// réponse d'API — la colonne du référent doit rester vide, pas se
    /// remplir d'un zéro inventé.
    #[test]
    fn statusline_sans_bloc_limites_ne_pousse_rien() {
        let sans_limites = serde_json::json!({
            "session_id": "abc",
            "context_window": {"used_percentage": 42}
        });
        assert!(rate_limit_facts_from_statusline(&sans_limites).is_empty());
    }

    /// La barre de contexte porte AUSSI un `used_percentage`, à un autre
    /// endroit et pour une autre grandeur. Le confondre avec la consommation
    /// de forfait afficherait le remplissage du contexte comme un quota.
    #[test]
    fn statusline_ne_prend_pas_le_contexte_pour_une_limite() {
        let contexte_seul = serde_json::json!({
            "context_window": {"used_percentage": 97, "remaining_percentage": 3}
        });
        assert!(rate_limit_facts_from_statusline(&contexte_seul).is_empty());
    }

    #[test]
    fn statusline_garde_la_fenetre_presente_et_ignore_l_absente() {
        let une_seule = serde_json::json!({
            "rate_limits": {"five_hour": {"used_percentage": 5, "resets_at": 1787590200_i64}}
        });
        let facts = rate_limit_facts_from_statusline(&une_seule);
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].window, "five_hour");
    }

    #[test]
    fn statusline_tronque_le_pourcentage_fractionnaire() {
        let fractionnaire = serde_json::json!({
            "rate_limits": {"five_hour": {"used_percentage": 19.87, "resets_at": 1787590200_i64}}
        });
        assert_eq!(
            rate_limit_facts_from_statusline(&fractionnaire)[0].used_percent,
            Some(19)
        );
    }

    /// Hors domaine ou non numérique : on n'invente pas, et on ne rabat pas
    /// dans les bornes. La fenêtre survit par son `resets_at` attesté, sans
    /// pourcentage — le daemon aurait refusé un 150 %.
    #[test]
    fn statusline_ecarte_un_pourcentage_hors_domaine_sans_le_rabattre() {
        let aberrant = serde_json::json!({
            "rate_limits": {
                "five_hour": {"used_percentage": 150, "resets_at": 1787590200_i64},
                "seven_day": {"used_percentage": "beaucoup", "resets_at": 1788136905_i64}
            }
        });
        let facts = rate_limit_facts_from_statusline(&aberrant);
        assert_eq!(facts.len(), 2);
        assert!(facts.iter().all(|fact| fact.used_percent.is_none()));
        assert!(facts.iter().all(|fact| fact.resets_at.is_some()));
    }

    /// Un instant de retour nul ou négatif serait refusé par le daemon : le
    /// hook ne le fabrique pas en `None` silencieux d'une fenêtre par ailleurs
    /// vide — la fenêtre disparaît entièrement.
    #[test]
    fn statusline_ne_pousse_pas_une_fenetre_sans_aucune_valeur() {
        let vides = serde_json::json!({
            "rate_limits": {
                "five_hour": {},
                "seven_day": {"resets_at": 0}
            }
        });
        assert!(rate_limit_facts_from_statusline(&vides).is_empty());
    }

    /// Le statut n'est pas attesté par ce payload : il ne doit jamais valoir
    /// « allowed » (verdict inventé) ni « rejected » (qui ferait afficher
    /// « épuisée » sur une fenêtre saine).
    #[test]
    fn statusline_n_invente_aucun_verdict_de_statut() {
        assert_eq!(STATUSLINE_LIMIT_STATUS, "unknown");
        assert_ne!(STATUSLINE_LIMIT_STATUS, "allowed");
        assert_ne!(STATUSLINE_LIMIT_STATUS, "rejected");
    }

    /// Jonction : ce que le hook relève doit remplir la colonne du référent.
    ///
    /// Relie l'extraction au rendu réel de `who`, avec le statut non attesté
    /// tel qu'il sera posé. Sans cet oracle, l'extraction pourrait être juste
    /// et la colonne rester vide — c'est la colonne qui est la mission.
    #[test]
    fn statusline_remplit_la_colonne_du_referent() {
        let facts = rate_limit_facts_from_statusline(&statusline_payload_complet());
        let agent = AgentInfo {
            agent_id: uuid::Uuid::new_v4().to_string(),
            display_name: "bridget".to_string(),
            agent_type: "claude".to_string(),
            connection_id: "conn-referent".to_string(),
            host: "local".to_string(),
            transport: "unix".to_string(),
            channel: None,
            mode: Some(PresenceMode::Tmux),
            location: None,
            os: "macOS".to_string(),
            state: "connected".to_string(),
            last_seen_secs: 0,
            reconnect_count: 0,
            domain: None,
            model: Some("claude-fable-5".to_string()),
            effort: None,
            rate_limits: facts
                .iter()
                .map(|fact| bridget_transport::protocol::RateLimitFact {
                    credits: None,
                    window: fact.window.clone(),
                    status: STATUSLINE_LIMIT_STATUS.to_string(),
                    resets_at: fact.resets_at,
                    used_percent: fact.used_percent,
                })
                .collect(),
            model_mismatch: None,
            disk_space: None,
            provider: None,
            persistent: None,
            execution: None,
            agent_link: None,
        };
        let rendered = format_rate_limit(&agent);
        assert!(rendered.contains("5h 19% rst "), "{rendered}");
        assert!(rendered.contains("7d 61% rst "), "{rendered}");
        // Le statut non attesté ne doit pas fuir à l'écran.
        assert!(!rendered.contains("unknown"), "{rendered}");
        assert!(!rendered.contains("épuisée"), "{rendered}");
        assert_ne!(
            rendered, "—",
            "la colonne du référent ne doit plus être vide"
        );
    }

    /// ORACLE PRINCIPAL DU CORRECTIF : le hook n'émet AUCUN `Register`.
    ///
    /// La propriété se mesure à LA SOURCE — les trames émises — et non par une
    /// inspection de l'annuaire après la poussée. Ce choix n'est pas un confort
    /// de test, c'est une nécessité démontrée : à la fermeture d'une connexion
    /// le daemon appelle `router.unregister_by_conn`, si bien qu'une présence
    /// transitoire a TOUJOURS disparu au moment où on interroge `ListAgents`.
    /// Un oracle tardif passe donc au vert même avec le `Register` remis en
    /// place — vérifié : le mutant survivait. C'est exactement le piège qui a
    /// rendu le constat d'origine difficile à saisir (« observée comme agent
    /// LIBRE, disparue avant inspection »).
    ///
    /// `Register` étant le seul chemin vers `router.register` hors tests, ne
    /// pas l'émettre suffit à ne jamais peupler l'annuaire.
    #[test]
    fn statusline_n_emet_aucune_declaration_de_presence() {
        let unique = format!("{}-{}", std::process::id(), line!());
        let socket = std::env::temp_dir().join(format!("bridget-statusline-{unique}.sock"));
        std::fs::remove_file(&socket).ok();
        let listener = UnixListener::bind(&socket).unwrap();

        // Faux daemon : il accuse tout, et retient ce qu'on lui a dit.
        let recu = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let journal = std::sync::Arc::clone(&recu);
        let serveur = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut writer = BufWriter::new(stream.try_clone().unwrap());
            let reader = BufReader::new(stream);
            for line in reader.lines() {
                let Ok(line) = line else { break };
                journal.lock().unwrap().push(line);
                let ack = DaemonToWrapper::Ack {
                    id: "rate-limit".to_string(),
                    project_warnings: Vec::new(),
                };
                if writeln!(writer, "{}", encode(&ack).unwrap()).is_err() || writer.flush().is_err()
                {
                    break;
                }
            }
        });

        let facts = rate_limit_facts_from_statusline(&statusline_payload_complet());
        send_rate_limits_to_daemon(&socket, "89000000-0000-4000-8000-000000000105", &facts)
            .unwrap();
        serveur.join().unwrap();
        std::fs::remove_file(&socket).ok();

        let trames = recu.lock().unwrap().clone();
        let declarations: Vec<_> = trames
            .iter()
            .filter(|trame| {
                matches!(
                    decode::<WrapperToDaemon>(trame.trim()),
                    Ok(WrapperToDaemon::Register { .. })
                )
            })
            .collect();
        assert!(
            declarations.is_empty(),
            "le hook s'est déclaré à l'annuaire : {declarations:?}"
        );

        // Et il n'a rien émis d'autre que ses deux faits, attribués au référent.
        let fenetres: Vec<_> = trames
            .iter()
            .map(|trame| decode::<WrapperToDaemon>(trame.trim()).unwrap())
            .map(|message| match message {
                WrapperToDaemon::RateLimit { agent, window, .. } => (agent, window),
                other => panic!("trame inattendue émise par le hook : {other:?}"),
            })
            .collect();
        assert_eq!(
            fenetres,
            vec![
                (
                    "89000000-0000-4000-8000-000000000105".to_string(),
                    "five_hour".to_string()
                ),
                (
                    "89000000-0000-4000-8000-000000000105".to_string(),
                    "seven_day".to_string()
                ),
            ],
            "le hook doit émettre ses deux faits, et rien de plus"
        );
    }

    /// ORACLE D'ATTRIBUTION, contre un daemon jetable : la poussée sans
    /// déclaration remplit bel et bien la colonne du référent.
    ///
    /// Complément indispensable du précédent : supprimer le `Register` sans
    /// vérifier l'attribution échangerait un annuaire bruyant contre une
    /// colonne muette. On observe par la projection publique `ListAgents`,
    /// celle-là même que lisent les rondes.
    ///
    /// Le daemon est jetable : socket et base sous `TMPDIR`, uniques, détruits
    /// en fin de test. Jamais la production.
    #[test]
    fn statusline_attribue_ses_faits_sans_se_declarer() {
        let root = PathBuf::from("/tmp").join(format!("bgsl-{}", uuid::Uuid::new_v4().simple()));
        crate::environment::ensure_private_directory(&root).unwrap();
        let socket = root.join("bridget.sock");
        let db_path = root.join("bridget.db");
        let config = crate::daemon::DaemonConfig {
            socket_path: socket.clone(),
            db_path: db_path.clone(),
            log_path: root.join("daemon.log"),
            ..Default::default()
        };
        let daemon_socket = socket.clone();
        thread::spawn(move || {
            let _ = crate::daemon::run(config);
        });
        for _ in 0..100 {
            if daemon_socket.exists() {
                break;
            }
            thread::sleep(std::time::Duration::from_millis(50));
        }
        assert!(socket.exists(), "daemon jetable non prêt");

        // Le référent, lui, est un vrai agent : il se déclare. C'est la cible
        // des faits, et le seul nom qui a le droit d'être à l'annuaire.
        let referent = UnixStream::connect(&socket).unwrap();
        let mut referent_writer = BufWriter::new(referent.try_clone().unwrap());
        let mut referent_reader = BufReader::new(referent);
        let register = WrapperToDaemon::Register {
            agent_type: "claude".to_string(),
            identity_version: 2,
            agent_id: "89000000-0000-4000-8000-000000000105".to_string(),
            host: None,
            transport: None,
            channel: None.into(),
            mode: Some(PresenceMode::Tmux),
            location: None,
            os: None,
            // Indispensable, et pas un détail de fixture : une présence est
            // indexée par instance, et `presence_of_agent` remonte
            // agent → connexion → instance. Sans `instance_id`, le référent
            // serait à l'annuaire mais sans présence — donc sans colonne à
            // remplir, et l'oracle mesurerait autre chose que la mission.
            instance_id: Some("instance-referent-oracle".to_string()),
            domain: None,
            turn_in_progress: false,
            journal_available: None,
            turn_attest: false,
            queue_fallback: false,
        };
        writeln!(referent_writer, "{}", encode(&register).unwrap()).unwrap();
        referent_writer.flush().unwrap();
        let mut line = String::new();
        referent_reader.read_line(&mut line).unwrap();
        assert!(
            matches!(
                decode(line.trim()).unwrap(),
                DaemonToWrapper::Registered { .. }
            ),
            "le référent doit être enregistré : {line}"
        );

        // LA POUSSÉE, par le chemin de production exact.
        let facts = rate_limit_facts_from_statusline(&statusline_payload_complet());
        assert_eq!(facts.len(), 2, "le payload d'essai porte deux fenêtres");
        send_rate_limits_to_daemon(&socket, "89000000-0000-4000-8000-000000000105", &facts)
            .expect("la poussée doit aboutir sans se déclarer");

        let agents = match ask_agent_list(&socket) {
            DaemonToWrapper::AgentList { agents } => agents,
            other => panic!("AgentList attendu, reçu {other:?}"),
        };

        // On n'assert PAS ici l'absence de `cli-statusline-*` : la connexion
        // du hook est déjà refermée, donc déjà désenregistrée. L'assertion
        // serait vraie quoi qu'il arrive, y compris avec le défaut. Cette
        // propriété-là se prouve à la source, dans l'oracle précédent.
        let referent_info = agents
            .iter()
            .find(|agent| agent.agent_id == "89000000-0000-4000-8000-000000000105")
            .expect("le référent doit rester à l'annuaire");
        let mut windows: Vec<_> = referent_info
            .rate_limits
            .iter()
            .map(|fact| (fact.window.as_str(), fact.used_percent))
            .collect();
        windows.sort_unstable();
        assert_eq!(
            windows,
            vec![("five_hour", Some(19)), ("seven_day", Some(61))],
            "les faits poussés doivent être attribués au référent"
        );
        assert!(
            referent_info
                .rate_limits
                .iter()
                .all(|fact| fact.status == STATUSLINE_LIMIT_STATUS),
            "le statut non attesté doit être conservé tel quel"
        );

        std::fs::remove_file(&socket).ok();
        std::fs::remove_file(&db_path).ok();
    }

    /// Interroge la projection publique de l'annuaire, sans se déclarer non
    /// plus : l'observateur ne doit pas fausser ce qu'il mesure.
    fn ask_agent_list(socket: &Path) -> DaemonToWrapper {
        let stream = UnixStream::connect(socket).unwrap();
        let mut writer = BufWriter::new(stream.try_clone().unwrap());
        let mut reader = BufReader::new(stream);
        writeln!(writer, "{}", encode(&WrapperToDaemon::ListAgents).unwrap()).unwrap();
        writer.flush().unwrap();
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        decode(line.trim()).unwrap()
    }
