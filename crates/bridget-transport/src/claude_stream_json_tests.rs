#[test]
fn spec100_claude_requires_success_and_keeps_path_before_truncation() {
    let mut pending = std::collections::HashMap::new();
    let tool = serde_json::json!({"type":"assistant","message":{"content":[{"type":"tool_use","id":"w","name":"Write","input":{"content":"x".repeat(2000),"file_path":"/project/x"}}]}});
    assert!(super::confirmed_tool_writes(&tool, &mut pending).is_empty());
    let done = serde_json::json!({"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"w","is_error":false}]}});
    assert_eq!(
        super::confirmed_tool_writes(&done, &mut pending),
        vec!["/project/x"]
    );
    assert!(super::confirmed_tool_writes(&done, &mut pending).is_empty());
    super::confirmed_tool_writes(&tool, &mut pending);
    assert!(
        super::confirmed_tool_writes(&done, &mut pending).is_empty(),
        "une répétition assistant/résultat ne recrée pas l'écriture"
    );
    super::confirmed_tool_writes(&serde_json::json!({"type":"result"}), &mut pending);
    super::confirmed_tool_writes(&tool, &mut pending);
    let failed = serde_json::json!({"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"w","is_error":true}]}});
    assert!(super::confirmed_tool_writes(&failed, &mut pending).is_empty());
    assert!(super::confirmed_tool_writes(&done, &mut pending).is_empty());
}
use super::*;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::Instant;

static FIXTURE_SEQ: AtomicU64 = AtomicU64::new(0);

#[test]
fn spec105_claude_trame_transmet_le_contrat_et_le_corps() {
    for requested in [false, true] {
        let mut child = Command::new("/bin/cat")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let writer = Arc::new(Mutex::new(child.stdin.take()));
        let mut message = BridgetMessage::new("alice", "bob", "Texte utile\nligne suivante");
        message.reply = requested;
        message.in_reply_to = Some("parent".into());
        write_input(&writer, &message, Some("Instruction privée")).unwrap();
        drop(writer);
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        let frame: Value = serde_json::from_slice(&output.stdout).unwrap();
        let prompt = frame["message"]["content"][0]["text"].as_str().unwrap();
        assert!(prompt.contains("Instruction privée"));
        assert!(prompt.contains(&message.id));
        assert!(prompt.contains("alice"));
        assert!(prompt.contains("parent"));
        assert!(prompt.ends_with(&message.body));
        assert_eq!(prompt.contains("Pas d'accusé de réception"), !requested);
        assert_eq!(prompt.contains("relaie automatiquement"), requested);
    }
}

#[test]
fn spec174_claude_taches_bornees_et_notifications_correlees() {
    let mut tasks = BackgroundTasks::default();
    let start = json!({"type":"system","subtype":"task_started","task_id":"a"});
    tasks.observe(&start);
    tasks.observe(&start);
    tasks.observe(
        &json!({"type":"system","subtype":"task_notification","task_id":"b","status":"completed"}),
    );
    assert_eq!(tasks.live.len(), 1);
    tasks.observe(&json!({"type":"system","subtype":"task_updated","task_id":"a","patch":{"status":"paused"}}));
    assert!(tasks.pending());
    tasks.observe(&json!({"type":"system","subtype":"task_updated","task_id":"a","patch":{"status":"killed"}}));
    assert!(!tasks.pending());
    tasks.observe(
        &json!({"type":"system","subtype":"task_started","task_id":"watcher","ambient":true}),
    );
    assert!(!tasks.pending());
    tasks.observe(&json!({"type":"system","subtype":"task_progress","task_id":"a"}));
    tasks.observe(&json!({"type":"system","subtype":"task_updated","task_id":"a","patch":{"status":"running"}}));
    tasks.observe(&json!({"type":"system","subtype":"task_progress","task_id":"watcher"}));
    assert!(
        !tasks.pending(),
        "une progression tardive ne recrée aucune tâche"
    );
    tasks.observe(&start);
    tasks.observe(&json!({"type":"system","subtype":"task_notification","task_id":"a","status":"completed","skip_transcript":true}));
    assert!(
        !tasks.pending(),
        "une fin masquée clôt aussi une tâche suivie"
    );
    for n in 0..1025 {
        tasks.observe(&json!({"type":"system","subtype":"task_started","task_id":n.to_string()}));
    }
    assert_eq!(tasks.live.len(), 1024);
    assert!(tasks.unconfirmed);
    tasks.live.clear();
    assert!(
        tasks.pending(),
        "un inventaire tronqué ne prouve pas la fin"
    );
    let mut invalid = BackgroundTasks::default();
    invalid.observe(&json!({"type":"system","subtype":"task_started","task_id":"x".repeat(257)}));
    assert!(invalid.pending());
    assert!(invalid.live.is_empty());
}

#[test]
fn spec174_claude_fond_conserve_annulation_echec_et_echeance() {
    for reason in ["aborted_tools", "api_error", "completed"] {
        let frames = [
            json!({"type":"system","subtype":"task_started","task_id":"running"}),
            json!({"type":"result","terminal_reason":reason,"is_error":reason == "api_error","result":"interim"}),
        ];
        let output = frames
            .iter()
            .map(|frame| format!("printf '%s\\n' '{}'; ", frame))
            .collect::<String>();
        let mut configured = options();
        configured.args[1] = format!("IFS= read -r line; {output} IFS= read -r line");
        configured.notify_timeout_secs = 1;
        let mut transport = ClaudeStreamJsonTransport::spawn(configured).unwrap();
        transport.deliver(&message("background-terminal")).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        let terminal = loop {
            let found = transport
                .drain_events()
                .into_iter()
                .find_map(|e| match e.kind {
                    ManagedEventKind::TurnFinished { terminal, .. } => Some(terminal),
                    ManagedEventKind::DeliveryRejected { reason, .. } => {
                        Some(ManagedTerminal::Failed { detail: reason })
                    }
                    _ => None,
                });
            if let Some(terminal) = found {
                break terminal;
            }
            assert!(Instant::now() < deadline, "échéance perdue");
            thread::sleep(Duration::from_millis(10));
        };
        match reason {
            "aborted_tools" => assert!(matches!(terminal, ManagedTerminal::Cancelled)),
            "api_error" => assert!(
                matches!(terminal, ManagedTerminal::Failed { detail } if detail == "api_error")
            ),
            _ => assert!(
                matches!(terminal, ManagedTerminal::Failed { detail } if detail == "échéance Claude dépassée")
            ),
        }
        transport.stop();
    }
}

#[test]
fn spec174_claude_attend_le_rapport_apres_le_travail_de_fond() {
    let root = std::env::temp_dir().join(format!(
        "bridget-background-{}-{}",
        std::process::id(),
        FIXTURE_SEQ.fetch_add(1, AtomicOrdering::Relaxed)
    ));
    fs::create_dir_all(&root).unwrap();
    let script = r#"
import json, pathlib, sys, time
root = pathlib.Path(sys.argv[1])
def emit(value):
    print(json.dumps(value), flush=True)
def gate(n):
    emit({'type':'system','subtype':'status','barrier':n})
    while not (root / str(n)).exists():
        time.sleep(.01)
def result(text):
    emit({'type':'result','is_error':False,'terminal_reason':'completed','result':text})
sys.stdin.readline()
emit({'type':'system','subtype':'task_started','task_id':'calculation','task_type':'local_bash','is_backgrounded':True})
result('calcul en cours, rapport a venir')
gate(1)
emit({'type':'system','subtype':'task_notification','task_id':'foreign','status':'completed'})
result('toujours en cours')
gate(2)
emit({'type':'system','subtype':'task_notification','task_id':'calculation','status':'completed'})
gate(3)
result('rapport final, commit termine')
sys.stdin.readline()
"#;
    let mut configured = options();
    configured.command = "python3".into();
    configured.args = vec![
        "-u".into(),
        "-c".into(),
        script.into(),
        root.to_string_lossy().into_owned(),
    ];
    configured.notify_timeout_secs = 10;
    let mut transport = ClaudeStreamJsonTransport::spawn(configured).unwrap();
    transport
        .activate_journal(&root, "background271", None)
        .unwrap();
    transport.deliver(&message("background-mission")).unwrap();
    let mut events = Vec::new();
    for stage in 1..=3 {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            events.extend(transport.drain_events());
            if events.iter().any(|e| {
                serde_json::from_slice::<Value>(&e.raw)
                    .ok()
                    .is_some_and(|v| v["barrier"] == stage)
            }) {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "barrière fournisseur {stage} absente"
            );
            thread::sleep(Duration::from_millis(10));
        }
        thread::sleep(Duration::from_millis(50));
        events.extend(transport.drain_events());
        assert!(
            transport.is_busy(),
            "mission publiée avant le rapport final, étape {stage}"
        );
        assert!(
            !events.iter().any(|e| matches!(
                e.kind,
                ManagedEventKind::TurnFinished { .. } | ManagedEventKind::DeliveryRejected { .. }
            )),
            "résultat prématuré, étape {stage}"
        );
        fs::write(root.join(stage.to_string()), b"continue").unwrap();
    }
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        events.extend(transport.drain_events());
        if events
            .iter()
            .any(|e| matches!(e.kind, ManagedEventKind::TurnFinished { .. }))
        {
            break;
        }
        assert!(Instant::now() < deadline, "rapport final absent");
        thread::sleep(Duration::from_millis(10));
    }
    let results: Vec<_> = events
        .iter()
        .filter_map(|e| match &e.kind {
            ManagedEventKind::TurnFinished {
                message,
                response,
                terminal: ManagedTerminal::Completed,
            } => Some((message.id.as_str(), response.as_str())),
            _ => None,
        })
        .collect();
    assert_eq!(
        results,
        vec![(
            "background-mission",
            "calcul en cours, rapport a venir\n\ntoujours en cours\n\nrapport final, commit termine"
        )]
    );
    transport.stop();
    let path = fs::read_dir(root.join("background271"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let records = crate::journal::valid_events(&path);
    let results: Vec<_> = records
        .iter()
        .filter(|r| r["event"] == "provider_result")
        .collect();
    assert_eq!(results.len(), 3);
    assert_eq!(
        results[0]["payload"]["finalization"],
        "waiting_background_tasks"
    );
    assert_eq!(
        results[1]["payload"]["finalization"],
        "waiting_background_tasks"
    );
    assert_eq!(results[2]["payload"]["finalization"], "terminal_result");
    assert_eq!(
        results[0]["payload"]["background_tasks"],
        json!({"live_count":1,"unconfirmed":false})
    );
    assert_eq!(
        results[2]["payload"]["background_tasks"],
        json!({"live_count":0,"unconfirmed":false})
    );
    assert!(
        results
            .iter()
            .all(|r| r["message_id"] == "background-mission")
    );
    fs::remove_dir_all(root).unwrap();
}

fn options() -> ClaudeStreamJsonOptions {
    ClaudeStreamJsonOptions {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                concat!(
                    "while IFS= read -r line; do ",
                    "printf '%s\\n' '{\"type\":\"system\",\"subtype\":\"status\",\"status\":\"requesting\"}'; ",
                    "printf '%s\\n' '{\"type\":\"stream_event\",\"event\":{\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"réponse Claude\"}}}'; ",
                    "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",\"result\":\"réponse Claude\"}'; ",
                    "done"
                )
                .to_string(),
            ],
            provider_kind: "claude".to_string(),
            queue_capacity: 2,
            notify_timeout_secs: 2,
            provider_observation: None,
            session_store_root: None,
            agent_name: None,
        }
}

#[test]
fn native174_claude_echeance_explicite_depasse_defaut_fournisseur() {
    let mut configured = options();
    configured.notify_timeout_secs = 1;
    configured.args[1] = format!("sleep 2; {}", configured.args[1]);
    let mut transport = ClaudeStreamJsonTransport::spawn(configured).unwrap();
    let mut request = message("native-deadline174");
    request.deadline_at = Some(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 5,
    );
    transport.deliver(&request).unwrap();
    let until = Instant::now() + Duration::from_secs(4);
    loop {
        if let Some(terminal) =
            transport
                .drain_events()
                .into_iter()
                .find_map(|event| match event.kind {
                    ManagedEventKind::TurnFinished { terminal, .. } => Some(terminal),
                    ManagedEventKind::DeliveryRejected { reason, .. } => {
                        Some(ManagedTerminal::Failed { detail: reason })
                    }
                    _ => None,
                })
        {
            assert!(
                matches!(terminal, ManagedTerminal::Completed),
                "{terminal:?}"
            );
            break;
        }
        assert!(Instant::now() < until, "résultat de la fixture absent");
        thread::sleep(Duration::from_millis(10));
    }
    transport.stop();
}

#[test]
fn provider_kind_reste_celui_declare_par_le_registre() {
    for provider_kind in ["claude", "anthropic", "glm", "deepseek"] {
        let mut configured = options();
        configured.provider_kind = provider_kind.to_string();
        let transport = ClaudeStreamJsonTransport::spawn(configured).unwrap();
        let contexts = transport
            .drain_events()
            .into_iter()
            .filter_map(|event| match event.kind {
                ManagedEventKind::ProviderContextObserved { identity } => {
                    Some(identity.provider_kind)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(contexts, vec![provider_kind.to_string()]);
        assert_eq!(
            transport.provider_identity().unwrap().provider_kind,
            provider_kind
        );
        transport.stop();
    }
}

fn message(id: &str) -> BridgetMessage {
    let mut message = BridgetMessage::new("bridget", "claude", "mission");
    message.id = id.to_string();
    message.reply = true;
    message
}

#[test]
#[allow(non_snake_case)]
fn TEMOIN_claude_refus_update_hors_vocabulaire_emet_JournalFailed() {
    // Preuve d'abord : écriture valide passe sans JournalFailed.
    let root = std::env::temp_dir().join(format!(
        "bridget-claude-act-kind-{}-{}",
        std::process::id(),
        FIXTURE_SEQ.fetch_add(1, AtomicOrdering::SeqCst)
    ));
    fs::create_dir_all(&root).unwrap();
    let events = Arc::new(Mutex::new(VecDeque::new()));
    let journal = Arc::new(Mutex::new(Some(
        JournalWriter::start(
            &root,
            "claude-1",
            "session-1",
            Arc::new(Mutex::new(crate::acp::AcpEventQueue::default())),
        )
        .unwrap(),
    )));
    record_or_terminal(
        &journal,
        &events,
        "update",
        Some("m1"),
        json!({"kind":"tool","text":"Read"}),
    );
    assert!(
        events.lock().unwrap().is_empty(),
        "écriture tool valide ne doit pas émettre JournalFailed"
    );
    record_or_terminal(
        &journal,
        &events,
        "update",
        Some("m1"),
        json!({"kind":"intent","text":"fantôme"}),
    );
    let drained: Vec<_> = events.lock().unwrap().drain(..).collect();
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

/// Dette 3 — oracle bout-en-bout : le PILOTE (tool_use → record_tool_uses)
/// écrit un kind interdit ; l'événement terminal JournalFailed apparaît
/// dans drain_events. Pas un appel direct à record_or_terminal.
#[test]
#[allow(non_snake_case)]
fn TEMOIN_claude_pilote_kind_interdit_emet_JournalFailed_bout_en_bout() {
    // Preuve d'abord : cas valide → tool au journal, PAS de JournalFailed.
    let (events_ok, journal_ok) = {
        let _suite = crate::act_kind::pilot_kind_suite_lock();
        claude_tool_turn_managed_events()
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
            .any(|event| { event["event"] == "update" && event["payload"]["kind"] == "tool" }),
        "cas valide doit journaliser kind=tool, got {journal_ok:?}"
    );

    // Chemin réel : provider tool_use → pilote → enqueue refuse → JournalFailed.
    let (events_bad, journal_bad) =
        crate::act_kind::with_forced_pilot_update_kind("intent", claude_tool_turn_managed_events);
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
            .all(|event| { event["payload"]["kind"] != "intent" }),
        "le kind refusé ne doit jamais atteindre le JSONL, got {journal_bad:?}"
    );
}

fn claude_tool_turn_managed_events() -> (Vec<ManagedEvent>, Vec<Value>) {
    let root = std::env::temp_dir().join(format!(
        "bridget-claude-e2e-act-{}-{}",
        std::process::id(),
        FIXTURE_SEQ.fetch_add(1, AtomicOrdering::Relaxed)
    ));
    fs::create_dir_all(&root).unwrap();
    let mut opts = options();
    opts.args[1] = concat!(
            "while IFS= read -r line; do ",
            "printf '%s\\n' '{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"tool_use\",\"id\":\"toolu_demo\",\"name\":\"Read\",\"input\":{\"file_path\":\"/tmp/demo.toml\",\"limit\":1}}]}}'; ",
            "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"subtype\":\"success\",\"stop_reason\":\"end_turn\",\"result\":\"ok\"}'; ",
            "done"
        )
        .to_string();
    let mut transport = ClaudeStreamJsonTransport::spawn(opts).unwrap();
    transport
        .activate_journal(&root, "claude-e2e", None)
        .unwrap();
    transport.deliver(&message("claude-e2e-1")).unwrap();
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
    // Drainer le reste (JournalFailed peut arriver juste avant/après fin).
    events.extend(transport.drain_events());
    transport.stop();
    let journal_path = fs::read_dir(root.join("claude-e2e"))
        .expect("répertoire journal")
        .next()
        .expect("fichier journal")
        .expect("entrée journal")
        .path();
    let journal_events = crate::journal::valid_events(&journal_path);
    let proof = journal_events
        .iter()
        .find(|event| event["event"] == "provider_result")
        .expect("métadonnées terminales corrélées");
    assert_eq!(proof["message_id"], "claude-e2e-1");
    assert_eq!(proof["payload"]["subtype"], "success");
    assert_eq!(proof["payload"]["terminal_reason"], "missing");
    assert_eq!(proof["payload"]["outcome"], "completed");
    assert!(proof["payload"].get("result").is_none());
    let _ = fs::remove_dir_all(&root);
    (events, journal_events)
}

#[test]
#[allow(non_snake_case)]
fn mutant_avale_err_record_tue_TEMOIN_claude_JournalFailed() {
    fn broken_swallow(
        journal: &Journal,
        _events: &Arc<Mutex<VecDeque<ManagedEvent>>>,
        event: &str,
        message_id: Option<&str>,
        payload: Value,
    ) {
        let _ = record(journal, event, message_id, payload);
    }
    let root = std::env::temp_dir().join(format!(
        "bridget-claude-act-kind-mutant-{}-{}",
        std::process::id(),
        FIXTURE_SEQ.fetch_add(1, AtomicOrdering::SeqCst)
    ));
    fs::create_dir_all(&root).unwrap();
    let events = Arc::new(Mutex::new(VecDeque::new()));
    let journal = Arc::new(Mutex::new(Some(
        JournalWriter::start(
            &root,
            "claude-1",
            "session-1",
            Arc::new(Mutex::new(crate::acp::AcpEventQueue::default())),
        )
        .unwrap(),
    )));
    broken_swallow(
        &journal,
        &events,
        "update",
        Some("m1"),
        json!({"kind":"intent"}),
    );
    assert!(
        events.lock().unwrap().is_empty(),
        "le mutant avale l'erreur"
    );
    record_or_terminal(
        &journal,
        &events,
        "update",
        Some("m1"),
        json!({"kind":"intent"}),
    );
    assert!(
        events.lock().unwrap().iter().any(|event| matches!(
            &event.kind,
            ManagedEventKind::JournalFailed { detail }
                if detail.contains("hors vocabulaire")
        )),
        "TEMOIN_claude_JournalFailed doit mourir si Err est avalée"
    );
    if let Some(writer) = journal.lock().unwrap().take() {
        writer.stop();
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn session_native_preserve_les_lignes_et_termine_le_tour() {
    let root = std::env::temp_dir().join(format!(
        "bridget-claude-native-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    let mut transport = ClaudeStreamJsonTransport::spawn(options()).unwrap();
    transport
        .activate_journal(&root, "claude-native", None)
        .unwrap();
    transport.deliver(&message("claude-1")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut events = Vec::new();
    while Instant::now() < deadline {
        events.extend(transport.drain_events());
        if events.iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::TurnFinished {
                    ref response,
                    terminal: ManagedTerminal::Completed,
                    ..
                } if response == "réponse Claude"
            )
        }) {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(events.iter().any(|event| {
            matches!(event.kind, ManagedEventKind::PromptDispatched { ref message_id } if message_id == "claude-1")
        }));
    let raw = events
        .iter()
        .find(|event| event.raw.starts_with(br#"{"type":"stream_event""#))
        .expect("notification stream-event source");
    assert_eq!(raw.source, ManagedEventSource::ClaudeStreamJson);
    assert_eq!(raw.origin, ManagedEventOrigin::SourceLine);
    // Même ligne que le fournisseur historique dfa2134, sans sa seule
    // terminaison LF. Une reconstruction sémantique ne suffit pas.
    assert_eq!(
        raw.raw.as_slice(),
        include_bytes!(
            "../../../specs/089-communication-core/contracts/fixtures/native-claude-delta.jsonl"
        )
        .strip_suffix(b"\n")
        .expect("fixture JSONL terminée par LF")
    );
    transport.stop();
    let _ = fs::remove_dir_all(root);
}

/// (A) Témoin retranscription : deltas → `update` journal avec TEXTE EXACT.
/// Mutant : retirer `record(..., "update", ...)` sur `/event/delta/text`,
/// ou vider `content` (`content:""`) — A doit mourir (présence ≠ propriété).
#[allow(non_snake_case)]
#[test]
fn TEMOIN_A_claude_stream_json_retranscrit_les_deltas_en_update() {
    let events = journal_fixture_deltas();
    let updates: Vec<_> = events
        .iter()
        .filter(|event| event["event"] == "update")
        .collect();
    assert!(
        updates
            .iter()
            .all(|event| event["payload"]["kind"] == "text")
    );
    let contents: Vec<&str> = updates
        .iter()
        .filter_map(|event| event["payload"]["content"].as_str())
        .collect();
    // Contenu en dur (comme B) : cardinal+kind seuls laissent passer content:"".
    assert_eq!(
        contents,
        vec!["BONJOUR ", "JE SUIS VIVANT"],
        "deux deltas exacts, pas de troisième update issu du result: {contents:?} via {events:?}"
    );
    assert!(
        updates
            .iter()
            .all(|event| event["message_id"] == "claude-abc-1"),
        "même message_id que le tour, reçu {updates:?}"
    );
}

/// (B) Témoin M1 : flux result-only — le TEXTE FINAL EXACT dans le fil,
/// même message_id. Mutant : retirer le repli result→record(update)
/// (text_updates==0) → left=[] / right=["REPONSE FINALE"].
#[allow(non_snake_case)]
#[test]
fn TEMOIN_B_claude_stream_json_reponse_finale_result_only_dans_le_fil() {
    let events = journal_fixture_result_only();
    let updates: Vec<_> = events
        .iter()
        .filter(|event| event["event"] == "update")
        .collect();
    let contents: Vec<&str> = updates
        .iter()
        .filter_map(|event| event["payload"]["content"].as_str())
        .collect();
    assert_eq!(
        contents,
        vec!["REPONSE FINALE"],
        "repli result-only → update exact, reçu {contents:?} via {events:?}"
    );
    assert!(
        updates
            .iter()
            .all(|event| event["message_id"] == "claude-abc-1"),
        "même message_id que le tour, reçu {updates:?}"
    );
}

/// (C) Témoin M2 : turn_end.payload.stop_reason = issue attestée
/// (`completed` = terminal_reason fournisseur). Mutant : turn_end {} →
/// attach affiche « inconnu » (left=Null / right="completed").
#[allow(non_snake_case)]
#[test]
fn TEMOIN_C_claude_stream_json_turn_end_porte_stop_reason_completed() {
    let events = journal_fixture_result_only();
    let turn_end = events
        .iter()
        .find(|event| event["event"] == "turn_end")
        .expect("turn_end attendu");
    assert_eq!(
        turn_end["payload"]["stop_reason"].as_str(),
        Some("completed"),
        "stop_reason manquant → attach affiche « inconnu », payload={}",
        turn_end["payload"]
    );
    assert_eq!(
        turn_end["payload"]["routed_to"].as_str(),
        Some("bridget"),
        "reply=true doit porter routed_to, payload={}",
        turn_end["payload"]
    );
}

/// (TOOL) Appel d'outil Claude → update kind=tool avec NOM + CONTENU input.
/// Source : message assistant (input complet), pas content_block_start (input {}).
/// Mutant : retirer record_tool_uses_from_assistant, ou vider text/detail → meurt.
#[allow(non_snake_case)]
#[test]
fn TEMOIN_TOOL_claude_stream_json_retranscrit_tool_use_nom_et_contenu() {
    let events = journal_fixture_tool_use();
    let tools: Vec<_> = events
        .iter()
        .filter(|event| event["event"] == "update" && event["payload"]["kind"] == "tool")
        .collect();
    assert_eq!(
        tools.len(),
        1,
        "un seul acte tool attendu, reçu {tools:?} via {events:?}"
    );
    assert_eq!(
        tools[0]["payload"]["text"].as_str(),
        Some("Read"),
        "nom d'outil en dur, payload={}",
        tools[0]["payload"]
    );
    assert_eq!(
        tools[0]["payload"]["tool"].as_str(),
        Some("Read"),
        "champ tool en dur, payload={}",
        tools[0]["payload"]
    );
    assert_eq!(
        tools[0]["payload"]["detail"].as_str(),
        Some(r#"{"file_path":"/tmp/demo.toml","limit":1}"#),
        "input complet en dur (pas une présence vide), payload={}",
        tools[0]["payload"]
    );
    assert_eq!(
        tools[0]["message_id"].as_str(),
        Some("claude-abc-1"),
        "même message_id, reçu {tools:?}"
    );
}

/// Un fournisseur qui sort aussitôt après son résultat ferme stdout avant
/// que le fil de la mission ne relève sa réponse : elle ne doit pas se perdre.
#[test]
fn spec243_sortie_immediate_apres_resultat_garde_la_reponse() {
    for round in 0..40 {
        let mut configured = options();
        configured.command = "/bin/sh".into();
        configured.args = vec![
            "-c".into(),
            concat!(
                "IFS= read -r _; ",
                "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,",
                "\"terminal_reason\":\"completed\",\"result\":\"fin-243\"}'"
            )
            .into(),
        ];
        configured.notify_timeout_secs = 10;
        let mut transport = ClaudeStreamJsonTransport::spawn(configured).unwrap();
        transport.deliver(&message("sortie-243")).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        let response = loop {
            let found = transport
                .drain_events()
                .into_iter()
                .find_map(|event| match event.kind {
                    ManagedEventKind::TurnFinished { response, .. } => Some(Ok(response)),
                    ManagedEventKind::DeliveryRejected { reason, .. } => Some(Err(reason)),
                    _ => None,
                });
            if let Some(found) = found {
                break found;
            }
            assert!(
                Instant::now() < deadline,
                "fin de tour absente, tour {round}"
            );
            thread::sleep(Duration::from_millis(2));
        };
        assert_eq!(response, Ok("fin-243".to_string()), "tour {round}");
    }
}

fn journal_fixture_deltas() -> Vec<Value> {
    journal_fixture(concat!(
        "while IFS= read -r line; do ",
        "printf '%s\\n' '{\"type\":\"stream_event\",\"event\":{\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"BONJOUR \"}}}'; ",
        "printf '%s\\n' '{\"type\":\"stream_event\",\"event\":{\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"JE SUIS VIVANT\"}}}'; ",
        "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",\"stop_reason\":\"end_turn\",\"result\":\"BONJOUR JE SUIS VIVANT\"}'; ",
        "done"
    ))
}

fn journal_fixture_result_only() -> Vec<Value> {
    journal_fixture(concat!(
        "while IFS= read -r line; do ",
        "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",\"stop_reason\":\"end_turn\",\"result\":\"REPONSE FINALE\"}'; ",
        "done"
    ))
}

fn journal_fixture_tool_use() -> Vec<Value> {
    let _suite = crate::act_kind::pilot_kind_suite_lock();
    // Forme mesurée L19 de /tmp/flux-outil.jsonl : assistant + tool_use + input.
    journal_fixture(concat!(
        "while IFS= read -r line; do ",
        "printf '%s\\n' '{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"tool_use\",\"id\":\"toolu_demo\",\"name\":\"Read\",\"input\":{\"file_path\":\"/tmp/demo.toml\",\"limit\":1}}]}}'; ",
        "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",\"stop_reason\":\"end_turn\",\"result\":\"ok\"}'; ",
        "done"
    ))
}

fn journal_fixture(provider_script: &str) -> Vec<Value> {
    // Compteur atomique : sous parallelisme, SystemTime::nanos peut
    // coincider et un remove_dir_all efface le journal d'un autre témoin.
    let root = std::env::temp_dir().join(format!(
        "bridget-claude-abc-{}-{}",
        std::process::id(),
        FIXTURE_SEQ.fetch_add(1, AtomicOrdering::Relaxed)
    ));
    fs::create_dir_all(&root).unwrap();
    let mut opts = options();
    opts.args[1] = provider_script.to_string();
    let mut transport = ClaudeStreamJsonTransport::spawn(opts).unwrap();
    transport
        .activate_journal(&root, "claude-abc", None)
        .unwrap();
    transport.deliver(&message("claude-abc-1")).unwrap();
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
    assert!(finished, "le tour Claude n'a pas terminé");
    transport.stop();
    let journal_path = fs::read_dir(root.join("claude-abc"))
        .expect("répertoire journal")
        .next()
        .expect("fichier journal")
        .expect("entrée journal")
        .path();
    let events = crate::journal::valid_events(&journal_path);
    let _ = fs::remove_dir_all(&root);
    events
}

#[test]
fn formats_fournisseur_imposes_ne_peuvent_pas_etre_ecrases() {
    let mut args = vec!["--output-format".to_string(), "json".to_string()];
    assert!(ensure_stream_arguments(&mut args).is_err());
}

/// Contrat changé le 28/08, sur mesures : jusqu'ici, faute de trame
/// d'interruption attestée, l'annulation d'un tour actif tuait le groupe de
/// processus. Claude accuse un `control_request` en 3 ms et rend un
/// terminal `aborted_tools` en 25 ms, sous-processus arrêté sans signal
/// POSIX, tour suivant réussi en 1,594 s. On coupe donc le tour SANS tuer
/// l'agent — c'est déjà ce que fait Codex (`codex_app_server.rs`), qui
/// signale et laisse le worker émettre `turn/interrupt`.
///
/// Le faux fournisseur répond ici exactement les formes mesurées.
#[test]
fn message_humain_actif_interrompt_claude_et_declenche_la_remise() {
    let mut slow = options();
    slow.args[1] = concat!(
        "while IFS= read -r line; do case \"$line\" in ",
        "*control_request*) ",
        "rid=$(printf '%s' \"$line\" | sed 's/.*\"request_id\":\"\\([^\"]*\\)\".*/\\1/'); ",
        "printf '{\"type\":\"control_response\",\"response\":{\"subtype\":\"success\",",
        "\"request_id\":\"%s\",\"response\":{\"still_queued\":[]}}}\\n' \"$rid\"; ",
        "printf '{\"type\":\"result\",\"is_error\":true,",
        "\"terminal_reason\":\"aborted_tools\"}\\n';; ",
        "*) : ;; esac; done"
    )
    .to_string();
    let mut transport = ClaudeStreamJsonTransport::spawn(slow).unwrap();
    transport.deliver(&message("claude-actif")).unwrap();
    let start_deadline = Instant::now() + Duration::from_secs(2);
    while !transport.is_busy() && Instant::now() < start_deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        transport.is_busy(),
        "le tour Claude initial doit être actif"
    );

    let mut human = message("claude-humain");
    human.from = "superviseur".to_string();
    human.origin = Some(MessageOrigin::Human);
    transport.deliver(&human).unwrap();

    let deadline = Instant::now() + Duration::from_secs(3);
    let mut interrupted = false;
    let mut human_dispatched = false;
    while Instant::now() < deadline {
        for event in transport.drain_events() {
            interrupted |= matches!(
                event.kind,
                ManagedEventKind::TurnFinished {
                    ref message,
                    terminal: ManagedTerminal::Cancelled,
                    ..
                } if message.id == "claude-actif"
            );
            human_dispatched |= matches!(
                event.kind,
                ManagedEventKind::PromptDispatched { ref message_id }
                    if message_id == "claude-humain"
            );
        }
        if interrupted && human_dispatched {
            assert!(
                transport.is_alive(),
                "Claude doit rester utilisable après interruption"
            );
            transport.stop();
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    transport.stop();
    panic!(
        "le message humain devait interrompre le tour actif puis être remis, interruption={interrupted}, remis={human_dispatched}"
    );
}

#[test]
fn interruption_active_coupe_le_tour_sans_tuer_l_agent_claude() {
    let mut slow = options();
    slow.args[1] = concat!(
        "while IFS= read -r line; do case \"$line\" in ",
        "*control_request*) ",
        "rid=$(printf '%s' \"$line\" | sed 's/.*\"request_id\":\"\\([^\"]*\\)\".*/\\1/'); ",
        "printf '{\"type\":\"control_response\",\"response\":{\"subtype\":\"success\",",
        "\"request_id\":\"%s\",\"response\":{\"still_queued\":[]}}}\\n' \"$rid\"; ",
        "printf '{\"type\":\"result\",\"is_error\":true,",
        "\"terminal_reason\":\"aborted_tools\"}\\n';; ",
        "*) : ;; esac; done"
    )
    .to_string();
    let mut transport = ClaudeStreamJsonTransport::spawn(slow).unwrap();
    transport.deliver(&message("claude-cancel")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while !transport.is_busy() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(transport.is_busy());
    assert!(transport.cancel_delivery("claude-cancel", "annulé par le daemon"));

    let deadline = Instant::now() + Duration::from_secs(5);
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
    // Assertion métier 1 : `aborted_tools` est un terminal d'annulation,
    // pas un échec — sinon l'agent serait classé bloqué par la ronde.
    assert!(
        events.iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::TurnFinished {
                    terminal: ManagedTerminal::Cancelled,
                    ..
                }
            )
        }),
        "terminal d'interruption absent : {events:?}"
    );
    // Assertion métier 2 : c'est tout l'objet de la trame — l'agent
    // survit à l'interruption et peut enchaîner le tour suivant.
    assert!(
        transport.is_alive(),
        "l'agent a été tué au lieu d'être interrompu"
    );
    transport.stop();
}

#[test]
fn user_input181_claude_callback_roundtrip_and_permissions() {
    let request = json!({"type":"control_request","request_id":"ask181","request":{"subtype":"can_use_tool","tool_name":"AskUserQuestion","input":{"questions":[{"question":"Quel choix ?","header":"Choix","multiSelect":true,"options":[{"label":"A","description":"Premier"},{"label":"B","description":"Second"}]}]}}});
    let mut configured = options();
    configured.args[1] = format!(
        "printf '%s\\n' '{{\"type\":\"system\",\"subtype\":\"init\",\"session_id\":\"session181\"}}'; while IFS= read -r line; do printf '%s\\n' '{request}'; IFS= read -r answer; printf '%s\\n' '{{\"type\":\"result\",\"result\":\"answered\",\"terminal_reason\":\"completed\",\"is_error\":false}}'; done"
    );
    let mut transport = ClaudeStreamJsonTransport::spawn(configured).unwrap();
    transport.deliver(&message("ask181-turn")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut callback = None;
    while Instant::now() < deadline && callback.is_none() {
        for event in transport.drain_events() {
            if let ManagedEventKind::ProviderRequest { request } = event.kind {
                callback = Some(request);
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
    let callback = callback.expect("question Claude publiée");
    assert_eq!(
        callback.details["questions"][0]["options"][0]["description"],
        "Premier"
    );
    let mut answer = ChildRequestAnswer {
        request_id: "daemon181".into(),
        provider_session: callback.provider_session.clone(),
        turn_id: callback.turn_id.clone(),
        provider_request_id: callback.provider_request_id.clone(),
        decision: ChildRequestDecision::Accept,
        answer: Some(json!({"Quel choix ?":{"selected":["A","B"],"text":"avec précision"}})),
        outcome: "human_user_input".into(),
    };
    let frame = claude_user_input_response(&callback, &answer).unwrap();
    assert_eq!(
        frame["response"]["response"]["updatedInput"]["answers"]["Quel choix ?"],
        "A, B, avec précision"
    );
    answer.provider_session = "foreign".into();
    assert!(transport.answer_provider_request(&answer).is_err());
    answer.provider_session = callback.provider_session.clone();
    transport.answer_provider_request(&answer).unwrap();
    assert!(
        transport.answer_provider_request(&answer).is_err(),
        "aucune seconde écriture SDK"
    );
    let mut events = Vec::new();
    while Instant::now() < deadline {
        events.extend(transport.drain_events());
        if events
            .iter()
            .any(|e| matches!(e.kind, ManagedEventKind::TurnFinished { .. }))
        {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    transport.stop();
    assert!(
        events.iter().any(|e| matches!(
            e.kind,
            ManagedEventKind::TurnFinished {
                terminal: ManagedTerminal::Completed,
                ..
            }
        )),
        "{events:?}"
    );
    let mut unrelated = request.clone();
    unrelated["request"]["tool_name"] = json!("Bash");
    assert!(claude_user_input(&unrelated, "s", "t").is_none());
}

#[test]
fn delivery171_tool_refusal_preserves_report_and_control_refusal_stays_failed() {
    for (interrupted, provider_error, legacy) in [
        (false, false, false),
        (true, false, false),
        (false, true, false),
        (false, false, true),
        (true, false, true),
        (false, true, true),
    ] {
        let mut configured = options();
        let control = if interrupted {
            "printf '%s\\n' '{\"type\":\"control_request\",\"request_id\":\"denied171\",\"request\":{\"subtype\":\"can_use_tool\"}}'; IFS= read -r answer; "
        } else {
            ""
        };
        let mut result = json!({"type":"result","is_error":provider_error,
                "terminal_reason":if provider_error {"api_error"} else {"completed"},
                "permission_denials":[{"tool_name":"Bash"}],
                "result":"HEAD correct. La seconde commande a été bloquée par le hook."});
        if legacy {
            result.as_object_mut().unwrap().remove("terminal_reason");
            result["subtype"] = json!(if provider_error {
                "error_during_execution"
            } else {
                "success"
            });
        }
        configured.args[1] =
            format!("while IFS= read -r line; do {control}printf '%s\\n' '{result}'; done");
        let mut transport = ClaudeStreamJsonTransport::spawn(configured).unwrap();
        transport.deliver(&message("denial-report171")).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events.iter().any(|event| {
                matches!(
                    &event.kind,
                    ManagedEventKind::TurnFinished { .. }
                        | ManagedEventKind::DeliveryRejected { .. }
                )
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        transport.stop();
        if !interrupted && !provider_error {
            assert!(events.iter().any(|event| matches!(&event.kind,
                    ManagedEventKind::TurnFinished {response,terminal:ManagedTerminal::Completed,..}
                    if response.contains("HEAD correct") && response.contains("bloquée par le hook") && response.contains("1 appel(s) d'outil refusé(s)"))), "{events:?}");
            assert!(
                !events
                    .iter()
                    .any(|event| matches!(&event.kind, ManagedEventKind::DeliveryRejected { .. }))
            );
        } else {
            let expected = if interrupted {
                "provider_permission_denied"
            } else if legacy {
                "error_during_execution"
            } else {
                "api_error"
            };
            assert!(
                events.iter().any(|event| matches!(&event.kind,
                    ManagedEventKind::DeliveryRejected {reason,..} if reason==expected)),
                "{events:?}"
            );
            assert!(!events.iter().any(|event| matches!(
                &event.kind,
                ManagedEventKind::TurnFinished {
                    terminal: ManagedTerminal::Completed,
                    ..
                }
            )));
        }
    }
}

#[test]
fn spec221_claude_result_optional_reason_and_closed_diagnostics() {
    let base = json!({"type":"result","subtype":"success","is_error":false,
            "num_turns":0,"result":"PRIVATE_RESULT","errors":["PRIVATE_ERROR"]});
    for reason in [None, Some(Value::Null), Some(json!("completed"))] {
        let mut frame = base.clone();
        if let Some(reason) = reason {
            frame["terminal_reason"] = reason;
        }
        assert!(matches!(
            classify_claude_result(&frame, false, false, None),
            ManagedTerminal::Completed
        ));
        assert!(matches!(classify_claude_result(&frame,true,false,None),
                ManagedTerminal::Failed { detail } if detail=="provider_permission_denied"));
    }
    let mut explicit = base.clone();
    explicit["terminal_reason"] = json!("completed");
    explicit["subtype"] = Value::Null;
    assert!(matches!(
        classify_claude_result(&explicit, false, false, None),
        ManagedTerminal::Completed
    ));
    explicit["is_error"] = json!(true);
    assert!(matches!(classify_claude_result(&explicit,false,false,None),
            ManagedTerminal::Failed { detail } if detail=="terminal Claude inconnu"));
    for (subtype, expected) in [
        ("error_max_turns", "max_turns"),
        ("error_max_budget_usd", "budget_exhausted"),
        (
            "error_max_structured_output_retries",
            "structured_output_retry_exhausted",
        ),
        ("error_during_execution", "error_during_execution"),
    ] {
        let mut frame = base.clone();
        frame["is_error"] = json!(true);
        frame["subtype"] = json!(subtype);
        assert!(matches!(classify_claude_result(&frame,false,false,None),
                ManagedTerminal::Failed { detail } if detail==expected));
    }
    for patch in [
        json!({"terminal_reason":"PRIVATE_REASON"}),
        json!({"terminal_reason":false}),
        json!({"terminal_reason":""}),
        json!({"is_error":true}),
        json!({"is_error":null}),
        json!({"subtype":"PRIVATE_SUBTYPE"}),
        json!({"subtype":"error_during_execution","terminal_reason":"completed"}),
    ] {
        let mut frame = base.clone();
        frame
            .as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());
        let terminal = classify_claude_result(&frame, false, false, None);
        assert!(
            matches!(terminal, ManagedTerminal::Failed { .. }),
            "{frame:?}"
        );
        let metadata =
            serde_json::to_string(&claude_result_metadata(&frame, &terminal, false)).unwrap();
        assert!(
            !metadata.contains("PRIVATE") && metadata.len() < 512,
            "{metadata}"
        );
    }
    let mut frame = base.clone();
    frame["terminal_reason"] = json!("aborted_streaming");
    assert!(matches!(
        classify_claude_result(&frame, false, true, None),
        ManagedTerminal::Cancelled
    ));
    frame["terminal_reason"] = json!("api_error");
    assert!(matches!(classify_claude_result(&frame,false,true,None),
            ManagedTerminal::Failed { detail } if detail=="usage_limit"));
    assert!(matches!(
        classify_claude_result(&json!({"stop_reason":"end_turn"}), false, false, None),
        ManagedTerminal::Failed { .. }
    ));
}

#[test]
fn spec258_oauth_expire_exige_terminal_erreur_et_diagnostic_exact_du_tour() {
    let diagnostic = "Failed to authenticate: OAuth session expired and could not be refreshed";
    let error = json!({"type":"result","terminal_reason":"api_error","is_error":true});
    assert!(
        matches!(classify_claude_result(&error,false,false,Some(diagnostic)),ManagedTerminal::Failed{detail} if detail=="api_error:oauth_session_expired")
    );
    let success = json!({"type":"result","terminal_reason":"completed","subtype":"success","is_error":false,"result":diagnostic});
    assert!(matches!(
        classify_claude_result(&success, false, false, Some(diagnostic)),
        ManagedTerminal::Completed
    ));
    for text in [
        "OAuth session expired",
        "Compte refusé",
        "un assistant cite OAuth session expired and could not be refreshed",
    ] {
        assert!(
            matches!(classify_claude_result(&error,false,false,Some(text)),ManagedTerminal::Failed{detail} if detail=="api_error")
        );
    }
    assert!(
        matches!(classify_claude_result(&error,false,false,None),ManagedTerminal::Failed{detail} if detail=="api_error")
    );
}

#[test]
fn spec258_diagnostic_oauth_assistant_correlé_est_conserve_dans_refus_et_journal() {
    for typed_error in [false, true] {
        let mut failing = options();
        failing.args[1]=concat!("while IFS= read -r line; do ",
            "printf '%s\\n' '{\"type\":\"assistant\",\"isApiErrorMessage\":true,\"message\":{\"content\":[{\"type\":\"text\",\"text\":\"Failed to authenticate: OAuth session expired and could not be refreshed\"}]}}' ",
            "'{\"type\":\"result\",\"is_error\":true,\"terminal_reason\":\"api_error\",\"result\":\"\"}'; done").to_string();
        if typed_error {
            failing.args[1] = failing.args[1].replace(
                "\"isApiErrorMessage\":true",
                "\"error\":\"authentication_failed\"",
            );
        }
        let root = std::env::temp_dir().join(format!(
            "bridget-oauth258-{}-{}",
            std::process::id(),
            FIXTURE_SEQ.fetch_add(1, AtomicOrdering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        failing.session_store_root = Some(root.clone());
        failing.agent_name = Some("oauth258".into());
        let mut transport = ClaudeStreamJsonTransport::spawn(failing).unwrap();
        transport.activate_journal(&root, "oauth258", None).unwrap();
        transport.deliver(&message("oauth258")).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events.iter().any(|e|matches!(&e.kind,ManagedEventKind::DeliveryRejected{reason,..} if reason=="api_error:oauth_session_expired")){break;}
            thread::sleep(Duration::from_millis(10));
        }
        assert!(events.iter().any(|e|matches!(&e.kind,ManagedEventKind::DeliveryRejected{message_id,reason} if message_id=="oauth258" && reason=="api_error:oauth_session_expired")),"{events:?}");
        transport.stop();
        let journal_path = fs::read_dir(root.join("oauth258"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let entries = crate::journal::valid_events(&journal_path);
        assert!(entries.iter().any(|e| e["event"] == "provider_result"
            && e["payload"]["terminal_reason"] == "api_error"
            && e["payload"]["diagnostic_code"] == "oauth_session_expired"));
    }
}

#[test]
fn spec258_citation_oauth_sans_marqueur_api_ne_devient_pas_cause_du_refus() {
    let mut fake = options();
    fake.args[1]=concat!("while IFS= read -r line; do ",
            "printf '%s\\n' '{\"type\":\"assistant\",\"message\":{\"content\":[{\"type\":\"text\",\"text\":\"Failed to authenticate: OAuth session expired and could not be refreshed\"}]}}' ",
            "'{\"type\":\"result\",\"is_error\":true,\"terminal_reason\":\"api_error\",\"result\":\"different provider error\"}'; done").into();
    let mut transport = ClaudeStreamJsonTransport::spawn(fake).unwrap();
    transport.deliver(&message("quote258")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut events = Vec::new();
    while Instant::now() < deadline {
        events.extend(transport.drain_events());
        if events
            .iter()
            .any(|e| matches!(&e.kind, ManagedEventKind::DeliveryRejected { .. }))
        {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(events.iter().any(|e|matches!(&e.kind,ManagedEventKind::DeliveryRejected{reason,..} if reason=="api_error")),"{events:?}");
    transport.stop();
}

#[test]
fn terminal_fournisseur_en_erreur_refuse_la_livraison() {
    let mut failing = options();
    failing.args[1] = concat!(
        "while IFS= read -r line; do ",
        "printf '%s\\n' '{\"type\":\"result\",\"is_error\":true,",
        "\"terminal_reason\":\"api_error\",\"result\":\"authentification refusée\"}'; ",
        "done"
    )
    .to_string();
    let mut transport = ClaudeStreamJsonTransport::spawn(failing).unwrap();
    transport.deliver(&message("claude-reject")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(1);
    let mut events = Vec::new();
    while Instant::now() < deadline {
        events.extend(transport.drain_events());
        if events.iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
                    if message_id == "claude-reject" && reason == "api_error"
            )
        }) {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(events.iter().any(|event| {
        matches!(
            event.kind,
            ManagedEventKind::DeliveryRejected { ref message_id, ref reason }
                if message_id == "claude-reject" && reason == "api_error"
        )
    }));
    transport.stop();
}

#[test]
fn spec172_assistant_rate_limit_is_structured_and_recovery_replaces_it() {
    for (error, recovery, terminal_reason, expected, dated) in [
        ("rate_limit", false, "api_error", "usage_limit", false),
        (
            "authentication_failed",
            false,
            "api_error",
            "api_error",
            false,
        ),
        ("", false, "api_error", "api_error", false),
        ("rate_limit", true, "api_error", "api_error", false),
        ("rate_limit", true, "completed", "completed", false),
        ("absent", false, "completed", "completed", false),
        ("rate_limit", false, "api_error", "usage_limit", true),
        ("rate_then_auth", false, "api_error", "api_error", false),
        ("rate_then_empty", false, "api_error", "api_error", false),
    ] {
        let mut frames = vec![json!({"type":"assistant","error":error,
                "message":{"content":[{"type":"text","text":"You've hit your session limit · resets 1:20pm (UTC)"}]}})];
        if error.starts_with("rate_then_") {
            frames[0]["error"] = json!("rate_limit");
            if error == "rate_then_auth" {
                frames.push(json!({"type":"assistant", "error":"authentication_failed",
                        "message":{"content":[{"type":"text","text":"Connection refused"}]}}));
            } else {
                frames.push(json!({"type":"assistant", "message":{"content":[]}}));
            }
        }
        if error == "absent" {
            frames[0].as_object_mut().unwrap().remove("error");
            frames[0]["message"]["content"][0]["text"] = json!("Provider response recovered");
        }
        if dated {
            frames.insert(
                0,
                json!({"type":"rate_limit_event", "rate_limit_info":{
                    "status":"rejected", "rateLimitType":"five_hour", "resetsAt":200}}),
            );
        }
        if recovery {
            frames.push(json!({"type":"assistant","message":{"content":[{"type":"text","text":"Provider response recovered"}]}}));
        }
        frames.push(
            json!({"type":"result","is_error":terminal_reason!="completed",
                "terminal_reason":terminal_reason,"result":"Provider result"}),
        );
        let output = frames
            .into_iter()
            .map(|frame| {
                let quoted = frame.to_string().replace('\'', "'\\''");
                format!("printf '%s\\n' '{quoted}'; ")
            })
            .collect::<String>();
        let mut configured = options();
        configured.args[1] = format!("while IFS= read -r line; do {output}done");
        let mut transport = ClaudeStreamJsonTransport::spawn(configured).unwrap();
        transport.deliver(&message("quota172")).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            events.extend(transport.drain_events());
            if events.iter().any(|event| {
                matches!(
                    &event.kind,
                    ManagedEventKind::TurnFinished { .. }
                        | ManagedEventKind::DeliveryRejected { .. }
                )
            }) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        transport.stop();
        if expected == "completed" {
            assert!(
                events.iter().any(|event| matches!(
                    &event.kind,
                    ManagedEventKind::TurnFinished {
                        terminal: ManagedTerminal::Completed,
                        ..
                    }
                )),
                "{events:?}"
            );
            assert!(
                events.iter().any(|event| matches!(&event.kind,
                    ManagedEventKind::TurnFinished {response,terminal:ManagedTerminal::Completed,..}
                    if response == "Provider response recovered")),
                "successful response must replace the quota notice"
            );
        } else {
            assert!(
                events.iter().any(|event| matches!(&event.kind,
                    ManagedEventKind::DeliveryRejected {reason,..} if reason==expected)),
                "{events:?}"
            );
        }
        let facts = events
            .iter()
            .filter_map(|event| match &event.kind {
                ManagedEventKind::RateLimitObserved {
                    window,
                    status,
                    resets_at,
                    used_percent,
                    credits,
                } => {
                    assert_eq!(window, if dated { "five_hour" } else { "unknown" });
                    assert_eq!(*resets_at, dated.then_some(200));
                    assert!(used_percent.is_none() && credits.is_none());
                    Some(status.as_str())
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            facts,
            if error == "rate_limit" || error.starts_with("rate_then_") {
                if recovery {
                    vec!["rejected", "allowed"]
                } else {
                    vec!["rejected"]
                }
            } else if error == "absent" {
                vec!["allowed"]
            } else {
                vec![]
            }
        );
    }
}

#[test]
fn spec172_rate_limit_window_is_optional_but_reset_is_structured() {
    let event = rate_limit_event(&json!({"type":"rate_limit_event",
            "rate_limit_info":{"status":"rejected","resetsAt":200}}))
    .unwrap();
    assert!(matches!(event, ManagedEventKind::RateLimitObserved {
            window, status, resets_at: Some(200), credits: None, ..
        } if window == "unknown" && status == "rejected"));
    assert!(rate_limit_event(&json!({"rate_limit_info":{"rateLimitType":"five_hour"}})).is_none());
}

#[test]
fn rate_limit_event_reel_devient_un_fait_sans_inventer_de_retour() {
    let event: Value = serde_json::from_str(
            r#"{"type":"rate_limit_event","rate_limit_info":{"status":"rejected","resetsAt":1787572200,"rateLimitType":"five_hour"}}"#,
        )
        .unwrap();
    assert!(matches!(
        rate_limit_event(&event),
        Some(ManagedEventKind::RateLimitObserved {
            ref window,
            ref status,
            resets_at: Some(1_787_572_200),
            used_percent: None,
            credits: None,
        }) if window == "five_hour" && status == "rejected"
    ));

    let no_reset: Value = serde_json::from_str(
            r#"{"type":"rate_limit_event","rate_limit_info":{"status":"allowed","rateLimitType":"five_hour"}}"#,
        )
        .unwrap();
    assert!(matches!(
        rate_limit_event(&no_reset),
        Some(ManagedEventKind::RateLimitObserved {
            resets_at: None,
            used_percent: None,
            ..
        })
    ));
}

#[test]
fn init_annonce_un_modele_distinct_et_le_journalise() {
    let event: Value =
        serde_json::from_str(r#"{"type":"system","subtype":"init","model":"claude-opus-4-6"}"#)
            .unwrap();
    assert_eq!(
        served_model_from_claude(&event).as_deref(),
        Some("claude-opus-4-6")
    );

    let mut mismatch = options();
    mismatch.args = vec![
            "-c".to_string(),
            concat!(
                "while IFS= read -r line; do ",
                "printf '%s\\n' '{\"type\":\"system\",\"subtype\":\"init\",\"model\":\"claude-opus-4-6\"}'; ",
                "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",\"result\":\"ok\"}'; ",
                "done"
            )
            .to_string(),
            "--model".to_string(),
            "claude-opus-5".to_string(),
        ];
    let root = std::env::temp_dir().join(format!(
        "bridget-claude-mismatch-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    let mut transport = ClaudeStreamJsonTransport::spawn(mismatch).unwrap();
    transport
        .activate_journal(&root, "claude-mismatch", None)
        .unwrap();
    transport.deliver(&message("claude-gap")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut events = Vec::new();
    while Instant::now() < deadline {
        events.extend(transport.drain_events());
        if events.iter().any(|event| {
            matches!(
                event.kind,
                ManagedEventKind::ModelObserved { ref model } if model == "claude-opus-4-6"
            )
        }) {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(events.iter().any(|event| {
        matches!(
            event.kind,
            ManagedEventKind::ModelObserved { ref model } if model == "claude-opus-4-6"
        )
    }));
    transport.stop();
    let contents = fs::read_dir(root.join("claude-mismatch"))
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "jsonl"))
        .map(|entry| fs::read_to_string(entry.path()).unwrap())
        .collect::<String>();
    assert!(
        contents.contains("\"event\":\"model_mismatch\""),
        "journal sans écart: {contents}"
    );
    assert!(contents.contains("claude-opus-5"));
    assert!(contents.contains("claude-opus-4-6"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn flux_muet_ou_modele_identique_ne_produit_aucun_ecart() {
    assert!(
        served_model_from_claude(
            &serde_json::from_str(r#"{"type":"system","subtype":"status","status":"requesting"}"#)
                .unwrap()
        )
        .is_none()
    );
    assert!(
        served_model_from_claude(
            &serde_json::from_str(r#"{"type":"system","subtype":"init"}"#).unwrap()
        )
        .is_none()
    );

    let mut matching = options();
    matching.args = vec![
            "-c".to_string(),
            concat!(
                "while IFS= read -r line; do ",
                "printf '%s\\n' '{\"type\":\"system\",\"subtype\":\"init\",\"model\":\"claude-opus-5\"}'; ",
                "printf '%s\\n' '{\"type\":\"result\",\"is_error\":false,\"terminal_reason\":\"completed\",\"result\":\"ok\"}'; ",
                "done"
            )
            .to_string(),
            "--model".to_string(),
            "claude-opus-5".to_string(),
        ];
    let root = std::env::temp_dir().join(format!(
        "bridget-claude-match-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    let mut transport = ClaudeStreamJsonTransport::spawn(matching).unwrap();
    transport
        .activate_journal(&root, "claude-match", None)
        .unwrap();
    transport.deliver(&message("claude-ok")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut events = Vec::new();
    while Instant::now() < deadline {
        events.extend(transport.drain_events());
        if events
            .iter()
            .any(|event| matches!(event.kind, ManagedEventKind::ModelObserved { .. }))
        {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(events.iter().any(|event| {
        matches!(
            event.kind,
            ManagedEventKind::ModelObserved { ref model } if model == "claude-opus-5"
        )
    }));
    transport.stop();
    let contents = fs::read_dir(root.join("claude-match"))
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "jsonl"))
        .map(|entry| fs::read_to_string(entry.path()).unwrap())
        .collect::<String>();
    assert!(
        !contents.contains("model_mismatch"),
        "écart inventé: {contents}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn usage_result_reel_devient_un_fait_complet() {
    let event: Value = serde_json::from_str(
            r#"{"type":"result","usage":{"input_tokens":2,"output_tokens":175,"cache_creation_input_tokens":40804,"cache_read_input_tokens":13907}}"#,
        )
        .unwrap();
    assert!(matches!(
        usage_event(&event),
        Some(ManagedEventKind::UsageObserved {
            input_tokens: 2,
            output_tokens: 175,
            cache_creation_input_tokens: 40_804,
            cache_read_input_tokens: 13_907,
        })
    ));
}

#[test]
fn usage_incomplet_ne_devient_pas_un_zero_invente() {
    let event: Value =
        serde_json::from_str(r#"{"type":"result","usage":{"input_tokens":2,"output_tokens":10}}"#)
            .unwrap();
    assert!(usage_event(&event).is_none());
}

#[test]
fn usage_assistant_puis_result_ne_compte_qu_un_echantillon() {
    let assistant: Value = serde_json::from_str(
            r#"{"type":"assistant","message":{"usage":{"input_tokens":2,"output_tokens":175,"cache_creation_input_tokens":40804,"cache_read_input_tokens":13907}}}"#,
        )
        .unwrap();
    let result: Value = serde_json::from_str(
            r#"{"type":"result","usage":{"input_tokens":2,"output_tokens":175,"cache_creation_input_tokens":40804,"cache_read_input_tokens":13907}}"#,
        )
        .unwrap();
    let counted = [assistant, result]
        .iter()
        .filter(|event| usage_event(event).is_some())
        .count();
    assert_eq!(
        counted, 1,
        "un tour assistant+result ne doit poser qu'un échantillon"
    );
    assert!(usage_event(&serde_json::from_str(
            r#"{"type":"assistant","message":{"usage":{"input_tokens":2,"output_tokens":175,"cache_creation_input_tokens":40804,"cache_read_input_tokens":13907}}}"#,
        )
        .unwrap())
        .is_none());
}

/// P2 bout-en-bout : `--resume` fantôme → message nommé + session neuve.
/// Mutant : avaler l'échec sans Error event → oracle meurt seul.
#[allow(non_snake_case)]
#[test]
fn TEMOIN_P2_spawn_repli_nomme_apres_echec_resume() {
    use crate::claude_provider_session::CLAUDE_RESUME_FAILED_PREFIX;
    let root = std::env::temp_dir().join(format!(
        "bridget-claude-resume-fail-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    let fake = root.join("fake-claude");
    fs::write(
            &fake,
            r#"#!/bin/sh
if [ "$1" = "--resume" ]; then
  printf '%s\n' '{"type":"result","subtype":"error_during_execution","is_error":true,"errors":["No conversation found with session ID: 00000000-0000-0000-0000-000000000000"]}'
  exit 1
fi
printf '%s\n' '{"type":"system","subtype":"init","session_id":"fresh-session-999","model":"claude-opus-5"}'
while IFS= read -r line; do
  printf '%s\n' '{"type":"result","is_error":false,"terminal_reason":"completed","result":"ok"}'
done
"#,
        )
        .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&fake).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake, permissions).unwrap();
    }
    let store = crate::claude_provider_session::ProviderSessionStore::new(&root, "agent-x");
    store.store("00000000-0000-0000-0000-000000000000").unwrap();
    let options = ClaudeStreamJsonOptions {
        command: fake.to_string_lossy().into_owned(),
        args: Vec::new(),
        provider_kind: "claude".to_string(),
        queue_capacity: 2,
        notify_timeout_secs: 2,
        provider_observation: None,
        session_store_root: Some(root.clone()),
        agent_name: Some("agent-x".to_string()),
    };
    let transport = ClaudeStreamJsonTransport::spawn(options).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut events = Vec::new();
    while Instant::now() < deadline {
        events.extend(transport.drain_events());
        if events.iter().any(|event| {
            matches!(
                &event.kind,
                ManagedEventKind::Error { detail }
                    if detail.starts_with(CLAUDE_RESUME_FAILED_PREFIX)
            )
        }) {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    let notice = events.iter().find_map(|event| match &event.kind {
        ManagedEventKind::Error { detail } => Some(detail.clone()),
        _ => None,
    });
    let notice = notice.expect("le message d'échec de reprise doit exister");
    assert!(
        notice.starts_with(CLAUDE_RESUME_FAILED_PREFIX),
        "échec non nommé: {notice}"
    );
    assert!(
        notice.contains("conversation introuvable"),
        "cause absente: {notice}"
    );
    assert!(
        notice.contains("00000000-0000-0000-0000-000000000000"),
        "identifiant absent: {notice}"
    );
    let fresh = crate::claude_provider_session::ProviderSessionStore::new(&root, "agent-x");
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if fresh.load().as_deref() == Some("fresh-session-999") {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(fresh.load().as_deref(), Some("fresh-session-999"));
    assert_ne!(
        fresh.load().as_deref(),
        Some("00000000-0000-0000-0000-000000000000")
    );
    transport.stop();
    let _ = fs::remove_dir_all(root);
}

/// P4 — session persistée : le spawn rend la main avant le plafond daemon.
/// Mutant : remettre read_line dans spawn_claude_child → oracle meurt seul.
#[allow(non_snake_case)]
#[test]
fn TEMOIN_P4_session_persistee_ne_bloque_pas_le_lancement_sous_delai_daemon() {
    let root = std::env::temp_dir().join(format!(
        "bridget-claude-resume-timeout-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    let fake = root.join("fake-claude");
    fs::write(
            &fake,
            r#"#!/bin/sh
resume=0
for arg in "$@"; do [ "$arg" = "--resume" ] && resume=1; done
if [ "$resume" = 1 ]; then sleep 15; fi
printf '%s\n' '{"type":"system","subtype":"init","session_id":"slow-or-fast","model":"claude-opus-5"}'
exit 0
"#,
        )
        .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&fake).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&fake, permissions).unwrap();
    }
    let options_neuf = ClaudeStreamJsonOptions {
        command: fake.to_string_lossy().into_owned(),
        args: Vec::new(),
        provider_kind: "claude".to_string(),
        queue_capacity: 2,
        notify_timeout_secs: 2,
        provider_observation: None,
        session_store_root: Some(root.clone()),
        agent_name: Some("agent-neuf".to_string()),
    };
    let t0 = Instant::now();
    let t_neuf = ClaudeStreamJsonTransport::spawn(options_neuf).expect("sans session");
    assert!(t0.elapsed() < DAEMON_SPAWN_TIMEOUT);
    t_neuf.stop();

    let store = crate::claude_provider_session::ProviderSessionStore::new(&root, "agent-lent");
    store.store("aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee").unwrap();
    let options_lent = ClaudeStreamJsonOptions {
        command: fake.to_string_lossy().into_owned(),
        args: Vec::new(),
        provider_kind: "claude".to_string(),
        queue_capacity: 2,
        notify_timeout_secs: 2,
        provider_observation: None,
        session_store_root: Some(root.clone()),
        agent_name: Some("agent-lent".to_string()),
    };
    let t1 = Instant::now();
    let t_lent = ClaudeStreamJsonTransport::spawn(options_lent).expect("avec session");
    assert!(
        t1.elapsed() < DAEMON_SPAWN_TIMEOUT,
        "spawn bloqué: {:?}",
        t1.elapsed()
    );
    assert!(
        t1.elapsed() < Duration::from_secs(2),
        "spawn trop lent: {:?}",
        t1.elapsed()
    );
    t_lent.stop();
    let _ = fs::remove_dir_all(root);
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
fn edit159_promote_transport_claude_enfile() {
    let mut transport = ClaudeStreamJsonTransport::spawn(options()).unwrap();
    let mut promoted = message("claude-promoted159");
    promoted.intent = Some(MessageIntent::SteerCurrent);
    promoted.references.push("steer-or-queue".into());
    transport.deliver(&promoted).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut finished = false;
    while Instant::now() < deadline && !finished {
        for event in transport.drain_events() {
            assert!(
                !matches!(&event.kind, ManagedEventKind::DeliveryRejected { message_id, .. } if message_id == "claude-promoted159")
            );
            finished |= matches!(&event.kind, ManagedEventKind::TurnFinished { message, .. } if message.id == "claude-promoted159");
        }
        if !finished {
            thread::sleep(Duration::from_millis(10));
        }
    }
    transport.stop();
    assert!(
        finished,
        "Claude traite la promotion comme une remise FIFO ordinaire"
    );
}

fn isolated_trace(choice: &Value) -> String {
    [
            json!({"type":"system","subtype":"init","tools":["StructuredOutput"],
                "mcp_servers":[],"skills":[],"plugins":[],"slash_commands":[],
                "capabilities":["interrupt_receipt_v1","interrupt_cancel_queued_v1","msg_lifecycle_v1"]}),
            json!({"type":"assistant","message":{"content":[{
                "type":"tool_use","name":"StructuredOutput","input":choice
            }]}}),
            json!({"type":"result","subtype":"success","is_error":false,
                "structured_output":choice,"result":choice.to_string()}),
        ]
        .into_iter()
        .map(|frame| frame.to_string())
        .collect::<Vec<_>>()
        .join("\n")
            + "\n"
}

// Même patron de faux fournisseur Python que le banc ManagedSession.
// Le fichier exécutable reçoit les véritables flags, sans lanceur modèle.
fn isolated_fixture(source: &str, trace: &str) -> (PathBuf, ClaudeIsolatedDecisionOptions) {
    use std::os::unix::fs::PermissionsExt;
    let root = std::env::temp_dir().join(format!(
        "bridget-claude-decision168-{}-{}",
        std::process::id(),
        FIXTURE_SEQ.fetch_add(1, AtomicOrdering::Relaxed)
    ));
    fs::create_dir(&root).unwrap();
    let executable = root.join("claude-fixture");
    fs::write(root.join("trace.jsonl"), trace).unwrap();
    fs::write(
            &executable,
            format!(
                "#!/usr/bin/python3\nimport json,os,pathlib,signal,stat,sys,time\nroot=pathlib.Path(__file__).parent\n{source}\n"
            ),
        )
        .unwrap();
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
    // /usr/bin/python3 peut être le lanceur Xcode, qui ajoute CPATH/SDKROOT
    // et consomme le budget avant l'entrée du faux programme. Tester le
    // transport avec l'interpréteur réellement résolu, comme sur Linux.
    let resolved = std::process::Command::new("/usr/bin/python3")
        .args(["-c", "import sys; print(sys.executable)"])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .output()
        .unwrap();
    assert!(resolved.status.success());
    let interpreter = PathBuf::from(String::from_utf8(resolved.stdout).unwrap().trim())
        .canonicalize()
        .unwrap();
    let options = ClaudeIsolatedDecisionOptions {
        command: interpreter.to_string_lossy().into_owned(),
        command_args: vec![executable.to_string_lossy().into_owned()],
        model: "claude-fixture-168".into(),
        environment: vec![
            ("HOME".into(), root.to_string_lossy().into_owned()),
            ("PATH".into(), "/usr/bin:/bin".into()),
            (
                "CLAUDE_CONFIG_DIR".into(),
                root.join("auth").to_string_lossy().into_owned(),
            ),
        ],
        request_json: json!({"type":"user","message":{"role":"user","content":"choisir"}})
            .to_string(),
        deadline: Instant::now() + Duration::from_secs(2),
    };
    (root, options)
}

#[test]
fn spec168_faux_executable_verifie_flags_env_cwd_et_stderr_neutralise() {
    let trace = isolated_trace(&json!({"candidate_id":"c001","reason":"ok"}));
    let (root, mut options) = isolated_fixture(
        r#"(root/'started').write_text('started')
args=sys.argv[1:]
required=['--print','--model','claude-fixture-168','--safe-mode','--restricted','--tools','','--strict-mcp-config','--mcp-config','{"mcpServers":{}}','--setting-sources','','--disable-slash-commands','--no-session-persistence','--input-format','stream-json','--output-format','stream-json','--verbose','--json-schema']
assert args[:-1]==required, args
schema=json.loads(args[-1])
assert schema['additionalProperties'] is False
assert schema['required']==['candidate_id']
assert set(schema['properties'])=={'candidate_id','effort','reason'}
assert json.loads(sys.stdin.read())=={'type':'user','message':{'role':'user','content':'choisir'}}
cwd=os.getcwd()
before=os.listdir(cwd)
stderr=os.fstat(2)
assert stat.S_ISCHR(stderr.st_mode) and stderr.st_rdev==os.stat(os.devnull).st_rdev
os.write(2,b'PRIVATE_STDERR_MUST_NOT_BE_CAPTURED'*10000)
(root/'seen.json').write_text(json.dumps({'args':args,'env':dict(os.environ),'cwd':cwd,'mode':stat.S_IMODE(os.stat(cwd).st_mode),'before':before}))
sys.stdout.write((root/'trace.jsonl').read_text())"#,
        &trace,
    );
    let parent_cwd = std::env::current_dir().unwrap();
    let parent_home = std::env::var_os("HOME");
    // Le budget peut être épuisé par la préparation ou le scheduling avant
    // l’entrée transport : aucune nouvelle durée ne doit autoriser un enfant.
    options.deadline = Instant::now();
    assert_eq!(
        run_isolated_claude_decision(&options),
        Err(ClaudeIsolatedDecisionError::Timeout)
    );
    assert!(!root.join("started").exists());
    assert!(!root.join("seen.json").exists());
    options.deadline = Instant::now() + Duration::from_secs(2);
    let result = run_isolated_claude_decision(&options);
    let observed: Value =
        serde_json::from_slice(&fs::read(root.join("seen.json")).unwrap()).unwrap();
    let child_cwd = PathBuf::from(observed["cwd"].as_str().unwrap());
    assert_eq!(result.unwrap().candidate_id, "c001");
    assert_eq!(observed["mode"], 0o700);
    assert_eq!(observed["before"], json!([]));
    assert_ne!(child_cwd, parent_cwd);
    assert!(!child_cwd.exists(), "répertoire enfant retiré après drain");
    let environment = observed["env"].as_object().unwrap();
    assert_eq!(environment["HOME"], root.to_string_lossy().as_ref());
    assert_eq!(environment["PATH"], "/usr/bin:/bin");
    for name in environment.keys() {
        assert!(
            matches!(
                name.as_str(),
                "HOME" | "PATH" | "CLAUDE_CONFIG_DIR" | "LC_CTYPE"
            ) || (cfg!(target_os = "macos") && name == "__CF_USER_TEXT_ENCODING"),
            "variable inattendue chez l'enfant : {name}"
        );
    }
    for forbidden in [
        "--bare",
        "--max-turns",
        "--resume",
        "--add-dir",
        "--dangerously-skip-permissions",
    ] {
        assert!(
            !observed["args"]
                .as_array()
                .unwrap()
                .iter()
                .any(|arg| arg == forbidden)
        );
    }
    assert_eq!(std::env::current_dir().unwrap(), parent_cwd);
    assert_eq!(std::env::var_os("HOME"), parent_home);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec168_stdin_non_lu_expire_depuis_le_lancement_et_arrete_enfant() {
    let (root, mut options) = isolated_fixture(
        "(root/'pid').write_text(str(os.getpid()))\ntime.sleep(5)",
        "",
    );
    options.request_json = "x".repeat(64 * 1024);
    // macOS : python3 démarre en ~300 ms ; l'échéance doit suivre l'écriture du pid.
    options.deadline = Instant::now() + Duration::from_millis(2000);
    let started = Instant::now();
    let result = run_isolated_claude_decision(&options);
    let elapsed = started.elapsed();
    let pid: i32 = fs::read_to_string(root.join("pid"))
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(result, Err(ClaudeIsolatedDecisionError::Timeout));
    assert!(Instant::now() >= options.deadline && elapsed < Duration::from_secs(4));
    // SAFETY: kill(pid, 0) est l'idiome POSIX de sonde d'existence ; aucun signal n'est envoyé.
    assert!(unsafe { libc::kill(pid, 0) } < 0);
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ESRCH)
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec168_stdout_excessif_est_borne_et_arrete_sans_attendre_la_fin_naturelle() {
    let (root, options) = isolated_fixture(
        "sys.stdin.read()\n(root/'pid').write_text(str(os.getpid()))\nos.write(1,b'x'*(64*1024+1))\ntime.sleep(5)",
        "",
    );
    let started = Instant::now();
    let result = run_isolated_claude_decision(&options);
    let pid: i32 = fs::read_to_string(root.join("pid"))
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(result, Err(ClaudeIsolatedDecisionError::OutputTooLarge));
    assert!(started.elapsed() < Duration::from_secs(2));
    // SAFETY: kill(pid, 0) est l'idiome POSIX de sonde d'existence ; aucun signal n'est envoyé.
    assert!(unsafe { libc::kill(pid, 0) } < 0);
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ESRCH)
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec168_sortie_leader_ne_libere_pas_un_descendant_qui_tient_stdout() {
    let trace = isolated_trace(&json!({"candidate_id":"c001"}));
    let (root, mut options) = isolated_fixture(
        r#"sys.stdin.read()
(root/'leader').write_text(str(os.getpid()))
pid=os.fork()
if pid==0:
    signal.signal(signal.SIGTERM,signal.SIG_IGN)
    def finish(*_):
        (root/'descendant-stopped').write_text('INT')
        sys.exit(0)
    signal.signal(signal.SIGINT,finish)
    (root/'descendant').write_text(str(os.getpid()))
    while True:
        signal.pause()
else:
    while not (root/'descendant').exists():
        time.sleep(.005)
    sys.stdout.write((root/'trace.jsonl').read_text())
    sys.stdout.flush()
    sys.exit(0)"#,
        &trace,
    );
    // Échéance après le fork même quand python3 démarre lentement (macOS chargé).
    options.deadline = Instant::now() + Duration::from_millis(2000);
    let result = run_isolated_claude_decision(&options);
    let leader: i32 = fs::read_to_string(root.join("leader"))
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(result, Err(ClaudeIsolatedDecisionError::Timeout));
    assert!(Instant::now() >= options.deadline);
    assert_eq!(
        fs::read_to_string(root.join("descendant-stopped")).unwrap(),
        "INT"
    );
    assert!(
        // SAFETY: kill(pid, 0) est l'idiome POSIX de sonde d'existence ; aucun signal n'est envoyé.
        unsafe { libc::kill(-leader, 0) } < 0,
        "groupe détenu entièrement drainé"
    );
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ESRCH)
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec168_exit_nonzero_refuse_meme_une_sortie_structuree_valide() {
    let (root, options) = isolated_fixture(
        "sys.stdin.read()\nsys.stdout.write((root/'trace.jsonl').read_text())\nsys.exit(7)",
        &isolated_trace(&json!({"candidate_id":"c001"})),
    );
    assert_eq!(
        run_isolated_claude_decision(&options),
        Err(ClaudeIsolatedDecisionError::InvalidOutput)
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn spec168_formateur_refuse_outils_capacites_et_reponses_contradictoires() {
    let trace = isolated_trace(&json!({"candidate_id":"c001","reason":"ok"}));
    let base: Vec<Value> = trace
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let mut invalid = Vec::new();
    let mut frames = base.clone();
    frames[0]["tools"] = json!(["StructuredOutput", "Bash"]);
    invalid.push(frames);
    for field in [
        "mcp_servers",
        "skills",
        "plugins",
        "slash_commands",
        "capabilities",
    ] {
        let mut frames = base.clone();
        frames[0][field] = json!(["forbidden"]);
        invalid.push(frames);
    }
    let mut frames = base.clone();
    frames[1]["message"]["content"][0]["name"] = json!("Bash");
    invalid.push(frames);
    let mut frames = base.clone();
    frames[1]["message"]["content"][0]["input"]["candidate_id"] = json!("c002");
    invalid.push(frames);
    let mut frames = base.clone();
    frames[2]["result"] = json!("{\"candidate_id\":\"c002\",\"reason\":\"ok\"}");
    invalid.push(frames);
    let mut frames = base.clone();
    frames[2]["is_error"] = json!(true);
    invalid.push(frames);
    let mut frames = base.clone();
    frames.push(base[2].clone());
    invalid.push(frames);
    let mut frames = base.clone();
    frames.insert(1, base[0].clone());
    invalid.push(frames);
    let mut frames = base.clone();
    frames.insert(2, base[1].clone());
    invalid.push(frames);
    invalid.push(base[1..].to_vec());
    let mut frames = base.clone();
    frames.insert(
        2,
        json!({"type":"control_request","request":{"type":"tool_use","name":"Bash"}}),
    );
    invalid.push(frames);
    for frames in invalid {
        let trace = frames
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            parse_isolated_decision_output(trace.as_bytes()),
            Err(ClaudeIsolatedDecisionError::InvalidOutput)
        );
    }
    let duplicate = trace.replace(
            "\"structured_output\":{\"candidate_id\":\"c001\",\"reason\":\"ok\"}",
            "\"structured_output\":{\"candidate_id\":\"c001\",\"candidate_id\":\"c001\",\"reason\":\"ok\"}",
        );
    assert_ne!(duplicate, trace);
    assert_eq!(
        parse_isolated_decision_output(duplicate.as_bytes()),
        Err(ClaudeIsolatedDecisionError::InvalidOutput)
    );
    // Ces deux refus passent aussi par le véritable exécutable et le drain.
    for trace in [
        trace.replace("StructuredOutput", "Bash"),
        isolated_trace(&json!({"candidate_id":"c001","command":"forbidden"})),
    ] {
        let (root, options) = isolated_fixture(
            "sys.stdin.read()\nsys.stdout.write((root/'trace.jsonl').read_text())",
            &trace,
        );
        assert_eq!(
            run_isolated_claude_decision(&options),
            Err(ClaudeIsolatedDecisionError::InvalidOutput)
        );
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn spec168_objets_json_sont_bornes_avant_leur_decodage_detaille() {
    let base = isolated_trace(&json!({"candidate_id":"c001"}));
    assert_eq!(
        parse_isolated_decision_output(base.as_bytes())
            .unwrap()
            .candidate_id,
        "c001"
    );
    let large = isolated_trace(&json!({"candidate_id":"c001","reason":"x".repeat(4096)}));
    let (root, options) = isolated_fixture(
        "sys.stdin.read()\nsys.stdout.write((root/'trace.jsonl').read_text())",
        &large,
    );
    assert_eq!(
        run_isolated_claude_decision(&options),
        Err(ClaudeIsolatedDecisionError::OutputTooLarge)
    );
    fs::remove_dir_all(root).unwrap();
    for field in ["structured_output", "result"] {
        let mut frames: Vec<Value> = base
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        frames[2][field] = if field == "result" {
            json!(format!("{}{{\"candidate_id\":\"c001\"}}", " ".repeat(4096)))
        } else {
            json!({"candidate_id":"c001","unknown":"x".repeat(4096)})
        };
        let trace = frames
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(
            parse_isolated_decision_output(trace.as_bytes()),
            Err(ClaudeIsolatedDecisionError::OutputTooLarge)
        );
    }
    // Même sans JSON valide, aucun parsing n'a lieu au-delà de la borne stdout.
    assert_eq!(
        parse_isolated_decision_output(&vec![b'x'; MAX_ISOLATED_DECISION_STDOUT_BYTES + 1]),
        Err(ClaudeIsolatedDecisionError::OutputTooLarge)
    );
    // Les espaces comptent aussi dans les 4 Kio bruts de StructuredOutput.
    let padded = base.replacen(
        "\"input\":{\"candidate_id\":\"c001\"}",
        &format!(
            "\"input\":{{{}\"candidate_id\":\"c001\"}}",
            " ".repeat(4096)
        ),
        1,
    );
    assert_ne!(padded, base);
    assert_eq!(
        parse_isolated_decision_output(padded.as_bytes()),
        Err(ClaudeIsolatedDecisionError::OutputTooLarge)
    );
}
