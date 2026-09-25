//! Recette réelle isolée : journal → faits → daemon → destinataire, puis restart.
//! Réutilise les échanges privés100. Le support historique idempotent est exclu
//! car son arrêt SIGKILL de groupe est interdit. Aucun fournisseur n'est lancé.
use bridget_transport::journal::{JournalLiveFeed, JournalWriter, confirmed_write_payload};
use bridget_transport::protocol::{
    ObservationKind as Kind, ObservationRequest as Request, decode, encode,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

const SOURCE: &str = "10100000-0000-4000-8000-000000000001";
const OWNER: &str = "10100000-0000-4000-8000-000000000002";
const OTHER: &str = "10100000-0000-4000-8000-000000000003";

struct PrivateDaemon {
    child: Child,
    root: PathBuf,
}

impl PrivateDaemon {
    fn start(root: &Path) -> Self {
        let log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .mode(0o600)
            .open(root.join("daemon.log"))
            .unwrap();
        let child = Command::new(env!("CARGO_BIN_EXE_bridget"))
            .arg("daemon")
            .env_clear()
            .env("HOME", root)
            .env("BRIDGET_HOME", root)
            .env("BRIDGET_SOCKET", root.join("s"))
            .env("PATH", "/usr/bin:/bin")
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log))
            .spawn()
            .unwrap();
        let mut daemon = Self {
            child,
            root: root.into(),
        };
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            assert!(
                daemon.child.try_wait().unwrap().is_none(),
                "daemon arrêté : {}",
                std::fs::read_to_string(root.join("daemon.log")).unwrap_or_default()
            );
            if UnixStream::connect(root.join("s")).is_ok() {
                break;
            }
            assert!(Instant::now() < deadline, "daemon privé non prêt");
            std::thread::sleep(Duration::from_millis(25));
        }
        daemon
    }

    fn stop(&mut self) -> Result<(), String> {
        if self.child.try_wait().map_err(|e| e.to_string())?.is_some() {
            return Ok(());
        }
        let pid = self.child.id();
        let ps = Command::new("/bin/ps")
            .args(["-p", &pid.to_string(), "-o", "ppid=", "-o", "command="])
            .output()
            .map_err(|e| e.to_string())?;
        let command = String::from_utf8_lossy(&ps.stdout);
        if !ps.status.success()
            || command
                .split_whitespace()
                .next()
                .and_then(|p| p.parse::<u32>().ok())
                != Some(std::process::id())
            || !command.contains(&format!("{} daemon", env!("CARGO_BIN_EXE_bridget")))
            || command.to_ascii_lowercase().contains("firefox")
        {
            return Err(format!("PID {pid} non attesté, aucun signal envoyé"));
        }
        // PID unique, enfant direct vérifié ; jamais de groupe ni SIGKILL.
        if unsafe { libc::kill(pid as i32, libc::SIGTERM) } != 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        std::thread::sleep(Duration::from_secs(3));
        let deadline = Instant::now() + Duration::from_secs(7);
        while self.child.try_wait().map_err(|e| e.to_string())?.is_none() {
            if Instant::now() >= deadline {
                return Err(format!("PID {pid} toujours vivant, aucun arrêt forcé"));
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        Ok(())
    }
}

impl Drop for PrivateDaemon {
    fn drop(&mut self) {
        if let Err(error) = self.stop() {
            eprintln!("{} : {error}", self.root.display());
        }
    }
}

struct Peer {
    writer: UnixStream,
    reader: BufReader<UnixStream>,
}

impl Peer {
    fn register(root: &Path, id: &str, instance: &str) -> Self {
        let writer = UnixStream::connect(root.join("s")).unwrap();
        writer
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        writer
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut peer = Self {
            reader: BufReader::new(writer.try_clone().unwrap()),
            writer,
        };
        peer.send(decode(&json!({"type":"Register","identity_version":2,"agent_type":"fixture","agent_id":id,
            "instance_id":instance,"host":"private-spec101","transport":"acp","mode":"acp","journal_available":false}).to_string()).unwrap());
        let DaemonToWrapper::Registered {
            agent_id,
            credential: Some(credential),
        } = peer.read()
        else {
            panic!("identité primaire attestée requise")
        };
        assert_eq!(agent_id, id);
        if id == OWNER {
            bridget_transport::fsutil::write_private_file_atomic(&root.join("name"), id.as_bytes())
                .unwrap();
            let proof = root.join("agent-names").join(format!(
                "proof-{:x}.json",
                Sha256::digest(instance.as_bytes())
            ));
            bridget_transport::fsutil::write_private_file_atomic(
                &proof,
                &serde_json::to_vec(
                    &json!({"agent_id":id,"instance_id":instance,"credential":credential}),
                )
                .unwrap(),
            )
            .unwrap();
        }
        peer
    }

    fn send(&mut self, message: WrapperToDaemon) {
        writeln!(self.writer, "{}", encode(&message).unwrap()).unwrap();
        self.writer.flush().unwrap();
    }

    fn read(&mut self) -> DaemonToWrapper {
        let mut line = String::new();
        assert!(
            self.reader.read_line(&mut line).unwrap() > 0,
            "socket fermée"
        );
        decode(line.trim()).unwrap()
    }

    fn events(&mut self, request: Request) -> Value {
        self.send(WrapperToDaemon::ObservationRequest { request });
        let DaemonToWrapper::ObservationResult { result } = self.read() else {
            panic!("reçu observation attendu")
        };
        result
    }

    fn delivered(&mut self, needle: &str) -> bridget_core::BridgetMessage {
        let DaemonToWrapper::Deliver(message) = self.read() else {
            panic!("notification attendue")
        };
        assert!(message.body.contains(needle), "{}", message.body);
        message
    }

    fn barrier(&mut self) {
        self.send(WrapperToDaemon::ListAgents);
        assert!(matches!(self.read(), DaemonToWrapper::AgentList { .. }));
    }
}

fn sub(event: Kind, agent: Option<&str>, once: bool) -> Request {
    Request::Sub {
        event,
        agent: agent.map(str::to_string),
        file: None,
        once,
        ttl_secs: Some(120),
    }
}

// Même pompage que AttachRelayWorker, qui reste privé au module wrapper.
// Le journal, sa validation, son flush et sa conversion sont réels.
fn forward_journal(
    root: &Path,
    agent: &str,
    peer: &mut Peer,
    event: &str,
    message: &str,
    payload: Value,
) -> Vec<u8> {
    let feed = JournalLiveFeed::default();
    let journal = JournalWriter::start_with_live_feed_and_failure(
        root.join("journals"),
        agent,
        "spec101",
        Arc::new(|detail| panic!("journal : {detail}")),
        Some(feed.clone()),
    )
    .unwrap();
    journal.enqueue(event, Some(message), payload).unwrap();
    journal.stop();
    let (facts, lost) = feed.take_observations();
    assert_eq!(lost, 0);
    assert_eq!(
        facts.len(),
        usize::from(!message.starts_with("bridget-observation:"))
    );
    for fact in facts {
        peer.send(fact);
    }
    peer.barrier();
    let entry = std::fs::read_dir(root.join("journals").join(agent))
        .unwrap()
        .find_map(|entry| {
            let path = entry.unwrap().path();
            (path.extension().is_some_and(|s| s == "jsonl")).then_some(path)
        })
        .unwrap();
    std::fs::read(entry)
        .unwrap()
        .split(|b| *b == b'\n')
        .find(|line| !line.is_empty())
        .unwrap()
        .to_vec()
}

#[test]
fn spec101_real_daemon_journal_share_collision_and_restart() {
    let root = PathBuf::from("/tmp").join(format!("bg101-{}", uuid::Uuid::new_v4().simple()));
    bridget_transport::fsutil::create_private_dir(&root).unwrap();
    let mut daemon = PrivateDaemon::start(&root);
    let mut source = Peer::register(&root, SOURCE, "source101");
    let mut owner = Peer::register(&root, OWNER, "owner101");
    let mut other = Peer::register(&root, OTHER, "other101");
    assert_eq!(
        owner.events(sub(Kind::TurnEnded, Some("unknown"), true))["reason"],
        "agent_not_found"
    );
    assert_eq!(
        owner.events(sub(Kind::TurnEnded, Some(SOURCE), true))["reason"],
        "no_compatible_source"
    );
    for peer in [&mut source, &mut other] {
        peer.send(WrapperToDaemon::ObservationCapabilities {
            events: vec![Kind::TurnEnded, Kind::FileWritten],
        });
        peer.barrier();
    }
    source.send(WrapperToDaemon::JournalReady);
    source.barrier();
    let types = owner.events(Request::Types {});
    assert_eq!(types["events"][0]["available"], true);
    assert_eq!(types["events"][1]["available"], false);
    assert_eq!(
        owner.events(sub(Kind::TurnEnded, Some(SOURCE), true))["status"],
        "subscribed"
    );
    let excerpt = forward_journal(
        &root,
        SOURCE,
        &mut source,
        "turn_end",
        "human-turn",
        json!({"stop_reason":"completed","text":"preuve réelle 🦀"}),
    );
    let notice = owner.delivered("turn_ended");
    assert!(notice.id.starts_with("bridget-observation:"));
    assert_eq!(notice.origin, Some(bridget_core::MessageOrigin::System));
    assert!(!notice.reply);
    assert!(
        owner.events(Request::List {})["subscriptions"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let collision = owner.events(sub(Kind::FileCollision, None, true));
    assert_eq!(collision["status"], "subscribed");
    forward_journal(
        &root,
        SOURCE,
        &mut source,
        "update",
        "write-a",
        confirmed_write_payload("/project/a.rs", Path::new("/project")).unwrap(),
    );
    forward_journal(
        &root,
        OTHER,
        &mut other,
        "update",
        "write-b",
        confirmed_write_payload("/project/./a.rs", Path::new("/project")).unwrap(),
    );
    let notice = owner.delivered("file_collision");
    assert!(notice.body.contains(SOURCE) && notice.body.contains(OTHER));
    assert!(notice.body.contains("/project/a.rs"));

    // Le serveur wrapper répond à Subscribe avec les octets réellement flushés.
    let replay = std::thread::spawn(move || {
        let DaemonToWrapper::Subscribe {
            subscription_id, ..
        } = source.read()
        else {
            panic!("lecture journal attendue")
        };
        source.send(WrapperToDaemon::Subscribed {
            subscription_id: subscription_id.clone(),
        });
        source.send(WrapperToDaemon::JournalFragment {
            subscription_id: subscription_id.clone(),
            seq: 1,
            offset: 0,
            final_fragment: true,
            bytes: excerpt,
        });
        source.send(WrapperToDaemon::SnapshotCaughtUp {
            subscription_id,
            through_seq: Some(1),
        });
        source
    });
    let shared = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .args(["journal", SOURCE, "--to", OTHER, "--reply"])
        .env_clear()
        .env("HOME", &root)
        .env("BRIDGET_HOME", &root)
        .env("BRIDGET_SOCKET", root.join("s"))
        .env("BRIDGET_AGENT_ID_FILE", root.join("name"))
        .env("BRIDGET_AGENT_INSTANCE_ID", "owner101")
        .output()
        .unwrap();
    assert!(
        shared.status.success(),
        "{}",
        String::from_utf8_lossy(&shared.stderr)
    );
    let mut source = replay.join().unwrap();
    let shared = other.delivered("preuve réelle 🦀");
    assert!(shared.reply && shared.body.contains(SOURCE));
    assert_eq!(shared.from, OWNER);
    let mut reply = bridget_core::BridgetMessage::new(OTHER, OWNER, "extrait relu");
    reply.in_reply_to = Some(shared.id);
    other.send(WrapperToDaemon::Send(reply));
    assert!(matches!(other.read(), DaemonToWrapper::Ack { .. }));
    owner.delivered("extrait relu");

    let persistent = owner.events(sub(Kind::TurnEnded, Some(SOURCE), false));
    let subscription = persistent["subscription"]["id"].as_str().unwrap();
    // CLI ferme sa vue : absorber son Unsubscribe avant la barrière suivante.
    source.send(WrapperToDaemon::ObservationCapabilities { events: vec![] });
    loop {
        if matches!(source.read(), DaemonToWrapper::Unsubscribe { .. }) {
            break;
        }
    }
    source.barrier();
    // Session 119 : l'état change aussitôt ; l'avis attend 30 s de stabilité
    // et un aller-retour bref n'en produit aucun.
    assert_eq!(
        owner.events(Request::List {})["subscriptions"][0]["state"],
        "source_unavailable"
    );
    source.send(WrapperToDaemon::ObservationCapabilities {
        events: vec![Kind::TurnEnded, Kind::FileWritten],
    });
    source.barrier();
    assert_eq!(
        owner.events(Request::List {})["subscriptions"][0]["state"],
        "active"
    );
    // Une fin provoquée par une notification ne produit aucun fait observable.
    forward_journal(
        &root,
        SOURCE,
        &mut source,
        "turn_end",
        "bridget-observation:test",
        json!({"stop_reason":"completed"}),
    );
    assert_eq!(
        owner.events(Request::List {})["subscriptions"][0]["state"],
        "active"
    );
    let original_instance = persistent["daemon_instance"].clone();
    drop((source, owner, other));
    daemon.stop().unwrap();

    let mut restarted = PrivateDaemon::start(&root);
    let mut owner = Peer::register(&root, OWNER, "owner101");
    // Session 122 : l'abonnement est repris sans nouvel abonnement.
    owner.delivered("repris automatiquement");
    let restored = owner.events(Request::List {});
    assert_ne!(restored["daemon_instance"], original_instance);
    assert_eq!(restored["subscriptions"][0]["id"], subscription);
    assert_eq!(restored["subscriptions"][0]["state"], "source_unavailable");
    let mut source = Peer::register(&root, SOURCE, "source101");
    source.send(WrapperToDaemon::ObservationCapabilities {
        events: vec![Kind::TurnEnded],
    });
    source.barrier();
    assert_eq!(
        owner.events(Request::List {})["subscriptions"][0]["state"],
        "active"
    );
    forward_journal(
        &root,
        SOURCE,
        &mut source,
        "turn_end",
        "after-restart",
        json!({"stop_reason":"completed"}),
    );
    owner.delivered("turn_ended");
    assert_eq!(
        owner.events(Request::Unsub {
            id: subscription.into()
        })["status"],
        "unsubscribed"
    );
    assert!(
        owner.events(Request::List {})["subscriptions"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    drop((source, owner));
    restarted.stop().unwrap();
}
