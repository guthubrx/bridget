#![cfg(feature = "test-support")]
//! Scénarios historiques sur le harnais isolé commun. Les trois oracles rename
//! vivent désormais dans core_089_identity_test : nom durable, route UUID stable.
#[path = "support/idempotent.rs"]
pub mod fixture;
use bridget_core::BridgetMessage;
use bridget_daemon::registry::AgentRegistry;
use bridget_transport::protocol::{decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use fixture::*;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::time::{Duration, Instant};

fn ack(peer: &mut Client, id: &str) {
    assert!(matches!(peer.receive(), DaemonToWrapper::Ack { id: actual } if actual == id));
}
fn request_states(root: &Path) -> Vec<bridget_transport::protocol::RequestInfo> {
    let mut peer = Client::connect(&socket(root));
    peer.send(WrapperToDaemon::ListRequests {
        sender: ACTOR.into(),
        limit: 100,
    });
    match peer.receive() {
        DaemonToWrapper::RequestList { requests } => requests,
        other => panic!("demandes attendues : {other:?}"),
    }
}
#[test]
fn test_two_agents_communicate() {
    let root = test_root("two-agents");
    let daemon = spawn_daemon(&root, None);
    let mut sender = register_agent_as(&socket(&root), ACTOR, "sender");
    let mut recipient = register_recipient(&socket(&root));
    let request = BridgetMessage::new(
        ACTOR,
        RECIPIENT,
        "Bonjour Claude, analyse ce fichier stp — été 🐙",
    );
    sender.send(WrapperToDaemon::Send(request.clone()));
    ack(&mut sender, &request.id);
    let received = match recipient.receive() {
        DaemonToWrapper::Deliver(message) => message,
        other => panic!("remise attendue : {other:?}"),
    };
    assert_eq!(received.id, request.id);
    assert_eq!(received.from, ACTOR);
    assert_eq!(received.to, RECIPIENT);
    assert_eq!(received.body.as_bytes(), request.body.as_bytes());
    drop((sender, recipient));
    daemon.stop();
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn test_agent_not_found() {
    let root = test_root("missing-agent");
    let daemon = spawn_daemon(&root, None);
    let mut sender = register_agent_as(&socket(&root), ACTOR, "sender");
    let message = BridgetMessage::new(ACTOR, RECIPIENT, "destinataire absent");
    sender.send(WrapperToDaemon::Send(message.clone()));
    assert!(matches!(sender.receive(), DaemonToWrapper::Nack { id, .. } if id == message.id));
    assert!(request_states(&root).is_empty());
    drop(sender);
    daemon.stop();
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn test_reply_from_ephemeral_cli_is_rejected() {
    let root = test_root("ephemeral-reply");
    let daemon = spawn_daemon(&root, None);
    let _recipient = register_recipient(&socket(&root));
    let mut cli = Client::connect(&socket(&root));
    cli.send(WrapperToDaemon::Register {
        identity_version: 2,
        agent_id: ACTOR.into(),
        agent_type: "cli".into(),
        host: None,
        transport: None,
        channel: None.into(),
        mode: None,
        location: None,
        os: None,
        instance_id: None,
        domain: None,
        turn_in_progress: false,
        journal_available: None,
    });
    assert!(matches!(cli.receive(), DaemonToWrapper::Registered { .. }));
    let mut request = BridgetMessage::new(ACTOR, RECIPIENT, "réponds-user");
    request.reply = true;
    cli.send(WrapperToDaemon::Send(request.clone()));
    assert!(matches!(cli.receive(), DaemonToWrapper::Nack { id, reason }
        if id == request.id && reason.contains("--reply requiert un agent Bridget connecté")));
    assert!(request_states(&root).is_empty());
    drop(cli);
    daemon.stop();
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn test_only_sender_can_cancel_request() {
    let root = test_root("cancel-owner");
    let daemon = spawn_daemon(&root, None);
    let mut sender = register_agent_as(&socket(&root), ACTOR, "shared-cli-mcp-instance");
    let mut recipient = register_recipient(&socket(&root));
    let mut intruder = register_agent_as(&socket(&root), MATRIX_AGENT, "intruder");
    let mut request = BridgetMessage::new(ACTOR, RECIPIENT, "travail");
    request.reply = true;
    sender.send(WrapperToDaemon::Send(request.clone()));
    ack(&mut sender, &request.id);
    assert!(matches!(recipient.receive(), DaemonToWrapper::Deliver(_)));
    // Même usurper le sender ne doit pas autoriser la connexion étrangère.
    for claimed_sender in [MATRIX_AGENT, ACTOR] {
        intruder.send(WrapperToDaemon::CancelRequest {
            id: request.id.clone(),
            sender: claimed_sender.into(),
            reason: None,
        });
        let refusal = intruder.receive();
        assert!(
            matches!(refusal, DaemonToWrapper::Nack { .. }),
            "sender={claimed_sender} : {refusal:?}"
        );
        let states = request_states(&root);
        assert_eq!(states.len(), 1);
        assert_eq!(states[0].state, "open");
    }
    // Couture réelle de la CLI : même principal auxiliaire que rename/MCP.
    let without_marker = run_linked_cli(&root, &["cancel", &request.id]);
    assert!(!without_marker.status.success());
    assert!(output_text(&without_marker).contains("identity_not_found"));
    assert_eq!(request_states(&root)[0].state, "open");
    let identity_file = root.join("state/cancel-agent-id");
    private_write(&identity_file, ACTOR).unwrap();
    let mut command = isolated_command(&root);
    command
        .args(["cancel", &request.id])
        .env("BRIDGET_AGENT_ID_FILE", identity_file)
        .env("BRIDGET_AGENT_INSTANCE_ID", "shared-cli-mcp-instance");
    let output = run_command(command);
    assert!(output.status.success(), "{}", output_text(&output));
    assert_eq!(request_states(&root)[0].state, "cancelled");
    drop((sender, recipient, intruder));
    daemon.stop();
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn test_request_cancellation_stops_reminders() {
    let root = test_root("cancel-reminders");
    let daemon = spawn_daemon(&root, None);
    let mut sender = register_agent_as(&socket(&root), ACTOR, "sender");
    let mut recipient = register_recipient(&socket(&root));
    let mut request = BridgetMessage::new(ACTOR, RECIPIENT, "travail devenu inutile");
    request.reply = true;
    request.reply_timeout = Some(6);
    sender.send(WrapperToDaemon::Send(request.clone()));
    ack(&mut sender, &request.id);
    assert!(
        matches!(recipient.receive(), DaemonToWrapper::Deliver(message) if message.id == request.id)
    );
    sender.send(WrapperToDaemon::CancelRequest {
        id: request.id.clone(),
        sender: ACTOR.into(),
        reason: Some("priorité changée".into()),
    });
    assert!(
        matches!(sender.receive(), DaemonToWrapper::RequestCancelled { state, .. } if state == "cancelled")
    );
    assert!(
        matches!(recipient.receive(), DaemonToWrapper::CancelDelivery { id, .. } if id == request.id)
    );
    // Une autre demande traverse réellement les paliers : pas un sleep-oracle.
    let mut sentinel = BridgetMessage::new(ACTOR, RECIPIENT, "sentinelle de rappel");
    sentinel.reply = true;
    sentinel.reply_timeout = Some(6);
    sender.send(WrapperToDaemon::Send(sentinel.clone()));
    ack(&mut sender, &sentinel.id);
    assert!(
        matches!(recipient.receive(), DaemonToWrapper::Deliver(message) if message.id == sentinel.id)
    );
    sender
        .reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_secs(12)))
        .unwrap();
    assert!(matches!(sender.receive(), DaemonToWrapper::Deliver(_)));
    let states = request_states(&root);
    assert_eq!(
        states.iter().find(|r| r.id == request.id).unwrap().state,
        "cancelled"
    );
    assert_eq!(
        states.iter().find(|r| r.id == sentinel.id).unwrap().state,
        "timed_out"
    );
    let store = fixture_store(&root.join("state/bridget.db")).unwrap();
    let events = store.guichet_coordination_events().unwrap();
    assert!(
        !events.is_empty(),
        "le timer a réellement produit un rappel"
    );
    assert!(
        events.iter().all(|event| event.request_id == sentinel.id),
        "rappel après annulation : {events:?}"
    );
    drop((store, sender, recipient));
    daemon.stop();
    std::fs::remove_dir_all(root).unwrap();
}
fn stdio_registry(root: &Path, counting: bool) -> AgentRegistry {
    let counter = root.join("prompt-count");
    let script = format!(
        r#"
read initialize
echo '{{"jsonrpc":"2.0","id":1,"result":{{"protocolVersion":1}}}}'
read session
echo '{{"jsonrpc":"2.0","id":2,"result":{{"sessionId":"stdio-ouvert"}}}}'
while IFS= read -r prompt; do
    printf x >> '{}'
    echo '{{"jsonrpc":"2.0","method":"session/update","params":{{"sessionId":"stdio-ouvert","update":{{"sessionUpdate":"agent_message_chunk","content":{{"type":"text","text":"fixture-response"}}}}}}}}'
    echo '{{"jsonrpc":"2.0","id":3,"result":{{"stopReason":"end_turn"}}}}'
    {}
done
"#,
        counter.display(),
        if counting { ":" } else { "break" }
    );
    let mut declared: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/registry/codex-claude.json")).unwrap();
    assert_eq!(declared["agents"].as_object().unwrap().len(), 2);
    assert!(declared["agents"].get("stdio-ouvert").is_none());
    // Type ajouté au registre, pas une branche codex/claude du wrapper.
    declared["agents"]["stdio-ouvert"] = serde_json::json!({
        "command":"/bin/sh", "args":["-c",script], "protocol":"acp",
        "permissions":"allow", "queue_capacity":2,"notify_timeout_secs":30
    });
    let path = root.join("registry.json");
    let json = serde_json::to_string(&declared).unwrap();
    private_write(&path, &json).unwrap();
    AgentRegistry::from_json(&json, path).unwrap()
}
#[test]
fn test_unknown_registry_type_completes_a_tracked_exchange() {
    let root = test_root("unknown-type");
    let registry = stdio_registry(&root, false);
    let daemon = spawn_daemon(&root, None);
    let mut sender = register_agent_as(&socket(&root), ACTOR, "sender");
    let wrapper = WrapperProcess::start(&root, &registry, ACP_AGENT);
    wait_for_registered_agent(&socket(&root), ACP_AGENT);
    let mut request = BridgetMessage::new(ACTOR, ACP_AGENT, "réponds via ACP");
    request.reply = true;
    sender.send(WrapperToDaemon::Send(request.clone()));
    ack(&mut sender, &request.id);
    let returned = match sender.receive() {
        DaemonToWrapper::Deliver(message) => message,
        other => panic!("réponse du wrapper réel attendue : {other:?}"),
    };
    assert_eq!(returned.in_reply_to.as_deref(), Some(request.id.as_str()));
    assert_eq!(returned.body, "fixture-response");
    assert_eq!(returned.from, ACP_AGENT);
    assert_eq!(request_states(&root)[0].state, "answered");
    wrapper.join().unwrap();
    assert_eq!(std::fs::read(root.join("prompt-count")).unwrap(), b"x");
    drop(sender);
    daemon.stop();
    std::fs::remove_dir_all(root).unwrap();
}
fn accept_wrapper(
    listener: &UnixListener,
) -> (BufReader<UnixStream>, BufWriter<UnixStream>, String) {
    listener.set_nonblocking(true).unwrap();
    let deadline = Instant::now() + Duration::from_secs(8);
    let stream = loop {
        match listener.accept() {
            Ok((stream, _)) => break stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline, "reconnexion wrapper absente");
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("accept : {error}"),
        }
    };
    stream.set_nonblocking(false).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut writer = BufWriter::new(stream.try_clone().unwrap());
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let instance = match decode(&line).unwrap() {
        WrapperToDaemon::Register {
            instance_id: Some(instance),
            agent_id,
            ..
        } => {
            assert_eq!(agent_id, ACP_AGENT);
            instance
        }
        other => panic!("Register attendu : {other:?}"),
    };
    writeln!(
        writer,
        "{}",
        encode(&DaemonToWrapper::Registered {
            credential: None,
            agent_id: ACP_AGENT.into()
        })
        .unwrap()
    )
    .unwrap();
    writer.flush().unwrap();
    (reader, writer, instance)
}
fn wait_ack(reader: &mut BufReader<UnixStream>) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        assert!(Instant::now() < deadline, "accusé du wrapper absent");
        reader
            .get_ref()
            .set_read_timeout(Some(deadline.saturating_duration_since(Instant::now())))
            .unwrap();
        let mut line = String::new();
        assert_ne!(reader.read_line(&mut line).unwrap(), 0);
        match decode(&line).unwrap() {
            WrapperToDaemon::DeliverAcked {
                delivery_id,
                delivery_generation,
            } => {
                assert_eq!(delivery_id, "delivery-reconnect");
                assert_eq!(delivery_generation, 71);
                return;
            }
            WrapperToDaemon::DeliveryRejected { reason, .. } => panic!("remise rejetée : {reason}"),
            _ => {}
        }
    }
}
#[test]
fn redelivery_idempotente_apres_reconnexion_n_injecte_qu_un_prompt() {
    let root = test_root("wrapper-redelivery");
    let registry = stdio_registry(&root, true);
    let listener = UnixListener::bind(socket(&root)).unwrap();
    let wrapper = WrapperProcess::start(&root, &registry, ACP_AGENT);
    let (mut reader, mut writer, instance) = accept_wrapper(&listener);
    let mut message = BridgetMessage::new(ACTOR, ACP_AGENT, "une seule injection");
    message.id = "idempotent-reconnect".into();
    let delivery = DaemonToWrapper::DeliverIdempotent {
        delivery_id: "delivery-reconnect".into(),
        recipient_instance_id: instance.clone(),
        delivery_generation: 71,
        expires_at: i64::MAX,
        message,
        execution: None,
    };
    let exact = encode(&delivery).unwrap();
    writeln!(writer, "{exact}").unwrap();
    writer.flush().unwrap();
    wait_ack(&mut reader);
    wait_for_counter(&root.join("prompt-count"), 1);
    drop((writer, reader));
    let (mut reader, mut writer, next_instance) = accept_wrapper(&listener);
    assert_eq!(next_instance, instance);
    writeln!(writer, "{exact}").unwrap();
    writer.flush().unwrap();
    wait_ack(&mut reader);
    writeln!(writer, "{}", encode(&DaemonToWrapper::Disconnect).unwrap()).unwrap();
    writer.flush().unwrap();
    wrapper.join().unwrap();
    // Compteur FINAL après arrêt. Muter la dédup produit xx au lieu de x ;
    // le fournisseur ne peut plus recevoir un prompt après l'assertion.
    assert_eq!(std::fs::read(root.join("prompt-count")).unwrap(), b"x");
    drop((reader, writer, listener));
    std::fs::remove_dir_all(root).unwrap();
}
