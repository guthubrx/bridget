    use super::*;

    #[test]
    fn spec241_registration_reply_lost_recovers_before_one_mutation() {
        let identity = "24100000-0000-4000-8000-000000000241";
        let path = crate::mcp_identity::mock_socket("241-register-reply");
        let listener = UnixListener::bind(&path).unwrap();
        crate::mcp_identity::mock_private_identity(&path, identity, "instance-241");
        let server = thread::spawn(move || {
            let mut registrations = 0;
            for attempt in 0..2 {
                let (mut stream, _) = listener.accept().unwrap();
                stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                assert!(matches!(decode::<WrapperToDaemon>(line.trim()).unwrap(),
                    WrapperToDaemon::RegisterAuxiliary { agent_id, instance_id, .. }
                    if agent_id == identity && instance_id == "instance-241"));
                registrations += 1;
                if attempt == 0 { continue; } // ACK d'inscription perdu, aucune mutation lue.
                writeln!(stream, "{}", encode(&DaemonToWrapper::Registered {
                    agent_id: identity.into(), credential: None }).unwrap()).unwrap();
                line.clear();
                reader.read_line(&mut line).unwrap();
                assert!(matches!(decode::<WrapperToDaemon>(line.trim()).unwrap(),
                    WrapperToDaemon::NativeDelegation { request: bridget_transport::protocol::NativeDelegationRequest::Cancel {task_id} }
                    if task_id == "task-241"));
                writeln!(stream, "{}", encode(&DaemonToWrapper::NativeDelegationResult {
                    result: serde_json::json!({"status":"cancelled"}) }).unwrap()).unwrap();
            }
            registrations
        });
        let mut connection = registered_connection_until(identity, "instance-241", &path,
            Instant::now()+Duration::from_secs(2)).expect("le raccordement doit récupérer son ACK perdu");
        let response = connection.send_then_wait(&WrapperToDaemon::NativeDelegation {
            request: bridget_transport::protocol::NativeDelegationRequest::Cancel {task_id:"task-241".into()}
        }).unwrap();
        assert!(matches!(response,DaemonToWrapper::NativeDelegationResult {result} if result["status"]=="cancelled"));
        assert_eq!(server.join().unwrap(),2);
        let _=std::fs::remove_file(path);
    }

    #[test]
    fn spec241_registration_identity_refusal_is_not_retried() {
        let identity = "24100000-0000-4000-8000-000000000241";
        let path = crate::mcp_identity::mock_socket("241-register-refused");
        let listener=UnixListener::bind(&path).unwrap();
        crate::mcp_identity::mock_private_identity(&path,identity,"instance-241");
        let (done_tx,done_rx)=mpsc::channel();
        let server=thread::spawn(move || {
            let (mut stream,_)=listener.accept().unwrap();
            let mut line=String::new();
            BufReader::new(stream.try_clone().unwrap()).read_line(&mut line).unwrap();
            writeln!(stream,"{}",encode(&DaemonToWrapper::Nack {id:"register-auxiliary".into(),
                reason:"auxiliary_identity_unproven : primary_not_live".into()}).unwrap()).unwrap();
            done_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            listener.set_nonblocking(true).unwrap();
            assert!(matches!(listener.accept(),Err(e) if e.kind()==io::ErrorKind::WouldBlock));
        });
        let result=registered_connection_until(identity,"instance-241",&path,Instant::now()+Duration::from_secs(1));
        assert!(matches!(result,Err(ClientError::Technical {code:"auxiliary_identity_unproven",..})));
        done_tx.send(()).unwrap();server.join().unwrap();let _=std::fs::remove_file(path);
    }

    #[test]
    fn spec241_registration_never_renews_the_callers_deadline() {
        let identity="24100000-0000-4000-8000-000000000241";
        let path=crate::mcp_identity::mock_socket("241-register-deadline");
        let listener=UnixListener::bind(&path).unwrap();
        crate::mcp_identity::mock_private_identity(&path,identity,"instance-241");
        let (done_tx,done_rx)=mpsc::channel();
        let server=thread::spawn(move || {
            let (stream,_)=listener.accept().unwrap();
            let mut line=String::new();
            let mut reader=BufReader::new(stream);
            reader.read_line(&mut line).unwrap();
            // Garder le pair vivant mais sans ACK jusqu'à expiration réelle.
            done_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            drop(reader);
            listener.set_nonblocking(true).unwrap();
            assert!(matches!(listener.accept(),Err(e) if e.kind()==io::ErrorKind::WouldBlock));
        });
        let start=Instant::now();
        let response=registered_connection_until(identity,"instance-241",&path,start+Duration::from_millis(80));
        assert!(matches!(response,Err(ClientError::Technical{code:"daemon_unreachable",..})));
        assert!(start.elapsed()<Duration::from_millis(350));
        done_tx.send(()).unwrap();server.join().unwrap();let _=std::fs::remove_file(path);
    }

    #[test]
    fn native151_facade_sonde_attente_annonce_et_ancien_daemon() {
        for supports_wait in [true, false] {
            let path = crate::mcp_identity::mock_socket("wait-probe");
            let listener = UnixListener::bind(&path).unwrap();
            let server = thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                assert!(matches!(
                    decode::<WrapperToDaemon>(line.trim()).unwrap(),
                    WrapperToDaemon::DaemonIdentityRequest
                ));
                let mut report = serde_json::json!({"type":"DaemonIdentityReport","host":"fixture","db_path":"fixture","instance_id":"fixture"});
                if supports_wait {
                    report["features"] = serde_json::json!(["wait_v1"]);
                }
                writeln!(stream, "{report}").unwrap();
                line.clear();
                assert_eq!(
                    reader.read_line(&mut line).unwrap(),
                    0,
                    "aucune trame nouvelle après la sonde"
                );
            });
            let supported = daemon_supports_wait(&path);
            server.join().unwrap();
            std::fs::remove_file(path).unwrap();
            assert_eq!(supported.unwrap(), supports_wait);
        }
    }

    #[test]
    fn native151_facade_sonde_attente_garde_budget_client_borne() {
        let path = crate::mcp_identity::mock_socket("wait-probe-timeout");
        let listener = UnixListener::bind(&path).unwrap();
        let server = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(12)))
                .unwrap();
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            line.clear();
            assert_eq!(reader.read_line(&mut line).unwrap(), 0);
        });
        let start = Instant::now();
        let supported = daemon_supports_wait(&path);
        server.join().unwrap();
        std::fs::remove_file(path).unwrap();
        assert!(matches!(supported, Err(ClientError::Technical { code: "daemon_unreachable", .. })));
        assert!((Duration::from_secs(9)..Duration::from_secs(12)).contains(&start.elapsed()));
    }

    #[test]
    fn spec147_members_client_uses_one_combined_hello_and_refuses_old_peer() {
        for supports_members in [false, true] {
            let path = crate::mcp_identity::mock_socket("members-negotiation");
            let listener = UnixListener::bind(&path).unwrap();
            let actor = "14700000-0000-4000-8000-000000000001";
            crate::mcp_identity::mock_private_identity(&path, actor, "instance147members");
            let server = thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                assert!(matches!(
                    decode::<WrapperToDaemon>(line.trim()).unwrap(),
                    WrapperToDaemon::RegisterAuxiliary { .. }
                ));
                writeln!(
                    stream,
                    "{}",
                    encode(&DaemonToWrapper::Registered {
                        agent_id: actor.into(),
                        credential: None
                    })
                    .unwrap()
                )
                .unwrap();
                line.clear();
                reader.read_line(&mut line).unwrap();
                let WrapperToDaemon::ClientHello { capabilities, .. } =
                    decode(line.trim()).unwrap()
                else {
                    panic!("hello requis")
                };
                assert_eq!(
                    capabilities,
                    vec![
                        ClientCapability::ThreadMembersV1,
                        ClientCapability::CommunicationProjectsV1
                    ]
                );
                writeln!(
                    stream,
                    "{}",
                    encode(&DaemonToWrapper::ClientWelcome {
                        version: 1,
                        build_id: "fixture".into(),
                        horizon_secs: 60,
                        issued_at_tolerance_secs: 5,
                        capabilities: if supports_members {
                            capabilities
                        } else {
                            vec![ClientCapability::CommunicationProjectsV1]
                        }
                    })
                    .unwrap()
                )
                .unwrap();
                line.clear();
                let bytes = reader.read_line(&mut line).unwrap();
                if !supports_members {
                    assert_eq!(bytes, 0, "aucune requête ni repli après cap absente");
                    return;
                }
                assert!(
                    matches!(decode::<WrapperToDaemon>(line.trim()).unwrap(),WrapperToDaemon::ThreadRequest{request} if request.request.name()=="add_members")
                );
                writeln!(
                    stream,
                    "{}",
                    encode(&DaemonToWrapper::ThreadResult {
                        result: bridget_transport::protocol::ThreadResult {
                            version: 1,
                            result: serde_json::json!({"status":"no_change"})
                        }
                    })
                    .unwrap()
                )
                .unwrap();
            });
            let request=serde_json::from_value(serde_json::json!({"version":1,"cross_project_reason":"mandat explicite","request":{"action":"add_members","thread_id":actor,"members":[actor],"operation_id":"14700000-0000-4000-8000-000000000002"}})).unwrap();
            let result = thread_request(actor, "instance147members", &path, request);
            if supports_members {
                assert_eq!(result.unwrap().result["status"], "no_change");
            } else {
                assert!(matches!(
                    result,
                    Err(ClientError::Technical {
                        code: "thread_members_unsupported",
                        ..
                    })
                ));
            }
            server.join().unwrap();
            let _ = std::fs::remove_dir_all(path.parent().unwrap());
        }
    }

    #[test]
    fn spec147_old_daemon_capability_refusal_is_terminal_and_typed() {
        let path =
            std::env::temp_dir().join(format!("h147old-{}.sock", uuid::Uuid::new_v4().simple()));
        let listener = UnixListener::bind(&path).unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            reader.read_line(&mut String::new()).unwrap();
            writeln!(
                stream,
                "{}",
                encode(&DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client
                })
                .unwrap()
            )
            .unwrap();
            let mut hello = String::new();
            reader.read_line(&mut hello).unwrap();
            assert!(hello.contains("human_thread_watch_v1"));
            writeln!(
                stream,
                "{}",
                encode(&DaemonToWrapper::ClientRejected {
                    reason: bridget_transport::protocol::ClientRefusal::CapabilityNotNegotiated
                })
                .unwrap()
            )
            .unwrap();
            assert_eq!(
                reader.read_line(&mut String::new()).unwrap(),
                0,
                "aucun repli agent ni Register"
            );
        });
        let mut events = Vec::new();
        human_thread_watch(
            &path,
            bridget_transport::protocol::HumanThreadWatchV1 {
                version: 1,
                t3_thread_id: "14700000-0000-4000-8000-000000000001".into(),
                project_root: "/fixture".into(),
            },
            |event| {
                events.push(event.clone());
                Ok(())
            },
            || false,
        )
        .unwrap();
        server.join().unwrap();
        std::fs::remove_file(path).unwrap();
        assert_eq!(
            events,
            [bridget_transport::protocol::HumanThreadWatchEvent::error(
                bridget_transport::protocol::HumanThreadViewError::UnsupportedVersion
            )]
        );
    }

    #[test]
    fn spec147_transport_disconnect_is_not_a_permanent_version_refusal() {
        let path =
            std::env::temp_dir().join(format!("h147c-{}.sock", uuid::Uuid::new_v4().simple()));
        let listener = UnixListener::bind(&path).unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            reader.read_line(&mut String::new()).unwrap();
            writeln!(
                stream,
                "{}",
                encode(&DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client
                })
                .unwrap()
            )
            .unwrap();
            reader.read_line(&mut String::new()).unwrap();
        });
        let mut events = Vec::new();
        let result = human_thread_watch(
            &path,
            bridget_transport::protocol::HumanThreadWatchV1 {
                version: 1,
                t3_thread_id: "14700000-0000-4000-8000-000000000001".into(),
                project_root: "/fixture".into(),
            },
            |event| {
                events.push(event.clone());
                Ok(())
            },
            || false,
        );
        server.join().unwrap();
        std::fs::remove_file(path).unwrap();
        assert!(
            result.is_err(),
            "une coupure de transport doit rester reprenable"
        );
        assert!(
            events.is_empty(),
            "aucun unsupported_version métier inventé"
        );
    }
    use std::io::BufRead;
    use std::os::unix::net::UnixListener;
    use std::sync::mpsc;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    use std::thread;

    fn pair(budget: Duration) -> (DaemonConnection, UnixStream) {
        let (stream, peer) = UnixStream::pair().unwrap();
        (
            DaemonConnection {
                reader: BufReader::new(stream.try_clone().unwrap()),
                writer: stream,
                deadline: Instant::now() + budget,
                poisoned: false,
                max_response_bytes: MAX_DAEMON_RESPONSE_BYTES,
            },
            peer,
        )
    }

    #[test]
    fn updates179_activation_reply_is_deferred_without_replay() {
        for mutation in [false, true] {
            let (mut connection, mut peer) = pair(Duration::from_secs(2));
            writeln!(
                peer,
                "{}",
                encode(&DaemonToWrapper::UpdateInProgress).unwrap()
            )
            .unwrap();
            let command = WrapperToDaemon::DaemonIdentityRequest;
            let result = if mutation {
                connection.send_then_wait(&command)
            } else {
                connection.exchange(&command)
            };
            assert!(
                matches!(result, Err(ClientError::Technical {code, ..}) if code == "update_in_progress")
            );
            assert!(connection.poisoned);
            assert!(connection.exchange(&command).is_err());
            let mut received = String::new();
            std::io::Read::read_to_string(&mut BufReader::new(peer), &mut received).unwrap();
            assert_eq!(received.lines().count(), 1);
        }
    }

    #[test]
    fn spec174_disconnect_preserve_issue_et_ferme_connexion() {
        for mutation in [false, true] {
            let (mut connection, mut peer) = pair(Duration::from_secs(2));
            writeln!(peer, "{}", encode(&DaemonToWrapper::Disconnect).unwrap()).unwrap();
            let command = WrapperToDaemon::DaemonIdentityRequest;
            let result = if mutation {
                connection.send_then_wait(&command)
            } else {
                connection.exchange(&command)
            };
            let expected = if mutation { "outcome_unknown" } else { "daemon_unreachable" };
            assert!(matches!(result, Err(ClientError::Technical { code, message })
                if code == expected && message.contains("ne prouve ni son échec ni son annulation")
                    && message.contains("Conservez les identifiants")));
            assert!(connection.poisoned);
            assert!(connection.exchange(&command).is_err());
            let mut received = String::new();
            std::io::Read::read_to_string(&mut BufReader::new(peer), &mut received).unwrap();
            assert_eq!(received.lines().count(), 1, "aucun rejeu sur le lien fermé");
        }
    }

    #[test]
    fn spec174_wake_coupure_socket_reelle_sans_rejeu_ni_lecture_modifiee() {
        use bridget_transport::protocol::{WAKE_CONTRACT_VERSION, WakeAction, WakeRequest};

        let identity = "17400000-0000-4000-8000-000000000174";
        for disconnect in [true, false] {
            let path = crate::mcp_identity::mock_socket("174-wake-cut");
            let listener = UnixListener::bind(&path).unwrap();
            crate::mcp_identity::mock_private_identity(&path, identity, "instance-174");
            let server = thread::spawn(move || {
                let mut seen = Vec::new();
                for _ in 0..2 {
                    let (mut stream, _) = listener.accept().unwrap();
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    let WrapperToDaemon::RegisterAuxiliary { agent_id, .. } =
                        decode(line.trim()).unwrap()
                    else {
                        panic!("RegisterAuxiliary attendu");
                    };
                    writeln!(
                        stream,
                        "{}",
                        encode(&DaemonToWrapper::Registered {
                            credential: None,
                            agent_id
                        })
                        .unwrap()
                    )
                    .unwrap();
                    line.clear();
                    reader.read_line(&mut line).unwrap();
                    seen.push(line);
                    if disconnect {
                        writeln!(
                            stream,
                            "{}",
                            encode(&DaemonToWrapper::Disconnect).unwrap()
                        )
                        .unwrap();
                    }
                }
                seen
            });
            let cancel = WakeRequest {
                version: WAKE_CONTRACT_VERSION,
                request: WakeAction::Cancel {
                    schedule_id: "wake-174".into(),
                },
            };
            let cut = wake_request(Some((identity, "instance-174")), &path, cancel);
            assert!(
                matches!(&cut, Err(ClientError::Technical { code: "outcome_unknown", message })
                    if message.contains("Relisez la liste des réveils")
                        && (!disconnect || message.contains("ne prouve ni son échec ni son annulation"))),
                "{cut:?}"
            );
            let list = WakeRequest {
                version: WAKE_CONTRACT_VERSION,
                request: WakeAction::List {},
            };
            let read = wake_request(Some((identity, "instance-174")), &path, list);
            assert!(
                matches!(&read, Err(ClientError::Technical { code: "daemon_unreachable", message })
                    if !message.contains("Relisez")),
                "{read:?}"
            );
            let seen = server.join().unwrap();
            let _ = std::fs::remove_file(path);
            assert_eq!(seen.len(), 2, "aucun rejeu automatique de la mutation");
        }
    }

    fn registered_server(
        path: &Path,
        response: DaemonToWrapper,
    ) -> (thread::JoinHandle<()>, mpsc::Receiver<WrapperToDaemon>) {
        let listener = UnixListener::bind(path).unwrap();
        crate::mcp_identity::mock_private_identity(
            path,
            "89000000-0000-4000-8000-000000000194",
            "instance-094",
        );
        let (seen_tx, seen_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let WrapperToDaemon::RegisterAuxiliary { agent_id, .. } = decode(line.trim()).unwrap()
            else {
                panic!("Register attendu");
            };
            writeln!(
                stream,
                "{}",
                encode(&DaemonToWrapper::Registered {
                    credential: None,
                    agent_id
                })
                .unwrap()
            )
            .unwrap();
            stream.flush().unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap();
            seen_tx.send(decode(line.trim()).unwrap()).unwrap();
            writeln!(stream, "{}", encode(&response).unwrap()).unwrap();
            stream.flush().unwrap();
        });
        (server, seen_rx)
    }

    #[test]
    fn spec094_rename_confirme_uuid_et_nom_normalise_sans_accepter_une_substitution() {
        use bridget_transport::protocol::DisplayNameOutcome;

        let identity = "89000000-0000-4000-8000-000000000194";
        for (returned_name, accepted) in [("Agent Vingt Trois", true), ("Autre agent", false)] {
            let path = crate::mcp_identity::mock_socket("rename");
            let response = DaemonToWrapper::DisplayNameResult {
                outcome: DisplayNameOutcome::Applied {
                    agent_id: identity.to_string(),
                    display_name: returned_name.to_string(),
                    revision: 2,
                },
            };
            let (server, seen) = registered_server(&path, response);
            let result =
                rename_display_name(identity, "instance-094", &path, "Agent   Vingt Trois");
            assert_eq!(result.is_ok(), accepted, "réponse={returned_name}");
            assert!(matches!(
                seen.recv().unwrap(),
                WrapperToDaemon::DisplayNameSet { .. }
            ));
            server.join().unwrap();
            let _ = std::fs::remove_file(path);
        }
    }

    #[test]
    fn spec094_domaine_distingue_ack_memoire_et_echec_de_persistance() {
        let root =
            std::env::temp_dir().join(format!("b94-domain-{}", uuid::Uuid::new_v4().simple()));
        crate::environment::ensure_private_directory(&root).unwrap();
        let path = root.join("daemon.sock");
        let (server, seen) = registered_server(
            &path,
            DaemonToWrapper::Ack {
                id: "domain".to_string(),
                project_warnings: Vec::new(),
            },
        );
        let identity = "89000000-0000-4000-8000-000000000194";
        // Le répertoire des domaines doit rester accessible au préflight du
        // verrou. Seule la cible finale est rendue impropre à un remplacement,
        // afin de prouver l'échec durable APRES l'ACK mémoire.
        std::fs::create_dir_all(root.join("agent-domains").join(identity)).unwrap();
        let result = set_domain(
            identity,
            "instance-094",
            &path,
            Some("documentation".to_string()),
        );
        assert!(matches!(
            result,
            Err(ClientError::Technical {
                code: "domain_persistence_failed",
                ref message,
            }) if message.contains("appliqué en mémoire")
                && message.contains("persistance non confirmée")
        ));
        assert!(matches!(
            seen.recv_timeout(Duration::from_secs(2)).unwrap(),
            WrapperToDaemon::Domain {
                domain: Some(domain), ..
            } if domain == "documentation"
        ));
        server.join().unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec094_deux_domaines_concurrents_gardent_le_meme_ordre_en_memoire_et_sur_disque() {
        let _observer_guard = lock_domain_test_observers();
        let root = std::env::temp_dir().join(format!(
            "b94-domain-order-{}",
            uuid::Uuid::new_v4().simple()
        ));
        crate::environment::ensure_private_directory(&root).unwrap();
        let path = root.join("daemon.sock");
        let listener = UnixListener::bind(&path).unwrap();
        crate::mcp_identity::mock_private_identity(
            &path,
            "89000000-0000-4000-8000-000000000294",
            "instance-094",
        );
        let (seen_tx, seen_rx) = mpsc::channel();
        let memory_domain = Arc::new(std::sync::Mutex::new(None));
        let memory_domain_for_server = Arc::clone(&memory_domain);
        let server = thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let WrapperToDaemon::RegisterAuxiliary { agent_id, .. } =
                    decode(line.trim()).unwrap()
                else {
                    panic!("Register attendu");
                };
                writeln!(
                    stream,
                    "{}",
                    encode(&DaemonToWrapper::Registered {
                        credential: None,
                        agent_id
                    })
                    .unwrap()
                )
                .unwrap();
                stream.flush().unwrap();
                line.clear();
                reader.read_line(&mut line).unwrap();
                let WrapperToDaemon::Domain {
                    domain: Some(domain),
                    ..
                } = decode(line.trim()).unwrap()
                else {
                    panic!("Domain attendu");
                };
                *memory_domain_for_server.lock().unwrap() = Some(domain.clone());
                seen_tx.send(domain).unwrap();
                writeln!(
                    stream,
                    "{}",
                    encode(&DaemonToWrapper::Ack {
                        id: "domain".to_string(),
                        project_warnings: Vec::new(),
                    })
                    .unwrap()
                )
                .unwrap();
                stream.flush().unwrap();
            }
        });

        let identity = "89000000-0000-4000-8000-000000000294";
        let first_observed = Arc::new(AtomicBool::new(false));
        let first_observed_for_hook = Arc::clone(&first_observed);
        let (blocked_tx, blocked_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let (contended_tx, contended_rx) = mpsc::channel();
        let release_rx = Arc::new(std::sync::Mutex::new(release_rx));
        let release_rx_for_hook = Arc::clone(&release_rx);
        let identity_for_hook = identity.to_string();
        replace_domain_after_ack_observer(Some(Arc::new(move |observed_identity| {
            if observed_identity == identity_for_hook
                && !first_observed_for_hook.swap(true, Ordering::SeqCst)
            {
                blocked_tx.send(()).unwrap();
                release_rx_for_hook.lock().unwrap().recv().unwrap();
            }
        })));
        let contention_reported = Arc::new(AtomicBool::new(false));
        let contention_reported_for_hook = Arc::clone(&contention_reported);
        let identity_for_contention = identity.to_string();
        replace_domain_lock_contention_observer(Some(Arc::new(move |observed_identity| {
            if observed_identity == identity_for_contention
                && !contention_reported_for_hook.swap(true, Ordering::SeqCst)
            {
                contended_tx.send(()).unwrap();
            }
        })));

        let first_path = path.clone();
        let first = thread::spawn(move || {
            set_domain(
                identity,
                "instance-094",
                &first_path,
                Some("premier".to_string()),
            )
        });
        assert_eq!(seen_rx.recv().unwrap(), "premier");
        blocked_rx.recv().unwrap();
        let second_path = path.clone();
        let second = thread::spawn(move || {
            set_domain(
                identity,
                "instance-094",
                &second_path,
                Some("second".to_string()),
            )
        });
        contended_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("le second appel doit rencontrer le verrou réellement tenu");
        let second_reached_daemon_while_first_blocked = seen_rx.try_recv().is_ok();
        release_tx.send(()).unwrap();
        assert!(first.join().unwrap().is_ok());
        assert!(second.join().unwrap().is_ok());
        assert_eq!(
            seen_rx.recv_timeout(Duration::from_secs(2)).unwrap(),
            "second"
        );
        server.join().unwrap();
        replace_domain_after_ack_observer(None);
        replace_domain_lock_contention_observer(None);

        assert!(
            !second_reached_daemon_while_first_blocked,
            "le second domaine a dépassé le premier avant sa persistance"
        );
        assert_eq!(
            std::fs::read_to_string(root.join("agent-domains").join(identity)).unwrap(),
            "second"
        );
        assert_eq!(memory_domain.lock().unwrap().as_deref(), Some("second"));
        let _ = std::fs::remove_file(path);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec094_control_status_negocie_lookup_borne_a_l_instance_sans_mutation() {
        use bridget_transport::protocol::ControlStateRefusal;

        let path = crate::mcp_identity::mock_socket("control");
        let listener = UnixListener::bind(&path).unwrap();
        crate::mcp_identity::mock_private_identity(
            &path,
            "89000000-0000-4000-8000-000000000194",
            "instance-094",
        );
        let (hello_tx, hello_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let (mut registration, _) = listener.accept().unwrap();
            let mut registration_reader = BufReader::new(registration.try_clone().unwrap());
            let mut line = String::new();
            registration_reader.read_line(&mut line).unwrap();
            let WrapperToDaemon::RegisterAuxiliary { agent_id, .. } = decode(line.trim()).unwrap()
            else {
                panic!("Register attendu avant la lecture Client");
            };
            writeln!(
                registration,
                "{}",
                encode(&DaemonToWrapper::Registered {
                    credential: None,
                    agent_id
                })
                .unwrap()
            )
            .unwrap();
            registration.flush().unwrap();
            drop(registration_reader);
            drop(registration);

            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            line.clear();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode(line.trim()).unwrap(),
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Client
                }
            ));
            writeln!(
                stream,
                "{}",
                encode(&DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client,
                })
                .unwrap()
            )
            .unwrap();
            stream.flush().unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap();
            let hello = decode(line.trim()).unwrap();
            hello_tx.send(hello).unwrap();
            writeln!(
                stream,
                "{}",
                encode(&DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: "test-094".to_string(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: vec![ClientCapability::Lookup],
                })
                .unwrap()
            )
            .unwrap();
            stream.flush().unwrap();
            line.clear();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode(line.trim()).unwrap(),
                WrapperToDaemon::ControlStateRead { .. }
            ));
            writeln!(
                stream,
                "{}",
                encode(&DaemonToWrapper::ControlStateRejected {
                    reason: ControlStateRefusal::StoreUnavailable,
                })
                .unwrap()
            )
            .unwrap();
            stream.flush().unwrap();
        });
        assert!(
            read_control_status(
                "89000000-0000-4000-8000-000000000194",
                "instance-094",
                &path,
                0,
            )
            .is_err()
        );
        match hello_rx.recv().unwrap() {
            WrapperToDaemon::ClientHello {
                issuer_scope,
                capabilities,
                ..
            } => {
                assert_eq!(
                    issuer_scope,
                    crate::communication::issuer_scope("instance-094")
                );
                assert_eq!(capabilities, vec![ClientCapability::Lookup]);
                assert!(!capabilities.contains(&ClientCapability::ControlStateV1));
            }
            other => panic!("ClientHello attendu, reçu {other:?}"),
        }
        server.join().unwrap();
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn spec094_control_status_stoppe_si_l_attestation_auxiliaire_est_perimee() {
        let path = crate::mcp_identity::mock_socket("control-stale");
        let listener = UnixListener::bind(&path).unwrap();
        crate::mcp_identity::mock_private_identity(
            &path,
            "89000000-0000-4000-8000-000000000194",
            "instance-perimee",
        );
        let (check_tx, check_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode(line.trim()).unwrap(),
                WrapperToDaemon::RegisterAuxiliary {
                    instance_id, ..
                } if instance_id == "instance-perimee"
            ));
            writeln!(
                stream,
                "{}",
                encode(&DaemonToWrapper::Nack {
                    id: "register".to_string(),
                    reason: "instance remplacée".to_string(),
                })
                .unwrap()
            )
            .unwrap();
            stream.flush().unwrap();
            check_rx.recv().unwrap();
            listener.set_nonblocking(true).unwrap();
            assert!(
                matches!(
                    listener.accept(),
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock
                ),
                "une connexion Client a contourné l'attestation refusée"
            );
        });
        assert!(
            read_control_status(
                "89000000-0000-4000-8000-000000000194",
                "instance-perimee",
                &path,
                0,
            )
            .is_err()
        );
        check_tx.send(()).unwrap();
        server.join().unwrap();
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn une_reponse_sans_lf_ne_devient_pas_un_accuse_valide() {
        let (mut connection, mut peer) = pair(Duration::from_secs(2));
        peer.write_all(b"{\"type\":\"Ack\",\"id\":\"ok\"}").unwrap();
        peer.shutdown(std::net::Shutdown::Write).unwrap();
        assert!(
            matches!(
                connection.read_response("outcome_unknown"),
                Err(ClientError::Technical {
                    code: "outcome_unknown",
                    ..
                })
            ),
            "EOF partiel accepté"
        );
    }

    #[test]
    fn le_goutte_a_goutte_ne_renouvelle_pas_le_budget_global() {
        let (mut connection, mut peer) = pair(Duration::from_millis(150));
        let writer = thread::spawn(move || {
            for byte in b"{\"type\":\"Ack\",\"id\":\"ok\"}\n" {
                if peer.write_all(&[*byte]).is_err() {
                    break;
                }
                thread::sleep(Duration::from_millis(25));
            }
        });
        let started = Instant::now();
        let result = connection.read_response("outcome_unknown");
        let elapsed = started.elapsed();
        drop(connection);
        writer.join().unwrap();
        assert!(
            matches!(
                result,
                Err(ClientError::Technical {
                    code: "outcome_unknown",
                    ..
                })
            ),
            "réponse après échéance acceptée"
        );
        assert!(
            elapsed < Duration::from_millis(450),
            "budget renouvelé : {elapsed:?}"
        );
    }

    #[test]
    fn la_borne_de_reponse_compte_le_lf_et_ne_restitue_pas_le_contenu() {
        const LIMIT: usize = 16 * 1024 * 1024;
        for excess in [0, 1] {
            let (mut connection, mut peer) = pair(Duration::from_secs(5));
            let writer = thread::spawn(move || {
                let suffix = b"{\"type\":\"Ack\",\"id\":\"ok\"}\n";
                let mut bytes = vec![b' '; LIMIT + excess - suffix.len()];
                bytes.extend_from_slice(suffix);
                let _ = peer.write_all(&bytes);
            });
            let result = connection.read_response("outcome_unknown");
            drop(connection);
            writer.join().unwrap();
            assert_eq!(
                result.is_ok(),
                excess == 0,
                "borne LF incluse, dépassement={excess}, réponse={result:?}"
            );
        }
    }

    #[test]
    fn erreur_de_couture_ne_recopie_pas_un_secret_dans_son_diagnostic() {
        let result = unexpected_response::<()>(DaemonToWrapper::Nack {
            id: "id".into(),
            reason: "CANARI_SECRET_089".into(),
        });
        assert!(!format!("{result:?}").contains("CANARI_SECRET_089"));
    }

    #[test]
    fn erreur_empoisonne_la_connexion_meme_si_une_reponse_valide_suit() {
        let (mut connection, mut peer) = pair(Duration::from_secs(2));
        peer.write_all(b"invalide\n{\"type\":\"Ack\",\"id\":\"tardif\"}\n")
            .unwrap();
        assert!(connection.read_response("outcome_unknown").is_err());
        // Le second message est déjà dans le buffer : sans la garde poisoned,
        // cette lecture le prendrait pour l'accusé d'une nouvelle opération.
        assert!(!connection.reader.buffer().is_empty());
        assert!(connection.read_response("outcome_unknown").is_err());
        assert!(connection.exchange(&WrapperToDaemon::ListAgents).is_err());
    }

    #[test]
    fn trois_phases_lentes_consument_une_seule_echeance() {
        use std::io::BufRead;
        let (mut connection, mut peer) = pair(Duration::from_millis(1000));
        let peer_reader = peer.try_clone().unwrap();
        let (seen_tx, seen_rx) = std::sync::mpsc::channel();
        let writer = thread::spawn(move || {
            peer_reader
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut reader = BufReader::new(peer_reader);
            for _ in 0..3 {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 {
                    break;
                }
                seen_tx.send(()).unwrap();
                thread::sleep(Duration::from_millis(400)); // lenteur du pair, pas synchronisation du test
                if peer
                    .write_all(b"{\"type\":\"Ack\",\"id\":\"phase\"}\n")
                    .is_err()
                {
                    break;
                }
            }
        });
        let started = Instant::now();
        let results = (0..3)
            .map(|_| connection.exchange(&WrapperToDaemon::ListAgents))
            .collect::<Vec<_>>();
        let elapsed = started.elapsed();
        drop(connection);
        writer.join().unwrap();
        assert_eq!(
            seen_rx.try_iter().count(),
            3,
            "les trois phases doivent être réellement tentées"
        );
        assert!(results[0].is_ok() && results[1].is_ok(), "{results:?}");
        assert!(
            results[2].is_err(),
            "renouveler le budget à chaque échange accepterait les trois réponses"
        );
        assert!(elapsed < Duration::from_millis(2000), "{elapsed:?}");
    }

    #[test]
    fn backlog_reel_plein_est_refuse_sans_attendre_un_accept() {
        use std::os::unix::net::UnixListener;
        let path = std::path::PathBuf::from("/tmp")
            .join(format!("b89-connect-{}", uuid::Uuid::new_v4().simple()));
        let listener = UnixListener::bind(&path).unwrap();
        // SAFETY: listen ne déréférence aucun pointeur ; ses arguments sont des entiers.
        assert_eq!(unsafe { libc::listen(listener.as_raw_fd(), 1) }, 0);
        // Une connexion dans la file, personne n'appelle accept pendant la sonde.
        let mut fillers = Vec::new();
        let mut refusal = None;
        for _ in 0..64 {
            let started = Instant::now();
            match connect_nonblocking(&path, started + Duration::from_millis(150)) {
                Ok(stream) => fillers.push(stream),
                Err(error) => {
                    refusal = Some((error, started.elapsed()));
                    break;
                }
            }
        }
        assert!(!fillers.is_empty());
        let (error, elapsed) = refusal.expect("borne réelle de la file atteinte sans accept");
        // Observé macOS : ECONNREFUSED sur file AF_UNIX pleine ; Linux peut
        // rendre EAGAIN. Ne pas prétendre avoir traversé EINPROGRESS/poll
        // lorsque le noyau a refusé immédiatement la connexion.
        assert!(
            matches!(
                error.kind(),
                io::ErrorKind::TimedOut
                    | io::ErrorKind::WouldBlock
                    | io::ErrorKind::ConnectionRefused
            ),
            "{error}"
        );
        assert!(
            elapsed < Duration::from_millis(500),
            "connect a ignoré la deadline : {elapsed:?}"
        );
        assert_eq!(
            // SAFETY: fcntl avec une commande à argument entier ne déréférence aucun pointeur.
            unsafe { libc::fcntl(fillers[0].as_raw_fd(), libc::F_GETFD) } & libc::FD_CLOEXEC,
            libc::FD_CLOEXEC
        );
        drop(fillers);
        drop(listener);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn budget_expire_et_chemin_nul_refuses_avant_toute_connexion() {
        use std::os::unix::ffi::OsStringExt;
        use std::os::unix::net::UnixListener;
        let path = std::path::PathBuf::from("/tmp")
            .join(format!("b89-no-connect-{}", uuid::Uuid::new_v4().simple()));
        let listener = UnixListener::bind(&path).unwrap();
        listener.set_nonblocking(true).unwrap();
        assert_eq!(
            connect_nonblocking(&path, Instant::now() - Duration::from_millis(1))
                .unwrap_err()
                .kind(),
            io::ErrorKind::TimedOut
        );
        let mut nul = path.as_os_str().as_bytes().to_vec();
        nul.extend_from_slice(b"\0suffixe");
        assert_eq!(
            connect_nonblocking(
                Path::new(&std::ffi::OsString::from_vec(nul)),
                Instant::now() + Duration::from_secs(1)
            )
            .unwrap_err()
            .kind(),
            io::ErrorKind::InvalidInput
        );
        assert!(
            matches!(listener.accept(), Err(error) if error.kind() == io::ErrorKind::WouldBlock)
        );
        drop(listener);
        std::fs::remove_file(path).unwrap();
    }

    /// Session 104 : un daemon antérieur ne connaît pas `ledger_search`. Le
    /// client rend une erreur technique explicite (Nack → `daemon_protocol`,
    /// coupure → `outcome_unknown`) et ne se replie jamais sur la projection
    /// globale ni sur un fichier local.
    #[test]
    fn spec104_ancien_daemon_erreur_honnete_sans_repli() {
        let root =
            std::env::temp_dir().join(format!("b104-old-daemon-{}", uuid::Uuid::new_v4().simple()));
        crate::environment::ensure_private_directory(&root).unwrap();
        let path = root.join("daemon.sock");
        let listener = UnixListener::bind(&path).unwrap();
        crate::mcp_identity::mock_private_identity(
            &path,
            "10400000-0000-4000-8000-000000000104",
            "instance-104",
        );
        let server = thread::spawn(move || {
            for turn in 0..2 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let WrapperToDaemon::RegisterAuxiliary { agent_id, .. } =
                    decode(line.trim()).unwrap()
                else {
                    panic!("RegisterAuxiliary attendu");
                };
                writeln!(
                    stream,
                    "{}",
                    encode(&DaemonToWrapper::Registered {
                        credential: None,
                        agent_id
                    })
                    .unwrap()
                )
                .unwrap();
                stream.flush().unwrap();
                line.clear();
                reader.read_line(&mut line).unwrap();
                assert!(line.contains("\"ledger_search\""), "{line}");
                if turn == 0 {
                    // Daemon ancien qui répond par un refus générique.
                    writeln!(
                        stream,
                        "{}",
                        encode(&DaemonToWrapper::Nack {
                            id: "ledger".into(),
                            reason: "commande inconnue".into()
                        })
                        .unwrap()
                    )
                    .unwrap();
                    stream.flush().unwrap();
                } // turn 1 : coupure sèche, aucune réponse.
            }
        });
        let request = bridget_transport::protocol::LedgerSearchRequest {
            source: bridget_transport::protocol::LedgerSearchSource::Messages,
            query: "x".into(),
            author: None,
            peer: None,
            since: None,
            until: None,
            limit: None,
            cursor: None,
            thread_id: None,
        };
        let nack = ledger_search(
            "10400000-0000-4000-8000-000000000104",
            "instance-104",
            &path,
            request.clone(),
        );
        assert!(
            matches!(
                &nack,
                Err(ClientError::Technical {
                    code: "daemon_protocol",
                    ..
                })
            ),
            "{nack:?}"
        );
        let cut = ledger_search(
            "10400000-0000-4000-8000-000000000104",
            "instance-104",
            &path,
            request,
        );
        assert!(
            matches!(&cut, Err(ClientError::Technical { .. })),
            "coupure = erreur technique, jamais une page : {cut:?}"
        );
        server.join().unwrap();
        std::fs::remove_file(path).unwrap();
    }
