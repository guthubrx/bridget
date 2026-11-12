use super::*;
use crate::t3code_contract::{LatestTurn, Message};
use bridget_transport::poison::RecoverPoison;

#[test]
fn spec226_slow_delivery_does_not_hold_the_snapshot_mutex() {
    let (mut worker, _peer) = worker099();
    let session = worker.session.clone();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Mutex::new(release_rx);
    let delivery = thread::spawn(move || {
        session.call_detached(|_| {
            entered_tx.send(()).unwrap();
            release_rx.lock().unwrap().recv().unwrap();
            Ok(())
        })
    });
    entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let session = worker.session.clone();
    let (read_tx, read_rx) = mpsc::channel();
    let reader = thread::spawn(move || read_tx.send(session.call(|_| Ok(()))).unwrap());
    let read = read_rx.recv_timeout(Duration::from_secs(2));
    release_tx.send(()).unwrap();
    assert_eq!(delivery.join().unwrap(), Ok(()));
    reader.join().unwrap();
    assert_eq!(read.unwrap(), Ok(()));
    worker.journal.stop();
    worker.relay.shutdown();
}

#[test]
fn spec226_detached_transport_never_retries_and_runtime_change_is_unknown() {
    let (mut worker, _peer) = worker099();
    let calls = std::cell::Cell::new(0);
    let result = worker.session.call_detached::<()>(|_| {
        calls.set(calls.get() + 1);
        Err(ContractError::Transport("rpc_response_timeout".into()))
    });
    assert_eq!(
        result,
        Err(ContractError::Transport("rpc_response_timeout".into()))
    );
    assert_eq!(calls.get(), 1);
    let result = worker.session.call_detached(|_| {
        worker.session.reset_runtime(ServerRuntime {
            pid: std::process::id(),
            port: 2,
        });
        Ok(())
    });
    assert_eq!(
        result,
        Err(ContractError::Transport(
            "rpc_runtime_changed_effect_unknown".into()
        ))
    );
    worker.journal.stop();
    worker.relay.shutdown();
}

#[test]
fn spec226_timeout_grace_requires_same_server_and_recent_success() {
    let now = Instant::now();
    let runtime = ServerRuntime {
        pid: 42,
        port: 4321,
    };
    let previous = Some((now, runtime.clone(), 123));
    let current = Some((runtime.clone(), 123));
    assert!(retain_links_after_timeout(
        "http_timeout",
        previous.clone(),
        current.clone(),
        now
    ));
    assert!(retain_links_after_timeout(
        "rpc_response_timeout",
        previous.clone(),
        current.clone(),
        now
    ));
    assert!(!retain_links_after_timeout(
        "http_response_interrupted",
        previous.clone(),
        current.clone(),
        now
    ));
    assert!(!retain_links_after_timeout(
        "http_timeout",
        None,
        current.clone(),
        now
    ));
    assert!(!retain_links_after_timeout(
        "http_timeout",
        previous.clone(),
        None,
        now
    ));
    assert!(!retain_links_after_timeout(
        "http_timeout",
        previous.clone(),
        Some((runtime, 124)),
        now
    ));
    assert!(!retain_links_after_timeout(
        "http_timeout",
        previous.clone(),
        Some((
            ServerRuntime {
                pid: 43,
                port: 4321
            },
            123
        )),
        now
    ));
    assert!(!retain_links_after_timeout(
        "http_timeout",
        previous,
        current,
        now + SNAPSHOT_TIMEOUT_GRACE
    ));
}

#[test]
fn spec226_heartbeat_only_during_bounded_snapshot_grace() {
    let (mut worker, peer) = worker099();
    peer.set_nonblocking(true).unwrap();
    let mut reader = BufReader::new(peer);
    let drain = |reader: &mut BufReader<UnixStream>| {
        let mut frames = Vec::new();
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => frames.push(serde_json::from_str::<WrapperToDaemon>(&line).unwrap()),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => panic!("lecture du lien : {e}"),
            }
        }
        frames
    };
    let _initial = drain(&mut reader);
    worker.last_key = "stale".into();
    worker.observation_events = vec![bridget_transport::protocol::ObservationKind::TurnEnded];
    worker.handle_event(LinkEvent::Unavailable);
    assert!(worker.last_key.is_empty());
    assert!(worker.observation_events.is_empty());
    assert!(
        !drain(&mut reader)
            .iter()
            .any(|frame| matches!(frame, WrapperToDaemon::Heartbeat))
    );
    worker.last_key = "stale".into();
    worker.handle_event(LinkEvent::SnapshotTimeout);
    assert!(worker.last_key.is_empty());
    assert!(worker.daemon_alive.load(Ordering::SeqCst));
    assert!(
        drain(&mut reader)
            .iter()
            .any(|frame| matches!(frame, WrapperToDaemon::Heartbeat))
    );
    worker.journal.stop();
    worker.relay.shutdown();
}

fn replies105(peer: UnixStream) -> Vec<bridget_core::BridgetMessage> {
    peer.set_nonblocking(true).unwrap();
    let mut reader = BufReader::new(peer);
    let mut replies = Vec::new();
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                if let WrapperToDaemon::Send(message) = serde_json::from_str(&line).unwrap() {
                    replies.push(message);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) => panic!("lecture des trames : {e}"),
        }
    }
    replies
}

#[test]
fn spec140_titre_json_borne_et_entete_historique_inchange() {
    let mut message = bridget_core::BridgetMessage::new("bridget", "bob", "corps intact");
    message.id = "notice-140".into();
    message.thread_notice = Some(bridget_core::ThreadNotice {
        version: 1,
        thread_id: "fil-140".into(),
        through_seq: 2,
        generation: 1,
    });
    let historical = envelope(&message);
    let mut wire = serde_json::to_value(&message).unwrap();
    wire["thread_display_title"] =
        serde_json::json!("  Projet\n\t\"politique\" \\ suite\u{202e}\u{0007} ");
    let enriched: bridget_core::BridgetMessage = serde_json::from_value(wire.clone()).unwrap();
    let actual = envelope(&enriched);
    let heading = historical.split_once("\n\n").unwrap().0;
    assert!(actual.starts_with(&format!("{heading}\n\nTitre du fil Bridget : \"Projet \\\"politique\\\" \\\\ suite\"\n\ncorps intact")), "{actual}");
    wire["thread_display_title"] = serde_json::json!("界".repeat(500));
    let bounded = envelope(&serde_json::from_value(wire.clone()).unwrap());
    let title_json = bounded
        .split_once("Titre du fil Bridget : ")
        .unwrap()
        .1
        .split_once('\n')
        .unwrap()
        .0;
    assert_eq!(
        serde_json::from_str::<String>(title_json)
            .unwrap()
            .chars()
            .count(),
        200
    );
    wire["thread_display_title"] = serde_json::json!(" \n\u{202e}\u{0007}");
    assert_eq!(envelope(&serde_json::from_value(wire).unwrap()), historical);
    assert_eq!(message.body, "corps intact");
}

/// Message ordinaire sans réponse attendue : le cas groupable.
fn plain114(id: &str) -> bridget_core::BridgetMessage {
    let mut message = bridget_core::BridgetMessage::new("alice", "bob", format!("corps {id}"));
    message.id = id.into();
    message
}

#[test]
fn native151_groupe_remises_natives_message() {
    let (mut worker, _peer) = worker099();
    for index in 0..3 {
        let message = plain114(&format!("native-result-task-{index}"));
        assert!(!message.reply);
        assert!(message.thread_notice.is_none());
        assert_eq!(batch_family(&message), Some(BatchFamily::Message));
        worker.handle_frame(DaemonToWrapper::Deliver {
            message,
            mid_turn_ok: None,
        });
    }
    assert_eq!(worker.batchable_prefix(), 3);
    worker.journal.stop();
    worker.relay.shutdown();
}

/// Session 114 : plusieurs comptes rendus sans réponse attendue tiennent
/// dans un seul tour, au lieu d'en ouvrir un chacun.
#[test]
fn spec114_lot_sans_reponse_attendue_partage_un_seul_tour() {
    let (mut worker, _peer) = worker099();
    let messages: Vec<_> = (0..3)
        .map(|index| plain114(&format!("r-{index}")))
        .collect();
    for message in &messages {
        worker.handle_frame(DaemonToWrapper::Deliver {
            message: message.clone(),
            mid_turn_ok: None,
        });
    }
    assert_eq!(worker.batchable_prefix(), 3, "les trois partagent un tour");
    let server = http_once099(&worker, 200, r#"{"sequence":1}"#.into());
    let result = worker.dispatch_batch(&messages, &summary(None), None);
    let request = server.join().unwrap();
    worker.journal.stop();
    worker.relay.shutdown();
    assert!(result.is_ok(), "{result:?}");
    for message in &messages {
        assert!(
            request.contains(&message.body),
            "le tour unique porte {}",
            message.id
        );
    }
    assert!(
        request.contains("3 messages Bridget"),
        "le destinataire sait qu'il lit un lot"
    );
}

#[test]
fn spec134_lot_associe_chaque_nom_au_bon_uuid() {
    let first_id = "87515984-f549-49ae-ab9c-a6c47dada0e5";
    let second_id = "cbd8d228-62ef-494b-80b7-bf08333591d6";
    let mut first = bridget_core::BridgetMessage::new(first_id, "bob", "premier");
    first.id = "m-1".into();
    first.from_display_name = Some("cx-coordinator".into());
    let mut second = bridget_core::BridgetMessage::new(second_id, "bob", "second");
    second.id = "m-2".into();
    second.from_display_name = Some("Regional".into());

    let text = batch_envelope(&[first, second]);

    assert!(
        text.contains(&format!("de cx-coordinator ({first_id}) (id m-1)")),
        "{text}"
    );
    assert!(
        text.contains(&format!("de Regional ({second_id}) (id m-2)")),
        "{text}"
    );
}

/// Un message qui porte sa consigne propre, ou dont la réponse est
/// attendue, ne peut pas partager le tour d'un autre.
#[test]
fn spec131_reponse_relayee_signalee_a_l_utilisateur_du_fil() {
    let mut msg = bridget_core::BridgetMessage::new("agent-a", "coordinateur", "Tu valides ?");
    msg.reply = true;
    let text = envelope(&msg);
    assert!(text.contains("transmettra ta réponse finale à agent-a"));
    assert!(
        text.contains("« ↪ Réponse à agent-a (relayée par Bridget) : »"),
        "{text}"
    );
}

#[test]
fn spec133_enveloppe_affiche_le_parent_et_la_reference_opaque() {
    let mut msg = bridget_core::BridgetMessage::new("parent", "destinataire", "mission");
    msg.delegated_origin = Some(bridget_core::DelegatedOrigin {
        provider: "codex".into(),
        child_ref: "0123456789abcdef".into(),
    });
    let text = envelope(&msg);
    assert!(text.contains("parent (via sous-agent codex 0123456789abcdef)"));
    assert!(!text.contains("session-native-secrete"));
}

#[test]
fn spec130_notifications_successives_partagent_un_tour_sans_messages() {
    let (mut worker, _peer) = worker099();
    for id in ["bridget-observation:1", "bridget-observation:2"] {
        worker.handle_frame(DaemonToWrapper::Deliver {
            message: plain114(id),
            mid_turn_ok: None,
        });
    }
    worker.handle_frame(DaemonToWrapper::Deliver {
        message: plain114("message"),
        mid_turn_ok: None,
    });
    assert_eq!(
        worker.batchable_prefix(),
        2,
        "les deux notifications, pas le message"
    );
    let notifications: Vec<_> = ["bridget-observation:1", "bridget-observation:2"]
        .into_iter()
        .map(plain114)
        .collect();
    let text = batch_envelope(&notifications);
    assert!(
        text.starts_with("🔔 2 notifications Bridget groupées"),
        "{text}"
    );
    assert!(text.contains("Pas d'accusé de réception"));
    worker.journal.stop();
    worker.relay.shutdown();
}

#[test]
fn spec130_seul_un_message_sans_reponse_pilote_un_tour_en_cours() {
    let mut question = plain114("question");
    question.reply = true;
    let observation = plain114("bridget-observation:1");
    assert!(groupable(&plain114("compte-rendu")));
    assert!(
        !groupable(&question),
        "demande suivie : attente de la fin du tour"
    );
    assert!(
        !groupable(&observation),
        "notification : attente de la fin du tour"
    );
}

#[test]
fn spec114_message_a_consigne_propre_n_est_jamais_groupe() {
    let mut question = plain114("question");
    question.reply = true;
    let mut notice = plain114("notice");
    notice.thread_notice = Some(bridget_core::ThreadNotice {
        version: 1,
        thread_id: "33333333-3333-4333-8333-333333333333".into(),
        through_seq: 3,
        generation: 1,
    });
    let observation = plain114("bridget-observation:1");
    for special in [question, notice, observation] {
        let (mut worker, _peer) = worker099();
        worker.handle_frame(DaemonToWrapper::Deliver {
            message: special.clone(),
            mid_turn_ok: None,
        });
        worker.handle_frame(DaemonToWrapper::Deliver {
            message: plain114("suivant"),
            mid_turn_ok: None,
        });
        assert_eq!(worker.batchable_prefix(), 1, "{} en tête", special.id);
        worker.journal.stop();
        worker.relay.shutdown();

        let (mut worker, _peer) = worker099();
        worker.handle_frame(DaemonToWrapper::Deliver {
            message: plain114("premier"),
            mid_turn_ok: None,
        });
        worker.handle_frame(DaemonToWrapper::Deliver {
            message: special.clone(),
            mid_turn_ok: None,
        });
        assert_eq!(worker.batchable_prefix(), 1, "{} en second", special.id);
        worker.journal.stop();
        worker.relay.shutdown();
    }
}

/// Le lot reste lisible : borné en nombre et en taille.
#[test]
fn spec114_lot_borne_en_nombre_et_en_taille() {
    let (mut worker, _peer) = worker099();
    for index in 0..(BATCH_BOUND + 3) {
        worker.handle_frame(DaemonToWrapper::Deliver {
            message: plain114(&format!("m-{index}")),
            mid_turn_ok: None,
        });
    }
    assert_eq!(worker.batchable_prefix(), BATCH_BOUND, "borne de nombre");
    worker.journal.stop();
    worker.relay.shutdown();

    let (mut worker, _peer) = worker099();
    let mut enorme = plain114("enorme");
    enorme.body = "x".repeat(BATCH_BODY_BOUND);
    worker.handle_frame(DaemonToWrapper::Deliver {
        message: enorme,
        mid_turn_ok: None,
    });
    worker.handle_frame(DaemonToWrapper::Deliver {
        message: plain114("suivant"),
        mid_turn_ok: None,
    });
    assert_eq!(worker.batchable_prefix(), 1, "borne de taille");
    worker.journal.stop();
    worker.relay.shutdown();
}

/// Un message seul garde mot pour mot son enveloppe historique.
#[test]
fn spec114_enveloppe_d_un_message_seul_est_inchangee() {
    for message in [plain114("seul"), {
        let mut question = plain114("question");
        question.reply = true;
        question
    }] {
        assert_eq!(
            batch_envelope(std::slice::from_ref(&message)),
            envelope(&message)
        );
    }
}

#[test]
fn mobile177_human_single_and_batched_instructions_reply_here() {
    let human =
        bridget_core::BridgetMessage::new("human", "agent177", "Coucou depuis le téléphone");
    assert!(envelope(&human).contains(bridget_core::envelope::HUMAN_REPLY_NOTICE));
    let other = bridget_core::BridgetMessage::new("agent178", "agent177", "Faits utiles");
    for messages in [vec![human.clone(), human.clone()], vec![human, other]] {
        let rendered = batch_envelope(&messages);
        assert!(rendered.contains(bridget_core::envelope::HUMAN_REPLY_NOTICE));
        assert!(!rendered.contains("Le pont ne relaie pas ta réponse finale"));
    }
}

#[test]
fn spec253_lot_sans_reponse_conserve_le_parcours_de_mission() {
    let text = batch_envelope(&[plain114("une"), plain114("deux")]);
    assert!(text.contains(bridget_core::envelope::NO_REPLY_NOTICE));
    assert!(!text.contains("Le pont ne relaie pas ta réponse finale pour ce tour"));
}

#[test]
fn spec105_notification_et_reponse_ne_demandent_pas_de_relais() {
    for parent in [None, Some("question")] {
        let mut msg = bridget_core::BridgetMessage::new("alice", "bob", "Bien reçu");
        msg.in_reply_to = parent.map(str::to_string);
        assert!(!envelope(&msg).contains("transmettra ta réponse"));
        assert!(envelope(&msg).contains("Pas d'accusé de réception"));
        msg.reply = true;
        assert!(envelope(&msg).contains("transmettra ta réponse"));
    }
}

#[test]
fn spec102_v28_alerte_de_fil_injectee_sans_attente_ni_relais() {
    let (mut worker, peer) = worker099();
    let mut msg = bridget_core::BridgetMessage::new(
        "bridget",
        &worker.agent_id,
        "Sollicitation dans le fil t, nouveautés jusqu'à 3.",
    );
    msg.id = "thread-notice:t:agent:1".into();
    msg.origin = Some(bridget_core::MessageOrigin::System);
    msg.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    msg.thread_notice = Some(bridget_core::ThreadNotice {
        version: 1,
        thread_id: "33333333-3333-4333-8333-333333333333".into(),
        through_seq: 3,
        generation: 1,
    });
    let text = envelope(&msg);
    assert!(
        text.contains("bridget_thread") && text.contains("33333333-3333-4333-8333-333333333333")
    );
    assert!(text.contains("jusqu'à 3"));
    assert!(
        !text.contains("transmettra ta réponse"),
        "aucun relais de réponse finale"
    );
    assert!(
        !text.starts_with("🔔"),
        "une sollicitation n'est pas une observation"
    );
    let server = http_once099(&worker, 200, r#"{"sequence":1}"#.into());
    let result = worker.dispatch_with_id(&msg, &summary(None));
    let request = server.join().unwrap();
    worker.journal.stop();
    worker.relay.shutdown();
    assert!(result.is_ok(), "{result:?}");
    assert!(
        request.contains("bridget_thread"),
        "l'enveloppe typée est injectée"
    );
    assert!(
        worker.state.pending.is_empty(),
        "aucune attente de message direct"
    );
    assert!(ThreadState::load(&worker.state_path).pending.is_empty());
    assert!(replies105(peer).is_empty(), "aucune réponse automatique");
}

#[test]
fn spec105_dispatch_sans_demande_ne_cree_aucune_attente() {
    for parent in [None, Some("question")] {
        let (mut worker, _peer) = worker099();
        let mut msg = bridget_core::BridgetMessage::new("alice", &worker.agent_id, ".");
        msg.in_reply_to = parent.map(str::to_string);
        let server = http_once099(&worker, 200, r#"{"sequence":1}"#.into());
        let result = worker.dispatch_with_id(&msg, &summary(None));
        let request = server.join().unwrap();
        worker.journal.stop();
        worker.relay.shutdown();
        assert!(request.starts_with("POST /api/orchestration/dispatch"));
        assert!(result.is_ok(), "{result:?}");
        assert!(worker.state.pending.is_empty());
        assert!(ThreadState::load(&worker.state_path).pending.is_empty());
    }
}

#[test]
fn spec105_ancienne_reponse_prete_ne_part_pas_sans_preuve() {
    let (mut worker, peer) = worker099();
    worker.state = serde_json::from_value(serde_json::json!({"pending":[{
        "request_id":"legacy", "from":"alice", "message_id":"u",
        "anchor_turn_id":null, "dispatched_at":"", "response":"Noté."
    }]}))
    .unwrap();
    worker.state.save(&worker.state_path).unwrap();
    worker.state = ThreadState::load(&worker.state_path);
    worker.send_ready_responses();
    worker.journal.stop();
    worker.relay.shutdown();
    assert!(worker.response_sent.is_empty());
    assert!(replies105(peer).is_empty());
    assert_eq!(worker.state.pending.len(), 1, "préserver sans relayer");
}

#[test]
fn spec105_reprise_exige_demande_ouverte_et_identites_exactes() {
    for case in [
        "absente",
        "expediteur",
        "cible",
        "expiree",
        "annulee",
        "ouverte",
    ] {
        let (mut worker, peer) = worker099();
        let mut old = pending("u", None);
        old.reply_requested = None;
        old.response = Some("OK".into());
        worker.state.pending.push(old);
        let mut request = RequestInfo {
            id: "req".into(),
            sender: "alice".into(),
            target: worker.agent_id.clone(),
            state: "open".into(),
            created_at: 0,
            deadline_at: i64::MAX,
            cancel_reason: None,
            deferred_reminder_level: None,
            deferred_reminder_at: None,
        };
        match case {
            "expediteur" => request.sender = "autre".into(),
            "cible" => request.target = "autre".into(),
            "expiree" => request.deadline_at = 0,
            "annulee" => request.state = "cancelled".into(),
            _ => {}
        }
        worker.handle_frame(DaemonToWrapper::RequestList {
            requests: if case == "absente" {
                vec![]
            } else {
                vec![request]
            },
        });
        worker.state = ThreadState::load(&worker.state_path);
        worker.send_ready_responses();
        worker.send_ready_responses();
        worker.journal.stop();
        worker.relay.shutdown();
        assert_eq!(
            worker.response_sent.len(),
            usize::from(case == "ouverte"),
            "{case}"
        );
        let replies = replies105(peer);
        assert_eq!(replies.len(), usize::from(case == "ouverte"), "{case}");
        if case == "ouverte" {
            let r = &replies[0];
            assert!(
                !r.reply
                    && r.body == "OK"
                    && r.to == "alice"
                    && r.in_reply_to.as_deref() == Some("req")
            );
        } else if case == "annulee" {
            assert!(worker.state.pending.is_empty());
        } else {
            assert_eq!(worker.state.pending[0].reply_requested, None);
        }
    }
}

#[test]
fn spec105_question_suivie_relayee_une_fois_puis_arret() {
    let (mut worker, peer) = worker099();
    let mut question = bridget_core::BridgetMessage::new("alice", &worker.agent_id, "Valide ?");
    question.reply = true;
    question.in_reply_to = Some("question-precedente".into());
    let server = http_once099(&worker, 200, r#"{"sequence":1}"#.into());
    worker.dispatch_with_id(&question, &summary(None)).unwrap();
    server.join().unwrap();
    worker.state = ThreadState::load(&worker.state_path);
    let detail = ThreadDetail {
        activities: vec![],
        activities_available: false,
        latest_turn: None,
        messages: vec![
            message(
                &worker.state.pending[0].message_id,
                "user",
                "Valide ?",
                None,
                false,
            ),
            message("answer", "assistant", "OK", Some("t"), false),
        ],
    };
    worker.settle_pending(&detail, &summary(Some(("t", "completed"))));
    let replies = replies105(peer);
    assert_eq!(replies.len(), 1);
    let response = &replies[0];
    assert_eq!(response.in_reply_to.as_deref(), Some(question.id.as_str()));
    assert!(!response.reply);
    assert_eq!(response.body, "OK");
    let (mut recipient, _peer) = worker099();
    let server = http_once099(&recipient, 200, r#"{"sequence":1}"#.into());
    recipient
        .dispatch_with_id(response, &summary(None))
        .unwrap();
    server.join().unwrap();
    assert!(
        recipient.state.pending.is_empty(),
        "pas de réponse à la réponse"
    );
    for w in [&mut worker, &mut recipient] {
        w.journal.stop();
        w.relay.shutdown();
    }
}

/// Session 117 : un blocage du journal n'est signalé qu'une fois, après le
/// délai de grâce, et un nouveau motif recommence le compte.
#[test]
fn spec117_blocage_du_journal_signale_une_fois_apres_le_delai() {
    let (mut worker, _peer) = worker099();
    worker.note_journal_block("journal non rattrapé", || "détail".into());
    assert!(
        matches!(
            worker.journal_block,
            Some(("journal non rattrapé", _, false))
        ),
        "un blocage bref n'est pas signalé"
    );
    if let Some((_, since, _)) = worker.journal_block.as_mut() {
        *since = Instant::now() - JOURNAL_BLOCK_GRACE;
    }
    worker.note_journal_block("journal non rattrapé", || "détail".into());
    assert!(
        matches!(worker.journal_block, Some((_, _, true))),
        "signalé"
    );
    worker.note_journal_block("journal non rattrapé", || panic!("signalé deux fois"));
    worker.note_journal_block("activité refusée par le journal", || "x".into());
    assert!(
        matches!(
            worker.journal_block,
            Some(("activité refusée par le journal", _, false))
        ),
        "un nouveau motif repart de zéro"
    );
    worker.clear_journal_block();
    assert!(worker.journal_block.is_none());
    worker.journal.stop();
    worker.relay.shutdown();
}

/// Session 116 : un tour prouvé silencieux n'est annoncé qu'après plusieurs
/// lectures concordantes. Si son texte apparaît entre-temps, projection
/// T3 en retard, c'est lui qui part, pas l'annonce de silence.
#[test]
fn spec116_silence_engage_apres_lectures_concordantes_seulement() {
    let (mut worker, _peer) = worker099();
    worker.state.pending.push(pending("u1", None));
    worker
        .state
        .turn_origins
        .insert("t1".to_string(), "u1".to_string());
    let muet = detail116(
        vec![message("u1", "user", "demande", None, false)],
        ("t1", "completed"),
    );
    let s = summary(Some(("t1", "completed")));
    for _ in 1..SETTLE_ATTEMPTS {
        worker.settle_pending(&muet, &s);
        assert_eq!(worker.state.pending[0].response, None, "pas encore engagé");
    }
    let tardif = detail116(
        vec![
            message("u1", "user", "demande", None, false),
            message(
                "a1",
                "assistant",
                "réponse projetée tard",
                Some("t1"),
                false,
            ),
        ],
        ("t1", "completed"),
    );
    worker.settle_pending(&tardif, &s);
    assert_eq!(
        worker.state.pending[0].response.as_deref(),
        Some("réponse projetée tard"),
        "le texte arrivé tard l'emporte sur l'annonce de silence"
    );

    let (mut worker, _peer) = worker099();
    worker.state.pending.push(pending("u1", None));
    worker
        .state
        .turn_origins
        .insert("t1".to_string(), "u1".to_string());
    for _ in 0..SETTLE_ATTEMPTS {
        worker.settle_pending(&muet, &s);
    }
    assert_eq!(
        worker.state.pending[0].response.as_deref(),
        Some(SILENT_TURN_NOTICE),
        "un silence confirmé est annoncé à l'expéditeur"
    );
    worker.journal.stop();
    worker.relay.shutdown();
}

#[test]
fn spec105_attente_non_attestee_ne_correlle_ni_ne_force_la_lecture() {
    for flag in [None, Some(false)] {
        let (mut worker, _peer) = worker099();
        let mut old = pending("u", Some("ancre-ancienne"));
        old.reply_requested = flag;
        worker.state.pending.push(old);
        let detail = ThreadDetail {
            activities: vec![],
            activities_available: false,
            latest_turn: None,
            messages: vec![],
        };
        worker.settle_pending(&detail, &summary(None));
        assert_eq!(worker.state.pending[0].attempts, 0);
        assert!(!worker.state.pending[0].needs_correlation());
        let summary = summary(None);
        worker.last_key = summary.change_key();
        let server = http_once099(&worker, 200, r#"{"thread":{"messages":[]}}"#.into());
        worker.handle_event(LinkEvent::Tick(Box::new(summary)));
        assert!(
            server.join().unwrap().is_empty(),
            "pas de HTTP pour une attente sans preuve"
        );
        worker.journal.stop();
        worker.relay.shutdown();
    }
}

#[test]
fn spec101_t3_terminal_state_is_closed() {
    assert!(!turn_is_final("future_unknown_state"));
    assert!(turn_is_final("completed"));
    assert!(turn_is_final("error"));
    assert!(turn_is_final("interrupted"));
}

#[test]
fn spec101_t3_observation_notification_ne_demande_pas_de_reponse() {
    let mut msg = bridget_core::BridgetMessage::new("bridget", "target", "fait observé");
    msg.id = "bridget-observation:test".into();
    msg.origin = Some(bridget_core::MessageOrigin::System);
    let text = envelope(&msg);
    assert!(!text.contains("transmettra ta réponse"));
    assert!(text.contains("Pas d'accusé de réception"));
}

#[test]
fn spec101_t3_fin_sans_texte_et_origine_non_ambigue() {
    let detail = contract::parse_thread_detail(r#"{"thread":{
            "messages":[{"id":"u","role":"user","text":"travaille","streaming":false,"turnId":null,"createdAt":"2026-09-16T12:00:00Z"}],
            "latestTurn":{"turnId":"t","state":"interrupted","requestedAt":"2026-09-16T12:00:00Z"},"activities":[]}}"#).unwrap();
    let latest = detail.latest_turn.as_ref().unwrap();
    assert_eq!(observation_origin(&detail, latest), Some("u"));
    let mut ambiguous = detail.clone();
    ambiguous.messages.push(ambiguous.messages[0].clone());
    assert_eq!(observation_origin(&ambiguous, latest), None);
}

#[test]
fn spec101_t3_lectures_et_echecs_ne_sont_pas_ecritures() {
    let mut activity = contract::Activity {
        id: "a".into(),
        kind: "tool.completed".into(),
        turn_id: Some("t".into()),
        payload: serde_json::json!({"itemType":"file_change","status":"completed",
                "data":{"toolName":"ReadFile","files":[{"path":"/tmp/fichier"}]}}),
    };
    assert!(activity_payloads(&activity, "claude", Path::new("/tmp")).is_empty());
    activity.payload["data"]["toolName"] = serde_json::json!("Write");
    assert!(activity_payloads(&activity, "claude", Path::new("/tmp")).is_empty());
    assert_eq!(
        activity_payloads(&activity, "codex", Path::new("/tmp")).len(),
        1
    );
    activity.payload["status"] = serde_json::json!("failed");
    assert!(activity_payloads(&activity, "claude", Path::new("/tmp")).is_empty());
    activity.kind = "approval.requested".into();
    assert_eq!(
        activity_payloads(&activity, "claude", Path::new("/tmp"))[0]["kind"],
        "approval"
    );
}

#[test]
fn spec101_t3_journal_confirme_fin_permission_et_fichiers_sans_doublon() {
    let (mut worker, _peer) = worker099();
    worker.state.seeded = true;
    worker.state.observations_seeded = true;
    let detail = contract::parse_thread_detail(r#"{"thread":{
            "messages":[{"id":"u","role":"user","streaming":false,"createdAt":"date"}],
            "latestTurn":{"turnId":"t","state":"completed","requestedAt":"date"},
            "activities":[
              {"id":"permission","kind":"approval.requested","turnId":"t","payload":{}},
              {"id":"write","kind":"tool.completed","turnId":"t","payload":{"itemType":"file_change","status":"completed","data":{"files":[{"path":"/tmp/a"},{"path":"/tmp/b"}]}}}
            ]}}"#).unwrap();
    let mut summary = summary(Some(("t", "completed")));
    summary.provider_instance_id = Some("codex".into());
    worker.project_journal(&detail, &summary);
    worker.journal.stop();
    let entries = std::fs::read_dir(&worker.journal_dir)
        .unwrap()
        .flat_map(|path| bridget_transport::journal::valid_events(&path.unwrap().path()))
        .collect::<Vec<_>>();
    assert_eq!(
        entries
            .iter()
            .filter(|v| v["payload"]["stop_reason"] == "completed")
            .count(),
        1
    );
    assert_eq!(
        entries
            .iter()
            .filter(|v| v["payload"]["write_confirmed"] == true)
            .count(),
        2
    );
    assert_eq!(
        entries
            .iter()
            .filter(|v| v["payload"]["kind"] == "approval")
            .count(),
        1
    );
    worker.journal_inflight.clear();
    worker.journal_readers.clear();
    worker.project_journal(&detail, &summary);
    assert!(
        worker.journal_inflight.is_empty(),
        "aucun fait n'est réémis après flush sans curseur"
    );
    worker.relay.shutdown();
}

#[test]
fn spec101_t3_tour_notification_porte_origine_exclue_du_collecteur() {
    let (mut worker, _peer) = worker099();
    worker.state.seeded = true;
    worker.state.observations_seeded = true;
    worker.state.notification_messages.push("u".into());
    let detail = contract::parse_thread_detail(
        r#"{"thread":{
            "messages":[{"id":"u","role":"user","streaming":false,"createdAt":"date"}],
            "latestTurn":{"turnId":"t","state":"error","requestedAt":"date"},"activities":[]}}"#,
    )
    .unwrap();
    worker.project_journal(&detail, &summary(Some(("t", "error"))));
    worker.journal.stop();
    let entries = std::fs::read_dir(&worker.journal_dir)
        .unwrap()
        .flat_map(|path| bridget_transport::journal::valid_events(&path.unwrap().path()))
        .collect::<Vec<_>>();
    let end = entries.iter().find(|v| v["event"] == "turn_end").unwrap();
    assert!(
        end["message_id"]
            .as_str()
            .unwrap()
            .starts_with("bridget-observation:")
    );
    assert!(worker.state.pending.is_empty());
    worker.relay.shutdown();
}

#[test]
fn spec101_t3_seed_ne_consomme_pas_la_fin_future_du_tour_actif() {
    let (mut worker, _peer) = worker099();
    let mut detail = contract::parse_thread_detail(
        r#"{"thread":{
            "messages":[{"id":"u","role":"user","streaming":false,"createdAt":"date"},
                {"id":"a","role":"assistant","streaming":true,"turnId":"t"}],
            "latestTurn":{"turnId":"t","state":"running","requestedAt":"date"},"activities":[]}}"#,
    )
    .unwrap();
    worker.project_journal(&detail, &summary(Some(("t", "running"))));
    assert!(!worker.state.ended_turns.contains(&"t".to_string()));
    detail.latest_turn.as_mut().unwrap().state = "completed".into();
    worker.project_journal(&detail, &summary(Some(("t", "completed"))));
    worker.journal.stop();
    let entries = std::fs::read_dir(&worker.journal_dir)
        .unwrap()
        .flat_map(|path| bridget_transport::journal::valid_events(&path.unwrap().path()))
        .collect::<Vec<_>>();
    assert_eq!(
        entries
            .iter()
            .filter(|v| v["payload"]["stop_reason"] == "completed")
            .count(),
        1
    );
    worker.relay.shutdown();
}

// Réutilise LinkWorker, Session et socketpair : aucun daemon ou compte réel.
#[test]
fn spec101_t3_seed_ignore_fin_historique_sans_texte() {
    let (mut worker, _peer) = worker099();
    let detail = contract::parse_thread_detail(
        r#"{"thread":{
            "messages":[{"id":"u","role":"user","streaming":false,"createdAt":"date"}],
            "latestTurn":{"turnId":"t","state":"error","requestedAt":"date"},"activities":[]}}"#,
    )
    .unwrap();
    let summary = summary(Some(("t", "error")));
    worker.project_journal(&detail, &summary);
    worker.project_journal(&detail, &summary);
    worker.journal.stop();
    let entries = std::fs::read_dir(&worker.journal_dir)
        .unwrap()
        .flat_map(|path| bridget_transport::journal::valid_events(&path.unwrap().path()))
        .collect::<Vec<_>>();
    assert!(!entries.iter().any(|v| v["event"] == "turn_end"));
    worker.relay.shutdown();
}

fn worker099() -> (LinkWorker, UnixStream) {
    let root = std::env::temp_dir().join(format!("t3-spec099-{}", uuid::Uuid::new_v4()));
    let paths = Paths::at(root.clone(), root.join("bridget.sock"));
    paths.ensure().unwrap();
    let runtime = ServerRuntime {
        port: 1,
        pid: std::process::id(),
    };
    let session = Arc::new(Session::new(
        &paths,
        runtime,
        TokenFile {
            installation_id: "test".into(),
            label: "test".into(),
            session_id: "test".into(),
            token: "synthetic".into(),
            expires_at: String::new(),
            issued_at: String::new(),
        },
    ));
    let (stream, peer) = UnixStream::pair().unwrap();
    let worker = LinkWorker::new(
        &paths,
        session,
        &summary(None),
        stable_uuid("test"),
        stable_uuid("instance:test"),
        BufWriter::new(stream.try_clone().unwrap()),
        stream,
    )
    .unwrap();
    (worker, peer)
}

#[test]
fn user_input181_t3_capacity_updates_existing_receipt_without_eviction() {
    let (mut worker, _peer) = worker099();
    let frame = |id: String, state: &str| DaemonToWrapper::UserInputResult {
        result: json!({"item":{"request_id":id,"origin":"t3","state":state,"callback":{"provider_session":worker.thread_id.clone()}}}),
    };
    let frames = (0..33)
        .map(|i| frame(format!("input-{i}"), "pending"))
        .collect::<Vec<_>>();
    for message in frames.iter().take(32) {
        worker.handle_frame(message.clone());
    }
    assert_eq!(worker.user_inputs.len(), 32);
    worker.handle_frame(frames[0].clone());
    assert_eq!(
        worker.user_inputs.len(),
        32,
        "a replay at capacity must retain the existing request"
    );
    assert!(worker.user_inputs.contains_key("input-0"));
    worker.handle_frame(frames[32].clone());
    assert_eq!(worker.user_inputs.len(), 32);
    assert!(!worker.user_inputs.contains_key("input-32"));
    let closed = DaemonToWrapper::UserInputResult {
        result: json!({"item":{"request_id":"input-0","origin":"t3","state":"applied","callback":{"provider_session":worker.thread_id.clone()}}}),
    };
    worker.handle_frame(closed);
    assert_eq!(worker.user_inputs.len(), 31);
    worker.relay.shutdown();
    worker.journal.stop();
}

#[test]
fn user_input181_t3_link_publishes_routes_and_reconciles_without_prompt() {
    use crate::t3code_contract_v2 as v2;
    use bridget_transport::protocol::{
        ChildRequestAnswer, ChildRequestClosedState, ChildRequestDecision,
    };
    let (mut worker, peer) = worker099();
    peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    let mut reader = BufReader::new(peer);
    let thread = worker.thread_id.clone();
    let request_id = uuid::Uuid::new_v4().to_string();
    let command_id = uuid::Uuid::new_v4().to_string();
    let questions = serde_json::json!([{"id":"q","header":"Choix","question":"Lequel ?","options":[{"label":"A","description":"Exact"}],"allowCustomAnswer":true}]);
    let source = |status: &str, answers: serde_json::Value| serde_json::json!({"projection":{"thread":{"id":thread},"runs":[{"id":"r","threadId":thread,"ordinal":1,"userMessageId":"u","rootNodeId":"root","status":"waiting","requestedAt":"2026-10-25T08:00:00Z"}],"nodes":[{"id":"root","kind":"root_turn","parentNodeId":null}],"messages":[],"turnItems":[{"id":"i","threadId":thread,"runId":"r","nodeId":"root","type":"user_input_request","status":"pending","requestId":"source181","questions":questions}],"runtimeRequests":[{"id":"source181","nodeId":"root","kind":"user_input","status":status,"responseCapability":{"type":"live","providerSessionId":"sdk181"},"answers":answers}]}});
    let pending = source("pending", serde_json::Value::Null);
    worker.refresh_user_inputs(&v2::parse_detail(&pending.to_string(), &thread).unwrap());
    let callback = loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        if let Ok(WrapperToDaemon::ChildRequest { request }) = serde_json::from_str(line.trim()) {
            break request;
        }
    };
    worker.handle_frame(DaemonToWrapper::UserInputResult{result:serde_json::json!({"item":{"request_id":request_id,"origin":"t3","state":"pending","callback":callback}})});
    let mut answer = ChildRequestAnswer {
        request_id: request_id.clone(),
        provider_session: "foreign".into(),
        turn_id: "r".into(),
        provider_request_id: callback.provider_request_id.clone(),
        decision: ChildRequestDecision::Accept,
        answer: Some(serde_json::json!({"q":["A","complément"]})),
        outcome: command_id.clone(),
    };
    worker.answer_user_input(answer.clone());
    assert_eq!(worker.user_inputs[&request_id]["state"], "pending");
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    {
        let mut inner = worker.session.lock();
        inner.runtime.port = port;
        inner.client = Client::for_base_url(&format!("http://127.0.0.1:{port}"), "fixture");
        inner.client.set_test_protocol(true);
    }
    let body = pending.to_string();
    let path = format!("GET /api/orchestration/threads/{thread}/bounded?compactTurnItems=1 ");
    let server = thread::spawn(move || {
        v2::tests::http_reply(&listener, 200, &body, &path);
        v2::tests::rpc_reply(
            &listener,
            "orchestration.dispatchCommand",
            Some(serde_json::json!({"sequence":99})),
        )
    });
    answer.provider_session = thread.clone();
    worker.answer_user_input(answer);
    let wire = server.join().unwrap();
    assert_eq!(wire["type"], "runtime-request.respond");
    assert_eq!(wire["commandId"], command_id);
    assert_eq!(wire["answers"], serde_json::json!({"q":["A","complément"]}));
    let resolved = source("resolved", wire["answers"].clone());
    worker.refresh_user_inputs(&v2::parse_detail(&resolved.to_string(), &thread).unwrap());
    let closed = loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        if let Ok(WrapperToDaemon::ChildRequestClosed {
            request_id: id,
            state,
        }) = serde_json::from_str(line.trim())
        {
            break (id, state);
        }
    };
    assert_eq!(
        closed,
        (request_id, ChildRequestClosedState::AlreadyApplied)
    );
    worker.relay.shutdown();
    worker.journal.stop();
}

#[test]
fn spec138_t3_owner_announces_unknown_project_instead_of_using_adapter_cwd() {
    let (mut worker, peer) = worker099();
    peer.set_nonblocking(true).unwrap();
    let mut reader = BufReader::new(peer);
    let mut fact = None;
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let value: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
                if value["type"] == "CommunicationProjectFact" {
                    fact = Some(value);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(error) => panic!("lecture fait T3: {error}"),
        }
    }
    let fact = fact.expect("annonce propriétaire T3 absente");
    assert_eq!(fact["source"], "t3");
    assert_eq!(fact["root"], "");
    assert_eq!(fact["host"], crate::wrapper::host_name());
    assert!(fact["worktree_root"].is_null());
    worker.relay.shutdown();
    worker.journal.stop();
}

#[test]
fn spec145_t3_binding_follows_each_changed_project_fact() {
    let (mut worker, peer) = worker099();
    let mut changed = summary(None);
    changed.workspace_root = Some("/new-project".into());
    worker.report_project_context(&changed);
    peer.set_nonblocking(true).unwrap();
    let mut reader = BufReader::new(peer);
    let mut frames = Vec::new();
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => frames.push(serde_json::from_str::<serde_json::Value>(line.trim()).unwrap()),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) => panic!("lecture fixture: {e}"),
        }
    }
    let bindings = frames
        .iter()
        .enumerate()
        .filter(|(_, f)| f["type"] == "T3ThreadBindingFact")
        .collect::<Vec<_>>();
    assert_eq!(bindings.len(), 2);
    for (index, binding) in bindings {
        assert_eq!(frames[index - 1]["type"], "CommunicationProjectFact");
        assert_eq!(binding["version"], 1);
        assert_eq!(binding["t3_thread_id"], changed.id);
    }
    worker.relay.shutdown();
    worker.journal.stop();
}

#[test]
fn spec138_t3_reannounces_only_changed_project_facts_and_keeps_verdict_out_of_conversation() {
    let (mut worker, peer) = worker099();
    peer.set_nonblocking(true).unwrap();
    let mut reader = BufReader::new(peer);
    let drain = |reader: &mut BufReader<UnixStream>| {
        let mut facts = Vec::new();
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    let value: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
                    if value["type"] == "CommunicationProjectFact" {
                        facts.push(value);
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(error) => panic!("lecture fait T3: {error}"),
            }
        }
        facts
    };
    assert_eq!(drain(&mut reader).len(), 1);
    let mut current = summary(None);
    current.workspace_root = Some("/tmp/project-a".into());
    current.worktree_path = Some("/tmp/project-b".into());
    worker.last_key = current.change_key();
    worker.handle_event(LinkEvent::Tick(Box::new(current.clone())));
    worker.handle_event(LinkEvent::Tick(Box::new(current.clone())));
    let facts = drain(&mut reader);
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0]["root"], "/tmp/project-a");
    assert_eq!(facts[0]["worktree_root"], "/tmp/project-b");
    // Le transport expose la contradiction telle quelle ; il ne choisit
    // pas de lui-même le worktree comme projet. Le daemon tranche.
    worker.handle_frame(DaemonToWrapper::ProjectContextResult {
        project: None,
        project_warnings: Vec::new(),
    });
    assert!(worker.queue.is_empty());
    assert!(worker.state.pending.is_empty());
    assert!(drain(&mut reader).is_empty());
    current.workspace_root = None;
    worker.handle_event(LinkEvent::Tick(Box::new(current)));
    let facts = drain(&mut reader);
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0]["root"], "");
    worker.relay.shutdown();
    worker.journal.stop();
}

#[test]
fn incident171_elargissement_impossible_garde_la_page_bornee() {
    // Fil T3 de 30 532 éléments : la page élargie dépassait 16 Mio. Le
    // pont abandonnait alors la page bornée déjà lue, ne s'amorçait jamais
    // et ne voyait ni le tour démarré ni la réponse finale.
    let (mut worker, _peer) = worker099();
    assert!(!worker.state.seeded);
    let body = serde_json::json!({"thread":{
        "messages":[{"id":"u","role":"user","streaming":false,"createdAt":"date"}],
        "latestTurn":{"turnId":"t","state":"running","requestedAt":"date"},
        "activities":[]
    }})
    .to_string();
    // Une seule réponse : la seconde lecture (élargie) échoue au transport.
    let server = http_once099(&worker, 200, body.clone());
    let summary = summary(Some(("t", "running")));
    assert!(worker.refresh(&summary));
    assert!(server.join().unwrap().contains("turnLimit=20"));
    assert_eq!(worker.detail_page_cap, Some(DETAIL_PAGE));
    assert!(worker.state.seeded);
    assert!(worker.state.seen.iter().any(|id| id == "u"));
    // Passage suivant : une seule lecture bornée, sans nouvel élargissement.
    let server = http_once099(&worker, 200, body);
    assert!(worker.refresh(&summary));
    assert!(server.join().unwrap().contains("turnLimit=20"));
    worker.relay.shutdown();
    worker.journal.stop();
}

#[test]
fn spec101_http_capacites_honnetes_et_lacune_signalee() {
    use bridget_transport::protocol::ObservationKind as Kind;
    for provider in ["codex", "claudeAgent"] {
        let (mut worker, peer) = worker099();
        worker.state.seeded = true;
        worker.state.observations_seeded = true;
        worker
            .state
            .seen_activities
            .push("sortie-de-la-fenetre".into());
        worker.observation_events = vec![Kind::TurnEnded];
        let server = http_once099(&worker, 200, serde_json::json!({"thread":{
                "messages":[{"id":"u","role":"user","streaming":false,"createdAt":"date"}],
                "latestTurn":{"turnId":"t","state":"running","requestedAt":"date"},
                "activities":[{"id":"nouvelle","kind":"approval.requested","turnId":"t","payload":{}}]
            }}).to_string());
        let mut summary = summary(Some(("t", "running")));
        summary.provider_instance_id = Some(provider.into());
        assert!(worker.refresh(&summary));
        assert!(server.join().unwrap().starts_with("GET "));
        assert!(worker.observation_events.contains(&Kind::TurnEnded));
        assert!(
            worker
                .observation_events
                .contains(&Kind::PermissionRequired)
        );
        assert_eq!(
            worker.observation_events.contains(&Kind::FileWritten),
            provider == "codex"
        );
        peer.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        let mut reader = BufReader::new(peer);
        let mut saw_capabilities = false;
        loop {
            let mut line = String::new();
            assert!(reader.read_line(&mut line).unwrap() > 0);
            match decode::<WrapperToDaemon>(line.trim()).unwrap() {
                WrapperToDaemon::ObservationCapabilities { events, .. } => {
                    assert!(!events.is_empty(), "pas de fausse déconnexion");
                    saw_capabilities = true;
                }
                WrapperToDaemon::ObservationGap { dropped } => {
                    assert_eq!(dropped, 0, "ne pas quantifier une perte inconnue");
                    assert!(saw_capabilities);
                    break;
                }
                _ => {}
            }
        }
        worker.journal.stop();
        worker.relay.shutdown();
    }
}

#[test]
fn spec099_reponse_non_acquittee_et_autres_attentes_survivent() {
    let (mut worker, _peer) = worker099();
    let first = pending("uA", None);
    let mut second = pending("uB", None);
    second.request_id = "other".into();
    worker.state.pending = vec![first, second];
    let detail = ThreadDetail {
        activities: Vec::new(),
        activities_available: false,
        messages: vec![
            message("uA", "user", "nous", None, false),
            message("aA", "assistant", "réponse durable", Some("tA"), false),
        ],
        latest_turn: None,
    };
    worker.settle_pending(&detail, &summary(Some(("tA", "completed"))));
    let saved = ThreadState::load(&worker.state_path);
    assert_eq!(
        saved.pending.len(),
        2,
        "aucune attente ne disparaît avant confirmation"
    );
    assert!(
        serde_json::to_string(&saved)
            .unwrap()
            .contains("réponse durable")
    );
    worker.handle_frame(DaemonToWrapper::Nack {
        id: reply_id("req"),
        reason: "absent".into(),
    });
    assert_eq!(worker.state.pending.len(), 2);
    worker.handle_frame(DaemonToWrapper::RequestList { requests: vec![] });
    assert_eq!(
        worker.state.pending.len(),
        2,
        "une liste bornée vide ne prouve rien"
    );
    worker.handle_frame(DaemonToWrapper::Ack {
        project_warnings: Vec::new(),
        id: reply_id("req"),
    });
    assert_eq!(ThreadState::load(&worker.state_path).pending.len(), 1);
    worker.journal.stop();
    worker.relay.shutdown();
}

#[test]
fn spec099_journal_conserve_dix_mille_caracteres_unicode() {
    let (mut worker, _peer) = worker099();
    worker.state.seeded = true;
    let text = "é🦀漢".repeat(4000);
    let detail = ThreadDetail {
        activities: Vec::new(),
        activities_available: false,
        messages: vec![message("long", "assistant", &text, Some("t"), false)],
        latest_turn: None,
    };
    worker.project_journal(&detail, &summary(None));
    worker.journal.stop();
    let events: Vec<_> = std::fs::read_dir(&worker.journal_dir)
        .unwrap()
        .flat_map(|p| bridget_transport::journal::valid_events(&p.unwrap().path()))
        .collect();
    assert!(
        events.iter().any(|v| v["payload"]["text"] == text),
        "sortie intégrale"
    );
    worker.relay.shutdown();
}

#[test]
fn spec099_journal_refuse_n_avance_pas_le_repere() {
    let (mut worker, _peer) = worker099();
    worker.state.seeded = true;
    worker.journal.stop();
    let detail = ThreadDetail {
        activities: Vec::new(),
        activities_available: false,
        messages: vec![message("lost", "assistant", "texte", Some("t"), false)],
        latest_turn: None,
    };
    worker.project_journal(&detail, &summary(None));
    assert!(!worker.state.seen.iter().any(|id| id == "lost"));
    assert!(!worker.state.ended_turns.iter().any(|id| id == "t"));
    worker.relay.shutdown();
}

/// Session 112 : une échéance de tour posée par le daemon est un budget
/// d'exécution ; une longue attente en file ne l'entame pas.
#[test]
fn spec112_attente_en_file_ne_consomme_pas_le_budget_de_tour() {
    let (mut worker, _peer) = worker099();
    let mut message = bridget_core::BridgetMessage::new("alice", "bob", "budget");
    message.id = "budget-45min".into();
    message.deadline_at = Some(unix_now() + 2_700);
    worker.handle_frame(DaemonToWrapper::Deliver {
        message,
        mid_turn_ok: None,
    });
    // Antidater la réception au-delà du budget : sans report, la remise
    // serait périmée ; avec report, elle attend toujours.
    if let Some(entry) = worker.queue.front_mut() {
        entry.1 = Instant::now() - Duration::from_secs(3_600);
    }
    let (_tx, inbox) = mpsc::channel();
    worker.drive_queue(&inbox);
    assert_eq!(
        worker.queue.len(),
        1,
        "le budget court à partir du dispatch"
    );
    let received = worker.queue[0].1;
    let reporte = deadline_after_wait(unix_now() + 2_700 - 3_600, received);
    assert!(
        reporte >= unix_now() + 2_700 - 2,
        "report de l'attente sur l'échéance : {reporte}"
    );
}

/// Une remise déjà périmée à sa réception n'est pas ressuscitée par le report.
#[test]
fn spec112_echeance_deja_passee_a_la_reception_reste_ecartee() {
    let (mut worker, _peer) = worker099();
    let mut message = bridget_core::BridgetMessage::new("alice", "bob", "stale");
    message.id = "stale".into();
    message.deadline_at = Some(unix_now() - 10);
    worker.handle_frame(DaemonToWrapper::Deliver {
        message,
        mid_turn_ok: None,
    });
    let (_tx, inbox) = mpsc::channel();
    worker.drive_queue(&inbox);
    assert!(worker.queue.is_empty());
}

/// Session 111 : un message sans échéance attend la fin du tour, même
/// bien après l'ancien délai de deux minutes qui le jetait.
#[test]
fn spec111_remise_sans_echeance_survit_a_une_longue_attente() {
    let (mut worker, _peer) = worker099();
    let mut message = bridget_core::BridgetMessage::new("alice", "bob", "libération");
    message.id = "attente-longue".into();
    worker.handle_frame(DaemonToWrapper::Deliver {
        message,
        mid_turn_ok: None,
    });
    // Antidater la réception bien au-delà de l'ancien seuil.
    let ancien_seuil = worker.turn_wait * 10;
    if let Some(entry) = worker.queue.front_mut() {
        entry.1 = Instant::now() - ancien_seuil;
    }
    let (_tx, inbox) = mpsc::channel();
    worker.drive_queue(&inbox);
    assert_eq!(
        worker.queue.len(),
        1,
        "une remise sans échéance ne périme pas sur la seule attente"
    );
    assert_eq!(delivery_message(&worker.queue[0].0).id, "attente-longue");
}

/// Une échéance réellement portée par le message reste respectée.
#[test]
fn spec111_remise_avec_echeance_depassee_est_toujours_ecartee() {
    let (mut worker, _peer) = worker099();
    let mut message = bridget_core::BridgetMessage::new("alice", "bob", "périmée");
    message.id = "echeance-depassee".into();
    message.deadline_at = Some(unix_now() - 1);
    worker.handle_frame(DaemonToWrapper::Deliver {
        message,
        mid_turn_ok: None,
    });
    let (_tx, inbox) = mpsc::channel();
    worker.drive_queue(&inbox);
    assert!(
        worker.queue.is_empty(),
        "une échéance dépassée écarte toujours la remise"
    );
}

/// La borne de file remplace l'ancien garde-fou temporel : au-delà, la plus
/// ancienne est écartée, et la file cesse de grandir.
#[test]
fn spec111_file_saturee_ecarte_la_plus_ancienne() {
    let (mut worker, _peer) = worker099();
    for index in 0..(QUEUE_BOUND + 5) {
        let mut message = bridget_core::BridgetMessage::new("alice", "bob", "lot");
        message.id = format!("m-{index}");
        worker.handle_frame(DaemonToWrapper::Deliver {
            message,
            mid_turn_ok: None,
        });
    }
    assert_eq!(worker.queue.len(), QUEUE_BOUND, "file bornée");
    assert_eq!(
        delivery_message(&worker.queue[0].0).id,
        "m-5",
        "les plus anciennes sont écartées, les récentes conservées"
    );
    assert_eq!(
        delivery_message(&worker.queue[QUEUE_BOUND - 1].0).id,
        format!("m-{}", QUEUE_BOUND + 4)
    );
}

/// Session 113 : toutes les trames émises vers le daemon, sans filtrage.
fn frames113(peer: UnixStream) -> Vec<WrapperToDaemon> {
    peer.set_nonblocking(true).unwrap();
    let mut reader = BufReader::new(peer);
    let mut frames = Vec::new();
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => frames.push(serde_json::from_str(&line).unwrap()),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) => panic!("lecture des trames : {e}"),
        }
    }
    frames
}

fn idempotent113(id: &str) -> DaemonToWrapper {
    let mut message = bridget_core::BridgetMessage::new("alice", "bob", "suivie");
    message.id = id.into();
    message.reply = true;
    DaemonToWrapper::DeliverIdempotent {
        delivery_id: format!("d-{id}"),
        recipient_instance_id: stable_uuid("instance:test"),
        delivery_generation: 1,
        expires_at: unix_now() as i64 + 3600,
        message,
        execution: None,
        mid_turn_ok: None,
    }
}

fn saga_closed113(frames: &[WrapperToDaemon], delivery_id: &str) -> bool {
    frames.iter().any(|frame| {
        matches!(
            frame,
            WrapperToDaemon::DeliveryIndeterminate { delivery_id: id, delivery_generation: 1 }
                if id == delivery_id
        )
    })
}

fn rejection113<'a>(frames: &'a [WrapperToDaemon], message_id: &str) -> Option<&'a str> {
    frames.iter().find_map(|frame| match frame {
        WrapperToDaemon::DeliveryRejected { id, reason, .. } if id == message_id => {
            Some(reason.as_str())
        }
        _ => None,
    })
}

/// La saturation historique ne doit pas évincer une remise durable.
#[test]
fn spec244_saturation_ne_clot_pas_la_remise_idempotente() {
    let (mut worker, peer) = worker099();
    worker.handle_frame(idempotent113("premiere"));
    for index in 0..QUEUE_BOUND {
        let mut message = bridget_core::BridgetMessage::new("alice", "bob", "lot");
        message.id = format!("m-{index}");
        worker.handle_frame(DaemonToWrapper::Deliver {
            message,
            mid_turn_ok: None,
        });
    }
    assert_eq!(worker.queue.len(), QUEUE_BOUND);
    let frames = frames113(peer);
    assert!(!saga_closed113(&frames, "d-premiere"));
    assert!(rejection113(&frames, "premiere").is_none());
    assert_eq!(delivery_message(&worker.queue[0].0).id, "premiere");
}

#[test]
fn spec244_saturation_passe_sur_disque_et_reprend_dans_l_ordre() {
    let (mut worker, peer) = worker099();
    let mut frames = Vec::new();
    for i in 0..200 {
        worker.handle_frame(idempotent113(&format!("m-{i}")));
        frames.extend(frames113(peer.try_clone().unwrap()));
    }
    worker.handle_frame(idempotent113("m-199"));
    assert_eq!(worker.queue.len(), QUEUE_BOUND);
    assert!(worker.overflow.has_pending());
    frames.extend(frames113(peer));
    assert!(frames.iter().all(|f| !matches!(
        f,
        WrapperToDaemon::DeliveryIndeterminate { .. } | WrapperToDaemon::DeliveryRejected { .. }
    )));
    worker.queue.clear();
    worker.refill_deliveries();
    assert_eq!(
        delivery_message(&worker.queue[0].0).id,
        format!("m-{QUEUE_BOUND}")
    );
    worker.handle_frame(DaemonToWrapper::CancelDelivery {
        id: "m-199".into(),
        reason: "test".into(),
        interrupt_active: None,
    });
    let mut count = QUEUE_BOUND * 2;
    while worker.overflow.has_pending() {
        worker.queue.clear();
        worker.refill_deliveries();
        for (frame, _) in &worker.queue {
            assert_ne!(delivery_message(frame).id, "m-199");
        }
        count += worker.queue.len();
    }
    assert_eq!(count, 199);
}

/// Une échéance passée avant démarrage vaut refus explicite : le motif
/// porte « échéance », ce sur quoi le daemon clôt la demande suivie.
#[test]
fn spec113_echeance_passee_signale_refus_et_saga_close() {
    let (mut worker, peer) = worker099();
    let mut frame = idempotent113("perimee");
    if let DaemonToWrapper::DeliverIdempotent { message, .. } = &mut frame {
        message.deadline_at = Some(unix_now() - 1);
    }
    worker.handle_frame(frame);
    let (_tx, inbox) = mpsc::channel();
    worker.drive_queue(&inbox);
    assert!(worker.queue.is_empty());
    let frames = frames113(peer);
    assert!(saga_closed113(&frames, "d-perimee"), "saga close");
    assert!(
        rejection113(&frames, "perimee").is_some_and(|reason| reason.contains("échéance")),
        "refus porteur du motif d'échéance"
    );
}

/// Une demande déjà close côté daemon (répondue, annulée, expirée) a déjà
/// son issue : la saga est close, aucun second échec n'est fabriqué.
#[test]
fn spec113_demande_close_clot_la_saga_sans_second_echec() {
    let (mut worker, peer) = worker099();
    worker.handle_frame(idempotent113("close"));
    worker.handle_frame(idempotent113("annulee"));
    worker.handle_frame(DaemonToWrapper::RequestList {
        requests: vec![RequestInfo {
            id: "close".into(),
            sender: "alice".into(),
            target: worker.agent_id.clone(),
            state: "answered".into(),
            created_at: 0,
            deadline_at: i64::MAX,
            cancel_reason: None,
            deferred_reminder_level: None,
            deferred_reminder_at: None,
        }],
    });
    worker.handle_frame(DaemonToWrapper::CancelDelivery {
        id: "annulee".into(),
        reason: "test".into(),
        interrupt_active: None,
    });
    assert!(worker.queue.is_empty());
    let frames = frames113(peer);
    assert!(
        saga_closed113(&frames, "d-close"),
        "saga de la demande répondue close"
    );
    assert!(
        saga_closed113(&frames, "d-annulee"),
        "saga de la demande annulée close"
    );
    assert!(rejection113(&frames, "close").is_none());
    assert!(rejection113(&frames, "annulee").is_none());
}

#[test]
fn spec099_controles_retirent_seulement_la_demande_annulee() {
    let (mut worker, _peer) = worker099();
    let mut cancelled = bridget_core::BridgetMessage::new("alice", "bob", "annulée");
    cancelled.id = "cancelled".into();
    let mut next = cancelled.clone();
    next.id = "next".into();
    let started = Instant::now();
    worker.handle_frame(DaemonToWrapper::Deliver {
        message: cancelled,
        mid_turn_ok: None,
    });
    worker.handle_frame(DaemonToWrapper::Deliver {
        message: next,
        mid_turn_ok: None,
    });
    worker.handle_frame(DaemonToWrapper::CancelDelivery {
        id: "cancelled".into(),
        reason: "test".into(),
        interrupt_active: None,
    });
    assert!(
        started.elapsed() < Duration::from_millis(100),
        "aucune attente fournisseur dans le traitement des contrôles"
    );
    assert_eq!(worker.queue.len(), 1);
    assert_eq!(delivery_message(&worker.queue[0].0).id, "next");
    worker.handle_frame(DaemonToWrapper::Disconnect);
    let (_tx, inbox) = mpsc::channel();
    worker.drive_queue(&inbox);
    assert!(!worker.daemon_alive.load(Ordering::SeqCst));
    assert!(
        worker.state.pending.is_empty(),
        "aucune acceptation après disparition de l'autorité"
    );
    worker.journal.stop();
    worker.relay.shutdown();
}

#[test]
fn spec099_peremption_demande_et_tour_avant_tout_http() {
    let (mut worker, _peer) = worker099();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    worker.session.reset_runtime(ServerRuntime {
        port: listener.local_addr().unwrap().port(),
        pid: std::process::id(),
    });
    let mut message = bridget_core::BridgetMessage::new("alice", "bob", "périmée");
    message.deadline_at = Some(unix_now().saturating_sub(1));
    worker.handle_frame(DaemonToWrapper::Deliver {
        message: message.clone(),
        mid_turn_ok: None,
    });
    let (_tx, inbox) = mpsc::channel();
    worker.drive_queue(&inbox);
    assert!(worker.queue.is_empty());
    message.deadline_at = Some(unix_now() + 1000);
    message.reply_timeout = Some(0);
    worker.handle_frame(DaemonToWrapper::Deliver {
        message: message.clone(),
        mid_turn_ok: None,
    });
    worker.drive_queue(&inbox);
    assert!(
        worker.queue.is_empty(),
        "l'échéance demande peut précéder le tour"
    );
    message.reply_timeout = None;
    worker.handle_frame(DaemonToWrapper::Deliver {
        message: message.clone(),
        mid_turn_ok: None,
    });
    worker.requests.insert(
        message.id.clone(),
        RequestInfo {
            id: message.id,
            sender: "alice".into(),
            target: worker.agent_id.clone(),
            state: "open".into(),
            created_at: unix_now() as i64 - 100,
            deadline_at: unix_now() as i64 - 1,
            cancel_reason: None,
            deferred_reminder_level: None,
            deferred_reminder_at: None,
        },
    );
    worker.drive_queue(&inbox);
    assert!(worker.queue.is_empty());
    assert!(
        matches!(listener.accept(), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock),
        "aucun appel fournisseur pour un travail périmé"
    );
    worker.journal.stop();
    worker.relay.shutdown();
}

#[test]
fn spec099_journal_echec_apres_enqueue_ne_confirme_rien() {
    let (mut worker, _peer) = worker099();
    worker.state.seeded = true;
    let date = &contract::iso_now()[..10];
    // SessionJournal ouvre son fichier lors du premier append : rendre
    // ce chemin inouvrable provoque un échec réel APRÈS admission en file.
    std::fs::create_dir(worker.journal_dir.join(format!("{date}.jsonl"))).unwrap();
    let detail = ThreadDetail {
        activities: Vec::new(),
        activities_available: false,
        messages: vec![message("failed", "assistant", "texte", Some("t"), false)],
        latest_turn: None,
    };
    worker
        .journal
        .enqueue(
            "update",
            Some("failed"),
            serde_json::json!({"kind": "text", "text": "texte"}),
        )
        .unwrap();
    worker.journal.stop();
    assert!(worker.journal_failed.load(Ordering::SeqCst));
    worker.project_journal(&detail, &summary(None));
    worker.confirm_journal();
    assert!(!worker.state.seen.contains(&"failed".to_string()));
    assert!(
        !ThreadState::load(&worker.state_path)
            .seen
            .contains(&"failed".to_string())
    );
    worker.relay.shutdown();
}

#[test]
fn spec099_reprise_journal_apres_flush_sans_curseur_ne_duplique_pas() {
    let (mut worker, _peer) = worker099();
    worker.state.seeded = true;
    let detail = ThreadDetail {
        activities: Vec::new(),
        activities_available: false,
        messages: vec![message(
            "once",
            "assistant",
            "é🦀".repeat(6000).as_str(),
            Some("t"),
            false,
        )],
        latest_turn: None,
    };
    worker.project_journal(&detail, &summary(None));
    worker.journal.stop();
    assert!(
        !worker.state.seen.contains(&"once".to_string()),
        "enqueue seul ne confirme pas"
    );
    // Simuler le redémarrage entre append et conservation du curseur.
    worker.journal_inflight.clear();
    worker.journal_readers.clear();
    worker.project_journal(&detail, &summary(None));
    assert!(worker.state.seen.contains(&"once".to_string()));
    assert!(worker.journal_inflight.is_empty(), "rien à réémettre");
    let events: Vec<_> = std::fs::read_dir(&worker.journal_dir)
        .unwrap()
        .flat_map(|p| bridget_transport::journal::valid_events(&p.unwrap().path()))
        .collect();
    assert_eq!(
        events
            .iter()
            .filter(|v| v["event"] == "update" && v["message_id"] == "once")
            .count(),
        1
    );
    worker.relay.shutdown();
}

#[test]
fn spec099_reponse_rechargee_sans_reexecution_et_ack_perdu_reconcilie() {
    let (mut worker, _peer) = worker099();
    let mut ready = pending("uA", None);
    ready.response = Some("déjà produite".into());
    worker.state.pending.push(ready);
    worker.state.save(&worker.state_path).unwrap();
    worker.state = ThreadState::load(&worker.state_path);
    worker.send_ready_responses();
    assert_eq!(
        worker.state.pending.len(),
        1,
        "l'envoi ne vaut pas acquittement"
    );
    worker.handle_frame(DaemonToWrapper::RequestList {
        requests: vec![RequestInfo {
            id: "req".into(),
            sender: "alice".into(),
            target: worker.agent_id.clone(),
            state: "answered".into(),
            created_at: 0,
            deadline_at: i64::MAX,
            cancel_reason: None,
            deferred_reminder_level: None,
            deferred_reminder_at: None,
        }],
    });
    assert!(ThreadState::load(&worker.state_path).pending.is_empty());
    assert!(
        worker.queue.is_empty(),
        "la reprise de réponse ne crée aucune commande fournisseur"
    );
    worker.journal.stop();
    worker.relay.shutdown();
}

#[test]
fn spec099_ancien_etat_pending_reste_lisible() {
    let state: ThreadState = serde_json::from_value(serde_json::json!({"pending": [{
        "request_id": "legacy", "from": "alice", "message_id": "u", "anchor_turn_id": null,
        "dispatched_at": "2026-09-16T00:00:00Z", "attempts": 0
    }]}))
    .unwrap();
    assert!(state.pending[0].response.is_none());
}

#[test]
fn spec099_annulation_signalee_par_lecteur_interdit_la_frontiere_http() {
    let (mut worker, _peer) = worker099();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    worker.session.reset_runtime(ServerRuntime {
        port: listener.local_addr().unwrap().port(),
        pid: std::process::id(),
    });
    let message = bridget_core::BridgetMessage::new("alice", &worker.agent_id, "ne démarre pas");
    // Le lecteur a reçu CancelDelivery mais sa trame n'a pas encore été
    // traitée par le worker occupé à persister une remise.
    worker
        .cancelled
        .lock()
        .unwrap()
        .insert(message.id.clone(), Instant::now());
    assert!(worker.dispatch_with_id(&message, &summary(None)).is_err());
    assert!(matches!(listener.accept(), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock));
    worker.journal.stop();
    worker.relay.shutdown();
}

#[test]
fn spec099_journal_hors_borne_rend_une_lacune_explicite() {
    let (mut worker, _peer) = worker099();
    worker.state.seeded = true;
    // Les contrôles s'échappent sur six octets en JSON : la limite doit
    // porter sur la ligne sérialisée, pas la longueur du texte source.
    let detail = ThreadDetail {
        activities: Vec::new(),
        activities_available: false,
        messages: vec![message(
            "oversized",
            "assistant",
            &"\u{1}".repeat(800_000),
            None,
            false,
        )],
        latest_turn: None,
    };
    worker.project_journal(&detail, &summary(None));
    worker.journal.stop();
    worker.confirm_journal();
    let events: Vec<_> = std::fs::read_dir(&worker.journal_dir)
        .unwrap()
        .flat_map(|p| bridget_transport::journal::valid_events(&p.unwrap().path()))
        .collect();
    assert!(
        events
            .iter()
            .any(|v| v["event"] == "error" && v["payload"]["gap"] == true)
    );
    assert!(!events.iter().any(|v| v["event"] == "update"));
    assert!(
        worker.state.seen.contains(&"oversized".to_string()),
        "la lacune n'est confirmée qu'après append"
    );
    worker.relay.shutdown();
}

// Aucun équivalent HTTP dans ce module : un seul échange sur boucle
// locale, délai d'attente borné, aucun sous-processus ni fournisseur réel.
fn http_once099(worker: &LinkWorker, status: u16, body: String) -> thread::JoinHandle<String> {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    worker.session.reset_runtime(ServerRuntime {
        port: listener.local_addr().unwrap().port(),
        pid: std::process::id(),
    });
    worker
        .session
        .inner
        .lock()
        .unwrap()
        .client
        .set_test_protocol(false);
    thread::spawn(move || {
        // Session 132 : 1 s ne suffisait pas sous forte charge (accept tardif).
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline =>
                {
                    thread::sleep(Duration::from_millis(5))
                }
                _ => return String::new(),
            }
        };
        stream.set_nonblocking(false).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut headers = String::new();
        let mut length = 0;
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            if let Some((name, value)) = line.split_once(':')
                && name.eq_ignore_ascii_case("content-length")
            {
                length = value.trim().parse().unwrap();
            }
            headers.push_str(&line);
            if line == "\r\n" || line.is_empty() {
                break;
            }
        }
        let mut content = vec![0; length];
        reader.read_exact(&mut content).unwrap();
        write!(
            stream,
            "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
        headers + &String::from_utf8(content).unwrap()
    })
}

#[test]
fn spec099_refus_publication_est_repris_sur_tick_inchange() {
    let (mut worker, _peer) = worker099();
    worker.state.seeded = true;
    let summary = summary(None);
    worker.last_key = summary.change_key();
    let detail = ThreadDetail {
        activities: Vec::new(),
        activities_available: false,
        messages: vec![message("retry", "assistant", "conservé", Some("t"), false)],
        latest_turn: None,
    };
    // Même branche Err que queueFull, sans failure sink asynchrone : cet
    // oracle n'affirme pas provoquer une saturation réelle de la file.
    worker.journal.stop();
    worker.project_journal(&detail, &summary);
    worker.confirm_journal();
    assert!(!worker.journal_failed.load(Ordering::SeqCst));
    assert!(worker.journal_inflight.is_empty());
    worker.journal = Arc::new(
        JournalWriter::start_with_live_feed_and_failure(
            worker.journal_dir.parent().unwrap(),
            &worker.agent_id,
            "retry-instance",
            Arc::new(|_| {}),
            None,
        )
        .unwrap(),
    );
    let server = http_once099(&worker, 200, serde_json::json!({"thread": {"messages": [{"id": "retry", "role": "assistant", "text": "conservé", "streaming": false, "turnId": "t"}]}}).to_string());
    worker.handle_event(LinkEvent::Tick(Box::new(summary)));
    worker.journal.stop();
    let request = server.join().unwrap();
    worker.confirm_journal();
    worker.relay.shutdown();
    assert!(
        request.starts_with("GET /api/orchestration/threads/"),
        "un refus ne fige pas la clé du fil"
    );
    assert!(worker.state.seen.contains(&"retry".to_string()));
    assert!(worker.state.ended_turns.contains(&"t".to_string()));
}

#[test]
fn spec099_post_accepte_issue_illisible_conserve_la_correlation() {
    for (status, response) in [(200, "{}"), (500, "erreur après acceptation")] {
        let (mut worker, _peer) = worker099();
        let mut request =
            bridget_core::BridgetMessage::new("alice", &worker.agent_id, "travail accepté");
        request.reply = true;
        let server = http_once099(&worker, status, response.into());
        assert!(worker.dispatch_with_id(&request, &summary(None)).is_err());
        let accepted = server.join().unwrap();
        assert!(accepted.starts_with("POST /api/orchestration/dispatch"));
        let saved = ThreadState::load(&worker.state_path);
        assert_eq!(
            saved.pending.len(),
            1,
            "une issue HTTP inconnue ne prouve pas le refus du tour"
        );
        let detail = ThreadDetail {
            activities: Vec::new(),
            activities_available: false,
            messages: vec![
                message(&saved.pending[0].message_id, "user", "accepté", None, false),
                message("answer", "assistant", "réponse récupérée", Some("t"), false),
            ],
            latest_turn: None,
        };
        worker.settle_pending(&detail, &summary(Some(("t", "completed"))));
        assert_eq!(
            worker.state.pending[0].response.as_deref(),
            Some("réponse récupérée")
        );
        worker.journal.stop();
        worker.relay.shutdown();
    }
}

#[test]
fn spec147_v2_lost_receipt_keeps_pending_and_recovers_exact_answer() {
    let (mut worker, _peer) = worker099();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    worker.session.reset_runtime(ServerRuntime {
        port: listener.local_addr().unwrap().port(),
        pid: std::process::id(),
    });
    worker
        .session
        .inner
        .lock()
        .unwrap()
        .client
        .set_test_protocol(true);
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut socket = tungstenite::accept(stream).unwrap();
        let frame: serde_json::Value =
            serde_json::from_str(socket.read().unwrap().to_text().unwrap()).unwrap();
        frame["payload"].clone() // Connexion perdue après admission simulée.
    });
    let mut request = bridget_core::BridgetMessage::new("alice", &worker.agent_id, "question");
    request.reply = true;
    assert!(worker.dispatch_with_id(&request, &summary(None)).is_err());
    let accepted = server.join().unwrap();
    let saved = ThreadState::load(&worker.state_path);
    assert_eq!(saved.pending.len(), 1);
    assert_eq!(accepted["messageId"], saved.pending[0].message_id);
    assert!(accepted.get("deliveryIntent").is_none());
    let mut detail = detail116(
        vec![
            message(
                &saved.pending[0].message_id,
                "user",
                "question",
                Some("r"),
                false,
            ),
            message("a", "assistant", "réponse récupérée", Some("r"), false),
        ],
        ("r", "completed"),
    );
    detail.activities.push(contract::Activity {id:"v2-run:r".into(),kind:"t3v2.run".into(),turn_id:Some("r".into()),payload:serde_json::json!({"status":"completed","userMessageId":saved.pending[0].message_id})});
    worker.settle_pending(&detail, &summary(Some(("r", "completed"))));
    assert_eq!(
        worker.state.pending[0].response.as_deref(),
        Some("réponse récupérée")
    );
    worker.journal.stop();
    worker.relay.shutdown();
}

#[test]
fn spec147_v2_queued_run_does_not_close_running_observation() {
    let (mut worker, _peer) = worker099();
    let mut detail = detail116(
        vec![
            message("u", "user", "question", Some("running"), false),
            message("a", "assistant", "en cours", Some("running"), false),
            message("queued-user", "user", "attente", Some("queued"), false),
        ],
        ("queued", "queued"),
    );
    for (id, status, user) in [
        ("running", "running", "u"),
        ("queued", "queued", "queued-user"),
    ] {
        detail.activities.push(contract::Activity {
            id: format!("v2-run:{id}"),
            kind: "t3v2.run".into(),
            turn_id: Some(id.into()),
            payload: serde_json::json!({"status":status,"userMessageId":user}),
        });
    }
    let summary = summary(Some(("queued", "queued")));
    worker.record_turn_origin(&detail, &summary);
    worker.project_journal(&detail, &summary);
    assert!(!worker.state.ended_turns.contains(&"running".into()));
    worker.project_journal(&detail, &summary);
    assert!(!worker.journal_inflight.contains("end:running"));
    detail.activities[0].payload["status"] = serde_json::json!("completed");
    worker.project_journal(&detail, &summary);
    worker.journal.stop();
    worker.confirm_journal();
    assert!(worker.state.ended_turns.contains(&"running".into()));
    assert!(!worker.state.ended_turns.contains(&"queued".into()));
    worker.relay.shutdown();
}

fn message(id: &str, role: &str, text: &str, turn: Option<&str>, streaming: bool) -> Message {
    Message {
        created_at: None,
        id: id.to_string(),
        role: role.to_string(),
        text: text.to_string(),
        streaming,
        turn_id: turn.map(str::to_string),
    }
}

fn summary(latest: Option<(&str, &str)>) -> ThreadSummary {
    ThreadSummary {
        id: "thread-1".to_string(),
        project_id: "p".to_string(),
        workspace_root: None,
        title: "Fil".to_string(),
        provider_instance_id: None,
        runtime_mode: "approval-required".to_string(),
        interaction_mode: "default".to_string(),
        worktree_path: None,
        archived_at: None,
        deleted_at: None,
        settled_override: None,
        relationship_to_parent: None,
        updated_at: "2026-09-14T00:00:00Z".to_string(),
        latest_turn: latest.map(|(id, state)| LatestTurn {
            requested_at: None,
            turn_id: id.to_string(),
            state: state.to_string(),
            assistant_message_id: None,
        }),
        session: None,
    }
}

#[test]
fn spec165_decision_conserve_racines_et_travail_actif_seulement() {
    let root = summary(Some(("done", "completed")));
    assert!(thread_requires_link(&root, false));

    let mut fork = root.clone();
    fork.relationship_to_parent = Some("fork".into());
    assert!(thread_requires_link(&fork, false));

    let mut child = root.clone();
    child.relationship_to_parent = Some("subagent".into());
    assert!(!thread_requires_link(&child, false));
    assert!(thread_requires_link(&child, true));

    child.latest_turn.as_mut().unwrap().state = "running".into();
    assert!(thread_requires_link(&child, false));
    child.latest_turn = None;
    child.session = Some(contract::SessionSummary {
        provider_name: "codex".into(),
        provider_instance_id: Some("codex".into()),
        status: "running".into(),
        active_turn_id: Some("active".into()),
    });
    assert!(thread_requires_link(&child, false));
    child.archived_at = Some("now".into());
    assert!(!thread_requires_link(&child, true));
}

#[test]
fn spec165_mille_enfants_terminaux_ne_deviennent_pas_mille_liens() {
    let mut threads = Vec::new();
    for index in 0..1_000 {
        let mut child = summary(Some(("done", "completed")));
        child.id = format!("child-{index}");
        child.relationship_to_parent = Some("subagent".into());
        threads.push(child);
    }
    for index in 0..30 {
        let mut root = summary(Some(("done", "completed")));
        root.id = format!("root-{index}");
        threads.push(root);
    }
    assert_eq!(
        threads
            .iter()
            .filter(|thread| thread_requires_link(thread, false))
            .count(),
        30
    );
}

#[test]
fn spec165_action_draine_sans_repousser_la_borne() {
    let mut child = summary(Some(("done", "completed")));
    child.relationship_to_parent = Some("subagent".into());
    let now = Instant::now();
    let wait = Duration::from_secs(120);
    assert_eq!(
        link_action(&child, false, None, None, false, now, wait),
        LinkAction::Ignore
    );
    assert_eq!(
        link_action(&child, false, None, None, true, now, wait),
        LinkAction::Open
    );
    for _ in 0..5 {
        assert_eq!(
            link_action(&child, true, None, Some(now), false, now, wait),
            LinkAction::Drain
        );
    }
    let key = child.change_key();
    assert_eq!(
        link_action(&child, true, Some(&key), Some(now), false, now, wait),
        LinkAction::Close
    );
    child.updated_at = "nouvelle-cle-terminale".into();
    assert_eq!(
        link_action(&child, true, None, Some(now - wait), false, now, wait),
        LinkAction::Close
    );
    child.latest_turn.as_mut().unwrap().state = "running".into();
    assert_eq!(
        link_action(&child, true, None, Some(now), false, now, wait),
        LinkAction::Tick
    );
}

#[test]
fn spec165_obligation_couvre_pending_et_tour_connu_non_projete() {
    let completed = summary(Some(("turn-final", "completed")));
    let mut state = ThreadState::default();
    assert!(!state.has_drain_obligation(&completed));
    state.pending.push(pending("message", Some("turn-final")));
    assert!(state.has_drain_obligation(&completed));
    state.pending.clear();
    state
        .turn_origins
        .insert("turn-final".into(), "message".into());
    assert!(state.has_drain_obligation(&completed));
    state.ended_turns.push("turn-final".into());
    assert!(!state.has_drain_obligation(&completed));
}

#[test]
fn spec165_cent_obligations_rouvrent_trente_deux_drains() {
    let mut threads = Vec::new();
    let mut obligations = HashSet::new();
    for index in 0..100 {
        let mut child = summary(Some(("done", "completed")));
        child.id = format!("obligation-{index:03}");
        child.provider_instance_id = Some("codex".into());
        child.relationship_to_parent = Some("subagent".into());
        obligations.insert(child.id.clone());
        threads.push(child);
    }
    let selected = select_drain_reopens(
        &threads,
        &obligations,
        &HashSet::new(),
        &HashSet::new(),
        &HashSet::new(),
        0,
    );
    assert_eq!(selected.len(), 32);
    assert_eq!(obligations.len() - selected.len(), 68);

    let cooldowns: HashSet<String> = selected.iter().map(|id| (*id).to_string()).collect();
    let next = select_drain_reopens(
        &threads,
        &obligations,
        &HashSet::new(),
        &HashSet::new(),
        &cooldowns,
        0,
    );
    assert_eq!(next.len(), 32);
    assert!(next.iter().all(|id| !cooldowns.contains(*id)));
}

#[test]
fn spec165_reconciliation_retire_un_lien_absent_du_snapshot() {
    let present = summary(Some(("done", "completed")));
    let summaries = HashMap::from([(present.id.as_str(), &present)]);
    let linked = [present.id.clone(), "thread-supprime".to_string()];
    assert_eq!(
        links_missing_from_snapshot(linked.iter(), &summaries),
        vec!["thread-supprime".to_string()]
    );
}

#[test]
fn spec165_reprise_ne_consomme_la_tentative_qu_a_la_fermeture() {
    let id = "obligation";
    let mut obligations = HashSet::from([id.to_string()]);
    let mut attempted = HashSet::new();
    let mut drain_reopens = HashSet::new();

    record_reopen_started(id, &mut obligations, &mut drain_reopens);
    assert!(drain_reopens.contains(id));
    assert!(!attempted.contains(id));

    // Une mort du worker peut donc remettre l'obligation en attente.
    obligations.insert(id.to_string());
    assert!(!attempted.contains(id));

    record_terminal_child_closed(id, &mut obligations, &mut attempted);
    assert!(attempted.contains(id));
    assert!(!obligations.contains(id));
}

#[test]
fn spec165_remise_invalide_la_cle_drainee() {
    let (mut worker, _peer) = worker099();
    worker.last_key = summary(Some(("done", "completed"))).change_key();
    worker.journal_caught_up = true;
    worker.publish_drain_state(true);
    assert!(worker.drained_key.lock().recover_poison().is_some());
    worker.handle_frame(DaemonToWrapper::Deliver {
        message: plain114("pendant-drain"),
        mid_turn_ok: None,
    });
    worker.publish_drain_state(true);
    assert!(worker.drained_key.lock().recover_poison().is_none());
    worker.journal.stop();
    worker.relay.shutdown();
}

#[test]
fn spec165_dispatch_attend_un_nouveau_snapshot_avant_drain() {
    let (mut worker, _peer) = worker099();
    let previous = summary(Some(("done", "completed"))).change_key();
    worker.last_key = previous.clone();
    worker.awaiting_snapshot_change = Some(previous.clone());
    worker.journal_caught_up = true;
    worker.publish_drain_state(true);
    assert!(worker.drained_key.lock().recover_poison().is_none());
    worker.observe_snapshot_change(&previous);
    assert!(worker.awaiting_snapshot_change.is_some());
    worker.observe_snapshot_change("nouveau-tour");
    worker.publish_drain_state(true);
    assert!(worker.drained_key.lock().recover_poison().as_deref() == Some(previous.as_str()));
    worker.journal.stop();
    worker.relay.shutdown();
}

#[test]
fn spec221_journal_reader_serves_eight_entries_without_worker_progress() {
    use bridget_transport::protocol::AttachWindow;
    use std::io::Write;
    let (mut worker, peer) = worker099();
    for n in 0..8 {
        worker
            .journal
            .enqueue(
                "update",
                Some(&format!("journal-{n}")),
                json!({"kind":"text","text":format!("entry-{n}")}),
            )
            .unwrap();
    }
    worker.journal.stop();
    let before = serde_json::to_value(&worker.state).unwrap();
    let (input, mut daemon) = UnixStream::pair().unwrap();
    let (events, inbox) = mpsc::channel();
    // Le worker ne consomme aucun événement, comme pendant un appel HTTP
    // en attente. Le vrai lecteur socket et le vrai relais restent actifs.
    let reader = spawn_reader(
        BufReader::new(input),
        events,
        worker.daemon_alive.clone(),
        worker.cancelled.clone(),
        worker.drained_key.clone(),
        worker.inbound_deliveries.clone(),
        "journal-blocked-worker",
        Some(worker.relay.subscription_control()),
    )
    .unwrap();
    writeln!(
        daemon,
        "{}",
        bridget_transport::protocol::encode(&DaemonToWrapper::Subscribe {
            subscription_id: "journal-eight".into(),
            agent: worker.agent_id.clone(),
            window: AttachWindow::Tail(8),
        })
        .unwrap()
    )
    .unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    let mut output = BufReader::new(peer);
    let mut seqs = std::collections::BTreeSet::new();
    let mut caught = false;
    let until = Instant::now() + Duration::from_secs(3);
    while Instant::now() < until {
        let mut line = String::new();
        output.read_line(&mut line).unwrap();
        match decode::<WrapperToDaemon>(&line).unwrap() {
            WrapperToDaemon::JournalFragment {
                subscription_id,
                seq,
                final_fragment: true,
                ..
            } if subscription_id == "journal-eight" => {
                seqs.insert(seq);
            }
            WrapperToDaemon::SnapshotCaughtUp {
                subscription_id, ..
            } if subscription_id == "journal-eight" => {
                caught = true;
                break;
            }
            WrapperToDaemon::AttachRejected { reason, .. } => {
                panic!("journal rejected: {reason:?}")
            }
            _ => {}
        }
    }
    assert!(caught);
    assert_eq!(seqs.len(), 8);
    assert!(inbox.try_recv().is_err());
    writeln!(
        daemon,
        "{}",
        bridget_transport::protocol::encode(&DaemonToWrapper::Unsubscribe {
            subscription_id: "journal-eight".into()
        })
        .unwrap()
    )
    .unwrap();
    writeln!(
        daemon,
        "{}",
        bridget_transport::protocol::encode(&DaemonToWrapper::Disconnect).unwrap()
    )
    .unwrap();
    let next = inbox.recv_timeout(Duration::from_secs(1)).unwrap();
    assert!(matches!(next,LinkEvent::Frame(frame) if matches!(*frame,DaemonToWrapper::Disconnect)));
    assert_eq!(serde_json::to_value(&worker.state).unwrap(), before);
    drop(daemon);
    reader.join().unwrap();
    worker.relay.shutdown();
}

#[test]
fn spec221_journal_saturation_never_blocks_reader_on_writer() {
    use bridget_transport::protocol::{AttachRefusal, AttachWindow};
    use std::io::Write;
    let (mut worker, _output) = worker099();
    let writer_lock = worker.writer.lock().unwrap();
    let mut accepted = 0;
    for n in 0..128 {
        if worker
            .relay
            .subscribe(format!("full-{n}"), AttachWindow::Tail(1))
            .is_err()
        {
            break;
        }
        accepted += 1;
    }
    // Le fil du relais peut retirer une commande puis bloquer sur le writer
    // tenu ici : une place se libère. Le laisser se poser, puis remplir de
    // nouveau jusqu'au refus, pour une file réellement saturée.
    thread::sleep(Duration::from_millis(200));
    for n in 128..256 {
        if worker
            .relay
            .subscribe(format!("full-{n}"), AttachWindow::Tail(1))
            .is_err()
        {
            break;
        }
        accepted += 1;
    }
    assert!(accepted > 0 && accepted < 256);
    let (input, mut daemon) = UnixStream::pair().unwrap();
    let (events, inbox) = mpsc::channel();
    let reader = spawn_reader(
        BufReader::new(input),
        events,
        worker.daemon_alive.clone(),
        worker.cancelled.clone(),
        worker.drained_key.clone(),
        worker.inbound_deliveries.clone(),
        "journal-saturated",
        Some(worker.relay.subscription_control()),
    )
    .unwrap();
    writeln!(
        daemon,
        "{}",
        bridget_transport::protocol::encode(&DaemonToWrapper::Subscribe {
            subscription_id: "rejected".into(),
            agent: worker.agent_id.clone(),
            window: AttachWindow::Tail(1),
        })
        .unwrap()
    )
    .unwrap();
    writeln!(
        daemon,
        "{}",
        bridget_transport::protocol::encode(&DaemonToWrapper::Disconnect).unwrap()
    )
    .unwrap();
    let first = inbox.recv_timeout(Duration::from_secs(1)).unwrap();
    assert!(
        matches!(
            &first,
            LinkEvent::JournalRejected {
                reason: AttachRefusal::CommandQueueSaturated,
                ..
            }
        ),
        "{}",
        match &first {
            LinkEvent::Frame(frame) => format!(
                "trame {}",
                bridget_transport::protocol::encode(frame.as_ref()).unwrap_or_default()
            ),
            LinkEvent::JournalRejected { .. } => "refus d'une autre cause".into(),
            _ => "autre événement".into(),
        }
    );
    assert!(
        matches!(inbox.recv_timeout(Duration::from_secs(1)).unwrap(),
            LinkEvent::Frame(frame) if matches!(*frame,DaemonToWrapper::Disconnect))
    );
    drop(daemon);
    reader.join().unwrap();
    drop(writer_lock);
    worker.journal.stop();
    worker.relay.shutdown();
}

#[test]
fn spec165_lecteur_invalide_le_drain_avant_la_file_worker() {
    use std::io::Write;

    let (reader_stream, mut peer) = UnixStream::pair().unwrap();
    let (events, inbox) = mpsc::channel();
    let drained_key = Arc::new(Mutex::new(Some("terminal".into())));
    let inbound_deliveries = Arc::new(AtomicUsize::new(0));
    let reader = spawn_reader(
        BufReader::new(reader_stream),
        events,
        Arc::new(AtomicBool::new(true)),
        Arc::new(Mutex::new(BTreeMap::new())),
        drained_key.clone(),
        inbound_deliveries.clone(),
        "course-drain",
        None,
    )
    .unwrap();
    let frame = DaemonToWrapper::Deliver {
        message: plain114("pendant-fermeture"),
        mid_turn_ok: None,
    };
    writeln!(
        peer,
        "{}",
        bridget_transport::protocol::encode(&frame).unwrap()
    )
    .unwrap();
    let event = inbox.recv_timeout(Duration::from_secs(1)).unwrap();
    assert!(drained_key.lock().recover_poison().is_none());
    assert_eq!(inbound_deliveries.load(Ordering::SeqCst), 1);

    let (mut worker, _worker_peer) = worker099();
    worker.last_key = "terminal".into();
    worker.journal_caught_up = true;
    worker.drained_key = drained_key.clone();
    worker.inbound_deliveries = inbound_deliveries.clone();
    worker.publish_drain_state(true);
    assert!(drained_key.lock().unwrap().is_none());
    // Simule l'écriture périmée qu'un publish commencé avant la lecture
    // réseau aurait pu faire après l'invalidation du lecteur.
    *drained_key.lock().unwrap() = Some("terminal".into());
    let LinkEvent::Frame(frame) = event else {
        panic!("une remise était attendue");
    };
    worker.handle_frame(*frame);
    assert_eq!(inbound_deliveries.load(Ordering::SeqCst), 0);
    assert!(
        drained_key.lock().unwrap().is_none(),
        "la mise en file efface elle-même toute publication périmée"
    );
    worker.publish_drain_state(true);
    assert!(drained_key.lock().unwrap().is_none());
    worker.journal.stop();
    worker.relay.shutdown();
    drop(peer);
    reader.join().unwrap();
}

#[test]
fn spec244_fermeture_conserve_la_remise_non_injectee() {
    let (mut worker, peer) = worker099();
    worker.handle_frame(idempotent113("fermeture"));
    assert_eq!(worker.queue.len(), 1);

    worker.handle_event(LinkEvent::Close);
    assert!(worker.queue.is_empty());
    worker.journal.stop();
    worker.relay.shutdown();

    let frames = frames113(peer);
    assert!(!saga_closed113(&frames, "d-fermeture"));
    assert!(rejection113(&frames, "fermeture").is_none());
    assert!(
        frames
            .iter()
            .any(|frame| matches!(frame, WrapperToDaemon::Unregister))
    );
}

#[test]
fn spec165_panique_worker_ferme_et_joint_le_lecteur_une_fois() {
    let (reader_stream, peer) = UnixStream::pair().unwrap();
    let shutdown = reader_stream.try_clone().unwrap();
    let (events, _inbox) = mpsc::channel();
    let reader = spawn_reader(
        BufReader::new(reader_stream),
        events.clone(),
        Arc::new(AtomicBool::new(true)),
        Arc::new(Mutex::new(BTreeMap::new())),
        Arc::new(Mutex::new(None)),
        Arc::new(AtomicUsize::new(0)),
        "panic-worker",
        None,
    )
    .unwrap();
    let worker = thread::spawn(|| panic!("panique attendue du worker"));
    let link = Link::from_parts(
        events,
        worker,
        reader,
        shutdown,
        Arc::new(Mutex::new(None)),
        Arc::new(AtomicUsize::new(0)),
        Duration::from_secs(1),
    );
    link.finish_dead();
    drop(peer);
}

fn pending(message_id: &str, anchor: Option<&str>) -> Pending {
    Pending {
        request_id: "req".to_string(),
        from: "alice".to_string(),
        message_id: message_id.to_string(),
        anchor_turn_id: anchor.map(str::to_string),
        dispatched_at: String::new(),
        attempts: 0,
        response: None,
        reply_requested: Some(true),
    }
}

#[test]
fn spec147_v2_correlation_uses_run_not_rank_and_waits_for_completion() {
    let mut detail = detail116(
        vec![
            message("u", "user", "question", Some("r"), false),
            message("steer", "user", "complément", Some("r"), false),
            message("a", "assistant", "bonne réponse", Some("r"), false),
            message("other", "assistant", "autre réponse", Some("next"), false),
        ],
        ("next", "completed"),
    );
    detail.activities.push(contract::Activity {
        id: "v2-run:r".into(),
        kind: "t3v2.run".into(),
        turn_id: Some("r".into()),
        payload: serde_json::json!({"status":"queued","userMessageId":"u"}),
    });
    let summary = summary(Some(("next", "completed")));
    for id in ["u", "steer"] {
        assert_eq!(
            correlate_sans_origine(&detail, &summary, &pending(id, None)),
            Correlation::Waiting
        );
    }
    detail.activities[0].payload["status"] = serde_json::json!("completed");
    for id in ["u", "steer"] {
        assert_eq!(
            correlate_sans_origine(&detail, &summary, &pending(id, None)),
            Correlation::Answered("bonne réponse".into())
        );
    }
    detail.messages.retain(|m| m.id != "a");
    detail.activities[0].payload["status"] = serde_json::json!("interrupted");
    assert_eq!(
        correlate_sans_origine(&detail, &summary, &pending("u", None)),
        Correlation::Silent
    );
    assert_eq!(
        correlate_sans_origine(&detail, &summary, &pending("absent", None)),
        Correlation::Missing
    );
}

/// Appariement historique, sans preuve directe d'origine.
fn correlate_sans_origine(
    detail: &ThreadDetail,
    summary: &ThreadSummary,
    pending: &Pending,
) -> Correlation {
    correlate(detail, summary, pending, &BTreeMap::new())
}

fn detail116(messages: Vec<Message>, latest: (&str, &str)) -> ThreadDetail {
    ThreadDetail {
        activities: Vec::new(),
        activities_available: false,
        messages,
        latest_turn: Some(LatestTurn {
            requested_at: None,
            turn_id: latest.0.to_string(),
            state: latest.1.to_string(),
            assistant_message_id: None,
        }),
    }
}

fn origins116(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(turn, origin)| (turn.to_string(), origin.to_string()))
        .collect()
}

/// Session 116, cas réel du 20/09 sur un fil Codex : notre premier message
/// a eu un tour qui n'a rien écrit, le suivant a répondu. Par rang, les deux
/// demandes étaient abandonnées ; par preuve directe, chacune a son issue.
#[test]
fn spec116_tour_silencieux_ne_fausse_plus_les_demandes_voisines() {
    let detail = detail116(
        vec![
            message("u1", "user", "question 1", None, false),
            message("u2", "user", "question 2", None, false),
            message("a2", "assistant", "réponse 2", Some("t2"), false),
        ],
        ("t2", "completed"),
    );
    let s = summary(Some(("t2", "completed")));
    assert_eq!(
        correlate_sans_origine(&detail, &s, &pending("u2", None)),
        Correlation::Ambiguous,
        "le rang seul échoue : deux messages, un seul tour écrit"
    );
    let origins = origins116(&[("t1", "u1"), ("t2", "u2")]);
    assert_eq!(
        correlate(&detail, &s, &pending("u1", None), &origins),
        Correlation::Silent,
        "le tour prouvé de u1 est clos sans texte"
    );
    assert_eq!(
        correlate(&detail, &s, &pending("u2", None), &origins),
        Correlation::Answered("réponse 2".to_string())
    );
}

/// Session 116, cas réel d'un fil Claude : après la réponse à notre
/// message, l'agent est réveillé par la fin d'un sous-agent et ouvre
/// d'autres tours sans message. Ils ne font pas partie de la réponse.
#[test]
fn spec116_reveils_en_arriere_plan_exclus_de_la_reponse() {
    let detail = detail116(
        vec![
            message("u1", "user", "demande", None, false),
            message("a1", "assistant", "ma réponse", Some("t1"), false),
            message("a1b", "assistant", "suite", Some("t1"), false),
            message(
                "r1",
                "assistant",
                "le sous-agent a fini",
                Some("tR1"),
                false,
            ),
            message("r2", "assistant", "autre réveil", Some("tR2"), false),
        ],
        ("tR2", "completed"),
    );
    let s = summary(Some(("tR2", "completed")));
    assert_eq!(
        correlate_sans_origine(&detail, &s, &pending("u1", None)),
        Correlation::Ambiguous,
        "le rang seul échoue : un message, trois tours"
    );
    assert_eq!(
        correlate(
            &detail,
            &s,
            &pending("u1", None),
            &origins116(&[("t1", "u1")])
        ),
        Correlation::Answered("ma réponse\n\nsuite".to_string())
    );
}

/// Objection de l'ancienne contre-revue, conservée : notre tour s'arrête
/// sans écrire, puis l'humain obtient sa réponse. Avec la preuve directe,
/// la réponse de l'humain n'est jamais attribuée à Bridget.
#[test]
fn spec116_preuve_directe_n_attribue_jamais_le_tour_de_l_humain() {
    let detail = detail116(
        vec![
            message("uB", "user", "nous", None, false),
            message("uH", "user", "humain", None, false),
            message("aH", "assistant", "réponse à l'humain", Some("tH"), false),
        ],
        ("tH", "completed"),
    );
    let s = summary(Some(("tH", "completed")));
    let origins = origins116(&[("tB", "uB"), ("tH", "uH")]);
    assert_eq!(
        correlate(&detail, &s, &pending("uB", None), &origins),
        Correlation::Silent
    );
}

/// Un tour prouvé encore en cours, ou dont un message s'écrit, attend.
#[test]
fn spec116_tour_prouve_en_cours_attend() {
    let origins = origins116(&[("t1", "u1")]);
    let en_cours = detail116(
        vec![message("u1", "user", "demande", None, false)],
        ("t1", "running"),
    );
    assert_eq!(
        correlate(
            &en_cours,
            &summary(Some(("t1", "running"))),
            &pending("u1", None),
            &origins
        ),
        Correlation::Waiting,
        "un tour ouvert n'est pas silencieux"
    );
    let en_ecriture = detail116(
        vec![
            message("u1", "user", "demande", None, false),
            message("a1", "assistant", "part", Some("t1"), true),
        ],
        ("t1", "completed"),
    );
    assert_eq!(
        correlate(
            &en_ecriture,
            &summary(Some(("t1", "completed"))),
            &pending("u1", None),
            &origins
        ),
        Correlation::Waiting
    );
    // Notre message hors de la page : la preuve seule ne suffit pas.
    assert_eq!(
        correlate(
            &en_ecriture,
            &summary(Some(("t1", "completed"))),
            &pending("absent", None),
            &origins116(&[("t1", "absent")])
        ),
        Correlation::Missing
    );
}

#[test]
fn spec098_correlation_par_rang_fifo_avec_message_humain_intercale() {
    // Ordre observé en sonde : user A (nous), user B (humain), assistant A, assistant B.
    let detail = ThreadDetail {
        activities: Vec::new(),
        activities_available: false,
        messages: vec![
            message("u0", "user", "avant", None, false),
            message("a0", "assistant", "réponse avant", Some("t0"), false),
            message("uA", "user", "nous", None, false),
            message("uB", "user", "humain", None, false),
            message("aA", "assistant", "pour nous", Some("tA"), false),
            message("aB", "assistant", "pour l'humain", Some("tB"), false),
        ],
        latest_turn: Some(LatestTurn {
            requested_at: None,
            turn_id: "tB".to_string(),
            state: "completed".to_string(),
            assistant_message_id: None,
        }),
    };
    let s = summary(Some(("tB", "completed")));
    assert_eq!(
        correlate_sans_origine(&detail, &s, &pending("uA", Some("t0"))),
        Correlation::Answered("pour nous".to_string())
    );
    assert_eq!(
        correlate_sans_origine(&detail, &s, &pending("uB", Some("t0"))),
        Correlation::Answered("pour l'humain".to_string())
    );
}

#[test]
fn spec098_correlation_attend_la_fin_du_tour() {
    let detail = ThreadDetail {
        activities: Vec::new(),
        activities_available: false,
        messages: vec![
            message("uA", "user", "nous", None, false),
            message("aA", "assistant", "partiel", Some("tA"), true),
        ],
        latest_turn: Some(LatestTurn {
            requested_at: None,
            turn_id: "tA".to_string(),
            state: "running".to_string(),
            assistant_message_id: None,
        }),
    };
    let s = summary(Some(("tA", "running")));
    assert_eq!(
        correlate_sans_origine(&detail, &s, &pending("uA", None)),
        Correlation::Waiting
    );
    let done = ThreadDetail {
        activities: Vec::new(),
        activities_available: false,
        messages: vec![
            message("uA", "user", "nous", None, false),
            message("aA", "assistant", "complet", Some("tA"), false),
        ],
        latest_turn: Some(LatestTurn {
            requested_at: None,
            turn_id: "tA".to_string(),
            state: "completed".to_string(),
            assistant_message_id: None,
        }),
    };
    assert_eq!(
        correlate(
            &done,
            &summary(Some(("tA", "completed"))),
            &pending("uA", None),
            &BTreeMap::new(),
        ),
        Correlation::Answered("complet".to_string())
    );
}

#[test]
fn spec098_correlation_sans_message_ni_tour() {
    let detail = ThreadDetail {
        activities: Vec::new(),
        activities_available: false,
        messages: vec![message("uA", "user", "nous", None, false)],
        latest_turn: None,
    };
    // Fil au travail : le tour peut encore naître, on patiente.
    let mut occupe = summary(None);
    occupe.session = Some(crate::t3code_contract::SessionSummary {
        provider_name: "claudeAgent".to_string(),
        provider_instance_id: None,
        status: "running".to_string(),
        active_turn_id: Some("tA".to_string()),
    });
    assert_eq!(
        correlate_sans_origine(&detail, &occupe, &pending("uA", None)),
        Correlation::Waiting
    );
    // Fil au repos sans aucun tour : plus rien ne viendra.
    assert_eq!(
        correlate_sans_origine(&detail, &summary(None), &pending("uA", None)),
        Correlation::Ambiguous
    );
    assert_eq!(
        correlate_sans_origine(&detail, &summary(None), &pending("uZ", None)),
        Correlation::Missing
    );
}

#[test]
fn spec098_tour_sans_reponse_n_attribue_pas_le_tour_du_voisin() {
    // Objection de contre-revue : notre tour est interrompu sans produire
    // de message assistant, puis l'humain obtient le sien. Un appariement
    // ordinal donnerait la réponse de l'humain à Bridget.
    let detail = ThreadDetail {
        activities: Vec::new(),
        activities_available: false,
        messages: vec![
            message("uB", "user", "nous", None, false),
            message("uH", "user", "humain", None, false),
            message("aH", "assistant", "réponse à l'humain", Some("tH"), false),
        ],
        latest_turn: Some(LatestTurn {
            requested_at: None,
            turn_id: "tH".to_string(),
            state: "completed".to_string(),
            assistant_message_id: None,
        }),
    };
    let s = summary(Some(("tH", "completed")));
    assert_eq!(
        correlate_sans_origine(&detail, &s, &pending("uB", None)),
        Correlation::Ambiguous,
        "la réponse de l'humain ne doit jamais être attribuée à Bridget"
    );
    // L'humain, lui, garde un appariement faux aussi : deux utilisateurs
    // pour un seul tour, aucun rang n'est prouvé.
    assert_eq!(
        correlate_sans_origine(&detail, &s, &pending("uH", None)),
        Correlation::Ambiguous
    );
}

#[test]
fn spec098_tour_supplementaire_sans_message_utilisateur_est_ambigu() {
    // Fan-out : t3code peut ouvrir un tour sans message utilisateur associé.
    let detail = ThreadDetail {
        activities: Vec::new(),
        activities_available: false,
        messages: vec![
            message("uA", "user", "nous", None, false),
            message("aX", "assistant", "sous-agent", Some("tX"), false),
            message("aA", "assistant", "réponse", Some("tA"), false),
        ],
        latest_turn: Some(LatestTurn {
            requested_at: None,
            turn_id: "tA".to_string(),
            state: "completed".to_string(),
            assistant_message_id: None,
        }),
    };
    assert_eq!(
        correlate(
            &detail,
            &summary(Some(("tA", "completed"))),
            &pending("uA", None),
            &BTreeMap::new(),
        ),
        Correlation::Ambiguous
    );
}

#[test]
fn spec098_journal_du_service_reste_hors_du_namespace() {
    // launchd crée ses fichiers de sortie en 0644 ; un seul fichier non
    // privé dans le namespace empêche le daemon Bridget de démarrer.
    let home = "/Users/essai";
    let log = service::log_path(home);
    let namespace =
        crate::environment::root_for_home(std::path::Path::new(home)).expect("racine du namespace");
    assert!(
        !log.starts_with(&namespace),
        "journal {} sous le namespace {}",
        log.display(),
        namespace.display()
    );
}

#[test]
fn spec098_identifiant_de_reponse_est_stable_par_demande() {
    // Un rejeu après panne ne doit pas produire une seconde réponse.
    assert_eq!(reply_id("demande-1"), reply_id("demande-1"));
    assert_ne!(reply_id("demande-1"), reply_id("demande-2"));
    assert!(reply_id("demande-1").starts_with("t3-"));
}

#[test]
fn spec098_identite_stable_et_v4_canonique() {
    let a = stable_uuid("thread-1");
    assert_eq!(a, stable_uuid("thread-1"));
    assert_ne!(a, stable_uuid("thread-2"));
    assert!(bridget_core::router::validate_agent_id(&a).is_ok(), "{a}");
}

#[test]
fn spec098_type_d_agent_depuis_le_fournisseur() {
    assert_eq!(agent_type_for("claudeAgent"), "claude");
    assert_eq!(agent_type_for("codex"), "codex");
    assert_eq!(agent_type_for("antigravity"), "gemini");
    assert_eq!(agent_type_for("Gemini"), "gemini");
    assert_eq!(agent_type_for("autre"), "autre");
}

#[test]
fn spec098_fil_range_n_est_pas_un_agent() {
    // t3code range les fils issus d'un import d'historique : les exposer
    // noierait l'annuaire sous des centaines de conversations passées.
    let mut range = summary(None);
    range.settled_override = Some("settled".to_string());
    assert!(!range.is_live());
    let mut repris = summary(None);
    repris.settled_override = Some("active".to_string());
    assert!(repris.is_live());
    assert!(summary(None).is_live());
}

#[test]
fn spec098_fil_neuf_sans_session_reste_joignable_par_son_modele() {
    // Forme réelle d'un fil créé et jamais démarré (t3code 0.0.41) :
    // `session` est nul, seul `modelSelection.instanceId` nomme le fournisseur.
    let mut neuf = summary(None);
    neuf.provider_instance_id = Some("claudeAgent".to_string());
    assert_eq!(thread_provider(&neuf).as_deref(), Some("claude"));
    // La session, quand elle existe, prime : elle atteste ce qui tourne.
    let mut demarre = summary(None);
    demarre.provider_instance_id = Some("claudeAgent".to_string());
    demarre.session = Some(crate::t3code_contract::SessionSummary {
        provider_name: "codex".to_string(),
        provider_instance_id: Some("codex".to_string()),
        status: "stopped".to_string(),
        active_turn_id: None,
    });
    assert_eq!(thread_provider(&demarre).as_deref(), Some("codex"));
    // Sans l'un ni l'autre, le fil n'est pas exposé plutôt que mal typé.
    assert_eq!(thread_provider(&summary(None)), None);
}

#[test]
fn spec098_etat_de_fil_borne_et_relu() {
    let dir = std::env::temp_dir().join(format!("t3code-098-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("répertoire de test");
    let path = dir.join("t.json");
    let mut state = ThreadState::default();
    for i in 0..(SEEN_BOUND + 10) {
        state.remember(&format!("m{i}"));
    }
    state.pending.push(pending("x", Some("t0")));
    state.save(&path).expect("save");
    let reloaded = ThreadState::load(&path);
    assert_eq!(reloaded.seen, state.seen);
    assert_eq!(reloaded.pending, state.pending);
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[test]
fn spec118_journal_en_retard_une_seule_annonce_sans_alternance() {
    use bridget_transport::protocol::ObservationKind as Kind;
    let (mut worker, peer) = worker099();
    worker.state.seeded = true;
    worker.state.observations_seeded = true;
    worker.observation_events = vec![Kind::TurnEnded];
    // Journal plus long que les tranches lues par ces lectures : jamais
    // rattrapé ici. Une seule ligne sans fin : rien à analyser.
    std::fs::write(
        worker.journal_dir.join("0000-00-00.jsonl"),
        "x".repeat(64 * 1024 * 1024),
    )
    .unwrap();
    let body = serde_json::json!({"thread":{
        "messages":[{"id":"u","role":"user","streaming":false,"createdAt":"date"}],
        "latestTurn":{"turnId":"t","state":"running","requestedAt":"date"},
        "activities":[]
    }})
    .to_string();
    for _ in 0..2 {
        let server = http_once099(&worker, 200, body.clone());
        worker.refresh(&summary(Some(("t", "running"))));
        assert!(server.join().unwrap().starts_with("GET "));
    }
    peer.set_read_timeout(Some(Duration::from_millis(300)))
        .unwrap();
    let mut reader = BufReader::new(peer);
    let mut announced = Vec::new();
    let mut line = String::new();
    while reader.read_line(&mut line).is_ok_and(|n| n > 0) {
        if let Ok(WrapperToDaemon::ObservationCapabilities { events, .. }) =
            decode::<WrapperToDaemon>(line.trim())
        {
            announced.push(events);
        }
        line.clear();
    }
    assert_eq!(
        announced,
        vec![Vec::<Kind>::new()],
        "une annonce, pas d'alternance"
    );
    worker.journal.stop();
    worker.relay.shutdown();
}

fn dated(id: &str, role: &str, at: &str, turn: Option<&str>) -> Message {
    let mut m = message(id, role, "texte", turn, false);
    m.created_at = Some(at.into());
    m
}

#[test]
fn spec121_tour_spontane_prouve_seulement_si_la_page_le_couvre() {
    let turn = |at: &str| LatestTurn {
        turn_id: "t1".into(),
        state: "completed".into(),
        assistant_message_id: None,
        requested_at: Some(at.into()),
    };
    let detail = ThreadDetail {
        messages: vec![
            dated("u", "user", "2026-09-25T01:00:00.000Z", None),
            dated("a", "assistant", "2026-09-25T01:00:05.000Z", Some("t0")),
        ],
        ..Default::default()
    };
    assert!(spontaneous_turn(&detail, &turn("2026-09-25T02:00:00.000Z")));
    assert!(
        !spontaneous_turn(&detail, &turn("2026-09-25T01:00:00.000Z")),
        "un message porte l'horodatage : origine ordinaire"
    );
    assert!(
        !spontaneous_turn(&detail, &turn("2026-09-25T00:00:00.000Z")),
        "page trop courte pour conclure"
    );
}

#[test]
fn spec121_fil_claude_observable_malgre_un_reveil_d_arriere_plan() {
    use bridget_transport::protocol::ObservationKind as Kind;
    let (mut worker, _peer) = worker099();
    worker.state.seeded = true;
    worker.state.observations_seeded = true;
    let server = http_once099(&worker, 200, serde_json::json!({"thread":{
            "messages":[
                {"id":"u","role":"user","streaming":false,"createdAt":"2026-09-25T01:00:00.000Z"},
                {"id":"a","role":"assistant","text":"fait","turnId":"t0","streaming":false,"createdAt":"2026-09-25T01:00:05.000Z"}
            ],
            "latestTurn":{"turnId":"t1","state":"completed","requestedAt":"2026-09-25T02:00:00.000Z"},
            "activities":[]
        }}).to_string());
    let mut summary = summary(Some(("t1", "completed")));
    summary.provider_instance_id = Some("claudeAgent".into());
    worker.refresh(&summary);
    assert!(server.join().unwrap().starts_with("GET "));
    assert!(worker.observation_events.contains(&Kind::TurnEnded));
    assert_eq!(
        worker.state.turn_origins.get("t1").map(String::as_str),
        Some("t3-turn:t1")
    );
    worker.journal.stop();
    let journal: String = std::fs::read_dir(&worker.journal_dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| std::fs::read_to_string(e.path()).unwrap_or_default())
        .collect();
    let end = journal
        .lines()
        .find(|l| l.contains(r#""event":"turn_end""#) && l.contains(r#""t3_turn_id":"t1""#))
        .expect("fin du tour spontané journalisée");
    assert!(
        end.contains("stop_reason"),
        "fin observable, pas une lacune : {end}"
    );
    worker.relay.shutdown();
}

#[test]
fn spec121_reponse_incertaine_marquee_et_bornee_au_message_suivant() {
    let detail = ThreadDetail {
        messages: vec![
            message("q", "user", "ta demande", None, false),
            message("r1", "assistant", "première partie", Some("t"), false),
            message("r2", "assistant", "seconde partie", Some("t"), false),
            message("q2", "user", "autre demande", None, false),
            message("r3", "assistant", "réponse à autrui", Some("t2"), false),
        ],
        ..Default::default()
    };
    let text = uncertain_answer(&detail, "q").unwrap();
    assert!(text.starts_with("[Bridget : appariement incertain."));
    assert!(text.contains("première partie\n\nseconde partie"));
    assert!(!text.contains("réponse à autrui"));
    assert_eq!(
        uncertain_answer(&detail, "q2")
            .as_deref()
            .map(|t| t.contains("réponse à autrui")),
        Some(true)
    );
    assert_eq!(uncertain_answer(&detail, "absent"), None);
    let muet = ThreadDetail {
        messages: vec![message("q", "user", "ta demande", None, false)],
        ..Default::default()
    };
    assert_eq!(uncertain_answer(&muet, "q"), None);
}

/// Le 25/09 à 07:49 : tour t1 clos, t2 lancé 0,6 s plus tard par un message,
/// entre deux lectures du pont. Rend (lacune signalée, ligne de fin de t1).
fn fin_de_tour_rapide(origin: Option<&str>, notification: bool) -> (bool, String) {
    let (mut worker, peer) = worker099();
    worker.state.seeded = true;
    worker.state.observations_seeded = true;
    if let Some(origin) = origin {
        worker.state.turn_origins.insert("t1".into(), origin.into());
    }
    if notification {
        worker.state.notification_messages.push("u1".into());
    }
    let server = http_once099(&worker, 200, serde_json::json!({"thread":{
            "messages":[
                {"id":"u1","role":"user","streaming":false,"createdAt":"2026-09-25T07:35:49.841Z"},
                {"id":"a1","role":"assistant","text":"fait","turnId":"t1","streaming":false,"createdAt":"2026-09-25T07:49:05.882Z"},
                {"id":"u2","role":"user","streaming":false,"createdAt":"2026-09-25T07:49:06.441Z"}
            ],
            "latestTurn":{"turnId":"t2","state":"running","requestedAt":"2026-09-25T07:49:06.441Z"},
            "activities":[]
        }}).to_string());
    worker.refresh(&summary(Some(("t2", "running"))));
    assert!(server.join().unwrap().starts_with("GET "));
    worker.journal.stop();
    peer.set_read_timeout(Some(Duration::from_millis(300)))
        .unwrap();
    let mut reader = BufReader::new(peer);
    let mut gap = false;
    let mut line = String::new();
    while reader.read_line(&mut line).is_ok_and(|n| n > 0) {
        gap |= matches!(
            decode::<WrapperToDaemon>(line.trim()),
            Ok(WrapperToDaemon::ObservationGap { .. })
        );
        line.clear();
    }
    let journal: String = std::fs::read_dir(&worker.journal_dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| std::fs::read_to_string(e.path()).unwrap_or_default())
        .collect();
    let end = journal
        .lines()
        .find(|l| l.contains(r#""event":"turn_end""#) && l.contains(r#""t3_turn_id":"t1""#))
        .expect("fin de t1 journalisée")
        .to_string();
    worker.relay.shutdown();
    (gap, end)
}

#[test]
fn spec125_tour_suivi_de_pres_fin_observable_si_origine_connue() {
    let (gap, end) = fin_de_tour_rapide(Some("u1"), false);
    assert!(!gap, "origine connue : pas de lacune");
    assert!(end.contains(r#""stop_reason":"ended""#), "{end}");
    assert!(end.contains(r#""message_id":"u1""#), "{end}");
    // Origine inconnue : la prudence d'avant reste la règle.
    let (gap, end) = fin_de_tour_rapide(None, false);
    assert!(gap, "origine inconnue : lacune signalée");
    assert!(!end.contains("stop_reason"), "{end}");
    // Tour provoqué par une notification : aucun fait observable.
    let (gap, end) = fin_de_tour_rapide(Some("u1"), true);
    assert!(!gap);
    assert!(
        end.contains(r#""message_id":"bridget-observation:t1""#),
        "{end}"
    );
}

#[test]
fn spec129_remise_pendant_un_tour_seulement_si_pilotable_et_sans_reponse() {
    assert!(deliverable_while_busy(Some("claude"), true));
    assert!(
        !deliverable_while_busy(Some("cursor"), true),
        "Cursor149 ne guide pas nativement"
    );
    assert!(
        deliverable_while_busy(Some("codex"), true),
        "CodexV2 sait guider le tour actif"
    );
    assert!(!deliverable_while_busy(None, true));
    assert!(
        !deliverable_while_busy(Some("claude"), false),
        "demande suivie, sollicitation ou notification en tête : attente"
    );
    // Un fil GLM est servi par le pilote Claude de T3.
    assert_eq!(agent_type_for("claudeAgent"), "claude");
}

#[test]
fn spec120_declaration_retablie_seulement_si_absente_et_serveur_vivant() {
    let base = std::env::temp_dir().join(format!("t3-spec120-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&base).unwrap();
    let file = contract::runtime_file(&base);
    let ours = format!(
        r#"{{"version":1,"pid":{},"host":"127.0.0.1","port":3773,"origin":"http://127.0.0.1:3773","startedAt":"2026-09-22T18:07:49.000Z"}}"#,
        std::process::id()
    );
    // Notre serveur arrêté : le fichier absent reste absent.
    assert!(!restore_declaration(&base, &ours, || false));
    assert!(!file.exists());
    // Effacé par l'autre application, notre serveur vivant : rétabli tel quel.
    assert!(restore_declaration(&base, &ours, || true));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), ours);
    // Une déclaration valide, même étrangère, n'est jamais écrasée.
    let foreign = ours.replace("3773", "3774");
    std::fs::write(&file, &foreign).unwrap();
    assert!(!restore_declaration(&base, &ours, || true));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), foreign);
    // Une déclaration d'un serveur mort est remplacée.
    std::fs::write(
        &file,
        ours.replace(&std::process::id().to_string(), "999999"),
    )
    .unwrap();
    assert!(restore_declaration(&base, &ours, || true));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), ours);
    let _ = std::fs::remove_dir_all(base);
}

#[test]
fn spec118_oubli_limite_a_ce_que_t3_ne_montre_plus() {
    let mut state = ThreadState::default();
    for i in 0..(SEEN_BOUND + 10) {
        state.remember(&format!("m{i}"));
    }
    state.remember("t3-activity:a1:0");
    state.remember("bridget-observation:a1:1");
    state.remember("t3-activity:disparue:0");
    let detail = ThreadDetail {
        messages: vec![message("m0", "user", "le plus ancien", None, false)],
        activities: vec![contract::Activity {
            id: "a1".into(),
            kind: "tool.completed".into(),
            turn_id: None,
            payload: serde_json::Value::Null,
        }],
        ..Default::default()
    };
    state.forget_invisible(&detail);
    assert_eq!(
        state.seen,
        ["m0", "t3-activity:a1:0", "bridget-observation:a1:1"],
        "le plus ancien reste s'il est encore montré ; ce qui a disparu est oublié"
    );
    // Sous la borne, rien n'est oublié, même invisible.
    let mut small = ThreadState::default();
    small.remember("absent");
    small.forget_invisible(&ThreadDetail::default());
    assert_eq!(small.seen, ["absent"]);
}

#[test]
fn spec118_un_grand_fil_n_est_recopie_qu_une_fois() {
    let (mut worker, _peer) = worker099();
    worker.state.seeded = true;
    let count = SEEN_BOUND + 50;
    let detail = ThreadDetail {
        messages: (0..count)
            .map(|i| message(&format!("m{i}"), "assistant", "texte", Some("t"), false))
            .collect(),
        ..Default::default()
    };
    // Plusieurs lectures successives, chacune après confirmation complète.
    for _ in 0..4 {
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            worker.project_journal(&detail, &summary(None));
            if worker.journal_inflight.is_empty() && worker.state.seen.len() >= count {
                break;
            }
            assert!(Instant::now() < deadline, "journal jamais confirmé");
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    let copies: usize = std::fs::read_dir(&worker.journal_dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| {
            std::fs::read_to_string(e.path())
                .unwrap_or_default()
                .lines()
                .filter(|line| line.contains(r#""event":"update""#))
                .count()
        })
        .sum();
    assert_eq!(copies, count, "chaque message écrit une seule fois");
    worker.journal.stop();
    worker.relay.shutdown();
}

// --- Session 149 : état de fil à nom borné (identifiants T3 très longs) ---

fn paths149() -> Paths {
    let root = std::env::temp_dir().join(format!("t3-149-{}", uuid::Uuid::new_v4().simple()));
    let paths = Paths::at(root.clone(), root.join("b.sock"));
    paths.ensure().unwrap();
    paths
}

#[allow(clippy::field_reassign_with_default)] // état hérité reconstruit champ par champ pour le test
fn legacy_state149() -> ThreadState {
    let mut state = ThreadState::default();
    state.seen = vec!["m1".into(), "m2".into()];
    state.ended_turns = vec!["t1".into()];
    state.seeded = true;
    state.observations_seeded = true;
    state.seen_activities = vec!["a1".into()];
    state.notification_messages = vec!["n1".into()];
    state.turn_origins.insert("t1".into(), "origine".into());
    let mut waiting = pending("m2", Some("t1"));
    waiting.request_id = "req-149".into();
    waiting.attempts = 3;
    waiting.response = Some("réponse préparée".into());
    state.pending = vec![waiting, pending("m3", None)];
    state
}

#[test]
fn spec149_state_nom_borne_deterministe_et_ecriture_privee() {
    use std::os::unix::fs::PermissionsExt;
    let paths = paths149();
    let ids = [
        "x".repeat(343),
        "y".repeat(341),
        "界".repeat(114),
        "thread:delegated-task:command%3Amcp%3Ax".to_string(),
        "../../etc/passwd".to_string(),
        "/etc/passwd".to_string(),
        "a\\b".to_string(),
    ];
    let folder = paths.threads().join("thread-states-v1");
    let mut names = HashSet::new();
    for id in &ids {
        let path = paths.thread_state(id);
        assert_eq!(path, paths.thread_state(id), "déterministe");
        assert_eq!(path.parent().unwrap(), folder, "dossier fixe : {id:?}");
        let name = path.file_name().unwrap().to_str().unwrap();
        assert_eq!(name, format!("{}.json", stable_uuid(id)));
        assert_eq!(name.len(), 41);
        assert!(names.insert(name.to_string()), "noms distincts");
        let mut state = ThreadState::default();
        state.remember(id);
        state.save(&path).expect("écriture réelle du nom borné");
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(paths.load_thread_state(id), state, "rechargement");
    }
    assert_eq!(
        std::fs::read_dir(&folder).unwrap().count(),
        ids.len(),
        "aucun résidu"
    );
    assert_eq!(std::fs::read_dir(paths.threads()).unwrap().count(), 1);
    assert_eq!(
        std::fs::read_dir(&paths.dir).unwrap().count(),
        1,
        "rien hors de threads/"
    );
}

#[test]
fn spec149_state_legacy_migre_exactement_puis_ecrit_seulement_le_nouveau() {
    let paths = paths149();
    let id = "thread-legacy-149";
    let legacy = paths.threads().join(format!("{id}.json"));
    let original = legacy_state149();
    original.save(&legacy).unwrap();
    let legacy_bytes = std::fs::read(&legacy).unwrap();
    let loaded = paths.load_thread_state(id);
    assert_eq!(
        loaded, original,
        "seen, ended_turns, pending et origines intacts"
    );
    assert_eq!(loaded.pending[0].request_id, "req-149");
    assert_eq!(
        loaded.pending[0].response.as_deref(),
        Some("réponse préparée")
    );
    assert_eq!(loaded.pending[0].reply_requested, Some(true));
    assert!(!paths.thread_state(id).exists(), "la lecture n'écrit rien");
    let mut next = loaded;
    next.remember("m-nouveau");
    next.save(&paths.thread_state(id)).unwrap();
    assert_eq!(
        std::fs::read(&legacy).unwrap(),
        legacy_bytes,
        "legacy intact"
    );
    assert_eq!(
        paths.load_thread_state(id),
        next,
        "le nouveau fichier prime"
    );
}

#[test]
fn spec149_state_nouveau_prime_et_invalide_ne_ressuscite_pas_legacy() {
    let paths = paths149();
    let id = "thread-prime-149";
    legacy_state149()
        .save(&paths.threads().join(format!("{id}.json")))
        .unwrap();
    let mut fresh = ThreadState::default();
    fresh.remember("seulement-nouveau");
    fresh.save(&paths.thread_state(id)).unwrap();
    assert_eq!(paths.load_thread_state(id), fresh);
    std::fs::write(paths.thread_state(id), "{ pas du json").unwrap();
    assert_eq!(
        paths.load_thread_state(id),
        ThreadState::default(),
        "invalide"
    );
    std::fs::remove_file(paths.thread_state(id)).unwrap();
    std::os::unix::fs::symlink("cible-absente", paths.thread_state(id)).unwrap();
    assert_eq!(
        paths.load_thread_state(id),
        ThreadState::default(),
        "lien cassé"
    );
}

#[test]
fn spec149_state_legacy_refuse_chemins_hors_dossier_et_noms_trop_longs() {
    let paths = paths149();
    let mut leak = ThreadState::default();
    leak.remember("fuite");
    leak.save(&paths.dir.join("outside.json")).unwrap();
    leak.save(&paths.root.join("outside.json")).unwrap();
    let absolute = format!("{}/outside", paths.root.display());
    for id in [
        "../outside".to_string(),
        "..\\outside".to_string(),
        absolute,
        "a\nb".to_string(),
        "a\u{202e}b".to_string(),
        String::new(),
        "z".repeat(251),
    ] {
        if !id.contains(['/', '\\']) && !id.is_empty() && id.len() <= 250 {
            leak.save(&paths.threads().join(format!("{id}.json")))
                .unwrap();
        }
        assert_eq!(
            paths.load_thread_state(&id),
            ThreadState::default(),
            "{id:?}"
        );
    }
    let limit = "l".repeat(250);
    // Écriture directe : l'ancien code n'avait pas le suffixe du fichier atomique.
    let text = serde_json::to_string(&leak).unwrap();
    std::fs::write(paths.threads().join(format!("{limit}.json")), text).unwrap();
    assert_eq!(
        paths.load_thread_state(&limit),
        leak,
        "borne 250 octets incluse"
    );
}

#[test]
fn spec149_state_link_worker_garde_l_id_original_et_ecrit_le_nouveau_chemin() {
    let paths = paths149();
    let session = Arc::new(Session::new(
        &paths,
        ServerRuntime {
            port: 1,
            pid: std::process::id(),
        },
        TokenFile {
            installation_id: "test".into(),
            label: "test".into(),
            session_id: "test".into(),
            token: "synthetic".into(),
            expires_at: String::new(),
            issued_at: String::new(),
        },
    ));
    for (id, migrated) in [
        ("w".repeat(343), false),
        ("fil-ancien-149".to_string(), true),
    ] {
        let legacy = paths.threads().join(format!("{id}.json"));
        if migrated {
            legacy_state149().save(&legacy).unwrap();
        }
        let mut thread = summary(None);
        thread.id = id.clone();
        let (stream, _peer) = UnixStream::pair().unwrap();
        let mut worker = LinkWorker::new(
            &paths,
            session.clone(),
            &thread,
            stable_uuid(&id),
            stable_uuid(&format!("instance:{id}")),
            BufWriter::new(stream.try_clone().unwrap()),
            stream,
        )
        .unwrap();
        assert_eq!(worker.thread_id, id);
        assert_eq!(worker.state_path, paths.thread_state(&id));
        assert_eq!(worker.state == legacy_state149(), migrated);
        worker.state.remember("après-reprise");
        worker
            .state
            .save(&worker.state_path)
            .expect("save nouveau chemin");
        assert_eq!(ThreadState::load(&worker.state_path), worker.state);
        assert_eq!(legacy.exists(), migrated, "aucun legacy créé");
        worker.journal.stop();
        worker.relay.shutdown();
    }
}
#[test]
fn mobile177_tool_progress_obeys_the_real_journal_vocabulary() {
    let (mut worker, _peer) = worker099();
    let activity = contract::Activity {
        id: "a".into(),
        kind: "tool.progress".into(),
        turn_id: Some("r".into()),
        payload: serde_json::json!({"itemType":"dynamic_tool","status":"running","tool":"Read"}),
    };
    let payload = activity_payloads(&activity, "codex", Path::new("/tmp")).remove(0);
    assert_eq!(payload["kind"], "tool");
    worker
        .journal
        .enqueue("update", Some("tool-progress-fixture"), payload)
        .unwrap();
    worker.journal.stop();
    worker.relay.shutdown();
}

#[test]
fn mobile177_t3_optional_guidance_waits_then_delivers_same_message() {
    for (v2, provider) in [(true, "cursor"), (false, "claude")] {
        let (mut worker, peer) = worker099();
        worker
            .session
            .inner
            .lock()
            .unwrap()
            .client
            .set_test_protocol(v2);
        let mut frame = idempotent113("mobile-optional");
        if let DaemonToWrapper::DeliverIdempotent { message, .. } = &mut frame {
            message.reply = false;
            message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
            message.references.push("steer-or-queue".into());
        }
        worker.handle_frame(frame);
        let mut thread = summary(Some(("active", "running")));
        thread.provider_instance_id = Some(provider.into());
        thread.session = Some(contract::SessionSummary {
            provider_name: provider.into(),
            provider_instance_id: None,
            status: "running".into(),
            active_turn_id: Some("active".into()),
        });
        assert_eq!(worker.next_delivery_index(&thread), None);
        assert_eq!(worker.queue.len(), 1);
        assert_eq!(delivery_message(&worker.queue[0].0).id, "mobile-optional");
        assert!(delivery_message(&worker.queue[0].0).intent.is_none());
        assert!(!saga_closed113(&frames113(peer), "d-mobile-optional"));
        thread.session.as_mut().unwrap().active_turn_id = None;
        assert_eq!(worker.next_delivery_index(&thread), Some(0));
        worker.journal.stop();
        worker.relay.shutdown();
    }
    let (mut worker, _peer) = worker099();
    worker
        .session
        .inner
        .lock()
        .unwrap()
        .client
        .set_test_protocol(true);
    let mut message = plain114("live-claude");
    message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
    message.references.push("steer-or-queue".into());
    worker.handle_frame(DaemonToWrapper::Deliver {
        message,
        mid_turn_ok: None,
    });
    let mut thread = summary(None);
    thread.provider_instance_id = Some("claude".into());
    assert_eq!(worker.next_delivery_index(&thread), Some(0));
    assert_eq!(
        delivery_message(&worker.queue[0].0).intent,
        Some(bridget_core::MessageIntent::SteerCurrent)
    );
    worker.journal.stop();
    worker.relay.shutdown();
}

#[test]
fn spec248_compte_rendu_codex_v2_pendant_tour_sans_reponse_ni_redemarrage() {
    for (v2, allowed, reply, expected) in [
        (true, Some(true), false, Some(0)),
        (true, None, false, None),
        (false, Some(true), false, None),
        (true, Some(true), true, None),
    ] {
        let (mut worker, _peer) = worker099();
        worker
            .session
            .inner
            .lock()
            .unwrap()
            .client
            .set_test_protocol(v2);
        let mut message = plain114("compte-rendu248");
        message.reply = reply;
        worker.handle_frame(DaemonToWrapper::Deliver {
            message,
            mid_turn_ok: allowed,
        });
        let mut thread = summary(Some(("actif248", "running")));
        thread.provider_instance_id = Some("codex".into());
        thread.session = Some(contract::SessionSummary {
            provider_name: "codex".into(),
            provider_instance_id: None,
            status: "running".into(),
            active_turn_id: Some("actif248".into()),
        });
        assert_eq!(
            worker.next_delivery_index(&thread),
            expected,
            "v2={v2},allowed={allowed:?},reply={reply}"
        );
        assert_eq!(worker.queue.len(), 1, "aucune remise écartée ni doublée");
        if expected.is_some() {
            let message = delivery_message(&worker.queue[0].0).clone();
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            {
                let mut inner = worker.session.lock();
                inner.runtime.port = listener.local_addr().unwrap().port();
                inner.client = Client::for_base_url(
                    &format!("http://127.0.0.1:{}", inner.runtime.port),
                    "fixture",
                );
                inner.client.set_test_protocol(true);
            }
            let server = std::thread::spawn(move || {
                crate::t3code_contract_v2::tests::rpc_reply(
                    &listener,
                    "orchestration.dispatchCommand",
                    Some(json!({"sequence":1})),
                )
            });
            worker
                .dispatch_batch(&[message], &thread, Some("steer"))
                .unwrap();
            let request = server.join().unwrap();
            assert_eq!(request["deliveryIntent"], "steer");
            assert_eq!(request["commandId"], "compte-rendu248");
        }
        worker.journal.stop();
        worker.relay.shutdown();
    }
}

#[test]
fn mobile180_t3_codex_explicit_guidance_keeps_intent_while_busy() {
    let (mut worker, _peer) = worker099();
    worker
        .session
        .inner
        .lock()
        .unwrap()
        .client
        .set_test_protocol(true);
    let mut frame = idempotent113("mobile180-codex");
    if let DaemonToWrapper::DeliverIdempotent { message, .. } = &mut frame {
        message.reply = false;
        message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
        message.references.push("steer-or-queue".into());
    }
    worker.handle_frame(frame);
    let mut thread = summary(Some(("active", "running")));
    thread.provider_instance_id = Some("codex".into());
    thread.session = Some(contract::SessionSummary {
        provider_name: "codex".into(),
        provider_instance_id: None,
        status: "running".into(),
        active_turn_id: Some("active".into()),
    });
    assert_eq!(worker.next_delivery_index(&thread), Some(0));
    assert_eq!(
        delivery_message(&worker.queue[0].0).intent,
        Some(bridget_core::MessageIntent::SteerCurrent)
    );
    assert!(
        deliverable_while_busy(Some("codex"), true),
        "comptes rendus autorisés pendant le tourV2"
    );
    worker.journal.stop();
    worker.relay.shutdown();
}

#[test]
fn guidance156_t3_steer_fournisseur_hors_liste_refuse() {
    // Cursor149 n'a pas activeSteering : son fallbackRestart ne prouve pas
    // un guidage natif. Aucun steer natif n'est envoyé à Cursor ou inconnu.
    for provider in [Some("cursor"), None] {
        let (mut worker, peer) = worker099();
        worker
            .session
            .inner
            .lock()
            .unwrap()
            .client
            .set_test_protocol(true);
        let mut frame = idempotent113("guide-refuse");
        if let DaemonToWrapper::DeliverIdempotent { message, .. } = &mut frame {
            message.reply = false;
            message.intent = Some(bridget_core::MessageIntent::SteerCurrent);
        }
        worker.handle_frame(frame);
        let mut thread = summary(None);
        thread.provider_instance_id = provider.map(str::to_string);
        assert_eq!(worker.next_delivery_index(&thread), None);
        assert!(worker.queue.is_empty(), "le guidage refusé sort de la file");
        let frames = frames113(peer);
        assert!(saga_closed113(&frames, "d-guide-refuse"));
        assert_eq!(
            rejection113(&frames, "guide-refuse"),
            Some("steer_unavailable")
        );
        worker.journal.stop();
        worker.relay.shutdown();
    }
    // Même oracle de capacité : V1 ne sait encoder aucune intention
    // explicite. Le refus se produit avant tout HTTP fournisseur.
    for intent in [
        bridget_core::MessageIntent::SteerCurrent,
        bridget_core::MessageIntent::InterruptAndStart,
    ] {
        let (mut worker, peer) = worker099();
        worker
            .session
            .inner
            .lock()
            .unwrap()
            .client
            .set_test_protocol(false);
        let mut frame = idempotent113("guide-v1");
        if let DaemonToWrapper::DeliverIdempotent { message, .. } = &mut frame {
            message.reply = false;
            message.intent = Some(intent);
        }
        worker.handle_frame(frame);
        let mut thread = summary(None);
        thread.provider_instance_id = Some("claude".into());
        assert_eq!(worker.next_delivery_index(&thread), None);
        let frames = frames113(peer);
        assert!(saga_closed113(&frames, "d-guide-v1"));
        let reason = rejection113(&frames, "guide-v1").unwrap();
        assert!(reason == "steer_unavailable" || reason.contains("deliveryIntent"));
        assert!(worker.state.pending.is_empty());
        worker.journal.stop();
        worker.relay.shutdown();
    }
}

#[test]
fn guidance156_t3_guidage_derriere_tete_bloquee() {
    for intent in [
        bridget_core::MessageIntent::SteerCurrent,
        bridget_core::MessageIntent::InterruptAndStart,
    ] {
        let (mut worker, _peer) = worker099();
        worker
            .session
            .inner
            .lock()
            .unwrap()
            .client
            .set_test_protocol(true);
        let mut waiting = plain114("suivie-en-tete");
        waiting.reply = true;
        worker.handle_frame(DaemonToWrapper::Deliver {
            message: waiting,
            mid_turn_ok: Some(true),
        });
        let mut guidance = plain114("guidage");
        guidance.intent = Some(intent);
        worker.handle_frame(DaemonToWrapper::Deliver {
            message: guidance,
            mid_turn_ok: None,
        });
        let mut thread = summary(Some(("tour-actif", "running")));
        thread.session = Some(contract::SessionSummary {
            provider_name: "claude".into(),
            provider_instance_id: None,
            status: "running".into(),
            active_turn_id: Some("tour-actif".into()),
        });
        assert_eq!(
            worker.next_delivery_index(&thread),
            Some(1),
            "le guidage passe la tête bloquée"
        );
        assert_eq!(delivery_message(&worker.queue[0].0).id, "suivie-en-tete");
        assert_eq!(delivery_message(&worker.queue[1].0).id, "guidage");
        worker.journal.stop();
        worker.relay.shutdown();
    }
}

#[test]
fn spec253_t3_queued_replay_keeps_link_and_first_deadline() {
    let (mut worker, _peer) = worker099();
    let mut message = plain114("replay253");
    message.deadline_at = Some(100);
    let original = DaemonToWrapper::DeliverIdempotent {
        delivery_id: "delivery253".into(),
        recipient_instance_id: stable_uuid("instance:test"),
        delivery_generation: 1,
        expires_at: 999,
        message,
        execution: None,
        mid_turn_ok: Some(false),
    };
    worker.enqueue_delivery(original.clone());
    let mut replay = original.clone();
    if let DaemonToWrapper::DeliverIdempotent {
        message,
        mid_turn_ok,
        ..
    } = &mut replay
    {
        message.deadline_at = Some(200);
        message.from_display_name = Some("titre".into());
        *mid_turn_ok = Some(true);
    }
    worker.enqueue_delivery(replay);
    assert!(worker.daemon_alive.load(Ordering::SeqCst));
    assert_eq!(worker.queue.len(), 1);
    assert!(
        matches!(&worker.queue[0].0,DaemonToWrapper::DeliverIdempotent {message,mid_turn_ok:Some(false),..} if message.deadline_at==Some(100))
    );
    worker.enqueue_delivery(DaemonToWrapper::Deliver {
        message: plain114("neighbor253"),
        mid_turn_ok: None,
    });
    assert_eq!(worker.queue.len(), 2);
    if let DaemonToWrapper::DeliverIdempotent { mut message, .. } = original {
        message.body = "forged".into();
        worker.enqueue_delivery(DaemonToWrapper::DeliverIdempotent {
            delivery_id: "delivery253".into(),
            recipient_instance_id: stable_uuid("instance:test"),
            delivery_generation: 1,
            expires_at: 999,
            message,
            execution: None,
            mid_turn_ok: Some(false),
        });
    }
    assert!(!worker.daemon_alive.load(Ordering::SeqCst));
    worker.journal.stop();
    worker.relay.shutdown();
}

#[test]
fn guidance156_t3_guidage_hors_lot() {
    for intent in [
        bridget_core::MessageIntent::SteerCurrent,
        bridget_core::MessageIntent::InterruptAndStart,
    ] {
        for guidance_first in [true, false] {
            let (mut worker, _peer) = worker099();
            let mut guidance = plain114("guidage");
            guidance.intent = Some(intent);
            let mut messages = vec![guidance, plain114("ordinaire")];
            if !guidance_first {
                messages.reverse();
            }
            for message in messages {
                worker.handle_frame(DaemonToWrapper::Deliver {
                    message,
                    mid_turn_ok: Some(true),
                });
            }
            assert_eq!(worker.batchable_prefix(), 1, "aucun guidage dans un lot");
            worker.journal.stop();
            worker.relay.shutdown();
        }
    }
}

#[test]
fn guidance156_t3_groupable_claude_inchange() {
    // Le daemon fournit le droit. Le pont n'en déduit rien depuis from
    // ou origin ; l'absence du flag reste false sur les deux trames.
    for idempotent in [false, true] {
        for allowed in [None, Some(false), Some(true)] {
            let (mut worker, _peer) = worker099();
            worker
                .session
                .inner
                .lock()
                .unwrap()
                .client
                .set_test_protocol(true);
            let frame = if idempotent {
                let mut frame = idempotent113("ordinaire");
                if let DaemonToWrapper::DeliverIdempotent {
                    message,
                    mid_turn_ok,
                    ..
                } = &mut frame
                {
                    message.reply = false;
                    *mid_turn_ok = allowed;
                }
                frame
            } else {
                DaemonToWrapper::Deliver {
                    message: plain114("ordinaire"),
                    mid_turn_ok: allowed,
                }
            };
            worker.handle_frame(frame);
            let mut thread = summary(Some(("tour", "running")));
            thread.session = Some(contract::SessionSummary {
                provider_name: "claude".into(),
                provider_instance_id: None,
                status: "running".into(),
                active_turn_id: Some("tour".into()),
            });
            assert_eq!(
                worker.next_delivery_index(&thread),
                (allowed == Some(true)).then_some(0)
            );
            assert!(
                groupable(delivery_message(&worker.queue[0].0)),
                "groupable129 conservé"
            );
            thread.session.as_mut().unwrap().provider_name = "cursor".into();
            assert_eq!(
                worker.next_delivery_index(&thread),
                None,
                "Cursor149 attend malgré un flag vrai"
            );
            worker.journal.stop();
            worker.relay.shutdown();
        }
    }
    let (mut worker, _peer) = worker099();
    for id in ["a", "b"] {
        worker.handle_frame(DaemonToWrapper::Deliver {
            message: plain114(id),
            mid_turn_ok: Some(true),
        });
    }
    assert_eq!(
        worker.batchable_prefix(),
        2,
        "FIFO et regroupement ordinaire129 conservés"
    );
    worker.journal.stop();
    worker.relay.shutdown();
}

#[test]
fn spec279_expired_queued_request_names_budget_and_scope_without_injecting() {
    let (mut worker, peer) = worker099();
    let mut frame = idempotent113("budget279");
    if let DaemonToWrapper::DeliverIdempotent { message, .. } = &mut frame {
        message.reply = true;
        message.reply_timeout = None;
    }
    worker.handle_frame(frame);
    worker.handle_frame(DaemonToWrapper::RequestList {
        requests: vec![RequestInfo {
            id: "budget279".into(),
            sender: "alice".into(),
            target: worker.agent_id.clone(),
            state: "open".into(),
            created_at: 0,
            deadline_at: 0,
            cancel_reason: None,
            deferred_reminder_level: None,
            deferred_reminder_at: None,
        }],
    });
    let (_tx, inbox) = mpsc::channel();
    worker.drive_queue(&inbox);
    assert!(worker.queue.is_empty());
    let frames = frames113(peer);
    let reason = rejection113(&frames, "budget279").unwrap();
    assert!(
        reason.contains("60 s")
            && reason.contains("attente en file")
            && reason.contains("ce pont ne l'a pas injectée")
    );
    assert!(reason.contains("propriétaire") && reason.contains("autres travaux"));
    assert!(saga_closed113(&frames, "d-budget279"));
}
