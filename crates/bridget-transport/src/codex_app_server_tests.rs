use super::*;
use crate::poison::RecoverPoison;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::Instant;

static FIXTURE_SEQ: AtomicU64 = AtomicU64::new(0);

#[test]
fn spec172_codex_writer_lock_has_actionable_private_projection() {
    let id = "11111111-1111-4111-8111-111111111111";
    let error = json!({
        "code": -32600,
        "message": format!("thread {id} already has an active writer"),
        "data": {"private": "secret-that-must-stay-private"}
    });
    let reason = provider_error_reason(&error);
    assert!(reason.starts_with("codex_thread_already_active:"));
    assert!(reason.contains("Codex conserve le droit d’écriture"));
    assert!(reason.contains("si elle est encore ouverte"));
    assert!(reason.contains("Si le refus persiste"));
    let mut non_pilotable = error.clone();
    non_pilotable["data"]["codexErrorInfo"] = json!({"activeTurnNotSteerable": true});
    assert!(provider_error_reason(&non_pilotable).contains("; tour non pilotable"));
    assert!(reason.contains(&provider_fingerprint(
        b"error",
        error.to_string().as_bytes()
    )));
    assert!(!reason.contains(id));
    assert!(!reason.contains("secret-that-must-stay-private"));
}

#[test]
fn spec172_codex_writer_lock_does_not_classify_unknown_errors() {
    let message = "thread 11111111-1111-4111-8111-111111111111 already has an active writer";
    for (code, message) in [
        (-32601, message.to_string()),
        (
            -32600,
            "thread private-secret already has an active writer".to_string(),
        ),
        (-32600, format!("{message}; private-secret")),
        (-32600, "other private-secret".to_string()),
    ] {
        let error = json!({"code":code, "message":message});
        let reason = provider_error_reason(&error);
        assert!(reason.starts_with("erreur Codex (code "));
        assert!(!reason.contains("private-secret"));
    }
    assert_eq!(
        provider_error_reason(&json!({"code":-32001,"message":message})),
        CODEX_SATURATED_REASON
    );
}

#[test]
fn spec091_catalogue_muet_expire_sans_accumuler_de_reponses_en_attente() {
    let writer: Writer = Arc::new(Mutex::new(Some(Box::new(Vec::<u8>::new()))));
    let waiters: Waiters = Arc::new(Mutex::new(HashMap::new()));
    let next_id = Arc::new(AtomicU64::new(1));
    let start = Instant::now();
    for _ in 0..3 {
        assert!(
            request_with_timeout(
                &writer,
                &waiters,
                &next_id,
                "model/list",
                json!({"limit":100}),
                Duration::from_millis(1)
            )
            .is_err()
        );
        assert!(waiters.lock().unwrap().is_empty());
    }
    assert!(start.elapsed() < Duration::from_secs(1));
    assert_eq!(next_id.load(AtomicOrdering::SeqCst), 4);
}

#[test]
fn catalogue_reprise_pagine_sans_ignorer_la_borne_ni_un_curseur_invalide() {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut count = 0;
    let result = collect_thread_pages(deadline, |params, remaining| {
            assert!(remaining <= Duration::from_secs(10));
            assert_eq!(params["limit"], 100);
            assert_eq!(params["cursor"], if count == 0 { Value::Null } else { json!(count.to_string()) });
            count += 1;
            Ok(json!({"data":[{"id":uuid::Uuid::new_v4().to_string(),"name":"N","cwd":"/tmp"}],"nextCursor": if count == 2 { Value::Null } else { json!(count.to_string()) }}))
        }).unwrap();
    assert_eq!(result.len(), 2);
    count = 0;
    // Mutation : retirer la borne permettrait une onzième page. Le
    // catalogue incomplet n'est jamais livré au sélecteur comme complet.
    assert!(collect_thread_pages(deadline, |_, _| {
            count += 1;
            assert!(count <= 10);
            Ok(json!({"data":(0..100).map(|_| json!({"id":uuid::Uuid::new_v4().to_string(),"name":"N","cwd":"/tmp"})).collect::<Vec<_>>(),"nextCursor":count.to_string()}))
        }).is_err());
    assert_eq!(count, 10);
    for invalid in [
        json!({"data":[]}),
        json!({"data":[],"nextCursor":1}),
        json!({"data":[],"nextCursor":"same"}),
    ] {
        assert!(collect_thread_pages(deadline, |_, _| Ok(invalid.clone())).is_err());
    }
    assert!(collect_thread_pages(Instant::now(), |_, _| panic!("RPC après échéance")).is_err());
}

#[test]
fn spec090_catalogue_interactif_utilise_state_db_seul_a_chaque_page() {
    let deadline = Instant::now() + Duration::from_secs(10);
    let expected_ids = [uuid::Uuid::new_v4(), uuid::Uuid::new_v4()];
    let mut page = 0;

    let threads = collect_thread_pages(deadline, |params, remaining| {
        assert!(!remaining.is_zero());
        assert_eq!(
            params.get("useStateDbOnly"),
            Some(&Value::Bool(true)),
            "sans la projection state DB, Codex rescane et répare les rollouts avant de répondre"
        );
        assert_eq!(
            params.get("searchTerm"),
            None,
            "le menu et la détection des noms ambigus exigent toujours le catalogue complet"
        );
        assert_eq!(
            params["cursor"],
            if page == 0 {
                Value::Null
            } else {
                json!("page-2")
            }
        );
        let response = json!({
            "data": [{
                "id": expected_ids[page].to_string(),
                "name": "horizon-calliope",
                "cwd": "/Users/moi/Nextcloud/10.Scripts/63.studio-horizon"
            }],
            "nextCursor": if page == 0 { json!("page-2") } else { Value::Null }
        });
        page += 1;
        Ok(response)
    })
    .expect("le catalogue public complet doit rester disponible sous la borne existante");

    assert_eq!(page, 2, "le drapeau doit accompagner toutes les pages");
    assert_eq!(
        threads.len(),
        2,
        "les doublons de nom restent visibles au sélecteur"
    );
    assert_eq!(threads[0].name.as_deref(), Some("horizon-calliope"));
    assert_eq!(threads[1].name.as_deref(), Some("horizon-calliope"));
}

#[test]
fn lecteur_interactif_ne_repond_jamais_pour_humain_et_conserve_raw() {
    struct Capture(Arc<Mutex<Vec<u8>>>);
    impl Write for Capture {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let output = Arc::new(Mutex::new(Vec::new()));
    let observations = Arc::new((Mutex::new(Observations::default()), Condvar::new()));
    let queue = Arc::new((
        Mutex::new(QueueState {
            messages: VecDeque::new(),
            steer: VecDeque::new(),
            active: None,
            steering_open: false,
            closed: false,
            external_turn: None,
        }),
        Condvar::new(),
    ));
    let alive = Arc::new(AtomicBool::new(true));
    let detail = Arc::new(Mutex::new(None));
    let pending_request = Arc::new(Mutex::new(None));
    let journal_root = root("interactive-reader");
    let journal = Arc::new(Mutex::new(Some(
        JournalWriter::start(
            &journal_root,
            "interactive",
            "thread",
            Arc::new(Mutex::new(crate::acp::AcpEventQueue::default())),
        )
        .unwrap(),
    )));
    let (tx, rx) = mpsc::channel::<std::io::Result<String>>();
    let waiters = Arc::new(Mutex::new(HashMap::new()));
    let reader = spawn_reader(
        Box::new(rx.into_iter()),
        ReaderContext {
            waiters: waiters.clone(),
            cwd: "/tmp".into(),
            observations: observations.clone(),
            alive: alive.clone(),
            journal: journal.clone(),
            queue: queue.clone(),
            pending_request: pending_request.clone(),
            child_requests: Arc::new(Mutex::new(Vec::new())),
            forward_child_requests: false,
            provider_session: "codex-test".into(),
            pinned_model: None,
            active_detail: detail.clone(),
            writer: Arc::new(Mutex::new(Some(Box::new(Capture(output.clone()))))),
            permissions: "allow".into(),
            dynamic_tool_handler: None,
            sandbox_posture: "complete",
            interactive: true,
            selected_thread: Arc::new(Mutex::new(Some("thread".into()))),
            permission_observation: Arc::new(Mutex::new(None)),
        },
    );
    let serial = AtomicU64::new(0);
    let barrier = |raw: &str| {
        tx.send(Ok(raw.into())).unwrap();
        let sentinel = json!({"method": "test/barrier", "params": {"n": serial.fetch_add(1, AtomicOrdering::SeqCst)}}).to_string();
        tx.send(Ok(sentinel.clone())).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut state = observations.0.lock().unwrap();
        while !state
            .events
            .iter()
            .any(|event| event.raw == sentinel.as_bytes())
        {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .expect("événement reader attendu");
            state = observations.1.wait_timeout(state, remaining).unwrap().0;
        }
    };
    barrier(
        r#"{ "method":"turn/started", "params":{"threadId":"thread","turn":{"id":"t","status":"inProgress"}} }"#,
    );
    assert_eq!(queue.0.lock().unwrap().external_turn.as_deref(), Some("t"));
    // SPEC100 : le vrai lecteur ne publie qu'une réussite structurée,
    // ignore échec, doublon et ancien tour (aucun processus fournisseur).
    for (turn, id, status) in [
        ("old", "late", "completed"),
        ("t", "failed", "failed"),
        ("t", "write", "completed"),
        ("t", "write", "completed"),
    ] {
        barrier(&json!({"method":"item/completed","params":{"threadId":"thread","turnId":turn,
                "item":{"type":"fileChange","id":id,"status":status,"changes":[{"path":"/project/spec100.rs"}]}}}).to_string());
    }
    for raw in [
        r#"{ "id":7, "method":"item/commandExecution/requestApproval", "params":{"threadId":"thread","turnId":"t","future":42} }"#,
        r#"{ "id":"eight", "method":"mcpServer/elicitation/request", "params":{"serverName":"bridget","mode":"form","requestedSchema":{}} }"#,
        r#"{  "id":9, "method":"future/request", "params": {"unknown": true}  }"#,
    ] {
        barrier(raw);
    }
    // Mutation : appeler les réponses automatiques managed (allow), y
    // compris celle de refus inconnu, rendrait cette assertion rouge.
    assert!(output.lock().unwrap().is_empty());
    barrier(
        r#"{"method":"turn/completed","params":{"threadId":"thread","turn":{"id":"t","status":"completed"}}}"#,
    );
    assert!(queue.0.lock().unwrap().external_turn.is_none());
    let state = observations.0.lock().unwrap();
    assert!(state.events.iter().any(|e| matches!(
        e.kind,
        ManagedEventKind::ActivityObserved { in_progress: true }
    )));
    assert!(state.events.iter().any(|e| matches!(
        e.kind,
        ManagedEventKind::ActivityObserved { in_progress: false }
    )));
    assert!(
        !state
            .events
            .iter()
            .any(|e| matches!(e.kind, ManagedEventKind::PromptDispatched { .. }))
    );
    assert!(state.terminal_by_turn.is_empty());
    drop(state);

    // Mutation : tester queue.active.is_none() au lieu du tour observé
    // perd le tour humain arrivé pendant une réservation Bridget.
    let (cancel, _receiver) = mpsc::channel();
    queue.0.lock().unwrap().active = Some(ActiveTurn {
        message_id: "pending".into(),
        cancel,
    });
    barrier(
        r#"{"method":"turn/started","params":{"threadId":"thread","turn":{"id":"human-race","status":"inProgress"}}}"#,
    );
    barrier(
        r#"{"method":"item/started","params":{"threadId":"thread","turnId":"human-race","item":{"type":"userMessage","clientId":"human","content":[]}}}"#,
    );
    assert_eq!(
        queue.0.lock().unwrap().external_turn.as_deref(),
        Some("human-race")
    );
    assert_eq!(detail.lock().unwrap().as_ref().unwrap().message_id, "");
    barrier(
        r#"{"id":10,"method":"item/commandExecution/requestApproval","params":{"threadId":"thread","turnId":"human-race"}}"#,
    );
    assert!(
        pending_request
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .message_id
            .is_none(),
        "la réservation n'est pas l'auteur de la permission humaine"
    );
    assert!(!source_matches_active_turn(
        detail.lock().unwrap().as_ref().unwrap(),
        &json!({"params":{"threadId":"thread","turnId":"different"}})
    ));
    barrier(
        r#"{"method":"turn/completed","params":{"threadId":"thread","turn":{"id":"human-race","status":"completed"}}}"#,
    );
    barrier(
        r#"{"method":"turn/started","params":{"threadId":"thread","turn":{"id":"owned","status":"inProgress"}}}"#,
    );
    barrier(
        r#"{"method":"item/started","params":{"threadId":"thread","turnId":"owned","item":{"type":"userMessage","clientId":"pending","content":[]}}}"#,
    );
    assert_eq!(
        detail.lock().unwrap().as_ref().unwrap().message_id,
        "pending"
    );
    assert!(
        !observations
            .0
            .lock()
            .unwrap()
            .events
            .iter()
            .any(|e| matches!(e.kind, ManagedEventKind::PromptDispatched { .. })),
        "pas d'ACK avant consommation"
    );
    barrier(
        r#"{"method":"item/completed","params":{"threadId":"thread","turnId":"ancien","item":{"type":"userMessage","clientId":"pending","content":[]}}}"#,
    );
    assert!(
        !observations
            .0
            .lock()
            .unwrap()
            .events
            .iter()
            .any(|e| matches!(e.kind, ManagedEventKind::PromptDispatched { .. })),
        "ancien tour : aucun ACK malgré le bon clientId"
    );
    barrier(
        r#"{"method":"item/completed","params":{"threadId":"thread","turnId":"owned","item":{"type":"userMessage","clientId":"pending","content":[]}}}"#,
    );
    barrier(
        r#"{"method":"item/completed","params":{"threadId":"thread","turnId":"owned","item":{"type":"userMessage","clientId":"pending","content":[]}}}"#,
    );
    assert_eq!(observations.0.lock().unwrap().events.iter().filter(|e| matches!(&e.kind, ManagedEventKind::PromptDispatched { message_id } if message_id == "pending")).count(), 1);
    barrier(
        r#"{"method":"item/started","params":{"threadId":"thread","turnId":"owned","item":{"type":"userMessage","clientId":"human-steer","content":[{"type":"text","text":"AJOUT HUMAIN"}]}}}"#,
    );
    assert_eq!(
        detail.lock().unwrap().as_ref().unwrap().message_id,
        "pending",
        "l'ajout humain ne vole pas la corrélation du tour"
    );
    barrier(
        r#"{"method":"turn/completed","params":{"threadId":"thread","turn":{"id":"owned","status":"completed"}}}"#,
    );
    assert!(
        observations
            .0
            .lock()
            .unwrap()
            .terminal_by_turn
            .contains_key("owned"),
        "le worker doit recevoir son terminal malgré l'ajout humain"
    );
    // Le worker n'a PAS encore libéré sa réservation. Le lecteur doit
    // pourtant attester le tour suivant et clear_active ne peut l'effacer.
    barrier(
        r#"{"method":"turn/started","params":{"threadId":"thread","turn":{"id":"next-human","status":"inProgress"}}}"#,
    );
    clear_active(&queue);
    assert_eq!(
        queue.0.lock().unwrap().external_turn.as_deref(),
        Some("next-human")
    );
    assert_eq!(
        detail.lock().unwrap().as_ref().unwrap().turn_id.as_deref(),
        Some("next-human")
    );
    barrier(
        r#"{"method":"turn/completed","params":{"threadId":"thread","turn":{"id":"ancien","status":"completed"}}}"#,
    );
    assert!(
        !observations
            .0
            .lock()
            .unwrap()
            .terminal_by_turn
            .contains_key("ancien")
    );
    assert_eq!(
        queue.0.lock().unwrap().external_turn.as_deref(),
        Some("next-human")
    );
    let (result_tx, result_rx) = mpsc::channel();
    waiters.lock().unwrap().insert(999, result_tx);
    tx.send(Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "Space limit exceeded: message too long",
    )))
    .unwrap();
    // Une erreur de lecture ne doit plus se transformer en un faux EOF.
    let error = result_rx
        .recv_timeout(Duration::from_secs(3))
        .unwrap()
        .unwrap_err();
    assert!(
        error.contains("InvalidData") && error.contains("message too long"),
        "{error}"
    );
    assert!(!error.contains("stdout Codex fermé"));
    drop(tx);
    reader.join().unwrap();
    assert!(!alive.load(Ordering::SeqCst));
    assert!(
        !observations
            .0
            .lock()
            .unwrap()
            .events
            .iter()
            .any(|event| matches!(event.kind, ManagedEventKind::JournalFailed { .. })),
        "une saisie supplémentaire doit traverser la vraie garde du journal"
    );
    journal.lock().unwrap().take().unwrap().stop();
    let logs: String = fs::read_dir(journal_root.join("interactive"))
        .unwrap()
        .map(|entry| fs::read_to_string(entry.unwrap().path()).unwrap())
        .collect();
    assert_eq!(
        logs.lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .filter(|entry| entry["payload"]["write_confirmed"] == true)
            .count(),
        1
    );
    assert!(
        logs.lines()
            .map(|line| serde_json::from_str::<Value>(line).unwrap())
            .any(|entry| entry["event"] == "user_message"
                && entry["payload"]["body"] == "AJOUT HUMAIN"
                && entry["payload"]["from"] == "human")
    );
}

#[test]
fn spec093_lecteur_interactif_ignore_les_fils_codex_tiers_sans_fermer_le_parent() {
    struct Capture(Arc<Mutex<Vec<u8>>>);
    impl Write for Capture {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let output = Arc::new(Mutex::new(Vec::new()));
    let observations = Arc::new((Mutex::new(Observations::default()), Condvar::new()));
    let queue = Arc::new((
        Mutex::new(QueueState {
            messages: VecDeque::new(),
            steer: VecDeque::new(),
            active: None,
            steering_open: false,
            closed: false,
            external_turn: None,
        }),
        Condvar::new(),
    ));
    let alive = Arc::new(AtomicBool::new(true));
    let detail = Arc::new(Mutex::new(None));
    let pending_request = Arc::new(Mutex::new(None));
    let journal_root = root("foreign-codex-threads");
    let journal = Arc::new(Mutex::new(Some(
        JournalWriter::start(
            &journal_root,
            "interactive",
            "parent-thread",
            Arc::new(Mutex::new(crate::acp::AcpEventQueue::default())),
        )
        .unwrap(),
    )));
    let (tx, rx) = mpsc::channel::<std::io::Result<String>>();
    let waiters = Arc::new(Mutex::new(HashMap::new()));
    let reader = spawn_reader(
        Box::new(rx.into_iter()),
        ReaderContext {
            waiters: waiters.clone(),
            cwd: "/tmp".into(),
            observations: observations.clone(),
            alive: alive.clone(),
            journal: journal.clone(),
            queue: queue.clone(),
            pending_request: pending_request.clone(),
            child_requests: Arc::new(Mutex::new(Vec::new())),
            forward_child_requests: false,
            provider_session: "codex-test".into(),
            pinned_model: None,
            active_detail: detail.clone(),
            writer: Arc::new(Mutex::new(Some(Box::new(Capture(output.clone()))))),
            permissions: "allow".into(),
            dynamic_tool_handler: None,
            sandbox_posture: "complete",
            interactive: true,
            selected_thread: Arc::new(Mutex::new(Some("parent-thread".into()))),
            permission_observation: Arc::new(Mutex::new(None)),
        },
    );
    let send_and_wait = |raw: &str| {
        tx.send(Ok(raw.into())).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut state = observations.0.lock().unwrap();
        while alive.load(Ordering::SeqCst)
            && !state.events.iter().any(|event| event.raw == raw.as_bytes())
        {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .expect("événement parent attendu avant échéance");
            state = observations.1.wait_timeout(state, remaining).unwrap().0;
        }
        assert!(
            state.events.iter().any(|event| event.raw == raw.as_bytes()),
            "le reader doit traiter l'événement parent suivant"
        );
    };

    send_and_wait(
        r#"{"method":"turn/started","params":{"threadId":"parent-thread","turn":{"id":"parent-turn","status":"inProgress"}}}"#,
    );
    send_and_wait(
        r#"{"method":"item/started","params":{"threadId":"parent-thread","turnId":"parent-turn","item":{"type":"userMessage","content":[{"type":"text","text":"MESSAGE PARENT"}]}}}"#,
    );
    for raw in [
        r#"{"method":"thread/started","params":{"thread":{"id":"child-one","parentThreadId":"parent-thread","status":{"type":"active"}}}}"#,
        r#"{"method":"turn/started","params":{"threadId":"child-one","turn":{"id":"child-turn","status":"inProgress"}}}"#,
        r#"{"method":"thread/status/changed","params":{"threadId":"child-one","status":{"type":"idle"}}}"#,
        r#"{"method":"thread/status/changed","params":{"threadId":"resumed-child","status":{"type":"active"}}}"#,
        r#"{"method":"item/agentMessage/delta","params":{"threadId":"child-one","turnId":"child-turn","itemId":"child-answer","delta":"NE PAS ATTRIBUER"}}"#,
        r#"{"id":71,"method":"item/permissions/requestApproval","params":{"threadId":"child-one","turnId":"child-turn","permissions":{}}}"#,
    ] {
        tx.send(Ok(raw.into())).unwrap();
    }
    send_and_wait(
        r#"{"method":"item/agentMessage/delta","params":{"threadId":"parent-thread","turnId":"parent-turn","itemId":"parent-answer","delta":"PARENT TOUJOURS ACTIF"}}"#,
    );

    assert!(alive.load(Ordering::SeqCst));
    assert_eq!(
        queue.0.lock().unwrap().external_turn.as_deref(),
        Some("parent-turn")
    );
    {
        let active = detail.lock().unwrap();
        let active = active.as_ref().expect("tour parent conservé");
        assert_eq!(active.thread_id, "parent-thread");
        assert_eq!(active.turn_id.as_deref(), Some("parent-turn"));
    }
    assert!(pending_request.lock().unwrap().is_none());
    assert!(output.lock().unwrap().is_empty());
    assert_eq!(
        observations
            .0
            .lock()
            .unwrap()
            .response_by_turn
            .get("parent-turn")
            .map(String::as_str),
        Some("PARENT TOUJOURS ACTIF")
    );
    assert!(
        !observations
            .0
            .lock()
            .unwrap()
            .response_by_turn
            .contains_key("child-turn")
    );

    let (result_tx, result_rx) = mpsc::channel();
    waiters.lock().unwrap().insert(993, result_tx);
    drop(tx);
    reader.join().unwrap();
    assert!(!alive.load(Ordering::SeqCst));
    assert_eq!(
        result_rx
            .recv_timeout(Duration::from_secs(3))
            .unwrap()
            .unwrap_err(),
        "flux Codex fermé"
    );

    journal.lock().unwrap().take().unwrap().stop();
    let logs: String = fs::read_dir(journal_root.join("interactive"))
        .unwrap()
        .map(|entry| fs::read_to_string(entry.unwrap().path()).unwrap())
        .collect();
    assert!(logs.contains("parent-turn"));
    assert!(!logs.contains("child-one"));
    assert!(!logs.contains("resumed-child"));
}

fn root(label: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "bridget-codex-native-{label}-{}-{}",
        std::process::id(),
        FIXTURE_SEQ.fetch_add(1, AtomicOrdering::Relaxed)
    ));
    fs::create_dir_all(&root).expect("racine temporaire");

    root
}

#[test]
fn requete_fournisseur_inconnue_recoit_un_refus_json_rpc_redacte() {
    let sentinel = "SENTINELLE-METHODE-INCONNUE";
    let (reply, diagnostic, delegated_reference) = unsupported_provider_request_response(&json!({
        "id": "dynamic-tool-42",
        "method": sentinel,
        "params": { "secret": "ne-pas-publier" },
    }))
    .expect("réponse aux requêtes avec id");
    assert_eq!(reply["id"], "dynamic-tool-42");
    assert_eq!(reply["error"]["code"], -32601);
    assert_eq!(
        reply["error"]["data"]["code"],
        "unsupported_provider_request"
    );
    assert_eq!(diagnostic["code"], "unsupported_provider_request");
    let serialized = serde_json::to_string(&json!({ "reply": reply, "diagnostic": diagnostic }))
        .expect("diagnostic sérialisable");
    assert!(!serialized.contains(sentinel));
    assert!(!serialized.contains("ne-pas-publier"));
    assert!(serialized.contains("sha256:"));
    assert!(delegated_reference.starts_with("sha256:"));
    assert_ne!(delegated_reference, sentinel);
    assert!(!delegated_reference.contains("commandExecution"));
    assert!(!delegated_reference.contains("ne-pas-publier"));
}

#[test]
fn elicitation_mcp_vide_de_bridget_est_acceptee_et_un_formulaire_est_refuse() {
    let (reply, payload) = mcp_elicitation_response(&json!({
        "id": "elicitation-vide",
        "method": "mcpServer/elicitation/request",
        "params": {
            "serverName": "bridget",
            "mode": "form",
            "message": "ne pas enregistrer ce texte",
            "requestedSchema": {"type": "object", "properties": {}, "required": []}
        }
    }))
    .expect("élicitation MCP valide");
    assert_eq!(reply["id"], "elicitation-vide");
    assert_eq!(reply["result"]["action"], "accept");
    assert_eq!(reply["result"]["content"], json!({}));
    assert_eq!(payload["code"], "mcp_elicitation_autoaccepted");
    assert!(
        !serde_json::to_string(&payload)
            .expect("payload sérialisable")
            .contains("ne pas enregistrer ce texte")
    );

    let (reply, payload) = mcp_elicitation_response(&json!({
        "id": "elicitation-secret",
        "method": "mcpServer/elicitation/request",
        "params": {
            "serverName": "bridget",
            "mode": "form",
            "requestedSchema": {
                "type": "object",
                "properties": {"secret": {"type": "string"}},
                "required": ["secret"]
            }
        }
    }))
    .expect("élicitation MCP valide");
    assert_eq!(reply["result"]["action"], "decline");
    assert_eq!(reply["result"].get("content"), None);
    assert_eq!(payload["code"], "mcp_elicitation_declined");
    assert_eq!(payload["required_field_count"], 1);
}

#[test]
#[allow(non_snake_case)]
fn TEMOIN_codex_refus_update_hors_vocabulaire_emet_JournalFailed() {
    let root = root("act-kind-visible");
    let observations = Arc::new((Mutex::new(Observations::default()), Condvar::new()));
    let journal = Arc::new(Mutex::new(Some(
        JournalWriter::start(
            &root,
            "codex-1",
            "session-1",
            Arc::new(Mutex::new(crate::acp::AcpEventQueue::default())),
        )
        .unwrap(),
    )));
    record_or_terminal(
        &journal,
        &observations,
        "update",
        Some("m1"),
        json!({"kind":"command","text":"ls"}),
    );
    assert!(
        observations
            .0
            .lock()
            .unwrap()
            .events
            .iter()
            .all(|event| !matches!(event.kind, ManagedEventKind::JournalFailed { .. })),
        "écriture command valide ne doit pas émettre JournalFailed"
    );
    record_or_terminal(
        &journal,
        &observations,
        "update",
        Some("m1"),
        json!({"kind":"intent","text":"fantôme"}),
    );
    let drained: Vec<_> = observations.0.lock().unwrap().events.drain(..).collect();
    assert!(
        drained.iter().any(|event| matches!(
            &event.kind,
            ManagedEventKind::JournalFailed { detail }
                if detail.contains("hors vocabulaire")
        )),
        "refus hors vocabulaire doit être visible via JournalFailed, got {drained:?}"
    );
    if let Some(writer) = journal.lock().unwrap().take() {
        writer.stop();
    }
    let _ = fs::remove_dir_all(root);
}

/// Dette 3 — oracle bout-en-bout Codex : commandExecution → record_active_act
/// avec kind interdit → JournalFailed drainé. Preuve d'abord qu'un cas valide passe.
#[test]
#[allow(non_snake_case)]
fn TEMOIN_codex_pilote_kind_interdit_emet_JournalFailed_bout_en_bout() {
    let (events_ok, journal_ok) = {
        let _suite = crate::act_kind::pilot_kind_suite_lock();
        codex_activity_turn_managed_events("e2e-ok")
    };
    assert!(
        events_ok
            .iter()
            .all(|event| { !matches!(event.kind, ManagedEventKind::JournalFailed { .. }) }),
        "cas valide ne doit pas émettre JournalFailed, got {events_ok:?}"
    );
    assert!(
        journal_ok
            .iter()
            .any(|event| { event["event"] == "update" && event["payload"]["kind"] == "command" }),
        "cas valide doit journaliser kind=command, got {journal_ok:?}"
    );

    let (events_bad, journal_bad) =
        crate::act_kind::with_forced_pilot_update_kind("intent", || {
            codex_activity_turn_managed_events("e2e-bad")
        });
    assert!(
        events_bad.iter().any(|event| matches!(
            &event.kind,
            ManagedEventKind::JournalFailed { detail }
                if detail.contains("hors vocabulaire")
        )),
        "kind interdit du pilote doit être VISIBLE via JournalFailed, got {events_bad:?}"
    );
    assert!(
        journal_bad
            .iter()
            .all(|event| event["payload"]["kind"] != "intent"),
        "le kind refusé ne doit jamais atteindre le JSONL, got {journal_bad:?}"
    );
}

fn codex_activity_turn_managed_events(label: &str) -> (Vec<ManagedEvent>, Vec<Value>) {
    codex_activity_turn_managed_events_with(label, &[], None)
}

/// Variante paramétrée : variables d'environnement supplémentaires pour la
/// fixture, et arguments de lancement (posture SPEC-088).
fn codex_activity_turn_managed_events_with(
    label: &str,
    extra_environment: &[(&str, &str)],
    args: Option<Vec<String>>,
) -> (Vec<ManagedEvent>, Vec<Value>) {
    let root = root(label);
    let trace = root.join("trace.jsonl");
    let mut environment = vec![
        (
            "BRIDGET_CODEX_TRACE".to_string(),
            trace.to_string_lossy().into_owned(),
        ),
        ("BRIDGET_CODEX_ACTIVITY".to_string(), "1".to_string()),
    ];
    environment.extend(
        extra_environment
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string())),
    );
    let mut options = fake_options(&trace);
    if let Some(args) = args {
        options.args = args;
    }
    let mut transport =
        CodexAppServerTransport::spawn_with_environment(options, &environment, false)
            .expect("session native e2e");
    transport
        .activate_journal(&root, "codex-native", None)
        .expect("journal e2e");
    transport.deliver(&message(label)).expect("livraison e2e");
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut events = Vec::new();
    while Instant::now() < deadline {
        events.extend(transport.drain_events());
        if events.iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::TurnFinished {
                    terminal: ManagedTerminal::Completed,
                    ..
                }
            )
        }) {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    events.extend(transport.drain_events());
    transport.stop();
    let journal_path = fs::read_dir(root.join("codex-native"))
        .expect("répertoire journal")
        .next()
        .expect("fichier journal")
        .expect("entrée journal")
        .path();
    let journal_events = crate::journal::valid_events(&journal_path);
    let _ = fs::remove_dir_all(&root);
    (events, journal_events)
}

/// SPEC-088 T006 révisé : ligne de sandbox + fin en ÉCHEC du même item ⇒
/// UN acte `refusal` (signalement non attesté) ; ligne + fin réussie ⇒
/// aucun ; fin en échec sans ligne ⇒ aucun refus mais l'acte `command`
/// porte l'échec. Contrôle positif : la posture est lue des arguments.
#[test]
fn spec_088_signalement_de_sandbox_exige_la_ligne_et_l_echec_du_meme_item() {
    let _suite = crate::act_kind::pilot_kind_suite_lock();
    let discovery_args = || {
        let mut args = fake_options(Path::new("/tmp/unused")).args;
        // Après le script `sh -c` : paramètres positionnels inoffensifs,
        // mais visibles pour la lecture de posture.
        args.extend(["--sandbox".to_string(), "read-only".to_string()]);
        args
    };
    let refusals_of = |journal: &[Value]| -> Vec<Value> {
        journal
            .iter()
            .filter(|event| event["event"] == "update" && event["payload"]["kind"] == "refusal")
            .cloned()
            .collect()
    };

    // 1. Ligne bwrap (deux fois) + fin en échec ⇒ un seul acte.
    let (_events, journal) = codex_activity_turn_managed_events_with(
        "spec-088-line-failed",
        &[
            ("BRIDGET_CODEX_SANDBOX_REFUSAL", "1"),
            ("BRIDGET_CODEX_COMMAND_COMPLETION", "failed"),
        ],
        Some(discovery_args()),
    );
    let refusals = refusals_of(&journal);
    assert_eq!(refusals.len(), 1, "un seul signalement, got {journal:?}");
    let payload = &refusals[0]["payload"];
    assert_eq!(payload["layer"], "provider_sandbox");
    assert_eq!(payload["evidence"], "output_and_exit");
    assert_eq!(payload["provider"], "codex");
    assert_eq!(payload["posture"], "discovery");
    assert_eq!(payload["item_id"], "command-1");
    assert_eq!(payload["exit_code"], 1);
    assert_eq!(payload["attributed_to"], "bridget");
    assert_eq!(payload["gesture"]["target"], "shell");
    assert!(
        journal.iter().any(|event| {
            event["payload"]["kind"] == "command"
                && event["payload"]["state"] == "failed"
                && event["payload"]["item_id"] == "command-1"
        }),
        "la fin de commande est journalisée en échec, got {journal:?}"
    );

    // 2. Ligne bwrap + fin RÉUSSIE ⇒ aucun signalement (un `echo` ne prouve rien).
    let (_events, journal_ok) = codex_activity_turn_managed_events_with(
        "spec-088-line-ok",
        &[
            ("BRIDGET_CODEX_SANDBOX_REFUSAL", "1"),
            ("BRIDGET_CODEX_COMMAND_COMPLETION", "completed"),
        ],
        Some(discovery_args()),
    );
    assert!(
        refusals_of(&journal_ok).is_empty(),
        "sortie 0 ⇒ aucun refus, got {journal_ok:?}"
    );
    assert!(
        journal_ok.iter().any(|event| {
            event["payload"]["kind"] == "command"
                && event["payload"]["state"] == "completed"
                && event["payload"]["exit_code"] == 0
                && event["payload"]["output_tail"]
                    .as_str()
                    .is_some_and(|tail| tail.contains("BRIDGET-TEST-fixture"))
        }),
        "la fin réussie porte le code 0 et la queue de sortie, got {journal_ok:?}"
    );

    // 3. Fin en échec SANS ligne reconnue ⇒ aucun refus.
    let (_events, journal_failed_only) = codex_activity_turn_managed_events_with(
        "spec-088-failed-only",
        &[("BRIDGET_CODEX_COMMAND_COMPLETION", "failed")],
        None,
    );
    assert!(
        refusals_of(&journal_failed_only).is_empty(),
        "échec sans motif ⇒ aucun refus"
    );

    // 4. Ligne sans aucune fin ⇒ rien (une ligne seule n'est pas une preuve).
    let (_events, journal_line_only) = codex_activity_turn_managed_events_with(
        "spec-088-line-only",
        &[("BRIDGET_CODEX_SANDBOX_REFUSAL", "1")],
        None,
    );
    assert!(
        refusals_of(&journal_line_only).is_empty(),
        "ligne seule ⇒ aucun refus"
    );
    assert_eq!(
        sandbox_posture_from_args(&fake_options(Path::new("/tmp/x")).args),
        "complete"
    );
}

#[test]
#[allow(non_snake_case)]
fn mutant_avale_err_record_tue_TEMOIN_codex_JournalFailed() {
    fn broken_swallow(
        journal: &Journal,
        _observations: &Arc<(Mutex<Observations>, Condvar)>,
        event: &str,
        message_id: Option<&str>,
        payload: Value,
    ) {
        let _ = record(journal, event, message_id, payload);
    }
    let root = root("act-kind-mutant");
    let observations = Arc::new((Mutex::new(Observations::default()), Condvar::new()));
    let journal = Arc::new(Mutex::new(Some(
        JournalWriter::start(
            &root,
            "codex-1",
            "session-1",
            Arc::new(Mutex::new(crate::acp::AcpEventQueue::default())),
        )
        .unwrap(),
    )));
    broken_swallow(
        &journal,
        &observations,
        "update",
        Some("m1"),
        json!({"kind":"intent"}),
    );
    assert!(
        observations.0.lock().unwrap().events.is_empty(),
        "le mutant avale l'erreur"
    );
    record_or_terminal(
        &journal,
        &observations,
        "update",
        Some("m1"),
        json!({"kind":"intent"}),
    );
    assert!(
        observations
            .0
            .lock()
            .unwrap()
            .events
            .iter()
            .any(|event| matches!(
                &event.kind,
                ManagedEventKind::JournalFailed { detail }
                    if detail.contains("hors vocabulaire")
            )),
        "TEMOIN_codex_JournalFailed doit mourir si Err est avalée"
    );
    if let Some(writer) = journal.lock().unwrap().take() {
        writer.stop();
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn spec091_selection_reelle_du_prochain_tour_sans_prompt_cache_ni_perte_du_fil() {
    use crate::protocol::{
        RuntimeSelection, RuntimeSelectionOutcome as Outcome, RuntimeSelectionRefusal as Refusal,
    };
    let root = root("runtime091");
    let trace = root.join("wire.jsonl");
    let mut options = fake_options(&trace);
    options.command = "/usr/bin/python3".into();
    options.args = vec!["-u".into(), "-c".into(), r#"
import json, sys
settings = {'model':'first','effort':'low'}
for line in sys.stdin:
    with open(sys.argv[1], 'a') as trace: trace.write(line)
    value = json.loads(line); method = value.get('method'); params = value.get('params', {})
    result = {}
    if method == 'initialize': result = {'userAgent':'fixture091','codexHome':'/tmp','platformFamily':'unix','platformOs':'macos'}
    if method == 'thread/start': result = {'thread':{'id':'same-thread'},'model':'first','reasoningEffort':'low'}
    if method == 'model/list': result = {'data':[{'model':m,'supportedReasoningEfforts':[{'reasoningEffort':e} for e in ['low','high']]} for m in ['first','second']], 'nextCursor':None}
    if method == 'thread/settings/update': settings = {'model':params['model'],'effort':params['effort']}
    if method == 'turn/start': result = {'turn':{'id':str(value['id'])}}
    if 'id' in value: print(json.dumps({'id':value['id'],'result':result}), flush=True)
    if method == 'thread/settings/update':
        print(json.dumps({'method':'thread/settings/updated','params':{'threadId':params['threadId'],'threadSettings':settings}}), flush=True)
    if method == 'turn/start':
        print(json.dumps({'method':'turn/completed','params':{'threadId':params['threadId'],'turn':{'id':str(value['id']),'status':'completed'}}}), flush=True)
"#.into(), trace.to_string_lossy().into_owned()];
    let mut session = CodexAppServerTransport::spawn(options).unwrap();
    let crate::protocol::RuntimeCatalogueOutcome::Observed { models } = session.runtime_catalogue()
    else {
        panic!("catalogue de la session attendu");
    };
    assert_eq!(
        models.iter().map(|m| m.model.as_str()).collect::<Vec<_>>(),
        ["first", "second"]
    );
    assert_eq!(models[0].supported_efforts, ["low", "high"]);
    assert!(
        !fs::read_to_string(&trace)
            .unwrap()
            .contains("thread/settings/update"),
        "la lecture du catalogue ne sélectionne aucun réglage"
    );
    let requested = RuntimeSelection {
        model: "second".into(),
        effort: "high".into(),
    };
    assert_eq!(
        session.select_runtime(requested.clone()),
        Outcome::Selected {
            selection: requested
        }
    );
    let before = fs::read_to_string(&trace).unwrap();
    assert!(
        !before.contains("turn/start"),
        "la sélection ne doit pas lancer de prompt"
    );
    assert_eq!(
        session.select_runtime(RuntimeSelection {
            model: "absent".into(),
            effort: "high".into()
        }),
        Outcome::Refused {
            reason: Refusal::ModelUnavailable
        }
    );
    assert_eq!(
        session.select_runtime(RuntimeSelection {
            model: "first".into(),
            effort: "invented".into()
        }),
        Outcome::Refused {
            reason: Refusal::EffortUnavailable
        }
    );
    for (id, model, effort) in [("change-1", "second", "high"), ("change-2", "first", "low")] {
        if id == "change-2" {
            assert!(matches!(
                session.select_runtime(RuntimeSelection {
                    model: model.into(),
                    effort: effort.into()
                }),
                Outcome::Selected { .. }
            ));
        }
        session.deliver(&message(id)).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut finished = false;
        let mut observed = false;
        while Instant::now() < deadline && !(finished && observed) {
            for event in session.drain_events() {
                if let ManagedEventKind::RuntimeObserved {
                    model: actual,
                    effort: actual_effort,
                } = &event.kind
                    && actual == model
                    && actual_effort.as_deref() == Some(effort)
                {
                    assert_eq!(
                        event.origin,
                        crate::managed_session::ManagedEventOrigin::SourceLine
                    );
                    assert!(
                        String::from_utf8_lossy(&event.raw).contains("thread/settings/updated")
                    );
                    observed = true;
                }
                finished |= matches!(event.kind, ManagedEventKind::TurnFinished { .. });
            }
            thread::sleep(Duration::from_millis(5));
        }
        assert!(
            finished && observed,
            "tour et réglages doivent être attestés"
        );
    }
    session.stop();
    let frames: Vec<Value> = fs::read_to_string(trace)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let turns: Vec<_> = frames
        .iter()
        .filter(|frame| frame["method"] == "turn/start")
        .collect();
    assert_eq!(turns.len(), 2);
    let updates: Vec<_> = frames
        .iter()
        .filter(|frame| frame["method"] == "thread/settings/update")
        .collect();
    assert_eq!(updates.len(), 2);
    for (update, model, effort) in [(updates[0], "second", "high"), (updates[1], "first", "low")] {
        assert_eq!(
            update["params"],
            json!({"threadId":"same-thread","model":model,"effort":effort})
        );
    }
    assert!(
        turns
            .iter()
            .all(|turn| turn["params"]["threadId"] == "same-thread"
                && turn["params"].get("model").is_none())
    );
    // Mutation : retirer la commande native settings/update casse cet oracle ;
    // remplacer RuntimeObserved par une déclaration synthétique le casse aussi.
}

#[test]
fn mobile183_compaction_reaches_the_real_journal_without_foreign_items() {
    let _suite = crate::act_kind::pilot_kind_suite_lock();
    let (events, journal) = codex_activity_turn_managed_events_with(
        "mobile183",
        &[("BRIDGET_CODEX_COMPACTION", "1")],
        None,
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e.kind, ManagedEventKind::JournalFailed { .. }))
    );
    let facts: Vec<_> = journal
        .iter()
        .filter(|e| e["payload"]["tool_type"] == "context_compaction")
        .collect();
    assert_eq!(facts.len(), 2, "{journal:?}");
    assert_eq!(facts[0]["payload"]["status"], "running");
    assert_eq!(facts[1]["payload"]["status"], "completed");
    assert!(
        facts
            .iter()
            .all(|e| e["payload"]["item_id"] == "compact183")
    );
}

#[test]
fn spec217_model_refusal_requires_closed_bounded_http_envelope() {
    let body = json!({"type":"error","status":400,"error":{"type":"invalid_request_error",
            "code":"model_not_found","message":"PRIVATE_MODEL_ACCOUNT"}});
    let event = |body: &Value| json!({"params":{"turn":{"error":{"codexErrorInfo":"other","message":body.to_string()}}}});
    let value = event(&body);
    assert_eq!(
        codex_turn_stop_reason(&value, "failed"),
        "terminal Codex: failed:cause=model_not_found"
    );
    assert!(
        matches!(codex_turn_terminal(&value,"failed"),ManagedTerminal::Failed {detail}
            if detail=="terminal Codex: failed:cause=model_not_found")
    );
    for (path, bad) in [
        ("/status", json!(401)),
        ("/status", json!("400")),
        ("/type", json!("PRIVATE")),
        ("/error/type", json!("other")),
        ("/error/code", json!("PRIVATE_SECRET")),
    ] {
        let mut invalid = body.clone();
        *invalid.pointer_mut(path).unwrap() = bad;
        assert_eq!(codex_turn_stop_reason(&event(&invalid), "failed"), "failed");
    }
    for error in [
        json!({"codexErrorInfo":"other","message":"model_not_found"}),
        json!({"codexErrorInfo":"unknown","message":body.to_string()}),
        json!({"codexErrorInfo":"other","message":" ".repeat(4097)}),
        json!({"codexErrorInfo":"other","message":body}),
    ] {
        assert_eq!(
            codex_turn_stop_reason(&json!({"params":{"turn":{"error":error}}}), "failed"),
            "failed"
        );
    }
    assert!(matches!(
        codex_turn_terminal(&value, "completed"),
        ManagedTerminal::Completed
    ));
    assert!(matches!(
        codex_turn_terminal(&value, "interrupted"),
        ManagedTerminal::Cancelled
    ));
    assert_eq!(codex_turn_stop_reason(&value, "completed"), "completed");
}

#[test]
fn spec204_terminal_cause_requires_attested_code_and_failed_status() {
    let overloaded = json!({"params":{"turn":{"error":{"codexErrorInfo":"serverOverloaded",
            "message":"PRIVATE_PROVIDER_MESSAGE", "additionalDetails":"PRIVATE_DETAIL"}}}});
    assert!(
        matches!(codex_turn_terminal(&overloaded,"failed"), ManagedTerminal::Failed { detail }
            if detail == "terminal Codex: failed:cause=server_overloaded")
    );
    for error in [
        Value::Null,
        json!({"message":"serverOverloaded"}),
        json!({"codexErrorInfo":"PRIVATE_UNKNOWN_CODE"}),
        json!({"codexErrorInfo":{"serverOverloaded":"PRIVATE"}}),
    ] {
        let value = json!({"params":{"turn":{"error":error}}});
        assert!(
            matches!(codex_turn_terminal(&value,"failed"), ManagedTerminal::Failed { detail }
                if detail == "terminal Codex: failed")
        );
    }
    assert_eq!(
        codex_turn_stop_reason(&overloaded, "failed"),
        "terminal Codex: failed:cause=server_overloaded"
    );
    assert_eq!(codex_turn_stop_reason(&json!({}), "failed"), "failed");
    assert_eq!(
        codex_turn_stop_reason(&overloaded, "completed"),
        "completed"
    );
    assert_eq!(
        codex_turn_stop_reason(&overloaded, "interrupted"),
        "interrupted"
    );
    assert!(matches!(
        codex_turn_terminal(&overloaded, "completed"),
        ManagedTerminal::Completed
    ));
    assert!(matches!(
        codex_turn_terminal(&overloaded, "interrupted"),
        ManagedTerminal::Cancelled
    ));
}

#[test]
fn spec204_failed_terminal_reaches_real_transport_and_journal() {
    let root = root("spec204-terminal");
    let trace = root.join("trace.jsonl");
    let mut options = fake_options(&trace);
    let old = r#""id":"turn-native","status":"completed","items":[]"#;
    let new = r#""id":"turn-native","status":"failed","items":[],"error":{"codexErrorInfo":"serverOverloaded","message":"PRIVATE_PROVIDER_MESSAGE"}"#;
    assert!(options.args[1].contains(old));
    options.args[1] = options.args[1].replace(old, new);
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[(
            "BRIDGET_CODEX_TRACE".into(),
            trace.to_string_lossy().into_owned(),
        )],
        false,
    )
    .unwrap();
    transport
        .activate_journal(&root, "codex-native", None)
        .unwrap();
    transport.deliver(&message("spec204")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(4);
    let mut finished = false;
    while Instant::now() < deadline && !finished {
        for event in transport.drain_events() {
            if let ManagedEventKind::TurnFinished { terminal, .. } = event.kind {
                assert!(matches!(terminal,ManagedTerminal::Failed {detail}
                        if detail=="terminal Codex: failed:cause=server_overloaded"));
                finished = true;
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(finished);
    transport.stop();
    let journal_path = fs::read_dir(root.join("codex-native"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let events = crate::journal::valid_events(&journal_path);
    assert!(events.iter().any(|event| event["event"] == "turn_end"
        && event["payload"]["stop_reason"] == "terminal Codex: failed:cause=server_overloaded"));
    assert!(
        !serde_json::to_string(&events)
            .unwrap()
            .contains("PRIVATE_PROVIDER_MESSAGE")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec217_failed_terminal_reaches_real_transport_and_journal() {
    let root = root("spec217-terminal");
    let trace = root.join("trace.jsonl");
    let mut options = fake_options(&trace);
    let old = r#""id":"turn-native","status":"completed","items":[]"#;
    let error = json!({"codexErrorInfo":"other","message":json!({"type":"error","status":400,
            "error":{"type":"invalid_request_error","code":"model_not_found","message":"PRIVATE_PROVIDER_MESSAGE"}}).to_string()});
    let new = format!(
        r#""id":"turn-native","status":"failed","items":[],"error":{}"#,
        error
    );
    assert!(options.args[1].contains(old));
    options.args[1] = options.args[1].replace(old, &new);
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[(
            "BRIDGET_CODEX_TRACE".into(),
            trace.to_string_lossy().into_owned(),
        )],
        false,
    )
    .unwrap();
    transport
        .activate_journal(&root, "codex-native", None)
        .unwrap();
    transport.deliver(&message("spec217")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(4);
    let mut finished = false;
    while Instant::now() < deadline && !finished {
        for event in transport.drain_events() {
            if let ManagedEventKind::TurnFinished { terminal, .. } = event.kind {
                assert!(matches!(terminal,ManagedTerminal::Failed {detail}
                        if detail=="terminal Codex: failed:cause=model_not_found"));
                finished = true;
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(finished);
    transport.stop();
    let journal_path = fs::read_dir(root.join("codex-native"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let events = crate::journal::valid_events(&journal_path);
    assert!(events.iter().any(|event| event["event"] == "turn_end"
        && event["payload"]["stop_reason"] == "terminal Codex: failed:cause=model_not_found"));
    assert!(
        !serde_json::to_string(&events)
            .unwrap()
            .contains("PRIVATE_PROVIDER_MESSAGE")
    );
    fs::remove_dir_all(root).unwrap();
}

fn fake_options(_trace: &std::path::Path) -> CodexAppServerOptions {
    CodexAppServerOptions {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                r#"started=0; saturated=0; while IFS= read -r line; do
                    printf '%s\n' "$line" >> "$BRIDGET_CODEX_TRACE"
                    case "$line" in
                        *'"method":"initialize"'*) if [ -n "${BRIDGET_CODEX_CHILD_PID:-}" ]; then sleep 60 & printf '%s' "$!" > "$BRIDGET_CODEX_CHILD_PID"; fi; printf '%s\n' '{"id":1,"result":{"userAgent":"fake","codexHome":"/tmp","platformFamily":"unix","platformOs":"macos"}}' ;;
                        *'"method":"initialized"'*) started=1 ;;
                        *'"method":"thread/start"'*) printf '%s\n' '{"id":2,"result":{"thread":{"id":"thread-native"},"model":"gpt-5.6-terra","reasoningEffort":"high"}}' ;;
                        *'"method":"account/rateLimits/read"'*)
                            printf '%s\n' '{"id":3,"result":{"rateLimits":{"primary":{"usedPercent":42,"windowDurationMins":300,"resetsAt":1787572200},"rateLimitReachedType":null}}}'
                            if [ "$BRIDGET_CODEX_CREDIT_UPDATES" = "1" ]; then
                                printf '%s\n' '{"method":"account/rateLimits/updated","params":{"rateLimits":{"primary":{"usedPercent":100,"windowDurationMins":10080,"resetsAt":1787572200},"credits":{"hasCredits":true,"unlimited":false},"spendControlReached":false,"rateLimitReachedType":"rate_limit_reached"}}}'
                                printf '%s\n' '{"method":"account/rateLimits/updated","params":{"rateLimits":{"credits":null}}}'
                            fi
                            ;;
                        *'"method":"turn/start"'*)
                            if [ "$started" != 1 ]; then printf '%s\n' '{"id":4,"error":{"code":-32099,"message":"initialized absent"}}'
                            elif [ "$saturated" = 0 ]; then saturated=1; printf '%s\n' '{"id":4,"error":{"code":-32001,"message":"saturated"}}'
                            elif [ "${BRIDGET_CODEX_TURN_ERROR:-0}" = 1 ]; then malicious_reason='{"id":5,"error":{"code":-32098,"message":"SENTINELLE-RAISON-019\u001b[2J\r\u202e"}}'; printf '%s\n' "$malicious_reason" >> "$BRIDGET_CODEX_OUTPUT_TRACE"; printf '%s\n' "$malicious_reason"
                            else
                                printf '%s\n' '{"id":5,"result":{"turn":{"id":"turn-native"}}}'
                                if [ "${BRIDGET_CODEX_COMPACTION:-0}" = 1 ]; then
                                    printf '%s\n' '{"method":"item/started","params":{"threadId":"foreign","turnId":"turn-native","item":{"type":"contextCompaction","id":"bad"}}}'
                                    printf '%s\n' '{"method":"item/started","params":{"threadId":"thread-native","turnId":"turn-native","item":{"type":"contextCompaction","id":"compact183"}}}'
                                    printf '%s\n' '{"method":"item/completed","params":{"threadId":"thread-native","turnId":"turn-native","item":{"type":"contextCompaction","id":"compact183"}}}'
                                fi
                                if [ "${BRIDGET_CODEX_REQUEST_APPROVAL:-0}" = 1 ]; then
                                    case "${BRIDGET_CODEX_REQUEST_VARIANT:-known}" in
                                        method) printf '%s\n' '{"id":"approval-native","method":"SENTINELLE-METHOD-019\u001b[2J\r\u202e","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"i","command":"SENTINELLE-SECRETE-019"}}' ;;
                                        request_id) printf '%s\n' '{"id":"SENTINELLE-REQUEST-ID-019","method":"item/commandExecution/requestApproval","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"i","command":"SENTINELLE-SECRETE-019","availableDecisions":["accept","decline","cancel"]}}' ;;
                                        turn_id) printf '%s\n' '{"id":"approval-native","method":"item/commandExecution/requestApproval","params":{"threadId":"thread-native","turnId":"SENTINELLE-TURN-ID-019","itemId":"i","command":"SENTINELLE-SECRETE-019","availableDecisions":["accept","decline","cancel"]}}' ;;
                                        *) printf '%s\n' '{"id":"approval-native","method":"item/commandExecution/requestApproval","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"i","command":"SENTINELLE-SECRETE-019","availableDecisions":["accept","decline","cancel"]}}' ;;
                                    esac
                                    # Attend la décision JSON-RPC (présence), puis poursuit
                                    # le tour (effet) — sauf HOLD/EXIT pour les bancs 019.
                                    saw_decision=0
                                    while IFS= read -r reply; do
                                        printf '%s\n' "$reply" >> "$BRIDGET_CODEX_TRACE"
                                        case "$reply" in
                                            *'"decision"'*) saw_decision=1; break ;;
                                        esac
                                    done
                                    if [ "${BRIDGET_CODEX_EXIT_AFTER_REQUEST:-0}" = 1 ]; then exit 0; fi
                                    if [ "${BRIDGET_CODEX_HOLD_TURN:-0}" = 1 ]; then :;
                                    elif [ "$saw_decision" = 1 ]; then
                                        printf '%s\n' '{"method":"item/agentMessage/delta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"i","delta":"tour poursuivi apres autorisation"}}'
                                        printf '%s\n' '{"method":"turn/completed","params":{"threadId":"thread-native","turn":{"id":"turn-native","status":"completed","items":[]}}}'
                                    fi
                                elif [ "${BRIDGET_CODEX_HOLD_TURN:-0}" != 1 ]; then
                                    if [ "${BRIDGET_CODEX_ACTIVITY:-0}" = 1 ]; then
                                        printf '%s\n' '{"method":"item/reasoning/summaryPartAdded","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"reasoning-1","summaryIndex":0}}'
                                        printf '%s\n' '{"method":"item/reasoning/summaryTextDelta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"reasoning-1","summaryIndex":0,"delta":"Je compare"}}'
                                        printf '%s\n' '{"method":"item/reasoning/summaryTextDelta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"reasoning-1","summaryIndex":0,"delta":" les options."}}'
                                        printf '%s\n' '{"method":"item/reasoning/textDelta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"reasoning-1","contentIndex":0,"delta":"raison brute"}}'
                                        printf '%s\n' '{"method":"item/started","params":{"threadId":"thread-native","turnId":"turn-native","startedAtMs":1,"item":{"type":"commandExecution","id":"exec-mesure","command":"echo MESURE_CODEX_CMD_77","cwd":"/tmp/bt","status":"inProgress"}}}'
                                        printf '%s\n' '{"method":"item/commandExecution/outputDelta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"command-1","delta":"313 passés"}}'
                                        if [ "${BRIDGET_CODEX_SANDBOX_REFUSAL:-0}" = 1 ]; then
                                            printf '%s\n' '{"method":"item/commandExecution/outputDelta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"command-1","delta":"bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted\nls: cannot access x\n"}}'
                                            printf '%s\n' '{"method":"item/commandExecution/outputDelta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"command-1","delta":"bwrap: execvp: Permission denied\n"}}'
                                        fi
                                        if [ "${BRIDGET_CODEX_COMMAND_COMPLETION:-}" = failed ]; then
                                            printf '%s\n' '{"method":"item/completed","params":{"threadId":"thread-native","turnId":"turn-native","item":{"type":"commandExecution","id":"command-1","command":"echo MESURE_CODEX_CMD_77","cwd":"/tmp/bt","status":"failed","exitCode":1,"aggregatedOutput":"bwrap: loopback: Failed RTM_NEWADDR: Operation not permitted\n"}}}'
                                        elif [ "${BRIDGET_CODEX_COMMAND_COMPLETION:-}" = completed ]; then
                                            printf '%s\n' '{"method":"item/completed","params":{"threadId":"thread-native","turnId":"turn-native","item":{"type":"commandExecution","id":"command-1","command":"echo MESURE_CODEX_CMD_77","cwd":"/tmp/bt","status":"completed","exitCode":0,"aggregatedOutput":"313 passés\nBRIDGET-TEST-fixture\n"}}}'
                                        fi
                                        printf '%s\n' '{"method":"item/commandExecution/outputDelta","params":{"threadId":"foreign-thread","turnId":"foreign-turn","itemId":"foreign-command","delta":"ne pas attribuer"}}'
                                        printf '%s\n' '{"method":"item/fileChange/patchUpdated","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"file-1","changes":[{"path":"src/main.rs","kind":{"type":"update"},"diff":"@@ -1 +1 @@"}]}}'
                                        printf '%s\n' '{"method":"item/plan/delta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"plan-1","delta":"Tester le flux"}}'
                                        printf '%s\n' '{"id":"approval-7","method":"item/commandExecution/requestApproval","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"approval-1","startedAtMs":1787686800000,"command":"cargo test -p bridget-transport","reason":"sortie réseau","availableDecisions":["accept","decline","cancel"]}}'
                                        printf '%s\n' '{"id":"approval-8","method":"item/fileChange/requestApproval","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"approval-2","startedAtMs":1787686800000,"reason":"écrire le fichier","availableDecisions":["accept","decline","cancel"]}}'
                                        printf '%s\n' '{"id":"approval-9","method":"item/permissions/requestApproval","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"approval-3","startedAtMs":1787686800000,"cwd":"/tmp","permissions":{},"reason":"accès réseau","availableDecisions":["accept","decline","cancel"]}}'
                                        printf '%s\n' '{"method":"item/futureWidget/delta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"future-1","delta":"ne pas inventer"}}'
                                    fi
                                    case "${BRIDGET_CODEX_TEXT_MODE:-default}" in
                                        deltas)
                                            printf '%s\n' '{"method":"item/agentMessage/delta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"i","delta":"BONJOUR "}}'
                                            printf '%s\n' '{"method":"item/agentMessage/delta","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"i","delta":"JE SUIS VIVANT"}}'
                                            printf '%s\n' '{"method":"item/completed","params":{"threadId":"thread-native","turnId":"turn-native","completedAtMs":1,"item":{"id":"i","type":"agentMessage","text":"BONJOUR JE SUIS VIVANT"}}}'
                                            ;;
                                        final_only)
                                            printf '%s\n' '{"method":"item/completed","params":{"threadId":"thread-native","turnId":"turn-native","completedAtMs":1,"item":{"id":"i","type":"agentMessage","text":"REPONSE FINALE"}}}'
                                            ;;
                                        *)
                                            printf '%s\n' '{  "method" : "item/agentMessage/delta" , "params" : { "threadId" : "thread-native" , "turnId" : "turn-native" , "itemId":"i", "delta" : "réponse native" } }'
                                            ;;
                                    esac
                                    printf '%s\n' '{"method":"turn/completed","params":{"threadId":"thread-native","turn":{"id":"turn-native","status":"completed","items":[]}}}'
                                fi
                            fi ;;
                        *'"method":"turn/steer"'*)
                            steer_id=$(printf '%s' "$line" | sed 's/.*"id":\([0-9]*\).*/\1/')
                            if [ "${BRIDGET_CODEX_STEER_REFUSE:-0}" = 1 ]; then
                                printf '%s\n' "{\"id\":$steer_id,\"error\":{\"code\":-32600,\"message\":\"cannot steer a review turn\",\"data\":{\"message\":\"cannot steer a review turn\",\"codexErrorInfo\":{\"activeTurnNotSteerable\":{\"turnKind\":\"review\"}}}}}"
                            else
                                printf '%s\n' "{\"id\":$steer_id,\"result\":{}}"
                                if [ "${BRIDGET_CODEX_STEER_CONSUME:-0}" = 1 ]; then
                                    steer_message_id=$(printf '%s' "$line" | sed 's/.*"clientUserMessageId":"\([^"]*\)".*/\1/')
            thread_bootstrap: CodexThreadBootstrap::Start,
                                    provider_item_id="provider-item-$steer_message_id"
                                    printf '%s\n' "{\"method\":\"item/started\",\"params\":{\"threadId\":\"thread-native\",\"turnId\":\"turn-native\",\"startedAtMs\":1,\"item\":{\"id\":\"$provider_item_id\",\"clientId\":\"$steer_message_id\",\"type\":\"userMessage\",\"content\":[]}}}"
                                fi
                            fi ;;
                        *'"method":"native155/requests"'*) cat "$BRIDGET_CODEX_CHILD_REQUESTS_FILE" ;;
                        *'"method":"native155/stall"'*)
                            : > "$BRIDGET_CODEX_STALL_READY"
                            while [ ! -f "$BRIDGET_CODEX_STALL_RELEASE" ]; do sleep 0.01; done
                            printf '%s\n' '{"method":"turn/completed","params":{"threadId":"thread-native","turn":{"id":"turn-native","status":"completed","items":[]}}}'
                            while :; do sleep 1; done ;;
                        *'"method":"native155/finish"'*) printf '%s\n' '{"method":"turn/completed","params":{"threadId":"thread-native","turn":{"id":"turn-native","status":"completed","items":[]}}}' ;;
                        *'"method":"turn/interrupt"'*)
                            interrupt_id=$(printf '%s' "$line" | sed 's/.*"id":\([0-9]*\).*/\1/')
                            printf '%s\n' "{\"id\":$interrupt_id,\"result\":{}}"
                            printf '%s\n' '{"method":"turn/completed","params":{"threadId":"thread-native","turn":{"id":"turn-native","status":"interrupted","items":[]}}}' ;;
                    esac
                done"#.to_string(),
            ],
            queue_capacity: 2,
            notify_timeout_secs: 2,
            provider_observation: None,
            thread_bootstrap: Default::default(),
            model: Some("gpt-5.6-terra".to_string()),
            permissions: "allow".to_string(),
            child_requests: false,
            dynamic_tool_handler: None,
        }
}

fn message(id: &str) -> BridgetMessage {
    let mut message = BridgetMessage::new("bridget", "codex-native", format!("mission {id}"));
    message.id = id.to_string();
    message.reply = true;
    message
}

fn journal_detail_fixture(label: &str, with_activity: bool) -> (Vec<Value>, Vec<Value>) {
    let _suite = if with_activity {
        Some(crate::act_kind::pilot_kind_suite_lock())
    } else {
        None
    };
    let root = root(label);
    let trace = root.join("trace.jsonl");
    let mut environment = vec![(
        "BRIDGET_CODEX_TRACE".to_string(),
        trace.to_string_lossy().into_owned(),
    )];
    if with_activity {
        environment.push(("BRIDGET_CODEX_ACTIVITY".to_string(), "1".to_string()));
    }
    let mut transport =
        CodexAppServerTransport::spawn_with_environment(fake_options(&trace), &environment, false)
            .expect("session native de détail");
    transport
        .activate_journal(&root, "codex-native", None)
        .expect("journal de détail activé");
    transport
        .deliver(&message(label))
        .expect("livraison de détail");

    let deadline = Instant::now() + Duration::from_secs(3);
    let mut finished = false;
    while Instant::now() < deadline {
        finished |= transport.drain_events().iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::TurnFinished {
                    terminal: ManagedTerminal::Completed,
                    ..
                }
            )
        });
        if finished {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(finished, "le tour de détail n'a pas terminé");
    if with_activity {
        // Le terminal est émis avant que le faux fournisseur ne reprenne
        // sa boucle stdin. Sa trace, et non la fin du tour, prouve qu'il
        // a effectivement reçu les décisions JSON-RPC avant l'arrêt.
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let frames: Vec<Value> = fs::read_to_string(&trace)
                .unwrap_or_default()
                .lines()
                .filter_map(|line| serde_json::from_str(line).ok())
                .collect();
            let received = ["approval-7", "approval-8", "approval-9"].iter().all(|id| {
                frames.iter().any(|frame| {
                    frame["id"].as_str() == Some(*id)
                        && frame["result"]["decision"].as_str() == Some("accept")
                })
            });
            if received {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "le fournisseur n'a pas reçu les trois décisions: {frames:?}"
            );
            thread::sleep(Duration::from_millis(5));
        }
    }
    transport.stop();

    let journal_path = fs::read_dir(root.join("codex-native"))
        .expect("répertoire du journal")
        .next()
        .expect("fichier du journal")
        .expect("entrée du journal")
        .path();
    let events = crate::journal::valid_events(&journal_path);
    let frames = fs::read_to_string(&trace)
        .expect("trace fournisseur")
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .collect();
    fs::remove_dir_all(root).expect("nettoyage de la fixture");
    (events, frames)
}

fn journal_text_fixture(label: &str, text_mode: &str) -> Vec<Value> {
    let root = root(label);
    let trace = root.join("trace.jsonl");
    let environment = vec![
        (
            "BRIDGET_CODEX_TRACE".to_string(),
            trace.to_string_lossy().into_owned(),
        ),
        ("BRIDGET_CODEX_TEXT_MODE".to_string(), text_mode.to_string()),
    ];
    let mut transport =
        CodexAppServerTransport::spawn_with_environment(fake_options(&trace), &environment, false)
            .expect("session native texte");
    transport
        .activate_journal(&root, "codex-native", None)
        .expect("journal texte activé");
    let mut msg = message(label);
    msg.id = "codex-abc-1".to_string();
    transport.deliver(&msg).expect("livraison texte");

    let deadline = Instant::now() + Duration::from_secs(3);
    let mut finished = false;
    while Instant::now() < deadline {
        finished |= transport.drain_events().iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::TurnFinished {
                    terminal: ManagedTerminal::Completed,
                    ..
                }
            )
        });
        if finished {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(finished, "le tour texte Codex n'a pas terminé");
    transport.stop();

    let journal_path = fs::read_dir(root.join("codex-native"))
        .expect("répertoire du journal")
        .next()
        .expect("fichier du journal")
        .expect("entrée du journal")
        .path();
    let events = crate::journal::valid_events(&journal_path);
    fs::remove_dir_all(root).expect("nettoyage de la fixture texte");
    events
}

fn assert_whitelist_events_exclude(text: &str, forbidden: &str) {
    for line in text.lines().filter(|line| !line.is_empty()) {
        let Ok(entry) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if entry["event"] == "provider_request" {
            let payload = serde_json::to_string(&entry["payload"]).expect("payload sérialisable");
            assert!(
                !payload.contains(forbidden),
                "provider_request ne doit pas porter {forbidden:?}: {payload}"
            );
        }
        if entry["event"] == "error"
            && let Some(pending) = entry.pointer("/payload/pending_provider_request")
        {
            let pending = serde_json::to_string(pending).expect("pending sérialisable");
            assert!(
                !pending.contains(forbidden),
                "pending_provider_request ne doit pas porter {forbidden:?}: {pending}"
            );
        }
    }
}

fn assert_no_provider_controls(text: &str) {
    for forbidden in ["\u{1b}", "\r", "\u{202e}", "\\u001b", "\\r", "\\u202e"] {
        assert!(
            !text.contains(forbidden),
            "contrôle fournisseur durable interdit: {forbidden:?}"
        );
    }
}

fn assert_fingerprint(value: &str) {
    assert_eq!(value.len(), "sha256:".len() + 64);
    assert!(value.starts_with("sha256:"));
    assert!(
        value["sha256:".len()..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()),
        "empreinte hexadécimale minuscule attendue"
    );
}

fn assert_provider_projection_redacts(variant: &str, label: &str, sentinel: &str) {
    let root = root(label);
    let trace = root.join("trace.jsonl");
    let mut options = fake_options(&trace);
    options.notify_timeout_secs = 1;
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[
            (
                "BRIDGET_CODEX_TRACE".to_string(),
                trace.to_string_lossy().into_owned(),
            ),
            (
                "BRIDGET_CODEX_REQUEST_APPROVAL".to_string(),
                "1".to_string(),
            ),
            (
                "BRIDGET_CODEX_REQUEST_VARIANT".to_string(),
                variant.to_string(),
            ),
            ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
        ],
        false,
    )
    .expect("session native");
    transport
        .activate_journal(&root, "codex-native", None)
        .expect("journal activé");
    let message_id = format!("provider-{label}-019");
    transport.deliver(&message(&message_id)).expect("livraison");

    let deadline = Instant::now() + Duration::from_secs(3);
    let mut events = Vec::new();
    while Instant::now() < deadline {
        events.extend(transport.drain_events());
        if events.iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
                    if message_id == &format!("provider-{label}-019")
                        && reason == "échéance Codex dépassée"
            )
        }) {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        events.iter().any(|event| {
            event
                .raw
                .windows(sentinel.len())
                .any(|window| window == sentinel.as_bytes())
        }),
        "sentinelle absente de la trame fournisseur effectivement lue"
    );
    if variant == "method" {
        let detail = events
            .iter()
            .find_map(|event| match &event.kind {
                ManagedEventKind::Update { detail }
                    if event
                        .raw
                        .windows(sentinel.len())
                        .any(|window| window == sentinel.as_bytes()) =>
                {
                    Some(detail.as_str())
                }
                _ => None,
            })
            .expect("détail opérateur de la méthode inconnue");
        assert!(!detail.contains(sentinel));
        assert_no_provider_controls(detail);
        assert!(detail.contains("sha256:"));
    }

    transport.stop();
    let journal_path = fs::read_dir(root.join("codex-native"))
        .expect("répertoire journal")
        .next()
        .expect("fichier journal")
        .expect("entrée journal")
        .path();
    let entries = crate::journal::valid_events(&journal_path);
    let request = entries
        .iter()
        .find(|entry| entry["event"] == "provider_request")
        .expect("requête fournisseur durable avant l'échéance");
    let error = entries
        .iter()
        .find(|entry| entry["event"] == "error")
        .expect("échéance durable");
    for field in ["method", "request_id", "turn_id"] {
        assert!(
            request["payload"][field]
                .as_str()
                .is_some_and(|value| !value.is_empty()),
            "corrélation {field} non vide"
        );
    }
    assert_fingerprint(
        request["payload"]["request_id"]
            .as_str()
            .expect("empreinte de requête"),
    );
    assert_fingerprint(
        request["payload"]["turn_id"]
            .as_str()
            .expect("empreinte de tour"),
    );
    match variant {
        "method" => assert_fingerprint(
            request["payload"]["method"]
                .as_str()
                .expect("empreinte de méthode"),
        ),
        _ => assert_eq!(
            request["payload"]["method"],
            COMMAND_EXECUTION_APPROVAL_METHOD
        ),
    }
    assert_eq!(
        error["payload"]["pending_provider_request"], request["payload"],
        "projection stable entre la requête et sa borne terminale"
    );
    let journal = fs::read_to_string(&journal_path).expect("journal brut");
    assert!(
        !journal.contains(sentinel),
        "chaîne fournisseur libre persistée via {variant}"
    );
    assert_no_provider_controls(&journal);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_019_identifiant_json_rpc_invalide_ne_recopie_pas_son_contenu() {
    let identifier = project_request_id(&json!({
        "secret": "SENTINELLE-ID-019"
    }));
    assert_fingerprint(&identifier);
    assert!(!identifier.contains("SENTINELLE-ID-019"));
}

#[test]
fn test_019_empreintes_sont_stables_et_separees_par_domaine() {
    let method = provider_fingerprint(b"method", b"meme-valeur");
    let request = provider_fingerprint(b"request-id:string", b"meme-valeur");
    let turn = provider_fingerprint(b"turn-id", b"meme-valeur");
    assert_fingerprint(&method);
    assert_fingerprint(&request);
    assert_fingerprint(&turn);
    assert_eq!(method, provider_fingerprint(b"method", b"meme-valeur"));
    assert_ne!(method, request);
    assert_ne!(method, turn);
    assert_ne!(request, turn);
    assert_eq!(
        project_provider_method(COMMAND_EXECUTION_APPROVAL_METHOD),
        COMMAND_EXECUTION_APPROVAL_METHOD
    );
}

#[test]
fn test_019_methode_libre_est_opaque_et_sans_controle() {
    assert_provider_projection_redacts("method", "method-redaction", "SENTINELLE-METHOD-019");
}

#[test]
fn test_019_identifiant_chaine_valide_est_opaque() {
    assert_provider_projection_redacts(
        "request_id",
        "request-id-redaction",
        "SENTINELLE-REQUEST-ID-019",
    );
}

#[test]
fn test_019_identifiant_de_tour_est_opaque() {
    assert_provider_projection_redacts("turn_id", "turn-id-redaction", "SENTINELLE-TURN-ID-019");
}

#[test]
fn test_019_raison_fournisseur_est_opaque_et_sans_controle() {
    let root = root("provider-reason-redaction");
    let trace = root.join("trace.jsonl");
    let output_trace = root.join("provider-output.jsonl");
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        fake_options(&trace),
        &[
            (
                "BRIDGET_CODEX_TRACE".to_string(),
                trace.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_TURN_ERROR".to_string(), "1".to_string()),
            (
                "BRIDGET_CODEX_OUTPUT_TRACE".to_string(),
                output_trace.to_string_lossy().into_owned(),
            ),
        ],
        false,
    )
    .expect("session native");
    transport
        .activate_journal(&root, "codex-native", None)
        .expect("journal activé");
    transport
        .deliver(&message("provider-reason-019"))
        .expect("livraison");

    let deadline = Instant::now() + Duration::from_secs(3);
    let mut events = Vec::new();
    while Instant::now() < deadline {
        events.extend(transport.drain_events());
        if events.iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
                    if message_id == "provider-reason-019" && reason.contains("-32098")
            )
        }) {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    let rejected_reason = events
        .iter()
        .find_map(|event| match &event.kind {
            ManagedEventKind::DeliveryRejected { message_id, reason }
                if message_id == "provider-reason-019" && reason.contains("-32098") =>
            {
                Some(reason.as_str())
            }
            _ => None,
        })
        .expect("la réponse d'erreur fournisseur n'a pas été lue");
    assert!(!rejected_reason.contains("SENTINELLE-RAISON-019"));
    assert_no_provider_controls(rejected_reason);
    assert!(rejected_reason.contains("sha256:"));
    let emitted = fs::read_to_string(&output_trace).expect("sortie fournisseur attestée");
    assert!(emitted.contains("SENTINELLE-RAISON-019"));

    transport.stop();
    let journal_path = fs::read_dir(root.join("codex-native"))
        .expect("répertoire journal")
        .next()
        .expect("fichier journal")
        .expect("entrée journal")
        .path();
    let entries = crate::journal::valid_events(&journal_path);
    let error = entries
        .iter()
        .find(|entry| entry["event"] == "error")
        .expect("erreur fournisseur durable");
    assert!(
        error["payload"]["reason"]
            .as_str()
            .is_some_and(|reason| !reason.is_empty()),
        "raison durable non vide"
    );
    assert!(
        error["payload"]["reason"]
            .as_str()
            .is_some_and(|reason| reason.contains("sha256:")),
        "référence opaque de l'erreur fournisseur"
    );
    let journal = fs::read_to_string(&journal_path).expect("journal brut");
    assert!(
        !journal.contains("SENTINELLE-RAISON-019"),
        "raison fournisseur libre persistée"
    );
    assert_no_provider_controls(&journal);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_019_saturation_du_journal_devient_un_echec_observable() {
    let root = root("provider-request-journal-full");
    let trace = root.join("trace.jsonl");
    let mut options = fake_options(&trace);
    options.notify_timeout_secs = 1;
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[
            (
                "BRIDGET_CODEX_TRACE".to_string(),
                trace.to_string_lossy().into_owned(),
            ),
            (
                "BRIDGET_CODEX_REQUEST_APPROVAL".to_string(),
                "1".to_string(),
            ),
            ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
        ],
        false,
    )
    .expect("session native");
    *transport.journal.lock().recover_poison() = Some(JournalWriter::saturated_for_test());
    transport
        .deliver(&message("provider-journal-full-019"))
        .expect("livraison");

    let deadline = Instant::now() + Duration::from_secs(3);
    let mut events = Vec::new();
    while Instant::now() < deadline {
        events.extend(transport.drain_events());
        if events.iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::JournalFailed { ref detail }
                    if detail.contains("requête fournisseur impossible")
            )
        }) {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    let failure = events
        .iter()
        .find(|event| matches!(event.kind, ManagedEventKind::JournalFailed { .. }))
        .expect("échec de journalisation observable");
    assert!(matches!(
        failure.kind,
        ManagedEventKind::JournalFailed { ref detail }
            if detail.contains("journal ACP saturé")
                || detail.contains("requête fournisseur impossible")
    ));
    if matches!(
        failure.kind,
        ManagedEventKind::JournalFailed { ref detail }
            if detail.contains("requête fournisseur impossible")
    ) {
        assert!(
            failure
                .raw
                .windows(b"item/commandExecution/requestApproval".len())
                .any(|window| window == b"item/commandExecution/requestApproval")
        );
    }

    transport.stop();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_019_requete_fournisseur_est_tracee_avant_echeance_sans_payload() {
    let root = root("provider-request");
    let trace = root.join("trace.jsonl");
    let mut options = fake_options(&trace);
    options.notify_timeout_secs = 1;
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[
            (
                "BRIDGET_CODEX_TRACE".to_string(),
                trace.to_string_lossy().into_owned(),
            ),
            (
                "BRIDGET_CODEX_REQUEST_APPROVAL".to_string(),
                "1".to_string(),
            ),
            ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
        ],
        false,
    )
    .expect("session native");
    transport
        .activate_journal(&root, "codex-native", None)
        .expect("journal activé");
    transport
        .deliver(&message("provider-request-019"))
        .expect("livraison");

    let deadline = Instant::now() + Duration::from_secs(3);
    let mut events = Vec::new();
    while Instant::now() < deadline {
        events.extend(transport.drain_events());
        if events.iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
                    if message_id == "provider-request-019"
                        && reason == "échéance Codex dépassée"
            )
        }) {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(events.iter().any(|event| matches!(
        event.kind,
        ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
            if message_id == "provider-request-019"
                && reason == "échéance Codex dépassée"
    )));

    let provider_frame = events
        .iter()
        .find(|event| {
            event
                .raw
                .windows(b"item/commandExecution/requestApproval".len())
                .any(|window| window == b"item/commandExecution/requestApproval")
        })
        .expect("requête effectivement lue depuis stdout du faux pilote");
    let provider_frame = std::str::from_utf8(&provider_frame.raw).expect("trame fournisseur UTF-8");
    assert!(provider_frame.contains("SENTINELLE-SECRETE-019"));

    transport.stop();
    let journal_path = fs::read_dir(root.join("codex-native"))
        .expect("répertoire journal")
        .next()
        .expect("fichier journal")
        .expect("entrée journal")
        .path();
    let entries = crate::journal::valid_events(&journal_path);
    let request = entries
        .iter()
        .find(|entry| entry["event"] == "provider_request")
        .expect("requête fournisseur durable avant l'échéance");
    assert_eq!(request["message_id"], "provider-request-019");
    assert_eq!(
        request["payload"],
        json!({
            "provider": "codex",
            "method": "item/commandExecution/requestApproval",
            "request_id": provider_fingerprint(b"request-id:string", b"approval-native"),
            "turn_id": provider_fingerprint(b"turn-id", b"turn-native"),
            "state": "pending"
        })
    );
    let journal = fs::read_to_string(&journal_path).expect("journal brut");
    // Compromis explicite avec la visibilité GUI (spec 032) : la sentinelle
    // peut apparaître dans un acte `approval` (command/reason), jamais dans
    // provider_request ni pending_provider_request (liste blanche FR-1903).
    assert_whitelist_events_exclude(&journal, "SENTINELLE-SECRETE-019");
    let request_seq = request["seq"].as_u64().expect("séquence requête");
    let error = entries
        .iter()
        .find(|entry| entry["event"] == "error")
        .expect("échéance durable");
    assert!(request_seq < error["seq"].as_u64().expect("séquence erreur"));
    assert_eq!(error["message_id"], request["message_id"]);
    assert_eq!(
        error["payload"]["pending_provider_request"],
        request["payload"]
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_019_eof_reste_correle_a_la_derniere_requete_fournisseur() {
    let root = root("provider-request-eof");
    let trace = root.join("trace.jsonl");
    let mut options = fake_options(&trace);
    options.notify_timeout_secs = 60;
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[
            (
                "BRIDGET_CODEX_TRACE".to_string(),
                trace.to_string_lossy().into_owned(),
            ),
            (
                "BRIDGET_CODEX_REQUEST_APPROVAL".to_string(),
                "1".to_string(),
            ),
            (
                "BRIDGET_CODEX_EXIT_AFTER_REQUEST".to_string(),
                "1".to_string(),
            ),
        ],
        false,
    )
    .expect("session native");
    transport
        .activate_journal(&root, "codex-native", None)
        .expect("journal activé");
    transport
        .deliver(&message("provider-eof-019"))
        .expect("livraison");

    let deadline = Instant::now() + Duration::from_secs(3);
    let mut events = Vec::new();
    while Instant::now() < deadline {
        events.extend(transport.drain_events());
        if events.iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
                    if message_id == "provider-eof-019"
                        && reason == "stdout Codex fermé pendant le tour"
            )
        }) {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(events.iter().any(|event| matches!(
        event.kind,
        ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
            if message_id == "provider-eof-019"
                && reason == "stdout Codex fermé pendant le tour"
    )));
    assert!(events.iter().any(|event| {
        event
            .raw
            .windows(b"item/commandExecution/requestApproval".len())
            .any(|window| window == b"item/commandExecution/requestApproval")
    }));

    transport.stop();
    let journal_path = fs::read_dir(root.join("codex-native"))
        .expect("répertoire journal")
        .next()
        .expect("fichier journal")
        .expect("entrée journal")
        .path();
    let entries = crate::journal::valid_events(&journal_path);
    let request = entries
        .iter()
        .find(|entry| entry["event"] == "provider_request")
        .expect("requête fournisseur durable avant EOF");
    let error = entries
        .iter()
        .find(|entry| entry["event"] == "error")
        .expect("EOF durable");
    assert!(
        request["seq"].as_u64().expect("séquence requête")
            < error["seq"].as_u64().expect("séquence erreur")
    );
    assert_eq!(error["message_id"], request["message_id"]);
    assert_eq!(
        error["payload"]["pending_provider_request"],
        request["payload"]
    );

    let _ = fs::remove_dir_all(root);
}

/// (A) Deltas → update avec TEXTE EXACT. Mutant content:"" doit tuer A seul.
#[allow(non_snake_case)]
#[test]
fn TEMOIN_A_codex_app_server_retranscrit_les_deltas_en_update() {
    let events = journal_text_fixture("temoin-a", "deltas");
    let updates: Vec<_> = events
        .iter()
        .filter(|event| event["event"] == "update" && event["payload"]["kind"] == "text")
        .collect();
    let contents: Vec<&str> = updates
        .iter()
        .filter_map(|event| event["payload"]["content"].as_str())
        .collect();
    assert_eq!(
        contents,
        vec!["BONJOUR ", "JE SUIS VIVANT"],
        "deux deltas exacts sans doublon item/completed: {contents:?} via {events:?}"
    );
    assert!(
        updates
            .iter()
            .all(|event| event["message_id"] == "codex-abc-1"),
        "même message_id, reçu {updates:?}"
    );
}

/// (B) Tour sans delta : item/completed agentMessage → texte final exact.
/// Mutant : retirer maybe_record_final_agent_message → left=[].
#[allow(non_snake_case)]
#[test]
fn TEMOIN_B_codex_app_server_reponse_finale_sans_delta_dans_le_fil() {
    let events = journal_text_fixture("temoin-b", "final_only");
    let updates: Vec<_> = events
        .iter()
        .filter(|event| event["event"] == "update" && event["payload"]["kind"] == "text")
        .collect();
    let contents: Vec<&str> = updates
        .iter()
        .filter_map(|event| event["payload"]["content"].as_str())
        .collect();
    assert_eq!(
        contents,
        vec!["REPONSE FINALE"],
        "repli item/completed → update exact, reçu {contents:?} via {events:?}"
    );
    assert!(
        updates
            .iter()
            .all(|event| event["message_id"] == "codex-abc-1"),
        "même message_id, reçu {updates:?}"
    );
}

/// (C) turn_end d'un SUCCÈS porte stop_reason=completed (+ routed_to si reply).
/// Distinct de l'échéance DeliveryRejected sans turn_end. Mutant turn_end {} → C meurt.
#[allow(non_snake_case)]
#[test]
fn TEMOIN_C_codex_app_server_turn_end_porte_stop_reason_completed() {
    let events = journal_text_fixture("temoin-c", "final_only");
    let turn_end = events
        .iter()
        .find(|event| event["event"] == "turn_end")
        .expect("turn_end attendu sur succès");
    assert_eq!(
        turn_end["payload"]["stop_reason"].as_str(),
        Some("completed"),
        "stop_reason manquant → attach « inconnu », payload={}",
        turn_end["payload"]
    );
    assert_eq!(
        turn_end["payload"]["routed_to"].as_str(),
        Some("bridget"),
        "reply=true doit porter routed_to, payload={}",
        turn_end["payload"]
    );
}

/// Point D du chantier : un message arrivé pendant un tour actif est INJECTÉ
/// dans ce tour par `turn/steer`, sans l'interrompre et sans attendre sa fin.
///
/// Assertion métier : la requête porte `expectedTurnId` = le tour ACTIF
/// (`turn-native`), et non l'identifiant du message ni un tour mis en file
/// (contrainte t3code CodexSessionRuntime.ts:1853).
///
/// Mutant qui le tue : router le second message vers `messages` au lieu de
/// `steer` dans `deliver` — aucun `turn/steer` n'est alors émis, le message
/// attend la fin du tour, et c'est exactement le défaut que le chantier
/// demande de corriger.
#[allow(non_snake_case)]
#[test]
fn TEMOIN_E_codex_message_arrive_est_pilote_dans_le_tour_en_cours() {
    let root = root("temoin-steer");
    let trace = root.join("trace.jsonl");
    let mut options = fake_options(&trace);
    options.notify_timeout_secs = 6;
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[
            (
                "BRIDGET_CODEX_TRACE".to_string(),
                trace.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
        ],
        false,
    )
    .expect("session native pilotage");

    let mut premier = message("temoin-steer");
    premier.id = "codex-steer-tour".to_string();
    transport.deliver(&premier).expect("livraison du premier");

    // Attendre que le tour soit RÉELLEMENT actif : piloter avant que
    // `turn/start` ait rendu son identifiant n'aurait aucun sens.
    let attente = Instant::now() + Duration::from_secs(5);
    while Instant::now() < attente {
        if fs::read_to_string(&trace)
            .unwrap_or_default()
            .contains("\"method\":\"turn/start\"")
        {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    thread::sleep(Duration::from_millis(150));

    let mut second = message("temoin-steer-2");
    second.id = "codex-steer-injecte".to_string();
    second.from = "superviseur".to_string();
    second.intent = Some(MessageIntent::SteerCurrent);
    transport.deliver(&second).expect("livraison du second");

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut ligne_steer = None;
    while Instant::now() < deadline {
        let contenu = fs::read_to_string(&trace).unwrap_or_default();
        if let Some(ligne) = contenu
            .lines()
            .find(|ligne| ligne.contains("\"method\":\"turn/steer\""))
        {
            ligne_steer = Some(ligne.to_string());
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    transport.stop();

    let ligne_steer = ligne_steer.unwrap_or_else(|| {
        panic!(
            "aucun turn/steer émis : le message a attendu la fin du tour; trace={}",
            fs::read_to_string(&trace).unwrap_or_else(|error| error.to_string())
        )
    });
    assert!(
        ligne_steer.contains("\"expectedTurnId\":\"turn-native\""),
        "turn/steer doit viser le tour ACTIF, ligne={ligne_steer}"
    );
    assert!(
        ligne_steer.contains("\"clientUserMessageId\":\"codex-steer-injecte\""),
        "turn/steer doit porter le message arrivé, ligne={ligne_steer}"
    );
    let steer: Value = serde_json::from_str(&ligne_steer).unwrap();
    let prompt = steer["params"]["input"][0]["text"].as_str().unwrap();
    let header = prompt.lines().next().unwrap();
    let envelope: Value = serde_json::from_str(
        header
            .strip_prefix("[Message Bridget : ")
            .unwrap()
            .strip_suffix(']')
            .unwrap(),
    )
    .unwrap();
    assert_eq!(envelope["from"], second.from);
    assert_eq!(envelope["id"], second.id);
    assert_eq!(envelope["reply"], second.reply);
    assert!(prompt.contains(&second.body));
    let _ = fs::remove_dir_all(root);
}

/// Un tour NON PILOTABLE (`activeTurnNotSteerable`, mesuré le 28/08 contre
/// codex-cli 0.150.1) doit rendre le message à la file PRINCIPALE, pour
/// qu'il soit traité au tour suivant — et non le rejouer en boucle dans la
/// file de pilotage, ce qui bloquerait tout ce qui suit.
///
/// Assertion métier : après l'échéance du premier tour, un SECOND
/// `turn/start` est émis — preuve que le message refusé est reparti en mode
/// « queue ». Mutant qui le tue : remplacer `state.messages.push_front` par
/// `state.steer.push_front` dans la branche non pilotable.
#[allow(non_snake_case)]
#[test]
fn TEMOIN_F_codex_tour_non_pilotable_rend_le_message_a_la_file() {
    let root = root("temoin-non-pilotable");
    let trace = root.join("trace.jsonl");
    let mut options = fake_options(&trace);
    options.notify_timeout_secs = 3;
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[
            (
                "BRIDGET_CODEX_TRACE".to_string(),
                trace.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
            ("BRIDGET_CODEX_STEER_REFUSE".to_string(), "1".to_string()),
        ],
        false,
    )
    .expect("session native non pilotable");

    let mut premier = message("np-1");
    premier.id = "codex-np-tour".to_string();
    transport.deliver(&premier).expect("livraison du premier");

    let attente = Instant::now() + Duration::from_secs(5);
    while Instant::now() < attente {
        if fs::read_to_string(&trace)
            .unwrap_or_default()
            .contains("\"method\":\"turn/start\"")
        {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    thread::sleep(Duration::from_millis(150));

    let mut second = message("np-2");
    second.id = "codex-np-refuse".to_string();
    second.from = "humain".to_string();
    second.intent = Some(MessageIntent::SteerCurrent);
    transport.deliver(&second).expect("livraison du second");

    // Le premier tour tombe sur son échéance (HOLD_TURN), puis le worker
    // doit reprendre le message rendu à la file principale.
    let deadline = Instant::now() + Duration::from_secs(14);
    let mut deux_turn_start = false;
    while Instant::now() < deadline {
        let contenu = fs::read_to_string(&trace).unwrap_or_default();
        let repris = contenu.lines().any(|ligne| {
            ligne.contains("\"method\":\"turn/start\"") && ligne.contains("codex-np-refuse")
        });
        if repris {
            deux_turn_start = true;
            break;
        }
        thread::sleep(Duration::from_millis(40));
    }
    transport.stop();

    let contenu = fs::read_to_string(&trace).unwrap_or_default();
    assert!(
        contenu.contains("\"method\":\"turn/steer\""),
        "le second message devait d'abord être tenté en pilotage; trace={contenu}"
    );
    assert!(
        deux_turn_start,
        "un tour non pilotable doit rendre le message à la file : \
             aucun second turn/start observé, le message est resté bloqué en \
             pilotage; trace={contenu}"
    );
    let _ = fs::remove_dir_all(root);
}

/// Un message humain reçu pendant un tour actif doit interrompre ce tour et
/// démarrer le sien. Il ne doit jamais être injecté via `turn/steer`, car
/// Codex rattache alors sa réponse au message précédent.
#[allow(non_snake_case)]
#[test]
fn TEMOIN_G_codex_message_humain_interrompt_et_recoit_un_tour_dedie() {
    let root = root("temoin-steer-sans-consommation");
    let trace = root.join("trace.jsonl");
    let mut options = fake_options(&trace);
    options.notify_timeout_secs = 60;
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[
            (
                "BRIDGET_CODEX_TRACE".to_string(),
                trace.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
        ],
        false,
    )
    .expect("session native interruption humaine");

    let mut ancien = message("ancien-bloque");
    ancien.id = "codex-ancien-bloque".to_string();
    transport
        .deliver(&ancien)
        .expect("livraison du tour ancien");

    let attente = Instant::now() + Duration::from_secs(5);
    while Instant::now() < attente {
        if fs::read_to_string(&trace)
            .unwrap_or_default()
            .contains("\"method\":\"turn/start\"")
        {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    let mut humain = message("humain-prioritaire");
    humain.id = "codex-humain-prioritaire".to_string();
    humain.from = "humain".to_string();
    humain.origin = Some(MessageOrigin::Human);
    humain.deadline_at = Some(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("horloge")
            .as_secs()
            + 1,
    );
    transport.deliver(&humain).expect("livraison humaine");

    let mut systeme = message("systeme-apres-humain");
    systeme.id = "codex-systeme-apres-humain".to_string();
    transport.deliver(&systeme).expect("livraison système");

    let deadline = Instant::now() + Duration::from_secs(6);
    let mut humain_repris = false;
    while Instant::now() < deadline {
        let contenu = fs::read_to_string(&trace).unwrap_or_default();
        humain_repris = contenu.lines().any(|ligne| {
            ligne.contains("\"method\":\"turn/start\"")
                && ligne.contains("codex-humain-prioritaire")
        });
        if humain_repris {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    transport.stop();

    let contenu = fs::read_to_string(&trace).unwrap_or_default();
    let position_humain = contenu
        .find("codex-humain-prioritaire")
        .expect("turn/start humain attendu");
    let position_systeme = contenu.find("codex-systeme-apres-humain");
    assert!(
        contenu.contains("\"method\":\"turn/interrupt\""),
        "le message humain doit interrompre le tour ancien; trace={contenu}"
    );
    assert!(
        humain_repris,
        "le message humain doit démarrer son propre tour avant le système; trace={contenu}"
    );
    assert!(
        position_systeme.is_none_or(|position| position_humain < position),
        "un message système ne doit pas doubler l'humain restitué; trace={contenu}"
    );
    let _ = fs::remove_dir_all(root);
}

/// Un résultat JSON-RPC `turn/steer` ne solde jamais une remise humaine.
/// Le faux serveur garde le tour ouvert et n'émet pas `userMessage` :
/// aucun `PromptDispatched` humain ne doit sortir avant preuve contraire.
#[allow(non_snake_case)]
#[test]
fn TEMOIN_H_codex_steer_ack_sans_consommation_ne_solde_pas_la_remise() {
    let root = root("temoin-steer-ack-sans-consommation");
    let trace = root.join("trace.jsonl");
    let mut options = fake_options(&trace);
    options.notify_timeout_secs = 8;
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[
            (
                "BRIDGET_CODEX_TRACE".to_string(),
                trace.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
        ],
        false,
    )
    .expect("session native ack sans consommation");

    let mut ancien = message("ack-sans-consommation-ancien");
    ancien.id = "codex-ack-sans-consommation-ancien".to_string();
    transport
        .deliver(&ancien)
        .expect("livraison du tour ancien");
    let attente = Instant::now() + Duration::from_secs(5);
    while Instant::now() < attente {
        if fs::read_to_string(&trace)
            .unwrap_or_default()
            .contains("\"method\":\"turn/start\"")
        {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    let ready = Instant::now() + Duration::from_secs(2);
    while !transport.queue.0.lock().unwrap().steering_open {
        assert!(Instant::now() < ready, "le tour n'a pas ouvert le guidage");
        thread::sleep(Duration::from_millis(5));
    }
    // Même un nom humain et une échéance fournie ne valident pas
    // l'origine : ce steer agent ne doit jamais interrompre le tour.
    let mut agent = message("deadline-agent");
    agent.id = "codex-deadline-agent".into();
    agent.from = "humain".into();
    agent.intent = Some(MessageIntent::SteerCurrent);
    agent.deadline_at = Some(0);
    transport
        .deliver(&agent)
        .expect("steer agent avec échéance fournie");
    let deadline = Instant::now() + Duration::from_millis(500);
    let mut steer_agent = false;
    while Instant::now() < deadline {
        let contenu = fs::read_to_string(&trace).unwrap_or_default();
        steer_agent |= contenu.lines().any(|line| {
            line.contains("\"method\":\"turn/steer\"") && line.contains("codex-deadline-agent")
        });
        assert!(
            !contenu.contains("\"method\":\"turn/interrupt\""),
            "une échéance de steer agent ne doit pas interrompre le tour: {contenu}"
        );
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        steer_agent,
        "le steer agent n'a pas atteint le faux fournisseur"
    );

    let mut humain = message("ack-sans-consommation-humain");
    humain.id = "codex-ack-sans-consommation-humain".to_string();
    humain.from = "humain".to_string();
    humain.origin = Some(MessageOrigin::Human);
    humain.intent = Some(MessageIntent::SteerCurrent);
    humain.deadline_at = Some(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("horloge")
            .as_secs()
            + 2,
    );
    transport.deliver(&humain).expect("livraison humaine");

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut steer_observe = false;
    let mut prompt_humain = false;
    while Instant::now() < deadline {
        let contenu = fs::read_to_string(&trace).unwrap_or_default();
        steer_observe |= contenu.lines().any(|line| {
            line.contains("\"method\":\"turn/steer\"")
                && line.contains("codex-ack-sans-consommation-humain")
        });
        assert!(
            !contenu.contains("\"method\":\"turn/interrupt\""),
            "le steer humain explicite attend son échéance avant le repli: {contenu}"
        );
        for event in transport.drain_events() {
            prompt_humain |= matches!(
                event.kind,
                ManagedEventKind::PromptDispatched { ref message_id }
                    if message_id == "codex-ack-sans-consommation-humain"
            );
        }
        if steer_observe {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    thread::sleep(Duration::from_millis(100));
    for event in transport.drain_events() {
        prompt_humain |= matches!(
            event.kind,
            ManagedEventKind::PromptDispatched { ref message_id }
                if message_id == "codex-ack-sans-consommation-humain"
        );
    }
    let deadline = Instant::now() + Duration::from_secs(4);
    let mut fallback = false;
    while Instant::now() < deadline {
        fallback = fs::read_to_string(&trace)
            .unwrap_or_default()
            .contains("\"method\":\"turn/interrupt\"");
        if fallback {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    transport.stop();

    assert!(
        fallback,
        "le steer humain validé non consommé garde son repli sur échéance"
    );
    assert!(steer_observe, "turn/steer attendu");
    assert!(
        !prompt_humain,
        "un accusé turn/steer sans item/completed userMessage ne doit pas acquitter la remise"
    );
    let _ = fs::remove_dir_all(root);
}

/// La même remise est acquittée dès que le flux natif atteste exactement le
/// `clientUserMessageId` dans un `item/completed userMessage` du tour actif.
#[allow(non_snake_case)]
#[test]
fn TEMOIN_I_codex_user_message_corrige_acquitte_la_remise_humaine() {
    let root = root("temoin-steer-consomme");
    let trace = root.join("trace.jsonl");
    let mut options = fake_options(&trace);
    options.notify_timeout_secs = 8;
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[
            (
                "BRIDGET_CODEX_TRACE".to_string(),
                trace.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
            ("BRIDGET_CODEX_STEER_CONSUME".to_string(), "1".to_string()),
        ],
        false,
    )
    .expect("session native steering consommé");

    let mut ancien = message("consomme-ancien");
    ancien.id = "codex-consomme-ancien".to_string();
    transport
        .deliver(&ancien)
        .expect("livraison du tour ancien");
    let attente = Instant::now() + Duration::from_secs(5);
    while Instant::now() < attente {
        if fs::read_to_string(&trace)
            .unwrap_or_default()
            .contains("\"method\":\"turn/start\"")
        {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }

    let mut humain = message("consomme-humain");
    humain.id = "codex-consomme-humain".to_string();
    humain.from = "humain".to_string();
    humain.intent = Some(MessageIntent::SteerCurrent);
    humain.deadline_at = Some(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("horloge")
            .as_secs()
            + 30,
    );
    transport.deliver(&humain).expect("livraison humaine");

    let deadline = Instant::now() + Duration::from_secs(3);
    let mut prompt_humain = false;
    while Instant::now() < deadline {
        for event in transport.drain_events() {
            prompt_humain |= matches!(
                event.kind,
                ManagedEventKind::PromptDispatched { ref message_id }
                    if message_id == "codex-consomme-humain"
            );
        }
        if prompt_humain {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    transport.stop();

    assert!(
        prompt_humain,
        "la preuve userMessage corrélée devait déclencher PromptDispatched"
    );
    let _ = fs::remove_dir_all(root);
}

fn active_steer_turn_detail() -> ActiveTurnDetail {
    Arc::new(Mutex::new(Some(CodexTurnDetail {
        message_id: "tour-courant".to_string(),
        thread_id: "thread-native".to_string(),
        turn_id: Some("turn-native".to_string()),
        ..Default::default()
    })))
}

fn user_message_started(client_id: Option<&str>, turn_id: &str) -> Value {
    let mut item = json!({
        "id": "provider-item-001",
        "type": "userMessage",
        "content": [],
    });
    if let Some(client_id) = client_id {
        item["clientId"] = Value::String(client_id.to_string());
    }
    json!({
        "params": {
            "threadId": "thread-native",
            "turnId": turn_id,
            "item": item,
        },
    })
}

#[test]
fn provider_item_id_est_observe_uniquement_sur_le_tour_actif() {
    let active_detail = active_steer_turn_detail();
    let event = user_message_started(Some("message-humain-001"), "turn-native");

    observe_provider_item_id(&active_detail, &event);
    let mut late = user_message_started(Some("message-humain-001"), "tour-tardif");
    late["params"]["item"]["id"] = Value::String("provider-item-tardif".to_string());
    observe_provider_item_id(&active_detail, &late);

    let observed = active_detail.lock().recover_poison();
    assert_eq!(
        observed
            .as_ref()
            .expect("tour actif")
            .provider_item_id
            .as_deref(),
        Some("provider-item-001")
    );
}

#[test]
#[allow(non_snake_case)]
fn TEMOIN_J_codex_item_started_client_id_distingue_id_fournisseur() {
    let observations = Arc::new((Mutex::new(Observations::default()), Condvar::new()));
    let active_detail = active_steer_turn_detail();
    let event = user_message_started(Some("message-humain-001"), "turn-native");

    observe_steered_user_message_visibility(&observations, &active_detail, &event);

    let mut observed = observations.0.lock().recover_poison();
    let consumed = observed
        .consumed_steers_by_turn
        .get(&("thread-native".to_string(), "turn-native".to_string()))
        .expect("clientId visible pour le tour actif");
    assert!(
        consumed.contains("message-humain-001"),
        "la corrélation doit utiliser clientId, pas l'identifiant interne"
    );
    assert!(
        !consumed.contains("provider-item-001"),
        "l'identifiant interne fournisseur ne doit jamais acquitter Bridget"
    );
    let mut accepted = vec![message("message-humain-001")];
    let confirmed =
        consume_accepted_steers(&mut observed, "thread-native", "turn-native", &mut accepted);
    assert_eq!(confirmed.len(), 1, "une seule remise est attestée");
    assert!(
        accepted.is_empty(),
        "aucune remise distincte ne doit rester"
    );
    assert!(
        consume_accepted_steers(&mut observed, "thread-native", "turn-native", &mut accepted,)
            .is_empty(),
        "la même preuve ne peut pas produire un second acquittement"
    );
}

#[test]
#[allow(non_snake_case)]
fn TEMOIN_K_codex_item_started_refuse_mauvais_client_id_et_tour_tardif() {
    let observations = Arc::new((Mutex::new(Observations::default()), Condvar::new()));
    let active_detail = active_steer_turn_detail();
    let wrong_id = user_message_started(Some("client-inconnu"), "turn-native");
    observe_steered_user_message_visibility(&observations, &active_detail, &wrong_id);

    let mut observed = observations.0.lock().recover_poison();
    let mut accepted = vec![message("message-humain-001")];
    assert!(
        consume_accepted_steers(&mut observed, "thread-native", "turn-native", &mut accepted,)
            .is_empty(),
        "un clientId inconnu ne doit pas produire d'acquittement"
    );
    drop(observed);

    let late = user_message_started(Some("message-humain-001"), "tour-termine");
    let late_observations = Arc::new((Mutex::new(Observations::default()), Condvar::new()));
    observe_steered_user_message_visibility(&late_observations, &active_detail, &late);
    assert!(
        late_observations
            .0
            .lock()
            .recover_poison()
            .consumed_steers_by_turn
            .is_empty(),
        "un événement d'un autre tour ne doit laisser aucune preuve"
    );
}

#[test]
#[allow(non_snake_case)]
fn TEMOIN_L_codex_item_started_sans_client_id_ne_consomme_pas() {
    let observations = Arc::new((Mutex::new(Observations::default()), Condvar::new()));
    let active_detail = active_steer_turn_detail();
    let event = user_message_started(None, "turn-native");

    observe_steered_user_message_visibility(&observations, &active_detail, &event);

    assert!(
        observations
            .0
            .lock()
            .recover_poison()
            .consumed_steers_by_turn
            .is_empty(),
        "sans clientId il n'existe aucune preuve de consommation"
    );
}

fn journal_echeance_fixture(label: &str) -> Vec<Value> {
    let root = root(label);
    let trace = root.join("trace.jsonl");
    // HOLD_TURN : turn/start OK, aucun turn/completed → wait_for_turn
    // tombe sur « échéance Codex dépassée » (notify_timeout_secs=2).
    let mut options = fake_options(&trace);
    options.notify_timeout_secs = 1;
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[
            (
                "BRIDGET_CODEX_TRACE".to_string(),
                trace.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
        ],
        false,
    )
    .expect("session native échéance");
    transport
        .activate_journal(&root, "codex-native", None)
        .expect("journal échéance activé");
    let mut msg = message(label);
    msg.id = "codex-echeance-1".to_string();
    transport.deliver(&msg).expect("livraison échéance");

    let deadline = Instant::now() + Duration::from_secs(4);
    let mut rejected = false;
    while Instant::now() < deadline {
        rejected |= transport.drain_events().iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::DeliveryRejected { ref reason, .. }
                    if reason == "échéance Codex dépassée"
            )
        });
        if rejected {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        rejected,
        "DeliveryRejected échéance attendu; trace={}",
        fs::read_to_string(&trace).unwrap_or_else(|error| error.to_string())
    );
    assert!(
        fs::read_to_string(&trace)
            .unwrap_or_default()
            .contains("\"method\":\"turn/interrupt\""),
        "l'échéance fournisseur doit interrompre le tour avant le terminal"
    );
    transport.stop();

    let journal_path = fs::read_dir(root.join("codex-native"))
        .expect("répertoire du journal")
        .next()
        .expect("fichier du journal")
        .expect("entrée du journal")
        .path();
    let events = crate::journal::valid_events(&journal_path);
    fs::remove_dir_all(root).expect("nettoyage échéance");
    events
}

/// (D) Séparation verrouillée : échéance → error, AUCUN turn_end.
/// Mutant INTERDIT : écrire turn_end completed sur DeliveryRejected → D meurt
/// (A/B/C restent verts — c'est exactement le trou signalé par cursor6).
#[allow(non_snake_case)]
#[test]
fn TEMOIN_D_codex_app_server_echeance_ecrit_error_sans_turn_end() {
    let events = journal_echeance_fixture("temoin-d");
    let errors: Vec<_> = events
        .iter()
        .filter(|event| event["event"] == "error")
        .collect();
    assert_eq!(
        errors.len(),
        1,
        "une seule error d'échéance, reçu {errors:?} via {events:?}"
    );
    assert_eq!(
        errors[0]["payload"]["reason"].as_str(),
        Some("échéance Codex dépassée"),
        "raison en dur, reçu {}",
        errors[0]["payload"]
    );
    assert_eq!(
        errors[0]["payload"]["terminal_kind"].as_str(),
        Some(crate::journal::TURN_FAILED_KIND),
        "échéance DOIT porter terminal_kind=turn_failed sinon la ronde classe OCCUPE; reçu {}",
        errors[0]["payload"]
    );
    assert_eq!(
        errors[0]["message_id"].as_str(),
        Some("codex-echeance-1"),
        "même message_id, reçu {errors:?}"
    );
    let turn_ends: Vec<_> = events
        .iter()
        .filter(|event| event["event"] == "turn_end")
        .collect();
    assert!(
        turn_ends.is_empty(),
        "échéance SANS turn_end (≠ succès Claude à payload vide) ; reçu {turn_ends:?}"
    );
}

/// (TOOL) Commande Codex → update kind=command avec le NOM réel
/// (`item.command`), pas la sortie stdout. Source mesurée : item/started
/// type=commandExecution. Mutant : remettre outputDelta en texte d'acte,
/// ou vider command → meurt.
#[test]
#[allow(non_snake_case)]
fn TEMOIN_TOOL_codex_app_server_retranscrit_command_execution_nom() {
    let (present, _outbound) = journal_detail_fixture("temoin-tool-command", true);
    let commands: Vec<_> = present
        .iter()
        .filter(|event| event["event"] == "update" && event["payload"]["kind"] == "command")
        .collect();
    assert_eq!(
        commands.len(),
        1,
        "un seul acte command attendu (pas les outputDelta), reçu {commands:?}"
    );
    assert_eq!(
        commands[0]["payload"]["text"].as_str(),
        Some("echo MESURE_CODEX_CMD_77"),
        "commande en dur (pas la sortie), payload={}",
        commands[0]["payload"]
    );
    assert_eq!(
        commands[0]["payload"]["detail"].as_str(),
        Some("item/started"),
        "provenance item/started en dur, payload={}",
        commands[0]["payload"]
    );
    assert!(
        commands
            .iter()
            .all(|event| event["payload"]["text"] != "313 passés"),
        "un mutant qui rejoue outputDelta comme nom laisserait 313 passés"
    );
}

#[test]
fn journal_codex_atteste_presence_puis_absence_et_ne_valide_pas_approbation() {
    let (present, outbound) = journal_detail_fixture("detail-present", true);
    let reasoning = present
        .iter()
        .filter(|event| event["event"] == "reasoning")
        .collect::<Vec<_>>();
    assert_eq!(reasoning.len(), 1, "une seule attestation terminale");
    assert_eq!(
        reasoning[0]["payload"],
        json!({
            "available": true,
            "summary": "Je compare les options.",
            "raw": "raison brute"
        })
    );
    let reasoning_index = present
        .iter()
        .position(|event| event["event"] == "reasoning")
        .expect("attestation de présence");
    let turn_end_index = present
        .iter()
        .position(|event| event["event"] == "turn_end")
        .expect("fin de tour présente");
    assert_eq!(
        reasoning_index + 1,
        turn_end_index,
        "le raisonnement n'est attesté qu'au terminal"
    );
    for expected in [
        json!({"kind":"command", "text":"echo MESURE_CODEX_CMD_77", "detail":"item/started"}),
        json!({"kind":"file", "text":"src/main.rs", "detail":"update"}),
        json!({"kind":"plan", "text":"Tester le flux"}),
        json!({
            "kind":"approval",
            "text":"cargo test -p bridget-transport",
            "detail":"item/commandExecution/requestApproval"
        }),
        json!({
            "kind":"approval",
            "text":"écrire le fichier",
            "detail":"item/fileChange/requestApproval"
        }),
        json!({
            "kind":"approval",
            "text":"accès réseau",
            "detail":"item/permissions/requestApproval"
        }),
    ] {
        assert!(
            present
                .iter()
                .any(|event| event["event"] == "update" && event["payload"] == expected),
            "acte Codex absent du journal: {expected}"
        );
    }
    assert!(
        present.iter().all(|event| {
            !(event["event"] == "update"
                && event["payload"]["kind"] == "command"
                && event["payload"]["text"] == "313 passés")
        }),
        "outputDelta ne doit plus être journalisé comme nom de commande"
    );
    assert_eq!(
        present
            .iter()
            .filter(|event| { event["event"] == "update" && event["payload"]["kind"] != "text" })
            .count(),
        6,
        "une notification inconnue doit rester un événement système inerte"
    );
    for approval_id in ["approval-7", "approval-8", "approval-9"] {
        assert!(
            outbound.iter().any(|frame| {
                frame["id"].as_str() == Some(approval_id)
                    && frame["result"]["decision"].as_str() == Some("accept")
            }),
            "décision JSON-RPC absente pour {approval_id}: {outbound:?}"
        );
    }

    let (absent, _) = journal_detail_fixture("detail-absent", false);
    let reasoning = absent
        .iter()
        .filter(|event| event["event"] == "reasoning")
        .collect::<Vec<_>>();
    assert_eq!(
        reasoning.len(),
        1,
        "l'absence n'est attestée qu'au terminal"
    );
    assert_eq!(reasoning[0]["payload"], json!({"available": false}));
    let reasoning_index = absent
        .iter()
        .position(|event| event["event"] == "reasoning")
        .expect("attestation d'absence");
    let turn_end_index = absent
        .iter()
        .position(|event| event["event"] == "turn_end")
        .expect("fin du tour sans raisonnement");
    assert_eq!(
        reasoning_index + 1,
        turn_end_index,
        "un silence intermédiaire ne doit jamais attester l'absence"
    );
}

#[test]
fn approval_response_choisit_accept_ou_decline_selon_permissions() {
    let request = json!({
        "id": "approval-unit",
        "method": "item/commandExecution/requestApproval",
        "params": {
            "command": "git worktree add", "turnId": "t1",
            "availableDecisions": ["accept", "decline"]
        }
    });
    let (allow_reply, allow_payload) = approval_response(&request, "allow").expect("allow");
    assert_eq!(
        allow_reply,
        json!({ "id": "approval-unit", "result": { "decision": "accept" } }),
        "attente en dur: allow → accept"
    );
    assert_eq!(allow_payload["decision"], "accept");
    let (deny_reply, deny_payload) = approval_response(&request, "deny").expect("deny");
    assert_eq!(
        deny_reply,
        json!({ "id": "approval-unit", "result": { "decision": "decline" } }),
        "attente en dur: deny → decline"
    );
    assert_eq!(deny_payload["decision"], "decline");
    assert!(
        approval_response(
            &json!({ "method": "item/commandExecution/requestApproval" }),
            "allow"
        )
        .is_none(),
        "sans id: aucune réponse inventée"
    );
    for (label, permissions, choices, expected) in [
        (
            "deny sans decline",
            "deny",
            json!(["accept", "cancel"]),
            "cancel",
        ),
        (
            "allow sans accept",
            "allow",
            json!(["decline", "cancel"]),
            "decline",
        ),
        ("allow accept proposé", "allow", json!(["accept"]), "accept"),
        (
            "allow refus prioritaire",
            "allow",
            json!(["cancel", "decline"]),
            "decline",
        ),
        ("deny cancel proposé", "deny", json!(["cancel"]), "cancel"),
    ] {
        let offered = json!({
            "id": 155000,
            "method": "item/commandExecution/requestApproval",
            "params": { "availableDecisions": choices }
        });
        let (reply, payload) = approval_response(&offered, permissions).expect(label);
        assert_eq!(
            reply["id"], offered["id"],
            "{label}: identifiant repris tel quel"
        );
        assert_eq!(reply["result"]["decision"], expected, "{label}");
        assert!(
            reply.get("error").is_none(),
            "{label}: décision proposée et fermée"
        );
        assert_eq!(
            payload["decision"], expected,
            "{label}: diagnostic cohérent"
        );
        assert!(matches!(
            reply["result"]["decision"].as_str(),
            Some("accept" | "decline" | "cancel")
        ));
    }
    for (label, permissions, choices) in [
        (
            "deny acceptForSession seul",
            "deny",
            Some(json!(["acceptForSession"])),
        ),
        (
            "deny amendement seul",
            "deny",
            Some(json!(["acceptWithExecpolicyAmendment"])),
        ),
        ("deny accept seul", "deny", Some(json!(["accept"]))),
        ("offre vide", "allow", Some(json!([]))),
        ("offre absente", "allow", None),
    ] {
        let mut offered = json!({
            "id": format!("approval-155-{label}"),
            "method": "item/commandExecution/requestApproval",
            "params": {}
        });
        if let Some(choices) = choices {
            offered["params"]["availableDecisions"] = choices;
        }
        let (reply, _) = approval_response(&offered, permissions).expect(label);
        assert_eq!(
            reply["id"], offered["id"],
            "{label}: identifiant repris tel quel"
        );
        assert!(
            reply.get("result").is_none(),
            "{label}: aucun octroi inventé : {reply}"
        );
        assert_eq!(reply["error"]["code"], -32000, "{label}");
        assert_eq!(
            reply["error"]["message"], "child_request_refused",
            "{label}"
        );
    }
}

/// ORACLE PRESENCE + EFFET dans le même corps.
/// Présence : une réponse JSON-RPC portant `decision` est émise (trace non vide).
/// Effet : le tour se poursuit après cette réponse (texte + Completed).
/// Attentes écrites en dur — jamais un appel à `approval_response`.
#[test]
#[allow(non_snake_case)]
fn TEMOIN_codex_approbation_repond_et_le_tour_se_poursuit() {
    let root = root("approval-presence-effet");
    let trace = root.join("trace.jsonl");
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        fake_options(&trace),
        &[
            (
                "BRIDGET_CODEX_TRACE".to_string(),
                trace.to_string_lossy().into_owned(),
            ),
            (
                "BRIDGET_CODEX_REQUEST_APPROVAL".to_string(),
                "1".to_string(),
            ),
        ],
        false,
    )
    .expect("session native");
    transport
        .activate_journal(&root, "codex-native", None)
        .expect("journal activé");
    transport
        .deliver(&message("approval-presence-effet"))
        .expect("livraison");

    let deadline = Instant::now() + Duration::from_secs(3);
    let mut finished = false;
    let mut saw_continuation = false;
    while Instant::now() < deadline {
        for event in transport.drain_events() {
            if let ManagedEventKind::TurnFinished {
                terminal: ManagedTerminal::Completed,
                response,
                ..
            } = &event.kind
            {
                finished = true;
                saw_continuation = response.contains("tour poursuivi apres autorisation");
            }
        }
        if finished {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    transport.stop();

    let frames: Vec<Value> = fs::read_to_string(&trace)
        .expect("trace fournisseur")
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    // ORACLE PRESENCE — non-vacuité : au moins une décision émise, id repris.
    let decisions: Vec<_> = frames
        .iter()
        .filter(|frame| {
            frame["id"].as_str() == Some("approval-native")
                && frame.get("result").is_some()
                && frame.get("method").is_none()
        })
        .collect();
    assert!(
        !decisions.is_empty(),
        "PRESENCE: aucune réponse d'approbation dans la trace (vide)"
    );
    assert_eq!(
        decisions[0]["result"]["decision"].as_str(),
        Some("accept"),
        "PRESENCE: décision attendue en dur accept, reçu {}",
        decisions[0]
    );

    let journal_path = fs::read_dir(root.join("codex-native"))
        .expect("répertoire journal")
        .next()
        .expect("fichier journal")
        .expect("entrée journal")
        .path();
    let entries = crate::journal::valid_events(&journal_path);
    let permission = entries
        .iter()
        .find(|entry| entry["event"] == "permission")
        .expect("PRESENCE: événement permission journalisé");
    assert_eq!(
        permission["payload"]["decision"].as_str(),
        Some("accept"),
        "PRESENCE journal: décision en dur accept"
    );
    assert_eq!(
        permission["message_id"].as_str(),
        Some("approval-presence-effet"),
        "PRESENCE: message_id non vide"
    );

    // ORACLE EFFET — le tour continue après la réponse (pas une garde morte).
    assert!(
        finished,
        "EFFET: le tour doit atteindre Completed après la décision"
    );
    assert!(
        saw_continuation,
        "EFFET: texte de poursuite absent — le faux fournisseur n'a pas repris après décision"
    );
    let decision_pos = frames
        .iter()
        .position(|frame| {
            frame["id"].as_str() == Some("approval-native")
                && frame["result"]["decision"].as_str() == Some("accept")
        })
        .expect("position de la décision");
    // La poursuite est côté stdout du faux (pas dans TRACE stdin). On vérifie
    // l'ordre journal : permission avant turn_end.
    let permission_seq = permission["seq"].as_u64().expect("seq permission");
    let turn_end = entries
        .iter()
        .find(|entry| entry["event"] == "turn_end")
        .expect("EFFET: turn_end après poursuite");
    assert!(
        permission_seq < turn_end["seq"].as_u64().expect("seq turn_end"),
        "EFFET: permission doit précéder turn_end (décision puis poursuite)"
    );
    assert!(
        decision_pos < frames.len(),
        "PRESENCE: index de décision dans un univers non vide"
    );

    let _ = fs::remove_dir_all(root);
}

/// T012 : présence réelle, absence de réponse automatique, puis nettoyage.
/// La fixture réutilise le serveur149. Son déclencheur ne part qu'après
/// l'observation du tour, sans course avec la réponse de turn/start.
#[test]
fn native155_transport_command_forward_borne_et_fin_tour() {
    use crate::protocol::{ChildProviderRequestId, ChildRequestAnswer, ChildRequestDecision};
    let root = root("155-command-forward");
    let trace = root.join("trace.jsonl");
    let requests_file = root.join("requests.jsonl");
    let stall_ready = root.join("stdin-stalled");
    let stall_release = root.join("emit-terminal");
    let requests: Vec<Value> = (0..5)
        .map(|index| {
            json!({
                "id": if index == 0 { json!(i64::MIN) } else { json!(format!("cmd-{index}")) },
                "method":"item/commandExecution/requestApproval",
                "params":{"threadId":"thread-native","turnId":"turn-native",
                    "itemId":format!("item-{index}"),"startedAtMs":1787686800000i64,
                    "kind":"command","environmentId":"local",
                    "command":"printf couleur","cwd":"/tmp",
                    "commandActions":[{"type":"unknown","command":"printf couleur"}],
                    "availableDecisions":["accept","cancel"]}
            })
        })
        .collect();
    // R2 : le cwd reste une citation exacte, contrôles et guillemets
    // compris. Il ne transforme jamais la commande en effet confiné.
    for cwd in [
        json!("/source/non-verifie"),
        json!("/source/\"citation\"\n\u{1b}[2J\r\u{202e}"),
        Value::Null,
    ] {
        let mut source = requests[0].clone();
        source["params"]["cwd"] = cwd.clone();
        let normalized = normalize_child_provider_request(&source, "session-source155")
            .unwrap()
            .0;
        assert_eq!(normalized.details.get("cwd"), Some(&cwd));
        assert_eq!(normalized.details["effects"]["command"], true);
    }
    let mut absent = requests[0].clone();
    absent["params"].as_object_mut().unwrap().remove("cwd");
    assert_eq!(
        normalize_child_provider_request(&absent, "session-source155")
            .unwrap()
            .0
            .details
            .get("cwd"),
        Some(&Value::Null)
    );
    let mut too_long = requests[0].clone();
    let cwd = format!("/{}", "é".repeat(512));
    assert_eq!(cwd.len(), 1025, "borne en octets UTF-8, pas en caractères");
    too_long["params"]["cwd"] = json!(cwd);
    assert_eq!(
        normalize_child_provider_request(&too_long, "session-source155").unwrap_err(),
        "details_incomplete"
    );
    fs::write(
        &requests_file,
        requests
            .iter()
            .map(|value| format!("{value}\n"))
            .collect::<String>(),
    )
    .unwrap();
    let mut options = fake_options(&trace);
    options.child_requests = true;
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[
            (
                "BRIDGET_CODEX_TRACE".into(),
                trace.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_HOLD_TURN".into(), "1".into()),
            (
                "BRIDGET_CODEX_CHILD_REQUESTS_FILE".into(),
                requests_file.to_string_lossy().into_owned(),
            ),
            (
                "BRIDGET_CODEX_STALL_READY".into(),
                stall_ready.to_string_lossy().into_owned(),
            ),
            (
                "BRIDGET_CODEX_STALL_RELEASE".into(),
                stall_release.to_string_lossy().into_owned(),
            ),
        ],
        false,
    )
    .unwrap();
    transport.deliver(&message("155-command-forward")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while transport
        .provider_identity()
        .unwrap()
        .active_turn_id
        .as_deref()
        != Some("turn-native")
        && Instant::now() < deadline
    {
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        transport
            .provider_identity()
            .unwrap()
            .active_turn_id
            .as_deref(),
        Some("turn-native")
    );
    write_value(&transport.writer, json!({"method":"native155/requests"})).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut forwarded = Vec::new();
    let mut frames = Vec::new();
    while Instant::now() < deadline {
        for event in transport.drain_events() {
            if let ManagedEventKind::ProviderRequest { request } = event.kind {
                forwarded.push(request);
            }
        }
        frames = fs::read_to_string(&trace)
            .unwrap()
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .collect();
        if forwarded.len() == 4
            && frames
                .iter()
                .any(|frame| frame["id"] == "cmd-4" && frame.get("error").is_some())
        {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    if forwarded.len() != 4 {
        transport.stop();
    }
    assert_eq!(
        forwarded.len(),
        4,
        "quatre attentes doivent remonter, reçues {forwarded:?}"
    );
    assert_eq!(
        forwarded[0].provider_request_id,
        ChildProviderRequestId::Number(i64::MIN)
    );
    assert!(forwarded.iter().all(|request| {
        request.provider_session
            == transport
                .provider_identity()
                .unwrap()
                .provider_session_id
                .as_deref()
                .unwrap()
            && request.turn_id == "turn-native"
    }));
    for original in &requests[..4] {
        assert!(
            !frames.iter().any(|frame| frame["id"] == original["id"]
                && (frame.get("result").is_some() || frame.get("error").is_some())),
            "une commande forward n'a aucune réponse automatique : {frames:?}"
        );
    }
    assert!(frames.iter().any(|frame| frame
        == &json!({"id":"cmd-4","error":{"code":-32000,"message":"child_request_refused"}})));
    // H2 : le fournisseur reste vivant et cesse de lire stdin. La
    // synchronisation fichier déclenche stdout APRES le writer saturé.
    write_value(&transport.writer, json!({"method":"native155/stall"})).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while !stall_ready.is_file() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        stall_ready.is_file(),
        "le faux fournisseur a fermé sa boucle de lecture stdin"
    );
    let writer = transport.writer.clone();
    let (done, result) = mpsc::channel();
    let writing_started = Instant::now();
    let writing = thread::spawn(move || {
        let outcome = write_value(
            &writer,
            json!({"method":"native155/full-pipe","params":{"text":"x".repeat(2 * 1024 * 1024)}}),
        );
        let _ = done.send(outcome);
    });
    let deadline = Instant::now() + Duration::from_secs(1);
    while transport.writer.try_lock().is_ok() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(1));
    }
    let writing_locked = transport.writer.try_lock().is_err();
    fs::write(&stall_release, b"terminal").unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut finished = false;
    let mut terminal_during_write = false;
    while Instant::now() < deadline {
        finished |= transport.drain_events().iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::TurnFinished {
                    terminal: ManagedTerminal::Completed,
                    ..
                }
            )
        });
        if finished {
            terminal_during_write = transport.writer.try_lock().is_err();
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    let answer = ChildRequestAnswer {
        request_id: "local-request".into(),
        provider_session: forwarded[0].provider_session.clone(),
        turn_id: forwarded[0].turn_id.clone(),
        provider_request_id: forwarded[0].provider_request_id.clone(),
        decision: ChildRequestDecision::Accept,
        answer: None,
        outcome: "accepted".into(),
    };
    let stale = transport.answer_provider_request(&answer);
    let outcome = result.recv_timeout(Duration::from_secs(4));
    let writing_elapsed = writing_started.elapsed();
    let supplier_alive = transport
        .child
        .lock()
        .unwrap()
        .try_wait()
        .unwrap()
        .is_none();
    if outcome.is_err() {
        // Filet RED : libérer notre propre groupe attesté si l'ancien
        // writer reste bloqué. Helper149/154 borné TERM/INT, jamais KILL.
        let mut child = transport.child.lock().unwrap();
        crate::managed_session::stop_owned_child(&mut child, true, Duration::from_secs(4)).unwrap();
    }
    writing.join().unwrap();
    let stop_started = Instant::now();
    transport.stop();
    assert!(
        writing_locked && terminal_during_write,
        "stdout doit continuer pendant le pipe plein, sans attendre writer"
    );
    assert!(
        supplier_alive,
        "l'erreur n'est pas un simple EOF fournisseur"
    );
    assert!(
        writing_elapsed < Duration::from_secs(4),
        "le budget total STDIO de3s a dépassé sa marge de planification : {writing_elapsed:?}"
    );
    assert!(
        matches!(outcome, Ok(Err(TransportError::Io(ref error))) if error.contains("budget")),
        "pipe plein : erreur IO bornée attendue, reçu {outcome:?}"
    );
    assert!(
        stop_started.elapsed() < Duration::from_secs(1),
        "shutdown reste retenu derrière writer"
    );
    assert!(
        finished,
        "le lecteur traite la fin malgré quatre attentes ouvertes"
    );
    assert!(
        stale.is_err(),
        "la fin du tour retire les attentes répondables"
    );
    let _ = fs::remove_dir_all(root);
}

/// T012 : les corps correspondent aux paramètres0.162.1. Ne pas inventer
/// des offres pour fileChange/permissions afin de fabriquer un refus.
#[test]
fn native155_transport_methodes_et_offres_refusees() {
    let root = root("155-refus-command");
    let trace = root.join("trace.jsonl");
    let requests_file = root.join("requests.jsonl");
    let command = json!({"threadId":"thread-native","turnId":"turn-native","itemId":"i",
            "startedAtMs":1787686800000i64,"command":"printf couleur","cwd":"/tmp"});
    let mut requests = vec![
        json!({"id":"file","method":"item/fileChange/requestApproval","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"f","startedAtMs":1787686800000i64,"grantRoot":null}}),
        json!({"id":"root","method":"item/fileChange/requestApproval","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"f","startedAtMs":1787686800000i64,"grantRoot":"/tmp"}}),
        json!({"id":"permissions","method":"item/permissions/requestApproval","params":{"threadId":"thread-native","turnId":"turn-native","itemId":"p","startedAtMs":1787686800000i64,"cwd":"/tmp","permissions":{"network":{"enabled":true}}}}),
    ];
    for (id, offered) in [
        ("session", json!(["acceptForSession"])),
        (
            "amendment",
            json!([{"acceptWithExecpolicyAmendment":{"execpolicy_amendment":["printf"]}}]),
        ),
        (
            "network-rule",
            json!([{"applyNetworkPolicyAmendment":{"network_policy_amendment":{"action":"allow","host":"example.com"}}}]),
        ),
    ] {
        let mut params = command.clone();
        params["availableDecisions"] = offered;
        requests.push(
            json!({"id":id,"method":"item/commandExecution/requestApproval","params":params}),
        );
    }
    let mut foreign = command;
    foreign["turnId"] = json!("foreign-turn");
    foreign["availableDecisions"] = json!(["accept", "decline", "cancel"]);
    requests.push(
        json!({"id":"foreign","method":"item/commandExecution/requestApproval","params":foreign}),
    );
    fs::write(
        &requests_file,
        requests
            .iter()
            .map(|value| format!("{value}\n"))
            .collect::<String>(),
    )
    .unwrap();
    let mut options = fake_options(&trace);
    options.child_requests = true;
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[
            (
                "BRIDGET_CODEX_TRACE".into(),
                trace.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_HOLD_TURN".into(), "1".into()),
            (
                "BRIDGET_CODEX_CHILD_REQUESTS_FILE".into(),
                requests_file.to_string_lossy().into_owned(),
            ),
        ],
        false,
    )
    .unwrap();
    transport.deliver(&message("155-refus-command")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while transport
        .provider_identity()
        .unwrap()
        .active_turn_id
        .as_deref()
        != Some("turn-native")
        && Instant::now() < deadline
    {
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        transport
            .provider_identity()
            .unwrap()
            .active_turn_id
            .as_deref(),
        Some("turn-native")
    );
    write_value(&transport.writer, json!({"method":"native155/requests"})).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut events = Vec::new();
    let mut replies = Vec::new();
    while Instant::now() < deadline {
        events.extend(transport.drain_events());
        replies = fs::read_to_string(&trace)
            .unwrap()
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter(|frame| {
                frame.get("method").is_none()
                    && (frame.get("result").is_some() || frame.get("error").is_some())
            })
            .collect();
        if replies.len() == requests.len() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    transport.stop();
    events.extend(transport.drain_events());
    assert!(
        !events
            .iter()
            .any(|event| matches!(event.kind, ManagedEventKind::ProviderRequest { .. })),
        "aucune méthode interdite ne remonte"
    );
    for request in requests {
        assert_eq!(
            replies.iter().find(|reply| reply["id"] == request["id"]),
            Some(
                &json!({"id":request["id"],"error":{"code":-32000,"message":"child_request_refused"}})
            ),
            "refus fermé pour {request}"
        );
    }
    let _ = fs::remove_dir_all(root);
}

/// T014 : aucun formulaire tiers ne devient une question Parent.
#[test]
fn native155_transport_mcp_tiers_et_exception_bridget() {
    let root = root("155-mcp");
    let trace = root.join("trace.jsonl");
    let requests_file = root.join("requests.jsonl");
    let mut requests = Vec::new();
    for (id, mode, extra) in [
        (
            "typed",
            "form",
            json!({"message":"Choix","requestedSchema":{"type":"object","properties":{"colour":{"type":"string","enum":["bleu","vert"]}},"required":["colour"]}}),
        ),
        (
            "empty",
            "form",
            json!({"message":"Vide","requestedSchema":{"type":"object","properties":{}}}),
        ),
        (
            "metadata",
            "form",
            json!({"message":"Choix","requestedSchema":{"type":"object","properties":{}},"_meta":{"unknown":true}}),
        ),
        (
            "openai",
            "openai/form",
            json!({"message":"Choix","requestedSchema":{}}),
        ),
        (
            "alias",
            "openaiForm",
            json!({"message":"Choix","requestedSchema":{}}),
        ),
        (
            "verify",
            "openai/userVerification",
            json!({"challenge":"opaque","description":"Vérifier","title":"Validation"}),
        ),
        (
            "url",
            "url",
            json!({"message":"Lien","elicitationId":"e","url":"https://example.com"}),
        ),
    ] {
        let mut params = extra;
        params["serverName"] = json!("third-party");
        params["threadId"] = json!("thread-native");
        params["turnId"] = json!("turn-native");
        params["mode"] = json!(mode);
        requests.push(json!({"id":id,"method":"mcpServer/elicitation/request","params":params}));
    }
    requests.push(json!({"id":"bridget","method":"mcpServer/elicitation/request","params":{"serverName":"bridget","threadId":"thread-native","turnId":"turn-native","mode":"form","message":"Vide","requestedSchema":{"type":"object","properties":{}}}}));
    fs::write(
        &requests_file,
        requests
            .iter()
            .map(|value| format!("{value}\n"))
            .collect::<String>(),
    )
    .unwrap();
    let mut options = fake_options(&trace);
    options.child_requests = true;
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[
            (
                "BRIDGET_CODEX_TRACE".into(),
                trace.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_HOLD_TURN".into(), "1".into()),
            (
                "BRIDGET_CODEX_CHILD_REQUESTS_FILE".into(),
                requests_file.to_string_lossy().into_owned(),
            ),
        ],
        false,
    )
    .unwrap();
    transport.deliver(&message("155-mcp")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while transport
        .provider_identity()
        .unwrap()
        .active_turn_id
        .as_deref()
        != Some("turn-native")
        && Instant::now() < deadline
    {
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        transport
            .provider_identity()
            .unwrap()
            .active_turn_id
            .as_deref(),
        Some("turn-native")
    );
    write_value(&transport.writer, json!({"method":"native155/requests"})).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut events = Vec::new();
    let mut replies = Vec::new();
    while Instant::now() < deadline {
        events.extend(transport.drain_events());
        replies = fs::read_to_string(&trace)
            .unwrap()
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter(|frame| frame.get("method").is_none())
            .collect();
        if replies.len() == requests.len() {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    transport.stop();
    events.extend(transport.drain_events());
    assert!(
        !events
            .iter()
            .any(|event| matches!(event.kind, ManagedEventKind::ProviderRequest { .. }))
    );
    for request in &requests[..requests.len() - 1] {
        assert_eq!(
            replies.iter().find(|reply| reply["id"] == request["id"]),
            Some(&json!({"id":request["id"],"result":{"action":"decline"}})),
            "tiers refusé sans content : {request}"
        );
        let expected_code = if request["params"]["mode"] == "url" {
            "url_mode_unsupported"
        } else {
            "approval_method_unsupported"
        };
        let expected_reference =
            provider_fingerprint(b"delegated-runtime", b"mcpServer/elicitation/request");
        assert!(
            events.iter().any(|event| {
                serde_json::from_slice::<Value>(&event.raw)
                    .ok()
                    .is_some_and(|source| source["id"] == request["id"])
                    && matches!(&event.kind, ManagedEventKind::Diagnostic { code, reference }
                        if code == expected_code && reference == &expected_reference)
            }),
            "motif {expected_code} attesté pour {request}"
        );
    }
    assert_eq!(
        replies.iter().find(|reply| reply["id"] == "bridget"),
        Some(&json!({"id":"bridget","result":{"action":"accept","content":{}}}))
    );
    let _ = fs::remove_dir_all(root);
}

/// T014 : secret normatif, puis enveloppe fournisseur distincte de l'API.
#[test]
fn native155_transport_user_input_secret_et_reponses() {
    transport_user_input_roundtrip(true);
}

#[test]
fn user_input181_primary_codex_roundtrip() {
    transport_user_input_roundtrip(false);
}

fn transport_user_input_roundtrip(forward_child_requests: bool) {
    use crate::protocol::{ChildRequestAnswer, ChildRequestDecision};
    let root = root(if forward_child_requests {
        "155-input"
    } else {
        "181-input"
    });
    let trace = root.join("trace.jsonl");
    let requests_file = root.join("requests.jsonl");
    let requests: Vec<Value> = ["secret", "accept", "decline", "cancel"].into_iter().map(|id| json!({
            "id":id,"method":"item/tool/requestUserInput","params":{"threadId":"thread-native","turnId":"turn-native","itemId":id,"isBlocking":true,
                "questions":[{"id":"q1","header":"Choix","question":"Quelle couleur","isSecret":id == "secret","isOther":false,"options":[{"label":"bleu","description":"Couleur"}]}]}
        })).collect();
    fs::write(
        &requests_file,
        requests
            .iter()
            .map(|value| format!("{value}\n"))
            .collect::<String>(),
    )
    .unwrap();
    let mut options = fake_options(&trace);
    options.child_requests = forward_child_requests;
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[
            (
                "BRIDGET_CODEX_TRACE".into(),
                trace.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_HOLD_TURN".into(), "1".into()),
            (
                "BRIDGET_CODEX_CHILD_REQUESTS_FILE".into(),
                requests_file.to_string_lossy().into_owned(),
            ),
        ],
        false,
    )
    .unwrap();
    transport.deliver(&message("155-input")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while transport
        .provider_identity()
        .unwrap()
        .active_turn_id
        .as_deref()
        != Some("turn-native")
        && Instant::now() < deadline
    {
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        transport
            .provider_identity()
            .unwrap()
            .active_turn_id
            .as_deref(),
        Some("turn-native")
    );
    write_value(&transport.writer, json!({"method":"native155/requests"})).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut forwarded = Vec::new();
    while Instant::now() < deadline {
        for event in transport.drain_events() {
            if let ManagedEventKind::ProviderRequest { request } = event.kind {
                forwarded.push(request);
            }
        }
        if forwarded.len() == 3 {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    if forwarded.len() != 3 {
        transport.stop();
    }
    assert_eq!(
        forwarded.len(),
        3,
        "seules les trois questions non secrètes remontent"
    );
    for (request, decision) in forwarded.iter().zip([
        ChildRequestDecision::Accept,
        ChildRequestDecision::Decline,
        ChildRequestDecision::Cancel,
    ]) {
        assert_eq!(request.question_ids, vec!["q1"]);
        let answer = ChildRequestAnswer {
            request_id: "local-request".into(),
            provider_session: request.provider_session.clone(),
            turn_id: request.turn_id.clone(),
            provider_request_id: request.provider_request_id.clone(),
            decision,
            answer: if decision == ChildRequestDecision::Accept {
                Some(json!({"q1":["bleu"]}))
            } else {
                None
            },
            outcome: "answered".into(),
        };
        transport
            .answer_provider_request(&answer)
            .expect("réponse fournisseur écrite");
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut replies = Vec::new();
    while Instant::now() < deadline {
        replies = fs::read_to_string(&trace)
            .unwrap()
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter(|frame| frame.get("method").is_none())
            .collect();
        if replies.len() == 4 {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    transport.stop();
    assert_eq!(
        replies.iter().find(|reply| reply["id"] == "accept"),
        Some(&json!({"id":"accept","result":{"answers":{"q1":{"answers":["bleu"]}}}}))
    );
    for id in ["secret", "decline", "cancel"] {
        assert_eq!(
            replies.iter().find(|reply| reply["id"] == id),
            Some(&json!({"id":id,"error":{"code":-32000,"message":"child_request_refused"}}))
        );
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn passerelle_codex_marque_seulement_les_erreurs_terminales() {
    let terminal = codex_journal_payload(
        "error",
        json!({
            "reason": "échéance Codex dépassée",
            "pending_provider_request": {"state": "pending"}
        }),
    );
    assert_eq!(terminal["terminal_kind"], crate::journal::TURN_FAILED_KIND);
    assert_eq!(
        terminal["pending_provider_request"],
        json!({"state": "pending"})
    );

    let update = codex_journal_payload("update", json!({"kind": "text"}));
    assert!(update.get("terminal_kind").is_none());

    let malformed = codex_journal_payload("error", json!("payload invalide"));
    assert_eq!(malformed, json!("payload invalide"));
}

#[test]
fn session_native_respecte_sequence_saturation_et_octets_sources() {
    let root = root("sequence");
    let trace = root.join("trace.jsonl");
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        fake_options(&trace),
        &[
            (
                "BRIDGET_CODEX_TRACE".to_string(),
                trace.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_CREDIT_UPDATES".to_string(), "1".into()),
        ],
        false,
    )
    .expect("session native");
    let feed = JournalLiveFeed::default();
    transport
        .activate_journal(&root, "codex-native", Some(feed))
        .expect("journal activé");
    let mut sent = message("native-1");
    sent.reply = true;
    sent.body = "<!doctype html>\n<script>const texte = 'été';</script>\n".to_string();
    transport.deliver(&sent).expect("livraison");

    let deadline = Instant::now() + Duration::from_secs(3);
    let mut events = Vec::new();
    while Instant::now() < deadline {
        events.extend(transport.drain_events());
        if events.iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::TurnFinished { ref response, .. } if response == "réponse native"
            )
        }) {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(events.iter().any(|event| matches!(
        event.kind,
        ManagedEventKind::TurnFinished { ref response, terminal: ManagedTerminal::Completed, .. }
            if response == "réponse native"
    )));
    assert!(events.iter().any(|event| matches!(
        event.kind,
        ManagedEventKind::RuntimeObserved { ref model, effort: Some(ref effort) }
            if model == "gpt-5.6-terra" && effort == "high"
    )));
    assert!(events.iter().any(|event| matches!(
        event.kind,
        ManagedEventKind::RateLimitObserved {
            ref window,
            ref status,
            resets_at: Some(1_787_572_200),
            used_percent: Some(42),
            credits: None,
        } if window == "primary/300m" && status == "available"
    )));
    assert!(events.iter().any(|event| matches!(&event.kind,
            ManagedEventKind::RateLimitObserved { used_percent:Some(100),credits:Some(credits),.. }
                if credits.usable())));
    let administrative: Vec<_> = events
        .iter()
        .filter_map(|event| match &event.kind {
            ManagedEventKind::RateLimitObserved {
                window, credits, ..
            } if window == "account/credits" => Some(credits),
            _ => None,
        })
        .collect();
    assert_eq!(administrative.len(), 2);
    assert!(administrative[0].as_ref().unwrap().usable());
    assert!(
        administrative[1].is_none(),
        "la notification sparse révoque les crédits"
    );
    let raw = events.iter().find(|event| {
        matches!(event.kind, ManagedEventKind::Update { .. })
            && event
                .raw
                .starts_with(b"{  \"method\" : \"item/agentMessage/delta\"")
    });
    let raw = raw.expect("notification native source");
    assert_eq!(raw.origin, ManagedEventOrigin::SourceLine);
    assert_eq!(raw.source, ManagedEventSource::CodexAppServer);
    // Oracle externe, extrait du faux fournisseur historique (dfa2134).
    // Le lecteur retire seulement le LF JSONL : aucun parse/réencodage
    // ne peut remplacer les espaces ni les octets UTF-8 du fournisseur.
    assert_eq!(
        raw.raw.as_slice(),
        include_bytes!(
            "../../../specs/089-communication-core/contracts/fixtures/native-codex-delta.jsonl"
        )
        .strip_suffix(b"\n")
        .expect("fixture JSONL terminée par LF")
    );

    let trace = fs::read_to_string(&trace).expect("trace fournisseur");
    let frames = trace.lines().collect::<Vec<_>>();
    assert_eq!(
        frames
            .iter()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter_map(|frame| frame
                .get("method")
                .and_then(Value::as_str)
                .map(str::to_owned))
            .collect::<Vec<_>>(),
        vec![
            "initialize",
            "initialized",
            "thread/start",
            "account/rateLimits/read",
            "turn/start",
            "turn/start"
        ]
    );
    assert!(frames.iter().all(|frame| !frame.contains("jsonrpc")));
    // Oracle au destinataire : enveloppe complète et corps inchangé,
    // sans ordre de rendu/publication ajouté au contenu HTML.
    let turns = frames
        .iter()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .filter(|frame| frame["method"] == "turn/start")
        .collect::<Vec<_>>();
    assert_eq!(turns.len(), 2);
    for turn in turns {
        let prompt = turn["params"]["input"][0]["text"].as_str().unwrap();
        let (header, rest) = prompt.split_once('\n').unwrap();
        let envelope: Value = serde_json::from_str(
            header
                .strip_prefix("[Message Bridget : ")
                .unwrap()
                .strip_suffix(']')
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            envelope,
            json!({"from": sent.from, "to": sent.to, "id": sent.id, "reply": true, "in_reply_to": null})
        );
        assert_eq!(
            rest.split_once("\n\n[Réponse Bridget]\n")
                .unwrap()
                .0
                .as_bytes(),
            sent.body.as_bytes()
        );
        assert!(rest.contains("le wrapper relaie automatiquement ta réponse finale"));
        assert!(rest.contains("catalogue d'outils disponible"));
        assert!(!rest.contains("kind:html"));
    }
    transport.stop();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn absence_de_signaux_codex_ne_cree_aucun_fait() {
    assert_eq!(runtime_from_thread_start(&json!({})), None);
    assert!(rate_limits_from_snapshot(None).is_empty());
    assert!(rate_limits_from_snapshot(Some(&json!({ "primary": null }))).is_empty());
}

#[test]
fn snapshot_codex_emet_primary_et_secondary_sans_ecrasement() {
    let facts = rate_limits_from_snapshot(Some(&json!({
        "primary": {
            "usedPercent": 19,
            "windowDurationMins": 300,
            "resetsAt": 1_787_572_200
        },
        "secondary": {
            "usedPercent": 61,
            "windowDurationMins": 10080,
            "resetsAt": 1_787_700_000
        },
        "rateLimitReachedType": null
    })));
    assert_eq!(facts.len(), 2);
    assert_eq!(
        facts[0],
        (
            "primary/300m".to_string(),
            "available".to_string(),
            Some(1_787_572_200),
            Some(19),
            None
        )
    );
    assert_eq!(
        facts[1],
        (
            "secondary/10080m".to_string(),
            "available".to_string(),
            Some(1_787_700_000),
            Some(61),
            None
        )
    );
}

#[test]
fn percent_hors_bornes_conserve_la_fenetre_sans_pourcent() {
    // Mandat : % omis, fenêtre conservée — pas d'effacement via `?`.
    let facts = rate_limits_from_snapshot(Some(&json!({
        "primary": {
            "usedPercent": 250,
            "windowDurationMins": 300,
            "resetsAt": 1_787_572_200
        },
        "rateLimitReachedType": null
    })));
    assert_eq!(
        facts.len(),
        1,
        "la fenêtre doit survivre à un % hors bornes"
    );
    assert_eq!(facts[0].0, "primary/300m");
    assert_eq!(facts[0].1, "available");
    assert_eq!(facts[0].2, Some(1_787_572_200));
    assert_eq!(facts[0].3, None, "% omis, pas fenêtre effacée");
    let mut snapshot = json!({"primary":{"usedPercent":100,"resetsAt":500},
            "credits":{"hasCredits":true,"unlimited":false,"balance":"opaque"},
            "spendControlReached":false,"rateLimitReachedType":"rate_limit_reached"});
    for (has, unlimited, spend, usable) in [
        (true, false, false, true),
        (false, true, false, true),
        (false, false, false, false),
        (true, false, true, false),
    ] {
        snapshot["credits"]["hasCredits"] = json!(has);
        snapshot["credits"]["unlimited"] = json!(unlimited);
        snapshot["spendControlReached"] = json!(spend);
        let facts = rate_limits_from_snapshot(Some(&snapshot));
        assert_eq!(facts[0].3, Some(100));
        assert_eq!(facts[0].1, "rate_limit_reached");
        assert_eq!(facts[0].4.as_ref().unwrap().usable(), usable);
    }
    snapshot["spendControlReached"] = json!(false);
    snapshot["credits"]["hasCredits"] = json!("true");
    assert!(rate_limits_from_snapshot(Some(&snapshot))[0].4.is_none());
    assert!(
        rate_limit_update_facts(Some(&json!({"credits":null})))[0]
            .4
            .is_none()
    );
    assert_eq!(
        rate_limit_update_facts(Some(&json!({})))[0].0,
        "account/credits"
    );
    assert!(rate_limit_update_facts(None).is_empty());
    let spend = rate_limits_from_snapshot(Some(&json!({"spendControlReached":true})));
    assert_eq!(spend.len(), 1);
    assert!(spend[0].4.as_ref().unwrap().spend_control_reached);
    assert!(!spend[0].4.as_ref().unwrap().usable());
    let spent_malformed =
        codex_credits(&json!({"credits":{"hasCredits":"bad"},"spendControlReached":true})).unwrap();
    assert_eq!(spent_malformed.has_credits, None);
    assert!(!spent_malformed.usable());
}

#[test]
fn annulation_interrompt_le_tour_natif() {
    let root = root("cancel");
    let trace = root.join("trace.jsonl");
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        fake_options(&trace),
        &[
            (
                "BRIDGET_CODEX_TRACE".to_string(),
                trace.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
        ],
        false,
    )
    .expect("session native");
    let sent = message("cancel-1");
    transport.deliver(&sent).expect("livraison");
    let deadline = Instant::now() + Duration::from_secs(1);
    while Instant::now() < deadline && !transport.is_busy() {
        thread::sleep(Duration::from_millis(5));
    }
    assert!(
        transport.cancel_delivery("cancel-1", "annulation test"),
        "annulation absente; trace={}; events={:?}",
        fs::read_to_string(&trace).unwrap_or_else(|error| error.to_string()),
        transport.drain_events(),
    );
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut events = Vec::new();
    while Instant::now() < deadline {
        events.extend(transport.drain_events());
        if events.iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::TurnFinished {
                    terminal: ManagedTerminal::Cancelled,
                    ..
                }
            )
        }) {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(events.iter().any(|event| matches!(
        event.kind,
        ManagedEventKind::TurnFinished {
            terminal: ManagedTerminal::Cancelled,
            ..
        }
    )));
    transport.stop();
    let _ = fs::remove_dir_all(root);
}

#[test]
fn eof_pendant_un_tour_reveille_le_worker_et_nettoie_le_groupe() {
    let root = root("eof-turn");
    let trace = root.join("trace.jsonl");
    let mut options = fake_options(&trace);
    // L'ancien worker attendait jusqu'à l'échéance du tour : cette valeur
    // rend le mutant visible sans faire dormir le test pendant une minute.
    options.notify_timeout_secs = 60;
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[
            (
                "BRIDGET_CODEX_TRACE".to_string(),
                trace.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
        ],
        false,
    )
    .expect("session native");
    transport.deliver(&message("eof-1")).expect("livraison");

    let started = Instant::now();
    let mut events = Vec::new();
    while started.elapsed() < Duration::from_secs(2) {
        events.extend(transport.drain_events());
        // La seconde requête tracée précède sa réponse. Seul cet événement
        // prouve que le worker a reçu l'identifiant du tour actif.
        if events.iter().any(|event| {
            matches!(&event.kind,
                ManagedEventKind::PromptDispatched {message_id} if message_id == "eof-1")
        }) {
            break;
        }
        thread::sleep(Duration::from_millis(5));
    }
    assert!(
        events.iter().any(|event| matches!(&event.kind,
                ManagedEventKind::PromptDispatched {message_id} if message_id == "eof-1")),
        "le tour suspendu n'a pas reçu son identifiant: {events:?}"
    );
    let adapter_pid = transport.process_id() as i32;

    // SAFETY: kill ne déréférence aucun pointeur ; ses arguments sont des entiers.
    assert_eq!(unsafe { libc::kill(adapter_pid, libc::SIGTERM) }, 0);
    let deadline = Instant::now() + Duration::from_secs(1);
    while Instant::now() < deadline {
        events.extend(transport.drain_events());
        if events.iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
                    if message_id == "eof-1" && reason == "stdout Codex fermé pendant le tour"
            )
        }) {
            break;
        }
        thread::sleep(Duration::from_millis(5));
    }
    assert!(
        events.iter().any(|event| matches!(
            event.kind,
            ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
                if message_id == "eof-1" && reason == "stdout Codex fermé pendant le tour"
        )),
        "EOF pendant le tour n'a pas terminé le worker: {events:?}"
    );

    transport.stop();
    // SAFETY: kill(pid, 0) est l'idiome POSIX de sonde d'existence ; aucun signal n'est envoyé.
    assert_eq!(unsafe { libc::kill(adapter_pid, 0) }, -1);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn stop_pendant_un_tour_termine_le_groupe_enfant() {
    let root = root("stop-group");
    let trace = root.join("trace.jsonl");
    let child_pid_path = root.join("child.pid");
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        fake_options(&trace),
        &[
            (
                "BRIDGET_CODEX_TRACE".to_string(),
                trace.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_HOLD_TURN".to_string(), "1".to_string()),
            (
                "BRIDGET_CODEX_CHILD_PID".to_string(),
                child_pid_path.to_string_lossy().into_owned(),
            ),
        ],
        false,
    )
    .expect("session native");
    transport.deliver(&message("stop-1")).expect("livraison");
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline && !child_pid_path.is_file() {
        thread::sleep(Duration::from_millis(5));
    }
    let child_pid = fs::read_to_string(&child_pid_path)
        .expect("descendant Codex")
        .trim()
        .parse::<i32>()
        .expect("pid descendant");

    let started = Instant::now();
    transport.stop();
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "stop attend encore l'échéance du tour"
    );
    let reaped_deadline = Instant::now() + Duration::from_secs(1);
    // SAFETY: kill(pid, 0) est l'idiome POSIX de sonde d'existence ; aucun signal n'est envoyé.
    while Instant::now() < reaped_deadline && unsafe { libc::kill(child_pid, 0) } == 0 {
        thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(
        // SAFETY: kill(pid, 0) est l'idiome POSIX de sonde d'existence ; aucun signal n'est envoyé.
        unsafe { libc::kill(child_pid, 0) },
        -1,
        "le groupe enfant Codex survit au shutdown"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn reroutage_codex_atteste_le_modele_sans_inventer_si_muet() {
    let rerouted: Value =
        serde_json::from_str(r#"{"method":"modelRerouted","params":{"to":"gpt-5.4"}}"#).unwrap();
    assert_eq!(
        served_model_from_codex(&rerouted).as_deref(),
        Some("gpt-5.4")
    );
    let mute: Value = serde_json::from_str(r#"{"method":"modelRerouted","params":{}}"#).unwrap();
    assert!(served_model_from_codex(&mute).is_none());
}

#[test]
fn reprise_et_bifurcation_codex_emettent_la_requete_attestee_ou_refusent() {
    let compatible = ProviderObservation {
        binary_path: "/fixtures/codex".to_string(),
        binary_version: "0.150.1".to_string(),
        binary_digest: "0".repeat(64),
        contract_version: "codex-app-server-fixture-v1".to_string(),
        operations: vec![ProviderOperation::Resume, ProviderOperation::Fork],
    };
    let cwd = Path::new("/workspace");
    let resume = CodexThreadBootstrap::Resume {
        thread_id: "thread-parent".to_string(),
    };
    let mut interactive = fake_options(Path::new("/tmp/non-execute"));
    interactive.thread_bootstrap = resume.clone();
    assert!(validate_endpoint_bootstrap(&interactive, true).is_ok());
    assert!(validate_endpoint_bootstrap(&interactive, false).is_err());
    assert_eq!(
        thread_bootstrap_request(&resume, cwd, Some("explicite"), true).1,
        json!({"threadId": "thread-parent", "model": "explicite", "excludeTurns": true})
    );
    let fork = CodexThreadBootstrap::Fork {
        thread_id: "thread-parent".to_string(),
    };
    assert!(validate_thread_bootstrap(&Some(compatible.clone()), &resume).is_ok());
    assert!(validate_thread_bootstrap(&Some(compatible.clone()), &fork).is_ok());
    assert_eq!(
        thread_bootstrap_request(&resume, cwd, None, false),
        ("thread/resume", json!({ "threadId": "thread-parent" }))
    );
    assert_eq!(
        thread_bootstrap_request(&fork, cwd, None, false),
        ("thread/fork", json!({ "threadId": "thread-parent" }))
    );
    assert!(matches!(
        validate_thread_bootstrap(&None, &resume),
        Err(TransportError::DeliveryFailed(reason)) if reason.contains("thread/resume")
    ));
    let fork_incompatible = ProviderObservation {
        operations: vec![ProviderOperation::Resume],
        ..compatible
    };
    assert!(matches!(
        validate_thread_bootstrap(&Some(fork_incompatible), &fork),
        Err(TransportError::DeliveryFailed(reason)) if reason.contains("thread/fork")
    ));
}

#[test]
fn reprise_geree_projette_les_restrictions_effectives_ou_refuse_avant_le_fil() {
    for sandbox in ["read-only", "workspace-write", "missing"] {
        for bootstrap in [
            CodexThreadBootstrap::Start,
            CodexThreadBootstrap::Resume {
                thread_id: "thread-native".into(),
            },
            CodexThreadBootstrap::Fork {
                thread_id: "thread-native".into(),
            },
        ] {
            let root = root("resume-policy");
            let trace = root.join("trace.jsonl");
            let is_start = matches!(bootstrap, CodexThreadBootstrap::Start);
            let mut options = fake_options(&trace);
            options.permissions = "deny".into();
            options.thread_bootstrap = bootstrap;
            options.provider_observation = Some(ProviderObservation {
                binary_path: "/fixtures/codex".into(),
                binary_version: "fixture".into(),
                binary_digest: "0".repeat(64),
                contract_version: "codex-app-server-fixture-v1".into(),
                operations: vec![ProviderOperation::Resume, ProviderOperation::Fork],
            });
            options.args = vec!["-c".into(), r#"
                    while IFS= read -r line; do
                        printf '%s\n' "$line" >> "$BRIDGET_CODEX_TRACE"
                        id=$(printf '%s' "$line" | sed 's/.*"id":\([0-9]*\).*/\1/')
                        case "$line" in
                            *'"method":"initialize"'*) printf '%s\n' '{"id":1,"result":{"userAgent":"fake","codexHome":"/tmp","platformFamily":"unix","platformOs":"macos"}}' ;;
                            *'"method":"config/read"'*)
                                if [ "$POLICY_SANDBOX" = missing ]; then
                                    printf '{"id":%s,"result":{"config":{}}}\n' "$id"
                                else
                                    printf '{"id":%s,"result":{"config":{"approval_policy":"never","sandbox_mode":"%s"}}}\n' "$id" "$POLICY_SANDBOX"
                                fi ;;
                            *'"method":"thread/start"'*|*'"method":"thread/resume"'*|*'"method":"thread/fork"'*) printf '{"id":%s,"result":{"thread":{"id":"thread-native"}}}\n' "$id" ;;
                            *'"method":"account/rateLimits/read"'*) printf '{"id":%s,"result":{}}\n' "$id" ;;
                        esac
                    done
                "#.into()];
            let result = CodexAppServerTransport::spawn_with_environment(
                options,
                &[
                    (
                        "BRIDGET_CODEX_TRACE".into(),
                        trace.to_string_lossy().into_owned(),
                    ),
                    ("POLICY_SANDBOX".into(), sandbox.into()),
                ],
                false,
            );
            if sandbox == "missing" && !is_start {
                assert!(
                    matches!(result, Err(TransportError::DeliveryFailed(ref reason)) if reason.contains("configuration effective approval_policy absente"))
                );
            } else {
                result.expect("transport restreint").stop();
            }
            let frames: Vec<Value> = fs::read_to_string(&trace)
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect();
            let thread = frames.iter().find(|frame| {
                frame["method"]
                    .as_str()
                    .is_some_and(|method| method.starts_with("thread/"))
            });
            if sandbox == "missing" && !is_start {
                assert!(
                    thread.is_none(),
                    "aucun fil ne démarre sans politique attestée"
                );
            } else {
                let params = &thread.unwrap()["params"];
                assert_eq!(params["cwd"], json!(std::env::current_dir().unwrap()));
                if is_start {
                    assert!(!frames.iter().any(|frame| frame["method"] == "config/read"));
                } else {
                    assert_eq!(params["sandbox"], sandbox);
                    assert_eq!(params["approvalPolicy"], "never");
                }
            }
            fs::remove_dir_all(root).unwrap();
        }
    }
}
#[test]
fn consigne_privee_preserve_le_corps_visible_du_message() {
    let body = "Demande utilisateur visible.";
    let prompt = private_prompt(Some("Privilégie les sources attestées."), body);

    assert!(prompt.contains("Privilégie les sources attestées."));
    assert!(prompt.ends_with(body));
    assert_eq!(body, "Demande utilisateur visible.");
    assert_eq!(private_prompt(None, body), body);
}

#[test]
fn consigne_reponse_distingue_mcp_interactif_et_relais_gere() {
    let mut message = message("demande-liée");
    message.in_reply_to = Some("parent".into());
    for interactive in [false, true] {
        let prompt = communication_prompt(&message, interactive);
        assert!(prompt.contains("\"in_reply_to\":\"parent\""));
        assert_eq!(
            prompt.contains("réponds une seule fois par l'outil bridget_send"),
            interactive
        );
        assert_eq!(
            prompt.contains("le wrapper relaie automatiquement"),
            !interactive
        );
        assert!(prompt.contains("un refus du shell ne prouve pas une panne MCP"));
    }
}

#[test]
fn spec133_codex_affiche_la_delegation_sans_session_native() {
    let mut message = message("delegated-codex");
    message.delegated_origin = Some(bridget_core::DelegatedOrigin {
        provider: "cursor".into(),
        child_ref: "0123456789abcdef".into(),
    });
    let prompt = communication_prompt(&message, false);
    assert!(prompt.contains("via sous-agent cursor 0123456789abcdef"));
    assert!(prompt.contains("\"delegated_origin\""));
    assert!(!prompt.contains("session-native-secrete"));
}

#[test]
fn spec105_codex_sans_demande_ne_prescrit_aucun_envoi() {
    for interactive in [false, true] {
        let mut message = message("notification");
        message.reply = false;
        message.in_reply_to = Some("question".into());
        let prompt = communication_prompt(&message, interactive);
        assert!(prompt.contains("Pas d'accusé de réception"));
        assert!(!prompt.contains("réponds une seule fois"));
        assert!(!prompt.contains("relaie automatiquement"));
        assert!(prompt.contains(&message.body));
    }
}

#[test]
fn document_html_reste_un_corps_sans_consigne_ui_implicite() {
    let body = "<html>  <script>window.contenu = 'é';</script>\n</html>";
    assert_eq!(private_prompt(None, body).as_bytes(), body.as_bytes());
    assert_eq!(private_prompt(Some("  "), body).as_bytes(), body.as_bytes());
}

#[test]
fn outil_dynamique_codex_rejoint_lexecuteur_bridget_atteste() {
    let request = json!({
        "id": "dynamic-1",
        "method": "item/tool/call",
        "params": {
            "tool": "bridget_publish_artifact",
            "arguments": { "kind": "html" },
        }
    });
    let handler: DynamicToolHandler = Arc::new(|tool, arguments| {
        assert_eq!(tool, "bridget_publish_artifact");
        assert_eq!(arguments["kind"], "html");
        Ok(json!({ "status": "published", "receipt": "artifact-1" }))
    });

    let (reply, detail) = dynamic_tool_response(&request, Some(&handler)).expect("réponse");
    assert_eq!(detail, "publication d’artefact Bridget");
    assert_eq!(reply["result"]["success"], true);
    assert!(
        reply["result"]["contentItems"][0]["text"]
            .as_str()
            .is_some_and(|text| text.contains("published"))
    );
}

#[test]
fn outil_dynamique_inconnu_est_refuse_explicitement() {
    let request = json!({
        "id": "dynamic-2",
        "method": "item/tool/call",
        "params": { "tool": "arbitrary_tool", "arguments": {} }
    });
    let handler: DynamicToolHandler =
        Arc::new(|_, _| panic!("outil inconnu ne doit jamais atteindre l'exécuteur"));

    let (reply, detail) = dynamic_tool_response(&request, Some(&handler)).expect("réponse");
    assert_eq!(detail, "outil dynamique Bridget refusé");
    assert_eq!(reply["result"]["success"], false);
}

#[test]
fn outil_dynamique_lecture_conserve_reference_et_document_inerte() {
    let arguments = json!({"artifact_id": "artifact-089", "version": 1});
    let document = "<!doctype html>\n<script>alerte('é');</script>\n";
    let expected = json!({"status":"read", "artifact_id":"artifact-089", "content":document});
    let expected_arguments = arguments.clone();
    let result = expected.clone();
    let handler: DynamicToolHandler = Arc::new(move |tool, actual| {
        assert_eq!(tool, "bridget_read_artifact");
        assert_eq!(actual, &expected_arguments);
        Ok(result.clone())
    });
    let request = json!({"id":"read-089", "method":"item/tool/call", "params":{
        "tool":"bridget_read_artifact", "arguments":arguments
    }});
    let (reply, detail) = dynamic_tool_response(&request, Some(&handler)).unwrap();
    assert_eq!(detail, "lecture d’artefact Bridget");
    assert_eq!(reply["id"], "read-089");
    assert_eq!(reply["result"]["success"], true);
    assert_eq!(reply["result"]["contentItems"][0]["type"], "inputText");
    let content = reply["result"]["contentItems"][0]["text"].as_str().unwrap();
    assert_eq!(serde_json::from_str::<Value>(content).unwrap(), expected);
}

#[test]
fn outil_dynamique_lecture_sans_autorite_ou_en_echec_ne_pretend_pas_reussir() {
    let request = json!({"id":89, "method":"item/tool/call", "params":{
        "tool":"bridget_read_artifact", "arguments":{"artifact_id":"absent"}
    }});
    let (reply, _) = dynamic_tool_response(&request, None).unwrap();
    assert_eq!(reply["result"]["success"], false);
    let handler: DynamicToolHandler = Arc::new(|_, _| Err("SECRET_FIXTURE_089".to_string()));
    let (reply, _) = dynamic_tool_response(&request, Some(&handler)).unwrap();
    assert_eq!(reply["result"]["success"], false);
    assert!(!reply.to_string().contains("SECRET_FIXTURE_089"));
}

#[test]
fn elicitation_bridget_formulaire_implicite_est_acceptee() {
    let request = json!({
        "id": "elicitation-1",
        "method": "mcpServer/elicitation/request\n",
        "params": {
            "serverName": "bridget",
            "request": {
                "method": "elicitation/create",
                "params": {
                    "message": "Publication prête.",
                    "requestedSchema": { "type": "object", "properties": {} }
                }
            }
        }
    });

    assert!(is_mcp_elicitation_request(
        request["method"].as_str().expect("méthode")
    ));
    let (reply, payload) = mcp_elicitation_response(&request).expect("réponse MCP");
    assert_eq!(reply["result"]["action"], "accept");
    assert_eq!(reply["result"]["content"], json!({}));
    assert_eq!(payload["code"], "mcp_elicitation_autoaccepted");
}

#[test]
fn elicitation_mcp_externe_ou_avec_champ_requis_est_declinee() {
    let request = json!({
        "id": "elicitation-2",
        "method": "mcpServer/elicitation/request",
        "params": {
            "serverName": "externe",
            "requestedSchema": {
                "type": "object",
                "properties": { "secret": { "type": "string" } },
                "required": ["secret"]
            }
        }
    });

    let (reply, payload) = mcp_elicitation_response(&request).expect("réponse MCP");
    assert_eq!(reply["result"]["action"], "decline");
    assert!(reply["result"].get("content").is_none());
    assert_eq!(payload["code"], "mcp_elicitation_declined");
}
#[test]
fn edit159_promote_transport_codex_tour_actif_guide() {
    let root = root("edit159-active");
    let trace = root.join("trace.jsonl");
    let mut options = fake_options(&trace);
    options.notify_timeout_secs = 20;
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[
            (
                "BRIDGET_CODEX_TRACE".into(),
                trace.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_HOLD_TURN".into(), "1".into()),
        ],
        false,
    )
    .unwrap();
    transport.deliver(&message("active159")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut started = false;
    while Instant::now() < deadline && !started {
        started = transport.drain_events().iter().any(|event|
                matches!(&event.kind, ManagedEventKind::PromptDispatched { message_id } if message_id == "active159"));
        if !started {
            thread::sleep(Duration::from_millis(10));
        }
    }
    assert!(started, "tour fournisseur réellement ouvert");
    let mut promoted = message("promoted159");
    promoted.intent = Some(MessageIntent::SteerCurrent);
    promoted.references.push("steer-or-queue".into());
    transport.deliver(&promoted).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let trace = fs::read_to_string(&trace).unwrap_or_default();
        if trace
            .lines()
            .any(|line| line.contains("\"method\":\"turn/steer\"") && line.contains("promoted159"))
        {
            assert!(trace.contains("\"expectedTurnId\":\"turn-native\""));
            assert!(!trace.contains("\"method\":\"turn/interrupt\""));
            break;
        }
        assert!(
            Instant::now() < deadline,
            "le marqueur doit guider le tour actif"
        );
        thread::sleep(Duration::from_millis(10));
    }
    transport.stop();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn edit159_promote_transport_codex_sans_tour_enfile() {
    let root = root("edit159-idle");
    let trace = root.join("trace.jsonl");
    let mut options = fake_options(&trace);
    options.notify_timeout_secs = 20;
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[(
            "BRIDGET_CODEX_TRACE".into(),
            trace.to_string_lossy().into_owned(),
        )],
        false,
    )
    .unwrap();
    let mut promoted = message("promoted159");
    promoted.intent = Some(MessageIntent::SteerCurrent);
    promoted.references.push("steer-or-queue".into());
    transport.deliver(&promoted).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut finished = false;
    while Instant::now() < deadline && !finished {
        for event in transport.drain_events() {
            assert!(
                !matches!(&event.kind, ManagedEventKind::DeliveryRejected { message_id, .. } if message_id == "promoted159"),
                "pas de rejet quand le guidage manque"
            );
            finished |= matches!(&event.kind, ManagedEventKind::TurnFinished { message, .. } if message.id == "promoted159");
        }
        if !finished {
            thread::sleep(Duration::from_millis(10));
        }
    }
    assert!(finished, "le marqueur doit obtenir un tour FIFO propre");
    let trace = fs::read_to_string(&trace).unwrap();
    assert!(!trace.contains("\"method\":\"turn/steer\""));
    assert!(trace.contains("\"method\":\"turn/start\""));
    transport.stop();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn edit159_promote_transport_codex_steering_open_ferme_enfile() {
    let root = root("edit159-closed");
    let trace = root.join("trace.jsonl");
    let mut options = fake_options(&trace);
    options.notify_timeout_secs = 20;
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[
            (
                "BRIDGET_CODEX_TRACE".into(),
                trace.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_HOLD_TURN".into(), "1".into()),
        ],
        false,
    )
    .unwrap();
    transport.deliver(&message("active159")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut started = false;
    while Instant::now() < deadline && !started {
        started = transport.drain_events().iter().any(|event|
                matches!(&event.kind, ManagedEventKind::PromptDispatched { message_id } if message_id == "active159"));
        if !started {
            thread::sleep(Duration::from_millis(10));
        }
    }
    assert!(started, "tour fournisseur réellement ouvert");
    transport.queue.0.lock().unwrap().steering_open = false;
    transport.deliver(&message("older159")).unwrap();
    let mut promoted = message("promoted159");
    promoted.intent = Some(MessageIntent::SteerCurrent);
    promoted.references.push("steer-or-queue".into());
    promoted.origin = Some(MessageOrigin::Human);
    let source = promoted.clone();
    transport.deliver(&promoted).unwrap();
    {
        let queue = transport.queue.0.lock().unwrap();
        assert_eq!(
            queue
                .messages
                .iter()
                .map(|m| m.id.as_str())
                .collect::<Vec<_>>(),
            ["older159", "promoted159"]
        );
        assert!(queue.steer.is_empty());
        assert_eq!(queue.messages.back().unwrap().intent, None);
        assert_eq!(queue.messages.back().unwrap().body, source.body);
    }
    assert_eq!(
        promoted, source,
        "copie remise seulement, enveloppe appelante intacte"
    );
    let mut overflow = promoted.clone();
    overflow.id = "overflow159".into();
    transport.deliver(&overflow).unwrap();
    assert!(transport.drain_events().iter().any(
        |event| matches!(&event.kind, ManagedEventKind::DeliveryRejected { message_id, reason }
                if message_id == "overflow159" && reason == "file Codex pleine")
    ));
    assert_eq!(transport.queue.0.lock().unwrap().messages.len(), 2);
    assert!(
        !fs::read_to_string(&trace)
            .unwrap()
            .contains("\"method\":\"turn/interrupt\"")
    );
    transport.stop();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn edit159_promote_transport_sans_marqueur_inchange() {
    let root = root("edit159-legacy");
    let trace = root.join("trace.jsonl");
    let mut options = fake_options(&trace);
    options.notify_timeout_secs = 20;
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[
            (
                "BRIDGET_CODEX_TRACE".into(),
                trace.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_HOLD_TURN".into(), "1".into()),
        ],
        false,
    )
    .unwrap();
    let mut legacy = message("legacy159");
    legacy.intent = Some(MessageIntent::SteerCurrent);
    transport.deliver(&legacy).unwrap();
    assert!(transport.drain_events().iter().any(|event|
            matches!(&event.kind, ManagedEventKind::DeliveryRejected { message_id, reason }
                if message_id == "legacy159" && reason == "pilotage Codex indisponible pour le tour courant")));
    transport.deliver(&message("active159")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut started = false;
    while Instant::now() < deadline && !started {
        started = transport.drain_events().iter().any(|event|
                matches!(&event.kind, ManagedEventKind::PromptDispatched { message_id } if message_id == "active159"));
        if !started {
            thread::sleep(Duration::from_millis(10));
        }
    }
    assert!(started, "tour fournisseur réellement ouvert");
    transport.queue.0.lock().unwrap().steering_open = false;
    transport.deliver(&message("first159")).unwrap();
    transport.deliver(&message("second159")).unwrap();
    let mut promoted = message("full159");
    promoted.intent = Some(MessageIntent::SteerCurrent);
    promoted.references.push("steer-or-queue".into());
    transport.deliver(&promoted).unwrap();
    assert!(transport.drain_events().iter().any(
        |event| matches!(&event.kind, ManagedEventKind::DeliveryRejected { message_id, reason }
                if message_id == "full159" && reason == "file Codex pleine")
    ));
    let queue = transport.queue.0.lock().unwrap();
    assert_eq!(
        queue
            .messages
            .iter()
            .map(|m| m.id.as_str())
            .collect::<Vec<_>>(),
        ["first159", "second159"]
    );
    drop(queue);
    transport.stop();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn edit159_promote_transport_codex_erreur_ambigue_doublon_possible() {
    let root = root("edit159-ambiguous");
    let trace = root.join("trace.jsonl");
    let mut options = fake_options(&trace);
    options.notify_timeout_secs = 20;
    let applied = root.join("applied.jsonl");
    let script = &mut options.args[1];
    *script = script.replace("if [ \"${BRIDGET_CODEX_STEER_REFUSE:-0}\" = 1 ]; then", concat!(
            "if [ \"${BRIDGET_CODEX_STEER_REFUSE:-0}\" = 1 ]; then ",
            "printf '%s\\n' \"$line\" >> \"$BRIDGET_CODEX_APPLIED_TRACE\"; ",
            "steered=$(printf '%s' \"$line\" | sed 's/.*\"clientUserMessageId\":\"\\([^\"]*\\)\".*/\\1/'); ",
            "printf '%s\\n' \"{\\\"method\\\":\\\"item/completed\\\",\\\"params\\\":{\\\"threadId\\\":\\\"thread-native\\\",\\\"turnId\\\":\\\"turn-native\\\",\\\"item\\\":{\\\"id\\\":\\\"applied159\\\",\\\"clientId\\\":\\\"$steered\\\",\\\"type\\\":\\\"userMessage\\\",\\\"content\\\":[]}}}\"; "));
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[
            (
                "BRIDGET_CODEX_TRACE".into(),
                trace.to_string_lossy().into_owned(),
            ),
            (
                "BRIDGET_CODEX_APPLIED_TRACE".into(),
                applied.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_HOLD_TURN".into(), "1".into()),
            ("BRIDGET_CODEX_STEER_REFUSE".into(), "1".into()),
        ],
        false,
    )
    .unwrap();
    transport.deliver(&message("active159")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut started = false;
    while Instant::now() < deadline && !started {
        started = transport.drain_events().iter().any(|event|
                matches!(&event.kind, ManagedEventKind::PromptDispatched { message_id } if message_id == "active159"));
        if !started {
            thread::sleep(Duration::from_millis(10));
        }
    }
    assert!(started, "tour fournisseur réellement ouvert");
    // Geler uniquement le prochain retrait FIFO pour examiner le repli committé en mémoire.
    transport.queue.0.lock().unwrap().external_turn = Some("fixture-next-turn".into());
    let mut promoted = message("promoted159");
    promoted.intent = Some(MessageIntent::SteerCurrent);
    promoted.references.push("steer-or-queue".into());
    transport.deliver(&promoted).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while transport.queue.0.lock().unwrap().steering_open {
        assert!(Instant::now() < deadline, "refus fournisseur réel attendu");
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        fs::read_to_string(&applied)
            .unwrap()
            .contains("promoted159"),
        "le fournisseur a appliqué avant de renvoyer l'erreur"
    );
    {
        let mut writer = transport.writer.lock().unwrap();
        let writer = writer.as_mut().unwrap();
        writeln!(writer, "{{\"method\":\"native155/finish\"}}").unwrap();
        writer.flush().unwrap();
    }
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let queue = transport.queue.0.lock().unwrap();
        if queue.messages.iter().any(|m| m.id == "promoted159") {
            break;
        }
        drop(queue);
        assert!(
            Instant::now() < deadline,
            "message accepté restitué à la FIFO"
        );
        thread::sleep(Duration::from_millis(10));
    }
    transport.queue.0.lock().unwrap().external_turn = None;
    transport.queue.1.notify_one();
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let content = fs::read_to_string(&trace).unwrap_or_default();
        if content
            .lines()
            .any(|line| line.contains("\"method\":\"turn/start\"") && line.contains("promoted159"))
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "le repli émet un nouveau tour avec le même message"
        );
        thread::sleep(Duration::from_millis(10));
    }
    transport.stop();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn edit159_promote_transport_codex_refus_apres_acceptation_enfile() {
    let root = root("edit159-refused");
    let trace = root.join("trace.jsonl");
    let mut options = fake_options(&trace);
    options.notify_timeout_secs = 20;
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[
            (
                "BRIDGET_CODEX_TRACE".into(),
                trace.to_string_lossy().into_owned(),
            ),
            ("BRIDGET_CODEX_HOLD_TURN".into(), "1".into()),
            ("BRIDGET_CODEX_STEER_REFUSE".into(), "1".into()),
        ],
        false,
    )
    .unwrap();
    transport.deliver(&message("active159")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut started = false;
    while Instant::now() < deadline && !started {
        started = transport.drain_events().iter().any(|event|
                matches!(&event.kind, ManagedEventKind::PromptDispatched { message_id } if message_id == "active159"));
        if !started {
            thread::sleep(Duration::from_millis(10));
        }
    }
    assert!(started, "tour fournisseur réellement ouvert");
    // Geler uniquement le prochain retrait FIFO pour examiner le repli committé en mémoire.
    transport.queue.0.lock().unwrap().external_turn = Some("fixture-next-turn".into());
    let mut promoted = message("promoted159");
    promoted.intent = Some(MessageIntent::SteerCurrent);
    promoted.references.push("steer-or-queue".into());
    transport.deliver(&promoted).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while transport.queue.0.lock().unwrap().steering_open {
        assert!(Instant::now() < deadline, "refus fournisseur réel attendu");
        thread::sleep(Duration::from_millis(10));
    }
    {
        let mut writer = transport.writer.lock().unwrap();
        let writer = writer.as_mut().unwrap();
        writeln!(writer, "{{\"method\":\"native155/finish\"}}").unwrap();
        writer.flush().unwrap();
    }
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let queue = transport.queue.0.lock().unwrap();
        if queue.messages.iter().any(|m| m.id == "promoted159") {
            break;
        }
        drop(queue);
        assert!(
            Instant::now() < deadline,
            "message accepté restitué à la FIFO"
        );
        thread::sleep(Duration::from_millis(10));
    }
    transport.queue.0.lock().unwrap().external_turn = None;
    transport.queue.1.notify_one();
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let content = fs::read_to_string(&trace).unwrap_or_default();
        if content
            .lines()
            .any(|line| line.contains("\"method\":\"turn/start\"") && line.contains("promoted159"))
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "le repli émet un nouveau tour avec le même message"
        );
        thread::sleep(Duration::from_millis(10));
    }
    transport.stop();
    fs::remove_dir_all(root).unwrap();
}
fn repair185_options(trace: &Path) -> CodexAppServerOptions {
    let mut options = fake_options(trace);
    options.provider_observation = Some(ProviderObservation {
        binary_path: "/fixtures/codex".into(),
        binary_version: "fixture185".into(),
        binary_digest: "0".repeat(64),
        contract_version: "codex-app-server-fixture-v1".into(),
        operations: vec![ProviderOperation::Resume],
    });
    options
}

#[test]
fn spec185_workspace_repair_refuses_nonempty_admission_without_stopping() {
    let root = root("repair185-busy");
    let trace = root.join("trace.jsonl");
    for case in ["queued", "steer", "external", "unread-event", "approval"] {
        let mut transport = CodexAppServerTransport::spawn_with_environment(
            repair185_options(&trace),
            &[(
                "BRIDGET_CODEX_TRACE".into(),
                trace.to_string_lossy().into_owned(),
            )],
            false,
        )
        .unwrap();
        if case != "unread-event" {
            transport.drain_events();
        }
        {
            let mut queue = transport.queue.0.lock().unwrap();
            match case {
                "queued" => queue.messages.push_back(message("not-dispatched185")),
                "steer" => queue.steer.push_back(message("not-steered185")),
                "external" => queue.external_turn = Some("human-turn185".into()),
                _ => {}
            }
        }
        if case == "approval" {
            *transport.pending_request.lock().unwrap() = Some(PendingProviderRequest {
                message_id: None,
                method: "item/commandExecution/requestApproval".into(),
                request_id: "approval185".into(),
                turn_id: None,
            });
        }
        let result = transport.stop_for_workspace_repair(Duration::from_secs(1));
        assert!(
            matches!(result, Err(TransportError::DeliveryFailed(ref code))
                if code == "workspace_repair_busy"),
            "{case}: {result:?}"
        );
        assert!(transport.alive.load(Ordering::SeqCst));
        assert!(!transport.queue.0.lock().unwrap().closed);
        {
            let mut queue = transport.queue.0.lock().unwrap();
            queue.messages.clear();
            queue.steer.clear();
            queue.external_turn = None;
        }
        transport.pending_request.lock().unwrap().take();
        transport.stop();
    }
    std::fs::remove_dir_all(root).unwrap();
}

/// « Occupé » est une réponse passagère : des événements de démarrage
/// peuvent encore arriver sous charge. Réessayer jusqu'à une réponse stable.
fn repair185_settled(
    transport: &mut CodexAppServerTransport,
    budget: Duration,
) -> Result<ManagedProviderIdentity, TransportError> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        transport.drain_events();
        match transport.stop_for_workspace_repair(budget) {
            Err(TransportError::DeliveryFailed(code))
                if code == "workspace_repair_busy" && Instant::now() < deadline =>
            {
                thread::sleep(Duration::from_millis(20));
            }
            result => return result,
        }
    }
}

#[test]
fn spec185_workspace_repair_stops_exact_idle_provider_and_keeps_thread() {
    let root = root("repair185-idle");
    let trace = root.join("trace.jsonl");
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        repair185_options(&trace),
        &[(
            "BRIDGET_CODEX_TRACE".into(),
            trace.to_string_lossy().into_owned(),
        )],
        false,
    )
    .unwrap();
    transport.drain_events();
    let pid = transport.process_id();
    let identity = repair185_settled(&mut transport, Duration::from_secs(2)).unwrap();
    assert_eq!(
        identity.provider_thread_id.as_deref(),
        Some("thread-native")
    );
    assert!(
        transport
            .child
            .lock()
            .unwrap()
            .try_wait()
            .unwrap()
            .is_some()
    );
    // SAFETY: kill(pid, 0) est l'idiome POSIX de sonde d'existence ; aucun signal n'est envoyé.
    assert!(unsafe { libc::kill(-(pid as i32), 0) } < 0);
    assert!(transport.queue.0.lock().unwrap().closed);
    assert!(!transport.alive.load(Ordering::SeqCst));
    assert!(
        !std::fs::read_to_string(&trace)
            .unwrap()
            .contains("\"method\":\"turn/start\"")
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec185_workspace_repair_requires_attested_resume_before_stopping() {
    let root = root("repair185-no-resume");
    let trace = root.join("trace.jsonl");
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        fake_options(&trace),
        &[(
            "BRIDGET_CODEX_TRACE".into(),
            trace.to_string_lossy().into_owned(),
        )],
        false,
    )
    .unwrap();
    transport.drain_events();
    assert!(
        matches!(repair185_settled(&mut transport, Duration::from_secs(1)),
            Err(TransportError::DeliveryFailed(ref code)) if code == "workspace_repair_resume_unattested")
    );
    assert!(transport.alive.load(Ordering::SeqCst));
    assert!(!transport.queue.0.lock().unwrap().closed);
    transport.stop();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec185_workspace_repair_uncertain_stop_is_bounded_and_never_claims_success() {
    let root = root("repair185-resistant");
    let trace = root.join("trace.jsonl");
    let mut options = repair185_options(&trace);
    let shell = options.args.last_mut().unwrap();
    *shell = format!("trap '' TERM INT\n{shell}\n/bin/sleep 0.3\n");
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        options,
        &[(
            "BRIDGET_CODEX_TRACE".into(),
            trace.to_string_lossy().into_owned(),
        )],
        false,
    )
    .unwrap();
    transport.drain_events();
    let start = Instant::now();
    let result = transport.stop_for_workspace_repair(Duration::from_millis(60));
    assert!(
        matches!(result,Err(TransportError::DeliveryFailed(ref code))
            if code=="workspace_repair_stop_unconfirmed"),
        "{result:?}"
    );
    assert!(start.elapsed() < Duration::from_millis(250));
    assert!(transport.queue.0.lock().unwrap().closed);
    assert!(!transport.alive.load(Ordering::SeqCst));
    // La fixture s'arrête seule ; aucun SIGKILL ni processus étranger.
    let deadline = Instant::now() + Duration::from_secs(2);
    while transport
        .child
        .lock()
        .unwrap()
        .try_wait()
        .unwrap()
        .is_none()
    {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    }
    for handle in [&transport.reader_handle, &transport.worker_handle] {
        if let Some(thread) = handle.lock().unwrap().take() {
            thread.join().unwrap();
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec185_repair_does_not_wait_for_provider_request_lock() {
    let root = root("repair185-lock-order");
    let trace = root.join("trace.jsonl");
    let mut transport = CodexAppServerTransport::spawn_with_environment(
        repair185_options(&trace),
        &[(
            "BRIDGET_CODEX_TRACE".into(),
            trace.to_string_lossy().into_owned(),
        )],
        false,
    )
    .unwrap();
    transport.drain_events();
    let pending = transport.pending_request.clone();
    let held = pending.lock().unwrap();
    let (sender, receiver) = mpsc::channel();
    let thread = std::thread::spawn(move || {
        let start = Instant::now();
        let result = transport.stop_for_workspace_repair(Duration::from_millis(60));
        sender.send((start.elapsed(), result)).unwrap();
        transport
    });
    let observed = receiver.recv_timeout(Duration::from_millis(200));
    // Libérer même en échec : le témoin ancien ne bloque jamais la suite.
    drop(held);
    let transport = thread.join().unwrap();
    assert!(
        observed.is_ok(),
        "l'arrêt attendait le verrou détenu par le lecteur"
    );
    let (elapsed, result) = observed.unwrap();
    assert!(elapsed < Duration::from_millis(150));
    assert!(
        matches!(result,Err(TransportError::DeliveryFailed(ref code)) if code=="workspace_repair_busy")
    );
    assert!(transport.is_alive());
    transport.stop();
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec185_failed_resume_keeps_owned_process_and_bounds_reader_cleanup() {
    let root = root("repair185-failed-resume");
    let trace = root.join("trace.jsonl");
    let mut options = repair185_options(&trace);
    options.command = "/usr/bin/python3".into();
    options.args=vec!["-u".into(),"-c".into(),r#"
import json,sys,signal,subprocess,time
signal.signal(signal.SIGTERM,signal.SIG_IGN)
signal.signal(signal.SIGINT,signal.SIG_IGN)
for line in sys.stdin:
 q=json.loads(line)
 with open(sys.argv[1],"a") as log: log.write(line)
 if "id" not in q: continue
 m=q.get("method");r={}
 if m=="initialize":r={"userAgent":"fixture185","codexHome":"/tmp","platformFamily":"unix","platformOs":"linux"}
 if m=="config/read":r={"config":{"approval_policy":"never","sandbox_mode":"read-only"}}
 if m=="thread/resume":
  subprocess.Popen(["/bin/sleep","7"])
  r={"thread":{"id":"foreign-thread185"}}
 print(json.dumps({"id":q["id"],"result":r}),flush=True)
 if m=="thread/resume":time.sleep(7);break
"#.into(),trace.to_string_lossy().into_owned()];
    options.thread_bootstrap = CodexThreadBootstrap::Resume {
        thread_id: "expected185".into(),
    };
    let started = Instant::now();
    let result = CodexAppServerTransport::spawn_repair_with_environment(options, &[], false);
    // La sonde account/rateLimits volontairement sans réponse coûte 2 s ;
    // le nettoyage possède ensuite sa borne propre de 4 s.
    assert!(
        started.elapsed() < Duration::from_secs(7),
        "reprise et nettoyage hors bornes : {:?}",
        started.elapsed()
    );
    let mut owner = match result {
        Err(TransportError::StartUnconfirmed { owner, .. }) => owner,
        Err(error) => panic!("propriétaire non conservé : {error:?}"),
        Ok(_) => panic!("reprise étrangère acceptée"),
    };
    let pid = owner.process_id();
    assert!(pid > 0);
    owner.stop(Duration::from_secs(5)).unwrap();
    let rpc = std::fs::read_to_string(trace).unwrap();
    assert_eq!(
        rpc.lines()
            .filter(|v| v.contains("\"method\":\"thread/resume\""))
            .count(),
        1
    );
    assert!(!rpc.contains("\"method\":\"turn/start\""));
    std::fs::remove_dir_all(root).unwrap();
}
