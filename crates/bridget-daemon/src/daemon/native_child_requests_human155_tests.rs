//! T028 : boîte humaine existante et fixture155 partagée, sans fournisseur.
use super::*;
use bridget_transport::protocol::{
    HumanInboxKind, HumanInboxListFilter, HumanInboxProducer, HumanInboxRefusal, HumanInboxState,
    HumanInboxSubject,
};

#[test]
fn native155_human_faits_options_decision_attestee_acquittee() {
    let mut fixture = RequestFixture155::new("human-valid155");
    let request = fixture.approval(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    let item_id = row.inbox_item_id.as_deref().unwrap();
    assert!(matches!(
        handle_wrapper_message(
            "human155",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client
            },
            &fixture.state
        ),
        Some(DaemonToWrapper::RoleAccepted { .. })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "human155",
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: crate::communication::issuer_scope("bridget-ui-control"),
                capabilities: vec![ClientCapability::ControlStateV1],
            },
            &fixture.state
        ),
        Some(DaemonToWrapper::ClientWelcome { .. })
    ));
    let items = match handle_human_inbox_list(
        &fixture.state,
        "human155",
        1,
        HumanInboxListFilter::Open,
        10,
    ) {
        DaemonToWrapper::HumanInbox {
            items,
            open_count: 1,
        } => items,
        other => panic!("un item humain réellement déposé : {other:?}"),
    };
    let item = items.iter().find(|item| item.id == item_id).unwrap();
    assert_eq!(item.options, vec!["accept", "decline"]);
    let context: Value = serde_json::from_str(&item.context).unwrap();
    assert_eq!(
        context["facts"]["method"],
        "item/commandExecution/requestApproval"
    );
    assert_eq!(
        context["facts"]["command"],
        format!("{:?}", "printf fixture155")
    );
    assert!(context["facts"]["cwd"].is_null());
    assert_eq!(context["facts"]["cwd_verified"], false);
    assert_eq!(context["facts"]["parent_sandbox"], "dangerFullAccess");
    assert_eq!(
        context["model_text"]["quoted_untrusted"],
        "Commande fixture"
    );
    assert!(
        item.context.find("\"facts\"").unwrap()
            < item.context.find("\"quoted_untrusted\"").unwrap()
    );
    assert!(matches!(
        handle_human_inbox_resolve(&fixture.state, "conn-1", 1, item_id, "accept"),
        DaemonToWrapper::HumanInboxRejected {
            reason: HumanInboxRefusal::HumanPrincipalRequired
        }
    ));
    let decision =
        match handle_human_inbox_resolve(&fixture.state, "human155", 1, item_id, "accept") {
            DaemonToWrapper::HumanInbox { items, .. } => items[0].decision.clone().unwrap(),
            other => panic!("décision du principal humain : {other:?}"),
        };
    assert_eq!(decision.actor, "humain");
    native_delegation::tick(&fixture.state);
    let updated = fixture.row(&request);
    assert_eq!(updated.state, "answered");
    assert_eq!(updated.decision.as_deref(), Some("accept"));
    assert_eq!(updated.decided_by.as_deref(), Some("human"));
    assert_eq!(updated.outcome.as_deref(), Some("answered_by_human"));
    assert!(updated.inbox_settled);
    {
        let st = fixture.state.lock().unwrap();
        let item = crate::human_inbox::get(st.store.connection(), item_id)
            .unwrap()
            .unwrap();
        assert!(
            item.acked_at.is_some(),
            "application durable avant acquittement de l'item"
        );
        assert_eq!(
            item.decision.as_ref().unwrap().decision_id,
            decision.decision_id
        );
    }
    assert!(fixture.frames().iter().any(|frame| matches!(frame, DaemonToWrapper::ChildRequestAnswer { answer }
        if answer.request_id == row.request_id && answer.decision == ChildRequestDecision::Accept && answer.answer.is_none())));
    handle_wrapper_message(
        "child155",
        WrapperToDaemon::ChildRequestAnswered {
            request_id: row.request_id.clone(),
            applied: true,
            reason: None,
        },
        &fixture.state,
    );
    assert_eq!(fixture.row(&request).delivery_state, "delivered");
}

#[test]
fn native155_human_decisions_obsoletes_expiration_annulation_purge() {
    for cause in ["expired", "cancelled", "purged"] {
        let mut fixture = RequestFixture155::new("human-stale155");
        let request = fixture.approval(1);
        fixture.receive(request.clone());
        let row = fixture.row(&request);
        let item_id = row.inbox_item_id.as_deref().unwrap();
        assert!(matches!(
            handle_wrapper_message(
                "human155",
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Client
                },
                &fixture.state
            ),
            Some(DaemonToWrapper::RoleAccepted { .. })
        ));
        assert!(matches!(
            handle_wrapper_message(
                "human155",
                WrapperToDaemon::ClientHello {
                    contract_version: CLIENT_CONTRACT_VERSION,
                    issuer_scope: crate::communication::issuer_scope("bridget-ui-control"),
                    capabilities: vec![ClientCapability::ControlStateV1],
                },
                &fixture.state
            ),
            Some(DaemonToWrapper::ClientWelcome { .. })
        ));
        // Le backend152 doit exister AVANT le premier tick qui demande
        // STOP. Une confirmation tardive ne répare pas un receiver expiré.
        let completion = if cause == "expired" {
            None
        } else {
            let stop = fixture.state.lock().unwrap().managed_spawns[&fixture.task.task_id]
                .stop
                .clone();
            Some(thread::spawn(move || {
                let deadline = Instant::now() + Duration::from_secs(3);
                while !stop.is_requested() && Instant::now() < deadline {
                    thread::sleep(Duration::from_millis(1));
                }
                let requested = stop.is_requested();
                if requested {
                    stop.complete(StopOutcome::Stopped);
                }
                requested
            }))
        };
        if cause == "expired" {
            fixture.expired(&row);
        } else {
            let mut st = fixture.state.lock().unwrap();
            handle_locked(
                "conn-1",
                NativeDelegationRequest::Cancel {
                    task_id: fixture.task.task_id.clone(),
                },
                &mut st,
            )
            .unwrap();
        }
        if cause != "purged" {
            assert!(matches!(
                handle_human_inbox_resolve(&fixture.state, "human155", 1, item_id, "accept"),
                DaemonToWrapper::HumanInbox { .. }
            ));
            native_delegation::tick(&fixture.state);
            if let Some(completion) = completion {
                assert!(completion.join().unwrap(), "arrêt réellement demandé");
            }
            let updated = fixture.row(&request);
            assert!(matches!(updated.state.as_str(), "expired" | "cancelled"));
            assert!(updated.inbox_settled);
            assert_ne!(updated.decision.as_deref(), Some("accept"));
            let st = fixture.state.lock().unwrap();
            assert!(
                crate::human_inbox::get(st.store.connection(), item_id)
                    .unwrap()
                    .unwrap()
                    .acked_at
                    .is_some(),
                "{cause}: décision tardive acquittée sans application"
            );
        } else {
            // Prépurge réelle : Cancel/tick doit solder l'item avant toute
            // suppression du lien155. Aucune clôture manuelle ne masque T029.
            native_delegation::tick(&fixture.state);
            assert!(
                completion.unwrap().join().unwrap(),
                "arrêt réellement demandé"
            );
            {
                let st = fixture.state.lock().unwrap();
                assert_eq!(
                    crate::human_inbox::get(st.store.connection(), item_id)
                        .unwrap()
                        .unwrap()
                        .state,
                    HumanInboxState::ClosedSelf
                );
                assert!(
                    st.delegation_store
                        .get_child_request(&row.request_id)
                        .unwrap()
                        .unwrap()
                        .inbox_settled
                );
            }
            handle_wrapper_message(
                "child155",
                WrapperToDaemon::ChildRequestClosed {
                    request_id: row.request_id.clone(),
                    state: ChildRequestClosedState::ClosedBeforeApply,
                },
                &fixture.state,
            );
            native_delegation::tick(&fixture.state);
            {
                let st = fixture.state.lock().unwrap();
                let task = st
                    .delegation_store
                    .get(&fixture.task.task_id)
                    .unwrap()
                    .unwrap();
                assert_eq!(task.state, "cancelled");
                assert!(task.cleanup_done);
                assert_eq!(
                    st.delegation_store
                        .purge_trees(unix_timestamp() + 8 * 24 * 3600, None, 1, |_| true)
                        .unwrap(),
                    1
                );
                assert!(
                    st.delegation_store
                        .get_child_request(&row.request_id)
                        .unwrap()
                        .is_none()
                );
            }
            assert!(
                matches!(
                    handle_human_inbox_resolve(&fixture.state, "human155", 1, item_id, "accept"),
                    DaemonToWrapper::HumanInboxRejected {
                        reason: HumanInboxRefusal::AlreadyResolved
                    }
                ),
                "après purge, l'item déjà fermé rejette le nouvel événement sans créer de décision"
            );
            let st = fixture.state.lock().unwrap();
            let item = crate::human_inbox::get(st.store.connection(), item_id)
                .unwrap()
                .unwrap();
            assert_eq!(item.state, HumanInboxState::ClosedSelf);
            assert!(item.decision.is_none());
        }
        assert!(
            !fixture.frames().iter().any(
                |frame| matches!(frame, DaemonToWrapper::ChildRequestAnswer { answer }
            if answer.decision == ChildRequestDecision::Accept)
            ),
            "{cause}: aucun accord fournisseur"
        );
    }
}

#[test]
fn native155_human_250_decisions_obsoletes_ne_bloquent_pas_la_nouvelle() {
    let mut fixture = RequestFixture155::new("human-backlog155");
    assert!(matches!(
        handle_wrapper_message(
            "human155",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client
            },
            &fixture.state
        ),
        Some(DaemonToWrapper::RoleAccepted { .. })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "human155",
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: crate::communication::issuer_scope("bridget-ui-control"),
                capabilities: vec![ClientCapability::ControlStateV1],
            },
            &fixture.state
        ),
        Some(DaemonToWrapper::ClientWelcome { .. })
    ));
    {
        let st = fixture.state.lock().unwrap();
        let actor = control_client_actor(&st, "human155").unwrap();
        for index in 0..250 {
            // Historique orphelin : aucun task/request encore exécutable. Les
            // timestamps anciens placent la vraie décision derrière la borne200.
            let subject = HumanInboxSubject {
                objective_id: None,
                delegation_id: Some(format!("purged-{index}")),
                message_id: None,
                project_id: None,
                agent_id: None,
            };
            let options = vec!["accept".into(), "decline".into()];
            let key = format!("native-child-request:obsolete-{index}");
            let deposited = crate::human_inbox::deposit(
                st.store.connection(),
                crate::human_inbox::DepositRequest {
                    dedup_key: &key,
                    kind: HumanInboxKind::InterventionRequired,
                    subject: &subject,
                    context: "{}",
                    options: &options,
                    producer: HumanInboxProducer::Daemon,
                    now: 100,
                },
            )
            .unwrap()
            .unwrap();
            crate::human_inbox::resolve(
                st.store.connection(),
                &deposited.item_id,
                "accept",
                actor,
                110,
            )
            .unwrap()
            .unwrap();
        }
        assert_eq!(
            crate::human_inbox::pending_decisions(
                st.store.connection(),
                HumanInboxProducer::Daemon,
                200
            )
            .unwrap()
            .len(),
            200
        );
    }
    let request = fixture.approval(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    let item_id = row.inbox_item_id.as_deref().unwrap();
    assert!(matches!(
        handle_human_inbox_resolve(&fixture.state, "human155", 1, item_id, "accept"),
        DaemonToWrapper::HumanInbox { .. }
    ));
    native_delegation::tick(&fixture.state);
    assert_eq!(
        fixture.row(&request).state,
        "answered",
        "lecture exacte de l'item, pas les200 premières décisions globales"
    );
    let frames = fixture.frames();
    let answers: Vec<_> = frames
        .iter()
        .filter_map(|frame| {
            if let DaemonToWrapper::ChildRequestAnswer { answer } = frame {
                Some(answer)
            } else {
                None
            }
        })
        .collect();
    assert!(!answers.is_empty(), "la nouvelle décision atteint l'enfant");
    assert!(
        answers
            .iter()
            .all(|answer| answer.request_id == row.request_id
                && answer.decision == ChildRequestDecision::Accept),
        "aucun accord issu des250 items obsolètes"
    );
    {
        let st = fixture.state.lock().unwrap();
        assert!(
            crate::human_inbox::get(st.store.connection(), item_id)
                .unwrap()
                .unwrap()
                .acked_at
                .is_some()
        );
    }
}

#[test]
fn native155_human_item_ouvert_ferme_a_expiration() {
    let mut fixture = RequestFixture155::new("human-expire155");
    let request = fixture.approval(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    let item_id = row.inbox_item_id.as_deref().unwrap();
    fixture.expired(&row);
    native_delegation::tick(&fixture.state);
    {
        let st = fixture.state.lock().unwrap();
        let item = crate::human_inbox::get(st.store.connection(), item_id)
            .unwrap()
            .unwrap();
        assert_eq!(item.state, HumanInboxState::ClosedSelf);
        assert!(item.decision.is_none());
        assert_eq!(
            st.delegation_store
                .get_child_request(&row.request_id)
                .unwrap()
                .unwrap()
                .state,
            "expired"
        );
        assert!(
            st.delegation_store
                .get_child_request(&row.request_id)
                .unwrap()
                .unwrap()
                .inbox_settled
        );
    }
    assert!(!fixture.frames().iter().any(
        |frame| matches!(frame, DaemonToWrapper::ChildRequestAnswer { answer }
        if answer.decision == ChildRequestDecision::Accept)
    ));
}

#[test]
fn native155_human_ne_peut_pas_repondre_a_une_question() {
    let mut fixture = RequestFixture155::new("human-question155");
    let request = fixture.question(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    assert_eq!(row.verdict, "parent");
    assert!(row.inbox_item_id.is_none());
    assert!(matches!(
        handle_wrapper_message(
            "human155",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client
            },
            &fixture.state
        ),
        Some(DaemonToWrapper::RoleAccepted { .. })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "human155",
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: crate::communication::issuer_scope("bridget-ui-control"),
                capabilities: vec![ClientCapability::ControlStateV1],
            },
            &fixture.state
        ),
        Some(DaemonToWrapper::ClientWelcome { .. })
    ));
    let item_id = {
        let st = fixture.state.lock().unwrap();
        assert_eq!(
            crate::human_inbox::open_count(st.store.connection()).unwrap(),
            0
        );
        // Un lien historique erroné ne change pas la route Parent vérifiée.
        let subject = HumanInboxSubject {
            objective_id: None,
            delegation_id: Some(fixture.task.task_id.clone()),
            message_id: Some(fixture.task.mission.clone()),
            project_id: None,
            agent_id: Some(fixture.task.child.clone()),
        };
        let key = format!("native-child-request:{}", row.request_id);
        let options = vec!["accept".into(), "decline".into()];
        let deposited = crate::human_inbox::deposit(
            st.store.connection(),
            crate::human_inbox::DepositRequest {
                dedup_key: &key,
                kind: HumanInboxKind::InterventionRequired,
                subject: &subject,
                context: "{}",
                options: &options,
                producer: HumanInboxProducer::Daemon,
                now: unix_timestamp(),
            },
        )
        .unwrap()
        .unwrap();
        st.delegation_store
            .link_child_inbox(&row.request_id, &deposited.item_id)
            .unwrap();
        deposited.item_id
    };
    assert!(matches!(
        handle_human_inbox_resolve(&fixture.state, "human155", 1, &item_id, "accept"),
        DaemonToWrapper::HumanInbox { .. }
    ));
    native_delegation::tick(&fixture.state);
    let unchanged = fixture.row(&request);
    assert_eq!(unchanged.state, "open");
    assert!(unchanged.decision.is_none());
    assert_eq!(unchanged.delivery_state, "none");
    assert!(!fixture.frames().iter().any(
        |frame| matches!(frame, DaemonToWrapper::ChildRequestAnswer { answer }
        if answer.request_id == row.request_id)
    ));
    {
        let st = fixture.state.lock().unwrap();
        assert!(
            crate::human_inbox::get(st.store.connection(), &item_id)
                .unwrap()
                .unwrap()
                .acked_at
                .is_some(),
            "acquittement sans transformer la question en approbation"
        );
    }
}
