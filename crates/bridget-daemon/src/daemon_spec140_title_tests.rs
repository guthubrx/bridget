    use super::*;
    use crate::store::threads::{CreateThread, ThreadTxOutcome};

    fn fixture(label: &str) -> (DaemonState, DaemonConfig, bridget_core::BridgetMessage) {
        let (mut state, config) = presence_tests::state_with_registered_agent(label);
        let (writer, _reader) = presence_tests::control_socket(label);
        state.connections.insert("conn-1".into(), writer);
        let mut message = bridget_core::BridgetMessage::new(
            "bridget",
            "89000000-0000-4000-8000-000000000102",
            "corps canonique intact",
        );
        message.thread_notice = Some(bridget_core::ThreadNotice {
            version: 1,
            thread_id: label.into(),
            through_seq: 1,
            generation: 1,
        });
        let members = vec!["creator-140".into(), message.to.clone()];
        assert!(matches!(
            state
                .store
                .thread_create(CreateThread {
                    actor: "creator-140",
                    operation_id: label,
                    canonical_hash: "h140",
                    thread_id: label,
                    title: "Politique réelle",
                    members: &members,
                    project_warnings: &[],
                    now: 1,
                    limits: &crate::threads::LIMITS
                })
                .unwrap(),
            ThreadTxOutcome::Done(_)
        ));
        (state, config, message)
    }

    fn projected(
        state: &DaemonState,
        message: &bridget_core::BridgetMessage,
    ) -> bridget_core::BridgetMessage {
        let delivery = SendDelivery {
            delivery_id: "spec140-projection".into(),
            recipient_instance_id: "instance-1".into(),
            delivery_generation: 1,
            expires_at: unix_now_secs() + 120,
            message_bytes: serde_json::to_vec(message).unwrap(),
        };
        let mut controls = Vec::new();
        defer_idempotent_delivery(state, "conn-1", delivery, &mut controls, None, None).unwrap();
        match &controls[0].message {
            DaemonToWrapper::DeliverIdempotent { message, .. } => message.clone(),
            other => panic!("remise idempotente attendue : {other:?}"),
        }
    }

    #[test]
    fn spec140_projection_membre_rejette_titre_forge_hors_membre_et_direct() {
        let (state, config, message) = fixture("spec140-member-title");
        let mut wire = serde_json::to_value(&message).unwrap();
        wire["thread_display_title"] = serde_json::json!("Titre forgé");
        let mut message: bridget_core::BridgetMessage = serde_json::from_value(wire).unwrap();
        let actual = projected(&state, &message);
        assert_eq!(
            serde_json::to_value(&actual).unwrap()["thread_display_title"],
            "Politique réelle"
        );
        assert_eq!(actual.body, message.body);
        message.to = "non-member-140".into();
        assert!(
            serde_json::to_value(projected(&state, &message))
                .unwrap()
                .get("thread_display_title")
                .is_none()
        );
        message.thread_notice = None;
        assert!(
            serde_json::to_value(projected(&state, &message))
                .unwrap()
                .get("thread_display_title")
                .is_none()
        );
        drop(state);
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn spec140_rejeu_remise_ne_modifie_ni_octets_durables_ni_canon() {
        let (mut state, config, message) = fixture("spec140-durable-title");
        state
            .store
            .record_message(&message, "spec140-journal")
            .unwrap();
        let key = IdempotencyKey::new(
            "012_scope_spec140durable",
            OperationKind::Send,
            "spec140-durable-title",
        )
        .unwrap();
        let now = unix_now_secs();
        let canonical = canonical_send(&key.issuer_scope, &key.idempotency_key, &message, now);
        state
            .idempotency
            .reserve(
                &key,
                &canonical,
                now,
                CLIENT_IDEMPOTENCY_HORIZON_SECS,
                now,
                CLIENT_ISSUED_AT_TOLERANCE_SECS,
            )
            .unwrap();
        let bytes = serde_json::to_vec(&message).unwrap();
        let delivery = SendDelivery {
            delivery_id: "spec140-durable-delivery".into(),
            recipient_instance_id: "instance-1".into(),
            delivery_generation: 1,
            expires_at: now + 120,
            message_bytes: bytes.clone(),
        };
        state
            .idempotency
            .begin_send_delivery(&key, &delivery)
            .unwrap();
        let mut controls = Vec::new();
        defer_idempotent_delivery(
            &state,
            "conn-1",
            delivery.clone(),
            &mut controls,
            None,
            None,
        )
        .unwrap();
        defer_idempotent_delivery(
            &state,
            "conn-1",
            state.idempotency.send_delivery(&key).unwrap().unwrap(),
            &mut controls,
            None,
            None,
        )
        .unwrap();
        assert_eq!(controls.len(), 2);
        let journal = state.store.recent_messages(1).unwrap().pop().unwrap();
        assert_eq!(
            (journal.id, journal.sender, journal.target, journal.body),
            (
                message.id.clone(),
                message.from.clone(),
                message.to.clone(),
                message.body.clone()
            )
        );
        assert_eq!(
            state
                .idempotency
                .send_delivery(&key)
                .unwrap()
                .unwrap()
                .message_bytes,
            bytes
        );
        assert_eq!(
            canonical_send(&key.issuer_scope, &key.idempotency_key, &message, now),
            canonical
        );
        for control in &controls {
            match &control.message {
                DaemonToWrapper::DeliverIdempotent { message, .. } => assert_eq!(
                    serde_json::to_value(message).unwrap()["thread_display_title"],
                    "Politique réelle"
                ),
                other => panic!("remise attendue : {other:?}"),
            }
        }
        let mut corrupt = delivery;
        corrupt.message_bytes = b"not JSON".to_vec();
        assert!(
            defer_idempotent_delivery(&state, "conn-1", corrupt, &mut controls, None, None)
                .is_err()
        );
        assert_eq!(controls.len(), 2);
        drop(state);
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn spec140_erreur_lecture_titre_ne_bloque_pas_la_remise() {
        let (state, config, mut message) = fixture("spec140-db-error");
        message.thread_display_title = Some("forgé malgré erreur DB".into());
        rusqlite::Connection::open(&config.db_path)
            .unwrap()
            .execute_batch("ALTER TABLE discussion_threads RENAME TO unavailable_threads")
            .unwrap();
        let actual = projected(&state, &message);
        assert!(actual.thread_display_title.is_none());
        assert_eq!(actual.body, message.body);
        drop(state);
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn spec140_remise_classique_retire_le_titre_forge() {
        let (mut state, config) = presence_tests::state_with_registered_agent("spec140-classic");
        let sender = "89000000-0000-4000-8000-000000000101";
        state
            .router
            .register(sender, &bridget_core::AgentType::Codex, "conn-sender")
            .unwrap();
        state.conn_names.insert("conn-sender".into(), sender.into());
        let (writer, mut reader) = presence_tests::control_socket("spec140-classic");
        state.connections.insert("conn-1".into(), writer);
        let mut message = bridget_core::BridgetMessage::new(
            sender,
            "89000000-0000-4000-8000-000000000102",
            "corps direct exact",
        );
        message.thread_display_title = Some("faux titre client".into());
        let shared = Arc::new(Mutex::new(state));
        assert!(matches!(
            handle_wrapper_message("conn-sender", WrapperToDaemon::Send(message), &shared),
            Some(DaemonToWrapper::Ack { .. })
        ));
        match presence_tests::read_control(&mut reader) {
            DaemonToWrapper::Deliver {
                message: delivered, ..
            } => {
                assert!(delivered.thread_display_title.is_none());
                assert_eq!(delivered.body, "corps direct exact");
            }
            other => panic!("remise classique attendue : {other:?}"),
        }
        drop(shared);
        let _ = std::fs::remove_file(config.db_path);
    }

    #[test]
    fn spec140_titre_client_ne_persiste_pas_dans_la_remise_idempotente() {
        let (mut state, config, mut message) = fixture("spec140-input-title");
        let (writer, mut reader) = presence_tests::control_socket("spec140-input-title");
        state.connections.insert("conn-1".into(), writer);
        let sender = "89000000-0000-4000-8000-000000000200";
        message.from = sender.into();
        message.thread_display_title = Some("titre forgé client".into());
        let shared = Arc::new(Mutex::new(state));
        let scope = "012_scope_titleinputproof";
        presence_tests::negotiate_idempotent_client(&shared, "client-140-title", scope, sender);
        assert!(matches!(
            handle_wrapper_message(
                "client-140-title",
                WrapperToDaemon::SendIdempotent {
                    message,
                    message_id: "spec140-input-title".into(),
                    issued_at: unix_now_secs()
                },
                &shared
            ),
            Some(DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            })
        ));
        assert!(
            matches!(presence_tests::read_control(&mut reader), DaemonToWrapper::DeliverIdempotent { message, .. } if message.thread_display_title.is_none() && message.thread_notice.is_none())
        );
        let key = IdempotencyKey::new(scope, OperationKind::Send, "spec140-input-title").unwrap();
        let bytes = shared
            .lock()
            .unwrap()
            .idempotency
            .send_delivery(&key)
            .unwrap()
            .unwrap()
            .message_bytes;
        let persisted: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(
            persisted.get("thread_display_title").is_none(),
            "{persisted}"
        );
        assert!(persisted.get("thread_notice").is_none());
        drop(shared);
        let _ = std::fs::remove_file(config.db_path);
    }
