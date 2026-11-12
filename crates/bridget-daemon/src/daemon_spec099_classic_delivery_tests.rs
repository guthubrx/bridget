use super::*;
#[path = "daemon/loop_control_tests.rs"]
mod loop_control_tests;
use bridget_core::BridgetMessage;
use std::os::unix::fs::DirBuilderExt;

const SENDER: &str = "99000000-0000-4000-8000-000000000001";
const TARGET: &str = "99000000-0000-4000-8000-000000000002";
const WITNESS: &str = "99000000-0000-4000-8000-000000000003";

#[test]
fn spec244_entree_publique_conserve_les_reponses_modernes_et_historiques_closes() {
    for state_name in ["timed_out", "cancelled", "answered"] {
        for modern in [false, true] {
            let (state, root, _peers) = fixture();
            let _cleanup = FixtureRoot(root.clone());
            let mut original = BridgetMessage::new(SENDER, TARGET, "question");
            original.id = "closed244-public".into();
            let mut reply = BridgetMessage::new(TARGET, SENDER, "réponse à conserver");
            reply.in_reply_to = Some(original.id.clone());
            if modern {
                let outbox =
                    crate::reply_outbox::ReplyOutbox::open(&root, "spec099-target").unwrap();
                reply = outbox.retain(&original, reply, unix_now_secs()).unwrap();
                assert!(
                    reply.id.starts_with("reply244-"),
                    "identifiant construit par le producteur réel"
                );
            }
            {
                let st = state.lock().unwrap();
                st.store
                    .create_request(&original.id, SENDER, TARGET, 120)
                    .unwrap();
                st.store
                    .connection()
                    .execute(
                        "UPDATE tracked_requests SET state=?1 WHERE id=?2",
                        rusqlite::params![state_name, original.id],
                    )
                    .unwrap();
            }
            for _ in 0..2 {
                let response =
                    handle_wrapper_message("target", WrapperToDaemon::Send(reply.clone()), &state)
                        .unwrap();
                assert!(
                    matches!(response, DaemonToWrapper::Ack { .. }),
                    "{response:?}"
                );
            }
            let st = state.lock().unwrap();
            assert_eq!(
                st.store.get_request(&original.id).unwrap().unwrap().state,
                state_name
            );
            let deliveries: i64 = st
                .store
                .connection()
                .query_row(
                    "SELECT count(*) FROM send_deliveries WHERE idempotency_key=?1",
                    [&reply.id],
                    |r| r.get(0),
                )
                .unwrap();
            let saved: i64 = st
                .store
                .connection()
                .query_row(
                    "SELECT count(*) FROM ledger WHERE id=?1 AND target=?2 AND body=?3",
                    rusqlite::params![reply.id, SENDER, reply.body],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(deliveries, 0);
            assert_eq!(saved, 1);
        }
    }
}

#[test]
fn spec244_reponse_tardive_conservee_sans_reveil_ni_reouverture() {
    let (state, root, _peers) = fixture();
    let _cleanup = FixtureRoot(root);
    let mut st = state.lock().unwrap();
    for closed in ["timed_out", "cancelled"] {
        let id = format!("closed244-{closed}");
        st.store.create_request(&id, SENDER, TARGET, 120).unwrap();
        st.store
            .connection()
            .execute(
                "UPDATE tracked_requests SET state=?1 WHERE id=?2",
                rusqlite::params![closed, id],
            )
            .unwrap();
        let mut reply = BridgetMessage::new(TARGET, SENDER, "réponse conservée après clôture");
        reply.id = format!("reply244-{closed}");
        reply.in_reply_to = Some(id.clone());
        for _ in 0..2 {
            assert!(matches!(
                retain_wrapper_reply("target", reply.clone(), "spec099-target", &mut st),
                DaemonToWrapper::Ack { .. }
            ));
        }
        assert_eq!(st.store.get_request(&id).unwrap().unwrap().state, closed);
        let deliveries: i64 = st
            .store
            .connection()
            .query_row(
                "SELECT count(*) FROM send_deliveries WHERE idempotency_key=?1",
                [&reply.id],
                |r| r.get(0),
            )
            .unwrap();
        let saved: i64 = st
            .store
            .connection()
            .query_row(
                "SELECT count(*) FROM ledger WHERE id=?1 AND sender=?2 AND target=?3 AND body=?4",
                rusqlite::params![reply.id, TARGET, SENDER, reply.body],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(deliveries, 0, "aucune injection après la clôture");
        assert_eq!(saved, 1, "la réponse reste consultable une fois");
    }
}

#[test]
fn spec244_reponse_wrapper_deposee_une_fois_et_identite_correlee() {
    let (state, root, _peers) = fixture();
    let _cleanup = FixtureRoot(root);
    let mut st = state.lock().unwrap();
    st.store
        .create_request("direct244", SENDER, TARGET, 120)
        .unwrap();
    let mut reply = BridgetMessage::new(TARGET, SENDER, "réponse directe");
    reply.id = "reply244-exact".into();
    reply.in_reply_to = Some("direct244".into());
    let bad = retain_wrapper_reply("witness", reply.clone(), "spec099-witness", &mut st);
    assert!(
        matches!(bad,DaemonToWrapper::Nack{reason,..} if reason=="wrapper_reply_identity_required")
    );
    let result = retain_wrapper_reply("target", reply.clone(), "spec099-target", &mut st);
    assert!(matches!(result, DaemonToWrapper::Ack { .. }), "{result:?}");
    let key = IdempotencyKey::new(
        crate::communication::issuer_scope("wrapper-reply:spec099-target"),
        OperationKind::Send,
        &reply.id,
    )
    .unwrap();
    let first = st.idempotency.stored_send_delivery(&key).unwrap().unwrap();
    let repeated = retain_wrapper_reply("target", reply.clone(), "spec099-target", &mut st);
    assert!(
        matches!(repeated, DaemonToWrapper::Ack { .. }),
        "{repeated:?}"
    );
    assert_eq!(
        st.idempotency
            .stored_send_delivery(&key)
            .unwrap()
            .unwrap()
            .delivery_id,
        first.delivery_id
    );
    reply.body = "autre corps".into();
    assert!(
        matches!(retain_wrapper_reply("target",reply,"spec099-target",&mut st),DaemonToWrapper::Nack{reason,..} if reason.contains("EnvelopeMismatch"))
    );
}

#[test]
fn tracking230_sommeil_refuse_envoi_classique_et_principal_agent() {
    let (state, root, peers) = fixture();
    let _cleanup = FixtureRoot(root);
    let request = bridget_transport::agent_tracking::AgentTrackingRequest::Set {
        request_id: Uuid::new_v4().to_string(),
        agent_id: TARGET.into(),
        dormant: true,
        synchronize: None,
        expected_revision: None,
    };
    let refused = agent_tracking::human("sender", &request, &state);
    assert!(
        matches!(refused,DaemonToWrapper::AgentTrackingResult{result} if result["code"]=="human_principal_required")
    );
    {
        let mut st = state.lock().unwrap();
        let agent = agent_tracking::directory(&mut st)
            .unwrap()
            .into_iter()
            .find(|a| a.agent.agent_id == TARGET)
            .unwrap();
        crate::agent_tracking::set(
            st.store.connection(),
            "human",
            &Uuid::new_v4().to_string(),
            b"sleep-target",
            &agent,
            true,
            Some(false),
            None,
            100,
        )
        .unwrap();
    }
    let response = handle_wrapper_message(
        "sender",
        WrapperToDaemon::Send(BridgetMessage::new(SENDER, TARGET, "demande nouvelle")),
        &state,
    )
    .unwrap();
    assert!(
        matches!(response,DaemonToWrapper::Nack{reason,..} if reason.contains("Agent dormant"))
    );
    peers[1]
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    assert!(
        BufReader::new(peers[1].try_clone().unwrap())
            .read_line(&mut String::new())
            .is_err(),
        "aucune remise au dormant"
    );
    let st = state.lock().unwrap();
    assert!(
        st.router.get_agent(TARGET).is_some(),
        "le sommeil n'arrête pas le processus"
    );
    assert_eq!(
        wake_target_guard(
            &st,
            SENDER,
            &bridget_transport::protocol::WakeTarget::Agent { id: TARGET.into() }
        ),
        Err("agent_dormant")
    );
}

#[test]
fn tracking230_reponse_admise_conservee_sans_solliciter_parent_dormant() {
    let (state, root, peers) = fixture();
    let _cleanup = FixtureRoot(root);
    let mut request = BridgetMessage::new(SENDER, TARGET, "travail déjà demandé");
    request.id = "230-admitted-request".into();
    request.reply = true;
    assert!(matches!(
        handle_wrapper_message("sender", WrapperToDaemon::Send(request), &state),
        Some(DaemonToWrapper::Ack { .. })
    ));
    let mut reader = BufReader::new(peers[1].try_clone().unwrap());
    assert!(matches!(
        presence_tests::read_control(&mut reader),
        DaemonToWrapper::Deliver { .. }
    ));
    {
        let mut st = state.lock().unwrap();
        let agent = agent_tracking::directory(&mut st)
            .unwrap()
            .into_iter()
            .find(|a| a.agent.agent_id == SENDER)
            .unwrap();
        crate::agent_tracking::set(
            st.store.connection(),
            "human",
            &Uuid::new_v4().to_string(),
            b"sleep-parent",
            &agent,
            true,
            Some(false),
            None,
            100,
        )
        .unwrap();
    }
    let mut forged = BridgetMessage::new(WITNESS, SENDER, "réponse étrangère");
    forged.in_reply_to = Some("230-admitted-request".into());
    assert!(matches!(
        handle_wrapper_message("witness", WrapperToDaemon::Send(forged), &state),
        Some(DaemonToWrapper::Nack { .. })
    ));
    let mut reply = BridgetMessage::new(TARGET, SENDER, "résultat conservé");
    reply.in_reply_to = Some("230-admitted-request".into());
    for _ in 0..2 {
        assert!(matches!(
            handle_wrapper_message("target", WrapperToDaemon::Send(reply.clone()), &state),
            Some(DaemonToWrapper::Ack { .. })
        ));
    }
    let mut conflict = reply.clone();
    conflict.body = "autre résultat avec le même identifiant".into();
    assert!(matches!(
        handle_wrapper_message("target", WrapperToDaemon::Send(conflict), &state),
        Some(DaemonToWrapper::Nack { .. })
    ));
    let st = state.lock().unwrap();
    assert_eq!(
        st.store
            .get_request("230-admitted-request")
            .unwrap()
            .unwrap()
            .state,
        "answered"
    );
    let count: i64 = st
        .store
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM ledger WHERE id=?1 AND body=?2",
            rusqlite::params![reply.id, reply.body],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        count, 1,
        "le résultat est durable et son rejeu ne l'écrase pas"
    );
    peers[0]
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    assert!(
        BufReader::new(peers[0].try_clone().unwrap())
            .read_line(&mut String::new())
            .is_err(),
        "aucun nouveau tour au parent dormant"
    );
}

#[test]
fn spec099_notification_systeme_bornee_meme_si_writer_occupe() {
    let (socket, _peer) = UnixStream::pair().unwrap();
    let writer = Arc::new(Mutex::new(BufWriter::new(socket)));
    let guard = writer.lock().unwrap();
    let blocked = writer.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    let task = std::thread::spawn(move || {
        tx.send(deliver_to_agent(&blocked, TARGET, "notification système").is_err())
            .unwrap();
    });
    let result = rx.recv_timeout(Duration::from_millis(1500));
    drop(guard);
    task.join().unwrap();
    assert_eq!(result.ok(), Some(true), "notification non bornée");
}

// Même montage socketpair que core_089_cancel_pressure_tests ; aucun
// processus fournisseur ni modification du HOME de la session.
fn fixture() -> (Arc<Mutex<DaemonState>>, PathBuf, Vec<UnixStream>) {
    let root = std::env::temp_dir().join(format!("bg099-{}", uuid::Uuid::new_v4().simple()));
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&root)
        .unwrap();
    let config = DaemonConfig {
        socket_path: root.join("s"),
        db_path: root.join("state.db"),
        log_path: root.join("log"),
        circuit_breaker_window: 180,
        circuit_breaker_limit: 8,
        dedup_window: 180,
        quarantine_window: 3600,
        retention_days: 7,
    };
    let (tx, _rx) = mpsc::channel();
    let mut state = DaemonState::new(&config, tx).unwrap();
    let mut peers = Vec::new();
    for (conn, agent) in [("sender", SENDER), ("target", TARGET), ("witness", WITNESS)] {
        let (server, peer) = UnixStream::pair().unwrap();
        state
            .connections
            .insert(conn.into(), Arc::new(Mutex::new(BufWriter::new(server))));
        let response = handle_register_with_channel(
            conn,
            2,
            "fixture".into(),
            agent.into(),
            Some("audit".into()),
            Some("acp".into()),
            ChannelReport::Known("unix".into()),
            Some(PresenceMode::Acp),
            None,
            Some("test".into()),
            Some(format!("spec099-{conn}")),
            None,
            false,
            Some(false),
            &mut state,
            false,
        );
        assert!(matches!(response, DaemonToWrapper::Registered { .. }));
        peers.push(peer);
    }
    (Arc::new(Mutex::new(state)), root, peers)
}

#[test]
fn spec138_direct_guard_and_scoped_directory_precede_delivery() {
    use bridget_transport::protocol::{CommunicationDirectoryScope, CommunicationProject};
    let (state, root, peers) = fixture();
    let _cleanup = FixtureRoot(root);
    {
        let mut st = state.lock().unwrap();
        for (conn, root) in [("sender", "/a/.git"), ("target", "/b/.git")] {
            let project = CommunicationProject {
                host: "audit".into(),
                root: root.into(),
            };
            st.communication_projects
                .insert(conn.into(), (Some(project.clone()), Some(project), false));
        }
    }
    let local = handle_wrapper_message(
        "sender",
        WrapperToDaemon::DirectoryScoped {
            scope: CommunicationDirectoryScope::SameProject,
        },
        &state,
    )
    .unwrap();
    assert!(
        matches!(local, DaemonToWrapper::CommunicationDirectory { agents, .. } if agents.len() == 1 && agents[0].agent.agent_id == SENDER)
    );
    let global = handle_wrapper_message(
        "sender",
        WrapperToDaemon::DirectoryScoped {
            scope: CommunicationDirectoryScope::Global,
        },
        &state,
    )
    .unwrap();
    assert!(
        matches!(global, DaemonToWrapper::CommunicationDirectory { agents, .. } if agents.len() == 3)
    );
    let message = BridgetMessage::new(SENDER, TARGET, "PRIVATE_BODY_exact");
    let denied =
        handle_wrapper_message("sender", WrapperToDaemon::Send(message.clone()), &state).unwrap();
    assert!(
        matches!(denied, DaemonToWrapper::Nack {reason,..} if reason.starts_with("cross_project_reason_required"))
    );
    peers[1]
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut reader = BufReader::new(peers[1].try_clone().unwrap());
    assert!(
        reader.read_line(&mut String::new()).is_err(),
        "aucune notification avant motif"
    );
    let mut accepted = message;
    accepted.cross_project_reason = Some("shared review".into());
    let response =
        handle_wrapper_message("sender", WrapperToDaemon::Send(accepted), &state).unwrap();
    assert!(
        matches!(response, DaemonToWrapper::Ack {project_warnings,..} if project_warnings.len() == 1 && project_warnings[0].code == "cross_project")
    );
    let delivered = presence_tests::read_control(&mut reader);
    assert!(
        matches!(delivered, DaemonToWrapper::Deliver { message, .. } if message.body == "PRIVATE_BODY_exact")
    );
}

#[test]
fn spec138_project_announcement_recovers_absence_but_not_conflict_and_aux_inherits() {
    use bridget_transport::protocol::{CommunicationDirectoryScope, CommunicationProjectSource};
    let (state, root, _peers) = fixture();
    let _cleanup = FixtureRoot(root.clone());
    state.lock().unwrap().host = "audit".into();
    let report = |root: String| WrapperToDaemon::CommunicationProjectFact {
        root,
        source: CommunicationProjectSource::Git,
        host: "audit".into(),
        worktree_root: None,
    };
    assert!(matches!(
        handle_wrapper_message("sender", report(String::new()), &state),
        Some(DaemonToWrapper::ProjectContextResult { project: None, .. })
    ));
    let workspace = root.join("repository");
    assert!(std::process::Command::new("git").args(["init", "--quiet"])
        .arg(&workspace).env_remove("GIT_DIR").env_remove("GIT_WORK_TREE")
        .env_remove("GIT_COMMON_DIR").status().unwrap().success());
    let workspace = workspace.to_string_lossy().into_owned();
    assert!(matches!(
        handle_wrapper_message("sender", report(workspace.clone()), &state),
        Some(DaemonToWrapper::ProjectContextResult {
            project: Some(_),
            ..
        })
    ));
    let registration = {
        let st = state.lock().unwrap();
        WrapperToDaemon::RegisterAuxiliary {
            agent_id: SENDER.into(),
            instance_id: st.conn_instances["sender"].clone(),
            credential: st.identity_credentials["sender"].clone(),
        }
    };
    assert!(matches!(
        handle_wrapper_message("aux", registration, &state),
        Some(DaemonToWrapper::Registered { .. })
    ));
    assert!(
        matches!(handle_wrapper_message("aux", WrapperToDaemon::ClientHello {contract_version: CLIENT_CONTRACT_VERSION,issuer_scope:"scope138_auxiliary_namespace".into(),capabilities:vec![ClientCapability::CommunicationProjectsV1,ClientCapability::ExecutionControlV1]}, &state), Some(DaemonToWrapper::ClientWelcome {capabilities,..}) if capabilities == vec![ClientCapability::CommunicationProjectsV1])
    );
    assert!(matches!(
        handle_wrapper_message(
            "aux",
            WrapperToDaemon::DirectoryScoped {
                scope: CommunicationDirectoryScope::SameProject
            },
            &state
        ),
        Some(DaemonToWrapper::CommunicationDirectory {
            project: Some(_),
            ..
        })
    ));
    assert!(matches!(
        handle_wrapper_message("aux", report(workspace.clone()), &state),
        Some(DaemonToWrapper::Nack { .. })
    ));
    let other = root.join("other-repository");
    assert!(
        std::process::Command::new("git")
            .arg("init")
            .arg(&other)
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(matches!(
        handle_wrapper_message(
            "sender",
            report(other.to_string_lossy().into_owned()),
            &state
        ),
        Some(DaemonToWrapper::ProjectContextResult { project: None, .. })
    ));
    assert!(matches!(
        handle_wrapper_message("sender", report(workspace), &state),
        Some(DaemonToWrapper::ProjectContextResult { project: None, .. })
    ));
}

#[test]
fn spec138_reply_grant_and_warning_survive_replay_after_closed_request() {
    use bridget_transport::protocol::CommunicationProject;
    let (state, root, _peers) = fixture();
    let _cleanup = FixtureRoot(root);
    let mut st = state.lock().unwrap();
    for (conn, root, scope) in [
        ("sender", "/a/.git", "scope138_sender_namespace"),
        ("target", "/b/.git", "scope138_target_namespace"),
    ] {
        let project = CommunicationProject {
            host: "audit".into(),
            root: root.into(),
        };
        st.communication_projects
            .insert(conn.into(), (Some(project.clone()), Some(project), false));
        st.client_negotiations.insert(
            conn.into(),
            NegotiatedClient {
                version: CLIENT_CONTRACT_VERSION,
                issuer_scope: scope.into(),
                capabilities: vec![ClientCapability::SendIdempotent],
            },
        );
    }
    let mut request = BridgetMessage::new(SENDER, TARGET, "request exact");
    request.reply = true;
    request.cross_project_reason = Some(" shared review ".into());
    let issued_at = unix_now_secs();
    let mut controls = Vec::new();
    let response = handle_idempotent_send(
        "sender",
        request,
        "request138".into(),
        issued_at,
        IdempotentSendAdmission {
            project: None,
            issued_at_tolerance_secs: CLIENT_ISSUED_AT_TOLERANCE_SECS,
        },
        &mut st,
        &mut controls,
    );
    assert!(
        matches!(response, DaemonToWrapper::IdempotencyResult {project_warnings,..} if project_warnings[0].reason.as_deref() == Some("shared review"))
    );
    assert_eq!(controls.len(), 1);
    let witness_project = CommunicationProject {
        host: "audit".into(),
        root: "/c/.git".into(),
    };
    st.communication_projects.insert(
        "witness".into(),
        (Some(witness_project.clone()), Some(witness_project), false),
    );
    st.client_negotiations.insert(
        "witness".into(),
        NegotiatedClient {
            version: CLIENT_CONTRACT_VERSION,
            issuer_scope: "scope138_witness_namespace".into(),
            capabilities: vec![ClientCapability::SendIdempotent],
        },
    );
    let mut forged_reply = BridgetMessage::new(WITNESS, SENDER, "third participant");
    forged_reply.in_reply_to = Some("request138".into());
    controls.clear();
    let forged = handle_idempotent_send(
        "witness",
        forged_reply,
        "forged138".into(),
        issued_at,
        IdempotentSendAdmission {
            project: None,
            issued_at_tolerance_secs: CLIENT_ISSUED_AT_TOLERANCE_SECS,
        },
        &mut st,
        &mut controls,
    );
    assert!(
        matches!(&forged, DaemonToWrapper::IdempotencyResult {issue: IdempotencyIssue::Rejected {category,..},..} if category == "cross_project_reason_required"),
        "{forged:?}"
    );
    assert!(controls.is_empty());
    let mut reply = BridgetMessage::new(TARGET, SENDER, "answer exact");
    reply.in_reply_to = Some("request138".into());
    let admission = || IdempotentSendAdmission {
        project: None,
        issued_at_tolerance_secs: CLIENT_ISSUED_AT_TOLERANCE_SECS,
    };
    controls.clear();
    let first = handle_idempotent_send(
        "target",
        reply.clone(),
        "reply138".into(),
        issued_at,
        admission(),
        &mut st,
        &mut controls,
    );
    assert!(
        matches!(&first, DaemonToWrapper::IdempotencyResult {project_warnings,..} if project_warnings[0].reason.as_deref() == Some("shared review")),
        "{first:?}"
    );
    assert_eq!(controls.len(), 1);
    assert!(
        st.store
            .mark_answered("request138", TARGET, SENDER, None)
            .unwrap()
    );
    controls.clear();
    let closed_request = handle_idempotent_send(
        "target",
        reply.clone(),
        "afterclose138".into(),
        issued_at,
        admission(),
        &mut st,
        &mut controls,
    );
    assert!(
        matches!(&closed_request, DaemonToWrapper::IdempotencyResult {issue: IdempotencyIssue::Rejected {category,..},..} if category == "cross_project_reason_required"),
        "{closed_request:?}"
    );
    assert!(controls.is_empty());
    st.communication_projects.clear();
    controls.clear();
    let replay = handle_idempotent_send(
        "target",
        reply.clone(),
        "reply138".into(),
        issued_at,
        admission(),
        &mut st,
        &mut controls,
    );
    assert_eq!(
        serde_json::to_value(&replay).unwrap(),
        serde_json::to_value(&first).unwrap()
    );
    assert!(controls.is_empty());
    let key =
        IdempotencyKey::new("scope138_target_namespace", OperationKind::Send, "reply138").unwrap();
    let delivery = st.idempotency.send_delivery(&key).unwrap().unwrap();
    st.idempotency
        .acknowledge_send_delivery(
            &delivery.delivery_id,
            &delivery.recipient_instance_id,
            delivery.delivery_generation,
        )
        .unwrap();
    let accepted_replay = handle_idempotent_send(
        "target",
        reply,
        "reply138".into(),
        issued_at,
        admission(),
        &mut st,
        &mut controls,
    );
    assert!(
        matches!(accepted_replay, DaemonToWrapper::IdempotencyResult {issue:IdempotencyIssue::Accepted {..},project_warnings,..} if project_warnings[0].code == "cross_project")
    );
    assert!(controls.is_empty());
}

#[test]
fn spec138_steer_guard_refuses_crossproject_before_provider() {
    use bridget_transport::protocol::CommunicationProject;
    let (state, root, mut peers) = fixture();
    let _cleanup = FixtureRoot(root);
    {
        let mut st = state.lock().unwrap();
        for (conn, root) in [("sender", "/a/.git"), ("target", "/b/.git")] {
            let project = CommunicationProject {
                host: "audit".into(),
                root: root.into(),
            };
            st.communication_projects
                .insert(conn.into(), (Some(project.clone()), Some(project), false));
        }
        st.execution_store
            .record_starting("submission138", "execution138", TARGET, 10)
            .unwrap();
        assert!(matches!(
            st.execution_store
                .transition_if_current(
                    "execution138",
                    "starting",
                    0,
                    1,
                    "running",
                    "provider_accepted",
                    11
                )
                .unwrap(),
            crate::execution_store::ConditionalTransition::Applied(_)
        ));
        st.client_negotiations.insert(
            "sender".into(),
            NegotiatedClient {
                version: CLIENT_CONTRACT_VERSION,
                issuer_scope: "scope138_control_namespace".into(),
                capabilities: vec![ClientCapability::ExecutionControlV1],
            },
        );
        st.connection_roles
            .insert("sender".into(), ConnectionRole::Client);
    }
    let mut message = BridgetMessage::new(SENDER, TARGET, "steer exact");
    message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
    let command = ExecutionControlCommand {
        version: 1,
        command_id: "control138-denied".into(),
        execution_id: "execution138".into(),
        generation: 1,
        revision: 1,
        operation: ExecutionControlOperation::SteerCurrent,
        message: Some(message),
    };
    let response = handle_wrapper_message(
        "sender",
        WrapperToDaemon::ControlExecution {
            command: command.clone(),
        },
        &state,
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(&response).unwrap()["outcome"]["refused"],
        "cross_project_reason_required",
        "{response:?}"
    );
    let mut forged = command;
    forged.command_id = "control138-forged".into();
    forged.message.as_mut().unwrap().from = TARGET.into();
    forged.message.as_mut().unwrap().cross_project_reason = Some("shared review".into());
    let refusal = handle_wrapper_message(
        "sender",
        WrapperToDaemon::ControlExecution {
            command: forged.clone(),
        },
        &state,
    )
    .unwrap();
    assert!(
        matches!(
            &refusal,
            DaemonToWrapper::ControlExecutionResult {
                outcome: ExecutionControlOutcome::Refused(ExecutionControlRefusal::InvalidCommand),
                ..
            }
        ),
        "{refusal:?}"
    );
    let replay = handle_wrapper_message(
        "sender",
        WrapperToDaemon::ControlExecution { command: forged },
        &state,
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(replay).unwrap(),
        serde_json::to_value(refusal).unwrap()
    );
    peers[1]
        .set_read_timeout(Some(Duration::from_millis(30)))
        .unwrap();
    let mut byte = [0];
    assert!(
        std::io::Read::read(&mut peers[1], &mut byte).is_err(),
        "provider received a refused control"
    );
}

#[test]
fn spec101_upstream_gap_requires_primary_and_dnd_preserves_loss_counters() {
    use bridget_transport::protocol::{ObservationKind as Kind, ObservationRequest as Request};
    let (state, _root, _peers) = fixture();
    handle_wrapper_message(
        "sender",
        WrapperToDaemon::ObservationGap { dropped: 99 },
        &state,
    );
    assert_eq!(state.lock().unwrap().observations.facts_lost, 0);
    handle_wrapper_message(
        "sender",
        WrapperToDaemon::ObservationCapabilities {
            runtime_catalogue: false,
            events: vec![Kind::TurnEnded],
        },
        &state,
    );
    handle_wrapper_message(
        "witness",
        WrapperToDaemon::ObservationRequest {
            request: Request::Sub {
                event: Kind::TurnEnded,
                agent: Some(SENDER.into()),
                file: None,
                once: true,
                ttl_secs: Some(60),
            },
        },
        &state,
    );
    {
        let mut st = state.lock().unwrap();
        st.auxiliary_connections.insert("sender".into());
        let instance = st.conn_instances["witness"].clone();
        st.presences.get_mut(&instance).unwrap().dnd_until =
            Some(Instant::now() + Duration::from_secs(30));
    }
    handle_wrapper_message(
        "sender",
        WrapperToDaemon::ObservationGap { dropped: 99 },
        &state,
    );
    assert_eq!(state.lock().unwrap().observations.facts_lost, 0);
    state.lock().unwrap().auxiliary_connections.remove("sender");
    handle_wrapper_message(
        "sender",
        WrapperToDaemon::ObservationGap { dropped: 44 },
        &state,
    );
    handle_wrapper_message(
        "sender",
        WrapperToDaemon::ObservationGap { dropped: 0 },
        &state,
    );
    let Some(DaemonToWrapper::ObservationResult { result }) = handle_wrapper_message(
        "witness",
        WrapperToDaemon::ObservationRequest {
            request: Request::List {},
        },
        &state,
    ) else {
        panic!("résultat requis")
    };
    assert_eq!(result["facts_lost"], 44);
    assert_eq!(result["observation_gaps"], 2);
    assert_eq!(result["subscriptions"][0]["facts_lost_total"], 44);
    assert_eq!(result["subscriptions"][0]["observation_gaps"], 2);
    assert_eq!(
        result["notifications_lost"], 1,
        "DND perd la notice, pas le diagnostic consultable"
    );
    assert_eq!(result["subscriptions"].as_array().unwrap().len(), 1);
}

#[test]
fn spec101_sqlite_contention_rolls_back_subscription_and_once_without_waiting() {
    use bridget_transport::protocol::{ObservationKind as Kind, ObservationRequest as Request};
    let (state, _root, _peers) = fixture();
    handle_wrapper_message(
        "sender",
        WrapperToDaemon::ObservationCapabilities {
            runtime_catalogue: false,
            events: vec![Kind::TurnEnded],
        },
        &state,
    );
    let subscribe = || WrapperToDaemon::ObservationRequest {
        request: Request::Sub {
            event: Kind::TurnEnded,
            agent: Some(SENDER.into()),
            file: None,
            once: true,
            ttl_secs: Some(60),
        },
    };
    handle_wrapper_message("witness", subscribe(), &state);
    let blocker = rusqlite::Connection::open(&state.lock().unwrap().db_path).unwrap();
    blocker.execute_batch("BEGIN IMMEDIATE").unwrap();
    let started = Instant::now();
    assert!(
        matches!(handle_wrapper_message("witness", subscribe(), &state), Some(DaemonToWrapper::ObservationResult { result }) if result["reason"] == "subscription_persistence_failed")
    );
    handle_wrapper_message(
        "sender",
        WrapperToDaemon::ObservedActivity {
            seq: 1,
            event: Kind::TurnEnded,
            file: None,
        },
        &state,
    );
    assert!(started.elapsed() < Duration::from_millis(500));
    assert_eq!(state.lock().unwrap().observations.subscriptions.len(), 1);
    blocker.execute_batch("ROLLBACK").unwrap();
    handle_wrapper_message(
        "sender",
        WrapperToDaemon::ObservedActivity {
            seq: 2,
            event: Kind::TurnEnded,
            file: None,
        },
        &state,
    );
    assert!(state.lock().unwrap().observations.subscriptions.is_empty());
    assert_eq!(
        state.lock().unwrap().store.observation_snapshot().unwrap(),
        b"[]"
    );
}

#[test]
fn spec101_capabilities_auth_persistence_and_failure() {
    use bridget_transport::protocol::{ObservationKind as Kind, ObservationRequest as Request};
    let (state, _root, _peers) = fixture();
    let request = || WrapperToDaemon::ObservationRequest {
        request: Request::Sub {
            event: Kind::TurnEnded,
            agent: Some(SENDER.into()),
            file: None,
            once: true,
            ttl_secs: Some(60),
        },
    };
    let result = handle_wrapper_message("witness", request(), &state);
    assert!(
        matches!(result, Some(DaemonToWrapper::ObservationResult {result}) if result["reason"] == "no_compatible_source")
    );
    state
        .lock()
        .unwrap()
        .auxiliary_connections
        .insert("sender".into());
    assert!(matches!(
        handle_wrapper_message(
            "sender",
            WrapperToDaemon::ObservationCapabilities {
                runtime_catalogue: false,
                events: vec![Kind::TurnEnded]
            },
            &state
        ),
        Some(DaemonToWrapper::Nack { .. })
    ));
    state.lock().unwrap().auxiliary_connections.remove("sender");
    handle_wrapper_message(
        "sender",
        WrapperToDaemon::ObservedActivity {
            seq: 1,
            event: Kind::TurnEnded,
            file: None,
        },
        &state,
    );
    assert!(state.lock().unwrap().observation_sequences.is_empty());
    handle_wrapper_message(
        "sender",
        WrapperToDaemon::ObservationCapabilities {
            runtime_catalogue: false,
            events: vec![Kind::TurnEnded, Kind::FileWritten],
        },
        &state,
    );
    let result = handle_wrapper_message("witness", request(), &state);
    assert!(
        matches!(result, Some(DaemonToWrapper::ObservationResult {result}) if result["status"] == "subscribed")
    );
    {
        let st = state.lock().unwrap();
        assert_eq!(
            serde_json::from_slice::<Vec<serde_json::Value>>(
                &st.store.observation_snapshot().unwrap()
            )
            .unwrap()
            .len(),
            1
        );
        st.store
            .connection()
            .execute_batch("PRAGMA query_only=ON")
            .unwrap();
    }
    let result = handle_wrapper_message("witness", request(), &state);
    assert!(
        matches!(result, Some(DaemonToWrapper::ObservationResult {result}) if result["reason"] == "subscription_persistence_failed")
    );
    assert_eq!(state.lock().unwrap().observations.subscriptions.len(), 1);
    // Aucun INSERT pour un fait non pertinent, même si SQLite est read-only.
    let changes = state.lock().unwrap().store.connection().total_changes();
    handle_wrapper_message(
        "sender",
        WrapperToDaemon::ObservedActivity {
            seq: 1,
            event: Kind::FileWritten,
            file: Some("/private/irrelevant".into()),
        },
        &state,
    );
    assert_eq!(state.lock().unwrap().observation_sequences["sender"], 1);
    assert_eq!(
        state.lock().unwrap().store.connection().total_changes(),
        changes
    );
    handle_wrapper_message(
        "sender",
        WrapperToDaemon::ObservedActivity {
            seq: 2,
            event: Kind::TurnEnded,
            file: None,
        },
        &state,
    );
    assert_eq!(
        state.lock().unwrap().observations.subscriptions.len(),
        1,
        "once non consommé si sauvegarde impossible"
    );
    state
        .lock()
        .unwrap()
        .store
        .connection()
        .execute_batch("PRAGMA query_only=OFF")
        .unwrap();
    handle_wrapper_message(
        "sender",
        WrapperToDaemon::ObservedActivity {
            seq: 3,
            event: Kind::TurnEnded,
            file: None,
        },
        &state,
    );
    assert_eq!(
        state.lock().unwrap().store.observation_snapshot().unwrap(),
        b"[]"
    );
}

#[test]
fn spec101_missing_agent_offline_source_and_invalid_announcements_are_refused() {
    use bridget_transport::protocol::{ObservationKind as Kind, ObservationRequest as Request};
    let (state, _root, _peers) = fixture();
    let subscribe = |agent: &str| WrapperToDaemon::ObservationRequest {
        request: Request::Sub {
            event: Kind::TurnEnded,
            agent: Some(agent.into()),
            file: None,
            once: false,
            ttl_secs: Some(60),
        },
    };
    assert!(
        matches!(handle_wrapper_message("witness", subscribe("unknown"), &state), Some(DaemonToWrapper::ObservationResult { result }) if result["reason"] == "agent_not_found")
    );
    for events in [
        vec![Kind::FileCollision],
        vec![Kind::TurnEnded, Kind::TurnEnded],
        vec![Kind::TurnEnded; 4],
    ] {
        assert!(matches!(
            handle_wrapper_message(
                "sender",
                WrapperToDaemon::ObservationCapabilities {
                    events,
                    runtime_catalogue: false
                },
                &state
            ),
            Some(DaemonToWrapper::Nack { .. })
        ));
    }
    handle_wrapper_message(
        "sender",
        WrapperToDaemon::ObservationCapabilities {
            events: vec![],
            runtime_catalogue: false,
        },
        &state,
    );
    assert!(
        matches!(handle_wrapper_message("witness", subscribe(SENDER), &state), Some(DaemonToWrapper::ObservationResult { result }) if result["reason"] == "source_unavailable")
    );
    handle_wrapper_message(
        "sender",
        WrapperToDaemon::ObservationCapabilities {
            runtime_catalogue: false,
            events: vec![Kind::TurnEnded],
        },
        &state,
    );
    handle_wrapper_message("witness", subscribe(SENDER), &state);
    handle_wrapper_message("sender", WrapperToDaemon::Unregister, &state);
    assert!(
        !state
            .lock()
            .unwrap()
            .observations
            .accepts("sender", Kind::TurnEnded)
    );
    assert!(
        matches!(handle_wrapper_message("witness", subscribe(SENDER), &state), Some(DaemonToWrapper::ObservationResult { result }) if result["reason"] == "source_unavailable")
    );
}

#[test]
fn spec100_owner_source_collision_and_nonblocking_notifications() {
    use bridget_transport::protocol::{ObservationKind as Kind, ObservationRequest as Request};
    let (state, _root, mut peers) = fixture();
    for conn in ["sender", "target"] {
        handle_wrapper_message(
            conn,
            WrapperToDaemon::ObservationCapabilities {
                runtime_catalogue: false,
                events: vec![Kind::FileWritten, Kind::TurnEnded, Kind::PermissionRequired],
            },
            &state,
        );
    }
    let request = Request::Sub {
        event: Kind::FileCollision,
        agent: None,
        file: Some("/p/*".into()),
        once: true,
        ttl_secs: Some(60),
    };
    assert!(matches!(
        handle_wrapper_message(
            "intruder",
            WrapperToDaemon::ObservationRequest {
                request: request.clone()
            },
            &state
        ),
        Some(DaemonToWrapper::Nack { .. })
    ));
    let Some(DaemonToWrapper::ObservationResult { result }) = handle_wrapper_message(
        "witness",
        WrapperToDaemon::ObservationRequest { request },
        &state,
    ) else {
        panic!("reçu requis")
    };
    let id = result["subscription"]["id"].as_str().unwrap();
    let result = handle_wrapper_message(
        "sender",
        WrapperToDaemon::ObservationRequest {
            request: Request::Unsub { id: id.into() },
        },
        &state,
    );
    assert!(
        matches!(result,Some(DaemonToWrapper::ObservationResult{result}) if result["status"]=="rejected")
    );
    for (conn, seq) in [("sender", 1), ("target", 1), ("target", 1)] {
        handle_wrapper_message(
            conn,
            WrapperToDaemon::ObservedActivity {
                seq,
                event: Kind::FileWritten,
                file: Some("/p/./x".into()),
            },
            &state,
        );
    }
    let peer = peers.pop().unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    let mut reader = BufReader::new(peer);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let delivered = decode::<DaemonToWrapper>(line.trim()).unwrap();
    let DaemonToWrapper::Deliver { message, .. } = delivered else {
        panic!("notification ordinaire attendue")
    };
    assert!(message.id.starts_with("bridget-observation:"));
    assert!(!message.reply);
    assert!(message.body.contains(SENDER) && message.body.contains(TARGET));
    assert!(message.body.contains("/p/x"));
    let writer = state.lock().unwrap().connections["witness"].clone();
    let held = writer.lock().unwrap();
    handle_wrapper_message(
        "witness",
        WrapperToDaemon::ObservationRequest {
            request: Request::Sub {
                event: Kind::TurnEnded,
                agent: None,
                file: None,
                once: false,
                ttl_secs: None,
            },
        },
        &state,
    );
    let started = Instant::now();
    for seq in 2..1000 {
        handle_wrapper_message(
            "sender",
            WrapperToDaemon::ObservedActivity {
                seq,
                event: Kind::TurnEnded,
                file: None,
            },
            &state,
        );
    }
    assert!(matches!(
        handle_wrapper_message("target", WrapperToDaemon::ListAgents, &state),
        Some(DaemonToWrapper::AgentList { .. })
    ));
    let mut witness = BridgetMessage::new(
        SENDER,
        TARGET,
        "Extrait du journal Bridget — témoin de communication",
    );
    witness.reply = true;
    let witness_id = witness.id.clone();
    assert!(matches!(
        handle_wrapper_message("sender", WrapperToDaemon::Send(witness), &state),
        Some(DaemonToWrapper::Ack { .. })
    ));
    let peer = peers.pop().unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    let mut target = BufReader::new(peer);
    let mut line = String::new();
    target.read_line(&mut line).unwrap();
    assert!(
        matches!(decode::<DaemonToWrapper>(line.trim()).unwrap(),DaemonToWrapper::Deliver { message, .. } if message.reply && message.id==witness_id)
    );
    assert!(
        state
            .lock()
            .unwrap()
            .pending_replies
            .iter()
            .any(|p| p.msg_id == witness_id)
    );
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "writer occupé ne bloque pas le dispatch : {:?}",
        started.elapsed()
    );
    // Un client auxiliaire ne peut inventer un fait pour son agent.
    state
        .lock()
        .unwrap()
        .auxiliary_connections
        .insert("sender".into());
    handle_wrapper_message(
        "sender",
        WrapperToDaemon::ObservedActivity {
            seq: 1001,
            event: Kind::TurnEnded,
            file: None,
        },
        &state,
    );
    assert_eq!(state.lock().unwrap().observation_sequences["sender"], 999);
    drop(held);
    handle_wrapper_message(
        "witness",
        WrapperToDaemon::ObservationRequest {
            request: Request::Sub {
                event: Kind::PermissionRequired,
                agent: None,
                file: None,
                once: true,
                ttl_secs: None,
            },
        },
        &state,
    );
    let before = {
        let mut st = state.lock().unwrap();
        let instance = st.conn_instances["witness"].clone();
        st.presences.get_mut(&instance).unwrap().dnd_until =
            Some(Instant::now() + Duration::from_secs(30));
        st.observation_lost
            .load(std::sync::atomic::Ordering::Relaxed)
    };
    handle_wrapper_message(
        "target",
        WrapperToDaemon::ObservedActivity {
            seq: 2,
            event: Kind::PermissionRequired,
            file: None,
        },
        &state,
    );
    assert!(
        state
            .lock()
            .unwrap()
            .observation_lost
            .load(std::sync::atomic::Ordering::Relaxed)
            > before
    );
}

#[test]
fn spec100_notification_queue_saturation_is_nonblocking_and_drains() {
    let (server, peer) = UnixStream::pair().unwrap();
    let writer = Arc::new(Mutex::new(BufWriter::new(server)));
    let held = writer.lock().unwrap();
    let (tx, lost) = observation_output();
    let start = Instant::now();
    let mut accepted = 0;
    let mut full = 0;
    for _ in 0..128 {
        match tx.try_send(ObservationJob::Control(
            Instant::now() + Duration::from_secs(5),
            DeferredControl {
                writer: writer.clone(),
                message: DaemonToWrapper::Ack {
                    project_warnings: Vec::new(),
                    id: "queue-test".into(),
                },
            },
        )) {
            Ok(()) => accepted += 1,
            Err(mpsc::TrySendError::Full(_)) => full += 1,
            Err(error) => panic!("file interrompue : {error}"),
        }
    }
    assert!((64..=65).contains(&accepted));
    assert!(full > 0);
    assert!(start.elapsed() < Duration::from_secs(1));
    drop(held);
    drop(tx);
    peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    let mut reader = BufReader::new(peer);
    for _ in 0..accepted {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert!(
            matches!(decode::<DaemonToWrapper>(line.trim()).unwrap(), DaemonToWrapper::Ack { id , ..} if id == "queue-test")
        );
    }
    assert_eq!(lost.load(std::sync::atomic::Ordering::Relaxed), 0);
}

#[test]
fn spec100_expired_notification_is_lost_without_blocking_next_delivery() {
    let (server, peer) = UnixStream::pair().unwrap();
    let writer = Arc::new(Mutex::new(BufWriter::new(server)));
    let (tx, lost) = observation_output();
    for (id, deadline) in [
        ("expired", Instant::now()),
        ("fresh", Instant::now() + Duration::from_secs(5)),
    ] {
        tx.try_send(ObservationJob::Control(
            deadline,
            DeferredControl {
                writer: writer.clone(),
                message: DaemonToWrapper::Ack {
                    project_warnings: Vec::new(),
                    id: id.into(),
                },
            },
        ))
        .unwrap();
    }
    drop(tx);
    peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    let mut line = String::new();
    BufReader::new(peer).read_line(&mut line).unwrap();
    assert!(
        matches!(decode::<DaemonToWrapper>(line.trim()).unwrap(), DaemonToWrapper::Ack { id , ..} if id == "fresh")
    );
    assert_eq!(lost.load(std::sync::atomic::Ordering::Relaxed), 1);
}

fn pressure(hold_writer: bool) {
    let (state, root, peers) = fixture();
    let writer = Arc::clone(state.lock().unwrap().connections.get("target").unwrap());
    // Filet de sécurité pour que l'ancienne implémentation ne suspende pas
    // le banc ; la borne métier reste 1 seconde, pas ces 4 secondes.
    writer
        .lock()
        .unwrap()
        .get_ref()
        .set_write_timeout(Some(Duration::from_secs(4)))
        .unwrap();
    let held = hold_writer.then(|| writer.lock().unwrap());
    let mut message = BridgetMessage::new(SENDER, TARGET, "x".repeat(1024 * 1024));
    message.id = "spec099-pressure".into();
    message.reply = hold_writer;
    let send_state = Arc::clone(&state);
    let (send_tx, send_rx) = mpsc::channel();
    let send_started = Instant::now();
    let send = thread::spawn(move || {
        send_tx
            .send(handle_wrapper_message(
                "sender",
                WrapperToDaemon::Send(message),
                &send_state,
            ))
            .unwrap();
    });
    let db = rusqlite::Connection::open_with_flags(
        root.join("state.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    db.busy_timeout(Duration::from_millis(20)).unwrap();
    let barrier = Instant::now() + Duration::from_secs(2);
    let mut admitted = false;
    while Instant::now() < barrier {
        let query = if hold_writer {
            "SELECT EXISTS(SELECT 1 FROM tracked_requests WHERE id='spec099-pressure')"
        } else {
            "SELECT EXISTS(SELECT 1 FROM ledger WHERE id='spec099-pressure')"
        };
        admitted = db
            .query_row(query, [], |row| row.get::<_, bool>(0))
            .unwrap_or(false);
        if admitted {
            break;
        }
        thread::yield_now();
    }
    let probe_state = Arc::clone(&state);
    let (probe_tx, probe_rx) = mpsc::channel();
    let probe = thread::spawn(move || {
        for _ in 0..10 {
            let start = Instant::now();
            let response =
                handle_wrapper_message("witness", WrapperToDaemon::ListAgents, &probe_state);
            let ok = matches!(response, Some(DaemonToWrapper::AgentList { .. }))
                && start.elapsed() < Duration::from_secs(1);
            probe_tx.send(ok).unwrap();
        }
        let witness = BridgetMessage::new(WITNESS, SENDER, "independent witness");
        let start = Instant::now();
        let response =
            handle_wrapper_message("witness", WrapperToDaemon::Send(witness), &probe_state);
        probe_tx
            .send(
                matches!(response, Some(DaemonToWrapper::Ack { .. }))
                    && start.elapsed() < Duration::from_secs(1),
            )
            .unwrap();
    });
    let mut probes = Vec::new();
    for _ in 0..11 {
        match probe_rx.recv_timeout(Duration::from_millis(900)) {
            Ok(ok) => probes.push(ok),
            Err(_) => break,
        }
    }
    let result = send_rx.recv_timeout(Duration::from_millis(1100));
    let elapsed = send_started.elapsed();
    // Libérer toutes les ressources AVANT les assertions, y compris sur
    // l'ancienne implémentation volontairement défaillante.
    drop(held);
    for peer in &peers {
        let _ = peer.shutdown(std::net::Shutdown::Both);
    }
    send.join().unwrap();
    probe.join().unwrap();
    drop(db);
    drop(state);
    drop(writer);
    drop(peers);
    std::fs::remove_dir_all(root).unwrap();
    assert!(admitted, "le message n'a pas atteint la frontière durable");
    assert_eq!(
        probes,
        vec![true; 11],
        "un writer bloqué retient l'annuaire ou l'échange témoin"
    );
    assert!(
        elapsed < Duration::from_secs(2),
        "remise non bornée: {elapsed:?}"
    );
    assert!(
        matches!(result, Ok(Some(DaemonToWrapper::Nack { .. }))),
        "aucun Ack sur une remise échouée: {result:?}"
    );
}

#[test]
fn spec099_socket_saturee_isolee_des_autres_agents() {
    pressure(false);
}

#[test]
fn spec099_writer_verrouille_isole_des_autres_agents() {
    pressure(true);
}

#[test]
fn spec099_t3code_recoit_annulation_structuree_sans_nouveau_tour() {
    let (state, root, peers) = fixture();
    {
        let mut state = state.lock().unwrap();
        let target = state.presences.get_mut("spec099-target").unwrap();
        target.mode = Some(PresenceMode::Cli);
        target.transport = "t3code".into();
        state
            .store
            .create_request("spec099-cancel", SENDER, TARGET, 60)
            .unwrap();
    }
    let response = handle_wrapper_message(
        "sender",
        WrapperToDaemon::CancelRequest {
            id: "spec099-cancel".into(),
            sender: SENDER.into(),
            reason: None,
        },
        &state,
    );
    assert!(matches!(
        response,
        Some(DaemonToWrapper::RequestCancelled { .. })
    ));
    let mut reader = BufReader::new(peers[1].try_clone().unwrap());
    assert!(matches!(presence_tests::read_control(&mut reader),
            DaemonToWrapper::CancelDelivery { id, .. } if id == "spec099-cancel"));
    drop(reader);
    drop(state);
    drop(peers);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec099_reponse_immediate_reste_correlable() {
    let (state, root, peers) = fixture();
    let mut message = BridgetMessage::new(SENDER, TARGET, "instant request");
    message.id = "spec099-immediate".into();
    message.reply = true;
    let response = handle_wrapper_message("sender", WrapperToDaemon::Send(message), &state);
    assert!(matches!(response, Some(DaemonToWrapper::Ack { .. })));
    let mut reader = BufReader::new(peers[1].try_clone().unwrap());
    let delivered = presence_tests::read_control(&mut reader);
    assert!(matches!(
        delivered,
        DaemonToWrapper::Deliver { message: _, .. }
    ));
    let mut reply = BridgetMessage::new(TARGET, SENDER, "instant answer");
    reply.in_reply_to = Some("spec099-immediate".into());
    let response = handle_wrapper_message("target", WrapperToDaemon::Send(reply), &state);
    assert!(matches!(response, Some(DaemonToWrapper::Ack { .. })));
    assert_eq!(
        state
            .lock()
            .unwrap()
            .store
            .get_request("spec099-immediate")
            .unwrap()
            .unwrap()
            .state,
        "answered"
    );
    drop(reader);
    drop(state);
    drop(peers);
    std::fs::remove_dir_all(root).unwrap();
}
