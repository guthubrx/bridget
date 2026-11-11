use super::*;

#[test]
fn spec105_relais_gere_respecte_contrat_sur_tous_les_pilotes() {
    use bridget_transport::ManagedEventSource;
    for source in [
        ManagedEventSource::Acp,
        ManagedEventSource::ClaudeStreamJson,
        ManagedEventSource::CodexAppServer,
    ] {
        for automatic in [false, true] {
            for requested in [false, true] {
                for body in ["OK", ".", "Résultat utile"] {
                    let root = std::env::temp_dir()
                        .join(format!("bridget-spec105-{}", uuid::Uuid::new_v4()));
                    std::fs::create_dir_all(&root).unwrap();
                    let (stream, peer) = UnixStream::pair().unwrap();
                    let writer = Arc::new(Mutex::new(Some(BufWriter::new(stream))));
                    let mut tracker =
                        IdempotentDeliveryTracker::open_at(&root, "instance").unwrap();
                    let mut message = bridget_core::BridgetMessage::new("alice", "bob", "question");
                    message.reply = requested;
                    message.in_reply_to = Some("parent".into());
                    let request_id = message.id.clone();
                    let event = ManagedEvent::internal(
                        source,
                        vec![],
                        ManagedEventKind::TurnFinished {
                            message,
                            response: body.into(),
                            terminal: ManagedTerminal::Completed,
                        },
                    );
                    forward_managed_events_with_redaction(
                        &writer,
                        "bob",
                        vec![event],
                        &mut tracker,
                        &mut HashMap::new(),
                        &mut None,
                        None,
                        automatic,
                        None,
                        None,
                    );
                    drop(writer);
                    let frames: Vec<WrapperToDaemon> = BufReader::new(peer)
                        .lines()
                        .map(|line| decode(&line.unwrap()).unwrap())
                        .collect();
                    let replies: Vec<_> = frames
                        .iter()
                        .filter_map(|frame| match frame {
                            WrapperToDaemon::Send(message) => Some(message),
                            _ => None,
                        })
                        .collect();
                    assert_eq!(
                        replies.len(),
                        usize::from(automatic && requested),
                        "{source:?}/{automatic}/{requested}/{body}"
                    );
                    if let Some(reply) = replies.first() {
                        assert!(!reply.reply);
                        assert_eq!(reply.to, "alice");
                        assert_eq!(reply.body, body);
                        assert_eq!(reply.in_reply_to.as_deref(), Some(request_id.as_str()));
                    }
                }
            }
        }
    }
}

#[test]
fn spec_068_notification_deleguee_est_systeme_ordonnee_et_accusee_apres_injection() {
    let event = DelegatedRuntimeEventFrame {
        cursor: 7,
        event_id: "event-7".to_string(),
        link_id: "link-7".to_string(),
        child_instance_id: "enfant-7".to_string(),
        child_execution_id: "execution-7".to_string(),
        kind: DelegatedRuntimeEventKind::Warning,
        code: "unsupported_provider_request".to_string(),
        reference: format!("sha256:{}", "a".repeat(64)),
        observed_at: 1_700_000_000,
        project: None,
    };
    let message = delegated_runtime_message(&event, "parent-7");
    assert_eq!(message.from, "bridget");
    assert_eq!(message.to, "parent-7");
    assert_eq!(message.origin, Some(bridget_core::MessageOrigin::System));
    assert_eq!(message.intent, Some(bridget_core::MessageIntent::QueueOnly));
    assert!(!message.reply);
    assert!(!message.body.contains("params"));

    let root = std::env::temp_dir().join(format!("bridget-spec-068-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let mut tracker = IdempotentDeliveryTracker::open_at(&root, "parent-7").unwrap();
    tracker.record_delegated_runtime(&message.id, event.event_id.clone());
    assert!(matches!(
        tracker.prompt_dispatched(&message.id, 1_700_000_001),
        Some(WrapperToDaemon::DelegatedRuntimeEventAcknowledged { event_id }) if event_id == "event-7"
    ));
    assert!(
        tracker
            .prompt_dispatched(&message.id, 1_700_000_002)
            .is_none()
    );
    std::fs::remove_dir_all(root).unwrap();
}

/// Session 114 : un lot n'injecte qu'une fois, mais chaque remise garde son
/// accusé — sinon le daemon croirait la moitié du lot encore en vol. Un
/// rejeu déjà accusé n'entre pas dans le lot et n'est donc pas réinjecté.
#[test]
fn spec114_lot_injecte_une_fois_et_accuse_chaque_remise() {
    let root = std::env::temp_dir().join(format!("bridget-spec-114-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let mut tracker = IdempotentDeliveryTracker::open_at(&root, "equipier").unwrap();
    let lot = |ids: [&str; 2]| {
        ids.map(|id| {
            let mut message =
                bridget_core::BridgetMessage::new("coordinateur", "equipier", "compte rendu");
            message.id = id.to_string();
            BatchedDelivery::Idempotent {
                delivery_id: format!("d-{id}"),
                recipient_instance_id: "equipier".to_string(),
                delivery_generation: 7,
                expires_at: 500,
                message,
            }
        })
        .into_iter()
        .collect::<Vec<_>>()
    };

    let mut injections = Vec::new();
    let reports = deliver_batch_to_interactive(&mut tracker, lot(["a", "b"]), 100, |messages| {
        injections.push(messages.iter().map(|m| m.id.clone()).collect::<Vec<_>>());
        Ok(())
    });
    assert_eq!(injections, vec![vec!["a".to_string(), "b".to_string()]]);
    let accuses: Vec<&str> = reports
        .iter()
        .filter_map(|report| match report {
            WrapperToDaemon::DeliverAcked { delivery_id, .. } => Some(delivery_id.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(accuses, vec!["d-a", "d-b"], "chaque remise a son accusé");

    // Rejeu du lot : rien n'est réinjecté, les accusés repartent seuls.
    let replay = deliver_batch_to_interactive(&mut tracker, lot(["a", "b"]), 101, |_| {
        injections.push(vec!["rejeu".to_string()]);
        Ok(())
    });
    assert_eq!(injections.len(), 1, "aucune seconde injection");
    assert_eq!(replay.len(), 2, "les deux accusés sont rejoués");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec_068_diagnostic_codex_est_remonte_sans_octets_fournisseur() {
    let (writer_stream, reader_stream) = UnixStream::pair().unwrap();
    reader_stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    let writer = Arc::new(Mutex::new(Some(BufWriter::new(writer_stream))));
    let root =
        std::env::temp_dir().join(format!("bridget-spec-068-forward-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let mut tracker = IdempotentDeliveryTracker::open_at(&root, "child-1").unwrap();
    let mut bindings = HashMap::new();
    bindings.insert(
        "message-1".to_string(),
        ManagedExecutionBinding {
            execution_id: "execution-1".to_string(),
            generation: 1,
            revision: 0,
            state: "running".to_string(),
            provider_kind: "codex".to_string(),
            execution_path: "codex_app_server".to_string(),
            approval_requests: 0,
            last_approval_request: None,
            remote_result: None,
        },
    );
    let reference = format!("sha256:{}", "b".repeat(64));
    assert!(!forward_managed_events(
        &writer,
        "child-1",
        vec![ManagedEvent::internal(
            bridget_transport::ManagedEventSource::CodexAppServer,
            b"params=secret-ne-pas-remonter".to_vec(),
            ManagedEventKind::Diagnostic {
                code: "unsupported_provider_request".to_string(),
                reference: reference.clone(),
            },
        )],
        &mut tracker,
        &mut bindings
    ));
    let mut line = String::new();
    BufReader::new(reader_stream).read_line(&mut line).unwrap();
    assert!(matches!(
        decode(line.trim()).unwrap(),
        WrapperToDaemon::DelegatedRuntimeEvent { execution_id, kind: DelegatedRuntimeEventKind::Warning, code, reference: observed }
            if execution_id == "execution-1" && code == "unsupported_provider_request" && observed == reference
    ));
    assert!(!line.contains("secret-ne-pas-remonter"));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec_068_terminal_enfant_est_failed_distinct_et_redacted() {
    let (writer_stream, reader_stream) = UnixStream::pair().unwrap();
    reader_stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .unwrap();
    let writer = Arc::new(Mutex::new(Some(BufWriter::new(writer_stream))));
    let root =
        std::env::temp_dir().join(format!("bridget-spec-068-failed-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    let mut tracker = IdempotentDeliveryTracker::open_at(&root, "child-2").unwrap();
    let mut bindings = HashMap::new();
    bindings.insert(
        "message-2".to_string(),
        ManagedExecutionBinding {
            execution_id: "execution-2".to_string(),
            generation: 1,
            revision: 0,
            state: "running".to_string(),
            provider_kind: "codex".to_string(),
            execution_path: "codex_app_server".to_string(),
            approval_requests: 0,
            last_approval_request: None,
            remote_result: None,
        },
    );
    let mut message = bridget_core::BridgetMessage::new("humain", "child-2", "travaille");
    message.id = "message-2".to_string();
    assert!(!forward_managed_events(
        &writer,
        "child-2",
        vec![ManagedEvent::internal(
            bridget_transport::ManagedEventSource::CodexAppServer,
            b"detail=secret-ne-pas-remonter".to_vec(),
            ManagedEventKind::TurnFinished {
                message,
                response: String::new(),
                terminal: ManagedTerminal::Failed {
                    detail: "secret-ne-pas-remonter".to_string()
                },
            },
        )],
        &mut tracker,
        &mut bindings
    ));
    let mut reader = BufReader::new(reader_stream);
    let mut failures = Vec::new();
    for _ in 0..4 {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        if let WrapperToDaemon::DelegatedRuntimeEvent {
            kind,
            code,
            reference,
            ..
        } = decode(line.trim()).unwrap()
        {
            failures.push((kind, code, reference));
            assert!(!line.contains("secret-ne-pas-remonter"));
        }
    }
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].0, DelegatedRuntimeEventKind::Failed);
    assert_eq!(failures[0].1, "provider_failed");
    assert!(failures[0].2.starts_with("sha256:"));
    assert!(!bindings.contains_key("message-2"));
    std::fs::remove_dir_all(root).unwrap();
}
