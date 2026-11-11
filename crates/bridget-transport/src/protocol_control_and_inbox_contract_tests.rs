use super::*;

fn control_state_frame() -> ControlStateFrame {
    ControlStateFrame {
        version: CONTROL_STATE_CONTRACT_VERSION,
        generation: 13,
        paused: true,
        paused_since: Some(1_788_400_000),
        paused_by: Some("humain".to_string()),
        pause_reason: Some("revue en cours".to_string()),
        auto_objectives_cap: 5,
        agent_posture: Some(AgentPosture::Complete),
        auto_reassignment: Some(true),
        updated_at: 1_788_400_000,
    }
}

#[test]
fn spec_087_trames_etat_de_controle_font_l_aller_retour() {
    let read = WrapperToDaemon::ControlStateRead {
        version: CONTROL_STATE_CONTRACT_VERSION,
    };
    let encoded = encode(&read).unwrap();
    assert!(
        encoded.contains(r#""type":"control_state_read""#),
        "{encoded}"
    );
    assert!(matches!(
        decode::<WrapperToDaemon>(&encoded).unwrap(),
        WrapperToDaemon::ControlStateRead { version: 1 }
    ));
    let focus_read = WrapperToDaemon::ControlFocusRead {
        version: CONTROL_STATE_CONTRACT_VERSION,
    };
    let encoded = encode(&focus_read).unwrap();
    assert!(
        encoded.contains(r#""type":"control_focus_read""#),
        "{encoded}"
    );
    assert!(matches!(
        decode::<WrapperToDaemon>(&encoded).unwrap(),
        WrapperToDaemon::ControlFocusRead { version: 1 }
    ));
    let focus = ControlFocusFrame {
        objective_id: "objective-1".to_string(),
        goal: "Vérifier le focus".to_string(),
        project_id: "projet-1".to_string(),
        updated_at: 42,
    };
    let publish = WrapperToDaemon::ControlFocusPublish {
        version: CONTROL_STATE_CONTRACT_VERSION,
        focus: Some(focus.clone()),
    };
    let encoded = encode(&publish).unwrap();
    assert!(
        encoded.contains(r#""type":"control_focus_publish""#),
        "{encoded}"
    );
    assert!(matches!(
        decode::<WrapperToDaemon>(&encoded).unwrap(),
        WrapperToDaemon::ControlFocusPublish { focus: Some(decoded), .. } if decoded == focus
    ));
    let set = WrapperToDaemon::ControlStateSet {
        version: CONTROL_STATE_CONTRACT_VERSION,
        command_id: "control-1".to_string(),
        expected_generation: 12,
        paused: Some(true),
        auto_objectives_cap: None,
        reason: Some("revue".to_string()),
        agent_posture: None,
        auto_reassignment: None,
    };
    let encoded = encode(&set).unwrap();
    assert!(
        !encoded.contains("auto_objectives_cap"),
        "champ nul omis : {encoded}"
    );
    let WrapperToDaemon::ControlStateSet { paused, reason, .. } = decode(&encoded).unwrap() else {
        panic!("variante attendue");
    };
    assert_eq!(paused, Some(true));
    assert_eq!(reason.as_deref(), Some("revue"));
    let state = DaemonToWrapper::ControlState {
        state: control_state_frame(),
        inbox_open_count: 3,
    };
    let encoded = encode(&state).unwrap();
    assert!(encoded.contains(r#""type":"control_state""#), "{encoded}");
    let DaemonToWrapper::ControlState {
        state,
        inbox_open_count,
    } = decode(&encoded).unwrap()
    else {
        panic!("variante attendue");
    };
    assert_eq!(state, control_state_frame());
    assert_eq!(inbox_open_count, 3);
    let projected = DaemonToWrapper::ControlFocus {
        focus: Some(focus.clone()),
    };
    assert!(matches!(
        decode::<DaemonToWrapper>(&encode(&projected).unwrap()).unwrap(),
        DaemonToWrapper::ControlFocus { focus: Some(decoded) } if decoded == focus
    ));
    let history = DaemonToWrapper::ControlHistory {
        events: vec![ControlEventFrame {
            at: 1,
            actor: "humain".to_string(),
            kind: "pause_on".to_string(),
            reason: None,
            generation_after: 1,
        }],
    };
    assert!(decode::<DaemonToWrapper>(&encode(&history).unwrap()).is_ok());
    assert!(
        decode::<WrapperToDaemon>(
            &encode(&WrapperToDaemon::ControlHistory {
                version: 1,
                limit: 20
            })
            .unwrap()
        )
        .is_ok()
    );
    let rejected = DaemonToWrapper::ControlStateRejected {
        reason: ControlStateRefusal::GenerationMismatch { current: 13 },
    };
    let encoded = encode(&rejected).unwrap();
    assert!(
        encoded.contains(r#""kind":"generation_mismatch""#),
        "{encoded}"
    );
}

#[test]
fn spec_087_trames_boite_humaine_font_l_aller_retour() {
    let deposit = WrapperToDaemon::HumanInboxDeposit {
        version: HUMAN_INBOX_CONTRACT_VERSION,
        dedup_key: "chain-exhausted:d1".to_string(),
        kind: HumanInboxKind::ChainExhausted,
        subject: HumanInboxSubject {
            objective_id: Some("o1".to_string()),
            delegation_id: Some("d1".to_string()),
            ..HumanInboxSubject::default()
        },
        context: r#"{"summary":"chaîne épuisée"}"#.to_string(),
        options: vec!["reassign:a1".to_string(), "cancel".to_string()],
    };
    let encoded = encode(&deposit).unwrap();
    assert!(
        encoded.contains(r#""type":"human_inbox_deposit""#),
        "{encoded}"
    );
    assert!(encoded.contains(r#""kind":"chain_exhausted""#), "{encoded}");
    let WrapperToDaemon::HumanInboxDeposit { kind, subject, .. } = decode(&encoded).unwrap() else {
        panic!("variante attendue");
    };
    assert_eq!(kind, HumanInboxKind::ChainExhausted);
    assert_eq!(subject.message_id, None);
    let item = HumanInboxItemFrame {
        id: "i1".to_string(),
        dedup_key: "chain-exhausted:d1".to_string(),
        kind: HumanInboxKind::ChainExhausted,
        subject: HumanInboxSubject::default(),
        context: "{}".to_string(),
        options: vec!["ack".to_string()],
        state: HumanInboxState::Resolved,
        producer: HumanInboxProducer::Guichet,
        created_at: 1,
        resolved_at: Some(2),
        occurrences: 2,
        decision: Some(HumanInboxDecision {
            decision_id: "dec-1".to_string(),
            choice: "ack".to_string(),
            actor: "humain".to_string(),
            at: 2,
        }),
        acked_at: None,
    };
    let frame = DaemonToWrapper::HumanInbox {
        items: vec![item.clone()],
        open_count: 0,
    };
    let DaemonToWrapper::HumanInbox { items, open_count } =
        decode(&encode(&frame).unwrap()).unwrap()
    else {
        panic!("variante attendue");
    };
    assert_eq!(items, vec![item]);
    assert_eq!(open_count, 0);
    for frame in [
        WrapperToDaemon::HumanInboxList {
            version: 1,
            state: HumanInboxListFilter::Open,
            limit: 50,
        },
        WrapperToDaemon::HumanInboxResolve {
            version: 1,
            command_id: "c".to_string(),
            item_id: "i1".to_string(),
            choice: "ack".to_string(),
        },
        WrapperToDaemon::HumanInboxDecisions {
            version: 1,
            limit: 20,
        },
        WrapperToDaemon::HumanInboxAck {
            version: 1,
            decision_id: "dec-1".to_string(),
        },
        WrapperToDaemon::HumanInboxClose {
            version: 1,
            item_id: "i1".to_string(),
            reason: "object_vanished".to_string(),
        },
    ] {
        let encoded = encode(&frame).unwrap();
        assert!(decode::<WrapperToDaemon>(&encoded).is_ok(), "{encoded}");
    }
    let batch = DaemonToWrapper::HumanInboxDecisionsBatch {
        decisions: vec![HumanInboxPendingDecision {
            decision_id: "dec-1".to_string(),
            item_id: "i1".to_string(),
            dedup_key: "k".to_string(),
            kind: HumanInboxKind::BudgetReached,
            subject: HumanInboxSubject::default(),
            choice: "raise_budget".to_string(),
            at: 3,
        }],
    };
    assert!(decode::<DaemonToWrapper>(&encode(&batch).unwrap()).is_ok());
    let rejected = DaemonToWrapper::HumanInboxRejected {
        reason: HumanInboxRefusal::ChoiceNotOffered,
    };
    assert!(encode(&rejected).unwrap().contains("choice_not_offered"));
}

#[test]
fn spec_087_kind_sql_derive_d_une_seule_source() {
    // Chaque variante de ALL rend un libellé unique, égal à son nom de fil.
    let mut seen = std::collections::BTreeSet::new();
    for kind in HumanInboxKind::ALL {
        let wire = serde_json::to_string(&kind).unwrap();
        assert_eq!(
            wire,
            format!("\"{}\"", kind.as_sql()),
            "fil ≠ SQL pour {kind:?}"
        );
        assert!(seen.insert(kind.as_sql()), "doublon {kind:?}");
        assert_eq!(HumanInboxKind::from_sql(kind.as_sql()), Some(kind));
    }
    assert_eq!(
        HumanInboxKind::sql_in_clause(),
        "IN ('intervention_required', 'chain_exhausted', 'review_verdict_pending', 'activation_approval', 'budget_reached', 'reply_debt', 'focus_waiting_agent', 'object_vanished', 'human_route_replaced')"
    );
    assert_eq!(HumanInboxKind::from_sql("inconnu"), None);
}

#[test]
fn spec_087_delegate_avec_origine_ou_focus_exige_la_version_revue() {
    let base = ServiceRequestPayload::Delegate {
        goal: "but".to_string(),
        review_target: None,
        explicit_target: None,
        required_tags: Vec::new(),
        duration: GuichetDurationClass::Normale,
        suite: ServiceSuiteDeclaration::Aucune,
        depends_on: Vec::new(),
        references: Vec::new(),
        origin: None,
        focus: None,
    };
    assert_eq!(base.required_contract_version(), SERVICE_CONTRACT_VERSION);
    let encoded = serde_json::to_string(&base).unwrap();
    assert!(
        !encoded.contains("origin") && !encoded.contains("focus"),
        "{encoded}"
    );
    let legacy: ServiceRequestPayload =
        serde_json::from_str(r#"{"goal":"but","duration":"normale","suite":{"kind":"aucune"}}"#)
            .unwrap();
    assert_eq!(legacy, base);
    let origin = Some(DelegateOrigin::Human {
        message_id: "m1".to_string(),
        observed: ObservedHumanMessageFrame {
            message_id: "m1".to_string(),
            ts: 1,
            sender: "humain".to_string(),
            target: "guichet".to_string(),
            body: "Travaille sur X".to_string(),
        },
        attestation: HumanOriginAttestationFrame {
            version: 1,
            issuer_scope: "bridget-ui".to_string(),
            canonical_request_sha256: "0".repeat(64),
            signature: "1".repeat(64),
        },
    });
    let focus = Some(DelegateFocus {
        project_id: "p1".to_string(),
        on_conflict: Some(FocusConflictPolicy::Queue),
    });
    let with_origin = ServiceRequestPayload::Delegate {
        goal: "but".to_string(),
        review_target: None,
        explicit_target: None,
        required_tags: Vec::new(),
        duration: GuichetDurationClass::Normale,
        suite: ServiceSuiteDeclaration::Aucune,
        depends_on: Vec::new(),
        references: Vec::new(),
        origin,
        focus,
    };
    assert_eq!(
        with_origin.required_contract_version(),
        REVIEW_DELEGATE_CONTRACT_VERSION
    );
    let encoded = serde_json::to_string(&with_origin).unwrap();
    assert!(encoded.contains(r#""kind":"human""#), "{encoded}");
    assert!(encoded.contains(r#""on_conflict":"queue""#), "{encoded}");
    let decoded: ServiceRequestPayload = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded, with_origin);
}
