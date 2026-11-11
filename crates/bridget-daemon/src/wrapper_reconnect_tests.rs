use bridget_transport::poison::RecoverPoison;
    use super::*;

    #[test]
    fn spec138_owner_announces_git_cwd_on_each_registration_without_consuming_frames() {
        let root = std::env::temp_dir().join(format!("s138-{}", uuid::Uuid::new_v4().simple()));
        crate::environment::ensure_private_directory(&root).unwrap();
        let socket = root.join("daemon.sock");
        let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
        let expected_root = std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let expected_host = host_name();
        let host = expected_host.clone();
        let server = std::thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let identity = match decode::<WrapperToDaemon>(line.trim()).unwrap() {
                    WrapperToDaemon::Register { agent_id, .. } => agent_id,
                    _ => panic!("Register attendu"),
                };
                writeln!(
                    stream,
                    "{}",
                    encode(&DaemonToWrapper::Registered {
                        agent_id: identity,
                        credential: None,
                    })
                    .unwrap()
                )
                .unwrap();
                writeln!(stream, "{}", encode(&DaemonToWrapper::Disconnect).unwrap()).unwrap();
                stream.flush().unwrap();
                let mut fact = None;
                for _ in 0..3 {
                    line.clear();
                    assert!(reader.read_line(&mut line).unwrap() > 0);
                    let value: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
                    if value["type"] == "CommunicationProjectFact" {
                        fact = Some(value);
                    }
                }
                let fact = fact.expect("fait Git propriétaire absent");
                assert_eq!(fact["root"], expected_root);
                assert_eq!(fact["host"], expected_host);
                assert_eq!(fact["source"], "git");
                assert!(fact["worktree_root"].is_null());
            }
        });
        for _ in 0..2 {
            let (mut reader, _writer, _) = connect_and_register_with_domain_at(
                &socket,
                "codex",
                None,
                &host,
                INTERACTIVE_AGENT_PROTOCOL,
                None,
                PresenceMode::Cli,
                None,
                "Linux",
                "instance138",
                false,
                false,
            )
            .unwrap();
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode::<DaemonToWrapper>(line.trim()).unwrap(),
                DaemonToWrapper::Disconnect
            ));
        }
        server.join().unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec_067_redaction_precede_le_sink_durable_des_sorties_structurees() {
        let secret = "S067_SYNTHETIC_SECRET";
        let root = mcp_test_root("redaction-structured");
        std::fs::create_dir_all(&root).unwrap();
        let (writer_stream, reader_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(Some(BufWriter::new(writer_stream))));
        let mut tracker = IdempotentDeliveryTracker::open_at(&root, "instance-redaction").unwrap();
        let mut redaction = Some(OutputRedactionLease::new(vec![secret.as_bytes().to_vec()]));
        assert!(!forward_managed_events_with_redaction(
            &writer,
            "agent-redaction",
            vec![ManagedEvent::source_line(
                bridget_transport::ManagedEventSource::ClaudeStreamJson,
                secret.as_bytes().to_vec(),
                ManagedEventKind::ModelObserved {
                    model: format!("model-{secret}"),
                },
            )],
            &mut tracker,
            &mut HashMap::new(),
            &mut redaction,
            None,
            true,
            None,
            None,
        ));
        let mut line = String::new();
        BufReader::new(reader_stream).read_line(&mut line).unwrap();
        assert!(!line.contains(secret));
        assert!(matches!(
            decode(line.trim_end()).unwrap(),
            WrapperToDaemon::ServedModel { model, .. } if model == "model-*********************"
        ));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec_067_redaction_couvre_fragment_et_canaux_distincts() {
        let secret = b"S067_SYNTHETIC_SECRET".to_vec();
        let mut lease = OutputRedactionLease::new(vec![secret.clone()]);
        let mut stdout = lease.redact("stdout", b"avant-S067_SYN", false);
        stdout.extend(lease.redact("stdout", b"THETIC_SECRET-apres", true));
        let stderr = lease.redact("stderr", &secret, true);
        assert!(!stdout.windows(secret.len()).any(|window| window == secret));
        assert!(!stderr.windows(secret.len()).any(|window| window == secret));
        assert!(stdout.contains(&42));
    }
    #[test]
    fn spec_024_canal_federe_prefere_la_nouvelle_cle_et_lit_l_alias() {
        assert_eq!(
            crate::connection_channel::attested_channel_from_sources(
                None,
                None,
                Some("transport=ssh-unix\n")
            ),
            Some("ssh-unix".to_string())
        );
        assert_eq!(
            crate::connection_channel::attested_channel_from_sources(
                None,
                None,
                Some("transport=ancien\nchannel=ssh-unix\n")
            ),
            Some("ssh-unix".to_string())
        );
        assert_eq!(
            crate::connection_channel::attested_channel_from_sources(
                None,
                None,
                Some("channel=   \ntransport=unix\n")
            ),
            Some("unix".to_string())
        );
    }

    #[test]
    fn spec_024_enregistrement_tmux_ecrit_protocole_et_canal_separes() {
        let socket = std::env::temp_dir().join(format!(
            "bridget-g11-register-{}.sock",
            uuid::Uuid::new_v4()
        ));
        let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
        let (release, stalled) = std::sync::mpsc::channel();
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode(line.trim()).unwrap(),
                WrapperToDaemon::Register {
                    transport: Some(protocol),
                    channel: ChannelReport::Known(channel),
                    mode: Some(PresenceMode::Tmux),
                    ..
                } if protocol == "tmux" && channel == "ssh-unix"
            ));
            let mut writer = BufWriter::new(stream);
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::Registered {
                    credential: None,
                    agent_id: "lab-agent".to_string()
                })
                .unwrap()
            )
            .unwrap();
            writer.flush().unwrap();

            line.clear();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode(line.trim()).unwrap(),
                WrapperToDaemon::DiskSpace { fact }
                    if fact.volume == "/"
                        && fact.free_bytes > 0
                        && fact.observed_at_unix > 0
            ));
            // Plus aucune lecture : le buffer noyau doit se remplir réellement.
            // Watchdog du pair, pas une attente de synchronisation arbitraire.
            let _ = stalled.recv_timeout(Duration::from_secs(5));
        });

        let (_, writer, name) = connect_and_register_at(ConnectAndRegisterAtArgs {
socket: &socket,
agent_type: "codex",
agent_id: Some("lab-agent"),
host: "lab-host",
protocol: INTERACTIVE_AGENT_PROTOCOL,
channel: Some("ssh-unix"),
mode: PresenceMode::Tmux,
location: Some("bridget:2.1"),
os: "Linux",
instance_id: "instance-lab",
domain: Some("bridget"),
turn_in_progress: false,
turn_attest: false,
})
        .unwrap();
        assert_eq!(name, "lab-agent");
        assert_eq!(
            writer.get_ref().write_timeout().unwrap(),
            Some(Duration::from_secs(1))
        );
        let writer = Arc::new(Mutex::new(Some(writer)));
        let started = Instant::now();
        send_wrapper_message(
            &writer,
            WrapperToDaemon::Send(bridget_core::BridgetMessage::new(
                "lab-agent",
                "peer",
                "x".repeat(4 * 1024 * 1024),
            )),
        );
        // Mutation : sans timeout le pair ne ferme qu'à son watchdog (5 s).
        // Sans invalidation, le flush de cleanup répéterait l'attente.
        // Borne sous le watchdog : la sérialisation de 4 Mio sous charge en
        // mode debug dépassait parfois 3 s, sans rien dire du délai d'écriture.
        assert!(started.elapsed() < Duration::from_millis(4500), "{:?}", started.elapsed());
        assert!(writer.lock().unwrap().is_none());
        release.send(()).unwrap();
        server.join().unwrap();
        let _ = std::fs::remove_file(socket);
    }

    #[test]
    fn contexte_tmux_exige_pane_et_localisation() {
        assert_eq!(
            parse_tmux_context("%42\tbridget:3.1\n"),
            Ok(("%42".to_string(), "bridget:3.1".to_string()))
        );
        assert!(parse_tmux_context("%42").is_err());
        assert!(parse_tmux_context("%42\t").is_err());
    }

    #[test]
    fn chemin_transcript_claude_suit_le_slug_du_cwd() {
        assert_eq!(
            claude_transcript_directory(
                Path::new("/home/fixture"),
                Path::new("/projet/.worktrees/session_014")
            ),
            PathBuf::from("/home/fixture/.claude/projects/-projet--worktrees-session-014")
        );
    }

    #[test]
    fn evenement_de_limite_claude_est_transmis_sans_transformation() {
        let root = mcp_test_root("rate-limit");
        std::fs::create_dir_all(&root).unwrap();
        let (writer_stream, reader_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(Some(BufWriter::new(writer_stream))));
        let mut tracker = IdempotentDeliveryTracker::open_at(&root, "instance-rate-limit").unwrap();
        let event = ManagedEvent::source_line(
            bridget_transport::ManagedEventSource::ClaudeStreamJson,
            br#"{"type":"rate_limit_event","rate_limit_info":{"status":"rejected","resetsAt":1787572200,"rateLimitType":"five_hour"}}"#.to_vec(),
            ManagedEventKind::RateLimitObserved {
                credits: None,
                window: "five_hour".to_string(),
                status: "rejected".to_string(),
                resets_at: Some(1_787_572_200),
                used_percent: None,
            },
        );

        assert!(!forward_managed_events(
            &writer,
            "claude-1",
            vec![event],
            &mut tracker,
            &mut HashMap::new(),
        ));
        let mut line = String::new();
        BufReader::new(reader_stream).read_line(&mut line).unwrap();
        assert!(matches!(
            decode(line.trim_end()).unwrap(),
            WrapperToDaemon::RateLimit {
                agent,
                window,
                status,
                resets_at: Some(1_787_572_200),
                used_percent: None,
                credits: None,
                source: bridget_transport::protocol::RateLimitSource::ClaudeStreamJson,
            } if agent == "claude-1" && window == "five_hour" && status == "rejected"
        ));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn modele_servi_claude_est_transmis_sans_decision() {
        let root = mcp_test_root("served-model");
        std::fs::create_dir_all(&root).unwrap();
        let (writer_stream, reader_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(Some(BufWriter::new(writer_stream))));
        let mut tracker = IdempotentDeliveryTracker::open_at(&root, "instance-served").unwrap();
        let event = ManagedEvent::source_line(
            bridget_transport::ManagedEventSource::ClaudeStreamJson,
            br#"{"type":"system","subtype":"init","model":"claude-opus-4-6"}"#.to_vec(),
            ManagedEventKind::ModelObserved {
                model: "claude-opus-4-6".to_string(),
            },
        );

        assert!(!forward_managed_events(
            &writer,
            "claude-1",
            vec![event],
            &mut tracker,
            &mut HashMap::new(),
        ));
        let mut line = String::new();
        BufReader::new(reader_stream).read_line(&mut line).unwrap();
        assert!(matches!(
            decode(line.trim_end()).unwrap(),
            WrapperToDaemon::ServedModel { agent, model }
                if agent == "claude-1" && model == "claude-opus-4-6"
        ));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn evenement_d_usage_claude_est_transmis_sans_zero_invente() {
        let root = mcp_test_root("usage");
        std::fs::create_dir_all(&root).unwrap();
        let (writer_stream, reader_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(Some(BufWriter::new(writer_stream))));
        let mut tracker = IdempotentDeliveryTracker::open_at(&root, "instance-usage").unwrap();
        let event = ManagedEvent::source_line(
            bridget_transport::ManagedEventSource::ClaudeStreamJson,
            br#"{"type":"assistant","message":{"usage":{"input_tokens":2,"output_tokens":175,"cache_creation_input_tokens":40804,"cache_read_input_tokens":13907}}}"#.to_vec(),
            ManagedEventKind::UsageObserved {
                input_tokens: 2,
                output_tokens: 175,
                cache_creation_input_tokens: 40_804,
                cache_read_input_tokens: 13_907,
            },
        );

        assert!(!forward_managed_events(
            &writer,
            "claude-1",
            vec![event],
            &mut tracker,
            &mut HashMap::new(),
        ));
        let mut line = String::new();
        BufReader::new(reader_stream).read_line(&mut line).unwrap();
        assert!(matches!(
            decode(line.trim_end()).unwrap(),
            WrapperToDaemon::Usage {
                agent,
                input_tokens: 2,
                output_tokens: 175,
                execution_id: None,
                execution_generation: None,
                cache_creation_input_tokens: 40_804,
                cache_read_input_tokens: 13_907,
                provider_kind: None,
                source: bridget_transport::protocol::UsageSource::ClaudeStreamJson,
            } if agent == "claude-1"
        ));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn sonde_claude_ignore_l_historique_et_remonte_modele_effort_opaques() {
        let root = std::env::temp_dir().join(format!(
            "bridget-claude-probe-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("ancien.jsonl"),
            concat!(
                r#"{"type":"assistant","effort":"ancien","message":{"model":"claude-ancien"}}"#,
                "\n"
            ),
        )
        .unwrap();
        let locator = ClaudeTranscriptLocator::new(root.clone());
        std::fs::write(
            root.join("courant.jsonl"),
            concat!(
                r#"{"type":"assistant","isSidechain":false,"effort":"opaque-fournisseur","message":{"model":"claude-runtime-reel"}}"#,
                "\n"
            ),
        )
        .unwrap();

        let mut probe = RuntimeProbe::claude(locator);
        let observed = probe.poll().expect("le transcript courant est observé");

        assert_eq!(observed.model, "claude-runtime-reel");
        assert_eq!(observed.effort.as_deref(), Some("opaque-fournisseur"));
        assert_eq!(
            probe.source(),
            bridget_transport::protocol::RuntimeSource::ClaudeTranscript
        );
        assert!(probe.poll().is_none(), "aucun trafic sans changement");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn sonde_republication_apres_reconnect_meme_observation() {
        let root = std::env::temp_dir().join(format!(
            "bridget-claude-probe-reconnect-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let locator = ClaudeTranscriptLocator::new(root.clone());
        std::fs::write(
            root.join("courant.jsonl"),
            concat!(
                r#"{"type":"assistant","isSidechain":false,"effort":"high","message":{"model":"claude-fable-5"}}"#,
                "\n"
            ),
        )
        .unwrap();

        let mut probe = RuntimeProbe::claude(locator);
        let first = probe.poll().expect("première observation");
        assert_eq!(first.model, "claude-fable-5");
        assert!(
            probe.poll().is_none(),
            "sans invalidation, pas de republication"
        );

        // Simule la reconnexion après un redémarrage daemon : la présence est
        // vide, l'observation locale est pourtant inchangée.
        probe.invalidate_after_reconnect();
        let again = probe.poll().expect("republication forcée après reconnect");
        assert_eq!(again, first);

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn sonde_ne_bascule_pas_vers_un_transcript_voisin_plus_recent() {
        // Deux sessions Claude du même projet : la nôtre (fable-5) puis une
        // voisine (opus-5) écrite plus tard. Sans pin, max(mtime) basculait
        // vers le voisin — c'est l'anomalie observée sur la ligne bridget.
        let root = std::env::temp_dir().join(format!(
            "bridget-claude-probe-voisin-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let locator = ClaudeTranscriptLocator::new(root.clone());
        std::fs::write(
            root.join("aaaa-session-fable.jsonl"),
            concat!(
                r#"{"type":"assistant","isSidechain":false,"effort":"high","message":{"model":"claude-fable-5"}}"#,
                "\n"
            ),
        )
        .unwrap();
        // Garantir une mtime strictement postérieure pour le voisin.
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(
            root.join("bbbb-session-opus.jsonl"),
            concat!(
                r#"{"type":"assistant","isSidechain":false,"effort":"high","message":{"model":"claude-opus-5"}}"#,
                "\n"
            ),
        )
        .unwrap();

        let mut probe = RuntimeProbe::claude(locator);
        let first = probe.poll().expect("session fable d'abord");
        assert_eq!(first.model, "claude-fable-5");

        // Force une re-résolution de chemin (comme le refresh périodique).
        probe.invalidate_after_reconnect();
        // Réécrire le voisin pour qu'il soit le plus récent.
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(
            root.join("bbbb-session-opus.jsonl"),
            concat!(
                r#"{"type":"assistant","isSidechain":false,"effort":"high","message":{"model":"claude-opus-5"}}"#,
                "\n",
                r#"{"type":"assistant","isSidechain":false,"effort":"high","message":{"model":"claude-opus-5"}}"#,
                "\n"
            ),
        )
        .unwrap();

        let again = probe.poll().expect("republication de LA même session");
        assert_eq!(
            again.model, "claude-fable-5",
            "la sonde ne doit pas adopter le transcript voisin opus-5"
        );

        std::fs::remove_dir_all(root).unwrap();
    }

    fn mcp_test_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "bridget-mcp-wrapper-{label}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ))
    }

    fn user_config_snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        fn visit(root: &Path, directory: &Path, snapshot: &mut BTreeMap<PathBuf, Vec<u8>>) {
            let Ok(entries) = std::fs::read_dir(directory) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    visit(root, &path, snapshot);
                } else if path.is_file() {
                    snapshot.insert(
                        path.strip_prefix(root).unwrap().to_path_buf(),
                        std::fs::read(&path).unwrap(),
                    );
                }
            }
        }

        let mut snapshot = BTreeMap::new();
        for relative in [".claude", ".codex", ".gemini"] {
            visit(root, &root.join(relative), &mut snapshot);
        }
        snapshot
    }

    fn write_user_config_sentinels(root: &Path) {
        for (relative, contents) in [
            (".claude/settings.json", b"claude-user-config".as_slice()),
            (".codex/config.toml", b"codex-user-config".as_slice()),
            (".gemini/settings.json", b"gemini-user-config".as_slice()),
        ] {
            let path = root.join(relative);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, contents).unwrap();
        }
    }

    fn relay_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("bridget-relay-{name}-{}", std::process::id()))
    }

    fn receipt_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "bridget-receipt-wrapper-{name}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ))
    }

    fn idempotent_message(id: &str) -> bridget_core::BridgetMessage {
        let mut message = bridget_core::BridgetMessage::new("guichet", "equipier", "tâche");
        message.id = id.to_string();
        message
    }

    #[test]
    fn branchement_mcp_n_ecrit_aucune_configuration_utilisateur() {
        let root = mcp_test_root("config-vide");
        write_user_config_sentinels(&root);
        let before = user_config_snapshot(&root);
        let server = interactive_mcp_server_entry().unwrap();

        let config =
            claude_mcp_config_in(&root.join(".cache/bridget"), &server, "fixture").unwrap();
        assert!(config.path().exists());
        let claude =
            serde_json::from_slice::<serde_json::Value>(&std::fs::read(config.path()).unwrap())
                .unwrap();
        assert_eq!(claude["mcpServers"]["bridget"]["type"], "stdio");
        assert!(claude["mcpServers"]["bridget"].get("name").is_none());
        assert_eq!(
            claude["mcpServers"]["bridget"]["env"]["HOME"],
            std::env::var("HOME").unwrap()
        );
        assert_eq!(server["name"], "bridget");
        drop(config);

        assert_eq!(user_config_snapshot(&root), before);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spawn_gere_injecte_mcp_identite_et_path() {
        let root = mcp_test_root("gere-equip");
        let socket = root.join("bridget.sock");
        let name_file = root.join("agent-names").join("instance-instance-1");
        let env =
            managed_adapter_environment("instance-1", Some("fable-reviewer"), Some(&name_file))
                .unwrap();
        let pairs: Vec<(String, String)> = string_environment(&env);
        assert_eq!(
            pairs
                .iter()
                .find(|(key, _)| key == "BRIDGET_AGENT_ID")
                .map(|(_, value)| value.as_str()),
            Some("fable-reviewer")
        );
        assert_eq!(
            pairs
                .iter()
                .find(|(key, _)| key == "BRIDGET_AGENT_ID_FILE")
                .map(|(_, value)| value.as_str()),
            Some(name_file.to_str().unwrap())
        );
        let path = pairs
            .iter()
            .find(|(key, _)| key == "PATH")
            .map(|(_, value)| value.as_str())
            .unwrap();
        let directory = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        // Intention : premier élément = binaire courant (mutant « déjà présent
        // plus loin » doit rougir).
        assert_eq!(
            path.split(':').map(str::trim).next(),
            Some(directory.as_str()),
            "{path}"
        );

        let mut claude_args = vec!["--model".to_string(), "claude-opus-5".to_string()];
        let config = apply_managed_mcp(
            "claude_stream_json",
            "claude",
            &mut claude_args,
            "instance-1",
            &socket,
        )
        .unwrap()
        .expect("config MCP Claude");
        assert!(
            claude_args
                .iter()
                .any(|argument| argument == "--mcp-config")
        );
        assert!(
            claude_args
                .iter()
                .any(|argument| argument == "--strict-mcp-config")
        );
        assert!(config.path().exists());
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(config.path())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );

        let mut codex_args = vec!["app-server".to_string()];
        assert!(
            apply_managed_mcp(
                "codex_app_server",
                "codex",
                &mut codex_args,
                "instance-1",
                &socket
            )
            .unwrap()
            .is_none()
        );
        assert_eq!(codex_args[0], "app-server");
        assert_eq!(codex_args[1], "-c");
        assert!(codex_args[2].contains("mcp_servers.bridget"));

        let mut none_args = Vec::new();
        assert!(
            apply_managed_mcp("claude_stream_json", "none", &mut none_args, "x", &socket)
                .unwrap()
                .is_none()
        );
        assert!(none_args.is_empty());

        let unnamed = string_environment(
            &managed_adapter_environment("instance-1", None, Some(&name_file)).unwrap(),
        );
        assert!(
            unnamed.iter().all(|(key, _)| key != "BRIDGET_AGENT_ID"),
            "{unnamed:?}"
        );
        assert_eq!(
            instance_name_state_path(&socket, "abc"),
            socket
                .parent()
                .unwrap()
                .join("agent-names")
                .join("instance-abc")
        );

        let orphan = root.join("mcp-crash.json");
        std::fs::write(&orphan, b"{}").unwrap();
        purge_orphan_mcp_configs(&root);
        assert!(!orphan.exists());

        drop(config);
        std::fs::remove_dir_all(root).unwrap();
    }

    /// ORACLE : managed-wrapper transmet la politique `permissions`.
    /// Séquence exacte de `launch_acp_with_status` : clone → MCP → policy.
    /// Attentes en dur — jamais un appel qui reconstruit le drapeau attendu.
    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_managed_wrapper_transmet_la_politique_d_autorisation() {
        let root = std::env::temp_dir().join(format!(
            "bridget-managed-policy-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let socket = root.join("bridget.sock");

        // Codex allow : drapeau présent, avant app-server, univers non vide.
        let mut codex_allow = vec![
            "-c".to_string(),
            "model=\"gpt-5.6-terra\"".to_string(),
            "app-server".to_string(),
        ];
        apply_managed_mcp(
            "codex_app_server",
            "codex",
            &mut codex_allow,
            "policy-allow",
            &socket,
        )
        .unwrap();
        apply_managed_permission_policy("codex_app_server", "allow", &mut codex_allow);
        assert!(!codex_allow.is_empty(), "PRESENCE: argv managed non vide");
        assert_eq!(
            codex_allow
                .iter()
                .filter(|a| *a == "--dangerously-bypass-approvals-and-sandbox")
                .count(),
            1,
            "PRESENCE allow Codex: un drapeau, reçu {codex_allow:?}"
        );
        let bypass_pos = codex_allow
            .iter()
            .position(|a| a == "--dangerously-bypass-approvals-and-sandbox")
            .expect("drapeau Codex");
        let app_pos = codex_allow
            .iter()
            .position(|a| a == "app-server")
            .expect("app-server");
        assert!(
            bypass_pos < app_pos,
            "EFFET: drapeau avant app-server, reçu {codex_allow:?}"
        );

        // Codex deny : le même chemin retire le drapeau (sinon deny ment).
        apply_managed_permission_policy("codex_app_server", "deny", &mut codex_allow);
        assert!(
            codex_allow
                .iter()
                .all(|a| a != "--dangerously-bypass-approvals-and-sandbox" && a != "--yolo"),
            "deny Codex retire le contournement, reçu {codex_allow:?}"
        );

        // Claude allow / deny — même chemin managed.
        let mut claude_allow = vec!["--model".to_string(), "claude-opus-5".to_string()];
        apply_managed_permission_policy("claude_stream_json", "allow", &mut claude_allow);
        assert!(
            claude_allow
                .iter()
                .any(|a| a == "--dangerously-skip-permissions"),
            "PRESENCE allow Claude: skip, reçu {claude_allow:?}"
        );
        assert!(
            claude_allow.iter().any(|a| a == "bypassPermissions"),
            "PRESENCE allow Claude: mode, reçu {claude_allow:?}"
        );
        apply_managed_permission_policy("claude_stream_json", "deny", &mut claude_allow);
        assert!(
            claude_allow
                .iter()
                .all(|a| a != "--dangerously-skip-permissions" && a != "bypassPermissions"),
            "deny Claude retire le bypass, reçu {claude_allow:?}"
        );

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec091_mcp_conserve_les_overrides_codex_des_deux_niveaux() {
        let mut args: Vec<String> = [
            "--sandbox",
            "workspace-write",
            "-c",
            "approval_policy=\"never\"",
            "--config=model_reasoning_effort=\"high\"",
            "-c=model=\"gpt-5.6-terra\"",
            "app-server",
            "-c",
            "sandbox_mode=\"workspace-write\"",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        apply_managed_mcp(
            "codex_app_server",
            "codex",
            &mut args,
            "unused",
            Path::new("/tmp/unused.sock"),
        )
        .unwrap();
        assert_eq!(
            &args[..9],
            [
                "--sandbox",
                "workspace-write",
                "app-server",
                "-c",
                "approval_policy=\"never\"",
                "--config=model_reasoning_effort=\"high\"",
                "-c=model=\"gpt-5.6-terra\"",
                "-c",
                "sandbox_mode=\"workspace-write\"",
            ]
        );
        assert_eq!(args[9], "-c");
        assert!(args[10].starts_with("mcp_servers.bridget="));
        let mut invalid = vec!["-c".to_owned(), "app-server".to_owned()];
        assert!(
            apply_managed_mcp(
                "codex_app_server",
                "codex",
                &mut invalid,
                "unused",
                Path::new("/tmp/unused.sock")
            )
            .is_err()
        );
    }

    #[test]
    fn spec091_politique_mcp_reste_octet_pour_octet_stable() {
        let server = serde_json::json!({
            "command": "/tmp/bridget-test", "env": {
                "HOME": "/tmp/home", "BRIDGET_HOME": "/tmp/state",
                "BRIDGET_SOCKET": "/tmp/state/bridget.sock"
            }
        });
        // Oracle indépendant : omettre cette politique reproduit le refus
        // réel avec approval_policy=never ; `auto` n'est PAS une autorisation.
        assert_eq!(
            codex_mcp_override(&server).unwrap(),
            concat!(
                "mcp_servers.bridget={command=\"/tmp/bridget-test\",args=[\"mcp\"],",
                "env={HOME=\"/tmp/home\",BRIDGET_HOME=\"/tmp/state\",BRIDGET_SOCKET=\"/tmp/state/bridget.sock\"},",
                "tool_timeout_sec=3700,default_tools_approval_mode=\"prompt\",tools={",
                "bridget_who={approval_mode=\"approve\"},",
                "bridget_specialists={approval_mode=\"approve\"},",
                "bridget_send={approval_mode=\"approve\"},",
                "bridget_ledger={approval_mode=\"approve\"},",
                "bridget_journal={approval_mode=\"approve\"},",
                "bridget_events={approval_mode=\"approve\"},",
                "bridget_thread={approval_mode=\"approve\"},",
                "bridget_handoff={approval_mode=\"approve\"},",
                "bridget_cancel={approval_mode=\"approve\"},",
                "bridget_read_artifact={approval_mode=\"approve\"},",
                "bridget_publish_artifact={approval_mode=\"approve\"},",
                "bridget_rename={approval_mode=\"approve\"},",
                "bridget_dnd={approval_mode=\"approve\"},",
                "bridget_domain={approval_mode=\"approve\"},",
                "bridget_runtime={approval_mode=\"approve\"},",
                "bridget_status={approval_mode=\"approve\"},",
                "bridget_control_status={approval_mode=\"approve\"},",
                "bridget_capabilities={approval_mode=\"approve\"},",
                "bridget_refresh_models={approval_mode=\"approve\"},",
                "bridget_read_commit_file={approval_mode=\"approve\"},",
                "bridget_delegate={approval_mode=\"approve\"},",
                "bridget_auto_delegate={approval_mode=\"approve\"},",
                "bridget_task_status={approval_mode=\"approve\"},",
                "bridget_task_cancel={approval_mode=\"approve\"},",
                "bridget_task_wait={approval_mode=\"approve\"},",
                "bridget_request_wait={approval_mode=\"approve\"},",
                "bridget_task_ack={approval_mode=\"approve\"},",
                "bridget_task_continue={approval_mode=\"approve\"},",
                "bridget_task_repair_workspace={approval_mode=\"approve\"},",
                "bridget_task_respond={approval_mode=\"approve\"}}}"
            )
        );
    }

    #[test]
    fn native151_facade_montage_bridget_autorise_attentes_et_delai() {
        let server = serde_json::json!({"command":"/tmp/bridget-test", "env":{"HOME":"/tmp/home", "BRIDGET_HOME":"/tmp/state", "BRIDGET_SOCKET":"/tmp/state/bridget.sock"}});
        let policy = codex_mcp_override(&server).unwrap();
        assert!(policy.contains("tool_timeout_sec=3700"));
        for name in [
            "bridget_task_wait",
            "bridget_request_wait",
            "bridget_task_ack",
        ] {
            assert!(policy.contains(&format!("{name}={{approval_mode=\"approve\"}}")));
            assert!(BRIDGET_SAFE_MCP_TOOLS.contains(&name));
        }
        let mut args = vec!["--config=mcp_servers.other.tool_timeout_sec=47".into()];
        apply_managed_mcp(
            "codex_app_server",
            "codex",
            &mut args,
            "unused",
            Path::new("/tmp/unused.sock"),
        )
        .unwrap();
        assert_eq!(args[0], "--config=mcp_servers.other.tool_timeout_sec=47");
    }

    #[test]
    fn spec094_codex_autorise_exactement_les_outils_de_communication_surs() {
        let server = serde_json::json!({
            "command": "/tmp/bridget-test", "env": {
                "HOME": "/tmp/home", "BRIDGET_HOME": "/tmp/state",
                "BRIDGET_SOCKET": "/tmp/state/bridget.sock"
            }
        });
        let policy = codex_mcp_override(&server).unwrap();
        for name in [
            "bridget_who",
            "bridget_send",
            "bridget_ledger",
            "bridget_journal",
            "bridget_events",
            "bridget_cancel",
            "bridget_read_artifact",
            "bridget_publish_artifact",
            "bridget_rename",
            "bridget_dnd",
            "bridget_domain",
            "bridget_runtime",
            "bridget_status",
            "bridget_control_status",
            "bridget_capabilities",
            "bridget_delegate",
            "bridget_auto_delegate",
            "bridget_task_status",
            "bridget_task_cancel",
            "bridget_task_wait",
            "bridget_request_wait",
            "bridget_task_ack",
            "bridget_task_continue",
            "bridget_task_respond",
        ] {
            assert!(
                policy.contains(&format!("{name}={{approval_mode=\"approve\"}}")),
                "autorisation Codex absente : {name} dans {policy}"
            );
        }
        assert!(policy.contains("default_tools_approval_mode=\"prompt\""));
        assert!(!policy.contains("guichet_delegate={approval_mode=\"approve\"}"));
    }

    #[test]
    fn native148_claude_allows_native_delegation_without_global_mcp_approval() {
        let mut args = Vec::new();
        append_claude_allowed_tools(&mut args);
        assert_eq!(args[0], "--allowedTools");
        let permitted: Vec<_> = args[1].split(',').collect();
        assert_eq!(permitted.len(), 30);
        for tool in [
            "bridget_capabilities",
            "bridget_delegate",
            "bridget_auto_delegate",
            "bridget_task_status",
            "bridget_task_cancel",
            "bridget_task_wait",
            "bridget_request_wait",
            "bridget_task_ack",
            "bridget_task_continue",
            "bridget_task_respond",
        ] {
            assert!(permitted.contains(&format!("mcp__bridget__{tool}").as_str()));
        }
        assert!(!permitted.iter().any(|tool| tool.contains('*')));
        assert!(!permitted.contains(&"mcp__bridget__guichet_delegate"));
    }

    #[test]
    #[ignore = "requiert le binaire Codex installé ; config/read réel, sans appel modèle"]
    fn spec091_codex_lit_la_politique_mcp_injectee() {
        let registry = crate::registry::AgentRegistry::from_json("{}", "/tmp/unused-registry.json")
            .unwrap()
            .for_spawn_posture(
                "codex",
                bridget_transport::protocol::SpawnPosture::Development,
            )
            .unwrap();
        let mut args = registry.get("codex").unwrap().args.clone();
        // VRAI assemblage registre -> projection wrapper -> parseur Codex.
        // Un simple `-c ... app-server` ne reproduit pas la collision entre
        // les overrides globaux et ceux du profil placés après app-server.
        apply_managed_mcp(
            "codex_app_server",
            "codex",
            &mut args,
            "unused",
            Path::new("/tmp/unused.sock"),
        )
        .unwrap();
        let output = std::process::Command::new("python3")
            .arg("-c")
            .arg(include_str!("../tests/fixtures/codex_mcp_policy_091.py"))
            .arg(serde_json::to_string(&args).unwrap())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn projections_mcp_portent_le_namespace_sans_modifier_home_fournisseur() {
        let server = mcp_server_entry().unwrap();
        let expected = mcp_server_environment().unwrap();
        let pairs = server["env"].as_array().unwrap();
        assert_eq!(pairs.len(), 3);
        for pair in pairs {
            assert_eq!(pair["value"], expected[pair["name"].as_str().unwrap()]);
        }
        assert_eq!(interactive_mcp_server_entry().unwrap()["env"], expected);
        let codex = codex_mcp_override(&interactive_mcp_server_entry().unwrap()).unwrap();
        for name in ["HOME", "BRIDGET_HOME", "BRIDGET_SOCKET"] {
            assert!(codex.contains(&format!(
                "{name}={}",
                serde_json::to_string(&expected[name]).unwrap()
            )));
        }
        assert_eq!(expected["HOME"], std::env::var("HOME").unwrap());
    }

    #[test]
    fn nom_persistant_ne_lit_ni_n_ecrase_une_cible_liee() {
        let root = std::env::temp_dir().join(format!("b89-name-{}", uuid::Uuid::new_v4()));
        crate::environment::ensure_private_directory(&root).unwrap();
        let original = root.join("original");
        save_persistent_name_at(&original, "nom-intact").unwrap();
        let linked = root.join("session");
        std::os::unix::fs::symlink(&original, &linked).unwrap();
        assert!(load_persistent_name_at(&linked).is_none());
        assert!(save_persistent_name_at(&linked, "nouveau-nom").is_err());
        assert_eq!(std::fs::read_to_string(&original).unwrap(), "nom-intact");
        assert_eq!(
            load_persistent_name_at(&original).as_deref(),
            Some("nom-intact")
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn fichier_mcp_ephemere_est_nettoye_avant_un_refus_de_spawn() {
        let root = mcp_test_root("refus-avant-spawn");
        write_user_config_sentinels(&root);
        let before = user_config_snapshot(&root);
        let server = interactive_mcp_server_entry().unwrap();
        let config =
            claude_mcp_config_in(&root.join(".cache/bridget"), &server, "refused").unwrap();
        let path = config.path().to_path_buf();

        assert!(validate_wrapper_args(&["interdit;".to_string()]).is_err());
        drop(config);

        assert!(!path.exists(), "le garde nettoie le fichier avant le spawn");
        assert_eq!(user_config_snapshot(&root), before);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn redelivery_after_seen_never_injects_a_second_prompt() {
        let root = receipt_root("seen");
        let instance_id = "instance_012_aaaaaaaaaaaa";
        let mut tracker = IdempotentDeliveryTracker::open_at(&root, instance_id).unwrap();
        let first_action = tracker.receive(
            "delivery_seen".to_string(),
            instance_id.to_string(),
            17,
            500,
            idempotent_message("prompt-unique"),
            100,
        );
        assert!(matches!(
            first_action,
            IdempotentDeliveryAction::Inject { .. }
        ));

        // Simule le redémarrage du daemon après remise mais avant son issue :
        // le wrapper vivant revoit la même remise et ne peut pas la réinjecter.
        let redelivery = tracker.receive(
            "delivery_seen".to_string(),
            instance_id.to_string(),
            17,
            500,
            idempotent_message("prompt-unique"),
            101,
        );
        assert!(matches!(redelivery, IdempotentDeliveryAction::Pending));
        drop(tracker);
        // Si le wrapper a réellement disparu, Seen seul reste ambigu.
        let mut tracker = IdempotentDeliveryTracker::open_at(&root, instance_id).unwrap();
        assert!(matches!(tracker.receive("delivery_seen".into(), instance_id.into(), 17,
            500, idempotent_message("prompt-unique"), 102),
            IdempotentDeliveryAction::Report(WrapperToDaemon::DeliveryIndeterminate { .. })));
        drop(tracker);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec244_sonde_de_recu_ne_modifie_pas_seen_et_ne_cree_pas_de_prompt() {
        let root=receipt_root("probe244");
        let instance="instance_probe244";
        let mut tracker=IdempotentDeliveryTracker::open_at(&root,instance).unwrap();
        assert!(tracker.receipt_report("absent".into(),instance,1,100).is_none());
        assert!(matches!(tracker.receive("d244".into(),instance.into(),7,500,idempotent_message("m244"),100),IdempotentDeliveryAction::Inject{..}));
        assert!(tracker.receipt_report("d244".into(),instance,7,101).is_none());
        assert!(matches!(tracker.prompt_dispatched("m244",102),Some(WrapperToDaemon::DeliverAcked{..})));
        drop(tracker);
        let tracker=IdempotentDeliveryTracker::open_at(&root,instance).unwrap();
        assert!(tracker.receipt_report("d244".into(),"foreign",7,103).is_none());
        assert!(matches!(tracker.receipt_report("d244".into(),instance,7,103),Some(WrapperToDaemon::DeliverAcked{delivery_generation:7,..})));
        assert!(tracker.receipt_report("d244".into(),instance,7,501).is_none());
        drop(tracker);std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn replaced_instance_is_indeterminate_without_injection() {
        let root = receipt_root("replaced");
        let mut tracker =
            IdempotentDeliveryTracker::open_at(&root, "instance_012_newwrapper").unwrap();
        let action = tracker.receive(
            "delivery_replaced".to_string(),
            "instance_012_oldwrapper".to_string(),
            29,
            500,
            idempotent_message("prompt-replaced"),
            100,
        );
        assert!(matches!(
            action,
            IdempotentDeliveryAction::Report(WrapperToDaemon::DeliveryIndeterminate {
                delivery_id,
                delivery_generation: 29,
            }) if delivery_id == "delivery_replaced"
        ));
        drop(tracker);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec244_doublon_en_cours_attend_le_dispatch_unique() {
        let root = receipt_root("244-pending");
        let instance = "instance_244_pending";
        let mut tracker = IdempotentDeliveryTracker::open_at(&root, instance).unwrap();
        let receive = |tracker: &mut IdempotentDeliveryTracker| tracker.receive(
            "delivery_pending".into(), instance.into(), 1, 500,
            idempotent_message("message_pending"), 100,
        );
        assert!(matches!(receive(&mut tracker), IdempotentDeliveryAction::Inject { .. }));
        assert!(matches!(receive(&mut tracker), IdempotentDeliveryAction::Pending));
        assert!(matches!(tracker.prompt_dispatched("message_pending", 101),
            Some(WrapperToDaemon::DeliverAcked { delivery_generation: 1, .. })));
        assert!(tracker.prompt_dispatched("message_pending", 102).is_none());
        assert!(matches!(receive(&mut tracker), IdempotentDeliveryAction::Report(WrapperToDaemon::DeliverAcked { .. })));
        drop(tracker);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn prompt_dispatched_persists_the_ack_before_reporting_it() {
        let root = receipt_root("ack");
        let instance_id = "instance_012_aaaaaaaaaaaa";
        let mut tracker = IdempotentDeliveryTracker::open_at(&root, instance_id).unwrap();
        assert!(matches!(
            tracker.receive(
                "delivery_acked".to_string(),
                instance_id.to_string(),
                31,
                500,
                idempotent_message("prompt-acked"),
                100,
            ),
            IdempotentDeliveryAction::Inject { .. }
        ));
        assert!(matches!(
            tracker.prompt_dispatched("prompt-acked", 101),
            Some(WrapperToDaemon::DeliverAcked {
                delivery_id,
                delivery_generation: 31,
            }) if delivery_id == "delivery_acked"
        ));
        assert!(matches!(
            tracker.receive(
                "delivery_acked".to_string(),
                instance_id.to_string(),
                31,
                500,
                idempotent_message("prompt-acked"),
                102,
            ),
            IdempotentDeliveryAction::Report(WrapperToDaemon::DeliverAcked { .. })
        ));
        drop(tracker);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn livraison_idempotente_interactive_injecte_une_fois_et_rejoue_l_accuse() {
        const CHILD: &str = "BRIDGET_WRAPPER_IDEMPOTENT_RECONNECT_CHILD";
        const ROOT: &str = "BRIDGET_WRAPPER_IDEMPOTENT_RECONNECT_ROOT";

        if std::env::var_os(CHILD).is_some() {
            let root = PathBuf::from(
                std::env::var_os(ROOT).expect("racine de reçus transmise au sous-processus"),
            );
            exercise_idempotent_interactive_reconnect(&root);
            return;
        }

        let root = receipt_root("interactive-idempotent");
        let executable = std::env::current_exe().expect("binaire de test courant");
        assert_ne!(
            executable.file_name().and_then(|name| name.to_str()),
            Some("firefox"),
            "le témoin de reconnexion doit ouvrir le binaire de test, jamais Firefox"
        );
        let child = Command::new(executable)
            .args([
                "--exact",
                "wrapper::reconnect_tests::livraison_idempotente_interactive_injecte_une_fois_et_rejoue_l_accuse",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(CHILD, "1")
            .env(ROOT, &root)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("lancement du témoin de reconnexion isolé");
        let output = wait_child_bounded(child, "témoin de reconnexion idempotente");
        let _ = std::fs::remove_dir_all(&root);
        assert!(
            output.status.success(),
            "le témoin isolé doit rejouer l'accusé sans seconde injection:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    fn exercise_idempotent_interactive_reconnect(root: &Path) {
        let instance_id = "instance_012_interactive";
        let mut injections = 0;
        let message = idempotent_message("interactive-prompt");
        let mut tracker = IdempotentDeliveryTracker::open_at(root, instance_id).unwrap();

        let first = deliver_idempotent_to_interactive(
            &mut tracker,
            "interactive-delivery".to_string(),
            instance_id.to_string(),
            61,
            500,
            message.clone(),
            100,
            |_| {
                injections += 1;
                Ok(())
            },
        );
        assert_eq!(injections, 1);
        assert!(matches!(
            first.as_slice(),
            [WrapperToDaemon::DeliverAcked {
                delivery_id,
                delivery_generation: 61,
            }] if delivery_id == "interactive-delivery"
        ));

        // La reconnexion réouvre le même store de reçus : le daemon peut
        // redélivrer, mais le pane ne reçoit jamais un second prompt.
        drop(tracker);
        let mut reconnected = IdempotentDeliveryTracker::open_at(root, instance_id).unwrap();
        let replay = deliver_idempotent_to_interactive(
            &mut reconnected,
            "interactive-delivery".to_string(),
            instance_id.to_string(),
            61,
            500,
            message,
            101,
            |_| {
                injections += 1;
                Ok(())
            },
        );
        assert_eq!(injections, 1);
        assert!(matches!(
            replay.as_slice(),
            [WrapperToDaemon::DeliverAcked {
                delivery_id,
                delivery_generation: 61,
            }] if delivery_id == "interactive-delivery"
        ));
        drop(reconnected);
    }

    fn wait_child_bounded(mut child: std::process::Child, label: &str) -> std::process::Output {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if child.try_wait().expect("état du sous-processus").is_some() {
                return child
                    .wait_with_output()
                    .expect("sortie du sous-processus de reconnexion");
            }
            if Instant::now() >= deadline {
                let pid = child.id();
                child.kill().expect("arrêt du sous-processus bloqué");
                let _ = child.wait();
                panic!("timeout : {label} (pid {pid})");
            }
            thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn same_message_id_from_two_scopes_keeps_each_delivery_acknowledgement() {
        let root = receipt_root("same-message-id");
        let instance_id = "instance_012_aaaaaaaaaaaa";
        let mut tracker = IdempotentDeliveryTracker::open_at(&root, instance_id).unwrap();
        for (delivery_id, generation) in [("delivery_scope_a", 41), ("delivery_scope_b", 43)] {
            assert!(matches!(
                tracker.receive(
                    delivery_id.to_string(),
                    instance_id.to_string(),
                    generation,
                    500,
                    idempotent_message("same-client-message-id"),
                    100,
                ),
                IdempotentDeliveryAction::Inject { .. }
            ));
        }
        assert!(matches!(
            tracker.prompt_dispatched("same-client-message-id", 101),
            Some(WrapperToDaemon::DeliverAcked {
                delivery_id,
                delivery_generation: 41,
            }) if delivery_id == "delivery_scope_a"
        ));
        assert!(matches!(
            tracker.prompt_dispatched("same-client-message-id", 102),
            Some(WrapperToDaemon::DeliverAcked {
                delivery_id,
                delivery_generation: 43,
            }) if delivery_id == "delivery_scope_b"
        ));
        drop(tracker);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn historic_and_idempotent_same_message_id_keep_dispatch_order() {
        let root = receipt_root("historic-order");
        let instance_id = "instance_012_aaaaaaaaaaaa";
        let message_id = "same-historic-and-idempotent";
        let mut tracker = IdempotentDeliveryTracker::open_at(&root, instance_id).unwrap();
        tracker.record_historic(message_id);
        assert!(matches!(
            tracker.receive(
                "delivery_after_historic".to_string(),
                instance_id.to_string(),
                47,
                500,
                idempotent_message(message_id),
                100,
            ),
            IdempotentDeliveryAction::Inject { .. }
        ));
        assert!(tracker.prompt_dispatched(message_id, 101).is_none());
        assert!(matches!(
            tracker.prompt_dispatched(message_id, 102),
            Some(WrapperToDaemon::DeliverAcked {
                delivery_id,
                delivery_generation: 47,
            }) if delivery_id == "delivery_after_historic"
        ));
        drop(tracker);

        let root_reverse = receipt_root("idempotent-order");
        let mut reverse = IdempotentDeliveryTracker::open_at(&root_reverse, instance_id).unwrap();
        assert!(matches!(
            reverse.receive(
                "delivery_before_historic".to_string(),
                instance_id.to_string(),
                53,
                500,
                idempotent_message(message_id),
                100,
            ),
            IdempotentDeliveryAction::Inject { .. }
        ));
        reverse.record_historic(message_id);
        assert!(matches!(
            reverse.prompt_dispatched(message_id, 101),
            Some(WrapperToDaemon::DeliverAcked {
                delivery_id,
                delivery_generation: 53,
            }) if delivery_id == "delivery_before_historic"
        ));
        assert!(reverse.prompt_dispatched(message_id, 102).is_none());
        drop(reverse);
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_dir_all(root_reverse).unwrap();
    }

    fn relay_emitter() -> (RelayEvents, RelayEmitter) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = events.clone();
        let emitter: Arc<dyn Fn(WrapperToDaemon) + Send + Sync> = Arc::new(move |message| {
            captured
                .lock()
                .recover_poison()
                .push(message);
        });
        (events, emitter)
    }

    /// Journal d'observation réservé aux tests qui vérifient l'ordonnancement
    /// du relais. Chaque émission réveille le test : celui-ci attend donc un
    /// fait du worker, et non une fenêtre arbitraire dépendante de la charge.
    struct ObservableRelayEvents {
        messages: Mutex<Vec<WrapperToDaemon>>,
        emitted: Condvar,
    }

    fn observable_relay_emitter() -> (Arc<ObservableRelayEvents>, RelayEmitter) {
        let events = Arc::new(ObservableRelayEvents {
            messages: Mutex::new(Vec::new()),
            emitted: Condvar::new(),
        });
        let captured = events.clone();
        let emitter: Arc<dyn Fn(WrapperToDaemon) + Send + Sync> = Arc::new(move |message| {
            captured
                .messages
                .lock()
                .recover_poison()
                .push(message);
            captured.emitted.notify_all();
        });
        (events, emitter)
    }

    fn wait_for_relay_state(
        events: &ObservableRelayEvents,
        condition: impl Fn(&[WrapperToDaemon]) -> bool,
    ) {
        let mut messages = events
            .messages
            .lock()
            .recover_poison();
        while !condition(&messages) {
            messages = events
                .emitted
                .wait(messages)
                .recover_poison();
        }
    }

    fn wait_for(condition: impl Fn() -> bool) {
        // Attente de vivacité seulement (aucun oracle de durée) : 10 s laissent
        // le worker progresser quand toute la suite charge la machine.
        for _ in 0..1000 {
            if condition() {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("condition du worker de relais non atteinte dans la borne de test");
    }

    fn acp_definition(forbidden_env: &[&str]) -> crate::registry::AgentDefinition {
        crate::registry::AgentDefinition {
            command: "adapter".to_string(),
            args: Vec::new(),
            protocol: "acp".to_string(),
            forbidden_env: forbidden_env.iter().map(ToString::to_string).collect(),
            pass_env: Vec::new(),
            claude_config_dir: None,
            codex_home: None,
            permissions: "allow".to_string(),
            queue_capacity: 32,
            held_queue: false,
            notify_timeout_secs: 600,
            mcp: crate::registry::McpDefinition::default(),
            capabilities: bridget_transport::AdapterCapabilities {
                account_profiles: false,
                execution_paths: vec!["acp".to_string()],
                models: std::collections::BTreeMap::new(),
                observed: None,
            },
        }
    }

    #[test]
    fn transport_claude_managed_publie_le_type_fourni_au_wrapper() {
        let root = std::env::temp_dir().join(format!(
            "bridget-wrapper-provider-kind-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let definition = crate::registry::AgentDefinition {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                "while IFS= read -r _; do :; done".to_string(),
            ],
            protocol: "claude_stream_json".to_string(),
            forbidden_env: Vec::new(),
            pass_env: Vec::new(),
            claude_config_dir: None,
            codex_home: None,
            permissions: "allow".to_string(),
            queue_capacity: 1,
            held_queue: false,
            notify_timeout_secs: 1,
            mcp: crate::registry::McpDefinition::default(),
            capabilities: bridget_transport::AdapterCapabilities::default(),
        };
        let transport = spawn_managed_session_transport(
            "glm",
            &definition,
            &definition.args,
            &[],
            Vec::new(),
            false,
            &root,
            Some("glm-test".to_string()),
            "test-instance",
            &root.join("bridget.sock"),
            None,
        )
        .unwrap();
        assert!(transport.drain_events().into_iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::ProviderContextObserved { identity }
                    if identity.provider_kind == "glm"
            )
        }));
        transport.stop();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn journal_failure_requires_an_immediate_transport_shutdown() {
        assert!(journal_failure_requires_shutdown(&[ManagedEvent {
            source: bridget_transport::ManagedEventSource::Acp,
            origin: bridget_transport::ManagedEventOrigin::Internal,
            raw: "journal ACP saturé".as_bytes().to_vec(),
            kind: ManagedEventKind::JournalFailed {
                detail: "journal ACP saturé".to_string(),
            },
        }]));
        assert!(!journal_failure_requires_shutdown(&[ManagedEvent {
            source: bridget_transport::ManagedEventSource::Acp,
            origin: bridget_transport::ManagedEventOrigin::Internal,
            raw: b"diagnostic non terminal".to_vec(),
            kind: ManagedEventKind::Error {
                detail: "diagnostic non terminal".to_string(),
            },
        }]));
    }

    #[test]
    fn api_key_forbidden_refuse_le_lancement_en_nommant_la_variable() {
        let variable = crate::registry::forbidden_environment_variable(
            &acp_definition(&["OPENAI_API_KEY", "CODEX_API_KEY"]),
            false,
            |variable| matches!(variable, "OPENAI_API_KEY" | "CODEX_API_KEY"),
        )
        .expect("clé API refusée");
        let error = billing_guard_error(&variable);
        assert!(error.contains("OPENAI_API_KEY"));
        assert!(!error.contains("CODEX_API_KEY"));
        assert!(error.contains("BRIDGET_ALLOW_API_KEY=1"));
    }

    #[test]
    fn api_key_forbidden_utilise_la_seconde_variable_si_elle_est_seule() {
        let error = crate::registry::forbidden_environment_variable(
            &acp_definition(&["OPENAI_API_KEY", "CODEX_API_KEY"]),
            false,
            |variable| variable == "CODEX_API_KEY",
        )
        .expect("seconde clé API refusée");
        assert!(error.contains("CODEX_API_KEY"));
    }

    #[test]
    fn api_key_forbidden_accepte_le_contournement_explicite() {
        assert!(
            crate::registry::forbidden_environment_variable(
                &acp_definition(&["ANTHROPIC_API_KEY"]),
                true,
                |variable| variable == "ANTHROPIC_API_KEY",
            )
            .is_none()
        );
    }

    #[test]
    fn seul_le_contournement_egal_a_un_est_accepte() {
        assert!(crate::registry::allow_api_key_value(Some("1")));
        for value in [None, Some("0"), Some("true"), Some("01")] {
            assert!(
                !crate::registry::allow_api_key_value(value),
                "valeur non autorisée: {value:?}"
            );
        }
    }

    #[test]
    fn equipier_flag_is_removed_before_the_agent_is_started() {
        let (equipier, remaining) = split_equipier_flag(&[
            "--equipier".to_string(),
            "resume".to_string(),
            "session".to_string(),
        ]);
        assert!(equipier);
        assert_eq!(remaining, vec!["resume", "session"]);
    }

    #[test]
    fn equipier_refuse_le_pont_zed_avant_tout_processus() {
        use std::os::unix::fs::PermissionsExt;
        // C2 revue G10 : launch_acp_with_status (--equipier) partage la garde.
        const CHILD: &str = "BRIDGET_089_ZED_REFUSAL_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let root = PathBuf::from(format!(
                "/tmp/b89zed-{}",
                &uuid::Uuid::new_v4().simple().to_string()[..12]
            ));
            std::fs::create_dir(&root).unwrap();
            std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
            let root = root.canonicalize().unwrap();
            for name in ["home", "state", "tmp"] {
                std::fs::create_dir(root.join(name)).unwrap();
                std::fs::set_permissions(root.join(name), std::fs::Permissions::from_mode(0o700))
                    .unwrap();
            }
            // Même test dans son processus privé : aucune mutation HOME ou
            // BRIDGET_HOME globale pendant les autres tests parallèles.
            let output = Command::new("/usr/bin/perl")
                .args(["-e", "alarm 20; exec @ARGV"])
                .arg(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "wrapper::reconnect_tests::equipier_refuse_le_pont_zed_avant_tout_processus",
                    "--nocapture",
                ])
                .env_clear()
                .env(CHILD, "1")
                .env("HOME", root.join("home"))
                .env("BRIDGET_HOME", root.join("state"))
                .env("BRIDGET_SOCKET", root.join("state/s"))
                .env("TMPDIR", root.join("tmp"))
                .env("PATH", "/usr/bin:/bin")
                .output()
                .unwrap();
            std::fs::remove_dir_all(&root).unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }
        let home = PathBuf::from(std::env::var_os("HOME").unwrap());
        let root = PathBuf::from(std::env::var_os("BRIDGET_HOME").unwrap());
        let registry = crate::registry::AgentRegistry::from_json(
            &serde_json::json!({"agents":{"legacy":{"command":home.join("codex-acp"),"protocol":"acp"}}}).to_string(),
            root.join("agents.json").display().to_string(),
        )
        .unwrap();
        let error = launch_acp_with(
            "legacy",
            &[],
            Some("89000000-0000-4000-8000-000000000106"),
            &registry,
            &root.join("s"),
            &home,
        )
        .expect_err("le pont Zed doit être refusé avant connexion daemon");
        let message = error.to_string();
        assert!(
            message.contains("pont Zed") && message.contains("@zed-industries/codex-acp"),
            "refus inattendu: {message}"
        );
    }

    #[test]
    fn le_domaine_derive_nomme_le_depot_courant() {
        // Le paquet livré n'a pas de .git. Le dépôt de l'oracle est privé,
        // indépendant du checkout de compilation et du cwd des autres tests.
        let root = std::env::temp_dir().join(format!("b89-domain-{}", uuid::Uuid::new_v4()));
        let repository = root.join("depot-atteste");
        let nested = repository.join("sous-repertoire");
        std::fs::create_dir_all(&nested).unwrap();
        assert!(
            Command::new("git")
                .arg("init")
                .arg(&repository)
                .output()
                .unwrap()
                .status
                .success()
        );
        assert_eq!(derive_domain_at(&nested).as_deref(), Some("depot-atteste"));
        let archive = root.join("archive-sans-git");
        std::fs::create_dir(&archive).unwrap();
        // Mutant : toujours retenir le cwd casse le premier assert ; exiger
        // une racine Git casse celui-ci. Aucun nom reconstruit par l'oracle.
        assert_eq!(
            derive_domain_at(&archive).as_deref(),
            Some("archive-sans-git")
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec094_reconnexion_enregistre_le_derive_puis_reapplique_l_override_sans_perdre_le_reader() {
        let root = PathBuf::from("/tmp").join(format!(
            "bgdomain-reconnect-{}",
            uuid::Uuid::new_v4().simple()
        ));
        crate::environment::ensure_private_directory(&root).unwrap();
        let socket = root.join("daemon.sock");
        let identity = "89000000-0000-4000-8000-000000000394";
        let instance_id = "instance-domain-094";
        bridget_transport::fsutil::write_private_file_atomic(
            &root.join("agent-domains").join(identity),
            b"revue-croisee",
        )
        .unwrap();
        let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
        let expected_derived = derive_domain();
        let expected_derived_for_server = expected_derived.clone();
        let credential =
            bridget_transport::protocol::IdentityCredential::new(uuid::Uuid::new_v4().to_string());
        let server = std::thread::spawn(move || {
            let (mut primary, _) = listener.accept().unwrap();
            let mut primary_reader = BufReader::new(primary.try_clone().unwrap());
            let mut line = String::new();
            primary_reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode(line.trim()).unwrap(),
                WrapperToDaemon::Register { agent_id, domain, .. }
                    if agent_id == identity && domain == expected_derived_for_server
            ));
            // SPEC172 : une réponse distante peut dépasser la lecture de session à 1 s.
            std::thread::sleep(Duration::from_millis(1500));
            writeln!(
                primary,
                "{}",
                encode(&DaemonToWrapper::Registered {
                    credential: Some(credential.clone()),
                    agent_id: identity.to_string(),
                })
                .unwrap()
            )
            .unwrap();
            writeln!(primary, "{}", encode(&DaemonToWrapper::Disconnect).unwrap()).unwrap();
            primary.flush().unwrap();

            let (mut auxiliary, _) = listener.accept().unwrap();
            let mut auxiliary_reader = BufReader::new(auxiliary.try_clone().unwrap());
            line.clear();
            auxiliary_reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode(line.trim()).unwrap(),
                WrapperToDaemon::RegisterAuxiliary { agent_id, credential: presented, .. }
                    if agent_id == identity && presented == credential
            ));
            writeln!(
                auxiliary,
                "{}",
                encode(&DaemonToWrapper::Registered {
                    credential: None,
                    agent_id: identity.to_string(),
                })
                .unwrap()
            )
            .unwrap();
            auxiliary.flush().unwrap();
            line.clear();
            auxiliary_reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode(line.trim()).unwrap(),
                WrapperToDaemon::Domain { agent, domain: Some(domain) }
                    if agent == identity && domain == "revue-croisee"
            ));
            writeln!(
                auxiliary,
                "{}",
                encode(&DaemonToWrapper::Ack {
                    project_warnings: Vec::new(),
                    id: "domain".to_string(),
                })
                .unwrap()
            )
            .unwrap();
            auxiliary.flush().unwrap();
        });

        let (mut reader, _writer, registered_name) = connect_and_register_with_domain_at(
            &socket,
            "codex",
            Some(identity),
            "hote-094",
            INTERACTIVE_AGENT_PROTOCOL,
            None,
            PresenceMode::Cli,
            None,
            "Linux",
            instance_id,
            false,
            false,
        )
        .unwrap();
        assert_eq!(registered_name, identity);
        assert_eq!(reader.get_ref().read_timeout().unwrap(), Some(Duration::from_secs(1)));
        let mut deferred = String::new();
        reader.read_line(&mut deferred).unwrap();
        assert!(matches!(
            decode(deferred.trim()).unwrap(),
            DaemonToWrapper::Disconnect
        ));
        server.join().unwrap();
        assert!(
            root.join("agent-domains")
                .join(format!(".{identity}.lock"))
                .exists()
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec094_reconnexion_attend_la_persistance_concurrente_avant_de_lire_l_override() {
        let _observer_guard = crate::communication::client::lock_domain_test_observers();
        let root = PathBuf::from("/tmp").join(format!(
            "bgdomain-reconnect-race-{}",
            uuid::Uuid::new_v4().simple()
        ));
        crate::environment::ensure_private_directory(&root).unwrap();
        let socket = root.join("daemon.sock");
        let identity = "89000000-0000-4000-8000-000000000494";
        let instance_id = "instance-domain-race-094";
        let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
        crate::mcp_identity::mock_private_identity(&socket, identity, instance_id);
        let credential =
            bridget_transport::protocol::IdentityCredential::new(uuid::Uuid::new_v4().to_string());
        let expected_derived = derive_domain();
        let (seen_tx, seen_rx) = mpsc::channel();
        let server = std::thread::spawn(move || {
            let (mut setter, _) = listener.accept().unwrap();
            let mut setter_reader = BufReader::new(setter.try_clone().unwrap());
            let mut line = String::new();
            setter_reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode(line.trim()).unwrap(),
                WrapperToDaemon::RegisterAuxiliary { agent_id, .. } if agent_id == identity
            ));
            writeln!(
                setter,
                "{}",
                encode(&DaemonToWrapper::Registered {
                    credential: None,
                    agent_id: identity.to_string(),
                })
                .unwrap()
            )
            .unwrap();
            setter.flush().unwrap();
            line.clear();
            setter_reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode(line.trim()).unwrap(),
                WrapperToDaemon::Domain { agent, domain: Some(domain) }
                    if agent == identity && domain == "nouveau"
            ));
            seen_tx.send("setter").unwrap();
            writeln!(
                setter,
                "{}",
                encode(&DaemonToWrapper::Ack {
                    project_warnings: Vec::new(),
                    id: "domain".to_string(),
                })
                .unwrap()
            )
            .unwrap();
            setter.flush().unwrap();

            let (mut primary, _) = listener.accept().unwrap();
            let mut primary_reader = BufReader::new(primary.try_clone().unwrap());
            line.clear();
            primary_reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode(line.trim()).unwrap(),
                WrapperToDaemon::Register { agent_id, domain, .. }
                    if agent_id == identity && domain == expected_derived
            ));
            seen_tx.send("primary").unwrap();
            writeln!(
                primary,
                "{}",
                encode(&DaemonToWrapper::Registered {
                    credential: Some(credential.clone()),
                    agent_id: identity.to_string(),
                })
                .unwrap()
            )
            .unwrap();
            writeln!(primary, "{}", encode(&DaemonToWrapper::Disconnect).unwrap()).unwrap();
            primary.flush().unwrap();

            let (mut auxiliary, _) = listener.accept().unwrap();
            let mut auxiliary_reader = BufReader::new(auxiliary.try_clone().unwrap());
            line.clear();
            auxiliary_reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode(line.trim()).unwrap(),
                WrapperToDaemon::RegisterAuxiliary { agent_id, credential: presented, .. }
                    if agent_id == identity && presented == credential
            ));
            writeln!(
                auxiliary,
                "{}",
                encode(&DaemonToWrapper::Registered {
                    credential: None,
                    agent_id: identity.to_string(),
                })
                .unwrap()
            )
            .unwrap();
            auxiliary.flush().unwrap();
            line.clear();
            auxiliary_reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode(line.trim()).unwrap(),
                WrapperToDaemon::Domain { agent, domain: Some(domain) }
                    if agent == identity && domain == "nouveau"
            ));
            seen_tx.send("override").unwrap();
            writeln!(
                auxiliary,
                "{}",
                encode(&DaemonToWrapper::Ack {
                    project_warnings: Vec::new(),
                    id: "domain".to_string(),
                })
                .unwrap()
            )
            .unwrap();
            auxiliary.flush().unwrap();
        });

        let (blocked_tx, blocked_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let release_rx = Arc::new(Mutex::new(release_rx));
        let release_rx_for_hook = Arc::clone(&release_rx);
        crate::communication::client::replace_domain_after_ack_observer(Some(Arc::new(
            move |observed_identity| {
                if observed_identity == identity {
                    blocked_tx.send(()).unwrap();
                    release_rx_for_hook.lock().unwrap().recv().unwrap();
                }
            },
        )));
        let (contended_tx, contended_rx) = mpsc::channel();
        let contention_seen = Arc::new(AtomicBool::new(false));
        let contention_seen_for_hook = Arc::clone(&contention_seen);
        crate::communication::client::replace_domain_lock_contention_observer(Some(Arc::new(
            move |observed_identity| {
                if observed_identity == identity
                    && !contention_seen_for_hook.swap(true, Ordering::SeqCst)
                {
                    contended_tx.send(()).unwrap();
                }
            },
        )));

        let setter_socket = socket.clone();
        let setter = std::thread::spawn(move || {
            crate::communication::client::set_domain(
                identity,
                instance_id,
                &setter_socket,
                Some("nouveau".to_string()),
            )
        });
        assert_eq!(
            seen_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
            "setter"
        );
        blocked_rx.recv_timeout(Duration::from_secs(2)).unwrap();

        let reconnect_socket = socket.clone();
        let reconnect = std::thread::spawn(move || {
            let (mut reader, _writer, registered_name) = connect_and_register_with_domain_at(
                &reconnect_socket,
                "codex",
                Some(identity),
                "hote-094",
                INTERACTIVE_AGENT_PROTOCOL,
                None,
                PresenceMode::Cli,
                None,
                "Linux",
                instance_id,
                false,
                false,
            )?;
            let mut deferred = String::new();
            reader
                .read_line(&mut deferred)
                .map_err(|error| error.to_string())?;
            if !matches!(
                decode(deferred.trim()).map_err(|error| error.to_string())?,
                DaemonToWrapper::Disconnect
            ) {
                return Err("trame différée principale perdue".to_string());
            }
            Ok::<_, String>(registered_name)
        });
        contended_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("la reconnexion doit rencontrer le verrou du setter");
        assert!(
            seen_rx.try_recv().is_err(),
            "la reconnexion a lu le domaine avant sa persistance"
        );
        release_tx.send(()).unwrap();
        assert!(setter.join().unwrap().is_ok());
        assert_eq!(reconnect.join().unwrap().unwrap(), identity);
        assert_eq!(
            seen_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
            "primary"
        );
        assert_eq!(
            seen_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
            "override"
        );
        server.join().unwrap();
        crate::communication::client::replace_domain_after_ack_observer(None);
        crate::communication::client::replace_domain_lock_contention_observer(None);
        assert_eq!(
            std::fs::read_to_string(root.join("agent-domains").join(identity)).unwrap(),
            "nouveau"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn le_nom_choisi_par_l_utilisateur_survit_a_la_reconnexion() {
        // `bridget rename` n'écrit que dans ce fichier ; le wrapper doit s'y
        // référer, sinon un agent renommé revient sous son nom d'origine à
        // chaque coupure — y compris une rupture de tunnel en fédération SSH.
        let path =
            std::env::temp_dir().join(format!("bridget-nom-{}-{}", std::process::id(), "renomme"));
        std::fs::write(&path, "agent-1\n").unwrap();
        assert_eq!(resolve_current_name(&path, "codex-8"), "agent-1");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn nom_illisible_ou_vide_retombe_sur_le_nom_d_enregistrement() {
        let absent = std::env::temp_dir().join("bridget-nom-inexistant-xyz");
        let _ = std::fs::remove_file(&absent);
        assert_eq!(resolve_current_name(&absent, "codex-8"), "codex-8");

        let vide = std::env::temp_dir().join(format!("bridget-nom-vide-{}", std::process::id()));
        std::fs::write(&vide, "   \n").unwrap();
        assert_eq!(resolve_current_name(&vide, "codex-8"), "codex-8");
        let _ = std::fs::remove_file(&vide);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn spec221_drain_wait_reports_before_eventual_stop_and_keeps_supervision() {
        const HELPER: &str = "BRIDGET_SPEC221_DRAIN_WAIT_HELPER";
        if std::env::var_os(HELPER).is_none() {
            for mode in ["after", "before"] {
                let status = Command::new(std::env::current_exe().unwrap())
                    .args(["--exact", "wrapper::reconnect_tests::spec221_drain_wait_reports_before_eventual_stop_and_keeps_supervision", "--nocapture", "--test-threads=1"])
                    .env(HELPER, mode).status().unwrap();
                assert!(status.success(), "phase de fermeture {mode}");
            }
            return;
        }
        use std::os::unix::fs::PermissionsExt;
        crate::managed_process::remote_drain::arm().unwrap();
        let root = relay_root("spec221-owned-survivor");
        let markers = root.join("markers");
        std::fs::create_dir_all(&markers).unwrap();
        std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::set_permissions(&markers, std::fs::Permissions::from_mode(0o700)).unwrap();
        let before_barrier = std::env::var(HELPER).unwrap() == "before";
        let journal_writer = before_barrier.then(|| JournalWriter::start(
            root.join("sessions"), "agent-221", "provider-session-221",
            Arc::new(Mutex::new(bridget_transport::AcpEventQueue::default())),
        ).unwrap());
        let mut child = Command::new("/usr/bin/python3")
            .args(["-c", "import signal,sys; signal.signal(signal.SIGTERM,signal.SIG_IGN); print('READY',flush=True); sys.stdin.readline()"])
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().unwrap();
        let pid = child.id();
        let mut ready = String::new();
        BufReader::new(child.stdout.take().unwrap()).read_line(&mut ready).unwrap();
        assert_eq!(ready.trim(), "READY");
        let mut release = child.stdin.take().unwrap();
        let mut owner = if before_barrier { Some(child) } else { drop(child); None };
        let monitor_root = root.clone();
        let monitor = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(15);
            let mut recorded = Vec::new();
            while Instant::now() < deadline {
                if let Ok(files) = std::fs::read_dir(monitor_root.join("sessions/agent-221")) {
                    for file in files.flatten() {
                        recorded.extend(bridget_transport::journal::valid_events(&file.path()));
                    }
                }
                if !recorded.is_empty() { break; }
                thread::sleep(Duration::from_millis(20));
            }
            let premature_proof = std::fs::read_dir(monitor_root.join("markers")).unwrap()
                .any(|file| file.unwrap().path().extension().is_some_and(|v|v=="drained"));
            let alive_before_release = crate::managed_process::process_birth(pid).is_ok();
            // Toujours libérer le processus privé, même si le diagnostic manque.
            let _ = release.write_all(b"release
");
            drop(release);
            (recorded, premature_proof, alive_before_release)
        });
        let command = bridget_transport::protocol::WorkerCommandIdentity {
            command_id:"command-221".into(),task_id:"task-221".into(),instance_id:"instance-221".into(),generation:7,
        };
        let proof = serde_json::from_value(serde_json::json!({
            "version":1,"worker_digest":"a".repeat(64),"command_digest":"b".repeat(64),
            "marker_digest":"c".repeat(64),"pgid":std::process::id(),"birth":1
        })).unwrap();
        let guard = RemoteDrainGuard {
            root:root.clone(),command,proof,machine:"machine-221".into(),
            journal:Mutex::new(None),
            signaler:Mutex::new(None),
        };
        guard.set_journal(&root.join("sessions"), "old-agent", "instance-221", None);
        guard.set_journal(&root.join("sessions"), "agent-221", "provider-session-221", journal_writer.clone());
        if let Some(child) = &mut owner {
            guard.begin_stop();
            child.wait().unwrap(); // Le lecteur/propriétaire retient réellement la barrière.
        }
        drop(owner);
        if let Some(writer) = &journal_writer { writer.stop(); }
        drop(guard); // Vraies phases de production, pas une copie dans le test.
        let (events, premature_proof, alive) = monitor.join().unwrap();
        assert!(!root.join("sessions/old-agent").exists(), "le diagnostic suit le journal réellement activé");
        assert_eq!(events.len(),1,"une seule phase dépasse le délai dans chaque fixture");
        assert_eq!(events[0]["payload"]["cause"], if before_barrier { "owners_waiting" } else { "descendants_alive" });
        assert_eq!(events[0]["payload"]["phase"], if before_barrier { "closing_transport" } else { "draining_descendants" });
        assert_eq!(events[0]["session_id"], "provider-session-221", "le diagnostic garde l'identité du journal fournisseur");
        assert!(!premature_proof && alive,"le diagnostic n'abandonne ni ne clôt le survivant");
        assert!(crate::managed_process::process_birth(pid).is_err());
        assert_eq!(std::fs::read_dir(&markers).unwrap().count(),1,"preuve seulement après récolte complète");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn spec221_signaler_creation_refusal_is_explicit_without_retry() {
        const HELPER: &str = "BRIDGET_SPEC221_DRAIN_SPAWN_REFUSAL";
        if std::env::var_os(HELPER).is_none() {
            let status = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "wrapper::reconnect_tests::spec221_signaler_creation_refusal_is_explicit_without_retry", "--nocapture", "--test-threads=1"])
                .env(HELPER, "1").status().unwrap();
            assert!(status.success());
            return;
        }
        assert_ne!(bridget_transport::fsutil::effective_uid(), 0, "recette non privilégiée");
        crate::managed_process::remote_drain::arm().unwrap();
        let root = relay_root("spec221-signaler-unavailable");
        std::fs::create_dir_all(root.join("markers")).unwrap();
        use std::os::unix::fs::PermissionsExt;
        for directory in [&root, &root.join("markers")] {
            std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700)).unwrap();
        }
        let writer = JournalWriter::start(root.join("sessions"), "agent-221", "instance-221",
            Arc::new(Mutex::new(bridget_transport::AcpEventQueue::default()))).unwrap();
        let command = bridget_transport::protocol::WorkerCommandIdentity {
            command_id: "command-221".into(), task_id: "task-221".into(),
            instance_id: "instance-221".into(), generation: 7,
        };
        let proof = serde_json::from_value(serde_json::json!({
            "version":1,"worker_digest":"a".repeat(64),"command_digest":"b".repeat(64),
            "marker_digest":"c".repeat(64),"pgid":std::process::id(),"birth":1
        })).unwrap();
        let guard = RemoteDrainGuard {
            root:root.clone(),command,proof,machine:"machine-221".into(),
            journal:Mutex::new(None),signaler:Mutex::new(None),
        };
        guard.set_journal(&root.join("sessions"), "agent-221", "instance-221", Some(writer.clone()));
        let mut original = libc::rlimit { rlim_cur:0, rlim_max:0 };
        assert_eq!(unsafe { libc::getrlimit(libc::RLIMIT_NPROC, &mut original) }, 0);
        let limited = libc::rlimit { rlim_cur:0, rlim_max:original.rlim_max };
        assert_eq!(unsafe { libc::setrlimit(libc::RLIMIT_NPROC, &limited) }, 0);
        let started = Instant::now();
        guard.begin_stop(); // Vrai refus EAGAIN de création, sans injection de fonction.
        let elapsed = started.elapsed();
        assert_eq!(unsafe { libc::setrlimit(libc::RLIMIT_NPROC, &original) }, 0);
        assert!(elapsed < Duration::from_secs(1), "aucune boucle d'allocation");
        assert!(guard.signaler.lock().unwrap().is_none());
        writer.stop();
        let mut entries = Vec::new();
        for file in std::fs::read_dir(root.join("sessions/agent-221")).unwrap() {
            entries.extend(bridget_transport::journal::valid_events(&file.unwrap().path()));
        }
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["payload"]["cause"], "signaler_unavailable");
        assert_eq!(std::fs::read_dir(root.join("markers")).unwrap().count(), 0, "le refus ne vaut pas preuve d'arrêt");
        drop(guard); // Aucun enfant : seule la récolte complète autorise la preuve fixture.
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn spec221_drain_refusal_is_native_bounded_and_not_a_stop_proof() {
        let root = relay_root("spec221-drain-refusal");
        let command = bridget_transport::protocol::WorkerCommandIdentity {
            command_id: "command-221".into(), task_id: "task-221".into(),
            instance_id: "instance-221".into(), generation: 7,
        };
        for cause in ["signal_refused", "descendants_alive", "reap_unavailable", "proof_store_unavailable", "owners_waiting", "signaler_unavailable"] {
            record_remote_drain_wait(&root, "agent-221", "instance-221", &command, "machine-221", cause).unwrap();
        }
        assert!(record_remote_drain_wait(&root, "agent-221", "instance-221", &command, "machine-221", "PRIVATE_PROVIDER_MESSAGE").is_err());
        assert!(remote_drain_wait_payload(&command, "machine-221", "PRIVATE_PHASE", "owners_waiting").is_err());
        let mut entries = Vec::new();
        for file in std::fs::read_dir(root.join("agent-221")).unwrap() {
            entries.extend(bridget_transport::journal::valid_events(&file.unwrap().path()));
        }
        assert_eq!(entries.len(), 6);
        for entry in entries {
            assert_eq!(entry["event"], "error");
            let payload = &entry["payload"];
            assert_eq!(payload["code"], "native_drain_stop_unconfirmed");
            assert_eq!(payload["scope"], "task");
            assert_eq!(payload["task_id"], "task-221");
            assert_eq!(payload["generation"], 7);
            assert!(payload["next_action"].as_str().unwrap().contains("Conservez"));
            assert!(payload["other_work"].as_str().unwrap().contains("autres travaux"));
            assert!(payload.get("cleanup_done").is_none());
            assert!(payload.get("terminal").is_none());
            assert!(!entry.to_string().contains("PRIVATE_PROVIDER_MESSAGE"));
            assert!(entry.to_string().len() < 2048);
        }
        assert!(!root.join("markers").exists(), "le diagnostic ne publie aucune preuve d'arrêt");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn relais_ne_lit_jamais_le_journal_sans_abonne() {
        let root = relay_root("sans-abonne");
        std::fs::create_dir_all(&root).unwrap();
        let reads = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let probe = reads.clone();
        let hooks = AttachRelayHooks {
            before_command: Arc::new(|| {}),
            before_read: Arc::new(move || {
                probe.fetch_add(1, Ordering::SeqCst);
            }),
            ..AttachRelayHooks::default()
        };
        let (_, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            1,
            emitter,
            hooks,
        );
        thread::sleep(Duration::from_millis(30));
        assert_eq!(reads.load(Ordering::SeqCst), 0);
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec101_relay_reports_upstream_saturation_before_remaining_facts() {
        let root = relay_root("spec101-upstream-gap");
        let feed = JournalLiveFeed::default();
        // Trois flushs de100 évitent de saturer le writer lui-même ; seule la
        // file indépendante de256 observations reste volontairement non lue.
        for _ in 0..3 {
            let writer = JournalWriter::start_with_live_feed(
                &root,
                "agent",
                "session",
                Arc::new(Mutex::new(bridget_transport::AcpEventQueue::default())),
                Some(feed.clone()),
            )
            .unwrap();
            for _ in 0..100 {
                writer
                    .enqueue(
                        "turn_end",
                        Some("human"),
                        serde_json::json!({"stop_reason":"completed"}),
                    )
                    .unwrap();
            }
            writer.stop();
        }
        let (tx, rx) = mpsc::channel();
        let mut relay = AttachRelayWorker::start_with_clock(
            root.join("agent"),
            Arc::new(|| "2026-09-16".into()),
            1,
            Some(feed),
            Arc::new(move |message| {
                let _ = tx.send(message);
            }),
            AttachRelayHooks::default(),
        );
        assert!(matches!(
            rx.recv_timeout(Duration::from_secs(2)).unwrap(),
            WrapperToDaemon::ObservationGap { dropped: 44 }
        ));
        for seq in 1..=256 {
            assert!(
                matches!(rx.recv_timeout(Duration::from_secs(2)).unwrap(), WrapperToDaemon::ObservedActivity { seq: actual, .. } if actual == seq)
            );
        }
        relay.shutdown();
    }

    #[test]
    fn spec100_relay_observes_without_attach_view_or_disk_reader() {
        let root = relay_root("spec100-without-view");
        let feed = JournalLiveFeed::default();
        let writer = JournalWriter::start_with_live_feed(
            &root,
            "agent",
            "session",
            Arc::new(Mutex::new(bridget_transport::AcpEventQueue::default())),
            Some(feed.clone()),
        )
        .unwrap();
        let (tx, rx) = mpsc::channel();
        let reads = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let probe = reads.clone();
        let mut relay = AttachRelayWorker::start_with_clock(
            root.join("agent"),
            Arc::new(|| "2026-09-16".into()),
            1,
            Some(feed),
            Arc::new(move |message| {
                let _ = tx.send(message);
            }),
            AttachRelayHooks {
                before_read: Arc::new(move || {
                    probe.fetch_add(1, Ordering::SeqCst);
                }),
                ..AttachRelayHooks::default()
            },
        );
        writer
            .enqueue(
                "turn_end",
                Some("human"),
                serde_json::json!({"stop_reason":"completed"}),
            )
            .unwrap();
        writer.stop();
        let message = rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(matches!(
            message,
            WrapperToDaemon::ObservedActivity {
                seq: 1,
                event: bridget_transport::protocol::ObservationKind::TurnEnded,
                ..
            }
        ));
        assert_eq!(reads.load(Ordering::SeqCst), 0);
        relay.shutdown();
    }

    #[test]
    #[allow(non_snake_case)]
    fn TEMOIN_relay_live_update_sans_kind_est_refuse_sans_interblocage() {
        // Gate daemon : un update relay sans payload.kind doit échouer
        // explicitement (Err), jamais interbloquer via panic+barrière worker.
        let journal_root = relay_root("kind-required");
        let journal_events = Arc::new(Mutex::new(bridget_transport::AcpEventQueue::default()));
        let writer = JournalWriter::start_with_live_feed(
            &journal_root,
            "agent-kind",
            "session-kind",
            journal_events,
            None,
        )
        .unwrap();
        let rejected = writer.enqueue("update", None, serde_json::json!({"content":"live"}));
        assert!(
            rejected
                .as_ref()
                .is_err_and(|detail| detail.contains("sans payload.kind")),
            "refus explicite attendu, got {rejected:?}"
        );
        writer.stop();
        let _ = std::fs::remove_dir_all(journal_root);
    }

    #[test]
    fn bascule_snapshot_vers_live_preserve_la_continuite_sans_doublon() {
        let journal_root = relay_root("bascule-live");
        let root = journal_root.join("agent-live");
        std::fs::create_dir_all(&root).unwrap();
        let date = current_host_date();
        std::fs::write(root.join(format!("{date}.jsonl")), b"{\"v\":1,\"seq\":1}\n").unwrap();
        let feed = JournalLiveFeed::default();
        let journal_events = Arc::new(Mutex::new(bridget_transport::AcpEventQueue::default()));
        let writer = JournalWriter::start_with_live_feed(
            &journal_root,
            "agent-live",
            "session-live",
            journal_events,
            Some(feed.clone()),
        )
        .unwrap();
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with_live(
            root.clone(),
            date,
            4,
            feed.clone(),
            emitter,
            AttachRelayHooks::default(),
        );
        worker
            .subscribe("sub-live".to_string(), AttachWindow::Seq(0))
            .unwrap();
        wait_for(|| {
            let messages = events.lock().unwrap();
            messages.iter().any(|message| {
                matches!(message, WrapperToDaemon::SnapshotCaughtUp { subscription_id, through_seq: Some(1) } if subscription_id == "sub-live")
            })
        });

        writer
            .enqueue(
                "update",
                None,
                serde_json::json!({"kind":"text","content":"live"}),
            )
            .unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| {
                matches!(
                    message,
                    WrapperToDaemon::LiveJournalFragment {
                        seq: 2,
                        final_fragment: true,
                        ..
                    }
                )
            })
        });

        let seqs = events
            .lock()
            .unwrap()
            .iter()
            .filter_map(|message| match message {
                WrapperToDaemon::JournalFragment {
                    subscription_id,
                    seq,
                    final_fragment: true,
                    ..
                } if subscription_id == "sub-live" => Some(*seq),
                WrapperToDaemon::LiveJournalFragment {
                    seq,
                    final_fragment: true,
                    ..
                } => Some(*seq),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(seqs, vec![1, 2]);
        assert!(!events.lock().unwrap().iter().any(|message| matches!(
            message,
            WrapperToDaemon::Gap { subscription_id, .. } if subscription_id == "sub-live"
        )));
        writer.stop();
        worker.shutdown();
        std::fs::remove_dir_all(journal_root).unwrap();
    }

    #[test]
    fn perte_du_flux_live_signale_un_gap_puis_rattrape_le_journal() {
        let journal_root = relay_root("gap-live");
        let root = journal_root.join("agent-gap");
        std::fs::create_dir_all(&root).unwrap();
        let date = current_host_date();
        let path = root.join(format!("{date}.jsonl"));
        std::fs::write(&path, b"{\"v\":1,\"seq\":1}\n").unwrap();
        let feed = JournalLiveFeed::new(1);
        let journal_events = Arc::new(Mutex::new(bridget_transport::AcpEventQueue::default()));
        let writer = JournalWriter::start_with_live_feed(
            &journal_root,
            "agent-gap",
            "session-gap",
            journal_events,
            Some(feed.clone()),
        )
        .unwrap();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let arm = Arc::new(AtomicBool::new(false));
        let worker_barrier = Arc::clone(&barrier);
        let worker_arm = Arc::clone(&arm);
        let hooks = AttachRelayHooks {
            before_live_read: Arc::new(move || {
                if worker_arm.swap(false, Ordering::SeqCst) {
                    worker_barrier.wait();
                    worker_barrier.wait();
                }
            }),
            ..AttachRelayHooks::default()
        };
        let (events, emitter) = relay_emitter();
        let mut worker =
            AttachRelayWorker::start_with_live(root.clone(), date, 4, feed.clone(), emitter, hooks);
        worker
            .subscribe("sub-gap".to_string(), AttachWindow::Seq(0))
            .unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| {
                matches!(message, WrapperToDaemon::SnapshotCaughtUp { subscription_id, through_seq: Some(1) } if subscription_id == "sub-gap")
            })
        });

        arm.store(true, Ordering::SeqCst);
        barrier.wait();
        writer
            .enqueue(
                "update",
                None,
                serde_json::json!({"kind":"text","content":"deux"}),
            )
            .unwrap();
        writer
            .enqueue(
                "update",
                None,
                serde_json::json!({"kind":"text","content":"trois"}),
            )
            .unwrap();
        writer.stop();
        barrier.wait();

        wait_for(|| {
            let messages = events.lock().unwrap();
            messages.iter().any(|message| {
                matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 3, final_fragment: true, .. } if subscription_id == "sub-gap")
            })
        });
        let messages = events.lock().unwrap();
        let gaps = messages
            .iter()
            .filter(|message| {
                matches!(
                    message,
                    WrapperToDaemon::Gap {
                        subscription_id,
                        from_seq: 2,
                        to_seq: 3,
                        reason: Some(reason),
                    } if subscription_id == "sub-gap" && reason == "live_feed_overrun"
                )
            })
            .count();
        assert_eq!(gaps, 1, "la perte mémoire doit être coalescée");
        let seqs = messages
            .iter()
            .filter_map(|message| match message {
                WrapperToDaemon::JournalFragment {
                    subscription_id,
                    seq,
                    final_fragment: true,
                    ..
                } if subscription_id == "sub-gap" => Some(*seq),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(seqs, vec![1, 2, 3]);
        drop(messages);
        worker.shutdown();
        std::fs::remove_dir_all(journal_root).unwrap();
    }

    #[test]
    fn relais_refuse_la_saturation_et_le_controle_passe_pendant_un_rejeu_suspendu() {
        let root = relay_root("controle");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("2026-08-22.jsonl"), b"{\"v\":1,\"seq\":1}\n").unwrap();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let first_read = Arc::new(AtomicBool::new(true));
        let barrier_for_worker = barrier.clone();
        let first_for_worker = first_read.clone();
        let (control_sender, control_receiver) = mpsc::channel();
        let hooks = AttachRelayHooks {
            before_command: Arc::new(|| {}),
            before_read: Arc::new(move || {
                if first_for_worker.swap(false, Ordering::SeqCst) {
                    barrier_for_worker.wait();
                    barrier_for_worker.wait();
                }
            }),
            before_live_read: Arc::new(|| {}),
            control_observed: Arc::new(move || {
                let _ = control_sender.send(());
            }),
        };
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            1,
            emitter,
            hooks,
        );
        worker
            .subscribe("sub-1".to_string(), AttachWindow::Today)
            .unwrap();
        barrier.wait();
        worker
            .subscribe("sub-2".to_string(), AttachWindow::Today)
            .unwrap();
        assert_eq!(
            worker.subscribe("sub-3".to_string(), AttachWindow::Today),
            Err(AttachRefusal::CommandQueueSaturated)
        );
        worker.unsubscribe("sub-1".to_string());
        control_receiver
            .recv_timeout(Duration::from_millis(250))
            .expect("désabonnement reçu par le canal de contrôle");
        barrier.wait();
        thread::sleep(Duration::from_millis(30));
        assert!(!events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, .. } if subscription_id == "sub-1")));
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn relais_marque_une_fenetre_vide_sans_through_seq() {
        let root = relay_root("vide");
        std::fs::create_dir_all(&root).unwrap();
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            1,
            emitter,
            AttachRelayHooks::default(),
        );
        worker
            .subscribe("sub-vide".to_string(), AttachWindow::Today)
            .unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::SnapshotCaughtUp { subscription_id, through_seq: None } if subscription_id == "sub-vide"))
        });
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn relais_rejoue_la_rotation_par_tranches_et_conserve_la_continuite() {
        let root = relay_root("rotation");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("2026-08-22.jsonl"), b"{\"v\":1,\"seq\":5}\n").unwrap();
        std::fs::write(root.join("2026-08-23.jsonl"), b"{\"v\":1,\"seq\":6}\n").unwrap();
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-23".to_string(),
            1,
            emitter,
            AttachRelayHooks::default(),
        );
        worker
            .subscribe("sub-rotation".to_string(), AttachWindow::Seq(5))
            .unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::SnapshotCaughtUp { subscription_id, through_seq: Some(6) } if subscription_id == "sub-rotation"))
        });
        std::fs::write(root.join("2026-08-24.jsonl"), b"{\"v\":1,\"seq\":7}\n").unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { seq: 7, subscription_id, .. } if subscription_id == "sub-rotation"))
        });
        let sequences = events
            .lock()
            .unwrap()
            .iter()
            .filter_map(|message| match message {
                WrapperToDaemon::JournalFragment {
                    seq,
                    subscription_id,
                    final_fragment: true,
                    ..
                } if subscription_id == "sub-rotation" => Some(*seq),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(sequences, vec![5, 6, 7]);
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec221_journal_read_error_releases_id_before_resubscribe() {
        let root=relay_root("read-error-reuse");std::fs::create_dir_all(&root).unwrap();
        let path=root.join("2026-08-22.jsonl");
        std::fs::write(&path,b"{\"v\":1,\"seq\":1}\n").unwrap();
        let hook_path=path.clone();let once=Arc::new(AtomicBool::new(true));let hook_once=once.clone();
        let hooks=AttachRelayHooks {before_read:Arc::new(move||{
            if hook_once.swap(false,Ordering::SeqCst) {
                std::fs::remove_file(&hook_path).unwrap();std::fs::create_dir(&hook_path).unwrap();
            }
        }),..AttachRelayHooks::default()};
        let (events,base_emitter)=relay_emitter();
        let slot=Arc::new(Mutex::new(None::<AttachRelayControl>));let callback_slot=slot.clone();
        let emitter:RelayEmitter=Arc::new(move|event| {
            if let WrapperToDaemon::End {subscription_id,..}=&event
                && let Some(control)=callback_slot.lock().unwrap().as_ref() {
                    control.unsubscribe(subscription_id.clone());
                }
            base_emitter(event);
        });
        let mut worker=AttachRelayWorker::start_with(root.clone(),"2026-08-22".into(),4,emitter,hooks);
        *slot.lock().unwrap()=Some(worker.subscription_control());
        worker.subscribe("same-id".into(),AttachWindow::Today).unwrap();
        wait_for(||events.lock().unwrap().iter().any(|event|
            matches!(event,WrapperToDaemon::End {subscription_id,..} if subscription_id=="same-id")));
        // End peut être émis juste avant le retrait partagé : attendre le fait.
        wait_for(||!worker.control_state.lock().unwrap().active.contains("same-id"));
        worker.unsubscribe("same-id".into());
        assert!(!worker.control_state.lock().unwrap().cancelled.contains("same-id"));
        std::fs::remove_dir(&path).unwrap();
        std::fs::write(&path,b"{\"v\":1,\"seq\":1}\n").unwrap();
        worker.subscribe("same-id".into(),AttachWindow::Today).unwrap();
        wait_for(||events.lock().unwrap().iter().any(|event|
            matches!(event,WrapperToDaemon::SnapshotCaughtUp {subscription_id,..} if subscription_id=="same-id")));
        assert_eq!(events.lock().unwrap().iter().filter(|event|
            matches!(event,WrapperToDaemon::Subscribed {subscription_id} if subscription_id=="same-id")).count(),2);
        worker.shutdown();std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unsubscribe_avant_consommation_annule_le_subscribe_sans_fuite() {
        let root = relay_root("unsubscribe-avant-consommation");
        std::fs::create_dir_all(&root).unwrap();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let first_command = Arc::new(AtomicBool::new(true));
        let barrier_for_worker = barrier.clone();
        let first_for_worker = first_command.clone();
        let hooks = AttachRelayHooks {
            before_command: Arc::new(move || {
                if first_for_worker.swap(false, Ordering::SeqCst) {
                    barrier_for_worker.wait();
                    barrier_for_worker.wait();
                }
            }),
            ..AttachRelayHooks::default()
        };
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            1,
            emitter,
            hooks,
        );
        barrier.wait();
        worker
            .subscribe("sub-annule".to_string(), AttachWindow::Today)
            .unwrap();
        worker.unsubscribe("sub-annule".to_string());
        for index in 0..(ATTACH_RELAY_CONTROL_IDS_CAPACITY * 4) {
            worker.unsubscribe("sub-annule".to_string());
            worker.unsubscribe(format!("inconnu-{index}"));
        }
        let state = worker
            .control_state
            .lock()
            .recover_poison();
        assert_eq!(state.pending.len(), 1);
        assert!(state.active.is_empty());
        assert_eq!(state.cancelled.len(), 1);
        drop(state);
        barrier.wait();
        thread::sleep(Duration::from_millis(30));
        assert!(!events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::Subscribed { subscription_id } if subscription_id == "sub-annule")));
        thread::sleep(Duration::from_millis(30));
        let state = worker
            .control_state
            .lock()
            .recover_poison();
        assert!(state.pending.is_empty());
        assert!(state.active.is_empty());
        assert!(state.cancelled.is_empty());
        drop(state);
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn shutdown_reste_borne_malgre_un_spam_de_commandes() {
        let root = relay_root("stop-prioritaire");
        std::fs::create_dir_all(&root).unwrap();
        let (_, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            1,
            emitter,
            AttachRelayHooks::default(),
        );
        let sender = worker.commands.clone();
        let running = Arc::new(AtomicBool::new(true));
        let sending = running.clone();
        let spam = thread::spawn(move || {
            let mut index = 0_u64;
            while sending.load(Ordering::SeqCst) {
                let _ = sender.try_send(AttachRelayCommand::Subscribe {
                    subscription_id: format!("spam-{index}"),
                    window: AttachWindow::Today,
                });
                index = index.saturating_add(1);
            }
        });
        let started = std::time::Instant::now();
        worker.shutdown();
        running.store(false, Ordering::SeqCst);
        spam.join().unwrap();
        assert!(
            started.elapsed() < Duration::from_millis(250),
            "le Stop ne doit pas attendre la file saturée"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    /// Oracle : si le worker est coincé dans un hook non coopératif,
    /// `shutdown` DOIT quand même rendre la main dans `ATTACH_RELAY_SHUTDOWN_BOUND`.
    #[test]
    fn shutdown_rend_la_main_dans_sa_borne_si_le_hook_commande_bloque() {
        let root = relay_root("stop-borne-hook");
        std::fs::create_dir_all(&root).unwrap();
        let (entered_tx, entered_rx) = std::sync::mpsc::sync_channel(1);
        let (release_tx, release_rx) = std::sync::mpsc::sync_channel(1);
        let (finished_tx, finished_rx) = std::sync::mpsc::sync_channel(1);
        let release_rx = Mutex::new(release_rx);
        let hooks = AttachRelayHooks {
            before_command: Arc::new(move || {
                let _ = entered_tx.try_send(());
                // Bloqué DURANT shutdown, libérable ensuite : l'oracle ne
                // laisse plus de thread volontairement orphelin dans la suite.
                let _ = release_rx
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(10));
                let _ = finished_tx.try_send(());
            }),
            ..AttachRelayHooks::default()
        };
        let (_, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            2,
            emitter,
            hooks,
        );
        entered_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("hook effectivement atteint");
        let started = Instant::now();
        worker.shutdown();
        let elapsed = started.elapsed();
        release_tx.send(()).unwrap();
        finished_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("hook libéré après shutdown");
        assert!(
            elapsed <= ATTACH_RELAY_SHUTDOWN_BOUND + Duration::from_millis(250),
            "shutdown n'a pas rendu la main dans sa borne ({:?})",
            ATTACH_RELAY_SHUTDOWN_BOUND
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn lignes_illisibles_et_trop_grandes_signalent_puis_laissent_progresser() {
        let root = relay_root("diagnostics-journal");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("2026-08-22.jsonl");
        std::fs::write(&path, b"{invalide}\n").unwrap();
        let mut too_large = b"{\"v\":1,\"seq\":8,\"payload\":\"".to_vec();
        too_large.extend(std::iter::repeat_n(b'x', 4 * 1024 * 1024));
        too_large.extend_from_slice(b"\"}\n{\"v\":1,\"seq\":9}\n");
        std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(&too_large)
            .unwrap();
        let (events, emitter) = relay_emitter();
        let reads = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let read_probe = reads.clone();
        let hooks = AttachRelayHooks {
            before_read: Arc::new(move || {
                read_probe.fetch_add(1, Ordering::SeqCst);
            }),
            ..AttachRelayHooks::default()
        };
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            2,
            emitter,
            hooks,
        );
        worker
            .subscribe("sub-diagnostic".to_string(), AttachWindow::Today)
            .unwrap();
        wait_for(|| {
            let messages = events.lock().unwrap();
            messages.iter().any(|message| matches!(message, WrapperToDaemon::JournalReadError { subscription_id, .. } if subscription_id == "sub-diagnostic"))
                && messages.iter().any(|message| matches!(message, WrapperToDaemon::Gap { from_seq: 8, to_seq: 8, reason: Some(reason), .. } if reason == "event_too_large"))
                && messages.iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { seq: 9, final_fragment: true, .. }))
        });
        assert!(reads.load(Ordering::SeqCst) > 1);
        worker.shutdown();
        assert_eq!(
            events
                .lock()
                .unwrap()
                .iter()
                .filter(|event| matches!(event, WrapperToDaemon::Gap { .. }))
                .count(),
            1,
            "une séquence trop grande n'est pas aussi annoncée comme absente au fragment suivant"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn queue_partielle_corrompue_devient_un_diagnostic_sans_bloquer_le_suivi() {
        let root = relay_root("queue-partielle");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("2026-08-22.jsonl");
        std::fs::write(&path, b"{incomplet").unwrap();
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            1,
            emitter,
            AttachRelayHooks::default(),
        );
        worker
            .subscribe("sub-partiel".to_string(), AttachWindow::Today)
            .unwrap();
        thread::sleep(Duration::from_millis(30));
        assert!(
            !events
                .lock()
                .unwrap()
                .iter()
                .any(|message| matches!(message, WrapperToDaemon::JournalReadError { .. }))
        );
        std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(b"\n{\"v\":1,\"seq\":10}\n")
            .unwrap();
        wait_for(|| {
            let messages = events.lock().unwrap();
            messages.iter().any(|message| matches!(message, WrapperToDaemon::JournalReadError { subscription_id, .. } if subscription_id == "sub-partiel"))
                && messages.iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 10, final_fragment: true, .. } if subscription_id == "sub-partiel"))
        });
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn today_et_date_suivent_la_rotation_apres_le_snapshot() {
        let root = relay_root("selecteurs-rotation");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("2026-08-22.jsonl"), b"{\"v\":1,\"seq\":1}\n").unwrap();
        let date = Arc::new(Mutex::new("2026-08-22".to_string()));
        let clock_date = date.clone();
        let clock: Arc<dyn Fn() -> String + Send + Sync> =
            Arc::new(move || clock_date.lock().unwrap().clone());
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with_clock(
            root.clone(),
            clock,
            4,
            None,
            emitter,
            AttachRelayHooks::default(),
        );
        worker
            .subscribe("sub-today".to_string(), AttachWindow::Today)
            .unwrap();
        worker
            .subscribe(
                "sub-date".to_string(),
                AttachWindow::Date("2026-08-22".to_string()),
            )
            .unwrap();
        wait_for(|| {
            events
                .lock()
                .unwrap()
                .iter()
                .filter(|message| matches!(message, WrapperToDaemon::SnapshotCaughtUp { .. }))
                .count()
                == 2
        });
        *date.lock().unwrap() = "2026-08-23".to_string();
        std::fs::write(root.join("2026-08-23.jsonl"), b"{\"v\":1,\"seq\":2}\n").unwrap();
        wait_for(|| {
            let messages = events.lock().unwrap();
            ["sub-today", "sub-date"].iter().all(|subscription_id| messages.iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id: actual, seq: 2, final_fragment: true, .. } if actual == subscription_id)))
        });
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn today_vide_ne_rejoue_pas_l_historique_anterieur() {
        let root = relay_root("today-vide");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("2026-08-21.jsonl"), b"{\"v\":1,\"seq\":1}\n").unwrap();
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            1,
            emitter,
            AttachRelayHooks::default(),
        );
        worker
            .subscribe("sub-today-vide".to_string(), AttachWindow::Today)
            .unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::SnapshotCaughtUp { subscription_id, through_seq: None } if subscription_id == "sub-today-vide"))
        });
        std::fs::write(root.join("2026-08-22.jsonl"), b"{\"v\":1,\"seq\":2}\n").unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 2, final_fragment: true, .. } if subscription_id == "sub-today-vide"))
        });
        assert!(!events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 1, .. } if subscription_id == "sub-today-vide")));
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn troncature_et_remplacement_repartent_du_nouvel_etat() {
        let root = relay_root("source-reinitialisee");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("2026-08-22.jsonl");
        std::fs::write(&path, b"{\"v\":1,\"seq\":1,\"padding\":\"longue\"}\n").unwrap();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let arm = Arc::new(AtomicBool::new(false));
        let barrier_worker = barrier.clone();
        let arm_worker = arm.clone();
        let hooks = AttachRelayHooks {
            before_read: Arc::new(move || {
                if arm_worker.swap(false, Ordering::SeqCst) {
                    barrier_worker.wait();
                    barrier_worker.wait();
                }
            }),
            ..AttachRelayHooks::default()
        };
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            1,
            emitter,
            hooks,
        );
        worker
            .subscribe("sub-reset".to_string(), AttachWindow::Today)
            .unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 1, final_fragment: true, .. } if subscription_id == "sub-reset"))
        });

        arm.store(true, Ordering::SeqCst);
        barrier.wait();
        std::fs::write(&path, b"{\"v\":1,\"seq\":2}\n").unwrap();
        barrier.wait();
        wait_for(|| {
            let messages = events.lock().unwrap();
            messages.iter().any(|message| matches!(message, WrapperToDaemon::End { subscription_id, reason } if subscription_id == "sub-reset" && reason == "source_truncated"))
                && !messages.iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 2, .. } if subscription_id == "sub-reset"))
        });
        worker
            .subscribe("sub-troncature-reprise".to_string(), AttachWindow::Seq(2))
            .unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 2, final_fragment: true, .. } if subscription_id == "sub-troncature-reprise"))
        });

        arm.store(true, Ordering::SeqCst);
        barrier.wait();
        let replaced = root.join("previous.jsonl");
        std::fs::rename(&path, replaced).unwrap();
        std::fs::write(&path, b"{\"v\":1,\"seq\":3}\n").unwrap();
        barrier.wait();
        wait_for(|| {
            let messages = events.lock().unwrap();
            messages.iter().any(|message| matches!(message, WrapperToDaemon::End { subscription_id, reason } if subscription_id == "sub-troncature-reprise" && reason == "source_replaced"))
                && !messages.iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 3, .. } if subscription_id == "sub-troncature-reprise"))
        });
        worker
            .subscribe("sub-remplacement-reprise".to_string(), AttachWindow::Seq(3))
            .unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 3, final_fragment: true, .. } if subscription_id == "sub-remplacement-reprise"))
        });
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn disparition_d_un_fichier_termine_l_abonnement_sans_boucle() {
        let root = relay_root("source-supprimee");
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("2026-08-22.jsonl");
        std::fs::write(&path, b"{\"v\":1,\"seq\":1}\n").unwrap();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let once = Arc::new(AtomicBool::new(true));
        let barrier_worker = barrier.clone();
        let once_worker = once.clone();
        let hooks = AttachRelayHooks {
            before_read: Arc::new(move || {
                if once_worker.swap(false, Ordering::SeqCst) {
                    barrier_worker.wait();
                    barrier_worker.wait();
                }
            }),
            ..AttachRelayHooks::default()
        };
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            1,
            emitter,
            hooks,
        );
        worker
            .subscribe("sub-source".to_string(), AttachWindow::Today)
            .unwrap();
        barrier.wait();
        std::fs::remove_file(path).unwrap();
        barrier.wait();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::End { subscription_id, reason } if subscription_id == "sub-source" && reason.contains("source de journal indisponible")))
        });
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn sorties_des_abonnements_restent_equitablement_entrelacees() {
        let root = relay_root("equite-sortie");
        std::fs::create_dir_all(&root).unwrap();
        let mut large = b"{\"v\":1,\"seq\":1,\"payload\":\"".to_vec();
        large.extend(std::iter::repeat_n(b'a', MAX_ATTACH_FRAGMENT_BYTES * 4));
        large.extend_from_slice(b"\"}\n");
        std::fs::write(root.join("2026-08-22.jsonl"), large).unwrap();
        std::fs::write(root.join("2026-08-23.jsonl"), b"").unwrap();
        let (events, emitter) = observable_relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-23".to_string(),
            4,
            emitter,
            AttachRelayHooks::default(),
        );
        worker
            .subscribe("a-gros".to_string(), AttachWindow::Seq(0))
            .unwrap();
        worker
            .subscribe(
                "b-live".to_string(),
                AttachWindow::Date("2026-08-23".to_string()),
            )
            .unwrap();
        wait_for_relay_state(&events, |messages| {
            messages.iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 1, .. } if subscription_id == "a-gros"))
                && messages.iter().any(|message| matches!(message, WrapperToDaemon::SnapshotCaughtUp { subscription_id, .. } if subscription_id == "b-live"))
        });
        std::fs::write(root.join("2026-08-24.jsonl"), b"{\"v\":1,\"seq\":2}\n").unwrap();
        wait_for_relay_state(&events, |messages| {
            messages.iter().filter(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 1, .. } if subscription_id == "a-gros")).count() >= 2
                && messages.iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 2, .. } if subscription_id == "b-live"))
                && messages.iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 1, final_fragment: true, .. } if subscription_id == "a-gros"))
        });
        let messages = events
            .messages
            .lock()
            .recover_poison();
        let b = messages.iter().position(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 2, .. } if subscription_id == "b-live")).expect("fragment B");
        let a_final = messages.iter().position(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 1, final_fragment: true, .. } if subscription_id == "a-gros")).expect("final A");
        assert!(b < a_final, "B doit passer avant le dernier fragment de A");
        drop(messages);
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reset_generation_purge_le_rejeu_avant_un_nouvel_abonnement() {
        let root = relay_root("reset-generation");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("2026-08-22.jsonl"), b"{\"v\":1,\"seq\":1}\n").unwrap();
        let (events, emitter) = relay_emitter();
        let mut worker = AttachRelayWorker::start_with(
            root.clone(),
            "2026-08-22".to_string(),
            2,
            emitter,
            AttachRelayHooks::default(),
        );
        worker
            .subscribe("ancienne-generation".to_string(), AttachWindow::Today)
            .unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 1, .. } if subscription_id == "ancienne-generation"))
        });
        worker.reset_generation();
        std::fs::write(root.join("2026-08-23.jsonl"), b"{\"v\":1,\"seq\":2}\n").unwrap();
        thread::sleep(Duration::from_millis(50));
        assert!(!events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 2, .. } if subscription_id == "ancienne-generation")));
        worker
            .subscribe("nouvelle-generation".to_string(), AttachWindow::Seq(2))
            .unwrap();
        wait_for(|| {
            events.lock().unwrap().iter().any(|message| matches!(message, WrapperToDaemon::JournalFragment { subscription_id, seq: 2, final_fragment: true, .. } if subscription_id == "nouvelle-generation"))
        });
        worker.shutdown();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reconnect_delay_grows_then_stays_capped() {
        let first = reconnect_delay(0);
        let second = reconnect_delay(1);
        let sixth = reconnect_delay(5);
        let later = reconnect_delay(20);

        assert!(first <= Duration::from_millis(1200));
        assert!(second >= Duration::from_millis(1600));
        assert!(sixth <= RECONNECT_MAX_DELAY);
        assert!(later <= RECONNECT_MAX_DELAY);
    }

    #[test]
    fn reconnect_delay_plafonne_sans_regrouper_la_gigue() {
        let samples: Vec<Duration> = (0..256).map(|_| reconnect_delay(6)).collect();
        assert!(
            samples
                .iter()
                .all(|delay| (Duration::from_secs(24)..=RECONNECT_MAX_DELAY).contains(delay))
        );
        assert!(samples.iter().filter(|delay| **delay == RECONNECT_MAX_DELAY).count() < 32);
    }

    #[test]
    fn transition_execution_ignore_un_message_inconnu_et_evenement_tardif() {
        let root = mcp_test_root("execution-correlation");
        std::fs::create_dir_all(&root).unwrap();
        let (writer_stream, reader_stream) = UnixStream::pair().unwrap();
        reader_stream
            .set_read_timeout(Some(Duration::from_millis(30)))
            .unwrap();
        let writer = Arc::new(Mutex::new(Some(BufWriter::new(writer_stream))));
        let mut bindings = HashMap::new();
        bindings.insert(
            "message-active".to_string(),
            ManagedExecutionBinding {
                execution_id: "execution-active".to_string(),
                generation: 3,
                provider_kind: "fixture".to_string(),
                execution_path: "fixture".to_string(),
                state: "starting".to_string(),
                approval_requests: 0,
                last_approval_request: None,
                remote_result: None,
                revision: 0,
            },
        );

        publish_execution_transition(
            &writer,
            &mut bindings,
            "message-obsolete",
            "running",
            "late",
        );

        let mut reader = BufReader::new(reader_stream);
        let mut line = String::new();
        let error = reader.read_line(&mut line).unwrap_err();
        assert!(matches!(
            error.kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        ));
        assert_eq!(bindings["message-active"].state, "starting");

        publish_execution_transition(
            &writer,
            &mut bindings,
            "message-active",
            "running",
            "provider_accepted",
        );
        reader.read_line(&mut line).unwrap();
        assert!(matches!(
            decode(line.trim_end()).unwrap(),
            WrapperToDaemon::ExecutionStateChanged { transition }
                if transition.execution_id == "execution-active"
                    && transition.expected_state == "starting"
                    && transition.expected_revision == 0
                    && transition.next_state == "running"
        ));
        assert_eq!(bindings["message-active"].revision, 1);

        line.clear();
        publish_execution_transition(
            &writer,
            &mut bindings,
            "message-active",
            "running",
            "duplicate",
        );
        let duplicate = reader.read_line(&mut line).unwrap_err();
        assert!(matches!(
            duplicate.kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        ));

        line.clear();
        publish_execution_transition(
            &writer,
            &mut bindings,
            "message-active",
            "completed",
            "completed",
        );
        reader.read_line(&mut line).unwrap();
        assert!(matches!(
            decode(line.trim_end()).unwrap(),
            WrapperToDaemon::ExecutionStateChanged { transition }
                if transition.expected_state == "running"
                    && transition.expected_revision == 1
                    && transition.next_state == "completed"
        ));
        assert!(!bindings.contains_key("message-active"));

        line.clear();
        publish_execution_transition(&writer, &mut bindings, "message-active", "failed", "late");
        let late = reader.read_line(&mut line).unwrap_err();
        assert!(matches!(
            late.kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        ));

        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn evenement_terminal_tardif_n_emet_ni_etat_ni_reponse() {
        let root = mcp_test_root("terminal-tardif");
        std::fs::create_dir_all(&root).unwrap();
        let (writer_stream, reader_stream) = UnixStream::pair().unwrap();
        reader_stream
            .set_read_timeout(Some(Duration::from_millis(30)))
            .unwrap();
        let writer = Arc::new(Mutex::new(Some(BufWriter::new(writer_stream))));
        let mut tracker = IdempotentDeliveryTracker::open_at(&root, "terminal-tardif").unwrap();
        let mut bindings = HashMap::from([(
            "message-active".to_string(),
            ManagedExecutionBinding {
                execution_id: "execution-active".to_string(),
                provider_kind: "fixture".to_string(),
                execution_path: "fixture".to_string(),
                generation: 3,
                state: "running".to_string(),
                approval_requests: 0,
                last_approval_request: None,
                remote_result: None,
                revision: 1,
            },
        )]);
        let mut obsolete = bridget_core::BridgetMessage::new("humain", "agent", "message ancien");
        obsolete.id = "message-obsolete".to_string();

        assert!(!forward_managed_events(
            &writer,
            "agent",
            vec![ManagedEvent::internal(
                bridget_transport::ManagedEventSource::Acp,
                b"terminal tardif".to_vec(),
                ManagedEventKind::TurnFinished {
                    message: obsolete,
                    response: "réponse qui ne doit jamais repartir".to_string(),
                    terminal: ManagedTerminal::Completed,
                },
            )],
            &mut tracker,
            &mut bindings,
        ));

        assert_eq!(bindings["message-active"].state, "running");
        assert_eq!(bindings["message-active"].revision, 1);
        let mut reader = BufReader::new(reader_stream);
        let mut line = String::new();
        let error = reader.read_line(&mut line).unwrap_err();
        assert!(matches!(
            error.kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        ));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn boucles_d_autorisation_sont_bornees_par_une_issue_machine() {
        let (writer_stream, reader_stream) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(Some(BufWriter::new(writer_stream))));
        let mut bindings = HashMap::from([(
            "message-active".to_string(),
            ManagedExecutionBinding {
                execution_id: "execution-active".to_string(),
                provider_kind: "fixture".to_string(),
                execution_path: "fixture".to_string(),
                generation: 3,
                state: "running".to_string(),
                revision: 1,
                approval_requests: 0,
                last_approval_request: None,
                remote_result: None,
            },
        )]);

        for request_id in ["approval-1", "approval-2", "approval-3", "approval-4"] {
            publish_approval_wait(&writer, &mut bindings, request_id);
        }

        let mut reader = BufReader::new(reader_stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert!(matches!(
            decode(line.trim_end()).unwrap(),
            WrapperToDaemon::ExecutionStateChanged { transition }
                if transition.next_state == "waiting_approval"
                    && transition.reason == "permission_required"
        ));
        line.clear();
        reader.read_line(&mut line).unwrap();
        assert!(matches!(
            decode(line.trim_end()).unwrap(),
            WrapperToDaemon::ExecutionStateChanged { transition }
                if transition.next_state == "failed"
                    && transition.reason == "approval_loop_detected"
                    && transition.expected_state == "waiting_approval"
                    && transition.expected_revision == 2
        ));
        assert!(!bindings.contains_key("message-active"));
    }
    #[test]
    fn spec_079_binding_idempotent_precede_le_filtre_de_doublon() {
        let mut bindings = HashMap::new();
        assert!(!bind_idempotent_delivery_execution(
            &mut bindings,
            "message-079",
            None,
            "codex",
            "acp",
        ));
        assert!(bindings.is_empty());

        assert!(bind_idempotent_delivery_execution(
            &mut bindings,
            "message-079",
            Some(ExecutionDeliveryContext {
                execution_id: "execution-079".to_string(),
                generation: 2,
                revision: 3,
            }),
            "codex",
            "acp",
        ));
        let binding = bindings.get("message-079").expect("binding créé");
        assert_eq!(binding.execution_id, "execution-079");
        assert_eq!(binding.generation, 2);
        assert_eq!(binding.revision, 3);
        assert_eq!(binding.provider_kind, "codex");
        assert_eq!(binding.execution_path, "acp");

        assert!(bind_idempotent_delivery_execution(
            &mut bindings,
            "message-079",
            Some(ExecutionDeliveryContext {
                execution_id: "execution-divergente".to_string(),
                generation: 99,
                revision: 99,
            }),
            "claude",
            "tmux",
        ));
        assert_eq!(
            bindings["message-079"].execution_id, "execution-079",
            "un rejeu ne remplace jamais la première corrélation admise"
        );
    }
    #[test]
    fn edit159_promote_transport_file_ordinaire_pleine_indeterminee() {
        let root = receipt_root("edit159-full");
        let instance = "instance-edit159";
        let mut tracker = IdempotentDeliveryTracker::open_at(&root, instance).unwrap();
        let now = unix_now_secs();
        let mut promoted = idempotent_message("full159");
        promoted.intent = Some(bridget_core::MessageIntent::SteerCurrent);
        promoted.references.push("steer-or-queue".into());
        assert!(matches!(
            tracker.receive(
                "delivery-full159".into(),
                instance.into(),
                7,
                now + 60,
                promoted,
                now
            ),
            IdempotentDeliveryAction::Inject { .. }
        ));
        let (stream, peer) = UnixStream::pair().unwrap();
        peer.set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        let writer = Arc::new(Mutex::new(Some(BufWriter::new(stream))));
        assert!(!forward_managed_events(
            &writer,
            "equipier",
            vec![ManagedEvent::internal(
                bridget_transport::ManagedEventSource::CodexAppServer,
                Vec::new(),
                ManagedEventKind::DeliveryRejected {
                    message_id: "full159".into(),
                    reason: "file Codex pleine".into()
                },
            )],
            &mut tracker,
            &mut HashMap::new()
        ));
        let mut reader = BufReader::new(peer);
        let mut frames = Vec::new();
        for _ in 0..3 {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            frames.push(decode::<WrapperToDaemon>(line.trim()).unwrap());
        }
        assert!(frames.iter().any(|frame| matches!(frame, WrapperToDaemon::DeliveryIndeterminate { delivery_id, delivery_generation: 7 } if delivery_id == "delivery-full159")));
        assert!(frames.iter().any(|frame| matches!(frame, WrapperToDaemon::DeliveryRejected { id, reason, .. } if id == "full159" && reason == "file Codex pleine")));
        assert!(
            !frames
                .iter()
                .any(|frame| matches!(frame, WrapperToDaemon::DeliverAcked { .. }))
        );
        std::fs::remove_dir_all(root).unwrap();
    }
