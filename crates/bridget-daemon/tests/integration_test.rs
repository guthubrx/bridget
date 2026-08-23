//! Test d'intégration : deux faux wrappers communiquent via le daemon bridget.
//!
//! Ce test lance le daemon en arrière-plan, connecte deux faux agents,
//! et vérifie qu'un message envoyé par l'un est bien reçu par l'autre.

use bridget_core::BridgetMessage;
use bridget_transport::protocol::{decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn unique_socket_path() -> PathBuf {
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    PathBuf::from(format!("/tmp/bridget-test-{}-{}.sock", pid, nanos))
}

struct FakeAgent {
    name: String,
    writer: BufWriter<UnixStream>,
    reader: BufReader<UnixStream>,
    received: Arc<Mutex<Vec<BridgetMessage>>>,
}

impl FakeAgent {
    fn connect(
        socket_path: &PathBuf,
        agent_type: &str,
        name: Option<&str>,
    ) -> Result<Self, String> {
        let stream = UnixStream::connect(socket_path).map_err(|e| e.to_string())?;
        let reader_stream = stream.try_clone().map_err(|e| e.to_string())?;
        let writer = BufWriter::new(stream);
        let reader = BufReader::new(reader_stream);

        let reg = WrapperToDaemon::Register {
            agent_type: agent_type.to_string(),
            name: name.map(|s| s.to_string()),
            host: Some("test-host".to_string()),
            transport: Some("unix".to_string()),
            os: Some("Linux".to_string()),
            instance_id: None,
            domain: None,
            turn_in_progress: false,
        };
        let reg_json = encode(&reg).map_err(|e| e.to_string())?;

        let mut agent = FakeAgent {
            name: String::new(),
            writer,
            reader,
            received: Arc::new(Mutex::new(vec![])),
        };

        writeln!(agent.writer, "{}", reg_json).map_err(|e| e.to_string())?;
        agent.writer.flush().map_err(|e| e.to_string())?;

        // Lire la réponse Registered
        let mut line = String::new();
        agent
            .reader
            .read_line(&mut line)
            .map_err(|e| e.to_string())?;
        let resp: DaemonToWrapper = decode(line.trim()).map_err(|e| e.to_string())?;
        match resp {
            DaemonToWrapper::Registered { name } => {
                agent.name = name;
                Ok(agent)
            }
            _ => Err("pas de Registered".to_string()),
        }
    }

    fn send(&mut self, to: &str, body: &str) -> Result<(), String> {
        let msg = BridgetMessage::new(&self.name, to, body);
        self.send_message(msg)
    }

    fn send_message(&mut self, msg: BridgetMessage) -> Result<(), String> {
        let wtd = WrapperToDaemon::Send(msg);
        let json = encode(&wtd).map_err(|e| e.to_string())?;
        writeln!(self.writer, "{}", json).map_err(|e| e.to_string())?;
        self.writer.flush().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn rename(&mut self, name: &str) -> Result<DaemonToWrapper, String> {
        let request = WrapperToDaemon::Rename {
            current_name: self.name.clone(),
            name: name.to_string(),
        };
        writeln!(
            self.writer,
            "{}",
            encode(&request).map_err(|e| e.to_string())?
        )
        .map_err(|e| e.to_string())?;
        self.writer.flush().map_err(|e| e.to_string())?;
        let response = self.read_response()?;
        if let DaemonToWrapper::Renamed { name, .. } = &response {
            self.name = name.clone();
        }
        Ok(response)
    }

    fn read_response(&mut self) -> Result<DaemonToWrapper, String> {
        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .map_err(|e| e.to_string())?;
        decode(line.trim()).map_err(|e| e.to_string())
    }

    /// Thread qui écoute les messages entrants en push.
    fn start_receiver(agent_arc: Arc<Mutex<FakeAgent>>) {
        thread::spawn(move || loop {
            let mut line = String::new();
            {
                let mut agent = agent_arc.lock().unwrap();
                match agent.reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(_) => break,
                }
            }
            if let Ok(msg) = decode::<DaemonToWrapper>(line.trim())
                && let DaemonToWrapper::Deliver(bm) = msg {
                    let agent = agent_arc.lock().unwrap();
                    agent.received.lock().unwrap().push(bm);
                }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bridget_daemon::daemon::{self, DaemonConfig};
    use bridget_daemon::registry::AgentRegistry;
    use bridget_daemon::wrapper::launch_acp_with;
    use std::os::unix::net::UnixListener;

    fn registry_with_unknown_stdio_agent() -> (AgentRegistry, PathBuf) {
        let mut fixture: serde_json::Value = serde_json::from_str(include_str!(
            "fixtures/registry/codex-claude.json"
        ))
        .expect("fixture de registre valide");
        assert_eq!(
            fixture["agents"]
                .as_object()
                .expect("agents de fixture")
                .keys()
                .cloned()
                .collect::<Vec<_>>(),
            vec!["claude".to_string(), "codex".to_string()],
            "la fixture versionnée ne déclare initialement que Codex et Claude"
        );
        fixture["agents"]["stdio-ouvert"] = serde_json::json!({
            "command": "sh",
            "args": [
                "-c",
                r#"
read initialize
echo '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1}}'
read new_session
echo '{"jsonrpc":"2.0","id":2,"result":{"sessionId":"stdio-ouvert"}}'
read prompt
echo '{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"stdio-ouvert","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"fixture-response"}}}}'
echo '{"jsonrpc":"2.0","id":3,"result":{"stopReason":"end_turn"}}'
"#
            ],
            "protocol": "acp",
            "permissions": "allow",
            "queue_capacity": 2,
            "notify_timeout_secs": 1
        });
        let dir = std::env::temp_dir().join(format!(
            "bridget-registry-fixture-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("horloge système")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("répertoire de registre temporaire");
        let path = dir.join("agents.json");
        std::fs::write(
            &path,
            serde_json::to_string_pretty(&fixture).expect("registre sérialisable"),
        )
        .expect("registre dynamique écrit");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .expect("permissions privées du registre dynamique");
        let registry = AgentRegistry::from_json(
            &std::fs::read_to_string(&path).expect("registre dynamique lisible"),
            &path,
        )
        .expect("registre dynamique chargé");
        (registry, dir)
    }

    fn registry_with_counting_stdio_agent() -> (AgentRegistry, PathBuf, PathBuf) {
        let (_, directory) = registry_with_unknown_stdio_agent();
        let path = directory.join("agents.json");
        let mut fixture: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let counter = directory.join("prompt-count");
        let script = format!(
            r#"
read initialize
echo '{{"jsonrpc":"2.0","id":1,"result":{{"protocolVersion":1}}}}'
read new_session
echo '{{"jsonrpc":"2.0","id":2,"result":{{"sessionId":"stdio-count"}}}}'
read prompt
printf x >> '{}'
echo '{{"jsonrpc":"2.0","method":"session/update","params":{{"sessionId":"stdio-count","update":{{"sessionUpdate":"agent_message_chunk","content":{{"type":"text","text":"prompt-lu"}}}}}}}}'
echo '{{"jsonrpc":"2.0","id":3,"result":{{"stopReason":"end_turn"}}}}'
sleep 2
"#,
            counter.display()
        );
        fixture["agents"]["stdio-ouvert"]["args"] = serde_json::json!(["-c", script]);
        std::fs::write(&path, serde_json::to_string_pretty(&fixture).unwrap()).unwrap();
        let registry = AgentRegistry::from_json(&std::fs::read_to_string(&path).unwrap(), &path)
            .unwrap();
        (registry, directory, counter)
    }

    fn wait_for_report(reader: &mut BufReader<UnixStream>) -> WrapperToDaemon {
        for _ in 0..80 {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => panic!("wrapper fermé avant son accusé idempotent"),
                Ok(_) => {
                    let decoded = decode(line.trim()).unwrap();
                    if let report @ (WrapperToDaemon::DeliverAcked { .. }
                    | WrapperToDaemon::DeliveryIndeterminate { .. }) = decoded
                    {
                        return report;
                    }
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) => {}
                Err(error) => panic!("lecture wrapper impossible: {error}"),
            }
        }
        panic!("accusé idempotent absent")
    }

    fn register_acp_wrapper(
        listener: &UnixListener,
    ) -> (BufReader<UnixStream>, BufWriter<UnixStream>, String) {
        let (stream, _) = listener.accept().expect("connexion du wrapper ACP");
        let read_stream = stream.try_clone().unwrap();
        read_stream
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        let mut reader = BufReader::new(read_stream);
        let mut writer = BufWriter::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let instance_id = match decode(line.trim()).unwrap() {
            WrapperToDaemon::Register {
                instance_id: Some(instance_id),
                ..
            } => instance_id,
            unexpected => panic!("Register ACP attendu : {unexpected:?}"),
        };
        writeln!(
            writer,
            "{}",
            encode(&DaemonToWrapper::Registered {
                name: "extension".to_string(),
            })
            .unwrap()
        )
        .unwrap();
        writer.flush().unwrap();
        (reader, writer, instance_id)
    }

    #[test]
    fn redelivery_idempotente_apres_reconnexion_n_injecte_qu_un_prompt() {
        let (registry, directory, counter) = registry_with_counting_stdio_agent();
        let socket = unique_socket_path();
        let _ = std::fs::remove_file(&socket);
        let listener = UnixListener::bind(&socket).unwrap();
        let home = directory.join("home");
        std::fs::create_dir_all(&home).unwrap();
        let wrapper_registry = registry.clone();
        let wrapper_socket = socket.clone();
        let wrapper = thread::spawn(move || {
            launch_acp_with(
                "stdio-ouvert",
                &[],
                Some("extension"),
                &wrapper_registry,
                &wrapper_socket,
                &home,
            )
            .map_err(|error| error.to_string())
        });

        let (mut first_reader, mut first_writer, instance_id) = register_acp_wrapper(&listener);
        let mut message = BridgetMessage::new("maicie", "extension", "une seule injection");
        message.id = "idempotent-reconnect".to_string();
        let delivery = DaemonToWrapper::DeliverIdempotent {
            delivery_id: "delivery-reconnect".to_string(),
            recipient_instance_id: instance_id.clone(),
            delivery_generation: 71,
            expires_at: i64::MAX,
            message: message.clone(),
        };
        writeln!(first_writer, "{}", encode(&delivery).unwrap()).unwrap();
        first_writer.flush().unwrap();
        for _ in 0..80 {
            if std::fs::read(&counter).map_or(0, |bytes| bytes.len()) == 1 {
                break;
            }
            thread::sleep(Duration::from_millis(25));
        }
        assert_eq!(std::fs::read(&counter).unwrap().len(), 1);
        assert!(matches!(
            wait_for_report(&mut first_reader),
            WrapperToDaemon::DeliverAcked {
                delivery_id,
                delivery_generation: 71,
            } if delivery_id == "delivery-reconnect"
        ));
        drop(first_writer);
        drop(first_reader);

        let (mut second_reader, mut second_writer, reconnected_instance_id) =
            register_acp_wrapper(&listener);
        assert_eq!(reconnected_instance_id, instance_id);
        writeln!(second_writer, "{}", encode(&delivery).unwrap()).unwrap();
        second_writer.flush().unwrap();
        assert!(matches!(
            wait_for_report(&mut second_reader),
            WrapperToDaemon::DeliverAcked {
                delivery_id,
                delivery_generation: 71,
            } if delivery_id == "delivery-reconnect"
        ));
        assert_eq!(std::fs::read(&counter).unwrap().len(), 1);
        drop(second_writer);
        drop(second_reader);
        assert_eq!(wrapper.join().unwrap(), Ok(()));
        let _ = std::fs::remove_file(socket);
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn test_unknown_registry_type_completes_a_tracked_exchange() {
        let (registry, registry_dir) = registry_with_unknown_stdio_agent();
        let socket = unique_socket_path();
        let db_path = PathBuf::from(format!(
            "/tmp/bridget-test-registry-open-{}.db",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&socket);
        let _ = std::fs::remove_file(&db_path);
        let config = DaemonConfig {
            socket_path: socket.clone(),
            db_path: db_path.clone(),
            log_path: PathBuf::from("/tmp/bridget-test-registry-open.log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        };
        thread::spawn(move || {
            let _ = daemon::run(config);
        });
        for _ in 0..50 {
            if socket.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(socket.exists(), "socket de daemon absente");

        let mut sender = FakeAgent::connect(&socket, "codex", Some("sender")).unwrap();
        let home = registry_dir.join("home");
        std::fs::create_dir_all(&home).unwrap();
        let wrapper_registry = registry.clone();
        let wrapper_socket = socket.clone();
        let wrapper = thread::spawn(move || {
            launch_acp_with(
                "stdio-ouvert",
                &[],
                Some("extension"),
                &wrapper_registry,
                &wrapper_socket,
                &home,
            )
            .map_err(|error| error.to_string())
        });

        let mut request = BridgetMessage::new(&sender.name, "extension", "réponds via ACP");
        request.reply = true;
        let mut delivered = false;
        for _ in 0..100 {
            sender.send_message(request.clone()).unwrap();
            match sender.read_response().unwrap() {
                DaemonToWrapper::Ack { .. } => {
                    delivered = true;
                    break;
                }
                DaemonToWrapper::Nack { .. } => thread::sleep(Duration::from_millis(10)),
                other => panic!("accusé de livraison inattendu : {other:?}"),
            }
        }
        assert!(delivered, "le wrapper réel ne s'est pas enregistré");
        let returned = match sender.read_response().unwrap() {
            DaemonToWrapper::Deliver(message) => message,
            other => panic!("réponse ACP attendue : {other:?}"),
        };
        assert_eq!(returned.in_reply_to.as_deref(), Some(request.id.as_str()));
        assert_eq!(returned.body, "fixture-response");

        let list = WrapperToDaemon::ListRequests {
            sender: sender.name.clone(),
            limit: 200,
        };
        writeln!(sender.writer, "{}", encode(&list).unwrap()).unwrap();
        sender.writer.flush().unwrap();
        match sender.read_response().unwrap() {
            DaemonToWrapper::RequestList { requests } => {
                assert_eq!(requests.len(), 1);
                assert_eq!(requests[0].state, "answered");
            }
            other => panic!("vue des demandes attendue : {other:?}"),
        }

        assert_eq!(
            wrapper.join().expect("thread wrapper joint"),
            Ok(()),
            "le wrapper réel doit terminer proprement après EOF ACP"
        );
        let _ = std::fs::remove_file(&socket);
        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_dir_all(registry_dir);
    }

    #[test]
    fn test_two_agents_communicate() {
        let socket = unique_socket_path();
        let db_path = PathBuf::from(format!("/tmp/bridget-test-{}.db", std::process::id()));

        // Nettoyer
        let _ = std::fs::remove_file(&socket);
        let _ = std::fs::remove_file(&db_path);

        let config = DaemonConfig {
            socket_path: socket.clone(),
            db_path: db_path.clone(),
            log_path: PathBuf::from("/tmp/bridget-test.log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        };

        // Lancer le daemon dans un thread
        let cfg = config.clone();
        let _daemon_handle = thread::spawn(move || {
            let _ = daemon::run(cfg);
        });

        // Attendre que la socket soit prête
        for _ in 0..50 {
            if socket.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
        assert!(socket.exists(), "socket non créée");

        // Connecter deux agents
        let agent_a = FakeAgent::connect(&socket, "codex", None).expect("agent A");
        let agent_b = FakeAgent::connect(&socket, "claude", None).expect("agent B");

        assert_eq!(agent_a.name, "codex-1");
        assert_eq!(agent_b.name, "claude-1");

        // Wrap dans Arc<Mutex> pour le receiver
        let received_b = agent_b.received.clone();

        let agent_a = Arc::new(Mutex::new(agent_a));
        let agent_b = Arc::new(Mutex::new(agent_b));

        FakeAgent::start_receiver(agent_b.clone());

        // Agent A envoie un message à Agent B
        {
            let mut a = agent_a.lock().unwrap();
            a.send("claude-1", "Bonjour Claude, analyse ce fichier stp")
                .unwrap();
            let resp = a.read_response().unwrap();
            assert!(
                matches!(resp, DaemonToWrapper::Ack { .. }),
                "pas d'Ack: {:?}",
                resp
            );
        }

        // Attendre la réception
        thread::sleep(Duration::from_millis(500));

        // Vérifier que B a reçu
        {
            let msgs = received_b.lock().unwrap();
            assert_eq!(msgs.len(), 1, "B devrait avoir reçu 1 message");
            assert_eq!(msgs[0].from, "codex-1");
            assert_eq!(msgs[0].to, "claude-1");
            assert_eq!(msgs[0].body, "Bonjour Claude, analyse ce fichier stp");
        }

        // Nettoyer
        std::fs::remove_file(&socket).ok();
        std::fs::remove_file(&db_path).ok();
    }

    #[test]
    fn test_agent_not_found() {
        let socket = unique_socket_path();
        let db_path = PathBuf::from(format!("/tmp/bridget-test-nf-{}.db", std::process::id()));

        let _ = std::fs::remove_file(&socket);
        let _ = std::fs::remove_file(&db_path);

        let config = DaemonConfig {
            socket_path: socket.clone(),
            db_path: db_path.clone(),
            log_path: PathBuf::from("/tmp/bridget-test-nf.log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        };

        let cfg = config.clone();
        thread::spawn(move || {
            let _ = daemon::run(cfg);
        });

        for _ in 0..50 {
            if socket.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }

        let mut agent = FakeAgent::connect(&socket, "codex", None).expect("agent");
        assert_eq!(agent.name, "codex-1");

        // Envoyer vers un agent qui n'existe pas
        agent.send("ghost-agent", "hello?").unwrap();
        let resp = agent.read_response().unwrap();
        assert!(
            matches!(resp, DaemonToWrapper::Nack { .. }),
            "devrait être Nack: {:?}",
            resp
        );

        std::fs::remove_file(&socket).ok();
        std::fs::remove_file(&db_path).ok();
    }

    #[test]
    fn test_reply_from_ephemeral_cli_is_rejected() {
        let socket = unique_socket_path();
        let db_path = PathBuf::from(format!(
            "/tmp/bridget-test-cli-reply-{}.db",
            std::process::id()
        ));
        let config = DaemonConfig {
            socket_path: socket.clone(),
            db_path: db_path.clone(),
            log_path: PathBuf::from("/tmp/bridget-test-cli-reply.log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        };
        thread::spawn(move || {
            let _ = daemon::run(config);
        });
        for _ in 0..50 {
            if socket.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }

        let mut recipient = FakeAgent::connect(&socket, "codex", Some("recipient")).unwrap();
        let mut cli = FakeAgent::connect(&socket, "cli", Some("cli-send-test")).unwrap();
        let mut msg = BridgetMessage::new("cli-send-test", "recipient", "réponds-user");
        msg.reply = true;
        cli.send_message(msg).unwrap();

        match cli.read_response().unwrap() {
            DaemonToWrapper::Nack { reason, .. } => {
                assert!(reason.contains("--reply requiert un agent Bridget connecté"));
            }
            response => panic!("un client éphémère doit être refusé : {response:?}"),
        }

        // Le destinataire n'a rien reçu : une requête impossible n'est pas livrée.
        recipient.reader.get_mut().set_nonblocking(true).unwrap();
        let mut line = String::new();
        assert!(
            matches!(recipient.reader.read_line(&mut line), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
        );

        std::fs::remove_file(&socket).ok();
        std::fs::remove_file(&db_path).ok();
    }

    #[test]
    fn test_agent_can_rename_without_reconnecting() {
        let socket = unique_socket_path();
        let db_path = PathBuf::from(format!(
            "/tmp/bridget-test-rename-{}.db",
            std::process::id()
        ));
        let config = DaemonConfig {
            socket_path: socket.clone(),
            db_path: db_path.clone(),
            log_path: PathBuf::from("/tmp/bridget-test-rename.log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        };
        thread::spawn(move || {
            let _ = daemon::run(config);
        });
        for _ in 0..50 {
            if socket.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
        let mut a = FakeAgent::connect(&socket, "codex", None).unwrap();
        let mut b = FakeAgent::connect(&socket, "claude", None).unwrap();
        assert!(matches!(
            a.rename("analyse").unwrap(),
            DaemonToWrapper::Renamed { .. }
        ));
        b.send("analyse", "message après renommage").unwrap();
        assert!(matches!(
            b.read_response().unwrap(),
            DaemonToWrapper::Ack { .. }
        ));
        b.send("codex-1", "ancien nom").unwrap();
        assert!(matches!(
            b.read_response().unwrap(),
            DaemonToWrapper::Nack { .. }
        ));
        std::fs::remove_file(&socket).ok();
        std::fs::remove_file(&db_path).ok();
    }

    #[test]
    fn test_request_cancellation_stops_reminders() {
        let socket = unique_socket_path();
        let db_path = PathBuf::from(format!(
            "/tmp/bridget-test-cancel-{}.db",
            std::process::id()
        ));
        let config = DaemonConfig {
            socket_path: socket.clone(),
            db_path: db_path.clone(),
            log_path: PathBuf::from("/tmp/bridget-test-cancel.log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        };
        thread::spawn(move || {
            let _ = daemon::run(config);
        });
        for _ in 0..50 {
            if socket.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }

        let requester = Arc::new(Mutex::new(
            FakeAgent::connect(&socket, "codex", Some("requester")).unwrap(),
        ));
        let worker = Arc::new(Mutex::new(
            FakeAgent::connect(&socket, "claude", Some("worker")).unwrap(),
        ));
        let received = worker.lock().unwrap().received.clone();
        FakeAgent::start_receiver(worker.clone());

        let mut request = BridgetMessage::new("requester", "worker", "travail devenu inutile");
        request.reply = true;
        request.reply_timeout = Some(3);
        let request_id = request.id.clone();
        {
            let mut requester = requester.lock().unwrap();
            requester.send_message(request).unwrap();
            assert!(matches!(
                requester.read_response().unwrap(),
                DaemonToWrapper::Ack { .. }
            ));
            let cancel = WrapperToDaemon::CancelRequest {
                id: request_id.clone(),
                sender: "requester".to_string(),
                reason: Some("priorité changée".to_string()),
            };
            writeln!(requester.writer, "{}", encode(&cancel).unwrap()).unwrap();
            requester.writer.flush().unwrap();
            assert!(matches!(
                requester.read_response().unwrap(),
                DaemonToWrapper::RequestCancelled { state, .. } if state == "cancelled"
            ));
        }

        thread::sleep(Duration::from_secs(4));
        let messages = received.lock().unwrap();
        assert_eq!(messages.len(), 2, "la demande et son annulation seulement");
        assert_eq!(messages[1].from, "bridget");
        assert!(messages[1].body.contains("Aucune réponse n'est requise"));

        std::fs::remove_file(&socket).ok();
        std::fs::remove_file(&db_path).ok();
    }

    #[test]
    fn test_only_sender_can_cancel_request() {
        let socket = unique_socket_path();
        let db_path = PathBuf::from(format!("/tmp/bridget-test-owner-{}.db", std::process::id()));
        let config = DaemonConfig {
            socket_path: socket.clone(),
            db_path: db_path.clone(),
            log_path: PathBuf::from("/tmp/bridget-test-owner.log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        };
        thread::spawn(move || {
            let _ = daemon::run(config);
        });
        for _ in 0..50 {
            if socket.exists() {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
        let mut requester = FakeAgent::connect(&socket, "codex", Some("requester")).unwrap();
        let _worker = FakeAgent::connect(&socket, "claude", Some("worker")).unwrap();
        let mut intruder = FakeAgent::connect(&socket, "codex", Some("intruder")).unwrap();
        let mut request = BridgetMessage::new("requester", "worker", "travail");
        request.reply = true;
        let request_id = request.id.clone();
        requester.send_message(request).unwrap();
        assert!(matches!(
            requester.read_response().unwrap(),
            DaemonToWrapper::Ack { .. }
        ));
        let cancel = WrapperToDaemon::CancelRequest {
            id: request_id,
            sender: "intruder".to_string(),
            reason: None,
        };
        writeln!(intruder.writer, "{}", encode(&cancel).unwrap()).unwrap();
        intruder.writer.flush().unwrap();
        assert!(
            matches!(intruder.read_response().unwrap(), DaemonToWrapper::Nack { reason, .. } if reason.contains("seul l'émetteur"))
        );
        std::fs::remove_file(&socket).ok();
        std::fs::remove_file(&db_path).ok();
    }
}
