//! T034 : attente151 réveillée par une question155 committée.
//! Module enfant du harnais155, sans mémoire de waiter ni processus.
use super::*;

#[test]
fn native155_attente_reveillee_par_une_demande() {
    let fixture = RequestFixture155::new("wait-parent155");
    let (active, epoch) = {
        let st = fixture.state.lock().unwrap();
        (st.active_waits.clone(), st.delegation_store.wait_epoch())
    };
    let worker_state = fixture.state.clone();
    let task_id = fixture.task.task_id.clone();
    let (published, received) = std::sync::mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        let DaemonToWrapper::NativeDelegationResult { result } = native_delegation::handle(
            "conn-1",
            NativeDelegationRequest::Wait {
                task_ids: vec![task_id],
                wait_timeout: Some(4),
            },
            &worker_state,
        ) else {
            panic!("résultat Wait151 attendu");
        };
        published.send(result).unwrap();
    });
    // Le compteur existant établit l'admission de l'attente AVANT l'insertion.
    let deadline = Instant::now() + Duration::from_secs(2);
    while active.load(std::sync::atomic::Ordering::Acquire) == 0 && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(active.load(std::sync::atomic::Ordering::Acquire), 1);
    assert!(
        matches!(
            received.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Empty)
        ),
        "aucun retour sans question ou tâche terminale"
    );
    let before = epoch.get();
    let request = fixture.question(340);
    assert!(fixture.receive(request.clone()).is_none());
    let row = fixture.row(&request);
    assert_eq!((&row.state[..], &row.verdict[..]), ("open", "parent"));
    assert!(
        epoch.get() != before,
        "notification WaitEpoch après l'insertion Parent"
    );
    // Lecture par une autre connexion : l'avis ne peut réveiller une attente
    // avec une projection qui n'existe pas encore dans la transaction durable.
    let committed = rusqlite::Connection::open_with_flags(
        &fixture._config.db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    assert_eq!(
        committed
            .query_row(
                "SELECT state FROM native_child_requests WHERE request_id=?1",
                [&row.request_id],
                |record| record.get::<_, String>(0)
            )
            .unwrap(),
        "open"
    );
    let result = received.recv_timeout(Duration::from_secs(2));
    worker.join().unwrap();
    let result = result.expect("une question Parent réveille Wait sans atteindre son échéance4s");
    assert_eq!(result["timed_out"], false);
    assert_eq!(
        result["ready"],
        json!([]),
        "une demande n'est pas une tâche terminale"
    );
    assert_eq!(result["pending"], json!([fixture.task.task_id]));
    let pending = result["pending_requests"]
        .as_array()
        .expect("pending_requests à la RACINE du résultat Wait");
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0]["task_id"], fixture.task.task_id);
    assert_eq!(pending[0]["request_id"], row.request_id);
    assert_eq!(pending[0]["kind"], "user_input");
    assert_eq!(pending[0]["verdict"], "parent");
    assert_eq!(pending[0]["deadline_at"], row.deadline_at);
    assert_eq!(active.load(std::sync::atomic::Ordering::Acquire), 0);

    // Une demande déjà ouverte est actionnable immédiatement. La première
    // attente ne consomme pas la demande et n'ajoute aucun curseur au magasin.
    let start = Instant::now();
    let DaemonToWrapper::NativeDelegationResult {
        result: already_open,
    } = native_delegation::handle(
        "conn-1",
        NativeDelegationRequest::Wait {
            task_ids: vec![fixture.task.task_id.clone()],
            wait_timeout: Some(4),
        },
        &fixture.state,
    )
    else {
        panic!("résultat Wait151 attendu");
    };
    assert!(
        start.elapsed() < Duration::from_secs(1),
        "demande déjà ouverte : retour immédiat"
    );
    assert_eq!(already_open["timed_out"], false);
    assert_eq!(already_open["ready"], json!([]));
    assert_eq!(already_open["pending"], json!([fixture.task.task_id]));
    assert_eq!(already_open["pending_requests"], result["pending_requests"]);
    assert_eq!(
        fixture.row(&request),
        row,
        "deux lectures sans consommation"
    );

    // ready garde les tâches terminales, pending garde les identifiants des
    // tâches non terminales, même lorsque le lot contient déjà un résultat.
    let pending_task = {
        let mut st = fixture.state.lock().unwrap();
        assert!(
            st.delegation_store
                .close_child_request(
                    &row.request_id,
                    "cancelled",
                    "cancelled_by_parent",
                    unix_timestamp()
                )
                .unwrap()
        );
        let mut terminal = fixture.task.clone();
        terminal.state = "result_available".into();
        terminal.result = Some("résultat fixture155".into());
        st.delegation_store.save(&terminal).unwrap();
        let view = handle_locked(
            "conn-1",
            delegate_request(
                "native-test",
                "wait-pending155",
                &fixture.task.cwd,
                Some(SpawnPosture::Development),
            ),
            &mut st,
        )
        .unwrap();
        view["task_id"].as_str().unwrap().to_owned()
    };
    let DaemonToWrapper::NativeDelegationResult { result: terminal } = native_delegation::handle(
        "conn-1",
        NativeDelegationRequest::Wait {
            task_ids: vec![fixture.task.task_id.clone(), pending_task.clone()],
            wait_timeout: Some(1),
        },
        &fixture.state,
    ) else {
        panic!("résultat Wait151 attendu");
    };
    assert_eq!(terminal["timed_out"], false);
    let ready = terminal["ready"].as_array().unwrap();
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0]["task_id"], fixture.task.task_id);
    assert_eq!(ready[0]["status"], "result_available");
    assert_eq!(terminal["pending"], json!([pending_task]));
    assert_eq!(terminal["pending_requests"], json!([]));

    // Human reste visible dans status mais ne satisfait pas le prédicat Wait.
    let human = RequestFixture155::new("wait-human155");
    let approval = human.approval(341);
    human.receive(approval.clone());
    let human_row = human.row(&approval);
    assert_eq!(
        (&human_row.state[..], &human_row.verdict[..]),
        ("open", "human")
    );
    let status = {
        let mut st = human.state.lock().unwrap();
        handle_locked(
            "conn-1",
            NativeDelegationRequest::Status {
                task_id: human.task.task_id.clone(),
            },
            &mut st,
        )
        .unwrap()
    };
    assert!(
        status["pending_requests"]
            .as_array()
            .unwrap()
            .iter()
            .any(|request| request["request_id"] == human_row.request_id
                && request["verdict"] == "human")
    );
    let start = Instant::now();
    let DaemonToWrapper::NativeDelegationResult { result: only_human } = native_delegation::handle(
        "conn-1",
        NativeDelegationRequest::Wait {
            task_ids: vec![human.task.task_id.clone()],
            wait_timeout: Some(1),
        },
        &human.state,
    ) else {
        panic!("résultat Wait151 attendu");
    };
    assert!(
        start.elapsed() >= Duration::from_millis(900),
        "Human seul ne rend pas la main avant l'échéance"
    );
    assert_eq!(only_human["timed_out"], true);
    assert_eq!(only_human["ready"], json!([]));
    assert_eq!(only_human["pending"], json!([human.task.task_id]));
    assert_eq!(only_human["pending_requests"], json!([]));
    assert_eq!(
        human.row(&approval),
        human_row,
        "Wait ne consomme pas la demande Human"
    );
}
