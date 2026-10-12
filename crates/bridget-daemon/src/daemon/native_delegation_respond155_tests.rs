//! T024/T026/T027 : réponses par les puits natifs, avec la fixture du cœur155.
use super::*;
use crate::communication::client::ClientError;
use serde_json::json;
use std::collections::HashSet;

#[test]
fn native155_respond_accept_question_nested_answer() {
    let mut fixture = RequestFixture155::new("respond-accept155");
    let request = fixture.question(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    let answer = json!({"q1":["bleu","vert"]});
    let response = {
        let mut st = fixture.state.lock().unwrap();
        handle_locked(
            "conn-1",
            NativeDelegationRequest::Respond {
                task_id: fixture.task.task_id.clone(),
                request_id: row.request_id.clone(),
                decision: ChildRequestDecision::Accept,
                answer: Some(answer.clone()),
            },
            &mut st,
        )
    }
    .unwrap();
    assert_eq!(
        response,
        json!({"version":1,"request_id":row.request_id,"state":"answered","outcome":"answered_by_parent","decision":"accept"})
    );
    native_delegation::tick(&fixture.state);
    let frames = fixture.frames();
    let delivered = frames
        .iter()
        .find_map(|frame| {
            if let DaemonToWrapper::ChildRequestAnswer { answer } = frame {
                Some(answer)
            } else {
                None
            }
        })
        .expect("la réponse parent doit atteindre l'enfant");
    assert_eq!(delivered.answer, Some(answer));
    assert_eq!(delivered.decision, ChildRequestDecision::Accept);
    assert_eq!(delivered.provider_session, request.provider_session);
    assert_eq!(delivered.turn_id, request.turn_id);
    assert_eq!(delivered.provider_request_id, request.provider_request_id);
    assert_eq!(fixture.row(&request).state, "answered");
    assert_eq!(fixture.row(&request).delivery_state, "sent");
}

#[test]
fn native155_respond_decline_question() {
    let mut fixture = RequestFixture155::new("respond-decline155");
    let request = fixture.question(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    let response = {
        let mut st = fixture.state.lock().unwrap();
        handle_locked(
            "conn-1",
            NativeDelegationRequest::Respond {
                task_id: fixture.task.task_id.clone(),
                request_id: row.request_id.clone(),
                decision: ChildRequestDecision::Decline,
                answer: None,
            },
            &mut st,
        )
    }
    .unwrap();
    assert_eq!(response["decision"], "decline");
    assert_eq!(response["state"], "answered");
    native_delegation::tick(&fixture.state);
    assert!(fixture.frames().iter().any(|frame|matches!(frame,DaemonToWrapper::ChildRequestAnswer{answer}
        if answer.request_id==row.request_id && answer.decision==ChildRequestDecision::Decline && answer.answer.is_none())));
}

#[test]
fn native155_respond_approbation_route_humaine_refusee() {
    let mut fixture = RequestFixture155::new("respond-human155");
    let request = fixture.approval(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    assert_eq!(row.verdict, "human");
    for answer in [None, Some(json!({"q1":["bleu"]}))] {
        let result = {
            let mut st = fixture.state.lock().unwrap();
            handle_locked(
                "conn-1",
                NativeDelegationRequest::Respond {
                    task_id: fixture.task.task_id.clone(),
                    request_id: row.request_id.clone(),
                    decision: ChildRequestDecision::Accept,
                    answer,
                },
                &mut st,
            )
        };
        assert_eq!(result.unwrap_err(), "parent_cannot_approve");
        assert_eq!(fixture.row(&request), row);
    }
    assert!(!fixture.frames().iter().any(|frame|matches!(frame,DaemonToWrapper::ChildRequestAnswer{answer} if answer.decision==ChildRequestDecision::Accept)));
}

#[test]
fn native155_respond_identite_proprietaire_et_ownself_refusees() {
    let fixture = RequestFixture155::new("respond-owner155");
    let request = fixture.question(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    let (stranger_writer, _stranger_reader) =
        crate::daemon::presence_tests::control_socket("stranger155");
    {
        let mut st = fixture.state.lock().unwrap();
        let host = st.host.clone();
        let stranger = Uuid::new_v4().to_string();
        let instance = Uuid::new_v4().to_string();
        assert!(matches!(
            handle_register_with_channel(
                "stranger155",
                2,
                "native-test".into(),
                stranger,
                Some(host),
                Some("codex_app_server".into()),
                ChannelReport::Known("unix".into()),
                Some(PresenceMode::Acp),
                None,
                Some("macOS".into()),
                Some(instance),
                None,
                false,
                Some(true),
                &mut st
            ),
            DaemonToWrapper::Registered { .. }
        ));
        st.connections.insert("stranger155".into(), stranger_writer);
        assert!(live_connection_identity(&st, "stranger155").is_some());
    }
    for conn in ["stranger155", "child155"] {
        let result = {
            let mut st = fixture.state.lock().unwrap();
            handle_locked(
                conn,
                NativeDelegationRequest::Respond {
                    task_id: fixture.task.task_id.clone(),
                    request_id: row.request_id.clone(),
                    decision: ChildRequestDecision::Accept,
                    answer: Some(json!({"q1":["bleu"]})),
                },
                &mut st,
            )
        };
        assert_eq!(
            result.unwrap_err(),
            "native_owner_identity_mismatch",
            "{conn}"
        );
        assert_eq!(fixture.row(&request), row);
    }
}

#[test]
fn native155_respond_rejeu_identique_et_autre_decision_close() {
    let fixture = RequestFixture155::new("respond-replay155");
    let request = fixture.question(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    let response = NativeDelegationRequest::Respond {
        task_id: fixture.task.task_id.clone(),
        request_id: row.request_id.clone(),
        decision: ChildRequestDecision::Accept,
        answer: Some(json!({"q1":["bleu"]})),
    };
    let first = {
        let mut st = fixture.state.lock().unwrap();
        handle_locked("conn-1", response.clone(), &mut st)
    }
    .unwrap();
    let decided = fixture.row(&request);
    let second = {
        let mut st = fixture.state.lock().unwrap();
        handle_locked("conn-1", response, &mut st)
    }
    .unwrap();
    assert_eq!(first, second);
    assert_eq!(fixture.row(&request), decided);
    let other = {
        let mut st = fixture.state.lock().unwrap();
        handle_locked(
            "conn-1",
            NativeDelegationRequest::Respond {
                task_id: fixture.task.task_id.clone(),
                request_id: row.request_id.clone(),
                decision: ChildRequestDecision::Decline,
                answer: None,
            },
            &mut st,
        )
    };
    assert_eq!(other.unwrap_err(), "request_closed");
    assert_eq!(fixture.row(&request), decided);
}

#[test]
fn native155_respond_decision_durable_et_non_proposee() {
    let fixture = RequestFixture155::new("respond-offer155");
    let identity = crate::mcp_identity::ResolvedIdentity {
        name: fixture.task.owner.clone(),
        instance_id: fixture.task.owner_instance.clone(),
        delegated_origin: None,
    };
    let result = crate::delegation_mcp::execute(
        &identity,
        "bridget_task_respond",
        &json!({"task_id":fixture.task.task_id,
        "request_id":"request155","decision":"acceptForSession"}),
        Path::new("/missing-native155.sock"),
    );
    assert!(
        matches!(result,Err(ClientError::Technical{code:"native_delegation_refused",message}) if message=="decision_not_offered"),
        "décision durable refusée avant la connexion"
    );
    let mut request = fixture.question(1);
    request.offered = vec![ChildRequestDecision::Accept];
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    let result = {
        let mut st = fixture.state.lock().unwrap();
        handle_locked(
            "conn-1",
            NativeDelegationRequest::Respond {
                task_id: fixture.task.task_id.clone(),
                request_id: row.request_id.clone(),
                decision: ChildRequestDecision::Decline,
                answer: None,
            },
            &mut st,
        )
    };
    assert_eq!(result.unwrap_err(), "decision_not_offered");
    assert_eq!(fixture.row(&request), row);
}

#[test]
fn native155_respond_answer_invalide() {
    let fixture = RequestFixture155::new("respond-answer155");
    let request = fixture.question(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    for (decision, answer) in [
        (ChildRequestDecision::Accept, None),
        (
            ChildRequestDecision::Accept,
            Some(json!({"other":["bleu"]})),
        ),
        (ChildRequestDecision::Accept, Some(json!({"q1":"bleu"}))),
        (
            ChildRequestDecision::Accept,
            Some(json!({"q1":{"answers":["bleu"]}})),
        ),
        (
            ChildRequestDecision::Accept,
            Some(json!({"q1":["x".repeat(8192)]})),
        ),
        (ChildRequestDecision::Decline, Some(json!({"q1":["bleu"]}))),
    ] {
        let result = {
            let mut st = fixture.state.lock().unwrap();
            handle_locked(
                "conn-1",
                NativeDelegationRequest::Respond {
                    task_id: fixture.task.task_id.clone(),
                    request_id: row.request_id.clone(),
                    decision,
                    answer,
                },
                &mut st,
            )
        };
        assert_eq!(result.unwrap_err(), "answer_invalid");
        assert_eq!(fixture.row(&request), row);
    }
}

#[test]
fn native155_respond_parent_revoque() {
    let fixture = RequestFixture155::new("respond-revoke155");
    let request = fixture.question(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    let result = {
        let mut st = fixture.state.lock().unwrap();
        st.delegation_store
            .grant_agent(
                &fixture.task.owner,
                &fixture.task.owner_instance,
                Path::new(&fixture.task.cwd),
                SpawnPosture::Development,
                true,
            )
            .unwrap();
        handle_locked(
            "conn-1",
            NativeDelegationRequest::Respond {
                task_id: fixture.task.task_id.clone(),
                request_id: row.request_id.clone(),
                decision: ChildRequestDecision::Accept,
                answer: Some(json!({"q1":["bleu"]})),
            },
            &mut st,
        )
    };
    assert_eq!(result.unwrap_err(), "permission_not_inherited");
    assert_eq!(fixture.row(&request), row);
}

#[test]
fn native155_respond_apres_echeance() {
    let mut fixture = RequestFixture155::new("respond-deadline155");
    let request = fixture.question(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    fixture.expired(&row);
    let expired = fixture.row(&request);
    let result = {
        let mut st = fixture.state.lock().unwrap();
        handle_locked(
            "conn-1",
            NativeDelegationRequest::Respond {
                task_id: fixture.task.task_id.clone(),
                request_id: row.request_id.clone(),
                decision: ChildRequestDecision::Accept,
                answer: Some(json!({"q1":["bleu"]})),
            },
            &mut st,
        )
    };
    assert_eq!(result.unwrap_err(), "request_expired");
    assert_eq!(fixture.row(&request), expired);
    assert!(!fixture.frames().iter().any(|frame|matches!(frame,DaemonToWrapper::ChildRequestAnswer{answer} if answer.decision==ChildRequestDecision::Accept)));
}

#[test]
fn native155_respond_mcp_champs_inconnus() {
    let fixture = RequestFixture155::new("respond-shape155");
    let identity = crate::mcp_identity::ResolvedIdentity {
        name: fixture.task.owner.clone(),
        instance_id: fixture.task.owner_instance.clone(),
        delegated_origin: None,
    };
    let result = crate::delegation_mcp::execute(
        &identity,
        "bridget_task_respond",
        &json!({"task_id":fixture.task.task_id,
        "request_id":"request155","decision":"accept","answer":{"q1":["bleu"]},"parent":"forged"}),
        Path::new("/missing-native155.sock"),
    );
    assert!(matches!(result,Err(ClientError::InvalidParams(message)) if message=="champ inconnu"));
}

#[test]
fn native155_respond_statut_liste_ouvertes() {
    let mut fixture = RequestFixture155::new("respond-status155");
    let question = fixture.question(1);
    let approval = fixture.approval(2);
    fixture.receive(question.clone());
    fixture.receive(approval.clone());
    let q = fixture.row(&question);
    let h = fixture.row(&approval);
    let before = {
        let mut st = fixture.state.lock().unwrap();
        handle_locked(
            "conn-1",
            NativeDelegationRequest::Status {
                task_id: fixture.task.task_id.clone(),
            },
            &mut st,
        )
    }
    .unwrap();
    let pending = before["pending_requests"]
        .as_array()
        .expect("projection additive des demandes ouvertes");
    assert_eq!(pending.len(), 2);
    assert!(
        pending
            .iter()
            .any(|r| r["request_id"] == q.request_id && r["verdict"] == "parent")
    );
    assert!(
        pending
            .iter()
            .any(|r| r["request_id"] == h.request_id && r["verdict"] == "human")
    );
    {
        let mut st = fixture.state.lock().unwrap();
        handle_locked(
            "conn-1",
            NativeDelegationRequest::Respond {
                task_id: fixture.task.task_id.clone(),
                request_id: q.request_id.clone(),
                decision: ChildRequestDecision::Accept,
                answer: Some(json!({"q1":["bleu"]})),
            },
            &mut st,
        )
        .unwrap();
    }
    native_delegation::tick(&fixture.state);
    fixture.frames();
    handle_wrapper_message(
        "child155",
        WrapperToDaemon::ChildRequestAnswered {
            request_id: q.request_id.clone(),
            applied: true,
            reason: None,
        },
        &fixture.state,
    );
    let after = {
        let mut st = fixture.state.lock().unwrap();
        handle_locked(
            "conn-1",
            NativeDelegationRequest::Status {
                task_id: fixture.task.task_id.clone(),
            },
            &mut st,
        )
    }
    .unwrap();
    assert_eq!(after["pending_requests"].as_array().unwrap().len(), 1);
    let recent = after["recent_requests"].as_array().unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0]["request_id"], q.request_id);
    assert_eq!(recent[0]["delivery_state"], "delivered");
    assert_eq!(recent[0]["decision"], "accept");
}

#[test]
fn native155_respond_statut_cles_existantes_inchangees() {
    let fixture = RequestFixture155::new("respond-keys155");
    let before = {
        let mut st = fixture.state.lock().unwrap();
        handle_locked(
            "conn-1",
            NativeDelegationRequest::Status {
                task_id: fixture.task.task_id.clone(),
            },
            &mut st,
        )
    }
    .unwrap();
    let request = fixture.question(1);
    fixture.receive(request.clone());
    let after = {
        let mut st = fixture.state.lock().unwrap();
        handle_locked(
            "conn-1",
            NativeDelegationRequest::Status {
                task_id: fixture.task.task_id.clone(),
            },
            &mut st,
        )
    }
    .unwrap();
    for key in ["pending_requests", "recent_requests"] {
        assert!(after.get(key).is_some(), "clé additive {key}");
    }
    let old = before
        .as_object()
        .unwrap()
        .keys()
        .filter(|key| !matches!(key.as_str(), "pending_requests" | "recent_requests"))
        .collect::<HashSet<_>>();
    let current = after
        .as_object()
        .unwrap()
        .keys()
        .filter(|key| !matches!(key.as_str(), "pending_requests" | "recent_requests"))
        .collect::<HashSet<_>>();
    assert_eq!(old, current);
    for key in [
        "task_id",
        "status",
        "child_agent_id",
        "message_id",
        "model",
        "posture",
        "cwd",
        "result",
        "error",
    ] {
        assert_eq!(before[key], after[key], "clé existante {key}");
    }
}

#[test]
fn native155_respond_pause_refuse() {
    let fixture = RequestFixture155::new("respond-pause155");
    let request = fixture.question(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    {
        let st = fixture.state.lock().unwrap();
        let control = crate::referent_control::read(st.store.connection()).unwrap();
        crate::referent_control::set(
            st.store.connection(),
            crate::referent_control::ControlMutation {
                command_id: "respond-pause155",
                expected_generation: control.generation,
                paused: Some(true),
                auto_objectives_cap: None,
                reason: Some("fixture"),
                actor: "humain",
                now: unix_timestamp(),
                agent_posture: None,
                auto_reassignment: None,
            },
        )
        .unwrap()
        .unwrap();
    }
    let result = {
        let mut st = fixture.state.lock().unwrap();
        handle_locked(
            "conn-1",
            NativeDelegationRequest::Respond {
                task_id: fixture.task.task_id.clone(),
                request_id: row.request_id.clone(),
                decision: ChildRequestDecision::Accept,
                answer: Some(json!({"q1":["bleu"]})),
            },
            &mut st,
        )
    };
    assert_eq!(result.unwrap_err(), "referent_paused");
    assert_eq!(fixture.row(&request), row);
    let status = {
        let mut st = fixture.state.lock().unwrap();
        handle_locked(
            "conn-1",
            NativeDelegationRequest::Status {
                task_id: fixture.task.task_id.clone(),
            },
            &mut st,
        )
    };
    assert!(status.is_ok(), "la pause n'interdit pas la lecture");
}

#[test]
fn native155_respond_apres_levee_reussit() {
    let mut fixture = RequestFixture155::new("respond-resume155");
    let old = fixture.question(1);
    fixture.receive(old.clone());
    let row = fixture.row(&old);
    {
        let st = fixture.state.lock().unwrap();
        let control = crate::referent_control::read(st.store.connection()).unwrap();
        crate::referent_control::set(
            st.store.connection(),
            crate::referent_control::ControlMutation {
                command_id: "respond-stop155",
                expected_generation: control.generation,
                paused: Some(true),
                auto_objectives_cap: None,
                reason: Some("fixture"),
                actor: "humain",
                now: unix_timestamp(),
                agent_posture: None,
                auto_reassignment: None,
            },
        )
        .unwrap()
        .unwrap();
    }
    native_delegation::tick(&fixture.state);
    assert!(fixture.frames().iter().any(|frame|matches!(frame,DaemonToWrapper::ChildRequestCancel{request_id,..} if request_id==&row.request_id)));
    handle_wrapper_message(
        "child155",
        WrapperToDaemon::ChildRequestClosed {
            request_id: row.request_id.clone(),
            state: ChildRequestClosedState::ClosedBeforeApply,
        },
        &fixture.state,
    );
    let closed = fixture.row(&old);
    assert_ne!(closed.state, "open");
    {
        let st = fixture.state.lock().unwrap();
        let control = crate::referent_control::read(st.store.connection()).unwrap();
        crate::referent_control::set(
            st.store.connection(),
            crate::referent_control::ControlMutation {
                command_id: "respond-resume155",
                expected_generation: control.generation,
                paused: Some(false),
                auto_objectives_cap: None,
                reason: Some("fixture"),
                actor: "humain",
                now: unix_timestamp(),
                agent_posture: None,
                auto_reassignment: None,
            },
        )
        .unwrap()
        .unwrap();
    }
    let mut next = fixture.question(2);
    next.turn_id = "turn-after-resume155".into();
    assert_ne!(next.turn_id, old.turn_id);
    fixture.receive(next.clone());
    let row = fixture.row(&next);
    assert_ne!(row.request_id, closed.request_id);
    assert_eq!(row.state, "open");
    let result = {
        let mut st = fixture.state.lock().unwrap();
        handle_locked(
            "conn-1",
            NativeDelegationRequest::Respond {
                task_id: fixture.task.task_id.clone(),
                request_id: row.request_id.clone(),
                decision: ChildRequestDecision::Accept,
                answer: Some(json!({"q1":["bleu"]})),
            },
            &mut st,
        )
    }
    .unwrap();
    assert_eq!(result["state"], "answered");
    assert_eq!(fixture.row(&old), closed);
    native_delegation::tick(&fixture.state);
    assert!(fixture.frames().iter().any(|frame|matches!(frame,DaemonToWrapper::ChildRequestAnswer{answer} if answer.request_id==row.request_id && answer.decision==ChildRequestDecision::Accept)));
}
