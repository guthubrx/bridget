    use super::*;
    use bridget_transport::protocol::{REVIEW_DELEGATE_CONTRACT_VERSION, SERVICE_CONTRACT_VERSION};
    use rusqlite::params;
    use std::os::unix::net::UnixListener;

    fn temporary_socket_path() -> std::path::PathBuf {
        crate::mcp_identity::mock_socket("cli-canon")
    }

    pub(crate) fn caller_identity() -> crate::mcp_identity::ResolvedIdentity {
        crate::mcp_identity::ResolvedIdentity {
            name: "89000000-0000-4000-8000-000000000200".into(),
            instance_id: "test-cli-instance".into(),
            delegated_origin: None,
        }
    }

    pub(crate) fn owner_connection(path: &Path) -> DaemonConnection {
        let identity = caller_identity();
        let mut owner = DaemonConnection::connect(path).unwrap();
        let mut registration = cli_register("fixture");
        if let WrapperToDaemon::Register {
            agent_id,
            agent_type,
            instance_id,
            ..
        } = &mut registration
        {
            *agent_id = identity.name.clone();
            *agent_type = "fixture".into();
            *instance_id = Some(identity.instance_id.clone());
        }
        let DaemonToWrapper::Registered {
            credential: Some(credential),
            ..
        } = owner.exchange(&registration).unwrap()
        else {
            panic!("credential propriétaire requis");
        };
        crate::mcp_identity::save_credential(
            path.parent().unwrap(),
            &identity.name,
            &identity.instance_id,
            credential,
        )
        .unwrap();
        owner
    }

    pub(crate) fn start_real_daemon() -> (std::path::PathBuf, std::path::PathBuf) {
        // Le namespace du daemon est privé : ne jamais valider/réutiliser
        // l'ensemble du répertoire temporaire partagé comme son état.
        let root = PathBuf::from("/tmp").join(format!("bgcanon-{}", uuid::Uuid::new_v4().simple()));
        crate::environment::ensure_private_directory(&root).unwrap();
        let socket_path = root.join("bridget.sock");
        let db_path = root.join("bridget.db");
        let config = DaemonConfig {
            socket_path: socket_path.clone(),
            db_path: db_path.clone(),
            log_path: db_path.with_extension("log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        };
        std::thread::spawn(move || {
            let _ = daemon::run(config);
        });
        let deadline = std::time::Instant::now() + crate::communication::client::DAEMON_BUDGET;
        while std::time::Instant::now() < deadline {
            if socket_path.exists() {
                return (socket_path, db_path);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("daemon réel T1208 non démarré dans le budget privé de test");
    }

    fn stored_canonical_bytes(
        db_path: &std::path::Path,
        options: &IdempotentSendOptions,
    ) -> Vec<u8> {
        let connection = rusqlite::Connection::open(db_path).unwrap();
        connection
            .query_row(
                "SELECT canonical_bytes FROM idempotency_records
                 WHERE issuer_scope = ?1 AND operation_kind = 'send' AND idempotency_key = ?2",
                params![options.issuer_scope, options.id],
                |row| row.get(0),
            )
            .unwrap()
    }

    fn idempotency_issue(response: DaemonToWrapper) -> IdempotencyIssue {
        match response {
            DaemonToWrapper::IdempotencyResult { issue, .. } => issue,
            unexpected => panic!("issue idempotente attendue : {unexpected:?}"),
        }
    }

    fn write_response(writer: &mut BufWriter<UnixStream>, response: DaemonToWrapper) {
        writeln!(writer, "{}", encode(&response).unwrap()).unwrap();
        writer.flush().unwrap();
    }

    fn start_reference_server(
        path: std::path::PathBuf,
        expected_connections: usize,
    ) -> std::thread::JoinHandle<Vec<String>> {
        let listener = UnixListener::bind(&path).unwrap();
        std::thread::spawn(move || {
            let mut sends = Vec::new();
            let mut first_send: Option<String> = None;
            for _ in 0..expected_connections {
                let (stream, _) = listener.accept().unwrap();
                let read_stream = stream.try_clone().unwrap();
                let mut reader = BufReader::new(read_stream);
                let mut writer = BufWriter::new(stream);

                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                assert!(matches!(
                    decode::<WrapperToDaemon>(line.trim_end()).unwrap(),
                    WrapperToDaemon::RoleHandshake {
                        role: ConnectionRole::Client
                    }
                ));
                write_response(
                    &mut writer,
                    DaemonToWrapper::RoleAccepted {
                        role: ConnectionRole::Client,
                    },
                );

                line.clear();
                reader.read_line(&mut line).unwrap();
                let WrapperToDaemon::RegisterAuxiliary {
                    agent_id,
                    instance_id,
                    credential,
                } = decode(line.trim()).unwrap()
                else {
                    panic!("preuve auxiliaire attendue");
                };
                let expected =
                    crate::mcp_identity::auxiliary_registration(&agent_id, &instance_id, &path)
                        .unwrap();
                assert!(
                    matches!(expected, WrapperToDaemon::RegisterAuxiliary { credential: expected, .. } if expected == credential)
                );
                write_response(
                    &mut writer,
                    DaemonToWrapper::Registered {
                        agent_id,
                        credential: None,
                    },
                );
                line.clear();
                reader.read_line(&mut line).unwrap();
                assert!(matches!(
                    decode::<WrapperToDaemon>(line.trim_end()).unwrap(),
                    WrapperToDaemon::ClientHello {
                        contract_version: CLIENT_CONTRACT_VERSION,
                        capabilities,
                        ..
                    } if capabilities == vec![ClientCapability::SendIdempotent]
                ));
                write_response(
                    &mut writer,
                    DaemonToWrapper::ClientWelcome {
                        version: CLIENT_CONTRACT_VERSION,
                        build_id: "test-build".to_string(),
                        horizon_secs: 300,
                        issued_at_tolerance_secs: 5,
                        capabilities: vec![ClientCapability::SendIdempotent],
                    },
                );

                line.clear();
                reader.read_line(&mut line).unwrap();
                let command: WrapperToDaemon = decode(line.trim_end()).unwrap();
                let serialized = encode(&command).unwrap();
                let issue = if first_send
                    .as_ref()
                    .is_some_and(|first| first != &serialized)
                {
                    IdempotencyIssue::EnvelopeMismatch
                } else {
                    first_send.get_or_insert_with(|| serialized.clone());
                    IdempotencyIssue::Accepted {
                        expires_at: 123_456,
                    }
                };
                sends.push(serialized);
                write_response(
                    &mut writer,
                    DaemonToWrapper::IdempotencyResult {
                        operation_kind: "send".to_string(),
                        idempotency_key: "message-t1208".to_string(),
                        issue,
                        project_warnings: Vec::new(),
                    },
                );
            }
            std::fs::remove_file(path).unwrap();
            sends
        })
    }

    /// Client de référence volontairement indépendant de la projection CLI :
    /// il déroule les trois étapes publiées du contrat sur le socket.
    fn reference_client_send(
        path: &std::path::Path,
        message: &BridgetMessage,
        options: &IdempotentSendOptions,
    ) -> DaemonToWrapper {
        let stream = UnixStream::connect(path).unwrap();
        let read_stream = stream.try_clone().unwrap();
        let mut writer = BufWriter::new(stream);
        let mut reader = BufReader::new(read_stream);

        write_control_message(
            &mut writer,
            &WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client,
            },
        )
        .unwrap();
        assert!(matches!(
            read_control_message(&mut reader).unwrap(),
            DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Client
            }
        ));
        write_control_message(
            &mut writer,
            &crate::mcp_identity::auxiliary_registration(
                &message.from,
                &caller_identity().instance_id,
                path,
            )
            .unwrap(),
        )
        .unwrap();
        assert!(matches!(
            read_control_message(&mut reader).unwrap(),
            DaemonToWrapper::Registered { .. }
        ));
        write_control_message(
            &mut writer,
            &WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: options.issuer_scope.clone(),
                capabilities: vec![ClientCapability::SendIdempotent],
            },
        )
        .unwrap();
        assert!(matches!(
            read_control_message(&mut reader).unwrap(),
            DaemonToWrapper::ClientWelcome { .. }
        ));
        write_control_message(
            &mut writer,
            &WrapperToDaemon::SendIdempotent {
                message: message.clone(),
                message_id: options.id.clone(),
                issued_at: options.issued_at,
            },
        )
        .unwrap();
        read_control_message(&mut reader).unwrap()
    }

    fn example_options() -> IdempotentSendOptions {
        IdempotentSendOptions {
            id: "message-t1208".to_string(),
            issued_at: 123_000,
            issuer_scope: "012_scope_aaaaaaaaaaaa".to_string(),
        }
    }

    #[test]
    fn options_idempotentes_sont_obligatoires_ensemble() {
        assert_eq!(idempotent_options(None, None, None).unwrap(), None);
        for (id, issued_at, issuer_scope) in [
            (Some("id".to_string()), None, None),
            (None, Some("123".to_string()), None),
            (None, None, Some("scope".to_string())),
            (Some("id".to_string()), Some("123".to_string()), None),
            (Some("id".to_string()), None, Some("scope".to_string())),
            (None, Some("123".to_string()), Some("scope".to_string())),
        ] {
            assert!(idempotent_options(id, issued_at, issuer_scope).is_err());
        }
        assert!(
            idempotent_options(
                Some("id".to_string()),
                Some("pas-un-instant".to_string()),
                Some("scope".to_string())
            )
            .is_err()
        );
    }

    #[test]
    fn depot_guichet_rejoue_le_canon_ferme_avec_les_trois_cles() {
        let args = vec![
            "deposer",
            "delivery-report",
            "--from",
            "89000000-0000-4000-8000-000000000011",
            "--objective",
            "objective-1",
            "--delegation",
            "delegation-1",
            "--hash",
            "aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899",
            "--in-reply-to",
            "message-1",
            "--id",
            "deposit-1",
            "--issued-at",
            "1787500000",
            "--issuer-scope",
            "015_scope_0123456789abcdef0123456789abcdef",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        let first = parse_guichet_deposit(&args).unwrap();
        let second = parse_guichet_deposit(&args).unwrap();
        assert_eq!(encode(&first).unwrap(), encode(&second).unwrap());
        assert!(matches!(
            first,
            WrapperToDaemon::ServiceRequest {
                version: SERVICE_CONTRACT_VERSION,
                request_id,
                issued_at: 1_787_500_000,
                operation: ServiceRequestOperation::DeliveryReport,
                payload: ServiceRequestPayload::DeliveryReport { in_reply_to, .. },
                ..
            } if request_id == "deposit-1" && in_reply_to == "message-1"
        ));
    }

    #[test]
    fn depot_guichet_parse_les_trois_mutations_sans_chemin_ni_principal_annexe() {
        let cases = [
            (
                vec![
                    "deposer",
                    "delegate",
                    "--from",
                    "89000000-0000-4000-8000-000000000017",
                    "--goal",
                    "lot central",
                    "--target",
                    "89000000-0000-4000-8000-000000000018",
                    "--duration",
                    "courte",
                ],
                ServiceRequestOperation::Delegate,
            ),
            (
                vec![
                    "deposer",
                    "registre-add",
                    "--from",
                    "89000000-0000-4000-8000-000000000017",
                    "--line",
                    "kind=add id=constat-1",
                ],
                ServiceRequestOperation::RegistreAdd,
            ),
            (
                vec![
                    "deposer",
                    "objective-close",
                    "--from",
                    "89000000-0000-4000-8000-000000000017",
                    "--objective",
                    "objective-1",
                    "--reason",
                    "objectif atteint",
                ],
                ServiceRequestOperation::ObjectiveClose,
            ),
        ];
        for (args, expected_operation) in cases {
            let args = args.into_iter().map(str::to_string).collect::<Vec<_>>();
            assert!(matches!(
                parse_guichet_deposit(&args).unwrap(),
                WrapperToDaemon::ServiceRequest { operation, .. }
                    if operation == expected_operation
            ));
        }
        for forbidden in ["--database", "--journal", "--principal"] {
            let args = [
                "deposer",
                "registre-add",
                "--from",
                "89000000-0000-4000-8000-000000000017",
                "--line",
                "kind=add id=constat-1",
                forbidden,
                "/tmp/forged",
            ]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
            assert!(parse_guichet_deposit(&args).is_err());
        }
    }

    #[test]
    fn depot_delegate_versionne_atomiquement_sa_cible_de_revue() {
        let ordinary = [
            "deposer",
            "delegate",
            "--from",
            "89000000-0000-4000-8000-000000000017",
            "--goal",
            "relire le lot",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        assert!(matches!(
            parse_guichet_deposit(&ordinary).unwrap(),
            WrapperToDaemon::ServiceRequest {
                version: SERVICE_CONTRACT_VERSION,
                payload: ServiceRequestPayload::Delegate {
                    review_target: None,
                    ..
                },
                ..
            }
        ));

        let targeted = [
            "deposer",
            "delegate",
            "--from",
            "89000000-0000-4000-8000-000000000017",
            "--goal",
            "relire le lot",
            "--review-ref",
            "origin/session-047-verdict-tete-reecrite",
            "--expected-head",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        assert!(matches!(
            parse_guichet_deposit(&targeted).unwrap(),
            WrapperToDaemon::ServiceRequest {
                version: REVIEW_DELEGATE_CONTRACT_VERSION,
                payload: ServiceRequestPayload::Delegate {
                    review_target: Some(ReviewTarget { target_ref, expected_head }),
                    ..
                },
                ..
            } if target_ref == "origin/session-047-verdict-tete-reecrite"
                && expected_head == "a".repeat(40)
        ));

        for missing in ["--review-ref", "--expected-head"] {
            let mut incomplete = targeted.clone();
            let position = incomplete
                .iter()
                .position(|value| value == missing)
                .unwrap();
            incomplete.drain(position..=position + 1);
            assert!(
                parse_guichet_deposit(&incomplete).is_err(),
                "option isolée acceptée : {missing}"
            );
        }
    }

    #[test]
    fn depot_guichet_refuse_les_formes_ouvertes_ou_incompletes() {
        let hash = "0".repeat(64);
        let missing_link = vec![
            "deposer",
            "delivery-report",
            "--from",
            "codex-1",
            "--objective",
            "objective-1",
            "--delegation",
            "delegation-1",
            "--hash",
            hash.as_str(),
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        assert!(parse_guichet_deposit(&missing_link).is_err());
        let free_form = vec!["deposer", "message-libre", "--from", "codex-1"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        assert!(parse_guichet_deposit(&free_form).is_err());
    }

    #[test]
    fn projection_cli_et_client_reference_produisent_le_meme_envoi_et_le_meme_rejet() {
        let path = temporary_socket_path();
        let server = start_reference_server(path.clone(), 3);
        let options = example_options();
        let identity = caller_identity();
        crate::mcp_identity::mock_private_identity(&path, &identity.name, &identity.instance_id);
        let mut message = BridgetMessage::new(&identity.name, "codex-1", "bonjour");
        message.id = options.id.clone();

        let reference = reference_client_send(&path, &message, &options);
        let cli = send_idempotent_to_daemon_at(&path, &message, &options, Some(&caller_identity()))
            .unwrap();
        assert!(matches!(
            (reference, cli),
            (
                DaemonToWrapper::IdempotencyResult {
                    issue: IdempotencyIssue::Accepted { .. },
                    ..
                },
                DaemonToWrapper::IdempotencyResult {
                    issue: IdempotencyIssue::Accepted { .. },
                    ..
                }
            )
        ));

        let mut divergent = message.clone();
        divergent.body = "message différent".to_string();
        assert!(matches!(
            send_idempotent_to_daemon_at(&path, &divergent, &options, Some(&caller_identity()))
                .unwrap(),
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::EnvelopeMismatch,
                ..
            }
        ));

        let sends = server.join().unwrap();
        assert_eq!(sends[0], sends[1], "le canon publié est identique");
        assert_ne!(sends[0], sends[2]);
        assert!(sends[0].contains("bonjour"));
        assert!(sends[2].contains("message différent"));
    }

    #[test]
    fn projection_cli_et_reference_partagent_le_canon_du_daemon_reel() {
        let (socket_path, db_path) = start_real_daemon();
        let _owner = owner_connection(&socket_path);
        let issued_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let options = IdempotentSendOptions {
            id: "message-t1208-reel".to_string(),
            issued_at,
            issuer_scope: "012_scope_aaaaaaaaaaaa".to_string(),
        };
        let mut message =
            BridgetMessage::new(&caller_identity().name, "destinataire-absent", "bonjour");
        message.id = options.id.clone();
        message.reply = true;
        message.reply_timeout = Some(12);

        let reference = idempotency_issue(reference_client_send(&socket_path, &message, &options));
        let cli = idempotency_issue(
            send_idempotent_to_daemon_at(
                &socket_path,
                &message,
                &options,
                Some(&caller_identity()),
            )
            .unwrap(),
        );
        assert_eq!(reference, cli, "référence et CLI rejouent la même issue");
        let canonical = stored_canonical_bytes(&db_path, &options);

        let mut divergences = Vec::new();
        let mut body = message.clone();
        body.body = "bonjour divergent".to_string();
        divergences.push(body);
        let mut target = message.clone();
        target.to = "autre-destinataire".to_string();
        divergences.push(target);
        let mut reply = message.clone();
        reply.reply = false;
        divergences.push(reply);
        let mut deadline = message.clone();
        deadline.deadline_at = Some((issued_at + 13) as u64);
        divergences.push(deadline);

        for divergent in divergences {
            assert_eq!(
                idempotency_issue(
                    send_idempotent_to_daemon_at(
                        &socket_path,
                        &divergent,
                        &options,
                        Some(&caller_identity())
                    )
                    .unwrap(),
                ),
                IdempotencyIssue::EnvelopeMismatch,
            );
            assert_eq!(
                stored_canonical_bytes(&db_path, &options),
                canonical,
                "EnvelopeMismatch ne doit jamais modifier le record initial",
            );
        }
    }

    /// ORACLE — l'annuaire distingue trois réponses à « survit au redémarrage ? » :
    /// oui, non, et « nul ne l'atteste ».
    ///
    /// Le troisième cas n'est pas un non. Le 28/08, lire un `0` là où la
    /// question ne se posait pas a failli faire relancer cinq agents sains,
    /// donc détruire leur contexte. La ligne rendue doit porter la réponse,
    /// pas seulement l'en-tête.
    #[test]
    fn who_distingue_persistant_ephemere_et_non_atteste() {
        let agent = |name: &str, persistent: Option<bool>| AgentInfo {
            agent_id: uuid::Uuid::new_v4().to_string(),
            display_name: name.to_string(),
            agent_type: "fixture".to_string(),
            connection_id: format!("conn-{name}"),
            host: "local".to_string(),
            transport: "claude_stream_json".to_string(),
            channel: Some("ssh-unix".to_string()),
            mode: Some(PresenceMode::Cli),
            location: None,
            os: "Linux".to_string(),
            state: "connected".to_string(),
            last_seen_secs: 0,
            reconnect_count: 0,
            domain: Some("bridget".to_string()),
            model: None,
            effort: None,
            rate_limits: vec![],
            model_mismatch: None,
            disk_space: None,
            provider: None,
            persistent,
            agent_link: None,
            execution: None,
        };
        let rendered = render_who(
            &[
                agent("survivant", Some(true)),
                agent("ephemere", Some(false)),
                agent("hors-flotte", None),
            ],
            None,
        );

        assert!(rendered.contains("PERSIST"), "colonne absente de l'en-tête");
        let ligne = |nom: &str| {
            rendered
                .lines()
                .find(|line| line.contains(nom))
                .unwrap_or_else(|| panic!("ligne {nom} absente"))
                .to_string()
        };
        assert!(
            ligne("survivant").contains("  oui  "),
            "un agent persistant doit se lire « oui » : {}",
            ligne("survivant")
        );
        assert!(
            ligne("ephemere").contains("  non  "),
            "un agent qui sera drainé doit se lire « non » : {}",
            ligne("ephemere")
        );
        assert!(
            !ligne("hors-flotte").contains("non"),
            "une persistance non attestée ne doit jamais se lire « non » : {}",
            ligne("hors-flotte")
        );
        assert!(
            ligne("hors-flotte").contains('—'),
            "une persistance non attestée doit se lire « — » : {}",
            ligne("hors-flotte")
        );
    }

    /// ORACLE — la sortie machine publie toujours la clé, `null` compris.
    /// Une ronde doit pouvoir distinguer « indéterminable » d'un daemon trop
    /// ancien pour connaître le champ ; un champ escamoté rendrait les deux
    /// cas identiques et ramènerait le détour par SQL.
    #[test]
    fn json_publie_la_persistance_meme_indeterminee() {
        let agent = |persistent: Option<bool>| AgentInfo {
            agent_id: uuid::Uuid::new_v4().to_string(),
            display_name: "jc1-flux".to_string(),
            agent_type: "claude".to_string(),
            connection_id: "conn-182".to_string(),
            host: "cartae".to_string(),
            transport: "claude_stream_json".to_string(),
            channel: Some("ssh-unix".to_string()),
            mode: Some(PresenceMode::Cli),
            location: None,
            os: "Linux".to_string(),
            state: "connected".to_string(),
            last_seen_secs: 0,
            reconnect_count: 1,
            domain: Some("bridget".to_string()),
            model: None,
            effort: None,
            rate_limits: vec![],
            model_mismatch: None,
            disk_space: None,
            provider: None,
            agent_link: None,
            persistent,
            execution: None,
        };

        let atteste = serde_json::to_value(agent(Some(true))).unwrap();
        assert_eq!(
            atteste.get("persistent"),
            Some(&serde_json::Value::Bool(true))
        );
        let inconnu = serde_json::to_value(agent(None)).unwrap();
        assert_eq!(
            inconnu.get("persistent"),
            Some(&serde_json::Value::Null),
            "la clé doit rester présente et valoir null quand nul ne l'atteste"
        );

        // DTO d'identité courant sans champ optionnel persistent : on ne
        // fabrique pas ce fait. Ce n'est pas une compatibilité nom-seul v0.
        let ancien: AgentInfo = serde_json::from_str(
            r#"{"agent_id":"89000000-0000-4000-8000-000000000011","display_name":"vieux","agent_type":"codex","connection_id":"c","host":"h",
                "transport":"tmux","os":"Linux","state":"connected",
                "last_seen_secs":0,"reconnect_count":0}"#,
        )
        .unwrap();
        assert_eq!(ancien.persistent, None);
        let mut sans_identite = serde_json::to_value(&ancien).unwrap();
        sans_identite.as_object_mut().unwrap().remove("agent_id");
        assert!(serde_json::from_value::<AgentInfo>(sans_identite).is_err());
    }

    #[test]
    fn who_affiche_mode_et_localisation_sans_dependre_d_un_tty() {
        let agent = |name: &str, mode: Option<PresenceMode>, location: Option<&str>| AgentInfo {
            agent_id: uuid::Uuid::new_v4().to_string(),
            display_name: name.to_string(),
            agent_type: "fixture".to_string(),
            connection_id: format!("conn-{name}"),
            host: "local".to_string(),
            transport: "tmux".to_string(),
            channel: Some("ssh-unix".to_string()),
            mode,
            location: location.map(str::to_string),
            os: "macOS".to_string(),
            state: "connected".to_string(),
            last_seen_secs: 0,
            reconnect_count: 0,
            domain: None,
            model: None,
            effort: None,
            rate_limits: vec![],
            model_mismatch: None,
            agent_link: None,
            disk_space: None,
            provider: None,
            persistent: None,
            execution: None,
        };
        let mut acp = agent("acp-gere", Some(PresenceMode::Acp), None);
        acp.disk_space = Some(bridget_transport::protocol::DiskSpaceFact {
            volume: "/".to_string(),
            free_bytes: 47_300_000_000,
            observed_at_unix: 1_788_000_000,
        });
        let rendered = render_who(
            &[
                acp,
                agent(
                    "tmux-interactif",
                    Some(PresenceMode::Tmux),
                    Some("bridget:4.2"),
                ),
                agent("cli-ephemere", Some(PresenceMode::Cli), None),
            ],
            None,
        );

        assert!(rendered.starts_with("Agents connectés :\n"));
        assert!(rendered.contains("MODE"));
        assert!(rendered.contains("CANAL"));
        assert!(rendered.contains("LOCALISATION"));
        assert!(rendered.contains("LIMITE"));
        assert!(rendered.contains("DISQUE"));
        assert!(rendered.contains("44.1 Gio libres"));
        assert!(rendered.contains("acp-gere"));
        assert!(rendered.contains("tmux-interactif"));
        assert!(rendered.contains("cli-ephemere"));
        assert!(rendered.contains("bridget:4.2"));
        assert!(rendered.contains("acp"));
        assert!(rendered.contains("tmux"));
        assert!(rendered.contains("ssh-unix"));
        assert!(rendered.contains("cli"));
        assert!(!rendered.contains('\u{1b}'));
    }

    #[test]
    fn who_rend_une_limite_compacte_par_fenetre_et_garde_l_absence() {
        let mut agent = AgentInfo {
            agent_id: uuid::Uuid::new_v4().to_string(),
            display_name: "claude-1".to_string(),
            agent_type: "claude".to_string(),
            connection_id: "conn-claude".to_string(),
            host: "local".to_string(),
            transport: "stdio".to_string(),
            channel: None,
            mode: Some(PresenceMode::Cli),
            location: None,
            os: "macOS".to_string(),
            state: "connected".to_string(),
            last_seen_secs: 0,
            reconnect_count: 0,
            domain: None,
            model: Some("claude-opus-5".to_string()),
            effort: None,
            rate_limits: vec![bridget_transport::protocol::RateLimitFact {
                credits: None,
                window: "five_hour".to_string(),
                status: "allowed".to_string(),
                resets_at: Some(1_787_572_200),
                used_percent: Some(19),
            }],
            agent_link: None,
            model_mismatch: None,
            disk_space: None,
            provider: None,
            persistent: None,
            execution: None,
        };
        let rendered = format_rate_limit(&agent);
        assert!(rendered.starts_with("5h 19% rst "), "{rendered}");
        assert!(!rendered.contains("seven_day"), "fenêtre absente = absente");

        agent
            .rate_limits
            .push(bridget_transport::protocol::RateLimitFact {
                credits: None,
                window: "seven_day".to_string(),
                status: "allowed".to_string(),
                resets_at: Some(1_787_700_000),
                used_percent: Some(61),
            });
        let both = format_rate_limit(&agent);
        assert!(both.contains("5h 19% rst "), "{both}");
        assert!(both.contains(" · "), "{both}");
        assert!(both.contains("7d 61% rst "), "{both}");

        agent.rate_limits = vec![bridget_transport::protocol::RateLimitFact {
            credits: None,
            window: "five_hour".to_string(),
            status: "rejected".to_string(),
            resets_at: None,
            used_percent: None,
        }];
        assert_eq!(format_rate_limit(&agent), "5h épuisée");

        agent.rate_limits.clear();
        assert_eq!(format_rate_limit(&agent), "—");

        assert_eq!(abbreviate_window("primary/300m"), "5h");
        assert_eq!(abbreviate_window("secondary/10080m"), "7d");
        assert_eq!(abbreviate_window("exotic_quota_xyz"), "exoticquotax");
        // Faux-ami : contains("five_hour") aurait menti ; frontière de jeton non.
        assert_ne!(abbreviate_window("not_five_hour_custom"), "5h");
        assert_eq!(
            abbreviate_window("not_five_hour_custom"),
            shorten_raw_window("not_five_hour_custom")
        );
        assert_eq!(abbreviate_window("five_hour"), "5h");
        assert_eq!(abbreviate_window("x/five_hour"), "5h");
    }

    #[test]
    fn who_marque_l_ecart_de_modele_et_reste_muet_sans_signal() {
        let mut agent = AgentInfo {
            agent_id: uuid::Uuid::new_v4().to_string(),
            display_name: "claude-1".to_string(),
            agent_type: "claude".to_string(),
            connection_id: "conn-claude".to_string(),
            host: "local".to_string(),
            transport: "stdio".to_string(),
            channel: None,
            mode: Some(PresenceMode::Cli),
            location: None,
            os: "macOS".to_string(),
            state: "connected".to_string(),
            last_seen_secs: 0,
            reconnect_count: 0,
            domain: None,
            model: Some("claude-opus-5".to_string()),
            effort: None,
            agent_link: None,
            rate_limits: vec![],
            model_mismatch: None,
            disk_space: None,
            provider: None,
            persistent: None,
            execution: None,
        };
        assert_eq!(format_model(&agent), "claude-opus-5");
        agent.model_mismatch = Some(bridget_transport::protocol::ModelMismatchFact {
            pinned: "claude-opus-5".to_string(),
            served: "claude-opus-4-6".to_string(),
        });
        let rendered = render_who(std::slice::from_ref(&agent), None);
        assert!(rendered.contains("claude-opus-4-6 ≠ claude-opus-5"));
        agent.model = None;
        agent.model_mismatch = None;
        assert_eq!(format_model(&agent), "—");
    }

    #[test]
    fn who_et_status_signalent_exactement_un_daemon_perime() {
        // Observe les identifiants et l'ATTRIBUTION — pas un libellé humain
        // (le gate fondateur a déjà payé ce piège cette nuit).
        let ici = crate::build_info::local_host();
        assert!(
            stale_daemon_warning_for_status(Some(crate::build_info::BUILD_ID), Some(&ici))
                .is_none()
        );
        let warning =
            stale_daemon_warning_for_status(Some("daemon-ancien"), Some("monordinateur")).unwrap();
        assert!(warning.contains("daemon-ancien"));
        assert!(warning.contains(crate::build_info::BUILD_ID));
        // La VALEUR portée est la machine du daemon, pas la mienne : un oracle
        // qui vérifierait seulement la présence d'un hôte passerait aussi avec
        // le mauvais.
        assert!(
            warning.contains("build du daemon différent sur monordinateur"),
            "{warning}"
        );
        // Machine non attestée : le message le dit, il ne suppose pas la mienne.
        let sans_hote = stale_daemon_warning_for_status(Some("daemon-ancien"), None).unwrap();
        assert!(
            sans_hote.contains(crate::build_info::MACHINE_NON_ATTESTEE),
            "{sans_hote}"
        );
        assert!(
            !sans_hote.contains(&format!("build du daemon différent sur {ici}")),
            "{sans_hote}"
        );
    }

    /// Daemon injoignable : aucune attestation, donc aucun chiffre.
    ///
    /// CE QUE CE TEST NE PROUVE PAS, et je le dis parce que je l'ai cru :
    /// il ne franchit PAS la garde de comptage. `get_status` sort dès que la
    /// socket est absente, bien avant elle. Le mutant qui retire la garde
    /// laisse ce test VERT. La garde elle-même est éprouvée par
    /// `daemon::attribution_tests::compter_n_est_permis_que_sur_la_base_attestee_par_le_daemon`.
    #[test]
    fn status_ne_compte_pas_les_messages_d_une_base_qui_n_est_pas_celle_du_daemon() {
        let config = DaemonConfig {
            socket_path: std::env::temp_dir()
                .join(format!("b89-status-{}.sock", uuid::Uuid::new_v4())),
            ..DaemonConfig::default()
        };
        // Daemon injoignable → aucune attestation, donc aucun compte.
        let status = daemon::get_status(&config).expect("statut hors ligne observable");
        assert!(
            status.message_count.is_none(),
            "sans attestation du daemon, aucun compte ne doit être rendu"
        );
        assert!(status.daemon_db_path.is_none());
        assert!(status.daemon_host.is_none());
    }
