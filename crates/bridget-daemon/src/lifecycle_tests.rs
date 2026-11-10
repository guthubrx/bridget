    #[test]
    fn native148_child_never_inherits_t3_session_credentials() {
        let registry=super::AgentRegistry::from_json(r#"{"agents":{"fixture":{"command":"/bin/sh","pass_env":["BRIDGET_T3_MCP_ENDPOINT","BRIDGET_T3_MCP_AUTHORIZATION","SAFE_SETTING"]}}}"#,"/tmp/native-env148.json").unwrap();
        let source = super::SourceEnvironment::from([
            (
                "HOME".into(),
                std::ffi::OsString::from("/tmp/native148-home"),
            ),
            (
                "BRIDGET_T3_MCP_ENDPOINT".into(),
                std::ffi::OsString::from("http://127.0.0.1:1/mcp"),
            ),
            (
                "BRIDGET_T3_MCP_AUTHORIZATION".into(),
                std::ffi::OsString::from("private-session"),
            ),
            ("SAFE_SETTING".into(), std::ffi::OsString::from("retained")),
        ]);
        let environment =
            super::build_environment(registry.get("fixture").unwrap(), &source).unwrap();
        assert!(!environment.contains_key("BRIDGET_T3_MCP_ENDPOINT"));
        assert!(!environment.contains_key("BRIDGET_T3_MCP_AUTHORIZATION"));
        assert_eq!(
            environment.get("SAFE_SETTING"),
            Some(&std::ffi::OsString::from("retained"))
        );
    }
    use super::*;
    use crate::desired_state::DesiredStateStore;
    use crate::fleet::FleetConfig;
    use std::fs;

    const NOW: i64 = 2_000_000;

    fn root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "bridget-t905-{label}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ))
    }

    fn supervisor(root: &Path, quota: usize) -> FleetSupervisor {
        fs::create_dir_all(root).unwrap();
        FleetSupervisor::open(
            &root.join("bridget.db"),
            DesiredStateStore::at_path(root.join("fleet.json")),
            FleetConfig {
                quota,
                persistent_horizon_secs: 3600,
                ephemeral_horizon_secs: 300,
                issued_at_tolerance_secs: 30,
            },
        )
        .unwrap()
    }

    fn registry(command: &str, protocol: &str, forbidden: &[&str]) -> AgentRegistry {
        AgentRegistry::from_json(
            &serde_json::json!({
                "agents": {
                    "fixture": {
                        "command": command,
                        "protocol": protocol,
                        "forbidden_env": forbidden,
                        "pass_env": ["SPECIAL_AUTH"]
                    }
                }
            })
            .to_string(),
            "/tmp/t905-agents.json",
        )
        .unwrap()
    }

    fn registry_with_capabilities(args: &[&str], capabilities: serde_json::Value) -> AgentRegistry {
        AgentRegistry::from_json(
            &serde_json::json!({
                "agents": {
                    "fixture": {
                        "command": "/bin/sh",
                        "args": args,
                        "protocol": "acp",
                        "forbidden_env": [],
                        "pass_env": ["SPECIAL_AUTH"],
                        "capabilities": capabilities
                    }
                }
            })
            .to_string(),
            "/tmp/t905-capabilities.json",
        )
        .unwrap()
    }

    fn source(home: &Path) -> SourceEnvironment {
        BTreeMap::from([
            ("HOME".to_string(), home.as_os_str().to_owned()),
            ("PATH".to_string(), OsString::from("/bin:/usr/bin")),
            ("USER".to_string(), OsString::from("tester")),
            ("LANG".to_string(), OsString::from("fr_FR.UTF-8")),
            ("TMPDIR".to_string(), OsString::from("/tmp")),
            ("SPECIAL_AUTH".to_string(), OsString::from("présent")),
        ])
    }

    /// Deux machines DISTINCTES : c'est la situation fédérée réelle, et c'est
    /// la seule où l'oracle du point 5 peut distinguer les deux hôtes.
    fn hosts_fixture() -> SpawnHosts {
        SpawnHosts {
            searched_on: "monordinateur".to_string(),
            requested_from: "cartae".to_string(),
        }
    }

    fn order(root: &Path, id: &str, name: &str) -> SpawnOrder {
        SpawnOrder {
            posture: None,
            agent_type: "fixture".to_string(),
            requested_name: Some(name.to_string()),
            cwd: root.to_path_buf(),
            persistent: false,
            command_id: id.to_string(),
            issued_at: NOW,
            deadline_at: NOW + 10,
            ownership: None,
            project: None,
            remote_placement: None,
        }
    }

    fn rejection(decision: SpawnDecision) -> SpawnRefusal {
        match decision {
            SpawnDecision::Rejected(reason) => reason,
            other => panic!("refus attendu, obtenu: {other:?}"),
        }
    }

    #[test]
    fn native157_preflight_binaire_absent() {
        let root = root("native157-missing");
        let supervisor = supervisor(&root, 1);
        let command = root.join("binaire-absent157");
        let registry = registry(command.to_str().unwrap(), "acp", &[]);
        let env = source(&root);
        let refusal = launch_preflight(&registry, "fixture", &env).unwrap_err();
        assert_eq!(refusal_record(&refusal).0, "command_missing");
        assert!(
            matches!(refusal, SpawnRefusal::CommandMissing { ref command, .. } if command.ends_with("binaire-absent157"))
        );
        let request = order(
            &root,
            "native157-missing",
            "89000000-0000-4000-8000-000000000701",
        );
        let refusal = rejection(
            submit_spawn(
                &supervisor,
                &registry,
                &env,
                &request,
                NOW,
                false,
                &hosts_fixture(),
            )
            .unwrap(),
        );
        assert_eq!(refusal_record(&refusal).0, "command_missing");
        assert!(
            !supervisor.knows_command(&request.command_id),
            "refus avant réservation"
        );
        assert_eq!(supervisor.active_count(), 0);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn native157_preflight_cle_api_interdite_binaire_present() {
        let root = root("native157-billing");
        let supervisor = supervisor(&root, 1);
        let registry = registry("/bin/sh", "acp", &["OPENAI_API_KEY"]);
        let mut env = source(&root);
        env.insert(
            "OPENAI_API_KEY".into(),
            OsString::from("fixture-interdite157"),
        );
        assert!(
            command_exists("/bin/sh", &env),
            "binaire réellement présent"
        );
        let refusal = launch_preflight(&registry, "fixture", &env).unwrap_err();
        assert_eq!(refusal_record(&refusal).0, "billing_guard");
        assert!(
            matches!(refusal, SpawnRefusal::BillingGuard { ref variable } if variable == "OPENAI_API_KEY")
        );
        let request = order(
            &root,
            "native157-billing",
            "89000000-0000-4000-8000-000000000702",
        );
        let refusal = rejection(
            submit_spawn(
                &supervisor,
                &registry,
                &env,
                &request,
                NOW,
                false,
                &hosts_fixture(),
            )
            .unwrap(),
        );
        assert_eq!(refusal_record(&refusal).0, "billing_guard");
        assert!(
            !supervisor.knows_command(&request.command_id),
            "refus avant réservation"
        );
        assert_eq!(supervisor.active_count(), 0);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn native157_preflight_home_absent_binaire_present() {
        let root = root("native157-home");
        let supervisor = supervisor(&root, 1);
        let registry = registry("/bin/sh", "acp", &[]);
        let mut env = source(&root);
        env.remove("HOME");
        assert!(
            command_exists("/bin/sh", &env),
            "binaire réellement présent sans HOME"
        );
        let refusal = launch_preflight(&registry, "fixture", &env).unwrap_err();
        assert_eq!(refusal_record(&refusal).0, "env_unfit");
        assert!(
            matches!(refusal, SpawnRefusal::EnvUnfit { ref detail } if detail.contains("HOME absent"))
        );
        let request = order(
            &root,
            "native157-home",
            "89000000-0000-4000-8000-000000000703",
        );
        let refusal = rejection(
            submit_spawn(
                &supervisor,
                &registry,
                &env,
                &request,
                NOW,
                false,
                &hosts_fixture(),
            )
            .unwrap(),
        );
        assert_eq!(refusal_record(&refusal).0, "env_unfit");
        assert!(
            !supervisor.knows_command(&request.command_id),
            "erreur environnement propagée avant réservation"
        );
        assert_eq!(supervisor.active_count(), 0);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn native157_preflight_protocole_non_gere() {
        let root = root("native157-protocol");
        let supervisor = supervisor(&root, 1);
        // Capacité déclarée valide : le refus vise le protocole non géré, pas sa capacité absente.
        let registry = AgentRegistry::from_json(
            &serde_json::json!({"agents":{"fixture":{
                "command":"/bin/sh", "protocol":"tmux", "forbidden_env":[],
                "capabilities":{"execution_paths":["tmux"]}
            }}})
            .to_string(),
            "/tmp/native157-protocol.json",
        )
        .unwrap();
        assert!(validate_launch_capabilities("fixture", registry.get("fixture").unwrap()).is_ok());
        let env = source(&root);
        let refusal = launch_preflight(&registry, "fixture", &env).unwrap_err();
        assert_eq!(refusal_record(&refusal).0, "negotiation_failed");
        assert!(
            matches!(refusal, SpawnRefusal::NegotiationFailed { ref detail } if detail.contains("tmux"))
        );
        let request = order(
            &root,
            "native157-protocol",
            "89000000-0000-4000-8000-000000000704",
        );
        let refusal = rejection(
            submit_spawn(
                &supervisor,
                &registry,
                &env,
                &request,
                NOW,
                false,
                &hosts_fixture(),
            )
            .unwrap(),
        );
        assert_eq!(refusal_record(&refusal).0, "negotiation_failed");
        assert!(!supervisor.knows_command(&request.command_id));
        assert_eq!(supervisor.active_count(), 0);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn native157_preflight_definition_epinglee() {
        let root = root("native157-pinned");
        let supervisor = supervisor(&root, 1);
        let original = registry("/bin/sh", "acp", &[]);
        let resolved = original.resolved_definition("fixture").unwrap();
        let pinned = AgentRegistry::from_resolved("fixture", &resolved).unwrap();
        let changed = registry("/commande-mutante-absente157", "acp", &[]);
        let env = source(&root);
        assert!(launch_preflight(&pinned, "fixture", &env).is_ok());
        assert_eq!(
            refusal_record(&launch_preflight(&changed, "fixture", &env).unwrap_err()).0,
            "command_missing"
        );
        let request = order(
            &root,
            "native157-pinned",
            "89000000-0000-4000-8000-000000000705",
        );
        let prepared = match submit_spawn(
            &supervisor,
            &pinned,
            &env,
            &request,
            NOW,
            false,
            &hosts_fixture(),
        )
        .unwrap()
        {
            SpawnDecision::Ready(prepared) => prepared,
            other => panic!("définition figée lançable attendue : {other:?}"),
        };
        assert_eq!(prepared.command, "/bin/sh");
        assert_eq!(*prepared.resolved_definition, resolved);
        let recovered = prepare_recovery(
            &env,
            RecoveryCandidate {
                lease: prepared.lease,
                agent_type: prepared.agent_type,
                cwd: prepared.cwd,
                resolved_definition: Some(*prepared.resolved_definition),
                runtime_execution: None,
            },
        )
        .unwrap();
        assert_eq!(recovered.command, "/bin/sh");
        assert_eq!(*recovered.resolved_definition, resolved);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn environnement_construit_ne_copie_que_baseline_et_pass_env() {
        let root = root("env");
        fs::create_dir_all(&root).unwrap();
        let definition = registry("/bin/sh", "acp", &[])
            .get("fixture")
            .unwrap()
            .clone();
        let mut source = source(&root);
        source.insert("SECRET_INATTENDU".to_string(), OsString::from("non"));
        let env = build_environment(&definition, &source).unwrap();
        assert_eq!(env.get("SPECIAL_AUTH"), Some(&OsString::from("présent")));
        assert!(!env.contains_key("SECRET_INATTENDU"));
        let directory = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let path = env.get("PATH").unwrap().to_string_lossy();
        // Intention : le binaire courant doit gagner la résolution (premier élément).
        assert_eq!(
            path.split(':').map(str::trim).next(),
            Some(directory.as_str()),
            "PATH géré sans préfixe du binaire courant: {path}"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn profil_claude_prive_ecrase_toute_valeur_ambiante_et_refuse_les_droits_larges() {
        let root = root("claude-profile");
        let profile = root.join("profile");
        fs::create_dir_all(&profile).unwrap();
        fs::set_permissions(&profile, fs::Permissions::from_mode(0o700)).unwrap();
        let mut definition = registry("/bin/sh", "claude_stream_json", &[])
            .get("fixture")
            .unwrap()
            .clone();
        definition.claude_config_dir = Some(profile.to_string_lossy().into_owned());
        let mut source = source(&root);
        source.insert("CLAUDE_CONFIG_DIR".to_string(), OsString::from("/ambient"));
        let env = build_environment(&definition, &source).unwrap();
        assert_eq!(
            env.get("CLAUDE_CONFIG_DIR"),
            Some(&profile.as_os_str().to_owned())
        );

        fs::set_permissions(&profile, fs::Permissions::from_mode(0o755)).unwrap();
        let refusal = build_environment(&definition, &source).unwrap_err();
        assert!(matches!(
            refusal,
            SpawnRefusal::EnvUnfit { detail } if detail.contains("attendu 0700")
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn session_claude_native_est_preparable_comme_equipier_gere() {
        let root = root("claude-stream-json");
        fs::create_dir_all(&root).unwrap();
        let supervisor = supervisor(&root, 1);
        let registry = AgentRegistry::from_json(
            &serde_json::json!({
                "agents": {
                    "fixture": {
                        "command": "/bin/sh",
                        "args": ["--model", "claude-opus-5"],
                        "protocol": "claude_stream_json",
                        "forbidden_env": [],
                        "pass_env": ["SPECIAL_AUTH"],
                        "capabilities": {
                            "execution_paths": ["claude_stream_json"],
                            "models": {"claude-opus-5": {"efforts": []}}
                        }
                    }
                }
            })
            .to_string(),
            "/tmp/t905-claude-native.json",
        )
        .unwrap();
        let decision = submit_spawn(
            &supervisor,
            &registry,
            &source(&root),
            &order(
                &root,
                "claude-native",
                "89000000-0000-4000-8000-000000000201",
            ),
            NOW,
            false,
            &hosts_fixture(),
        )
        .unwrap();
        assert!(matches!(decision, SpawnDecision::Ready(_)));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn capacite_absente_refuse_avant_reservation_et_sans_processus() {
        let root = root("capability-preflight");
        fs::create_dir_all(&root).unwrap();
        let supervisor = supervisor(&root, 1);
        let registry = registry_with_capabilities(
            &["--model", "gpt-5.6-terra"],
            serde_json::json!({
                "execution_paths": ["acp"],
                "models": {"gpt-5.5": {"efforts": ["low"]}}
            }),
        );
        let order = order(
            &root,
            "command-unsupported-model",
            "89000000-0000-4000-8000-000000000202",
        );
        let refusal = rejection(
            submit_spawn(
                &supervisor,
                &registry,
                &source(&root),
                &order,
                NOW,
                false,
                &hosts_fixture(),
            )
            .unwrap(),
        );
        assert_eq!(
            refusal,
            SpawnRefusal::UnsupportedCapability {
                agent_type: "fixture".to_string(),
                model: "gpt-5.6-terra".to_string(),
                capability: "modèle pris en charge par l'adaptateur".to_string(),
            }
        );
        assert!(!supervisor.knows_command(&order.command_id));
        assert_eq!(supervisor.active_count(), 0);
        // Mutation discriminante : déplacer la garde après request_spawn rend
        // cette commande connue et transformerait ce refus sans processus en
        // état durable résiduel.
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn observation_fournisseur_incoherente_refuse_avant_reservation() {
        let root = root("provider-observation-preflight");
        fs::create_dir_all(&root).unwrap();
        let supervisor = supervisor(&root, 1);
        let registry = registry_with_capabilities(
            &[],
            serde_json::json!({
                "execution_paths": ["acp"],
                "models": {},
                "observed": {
                    "binary_path": "/bin/sh",
                    "binary_version": "fixture-1",
                    "binary_digest": "0000000000000000000000000000000000000000000000000000000000000000",
                    "contract_version": "acp-v1",
                    "operations": []
                }
            }),
        );
        let order = order(
            &root,
            "command-provider-observation",
            "89000000-0000-4000-8000-000000000202",
        );
        let refusal = rejection(
            submit_spawn(
                &supervisor,
                &registry,
                &source(&root),
                &order,
                NOW,
                false,
                &hosts_fixture(),
            )
            .unwrap(),
        );
        assert!(matches!(
            refusal,
            // Session 221 (a83186df) : une empreinte bien formée mais différente
            // désigne un programme modifié, requalifié seulement pour Codex.
            SpawnRefusal::UnsupportedCapability { ref capability, .. }
                if capability == "compatibilité du programme modifié non vérifiée"
        ));
        assert!(!supervisor.knows_command(&order.command_id));
        assert_eq!(supervisor.active_count(), 0);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn effort_non_declare_est_refuse_et_reprise_revalidee_sur_definition_figee() {
        let root = root("capability-effort");
        fs::create_dir_all(&root).unwrap();
        let registry = registry_with_capabilities(
            &["model=gpt-5.6-terra", "effort=high"],
            serde_json::json!({
                "execution_paths": ["acp"],
                "models": {"gpt-5.6-terra": {"efforts": ["low"]}}
            }),
        );
        let supervisor = supervisor(&root, 1);
        let request = order(
            &root,
            "command-unsupported-effort",
            "89000000-0000-4000-8000-000000000202",
        );
        assert!(matches!(
            rejection(
                submit_spawn(&supervisor, &registry, &source(&root), &request, NOW, false, &hosts_fixture())
                    .unwrap()
            ),
            SpawnRefusal::UnsupportedCapability { ref capability, .. }
                if capability == "effort 'high' accepté par le modèle"
        ));

        let resolved = registry.resolved_definition("fixture").unwrap();
        let candidate = RecoveryCandidate {
            lease: SpawnLease {
                command_id: "recovery-unsupported-effort".to_string(),
                name: "89000000-0000-4000-8000-000000000202".to_string(),
                instance_id: "instance-recovery".to_string(),
                generation: 1,
                deadline_at: NOW + 10,
                persistent: true,
                project: None,
                link_id: None,
                ownership: None,
                agent_path: None,
                remote_placement: None,
            },
            agent_type: "fixture".to_string(),
            cwd: root.clone(),
            resolved_definition: Some(resolved),
            runtime_execution: None,
        };
        assert!(matches!(
            prepare_recovery(&source(&root), candidate),
            Err(SpawnRefusal::UnsupportedCapability { ref capability, .. })
                if capability == "effort 'high' accepté par le modèle"
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn matrice_sc003_couvre_les_onze_familles_sans_residu_operationnel() {
        // Huit refus indépendants ; NameActive et QuotaExceeded nécessitent
        // une génération témoin active et sont exercés plus bas.
        for (label, expected) in [
            (
                "unknown",
                SpawnRefusal::UnknownType {
                    requested_type: "absent".to_string(),
                    known_types: Vec::new(),
                    registry: "/tmp/t905-agents.json".to_string(),
                },
            ),
            (
                "missing",
                SpawnRefusal::CommandMissing {
                    command: "/commande/introuvable".to_string(),
                    registry: "/tmp/t905-agents.json".to_string(),
                },
            ),
            (
                "billing",
                SpawnRefusal::BillingGuard {
                    variable: "API_KEY".to_string(),
                },
            ),
            (
                "env",
                SpawnRefusal::EnvUnfit {
                    detail: "HOME absent de l'environnement du daemon".to_string(),
                },
            ),
            (
                "cwd",
                SpawnRefusal::CwdGone {
                    searched_on: "monordinateur".to_string(),
                    requested_from: "cartae".to_string(),
                },
            ),
            (
                "negotiation",
                SpawnRefusal::UnsupportedCapability {
                    agent_type: "fixture".to_string(),
                    model: "<non déclaré>".to_string(),
                    capability: "chemin d'exécution 'tmux'".to_string(),
                },
            ),
            ("timeout", SpawnRefusal::SpawnTimeout),
            ("expired", SpawnRefusal::IdempotencyExpired),
        ] {
            let root = root(label);
            fs::create_dir_all(&root).unwrap();
            let supervisor = supervisor(&root, 2);
            let mut request = order(
                &root,
                &format!("command-{label}"),
                "89000000-0000-4000-8000-000000000203",
            );
            let mut env = source(&root);
            let registry = match label {
                "unknown" => {
                    request.agent_type = "absent".to_string();
                    registry("/bin/sh", "acp", &[])
                }
                "missing" => registry("/commande/introuvable", "acp", &[]),
                "billing" => {
                    env.insert("API_KEY".to_string(), OsString::from("secret"));
                    registry("/bin/sh", "acp", &["API_KEY"])
                }
                "env" => {
                    env.remove("HOME");
                    registry("/bin/sh", "acp", &[])
                }
                "cwd" => {
                    request.cwd = root.join("disparu");
                    registry("/bin/sh", "acp", &[])
                }
                "negotiation" => registry("/bin/sh", "tmux", &[]),
                "timeout" => {
                    request.issued_at = NOW - 2;
                    request.deadline_at = NOW - 1;
                    registry("/bin/sh", "acp", &[])
                }
                "expired" => {
                    request.issued_at = NOW - 400;
                    request.deadline_at = NOW - 390;
                    registry("/bin/sh", "acp", &[])
                }
                _ => unreachable!(),
            };
            let mut expected = expected;
            if let SpawnRefusal::UnknownType { known_types, .. } = &mut expected {
                // La composition du catalogue appartient au registre. Cette
                // matrice vérifie que lifecycle relaie son instantané exact,
                // sans recopier une liste native qui deviendrait périmée.
                *known_types = registry.known_types();
            }
            let actual = rejection(
                submit_spawn(
                    &supervisor,
                    &registry,
                    &env,
                    &request,
                    NOW,
                    false,
                    &hosts_fixture(),
                )
                .unwrap(),
            );
            assert_eq!(actual, expected, "famille {label}");
            let replay = rejection(
                submit_spawn(
                    &supervisor,
                    &registry,
                    &env,
                    &request,
                    NOW,
                    false,
                    &hosts_fixture(),
                )
                .unwrap(),
            );
            assert_eq!(replay, expected, "rejeu divergent pour {label}");
            assert_eq!(supervisor.active_count(), 0, "résidu actif pour {label}");
            let _ = fs::remove_dir_all(root);
        }

        let nq_root = root("name-quota");
        fs::create_dir_all(&nq_root).unwrap();
        let nq_supervisor = supervisor(&nq_root, 1);
        let nq_registry = registry("/bin/sh", "acp", &[]);
        let env = source(&nq_root);
        assert!(matches!(
            submit_spawn(
                &nq_supervisor,
                &nq_registry,
                &env,
                &order(
                    &nq_root,
                    "command-first",
                    "89000000-0000-4000-8000-000000000203"
                ),
                NOW,
                false,
                &hosts_fixture(),
            )
            .unwrap(),
            SpawnDecision::Ready(_)
        ));
        assert_eq!(nq_supervisor.active_count(), 1);
        assert_eq!(
            rejection(
                submit_spawn(
                    &nq_supervisor,
                    &nq_registry,
                    &env,
                    &order(
                        &nq_root,
                        "command-name",
                        "89000000-0000-4000-8000-000000000203"
                    ),
                    NOW,
                    false,
                    &hosts_fixture(),
                )
                .unwrap()
            ),
            SpawnRefusal::NameActive
        );
        assert_eq!(nq_supervisor.active_count(), 1);
        assert_eq!(
            rejection(
                submit_spawn(
                    &nq_supervisor,
                    &nq_registry,
                    &env,
                    &order(
                        &nq_root,
                        "command-quota",
                        "89000000-0000-4000-8000-000000000204"
                    ),
                    NOW,
                    false,
                    &hosts_fixture(),
                )
                .unwrap()
            ),
            SpawnRefusal::QuotaExceeded { limit: 1 }
        );
        assert_eq!(nq_supervisor.active_count(), 1);
        let _ = fs::remove_dir_all(nq_root);

        let root = root("recovering");
        fs::create_dir_all(&root).unwrap();
        let supervisor = supervisor(&root, 1);
        let registry = registry("/bin/sh", "acp", &[]);
        assert_eq!(
            rejection(
                submit_spawn(
                    &supervisor,
                    &registry,
                    &source(&root),
                    &order(
                        &root,
                        "command-recovering",
                        "89000000-0000-4000-8000-000000000205"
                    ),
                    NOW,
                    true,
                    &hosts_fixture(),
                )
                .unwrap()
            ),
            SpawnRefusal::DaemonRecovering
        );
        assert_eq!(supervisor.active_count(), 0);
        assert!(!supervisor.knows_command("command-recovering"));

        let known = order(
            &root,
            "command-known-before-recovery",
            "89000000-0000-4000-8000-000000000206",
        );
        assert!(matches!(
            submit_spawn(
                &supervisor,
                &registry,
                &source(&root),
                &known,
                NOW,
                false,
                &hosts_fixture()
            )
            .unwrap(),
            SpawnDecision::Ready(_)
        ));
        assert!(matches!(
            submit_spawn(
                &supervisor,
                &registry,
                &source(&root),
                &known,
                NOW,
                true,
                &hosts_fixture()
            )
            .unwrap(),
            SpawnDecision::Await(_)
        ));
        let _ = fs::remove_dir_all(root);
    }

    /// POINT 5 — le refus doit dire OÙ le daemon a cherché et QUI a demandé.
    ///
    /// Contrôle positif d'abord : un `cwd` existant est accepté, donc l'absence
    /// de refus ci-dessous n'est pas le silence d'une garde morte.
    ///
    /// Mutant qui tue ce test : recopier `hosts.searched_on` dans les deux
    /// champs → l'assertion sur `requested_from == "cartae"` meurt. Un oracle
    /// qui vérifierait seulement que les champs sont non vides survivrait.
    #[test]
    fn le_refus_de_cwd_nomme_la_machine_cherchee_et_la_machine_demandeuse() {
        let root = root("cwd-attribution");
        fs::create_dir_all(&root).unwrap();
        let supervisor = supervisor(&root, 4);
        let registry = registry("/bin/sh", "acp", &[]);
        let env = source(&root);

        // Contrôle positif : le répertoire existe, la garde laisse passer.
        assert!(matches!(
            submit_spawn(
                &supervisor,
                &registry,
                &env,
                &order(&root, "cwd-present", "89000000-0000-4000-8000-000000000207"),
                NOW,
                false,
                &hosts_fixture(),
            )
            .unwrap(),
            SpawnDecision::Ready(_)
        ));

        // Répertoire absent : le refus porte les DEUX machines.
        let mut absent = order(&root, "cwd-absent", "89000000-0000-4000-8000-000000000208");
        absent.cwd = root.join("repertoire-qui-n-existe-pas");
        let refusal = rejection(
            submit_spawn(
                &supervisor,
                &registry,
                &env,
                &absent,
                NOW,
                false,
                &hosts_fixture(),
            )
            .unwrap(),
        );
        match refusal {
            SpawnRefusal::CwdGone {
                searched_on,
                requested_from,
            } => {
                assert_eq!(searched_on, "monordinateur", "machine où l'on a cherché");
                assert_eq!(requested_from, "cartae", "machine qui a demandé");
            }
            other => panic!("refus attendu CwdGone, obtenu {other:?}"),
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn core_089_projet_refuse_avant_reservation_et_reparable_au_meme_id() {
        let root = root("removed-project");
        let supervisor = supervisor(&root, 2);
        let registry = registry("/bin/sh", "acp", &[]);
        let mut spawn = order(
            &root,
            "core-089-project",
            "00000089-0000-4000-8000-000000000001",
        );
        spawn.project = Some(bridget_transport::protocol::ProjectReference {
            project_id: "historical-project".to_string(),
            binding_generation: 7,
        });
        let refusal = rejection(
            submit_spawn(
                &supervisor,
                &registry,
                &source(&root),
                &spawn,
                NOW,
                false,
                &hosts_fixture(),
            )
            .unwrap(),
        );
        assert_eq!(
            refusal,
            SpawnRefusal::DockerRuntimeUnavailable {
                project_id: "historical-project".to_string(),
            }
        );
        // Mutation : une garde après request_spawn laisse ici une clé connue.
        assert!(!supervisor.knows_command(&spawn.command_id));
        assert_eq!(supervisor.active_count(), 0);
        assert!(supervisor.desired_fleet().unwrap().equipiers.is_empty());

        // Changement EXPLICITE du demandeur, pas une suppression automatique.
        spawn.project = None;
        assert!(matches!(
            submit_spawn(
                &supervisor,
                &registry,
                &source(&root),
                &spawn,
                NOW,
                false,
                &hosts_fixture()
            )
            .unwrap(),
            SpawnDecision::Ready(_)
        ));
        drop(supervisor);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn core_089_reprise_refuse_le_runtime_meme_sans_reference_projet() {
        use crate::desired_state::{ContainerAgentExecution, ContainerAgentExecutionState};
        let root = root("removed-recovery");
        let registry = registry("/bin/sh", "acp", &[]);
        let mut candidate = RecoveryCandidate {
            lease: SpawnLease {
                command_id: "historical-recovery".to_string(),
                name: "00000089-0000-4000-8000-000000000002".to_string(),
                instance_id: "historical-instance".to_string(),
                generation: 7,
                deadline_at: NOW + 10,
                persistent: true,
                project: None,
                link_id: None,
                ownership: None,
                agent_path: None,
                remote_placement: None,
            },
            agent_type: "fixture".to_string(),
            cwd: root.clone(),
            resolved_definition: Some(registry.resolved_definition("fixture").unwrap()),
            runtime_execution: Some(ContainerAgentExecution {
                agent_instance_id: "historical-instance".to_string(),
                generation: 7,
                project_id: "orphaned-project".to_string(),
                binding_generation: 4,
                environment_epoch: 1,
                container_id: "a".repeat(64),
                exec_id: uuid::Uuid::new_v4().to_string(),
                cwd: root.clone(),
                state: ContainerAgentExecutionState::Running,
                provider_identity: "fixture".to_string(),
            }),
        };
        // Le cwd n'existe même pas : la métadonnée interdit le lanceur hôte
        // avant toute tentative de récupération de l'environnement.
        assert_eq!(
            prepare_recovery(&source(&root), candidate.clone()).unwrap_err(),
            SpawnRefusal::DockerRuntimeUnavailable {
                project_id: "orphaned-project".to_string()
            }
        );
        candidate.runtime_execution = None;
        candidate.lease.project = Some(bridget_transport::protocol::ProjectReference {
            project_id: "project-only".to_string(),
            binding_generation: 5,
        });
        assert_eq!(
            prepare_recovery(&source(&root), candidate).unwrap_err(),
            SpawnRefusal::DockerRuntimeUnavailable {
                project_id: "project-only".to_string()
            }
        );
        assert!(!root.exists());
    }

    #[test]
    fn core_089_rejeu_terminal_historique_ne_relance_ni_ne_reecrit_le_resultat() {
        let root = root("historical-replay");
        let supervisor = supervisor(&root, 2);
        let registry = registry("/bin/sh", "acp", &[]);
        let mut spawn = order(
            &root,
            "historical-project-terminal",
            "00000089-0000-4000-8000-000000000003",
        );
        spawn.project = Some(bridget_transport::protocol::ProjectReference {
            project_id: "archived-project".to_string(),
            binding_generation: 3,
        });
        // Reconstitution d'une issue écrite par l'ancien produit, au niveau
        // du store/superviseur. Aucun moteur ni processus n'est lancé.
        let SpawnSubmission::Start(lease) = supervisor.request_spawn(&spawn, NOW).unwrap() else {
            panic!("réservation historique attendue");
        };
        let definition = registry.resolved_definition("fixture").unwrap();
        supervisor.mark_starting(&lease, NOW, &definition).unwrap();
        supervisor
            .register_connected(&lease, &lease.instance_id, NOW)
            .unwrap();
        drop(supervisor);
        let supervisor = super::tests::supervisor(&root, 2);
        let before = fs::read(root.join("fleet.json")).unwrap();
        assert!(
            matches!(submit_spawn(&supervisor, &registry, &source(&root),
            &spawn, NOW + 1, false, &hosts_fixture()).unwrap(),
            SpawnDecision::Accepted { definition: Some(ref saved), .. } if saved == &definition)
        );
        assert_eq!(supervisor.active_count(), 0);
        assert_eq!(fs::read(root.join("fleet.json")).unwrap(), before);
        // Même clé, canon changé : ne contourne ni le rejet ni l'idempotence.
        spawn.project = None;
        assert!(matches!(
            submit_spawn(
                &supervisor,
                &registry,
                &source(&root),
                &spawn,
                NOW + 1,
                false,
                &hosts_fixture()
            )
            .unwrap(),
            SpawnDecision::EnvelopeMismatch
        ));
        assert_eq!(fs::read(root.join("fleet.json")).unwrap(), before);
        drop(supervisor);
        fs::remove_dir_all(root).unwrap();
    }
