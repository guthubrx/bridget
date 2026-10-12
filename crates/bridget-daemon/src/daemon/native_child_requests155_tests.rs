//! Oracles155 T017/T019–T022. Harnais149 réutilisé ; aucun fournisseur lancé.
//! Raccord provisoire sous les tests149 jusqu'au GO du code de production.
#![cfg(test)]
use super::*;
use crate::delegation::ChildRequest;
use bridget_transport::protocol::{
    ChildProviderRequest, ChildProviderRequestId, ChildRequestClosedState, ChildRequestDecision,
    ChildRequestKind,
};

// Fixture propre aux demandes155 : trois familles utilisent la même admission
// réelle149, le bail préparé, la route et la présence effectivement inscrites.
struct RequestFixture155 {
    state: Arc<Mutex<DaemonState>>,
    task: crate::delegation::Task,
    parent_reader: BufReader<UnixStream>,
    child_reader: BufReader<UnixStream>,
    _config: DaemonConfig,
    _managed: Receiver<ManagedSupervisorCommand>,
}

impl RequestFixture155 {
    fn new(label: &str) -> Self {
        let (mut st, config, rx, parent_reader) = fixture_with_reader(label);
        let cwd = st
            .fixture_root
            .as_ref()
            .unwrap()
            .0
            .join("project-149")
            .canonicalize()
            .unwrap();
        insert_fact(
            &mut st,
            codex_fact(
                cwd.to_str().unwrap(),
                json!({"type":"dangerFullAccess"}),
                "on-request",
            ),
            None,
        );
        let mut request = delegate_request(
            "native-test",
            label,
            cwd.to_str().unwrap(),
            Some(SpawnPosture::Development),
        );
        if let NativeDelegationRequest::Delegate { child_requests, .. } = &mut request {
            *child_requests = Some("forward".into());
        }
        let view = handle_locked("conn-1", request, &mut st).unwrap();
        let mut task = st
            .delegation_store
            .get(view["task_id"].as_str().unwrap())
            .unwrap()
            .unwrap();
        spawn_task(&mut st, &mut task).unwrap();
        let ManagedSupervisorCommand::Start { prepared, .. } = rx.try_recv().unwrap() else {
            panic!("Start préparé attendu, sans processus")
        };
        let (writer, child_reader) = crate::daemon::presence_tests::control_socket("rq155");
        let host = st.host.clone();
        assert!(matches!(
            handle_register_with_channel(
                "child155",
                2,
                prepared.agent_type.clone(),
                prepared.lease.name.clone(),
                Some(host),
                Some(prepared.resolved_definition.protocol.clone()),
                ChannelReport::Known("unix".into()),
                Some(PresenceMode::Acp),
                None,
                Some("macOS".into()),
                Some(prepared.lease.instance_id.clone()),
                None,
                false,
                Some(true),
                &mut st
            ),
            DaemonToWrapper::Registered { .. }
        ));
        st.connections.insert("child155".into(), writer);
        st.fleet
            .register_connected(
                &prepared.lease,
                &prepared.lease.instance_id,
                unix_timestamp(),
            )
            .unwrap();
        st.managed_spawns
            .get_mut(&task.task_id)
            .unwrap()
            .wrapper_conn = Some("child155".into());
        task.state = "working".into();
        task.mission_deadline_at = Some(unix_timestamp() + 7200);
        st.store
            .create_request(&task.mission, &task.owner, &task.child, 7200)
            .unwrap();
        st.delegation_store.save(&task).unwrap();
        assert_eq!(
            live_connection_identity(&st, "child155"),
            Some((task.child.clone(), task.child_instance.clone().unwrap()))
        );
        Self {
            state: Arc::new(Mutex::new(st)),
            task,
            parent_reader,
            child_reader,
            _config: config,
            _managed: rx,
        }
    }

    fn question(&self, id: i64) -> ChildProviderRequest {
        ChildProviderRequest {
            version: 1,
            provider_session: "sess-child155".into(),
            turn_id: "turn155".into(),
            provider_request_id: ChildProviderRequestId::Number(id),
            kind: ChildRequestKind::UserInput,
            details: json!({"method":"item/tool/requestUserInput","duration":"once","complete":true,"secret":false,
                "effects":{"command":false,"network":false,"file_write":false},"paths_cited":[],
                "questions":[{"id":"q1","header":"Choix","question":"Quelle couleur ?","isSecret":false}],"reason":"Question fixture"}),
            offered: vec![
                ChildRequestDecision::Accept,
                ChildRequestDecision::Decline,
                ChildRequestDecision::Cancel,
            ],
            question_ids: vec!["q1".into()],
            truncated: false,
        }
    }

    fn approval(&self, id: i64) -> ChildProviderRequest {
        let mut request = self.question(id);
        request.kind = ChildRequestKind::ApprovalCommand;
        request.question_ids.clear();
        request.details = json!({"method":"item/commandExecution/requestApproval","kind":"command","duration":"once",
            "complete":true,"secret":false,"effects":{"command":true,"network":false,"file_write":false},
            "command":"printf fixture155","paths_cited":[],"reason":"Commande fixture"});
        request
    }

    fn receive(&self, request: ChildProviderRequest) -> Option<DaemonToWrapper> {
        handle_wrapper_message(
            "child155",
            WrapperToDaemon::ChildRequest { request },
            &self.state,
        )
    }

    fn row(&self, request: &ChildProviderRequest) -> ChildRequest {
        let st = self.state.lock().unwrap();
        let id:String = st.delegation_store.test_connection().query_row(
            "SELECT request_id FROM native_child_requests WHERE task_id=?1 AND provider_session=?2 AND turn_id=?3 AND provider_request_id=?4",
            rusqlite::params![self.task.task_id, request.provider_session, request.turn_id, serde_json::to_string(&request.provider_request_id).unwrap()],
            |row| row.get(0)).expect("une trame liée doit avoir une ligne persistée");
        st.delegation_store.get_child_request(&id).unwrap().unwrap()
    }

    fn decide(&self, row: &ChildRequest) {
        assert!(
            self.state
                .lock()
                .unwrap()
                .delegation_store
                .decide_child_request(
                    &row.request_id,
                    "accept",
                    if row.kind == "user_input" {
                        Some("{\"q1\":[\"bleu\"]}")
                    } else {
                        None
                    },
                    if row.verdict == "human" {
                        "human"
                    } else {
                        "parent"
                    },
                    if row.verdict == "human" {
                        "accepted_by_human"
                    } else {
                        "answered_by_parent"
                    },
                    unix_timestamp()
                )
                .unwrap()
        );
    }

    fn frames(&mut self) -> Vec<DaemonToWrapper> {
        self.child_reader
            .get_ref()
            .set_read_timeout(Some(Duration::from_millis(50)))
            .unwrap();
        let mut frames = Vec::new();
        loop {
            let mut line = String::new();
            match self.child_reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => frames.push(decode(line.trim()).unwrap()),
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                    ) =>
                {
                    break;
                }
                Err(error) => panic!("socket enfant155 : {error}"),
            }
        }
        frames
    }

    fn expired(&self, row: &ChildRequest) {
        self.state
            .lock()
            .unwrap()
            .delegation_store
            .test_connection()
            .execute(
                "UPDATE native_child_requests SET deadline_at=?2 WHERE request_id=?1",
                rusqlite::params![row.request_id, unix_timestamp() - 1],
            )
            .unwrap();
    }
}

#[test]
fn native155_request_tuple_idempotence_conflit_tour_et_identite() {
    let fixture = RequestFixture155::new("tuple155");
    let request = fixture.question(1);
    fixture.receive(request.clone());
    let first = fixture.row(&request);
    assert_eq!((&first.state[..], &first.verdict[..]), ("open", "parent"));
    fixture.receive(request.clone());
    assert_eq!(fixture.row(&request), first);
    let mut different = request.clone();
    different.details["reason"] = json!("Autre contenu");
    assert!(
        matches!(fixture.receive(different), Some(DaemonToWrapper::Nack { reason, .. }) if reason == "request_conflict")
    );
    let mut next = request.clone();
    next.turn_id = "turn156".into();
    fixture.receive(next.clone());
    assert_ne!(fixture.row(&next).request_id, first.request_id);
    let mut numeric_string = request.clone();
    numeric_string.provider_request_id = ChildProviderRequestId::String("1".into());
    fixture.receive(numeric_string.clone());
    assert_ne!(
        fixture.row(&numeric_string).request_id,
        first.request_id,
        "ID JSON typé conservé"
    );
    let reply = handle_wrapper_message(
        "conn-1",
        WrapperToDaemon::ChildRequest {
            request: request.clone(),
        },
        &fixture.state,
    );
    assert!(
        matches!(reply, Some(DaemonToWrapper::Nack { reason, .. }) if reason == "request_unlinked")
    );
    let st = fixture.state.lock().unwrap();
    assert_eq!(
        st.delegation_store
            .child_request_counts(&fixture.task.task_id)
            .unwrap(),
        (3, 3)
    );
    let mut terminal = fixture.task.clone();
    terminal.state = "failed".into();
    st.delegation_store.save(&terminal).unwrap();
    drop(st);
    assert!(
        matches!(fixture.receive(fixture.question(9)),Some(DaemonToWrapper::Nack{reason,..}) if reason == "request_unlinked"),
        "tâche terminale rejetée avant admission"
    );
}

#[test]
fn native155_request_verdicts_et_forbidden_sans_inbox() {
    for (label, expected) in [
        ("parent155", "parent"),
        ("human155", "human"),
        ("forbidden155", "forbidden"),
    ] {
        let fixture = RequestFixture155::new(label);
        let mut request = if expected == "parent" {
            fixture.question(1)
        } else {
            fixture.approval(1)
        };
        if expected == "forbidden" {
            request.details["duration"] = json!("session");
        }
        fixture.receive(request.clone());
        let row = fixture.row(&request);
        assert_eq!(row.verdict, expected);
        let st = fixture.state.lock().unwrap();
        let count = crate::human_inbox::open_count(st.store.connection()).unwrap();
        if expected == "human" {
            assert_eq!(count, 1);
            assert!(row.inbox_item_id.is_some());
        } else {
            assert_eq!(count, 0);
        }
        if expected == "forbidden" {
            assert_eq!(row.state, "refused");
            assert_eq!(row.outcome.as_deref(), Some("refused:duration_unsupported"));
        }
    }
}

#[test]
fn native155_request_budgets_ouverte_et_totale() {
    for total_limit in [false, true] {
        let mut fixture = RequestFixture155::new(if total_limit { "total155" } else { "open155" });
        if total_limit {
            // Runtime r2 : ID fournisseur 8 refusé par le disjoncteur149
            // (limite 8 échanges / 180 s), sans ledger ni trame parent.
            // Ce montage teste le budget155 de 32 lignes.
            fixture.state.lock().unwrap().circuit_breaker =
                CircuitBreaker::new(fixture._config.circuit_breaker_window, 64);
        }
        let accepted = if total_limit { 32 } else { 4 };
        for id in 0..accepted {
            let request = fixture.question(id);
            fixture.receive(request.clone());
            let row = fixture.row(&request);
            assert_eq!(row.state, "open");
            // Le parent privé consomme chaque avis ; il ne doit pas saturer
            // sa socket pendant les32 demandes du scénario de budget.
            let _ = crate::daemon::presence_tests::read_control(&mut fixture.parent_reader);
            if total_limit {
                assert!(
                    fixture
                        .state
                        .lock()
                        .unwrap()
                        .delegation_store
                        .close_child_request(
                            &row.request_id,
                            "cancelled",
                            "cancelled_by_parent",
                            unix_timestamp()
                        )
                        .unwrap()
                );
            }
        }
        let refused = fixture.question(accepted);
        let reply = fixture.receive(refused.clone());
        let mut frames = fixture.frames();
        if let Some(reply) = reply {
            frames.push(reply);
        }
        let refusal = frames
            .iter()
            .find_map(|frame| {
                if let DaemonToWrapper::ChildRequestAnswer { answer } = frame {
                    (answer.provider_request_id == refused.provider_request_id).then_some(answer)
                } else {
                    None
                }
            })
            .expect("refus wire lié au tuple exact, pas un Nack qui laisse l'attente ouverte");
        assert_eq!(refusal.outcome, "refused:request_budget_exhausted");
        assert_eq!(refusal.decision, ChildRequestDecision::Decline);
        assert_eq!(refusal.provider_session, refused.provider_session);
        assert_eq!(refusal.turn_id, refused.turn_id);
        let counts = fixture
            .state
            .lock()
            .unwrap()
            .delegation_store
            .child_request_counts(&fixture.task.task_id)
            .unwrap();
        assert!(
            counts.0 <= 4 && counts.1 <= 32,
            "plafonds persistés {counts:?}"
        );
    }
}

#[test]
fn native155_request_echeances_fenetres_et_decision_tardive() {
    for (human, window) in [(false, 600), (true, 3600)] {
        let fixture = RequestFixture155::new("window155");
        let request = if human {
            fixture.approval(1)
        } else {
            fixture.question(1)
        };
        let before = unix_timestamp();
        fixture.receive(request.clone());
        let after = unix_timestamp();
        let row = fixture.row(&request);
        assert!((before + window..=after + window).contains(&row.deadline_at));
        fixture.expired(&row);
        let response = {
            let mut st = fixture.state.lock().unwrap();
            handle_locked(
                "conn-1",
                serde_json::from_value(
                    json!({"operation":"respond","task_id":fixture.task.task_id,
                "request_id":row.request_id,"decision":"accept","answer":{"q1":["bleu"]}}),
                )
                .unwrap(),
                &mut st,
            )
        };
        assert_eq!(
            response.unwrap_err(),
            "request_expired",
            "échéance avant route"
        );
    }
    for remaining in [29, 90] {
        let fixture = RequestFixture155::new("clip155");
        let mission_deadline = unix_timestamp() + remaining;
        {
            let st = fixture.state.lock().unwrap();
            let mut task = fixture.task.clone();
            task.mission_deadline_at = Some(mission_deadline);
            st.delegation_store.save(&task).unwrap();
        }
        let request = fixture.question(1);
        fixture.receive(request.clone());
        let row = fixture.row(&request);
        if remaining == 29 {
            assert_eq!(row.outcome.as_deref(), Some("refused:deadline_too_close"));
        } else {
            assert_eq!(row.deadline_at, mission_deadline - 30);
        }
    }
}

#[test]
fn native155_request_avis_idempotent() {
    let mut fixture = RequestFixture155::new("notice155");
    let request = fixture.question(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    fixture.receive(request.clone());
    native_delegation::tick(&fixture.state);
    let st = fixture.state.lock().unwrap();
    let messages = st.store.recent_messages(100).unwrap();
    let expected = format!("native-request-{}", row.request_id);
    let notices = messages
        .iter()
        .filter(|entry| entry.id == expected)
        .collect::<Vec<_>>();
    assert_eq!(notices.len(), 1);
    assert_eq!(notices[0].target, fixture.task.owner);
    assert!(notices[0].body.contains(&row.request_id));
    drop(st);
    fixture
        .parent_reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(50)))
        .unwrap();
    let mut system_notice = false;
    loop {
        let mut line = String::new();
        match fixture.parent_reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let frame: DaemonToWrapper = decode(line.trim()).unwrap();
                let message = match frame {
                    DaemonToWrapper::Deliver(message)
                    | DaemonToWrapper::DeliverExecution { message, .. }
                    | DaemonToWrapper::DeliverIdempotent { message, .. } => Some(message),
                    _ => None,
                };
                if let Some(message) = message.filter(|message| message.id == expected) {
                    assert_eq!(message.origin, Some(bridget_core::MessageOrigin::System));
                    system_notice = true;
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) =>
            {
                break;
            }
            Err(error) => panic!("avis parent155 : {error}"),
        }
    }
    assert!(system_notice, "avis système réellement remis au parent");
}

#[test]
fn native155_human_depot_idempotent_apres_panne() {
    let fixture = RequestFixture155::new("deposit155");
    let request = fixture.approval(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    let item = row.inbox_item_id.unwrap();
    {
        let st = fixture.state.lock().unwrap();
        st.delegation_store
            .test_connection()
            .execute(
                "UPDATE native_child_requests SET inbox_item_id=NULL WHERE request_id=?1",
                [&row.request_id],
            )
            .unwrap();
    }
    fixture.receive(request.clone());
    native_delegation::tick(&fixture.state);
    assert_eq!(
        fixture.row(&request).inbox_item_id.as_deref(),
        Some(item.as_str())
    );
    let st = fixture.state.lock().unwrap();
    assert_eq!(
        crate::human_inbox::open_count(st.store.connection()).unwrap(),
        1
    );
    assert_eq!(
        crate::human_inbox::get(st.store.connection(), &item)
            .unwrap()
            .unwrap()
            .occurrences,
        1,
        "pas de second dépôt pour réparer le lien"
    );
}

#[test]
fn native155_human_item_deja_resolu_reutilise() {
    let fixture = RequestFixture155::new("resolved155");
    let request = fixture.approval(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    let item = row.inbox_item_id.unwrap();
    {
        let st = fixture.state.lock().unwrap();
        crate::human_inbox::resolve(
            st.store.connection(),
            &item,
            "decline",
            "humain",
            unix_timestamp(),
        )
        .unwrap()
        .unwrap();
        st.delegation_store
            .test_connection()
            .execute(
                "UPDATE native_child_requests SET inbox_item_id=NULL WHERE request_id=?1",
                [&row.request_id],
            )
            .unwrap();
    }
    fixture.receive(request.clone());
    assert_eq!(
        fixture.row(&request).inbox_item_id.as_deref(),
        Some(item.as_str())
    );
    let st = fixture.state.lock().unwrap();
    assert_eq!(
        crate::human_inbox::open_count(st.store.connection()).unwrap(),
        0
    );
    let count: i64 = st
        .store
        .connection()
        .query_row("SELECT count(*) FROM human_inbox", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn native155_lifecycle_renvoi_jusqu_a_accuse() {
    let mut fixture = RequestFixture155::new("resend155");
    let request = fixture.question(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    fixture.decide(&row);
    // Bloquer la socket avant le tick : la lecture indépendante observe le
    // commit SENT alors qu'aucune écriture au wrapper n'est encore possible.
    let writer = fixture.state.lock().unwrap().connections["child155"].clone();
    let gate = writer.lock().unwrap();
    let state = fixture.state.clone();
    let blocked_since = Instant::now();
    let worker = std::thread::spawn(move || native_delegation::tick(&state));
    let reader = rusqlite::Connection::open(&fixture._config.db_path).unwrap();
    reader.busy_timeout(Duration::from_millis(50)).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut sent_before_write = false;
    while Instant::now() < deadline {
        let persisted = reader.query_row(
            "SELECT delivery_state,sent_at FROM native_child_requests WHERE request_id=?1",
            [&row.request_id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<i64>>(1)?)),
        );
        if matches!(persisted,Ok((state,Some(_))) if state == "sent") {
            sent_before_write = true;
            break;
        }
        if worker.is_finished() {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let sent_before_timeout = fixture.row(&request);
    while !worker.is_finished() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    if !worker.is_finished() {
        drop(gate);
        worker.join().unwrap();
        panic!("le tick doit rendre la main sous2s malgré le writer verrouillé");
    }
    worker.join().unwrap();
    assert!(
        sent_before_write,
        "SENT committé avant write ; lecture bornée2s"
    );
    assert!(
        blocked_since.elapsed() >= Duration::from_secs(1),
        "le verrou couvre l'expiration du budget d'écriture1s"
    );
    assert_eq!(
        fixture.row(&request),
        sent_before_timeout,
        "sans première écriture, SENT reste inchangé et réessayable"
    );
    assert_eq!(sent_before_timeout.delivery_state, "sent");
    assert!(sent_before_timeout.sent_at.is_some());
    assert!(sent_before_timeout.void_reason.is_none());
    assert!(
        fixture.frames().is_empty(),
        "aucune trame enfant avant libération du writer"
    );
    drop(gate);
    native_delegation::tick(&fixture.state);
    let first = fixture
        .frames()
        .into_iter()
        .filter_map(|frame| {
            if let DaemonToWrapper::ChildRequestAnswer { answer } = frame {
                Some(answer)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(first.len(), 1);
    assert_eq!(fixture.row(&request).delivery_state, "sent");
    assert!(fixture.row(&request).sent_at.is_some());
    native_delegation::tick(&fixture.state);
    let second = fixture
        .frames()
        .into_iter()
        .filter_map(|frame| {
            if let DaemonToWrapper::ChildRequestAnswer { answer } = frame {
                Some(answer)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(second, first);
    handle_wrapper_message(
        "conn-1",
        WrapperToDaemon::ChildRequestAnswered {
            request_id: row.request_id.clone(),
            applied: true,
            reason: None,
        },
        &fixture.state,
    );
    assert_eq!(
        fixture.row(&request).delivery_state,
        "sent",
        "autre connexion ignorée"
    );
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
    native_delegation::tick(&fixture.state);
    assert!(
        !fixture
            .frames()
            .iter()
            .any(|frame| matches!(frame, DaemonToWrapper::ChildRequestAnswer { .. }))
    );
    fixture.receive(request.clone());
    assert_eq!(fixture.row(&request).delivery_state, "delivered");
    assert_eq!(
        fixture
            .state
            .lock()
            .unwrap()
            .delegation_store
            .child_request_counts(&fixture.task.task_id)
            .unwrap()
            .1,
        1,
        "replay terminal sans nouvelle ligne"
    );
}

#[test]
fn native155_lifecycle_accuse_negatif_annule() {
    let mut fixture = RequestFixture155::new("negative155");
    let request = fixture.question(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    fixture.decide(&row);
    native_delegation::tick(&fixture.state);
    fixture.frames();
    handle_wrapper_message(
        "child155",
        WrapperToDaemon::ChildRequestAnswered {
            request_id: row.request_id.clone(),
            applied: false,
            reason: Some("closed".into()),
        },
        &fixture.state,
    );
    assert_eq!(fixture.row(&request).delivery_state, "void");
    native_delegation::tick(&fixture.state);
    assert!(
        !fixture
            .frames()
            .iter()
            .any(|f| matches!(f, DaemonToWrapper::ChildRequestAnswer { .. }))
    );
}

#[test]
fn native155_lifecycle_enfant_disparu_accord_sent_reste_unknown() {
    let mut fixture = RequestFixture155::new("lost155");
    let request = fixture.approval(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    fixture.decide(&row);
    native_delegation::tick(&fixture.state);
    fixture.frames();
    fixture.state.lock().unwrap().connections.remove("child155");
    native_delegation::tick(&fixture.state);
    let persisted = fixture.row(&request);
    assert_eq!(persisted.delivery_state, "unknown");
    assert_eq!(
        persisted.outcome.as_deref(),
        Some("cancelled_outcome_unknown")
    );
    assert_eq!(persisted.decision.as_deref(), Some("accept"));
}

#[test]
fn native155_lifecycle_plantage_entre_sent_et_envoi() {
    let fixture = RequestFixture155::new("crash155");
    let request = fixture.approval(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    fixture.decide(&row);
    {
        let st = fixture.state.lock().unwrap();
        assert!(
            st.delegation_store
                .claim_child_request(&row.request_id, unix_timestamp())
                .unwrap()
        );
        native_delegation::prepare_restart(&st).unwrap();
    }
    let persisted = fixture.row(&request);
    assert_eq!(persisted.delivery_state, "unknown");
    assert_eq!(
        persisted.outcome.as_deref(),
        Some("cancelled_outcome_unknown")
    );
}

#[test]
fn native155_lifecycle_accord_apres_echeance_jamais_envoye() {
    for sent in [false, true] {
        let mut fixture = RequestFixture155::new("late155");
        let request = fixture.approval(1);
        fixture.receive(request.clone());
        let row = fixture.row(&request);
        fixture.decide(&row);
        if sent {
            native_delegation::tick(&fixture.state);
            fixture.frames();
        }
        fixture.expired(&row);
        native_delegation::tick(&fixture.state);
        let frames = fixture.frames();
        assert!(!frames.iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestAnswer{answer} if answer.decision == ChildRequestDecision::Accept)));
        assert!(frames.iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestCancel{request_id,..} if request_id == &row.request_id)),"fermeture requise même pending invalidé");
        if !sent {
            assert_eq!(fixture.row(&request).delivery_state, "void");
        }
        handle_wrapper_message(
            "child155",
            WrapperToDaemon::ChildRequestClosed {
                request_id: row.request_id.clone(),
                state: ChildRequestClosedState::ClosedBeforeApply,
            },
            &fixture.state,
        );
        assert_eq!(fixture.row(&request).delivery_state, "void");
    }
}

#[test]
fn native155_lifecycle_expiration_refus_et_cloture_none_pending_void() {
    for delivery in ["none", "pending", "void"] {
        for approval in [false, true] {
            let mut fixture = RequestFixture155::new("expire155");
            let request = if approval {
                fixture.approval(1)
            } else {
                fixture.question(1)
            };
            fixture.receive(request.clone());
            let row = fixture.row(&request);
            if delivery != "none" {
                fixture.decide(&row);
            }
            if delivery == "void" {
                fixture
                    .state
                    .lock()
                    .unwrap()
                    .delegation_store
                    .void_pending_child_request(&row.request_id, "cancelled_by_parent")
                    .unwrap();
            }
            fixture.expired(&row);
            native_delegation::tick(&fixture.state);
            let frames = fixture.frames();
            assert!(!frames.iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestAnswer{answer} if answer.decision == ChildRequestDecision::Accept)));
            assert!(frames.iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestCancel{request_id,..} if request_id == &row.request_id)));
            native_delegation::tick(&fixture.state);
            assert!(fixture.frames().iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestCancel{request_id,..} if request_id == &row.request_id)),"Cancel répété tant que Closed absent");
            handle_wrapper_message(
                "child155",
                WrapperToDaemon::ChildRequestClosed {
                    request_id: row.request_id.clone(),
                    state: ChildRequestClosedState::ClosedBeforeApply,
                },
                &fixture.state,
            );
            native_delegation::tick(&fixture.state);
            assert!(!fixture.frames().iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestAnswer{answer} if answer.decision == ChildRequestDecision::Accept)));
        }
    }
}

#[test]
fn native155_lifecycle_invalidations_avant_renvoi_et_finalisation() {
    for cause in ["cancel", "end", "pause", "revoke", "instance", "restart"] {
        for sent in [false, true] {
            let mut fixture = RequestFixture155::new("close155");
            let request = fixture.approval(1);
            fixture.receive(request.clone());
            let row = fixture.row(&request);
            fixture.decide(&row);
            if sent {
                native_delegation::tick(&fixture.state);
                fixture.frames();
            }
            {
                let st = fixture.state.lock().unwrap();
                let mut task = st
                    .delegation_store
                    .get(&fixture.task.task_id)
                    .unwrap()
                    .unwrap();
                match cause {
                    "cancel" => {
                        task.state = "cancelling".into();
                        st.delegation_store.save(&task).unwrap();
                    }
                    "end" => {
                        task.state = "result_available".into();
                        st.delegation_store.save(&task).unwrap();
                    }
                    "pause" => {
                        let control = crate::referent_control::read(st.store.connection()).unwrap();
                        crate::referent_control::set(
                            st.store.connection(),
                            crate::referent_control::ControlMutation {
                                command_id: "pause155",
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
                    "revoke" => {
                        st.delegation_store
                            .grant_agent(
                                &task.owner,
                                &task.owner_instance,
                                Path::new(&task.cwd),
                                SpawnPosture::Development,
                                true,
                            )
                            .unwrap();
                    }
                    "instance" => {
                        task.child_instance = Some("other155".into());
                        st.delegation_store.save(&task).unwrap();
                    }
                    "restart" => {
                        native_delegation::prepare_restart(&st).unwrap();
                    }
                    _ => unreachable!(),
                }
            }
            native_delegation::tick(&fixture.state);
            let frames = fixture.frames();
            assert!(!frames.iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestAnswer{answer} if answer.decision == ChildRequestDecision::Accept)),"{cause}/{sent}");
            let persisted = fixture.row(&request);
            if cause == "instance" && sent {
                assert_eq!(
                    persisted.delivery_state, "sent",
                    "ancienne connexion encore vivante : attente de Closed"
                );
                assert!(frames.iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestCancel{request_id,..} if request_id == &row.request_id)));
                handle_wrapper_message(
                    "child155",
                    WrapperToDaemon::ChildRequestClosed {
                        request_id: row.request_id.clone(),
                        state: ChildRequestClosedState::ClosedBeforeApply,
                    },
                    &fixture.state,
                );
                assert_eq!(fixture.row(&request).delivery_state, "void");
            } else if cause == "restart" || cause == "instance" {
                assert!(
                    matches!(persisted.delivery_state.as_str(), "void" | "unknown"),
                    "{cause}/{sent}"
                );
            } else {
                assert!(frames.iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestCancel{request_id,..} if request_id == &row.request_id)),"{cause}/{sent} attend Closed");
                if cause == "cancel" {
                    let st = fixture.state.lock().unwrap();
                    let task = st
                        .delegation_store
                        .get(&fixture.task.task_id)
                        .unwrap()
                        .unwrap();
                    assert_ne!(task.state, "cancelled", "aucun état final sans fermeture");
                    assert!(!task.cleanup_done);
                }
            }
        }
    }
}

#[test]
fn native155_lifecycle_annulation_course_closed_avant_ou_apres_application() {
    for already_applied in [false, true] {
        let mut fixture = RequestFixture155::new("race155");
        let request = fixture.approval(1);
        fixture.receive(request.clone());
        let row = fixture.row(&request);
        fixture.decide(&row);
        native_delegation::tick(&fixture.state);
        fixture.frames();
        {
            let st = fixture.state.lock().unwrap();
            let mut task = fixture.task.clone();
            task.state = "cancelling".into();
            st.delegation_store.save(&task).unwrap();
        }
        native_delegation::tick(&fixture.state);
        assert!(
            fixture
                .frames()
                .iter()
                .any(|f| matches!(f, DaemonToWrapper::ChildRequestCancel { .. }))
        );
        handle_wrapper_message(
            "child155",
            WrapperToDaemon::ChildRequestClosed {
                request_id: row.request_id.clone(),
                state: if already_applied {
                    ChildRequestClosedState::AlreadyApplied
                } else {
                    ChildRequestClosedState::ClosedBeforeApply
                },
            },
            &fixture.state,
        );
        assert_eq!(
            fixture.row(&request).delivery_state,
            if already_applied { "delivered" } else { "void" }
        );
        fixture.receive(request.clone());
        native_delegation::tick(&fixture.state);
        assert!(!fixture.frames().iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestAnswer{answer} if answer.decision == ChildRequestDecision::Accept)));
    }
}

#[test]
fn native155_lifecycle_accord_pending_annulation_puis_reconnexion_void() {
    let mut fixture = RequestFixture155::new("reconnect155");
    let request = fixture.approval(1);
    fixture.receive(request.clone());
    let row = fixture.row(&request);
    fixture.decide(&row);
    {
        let st = fixture.state.lock().unwrap();
        let mut task = fixture.task.clone();
        task.state = "cancelling".into();
        st.delegation_store.save(&task).unwrap();
    }
    native_delegation::tick(&fixture.state);
    assert_eq!(fixture.row(&request).delivery_state, "void");
    assert!(fixture.frames().iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestCancel{request_id,..} if request_id == &row.request_id)));
    {
        let mut st = fixture.state.lock().unwrap();
        st.connections.remove("child155");
        let (writer, reader) = crate::daemon::presence_tests::control_socket("re155");
        let host = st.host.clone();
        let agent_type = st.presences[fixture.task.child_instance.as_deref().unwrap()]
            .agent_type
            .clone();
        // Résultat du montage précédent consigné AVANT désinscription : le
        // retrait du writer seul conserve une route déjà prise (Nack register).
        let original = handle_register_with_channel(
            "child155",
            2,
            "native-test".into(),
            fixture.task.child.clone(),
            Some(host),
            Some("codex_app_server".into()),
            ChannelReport::Known("unix".into()),
            Some(PresenceMode::Acp),
            None,
            Some("macOS".into()),
            fixture.task.child_instance.clone(),
            None,
            false,
            Some(true),
            &mut st,
        );
        eprintln!("diagnostic reconnexion155 avant désinscription route : {original:?}");
        st.router.unregister_by_conn("child155");
        let host = st.host.clone();
        let registered = handle_register_with_channel(
            "child155",
            2,
            agent_type,
            fixture.task.child.clone(),
            Some(host),
            Some("codex_app_server".into()),
            ChannelReport::Known("unix".into()),
            Some(PresenceMode::Acp),
            None,
            Some("macOS".into()),
            fixture.task.child_instance.clone(),
            None,
            false,
            Some(true),
            &mut st,
        );
        assert!(
            matches!(registered, DaemonToWrapper::Registered { .. }),
            "{registered:?}"
        );
        st.connections.insert("child155".into(), writer);
        fixture.child_reader = reader;
    }
    fixture.receive(request.clone());
    native_delegation::tick(&fixture.state);
    assert!(!fixture.frames().iter().any(|f|matches!(f,DaemonToWrapper::ChildRequestAnswer{answer} if answer.decision == ChildRequestDecision::Accept)));
    handle_wrapper_message(
        "child155",
        WrapperToDaemon::ChildRequestClosed {
            request_id: row.request_id.clone(),
            state: ChildRequestClosedState::ClosedBeforeApply,
        },
        &fixture.state,
    );
    assert_eq!(fixture.row(&request).delivery_state, "void");
}

#[path = "native_delegation_respond155_tests.rs"]
mod respond155;

#[path = "native_delegation_chain155_tests.rs"]
mod chain155;

#[path = "native_child_requests_human155_tests.rs"]
mod human155;

#[path = "native_delegation_wait155_tests.rs"]
mod wait155;
