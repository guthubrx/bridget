//! Couture du délai de la sonde d'identité.
//!
//! Le pair lit la vraie première trame puis garde la connexion ouverte sans
//! répondre. Le parent borne lui-même le processus CLI afin que le mutant sans
//! délai échoue sans laisser d'enfant vivant.

use bridget_transport::protocol::{ConnectionRole, WrapperToDaemon, decode};
use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

const CLIENT_DEADLINE: Duration = Duration::from_secs(4);
const CHILD_STOP_DEADLINE: Duration = Duration::from_secs(2);

struct FixtureRoot(PathBuf);

impl FixtureRoot {
    fn new() -> Self {
        let root = PathBuf::from(format!("/tmp/bg51-{}", Uuid::new_v4()));
        bridget_daemon::environment::Namespace::resolve(
            Some(root.join("state")),
            None,
            Some(root.clone()),
        )
        .expect("namespace privé")
        .prepare()
        .expect("créer la fixture");
        Self(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn socket(&self) -> PathBuf {
        self.0.join("state/bridget.sock")
    }
}

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct ChildGuard(Option<Child>);

impl ChildGuard {
    fn spawn(home: &Path) -> Self {
        let child = Command::new(env!("CARGO_BIN_EXE_bridget"))
            .arg("status")
            .env_clear()
            .env("HOME", home)
            .env("BRIDGET_HOME", home.join("state"))
            .env("BRIDGET_SOCKET", home.join("state/bridget.sock"))
            .env("HOSTNAME", "banc-pair-muet")
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("lancer le vrai client");
        Self(Some(child))
    }

    fn child_mut(&mut self) -> &mut Child {
        self.0.as_mut().expect("enfant présent")
    }

    fn output(mut self) -> Output {
        self.0
            .take()
            .expect("enfant présent")
            .wait_with_output()
            .expect("collecter le vrai client")
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let Some(child) = self.0.as_mut() else {
            return;
        };
        if child.try_wait().ok().flatten().is_some() {
            return;
        }
        let _ = unsafe { libc::kill(child.id() as i32, libc::SIGTERM) };
        let deadline = Instant::now() + CHILD_STOP_DEADLINE;
        while Instant::now() < deadline {
            if child.try_wait().ok().flatten().is_some() {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}

struct SilentPeer {
    stop: mpsc::Sender<()>,
    join: Option<thread::JoinHandle<usize>>,
}

impl SilentPeer {
    fn start(socket: &Path) -> (Self, mpsc::Receiver<WrapperToDaemon>) {
        let listener = UnixListener::bind(socket).expect("lier la socket muette");
        listener.set_nonblocking(true).unwrap();
        let (observed_tx, observed_rx) = mpsc::channel();
        let (stop_tx, stop_rx) = mpsc::channel();
        let join = thread::spawn(move || {
            // Une mauvaise socket doit faire échouer l'oracle, jamais suspendre
            // le Drop du pair pendant une panique du parent.
            let deadline = Instant::now() + CLIENT_DEADLINE;
            let first = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(error) => panic!("accepter la sonde : {error}"),
                }
                if Instant::now() >= deadline
                    || !matches!(
                        stop_rx.recv_timeout(Duration::from_millis(10)),
                        Err(mpsc::RecvTimeoutError::Timeout)
                    )
                {
                    return 0;
                }
            };
            first.set_nonblocking(false).unwrap();
            first
                .set_read_timeout(Some(Duration::from_secs(2)))
                .expect("borner la lecture du banc");
            let mut line = String::new();
            BufReader::new(first.try_clone().expect("cloner le pair"))
                .read_line(&mut line)
                .expect("lire la trame de sonde");
            let message = decode(line.trim()).expect("décoder la vraie trame");
            observed_tx
                .send(message)
                .expect("publier la trame observée");

            listener
                .set_nonblocking(true)
                .expect("observer une éventuelle seconde connexion");
            let mut held_streams: Vec<UnixStream> = vec![first];
            loop {
                match listener.accept() {
                    Ok((stream, _)) => held_streams.push(stream),
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(error) => panic!("accepter la connexion suivante: {error}"),
                }
                if stop_rx.recv_timeout(Duration::from_millis(10)).is_ok() {
                    return held_streams.len();
                }
            }
        });
        (
            Self {
                stop: stop_tx,
                join: Some(join),
            },
            observed_rx,
        )
    }

    fn finish(mut self) -> usize {
        let _ = self.stop.send(());
        self.join
            .take()
            .expect("thread du pair présent")
            .join()
            .expect("pair muet terminé")
    }
}

impl Drop for SilentPeer {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

#[test]
fn pair_jamais_connecte_se_ferme_sans_attendre_accept() {
    let fixture = FixtureRoot::new();
    let (peer, _observed) = SilentPeer::start(&fixture.socket());
    assert_eq!(peer.finish(), 0);
}

#[test]
fn pair_accepte_et_muet_rend_l_identite_indisponible_sans_seconde_connexion() {
    let fixture = FixtureRoot::new();
    let (peer, observed) = SilentPeer::start(&fixture.socket());
    let mut client = ChildGuard::spawn(fixture.path());

    let first_message = observed
        .recv_timeout(Duration::from_secs(2))
        .expect("le pair doit réellement lire la trame du vrai client");
    assert!(matches!(
        first_message,
        WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Client
        }
    ));

    let deadline = Instant::now() + CLIENT_DEADLINE;
    let completed = loop {
        if client
            .child_mut()
            .try_wait()
            .expect("observer le vrai client")
            .is_some()
        {
            break true;
        }
        if Instant::now() >= deadline {
            break false;
        }
        thread::sleep(Duration::from_millis(10));
    };

    if !completed {
        let child = client.child_mut();
        let _ = unsafe { libc::kill(child.id() as i32, libc::SIGTERM) };
        let stop_deadline = Instant::now() + CHILD_STOP_DEADLINE;
        while Instant::now() < stop_deadline && child.try_wait().ok().flatten().is_none() {
            thread::sleep(Duration::from_millis(10));
        }
    }
    let output = client.output();
    let connections = peer.finish();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        completed,
        "le client est resté suspendu au-delà de {CLIENT_DEADLINE:?}"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "le silence doit rendre l'échec technique défini"
    );
    assert!(
        output.stdout.is_empty(),
        "aucun statut partiel ne doit être rendu comme un succès"
    );
    assert!(
        stderr.contains("identité du daemon indisponible"),
        "diagnostic incomplet: {stderr}"
    );
    assert!(
        stderr.contains("délai de lecture"),
        "le délai dépassé doit être nommé: {stderr}"
    );
    assert_eq!(
        connections, 1,
        "la collecte ne doit pas ouvrir une seconde connexion après l'expiration"
    );
}
