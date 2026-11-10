    use super::*;
    use crate::remote_worker::{WorkerResultReservation, WorkerStore};
    use bridget_transport::protocol::{
        RemotePlacement, WorkerCommandIdentity, WorkerIdentity, WorkerResultIdentity,
        WorkerResultSlot,
    };

    struct Fixture {
        root: PathBuf,
        context: WorkerResultContext,
        reservation: WorkerResultReservation,
    }
    impl Fixture {
        fn new() -> Self {
            let root =
                std::env::temp_dir().join(format!("bridget-wrapper162-{}", uuid::Uuid::new_v4()));
            crate::environment::ensure_private_directory(&root).unwrap();
            let context = WorkerResultContext {
                worker: WorkerIdentity {
                    machine_id: "cartae".into(),
                    worker_id: "worker-cartae".into(),
                    provisioning_revision: 1,
                },
                command: WorkerCommandIdentity {
                    command_id: "command162".into(),
                    task_id: "task162".into(),
                    instance_id: "instance162".into(),
                    generation: 1,
                },
                store_root: root.join("worker"),
            };
            let store = WorkerStore::open(&context.store_root, context.worker.clone()).unwrap();
            let placement = RemotePlacement {
                machine_id: "cartae".into(),
                worker_id: "worker-cartae".into(),
                provisioning_revision: 1,
                project_id: "project162".into(),
                mapping_revision: 1,
                origin_host: "mac".into(),
                origin_cwd: "/project".into(),
                source_root_identity: "repo".into(),
                relative_cwd: ".".into(),
                execution_cwd: "/project".into(),
                base_commit: None,
                definition_digest: "a".repeat(64),
                permission_digest: "b".repeat(64),
            };
            store
                .reserve_command(&context.command, &"c".repeat(64), &placement, 1000, 100)
                .unwrap();
            let slot = WorkerResultSlot {
                identity: WorkerResultIdentity {
                    command: context.command.clone(),
                    mission_id: "mission162".into(),
                    round: 0,
                    execution_id: "execution162".into(),
                    execution_generation: 1,
                },
                admission_digest: "d".repeat(64),
            };
            let reservation = store.reserve_result_slot(&slot, 101).unwrap();
            Self {
                root,
                context,
                reservation,
            }
        }
        fn message(&self) -> bridget_core::BridgetMessage {
            let mut message = bridget_core::BridgetMessage::new("parent", "child", "mission");
            message.id = "mission162".into();
            message.reply = true;
            message.intent=Some(bridget_core::MessageIntent::TriggerTurn);
            message
        }
        fn binding(&self) -> ManagedExecutionBinding {
            ManagedExecutionBinding {
                execution_id: "execution162".into(),
                provider_kind: "codex".into(),
                execution_path: "codex_app_server".into(),
                generation: 1,
                state: "running".into(),
                revision: 1,
                approval_requests: 0,
                last_approval_request: None,
                remote_result: Some(self.reservation.clone()),
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn remote162_wrapper_slot_absent_refuse_avant_injection() {
        let fixture = Fixture::new();
        let mut message = fixture.message();
        assert!(
            remote_result_for_delivery(
                Some(&fixture.context),
                &message,
                Some(("execution162", 1)),
                &HashMap::new()
            )
            .unwrap()
            .is_some()
        );
        message.id = "mission-non-reservee".into();
        assert!(
            remote_result_for_delivery(
                Some(&fixture.context),
                &message,
                Some(("execution162", 1)),
                &HashMap::new()
            )
            .is_err()
        );
        assert!(
            remote_result_for_delivery(
                Some(&fixture.context),
                &fixture.message(),
                None,
                &HashMap::new()
            )
            .is_err()
        );
        message.origin = Some(bridget_core::MessageOrigin::System);
        assert!(
            remote_result_for_delivery(Some(&fixture.context), &message, None, &HashMap::new())
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn spec244_reprise_reponse_wrapper_sans_nouveau_tour_fournisseur() {
        let fixture=Fixture::new();let state=fixture.root.join("receipts");
        let mut tracker=IdempotentDeliveryTracker::open_at(&state,"instance162").unwrap();
        tracker.requires_binding=true;tracker.modern_reply_core=true;
        let mut direct=fixture.message();direct.id="direct244-durable".into();direct.intent=None;tracker.admit_ordinary(&direct);
        let (stream,peer)=UnixStream::pair().unwrap();let writer=Arc::new(Mutex::new(Some(BufWriter::new(stream))));
        let event=ManagedEvent::internal(bridget_transport::ManagedEventSource::CodexAppServer,vec![],
            ManagedEventKind::TurnFinished{message:direct,response:"DIRECT244OK".into(),terminal:ManagedTerminal::Completed});
        let mut bindings=HashMap::new();assert!(!forward_managed_events(&writer,"child",vec![event],&mut tracker,&mut bindings));
        let stored=tracker.reply_outbox.as_ref().unwrap().pending(unix_now_secs()).unwrap().unwrap();drop(tracker);drop(writer);drop(peer);
        let mut tracker=IdempotentDeliveryTracker::open_at(&state,"instance162").unwrap();
        tracker.modern_reply_core=true;tracker.last_reply_retry=Instant::now()-Duration::from_secs(6);
        let (stream,peer)=UnixStream::pair().unwrap();let writer=Arc::new(Mutex::new(Some(BufWriter::new(stream))));
        tracker.retry_reply(&writer).unwrap();drop(writer);
        let frames=BufReader::new(peer).lines().map(|line|decode(&line.unwrap()).unwrap()).collect::<Vec<_>>();
        assert!(matches!(frames.as_slice(),[WrapperToDaemon::Send(m)] if m.id==stored.id && m.body==stored.body));
        tracker.acknowledge_reply(&stored.id).unwrap();
        assert!(tracker.reply_outbox.as_ref().unwrap().pending(unix_now_secs()).unwrap().is_none());
    }

    #[test]
    fn spec244_terminal_distant_sans_binding_ne_devient_pas_une_reponse_ordinaire() {
        let fixture=Fixture::new();let (stream,peer)=UnixStream::pair().unwrap();
        let writer=Arc::new(Mutex::new(Some(BufWriter::new(stream))));
        let mut tracker=IdempotentDeliveryTracker::open_at(&fixture.root.join("receipts"),"instance162").unwrap();
        tracker.requires_binding=true;tracker.modern_reply_core=true;
        let mut bindings=HashMap::new();
        let event=ManagedEvent::internal(bridget_transport::ManagedEventSource::CodexAppServer,vec![],
            ManagedEventKind::TurnFinished{message:fixture.message(),response:"ancien résultat".into(),terminal:ManagedTerminal::Completed});
        assert!(!forward_managed_events(&writer,"child",vec![event],&mut tracker,&mut bindings));
        assert!(tracker.reply_outbox.is_none());drop(writer);
        assert!(BufReader::new(peer).lines().next().is_none());
        assert!(WorkerStore::open(&fixture.context.store_root,fixture.context.worker.clone()).unwrap()
            .read_result(&fixture.reservation.slot.identity).unwrap().is_none());
    }

    #[test]
    fn spec244_message_direct_distant_ne_demande_pas_le_slot_de_la_mission() {
        let fixture=Fixture::new();
        let mut direct=fixture.message();
        direct.id="direct244".into();direct.intent=None;
        let mut bindings=HashMap::from([("mission162".into(),fixture.binding())]);
        assert!(remote_result_for_delivery(Some(&fixture.context),&direct,None,&bindings).unwrap().is_none());
        let (stream,peer)=UnixStream::pair().unwrap();
        let writer=Arc::new(Mutex::new(Some(BufWriter::new(stream))));
        let mut tracker=IdempotentDeliveryTracker::open_at(&fixture.root.join("receipts"),"instance162").unwrap();
        tracker.admit_ordinary(&direct);
        let event=ManagedEvent::internal(bridget_transport::ManagedEventSource::CodexAppServer,vec![],
            ManagedEventKind::TurnFinished{message:direct,response:"DIRECT244OK".into(),terminal:ManagedTerminal::Completed});
        assert!(!forward_managed_events(&writer,"child",vec![event],&mut tracker,&mut bindings));
        assert_eq!(bindings["mission162"].state,"running");
        assert!(!tracker.ordinary_active("direct244"));
        drop(writer);
        let frames=BufReader::new(peer).lines().map(|line|decode(&line.unwrap()).unwrap()).collect::<Vec<_>>();
        assert_eq!(frames.iter().filter(|f|matches!(f,WrapperToDaemon::Send(m) if m.in_reply_to.as_deref()==Some("direct244") && m.body=="DIRECT244OK")).count(),1);
        assert!(WorkerStore::open(&fixture.context.store_root,fixture.context.worker.clone()).unwrap()
            .read_result(&fixture.reservation.slot.identity).unwrap().is_none(),"la réponse ordinaire ne devient pas le résultat de la tâche");
    }

    #[test]
    fn remote162_wrapper_blob_sans_send_apres_arret_worker() {
        let fixture = Fixture::new(); // WorkerStore déjà fermé, wrapper reste producteur.
        let (stream, peer) = UnixStream::pair().unwrap();
        let writer = Arc::new(Mutex::new(Some(BufWriter::new(stream))));
        let mut tracker =
            IdempotentDeliveryTracker::open_at(&fixture.root.join("receipts"), "instance162")
                .unwrap();
        let mut bindings = HashMap::from([("mission162".into(), fixture.binding())]);
        let event = ManagedEvent::internal(
            bridget_transport::ManagedEventSource::CodexAppServer,
            vec![],
            ManagedEventKind::TurnFinished {
                message: fixture.message(),
                response: "résultat durable".into(),
                terminal: ManagedTerminal::Completed,
            },
        );
        assert!(!forward_managed_events(
            &writer,
            "child",
            vec![event],
            &mut tracker,
            &mut bindings
        ));
        let repeated = ManagedEvent::internal(
            bridget_transport::ManagedEventSource::CodexAppServer,
            vec![],
            ManagedEventKind::TurnFinished {
                message: fixture.message(),
                response: "résultat durable".into(),
                terminal: ManagedTerminal::Completed,
            },
        );
        assert!(!forward_managed_events(
            &writer,
            "child",
            vec![repeated],
            &mut tracker,
            &mut bindings
        ));
        drop(writer);
        let frames: Vec<WrapperToDaemon> = BufReader::new(peer)
            .lines()
            .map(|line| decode(&line.unwrap()).unwrap())
            .collect();
        assert!(!frames.iter().any(|frame| matches!(
            frame,
            WrapperToDaemon::Send(_) | WrapperToDaemon::SendIdempotent { .. }
        )));
        let store =
            WorkerStore::open(&fixture.context.store_root, fixture.context.worker.clone()).unwrap();
        let result = store
            .read_result(&fixture.reservation.slot.identity)
            .unwrap()
            .unwrap();
        assert_eq!(result.body, "résultat durable");
        assert_eq!(result.identity, fixture.reservation.slot.identity);
        result.validate().unwrap();
    }

    #[test]
    fn remote162_wrapper_rejeu_stable_et_divergence_sans_remplacement() {
        let fixture = Fixture::new();
        let binding = fixture.binding();
        assert!(publish_remote_final(Some(&binding), &fixture.message(), "un résultat").unwrap());
        let first = std::fs::read(&fixture.reservation.blob_path).unwrap();
        assert!(publish_remote_final(Some(&binding), &fixture.message(), "un résultat").unwrap());
        assert_eq!(
            first,
            std::fs::read(&fixture.reservation.blob_path).unwrap()
        );
        assert!(publish_remote_final(Some(&binding), &fixture.message(), "divergence").is_err());
        assert_eq!(
            first,
            std::fs::read(&fixture.reservation.blob_path).unwrap()
        );
        assert!(!publish_remote_final(None, &fixture.message(), "local").unwrap());
    }

    #[test]
    fn remote162_wrapper_contexte_et_generation_etrangers_refuses() {
        let fixture = Fixture::new();
        assert!(
            fixture
                .context
                .validate_for_instance("autre-instance")
                .is_err()
        );
        let mut forged = fixture.context.clone();
        forged.command.instance_id = "../escape".into();
        assert!(forged.validate_for_instance("../escape").is_err());
        let mut binding = fixture.binding();
        binding.generation = 2;
        assert!(publish_remote_final(Some(&binding), &fixture.message(), "résultat").is_err());
        let bindings = HashMap::from([("mission162".into(), binding)]);
        assert!(
            remote_result_for_delivery(
                Some(&fixture.context),
                &fixture.message(),
                Some(("execution162", 1)),
                &bindings
            )
            .is_err()
        );
    }
