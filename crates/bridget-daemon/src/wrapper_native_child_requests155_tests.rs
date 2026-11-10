    use super::*;
    use bridget_transport::protocol::{
        ChildProviderRequest, ChildProviderRequestId, ChildRequestAnswer, ChildRequestClosedState,
        ChildRequestDecision, ChildRequestKind,
    };
    use bridget_transport::{Transport, TransportError};
    use serde_json::json;

    #[derive(Clone, Default)]
    struct Session155 {
        writes: Arc<Mutex<Vec<ChildRequestAnswer>>>,
        entered: Option<std::sync::mpsc::SyncSender<()>>,
        release: Option<Arc<std::sync::Barrier>>,
        busy: Option<bool>,
    }
    impl Transport for Session155 {
        fn deliver(&mut self, _: &bridget_core::BridgetMessage) -> Result<(), TransportError> {
            Ok(())
        }
        fn is_alive(&self) -> bool {
            true
        }
        fn connection_id(&self) -> &str {
            "wrapper155"
        }
    }
    impl ManagedSession for Session155 {
        fn answer_provider_request(
            &mut self,
            answer: &ChildRequestAnswer,
        ) -> Result<(), TransportError> {
            self.writes.lock().unwrap().push(answer.clone());
            if let Some(entered) = &self.entered {
                entered.send(()).unwrap();
            }
            if let Some(release) = &self.release {
                release.wait();
            }
            Ok(())
        }
        fn descriptor(&self) -> ManagedSessionDescriptor {
            ManagedSessionDescriptor {
                transport: "codex_app_server".into(),
                mode: bridget_transport::protocol::PresenceMode::Acp,
                location: None,
            }
        }
        fn process_id(&self) -> u32 {
            0
        }
        fn activate_journal(
            &self,
            _: &Path,
            _: &str,
            _: Option<JournalLiveFeed>,
        ) -> std::io::Result<()> {
            Ok(())
        }
        fn drain_events(&self) -> Vec<ManagedEvent> {
            Vec::new()
        }
        fn cancel_delivery(&self, _: &str, _: &str) -> bool {
            false
        }
        fn stop(&self) {}
        fn is_busy(&self) -> bool {
            self.busy.unwrap_or(true)
        }
    }

    #[test]
    fn spec172_remote_reconnect_deadline_preserves_provider_and_stops_retrying() {
        let session = Session155::default();
        let writer = Arc::new(Mutex::new(None));
        let started = Instant::now();
        let absent = std::env::temp_dir().join(format!("reconnect172-absent-{}", uuid::Uuid::new_v4()));
        let result = reconnect_managed_session(
            &absent, &writer, &session, &session.descriptor(), "codex_app_server", "fixture",
            &absent, "fixture", None, "test", "instance172", "agent172", false, None,
            Some(started + Duration::from_millis(60)),
        );
        assert!(result.is_none());
        assert!(started.elapsed() < Duration::from_secs(2));
        assert!(writer.lock().unwrap().is_none());
        assert!(!absent.exists());
    }

    fn request155() -> ChildProviderRequest {
        ChildProviderRequest {
            version: 1,
            provider_session: "session155".into(),
            turn_id: "turn155".into(),
            provider_request_id: ChildProviderRequestId::Number(155),
            kind: ChildRequestKind::UserInput,
            details: json!({"questions":[{"id":"q","question":"Choisir","isSecret":false}]}),
            offered: vec![
                ChildRequestDecision::Accept,
                ChildRequestDecision::Decline,
                ChildRequestDecision::Cancel,
            ],
            question_ids: vec!["q".into()],
            truncated: false,
        }
    }
    fn answer155() -> ChildRequestAnswer {
        let request = request155();
        ChildRequestAnswer {
            request_id: "request155".into(),
            provider_session: request.provider_session,
            turn_id: request.turn_id,
            provider_request_id: request.provider_request_id,
            decision: ChildRequestDecision::Accept,
            answer: Some(json!({"q":["oui"]})),
            outcome: "answered_by_parent".into(),
        }
    }
    fn cancel155() -> DaemonToWrapper {
        let request = request155();
        DaemonToWrapper::ChildRequestCancel {
            request_id: "request155".into(),
            provider_session: request.provider_session,
            turn_id: request.turn_id,
            provider_request_id: request.provider_request_id,
        }
    }
    fn fixture155() -> (
        Arc<Mutex<Option<BufWriter<UnixStream>>>>,
        BufReader<UnixStream>,
        Arc<Mutex<ManagedChildRequests>>,
        Session155,
        HashMap<String, ManagedExecutionBinding>,
    ) {
        let (stream, peer) = UnixStream::pair().unwrap();
        peer.set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        let writer = Arc::new(Mutex::new(Some(BufWriter::new(stream))));
        let requests = Arc::new(Mutex::new(ManagedChildRequests {
            waiting: vec![ManagedChildWaiting {
                request: request155(),
                request_id: Some("request155".into()),
                message_id: Some("message155".into()),
                uncertain: false,
            }],
            settled: Default::default(),
        }));
        let mut bindings = HashMap::new();
        bindings.insert(
            "message155".into(),
            ManagedExecutionBinding {
                execution_id: "execution155".into(),
                provider_kind: "codex".into(),
                execution_path: "codex_app_server".into(),
                generation: 1,
                revision: 1,
                state: "waiting_user_input".into(),
                approval_requests: 0,
                last_approval_request: None,
                remote_result: None,
            },
        );
        (
            writer,
            BufReader::new(peer),
            requests,
            Session155::default(),
            bindings,
        )
    }
    fn frames155(reader: &mut BufReader<UnixStream>) -> Vec<WrapperToDaemon> {
        let mut frames = Vec::new();
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => frames.push(decode(line.trim()).unwrap()),
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    break;
                }
                Err(error) => panic!("lecture fixture155 : {error}"),
            }
        }
        frames
    }

    #[test]
    fn native160_wrapper_refus_injection_idempotente_liaison_optionnelle() {
        for context in [
            None,
            Some(ExecutionDeliveryContext {
                execution_id: "execution160".into(),
                generation: 2,
                revision: 0,
            }),
        ] {
            let (writer, mut reader, _requests, _session, mut bindings) = fixture155();
            bindings.clear();
            let bound = bind_idempotent_delivery_execution(
                &mut bindings,
                "message160",
                context,
                "codex",
                "codex_app_server",
            );
            send_wrapper_message(
                &writer,
                bound_delivery_rejection(
                    &bindings,
                    "message160".into(),
                    "injection refusée".into(),
                ),
            );
            let frames = frames155(&mut reader);
            assert_eq!(
                frames.len(),
                1,
                "aucune transition failed synchrone idempotente"
            );
            assert!(matches!(&frames[0], WrapperToDaemon::DeliveryRejected {
                execution_id, generation, ..
            } if execution_id.as_deref() == bound.then_some("execution160")
                && *generation == bound.then_some(2)));
            assert_eq!(
                bindings.get("message160").map(|b| b.state.as_str()),
                bound.then_some("starting")
            );
        }
    }

    #[test]
    fn native160_wrapper_refus_deliver_execution_capture_avant_failed() {
        let (writer, mut reader, _requests, _session, mut bindings) = fixture155();
        bindings.get_mut("message155").unwrap().state = "starting".into();
        let rejection =
            bound_delivery_rejection(&bindings, "message155".into(), "injection refusée".into());
        publish_execution_transition(
            &writer,
            &mut bindings,
            "message155",
            "failed",
            "provider_unavailable",
        );
        send_wrapper_message(&writer, rejection);
        assert!(!bindings.contains_key("message155"));
        let frames = frames155(&mut reader);
        assert!(
            matches!(&frames[0], WrapperToDaemon::ExecutionStateChanged { transition }
            if transition.next_state == "failed")
        );
        assert!(matches!(&frames[1], WrapperToDaemon::DeliveryRejected {
            execution_id: Some(id), generation: Some(1), ..
        } if id == "execution155"));
    }

    #[test]
    fn native160_wrapper_refus_forward_capture_avant_terminal() {
        for (surface, expected_state, expected_reason) in [
            ("async", "failed", "limite"),
            ("empty", "completed", "réponse vide"),
            ("pause", "interrupted", "tour annulé par le pilote"),
        ] {
            let (writer, mut reader, _requests, _session, mut bindings) = fixture155();
            let root =
                std::env::temp_dir().join(format!("native160-{surface}-{}", uuid::Uuid::new_v4()));
            crate::environment::ensure_private_directory(&root).unwrap();
            let mut tracker = IdempotentDeliveryTracker::open_at(&root, "native160").unwrap();
            let mut message = bridget_core::BridgetMessage::new("parent", "agent", "mission");
            message.id = "message155".into();
            message.reply = true;
            let kind = match surface {
                "async" => ManagedEventKind::DeliveryRejected {
                    message_id: message.id,
                    reason: "limite".into(),
                },
                "empty" => ManagedEventKind::TurnFinished {
                    message,
                    response: String::new(),
                    terminal: ManagedTerminal::Completed,
                },
                "pause" => {
                    publish_execution_transition(
                        &writer,
                        &mut bindings,
                        "message155",
                        "interrupting",
                        "interrupt_requested",
                    );
                    ManagedEventKind::TurnFinished {
                        message,
                        response: String::new(),
                        terminal: ManagedTerminal::Cancelled,
                    }
                }
                _ => unreachable!("trois chemins prescrits"),
            };
            forward_managed_events(
                &writer,
                "agent",
                vec![ManagedEvent::internal(
                    bridget_transport::ManagedEventSource::Acp,
                    Vec::new(),
                    kind,
                )],
                &mut tracker,
                &mut bindings,
            );
            assert!(!bindings.contains_key("message155"));
            let frames = frames155(&mut reader);
            assert!(frames.iter().any(
                |f| matches!(f, WrapperToDaemon::ExecutionStateChanged { transition }
                if transition.next_state == expected_state)
            ));
            assert!(
                frames
                    .iter()
                    .any(|f| matches!(f, WrapperToDaemon::DeliveryRejected {
                reason, execution_id: Some(execution_id), generation: Some(1), ..
            } if reason == expected_reason && execution_id == "execution155"))
            );
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn held159_turnstate_reattestation_periodique() {
        let (writer, mut reader, _, mut session, _) = fixture155();
        session.busy = Some(true);
        let start = Instant::now();
        let mut last = start;
        // Boucle chargée : 600 étapes de 50 ms, contrôle de la vraie minuterie à chaque passage.
        // Seule la minuterie émet ses trames ; pas de trafic artificiel qui sature le pair.
        for step in 0..=600 {
            let now = start + Duration::from_millis(step * 50);
            reattest_managed_turn_if_due(&writer, &session, &mut last, now);
            assert!(now.saturating_duration_since(last) <= Duration::from_secs(30));
        }
        let states: Vec<_> = frames155(&mut reader)
            .into_iter()
            .filter_map(|frame| match frame {
                WrapperToDaemon::TurnState { in_progress } => Some(in_progress),
                _ => None,
            })
            .collect();
        assert!(!states.is_empty(), "réattestation même sous trafic continu");
        assert!(states.iter().all(|busy| *busy), "état réel de la session");
        assert!(last > start);
    }

    #[test]
    fn held159_turnstate_reattestation_d_un_repos_perdu() {
        let (writer, mut reader, _, mut session, _) = fixture155();
        session.busy = Some(false);
        let start = Instant::now();
        let mut last = start;
        // La fin précédente n'a pas été remise : aucun faux événement terminal injecté.
        reattest_managed_turn_if_due(
            &writer,
            &session,
            &mut last,
            start + Duration::from_secs(30),
        );
        let states: Vec<_> = frames155(&mut reader)
            .into_iter()
            .filter_map(|frame| match frame {
                WrapperToDaemon::TurnState { in_progress } => Some(in_progress),
                _ => None,
            })
            .collect();
        assert_eq!(
            states,
            vec![false],
            "le repos réel est réattesté dans les30s"
        );
    }

    #[test]
    fn held159_turnstate_refus_de_guidage_en_plein_tour_n_envoie_pas_false() {
        // Réemploi fixture155/frames155 : vrais octets socket, aucun pilote nouveau.
        let (writer, mut reader, _requests, mut session, mut bindings) = fixture155();
        session.busy = Some(true);
        let root = std::env::temp_dir().join(format!("turn159-{}", uuid::Uuid::new_v4()));
        crate::environment::ensure_private_directory(&root).unwrap();
        let mut tracker = IdempotentDeliveryTracker::open_at(&root, "turn159").unwrap();
        let events = vec![
            ManagedEvent::internal(
                bridget_transport::ManagedEventSource::CodexAppServer,
                Vec::new(),
                ManagedEventKind::ActivityObserved { in_progress: true },
            ),
            ManagedEvent::internal(
                bridget_transport::ManagedEventSource::CodexAppServer,
                Vec::new(),
                ManagedEventKind::DeliveryRejected {
                    message_id: "message155".into(),
                    reason: "guidage refusé".into(),
                },
            ),
        ];
        assert!(!forward_managed_events_with_redaction(
            &writer,
            "child155",
            events,
            &mut tracker,
            &mut bindings,
            &mut None,
            None,
            true,
            None,
            Some(session.is_busy())
        ));
        let states: Vec<bool> = frames155(&mut reader)
            .into_iter()
            .filter_map(|frame| match frame {
                WrapperToDaemon::TurnState { in_progress } => Some(in_progress),
                _ => None,
            })
            .collect();
        assert_eq!(
            states.first(),
            Some(&true),
            "état observé du fournisseur transmis"
        );
        assert_eq!(
            states.last(),
            Some(&true),
            "un refus de guidage ne prouve pas la fin du tour"
        );
        drop(tracker);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn held159_turnstate_refus_sans_tour_en_cours_envoie_false() {
        // Réemploi fixture155/frames155 : vrais octets socket, aucun pilote nouveau.
        let (writer, mut reader, _requests, mut session, mut bindings) = fixture155();
        session.busy = Some(false);
        let root = std::env::temp_dir().join(format!("turn159-{}", uuid::Uuid::new_v4()));
        crate::environment::ensure_private_directory(&root).unwrap();
        let mut tracker = IdempotentDeliveryTracker::open_at(&root, "turn159").unwrap();
        let events = vec![
            ManagedEvent::internal(
                bridget_transport::ManagedEventSource::CodexAppServer,
                Vec::new(),
                ManagedEventKind::ActivityObserved { in_progress: false },
            ),
            ManagedEvent::internal(
                bridget_transport::ManagedEventSource::CodexAppServer,
                Vec::new(),
                ManagedEventKind::DeliveryRejected {
                    message_id: "message155".into(),
                    reason: "guidage refusé".into(),
                },
            ),
        ];
        assert!(!forward_managed_events_with_redaction(
            &writer,
            "child155",
            events,
            &mut tracker,
            &mut bindings,
            &mut None,
            None,
            true,
            None,
            Some(session.is_busy())
        ));
        let states: Vec<bool> = frames155(&mut reader)
            .into_iter()
            .filter_map(|frame| match frame {
                WrapperToDaemon::TurnState { in_progress } => Some(in_progress),
                _ => None,
            })
            .collect();
        assert_eq!(
            states.first(),
            Some(&false),
            "état observé du fournisseur transmis"
        );
        assert_eq!(
            states.last(),
            Some(&false),
            "un refus de guidage ne prouve pas la fin du tour"
        );
        drop(tracker);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn native155_wrapper_demande_puis_reprise() {
        let (writer, mut reader, requests, mut session, mut bindings) = fixture155();
        requests.lock().unwrap().waiting.clear();
        bindings.get_mut("message155").unwrap().state = "running".into();
        let root =
            std::env::temp_dir().join(format!("bridget-wrapper155-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let mut tracker = IdempotentDeliveryTracker::open_at(&root, "instance155").unwrap();
        forward_managed_events_with_redaction(
            &writer,
            "child155",
            vec![ManagedEvent::internal(
                bridget_transport::ManagedEventSource::CodexAppServer,
                vec![],
                ManagedEventKind::ProviderRequest {
                    request: request155(),
                },
            )],
            &mut tracker,
            &mut bindings,
            &mut None,
            None,
            true,
            Some(&requests),
            None,
        );
        let frames = frames155(&mut reader);
        drop(tracker);
        std::fs::remove_dir_all(root).unwrap();
        assert!(frames.iter().any(|frame| matches!(frame,WrapperToDaemon::ChildRequest { request } if request == &request155())),"la demande doit remonter : {frames:?}");
        assert_eq!(bindings["message155"].state, "waiting_user_input");
        assert_eq!(bindings["message155"].approval_requests, 0);
        assert!(!dispatch_managed_child_request(
            &writer,
            &mut session,
            &mut bindings,
            &requests,
            DaemonToWrapper::ChildRequestAnswer {
                answer: answer155()
            }
        ));
        assert_eq!(session.writes.lock().unwrap().as_slice(), &[answer155()]);
        assert_eq!(bindings["message155"].state, "running");
        assert!(frames155(&mut reader).iter().any(|frame| matches!(frame,WrapperToDaemon::ChildRequestAnswered { request_id,applied:true,.. } if request_id=="request155")));
    }

    #[test]
    fn native155_wrapper_reponse_ancien_tour_refusee() {
        let (writer, mut reader, requests, mut session, mut bindings) = fixture155();
        for mismatch in ["turn", "session", "provider_id", "request_id"] {
            let mut answer = answer155();
            match mismatch {
                "turn" => answer.turn_id = "old-turn".into(),
                "session" => answer.provider_session = "old-session".into(),
                "provider_id" => {
                    answer.provider_request_id = ChildProviderRequestId::String("155".into())
                }
                _ => answer.request_id = "old-request".into(),
            }
            dispatch_managed_child_request(
                &writer,
                &mut session,
                &mut bindings,
                &requests,
                DaemonToWrapper::ChildRequestAnswer { answer },
            );
            assert!(
                frames155(&mut reader).iter().any(|frame| matches!(
                    frame,
                    WrapperToDaemon::ChildRequestAnswered { applied: false, .. }
                )),
                "{mismatch} doit être refusé"
            );
        }
        assert!(session.writes.lock().unwrap().is_empty());
        assert_eq!(bindings["message155"].state, "waiting_user_input");
    }

    #[test]
    fn native155_wrapper_fermeture_puis_reponse_jamais_ecrite() {
        let (writer, mut reader, requests, mut session, mut bindings) = fixture155();
        dispatch_managed_child_request(
            &writer,
            &mut session,
            &mut bindings,
            &requests,
            cancel155(),
        );
        let closed = frames155(&mut reader);
        assert!(closed.iter().any(|frame| matches!(frame,WrapperToDaemon::ChildRequestClosed { request_id,state:ChildRequestClosedState::ClosedBeforeApply } if request_id=="request155")),"fermeture après refus fournisseur : {closed:?}");
        let writes = session.writes.lock().unwrap().clone();
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].decision, ChildRequestDecision::Decline);
        assert_eq!(writes[0].answer, None);
        dispatch_managed_child_request(
            &writer,
            &mut session,
            &mut bindings,
            &requests,
            DaemonToWrapper::ChildRequestAnswer {
                answer: answer155(),
            },
        );
        assert_eq!(
            *session.writes.lock().unwrap(),
            writes,
            "aucun accept après fermeture"
        );
        assert!(frames155(&mut reader).iter().any(|frame| matches!(
            frame,
            WrapperToDaemon::ChildRequestAnswered { applied: false, .. }
        )));
    }

    #[test]
    fn native155_wrapper_reponse_puis_fermeture_already_applied() {
        let (writer, mut reader, requests, mut session, bindings) = fixture155();
        let (entered_tx, entered_rx) = std::sync::mpsc::sync_channel(1);
        let release = Arc::new(std::sync::Barrier::new(2));
        session.entered = Some(entered_tx);
        session.release = Some(release.clone());
        let writes = session.writes.clone();
        let writer_answer = writer.clone();
        let requests_answer = requests.clone();
        let mut answer_bindings = bindings.clone();
        let answer_worker = thread::spawn(move || {
            dispatch_managed_child_request(
                &writer_answer,
                &mut session,
                &mut answer_bindings,
                &requests_answer,
                DaemonToWrapper::ChildRequestAnswer {
                    answer: answer155(),
                },
            )
        });
        if entered_rx.recv_timeout(Duration::from_secs(1)).is_err() {
            answer_worker.join().unwrap();
            panic!("la réponse doit atteindre la session fournisseur avant la fermeture");
        }
        assert!(
            requests.try_lock().is_err(),
            "le même verrou doit couvrir l'écriture fournisseur"
        );
        let writer_cancel = writer.clone();
        let requests_cancel = requests.clone();
        let mut cancel_bindings = bindings;
        let mut cancel_session = Session155 {
            writes: writes.clone(),
            ..Default::default()
        };
        let (started_tx, started_rx) = std::sync::mpsc::sync_channel(1);
        let cancel_worker = thread::spawn(move || {
            started_tx.send(()).unwrap();
            dispatch_managed_child_request(
                &writer_cancel,
                &mut cancel_session,
                &mut cancel_bindings,
                &requests_cancel,
                cancel155(),
            )
        });
        started_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        release.wait();
        answer_worker.join().unwrap();
        cancel_worker.join().unwrap();
        assert_eq!(writes.lock().unwrap().as_slice(), &[answer155()]);
        let frames = frames155(&mut reader);
        assert!(frames.iter().any(|frame| matches!(
            frame,
            WrapperToDaemon::ChildRequestAnswered { applied: true, .. }
        )));
        assert!(
            frames.iter().any(|frame| matches!(
                frame,
                WrapperToDaemon::ChildRequestClosed {
                    state: ChildRequestClosedState::AlreadyApplied,
                    ..
                }
            )),
            "{frames:?}"
        );
    }

    #[test]
    fn native155_wrapper_renvoi_apres_reglement() {
        for answered in [false, true] {
            let (writer, mut reader, requests, mut session, mut bindings) = fixture155();
            let first = if answered {
                DaemonToWrapper::ChildRequestAnswer {
                    answer: answer155(),
                }
            } else {
                cancel155()
            };
            dispatch_managed_child_request(&writer, &mut session, &mut bindings, &requests, first);
            let original: Vec<_> = frames155(&mut reader)
                .into_iter()
                .filter(|frame| {
                    matches!(
                        frame,
                        WrapperToDaemon::ChildRequestAnswered { .. }
                            | WrapperToDaemon::ChildRequestClosed { .. }
                    )
                })
                .collect();
            assert!(!original.is_empty(), "un règlement doit être accusé");
            let writes = session.writes.lock().unwrap().clone();
            let replay = if answered {
                DaemonToWrapper::ChildRequestAnswer {
                    answer: answer155(),
                }
            } else {
                cancel155()
            };
            dispatch_managed_child_request(&writer, &mut session, &mut bindings, &requests, replay);
            let replay: Vec<_> = frames155(&mut reader)
                .into_iter()
                .filter(|frame| {
                    matches!(
                        frame,
                        WrapperToDaemon::ChildRequestAnswered { .. }
                            | WrapperToDaemon::ChildRequestClosed { .. }
                    )
                })
                .collect();
            assert_eq!(
                serde_json::to_value(&original).unwrap(),
                serde_json::to_value(&replay).unwrap()
            );
            assert_eq!(
                *session.writes.lock().unwrap(),
                writes,
                "le renvoi ne réécrit jamais au fournisseur"
            );
            assert_eq!(requests.lock().unwrap().settled.len(), 1);
            assert!(requests.lock().unwrap().waiting.is_empty());
        }
    }
