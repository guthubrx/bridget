//! Test d'intégration : deux faux wrappers communiquent via le daemon bridget.
//!
//! Ce test lance le daemon en arrière-plan, connecte deux faux agents,
//! et vérifie qu'un message envoyé par l'un est bien reçu par l'autre.

use bridget_core::BridgetMessage;
use bridget_transport::protocol::{decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::io::{BufRead, BufReader, BufWriter, Write};
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
    fn connect(socket_path: &PathBuf, agent_type: &str, name: Option<&str>) -> Result<Self, String> {
        let stream = UnixStream::connect(socket_path).map_err(|e| e.to_string())?;
        let reader_stream = stream.try_clone().map_err(|e| e.to_string())?;
        let writer = BufWriter::new(stream);
        let reader = BufReader::new(reader_stream);

        let reg = WrapperToDaemon::Register {
            agent_type: agent_type.to_string(),
            name: name.map(|s| s.to_string()),
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
        agent.reader.read_line(&mut line).map_err(|e| e.to_string())?;
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
        let wtd = WrapperToDaemon::Send(msg);
        let json = encode(&wtd).map_err(|e| e.to_string())?;
        writeln!(self.writer, "{}", json).map_err(|e| e.to_string())?;
        self.writer.flush().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn rename(&mut self, name: &str) -> Result<DaemonToWrapper, String> {
        let request = WrapperToDaemon::Rename { current_name: self.name.clone(), name: name.to_string() };
        writeln!(self.writer, "{}", encode(&request).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        self.writer.flush().map_err(|e| e.to_string())?;
        let response = self.read_response()?;
        if let DaemonToWrapper::Renamed { name, .. } = &response { self.name = name.clone(); }
        Ok(response)
    }

    fn read_response(&mut self) -> Result<DaemonToWrapper, String> {
        let mut line = String::new();
        self.reader.read_line(&mut line).map_err(|e| e.to_string())?;
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
            if let Ok(msg) = decode::<DaemonToWrapper>(line.trim()) {
                if let DaemonToWrapper::Deliver(bm) = msg {
                    let agent = agent_arc.lock().unwrap();
                    agent.received.lock().unwrap().push(bm);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bridget_daemon::daemon::{self, DaemonConfig};

    #[test]
    fn test_two_agents_communicate() {
        let socket = unique_socket_path();
        let db_path = PathBuf::from(format!(
            "/tmp/bridget-test-{}.db",
            std::process::id()
        ));

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
        let daemon_handle = thread::spawn(move || {
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
            a.send("claude-1", "Bonjour Claude, analyse ce fichier stp").unwrap();
            let resp = a.read_response().unwrap();
            assert!(matches!(resp, DaemonToWrapper::Ack { .. }), "pas d'Ack: {:?}", resp);
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
        let db_path = PathBuf::from(format!(
            "/tmp/bridget-test-nf-{}.db",
            std::process::id()
        ));

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
    fn test_agent_can_rename_without_reconnecting() {
        let socket = unique_socket_path();
        let db_path = PathBuf::from(format!("/tmp/bridget-test-rename-{}.db", std::process::id()));
        let config = DaemonConfig { socket_path: socket.clone(), db_path: db_path.clone(), log_path: PathBuf::from("/tmp/bridget-test-rename.log"), circuit_breaker_window: 180, circuit_breaker_limit: 8, dedup_window: 180, quarantine_window: 3600, retention_days: 7 };
        thread::spawn(move || { let _ = daemon::run(config); });
        for _ in 0..50 { if socket.exists() { break; } thread::sleep(Duration::from_millis(50)); }
        let mut a = FakeAgent::connect(&socket, "codex", None).unwrap();
        let mut b = FakeAgent::connect(&socket, "claude", None).unwrap();
        assert!(matches!(a.rename("analyse").unwrap(), DaemonToWrapper::Renamed { .. }));
        b.send("analyse", "message après renommage").unwrap();
        assert!(matches!(b.read_response().unwrap(), DaemonToWrapper::Ack { .. }));
        b.send("codex-1", "ancien nom").unwrap();
        assert!(matches!(b.read_response().unwrap(), DaemonToWrapper::Nack { .. }));
        std::fs::remove_file(&socket).ok();
        std::fs::remove_file(&db_path).ok();
    }
}
