//! Concurrence réelle des sockets et des connexions SQLite, sans barrière nue.
#[path = "support/idempotent.rs"]
pub mod fixture;

use bridget_daemon::idempotency::{IdempotencyKey, IdempotencyStore, OperationKind, Reservation};
use bridget_daemon::store::Store;
use bridget_transport::protocol::{decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use fixture::*;
use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

fn output(mcp: &mut McpProcess) -> Value {
    if mcp.output.buffer().is_empty() {
        let mut fd = libc::pollfd {
            fd: mcp.output.get_ref().as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        assert!(
            unsafe { libc::poll(&mut fd, 1, 5000) } > 0,
            "réponse MCP absente"
        );
    }
    let mut line = String::new();
    mcp.output.read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap()
}

#[test]
fn mcp_binaire_huit_register_coexistants_neuvieme_busy_sans_neuvieme_socket() {
    let root = test_root("concurrency-mcp");
    let listener = UnixListener::bind(socket(&root)).unwrap();
    fs::set_permissions(socket(&root), fs::Permissions::from_mode(0o600)).unwrap();
    listener.set_nonblocking(true).unwrap();
    let credential =
        bridget_transport::protocol::IdentityCredential::new("concurrent-mock-proof".into());
    save_fixture_credential(
        &socket(&root),
        ACTOR,
        "eight-real-connections",
        credential.clone(),
    );
    let (ready_tx, ready_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let server = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut connections = Vec::new();
        for _ in 0..8 {
            let stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "huit connexions non atteintes");
                        thread::sleep(Duration::from_millis(1));
                    }
                    Err(e) => panic!("{e}"),
                }
            };
            stream.set_nonblocking(false).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            stream
                .set_write_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let writer = BufWriter::new(stream);
            // L'annuaire est public : `bridget_who` interroge sans preuve ni
            // enregistrement, la première trame est directement ListAgents.
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode::<WrapperToDaemon>(line.trim_end()).unwrap(),
                WrapperToDaemon::ListAgents
            ));
            connections.push((reader, writer));
        }
        // Les HUIT sockets restent ouvertes et leurs commandes attendent :
        // aucune réponse anticipée ne libère un emplacement de concurrence.
        ready_tx.send(()).unwrap();
        release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(
            matches!(listener.accept(), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock),
            "busy ne doit créer ni neuvième socket ni Register"
        );
        for (_reader, mut writer) in connections {
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::AgentList { agents: vec![] }).unwrap()
            )
            .unwrap();
            writer.flush().unwrap();
        }
    });
    let mut mcp = McpProcess::start(&root, ACTOR, "eight-real-connections");
    mcp.request(json!({"jsonrpc":"2.0","id":0,"method":"initialize"}));
    mcp.notify(json!({"jsonrpc":"2.0","method":"notifications/initialized"}));
    for id in 1..=8 {
        mcp.notify(json!({"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":"bridget_who","arguments":{}}}));
    }
    ready_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let busy = mcp.request(json!({"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"bridget_who","arguments":{}}}));
    assert_eq!(busy["id"], 9);
    assert_eq!(busy["result"]["code"], "busy", "{busy}");
    release_tx.send(()).unwrap();
    let mut ids = Vec::new();
    for _ in 0..8 {
        let response = output(&mut mcp);
        assert_eq!(
            response["result"]["structuredContent"]["agents"],
            json!([]),
            "{response}"
        );
        ids.push(response["id"].as_u64().unwrap());
    }
    ids.sort_unstable();
    assert_eq!(ids, (1..=8).collect::<Vec<_>>());
    server.join().unwrap();
    mcp.stop();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn ouvertures_sqlite_concurrentes_et_meme_cle_reservent_un_seul_record() {
    for divergent in [false, true] {
        for cycle in 0..10 {
            let root = test_root("concurrency-sqlite");
            let database = root.join("state/bridget.db");
            let (ready_tx, ready_rx) = mpsc::channel();
            let (result_tx, result_rx) = mpsc::channel();
            let mut workers = Vec::new();
            let mut releases = Vec::new();
            for index in 0..8 {
                let (go_tx, go_rx) = mpsc::channel();
                releases.push(go_tx);
                let db = database.clone();
                let ready = ready_tx.clone();
                let result = result_tx.clone();
                workers.push(thread::spawn(move || {
                    ready.send(()).unwrap();
                    go_rx.recv_timeout(Duration::from_secs(5)).unwrap();
                    let bytes = if divergent && index % 2 == 1 {
                        b"canon-B"
                    } else {
                        b"canon-A"
                    };
                    let attempt = (|| -> Result<Reservation, String> {
                        // L'ouverture, PAS seulement le SELECT, est concurrente.
                        let store = Store::open(&db).map_err(|e| format!("Store::open: {e:?}"))?;
                        let idem = IdempotencyStore::open(&db)
                            .map_err(|e| format!("IdempotencyStore::open: {e:?}"))?;
                        let key = IdempotencyKey::new(SCOPE, OperationKind::Send, "concurrent-089")
                            .unwrap();
                        let result = idem
                            .reserve(&key, bytes, 1000, 600, 1000, 0)
                            .map_err(|e| format!("reserve: {e:?}"));
                        drop(store);
                        result
                    })();
                    result.send((bytes.to_vec(), attempt)).unwrap();
                }));
            }
            for _ in 0..8 {
                ready_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            }
            for go in releases {
                go.send(()).unwrap();
            }
            let mut results = Vec::new();
            for _ in 0..8 {
                results.push(result_rx.recv_timeout(Duration::from_secs(10)).unwrap());
            }
            for worker in workers {
                worker.join().unwrap();
            }
            let winners = results
                .iter()
                .filter(|(_, r)| matches!(r, Ok(Reservation::Prepared { .. })))
                .collect::<Vec<_>>();
            assert_eq!(
                winners.len(),
                1,
                "cycle {cycle}, divergent={divergent}: {results:?}"
            );
            let canonical = &winners[0].0;
            for (bytes, result) in &results {
                assert!(
                    matches!(result, Ok(Reservation::Prepared { .. }))
                        || (bytes == canonical && matches!(result, Ok(Reservation::Replayed(_))))
                        || (bytes != canonical
                            && matches!(result, Ok(Reservation::EnvelopeMismatch))),
                    "cycle {cycle}: {results:?}"
                );
            }
            let conn = rusqlite::Connection::open_with_flags(
                &database,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
            )
            .unwrap();
            let rows: i64 = conn
                .query_row("SELECT COUNT(*) FROM idempotency_records", [], |r| r.get(0))
                .unwrap();
            let stored: Vec<u8> = conn
                .query_row("SELECT canonical_bytes FROM idempotency_records", [], |r| {
                    r.get(0)
                })
                .unwrap();
            assert_eq!(rows, 1);
            assert_eq!(&stored, canonical);
            let rights: (String, i64) = conn
                .query_row(
                    "SELECT agent_posture, auto_reassignment FROM control_state WHERE id=1",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .unwrap();
            assert_eq!(
                rights,
                ("discovery".into(), 0),
                "une base neuve reste prudente malgré les ouvertures concurrentes"
            );
            drop(conn);
            fs::remove_dir_all(root).unwrap();
        }
    }
}

#[test]
fn huit_retries_socket_sur_daemon_reel_ne_preparent_qu_une_remise() {
    use bridget_transport::protocol::IdempotencyIssue;
    let root = test_root("concurrency-retry");
    let daemon = spawn_daemon(&root, None);
    let mut recipient = register_recipient_as(&socket(&root), "concurrent-recipient");
    let issued_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let command = idempotent_send("eight-same-key".into(), issued_at);
    let (ready_tx, ready_rx) = mpsc::channel();
    let (result_tx, result_rx) = mpsc::channel();
    let mut releases = Vec::new();
    let mut workers = Vec::new();
    for _ in 0..8 {
        let path = socket(&root);
        let command = command.clone();
        let ready = ready_tx.clone();
        let result = result_tx.clone();
        let (go_tx, go_rx) = mpsc::channel();
        releases.push(go_tx);
        workers.push(thread::spawn(move || {
            let mut client = negotiate_client(&path);
            ready.send(()).unwrap();
            go_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            client.send(command);
            result.send(client.receive()).unwrap();
        }));
    }
    for _ in 0..8 {
        ready_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    }
    for release in releases {
        release.send(()).unwrap();
    }
    for _ in 0..8 {
        assert!(matches!(
            result_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::OutcomeUnknown { .. },
                ..
            }
        ));
    }
    for worker in workers {
        worker.join().unwrap();
    }
    let delivery = receive_delivery(&mut recipient);
    match delivery {
        DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            ..
        } => {
            recipient.send(WrapperToDaemon::DeliverAcked {
                delivery_id,
                delivery_generation,
            });
        }
        _ => unreachable!(),
    }
    // Barrière après ACK sur sa connexion ; aucune autre remise intercalée.
    recipient.send(WrapperToDaemon::ListAgents);
    assert!(matches!(
        recipient.receive(),
        DaemonToWrapper::AgentList { .. }
    ));
    for _ in 0..8 {
        assert!(matches!(
            retry_command_issue(&socket(&root), command.clone()),
            IdempotencyIssue::Accepted { .. }
        ));
    }
    let db = rusqlite::Connection::open_with_flags(
        root.join("state/bridget.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    for table in ["idempotency_records", "send_deliveries", "ledger"] {
        let count: i64 = db
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1, "une seule écriture durable : {table}");
    }
    assert_no_delivery(&mut recipient);
    drop(db);
    drop(recipient);
    daemon.stop();
    fs::remove_dir_all(root).unwrap();
}
