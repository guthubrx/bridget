#![allow(clippy::field_reassign_with_default)]
use super::{
    CLIENT_CONTRACT_VERSION, ConnectionRole, DaemonConfig, DaemonToWrapper, WrapperToDaemon,
    encode, get_status,
};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;

/// Sert l'aller-retour d'identité complet qu'emprunte réellement
/// `get_status`, puis rend la connexion au scénario qui suit. Sans cette
/// négociation, un EOF ici ne prouverait que l'échec de la sonde préalable.
fn serve_daemon_identity(stream: UnixStream) -> UnixStream {
    serve_daemon_identity_at(stream, "fixture-host", "/fixture/status.db")
}

fn serve_daemon_identity_at(mut stream: UnixStream, host: &str, db_path: &str) -> UnixStream {
    let read_stream = stream.try_clone().expect("cloner la connexion d'identité");
    let mut reader = BufReader::new(read_stream);
    let mut line = String::new();

    assert!(reader.read_line(&mut line).expect("lire le rôle") > 0);
    assert!(matches!(
        super::decode(line.trim()).expect("décoder le rôle"),
        WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Client
        }
    ));
    writeln!(
        stream,
        "{}",
        encode(&DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        })
        .expect("encoder l'acceptation")
    )
    .expect("répondre au rôle");
    stream.flush().expect("flush du rôle");

    line.clear();
    assert!(reader.read_line(&mut line).expect("lire l'accueil") > 0);
    assert!(matches!(
        super::decode(line.trim()).expect("décoder l'accueil"),
        WrapperToDaemon::ClientHello { .. }
    ));
    writeln!(
        stream,
        "{}",
        encode(&DaemonToWrapper::ClientWelcome {
            version: CLIENT_CONTRACT_VERSION,
            build_id: "fixture-status".to_string(),
            horizon_secs: 60,
            issued_at_tolerance_secs: 5,
            capabilities: vec![],
        })
        .expect("encoder l'accueil")
    )
    .expect("répondre à l'accueil");
    stream.flush().expect("flush de l'accueil");

    line.clear();
    assert!(reader.read_line(&mut line).expect("lire la demande") > 0);
    assert!(matches!(
        super::decode(line.trim()).expect("décoder la demande"),
        WrapperToDaemon::DaemonIdentityRequest
    ));
    writeln!(
        stream,
        "{}",
        encode(&DaemonToWrapper::DaemonIdentityReport {
            host: host.to_string(),
            db_path: db_path.to_string(),
            instance_id: "fixture-instance".to_string(),
            features: Vec::new(),
        })
        .expect("encoder le rapport")
    )
    .expect("répondre au rapport");
    stream.flush().expect("flush du rapport");
    stream
}

#[test]
fn statut_refuse_un_accuse_invalide_ou_non_correle_sans_lire_la_base() {
    use std::os::unix::fs::DirBuilderExt;
    for bad_ack in [None, Some(false), Some(true)] {
        let root = std::path::PathBuf::from(format!("/tmp/b89status-{}", uuid::Uuid::new_v4()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&root)
            .unwrap();
        let socket = root.join("s");
        let db = root.join("must-not-create.db");
        let listener = UnixListener::bind(&socket).unwrap();
        let daemon_db = db.clone();
        let handle = thread::spawn(move || {
            let (first, _) = listener.accept().unwrap();
            drop(serve_daemon_identity_at(
                first,
                &crate::build_info::local_host(),
                daemon_db.to_str().unwrap(),
            ));
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(3)))
                .unwrap();
            let mut line = String::new();
            BufReader::new(stream.try_clone().unwrap())
                .read_line(&mut line)
                .unwrap();
            let WrapperToDaemon::Register { agent_id, .. } = super::decode(line.trim()).unwrap()
            else {
                panic!("Register attendu")
            };
            // Le pair malformé fournit aussi une liste plausible : l'ancien
            // client ignorait l'accusé et annonçait un inventaire valide.
            let ack = match bad_ack {
                Some(true) => "{}".to_string(),
                Some(false) => encode(&DaemonToWrapper::Registered {
                    credential: None,
                    agent_id: format!("autre-{agent_id}"),
                })
                .unwrap(),
                None => encode(&DaemonToWrapper::Registered {
                    credential: None,
                    agent_id,
                })
                .unwrap(),
            };
            writeln!(
                stream,
                "{ack}\n{}",
                encode(&DaemonToWrapper::AgentList { agents: vec![] }).unwrap()
            )
            .unwrap();
            let mut rest = String::new();
            let _ = BufReader::new(stream).read_line(&mut rest);
        });
        let mut config = DaemonConfig::default();
        config.socket_path = socket;
        config.db_path = db.clone();
        let status = get_status(&config).unwrap();
        handle.join().unwrap();
        assert!(status.running);
        assert_eq!(
            status.agents_inventory_available,
            bad_ack.is_none(),
            "validation de l'accusé"
        );
        assert!(
            !db.exists(),
            "la commande de lecture a ouvert une base locale"
        );
        assert!(status.message_count.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn inventaire_muet_consomme_le_budget_de_status_sans_devenir_vide() {
    use std::time::{Duration, Instant};
    let path = std::path::PathBuf::from(format!("/tmp/b89status-{}.sock", uuid::Uuid::new_v4()));
    let listener = UnixListener::bind(&path).unwrap();
    let (stop_tx, stop_rx) = std::sync::mpsc::channel();
    let (seen_tx, seen_rx) = std::sync::mpsc::channel();
    let handle = thread::spawn(move || {
        let (first, _) = listener.accept().unwrap();
        drop(serve_daemon_identity(first));
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let WrapperToDaemon::Register { agent_id, .. } = super::decode(line.trim()).unwrap() else {
            panic!("Register attendu")
        };
        writeln!(
            stream,
            "{}",
            encode(&DaemonToWrapper::Registered {
                credential: None,
                agent_id
            })
            .unwrap()
        )
        .unwrap();
        line.clear();
        reader.read_line(&mut line).unwrap();
        assert!(matches!(
            super::decode(line.trim()).unwrap(),
            WrapperToDaemon::ListAgents
        ));
        seen_tx.send(()).unwrap();
        // La durée est une faute du pair, pas un moyen de synchroniser le test.
        let _ = stop_rx.recv_timeout(Duration::from_secs(3));
    });
    let mut config = DaemonConfig::default();
    config.socket_path = path.clone();
    let started = Instant::now();
    let status = super::get_status_until(&config, started + Duration::from_millis(500)).unwrap();
    let elapsed = started.elapsed();
    stop_tx.send(()).unwrap();
    handle.join().unwrap();
    seen_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(status.running && !status.agents_inventory_available);
    assert!(
        elapsed < Duration::from_secs(2),
        "attente non bornée : {elapsed:?}"
    );
}

#[test]
fn sonde_identite_lente_ne_renouvelle_pas_le_budget_entre_phases() {
    use std::time::{Duration, Instant};
    let path = std::path::PathBuf::from(format!("/tmp/b89ident-{}.sock", uuid::Uuid::new_v4()));
    let listener = UnixListener::bind(&path).unwrap();
    let (seen_tx, seen_rx) = std::sync::mpsc::channel();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
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
                features: Vec::new(),
            },
        ] {
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap_or(0) == 0 {
                break;
            }
            seen_tx.send(()).unwrap();
            thread::sleep(Duration::from_millis(400));
            if writeln!(stream, "{}", encode(&response).unwrap()).is_err() {
                break;
            }
        }
    });
    let started = Instant::now();
    let result = super::daemon_identity_until(&path, started + Duration::from_secs(1));
    let elapsed = started.elapsed();
    handle.join().unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(
        seen_rx.try_iter().count(),
        3,
        "troisième phase réellement atteinte"
    );
    assert!(
        result.is_err(),
        "réinitialiser le délai accepterait les trois réponses"
    );
    assert!(elapsed < Duration::from_secs(2), "{elapsed:?}");
}

#[test]
fn socket_absente_donne_un_inventaire_vide_disponible() {
    let mut config = DaemonConfig::default();
    config.socket_path = std::env::temp_dir().join(format!(
        "bridget-no-such-socket-{}.sock",
        uuid::Uuid::new_v4()
    ));
    let status = get_status(&config).expect("l'absence déterminée est un statut");
    assert!(!status.running);
    assert!(status.agents_inventory_available);
    assert!(status.agents.is_empty());
}

#[test]
fn eof_apres_identite_et_register_garde_un_inventaire_indisponible() {
    let path = std::env::temp_dir().join(format!("bridget-eof-{}.sock", uuid::Uuid::new_v4()));
    let listener = UnixListener::bind(&path).unwrap();
    let thread_path = path.clone();
    let handle = thread::spawn(move || {
        let (first, _) = listener.accept().unwrap();
        drop(serve_daemon_identity(first));
        let (mut stream, _) = listener.accept().unwrap();
        let mut line = String::new();
        BufReader::new(stream.try_clone().unwrap())
            .read_line(&mut line)
            .unwrap();
        let WrapperToDaemon::Register { agent_id, .. } =
            super::decode(line.trim()).expect("décoder Register")
        else {
            panic!("Register attendu")
        };
        writeln!(
            stream,
            "{}",
            encode(&DaemonToWrapper::Registered {
                credential: None,
                agent_id
            })
            .unwrap()
        )
        .unwrap();
        stream.flush().unwrap();
        let mut list_line = String::new();
        BufReader::new(stream.try_clone().unwrap())
            .read_line(&mut list_line)
            .unwrap();
        assert!(matches!(
            super::decode(list_line.trim()).expect("décoder ListAgents"),
            WrapperToDaemon::ListAgents
        ));
    });
    let mut config = DaemonConfig::default();
    config.socket_path = path.clone();
    let status = get_status(&config).expect("le daemon a été attesté");
    handle.join().unwrap();
    let _ = std::fs::remove_file(thread_path);
    assert!(status.running);
    assert!(!status.agents_inventory_available);
}

#[test]
fn agent_list_vide_confirme_inventaire_disponible() {
    let path =
        std::env::temp_dir().join(format!("bridget-agent-list-{}.sock", uuid::Uuid::new_v4()));
    let listener = UnixListener::bind(&path).unwrap();
    let thread_path = path.clone();
    let handle = thread::spawn(move || {
        let (first, _) = listener.accept().unwrap();
        drop(serve_daemon_identity(first));
        let (mut stream, _) = listener.accept().unwrap();
        let mut line = String::new();
        BufReader::new(stream.try_clone().unwrap())
            .read_line(&mut line)
            .unwrap();
        let WrapperToDaemon::Register { agent_id, .. } =
            super::decode(line.trim()).expect("décoder Register")
        else {
            panic!("Register attendu")
        };
        writeln!(
            stream,
            "{}",
            encode(&DaemonToWrapper::Registered {
                credential: None,
                agent_id
            })
            .unwrap()
        )
        .unwrap();
        stream.flush().unwrap();
        let mut list_line = String::new();
        BufReader::new(stream.try_clone().unwrap())
            .read_line(&mut list_line)
            .unwrap();
        assert!(matches!(
            super::decode(list_line.trim()).expect("décoder ListAgents"),
            WrapperToDaemon::ListAgents
        ));
        writeln!(
            stream,
            "{}",
            encode(&DaemonToWrapper::AgentList { agents: vec![] }).unwrap()
        )
        .unwrap();
        stream.flush().unwrap();
    });
    let mut config = DaemonConfig::default();
    config.socket_path = path.clone();
    let status = get_status(&config).expect("la liste vide est un statut");
    handle.join().unwrap();
    let _ = std::fs::remove_file(thread_path);
    assert_eq!(
        (status.running, status.agents_inventory_available),
        (true, true)
    );
    assert!(status.agents.is_empty());
}
