use super::*;
use bridget_transport::poison::RecoverPoison;

#[test]
fn spec271_completed_native_round_names_continue_before_physical_project_check() {
    let (template, template_config, tasks) = control158_state_with_tasks("q271t");
    let mut task = tasks[1].clone();
    drop(template);
    let _ = std::fs::remove_file(template_config.db_path);
    let (st, config) = state_with_registered_agent("queue271");
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        &crate::communication::issuer_scope("queue271"),
        "89000000-0000-4000-8000-000000000200",
    );
    let (pending_writer, _pending_reader) = control_socket("queue271-pending");
    {
        let mut state = shared.lock().unwrap();
        task.owner = state.conn_names["guidance-client"].clone();
        task.owner_instance = state.conn_instances["guidance-client"].clone();
        task.child = "89000000-0000-4000-8000-000000000102".into();
        task.child_instance = Some(state.conn_instances["conn-1"].clone());
        task.state = "result_available".into();
        state
            .delegation_store
            .insert(
                &task,
                "queue271",
                &serde_json::to_vec(&task.request).unwrap(),
            )
            .unwrap();
        for intent in [
            None,
            Some(bridget_core::MessageIntent::QueueOnly),
            Some(bridget_core::MessageIntent::SteerCurrent),
            Some(bridget_core::MessageIntent::InterruptAndStart),
        ] {
            let mut message = idempotent_message("complément après résultat");
            message.intent = intent;
            let result = prepare_dispatch(
                &mut state,
                &mut message,
                "guidance-client",
                task.owner.clone(),
                "fixture".into(),
                "fixture".into(),
                false,
            );
            let Err((code, reason)) = result else {
                panic!("nouvelle ronde native requise")
            };
            assert_eq!(code, "intent_not_permitted");
            assert!(reason.contains("bridget_task_continue") && reason.contains(&task.task_id));
            assert!(!reason.contains("cross_project_reason_required"));
        }
        assert_eq!(
            state
                .delegation_store
                .get(&task.task_id)
                .unwrap()
                .unwrap()
                .state,
            "result_available"
        );
        state
            .store
            .create_request("tracked271", &task.child, &task.owner, 600)
            .unwrap();
        for (intent, native, reply) in [
            (Some(bridget_core::MessageIntent::StoreOnly), false, false),
            (None, true, false),
            (None, false, true),
        ] {
            let mut message = idempotent_message("réponse ou avis existant");
            message.intent = intent;
            message.in_reply_to = reply.then(|| "tracked271".into());
            let result = prepare_dispatch(
                &mut state,
                &mut message,
                "guidance-client",
                task.owner.clone(),
                "fixture".into(),
                "fixture".into(),
                native,
            );
            assert!(match result {
                Ok(_) => true,
                Err((code, _)) => code != "intent_not_permitted",
            }, "l’exemption doit franchir la garde de fin de ronde");
        }
        task.state = "working".into();
        state.delegation_store.save(&task).unwrap();
        state.connections.insert("conn-1".into(), pending_writer);
    }
    let mut queued = idempotent_message("déjà admis avant la fin");
    queued.intent = Some(bridget_core::MessageIntent::QueueOnly);
    let issued_at = unix_now_secs();
    let send = || WrapperToDaemon::SendIdempotent {
        message: queued.clone(),
        message_id: "pending271".into(),
        issued_at,
    };
    assert!(matches!(
        handle_wrapper_message("guidance-client", send(), &shared),
        Some(DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::OutcomeUnknown { .. },
            ..
        })
    ));
    {
        let state = shared.lock().unwrap();
        task.state = "result_available".into();
        state.delegation_store.save(&task).unwrap();
    }
    assert!(
        matches!(
            handle_wrapper_message("guidance-client", send(), &shared),
            Some(DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            })
        ),
        "un rejeu en file ne devient pas une nouvelle consigne refusée"
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn spec268_journal_terminal_nettoye_reste_connu_sans_presence_volatile() {
    let (shared, config, tasks) = control158_state_with_tasks("journal-durable");
    {
        let state = shared.lock().unwrap();
        let mut task = tasks[0].clone();
        assert!(!state.presences.values().any(|p| p.name == task.child));
        for phase in ["result_available", "failed", "cancelled"] {
            task.state = phase.into();
            task.cleanup_done = true;
            state.delegation_store.save(&task).unwrap();
            assert_eq!(
                attach_refusal_for_subscription(&state, &task.child)
                    .unwrap_err()
                    .reason,
                AttachRefusal::AgentStopped,
                "la fin durable nettoyée doit survivre à la perte de présence : {phase}"
            );
        }
        for (phase, cleaned) in [("working", true), ("cancelled", false)] {
            task.state = phase.into();
            task.cleanup_done = cleaned;
            state.delegation_store.save(&task).unwrap();
            assert_eq!(
                attach_refusal_for_subscription(&state, &task.child)
                    .unwrap_err()
                    .reason,
                AttachRefusal::AgentUnknown,
                "un état sans arrêt confirmé ne doit pas devenir arrêté"
            );
        }
        assert_eq!(
            attach_refusal_for_subscription(&state, "inconnu268")
                .unwrap_err()
                .reason,
            AttachRefusal::AgentUnknown
        );
        let mut duplicate = tasks[1].clone();
        duplicate.child = task.child.clone();
        assert!(
            state.delegation_store.save(&duplicate).is_err(),
            "l'index unique existant interdit une ancienne tâche homonyme"
        );
        state
            .store
            .connection()
            .execute_batch("DROP TABLE native_delegations")
            .unwrap();
        assert_eq!(
            attach_refusal_for_subscription(&state, &task.child)
                .unwrap_err()
                .reason,
            AttachRefusal::AgentUnknown,
            "une lecture durable en erreur ne prouve aucun arrêt"
        );
    }
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

// T017 : vrais sockets, preuve primaire réemployée et une seule négociation.
fn edit159_cancel_auxiliary(
    f: &Held159SweepFixture,
    connection: &str,
    scope: &str,
    agent: &str,
) -> BufReader<UnixStream> {
    let (writer, peer) = control_socket("edit159-aux");
    let (instance_id, credential) = {
        let mut st = f.shared.lock().unwrap();
        let owner = st.router.get_agent(agent).unwrap().connection_id.clone();
        let instance_id = st.conn_instances[&owner].clone();
        let credential = st.identity_credentials[&owner].clone();
        st.connections.insert(connection.into(), writer);
        (instance_id, credential)
    };
    assert!(matches!(
        handle_wrapper_message(
            connection,
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client,
            },
            &f.shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        })
    ));
    assert!(matches!(
        handle_wrapper_message(
            connection,
            WrapperToDaemon::RegisterAuxiliary {
                agent_id: agent.into(),
                instance_id,
                credential,
            },
            &f.shared
        ),
        Some(DaemonToWrapper::Registered {
            credential: None,
            ..
        })
    ));
    assert!(
        matches!(handle_wrapper_message(connection, WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION, issuer_scope: scope.into(),
            capabilities: vec![ClientCapability::QueueEditV1],
        }, &f.shared), Some(DaemonToWrapper::ClientWelcome { capabilities, .. })
            if capabilities == vec![ClientCapability::QueueEditV1])
    );
    assert_eq!(
        live_connection_identity(&f.shared.lock().unwrap(), connection)
            .unwrap()
            .0,
        agent
    );
    peer
}

fn edit159_cancel_fixture() -> (Held159SweepFixture, BufReader<UnixStream>) {
    let f = held159_sweep_fixture();
    // held159-client garde son Hello SendOnly; une nouvelle connexion édite.
    let peer = edit159_cancel_auxiliary(
        &f,
        "edit159-client",
        "012_scope_held159_oracle",
        &f.message.from,
    );
    (f, peer)
}

fn edit159_cancel_request(
    f: &Held159SweepFixture,
    connection: &str,
    request: &str,
) -> DaemonToWrapper {
    handle_wrapper_message(
        connection,
        WrapperToDaemon::QueueEditRequest {
            request_id: request.into(),
            target: f.message.to.clone(),
            action: bridget_transport::protocol::QueueEditAction::Cancel,
            message_id: f.message.id.clone(),
            before_message_id: None,
        },
        &f.shared,
    )
    .unwrap()
}

fn edit159_cancel_applied(response: &DaemonToWrapper, request: &str) {
    assert!(
        matches!(response, DaemonToWrapper::QueueEditResult {
            request_id, state: bridget_transport::protocol::QueueEditState::Applied,
            refusal: None, effect: None,
        } if request_id == request),
        "résultat de retrait final attendu: {response:?}"
    );
}

#[test]
fn edit159_cancel_retient_puis_retire_jamais_livre() {
    let (mut f, _editor_peer) = edit159_cancel_fixture();
    edit159_cancel_applied(
        &edit159_cancel_request(&f, "edit159-client", "cancel159-never"),
        "cancel159-never",
    );
    assert_eq!(
        held159_release_phase(&f.shared, &f.delivery_id),
        "cancelled"
    );
    assert!(
        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::TurnState { in_progress: false },
            &f.shared
        )
        .is_none()
    );
    assert_eq!(
        held159_release_phase(&f.shared, &f.delivery_id),
        "cancelled"
    );
    f._target
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(20)))
        .unwrap();
    let mut line = String::new();
    assert!(
        matches!(f._target.read_line(&mut line), Err(e)
            if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut)),
        "{line}"
    );
    assert!(line.is_empty(), "aucune remise retirée: {line}");
}

#[test]
fn edit159_cancel_suivi_de_reponse_clos_en_base_et_en_memoire() {
    let (f, _editor_peer) = edit159_cancel_fixture();
    assert!(
        f.shared
            .lock()
            .unwrap()
            .pending_replies
            .iter()
            .any(|p| p.msg_id == f.message.id)
    );
    assert!(f.shared.lock().unwrap().held_ids.contains(&f.message.id));
    edit159_cancel_applied(
        &edit159_cancel_request(&f, "edit159-client", "cancel159-tracking"),
        "cancel159-tracking",
    );
    let st = f.shared.lock().unwrap();
    assert_eq!(
        st.store.get_request(&f.message.id).unwrap().unwrap().state,
        "cancelled"
    );
    assert!(!st.pending_replies.iter().any(|p| p.msg_id == f.message.id));
    assert!(!st.held_ids.contains(&f.message.id));
}

#[test]
fn edit159_cancel_aucun_rappel_apres_retrait() {
    let (f, _editor_peer) = edit159_cancel_fixture();
    edit159_cancel_applied(
        &edit159_cancel_request(&f, "edit159-client", "cancel159-reminders"),
        "cancel159-reminders",
    );
    for elapsed in [20, 40, 60, 91] {
        assert!(
            held159_sweep_cycle(&f, elapsed).is_empty(),
            "aucun palier après retrait/{elapsed}"
        );
    }
    let st = f.shared.lock().unwrap();
    assert_eq!(
        st.store.get_request(&f.message.id).unwrap().unwrap().state,
        "cancelled"
    );
    assert!(
        st.store
            .latest_deferred_reminder(&f.message.id)
            .unwrap()
            .is_none()
    );
}

#[test]
fn edit159_cancel_droits_auteur_parent_humain() {
    for role in ["auteur", "parent", "humain"] {
        let (f, _editor_peer) = edit159_cancel_fixture();
        let mut retained_peers = Vec::new();
        let connection = match role {
            "auteur" => "edit159-client",
            "parent" => {
                let parent = "89000000-0000-4000-8000-000000000201";
                negotiate_idempotent_client(
                    &f.shared,
                    "edit159-parent-bootstrap",
                    "012_scope_edit159_parent",
                    parent,
                );
                let (writer, peer) = control_socket("edit159-parent-primary");
                {
                    let mut st = f.shared.lock().unwrap();
                    let primary = st.router.get_agent(parent).unwrap().connection_id.clone();
                    st.connections.insert(primary, writer);
                    guidance156_link_parent(&mut st, "edit159-parent-bootstrap", "conn-1");
                }
                retained_peers.push(peer);
                retained_peers.push(edit159_cancel_auxiliary(
                    &f,
                    "edit159-parent",
                    "012_scope_edit159_parent",
                    parent,
                ));
                "edit159-parent"
            }
            "humain" => {
                let (writer, peer) = control_socket("edit159-human");
                retained_peers.push(peer);
                f.shared
                    .lock()
                    .unwrap()
                    .connections
                    .insert("edit159-human".into(), writer);
                assert!(matches!(
                    handle_wrapper_message(
                        "edit159-human",
                        WrapperToDaemon::RoleHandshake {
                            role: ConnectionRole::Attach,
                        },
                        &f.shared
                    ),
                    Some(DaemonToWrapper::RoleAccepted {
                        role: ConnectionRole::Attach
                    })
                ));
                assert!(matches!(
                    handle_wrapper_message(
                        "edit159-human",
                        WrapperToDaemon::ClientHello {
                            contract_version: CLIENT_CONTRACT_VERSION,
                            issuer_scope: "012_scope_edit159_human".into(),
                            capabilities: vec![ClientCapability::QueueEditV1],
                        },
                        &f.shared
                    ),
                    Some(DaemonToWrapper::ClientWelcome { .. })
                ));
                assert_eq!(
                    guidance_source(&f.shared.lock().unwrap(), Some("edit159-human")).0,
                    crate::guidance::GuidanceActor::AttachedHuman
                );
                "edit159-human"
            }
            _ => unreachable!(),
        };
        let request = format!("cancel159-rights-{role}");
        let first = edit159_cancel_request(&f, connection, &request);
        edit159_cancel_applied(&first, &request);
        assert_eq!(
            held159_release_phase(&f.shared, &f.delivery_id),
            "cancelled"
        );
        let replay = edit159_cancel_request(&f, connection, &request);
        assert_eq!(
            encode(&replay).unwrap(),
            encode(&first).unwrap(),
            "même résultat rejoué pour {role}"
        );
        assert_eq!(
            held159_release_phase(&f.shared, &f.delivery_id),
            "cancelled"
        );
        drop(retained_peers);
    }
}

#[test]
fn edit159_cancel_destinataire_et_pair_refuses() {
    for recipient in [true, false] {
        let (f, _editor_peer) = edit159_cancel_fixture();
        let actor = if recipient {
            f.message.to.as_str()
        } else {
            "89000000-0000-4000-8000-000000000301"
        };
        negotiate_idempotent_client(
            &f.shared,
            "edit159-denied-bootstrap",
            "012_scope_edit159_denied",
            actor,
        );
        let (writer, peer) = control_socket("edit159-denied-primary");
        {
            let mut st = f.shared.lock().unwrap();
            let primary = st.router.get_agent(actor).unwrap().connection_id.clone();
            st.connections.insert(primary, writer);
        }
        let _denied_editor_peer =
            edit159_cancel_auxiliary(&f, "edit159-denied", "012_scope_edit159_denied", actor);
        assert_eq!(
            live_connection_identity(&f.shared.lock().unwrap(), "edit159-denied")
                .unwrap()
                .0,
            actor
        );
        let response = edit159_cancel_request(&f, "edit159-denied", "cancel159-denied");
        let reason = if recipient {
            "self_target"
        } else {
            "not_parent"
        };
        assert!(
            matches!(response, DaemonToWrapper::QueueEditResult {
                state: bridget_transport::protocol::QueueEditState::Refused, refusal: Some(ref code), effect: None, ..
            } if code.contains("intent_not_permitted") && code.contains(reason)),
            "{response:?}"
        );
        assert_eq!(held159_release_phase(&f.shared, &f.delivery_id), "held");
        let st = f.shared.lock().unwrap();
        assert_eq!(
            st.store.get_request(&f.message.id).unwrap().unwrap().state,
            "open"
        );
        assert!(st.held_ids.contains(&f.message.id));
        assert!(st.pending_replies.iter().any(|p| p.msg_id == f.message.id));
        drop(peer);
    }
}

#[test]
fn edit159_cancel_pause_admis() {
    let (f, _editor_peer) = edit159_cancel_fixture();
    {
        let st = f.shared.lock().unwrap();
        assert!(
            crate::referent_control::set(
                st.store.connection(),
                crate::referent_control::ControlMutation {
                    command_id: "edit159-cancel-pause",
                    expected_generation: 0,
                    paused: Some(true),
                    auto_objectives_cap: None,
                    reason: Some("oracle cancel159"),
                    actor: "human",
                    now: unix_now_secs(),
                    agent_posture: None,
                    auto_reassignment: None,
                }
            )
            .unwrap()
            .is_ok()
        );
    }
    edit159_cancel_applied(
        &edit159_cancel_request(&f, "edit159-client", "cancel159-paused"),
        "cancel159-paused",
    );
    assert_eq!(
        held159_release_phase(&f.shared, &f.delivery_id),
        "cancelled"
    );
    assert!(
        crate::referent_control::read(f.shared.lock().unwrap().store.connection())
            .unwrap()
            .paused
    );
}

#[test]
fn edit159_cancel_aucune_interruption_de_tour() {
    let (mut f, _editor_peer) = edit159_cancel_fixture();
    let before = {
        let st = f.shared.lock().unwrap();
        let p = &st.presences["instance-1"];
        (
            p.state.clone(),
            p.busy_since,
            p.busy_period_start,
            p.last_attested_at,
        )
    };
    edit159_cancel_applied(
        &edit159_cancel_request(&f, "edit159-client", "cancel159-no-interrupt"),
        "cancel159-no-interrupt",
    );
    {
        let st = f.shared.lock().unwrap();
        let p = &st.presences["instance-1"];
        assert_eq!(
            (
                p.state.clone(),
                p.busy_since,
                p.busy_period_start,
                p.last_attested_at
            ),
            before
        );
        assert_eq!(
            st.idempotency
                .count_interrupts_target(&f.message.to, unix_now_secs() - 600)
                .unwrap()
                .0,
            0
        );
    }
    f._target
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(20)))
        .unwrap();
    let mut line = String::new();
    assert!(
        matches!(f._target.read_line(&mut line), Err(e)
            if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut)),
        "{line}"
    );
    assert!(
        line.is_empty(),
        "aucun CancelDelivery ni autre commande: {line}"
    );
}

#[test]
fn edit159_cancel_crash_apres_commit_message_non_livre() {
    let (f, _editor_peer) = edit159_cancel_fixture();
    edit159_cancel_applied(
        &edit159_cancel_request(&f, "edit159-client", "cancel159-restart"),
        "cancel159-restart",
    );
    assert_eq!(
        held159_release_phase(&f.shared, &f.delivery_id),
        "cancelled"
    );
    let Held159SweepFixture {
        shared,
        config,
        message,
        delivery_id,
        author,
        _target,
        ..
    } = f;
    drop(author);
    drop(_target);
    drop(_editor_peer);
    let mut old = Arc::try_unwrap(shared)
        .ok()
        .expect("aucun autre propriétaire de fixture")
        .into_inner()
        .unwrap();
    let original_instance = old.instance_id.clone();
    let fixture_root = old.fixture_root.take().unwrap();
    let registry_home = fixture_root.0.clone();
    drop(old); // Fermeture de toutes les connexions SQL: frontière durable après COMMIT.
    let mut st = {
        let guard = HOME_REGISTRY_LOCK.lock().recover_poison();
        let previous = std::env::var_os("HOME");
        // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
        unsafe { std::env::set_var("HOME", &registry_home) };
        let (tx, _rx) = mpsc::channel();
        let result = DaemonState::new(&config, tx);
        if let Some(home) = previous {
            // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
            unsafe { std::env::set_var("HOME", home) };
        } else {
            // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
            unsafe { std::env::remove_var("HOME") };
        }
        drop(guard);
        result.unwrap()
    };
    assert_ne!(st.instance_id, original_instance);
    st.fixture_root = Some(fixture_root);
    st.registry = crate::registry::AgentRegistry::from_json(
            r#"{"agents":{"claude":{"command":"/bin/sh","protocol":"acp","held_queue":true,"queue_capacity":32}}}"#,
            st.registry.source().to_path_buf(),
        ).unwrap();
    let (writer, mut reader) = control_socket("edit159-after-commit");
    st.connections.insert("edit159-reconnected".into(), writer);
    assert!(matches!(
        handle_register_with_channel(
            "edit159-reconnected",
            2,
            "claude".into(),
            message.to.clone(),
            Some("fixture".into()),
            Some("acp".into()),
            ChannelReport::Known("unix".into()),
            Some(PresenceMode::Acp),
            None,
            Some("test".into()),
            Some("edit159-restart-instance".into()),
            None,
            false,
            Some(true),
            &mut st,
            true,
        ),
        DaemonToWrapper::Registered { .. }
    ));
    assert!(!st.held_ids.contains(&message.id));
    assert!(
        !st.idempotency
            .dispatching_deliveries_for_instance("edit159-restart-instance", unix_now_secs())
            .unwrap()
            .iter()
            .any(|d| d.delivery_id == delivery_id)
    );
    assert!(!schedule_held_release(&mut st, &message.to, Instant::now()));
    assert_eq!(
        st.idempotency
            .send_delivery_phase(&IdempotencyKey {
                issuer_scope: "012_scope_held159_oracle".into(),
                operation_kind: OperationKind::Send,
                idempotency_key: message.id.clone()
            })
            .unwrap()
            .as_deref(),
        Some("cancelled")
    );
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(20)))
        .unwrap();
    let mut line = String::new();
    assert!(
        matches!(reader.read_line(&mut line), Err(e)
            if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut)),
        "{line}"
    );
    assert!(line.is_empty());
}

// T019/T021 : même montage des deux seuls rôles autorisés, sockets gardées.
fn edit159_queue_actor(
    f: &Held159SweepFixture,
    human: bool,
) -> (&'static str, Vec<BufReader<UnixStream>>) {
    let mut peers = Vec::new();
    if human {
        let (writer, peer) = control_socket("edit159-humanB");
        peers.push(peer);
        f.shared
            .lock()
            .unwrap()
            .connections
            .insert("edit159-humanB".into(), writer);
        assert!(matches!(
            handle_wrapper_message(
                "edit159-humanB",
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Attach,
                },
                &f.shared
            ),
            Some(DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Attach
            })
        ));
        assert!(matches!(
            handle_wrapper_message(
                "edit159-humanB",
                WrapperToDaemon::ClientHello {
                    contract_version: CLIENT_CONTRACT_VERSION,
                    issuer_scope: "012_scope_edit159_humanB".into(),
                    capabilities: vec![ClientCapability::QueueEditV1],
                },
                &f.shared
            ),
            Some(DaemonToWrapper::ClientWelcome { .. })
        ));
        assert_eq!(
            guidance_source(&f.shared.lock().unwrap(), Some("edit159-humanB")).0,
            crate::guidance::GuidanceActor::AttachedHuman
        );
        ("edit159-humanB", peers)
    } else {
        let parent = "89000000-0000-4000-8000-000000000201";
        negotiate_idempotent_client(
            &f.shared,
            "edit159-parentB-bootstrap",
            "012_scope_edit159_parentB",
            parent,
        );
        let (writer, peer) = control_socket("edit159-parentB-primary");
        peers.push(peer);
        {
            let mut st = f.shared.lock().unwrap();
            let primary = st.router.get_agent(parent).unwrap().connection_id.clone();
            st.connections.insert(primary, writer);
            guidance156_link_parent(&mut st, "edit159-parentB-bootstrap", "conn-1");
        }
        peers.push(edit159_cancel_auxiliary(
            f,
            "edit159-parentB",
            "012_scope_edit159_parentB",
            parent,
        ));
        ("edit159-parentB", peers)
    }
}

fn edit159_reorder_request(
    f: &Held159SweepFixture,
    connection: &str,
    request: &str,
    message: &str,
    before: Option<&str>,
) -> DaemonToWrapper {
    handle_wrapper_message(
        connection,
        WrapperToDaemon::QueueEditRequest {
            request_id: request.into(),
            target: f.message.to.clone(),
            action: bridget_transport::protocol::QueueEditAction::Reorder,
            message_id: message.into(),
            before_message_id: before.map(str::to_owned),
        },
        &f.shared,
    )
    .unwrap()
}

#[test]
fn edit159_reorder_ordre_de_liberation_suit() {
    let (mut f, _editor) = edit159_cancel_fixture();
    let (connection, _peers) = edit159_queue_actor(&f, false);
    let mut second = f.message.clone();
    second.id = "edit159-order-B".into();
    second.body = "voisin B159".into();
    second.reply = false;
    second.reply_timeout = None;
    let mut third = second.clone();
    third.id = "edit159-order-C".into();
    third.body = "voisin C159".into();
    for message in [&second, &third] {
        assert!(matches!(
            held159_submit(&f.shared, message.clone(), &message.id, unix_now_secs()),
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ));
    }
    edit159_cancel_applied(
        &edit159_reorder_request(
            &f,
            connection,
            "reorder159-order",
            &third.id,
            Some(&f.message.id),
        ),
        "reorder159-order",
    );
    for expected in [&third.id, &f.message.id, &second.id] {
        held159_release_event(&f.shared, WrapperToDaemon::TurnState { in_progress: false });
        let response = read_control(&mut f._target);
        assert!(
            matches!(response, DaemonToWrapper::DeliverIdempotent { ref message, .. } if &message.id == expected),
            "{expected}/{response:?}"
        );
        held159_release_event(&f.shared, WrapperToDaemon::TurnState { in_progress: true });
    }
}

#[test]
fn edit159_reorder_droits_parent_humain() {
    for human in [false, true] {
        let (f, _editor) = edit159_cancel_fixture();
        let (connection, _peers) = edit159_queue_actor(&f, human);
        let response =
            edit159_reorder_request(&f, connection, "reorder159-rights", &f.message.id, None);
        edit159_cancel_applied(&response, "reorder159-rights");
        assert_eq!(held159_release_phase(&f.shared, &f.delivery_id), "held");
        assert_eq!(
            f.shared
                .lock()
                .unwrap()
                .store
                .get_request(&f.message.id)
                .unwrap()
                .unwrap()
                .state,
            "open"
        );
    }
}

#[test]
fn edit159_reorder_destinataire_refuse() {
    let (f, _editor) = edit159_cancel_fixture();
    let _peer = edit159_cancel_auxiliary(
        &f,
        "edit159-reorder-self",
        "012_scope_edit159_reorderself",
        &f.message.to,
    );
    let before: (i64, Vec<u8>) = f
        .shared
        .lock()
        .unwrap()
        .store
        .connection()
        .query_row(
            "SELECT queue_rank,message_bytes FROM send_deliveries WHERE delivery_id=?1",
            [&f.delivery_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    let response = edit159_reorder_request(
        &f,
        "edit159-reorder-self",
        "reorder159-self",
        &f.message.id,
        None,
    );
    assert!(
        matches!(response, DaemonToWrapper::QueueEditResult {
            state: bridget_transport::protocol::QueueEditState::Refused, refusal: Some(ref code), ..
        } if code.contains("intent_not_permitted") && code.contains("self_target")),
        "{response:?}"
    );
    let after: (i64, Vec<u8>) = f
        .shared
        .lock()
        .unwrap()
        .store
        .connection()
        .query_row(
            "SELECT queue_rank,message_bytes FROM send_deliveries WHERE delivery_id=?1",
            [&f.delivery_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(after, before);
    assert_eq!(held159_release_phase(&f.shared, &f.delivery_id), "held");
}

#[test]
fn edit159_reorder_pause_admis() {
    let (f, _editor) = edit159_cancel_fixture();
    let (connection, _peers) = edit159_queue_actor(&f, false);
    {
        let st = f.shared.lock().unwrap();
        assert!(
            crate::referent_control::set(
                st.store.connection(),
                crate::referent_control::ControlMutation {
                    command_id: "edit159-reorder-pause",
                    expected_generation: 0,
                    paused: Some(true),
                    auto_objectives_cap: None,
                    reason: Some("oracle reorder159"),
                    actor: "human",
                    now: unix_now_secs(),
                    agent_posture: None,
                    auto_reassignment: None,
                }
            )
            .unwrap()
            .is_ok()
        );
    }
    edit159_cancel_applied(
        &edit159_reorder_request(&f, connection, "reorder159-pause", &f.message.id, None),
        "reorder159-pause",
    );
    assert_eq!(held159_release_phase(&f.shared, &f.delivery_id), "held");
    assert!(
        crate::referent_control::read(f.shared.lock().unwrap().store.connection())
            .unwrap()
            .paused
    );
}

// T021 : Held réel et présence déjà attestée ; le Register public est couvert par T020.
fn edit159_promote_fixture(fallback: bool) -> (Held159SweepFixture, BufReader<UnixStream>) {
    let (f, peer) = edit159_cancel_fixture();
    {
        let mut st = f.shared.lock().unwrap();
        let (agent, instance) = live_connection_identity(&st, "conn-1").unwrap();
        assert_eq!(agent, f.message.to);
        assert_eq!(instance, "instance-1");
        // Même montage de faits que turn_attest/busy dans held159_fixture.
        let presence = st.presences.get_mut(&instance).unwrap();
        assert!(presence.turn_attest);
        assert_eq!(presence.state, "busy");
        presence.queue_fallback = fallback;
    }
    assert_eq!(held159_release_phase(&f.shared, &f.delivery_id), "held");
    (f, peer)
}

fn edit159_promote_request(
    f: &Held159SweepFixture,
    connection: &str,
    request: &str,
) -> DaemonToWrapper {
    handle_wrapper_message(
        connection,
        WrapperToDaemon::QueueEditRequest {
            request_id: request.into(),
            target: f.message.to.clone(),
            action: bridget_transport::protocol::QueueEditAction::Promote,
            message_id: f.message.id.clone(),
            before_message_id: None,
        },
        &f.shared,
    )
    .unwrap()
}

#[test]
fn edit159_promote_libere_et_pousse_en_guidage() {
    let (mut f, _editor) = edit159_promote_fixture(true);
    let (connection, _peers) = edit159_queue_actor(&f, false);
    let before: (Vec<u8>, Vec<u8>, u64) = f.shared.lock().unwrap().store.connection().query_row(
            "SELECT d.message_bytes,r.canonical_bytes,d.delivery_generation FROM send_deliveries d
             JOIN idempotency_records r USING(issuer_scope,operation_kind,idempotency_key) WHERE d.delivery_id=?1",
            [&f.delivery_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).unwrap();
    let response = edit159_promote_request(&f, connection, "promote159-push");
    assert!(
        matches!(
            response,
            DaemonToWrapper::QueueEditResult {
                state: bridget_transport::protocol::QueueEditState::Applied,
                refusal: None,
                effect: Some(bridget_transport::protocol::QueueEditEffect::Unconfirmed),
                ..
            }
        ),
        "{response:?}"
    );
    let pushed = read_control(&mut f._target);
    assert!(
        matches!(pushed, DaemonToWrapper::DeliverIdempotent { ref delivery_id, ref message, mid_turn_ok: Some(true), .. }
            if delivery_id == &f.delivery_id && message.id == f.message.id
                && message.from == f.message.from && message.to == f.message.to
                && message.body == f.message.body && message.reply == f.message.reply
                && message.intent == Some(bridget_core::MessageIntent::SteerCurrent)
                && message.references.iter().any(|r| r == "steer-or-queue")),
        "{pushed:?}"
    );
    let after: (Vec<u8>, Vec<u8>, u64) = f.shared.lock().unwrap().store.connection().query_row(
            "SELECT d.message_bytes,r.canonical_bytes,d.delivery_generation FROM send_deliveries d
             JOIN idempotency_records r USING(issuer_scope,operation_kind,idempotency_key) WHERE d.delivery_id=?1",
            [&f.delivery_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).unwrap();
    assert_eq!(after, before);
    assert_eq!(
        held159_release_phase(&f.shared, &f.delivery_id),
        "dispatching"
    );
}

#[test]
fn edit159_promote_garde_156_obligatoire_pair_refuse() {
    let (f, _editor) = edit159_promote_fixture(true);
    // L'auteur est un pair, sans lien parent : aucun droit de promotion.
    let response = edit159_promote_request(&f, "edit159-client", "promote159-peer");
    assert!(
        matches!(response, DaemonToWrapper::QueueEditResult {
            state: bridget_transport::protocol::QueueEditState::Refused, refusal: Some(ref code), ..
        } if code.contains("intent_not_permitted") && code.contains("not_parent")),
        "{response:?}"
    );
    assert_eq!(held159_release_phase(&f.shared, &f.delivery_id), "held");
    assert!(f.shared.lock().unwrap().held_ids.contains(&f.message.id));
}

#[test]
fn edit159_promote_destinataire_non_occupe_no_active_turn() {
    let (f, _editor) = edit159_promote_fixture(true);
    let (connection, _peers) = edit159_queue_actor(&f, false);
    // Fait de présence au repos, sans TurnState false qui libérerait Held.
    {
        let mut st = f.shared.lock().unwrap();
        let presence = st.presences.get_mut("instance-1").unwrap();
        presence.state = "connected".into();
        presence.busy_since = None;
        presence.busy_period_start = None;
        presence.last_attested_at = Some(Instant::now());
    }
    let response = edit159_promote_request(&f, connection, "promote159-idle");
    assert!(
        matches!(response, DaemonToWrapper::QueueEditResult {
            state: bridget_transport::protocol::QueueEditState::Refused, refusal: Some(ref code), ..
        } if code == "no_active_turn"),
        "{response:?}"
    );
    assert_eq!(held159_release_phase(&f.shared, &f.delivery_id), "held");
}

#[test]
fn edit159_promote_porteur_sans_queue_fallback_steer_unavailable_message_reste_retenu() {
    let (f, _editor) = edit159_promote_fixture(false);
    let (connection, _peers) = edit159_queue_actor(&f, false);
    let response = edit159_promote_request(&f, connection, "promote159-no-fallback");
    assert!(
        matches!(response, DaemonToWrapper::QueueEditResult {
            state: bridget_transport::protocol::QueueEditState::Refused, refusal: Some(ref code), ..
        } if code == "steer_unavailable"),
        "{response:?}"
    );
    assert_eq!(held159_release_phase(&f.shared, &f.delivery_id), "held");
    assert!(f.shared.lock().unwrap().held_ids.contains(&f.message.id));
}

#[test]
fn edit159_promote_pause_humain_admis() {
    let (mut f, _editor) = edit159_promote_fixture(true);
    let (connection, _peers) = edit159_queue_actor(&f, true);
    {
        let st = f.shared.lock().unwrap();
        assert!(
            crate::referent_control::set(
                st.store.connection(),
                crate::referent_control::ControlMutation {
                    command_id: "edit159-promote-pause",
                    expected_generation: 0,
                    paused: Some(true),
                    auto_objectives_cap: None,
                    reason: Some("oracle promote159"),
                    actor: "human",
                    now: unix_now_secs(),
                    agent_posture: None,
                    auto_reassignment: None,
                }
            )
            .unwrap()
            .is_ok()
        );
    }
    let response = edit159_promote_request(&f, connection, "promote159-human-pause");
    assert!(
        matches!(
            response,
            DaemonToWrapper::QueueEditResult {
                state: bridget_transport::protocol::QueueEditState::Applied,
                refusal: None,
                effect: Some(bridget_transport::protocol::QueueEditEffect::Unconfirmed),
                ..
            }
        ),
        "{response:?}"
    );
    assert!(
        matches!(read_control(&mut f._target), DaemonToWrapper::DeliverIdempotent { message, .. }
            if message.id == f.message.id && message.intent == Some(bridget_core::MessageIntent::SteerCurrent))
    );
    assert!(
        crate::referent_control::read(f.shared.lock().unwrap().store.connection())
            .unwrap()
            .paused
    );
}

#[test]
fn edit159_promote_sans_droit_refuse() {
    let (f, _editor) = edit159_promote_fixture(true);
    let (writer, _peer) = control_socket("edit159-promote-bare");
    f.shared
        .lock()
        .unwrap()
        .connections
        .insert("edit159-promote-bare".into(), writer);
    assert!(matches!(
        handle_wrapper_message(
            "edit159-promote-bare",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client,
            },
            &f.shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "edit159-promote-bare",
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: "012_scope_edit159_bare".into(),
                capabilities: vec![ClientCapability::QueueEditV1],
            },
            &f.shared
        ),
        Some(DaemonToWrapper::ClientWelcome { .. })
    ));
    assert_eq!(
        guidance_source(&f.shared.lock().unwrap(), Some("edit159-promote-bare")).0,
        crate::guidance::GuidanceActor::BareClient
    );
    let response = edit159_promote_request(&f, "edit159-promote-bare", "promote159-bare");
    assert!(
        matches!(response, DaemonToWrapper::QueueEditResult {
            state: bridget_transport::protocol::QueueEditState::Refused, refusal: Some(ref code), ..
        } if code.contains("intent_not_permitted") && code.contains("identity_unattested")),
        "{response:?}"
    );
    assert_eq!(held159_release_phase(&f.shared, &f.delivery_id), "held");
}

#[test]
fn edit159_promote_aucun_doublon_cote_daemon() {
    let (mut f, _editor) = edit159_promote_fixture(true);
    let (connection, _peers) = edit159_queue_actor(&f, false);
    let applied = edit159_promote_request(&f, connection, "promote159-once");
    assert!(
        matches!(
            applied,
            DaemonToWrapper::QueueEditResult {
                state: bridget_transport::protocol::QueueEditState::Applied,
                refusal: None,
                effect: Some(bridget_transport::protocol::QueueEditEffect::Unconfirmed),
                ..
            }
        ),
        "{applied:?}"
    );
    assert!(
        matches!(read_control(&mut f._target), DaemonToWrapper::DeliverIdempotent { message, .. }
            if message.id == f.message.id && message.intent == Some(bridget_core::MessageIntent::SteerCurrent))
    );
    let replay = edit159_promote_request(&f, connection, "promote159-once");
    assert_eq!(
        bridget_transport::protocol::encode(&applied).unwrap(),
        bridget_transport::protocol::encode(&replay).unwrap()
    );
    let st = f.shared.lock().unwrap();
    assert_eq!(
        st.store
            .connection()
            .query_row(
                "SELECT COUNT(*) FROM queue_edits WHERE request_id='promote159-once'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    assert!(!st.held_ids.contains(&f.message.id));
    drop(st);
    f._target
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(20)))
        .unwrap();
    let mut line = String::new();
    assert!(
        matches!(f._target.read_line(&mut line), Err(e)
            if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut)),
        "{line}"
    );
    assert!(line.is_empty());
}

// T008 : fixture dédiée, onze usages, primitives149/T004/T005 et suivi mémoire réels.
struct Held159SweepFixture {
    shared: Arc<Mutex<DaemonState>>,
    config: DaemonConfig,
    author: BufReader<UnixStream>,
    _target: BufReader<UnixStream>,
    message: BridgetMessage,
    delivery_id: String,
    started: Instant,
}

fn held159_sweep_fixture() -> Held159SweepFixture {
    let label = format!("hs-{}", &Uuid::new_v4().simple().to_string()[..12]);
    let (shared, config, target) = held159_fixture(&label, Some(true), true, true, 32);
    let (writer, author) = control_socket(&format!("{label}-a"));
    let mut st = shared.lock().unwrap();
    let sender = "89000000-0000-4000-8000-000000000200";
    let from_conn = st.router.get_agent(sender).unwrap().connection_id.clone();
    st.connections.insert(from_conn, writer);
    let owner = st.presences["instance-1"].name.clone();
    let mut message = BridgetMessage::new(sender, &owner, "demande réellement retenue159");
    message.id = "held159-sweep-message".into();
    message.reply = true;
    message.reply_timeout = Some(60);
    let now = unix_now_secs();
    drop(st);
    // Vrai handler et HELLO T005 : le suivi mémoire est installé après COMMIT.
    let response = held159_submit(&shared, message.clone(), &message.id, now);
    let delivery_id = match response {
        DaemonToWrapper::IdempotencyResult {
            issue:
                IdempotencyIssue::OutcomeUnknown {
                    delivery_id: Some(id),
                    ..
                },
            ..
        } => id,
        other => panic!("admission held réelle attendue : {other:?}"),
    };
    assert_eq!(held159_release_phase(&shared, &delivery_id), "held");
    let st = shared.lock().unwrap();
    let started = st
        .pending_replies
        .iter()
        .find(|p| p.msg_id == message.id)
        .unwrap()
        .created_at;
    assert_eq!(
        st.store.get_request(&message.id).unwrap().unwrap().state,
        "open"
    );
    drop(st);
    Held159SweepFixture {
        shared,
        config,
        author,
        _target: target,
        message,
        delivery_id,
        started,
    }
}

// Orchestration des fonctions existantes, jamais une copie de la décision de balayage.
// Le second entretien observe la clôture SQL réelle du palier3.
fn held159_sweep_cycle(f: &Held159SweepFixture, elapsed: u64) -> Vec<ReminderAction> {
    let mut st = f.shared.lock().unwrap();
    maintain_stores(&mut st, unix_now_secs());
    let actions = collect_reminder_actions(&mut st, f.started + Duration::from_secs(elapsed));
    maintain_stores(&mut st, unix_now_secs());
    actions
}

#[test]
fn held159_sweep_enregistrement_expire_retire() {
    let f = held159_sweep_fixture();
    {
        let st = f.shared.lock().unwrap();
        let expired = unix_now_secs() - 1;
        st.store.connection().execute("UPDATE idempotency_records SET expires_at=?1 WHERE issuer_scope='012_scope_held159_oracle' AND idempotency_key=?2",
                rusqlite::params![expired, f.message.id]).unwrap();
        st.store
            .connection()
            .execute(
                "UPDATE send_deliveries SET expires_at=?1 WHERE delivery_id=?2",
                rusqlite::params![expired, f.delivery_id],
            )
            .unwrap();
    }
    let _ = held159_sweep_cycle(&f, 0);
    assert_eq!(
        held159_release_phase(&f.shared, &f.delivery_id),
        "cancelled"
    );
    let st = f.shared.lock().unwrap();
    let result: (String, String, String) = st.store.connection().query_row("SELECT public_result_kind,public_result_category,public_result_reason FROM idempotency_records WHERE issuer_scope='012_scope_held159_oracle' AND idempotency_key=?1",
            [&f.message.id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))).unwrap();
    assert_eq!(result.0, "rejected");
    assert_eq!(result.1, "expired_in_queue");
    assert!(result.2.contains("expired_in_queue"), "{}", result.2);
}

#[test]
fn held159_sweep_demande_close_retire() {
    let f = held159_sweep_fixture();
    f.shared
        .lock()
        .unwrap()
        .store
        .cancel_request(&f.message.id, &f.message.from, Some("clôture159"))
        .unwrap();
    let _ = held159_sweep_cycle(&f, 0);
    assert_eq!(
        held159_release_phase(&f.shared, &f.delivery_id),
        "cancelled"
    );
    let st = f.shared.lock().unwrap();
    let request = st.store.get_request(&f.message.id).unwrap().unwrap();
    assert_eq!(request.state, "cancelled");
    assert_eq!(request.cancel_reason.as_deref(), Some("clôture159"));
    assert!(!st.pending_replies.iter().any(|p| p.msg_id == f.message.id));
}

#[test]
fn held159_sweep_demande_expiree_retire() {
    let f = held159_sweep_fixture();
    f.shared
        .lock()
        .unwrap()
        .store
        .set_request_deadline(&f.message.id, unix_now_secs() - 1)
        .unwrap();
    let _ = held159_sweep_cycle(&f, 0);
    assert_eq!(
        held159_release_phase(&f.shared, &f.delivery_id),
        "cancelled"
    );
    assert_eq!(
        f.shared
            .lock()
            .unwrap()
            .store
            .get_request(&f.message.id)
            .unwrap()
            .unwrap()
            .state,
        "timed_out"
    );
}

#[test]
fn held159_sweep_auteur_prevenu() {
    let mut f = held159_sweep_fixture();
    f.shared
        .lock()
        .unwrap()
        .store
        .cancel_request(&f.message.id, &f.message.from, Some("retrait159"))
        .unwrap();
    let _ = held159_sweep_cycle(&f, 0);
    f.shared
        .lock()
        .unwrap()
        .flush_pending_orphan_emitter_notices();
    f.author
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(250)))
        .unwrap();
    let mut line = String::new();
    f.author
        .read_line(&mut line)
        .expect("avis réel attendu sur la socket primaire auteur");
    let DaemonToWrapper::Deliver { message, .. } = decode(line.trim()).unwrap() else {
        panic!("avis auteur attendu : {line}")
    };
    assert_eq!(message.to, f.message.from);
    assert!(
        message.body.contains(&f.message.id) && message.body.contains("request_closed_in_queue"),
        "{}",
        message.body
    );
    assert!(
        !message.body.contains("ORPHELIN"),
        "phase cancelled : le texte doit dire retrait/rejet"
    );
    assert_eq!(
        held159_release_phase(&f.shared, &f.delivery_id),
        "cancelled"
    );
}

#[test]
fn held159_sweep_retrait_resultat_envoi_rejected_avec_raison() {
    let f = held159_sweep_fixture();
    f.shared
        .lock()
        .unwrap()
        .store
        .cancel_request(&f.message.id, &f.message.from, Some("retrait159"))
        .unwrap();
    let _ = held159_sweep_cycle(&f, 0);
    let key = IdempotencyKey::new(
        "012_scope_held159_oracle",
        OperationKind::Send,
        &f.message.id,
    )
    .unwrap();
    assert!(
        matches!(f.shared.lock().unwrap().idempotency.lookup(&key, unix_now_secs()).unwrap(),
            crate::idempotency::LookupResult::Rejected { category, reason, .. } if category == "request_closed_in_queue" && reason.contains("request_closed_in_queue"))
    );
    assert_eq!(
        held159_release_phase(&f.shared, &f.delivery_id),
        "cancelled"
    );
}

#[test]
fn held159_sweep_parite_avec_les_causes_du_pont() {
    for cause in ["answered", "cancelled", "timed_out", "deadline"] {
        let f = held159_sweep_fixture();
        {
            let mut st = f.shared.lock().unwrap();
            match cause {
                "answered" => {
                    assert!(
                        st.store
                            .mark_answered(
                                &f.message.id,
                                &f.message.to,
                                &f.message.from,
                                Some("réponse acquise159")
                            )
                            .unwrap()
                    );
                }
                "cancelled" => {
                    st.store
                        .cancel_request(&f.message.id, &f.message.from, Some("annulé159"))
                        .unwrap();
                }
                "timed_out" => {
                    assert!(st.store.mark_timed_out(&f.message.id).unwrap());
                }
                "deadline" => {
                    assert!(
                        st.store
                            .set_request_deadline(&f.message.id, unix_now_secs() - 1)
                            .unwrap()
                    );
                }
                _ => unreachable!(),
            }
        }
        let _ = held159_sweep_cycle(&f, 0);
        assert_eq!(
            held159_release_phase(&f.shared, &f.delivery_id),
            "cancelled",
            "cause={cause}"
        );
        let st = f.shared.lock().unwrap();
        let request = st.store.get_request(&f.message.id).unwrap().unwrap();
        assert_eq!(
            request.state,
            if cause == "deadline" {
                "timed_out"
            } else {
                cause
            }
        );
        let result: (String, String) = st.store.connection().query_row("SELECT public_result_category,public_result_reason FROM idempotency_records WHERE issuer_scope='012_scope_held159_oracle' AND idempotency_key=?1",
                [&f.message.id], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!(result.0, "request_closed_in_queue");
        assert!(result.1.contains("request_closed_in_queue"), "{}", result.1);
    }
}

#[test]
fn held159_sweep_retenu_non_expire_conserve() {
    let f = held159_sweep_fixture();
    let before: (Vec<u8>, i64) = f
        .shared
        .lock()
        .unwrap()
        .store
        .connection()
        .query_row(
            "SELECT message_bytes,queue_rank FROM send_deliveries WHERE delivery_id=?1",
            [&f.delivery_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert!(held159_sweep_cycle(&f, 0).is_empty());
    assert_eq!(held159_release_phase(&f.shared, &f.delivery_id), "held");
    let st = f.shared.lock().unwrap();
    assert_eq!(
        st.store
            .connection()
            .query_row(
                "SELECT message_bytes,queue_rank FROM send_deliveries WHERE delivery_id=?1",
                [&f.delivery_id],
                |r| Ok((r.get::<_, Vec<u8>>(0)?, r.get::<_, i64>(1)?))
            )
            .unwrap(),
        before
    );
    assert_eq!(
        st.store.get_request(&f.message.id).unwrap().unwrap().state,
        "open"
    );
    assert_eq!(
        st.store
            .connection()
            .query_row(
                "SELECT count(*) FROM orphan_emitter_notices WHERE delivery_id=?1",
                [&f.delivery_id],
                |r| r.get::<_, i64>(0)
            )
            .unwrap(),
        0
    );
    assert!(st.pending_replies.iter().any(|p| p.msg_id == f.message.id));
}

#[test]
fn held159_sweep_pending_replies_retire_apres_commit() {
    let f = held159_sweep_fixture();
    {
        let mut st = f.shared.lock().unwrap();
        st.store
            .cancel_request(&f.message.id, &f.message.from, Some("clôture159"))
            .unwrap();
        st.store.connection().execute_batch("CREATE TRIGGER held159_refuse_notice BEFORE INSERT ON orphan_emitter_notices BEGIN SELECT RAISE(ABORT,'held159-before-commit'); END;").unwrap();
    }
    let _ = held159_sweep_cycle(&f, 91);
    assert_eq!(held159_release_phase(&f.shared, &f.delivery_id), "held");
    {
        let st = f.shared.lock().unwrap();
        assert!(
            st.pending_replies.iter().any(|p| p.msg_id == f.message.id),
            "pas de retrait mémoire après rollback, même au-delà du délai+grâce"
        );
        assert_eq!(st.store.connection().query_row("SELECT public_result_kind FROM idempotency_records WHERE issuer_scope='012_scope_held159_oracle' AND idempotency_key=?1",
                [&f.message.id], |r| r.get::<_, Option<String>>(0)).unwrap(), None);
        assert_eq!(
            st.store
                .connection()
                .query_row(
                    "SELECT count(*) FROM orphan_emitter_notices WHERE delivery_id=?1",
                    [&f.delivery_id],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
        st.store
            .connection()
            .execute_batch("DROP TRIGGER held159_refuse_notice;")
            .unwrap();
    }
    let _ = held159_sweep_cycle(&f, 91);
    assert_eq!(
        held159_release_phase(&f.shared, &f.delivery_id),
        "cancelled"
    );
    assert!(
        !f.shared
            .lock()
            .unwrap()
            .pending_replies
            .iter()
            .any(|p| p.msg_id == f.message.id)
    );
}

#[test]
fn held159_sweep_aucun_rappel_palier_1_2_pour_une_demande_retenue() {
    let f = held159_sweep_fixture();
    for elapsed in [20, 40] {
        assert!(
            held159_sweep_cycle(&f, elapsed).is_empty(),
            "aucun Gentle/Firm/Deferred pendant held"
        );
        assert_eq!(held159_release_phase(&f.shared, &f.delivery_id), "held");
    }
    let st = f.shared.lock().unwrap();
    assert_eq!(
        st.store
            .get_request(&f.message.id)
            .unwrap()
            .unwrap()
            .escalation_level,
        0
    );
    assert!(
        st.store
            .latest_deferred_reminder(&f.message.id)
            .unwrap()
            .is_none()
    );
}

#[test]
fn held159_sweep_palier_3_ferme_la_demande_et_le_balayage_retire_le_message() {
    let f = held159_sweep_fixture();
    assert!(held159_sweep_cycle(&f, 20).is_empty());
    assert!(held159_sweep_cycle(&f, 40).is_empty());
    let actions = held159_sweep_cycle(&f, 60);
    assert_eq!(
        actions
            .iter()
            .filter(|a| matches!(a, ReminderAction::Timeout { .. }))
            .count(),
        1
    );
    assert_eq!(
        f.shared
            .lock()
            .unwrap()
            .store
            .get_request(&f.message.id)
            .unwrap()
            .unwrap()
            .state,
        "timed_out"
    );
    assert_eq!(
        held159_release_phase(&f.shared, &f.delivery_id),
        "cancelled"
    );
    assert!(
        !f.shared
            .lock()
            .unwrap()
            .pending_replies
            .iter()
            .any(|p| p.msg_id == f.message.id)
    );
    assert!(
        !held159_sweep_cycle(&f, 61)
            .iter()
            .any(|a| matches!(a, ReminderAction::Timeout { .. }))
    );
}

#[test]
fn held159_sweep_held_ids_reconstruit_au_demarrage() {
    let Held159SweepFixture {
        shared,
        config,
        message,
        delivery_id,
        author,
        _target,
        ..
    } = held159_sweep_fixture();
    let mut old = Arc::try_unwrap(shared)
        .ok()
        .expect("fixture sans autre propriétaire")
        .into_inner()
        .unwrap();
    let original_instance = old.instance_id.clone();
    let fixture_root = old.fixture_root.take().unwrap();
    let registry_home = fixture_root.0.clone();
    drop(old); // Fermeture réelle de toutes les connexions SQL avant reconstruction.
    let reopened = {
        let guard = HOME_REGISTRY_LOCK.lock().recover_poison();
        let previous = std::env::var_os("HOME");
        // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
        unsafe { std::env::set_var("HOME", &registry_home) };
        let (tx, _rx) = mpsc::channel();
        let result = DaemonState::new(&config, tx);
        if let Some(home) = previous {
            // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
            unsafe { std::env::set_var("HOME", home) };
        } else {
            // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
            unsafe { std::env::remove_var("HOME") };
        }
        drop(guard);
        result.unwrap()
    };
    let mut st = reopened;
    assert_ne!(st.instance_id, original_instance);
    st.fixture_root = Some(fixture_root);
    let (source_writer, _source_reader) = control_socket("hs-reopen-s");
    let (target_writer, _target_reader) = control_socket("hs-reopen-t");
    st.connections.insert("sweep-source".into(), source_writer);
    st.connections.insert("sweep-target".into(), target_writer);
    for (conn, agent, instance, busy) in [
        (
            "sweep-source",
            message.from.as_str(),
            "sweep-source-instance",
            false,
        ),
        (
            "sweep-target",
            message.to.as_str(),
            "sweep-target-instance",
            true,
        ),
    ] {
        assert!(matches!(
            handle_register_with_channel(
                conn,
                2,
                "claude".into(),
                agent.into(),
                Some("fixture".into()),
                Some("acp".into()),
                ChannelReport::Known("unix".into()),
                Some(PresenceMode::Acp),
                None,
                Some("test".into()),
                Some(instance.into()),
                None,
                busy,
                Some(true),
                &mut st,
                false
            ),
            DaemonToWrapper::Registered { .. }
        ));
    }
    st.restore_pending_for_agent(&message.to, "sweep-target");
    let pending = st
        .pending_replies
        .iter()
        .find(|p| p.msg_id == message.id)
        .expect("suivi SQL réellement rechargé");
    let started = pending.created_at;
    assert!(collect_reminder_actions(&mut st, started + Duration::from_secs(20)).is_empty());
    assert!(collect_reminder_actions(&mut st, started + Duration::from_secs(40)).is_empty());
    assert_eq!(
        st.store.get_request(&message.id).unwrap().unwrap().state,
        "open"
    );
    assert!(
        st.store
            .latest_deferred_reminder(&message.id)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        st.store
            .connection()
            .query_row(
                "SELECT phase FROM send_deliveries WHERE delivery_id=?1",
                [&delivery_id],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "held"
    );
    drop(author);
    drop(_target);
}
use bridget_core::BridgetMessage;
use std::collections::BTreeMap;
use std::ffi::{CString, OsString};
use std::io::Read;
use std::net::Shutdown;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::process::{Child, Command};

const T908_DAEMON_CHILD_ENV: &str = "BRIDGET_T908_DAEMON_CHILD";
const T908_ROOT_ENV: &str = "BRIDGET_T908_ROOT";
const SERVICE_NEGOTIATION_FIXTURE: &str = include_str!(
    "../../../specs/015-guichet-maicie/contracts/fixtures/service-negotiation-v1.jsonl"
);

// HOME est un état de processus : sérialiser les fixtures qui le remplacent.
static HOME_REGISTRY_LOCK: Mutex<()> = Mutex::new(());

pub(super) fn control_socket(
    label: &str,
) -> (Arc<Mutex<BufWriter<UnixStream>>>, BufReader<UnixStream>) {
    let path = std::env::temp_dir().join(format!(
        "bc-{label}-{}.sock",
        &uuid::Uuid::new_v4().simple().to_string()[..12]
    ));
    let listener = UnixListener::bind(&path).unwrap();
    let client = UnixStream::connect(&path).unwrap();
    let (server, _) = listener.accept().unwrap();
    std::fs::remove_file(path).unwrap();
    (
        Arc::new(Mutex::new(BufWriter::new(server))),
        BufReader::new(client),
    )
}

pub(super) fn read_control(reader: &mut BufReader<UnixStream>) -> DaemonToWrapper {
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    decode(line.trim()).unwrap()
}

fn normalized_attach_frame(mut message: DaemonToWrapper) -> String {
    match &mut message {
        DaemonToWrapper::JournalFragment {
            subscription_id, ..
        }
        | DaemonToWrapper::SnapshotCaughtUp {
            subscription_id, ..
        }
        | DaemonToWrapper::Gap {
            subscription_id, ..
        }
        | DaemonToWrapper::JournalReadError {
            subscription_id, ..
        }
        | DaemonToWrapper::End {
            subscription_id, ..
        }
        | DaemonToWrapper::Subscribed { subscription_id }
        | DaemonToWrapper::Subscribe {
            subscription_id, ..
        }
        | DaemonToWrapper::Unsubscribe { subscription_id } => subscription_id.clear(),
        DaemonToWrapper::AttachRejected {
            subscription_id, ..
        } => *subscription_id = None,
        _ => {}
    }
    encode(&message).unwrap()
}

fn install_attach_view(
    state: &mut DaemonState,
    subscription_id: &str,
    attach_conn: &str,
    writer: &Arc<Mutex<BufWriter<UnixStream>>>,
) {
    let view = AttachView::start(
        subscription_id.to_string(),
        writer,
        state.view_closed_tx.clone(),
    )
    .unwrap();
    state.attach_subscriptions.insert(
        subscription_id.to_string(),
        AttachSubscription {
            agent: "89000000-0000-4000-8000-000000000102".to_string(),
            attach_conn: attach_conn.to_string(),
            wrapper_conn: "conn-1".to_string(),
            caught_up: false,
        },
    );
    state.attach_views.insert(subscription_id.to_string(), view);
}

#[test]
fn guichet_frame_limit_counts_the_wire_newline_and_parses_whitespace() {
    let base = r#"{ "type" : "guichet_claim_next", "version" : 1 }"#;
    assert!(raw_guichet_frame(base));
    let exact = format!(
        "{base}{}",
        " ".repeat(MAX_GUICHET_FRAME_BYTES - 1 - base.len())
    );
    let oversized = format!("{exact} ");
    assert_eq!(exact.len() + 1, MAX_GUICHET_FRAME_BYTES);
    assert_eq!(oversized.len() + 1, MAX_GUICHET_FRAME_BYTES + 1);
    // Mutation discriminante : passer de `>` à `>=`, ou retomber sur une
    // sous-chaîne compacte, rejetterait la première trame à tort ou
    // laisserait passer la seconde malgré ses espaces JSON valides.
    assert!(!guichet_frame_exceeds_wire_limit(&exact, exact.len() + 1));
    assert!(guichet_frame_exceeds_wire_limit(&exact, exact.len() + 2)); // CRLF compte aussi.
    assert!(guichet_frame_exceeds_wire_limit(
        &oversized,
        oversized.len() + 1
    ));
}

#[test]
fn unreachable_presence_remains_visible() {
    let base = std::env::temp_dir().join(format!("bridget-presence-{}", std::process::id()));
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
    let mut state = DaemonState::new(&config, managed_tx).unwrap();
    state
        .router
        .register(
            "89000000-0000-4000-8000-000000000107",
            &bridget_core::AgentType::Codex,
            "conn-1",
        )
        .unwrap();
    state
        .conn_instances
        .insert("conn-1".to_string(), "instance-1".to_string());
    state.presences.insert(
        "instance-1".to_string(),
        Presence {
            name: "89000000-0000-4000-8000-000000000107".to_string(),
            agent_type: "codex".to_string(),
            host: "projet-a".to_string(),
            transport: "tmux".to_string(),
            channel: Some("ssh-unix".to_string()),
            mode: Some(PresenceMode::Tmux),
            location: None,
            journal_available: false,
            os: "Linux".to_string(),
            state: "connected".to_string(),
            busy_since: None,
            busy_period_start: None,
            last_attested_at: None,
            turn_attest: false,
            queue_fallback: false,
            capacity_seen: Instant::now(),
            link_seen: Instant::now(),
            reconnect_count: 0,
            model: Some("gpt-5.3-codex".to_string()),
            effort: Some("xhigh".to_string()),
            rate_limits: Default::default(),
            served_model: None,
            derived_domain: Some("projet-a".to_string()),
            domain: Some("projet-a".to_string()),
            dnd_until: None,
            disk_space: None,
        },
    );
    state.router.unregister_by_conn("conn-1");
    state.mark_unreachable("conn-1");

    let agents = state.agent_infos();
    assert_eq!(agents.len(), 1);
    assert_eq!(
        agents
            .iter()
            .map(|agent| agent.agent_id.as_str())
            .collect::<Vec<_>>(),
        vec!["89000000-0000-4000-8000-000000000107"]
    );
    assert_eq!(agents[0].host, "projet-a");
    assert_eq!(agents[0].os, "Linux");
    assert_eq!(agents[0].state, "unreachable");
    // FR-010 : la dernière capacité connue survit à la perte de connexion.
    assert_eq!(agents[0].model.as_deref(), Some("gpt-5.3-codex"));
    assert_eq!(agents[0].effort.as_deref(), Some("xhigh"));
    if let Err(e) = std::fs::remove_file(&config.db_path) {
        log::warn!(
            "Impossible de supprimer la base {}: {}",
            config.db_path.display(),
            e
        );
    }
}

#[test]
fn sonde_build_id_partage_la_derivation_de_portee_client() {
    let expected = crate::communication::issuer_scope(BUILD_ID_PROBE_IDENTITY);
    assert_eq!(build_id_probe_issuer_scope(), expected);
    assert!(crate::idempotency::validate_issuer_scope(&expected).is_ok());
}

/// Construit un état minimal avec un agent enregistré et sa présence.
// T007 : composition des fixtures existantes, 19 consommateurs réels.
fn held159_release_fixture(
    label: &str,
) -> (Arc<Mutex<DaemonState>>, DaemonConfig, BufReader<UnixStream>) {
    let compact = format!("h159-{}", &uuid::Uuid::new_v4().simple().to_string()[..12]);
    let (mut st, config) = state_with_registered_agent(&compact);
    // Registre propre à cette fixture en RAM ; HOME n'est pas une autorité de Namespace.
    st.registry = AgentRegistry::from_json(
            r#"{"agents":{"claude":{"command":"/bin/sh","protocol":"acp","forbidden_env":[],"pass_env":[],"held_queue":true}}}"#,
            config.db_path.with_extension("agents.json"),
        ).unwrap();
    let (writer, reader) = control_socket(&compact);
    let _ = label; // Le nom d'oracle ne devient pas un long chemin socket Unix.
    st.connections.insert("conn-1".into(), writer);
    let p = st.presences.get_mut("instance-1").unwrap();
    p.turn_attest = true;
    p.last_attested_at = Some(Instant::now());
    p.busy_period_start = None;
    (Arc::new(Mutex::new(st)), config, reader)
}

// Dépôt réel Held dans la saga, sans libération simulée ni SQL de phase.
fn held159_release_deposit(shared: &Arc<Mutex<DaemonState>>, id: &str, reply: bool) {
    let mut st = shared.lock().unwrap();
    let owner = st.presences["instance-1"].name.clone();
    let mut message = bridget_core::BridgetMessage::new("peer-a", &owner, id);
    message.id = id.into();
    message.reply = reply;
    let key = IdempotencyKey::new("012_scope_held159_release", OperationKind::Send, id).unwrap();
    let now = unix_now_secs();
    let bytes = serde_json::to_vec(&message).unwrap();
    st.idempotency
        .reserve(
            &key,
            &bytes,
            now,
            CLIENT_IDEMPOTENCY_HORIZON_SECS,
            now,
            CLIENT_ISSUED_AT_TOLERANCE_SECS,
        )
        .unwrap();
    let delivery = SendDelivery {
        delivery_id: id.into(),
        recipient_instance_id: "instance-1".into(),
        delivery_generation: 1,
        expires_at: now + CLIENT_IDEMPOTENCY_HORIZON_SECS,
        message_bytes: bytes,
    };
    let tracking = crate::idempotency::ReplyTracking {
        request_id: id.into(),
        sender: "peer-a".into(),
        target: owner.clone(),
        created_at: now,
        deadline_at: now + 600,
    };
    st.idempotency
        .begin_send_delivery_with_guidance(
            &key,
            &delivery,
            reply.then_some(&tracking),
            None,
            None,
            None,
            Some(&owner),
        )
        .unwrap();
}

fn held159_release_phase(shared: &Arc<Mutex<DaemonState>>, id: &str) -> String {
    shared
        .lock()
        .unwrap()
        .store
        .connection()
        .query_row(
            "SELECT phase FROM send_deliveries WHERE delivery_id=?1",
            [id],
            |r| r.get(0),
        )
        .unwrap()
}

// Même drain hors State que la boucle de connexion, pas de hook métier.
fn held159_release_event(shared: &Arc<Mutex<DaemonState>>, msg: WrapperToDaemon) {
    assert!(handle_wrapper_message("conn-1", msg, shared).is_none());
    let controls = shared
        .lock()
        .unwrap()
        .pending_post_response_controls
        .remove("conn-1")
        .unwrap_or_default();
    assert!(execute_controls(controls).is_empty());
}

#[test]
fn held159_release_repos_atteste_libere_le_plus_ancien() {
    let (shared, _config, mut reader) =
        held159_release_fixture("held159_release_repos_atteste_libere_le_plus_ancien");
    held159_release_deposit(&shared, "oldest", false);
    held159_release_deposit(&shared, "next", false);
    held159_release_event(&shared, WrapperToDaemon::TurnState { in_progress: false });
    assert_eq!(held159_release_phase(&shared, "oldest"), "dispatching");
    assert_eq!(held159_release_phase(&shared, "next"), "held");
    assert!(
        matches!(read_control(&mut reader),DaemonToWrapper::DeliverIdempotent{delivery_id,..} if delivery_id=="oldest")
    );
}

#[test]
fn held159_release_un_seul_par_cycle_de_tour() {
    let (shared, _config, _reader) =
        held159_release_fixture("held159_release_un_seul_par_cycle_de_tour");
    held159_release_deposit(&shared, "first", false);
    held159_release_deposit(&shared, "second", false);
    held159_release_event(&shared, WrapperToDaemon::TurnState { in_progress: false });
    assert_eq!(held159_release_phase(&shared, "first"), "dispatching");
    held159_release_event(
        &shared,
        WrapperToDaemon::DeliverAcked {
            delivery_id: "first".into(),
            delivery_generation: 1,
        },
    );
    held159_release_event(&shared, WrapperToDaemon::TurnState { in_progress: false });
    assert_eq!(held159_release_phase(&shared, "second"), "held");
    held159_release_event(&shared, WrapperToDaemon::TurnState { in_progress: true });
    held159_release_event(&shared, WrapperToDaemon::TurnState { in_progress: false });
    assert_eq!(held159_release_phase(&shared, "second"), "dispatching");
}

#[test]
fn held159_release_pas_de_liberation_si_busy() {
    let (shared, _config, _reader) =
        held159_release_fixture("held159_release_pas_de_liberation_si_busy");
    held159_release_deposit(&shared, "busy-held", false);
    held159_release_event(&shared, WrapperToDaemon::TurnState { in_progress: true });
    let _ = collect_reminder_actions(&mut shared.lock().unwrap(), Instant::now());
    assert_eq!(held159_release_phase(&shared, "busy-held"), "held");
}

#[test]
fn held159_release_refus_de_guidage_ne_libere_pas() {
    let (shared, _config, _reader) =
        held159_release_fixture("held159_release_refus_de_guidage_ne_libere_pas");
    held159_release_deposit(&shared, "rejected-steer-neighbor", false);
    // Sortie véridique du refus en tour actif, couverte côté wrapper par T006.
    held159_release_event(&shared, WrapperToDaemon::TurnState { in_progress: true });
    let _ = collect_reminder_actions(&mut shared.lock().unwrap(), Instant::now());
    assert_eq!(shared.lock().unwrap().presences["instance-1"].state, "busy");
    assert_eq!(
        held159_release_phase(&shared, "rejected-steer-neighbor"),
        "held"
    );
}

#[test]
fn held159_release_ack_puis_faux_signal_terminal_ne_libere_pas() {
    let (shared, _config, _reader) =
        held159_release_fixture("held159_release_ack_puis_faux_signal_terminal_ne_libere_pas");
    held159_release_deposit(&shared, "acked", false);
    held159_release_deposit(&shared, "neighbor", false);
    held159_release_event(&shared, WrapperToDaemon::TurnState { in_progress: false });
    assert_eq!(held159_release_phase(&shared, "acked"), "dispatching");
    held159_release_event(
        &shared,
        WrapperToDaemon::DeliverAcked {
            delivery_id: "acked".into(),
            delivery_generation: 1,
        },
    );
    held159_release_event(&shared, WrapperToDaemon::TurnState { in_progress: false });
    assert_eq!(held159_release_phase(&shared, "neighbor"), "held");
}

#[test]
fn held159_release_declencheurs_concurrents_une_seule_liberation() {
    let (shared, _config, _reader) =
        held159_release_fixture("held159_release_declencheurs_concurrents_une_seule_liberation");
    held159_release_deposit(&shared, "race-first", false);
    held159_release_deposit(&shared, "race-second", false);
    let barrier = Arc::new(std::sync::Barrier::new(3));
    let joins: Vec<_> = (0..2)
        .map(|_| {
            let s = shared.clone();
            let b = barrier.clone();
            std::thread::spawn(move || {
                b.wait();
                assert!(
                    handle_wrapper_message(
                        "conn-1",
                        WrapperToDaemon::TurnState { in_progress: false },
                        &s
                    )
                    .is_none()
                );
            })
        })
        .collect();
    barrier.wait();
    for j in joins {
        j.join().unwrap();
    }
    let n: i64 = shared
        .lock()
        .unwrap()
        .store
        .connection()
        .query_row(
            "SELECT count(*) FROM send_deliveries WHERE phase='dispatching' AND turn_gate=1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(n, 1);
    assert_eq!(held159_release_phase(&shared, "race-second"), "held");
}

#[test]
fn held159_release_reservation_levee_par_true() {
    let (shared, _config, _reader) =
        held159_release_fixture("held159_release_reservation_levee_par_true");
    held159_release_deposit(&shared, "gate-true", false);
    held159_release_deposit(&shared, "next-true", false);
    held159_release_event(&shared, WrapperToDaemon::TurnState { in_progress: false });
    assert_eq!(held159_release_phase(&shared, "gate-true"), "dispatching");
    held159_release_event(&shared, WrapperToDaemon::TurnState { in_progress: true });
    let gate: i64 = shared
        .lock()
        .unwrap()
        .store
        .connection()
        .query_row(
            "SELECT turn_gate FROM send_deliveries WHERE delivery_id='gate-true'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(gate, 0);
    assert_eq!(held159_release_phase(&shared, "next-true"), "held");
}

#[test]
fn held159_release_reservation_levee_par_remise_indeterminee() {
    let (shared, _config, _reader) =
        held159_release_fixture("held159_release_reservation_levee_par_remise_indeterminee");
    held159_release_deposit(&shared, "uncertain", false);
    held159_release_deposit(&shared, "after-uncertain", false);
    held159_release_event(&shared, WrapperToDaemon::TurnState { in_progress: false });
    assert_eq!(held159_release_phase(&shared, "uncertain"), "dispatching");
    held159_release_event(
        &shared,
        WrapperToDaemon::DeliveryIndeterminate {
            delivery_id: "uncertain".into(),
            delivery_generation: 1,
        },
    );
    assert_eq!(held159_release_phase(&shared, "uncertain"), "indeterminate");
    let gate: i64 = shared
        .lock()
        .unwrap()
        .store
        .connection()
        .query_row(
            "SELECT turn_gate FROM send_deliveries WHERE delivery_id='uncertain'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(gate, 0);
}

#[test]
fn held159_release_reservation_survit_au_redemarrage_du_daemon() {
    let (shared, config, _reader) = held159_release_fixture("held159-release-restart");
    held159_release_deposit(&shared, "restart-gate", false);
    held159_release_deposit(&shared, "restart-next", false);
    held159_release_event(&shared, WrapperToDaemon::TurnState { in_progress: false });
    assert_eq!(
        held159_release_phase(&shared, "restart-gate"),
        "dispatching"
    );
    let registry = shared.lock().unwrap().registry.clone();
    let root = shared.lock().unwrap().fixture_root.take();
    drop(shared);
    let (tx, _rx) = mpsc::channel();
    let mut fresh = DaemonState::new(&config, tx).unwrap();
    // Le daemon est reconstruit ; seul le registre privé de fixture est réinstallé en RAM.
    fresh.registry = registry;
    fresh.fixture_root = root;
    assert!(fresh.presences.is_empty());
    let (writer, _reader) = control_socket("held159-release-restarted");
    fresh.connections.insert("conn-1".into(), writer);
    let response = handle_register_with_channel(
        "conn-1",
        2,
        "claude".into(),
        "89000000-0000-4000-8000-000000000102".into(),
        Some("macbook".into()),
        Some("acp".into()),
        ChannelReport::Known("unix".into()),
        Some(PresenceMode::Acp),
        None,
        Some("macOS".into()),
        Some("instance-1".into()),
        None,
        false,
        Some(true),
        &mut fresh,
        true,
    );
    assert!(matches!(response, DaemonToWrapper::Registered { .. }));
    let restarted = Arc::new(Mutex::new(fresh));
    held159_release_event(
        &restarted,
        WrapperToDaemon::TurnState { in_progress: false },
    );
    assert_eq!(held159_release_phase(&restarted, "restart-next"), "held");
    let gate: i64 = restarted
        .lock()
        .unwrap()
        .store
        .connection()
        .query_row(
            "SELECT turn_gate FROM send_deliveries WHERE delivery_id='restart-gate'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(gate, 1);
}

#[test]
fn held159_release_reconnexion_libere() {
    let (shared, _config, _reader) = held159_release_fixture("held159_release_reconnexion_libere");
    held159_release_deposit(&shared, "register-held", false);
    let owner = shared.lock().unwrap().presences["instance-1"].name.clone();
    // Même demi-EOF réel que fermeture_ancienne_connexion_n_ecrase_pas_busy_de_la_reinscription.
    // Le routeur libère l'UUID ; l'ancienne carte instance reste jusqu'au takeover du Register.
    assert!(
        shared
            .lock()
            .unwrap()
            .router
            .unregister_by_conn("conn-1")
            .is_some()
    );
    let (writer, _reconnected_reader) = control_socket("held159-reconnected");
    shared
        .lock()
        .unwrap()
        .connections
        .insert("conn-2".into(), writer);
    let response = handle_register_with_channel(
        "conn-2",
        2,
        "claude".into(),
        owner,
        Some("macbook".into()),
        Some("acp".into()),
        ChannelReport::Known("unix".into()),
        Some(PresenceMode::Acp),
        None,
        Some("macOS".into()),
        Some("instance-1".into()),
        None,
        false,
        Some(true),
        &mut shared.lock().unwrap(),
        true,
    );
    assert!(
        matches!(response, DaemonToWrapper::Registered { .. }),
        "réponse Register de reconnexion : {response:?}"
    );
    assert_eq!(
        held159_release_phase(&shared, "register-held"),
        "dispatching"
    );
}

#[test]
fn held159_release_fin_d_execution_libere() {
    let (shared, _config, _reader) =
        held159_release_fixture("held159_release_fin_d_execution_libere");
    held159_release_deposit(&shared, "execution-held", false);
    let owner = shared.lock().unwrap().presences["instance-1"].name.clone();
    shared
        .lock()
        .unwrap()
        .execution_store
        .record_starting("held-submission", "held-execution", &owner, unix_now_secs())
        .unwrap();
    held159_release_event(
        &shared,
        WrapperToDaemon::ExecutionStateChanged {
            transition: bridget_transport::protocol::ExecutionStateTransition {
                execution_id: "held-execution".into(),
                generation: 1,
                expected_state: "starting".into(),
                expected_revision: 0,
                next_state: "failed".into(),
                reason: "provider_failed".into(),
                observed_at: unix_now_secs(),
            },
        },
    );
    assert_eq!(
        held159_release_phase(&shared, "execution-held"),
        "dispatching"
    );
}

#[test]
fn held159_release_balayage_libere_si_repos_reattesté() {
    let (shared, _config, _reader) =
        held159_release_fixture("held159_release_balayage_libere_si_repos_reattesté");
    held159_release_deposit(&shared, "sweep-idle", false);
    let _ = collect_reminder_actions(&mut shared.lock().unwrap(), Instant::now());
    assert_eq!(held159_release_phase(&shared, "sweep-idle"), "dispatching");
}

#[test]
fn held159_release_wrapper_muet_attend_la_sortie_forcee() {
    let (shared, _config, _reader) =
        held159_release_fixture("held159_release_wrapper_muet_attend_la_sortie_forcee");
    held159_release_deposit(&shared, "mute-held", false);
    let now = Instant::now();
    let timeout =
        live_notify_timeout_secs(&shared.lock().unwrap().registry, "claude") + TIMEOUT_GRACE_PERIOD;
    {
        let mut st = shared.lock().unwrap();
        let agent = st.presences["instance-1"].name.clone();
        let p = st.presences.get_mut("instance-1").unwrap();
        // Une expiration garde busy_period_start : true frais n'est pas un repos.
        p.state = "connected".into();
        p.busy_period_start = Some(now - Duration::from_secs(timeout));
        p.last_attested_at = Some(now);
        assert!(!schedule_held_release(&mut st, &agent, now));
        let p = st.presences.get_mut("instance-1").unwrap();
        p.state = "busy".into();
        p.busy_period_start = Some(now - Duration::from_secs(timeout));
        p.last_attested_at = Some(now - Duration::from_secs(91));
    }
    let _ = collect_reminder_actions(&mut shared.lock().unwrap(), now);
    assert_eq!(held159_release_phase(&shared, "mute-held"), "held");
    {
        let mut st = shared.lock().unwrap();
        st.release_stale_busy_turns_at(now + Duration::from_secs(1));
    }
    let _ = collect_reminder_actions(&mut shared.lock().unwrap(), now + Duration::from_secs(1));
    assert_eq!(held159_release_phase(&shared, "mute-held"), "dispatching");
    assert_eq!(
        shared.lock().unwrap().presences["instance-1"].last_attested_at,
        Some(now - Duration::from_secs(91))
    );
}

#[test]
fn held159_release_echeance_de_tour_posee_a_la_liberation() {
    let (shared, _config, mut reader) =
        held159_release_fixture("held159_release_echeance_de_tour_posee_a_la_liberation");
    held159_release_deposit(&shared, "release-deadline", false);
    let before = unix_now_secs() as u64;
    held159_release_event(&shared, WrapperToDaemon::TurnState { in_progress: false });
    assert_eq!(
        held159_release_phase(&shared, "release-deadline"),
        "dispatching"
    );
    let DaemonToWrapper::DeliverIdempotent { message, .. } = read_control(&mut reader) else {
        panic!("remise attendue")
    };
    let timeout = live_notify_timeout_secs(&shared.lock().unwrap().registry, "claude");
    assert!(
        (before + timeout..=unix_now_secs() as u64 + timeout)
            .contains(&message.deadline_at.unwrap())
    );
    let bytes: Vec<u8> = shared
        .lock()
        .unwrap()
        .store
        .connection()
        .query_row(
            "SELECT message_bytes FROM send_deliveries WHERE delivery_id='release-deadline'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<bridget_core::BridgetMessage>(&bytes)
            .unwrap()
            .deadline_at,
        None
    );
}

#[test]
fn held159_release_demande_close_juste_avant_ne_libere_pas_et_retire() {
    let (shared, _config, _reader) = held159_release_fixture(
        "held159_release_demande_close_juste_avant_ne_libere_pas_et_retire",
    );
    held159_release_deposit(&shared, "closed-request", true);
    shared
        .lock()
        .unwrap()
        .store
        .connection()
        .execute(
            "UPDATE tracked_requests SET state='answered' WHERE id='closed-request'",
            [],
        )
        .unwrap();
    held159_release_event(&shared, WrapperToDaemon::TurnState { in_progress: false });
    assert_eq!(
        held159_release_phase(&shared, "closed-request"),
        "cancelled"
    );
}

#[test]
fn held159_release_echeance_juste_avant_ne_libere_pas() {
    let (shared, _config, _reader) =
        held159_release_fixture("held159_release_echeance_juste_avant_ne_libere_pas");
    held159_release_deposit(&shared, "expired-request", true);
    shared
        .lock()
        .unwrap()
        .store
        .connection()
        .execute(
            "UPDATE tracked_requests SET deadline_at=?1 WHERE id='expired-request'",
            [unix_now_secs() - 1],
        )
        .unwrap();
    held159_release_event(&shared, WrapperToDaemon::TurnState { in_progress: false });
    assert_eq!(
        held159_release_phase(&shared, "expired-request"),
        "cancelled"
    );
}

#[test]
fn held159_release_pause_ne_libere_pas() {
    let (shared, _config, _reader) = held159_release_fixture("held159_release_pause_ne_libere_pas");
    held159_release_deposit(&shared, "paused-held", false);
    let st = shared.lock().unwrap();
    let generation = crate::referent_control::read(st.store.connection())
        .unwrap()
        .generation;
    crate::referent_control::set(
        st.store.connection(),
        crate::referent_control::ControlMutation {
            command_id: "held-pause",
            expected_generation: generation,
            paused: Some(true),
            auto_objectives_cap: None,
            reason: None,
            actor: "humain",
            now: unix_now_secs(),
            agent_posture: None,
            auto_reassignment: None,
        },
    )
    .unwrap()
    .unwrap();
    drop(st);
    held159_release_event(&shared, WrapperToDaemon::TurnState { in_progress: false });
    assert_eq!(held159_release_phase(&shared, "paused-held"), "held");
}

#[test]
fn held159_release_pause_interruption_fin_de_tour_aucune_liberation() {
    let (shared, _config, _reader) =
        held159_release_fixture("held159_release_pause_interruption_fin_de_tour_aucune_liberation");
    held159_release_deposit(&shared, "pause-pending-held", false);
    let owner = shared.lock().unwrap().presences["instance-1"].name.clone();
    {
        let st = shared.lock().unwrap();
        st.execution_store
            .record_starting(
                "pending-submission",
                "pending-execution",
                &owner,
                unix_now_secs(),
            )
            .unwrap();
        st.execution_store
            .record_pause_interruption("pending-execution", &owner, 1, unix_now_secs())
            .unwrap();
    }
    held159_release_event(
        &shared,
        WrapperToDaemon::ExecutionStateChanged {
            transition: bridget_transport::protocol::ExecutionStateTransition {
                execution_id: "pending-execution".into(),
                generation: 1,
                expected_state: "starting".into(),
                expected_revision: 0,
                next_state: "failed".into(),
                reason: "provider_failed".into(),
                observed_at: unix_now_secs(),
            },
        },
    );
    assert_eq!(held159_release_phase(&shared, "pause-pending-held"), "held");
}

#[test]
fn held159_release_levee_de_pause_reprise_d_abord_puis_liberation() {
    let (shared, _config, mut reader) = held159_release_fixture("held159-release-resume");
    held159_release_deposit(&shared, "pause-resume-neighbor", false);
    let owner = shared.lock().unwrap().presences["instance-1"].name.clone();
    {
        let mut st = shared.lock().unwrap();
        spec_087_register_human_control_client(&mut st);
        let mut message =
            bridget_core::BridgetMessage::new("humain", &owner, "reprise prioritaire");
        message.origin = Some(bridget_core::MessageOrigin::Human);
        message.intent = Some(bridget_core::MessageIntent::TriggerTurn);
        st.execution_store
            .admit_starting_message(&message, "paused-execution", unix_now_secs())
            .unwrap();
        st.execution_store
            .transition_if_current(
                "paused-execution",
                "starting",
                0,
                1,
                "interrupted",
                "control_pause",
                unix_now_secs(),
            )
            .unwrap();
        st.execution_store
            .record_pause_interruption("paused-execution", &owner, 2, unix_now_secs())
            .unwrap();
    }
    assert!(matches!(
        handle_control_state_set(
            &shared,
            "ui-control",
            1,
            "held-pause",
            0,
            Some(true),
            None,
            Some("test"),
            None,
            None
        ),
        DaemonToWrapper::ControlState { .. }
    ));
    held159_release_event(&shared, WrapperToDaemon::TurnState { in_progress: false });
    assert_eq!(
        held159_release_phase(&shared, "pause-resume-neighbor"),
        "held"
    );
    assert!(matches!(
        handle_control_state_set(
            &shared,
            "ui-control",
            1,
            "held-resume",
            1,
            Some(false),
            None,
            Some("test"),
            None,
            None
        ),
        DaemonToWrapper::ControlState { .. }
    ));
    let DaemonToWrapper::DeliverIdempotent {
        execution, message, ..
    } = read_control(&mut reader)
    else {
        panic!("reprise attendue")
    };
    assert!(execution.is_some());
    assert_eq!(message.body, "reprise prioritaire");
    // La première remise est la reprise ; la libération suit le vrai cycle du porteur.
    held159_release_event(&shared, WrapperToDaemon::TurnState { in_progress: true });
    held159_release_event(&shared, WrapperToDaemon::TurnState { in_progress: false });
    assert_eq!(
        held159_release_phase(&shared, "pause-resume-neighbor"),
        "dispatching"
    );
    assert!(
        matches!(read_control(&mut reader),DaemonToWrapper::DeliverIdempotent{delivery_id,execution:None,..}if delivery_id=="pause-resume-neighbor")
    );
}

#[test]
fn held159_release_reservation_levee_apres_ack_et_30s_sans_busy() {
    for old in [false, true] {
        let label = if old {
            "held159-old-ack"
        } else {
            "held159-young-ack"
        };
        let (shared, _config, mut reader) = held159_release_fixture(label);
        held159_release_deposit(&shared, "ack-reservation", false);
        held159_release_deposit(&shared, "ack-neighbor", false);
        held159_release_event(&shared, WrapperToDaemon::TurnState { in_progress: false });
        assert_eq!(
            held159_release_phase(&shared, "ack-reservation"),
            "dispatching"
        );
        assert!(
            matches!(read_control(&mut reader),DaemonToWrapper::DeliverIdempotent{delivery_id,..}if delivery_id=="ack-reservation")
        );
        let now = unix_now_secs();
        let first = now - if old { 31 } else { 10 };
        shared
            .lock()
            .unwrap()
            .idempotency
            .acknowledge_send_delivery_at("ack-reservation", "instance-1", 1, first)
            .unwrap();
        let at: Option<i64> = shared
            .lock()
            .unwrap()
            .store
            .connection()
            .query_row(
                "SELECT acked_at FROM send_deliveries WHERE delivery_id='ack-reservation'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(at, Some(first));
        shared
            .lock()
            .unwrap()
            .idempotency
            .acknowledge_send_delivery_at("ack-reservation", "instance-1", 1, now)
            .unwrap();
        let after: Option<i64> = shared
            .lock()
            .unwrap()
            .store
            .connection()
            .query_row(
                "SELECT acked_at FROM send_deliveries WHERE delivery_id='ack-reservation'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(after, at);
        let _ = collect_reminder_actions(&mut shared.lock().unwrap(), Instant::now());
        assert_eq!(
            held159_release_phase(&shared, "ack-neighbor"),
            if old { "dispatching" } else { "held" }
        );
        let controls = shared
            .lock()
            .unwrap()
            .pending_post_response_controls
            .remove("conn-1")
            .unwrap_or_default();
        assert!(execute_controls(controls).is_empty());
        if old {
            assert!(
                matches!(read_control(&mut reader),DaemonToWrapper::DeliverIdempotent{delivery_id,..}if delivery_id=="ack-neighbor")
            );
        }
    }
}

pub(super) fn state_with_registered_agent(_label: &str) -> (DaemonState, DaemonConfig) {
    state_with_registered_agent_configured(_label, |_| {})
}

pub(super) fn state_with_registered_agent_configured(
    _label: &str,
    configure: impl FnOnce(&mut DaemonConfig),
) -> (DaemonState, DaemonConfig) {
    let base = std::env::temp_dir().join(format!(
        "brt-{}",
        &uuid::Uuid::new_v4().simple().to_string()[..12]
    ));
    // Résoudre les liens du dossier temporaire, comme les chemins enregistrés.
    std::fs::create_dir_all(&base).unwrap();
    let base = base.canonicalize().unwrap();
    let registry_home = base.join("home");
    let fixture_root = FixtureRoot(registry_home.clone());
    std::fs::create_dir_all(registry_home.join(".config/bridget")).unwrap();
    let registry_file = registry_home.join(".config/bridget/agents.json");
    std::fs::write(
        &registry_file,
        serde_json::json!({
            "agents": {
                "claude": {
                    "command": "/bin/sh",
                    "protocol": "acp",
                    "forbidden_env": [],
                    "pass_env": []
                }
            }
        })
        .to_string(),
    )
    .unwrap();
    std::fs::set_permissions(&registry_file, std::fs::Permissions::from_mode(0o600)).unwrap();
    let mut config = DaemonConfig {
        socket_path: base.with_extension("sock"),
        db_path: base.with_extension("db"),
        log_path: base.with_extension("log"),
        circuit_breaker_window: 180,
        circuit_breaker_limit: 8,
        dedup_window: 180,
        quarantine_window: 3600,
        retention_days: 7,
    };
    configure(&mut config);
    // Un test qui panique en tenant ce verrou ne doit pas empoisonner les
    // suivants : on relit le verrou empoisonné plutôt que de propager.
    let _home_lock = HOME_REGISTRY_LOCK.lock().recover_poison();
    let (managed_tx, _managed_rx) = mpsc::channel();
    let previous_home = std::env::var_os("HOME");
    // DaemonState::new charge le registre via HOME ; la fixture doit lui
    // fournir une racine locale avant toute construction d'état.
    // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
    unsafe { std::env::set_var("HOME", &registry_home) };
    let state_result = DaemonState::new(&config, managed_tx);
    if let Some(home) = previous_home {
        // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
        unsafe { std::env::set_var("HOME", home) };
    } else {
        // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
        unsafe { std::env::remove_var("HOME") };
    }
    let mut state = state_result.unwrap();
    state.fixture_root = Some(fixture_root);
    assert!(matches!(
        handle_register_with_channel(
            "conn-1",
            2,
            "claude".into(),
            "89000000-0000-4000-8000-000000000102".into(),
            Some("macbook".into()),
            Some("acp".into()),
            ChannelReport::Known("unix".into()),
            Some(PresenceMode::Acp),
            None,
            Some("macOS".into()),
            Some("instance-1".into()),
            None,
            false,
            Some(true),
            &mut state,
            false,
        ),
        DaemonToWrapper::Registered {
            credential: Some(_),
            ..
        }
    ));
    state
        .conn_instances
        .insert("conn-1".to_string(), "instance-1".to_string());
    state.presences.insert(
        "instance-1".to_string(),
        Presence {
            name: "89000000-0000-4000-8000-000000000102".to_string(),
            agent_type: "claude".to_string(),
            host: "macbook".to_string(),
            transport: "acp".to_string(),
            channel: Some("unix".to_string()),
            mode: Some(PresenceMode::Acp),
            location: None,
            journal_available: true,
            os: "macOS".to_string(),
            state: "connected".to_string(),
            busy_since: None,
            busy_period_start: None,
            last_attested_at: None,
            turn_attest: false,
            queue_fallback: false,
            capacity_seen: Instant::now(),
            link_seen: Instant::now(),
            reconnect_count: 0,
            model: None,
            effort: None,
            rate_limits: Default::default(),
            served_model: None,
            derived_domain: None,
            domain: None,
            dnd_until: None,
            disk_space: None,
        },
    );
    (state, config)
}

#[test]
fn tracking230_echeance_dormante_close_sans_rattrapage() {
    use crate::wake_schedules::WakePhase;
    use bridget_transport::protocol::{WakeAction, WakeCadence, WakeRequest, WakeTarget};
    let (mut st, _config) = state_with_registered_agent("tracking230-calendar");
    let owner = st.presences["instance-1"].name.clone();
    let (writer, mut reader) = control_socket("tracking230-calendar");
    st.connections.insert("conn-1".into(), writer);
    let state = Arc::new(Mutex::new(st));
    let now = unix_now_secs();
    let clock = Instant::now();
    handle_wake_request(
        "conn-1",
        WakeRequest {
            version: 1,
            request: WakeAction::Schedule {
                target: WakeTarget::SelfTarget {},
                cadence: WakeCadence::Interval { seconds: 60 },
                label: None,
            },
        },
        &state,
        now,
    );
    let id = state
        .lock()
        .unwrap()
        .store
        .list_wake_schedules(Some(&owner))
        .unwrap()[0]
        .schedule_id
        .clone();
    {
        let mut st = state.lock().unwrap();
        let agent = agent_tracking::directory(&mut st)
            .unwrap()
            .into_iter()
            .find(|a| a.agent.agent_id == owner)
            .unwrap();
        crate::agent_tracking::set(
            st.store.connection(),
            "human",
            &Uuid::new_v4().to_string(),
            b"sleep",
            &agent,
            true,
            Some(false),
            None,
            now,
        )
        .unwrap();
    }
    scheduled_wakes_tick(&state, now + 60, clock + Duration::from_secs(60));
    {
        let mut st = state.lock().unwrap();
        let schedule = st.store.wake_schedule(&id).unwrap().unwrap();
        assert_eq!(
            schedule.occurrence.unwrap().phase,
            WakePhase::MissedBeforeDeparture
        );
        assert_eq!(schedule.next_at, now + 120);
        let agent = agent_tracking::directory(&mut st)
            .unwrap()
            .into_iter()
            .find(|a| a.agent.agent_id == owner)
            .unwrap();
        crate::agent_tracking::set(
            st.store.connection(),
            "human",
            &Uuid::new_v4().to_string(),
            b"wake",
            &agent,
            false,
            Some(false),
            None,
            now + 61,
        )
        .unwrap();
    }
    scheduled_wakes_tick(&state, now + 61, clock + Duration::from_secs(61));
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    assert!(
        reader.read_line(&mut String::new()).is_err(),
        "pas de rattrapage après réveil"
    );
    let st = state.lock().unwrap();
    let schedule = st.store.wake_schedule(&id).unwrap().unwrap();
    assert_eq!(
        schedule.next_at,
        now + 120,
        "la prochaine échéance reste programmée"
    );
}

#[test]
fn wake158b_daemon_g4_g5_pause_et_grace_avant_depart() {
    use crate::wake_schedules::WakePhase;
    use bridget_transport::protocol::{WakeAction, WakeCadence, WakeRequest, WakeTarget};
    for mode in [
        "pause",
        "offline",
        "dnd",
        "capacite",
        "circuit",
        "depart_unknown",
        "sql099",
        "expiration_writer",
    ] {
        let (mut st, config) = state_with_registered_agent(&format!("w158b-{mode}"));
        let owner = st.presences["instance-1"].name.clone();
        let (writer, mut reader) = control_socket(&format!("w158b-{mode}"));
        st.connections.insert("conn-1".into(), writer.clone());
        let state = Arc::new(Mutex::new(st));
        let at = unix_now_secs();
        let clock = Instant::now();
        let response = handle_wake_request(
            "conn-1",
            WakeRequest {
                version: 1,
                request: WakeAction::Schedule {
                    target: WakeTarget::SelfTarget {},
                    cadence: WakeCadence::Interval { seconds: 60 },
                    label: Some("libellé complet158B".into()),
                },
            },
            &state,
            at,
        );
        assert!(
            matches!(response, DaemonToWrapper::WakeResult { .. }),
            "{response:?}"
        );
        let initial = state
            .lock()
            .unwrap()
            .store
            .list_wake_schedules(Some(&owner))
            .unwrap();
        assert_eq!(
            initial.len(),
            1,
            "le handler admet réellement la programmation"
        );
        let id = initial[0].schedule_id.clone();
        let due = initial[0].next_at;
        assert_eq!(due, at + 60);
        {
            let mut st = state.lock().unwrap();
            if mode == "dnd" {
                st.presences.get_mut("instance-1").unwrap().dnd_until =
                    Some(clock + Duration::from_secs(1800));
            } else if mode == "capacite" {
                let registry_path = st.registry.source().to_path_buf();
                st.registry = crate::registry::AgentRegistry::from_json(
                    &serde_json::json!({"agents":{"claude":{"command":"/bin/sh","protocol":"tmux",
                            "forbidden_env":[],"pass_env":[]}}})
                    .to_string(),
                    registry_path,
                )
                .unwrap();
                assert_eq!(st.registry.get("claude").unwrap().protocol, "tmux");
            } else if mode == "circuit" {
                for _ in 0..8 {
                    st.circuit_breaker.record_at(&owner, &owner, clock);
                }
            } else if mode != "depart_unknown" {
                st.connections.remove("conn-1");
            }
            if mode == "pause" {
                let current = crate::referent_control::read(st.store.connection()).unwrap();
                crate::referent_control::set(
                    st.store.connection(),
                    crate::referent_control::ControlMutation {
                        command_id: "pause-wake158b",
                        expected_generation: current.generation,
                        paused: Some(true),
                        auto_objectives_cap: None,
                        reason: Some("pause15min"),
                        actor: "humain",
                        now: due,
                        agent_posture: None,
                        auto_reassignment: None,
                    },
                )
                .unwrap()
                .unwrap();
            }
        }
        scheduled_wakes_tick(&state, due, clock);
        let mut effective_due = due;
        if mode == "pause" {
            assert!(
                state
                    .lock()
                    .unwrap()
                    .store
                    .wake_schedule(&id)
                    .unwrap()
                    .unwrap()
                    .occurrence
                    .is_none(),
                "aucun gel pendant pause"
            );
            scheduled_wakes_tick(&state, due + 900, clock + Duration::from_secs(900));
            assert!(
                state
                    .lock()
                    .unwrap()
                    .store
                    .wake_schedule(&id)
                    .unwrap()
                    .unwrap()
                    .occurrence
                    .is_none()
            );
            {
                let st = state.lock().unwrap();
                let current = crate::referent_control::read(st.store.connection()).unwrap();
                crate::referent_control::set(
                    st.store.connection(),
                    crate::referent_control::ControlMutation {
                        command_id: "resume-wake158b",
                        expected_generation: current.generation,
                        paused: Some(false),
                        auto_objectives_cap: None,
                        reason: None,
                        actor: "humain",
                        now: due + 900,
                        agent_posture: None,
                        auto_reassignment: None,
                    },
                )
                .unwrap()
                .unwrap();
                st.delegation_store
                    .set_delegation_halt("halt-wake158b", true, "humain", None, || Ok(0), due + 900)
                    .unwrap();
            }
            effective_due = due + 900;
            scheduled_wakes_tick(&state, effective_due, clock + Duration::from_secs(900));
        }
        let frozen = state
            .lock()
            .unwrap()
            .store
            .wake_schedule(&id)
            .unwrap()
            .unwrap();
        let wake = frozen
            .occurrence
            .clone()
            .expect("occurrence réelle gelée après admission du tick");
        assert_eq!(wake.due_at, effective_due);
        assert_eq!(wake.grace_until, effective_due + 600);
        assert_eq!(wake.owner, owner);
        assert_eq!(wake.label, "libellé complet158B");
        assert!(
            wake.body.contains(&owner) && wake.body.contains(&wake.label),
            "attribution/label complets : {}",
            wake.body
        );
        assert!(frozen.next_at > effective_due);
        assert!(
            !wake.phase.terminal(),
            "aucune fausse confirmation avant ACK"
        );
        if mode == "depart_unknown" {
            assert_eq!(wake.phase, WakePhase::DispatchingAgent);
            match read_control(&mut reader) {
                DaemonToWrapper::DeliverIdempotent { message, .. } => {
                    assert_eq!(message.origin, Some(bridget_core::MessageOrigin::System));
                    assert_eq!(message.body, wake.body);
                }
                other => panic!("départ099 réel attendu : {other:?}"),
            }
        }
        if mode == "expiration_writer" {
            state
                .lock()
                .unwrap()
                .connections
                .insert("conn-1".into(), writer.clone());
            let held = Arc::new(std::sync::Barrier::new(2));
            let release = Arc::new(std::sync::Barrier::new(2));
            let held_child = held.clone();
            let release_child = release.clone();
            let blocked_writer = writer.clone();
            let blocker = thread::spawn(move || {
                let guard = blocked_writer.lock().unwrap();
                held_child.wait();
                release_child.wait();
                thread::sleep(Duration::from_millis(200));
                drop(guard);
            });
            held.wait();
            let boundary_budget = Instant::now() + Duration::from_secs(2);
            let (departure_clock, wall_tick) = loop {
                let candidate_clock = Instant::now();
                let wall = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap();
                if (900..950).contains(&wall.subsec_millis())
                    && candidate_clock.elapsed() < Duration::from_millis(50)
                {
                    break (candidate_clock, wall);
                }
                assert!(
                    Instant::now() < boundary_budget,
                    "fraction de seconde observable"
                );
                thread::sleep(Duration::from_millis(1));
            };
            release.wait();
            let outcome = deliver_frozen_wake(
                &state,
                &wake,
                Some(&id),
                wake.grace_until - 1,
                departure_clock,
            );
            blocker.join().unwrap();
            let wall_after = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap();
            assert!(
                departure_clock.elapsed() < Duration::from_secs(1),
                "bornewriter1s observée"
            );
            assert!(
                wall_after.as_secs() > wall_tick.as_secs(),
                "seconde réellement franchie : {wall_tick:?} -> {wall_after:?}"
            );
            eprintln!(
                "G5 writer expiry : wall={wall_tick:?}->{wall_after:?}, logical_now={}, grace={}, outcome={outcome:?}",
                wake.grace_until - 1,
                wake.grace_until
            );
            let key = IdempotencyKey::new(
                crate::wake_schedules::SCHEDULED_WAKE_SCOPE,
                OperationKind::Send,
                &wake.id,
            )
            .unwrap();
            assert_eq!(
                state
                    .lock()
                    .unwrap()
                    .idempotency
                    .send_delivery_phase(&key)
                    .unwrap(),
                None,
                "aucun départ099 après expiration franchie pendant writer"
            );
            assert!(
                matches!(
                    outcome,
                    Ok(WakePhase::Deferred) | Ok(WakePhase::MissedBeforeDeparture)
                ),
                "{outcome:?}"
            );
            reader.get_ref().set_nonblocking(true).unwrap();
            let mut line = String::new();
            assert!(
                matches!(reader.read_line(&mut line),Err(e) if e.kind()==std::io::ErrorKind::WouldBlock),
                "aucune trame après expiration"
            );
            let saved = state
                .lock()
                .unwrap()
                .store
                .wake_schedule(&id)
                .unwrap()
                .unwrap()
                .occurrence
                .unwrap();
            assert_eq!(saved.canonical_bytes, wake.canonical_bytes);
            assert_eq!(saved.grace_until, wake.grace_until);
            drop(state);
            let _ = std::fs::remove_file(&config.db_path);
            continue;
        }
        if mode == "sql099" {
            state
                .lock()
                .unwrap()
                .store
                .connection()
                .execute_batch("DROP TABLE send_deliveries")
                .unwrap();
        }
        if mode == "circuit" {
            let mut st = state.lock().unwrap();
            for _ in 0..8 {
                st.circuit_breaker
                    .record_at(&owner, &owner, clock + Duration::from_secs(1499));
            }
        }
        scheduled_wakes_tick(
            &state,
            effective_due + 599,
            clock + Duration::from_secs(1499),
        );
        let before_expiry = state
            .lock()
            .unwrap()
            .store
            .wake_schedule(&id)
            .unwrap()
            .unwrap()
            .occurrence
            .unwrap();
        assert_eq!(before_expiry.id, wake.id);
        assert_eq!(before_expiry.canonical_bytes, wake.canonical_bytes);
        assert_eq!(
            before_expiry.grace_until, wake.grace_until,
            "aucun renouvellement au retry"
        );
        scheduled_wakes_tick(
            &state,
            effective_due + 600,
            clock + Duration::from_secs(1500),
        );
        let terminal = state
            .lock()
            .unwrap()
            .store
            .wake_schedule(&id)
            .unwrap()
            .unwrap()
            .occurrence
            .unwrap();
        if mode == "sql099" {
            assert_ne!(
                terminal.phase,
                WakePhase::MissedBeforeDeparture,
                "erreurSQL n'est pas absence099 prouvée"
            );
            assert_ne!(terminal.phase, WakePhase::Confirmed);
        } else if mode == "depart_unknown" {
            assert_eq!(
                terminal.phase,
                WakePhase::OutcomeUnknown,
                "départ sans ACK n'est jamais absence d'injection"
            );
        } else {
            assert_eq!(
                terminal.phase,
                WakePhase::MissedBeforeDeparture,
                "borne600s sans remise099"
            );
        }
        reader.get_ref().set_nonblocking(true).unwrap();
        let mut line = String::new();
        assert!(
            matches!(reader.read_line(&mut line), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
            "aucune remise ni relance fournisseur"
        );
        drop(state);
        let _ = std::fs::remove_file(&config.db_path);
    }
    // Même groupe4/5 : réemploi149 de l'admission et du bail natifs réels.
    for closing in ["cancel", "sql"] {
        use super::native_delegation::native_delegation_permissions149_tests::{
            fixture_with_reader, mission_native,
        };
        let (mut st, config, commands, _parent_reader) = fixture_with_reader("w158b-native");
        let task = mission_native(&mut st, &commands, "wake-origin158b", "working");
        assert_eq!(
            task.effective_posture,
            Some(bridget_transport::protocol::SpawnPosture::Discovery)
        );
        let instance = task.child_instance.clone().unwrap();
        let (writer, mut child_reader) = control_socket("w158b-native-child");
        st.connections
            .insert("wake-native-child".into(), writer.clone());
        let host = st.host.clone();
        assert!(matches!(
            handle_register_with_channel(
                "wake-native-child",
                2,
                "native-test".into(),
                task.child.clone(),
                Some(host),
                Some("codex_app_server".into()),
                ChannelReport::Known("unix".into()),
                Some(PresenceMode::Acp),
                None,
                Some("macOS".into()),
                Some(instance.clone()),
                None,
                false,
                Some(true),
                &mut st,
                false
            ),
            DaemonToWrapper::Registered {
                credential: Some(_),
                ..
            }
        ));
        st.conn_instances
            .insert("wake-native-child".into(), instance.clone());
        let mut child_presence = st.presences["instance-1"].clone();
        child_presence.name = task.child.clone();
        child_presence.agent_type = "native-test".into();
        child_presence.transport = "codex_app_server".into();
        child_presence.host = st.host.clone();
        st.presences.insert(instance.clone(), child_presence);
        let state = Arc::new(Mutex::new(st));
        let at = unix_now_secs();
        let response = handle_wake_request(
            "wake-native-child",
            WakeRequest {
                version: 1,
                request: WakeAction::Schedule {
                    target: WakeTarget::SelfTarget {},
                    cadence: WakeCadence::Interval { seconds: 60 },
                    label: Some("self discovery natif158B".into()),
                },
            },
            &state,
            at,
        );
        assert!(
            matches!(response, DaemonToWrapper::WakeResult { .. }),
            "{response:?}"
        );
        let schedules = state
            .lock()
            .unwrap()
            .store
            .list_wake_schedules(Some(&task.child))
            .unwrap();
        assert_eq!(
            schedules.len(),
            1,
            "enfant discovery actif peut programmer self"
        );
        let id = schedules[0].schedule_id.clone();
        let origin = schedules[0]
            .owner_native
            .as_ref()
            .expect("preuve native capturée, aucun repli ordinaire");
        assert_eq!(origin.task_id, task.task_id);
        assert_eq!(origin.instance_id, instance);
        assert_eq!(origin.mission, task.mission);
        {
            let st = state.lock().unwrap();
            if closing == "cancel" {
                let (receipt, changed) = st
                    .delegation_store
                    .human_cancel(
                        task.root_owner(),
                        &task.task_id,
                        &Uuid::new_v4().to_string(),
                    )
                    .unwrap();
                assert!(changed);
                assert_eq!(
                    receipt["status"], "cancelling",
                    "fin demandée par human_cancel réel"
                );
            } else {
                st.store
                    .connection()
                    .execute_batch("DROP TABLE native_delegations")
                    .unwrap();
            }
        }
        scheduled_wakes_tick(&state, at + 60, Instant::now());
        let saved = state
            .lock()
            .unwrap()
            .store
            .wake_schedule(&id)
            .unwrap()
            .unwrap();
        assert!(
            saved.owner_native.is_some(),
            "SQLerror/fin ne transforme pas l'enfant en agent ordinaire"
        );
        if closing == "cancel" {
            assert!(
                saved.cancelled_at.is_some(),
                "fin native annule les occurrences futures"
            );
        }
        assert!(
            saved.occurrence.as_ref().is_none_or(|wake| !matches!(
                wake.phase,
                WakePhase::Confirmed | WakePhase::DispatchingAgent
            )),
            "aucun départ après fin ou SQL illisible"
        );
        child_reader.get_ref().set_nonblocking(true).unwrap();
        let mut line = String::new();
        assert!(
            matches!(child_reader.read_line(&mut line), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
            "aucune remise à l'enfant pourtant encore connecté"
        );
        drop(state);
        drop(writer);
        drop(commands);
        let _ = std::fs::remove_file(&config.db_path);
    }
}

#[test]
fn wake158b_daemon_g6_fil_commit_uuid_et_revalidation() {
    use crate::wake_schedules::WakePhase;
    use bridget_transport::protocol::{WakeAction, WakeCadence, WakeRequest, WakeTarget};
    for refusal in ["aucun", "projet", "membre"] {
        let (mut st, config) = state_with_registered_agent(&format!("w158b-fil-{refusal}"));
        let owner = st.presences["instance-1"].name.clone();
        let peer = "15800000-0000-4000-8000-000000000006";
        let (writer, _owner_reader) = control_socket("w158b-fil-owner");
        st.connections.insert("conn-1".into(), writer);
        let (peer_writer, _peer_reader) = control_socket("w158b-fil-peer");
        st.connections.insert("conn-fil-peer".into(), peer_writer);
        assert!(matches!(
            handle_register_with_channel(
                "conn-fil-peer",
                2,
                "claude".into(),
                peer.into(),
                Some("macbook".into()),
                Some("acp".into()),
                ChannelReport::Known("unix".into()),
                Some(PresenceMode::Acp),
                None,
                Some("macOS".into()),
                Some("instance-fil-peer".into()),
                None,
                false,
                Some(true),
                &mut st,
                false
            ),
            DaemonToWrapper::Registered {
                credential: Some(_),
                ..
            }
        ));
        let mut presence = st.presences["instance-1"].clone();
        presence.name = peer.into();
        st.presences.insert("instance-fil-peer".into(), presence);
        st.conn_instances
            .insert("conn-fil-peer".into(), "instance-fil-peer".into());
        let project = bridget_transport::protocol::CommunicationProject {
            host: "macbook".into(),
            root: config
                .db_path
                .parent()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        };
        st.communication_projects
            .insert("conn-1".into(), (Some(project.clone()), None, false));
        st.communication_projects
            .insert("conn-fil-peer".into(), (Some(project), None, false));
        let at = unix_now_secs();
        let request = serde_json::from_value(serde_json::json!({"version":1,"request":{
                "action":"create","title":"réveil fil158B","members":[peer],"operation_id":Uuid::new_v4().to_string()
            }})).unwrap();
        let (created, _) = crate::threads::handle_with_change(
            &st.store,
            &DaemonDirectory { state: &st },
            &owner,
            request,
            at,
        );
        assert_eq!(created.result["status"], "created", "{}", created.result);
        let thread_id = created.result["thread_id"].as_str().unwrap().to_string();
        let post = serde_json::from_value(serde_json::json!({"version":1,"request":{
            "action":"post","thread_id":thread_id,"body":"sollicitation ordinaire conservée",
            "notify":[owner],"operation_id":Uuid::new_v4().to_string()
        }}))
        .unwrap();
        let (posted, _) = crate::threads::handle_with_change(
            &st.store,
            &DaemonDirectory { state: &st },
            peer,
            post,
            at,
        );
        assert_eq!(posted.result["status"], "posted", "{}", posted.result);
        let state = Arc::new(Mutex::new(st));
        let label = "é".repeat(1024);
        let response = handle_wake_request(
            "conn-1",
            WakeRequest {
                version: 1,
                request: WakeAction::Schedule {
                    target: WakeTarget::Thread {
                        id: thread_id.clone(),
                    },
                    cadence: WakeCadence::Interval { seconds: 60 },
                    label: Some(label.clone()),
                },
            },
            &state,
            at,
        );
        assert!(
            matches!(response, DaemonToWrapper::WakeResult { .. }),
            "{response:?}"
        );
        let initial = state
            .lock()
            .unwrap()
            .store
            .list_wake_schedules(Some(&owner))
            .unwrap();
        assert_eq!(initial.len(), 1);
        let id = initial[0].schedule_id.clone();
        {
            let mut st = state.lock().unwrap();
            if refusal == "projet" {
                st.communication_projects
                    .get_mut("conn-fil-peer")
                    .unwrap()
                    .0
                    .as_mut()
                    .unwrap()
                    .root
                    .push_str("/autre-projet");
            } else if refusal == "membre" {
                // Retrait cohérent du membre et de ses dépendances dans la DB isolée.
                let tx = st.store.connection().unchecked_transaction().unwrap();
                assert_eq!(
                    tx.execute(
                        "DELETE FROM thread_wakes WHERE thread_id=?1 AND agent_id=?2",
                        rusqlite::params![thread_id, owner]
                    )
                    .unwrap(),
                    1,
                    "réveil du post ordinaire pour ce membre"
                );
                tx.execute(
                    "DELETE FROM thread_reads WHERE thread_id=?1 AND agent_id=?2",
                    rusqlite::params![thread_id, owner],
                )
                .unwrap();
                assert_eq!(
                    tx.execute(
                        "DELETE FROM discussion_members WHERE thread_id=?1 AND agent_id=?2",
                        rusqlite::params![thread_id, owner]
                    )
                    .unwrap(),
                    1
                );
                tx.commit().unwrap();
            }
        }
        scheduled_wakes_tick(&state, at + 60, Instant::now());
        let st = state.lock().unwrap();
        let saved = st.store.wake_schedule(&id).unwrap().unwrap();
        let count: i64 = st
            .store
            .connection()
            .query_row(
                "SELECT count(*) FROM discussion_entries WHERE thread_id=?1",
                [&thread_id],
                |r| r.get(0),
            )
            .unwrap();
        if refusal != "aucun" {
            assert_eq!(count, 1, "aucun post après retrait de droit {refusal}");
            assert_ne!(
                saved.occurrence.as_ref().map(|w| &w.phase),
                Some(&WakePhase::CommittedThread)
            );
        } else {
            assert_eq!(count, 2);
            let wake = saved.occurrence.as_ref().unwrap();
            assert_eq!(
                wake.phase,
                WakePhase::CommittedThread,
                "frontière commit102, pas ACK de l'alerte"
            );
            let operation = wake.thread_operation_id.as_ref().unwrap();
            assert!(Uuid::parse_str(operation).is_ok());
            let receipt_count: i64 = st
                .store
                .connection()
                .query_row(
                    "SELECT count(*) FROM thread_operations WHERE operation_id=?1",
                    [operation],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(receipt_count, 1, "UUID102 gelé unique");
            let (author, body, kind):(String,String,Option<String>) = st.store.connection().query_row(
                    "SELECT author_id,body,kind FROM discussion_entries WHERE thread_id=?1 ORDER BY seq DESC LIMIT 1", [&thread_id],
                    |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).unwrap();
            assert_eq!(author, owner);
            assert!(
                body.ends_with(&label),
                "label UTF8 entier2048octets après préfixe"
            );
            assert!(kind.is_none());
        }
        drop(st);
        if refusal == "aucun" {
            // Crash après commit102 mais avant la clôture locale du réveil.
            let mut frozen = saved.occurrence.clone().unwrap();
            frozen.phase = WakePhase::Prepared;
            let operation = frozen.thread_operation_id.as_ref().unwrap();
            let retry_at = frozen.grace_until + 1;
            let original_hash: String;
            {
                let mut st = state.lock().unwrap();
                assert_eq!(st.store.connection().execute(
                        "UPDATE wake_schedules SET payload=json_set(payload,'$.occurrence.phase','prepared') WHERE schedule_id=?1 AND json_extract(payload,'$.occurrence.id')=?2",
                        rusqlite::params![id, frozen.id]).unwrap(), 1);
                st.communication_projects
                    .get_mut("conn-1")
                    .unwrap()
                    .0
                    .as_mut()
                    .unwrap()
                    .root
                    .push_str("/projet-retiré");
                original_hash = st.store.connection().query_row(
                        "SELECT canonical_hash FROM thread_operations WHERE actor_id=?1 AND operation_id=?2",
                        rusqlite::params![owner, operation], |r|r.get(0)).unwrap();
                assert_eq!(st.store.connection().execute(
                        "UPDATE thread_operations SET canonical_hash='canon-divergent158B' WHERE actor_id=?1 AND operation_id=?2",
                        rusqlite::params![owner, operation]).unwrap(), 1);
            }
            assert_eq!(
                deliver_frozen_wake(&state, &frozen, Some(&id), retry_at, Instant::now()),
                Err("envelope_mismatch"),
                "même UUID102 mais canon différent : aucun nouveau post"
            );
            {
                let st = state.lock().unwrap();
                st.store.connection().execute(
                        "UPDATE thread_operations SET canonical_hash=?3 WHERE actor_id=?1 AND operation_id=?2",
                        rusqlite::params![owner, operation, original_hash]).unwrap();
                st.store
                    .connection()
                    .execute_batch(
                        "ALTER TABLE thread_operations RENAME TO thread_operations_fixture158B;",
                    )
                    .unwrap();
            }
            let unreadable =
                deliver_frozen_wake(&state, &frozen, Some(&id), retry_at, Instant::now());
            {
                let st = state.lock().unwrap();
                st.store
                    .connection()
                    .execute_batch(
                        "ALTER TABLE thread_operations_fixture158B RENAME TO thread_operations;",
                    )
                    .unwrap();
                assert_eq!(
                    st.store
                        .wake_schedule(&id)
                        .unwrap()
                        .unwrap()
                        .occurrence
                        .unwrap()
                        .phase,
                    WakePhase::Prepared,
                    "les refus ne confirment pas le post"
                );
                let unchanged: i64 = st
                    .store
                    .connection()
                    .query_row(
                        "SELECT count(*) FROM discussion_entries WHERE thread_id=?1",
                        [&thread_id],
                        |r| r.get(0),
                    )
                    .unwrap();
                assert_eq!(
                    unchanged, count,
                    "collision et panne de lecture ne republient pas"
                );
            }
            assert_eq!(
                unreadable,
                Err("storage_unavailable"),
                "erreur SQL102 distincte d'un reçu absent"
            );
            scheduled_wakes_tick(&state, retry_at, Instant::now());
            let st = state.lock().unwrap();
            let recovered = st
                .store
                .wake_schedule(&id)
                .unwrap()
                .unwrap()
                .occurrence
                .unwrap();
            assert_eq!(
                recovered.phase,
                WakePhase::CommittedThread,
                "reçu102 retrouvé après grâce600s et retrait du projet"
            );
            assert_eq!(recovered.id, frozen.id);
            assert_eq!(recovered.thread_operation_id, frozen.thread_operation_id);
            assert_eq!(recovered.issued_at, frozen.issued_at);
            assert_eq!(recovered.canonical_bytes, frozen.canonical_bytes);
            let unchanged: i64 = st
                .store
                .connection()
                .query_row(
                    "SELECT count(*) FROM discussion_entries WHERE thread_id=?1",
                    [&thread_id],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(
                unchanged, count,
                "reprise du commit102 sans seconde contribution"
            );
        }
        let cancel_at = if refusal == "aucun" {
            saved.occurrence.as_ref().unwrap().grace_until + 2
        } else {
            at + 61
        };
        let response = handle_wake_request(
            "conn-1",
            WakeRequest {
                version: 1,
                request: WakeAction::Cancel {
                    schedule_id: id.clone(),
                },
            },
            &state,
            cancel_at,
        );
        assert!(matches!(response, DaemonToWrapper::WakeResult { .. }));
        scheduled_wakes_tick(&state, cancel_at + 600, Instant::now());
        let st = state.lock().unwrap();
        let after: i64 = st
            .store
            .connection()
            .query_row(
                "SELECT count(*) FROM discussion_entries WHERE thread_id=?1",
                [&thread_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            after, count,
            "aucune republication de l'UUID ni occurrence après cancel"
        );
        let ordinary:i64 = st.store.connection().query_row("SELECT count(*) FROM thread_wakes WHERE thread_id=?1 AND agent_id=?2 AND pending_seq>0",
                rusqlite::params![thread_id, owner], |r|r.get(0)).unwrap();
        if refusal != "membre" {
            assert_eq!(
                ordinary, 1,
                "pas TTL600s sur WakeRow partagée avec post ordinaire102"
            );
        }
        drop(st);
        // Frontière CLI-fil distincte du Schedule primaire : preuve149 réelle.
        let (aux_writer, _aux_reader) = control_socket("w158b-fil-cli-aux");
        let credential = {
            let mut st = state.lock().unwrap();
            if refusal == "aucun" {
                st.communication_projects.get_mut("conn-1").unwrap().0 =
                    saved.owner_project.clone();
            }
            st.connections.insert("conn-fil-cli-aux".into(), aux_writer);
            st.identity_credentials["conn-1"].clone()
        };
        assert!(matches!(
            handle_wrapper_message(
                "conn-fil-cli-aux",
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Client
                },
                &state
            ),
            Some(DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Client
            })
        ));
        assert!(matches!(
            handle_wrapper_message(
                "conn-fil-cli-aux",
                WrapperToDaemon::RegisterAuxiliary {
                    agent_id: owner.clone(),
                    instance_id: "instance-1".into(),
                    credential
                },
                &state
            ),
            Some(DaemonToWrapper::Registered {
                credential: None,
                ..
            })
        ));
        for conn in ["conn-fil-cli-absent", "conn-fil-cli-invalid"] {
            if conn.ends_with("invalid") {
                assert!(matches!(
                    handle_wrapper_message(
                        conn,
                        WrapperToDaemon::RoleHandshake {
                            role: ConnectionRole::Client
                        },
                        &state
                    ),
                    Some(DaemonToWrapper::RoleAccepted {
                        role: ConnectionRole::Client
                    })
                ));
                assert!(matches!(
                    handle_wrapper_message(
                        conn,
                        WrapperToDaemon::RegisterAuxiliary {
                            agent_id: owner.clone(),
                            instance_id: "instance-1".into(),
                            credential: bridget_transport::protocol::IdentityCredential::new(
                                "preuve-invalide158b".into()
                            )
                        },
                        &state
                    ),
                    Some(DaemonToWrapper::Nack { .. })
                ));
            }
            let key = format!("cli-fil-{refusal}-{conn}");
            let response = handle_wake_request(
                conn,
                WakeRequest {
                    version: 1,
                    request: WakeAction::CliOnce {
                        target: thread_id.clone(),
                        label: Some("sans contexte valide".into()),
                        id: key.clone(),
                    },
                },
                &state,
                at + 1300,
            );
            assert!(matches!(response, DaemonToWrapper::WakeResult { result }
                    if result.result["code"] == "identity_unavailable"));
            assert!(
                state
                    .lock()
                    .unwrap()
                    .idempotency
                    .cli_wake(&key)
                    .unwrap()
                    .is_none(),
                "refus avant réserve099"
            );
        }
        let key = format!("cli-fil-atteste-{refusal}");
        let response = handle_wake_request(
            "conn-fil-cli-aux",
            WakeRequest {
                version: 1,
                request: WakeAction::CliOnce {
                    target: thread_id.clone(),
                    label: Some("fil headless attesté".into()),
                    id: key.clone(),
                },
            },
            &state,
            at + 1300,
        );
        let DaemonToWrapper::WakeResult { result } = response else {
            panic!("WakeResult attendu");
        };
        if refusal == "aucun" {
            assert_eq!(result.result["status"], "prepared", "{}", result.result);
            scheduled_wakes_tick(&state, at + 1300, Instant::now());
            let st = state.lock().unwrap();
            assert_eq!(
                st.idempotency.cli_wake(&key).unwrap().unwrap().1,
                crate::idempotency::RecordState::Terminal,
                "commit102 clôture la commande099"
            );
            let (posts, author):(i64,String) = st.store.connection().query_row(
                    "SELECT count(*),author_id FROM discussion_entries WHERE thread_id=?1 AND seq=(SELECT max(seq) FROM discussion_entries WHERE thread_id=?1)",
                    [&thread_id], |r|Ok((r.get(0)?,r.get(1)?))).unwrap();
            assert_eq!(
                (posts, author),
                (1, owner.clone()),
                "CLI auxiliaire attribuée à l'acteur attesté"
            );
        } else {
            assert_eq!(result.result["status"], "refused", "{}", result.result);
            assert!(
                state
                    .lock()
                    .unwrap()
                    .idempotency
                    .cli_wake(&key)
                    .unwrap()
                    .is_none(),
                "droit membre/projet avant réserve099"
            );
        }
        let final_count: i64 = state
            .lock()
            .unwrap()
            .store
            .connection()
            .query_row(
                "SELECT count(*) FROM discussion_entries WHERE thread_id=?1",
                [&thread_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(final_count, count + i64::from(refusal == "aucun"));
        drop(state);
        let _ = std::fs::remove_file(&config.db_path);
    }
}

#[test]
fn spec251_observation_dormant_conserve_fait_sans_job_et_actif_reste_notifie() {
    use bridget_transport::protocol::{ObservationKind, ObservationRequest};
    let (mut st, _config) = state_with_registered_agent("observation251");
    let owner = st.presences["instance-1"].name.clone();
    let (writer, _reader) = control_socket("observation251");
    st.connections.insert("conn-1".into(), writer);
    let (tx, jobs) = mpsc::sync_channel(8);
    st.observation_tx = tx;
    let at = unix_now_secs();
    let clock = Instant::now();
    st.observations.set_source(
        "source251",
        "source251",
        vec![ObservationKind::PermissionRequired],
        clock,
    );
    for sleeping in [true, false] {
        let agent = agent_tracking::directory(&mut st)
            .unwrap()
            .into_iter()
            .find(|a| a.agent.agent_id == owner)
            .unwrap();
        crate::agent_tracking::set(
            st.store.connection(),
            "human",
            &Uuid::new_v4().to_string(),
            if sleeping { b"sleep" } else { b"awake" },
            &agent,
            sleeping,
            Some(false),
            None,
            at,
        )
        .unwrap();
        let t = clock + Duration::from_secs(if sleeping { 0 } else { 1 });
        let subscribed = st.observations.request(
            &owner,
            ObservationRequest::Sub {
                event: ObservationKind::PermissionRequired,
                agent: Some("source251".into()),
                file: None,
                once: true,
                ttl_secs: Some(3600),
            },
            t,
            at as u64,
        );
        assert_eq!(subscribed["status"], "subscribed");
        observe_fact_at(
            &mut st,
            crate::observation::Fact {
                event: ObservationKind::PermissionRequired,
                agent: "source251".into(),
                host: "fixture".into(),
                file: None,
                other_agent: None,
            },
            at,
            t,
        );
        if sleeping {
            assert!(
                matches!(jobs.try_recv(), Err(mpsc::TryRecvError::Empty)),
                "aucun job fournisseur au dormant"
            );
            assert_eq!(st.store.connection().query_row("SELECT COUNT(*) FROM observation_missed WHERE owner=?1 AND phase='missed_before_write'",[&owner],|r|r.get::<_,i64>(0)).unwrap(),1);
        } else {
            assert!(
                matches!(jobs.try_recv(), Ok(ObservationJob::Notice { .. })),
                "attente active reste utile"
            );
        }
    }
}

#[test]
fn wake158b_daemon_g8_g10_once_worker_et_retour_ack() {
    use crate::store::observation_missed::ObservationWriteOutcome;
    use bridget_transport::protocol::{ObservationKind, ObservationRequest};
    let (mut st, config) = state_with_registered_agent("w158b-worker");
    let owner = st.presences["instance-1"].name.clone();
    let (writer, _initial_reader) = control_socket("w158b-worker");
    st.connections.insert("conn-1".into(), writer.clone());
    let (tx, jobs) = mpsc::sync_channel(64);
    st.observation_tx = tx;
    let at = unix_now_secs();
    let clock = Instant::now();
    st.observations.set_source(
        "source158b",
        "source158b",
        vec![ObservationKind::PermissionRequired],
        clock,
    );
    for index in 0..65 {
        let instant = clock + Duration::from_millis(index * 250);
        let response = st.observations.request(
            &owner,
            ObservationRequest::Sub {
                event: ObservationKind::PermissionRequired,
                agent: Some("source158b".into()),
                file: None,
                once: true,
                ttl_secs: Some(3600),
            },
            instant,
            at as u64,
        );
        assert_eq!(response["status"], "subscribed", "{response}");
        observe_fact_at(
            &mut st,
            crate::observation::Fact {
                event: ObservationKind::PermissionRequired,
                agent: "source158b".into(),
                host: "fixture158b".into(),
                file: None,
                other_agent: None,
            },
            at,
            instant,
        );
    }
    assert!(
        st.observations.subscriptions.is_empty(),
        "once consommées, provenance durable conservée"
    );
    let state = Arc::new(Mutex::new(st));
    state
        .lock()
        .unwrap()
        .presences
        .get_mut("instance-1")
        .unwrap()
        .dnd_until = Some(clock + Duration::from_secs(1000));
    observer_return_tick(&state, at + 1, clock + Duration::from_secs(1));
    // Réemploi reconnexion_redeclare_busy_et_arret_propre_reste_stopped :
    // retirer la route pendant le même épisode absent/DND avant Registered.
    let (reconnected_writer, reconnected_reader) = control_socket("w158b-worker-return");
    {
        let mut st = state.lock().unwrap();
        assert_eq!(st.router.get_agent(&owner).unwrap().connection_id, "conn-1");
        let removed = st
            .router
            .unregister_by_conn("conn-1")
            .expect("route primaire avant coupure");
        assert_eq!(removed.agent_id, owner);
        st.mark_unreachable("conn-1");
        st.connections.remove("conn-1");
        assert!(st.router.get_agent(&owner).is_none());
        assert!(!st.conn_instances.contains_key("conn-1"));
        st.connections
            .insert("conn-return158b".into(), reconnected_writer.clone());
        let registered = handle_register_with_channel(
            "conn-return158b",
            2,
            "claude".into(),
            owner.clone(),
            Some("macbook".into()),
            Some("acp".into()),
            ChannelReport::Known("unix".into()),
            Some(PresenceMode::Acp),
            None,
            Some("macOS".into()),
            Some("instance-1".into()),
            None,
            false,
            Some(true),
            &mut st,
            false,
        );
        assert!(
            matches!(
                registered,
                DaemonToWrapper::Registered {
                    credential: Some(_),
                    ..
                }
            ),
            "reconnexion primaire réelle : {registered:?}"
        );
        assert_eq!(
            st.router.get_agent(&owner).unwrap().connection_id,
            "conn-return158b"
        );
        assert_eq!(st.conn_instances["conn-return158b"], "instance-1");
    }
    let mut reader = reconnected_reader;
    observer_return_tick(&state, at + 2, clock + Duration::from_secs(2)); // hook Registered
    state
        .lock()
        .unwrap()
        .presences
        .get_mut("instance-1")
        .unwrap()
        .dnd_until = None;
    observer_return_tick(&state, at + 2, clock + Duration::from_secs(2)); // hook levée DND
    assert!(
        state
            .lock()
            .unwrap()
            .store
            .observation_summaries_pending()
            .unwrap()
            .is_empty(),
        "aucun gel avant solde des pending_write du propriétaire"
    );
    observer_return_tick(&state, at + 5, clock + Duration::from_secs(5));
    let mut pending = state
        .lock()
        .unwrap()
        .store
        .observation_summaries_pending()
        .unwrap();
    assert_eq!(
        pending.len(),
        1,
        "DND puis Register du même épisode : une génération"
    );
    let summary = pending.remove(0);
    assert_eq!(summary.owner, owner);
    assert_eq!(
        summary.episode, 1,
        "Registered et levée DND du même épisode ne doublent pas le retour"
    );
    assert_eq!(summary.generation, 1);
    assert_eq!(
        summary.fact_seq, 65,
        "séquence SQLite distingue65faits au même observed_at"
    );
    let page_before = state
        .lock()
        .unwrap()
        .store
        .observation_missed_page(&owner, Some(64), None, at + 5)
        .unwrap();
    let page_repeated = state
        .lock()
        .unwrap()
        .store
        .observation_missed_page(&owner, Some(64), None, at + 5)
        .unwrap();
    assert_eq!(
        page_before, page_repeated,
        "missed read ne supprime ni fait ni preuve"
    );
    let first_facts = page_before["facts"]
        .as_array()
        .expect("faits réellement conservés");
    assert_eq!(first_facts.len(), 64);
    let after_seq = first_facts.last().unwrap()["fact_seq"].as_u64().unwrap();
    assert_eq!(page_before["next_after_seq"].as_u64(), Some(after_seq));
    let page_next = state
        .lock()
        .unwrap()
        .store
        .observation_missed_page(&owner, Some(64), Some(after_seq), at + 5)
        .unwrap();
    let remaining_facts = page_next["facts"].as_array().unwrap();
    assert_eq!(remaining_facts.len(), 1);
    assert!(
        remaining_facts[0]["fact_seq"].as_u64().unwrap() > after_seq,
        "pas de seconde lecture de la même page"
    );
    let (delivery_id, generation) = match read_control(&mut reader) {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            message,
            ..
        } => {
            assert_eq!(message.origin, Some(bridget_core::MessageOrigin::System));
            (delivery_id, delivery_generation)
        }
        other => panic!("résumé livré099 réel attendu : {other:?}"),
    };
    {
        let mut st = state.lock().unwrap();
        assert!(
            handle_delivery_ack(
                "conn-return158b",
                delivery_id.clone(),
                generation + 1,
                &mut st
            )
            .is_some(),
            "mauvaise génération refusée"
        );
    }
    assert_eq!(
        state
            .lock()
            .unwrap()
            .store
            .observation_summaries_pending()
            .unwrap()
            .len(),
        1
    );
    let job = jobs.try_recv().expect("canal64 réellement rempli");
    if let ObservationJob::Notice {
        owner,
        fact_id,
        result_tx,
        ..
    } = job
    {
        result_tx
            .try_send(ObservationWriteResult {
                owner,
                fact_id,
                outcome: ObservationWriteOutcome::Written,
            })
            .unwrap();
    } else {
        panic!("job corrélé once attendu");
    }
    observer_return_tick(&state, at + 6, clock + Duration::from_secs(6));
    assert_eq!(
        state
            .lock()
            .unwrap()
            .store
            .observation_summaries_pending()
            .unwrap()
            .len(),
        1,
        "callback tardif ne crée aucune nouvelle génération"
    );
    {
        let mut st = state.lock().unwrap();
        assert!(
            handle_delivery_ack("conn-return158b", delivery_id, generation, &mut st).is_none(),
            "ACK099 validé même answered_request=None"
        );
    }
    assert!(
        state
            .lock()
            .unwrap()
            .store
            .observation_summaries_pending()
            .unwrap()
            .is_empty()
    );
    let page_after = state
        .lock()
        .unwrap()
        .store
        .observation_missed_page(&owner, Some(64), None, at + 6)
        .unwrap();
    assert_eq!(
        page_before["facts"], page_after["facts"],
        "ACKrésumé et callbacktardif ne suppriment pas les faits conservés"
    );
    let page_next_after = state
        .lock()
        .unwrap()
        .store
        .observation_missed_page(&owner, Some(64), Some(after_seq), at + 6)
        .unwrap();
    assert_eq!(
        page_next["facts"], page_next_after["facts"],
        "la deuxième page reste conservée après ACK"
    );
    observer_return_tick(&state, at + 7, clock + Duration::from_secs(7));
    reader.get_ref().set_nonblocking(true).unwrap();
    let mut line = String::new();
    assert!(
        matches!(reader.read_line(&mut line), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock),
        "aucun deuxième résumé au même retour"
    );
    drop(jobs);
    drop(state);
    drop(writer);
    drop(reconnected_writer);
    let _ = std::fs::remove_file(&config.db_path);
}

// T008/T012 : neuf usages. Réemploi état/registre/SQLite et négociation
// publics ; aucune présence d'enfant ni aucun fournisseur lancé.
fn control158_state_with_tasks(
    label: &str,
) -> (
    Arc<Mutex<DaemonState>>,
    DaemonConfig,
    Vec<crate::delegation::Task>,
) {
    let label = format!(
        "158-{label}-{}",
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    );
    let (state, config) = state_with_registered_agent(&label);
    let ids: Vec<_> = (0..4).map(|_| uuid::Uuid::new_v4().to_string()).collect();
    let children: Vec<_> = (0..4).map(|_| uuid::Uuid::new_v4().to_string()).collect();
    let other_owner = uuid::Uuid::new_v4().to_string();
    let now = unix_now_secs();
    let mut tasks = Vec::new();
    for i in 0..4 {
        let owner = match i {
            0 => "89000000-0000-4000-8000-000000000102",
            1 | 2 => children[i - 1].as_str(),
            _ => other_owner.as_str(),
        };
        let request_id = format!("control158-{}", ids[i]);
        let task: crate::delegation::Task = serde_json::from_value(serde_json::json!({
                "task_id":ids[i], "owner":owner, "owner_instance":format!("owner-{i}"),
                "origin_owner_instance":format!("owner-{i}"),
                "root_owner_agent_id":if i < 3 { "89000000-0000-4000-8000-000000000102" } else { other_owner.as_str() },
                "parent_task_id":if i == 1 || i == 2 { Some(ids[i-1].clone()) } else { None },
                "request":{"operation":"delegate","request_id":request_id,"agent_type":"claude",
                    "model":"fixture","task":"mission privée158","cwd":"/tmp"},
                "definition":{"command":"/bin/sh","args":[],"protocol":"acp","forbidden_env":[],
                    "pass_env":[],"permissions":"allow","queue_capacity":32,"notify_timeout_secs":600,
                    "mcp":{"interactive":"none","acp_session":false},"digest":"fixture-control158"},
                "cwd":"/tmp", "child":children[i], "child_instance":null,
                "mission":uuid::Uuid::new_v4().to_string(), "created_at":now,
                "state":if i == 0 { "failed" } else { "queued" },
                "result":null,"error":null,"result_sent":false,
                "failure_sent":true,"cleanup_done":i == 0,"delivery_acked":i == 0
            })).unwrap();
        let canonical = serde_json::to_vec(&task.request).unwrap();
        state
            .delegation_store
            .insert(&task, &request_id, &canonical)
            .unwrap();
        tasks.push(state.delegation_store.get(&task.task_id).unwrap().unwrap());
    }
    // La pause empêche le tick d'admettre une branche non visée. L'arrêt
    // global ne doit pas lever cette pause ; cancelling reste traité.
    crate::referent_control::set(
        state.store.connection(),
        crate::referent_control::ControlMutation {
            command_id: "fixture158-pause",
            expected_generation: 0,
            paused: Some(true),
            auto_objectives_cap: None,
            reason: Some("fixture158"),
            actor: "cli",
            now,
            agent_posture: None,
            auto_reassignment: None,
        },
    )
    .unwrap()
    .unwrap();
    let shared = Arc::new(Mutex::new(state));
    assert!(matches!(
        handle_wrapper_message(
            "human158",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client,
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        })
    ));
    assert!(
        matches!(handle_wrapper_message("human158", WrapperToDaemon::ClientHello {
            contract_version:CLIENT_CONTRACT_VERSION,
            issuer_scope:crate::communication::issuer_scope("bridget-control-cli"),
            capabilities:vec![ClientCapability::ControlStateV1],
        }, &shared), Some(DaemonToWrapper::ClientWelcome { capabilities, .. })
            if capabilities.contains(&ClientCapability::ControlStateV1))
    );
    (shared, config, tasks)
}

#[test]
fn control158_overview_client_agent_negocie_refuse() {
    use bridget_transport::protocol::DelegationControlRefusal as E;
    let (shared, config, tasks) = control158_state_with_tasks("agent");
    let before = {
        let st = shared.lock().unwrap();
        (
            serde_json::to_vec(&st.delegation_store.tasks().unwrap()).unwrap(),
            st.delegation_store.delegation_halt().unwrap(),
            st.delegation_store.wait_epoch().get(),
        )
    };
    assert!(matches!(
        handle_wrapper_message(
            "agent158",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client,
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted { .. })
    ));
    assert!(
        matches!(handle_wrapper_message("agent158", WrapperToDaemon::ClientHello {
            contract_version:CLIENT_CONTRACT_VERSION,
            issuer_scope:crate::communication::issuer_scope("instance-agent158"),
            capabilities:vec![ClientCapability::ControlStateV1],
        }, &shared), Some(DaemonToWrapper::ClientWelcome { capabilities, .. })
            if capabilities.contains(&ClientCapability::ControlStateV1))
    );
    for request in [
        WrapperToDaemon::DelegationOverviewRead {
            version: 1,
            state: "all".into(),
            limit: 100,
        },
        WrapperToDaemon::DelegationCancel {
            version: 1,
            task_id: tasks[1].task_id.clone(),
            request_id: "agent-cancel".into(),
        },
        WrapperToDaemon::DelegationHaltSet {
            version: 1,
            command_id: "agent-halt".into(),
            halted: true,
            reason: None,
            cancel_active: true,
        },
    ] {
        assert!(matches!(
            handle_wrapper_message("agent158", request, &shared),
            Some(DaemonToWrapper::DelegationControlRejected {
                reason: E::HumanActorRequired
            })
        ));
    }
    let st = shared.lock().unwrap();
    assert_eq!(
        before,
        (
            serde_json::to_vec(&st.delegation_store.tasks().unwrap()).unwrap(),
            st.delegation_store.delegation_halt().unwrap(),
            st.delegation_store.wait_epoch().get()
        )
    );
    drop(st);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn control158_overview_client_non_negocie_refuse() {
    let (shared, config, _) = control158_state_with_tasks("neg");
    assert!(matches!(
        handle_wrapper_message(
            "bare158",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client,
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted { .. })
    ));
    let request = WrapperToDaemon::DelegationOverviewRead {
        version: 1,
        state: "active".into(),
        limit: 50,
    };
    assert!(matches!(
        handle_wrapper_message("bare158", request.clone(), &shared),
        Some(DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::NegotiationRequired
        })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "bare158",
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: crate::communication::issuer_scope("bridget-control-cli"),
                capabilities: vec![ClientCapability::Lookup],
            },
            &shared
        ),
        Some(DaemonToWrapper::ClientWelcome { .. })
    ));
    // Lookup ne remplace pas ControlStateV1 pour cette vue globale.
    assert!(matches!(
        handle_wrapper_message("bare158", request, &shared),
        Some(DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::CapabilityNotNegotiated
        })
    ));
    assert!(
        !shared
            .lock()
            .unwrap()
            .delegation_store
            .delegation_halt()
            .unwrap()
            .halted
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn control158_overview_cancel_cascade() {
    let (shared, config, tasks) = control158_state_with_tasks("cascade");
    let response = handle_wrapper_message(
        "human158",
        WrapperToDaemon::DelegationCancel {
            version: 1,
            task_id: tasks[1].task_id.clone(),
            request_id: "human-cascade".into(),
        },
        &shared,
    );
    let Some(DaemonToWrapper::DelegationOverview {
        items,
        total_active,
        ..
    }) = response
    else {
        panic!("vue mise à jour attendue, reçu {response:?}");
    };
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["task_id"], tasks[1].task_id);
    assert_eq!(items[0]["state"], "cancelled");
    assert!(items[0].get("result").is_none() && items[0].get("task").is_none());
    assert_eq!(total_active, 1);
    let st = shared.lock().unwrap();
    assert_eq!(
        st.delegation_store
            .get(&tasks[1].task_id)
            .unwrap()
            .unwrap()
            .state,
        "cancelled"
    );
    assert_eq!(
        st.delegation_store
            .get(&tasks[2].task_id)
            .unwrap()
            .unwrap()
            .state,
        "cancelled"
    );
    assert_eq!(
        serde_json::to_value(st.delegation_store.get(&tasks[0].task_id).unwrap().unwrap()).unwrap(),
        serde_json::to_value(&tasks[0]).unwrap()
    );
    assert_eq!(
        serde_json::to_value(st.delegation_store.get(&tasks[3].task_id).unwrap().unwrap()).unwrap(),
        serde_json::to_value(&tasks[3]).unwrap()
    );
    assert!(
        crate::referent_control::read(st.store.connection())
            .unwrap()
            .paused
    );
    drop(st);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn control158_halt_exige_acteur_humain() {
    use bridget_transport::protocol::DelegationControlRefusal as E;
    let (shared, config, _) = control158_state_with_tasks("actor");
    let request = WrapperToDaemon::DelegationHaltSet {
        version: 1,
        command_id: "actor-halt".into(),
        halted: true,
        reason: None,
        cancel_active: true,
    };
    assert!(matches!(
        handle_wrapper_message(
            "bare-halt158",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client,
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted { .. })
    ));
    assert!(matches!(
        handle_wrapper_message("bare-halt158", request.clone(), &shared),
        Some(DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::NegotiationRequired
        })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "bare-halt158",
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: crate::communication::issuer_scope("instance-agent158"),
                capabilities: vec![ClientCapability::Lookup],
            },
            &shared
        ),
        Some(DaemonToWrapper::ClientWelcome { .. })
    ));
    assert!(matches!(
        handle_wrapper_message("bare-halt158", request.clone(), &shared),
        Some(DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::CapabilityNotNegotiated
        })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "cap-halt158",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client,
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted { .. })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "cap-halt158",
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: crate::communication::issuer_scope("instance-agent158-cap"),
                capabilities: vec![ClientCapability::ControlStateV1],
            },
            &shared
        ),
        Some(DaemonToWrapper::ClientWelcome { .. })
    ));
    assert!(matches!(
        handle_wrapper_message("cap-halt158", request, &shared),
        Some(DaemonToWrapper::DelegationControlRejected {
            reason: E::HumanActorRequired
        })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "human158",
            WrapperToDaemon::DelegationHaltSet {
                version: 1,
                command_id: "invalid-cancel".into(),
                halted: false,
                reason: None,
                cancel_active: true,
            },
            &shared
        ),
        Some(DaemonToWrapper::DelegationControlRejected {
            reason: E::InvalidRequest
        })
    ));
    let st = shared.lock().unwrap();
    assert!(!st.delegation_store.delegation_halt().unwrap().halted);
    assert_eq!(
        st.delegation_store
            .test_connection()
            .query_row(
                "SELECT count(*) FROM native_delegation_halt_commands",
                [],
                |row| row.get::<_, u32>(0)
            )
            .unwrap(),
        0
    );
    drop(st);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn control158_halt_cancel_active_annule_les_arbres() {
    let (shared, config, tasks) = control158_state_with_tasks("halt-cascade");
    let response = handle_wrapper_message(
        "human158",
        WrapperToDaemon::DelegationHaltSet {
            version: 1,
            command_id: "A-cancel".into(),
            halted: true,
            reason: Some("arrêt humain".into()),
            cancel_active: true,
        },
        &shared,
    );
    assert!(
        matches!(response,Some(DaemonToWrapper::DelegationHaltState {
            halted:true,cancelled:2,replayed:false,by:Some(ref actor),reason:Some(ref reason),..
        }) if actor == "cli" && reason == "arrêt humain")
    );
    let st = shared.lock().unwrap();
    for task in &tasks[1..] {
        assert_eq!(
            st.delegation_store
                .get(&task.task_id)
                .unwrap()
                .unwrap()
                .state,
            "cancelled"
        );
    }
    assert_eq!(
        serde_json::to_value(st.delegation_store.get(&tasks[0].task_id).unwrap().unwrap()).unwrap(),
        serde_json::to_value(&tasks[0]).unwrap()
    );
    let saved: String = st
        .delegation_store
        .test_connection()
        .query_row(
            "SELECT response_json FROM native_delegation_halt_commands WHERE command_id='A-cancel'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&saved).unwrap()["cancelled"],
        2
    );
    assert!(
        crate::referent_control::read(st.store.connection())
            .unwrap()
            .paused
    );
    drop(st);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn control158_halt_sans_cancel_active_ne_touche_rien() {
    let (shared, config, _) = control158_state_with_tasks("halt-only");
    let before = {
        let st = shared.lock().unwrap();
        (
            serde_json::to_vec(&st.delegation_store.tasks().unwrap()).unwrap(),
            st.delegation_store.wait_epoch().get(),
            crate::referent_control::read(st.store.connection()).unwrap(),
        )
    };
    assert!(matches!(
        handle_wrapper_message(
            "human158",
            WrapperToDaemon::DelegationHaltSet {
                version: 1,
                command_id: "halt-only".into(),
                halted: true,
                reason: None,
                cancel_active: false,
            },
            &shared
        ),
        Some(DaemonToWrapper::DelegationHaltState {
            halted: true,
            cancelled: 0,
            replayed: false,
            ..
        })
    ));
    let st = shared.lock().unwrap();
    assert_eq!(
        before,
        (
            serde_json::to_vec(&st.delegation_store.tasks().unwrap()).unwrap(),
            st.delegation_store.wait_epoch().get(),
            crate::referent_control::read(st.store.connection()).unwrap()
        )
    );
    drop(st);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn control158_halt_commande_rejouee() {
    let (shared, config, _) = control158_state_with_tasks("replay");
    let a = WrapperToDaemon::DelegationHaltSet {
        version: 1,
        command_id: "replay-A".into(),
        halted: true,
        reason: Some("historique A".into()),
        cancel_active: false,
    };
    assert!(matches!(
        handle_wrapper_message("human158", a.clone(), &shared),
        Some(DaemonToWrapper::DelegationHaltState {
            halted: true,
            cancelled: 0,
            replayed: false,
            generation: 1,
            ..
        })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "human158",
            WrapperToDaemon::DelegationHaltSet {
                version: 1,
                command_id: "replay-B".into(),
                halted: false,
                reason: None,
                cancel_active: false,
            },
            &shared
        ),
        Some(DaemonToWrapper::DelegationHaltState {
            halted: false,
            generation: 2,
            ..
        })
    ));
    assert!(matches!(handle_wrapper_message("human158",a,&shared),
            Some(DaemonToWrapper::DelegationHaltState { halted:true,cancelled:0,replayed:true,generation:1,
                reason:Some(ref reason),.. }) if reason == "historique A"));
    assert!(
        !shared
            .lock()
            .unwrap()
            .delegation_store
            .delegation_halt()
            .unwrap()
            .halted
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn control158_halt_a_b_c_rejeu_a_ne_touche_pas_c() {
    let (shared, config, tasks) = control158_state_with_tasks("abc");
    let a = WrapperToDaemon::DelegationHaltSet {
        version: 1,
        command_id: "abc-A".into(),
        halted: true,
        reason: Some("A".into()),
        cancel_active: true,
    };
    assert!(matches!(
        handle_wrapper_message("human158", a.clone(), &shared),
        Some(DaemonToWrapper::DelegationHaltState {
            cancelled: 2,
            replayed: false,
            ..
        })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "human158",
            WrapperToDaemon::DelegationHaltSet {
                version: 1,
                command_id: "abc-B".into(),
                halted: false,
                reason: None,
                cancel_active: false,
            },
            &shared
        ),
        Some(DaemonToWrapper::DelegationHaltState { halted: false, .. })
    ));
    let (c, before, epoch) = {
        let st = shared.lock().unwrap();
        let mut c = tasks[3].clone();
        c.task_id = uuid::Uuid::new_v4().to_string();
        c.child = uuid::Uuid::new_v4().to_string();
        c.mission = uuid::Uuid::new_v4().to_string();
        st.delegation_store.insert(&c, "abc-C", b"abc-C").unwrap();
        let before =
            serde_json::to_vec(&st.delegation_store.get(&c.task_id).unwrap().unwrap()).unwrap();
        // Un payload illisible prouve que le rejeu connu ne relit aucune tâche.
        st.delegation_store
            .test_connection()
            .execute(
                "UPDATE native_delegations SET payload='{}' WHERE task_id=?1",
                [&tasks[0].task_id],
            )
            .unwrap();
        (c, before, st.delegation_store.wait_epoch().get())
    };
    assert!(matches!(
        handle_wrapper_message("human158", a, &shared),
        Some(DaemonToWrapper::DelegationHaltState {
            halted: true,
            cancelled: 2,
            replayed: true,
            ..
        })
    ));
    let st = shared.lock().unwrap();
    assert_eq!(
        before,
        serde_json::to_vec(&st.delegation_store.get(&c.task_id).unwrap().unwrap()).unwrap()
    );
    assert_eq!(epoch, st.delegation_store.wait_epoch().get());
    assert!(!st.delegation_store.delegation_halt().unwrap().halted);
    drop(st);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn control158_halt_a_b_c_rejeu_a_apres_redemarrage() {
    let (shared, config, tasks) = control158_state_with_tasks("abc-reopen");
    let a = WrapperToDaemon::DelegationHaltSet {
        version: 1,
        command_id: "reopen-A".into(),
        halted: true,
        reason: Some("A durable".into()),
        cancel_active: true,
    };
    assert!(matches!(
        handle_wrapper_message("human158", a.clone(), &shared),
        Some(DaemonToWrapper::DelegationHaltState {
            cancelled: 2,
            replayed: false,
            ..
        })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "human158",
            WrapperToDaemon::DelegationHaltSet {
                version: 1,
                command_id: "reopen-B".into(),
                halted: false,
                reason: None,
                cancel_active: false,
            },
            &shared
        ),
        Some(DaemonToWrapper::DelegationHaltState { halted: false, .. })
    ));
    let (c, before) = {
        let mut st = shared.lock().unwrap();
        let mut c = tasks[3].clone();
        c.task_id = uuid::Uuid::new_v4().to_string();
        c.child = uuid::Uuid::new_v4().to_string();
        c.mission = uuid::Uuid::new_v4().to_string();
        st.delegation_store
            .insert(&c, "reopen-C", b"reopen-C")
            .unwrap();
        let before =
            serde_json::to_vec(&st.delegation_store.get(&c.task_id).unwrap().unwrap()).unwrap();
        // Fermer/reouvrir le magasin disque, sans service ni processus neuf.
        st.delegation_store = crate::delegation::DelegationStore::open(&config.db_path).unwrap();
        (c, before)
    };
    assert!(matches!(handle_wrapper_message("human158",a,&shared),
            Some(DaemonToWrapper::DelegationHaltState { halted:true,cancelled:2,replayed:true,
                reason:Some(ref reason),.. }) if reason == "A durable"));
    let st = shared.lock().unwrap();
    assert_eq!(
        before,
        serde_json::to_vec(&st.delegation_store.get(&c.task_id).unwrap().unwrap()).unwrap()
    );
    assert!(!st.delegation_store.delegation_halt().unwrap().halted);
    drop(st);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn redemarrage_daemon_ne_deconnecte_pas_les_agents_geres() {
    let (mut state, config) = state_with_registered_agent("restart-managed-connection");
    assert!(
        state.notify_wrapper_on_daemon_shutdown("conn-1"),
        "une connexion non gérée garde le comportement Disconnect existant"
    );

    state
        .managed_by_instance
        .insert("instance-1".to_string(), "managed-restart".to_string());
    assert!(
        !state.notify_wrapper_on_daemon_shutdown("conn-1"),
        "un équipier géré et running doit rester reprenable au prochain boot"
    );
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn redemarrage_daemon_preserve_seulement_la_session_terminal_attestee() {
    let (mut state, config) = state_with_registered_agent("restart-native-tui");
    let presence = state.presences.get_mut("instance-1").unwrap();
    presence.mode = Some(PresenceMode::Cli);
    presence.agent_type = "codex".into();
    presence.transport = "codex_app_server".into();
    // Même descripteur qu'un Codex géré : jamais d'inférence par élimination.
    assert!(state.notify_wrapper_on_daemon_shutdown("conn-1"));
    let shared = Arc::new(Mutex::new(state));
    assert!(matches!(
        handle_wrapper_message("unknown", WrapperToDaemon::TerminalSessionReady, &shared),
        Some(DaemonToWrapper::Nack { .. })
    ));
    assert!(
        handle_wrapper_message("conn-1", WrapperToDaemon::TerminalSessionReady, &shared).is_none()
    );
    assert!(
        !shared
            .lock()
            .unwrap()
            .notify_wrapper_on_daemon_shutdown("conn-1")
    );
    shared.lock().unwrap().mark_unreachable("conn-1");
    assert!(
        shared
            .lock()
            .unwrap()
            .notify_wrapper_on_daemon_shutdown("conn-1")
    );
    let _ = std::fs::remove_file(config.db_path);
}

/// Ajoute un expéditeur réellement enregistré dont la connexion reste
/// vivante, mais dont la dernière capacité remonte à plus de trente
/// minutes. Le cas reproduit les agents tmux reconnectés ensemble : lien
/// frais, activité publique ancienne.
fn state_with_aged_sender(label: &str) -> (DaemonState, DaemonConfig) {
    let (mut state, config) = state_with_registered_agent(label);
    assert!(matches!(
        handle_register_with_channel(
            "conn-sender",
            2,
            "codex".to_string(),
            "89000000-0000-4000-8000-000000000103".to_string(),
            Some("cartae".to_string()),
            Some("tmux".to_string()),
            ChannelReport::Known("unix".to_string()),
            Some(PresenceMode::Tmux),
            Some("session:1.1".to_string()),
            Some("Linux".to_string()),
            Some("instance-sender".to_string()),
            Some("bridget".to_string()),
            false,
            Some(false),
            &mut state,
            false,
        ),
        DaemonToWrapper::Registered { credential: _, agent_id: name } if name == "89000000-0000-4000-8000-000000000103"
    ));
    let stale = Instant::now()
        .checked_sub(Duration::from_secs(1900))
        .expect("horloge de capacité vieillie");
    let sender = state.presences.get_mut("instance-sender").unwrap();
    sender.capacity_seen = stale;
    sender.link_seen = Instant::now()
        .checked_sub(Duration::from_secs(120))
        .expect("horloge de lien encore retenue");
    (state, config)
}

fn assert_sender_last_seen(state: &mut DaemonState, expected_fresh: bool) {
    assert!(
        state
            .presences
            .get("instance-sender")
            .unwrap()
            .link_seen
            .elapsed()
            >= Duration::from_secs(60),
        "l'activité MCP/CLI ne doit pas rajeunir le socket principal"
    );
    let sender = state
        .agent_infos()
        .into_iter()
        .find(|agent| agent.agent_id == "89000000-0000-4000-8000-000000000103")
        .expect("expéditeur enregistré visible dans l'annuaire");
    if expected_fresh {
        assert!(
            sender.last_seen_secs < 2,
            "un message émis maintenant doit rajeunir last_seen_secs, reçu={} s",
            sender.last_seen_secs
        );
    } else {
        assert!(
            sender.last_seen_secs >= 1800,
            "un refus ou une réception seule ne doit pas inventer une activité, reçu={} s",
            sender.last_seen_secs
        );
    }
}

#[test]
fn session_046_envoi_idempotent_ancien_agent_actif_rajeunit_last_seen() {
    let (state, config) = state_with_aged_sender("last-seen-idempotent");
    let shared = Arc::new(Mutex::new(state));
    negotiate_idempotent_client(
        &shared,
        "client-activity",
        "046_scope_aaaaaaaaaaaa",
        "89000000-0000-4000-8000-000000000103",
    );
    let mut message = BridgetMessage::new(
        "89000000-0000-4000-8000-000000000103",
        "89000000-0000-4000-8000-000000000102",
        "activité réelle",
    );
    message.hops = 4;
    let result = handle_wrapper_message(
        "client-activity",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "activity-idempotent".to_string(),
            issued_at: unix_now_secs(),
        },
        &shared,
    );
    assert!(matches!(
        result,
        Some(DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::OutcomeUnknown { .. },
            ..
        })
    ));
    assert_sender_last_seen(&mut shared.lock().unwrap(), true);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn session_046_envoi_historique_ancien_agent_actif_rajeunit_last_seen() {
    let (mut state, config) = state_with_aged_sender("last-seen-historique");
    let (target_writer, mut target_reader) = control_socket("last-seen-historique");
    state
        .connections
        .insert("conn-1".to_string(), target_writer);
    let shared = Arc::new(Mutex::new(state));
    let result = handle_wrapper_message(
        "conn-sender",
        WrapperToDaemon::Send(BridgetMessage::new(
            "89000000-0000-4000-8000-000000000103",
            "89000000-0000-4000-8000-000000000102",
            "activité historique réelle",
        )),
        &shared,
    );
    assert!(matches!(result, Some(DaemonToWrapper::Ack { .. })));
    assert!(matches!(
        read_control(&mut target_reader),
        DaemonToWrapper::Deliver { message, .. }
            if message.from == "89000000-0000-4000-8000-000000000103" && message.to == "89000000-0000-4000-8000-000000000102"
    ));
    assert_sender_last_seen(&mut shared.lock().unwrap(), true);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn session_046_envoi_refuse_ne_rajeunit_pas_last_seen() {
    let (state, config) = state_with_aged_sender("last-seen-refus");
    let shared = Arc::new(Mutex::new(state));
    negotiate_idempotent_client(
        &shared,
        "client-refused",
        "046_scope_bbbbbbbbbbbb",
        "89000000-0000-4000-8000-000000000103",
    );
    let mut message = BridgetMessage::new(
        "89000000-0000-4000-8000-000000000103",
        "agent-inconnu",
        "à refuser",
    );
    message.hops = 4;
    let result = handle_wrapper_message(
        "client-refused",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "activity-refused".to_string(),
            issued_at: unix_now_secs(),
        },
        &shared,
    );
    assert!(matches!(
        result,
        Some(DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::Rejected { .. },
            ..
        })
    ));
    assert_sender_last_seen(&mut shared.lock().unwrap(), false);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn session_046_reception_seule_ne_rajeunit_pas_last_seen() {
    let (mut state, config) = state_with_aged_sender("last-seen-reception");
    let (target_writer, mut target_reader) = control_socket("last-seen-reception");
    state.conn_names.insert(
        "conn-1".to_string(),
        "89000000-0000-4000-8000-000000000102".to_string(),
    );
    state
        .connections
        .insert("conn-sender".to_string(), target_writer);
    let shared = Arc::new(Mutex::new(state));
    let result = handle_wrapper_message(
        "conn-1",
        WrapperToDaemon::Send(BridgetMessage::new(
            "89000000-0000-4000-8000-000000000102",
            "89000000-0000-4000-8000-000000000103",
            "mandat reçu sans activité émise",
        )),
        &shared,
    );
    assert!(matches!(result, Some(DaemonToWrapper::Ack { .. })));
    let delivered = read_control(&mut target_reader);
    assert!(
        matches!(
            delivered,
            DaemonToWrapper::Deliver { ref message, .. }
                if message.from == "89000000-0000-4000-8000-000000000102" && message.to == "89000000-0000-4000-8000-000000000103"
        ),
        "trame reçue par la cible: {delivered:?}"
    );
    assert_sender_last_seen(&mut shared.lock().unwrap(), false);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn relevé_disque_post_enregistrement_est_projete_sans_changer_la_presence() {
    let (state, config) = state_with_registered_agent("releve-disque");
    let shared = Arc::new(Mutex::new(state));
    let fact = DiskSpaceFact {
        volume: "/".to_string(),
        free_bytes: 47_300_000_000,
        observed_at_unix: 1_788_000_000,
    };

    assert!(
        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::DiskSpace { fact: fact.clone() },
            &shared,
        )
        .is_none()
    );

    let mut state = shared.lock().unwrap();
    let presence = state.presences.get("instance-1").unwrap();
    assert_eq!(
        presence.state, "connected",
        "le relevé ne pilote pas l'état"
    );
    assert_eq!(presence.disk_space, Some(fact.clone()));
    let agents = state.agent_infos();
    assert_eq!(agents[0].disk_space, Some(fact));
    drop(state);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn spec_024_trame_tmux_federee_historique_separe_protocole_et_canal() {
    let (mut state, config) = state_with_registered_agent("g11-tmux-federe");
    state.router.unregister_by_conn("conn-1");
    state.conn_instances.clear();
    state.presences.clear();

    let response = handle_register_with_channel(
        "conn-lab",
        2,
        "codex".to_string(),
        "89000000-0000-4000-8000-000000000108".to_string(),
        Some("lab-host".to_string()),
        Some("ssh-unix".to_string()),
        ChannelReport::Omitted,
        Some(PresenceMode::Tmux),
        Some("bridget:2.1".to_string()),
        Some("Linux".to_string()),
        Some("instance-lab".to_string()),
        Some("bridget".to_string()),
        false,
        Some(true),
        &mut state,
        false,
    );
    assert!(matches!(response, DaemonToWrapper::Registered { .. }));

    let info = state.agent_infos().pop().expect("agent fédéré visible");
    assert_eq!(info.transport, "tmux");
    assert_eq!(info.channel.as_deref(), Some("ssh-unix"));
    assert_eq!(info.mode, Some(PresenceMode::Tmux));
    assert_eq!(info.location.as_deref(), Some("bridget:2.1"));

    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn spec_024_protocole_et_canal_explicites_restent_independants() {
    let (mut state, config) = state_with_registered_agent("g11-explicite");
    state.router.unregister_by_conn("conn-1");
    state.conn_instances.clear();
    state.presences.clear();

    let response = handle_register_with_channel(
        "conn-natif",
        2,
        "fixture".to_string(),
        "89000000-0000-4000-8000-000000000109".to_string(),
        Some("lab-host".to_string()),
        Some("codex_app_server".to_string()),
        ChannelReport::Known("ssh-unix".to_string()),
        Some(PresenceMode::Cli),
        None,
        Some("Linux".to_string()),
        Some("instance-native".to_string()),
        Some("bridget".to_string()),
        false,
        Some(true),
        &mut state,
        false,
    );
    assert!(matches!(response, DaemonToWrapper::Registered { .. }));

    let info = state.agent_infos().pop().expect("agent natif visible");
    assert_eq!(info.transport, "codex_app_server");
    assert_eq!(info.channel.as_deref(), Some("ssh-unix"));
    assert_eq!(info.agent_type, "fixture");

    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn relais_ui_redemarre_reprend_lidentite_humaine_sans_rester_non_inscrit() {
    let root =
        std::env::temp_dir().join(format!("bridget-ui-human-takeover-{}", std::process::id()));
    let (mut state, config) = recovery_fixture_state(&root);

    let register = |connection: &str, instance: &str, state: &mut DaemonState| {
        handle_register_with_channel(
            connection,
            2,
            "ui".to_string(),
            UI_HUMAN_AGENT_ID.to_string(),
            Some("localhost".to_string()),
            None,
            ChannelReport::Omitted,
            Some(PresenceMode::Cli),
            None,
            Some("macOS".to_string()),
            Some(instance.to_string()),
            Some("bridget".to_string()),
            false,
            Some(false),
            state,
            false,
        )
    };

    assert!(matches!(
        register("ui-old", "bridget-ui-old", &mut state),
        DaemonToWrapper::Registered { .. }
    ));
    assert!(matches!(
        register("ui-new", "bridget-ui-new", &mut state),
        DaemonToWrapper::Registered { .. }
    ));
    assert_eq!(
        state
            .router
            .get_agent(UI_HUMAN_AGENT_ID)
            .map(|agent| agent.connection_id.as_str()),
        Some("ui-new"),
        "le nouveau relais doit redevenir émetteur humain inscrit"
    );
    assert!(state.presences.contains_key("bridget-ui-new"));
    assert!(!state.presences.contains_key("bridget-ui-old"));
    let _ = std::fs::remove_file(config.db_path);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn envoi_ui_avec_reponse_route_lhumain_canonique_sans_perdre_son_libelle() {
    let root =
        std::env::temp_dir().join(format!("bridget-ui-reply-routing-{}", std::process::id()));
    let (mut state, config) = recovery_fixture_state(&root);
    let target = "61d32f19-3e02-4cbe-91a9-37d38066b4ab";
    assert!(matches!(
        handle_register_with_channel(
            "target-agent",
            2,
            "codex".to_string(),
            target.to_string(),
            Some("cartae".to_string()),
            Some("codex_app_server".to_string()),
            ChannelReport::Omitted,
            Some(PresenceMode::Cli),
            None,
            Some("Linux".to_string()),
            Some("target-instance".to_string()),
            Some("bridget".to_string()),
            false,
            Some(true),
            &mut state,
            false,
        ),
        DaemonToWrapper::Registered { .. }
    ));
    assert!(matches!(
        handle_register_with_channel(
            "ui-human",
            2,
            "ui".to_string(),
            UI_HUMAN_AGENT_ID.to_string(),
            Some("localhost".to_string()),
            None,
            ChannelReport::Omitted,
            Some(PresenceMode::Cli),
            None,
            Some("macOS".to_string()),
            Some("bridget-ui-test".to_string()),
            Some("bridget".to_string()),
            false,
            Some(false),
            &mut state,
            false,
        ),
        DaemonToWrapper::Registered { .. }
    ));
    let shared = Arc::new(Mutex::new(state));
    negotiate_idempotent_client(&shared, "ui-client", "081_scope_ui_replyrouting", "humain");
    let mut message = BridgetMessage::new(UI_HUMAN_SENDER, target, "message humain");
    message.reply = true;

    assert!(matches!(
        handle_wrapper_message(
            "ui-client",
            WrapperToDaemon::SendIdempotent {
                message,
                message_id: "ui-human-reply".to_string(),
                issued_at: unix_now_secs(),
            },
            &shared,
        ),
        Some(DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::OutcomeUnknown { .. },
            ..
        })
    ));
    let state = shared.lock().unwrap();
    assert!(state.pending_replies.iter().any(|pending| {
        pending.msg_id == "ui-human-reply"
            && pending.from == UI_HUMAN_SENDER
            && pending.from_conn == "ui-human"
    }));
    assert_eq!(
        provider_display_name(&state, UI_HUMAN_SENDER).as_deref(),
        Some("Utilisateur"),
        "le fournisseur ne doit jamais recevoir le profil historique du libellé humain"
    );
    let _ = std::fs::remove_file(config.db_path);
    let _ = std::fs::remove_dir_all(root);
}

// La lecture et la publication partagent l'autorité d'un vrai Register,
// pas des entrées fabriquées dans conn_names/conn_instances.
fn register_artifact_fixture(state: &mut DaemonState, conn: &str, agent: &str) -> UnixStream {
    let (daemon, peer) = UnixStream::pair().unwrap();
    state.connections.insert(
        conn.to_owned(),
        Arc::new(Mutex::new(BufWriter::new(daemon))),
    );
    let registered = handle_register_with_channel(
        conn,
        2,
        "fixture".into(),
        agent.into(),
        Some("fixture".into()),
        Some("acp".into()),
        ChannelReport::Known("unix".into()),
        Some(PresenceMode::Cli),
        None,
        Some("test".into()),
        Some("instance-artifact".into()),
        None,
        false,
        Some(false),
        state,
        false,
    );
    assert!(
        matches!(registered, DaemonToWrapper::Registered { .. }),
        "{registered:?}"
    );
    peer
}

#[test]
fn publication_artefact_sans_projet_reste_dans_la_portee_de_conversation() {
    let root = std::env::temp_dir().join(format!(
        "bridget-artifact-unbound-{}-{}",
        std::process::id(),
        Uuid::new_v4()
    ));
    let (mut state, config) = recovery_fixture_state(&root);
    let agent_id = "a3d27a89-80d5-4e0f-9b84-cf5523ecb026";
    let _peer = register_artifact_fixture(&mut state, "conn-artifact", agent_id);
    state.artifact_root = config.db_path.with_extension("artifacts");
    let publication: ArtifactPublicationV1 = serde_json::from_str(include_str!(
        "../tests/fixtures/artifacts/chart-external-v1.json"
    ))
    .unwrap();
    let shared = Arc::new(Mutex::new(state));

    let response = handle_wrapper_message(
        "conn-artifact",
        WrapperToDaemon::ArtifactPublish {
            contract_version: ARTIFACT_CONTRACT_VERSION,
            canonical_publication: publication.canonical_bytes(),
        },
        &shared,
    );
    let Some(DaemonToWrapper::ArtifactPublicationResult {
        receipt_json: Some(receipt_json),
        refusal_code: None,
        ..
    }) = response
    else {
        panic!("publication de conversation attendue: {response:?}");
    };
    let receipt: crate::artifact_types::ArtifactReceiptV1 =
        serde_json::from_slice(&receipt_json).unwrap();
    assert_eq!(
        receipt.conversation_reference,
        format!(
            "conversation:{}:{agent_id}",
            artifact_scope_for_agent(agent_id)
        )
    );
    drop(shared);
    let _ = std::fs::remove_dir_all(root);
}

fn recovery_fixture_state(root: &std::path::Path) -> (DaemonState, DaemonConfig) {
    std::fs::create_dir_all(root).unwrap();
    let config = DaemonConfig {
        socket_path: root.join("bridget.sock"),
        db_path: root.join("bridget.db"),
        log_path: root.join("daemon.log"),
        circuit_breaker_window: 180,
        circuit_breaker_limit: 8,
        dedup_window: 180,
        quarantine_window: 3600,
        retention_days: 7,
    };
    let (managed_tx, _managed_rx) = mpsc::channel();
    let mut state = DaemonState::new(&config, managed_tx).unwrap();
    state.registry = AgentRegistry::from_json(
        &serde_json::json!({
            "agents": {
                "fixture": {
                    "command": "/bin/sh",
                    "protocol": "acp",
                    "forbidden_env": [],
                    "pass_env": []
                }
            }
        })
        .to_string(),
        root.join("agents.json"),
    )
    .unwrap();
    state.source_env = BTreeMap::from([
        ("HOME".to_string(), root.as_os_str().to_owned()),
        ("PATH".to_string(), OsString::from("/bin:/usr/bin")),
        ("USER".to_string(), OsString::from("tester")),
        ("LANG".to_string(), OsString::from("C")),
        ("TMPDIR".to_string(), OsString::from("/tmp")),
    ]);
    (state, config)
}

fn recovery_fixture_definition() -> bridget_transport::ResolvedAgentDefinition {
    AgentRegistry::from_json(
            r#"{"agents":{"fixture":{"command":"/bin/sh","protocol":"acp","forbidden_env":[],"pass_env":[]}}}"#,
            "/tmp/recovery-fixture-definition.json",
        )
        .unwrap()
        .resolved_definition("fixture")
        .unwrap()
}

fn recovery_native_codex_definition() -> bridget_transport::ResolvedAgentDefinition {
    // Même protocole et même matrice qu'un Codex natif, mais `/bin/sh`
    // pour que la reprise passe les gardes de commande. Sans
    // `execution_paths`, la préparation refuse le protocole et l'oracle
    // ne verrait jamais `who recovering`.
    AgentRegistry::from_json(
            r#"{"agents":{"fixture":{"command":"/bin/sh","args":["-c","model=\"gpt-5.6-terra\"","app-server"],"protocol":"codex_app_server","forbidden_env":[],"pass_env":[],"capabilities":{"execution_paths":["codex_app_server"],"models":{"gpt-5.6-terra":{}}}}}}"#,
            "/tmp/recovery-native-codex-definition.json",
        )
        .unwrap()
        .resolved_definition("fixture")
        .unwrap()
}

fn persist_connected_fixture_with_definition(
    state: &DaemonState,
    name: &str,
    command_id: &str,
    now: i64,
    definition: &bridget_transport::ResolvedAgentDefinition,
) -> SpawnLease {
    let order = FleetSpawnOrder {
        remote_placement: None,
        posture: None,
        agent_type: "fixture".to_string(),
        requested_name: Some(name.to_string()),
        cwd: PathBuf::from("/tmp"),
        persistent: true,
        command_id: command_id.to_string(),
        issued_at: now,
        deadline_at: now + 60,
        project: None,
        ownership: None,
    };
    let lease = match state.fleet.request_spawn(&order, now).unwrap() {
        crate::fleet::SpawnSubmission::Start(lease) => lease,
        other => panic!("réservation persistante attendue: {other:?}"),
    };
    state.fleet.mark_starting(&lease, now, definition).unwrap();
    state
        .fleet
        .register_connected(&lease, &lease.instance_id, now + 1)
        .unwrap();
    lease
}

fn recovery_daemon_config(root: &std::path::Path) -> DaemonConfig {
    let cache = root.join(".state");
    DaemonConfig {
        socket_path: cache.join("bridget.sock"),
        db_path: cache.join("bridget.db"),
        log_path: cache.join("daemon.log"),
        circuit_breaker_window: 180,
        circuit_breaker_limit: 8,
        dedup_window: 180,
        quarantine_window: 3600,
        retention_days: 7,
    }
}

struct RecoveryDaemon(Child);

impl std::ops::Deref for RecoveryDaemon {
    type Target = Child;
    fn deref(&self) -> &Child {
        &self.0
    }
}

impl std::ops::DerefMut for RecoveryDaemon {
    fn deref_mut(&mut self) -> &mut Child {
        &mut self.0
    }
}

impl Drop for RecoveryDaemon {
    fn drop(&mut self) {
        if matches!(self.0.try_wait(), Ok(None)) {
            // Exclusivement l'enfant créé ci-dessous, jamais le daemon
            // de travail. Le crash de l'oracle reste explicite dans le test.
            // SAFETY: kill ne déréférence aucun pointeur ; ses arguments sont des entiers.
            unsafe {
                libc::kill(self.0.id() as libc::pid_t, libc::SIGTERM);
            }
            let deadline = Instant::now() + Duration::from_secs(3);
            while matches!(self.0.try_wait(), Ok(None)) && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(10));
            }
            if matches!(self.0.try_wait(), Ok(None)) {
                let _ = self.0.kill();
            }
        }
        let _ = self.0.wait();
    }
}

fn spawn_recovery_daemon(root: &std::path::Path) -> RecoveryDaemon {
    RecoveryDaemon(
        Command::new(std::env::current_exe().unwrap())
            .arg("--exact")
            .arg("daemon::presence_tests::recovery_daemon_child")
            .arg("--ignored")
            .arg("--nocapture")
            .env_clear()
            .env(T908_DAEMON_CHILD_ENV, "1")
            .env(T908_ROOT_ENV, root)
            .env("HOME", root)
            .env("BRIDGET_HOME", root.join(".state"))
            .env("BRIDGET_SOCKET", root.join(".state/bridget.sock"))
            .env(
                "PATH",
                std::env::var_os("PATH").unwrap_or_else(|| "/usr/bin:/bin".into()),
            )
            .env("BRIDGET_T908_MANAGED_EXECUTABLE", managed_test_binary())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap(),
    )
}

fn wait_daemon_socket(socket: &std::path::Path) {
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if socket.exists() && UnixStream::connect(socket).is_ok() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "socket daemon absente après reprise"
        );
        thread::sleep(Duration::from_millis(20));
    }
}

fn daemon_request(socket: &std::path::Path, request: WrapperToDaemon) -> DaemonToWrapper {
    let stream = UnixStream::connect(socket).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(8)))
        .unwrap();
    let mut writer = BufWriter::new(stream.try_clone().unwrap());
    writeln!(writer, "{}", encode(&request).unwrap()).unwrap();
    writer.flush().unwrap();
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    decode(line.trim()).unwrap()
}

#[test]
#[ignore = "processus enfant lancé par un autre test (variable T908_DAEMON_CHILD_ENV) ; sans effet lancé seul"]
fn recovery_daemon_child() {
    if std::env::var(T908_DAEMON_CHILD_ENV).ok().as_deref() != Some("1") {
        return;
    }
    let root = PathBuf::from(std::env::var_os(T908_ROOT_ENV).unwrap());
    run(recovery_daemon_config(&root)).unwrap();
}

fn persist_connected_fixture(
    state: &DaemonState,
    name: &str,
    command_id: &str,
    now: i64,
) -> SpawnLease {
    persist_connected_fixture_with_definition(
        state,
        name,
        command_id,
        now,
        &recovery_fixture_definition(),
    )
}

#[test]
fn reprise_reserve_tous_les_noms_avant_ouverture_et_devient_visible() {
    let root = PathBuf::from(format!(
        "/tmp/bg908-reserve-{}-{}",
        std::process::id(),
        &Uuid::new_v4().simple().to_string()[..8]
    ));
    let (state, config) = recovery_fixture_state(&root);
    let now = unix_timestamp();
    let previous = persist_connected_fixture(
        &state,
        "89000000-0000-4000-8000-000000000116",
        "initial-alpha",
        now,
    );
    drop(state);

    let (mut reopened, _) = recovery_fixture_state(&root);
    let recoveries = reserve_managed_recoveries(&mut reopened, now + 2).unwrap();
    assert_eq!(recoveries.len(), 1);
    assert_eq!(
        recoveries[0].0.lease.name,
        "89000000-0000-4000-8000-000000000116"
    );
    assert!(recoveries[0].0.lease.generation > previous.generation);
    assert!(reopened.recovering);
    assert_eq!(reopened.recovery_commands.len(), 1);
    let info = &reopened.agent_infos()[0];
    assert_eq!(info.state, "recovering");
    assert_eq!(info.transport, "acp");
    assert_eq!(info.mode, Some(PresenceMode::Acp));
    drop(reopened);
    let _ = std::fs::remove_file(config.socket_path);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn reprise_natif_codex_affiche_protocole_fige_pas_unix_acp() {
    let root = PathBuf::from(format!(
        "/tmp/bg-recovering-native-{}-{}",
        std::process::id(),
        &Uuid::new_v4().simple().to_string()[..8]
    ));
    let (state, config) = recovery_fixture_state(&root);
    let now = unix_timestamp();
    persist_connected_fixture_with_definition(
        &state,
        "89000000-0000-4000-8000-000000000118",
        "initial-coder-natif",
        now,
        &recovery_native_codex_definition(),
    );
    drop(state);

    let (mut reopened, _) = recovery_fixture_state(&root);
    let recoveries = reserve_managed_recoveries(&mut reopened, now + 2).unwrap();
    assert_eq!(recoveries.len(), 1);
    let info = &reopened.agent_infos()[0];
    assert_eq!(info.state, "recovering");
    assert_eq!(
        info.transport, "codex_app_server",
        "who recovering doit exposer le protocole figé, pas unix"
    );
    assert_eq!(
        info.mode,
        Some(PresenceMode::Cli),
        "who recovering doit exposer MODE=cli, pas acp"
    );
    drop(reopened);
    let _ = std::fs::remove_file(config.socket_path);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn matrice_reprise_isole_echec_et_quota_sans_retry_infini() {
    let root = PathBuf::from(format!(
        "/tmp/bg908-matrix-{}-{}",
        std::process::id(),
        &Uuid::new_v4().simple().to_string()[..8]
    ));
    let (_, config) = recovery_fixture_state(&root);
    let desired = DesiredStateStore::at_path(desired_state_path(&config));
    let mut fleet = crate::desired_state::DesiredFleet::default();
    let _env = crate::fleet::TestFleetQuotaGuard::set(2);
    for index in 0..5_u64 {
        fleet.equipiers.insert(
            format!("agent-{index:02}"),
            crate::desired_state::DesiredEquipier {
                remote_placement: None,
                agent_type: if index == 0 { "inconnu" } else { "fixture" }.to_string(),
                cwd: PathBuf::from("/tmp"),
                command_id: format!("ancien-{index}"),
                generation: index + 1,
                created: index.to_string(),
                persistent: true,
                lifecycle_state: DesiredLifecycleState::Running,
                resolved_definition: (index != 0).then(recovery_fixture_definition),
                domain: None,
                project: None,
                agent_link: None,
                runtime_execution: None,
            },
        );
    }
    desired.persist(&fleet).unwrap();

    let (mut reopened, _) = recovery_fixture_state(&root);
    assert_eq!(reopened.fleet.quota(), 2);
    let recoveries = reserve_managed_recoveries(&mut reopened, unix_timestamp()).unwrap();
    assert_eq!(recoveries.len(), reopened.fleet.quota());
    let remaining = reopened.fleet.desired_fleet().unwrap();
    assert_eq!(remaining.equipiers.len(), 5);
    for name in ["agent-00", "agent-03", "agent-04"] {
        assert_eq!(
            remaining.equipiers[name].lifecycle_state,
            DesiredLifecycleState::Stopped,
            "une reprise refusée reste visible et ne sera pas retentée au boot"
        );
    }
    let report = crate::recovery_trace::load_report(&reopened.fleet.recovery_losses_path())
        .unwrap()
        .expect("trace des absents attendue");
    let by_name: std::collections::BTreeMap<_, _> = report
        .absents
        .iter()
        .map(|entry| (entry.name.as_str(), entry.reason.as_str()))
        .collect();
    assert_eq!(
        by_name.get("agent-00"),
        Some(&crate::recovery_trace::REASON_FROZEN_DEFINITION)
    );
    assert_eq!(
        by_name.get("agent-03"),
        Some(&crate::recovery_trace::REASON_QUOTA)
    );
    assert_eq!(
        by_name.get("agent-04"),
        Some(&crate::recovery_trace::REASON_QUOTA)
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn reprise_trace_les_non_persistants_nommes_absents() {
    let root = PathBuf::from(format!(
        "/tmp/bg-d20-ephemere-{}-{}",
        std::process::id(),
        &Uuid::new_v4().simple().to_string()[..8]
    ));
    let (_, config) = recovery_fixture_state(&root);
    let fleet_path = desired_state_path(&config);
    let roster = crate::recovery_trace::NamedRosterStore::at_path(
        crate::recovery_trace::roster_path(&fleet_path),
    );
    roster.remember(
        "cursor-ephemere".to_string(),
        crate::recovery_trace::NamedRosterEntry {
            agent_type: "cursor".to_string(),
            persistent: false,
            domain: Some("bridget".to_string()),
        },
    );

    let (mut reopened, _) = recovery_fixture_state(&root);
    let recoveries = reserve_managed_recoveries(&mut reopened, unix_timestamp()).unwrap();
    assert!(recoveries.is_empty());
    let report = crate::recovery_trace::load_report(&reopened.fleet.recovery_losses_path())
        .unwrap()
        .expect("trace des non-persistants");
    assert_eq!(report.absents.len(), 1);
    assert_eq!(report.absents[0].name, "cursor-ephemere");
    assert_eq!(
        report.absents[0].reason,
        crate::recovery_trace::REASON_NON_PERSISTENT
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn reprise_trace_pertes_mixtes_au_meme_redemarrage() {
    let root = PathBuf::from(format!(
        "/tmp/bg-d20-mixte-{}-{}",
        std::process::id(),
        &Uuid::new_v4().simple().to_string()[..8]
    ));
    let (_, config) = recovery_fixture_state(&root);
    let desired = DesiredStateStore::at_path(desired_state_path(&config));
    let mut fleet = crate::desired_state::DesiredFleet::default();
    let _env = crate::fleet::TestFleetQuotaGuard::set(2);
    for index in 0..5_u64 {
        fleet.equipiers.insert(
            format!("agent-{index:02}"),
            crate::desired_state::DesiredEquipier {
                remote_placement: None,
                agent_type: if index == 0 { "inconnu" } else { "fixture" }.to_string(),
                cwd: PathBuf::from("/tmp"),
                command_id: format!("ancien-{index}"),
                generation: index + 1,
                created: index.to_string(),
                persistent: true,
                lifecycle_state: DesiredLifecycleState::Running,
                resolved_definition: (index != 0).then(recovery_fixture_definition),
                domain: None,
                project: None,
                agent_link: None,
                runtime_execution: None,
            },
        );
    }
    desired.persist(&fleet).unwrap();
    crate::recovery_trace::NamedRosterStore::at_path(crate::recovery_trace::roster_path(
        &desired_state_path(&config),
    ))
    .remember(
        "cursor-ephemere".to_string(),
        crate::recovery_trace::NamedRosterEntry {
            agent_type: "cursor".to_string(),
            persistent: false,
            domain: Some("bridget".to_string()),
        },
    );

    let (mut reopened, _) = recovery_fixture_state(&root);
    let _ = reserve_managed_recoveries(&mut reopened, unix_timestamp()).unwrap();
    let report = crate::recovery_trace::load_report(&reopened.fleet.recovery_losses_path())
        .unwrap()
        .expect("trace mixte");
    let by_name: std::collections::BTreeMap<_, _> = report
        .absents
        .iter()
        .map(|entry| (entry.name.as_str(), entry.reason.as_str()))
        .collect();
    assert_eq!(
        by_name.get("agent-00"),
        Some(&crate::recovery_trace::REASON_FROZEN_DEFINITION)
    );
    assert_eq!(
        by_name.get("agent-03"),
        Some(&crate::recovery_trace::REASON_QUOTA)
    );
    assert_eq!(
        by_name.get("agent-04"),
        Some(&crate::recovery_trace::REASON_QUOTA)
    );
    assert_eq!(
        by_name.get("cursor-ephemere"),
        Some(&crate::recovery_trace::REASON_NON_PERSISTENT)
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn reprise_retire_une_trace_residuelle_si_zero_perte() {
    let root = PathBuf::from(format!(
        "/tmp/bg-d20-stale-{}-{}",
        std::process::id(),
        &Uuid::new_v4().simple().to_string()[..8]
    ));
    let (_, config) = recovery_fixture_state(&root);
    let desired = DesiredStateStore::at_path(desired_state_path(&config));
    let mut fleet = crate::desired_state::DesiredFleet::default();
    fleet.equipiers.insert(
        "agent-ok".to_string(),
        crate::desired_state::DesiredEquipier {
            remote_placement: None,
            agent_type: "fixture".to_string(),
            cwd: PathBuf::from("/tmp"),
            command_id: "ancien-ok".to_string(),
            generation: 1,
            created: "1".to_string(),
            persistent: true,
            lifecycle_state: DesiredLifecycleState::Running,
            resolved_definition: Some(recovery_fixture_definition()),
            domain: None,
            project: None,
            agent_link: None,
            runtime_execution: None,
        },
    );
    desired.persist(&fleet).unwrap();
    let losses_path = crate::recovery_trace::report_path(&desired_state_path(&config));
    crate::recovery_trace::persist_report(
        &losses_path,
        1,
        vec![crate::recovery_trace::RecoveryLossEntry {
            name: "fantome".to_string(),
            reason: crate::recovery_trace::REASON_QUOTA.to_string(),
            detail: None,
        }],
    )
    .unwrap();
    assert!(losses_path.exists());

    let (mut reopened, _) = recovery_fixture_state(&root);
    let recoveries = reserve_managed_recoveries(&mut reopened, unix_timestamp()).unwrap();
    assert_eq!(recoveries.len(), 1);
    assert!(
        !reopened.fleet.recovery_losses_path().exists(),
        "zéro perte doit retirer le fichier, pas seulement le vider"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn roster_illisible_n_empeche_pas_la_reprise() {
    let root = PathBuf::from(format!(
        "/tmp/bg-d20-roster-ko-{}-{}",
        std::process::id(),
        &Uuid::new_v4().simple().to_string()[..8]
    ));
    let (_, config) = recovery_fixture_state(&root);
    let desired = DesiredStateStore::at_path(desired_state_path(&config));
    let mut fleet = crate::desired_state::DesiredFleet::default();
    fleet.equipiers.insert(
        "agent-ok".to_string(),
        crate::desired_state::DesiredEquipier {
            remote_placement: None,
            agent_type: "fixture".to_string(),
            cwd: PathBuf::from("/tmp"),
            command_id: "ancien-ok".to_string(),
            generation: 1,
            created: "1".to_string(),
            persistent: true,
            lifecycle_state: DesiredLifecycleState::Running,
            resolved_definition: Some(recovery_fixture_definition()),
            domain: None,
            project: None,
            agent_link: None,
            runtime_execution: None,
        },
    );
    desired.persist(&fleet).unwrap();
    std::fs::write(
        crate::recovery_trace::roster_path(&desired_state_path(&config)),
        "ce n'est pas du json",
    )
    .unwrap();

    let (mut reopened, _) = recovery_fixture_state(&root);
    let recoveries = reserve_managed_recoveries(&mut reopened, unix_timestamp())
        .expect("un roster illisible ne doit pas avorter la reprise");
    assert_eq!(recoveries.len(), 1);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn persistant_disparu_de_fleet_json_est_trace() {
    let root = PathBuf::from(format!(
        "/tmp/bg-d20-silence-{}-{}",
        std::process::id(),
        &Uuid::new_v4().simple().to_string()[..8]
    ));
    let (_, config) = recovery_fixture_state(&root);
    crate::recovery_trace::NamedRosterStore::at_path(crate::recovery_trace::roster_path(
        &desired_state_path(&config),
    ))
    .remember(
        "agent-z".to_string(),
        crate::recovery_trace::NamedRosterEntry {
            agent_type: "fixture".to_string(),
            persistent: true,
            domain: Some("bridget".to_string()),
        },
    );

    let (mut reopened, _) = recovery_fixture_state(&root);
    let recoveries = reserve_managed_recoveries(&mut reopened, unix_timestamp()).unwrap();
    assert!(recoveries.is_empty());
    let report = crate::recovery_trace::load_report(&reopened.fleet.recovery_losses_path())
        .unwrap()
        .expect("perte roster/fleet");
    assert_eq!(report.absents.len(), 1);
    assert_eq!(report.absents[0].name, "agent-z");
    assert_eq!(
        report.absents[0].reason,
        crate::recovery_trace::REASON_ABSENT_FROM_FLEET
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn reprise_passe_quand_bridget_fleet_quota_est_pose() {
    let root = PathBuf::from(format!(
        "/tmp/bg-quota-env-{}-{}",
        std::process::id(),
        &Uuid::new_v4().simple().to_string()[..8]
    ));
    let (_, config) = recovery_fixture_state(&root);
    let desired = DesiredStateStore::at_path(desired_state_path(&config));
    let mut fleet = crate::desired_state::DesiredFleet::default();
    for index in 0..10_u64 {
        fleet.equipiers.insert(
            format!("agent-{index:02}"),
            crate::desired_state::DesiredEquipier {
                remote_placement: None,
                agent_type: "fixture".to_string(),
                cwd: PathBuf::from("/tmp"),
                command_id: format!("ancien-{index}"),
                generation: index + 1,
                created: index.to_string(),
                persistent: true,
                lifecycle_state: DesiredLifecycleState::Running,
                resolved_definition: Some(recovery_fixture_definition()),
                domain: None,
                project: None,
                agent_link: None,
                runtime_execution: None,
            },
        );
    }
    desired.persist(&fleet).unwrap();

    let _env = crate::fleet::TestFleetQuotaGuard::set(16);
    let (mut reopened, _) = recovery_fixture_state(&root);
    assert_eq!(reopened.fleet.quota(), 16);
    let recoveries = reserve_managed_recoveries(&mut reopened, unix_timestamp()).unwrap();
    assert_eq!(recoveries.len(), 10);
    let remaining = reopened.fleet.desired_fleet().unwrap();
    assert_eq!(remaining.equipiers.len(), 10);
    assert!(remaining.equipiers.contains_key("agent-09"));
    assert!(
        crate::recovery_trace::load_report(&reopened.fleet.recovery_losses_path())
            .unwrap()
            .is_none(),
        "zéro perte ne doit pas laisser de trace"
    );
    assert!(
        !reopened.fleet.recovery_losses_path().exists(),
        "zéro perte retire le fichier"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn reprise_en_conflit_avec_un_wrapper_terminal_echoue_sans_retry() {
    let root = PathBuf::from(format!(
        "/tmp/bg908-conflict-{}-{}",
        std::process::id(),
        &Uuid::new_v4().simple().to_string()[..8]
    ));
    let (_, config) = recovery_fixture_state(&root);
    let desired = DesiredStateStore::at_path(desired_state_path(&config));
    let mut fleet = crate::desired_state::DesiredFleet::default();
    fleet.equipiers.insert(
        "89000000-0000-4000-8000-000000000116".to_string(),
        crate::desired_state::DesiredEquipier {
            remote_placement: None,
            agent_type: "fixture".to_string(),
            cwd: PathBuf::from("/tmp"),
            command_id: "ancien-alpha".to_string(),
            generation: 1,
            created: "initial".to_string(),
            persistent: true,
            lifecycle_state: DesiredLifecycleState::Running,
            resolved_definition: Some(recovery_fixture_definition()),
            domain: None,
            project: None,
            agent_link: None,
            runtime_execution: None,
        },
    );
    desired.persist(&fleet).unwrap();

    let (mut state, _) = recovery_fixture_state(&root);
    let recoveries = reserve_managed_recoveries(&mut state, unix_timestamp()).unwrap();
    assert_eq!(recoveries.len(), 1);
    let lease = recoveries[0].0.lease.clone();
    let shared = Arc::new(Mutex::new(state));

    assert!(matches!(
        handle_wrapper_message(
            "terminal-alpha",
            WrapperToDaemon::Register {
                agent_type: "fixture".to_string(),
                identity_version: 2,
                agent_id: "89000000-0000-4000-8000-000000000116".to_string(),
                host: Some("local".to_string()),
                transport: Some("tmux".to_string()),
                channel: Some("unix".to_string()).into(),
                mode: Some(PresenceMode::Tmux),
                location: Some("fixture:0.1".to_string()),
                os: Some("test".to_string()),
                instance_id: None,
                domain: None,
                turn_in_progress: false,
                journal_available: None,
                turn_attest: false,
                queue_fallback: false,
            },
            &shared,
        ),
        Some(DaemonToWrapper::Registered { credential: _, agent_id: ref name }) if name == "89000000-0000-4000-8000-000000000116"
    ));
    assert!(matches!(
        handle_wrapper_message(
            "managed-alpha",
            WrapperToDaemon::Register {
                agent_type: "fixture".to_string(),
                identity_version: 2,
                agent_id: "89000000-0000-4000-8000-000000000116".to_string(),
                host: Some("local".to_string()),
                transport: Some("acp".to_string()),
                channel: Some("unix".to_string()).into(),
                mode: Some(PresenceMode::Acp),
                location: None,
                os: Some("test".to_string()),
                instance_id: Some(lease.instance_id.clone()),
                domain: None,
                turn_in_progress: false,
                journal_available: None,
                turn_attest: false,
                queue_fallback: false,
            },
            &shared,
        ),
        Some(DaemonToWrapper::Nack { ref reason, .. }) if reason.contains("déjà")
    ));

    {
        let state = shared.lock().unwrap();
        state
            .fleet
            .fail(&lease, "name_active", "nom déjà actif")
            .unwrap();
    }
    let (event_tx, event_rx) = mpsc::channel();
    event_tx
        .send(ManagedSupervisorEvent::Failed {
            lease: lease.clone(),
            kind: "name_active".to_string(),
            reason: "nom déjà actif".to_string(),
            conn_id: None,
        })
        .unwrap();
    drain_managed_events(&shared, &event_rx);

    let state = shared.lock().unwrap();
    assert_eq!(
        state
            .fleet
            .desired_entry("89000000-0000-4000-8000-000000000116")
            .unwrap()
            .expect("la reprise échouée reste gérée")
            .lifecycle_state,
        DesiredLifecycleState::Stopped
    );
    assert!(!state.recovering);
    assert_eq!(
        state
            .router
            .get_agent("89000000-0000-4000-8000-000000000116")
            .unwrap()
            .connection_id,
        "terminal-alpha"
    );
    drop(state);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn stop_du_second_retablissement_gagne_avant_son_bootstrap() {
    let root = PathBuf::from(format!(
        "/tmp/bg908-stop-{}-{}",
        std::process::id(),
        &Uuid::new_v4().simple().to_string()[..8]
    ));
    let (_, config) = recovery_fixture_state(&root);
    let desired = DesiredStateStore::at_path(desired_state_path(&config));
    let mut fleet = crate::desired_state::DesiredFleet::default();
    for name in [
        "89000000-0000-4000-8000-000000000116",
        "89000000-0000-4000-8000-000000000117",
    ] {
        fleet.equipiers.insert(
            name.to_string(),
            crate::desired_state::DesiredEquipier {
                remote_placement: None,
                agent_type: "fixture".to_string(),
                cwd: PathBuf::from("/tmp"),
                command_id: format!("ancien-{name}"),
                generation: 1,
                created: "initial".to_string(),
                persistent: true,
                lifecycle_state: DesiredLifecycleState::Running,
                resolved_definition: Some(recovery_fixture_definition()),
                domain: None,
                project: None,
                agent_link: None,
                runtime_execution: None,
            },
        );
    }
    desired.persist(&fleet).unwrap();
    let (mut state, _) = recovery_fixture_state(&root);
    let (managed_tx, managed_rx) = mpsc::channel();
    state.managed_tx = managed_tx.clone();
    let recoveries = reserve_managed_recoveries(&mut state, unix_timestamp()).unwrap();
    assert_eq!(recoveries.len(), 2);
    let beta = state
        .managed_spawns
        .values()
        .find(|record| record.lease.name == "89000000-0000-4000-8000-000000000117")
        .unwrap();
    let completion = Arc::clone(&beta.stop);
    let beta_command = beta.lease.command_id.clone();
    let alpha_command = state
        .managed_spawns
        .values()
        .find(|record| record.lease.name == "89000000-0000-4000-8000-000000000116")
        .unwrap()
        .lease
        .command_id
        .clone();

    let gate = root.join("bootstrap-gate");
    let gate_ready = root.join("bootstrap-gate.ready");
    let gate_once = root.join("bootstrap-gate.once");
    let gate_c = CString::new(gate.as_os_str().as_bytes()).unwrap();
    // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
    assert_eq!(unsafe { libc::mkfifo(gate_c.as_ptr(), 0o600) }, 0);
    let shim = root.join("managed-shim.sh");
    std::fs::write(
            &shim,
            format!(
                "#!/bin/sh\nif [ \"$1\" = managed-bootstrap ]; then\n  if mkdir '{}' 2>/dev/null; then\n    : > '{}'\n    IFS= read -r release < '{}' || exit 1\n  fi\n  exec '{}' \"$@\"\nfi\ntrap 'exit 0' TERM\nwhile :; do sleep 1; done\n",
                gate_once.display(),
                gate_ready.display(),
                gate.display(),
                managed_test_binary().display(),
            ),
        )
        .unwrap();
    std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o700)).unwrap();

    let shared = Arc::new(Mutex::new(state));
    let (event_tx, event_rx) = mpsc::channel();
    let _supervisor = ManagedSupervisorGuard::from_existing_sender(
        managed_tx,
        managed_rx,
        Arc::clone(&shared.lock().unwrap().fleet),
        &config,
        event_tx,
        Some(shim),
    );
    for (prepared, stop) in recoveries {
        shared
            .lock()
            .unwrap()
            .managed_tx
            .send(ManagedSupervisorCommand::Start { prepared, stop })
            .unwrap();
    }
    let gate_deadline = Instant::now() + Duration::from_secs(5);
    while !gate_ready.exists() {
        assert!(
            Instant::now() < gate_deadline,
            "le premier bootstrap n'a pas atteint la barrière"
        );
        thread::yield_now();
    }

    let caller_state = Arc::clone(&shared);
    let caller = thread::spawn(move || {
        handle_wrapper_message(
            "control",
            WrapperToDaemon::StopOrder {
                agent_id: "89000000-0000-4000-8000-000000000117".to_string(),
                command_id: "stop-beta-recovery".to_string(),
            },
            &caller_state,
        )
    });
    let deadline = Instant::now() + Duration::from_secs(1);
    while !completion.is_actionable() {
        assert!(
            Instant::now() < deadline,
            "stop non visible avant bootstrap"
        );
        thread::yield_now();
    }

    std::fs::OpenOptions::new()
        .write(true)
        .open(&gate)
        .unwrap()
        .write_all(b"release\n")
        .unwrap();
    let stop_deadline = Instant::now() + Duration::from_secs(8);
    let result = loop {
        drain_managed_events(&shared, &event_rx);
        if caller.is_finished() {
            break caller.join().unwrap();
        }
        assert!(
            Instant::now() < stop_deadline,
            "le stop du second rétablissement n'a pas abouti"
        );
        thread::sleep(Duration::from_millis(10));
    };
    assert!(matches!(
        result,
        Some(DaemonToWrapper::StopResult {
            outcome: StopOutcome::Stopped | StopOutcome::StoppedForced { .. },
            ..
        })
    ));
    let state = shared.lock().unwrap();
    assert_eq!(
        state
            .fleet
            .desired_entry("89000000-0000-4000-8000-000000000117")
            .unwrap()
            .expect("beta reste géré après son arrêt")
            .lifecycle_state,
        DesiredLifecycleState::Stopped
    );
    assert!(!state.recovery_commands.contains(&beta_command));
    assert!(state.recovering, "alpha reste encore en reprise");
    drop(state);

    let alpha = Arc::clone(&shared.lock().unwrap().managed_spawns[&alpha_command].stop);
    alpha.mark_handshake_complete();
    let alpha_done = alpha.request();
    let cleanup_deadline = Instant::now() + Duration::from_secs(8);
    loop {
        drain_managed_events(&shared, &event_rx);
        if alpha_done.try_recv().is_ok() {
            break;
        }
        assert!(
            Instant::now() < cleanup_deadline,
            "nettoyage du premier rétablissement incomplet"
        );
        thread::sleep(Duration::from_millis(10));
    }
    drop(shared);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn sigkill_daemon_reconcilie_l_ancien_groupe_avant_une_reprise_unique() {
    let root = PathBuf::from(format!(
        "/tmp/bg908-e2e-{}-{}",
        std::process::id(),
        &Uuid::new_v4().simple().to_string()[..8]
    ));
    let config = recovery_daemon_config(&root);
    crate::environment::ensure_private_directory(config.db_path.parent().unwrap()).unwrap();
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(
        Command::new("git")
            .args(["init", "-q", root.to_str().unwrap()])
            .status()
            .unwrap()
            .success()
    );
    std::fs::write(root.join("wip.txt"), "base\n").unwrap();
    assert!(
        Command::new("git")
            .args([
                "-C",
                root.to_str().unwrap(),
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "add",
                "wip.txt"
            ])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args([
                "-C",
                root.to_str().unwrap(),
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "commit",
                "-qm",
                "base"
            ])
            .status()
            .unwrap()
            .success()
    );
    std::fs::write(root.join("wip.txt"), "non commité\n").unwrap();
    let registry_path = root.join(".state/agents.json");
    std::fs::create_dir_all(registry_path.parent().unwrap()).unwrap();
    let adapter = root.join("adapter.sh");
    let changed_adapter = root.join("adapter-changed.sh");
    let old_runs = root.join("old-adapter-runs");
    let resume_prompts = root.join("resume-prompts");
    let changed_runs = root.join("changed-adapter-runs");
    std::fs::write(
            &adapter,
            format!(
                "#!/bin/sh\necho old >> {}\nread initialize || exit 1\necho '{{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{{\"protocolVersion\":1}}}}'\nread session || exit 1\necho '{{\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{{\"sessionId\":\"recovery-session\"}}}}'\nwhile read line; do case \"$line\" in *session/prompt*) echo \"$line\" >> {} ;; esac; done\n",
                old_runs.display(), resume_prompts.display()
            ),
        )
        .unwrap();
    std::fs::set_permissions(&adapter, std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::write(
        &changed_adapter,
        format!(
            "#!/bin/sh\necho changed >> {}\nexit 91\n",
            changed_runs.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&changed_adapter, std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::write(
        &registry_path,
        serde_json::to_vec(&serde_json::json!({
            "agents": {
                "fixture": {
                    "command": adapter,
                    "protocol": "acp",
                    "permissions": "allow",
                    "forbidden_env": [],
                    "pass_env": [],
                    "queue_capacity": 2,
                    "notify_timeout_secs": 1
                }
            }
        }))
        .unwrap(),
    )
    .unwrap();
    std::fs::set_permissions(&registry_path, std::fs::Permissions::from_mode(0o600)).unwrap();

    // Précondition durable du scénario de reprise, pas contournement
    // d'une garde au lancement : le shell témoin n'offre pas le mode
    // découverte. La mutation humaine via CLI est éprouvée séparément
    // par core_089_host_session_test ; ici on prépare l'état historique.
    {
        let store = Store::open(&config.db_path).unwrap();
        let initial = crate::referent_control::read(store.connection()).unwrap();
        crate::referent_control::set(
            store.connection(),
            crate::referent_control::ControlMutation {
                command_id: "recovery-fixture-complete",
                expected_generation: initial.generation,
                paused: None,
                auto_objectives_cap: None,
                reason: None,
                actor: "test",
                now: unix_timestamp(),
                agent_posture: Some(bridget_transport::protocol::AgentPosture::Complete),
                auto_reassignment: None,
            },
        )
        .unwrap()
        .unwrap();
    }
    let mut first_daemon = spawn_recovery_daemon(&root);
    wait_daemon_socket(&config.socket_path);
    let now = unix_timestamp();
    let accepted = daemon_request(
        &config.socket_path,
        WrapperToDaemon::SpawnOrder {
            posture: None,
            agent_type: "fixture".to_string(),
            agent_id: Some("89000000-0000-4000-8000-000000000129".to_string()),
            cwd: root.to_string_lossy().to_string(),
            persistent: true,
            command_id: "initial-persistent".to_string(),
            issued_at: now,
            deadline_at: now + 20,
            project: None,
            ownership: None,
        },
    );
    assert!(
        matches!(
            &accepted,
            DaemonToWrapper::SpawnAccepted { agent_id: name, definition: Some(definition), .. }
                if name == "89000000-0000-4000-8000-000000000129"
                    && definition.command == adapter.to_string_lossy()
                    && definition.args.is_empty()
                    && definition.forbidden_env.is_empty()
                    && definition.digest.len() == 64
        ),
        "spawn réel refusé : {accepted:?}"
    );
    let marker_store =
        ManagedMarkerStore::at_directory(config.db_path.parent().unwrap().join("managed"));
    let prompt_deadline = Instant::now() + Duration::from_secs(4);
    while std::fs::read_to_string(&resume_prompts)
        .unwrap_or_default()
        .lines()
        .count()
        < 1
    {
        assert!(
            Instant::now() < prompt_deadline,
            "première carte de reprise absente"
        );
        thread::sleep(Duration::from_millis(10));
    }
    let first_marker = marker_store
        .load("89000000-0000-4000-8000-000000000129")
        .unwrap();
    assert!(crate::managed_process::group_exists(first_marker.pgid).unwrap());

    assert_ne!(first_daemon.id(), 0);
    assert_eq!(
        // SAFETY: kill ne déréférence aucun pointeur ; ses arguments sont des entiers.
        unsafe { libc::kill(first_daemon.id() as libc::pid_t, libc::SIGKILL) },
        0
    );
    let _ = first_daemon.wait().unwrap();

    // Le registre mutable dérive après le crash. La reprise doit pourtant
    // exécuter l'ancien adaptateur figé et republier la même preuve.
    std::fs::write(
        &registry_path,
        serde_json::to_vec(&serde_json::json!({
            "agents": {
                "fixture": {
                    "command": changed_adapter,
                    "args": ["new-runtime-meaning"],
                    "protocol": "acp",
                    "permissions": "deny",
                    "forbidden_env": ["CHANGED_KEY"],
                    "pass_env": ["LANG"],
                    "queue_capacity": 7,
                    "notify_timeout_secs": 9,
                    "mcp": {"interactive": "claude", "acp_session": true}
                }
            }
        }))
        .unwrap(),
    )
    .unwrap();
    std::fs::set_permissions(&registry_path, std::fs::Permissions::from_mode(0o600)).unwrap();

    let mut restarted = spawn_recovery_daemon(&root);
    wait_daemon_socket(&config.socket_path);
    let deadline = Instant::now() + Duration::from_secs(12);
    let agents = loop {
        if let DaemonToWrapper::AgentList { agents } =
            daemon_request(&config.socket_path, WrapperToDaemon::ListAgents)
            && agents.len() == 1
            && agents[0].agent_id == "89000000-0000-4000-8000-000000000129"
            && agents[0].state == "connected"
        {
            break agents;
        }
        assert!(
            Instant::now() < deadline,
            "reprise persistante non connectée"
        );
        thread::sleep(Duration::from_millis(25));
    };
    assert_eq!(agents.len(), 1, "une seule génération doit être visible");
    assert!(!crate::managed_process::group_exists(first_marker.pgid).unwrap());
    let second_marker = marker_store
        .load("89000000-0000-4000-8000-000000000129")
        .unwrap();
    assert_ne!(second_marker.instance_id, first_marker.instance_id);
    assert_ne!(second_marker.pgid, first_marker.pgid);
    let execution_deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let executions = std::fs::read_to_string(&old_runs).unwrap().lines().count();
        if executions == 2 {
            break;
        }
        assert!(
            Instant::now() < execution_deadline,
            "la commande figée doit être exécutée une fois avant et une fois après le crash"
        );
        thread::yield_now();
    }
    let prompt_deadline = Instant::now() + Duration::from_secs(4);
    let prompts = loop {
        let prompts = std::fs::read_to_string(&resume_prompts).unwrap_or_default();
        if prompts.lines().count() >= 2 {
            break prompts;
        }
        assert!(
            Instant::now() < prompt_deadline,
            "carte de reprise post-crash absente"
        );
        thread::sleep(Duration::from_millis(10));
    };
    assert!(prompts.contains("Carte de reprise Bridget"));
    assert!(prompts.contains("Identité figée"));
    assert!(prompts.contains("definition_digest="));
    assert!(prompts.contains("wip.txt"));
    assert!(
        !changed_runs.exists(),
        "le registre modifié ne doit jamais piloter la génération reprise"
    );
    assert!(matches!(
        daemon_request(
            &config.socket_path,
            WrapperToDaemon::SpawnOrder {
                posture: None,
                agent_type: "fixture".to_string(),
                agent_id: Some("89000000-0000-4000-8000-000000000129".to_string()),
                cwd: root.to_string_lossy().to_string(),
                persistent: true,
                command_id: "initial-persistent".to_string(),
                issued_at: now,
                deadline_at: now + 20,
                project: None,
                ownership: None,
            }
        ),
        DaemonToWrapper::SpawnAccepted { definition: Some(definition), .. }
            if definition.command == adapter.to_string_lossy()
                && definition.permissions == "allow"
                && definition.queue_capacity == 2
                && definition.notify_timeout_secs == 1
    ));

    assert!(matches!(
        daemon_request(
            &config.socket_path,
            WrapperToDaemon::StopOrder {
                agent_id: "89000000-0000-4000-8000-000000000129".to_string(),
                command_id: "stop-after-recovery".to_string(),
            }
        ),
        DaemonToWrapper::StopResult {
            outcome: StopOutcome::Stopped | StopOutcome::StoppedForced { .. },
            ..
        }
    ));
    assert_eq!(
        // SAFETY: kill ne déréférence aucun pointeur ; ses arguments sont des entiers.
        unsafe { libc::kill(restarted.id() as libc::pid_t, libc::SIGTERM) },
        0
    );
    let status = restarted.wait().unwrap();
    assert!(
        status.success() || status.code().is_none(),
        "arret daemon inattendu: {status:?}"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn connexion_attach_refuse_un_message_reserve_au_wrapper() {
    let (state, config) = state_with_registered_agent("attach-role");
    let shared = Arc::new(Mutex::new(state));

    let accepted = handle_wrapper_message(
        "attach-1",
        WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Attach,
        },
        &shared,
    );
    assert!(matches!(
        accepted,
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));

    let refusal = handle_wrapper_message(
        "attach-1",
        WrapperToDaemon::Runtime {
            agent: "89000000-0000-4000-8000-000000000102".to_string(),
            model: "gpt-5.5".to_string(),
            effort: None,
            source: bridget_transport::protocol::RuntimeSource::Declared,
        },
        &shared,
    );
    assert!(matches!(
        refusal,
        Some(DaemonToWrapper::AttachRejected {
            reason: AttachRefusal::MessageOutsideAttachRole,
            ..
        })
    ));
    assert_eq!(shared.lock().unwrap().router.list_agents().len(), 1);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn premier_message_wrapper_fige_le_role_avant_un_handshake_tardif() {
    let (state, config) = state_with_registered_agent("role-fige");
    let shared = Arc::new(Mutex::new(state));

    let first = handle_wrapper_message(
        "conn-role",
        WrapperToDaemon::Runtime {
            agent: "inconnu".to_string(),
            model: "modele".to_string(),
            effort: None,
            source: bridget_transport::protocol::RuntimeSource::Declared,
        },
        &shared,
    );
    assert!(matches!(first, Some(DaemonToWrapper::Nack { .. })));
    assert_eq!(
        shared.lock().unwrap().connection_roles.get("conn-role"),
        Some(&ConnectionRole::Wrapper)
    );

    let handshake = handle_wrapper_message(
        "conn-role",
        WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Attach,
        },
        &shared,
    );
    assert!(matches!(
        handshake,
        Some(DaemonToWrapper::AttachRejected {
            reason: AttachRefusal::MessageOutsideAttachRole,
            ..
        })
    ));
    assert_eq!(
        shared.lock().unwrap().connection_roles.get("conn-role"),
        Some(&ConnectionRole::Wrapper)
    );
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn requete_delegate_distingue_v1_historique_et_v2_ciblee() {
    let delegate = |review_target| bridget_transport::protocol::ServiceRequestPayload::Delegate {
        goal: "relire le lot".to_string(),
        review_target,
        explicit_target: None,
        required_tags: Vec::new(),
        duration: bridget_transport::protocol::GuichetDurationClass::Courte,
        suite: bridget_transport::protocol::ServiceSuiteDeclaration::Aucune,
        depends_on: Vec::new(),
        references: Vec::new(),
        origin: None,
        focus: None,
    };
    let target = bridget_transport::protocol::ReviewTarget {
        target_ref: "origin/session-047-verdict-tete-reecrite".to_string(),
        expected_head: "a".repeat(40),
    };
    let operation = bridget_transport::protocol::ServiceRequestOperation::Delegate;

    assert!(guichet_request_is_valid(
        SERVICE_CONTRACT_VERSION,
        "request-v1",
        operation,
        &delegate(None),
    ));
    assert!(guichet_request_is_valid(
        REVIEW_DELEGATE_CONTRACT_VERSION,
        "request-v2",
        operation,
        &delegate(Some(target.clone())),
    ));
    assert!(!guichet_request_is_valid(
        SERVICE_CONTRACT_VERSION,
        "request-v1-cible-interdite",
        operation,
        &delegate(Some(target)),
    ));
    assert!(!guichet_request_is_valid(
        REVIEW_DELEGATE_CONTRACT_VERSION,
        "request-v2-sans-cible",
        operation,
        &delegate(None),
    ));
    assert!(!guichet_request_is_valid(
        REVIEW_DELEGATE_CONTRACT_VERSION,
        "request-v2-autre-operation",
        bridget_transport::protocol::ServiceRequestOperation::RegistreAdd,
        &bridget_transport::protocol::ServiceRequestPayload::RegistreAdd {
            line: "kind=add id=constat-1".to_string(),
        },
    ));

    let request = |version, payload| WrapperToDaemon::ServiceRequest {
        version,
        issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
        request_id: format!("request-version-{version}"),
        issued_at: unix_timestamp(),
        from: "jc2".to_string(),
        to: "guichet".to_string(),
        operation,
        payload,
    };
    let (state, config) = state_with_registered_agent("delegate-review-version");
    let shared = Arc::new(Mutex::new(state));
    let unsupported = handle_wrapper_message(
        "delegate-review-version",
        request(
            3,
            delegate(Some(bridget_transport::protocol::ReviewTarget {
                target_ref: "origin/main".to_string(),
                expected_head: "a".repeat(40),
            })),
        ),
        &shared,
    );
    assert!(
        matches!(
            &unsupported,
            Some(DaemonToWrapper::ServiceRejected {
                reason: ServiceRefusal::UnsupportedVersion { supported_versions }
            }) if supported_versions == &vec![
                SERVICE_CONTRACT_VERSION,
                REVIEW_DELEGATE_CONTRACT_VERSION,
            ]
        ),
        "réponse inattendue : {unsupported:?}"
    );
    assert!(matches!(
        handle_wrapper_message(
            "delegate-review-version",
            request(
                SERVICE_CONTRACT_VERSION,
                delegate(Some(bridget_transport::protocol::ReviewTarget {
                    target_ref: "origin/main".to_string(),
                    expected_head: "a".repeat(40),
                },))
            ),
            &shared,
        ),
        Some(DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::InvalidEnvelope
        })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "delegate-review-version",
            request(REVIEW_DELEGATE_CONTRACT_VERSION, delegate(None)),
            &shared,
        ),
        Some(DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::InvalidEnvelope
        })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "delegate-review-version",
            request(
                REVIEW_DELEGATE_CONTRACT_VERSION,
                delegate(Some(bridget_transport::protocol::ReviewTarget {
                    target_ref: "origin/main".to_string(),
                    expected_head: "a".repeat(40),
                })),
            ),
            &shared,
        ),
        Some(DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::GreffeAuthorizationDenied
        })
    ));
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn service_service_negocie_sa_capacite_sans_lier_le_nom_au_droit() {
    let (state, config) = state_with_registered_agent("service-guichet");
    let shared = Arc::new(Mutex::new(state));
    let scope = "015_scope_0123456789abcdef0123456789abcdef";

    // Mutation discriminante : si un simple `from` ou nom de wrapper
    // autorisait le guichet, ce claim serait accepté au lieu du refus.
    assert!(matches!(
        handle_wrapper_message(
            "wrapper-guichet",
            WrapperToDaemon::Register {
                agent_type: "codex".to_string(),
                identity_version: 2,
                agent_id: "89000000-0000-4000-8000-000000000122".to_string(),
                host: None,
                transport: None,
                channel: None.into(),
                mode: None,
                location: None,
                os: None,
                instance_id: None,
                domain: None,
                turn_in_progress: false,
                journal_available: None,
                turn_attest: false,
                queue_fallback: false,
            },
            &shared,
        ),
        Some(DaemonToWrapper::Registered { .. })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "wrapper-guichet",
            WrapperToDaemon::GuichetClaimNext {
                version: SERVICE_CONTRACT_VERSION
            },
            &shared
        ),
        Some(DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::ServiceRoleRequired
        })
    ));

    for connection in ["service-without-capability", "service-capable"] {
        assert!(matches!(
            handle_wrapper_message(
                connection,
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Service,
                },
                &shared,
            ),
            Some(DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Service
            })
        ));
    }
    assert!(matches!(
        handle_wrapper_message(
            "service-without-capability",
            WrapperToDaemon::ServiceHello {
                version: SERVICE_CONTRACT_VERSION,
                service: "guichet".to_string(),
                issuer_scope: scope.to_string(),
                capabilities: Vec::new(),
            },
            &shared,
        ),
        Some(DaemonToWrapper::ServiceWelcome { capabilities, .. }) if capabilities.is_empty()
    ));
    assert!(matches!(
        handle_wrapper_message(
            "service-without-capability",
            WrapperToDaemon::GuichetClaimNext {
                version: SERVICE_CONTRACT_VERSION
            },
            &shared,
        ),
        Some(DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::CapabilityRequired
        })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "service-without-capability",
            WrapperToDaemon::GuichetReply {
                version: SERVICE_CONTRACT_VERSION,
                issuer_scope: scope.to_string(),
                request_id: "req-1".to_string(),
                claim_generation: 1,
                claim_token: "claim-1".to_string(),
                response_message_id: "message-1".to_string(),
                in_reply_to: "message-0".to_string(),
                outcome: bridget_transport::protocol::GuichetOutcome::Accepted,
                payload: bridget_transport::protocol::GuichetReplyPayload::DeliveryReport {
                    objective_id: "objective-1".to_string(),
                    delegation_id: "delegation-1".to_string(),
                    delivery_hash: "0".repeat(64),
                    review_verdict: None,
                },
            },
            &shared,
        ),
        Some(DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::CapabilityRequired
        })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "service-capable",
            WrapperToDaemon::ServiceHello {
                version: SERVICE_CONTRACT_VERSION,
                service: "guichet".to_string(),
                issuer_scope: scope.to_string(),
                capabilities: vec![ServiceCapability::GuichetV1],
            },
            &shared,
        ),
        Some(DaemonToWrapper::ServiceWelcome { capabilities, .. })
            if capabilities == vec![ServiceCapability::GuichetV1]
    ));
    assert!(matches!(
        handle_wrapper_message(
            "service-capable",
            WrapperToDaemon::GuichetClaimNext {
                version: SERVICE_CONTRACT_VERSION
            },
            &shared
        ),
        Some(DaemonToWrapper::GuichetEmpty {
            version: SERVICE_CONTRACT_VERSION
        })
    ));

    assert_eq!(shared.lock().unwrap().service_negotiations.len(), 2);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn service_capability_is_cleaned_after_a_broken_response_socket() {
    // Aucun faux agent à ancien nom court ni mutation globale de HOME :
    // ce scénario porte seulement la négociation d'une connexion service.
    let root = PathBuf::from("/tmp").join(format!("b89bp-{}", Uuid::new_v4().simple()));
    crate::environment::ensure_private_directory(&root).unwrap();
    let config = DaemonConfig {
        socket_path: root.join("daemon.sock"),
        db_path: root.join("daemon.db"),
        log_path: root.join("daemon.log"),
        circuit_breaker_window: 180,
        circuit_breaker_limit: 8,
        dedup_window: 180,
        quarantine_window: 3600,
        retention_days: 7,
    };
    let (managed_tx, _managed_rx) = mpsc::channel();
    let mut state = DaemonState::new(&config, managed_tx).unwrap();
    state.fixture_root = Some(FixtureRoot(root));
    let shared = Arc::new(Mutex::new(state));
    let listener = UnixListener::bind(&config.socket_path).unwrap();
    let (first_closed_tx, first_closed_rx) = mpsc::channel();
    let state_for_server = Arc::clone(&shared);
    let server = thread::spawn(move || {
        let (first, _) = listener.accept().unwrap();
        let first_failed = handle_connection(first, Arc::clone(&state_for_server)).is_err();
        first_closed_tx.send(first_failed).unwrap();

        let (second, _) = listener.accept().unwrap();
        handle_connection(second, state_for_server).is_ok()
    });
    let fixture = SERVICE_NEGOTIATION_FIXTURE.lines().collect::<Vec<_>>();

    let first = UnixStream::connect(&config.socket_path).unwrap();
    first
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let mut first_writer = BufWriter::new(first.try_clone().unwrap());
    let mut first_reader = BufReader::new(first);
    writeln!(first_writer, "{}", fixture[0]).unwrap();
    first_writer.flush().unwrap();
    let mut accepted = String::new();
    first_reader.read_line(&mut accepted).unwrap();
    assert_eq!(accepted.trim_end(), fixture[1]);

    // Mutation discriminante : sans la sortie commune de
    // `handle_connection`, le BrokenPipe ci-dessous laisserait la capacité
    // dans `service_negotiations` et la connexion suivante l'hériterait.
    writeln!(first_writer, "{}", fixture[2]).unwrap();
    first_writer.flush().unwrap();
    first_writer.get_ref().shutdown(Shutdown::Both).unwrap();
    drop(first_reader);
    drop(first_writer);
    assert!(
        first_closed_rx
            .recv_timeout(Duration::from_secs(3))
            .unwrap(),
        "la réponse ServiceWelcome doit échouer sur le socket fermé"
    );
    assert!(shared.lock().unwrap().service_negotiations.is_empty());

    let second = UnixStream::connect(&config.socket_path).unwrap();
    second
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let mut second_writer = BufWriter::new(second.try_clone().unwrap());
    let mut second_reader = BufReader::new(second);
    writeln!(second_writer, "{}", fixture[0]).unwrap();
    second_writer.flush().unwrap();
    let mut second_accepted = String::new();
    second_reader.read_line(&mut second_accepted).unwrap();
    assert_eq!(second_accepted.trim_end(), fixture[1]);

    let no_capability = WrapperToDaemon::ServiceHello {
        version: SERVICE_CONTRACT_VERSION,
        service: "guichet".to_string(),
        issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".to_string(),
        capabilities: Vec::new(),
    };
    writeln!(second_writer, "{}", encode(&no_capability).unwrap()).unwrap();
    second_writer.flush().unwrap();
    let mut welcome = String::new();
    second_reader.read_line(&mut welcome).unwrap();
    assert!(matches!(
        decode::<DaemonToWrapper>(welcome.trim_end()).unwrap(),
        DaemonToWrapper::ServiceWelcome { capabilities, .. } if capabilities.is_empty()
    ));
    writeln!(
        second_writer,
        "{}",
        encode(&WrapperToDaemon::GuichetClaimNext {
            version: SERVICE_CONTRACT_VERSION
        })
        .unwrap()
    )
    .unwrap();
    second_writer.flush().unwrap();
    let mut refusal = String::new();
    second_reader.read_line(&mut refusal).unwrap();
    assert_eq!(refusal.trim_end(), fixture[4]);

    drop(second_reader);
    drop(second_writer);
    assert!(server.join().unwrap(), "seconde connexion nettoyée");
    assert!(shared.lock().unwrap().service_negotiations.is_empty());
    let _ = std::fs::remove_file(config.socket_path);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn client_negocie_un_contrat_versionne_apres_son_role() {
    let (state, config) = state_with_registered_agent("client-negotiation");
    let shared = Arc::new(Mutex::new(state));
    assert!(matches!(
        handle_wrapper_message(
            "client-1",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client
            },
            &shared,
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        })
    ));
    let response = handle_wrapper_message(
        "client-1",
        WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: "012_scope_aaaaaaaaaaaa".to_string(),
            capabilities: vec![ClientCapability::SendIdempotent, ClientCapability::Lookup],
        },
        &shared,
    );
    assert!(matches!(
        response,
        Some(DaemonToWrapper::ClientWelcome {
            version: CLIENT_CONTRACT_VERSION,
            build_id,
            horizon_secs: CLIENT_IDEMPOTENCY_HORIZON_SECS,
            issued_at_tolerance_secs: CLIENT_ISSUED_AT_TOLERANCE_SECS,
            capabilities,
        }) if build_id == crate::build_info::BUILD_ID
            && capabilities == vec![ClientCapability::SendIdempotent, ClientCapability::Lookup]
    ));
    let negotiated = shared
        .lock()
        .unwrap()
        .client_negotiations
        .get("client-1")
        .cloned()
        .unwrap();
    assert_eq!(negotiated.version, CLIENT_CONTRACT_VERSION);
    assert_eq!(negotiated.issuer_scope, "012_scope_aaaaaaaaaaaa");
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn client_hello_avant_role_et_depuis_attach_sont_refuses_sans_negociation() {
    let (state, config) = state_with_registered_agent("client-order");
    let shared = Arc::new(Mutex::new(state));
    let hello = || WrapperToDaemon::ClientHello {
        contract_version: CLIENT_CONTRACT_VERSION,
        issuer_scope: "012_scope_bbbbbbbbbbbb".to_string(),
        capabilities: vec![ClientCapability::Lookup],
    };
    assert!(matches!(
        handle_wrapper_message("client-before-role", hello(), &shared),
        Some(DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::RoleHandshakeRequired
        })
    ));
    assert!(
        !shared
            .lock()
            .unwrap()
            .connection_roles
            .contains_key("client-before-role")
    );
    assert!(matches!(
        handle_wrapper_message(
            "attach-client-hello",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach
            },
            &shared,
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    assert!(matches!(
        handle_wrapper_message("attach-client-hello", hello(), &shared),
        Some(DaemonToWrapper::AttachRejected {
            reason: AttachRefusal::MessageOutsideAttachRole,
            ..
        })
    ));
    assert!(shared.lock().unwrap().client_negotiations.is_empty());
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn send_et_lookup_avant_welcome_ne_negocient_ni_ne_reservent() {
    let (state, config) = state_with_registered_agent("client-before-welcome");
    let shared = Arc::new(Mutex::new(state));
    handle_wrapper_message(
        "client-before-welcome",
        WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Client,
        },
        &shared,
    );
    let message = BridgetMessage::new(
        "client",
        "89000000-0000-4000-8000-000000000102",
        "sans welcome",
    );
    assert!(matches!(
        handle_wrapper_message(
            "client-before-welcome",
            WrapperToDaemon::SendIdempotent {
                message,
                message_id: "message-1".to_string(),
                issued_at: 1,
            },
            &shared,
        ),
        Some(DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::NegotiationRequired
        })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "client-before-welcome",
            WrapperToDaemon::Lookup {
                operation_kind: "send".to_string(),
                idempotency_key: "message-1".to_string(),
            },
            &shared,
        ),
        Some(DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::NegotiationRequired
        })
    ));
    assert!(shared.lock().unwrap().client_negotiations.is_empty());
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn client_refuse_version_et_scope_invalides_et_capacite_non_negociee() {
    let (state, config) = state_with_registered_agent("client-validation");
    let shared = Arc::new(Mutex::new(state));
    for connection in ["client-version", "client-scope", "client-capability"] {
        handle_wrapper_message(
            connection,
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client,
            },
            &shared,
        );
    }
    assert!(matches!(
        handle_wrapper_message(
            "client-version",
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION + 1,
                issuer_scope: "012_scope_cccccccccccc".to_string(),
                capabilities: vec![ClientCapability::Lookup],
            },
            &shared,
        ),
        Some(DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::UnsupportedVersion { supported_versions }
        }) if supported_versions == vec![CLIENT_CONTRACT_VERSION]
    ));
    assert!(matches!(
        handle_wrapper_message(
            "client-scope",
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: "invalide!".to_string(),
                capabilities: vec![ClientCapability::Lookup],
            },
            &shared,
        ),
        Some(DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::InvalidIssuerScope
        })
    ));
    handle_wrapper_message(
        "client-capability",
        WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: "012_scope_dddddddddddd".to_string(),
            capabilities: vec![ClientCapability::Lookup],
        },
        &shared,
    );
    assert!(matches!(
        handle_wrapper_message(
            "client-capability",
            WrapperToDaemon::SendIdempotent {
                message: BridgetMessage::new(
                    "client",
                    "89000000-0000-4000-8000-000000000102",
                    "hors capacite"
                ),
                message_id: "message-2".to_string(),
                issued_at: 1,
            },
            &shared,
        ),
        Some(DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::CapabilityNotNegotiated
        })
    ));
    assert!(
        shared
            .lock()
            .unwrap()
            .client_negotiations
            .contains_key("client-capability")
    );
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn client_borne_les_scopes_actifs_sans_rejeter_un_scope_deja_actif() {
    let (mut state, config) = state_with_registered_agent("client-scope-limit");
    for number in 0..MAX_ACTIVE_ISSUER_SCOPES {
        state.client_negotiations.insert(
            format!("existing-{number}"),
            NegotiatedClient {
                version: CLIENT_CONTRACT_VERSION,
                issuer_scope: format!("012_scope_{number:016x}"),
                capabilities: vec![ClientCapability::Lookup],
            },
        );
    }
    state
        .connection_roles
        .insert("client-limit".to_string(), ConnectionRole::Client);
    let shared = Arc::new(Mutex::new(state));
    assert!(matches!(
        handle_wrapper_message(
            "client-limit",
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: "012_scope_ffffffffffffffff".to_string(),
                capabilities: vec![ClientCapability::Lookup],
            },
            &shared,
        ),
        Some(DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::ActiveScopeLimit
        })
    ));
    shared
        .lock()
        .unwrap()
        .connection_roles
        .insert("client-retry".to_string(), ConnectionRole::Client);
    assert!(matches!(
        handle_wrapper_message(
            "client-retry",
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: "012_scope_0000000000000000".to_string(),
                capabilities: vec![ClientCapability::Lookup],
            },
            &shared,
        ),
        Some(DaemonToWrapper::ClientWelcome { .. })
    ));
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn wrapper_accepte_les_accuses_idempotents_et_register_historique_reste_wrapper() {
    let (mut state, config) = state_with_registered_agent("client-wrapper-matrix");
    let key = IdempotencyKey::new(
        "012_scope_aaaaaaaaaaaa",
        OperationKind::Send,
        "message-ack-wrapper",
    )
    .unwrap();
    let now = unix_now_secs();
    state
        .idempotency
        .reserve(
            &key,
            b"ack-wrapper",
            now,
            CLIENT_IDEMPOTENCY_HORIZON_SECS,
            now,
            CLIENT_ISSUED_AT_TOLERANCE_SECS,
        )
        .unwrap();
    let mut ack_message =
        bridget_core::BridgetMessage::new("sender-peer", "peer-1", "corps collège");
    ack_message.id = "message-ack-wrapper".to_string();
    let message_bytes = serde_json::to_vec(&ack_message).unwrap();
    state
        .idempotency
        .begin_send_delivery(
            &key,
            &SendDelivery {
                delivery_id: "delivery-1".to_string(),
                recipient_instance_id: "instance-1".to_string(),
                delivery_generation: 1,
                expires_at: now + CLIENT_IDEMPOTENCY_HORIZON_SECS,
                message_bytes,
            },
        )
        .unwrap();
    let shared = Arc::new(Mutex::new(state));
    assert!(
        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::DeliverAcked {
                delivery_id: "delivery-1".to_string(),
                delivery_generation: 1,
            },
            &shared,
        )
        .is_none()
    );
    assert_eq!(
        shared.lock().unwrap().connection_roles.get("conn-1"),
        Some(&ConnectionRole::Wrapper)
    );
    let record_count_after_ack = shared.lock().unwrap().idempotency.record_count().unwrap();
    assert!(matches!(
        handle_wrapper_message(
            "historic-register",
            WrapperToDaemon::Register {
                agent_type: "codex".to_string(),
                identity_version: 2,
                agent_id: "89000000-0000-4000-8000-000000000113".to_string(),
                host: None,
                transport: None,
                channel: None.into(),
                mode: None,
                location: None,
                os: None,
                instance_id: None,
                domain: None,
                turn_in_progress: false,
                journal_available: None,
                turn_attest: false,
                queue_fallback: false,
            },
            &shared,
        ),
        Some(DaemonToWrapper::Registered { .. })
    ));
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .connection_roles
            .get("historic-register"),
        Some(&ConnectionRole::Wrapper)
    );
    assert_eq!(
        shared.lock().unwrap().idempotency.record_count().unwrap(),
        record_count_after_ack
    );
    let _ = std::fs::remove_file(config.db_path);
}

pub(super) fn negotiate_idempotent_client(
    shared: &Arc<Mutex<DaemonState>>,
    connection: &str,
    scope: &str,
    sender: &str,
) {
    let (instance, credential) = {
        let mut state = shared.lock().unwrap();
        let sender = routing_agent_id(sender);
        let owner = state
            .router
            .get_agent(sender)
            .map(|route| route.connection_id.clone())
            .unwrap_or_else(|| format!("owner-{sender}"));
        if !state.identity_credentials.contains_key(&owner) {
            state.router.unregister_by_conn(&owner);
            assert!(matches!(
                handle_register_with_channel(
                    &owner,
                    2,
                    "fixture".into(),
                    sender.into(),
                    Some("fixture".into()),
                    Some("acp".into()),
                    ChannelReport::Known("unix".into()),
                    Some(PresenceMode::Acp),
                    None,
                    Some("test".into()),
                    Some(format!("instance-{owner}")),
                    None,
                    false,
                    Some(false),
                    &mut state,
                    false,
                ),
                DaemonToWrapper::Registered {
                    credential: Some(_),
                    ..
                }
            ));
        }
        if !state.connections.contains_key(&owner) {
            let _peer = super::spec094_live_test_connection(&mut state, &owner);
        }
        (
            state.conn_instances[&owner].clone(),
            state.identity_credentials[&owner].clone(),
        )
    };
    assert!(matches!(
        handle_wrapper_message(
            connection,
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client,
            },
            shared,
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        })
    ));
    assert!(matches!(
        handle_wrapper_message(
            connection,
            WrapperToDaemon::RegisterAuxiliary {
                agent_id: routing_agent_id(sender).into(),
                instance_id: instance,
                credential,
            },
            shared
        ),
        Some(DaemonToWrapper::Registered {
            credential: None,
            ..
        })
    ));
    assert!(matches!(
        handle_wrapper_message(
            connection,
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: scope.to_string(),
                capabilities: vec![ClientCapability::SendIdempotent],
            },
            shared,
        ),
        Some(DaemonToWrapper::ClientWelcome { .. })
    ));
}

fn idempotent_message(body: &str) -> BridgetMessage {
    let mut message = BridgetMessage::new(
        "89000000-0000-4000-8000-000000000200",
        "89000000-0000-4000-8000-000000000102",
        body,
    );
    message.hops = 4;
    message
}

#[test]
fn spec134_enregistrement_sature_publie_le_nom_t3_dans_enveloppe() {
    let sender = "89000000-0000-4000-8000-000000000299";
    let (mut state, config) = state_with_registered_agent("b134-saturation");
    state
        .store
        .connection()
        .execute_batch(
            "WITH RECURSIVE seq(n) AS (SELECT 1 UNION ALL SELECT n+1 FROM seq WHERE n<1000)
             INSERT INTO agent_identities(agent_id,created_at,updated_at)
             SELECT 'saturated-fixture-' || n,1,1 FROM seq;
             WITH RECURSIVE seq(n) AS (SELECT 1 UNION ALL SELECT n+1 FROM seq WHERE n<1000)
             INSERT OR IGNORE INTO agent_profiles
             SELECT 'saturated-fixture-' || n,
                 CASE n WHEN 1 THEN 'Agent' ELSE 'Agent (' || n || ')' END,
                 CASE n WHEN 1 THEN 'agent' ELSE 'agent (' || n || ')' END,
                 'round','blue','',1,1,1 FROM seq;",
        )
        .unwrap();
    let (writer, _reader) = control_socket("b134-saturation");
    state.connections.insert("conn-saturation".into(), writer);
    assert!(matches!(
        handle_register_with_channel(
            "conn-saturation",
            2,
            "claude".into(),
            sender.into(),
            Some("macbook".into()),
            Some("t3code".into()),
            ChannelReport::Unknown,
            Some(PresenceMode::Cli),
            None,
            Some("macOS".into()),
            Some("instance-saturation".into()),
            None,
            false,
            Some(true),
            &mut state,
            false,
        ),
        DaemonToWrapper::Registered { .. }
    ));
    let shared = Arc::new(Mutex::new(state));
    let result = handle_wrapper_message(
        "conn-saturation",
        WrapperToDaemon::DisplayNameSet {
            request: bridget_transport::protocol::DisplayNameRequest {
                version: 1,
                display_name: "opus-city-coder-1".into(),
            },
        },
        &shared,
    );
    assert!(matches!(
        result,
        Some(DaemonToWrapper::DisplayNameResult {
            outcome: bridget_transport::protocol::DisplayNameOutcome::Applied { .. },
        })
    ));
    let mut message = BridgetMessage::new(sender, "destinataire", "test");
    message.from_display_name = provider_display_name(&shared.lock().unwrap(), sender);
    assert!(
        crate::t3code::envelope(&message).contains(&format!("de opus-city-coder-1 ({sender})"))
    );
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn spec134_le_nom_livre_vient_du_profil_et_jamais_du_message_entrant() {
    let sender = "89000000-0000-4000-8000-000000000200";
    let (mut state, config) = state_with_registered_agent("spec134-display-authority");
    let (target_writer, mut target_reader) = control_socket("spec134-display-authority");
    state
        .connections
        .insert("conn-1".to_string(), target_writer);
    let shared = Arc::new(Mutex::new(state));
    negotiate_idempotent_client(
        &shared,
        "client-spec134-display",
        "134_scope_displaynameproof",
        sender,
    );
    crate::agent_profile::AgentProfileStore::open(&config.db_path)
        .unwrap()
        .rename_display_name(sender, "Regional", |_| false)
        .unwrap();

    let mut message = idempotent_message("nom sous autorité du daemon");
    message.from_display_name = Some("Nom forgé".to_string());
    let result = handle_wrapper_message(
        "client-spec134-display",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "spec134-display-authority".to_string(),
            issued_at: unix_now_secs(),
        },
        &shared,
    );
    assert!(matches!(
        result,
        Some(DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::OutcomeUnknown { .. },
            ..
        })
    ));

    let delivered = match read_control(&mut target_reader) {
        DaemonToWrapper::DeliverIdempotent { message, .. } => message,
        other => panic!("DeliverIdempotent attendu vers le wrapper: {other:?}"),
    };
    assert_eq!(delivered.from, sender);
    assert_eq!(delivered.from_display_name.as_deref(), Some("Regional"));
    assert_eq!(
        delivered.sender_label(),
        format!("Regional ({sender})"),
        "le nom forgé par l'appelant ne doit jamais atteindre le fournisseur"
    );

    let _ = std::fs::remove_file(config.db_path);
}

/// ORACLE — SendIdempotent reply=false (tous les mandats le service compagnon) doit
/// poser deadline_at au DEFAULT natif pour un type ABSENT de agents.json
/// (codex). Mutant : omettre stamp dans defer_idempotent_delivery → None.
/// Mutant : reposer 600 (fleet figé) → timeout hors 2700.
#[test]
#[allow(non_snake_case)]
fn TEMOIN_send_idempotent_reply_false_pose_deadline_codex_sans_agents_json() {
    let empty = AgentRegistry::from_json("{}", "/tmp/agents-temoin-codex-deadline.json")
        .expect("registre vide charge les natifs");
    assert_eq!(
        empty.get("codex").unwrap().notify_timeout_secs,
        DEFAULT_NOTIFY_TIMEOUT_SECS,
        "précondition : natif codex = DEFAULT, pas un 600 ailleurs"
    );
    assert_eq!(DEFAULT_NOTIFY_TIMEOUT_SECS, 2700);

    let (mut state, config) = state_with_registered_agent("idempotent-codex-deadline");
    // Comme relec6 : type codex, présence connectée, sans entrée agents.json.
    state.presences.get_mut("instance-1").unwrap().agent_type = "codex".to_string();
    let (target_writer, mut target_reader) = control_socket("idempotent-codex-deadline");
    state
        .connections
        .insert("conn-1".to_string(), target_writer);

    let shared = Arc::new(Mutex::new(state));
    negotiate_idempotent_client(
        &shared,
        "client-codex-deadline",
        "012_scope_codexdeadline",
        "89000000-0000-4000-8000-000000000200",
    );

    let mut message = idempotent_message("mandat le service compagnon sans reply");
    message.reply = false;
    assert!(
        message.deadline_at.is_none(),
        "contrôle positif : le client n'apporte pas de deadline"
    );

    let before = unix_now_secs() as u64;
    let result = handle_wrapper_message(
        "client-codex-deadline",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "mandat-codex-sans-deadline".to_string(),
            issued_at: unix_now_secs(),
        },
        &shared,
    );
    assert!(
        matches!(
            result,
            Some(DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            })
        ),
        "envoi idempotent attendu, reçu {result:?}"
    );

    let delivered = match read_control(&mut target_reader) {
        DaemonToWrapper::DeliverIdempotent { message, .. } => message,
        other => panic!("DeliverIdempotent attendu vers le wrapper: {other:?}"),
    };
    let after = unix_now_secs() as u64;
    let deadline = delivered
        .deadline_at
        .expect("deadline_at DOIT être posé pour reply=false sur DeliverIdempotent");
    let timeout = deadline.saturating_sub(before);
    assert!(
        timeout >= DEFAULT_NOTIFY_TIMEOUT_SECS.saturating_sub(2)
            && timeout
                <= DEFAULT_NOTIFY_TIMEOUT_SECS
                    .saturating_add(after.saturating_sub(before).saturating_add(2)),
        "codex sans agents.json doit recevoir DEFAULT={DEFAULT_NOTIFY_TIMEOUT_SECS}s ; \
             timeout≈{timeout} deadline={deadline} before={before} after={after}"
    );
    assert!(
        timeout > 660,
        "régression : encore le plafond 600 s (fleet/notify figé) ; timeout={timeout}"
    );

    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn message_humain_recoit_l_echeance_fournisseur_sauf_steer_explicite() {
    let registry = AgentRegistry::from_json("{}", "/tmp/agents-temoin-humain-deadline.json")
        .expect("registre natif");
    let mut message = idempotent_message("question humaine suivie");
    message.reply = true;
    message.origin = Some(bridget_core::MessageOrigin::Human);
    let before = unix_now_secs() as u64;
    stamp_turn_deadline_for_delivery(&mut message, &registry, "codex");
    let after = unix_now_secs() as u64;
    let timeout = message
        .deadline_at
        .expect("échéance humaine")
        .saturating_sub(before);
    assert!(
        timeout >= DEFAULT_NOTIFY_TIMEOUT_SECS.saturating_sub(1)
            && timeout
                <= DEFAULT_NOTIFY_TIMEOUT_SECS.saturating_add(after.saturating_sub(before) + 1),
        "une réponse humaine ne doit pas rétrécir l'échéance fournisseur ; timeout={timeout}"
    );

    let mut steer = idempotent_message("pilotage humain explicite");
    steer.origin = Some(bridget_core::MessageOrigin::Human);
    steer.intent = Some(bridget_core::MessageIntent::SteerCurrent);
    let before = unix_now_secs() as u64;
    stamp_turn_deadline_for_delivery(&mut steer, &registry, "codex");
    let after = unix_now_secs() as u64;
    let timeout = steer
        .deadline_at
        .expect("échéance de steer")
        .saturating_sub(before);
    assert!(
        timeout >= HUMAN_STEER_TIMEOUT_SECS.saturating_sub(1)
            && timeout <= HUMAN_STEER_TIMEOUT_SECS.saturating_add(after.saturating_sub(before) + 1),
        "le steer explicite doit rester borné ; timeout={timeout}"
    );
}

// Lien durable réel partagé par les oracles d'envoi et de garde156.
pub(super) fn guidance156_link_parent(st: &mut DaemonState, parent_conn: &str, target_conn: &str) {
    use crate::idempotency::{AgentLinkRecord, AgentLinkState};
    let parent = st.conn_instances[parent_conn].clone();
    let child = st.conn_instances[target_conn].clone();
    let link = AgentLinkRecord {
        link_id: Uuid::new_v4().to_string(),
        parent_instance_id: parent.clone(),
        child_instance_id: child.clone(),
        parent_execution_id: None,
        objective_id: None,
        delegation_id: None,
        project: None,
        role: "native_delegate".into(),
        agent_path: format!("{parent}/{child}"),
        state: AgentLinkState::Reserved,
        created_at: unix_now_secs(),
        closed_at: None,
        revision: 0,
    };
    st.idempotency.create_agent_link(&link).unwrap();
    st.idempotency
        .transition_agent_link(
            &link.link_id,
            &[AgentLinkState::Reserved],
            AgentLinkState::Open,
            unix_now_secs(),
        )
        .unwrap();
    assert_eq!(
        st.fleet
            .agent_link_for_child(&child)
            .unwrap()
            .unwrap()
            .parent_instance_id,
        parent
    );
}

#[test]
fn guidance156_send_canon_intention_demandee() {
    let passive = idempotent_message("canon demandé156");
    let scope = "012_scope_guidancecanon";
    let id = "canon156";
    let canon = canonical_send(scope, id, &passive, 123000);
    for (intent, byte) in [
        (bridget_core::MessageIntent::SteerCurrent, 3),
        (bridget_core::MessageIntent::InterruptAndStart, 4),
    ] {
        let mut message = passive.clone();
        message.intent = Some(intent);
        let demanded = canonical_send(scope, id, &message, 123000);
        assert_ne!(demanded, canon);
        assert!(demanded.windows(3).any(|window| window == [255, 0, byte]));
        assert_eq!(message.intent, Some(intent));
    }
}

#[test]
fn guidance156_send_rejeu_actif_puis_inactif_identique() {
    let (mut st, config) = state_with_registered_agent("guidance156-replay");
    let (writer, mut reader) = control_socket("guidance156-replay");
    st.connections.insert("conn-1".into(), writer);
    st.presences.get_mut("instance-1").unwrap().state = "busy".into();
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidancereplay",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "guidance-client", "conn-1");
    let mut message = idempotent_message("rejeu steer156");
    message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
    let issued_at = unix_now_secs();
    let send = || WrapperToDaemon::SendIdempotent {
        message: message.clone(),
        message_id: "guidance-replay".into(),
        issued_at,
    };
    let first = handle_wrapper_message("guidance-client", send(), &shared).unwrap();
    assert!(matches!(
        first,
        DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::OutcomeUnknown { .. },
            ..
        }
    ));
    assert!(
        matches!(read_control(&mut reader), DaemonToWrapper::DeliverIdempotent { message, .. }
            if message.intent == Some(bridget_core::MessageIntent::SteerCurrent))
    );
    let before = shared.lock().unwrap().idempotency.record_count().unwrap();
    {
        let mut st = shared.lock().unwrap();
        st.presences.get_mut("instance-1").unwrap().state = "stopped".into();
        st.router.unregister_by_conn("conn-1");
    }
    let replay = handle_wrapper_message("guidance-client", send(), &shared).unwrap();
    assert_eq!(encode(&replay).unwrap(), encode(&first).unwrap());
    assert_eq!(
        shared.lock().unwrap().idempotency.record_count().unwrap(),
        before
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_send_autre_intent_envelope_mismatch() {
    let (mut st, config) = state_with_registered_agent("guidance156-mismatch");
    let (writer, mut reader) = control_socket("guidance156-mismatch");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidancemismatch",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "guidance-client", "conn-1");
    let issued_at = unix_now_secs();
    let mut message = idempotent_message("intention différente156");
    message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
    assert!(matches!(
        handle_wrapper_message(
            "guidance-client",
            WrapperToDaemon::SendIdempotent {
                message: message.clone(),
                message_id: "guidance-mismatch".into(),
                issued_at
            },
            &shared
        ),
        Some(DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::OutcomeUnknown { .. },
            ..
        })
    ));
    read_control(&mut reader);
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    assert!(matches!(
        handle_wrapper_message(
            "guidance-client",
            WrapperToDaemon::SendIdempotent {
                message,
                message_id: "guidance-mismatch".into(),
                issued_at
            },
            &shared
        ),
        Some(DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::EnvelopeMismatch,
            ..
        })
    ));
    assert_eq!(
        shared.lock().unwrap().idempotency.record_count().unwrap(),
        1
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_send_queue_octets_inchanges() {
    let mut absent = idempotent_message("file156");
    absent.id = "queue156".into();
    let mut queue = absent.clone();
    queue.intent = None; // queue API n'ajoute aucune intention message.
    assert_eq!(
        serde_json::to_vec(&queue).unwrap(),
        serde_json::to_vec(&absent).unwrap()
    );
    assert_eq!(
        canonical_send("scope156", "queue156", &queue, 123000),
        vec![
            98, 114, 105, 100, 103, 101, 116, 47, 99, 108, 105, 101, 110, 116, 45, 115, 101, 110,
            100, 47, 118, 49, 0, 0, 0, 0, 0, 0, 0, 0, 8, 115, 99, 111, 112, 101, 49, 53, 54, 0, 0,
            0, 0, 0, 0, 0, 8, 113, 117, 101, 117, 101, 49, 53, 54, 0, 0, 0, 0, 0, 0, 0, 36, 56, 57,
            48, 48, 48, 48, 48, 48, 45, 48, 48, 48, 48, 45, 52, 48, 48, 48, 45, 56, 48, 48, 48, 45,
            48, 48, 48, 48, 48, 48, 48, 48, 48, 49, 48, 50, 0, 0, 0, 0, 0, 0, 0, 7, 102, 105, 108,
            101, 49, 53, 54, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8,
            0, 0, 0, 0, 0, 1, 224, 120
        ]
    );
    assert_eq!(
        canonical_send("scope156", "queue156", &queue, 123000),
        canonical_send("scope156", "queue156", &absent, 123000)
    );
}

#[test]
fn guidance156_send_steer_transmis_tel_quel() {
    let (mut st, config) = state_with_registered_agent("guidance156-steer");
    let (writer, mut reader) = control_socket("guidance156-steer");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidancesteer",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "guidance-client", "conn-1");
    let mut message = idempotent_message("steer transmis156");
    message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
    assert!(matches!(
        handle_wrapper_message(
            "guidance-client",
            WrapperToDaemon::SendIdempotent {
                message: message.clone(),
                message_id: "guidance-steer".into(),
                issued_at: unix_now_secs()
            },
            &shared
        ),
        Some(DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::OutcomeUnknown { .. },
            ..
        })
    ));
    let delivered = read_control(&mut reader);
    assert!(
        matches!(delivered, DaemonToWrapper::DeliverIdempotent { message, .. } if message.intent == Some(bridget_core::MessageIntent::SteerCurrent))
    );
    let before = shared.lock().unwrap().idempotency.record_count().unwrap();
    message.reply = true;
    message.body = "steer reply interdit156".into();
    assert!(
        matches!(handle_wrapper_message("guidance-client", WrapperToDaemon::SendIdempotent { message, message_id: "guidance-invalid-reply".into(), issued_at: unix_now_secs() }, &shared),
            Some(DaemonToWrapper::Nack { reason, .. }) if reason == "invalid_intent")
    );
    assert_eq!(
        shared.lock().unwrap().idempotency.record_count().unwrap(),
        before,
        "refus avant réservation"
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn spec269_steer_claude_stream_refuse_durablement_avant_remise() {
    let (template_state, template_config, tasks) = control158_state_with_tasks("steer269");
    let mut task = tasks[1].clone();
    drop(template_state);
    let _ = std::fs::remove_file(template_config.db_path);
    let (mut st, config) = state_with_registered_agent("steer269");
    let (writer, reader) = control_socket("steer269");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(&shared, "guidance-client", "012_scope_guidance_steer269", "89000000-0000-4000-8000-000000000200");
    {
        let mut st = shared.lock().unwrap();
        guidance156_link_parent(&mut st, "guidance-client", "conn-1");
        task.child = "89000000-0000-4000-8000-000000000102".into();
        task.child_instance = Some(st.conn_instances["conn-1"].clone());
        task.definition.protocol = "claude_stream_json".into();
        task.state = "working".into();
        st.delegation_store.insert(&task, "steer269", &serde_json::to_vec(&task.request).unwrap()).unwrap();
    }
    let mut message = idempotent_message("guidage269");
    message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
    let issued_at = unix_now_secs();
    for _ in 0..2 {
        let response = handle_wrapper_message("guidance-client", WrapperToDaemon::SendIdempotent {
            message: message.clone(), message_id: "steer269".into(), issued_at,
        }, &shared).unwrap();
        let DaemonToWrapper::IdempotencyResult { issue: IdempotencyIssue::Rejected { category, reason, .. }, .. } = response else {
            panic!("refus avant remise attendu : {response:?}")
        };
        assert_eq!(category, "steer_unavailable");
        assert!(reason.contains("avant remise") && reason.contains("intent=queue"));
    }
    reader.get_ref().set_nonblocking(true).unwrap();
    let mut byte = [0];
    assert!(matches!(std::io::Read::read(&mut reader.get_ref(), &mut byte), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock));
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_send_interrupt_admet_execution() {
    let (mut st, config) = state_with_registered_agent("guidance156-interrupt");
    let (writer, mut reader) = control_socket("guidance156-interrupt");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidanceinterrupt",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "guidance-client", "conn-1");
    let mut message = idempotent_message("interrupt exécuté156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    assert!(matches!(
        handle_wrapper_message(
            "guidance-client",
            WrapperToDaemon::SendIdempotent {
                message,
                message_id: "guidance-interrupt".into(),
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
        matches!(read_control(&mut reader), DaemonToWrapper::DeliverIdempotent { message, execution: Some(_), .. }
            if message.intent == Some(bridget_core::MessageIntent::InterruptAndStart))
    );
    let st = shared.lock().unwrap();
    assert_eq!(
        st.execution_store
            .execution_snapshot("execution-guidance-interrupt")
            .unwrap()
            .unwrap()
            .state,
        "starting"
    );
    drop(st);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_guard_pair_meme_projet() {
    let (mut st, config) = state_with_registered_agent("guidance156_guard_pair_meme_projet");
    let (writer, mut reader) = control_socket("guidance156_guard_pair_meme_projet");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_guard_pair_meme_projet",
        "89000000-0000-4000-8000-000000000200",
    );
    let mut message = idempotent_message("garde brute156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let response = handle_wrapper_message(
        "guidance-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "guard156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        !matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ),
        "refus attendu : {response:?}"
    );
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).is_err(),
        "aucune remise après refus : {line}"
    );
    let mut legacy = idempotent_message("garde historique156");
    legacy.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let response = handle_wrapper_message(
        "owner-89000000-0000-4000-8000-000000000200",
        WrapperToDaemon::Send(legacy),
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response, DaemonToWrapper::Nack { .. }),
        "historique : {response:?}"
    );
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).is_err(),
        "aucune remise après refus : {line}"
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_guard_autre_projet() {
    let (mut st, config) = state_with_registered_agent("guidance156_guard_autre_projet");
    let (writer, mut reader) = control_socket("guidance156_guard_autre_projet");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_guard_autre_projet",
        "89000000-0000-4000-8000-000000000200",
    );
    {
        let mut st = shared.lock().unwrap();
        for (conn, root) in [
            ("owner-89000000-0000-4000-8000-000000000200", "/from/.git"),
            ("conn-1", "/to/.git"),
        ] {
            let project = bridget_transport::protocol::CommunicationProject {
                host: "fixture".into(),
                root: root.into(),
            };
            st.communication_projects
                .insert(conn.into(), (Some(project.clone()), Some(project), false));
        }
    }
    let mut message = idempotent_message("garde brute156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let response = handle_wrapper_message(
        "guidance-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "guard156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        !matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ),
        "refus attendu : {response:?}"
    );
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).is_err(),
        "aucune remise après refus : {line}"
    );
    let mut legacy = idempotent_message("garde historique156");
    legacy.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let response = handle_wrapper_message(
        "owner-89000000-0000-4000-8000-000000000200",
        WrapperToDaemon::Send(legacy),
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response, DaemonToWrapper::Nack { .. }),
        "historique : {response:?}"
    );
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).is_err(),
        "aucune remise après refus : {line}"
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_guard_soi_meme() {
    let (mut st, config) = state_with_registered_agent("guidance156_guard_soi_meme");
    let (writer, mut reader) = control_socket("guidance156_guard_soi_meme");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_guard_soi_meme",
        "89000000-0000-4000-8000-000000000200",
    );
    let mut message = idempotent_message("garde brute156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    message.to = "89000000-0000-4000-8000-000000000200".into();
    message.from = "89000000-0000-4000-8000-000000000200".into();
    let response = handle_wrapper_message(
        "guidance-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "guard156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        !matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ),
        "refus attendu : {response:?}"
    );
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).is_err(),
        "aucune remise après refus : {line}"
    );
    let mut legacy = idempotent_message("garde historique156");
    legacy.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    legacy.to = "89000000-0000-4000-8000-000000000200".into();
    let response = handle_wrapper_message(
        "owner-89000000-0000-4000-8000-000000000200",
        WrapperToDaemon::Send(legacy),
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response, DaemonToWrapper::Nack { .. }),
        "historique : {response:?}"
    );
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).is_err(),
        "aucune remise après refus : {line}"
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_guard_non_atteste() {
    let (mut st, config) = state_with_registered_agent("guidance156_guard_non_atteste");
    let (writer, mut reader) = control_socket("guidance156_guard_non_atteste");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_guard_non_atteste",
        "89000000-0000-4000-8000-000000000200",
    );
    assert!(matches!(
        handle_wrapper_message(
            "bare",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted { .. })
    ));
    handle_wrapper_message(
        "bare",
        WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: "012_scope_bare156".into(),
            capabilities: vec![ClientCapability::SendIdempotent],
        },
        &shared,
    )
    .unwrap();
    let mut message = idempotent_message("garde brute156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    message.from = "humain".into();
    message.origin = Some(bridget_core::MessageOrigin::Human);
    let response = handle_wrapper_message(
        "bare",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "guard156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        !matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ),
        "refus attendu : {response:?}"
    );
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).is_err(),
        "aucune remise après refus : {line}"
    );
    let mut legacy = idempotent_message("garde historique156");
    legacy.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let response = handle_wrapper_message(
        "owner-89000000-0000-4000-8000-000000000200",
        WrapperToDaemon::Send(legacy),
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response, DaemonToWrapper::Nack { .. }),
        "historique : {response:?}"
    );
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).is_err(),
        "aucune remise après refus : {line}"
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_guard_preuve_enfant() {
    let (mut st, config) = state_with_registered_agent("guidance156_guard_preuve_enfant");
    let (writer, mut reader) = control_socket("guidance156_guard_preuve_enfant");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_guard_preuve_enfant",
        "89000000-0000-4000-8000-000000000200",
    );
    let mut message = idempotent_message("garde brute156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    guidance156_link_parent(&mut shared.lock().unwrap(), "guidance-client", "conn-1");
    message.delegated_origin = Some(bridget_core::DelegatedOrigin {
        provider: "codex".into(),
        child_ref: "child-proof156".into(),
    });
    let response = handle_wrapper_message(
        "guidance-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "guard156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        !matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ),
        "refus attendu : {response:?}"
    );
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).is_err(),
        "aucune remise après refus : {line}"
    );
    let mut legacy = idempotent_message("garde historique156");
    legacy.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    legacy.delegated_origin = Some(bridget_core::DelegatedOrigin {
        provider: "codex".into(),
        child_ref: "child-proof156".into(),
    });
    let response = handle_wrapper_message(
        "owner-89000000-0000-4000-8000-000000000200",
        WrapperToDaemon::Send(legacy),
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response, DaemonToWrapper::Nack { .. }),
        "historique : {response:?}"
    );
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).is_err(),
        "aucune remise après refus : {line}"
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_guard_from_forge() {
    let (mut st, config) = state_with_registered_agent("guidance156_guard_from_forge");
    let (writer, mut reader) = control_socket("guidance156_guard_from_forge");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_guard_from_forge",
        "89000000-0000-4000-8000-000000000200",
    );
    let mut message = idempotent_message("garde brute156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    message.from = "89000000-0000-4000-8000-000000000999".into();
    let response = handle_wrapper_message(
        "guidance-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "guard156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        !matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ),
        "refus attendu : {response:?}"
    );
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).is_err(),
        "aucune remise après refus : {line}"
    );
    let mut legacy = idempotent_message("garde historique156");
    legacy.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    legacy.from = "89000000-0000-4000-8000-000000000999".into();
    let response = handle_wrapper_message(
        "owner-89000000-0000-4000-8000-000000000200",
        WrapperToDaemon::Send(legacy),
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response, DaemonToWrapper::Nack { .. }),
        "historique : {response:?}"
    );
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).is_err(),
        "aucune remise après refus : {line}"
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_origin_human_forgee_par_agent_retiree() {
    let (mut st, config) =
        state_with_registered_agent("guidance156_origin_human_forgee_par_agent_retiree");
    let (writer, mut reader) = control_socket("guidance156_origin_human_forgee_par_agent_retiree");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_origin_human_forgee_par_agent_retiree",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "guidance-client", "conn-1");
    let mut message = idempotent_message("origine156");
    message.origin = Some(bridget_core::MessageOrigin::Human);
    message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
    handle_wrapper_message(
        "guidance-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "origin156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(read_control(&mut reader),DaemonToWrapper::DeliverIdempotent {message,..} if message.origin.is_none())
    );
    assert_eq!(
        shared.lock().unwrap().connection_roles["guidance-client"],
        ConnectionRole::Client
    );
    assert!(!matches!(
        handle_wrapper_message(
            "guidance-client",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_origin_human_attach_conservee() {
    let (mut st, config) = state_with_registered_agent("guidance156_origin_human_attach_conservee");
    let (writer, mut reader) = control_socket("guidance156_origin_human_attach_conservee");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_origin_human_attach_conservee",
        "89000000-0000-4000-8000-000000000200",
    );
    let (writer, _peer) = control_socket("attach156");
    shared
        .lock()
        .unwrap()
        .connections
        .insert("attach156".into(), writer);
    assert!(matches!(
        handle_wrapper_message(
            "attach156",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    let mut message = idempotent_message("origine156");
    message.origin = Some(bridget_core::MessageOrigin::Human);
    message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
    assert!(matches!(
        handle_wrapper_message("attach156", WrapperToDaemon::Send(message), &shared),
        Some(DaemonToWrapper::Ack { .. })
    ));
    assert!(
        matches!(read_control(&mut reader),DaemonToWrapper::Deliver {message,..} if message.origin==Some(bridget_core::MessageOrigin::Human))
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_pause_steer_agent_refuse() {
    let (mut st, config) = state_with_registered_agent("guidance156_pause_steer_agent_refuse");
    let (writer, mut reader) = control_socket("guidance156_pause_steer_agent_refuse");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_pause_steer_agent_refuse",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "guidance-client", "conn-1");
    {
        let st = shared.lock().unwrap();
        assert!(
            crate::referent_control::set(
                st.store.connection(),
                crate::referent_control::ControlMutation {
                    command_id: "pause156",
                    expected_generation: 0,
                    paused: Some(true),
                    auto_objectives_cap: None,
                    reason: Some("oracle156"),
                    actor: "human",
                    now: unix_now_secs(),
                    agent_posture: None,
                    auto_reassignment: None
                }
            )
            .unwrap()
            .is_ok()
        );
    }
    let mut message = idempotent_message("pause156");
    message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
    let response = handle_wrapper_message(
        "guidance-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "pause-message156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response,DaemonToWrapper::IdempotencyResult {issue:IdempotencyIssue::Rejected {category,..},..} if category=="referent_paused")
    );
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).is_err(),
        "aucune remise après refus : {line}"
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_pause_interrupt_agent_refuse() {
    let (mut st, config) = state_with_registered_agent("guidance156_pause_interrupt_agent_refuse");
    let (writer, mut reader) = control_socket("guidance156_pause_interrupt_agent_refuse");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_pause_interrupt_agent_refuse",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "guidance-client", "conn-1");
    {
        let st = shared.lock().unwrap();
        assert!(
            crate::referent_control::set(
                st.store.connection(),
                crate::referent_control::ControlMutation {
                    command_id: "pause156",
                    expected_generation: 0,
                    paused: Some(true),
                    auto_objectives_cap: None,
                    reason: Some("oracle156"),
                    actor: "human",
                    now: unix_now_secs(),
                    agent_posture: None,
                    auto_reassignment: None
                }
            )
            .unwrap()
            .is_ok()
        );
    }
    let mut message = idempotent_message("pause156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let response = handle_wrapper_message(
        "guidance-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "pause-message156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response,DaemonToWrapper::IdempotencyResult {issue:IdempotencyIssue::Rejected {category,..},..} if category=="referent_paused")
    );
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).is_err(),
        "aucune remise après refus : {line}"
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_pause_queue_admise() {
    let (mut st, config) = state_with_registered_agent("guidance156_pause_queue_admise");
    let (writer, mut reader) = control_socket("guidance156_pause_queue_admise");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_pause_queue_admise",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "guidance-client", "conn-1");
    {
        let st = shared.lock().unwrap();
        assert!(
            crate::referent_control::set(
                st.store.connection(),
                crate::referent_control::ControlMutation {
                    command_id: "pause156",
                    expected_generation: 0,
                    paused: Some(true),
                    auto_objectives_cap: None,
                    reason: Some("oracle156"),
                    actor: "human",
                    now: unix_now_secs(),
                    agent_posture: None,
                    auto_reassignment: None
                }
            )
            .unwrap()
            .is_ok()
        );
    }
    let message = idempotent_message("pause156");
    let response = handle_wrapper_message(
        "guidance-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "pause-message156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(matches!(
        response,
        DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::OutcomeUnknown { .. },
            ..
        }
    ));
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::DeliverIdempotent { .. }
    ));
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_pause_humain_admis() {
    let (mut st, config) = state_with_registered_agent("guidance156_pause_humain_admis");
    let (writer, mut reader) = control_socket("guidance156_pause_humain_admis");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_pause_humain_admis",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "guidance-client", "conn-1");
    {
        let st = shared.lock().unwrap();
        assert!(
            crate::referent_control::set(
                st.store.connection(),
                crate::referent_control::ControlMutation {
                    command_id: "pause156",
                    expected_generation: 0,
                    paused: Some(true),
                    auto_objectives_cap: None,
                    reason: Some("oracle156"),
                    actor: "human",
                    now: unix_now_secs(),
                    agent_posture: None,
                    auto_reassignment: None
                }
            )
            .unwrap()
            .is_ok()
        );
    }
    let (writer, _peer) = control_socket("pause-human156");
    shared
        .lock()
        .unwrap()
        .connections
        .insert("attach156".into(), writer);
    handle_wrapper_message(
        "attach156",
        WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Attach,
        },
        &shared,
    )
    .unwrap();
    let mut message = idempotent_message("humain pendant pause156");
    message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
    assert!(matches!(
        handle_wrapper_message("attach156", WrapperToDaemon::Send(message), &shared),
        Some(DaemonToWrapper::Ack { .. })
    ));
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::Deliver {
            mid_turn_ok: Some(true),
            ..
        }
    ));
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_pause_aucun_tour_cree() {
    let (mut st, config) = state_with_registered_agent("guidance156_pause_aucun_tour_cree");
    let (writer, mut reader) = control_socket("guidance156_pause_aucun_tour_cree");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_pause_aucun_tour_cree",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "guidance-client", "conn-1");
    {
        let st = shared.lock().unwrap();
        assert!(
            crate::referent_control::set(
                st.store.connection(),
                crate::referent_control::ControlMutation {
                    command_id: "pause156",
                    expected_generation: 0,
                    paused: Some(true),
                    auto_objectives_cap: None,
                    reason: Some("oracle156"),
                    actor: "human",
                    now: unix_now_secs(),
                    agent_posture: None,
                    auto_reassignment: None
                }
            )
            .unwrap()
            .is_ok()
        );
    }
    let mut message = idempotent_message("pause156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let response = handle_wrapper_message(
        "guidance-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "pause-message156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response,DaemonToWrapper::IdempotencyResult {issue:IdempotencyIssue::Rejected {category,..},..} if category=="referent_paused")
    );
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).is_err(),
        "aucune remise après refus : {line}"
    );
    assert!(
        shared
            .lock()
            .unwrap()
            .execution_store
            .execution_snapshot("execution-pause-message156")
            .unwrap()
            .is_none()
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_midturn_pair_attend_fin_de_tour() {
    let (mut st, config) =
        state_with_registered_agent("guidance156_midturn_pair_attend_fin_de_tour");
    let (writer, mut reader) = control_socket("guidance156_midturn_pair_attend_fin_de_tour");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_midturn_pair_attend_fin_de_tour",
        "89000000-0000-4000-8000-000000000200",
    );
    let message = idempotent_message("midturn156");
    handle_wrapper_message(
        "guidance-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "midturn-message156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::DeliverIdempotent {
            mid_turn_ok: Some(false),
            ..
        }
    ));
    let legacy = idempotent_message("même relation historique156");
    assert!(matches!(
        handle_wrapper_message(
            "owner-89000000-0000-4000-8000-000000000200",
            WrapperToDaemon::Send(legacy),
            &shared
        ),
        Some(DaemonToWrapper::Ack { .. })
    ));
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::Deliver {
            mid_turn_ok: Some(false),
            ..
        }
    ));
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_midturn_enfant_vers_parent_immediat() {
    let (mut st, config) =
        state_with_registered_agent("guidance156_midturn_enfant_vers_parent_immediat");
    let (writer, mut reader) = control_socket("guidance156_midturn_enfant_vers_parent_immediat");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_midturn_enfant_vers_parent_immediat",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "conn-1", "guidance-client");
    let message = idempotent_message("midturn156");
    handle_wrapper_message(
        "guidance-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "midturn-message156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::DeliverIdempotent {
            mid_turn_ok: Some(true),
            ..
        }
    ));
    let legacy = idempotent_message("même relation historique156");
    assert!(matches!(
        handle_wrapper_message(
            "owner-89000000-0000-4000-8000-000000000200",
            WrapperToDaemon::Send(legacy),
            &shared
        ),
        Some(DaemonToWrapper::Ack { .. })
    ));
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::Deliver {
            mid_turn_ok: Some(true),
            ..
        }
    ));
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_midturn_parent_immediat() {
    let (mut st, config) = state_with_registered_agent("guidance156_midturn_parent_immediat");
    let (writer, mut reader) = control_socket("guidance156_midturn_parent_immediat");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_midturn_parent_immediat",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "guidance-client", "conn-1");
    let message = idempotent_message("midturn156");
    handle_wrapper_message(
        "guidance-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "midturn-message156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::DeliverIdempotent {
            mid_turn_ok: Some(true),
            ..
        }
    ));
    let legacy = idempotent_message("même relation historique156");
    assert!(matches!(
        handle_wrapper_message(
            "owner-89000000-0000-4000-8000-000000000200",
            WrapperToDaemon::Send(legacy),
            &shared
        ),
        Some(DaemonToWrapper::Ack { .. })
    ));
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::Deliver {
            mid_turn_ok: Some(true),
            ..
        }
    ));
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_midturn_humain_attache_immediat() {
    let (mut st, config) =
        state_with_registered_agent("guidance156_midturn_humain_attache_immediat");
    let (writer, mut reader) = control_socket("guidance156_midturn_humain_attache_immediat");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_midturn_humain_attache_immediat",
        "89000000-0000-4000-8000-000000000200",
    );
    let (writer, _peer) = control_socket("mid-human156");
    shared
        .lock()
        .unwrap()
        .connections
        .insert("attach156".into(), writer);
    handle_wrapper_message(
        "attach156",
        WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Attach,
        },
        &shared,
    )
    .unwrap();
    let message = idempotent_message("midturn156");
    handle_wrapper_message("attach156", WrapperToDaemon::Send(message), &shared).unwrap();
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::Deliver {
            mid_turn_ok: Some(true),
            ..
        }
    ));
    // UI réelle : Register réservé puis auxiliaire attestée ; son rôle reste Client.
    {
        let mut st = shared.lock().unwrap();
        let host = st.host.clone();
        assert!(matches!(
            handle_register_with_channel(
                "ui-primary156",
                2,
                "ui".into(),
                UI_HUMAN_AGENT_ID.into(),
                Some(host),
                None,
                ChannelReport::Omitted,
                Some(PresenceMode::Cli),
                None,
                Some("test".into()),
                Some("bridget-ui-guidance156".into()),
                None,
                false,
                Some(false),
                &mut st,
                false
            ),
            DaemonToWrapper::Registered { .. }
        ));
        let _ = super::spec094_live_test_connection(&mut st, "ui-primary156");
    }
    negotiate_idempotent_client(
        &shared,
        "ui-client156",
        "012_scope_uimid156_attested",
        "humain",
    );
    let ui = BridgetMessage::new(
        UI_HUMAN_SENDER,
        "89000000-0000-4000-8000-000000000102",
        "UI sans origin forgée156",
    );
    let response = handle_wrapper_message(
        "ui-client156",
        WrapperToDaemon::SendIdempotent {
            message: ui.clone(),
            message_id: "ui-midturn156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ),
        "UI attestée : {response:?}"
    );
    assert!(
        matches!(read_control(&mut reader),DaemonToWrapper::DeliverIdempotent {message,mid_turn_ok:Some(true),..} if message.origin.is_none())
    );
    assert_eq!(
        shared.lock().unwrap().connection_roles["ui-client156"],
        ConnectionRole::Client
    );
    assert!(matches!(
        handle_wrapper_message("ui-primary156", WrapperToDaemon::Send(ui), &shared),
        Some(DaemonToWrapper::Ack { .. })
    ));
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::Deliver {
            mid_turn_ok: Some(true),
            ..
        }
    ));
    // Une UI présente n'atteste pas le Client nu qui emploie son libellé.
    handle_wrapper_message(
        "bare-human156",
        WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Client,
        },
        &shared,
    )
    .unwrap();
    handle_wrapper_message(
        "bare-human156",
        WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: "012_scope_barehuman156".into(),
            capabilities: vec![ClientCapability::SendIdempotent],
        },
        &shared,
    )
    .unwrap();
    handle_wrapper_message(
        "bare-human156",
        WrapperToDaemon::SendIdempotent {
            message: BridgetMessage::new(
                "humain",
                "89000000-0000-4000-8000-000000000102",
                "étiquette sans autorité156",
            ),
            message_id: "bare-human156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::DeliverIdempotent {
            mid_turn_ok: Some(false),
            ..
        }
    ));
    // Producteurs internes existants, jamais une origine fournie par un client.
    {
        let mut st = shared.lock().unwrap();
        let writer = st.connections["conn-1"].clone();
        deliver_to_agent(
            &writer,
            "89000000-0000-4000-8000-000000000102",
            "notice système156",
        )
        .unwrap();
        let now = unix_now_secs();
        let departure = ThreadWakeDeparture {
            thread_id: Uuid::new_v4().to_string(),
            agent_id: "89000000-0000-4000-8000-000000000102".into(),
            through_seq: 1,
            generation: 1,
            delivery_id: Uuid::new_v4().to_string(),
            delivery_key: "system-wake156".into(),
            recipient_instance_id: "instance-1".into(),
            delivery_generation: next_delivery_generation(),
            issued_at: now,
            expires_at: now + 600,
            target_conn: "conn-1".into(),
        };
        let mut controls = Vec::new();
        project_thread_wake(&mut st, &departure, &mut controls).unwrap();
        assert!(matches!(
            &controls[0].message,
            DaemonToWrapper::DeliverIdempotent {
                mid_turn_ok: Some(true),
                ..
            }
        ));
    }
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::Deliver {
            mid_turn_ok: Some(true),
            ..
        }
    ));
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_midturn_trame_sans_valeur_attend() {
    let (mut st, config) =
        state_with_registered_agent("guidance156_midturn_trame_sans_valeur_attend");
    let (writer, mut reader) = control_socket("guidance156_midturn_trame_sans_valeur_attend");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_midturn_trame_sans_valeur_attend",
        "89000000-0000-4000-8000-000000000200",
    );
    let message = idempotent_message("midturn156");
    handle_wrapper_message(
        "owner-89000000-0000-4000-8000-000000000200",
        WrapperToDaemon::Send(message),
        &shared,
    )
    .unwrap();
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::Deliver {
            mid_turn_ok: Some(false),
            ..
        }
    ));
    handle_wrapper_message(
        "guidance-client",
        WrapperToDaemon::SendIdempotent {
            message: idempotent_message("sans valeur idempotent156"),
            message_id: "no-proof-idemp156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::DeliverIdempotent {
            mid_turn_ok: Some(false),
            ..
        }
    ));
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_midturn_reprise_recalcule() {
    let (mut st, config) = state_with_registered_agent("guidance156_midturn_reprise_recalcule");
    let (writer, mut reader) = control_socket("guidance156_midturn_reprise_recalcule");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_midturn_reprise_recalcule",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "guidance-client", "conn-1");
    let message = idempotent_message("midturn156");
    handle_wrapper_message(
        "guidance-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "midturn-message156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::DeliverIdempotent {
            mid_turn_ok: Some(true),
            ..
        }
    ));
    {
        let mut st = shared.lock().unwrap();
        let child = st.conn_instances["conn-1"].clone();
        let link = st.fleet.agent_link_for_child(&child).unwrap().unwrap();
        st.idempotency
            .transition_agent_link(
                &link.link_id,
                &[crate::idempotency::AgentLinkState::Open],
                crate::idempotency::AgentLinkState::Closed,
                unix_now_secs(),
            )
            .unwrap();
        let record = st.idempotency.record_count().unwrap();
        schedule_idempotent_delivery_recovery(&mut st, "conn-1", "instance-1");
        assert_eq!(st.idempotency.record_count().unwrap(), record);
        assert!(matches!(
            &st.pending_post_response_controls["conn-1"][0].message,
            DaemonToWrapper::DeliverIdempotent {
                mid_turn_ok: Some(false),
                ..
            }
        ));
    }
    let legacy = idempotent_message("même relation historique156");
    assert!(matches!(
        handle_wrapper_message(
            "owner-89000000-0000-4000-8000-000000000200",
            WrapperToDaemon::Send(legacy),
            &shared
        ),
        Some(DaemonToWrapper::Ack { .. })
    ));
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::Deliver {
            mid_turn_ok: Some(false),
            ..
        }
    ));
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_control_interrupt_client_non_autorise() {
    let (mut st, config) =
        state_with_registered_agent("guidance156_control_interrupt_client_non_autorise");
    let (writer, mut reader) = control_socket("guidance156_control_interrupt_client_non_autorise");
    st.connections.insert("conn-1".into(), writer);
    st.execution_store
        .record_starting(
            "submission156",
            "execution156",
            "89000000-0000-4000-8000-000000000102",
            unix_now_secs(),
        )
        .unwrap();
    st.execution_store
        .transition_if_current(
            "execution156",
            "starting",
            0,
            1,
            "running",
            "provider_accepted",
            unix_now_secs(),
        )
        .unwrap();
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_control_interrupt_client_non_autorise",
        "89000000-0000-4000-8000-000000000200",
    );
    let mut st = shared.lock().unwrap();
    let message = None;
    let command = ExecutionControlCommand {
        version: 1,
        command_id: "control156".into(),
        execution_id: "execution156".into(),
        generation: 1,
        revision: 1,
        operation: ExecutionControlOperation::Interrupt,
        message,
    };
    let response = handle_execution_control(
        Some("guidance-client"),
        "012_scope_control156",
        command,
        &mut st,
    );
    assert!(matches!(
        response,
        DaemonToWrapper::ControlExecutionResult {
            outcome: ExecutionControlOutcome::Refused(ExecutionControlRefusal::NotPermitted),
            ..
        }
    ));
    drop(st);
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(reader.read_line(&mut line).is_err());
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_control_steer_pair_refuse() {
    let (mut st, config) = state_with_registered_agent("guidance156_control_steer_pair_refuse");
    let (writer, mut reader) = control_socket("guidance156_control_steer_pair_refuse");
    st.connections.insert("conn-1".into(), writer);
    st.execution_store
        .record_starting(
            "submission156",
            "execution156",
            "89000000-0000-4000-8000-000000000102",
            unix_now_secs(),
        )
        .unwrap();
    st.execution_store
        .transition_if_current(
            "execution156",
            "starting",
            0,
            1,
            "running",
            "provider_accepted",
            unix_now_secs(),
        )
        .unwrap();
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_control_steer_pair_refuse",
        "89000000-0000-4000-8000-000000000200",
    );
    let mut st = shared.lock().unwrap();
    let message = {
        let mut m = idempotent_message("controle156");
        m.intent = Some(bridget_core::MessageIntent::SteerCurrent);
        Some(m)
    };
    let command = ExecutionControlCommand {
        version: 1,
        command_id: "control156".into(),
        execution_id: "execution156".into(),
        generation: 1,
        revision: 1,
        operation: ExecutionControlOperation::SteerCurrent,
        message,
    };
    let response = handle_execution_control(
        Some("guidance-client"),
        "012_scope_control156",
        command,
        &mut st,
    );
    assert!(matches!(
        response,
        DaemonToWrapper::ControlExecutionResult {
            outcome: ExecutionControlOutcome::Refused(ExecutionControlRefusal::NotPermitted),
            ..
        }
    ));
    drop(st);
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(reader.read_line(&mut line).is_err());
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_control_pause_interne_admise() {
    let (mut st, config) = state_with_registered_agent("guidance156_control_pause_interne_admise");
    let (writer, mut reader) = control_socket("guidance156_control_pause_interne_admise");
    st.connections.insert("conn-1".into(), writer);
    st.execution_store
        .record_starting(
            "submission156",
            "execution156",
            "89000000-0000-4000-8000-000000000102",
            unix_now_secs(),
        )
        .unwrap();
    st.execution_store
        .transition_if_current(
            "execution156",
            "starting",
            0,
            1,
            "running",
            "provider_accepted",
            unix_now_secs(),
        )
        .unwrap();
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_control_pause_interne_admise",
        "89000000-0000-4000-8000-000000000200",
    );
    let mut st = shared.lock().unwrap();
    let message = None;
    let command = ExecutionControlCommand {
        version: 1,
        command_id: "control156".into(),
        execution_id: "execution156".into(),
        generation: 1,
        revision: 1,
        operation: ExecutionControlOperation::Interrupt,
        message,
    };
    let before = st
        .idempotency
        .count_interrupts_target(
            "89000000-0000-4000-8000-000000000102",
            unix_now_secs() - 600,
        )
        .unwrap();
    let response = handle_execution_control(None, "012_scope_control156", command, &mut st);
    assert!(matches!(
        response,
        DaemonToWrapper::ControlExecutionResult {
            outcome: ExecutionControlOutcome::OutcomeUnknown,
            ..
        }
    ));
    assert_eq!(
        st.idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap(),
        before
    );
    drop(st);
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::ControlExecutionDispatch { .. }
    ));
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_human_client_nu_refuse_pour_controle() {
    let (mut st, config) = state_with_registered_agent("g15600a0d635");
    let (writer, mut reader) = control_socket("g15600a0d635");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "g156-client",
        "012_scope_guard_tests156",
        "89000000-0000-4000-8000-000000000200",
    );
    assert!(matches!(
        handle_wrapper_message(
            "bare156",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        })
    ));
    handle_wrapper_message(
        "bare156",
        WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: "012_scope_bare_human156".into(),
            capabilities: vec![ClientCapability::SendIdempotent],
        },
        &shared,
    )
    .unwrap();
    let mut message = idempotent_message("bare-steer156");
    message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
    message.from = "human".into();
    let response = handle_wrapper_message(
        "bare156",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "bare-steer156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response,DaemonToWrapper::IdempotencyResult {issue:IdempotencyIssue::Rejected {ref category,ref reason,..},..} if category=="intent_not_permitted" && reason=="identity_unattested"),
        "{response:?}"
    );
    let mut queue = idempotent_message("human queue toujours admise156");
    queue.from = "human".into();
    let queued = handle_wrapper_message(
        "bare156",
        WrapperToDaemon::SendIdempotent {
            message: queue,
            message_id: "bare-queue156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(
            queued,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ),
        "{queued:?}"
    );
    assert!(
        matches!(read_control(&mut reader),DaemonToWrapper::DeliverIdempotent {message,..} if message.body=="human queue toujours admise156")
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        0
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_human_reconnexion_agent_sans_autorite() {
    let (mut st, config) = state_with_registered_agent("g1566c628943");
    let (writer, mut reader) = control_socket("g1566c628943");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "g156-client",
        "012_scope_guard_tests156",
        "89000000-0000-4000-8000-000000000200",
    );
    let result = handle_wrapper_message(
        "g156-client",
        WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Attach,
        },
        &shared,
    )
    .unwrap();
    assert!(
        !matches!(
            result,
            DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Attach
            }
        ),
        "rôle déjà Client : {result:?}"
    );
    assert_eq!(
        shared.lock().unwrap().connection_roles.get("g156-client"),
        Some(&ConnectionRole::Client)
    );
    let mut message = idempotent_message("reconnect-steer156");
    message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
    let response = handle_wrapper_message(
        "g156-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "reconnect-steer156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response, DaemonToWrapper::IdempotencyResult { issue:IdempotencyIssue::Rejected { ref category, .. }, .. } if category == "intent_not_permitted"),
        "{response:?}"
    );
    negotiate_idempotent_client(
        &shared,
        "reconnect156",
        "012_scope_reconnect156",
        "89000000-0000-4000-8000-000000000200",
    );
    let mut message = idempotent_message("reconnect proof agent156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let response = handle_wrapper_message(
        "reconnect156",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "reconnect-interrupt156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response, DaemonToWrapper::IdempotencyResult { issue:IdempotencyIssue::Rejected { ref category, .. }, .. } if category == "intent_not_permitted"),
        "{response:?}"
    );
    // Une nouvelle Attach brute du même compte reste hors modèle ADR011.
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).is_err(),
        "aucune remise : {line}"
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_human_attach_admis() {
    let (mut st, config) = state_with_registered_agent("g15666d50ccc");
    let (writer, mut reader) = control_socket("g15666d50ccc");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "g156-client",
        "012_scope_guard_tests156",
        "89000000-0000-4000-8000-000000000200",
    );
    let (writer, _attach_reader) = control_socket("g156-attached");
    shared
        .lock()
        .unwrap()
        .connections
        .insert("attach156".into(), writer);
    assert!(matches!(
        handle_wrapper_message(
            "attach156",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    assert!(!shared.lock().unwrap().conn_names.contains_key("attach156"));
    let mut message = idempotent_message("human attach steer156");
    message.from = "human".into();
    message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
    let response =
        handle_wrapper_message("attach156", WrapperToDaemon::Send(message), &shared).unwrap();
    assert!(
        matches!(response, DaemonToWrapper::Ack { .. }),
        "{response:?}"
    );
    assert!(
        matches!(read_control(&mut reader),DaemonToWrapper::Deliver {message,..} if message.intent==Some(bridget_core::MessageIntent::SteerCurrent))
    );
    assert_eq!(
        shared.lock().unwrap().connection_roles.get("attach156"),
        Some(&ConnectionRole::Attach)
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_rate_cooldown_30s() {
    let (mut st, config) = state_with_registered_agent("g156e984483c");
    let (writer, mut reader) = control_socket("g156e984483c");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "g156-client",
        "012_scope_guard_tests156",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "g156-client", "conn-1");
    let mut message = idempotent_message("cooldown-first156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let response = handle_wrapper_message(
        "g156-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "cooldown-first156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ),
        "{response:?}"
    );
    read_control(&mut reader);
    let mut message = idempotent_message("cooldown-second156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let response = handle_wrapper_message(
        "g156-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "cooldown-second156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response, DaemonToWrapper::IdempotencyResult { issue:IdempotencyIssue::Rejected { ref category, .. }, .. } if category == "interrupt_rate_limited"),
        "{response:?}"
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        1
    );
    let recorded = shared
        .lock()
        .unwrap()
        .idempotency
        .count_interrupts_pair(
            "89000000-0000-4000-8000-000000000200",
            "89000000-0000-4000-8000-000000000102",
            unix_now_secs() - 30,
        )
        .unwrap();
    assert_eq!(recorded.0, 1);
    let db = rusqlite::Connection::open(&config.db_path).unwrap();
    db.execute(
        "UPDATE guidance_events SET at=?1 WHERE outcome='admitted'",
        [unix_now_secs() - 30],
    )
    .unwrap();
    drop(db);
    let mut message = idempotent_message("cooldown-boundary156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let response = handle_wrapper_message(
        "g156-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "cooldown-boundary156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ),
        "{response:?}"
    );
    read_control(&mut reader);
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        2
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_rate_plafond_3_par_10min() {
    let (mut st, config) = state_with_registered_agent("g1563f3f8d4b");
    let (writer, mut reader) = control_socket("g1563f3f8d4b");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "g156-client",
        "012_scope_guard_tests156",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "g156-client", "conn-1");
    let now = unix_now_secs();
    shared
        .lock()
        .unwrap()
        .idempotency
        .record_guidance_event(
            now - 500,
            "other-parent-a",
            "89000000-0000-4000-8000-000000000102",
            "interrupt",
            "send_idempotent",
            Some("seed156"),
            "admitted",
        )
        .unwrap();
    shared
        .lock()
        .unwrap()
        .idempotency
        .record_guidance_event(
            now - 300,
            "other-parent-b",
            "89000000-0000-4000-8000-000000000102",
            "interrupt",
            "send_idempotent",
            Some("seed156"),
            "admitted",
        )
        .unwrap();
    shared
        .lock()
        .unwrap()
        .idempotency
        .record_guidance_event(
            now - 100,
            "other-parent-c",
            "89000000-0000-4000-8000-000000000102",
            "interrupt",
            "send_idempotent",
            Some("seed156"),
            "admitted",
        )
        .unwrap();
    let mut message = idempotent_message("target-cap156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let response = handle_wrapper_message(
        "g156-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "target-cap156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response, DaemonToWrapper::IdempotencyResult { issue:IdempotencyIssue::Rejected { ref category, .. }, .. } if category == "interrupt_rate_limited"),
        "{response:?}"
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        3
    );
    let db = rusqlite::Connection::open(&config.db_path).unwrap();
    db.execute(
        "UPDATE guidance_events SET at=?1 WHERE actor='other-parent-a'",
        [unix_now_secs() - 601],
    )
    .unwrap();
    drop(db);
    let mut message = idempotent_message("target-cap-boundary156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let response = handle_wrapper_message(
        "g156-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "target-cap-boundary156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ),
        "{response:?}"
    );
    read_control(&mut reader);
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        3
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_rate_retry_after_exact() {
    let (mut st, config) = state_with_registered_agent("g15608e07778");
    let (writer, _reader) = control_socket("g15608e07778");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "g156-client",
        "012_scope_guard_tests156",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "g156-client", "conn-1");
    let last = unix_now_secs() - 17;
    shared
        .lock()
        .unwrap()
        .idempotency
        .record_guidance_event(
            last,
            "89000000-0000-4000-8000-000000000200",
            "89000000-0000-4000-8000-000000000102",
            "interrupt",
            "send_idempotent",
            Some("seed156"),
            "admitted",
        )
        .unwrap();
    let mut message = idempotent_message("retry-after156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let response = handle_wrapper_message(
        "g156-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "retry-after156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    let db = rusqlite::Connection::open(&config.db_path).unwrap();
    let now:i64=db.query_row("SELECT at FROM guidance_events WHERE message_id='retry-after156' AND outcome LIKE 'refused:%' ORDER BY id DESC LIMIT 1",[],|row|row.get(0)).unwrap();
    drop(db);
    let DaemonToWrapper::IdempotencyResult {
        issue: IdempotencyIssue::Rejected {
            category, reason, ..
        },
        ..
    } = response
    else {
        panic!("refus quota attendu : {response:?}")
    };
    assert_eq!(category, "interrupt_rate_limited");
    let reason: serde_json::Value = serde_json::from_str(&reason).unwrap();
    assert_eq!(
        reason,
        serde_json::json!({"code":"interrupt_rate_limited","retry_after_secs":(last+30-now).max(0)})
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        1
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_rate_humain_exempt() {
    let (mut st, config) = state_with_registered_agent("g15660783bb7");
    let (writer, mut reader) = control_socket("g15660783bb7");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "g156-client",
        "012_scope_guard_tests156",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "g156-client", "conn-1");
    let now = unix_now_secs();
    shared
        .lock()
        .unwrap()
        .idempotency
        .record_guidance_event(
            now,
            "other-parent-a",
            "89000000-0000-4000-8000-000000000102",
            "interrupt",
            "send_idempotent",
            Some("seed156"),
            "admitted",
        )
        .unwrap();
    shared
        .lock()
        .unwrap()
        .idempotency
        .record_guidance_event(
            now,
            "other-parent-b",
            "89000000-0000-4000-8000-000000000102",
            "interrupt",
            "send_idempotent",
            Some("seed156"),
            "admitted",
        )
        .unwrap();
    shared
        .lock()
        .unwrap()
        .idempotency
        .record_guidance_event(
            now,
            "other-parent-c",
            "89000000-0000-4000-8000-000000000102",
            "interrupt",
            "send_idempotent",
            Some("seed156"),
            "admitted",
        )
        .unwrap();
    let (writer, _attach_reader) = control_socket("g156-human-rate");
    shared
        .lock()
        .unwrap()
        .connections
        .insert("attach156".into(), writer);
    assert!(matches!(
        handle_wrapper_message(
            "attach156",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted { .. })
    ));
    for n in 0..4 {
        let mut message = idempotent_message(&format!("human rate exempt156-{n}"));
        message.from = "human".into();
        message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
        let response =
            handle_wrapper_message("attach156", WrapperToDaemon::Send(message), &shared).unwrap();
        assert!(
            matches!(response, DaemonToWrapper::Ack { .. }),
            "{response:?}"
        );
        read_control(&mut reader);
    }
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        3
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_rate_refus_non_compte() {
    let (mut st, config) = state_with_registered_agent("g1569f556c4d");
    let (writer, mut reader) = control_socket("g1569f556c4d");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "g156-client",
        "012_scope_guard_tests156",
        "89000000-0000-4000-8000-000000000200",
    );
    let db = rusqlite::Connection::open(&config.db_path).unwrap();
    let peer_before = unix_now_secs();
    let mut message = idempotent_message("rate-peer-refused156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let response = handle_wrapper_message(
        "g156-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "rate-peer-refused156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response, DaemonToWrapper::IdempotencyResult { issue:IdempotencyIssue::Rejected { ref category, .. }, .. } if category == "intent_not_permitted"),
        "{response:?}"
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        0
    );
    let peer_after = unix_now_secs();
    let peer_counts: (i64, i64) = db.query_row(
            "SELECT count(*), coalesce(sum(outcome='admitted'),0) FROM guidance_events WHERE message_id=?1",
            ["rate-peer-refused156"], |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap();
    assert_eq!(
        peer_counts,
        (1, 0),
        "FR-016 : premier refus peer tracé une seule fois, aucune admission"
    );
    let peer_trace: (i64, String, String, String, String, String, String) = db.query_row(
            "SELECT at,actor,target,action,path,message_id,outcome FROM guidance_events WHERE message_id=?1",
            ["rate-peer-refused156"],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?)),
        ).unwrap();
    assert_eq!(
        (
            peer_trace.1.as_str(),
            peer_trace.2.as_str(),
            peer_trace.3.as_str(),
            peer_trace.4.as_str(),
            peer_trace.5.as_str()
        ),
        (
            "89000000-0000-4000-8000-000000000200",
            "89000000-0000-4000-8000-000000000102",
            "interrupt",
            "send_idempotent",
            "rate-peer-refused156"
        ),
        "faits réels du refus peer",
    );
    assert!(
        peer_before <= peer_trace.0 && peer_trace.0 <= peer_after,
        "date réelle du refus peer : {:?}",
        peer_trace
    );
    assert_eq!(peer_trace.6, "refused:intent_not_permitted");
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).is_err(),
        "aucune remise après refus peer : {line}"
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "g156-client", "conn-1");
    {
        let mut st = shared.lock().unwrap();
        for (conn, root) in [
            ("owner-89000000-0000-4000-8000-000000000200", "/from/.git"),
            ("conn-1", "/to/.git"),
        ] {
            let p = bridget_transport::protocol::CommunicationProject {
                host: "fixture".into(),
                root: root.into(),
            };
            st.communication_projects
                .insert(conn.into(), (Some(p.clone()), Some(p), false));
        }
    }
    let project_before = unix_now_secs();
    let mut message = idempotent_message("rate-project-refused156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let project_command = WrapperToDaemon::SendIdempotent {
        message,
        message_id: "rate-project-refused156".into(),
        issued_at: unix_now_secs(),
    };
    let response = handle_wrapper_message("g156-client", project_command.clone(), &shared).unwrap();
    assert!(
        !matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ),
        "{response:?}"
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        0
    );
    let DaemonToWrapper::IdempotencyResult {
        issue: IdempotencyIssue::Rejected { category, .. },
        ..
    } = &response
    else {
        panic!("refus durable du projet attendu : {response:?}");
    };
    let project_after = unix_now_secs();
    let project_counts: (i64, i64) = db.query_row(
            "SELECT count(*), coalesce(sum(outcome='admitted'),0) FROM guidance_events WHERE message_id=?1",
            ["rate-project-refused156"], |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap();
    assert_eq!(
        project_counts,
        (1, 0),
        "FR-016 : premier refus project tracé une seule fois, aucune admission"
    );
    let project_trace: (i64, String, String, String, String, String, String) = db.query_row(
            "SELECT at,actor,target,action,path,message_id,outcome FROM guidance_events WHERE message_id=?1",
            ["rate-project-refused156"],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?)),
        ).unwrap();
    assert_eq!(
        (
            project_trace.1.as_str(),
            project_trace.2.as_str(),
            project_trace.3.as_str(),
            project_trace.4.as_str(),
            project_trace.5.as_str()
        ),
        (
            "89000000-0000-4000-8000-000000000200",
            "89000000-0000-4000-8000-000000000102",
            "interrupt",
            "send_idempotent",
            "rate-project-refused156"
        ),
        "faits réels du refus project",
    );
    assert!(
        project_before <= project_trace.0 && project_trace.0 <= project_after,
        "date réelle du refus project : {:?}",
        project_trace
    );
    assert_eq!(project_trace.6, format!("refused:{category}"));
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).is_err(),
        "aucune remise après refus project : {line}"
    );
    // Le rejeu du même refus conserve son résultat et sa trace, même après
    // la suppression des faits de projet déjà prévue par la branche suivante.
    shared.lock().unwrap().communication_projects.clear();
    let replay = handle_wrapper_message("g156-client", project_command, &shared).unwrap();
    assert_eq!(
        encode(&replay).unwrap(),
        encode(&response).unwrap(),
        "rejeu exact du refus projet"
    );
    let replay_trace: (i64, String, String, String, String, String, String) = db.query_row(
            "SELECT at,actor,target,action,path,message_id,outcome FROM guidance_events WHERE message_id=?1",
            ["rate-project-refused156"],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?)),
        ).unwrap();
    assert_eq!(
        replay_trace, project_trace,
        "un rejeu ne relit ni ne retrace les faits"
    );
    let replay_counts: (i64, i64) = db.query_row(
            "SELECT count(*), coalesce(sum(outcome='admitted'),0) FROM guidance_events WHERE message_id=?1",
            ["rate-project-refused156"], |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap();
    assert_eq!(replay_counts, (1, 0));
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600,
            )
            .unwrap()
            .0,
        0
    );
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).is_err(),
        "aucune remise sur rejeu du refus projet : {line}"
    );
    db.execute_batch("CREATE TRIGGER g156_refuse_delivery BEFORE INSERT ON send_deliveries BEGIN SELECT RAISE(ABORT,'fixture persistence refusal156'); END;").unwrap();
    let persist_before = unix_now_secs();
    let mut message = idempotent_message("rate-persist-refused156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let response = handle_wrapper_message(
        "g156-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "rate-persist-refused156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        !matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ),
        "{response:?}"
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        0
    );
    let persist_after = unix_now_secs();
    let persist_counts: (i64, i64) = db.query_row(
            "SELECT count(*), coalesce(sum(outcome='admitted'),0) FROM guidance_events WHERE message_id=?1",
            ["rate-persist-refused156"], |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap();
    assert_eq!(
        persist_counts,
        (1, 0),
        "FR-016 : premier refus persist tracé une seule fois, aucune admission"
    );
    let persist_trace: (i64, String, String, String, String, String, String) = db.query_row(
            "SELECT at,actor,target,action,path,message_id,outcome FROM guidance_events WHERE message_id=?1",
            ["rate-persist-refused156"],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?)),
        ).unwrap();
    assert_eq!(
        (
            persist_trace.1.as_str(),
            persist_trace.2.as_str(),
            persist_trace.3.as_str(),
            persist_trace.4.as_str(),
            persist_trace.5.as_str()
        ),
        (
            "89000000-0000-4000-8000-000000000200",
            "89000000-0000-4000-8000-000000000102",
            "interrupt",
            "send_idempotent",
            "rate-persist-refused156"
        ),
        "faits réels du refus persist",
    );
    assert!(
        persist_before <= persist_trace.0 && persist_trace.0 <= persist_after,
        "date réelle du refus persist : {:?}",
        persist_trace
    );
    assert_eq!(persist_trace.6, "refused:persistence_failed");
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).is_err(),
        "aucune remise après refus persist : {line}"
    );
    db.execute_batch("DROP TRIGGER g156_refuse_delivery;")
        .unwrap();
    drop(db);
    let mut message = idempotent_message("rate-after-refus156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let response = handle_wrapper_message(
        "g156-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "rate-after-refus156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ),
        "{response:?}"
    );
    read_control(&mut reader);
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        1
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_rate_survit_au_redemarrage() {
    let (mut st, config) = state_with_registered_agent("g156c9af4b48");
    let (writer, mut reader) = control_socket("g156c9af4b48");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "g156-client",
        "012_scope_guard_tests156",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "g156-client", "conn-1");
    let mut message = idempotent_message("restart-first156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let response = handle_wrapper_message(
        "g156-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "restart-first156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ),
        "{response:?}"
    );
    read_control(&mut reader);
    let registry = shared.lock().unwrap().registry.clone();
    let root = shared.lock().unwrap().fixture_root.take();
    drop(shared);
    let (tx, _rx) = mpsc::channel();
    let mut st = DaemonState::new(&config, tx).unwrap();
    // Même base ; seuls le registre privé et les connexions sont rétablis en mémoire.
    st.registry = registry;
    st.fixture_root = root;
    assert!(st.presences.is_empty());
    let (writer, _reader) = control_socket("g156c9af4b48");
    st.connections.insert("conn-1".into(), writer);
    assert!(matches!(
        handle_register_with_channel(
            "conn-1",
            2,
            "claude".into(),
            "89000000-0000-4000-8000-000000000102".into(),
            Some("macbook".into()),
            Some("acp".into()),
            ChannelReport::Known("unix".into()),
            Some(PresenceMode::Acp),
            None,
            Some("macOS".into()),
            Some("instance-1-restarted156".into()),
            None,
            false,
            Some(true),
            &mut st,
            true,
        ),
        DaemonToWrapper::Registered { .. }
    ));
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "g156-client",
        "012_scope_guard_tests156",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "g156-client", "conn-1");
    let mut message = idempotent_message("restart-second156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let response = handle_wrapper_message(
        "g156-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "restart-second156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response, DaemonToWrapper::IdempotencyResult { issue:IdempotencyIssue::Rejected { ref category, .. }, .. } if category == "interrupt_rate_limited"),
        "{response:?}"
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        1
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_cancel_pair_demande_active_acp_sans_interruption() {
    let (mut st, config) = state_with_registered_agent("g15654afce7c");
    let (writer, mut reader) = control_socket("g15654afce7c");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "g156-client",
        "012_scope_guard_tests156",
        "89000000-0000-4000-8000-000000000200",
    );
    shared
        .lock()
        .unwrap()
        .set_turn_state("conn-1", true)
        .unwrap();
    shared
        .lock()
        .unwrap()
        .store
        .create_request(
            "cancel-request156",
            "89000000-0000-4000-8000-000000000200",
            "89000000-0000-4000-8000-000000000102",
            60,
        )
        .unwrap();
    let command = WrapperToDaemon::CancelRequest {
        id: "cancel-request156".into(),
        sender: "89000000-0000-4000-8000-000000000200".into(),
        reason: Some("annulation156".into()),
    };
    let before = shared
        .lock()
        .unwrap()
        .idempotency
        .count_interrupts_target(
            "89000000-0000-4000-8000-000000000102",
            unix_now_secs() - 600,
        )
        .unwrap()
        .0;
    let response = handle_wrapper_message(
        "owner-89000000-0000-4000-8000-000000000200",
        command.clone(),
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response,DaemonToWrapper::RequestCancelled {ref state,..} if state=="cancelled"),
        "{response:?}"
    );
    assert!(
        matches!(read_control(&mut reader),DaemonToWrapper::CancelDelivery {id,interrupt_active,..} if id=="cancel-request156" && interrupt_active==Some(false))
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .store
            .get_request("cancel-request156")
            .unwrap()
            .unwrap()
            .state,
        "cancelled"
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        before
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_cancel_pair_demande_en_file_retiree() {
    let (mut st, config) = state_with_registered_agent("g156a0151891");
    let (writer, mut reader) = control_socket("g156a0151891");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "g156-client",
        "012_scope_guard_tests156",
        "89000000-0000-4000-8000-000000000200",
    );
    shared
        .lock()
        .unwrap()
        .set_turn_state("conn-1", true)
        .unwrap();
    shared
        .lock()
        .unwrap()
        .store
        .create_request(
            "cancel-request156",
            "89000000-0000-4000-8000-000000000200",
            "89000000-0000-4000-8000-000000000102",
            60,
        )
        .unwrap();
    let command = WrapperToDaemon::CancelRequest {
        id: "cancel-request156".into(),
        sender: "89000000-0000-4000-8000-000000000200".into(),
        reason: Some("annulation156".into()),
    };
    let before = shared
        .lock()
        .unwrap()
        .idempotency
        .count_interrupts_target(
            "89000000-0000-4000-8000-000000000102",
            unix_now_secs() - 600,
        )
        .unwrap()
        .0;
    let response = handle_wrapper_message(
        "owner-89000000-0000-4000-8000-000000000200",
        command.clone(),
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response,DaemonToWrapper::RequestCancelled {ref state,..} if state=="cancelled"),
        "{response:?}"
    );
    assert!(
        matches!(read_control(&mut reader),DaemonToWrapper::CancelDelivery {id,interrupt_active,..} if id=="cancel-request156" && interrupt_active==Some(false))
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .store
            .get_request("cancel-request156")
            .unwrap()
            .unwrap()
            .state,
        "cancelled"
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        before
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_cancel_parent_interrompt() {
    let (mut st, config) = state_with_registered_agent("g1563798cd3b");
    let (writer, mut reader) = control_socket("g1563798cd3b");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "g156-client",
        "012_scope_guard_tests156",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "g156-client", "conn-1");
    shared
        .lock()
        .unwrap()
        .store
        .create_request(
            "cancel-request156",
            "89000000-0000-4000-8000-000000000200",
            "89000000-0000-4000-8000-000000000102",
            60,
        )
        .unwrap();
    let command = WrapperToDaemon::CancelRequest {
        id: "cancel-request156".into(),
        sender: "89000000-0000-4000-8000-000000000200".into(),
        reason: Some("annulation156".into()),
    };
    let before = shared
        .lock()
        .unwrap()
        .idempotency
        .count_interrupts_target(
            "89000000-0000-4000-8000-000000000102",
            unix_now_secs() - 600,
        )
        .unwrap()
        .0;
    let response = handle_wrapper_message(
        "owner-89000000-0000-4000-8000-000000000200",
        command.clone(),
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response,DaemonToWrapper::RequestCancelled {ref state,..} if state=="cancelled"),
        "{response:?}"
    );
    assert!(
        matches!(read_control(&mut reader),DaemonToWrapper::CancelDelivery {id,interrupt_active,..} if id=="cancel-request156" && interrupt_active==Some(true))
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .store
            .get_request("cancel-request156")
            .unwrap()
            .unwrap()
            .state,
        "cancelled"
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        before + 1
    );
    let replay = handle_wrapper_message(
        "owner-89000000-0000-4000-8000-000000000200",
        command,
        &shared,
    )
    .unwrap();
    assert!(matches!(replay,DaemonToWrapper::RequestCancelled {state,..} if state=="cancelled"));
    // Le rejeu peut notifier le retrait ; il ne doit jamais réinterrompre.
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    if reader.read_line(&mut line).is_ok() && !line.is_empty() {
        assert!(matches!(
            decode::<DaemonToWrapper>(line.trim()).unwrap(),
            DaemonToWrapper::CancelDelivery {
                interrupt_active: Some(false),
                ..
            }
        ));
    }
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        before + 1
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_cancel_parent_pause_sans_interruption() {
    let (mut st, config) = state_with_registered_agent("g1565fae8d8b");
    let (writer, mut reader) = control_socket("g1565fae8d8b");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "g156-client",
        "012_scope_guard_tests156",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "g156-client", "conn-1");
    {
        let st = shared.lock().unwrap();
        assert!(
            crate::referent_control::set(
                st.store.connection(),
                crate::referent_control::ControlMutation {
                    command_id: "cancel-pause156",
                    expected_generation: 0,
                    paused: Some(true),
                    auto_objectives_cap: None,
                    reason: Some("oracle156"),
                    actor: "human",
                    now: unix_now_secs(),
                    agent_posture: None,
                    auto_reassignment: None
                }
            )
            .unwrap()
            .is_ok()
        );
    }
    shared
        .lock()
        .unwrap()
        .store
        .create_request(
            "cancel-request156",
            "89000000-0000-4000-8000-000000000200",
            "89000000-0000-4000-8000-000000000102",
            60,
        )
        .unwrap();
    let command = WrapperToDaemon::CancelRequest {
        id: "cancel-request156".into(),
        sender: "89000000-0000-4000-8000-000000000200".into(),
        reason: Some("annulation156".into()),
    };
    let before = shared
        .lock()
        .unwrap()
        .idempotency
        .count_interrupts_target(
            "89000000-0000-4000-8000-000000000102",
            unix_now_secs() - 600,
        )
        .unwrap()
        .0;
    let response = handle_wrapper_message(
        "owner-89000000-0000-4000-8000-000000000200",
        command.clone(),
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response,DaemonToWrapper::RequestCancelled {ref state,..} if state=="cancelled"),
        "{response:?}"
    );
    assert!(
        matches!(read_control(&mut reader),DaemonToWrapper::CancelDelivery {id,interrupt_active,..} if id=="cancel-request156" && interrupt_active==Some(false))
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .store
            .get_request("cancel-request156")
            .unwrap()
            .unwrap()
            .state,
        "cancelled"
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        before
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_cancel_parent_plafond_sans_interruption() {
    let (mut st, config) = state_with_registered_agent("g15674fd2037");
    let (writer, mut reader) = control_socket("g15674fd2037");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "g156-client",
        "012_scope_guard_tests156",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "g156-client", "conn-1");
    shared
        .lock()
        .unwrap()
        .idempotency
        .record_guidance_event(
            unix_now_secs() - 100,
            "other-parent-a",
            "89000000-0000-4000-8000-000000000102",
            "interrupt",
            "send_idempotent",
            Some("seed156"),
            "admitted",
        )
        .unwrap();
    shared
        .lock()
        .unwrap()
        .idempotency
        .record_guidance_event(
            unix_now_secs() - 90,
            "other-parent-b",
            "89000000-0000-4000-8000-000000000102",
            "interrupt",
            "send_idempotent",
            Some("seed156"),
            "admitted",
        )
        .unwrap();
    shared
        .lock()
        .unwrap()
        .idempotency
        .record_guidance_event(
            unix_now_secs() - 80,
            "other-parent-c",
            "89000000-0000-4000-8000-000000000102",
            "interrupt",
            "send_idempotent",
            Some("seed156"),
            "admitted",
        )
        .unwrap();
    shared
        .lock()
        .unwrap()
        .store
        .create_request(
            "cancel-request156",
            "89000000-0000-4000-8000-000000000200",
            "89000000-0000-4000-8000-000000000102",
            60,
        )
        .unwrap();
    let command = WrapperToDaemon::CancelRequest {
        id: "cancel-request156".into(),
        sender: "89000000-0000-4000-8000-000000000200".into(),
        reason: Some("annulation156".into()),
    };
    let before = shared
        .lock()
        .unwrap()
        .idempotency
        .count_interrupts_target(
            "89000000-0000-4000-8000-000000000102",
            unix_now_secs() - 600,
        )
        .unwrap()
        .0;
    let response = handle_wrapper_message(
        "owner-89000000-0000-4000-8000-000000000200",
        command.clone(),
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response,DaemonToWrapper::RequestCancelled {ref state,..} if state=="cancelled"),
        "{response:?}"
    );
    assert!(
        matches!(read_control(&mut reader),DaemonToWrapper::CancelDelivery {id,interrupt_active,..} if id=="cancel-request156" && interrupt_active==Some(false))
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .store
            .get_request("cancel-request156")
            .unwrap()
            .unwrap()
            .state,
        "cancelled"
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        before
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_cancel_interruption_comptee() {
    let (mut st, config) = state_with_registered_agent("g15693c475ea");
    let (writer, mut reader) = control_socket("g15693c475ea");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "g156-client",
        "012_scope_guard_tests156",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "g156-client", "conn-1");
    shared
        .lock()
        .unwrap()
        .store
        .create_request(
            "cancel-request156",
            "89000000-0000-4000-8000-000000000200",
            "89000000-0000-4000-8000-000000000102",
            60,
        )
        .unwrap();
    let command = WrapperToDaemon::CancelRequest {
        id: "cancel-request156".into(),
        sender: "89000000-0000-4000-8000-000000000200".into(),
        reason: Some("annulation156".into()),
    };
    let before = shared
        .lock()
        .unwrap()
        .idempotency
        .count_interrupts_target(
            "89000000-0000-4000-8000-000000000102",
            unix_now_secs() - 600,
        )
        .unwrap()
        .0;
    let response = handle_wrapper_message(
        "owner-89000000-0000-4000-8000-000000000200",
        command.clone(),
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response,DaemonToWrapper::RequestCancelled {ref state,..} if state=="cancelled"),
        "{response:?}"
    );
    assert!(
        matches!(read_control(&mut reader),DaemonToWrapper::CancelDelivery {id,interrupt_active,..} if id=="cancel-request156" && interrupt_active==Some(true))
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .store
            .get_request("cancel-request156")
            .unwrap()
            .unwrap()
            .state,
        "cancelled"
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        before + 1
    );
    let replay = handle_wrapper_message(
        "owner-89000000-0000-4000-8000-000000000200",
        command,
        &shared,
    )
    .unwrap();
    assert!(matches!(replay,DaemonToWrapper::RequestCancelled {state,..} if state=="cancelled"));
    // Le rejeu peut notifier le retrait ; il ne doit jamais réinterrompre.
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    if reader.read_line(&mut line).is_ok() && !line.is_empty() {
        assert!(matches!(
            decode::<DaemonToWrapper>(line.trim()).unwrap(),
            DaemonToWrapper::CancelDelivery {
                interrupt_active: Some(false),
                ..
            }
        ));
    }
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        before + 1
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_human_cli_sans_preuve_refuse() {
    let (mut st, config) = state_with_registered_agent("g156-cli-bare");
    let (writer, mut target_reader) = control_socket("g156-cli-target");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    let root = std::env::temp_dir().join(format!(
        "g156cli-{}",
        &Uuid::new_v4().simple().to_string()[..8]
    ));
    for dir in ["", "home", "ns", "tmp"] {
        crate::environment::ensure_private_directory(&root.join(dir)).unwrap();
    }
    let socket = root.join("ns/bridget.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    listener.set_nonblocking(true).unwrap();
    let current = std::env::current_exe().unwrap();
    let binary = current.parent().unwrap().parent().unwrap().join("bridget");
    assert!(
        binary.is_file(),
        "binaire Bridget construit par T013 attendu : {} (current_exe={})",
        binary.display(),
        current.display()
    );
    let server = shared.clone();
    let pump = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let (stream, _) = loop {
            match listener.accept() {
                Ok(peer) => break peer,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "CLI privée jamais connectée");
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("accept CLI privée : {error}"),
            }
        };
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let writer = Arc::new(Mutex::new(BufWriter::new(stream.try_clone().unwrap())));
        server
            .lock()
            .unwrap()
            .connections
            .insert("cli-bare156".into(), writer.clone());
        let mut reader = BufReader::new(stream);
        let mut commands = Vec::new();
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap() == 0 {
                break;
            }
            let command = decode::<WrapperToDaemon>(line.trim()).unwrap();
            commands.push(command.clone());
            if let Some(response) = handle_wrapper_message("cli-bare156", command, &server) {
                let mut writer = writer.lock().unwrap();
                writeln!(writer, "{}", encode(&response).unwrap()).unwrap();
                writer.flush().unwrap();
            }
        }
        commands
    });
    let output = Command::new(&binary)
        .args([
            "send",
            "--intent",
            "steer",
            "--to",
            "89000000-0000-4000-8000-000000000102",
            "CLI privée sans preuve156",
        ])
        .env_clear()
        .env("HOME", root.join("home"))
        .env("BRIDGET_HOME", root.join("ns"))
        .env("BRIDGET_SOCKET", &socket)
        .env("TMPDIR", root.join("tmp"))
        .env("PATH", "/usr/bin:/bin")
        .current_dir(&root)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let commands = pump.join().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(1),
        "binary={} namespace={} stderr={stderr}",
        binary.display(),
        root.display()
    );
    assert!(
        stderr.contains("identity_unattested"),
        "refus daemon réel attendu : {stderr}"
    );
    assert!(commands.iter().any(|command|matches!(command,WrapperToDaemon::Send(message) if message.from=="human" && message.intent==Some(bridget_core::MessageIntent::SteerCurrent))),"vraie trame CLI : {commands:?}");
    assert!(commands.iter().any(|command|matches!(command,WrapperToDaemon::Register {agent_type,..} if agent_type=="cli")),"inscription CLI historique : {commands:?}");
    let st = shared.lock().unwrap();
    assert_eq!(
        st.connection_roles.get("cli-bare156"),
        Some(&ConnectionRole::Wrapper),
        "la connexion historique n’est pas promue Attach"
    );
    assert!(
        live_connection_identity(&st, "cli-bare156").is_none(),
        "Register CLI sans instance ne produit pas une identité attestée : {:?}",
        st.conn_instances.get("cli-bare156")
    );
    drop(st);
    target_reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut line = String::new();
    assert!(
        target_reader.read_line(&mut line).is_err(),
        "aucune remise : {line}"
    );
    drop(shared);
    std::fs::remove_dir_all(root).unwrap();
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_combo_envoi_controle_annulation_pause() {
    let (mut st, config) = state_with_registered_agent("g156-combo");
    let (writer, mut reader) = control_socket("g156-combo");
    st.connections.insert("conn-1".into(), writer);
    st.execution_store
        .record_starting(
            "combo-submission",
            "combo-execution",
            "89000000-0000-4000-8000-000000000102",
            unix_now_secs(),
        )
        .unwrap();
    st.execution_store
        .transition_if_current(
            "combo-execution",
            "starting",
            0,
            1,
            "running",
            "provider_accepted",
            unix_now_secs(),
        )
        .unwrap();
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "g156-client",
        "012_scope_guard_tests156",
        "89000000-0000-4000-8000-000000000200",
    );
    guidance156_link_parent(&mut shared.lock().unwrap(), "g156-client", "conn-1");
    let mut message = idempotent_message("combo first send156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let result = handle_wrapper_message(
        "g156-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "combo-send-first156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(
            result,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ),
        "{result:?}"
    );
    read_control(&mut reader);
    shared
        .lock()
        .unwrap()
        .store
        .create_request(
            "combo-cancel-blocked",
            "89000000-0000-4000-8000-000000000200",
            "89000000-0000-4000-8000-000000000102",
            60,
        )
        .unwrap();
    assert!(matches!(
        handle_wrapper_message(
            "owner-89000000-0000-4000-8000-000000000200",
            WrapperToDaemon::CancelRequest {
                id: "combo-cancel-blocked".into(),
                sender: "89000000-0000-4000-8000-000000000200".into(),
                reason: None
            },
            &shared
        ),
        Some(DaemonToWrapper::RequestCancelled { .. })
    ));
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::CancelDelivery {
            interrupt_active: Some(false),
            ..
        }
    ));
    let command = ExecutionControlCommand {
        version: 1,
        command_id: "combo-control-blocked".into(),
        execution_id: "combo-execution".into(),
        generation: 1,
        revision: 1,
        operation: ExecutionControlOperation::Interrupt,
        message: None,
    };
    let response = handle_execution_control(
        Some("g156-client"),
        "012_scope_combo_control156",
        command,
        &mut shared.lock().unwrap(),
    );
    assert!(matches!(
        response,
        DaemonToWrapper::ControlExecutionResult {
            outcome: ExecutionControlOutcome::Refused(ExecutionControlRefusal::NotPermitted),
            ..
        }
    ));
    let db = rusqlite::Connection::open(&config.db_path).unwrap();
    db.execute(
        "UPDATE guidance_events SET at=?1 WHERE outcome='admitted'",
        [unix_now_secs() - 31],
    )
    .unwrap();
    let command = ExecutionControlCommand {
        version: 1,
        command_id: "combo-control-admitted".into(),
        execution_id: "combo-execution".into(),
        generation: 1,
        revision: 1,
        operation: ExecutionControlOperation::Interrupt,
        message: None,
    };
    let response = handle_execution_control(
        Some("g156-client"),
        "012_scope_combo_control156",
        command,
        &mut shared.lock().unwrap(),
    );
    assert!(
        matches!(
            response,
            DaemonToWrapper::ControlExecutionResult {
                outcome: ExecutionControlOutcome::OutcomeUnknown,
                ..
            }
        ),
        "{response:?}"
    );
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::ControlExecutionDispatch { .. }
    ));
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        2
    );
    db.execute(
        "UPDATE guidance_events SET at=?1 WHERE outcome='admitted'",
        [unix_now_secs() - 31],
    )
    .unwrap();
    shared
        .lock()
        .unwrap()
        .store
        .create_request(
            "combo-cancel-admitted",
            "89000000-0000-4000-8000-000000000200",
            "89000000-0000-4000-8000-000000000102",
            60,
        )
        .unwrap();
    assert!(matches!(
        handle_wrapper_message(
            "owner-89000000-0000-4000-8000-000000000200",
            WrapperToDaemon::CancelRequest {
                id: "combo-cancel-admitted".into(),
                sender: "89000000-0000-4000-8000-000000000200".into(),
                reason: None
            },
            &shared
        ),
        Some(DaemonToWrapper::RequestCancelled { .. })
    ));
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::CancelDelivery {
            interrupt_active: Some(true),
            ..
        }
    ));
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        3
    );
    let mut message = idempotent_message("combo cancel blocks send156");
    message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
    let result = handle_wrapper_message(
        "g156-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: "combo-send-blocked156".into(),
            issued_at: unix_now_secs(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(result,DaemonToWrapper::IdempotencyResult {issue:IdempotencyIssue::Rejected {category,..},..} if category=="interrupt_rate_limited")
    );
    // Trois interruptions humaines admises malgré le quota agent complet.
    let (writer, _human_reader) = control_socket("g156-combo-human");
    shared
        .lock()
        .unwrap()
        .connections
        .insert("attach156".into(), writer);
    assert!(matches!(
        handle_wrapper_message(
            "attach156",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted { .. })
    ));
    for n in 0..3 {
        let mut message = idempotent_message(&format!("combo human interrupt156-{n}"));
        message.from = "human".into();
        message.intent = Some(bridget_core::MessageIntent::InterruptAndStart);
        let result =
            handle_wrapper_message("attach156", WrapperToDaemon::Send(message), &shared).unwrap();
        assert!(matches!(result, DaemonToWrapper::Ack { .. }), "{result:?}");
        read_control(&mut reader);
    }
    {
        let st = shared.lock().unwrap();
        assert!(
            crate::referent_control::set(
                st.store.connection(),
                crate::referent_control::ControlMutation {
                    command_id: "combo-pause156",
                    expected_generation: 0,
                    paused: Some(true),
                    auto_objectives_cap: None,
                    reason: Some("oracle156"),
                    actor: "human",
                    now: unix_now_secs(),
                    agent_posture: None,
                    auto_reassignment: None
                }
            )
            .unwrap()
            .is_ok()
        );
    }
    for (n, intent) in [
        bridget_core::MessageIntent::SteerCurrent,
        bridget_core::MessageIntent::InterruptAndStart,
    ]
    .into_iter()
    .enumerate()
    {
        let mut message = idempotent_message(&format!("combo paused idempotent156-{n}"));
        message.intent = Some(intent);
        let result = handle_wrapper_message(
            "g156-client",
            WrapperToDaemon::SendIdempotent {
                message,
                message_id: format!("combo-paused-idempotent-{n}"),
                issued_at: unix_now_secs(),
            },
            &shared,
        )
        .unwrap();
        assert!(
            matches!(result,DaemonToWrapper::IdempotencyResult {issue:IdempotencyIssue::Rejected {category,..},..} if category=="referent_paused")
        );
        let mut message = idempotent_message(&format!("combo paused legacy156-{n}"));
        message.intent = Some(intent);
        assert!(
            matches!(handle_wrapper_message("owner-89000000-0000-4000-8000-000000000200",WrapperToDaemon::Send(message),&shared),Some(DaemonToWrapper::Nack {reason,..}) if reason.contains("referent_paused"))
        );
        let message = if intent == bridget_core::MessageIntent::SteerCurrent {
            let mut m = idempotent_message("combo paused control steer156");
            m.intent = Some(intent);
            Some(m)
        } else {
            None
        };
        let command = ExecutionControlCommand {
            version: 1,
            command_id: format!("combo-paused-control-{n}"),
            execution_id: "combo-execution".into(),
            generation: 1,
            revision: 1,
            operation: if message.is_some() {
                ExecutionControlOperation::SteerCurrent
            } else {
                ExecutionControlOperation::Interrupt
            },
            message,
        };
        assert!(matches!(
            handle_execution_control(
                Some("g156-client"),
                "012_scope_combo_control156",
                command,
                &mut shared.lock().unwrap()
            ),
            DaemonToWrapper::ControlExecutionResult {
                outcome: ExecutionControlOutcome::Refused(ExecutionControlRefusal::NotPermitted),
                ..
            }
        ));
        let id = format!("combo-paused-cancel-{n}");
        shared
            .lock()
            .unwrap()
            .store
            .create_request(
                &id,
                "89000000-0000-4000-8000-000000000200",
                "89000000-0000-4000-8000-000000000102",
                60,
            )
            .unwrap();
        assert!(matches!(
            handle_wrapper_message(
                "owner-89000000-0000-4000-8000-000000000200",
                WrapperToDaemon::CancelRequest {
                    id,
                    sender: "89000000-0000-4000-8000-000000000200".into(),
                    reason: None
                },
                &shared
            ),
            Some(DaemonToWrapper::RequestCancelled { .. })
        ));
        assert!(matches!(
            read_control(&mut reader),
            DaemonToWrapper::CancelDelivery {
                interrupt_active: Some(false),
                ..
            }
        ));
    }
    let before: i64 = db
        .query_row("SELECT count(*) FROM guidance_events", [], |row| row.get(0))
        .unwrap();
    let command = ExecutionControlCommand {
        version: 1,
        command_id: "combo-internal-pause".into(),
        execution_id: "combo-execution".into(),
        generation: 1,
        revision: 1,
        operation: ExecutionControlOperation::Interrupt,
        message: None,
    };
    assert!(matches!(
        handle_execution_control(
            None,
            "012_scope_combo_pause156",
            command,
            &mut shared.lock().unwrap()
        ),
        DaemonToWrapper::ControlExecutionResult {
            outcome: ExecutionControlOutcome::OutcomeUnknown,
            ..
        }
    ));
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::ControlExecutionDispatch { .. }
    ));
    let after: i64 = db
        .query_row("SELECT count(*) FROM guidance_events", [], |row| row.get(0))
        .unwrap();
    assert_eq!(after, before, "pause interne hors journal et quota");
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .idempotency
            .count_interrupts_target(
                "89000000-0000-4000-8000-000000000102",
                unix_now_secs() - 600
            )
            .unwrap()
            .0,
        3
    );
    drop(db);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn held159_list_held_dans_l_ordre() {
    let label = format!("h159-list-{}", &Uuid::new_v4().simple().to_string()[..12]);
    let (mut st, config) = state_with_registered_agent(&label);
    st.registry = crate::registry::AgentRegistry::from_json(
            r#"{"agents":{"claude":{"command":"/bin/sh","protocol":"acp","queue_capacity":7,"held_queue":true}}}"#,
            "fixture-held159-list.json",
        ).unwrap();
    let (target_writer, mut target_reader) = control_socket(&label);
    target_reader
        .get_mut()
        .set_read_timeout(Some(Duration::from_millis(20)))
        .unwrap();
    st.connections.insert("conn-1".into(), target_writer);
    let shared = Arc::new(Mutex::new(st));
    let target = "89000000-0000-4000-8000-000000000102";
    let scope = "012_scope_held159_list_aaaaaaaa";
    let conn = "held159-list-reader";
    let (writer, _peer) = control_socket("held159-list-reader");
    shared
        .lock()
        .unwrap()
        .connections
        .insert(conn.into(), writer);
    assert!(matches!(
        handle_wrapper_message(
            conn,
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach,
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    assert!(
        matches!(handle_wrapper_message(conn, WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: scope.into(),
            capabilities: vec![ClientCapability::QueueListV1],
        }, &shared), Some(DaemonToWrapper::ClientWelcome { capabilities, .. })
            if capabilities == vec![ClientCapability::QueueListV1])
    );
    let now = unix_now_secs();
    // Fixtures gravées par la transaction de remise existante, sans sélection SQL copiée.
    {
        let mut st = shared.lock().unwrap();
        for (id, held, sender, body) in [
            (
                "held-z",
                true,
                "89000000-0000-4000-8000-000000000200",
                "rang100",
            ),
            (
                "held-a",
                true,
                "89000000-0000-4000-8000-000000000200",
                "rang300",
            ),
            (
                "held-m",
                true,
                "89000000-0000-4000-8000-000000000200",
                "rang200",
            ),
        ] {
            let key = IdempotencyKey::new(scope, OperationKind::Send, id).unwrap();
            let mut message = idempotent_message(body);
            message.id = id.into();
            message.from = sender.into();
            let bytes = serde_json::to_vec(&message).unwrap();
            st.idempotency
                .reserve(
                    &key,
                    &bytes,
                    now,
                    CLIENT_IDEMPOTENCY_HORIZON_SECS,
                    now,
                    CLIENT_ISSUED_AT_TOLERANCE_SECS,
                )
                .unwrap();
            let delivery = crate::idempotency::SendDelivery {
                delivery_id: id.into(),
                recipient_instance_id: "instance-1".into(),
                delivery_generation: 77,
                expires_at: now + CLIENT_IDEMPOTENCY_HORIZON_SECS,
                message_bytes: bytes,
            };
            st.idempotency
                .begin_send_delivery_with_guidance(
                    &key,
                    &delivery,
                    None,
                    None,
                    None,
                    None,
                    held.then_some(target),
                )
                .unwrap();
        }
    }
    // Le rang d'édition diffère de l'insertion et de l'ordre lexical des identifiants.
    let sql = rusqlite::Connection::open(&config.db_path).unwrap();
    for (id, rank) in [("held-z", 100), ("held-a", 300), ("held-m", 200)] {
        sql.execute(
            "UPDATE send_deliveries SET queue_rank=?1 WHERE delivery_id=?2",
            rusqlite::params![rank, id],
        )
        .unwrap();
    }
    let response = handle_wrapper_message(
        conn,
        WrapperToDaemon::QueueList {
            target: target.into(),
        },
        &shared,
    )
    .unwrap();
    let DaemonToWrapper::QueueListResult {
        entries,
        queue_bound,
    } = response
    else {
        panic!("résultat QueueList attendu : {response:?}");
    };
    assert_eq!(
        entries.len(),
        3,
        "les retenus durables doivent être visibles"
    );
    assert_eq!(
        entries
            .iter()
            .map(|e| e["message_id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["held-z", "held-m", "held-a"]
    );
    for (index, entry) in entries.iter().enumerate() {
        assert_eq!(entry["position"], index + 1);
        assert_eq!(entry["state"], "held");
        assert!(entry.get("body").is_none());
    }
    assert_eq!(queue_bound, Some(7));
    drop(sql);
    let mut line = String::new();
    assert!(
        matches!(target_reader.read_line(&mut line), Err(error)
            if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut)),
        "QueueList ne doit pas écrire au porteur"
    );
    drop(shared);
    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn held159_list_released_pour_dispatching() {
    let label = format!("h159-list-{}", &Uuid::new_v4().simple().to_string()[..12]);
    let (mut st, config) = state_with_registered_agent(&label);
    st.registry = crate::registry::AgentRegistry::from_json(
            r#"{"agents":{"claude":{"command":"/bin/sh","protocol":"acp","queue_capacity":7,"held_queue":true}}}"#,
            "fixture-held159-list.json",
        ).unwrap();
    let (target_writer, mut target_reader) = control_socket(&label);
    target_reader
        .get_mut()
        .set_read_timeout(Some(Duration::from_millis(20)))
        .unwrap();
    st.connections.insert("conn-1".into(), target_writer);
    let shared = Arc::new(Mutex::new(st));
    let target = "89000000-0000-4000-8000-000000000102";
    let scope = "012_scope_held159_list_aaaaaaaa";
    let conn = "held159-list-reader";
    let (writer, _peer) = control_socket("held159-list-reader");
    shared
        .lock()
        .unwrap()
        .connections
        .insert(conn.into(), writer);
    assert!(matches!(
        handle_wrapper_message(
            conn,
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach,
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    assert!(
        matches!(handle_wrapper_message(conn, WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: scope.into(),
            capabilities: vec![ClientCapability::QueueListV1],
        }, &shared), Some(DaemonToWrapper::ClientWelcome { capabilities, .. })
            if capabilities == vec![ClientCapability::QueueListV1])
    );
    let now = unix_now_secs();
    // Fixtures gravées par la transaction de remise existante, sans sélection SQL copiée.
    {
        let mut st = shared.lock().unwrap();
        {
            let (id, held, sender, body) = (
                "released159",
                false,
                "89000000-0000-4000-8000-000000000200",
                "déjà remis au porteur",
            );
            let key = IdempotencyKey::new(scope, OperationKind::Send, id).unwrap();
            let mut message = idempotent_message(body);
            message.id = id.into();
            message.from = sender.into();
            let bytes = serde_json::to_vec(&message).unwrap();
            st.idempotency
                .reserve(
                    &key,
                    &bytes,
                    now,
                    CLIENT_IDEMPOTENCY_HORIZON_SECS,
                    now,
                    CLIENT_ISSUED_AT_TOLERANCE_SECS,
                )
                .unwrap();
            let delivery = crate::idempotency::SendDelivery {
                delivery_id: id.into(),
                recipient_instance_id: "instance-1".into(),
                delivery_generation: 77,
                expires_at: now + CLIENT_IDEMPOTENCY_HORIZON_SECS,
                message_bytes: bytes,
            };
            st.idempotency
                .begin_send_delivery_with_guidance(
                    &key,
                    &delivery,
                    None,
                    None,
                    None,
                    None,
                    held.then_some(target),
                )
                .unwrap();
        }
    }
    let response = handle_wrapper_message(
        conn,
        WrapperToDaemon::QueueList {
            target: target.into(),
        },
        &shared,
    )
    .unwrap();
    let DaemonToWrapper::QueueListResult {
        entries,
        queue_bound,
    } = response
    else {
        panic!("résultat QueueList attendu : {response:?}");
    };
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["message_id"], "released159");
    assert_eq!(
        entries[0]["state"], "released",
        "dispatching n'est plus éditable"
    );
    assert_eq!(queue_bound, Some(7));
    let mut line = String::new();
    assert!(
        matches!(target_reader.read_line(&mut line), Err(error)
            if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut)),
        "QueueList ne doit pas écrire au porteur"
    );
    drop(shared);
    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn held159_list_filtre_par_droits_de_156() {
    let label = format!("h159-list-{}", &Uuid::new_v4().simple().to_string()[..12]);
    let (mut st, config) = state_with_registered_agent(&label);
    st.registry = crate::registry::AgentRegistry::from_json(
            r#"{"agents":{"claude":{"command":"/bin/sh","protocol":"acp","queue_capacity":7,"held_queue":true}}}"#,
            "fixture-held159-list.json",
        ).unwrap();
    let (target_writer, mut target_reader) = control_socket(&label);
    target_reader
        .get_mut()
        .set_read_timeout(Some(Duration::from_millis(20)))
        .unwrap();
    st.connections.insert("conn-1".into(), target_writer);
    let shared = Arc::new(Mutex::new(st));
    let target = "89000000-0000-4000-8000-000000000102";
    let scope = "012_scope_held159_list_aaaaaaaa";
    let conn = "held159-list-reader";
    let sender = "89000000-0000-4000-8000-000000000200";
    // Le helper existant atteste le producteur ; la lecture négocie sa propre connexion.
    negotiate_idempotent_client(&shared, "held159-list-producer", scope, sender);
    let (instance_id, credential) = {
        let st = shared.lock().unwrap();
        let owner = &st.router.get_agent(sender).unwrap().connection_id;
        (
            st.conn_instances[owner].clone(),
            st.identity_credentials[owner].clone(),
        )
    };
    assert!(matches!(
        handle_wrapper_message(
            conn,
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client,
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        })
    ));
    assert!(matches!(
        handle_wrapper_message(
            conn,
            WrapperToDaemon::RegisterAuxiliary {
                agent_id: sender.into(),
                instance_id,
                credential,
            },
            &shared
        ),
        Some(DaemonToWrapper::Registered {
            credential: None,
            ..
        })
    ));
    assert!(
        matches!(handle_wrapper_message(conn, WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: scope.into(),
            capabilities: vec![ClientCapability::QueueListV1],
        }, &shared), Some(DaemonToWrapper::ClientWelcome { capabilities, .. })
            if capabilities == vec![ClientCapability::QueueListV1])
    );
    let bare_conn = "held159-list-bare";
    let bare_scope = "012_scope_held159_bare_aaaaaaaa";
    let foreign_scope = "012_scope_held159_bare_bbbbbbbb";
    // Deux vrais clients nus : leurs scopes viennent du ClientHello, sans auxiliaire.
    for (reader_conn, issuer_scope) in [
        (bare_conn, bare_scope),
        ("held159-list-foreign", foreign_scope),
    ] {
        assert!(matches!(
            handle_wrapper_message(
                reader_conn,
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Client
                },
                &shared
            ),
            Some(DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Client
            })
        ));
        assert!(
            matches!(handle_wrapper_message(reader_conn, WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: issuer_scope.into(),
                capabilities: vec![ClientCapability::SendIdempotent, ClientCapability::QueueListV1],
            }, &shared), Some(DaemonToWrapper::ClientWelcome { capabilities, .. })
                if capabilities.len() == 2
                    && capabilities.contains(&ClientCapability::SendIdempotent)
                    && capabilities.contains(&ClientCapability::QueueListV1))
        );
    }
    {
        let st = shared.lock().unwrap();
        assert!(live_connection_identity(&st, bare_conn).is_none());
        assert_eq!(
            guidance_source(&st, Some(bare_conn)).0,
            crate::guidance::GuidanceActor::BareClient
        );
        assert_eq!(st.client_negotiations[bare_conn].issuer_scope, bare_scope);
    }
    let now = unix_now_secs();
    // Fixtures gravées par la transaction de remise existante, sans sélection SQL copiée.
    {
        let mut st = shared.lock().unwrap();
        for (id, held, sender, body, row_scope) in [
            (
                "foreign159",
                true,
                "89000000-0000-4000-8000-000000000300",
                "PRIVE_NE_DOIT_PAS_ETRE_PROJETE",
                scope,
            ),
            (
                "own159a",
                true,
                "89000000-0000-4000-8000-000000000200",
                "visible A",
                scope,
            ),
            (
                "own159b",
                false,
                "89000000-0000-4000-8000-000000000200",
                "visible B",
                scope,
            ),
            ("bare159a", true, "human", "visible nu A", bare_scope),
            ("bare159b", false, "human", "visible nu B", bare_scope),
            (
                "bare159foreign",
                true,
                "human",
                "PRIVE_SCOPE_NU_NE_DOIT_PAS_ETRE_PROJETE",
                foreign_scope,
            ),
        ] {
            let key = IdempotencyKey::new(row_scope, OperationKind::Send, id).unwrap();
            let mut message = idempotent_message(body);
            message.id = id.into();
            message.from = sender.into();
            let bytes = serde_json::to_vec(&message).unwrap();
            st.idempotency
                .reserve(
                    &key,
                    &bytes,
                    now,
                    CLIENT_IDEMPOTENCY_HORIZON_SECS,
                    now,
                    CLIENT_ISSUED_AT_TOLERANCE_SECS,
                )
                .unwrap();
            let delivery = crate::idempotency::SendDelivery {
                delivery_id: id.into(),
                recipient_instance_id: "instance-1".into(),
                delivery_generation: 77,
                expires_at: now + CLIENT_IDEMPOTENCY_HORIZON_SECS,
                message_bytes: bytes,
            };
            st.idempotency
                .begin_send_delivery_with_guidance(
                    &key,
                    &delivery,
                    None,
                    None,
                    None,
                    None,
                    held.then_some(target),
                )
                .unwrap();
        }
    }
    let before_version: i64 = shared
        .lock()
        .unwrap()
        .store
        .connection()
        .query_row("PRAGMA data_version", [], |row| row.get(0))
        .unwrap();
    let response = handle_wrapper_message(
        conn,
        WrapperToDaemon::QueueList {
            target: target.into(),
        },
        &shared,
    )
    .unwrap();
    let DaemonToWrapper::QueueListResult {
        entries,
        queue_bound,
    } = response
    else {
        panic!("résultat QueueList attendu : {response:?}");
    };
    assert_eq!(
        entries.len(),
        2,
        "un équipier ne lit que ses propres envois"
    );
    assert_eq!(entries[0]["message_id"], "own159a");
    assert_eq!(entries[1]["message_id"], "own159b");
    assert_eq!(entries[0]["state"], "held");
    assert_eq!(entries[1]["state"], "released");
    assert_eq!(entries[0]["position"], 1, "position après filtre de droits");
    assert!(
        !serde_json::to_string(&entries)
            .unwrap()
            .contains("PRIVE_NE_DOIT_PAS_ETRE_PROJETE")
    );
    assert_eq!(queue_bound, Some(7));
    let bare_response = handle_wrapper_message(
        bare_conn,
        WrapperToDaemon::QueueList {
            target: target.into(),
        },
        &shared,
    )
    .unwrap();
    let DaemonToWrapper::QueueListResult {
        entries: bare_entries,
        queue_bound,
    } = bare_response
    else {
        panic!("QueueList du vrai client nu attendu : {bare_response:?}");
    };
    assert_eq!(
        bare_entries.len(),
        2,
        "scope négocié : Held et Released propres seulement"
    );
    assert_eq!(bare_entries[0]["message_id"], "bare159a");
    assert_eq!(bare_entries[1]["message_id"], "bare159b");
    assert_eq!(bare_entries[0]["state"], "held");
    assert_eq!(bare_entries[1]["state"], "released");
    assert_eq!(bare_entries[0]["position"], 1);
    assert_eq!(bare_entries[1]["position"], 2);
    assert_eq!(queue_bound, Some(7));
    let projected = serde_json::to_string(&bare_entries).unwrap();
    assert!(!projected.contains("PRIVE_NE_DOIT_PAS_ETRE_PROJETE"));
    assert!(!projected.contains("PRIVE_SCOPE_NU_NE_DOIT_PAS_ETRE_PROJETE"));
    let after_version: i64 = shared
        .lock()
        .unwrap()
        .store
        .connection()
        .query_row("PRAGMA data_version", [], |row| row.get(0))
        .unwrap();
    assert_eq!(
        after_version, before_version,
        "les deux lectures ne committent aucune écriture"
    );
    let mut line = String::new();
    assert!(
        matches!(target_reader.read_line(&mut line), Err(error)
            if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut)),
        "QueueList ne doit pas écrire au porteur"
    );
    drop(shared);
    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn held159_list_borne_200() {
    let label = format!("h159-list-{}", &Uuid::new_v4().simple().to_string()[..12]);
    let (mut st, config) = state_with_registered_agent(&label);
    st.registry = crate::registry::AgentRegistry::from_json(
            r#"{"agents":{"claude":{"command":"/bin/sh","protocol":"acp","queue_capacity":256,"held_queue":true}}}"#,
            "fixture-held159-list.json",
        ).unwrap();
    let (target_writer, mut target_reader) = control_socket(&label);
    target_reader
        .get_mut()
        .set_read_timeout(Some(Duration::from_millis(20)))
        .unwrap();
    st.connections.insert("conn-1".into(), target_writer);
    let shared = Arc::new(Mutex::new(st));
    let target = "89000000-0000-4000-8000-000000000102";
    let scope = "012_scope_held159_list_aaaaaaaa";
    let conn = "held159-list-reader";
    let (writer, _peer) = control_socket("held159-list-reader");
    shared
        .lock()
        .unwrap()
        .connections
        .insert(conn.into(), writer);
    assert!(matches!(
        handle_wrapper_message(
            conn,
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach,
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    assert!(
        matches!(handle_wrapper_message(conn, WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: scope.into(),
            capabilities: vec![ClientCapability::QueueListV1],
        }, &shared), Some(DaemonToWrapper::ClientWelcome { capabilities, .. })
            if capabilities == vec![ClientCapability::QueueListV1])
    );
    let now = unix_now_secs();
    {
        let mut st = shared.lock().unwrap();
        for index in 0..201 {
            let id = format!("held-bound159-{index:03}");
            let key = IdempotencyKey::new(scope, OperationKind::Send, &id).unwrap();
            let mut message = idempotent_message(&format!("message{index:03}"));
            message.id = id.clone();
            let bytes = serde_json::to_vec(&message).unwrap();
            st.idempotency
                .reserve(
                    &key,
                    &bytes,
                    now,
                    CLIENT_IDEMPOTENCY_HORIZON_SECS,
                    now,
                    CLIENT_ISSUED_AT_TOLERANCE_SECS,
                )
                .unwrap();
            let delivery = crate::idempotency::SendDelivery {
                delivery_id: id,
                recipient_instance_id: "instance-1".into(),
                delivery_generation: 77,
                expires_at: now + CLIENT_IDEMPOTENCY_HORIZON_SECS,
                message_bytes: bytes,
            };
            st.idempotency
                .begin_send_delivery_with_guidance(
                    &key,
                    &delivery,
                    None,
                    None,
                    None,
                    None,
                    Some(target),
                )
                .unwrap();
        }
    }
    let response = handle_wrapper_message(
        conn,
        WrapperToDaemon::QueueList {
            target: target.into(),
        },
        &shared,
    )
    .unwrap();
    let DaemonToWrapper::QueueListResult {
        entries,
        queue_bound,
    } = response
    else {
        panic!("résultat QueueList attendu : {response:?}");
    };
    assert_eq!(entries.len(), 200);
    assert_eq!(entries[199]["message_id"], "held-bound159-199");
    assert_eq!(entries[199]["position"], 200);
    assert!(entries.iter().all(|entry| entry["state"] == "held"));
    assert_eq!(queue_bound, Some(256));
    let mut line = String::new();
    assert!(
        matches!(target_reader.read_line(&mut line), Err(error)
            if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut)),
        "QueueList ne doit pas écrire au porteur"
    );
    drop(shared);
    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn held159_list_queue_bound_capacite_du_type_sinon_null() {
    let label = format!("h159-list-{}", &Uuid::new_v4().simple().to_string()[..12]);
    let (mut st, config) = state_with_registered_agent(&label);
    st.registry = crate::registry::AgentRegistry::from_json(
            r#"{"agents":{"claude":{"command":"/bin/sh","protocol":"acp","queue_capacity":7,"held_queue":true}}}"#,
            "fixture-held159-list.json",
        ).unwrap();
    let (target_writer, mut target_reader) = control_socket(&label);
    target_reader
        .get_mut()
        .set_read_timeout(Some(Duration::from_millis(20)))
        .unwrap();
    st.connections.insert("conn-1".into(), target_writer);
    let shared = Arc::new(Mutex::new(st));
    let target = "89000000-0000-4000-8000-000000000102";
    let scope = "012_scope_held159_list_aaaaaaaa";
    let conn = "held159-list-reader";
    let (writer, _peer) = control_socket("held159-list-reader");
    shared
        .lock()
        .unwrap()
        .connections
        .insert(conn.into(), writer);
    assert!(matches!(
        handle_wrapper_message(
            conn,
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach,
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    assert!(
        matches!(handle_wrapper_message(conn, WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: scope.into(),
            capabilities: vec![ClientCapability::QueueListV1],
        }, &shared), Some(DaemonToWrapper::ClientWelcome { capabilities, .. })
            if capabilities == vec![ClientCapability::QueueListV1])
    );
    for (enabled, transport, expected) in [
        (true, "acp", Some(7)),
        (false, "acp", None),
        (true, "t3code", None),
        (false, "t3code", None),
    ] {
        {
            let mut st = shared.lock().unwrap();
            st.registry = crate::registry::AgentRegistry::from_json(
                &serde_json::json!({"agents":{"claude":{"command":"/bin/sh","protocol":"acp",
                        "queue_capacity":7,"held_queue":enabled}}})
                .to_string(),
                "fixture-held159-list.json",
            )
            .unwrap();
            st.presences.get_mut("instance-1").unwrap().transport = transport.into();
        }
        let response = handle_wrapper_message(
            conn,
            WrapperToDaemon::QueueList {
                target: target.into(),
            },
            &shared,
        )
        .unwrap();
        assert!(
            matches!(response, DaemonToWrapper::QueueListResult { queue_bound, .. }
                if queue_bound == expected),
            "held_queue={enabled}, transport={transport} : {response:?}"
        );
    }
    let mut line = String::new();
    assert!(
        matches!(target_reader.read_line(&mut line), Err(error)
            if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut)),
        "QueueList ne doit pas écrire au porteur"
    );
    drop(shared);
    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn held159_list_ne_modifie_rien() {
    let label = format!("h159-list-{}", &Uuid::new_v4().simple().to_string()[..12]);
    let (mut st, config) = state_with_registered_agent(&label);
    st.registry = crate::registry::AgentRegistry::from_json(
            r#"{"agents":{"claude":{"command":"/bin/sh","protocol":"acp","queue_capacity":7,"held_queue":true}}}"#,
            "fixture-held159-list.json",
        ).unwrap();
    let (target_writer, mut target_reader) = control_socket(&label);
    target_reader
        .get_mut()
        .set_read_timeout(Some(Duration::from_millis(20)))
        .unwrap();
    st.connections.insert("conn-1".into(), target_writer);
    let shared = Arc::new(Mutex::new(st));
    let target = "89000000-0000-4000-8000-000000000102";
    let scope = "012_scope_held159_list_aaaaaaaa";
    let conn = "held159-list-reader";
    let (writer, _peer) = control_socket("held159-list-reader");
    shared
        .lock()
        .unwrap()
        .connections
        .insert(conn.into(), writer);
    assert!(matches!(
        handle_wrapper_message(
            conn,
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach,
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    assert!(
        matches!(handle_wrapper_message(conn, WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: scope.into(),
            capabilities: vec![ClientCapability::QueueListV1],
        }, &shared), Some(DaemonToWrapper::ClientWelcome { capabilities, .. })
            if capabilities == vec![ClientCapability::QueueListV1])
    );
    let now = unix_now_secs();
    // Fixtures gravées par la transaction de remise existante, sans sélection SQL copiée.
    {
        let mut st = shared.lock().unwrap();
        for (id, held, sender, body) in [
            (
                "pure-held159",
                true,
                "89000000-0000-4000-8000-000000000200",
                "retenu figé",
            ),
            (
                "pure-released159",
                false,
                "89000000-0000-4000-8000-000000000200",
                "libéré figé",
            ),
        ] {
            let key = IdempotencyKey::new(scope, OperationKind::Send, id).unwrap();
            let mut message = idempotent_message(body);
            message.id = id.into();
            message.from = sender.into();
            let bytes = serde_json::to_vec(&message).unwrap();
            st.idempotency
                .reserve(
                    &key,
                    &bytes,
                    now,
                    CLIENT_IDEMPOTENCY_HORIZON_SECS,
                    now,
                    CLIENT_ISSUED_AT_TOLERANCE_SECS,
                )
                .unwrap();
            let delivery = crate::idempotency::SendDelivery {
                delivery_id: id.into(),
                recipient_instance_id: "instance-1".into(),
                delivery_generation: 77,
                expires_at: now + CLIENT_IDEMPOTENCY_HORIZON_SECS,
                message_bytes: bytes,
            };
            st.idempotency
                .begin_send_delivery_with_guidance(
                    &key,
                    &delivery,
                    None,
                    None,
                    None,
                    None,
                    held.then_some(target),
                )
                .unwrap();
        }
    }
    let before = std::fs::read(&config.db_path).unwrap();
    let wal_path = std::path::PathBuf::from(format!("{}-wal", config.db_path.display()));
    let before_wal = std::fs::read(&wal_path).ok();
    let count = shared.lock().unwrap().idempotency.record_count().unwrap();
    let response = handle_wrapper_message(
        conn,
        WrapperToDaemon::QueueList {
            target: target.into(),
        },
        &shared,
    )
    .unwrap();
    let DaemonToWrapper::QueueListResult {
        entries,
        queue_bound,
    } = response
    else {
        panic!("résultat QueueList attendu : {response:?}");
    };
    assert_eq!(std::fs::read(&config.db_path).unwrap(), before);
    assert_eq!(
        std::fs::read(&wal_path).ok(),
        before_wal,
        "aucune écriture durable par la lecture"
    );
    assert_eq!(
        shared.lock().unwrap().idempotency.record_count().unwrap(),
        count
    );
    assert_eq!(entries.len(), 2, "lecture des deux phases sans les changer");
    assert_eq!(entries[0]["state"], "held");
    assert_eq!(entries[1]["state"], "released");
    assert_eq!(queue_bound, Some(7));
    let mut line = String::new();
    assert!(
        matches!(target_reader.read_line(&mut line), Err(error)
            if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut)),
        "QueueList ne doit pas écrire au porteur"
    );
    drop(shared);
    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn held159_list_fil_t3_et_type_ferme_released_seulement() {
    let label = format!("h159-list-{}", &Uuid::new_v4().simple().to_string()[..12]);
    let (mut st, config) = state_with_registered_agent(&label);
    st.registry = crate::registry::AgentRegistry::from_json(
            r#"{"agents":{"claude":{"command":"/bin/sh","protocol":"acp","queue_capacity":7,"held_queue":true}}}"#,
            "fixture-held159-list.json",
        ).unwrap();
    let (target_writer, mut target_reader) = control_socket(&label);
    target_reader
        .get_mut()
        .set_read_timeout(Some(Duration::from_millis(20)))
        .unwrap();
    st.connections.insert("conn-1".into(), target_writer);
    let shared = Arc::new(Mutex::new(st));
    let target = "89000000-0000-4000-8000-000000000102";
    let scope = "012_scope_held159_list_aaaaaaaa";
    let conn = "held159-list-reader";
    let (writer, _peer) = control_socket("held159-list-reader");
    shared
        .lock()
        .unwrap()
        .connections
        .insert(conn.into(), writer);
    assert!(matches!(
        handle_wrapper_message(
            conn,
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach,
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    assert!(
        matches!(handle_wrapper_message(conn, WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: scope.into(),
            capabilities: vec![ClientCapability::QueueListV1],
        }, &shared), Some(DaemonToWrapper::ClientWelcome { capabilities, .. })
            if capabilities == vec![ClientCapability::QueueListV1])
    );
    let now = unix_now_secs();
    // Fixtures gravées par la transaction de remise existante, sans sélection SQL copiée.
    {
        let mut st = shared.lock().unwrap();
        for (id, held, sender, body) in [
            (
                "held-closed159",
                true,
                "89000000-0000-4000-8000-000000000200",
                "ancien retenu",
            ),
            (
                "released-closed159",
                false,
                "89000000-0000-4000-8000-000000000200",
                "libéré visible",
            ),
        ] {
            let key = IdempotencyKey::new(scope, OperationKind::Send, id).unwrap();
            let mut message = idempotent_message(body);
            message.id = id.into();
            message.from = sender.into();
            let bytes = serde_json::to_vec(&message).unwrap();
            st.idempotency
                .reserve(
                    &key,
                    &bytes,
                    now,
                    CLIENT_IDEMPOTENCY_HORIZON_SECS,
                    now,
                    CLIENT_ISSUED_AT_TOLERANCE_SECS,
                )
                .unwrap();
            let delivery = crate::idempotency::SendDelivery {
                delivery_id: id.into(),
                recipient_instance_id: "instance-1".into(),
                delivery_generation: 77,
                expires_at: now + CLIENT_IDEMPOTENCY_HORIZON_SECS,
                message_bytes: bytes,
            };
            st.idempotency
                .begin_send_delivery_with_guidance(
                    &key,
                    &delivery,
                    None,
                    None,
                    None,
                    None,
                    held.then_some(target),
                )
                .unwrap();
        }
    }
    for (enabled, transport) in [(false, "acp"), (true, "t3code")] {
        {
            let mut st = shared.lock().unwrap();
            st.registry = crate::registry::AgentRegistry::from_json(
                &serde_json::json!({"agents":{"claude":{"command":"/bin/sh","protocol":"acp",
                        "queue_capacity":7,"held_queue":enabled}}})
                .to_string(),
                "fixture-held159-list.json",
            )
            .unwrap();
            st.presences.get_mut("instance-1").unwrap().transport = transport.into();
        }
        let response = handle_wrapper_message(
            conn,
            WrapperToDaemon::QueueList {
                target: target.into(),
            },
            &shared,
        )
        .unwrap();
        let DaemonToWrapper::QueueListResult {
            entries,
            queue_bound,
        } = response
        else {
            panic!("résultat QueueList attendu : {response:?}");
        };
        assert_eq!(
            entries.len(),
            1,
            "type fermé ou fil T3 : aucune entrée éditable"
        );
        assert_eq!(entries[0]["message_id"], "released-closed159");
        assert_eq!(entries[0]["state"], "released");
        assert_eq!(queue_bound, None);
    }
    let mut line = String::new();
    assert!(
        matches!(target_reader.read_line(&mut line), Err(error)
            if matches!(error.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut)),
        "QueueList ne doit pas écrire au porteur"
    );
    drop(shared);
    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn guidance156_queue_list_ordre_rowid() {
    let (mut st, config) = state_with_registered_agent("guidance156_queue_list_ordre_rowid");
    let (writer, mut reader) = control_socket("guidance156_queue_list_ordre_rowid");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_queue_list_ordre_rowid",
        "89000000-0000-4000-8000-000000000200",
    );
    for index in 0..3 {
        let mut message = idempotent_message(&format!("file{index:03} {}", "é".repeat(130)));
        message.references = vec!["reference156".into()];
        let response = handle_wrapper_message(
            "guidance-client",
            WrapperToDaemon::SendIdempotent {
                message,
                message_id: format!("queue156-{index:03}"),
                issued_at: unix_now_secs(),
            },
            &shared,
        )
        .unwrap();
        assert!(matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ));
        read_control(&mut reader);
    }
    let (writer, _peer) = control_socket("queue-attach156");
    shared
        .lock()
        .unwrap()
        .connections
        .insert("queue-attach156".into(), writer);
    assert!(matches!(
        handle_wrapper_message(
            "queue-attach156",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    assert!(
        matches!(handle_wrapper_message("queue-attach156",WrapperToDaemon::ClientHello {contract_version:CLIENT_CONTRACT_VERSION,issuer_scope:"012_scope_queueattach156".into(),capabilities:vec![ClientCapability::QueueListV1]},&shared),Some(DaemonToWrapper::ClientWelcome {capabilities,..}) if capabilities==vec![ClientCapability::QueueListV1])
    );
    assert_eq!(
        shared.lock().unwrap().connection_roles["queue-attach156"],
        ConnectionRole::Attach
    );
    let response = handle_wrapper_message(
        "queue-attach156",
        WrapperToDaemon::QueueList {
            target: "89000000-0000-4000-8000-000000000102".into(),
        },
        &shared,
    )
    .unwrap();
    let DaemonToWrapper::QueueListResult {
        entries,
        queue_bound: _,
    } = response
    else {
        panic!("résultat file attendu : {response:?}")
    };
    assert_eq!(entries.len(), 3);
    for (index, entry) in entries.iter().enumerate() {
        assert_eq!(entry["message_id"], format!("queue156-{index:03}"));
        assert_eq!(entry["position"], index + 1);
        assert_eq!(entry["preview"].as_str().unwrap().chars().count(), 120);
        assert_eq!(entry["state"], "released");
        assert!(entry.get("references").is_none());
        assert!(entry.get("body").is_none());
    }
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_queue_list_apres_reaffectation_meme_ordre() {
    let (mut st, config) =
        state_with_registered_agent("guidance156_queue_list_apres_reaffectation_meme_ordre");
    let (writer, mut reader) =
        control_socket("guidance156_queue_list_apres_reaffectation_meme_ordre");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_queue_list_apres_reaffectation_meme_ordre",
        "89000000-0000-4000-8000-000000000200",
    );
    for index in 0..3 {
        let mut message = idempotent_message(&format!("file{index:03} {}", "é".repeat(130)));
        message.references = vec!["reference156".into()];
        let response = handle_wrapper_message(
            "guidance-client",
            WrapperToDaemon::SendIdempotent {
                message,
                message_id: format!("queue156-{index:03}"),
                issued_at: unix_now_secs(),
            },
            &shared,
        )
        .unwrap();
        assert!(matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ));
        read_control(&mut reader);
    }
    let (writer, _peer) = control_socket("queue-attach156");
    shared
        .lock()
        .unwrap()
        .connections
        .insert("queue-attach156".into(), writer);
    assert!(matches!(
        handle_wrapper_message(
            "queue-attach156",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    assert!(
        matches!(handle_wrapper_message("queue-attach156",WrapperToDaemon::ClientHello {contract_version:CLIENT_CONTRACT_VERSION,issuer_scope:"012_scope_queueattach156".into(),capabilities:vec![ClientCapability::QueueListV1]},&shared),Some(DaemonToWrapper::ClientWelcome {capabilities,..}) if capabilities==vec![ClientCapability::QueueListV1])
    );
    assert_eq!(
        shared.lock().unwrap().connection_roles["queue-attach156"],
        ConnectionRole::Attach
    );
    {
        let mut st = shared.lock().unwrap();
        let delivery = st
            .idempotency
            .queued_deliveries_for_recipient("instance-1", unix_now_secs(), 200, false)
            .unwrap()[1]
            .0
            .clone();
        assert_eq!(
            st.idempotency
                .reassign_dispatching_deliveries(
                    "instance-1",
                    "queue-reassigned156",
                    unix_now_secs()
                )
                .unwrap(),
            3
        );
        st.router.unregister_by_conn("conn-1");
        st.mark_unreachable("conn-1");
        st.connections.remove("conn-1");
        let host = st.host.clone();
        assert!(matches!(
            handle_register_with_channel(
                "queue-target-new156",
                2,
                "claude".into(),
                "89000000-0000-4000-8000-000000000102".into(),
                Some(host),
                Some("acp".into()),
                ChannelReport::Known("unix".into()),
                Some(PresenceMode::Acp),
                None,
                Some("test".into()),
                Some("queue-reassigned156".into()),
                None,
                false,
                Some(true),
                &mut st,
                false
            ),
            DaemonToWrapper::Registered { .. }
        ));
        let (writer, _peer) = control_socket("queue-newtarget156");
        st.connections.insert("queue-target-new156".into(), writer);
        assert!(
            st.idempotency
                .queued_deliveries_for_recipient("queue-reassigned156", unix_now_secs(), 200, false)
                .unwrap()
                .iter()
                .any(|(row, _, _)| row.delivery_id == delivery.delivery_id)
        );
    }
    let response = handle_wrapper_message(
        "queue-attach156",
        WrapperToDaemon::QueueList {
            target: "89000000-0000-4000-8000-000000000102".into(),
        },
        &shared,
    )
    .unwrap();
    let DaemonToWrapper::QueueListResult {
        entries,
        queue_bound: _,
    } = response
    else {
        panic!("résultat file attendu : {response:?}")
    };
    assert_eq!(entries.len(), 3);
    for (index, entry) in entries.iter().enumerate() {
        assert_eq!(entry["message_id"], format!("queue156-{index:03}"));
        assert_eq!(entry["position"], index + 1);
        assert_eq!(entry["preview"].as_str().unwrap().chars().count(), 120);
        assert_eq!(entry["state"], "released");
        assert!(entry.get("references").is_none());
        assert!(entry.get("body").is_none());
    }
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_queue_list_filtre_par_droits() {
    let (mut st, config) = state_with_registered_agent("guidance156_queue_list_filtre_par_droits");
    let (writer, mut reader) = control_socket("guidance156_queue_list_filtre_par_droits");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_queue_list_filtre_par_droits",
        "89000000-0000-4000-8000-000000000200",
    );
    for index in 0..3 {
        let mut message = idempotent_message(&format!("file{index:03} {}", "é".repeat(130)));
        message.references = vec!["reference156".into()];
        let response = handle_wrapper_message(
            "guidance-client",
            WrapperToDaemon::SendIdempotent {
                message,
                message_id: format!("queue156-{index:03}"),
                issued_at: unix_now_secs(),
            },
            &shared,
        )
        .unwrap();
        assert!(matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ));
        read_control(&mut reader);
    }
    let (writer, _peer) = control_socket("queue-attach156");
    shared
        .lock()
        .unwrap()
        .connections
        .insert("queue-attach156".into(), writer);
    assert!(matches!(
        handle_wrapper_message(
            "queue-attach156",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    assert!(
        matches!(handle_wrapper_message("queue-attach156",WrapperToDaemon::ClientHello {contract_version:CLIENT_CONTRACT_VERSION,issuer_scope:"012_scope_queueattach156".into(),capabilities:vec![ClientCapability::QueueListV1]},&shared),Some(DaemonToWrapper::ClientWelcome {capabilities,..}) if capabilities==vec![ClientCapability::QueueListV1])
    );
    assert_eq!(
        shared.lock().unwrap().connection_roles["queue-attach156"],
        ConnectionRole::Attach
    );
    let response = handle_wrapper_message(
        "queue-attach156",
        WrapperToDaemon::QueueList {
            target: "89000000-0000-4000-8000-000000000102".into(),
        },
        &shared,
    )
    .unwrap();
    let DaemonToWrapper::QueueListResult {
        entries,
        queue_bound: _,
    } = response
    else {
        panic!("résultat file attendu : {response:?}")
    };
    assert_eq!(entries.len(), 3);
    for (index, entry) in entries.iter().enumerate() {
        assert_eq!(entry["message_id"], format!("queue156-{index:03}"));
        assert_eq!(entry["position"], index + 1);
        assert_eq!(entry["preview"].as_str().unwrap().chars().count(), 120);
        assert_eq!(entry["state"], "released");
        assert!(entry.get("references").is_none());
        assert!(entry.get("body").is_none());
    }
    // Une vraie identité auxiliaire destinataire voit toutes les lignes.
    for (conn, sender, expected) in [
        ("queue-target156", "89000000-0000-4000-8000-000000000102", 3),
        ("queue-own156", "89000000-0000-4000-8000-000000000200", 3),
        ("queue-peer156", "89000000-0000-4000-8000-000000000300", 0),
    ] {
        let (instance, credential) = {
            let mut st = shared.lock().unwrap();
            let owner = st
                .router
                .get_agent(sender)
                .map(|r| r.connection_id.clone())
                .unwrap_or_else(|| "queue-peer-owner156".into());
            if !st.identity_credentials.contains_key(&owner) {
                let host = st.host.clone();
                assert!(matches!(
                    handle_register_with_channel(
                        &owner,
                        2,
                        "fixture".into(),
                        sender.into(),
                        Some(host),
                        Some("acp".into()),
                        ChannelReport::Known("unix".into()),
                        Some(PresenceMode::Acp),
                        None,
                        Some("test".into()),
                        Some("queue-peer-instance156".into()),
                        None,
                        false,
                        Some(false),
                        &mut st,
                        false
                    ),
                    DaemonToWrapper::Registered { .. }
                ));
                let _ = super::spec094_live_test_connection(&mut st, &owner);
            }
            (
                st.conn_instances[&owner].clone(),
                st.identity_credentials[&owner].clone(),
            )
        };
        handle_wrapper_message(
            conn,
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client,
            },
            &shared,
        )
        .unwrap();
        assert!(matches!(
            handle_wrapper_message(
                conn,
                WrapperToDaemon::RegisterAuxiliary {
                    agent_id: sender.into(),
                    instance_id: instance,
                    credential
                },
                &shared
            ),
            Some(DaemonToWrapper::Registered { .. })
        ));
        handle_wrapper_message(
            conn,
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: format!("012_scope_{conn}"),
                capabilities: vec![ClientCapability::QueueListV1],
            },
            &shared,
        )
        .unwrap();
        assert!(
            matches!(handle_wrapper_message(conn,WrapperToDaemon::QueueList {target:"89000000-0000-4000-8000-000000000102".into()},&shared),Some(DaemonToWrapper::QueueListResult {entries,..}) if entries.len()==expected)
        );
    }
    {
        let mut st = shared.lock().unwrap();
        guidance156_link_parent(&mut st, "queue-peer156", "conn-1");
    }
    assert!(
        matches!(handle_wrapper_message("queue-peer156",WrapperToDaemon::QueueList {target:"89000000-0000-4000-8000-000000000102".into()},&shared),Some(DaemonToWrapper::QueueListResult {entries,..}) if entries.len()==3)
    );
    // L'identité d'agent ne peut pas devenir Attach par un second handshake.
    assert!(!matches!(
        handle_wrapper_message(
            "queue-own156",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    assert_eq!(
        shared.lock().unwrap().connection_roles["queue-own156"],
        ConnectionRole::Client
    );
    // UI attestée : identité primaire réservée, auxiliaire réelle, capacité lecture seule.
    let (ui_instance, ui_credential, _ui_peer) = {
        let mut st = shared.lock().unwrap();
        let host = st.host.clone();
        assert!(matches!(
            handle_register_with_channel(
                "queue-ui-primary156",
                2,
                "ui".into(),
                UI_HUMAN_AGENT_ID.into(),
                Some(host),
                None,
                ChannelReport::Omitted,
                Some(PresenceMode::Cli),
                None,
                Some("test".into()),
                Some("bridget-ui-queue156".into()),
                None,
                false,
                Some(false),
                &mut st,
                false
            ),
            DaemonToWrapper::Registered { .. }
        ));
        let peer = super::spec094_live_test_connection(&mut st, "queue-ui-primary156");
        (
            st.conn_instances["queue-ui-primary156"].clone(),
            st.identity_credentials["queue-ui-primary156"].clone(),
            peer,
        )
    };
    assert!(matches!(
        handle_wrapper_message(
            "queue-ui156",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        })
    ));
    let response = handle_wrapper_message(
        "queue-ui156",
        WrapperToDaemon::RegisterAuxiliary {
            agent_id: UI_HUMAN_AGENT_ID.into(),
            instance_id: ui_instance,
            credential: ui_credential,
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response, DaemonToWrapper::Registered { .. }),
        "attestation UI : {response:?}"
    );
    let response = handle_wrapper_message(
        "queue-ui156",
        WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: "012_scope_queue_ui156_attested".into(),
            capabilities: vec![ClientCapability::QueueListV1],
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response,DaemonToWrapper::ClientWelcome {ref capabilities,..} if capabilities==&vec![ClientCapability::QueueListV1]),
        "capacité UI : {response:?}"
    );
    let response = handle_wrapper_message(
        "queue-ui156",
        WrapperToDaemon::QueueList {
            target: "89000000-0000-4000-8000-000000000102".into(),
        },
        &shared,
    )
    .unwrap();
    assert!(
        matches!(response,DaemonToWrapper::QueueListResult {ref entries,..} if entries.len()==3),
        "lecture UI complète : {response:?}"
    );
    assert!(!matches!(
        handle_wrapper_message(
            "queue-ui156",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    assert_eq!(
        shared.lock().unwrap().connection_roles["queue-ui156"],
        ConnectionRole::Client
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_queue_list_borne_200() {
    let (mut st, config) = state_with_registered_agent("guidance156_queue_list_borne_200");
    let (writer, mut reader) = control_socket("guidance156_queue_list_borne_200");
    st.connections.insert("conn-1".into(), writer);
    st.circuit_breaker = CircuitBreaker::new(config.circuit_breaker_window, 512);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_queue_list_borne_200",
        "89000000-0000-4000-8000-000000000200",
    );
    for index in 0..201 {
        let mut message = idempotent_message(&format!("file{index:03} {}", "é".repeat(130)));
        message.references = vec!["reference156".into()];
        let response = handle_wrapper_message(
            "guidance-client",
            WrapperToDaemon::SendIdempotent {
                message,
                message_id: format!("queue156-{index:03}"),
                issued_at: unix_now_secs(),
            },
            &shared,
        )
        .unwrap();
        assert!(matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ));
        read_control(&mut reader);
    }
    let (writer, _peer) = control_socket("queue-attach156");
    shared
        .lock()
        .unwrap()
        .connections
        .insert("queue-attach156".into(), writer);
    assert!(matches!(
        handle_wrapper_message(
            "queue-attach156",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    assert!(
        matches!(handle_wrapper_message("queue-attach156",WrapperToDaemon::ClientHello {contract_version:CLIENT_CONTRACT_VERSION,issuer_scope:"012_scope_queueattach156".into(),capabilities:vec![ClientCapability::QueueListV1]},&shared),Some(DaemonToWrapper::ClientWelcome {capabilities,..}) if capabilities==vec![ClientCapability::QueueListV1])
    );
    assert_eq!(
        shared.lock().unwrap().connection_roles["queue-attach156"],
        ConnectionRole::Attach
    );
    let response = handle_wrapper_message(
        "queue-attach156",
        WrapperToDaemon::QueueList {
            target: "89000000-0000-4000-8000-000000000102".into(),
        },
        &shared,
    )
    .unwrap();
    let DaemonToWrapper::QueueListResult {
        entries,
        queue_bound: _,
    } = response
    else {
        panic!("résultat file attendu : {response:?}")
    };
    assert_eq!(entries.len(), 200);
    assert_eq!(entries[199]["position"], 200);
    for (index, entry) in entries.iter().enumerate() {
        assert_eq!(entry["message_id"], format!("queue156-{index:03}"));
        assert_eq!(entry["position"], index + 1);
        assert_eq!(entry["preview"].as_str().unwrap().chars().count(), 120);
        assert_eq!(entry["state"], "released");
        assert!(entry.get("references").is_none());
        assert!(entry.get("body").is_none());
    }
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_queue_list_ne_modifie_rien() {
    let (mut st, config) = state_with_registered_agent("guidance156_queue_list_ne_modifie_rien");
    let (writer, mut reader) = control_socket("guidance156_queue_list_ne_modifie_rien");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_queue_list_ne_modifie_rien",
        "89000000-0000-4000-8000-000000000200",
    );
    for index in 0..3 {
        let mut message = idempotent_message(&format!("file{index:03} {}", "é".repeat(130)));
        message.references = vec!["reference156".into()];
        let response = handle_wrapper_message(
            "guidance-client",
            WrapperToDaemon::SendIdempotent {
                message,
                message_id: format!("queue156-{index:03}"),
                issued_at: unix_now_secs(),
            },
            &shared,
        )
        .unwrap();
        assert!(matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ));
        read_control(&mut reader);
    }
    let (writer, _peer) = control_socket("queue-attach156");
    shared
        .lock()
        .unwrap()
        .connections
        .insert("queue-attach156".into(), writer);
    assert!(matches!(
        handle_wrapper_message(
            "queue-attach156",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    assert!(
        matches!(handle_wrapper_message("queue-attach156",WrapperToDaemon::ClientHello {contract_version:CLIENT_CONTRACT_VERSION,issuer_scope:"012_scope_queueattach156".into(),capabilities:vec![ClientCapability::QueueListV1]},&shared),Some(DaemonToWrapper::ClientWelcome {capabilities,..}) if capabilities==vec![ClientCapability::QueueListV1])
    );
    assert_eq!(
        shared.lock().unwrap().connection_roles["queue-attach156"],
        ConnectionRole::Attach
    );
    let before = std::fs::read(&config.db_path).unwrap();
    let count = shared.lock().unwrap().idempotency.record_count().unwrap();
    let response = handle_wrapper_message(
        "queue-attach156",
        WrapperToDaemon::QueueList {
            target: "89000000-0000-4000-8000-000000000102".into(),
        },
        &shared,
    )
    .unwrap();
    let DaemonToWrapper::QueueListResult {
        entries,
        queue_bound: _,
    } = response
    else {
        panic!("résultat file attendu : {response:?}")
    };
    assert_eq!(entries.len(), 3);
    for (index, entry) in entries.iter().enumerate() {
        assert_eq!(entry["message_id"], format!("queue156-{index:03}"));
        assert_eq!(entry["position"], index + 1);
        assert_eq!(entry["preview"].as_str().unwrap().chars().count(), 120);
        assert_eq!(entry["state"], "released");
        assert!(entry.get("references").is_none());
        assert!(entry.get("body").is_none());
    }
    assert_eq!(
        shared.lock().unwrap().idempotency.record_count().unwrap(),
        count
    );
    assert_eq!(std::fs::read(&config.db_path).unwrap(), before);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_queue_negociation_requise() {
    let (mut st, config) = state_with_registered_agent("guidance156_queue_negociation_requise");
    let (writer, mut reader) = control_socket("guidance156_queue_negociation_requise");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_queue_negociation_requise",
        "89000000-0000-4000-8000-000000000200",
    );
    for index in 0..3 {
        let mut message = idempotent_message(&format!("file{index:03} {}", "é".repeat(130)));
        message.references = vec!["reference156".into()];
        let response = handle_wrapper_message(
            "guidance-client",
            WrapperToDaemon::SendIdempotent {
                message,
                message_id: format!("queue156-{index:03}"),
                issued_at: unix_now_secs(),
            },
            &shared,
        )
        .unwrap();
        assert!(matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ));
        read_control(&mut reader);
    }
    handle_wrapper_message(
        "queue-bare156",
        WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Client,
        },
        &shared,
    )
    .unwrap();
    assert!(matches!(
        handle_wrapper_message(
            "queue-bare156",
            WrapperToDaemon::QueueList {
                target: "89000000-0000-4000-8000-000000000102".into()
            },
            &shared
        ),
        Some(DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::NegotiationRequired
        })
    ));
    handle_wrapper_message(
        "queue-bare156",
        WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: "012_scope_queuenocap156".into(),
            capabilities: vec![ClientCapability::Lookup],
        },
        &shared,
    )
    .unwrap();
    assert!(matches!(
        handle_wrapper_message(
            "queue-bare156",
            WrapperToDaemon::QueueList {
                target: "89000000-0000-4000-8000-000000000102".into()
            },
            &shared
        ),
        Some(DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::CapabilityNotNegotiated
        })
    ));
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn guidance156_queue_bound_t3_64_sinon_null() {
    let (mut st, config) = state_with_registered_agent("guidance156_queue_bound_t3_64_sinon_null");
    let (writer, mut reader) = control_socket("guidance156_queue_bound_t3_64_sinon_null");
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "guidance-client",
        "012_scope_guidance156_queue_bound_t3_64_sinon_null",
        "89000000-0000-4000-8000-000000000200",
    );
    for index in 0..3 {
        let mut message = idempotent_message(&format!("file{index:03} {}", "é".repeat(130)));
        message.references = vec!["reference156".into()];
        let response = handle_wrapper_message(
            "guidance-client",
            WrapperToDaemon::SendIdempotent {
                message,
                message_id: format!("queue156-{index:03}"),
                issued_at: unix_now_secs(),
            },
            &shared,
        )
        .unwrap();
        assert!(matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ));
        read_control(&mut reader);
    }
    let (writer, _peer) = control_socket("queue-attach156");
    shared
        .lock()
        .unwrap()
        .connections
        .insert("queue-attach156".into(), writer);
    assert!(matches!(
        handle_wrapper_message(
            "queue-attach156",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    assert!(
        matches!(handle_wrapper_message("queue-attach156",WrapperToDaemon::ClientHello {contract_version:CLIENT_CONTRACT_VERSION,issuer_scope:"012_scope_queueattach156".into(),capabilities:vec![ClientCapability::QueueListV1]},&shared),Some(DaemonToWrapper::ClientWelcome {capabilities,..}) if capabilities==vec![ClientCapability::QueueListV1])
    );
    assert_eq!(
        shared.lock().unwrap().connection_roles["queue-attach156"],
        ConnectionRole::Attach
    );
    let response = handle_wrapper_message(
        "queue-attach156",
        WrapperToDaemon::QueueList {
            target: "89000000-0000-4000-8000-000000000102".into(),
        },
        &shared,
    )
    .unwrap();
    let DaemonToWrapper::QueueListResult {
        entries,
        queue_bound,
    } = response
    else {
        panic!("résultat file attendu : {response:?}")
    };
    assert_eq!(entries.len(), 3);
    for (index, entry) in entries.iter().enumerate() {
        assert_eq!(entry["message_id"], format!("queue156-{index:03}"));
        assert_eq!(entry["position"], index + 1);
        assert_eq!(entry["preview"].as_str().unwrap().chars().count(), 120);
        assert_eq!(entry["state"], "released");
        assert!(entry.get("references").is_none());
        assert!(entry.get("body").is_none());
    }
    assert_eq!(queue_bound, None);
    shared
        .lock()
        .unwrap()
        .presences
        .get_mut("instance-1")
        .unwrap()
        .transport = "t3code".into();
    assert!(matches!(
        handle_wrapper_message(
            "queue-attach156",
            WrapperToDaemon::QueueList {
                target: "89000000-0000-4000-8000-000000000102".into()
            },
            &shared
        ),
        Some(DaemonToWrapper::QueueListResult {
            queue_bound: None,
            ..
        })
    ));
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

// Réemploi composition : state_with_registered_agent + négociation auxiliaire149
// + control_socket. Utilisé par les huit oracles de ce lot, sans verdict miroir.
fn held159_fixture(
    label: &str,
    held_queue: Option<bool>,
    busy: bool,
    attest: bool,
    capacity: usize,
) -> (Arc<Mutex<DaemonState>>, DaemonConfig, BufReader<UnixStream>) {
    let (mut st, config) = state_with_registered_agent(label);
    let mut definition = serde_json::json!({"command":"/bin/sh","protocol":"acp",
            "forbidden_env":[],"pass_env":[],"queue_capacity":capacity,
            "capabilities":{"execution_paths":["acp"]}});
    if let Some(value) = held_queue {
        definition["held_queue"] = value.into();
    }
    st.registry = crate::registry::AgentRegistry::from_json(
        &serde_json::json!({"agents":{"claude":definition}}).to_string(),
        st.registry.source().to_path_buf(),
    )
    .unwrap();
    let presence = st.presences.get_mut("instance-1").unwrap();
    presence.state = if busy { "busy" } else { "connected" }.into();
    presence.turn_attest = attest;
    presence.busy_period_start = busy.then(Instant::now);
    presence.last_attested_at = attest.then(Instant::now);
    let (writer, reader) = control_socket(label);
    st.connections.insert("conn-1".into(), writer);
    let shared = Arc::new(Mutex::new(st));
    negotiate_idempotent_client(
        &shared,
        "held159-client",
        "012_scope_held159_oracle",
        "89000000-0000-4000-8000-000000000200",
    );
    (shared, config, reader)
}

fn held159_submit(
    shared: &Arc<Mutex<DaemonState>>,
    message: BridgetMessage,
    key: &str,
    issued_at: i64,
) -> DaemonToWrapper {
    handle_wrapper_message(
        "held159-client",
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: key.into(),
            issued_at,
        },
        shared,
    )
    .unwrap()
}

// Mesure le vrai canon/phase/enveloppe persisté et la vraie trame, sans calculer
// should_hold. Les métadonnées de remise de la trame doivent égaler celles SQL.
fn held159_assert_delivery(
    shared: &Arc<Mutex<DaemonState>>,
    config: &DaemonConfig,
    reader: &mut BufReader<UnixStream>,
    key: &str,
    expected_phase: &str,
    response: &DaemonToWrapper,
) -> (Vec<u8>, Vec<u8>, String) {
    let db = rusqlite::Connection::open(&config.db_path).unwrap();
    let (canon,bytes,phase,id,generation):(Vec<u8>,Vec<u8>,String,String,u64) = db.query_row(
            "SELECT r.canonical_bytes,d.message_bytes,d.phase,d.delivery_id,d.delivery_generation FROM idempotency_records r JOIN send_deliveries d USING(issuer_scope,operation_kind,idempotency_key) WHERE r.issuer_scope='012_scope_held159_oracle' AND r.idempotency_key=?1",
            [key], |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).unwrap();
    assert_eq!(phase, expected_phase, "{response:?}");
    assert!(
        matches!(response,DaemonToWrapper::IdempotencyResult {issue:IdempotencyIssue::OutcomeUnknown {delivery_id:Some(actual),..},..} if actual==&id),
        "admission conservée : {response:?}"
    );
    if expected_phase == "held" {
        reader
            .get_ref()
            .set_read_timeout(Some(Duration::from_millis(30)))
            .unwrap();
        let mut line = String::new();
        let error = reader
            .read_line(&mut line)
            .expect_err("aucune poussée retenue");
        assert!(
            matches!(
                error.kind(),
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
            ),
            "{error}"
        );
        assert!(line.is_empty(), "{line}");
    } else {
        match read_control(reader) {
            DaemonToWrapper::DeliverIdempotent {
                message,
                delivery_id,
                delivery_generation,
                ..
            } => {
                assert_eq!(delivery_id, id);
                assert_eq!(delivery_generation, generation);
                // SQL garde aussi les avertissements internes et un ordre JSON
                // distinct du struct wire. Comparer tous les champs métier typés.
                let envelope: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                if let DaemonToWrapper::IdempotencyResult {
                    project_warnings, ..
                } = response
                {
                    let persisted = envelope
                        .get("_project_warnings")
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!([]));
                    assert_eq!(serde_json::to_value(project_warnings).unwrap(), persisted);
                }
                let mut expected: BridgetMessage = serde_json::from_slice(&bytes).unwrap();
                let st = shared.lock().unwrap();
                // Présentation issue du vrai profil SQL, sans modifier le canon.
                expected.from_display_name = st.store.agent_display_name(&expected.from).unwrap();
                assert!(expected.thread_notice.is_none());
                assert!(expected.thread_display_title.is_none());
                if expected.deadline_at.is_none() {
                    let issued_at: i64 = db.query_row(
                            "SELECT issued_at FROM idempotency_records WHERE issuer_scope='012_scope_held159_oracle' AND operation_kind='send' AND idempotency_key=?1",
                            [key], |row| row.get(0)).unwrap();
                    let timeout =
                        if expected.intent == Some(bridget_core::MessageIntent::SteerCurrent) {
                            HUMAN_STEER_TIMEOUT_SECS
                        } else {
                            st.registry.get("claude").unwrap().notify_timeout_secs
                        };
                    let deadline = message.deadline_at.expect("échéance live de la poussée");
                    assert!(
                        deadline >= (issued_at as u64).saturating_add(timeout)
                            && deadline <= (unix_now_secs() as u64).saturating_add(timeout),
                        "deadline={deadline}, issued_at={issued_at}, timeout={timeout}"
                    );
                    expected.deadline_at = Some(deadline);
                }
                assert_eq!(
                    message, expected,
                    "tous les champs de remise, hors projections prouvées"
                );
            }
            other => panic!("vraie remise attendue : {other:?}"),
        }
    }
    (canon, bytes, phase)
}

#[test]
fn held159_hold_ferme_identique() {
    let mut message = idempotent_message("octets fermés159");
    message.deadline_at = Some(unix_now_secs() as u64 + 3600);
    let at = unix_now_secs();
    let mut snapshots = Vec::new();
    for (label, flag) in [("h159-om", None), ("h159-fa", Some(false))] {
        let (shared, config, mut reader) = held159_fixture(label, flag, true, true, 32);
        let response = held159_submit(&shared, message.clone(), "closed159", at);
        snapshots.push(held159_assert_delivery(
            &shared,
            &config,
            &mut reader,
            "closed159",
            "dispatching",
            &response,
        ));
        drop(shared);
        let _ = std::fs::remove_file(config.db_path);
    }
    assert_eq!(
        snapshots[0], snapshots[1],
        "canon, message et phase byte-identiques"
    );
}

#[test]
fn held159_hold_sans_turn_attest() {
    let (shared, config, mut reader) = held159_fixture("h159-na", Some(true), true, false, 32);
    let response = held159_submit(
        &shared,
        idempotent_message("sans attestation159"),
        "noattest159",
        unix_now_secs(),
    );
    held159_assert_delivery(
        &shared,
        &config,
        &mut reader,
        "noattest159",
        "dispatching",
        &response,
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn held159_hold_intention_presente() {
    for (index, intent) in [
        bridget_core::MessageIntent::QueueOnly,
        bridget_core::MessageIntent::TriggerTurn,
        bridget_core::MessageIntent::SteerCurrent,
        bridget_core::MessageIntent::InterruptAndStart,
    ]
    .into_iter()
    .enumerate()
    {
        let (shared, config, mut reader) =
            held159_fixture(&format!("h159-i{index}"), Some(true), true, true, 32);
        guidance156_link_parent(&mut shared.lock().unwrap(), "held159-client", "conn-1");
        let mut message = idempotent_message("intention explicite159");
        message.intent = Some(intent);
        let response = held159_submit(&shared, message, "intent159", unix_now_secs());
        let (_, _, delivery_id) = held159_assert_delivery(
            &shared,
            &config,
            &mut reader,
            "intent159",
            if intent == bridget_core::MessageIntent::QueueOnly {
                "held"
            } else {
                "dispatching"
            },
            &response,
        );
        if intent == bridget_core::MessageIntent::QueueOnly {
            let links: i64 = shared
                .lock()
                .unwrap()
                .store
                .connection()
                .query_row(
                    "SELECT COUNT(*) FROM send_delivery_execution_links WHERE delivery_id=?1",
                    [&delivery_id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(links, 0, "QueueOnly retenu sans exécution liée");
        }
        drop(shared);
        let _ = std::fs::remove_file(config.db_path);
    }
}

#[test]
fn held159_hold_non_busy_sans_retenus() {
    let (shared, config, mut reader) = held159_fixture("h159-ie", Some(true), false, true, 32);
    let response = held159_submit(
        &shared,
        idempotent_message("idle vide159"),
        "idleempty159",
        unix_now_secs(),
    );
    held159_assert_delivery(
        &shared,
        &config,
        &mut reader,
        "idleempty159",
        "dispatching",
        &response,
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn held159_hold_non_busy_avec_retenus() {
    let (shared, config, mut reader) = held159_fixture("h159-ih", Some(true), true, true, 32);
    let now = unix_now_secs();
    let first = held159_submit(
        &shared,
        idempotent_message("premier retenu159"),
        "seedheld159",
        now,
    );
    held159_assert_delivery(&shared, &config, &mut reader, "seedheld159", "held", &first);
    shared
        .lock()
        .unwrap()
        .presences
        .get_mut("instance-1")
        .unwrap()
        .state = "connected".into();
    let response = held159_submit(
        &shared,
        idempotent_message("après retenu159"),
        "idleheld159",
        now,
    );
    held159_assert_delivery(
        &shared,
        &config,
        &mut reader,
        "idleheld159",
        "held",
        &response,
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn held159_hold_busy() {
    let (shared, config, mut reader) = held159_fixture("h159-bu", Some(true), true, true, 32);
    let message = idempotent_message("busy159");
    let now = unix_now_secs();
    let response = held159_submit(&shared, message.clone(), "busy159", now);
    held159_assert_delivery(&shared, &config, &mut reader, "busy159", "held", &response);
    let replay = held159_submit(&shared, message, "busy159", now);
    assert_eq!(
        serde_json::to_value(&replay).unwrap(),
        serde_json::to_value(&response).unwrap(),
        "rejeu retenu conserve issue et UUID sans poussée"
    );
    let db = rusqlite::Connection::open(&config.db_path).unwrap();
    let count: i64 = db
        .query_row(
            "SELECT count(*) FROM send_deliveries WHERE phase='held' AND recipient_agent=?1",
            ["89000000-0000-4000-8000-000000000102"],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
    drop(db);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn held159_hold_file_pleine() {
    let (shared, config, mut reader) = held159_fixture("h159-qf", Some(true), true, true, 1);
    let now = unix_now_secs();
    let first = idempotent_message("premier159");
    let accepted = held159_submit(&shared, first, "queuefirst159", now);
    held159_assert_delivery(
        &shared,
        &config,
        &mut reader,
        "queuefirst159",
        "held",
        &accepted,
    );
    let message = idempotent_message("saturation159");
    let refused = held159_submit(&shared, message.clone(), "queuefull159", now);
    assert!(
        matches!(&refused,DaemonToWrapper::IdempotencyResult {issue:IdempotencyIssue::Rejected {category,..},..} if category=="queue_full"),
        "{refused:?}"
    );
    // Le reçu reste refusé après libération physique : aucun recalcul au rejeu.
    rusqlite::Connection::open(&config.db_path)
        .unwrap()
        .execute(
            "DELETE FROM send_deliveries WHERE idempotency_key='queuefirst159'",
            [],
        )
        .unwrap();
    let replay = held159_submit(&shared, message, "queuefull159", now);
    assert_eq!(
        serde_json::to_value(&replay).unwrap(),
        serde_json::to_value(&refused).unwrap()
    );
    let db = rusqlite::Connection::open(&config.db_path).unwrap();
    let count: i64 = db
        .query_row(
            "SELECT count(*) FROM send_deliveries WHERE idempotency_key='queuefull159'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
    let mut line = String::new();
    assert!(reader.read_line(&mut line).is_err());
    assert!(line.is_empty());
    drop(db);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn held159_hold_origine_human() {
    let (shared, config, mut reader) = held159_fixture("h159-hu", Some(true), true, true, 32);
    let (writer, _peer) = control_socket("h159-ha");
    shared
        .lock()
        .unwrap()
        .connections
        .insert("held159-attach".into(), writer);
    assert!(matches!(
        handle_wrapper_message(
            "held159-attach",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach
            },
            &shared
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    let mut message = idempotent_message("humain réel159");
    message.origin = Some(bridget_core::MessageOrigin::Human);
    assert!(matches!(
        handle_wrapper_message("held159-attach", WrapperToDaemon::Send(message), &shared),
        Some(DaemonToWrapper::Ack { .. })
    ));
    assert!(
        matches!(read_control(&mut reader),DaemonToWrapper::Deliver {message,..} if message.origin==Some(bridget_core::MessageOrigin::Human))
    );
    let held: i64 = shared
        .lock()
        .unwrap()
        .store
        .connection()
        .query_row(
            "SELECT count(*) FROM send_deliveries WHERE phase='held'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(held, 0);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn held159_hold_origine_system() {
    use bridget_transport::protocol::{WakeAction, WakeCadence, WakeRequest, WakeTarget};
    let (shared, config, mut reader) = held159_fixture("h159-sy", Some(true), true, true, 32);
    let at = unix_now_secs();
    let clock = Instant::now();
    let response = handle_wake_request(
        "conn-1",
        WakeRequest {
            version: 1,
            request: WakeAction::Schedule {
                target: WakeTarget::SelfTarget {},
                cadence: WakeCadence::Interval { seconds: 60 },
                label: Some("system159".into()),
            },
        },
        &shared,
        at,
    );
    assert!(
        matches!(response, DaemonToWrapper::WakeResult { .. }),
        "{response:?}"
    );
    let schedules = shared
        .lock()
        .unwrap()
        .store
        .list_wake_schedules(None)
        .unwrap();
    assert_eq!(schedules.len(), 1);
    scheduled_wakes_tick(&shared, schedules[0].next_at, clock);
    let id = match read_control(&mut reader) {
        DaemonToWrapper::DeliverIdempotent {
            message,
            delivery_id,
            ..
        } => {
            assert_eq!(message.origin, Some(bridget_core::MessageOrigin::System));
            delivery_id
        }
        other => panic!("réveil System réel : {other:?}"),
    };
    let db = rusqlite::Connection::open(&config.db_path).unwrap();
    let phase: String = db
        .query_row(
            "SELECT phase FROM send_deliveries WHERE delivery_id=?1",
            [id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(phase, "dispatching");
    drop(db);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn held159_hold_observation() {
    use bridget_transport::protocol::{ObservationKind, ObservationRequest};
    let (shared, config, mut reader) = held159_fixture("h159-ob", Some(true), true, true, 32);
    let clock = Instant::now();
    let at = unix_now_secs();
    {
        let mut st = shared.lock().unwrap();
        st.observations.set_source(
            "source159",
            "source159",
            vec![ObservationKind::PermissionRequired],
            clock,
        );
        let owner = st.presences["instance-1"].name.clone();
        let response = st.observations.request(
            &owner,
            ObservationRequest::Sub {
                event: ObservationKind::PermissionRequired,
                agent: Some("source159".into()),
                file: None,
                once: true,
                ttl_secs: Some(3600),
            },
            clock,
            at as u64,
        );
        assert_eq!(response["status"], "subscribed");
        observe_fact_at(
            &mut st,
            crate::observation::Fact {
                event: ObservationKind::PermissionRequired,
                agent: "source159".into(),
                host: "fixture159".into(),
                file: None,
                other_agent: None,
            },
            at,
            clock,
        );
    }
    assert!(
        matches!(read_control(&mut reader),DaemonToWrapper::Deliver {message,..}
            if message.origin==Some(bridget_core::MessageOrigin::System)&&message.id.starts_with("bridget-observation:"))
    );
    let held: i64 = shared
        .lock()
        .unwrap()
        .store
        .connection()
        .query_row(
            "SELECT count(*) FROM send_deliveries WHERE phase='held'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(held, 0);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn held159_hold_incident_delegue() {
    let (shared, config, _reader) = held159_fixture("h159-di", Some(true), true, true, 32);
    let (writer, mut parent_reader) = control_socket("h159-pa");
    {
        let mut st = shared.lock().unwrap();
        let parent_conn = "owner-89000000-0000-4000-8000-000000000200";
        st.connections.insert(parent_conn.into(), writer);
        let parent = st.conn_instances[parent_conn].clone();
        let mut presence = st.presences[&parent].clone();
        presence.state = "busy".into();
        presence.turn_attest = true;
        presence.agent_type = "claude".into();
        st.presences.insert(parent, presence);
        guidance156_link_parent(&mut st, parent_conn, "conn-1");
    }
    let reference = format!("sha256:{}", "c".repeat(64));
    assert!(
        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::DelegatedRuntimeEvent {
                execution_id: "child159".into(),
                kind: bridget_transport::protocol::DelegatedRuntimeEventKind::Warning,
                code: "unsupported_provider_request".into(),
                reference: reference.clone()
            },
            &shared
        )
        .is_none()
    );
    assert!(
        matches!(read_control(&mut parent_reader),DaemonToWrapper::DelegatedRuntimeEvent {event}
            if event.child_instance_id=="instance-1"&&event.reference==reference)
    );
    let held: i64 = shared
        .lock()
        .unwrap()
        .store
        .connection()
        .query_row(
            "SELECT count(*) FROM send_deliveries WHERE phase='held'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(held, 0);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn held159_hold_fil_t3() {
    let (shared, config, mut reader) = held159_fixture("h159-t3", Some(true), true, true, 32);
    shared
        .lock()
        .unwrap()
        .presences
        .get_mut("instance-1")
        .unwrap()
        .transport = "t3code".into();
    let response = held159_submit(
        &shared,
        idempotent_message("fil T3 existant159"),
        "t3thread159",
        unix_now_secs(),
    );
    held159_assert_delivery(
        &shared,
        &config,
        &mut reader,
        "t3thread159",
        "dispatching",
        &response,
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn held159_hold_alerte_fil() {
    let (shared, config, mut reader) = held159_fixture("h159-th", Some(true), true, true, 32);
    assert!(
        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::ThreadNoticeCapability {
                versions: vec![crate::threads::THREAD_NOTICE_VERSION]
            },
            &shared
        )
        .is_none()
    );
    let owner = "89000000-0000-4000-8000-000000000102";
    let peer = "89000000-0000-4000-8000-000000000200";
    let now = unix_now_secs();
    let mut controls = Vec::new();
    {
        let mut st = shared.lock().unwrap();
        let project = bridget_transport::protocol::CommunicationProject {
            host: "macbook".into(),
            root: config
                .db_path
                .parent()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        };
        st.communication_projects
            .insert("conn-1".into(), (Some(project.clone()), None, false));
        st.communication_projects
            .insert(format!("owner-{peer}"), (Some(project), None, false));
        let request=serde_json::from_value(serde_json::json!({"version":1,"request":{"action":"create","title":"alerte159","members":[peer],"operation_id":Uuid::new_v4().to_string()}})).unwrap();
        let (created, _) = crate::threads::handle_with_change(
            &st.store,
            &DaemonDirectory { state: &st },
            owner,
            request,
            now,
        );
        assert_eq!(created.result["status"], "created", "{}", created.result);
        let thread = created.result["thread_id"].as_str().unwrap();
        let request=serde_json::from_value(serde_json::json!({"version":1,"request":{"action":"post","thread_id":thread,"body":"vraie alerte159","notify":[owner],"operation_id":Uuid::new_v4().to_string()}})).unwrap();
        let (posted, _) = crate::threads::handle_with_change(
            &st.store,
            &DaemonDirectory { state: &st },
            peer,
            request,
            now,
        );
        assert_eq!(posted.result["status"], "posted", "{}", posted.result);
        dispatch_thread_wakes(&mut st, now, Instant::now(), &mut controls);
    }
    assert!(execute_controls(controls).is_empty());
    let id = match read_control(&mut reader) {
        DaemonToWrapper::DeliverIdempotent {
            message,
            delivery_id,
            ..
        } => {
            assert!(message.thread_notice.is_some());
            assert_eq!(message.origin, Some(bridget_core::MessageOrigin::System));
            delivery_id
        }
        other => panic!("alerte102 réelle attendue : {other:?}"),
    };
    let phase: String = shared
        .lock()
        .unwrap()
        .store
        .connection()
        .query_row(
            "SELECT phase FROM send_deliveries WHERE delivery_id=?1",
            [id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(phase, "dispatching");
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn held159_hold_mission_native() {
    use super::native_delegation::native_delegation_permissions149_tests::{
        fixture_with_reader, mission_native,
    };
    let (mut st, config, commands, _parent_reader) = fixture_with_reader("h159-nm");
    // d5af84f8 : une mission passe par son échange admis (mission_pending).
    let task = mission_native(&mut st, &commands, "held-native159", "mission_pending");
    st.registry = crate::registry::AgentRegistry::from_json(
        &serde_json::json!({"agents":{"native-test":{"command":"/bin/sh","args":["app-server"],
                "protocol":"codex_app_server","forbidden_env":[],"held_queue":true,
                "capabilities":{"execution_paths":["codex_app_server"]}}}})
        .to_string(),
        st.registry.source().to_path_buf(),
    )
    .unwrap();
    let instance = task.child_instance.clone().unwrap();
    let (writer, mut reader) = control_socket("h159-nc");
    st.connections.insert("held-native-child".into(), writer);
    let host = st.host.clone();
    assert!(matches!(
        handle_register_with_channel(
            "held-native-child",
            2,
            "native-test".into(),
            task.child.clone(),
            Some(host),
            Some("codex_app_server".into()),
            ChannelReport::Known("unix".into()),
            Some(PresenceMode::Acp),
            None,
            Some("macOS".into()),
            Some(instance.clone()),
            None,
            false,
            Some(true),
            &mut st,
            true
        ),
        DaemonToWrapper::Registered {
            credential: Some(_),
            ..
        }
    ));
    st.conn_instances
        .insert("held-native-child".into(), instance.clone());
    let mut presence = st.presences["instance-1"].clone();
    presence.name = task.child.clone();
    presence.agent_type = "native-test".into();
    presence.transport = "codex_app_server".into();
    presence.state = "busy".into();
    presence.turn_attest = true;
    st.presences.insert(instance, presence);
    assert_eq!(
        st.delegation_store
            .for_mission(&task.mission)
            .unwrap()
            .unwrap()
            .task_id,
        task.task_id
    );
    // La preuve for_mission seule : sans intention, seule la mission connue
    // échappe à la rétention (should_hold), toutes autres conditions égales.
    let mut passive = BridgetMessage::new(&task.owner, &task.child, "passif");
    passive.id = task.mission.clone();
    assert!(passive.intent.is_none());
    let definition = st.registry.get("native-test").unwrap();
    let child_presence = &st.presences[task.child_instance.as_ref().unwrap()];
    assert!(should_hold(
        definition,
        child_presence,
        &passive,
        false,
        false
    ));
    assert!(!should_hold(
        definition,
        child_presence,
        &passive,
        false,
        true
    ));
    // Depuis d5af84f8, l'échange natif n'accepte que la mission canonique admise.
    let message = super::native_delegation::remote_mission_message(&task).unwrap();
    let mut controls = Vec::new();
    let response = super::native_delegation::send_as(
        &mut st,
        &task.owner,
        &task.owner_instance,
        &task.task_id,
        message,
        unix_now_secs(),
        &mut controls,
    );
    assert!(
        matches!(
            response,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown {
                    delivery_id: Some(_),
                    ..
                },
                ..
            }
        ),
        "{response:?}"
    );
    assert!(execute_controls(controls).is_empty());
    let id = match read_control(&mut reader) {
        DaemonToWrapper::DeliverIdempotent {
            message,
            delivery_id,
            ..
        } => {
            assert_eq!(message.id, task.mission);
            delivery_id
        }
        other => panic!("mission admise réelle : {other:?}"),
    };
    let phase: String = st
        .store
        .connection()
        .query_row(
            "SELECT phase FROM send_deliveries WHERE delivery_id=?1",
            [id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(phase, "dispatching");
    drop(st);
    drop(commands);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn canonical_send_ignore_le_nom_affiche_et_le_timeout_relatif() {
    let mut original = idempotent_message("enveloppe stable");
    original.reply = true;
    original.reply_timeout = Some(20);
    original.deadline_at = Some(123_456);
    let mut renamed = original.clone();
    renamed.from = "guichet-renomme".to_string();
    renamed.reply_timeout = Some(90);
    assert_eq!(
        canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &original, 123_000),
        canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &renamed, 123_000)
    );
    let mut controle = original.clone();
    controle.origin = Some(bridget_core::MessageOrigin::Human);
    controle.intent = Some(bridget_core::MessageIntent::SteerCurrent);
    controle.references = vec!["delegation-1".to_string()];
    assert_ne!(
        canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &original, 123_000),
        canonical_send("012_scope_aaaaaaaaaaaa", "message-1", &controle, 123_000),
        "une intention ou une référence ne doit jamais rejouer un envoi passif"
    );
}

#[test]
fn spec251_information_dix_depots_sans_reveil_puis_vraie_demande() {
    let (state, _config) = state_with_registered_agent("info251");
    let shared = Arc::new(Mutex::new(state));
    negotiate_idempotent_client(
        &shared,
        "info-client",
        "012_scope_251aaaaaaaaa",
        "89000000-0000-4000-8000-000000000200",
    );
    let now = unix_now_secs();
    let scope = "012_scope_251aaaaaaaaa";
    {
        let mut st = shared.lock().unwrap();
        st.client_negotiations
            .get_mut("info-client")
            .unwrap()
            .capabilities
            .push(ClientCapability::SendInformationV1);
        st.presences.get_mut("instance-1").unwrap().dnd_until =
            Some(Instant::now() + Duration::from_secs(60));
        let agent = agent_tracking::directory(&mut st)
            .unwrap()
            .into_iter()
            .find(|a| a.agent.agent_id.ends_with("102"))
            .unwrap();
        crate::agent_tracking::set(
            st.store.connection(),
            "human",
            &Uuid::new_v4().to_string(),
            b"sleep",
            &agent,
            true,
            Some(false),
            None,
            now,
        )
        .unwrap();
    }
    let order = |i: usize| {
        let mut message = idempotent_message(&format!("Avis sans action {i}"));
        message.intent = Some(bridget_core::MessageIntent::StoreOnly);
        message.hops = 0;
        WrapperToDaemon::SendIdempotent {
            message,
            message_id: format!("info251-{i}"),
            issued_at: now,
        }
    };
    {
        let mut st = shared.lock().unwrap();
        for _ in 0..8 {
            st.circuit_breaker.record(
                &format!("client:{scope}"),
                "89000000-0000-4000-8000-000000000102",
            );
        }
        for i in 0..10 {
            let WrapperToDaemon::SendIdempotent {
                message,
                message_id,
                ..
            } = order(i)
            else {
                unreachable!()
            };
            st.deduplicator
                .mark_sent(&format!("{scope}:{}", message.content_key()), &message.to);
            st.envelope_guard
                .mark_relayed(&format!("{scope}:{message_id}"), &message.to);
        }
    }
    for i in 0..10 {
        let response = handle_wrapper_message("info-client", order(i), &shared);
        assert!(
            matches!(
                response,
                Some(DaemonToWrapper::IdempotencyResult {
                    issue: IdempotencyIssue::Accepted { .. },
                    ..
                })
            ),
            "{response:?}"
        );
        assert_eq!(
            serde_json::to_value(&response).unwrap()["issue"],
            serde_json::to_value(handle_wrapper_message("info-client", order(i), &shared)).unwrap()
                ["issue"]
        );
    }
    {
        let st = shared.lock().unwrap();
        assert_eq!(st.store.recent_messages(20).unwrap().len(), 10);
        for table in [
            "send_deliveries",
            "tracked_requests",
            "work_submissions",
            "executions",
            "execution_queue",
        ] {
            let count: i64 = st
                .store
                .connection()
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
                .unwrap();
            assert_eq!(count, 0, "{table}");
        }
        assert!(
            st.pending_post_response_controls
                .values()
                .all(Vec::is_empty)
        );
        let mut stored = idempotent_message("Interdit au dispatcher");
        stored.intent = Some(bridget_core::MessageIntent::StoreOnly);
        let delivery = SendDelivery {
            delivery_id: "forbidden251".into(),
            recipient_instance_id: "instance-1".into(),
            delivery_generation: 1,
            expires_at: now + 60,
            message_bytes: serde_json::to_vec(&stored).unwrap(),
        };
        let mut controls = Vec::new();
        assert!(
            defer_idempotent_delivery(&st, "conn-1", delivery, &mut controls, None, None).is_err()
        );
        assert!(
            controls.is_empty(),
            "aucun contrôle T3/ACP/Claude/Codex ne peut être produit"
        );
        assert!(st.pending_replies.is_empty());
        assert!(
            crate::agent_tracking::dormant(
                st.store.connection(),
                "89000000-0000-4000-8000-000000000102"
            )
            .unwrap()
        );
    }
    let mut changed = order(0);
    if let WrapperToDaemon::SendIdempotent { message, .. } = &mut changed {
        message.intent = None;
    }
    assert!(matches!(
        handle_wrapper_message("info-client", changed, &shared),
        Some(DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::EnvelopeMismatch,
            ..
        })
    ));
    {
        let mut st = shared.lock().unwrap();
        st.presences.get_mut("instance-1").unwrap().dnd_until = None;
        let agent = agent_tracking::directory(&mut st)
            .unwrap()
            .into_iter()
            .find(|a| a.agent.agent_id.ends_with("102"))
            .unwrap();
        crate::agent_tracking::set(
            st.store.connection(),
            "human",
            &Uuid::new_v4().to_string(),
            b"wake",
            &agent,
            false,
            Some(false),
            None,
            now,
        )
        .unwrap();
    }
    let limited = handle_wrapper_message(
        "info-client",
        WrapperToDaemon::SendIdempotent {
            message: idempotent_message("Action sous saturation"),
            message_id: "normal-limited251".into(),
            issued_at: now,
        },
        &shared,
    );
    assert!(
        matches!(limited,Some(DaemonToWrapper::IdempotencyResult{issue:IdempotencyIssue::Rejected{category,..},..}) if category=="circuit_breaker"),
        "le mode normal conserve ses gardes"
    );
    shared.lock().unwrap().circuit_breaker = CircuitBreaker::new(180, 8);
    let mut action = idempotent_message("Une vraie question");
    action.reply = true;
    let response = handle_wrapper_message(
        "info-client",
        WrapperToDaemon::SendIdempotent {
            message: action,
            message_id: "action251".into(),
            issued_at: now,
        },
        &shared,
    );
    assert!(
        matches!(
            response,
            Some(DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown {
                    delivery_id: Some(_),
                    ..
                },
                ..
            })
        ),
        "{response:?}"
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .store
            .get_request("action251")
            .unwrap()
            .unwrap()
            .state,
        "open"
    );
}

#[test]
fn spec251_information_refuse_correlation_et_capacite_absente_avant_capture() {
    let (state, _config) = state_with_registered_agent("info251-refus");
    let shared = Arc::new(Mutex::new(state));
    negotiate_idempotent_client(
        &shared,
        "info-client",
        "012_scope_251bbbbbbbbb",
        "89000000-0000-4000-8000-000000000200",
    );
    let mut message = idempotent_message("Un avis");
    message.intent = Some(bridget_core::MessageIntent::StoreOnly);
    let now = unix_now_secs();
    let order = WrapperToDaemon::SendIdempotent {
        message: message.clone(),
        message_id: "no-cap251".into(),
        issued_at: now,
    };
    assert!(matches!(
        handle_wrapper_message("info-client", order, &shared),
        Some(DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::CapabilityNotNegotiated
        })
    ));
    shared
        .lock()
        .unwrap()
        .client_negotiations
        .get_mut("info-client")
        .unwrap()
        .capabilities
        .push(ClientCapability::SendInformationV1);
    for i in 0..3 {
        let mut invalid = message.clone();
        match i {
            0 => invalid.reply = true,
            1 => invalid.in_reply_to = Some("result-native".into()),
            _ => invalid.reply_timeout = Some(60),
        };
        let order = WrapperToDaemon::SendIdempotent {
            message: invalid,
            message_id: format!("bad-info251-{i}"),
            issued_at: now,
        };
        assert!(
            matches!(handle_wrapper_message("info-client",order.clone(),&shared),Some(DaemonToWrapper::IdempotencyResult{issue:IdempotencyIssue::Rejected{category,..},..}) if category=="information_requires_uncorrelated_message")
        );
        assert!(matches!(
            handle_wrapper_message("info-client", order, &shared),
            Some(DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::Rejected { .. },
                ..
            })
        ));
    }
    assert!(
        matches!(handle_wrapper_message("conn-1",WrapperToDaemon::Send(message),&shared),Some(DaemonToWrapper::Nack{reason,..}) if reason.starts_with("information_requires_idempotent_send"))
    );
    let st = shared.lock().unwrap();
    assert!(st.store.recent_messages(10).unwrap().is_empty());
    assert!(st.pending_replies.is_empty());
    drop(st);
    let mut existing = idempotent_message("Travail préexistant conservé");
    existing.id = "collision-info251".into();
    shared
        .lock()
        .unwrap()
        .store
        .record_message(&existing, "fixture")
        .unwrap();
    let mut info = existing.clone();
    info.intent = Some(bridget_core::MessageIntent::StoreOnly);
    info.body = "Un autre avis".into();
    let order = WrapperToDaemon::SendIdempotent {
        message: info,
        message_id: existing.id.clone(),
        issued_at: now,
    };
    for _ in 0..2 {
        let reply = handle_wrapper_message("info-client", order.clone(), &shared);
        assert!(
            matches!(reply,Some(DaemonToWrapper::IdempotencyResult{issue:IdempotencyIssue::Rejected{category,..},..}) if category=="information_id_conflict")
        );
    }
    let st = shared.lock().unwrap();
    let messages = st.store.recent_messages(10).unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].body, existing.body);
}

#[test]
fn send_idempotent_rejoue_sans_rerouter_et_isole_les_scopes() {
    let (state, config) = state_with_registered_agent("idempotent-send");
    let shared = Arc::new(Mutex::new(state));
    let scope_a = "012_scope_aaaaaaaaaaaa";
    let scope_b = "012_scope_bbbbbbbbbbbb";
    negotiate_idempotent_client(
        &shared,
        "client-a",
        scope_a,
        "89000000-0000-4000-8000-000000000200",
    );
    let issued_at = unix_now_secs();
    let first = handle_wrapper_message(
        "client-a",
        WrapperToDaemon::SendIdempotent {
            message: idempotent_message("tâche durable"),
            message_id: "message-identique".to_string(),
            issued_at,
        },
        &shared,
    );
    let first_delivery = match first {
        Some(DaemonToWrapper::IdempotencyResult {
            issue:
                IdempotencyIssue::OutcomeUnknown {
                    delivery_id: Some(delivery_id),
                    ..
                },
            ..
        }) => delivery_id,
        other => panic!("réponse inattendue: {other:?}"),
    };
    negotiate_idempotent_client(
        &shared,
        "client-b",
        scope_b,
        "89000000-0000-4000-8000-000000000200",
    );
    let other_scope = handle_wrapper_message(
        "client-b",
        WrapperToDaemon::SendIdempotent {
            message: idempotent_message("tâche durable"),
            message_id: "message-identique".to_string(),
            issued_at,
        },
        &shared,
    );
    assert!(matches!(
        other_scope,
        Some(DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::OutcomeUnknown {
                delivery_id: Some(delivery_id),
                ..
            },
            ..
        }) if delivery_id != first_delivery
    ));
    let state = shared.lock().unwrap();
    let first_key = IdempotencyKey::new(scope_a, OperationKind::Send, "message-identique").unwrap();
    let second_key =
        IdempotencyKey::new(scope_b, OperationKind::Send, "message-identique").unwrap();
    assert_ne!(
        state
            .idempotency
            .send_delivery(&first_key)
            .unwrap()
            .unwrap()
            .delivery_generation,
        state
            .idempotency
            .send_delivery(&second_key)
            .unwrap()
            .unwrap()
            .delivery_generation
    );
    drop(state);
    // Un rejeu est jugé avant tout routage : retirer la cible
    // n'autorise jamais une nouvelle résolution pour cette clé connue.
    {
        let mut state = shared.lock().unwrap();
        assert!(state.router.unregister_by_conn("conn-1").is_some());
    }
    let replay = handle_wrapper_message(
        "client-a",
        WrapperToDaemon::SendIdempotent {
            message: idempotent_message("tâche durable"),
            message_id: "message-identique".to_string(),
            issued_at,
        },
        &shared,
    );
    assert!(matches!(
        replay,
        Some(DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::OutcomeUnknown {
                delivery_id: Some(delivery_id),
                ..
            },
            ..
        }) if delivery_id == first_delivery
    ));
    let mismatch = handle_wrapper_message(
        "client-a",
        WrapperToDaemon::SendIdempotent {
            message: idempotent_message("tâche différente"),
            message_id: "message-identique".to_string(),
            issued_at,
        },
        &shared,
    );
    assert!(matches!(
        mismatch,
        Some(DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::EnvelopeMismatch,
            ..
        })
    ));
    let state = shared.lock().unwrap();
    assert_eq!(state.idempotency.record_count().unwrap(), 2);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn send_idempotent_persiste_un_refus_et_la_voie_historique_ne_contamine_pas_le_socle() {
    let (state, config) = state_with_registered_agent("idempotent-rejection");
    let shared = Arc::new(Mutex::new(state));
    negotiate_idempotent_client(
        &shared,
        "client-reject",
        "012_scope_cccccccccccc",
        "89000000-0000-4000-8000-000000000200",
    );
    let issued_at = unix_now_secs();
    let mut unknown_target = BridgetMessage::new(
        "89000000-0000-4000-8000-000000000200",
        "inconnu",
        "à refuser",
    );
    unknown_target.hops = 4;
    let send = || WrapperToDaemon::SendIdempotent {
        message: unknown_target.clone(),
        message_id: "message-refuse".to_string(),
        issued_at,
    };
    let rejected = handle_wrapper_message("client-reject", send(), &shared);
    assert!(matches!(
        rejected.as_ref(),
        Some(DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::Rejected { category, .. },
            ..
        }) if category == "routing"
    ));
    let replay = handle_wrapper_message("client-reject", send(), &shared);
    assert!(matches!(
        replay.as_ref(),
        Some(DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::Rejected { category, .. },
            ..
        }) if category == "routing"
    ));
    let rejection_expiry = match rejected {
        Some(DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::Rejected { expires_at, .. },
            ..
        }) => expires_at,
        _ => unreachable!("refus idempotent attendu"),
    };
    let replay_expiry = match replay {
        Some(DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::Rejected { expires_at, .. },
            ..
        }) => expires_at,
        _ => unreachable!("rejeu du refus attendu"),
    };
    assert_eq!(replay_expiry, rejection_expiry);

    let _target_peer = super::spec094_live_test_connection(&mut shared.lock().unwrap(), "conn-1");
    let historic = handle_wrapper_message(
        "owner-89000000-0000-4000-8000-000000000200",
        WrapperToDaemon::Send(BridgetMessage::new(
            "89000000-0000-4000-8000-000000000200",
            "89000000-0000-4000-8000-000000000102",
            "ancienne voie",
        )),
        &shared,
    );
    assert!(matches!(historic, Some(DaemonToWrapper::Ack { .. })));
    assert_eq!(
        shared.lock().unwrap().idempotency.record_count().unwrap(),
        1
    );
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn send_idempotent_conserve_les_invariants_de_reponse_et_dnd() {
    let (mut state, config) = state_with_registered_agent("idempotent-reply");
    state
        .router
        .register(
            "89000000-0000-4000-8000-000000000122",
            &bridget_core::AgentType::Codex,
            "sender-wrapper",
        )
        .unwrap();
    state
        .store
        .create_request(
            "request-open",
            "89000000-0000-4000-8000-000000000102",
            "89000000-0000-4000-8000-000000000122",
            60,
        )
        .unwrap();
    state.presences.get_mut("instance-1").unwrap().dnd_until =
        Some(Instant::now() + Duration::from_secs(60));
    let shared = Arc::new(Mutex::new(state));
    negotiate_idempotent_client(
        &shared,
        "client-reply",
        "012_scope_replyyyyyyyyy",
        "89000000-0000-4000-8000-000000000122",
    );
    let mut reply = idempotent_message("réponse suivie");
    reply.from = "89000000-0000-4000-8000-000000000122".to_string();
    reply.reply = true;
    reply.in_reply_to = Some("request-open".to_string());
    let issued_at = unix_now_secs();
    let result = handle_wrapper_message(
        "client-reply",
        WrapperToDaemon::SendIdempotent {
            message: reply,
            message_id: "reply-idempotent".to_string(),
            issued_at,
        },
        &shared,
    );
    assert!(matches!(
        result,
        Some(DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::OutcomeUnknown { .. },
            ..
        })
    ));
    let state = shared.lock().unwrap();
    assert_eq!(
        state
            .store
            .get_request("reply-idempotent")
            .unwrap()
            .unwrap()
            .state,
        "open"
    );
    assert_eq!(
        state
            .store
            .get_request("reply-idempotent")
            .unwrap()
            .unwrap()
            .deadline_at,
        issued_at + 60
    );
    assert!(
        state
            .pending_replies
            .iter()
            .any(|pending| pending.msg_id == "reply-idempotent")
    );
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn accuse_idempotent_d_une_reponse_liee_resout_la_demande_sans_relance() {
    let (mut state, config) = state_with_registered_agent("idempotent-linked-reply");
    let (target_writer, mut target_reader) = control_socket("idempotent-linked-reply");
    state
        .connections
        .insert("conn-1".to_string(), target_writer);
    state
        .store
        .create_request(
            "request-open",
            "89000000-0000-4000-8000-000000000102",
            "89000000-0000-4000-8000-000000000124",
            60,
        )
        .unwrap();
    state.pending_replies.push(PendingReply {
        msg_id: "request-open".to_string(),
        from: "89000000-0000-4000-8000-000000000102".to_string(),
        from_conn: "conn-1".to_string(),
        to: "89000000-0000-4000-8000-000000000124".to_string(),
        target_conn: "client-reply".to_string(),
        timeout_secs: 60,
        created_at: Instant::now(),
        frozen_secs: 0,
        escalation_level: 0,
        deferred_level: None,
    });
    let shared = Arc::new(Mutex::new(state));
    negotiate_idempotent_client(
        &shared,
        "client-reply",
        "012_scope_replyyyyyyyyy",
        "89000000-0000-4000-8000-000000000124",
    );

    let mut response = BridgetMessage::new(
        "89000000-0000-4000-8000-000000000124",
        "89000000-0000-4000-8000-000000000102",
        "réponse MCP",
    );
    response.in_reply_to = Some("request-open".to_string());
    let issued_at = unix_now_secs();
    assert!(matches!(
        handle_wrapper_message(
            "client-reply",
            WrapperToDaemon::SendIdempotent {
                message: response,
                message_id: "mcp-linked-reply".to_string(),
                issued_at,
            },
            &shared,
        ),
        Some(DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::OutcomeUnknown { .. },
            ..
        })
    ));
    let (delivery_id, delivery_generation) = match read_control(&mut target_reader) {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            message,
            ..
        } => {
            assert_eq!(message.in_reply_to.as_deref(), Some("request-open"));
            (delivery_id, delivery_generation)
        }
        other => panic!("remise idempotente attendue: {other:?}"),
    };
    assert!(
        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::DeliverAcked {
                delivery_id,
                delivery_generation,
            },
            &shared,
        )
        .is_none()
    );

    let state = shared.lock().unwrap();
    assert_eq!(
        state
            .store
            .get_request("request-open")
            .unwrap()
            .unwrap()
            .state,
        "answered"
    );
    assert!(state.pending_replies.is_empty());
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn meme_cle_idempotente_avec_reponse_liee_divergente_est_refusee_sans_mutation() {
    let (state, config) = state_with_registered_agent("idempotent-linked-mismatch");
    state
        .store
        .create_request(
            "request-a",
            "89000000-0000-4000-8000-000000000102",
            "89000000-0000-4000-8000-000000000124",
            60,
        )
        .unwrap();
    state
        .store
        .create_request(
            "request-b",
            "89000000-0000-4000-8000-000000000102",
            "89000000-0000-4000-8000-000000000124",
            60,
        )
        .unwrap();
    let shared = Arc::new(Mutex::new(state));
    negotiate_idempotent_client(
        &shared,
        "client-reply",
        "012_scope_replyyyyyyyyy",
        "89000000-0000-4000-8000-000000000124",
    );
    let issued_at = unix_now_secs();
    for request_id in ["request-a", "request-b"] {
        let mut response = BridgetMessage::new(
            "89000000-0000-4000-8000-000000000124",
            "89000000-0000-4000-8000-000000000102",
            "réponse MCP",
        );
        response.in_reply_to = Some(request_id.to_string());
        let result = handle_wrapper_message(
            "client-reply",
            WrapperToDaemon::SendIdempotent {
                message: response,
                message_id: "mcp-linked-mismatch".to_string(),
                issued_at,
            },
            &shared,
        );
        if request_id == "request-a" {
            assert!(matches!(
                result,
                Some(DaemonToWrapper::IdempotencyResult {
                    issue: IdempotencyIssue::OutcomeUnknown { .. },
                    ..
                })
            ));
        } else {
            assert!(matches!(
                result,
                Some(DaemonToWrapper::IdempotencyResult {
                    issue: IdempotencyIssue::EnvelopeMismatch,
                    ..
                })
            ));
        }
    }
    let state = shared.lock().unwrap();
    assert_eq!(
        state.store.get_request("request-a").unwrap().unwrap().state,
        "open"
    );
    assert_eq!(
        state.store.get_request("request-b").unwrap().unwrap().state,
        "open"
    );
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn envoi_attach_force_humain_et_ne_clot_pas_une_demande_forgee() {
    let (mut state, config) = state_with_registered_agent("attach-humain");
    let listener = UnixListener::bind(config.socket_path.with_extension("target.sock")).unwrap();
    let _receiver = UnixStream::connect(config.socket_path.with_extension("target.sock")).unwrap();
    let (target_stream, _) = listener.accept().unwrap();
    state.connections.insert(
        "conn-1".to_string(),
        Arc::new(Mutex::new(BufWriter::new(target_stream))),
    );
    state
        .store
        .create_request(
            "request-1",
            "89000000-0000-4000-8000-000000000102",
            "codex-1",
            60,
        )
        .unwrap();
    let shared = Arc::new(Mutex::new(state));
    handle_wrapper_message(
        "attach-1",
        WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Attach,
        },
        &shared,
    );

    let mut forged = bridget_core::BridgetMessage::new(
        "codex-1",
        "89000000-0000-4000-8000-000000000102",
        "réponse forgée",
    );
    forged.in_reply_to = Some("request-1".to_string());
    let response = handle_wrapper_message("attach-1", WrapperToDaemon::Send(forged), &shared);
    assert!(matches!(response, Some(DaemonToWrapper::Ack { .. })));

    let state = shared.lock().unwrap();
    assert_eq!(state.store.recent_messages(1).unwrap()[0].sender, "humain");
    assert_eq!(
        state.store.get_request("request-1").unwrap().unwrap().state,
        "open"
    );
    let _ = std::fs::remove_file(config.db_path);
    let _ = std::fs::remove_file(config.socket_path.with_extension("target.sock"));
}

#[test]
fn abonnement_attach_attend_le_wrapper_et_ignore_la_fin_d_une_ancienne_generation() {
    let (mut state, config) = state_with_registered_agent("attach-cycle");
    let (wrapper_writer, mut wrapper_reader) = control_socket("wrapper");
    let (attach_writer, mut attach_reader) = control_socket("attach");
    state
        .connections
        .insert("conn-1".to_string(), wrapper_writer);
    state
        .connections
        .insert("attach-1".to_string(), attach_writer);
    let shared = Arc::new(Mutex::new(state));
    assert!(matches!(
        handle_wrapper_message(
            "attach-1",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach,
            },
            &shared,
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));

    assert!(
        handle_wrapper_message(
            "attach-1",
            WrapperToDaemon::Subscribe {
                agent: "89000000-0000-4000-8000-000000000102".to_string(),
                window: bridget_transport::AttachWindow::Today,
            },
            &shared,
        )
        .is_none()
    );
    let first_id = match read_control(&mut wrapper_reader) {
        DaemonToWrapper::Subscribe {
            subscription_id, ..
        } => subscription_id,
        other => panic!("commande wrapper inattendue: {}", encode(&other).unwrap()),
    };
    assert_eq!(
        shared.lock().unwrap().agent_infos().len(),
        1,
        "attach reste hors de who"
    );
    assert!(
        shared
            .lock()
            .unwrap()
            .attach_subscriptions
            .contains_key(&first_id)
    );
    assert!(
        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::Subscribed {
                subscription_id: first_id.clone(),
            },
            &shared,
        )
        .is_none()
    );
    assert!(matches!(
        read_control(&mut attach_reader),
        DaemonToWrapper::Subscribed { subscription_id } if subscription_id == first_id
    ));
    handle_wrapper_message(
        "conn-1",
        WrapperToDaemon::JournalReadError {
            subscription_id: first_id.clone(),
            line: 12,
            offset: 384,
            reason: "ligne corrompue".to_string(),
        },
        &shared,
    );
    assert!(matches!(
        read_control(&mut attach_reader),
        DaemonToWrapper::JournalReadError {
            subscription_id,
            line: 12,
            offset: 384,
            reason,
        } if subscription_id == first_id && reason == "ligne corrompue"
    ));

    handle_wrapper_message(
        "attach-1",
        WrapperToDaemon::Subscribe {
            agent: "89000000-0000-4000-8000-000000000102".to_string(),
            window: bridget_transport::AttachWindow::Seq(8),
        },
        &shared,
    );
    let second_id = match read_control(&mut wrapper_reader) {
        DaemonToWrapper::Subscribe {
            subscription_id, ..
        } => subscription_id,
        other => panic!("commande wrapper inattendue: {}", encode(&other).unwrap()),
    };
    handle_wrapper_message(
        "conn-1",
        WrapperToDaemon::Subscribed {
            subscription_id: second_id.clone(),
        },
        &shared,
    );
    assert!(matches!(
        read_control(&mut attach_reader),
        DaemonToWrapper::Subscribed { subscription_id } if subscription_id == second_id
    ));

    handle_wrapper_message(
        "conn-1",
        WrapperToDaemon::End {
            subscription_id: first_id.clone(),
            reason: "ancienne génération".to_string(),
        },
        &shared,
    );
    assert!(matches!(
        read_control(&mut attach_reader),
        DaemonToWrapper::End { subscription_id, .. } if subscription_id == first_id
    ));
    assert!(
        shared
            .lock()
            .unwrap()
            .attach_subscriptions
            .contains_key(&second_id)
    );

    let (controls, views) = {
        let mut state = shared.lock().unwrap();
        let (controls, views) = close_attach_subscriptions(&mut state, "attach-1");
        assert!(state.attach_subscriptions.is_empty());
        (controls, views)
    };
    assert!(execute_controls(controls).is_empty());
    for view in views {
        view.close_and_join();
    }
    assert!(matches!(
        read_control(&mut wrapper_reader),
        DaemonToWrapper::Unsubscribe { subscription_id } if subscription_id == second_id
    ));
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn abonnement_attach_refuse_un_pilote_sans_journal_malgre_son_mode() {
    let (mut state, config) = state_with_registered_agent("attach-tmux");
    let presence = state.presences.get_mut("instance-1").unwrap();
    presence.transport = "unix".to_string();
    presence.mode = Some(PresenceMode::Tmux);
    presence.location = Some("bridget:1.2".to_string());
    presence.journal_available = false;
    let shared = Arc::new(Mutex::new(state));
    assert!(matches!(
        handle_wrapper_message(
            "attach-1",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach,
            },
            &shared,
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "attach-1",
            WrapperToDaemon::Subscribe {
                agent: "89000000-0000-4000-8000-000000000102".to_string(),
                window: bridget_transport::AttachWindow::Today,
            },
            &shared,
        ),
        Some(DaemonToWrapper::AttachRejected {
            reason: AttachRefusal::JournalUnavailable,
            ..
        })
    ));
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn une_identite_connue_qui_change_de_voie_prend_le_mode_annonce() {
    // Session 097 : une session Claude relancée avec le nouveau wrapper
    // annonce `cli` + `claude_pty` ; l'ancien mode tmux mémorisé pour le
    // même UUID ne doit pas lui coller à la peau. Un binaire ancien qui
    // n'annonce aucun mode conserve, lui, l'observation précédente.
    let (mut state, _config) = state_with_registered_agent("presence-changement-de-voie");
    state.router.unregister_by_conn("conn-1");
    state.conn_instances.remove("conn-1");
    state.presences.clear();
    let agent = "89000000-0000-4000-8000-000000000122";
    let register = |state: &mut DaemonState,
                    conn: &str,
                    instance: &str,
                    transport: &str,
                    mode: Option<PresenceMode>,
                    location: Option<&str>| {
        assert!(matches!(
            handle_register(
                conn,
                "claude".to_string(),
                Some(agent.to_string()),
                Some("local".to_string()),
                Some(transport.to_string()),
                mode,
                location.map(str::to_string),
                Some("test".to_string()),
                Some(instance.to_string()),
                None,
                false,
                None,
                state,
            ),
            DaemonToWrapper::Registered { .. }
        ));
    };
    register(
        &mut state,
        "tmux-conn",
        "instance-tmux",
        "tmux",
        Some(PresenceMode::Tmux),
        Some("bridget:1.0"),
    );
    state.router.unregister_by_conn("tmux-conn");
    state.conn_instances.remove("tmux-conn");
    register(
        &mut state,
        "pty-conn",
        "instance-pty",
        "claude_pty",
        Some(PresenceMode::Cli),
        None,
    );
    let infos = state.agent_infos();
    let info = infos.iter().find(|a| a.agent_id == agent).unwrap();
    assert_eq!(info.mode, Some(PresenceMode::Cli));
    assert_eq!(info.transport, "claude_pty");
    assert_eq!(info.location, None);
    state.router.unregister_by_conn("pty-conn");
    state.conn_instances.remove("pty-conn");
    register(
        &mut state,
        "ancien-conn",
        "instance-ancienne",
        "unix",
        None,
        None,
    );
    let infos = state.agent_infos();
    let info = infos.iter().find(|a| a.agent_id == agent).unwrap();
    assert_eq!(
        info.mode,
        Some(PresenceMode::Cli),
        "un binaire sans champ mode garde l'observation précédente"
    );
}

#[test]
fn modes_reels_des_trois_enregistrements_ne_dependant_pas_du_transport() {
    let (mut state, config) = state_with_registered_agent("presence-modes");
    state.router.unregister_by_conn("conn-1");
    state.conn_instances.remove("conn-1");
    state.presences.clear();

    let registrations = [
        (
            "acp-conn",
            "89000000-0000-4000-8000-000000000119",
            "acp-instance",
            PresenceMode::Acp,
            None,
        ),
        (
            "tmux-conn",
            "89000000-0000-4000-8000-000000000120",
            "tmux-instance",
            PresenceMode::Tmux,
            Some("bridget:3.1"),
        ),
        (
            "cli-conn",
            "89000000-0000-4000-8000-000000000121",
            "cli-instance",
            PresenceMode::Cli,
            None,
        ),
    ];
    for (conn_id, name, instance_id, mode, location) in registrations {
        assert!(matches!(
            handle_register(
                conn_id,
                "fixture".to_string(),
                Some(name.to_string()),
                Some("local".to_string()),
                // Les trois chemins peuvent emprunter le même socket Unix :
                // seul le mode attesté autorise ou refuse attach.
                Some("unix".to_string()),
                Some(mode),
                location.map(str::to_string),
                Some("test".to_string()),
                Some(instance_id.to_string()),
                None,
                false,
                None,
                &mut state,
            ),
            DaemonToWrapper::Registered { .. }
        ));
    }

    let infos = state.agent_infos();
    let info = |name: &str| infos.iter().find(|agent| agent.agent_id == name).unwrap();
    assert_eq!(
        info("89000000-0000-4000-8000-000000000119").mode,
        Some(PresenceMode::Acp)
    );
    assert_eq!(
        info("89000000-0000-4000-8000-000000000120").mode,
        Some(PresenceMode::Tmux)
    );
    assert_eq!(
        info("89000000-0000-4000-8000-000000000120")
            .location
            .as_deref(),
        Some("bridget:3.1")
    );
    assert_eq!(
        info("89000000-0000-4000-8000-000000000121").mode,
        Some(PresenceMode::Cli)
    );
    assert!(
        info("89000000-0000-4000-8000-000000000121")
            .location
            .is_none()
    );

    let tmux_refusal =
        attach_refusal_for_subscription(&state, "89000000-0000-4000-8000-000000000120")
            .unwrap_err();
    assert_eq!(tmux_refusal.reason, AttachRefusal::JournalUnavailable);
    assert_eq!(tmux_refusal.mode, Some(PresenceMode::Tmux));
    assert_eq!(tmux_refusal.location.as_deref(), Some("bridget:3.1"));
    let cli_refusal =
        attach_refusal_for_subscription(&state, "89000000-0000-4000-8000-000000000121")
            .unwrap_err();
    assert_eq!(cli_refusal.reason, AttachRefusal::JournalUnavailable);
    assert_eq!(cli_refusal.mode, Some(PresenceMode::Cli));
    assert!(cli_refusal.location.is_none());

    // Un wrapper historique ne porte pas JournalReady mais son lancement
    // ACP n'atteint cette boucle qu'après activation du journal. Le
    // daemon neuf le laisse donc franchir le gate, sans exiger un
    // redémarrage atomique de tous les wrappers vivants.
    assert!(matches!(
        handle_register(
            "legacy-conn",
            "fixture".to_string(),
            Some("89000000-0000-4000-8000-000000000127".to_string()),
            Some("local".to_string()),
            Some("acp".to_string()),
            None,
            None,
            Some("test".to_string()),
            Some("legacy-instance".to_string()),
            None,
            false,
            None,
            &mut state,
        ),
        DaemonToWrapper::Registered { .. }
    ));
    let legacy_refusal =
        attach_refusal_for_subscription(&state, "89000000-0000-4000-8000-000000000127")
            .unwrap_err();
    assert_eq!(legacy_refusal.reason, AttachRefusal::WrapperUnavailable);
    assert!(legacy_refusal.mode.is_none());
    assert!(legacy_refusal.location.is_none());

    // Mutation discriminante de la transition : un wrapper nouveau
    // annonce explicitement `false` avant JournalReady et reste refusé.
    assert!(matches!(
        handle_register(
            "modern-conn",
            "fixture".to_string(),
            Some("89000000-0000-4000-8000-000000000128".to_string()),
            Some("local".to_string()),
            Some("acp".to_string()),
            Some(PresenceMode::Acp),
            None,
            Some("test".to_string()),
            Some("modern-instance".to_string()),
            None,
            false,
            Some(false),
            &mut state,
        ),
        DaemonToWrapper::Registered { .. }
    ));
    assert_eq!(
        attach_refusal_for_subscription(&state, "89000000-0000-4000-8000-000000000128")
            .unwrap_err()
            .reason,
        AttachRefusal::JournalUnavailable
    );

    // Le journal, et non le mode, est le gate attach. Une fois attesté
    // sur tmux comme sur un pilote CLI natif, le refus progresse jusqu'à
    // la disponibilité réelle du writer : la barrière de protocole est
    // donc franchie sans inférence.
    state
        .presences
        .get_mut("tmux-instance")
        .unwrap()
        .journal_available = true;
    assert_eq!(
        attach_refusal_for_subscription(&state, "89000000-0000-4000-8000-000000000120")
            .unwrap_err()
            .reason,
        AttachRefusal::WrapperUnavailable
    );
    state
        .presences
        .get_mut("cli-instance")
        .unwrap()
        .journal_available = true;
    assert_eq!(
        attach_refusal_for_subscription(&state, "89000000-0000-4000-8000-000000000121")
            .unwrap_err()
            .reason,
        AttachRefusal::WrapperUnavailable
    );
    let _ = std::fs::remove_file(config.db_path);
}

/// ORACLE PRINCIPAL — `agent_infos` ne ment plus.
///
/// Un `busy` dont `capacity_seen` a dépassé le retain (wrapper ACP sans
/// heartbeat d'autrefois) ne doit PAS réapparaître en `connected`/`unix`
/// inventés : zéro présence fantôme dans l'annuaire, nom libéré.
#[test]
fn presence_expiree_ne_projette_plus_connected_unix_invente() {
    let (mut state, config) = state_with_registered_agent("fantome-expire");
    state.set_turn_state("conn-1", true).unwrap();
    let stale = Instant::now()
        .checked_sub(PRESENCE_RETENTION + Duration::from_secs(1))
        .expect("horloge");
    let presence = state.presences.get_mut("instance-1").unwrap();
    presence.capacity_seen = stale;
    presence.link_seen = stale;

    let infos = state.agent_infos();
    assert!(
        infos
            .iter()
            .all(|agent| agent.agent_id != "89000000-0000-4000-8000-000000000102"),
        "le fantôme ne doit plus figurer dans l'annuaire: {infos:?}"
    );
    assert!(
        state
            .router
            .get_agent("89000000-0000-4000-8000-000000000102")
            .is_none(),
        "le nom doit être libéré du routeur après expiration de la présence"
    );
    assert!(!infos.iter().any(|agent| {
        agent.state == "connected" && agent.transport == "unix" && agent.mode.is_none()
    }));
    let _ = std::fs::remove_file(config.db_path);
}

/// ORACLE — à la purge de présence, une remise `dispatching` devient
/// `orphaned` (visible), pas un `outcome_unknown` muet.
///
/// Lot A : le retain lit `link_seen`. Stalifier seulement `capacity_seen`
/// laisserait le lien frais → présence non purgée → faux vert / faux rouge.
#[test]
fn spec244_purge_presence_conserve_les_remises_pour_la_meme_instance() {
    use crate::idempotency::{IdempotencyKey, OperationKind, Reservation, SendDelivery};

    let (mut state, config) = state_with_registered_agent("purge-orphelin");
    // busy + les DEUX horloges périmées (capacité et lien).
    state.set_turn_state("conn-1", true).unwrap();
    let stale = Instant::now()
        .checked_sub(PRESENCE_RETENTION + Duration::from_secs(1))
        .expect("horloge");
    let presence = state.presences.get_mut("instance-1").unwrap();
    presence.capacity_seen = stale;
    presence.link_seen = stale;

    let key = IdempotencyKey::new(
        "012_scope_aaaaaaaaaaaa",
        OperationKind::Send,
        "msg-jury-perdu",
    )
    .unwrap();
    assert!(matches!(
        state
            .idempotency
            .reserve(&key, b"canon-jury", 1_000_000, 3600, 1_000_000, 30),
        Ok(Reservation::Prepared { .. })
    ));
    let mut message = BridgetMessage::new(
        "bridget",
        "89000000-0000-4000-8000-000000000102",
        "mandat de jury",
    );
    message.id = "msg-jury-perdu".to_string();
    let message_bytes = serde_json::to_vec(&message).unwrap();
    state
        .idempotency
        .begin_send_delivery(
            &key,
            &SendDelivery {
                delivery_id: "delivery-jury".to_string(),
                recipient_instance_id: "instance-1".to_string(),
                delivery_generation: 1,
                expires_at: 1_000_000 + 3600,
                message_bytes,
            },
        )
        .unwrap();

    let _ = state.agent_infos();
    assert!(
        !state.presences.contains_key("instance-1"),
        "présence doit être purgée"
    );
    assert_eq!(
        state
            .idempotency
            .dispatching_deliveries_for_instance("instance-1", 1_000_000)
            .unwrap()
            .len(),
        1,
        "la remise durable attend le retour de la même instance"
    );
    assert_eq!(
        state
            .idempotency
            .orphaned_delivery_id(&key)
            .unwrap()
            .as_deref(),
        None
    );
    assert_eq!(
        state.idempotency.lookup(&key, 1_000_000).unwrap(),
        crate::idempotency::LookupResult::OutcomeUnknown {
            expires_at: 1_000_000 + 3600
        }
    );
    let _ = std::fs::remove_file(config.db_path);
}

/// ORACLE — moitié « pas en silence » : un Deliver ORPHELIN atteint l'émetteur.
/// Meurt si l'on retire le push (ou la notice durable) tout en gardant la phase.
#[test]
fn spec244_purge_presence_ne_fabrique_pas_un_echec_de_remise() {
    use crate::idempotency::{IdempotencyKey, OperationKind, Reservation, SendDelivery};
    use std::io::Read;
    use std::os::unix::net::UnixStream;

    let (mut state, config) = state_with_registered_agent("purge-deliver");
    // Émetteur « bridget » avec une vraie connexion lisible.
    state
        .router
        .register(
            "89000000-0000-4000-8000-000000000123",
            &bridget_core::AgentType::Claude,
            "conn-emitter",
        )
        .unwrap();
    state
        .conn_instances
        .insert("conn-emitter".to_string(), "instance-emitter".to_string());
    state.presences.insert(
        "instance-emitter".to_string(),
        Presence {
            name: "89000000-0000-4000-8000-000000000123".to_string(),
            agent_type: "claude".to_string(),
            host: "macbook".to_string(),
            transport: "acp".to_string(),
            channel: None,
            mode: Some(PresenceMode::Acp),
            location: None,
            journal_available: true,
            os: "macOS".to_string(),
            state: "connected".to_string(),
            busy_since: None,
            busy_period_start: None,
            last_attested_at: None,
            turn_attest: false,
            queue_fallback: false,
            capacity_seen: Instant::now(),
            link_seen: Instant::now(),
            reconnect_count: 0,
            model: None,
            effort: None,
            rate_limits: Default::default(),
            served_model: None,
            derived_domain: None,
            domain: None,
            dnd_until: None,
            disk_space: None,
        },
    );
    let (writer_stream, mut peer) = UnixStream::pair().unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    state.connections.insert(
        "conn-emitter".to_string(),
        Arc::new(Mutex::new(BufWriter::new(writer_stream))),
    );

    state.set_turn_state("conn-1", true).unwrap();
    let stale = Instant::now()
        .checked_sub(PRESENCE_RETENTION + Duration::from_secs(1))
        .expect("horloge");
    let presence = state.presences.get_mut("instance-1").unwrap();
    presence.capacity_seen = stale;
    presence.link_seen = stale;

    let key = IdempotencyKey::new(
        "012_scope_aaaaaaaaaaaa",
        OperationKind::Send,
        "msg-a-notifier",
    )
    .unwrap();
    assert!(matches!(
        state
            .idempotency
            .reserve(&key, b"canon-notify", 1_000_000, 3600, 1_000_000, 30),
        Ok(Reservation::Prepared { .. })
    ));
    let mut message = BridgetMessage::new(
        "89000000-0000-4000-8000-000000000123",
        "89000000-0000-4000-8000-000000000102",
        "corps",
    );
    message.id = "msg-a-notifier".to_string();
    state
        .idempotency
        .begin_send_delivery(
            &key,
            &SendDelivery {
                delivery_id: "delivery-notify".to_string(),
                recipient_instance_id: "instance-1".to_string(),
                delivery_generation: 1,
                expires_at: 1_000_000 + 3600,
                message_bytes: serde_json::to_vec(&message).unwrap(),
            },
        )
        .unwrap();

    let _ = state.agent_infos();

    peer.set_nonblocking(true).unwrap();
    let mut buf = [0u8; 4096];
    assert!(
        matches!(peer.read(&mut buf), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
    );
    assert_eq!(
        state
            .idempotency
            .dispatching_deliveries_for_instance("instance-1", 1_000_000)
            .unwrap()
            .len(),
        1
    );
    assert!(
        state
            .idempotency
            .pending_orphan_emitter_notices()
            .unwrap()
            .is_empty(),
        "aucune notice orpheline créée pour une simple absence de route"
    );
    let _ = std::fs::remove_file(config.db_path);
}

/// ORACLE — prouve que `flush_pending_orphan_emitter_notices` pousse une
/// notice en attente dès que l'émetteur a une connexion lisible.
///
/// **Ce qu'il ne prouve PAS** : que `handle_connection` appelle ce flush
/// après `Registered`. Ce branchement vit dans la boucle socket ; aucun
/// test unitaire ne l'atteint. Retirer `if registered_just_now { … }`
/// laisse cet oracle vert — trou déclaré, pas maquillé. Un témoin d'intégration
/// (vrai Register sur socket) fermerait le trou ; hors périmètre immédiat.
#[test]
fn register_emetteur_rejoue_les_notices_orphelines_en_attente() {
    use crate::idempotency::{IdempotencyKey, OperationKind, Reservation, SendDelivery};
    use std::io::Read;
    use std::os::unix::net::UnixStream;

    let (mut state, config) = state_with_registered_agent("purge-register-replay");
    // Remise dispatching vers le destinataire qui va être purgé.
    let key = IdempotencyKey::new(
        "012_scope_aaaaaaaaaaaa",
        OperationKind::Send,
        "msg-register-replay",
    )
    .unwrap();
    assert!(matches!(
        state
            .idempotency
            .reserve(&key, b"canon-reg", 1_000_000, 3600, 1_000_000, 30),
        Ok(Reservation::Prepared { .. })
    ));
    let mut message = BridgetMessage::new(
        "89000000-0000-4000-8000-000000000123",
        "89000000-0000-4000-8000-000000000102",
        "corps",
    );
    message.id = "msg-register-replay".to_string();
    state
        .idempotency
        .begin_send_delivery(
            &key,
            &SendDelivery {
                delivery_id: "delivery-register-replay".to_string(),
                recipient_instance_id: "instance-1".to_string(),
                delivery_generation: 1,
                expires_at: 1_000_000 + 3600,
                message_bytes: serde_json::to_vec(&message).unwrap(),
            },
        )
        .unwrap();

    // Purge sans émetteur en ligne → notice durable, Deliver impossible.
    state.set_turn_state("conn-1", true).unwrap();
    let stale = Instant::now()
        .checked_sub(PRESENCE_RETENTION + Duration::from_secs(1))
        .expect("horloge");
    let presence = state.presences.get_mut("instance-1").unwrap();
    presence.capacity_seen = stale;
    presence.link_seen = stale;
    let _ = state.agent_infos();
    assert_eq!(
        state
            .idempotency
            .pending_orphan_emitter_notices()
            .unwrap()
            .len(),
        1,
        "émetteur absent ⇒ notice reste en attente"
    );

    // L'émetteur revient en ligne (présence + socket). On appelle le flush
    // directement — prouve la fonction, pas le branchement Register
    // (voir doc de l'oracle : trou déclaré).
    state
        .router
        .register(
            "89000000-0000-4000-8000-000000000123",
            &bridget_core::AgentType::Claude,
            "conn-emitter",
        )
        .unwrap();
    state
        .conn_instances
        .insert("conn-emitter".to_string(), "instance-emitter".to_string());
    state.presences.insert(
        "instance-emitter".to_string(),
        Presence {
            name: "89000000-0000-4000-8000-000000000123".to_string(),
            agent_type: "claude".to_string(),
            host: "macbook".to_string(),
            transport: "acp".to_string(),
            channel: None,
            mode: Some(PresenceMode::Acp),
            location: None,
            journal_available: true,
            os: "macOS".to_string(),
            state: "connected".to_string(),
            busy_since: None,
            busy_period_start: None,
            last_attested_at: None,
            turn_attest: false,
            queue_fallback: false,
            capacity_seen: Instant::now(),
            link_seen: Instant::now(),
            reconnect_count: 0,
            model: None,
            effort: None,
            rate_limits: Default::default(),
            served_model: None,
            derived_domain: None,
            domain: None,
            dnd_until: None,
            disk_space: None,
        },
    );
    let (writer_stream, mut peer) = UnixStream::pair().unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    state.connections.insert(
        "conn-emitter".to_string(),
        Arc::new(Mutex::new(BufWriter::new(writer_stream))),
    );

    state.flush_pending_orphan_emitter_notices();

    let mut buf = [0u8; 4096];
    let n = peer
        .read(&mut buf)
        .expect("Deliver ORPHELIN attendu au retour de l'émetteur");
    let received = String::from_utf8_lossy(&buf[..n]);
    assert!(
        received.contains("ORPHELIN") && received.contains("msg-register-replay"),
        "Register émetteur doit rejouer la notice: {received}"
    );
    assert!(
        received.contains("clé est close"),
        "rejeu Register porte aussi la conduite: {received}"
    );
    assert!(
        state
            .idempotency
            .pending_orphan_emitter_notices()
            .unwrap()
            .is_empty()
    );
    let _ = std::fs::remove_file(config.db_path);
}

/// ORACLE — mort du wrapper (unregister + unreachable) → zéro présence live.
#[test]
fn mort_du_wrapper_emporte_la_presence_du_routeur() {
    let (mut state, config) = state_with_registered_agent("mort-wrapper");
    assert!(
        state
            .router
            .get_agent("89000000-0000-4000-8000-000000000102")
            .is_some()
    );
    state.router.unregister_by_conn("conn-1");
    state.mark_unreachable("conn-1");

    let infos = state.agent_infos();
    assert!(
        state
            .router
            .get_agent("89000000-0000-4000-8000-000000000102")
            .is_none()
    );
    assert!(
        !infos.iter().any(
            |agent| agent.agent_id == "89000000-0000-4000-8000-000000000102"
                && agent.state == "connected"
        ),
        "pas de connected résiduel: {infos:?}"
    );
    let unreachable = infos
        .iter()
        .find(|agent| agent.agent_id == "89000000-0000-4000-8000-000000000102")
        .expect("l'état unreachable reste listé distinctement");
    assert_eq!(unreachable.state, "unreachable");
    let _ = std::fs::remove_file(config.db_path);
}

/// ORACLE — takeover sur fantôme sans stop préalable.
#[test]
fn takeover_sur_fantome_reussit_sans_stop_prealable() {
    let (mut state, config) = state_with_registered_agent("takeover-fantome");
    // Simule le fantôme : présence expirée, nom encore au routeur avant
    // le premier agent_infos (ou un résidu après crash de retain).
    state.presences.clear();
    // conn_instances pointe vers une instance absente → dangling.
    assert!(
        state
            .router
            .get_agent("89000000-0000-4000-8000-000000000102")
            .is_some()
    );

    let registered = handle_register(
        "conn-takeover",
        "cursor".to_string(),
        Some("89000000-0000-4000-8000-000000000102".to_string()),
        Some("local".to_string()),
        Some("acp".to_string()),
        Some(PresenceMode::Acp),
        None,
        Some("macOS".to_string()),
        Some("instance-takeover".to_string()),
        Some("bridget".to_string()),
        false,
        Some(true),
        &mut state,
    );
    assert!(
        matches!(
            registered,
            DaemonToWrapper::Registered { credential: _, agent_id: ref name } if name == "89000000-0000-4000-8000-000000000102"
        ),
        "takeover refusé: {registered:?}"
    );
    let infos = state.agent_infos();
    let info = infos
        .iter()
        .find(|agent| agent.agent_id == "89000000-0000-4000-8000-000000000102")
        .expect("agent repris");
    assert_eq!(info.state, "connected");
    assert_eq!(info.transport, "acp");
    assert_eq!(info.mode, Some(PresenceMode::Acp));
    let _ = std::fs::remove_file(config.db_path);
}

/// Contre-épreuve : un agent vraiment live refuse le reclaim.
#[test]
fn takeover_refuse_un_nom_encore_live() {
    let (mut state, config) = state_with_registered_agent("takeover-live");
    let refused = handle_register(
        "conn-intrus",
        "cursor".to_string(),
        Some("89000000-0000-4000-8000-000000000102".to_string()),
        Some("local".to_string()),
        Some("acp".to_string()),
        Some(PresenceMode::Acp),
        None,
        Some("macOS".to_string()),
        Some("instance-intrus".to_string()),
        None,
        false,
        Some(true),
        &mut state,
    );
    assert!(
        matches!(refused, DaemonToWrapper::Nack { .. }),
        "un live ne doit pas être spolié: {refused:?}"
    );
    let _ = std::fs::remove_file(config.db_path);
}

/// ORACLE — Heartbeat rafraîchit le LIEN, jamais la capacité ni l'état.
///
/// Sans la séparation : un wrapper vivant / shell mort garde `capacity_seen`
/// frais et who / bridget-idle le comptent LIBRE. busy doit rester busy ;
/// `capacity_seen` (capacité) ne bouge pas sous heartbeat seul.
#[test]
fn spec240_late_prepare_expiry_preserves_worker_without_creating_a_task() {
    use bridget_transport::protocol::*;
    for stale_before_reply in [false, true] {
        let (mut st, config) = state_with_registered_agent("worker240-late-prepare");
        let worker = WorkerIdentity {
            machine_id: "cartae".into(),
            worker_id: "worker-cartae".into(),
            provisioning_revision: 1,
        };
        st.remote_workers.insert(
            "cartae".into(),
            RemoteWorkerConnection {
                identity: worker.clone(),
                conn_id: "worker".into(),
                epoch: "epoch".into(),
                capabilities: vec![WorkerCapability::WorkspaceFailureV1],
                catalogues: HashMap::new(),
                catalogue_refresh_id: "epoch".into(),
                catalogue_refresh_at: None,
                last_contact: Instant::now(),
                last_contact_at: unix_timestamp(),
                contact: WorkerContactState::Connected,
            },
        );
        let (writer, mut reader) = control_socket("late240");
        reader
            .get_ref()
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        st.connections.insert("worker".into(), writer);
        st.registry = AgentRegistry::from_json(&serde_json::json!({"machines":{"cartae":{
            "enabled":true,"worker_id":"worker-cartae","provisioning_revision":1,
            "credential_fingerprint":"a".repeat(64),"allowed_agent_types":["codex"],"mappings":{}
        }}}).to_string(), "/tmp/worker240-registry.json").unwrap();
        let shared = Arc::new(Mutex::new(st));
        let command = WorkerCommandIdentity {
            command_id: "prepare240".into(),
            task_id: "prepare240".into(),
            instance_id: String::new(),
            generation: 0,
        };
        let parent: ProviderPermissions = serde_json::from_value(serde_json::json!({"version":1,"source":"provider_turn",
            "run_id":"run240","provider_session_id":"session240","provider_instance_id":"codex","revision":1,
            "driver":"codex_app_server","cwd":"/source","runtime_mode":"full-access","interaction_mode":"default",
            "provider_policy":{"kind":"codex","approval_policy":"never","approvals_reviewer":"user","sandbox_policy":{"type":"dangerFullAccess"}}})).unwrap();
        bridget_transport::protocol::validate_permissions(&parent, true)
            .expect("preuve complète de la fixture, sans contourner le validateur réel");
        let prepare = WorkerPrepare {
            machine_id: "cartae".into(),
            project_id: "project".into(),
            provisioning_revision: 1,
            mapping_revision: 1,
            origin_host: "mac".into(),
            origin_cwd: "/source".into(),
            source_root: "/source".into(),
            remote_root: "/remote".into(),
            source_root_identity: "/repo.git".into(),
            relative_cwd: ".".into(),
            base_commit: None,
            agent_type: "codex".into(),
            model: "model".into(),
            effort: None,
            service_tier: None,
            posture: Some(SpawnPosture::Discovery),
            parent,
            workspace: None,
        };
        let operation = WorkerRequestOperation::PrepareDiagnosed {
            prepare: Box::new(WorkerRequestOperation::Prepare { prepare }),
        };
        assert!(late_worker_prepares::initial_prepare(&command, &operation));
        assert!(!late_worker_prepares::initial_prepare(
            &command,
            &WorkerRequestOperation::Cancel {
                target: WorkerCancelScope::Command {}
            }
        ));
        let (allow_late, wait_late) = mpsc::sync_channel(1);
        let responder = shared.clone();
        let server = std::thread::spawn(move || {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let DaemonToWrapper::WorkerRequest { request } = decode(line.trim()).unwrap() else {
                panic!("prepare attendu")
            };
            if stale_before_reply {
                remote_worker_disconnected(&mut responder.lock().unwrap(), "worker");
                assert!(
                    responder
                        .lock()
                        .unwrap()
                        .pending_worker_responses
                        .is_empty(),
                    "le tick de contact a retiré l'attente avant le retour du roundtrip"
                );
            }
            wait_late.recv().unwrap();
            let response = WorkerResponse {
                version: REMOTE_WORKER_VERSION,
                correlation_id: request.correlation_id,
                connection_epoch: request.connection_epoch,
                worker: request.worker,
                command: request.command,
                response: WorkerResponseOperation::Rejected {
                    reason: WorkerRefusal::PrepareExpired,
                },
            };
            assert!(
                handle_worker_message(
                    &responder,
                    "worker",
                    WrapperToDaemon::WorkerResponse {
                        response: response.clone()
                    }
                )
                .is_none()
            );
            response
        });
        let outcome = remote_worker_roundtrip(
            &shared,
            &worker,
            &command,
            operation,
            Instant::now() + Duration::from_millis(30),
        );
        assert_eq!(outcome.unwrap_err(), WorkerRefusal::WorkerUnavailable);
        assert!(shared.lock().unwrap().pending_worker_responses.is_empty());
        allow_late.send(()).unwrap();
        let response = server.join().unwrap();
        assert!(
            shared
                .lock()
                .unwrap()
                .delegation_store
                .get("prepare240")
                .unwrap()
                .is_none()
        );
        remote_worker_disconnected(&mut shared.lock().unwrap(), "worker");
        assert!(
            handle_worker_message(
                &shared,
                "worker",
                WrapperToDaemon::WorkerResponse {
                    response: response.clone()
                }
            )
            .is_none()
        );
        assert_eq!(
            shared.lock().unwrap().remote_workers["cartae"].contact,
            WorkerContactState::Stale,
            "une réponse périmée ignorée ne rafraîchit pas le contact"
        );
        let mut forged = response.clone();
        forged.command.task_id = "other".into();
        assert!(matches!(
            handle_worker_message(
                &shared,
                "worker",
                WrapperToDaemon::WorkerResponse { response: forged }
            ),
            Some(DaemonToWrapper::WorkerRejected {
                reason: WorkerRefusal::CommandMismatch
            })
        ));
        let mut stale = response;
        stale.connection_epoch = "old".into();
        assert!(matches!(
            handle_worker_message(
                &shared,
                "worker",
                WrapperToDaemon::WorkerResponse { response: stale }
            ),
            Some(DaemonToWrapper::WorkerRejected {
                reason: WorkerRefusal::StaleConnection
            })
        ));
        assert!(handle_worker_message(&shared, "worker", WrapperToDaemon::Heartbeat).is_none());
        let _ = std::fs::remove_file(config.db_path);
    }
}

#[test]
fn spec184_late_release_reply_preserves_worker_connection() {
    use bridget_transport::protocol::*;
    for cancelled in [false, true] {
        let (mut st, config) = state_with_registered_agent("worker184-late-release");
        let worker = WorkerIdentity {
            machine_id: "cartae".into(),
            worker_id: "worker-cartae".into(),
            provisioning_revision: 1,
        };
        let placement = RemotePlacement {
            machine_id: worker.machine_id.clone(),
            worker_id: worker.worker_id.clone(),
            provisioning_revision: 1,
            project_id: "project".into(),
            mapping_revision: 1,
            origin_host: "mac".into(),
            origin_cwd: "/source".into(),
            source_root_identity: "/repo.git".into(),
            relative_cwd: ".".into(),
            execution_cwd: "/remote".into(),
            base_commit: None,
            definition_digest: "a".repeat(64),
            permission_digest: "b".repeat(64),
        };
        let now = unix_timestamp();
        let order = FleetSpawnOrder {
            remote_placement: Some(placement),
            posture: None,
            agent_type: "codex".into(),
            project: None,
            requested_name: Some(Uuid::new_v4().to_string()),
            cwd: PathBuf::from("/remote"),
            persistent: true,
            command_id: Uuid::new_v4().to_string(),
            issued_at: now,
            deadline_at: now + 300,
            ownership: None,
        };
        let crate::fleet::SpawnSubmission::Start(lease) =
            st.fleet.request_spawn(&order, now).unwrap()
        else {
            panic!("start expected")
        };
        st.remote_workers.insert(
            "cartae".into(),
            RemoteWorkerConnection {
                identity: worker.clone(),
                conn_id: "worker".into(),
                epoch: "epoch".into(),
                capabilities: vec![],
                catalogues: HashMap::new(),
                catalogue_refresh_id: "epoch".into(),
                catalogue_refresh_at: None,
                last_contact: Instant::now(),
                last_contact_at: now,
                contact: WorkerContactState::Connected,
            },
        );
        let (writer, mut reader) = control_socket("late184");
        reader
            .get_ref()
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        st.connections.insert("worker".into(), writer);
        st.registry = AgentRegistry::from_json(&serde_json::json!({"machines":{"cartae":{
            "enabled":true,"worker_id":"worker-cartae","provisioning_revision":1,
            "credential_fingerprint":"a".repeat(64),"allowed_agent_types":["codex"],"mappings":{}
        }}}).to_string(), "/tmp/worker184-registry.json").unwrap();
        let shared = Arc::new(Mutex::new(st));
        let command = WorkerCommandIdentity {
            command_id: lease.command_id.clone(),
            task_id: order.command_id.clone(),
            instance_id: lease.instance_id.clone(),
            generation: lease.generation,
        };
        let (allow_late, wait_late) = mpsc::sync_channel(1);
        let responder = shared.clone();
        let server = std::thread::spawn(move || {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let DaemonToWrapper::WorkerRequest { request } = decode(line.trim()).unwrap() else {
                panic!("requête Release attendue");
            };
            assert!(matches!(
                request.request,
                WorkerRequestOperation::Release { .. }
            ));
            wait_late.recv_timeout(Duration::from_secs(2)).unwrap();
            let response = WorkerResponse {
                version: REMOTE_WORKER_VERSION,
                correlation_id: request.correlation_id,
                connection_epoch: request.connection_epoch,
                worker: request.worker,
                command: request.command,
                response: WorkerResponseOperation::Released {},
            };
            assert!(
                handle_worker_message(
                    &responder,
                    "worker",
                    WrapperToDaemon::WorkerResponse {
                        response: response.clone()
                    }
                )
                .is_none(),
                "un accusé Release tardif connu ne doit pas couper le worker"
            );
            response
        });
        let permit = WorkerReleasePermit {
            permit_id: Uuid::new_v4().to_string(),
            command_digest: "d".repeat(64),
            permission_revision: 1,
            mapping_revision: 1,
            provisioning_revision: 1,
            issued_at: now,
            expires_at: now + WORKER_RELEASE_LEASE_SECS,
        };
        let outcome = remote_worker_roundtrip(
            &shared,
            &worker,
            &command,
            WorkerRequestOperation::Release { permit },
            Instant::now() + Duration::from_millis(20),
        );
        assert!(
            matches!(outcome, Err(WorkerRefusal::WorkerUnavailable)),
            "{outcome:?}"
        );
        assert!(
            shared.lock().unwrap().pending_worker_responses.is_empty(),
            "la réponse doit arriver après retrait réel de l'attente"
        );
        if cancelled {
            shared
                .lock()
                .unwrap()
                .fleet
                .invalidate_for_stop(&lease)
                .unwrap();
        }
        let before = shared
            .lock()
            .unwrap()
            .fleet
            .desired_entry(&lease.name)
            .unwrap();
        allow_late.send(()).unwrap();
        let response = server.join().unwrap();
        assert_eq!(
            shared
                .lock()
                .unwrap()
                .fleet
                .desired_entry(&lease.name)
                .unwrap(),
            before,
            "l'accusé tardif ne doit pas réactiver une commande annulée"
        );
        assert!(shared.lock().unwrap().pending_worker_responses.is_empty());
        let mut foreign = response.clone();
        foreign.command.instance_id = Uuid::new_v4().to_string();
        assert!(matches!(
            handle_worker_message(
                &shared,
                "worker",
                WrapperToDaemon::WorkerResponse { response: foreign }
            ),
            Some(DaemonToWrapper::WorkerRejected {
                reason: WorkerRefusal::CommandMismatch
            })
        ));
        let mut stale = response;
        stale.connection_epoch = "old-epoch".into();
        assert!(matches!(
            handle_worker_message(
                &shared,
                "worker",
                WrapperToDaemon::WorkerResponse { response: stale }
            ),
            Some(DaemonToWrapper::WorkerRejected {
                reason: WorkerRefusal::StaleConnection
            })
        ));
        assert!(handle_worker_message(&shared, "worker", WrapperToDaemon::Heartbeat).is_none());
        let _ = std::fs::remove_file(config.db_path);
    }
}

#[test]
fn spec172_untracked_worker_refusal_keeps_known_command_channel() {
    use bridget_transport::protocol::*;
    let (mut st, config) = state_with_registered_agent("worker172-untracked-refusal");
    let worker = WorkerIdentity {
        machine_id: "cartae".into(),
        worker_id: "worker-cartae".into(),
        provisioning_revision: 1,
    };
    let placement = RemotePlacement {
        machine_id: worker.machine_id.clone(),
        worker_id: worker.worker_id.clone(),
        provisioning_revision: 1,
        project_id: "project".into(),
        mapping_revision: 1,
        origin_host: "mac".into(),
        origin_cwd: "/source".into(),
        source_root_identity: "/repo.git".into(),
        relative_cwd: ".".into(),
        execution_cwd: "/remote".into(),
        base_commit: None,
        definition_digest: "a".repeat(64),
        permission_digest: "b".repeat(64),
    };
    let now = unix_timestamp();
    let order = FleetSpawnOrder {
        remote_placement: Some(placement),
        posture: None,
        agent_type: "codex".into(),
        project: None,
        requested_name: Some(Uuid::new_v4().to_string()),
        cwd: PathBuf::from("/remote"),
        persistent: true,
        command_id: Uuid::new_v4().to_string(),
        issued_at: now,
        deadline_at: now + 300,
        ownership: None,
    };
    let crate::fleet::SpawnSubmission::Start(lease) = st.fleet.request_spawn(&order, now).unwrap()
    else {
        panic!("start expected")
    };
    st.remote_workers.insert(
        "cartae".into(),
        RemoteWorkerConnection {
            identity: worker.clone(),
            conn_id: "worker".into(),
            epoch: "epoch".into(),
            capabilities: vec![],
            catalogues: HashMap::new(),
            catalogue_refresh_id: "epoch".into(),
            catalogue_refresh_at: None,
            last_contact: Instant::now(),
            last_contact_at: now,
            contact: WorkerContactState::Connected,
        },
    );
    let shared = Arc::new(Mutex::new(st));
    let command = WorkerCommandIdentity {
        command_id: lease.command_id,
        task_id: order.command_id,
        instance_id: lease.instance_id,
        generation: lease.generation,
    };
    for reason in [
        WorkerRefusal::UnknownExecution,
        WorkerRefusal::ResourceLimit,
    ] {
        let response = WorkerResponse {
            version: REMOTE_WORKER_VERSION,
            correlation_id: Uuid::new_v4().to_string(),
            connection_epoch: "epoch".into(),
            worker: worker.clone(),
            command: command.clone(),
            response: WorkerResponseOperation::Rejected { reason },
        };
        assert!(
            handle_worker_message(
                &shared,
                "worker",
                WrapperToDaemon::WorkerResponse {
                    response: response.clone()
                }
            )
            .is_none(),
            "a refusal to an untracked status request must not close the worker channel"
        );
        let (tx, rx) = mpsc::sync_channel(1);
        shared.lock().unwrap().pending_worker_responses.insert(
            response.correlation_id.clone(),
            PendingWorkerResponse {
                conn_id: "worker".into(),
                epoch: "epoch".into(),
                worker: worker.clone(),
                command: command.clone(),
                reply: tx,
            },
        );
        assert!(
            handle_worker_message(
                &shared,
                "worker",
                WrapperToDaemon::WorkerResponse {
                    response: response.clone()
                }
            )
            .is_none()
        );
        assert!(
            matches!(rx.try_recv().unwrap(), WorkerResponseOperation::Rejected { reason: actual } if actual == reason),
            "a tracked refusal must still reach its caller"
        );
        let mut foreign = response.clone();
        foreign.command.instance_id = Uuid::new_v4().to_string();
        assert!(matches!(
            handle_worker_message(
                &shared,
                "worker",
                WrapperToDaemon::WorkerResponse { response: foreign }
            ),
            Some(DaemonToWrapper::WorkerRejected {
                reason: WorkerRefusal::CommandMismatch
            })
        ));
        let mut stale = response;
        stale.connection_epoch = "old-epoch".into();
        assert!(matches!(
            handle_worker_message(
                &shared,
                "worker",
                WrapperToDaemon::WorkerResponse { response: stale }
            ),
            Some(DaemonToWrapper::WorkerRejected {
                reason: WorkerRefusal::StaleConnection
            })
        ));
    }
    assert!(handle_worker_message(&shared, "worker", WrapperToDaemon::Heartbeat).is_none());
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn spec237_managed_observation_without_delegation_keeps_channel() {
    use bridget_transport::protocol::*;
    let (mut st, config) = state_with_registered_agent("worker237-managed-observation");
    let worker = WorkerIdentity {
        machine_id: "cartae".into(),
        worker_id: "worker-cartae".into(),
        provisioning_revision: 1,
    };
    let placement = RemotePlacement {
        machine_id: worker.machine_id.clone(),
        worker_id: worker.worker_id.clone(),
        provisioning_revision: 1,
        project_id: "project".into(),
        mapping_revision: 1,
        origin_host: "mac".into(),
        origin_cwd: "/source".into(),
        source_root_identity: "/repo.git".into(),
        relative_cwd: ".".into(),
        execution_cwd: "/remote".into(),
        base_commit: None,
        definition_digest: "a".repeat(64),
        permission_digest: "b".repeat(64),
    };
    let now = unix_timestamp();
    let order = FleetSpawnOrder {
        remote_placement: Some(placement),
        posture: None,
        agent_type: "codex".into(),
        project: None,
        requested_name: Some(Uuid::new_v4().to_string()),
        cwd: PathBuf::from("/remote"),
        persistent: true,
        command_id: Uuid::new_v4().to_string(),
        issued_at: now,
        deadline_at: now + 300,
        ownership: None,
    };
    let crate::fleet::SpawnSubmission::Start(lease) = st.fleet.request_spawn(&order, now).unwrap()
    else {
        panic!("start expected")
    };
    st.remote_workers.insert(
        "cartae".into(),
        RemoteWorkerConnection {
            identity: worker.clone(),
            conn_id: "worker".into(),
            epoch: "epoch".into(),
            capabilities: vec![],
            catalogues: HashMap::new(),
            catalogue_refresh_id: "epoch".into(),
            catalogue_refresh_at: None,
            last_contact: Instant::now(),
            last_contact_at: now,
            contact: WorkerContactState::Connected,
        },
    );
    let shared = Arc::new(Mutex::new(st));
    let command = WorkerCommandIdentity {
        command_id: lease.command_id,
        task_id: order.command_id,
        instance_id: lease.instance_id,
        generation: lease.generation,
    };
    for phase in [
        WorkerCommandPhase::Released,
        WorkerCommandPhase::Terminal,
        WorkerCommandPhase::Refused,
    ] {
        let response = WorkerResponse {
            version: REMOTE_WORKER_VERSION,
            correlation_id: Uuid::new_v4().to_string(),
            connection_epoch: "epoch".into(),
            worker: worker.clone(),
            command: command.clone(),
            response: WorkerResponseOperation::Observation {
                observation: WorkerObservation {
                    phase,
                    native_ready: false,
                    exit_code: None,
                    observed_at: now,
                    workspace: None,
                    permission_digest: None,
                    startup_failure: None,
                },
            },
        };
        assert!(
            handle_worker_message(
                &shared,
                "worker",
                WrapperToDaemon::WorkerResponse {
                    response: response.clone()
                }
            )
            .is_none(),
            "une observation gérée connue ne doit pas exiger une délégation"
        );
        let (tx, rx) = mpsc::sync_channel(1);
        shared.lock().unwrap().pending_worker_responses.insert(
            response.correlation_id.clone(),
            PendingWorkerResponse {
                conn_id: "worker".into(),
                epoch: "epoch".into(),
                worker: worker.clone(),
                command: command.clone(),
                reply: tx,
            },
        );
        assert!(
            handle_worker_message(
                &shared,
                "worker",
                WrapperToDaemon::WorkerResponse {
                    response: response.clone()
                }
            )
            .is_none()
        );
        assert!(
            matches!(rx.try_recv().unwrap(), WorkerResponseOperation::Observation { observation } if observation.phase == phase),
            "la réponse corrélée doit encore parvenir à son lecteur"
        );
        assert!(
            shared
                .lock()
                .unwrap()
                .delegation_store
                .tasks()
                .unwrap()
                .is_empty()
        );
        for field in ["generation", "task_id", "command_id"] {
            let mut bad = response.clone();
            match field {
                "generation" => bad.command.generation += 1,
                "task_id" => bad.command.task_id = Uuid::new_v4().to_string(),
                _ => bad.command.command_id = Uuid::new_v4().to_string(),
            }
            assert!(matches!(
                handle_worker_message(
                    &shared,
                    "worker",
                    WrapperToDaemon::WorkerResponse { response: bad }
                ),
                Some(DaemonToWrapper::WorkerRejected {
                    reason: WorkerRefusal::CommandMismatch
                })
            ));
        }
        let mut foreign = response.clone();
        foreign.command.instance_id = Uuid::new_v4().to_string();
        assert!(matches!(
            handle_worker_message(
                &shared,
                "worker",
                WrapperToDaemon::WorkerResponse { response: foreign }
            ),
            Some(DaemonToWrapper::WorkerRejected {
                reason: WorkerRefusal::CommandMismatch
            })
        ));
        let mut stale = response;
        stale.connection_epoch = "old-epoch".into();
        assert!(matches!(
            handle_worker_message(
                &shared,
                "worker",
                WrapperToDaemon::WorkerResponse { response: stale }
            ),
            Some(DaemonToWrapper::WorkerRejected {
                reason: WorkerRefusal::StaleConnection
            })
        ));
    }
    assert!(handle_worker_message(&shared, "worker", WrapperToDaemon::Heartbeat).is_none());
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn remote162_worker_auth_epoch_et_contact_sans_presence_fournisseur() {
    use bridget_transport::protocol::*;
    use sha2::{Digest, Sha256};
    let (mut state, config) = state_with_registered_agent("worker162-auth");
    let fingerprint = format!("{:x}", Sha256::digest(b"worker-secret"));
    state.registry = AgentRegistry::from_json(
        &serde_json::json!({"machines":{"cartae":{
            "enabled":true,"worker_id":"worker-cartae","provisioning_revision":1,
            "credential_fingerprint":fingerprint,"allowed_agent_types":["codex"],"mappings":{}
        }}})
        .to_string(),
        "/tmp/worker162-agents.json",
    )
    .unwrap();
    for conn in ["worker-a", "worker-b"] {
        state
            .connection_roles
            .insert(conn.into(), ConnectionRole::Worker);
        state
            .peer_uids
            .insert(conn.into(), bridget_transport::fsutil::effective_uid());
    }
    let shared = Arc::new(Mutex::new(state));
    let mut hello = WorkerHello {
        version: 1,
        identity: WorkerIdentity {
            machine_id: "cartae".into(),
            worker_id: "worker-cartae".into(),
            provisioning_revision: 1,
        },
        credential: IdentityCredential::new("bad-secret".into()),
        build: "test".into(),
        capability: ClientCapability::RemoteWorkerV1,
    };
    assert!(matches!(
        handle_worker_message(
            &shared,
            "worker-a",
            WrapperToDaemon::WorkerHello {
                hello: hello.clone()
            }
        ),
        Some(DaemonToWrapper::WorkerRejected {
            reason: WorkerRefusal::Unauthorized
        })
    ));
    hello.credential = IdentityCredential::new("worker-secret".into());
    let Some(DaemonToWrapper::WorkerAccepted { accepted: first }) = handle_worker_message(
        &shared,
        "worker-a",
        WrapperToDaemon::WorkerHello {
            hello: hello.clone(),
        },
    ) else {
        panic!("first hello refused")
    };
    let Some(DaemonToWrapper::WorkerAccepted { accepted: second }) =
        handle_worker_message(&shared, "worker-b", WrapperToDaemon::WorkerHello { hello })
    else {
        panic!("replacement refused")
    };
    assert_ne!(first.connection_epoch, second.connection_epoch);
    assert!(matches!(
        handle_worker_message(&shared, "worker-a", WrapperToDaemon::Heartbeat),
        Some(DaemonToWrapper::WorkerRejected { .. })
    ));
    assert!(handle_worker_message(&shared, "worker-b", WrapperToDaemon::Heartbeat).is_none());
    {
        let mut st = shared.lock().unwrap();
        assert!(!st.conn_names.contains_key("worker-b"));
        st.remote_workers.get_mut("cartae").unwrap().last_contact =
            Instant::now() - Duration::from_secs(31);
    }
    let (reply172, receiver172) = mpsc::sync_channel(1);
    shared.lock().unwrap().pending_candidate_probes_v2.insert(
        "probe172-disconnected".into(),
        PendingCandidateProbeV2 {
            conn_id: "worker-b".into(),
            epoch: second.connection_epoch.clone(),
            worker: second.identity.clone(),
            admission_id: "a-probe172".into(),
            policy_revision: 1,
            candidate_ids: std::collections::BTreeSet::from(["c001".into()]),
            reply: reply172,
        },
    );
    remote_worker_contact_tick(&shared);
    assert_eq!(
        shared.lock().unwrap().remote_workers["cartae"].contact,
        WorkerContactState::Stale
    );
    assert!(
        matches!(
            receiver172.try_recv(),
            Err(mpsc::TryRecvError::Disconnected)
        ),
        "une sonde V2 doit finir dès que le worker est déconnecté"
    );
    assert!(
        shared
            .lock()
            .unwrap()
            .pending_candidate_probes_v2
            .is_empty()
    );
    handle_worker_message(&shared, "worker-b", WrapperToDaemon::Heartbeat);
    assert_eq!(
        shared.lock().unwrap().remote_workers["cartae"].contact,
        WorkerContactState::Connected
    );
    let hello = WorkerHello {
        version: 1,
        identity: second.identity,
        credential: IdentityCredential::new("worker-secret".into()),
        build: "test".into(),
        capability: ClientCapability::RemoteWorkerV1,
    };
    shared.lock().unwrap().peer_uids.insert(
        "worker-probe".into(),
        bridget_transport::fsutil::effective_uid(),
    );
    assert!(matches!(
        handle_wrapper_message(
            "worker-probe",
            WrapperToDaemon::DaemonIdentityRequest,
            &shared
        ),
        Some(DaemonToWrapper::DaemonIdentityReport { .. })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "worker-probe",
            WrapperToDaemon::WorkerHello {
                hello: hello.clone()
            },
            &shared
        ),
        Some(DaemonToWrapper::WorkerAccepted { .. })
    ));
    assert!(
        shared.lock().unwrap().remote_workers["cartae"]
            .capabilities
            .is_empty()
    );
    let deadline_command = WorkerCommandIdentity {
        command_id: "deadline-command".into(),
        task_id: "deadline-task".into(),
        instance_id: "deadline-instance".into(),
        generation: 1,
    };
    let deadline_slot = WorkerResultSlot {
        identity: WorkerResultIdentity {
            command: deadline_command.clone(),
            mission_id: "deadline-mission".into(),
            round: 1,
            execution_id: "deadline-execution".into(),
            execution_generation: 1,
        },
        admission_digest: "c".repeat(64),
    };
    assert_eq!(
        remote_worker_roundtrip(
            &shared,
            &hello.identity,
            &deadline_command,
            WorkerRequestOperation::ReserveResultSlotUntil {
                slot: deadline_slot,
                expires_at_ms: 1234
            },
            Instant::now() + Duration::from_secs(1)
        ),
        Err(WorkerRefusal::CapabilityRequired),
        "aucune nouvelle opération ne part vers un worker sans capacité négociée"
    );

    shared.lock().unwrap().peer_uids.insert(
        "worker-lease".into(),
        bridget_transport::fsutil::effective_uid(),
    );
    let capabilities = vec![
        WorkerCapability::CandidateProbeV1,
        WorkerCapability::CandidateProbeV2,
        WorkerCapability::ReleaseLeaseV2,
        WorkerCapability::ParentSourceProofV1,
        WorkerCapability::ResultSlotDeadlineV1,
    ];
    assert!(matches!(
        handle_worker_message(&shared, "worker-lease",
            WrapperToDaemon::WorkerHelloWithCapabilities {
                hello: WorkerHelloWithCapabilities { hello: hello.clone(), capabilities: capabilities.clone() }
            }),
        Some(DaemonToWrapper::WorkerAcceptedWithCapabilities { accepted })
            if accepted.capabilities == capabilities
    ));
    assert!(matches!(
        handle_worker_message(
            &shared,
            "worker-lease",
            WrapperToDaemon::WorkerHelloWithCapabilities {
                hello: WorkerHelloWithCapabilities {
                    hello: hello.clone(),
                    capabilities: vec![
                        WorkerCapability::ReleaseLeaseV2,
                        WorkerCapability::ReleaseLeaseV2
                    ]
                }
            }
        ),
        Some(DaemonToWrapper::WorkerRejected { .. })
    ));
    assert!(
        !matches!(
            handle_wrapper_message("conn-1", WrapperToDaemon::WorkerHello { hello }, &shared),
            Some(DaemonToWrapper::WorkerAccepted { .. })
        ),
        "une identité wrapper ne devient pas worker"
    );
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn heartbeat_preserve_etat_metier() {
    let (mut state, config) = state_with_registered_agent("heartbeat-busy");
    state.set_turn_state("conn-1", true).unwrap();
    assert_eq!(state.presences.get("instance-1").unwrap().state, "busy");
    let before_capacity = state.presences.get("instance-1").unwrap().capacity_seen;
    let before_link = state.presences.get("instance-1").unwrap().link_seen;
    std::thread::sleep(Duration::from_millis(5));
    let shared = Arc::new(Mutex::new(state));
    assert!(handle_wrapper_message("conn-1", WrapperToDaemon::Heartbeat, &shared).is_none());
    let st = shared.lock().unwrap();
    let presence = st.presences.get("instance-1").unwrap();
    assert_eq!(
        presence.state, "busy",
        "Heartbeat ne doit jamais écraser l'état métier (busy→connected serait un mensonge)"
    );
    assert_eq!(
        presence.capacity_seen, before_capacity,
        "Heartbeat ne doit pas rafraîchir la capacité (capacity_seen)"
    );
    assert!(
        presence.link_seen > before_link,
        "Heartbeat doit rafraîchir le lien (link_seen), sinon le retain jette un busy live"
    );
    let _ = std::fs::remove_file(&config.db_path);
}

/// ORACLE lot A — zombie : wrapper qui heartbeate sans capacité.
/// Contrôle positif : une capacité fraîche reste visible.
/// Mutant : si Heartbeat remettait capacity_seen à jour, last_seen_secs
/// redeviendrait ~0 et l'assertion « capacité gelée » tomberait.
#[test]
fn heartbeat_ne_rajeunit_pas_une_capacite_morte() {
    let (mut state, config) = state_with_registered_agent("heartbeat-zombie");
    let stale = Instant::now()
        .checked_sub(Duration::from_secs(1900))
        .expect("horloge");
    {
        let presence = state.presences.get_mut("instance-1").unwrap();
        presence.state = "connected".to_string();
        presence.capacity_seen = stale;
        // Lien encore frais (socket vivant) — c'est le cas nominal zombie.
        presence.link_seen = Instant::now();
    }
    let shared = Arc::new(Mutex::new(state));
    for _ in 0..3 {
        assert!(handle_wrapper_message("conn-1", WrapperToDaemon::Heartbeat, &shared).is_none());
    }
    let mut st = shared.lock().unwrap();
    let presence = st
        .presences
        .get("instance-1")
        .expect("lien garde la présence");
    assert!(
        presence.capacity_seen.elapsed() >= Duration::from_secs(1800),
        "la capacité doit rester gelée sous heartbeat seul: {:?}",
        presence.capacity_seen.elapsed()
    );
    let infos = st.agent_infos();
    let agent = infos
        .iter()
        .find(|agent| agent.agent_id == "89000000-0000-4000-8000-000000000102")
        .expect("contrôle positif : présence légitime (lien frais) reste à l'annuaire");
    assert!(
        agent.last_seen_secs >= 1800,
        "who doit exposer l'âge de CAPACITÉ, pas du lien: last_seen_secs={}",
        agent.last_seen_secs
    );
    let _ = std::fs::remove_file(&config.db_path);
}

/// Contrôle positif lot A : un TurnState (capacité) rafraîchit capacity_seen.
#[test]
fn capacite_vraie_rafraichit_last_seen() {
    let (mut state, config) = state_with_registered_agent("capacite-vraie");
    let stale = Instant::now()
        .checked_sub(Duration::from_secs(1900))
        .expect("horloge");
    state.presences.get_mut("instance-1").unwrap().capacity_seen = stale;
    state.presences.get_mut("instance-1").unwrap().link_seen = stale;
    std::thread::sleep(Duration::from_millis(5));
    state.set_turn_state("conn-1", false).unwrap();
    let presence = state.presences.get("instance-1").unwrap();
    assert!(
        presence.capacity_seen.elapsed() < Duration::from_secs(2),
        "TurnState doit attestier une capacité fraîche"
    );
    assert!(
        presence.link_seen.elapsed() < Duration::from_secs(2),
        "une capacité touche aussi le lien"
    );
    let infos = state.agent_infos();
    let agent = infos
        .iter()
        .find(|agent| agent.agent_id == "89000000-0000-4000-8000-000000000102")
        .expect("présence légitime visible");
    assert!(
        agent.last_seen_secs < 2,
        "contrôle positif : capacité fraîche ⇒ last_seen_secs bas ({})",
        agent.last_seen_secs
    );
    let _ = std::fs::remove_file(&config.db_path);
}

/// ORACLE lot B — busy jury composé (A+B) : capacité gelée 1 h, heartbeat
/// seul → survit (retain = horloge lien).
#[test]
fn held159_turnstate_busy_period_start_non_remis_a_zero_par_true_repete() {
    let (mut st, config) = state_with_registered_agent("held159-period");
    st.presences.get_mut("instance-1").unwrap().turn_attest = true;
    let start = Instant::now();
    st.set_turn_state_at("conn-1", true, start).unwrap();
    let repeated = start + Duration::from_secs(30);
    st.set_turn_state_at("conn-1", true, repeated).unwrap();
    let p = &st.presences["instance-1"];
    assert_eq!(p.busy_period_start, Some(start));
    assert_eq!(p.last_attested_at, Some(repeated));
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn held159_turnstate_sortie_forcee_depuis_busy_period_start() {
    let (mut st, config) = state_with_registered_agent("held159-force");
    let _registry_lock = HOME_REGISTRY_LOCK.lock().recover_poison();
    let home = st.fixture_root.as_ref().unwrap().0.clone();
    let source = home.join("agents.json");
    let registry_json = serde_json::json!({"agents":{"claude":{
        "command":"/bin/sh","protocol":"acp","forbidden_env":[],"pass_env":[],
        "permissions":"allow","queue_capacity":32,"notify_timeout_secs":1,"held_queue":false
    }}})
    .to_string();
    std::fs::write(&source, &registry_json).unwrap();
    std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o600)).unwrap();
    st.registry = crate::registry::AgentRegistry::from_json(&registry_json, source).unwrap();
    let previous_root = std::env::var_os("BRIDGET_HOME");
    let previous_socket = std::env::var_os("BRIDGET_SOCKET");
    // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
    unsafe {
        std::env::set_var("BRIDGET_HOME", &home);
        std::env::set_var("BRIDGET_SOCKET", home.join("bridget.sock"));
    }
    let live_timeout = live_notify_timeout_secs(&st.registry, "claude");
    st.presences.get_mut("instance-1").unwrap().turn_attest = true;
    let start = Instant::now();
    st.set_turn_state_at("conn-1", true, start).unwrap();
    st.set_turn_state_at("conn-1", true, start + Duration::from_secs(30))
        .unwrap();
    st.release_stale_busy_turns_at(start + Duration::from_secs(31));
    assert_eq!(
        st.presences["instance-1"].state, "busy",
        "true frais malgré délai court dépassé"
    );
    st.release_stale_busy_turns_at(start + Duration::from_secs(120));
    assert_eq!(
        st.presences["instance-1"].state, "busy",
        "silence de90s ne dépasse pas trois périodes"
    );
    st.release_stale_busy_turns_at(start + Duration::from_secs(121));
    // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
    unsafe {
        if let Some(value) = previous_root {
            std::env::set_var("BRIDGET_HOME", value);
        } else {
            std::env::remove_var("BRIDGET_HOME");
        }
        if let Some(value) = previous_socket {
            std::env::set_var("BRIDGET_SOCKET", value);
        } else {
            std::env::remove_var("BRIDGET_SOCKET");
        }
    }
    assert_eq!(live_timeout, 1, "délai lu dans le vrai registre privé");
    let p = &st.presences["instance-1"];
    assert_eq!(
        p.state, "connected",
        "expirationconfig+30 ET silence supérieur à90s, même held_queuefalse"
    );
    assert_eq!(
        p.busy_period_start,
        Some(start),
        "motifexpiration conservé, pas fauxrepos"
    );
    assert_eq!(
        p.last_attested_at,
        Some(start + Duration::from_secs(30)),
        "aucune attestationfraîche fabriquée"
    );
    st.set_turn_state_at("conn-1", true, start + Duration::from_secs(122))
        .unwrap();
    assert_eq!(
        st.presences["instance-1"].busy_period_start,
        Some(start),
        "un true sans false conserve le début de la période occupée"
    );
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn held159_turnstate_wrapper_muet_trois_periodes() {
    let (mut st, config) = state_with_registered_agent("held159-silent");
    let source = st.registry.source().to_path_buf();
    st.registry = crate::registry::AgentRegistry::from_json(
        &serde_json::json!({"agents":{"claude":{
            "command":"/bin/sh","protocol":"acp","forbidden_env":[],"pass_env":[],
            "permissions":"allow","queue_capacity":32,"notify_timeout_secs":2700,"held_queue":false
        }}})
        .to_string(),
        source,
    )
    .unwrap();
    st.presences.get_mut("instance-1").unwrap().turn_attest = true;
    let start = Instant::now();
    st.set_turn_state_at("conn-1", true, start).unwrap();
    for secs in [30, 60, 90] {
        st.release_stale_busy_turns_at(start + Duration::from_secs(secs));
        let p = &st.presences["instance-1"];
        assert_eq!(p.state, "busy", "absence de signal ne prouve pas repos");
        assert_eq!(p.last_attested_at, Some(start));
    }
    st.release_stale_busy_turns_at(start + Duration::from_secs(2730));
    assert_eq!(
        st.presences["instance-1"].state, "connected",
        "expirationFR025, pas modèlelibre prouvé"
    );
    assert_eq!(st.presences["instance-1"].last_attested_at, Some(start));
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn held159_turnstate_busy_since_inchange() {
    let (mut st, config) = state_with_registered_agent("held159-legacy");
    assert!(!st.presences["instance-1"].turn_attest);
    let start = Instant::now();
    st.set_turn_state_at("conn-1", true, start).unwrap();
    let repeated = start + Duration::from_secs(30);
    st.set_turn_state_at("conn-1", true, repeated).unwrap();
    let p = &st.presences["instance-1"];
    assert_eq!(p.busy_since, Some(repeated));
    assert!(p.busy_period_start.is_none() && p.last_attested_at.is_none());
    let ttl =
        live_notify_timeout_secs(&st.registry, &p.agent_type).saturating_add(TIMEOUT_GRACE_PERIOD);
    st.release_stale_busy_turns_at(start + Duration::from_secs(ttl));
    assert_eq!(
        st.presences["instance-1"].state, "busy",
        "legacytrue répété garde sa borneancienne"
    );
    st.release_stale_busy_turns_at(repeated + Duration::from_secs(ttl));
    assert_eq!(st.presences["instance-1"].state, "connected");
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn lot_b_busy_jury_survit_sous_heartbeat_malgre_capacite_gelee() {
    let (mut state, config) = state_with_registered_agent("lotb-busy-compose");
    state.set_turn_state("conn-1", true).unwrap();
    let capacity_stale = Instant::now()
        .checked_sub(Duration::from_secs(3600))
        .expect("horloge");
    {
        let p = state.presences.get_mut("instance-1").unwrap();
        p.capacity_seen = capacity_stale;
        p.link_seen = capacity_stale;
    }
    let shared = Arc::new(Mutex::new(state));
    assert!(handle_wrapper_message("conn-1", WrapperToDaemon::Heartbeat, &shared).is_none());
    let mut st = shared.lock().unwrap();
    let p = st.presences.get("instance-1").expect("présence");
    assert!(
        p.capacity_seen.elapsed() >= Duration::from_secs(3500),
        "capacité doit rester gelée sous heartbeat seul"
    );
    assert!(
        presence_within_retention(p),
        "busy + lien frais ⇒ keep_b_compose"
    );
    let infos = st.agent_infos();
    assert!(
        infos
            .iter()
            .any(|a| a.agent_id == "89000000-0000-4000-8000-000000000102" && a.state == "busy"),
        "busy jury ne doit pas être purgé: {infos:?}"
    );
    let _ = std::fs::remove_file(&config.db_path);
}

/// ORACLE — client du protocole public : ListAgents atteste le busy libéré,
/// puis SendIdempotent livre le corps exact au wrapper (aucun métier le service compagnon).
#[test]
fn tour_non_abouti_redevient_mandatable_et_le_mandat_parvient() {
    use std::os::unix::fs::DirBuilderExt;
    let root = std::env::temp_dir().join(format!("bc-busy-{}", uuid::Uuid::new_v4().simple()));
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&root)
        .unwrap();
    let config = DaemonConfig {
        socket_path: root.join("s"),
        db_path: root.join("bridget.db"),
        log_path: root.join("log"),
        ..DaemonConfig::default()
    };
    let (managed_tx, _managed_rx) = mpsc::channel();
    let mut state = DaemonState::new(&config, managed_tx).unwrap();
    let target = "26b9ed92-9a62-4b1f-bd14-e3bce3a01111";
    state
        .router
        .register(target, &bridget_core::AgentType::Claude, "conn-1")
        .unwrap();
    state
        .conn_instances
        .insert("conn-1".into(), "instance-1".into());
    state.presences.insert(
        "instance-1".into(),
        Presence {
            name: target.to_string(),
            agent_type: "claude".to_string(),
            host: "macbook".to_string(),
            transport: "acp".to_string(),
            channel: Some("unix".to_string()),
            mode: Some(PresenceMode::Acp),
            location: None,
            journal_available: true,
            os: "macOS".to_string(),
            state: "connected".to_string(),
            busy_since: None,
            busy_period_start: None,
            last_attested_at: None,
            turn_attest: false,
            queue_fallback: false,
            capacity_seen: Instant::now(),
            link_seen: Instant::now(),
            reconnect_count: 0,
            model: None,
            effort: None,
            rate_limits: Default::default(),
            served_model: None,
            derived_domain: None,
            domain: None,
            dnd_until: None,
            disk_space: None,
        },
    );

    state.set_turn_state("conn-1", true).unwrap();
    let ttl =
        live_notify_timeout_secs(&state.registry, "claude").saturating_add(TIMEOUT_GRACE_PERIOD);
    let aged = Instant::now()
        .checked_sub(Duration::from_secs(ttl.saturating_add(1)))
        .expect("horloge busy_since");
    state.presences.get_mut("instance-1").unwrap().busy_since = Some(aged);

    let (writer_stream, peer) = UnixStream::pair().unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    state.connections.insert(
        "conn-1".to_string(),
        Arc::new(Mutex::new(BufWriter::new(writer_stream))),
    );
    assert_eq!(state.next_conn_id(), "conn-1");
    // Spec 099 : un client n'envoie qu'au nom d'une identité qu'il prouve.
    // L'expéditeur du mandat est un agent vivant dont le client public
    // détient la preuve de rattachement.
    const SENDER: &str = "98beefe0-0000-4000-8000-000000000201";
    let _sender_owner = super::spec094_live_test_connection(&mut state, "owner-sender");
    let DaemonToWrapper::Registered {
        credential: Some(sender_credential),
        ..
    } = handle_register_with_channel(
        "owner-sender",
        2,
        "fixture".into(),
        SENDER.into(),
        Some("fixture".into()),
        Some("acp".into()),
        ChannelReport::Known("unix".into()),
        Some(PresenceMode::Acp),
        None,
        Some("test".into()),
        Some("instance-sender".into()),
        None,
        false,
        Some(false),
        &mut state,
        false,
    )
    else {
        panic!("enregistrement de l'expéditeur attendu");
    };
    let shared = Arc::new(Mutex::new(state));
    let listener = UnixListener::bind(&config.socket_path).unwrap();
    listener.set_nonblocking(true).unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let server_stop = Arc::clone(&stop);
    let server_state = Arc::clone(&shared);
    let server = thread::spawn(move || {
        let mut connections = Vec::new();
        while !server_stop.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((stream, _)) => {
                    // Darwin hérite ici du listener non bloquant de la fixture.
                    // Le listener produit est bloquant : conserver ce contrat.
                    stream.set_nonblocking(false).unwrap();
                    stream
                        .set_read_timeout(Some(Duration::from_secs(3)))
                        .unwrap();
                    let connection_state = Arc::clone(&server_state);
                    connections.push(thread::spawn(move || {
                        if let Err(error) = handle_connection(stream, connection_state) {
                            eprintln!("connexion publique: {error}");
                        }
                    }));
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("accept client public: {error}"),
            }
        }
        for connection in connections {
            connection.join().unwrap();
        }
    });

    struct ServerGuard {
        stop: Arc<AtomicBool>,
        thread: Option<thread::JoinHandle<()>>,
    }
    impl Drop for ServerGuard {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::SeqCst);
            if let Some(thread) = self.thread.take() {
                // Les clients sont déclarés APRÈS cette garde, donc fermés
                // avant elle pendant unwind. Le timeout serveur borne
                // également un client accepté qui ne fermerait pas.
                let _ = thread.join();
            }
        }
    }
    let server_guard = ServerGuard {
        stop,
        thread: Some(server),
    };
    const MANDAT_BODY: &str = "MANDAT-BUSY-RECOVERY-ORACLE-98beefe0";
    // ListAgents est la projection historique publique, sur sa connexion
    // courte ; le rôle Client 012 est réservé aux opérations négociées.
    let directory = daemon_request(&config.socket_path, WrapperToDaemon::ListAgents);
    let DaemonToWrapper::AgentList { agents } = directory else {
        panic!("annuaire: {directory:?}")
    };
    assert!(
        agents
            .iter()
            .any(|agent| agent.agent_id == target && agent.state == "connected"),
        "la capacité libérée doit être publiée: {agents:?}"
    );
    let stream = UnixStream::connect(&config.socket_path).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let mut writer = BufWriter::new(stream.try_clone().unwrap());
    let mut reader = BufReader::new(stream);
    let mut exchange = |command: WrapperToDaemon| {
        writeln!(writer, "{}", encode(&command).unwrap()).unwrap();
        writer.flush().unwrap();
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        decode::<DaemonToWrapper>(line.trim())
            .unwrap_or_else(|error| panic!("réponse à {command:?}: {line:?}: {error}"))
    };
    assert!(matches!(
        exchange(WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Client
        }),
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        }
    ));
    assert!(matches!(
        exchange(WrapperToDaemon::RegisterAuxiliary {
            agent_id: SENDER.into(),
            instance_id: "instance-sender".into(),
            credential: sender_credential,
        }),
        DaemonToWrapper::Registered {
            credential: None,
            ..
        }
    ));
    assert!(matches!(
        exchange(WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: crate::communication::issuer_scope("busy-public-client"),
            capabilities: vec![ClientCapability::SendIdempotent],
        }),
        DaemonToWrapper::ClientWelcome { .. }
    ));
    let message = bridget_core::BridgetMessage::new(SENDER, target, MANDAT_BODY);
    let message_id = message.id.clone();
    let issue = exchange(WrapperToDaemon::SendIdempotent {
        message,
        message_id,
        issued_at: unix_now_secs(),
    });
    assert!(
        matches!(
            issue,
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ),
        "{issue:?}"
    );
    drop(writer);
    drop(reader);
    drop(server_guard);

    let mut line = String::new();
    BufReader::new(peer).read_line(&mut line).unwrap();
    let delivered: DaemonToWrapper = decode(line.trim()).unwrap();
    assert!(
        matches!(delivered, DaemonToWrapper::DeliverIdempotent { message, .. }
            if message.body == MANDAT_BODY && message.to == target),
        "le message filaire doit parvenir intact au wrapper"
    );
    let _ = std::fs::remove_file(&config.socket_path);
    let _ = std::fs::remove_file(&config.db_path);
    let _ = std::fs::remove_dir_all(root);
}

const MAX_NOTIFY_TIMEOUT_CHILD_ENV: &str = "BRIDGET_MAX_NOTIFY_TIMEOUT_CHILD";

/// Frontière du registre : `u64::MAX` ne doit jamais faire paniquer
/// `agent_infos` lors du calcul notify_timeout + grâce.
#[test]
fn notify_timeout_max_ne_panique_pas_dans_agent_infos() {
    if std::env::var(MAX_NOTIFY_TIMEOUT_CHILD_ENV).ok().as_deref() == Some("1") {
        let (mut state, config) = state_with_registered_agent("max-notify-timeout-child");
        state.set_turn_state("conn-1", true).unwrap();
        let infos = state.agent_infos();
        assert_eq!(infos[0].state, "busy");
        let _ = std::fs::remove_file(config.db_path);
        return;
    }

    let root = std::env::temp_dir().join(format!(
        "bridget-max-notify-timeout-{}",
        uuid::Uuid::new_v4().simple()
    ));
    let registry_dir = root.join(".config/bridget");
    std::fs::create_dir_all(&registry_dir).unwrap();
    std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).unwrap();
    let registry_path = registry_dir.join("agents.json");
    std::fs::write(
        &registry_path,
        serde_json::to_vec(&serde_json::json!({
            "agents": {
                "claude": {
                    "command": "/bin/true",
                    "protocol": "acp",
                    "forbidden_env": [],
                    "pass_env": [],
                    "notify_timeout_secs": u64::MAX
                }
            }
        }))
        .unwrap(),
    )
    .unwrap();
    std::fs::set_permissions(&registry_path, std::fs::Permissions::from_mode(0o600)).unwrap();

    let output = Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("daemon::presence_tests::notify_timeout_max_ne_panique_pas_dans_agent_infos")
        .arg("--nocapture")
        .env("HOME", &root)
        .env(MAX_NOTIFY_TIMEOUT_CHILD_ENV, "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "agent_infos a paniqué à u64::MAX: stdout={}; stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(root);
}

/// ORACLE lot B — contrôle positif : capacité/lien frais → survit
/// (ne pas remplacer l'immortalité par la mortalité universelle).
#[test]
fn lot_b_presence_fraiche_survit_au_retain() {
    let (mut state, config) = state_with_registered_agent("lotb-frais");
    state.presences.get_mut("instance-1").unwrap().capacity_seen = Instant::now();
    state.presences.get_mut("instance-1").unwrap().link_seen = Instant::now();
    {
        let p = state.presences.get("instance-1").unwrap();
        assert!(
            presence_within_retention(p),
            "capacité/lien frais ⇒ keep_b_compose"
        );
    }
    let infos = state.agent_infos();
    assert!(
        infos
            .iter()
            .any(|a| a.agent_id == "89000000-0000-4000-8000-000000000102"),
        "présence fraîche doit survivre: {infos:?}"
    );
    let _ = std::fs::remove_file(config.db_path);
}

/// ORACLE lot B — connected mort (relec1/relec5) : deux horloges
/// périmées → purgé sans redémarrage. Mutant : exemption connected.
#[test]
fn lot_b_connected_mort_est_purge_sans_redemarrage() {
    let (mut state, config) = state_with_registered_agent("lotb-connected-mort");
    let stale = Instant::now()
        .checked_sub(PRESENCE_RETENTION + Duration::from_secs(3600))
        .expect("horloge");
    {
        let p = state.presences.get_mut("instance-1").unwrap();
        p.state = "connected".to_string();
        p.capacity_seen = stale;
        p.link_seen = stale;
        assert!(
            presence_within_retention_mutant_exempt_connected(p),
            "mutant exemption garderait le mort"
        );
        assert!(
            !presence_within_retention(p),
            "connected mort ⇒ keep_b_compose=false"
        );
    }
    let infos = state.agent_infos();
    assert!(
        infos
            .iter()
            .all(|a| a.agent_id != "89000000-0000-4000-8000-000000000102"),
        "connected mort doit disparaître sans redémarrage: {infos:?}"
    );
    assert!(!state.presences.contains_key("instance-1"));
    assert!(
        state
            .router
            .get_agent("89000000-0000-4000-8000-000000000102")
            .is_none()
    );
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn journal_ready_atteste_le_gate_attach_apres_register() {
    let (mut state, config) = state_with_registered_agent("attach-journal-ready");
    state
        .presences
        .get_mut("instance-1")
        .unwrap()
        .journal_available = false;
    let shared = Arc::new(Mutex::new(state));

    assert_eq!(
        attach_refusal_for_subscription(
            &shared.lock().unwrap(),
            "89000000-0000-4000-8000-000000000102"
        )
        .unwrap_err()
        .reason,
        AttachRefusal::JournalUnavailable
    );
    assert!(
        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::JournalReady {
                receipt_probe: false
            },
            &shared
        )
        .is_none()
    );
    assert!(
        shared
            .lock()
            .unwrap()
            .presences
            .get("instance-1")
            .unwrap()
            .journal_available
    );
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn wrapper_acp_historique_garde_son_journal_apres_reconnexion_vers_daemon_neuf() {
    let (mut state, config) = state_with_registered_agent("legacy-journal-reconnect");
    assert!(matches!(
        handle_register(
            "legacy-wrapper",
            "codex".to_string(),
            Some("89000000-0000-4000-8000-000000000114".to_string()),
            Some("local".to_string()),
            Some("acp".to_string()),
            None,
            None,
            Some("test".to_string()),
            Some("legacy-journal-instance".to_string()),
            None,
            false,
            None,
            &mut state,
        ),
        DaemonToWrapper::Registered { .. }
    ));
    assert!(
        state
            .presences
            .get("legacy-journal-instance")
            .expect("présence historique")
            .journal_available,
        "mutation discriminante : sans la compatibilité de version, le gate retombe à faux"
    );
    assert_eq!(
        attach_refusal_for_subscription(&state, "89000000-0000-4000-8000-000000000114")
            .unwrap_err()
            .reason,
        AttachRefusal::WrapperUnavailable
    );
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn wrapper_recentre_reste_non_attachable_jusqu_a_journal_ready() {
    let (mut state, config) = state_with_registered_agent("modern-journal-register");
    assert!(matches!(
        handle_register(
            "modern-wrapper",
            "codex".to_string(),
            Some("89000000-0000-4000-8000-000000000115".to_string()),
            Some("local".to_string()),
            Some("acp".to_string()),
            Some(PresenceMode::Acp),
            None,
            Some("test".to_string()),
            Some("modern-journal-instance".to_string()),
            None,
            false,
            Some(false),
            &mut state,
        ),
        DaemonToWrapper::Registered { .. }
    ));
    assert_eq!(
        attach_refusal_for_subscription(&state, "89000000-0000-4000-8000-000000000115")
            .unwrap_err()
            .reason,
        AttachRefusal::JournalUnavailable,
        "mutation discriminante : confondre l'absence historique et false réintroduirait le gate ACP"
    );
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn enregistrement_auxiliaire_mcp_ne_revendique_pas_la_presence_du_wrapper_vivant() {
    let (mut state, config) = state_with_registered_agent("presence-mcp-fusion");
    let _owner = super::spec094_live_test_connection(&mut state, "conn-1");
    let credential = state.identity_credentials["conn-1"].clone();
    let rich = state.presences.get_mut("instance-1").unwrap();
    rich.domain = Some("coordination".to_string());
    rich.derived_domain = Some("coordination".to_string());
    rich.model = Some("gpt-5.6-terra".to_string());
    rich.effort = Some("high".to_string());

    assert!(matches!(
        register_auxiliary(
            &mut state,
            "mcp-child",
            "89000000-0000-4000-8000-000000000102".into(),
            "instance-1".into(),
            credential.clone()
        ),
        DaemonToWrapper::Registered { .. }
    ));
    assert_eq!(
        state.conn_names.get("mcp-child").map(String::as_str),
        Some("89000000-0000-4000-8000-000000000102")
    );
    assert_eq!(
        state.conn_instances.get("mcp-child").map(String::as_str),
        Some("instance-1")
    );
    assert!(state.auxiliary_connections.contains("mcp-child"));

    // Fermeture de la connexion utilisée par l'outil MCP : elle ne doit
    // ni voler l'instance, ni rendre le wrapper principal inaccessible.
    state.router.unregister_by_conn("mcp-child");
    state.mark_unreachable("mcp-child");
    let agents = state.agent_infos();
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0].agent_id, "89000000-0000-4000-8000-000000000102");
    assert_eq!(agents[0].transport, "acp");
    assert_eq!(agents[0].mode, Some(PresenceMode::Acp));
    assert_eq!(agents[0].domain.as_deref(), Some("coordination"));
    assert_eq!(agents[0].model.as_deref(), Some("gpt-5.6-terra"));
    assert_eq!(agents[0].effort.as_deref(), Some("high"));
    assert_eq!(agents[0].state, "connected");

    assert!(matches!(
        register_auxiliary(
            &mut state,
            "mcp-child-2",
            "89000000-0000-4000-8000-000000000102".into(),
            "instance-1".into(),
            credential
        ),
        DaemonToWrapper::Registered { .. }
    ));
    state.mark_unreachable("conn-1");
    assert_eq!(
        state.presences.get("instance-1").unwrap().state,
        "unreachable",
        "une connexion auxiliaire ne masque jamais la perte du wrapper propriétaire"
    );
    state.mark_unreachable("mcp-child-2");
    assert_eq!(
        state.presences.get("instance-1").unwrap().state,
        "unreachable"
    );

    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn gere_acp_de_type_inconnu_tient_mode_et_runtime_de_sa_definition_figee() {
    let (mut state, config) = state_with_registered_agent("managed-runtime-definition");
    state.router.unregister_by_conn("conn-1");
    state.conn_instances.remove("conn-1");
    state.presences.clear();
    state.registry = AgentRegistry::from_json(
            r#"{"agents":{"codex-terra":{"command":"fixture-acp","args":["-c","model=\"gpt-5.6-terra\"","-c","model_reasoning_effort=\"high\""],"protocol":"acp","capabilities":{"execution_paths":["acp"],"models":{"gpt-5.6-terra":{"efforts":["high"]}}}}}}"#,
            "/tmp/agents.json",
        )
        .unwrap();
    let definition = state.registry.resolved_definition("codex-terra").unwrap();
    let now = unix_timestamp();
    let order = FleetSpawnOrder {
        remote_placement: None,
        posture: None,
        agent_type: "codex-terra".to_string(),
        requested_name: Some("89000000-0000-4000-8000-000000000110".to_string()),
        cwd: PathBuf::from("/tmp"),
        persistent: false,
        command_id: "managed-runtime-definition".to_string(),
        issued_at: now,
        deadline_at: now + 60,
        project: None,
        ownership: None,
    };
    let lease = match state.fleet.request_spawn(&order, now).unwrap() {
        crate::fleet::SpawnSubmission::Start(lease) => lease,
        other => panic!("réservation inattendue: {other:?}"),
    };
    state.fleet.mark_starting(&lease, now, &definition).unwrap();
    state
        .managed_by_instance
        .insert(lease.instance_id.clone(), lease.command_id.clone());
    let instance_id = lease.instance_id.clone();
    let (wrapper_writer, _wrapper_reader) = control_socket("managed-terra-wrapper");
    state
        .connections
        .insert("managed-terra".to_string(), wrapper_writer);

    assert!(matches!(
        handle_register(
            "managed-terra",
            "codex-terra".to_string(),
            Some("89000000-0000-4000-8000-000000000110".to_string()),
            Some("local".to_string()),
            // Mutation discriminante : un wrapper historique ne connaît
            // pas PresenceMode et annonce seulement son canal Unix. La
            // définition gérée `protocol=acp` doit rester l'autorité.
            Some("unix".to_string()),
            None,
            None,
            Some("macOS".to_string()),
            Some(instance_id.clone()),
            None,
            false,
            None,
            &mut state,
        ),
        DaemonToWrapper::Registered { .. }
    ));
    state
        .fleet
        .register_connected(&lease, &instance_id, now + 1)
        .unwrap();
    let agent = state.agent_infos().pop().expect("géré visible");
    assert_eq!(agent.agent_id, "89000000-0000-4000-8000-000000000110");
    assert_eq!(agent.transport, "acp");
    assert_eq!(agent.channel.as_deref(), Some("unix"));
    assert_eq!(agent.mode, Some(PresenceMode::Acp));
    assert_eq!(agent.model.as_deref(), Some("gpt-5.6-terra"));
    assert_eq!(agent.effort.as_deref(), Some("high"));
    assert_eq!(
        attach_refusal_for_subscription(&state, "89000000-0000-4000-8000-000000000110").unwrap(),
        "managed-terra"
    );

    let _ = std::fs::remove_file(config.db_path);
}

fn install_gere_acp_pour_reconnexion(state: &mut DaemonState, label: &str) -> SpawnLease {
    state.registry = AgentRegistry::from_json(
        r#"{"agents":{"cursor":{"command":"cursor-agent","args":["acp"],"protocol":"acp"}}}"#,
        "/tmp/agents-cursor.json",
    )
    .unwrap();
    let definition = state.registry.resolved_definition("cursor").unwrap();
    let now = unix_timestamp();
    let order = FleetSpawnOrder {
        remote_placement: None,
        posture: None,
        agent_type: "cursor".to_string(),
        requested_name: Some("89000000-0000-4000-8000-000000000111".to_string()),
        cwd: PathBuf::from("/tmp"),
        persistent: false,
        command_id: label.to_string(),
        issued_at: now,
        deadline_at: now + 60,
        project: None,
        ownership: None,
    };
    let lease = match state.fleet.request_spawn(&order, now).unwrap() {
        crate::fleet::SpawnSubmission::Start(lease) => lease,
        other => panic!("réservation inattendue: {other:?}"),
    };
    state.fleet.mark_starting(&lease, now, &definition).unwrap();
    state
        .managed_by_instance
        .insert(lease.instance_id.clone(), lease.command_id.clone());
    state.managed_spawns.insert(
        lease.command_id.clone(),
        ManagedSpawnRecord {
            lease: lease.clone(),
            agent_type: "cursor".to_string(),
            requester_conns: Vec::new(),
            wrapper_conn: None,
            stop: Arc::new(ManagedStopControl::new()),
        },
    );
    let (wrapper_writer, _wrapper_reader) = control_socket(label);
    state.connections.insert(label.to_string(), wrapper_writer);
    lease
}

#[test]
fn relance_en_cours_reste_visible_sans_actions_concurrentes() {
    let (mut state, config) = state_with_registered_agent("relaunching-annuaire");
    let lease = install_gere_acp_pour_reconnexion(&mut state, "relaunching-annuaire");
    state
        .relaunch_requesters
        .insert(lease.command_id.clone(), vec!["requester".to_string()]);

    let agent = state
        .agent_infos()
        .into_iter()
        .find(|agent| agent.agent_id == lease.name)
        .expect("la relance en cours doit rester visible");
    assert_eq!(agent.state, "relaunching");

    drop(state);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn gere_acp_reconnecte_sans_champs_wrapper_tient_mode_et_domaine() {
    let (mut state, config) = state_with_registered_agent("managed-reconnect-hostile");
    state.router.unregister_by_conn("conn-1");
    state.conn_instances.remove("conn-1");
    state.presences.clear();
    let lease = install_gere_acp_pour_reconnexion(&mut state, "managed-reconnect-hostile");
    let instance_id = lease.instance_id.clone();

    assert!(matches!(
        handle_register(
            "managed-reconnect-hostile",
            "cursor".to_string(),
            Some("89000000-0000-4000-8000-000000000111".to_string()),
            Some("local".to_string()),
            Some("unix".to_string()),
            None,
            None,
            Some("macOS".to_string()),
            Some(instance_id.clone()),
            Some("bridget".to_string()),
            false,
            None,
            &mut state,
        ),
        DaemonToWrapper::Registered { .. }
    ));
    let first = state.agent_infos().pop().expect("inscription initiale");
    assert_eq!(first.mode, Some(PresenceMode::Acp));
    assert_eq!(first.domain.as_deref(), Some("bridget"));
    assert_eq!(first.reconnect_count, 0);

    state.router.unregister_by_conn("managed-reconnect-hostile");
    state.mark_unreachable("managed-reconnect-hostile");
    // Mutation discriminante : la ré-inscription production d'un cursor
    // déjà connu ne retrouve plus l'instance dans managed_by_instance
    // (reprise = nouvel identifiant, ou carte instance périmée).
    state.managed_by_instance.remove(&instance_id);

    let (wrapper_writer, _wrapper_reader) = control_socket("managed-reconnect-hostile-2");
    state
        .connections
        .insert("managed-reconnect-hostile-2".to_string(), wrapper_writer);
    assert!(matches!(
        handle_register(
            "managed-reconnect-hostile-2",
            "cursor".to_string(),
            Some("89000000-0000-4000-8000-000000000111".to_string()),
            Some("local".to_string()),
            Some("unix".to_string()),
            None,
            None,
            Some("macOS".to_string()),
            Some(instance_id.clone()),
            None,
            true,
            None,
            &mut state,
        ),
        DaemonToWrapper::Registered { .. }
    ));
    let reconnected = state.agent_infos().pop().expect("réinscrit");
    assert_eq!(reconnected.agent_id, "89000000-0000-4000-8000-000000000111");
    assert_eq!(reconnected.transport, "acp");
    assert_eq!(reconnected.mode, Some(PresenceMode::Acp));
    assert_eq!(reconnected.domain.as_deref(), Some("bridget"));
    assert!(
        reconnected.reconnect_count >= 1,
        "la coupure doit incrémenter reconnect_count, reçu {}",
        reconnected.reconnect_count
    );

    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn gere_acp_deja_connu_se_reinscrit_sous_nouvelle_instance_tient_mode_et_domaine() {
    let (mut state, config) = state_with_registered_agent("managed-rebind-instance");
    state.router.unregister_by_conn("conn-1");
    state.conn_instances.remove("conn-1");
    state.presences.clear();
    let lease = install_gere_acp_pour_reconnexion(&mut state, "managed-rebind-instance");
    let old_instance = lease.instance_id.clone();

    assert!(matches!(
        handle_register(
            "managed-rebind-instance",
            "cursor".to_string(),
            Some("89000000-0000-4000-8000-000000000111".to_string()),
            Some("local".to_string()),
            Some("unix".to_string()),
            None,
            None,
            Some("macOS".to_string()),
            Some(old_instance.clone()),
            Some("bridget".to_string()),
            false,
            None,
            &mut state,
        ),
        DaemonToWrapper::Registered { .. }
    ));
    state.router.unregister_by_conn("managed-rebind-instance");
    state.mark_unreachable("managed-rebind-instance");
    state.managed_by_instance.remove(&old_instance);

    let new_instance = "cursor5-reprise-instance".to_string();
    let (wrapper_writer, _wrapper_reader) = control_socket("managed-rebind-instance-2");
    state
        .connections
        .insert("managed-rebind-instance-2".to_string(), wrapper_writer);
    assert!(matches!(
        handle_register(
            "managed-rebind-instance-2",
            "cursor".to_string(),
            Some("89000000-0000-4000-8000-000000000111".to_string()),
            Some("local".to_string()),
            Some("unix".to_string()),
            None,
            None,
            Some("macOS".to_string()),
            Some(new_instance),
            None,
            true,
            None,
            &mut state,
        ),
        DaemonToWrapper::Registered { .. }
    ));
    let reconnected = state
        .agent_infos()
        .into_iter()
        .find(|agent| {
            agent.agent_id == "89000000-0000-4000-8000-000000000111" && agent.state != "unreachable"
        })
        .expect("réinscrit visible");
    assert_eq!(reconnected.transport, "acp");
    assert_eq!(reconnected.mode, Some(PresenceMode::Acp));
    assert_eq!(reconnected.domain.as_deref(), Some("bridget"));
    assert!(reconnected.reconnect_count >= 1);

    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn abonnement_attach_distingue_un_equipier_arrete_d_un_nom_inconnu() {
    let (mut state, config) = state_with_registered_agent("attach-stopped");
    state.router.unregister_by_conn("conn-1");
    state.mark_stopped("conn-1");
    let shared = Arc::new(Mutex::new(state));
    assert!(matches!(
        handle_wrapper_message(
            "attach-1",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach,
            },
            &shared,
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "attach-1",
            WrapperToDaemon::Subscribe {
                agent: "89000000-0000-4000-8000-000000000102".to_string(),
                window: bridget_transport::AttachWindow::Today,
            },
            &shared,
        ),
        Some(DaemonToWrapper::AttachRejected {
            reason: AttachRefusal::AgentStopped,
            ..
        })
    ));
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn inscription_claude_geree_expose_acp_domaine_et_accepte_attach() {
    let (mut state, config) = state_with_registered_agent("claude-managed-metadata");
    state.router.unregister_by_conn("conn-1");
    state.conn_instances.remove("conn-1");
    state.presences.clear();

    let (wrapper_writer, mut wrapper_reader) = control_socket("claude-managed-wrapper");
    let (attach_writer, _attach_reader) = control_socket("claude-managed-attach");
    state.connections.insert(
        "89000000-0000-4000-8000-000000000112".to_string(),
        wrapper_writer,
    );
    state
        .connections
        .insert("attach-claude-managed".to_string(), attach_writer);

    assert!(matches!(
        handle_register(
            "89000000-0000-4000-8000-000000000112",
            "claude".to_string(),
            Some("89000000-0000-4000-8000-000000000112".to_string()),
            Some("local".to_string()),
            Some("unix".to_string()),
            Some(PresenceMode::Acp),
            None,
            Some("macOS".to_string()),
            Some("instance-claude-managed".to_string()),
            Some("bridget".to_string()),
            false,
            None,
            &mut state,
        ),
        DaemonToWrapper::Registered { credential: _, agent_id: ref name } if name == "89000000-0000-4000-8000-000000000112"
    ));

    let agent = state.agent_infos().pop().expect("Claude inscrit");
    assert_eq!(agent.transport, "acp");
    assert_eq!(agent.channel.as_deref(), Some("unix"));
    assert_eq!(agent.mode, Some(PresenceMode::Acp));
    assert_eq!(agent.domain.as_deref(), Some("bridget"));

    let shared = Arc::new(Mutex::new(state));
    assert!(
        handle_wrapper_message(
            "89000000-0000-4000-8000-000000000112",
            WrapperToDaemon::JournalReady {
                receipt_probe: false
            },
            &shared,
        )
        .is_none()
    );
    assert!(matches!(
        handle_wrapper_message(
            "attach-claude-managed",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Attach,
            },
            &shared,
        ),
        Some(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        })
    ));
    assert!(
        handle_wrapper_message(
            "attach-claude-managed",
            WrapperToDaemon::Subscribe {
                agent: "89000000-0000-4000-8000-000000000112".to_string(),
                window: bridget_transport::AttachWindow::Today,
            },
            &shared,
        )
        .is_none(),
        "un Claude géré en ACP doit être attachable"
    );
    assert!(matches!(
        read_control(&mut wrapper_reader),
        DaemonToWrapper::Subscribe { .. }
    ));

    let (controls, views) = {
        let mut state = shared.lock().unwrap();
        close_attach_subscriptions(&mut state, "attach-claude-managed")
    };
    assert!(execute_controls(controls).is_empty());
    for view in views {
        view.close_and_join();
    }
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn confirmation_vers_vue_fermee_desabonne_le_wrapper() {
    let (mut state, config) = state_with_registered_agent("attach-closed-view");
    let (wrapper_writer, mut wrapper_reader) = control_socket("wrapper-closed-view");
    let (attach_writer, attach_reader) = control_socket("attach-closed-view");
    state
        .connections
        .insert("conn-1".to_string(), wrapper_writer);
    state
        .connections
        .insert("attach-1".to_string(), attach_writer);
    let shared = Arc::new(Mutex::new(state));
    handle_wrapper_message(
        "attach-1",
        WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Attach,
        },
        &shared,
    );
    handle_wrapper_message(
        "attach-1",
        WrapperToDaemon::Subscribe {
            agent: "89000000-0000-4000-8000-000000000102".to_string(),
            window: bridget_transport::AttachWindow::Today,
        },
        &shared,
    );
    let subscription_id = match read_control(&mut wrapper_reader) {
        DaemonToWrapper::Subscribe {
            subscription_id, ..
        } => subscription_id,
        other => panic!("commande wrapper inattendue: {}", encode(&other).unwrap()),
    };
    drop(attach_reader);
    handle_wrapper_message(
        "conn-1",
        WrapperToDaemon::Subscribed {
            subscription_id: subscription_id.clone(),
        },
        &shared,
    );
    assert!(matches!(
        read_control(&mut wrapper_reader),
        DaemonToWrapper::Unsubscribe { subscription_id: id } if id == subscription_id
    ));
    assert!(shared.lock().unwrap().attach_subscriptions.is_empty());
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn vue_suspendue_coalesce_un_gap_et_ne_garde_pas_de_fragment_orphelin() {
    let view = AttachView::suspended("sub-gap");
    let bytes = vec![b'x'; 600 * 1024];
    assert!(view.enqueue(DaemonToWrapper::JournalFragment {
        subscription_id: "sub-gap".to_string(),
        seq: 41,
        offset: 0,
        final_fragment: false,
        bytes: bytes.clone(),
    }));
    assert!(view.enqueue(DaemonToWrapper::JournalFragment {
        subscription_id: "sub-gap".to_string(),
        seq: 42,
        offset: 0,
        final_fragment: true,
        bytes,
    }));
    let queue = view.queue.0.lock().unwrap();
    assert_eq!(queue.gap.as_ref().map(|gap| (gap.0, gap.1)), Some((41, 41)));
    assert_eq!(queue.messages.len(), 1);
    assert_eq!(
        queue.messages.front().and_then(|message| message.seq),
        Some(42)
    );
}

#[test]
fn eviction_ignore_les_fragments_restants_de_la_sequence_lachee() {
    let view = AttachView::suspended("sub-fragments");
    let bytes = vec![b'x'; 600 * 1024];
    assert!(view.enqueue(DaemonToWrapper::JournalFragment {
        subscription_id: "sub-fragments".to_string(),
        seq: 41,
        offset: 0,
        final_fragment: false,
        bytes: bytes.clone(),
    }));
    assert!(!view.enqueue(DaemonToWrapper::JournalFragment {
        subscription_id: "sub-fragments".to_string(),
        seq: 41,
        offset: 1,
        final_fragment: false,
        bytes: bytes.clone(),
    }));
    assert!(!view.enqueue(DaemonToWrapper::JournalFragment {
        subscription_id: "sub-fragments".to_string(),
        seq: 41,
        offset: 2,
        final_fragment: false,
        bytes: b"continuation evincee".to_vec(),
    }));
    assert!(!view.enqueue(DaemonToWrapper::JournalFragment {
        subscription_id: "sub-fragments".to_string(),
        seq: 41,
        offset: 3,
        final_fragment: true,
        bytes: b"frontiere evincee".to_vec(),
    }));
    assert!(view.enqueue(DaemonToWrapper::JournalFragment {
        subscription_id: "sub-fragments".to_string(),
        seq: 42,
        offset: 4,
        final_fragment: true,
        bytes: b"sequence suivante admise".to_vec(),
    }));
    let queue = view.queue.0.lock().unwrap();
    assert_eq!(queue.dropping_seq, None);
    assert!(queue.messages.iter().all(|message| message.seq != Some(41)));
    assert_eq!(queue.gap.as_ref().map(|gap| (gap.0, gap.1)), Some((41, 41)));
    assert_eq!(
        queue
            .messages
            .iter()
            .filter(|message| message.seq == Some(42))
            .count(),
        1
    );
}

#[test]
fn saturation_non_sequencee_notifie_et_recolte_la_vue_exactement_une_fois() {
    let (mut state, config) = state_with_registered_agent("attach-non-seq-close");
    let (wrapper_writer, _wrapper_reader) = control_socket("attach-non-seq-wrapper");
    let (attach_writer, _attach_reader) = control_socket("attach-non-seq-view");
    let view = AttachView::suspended_with_sender("sub-non-seq", state.view_closed_tx.clone());
    state
        .connections
        .insert("conn-1".to_string(), wrapper_writer);
    state
        .connections
        .insert("attach-1".to_string(), attach_writer);
    state.attach_subscriptions.insert(
        "sub-non-seq".to_string(),
        AttachSubscription {
            agent: "89000000-0000-4000-8000-000000000102".to_string(),
            attach_conn: "attach-1".to_string(),
            wrapper_conn: "conn-1".to_string(),
            caught_up: false,
        },
    );
    state
        .attach_views
        .insert("sub-non-seq".to_string(), view.clone());
    {
        let mut queue = view.queue.0.lock().unwrap();
        queue.bytes = ATTACH_VIEW_BUFFER_BYTES;
    }
    assert!(!view.enqueue(DaemonToWrapper::SnapshotCaughtUp {
        subscription_id: "sub-non-seq".to_string(),
        through_seq: Some(7),
    }));
    assert!(!view.enqueue(DaemonToWrapper::Gap {
        subscription_id: "sub-non-seq".to_string(),
        from_seq: 8,
        to_seq: 9,
        reason: None,
    }));
    let (controls, views) = collect_closed_attach_views(&mut state);
    let _ = execute_controls(controls);
    for view in views {
        view.close_and_join();
    }
    assert!(state.attach_subscriptions.is_empty());
    assert!(state.attach_views.is_empty());
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn spec241_identity_response_does_not_join_another_views_writer() {
    let (mut state, config) = state_with_registered_agent("241");
    let view = AttachView::suspended_with_sender("241-slow", state.view_closed_tx.clone());
    let (release_tx, release_rx) = mpsc::channel();
    *view.worker.lock().unwrap() = Some(thread::spawn(move || {
        release_rx.recv_timeout(Duration::from_secs(3)).unwrap();
    }));
    state.attach_subscriptions.insert(
        "241-slow".into(),
        AttachSubscription {
            agent: "241-agent".into(),
            attach_conn: "241-attach".into(),
            wrapper_conn: "241-wrapper".into(),
            caught_up: false,
        },
    );
    state.attach_views.insert("241-slow".into(), view.clone());
    view.close_and_notify();
    let shared = Arc::new(Mutex::new(state));
    let serving = shared.clone();
    let (tx, rx) = mpsc::channel();
    let handler = thread::spawn(move || {
        tx.send(handle_wrapper_message(
            "241-probe",
            WrapperToDaemon::DaemonIdentityRequest,
            &serving,
        ))
        .unwrap();
    });
    let response = rx.recv_timeout(Duration::from_millis(150));
    release_tx.send(()).unwrap();
    handler.join().unwrap();
    // La libération du rédacteur sert seulement à fermer proprement le test.
    assert!(
        matches!(
            response,
            Ok(Some(DaemonToWrapper::DaemonIdentityReport { .. }))
        ),
        "une vue d'une autre session retient la réponse : {response:?}"
    );
    let (controls, views) = collect_closed_attach_views(&mut shared.lock().unwrap());
    let _ = execute_controls(controls);
    for view in views {
        view.close_and_join();
    }
    view.close_and_join();
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn end_abandonne_la_file_et_interdit_toute_frame_ulterieure() {
    let view = AttachView::suspended("sub-terminal");
    assert!(view.enqueue(DaemonToWrapper::JournalFragment {
        subscription_id: "sub-terminal".to_string(),
        seq: 5,
        offset: 0,
        final_fragment: true,
        bytes: b"avant-end".to_vec(),
    }));
    assert!(view.enqueue(DaemonToWrapper::End {
        subscription_id: "sub-terminal".to_string(),
        reason: "fin".to_string(),
    }));
    assert!(!view.enqueue(DaemonToWrapper::JournalReadError {
        subscription_id: "sub-terminal".to_string(),
        line: 7,
        offset: 42,
        reason: "trop tard".to_string(),
    }));
    let queue = view.queue.0.lock().unwrap();
    assert_eq!(queue.messages.len(), 1);
    assert!(
        queue
            .messages
            .front()
            .is_some_and(|message| message.terminal)
    );
    assert!(queue.messages.iter().all(|message| message.seq.is_none()));
}

#[test]
fn producteurs_de_vue_et_controle_partagent_des_lignes_json_decodables() {
    let (writer_stream, reader_stream) = UnixStream::pair().unwrap();
    let writer = Arc::new(Mutex::new(BufWriter::new(writer_stream)));
    let (closed_tx, _closed_rx) = mpsc::channel();
    let view = AttachView::start("sub-serialise".to_string(), &writer, closed_tx).unwrap();
    let reader = thread::spawn(move || {
        let mut reader = BufReader::new(reader_stream);
        (0..33)
            .map(|_| {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                decode::<DaemonToWrapper>(line.trim()).is_ok()
            })
            .collect::<Vec<_>>()
    });
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let control_barrier = barrier.clone();
    let control_writer = writer.clone();
    let controls = thread::spawn(move || {
        control_barrier.wait();
        let controls = (0..16)
            .map(|number| DeferredControl {
                writer: control_writer.clone(),
                message: DaemonToWrapper::Ack {
                    id: format!("ack-{number}"),
                    project_warnings: Vec::new(),
                },
            })
            .chain(std::iter::once(DeferredControl {
                writer: control_writer.clone(),
                message: DaemonToWrapper::End {
                    subscription_id: "sub-serialise".to_string(),
                    reason: "fin".to_string(),
                },
            }))
            .collect();
        execute_controls(controls)
    });
    for seq in 0..16 {
        assert!(view.enqueue(DaemonToWrapper::JournalFragment {
            subscription_id: "sub-serialise".to_string(),
            seq,
            offset: seq,
            final_fragment: true,
            bytes: vec![b'x'; 2048],
        }));
    }
    barrier.wait();
    assert!(controls.join().unwrap().is_empty());
    assert!(reader.join().unwrap().into_iter().all(|decoded| decoded));
    view.close_and_join();
}

#[test]
fn end_indelivrable_force_l_eof_de_la_vue() {
    let (writer_stream, mut peer) = UnixStream::pair().unwrap();
    writer_stream
        .set_write_timeout(Some(Duration::from_millis(50)))
        .unwrap();
    let writer = Arc::new(Mutex::new(BufWriter::new(writer_stream)));
    // Remplir le tampon noyau sans lire le pair, puis échouer sur End.
    let payload = "x".repeat(16 * 1024);
    while push_control_message(
        &writer,
        &DaemonToWrapper::Ack {
            project_warnings: Vec::new(),
            id: payload.clone(),
        },
    ) {}
    assert!(!push_control_message(
        &writer,
        &DaemonToWrapper::End {
            subscription_id: "sub-eof".to_string(),
            reason: "vue trop lente".to_string(),
        }
    ));
    peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    let mut bytes = [0_u8; 1024];
    loop {
        match peer.read(&mut bytes) {
            Ok(0) => break,
            Ok(_) => continue,
            Err(error)
                if error.kind() == std::io::ErrorKind::WouldBlock
                    || error.kind() == std::io::ErrorKind::TimedOut =>
            {
                panic!("End ni EOF recus par la vue")
            }
            Err(error) => panic!("lecture pair: {error}"),
        }
    }
}

#[test]
fn rejet_tardif_attach_est_reroute_puis_purge() {
    let (mut state, config) = state_with_registered_agent("attach-late-reject");
    let (writer, mut reader) = control_socket("attach-late-reject");
    state.connections.insert("attach-1".to_string(), writer);
    state.pending_attach_sends.insert(
        "message-humain".to_string(),
        PendingAttachSend {
            conn_id: "attach-1".to_string(),
            expires_at: Instant::now() + Duration::from_secs(60),
        },
    );
    let shared = Arc::new(Mutex::new(state));
    handle_wrapper_message(
        "conn-1",
        WrapperToDaemon::DeliveryRejected {
            execution_id: None,
            generation: None,
            id: "message-humain".to_string(),
            reason: "adaptateur refusé".to_string(),
        },
        &shared,
    );
    assert!(matches!(
        read_control(&mut reader),
        DaemonToWrapper::DeliveryRejected { id, reason }
            if id == "message-humain" && reason == "adaptateur refusé"
    ));
    assert!(shared.lock().unwrap().pending_attach_sends.is_empty());
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn pending_attach_expire_et_sont_nettoyes_a_la_deconnexion() {
    let (mut state, config) = state_with_registered_agent("attach-purge");
    state.pending_attach_sends.insert(
        "expired".to_string(),
        PendingAttachSend {
            conn_id: "attach-1".to_string(),
            expires_at: Instant::now() - Duration::from_secs(1),
        },
    );
    purge_expired_attach_sends(&mut state);
    assert!(state.pending_attach_sends.is_empty());
    state.pending_attach_sends.insert(
        "live".to_string(),
        PendingAttachSend {
            conn_id: "attach-1".to_string(),
            expires_at: Instant::now() + Duration::from_secs(60),
        },
    );
    let (controls, views) = close_attach_subscriptions(&mut state, "attach-1");
    assert!(controls.is_empty());
    for view in views {
        view.close_and_join();
    }
    assert!(state.pending_attach_sends.is_empty());
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn spec091_selection_autorisee_seulement_depuis_attach_et_recue_du_bon_wrapper() {
    use bridget_transport::protocol::{RuntimeSelection, RuntimeSelectionOutcome as Outcome};
    let (mut state, _config) = state_with_registered_agent("selection091");
    let selection = RuntimeSelection {
        model: "model-test".into(),
        effort: "high".into(),
    };
    let (writer, mut reader) = control_socket("selection091");
    state.connections.insert("conn-1".into(), writer);
    state
        .connection_roles
        .insert("viewer".into(), ConnectionRole::Attach);
    state.attach_subscriptions.insert(
        "subscription091".into(),
        AttachSubscription {
            agent: "target".into(),
            attach_conn: "viewer".into(),
            wrapper_conn: "conn-1".into(),
            caught_up: true,
        },
    );
    let shared = Arc::new(Mutex::new(state));
    let forbidden = select_runtime_from_attach(&shared, "not-attach", "target", selection.clone());
    assert!(matches!(
        forbidden,
        DaemonToWrapper::RuntimeSelectionResult {
            outcome: Outcome::Refused { .. }
        }
    ));
    assert!(shared.lock().unwrap().pending_runtime_selections.is_empty());
    let worker_state = shared.clone();
    let requested = selection.clone();
    let worker = thread::spawn(move || {
        select_runtime_from_attach(&worker_state, "viewer", "target", requested)
    });
    let DaemonToWrapper::SelectRuntime {
        token,
        selection: dispatched,
    } = read_control(&mut reader)
    else {
        panic!("ordre réel attendu");
    };
    assert_eq!(dispatched, selection);
    // L'observation simultanée doit rester possible pendant l'attente du reçu.
    assert!(shared.try_lock().is_ok());
    let report = WrapperToDaemon::RuntimeSelectionReported {
        token: token.clone(),
        outcome: Outcome::Selected {
            selection: selection.clone(),
        },
    };
    handle_wrapper_message("other-wrapper", report.clone(), &shared);
    assert!(
        shared
            .lock()
            .unwrap()
            .pending_runtime_selections
            .contains_key(&token),
        "mutation suppression du contrôle propriétaire doit casser cet assert"
    );
    handle_wrapper_message("conn-1", report, &shared);
    assert!(
        matches!(worker.join().unwrap(), DaemonToWrapper::RuntimeSelectionResult { outcome:Outcome::Selected { selection:got } } if got == selection)
    );
    assert!(shared.lock().unwrap().pending_runtime_selections.is_empty());
}

#[test]
fn deux_vues_attach_recoivent_le_meme_flux_et_sont_recoltees() {
    let (mut state, config) = state_with_registered_agent("attach-two-views");
    let (wrapper_writer, _wrapper_reader) = control_socket("attach-two-wrapper");
    let (first_writer, mut first_reader) = control_socket("attach-two-first");
    let (second_writer, mut second_reader) = control_socket("attach-two-second");
    state
        .connections
        .insert("conn-1".to_string(), wrapper_writer);
    state
        .connections
        .insert("attach-1".to_string(), first_writer.clone());
    state
        .connections
        .insert("attach-2".to_string(), second_writer.clone());
    install_attach_view(&mut state, "sub-first", "attach-1", &first_writer);
    install_attach_view(&mut state, "sub-second", "attach-2", &second_writer);
    let shared = Arc::new(Mutex::new(state));

    for subscription_id in ["sub-first", "sub-second"] {
        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::JournalFragment {
                subscription_id: subscription_id.to_string(),
                seq: 7,
                offset: 12,
                final_fragment: true,
                bytes: b"ligne complete".to_vec(),
            },
            &shared,
        );
        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::SnapshotCaughtUp {
                subscription_id: subscription_id.to_string(),
                through_seq: Some(7),
            },
            &shared,
        );
        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::Gap {
                subscription_id: subscription_id.to_string(),
                from_seq: 8,
                to_seq: 9,
                reason: Some("retard".to_string()),
            },
            &shared,
        );
    }

    let first = (0..3)
        .map(|_| normalized_attach_frame(read_control(&mut first_reader)))
        .collect::<Vec<_>>();
    let second = (0..3)
        .map(|_| normalized_attach_frame(read_control(&mut second_reader)))
        .collect::<Vec<_>>();
    assert_eq!(
        first, second,
        "les deux vues doivent recevoir les memes trames"
    );

    let (controls, views) = {
        let mut state = shared.lock().unwrap();
        let (controls, views) = close_attach_subscriptions(&mut state, "attach-1");
        let (second_controls, second_views) = close_attach_subscriptions(&mut state, "attach-2");
        assert!(state.attach_subscriptions.is_empty());
        assert!(state.attach_views.is_empty());
        (
            controls
                .into_iter()
                .chain(second_controls)
                .collect::<Vec<_>>(),
            views.into_iter().chain(second_views).collect::<Vec<_>>(),
        )
    };
    let _ = execute_controls(controls);
    for view in &views {
        view.close_and_join();
        assert!(view.worker.lock().unwrap().is_none());
    }
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn vue_lente_est_recoltee_sans_bloquer_le_daemon_ni_les_autres_vues() {
    let (mut state, config) = state_with_registered_agent("attach-slow-view");
    let (wrapper_stream, _wrapper_peer) = UnixStream::pair().unwrap();
    let (slow_stream, _slow_peer) = UnixStream::pair().unwrap();
    let (fast_stream, fast_peer) = UnixStream::pair().unwrap();
    fast_peer
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let wrapper_writer = Arc::new(Mutex::new(BufWriter::new(wrapper_stream)));
    let slow_writer = Arc::new(Mutex::new(BufWriter::new(slow_stream)));
    let fast_writer = Arc::new(Mutex::new(BufWriter::new(fast_stream)));
    state
        .connections
        .insert("conn-1".to_string(), wrapper_writer);
    state
        .connections
        .insert("attach-slow".to_string(), slow_writer.clone());
    state
        .connections
        .insert("attach-fast".to_string(), fast_writer.clone());
    install_attach_view(&mut state, "sub-slow", "attach-slow", &slow_writer);
    install_attach_view(&mut state, "sub-fast", "attach-fast", &fast_writer);
    let shared = Arc::new(Mutex::new(state));

    handle_wrapper_message(
        "conn-1",
        WrapperToDaemon::JournalFragment {
            subscription_id: "sub-fast".to_string(),
            seq: 1,
            offset: 0,
            final_fragment: true,
            bytes: b"temoin".to_vec(),
        },
        &shared,
    );
    let mut fast_reader = BufReader::new(fast_peer);
    assert!(matches!(
        read_control(&mut fast_reader),
        DaemonToWrapper::JournalFragment { subscription_id, seq: 1, .. } if subscription_id == "sub-fast"
    ));

    // Le pair lent n'est jamais lu : le worker doit atteindre son timeout
    // d'écriture, tandis que le chemin daemon garde le verrou disponible.
    for seq in 1..=512 {
        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::JournalFragment {
                subscription_id: "sub-slow".to_string(),
                seq,
                offset: seq * 4096,
                final_fragment: true,
                bytes: vec![b'x'; 4096],
            },
            &shared,
        );
    }
    let (observed_tx, observed_rx) = mpsc::channel();
    let observer_state = shared.clone();
    thread::spawn(move || {
        let count = observer_state.lock().unwrap().agent_infos().len();
        let _ = observed_tx.send(count);
    });
    assert_eq!(
        observed_rx
            .recv_timeout(Duration::from_millis(100))
            .unwrap(),
        1
    );

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        // Session 241 (db43c142) : le collecteur attach-cleanup récolte les
        // vues hors du traitement des messages ; même appel que lui ici.
        let (controls, views) = collect_closed_attach_views(&mut shared.lock().unwrap());
        let _ = execute_controls(controls);
        for view in views {
            view.close_and_join();
        }
        if !shared
            .lock()
            .unwrap()
            .attach_subscriptions
            .contains_key("sub-slow")
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "la vue lente doit etre fermee dans la borne"
        );
        thread::sleep(Duration::from_millis(10));
    }
    let state = shared.lock().unwrap();
    assert!(!state.attach_views.contains_key("sub-slow"));
    assert!(state.attach_subscriptions.contains_key("sub-fast"));
    drop(state);

    let (controls, views) = {
        let mut state = shared.lock().unwrap();
        close_attach_subscriptions(&mut state, "attach-fast")
    };
    let _ = execute_controls(controls);
    for view in views {
        view.close_and_join();
    }
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn runtime_observation_remplace_le_couple_en_bloc() {
    use bridget_transport::protocol::RuntimeSource;
    let (mut state, config) = state_with_registered_agent("runtime-atomique");

    // Observation initiale : un modèle qui expose son niveau d'effort.
    let ack = handle_runtime(
        "89000000-0000-4000-8000-000000000102",
        "claude-opus-5".to_string(),
        Some("high".to_string()),
        RuntimeSource::ClaudeHook,
        &mut state,
    );
    assert!(matches!(ack, DaemonToWrapper::Ack { .. }));
    let agents = state.agent_infos();
    assert_eq!(agents[0].model.as_deref(), Some("claude-opus-5"));
    assert_eq!(agents[0].effort.as_deref(), Some("high"));

    // Bascule vers un modèle sans niveau d'effort : l'ancien effort DOIT
    // disparaître. Le conserver afficherait « haiku + high », capacité qui
    // n'a jamais existé. Défaut soulevé par la contre-revue « agent-1 ».
    handle_runtime(
        "89000000-0000-4000-8000-000000000102",
        "claude-haiku-4-5-20251001".to_string(),
        None,
        RuntimeSource::ClaudeHook,
        &mut state,
    );
    let agents = state.agent_infos();
    assert_eq!(
        agents[0].model.as_deref(),
        Some("claude-haiku-4-5-20251001")
    );
    assert_eq!(agents[0].effort, None);

    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn limite_attestee_est_exposee_sans_decision_automatique() {
    use bridget_transport::protocol::RateLimitSource;
    let (mut state, config) = state_with_registered_agent("limite-attestee");

    let ack = handle_rate_limit(
        "89000000-0000-4000-8000-000000000102",
        "five_hour".to_string(),
        "rejected".to_string(),
        Some(1_787_572_200),
        None,
        None,
        RateLimitSource::ClaudeStreamJson,
        &mut state,
    );
    assert!(matches!(ack, DaemonToWrapper::Ack { .. }));
    let agent = state.agent_infos().pop().unwrap();
    assert_eq!(agent.state, "connected", "la limite ne change pas l'état");
    assert_eq!(agent.rate_limits.len(), 1);
    assert_eq!(agent.rate_limits[0].window, "five_hour");
    assert_eq!(agent.rate_limits[0].status, "rejected");
    assert_eq!(agent.rate_limits[0].resets_at, Some(1_787_572_200));

    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn fait_five_hour_n_efface_pas_fait_seven_day() {
    use bridget_transport::protocol::RateLimitSource;
    let (mut state, config) = state_with_registered_agent("limite-deux-fenetres");

    let first = handle_rate_limit(
        "89000000-0000-4000-8000-000000000102",
        "seven_day".to_string(),
        "allowed".to_string(),
        Some(1_787_700_000),
        Some(61),
        None,
        RateLimitSource::ClaudeStreamJson,
        &mut state,
    );
    assert!(matches!(first, DaemonToWrapper::Ack { .. }));
    let second = handle_rate_limit(
        "89000000-0000-4000-8000-000000000102",
        "five_hour".to_string(),
        "allowed".to_string(),
        Some(1_787_572_200),
        Some(19),
        None,
        RateLimitSource::ClaudeStreamJson,
        &mut state,
    );
    assert!(matches!(second, DaemonToWrapper::Ack { .. }));

    let agent = state.agent_infos().pop().unwrap();
    assert_eq!(agent.rate_limits.len(), 2, "les deux fenêtres coexistent");
    let windows: Vec<_> = agent
        .rate_limits
        .iter()
        .map(|fact| fact.window.as_str())
        .collect();
    assert!(windows.contains(&"five_hour"));
    assert!(windows.contains(&"seven_day"));
    let seven = agent
        .rate_limits
        .iter()
        .find(|fact| fact.window == "seven_day")
        .expect("7d conservée après upsert 5h");
    assert_eq!(seven.used_percent, Some(61));
    assert_eq!(seven.resets_at, Some(1_787_700_000));

    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn served_model_rend_l_ecart_visible_sans_inventer_si_muet() {
    let (mut state, config) = state_with_registered_agent("served-model-who");
    let presence = state.presences.values_mut().next().unwrap();
    presence.model = Some("claude-opus-5".to_string());
    presence.served_model = None;
    let silent = state.agent_infos().pop().unwrap();
    assert!(
        silent.model_mismatch.is_none(),
        "flux muet = pas de verdict"
    );

    let ack = handle_served_model(
        "89000000-0000-4000-8000-000000000102",
        "claude-opus-4-6".to_string(),
        &mut state,
    );
    assert!(matches!(ack, DaemonToWrapper::Ack { .. }));
    let agent = state.agent_infos().pop().unwrap();
    assert_eq!(agent.state, "connected", "l'écart ne change pas l'état");
    assert_eq!(agent.model.as_deref(), Some("claude-opus-5"));
    let gap = agent.model_mismatch.expect("écart visible");
    assert_eq!(gap.pinned, "claude-opus-5");
    assert_eq!(gap.served, "claude-opus-4-6");

    let match_ack = handle_served_model(
        "89000000-0000-4000-8000-000000000102",
        "claude-opus-5".to_string(),
        &mut state,
    );
    assert!(matches!(match_ack, DaemonToWrapper::Ack { .. }));
    let aligned = state.agent_infos().pop().unwrap();
    assert!(aligned.model_mismatch.is_none());

    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn usage_atteste_s_agrege_et_sans_source_reste_inconnu() {
    use bridget_transport::protocol::{UsageSource, UsageTokens};
    let (mut state, config) = state_with_registered_agent("usage-atteste");

    let ack = handle_usage(
        "89000000-0000-4000-8000-000000000102",
        UsageTokens {
            input_tokens: 2,
            output_tokens: 175,
            cache_creation_input_tokens: 40_804,
            cache_read_input_tokens: 13_907,
        },
        UsageSource::ClaudeStreamJson,
        None,
        None,
        None,
        &mut state,
    );
    assert!(matches!(ack, DaemonToWrapper::Ack { .. }));
    let window = handle_usage_window("89000000-0000-4000-8000-000000000102", 1, i64::MAX, &state);
    match window {
        DaemonToWrapper::UsageWindowResult {
            aggregate: Some(aggregate),
            ..
        } => {
            assert_eq!(aggregate.turns, 1);
            assert_eq!(aggregate.facturable_tokens, 40_981);
            assert_eq!(aggregate.cache_read_input_tokens, 13_907);
        }
        other => panic!("agrégat attesté attendu, reçu {other:?}"),
    }
    let unknown = handle_usage_window("tmux-sans-sonde", 1, i64::MAX, &state);
    assert!(matches!(
        unknown,
        DaemonToWrapper::UsageWindowResult {
            aggregate: None,
            ..
        }
    ));

    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn les_rappels_epargnent_un_agent_qui_ne_veut_pas_etre_derange() {
    // Destinataire joignable : tous les paliers passent.
    assert!(should_remind(false, 0));
    assert!(should_remind(false, 1));
    assert!(should_remind(false, 2));

    // Destinataire en « ne pas déranger » : les rappels qui lui sont
    // adressés sont retenus…
    assert!(!should_remind(true, 0));
    assert!(!should_remind(true, 1));
    // …mais pas la notification d'échec, qui part vers l'émetteur.
    assert!(should_remind(true, 2));
    assert!(should_remind(true, 3));
}

#[test]
fn domaine_surcharge_puis_reinitialise() {
    let (mut state, config) = state_with_registered_agent("domaine");
    // Domaine dérivé annoncé à l'enregistrement.
    if let Some(presence) = state.presences.get_mut("instance-1") {
        presence.derived_domain = Some("bridget".to_string());
        presence.domain = Some("bridget".to_string());
    }
    assert_eq!(state.agent_infos()[0].domain.as_deref(), Some("bridget"));

    let ack = handle_domain(
        "89000000-0000-4000-8000-000000000102",
        Some("revue-croisee".to_string()),
        &mut state,
    );
    assert!(matches!(ack, DaemonToWrapper::Ack { .. }));
    assert_eq!(
        state.agent_infos()[0].domain.as_deref(),
        Some("revue-croisee")
    );

    // La réinitialisation revient sur le domaine dérivé, pas sur rien.
    handle_domain("89000000-0000-4000-8000-000000000102", None, &mut state);
    assert_eq!(state.agent_infos()[0].domain.as_deref(), Some("bridget"));

    let nack = handle_domain("inconnu", None, &mut state);
    assert!(matches!(nack, DaemonToWrapper::Nack { .. }));

    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn dnd_expire_de_lui_meme_et_se_leve_a_la_demande() {
    let (mut state, config) = state_with_registered_agent("dnd");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Statut actif : l'état devient « dnd » et le temps restant est annoncé.
    handle_availability(
        "89000000-0000-4000-8000-000000000102",
        Some(now + 1800),
        &mut state,
    );
    assert_eq!(state.agent_infos()[0].state, "dnd");
    let presence = state.presences.get("instance-1").unwrap();
    assert!(presence.is_dnd());
    assert!(presence.dnd_minutes_left() <= 30);

    // Une commande `on` arrivée expirée ne doit pas se transformer en `off`.
    // L'expiration automatique reste portée par `is_dnd`, sans tâche de fond.
    let expired = handle_availability(
        "89000000-0000-4000-8000-000000000102",
        Some(now - 10),
        &mut state,
    );
    assert!(matches!(expired, DaemonToWrapper::Nack { .. }));
    assert_eq!(state.agent_infos()[0].state, "dnd");

    handle_availability(
        "89000000-0000-4000-8000-000000000102",
        Some(now + 600),
        &mut state,
    );
    assert_eq!(state.agent_infos()[0].state, "dnd");
    handle_availability("89000000-0000-4000-8000-000000000102", None, &mut state);
    assert_eq!(state.agent_infos()[0].state, "connected");
    assert!(!state.presences.get("instance-1").unwrap().is_dnd());

    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn runtime_refuse_hors_agent_enregistre_et_valeurs_invalides() {
    use bridget_transport::protocol::RuntimeSource;
    let (mut state, config) = state_with_registered_agent("runtime-refus");

    // Connexion inconnue du daemon.
    let nack = handle_runtime(
        "agent-inexistant",
        "modele".to_string(),
        None,
        RuntimeSource::Declared,
        &mut state,
    );
    assert!(
        matches!(nack, DaemonToWrapper::Nack { ref reason, .. } if reason.contains("introuvable"))
    );

    // Caractère de contrôle : casserait l'alignement de l'annuaire.
    let nack = handle_runtime(
        "89000000-0000-4000-8000-000000000102",
        "mod\u{1b}[31mele".to_string(),
        None,
        RuntimeSource::Declared,
        &mut state,
    );
    assert!(matches!(nack, DaemonToWrapper::Nack { .. }));

    // Valeur trop longue.
    let nack = handle_runtime(
        "89000000-0000-4000-8000-000000000102",
        "m".repeat(101),
        None,
        RuntimeSource::Declared,
        &mut state,
    );
    assert!(matches!(nack, DaemonToWrapper::Nack { .. }));

    // Aucun refus n'a pollué l'annuaire.
    assert_eq!(state.agent_infos()[0].model, None);

    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn runtime_survit_a_une_reconnexion_de_la_meme_instance() {
    use bridget_transport::protocol::RuntimeSource;
    let (mut state, config) = state_with_registered_agent("runtime-reconnexion");
    handle_runtime(
        "89000000-0000-4000-8000-000000000102",
        "gpt-5.3-codex".to_string(),
        Some("xhigh".to_string()),
        RuntimeSource::CodexRollout,
        &mut state,
    );

    // Coupure puis réenregistrement sous la même instance.
    state.router.unregister_by_conn("conn-1");
    state.mark_unreachable("conn-1");
    let response = handle_register(
        "conn-2",
        "codex".to_string(),
        Some("89000000-0000-4000-8000-000000000102".to_string()),
        Some("macbook".to_string()),
        Some("unix".to_string()),
        Some(PresenceMode::Acp),
        None,
        Some("macOS".to_string()),
        Some("instance-1".to_string()),
        Some("bridget".to_string()),
        false,
        None,
        &mut state,
    );
    assert!(matches!(response, DaemonToWrapper::Registered { .. }));

    let agents = state.agent_infos();
    assert_eq!(agents.len(), 1);
    assert_eq!(
        agents
            .iter()
            .map(|agent| agent.agent_id.as_str())
            .collect::<Vec<_>>(),
        vec!["89000000-0000-4000-8000-000000000102"]
    );
    assert_eq!(agents[0].state, "connected");
    assert_eq!(agents[0].model.as_deref(), Some("gpt-5.3-codex"));
    assert_eq!(agents[0].effort.as_deref(), Some("xhigh"));

    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn tour_busy_differe_les_rappels_sans_differe_l_echeance() {
    assert_eq!(deferred_reminder_level(true, 20, 60), Some(1));
    assert_eq!(deferred_reminder_level(true, 40, 60), Some(2));
    // La boucle traite elapsed >= T avant cet auxiliaire : busy ne peut
    // donc jamais différer le timeout du daemon.
    assert_eq!(deferred_reminder_level(true, 60, 60), Some(2));
    assert_eq!(deferred_reminder_level(false, 20, 60), None);
}

#[test]
fn rappel_reconnait_le_libelle_humain_du_relais_ui_canonique() {
    let root = std::env::temp_dir().join(format!(
        "bridget-rappel-ui-canonique-{}-{}",
        std::process::id(),
        Uuid::new_v4()
    ));
    let (mut state, config) = recovery_fixture_state(&root);
    let target = "61d32f19-3e02-4cbe-91a9-37d38066b4ab";
    assert!(matches!(
        handle_register_with_channel(
            "conn-1",
            2,
            "codex".to_string(),
            target.to_string(),
            Some("cartae".to_string()),
            Some("codex_app_server".to_string()),
            ChannelReport::Omitted,
            Some(PresenceMode::Cli),
            None,
            Some("Linux".to_string()),
            Some("target-instance".to_string()),
            Some("bridget".to_string()),
            false,
            Some(true),
            &mut state,
            false,
        ),
        DaemonToWrapper::Registered { .. }
    ));
    state.presences.insert(
        "instance-ui".to_string(),
        Presence {
            name: UI_HUMAN_AGENT_ID.to_string(),
            agent_type: "ui".to_string(),
            host: "localhost".to_string(),
            transport: "cli".to_string(),
            channel: Some("ssh-unix".to_string()),
            mode: None,
            location: None,
            journal_available: false,
            os: "linux".to_string(),
            state: "connected".to_string(),
            busy_since: None,
            busy_period_start: None,
            last_attested_at: None,
            turn_attest: false,
            queue_fallback: false,
            capacity_seen: Instant::now(),
            link_seen: Instant::now(),
            reconnect_count: 0,
            model: None,
            effort: None,
            rate_limits: Default::default(),
            served_model: None,
            derived_domain: None,
            domain: None,
            dnd_until: None,
            disk_space: None,
        },
    );
    state
        .store
        .create_request("requete-humaine", UI_HUMAN_SENDER, target, 60)
        .expect("demande suivie");
    let started = Instant::now()
        .checked_sub(Duration::from_secs(61))
        .expect("61 secondes en arrière");
    state.pending_replies.push(PendingReply {
        msg_id: "requete-humaine".to_string(),
        from: UI_HUMAN_SENDER.to_string(),
        from_conn: "ui-human".to_string(),
        to: target.to_string(),
        target_conn: "conn-1".to_string(),
        timeout_secs: 60,
        created_at: started,
        frozen_secs: 0,
        escalation_level: 0,
        deferred_level: None,
    });

    let actions = collect_reminder_actions(&mut state, Instant::now());
    assert!(
        actions.is_empty(),
        "une demande de l'interface ne doit pas créer de relance artificielle"
    );
    assert_eq!(
        state
            .store
            .get_request("requete-humaine")
            .expect("lecture demande")
            .expect("demande présente")
            .state,
        "open"
    );
    let _ = std::fs::remove_file(config.db_path);
    let _ = std::fs::remove_dir_all(root);
}

/// SPEC-087 T013 : en pause, les paliers 1 et 2 sont différés avec le
/// motif `pause`, le palier 3 reste émis ; la reprise libère le rappel.
/// Mutant : forcer `admit_autonomous_effect` à admettre rend ce test rouge.
#[test]
fn spec_087_pause_differe_les_relances_mais_pas_le_palier_trois() {
    let (mut state, config) = spec_087_state("pause-relances");
    state
        .store
        .create_request(
            "request-pause",
            "sender",
            "89000000-0000-4000-8000-000000000102",
            60,
        )
        .unwrap();
    let started = Instant::now();
    state.pending_replies.push(PendingReply {
        msg_id: "request-pause".to_string(),
        from: "sender".to_string(),
        from_conn: "conn-sender".to_string(),
        to: "89000000-0000-4000-8000-000000000102".to_string(),
        target_conn: "conn-1".to_string(),
        timeout_secs: 60,
        created_at: started,
        frozen_secs: 0,
        escalation_level: 0,
        deferred_level: None,
    });
    crate::referent_control::set(
        state.store.connection(),
        crate::referent_control::ControlMutation {
            command_id: "pause-relances",
            expected_generation: 0,
            paused: Some(true),
            auto_objectives_cap: None,
            reason: None,
            actor: "humain",
            now: 1,
            agent_posture: None,
            auto_reassignment: None,
        },
    )
    .unwrap()
    .unwrap();

    let first = collect_reminder_actions(&mut state, started + Duration::from_secs(20));
    assert!(
        first.iter().any(|action| matches!(
            action,
            ReminderAction::Deferred {
                level: 1,
                reason: "pause",
                ..
            }
        )),
        "palier 1 différé avec le motif pause"
    );
    assert!(
        !first
            .iter()
            .any(|action| matches!(action, ReminderAction::Gentle { .. })),
        "aucun rappel n'est émis en pause"
    );
    let second = collect_reminder_actions(&mut state, started + Duration::from_secs(41));
    assert!(
        !second
            .iter()
            .any(|action| matches!(action, ReminderAction::Firm { .. })),
        "le palier 2 non plus"
    );

    crate::referent_control::set(
        state.store.connection(),
        crate::referent_control::ControlMutation {
            command_id: "resume-relances",
            expected_generation: 1,
            paused: Some(false),
            auto_objectives_cap: None,
            reason: None,
            actor: "humain",
            now: 2,
            agent_posture: None,
            auto_reassignment: None,
        },
    )
    .unwrap()
    .unwrap();
    let resumed = collect_reminder_actions(&mut state, started + Duration::from_secs(42));
    assert!(
        resumed
            .iter()
            .any(|action| matches!(action, ReminderAction::Firm { .. })),
        "la reprise libère le rappel différé"
    );

    // Palier 3 : l'échéance reste notifiée à l'émetteur, même en pause.
    state
        .store
        .create_request(
            "request-pause-3",
            "sender",
            "89000000-0000-4000-8000-000000000102",
            60,
        )
        .unwrap();
    state.pending_replies.push(PendingReply {
        msg_id: "request-pause-3".to_string(),
        from: "sender".to_string(),
        from_conn: "conn-sender".to_string(),
        to: "89000000-0000-4000-8000-000000000102".to_string(),
        target_conn: "conn-1".to_string(),
        timeout_secs: 60,
        created_at: started,
        frozen_secs: 0,
        escalation_level: 0,
        deferred_level: None,
    });
    crate::referent_control::set(
        state.store.connection(),
        crate::referent_control::ControlMutation {
            command_id: "pause-relances-2",
            expected_generation: 2,
            paused: Some(true),
            auto_objectives_cap: None,
            reason: None,
            actor: "humain",
            now: 3,
            agent_posture: None,
            auto_reassignment: None,
        },
    )
    .unwrap()
    .unwrap();
    let expired = collect_reminder_actions(&mut state, started + Duration::from_secs(120));
    assert!(
        expired.iter().any(|action| matches!(
            action,
            ReminderAction::Timeout { msg_id, .. } if msg_id == "request-pause-3"
        )),
        "le palier 3 est une information, pas un réveil"
    );
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn boucle_busy_persiste_les_reports_et_expire_une_seule_fois() {
    let (mut state, config) = state_with_registered_agent("busy-boucle");
    state.set_turn_state("conn-1", true).unwrap();
    state
        .store
        .create_request(
            "request-busy",
            "sender",
            "89000000-0000-4000-8000-000000000102",
            60,
        )
        .unwrap();
    let started = Instant::now();
    state.pending_replies.push(PendingReply {
        msg_id: "request-busy".to_string(),
        from: "sender".to_string(),
        from_conn: "conn-sender".to_string(),
        to: "89000000-0000-4000-8000-000000000102".to_string(),
        target_conn: "conn-1".to_string(),
        timeout_secs: 60,
        created_at: started,
        frozen_secs: 0,
        escalation_level: 0,
        deferred_level: None,
    });

    let first = collect_reminder_actions(&mut state, started + Duration::from_secs(20));
    assert!(
        first
            .iter()
            .any(|action| matches!(action, ReminderAction::Deferred { level: 1, .. }))
    );
    assert!(!first.iter().any(|action| matches!(
        action,
        ReminderAction::Gentle { .. }
            | ReminderAction::Firm { .. }
            | ReminderAction::Timeout { .. }
    )));
    assert_eq!(
        state
            .store
            .latest_deferred_reminder("request-busy")
            .unwrap()
            .map(|event| event.0),
        Some(1)
    );

    let second = collect_reminder_actions(&mut state, started + Duration::from_secs(40));
    assert!(
        second
            .iter()
            .any(|action| matches!(action, ReminderAction::Deferred { level: 2, .. }))
    );
    assert!(!second.iter().any(|action| matches!(
        action,
        ReminderAction::Gentle { .. }
            | ReminderAction::Firm { .. }
            | ReminderAction::Timeout { .. }
    )));
    assert_eq!(
        state
            .store
            .latest_deferred_reminder("request-busy")
            .unwrap()
            .map(|event| event.0),
        Some(2)
    );

    let timeout = collect_reminder_actions(&mut state, started + Duration::from_secs(60));
    assert_eq!(
        timeout
            .iter()
            .filter(|action| matches!(action, ReminderAction::Timeout { .. }))
            .count(),
        1
    );
    assert_eq!(
        state
            .store
            .get_request("request-busy")
            .unwrap()
            .unwrap()
            .state,
        "timed_out"
    );
    let repeated = collect_reminder_actions(&mut state, started + Duration::from_secs(61));
    assert!(
        !repeated
            .iter()
            .any(|action| matches!(action, ReminderAction::Timeout { .. }))
    );
    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn spec099_t3code_aucun_rappel_ne_devient_un_tour_fournisseur() {
    let (mut state, _config) = state_with_registered_agent("spec099-t3-reminder");
    state.presences.get_mut("instance-1").unwrap().transport = "t3code".into();
    state
        .store
        .create_request(
            "spec099-t3-reminder",
            "sender",
            "89000000-0000-4000-8000-000000000102",
            60,
        )
        .unwrap();
    let started = Instant::now();
    state.pending_replies.push(PendingReply {
        msg_id: "spec099-t3-reminder".into(),
        from: "sender".into(),
        from_conn: "sender".into(),
        to: "89000000-0000-4000-8000-000000000102".into(),
        target_conn: "conn-1".into(),
        timeout_secs: 60,
        created_at: started,
        frozen_secs: 0,
        escalation_level: 0,
        deferred_level: None,
    });
    for seconds in [20, 40, 59] {
        let actions = collect_reminder_actions(&mut state, started + Duration::from_secs(seconds));
        assert!(!actions.iter().any(|action| matches!(
            action,
            ReminderAction::Gentle { .. } | ReminderAction::Firm { .. }
        )));
    }
    let actions = collect_reminder_actions(&mut state, started + Duration::from_secs(60));
    assert_eq!(
        actions
            .iter()
            .filter(|action| matches!(action,
            ReminderAction::Timeout { cancel_target: Some(conn), .. } if conn == "conn-1"))
            .count(),
        1
    );
}

#[test]
fn reconnexion_redeclare_busy_et_arret_propre_reste_stopped() {
    let (mut state, config) = state_with_registered_agent("tour-reconnexion");
    state.set_turn_state("conn-1", true).unwrap();
    let agents = state.agent_infos();
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0].agent_id, "89000000-0000-4000-8000-000000000102");
    assert_eq!(agents[0].state, "busy");
    state.router.unregister_by_conn("conn-1");
    state.mark_unreachable("conn-1");
    let response = handle_register(
        "conn-2",
        "claude".to_string(),
        Some("89000000-0000-4000-8000-000000000102".to_string()),
        Some("macbook".to_string()),
        Some("unix".to_string()),
        Some(PresenceMode::Acp),
        None,
        Some("macOS".to_string()),
        Some("instance-1".to_string()),
        None,
        true,
        None,
        &mut state,
    );
    assert!(matches!(response, DaemonToWrapper::Registered { .. }));
    assert_eq!(state.agent_infos()[0].state, "busy");
    state.router.unregister_by_conn("conn-2");
    state.mark_stopped("conn-2");
    assert_eq!(state.agent_infos()[0].state, "stopped");
    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn held159_adopt_adoption_a_l_enregistrement() {
    let label = format!(
        "held159_adopt_adoption_a_l_enregistrement-{}",
        Uuid::new_v4()
    );
    let (mut state, config) = state_with_registered_agent(&label);
    let target = state.presences.get("instance-1").unwrap().name.clone();
    let targets = [target.as_str()];
    let agent = state.presences.get("instance-1").unwrap().name.clone();
    let now = unix_now_secs();
    let sql = rusqlite::Connection::open(&config.db_path).unwrap();
    for (index, target) in targets.iter().enumerate() {
        let id = format!("held159-adopt-{index}");
        let key =
            IdempotencyKey::new("012_scope_held159_aaaaaaaa", OperationKind::Send, &id).unwrap();
        let mut message =
            bridget_core::BridgetMessage::new("author159", *target, "octets figés159\n");
        message.id = id.clone();
        let bytes = serde_json::to_vec(&message).unwrap();
        state
            .idempotency
            .reserve(
                &key,
                &bytes,
                now,
                CLIENT_IDEMPOTENCY_HORIZON_SECS,
                now,
                CLIENT_ISSUED_AT_TOLERANCE_SECS,
            )
            .unwrap();
        state
            .idempotency
            .transition(
                &key,
                crate::idempotency::RecordState::Prepared,
                crate::idempotency::RecordState::Dispatching,
            )
            .unwrap();
        sql.execute("INSERT INTO send_deliveries(delivery_id,issuer_scope,operation_kind,idempotency_key,recipient_instance_id,delivery_generation,phase,expires_at,message_bytes,recipient_agent) VALUES(?1,?2,'send',?1,?3,77,'held',?4,?5,?6)",rusqlite::params![id,key.issuer_scope,if *target==agent.as_str(){"instance-1"}else{"instance-other159"},now+CLIENT_IDEMPOTENCY_HORIZON_SECS,bytes,target]).unwrap();
    }
    let snapshot = || {
        sql.prepare("SELECT delivery_id,queue_rank,phase,message_bytes,recipient_agent,recipient_instance_id,delivery_generation FROM send_deliveries ORDER BY queue_rank").unwrap()
                .query_map([],|row|Ok((row.get::<_,String>(0)?,row.get::<_,i64>(1)?,row.get::<_,String>(2)?,row.get::<_,Vec<u8>>(3)?,row.get::<_,String>(4)?,row.get::<_,String>(5)?,row.get::<_,i64>(6)?)))
                .unwrap().collect::<Result<Vec<_>,_>>().unwrap()
    };
    let before = snapshot();
    state.router.unregister_by_conn("conn-1");
    state.mark_unreachable("conn-1");
    {
        let step = 1;
        let instance = "instance-held159-2".to_string();
        let response = handle_register(
            &format!("conn-held159-{step}"),
            "claude".into(),
            Some(agent.clone()),
            Some("macbook".into()),
            Some("acp".into()),
            Some(PresenceMode::Acp),
            None,
            Some("macOS".into()),
            Some(instance.clone()),
            None,
            false,
            Some(true),
            &mut state,
        );
        assert!(
            matches!(response, DaemonToWrapper::Registered { .. }),
            "{response:?}"
        );
        let after = snapshot();
        assert_eq!(before.len(), after.len());
        for (old, new) in before.iter().zip(&after) {
            assert_eq!(
                (&old.0, old.1, &old.2, &old.3, &old.4, old.6),
                (&new.0, new.1, &new.2, &new.3, &new.4, new.6),
                "phase/rang/canon/génération ne changent pas"
            );
            if old.4 == agent {
                assert_eq!(
                    new.5, instance,
                    "held adopté par l'identité actuelle après vrai Register"
                );
            } else {
                assert_eq!(new.5, old.5, "autre agent inchangé");
                assert_eq!(new, old);
            }
        }
        assert!(state.pending_post_response_controls.values().flatten().all(|control| !matches!(&control.message,DaemonToWrapper::DeliverIdempotent{delivery_id,..} if delivery_id.starts_with("held159-adopt-"))),"aucune remise d'un retenu lors de l'adoption");
    }
    drop(sql);
    drop(state);
    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn held159_adopt_adoption_sans_presence_apres_redemarrage_du_daemon() {
    let label = format!(
        "held159_adopt_adoption_sans_presence_apres_redemarrage_du_daemon-{}",
        Uuid::new_v4()
    );
    let (mut state, config) = state_with_registered_agent(&label);
    let target = state.presences.get("instance-1").unwrap().name.clone();
    let targets = [target.as_str()];
    let agent = state.presences.get("instance-1").unwrap().name.clone();
    let now = unix_now_secs();
    let sql = rusqlite::Connection::open(&config.db_path).unwrap();
    for (index, target) in targets.iter().enumerate() {
        let id = format!("held159-adopt-{index}");
        let key =
            IdempotencyKey::new("012_scope_held159_aaaaaaaa", OperationKind::Send, &id).unwrap();
        let mut message =
            bridget_core::BridgetMessage::new("author159", *target, "octets figés159\n");
        message.id = id.clone();
        let bytes = serde_json::to_vec(&message).unwrap();
        state
            .idempotency
            .reserve(
                &key,
                &bytes,
                now,
                CLIENT_IDEMPOTENCY_HORIZON_SECS,
                now,
                CLIENT_ISSUED_AT_TOLERANCE_SECS,
            )
            .unwrap();
        state
            .idempotency
            .transition(
                &key,
                crate::idempotency::RecordState::Prepared,
                crate::idempotency::RecordState::Dispatching,
            )
            .unwrap();
        sql.execute("INSERT INTO send_deliveries(delivery_id,issuer_scope,operation_kind,idempotency_key,recipient_instance_id,delivery_generation,phase,expires_at,message_bytes,recipient_agent) VALUES(?1,?2,'send',?1,?3,77,'held',?4,?5,?6)",rusqlite::params![id,key.issuer_scope,if *target==agent.as_str(){"instance-1"}else{"instance-other159"},now+CLIENT_IDEMPOTENCY_HORIZON_SECS,bytes,target]).unwrap();
    }
    let snapshot = || {
        sql.prepare("SELECT delivery_id,queue_rank,phase,message_bytes,recipient_agent,recipient_instance_id,delivery_generation FROM send_deliveries ORDER BY queue_rank").unwrap()
                .query_map([],|row|Ok((row.get::<_,String>(0)?,row.get::<_,i64>(1)?,row.get::<_,String>(2)?,row.get::<_,Vec<u8>>(3)?,row.get::<_,String>(4)?,row.get::<_,String>(5)?,row.get::<_,i64>(6)?)))
                .unwrap().collect::<Result<Vec<_>,_>>().unwrap()
    };
    let before = snapshot();
    for step in 1..=1 {
        let instance = format!("instance-held159-{}", step + 1);
        let fixture_root = state.fixture_root.take().unwrap();
        let registry_home = fixture_root.0.clone();
        drop(state);
        let home_lock = HOME_REGISTRY_LOCK.lock().recover_poison();
        let previous_home = std::env::var_os("HOME");
        let (managed_tx, _managed_rx) = mpsc::channel();
        // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
        unsafe {
            std::env::set_var("HOME", &registry_home);
        }
        let reopened = DaemonState::new(&config, managed_tx);
        if let Some(home) = previous_home {
            // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
            unsafe {
                std::env::set_var("HOME", home);
            }
        } else {
            // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
            unsafe {
                std::env::remove_var("HOME");
            }
        }
        drop(home_lock);
        state = reopened.unwrap();
        state.fixture_root = Some(fixture_root);
        assert!(
            state.presences.is_empty(),
            "redémarrage réel, pas de présence reconstruite artificiellement"
        );
        let response = handle_register(
            &format!("conn-held159-{step}"),
            "claude".into(),
            Some(agent.clone()),
            Some("macbook".into()),
            Some("acp".into()),
            Some(PresenceMode::Acp),
            None,
            Some("macOS".into()),
            Some(instance.clone()),
            None,
            false,
            Some(true),
            &mut state,
        );
        assert!(
            matches!(response, DaemonToWrapper::Registered { .. }),
            "{response:?}"
        );
        let after = snapshot();
        assert_eq!(before.len(), after.len());
        for (old, new) in before.iter().zip(&after) {
            assert_eq!(
                (&old.0, old.1, &old.2, &old.3, &old.4, old.6),
                (&new.0, new.1, &new.2, &new.3, &new.4, new.6),
                "phase/rang/canon/génération ne changent pas"
            );
            if old.4 == agent {
                assert_eq!(
                    new.5, instance,
                    "held adopté par l'identité actuelle après vrai Register"
                );
            } else {
                assert_eq!(new.5, old.5, "autre agent inchangé");
                assert_eq!(new, old);
            }
        }
        assert!(state.pending_post_response_controls.values().flatten().all(|control| !matches!(&control.message,DaemonToWrapper::DeliverIdempotent{delivery_id,..} if delivery_id.starts_with("held159-adopt-"))),"aucune remise d'un retenu lors de l'adoption");
    }
    drop(sql);
    drop(state);
    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn held159_adopt_double_redemarrage_nouvelle_instance() {
    let label = format!(
        "held159_adopt_double_redemarrage_nouvelle_instance-{}",
        Uuid::new_v4()
    );
    let (mut state, config) = state_with_registered_agent(&label);
    let target = state.presences.get("instance-1").unwrap().name.clone();
    let targets = [target.as_str()];
    let agent = state.presences.get("instance-1").unwrap().name.clone();
    let now = unix_now_secs();
    let sql = rusqlite::Connection::open(&config.db_path).unwrap();
    for (index, target) in targets.iter().enumerate() {
        let id = format!("held159-adopt-{index}");
        let key =
            IdempotencyKey::new("012_scope_held159_aaaaaaaa", OperationKind::Send, &id).unwrap();
        let mut message =
            bridget_core::BridgetMessage::new("author159", *target, "octets figés159\n");
        message.id = id.clone();
        let bytes = serde_json::to_vec(&message).unwrap();
        state
            .idempotency
            .reserve(
                &key,
                &bytes,
                now,
                CLIENT_IDEMPOTENCY_HORIZON_SECS,
                now,
                CLIENT_ISSUED_AT_TOLERANCE_SECS,
            )
            .unwrap();
        state
            .idempotency
            .transition(
                &key,
                crate::idempotency::RecordState::Prepared,
                crate::idempotency::RecordState::Dispatching,
            )
            .unwrap();
        sql.execute("INSERT INTO send_deliveries(delivery_id,issuer_scope,operation_kind,idempotency_key,recipient_instance_id,delivery_generation,phase,expires_at,message_bytes,recipient_agent) VALUES(?1,?2,'send',?1,?3,77,'held',?4,?5,?6)",rusqlite::params![id,key.issuer_scope,if *target==agent.as_str(){"instance-1"}else{"instance-other159"},now+CLIENT_IDEMPOTENCY_HORIZON_SECS,bytes,target]).unwrap();
    }
    let snapshot = || {
        sql.prepare("SELECT delivery_id,queue_rank,phase,message_bytes,recipient_agent,recipient_instance_id,delivery_generation FROM send_deliveries ORDER BY queue_rank").unwrap()
                .query_map([],|row|Ok((row.get::<_,String>(0)?,row.get::<_,i64>(1)?,row.get::<_,String>(2)?,row.get::<_,Vec<u8>>(3)?,row.get::<_,String>(4)?,row.get::<_,String>(5)?,row.get::<_,i64>(6)?)))
                .unwrap().collect::<Result<Vec<_>,_>>().unwrap()
    };
    let before = snapshot();
    for step in 1..=2 {
        let instance = format!("instance-held159-{}", step + 1);
        let fixture_root = state.fixture_root.take().unwrap();
        let registry_home = fixture_root.0.clone();
        drop(state);
        let home_lock = HOME_REGISTRY_LOCK.lock().recover_poison();
        let previous_home = std::env::var_os("HOME");
        let (managed_tx, _managed_rx) = mpsc::channel();
        // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
        unsafe {
            std::env::set_var("HOME", &registry_home);
        }
        let reopened = DaemonState::new(&config, managed_tx);
        if let Some(home) = previous_home {
            // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
            unsafe {
                std::env::set_var("HOME", home);
            }
        } else {
            // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
            unsafe {
                std::env::remove_var("HOME");
            }
        }
        drop(home_lock);
        state = reopened.unwrap();
        state.fixture_root = Some(fixture_root);
        assert!(
            state.presences.is_empty(),
            "redémarrage réel, pas de présence reconstruite artificiellement"
        );
        let response = handle_register(
            &format!("conn-held159-{step}"),
            "claude".into(),
            Some(agent.clone()),
            Some("macbook".into()),
            Some("acp".into()),
            Some(PresenceMode::Acp),
            None,
            Some("macOS".into()),
            Some(instance.clone()),
            None,
            false,
            Some(true),
            &mut state,
        );
        assert!(
            matches!(response, DaemonToWrapper::Registered { .. }),
            "{response:?}"
        );
        let after = snapshot();
        assert_eq!(before.len(), after.len());
        for (old, new) in before.iter().zip(&after) {
            assert_eq!(
                (&old.0, old.1, &old.2, &old.3, &old.4, old.6),
                (&new.0, new.1, &new.2, &new.3, &new.4, new.6),
                "phase/rang/canon/génération ne changent pas"
            );
            if old.4 == agent {
                assert_eq!(
                    new.5, instance,
                    "held adopté par l'identité actuelle après vrai Register"
                );
            } else {
                assert_eq!(new.5, old.5, "autre agent inchangé");
                assert_eq!(new, old);
            }
        }
        assert!(state.pending_post_response_controls.values().flatten().all(|control| !matches!(&control.message,DaemonToWrapper::DeliverIdempotent{delivery_id,..} if delivery_id.starts_with("held159-adopt-"))),"aucune remise d'un retenu lors de l'adoption");
    }
    drop(sql);
    drop(state);
    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn held159_adopt_autre_agent_non_touche() {
    let label = format!("held159_adopt_autre_agent_non_touche-{}", Uuid::new_v4());
    let (mut state, config) = state_with_registered_agent(&label);
    let target = state.presences.get("instance-1").unwrap().name.clone();
    let targets = [target.as_str(), "89000000-0000-4000-8000-000000000199"];
    let agent = state.presences.get("instance-1").unwrap().name.clone();
    let now = unix_now_secs();
    let sql = rusqlite::Connection::open(&config.db_path).unwrap();
    for (index, target) in targets.iter().enumerate() {
        let id = format!("held159-adopt-{index}");
        let key =
            IdempotencyKey::new("012_scope_held159_aaaaaaaa", OperationKind::Send, &id).unwrap();
        let mut message =
            bridget_core::BridgetMessage::new("author159", *target, "octets figés159\n");
        message.id = id.clone();
        let bytes = serde_json::to_vec(&message).unwrap();
        state
            .idempotency
            .reserve(
                &key,
                &bytes,
                now,
                CLIENT_IDEMPOTENCY_HORIZON_SECS,
                now,
                CLIENT_ISSUED_AT_TOLERANCE_SECS,
            )
            .unwrap();
        state
            .idempotency
            .transition(
                &key,
                crate::idempotency::RecordState::Prepared,
                crate::idempotency::RecordState::Dispatching,
            )
            .unwrap();
        sql.execute("INSERT INTO send_deliveries(delivery_id,issuer_scope,operation_kind,idempotency_key,recipient_instance_id,delivery_generation,phase,expires_at,message_bytes,recipient_agent) VALUES(?1,?2,'send',?1,?3,77,'held',?4,?5,?6)",rusqlite::params![id,key.issuer_scope,if *target==agent.as_str(){"instance-1"}else{"instance-other159"},now+CLIENT_IDEMPOTENCY_HORIZON_SECS,bytes,target]).unwrap();
    }
    let snapshot = || {
        sql.prepare("SELECT delivery_id,queue_rank,phase,message_bytes,recipient_agent,recipient_instance_id,delivery_generation FROM send_deliveries ORDER BY queue_rank").unwrap()
                .query_map([],|row|Ok((row.get::<_,String>(0)?,row.get::<_,i64>(1)?,row.get::<_,String>(2)?,row.get::<_,Vec<u8>>(3)?,row.get::<_,String>(4)?,row.get::<_,String>(5)?,row.get::<_,i64>(6)?)))
                .unwrap().collect::<Result<Vec<_>,_>>().unwrap()
    };
    let before = snapshot();
    state.router.unregister_by_conn("conn-1");
    state.mark_unreachable("conn-1");
    {
        let step = 1;
        let instance = "instance-held159-2".to_string();
        let response = handle_register(
            &format!("conn-held159-{step}"),
            "claude".into(),
            Some(agent.clone()),
            Some("macbook".into()),
            Some("acp".into()),
            Some(PresenceMode::Acp),
            None,
            Some("macOS".into()),
            Some(instance.clone()),
            None,
            false,
            Some(true),
            &mut state,
        );
        assert!(
            matches!(response, DaemonToWrapper::Registered { .. }),
            "{response:?}"
        );
        let after = snapshot();
        assert_eq!(before.len(), after.len());
        for (old, new) in before.iter().zip(&after) {
            assert_eq!(
                (&old.0, old.1, &old.2, &old.3, &old.4, old.6),
                (&new.0, new.1, &new.2, &new.3, &new.4, new.6),
                "phase/rang/canon/génération ne changent pas"
            );
            if old.4 == agent {
                assert_eq!(
                    new.5, instance,
                    "held adopté par l'identité actuelle après vrai Register"
                );
            } else {
                assert_eq!(new.5, old.5, "autre agent inchangé");
                assert_eq!(new, old);
            }
        }
        assert!(state.pending_post_response_controls.values().flatten().all(|control| !matches!(&control.message,DaemonToWrapper::DeliverIdempotent{delivery_id,..} if delivery_id.starts_with("held159-adopt-"))),"aucune remise d'un retenu lors de l'adoption");
    }
    drop(sql);
    drop(state);
    let _ = std::fs::remove_file(&config.db_path);
}

/// Oracle (iii) bout-en-bout Register : un nouvel instance_id pour le même
/// nom reprend les remises `dispatching` de l'instance morte.
#[test]
fn register_migre_les_remises_dispatching_de_l_ancienne_instance() {
    let (mut state, config) = state_with_registered_agent("migrate-dispatching");
    let key = IdempotencyKey::new(
        "012_scope_bbbbbbbbbbbb",
        OperationKind::Send,
        "msg-orphan-register",
    )
    .unwrap();
    let now = unix_now_secs();
    let mut message = bridget_core::BridgetMessage::new(
        "peer-a",
        "89000000-0000-4000-8000-000000000102",
        "collège en vol",
    );
    message.id = "msg-orphan-register".to_string();
    state
        .idempotency
        .reserve(
            &key,
            b"orphan-register",
            now,
            CLIENT_IDEMPOTENCY_HORIZON_SECS,
            now,
            CLIENT_ISSUED_AT_TOLERANCE_SECS,
        )
        .unwrap();
    state
        .idempotency
        .begin_send_delivery(
            &key,
            &SendDelivery {
                delivery_id: "delivery-orphan-register".to_string(),
                recipient_instance_id: "instance-1".to_string(),
                delivery_generation: 11,
                expires_at: now + CLIENT_IDEMPOTENCY_HORIZON_SECS,
                message_bytes: serde_json::to_vec(&message).unwrap(),
            },
        )
        .unwrap();
    assert_eq!(
        state
            .idempotency
            .dispatching_deliveries_for_instance("instance-1", now)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        state
            .store
            .recent_messages(20)
            .unwrap()
            .iter()
            .filter(|entry| entry.id == "msg-orphan-register")
            .count(),
        1,
        "visibilité d'émission avant respawn"
    );

    state.router.unregister_by_conn("conn-1");
    state.mark_unreachable("conn-1");
    let response = handle_register(
        "conn-new",
        "claude".to_string(),
        Some("89000000-0000-4000-8000-000000000102".to_string()),
        Some("macbook".to_string()),
        Some("acp".to_string()),
        Some(PresenceMode::Acp),
        None,
        Some("macOS".to_string()),
        Some("instance-2".to_string()),
        None,
        false,
        Some(true),
        &mut state,
    );
    assert!(matches!(response, DaemonToWrapper::Registered { .. }));
    assert!(
        !state.presences.contains_key("instance-1"),
        "l'ancienne instance doit être retirée après migration"
    );
    assert!(state.presences.contains_key("instance-2"));
    assert_eq!(
        state
            .idempotency
            .dispatching_deliveries_for_instance("instance-1", now)
            .unwrap()
            .len(),
        0
    );
    let revived = state
        .idempotency
        .dispatching_deliveries_for_instance("instance-2", now)
        .unwrap();
    assert_eq!(revived.len(), 1);
    assert_eq!(revived[0].delivery_id, "delivery-orphan-register");
    assert_eq!(revived[0].recipient_instance_id, "instance-2");
    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn spec_079_reprise_register_livre_le_message_exact_une_seule_fois() {
    let (mut state, config) = state_with_registered_agent("spec-079-recovery-register");
    let (writer, _reader) = control_socket("spec-079-recovery-register");
    state.connections.insert("conn-1".to_string(), writer);
    let mut message = bridget_core::BridgetMessage::new(
        "humain",
        "89000000-0000-4000-8000-000000000102",
        "reprends exactement ceci",
    );
    message.id = "message-recovery-079".to_string();
    message.origin = Some(bridget_core::MessageOrigin::Human);
    message.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    state
        .execution_store
        .admit_starting_message(&message, "execution-parent-079", unix_now_secs())
        .unwrap();

    schedule_execution_recovery(
        &mut state,
        "conn-1",
        "instance-1",
        "89000000-0000-4000-8000-000000000102",
        false,
    );

    let controls = state
        .pending_post_response_controls
        .get("conn-1")
        .expect("remise de reprise différée");
    assert_eq!(controls.len(), 1);
    let child_execution_id = match &controls[0].message {
        DaemonToWrapper::DeliverIdempotent {
            message: delivered,
            execution: Some(execution),
            ..
        } => {
            assert_eq!(delivered.id, message.id);
            assert_eq!(delivered.body, message.body);
            assert_eq!(delivered.origin, message.origin);
            assert_eq!(delivered.intent, message.intent);
            assert_eq!(execution.generation, 2);
            execution.execution_id.clone()
        }
        other => panic!("remise liée attendue, reçu {other:?}"),
    };
    assert!(matches!(
        state
            .execution_store
            .execution_snapshot("execution-parent-079")
            .unwrap(),
        Some(snapshot) if snapshot.state == "unreachable"
    ));
    assert!(matches!(
        state
            .execution_store
            .continuation_for(&child_execution_id)
            .unwrap(),
        Some(continuation)
            if continuation.parent_execution_id == "execution-parent-079"
                && continuation.mode
                    == crate::execution_store::ContinuationMode::Reconstructed
    ));

    schedule_execution_recovery(
        &mut state,
        "conn-1",
        "instance-1",
        "89000000-0000-4000-8000-000000000102",
        false,
    );
    assert_eq!(
        state.pending_post_response_controls["conn-1"].len(),
        1,
        "la remise dispatching existante doit être rejouée, pas remplacée"
    );
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn spec_079_ack_puis_deux_redemarrages_rejouent_une_seule_continuation() {
    fn reopen_fixture_state(config: &DaemonConfig, home: &std::path::Path) -> DaemonState {
        // Un test qui panique en tenant ce verrou ne doit pas empoisonner les
        // suivants : on relit le verrou empoisonné plutôt que de propager.
        let _home_lock = HOME_REGISTRY_LOCK.lock().recover_poison();
        let previous_home = std::env::var_os("HOME");
        // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
        unsafe { std::env::set_var("HOME", home) };
        let (managed_tx, _managed_rx) = mpsc::channel();
        let result = DaemonState::new(config, managed_tx);
        if let Some(previous_home) = previous_home {
            // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
            unsafe { std::env::set_var("HOME", previous_home) };
        } else {
            // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
            unsafe { std::env::remove_var("HOME") };
        }
        result.unwrap()
    }

    let (mut state, config) = state_with_registered_agent("spec-079-real-restart");
    let fixture_root = state.fixture_root.take().expect("racine fixture");
    let home = fixture_root.0.clone();
    let registry_file = home.join(".config/bridget/agents.json");
    let registry_json = serde_json::json!({
        "agents": {
            "claude": {
                "command": "/bin/sh",
                "protocol": "acp",
                "forbidden_env": [],
                "pass_env": []
            }
        }
    })
    .to_string();
    std::fs::write(&registry_file, &registry_json).unwrap();
    std::fs::set_permissions(&registry_file, std::fs::Permissions::from_mode(0o600)).unwrap();
    state.registry = AgentRegistry::from_json(&registry_json, &registry_file).unwrap();
    state.client_negotiations.insert(
        "ui-client-079".to_string(),
        NegotiatedClient {
            version: CLIENT_CONTRACT_VERSION,
            issuer_scope: crate::communication::issuer_scope("ui-client-real-restart-079"),
            capabilities: vec![ClientCapability::SendIdempotent],
        },
    );
    let (initial_writer, _initial_reader) = control_socket("spec-079-real-restart-initial");
    state
        .connections
        .insert("conn-1".to_string(), initial_writer);
    // Spec 099 : le client UI envoie au nom d'un agent attesté, rattaché
    // en auxiliaire avec la preuve de son propriétaire vivant.
    const SENDER: &str = "89000000-0000-4000-8000-000000000079";
    let _sender_owner = super::spec094_live_test_connection(&mut state, "owner-079");
    let DaemonToWrapper::Registered {
        credential: Some(sender_credential),
        ..
    } = handle_register_with_channel(
        "owner-079",
        2,
        "fixture".into(),
        SENDER.into(),
        Some("fixture".into()),
        Some("acp".into()),
        ChannelReport::Known("unix".into()),
        Some(PresenceMode::Acp),
        None,
        Some("test".into()),
        Some("instance-079".into()),
        None,
        false,
        Some(false),
        &mut state,
        false,
    )
    else {
        panic!("enregistrement de l'expéditeur attendu");
    };
    assert!(matches!(
        register_auxiliary(
            &mut state,
            "ui-client-079",
            SENDER.into(),
            "instance-079".into(),
            sender_credential,
        ),
        DaemonToWrapper::Registered {
            credential: None,
            ..
        }
    ));

    let now = unix_now_secs();
    let mut message = bridget_core::BridgetMessage::new(
        SENDER,
        "89000000-0000-4000-8000-000000000102",
        "message exact après acquittement",
    );
    message.id = "message-real-restart-079".to_string();
    message.origin = Some(bridget_core::MessageOrigin::Human);
    message.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    let mut controls = Vec::new();
    let response = handle_idempotent_send(
        "ui-client-079",
        message.clone(),
        message.id.clone(),
        now,
        IdempotentSendAdmission {
            project: None,
            issued_at_tolerance_secs: CLIENT_ISSUED_AT_TOLERANCE_SECS,
        },
        &mut state,
        &mut controls,
    );
    assert!(matches!(
        response,
        DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::OutcomeUnknown {
                delivery_id: Some(_),
                ..
            },
            ..
        }
    ));
    assert_eq!(controls.len(), 1);
    let (initial_delivery_id, initial_generation) = match &controls[0].message {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            message: delivered,
            execution: Some(execution),
            ..
        } => {
            assert_eq!(delivered.body, message.body);
            assert_eq!(execution.execution_id, "execution-message-real-restart-079");
            (delivery_id.clone(), *delivery_generation)
        }
        other => panic!("remise initiale liée attendue: {other:?}"),
    };
    state
        .idempotency
        .acknowledge_send_delivery(&initial_delivery_id, "instance-1", initial_generation)
        .unwrap();
    drop(state);

    let mut restarted = reopen_fixture_state(&config, &home);
    restarted.fixture_root = Some(fixture_root);
    let (restart_writer, _restart_reader) = control_socket("spec-079-real-restart-first");
    restarted
        .connections
        .insert("conn-restart-1".to_string(), restart_writer);
    assert!(matches!(
        handle_register(
            "conn-restart-1",
            "claude".to_string(),
            Some("89000000-0000-4000-8000-000000000102".to_string()),
            Some("macbook".to_string()),
            Some("acp".to_string()),
            Some(PresenceMode::Acp),
            None,
            Some("Linux".to_string()),
            Some("instance-restart-1".to_string()),
            None,
            false,
            Some(true),
            &mut restarted,
        ),
        DaemonToWrapper::Registered { .. }
    ));
    let first_controls = restarted
        .pending_post_response_controls
        .get("conn-restart-1")
        .expect("continuation après premier redémarrage");
    assert_eq!(first_controls.len(), 1);
    let (recovery_delivery_id, recovery_execution_id) = match &first_controls[0].message {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            message: delivered,
            execution: Some(execution),
            ..
        } => {
            assert_eq!(delivered.id, message.id);
            assert_eq!(delivered.body, message.body);
            (delivery_id.clone(), execution.execution_id.clone())
        }
        other => panic!("continuation liée attendue: {other:?}"),
    };
    assert!(matches!(
        restarted
            .execution_store
            .continuation_for(&recovery_execution_id)
            .unwrap(),
        Some(continuation)
            if continuation.parent_execution_id
                == "execution-message-real-restart-079"
                && continuation.mode
                    == crate::execution_store::ContinuationMode::Reconstructed
    ));

    let fixture_root = restarted.fixture_root.take().expect("racine conservée");
    drop(restarted);
    let mut restarted_again = reopen_fixture_state(&config, &home);
    restarted_again.fixture_root = Some(fixture_root);
    let recoverable = restarted_again
        .execution_store
        .recoverable_execution_ids_for_agent("89000000-0000-4000-8000-000000000102")
        .unwrap();
    assert_eq!(
        recoverable.as_slice(),
        std::slice::from_ref(&recovery_execution_id)
    );
    assert_eq!(
        restarted_again
            .idempotency
            .dispatching_deliveries_for_instance("instance-restart-1", unix_now_secs(),)
            .unwrap()
            .len(),
        1
    );
    assert!(
        restarted_again
            .idempotency
            .dispatching_delivery_for_execution(
                &recovery_execution_id,
                "instance-restart-1",
                unix_now_secs(),
            )
            .unwrap()
            .is_some()
    );
    let (second_writer, _second_reader) = control_socket("spec-079-real-restart-second");
    restarted_again
        .connections
        .insert("conn-restart-2".to_string(), second_writer);
    assert!(matches!(
        handle_register(
            "conn-restart-2",
            "claude".to_string(),
            Some("89000000-0000-4000-8000-000000000102".to_string()),
            Some("macbook".to_string()),
            Some("acp".to_string()),
            Some(PresenceMode::Acp),
            None,
            Some("Linux".to_string()),
            Some("instance-restart-2".to_string()),
            None,
            false,
            Some(true),
            &mut restarted_again,
        ),
        DaemonToWrapper::Registered { .. }
    ));
    let second_controls = restarted_again
        .pending_post_response_controls
        .get("conn-restart-2")
        .expect("rejeu après second redémarrage");
    assert_eq!(second_controls.len(), 1);
    match &second_controls[0].message {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            message: delivered,
            execution: Some(execution),
            ..
        } => {
            assert_eq!(delivery_id, &recovery_delivery_id);
            assert_eq!(delivered.body, message.body);
            assert_eq!(execution.execution_id, recovery_execution_id);
        }
        other => panic!("rejeu de continuation liée attendu: {other:?}"),
    }

    drop(restarted_again);
    let _ = std::fs::remove_file(config.db_path);
    let _ = std::fs::remove_file(config.socket_path);
    let _ = std::fs::remove_file(config.log_path);
}

#[test]
fn spec_079_tour_vivant_interdit_la_reconstruction() {
    let (mut state, config) = state_with_registered_agent("spec-079-recovery-busy");
    let mut message = bridget_core::BridgetMessage::new(
        "humain",
        "89000000-0000-4000-8000-000000000102",
        "tour vivant",
    );
    message.id = "message-busy-079".to_string();
    message.origin = Some(bridget_core::MessageOrigin::Human);
    message.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    state
        .execution_store
        .admit_starting_message(&message, "execution-busy-079", unix_now_secs())
        .unwrap();

    schedule_execution_recovery(
        &mut state,
        "conn-1",
        "instance-1",
        "89000000-0000-4000-8000-000000000102",
        true,
    );

    assert!(state.pending_post_response_controls.is_empty());
    assert!(matches!(
        state
            .execution_store
            .execution_snapshot("execution-busy-079")
            .unwrap(),
        Some(snapshot) if snapshot.state == "starting" && snapshot.generation == 1
    ));
    assert_eq!(
        state
            .idempotency
            .dispatching_delivery_for_execution("execution-busy-079", "instance-1", unix_now_secs())
            .unwrap(),
        None
    );
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn spec_091_scoped_spawn_requires_human_and_preserves_global_discovery() {
    use bridget_transport::protocol::{AgentPosture, SpawnPosture};
    let (mut state, config) = spec_087_state("spec-091-scoped-spawn");
    state.registry = AgentRegistry::from_json(
            r#"{"agents":{"codex":{"command":"/bin/sh","args":["app-server"],"protocol":"codex_app_server","capabilities":{"execution_paths":["codex_app_server"],"models":{}}}}}"#,
            "/tmp/registry-091.json",
        ).unwrap();
    let (tx, rx) = mpsc::channel();
    state.managed_tx = tx;
    let initial = crate::referent_control::read(state.store.connection()).unwrap();
    assert_eq!(initial.agent_posture, Some(AgentPosture::Discovery));
    let shared = Arc::new(Mutex::new(state));
    let order = WrapperToDaemon::SpawnOrder {
        posture: Some(SpawnPosture::Development),
        agent_type: "codex".to_string(),
        project: None,
        ownership: None,
        agent_id: Some(uuid::Uuid::new_v4().to_string()),
        cwd: std::env::temp_dir().to_string_lossy().into_owned(),
        persistent: false,
        command_id: "scoped-development-091".to_string(),
        issued_at: unix_timestamp(),
        deadline_at: unix_timestamp() + 60,
    };
    for (conn, issuer, capabilities) in [
        (
            "untrusted",
            "agent-091",
            vec![ClientCapability::ControlStateV1],
        ),
        ("no-capability", "bridget-control-cli", vec![]),
        (
            "human",
            "bridget-control-cli",
            vec![ClientCapability::ControlStateV1],
        ),
    ] {
        assert!(matches!(
            handle_wrapper_message(
                conn,
                WrapperToDaemon::RoleHandshake {
                    role: ConnectionRole::Client
                },
                &shared
            ),
            Some(DaemonToWrapper::RoleAccepted { .. })
        ));
        assert!(matches!(
            handle_wrapper_message(
                conn,
                WrapperToDaemon::ClientHello {
                    contract_version: CLIENT_CONTRACT_VERSION,
                    issuer_scope: crate::communication::issuer_scope(issuer),
                    capabilities,
                },
                &shared
            ),
            Some(DaemonToWrapper::ClientWelcome { .. })
        ));
        let result = handle_wrapper_message(conn, order.clone(), &shared);
        if conn != "human" {
            assert!(
                matches!(result, Some(DaemonToWrapper::SpawnRejected { reason: SpawnRefusal::UnsupportedCapability { capability, .. }, .. }) if capability == "posture_development_human_principal_required")
            );
            assert!(rx.try_recv().is_err());
        } else {
            assert!(result.is_none(), "{result:?}");
        }
    }
    let ManagedSupervisorCommand::Start { prepared, .. } = rx.try_recv().unwrap() else {
        panic!("spawn attendu")
    };
    assert_eq!(prepared.resolved_definition.permissions, "deny");
    assert!(
        prepared
            .resolved_definition
            .args
            .windows(2)
            .any(|pair| pair == ["--sandbox", "workspace-write"])
    );
    let original_registry = shared.lock().unwrap().registry.clone();
    shared.lock().unwrap().registry = AgentRegistry::from_json(
        r#"{"agents":{"codex":{"command":"/bin/sh","protocol":"acp"}}}"#,
        "/tmp/registry-changed-091.json",
    )
    .unwrap();
    assert!(handle_wrapper_message("human", order.clone(), &shared).is_none());
    assert!(
        rx.try_recv().is_err(),
        "le rejeu ne crée pas un second processus"
    );
    let mut divergent = order.clone();
    if let WrapperToDaemon::SpawnOrder { posture, .. } = &mut divergent {
        *posture = Some(SpawnPosture::Discovery);
    }
    assert!(matches!(
        handle_wrapper_message("human", divergent, &shared),
        Some(DaemonToWrapper::IdempotencyResult {
            issue: IdempotencyIssue::EnvelopeMismatch,
            ..
        })
    ));
    shared.lock().unwrap().registry = original_registry;
    let mut ordinary = order;
    if let WrapperToDaemon::SpawnOrder {
        posture,
        command_id,
        agent_id,
        ..
    } = &mut ordinary
    {
        *posture = None;
        *command_id = "ordinary-091".to_string();
        *agent_id = Some(uuid::Uuid::new_v4().to_string());
    }
    assert!(handle_wrapper_message("wrapper", ordinary, &shared).is_none());
    let ManagedSupervisorCommand::Start { prepared, .. } = rx.try_recv().unwrap() else {
        panic!("spawn ordinaire attendu")
    };
    assert_eq!(prepared.agent_type, "project-discovery-codex");
    assert!(
        prepared
            .resolved_definition
            .args
            .windows(2)
            .any(|pair| pair == ["--sandbox", "read-only"])
    );
    assert_eq!(
        crate::referent_control::read(shared.lock().unwrap().store.connection()).unwrap(),
        initial
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

/// SPEC-088 T015 : posture découverte ⇒ définition de découverte ; type
/// sans définition attestée ⇒ refus explicite, jamais de repli complet.
#[test]
fn spec_088_posture_decouverte_choisit_la_definition_ou_refuse() {
    use bridget_transport::protocol::AgentPosture;
    let (mut state, config) = spec_087_state("spec-088-posture");
    // Registre de test : « claude » en protocole acp, sans découverte
    // attestée ; « codex » en app-server, avec découverte.
    state.registry = AgentRegistry::from_json(
            &serde_json::json!({
                "agents": {
                    "claude": { "command": "/bin/sh", "protocol": "acp", "forbidden_env": [], "pass_env": [] },
                    "codex": { "command": "/bin/sh", "args": ["app-server"], "protocol": "codex_app_server", "forbidden_env": [], "pass_env": [] }
                }
            })
            .to_string(),
            "/tmp/agents-spec-088.json",
        )
        .unwrap();
    let now = unix_now_secs();
    let initial = crate::referent_control::read(state.store.connection()).unwrap();
    // Base neuve : Prudent ⇒ découverte.
    assert_eq!(initial.agent_posture, Some(AgentPosture::Discovery));
    assert_eq!(
        resolve_spawn_agent_type_for_posture(&state, "codex").unwrap(),
        "project-discovery-codex"
    );
    assert!(matches!(
        resolve_spawn_agent_type_for_posture(&state, "claude"),
        Err(SpawnRefusal::UnsupportedCapability { ref capability, ref agent_type, .. })
            if capability == "posture_decouverte" && agent_type == "claude"
    ));
    // Session 127 : un type absent du registre est annoncé inconnu, avec la
    // liste des types connus, et non comme une capacité manquante.
    assert!(matches!(
        resolve_spawn_agent_type_for_posture(&state, "gclaude"),
        Err(SpawnRefusal::UnknownType { ref requested_type, ref known_types, .. })
            if requested_type == "gclaude" && known_types.iter().any(|t| t == "codex")
    ));
    // Posture complète ⇒ le type demandé, tel quel.
    crate::referent_control::set(
        state.store.connection(),
        crate::referent_control::ControlMutation {
            command_id: "posture-complete-088",
            expected_generation: initial.generation,
            paused: None,
            auto_objectives_cap: None,
            reason: None,
            actor: "test",
            now,
            agent_posture: Some(AgentPosture::Complete),
            auto_reassignment: None,
        },
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        resolve_spawn_agent_type_for_posture(&state, "claude").unwrap(),
        "claude"
    );
    assert_eq!(
        resolve_spawn_agent_type_for_posture(&state, "codex").unwrap(),
        "codex"
    );
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn spec_087_pause_differe_la_continuation_de_reprise() {
    let (mut state, config) = spec_087_state("spec-087-continuation-pause");
    let (writer, _reader) = control_socket("spec-087-continuation-pause");
    state.connections.insert("conn-1".to_string(), writer);
    let now = unix_now_secs();
    let mut message = bridget_core::BridgetMessage::new(
        "humain",
        SPEC_087_AGENT_ID,
        "reprendre seulement après la pause",
    );
    message.id = "message-continuation-pause-087".to_string();
    message.origin = Some(bridget_core::MessageOrigin::Human);
    message.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    state
        .execution_store
        .admit_starting_message(&message, "execution-continuation-pause-087", now)
        .unwrap();

    let initial = crate::referent_control::read(state.store.connection()).unwrap();
    let paused = crate::referent_control::set(
        state.store.connection(),
        crate::referent_control::ControlMutation {
            command_id: "pause-continuation-087",
            expected_generation: initial.generation,
            paused: Some(true),
            auto_objectives_cap: None,
            reason: Some("test"),
            actor: "test",
            now,
            agent_posture: None,
            auto_reassignment: None,
        },
    )
    .unwrap()
    .unwrap();

    resume_executions_after_pause(&mut state);
    assert!(state.pending_post_response_controls.is_empty());
    assert!(matches!(
        state
            .execution_store
            .execution_snapshot("execution-continuation-pause-087")
            .unwrap(),
        Some(snapshot) if snapshot.state == "starting" && snapshot.generation == 1
    ));

    crate::referent_control::set(
        state.store.connection(),
        crate::referent_control::ControlMutation {
            command_id: "resume-continuation-087",
            expected_generation: paused.generation,
            paused: Some(false),
            auto_objectives_cap: None,
            reason: None,
            actor: "test",
            now: now + 1,
            agent_posture: None,
            auto_reassignment: None,
        },
    )
    .unwrap()
    .unwrap();
    schedule_execution_recovery(&mut state, "conn-1", "instance-1", SPEC_087_AGENT_ID, false);
    let controls = &state.pending_post_response_controls["conn-1"];
    assert_eq!(controls.len(), 1);
    assert!(matches!(
        &controls[0].message,
        DaemonToWrapper::DeliverIdempotent {
            execution: Some(execution),
            ..
        } if execution.generation == 2
    ));
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn spec_087_levee_de_pause_relance_l_execution_interrompue() {
    let (mut state, config) = spec_087_state("spec-087-pause-resume");
    let (writer, mut reader) = control_socket("spec-087-pause-resume");
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(250)))
        .unwrap();
    state.connections.insert("conn-1".to_string(), writer);
    spec_087_register_human_control_client(&mut state);
    let now = unix_now_secs();
    let mut message =
        bridget_core::BridgetMessage::new("humain", SPEC_087_AGENT_ID, "reprends moi");
    message.id = "message-pause-resume-087".to_string();
    message.origin = Some(bridget_core::MessageOrigin::Human);
    message.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    state
        .execution_store
        .admit_starting_message(&message, "execution-pause-resume-087", now)
        .unwrap();
    state
        .execution_store
        .transition_if_current(
            "execution-pause-resume-087",
            "starting",
            0,
            1,
            "interrupted",
            "control_pause",
            now,
        )
        .unwrap();
    state
        .execution_store
        .record_pause_interruption("execution-pause-resume-087", SPEC_087_AGENT_ID, 2, now)
        .unwrap();
    assert_eq!(
        state
            .execution_store
            .recoverable_execution_ids_for_agent(SPEC_087_AGENT_ID)
            .unwrap(),
        vec!["execution-pause-resume-087"]
    );
    let shared = Arc::new(Mutex::new(state));
    assert!(matches!(
        handle_control_state_set(
            &shared,
            "ui-control",
            1,
            "pause-resume-087",
            0,
            Some(true),
            None,
            Some("test"),
            None,
            None
        ),
        DaemonToWrapper::ControlState { .. }
    ));
    assert!(matches!(
        handle_control_state_set(
            &shared,
            "ui-control",
            1,
            "resume-pause-087",
            1,
            Some(false),
            None,
            Some("test"),
            None,
            None
        ),
        DaemonToWrapper::ControlState { .. }
    ));
    assert!(
        matches!(read_control(&mut reader), DaemonToWrapper::DeliverIdempotent { execution: Some(execution), .. } if execution.generation == 2)
    );
    assert!(
        shared
            .lock()
            .unwrap()
            .execution_store
            .pending_pause_interruptions()
            .unwrap()
            .is_empty()
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn spec_087_pause_interrompt_le_tour_en_cours() {
    let (mut state, config) = spec_087_state("spec-087-pause-interrupt");
    let (writer, mut reader) = control_socket("spec-087-pause-interrupt");
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(250)))
        .unwrap();
    state.connections.insert("conn-1".to_string(), writer);
    spec_087_register_human_control_client(&mut state);
    let now = unix_now_secs();
    let mut message = bridget_core::BridgetMessage::new("humain", SPEC_087_AGENT_ID, "arrête moi");
    message.id = "message-pause-interrupt-087".to_string();
    message.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    state
        .execution_store
        .admit_starting_message(&message, "execution-pause-interrupt-087", now)
        .unwrap();
    let shared = Arc::new(Mutex::new(state));
    assert!(matches!(
        handle_control_state_set(
            &shared,
            "ui-control",
            1,
            "pause-interrupt-087",
            0,
            Some(true),
            None,
            Some("test"),
            None,
            None
        ),
        DaemonToWrapper::ControlState { .. }
    ));
    assert!(
        matches!(read_control(&mut reader), DaemonToWrapper::ControlExecutionDispatch { command, .. } if command.operation == ExecutionControlOperation::Interrupt && command.execution_id == "execution-pause-interrupt-087")
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .execution_store
            .pending_pause_interruptions()
            .unwrap()
            .len(),
        1
    );
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn spec_087_levee_de_pause_agent_occupe_ne_relance_pas() {
    let (mut state, config) = spec_087_state("spec-087-pause-busy");
    let (writer, mut reader) = control_socket("spec-087-pause-busy");
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(250)))
        .unwrap();
    state.connections.insert("conn-1".to_string(), writer);
    state.presences.get_mut("instance-1").unwrap().busy_since = Some(Instant::now());
    spec_087_register_human_control_client(&mut state);
    let now = unix_now_secs();
    let mut message =
        bridget_core::BridgetMessage::new("humain", SPEC_087_AGENT_ID, "reste occupé");
    message.id = "message-pause-busy-087".to_string();
    message.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    state
        .execution_store
        .admit_starting_message(&message, "execution-pause-busy-087", now)
        .unwrap();
    state
        .execution_store
        .transition_if_current(
            "execution-pause-busy-087",
            "starting",
            0,
            1,
            "interrupted",
            "control_pause",
            now,
        )
        .unwrap();
    state
        .execution_store
        .record_pause_interruption("execution-pause-busy-087", SPEC_087_AGENT_ID, 1, now)
        .unwrap();
    let shared = Arc::new(Mutex::new(state));
    assert!(matches!(
        handle_control_state_set(
            &shared,
            "ui-control",
            1,
            "pause-busy-087",
            0,
            Some(true),
            None,
            None,
            None,
            None
        ),
        DaemonToWrapper::ControlState { .. }
    ));
    assert!(matches!(
        handle_control_state_set(
            &shared,
            "ui-control",
            1,
            "resume-busy-087",
            1,
            Some(false),
            None,
            None,
            None,
            None
        ),
        DaemonToWrapper::ControlState { .. }
    ));
    let state = shared.lock().unwrap();
    assert!(state.pending_post_response_controls.is_empty());
    assert_eq!(
        state.execution_store.pending_pause_interruptions().unwrap(),
        vec![(
            "execution-pause-busy-087".to_string(),
            SPEC_087_AGENT_ID.to_string()
        )]
    );
    drop(state);

    assert!(
        handle_wrapper_message(
            "conn-1",
            WrapperToDaemon::TurnState { in_progress: false },
            &shared
        )
        .is_none()
    );
    assert!(
        matches!(read_control(&mut reader), DaemonToWrapper::DeliverIdempotent { execution: Some(execution), .. } if execution.generation == 2)
    );
    let state = shared.lock().unwrap();
    assert!(
        state
            .execution_store
            .pending_pause_interruptions()
            .unwrap()
            .is_empty()
    );
    drop(state);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn spec_087_levee_de_pause_agent_absent_reprend_a_la_reconnexion() {
    let (mut state, config) = spec_087_state("spec-087-pause-absent");
    state.router.unregister_by_conn("conn-1");
    state.presences.remove("instance-1");
    spec_087_register_human_control_client(&mut state);
    let now = unix_now_secs();
    let mut message =
        bridget_core::BridgetMessage::new("humain", SPEC_087_AGENT_ID, "reprends à la reconnexion");
    message.id = "message-pause-absent-087".to_string();
    message.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    state
        .execution_store
        .admit_starting_message(&message, "execution-pause-absent-087", now)
        .unwrap();
    state
        .execution_store
        .transition_if_current(
            "execution-pause-absent-087",
            "starting",
            0,
            1,
            "interrupted",
            "control_pause",
            now,
        )
        .unwrap();
    state
        .execution_store
        .record_pause_interruption("execution-pause-absent-087", SPEC_087_AGENT_ID, 1, now)
        .unwrap();
    let shared = Arc::new(Mutex::new(state));
    assert!(matches!(
        handle_control_state_set(
            &shared,
            "ui-control",
            1,
            "pause-absent-087",
            0,
            Some(true),
            None,
            None,
            None,
            None
        ),
        DaemonToWrapper::ControlState { .. }
    ));
    assert!(matches!(
        handle_control_state_set(
            &shared,
            "ui-control",
            1,
            "resume-absent-087",
            1,
            Some(false),
            None,
            None,
            None,
            None
        ),
        DaemonToWrapper::ControlState { .. }
    ));
    let mut state = shared.lock().unwrap();
    assert!(state.pending_post_response_controls.is_empty());
    assert_eq!(
        state
            .execution_store
            .pending_pause_interruptions()
            .unwrap()
            .len(),
        1
    );
    let (writer, _reader) = control_socket("spec-087-pause-absent-reconnect");
    state.connections.insert("conn-2".to_string(), writer);
    state
        .router
        .register(
            SPEC_087_AGENT_ID,
            &bridget_core::AgentType::Claude,
            "conn-2",
        )
        .unwrap();
    state
        .conn_instances
        .insert("conn-2".to_string(), "instance-2".to_string());
    state.presences.insert(
        "instance-2".to_string(),
        Presence {
            name: SPEC_087_AGENT_ID.to_string(),
            agent_type: "claude".to_string(),
            host: "macbook".to_string(),
            transport: "acp".to_string(),
            channel: Some("unix".to_string()),
            mode: Some(PresenceMode::Acp),
            location: None,
            journal_available: true,
            os: "macOS".to_string(),
            state: "connected".to_string(),
            busy_since: None,
            busy_period_start: None,
            last_attested_at: None,
            turn_attest: false,
            queue_fallback: false,
            capacity_seen: Instant::now(),
            link_seen: Instant::now(),
            reconnect_count: 0,
            model: None,
            effort: None,
            rate_limits: Default::default(),
            served_model: None,
            derived_domain: None,
            domain: None,
            dnd_until: None,
            disk_space: None,
        },
    );
    schedule_execution_recovery(&mut state, "conn-2", "instance-2", SPEC_087_AGENT_ID, false);
    assert_eq!(state.pending_post_response_controls["conn-2"].len(), 1);
    assert!(
        state
            .execution_store
            .pending_pause_interruptions()
            .unwrap()
            .is_empty()
    );
    drop(state);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

/// Redémarrage daemon simulé : aucune présence préalable, le wrapper
/// Fixture SPEC-087 : identique à `state_with_registered_agent`, mais
/// l'agent est enregistré sous un UUID v4, comme l'exige le routeur depuis
/// la session 081. L'ancienne fixture enregistre `agent-2` et échoue sur
/// `InvalidAgentId` : c'est la cause d'une partie des rouges de main.
pub(super) const SPEC_087_AGENT_ID: &str = "6f1c2a4e-3b5d-4c7e-8f90-1a2b3c4d5e6f";

pub(super) fn spec_087_state(label: &str) -> (DaemonState, DaemonConfig) {
    let base = std::env::temp_dir().join(format!("bridget-{}-{}", label, std::process::id()));
    let registry_home = base.join("home");
    let fixture_root = FixtureRoot(registry_home.clone());
    std::fs::create_dir_all(registry_home.join(".config/bridget")).unwrap();
    let registry_file = registry_home.join(".config/bridget/agents.json");
    std::fs::write(
        &registry_file,
        serde_json::json!({
            "agents": {
                "claude": {
                    "command": "/bin/sh",
                    "protocol": "acp",
                    "forbidden_env": [],
                    "pass_env": []
                }
            }
        })
        .to_string(),
    )
    .unwrap();
    std::fs::set_permissions(&registry_file, std::fs::Permissions::from_mode(0o600)).unwrap();
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
    // Un test qui panique en tenant ce verrou ne doit pas empoisonner les
    // suivants : on relit le verrou empoisonné plutôt que de propager.
    let _home_lock = HOME_REGISTRY_LOCK.lock().recover_poison();
    let (managed_tx, _managed_rx) = mpsc::channel();
    let previous_home = std::env::var_os("HOME");
    // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
    unsafe { std::env::set_var("HOME", &registry_home) };
    let state_result = DaemonState::new(&config, managed_tx);
    if let Some(home) = previous_home {
        // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
        unsafe { std::env::set_var("HOME", home) };
    } else {
        // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
        unsafe { std::env::remove_var("HOME") };
    }
    let mut state = state_result.unwrap();
    state.fixture_root = Some(fixture_root);
    state
        .router
        .register(
            SPEC_087_AGENT_ID,
            &bridget_core::AgentType::Claude,
            "conn-1",
        )
        .unwrap();
    state
        .conn_instances
        .insert("conn-1".to_string(), "instance-1".to_string());
    state.presences.insert(
        "instance-1".to_string(),
        Presence {
            name: SPEC_087_AGENT_ID.to_string(),
            agent_type: "claude".to_string(),
            host: "macbook".to_string(),
            transport: "acp".to_string(),
            channel: Some("unix".to_string()),
            mode: Some(PresenceMode::Acp),
            location: None,
            journal_available: true,
            os: "macOS".to_string(),
            state: "connected".to_string(),
            busy_since: None,
            busy_period_start: None,
            last_attested_at: None,
            turn_attest: false,
            queue_fallback: false,
            capacity_seen: Instant::now(),
            link_seen: Instant::now(),
            reconnect_count: 0,
            model: None,
            effort: None,
            rate_limits: Default::default(),
            served_model: None,
            derived_domain: None,
            domain: None,
            dnd_until: None,
            disk_space: None,
        },
    );
    (state, config)
}

/// Ajoute une présence UI enregistrée sous l'identité humaine (T020).
fn spec_087_register_ui_presence(state: &mut DaemonState) {
    state
        .router
        .register(
            "conn-ui-id-placeholder",
            &bridget_core::AgentType::Claude,
            "conn-ui",
        )
        .ok();
    state
        .conn_names
        .insert("conn-ui".to_string(), UI_HUMAN_AGENT_ID.to_string());
    state
        .conn_instances
        .insert("conn-ui".to_string(), "instance-ui".to_string());
    state.presences.insert(
        "instance-ui".to_string(),
        Presence {
            name: UI_HUMAN_AGENT_ID.to_string(),
            agent_type: "ui".to_string(),
            host: "localhost".to_string(),
            transport: "cli".to_string(),
            channel: Some("ssh-unix".to_string()),
            mode: Some(PresenceMode::Cli),
            location: None,
            journal_available: false,
            os: "linux".to_string(),
            state: "connected".to_string(),
            busy_since: None,
            busy_period_start: None,
            last_attested_at: None,
            turn_attest: false,
            queue_fallback: false,
            capacity_seen: Instant::now(),
            link_seen: Instant::now(),
            reconnect_count: 0,
            model: None,
            effort: None,
            rate_limits: Default::default(),
            served_model: None,
            derived_domain: None,
            domain: None,
            dnd_until: None,
            disk_space: None,
        },
    );
}

fn spec_087_register_human_control_client(state: &mut DaemonState) {
    state.client_negotiations.insert(
        "ui-control".to_string(),
        NegotiatedClient {
            version: CLIENT_CONTRACT_VERSION,
            issuer_scope: crate::communication::issuer_scope("bridget-ui-control"),
            capabilities: vec![ClientCapability::ControlStateV1],
        },
    );
}

fn spec_087_focus_request(from: &str, request_id: &str) -> WrapperToDaemon {
    let payload = bridget_transport::protocol::ServiceRequestPayload::Delegate {
        goal: "Travaille sur l’import CSV".to_string(),
        review_target: None,
        explicit_target: None,
        required_tags: Vec::new(),
        duration: bridget_transport::protocol::GuichetDurationClass::Normale,
        suite: bridget_transport::protocol::ServiceSuiteDeclaration::Aucune,
        depends_on: Vec::new(),
        references: Vec::new(),
        origin: None,
        focus: Some(bridget_transport::protocol::DelegateFocus {
            project_id: "project-087".to_string(),
            on_conflict: None,
        }),
    };
    WrapperToDaemon::ServiceRequest {
        version: payload.required_contract_version(),
        issuer_scope: crate::communication::issuer_scope("bridget-ui-human-focus"),
        request_id: request_id.to_string(),
        issued_at: unix_now_secs(),
        from: from.to_string(),
        to: "guichet".to_string(),
        operation: bridget_transport::protocol::ServiceRequestOperation::Delegate,
        payload,
    }
}

/// SPEC-087 T020 : le daemon fabrique l'origine humaine pour le principal
/// humain, de façon déterministe et scellée ; il la refuse à un agent.
#[test]
fn spec_087_origine_humaine_fabriquee_par_le_daemon_et_refusee_aux_agents() {
    use bridget_transport::protocol::{DelegateOrigin, ServiceRequestPayload};
    let (mut state, config) = spec_087_state("spec-087-origine");
    spec_087_register_ui_presence(&mut state);
    // L'agent déclare bien son propre nom : c'est la garde d'origine qui
    // doit refuser, pas la cohérence d'émetteur.
    state
        .conn_names
        .insert("conn-1".to_string(), SPEC_087_AGENT_ID.to_string());
    let shared = Arc::new(Mutex::new(state));

    // Un agent enregistré qui réclame un focus est refusé, rien n'est déposé.
    let refused = handle_wrapper_message(
        "conn-1",
        spec_087_focus_request(SPEC_087_AGENT_ID, "focus-agent-1"),
        &shared,
    );
    assert!(
        matches!(
            refused,
            Some(DaemonToWrapper::ServiceRejected {
                reason: ServiceRefusal::HumanOriginForbidden
            })
        ),
        "refus d'origine attendu, reçu {refused:?}"
    );

    // Le principal humain : dépôt accepté, origine fabriquée.
    let first = handle_wrapper_message(
        "conn-ui",
        spec_087_focus_request(UI_HUMAN_AGENT_ID, "focus-humain-1"),
        &shared,
    );
    let Some(DaemonToWrapper::GuichetResult { issue, .. }) = first else {
        panic!("dépôt attendu, reçu {first:?}");
    };
    assert_ne!(issue, "canonical_bytes_mismatch");

    // Rejeu du même dépôt : même origine, donc pas de divergence d'enveloppe.
    let replay = handle_wrapper_message(
        "conn-ui",
        spec_087_focus_request(UI_HUMAN_AGENT_ID, "focus-humain-1"),
        &shared,
    );
    let Some(DaemonToWrapper::GuichetResult { issue, .. }) = replay else {
        panic!("rejeu attendu, reçu {replay:?}");
    };
    assert_ne!(
        issue, "canonical_bytes_mismatch",
        "l'origine doit être déterministe"
    );

    let mut st = shared.lock().unwrap();
    let now = unix_now_secs();
    let claim = match st.store.claim_next_guichet("guichet-test", now).unwrap() {
        crate::store::GuichetNext::Claimed(claim) => claim,
        other => panic!("dépôt relevable attendu: {other:?}"),
    };
    let canonical = String::from_utf8(claim.canonical_request.clone()).unwrap();
    let WrapperToDaemon::ServiceRequest { payload, .. } = decode(&canonical).unwrap() else {
        panic!("dépôt guichet attendu");
    };
    let ServiceRequestPayload::Delegate {
        origin:
            Some(DelegateOrigin::Human {
                message_id,
                observed,
                attestation,
            }),
        focus: Some(focus),
        goal,
        ..
    } = payload
    else {
        panic!("origine humaine et focus attendus");
    };
    assert!(message_id.starts_with("hmo-"));
    assert_eq!(observed.message_id, message_id);
    assert_eq!(observed.sender, UI_HUMAN_SENDER);
    assert_eq!(observed.target, "guichet");
    assert_eq!(observed.body, goal);
    assert_eq!(focus.project_id, "project-087");
    assert_eq!(
        attestation.signature,
        bridget_transport::protocol::human_message_content_seal(&observed),
        "le scellé se recalcule à l'identique côté service compagnon"
    );
    assert_eq!(attestation.canonical_request_sha256.len(), 64);
    let ledger_rows: i64 = st
        .store
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM ledger WHERE id = ?1 AND sender = ?2 AND target = 'guichet'",
            rusqlite::params![message_id, UI_HUMAN_SENDER],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(ledger_rows, 1, "le message humain d'origine est au ledger");
    assert!(
        claim.authorization_attestation.is_none(),
        "pas de garde du greffe pour le principal humain"
    );
    drop(st);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

/// SPEC-087 T048 : remplacer une route humaine encore vivante dépose une
/// alerte ; remplacer une route morte ne dépose rien.
#[test]
fn spec_087_reprise_d_une_route_humaine_vivante_est_signalee() {
    let (mut state, config) = spec_087_state("spec-087-route");
    spec_087_register_ui_presence(&mut state);
    // Rend la route routable sous l'identité humaine avec un écrivain.
    state.router.unregister_by_conn("conn-ui");
    state
        .router
        .register(
            UI_HUMAN_AGENT_ID,
            &bridget_core::AgentType::Claude,
            "conn-ui",
        )
        .unwrap();
    let (ui_writer, _ui_reader) = control_socket("spec-087-route-ui");
    state.connections.insert("conn-ui".to_string(), ui_writer);
    state
        .conn_instances
        .insert("conn-ui-2".to_string(), "instance-ui-2".to_string());
    state.replace_stale_ui_human_route("conn-ui-2", "ui", UI_HUMAN_AGENT_ID);
    let (items, open) = crate::human_inbox::list(
        state.store.connection(),
        bridget_transport::protocol::HumanInboxListFilter::Open,
        10,
    )
    .unwrap();
    assert_eq!(open, 1, "une route vivante remplacée est signalée");
    assert_eq!(
        items[0].kind,
        bridget_transport::protocol::HumanInboxKind::HumanRouteReplaced
    );
    assert!(items[0].context.contains("instance-ui-2"));

    // Route morte : présence hors fenêtre de rétention, aucune alerte.
    state
        .router
        .register(
            UI_HUMAN_AGENT_ID,
            &bridget_core::AgentType::Claude,
            "conn-ui-3",
        )
        .unwrap();
    let (dead_writer, _dead_reader) = control_socket("spec-087-route-dead");
    state
        .connections
        .insert("conn-ui-3".to_string(), dead_writer);
    state
        .conn_instances
        .insert("conn-ui-3".to_string(), "instance-ui-3".to_string());
    let mut stale = state.presences.get("instance-ui").cloned().unwrap();
    stale.link_seen = Instant::now() - PRESENCE_RETENTION - Duration::from_secs(1);
    state.presences.insert("instance-ui-3".to_string(), stale);
    state.replace_stale_ui_human_route("conn-ui-4", "ui", UI_HUMAN_AGENT_ID);
    assert_eq!(
        crate::human_inbox::open_count(state.store.connection()).unwrap(),
        1,
        "une route morte remplacée ne produit rien"
    );
    let _ = std::fs::remove_file(config.db_path);
}

/// SPEC-087 T019 : la pause survit à la reconstruction de l'état du daemon
/// sur la même base.
#[test]
fn spec_087_pause_survit_a_la_reconstruction_de_l_etat() {
    let (state, config) = spec_087_state("spec-087-persist");
    crate::referent_control::set(
        state.store.connection(),
        crate::referent_control::ControlMutation {
            command_id: "pause-persist",
            expected_generation: 0,
            paused: Some(true),
            auto_objectives_cap: Some(3),
            reason: Some("nuit"),
            actor: "humain",
            now: 42,
            agent_posture: None,
            auto_reassignment: None,
        },
    )
    .unwrap()
    .unwrap();
    drop(state);
    let (managed_tx, _managed_rx) = mpsc::channel();
    // Un test qui panique en tenant ce verrou ne doit pas empoisonner les
    // suivants : on relit le verrou empoisonné plutôt que de propager.
    let _home_lock = HOME_REGISTRY_LOCK.lock().recover_poison();
    let rebuilt = DaemonState::new(&config, managed_tx).unwrap();
    let control = crate::referent_control::read(rebuilt.store.connection()).unwrap();
    assert!(control.paused);
    assert_eq!(control.auto_objectives_cap, 3);
    assert_eq!(control.pause_reason.as_deref(), Some("nuit"));
    assert_eq!(control.generation, 1);
    drop(rebuilt);
    let _ = std::fs::remove_file(config.db_path);
}

/// SPEC-087 T042 : un agent qui agit par ses outils déclarés (client MCP,
/// capacités `[SendIdempotent]`) ne peut ni muter l'état de contrôle, ni
/// trancher la boîte, ni fabriquer une origine humaine.
#[test]
fn spec_087_un_agent_par_ses_outils_declares_ne_mute_rien() {
    use bridget_transport::protocol::{ControlStateRefusal, HumanInboxRefusal};
    let (mut state, config) = spec_087_state("spec-087-adverse");
    state
        .conn_names
        .insert("conn-1".to_string(), SPEC_087_AGENT_ID.to_string());
    let shared = Arc::new(Mutex::new(state));
    // Négociation client à la façon de la façade MCP.
    assert!(matches!(
        handle_wrapper_message(
            "mcp-client",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client,
            },
            &shared,
        ),
        Some(DaemonToWrapper::RoleAccepted { .. })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "mcp-client",
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: crate::communication::issuer_scope("instance-agent-2"),
                capabilities: vec![ClientCapability::SendIdempotent],
            },
            &shared,
        ),
        Some(DaemonToWrapper::ClientWelcome { .. })
    ));
    // Mutation de l'état : capacité non négociée.
    assert!(matches!(
        handle_wrapper_message(
            "mcp-client",
            WrapperToDaemon::ControlStateSet {
                version: 1,
                command_id: "adverse-1".to_string(),
                expected_generation: 0,
                paused: Some(false),
                auto_objectives_cap: Some(50),
                reason: None,
                agent_posture: None,
                auto_reassignment: None,
            },
            &shared,
        ),
        Some(DaemonToWrapper::ClientRejected {
            reason: ClientRefusal::CapabilityNotNegotiated
        })
    ));
    // Même avec la capacité, un périmètre d'émetteur non humain est refusé.
    assert!(matches!(
        handle_wrapper_message(
            "cap-client",
            WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client,
            },
            &shared,
        ),
        Some(DaemonToWrapper::RoleAccepted { .. })
    ));
    assert!(matches!(
        handle_wrapper_message(
            "cap-client",
            WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: crate::communication::issuer_scope("instance-agent-2"),
                capabilities: vec![ClientCapability::ControlStateV1],
            },
            &shared,
        ),
        Some(DaemonToWrapper::ClientWelcome { .. })
    ));
    let set_attempt = handle_wrapper_message(
        "cap-client",
        WrapperToDaemon::ControlStateSet {
            version: 1,
            command_id: "adverse-2".to_string(),
            expected_generation: 0,
            paused: Some(true),
            auto_objectives_cap: None,
            reason: None,
            agent_posture: None,
            auto_reassignment: None,
        },
        &shared,
    );
    assert!(
        matches!(
            set_attempt,
            Some(DaemonToWrapper::ControlStateRejected {
                reason: ControlStateRefusal::HumanPrincipalRequired
            })
        ),
        "refus du principal attendu, reçu {set_attempt:?}"
    );
    assert!(matches!(
        handle_wrapper_message(
            "cap-client",
            WrapperToDaemon::HumanInboxResolve {
                version: 1,
                command_id: "adverse-3".to_string(),
                item_id: "quelconque".to_string(),
                choice: "ack".to_string(),
            },
            &shared,
        ),
        Some(DaemonToWrapper::HumanInboxRejected {
            reason: HumanInboxRefusal::HumanPrincipalRequired
        })
    ));
    // Origine humaine forgée depuis une connexion d'agent enregistrée.
    let forged = spec_087_focus_request(SPEC_087_AGENT_ID, "adverse-4");
    assert!(matches!(
        handle_wrapper_message("conn-1", forged, &shared),
        Some(DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::HumanOriginForbidden
        })
    ));
    let st = shared.lock().unwrap();
    let control = crate::referent_control::read(st.store.connection()).unwrap();
    assert!(!control.paused, "rien n'a été écrit");
    assert_eq!(control.auto_objectives_cap, 5);
    assert_eq!(
        crate::human_inbox::open_count(st.store.connection()).unwrap(),
        0
    );
    assert!(
        matches!(
            st.store
                .connection()
                .query_row("SELECT COUNT(*) FROM guichet_requests", [], |row| row
                    .get::<_, i64>(0)),
            Ok(0)
        ),
        "aucun dépôt au guichet"
    );
    drop(st);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn publication_artefact_est_attestee_par_l_agent_et_le_projet_actif() {
    let base = std::env::temp_dir().join(format!(
        "bridget-artifact-publication-{}-{}",
        std::process::id(),
        Uuid::new_v4()
    ));
    let (mut state, config) = recovery_fixture_state(&base);
    let agent_id = "a3d27a89-80d5-4e0f-9b84-cf5523ecb026";
    let now = unix_timestamp();
    let _peer = register_artifact_fixture(&mut state, "conn-1", agent_id);
    state.artifact_root = config.db_path.with_extension("artifacts");
    state
        .store
        .bind_project_registration(
            "register-artifact-project",
            "project-artifact",
            "/srv/projects/artifact",
            now,
        )
        .unwrap();
    let order = FleetSpawnOrder {
        remote_placement: None,
        posture: None,
        agent_type: "fixture".to_string(),
        requested_name: Some(agent_id.to_string()),
        cwd: PathBuf::from("/srv/projects/artifact"),
        persistent: false,
        project: Some(ProjectReference {
            project_id: "project-artifact".to_string(),
            binding_generation: 1,
        }),
        command_id: "spawn-artifact-agent".to_string(),
        issued_at: now,
        deadline_at: now + 60,
        ownership: None,
    };
    assert!(matches!(
        state.fleet.request_spawn(&order, now).unwrap(),
        crate::fleet::SpawnSubmission::Start(_)
    ));
    let publication: ArtifactPublicationV1 = serde_json::from_str(include_str!(
        "../tests/fixtures/artifacts/chart-external-v1.json"
    ))
    .unwrap();
    let shared = Arc::new(Mutex::new(state));
    let response = handle_wrapper_message(
        "conn-1",
        WrapperToDaemon::ArtifactPublish {
            contract_version: ARTIFACT_CONTRACT_VERSION,
            canonical_publication: publication.canonical_bytes(),
        },
        &shared,
    );
    assert!(matches!(
        response,
        Some(DaemonToWrapper::ArtifactPublicationResult {
            receipt_json: Some(_),
            refusal_code: None,
            ..
        })
    ));
    let response = handle_wrapper_message(
        "conn-1",
        WrapperToDaemon::ArtifactPublish {
            contract_version: ARTIFACT_CONTRACT_VERSION,
            canonical_publication: publication.canonical_bytes(),
        },
        &shared,
    );
    assert!(matches!(
        response,
        Some(DaemonToWrapper::ArtifactPublicationResult {
            replayed: true,
            receipt_json: Some(_),
            ..
        })
    ));
    let Some(DaemonToWrapper::ArtifactPublicationResult {
        receipt_json: Some(bytes),
        ..
    }) = response
    else {
        panic!("reçu attendu")
    };
    let receipt: crate::artifact_types::ArtifactReceiptV1 = serde_json::from_slice(&bytes).unwrap();
    let read = WrapperToDaemon::ArtifactRead {
        request: ArtifactReadRequest {
            version: 1,
            artifact_ref: receipt.artifact_ref.clone(),
            version_ref: receipt.version_ref.clone(),
            kind: bridget_transport::protocol::ArtifactReadKind::Manifest,
            offset: 0,
            limit: 4096,
        },
    };
    assert!(matches!(
        handle_wrapper_message("conn-1", read.clone(), &shared),
        Some(DaemonToWrapper::ArtifactReadResult {
            outcome: ArtifactReadOutcome::Chunk { .. },
            ..
        })
    ));

    // Oracle conservé du relais retiré : une version d'un AUTRE projet
    // reste invisible, même si son identifiant est connu.
    let mut service = ArtifactService::open(
        &config.db_path,
        &config.db_path.with_extension("artifacts"),
        Default::default(),
    )
    .unwrap();
    let other = service
        .publish(
            ArtifactPublicationContext {
                project_id: "project-other".into(),
                conversation_reference: "conversation:other".into(),
                turn_reference: "turn:other".into(),
                created_by: agent_id.into(),
                origin_instance: "instance-other".into(),
                observed_at: now,
            },
            publication.clone(),
        )
        .unwrap();
    let other = match other {
        ArtifactPersistResult::Created(r) | ArtifactPersistResult::Replayed(r) => r,
    };
    let cross_project = WrapperToDaemon::ArtifactRead {
        request: ArtifactReadRequest {
            version: 1,
            artifact_ref: other.artifact_ref,
            version_ref: other.version_ref,
            kind: bridget_transport::protocol::ArtifactReadKind::Manifest,
            offset: 0,
            limit: 4096,
        },
    };
    assert!(matches!(
        handle_wrapper_message("conn-1", cross_project, &shared),
        Some(DaemonToWrapper::ArtifactReadResult {
            outcome: ArtifactReadOutcome::Rejected {
                reason: ArtifactReadRefusal::NotFound
            },
            ..
        })
    ));

    // Mutation : retirer le contrôle Active rend cette lecture possible.
    shared
        .lock()
        .unwrap()
        .store
        .mark_project_binding_path_missing("project-artifact", now + 1)
        .unwrap();
    assert!(matches!(
        handle_wrapper_message("conn-1", read.clone(), &shared),
        Some(DaemonToWrapper::ArtifactReadResult {
            outcome: ArtifactReadOutcome::Rejected {
                reason: ArtifactReadRefusal::ScopeUnavailable
            },
            ..
        })
    ));
    // La liaison redevient active, mais sa génération n'est PLUS celle
    // figée au lancement. Retirer cette comparaison casse cet oracle.
    let rebound = shared
        .lock()
        .unwrap()
        .store
        .rebind_project_binding("project-artifact", "/srv/projects/artifact-new", now + 2)
        .unwrap();
    assert_eq!(rebound.generation, 2);
    assert!(matches!(
        handle_wrapper_message("conn-1", read, &shared),
        Some(DaemonToWrapper::ArtifactReadResult {
            outcome: ArtifactReadOutcome::Rejected {
                reason: ArtifactReadRefusal::ScopeUnavailable
            },
            ..
        })
    ));
    assert!(
        matches!(handle_wrapper_message("conn-1", WrapperToDaemon::ArtifactPublish {
            contract_version: ARTIFACT_CONTRACT_VERSION, canonical_publication: publication.canonical_bytes(),
        }, &shared), Some(DaemonToWrapper::ArtifactPublicationResult { refusal_code: Some(code), .. }) if code == "project_binding_unavailable")
    );
    drop(service);
    drop(shared);
    let _ = std::fs::remove_dir_all(base);
}

/// ré-annonce un tour ouvert via Register. Sans `turn_in_progress=true`,
/// who afficherait `connected` — exactement le mensonge du constat.
#[test]
fn redemarrage_daemon_reinscription_tour_en_cours_affiche_busy() {
    let (mut state, config) = state_with_registered_agent("busy-cold-restart");
    state.router.unregister_by_conn("conn-1");
    state.conn_instances.clear();
    state.presences.clear();

    let response = handle_register(
        "conn-fresh",
        "codex".to_string(),
        Some("89000000-0000-4000-8000-000000000118".to_string()),
        Some("local".to_string()),
        Some("unix".to_string()),
        None,
        None,
        Some("macOS".to_string()),
        Some("instance-cold".to_string()),
        None,
        true,
        Some(false),
        &mut state,
    );
    assert!(matches!(response, DaemonToWrapper::Registered { .. }));
    let agent = state.agent_infos().pop().expect("agent réinscrit");
    assert_eq!(agent.agent_id, "89000000-0000-4000-8000-000000000118");
    assert_eq!(
        agent.state, "busy",
        "who doit restaurer busy depuis Register.turn_in_progress, pas attendre un prochain tour"
    );

    // Contrôle négatif du même oracle : sans le fait, connected reste honnête.
    state.router.unregister_by_conn("conn-fresh");
    state.conn_instances.clear();
    state.presences.clear();
    let idle = handle_register(
        "conn-idle",
        "codex".to_string(),
        Some("89000000-0000-4000-8000-000000000118".to_string()),
        Some("local".to_string()),
        Some("unix".to_string()),
        None,
        None,
        Some("macOS".to_string()),
        Some("instance-cold".to_string()),
        None,
        false,
        Some(false),
        &mut state,
    );
    assert!(matches!(idle, DaemonToWrapper::Registered { .. }));
    assert_eq!(state.agent_infos()[0].state, "connected");
    let _ = std::fs::remove_file(&config.db_path);
}

/// Course production : le router libère le nom (EOF), le nouveau Register
/// arrive AVANT mark_unreachable, avec l'ancienne carte instance encore
/// présente. Sans prise de possession, le Register est traité comme
/// auxiliaire MCP et busy est perdu — rouge sur cet oracle.
#[test]
fn fermeture_ancienne_connexion_n_ecrase_pas_busy_de_la_reinscription() {
    let (mut state, config) = state_with_registered_agent("busy-race-rebind");
    state.set_turn_state("conn-1", true).unwrap();
    assert_eq!(state.agent_infos()[0].state, "busy");

    // Première moitié du cleanup EOF : le nom est libre, la carte instance
    // de l'ancienne connexion est encore là (mark_unreachable pas encore).
    state.router.unregister_by_conn("conn-1");
    assert!(state.conn_instances.contains_key("conn-1"));

    let response = handle_register(
        "conn-2",
        "claude".to_string(),
        Some("89000000-0000-4000-8000-000000000102".to_string()),
        Some("macbook".to_string()),
        Some("unix".to_string()),
        Some(PresenceMode::Acp),
        None,
        Some("macOS".to_string()),
        Some("instance-1".to_string()),
        None,
        true,
        Some(false),
        &mut state,
    );
    assert!(matches!(response, DaemonToWrapper::Registered { .. }));
    assert_eq!(state.agent_infos()[0].state, "busy");
    assert!(
        state.conn_instances.contains_key("conn-2"),
        "la reconnexion du même équipier doit prendre l'instance, pas rester auxiliaire"
    );
    assert!(
        !state.conn_instances.contains_key("conn-1"),
        "l'ancienne carte instance doit être retirée au takeover"
    );

    // Seconde moitié du cleanup EOF (retardée) : ne doit plus toucher busy.
    state.mark_unreachable("conn-1");
    assert_eq!(
        state.agent_infos()[0].state,
        "busy",
        "l'EOF retardé de l'ancienne connexion ne doit pas effacer busy attesté"
    );
    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn projection_ledger_est_bornee_et_lit_le_store_du_daemon() {
    // Une projection globale n'exige aucune présence : ne pas enregistrer
    // le nom pré-v2 agent-2 de l'ancien harnais pour exercer une lecture.
    let root = std::env::temp_dir().join(format!("ledger-projection-{}", uuid::Uuid::new_v4()));
    let (state, _config) = recovery_fixture_state(&root);
    let mut first = bridget_core::BridgetMessage::new("alice", "bob", "bonjour");
    first.id = "m-1".to_string();
    let mut second = bridget_core::BridgetMessage::new("alice", "bob", "salut");
    second.id = "m-2".to_string();
    state.store.record_message(&first, "alice:bob").unwrap();
    state.store.record_message(&second, "alice:bob").unwrap();
    let shared = Arc::new(Mutex::new(state));

    let response = handle_wrapper_message(
        "conn-cli",
        WrapperToDaemon::LedgerProjection {
            scope: bridget_transport::protocol::LedgerScope::Messages,
            limit: u16::MAX,
        },
        &shared,
    );

    assert!(matches!(
        response,
        Some(DaemonToWrapper::LedgerProjection { messages, requests })
            if messages.len() == 2
                && messages.iter().any(|message| message.id == "m-1")
                && requests.is_empty()
    ));
    drop(shared);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn expiration_transport_est_idempotente_cote_daemon() {
    let (mut state, config) = state_with_registered_agent("expiration-unique");
    state
        .store
        .create_request(
            "request-timeout",
            "sender",
            "89000000-0000-4000-8000-000000000102",
            60,
        )
        .unwrap();
    state.pending_replies.push(PendingReply {
        msg_id: "request-timeout".to_string(),
        from: "sender".to_string(),
        from_conn: "conn-sender".to_string(),
        to: "89000000-0000-4000-8000-000000000102".to_string(),
        target_conn: "conn-1".to_string(),
        timeout_secs: 60,
        created_at: Instant::now(),
        frozen_secs: 0,
        escalation_level: 0,
        deferred_level: None,
    });
    let shared = Arc::new(Mutex::new(state));
    handle_wrapper_message(
        "conn-1",
        WrapperToDaemon::DeliveryRejected {
            execution_id: None,
            generation: None,
            id: "request-timeout".to_string(),
            reason: "échéance de livraison dépassée".to_string(),
        },
        &shared,
    );
    let state = shared.lock().unwrap();
    assert!(state.pending_replies.is_empty());
    assert_eq!(
        state
            .store
            .get_request("request-timeout")
            .unwrap()
            .unwrap()
            .state,
        "timed_out"
    );
    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn intercalage_timeout_et_transport_n_autorise_qu_une_notification() {
    let (mut state, config) = state_with_registered_agent("timeout-concurrent");
    state
        .store
        .create_request(
            "request-timeout",
            "sender",
            "89000000-0000-4000-8000-000000000102",
            60,
        )
        .unwrap();
    assert!(claim_timeout(&mut state.store, "request-timeout"));
    assert!(!claim_timeout(&mut state.store, "request-timeout"));
    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn vue_requests_expose_le_dernier_report_differe() {
    let (state, config) = state_with_registered_agent("vue-report");
    state
        .store
        .create_request(
            "request-report",
            "89000000-0000-4000-8000-000000000102",
            "cible",
            60,
        )
        .unwrap();
    state
        .store
        .record_deferred_reminder("request-report", 2)
        .unwrap();
    let shared = Arc::new(Mutex::new(state));
    let response = handle_wrapper_message(
        "conn-1",
        WrapperToDaemon::ListRequests {
            sender: "89000000-0000-4000-8000-000000000102".to_string(),
            limit: 200,
        },
        &shared,
    );
    match response {
        Some(DaemonToWrapper::RequestList { requests }) => {
            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0].deferred_reminder_level, Some(2));
            assert!(requests[0].deferred_reminder_at.is_some());
        }
        other => panic!("réponse requests inattendue: {other:?}"),
    }
    let _ = std::fs::remove_file(&config.db_path);
}

#[test]
fn open_request_is_restored_when_both_agents_reconnect() {
    let base = std::env::temp_dir().join(format!("bridget-restore-{}", std::process::id()));
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
    let mut state = DaemonState::new(&config, managed_tx).unwrap();
    state
        .store
        .create_request(
            "request-1",
            "89000000-0000-4000-8000-000000000125",
            "89000000-0000-4000-8000-000000000126",
            60,
        )
        .unwrap();
    state
        .router
        .register(
            "89000000-0000-4000-8000-000000000125",
            &bridget_core::AgentType::Codex,
            "conn-s",
        )
        .unwrap();
    state
        .router
        .register(
            "89000000-0000-4000-8000-000000000126",
            &bridget_core::AgentType::Claude,
            "conn-t",
        )
        .unwrap();
    state.restore_pending_for_agent("89000000-0000-4000-8000-000000000126", "conn-t");
    assert_eq!(state.pending_replies.len(), 1);
    assert_eq!(state.pending_replies[0].msg_id, "request-1");
    if let Err(e) = std::fs::remove_file(&config.db_path) {
        log::warn!(
            "Impossible de supprimer la base {}: {}",
            config.db_path.display(),
            e
        );
    }
}

fn managed_test_lease(command_id: &str) -> SpawnLease {
    SpawnLease {
        remote_placement: None,
        command_id: command_id.to_string(),
        name: "89000000-0000-4000-8000-000000000102".to_string(),
        instance_id: "instance-1".to_string(),
        generation: 1,
        deadline_at: unix_timestamp() + 60,
        persistent: false,
        project: None,
        link_id: None,
        ownership: None,
        agent_path: None,
    }
}

fn install_managed_test_spawn(
    state: &mut DaemonState,
    command_id: &str,
    connected: bool,
) -> (SpawnLease, Arc<ManagedStopControl>) {
    let now = unix_timestamp();
    let order = FleetSpawnOrder {
        remote_placement: None,
        posture: None,
        agent_type: "fixture".to_string(),
        requested_name: Some("89000000-0000-4000-8000-000000000102".to_string()),
        cwd: PathBuf::from("/tmp"),
        persistent: false,
        command_id: command_id.to_string(),
        issued_at: now,
        deadline_at: now + 60,
        project: None,
        ownership: None,
    };
    let lease = match state.fleet.request_spawn(&order, now).unwrap() {
        crate::fleet::SpawnSubmission::Start(lease) => lease,
        other => panic!("spawn de test non démarré: {other:?}"),
    };
    let definition = recovery_fixture_definition();
    state.fleet.mark_starting(&lease, now, &definition).unwrap();
    if connected {
        state
            .fleet
            .register_connected(&lease, &lease.instance_id, now + 1)
            .unwrap();
    }
    let stop = Arc::new(ManagedStopControl::new());
    state
        .managed_by_instance
        .insert(lease.instance_id.clone(), lease.command_id.clone());
    state.managed_spawns.insert(
        lease.command_id.clone(),
        ManagedSpawnRecord {
            lease: lease.clone(),
            agent_type: "fixture".to_string(),
            requester_conns: Vec::new(),
            wrapper_conn: connected.then(|| "conn-1".to_string()),
            stop: Arc::clone(&stop),
        },
    );
    (lease, stop)
}

fn managed_test_binary() -> PathBuf {
    let executable = std::env::current_exe()
        .unwrap()
        .parent()
        .and_then(std::path::Path::parent)
        .unwrap()
        .join("bridget");
    assert!(executable.exists(), "binaire bridget de test absent");
    executable
}

fn managed_test_prepared(lease: &SpawnLease, root: &std::path::Path) -> PreparedSpawn {
    let frozen_registry = AgentRegistry::from_json(
        r#"{"agents":{"fixture":{"command":"/bin/sh","protocol":"acp"}}}"#,
        "/tmp/managed-test-definition.json",
    )
    .unwrap();
    PreparedSpawn {
        specialist_authority: None,
        lease: lease.clone(),
        agent_type: "fixture".to_string(),
        command: "/bin/sh".to_string(),
        args: Vec::new(),
        resolved_definition: Box::new(frozen_registry.resolved_definition("fixture").unwrap()),
        cwd: root.to_path_buf(),
        env: BTreeMap::from([
            ("HOME".to_string(), root.as_os_str().to_owned()),
            (
                "BRIDGET_HOME".to_string(),
                root.join(".state").into_os_string(),
            ),
            (
                "BRIDGET_SOCKET".to_string(),
                root.join(".state/bridget.sock").into_os_string(),
            ),
            ("PATH".to_string(), OsString::from("/bin:/usr/bin")),
            ("USER".to_string(), OsString::from("tester")),
            ("LANG".to_string(), OsString::from("C")),
            ("TMPDIR".to_string(), OsString::from("/tmp")),
        ]),
    }
}

fn running_managed_test_group(
    lease: &SpawnLease,
    root: &std::path::Path,
    shell: &str,
) -> RunningManagedChild {
    let launch = ManagedLaunch {
        bootstrap_executable: managed_test_binary(),
        identity: ManagedIdentity {
            instance_id: lease.instance_id.clone(),
            command_id: lease.command_id.clone(),
            generation: lease.generation,
        },
        wrapper_executable: PathBuf::from("/bin/sh"),
        wrapper_args: vec!["-c".to_string(), shell.to_string()],
        cwd: root.to_path_buf(),
        env: managed_test_prepared(lease, root).env,
    };
    let marker_store = ManagedMarkerStore::at_directory(root.join("managed"));
    let mut child = crate::managed_process::spawn_managed_bootstrap(&launch)
        .unwrap()
        .wait_ready()
        .unwrap()
        .persist_marker(&marker_store, &lease.name)
        .unwrap()
        .release()
        .unwrap();
    child.set_status_nonblocking().unwrap();
    child
}

#[test]
fn stop_refuse_structurellement_un_wrapper_terminal() {
    let (state, config) = state_with_registered_agent("stop-not-managed");
    let shared = Arc::new(Mutex::new(state));

    assert!(matches!(
        handle_wrapper_message(
            "control",
            WrapperToDaemon::StopOrder {
                agent_id: "89000000-0000-4000-8000-000000000102".to_string(),
                command_id: "stop-terminal".to_string(),
            },
            &shared,
        ),
        Some(DaemonToWrapper::StopResult {
            command_id,
            outcome: StopOutcome::NotManaged,
        }) if command_id == "stop-terminal"
    ));
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn relance_est_refusee_tant_que_le_decommissionnement_possede_le_nom() {
    let (mut state, config) = state_with_registered_agent("decommission-relaunch-race");
    let (lease, _) = install_managed_test_spawn(&mut state, "spawn-before-decommission", true);
    state.fleet.invalidate_for_stop(&lease).unwrap();
    state.managed_spawns.clear();
    state.managed_by_instance.clear();
    state.decommissioning_names.insert(lease.name.clone());
    let shared = Arc::new(Mutex::new(state));

    assert!(matches!(
        handle_wrapper_message(
            "control",
            WrapperToDaemon::RelaunchOrder {
                agent_id: lease.name.clone(),
                command_id: "relaunch-during-decommission".to_string(),
            },
            &shared,
        ),
        Some(DaemonToWrapper::RelaunchResult {
            command_id,
            outcome: RelaunchOutcome::NotRelaunchable { reason },
        }) if command_id == "relaunch-during-decommission"
            && reason == "décommissionnement en cours"
    ));
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .fleet
            .desired_entry(&lease.name)
            .unwrap()
            .unwrap()
            .lifecycle_state,
        DesiredLifecycleState::Stopped
    );
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn stop_avant_marqueur_annule_la_generation_et_repond_apres_nettoyage() {
    let (mut state, config) = state_with_registered_agent("stop-before-marker");
    let (lease, stop) = install_managed_test_spawn(&mut state, "spawn-before-marker", false);
    let process_root = PathBuf::from(format!(
        "/tmp/bg907-before-{}-{}",
        std::process::id(),
        &Uuid::new_v4().simple().to_string()[..8]
    ));
    std::fs::create_dir_all(&process_root).unwrap();
    let prepared = managed_test_prepared(&lease, &process_root);
    let shared = Arc::new(Mutex::new(state));
    let caller_state = Arc::clone(&shared);
    let caller = thread::spawn(move || {
        handle_wrapper_message(
            "control",
            WrapperToDaemon::StopOrder {
                agent_id: "89000000-0000-4000-8000-000000000102".to_string(),
                command_id: "stop-before-marker".to_string(),
            },
            &caller_state,
        )
    });
    let deadline = Instant::now() + Duration::from_secs(2);
    while !stop.is_requested() {
        assert!(
            Instant::now() < deadline,
            "stop non transmis au superviseur"
        );
        thread::yield_now();
    }
    assert!(
        stop.is_actionable(),
        "un lancement sans wrapper doit être annulable immédiatement"
    );
    let (event_tx, event_rx) = mpsc::channel();
    let marker_store = ManagedMarkerStore::at_directory(process_root.join("managed"));
    let stderr_store = ManagedStderrStore::at_directory(process_root.join("stderr"));
    let fleet = Arc::clone(&shared.lock().unwrap().fleet);
    let mut active = HashMap::new();
    handle_managed_command(
        ManagedSupervisorCommand::Start {
            prepared,
            stop: Arc::clone(&stop),
        },
        &fleet,
        &marker_store,
        &stderr_store,
        &event_tx,
        &mut active,
        Some(&managed_test_binary()),
    );
    assert!(active.is_empty());
    assert!(
        marker_store
            .load("89000000-0000-4000-8000-000000000102")
            .is_err()
    );
    drain_managed_events(&shared, &event_rx);

    assert!(matches!(
        caller.join().unwrap(),
        Some(DaemonToWrapper::StopResult {
            command_id,
            outcome: StopOutcome::Stopped,
        }) if command_id == "stop-before-marker"
    ));
    assert!(shared.lock().unwrap().managed_spawns.is_empty());
    std::fs::remove_dir_all(process_root).unwrap();
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn stop_pendant_bootstrap_bloque_termine_le_processus_reel_avant_marqueur() {
    let (mut state, config) = state_with_registered_agent("stop-bootstrap-blocked");
    let (lease, stop) = install_managed_test_spawn(&mut state, "spawn-bootstrap-blocked", false);
    let process_root = PathBuf::from(format!(
        "/tmp/bg907-blocked-{}-{}",
        std::process::id(),
        &Uuid::new_v4().simple().to_string()[..8]
    ));
    std::fs::create_dir_all(&process_root).unwrap();
    let barrier = process_root.join("bootstrap-entered");
    let bootstrap = process_root.join("blocked-bootstrap.sh");
    std::fs::write(
        &bootstrap,
        "#!/bin/sh\n: > \"$BRIDGET_TEST_BARRIER\"\ntrap 'exit 0' TERM\nwhile :; do sleep 1; done\n",
    )
    .unwrap();
    std::fs::set_permissions(&bootstrap, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut prepared = managed_test_prepared(&lease, &process_root);
    prepared.env.insert(
        "BRIDGET_TEST_BARRIER".to_string(),
        barrier.as_os_str().to_owned(),
    );
    let shared = Arc::new(Mutex::new(state));
    let fleet = Arc::clone(&shared.lock().unwrap().fleet);
    let marker_store = ManagedMarkerStore::at_directory(process_root.join("managed"));
    let marker_store_for_thread = marker_store.clone();
    let stderr_store = ManagedStderrStore::at_directory(process_root.join("stderr"));
    let (event_tx, event_rx) = mpsc::channel();
    let stop_for_handler = Arc::clone(&stop);
    let handler = thread::spawn(move || {
        let mut active = HashMap::new();
        handle_managed_command(
            ManagedSupervisorCommand::Start {
                prepared,
                stop: stop_for_handler,
            },
            &fleet,
            &marker_store_for_thread,
            &stderr_store,
            &event_tx,
            &mut active,
            Some(&bootstrap),
        );
        active
    });
    let deadline = Instant::now() + Duration::from_secs(2);
    while !barrier.exists() {
        assert!(
            Instant::now() < deadline,
            "le bootstrap réel n'a pas atteint la barrière"
        );
        thread::sleep(Duration::from_millis(5));
    }
    let caller_state = Arc::clone(&shared);
    let caller = thread::spawn(move || {
        handle_wrapper_message(
            "control",
            WrapperToDaemon::StopOrder {
                agent_id: "89000000-0000-4000-8000-000000000102".to_string(),
                command_id: "stop-bootstrap-blocked".to_string(),
            },
            &caller_state,
        )
    });
    assert!(handler.join().unwrap().is_empty());
    drain_managed_events(&shared, &event_rx);
    assert!(matches!(
        caller.join().unwrap(),
        Some(DaemonToWrapper::StopResult {
            outcome: StopOutcome::Stopped,
            ..
        })
    ));
    assert!(
        marker_store
            .load("89000000-0000-4000-8000-000000000102")
            .is_err()
    );
    assert!(shared.lock().unwrap().managed_spawns.is_empty());
    std::fs::remove_dir_all(process_root).unwrap();
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn stop_apres_register_envoie_disconnect_puis_attend_la_fin_du_groupe() {
    let (mut state, config) = state_with_registered_agent("stop-connected");
    let (wrapper_writer, mut wrapper_reader) = control_socket("stop-wrapper");
    state
        .connections
        .insert("conn-1".to_string(), wrapper_writer);
    let (lease, stop) = install_managed_test_spawn(&mut state, "spawn-connected", true);
    let process_root = PathBuf::from(format!(
        "/tmp/bg907-connected-{}-{}",
        std::process::id(),
        &Uuid::new_v4().simple().to_string()[..8]
    ));
    std::fs::create_dir_all(&process_root).unwrap();
    let child = running_managed_test_group(
        &lease,
        &process_root,
        "trap 'exit 0' TERM; while :; do sleep 1; done",
    );
    let pgid = child.marker().marker().pgid;
    let marker_path = child.marker().path().to_path_buf();
    let fleet = Arc::clone(&state.fleet);
    let mut active = HashMap::from([(
        lease.instance_id.clone(),
        SupervisedProcess {
            prepared: managed_test_prepared(&lease, &process_root),
            child: SupervisedChild(child),
            registered: Some((
                "conn-1".to_string(),
                "89000000-0000-4000-8000-000000000102".to_string(),
            )),
            connected: true,
            failure_sent: false,
            stop: Arc::clone(&stop),
            stop_attempted: false,
        },
    )]);
    let shared = Arc::new(Mutex::new(state));
    let caller_state = Arc::clone(&shared);
    let caller = thread::spawn(move || {
        handle_wrapper_message(
            "control",
            WrapperToDaemon::StopOrder {
                agent_id: "89000000-0000-4000-8000-000000000102".to_string(),
                command_id: "stop-connected".to_string(),
            },
            &caller_state,
        )
    });

    assert!(matches!(
        read_control(&mut wrapper_reader),
        DaemonToWrapper::Disconnect
    ));
    let deadline = Instant::now() + Duration::from_secs(2);
    while !stop.is_actionable() {
        assert!(
            Instant::now() < deadline,
            "le handshake écrit n'a pas libéré le superviseur"
        );
        thread::yield_now();
    }
    let (event_tx, event_rx) = mpsc::channel();
    poll_managed_processes(&fleet, &event_tx, &mut active);
    assert!(
        active.is_empty(),
        "le groupe réel reste supervisé après sa disparition"
    );
    drain_managed_events(&shared, &event_rx);

    assert!(matches!(
        caller.join().unwrap(),
        Some(DaemonToWrapper::StopResult {
            command_id,
            outcome: StopOutcome::Stopped | StopOutcome::StoppedForced { .. },
        }) if command_id == "stop-connected"
    ));
    let state = shared.lock().unwrap();
    assert!(
        state
            .router
            .get_agent("89000000-0000-4000-8000-000000000102")
            .is_none()
    );
    assert_eq!(state.presences["instance-1"].state, "stopped");
    drop(state);
    assert!(!marker_path.exists());
    assert!(!crate::managed_process::group_exists(pgid).unwrap());
    std::fs::remove_dir_all(process_root).unwrap();
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn timeout_d_arret_conserve_le_groupe_reel_et_sa_supervision_jusqu_a_disparition() {
    let (mut state, config) = state_with_registered_agent("stop-timeout-real");
    let (wrapper_writer, mut wrapper_reader) = control_socket("stop-timeout-wrapper");
    state
        .connections
        .insert("conn-1".to_string(), wrapper_writer);
    let (lease, stop) = install_managed_test_spawn(&mut state, "spawn-timeout", true);
    let process_root = PathBuf::from(format!(
        "/tmp/bg907-timeout-{}-{}",
        std::process::id(),
        &Uuid::new_v4().simple().to_string()[..8]
    ));
    std::fs::create_dir_all(&process_root).unwrap();
    let child = running_managed_test_group(
        &lease,
        &process_root,
        "trap '' TERM; while :; do sleep 1; done",
    );
    let pgid = child.marker().marker().pgid;
    let marker_path = child.marker().path().to_path_buf();
    let fleet = Arc::clone(&state.fleet);
    let mut active = HashMap::from([(
        lease.instance_id.clone(),
        SupervisedProcess {
            prepared: managed_test_prepared(&lease, &process_root),
            child: SupervisedChild(child),
            registered: Some((
                "conn-1".to_string(),
                "89000000-0000-4000-8000-000000000102".to_string(),
            )),
            connected: true,
            failure_sent: false,
            stop: Arc::clone(&stop),
            stop_attempted: false,
        },
    )]);
    let shared = Arc::new(Mutex::new(state));
    let caller_state = Arc::clone(&shared);
    let caller = thread::spawn(move || {
        handle_wrapper_message(
            "control",
            WrapperToDaemon::StopOrder {
                agent_id: "89000000-0000-4000-8000-000000000102".to_string(),
                command_id: "stop-timeout".to_string(),
            },
            &caller_state,
        )
    });

    assert!(matches!(
        read_control(&mut wrapper_reader),
        DaemonToWrapper::Disconnect
    ));
    let deadline = Instant::now() + Duration::from_secs(2);
    while !stop.is_actionable() {
        assert!(
            Instant::now() < deadline,
            "stop non transmis au superviseur"
        );
        thread::yield_now();
    }
    let (event_tx, event_rx) = mpsc::channel();
    poll_managed_processes(&fleet, &event_tx, &mut active);
    assert!(active.contains_key(&lease.instance_id));
    drain_managed_events(&shared, &event_rx);
    assert!(matches!(
        caller.join().unwrap(),
        Some(DaemonToWrapper::StopResult {
            outcome: StopOutcome::Timeout { .. },
            ..
        })
    ));
    {
        let state = shared.lock().unwrap();
        assert!(state.managed_spawns.contains_key(&lease.command_id));
        assert_eq!(
            state.managed_by_instance.get(&lease.instance_id),
            Some(&lease.command_id)
        );
        assert!(
            state
                .router
                .get_agent("89000000-0000-4000-8000-000000000102")
                .is_some()
        );
        assert_eq!(state.presences["instance-1"].state, "connected");
    }
    assert!(marker_path.exists());
    assert!(crate::managed_process::group_exists(pgid).unwrap());

    crate::managed_process::signal_group(pgid, libc::SIGKILL).unwrap();
    let cleanup_deadline = Instant::now() + Duration::from_secs(2);
    while active.contains_key(&lease.instance_id) {
        poll_managed_processes(&fleet, &event_tx, &mut active);
        assert!(
            Instant::now() < cleanup_deadline,
            "le superviseur n'a pas observé la disparition réelle"
        );
        thread::sleep(Duration::from_millis(10));
    }
    drain_managed_events(&shared, &event_rx);
    assert!(!marker_path.exists());
    assert!(!crate::managed_process::group_exists(pgid).unwrap());
    assert!(shared.lock().unwrap().managed_spawns.is_empty());
    std::fs::remove_dir_all(process_root).unwrap();
    let _ = std::fs::remove_file(config.db_path);
}

/// Oracle anti-fantôme : si la négociation expire après naissance du
/// processus (BootstrapReady + RELEASE, pas encore Connected), le groupe
/// réel DOIT mourir sous SIGTERM. Meurt si l'enfant survit au refus.
#[test]
fn timeout_avant_connected_termine_le_groupe_reel_sans_fantome() {
    let (mut state, config) = state_with_registered_agent("spawn-timeout-fantome");
    let (requester, mut requester_reader) = control_socket("timeout-fantome-requester");
    state.connections.insert("requester".to_string(), requester);
    let (lease, stop) = install_managed_test_spawn(&mut state, "spawn-timeout-fantome", false);
    {
        let record = state
            .managed_spawns
            .get_mut(&lease.command_id)
            .expect("spawn enregistré");
        record.requester_conns.push("requester".to_string());
    }
    let process_root = PathBuf::from(format!(
        "/tmp/bg-fantome-{}-{}",
        std::process::id(),
        &Uuid::new_v4().simple().to_string()[..8]
    ));
    std::fs::create_dir_all(&process_root).unwrap();
    let child = running_managed_test_group(
        &lease,
        &process_root,
        "trap 'exit 0' TERM; while :; do sleep 1; done",
    );
    let pgid = child.marker().marker().pgid;
    let marker_path = child.marker().path().to_path_buf();
    assert!(
        crate::managed_process::group_exists(pgid).unwrap(),
        "précondition: enfant vivant"
    );

    let mut prepared = managed_test_prepared(&lease, &process_root);
    prepared.lease.deadline_at = unix_timestamp() - 1;
    let fleet = Arc::clone(&state.fleet);
    let mut active = HashMap::from([(
        lease.instance_id.clone(),
        SupervisedProcess {
            prepared,
            child: SupervisedChild(child),
            registered: None,
            connected: false,
            failure_sent: false,
            stop: Arc::clone(&stop),
            stop_attempted: false,
        },
    )]);
    let shared = Arc::new(Mutex::new(state));
    let (event_tx, event_rx) = mpsc::channel();
    poll_managed_processes(&fleet, &event_tx, &mut active);
    assert!(
        active.is_empty(),
        "le superviseur doit retirer le spawn expiré de active"
    );
    assert!(
        !crate::managed_process::group_exists(pgid).unwrap(),
        "fantôme: le groupe survit à une négociation expirée"
    );
    assert!(!marker_path.exists(), "marqueur résiduel après timeout");
    drain_managed_events(&shared, &event_rx);
    assert!(matches!(
        read_control(&mut requester_reader),
        DaemonToWrapper::SpawnRejected {
            reason: SpawnRefusal::SpawnTimeout,
            ..
        }
    ));
    assert!(shared.lock().unwrap().managed_spawns.is_empty());
    std::fs::remove_dir_all(process_root).unwrap();
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn stop_apres_register_traverse_le_wrapper_et_le_superviseur_reels() {
    let root = PathBuf::from(format!(
        "/tmp/bg907-e2e-{}-{}",
        std::process::id(),
        &Uuid::new_v4().simple().to_string()[..8]
    ));
    let cache = root.join(".state");
    let registry_path = root.join(".state/agents.json");
    let adapter = root.join("adapter.sh");
    crate::environment::ensure_private_directory(&cache).unwrap();
    std::fs::create_dir_all(registry_path.parent().unwrap()).unwrap();
    std::fs::write(
            &adapter,
            "#!/bin/sh\nread initialize\necho '{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"protocolVersion\":1}}'\nread session\necho '{\"jsonrpc\":\"2.0\",\"id\":2,\"result\":{\"sessionId\":\"fixture-session\"}}'\nwhile read line; do :; done\n",
        )
        .unwrap();
    std::fs::set_permissions(&adapter, std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::write(
        &registry_path,
        serde_json::to_vec(&serde_json::json!({
            "agents": {
                "fixture": {
                    "command": adapter,
                    "protocol": "acp",
                    "permissions": "allow",
                    "queue_capacity": 2,
                    "notify_timeout_secs": 1
                }
            }
        }))
        .unwrap(),
    )
    .unwrap();
    std::fs::set_permissions(&registry_path, std::fs::Permissions::from_mode(0o600)).unwrap();
    let config = DaemonConfig {
        socket_path: cache.join("bridget.sock"),
        db_path: cache.join("bridget.db"),
        log_path: cache.join("daemon.log"),
        circuit_breaker_window: 180,
        circuit_breaker_limit: 8,
        dedup_window: 180,
        quarantine_window: 3600,
        retention_days: 7,
    };
    let (managed_tx, managed_rx) = mpsc::channel();
    let mut state = DaemonState::new(&config, managed_tx.clone()).unwrap();
    let (lease, _stop) = install_managed_test_spawn(&mut state, "spawn-e2e", false);
    let mut prepared = managed_test_prepared(&lease, &root);
    prepared.agent_type = "fixture".to_string();
    prepared.command = adapter.to_string_lossy().into_owned();
    prepared.resolved_definition = Box::new(
        AgentRegistry::from_json(
            &serde_json::json!({
                "agents": {
                    "fixture": {
                        "command": adapter,
                        "protocol": "acp",
                        "permissions": "allow",
                        "queue_capacity": 2,
                        "notify_timeout_secs": 1
                    }
                }
            })
            .to_string(),
            "/tmp/managed-wrapper-frozen.json",
        )
        .unwrap()
        .resolved_definition("fixture")
        .unwrap(),
    );
    let shared = Arc::new(Mutex::new(state));
    let listener = UnixListener::bind(&config.socket_path).unwrap();
    let connection_state = Arc::clone(&shared);
    let connection = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        handle_connection(stream, connection_state).unwrap();
    });
    let (event_tx, event_rx) = mpsc::channel();
    let _supervisor = ManagedSupervisorGuard::from_existing_sender(
        managed_tx,
        managed_rx,
        Arc::clone(&shared.lock().unwrap().fleet),
        &config,
        event_tx,
        Some(managed_test_binary()),
    );
    let (managed_tx, stop) = {
        let state = shared.lock().unwrap();
        (
            state.managed_tx.clone(),
            Arc::clone(&state.managed_spawns[&lease.command_id].stop),
        )
    };
    managed_tx
        .send(ManagedSupervisorCommand::Start { prepared, stop })
        .unwrap();

    let ready_deadline = Instant::now() + Duration::from_secs(5);
    loop {
        drain_managed_events(&shared, &event_rx);
        let registered = shared
            .lock()
            .unwrap()
            .managed_spawns
            .get(&lease.command_id)
            .and_then(|record| record.wrapper_conn.as_ref())
            .is_some();
        if registered {
            break;
        }
        assert!(
            Instant::now() < ready_deadline,
            "le wrapper réel ne s'est pas enregistré"
        );
        thread::sleep(Duration::from_millis(10));
    }
    let marker_store = ManagedMarkerStore::at_directory(cache.join("managed"));
    let marker = marker_store
        .load("89000000-0000-4000-8000-000000000102")
        .unwrap();
    assert!(crate::managed_process::group_exists(marker.pgid).unwrap());

    let stop_state = Arc::clone(&shared);
    let (result_tx, result_rx) = mpsc::channel();
    thread::spawn(move || {
        let result = handle_wrapper_message(
            "control",
            WrapperToDaemon::StopOrder {
                agent_id: "89000000-0000-4000-8000-000000000102".to_string(),
                command_id: "stop-e2e".to_string(),
            },
            &stop_state,
        );
        result_tx.send(result).unwrap();
    });
    let stop_deadline = Instant::now() + Duration::from_secs(5);
    let result = loop {
        drain_managed_events(&shared, &event_rx);
        if let Ok(result) = result_rx.try_recv() {
            break result;
        }
        if Instant::now() >= stop_deadline {
            let pgid = marker.pgid as libc::pid_t;
            // SAFETY: kill ne déréférence aucun pointeur ; ses arguments sont des entiers.
            let _ = unsafe { libc::kill(-pgid, libc::SIGTERM) };
            let cleanup_deadline = Instant::now() + Duration::from_millis(500);
            while crate::managed_process::group_exists(marker.pgid).unwrap_or(false)
                && Instant::now() < cleanup_deadline
            {
                thread::sleep(Duration::from_millis(20));
            }
            if crate::managed_process::group_exists(marker.pgid).unwrap_or(false) {
                // SAFETY: kill ne déréférence aucun pointeur ; ses arguments sont des entiers.
                let _ = unsafe { libc::kill(-pgid, libc::SIGKILL) };
            }
            panic!(
                "la chaîne réelle stop n'a pas produit d'issue : le superviseur n'a pas observé la terminaison de l'adaptateur après Disconnect (groupe nettoyé directement)"
            );
        }
        thread::sleep(Duration::from_millis(10));
    };
    assert!(matches!(
        result,
        Some(DaemonToWrapper::StopResult {
            outcome: StopOutcome::Stopped | StopOutcome::StoppedForced { .. },
            ..
        })
    ));
    assert!(!crate::managed_process::group_exists(marker.pgid).unwrap());
    assert!(
        marker_store
            .load("89000000-0000-4000-8000-000000000102")
            .is_err()
    );
    {
        let state = shared.lock().unwrap();
        assert!(state.managed_spawns.is_empty());
        assert!(
            state
                .router
                .get_agent("89000000-0000-4000-8000-000000000102")
                .is_none()
        );
        assert_eq!(state.presences[&lease.instance_id].state, "stopped");
    }
    connection.join().unwrap();
    drop(shared);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn stop_marqueur_perime_retombe_sur_not_managed_sans_tuer_le_wrapper_terminal() {
    let (state, config) = state_with_registered_agent("stop-stale-fallback");
    let child = crate::managed_process::tests::spawn_test_bootstrap()
        .wait_ready()
        .unwrap();
    let owned_pid = child.ready().pid;
    let owned_pgid = child.ready().pgid;
    assert_eq!(owned_pid, owned_pgid);
    // SAFETY: getpgrp ne déréférence aucun pointeur ; ses arguments sont des entiers.
    assert_ne!(owned_pgid, unsafe { libc::getpgrp() } as u32);
    state
        .marker_store
        .persist(
            "89000000-0000-4000-8000-000000000102",
            &crate::managed_process::BootstrapReady {
                pid: owned_pid,
                pgid: owned_pgid,
                birth: u64::MAX,
                instance_id: "ancienne-instance".to_string(),
                command_id: "ancienne-commande".to_string(),
                generation: 1,
            },
        )
        .unwrap();
    let marker_store = state.marker_store.clone();
    let shared = Arc::new(Mutex::new(state));
    let response = handle_wrapper_message(
        "control",
        WrapperToDaemon::StopOrder {
            agent_id: "89000000-0000-4000-8000-000000000102".to_string(),
            command_id: "stop-stale-fallback".to_string(),
        },
        &shared,
    );
    let status = child.abandon().wait().unwrap();
    assert_eq!(
        status.code(),
        Some(125),
        "abandon du bootstrap par EOF, jamais terminaison par signal"
    );
    assert!(matches!(
        response,
        Some(DaemonToWrapper::StopResult {
            outcome: StopOutcome::NotManaged,
            ..
        })
    ));
    assert!(
        marker_store
            .load("89000000-0000-4000-8000-000000000102")
            .is_err()
    );
    assert!(
        shared
            .lock()
            .unwrap()
            .router
            .get_agent("89000000-0000-4000-8000-000000000102")
            .is_some()
    );
    let marker_directory = config.db_path.parent().unwrap().join("managed");
    let _ = std::fs::remove_file(config.db_path);
    let _ = std::fs::remove_dir_all(marker_directory);
}

#[test]
fn startup_failed_commande_absente_repond_le_motif_du_canal_sans_residu() {
    let (mut state, config) = state_with_registered_agent("managed-startup-failed");
    state.router.unregister_by_conn("conn-1");
    state.conn_instances.remove("conn-1");
    state.presences.remove("instance-1");
    let (requester, mut requester_reader) = control_socket("managed-requester");
    state.connections.insert("requester".to_string(), requester);
    let lease = managed_test_lease("command-missing");
    state
        .managed_by_instance
        .insert(lease.instance_id.clone(), lease.command_id.clone());
    state.managed_spawns.insert(
        lease.command_id.clone(),
        ManagedSpawnRecord {
            lease: lease.clone(),
            agent_type: "fixture".to_string(),
            requester_conns: vec!["requester".to_string()],
            wrapper_conn: None,
            stop: Arc::new(ManagedStopControl::new()),
        },
    );
    let shared = Arc::new(Mutex::new(state));
    let (event_tx, event_rx) = mpsc::channel();
    event_tx
        .send(ManagedSupervisorEvent::Failed {
            lease: lease.clone(),
            kind: "command_missing".to_string(),
            reason: "/adaptateur/disparu".to_string(),
            conn_id: None,
        })
        .unwrap();
    drain_managed_events(&shared, &event_rx);
    assert!(matches!(
        read_control(&mut requester_reader),
        DaemonToWrapper::SpawnRejected {
            command_id,
            reason: SpawnRefusal::CommandMissing { command, registry }
        } if command_id == lease.command_id
            && command == "/adaptateur/disparu"
            && registry == "canal managed-status"
    ));
    let state = shared.lock().unwrap();
    assert!(state.managed_spawns.is_empty());
    assert!(state.managed_by_instance.is_empty());
    assert!(state.managed_terminal_instances.contains("instance-1"));
    drop(state);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn mort_spontanee_rejette_les_demandes_marque_stopped_et_termine_les_vues() {
    let (mut state, config) = state_with_registered_agent("managed-exit");
    let (sender_writer, mut sender_reader) = control_socket("managed-sender");
    let (attach_writer, mut attach_reader) = control_socket("managed-attach");
    state
        .connections
        .insert("sender-conn".to_string(), sender_writer);
    state
        .connections
        .insert("attach-conn".to_string(), attach_writer.clone());
    install_attach_view(&mut state, "managed-sub", "attach-conn", &attach_writer);
    state.pending_replies.push(PendingReply {
        msg_id: "managed-request".to_string(),
        from: "sender".to_string(),
        from_conn: "sender-conn".to_string(),
        to: "89000000-0000-4000-8000-000000000102".to_string(),
        target_conn: "conn-1".to_string(),
        timeout_secs: 60,
        created_at: Instant::now(),
        frozen_secs: 0,
        escalation_level: 0,
        deferred_level: None,
    });
    let lease = managed_test_lease("command-exit");
    state
        .managed_by_instance
        .insert(lease.instance_id.clone(), lease.command_id.clone());
    state.managed_spawns.insert(
        lease.command_id.clone(),
        ManagedSpawnRecord {
            lease: lease.clone(),
            agent_type: "fixture".to_string(),
            requester_conns: Vec::new(),
            wrapper_conn: Some("conn-1".to_string()),
            stop: Arc::new(ManagedStopControl::new()),
        },
    );
    let executable = std::env::current_exe()
        .unwrap()
        .parent()
        .and_then(std::path::Path::parent)
        .unwrap()
        .join("bridget");
    assert!(executable.exists(), "binaire bridget de test absent");
    // TMPDIR du runner peut déjà être long : le bootstrap initialise une
    // socket Unix sous cette racine, limitée par sockaddr_un sur macOS.
    let process_root =
        PathBuf::from("/tmp").join(format!("b89death-{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&process_root).unwrap();
    let launch = ManagedLaunch {
        bootstrap_executable: executable,
        identity: ManagedIdentity {
            instance_id: lease.instance_id.clone(),
            command_id: lease.command_id.clone(),
            generation: lease.generation,
        },
        wrapper_executable: PathBuf::from("/bin/sh"),
        wrapper_args: vec!["-c".to_string(), "exit 7".to_string()],
        cwd: process_root.clone(),
        env: managed_test_prepared(&lease, &process_root).env,
    };
    let marker_store = ManagedMarkerStore::at_directory(process_root.join("managed"));
    // Garder le diagnostic du vrai bootstrap si sa préparation échoue :
    // EOF seul ne permet pas de distinguer un refus de namespace d'un crash.
    let child = crate::managed_process::spawn_managed_bootstrap_with_stderr(
        &launch,
        std::process::Stdio::inherit(),
    )
    .unwrap()
    .wait_ready()
    .unwrap()
    .persist_marker(&marker_store, "89000000-0000-4000-8000-000000000102")
    .unwrap()
    .release()
    .unwrap();
    let prepared = PreparedSpawn {
        specialist_authority: None,
        lease: lease.clone(),
        agent_type: "fixture".to_string(),
        command: "/bin/sh".to_string(),
        args: vec!["-c".to_string(), "exit 7".to_string()],
        resolved_definition: Box::new(bridget_transport::ResolvedAgentDefinition {
            command: "/bin/sh".to_string(),
            args: vec!["-c".to_string(), "exit 7".to_string()],
            protocol: "acp".to_string(),
            forbidden_env: Vec::new(),
            pass_env: Vec::new(),
            claude_config_dir: None,
            codex_home: None,
            permissions: "allow".to_string(),
            queue_capacity: 32,
            notify_timeout_secs: 600,
            mcp: bridget_transport::ResolvedMcpDefinition {
                interactive: "none".to_string(),
                acp_session: false,
            },
            capabilities: bridget_transport::AdapterCapabilities::default(),
            digest: "fixture-digest".to_string(),
        }),
        cwd: process_root.clone(),
        env: managed_test_prepared(&lease, &process_root).env,
    };
    let mut active = HashMap::from([(
        lease.instance_id.clone(),
        SupervisedProcess {
            prepared,
            child: SupervisedChild(child),
            registered: Some((
                "conn-1".to_string(),
                "89000000-0000-4000-8000-000000000102".to_string(),
            )),
            connected: true,
            failure_sent: false,
            stop: Arc::new(ManagedStopControl::new()),
            stop_attempted: false,
        },
    )]);
    let (observed_tx, observed_rx) = mpsc::channel();
    let observed = loop {
        poll_managed_processes(&state.fleet, &observed_tx, &mut active);
        if let Ok(event) = observed_rx.try_recv() {
            break event;
        }
        thread::sleep(Duration::from_millis(10));
    };
    assert!(matches!(
        &observed,
        ManagedSupervisorEvent::Exited { reason, .. }
            if reason.contains("exit status: 7")
    ));
    let shared = Arc::new(Mutex::new(state));
    let (event_tx, event_rx) = mpsc::channel();
    event_tx.send(observed).unwrap();
    drain_managed_events(&shared, &event_rx);
    assert!(matches!(
        read_control(&mut sender_reader),
        DaemonToWrapper::DeliveryRejected { id, reason }
            if id == "managed-request" && reason.contains("exit status: 7")
    ));
    assert!(matches!(
        read_control(&mut attach_reader),
        DaemonToWrapper::End { subscription_id, reason }
            if subscription_id == "managed-sub" && reason == "wrapper indisponible"
    ));
    let state = shared.lock().unwrap();
    assert_eq!(state.presences["instance-1"].state, "stopped");
    assert!(state.pending_replies.is_empty());
    assert!(state.attach_subscriptions.is_empty());
    assert!(
        state
            .router
            .get_agent("89000000-0000-4000-8000-000000000102")
            .is_none()
    );
    drop(state);
    let _ = std::fs::remove_dir_all(process_root);
    let _ = std::fs::remove_file(config.db_path);
}

// T028 : quatre chemins réels partagent ce montage de ronde et ses sockets.
fn native153_reponse_tardive_fixture(
    label: &str,
) -> (
    Arc<Mutex<DaemonState>>,
    DaemonConfig,
    crate::delegation::Task,
    BridgetMessage,
    BufReader<UnixStream>,
    BufReader<UnixStream>,
    mpsc::Receiver<()>,
    mpsc::Receiver<ManagedSupervisorCommand>,
) {
    let (mut state, config) = state_with_registered_agent(label);
    let owner = "89000000-0000-4000-8000-000000000102";
    let child = "89000000-0000-4000-8000-000000000200";
    let (owner_writer, owner_reader) = control_socket("153p");
    let (child_writer, child_reader) = control_socket("153c");
    state.connections.insert("conn-1".into(), owner_writer);
    state.connections.insert("late-child".into(), child_writer);
    let (managed_tx, managed_rx) = mpsc::channel();
    state.managed_tx = managed_tx;
    assert!(matches!(
        handle_register_with_channel(
            "late-child",
            2,
            "claude".into(),
            child.into(),
            Some("fixture".into()),
            Some("acp".into()),
            ChannelReport::Known("unix".into()),
            Some(PresenceMode::Acp),
            None,
            Some("test".into()),
            Some("instance-late-child".into()),
            None,
            false,
            Some(false),
            &mut state,
            false,
        ),
        DaemonToWrapper::Registered {
            credential: Some(_),
            ..
        }
    ));
    assert_eq!(
        live_connection_identity(&state, "conn-1"),
        Some((owner.into(), "instance-1".into()))
    );
    assert_eq!(
        live_connection_identity(&state, "late-child"),
        Some((child.into(), "instance-late-child".into()))
    );
    assert!(sender_is_authorized(&state, "late-child", child));
    let now = unix_now_secs();
    let task: crate::delegation::Task = serde_json::from_value(serde_json::json!({
        "task_id": uuid::Uuid::new_v4().to_string(),
        "owner": owner, "owner_instance": "instance-1", "origin_owner_instance": "instance-1",
        "request": {"operation":"delegate", "request_id":"late-delegate", "agent_type":"claude",
            "model":"fixture", "task":"ronde 1", "cwd":"/tmp", "keep_alive_secs":120},
        "definition": {"command":"/bin/sh", "args":[], "protocol":"acp", "forbidden_env":[],
            "pass_env":[], "permissions":"allow", "queue_capacity":32, "notify_timeout_secs":600,
            "mcp":{"interactive":"none", "acp_session":false}, "digest":"fixture-t028"},
        "cwd":"/tmp", "child":child, "child_instance":"instance-late-child",
        "mission":uuid::Uuid::new_v4().to_string(), "created_at":now,
        "state":"result_available", "result":"résultat de la première ronde", "error":null,
        "result_sent":true, "guarded":true, "idle_deadline_at":now + 120
    }))
    .unwrap();
    state
        .delegation_store
        .insert(&task, "late-delegate", b"canon-t028")
        .unwrap();
    let current = state
        .delegation_store
        .begin_round(
            &task.task_id,
            1,
            || now,
            crate::delegation::RoundRecord {
                round: 2,
                request_id: "late-continue".into(),
                digest: "canon-r2".into(),
            },
            &uuid::Uuid::new_v4().to_string(),
            "mission de la ronde 2",
        )
        .unwrap();
    assert_eq!(current.round_no(), 2);
    assert!(
        state
            .delegation_store
            .for_mission(&task.mission)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        state
            .delegation_store
            .past_mission_task(&task.mission)
            .unwrap()
            .unwrap()
            .task_id,
        task.task_id
    );
    let mut message = BridgetMessage::new(child, owner, "réponse tardive de la ronde 1");
    message.in_reply_to = Some(task.mission);
    message.hops = 4;
    let shared = Arc::new(Mutex::new(state));
    negotiate_idempotent_client(&shared, "late-client", "012_scope_lateresponse", child);
    let receiver = {
        let state = shared.lock().unwrap();
        assert_eq!(
            live_connection_identity(&state, "late-client"),
            Some((child.into(), "instance-late-child".into()))
        );
        assert!(sender_is_authorized(&state, "late-client", &message.from));
        assert_eq!(message.to, current.owner);
        assert_eq!(message.from, current.child);
        assert_eq!(
            current.child_instance.as_deref(),
            Some("instance-late-child")
        );
        let receiver = state.delegation_store.change_receiver().unwrap();
        state.delegation_store.take_changes();
        while receiver.try_recv().is_ok() {}
        while managed_rx.try_recv().is_ok() {}
        receiver
    };
    (
        shared,
        config,
        current,
        message,
        owner_reader,
        child_reader,
        receiver,
        managed_rx,
    )
}

#[test]
fn native153_ronde_reponse_tardive_chemin_idempotent() {
    let (shared, config, current, message, mut owner_reader, _child_reader, receiver, managed_rx) =
        native153_reponse_tardive_fixture("n153-late-id");
    let (epoch, meta, pending, departures) = {
        let state = shared.lock().unwrap();
        (
            state.delegation_store.wait_epoch().get(),
            state.delegation_store.projection_meta().unwrap(),
            state.pending_replies.len(),
            state.thread_wake_departures.len(),
        )
    };
    let key = IdempotencyKey {
        issuer_scope: "012_scope_lateresponse".into(),
        operation_kind: OperationKind::Send,
        idempotency_key: message.id.clone(),
    };
    let result = handle_wrapper_message(
        "late-client",
        WrapperToDaemon::SendIdempotent {
            message: message.clone(),
            message_id: message.id.clone(),
            issued_at: unix_now_secs(),
        },
        &shared,
    );
    assert!(
        matches!(
            result,
            Some(DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::Accepted { .. },
                ..
            })
        ),
        "réponse passée absorbée : {result:?}"
    );
    let state = shared.lock().unwrap();
    assert_eq!(
        serde_json::to_value(
            state
                .delegation_store
                .get(&current.task_id)
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        serde_json::to_value(&current).unwrap()
    );
    assert_eq!(state.delegation_store.wait_epoch().get(), epoch);
    let after = state.delegation_store.projection_meta().unwrap();
    assert_eq!((after.generation, after.seq), (meta.generation, meta.seq));
    assert!(state.delegation_store.take_changes().is_empty());
    assert!(receiver.try_recv().is_err());
    assert!(managed_rx.try_recv().is_err());
    assert_eq!(state.pending_replies.len(), pending);
    assert_eq!(state.thread_wake_departures.len(), departures);
    assert!(state.store.recent_messages(10).unwrap().is_empty());
    assert!(
        state
            .idempotency
            .send_delivery_phase(&key)
            .unwrap()
            .is_none()
    );
    owner_reader.get_ref().set_nonblocking(true).unwrap();
    let mut line = String::new();
    assert_eq!(
        owner_reader.read_line(&mut line).unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    assert!(line.is_empty());
    drop(state);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn native153_ronde_reponse_tardive_chemin_ordinaire() {
    let (shared, config, current, message, mut owner_reader, _child_reader, receiver, managed_rx) =
        native153_reponse_tardive_fixture("n153-late-ord");
    let (epoch, meta, pending, departures) = {
        let state = shared.lock().unwrap();
        (
            state.delegation_store.wait_epoch().get(),
            state.delegation_store.projection_meta().unwrap(),
            state.pending_replies.len(),
            state.thread_wake_departures.len(),
        )
    };
    let result = handle_wrapper_message(
        "late-child",
        WrapperToDaemon::Send(message.clone()),
        &shared,
    );
    assert!(matches!(result, Some(DaemonToWrapper::Ack { id, .. }) if id == message.id));
    let state = shared.lock().unwrap();
    assert_eq!(
        serde_json::to_value(
            state
                .delegation_store
                .get(&current.task_id)
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        serde_json::to_value(&current).unwrap()
    );
    assert_eq!(state.delegation_store.wait_epoch().get(), epoch);
    let after = state.delegation_store.projection_meta().unwrap();
    assert_eq!((after.generation, after.seq), (meta.generation, meta.seq));
    assert!(state.delegation_store.take_changes().is_empty());
    assert!(receiver.try_recv().is_err());
    assert!(managed_rx.try_recv().is_err());
    assert_eq!(state.pending_replies.len(), pending);
    assert_eq!(state.thread_wake_departures.len(), departures);
    assert!(state.store.recent_messages(10).unwrap().is_empty());
    owner_reader.get_ref().set_nonblocking(true).unwrap();
    let mut line = String::new();
    assert_eq!(
        owner_reader.read_line(&mut line).unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    assert!(line.is_empty());
    drop(state);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn native153_ronde_reponse_tardive_rejeu() {
    let (shared, config, current, message, mut owner_reader, _child_reader, receiver, managed_rx) =
        native153_reponse_tardive_fixture("n153-late-re");
    let (epoch, meta) = {
        let state = shared.lock().unwrap();
        (
            state.delegation_store.wait_epoch().get(),
            state.delegation_store.projection_meta().unwrap(),
        )
    };
    let issued_at = unix_now_secs();
    let key = IdempotencyKey {
        issuer_scope: "012_scope_lateresponse".into(),
        operation_kind: OperationKind::Send,
        idempotency_key: message.id.clone(),
    };
    let mut first = None;
    for _ in 0..3 {
        let response = handle_wrapper_message(
            "late-client",
            WrapperToDaemon::SendIdempotent {
                message: message.clone(),
                message_id: message.id.clone(),
                issued_at,
            },
            &shared,
        );
        assert!(
            matches!(
                response,
                Some(DaemonToWrapper::IdempotencyResult {
                    issue: IdempotencyIssue::Accepted { .. },
                    ..
                })
            ),
            "rejeu passé absorbé : {response:?}"
        );
        let encoded = serde_json::to_value(response).unwrap();
        if let Some(previous) = &first {
            assert_eq!(&encoded, previous);
        } else {
            first = Some(encoded);
        }
    }
    let state = shared.lock().unwrap();
    assert_eq!(state.idempotency.record_count().unwrap(), 1);
    assert!(
        state
            .idempotency
            .send_delivery_phase(&key)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        serde_json::to_value(
            state
                .delegation_store
                .get(&current.task_id)
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        serde_json::to_value(&current).unwrap()
    );
    assert_eq!(state.delegation_store.wait_epoch().get(), epoch);
    let after = state.delegation_store.projection_meta().unwrap();
    assert_eq!((after.generation, after.seq), (meta.generation, meta.seq));
    assert!(state.delegation_store.take_changes().is_empty());
    assert!(receiver.try_recv().is_err());
    assert!(managed_rx.try_recv().is_err());
    assert!(state.pending_replies.is_empty());
    assert!(state.thread_wake_departures.is_empty());
    assert!(state.store.recent_messages(10).unwrap().is_empty());
    owner_reader.get_ref().set_nonblocking(true).unwrap();
    let mut line = String::new();
    assert_eq!(
        owner_reader.read_line(&mut line).unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    assert!(line.is_empty());
    drop(state);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn native153_ronde_reponse_tardive_expediteur_inconnu() {
    let (
        shared,
        config,
        current,
        mut message,
        mut owner_reader,
        _child_reader,
        receiver,
        managed_rx,
    ) = native153_reponse_tardive_fixture("n153-late-oth");
    let stranger = "89000000-0000-4000-8000-000000000201";
    let (writer, _stranger_reader) = control_socket("153t");
    let (epoch, meta) = {
        let mut state = shared.lock().unwrap();
        state.connections.insert("late-stranger".into(), writer);
        assert!(matches!(
            handle_register_with_channel(
                "late-stranger",
                2,
                "claude".into(),
                stranger.into(),
                Some("fixture".into()),
                Some("acp".into()),
                ChannelReport::Known("unix".into()),
                Some(PresenceMode::Acp),
                None,
                Some("test".into()),
                Some("instance-stranger".into()),
                None,
                false,
                Some(false),
                &mut state,
                false,
            ),
            DaemonToWrapper::Registered {
                credential: Some(_),
                ..
            }
        ));
        assert_eq!(
            live_connection_identity(&state, "late-stranger"),
            Some((stranger.into(), "instance-stranger".into()))
        );
        assert!(sender_is_authorized(&state, "late-stranger", stranger));
        assert_ne!(current.child, stranger);
        while managed_rx.try_recv().is_ok() {}
        (
            state.delegation_store.wait_epoch().get(),
            state.delegation_store.projection_meta().unwrap(),
        )
    };
    message.from = stranger.into();
    let result = handle_wrapper_message(
        "late-stranger",
        WrapperToDaemon::Send(message.clone()),
        &shared,
    );
    assert!(matches!(result, Some(DaemonToWrapper::Ack { id, .. }) if id == message.id));
    let delivered = match read_control(&mut owner_reader) {
        DaemonToWrapper::Deliver {
            message: delivered, ..
        } => delivered,
        other => panic!("le tiers reste sur la remise ordinaire : {other:?}"),
    };
    assert_eq!(
        (
            delivered.id,
            delivered.from,
            delivered.to,
            delivered.in_reply_to,
            delivered.body
        ),
        (
            message.id,
            message.from,
            message.to,
            message.in_reply_to,
            message.body
        )
    );
    let state = shared.lock().unwrap();
    assert_eq!(
        serde_json::to_value(
            state
                .delegation_store
                .get(&current.task_id)
                .unwrap()
                .unwrap()
        )
        .unwrap(),
        serde_json::to_value(&current).unwrap()
    );
    assert_eq!(state.delegation_store.wait_epoch().get(), epoch);
    let after = state.delegation_store.projection_meta().unwrap();
    assert_eq!((after.generation, after.seq), (meta.generation, meta.seq));
    assert!(state.delegation_store.take_changes().is_empty());
    assert!(receiver.try_recv().is_err());
    assert_eq!(state.store.recent_messages(10).unwrap().len(), 1);
    drop(state);
    drop(shared);
    let _ = std::fs::remove_file(config.db_path);
}

/// (GARDE) Le plancher humain relève l'échéance sans supprimer la garde.
/// Mutant : retirer le `.max(HUMAN_REPLY_TIMEOUT_SECS)` → la demande
/// humaine retombe à 60 s et l'assertion meurt. Mutant : appliquer le
/// plancher à tout le monde → l'agent cesse d'expirer et meurt aussi.
#[test]
#[allow(non_snake_case)]
fn TEMOIN_GARDE_plancher_humain_releve_l_echeance_sans_supprimer_la_garde() {
    // Cas mesuré le 28/08 : demande `0f2c81ee89924`, fenêtre 60 s, expirée
    // en 61 s pendant que l'humain mettait huit heures à répondre.
    assert_eq!(
        effective_reply_timeout(60, false),
        60,
        "entre agents, la fenêtre demandée reste intacte"
    );
    assert!(
        effective_reply_timeout(60, true) > 61,
        "une demande impliquant un humain ne doit plus expirer en 61 secondes"
    );
    assert_eq!(
        effective_reply_timeout(60, true),
        HUMAN_REPLY_TIMEOUT_SECS,
        "le plancher humain s'applique quand la fenêtre demandée est plus courte"
    );
    // Plancher, pas plafond : un émetteur plus patient garde son délai.
    let plus_patient = HUMAN_REPLY_TIMEOUT_SECS + 3_600;
    assert_eq!(
        effective_reply_timeout(plus_patient, true),
        plus_patient,
        "le plancher ne doit jamais RACCOURCIR une échéance demandée"
    );
    // La garde subsiste : l'échéance humaine reste finie.
    assert!(
        HUMAN_REPLY_TIMEOUT_SECS < u64::MAX,
        "une file qui ne se vide jamais remplacerait une fausse alerte par une fuite"
    );
}

/// (GARDE) Sur le vrai chemin d'expiration : à 61 s, la demande vers un
/// humain n'expire pas, celle vers un agent expire. L'humain est reconnu
/// par `agent_type == "ui"`, pas par son nom.
/// Mutant : reconnaître par `name == "humain"` → un humain renommé
/// expirerait. Mutant : retirer le plancher → les DEUX expirent et le
/// compte d'expirations passe à 2.
#[test]
#[allow(non_snake_case)]
fn TEMOIN_GARDE_a_61_secondes_l_humain_n_expire_pas_et_l_agent_expire() {
    let (mut state, config) = state_with_registered_agent("garde-humain");
    // L'humain s'inscrit avec le type que pose `ui.rs`, et sous un nom qui
    // n'est PAS « humain » : la reconnaissance doit être structurelle.
    state.presences.insert(
        "instance-humain".to_string(),
        Presence {
            name: "operateur".to_string(),
            agent_type: "ui".to_string(),
            host: "localhost".to_string(),
            transport: "cli".to_string(),
            channel: Some("ssh-unix".to_string()),
            mode: None,
            location: None,
            journal_available: false,
            os: "linux".to_string(),
            state: "connected".to_string(),
            busy_since: None,
            busy_period_start: None,
            last_attested_at: None,
            turn_attest: false,
            queue_fallback: false,
            capacity_seen: Instant::now(),
            link_seen: Instant::now(),
            reconnect_count: 0,
            model: None,
            effort: None,
            rate_limits: Default::default(),
            served_model: None,
            derived_domain: None,
            domain: None,
            dnd_until: None,
            disk_space: None,
        },
    );
    // `claim_timeout` n'émet que pour une demande réellement suivie en
    // base : sans ces deux lignes, le témoin passerait au vert sur une
    // absence d'action, pas sur une garde.
    state
        .store
        .create_request("vers-humain", "bridget", "operateur", 60)
        .expect("demande suivie vers l'humain");
    state
        .store
        .create_request(
            "vers-agent",
            "bridget",
            "89000000-0000-4000-8000-000000000102",
            60,
        )
        .expect("demande suivie vers l'agent");
    let il_y_a_61s = Instant::now()
        .checked_sub(std::time::Duration::from_secs(61))
        .expect("61 secondes en arrière");
    state.pending_replies.push(PendingReply {
        msg_id: "vers-humain".to_string(),
        from: "bridget".to_string(),
        from_conn: "conn-1".to_string(),
        to: "operateur".to_string(),
        target_conn: "conn-1".to_string(),
        timeout_secs: 60,
        created_at: il_y_a_61s,
        frozen_secs: 0,
        escalation_level: 0,
        deferred_level: None,
    });
    state.pending_replies.push(PendingReply {
        msg_id: "vers-agent".to_string(),
        from: "bridget".to_string(),
        from_conn: "conn-1".to_string(),
        to: "89000000-0000-4000-8000-000000000102".to_string(),
        target_conn: "conn-1".to_string(),
        timeout_secs: 60,
        created_at: il_y_a_61s,
        frozen_secs: 0,
        escalation_level: 0,
        deferred_level: None,
    });

    let actions = collect_reminder_actions(&mut state, Instant::now());
    let expires: Vec<&str> = actions
        .iter()
        .filter_map(|action| match action {
            ReminderAction::Timeout { msg_id, .. } => Some(msg_id.as_str()),
            _ => None,
        })
        .collect();

    assert!(
        !expires.contains(&"vers-humain"),
        "la demande vers un humain ne doit PAS expirer à 61 s, reçu {expires:?}"
    );
    assert_eq!(
        expires,
        vec!["vers-agent"],
        "la garde doit rester entière vers un agent, reçu {expires:?}"
    );
    drop(state);
    let _ = std::fs::remove_file(config.db_path);
}

// T019 : quatre checkpoints durables et Prepared-only ordinaire.
#[test]
fn native160_relance_remise_reservation_absente() {
    let (mut state, config) = state_with_registered_agent("n160-remise-absente");
    let (writer, mut reader) = control_socket("n160-remise-absente");
    state.connections.insert("conn-1".into(), writer);
    let issued_at = unix_now_secs() - 7;
    let mut message = bridget_core::BridgetMessage::new(
        "humain",
        "89000000-0000-4000-8000-000000000102",
        "reprends ce corps exact",
    );
    message.id = "n160-remise-absente-message".into();
    message.origin = Some(bridget_core::MessageOrigin::Human);
    message.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    state
        .execution_store
        .admit_starting_message(&message, "n160-remise-absente-parent", issued_at)
        .unwrap();
    let crate::execution_store::ExecutionRecoveryOutcome::Reconstructed(reconstructed) = state
        .execution_store
        .reconstruct_active_for_agent(
            &message.to,
            "instance-1",
            "n160-remise-absente-child",
            issued_at,
        )
        .unwrap()
    else {
        panic!("reconstruction durable attendue");
    };
    let scope = state.idempotency.supervisor_scope().unwrap();
    let key = IdempotencyKey::new(
        scope,
        OperationKind::Send,
        format!("recovery-{}", reconstructed.snapshot.execution_id),
    )
    .unwrap();
    assert!(
        state
            .idempotency
            .stored_send_delivery(&key)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        deliver_reconstructed(
            &mut state,
            "conn-1",
            "instance-1",
            &reconstructed,
            issued_at,
            true
        )
        .unwrap(),
        "dispatching"
    );
    let stored = state
        .idempotency
        .stored_send_delivery(&key)
        .unwrap()
        .unwrap();
    assert_eq!(
        stored.delivery_id,
        format!("delivery-recovery-{}", reconstructed.snapshot.execution_id)
    );
    assert_eq!(stored.recipient_instance_id, "instance-1");
    let link = state
        .idempotency
        .delivery_execution_link(&stored.delivery_id)
        .unwrap()
        .unwrap();
    assert_eq!(link.execution_id, reconstructed.snapshot.execution_id);
    assert_eq!(link.submission_id, message.id);
    let controls = state
        .pending_post_response_controls
        .remove("conn-1")
        .unwrap();
    assert_eq!(controls.len(), 1);
    assert!(execute_controls(controls).is_empty());
    let frame = read_control(&mut reader);
    assert!(
        matches!(frame, DaemonToWrapper::DeliverIdempotent {
            ref delivery_id, ref message, execution: Some(ref execution), ..
        } if delivery_id == &stored.delivery_id && message.id == reconstructed.message.id
            && message.body == reconstructed.message.body && message.origin == reconstructed.message.origin
            && message.intent == reconstructed.message.intent
            && execution.execution_id == reconstructed.snapshot.execution_id && execution.generation == 2),
        "{frame:?}"
    );
    let before: (Vec<u8>, Vec<u8>, u64, i64, String, String) = state.store.connection().query_row(
            "SELECT r.canonical_bytes,d.message_bytes,d.delivery_generation,d.expires_at,d.recipient_instance_id,d.phase
             FROM send_deliveries d JOIN idempotency_records r USING(issuer_scope,operation_kind,idempotency_key)
             WHERE d.issuer_scope=?1 AND d.idempotency_key=?2",
            rusqlite::params![key.issuer_scope,key.idempotency_key],
            |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?)),
        ).unwrap();
    assert_eq!(
        before.0,
        canonical_send(
            &key.issuer_scope,
            &key.idempotency_key,
            &reconstructed.message,
            issued_at
        )
    );
    assert_eq!(
        before.1,
        serde_json::to_vec(&reconstructed.message).unwrap()
    );
    assert_eq!(before.5, "dispatching");
    drop(state);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn native160_relance_remise_reservation_sans_gravure() {
    let (mut state, config) = state_with_registered_agent("n160-remise-preparee");
    let (writer, mut reader) = control_socket("n160-remise-preparee");
    state.connections.insert("conn-1".into(), writer);
    let issued_at = unix_now_secs() - 7;
    let mut message = bridget_core::BridgetMessage::new(
        "humain",
        "89000000-0000-4000-8000-000000000102",
        "reprends ce corps exact",
    );
    message.id = "n160-remise-preparee-message".into();
    message.origin = Some(bridget_core::MessageOrigin::Human);
    message.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    state
        .execution_store
        .admit_starting_message(&message, "n160-remise-preparee-parent", issued_at)
        .unwrap();
    let crate::execution_store::ExecutionRecoveryOutcome::Reconstructed(reconstructed) = state
        .execution_store
        .reconstruct_active_for_agent(
            &message.to,
            "instance-1",
            "n160-remise-preparee-child",
            issued_at,
        )
        .unwrap()
    else {
        panic!("reconstruction durable attendue");
    };
    let scope = state.idempotency.supervisor_scope().unwrap();
    let key = IdempotencyKey::new(
        scope,
        OperationKind::Send,
        format!("recovery-{}", reconstructed.snapshot.execution_id),
    )
    .unwrap();
    let canonical = canonical_send(
        &key.issuer_scope,
        &key.idempotency_key,
        &reconstructed.message,
        issued_at,
    );
    let Reservation::Prepared { expires_at } = state
        .idempotency
        .reserve(
            &key,
            &canonical,
            issued_at,
            CLIENT_IDEMPOTENCY_HORIZON_SECS,
            unix_now_secs(),
            CLIENT_ISSUED_AT_TOLERANCE_SECS,
        )
        .unwrap()
    else {
        panic!("réservation préparée attendue");
    };
    assert_eq!(
        state
            .idempotency
            .prepared_expiry(&key, unix_now_secs())
            .unwrap(),
        Some(expires_at)
    );
    assert!(
        state
            .idempotency
            .send_delivery_phase(&key)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        deliver_reconstructed(
            &mut state,
            "conn-1",
            "instance-1",
            &reconstructed,
            issued_at,
            true
        )
        .unwrap(),
        "dispatching"
    );
    let stored = state
        .idempotency
        .stored_send_delivery(&key)
        .unwrap()
        .unwrap();
    assert_eq!(
        stored.delivery_id,
        format!("delivery-recovery-{}", reconstructed.snapshot.execution_id)
    );
    assert_eq!(stored.recipient_instance_id, "instance-1");
    let link = state
        .idempotency
        .delivery_execution_link(&stored.delivery_id)
        .unwrap()
        .unwrap();
    assert_eq!(link.execution_id, reconstructed.snapshot.execution_id);
    assert_eq!(link.submission_id, message.id);
    let controls = state
        .pending_post_response_controls
        .remove("conn-1")
        .unwrap();
    assert_eq!(controls.len(), 1);
    assert!(execute_controls(controls).is_empty());
    let frame = read_control(&mut reader);
    assert!(
        matches!(frame, DaemonToWrapper::DeliverIdempotent {
            ref delivery_id, ref message, execution: Some(ref execution), ..
        } if delivery_id == &stored.delivery_id && message.id == reconstructed.message.id
            && message.body == reconstructed.message.body && message.origin == reconstructed.message.origin
            && message.intent == reconstructed.message.intent
            && execution.execution_id == reconstructed.snapshot.execution_id && execution.generation == 2),
        "{frame:?}"
    );
    let before: (Vec<u8>, Vec<u8>, u64, i64, String, String) = state.store.connection().query_row(
            "SELECT r.canonical_bytes,d.message_bytes,d.delivery_generation,d.expires_at,d.recipient_instance_id,d.phase
             FROM send_deliveries d JOIN idempotency_records r USING(issuer_scope,operation_kind,idempotency_key)
             WHERE d.issuer_scope=?1 AND d.idempotency_key=?2",
            rusqlite::params![key.issuer_scope,key.idempotency_key],
            |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?)),
        ).unwrap();
    assert_eq!(
        before.0,
        canonical_send(
            &key.issuer_scope,
            &key.idempotency_key,
            &reconstructed.message,
            issued_at
        )
    );
    assert_eq!(
        before.1,
        serde_json::to_vec(&reconstructed.message).unwrap()
    );
    assert_eq!(before.5, "dispatching");
    assert_eq!(
        before.3, expires_at,
        "le rejeu ne prolonge pas la réservation"
    );
    drop(state);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn native160_relance_remise_dispatching_rejeu_sans_emission() {
    let (mut state, config) = state_with_registered_agent("n160-remise-dispatching");
    let (writer, mut reader) = control_socket("n160-remise-dispatching");
    state.connections.insert("conn-1".into(), writer);
    let issued_at = unix_now_secs() - 7;
    let mut message = bridget_core::BridgetMessage::new(
        "humain",
        "89000000-0000-4000-8000-000000000102",
        "reprends ce corps exact",
    );
    message.id = "n160-remise-dispatching-message".into();
    message.origin = Some(bridget_core::MessageOrigin::Human);
    message.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    state
        .execution_store
        .admit_starting_message(&message, "n160-remise-dispatching-parent", issued_at)
        .unwrap();
    let crate::execution_store::ExecutionRecoveryOutcome::Reconstructed(reconstructed) = state
        .execution_store
        .reconstruct_active_for_agent(
            &message.to,
            "instance-1",
            "n160-remise-dispatching-child",
            issued_at,
        )
        .unwrap()
    else {
        panic!("reconstruction durable attendue");
    };
    let scope = state.idempotency.supervisor_scope().unwrap();
    let key = IdempotencyKey::new(
        scope,
        OperationKind::Send,
        format!("recovery-{}", reconstructed.snapshot.execution_id),
    )
    .unwrap();
    assert_eq!(
        deliver_reconstructed(
            &mut state,
            "conn-1",
            "instance-1",
            &reconstructed,
            issued_at,
            true
        )
        .unwrap(),
        "dispatching"
    );
    let stored = state
        .idempotency
        .stored_send_delivery(&key)
        .unwrap()
        .unwrap();
    assert_eq!(
        stored.delivery_id,
        format!("delivery-recovery-{}", reconstructed.snapshot.execution_id)
    );
    assert_eq!(stored.recipient_instance_id, "instance-1");
    let link = state
        .idempotency
        .delivery_execution_link(&stored.delivery_id)
        .unwrap()
        .unwrap();
    assert_eq!(link.execution_id, reconstructed.snapshot.execution_id);
    assert_eq!(link.submission_id, message.id);
    let controls = state
        .pending_post_response_controls
        .remove("conn-1")
        .unwrap();
    assert_eq!(controls.len(), 1);
    assert!(execute_controls(controls).is_empty());
    let frame = read_control(&mut reader);
    assert!(
        matches!(frame, DaemonToWrapper::DeliverIdempotent {
            ref delivery_id, ref message, execution: Some(ref execution), ..
        } if delivery_id == &stored.delivery_id && message.id == reconstructed.message.id
            && message.body == reconstructed.message.body && message.origin == reconstructed.message.origin
            && message.intent == reconstructed.message.intent
            && execution.execution_id == reconstructed.snapshot.execution_id && execution.generation == 2),
        "{frame:?}"
    );
    let before: (Vec<u8>, Vec<u8>, u64, i64, String, String) = state.store.connection().query_row(
            "SELECT r.canonical_bytes,d.message_bytes,d.delivery_generation,d.expires_at,d.recipient_instance_id,d.phase
             FROM send_deliveries d JOIN idempotency_records r USING(issuer_scope,operation_kind,idempotency_key)
             WHERE d.issuer_scope=?1 AND d.idempotency_key=?2",
            rusqlite::params![key.issuer_scope,key.idempotency_key],
            |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?)),
        ).unwrap();
    assert_eq!(
        before.0,
        canonical_send(
            &key.issuer_scope,
            &key.idempotency_key,
            &reconstructed.message,
            issued_at
        )
    );
    assert_eq!(
        before.1,
        serde_json::to_vec(&reconstructed.message).unwrap()
    );
    assert_eq!(before.5, "dispatching");
    for _ in 0..2 {
        assert_eq!(
            deliver_reconstructed(
                &mut state,
                "conn-1",
                "instance-1",
                &reconstructed,
                issued_at,
                true
            )
            .unwrap(),
            "dispatching"
        );
        assert!(
            state
                .pending_post_response_controls
                .get("conn-1")
                .is_none_or(Vec::is_empty)
        );
        let after: (Vec<u8>, Vec<u8>, u64, i64, String, String) = state.store.connection().query_row(
            "SELECT r.canonical_bytes,d.message_bytes,d.delivery_generation,d.expires_at,d.recipient_instance_id,d.phase
             FROM send_deliveries d JOIN idempotency_records r USING(issuer_scope,operation_kind,idempotency_key)
             WHERE d.issuer_scope=?1 AND d.idempotency_key=?2",
            rusqlite::params![key.issuer_scope,key.idempotency_key],
            |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?)),
        ).unwrap();
        assert_eq!(
            after, before,
            "canonique, corps, génération, échéance et instance restent figés"
        );
    }
    reader.get_ref().set_nonblocking(true).unwrap();
    let mut line = String::new();
    assert_eq!(
        reader.read_line(&mut line).unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    drop(state);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn native160_relance_remise_acked_seule_phase_de_solde() {
    let (mut state, config) = state_with_registered_agent("n160-remise-acked");
    let (writer, mut reader) = control_socket("n160-remise-acked");
    state.connections.insert("conn-1".into(), writer);
    let issued_at = unix_now_secs() - 7;
    let mut message = bridget_core::BridgetMessage::new(
        "humain",
        "89000000-0000-4000-8000-000000000102",
        "reprends ce corps exact",
    );
    message.id = "n160-remise-acked-message".into();
    message.origin = Some(bridget_core::MessageOrigin::Human);
    message.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    state
        .execution_store
        .admit_starting_message(&message, "n160-remise-acked-parent", issued_at)
        .unwrap();
    let crate::execution_store::ExecutionRecoveryOutcome::Reconstructed(reconstructed) = state
        .execution_store
        .reconstruct_active_for_agent(
            &message.to,
            "instance-1",
            "n160-remise-acked-child",
            issued_at,
        )
        .unwrap()
    else {
        panic!("reconstruction durable attendue");
    };
    let scope = state.idempotency.supervisor_scope().unwrap();
    let key = IdempotencyKey::new(
        scope,
        OperationKind::Send,
        format!("recovery-{}", reconstructed.snapshot.execution_id),
    )
    .unwrap();
    assert_eq!(
        deliver_reconstructed(
            &mut state,
            "conn-1",
            "instance-1",
            &reconstructed,
            issued_at,
            true
        )
        .unwrap(),
        "dispatching"
    );
    let stored = state
        .idempotency
        .stored_send_delivery(&key)
        .unwrap()
        .unwrap();
    assert_eq!(
        stored.delivery_id,
        format!("delivery-recovery-{}", reconstructed.snapshot.execution_id)
    );
    assert_eq!(stored.recipient_instance_id, "instance-1");
    let link = state
        .idempotency
        .delivery_execution_link(&stored.delivery_id)
        .unwrap()
        .unwrap();
    assert_eq!(link.execution_id, reconstructed.snapshot.execution_id);
    assert_eq!(link.submission_id, message.id);
    let controls = state
        .pending_post_response_controls
        .remove("conn-1")
        .unwrap();
    assert_eq!(controls.len(), 1);
    assert!(execute_controls(controls).is_empty());
    let frame = read_control(&mut reader);
    assert!(
        matches!(frame, DaemonToWrapper::DeliverIdempotent {
            ref delivery_id, ref message, execution: Some(ref execution), ..
        } if delivery_id == &stored.delivery_id && message.id == reconstructed.message.id
            && message.body == reconstructed.message.body && message.origin == reconstructed.message.origin
            && message.intent == reconstructed.message.intent
            && execution.execution_id == reconstructed.snapshot.execution_id && execution.generation == 2),
        "{frame:?}"
    );
    // Vrai handler d’ACK : instance de connexion et génération de la remise vérifiées.
    assert!(
        handle_delivery_ack(
            "conn-1",
            stored.delivery_id.clone(),
            stored.delivery_generation,
            &mut state
        )
        .is_none()
    );
    let before: (Vec<u8>, Vec<u8>, u64, i64, String, String) = state.store.connection().query_row(
            "SELECT r.canonical_bytes,d.message_bytes,d.delivery_generation,d.expires_at,d.recipient_instance_id,d.phase
             FROM send_deliveries d JOIN idempotency_records r USING(issuer_scope,operation_kind,idempotency_key)
             WHERE d.issuer_scope=?1 AND d.idempotency_key=?2",
            rusqlite::params![key.issuer_scope,key.idempotency_key],
            |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?)),
        ).unwrap();
    assert_eq!(
        before.0,
        canonical_send(
            &key.issuer_scope,
            &key.idempotency_key,
            &reconstructed.message,
            issued_at
        )
    );
    assert_eq!(
        before.1,
        serde_json::to_vec(&reconstructed.message).unwrap()
    );
    assert_eq!(before.5, "acked");
    for _ in 0..2 {
        // Ancien ACK conservé, sans réaffectation. Gardes du caller hors de cet oracle.
        assert_eq!(
            deliver_reconstructed(
                &mut state,
                "conn-1",
                "instance-after-reconnect",
                &reconstructed,
                issued_at,
                true
            )
            .unwrap(),
            "acked"
        );
        assert!(
            state
                .pending_post_response_controls
                .get("conn-1")
                .is_none_or(Vec::is_empty)
        );
        let after: (Vec<u8>, Vec<u8>, u64, i64, String, String) = state.store.connection().query_row(
            "SELECT r.canonical_bytes,d.message_bytes,d.delivery_generation,d.expires_at,d.recipient_instance_id,d.phase
             FROM send_deliveries d JOIN idempotency_records r USING(issuer_scope,operation_kind,idempotency_key)
             WHERE d.issuer_scope=?1 AND d.idempotency_key=?2",
            rusqlite::params![key.issuer_scope,key.idempotency_key],
            |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?)),
        ).unwrap();
        assert_eq!(
            after, before,
            "canonique, corps, génération, échéance et instance restent figés"
        );
    }
    reader.get_ref().set_nonblocking(true).unwrap();
    let mut line = String::new();
    assert_eq!(
        reader.read_line(&mut line).unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    drop(state);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn native160_relance_remise_ordinaire_prepared_only() {
    let (mut state, config) = state_with_registered_agent("n160-remise-ordinary");
    let (writer, mut reader) = control_socket("n160-remise-ordinary");
    state.connections.insert("conn-1".into(), writer);
    let issued_at = unix_now_secs() - 7;
    let mut message = bridget_core::BridgetMessage::new(
        "humain",
        "89000000-0000-4000-8000-000000000102",
        "reprends ce corps exact",
    );
    message.id = "n160-remise-ordinary-message".into();
    message.origin = Some(bridget_core::MessageOrigin::Human);
    message.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    state
        .execution_store
        .admit_starting_message(&message, "n160-remise-ordinary-parent", issued_at)
        .unwrap();
    let crate::execution_store::ExecutionRecoveryOutcome::Reconstructed(reconstructed) = state
        .execution_store
        .reconstruct_active_for_agent(
            &message.to,
            "instance-1",
            "n160-remise-ordinary-child",
            issued_at,
        )
        .unwrap()
    else {
        panic!("reconstruction durable attendue");
    };
    let scope = state.idempotency.supervisor_scope().unwrap();
    let key = IdempotencyKey::new(
        scope,
        OperationKind::Send,
        format!("recovery-{}", reconstructed.snapshot.execution_id),
    )
    .unwrap();
    assert_eq!(
        deliver_reconstructed(
            &mut state,
            "conn-1",
            "instance-1",
            &reconstructed,
            issued_at,
            false
        )
        .unwrap(),
        "dispatching"
    );
    let stored = state
        .idempotency
        .stored_send_delivery(&key)
        .unwrap()
        .unwrap();
    assert_eq!(
        stored.delivery_id,
        format!("delivery-recovery-{}", reconstructed.snapshot.execution_id)
    );
    assert_eq!(stored.recipient_instance_id, "instance-1");
    let link = state
        .idempotency
        .delivery_execution_link(&stored.delivery_id)
        .unwrap()
        .unwrap();
    assert_eq!(link.execution_id, reconstructed.snapshot.execution_id);
    assert_eq!(link.submission_id, message.id);
    let controls = state
        .pending_post_response_controls
        .remove("conn-1")
        .unwrap();
    assert_eq!(controls.len(), 1);
    assert!(execute_controls(controls).is_empty());
    let frame = read_control(&mut reader);
    assert!(
        matches!(frame, DaemonToWrapper::DeliverIdempotent {
            ref delivery_id, ref message, execution: Some(ref execution), ..
        } if delivery_id == &stored.delivery_id && message.id == reconstructed.message.id
            && message.body == reconstructed.message.body && message.origin == reconstructed.message.origin
            && message.intent == reconstructed.message.intent
            && execution.execution_id == reconstructed.snapshot.execution_id && execution.generation == 2),
        "{frame:?}"
    );
    let before: (Vec<u8>, Vec<u8>, u64, i64, String, String) = state.store.connection().query_row(
            "SELECT r.canonical_bytes,d.message_bytes,d.delivery_generation,d.expires_at,d.recipient_instance_id,d.phase
             FROM send_deliveries d JOIN idempotency_records r USING(issuer_scope,operation_kind,idempotency_key)
             WHERE d.issuer_scope=?1 AND d.idempotency_key=?2",
            rusqlite::params![key.issuer_scope,key.idempotency_key],
            |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?)),
        ).unwrap();
    assert_eq!(
        before.0,
        canonical_send(
            &key.issuer_scope,
            &key.idempotency_key,
            &reconstructed.message,
            issued_at
        )
    );
    assert_eq!(
        before.1,
        serde_json::to_vec(&reconstructed.message).unwrap()
    );
    assert_eq!(before.5, "dispatching");
    for _ in 0..2 {
        assert!(
            matches!(
                deliver_reconstructed(
                    &mut state,
                    "conn-1",
                    "instance-1",
                    &reconstructed,
                    issued_at,
                    false
                ),
                Err(ReconstructedDeliveryError::NonReprenable(_))
            ),
            "ordinary refuse Replayed"
        );
        assert!(
            state
                .pending_post_response_controls
                .get("conn-1")
                .is_none_or(Vec::is_empty)
        );
        let after: (Vec<u8>, Vec<u8>, u64, i64, String, String) = state.store.connection().query_row(
            "SELECT r.canonical_bytes,d.message_bytes,d.delivery_generation,d.expires_at,d.recipient_instance_id,d.phase
             FROM send_deliveries d JOIN idempotency_records r USING(issuer_scope,operation_kind,idempotency_key)
             WHERE d.issuer_scope=?1 AND d.idempotency_key=?2",
            rusqlite::params![key.issuer_scope,key.idempotency_key],
            |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?)),
        ).unwrap();
        assert_eq!(
            after, before,
            "canonique, corps, génération, échéance et instance restent figés"
        );
    }
    reader.get_ref().set_nonblocking(true).unwrap();
    let mut line = String::new();
    assert_eq!(
        reader.read_line(&mut line).unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    drop(state);
    let _ = std::fs::remove_file(config.db_path);
}

#[test]
fn spec277_project_resolution_log_is_scoped_and_discards_free_data() {
    let owner = Some((
        "27700000-0000-4000-8000-000000000001".into(),
        "27700000-0000-4000-8000-000000000002".into(),
    ));
    for cause in [
        "project_resolution_empty",
        "project_authority_unestablished",
        "origin_worker_unavailable",
        "origin_proof_busy",
        "origin_proof_unavailable",
    ] {
        let value = project_resolution_diagnostic277(&owner, true, false, cause);
        assert_eq!(value["cause"], cause);
        assert_eq!(value["agent_id"], owner.as_ref().unwrap().0);
        assert_eq!(value["scope"], "this_connection");
    }
    assert_eq!(
        project_resolution_diagnostic277(&owner, false, true, "private text")["cause"],
        "connection_or_command_changed"
    );
    assert_eq!(
        project_resolution_diagnostic277(&owner, true, true, "private text")["cause"],
        "project_conflict"
    );
    let private = Some(("private-name".into(), "/private/path".into()));
    let value = project_resolution_diagnostic277(&private, true, false, "private text");
    assert!(value["agent_id"].is_null() && value["instance_id"].is_null());
    assert_eq!(value["cause"], "origin_proof_unavailable");
    assert!(!value.to_string().contains("private"));
}
