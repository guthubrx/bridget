//! Les façades ne transforment pas un inventaire inconnu en liste vide.
#[path = "support/idempotent.rs"]
pub mod fixture;

use bridget_transport::protocol::{CLIENT_CONTRACT_VERSION, ConnectionRole, decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn consultations_reelles_ne_creent_aucun_profil_durable() {
    let root = fixture::test_root("089-status-readonly");
    let daemon = fixture::spawn_daemon(&root, None);
    let mut observer = fixture::Client::connect(&fixture::socket(&root));
    observer.send(WrapperToDaemon::ListAgents);
    assert!(matches!(
        observer.receive(),
        DaemonToWrapper::AgentList { .. }
    ));
    let db = rusqlite::Connection::open_with_flags(
        root.join("state/bridget.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let counts = || {
        ["agent_identities", "agent_profiles"].map(|table| {
            let exists: bool = db
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            if exists {
                db.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                    row.get::<_, i64>(0)
                })
                .unwrap()
            } else {
                0
            }
        })
    };
    let before = counts();
    for args in [&["who"][..], &["agents", "--json"], &["status"]] {
        let output = fixture::run_isolated(&root, args, false);
        assert!(output.status.success(), "{}", fixture::output_text(&output));
        // Retirer l'exception éphémère de Register crée une identité et un
        // profil dès la première commande : oracle dans le vrai store maître.
        assert_eq!(counts(), before, "la lecture {args:?} a écrit un profil");
    }
    drop(daemon);
}

#[test]
fn who_agents_et_status_refusent_un_inventaire_non_atteste_sans_sortie_trompeuse() {
    for args in [&["who"][..], &["agents", "--json"], &["status"]] {
        let root = fixture::test_root("089-status");
        let listener = UnixListener::bind(fixture::socket(&root)).unwrap();
        // Parent déjà privé ; ne pas dépendre de l'umask du lanceur de tests
        // ni changer l'umask global pendant les autres scénarios parallèles.
        std::fs::set_permissions(
            fixture::socket(&root),
            std::fs::Permissions::from_mode(0o600),
        )
        .unwrap();
        listener.set_nonblocking(true).unwrap();
        let server = thread::spawn(move || {
            let accept = || {
                let deadline = Instant::now() + Duration::from_secs(5);
                loop {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            stream.set_nonblocking(false).unwrap();
                            stream
                                .set_read_timeout(Some(Duration::from_secs(3)))
                                .unwrap();
                            stream
                                .set_write_timeout(Some(Duration::from_secs(3)))
                                .unwrap();
                            break stream;
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            assert!(Instant::now() < deadline, "client absent");
                            thread::sleep(Duration::from_millis(10));
                        }
                        Err(error) => panic!("accept : {error}"),
                    }
                }
            };
            let mut stream = accept();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            for response in [
                DaemonToWrapper::RoleAccepted {
                    role: ConnectionRole::Client,
                },
                DaemonToWrapper::ClientWelcome {
                    version: CLIENT_CONTRACT_VERSION,
                    build_id: "fixture".into(),
                    horizon_secs: 60,
                    issued_at_tolerance_secs: 5,
                    capabilities: vec![],
                },
                DaemonToWrapper::DaemonIdentityReport {
                    host: "fixture".into(),
                    db_path: "/fixture".into(),
                    instance_id: "fixture".into(),
                },
            ] {
                let mut line = String::new();
                assert!(reader.read_line(&mut line).unwrap() > 0);
                decode::<WrapperToDaemon>(line.trim()).unwrap();
                writeln!(stream, "{}", encode(&response).unwrap()).unwrap();
            }
            drop(reader);
            drop(stream);
            let mut stream = accept();
            let mut line = String::new();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            reader.read_line(&mut line).unwrap();
            assert!(matches!(
                decode(line.trim()).unwrap(),
                WrapperToDaemon::Register { .. }
            ));
            // Un JSON décodable mais non corrélé, suivi d'une liste plausible :
            // ignorer Registered ou ignorer agents_inventory_available rendrait
            // une sortie de succès et ferait échouer les trois oracles CLI.
            writeln!(
                stream,
                "{}\n{}",
                encode(&DaemonToWrapper::Registered {
                    credential: None,
                    agent_id: "étranger".into()
                })
                .unwrap(),
                encode(&DaemonToWrapper::AgentList { agents: vec![] }).unwrap()
            )
            .unwrap();
            line.clear();
            let _ = reader.read_line(&mut line);
        });
        let output = fixture::run_isolated(&root, args, false);
        server
            .join()
            .unwrap_or_else(|_| panic!("serveur de {args:?}: {}", fixture::output_text(&output)));
        assert_eq!(
            output.status.code(),
            Some(1),
            "{}",
            fixture::output_text(&output)
        );
        assert!(
            output.stdout.is_empty(),
            "aucune sortie trompeuse : {args:?}"
        );
        assert!(String::from_utf8_lossy(&output.stderr).contains("inventaire indisponible"));
        assert!(!root.join("state/bridget.db").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
