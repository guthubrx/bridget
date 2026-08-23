use maicie::bridget_client::{
    AttachWindow, BridgetClient, BridgetClientError, BridgetClientLimits, IdempotencyIssue,
    PublicMessage, SpawnOrder, SpawnOutcome, SubscriptionEvent,
};
use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

#[test]
fn negocie_la_version_et_rejete_une_capacite_absente() {
    let fixture = SocketFixture::new("missing-capability");
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("client attendu");
        let (mut reader, mut writer) = split(stream);
        assert_client_handshake(&mut reader, &mut writer);
        let hello = read_json(&mut reader);
        assert_eq!(hello["type"], "ClientHello");
        assert_eq!(hello["contract_version"], 1);
        assert_eq!(hello["capabilities"], json!(["send_idempotent", "lookup"]));
        write_json(
            &mut writer,
            json!({
                "type": "ClientWelcome",
                "version": 1,
                "horizon_secs": 3600,
                "issued_at_tolerance_secs": 30,
                "capabilities": ["send_idempotent"]
            }),
        );
    });

    let error = match BridgetClient::connect(fixture.path(), "scope-client-012") {
        Ok(_) => panic!("la capability lookup doit etre exigee"),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        BridgetClientError::CapabilityMissing { capability } if capability == "lookup"
    ));
    server.join().expect("serveur termine");
}

#[test]
fn rejette_une_version_negociee_differente() {
    let fixture = SocketFixture::new("bad-version");
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("client attendu");
        let (mut reader, mut writer) = split(stream);
        assert_client_handshake(&mut reader, &mut writer);
        let _hello = read_json(&mut reader);
        write_json(
            &mut writer,
            json!({
                "type": "ClientWelcome",
                "version": 2,
                "horizon_secs": 3600,
                "issued_at_tolerance_secs": 30,
                "capabilities": ["send_idempotent", "lookup"]
            }),
        );
    });

    let error = match BridgetClient::connect(fixture.path(), "scope-client-012") {
        Ok(_) => panic!("une version negociee divergente doit etre refusee"),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        BridgetClientError::VersionUnsupported {
            requested: 1,
            received: 2
        }
    ));
    server.join().expect("serveur termine");
}

#[test]
fn conserve_le_message_id_et_le_corps_exact_sur_retry_mais_signale_la_divergence() {
    let fixture = SocketFixture::new("idempotency");
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("client attendu");
        let (mut reader, mut writer) = split(stream);
        assert_client_handshake(&mut reader, &mut writer);
        let _hello = read_json(&mut reader);
        write_welcome(&mut writer);

        let first = read_json(&mut reader);
        assert_eq!(first["type"], "SendIdempotent");
        assert_eq!(first["message_id"], "message-client-1");
        assert_eq!(first["message"]["id"], "message-client-1");
        assert_eq!(first["message"]["body"], "corps immuable");
        write_issue(&mut writer, "outcome_unknown");

        let retry = read_json(&mut reader);
        assert_eq!(retry, first, "le retry rejoue les octets logiques exacts");
        write_issue(&mut writer, "accepted");

        let divergent = read_json(&mut reader);
        assert_eq!(divergent["message_id"], "message-client-1");
        assert_eq!(divergent["message"]["body"], "corps divergent");
        write_issue(&mut writer, "envelope_mismatch");
    });

    let mut client = BridgetClient::connect(fixture.path(), "scope-client-012").unwrap();
    let immutable_message = message("corps immuable");
    assert!(matches!(
        client
            .send_idempotent(&immutable_message, "message-client-1", 1_700_000_000)
            .unwrap(),
        IdempotencyIssue::OutcomeUnknown { .. }
    ));
    assert!(matches!(
        client
            .send_idempotent(&immutable_message, "message-client-1", 1_700_000_000)
            .unwrap(),
        IdempotencyIssue::Accepted { .. }
    ));
    assert!(matches!(
        client
            .send_idempotent(
                &message("corps divergent"),
                "message-client-1",
                1_700_000_000
            )
            .unwrap(),
        IdempotencyIssue::EnvelopeMismatch
    ));
    server.join().expect("serveur termine");
}

#[test]
fn lit_gap_et_end_uniquement_depuis_l_abonnement_public() {
    let fixture = SocketFixture::new("subscription");
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("client idempotent attendu");
        let (mut reader, mut writer) = split(stream);
        assert_client_handshake(&mut reader, &mut writer);
        let _hello = read_json(&mut reader);
        write_welcome(&mut writer);

        let (stream, _) = listener.accept().expect("client attach attendu");
        let (mut reader, mut writer) = split(stream);
        let role = read_json(&mut reader);
        assert_eq!(role, json!({"type": "RoleHandshake", "role": "attach"}));
        write_json(
            &mut writer,
            json!({"type": "RoleAccepted", "role": "attach"}),
        );
        let subscribe = read_json(&mut reader);
        assert_eq!(subscribe["type"], "Subscribe");
        assert_eq!(subscribe["agent"], "prospective");
        write_json(
            &mut writer,
            json!({
                "type": "Gap",
                "subscription_id": "sub-1",
                "from_seq": 11,
                "to_seq": 13,
                "reason": "vue lente"
            }),
        );
        write_json(
            &mut writer,
            json!({"type": "End", "subscription_id": "sub-1", "reason": "fermeture"}),
        );
    });

    let adapter = BridgetClient::connect(fixture.path(), "scope-client-012").unwrap();
    let mut subscription = adapter
        .subscribe("prospective", AttachWindow::Today)
        .unwrap();
    assert!(matches!(
        subscription.next_event().unwrap(),
        SubscriptionEvent::Gap {
            subscription_id,
            from_seq: 11,
            to_seq: 13,
            ..
        } if subscription_id == "sub-1"
    ));
    assert!(matches!(
        subscription.next_event().unwrap(),
        SubscriptionEvent::End {
            subscription_id,
            reason
        } if subscription_id == "sub-1" && reason == "fermeture"
    ));
    server.join().expect("serveur termine");
}

#[test]
fn cancel_request_negocie_le_role_wrapper_sur_sa_connexion_ephemere() {
    let fixture = SocketFixture::new("cancel-wrapper");
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("client idempotent attendu");
        let (mut reader, mut writer) = split(stream);
        assert_client_handshake(&mut reader, &mut writer);
        let _hello = read_json(&mut reader);
        write_welcome(&mut writer);

        let (stream, _) = listener.accept().expect("connexion cancel attendue");
        let (mut reader, mut writer) = split(stream);
        assert_eq!(
            read_json(&mut reader),
            json!({"type": "RoleHandshake", "role": "wrapper"})
        );
        write_json(
            &mut writer,
            json!({"type": "RoleAccepted", "role": "wrapper"}),
        );
        assert_eq!(
            read_json(&mut reader),
            json!({
                "type": "CancelRequest",
                "id": "request-1",
                "sender": "maicie",
                "reason": "decision explicite"
            })
        );
        write_json(
            &mut writer,
            json!({"type": "RequestCancelled", "id": "request-1", "state": "cancelled"}),
        );
    });

    let client = BridgetClient::connect(fixture.path(), "scope-client-012").unwrap();
    let cancellation = client
        .cancel_request("request-1", "maicie", Some("decision explicite"))
        .unwrap();
    assert_eq!(cancellation.id, "request-1");
    assert_eq!(cancellation.state, "cancelled");
    server.join().expect("serveur termine");
}

#[test]
fn spawn_order_negocie_le_role_wrapper_sur_sa_connexion_ephemere() {
    let fixture = SocketFixture::new("spawn-wrapper");
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("client idempotent attendu");
        let (mut reader, mut writer) = split(stream);
        assert_client_handshake(&mut reader, &mut writer);
        let _hello = read_json(&mut reader);
        write_welcome(&mut writer);

        let (stream, _) = listener.accept().expect("connexion spawn attendue");
        let (mut reader, mut writer) = split(stream);
        assert_eq!(
            read_json(&mut reader),
            json!({"type": "RoleHandshake", "role": "wrapper"})
        );
        write_json(
            &mut writer,
            json!({"type": "RoleAccepted", "role": "wrapper"}),
        );
        let order = read_json(&mut reader);
        assert_eq!(order["type"], "SpawnOrder");
        assert_eq!(order["command_id"], "command-1");
        assert_eq!(order["agent_type"], "codex");
        write_json(
            &mut writer,
            json!({"type": "SpawnAccepted", "command_id": "command-1", "name": "sentry"}),
        );
    });

    let client = BridgetClient::connect(fixture.path(), "scope-client-012").unwrap();
    let outcome = client
        .spawn_order(&SpawnOrder {
            agent_type: "codex".to_string(),
            name: Some("sentry".to_string()),
            cwd: "/tmp".to_string(),
            persistent: true,
            command_id: "command-1".to_string(),
            issued_at: 1_700_000_000,
            deadline_at: 1_700_000_600,
        })
        .unwrap();
    assert!(matches!(
        outcome,
        SpawnOutcome::Accepted { command_id, name } if command_id == "command-1" && name == "sentry"
    ));
    server.join().expect("serveur termine");
}

#[test]
fn replay_spawn_order_reemet_les_octets_approuves_sans_reserialisation() {
    let fixture = SocketFixture::new("spawn-replay-exact");
    let listener = fixture.bind();
    let bytes = br#"{"type":"SpawnOrder","agent_type":"claude","name":null,"cwd":"/tmp","persistent":true,"command_id":"command-exact","issued_at":100,"deadline_at":160}"#.to_vec();
    let expected = bytes.clone();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("connexion spawn attendue");
        let (mut reader, mut writer) = split(stream);
        assert_eq!(
            read_json(&mut reader),
            json!({"type": "RoleHandshake", "role": "wrapper"})
        );
        write_json(
            &mut writer,
            json!({"type": "RoleAccepted", "role": "wrapper"}),
        );
        let mut raw = Vec::new();
        reader.read_until(b'\n', &mut raw).expect("SpawnOrder lu");
        assert_eq!(&raw[..raw.len() - 1], expected.as_slice());
        write_json(
            &mut writer,
            json!({
                "type": "SpawnAccepted",
                "command_id": "command-exact",
                "name": "claude-review",
                "definition": {"digest": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}
            }),
        );
    });

    let replay = BridgetClient::replay_spawn_order_bytes_at(
        fixture.path(),
        BridgetClientLimits::default(),
        &bytes,
    )
    .unwrap();
    assert!(matches!(
        replay.outcome,
        SpawnOutcome::Accepted { command_id, name }
            if command_id == "command-exact" && name == "claude-review"
    ));
    assert_eq!(
        replay.definition_digest.as_deref(),
        Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
    );
    server.join().expect("serveur termine");
}

#[test]
fn annuaire_est_lisible_sans_negociation_et_une_base_bridget_ne_peut_etre_lue() {
    let fixture = SocketFixture::new("directory");
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("lecteur annuaire attendu");
        let (mut reader, mut writer) = split(stream);
        assert_eq!(read_json(&mut reader), json!({"type": "ListAgents"}));
        write_json(
            &mut writer,
            json!({
                "type": "AgentList",
                "agents": [{
                    "name": "prospective",
                    "agent_type": "codex",
                    "connection_id": "conn-1",
                    "host": "local",
                    "transport": "unix",
                    "os": "macOS",
                    "state": "idle",
                    "last_seen_secs": 1,
                    "reconnect_count": 0,
                    "domain": "bridget",
                    "model": "gpt-5",
                    "effort": "high"
                }]
            }),
        );
    });

    let agents = BridgetClient::list_agents_at(fixture.path()).unwrap();
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0].name, "prospective");
    server.join().expect("serveur termine");

    let database_path = fixture.path().with_file_name("bridget.db");
    fs::write(&database_path, b"base Bridget a ne pas lire").unwrap();
    let error = match BridgetClient::connect(&database_path, "scope-client-012") {
        Ok(_) => panic!("un fichier SQLite n'est pas un socket Bridget"),
        Err(error) => error,
    };
    assert!(matches!(error, BridgetClientError::Connect { .. }));
    assert_eq!(
        fs::read(&database_path).unwrap(),
        b"base Bridget a ne pas lire"
    );
    fs::remove_file(database_path).unwrap();
}

#[test]
fn daemon_muet_expire_le_handshake_dans_le_budget_configure() {
    let fixture = SocketFixture::new("timeout");
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("client attendu");
        let (mut reader, _writer) = split(stream);
        assert_eq!(
            read_json(&mut reader),
            json!({"type": "RoleHandshake", "role": "client"})
        );
        thread::sleep(Duration::from_millis(80));
    });
    let limits = BridgetClientLimits {
        connect_timeout: Duration::from_secs(1),
        io_timeout: Duration::from_millis(10),
        max_frame_bytes: 1024,
    };

    let error = match BridgetClient::connect_with_limits(fixture.path(), "scope-client-012", limits)
    {
        Ok(_) => panic!("un daemon muet ne doit pas negocier"),
        Err(error) => error,
    };

    assert!(matches!(error, BridgetClientError::Timeout { .. }));
    server.join().expect("serveur termine");
}

#[test]
fn timeout_empoisonne_la_connexion_et_interdit_de_lire_une_reponse_tardive() {
    let fixture = SocketFixture::new("poison");
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("client attendu");
        let (mut reader, mut writer) = split(stream);
        assert_client_handshake(&mut reader, &mut writer);
        let _hello = read_json(&mut reader);
        write_welcome(&mut writer);
        assert_eq!(read_json(&mut reader)["type"], "Lookup");
        thread::sleep(Duration::from_millis(50));
        if writeln!(
            writer,
            "{}",
            serde_json::to_string(&json!({
                "type": "IdempotencyResult",
                "operation_kind": "send",
                "idempotency_key": "message-client-1",
                "issue": {"kind": "accepted", "expires_at": 1_700_003_600_i64}
            }))
            .expect("issue JSON")
        )
        .is_err()
            || writer.flush().is_err()
        {
            return;
        }
        reader
            .get_mut()
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        let mut unexpected = String::new();
        assert_eq!(reader.read_line(&mut unexpected).unwrap(), 0);
    });
    let limits = BridgetClientLimits {
        connect_timeout: Duration::from_secs(1),
        io_timeout: Duration::from_millis(10),
        max_frame_bytes: 1024,
    };
    let mut client =
        BridgetClient::connect_with_limits(fixture.path(), "scope-client-012", limits).unwrap();
    assert!(matches!(
        client.lookup("message-client-1"),
        Err(BridgetClientError::Timeout { .. })
    ));
    assert!(matches!(
        client.lookup("message-client-1"),
        Err(BridgetClientError::ConnectionUnusable)
    ));
    drop(client);
    server.join().expect("serveur termine");
}

#[test]
fn trame_tamponnee_apres_echeance_empoisonne_l_abonnement() {
    let fixture = SocketFixture::new("buffered-deadline");
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("connexion client attendue");
        let (mut reader, mut writer) = split(stream);
        assert_client_handshake(&mut reader, &mut writer);
        let _hello = read_json(&mut reader);
        write_welcome(&mut writer);

        let (stream, _) = listener.accept().expect("connexion attach attendue");
        let (mut reader, mut writer) = split(stream);
        assert_eq!(
            read_json(&mut reader),
            json!({"type": "RoleHandshake", "role": "attach"})
        );
        write_json(
            &mut writer,
            json!({"type": "RoleAccepted", "role": "attach"}),
        );
        assert_eq!(read_json(&mut reader)["type"], "Subscribe");

        let frames = [
            serde_json::to_string(&json!({
                "type": "SnapshotCaughtUp",
                "subscription_id": "sub-buffered",
                "through_seq": 7,
            }))
            .unwrap(),
            serde_json::to_string(&json!({
                "type": "End",
                "subscription_id": "sub-buffered",
                "reason": "fin",
            }))
            .unwrap(),
        ]
        .join("\n");
        writer.write_all(frames.as_bytes()).unwrap();
        writer.write_all(b"\n").unwrap();
        writer.flush().unwrap();
    });
    let limits = BridgetClientLimits {
        connect_timeout: Duration::from_secs(1),
        io_timeout: Duration::from_secs(1),
        max_frame_bytes: 1024,
    };
    let client =
        BridgetClient::connect_with_limits(fixture.path(), "scope-client-012", limits).unwrap();
    let mut subscription = client
        .subscribe("prospective", AttachWindow::Today)
        .unwrap();

    assert!(matches!(
        subscription
            .next_event_until(std::time::Instant::now() + Duration::from_secs(1))
            .unwrap(),
        SubscriptionEvent::SnapshotCaughtUp { .. }
    ));
    assert!(matches!(
        subscription.next_event_until(std::time::Instant::now() - Duration::from_millis(1)),
        Err(BridgetClientError::Timeout {
            operation: "lecture socket"
        })
    ));
    assert!(matches!(
        subscription.next_event_until(std::time::Instant::now() + Duration::from_secs(1)),
        Err(BridgetClientError::ConnectionUnusable)
    ));
    drop(subscription);
    server.join().expect("serveur termine");
}

#[test]
fn daemon_goutte_a_goutte_ne_renouvelle_pas_le_budget_global_de_lecture() {
    let fixture = SocketFixture::new("drip-timeout");
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("client attendu");
        let (mut reader, mut writer) = split(stream);
        assert_eq!(
            read_json(&mut reader),
            json!({"type": "RoleHandshake", "role": "client"})
        );
        let frame = format!(
            "{}\n",
            serde_json::to_string(&json!({"type": "RoleAccepted", "role": "client"}))
                .expect("frame JSON")
        );
        for byte in frame.bytes() {
            if writer.write_all(&[byte]).is_err() || writer.flush().is_err() {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
    });
    let limits = BridgetClientLimits {
        connect_timeout: Duration::from_secs(1),
        io_timeout: Duration::from_millis(25),
        max_frame_bytes: 1024,
    };

    let started = std::time::Instant::now();
    let error = match BridgetClient::connect_with_limits(fixture.path(), "scope-client-012", limits)
    {
        Ok(_) => panic!("une frame goutte-a-goutte doit expirer globalement"),
        Err(error) => error,
    };
    assert!(matches!(
        error,
        BridgetClientError::Timeout {
            operation: "lecture socket"
        }
    ));
    assert!(
        started.elapsed() < Duration::from_millis(100),
        "le budget ne doit pas etre renouvelle pour chaque octet"
    );
    server.join().expect("serveur termine");
}

#[test]
fn trame_sans_fin_de_ligne_depasse_la_borne_sans_croitre_sans_limite() {
    let fixture = SocketFixture::new("frame-limit");
    let listener = fixture.bind();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().expect("client attendu");
        let (mut reader, mut writer) = split(stream);
        assert_eq!(
            read_json(&mut reader),
            json!({"type": "RoleHandshake", "role": "client"})
        );
        writer
            .write_all(&[b'x'; 129])
            .expect("trame sans fin ecrite");
        writer.flush().expect("trame sans fin videe");
    });
    let limits = BridgetClientLimits {
        connect_timeout: Duration::from_secs(1),
        io_timeout: Duration::from_secs(1),
        max_frame_bytes: 128,
    };

    let error = match BridgetClient::connect_with_limits(fixture.path(), "scope-client-012", limits)
    {
        Ok(_) => panic!("une trame hors borne doit etre refusee"),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        BridgetClientError::FrameTooLarge {
            max_frame_bytes: 128
        }
    ));
    server.join().expect("serveur termine");
}

fn message(body: &str) -> PublicMessage {
    PublicMessage {
        id: "message-client-1".to_string(),
        from: "maicie".to_string(),
        to: "prospective".to_string(),
        body: body.to_string(),
        reply: true,
        hops: 4,
        reply_timeout: Some(60),
        deadline_at: Some(1_700_000_060),
        in_reply_to: None,
    }
}

fn assert_client_handshake(reader: &mut BufReader<UnixStream>, writer: &mut BufWriter<UnixStream>) {
    assert_eq!(
        read_json(reader),
        json!({"type": "RoleHandshake", "role": "client"})
    );
    write_json(writer, json!({"type": "RoleAccepted", "role": "client"}));
}

fn write_welcome(writer: &mut BufWriter<UnixStream>) {
    write_json(
        writer,
        json!({
            "type": "ClientWelcome",
            "version": 1,
            "horizon_secs": 3600,
            "issued_at_tolerance_secs": 30,
            "capabilities": ["send_idempotent", "lookup"]
        }),
    );
}

fn write_issue(writer: &mut BufWriter<UnixStream>, kind: &str) {
    let issue = match kind {
        "outcome_unknown" => json!({
            "kind": "outcome_unknown",
            "expires_at": 1_700_003_600,
            "delivery_id": "delivery-1"
        }),
        "accepted" => json!({"kind": "accepted", "expires_at": 1_700_003_600}),
        "envelope_mismatch" => json!({"kind": "envelope_mismatch"}),
        _ => panic!("issue inconnue"),
    };
    write_json(
        writer,
        json!({
            "type": "IdempotencyResult",
            "operation_kind": "send",
            "idempotency_key": "message-client-1",
            "issue": issue
        }),
    );
}

fn split(stream: UnixStream) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
    (
        BufReader::new(stream.try_clone().expect("clone socket")),
        BufWriter::new(stream),
    )
}

fn read_json(reader: &mut BufReader<UnixStream>) -> Value {
    let mut line = String::new();
    reader.read_line(&mut line).expect("ligne client");
    serde_json::from_str(&line).expect("JSON client")
}

fn write_json(writer: &mut BufWriter<UnixStream>, value: Value) {
    writeln!(writer, "{}", serde_json::to_string(&value).unwrap()).expect("ligne serveur");
    writer.flush().expect("flush serveur");
}

struct SocketFixture {
    path: PathBuf,
}

impl SocketFixture {
    fn new(label: &str) -> Self {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        Self {
            path: std::env::temp_dir().join(format!(
                "maicie-bridget-client-{}-{label}-{sequence}.sock",
                std::process::id()
            )),
        }
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }

    fn bind(&self) -> UnixListener {
        let _ = fs::remove_file(&self.path);
        UnixListener::bind(&self.path).expect("socket fixture")
    }
}

impl Drop for SocketFixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
