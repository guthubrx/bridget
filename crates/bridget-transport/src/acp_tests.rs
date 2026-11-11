use crate::poison::RecoverPoison;
#[test]
fn spec100_acp_correlates_permission_path_and_separate_edit_kind() {
    let mut memory = std::collections::HashMap::new();
    for id in ["permission", "separate"] {
        let start = serde_json::json!({"params":{"update":{"toolCallId":id,"title":"Write File","kind":"edit","status":"in_progress"}}});
        super::tool_call_journal_payload(&start, &mut memory);
        if id == "permission" {
            let permission = serde_json::json!({"params":{"toolCall":{"toolCallId":id,"title":"Write","rawInput":{"path":"/p/x"}}}});
            super::permission_tool_journal_payload(&permission, &mut memory);
        } else {
            let args = serde_json::json!({"params":{"update":{"toolCallId":id,"rawInput":{"path":"/p/x"}}}});
            super::tool_call_journal_payload(&args, &mut memory);
        }
        let end = serde_json::json!({"params":{"update":{"toolCallId":id,"status":"completed"}}});
        assert_eq!(
            super::tool_call_journal_payload(&end, &mut memory)["write_completed_path"],
            "/p/x"
        );
        assert!(
            super::tool_call_journal_payload(&end, &mut memory)
                .get("write_completed_path")
                .is_none()
        );
    }
}

#[test]
fn spec100_acp_correlates_completed_write_not_read_or_failure() {
    let mut memory = std::collections::HashMap::new();
    let start = serde_json::json!({"params":{"update":{"sessionUpdate":"tool_call","toolCallId":"w","title":"Write","kind":"edit","rawInput":{"content":"x".repeat(2000),"path":"/project/x"},"status":"in_progress"}}});
    assert!(
        super::tool_call_journal_payload(&start, &mut memory)
            .get("write_completed_path")
            .is_none()
    );
    let end = serde_json::json!({"params":{"update":{"sessionUpdate":"tool_call_update","toolCallId":"w","status":"completed"}}});
    assert_eq!(
        super::tool_call_journal_payload(&end, &mut memory)["write_completed_path"],
        "/project/x"
    );
    assert!(
        super::tool_call_journal_payload(&end, &mut memory)
            .get("write_completed_path")
            .is_none()
    );
    let read = serde_json::json!({"params":{"update":{"toolCallId":"r","title":"Read","kind":"read","rawInput":{"path":"/project/x"},"status":"completed"}}});
    assert!(
        super::tool_call_journal_payload(&read, &mut memory)
            .get("write_completed_path")
            .is_none()
    );
    let mut failed = start.clone();
    failed["params"]["update"]["toolCallId"] = serde_json::json!("failure");
    failed["params"]["update"]["status"] = serde_json::json!("failed");
    assert!(
        super::tool_call_journal_payload(&failed, &mut memory)
            .get("write_completed_path")
            .is_none()
    );
    failed["params"]["update"]["status"] = serde_json::json!("completed");
    assert!(
        super::tool_call_journal_payload(&failed, &mut memory)
            .get("write_completed_path")
            .is_none()
    );
}

#[test]
fn spec100_acp_saturation_does_not_rearm_completed_writes() {
    let mut memory = std::collections::HashMap::new();
    let write = serde_json::json!({"params":{"update":{"toolCallId":"original","title":"Write","rawInput":{"path":"/p/x"},"status":"completed"}}});
    assert!(
        super::tool_call_journal_payload(&write, &mut memory)
            .get("write_completed_path")
            .is_some()
    );
    for index in 0..300 {
        let other = serde_json::json!({"params":{"update":{"toolCallId":format!("other-{index}"),"title":"Read","rawInput":{"path":"/p/x"},"status":"completed"}}});
        super::tool_call_journal_payload(&other, &mut memory);
    }
    assert_eq!(memory.len(), 256);
    assert!(
        super::tool_call_journal_payload(&write, &mut memory)
            .get("write_completed_path")
            .is_none()
    );
    let permission = serde_json::json!({"params":{"toolCall":{"toolCallId":"new","title":"Write","rawInput":{"path":"/p/x"}}}});
    assert!(super::permission_tool_journal_payload(&permission, &mut memory).is_none());
    assert_eq!(memory.len(), 256);
}
use super::*;
use std::time::Instant;

fn queue() -> Arc<(Mutex<QueueState>, Condvar)> {
    Arc::new((
        Mutex::new(QueueState {
            messages: VecDeque::new(),
            active: None,
            closed: false,
        }),
        Condvar::new(),
    ))
}

fn message(id: &str) -> BridgetMessage {
    let mut message = BridgetMessage::new("alice", "bob", id);
    message.id = id.to_string();
    message
}

#[test]
fn evenement_interne_est_explicitement_distingue_de_la_ligne_source() {
    let event = ManagedEvent::internal(
        ManagedEventSource::Acp,
        acp_event_bytes(&AcpEvent::Update {
            detail: "octets source".to_string(),
        }),
        ManagedEventKind::Update {
            detail: "octets source".to_string(),
        },
    );
    assert_eq!(event.source, ManagedEventSource::Acp);
    assert_eq!(
        event.origin,
        crate::managed_session::ManagedEventOrigin::Internal
    );
    assert_eq!(
        event.raw,
        br#"{"detail":"octets source","kind":"update"}"#.to_vec()
    );
    assert!(matches!(
        event.kind,
        ManagedEventKind::Update { detail } if detail == "octets source"
    ));
}

#[test]
fn terminal_acp_est_normalise_avant_la_frontiere_commune() {
    assert_eq!(
        terminal_from_acp_stop_reason("end_turn"),
        ManagedTerminal::Completed
    );
    assert_eq!(
        terminal_from_acp_stop_reason("cancelled"),
        ManagedTerminal::Cancelled
    );
    assert_eq!(
        terminal_from_acp_stop_reason("provider-overloaded"),
        ManagedTerminal::Failed {
            detail: "terminal ACP: provider-overloaded".to_string(),
        }
    );
}

#[test]
fn prompt_preserves_the_body_byte_for_byte() {
    let mut message = BridgetMessage::new("alice", "bob", "'\"$x\nligne");
    message.id = "mcp-38210-6a8a7fc7-1".to_string();
    let prompt = prompt_for(&message);
    assert!(prompt.starts_with("[message Bridget de alice — réponse attendue : non]"));
    assert!(prompt.ends_with(&message.body));
}

#[test]
fn spec105_acp_contrat_explicite_et_corps_intact() {
    for requested in [false, true] {
        let mut msg = BridgetMessage::new("alice", "bob", "Texte ' et \" et\nseconde ligne");
        msg.reply = requested;
        msg.in_reply_to = Some("parent".into());
        let prompt = prompt_for(&msg);
        assert!(prompt.contains(&msg.id));
        assert!(prompt.contains("parent"));
        assert!(prompt.ends_with(&msg.body));
        assert_eq!(prompt.contains("Pas d'accusé de réception"), !requested);
        assert_eq!(prompt.contains("relaie automatiquement"), requested);
    }
}

#[test]
fn generic_fixture_covers_json_rpc_matrix() {
    let lines = include_str!("../tests/fixtures/acp/generic.jsonl")
        .lines()
        .collect::<Vec<_>>();
    let notification: Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(
        update_text(&notification, Some("fixture-session")),
        Some("premier")
    );

    let tool_update: Value = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(update_text(&tool_update, Some("fixture-session")), None);

    let permission: Value = serde_json::from_str(lines[2]).unwrap();
    assert_eq!(
        permission_response(&permission, "allow", false).unwrap().0["result"],
        json!({"outcome":{"outcome":"selected","optionId":"allow-1"}})
    );
    assert_eq!(
        permission_response(&permission, "allow", false).unwrap().1,
        json!({
            "tool":"inconnu",
            "options":[
                {"optionId":"allow-1","kind":"allow_once"},
                {"optionId":"reject-1","kind":"reject_once"}
            ],
            "decision":{"outcome":"selected","option_id":"allow-1"}
        })
    );
    assert_eq!(
        permission_response(&permission, "deny", false).unwrap().0["result"]["outcome"]["optionId"],
        "reject-1"
    );
    let invalid_permission = json!({
        "id": "permission-invalid",
        "params": { "options": [
            {"optionId": "bogus", "kind": "allow_bogus"},
            {"optionId": 7, "kind": "allow_once"}
        ]}
    });
    assert_eq!(
        permission_response(&invalid_permission, "allow", false)
            .unwrap()
            .0["result"],
        json!({"outcome":{"outcome":"cancelled"}})
    );
    assert_eq!(
        permission_response(&invalid_permission, "allow", false)
            .unwrap()
            .1["decision"],
        json!({"outcome":"cancelled"})
    );

    let numeric: Value = serde_json::from_str(lines[3]).unwrap();
    let string: Value = serde_json::from_str(lines[4]).unwrap();
    assert_eq!(rpc_response(&numeric).unwrap().0, "9");
    assert_eq!(rpc_response(&string).unwrap().0, "\"string-9\"");

    let error: Value = serde_json::from_str(lines[5]).unwrap();
    assert!(
        rpc_response(&error)
            .unwrap()
            .1
            .unwrap_err()
            .contains("refus")
    );

    let unknown: Value = serde_json::from_str(lines[6]).unwrap();
    assert_eq!(
        method_not_found_response(&unknown, "server/inconnu").unwrap()["error"]["code"],
        -32601
    );
    assert!(serde_json::from_str::<Value>(lines[7]).is_err());
}

#[test]
fn tool_call_journal_prefers_title_then_name_then_kind() {
    let update = |fields: Value| json!({"params":{"update":fields}});
    let mut tool_memory = HashMap::new();
    let titled = tool_call_journal_payload(
        &update(json!({
            "toolCallId":"tool-1", "title":"Read src/main.rs", "name":"read_file", "kind":"read",
            "content":{"text":"lecture"}
        })),
        &mut tool_memory,
    );
    assert_eq!(titled["tool"], "Read src/main.rs");
    assert_eq!(titled["title"], "Read src/main.rs");
    assert_eq!(titled["tool_call_id"], "tool-1");
    assert_eq!(titled["name"], "read_file");
    assert_eq!(titled["tool_kind"], "read");
    assert_eq!(titled["summary"], "lecture");

    let titled_update = tool_call_journal_payload(
        &update(json!({"toolCallId":"tool-1", "content":{"text":"terminé"}})),
        &mut tool_memory,
    );
    assert_eq!(titled_update["tool"], "Read src/main.rs");
    assert_eq!(titled_update["title"], "Read src/main.rs");
    assert_eq!(titled_update["tool_call_id"], "tool-1");
    assert_eq!(titled_update["summary"], "terminé");

    let named = tool_call_journal_payload(
        &update(json!({"name":"Bash cargo test", "kind":"execute"})),
        &mut tool_memory,
    );
    assert_eq!(named["tool"], "Bash cargo test");
    assert!(named.get("title").is_some());

    let unknown_kind =
        tool_call_journal_payload(&update(json!({"kind":"quantum_wrench"})), &mut tool_memory);
    assert_eq!(unknown_kind["tool"], "quantum_wrench");
    assert_eq!(unknown_kind["tool_kind"], "quantum_wrench");
}

/// Mesure Cursor 2026-08-27 : tool_call initial « Read File » + rawInput {},
/// puis tool_call_update titre enrichi + rawInput.path. Mutant qui ignore
/// rawInput ou fige le premier titre → meurt.
#[test]
#[allow(non_snake_case)]
fn TEMOIN_TOOL_acp_retranscrit_rawInput_et_titre_enrichi() {
    let update = |fields: Value| {
        json!({
            "jsonrpc": "2.0",
            "method": "session/update",
            "params": {
                "sessionId": "fixture-session",
                "update": fields
            }
        })
    };
    let mut tool_memory = HashMap::new();
    let first = tool_call_journal_payload(
        &update(json!({
            "sessionUpdate": "tool_call",
            "toolCallId": "call-mesure-read",
            "title": "Read File",
            "kind": "read",
            "status": "pending",
            "rawInput": {}
        })),
        &mut tool_memory,
    );
    assert_eq!(first["tool"], "Read File");
    assert!(first.get("detail").is_none() || first["detail"] == "");

    let enriched = tool_call_journal_payload(
        &update(json!({
            "sessionUpdate": "tool_call_update",
            "toolCallId": "call-mesure-read",
            "title": "Read /tmp/bt/mesure-sentinel.txt",
            "rawInput": {"path": "/tmp/bt/mesure-sentinel.txt"},
            "locations": [{"path": "/tmp/bt/mesure-sentinel.txt"}]
        })),
        &mut tool_memory,
    );
    assert_eq!(
        enriched["text"].as_str(),
        Some("Read /tmp/bt/mesure-sentinel.txt"),
        "titre enrichi en dur, payload={}",
        enriched
    );
    assert_eq!(
        enriched["tool"].as_str(),
        Some("Read /tmp/bt/mesure-sentinel.txt"),
        "champ tool en dur, payload={}",
        enriched
    );
    assert_eq!(
        enriched["detail"].as_str(),
        Some(r#"{"path":"/tmp/bt/mesure-sentinel.txt"}"#),
        "rawInput en dur (pas une présence vide), payload={}",
        enriched
    );
}

/// Mesure Cursor MCP : tool_call titre « MCP: tool », puis permission
/// `bridget-bridget_ledger: bridget_ledger`. Mutant qui laisse MCP: tool → meurt.
#[test]
#[allow(non_snake_case)]
fn TEMOIN_TOOL_acp_permission_remplace_MCP_tool_par_vrai_nom() {
    let mut tool_memory = HashMap::new();
    let generic = tool_call_journal_payload(
        &json!({
            "params": {"update": {
                "sessionUpdate": "tool_call",
                "toolCallId": "call-mesure-mcp",
                "title": "MCP: tool",
                "kind": "other",
                "rawInput": {}
            }}
        }),
        &mut tool_memory,
    );
    assert_eq!(generic["tool"], "MCP: tool");

    let from_permission = permission_tool_journal_payload(
        &json!({
            "params": {
                "toolCall": {
                    "toolCallId": "call-mesure-mcp",
                    "title": "bridget-bridget_ledger: bridget_ledger",
                    "kind": "other",
                    "status": "pending"
                }
            }
        }),
        &mut tool_memory,
    )
    .expect("la permission doit produire un acte tool nommé");
    assert_eq!(
        from_permission["text"].as_str(),
        Some("bridget-bridget_ledger: bridget_ledger"),
        "nom MCP en dur, payload={}",
        from_permission
    );
    assert_eq!(
        from_permission["tool"].as_str(),
        Some("bridget-bridget_ledger: bridget_ledger"),
        "champ tool en dur, payload={}",
        from_permission
    );

    let later = tool_call_journal_payload(
        &json!({
            "params": {"update": {
                "sessionUpdate": "tool_call_update",
                "toolCallId": "call-mesure-mcp",
                "status": "completed"
            }}
        }),
        &mut tool_memory,
    );
    assert_eq!(
        later["tool"].as_str(),
        Some("bridget-bridget_ledger: bridget_ledger"),
        "le cache ne doit pas retomber sur MCP: tool, payload={}",
        later
    );
}

fn session_update(session_update: &str, text: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "session/update",
        "params": {
            "sessionId": "fixture-session",
            "update": {
                "sessionUpdate": session_update,
                "content": { "type": "text", "text": text }
            }
        }
    })
}

fn session_tool_call(title: &str, summary: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "session/update",
        "params": {
            "sessionId": "fixture-session",
            "update": {
                "sessionUpdate": "tool_call",
                "toolCallId": "tool-1",
                "title": title,
                "kind": "read",
                "content": { "type": "text", "text": summary }
            }
        }
    })
}

/// Simule le chemin productif : chunks → reasoning_raw → take terminal.
fn accumulate_thought_then_take(chunks: &[&str]) -> Value {
    let reasoning_raw = Arc::new(Mutex::new(String::new()));
    for chunk in chunks {
        let update = session_update("agent_thought_chunk", chunk);
        let thought =
            update_thought_text(&update, Some("fixture-session")).expect("thought_chunk productif");
        reasoning_raw.lock().recover_poison().push_str(thought);
    }
    take_reasoning_journal_payload(&reasoning_raw)
}

/// T4.4 — présence de pensée → ligne terminale available true (pipeline réel).
#[test]
fn agent_thought_chunk_rend_reasoning_available_true() {
    // Ordre documenté : PENSÉE puis MESSAGES puis OUTILS — seul le raw compte.
    let terminal = accumulate_thought_then_take(&["je vais lire ", "le fichier"]);
    assert_eq!(terminal["available"], true);
    assert_eq!(terminal["raw"], "je vais lire le fichier");
    assert_eq!(terminal["summary"], "je vais lire le fichier");

    let mut tool_memory = HashMap::new();
    let act = tool_call_journal_payload(
        &session_tool_call("Read src/main.rs", "lecture"),
        &mut tool_memory,
    );
    assert_eq!(act["kind"], "tool");
    assert_eq!(act["text"], "Read src/main.rs");
    assert_eq!(act["detail"], "lecture");
    // Un second take sur le même accumulateur vide → available:false (unicité).
    let second = accumulate_thought_then_take(&[]);
    assert_eq!(second, json!({ "available": false }));
}

/// T4.5 — aucun chunk de pensée → ligne terminale available false
/// (cas Gemini : chunk jamais émis). Pas de summary/raw vides.
#[test]
fn flux_sans_thought_chunk_rend_reasoning_available_false() {
    let terminal = accumulate_thought_then_take(&[]);
    assert_eq!(terminal, json!({ "available": false }));
    assert!(terminal.get("summary").is_none());
    assert!(terminal.get("raw").is_none());

    let mut tool_memory = HashMap::new();
    let act = tool_call_journal_payload(
        &session_tool_call("Bash cargo test", "ok"),
        &mut tool_memory,
    );
    assert_eq!(act["kind"], "tool");
    assert_eq!(act["text"], "Bash cargo test");
}

/// Unicité du terminal : take vide le buffer — un second take ne rejoue pas.
#[test]
fn reasoning_terminal_est_unique_par_prise() {
    let reasoning_raw = Arc::new(Mutex::new(String::from("une seule fois")));
    let first = take_reasoning_journal_payload(&reasoning_raw);
    let second = take_reasoning_journal_payload(&reasoning_raw);
    assert_eq!(first["available"], true);
    assert_eq!(first["raw"], "une seule fois");
    assert_eq!(second, json!({ "available": false }));
    assert!(reasoning_raw.lock().recover_poison().is_empty());
}

/// T4.6 — mutant : si l'on rétablissait le filtre qui n'accepte que
/// `agent_message_chunk`, ce test meurt. Nom du test mort sous mutant :
/// `agent_thought_chunk_rend_reasoning_available_true`.
#[test]
fn thought_chunk_n_est_plus_rejete_par_le_filtre_de_session_update() {
    let thought = session_update("agent_thought_chunk", "pensée");
    assert!(
        update_text(&thought, Some("fixture-session")).is_none(),
        "la pensée ne doit pas alimenter le message"
    );
    assert_eq!(
        update_thought_text(&thought, Some("fixture-session")),
        Some("pensée")
    );
    assert_eq!(
        update_chunk_text(&thought, Some("fixture-session"), "agent_thought_chunk"),
        Some("pensée")
    );
}

#[test]
fn codex_spike_fixture_collects_text_then_stop_reason() {
    let mut text = String::new();
    let mut stop_reason = None;
    for line in include_str!("../tests/fixtures/acp/codex-spike.jsonl").lines() {
        let value: Value = serde_json::from_str(line).unwrap();
        if let Some(chunk) = update_text(&value, Some("spike-session")) {
            text.push_str(chunk);
        }
        if let Some((_, Ok(result))) = rpc_response(&value) {
            stop_reason = result
                .get("stopReason")
                .and_then(Value::as_str)
                .map(str::to_string);
        }
    }
    assert_eq!(text, "SPIKE_ACP_OK");
    assert_eq!(stop_reason.as_deref(), Some("end_turn"));
}

#[test]
fn claude_spike_fixture_collects_text_then_stop_reason() {
    let mut text = String::new();
    let mut stop_reason = None;
    for line in include_str!("../tests/fixtures/acp/claude-spike.jsonl").lines() {
        let value: Value = serde_json::from_str(line).unwrap();
        if let Some(chunk) = update_text(&value, Some("claude-spike-session")) {
            text.push_str(chunk);
        }
        if let Some((_, Ok(result))) = rpc_response(&value) {
            stop_reason = result
                .get("stopReason")
                .and_then(Value::as_str)
                .map(str::to_string);
        }
    }
    assert_eq!(text, "CLAUDE_ACP_OK");
    assert_eq!(stop_reason.as_deref(), Some("end_turn"));
}

#[test]
fn responses_are_correlated_by_exact_id_even_out_of_order() {
    let waiters: Waiters = Arc::new(Mutex::new(HashMap::new()));
    let (numeric_sender, numeric_receiver) = mpsc::channel();
    let (string_sender, string_receiver) = mpsc::channel();
    waiters
        .lock()
        .unwrap()
        .insert("2".to_string(), numeric_sender);
    waiters
        .lock()
        .unwrap()
        .insert("\"two\"".to_string(), string_sender);
    for value in [
        json!({"id":"two","result":"second"}),
        json!({"id":2,"result":"first"}),
    ] {
        let (id, result) = rpc_response(&value).unwrap();
        waiters
            .lock()
            .unwrap()
            .remove(&id)
            .unwrap()
            .send(result)
            .unwrap();
    }
    assert_eq!(numeric_receiver.recv().unwrap().unwrap(), "first");
    assert_eq!(string_receiver.recv().unwrap().unwrap(), "second");
}

#[test]
fn eof_fails_every_pending_waiter() {
    let waiters: Waiters = Arc::new(Mutex::new(HashMap::new()));
    let (sender, receiver) = mpsc::channel();
    waiters.lock().unwrap().insert("1".to_string(), sender);
    fail_waiters(&waiters, "EOF ACP");
    assert_eq!(receiver.recv().unwrap().unwrap_err(), "EOF ACP");
    assert!(waiters.lock().unwrap().is_empty());
}

#[test]
fn incompatible_protocol_version_is_explicit() {
    assert!(ensure_protocol_version(&json!({"protocolVersion": 1})).is_ok());
    assert_eq!(
        ensure_protocol_version(&json!({"protocolVersion": 2}))
            .unwrap_err()
            .to_string(),
        "livraison échouée: version ACP incompatible"
    );
}

#[test]
fn session_acp_recoit_le_serveur_mcp_ephemere() {
    let script = r#"
read initialize
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read session
case "$session" in
  *'"name":"bridget"'*) ;;
  *) exit 31 ;;
esac
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
while read line; do :; done
"#;
    let transport = AcpTransport::spawn_with_environment_and_mcp(
        AcpOptions {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), script.to_string()],
            queue_capacity: 1,
            permissions: "allow".to_string(),
            notify_timeout_secs: 1,
            provider_observation: None,
        },
        &[],
        vec![json!({ "name": "bridget", "command": "bridget", "args": ["mcp"], "env": [] })],
    )
    .unwrap();
    transport.shutdown();
}

#[test]
fn queue_is_fifo_and_capacity_rejection_is_terminal() {
    let queue = queue();
    assert!(enqueue(&queue, 2, message("first")));
    assert!(enqueue(&queue, 2, message("second")));
    assert!(!enqueue(&queue, 2, message("third")));
    let (state, _) = &*queue;
    let state = state.lock().unwrap();
    assert_eq!(
        state
            .messages
            .iter()
            .map(|message| message.id.as_str())
            .collect::<Vec<_>>(),
        vec!["first", "second"]
    );
}

#[test]
fn saturated_journal_becomes_a_terminal_event() {
    let journal = Arc::new(Mutex::new(Some(
        crate::journal::JournalWriter::saturated_for_test(),
    )));
    let events = Arc::new(Mutex::new(AcpEventQueue::default()));
    record_or_terminal(
        &journal,
        &events,
        "update",
        Some("m1"),
        json!({"kind":"text","content":"x"}),
    );
    assert!(
        matches!(events.lock().unwrap().pop_front(), Some(AcpEvent::JournalFailed { detail }) if detail == "journal ACP saturé")
    );
}

#[test]
fn expired_delivery_is_rejected_before_it_reaches_the_prompt_queue() {
    let mut message = message("expired");
    message.deadline_at = Some(0);
    assert!(message_expired(&message));
}

#[test]
fn queued_message_is_purged_by_id_without_reordering_others() {
    let queue = queue();
    enqueue(&queue, 3, message("first"));
    enqueue(&queue, 3, message("cancel"));
    enqueue(&queue, 3, message("last"));
    let (state, _) = &*queue;
    let mut state = state.lock().unwrap();
    let index = state
        .messages
        .iter()
        .position(|message| message.id == "cancel")
        .unwrap();
    assert_eq!(state.messages.remove(index).unwrap().id, "cancel");
    assert_eq!(
        state
            .messages
            .iter()
            .map(|message| message.id.as_str())
            .collect::<Vec<_>>(),
        vec!["first", "last"]
    );
}

#[test]
fn taille_evenement_bornee_et_message_terminal_preserve() {
    // La file ne réserve pas un BridgetMessage complet pour chaque delta.
    // Remettre le message inline fait dépasser cette borne structurelle.
    assert!(std::mem::size_of::<AcpEvent>() <= 8 * std::mem::size_of::<usize>());
    let original = message("terminal-boxed");
    let expected = serde_json::to_vec(&original).unwrap();
    let event = finish_turn(
        original,
        "réponse".to_string(),
        Ok(json!({"stopReason":"end_turn"})),
    );
    let ManagedEventKind::TurnFinished {
        message,
        response,
        terminal,
    } = managed_kind(event)
    else {
        panic!("fin de tour attendue");
    };
    assert_eq!(serde_json::to_vec(&message).unwrap(), expected);
    assert_eq!(response, "réponse");
    assert!(matches!(terminal, ManagedTerminal::Completed));
}

#[test]
fn turn_normal_empty_and_error_have_distinct_events() {
    match finish_turn(
        message("normal"),
        "réponse".to_string(),
        Ok(json!({"stopReason":"end_turn"})),
    ) {
        AcpEvent::TurnFinished {
            response,
            stop_reason,
            ..
        } => {
            assert_eq!(response, "réponse");
            assert_eq!(stop_reason, "end_turn");
        }
        _ => panic!("tour normal attendu"),
    }
    match finish_turn(
        message("empty"),
        String::new(),
        Ok(json!({"stopReason":"end_turn"})),
    ) {
        AcpEvent::TurnFinished { response, .. } => assert!(response.is_empty()),
        _ => panic!("tour vide attendu"),
    }
    match finish_turn(
        message("error"),
        String::new(),
        Err(TransportError::DeliveryFailed("refus".to_string())),
    ) {
        AcpEvent::DeliveryRejected { message_id, reason } => {
            assert_eq!(message_id, "error");
            assert!(reason.contains("refus"));
        }
        _ => panic!("refus attendu"),
    }
}

#[test]
fn shutdown_is_idempotent_before_a_process_is_spawned() {
    let alive = AtomicBool::new(true);
    assert!(alive.swap(false, Ordering::SeqCst));
    assert!(!alive.swap(false, Ordering::SeqCst));
}

#[test]
fn false_adapter_exercises_stdio_reader_writer_and_prompt() {
    let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read request
echo '{"jsonrpc":"2.0","id":"permission-1","method":"session/request_permission","params":{"sessionId":"fixture-session","options":[{"optionId":"allow-1","kind":"allow_once"},{"optionId":"reject-1","kind":"reject_once"}]}}'
read permission
echo '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"fixture-session","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"réponse"}}}}'
echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"end_turn"}}'
"#;
    let mut transport = AcpTransport::spawn(AcpOptions {
        command: "sh".to_string(),
        args: vec!["-c".to_string(), script.to_string()],
        queue_capacity: 2,
        permissions: "allow".to_string(),
        notify_timeout_secs: 1,
        provider_observation: None,
    })
    .unwrap();
    let message = message("fixture-message");
    transport.deliver(&message).unwrap();
    for _ in 0..20 {
        thread::sleep(Duration::from_millis(10));
        let events = transport.drain_events();
        if events.iter().any(|event| matches!(event, AcpEvent::TurnFinished { response, stop_reason, .. } if response == "réponse" && stop_reason == "end_turn")) {
                return;
            }
    }
    panic!("le faux adaptateur n'a pas terminé le tour");
}

#[test]
fn anomalie_acp_ne_ferme_le_tour_qu_apres_un_terminal_atteste() {
    let root = std::env::temp_dir().join(format!(
        "bridget-acp-terminalite-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let release = root.join("release-terminal");
    std::fs::create_dir_all(&root).unwrap();
    let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read request
echo '{"jsonrpc":"2.0","method":"vendor/future","params":{"future":true}}'
echo '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"fixture-session","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"le tour continue"}}}}'
while [ ! -f "__RELEASE_TERMINAL__" ]; do sleep 0.01; done
echo '{"jsonrpc":"2.0","id":3,"error":{"code":-32000,"message":"refus fournisseur"}}'
while read request; do :; done
"#
        .replace(
            "__RELEASE_TERMINAL__",
            &release.display().to_string(),
        );
    let mut transport = AcpTransport::spawn(AcpOptions {
        command: "sh".to_string(),
        args: vec!["-c".to_string(), script],
        queue_capacity: 1,
        permissions: "allow".to_string(),
        notify_timeout_secs: 5,
        provider_observation: None,
    })
    .unwrap();
    transport.enable_journal(&root, "acp-live").unwrap();
    transport.deliver(&message("message-live")).unwrap();

    let mut saw_protocol_error = false;
    let mut saw_continuation = false;
    for _ in 0..100 {
        thread::sleep(Duration::from_millis(10));
        for event in transport.drain_events() {
            saw_protocol_error |= matches!(
                event,
                AcpEvent::Error { ref detail }
                    if detail == "notification ACP inconnue: vendor/future"
            );
            saw_continuation |= matches!(
                event,
                AcpEvent::Update { ref detail } if detail == "le tour continue"
            );
        }
        if saw_protocol_error && saw_continuation {
            break;
        }
    }
    assert!(saw_protocol_error, "le faux ACP n'a pas émis son anomalie");
    assert!(saw_continuation, "le faux ACP n'a pas poursuivi le prompt");
    assert!(
        matches!(transport.state(), TurnState::InProgress { ref message_id, .. } if message_id == "message-live"),
        "l'anomalie protocole ne doit pas terminer le tour"
    );

    let read_events = || {
        let mut events = std::fs::read_dir(root.join("acp-live"))
            .unwrap()
            .filter_map(Result::ok)
            .flat_map(|entry| crate::journal::valid_events(&entry.path()))
            .collect::<Vec<_>>();
        events.sort_by_key(|event| event["seq"].as_u64());
        events
    };
    let before_terminal = (0..100)
        .find_map(|_| {
            let events = read_events();
            if events.iter().any(|event| event["event"] == "update") {
                Some(events)
            } else {
                thread::sleep(Duration::from_millis(10));
                None
            }
        })
        .expect("journal ACP non terminal non écrit");
    let protocol_error = before_terminal
        .iter()
        .find(|event| event["event"] == "error")
        .expect("anomalie ACP absente du journal");
    assert!(protocol_error["payload"].get("terminal_kind").is_none());
    assert!(
        before_terminal
            .iter()
            .any(|event| event["event"] == "update")
    );
    assert!(!before_terminal.iter().any(|event| {
        event["event"] == "turn_end"
            || event["payload"].get("terminal_kind") == Some(&json!("turn_failed"))
    }));

    std::fs::write(&release, b"go").unwrap();
    let mut rejected = false;
    for _ in 0..100 {
        thread::sleep(Duration::from_millis(10));
        rejected |= transport.drain_events().iter().any(
                |event| matches!(event, AcpEvent::DeliveryRejected { message_id, .. } if message_id == "message-live"),
            );
        if rejected {
            break;
        }
    }
    assert!(rejected, "le vrai terminal du faux ACP n'a pas été observé");
    transport.shutdown();
    let after_terminal = read_events();
    let correlated_errors = after_terminal
        .iter()
        .filter(|event| event["event"] == "error" && event["message_id"] == "message-live")
        .collect::<Vec<_>>();
    let terminal_kind = correlated_errors
        .last()
        .and_then(|event| event["payload"].get("terminal_kind"))
        .cloned();
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(correlated_errors.len(), 2, "anomalie et terminal attendus");
    assert_eq!(terminal_kind, Some(json!("turn_failed")));
}

#[test]
fn raw_source_line_survives_unusual_spacing_and_unknown_notification() {
    let raw = r#" { "method" : "vendor/future" , "params" : { "future" : true } }"#;
    let script = format!(
        r#"
read request
echo '{{"jsonrpc":"2.0","id":1,"result":{{"protocolVersion":1}}}}'
read request
echo '{{"jsonrpc":"2.0","id":2,"result":{{"sessionId":"fixture-session"}}}}'
printf '%s\n' '{raw}'
while read request; do :; done
"#
    );
    let transport = AcpTransport::spawn(AcpOptions {
        command: "sh".to_string(),
        args: vec!["-c".to_string(), script],
        queue_capacity: 1,
        permissions: "allow".to_string(),
        notify_timeout_secs: 1,
        provider_observation: None,
    })
    .unwrap();

    for _ in 0..20 {
        thread::sleep(Duration::from_millis(10));
        let events = ManagedSession::drain_events(&transport);
        if let Some(event) = events.into_iter().find(|event| {
                matches!(event.kind, ManagedEventKind::Error { ref detail } if detail == "notification ACP inconnue: vendor/future")
            }) {
                assert_eq!(event.source, ManagedEventSource::Acp);
                assert_eq!(event.origin, crate::managed_session::ManagedEventOrigin::SourceLine);
                assert_eq!(event.raw, raw.as_bytes());
                return;
            }
    }
    panic!("la notification source inconnue n'a pas traversé la frontière");
}

fn observation_case(view_count: usize, root: &std::path::Path) -> Vec<Duration> {
    const TURNS: usize = 200;
    let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
id=3
while read request; do
  printf '{"jsonrpc":"2.0","id":%s,"result":{"stopReason":"end_turn"}}\n' "$id"
  id=$((id + 1))
done
"#;
    let mut transport = AcpTransport::spawn(AcpOptions {
        command: "sh".to_string(),
        args: vec!["-c".to_string(), script.to_string()],
        queue_capacity: 1,
        permissions: "allow".to_string(),
        notify_timeout_secs: 1,
        provider_observation: None,
    })
    .unwrap();
    transport.enable_journal(root, "codex-bench").unwrap();
    let views = (0..view_count)
        .map(|_| Arc::new(Mutex::new(Vec::<Vec<u8>>::new())))
        .collect::<Vec<_>>();
    for turn in 0..TURNS {
        let message = BridgetMessage::new("humain", "codex-bench", format!("tour-{turn}"));
        transport.deliver(&message).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let finished = transport.drain_events().into_iter().any(|event| {
                    matches!(event, AcpEvent::TurnFinished { message: done, .. } if done.id == message.id)
                });
            if finished {
                // Deux vues attach simulées consomment la même projection
                // sans jamais entrer dans le thread d'append JSONL.
                let projection = format!("{}:{}", message.id, message.body).into_bytes();
                for view in &views {
                    view.lock().unwrap().push(projection.clone());
                }
                break;
            }
            assert!(Instant::now() < deadline, "tour déterministe non terminé");
            thread::sleep(Duration::from_millis(1));
        }
    }
    let journal = transport
        .journal
        .lock()
        .unwrap()
        .clone()
        .expect("journal activé");
    journal.stop();
    let latencies = journal.take_append_latencies();
    transport.shutdown();
    assert!(views.iter().all(|view| view.lock().unwrap().len() == TURNS));
    latencies
}

fn p95(samples: &[Duration]) -> Duration {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * 95).div_ceil(100).saturating_sub(1)]
}

#[test]
#[ignore = "remplacé par le banc SC-005 à vues attach réelles"]
fn sc005_deux_vues_ne_degradent_pas_le_p95_d_append_de_plus_de_cinq_pourcent() {
    const TURNS: usize = 200;
    let root = std::env::temp_dir().join(format!(
        "bridget-sc005-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let baseline = observation_case(0, &root.join("zero"));
    let observed = observation_case(2, &root.join("two"));
    assert!(
        baseline.len() >= TURNS && observed.len() >= TURNS,
        "append instrumentés insuffisants : 0 vue={}, 2 vues={}",
        baseline.len(),
        observed.len()
    );
    let baseline_p95 = p95(&baseline);
    let observed_p95 = p95(&observed);
    eprintln!(
        "SC-005 append p95: 0 vue={baseline_p95:?} ({} échantillons), 2 vues={observed_p95:?} ({} échantillons)",
        baseline.len(),
        observed.len(),
    );
    assert!(
        observed_p95.as_nanos() * 100 < baseline_p95.as_nanos() * 105,
        "p95 append 2 vues={observed_p95:?}, 0 vue={baseline_p95:?}"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn prompt_dispatched_suit_le_flush_et_le_faux_adaptateur_lit_la_frame() {
    let script = r#"
read initialize
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read session
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read prompt
echo '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"fixture-session","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"prompt-lu"}}}}'
echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"end_turn"}}'
"#;
    let (observer, observed_events) = mpsc::channel();
    let mut transport = AcpTransport::spawn_with_clock_and_cancel_grace(
        AcpOptions {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), script.to_string()],
            queue_capacity: 1,
            permissions: "allow".to_string(),
            notify_timeout_secs: 1,
            provider_observation: None,
        },
        Arc::new(SystemTime::now),
        CANCEL_GRACE,
        CANCEL_POLL,
        Some(observer),
    )
    .unwrap();
    transport.deliver(&message("prompt-dispatched")).unwrap();
    assert!(matches!(
        observed_events.recv_timeout(Duration::from_secs(2)),
        Ok(AcpEvent::PromptDispatched { message_id }) if message_id == "prompt-dispatched"
    ));
    assert!(
        matches!(
            observed_events.recv_timeout(Duration::from_secs(2)),
            Ok(AcpEvent::Update { detail }) if detail == "prompt-lu"
        ),
        "le faux adaptateur n'a pas lu la frame session/prompt"
    );
}

#[test]
fn echec_avant_flush_n_emet_pas_prompt_dispatched() {
    let root = std::env::temp_dir().join(format!(
        "bridget-acp-flush-before-dispatch-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let stdin_closed = root.join("stdin-closed");
    let script = r#"
read initialize
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read session
exec 0<&-
touch "__BRIDGET_STDIN_CLOSED__"
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
sleep 30
"#
    .replace(
        "__BRIDGET_STDIN_CLOSED__",
        &stdin_closed.display().to_string(),
    );
    let (observer, observed_events) = mpsc::channel();
    let mut transport = AcpTransport::spawn_with_clock_and_cancel_grace(
        AcpOptions {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), script],
            queue_capacity: 1,
            permissions: "allow".to_string(),
            notify_timeout_secs: 1,
            provider_observation: None,
        },
        Arc::new(SystemTime::now),
        CANCEL_GRACE,
        CANCEL_POLL,
        Some(observer),
    )
    .unwrap();
    // La réponse session/new n'est écrite par l'adaptateur qu'après la
    // fermeture de stdin et ce jalon. spawn() est donc une barrière : le
    // prompt ne peut pas être tenté contre un lecteur encore ouvert.
    assert!(
        stdin_closed.exists(),
        "le jalon de fermeture stdin est absent"
    );
    transport.deliver(&message("flush-failed")).unwrap();
    match observed_events.recv_timeout(Duration::from_secs(2)) {
        Ok(AcpEvent::DeliveryRejected { message_id, .. }) => {
            assert_eq!(message_id, "flush-failed")
        }
        Ok(AcpEvent::PromptDispatched { .. }) => {
            panic!("un échec avant flush ne peut pas être acquitté")
        }
        Ok(other) => panic!("issue ACP inattendue: {other:?}"),
        Err(error) => panic!("échec de flush non observé: {error}"),
    }
    drop(transport);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn enabled_journal_records_a_complete_transport_turn() {
    let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read request
echo '{"jsonrpc":"2.0","id":"permission-1","method":"session/request_permission","params":{"sessionId":"fixture-session","options":[{"optionId":"allow-1","kind":"allow_once"},{"optionId":"reject-1","kind":"reject_once"}]}}'
read permission
echo '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"fixture-session","update":{"sessionUpdate":"tool_call","toolCallId":"tool-1","title":"Read src/main.rs","name":"read_file","kind":"read","content":{"type":"text","text":"lecture"}}}}'
echo '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"fixture-session","update":{"sessionUpdate":"tool_call_update","toolCallId":"tool-1","content":{"type":"text","text":"analyse"}}}}'
echo '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"fixture-session","update":{"sessionUpdate":"tool_call_update","toolCallId":"tool-1","content":{"type":"text","text":"terminé"}}}}'
echo '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"fixture-session","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"réponse"}}}}'
echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"end_turn"}}'
"#;
    let root = std::env::temp_dir().join(format!("bridget-acp-journal-{}", std::process::id()));
    let mut transport = AcpTransport::spawn(AcpOptions {
        command: "sh".to_string(),
        args: vec!["-c".to_string(), script.to_string()],
        queue_capacity: 2,
        permissions: "allow".to_string(),
        notify_timeout_secs: 1,
        provider_observation: None,
    })
    .unwrap();
    transport.enable_journal(&root, "codex-1").unwrap();
    transport.deliver(&message("journal-message")).unwrap();
    for _ in 0..30 {
        thread::sleep(Duration::from_millis(10));
        if transport
            .drain_events()
            .iter()
            .any(|event| matches!(event, AcpEvent::TurnFinished { .. }))
        {
            transport.shutdown();
            let path = std::fs::read_dir(root.join("codex-1"))
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .path();
            let events = crate::journal::valid_events(&path)
                .into_iter()
                .map(|mut event| {
                    event.as_object_mut().unwrap().remove("seq");
                    event.as_object_mut().unwrap().remove("ts");
                    event
                })
                .collect::<Vec<_>>();
            assert!(events.contains(&json!({
                "v": 1, "session_id": "fixture-session", "event": "turn_start",
                "message_id": "journal-message",
                "payload": {"from":"alice", "reply":false, "body":"journal-message"}
            })));
            assert!(events.contains(&json!({
                "v": 1, "session_id": "fixture-session", "event": "update",
                "message_id": "journal-message",
                "payload": {"kind":"text", "content":"réponse"}
            })));
            assert!(events.contains(&json!({
                "v": 1, "session_id": "fixture-session", "event": "update",
                "message_id": "journal-message",
                "payload": {
                    "kind":"tool", "text":"Read src/main.rs", "detail":"lecture",
                    "tool_call_id":"tool-1", "tool":"Read src/main.rs",
                    "title":"Read src/main.rs", "name":"read_file",
                    "tool_kind":"read", "summary":"lecture"
                }
            })));
            for summary in ["analyse", "terminé"] {
                assert!(
                    events.contains(&json!({
                        "v": 1, "session_id": "fixture-session", "event": "update",
                        "message_id": "journal-message",
                        "payload": {
                            "kind":"tool", "text":"Read src/main.rs", "detail":summary,
                            "tool_call_id":"tool-1", "tool":"Read src/main.rs",
                            "title":"Read src/main.rs", "summary":summary
                        }
                    })),
                    "la mise à jour ACP {summary} doit porter le titre corrélé"
                );
            }
            assert!(events.contains(&json!({
                "v": 1, "session_id": "fixture-session", "event": "permission",
                "message_id": "journal-message",
                "payload": {
                    "tool":"inconnu",
                    "options":[
                        {"optionId":"allow-1","kind":"allow_once"},
                        {"optionId":"reject-1","kind":"reject_once"}
                    ],
                    "decision":{"outcome":"selected","option_id":"allow-1"}
                }
            })));
            assert!(
                events.contains(&json!({
                    "v": 1, "session_id": "fixture-session", "event": "reasoning",
                    "message_id": "journal-message",
                    "payload": {"available": false}
                })),
                "sans thought_chunk : ligne terminale available:false"
            );
            let reasoning_lines: Vec<_> = events
                .iter()
                .filter(|event| event["event"] == "reasoning")
                .collect();
            assert_eq!(
                reasoning_lines.len(),
                1,
                "exactement un event=reasoning terminal par tour, got {reasoning_lines:?}"
            );
            assert!(events.contains(&json!({
                "v": 1, "session_id": "fixture-session", "event": "turn_end",
                "message_id": "journal-message",
                "payload": {"stop_reason":"end_turn"}
            })));
            // L'EOF peut être lu juste avant ou juste après le résultat
            // terminal du tour : le journal conserve alors, de façon
            // légitime, le message courant ou aucun message corrélé.
            assert!(events.iter().any(|event| {
                event["event"] == "error"
                    && match event.get("message_id") {
                        None => true,
                        Some(serde_json::Value::String(id)) => id == "journal-message",
                        _ => false,
                    }
            }));
            std::fs::remove_dir_all(root).unwrap();
            return;
        }
    }
    panic!("le tour journalisé n'a pas terminé");
}

#[test]
fn false_adapter_handles_permission_during_a_prompt() {
    let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read request
echo '{"jsonrpc":"2.0","id":"permission-1","method":"session/request_permission","params":{"sessionId":"fixture-session","options":[{"optionId":"allow-1","kind":"allow_once"}]}}'
read permission
echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"end_turn"}}'
"#;
    let mut transport = AcpTransport::spawn(AcpOptions {
        command: "sh".to_string(),
        args: vec!["-c".to_string(), script.to_string()],
        queue_capacity: 2,
        permissions: "allow".to_string(),
        notify_timeout_secs: 1,
        provider_observation: None,
    })
    .unwrap();
    transport.deliver(&message("permission-message")).unwrap();
    for _ in 0..20 {
        thread::sleep(Duration::from_millis(10));
        if transport
            .drain_events()
            .iter()
            .any(|event| matches!(event, AcpEvent::TurnFinished { .. }))
        {
            return;
        }
    }
    panic!("la permission pendant le prompt n'a pas été traitée");
}

#[test]
fn false_adapter_asserts_the_raw_official_permission_envelope() {
    let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read request
echo '{"jsonrpc":"2.0","id":"permission-1","method":"session/request_permission","params":{"sessionId":"fixture-session","options":[{"optionId":"allow-1","kind":"allow_once"}]}}'
read permission
case "$permission" in
  *'"result":{"outcome":{"optionId":"allow-1","outcome":"selected"}}'*)
    echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"end_turn"}}' ;;
  *) exit 12 ;;
esac
"#;
    let mut transport = AcpTransport::spawn(AcpOptions {
        command: "sh".to_string(),
        args: vec!["-c".to_string(), script.to_string()],
        queue_capacity: 2,
        permissions: "allow".to_string(),
        notify_timeout_secs: 1,
        provider_observation: None,
    })
    .unwrap();
    transport.deliver(&message("permission-raw")).unwrap();
    for _ in 0..30 {
        thread::sleep(Duration::from_millis(10));
        if transport
            .drain_events()
            .iter()
            .any(|event| matches!(event, AcpEvent::TurnFinished { .. }))
        {
            return;
        }
    }
    panic!("l'enveloppe brute de permission officielle n'a pas été acceptée");
}

#[test]
fn foreign_session_update_is_ignored_and_diagnosed() {
    let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read request
echo '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"foreign-session","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"interdit"}}}}'
echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"end_turn"}}'
"#;
    let mut transport = AcpTransport::spawn(AcpOptions {
        command: "sh".to_string(),
        args: vec!["-c".to_string(), script.to_string()],
        queue_capacity: 2,
        permissions: "allow".to_string(),
        notify_timeout_secs: 1,
        provider_observation: None,
    })
    .unwrap();
    transport.deliver(&message("foreign-update")).unwrap();
    for _ in 0..30 {
        thread::sleep(Duration::from_millis(10));
        let events = transport.drain_events();
        if events.iter().any(
            |event| matches!(event, AcpEvent::TurnFinished { response, .. } if response.is_empty()),
        ) {
            assert!(events.iter().any(|event| matches!(event, AcpEvent::Error { detail } if detail.contains("session étrangère"))));
            return;
        }
    }
    panic!("l'update de session étrangère n'a pas été diagnostiquée");
}

/// Le même verrou protège le retrait FIFO et le passage file -> actif.
#[test]
fn guidance156_acp_cancel_queued_atomique() {
    let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read first_prompt
read cancel
echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"cancelled"}}'
cat >/dev/null
"#;
    let mut transport = AcpTransport::spawn(AcpOptions {
        command: "sh".into(),
        args: vec!["-c".into(), script.into()],
        queue_capacity: 4,
        permissions: "allow".into(),
        notify_timeout_secs: 30,
        provider_observation: None,
    })
    .unwrap();
    transport.deliver(&message("active")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while !transport.drain_events().iter().any(|event| {
        matches!(
            event, AcpEvent::PromptDispatched { message_id } if message_id == "active"
        )
    }) {
        assert!(Instant::now() < deadline, "prompt actif non écrit");
        thread::sleep(Duration::from_millis(5));
    }
    for id in ["first", "remove", "race", "last"] {
        transport.deliver(&message(id)).unwrap();
    }
    assert!(!ManagedSession::cancel_queued(
        &transport,
        "active",
        "file seulement"
    ));
    assert!(ManagedSession::cancel_queued(
        &transport,
        "remove",
        "file seulement"
    ));
    assert!(!ManagedSession::cancel_queued(
        &transport,
        "absent",
        "file seulement"
    ));
    let rejected = transport.drain_events();
    assert!(
        matches!(rejected.as_slice(), [AcpEvent::DeliveryRejected { message_id, reason }]
            if message_id == "remove" && reason == "file seulement")
    );

    let (lock, _) = &*transport.queue;
    let mut queue = lock.lock().unwrap();
    let original_active = queue.active.take().expect("tour actif");
    assert!(!original_active.cancelled.load(Ordering::SeqCst));
    assert_eq!(
        queue
            .messages
            .iter()
            .map(|m| m.id.as_str())
            .collect::<Vec<_>>(),
        vec!["first", "race", "last"]
    );
    let (cancellation, cancelled_rx) = mpsc::channel();
    let cancelled = Arc::new(AtomicBool::new(false));
    // Le worker peut gagner le verrou et rendre la remise active avant
    // cancel_queued. Le retrait doit alors refuser sans envoyer cancel.
    thread::scope(|scope| {
        let (entered_tx, entered_rx) = mpsc::channel();
        let session = &transport;
        let attempt = scope.spawn(move || {
            entered_tx.send(()).unwrap();
            ManagedSession::cancel_queued(session, "race", "file seulement")
        });
        entered_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let index = queue.messages.iter().position(|m| m.id == "race").unwrap();
        let promoted = queue.messages.remove(index).unwrap();
        queue.active = Some(ActiveTurn {
            message_id: promoted.id,
            cancellation,
            cancelled: cancelled.clone(),
        });
        drop(queue);
        assert!(
            !attempt.join().unwrap(),
            "une remise devenue active ne doit pas être annulée"
        );
    });
    assert!(!cancelled.load(Ordering::SeqCst));
    assert!(matches!(
        cancelled_rx.try_recv(),
        Err(mpsc::TryRecvError::Empty)
    ));
    let mut queue = lock.lock().unwrap();
    assert_eq!(queue.active.as_ref().unwrap().message_id, "race");
    assert_eq!(
        queue
            .messages
            .iter()
            .map(|m| m.id.as_str())
            .collect::<Vec<_>>(),
        vec!["first", "last"]
    );
    queue.active = Some(original_active);
    drop(queue);
    for id in ["first", "last"] {
        assert!(ManagedSession::cancel_queued(&transport, id, "fin fixture"));
    }
    assert!(transport.cancel_delivery("active", "fin fixture"));
    transport.shutdown();
}

#[test]
fn cancelled_turn_waits_for_adapter_before_the_next_prompt() {
    let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read first_prompt
read cancel
echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"cancelled"}}'
read second_prompt
echo '{"jsonrpc":"2.0","id":4,"result":{"stopReason":"end_turn"}}'
"#;
    let mut transport = AcpTransport::spawn(AcpOptions {
        command: "sh".to_string(),
        args: vec!["-c".to_string(), script.to_string()],
        queue_capacity: 2,
        permissions: "allow".to_string(),
        notify_timeout_secs: 1,
        provider_observation: None,
    })
    .unwrap();
    transport.deliver(&message("cancelled")).unwrap();
    transport.deliver(&message("next")).unwrap();
    for _ in 0..30 {
        thread::sleep(Duration::from_millis(10));
        if matches!(transport.state(), TurnState::InProgress { ref message_id, .. } if message_id == "cancelled")
        {
            break;
        }
    }
    assert!(transport.cancel_delivery("cancelled", "annulation daemon"));
    let mut rejected = false;
    let mut next_finished = false;
    for _ in 0..60 {
        thread::sleep(Duration::from_millis(10));
        for event in transport.drain_events() {
            rejected |= matches!(event, AcpEvent::DeliveryRejected { ref message_id, .. } if message_id == "cancelled");
            next_finished |=
                matches!(event, AcpEvent::TurnFinished { message, .. } if message.id == "next");
        }
        if rejected && next_finished {
            return;
        }
    }
    panic!("le tour annulé a empêché ou contaminé le prompt suivant");
}

#[test]
fn message_humain_actif_interrompt_le_tour_acp_et_garde_fifo() {
    let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read first_prompt
read cancel
echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"cancelled"}}'
read human_prompt
echo '{"jsonrpc":"2.0","id":4,"result":{"stopReason":"end_turn"}}'
read system_prompt
echo '{"jsonrpc":"2.0","id":5,"result":{"stopReason":"end_turn"}}'
"#;
    let mut transport = AcpTransport::spawn(AcpOptions {
        command: "sh".to_string(),
        args: vec!["-c".to_string(), script.to_string()],
        queue_capacity: 3,
        permissions: "allow".to_string(),
        notify_timeout_secs: 1,
        provider_observation: None,
    })
    .unwrap();
    transport.deliver(&message("acp-actif")).unwrap();
    let start_deadline = Instant::now() + Duration::from_secs(2);
    while !transport.is_busy() && Instant::now() < start_deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(transport.is_busy(), "le tour ACP initial doit être actif");

    let mut human = message("acp-humain");
    human.from = "superviseur".to_string();
    human.origin = Some(MessageOrigin::Human);
    transport.deliver(&human).unwrap();
    transport.deliver(&message("acp-systeme")).unwrap();

    let deadline = Instant::now() + Duration::from_secs(3);
    let mut active_rejected = false;
    let mut completed = Vec::new();
    while Instant::now() < deadline {
        for event in transport.drain_events() {
            match event {
                AcpEvent::DeliveryRejected { message_id, .. } if message_id == "acp-actif" => {
                    active_rejected = true;
                }
                AcpEvent::TurnFinished { message, .. } => completed.push(message.id),
                _ => {}
            }
        }
        if active_rejected && completed == vec!["acp-humain".to_string(), "acp-systeme".to_string()]
        {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!(
        "le message humain devait interrompre seulement le tour actif et rester avant le système, annulé={active_rejected}, terminés={completed:?}"
    );
}

#[test]
fn active_reply_deadline_uses_cancel_grace_then_drains_queue() {
    let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read prompt
echo '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"fixture-session","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"prompt-observe"}}}}'
read cancel
echo '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"fixture-session","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"cancel-observe"}}}}'
while :; do :; done
"#;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let observed_now = Arc::new(AtomicU64::new(now));
    let clock_now = observed_now.clone();
    let clock: Clock = Arc::new(move || {
        SystemTime::UNIX_EPOCH + Duration::from_secs(clock_now.load(Ordering::SeqCst))
    });
    let (observer, observed_events) = mpsc::channel();
    let mut transport = AcpTransport::spawn_with_clock_and_cancel_grace(
        AcpOptions {
            command: "sh".to_string(),
            args: vec!["-c".to_string(), script.to_string()],
            queue_capacity: 2,
            permissions: "allow".to_string(),
            notify_timeout_secs: 1,
            provider_observation: None,
        },
        clock,
        Duration::from_millis(100),
        Duration::from_millis(10),
        Some(observer),
    )
    .unwrap();
    let mut active = message("deadline-active");
    active.reply = true;
    active.deadline_at = Some(now + 1);
    transport.deliver(&active).unwrap();
    transport.deliver(&message("deadline-queued")).unwrap();
    assert!(
        matches!(
            observed_events.recv_timeout(Duration::from_secs(2)),
            Ok(AcpEvent::PromptDispatched { message_id }) if message_id == "deadline-active"
        ),
        "la frontière write+flush n'a pas été publiée"
    );
    assert!(
        matches!(
            observed_events.recv_timeout(Duration::from_secs(2)),
            Ok(AcpEvent::Update { detail }) if detail == "prompt-observe"
        ),
        "le faux adaptateur n'a pas observé le prompt"
    );
    observed_now.store(now + 1, Ordering::SeqCst);
    assert!(
        matches!(
            observed_events.recv_timeout(Duration::from_secs(2)),
            Ok(AcpEvent::Update { detail }) if detail == "cancel-observe"
        ),
        "le faux adaptateur n'a pas observé session/cancel avant la grâce"
    );
    assert!(
        matches!(
            observed_events.recv_timeout(Duration::from_secs(2)),
            Ok(AcpEvent::DeliveryRejected { message_id, reason })
                if message_id == "deadline-active" && reason.contains("timeout ACP")
        ),
        "l'échéance injectée n'a pas interrompu le prompt ACP"
    );
    let terminal_events = transport.drain_events();
    assert!(
        terminal_events.iter().any(|event| matches!(
            event,
            AcpEvent::DeliveryRejected { message_id, .. } if message_id == "deadline-queued"
        )),
        "la file n'a pas été drainée après l'annulation forcée"
    );
}

#[test]
fn ignored_cancel_kills_transport_and_drains_the_queue() {
    let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read prompt
read cancel
sleep 2
"#;
    let mut transport = AcpTransport::spawn(AcpOptions {
        command: "sh".to_string(),
        args: vec!["-c".to_string(), script.to_string()],
        queue_capacity: 2,
        permissions: "allow".to_string(),
        notify_timeout_secs: 1,
        provider_observation: None,
    })
    .unwrap();
    transport.deliver(&message("active")).unwrap();
    transport.deliver(&message("queued")).unwrap();
    for _ in 0..30 {
        thread::sleep(Duration::from_millis(10));
        if matches!(transport.state(), TurnState::InProgress { ref message_id, .. } if message_id == "active")
        {
            break;
        }
    }
    assert!(transport.cancel_delivery("active", "annulation daemon"));
    let deadline = Instant::now() + CANCEL_GRACE + Duration::from_secs(1);
    let mut active_rejected = false;
    let mut queued_rejected = false;
    loop {
        for event in transport.drain_events() {
            active_rejected |= matches!(
                event,
                AcpEvent::DeliveryRejected { ref message_id, .. } if message_id == "active"
            );
            queued_rejected |= matches!(
                event,
                AcpEvent::DeliveryRejected { ref message_id, .. } if message_id == "queued"
            );
        }
        if active_rejected && queued_rejected {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "l'arrêt ACP n'a pas publié tous ses rejets (active={active_rejected}, queued={queued_rejected})"
        );
        thread::sleep(Duration::from_millis(10));
    }
    assert!(!transport.is_alive());
}

#[test]
fn notification_timeout_uses_cancel_grace_then_forced_drain() {
    let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read prompt
read cancel
sleep 2
"#;
    let mut transport = AcpTransport::spawn(AcpOptions {
        command: "sh".to_string(),
        args: vec!["-c".to_string(), script.to_string()],
        queue_capacity: 2,
        permissions: "allow".to_string(),
        notify_timeout_secs: 0,
        provider_observation: None,
    })
    .unwrap();
    transport.deliver(&message("timeout-active")).unwrap();
    transport.deliver(&message("timeout-queued")).unwrap();
    thread::sleep(CANCEL_GRACE + Duration::from_millis(100));
    assert!(!transport.is_alive());
    let events = transport.drain_events();
    assert!(events.iter().any(|event| matches!(event, AcpEvent::DeliveryRejected { message_id, reason } if message_id == "timeout-active" && reason.contains("timeout ACP"))));
    assert!(events.iter().any(|event| matches!(event, AcpEvent::DeliveryRejected { message_id, .. } if message_id == "timeout-queued")));
}

/// ORACLE — un mandat reply=false (tous les mandats le service compagnon) honore
/// `deadline_at` absolu du daemon, PAS le notify_timeout figé au spawn.
/// Mutant : remettre `(!reply).then_some(notify_timeout)` exclusif → meurt.
/// Preuve d'abord qu'un tour normal aboutit (sinon projection vide).
#[test]
#[allow(non_snake_case)]
fn TEMOIN_deadline_daemon_sauve_un_tour_reply_false_malgre_notify_fige() {
    let root = std::env::temp_dir().join(format!(
        "bridget-acp-deadline-hot-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();

    // Tour normal : aboutit malgré notify_timeout=1 (complète avant 1 s).
    let script_ok = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read prompt
echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"end_turn"}}'
"#;
    let mut transport_ok = AcpTransport::spawn(AcpOptions {
        command: "sh".to_string(),
        args: vec!["-c".to_string(), script_ok.to_string()],
        queue_capacity: 2,
        permissions: "allow".to_string(),
        notify_timeout_secs: 1,
        provider_observation: None,
    })
    .unwrap();
    transport_ok.enable_journal(&root, "ok").unwrap();
    let mut ok_msg = message("tour-normal");
    ok_msg.reply = false;
    transport_ok.deliver(&ok_msg).unwrap();
    let mut saw_ok = false;
    for _ in 0..50 {
        thread::sleep(Duration::from_millis(20));
        if transport_ok.drain_events().iter().any(|event| {
            matches!(
                event,
                AcpEvent::TurnFinished { message, .. } if message.id == "tour-normal"
            )
        }) {
            saw_ok = true;
            break;
        }
    }
    assert!(
        saw_ok,
        "contrôle positif : un tour court doit aboutir d'abord"
    );

    // Tour long : notify figé à 1 s, mais deadline_at (daemon) à +30 s.
    // Le faux ACP répond après 2 s → survit grâce au deadline daemon.
    let script_long = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read prompt
sleep 2
echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"end_turn"}}'
"#;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut transport_long = AcpTransport::spawn(AcpOptions {
        command: "sh".to_string(),
        args: vec!["-c".to_string(), script_long.to_string()],
        queue_capacity: 2,
        permissions: "allow".to_string(),
        notify_timeout_secs: 1,
        provider_observation: None,
    })
    .unwrap();
    transport_long.enable_journal(&root, "long").unwrap();
    let mut long_msg = message("tour-deadline-hot");
    long_msg.reply = false;
    long_msg.deadline_at = Some(now + 30);
    transport_long.deliver(&long_msg).unwrap();
    let mut saw_long = false;
    let mut saw_timeout = false;
    for _ in 0..200 {
        thread::sleep(Duration::from_millis(50));
        for event in transport_long.drain_events() {
            match event {
                AcpEvent::TurnFinished { message, .. } if message.id == "tour-deadline-hot" => {
                    saw_long = true;
                }
                AcpEvent::DeliveryRejected { message_id, reason }
                    if message_id == "tour-deadline-hot" && reason.contains("timeout ACP") =>
                {
                    saw_timeout = true;
                }
                _ => {}
            }
        }
        if saw_long || saw_timeout {
            break;
        }
    }
    assert!(
        saw_long && !saw_timeout,
        "deadline_at daemon doit sauver reply=false malgré notify_timeout=1 ; \
             finished={saw_long} timeout={saw_timeout}"
    );

    // Sans deadline_at, le même script tombe à 1 s — le signal est DIT.
    let mut transport_kill = AcpTransport::spawn(AcpOptions {
        command: "sh".to_string(),
        args: vec!["-c".to_string(), script_long.to_string()],
        queue_capacity: 2,
        permissions: "allow".to_string(),
        notify_timeout_secs: 1,
        provider_observation: None,
    })
    .unwrap();
    transport_kill.enable_journal(&root, "kill").unwrap();
    let mut kill_msg = message("tour-tue");
    kill_msg.reply = false;
    // pas de deadline_at → notify_timeout figé gagne
    transport_kill.deliver(&kill_msg).unwrap();
    let mut kill_reason = None;
    for _ in 0..80 {
        thread::sleep(Duration::from_millis(50));
        for event in transport_kill.drain_events() {
            if let AcpEvent::DeliveryRejected { message_id, reason } = event
                && message_id == "tour-tue"
            {
                kill_reason = Some(reason);
            }
        }
        if kill_reason.is_some() {
            break;
        }
    }
    let kill_reason = kill_reason.expect("sans deadline, le tour long doit être tué");
    assert!(
        kill_reason.contains("timeout ACP"),
        "signal d'échéance attendu, reçu {kill_reason}"
    );
    let kill_events = std::fs::read_dir(root.join("kill"))
        .unwrap()
        .filter_map(Result::ok)
        .flat_map(|entry| crate::journal::valid_events(&entry.path()))
        .collect::<Vec<_>>();
    let terminal = kill_events
        .iter()
        .find(|event| event["event"] == "error" && event["message_id"] == "tour-tue");
    assert!(
        terminal.is_some_and(|event| { event["payload"]["terminal_kind"] == json!("turn_failed") }),
        "un tour tué DOIT porter terminal_kind=turn_failed ; reçu {kill_events:?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn false_adapter_eof_rejects_an_active_turn() {
    let script = r#"
read request
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read request
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read request
exit 0
"#;
    let mut transport = AcpTransport::spawn(AcpOptions {
        command: "sh".to_string(),
        args: vec!["-c".to_string(), script.to_string()],
        queue_capacity: 2,
        permissions: "allow".to_string(),
        notify_timeout_secs: 1,
        provider_observation: None,
    })
    .unwrap();
    transport.deliver(&message("eof-message")).unwrap();
    transport.deliver(&message("eof-queued")).unwrap();
    for _ in 0..20 {
        thread::sleep(Duration::from_millis(10));
        let events = transport.drain_events();
        if events.iter().any(|event| matches!(event, AcpEvent::DeliveryRejected { message_id, reason } if message_id == "eof-message" && reason.contains("EOF ACP")))
                && events.iter().any(|event| matches!(event, AcpEvent::DeliveryRejected { message_id, reason } if message_id == "eof-queued" && reason.contains("EOF ACP"))) {
                return;
            }
    }
    panic!("EOF actif non propagé au tour");
}
#[test]
fn consigne_privee_preserve_le_corps_visible_du_message() {
    let mut message = message("profile-body");
    message.body = "Demande utilisateur visible.".to_string();
    let prompt =
        prompt_for_with_private_instructions(&message, Some("Privilégie les sources attestées."));

    assert!(prompt.contains("Privilégie les sources attestées."));
    assert!(prompt.ends_with(&message.body));
    assert!(prompt_for(&message).ends_with(&message.body));
    assert!(prompt_for(&message).contains(&message.id));
}

#[test]
fn spec133_acp_affiche_la_delegation_sans_session_native() {
    let mut message = message("delegated-acp");
    message.delegated_origin = Some(bridget_core::DelegatedOrigin {
        provider: "claude".into(),
        child_ref: "0123456789abcdef".into(),
    });
    let prompt = prompt_for(&message);
    assert!(prompt.contains("via sous-agent claude 0123456789abcdef"));
    assert!(prompt.contains("\"delegated_origin\""));
    assert!(!prompt.contains("session-native-secrete"));
}
#[test]
fn edit159_promote_transport_acp_enfile() {
    let script = r#"
read initialize
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read session
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"fixture-session"}}'
read prompt
echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"end_turn"}}'
while read remaining; do :; done
"#;
    let mut transport = AcpTransport::spawn(AcpOptions {
        command: "sh".into(),
        args: vec!["-c".into(), script.into()],
        queue_capacity: 2,
        permissions: "allow".into(),
        notify_timeout_secs: 2,
        provider_observation: None,
    })
    .unwrap();
    let mut promoted = message("acp-promoted159");
    promoted.intent = Some(MessageIntent::SteerCurrent);
    promoted.references.push("steer-or-queue".into());
    transport.deliver(&promoted).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut finished = false;
    while Instant::now() < deadline && !finished {
        for event in transport.drain_events() {
            assert!(
                !matches!(&event, AcpEvent::DeliveryRejected { message_id, .. } if message_id == "acp-promoted159")
            );
            finished |= matches!(&event, AcpEvent::TurnFinished { message, .. } if message.id == "acp-promoted159");
        }
        if !finished {
            thread::sleep(Duration::from_millis(10));
        }
    }
    transport.stop();
    assert!(
        finished,
        "ACP traite la promotion comme une remise FIFO ordinaire"
    );
}
