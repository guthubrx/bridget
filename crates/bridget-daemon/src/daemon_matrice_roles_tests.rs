    use super::*;
    use std::io::{BufRead, BufReader, BufWriter, Write};
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::PathBuf;
    use std::sync::mpsc;
    use std::thread;

    #[test]
    fn metrics_execution_restent_bornees_et_sans_labels_de_contenu() {
        let metrics = Metrics::new();
        metrics.record_execution_admitted();
        metrics.record_execution_started(3);
        metrics.record_execution_transition_rejected();
        metrics.record_execution_delivery_failure();
        metrics.record_execution_queue_saturated();

        assert_eq!(metrics.execution_admitted.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.execution_started.load(Ordering::Relaxed), 1);
        assert_eq!(
            metrics
                .execution_start_latency_samples
                .load(Ordering::Relaxed),
            1
        );
        assert_eq!(
            metrics
                .execution_start_latency_secs_total
                .load(Ordering::Relaxed),
            3
        );
        assert_eq!(
            metrics
                .execution_transition_rejected
                .load(Ordering::Relaxed),
            1
        );
        assert_eq!(
            metrics.execution_delivery_failures.load(Ordering::Relaxed),
            1
        );
        assert_eq!(metrics.execution_queue_saturated.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.errors.load(Ordering::Relaxed), 2);
    }

    /// Etat minimal : on n'eprouve que la matrice, pas la presence.
    ///
    /// Le chemin porte l'horloge en plus du PID : deux executions successives
    /// du meme binaire ne doivent pas se disputer le meme fichier SQLite, sinon
    /// le banc devient intermittent et l'intermittence se paie plus tard.
    fn etat_nu(etiquette: &str) -> (Arc<Mutex<DaemonState>>, DaemonConfig) {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("horloge")
            .as_nanos();
        let base = std::env::temp_dir().join(format!(
            "bridget-{}-{}-{nanos}",
            etiquette,
            std::process::id()
        ));
        let config = DaemonConfig {
            socket_path: base.with_extension("sock"),
            db_path: base.with_extension("db"),
            log_path: base.with_extension("log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        };
        let (managed_tx, _managed_rx) = mpsc::channel();
        let state = Arc::new(Mutex::new(DaemonState::new(&config, managed_tx).unwrap()));
        (state, config)
    }

    #[test]
    fn spec_066_spawn_docker_refuse_avant_tout_lanceur_hote() {
        let (shared, config) = etat_nu("docker-no-fallback");
        let root =
            std::env::temp_dir().join(format!("bridget-066-docker-spawn-{}", Uuid::new_v4()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&root)
            .unwrap();
        let now = unix_now_secs();
        // Sans binding ni fournisseur déclaré : retirer la garde daemon mène
        // au refus de posture, pas au refus projet. L'oracle tue ce mutant.
        let response = handle_wrapper_message(
            "docker-spawn",
            WrapperToDaemon::SpawnOrder {
                posture: None,
                agent_type: "fixture".to_string(),
                project: Some(bridget_transport::protocol::ProjectReference {
                    project_id: "project-066".to_string(),
                    binding_generation: 1,
                }),
                agent_id: Some("89000000-0000-4000-8000-000000000066".to_string()),
                cwd: root.display().to_string(),
                persistent: false,
                command_id: "docker-spawn-command".to_string(),
                issued_at: now,
                deadline_at: now + 60,
                ownership: None,
            },
            &shared,
        );
        assert!(
            matches!(
                &response,
                Some(DaemonToWrapper::SpawnRejected {
                    reason: SpawnRefusal::DockerRuntimeUnavailable { project_id },
                    ..
                }) if project_id == "project-066"
            ),
            "refus attendu, reçu {response:?}"
        );
        assert!(shared.lock().unwrap().managed_spawns.is_empty());
        assert!(
            !shared
                .lock()
                .unwrap()
                .fleet
                .knows_command("docker-spawn-command")
        );
        let _ = std::fs::remove_file(config.db_path);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn spec_065_peer_uid_est_lu_sur_la_socket_unix_locale() {
        let (stream, _peer) = UnixStream::pair().unwrap();
        assert_eq!(unix_peer_uid(&stream).unwrap(), bridget_transport::fsutil::effective_uid());
    }

    #[test]
    fn attente_de_descendance_derive_le_parent_de_la_connexion_et_reprend_un_curseur() {
        let (shared, config) = etat_nu("attente-descendance");
        let parent_instance_id = "parent-atteste".to_string();
        let child = {
            let mut state = shared.lock().unwrap();
            state
                .conn_instances
                .insert("wrapper-parent".to_string(), parent_instance_id.clone());
            let order = FleetSpawnOrder {
                remote_placement: None,
                posture: None,
                agent_type: "fixture".to_string(),
                requested_name: Some("6b0c0eb7-f494-4f4a-a280-6a5f0d13c410".to_string()),
                cwd: PathBuf::from("/tmp"),
                persistent: false,
                command_id: "spawn-events-daemon".to_string(),
                issued_at: 1_788_000_000,
                deadline_at: 1_788_000_060,
                project: None,
                ownership: Some(bridget_transport::protocol::SpawnOwnership {
                    parent_instance_id,
                    parent_execution_id: Some("execution-parent".to_string()),
                    objective_id: Some("objective-1".to_string()),
                    delegation_id: None,
                    project: None,
                    role: "verification".to_string(),
                    max_children: Some(2),
                    max_depth: Some(3),
                }),
            };
            match state.fleet.request_spawn(&order, 1_788_000_000).unwrap() {
                crate::fleet::SpawnSubmission::Start(lease) => lease,
                other => panic!("réservation attendue : {other:?}"),
            }
        };

        let response = handle_wrapper_message(
            "wrapper-parent",
            WrapperToDaemon::WaitAgentLinks {
                after_cursor: None,
                timeout_ms: 0,
            },
            &shared,
        );
        assert!(matches!(
            response,
            Some(DaemonToWrapper::AgentLinkEvents {
                timed_out: false,
                through_cursor: Some(_),
                events,
            }) if events.len() == 1
                && events[0].child_instance_id == child.instance_id
                && events[0].state == "reserved"
        ));

        assert!(matches!(
            handle_wrapper_message(
                "wrapper-inconnu",
                WrapperToDaemon::WaitAgentLinks {
                    after_cursor: None,
                    timeout_ms: 0,
                },
                &shared,
            ),
            Some(DaemonToWrapper::Nack { reason, .. }) if reason == "wrapper non enregistré"
        ));

        let _ = std::fs::remove_file(config.db_path);
        let _ = std::fs::remove_file(config.socket_path);
        let _ = std::fs::remove_file(config.log_path);
    }

    struct NettoyageSondeIdentite {
        socket_path: PathBuf,
        db_path: PathBuf,
    }

    impl Drop for NettoyageSondeIdentite {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.socket_path);
            let _ = std::fs::remove_file(&self.db_path);
        }
    }

    struct NettoyageCheminInaccessible(PathBuf);

    impl Drop for NettoyageCheminInaccessible {
        fn drop(&mut self) {
            let _ = std::fs::set_permissions(&self.0, std::fs::Permissions::from_mode(0o700));
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn sert_une_identite(
        listener: UnixListener,
        state: Arc<Mutex<DaemonState>>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let (stream, _) = listener.accept().expect("connexion du client de sonde");
            handle_connection(stream, state).expect("service de la sonde");
        })
    }

    fn sert_un_rapport_ancien(listener: UnixListener) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let (stream, _) = listener.accept().expect("connexion du client ancien");
            let read_stream = stream.try_clone().expect("cloner le pair ancien");
            let mut reader = BufReader::new(read_stream);
            let mut writer = BufWriter::new(stream);
            let mut line = String::new();

            reader.read_line(&mut line).expect("lire le rôle");
            assert!(matches!(
                decode(line.trim()).expect("décoder le rôle"),
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Client
                }
            ));
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client
                })
                .expect("encoder l'acceptation")
            )
            .expect("répondre au rôle");
            writer.flush().expect("flush du rôle");

            line.clear();
            reader.read_line(&mut line).expect("lire le contrat");
            assert!(matches!(
                decode(line.trim()).expect("décoder le contrat"),
                WrapperToDaemon::ClientHello { .. }
            ));
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: "daemon-ancien".to_string(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: Vec::new(),
                })
                .expect("encoder l'accueil")
            )
            .expect("répondre au contrat");
            writer.flush().expect("flush du contrat");

            line.clear();
            reader.read_line(&mut line).expect("lire la sonde");
            assert!(matches!(
                decode(line.trim()).expect("décoder la sonde"),
                WrapperToDaemon::DaemonIdentityRequest
            ));
            // Forme réellement produite avant SPEC-048 : le daemon répond et
            // atteste sa présence, mais le champ d'instance récent manque.
            writeln!(
                writer,
                r#"{{"type":"DaemonIdentityReport","host":"ancien","db_path":"/ancien.db"}}"#
            )
            .expect("répondre avec l'ancien rapport");
            writer.flush().expect("flush de l'ancien rapport");
        })
    }

    #[test]
    fn absence_daemon_ancien_et_chemin_inaccessible_restent_distincts() {
        let chemin_absent = PathBuf::from(format!(
            "/tmp/bridget-identite-absente-{}-{}.sock",
            std::process::id(),
            Uuid::new_v4()
        ));
        assert!(
            daemon_identity(&chemin_absent)
                .expect("l'absence est observable")
                .is_none(),
            "une socket absente signifie qu'aucun daemon n'est observable"
        );

        let socket_path = PathBuf::from(format!(
            "/tmp/bridget-identite-ancienne-{}-{}.sock",
            std::process::id(),
            Uuid::new_v4()
        ));
        let _nettoyage = NettoyageSondeIdentite {
            socket_path: socket_path.clone(),
            db_path: socket_path.with_extension("db-inexistante"),
        };
        let listener = UnixListener::bind(&socket_path).expect("socket du daemon ancien");
        let serveur = sert_un_rapport_ancien(listener);
        let identite = daemon_identity(&socket_path)
            .expect("le daemon ancien répond")
            .expect("le daemon ancien est présent");
        serveur.join().expect("daemon ancien terminé");

        assert_eq!(identite.build_id, "daemon-ancien");
        assert!(identite.host.is_none());
        assert!(identite.db_path.is_none());
        assert!(identite.instance_id.is_none());

        let root_inaccessible = PathBuf::from(format!(
            "/tmp/bridget-identite-inaccessible-{}-{}",
            std::process::id(),
            Uuid::new_v4()
        ));
        std::fs::create_dir(&root_inaccessible).expect("créer le chemin inaccessible");
        let _nettoyage_inaccessible = NettoyageCheminInaccessible(root_inaccessible.clone());
        let socket_inaccessible = root_inaccessible.join("bridget.sock");
        let _listener = UnixListener::bind(&socket_inaccessible).expect("lier avant le refus");
        std::fs::set_permissions(&root_inaccessible, std::fs::Permissions::from_mode(0o000))
            .expect("rendre le chemin inaccessible");
        let error = daemon_identity(&socket_inaccessible)
            .expect_err("une erreur d'accès n'est pas une absence de daemon");
        assert!(error.contains("connexion impossible"), "{error}");
    }

    /// Oracle M1 local-vers-local : le socket est remplacé entre deux
    /// démarrages qui partagent délibérément hôte, binaire et base. BUILD_ID
    /// reste identique ; seule l'identité de démarrage doit changer.
    #[test]
    fn redemarrage_local_sur_le_meme_socket_change_l_identite_d_instance() {
        let (premier_etat, config) = etat_nu("identite-instance-redemarrage");
        let _nettoyage = NettoyageSondeIdentite {
            socket_path: config.socket_path.clone(),
            db_path: config.db_path.clone(),
        };
        let premier_listener = UnixListener::bind(&config.socket_path).expect("premier socket");
        let premier_serveur = sert_une_identite(premier_listener, premier_etat);
        let premier = daemon_identity(&config.socket_path)
            .expect("premiere sonde lisible")
            .expect("premier rapport d'identite");
        premier_serveur.join().expect("premier daemon termine");

        std::fs::remove_file(&config.socket_path).expect("retrait premier socket");
        let (managed_tx, _managed_rx) = mpsc::channel();
        let second_etat = Arc::new(Mutex::new(
            DaemonState::new(&config, managed_tx).expect("second daemon"),
        ));
        let second_listener = UnixListener::bind(&config.socket_path).expect("second socket");
        let second_serveur = sert_une_identite(second_listener, second_etat);
        let second = daemon_identity(&config.socket_path)
            .expect("seconde sonde lisible")
            .expect("second rapport d'identite");
        second_serveur.join().expect("second daemon termine");

        assert_eq!(premier.build_id, second.build_id, "meme binaire attendu");
        assert_eq!(premier.host, second.host, "meme hote attendu");
        assert_eq!(premier.db_path, second.db_path, "meme base attendue");
        let premier_id = premier
            .instance_id
            .as_deref()
            .expect("le premier rapport doit attester une instance");
        let second_id = second
            .instance_id
            .as_deref()
            .expect("le second rapport doit attester une instance");
        assert_ne!(
            premier_id, second_id,
            "un redemarrage local ne doit jamais reemployer l'identite d'instance"
        );
    }

    /// La sonde d'identite doit etre atteignable sur les DEUX roles.
    ///
    /// Le greffe se connecte en role Client pour deleguer, mais son guichet
    /// passe en role Service. Les deux matrices portaient le meme tiret bas et
    /// la variante y tombait : refus SILENCIEUX, identite non attestee, et une
    /// garde qui en conclurait « ne pas ecrire » — DEFINITIVEMENT. Une garde
    /// qui refuse toujours ressemble a une garde qui marche.
    ///
    /// Mutant qui tue ce test : reclasser `DaemonIdentityRequest` dans le bras
    /// de refus de l'un OU l'autre role -> le rejet apparait et l'assertion
    /// meurt en nommant le role fautif.
    #[test]
    fn spec221_c19_daemon_announces_reader_and_creator_without_creating_a_task() {
        let (shared,config)=etat_nu("c19-reader-feature");
        assert!(matches!(handle_wrapper_message("reader-feature",
            WrapperToDaemon::RoleHandshake{role:ConnectionRole::Client},&shared),
            Some(DaemonToWrapper::RoleAccepted{..})));
        let response=handle_wrapper_message("reader-feature",WrapperToDaemon::DaemonIdentityRequest,&shared);
        let Some(DaemonToWrapper::DaemonIdentityReport{features,..})=response
            else{panic!("identité attendue");};
        assert!(features.iter().any(|feature|feature=="commit_view_files_v1"));
        assert!(features.iter().any(|feature|feature=="worker_workspace_view_v1"));
        assert!(features.iter().any(|feature|feature=="worker_workspace_view_create_v1"));
        assert!(shared.lock().unwrap().delegation_store.tasks().unwrap().is_empty());
        drop(shared);let _=std::fs::remove_file(&config.db_path);
    }

    #[test]
    fn la_sonde_d_identite_est_atteignable_sur_les_roles_client_et_service() {
        let (shared, config) = etat_nu("sonde-identite");

        for (connexion, role) in [
            ("conn-client", ConnectionRole::Client),
            ("conn-service", ConnectionRole::Service),
        ] {
            assert!(matches!(
                handle_wrapper_message(connexion, WrapperToDaemon::RoleHandshake { role }, &shared,),
                Some(DaemonToWrapper::RoleAccepted { .. })
            ));

            // Demandee SANS negociation prealable : elle ne lit que ce que le
            // daemon atteste de lui-meme, elle n'ouvre aucun droit.
            let reponse =
                handle_wrapper_message(connexion, WrapperToDaemon::DaemonIdentityRequest, &shared);
            match reponse {
                Some(DaemonToWrapper::DaemonIdentityReport {
                    host,
                    db_path,
                    instance_id,
                    ..
                }) => {
                    // Les VALEURS, pas la presence : ce sont celles de l'etat.
                    let attendu = shared.lock().unwrap_or_else(|p| p.into_inner());
                    assert_eq!(host, attendu.host, "role {role:?}");
                    assert_eq!(
                        db_path,
                        attendu.db_path.display().to_string(),
                        "role {role:?}"
                    );
                    assert_eq!(instance_id, attendu.instance_id, "role {role:?}");
                }
                autre => panic!("role {role:?} : identite attendue, obtenu {autre:?}"),
            }
        }
        // Ne pas laisser de fichier derriere soi : c'est le meme reflexe que
        // pour les processus, et il a deja coute une soiree a ce chantier.
        let _ = std::fs::remove_file(&config.db_path);
    }
