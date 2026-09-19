//! Frontières de sécurité traversées par les binaires, sous namespace privé.
#[path = "support/idempotent.rs"]
pub mod fixture;
use bridget_transport::protocol::{
    ConnectionRole, SERVICE_CONTRACT_VERSION, ServiceCapability, ServiceRefusal,
    ServiceRequestOperation, ServiceRequestPayload,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use fixture::*;
use serde_json::json;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::time::{Duration, Instant};

#[test]
fn les_fichiers_naissent_prives_et_le_bootstrap_refuse_les_substitutions() {
    let root = test_root("security-private");
    let daemon = spawn_daemon(&root, None);
    let uid = unsafe { libc::geteuid() };
    for entry in fs::read_dir(root.join("state")).unwrap() {
        let meta = entry.unwrap().metadata().unwrap();
        assert_eq!(meta.uid(), uid);
        assert_eq!(meta.mode() & 0o077, 0);
        if meta.is_file() {
            assert_eq!(meta.mode() & 0o777, 0o600);
        }
    }
    daemon.stop();
    fs::remove_dir_all(root).unwrap();

    for substitution in ["symlink", "public"] {
        let root = test_root("security-substitution");
        let target = root.join("sentinel");
        private_write(&target, b"CANARI_PRIVE_INCHANGE").unwrap();
        let database = root.join("state/bridget.db");
        if substitution == "symlink" {
            std::os::unix::fs::symlink(&target, &database).unwrap();
        } else {
            private_write(&database, b"base non privee").unwrap();
            fs::set_permissions(&database, fs::Permissions::from_mode(0o644)).unwrap();
        }
        let result = run_isolated(&root, &["daemon"], false);
        assert!(
            !result.status.success(),
            "bootstrap accepté : {substitution}"
        );
        assert!(!socket(&root).exists());
        assert_eq!(fs::read(&target).unwrap(), b"CANARI_PRIVE_INCHANGE");
        assert!(!output_text(&result).contains("CANARI_PRIVE_INCHANGE"));
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn daemon_ne_traite_ni_la_trame_sans_lf_ni_la_trame_hors_borne() {
    let root = test_root("security-frame");
    let daemon = spawn_daemon(&root, None);
    for complete in [false, true] {
        let mut stream = UnixStream::connect(socket(&root)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        if complete {
            let over = vec![b' '; bridget_transport::jsonl::MAX_DAEMON_FRAME_BYTES + 1];
            let _ = stream.write_all(&over);
        } else {
            stream.write_all(b"{\"type\":\"ListAgents\"}").unwrap();
            stream.shutdown(std::net::Shutdown::Write).unwrap();
        }
        let mut response = String::new();
        let read = BufReader::new(stream).read_line(&mut response);
        assert!(
            matches!(read, Ok(0))
                || matches!(&read, Err(error) if matches!(error.kind(), std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::BrokenPipe)),
            "requête partielle traitée : {response}"
        );
        assert!(response.is_empty(), "aucun résultat partiel");
    }
    let mut observer = Client::connect(&socket(&root));
    observer.send(WrapperToDaemon::ListAgents);
    assert!(
        matches!(observer.receive(), DaemonToWrapper::AgentList { agents } if agents.is_empty())
    );
    drop(observer);
    daemon.stop();
    fs::remove_dir_all(root).unwrap();
}

fn service(root: &std::path::Path, capabilities: Vec<ServiceCapability>) -> Client {
    let mut client = Client::connect(&socket(root));
    client.send(WrapperToDaemon::RoleHandshake {
        role: ConnectionRole::Service,
    });
    assert!(matches!(
        client.receive(),
        DaemonToWrapper::RoleAccepted { .. }
    ));
    client.send(WrapperToDaemon::ServiceHello {
        version: SERVICE_CONTRACT_VERSION,
        service: "guichet".into(),
        issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".into(),
        capabilities,
    });
    assert!(matches!(
        client.receive(),
        DaemonToWrapper::ServiceWelcome { .. }
    ));
    client
}

#[test]
fn une_portee_valide_ne_permet_pas_de_changer_l_emetteur_du_depot() {
    let root = test_root("security-sender");
    let daemon = spawn_daemon(&root, None);
    let mut producer = register_agent_as(&socket(&root), ACTOR, "security-producer");
    let target = register_agent_as(&socket(&root), RECIPIENT, "security-target");
    let issued_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    producer.send(WrapperToDaemon::ServiceRequest {
        version: SERVICE_CONTRACT_VERSION,
        issuer_scope: "015_scope_0123456789abcdef0123456789abcdef".into(),
        request_id: "security-foreign-producer".into(),
        issued_at,
        from: RECIPIENT.into(),
        to: "guichet".into(),
        operation: ServiceRequestOperation::MissionStatus,
        payload: ServiceRequestPayload::Delegation {
            delegation_id: uuid::Uuid::new_v4().to_string(),
        },
    });
    // La portée bien formée ne confère pas l'identité du wrapper voisin.
    assert!(matches!(
        producer.receive(),
        DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::DeclaredSenderMismatch
        }
    ));
    let db = rusqlite::Connection::open_with_flags(
        root.join("state/bridget.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    for table in ["guichet_requests", "tracked_requests", "ledger"] {
        let count: i64 = db
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 0, "aucun effet durable dans {table}");
    }
    drop(db);
    drop(producer);
    drop(target);
    daemon.stop();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn guichet_borne_lf_inclus_et_capacite_non_heritee_apres_fermeture() {
    let root = test_root("security-capability");
    let daemon = spawn_daemon(&root, None);
    let mut capable = service(&root, vec![ServiceCapability::GuichetV1]);
    let request = WrapperToDaemon::GuichetClaimNext {
        version: SERVICE_CONTRACT_VERSION,
    };
    capable.send(request.clone());
    assert!(matches!(
        capable.receive(),
        DaemonToWrapper::GuichetEmpty { .. }
    ));
    let base = bridget_transport::protocol::encode(&request).unwrap();
    for excess in [0, 1] {
        let wire = format!(
            "{}{}\n",
            base,
            " ".repeat(64 * 1024 + excess - base.len() - 1)
        );
        capable.writer.write_all(wire.as_bytes()).unwrap();
        capable.writer.flush().unwrap();
        let response = capable.receive();
        assert_eq!(
            matches!(
                response,
                DaemonToWrapper::ServiceRejected {
                    reason: ServiceRefusal::FrameTooLarge
                }
            ),
            excess == 1,
            "{response:?}"
        );
    }
    capable
        .writer
        .get_ref()
        .shutdown(std::net::Shutdown::Both)
        .unwrap();
    drop(capable);
    let mut unprivileged = service(&root, vec![]);
    unprivileged.send(request);
    assert!(matches!(
        unprivileged.receive(),
        DaemonToWrapper::ServiceRejected {
            reason: ServiceRefusal::CapabilityRequired
        }
    ));
    let db = rusqlite::Connection::open(root.join("state/bridget.db")).unwrap();
    let count: i64 = db
        .query_row("SELECT COUNT(*) FROM guichet_requests", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(count, 0);
    drop(db);
    drop(unprivileged);
    daemon.stop();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn mcp_reel_garde_outcome_unknown_apres_reponse_invalide_sans_fuite_du_canari() {
    for fault in [
        "sans-lf",
        "invalide",
        "mauvaise-trame",
        "mauvais-id",
        "mauvaise-operation",
    ] {
        let root = test_root("security-mcp");
        let listener = UnixListener::bind(socket(&root)).unwrap();
        fs::set_permissions(socket(&root), fs::Permissions::from_mode(0o600)).unwrap();
        listener.set_nonblocking(true).unwrap();
        let credential =
            bridget_transport::protocol::IdentityCredential::new(format!("security-mock-{fault}"));
        save_fixture_credential(
            &socket(&root),
            ACTOR,
            "security-instance",
            credential.clone(),
        );
        let server = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(10);
            let stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline);
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    Err(error) => panic!("{error}"),
                }
            };
            stream.set_nonblocking(false).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut client = Client {
                reader: BufReader::new(stream.try_clone().unwrap()),
                writer: BufWriter::new(stream),
                owner: None,
            };
            // Client est ici côté serveur : décodage des requêtes indépendant.
            for stage in 0..4 {
                let mut line = String::new();
                client.reader.read_line(&mut line).unwrap();
                let command: WrapperToDaemon =
                    bridget_transport::protocol::decode(line.trim_end()).unwrap();
                let response = match (stage, command) {
                    (0, WrapperToDaemon::RoleHandshake { role }) => {
                        DaemonToWrapper::RoleAccepted { role }
                    }
                    (2, WrapperToDaemon::ClientHello { capabilities, .. }) => {
                        DaemonToWrapper::ClientWelcome {
                            version: 1,
                            build_id: "unknown".into(),
                            horizon_secs: 604800,
                            issued_at_tolerance_secs: 300,
                            capabilities,
                        }
                    }
                    (
                        1,
                        WrapperToDaemon::RegisterAuxiliary {
                            agent_id,
                            instance_id,
                            credential: received,
                        },
                    ) => {
                        assert_eq!(agent_id, ACTOR);
                        assert_eq!(instance_id, "security-instance");
                        assert_eq!(received, credential);
                        DaemonToWrapper::Registered {
                            agent_id,
                            credential: None,
                        }
                    }
                    (3, WrapperToDaemon::SendIdempotent { message_id, .. }) => {
                        let reply = match fault {
                            "mauvaise-trame" => json!({"type":"Ack","id":"CANARI_SECRET_089"}),
                            "mauvais-id" | "mauvaise-operation" => json!({
                                "type":"IdempotencyResult",
                                "operation_kind":if fault == "mauvaise-operation" { "spawn" } else { "send" },
                                "idempotency_key":if fault == "mauvais-id" { "CANARI_SECRET_089" } else { &message_id },
                                "issue":{"kind":"accepted","expires_at":1900000000}
                            }),
                            _ => json!({"type":"inconnu","secret":"CANARI_SECRET_089"}),
                        };
                        if matches!(fault, "mauvais-id" | "mauvaise-operation") {
                            assert!(matches!(
                                serde_json::from_value::<DaemonToWrapper>(reply.clone()).unwrap(),
                                DaemonToWrapper::IdempotencyResult { .. }
                            ));
                        }
                        client
                            .writer
                            .write_all(reply.to_string().as_bytes())
                            .unwrap();
                        if fault != "sans-lf" {
                            client.writer.write_all(b"\n").unwrap();
                        }
                        client.writer.flush().unwrap();
                        break;
                    }
                    (_, other) => panic!("étape {stage}: {other:?}"),
                };
                writeln!(
                    client.writer,
                    "{}",
                    bridget_transport::protocol::encode(&response).unwrap()
                )
                .unwrap();
                client.writer.flush().unwrap();
            }
        });
        let mut mcp = McpProcess::start(&root, ACTOR, "security-instance");
        let _ = mcp.request(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}));
        mcp.notify(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
        let response=mcp.request(json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"bridget_send","arguments":{"to":RECIPIENT,"body":"preuve publique"}}}));
        assert_eq!(
            response["result"]["structuredContent"]["status"], "outcome_unknown",
            "{fault}: {response}"
        );
        assert!(!response.to_string().contains("CANARI_SECRET_089"));
        mcp.stop();
        server.join().unwrap();
        fs::remove_dir_all(root).unwrap();
    }
}
