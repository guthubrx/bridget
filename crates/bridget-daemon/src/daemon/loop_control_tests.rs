use super::*;
use crate::guidance::{GuidanceAction, GuidancePath};
use bridget_core::MessageIntent;
use bridget_transport::protocol::{CommunicationProject, LoopControlOutcome, LoopControlRequest};

const TOKEN: &str = "1000000000004000800000000000000010000000000040008000000000000000";

fn loop_fixture(mode: MessageIntent) -> (Arc<Mutex<DaemonState>>, FixtureRoot, Vec<UnixStream>) {
    let (state, root, mut peers) = fixture();
    {
        let mut st = state.lock().unwrap();
        let project = CommunicationProject {
            host: "audit".into(),
            root: "/fixture/.git".into(),
        };
        for conn in ["human-loop", "loop"] {
            let (server, peer) = UnixStream::pair().unwrap();
            st.connections
                .insert(conn.into(), Arc::new(Mutex::new(BufWriter::new(server))));
            peers.push(peer);
        }
        st.connection_roles
            .insert("human-loop".into(), ConnectionRole::Attach);
        st.connection_roles
            .insert("loop".into(), ConnectionRole::Client);
        for conn in ["target", "loop"] {
            st.communication_projects.insert(
                conn.into(),
                (Some(project.clone()), Some(project.clone()), false),
            );
        }
        st.client_negotiations.insert(
            "loop".into(),
            NegotiatedClient {
                version: CLIENT_CONTRACT_VERSION,
                issuer_scope: "abcdefghijklmnop-loop264".into(),
                capabilities: vec![
                    ClientCapability::LoopControlV1,
                    ClientCapability::SendIdempotent,
                ],
            },
        );
        let authorized = super::super::loop_control::handle(
            "human-loop",
            LoopControlRequest::Authorize {
                token: TOKEN.into(),
                issuer_scope: "abcdefghijklmnop-loop264".into(),
                project,
                targets: vec![TARGET.into()],
                intent: mode,
            },
            &mut st,
        );
        assert!(
            matches!(authorized, LoopControlOutcome::Authorized { .. }),
            "{authorized:?}"
        );
        assert!(matches!(
            super::super::loop_control::handle(
                "loop",
                LoopControlRequest::Use {
                    token: TOKEN.into()
                },
                &mut st
            ),
            LoopControlOutcome::Bound { .. }
        ));
    }
    (state, FixtureRoot(root), peers)
}

#[test]
fn spec264_loop_modes_are_limited_without_human_or_parent_identity() {
    for (mode, action, other) in [
        (
            MessageIntent::SteerCurrent,
            GuidanceAction::Steer,
            GuidanceAction::Interrupt,
        ),
        (
            MessageIntent::InterruptAndStart,
            GuidanceAction::Interrupt,
            GuidanceAction::Steer,
        ),
    ] {
        let (state, _root, _peers) = loop_fixture(mode);
        let st = state.lock().unwrap();
        let facts = check_guidance(
            &st,
            Some("loop"),
            TARGET,
            action,
            GuidancePath::SendIdempotent,
            None,
            false,
            "test264",
        )
        .unwrap();
        assert!(matches!(
            facts.actor,
            crate::guidance::GuidanceActor::Loop { .. }
        ));
        assert!(!facts.actor_is_parent);
        assert!(live_connection_identity(&st, "loop").is_none());
        assert!(
            check_guidance(
                &st,
                Some("loop"),
                TARGET,
                other,
                GuidancePath::SendIdempotent,
                None,
                false,
                "other264"
            )
            .unwrap_err()
            .1
            .contains("loop_control_mode_mismatch")
        );
        assert!(
            check_guidance(
                &st,
                Some("loop"),
                TARGET,
                action,
                GuidancePath::Control,
                None,
                false,
                "control264"
            )
            .is_err()
        );
        assert!(
            check_guidance(
                &st,
                Some("loop"),
                WITNESS,
                action,
                GuidancePath::SendIdempotent,
                None,
                false,
                "foreign264"
            )
            .is_err()
        );
    }
}

#[test]
fn spec264_loop_revoke_invalidates_a_bound_connection_and_cannot_reauthorize() {
    let (state, _root, _peers) = loop_fixture(MessageIntent::InterruptAndStart);
    let mut st = state.lock().unwrap();
    assert!(matches!(
        super::super::loop_control::handle(
            "loop",
            LoopControlRequest::Revoke {
                token: TOKEN.into()
            },
            &mut st
        ),
        LoopControlOutcome::Revoked { .. }
    ));
    assert!(
        check_guidance(
            &st,
            Some("loop"),
            TARGET,
            GuidanceAction::Interrupt,
            GuidancePath::SendIdempotent,
            None,
            false,
            "revoked264"
        )
        .unwrap_err()
        .1
        .contains("loop_control_revoked")
    );
    assert!(matches!(
        super::super::loop_control::handle(
            "loop",
            LoopControlRequest::Use {
                token: TOKEN.into()
            },
            &mut st
        ),
        LoopControlOutcome::Bound { .. }
    ));
    assert!(
        check_guidance(
            &st,
            Some("loop"),
            TARGET,
            GuidanceAction::Interrupt,
            GuidancePath::SendIdempotent,
            None,
            false,
            "still-revoked264"
        )
        .unwrap_err()
        .1
        .contains("loop_control_revoked")
    );
}

#[test]
fn spec264_loop_scope_project_and_unattested_authorization_are_refused() {
    let (state, _root, _peers) = loop_fixture(MessageIntent::SteerCurrent);
    let mut st = state.lock().unwrap();
    let request = LoopControlRequest::Authorize {
        token: TOKEN.into(),
        issuer_scope: "abcdefghijklmnop-loop264".into(),
        project: CommunicationProject {
            host: "audit".into(),
            root: "/fixture/.git".into(),
        },
        targets: vec![TARGET.into()],
        intent: MessageIntent::SteerCurrent,
    };
    assert!(
        matches!(super::super::loop_control::handle("loop",request,&mut st),LoopControlOutcome::Refused {code,..} if code=="loop_authorization_requires_human")
    );
    st.client_negotiations.get_mut("loop").unwrap().issuer_scope = "other-loop".into();
    assert!(
        check_guidance(
            &st,
            Some("loop"),
            TARGET,
            GuidanceAction::Steer,
            GuidancePath::SendIdempotent,
            None,
            false,
            "scope264"
        )
        .unwrap_err()
        .1
        .contains("loop_control_scope_mismatch")
    );
    st.client_negotiations.get_mut("loop").unwrap().issuer_scope =
        "abcdefghijklmnop-loop264".into();
    st.communication_projects.remove("target");
    assert!(
        check_guidance(
            &st,
            Some("loop"),
            TARGET,
            GuidanceAction::Steer,
            GuidancePath::SendIdempotent,
            None,
            false,
            "project264"
        )
        .unwrap_err()
        .1
        .contains("loop_control_project_changed")
    );
}

#[test]
fn spec264_loop_interrupt_uses_the_existing_rate_guard() {
    let (state, _root, _peers) = loop_fixture(MessageIntent::InterruptAndStart);
    let st = state.lock().unwrap();
    let facts = check_guidance(
        &st,
        Some("loop"),
        TARGET,
        GuidanceAction::Interrupt,
        GuidancePath::SendIdempotent,
        None,
        false,
        "rate264",
    )
    .unwrap();
    record_guidance_outcome(&st, &facts, TARGET, "rate264", "admitted").unwrap();
    assert_eq!(
        check_guidance(
            &st,
            Some("loop"),
            TARGET,
            GuidanceAction::Interrupt,
            GuidancePath::SendIdempotent,
            None,
            false,
            "rate264-second"
        )
        .unwrap_err()
        .0,
        "interrupt_rate_limited"
    );
    let mut message = BridgetMessage::new("human", TARGET, "ordinary");
    assert_eq!(
        super::super::loop_control::check_message(&st, "loop", &message),
        Err("loop_control_mode_mismatch")
    );
    message.intent = Some(MessageIntent::InterruptAndStart);
    assert!(super::super::loop_control::check_message(&st, "loop", &message).is_ok());
}

#[test]
fn spec264_loop_respects_pause_and_do_not_disturb() {
    let (state, _root, _peers) = loop_fixture(MessageIntent::SteerCurrent);
    let mut st = state.lock().unwrap();
    let instance = st.conn_instances.get("target").unwrap().clone();
    st.presences.get_mut(&instance).unwrap().dnd_until =
        Some(Instant::now() + Duration::from_secs(60));
    assert!(
        check_guidance(
            &st,
            Some("loop"),
            TARGET,
            GuidanceAction::Steer,
            GuidancePath::SendIdempotent,
            None,
            false,
            "dnd264"
        )
        .unwrap_err()
        .1
        .contains("loop_target_do_not_disturb")
    );
    st.presences.get_mut(&instance).unwrap().dnd_until = None;
    st.store
        .connection()
        .execute("UPDATE control_state SET paused=1 WHERE id=1", [])
        .unwrap();
    assert_eq!(
        check_guidance(
            &st,
            Some("loop"),
            TARGET,
            GuidanceAction::Steer,
            GuidancePath::SendIdempotent,
            None,
            false,
            "paused264"
        )
        .unwrap_err()
        .0,
        "referent_paused"
    );
}
