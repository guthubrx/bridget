//! Frontière162 : vrai CLI worker, bootstrap et wrapper ; fournisseur fermé.
//! Sous-module colocalisé pour réemployer les processus et répertoires148/152.
//! Le relais ne simule que le maître worker ; le protocole natif va au vrai daemon.
//! Le quatrième cas retire ce relais et utilise l'admission NativeDelegate réelle.
//! L'unicité observée porte sur PID/naissance, reçu et compteur fournisseur.
//! Elle ne compte pas un éventuel bootstrap éphémère invisible entre deux lectures.

use super::*;
use bridget_daemon::remote_worker::{WorkerCommandReceipt, worker_command_digest};
use bridget_transport::protocol::*;
use sha2::{Digest, Sha256};
use std::io::{self, BufRead, BufReader, Write};
use std::net::Shutdown;
use std::os::fd::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};

const SECRET: &str = "fixture-worker162-private";
const ORIGIN: &str = "/origin162/project";
const REPOSITORY: &str = "fixture162:project";
const PROVIDER: &str = r#"import json
import os
import sys
import time
evidence = os.path.join(os.path.dirname(os.path.abspath(__file__)), "evidence.jsonl")
instance = os.environ["BRIDGET_AGENT_INSTANCE_ID"]
assert "BRIDGET_REMOTE_RESULT_CONTEXT" not in os.environ
assert not any(key in os.environ for key in ("OPENAI_API_KEY", "ANTHROPIC_API_KEY", "T3CODE_HOME"))
with open(evidence, "a", encoding="utf-8") as output:
    output.write(json.dumps(dict(event="started162", instance_id=instance, pid=os.getpid())) + "\n")
if os.path.exists(os.path.join(os.path.dirname(evidence), "fail-init162")):
    deadline = time.monotonic() + 15
    while not os.path.exists(os.path.join(os.path.dirname(evidence), "release-failure162")):
        if time.monotonic() >= deadline:
            raise RuntimeError("recette162: porte d'échec non ouverte")
        time.sleep(0.02)
    with open(evidence, "a", encoding="utf-8") as output:
        output.write(json.dumps(dict(event="failed-init162", instance_id=instance, exit_code=23)) + "\n")
    sys.exit(23)
if os.path.exists(os.path.join(os.path.dirname(evidence), "detached162")):
    import signal
    if os.fork() == 0:
        os.setsid()
        if os.fork() != 0:
            os._exit(0)
        os.setsid()
        mode = open(os.path.join(os.path.dirname(evidence), "detached162")).read()
        signal.signal(signal.SIGTERM, signal.SIG_DFL if mode == "normal" else signal.SIG_IGN)
        with open(evidence, "a", encoding="utf-8") as output:
            output.write(json.dumps(dict(event="detached162", instance_id=instance, pid=os.getpid(), pgid=os.getpgrp())) + "\n")
        # stdout hérité reste ouvert : le reader du transport attendrait EOF.
        deadline = time.monotonic() + 30
        while not os.path.exists(os.path.join(os.path.dirname(evidence), "release-detached162")):
            if time.monotonic() >= deadline:
                os._exit(24)
            time.sleep(.02)
        os._exit(0)
def emit(frame):
    print(json.dumps(frame), flush=True)
selected_model = None
for line in sys.stdin:
    frame = json.loads(line)
    method = frame.get("method")
    if method == "initialize":
        emit(dict(id=frame["id"], result=dict(userAgent="fixture162", codexHome=os.environ["HOME"], platformFamily="unix", platformOs=sys.platform)))
    elif method == "thread/start":
        params = frame["params"]
        selected_model = params.get("model")
        emit(dict(id=frame["id"], result=dict(thread=dict(id="thread162-" + instance), model="gpt-5.6-terra", reasoningEffort="high",
            approvalPolicy=params.get("approvalPolicy", "never"), approvalsReviewer="user", sandboxPolicy=params.get("sandboxPolicy", dict(type="readOnly", networkAccess=False)))))
    elif method == "account/rateLimits/read":
        emit(dict(id=frame["id"], result=dict(rateLimits={})))
    elif method == "turn/start":
        params = frame["params"]
        assert "NATIVE162_NOMINAL" in json.dumps(params["input"])
        turn = "turn162-" + instance
        with open(evidence, "a", encoding="utf-8") as output:
            output.write(json.dumps(dict(event="turn162", instance_id=instance, pid=os.getpid(), ppid=os.getppid(), pgid=os.getpgrp(), cwd=os.getcwd(), model=selected_model, params=params)) + "\n")
        emit(dict(id=frame["id"], result=dict(turn=dict(id=turn), approvalPolicy="never", approvalsReviewer="user", sandboxPolicy=dict(type="readOnly", networkAccess=False))))
        deadline = time.monotonic() + 30
        while not os.path.exists(os.path.join(os.getcwd(), "release162")):
            if time.monotonic() >= deadline:
                raise RuntimeError("recette162: porte de résultat non ouverte")
            time.sleep(0.02)
        emit(dict(method="item/agentMessage/delta", params=dict(threadId=params["threadId"], turnId=turn, itemId="answer162", delta="fixture162-remote-answer")))
        emit(dict(method="turn/completed", params=dict(threadId=params["threadId"], turn=dict(id=turn, status="completed", items=[]))))
    elif method == "turn/interrupt":
        emit(dict(id=frame["id"], result={}))
"#;

fn identity() -> WorkerIdentity {
    WorkerIdentity {
        machine_id: "cartae162".into(),
        worker_id: "worker162".into(),
        provisioning_revision: 1,
    }
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn send_frame(stream: &mut UnixStream, frame: &impl serde::Serialize) -> io::Result<()> {
    serde_json::to_writer(&mut *stream, frame).map_err(io::Error::other)?;
    stream.write_all(b"\n")
}

type WorkerConnection = (BufReader<UnixStream>, WorkerHello, bool);

// Relais privé de cette seule recette. Les connexions déjà classées passent
// ensuite octet pour octet ; aucune réécriture des faits de permission natifs.
struct Relay {
    stop: Arc<AtomicBool>,
    sockets: Arc<Mutex<Vec<UnixStream>>>,
    thread: Option<thread::JoinHandle<()>>,
    workers: mpsc::Receiver<WorkerConnection>,
    ready_delayed: Arc<AtomicBool>,
}

impl Relay {
    fn new(front: &Path, daemon: PathBuf) -> Self {
        Self::with_worker_capability(front, daemon, Duration::ZERO, false)
    }
    fn with_ready_delay(front: &Path, daemon: PathBuf, ready_delay: Duration) -> Self {
        Self::with_worker_capability(front, daemon, ready_delay, false)
    }
    /// Force l'annonce de capacité sur ce relais de recette, sans modifier le
    /// daemon réel ni son build. Le worker doit négocier l'enveloppe distincte avant
    /// de recevoir le probe.
    fn with_candidate_probe(front: &Path, daemon: PathBuf) -> Self {
        Self::with_worker_capability(front, daemon, Duration::ZERO, true)
    }
    fn with_worker_capability(
        front: &Path,
        daemon: PathBuf,
        ready_delay: Duration,
        candidate_probe_feature: bool,
    ) -> Self {
        let listener = UnixListener::bind(front).unwrap();
        fs::set_permissions(front, fs::Permissions::from_mode(0o600)).unwrap();
        listener.set_nonblocking(true).unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let sockets = Arc::new(Mutex::new(Vec::<UnixStream>::new()));
        let (sender, workers) = mpsc::channel();
        let ready_delayed = Arc::new(AtomicBool::new(false));
        let delayed = ready_delayed.clone();
        let stopped = stop.clone();
        let tracked = sockets.clone();
        let thread = thread::spawn(move || {
            let mut connections = Vec::new();
            while !stopped.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((client, _)) => {
                        tracked.lock().unwrap().push(client.try_clone().unwrap());
                        let daemon = daemon.clone();
                        let sender = sender.clone();
                        let tracked = tracked.clone();
                        let delayed = delayed.clone();
                        connections.push(thread::spawn(move || {
                            let started = Instant::now();
                            let mut stage = "accept";
                            eprintln!("relais162 fd client flags={}", unsafe {
                                libc::fcntl(client.as_raw_fd(), libc::F_GETFL)
                            });
                            let outcome = (|| -> io::Result<()> {
                                stage = "configure client";
                                // Sur la cible macOS observée, accept hérite
                                // O_NONBLOCK du listener. Ce lecteur de fixture
                                // utilise des lectures bloquantes sous timeout.
                                client.set_nonblocking(false)?;
                                eprintln!("relais162 fd client blocking flags={}", unsafe {
                                    libc::fcntl(client.as_raw_fd(), libc::F_GETFL)
                                });
                                client.set_read_timeout(Some(Duration::from_secs(10)))?;
                                let mut reader = BufReader::new(client);
                                stage = "connect backend";
                                let mut backend = UnixStream::connect(daemon)?;
                                eprintln!("relais162 fd backend flags={}", unsafe {
                                    libc::fcntl(backend.as_raw_fd(), libc::F_GETFL)
                                });
                                tracked.lock().unwrap().push(backend.try_clone()?);
                                let mut first = String::new();
                                stage = "read first client frame";
                                if reader.read_line(&mut first)? == 0 {
                                    return Ok(());
                                }
                                eprintln!(
                                    "relais162 first frame bytes={} identity_probe={}",
                                    first.len(),
                                    matches!(
                                        serde_json::from_str::<WrapperToDaemon>(&first),
                                        Ok(WrapperToDaemon::DaemonIdentityRequest)
                                    )
                                );
                                if matches!(
                                    serde_json::from_str::<WrapperToDaemon>(&first),
                                    Ok(WrapperToDaemon::DaemonIdentityRequest)
                                ) {
                                    stage = "write probe backend";
                                    backend.write_all(first.as_bytes())?;
                                    let mut backend_reader = BufReader::new(backend.try_clone()?);
                                    backend_reader
                                        .get_mut()
                                        .set_read_timeout(Some(Duration::from_secs(10)))?;
                                    let mut report = String::new();
                                    stage = "read identity backend";
                                    backend_reader.read_line(&mut report)?;
                                    eprintln!("relais162 identity response bytes={}", report.len());
                                    stage = "write identity client";
                                    if candidate_probe_feature {
                                        let mut identity = serde_json::from_str::<DaemonToWrapper>(&report)
                                            .map_err(io::Error::other)?;
                                        let DaemonToWrapper::DaemonIdentityReport { features, .. } = &mut identity else {
                                            return Err(io::Error::other("identité maître inattendue"));
                                        };
                                        if !features.iter().any(|feature| feature == "worker_candidate_probe_v1") {
                                            features.push("worker_candidate_probe_v1".into());
                                        }
                                        send_frame(reader.get_mut(), &identity)?;
                                    } else {
                                        reader.get_mut().write_all(report.as_bytes())?;
                                    }
                                    first.clear();
                                    stage = "read second client frame";
                                    if reader.read_line(&mut first)? == 0 {
                                        return Ok(());
                                    }
                                    eprintln!(
                                        "relais162 second frame bytes={} worker_hello={}",
                                        first.len(),
                                        matches!(
                                            serde_json::from_str::<WrapperToDaemon>(&first),
                                            Ok(WrapperToDaemon::WorkerHello { .. })
                                        )
                                    );
                                }
                                let worker = match serde_json::from_str::<WrapperToDaemon>(&first) {
                                    Ok(WrapperToDaemon::WorkerHello { hello }) => Some((hello, false)),
                                    Ok(WrapperToDaemon::WorkerHelloWithCapabilities { hello })
                                        if hello.capabilities == vec![WorkerCapability::CandidateProbeV1] =>
                                    {
                                        Some((hello.hello, true))
                                    }
                                    _ => None,
                                };
                                let worker_connection = worker.is_some();
                                if let Some((hello, candidate_probe)) = worker
                                    && ready_delay.is_zero()
                                {
                                    sender
                                        .send((reader, hello, candidate_probe))
                                        .map_err(|_| io::Error::other("maître de recette fermé"))?;
                                    return Ok(());
                                }
                                backend.set_read_timeout(None)?;
                                reader.get_mut().set_read_timeout(None)?;
                                backend.write_all(first.as_bytes())?;
                                let mut back_reader = backend.try_clone()?;
                                let mut client_writer = reader.get_ref().try_clone()?;
                                let reverse = thread::spawn(move || {
                                    let _ = io::copy(&mut back_reader, &mut client_writer);
                                    let _ = client_writer.shutdown(Shutdown::Both);
                                });
                                if worker_connection && !ready_delay.is_zero() {
                                    // Seule la première Ready réelle est retardée. Les octets,
                                    // identités et autres réponses restent inchangés.
                                    loop {
                                        let mut frame = String::new();
                                        match reader.read_line(&mut frame) {
                                            Ok(0) | Err(_) => break,
                                            Ok(_) => {}
                                        }
                                        if matches!(serde_json::from_str::<WrapperToDaemon>(&frame),
                                            Ok(WrapperToDaemon::WorkerResponse { response: WorkerResponse { response: WorkerResponseOperation::ReadyToRelease { .. }, .. } }))
                                            && !delayed.swap(true, Ordering::AcqRel)
                                        {
                                            thread::sleep(ready_delay);
                                        }
                                        if backend.write_all(frame.as_bytes()).is_err() {
                                            break;
                                        }
                                    }
                                } else {
                                    let _ = io::copy(&mut reader, &mut backend);
                                }
                                let _ = backend.shutdown(Shutdown::Both);
                                let _ = reverse.join();
                                Ok(())
                            })();
                            // EOF et arrêt de fixture ferment normalement les deux sens.
                            if let Err(error) = outcome {
                                eprintln!(
                                    "relais162 terminé stage={stage} elapsed_ms={}: {error}",
                                    started.elapsed().as_millis()
                                );
                            }
                        }));
                    }
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10))
                    }
                    Err(error) => panic!("accept relais162: {error}"),
                }
            }
            for socket in tracked.lock().unwrap().iter() {
                let _ = socket.shutdown(Shutdown::Both);
            }
            for connection in connections {
                let _ = connection.join();
            }
        });
        Self {
            stop,
            sockets,
            thread: Some(thread),
            workers,
            ready_delayed,
        }
    }

    fn connect(&self) -> Control {
        let (mut reader, hello, candidate_probe) = self
            .workers
            .recv_timeout(Duration::from_secs(12))
            .expect("WorkerHello réel absent");
        assert_eq!(hello.identity, identity());
        assert_eq!(hello.credential.expose_for_attestation(), SECRET);
        hello.validate().unwrap();
        let epoch = uuid::Uuid::new_v4().to_string();
        let accepted = WorkerAccepted {
            version: REMOTE_WORKER_VERSION,
            identity: identity(),
            connection_epoch: epoch.clone(),
        };
        let response = if candidate_probe {
            DaemonToWrapper::WorkerAcceptedWithCapabilities {
                accepted: WorkerAcceptedWithCapabilities {
                    accepted,
                    capabilities: vec![WorkerCapability::CandidateProbeV1],
                },
            }
        } else {
            DaemonToWrapper::WorkerAccepted { accepted }
        };
        send_frame(reader.get_mut(), &response).unwrap();
        Control {
            reader,
            epoch,
            candidate_probe,
        }
    }
}

impl Drop for Relay {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        for socket in self.sockets.lock().unwrap().iter() {
            let _ = socket.shutdown(Shutdown::Both);
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

struct Control {
    reader: BufReader<UnixStream>,
    epoch: String,
    candidate_probe: bool,
}
impl Control {
    fn request(
        &mut self,
        command: &WorkerCommandIdentity,
        operation: WorkerRequestOperation,
    ) -> WorkerResponseOperation {
        let request = WorkerRequest {
            version: REMOTE_WORKER_VERSION,
            correlation_id: uuid::Uuid::new_v4().to_string(),
            connection_epoch: self.epoch.clone(),
            worker: identity(),
            command: command.clone(),
            request: operation,
        };
        request.validate().unwrap();
        send_frame(
            self.reader.get_mut(),
            &DaemonToWrapper::WorkerRequest {
                request: request.clone(),
            },
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(12);
        loop {
            assert!(Instant::now() < deadline, "réponse worker162 absente");
            self.reader
                .get_mut()
                .set_read_timeout(Some(deadline.saturating_duration_since(Instant::now())))
                .unwrap();
            let mut line = String::new();
            assert!(
                self.reader.read_line(&mut line).unwrap() > 0,
                "worker162 a fermé sa connexion"
            );
            match serde_json::from_str::<WrapperToDaemon>(&line).unwrap() {
                WrapperToDaemon::Heartbeat => {}
                WrapperToDaemon::WorkerResponse { response } => {
                    response.validate().unwrap();
                    assert_eq!(response.worker, identity());
                    assert_eq!(response.connection_epoch, self.epoch);
                    if response.correlation_id == request.correlation_id {
                        assert_eq!(response.command, *command);
                        return response.response;
                    }
                }
                other => panic!("trame worker162 inattendue: {other:?}"),
            }
        }
    }
    fn candidate_probe(&mut self, probe: CandidateProbe) -> CandidateProbeResult {
        assert!(
            self.candidate_probe,
            "capacité candidate_probe non négociée"
        );
        assert_eq!(probe.connection_epoch, self.epoch);
        assert_eq!(probe.worker, identity());
        probe.validate_shape().unwrap();
        send_frame(
            self.reader.get_mut(),
            &DaemonToWrapper::CandidateProbe {
                probe: probe.clone(),
            },
        )
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(12);
        loop {
            assert!(Instant::now() < deadline, "résultat probe163 absent");
            self.reader
                .get_mut()
                .set_read_timeout(Some(deadline.saturating_duration_since(Instant::now())))
                .unwrap();
            let mut line = String::new();
            assert!(
                self.reader.read_line(&mut line).unwrap() > 0,
                "worker163 a fermé sa connexion pendant probe"
            );
            match serde_json::from_str::<WrapperToDaemon>(&line).unwrap() {
                WrapperToDaemon::Heartbeat => {}
                WrapperToDaemon::CandidateProbeResult { result }
                    if result.correlation_id == probe.correlation_id =>
                {
                    assert_eq!(result.connection_epoch, self.epoch);
                    assert_eq!(result.worker, identity());
                    assert_eq!(result.admission_id, probe.admission_id);
                    assert_eq!(result.policy_revision, probe.policy_revision);
                    return result;
                }
                other => panic!("trame probe163 inattendue: {other:?}"),
            }
        }
    }
    fn phase(&mut self, command: &WorkerCommandIdentity) -> WorkerCommandPhase {
        match self.request(command, WorkerRequestOperation::Reconcile {}) {
            WorkerResponseOperation::Observation { observation } => observation.phase,
            other => panic!("observation worker162 absente: {other:?}"),
        }
    }
    fn await_phase(&mut self, command: &WorkerCommandIdentity, expected: WorkerCommandPhase) {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            let phase = self.phase(command);
            if phase == expected {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "phase162 {phase:?}, attendue {expected:?}"
            );
            thread::sleep(Duration::from_millis(25));
        }
    }
    fn await_native_ready(&mut self, command: &WorkerCommandIdentity) {
        let deadline = Instant::now() + Duration::from_secs(12);
        loop {
            let observation = match self.request(command, WorkerRequestOperation::Reconcile {}) {
                WorkerResponseOperation::Observation { observation } => observation,
                other => panic!("observation native162 absente: {other:?}"),
            };
            if observation.phase == WorkerCommandPhase::Released && observation.native_ready {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "initialisation native162: {observation:?}"
            );
            assert!(
                !matches!(
                    observation.phase,
                    WorkerCommandPhase::Terminal | WorkerCommandPhase::Refused
                ),
                "échec natif162: {observation:?}"
            );
            thread::sleep(Duration::from_millis(25));
        }
    }
    fn disconnect(self) {
        let _ = self.reader.get_ref().shutdown(Shutdown::Both);
    }
    fn final_result(&mut self, command: &WorkerCommandIdentity) -> RemoteResultEnvelope {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            assert!(
                Instant::now() < deadline,
                "blob162 non réémis par le worker"
            );
            self.reader
                .get_mut()
                .set_read_timeout(Some(deadline.saturating_duration_since(Instant::now())))
                .unwrap();
            let mut line = String::new();
            assert!(self.reader.read_line(&mut line).unwrap() > 0);
            if let WrapperToDaemon::WorkerResponse { response } =
                serde_json::from_str(&line).unwrap()
            {
                response.validate().unwrap();
                assert_eq!(response.worker, identity());
                assert_eq!(response.connection_epoch, self.epoch);
                if let WorkerResponseOperation::FinalResult { result } = response.response {
                    assert_eq!(response.command, *command);
                    assert_eq!(result.identity.command, *command);
                    return result;
                }
            }
        }
    }
    fn no_final_result(&mut self) {
        // Le relais worker parcourt les résultats toutes les deux secondes.
        let deadline = Instant::now() + Duration::from_millis(2300);
        while Instant::now() < deadline {
            self.reader
                .get_mut()
                .set_read_timeout(Some(deadline.saturating_duration_since(Instant::now())))
                .unwrap();
            let mut line = String::new();
            match self.reader.read_line(&mut line) {
                Ok(0) => panic!("worker162 déconnecté pendant preuve de silence"),
                Ok(_) => {
                    let frame: WrapperToDaemon = serde_json::from_str(&line).unwrap();
                    assert!(
                        !matches!(
                            frame,
                            WrapperToDaemon::WorkerResponse {
                                response: WorkerResponse {
                                    response: WorkerResponseOperation::FinalResult { .. },
                                    ..
                                }
                            }
                        ),
                        "résultat confirmé réémis après reprise"
                    );
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                    ) => {}
                Err(error) => panic!("lecture worker162: {error}"),
            }
        }
    }
}

struct RemoteFixture {
    worker: Option<OwnedChild>,
    relay: Option<Relay>,
    fixture: Fixture,
    front: PathBuf,
    worker_state: PathBuf,
    credential: PathBuf,
    launches: usize,
}

impl RemoteFixture {
    fn new() -> Self {
        Self::with_candidate_probe(false)
    }
    fn new_with_candidate_probe() -> Self {
        Self::with_candidate_probe(true)
    }
    fn with_candidate_probe(candidate_probe: bool) -> Self {
        let fixture = Fixture::new_with_provider(60, false, Some(PROVIDER), false);
        let root = &fixture.root;
        // Épingler uniquement Git depuis le runner, sans hériter tout son PATH.
        // Le worker garde son environnement privé et la garde réelle >= 2.44.
        let runner_path = std::env::var_os("PATH").expect("fixture162: PATH du runner absent");
        let git = std::env::split_paths(&runner_path)
            .map(|directory| directory.join("git"))
            .find(|path| {
                path.metadata().is_ok_and(|metadata| {
                    metadata.is_file()
                        && std::os::unix::fs::PermissionsExt::mode(&metadata.permissions()) & 0o111
                            != 0
                })
            })
            .expect("fixture162: Git exécutable absent du PATH du runner")
            .canonicalize()
            .expect("fixture162: chemin Git non résolu");
        let version = Command::new(&git)
            .arg("--version")
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .output()
            .expect("fixture162: Git inaccessible");
        let output = String::from_utf8_lossy(&version.stdout);
        let numbers = output
            .strip_prefix("git version ")
            .unwrap_or("")
            .split('.')
            .take(2)
            .map(str::parse::<u32>)
            .collect::<Result<Vec<_>, _>>();
        assert!(
            version.status.success()
                && numbers.is_ok_and(|v| v.len() == 2 && (v[0], v[1]) >= (2, 44)),
            "fixture162: Git >= 2.44 requis dans PATH du runner, chemin={}, version={output:?}",
            git.display()
        );
        std::os::unix::fs::symlink(&git, root.join("bin/git")).unwrap();
        let project = root.join("project");
        let worker_state = root.join("worker-private");
        let secrets = root.join("secrets");
        for path in [&project, &worker_state, &secrets] {
            support::private_dir(path).unwrap();
        }
        for args in [
            vec!["init", "--quiet"],
            vec!["config", "--local", "remote.origin.url", REPOSITORY],
        ] {
            let output = Command::new("/usr/bin/git")
                .args(args)
                .current_dir(&project)
                .env_clear()
                .env("HOME", root.join("provider"))
                .env("PATH", "/usr/bin:/bin")
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "Git privé162: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let credential = secrets.join("worker.key");
        support::private_write(&credential, SECRET).unwrap();
        let path = root.join("state/agents.json");
        let mut registry: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        registry["machines"] = json!({"cartae162":{
            "enabled":true,"worker_id":"worker162","provisioning_revision":1,
            "credential_fingerprint":format!("{:x}", Sha256::digest(SECRET.as_bytes())),
            "allowed_agent_types":["fake-codex"],"mappings":{"project162":{
                "source_host":"origin162","source_root":ORIGIN,"remote_root":project,
                "expected_git_identity":REPOSITORY,"mapping_revision":1
            }}
        }});
        support::private_write(&path, serde_json::to_vec(&registry).unwrap()).unwrap();
        let front = root.join("state/worker.sock");
        let relay = if candidate_probe {
            Relay::with_candidate_probe(&front, support::socket(root))
        } else {
            Relay::new(&front, support::socket(root))
        };
        Self {
            worker: None,
            relay: Some(relay),
            fixture,
            front,
            worker_state,
            credential,
            launches: 0,
        }
    }
    fn start_master_with_one_slot(&mut self) {
        assert!(self.fixture.daemon.is_none());
        self.fixture.starts += 1;
        let log = self
            .fixture
            .root
            .join(format!("daemon-{}.log", self.fixture.starts));
        support::private_write(&log, []).unwrap();
        self.fixture.daemon = Some(OwnedChild(
            support::isolated_command(&self.fixture.root)
                .arg("daemon")
                .env("BRIDGET_FLEET_QUOTA", "1")
                .env("TMPDIR", self.fixture.root.join("tmp"))
                .env("RUST_LOG", "info")
                .stdout(Stdio::null())
                .stderr(fs::OpenOptions::new().append(true).open(log).unwrap())
                .spawn()
                .unwrap(),
        ));
        let deadline = Instant::now() + Duration::from_secs(10);
        while UnixStream::connect(support::socket(&self.fixture.root)).is_err() {
            assert!(Instant::now() < deadline, "maître162 quota1 absent");
            assert!(
                self.fixture
                    .daemon
                    .as_mut()
                    .unwrap()
                    .0
                    .try_wait()
                    .unwrap()
                    .is_none()
            );
            thread::sleep(Duration::from_millis(10));
        }
    }
    fn start_worker(&mut self) -> Control {
        self.spawn_worker();
        self.relay.as_ref().unwrap().connect()
    }
    fn spawn_worker(&mut self) {
        assert!(self.worker.is_none());
        self.launches += 1;
        let log = self
            .fixture
            .root
            .join(format!("worker162-{}.log", self.launches));
        support::private_write(&log, []).unwrap();
        let mut command = support::isolated_command(&self.fixture.root);
        command
            .arg("worker")
            .args([
                "--machine-id",
                "cartae162",
                "--worker-id",
                "worker162",
                "--provisioning-revision",
                "1",
            ])
            .arg("--credential-file")
            .arg(&self.credential)
            .arg("--state-root")
            .arg(&self.worker_state)
            .arg("--registry-file")
            .arg(self.fixture.root.join("state/agents.json"))
            .arg("--master-socket")
            .arg(&self.front)
            .env(
                "PATH",
                format!(
                    "{}:/usr/bin:/bin:/usr/sbin:/sbin",
                    self.fixture.root.join("bin").display()
                ),
            )
            .stdout(Stdio::null())
            .stderr(fs::OpenOptions::new().append(true).open(log).unwrap());
        self.worker = Some(OwnedChild(command.spawn().unwrap()));
    }
    fn stop_worker(&mut self) {
        if let Some(mut child) = self.worker.take() {
            child.stop();
        }
    }
    fn receipt_path(&self, command: &WorkerCommandIdentity) -> PathBuf {
        self.worker_state.join("store/commands").join(format!(
            "{:x}.json",
            Sha256::digest(command.command_id.as_bytes())
        ))
    }
    fn receipt(&self, command: &WorkerCommandIdentity) -> WorkerCommandReceipt {
        serde_json::from_slice(&fs::read(self.receipt_path(command)).unwrap()).unwrap()
    }
    fn starts(&self, instance: &str) -> usize {
        self.fixture
            .evidence()
            .iter()
            .filter(|item| item["event"] == "started162" && item["instance_id"] == instance)
            .count()
    }
    fn await_started(&self, instance: &str) {
        let deadline = Instant::now() + Duration::from_secs(12);
        while self.starts(instance) == 0 {
            assert!(
                Instant::now() < deadline,
                "fournisseur162 absent : {}",
                self.fixture.root.display()
            );
            thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(self.starts(instance), 1);
    }
    fn prepare_start(
        &mut self,
        control: &mut Control,
        label: &str,
    ) -> (WorkerCommandIdentity, WorkerStart, i64, ManagedMarker) {
        let mut command = WorkerCommandIdentity {
            command_id: format!("command162-{label}"),
            task_id: format!("task162-{label}"),
            instance_id: String::new(),
            generation: 0,
        };
        let prepare = WorkerPrepare {
            machine_id: identity().machine_id,
            project_id: "project162".into(),
            provisioning_revision: 1,
            mapping_revision: 1,
            origin_host: "origin162".into(),
            origin_cwd: ORIGIN.into(),
            source_root: ORIGIN.into(),
            remote_root: self
                .fixture
                .root
                .join("project")
                .to_string_lossy()
                .into_owned(),
            source_root_identity: REPOSITORY.into(),
            relative_cwd: ".".into(),
            base_commit: None,
            agent_type: "fake-codex".into(),
            model: "gpt-5.6-terra".into(),
            effort: Some("high".into()),
            posture: Some(SpawnPosture::Discovery),
            workspace: None,
            parent: ProviderPermissions {
                version: 1,
                source: "native_wrapper".into(),
                run_id: "run162".into(),
                provider_session_id: "session162".into(),
                provider_instance_id: "parent162".into(),
                revision: 1,
                driver: "codex_app_server".into(),
                cwd: ORIGIN.into(),
                runtime_mode: "full-access".into(),
                interaction_mode: "default".into(),
                provider_policy: json!({"kind":"codex","approval_policy":"never","approvals_reviewer":"user","sandbox_policy":{"type":"readOnly","networkAccess":false}}),
            },
        };
        let prepared = match control.request(&command, WorkerRequestOperation::Prepare { prepare })
        {
            WorkerResponseOperation::Prepared { prepared } => prepared,
            other => panic!("Prepare réel162 refusé: {other:?}"),
        };
        command.instance_id = format!("instance162-{label}");
        command.generation = if label == "restart" { 2 } else { 1 };
        let start = WorkerStart {
            prepared_token: prepared.token,
            child_id: uuid::Uuid::new_v4().to_string(),
            mission_id: uuid::Uuid::new_v4().to_string(),
            forward_child_requests: false,
            placement: prepared.placement,
            definition: prepared.definition,
            child_policy: prepared.child_policy,
            effective_posture: prepared.effective_posture,
            deadline_at: now() + 120,
        };
        if label == "restart" {
            let mut stale = start.clone();
            stale.prepared_token = format!("previous-{}", stale.prepared_token);
            assert_eq!(
                control.request(&command, WorkerRequestOperation::Start { start: stale }),
                WorkerResponseOperation::Rejected {
                    reason: WorkerRefusal::PrepareExpired
                }
            );
            assert!(!self.receipt_path(&command).exists());
            assert_eq!(
                fs::read_dir(self.worker_state.join("store/markers"))
                    .unwrap()
                    .count(),
                0
            );
            assert_eq!(self.starts(&command.instance_id), 0);
        }
        let ready_at = match control.request(
            &command,
            WorkerRequestOperation::Start {
                start: start.clone(),
            },
        ) {
            WorkerResponseOperation::ReadyToRelease { ready_at } => ready_at,
            other => panic!("bootstrap réel162 refusé: {other:?}"),
        };
        let receipt = self.receipt(&command);
        assert!(receipt.permit.is_none());
        let marker = receipt.marker.expect("marqueur durable avant Ready");
        assert_eq!(marker.command_id, command.command_id);
        assert_eq!(marker.instance_id, command.instance_id);
        assert_eq!(marker.generation, command.generation);
        assert_eq!(process_birth(marker.pgid).unwrap(), marker.birth);
        assert!(group_exists(marker.pgid).unwrap());
        assert_eq!(
            self.starts(&command.instance_id),
            0,
            "aucun fournisseur avant RELEASE"
        );
        self.fixture.native_groups.push(marker.clone());
        (command, start, ready_at, marker)
    }
    fn reserve_permit(
        &self,
        control: &mut Control,
        command: &WorkerCommandIdentity,
        start: &WorkerStart,
        ready_at: i64,
    ) -> WorkerReleasePermit {
        let slot = WorkerResultSlot {
            identity: WorkerResultIdentity {
                command: command.clone(),
                mission_id: start.mission_id.clone(),
                round: 1,
                execution_id: format!("execution-{}", start.mission_id),
                execution_generation: 1,
            },
            admission_digest: "a".repeat(64),
        };
        assert!(
            matches!(control.request(command, WorkerRequestOperation::ReserveResultSlot { slot: slot.clone() }),
            WorkerResponseOperation::ResultSlotReserved { slot: confirmed, .. } if confirmed == slot)
        );
        WorkerReleasePermit {
            permit_id: format!("permit-{}", command.command_id),
            command_digest: worker_command_digest(&identity(), command, start).unwrap(),
            permission_revision: 1,
            mapping_revision: 1,
            provisioning_revision: 1,
            issued_at: ready_at,
            expires_at: ready_at + 5,
        }
    }
    fn reserve_and_release(
        &self,
        control: &mut Control,
        command: &WorkerCommandIdentity,
        start: &WorkerStart,
        ready_at: i64,
    ) -> WorkerReleasePermit {
        let permit = self.reserve_permit(control, command, start, ready_at);
        assert_eq!(
            control.request(
                command,
                WorkerRequestOperation::Release {
                    permit: permit.clone()
                }
            ),
            WorkerResponseOperation::Released {}
        );
        assert_eq!(self.receipt(command).permit, Some(permit.clone()));
        permit
    }
    fn cancel(
        &self,
        control: &mut Control,
        command: &WorkerCommandIdentity,
        marker: &ManagedMarker,
    ) {
        assert_eq!(process_birth(marker.pgid).unwrap(), marker.birth);
        // La commande du groupe a été identifiée avant le signal émis par le worker.
        let observed = Command::new("/bin/ps")
            .args(["-ww", "-p", &marker.pgid.to_string(), "-o", "command="])
            .output()
            .unwrap();
        let process = String::from_utf8_lossy(&observed.stdout);
        assert!(
            process.contains(env!("CARGO_BIN_EXE_bridget"))
                && !process.to_lowercase().contains("firefox")
        );
        assert!(matches!(
            control.request(
                command,
                WorkerRequestOperation::Cancel {
                    target: WorkerCancelScope::Command {}
                }
            ),
            WorkerResponseOperation::Observation { .. }
        ));
        control.await_phase(command, WorkerCommandPhase::Terminal);
        assert!(
            wait_group_gone(
                marker.pgid,
                Duration::from_secs(6),
                Duration::from_millis(20)
            )
            .unwrap()
        );
    }
}

impl Drop for RemoteFixture {
    fn drop(&mut self) {
        self.stop_worker();
        self.fixture.stop_native_groups();
        self.relay.take();
    }
}

#[test]
fn worker163_probe_groupe_huit_profils_sans_prepare_ni_lancement() {
    let mut f = RemoteFixture::new_with_candidate_probe();
    let project = f.fixture.root.join("project");
    support::private_write(&project.join(".gitignore"), b".worktrees/\n").unwrap();
    let add = Command::new("/usr/bin/git")
        .args(["add", ".gitignore"])
        .current_dir(&project)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap();
    assert!(add.status.success(), "Git privé163: {:?}", add.stderr);
    let commit = Command::new("/usr/bin/git")
        .args([
            "-c",
            "user.name=fixture162",
            "-c",
            "user.email=fixture162@invalid",
            "commit",
            "--quiet",
            "-m",
            "probe163",
        ])
        .current_dir(&project)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap();
    assert!(commit.status.success(), "Git privé163: {:?}", commit.stderr);
    let base_commit = Command::new("/usr/bin/git")
        .args(["rev-parse", "--verify", "HEAD^{commit}"])
        .current_dir(&project)
        .env_clear()
        .env("HOME", f.fixture.root.join("provider"))
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap();
    assert!(
        base_commit.status.success(),
        "Git base163: {:?}",
        base_commit.stderr
    );
    let base_commit = String::from_utf8(base_commit.stdout)
        .unwrap()
        .trim()
        .to_owned();
    let branch = Command::new("/usr/bin/git")
        .args(["branch", "probe163-base", &base_commit])
        .current_dir(&project)
        .env_clear()
        .env("HOME", f.fixture.root.join("provider"))
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap();
    assert!(
        branch.status.success(),
        "Git branche163: {:?}",
        branch.stderr
    );
    support::private_write(&project.join("probe163-head"), b"head suivant\n").unwrap();
    let add_head = Command::new("/usr/bin/git")
        .args(["add", "probe163-head"])
        .current_dir(&project)
        .env_clear()
        .env("HOME", f.fixture.root.join("provider"))
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap();
    assert!(
        add_head.status.success(),
        "Git ajout tête163: {:?}",
        add_head.stderr
    );
    let commit_head = Command::new("/usr/bin/git")
        .args([
            "-c",
            "user.name=fixture162",
            "-c",
            "user.email=fixture162@invalid",
            "commit",
            "--quiet",
            "-m",
            "probe163-head",
        ])
        .current_dir(&project)
        .env_clear()
        .env("HOME", f.fixture.root.join("provider"))
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap();
    assert!(
        commit_head.status.success(),
        "Git tête163: {:?}",
        commit_head.stderr
    );
    let mut control = f.start_worker();
    assert!(control.candidate_probe);
    let candidate = CandidateProbeCandidate {
        candidate_id: "c01".into(),
        project_id: "project162".into(),
        origin_host: "origin162".into(),
        origin_cwd: ORIGIN.into(),
        source_root: ORIGIN.into(),
        remote_root: f
            .fixture
            .root
            .join("project")
            .to_string_lossy()
            .into_owned(),
        source_root_identity: REPOSITORY.into(),
        relative_cwd: ".".into(),
        base_commit: Some(base_commit.clone()),
        agent_type: "fake-codex".into(),
        model: "gpt-5.6-terra".into(),
        effort: Some("high".into()),
        posture: Some(SpawnPosture::Discovery),
        workspace: Some(WorkspaceRequest::Worktree {
            base_ref: Some("probe163-base".into()),
            branch: None,
        }),
    };
    let pinned_candidate = candidate.clone();
    let candidates = (1..=8)
        .map(|index| {
            let mut candidate = candidate.clone();
            candidate.candidate_id = format!("c0{index}");
            candidate
        })
        .collect();
    let parent = ProviderPermissions {
        version: 1,
        source: "native_wrapper".into(),
        run_id: "probe163".into(),
        provider_session_id: "session163".into(),
        provider_instance_id: "parent163".into(),
        revision: 1,
        driver: "codex_app_server".into(),
        cwd: ORIGIN.into(),
        runtime_mode: "full-access".into(),
        interaction_mode: "default".into(),
        provider_policy: json!({"kind":"codex","approval_policy":"never","approvals_reviewer":"user","sandbox_policy":{"type":"readOnly","networkAccess":false}}),
    };
    let result = control.candidate_probe(CandidateProbe {
        version: WORKER_CANDIDATE_PROBE_VERSION,
        correlation_id: "probe163-eight".into(),
        connection_epoch: control.epoch.clone(),
        worker: identity(),
        admission_id: "admission163".into(),
        policy_revision: 1,
        parent: parent.clone(),
        candidates,
    });
    assert_eq!(result.verdicts.len(), 8);
    assert!(
        result
            .verdicts
            .iter()
            .all(|verdict| matches!(verdict, CandidateProbeVerdict::Eligible { .. }))
    );
    for directory in ["commands", "slots", "workspaces", "markers"] {
        assert_eq!(
            fs::read_dir(f.worker_state.join("store").join(directory))
                .unwrap()
                .count(),
            0,
            "le probe ne doit rien écrire dans {directory}"
        );
    }
    assert!(
        f.fixture
            .evidence()
            .iter()
            .all(|event| event["event"] != "started162")
    );

    // Une reconnexion change l'époque : le verdict précédent n'est pas un
    // permis réutilisable, le nouveau probe est relu intégralement.
    let old_epoch = control.epoch.clone();
    control.disconnect();
    f.stop_worker();
    let mut control = f.start_worker();
    assert_ne!(control.epoch, old_epoch);
    let reprobed = control.candidate_probe(CandidateProbe {
        version: WORKER_CANDIDATE_PROBE_VERSION,
        correlation_id: "probe163-reconnect".into(),
        connection_epoch: control.epoch.clone(),
        worker: identity(),
        admission_id: "admission163".into(),
        policy_revision: 1,
        parent: parent.clone(),
        candidates: vec![candidate.clone()],
    });
    assert!(matches!(
        reprobed.verdicts.as_slice(),
        [CandidateProbeVerdict::Eligible { candidate_id, .. }] if candidate_id == "c01"
    ));

    // Une révocation est lue par la prochaine sonde ; elle ne réemploie aucun
    // fait positif précédent et ne transforme pas le refus en lancement.
    let registry_path = f.fixture.root.join("state/agents.json");
    let mut registry: Value = serde_json::from_slice(&fs::read(&registry_path).unwrap()).unwrap();
    registry["machines"]["cartae162"]["allowed_agent_types"] = json!([]);
    support::private_write(&registry_path, serde_json::to_vec(&registry).unwrap()).unwrap();
    let revoked = control.candidate_probe(CandidateProbe {
        version: WORKER_CANDIDATE_PROBE_VERSION,
        correlation_id: "probe163-revoked".into(),
        connection_epoch: control.epoch.clone(),
        worker: identity(),
        admission_id: "admission163".into(),
        policy_revision: 2,
        parent: parent.clone(),
        candidates: vec![candidate.clone()],
    });
    assert!(
        revoked
            .verdicts
            .iter()
            .all(|verdict| matches!(verdict, CandidateProbeVerdict::Ineligible { .. }))
    );
    // Le même candidat qui était Eligible avant la révocation ne peut pas
    // traverser Prepare : la vérification de droits précède désormais toute
    // création de worktree, slot ou fournisseur.
    let command = WorkerCommandIdentity {
        command_id: "command163-revoked".into(),
        task_id: "00000000-0000-4000-8000-000000000163".into(),
        instance_id: String::new(),
        generation: 0,
    };
    let prepare = WorkerPrepare {
        machine_id: identity().machine_id,
        project_id: "project162".into(),
        provisioning_revision: 1,
        mapping_revision: 1,
        origin_host: "origin162".into(),
        origin_cwd: ORIGIN.into(),
        source_root: ORIGIN.into(),
        remote_root: project.to_string_lossy().into_owned(),
        source_root_identity: REPOSITORY.into(),
        relative_cwd: ".".into(),
        base_commit: pinned_candidate.base_commit.clone(),
        agent_type: "fake-codex".into(),
        model: "gpt-5.6-terra".into(),
        effort: Some("high".into()),
        posture: Some(SpawnPosture::Discovery),
        workspace: Some(WorkspaceRequest::Worktree {
            base_ref: Some("probe163-base".into()),
            branch: None,
        }),
        parent: parent.clone(),
    };
    assert_eq!(
        control.request(&command, WorkerRequestOperation::Prepare { prepare }),
        WorkerResponseOperation::Rejected {
            reason: WorkerRefusal::Unauthorized
        }
    );
    for directory in ["commands", "slots", "workspaces", "markers"] {
        assert_eq!(
            fs::read_dir(f.worker_state.join("store").join(directory))
                .unwrap()
                .count(),
            0,
            "la révocation ne doit rien écrire dans {directory}"
        );
    }

    // Le HEAD du dépôt est plus récent que `probe163-base`. Après restauration
    // des droits, Prepare doit donc accepter la base future explicitement
    // épinglée, créer son worktree, mais ne jamais lancer le fournisseur.
    registry["machines"]["cartae162"]["allowed_agent_types"] = json!(["fake-codex"]);
    support::private_write(&registry_path, serde_json::to_vec(&registry).unwrap()).unwrap();
    let pinned_command = WorkerCommandIdentity {
        command_id: "command163-pinned-base".into(),
        task_id: "00000000-0000-4000-8000-000000000164".into(),
        instance_id: String::new(),
        generation: 0,
    };
    let pinned_prepare = WorkerPrepare {
        machine_id: identity().machine_id,
        project_id: pinned_candidate.project_id.clone(),
        provisioning_revision: 1,
        mapping_revision: 1,
        origin_host: pinned_candidate.origin_host.clone(),
        origin_cwd: pinned_candidate.origin_cwd.clone(),
        source_root: pinned_candidate.source_root.clone(),
        remote_root: pinned_candidate.remote_root.clone(),
        source_root_identity: pinned_candidate.source_root_identity.clone(),
        relative_cwd: pinned_candidate.relative_cwd.clone(),
        base_commit: pinned_candidate.base_commit.clone(),
        agent_type: pinned_candidate.agent_type.clone(),
        model: pinned_candidate.model.clone(),
        effort: pinned_candidate.effort.clone(),
        posture: pinned_candidate.posture,
        workspace: pinned_candidate.workspace.clone(),
        parent,
    };
    let prepared = match control.request(
        &pinned_command,
        WorkerRequestOperation::Prepare {
            prepare: pinned_prepare,
        },
    ) {
        WorkerResponseOperation::Prepared { prepared } => prepared,
        other => panic!("Prepare base épinglée163 refusé: {other:?}"),
    };
    assert_eq!(
        prepared.placement.base_commit.as_deref(),
        Some(base_commit.as_str())
    );
    for directory in ["commands", "slots", "markers"] {
        assert_eq!(
            fs::read_dir(f.worker_state.join("store").join(directory))
                .unwrap()
                .count(),
            0,
            "Prepare épinglé ne doit rien lancer dans {directory}"
        );
    }
    assert_eq!(
        fs::read_dir(f.worker_state.join("store/workspaces"))
            .unwrap()
            .count(),
        1,
        "le seul effet autorisé est le worktree préparé"
    );
    assert!(
        f.fixture
            .evidence()
            .iter()
            .all(|event| event["event"] != "started162")
    );
}

#[test]
fn remote162_e2e_release_rejeu_restart_worker_sans_second_fournisseur() {
    let mut f = RemoteFixture::new();
    let mut control = f.start_worker();
    let (command, start, ready_at, marker) = f.prepare_start(&mut control, "restart");
    let permit = f.reserve_permit(&mut control, &command, &start, ready_at);
    let held_receipt = f.receipt(&command);
    let mut old_generation = command.clone();
    old_generation.generation = 1;
    let mut old_instance = command.clone();
    old_instance.instance_id = "instance162-previous".into();
    for stale in [old_generation, old_instance] {
        let mut stale_permit = permit.clone();
        stale_permit.command_digest = worker_command_digest(&identity(), &stale, &start).unwrap();
        for operation in [
            WorkerRequestOperation::Cancel {
                target: WorkerCancelScope::Command {},
            },
            WorkerRequestOperation::Release {
                permit: stale_permit,
            },
        ] {
            assert_eq!(
                control.request(&stale, operation),
                WorkerResponseOperation::Rejected {
                    reason: WorkerRefusal::CommandMismatch
                }
            );
            assert_eq!(
                f.receipt(&command),
                held_receipt,
                "un ancien tuple ne consomme pas le reçu courant"
            );
            assert_eq!(process_birth(marker.pgid).unwrap(), marker.birth);
            assert!(group_exists(marker.pgid).unwrap());
            assert_eq!(f.starts(&command.instance_id), 0);
        }
    }
    assert_eq!(
        control.request(
            &command,
            WorkerRequestOperation::Release {
                permit: permit.clone()
            }
        ),
        WorkerResponseOperation::Released {}
    );
    assert_eq!(f.receipt(&command).permit, Some(permit.clone()));
    f.await_started(&command.instance_id);
    control.await_native_ready(&command);
    assert!(
        matches!(control.request(&command, WorkerRequestOperation::Start { start: start.clone() }),
        WorkerResponseOperation::Observation { observation } if observation.phase == WorkerCommandPhase::Released)
    );
    assert_eq!(
        control.request(&command, WorkerRequestOperation::Release { permit }),
        WorkerResponseOperation::Rejected {
            reason: WorkerRefusal::UnknownExecution
        }
    );
    assert_eq!(f.receipt(&command).marker, Some(marker.clone()));
    f.stop_worker();
    control.disconnect();
    assert_eq!(process_birth(marker.pgid).unwrap(), marker.birth);
    let mut control = f.start_worker();
    assert!(
        matches!(control.request(&command, WorkerRequestOperation::Start { start: start.clone() }),
        WorkerResponseOperation::Observation { observation } if observation.phase == WorkerCommandPhase::Released)
    );
    assert_eq!(f.receipt(&command).marker, Some(marker.clone()));
    assert_eq!(process_birth(marker.pgid).unwrap(), marker.birth);
    assert_eq!(f.starts(&command.instance_id), 1);
    // T022/T025 : vrai wrapper producteur, worker absent avant publication.
    // La coupure précède l'écriture ; elle ne prétend pas interrompre un fsync.
    let mut sender =
        support::register_agent_as(&support::socket(&f.fixture.root), PARENT, INSTANCE);
    let mut message = BridgetMessage::new(PARENT, &start.child_id, "NATIVE162_NOMINAL");
    message.id = start.mission_id.clone();
    message.reply = true;
    message.reply_timeout = Some(60);
    message.intent = Some(bridget_core::MessageIntent::TriggerTurn);
    sender.send(WrapperToDaemon::Send(message));
    assert!(
        matches!(f.fixture.receive(&mut sender), DaemonToWrapper::Ack { id, .. } if id == start.mission_id)
    );
    wait_evidence152(&f.fixture, "turn162", &command.instance_id);
    let reservation = bridget_daemon::remote_worker::find_result_reservation(
        &f.worker_state.join("store"),
        &identity(),
        &command,
        &start.mission_id,
        &format!("execution-{}", start.mission_id),
        1,
    )
    .unwrap();
    assert!(!reservation.blob_path.exists());
    f.stop_worker();
    control.disconnect();
    support::private_write(&f.fixture.root.join("project/release162"), b"go").unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !reservation.blob_path.exists() {
        assert!(
            Instant::now() < deadline,
            "wrapper162 n'a pas publié sans worker"
        );
        thread::sleep(Duration::from_millis(20));
    }
    let original_blob = fs::read(&reservation.blob_path).unwrap();
    let result: RemoteResultEnvelope = serde_json::from_slice(&original_blob).unwrap();
    result.validate().unwrap();
    assert_eq!(result.identity, reservation.slot.identity);
    assert_eq!(result.body, "fixture162-remote-answer");
    f.fixture.stop_native_groups();
    assert!(!group_exists(marker.pgid).unwrap());
    let mut control = f.start_worker();
    assert_eq!(control.final_result(&command), result);
    assert_eq!(fs::read(&reservation.blob_path).unwrap(), original_blob);
    // La première confirmation est perdue : reconnexion, même blob et même UUID.
    control.disconnect();
    let mut control = f.relay.as_ref().unwrap().connect();
    assert_eq!(control.final_result(&command), result);
    assert_eq!(fs::read(&reservation.blob_path).unwrap(), original_blob);
    let disposition = WorkerResultDisposition {
        identity: result.identity.clone(),
        result_digest: result.digest.clone(),
        disposition: RemoteResultDisposition::Accepted {},
    };
    assert_eq!(
        control.request(
            &command,
            WorkerRequestOperation::ResultDisposition {
                result: disposition.clone()
            }
        ),
        WorkerResponseOperation::DispositionRecorded {
            result: disposition.clone()
        }
    );
    let slot_receipt = reservation
        .blob_path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("receipt.json");
    let receipt: Value = serde_json::from_slice(&fs::read(&slot_receipt).unwrap()).unwrap();
    assert_eq!(
        receipt["disposition"],
        serde_json::to_value(&disposition).unwrap()
    );
    assert_eq!(receipt["producer_terminal"], true);
    f.stop_worker();
    control.disconnect();
    let mut control = f.start_worker();
    assert_eq!(control.phase(&command), WorkerCommandPhase::Terminal);
    control.no_final_result();
    let receipt: Value = serde_json::from_slice(&fs::read(slot_receipt).unwrap()).unwrap();
    assert_eq!(
        receipt["disposition"],
        serde_json::to_value(disposition).unwrap()
    );
    assert_eq!(f.starts(&command.instance_id), 1);
    assert_eq!(
        f.fixture
            .evidence()
            .iter()
            .filter(
                |event| event["event"] == "turn162" && event["instance_id"] == command.instance_id
            )
            .count(),
        1
    );
    sender.send(WrapperToDaemon::Unregister);
}

#[test]
fn remote162_e2e_permis_expire_et_coupure_avant_release_sans_fournisseur() {
    for case in ["expiry", "disconnect", "mapping-revoked"] {
        let mut f = RemoteFixture::new();
        let mut control = f.start_worker();
        let (command, start, ready_at, marker) = f.prepare_start(&mut control, case);
        if case == "disconnect" {
            control.disconnect();
            control = f.relay.as_ref().unwrap().connect();
        } else if case == "mapping-revoked" {
            let permit = f.reserve_permit(&mut control, &command, &start, ready_at);
            let registry_path = f.fixture.root.join("state/agents.json");
            let mut registry: Value =
                serde_json::from_slice(&fs::read(&registry_path).unwrap()).unwrap();
            assert!(
                registry["machines"]["cartae162"]["mappings"]
                    .as_object_mut()
                    .unwrap()
                    .remove("project162")
                    .is_some()
            );
            support::private_write(&registry_path, serde_json::to_vec(&registry).unwrap()).unwrap();
            assert_eq!(process_birth(marker.pgid).unwrap(), marker.birth);
            assert_eq!(f.starts(&command.instance_id), 0);
            assert_eq!(
                control.request(&command, WorkerRequestOperation::Release { permit }),
                WorkerResponseOperation::Rejected {
                    reason: WorkerRefusal::PlacementMismatch
                }
            );
        } else {
            // Vrai délai depuis Ready ; aucun passage forcé à l'état terminal.
            thread::sleep(Duration::from_millis(5200));
        }
        control.await_phase(&command, WorkerCommandPhase::Terminal);
        assert!(
            wait_group_gone(
                marker.pgid,
                Duration::from_secs(6),
                Duration::from_millis(20)
            )
            .unwrap()
        );
        assert!(f.receipt(&command).permit.is_none());
        let permit = WorkerReleasePermit {
            permit_id: "too-late162".into(),
            command_digest: worker_command_digest(&identity(), &command, &start).unwrap(),
            permission_revision: 1,
            mapping_revision: 1,
            provisioning_revision: 1,
            issued_at: ready_at,
            expires_at: ready_at + 5,
        };
        assert!(matches!(
            control.request(&command, WorkerRequestOperation::Release { permit }),
            WorkerResponseOperation::Rejected { .. }
        ));
        assert!(
            matches!(control.request(&command, WorkerRequestOperation::Start { start }),
            WorkerResponseOperation::Observation { observation } if observation.phase == WorkerCommandPhase::Terminal)
        );
        assert_eq!(f.starts(&command.instance_id), 0);
    }
}

#[cfg(target_os = "linux")]
#[test]
fn remote162_e2e_cancel_descendant_setsid_pipe_herite_ignore_term_restart() {
    for mode in ["normal", "ignore"] {
        let mut f = RemoteFixture::new();
        support::private_write(&f.fixture.root.join("detached162"), mode.as_bytes()).unwrap();
        let mut control = f.start_worker();
        let (command, start, ready_at, marker) = f.prepare_start(&mut control, "drain");
        f.reserve_and_release(&mut control, &command, &start, ready_at);
        f.await_started(&command.instance_id);
        control.await_native_ready(&command);
        let deadline = Instant::now() + Duration::from_secs(12);
        let pid = loop {
            if let Some(event) = f
                .fixture
                .evidence()
                .iter()
                .find(|event| event["event"] == "detached162")
            {
                break event["pid"].as_u64().unwrap() as u32;
            }
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(20));
        };
        let birth = process_birth(pid).unwrap();
        assert_ne!(pid, marker.pgid);
        let proof = f.worker_state.join("store/markers").join(format!(
            "{:x}.drained",
            Sha256::digest(command.command_id.as_bytes())
        ));
        assert!(matches!(
            control.request(
                &command,
                WorkerRequestOperation::Cancel {
                    target: WorkerCancelScope::Command {}
                }
            ),
            WorkerResponseOperation::Observation { .. }
        ));
        if mode == "ignore" {
            thread::sleep(Duration::from_millis(500));
            assert_ne!(control.phase(&command), WorkerCommandPhase::Terminal);
            assert_eq!(process_birth(pid).unwrap(), birth);
            assert_eq!(process_birth(marker.pgid).unwrap(), marker.birth);
            assert!(!proof.exists());
            f.stop_worker();
            control.disconnect();
            control = f.start_worker();
            assert_ne!(control.phase(&command), WorkerCommandPhase::Terminal);
            // Une sortie coopérative, jamais SIGKILL, débloque le pipe puis le drainage.
            support::private_write(&f.fixture.root.join("release-detached162"), b"go").unwrap();
        }
        control.await_phase(&command, WorkerCommandPhase::Terminal);
        assert!(process_birth(pid).is_err());
        assert!(process_birth(marker.pgid).is_err());
        let evidence: Value = serde_json::from_slice(&fs::read(proof).unwrap()).unwrap();
        assert_eq!(evidence["pgid"], marker.pgid);
        assert_eq!(evidence["birth"], marker.birth);
        assert_eq!(f.starts(&command.instance_id), 1);
    }
}

#[test]
fn remote162_e2e_cancel_naissance_divergente_preserve_processus_puis_arret_atteste() {
    let mut f = RemoteFixture::new();
    let mut control = f.start_worker();
    let (command, start, ready_at, marker) = f.prepare_start(&mut control, "birth");
    f.reserve_and_release(&mut control, &command, &start, ready_at);
    f.await_started(&command.instance_id);
    control.await_native_ready(&command);
    f.stop_worker();
    control.disconnect();
    // Injection disque privée entre deux workers : simule le PID recyclé sans
    // tenter de forcer le noyau à réattribuer un PID, ni cibler un tiers.
    let receipt_path = f.receipt_path(&command);
    let marker_path = f.worker_state.join("store/markers").join(format!(
        "{:x}.json",
        Sha256::digest(command.command_id.as_bytes())
    ));
    let original_receipt = fs::read(&receipt_path).unwrap();
    let original_marker = fs::read(&marker_path).unwrap();
    let mut receipt: Value = serde_json::from_slice(&original_receipt).unwrap();
    let mut wrong_marker = marker.clone();
    wrong_marker.birth = marker.birth.checked_add(1).unwrap();
    receipt["marker"] = serde_json::to_value(&wrong_marker).unwrap();
    support::private_write(&receipt_path, serde_json::to_vec(&receipt).unwrap()).unwrap();
    support::private_write(&marker_path, serde_json::to_vec(&wrong_marker).unwrap()).unwrap();
    let mut control = f.start_worker();
    assert_eq!(control.phase(&command), WorkerCommandPhase::Unknown);
    assert_eq!(
        control.request(
            &command,
            WorkerRequestOperation::Cancel {
                target: WorkerCancelScope::Command {}
            }
        ),
        WorkerResponseOperation::Rejected {
            reason: WorkerRefusal::UnknownExecution
        }
    );
    assert_eq!(process_birth(marker.pgid).unwrap(), marker.birth);
    assert!(group_exists(marker.pgid).unwrap());
    assert_eq!(f.starts(&command.instance_id), 1);
    f.stop_worker();
    control.disconnect();
    support::private_write(&receipt_path, original_receipt).unwrap();
    support::private_write(&marker_path, original_marker).unwrap();
    let mut control = f.start_worker();
    f.cancel(&mut control, &command, &marker);
    assert_eq!(f.starts(&command.instance_id), 1);
}

#[test]
fn remote162_e2e_delegate_maitre_reel_resultat_unique_cwd_et_modele() {
    for case in ["nominal", "startup-failed", "late-ready"] {
        real_master_case(case);
    }
}

fn real_master_case(case: &str) {
    let mut f = RemoteFixture::new();
    let socket = support::socket(&f.fixture.root);
    let mut probe = support::Client::connect(&socket);
    probe.send(WrapperToDaemon::DaemonIdentityRequest);
    let host = match probe.receive() {
        DaemonToWrapper::DaemonIdentityReport { host, .. } => host,
        other => panic!("identité maître162 absente: {other:?}"),
    };
    drop(probe);
    // Cette variante n'utilise aucun faux maître. Le registre est installé
    // avant le lancement du daemon qui admet réellement la délégation.
    f.fixture.stop();
    f.relay.take();
    fs::remove_file(&f.front).unwrap();
    f.front = socket.clone();
    let origin = f.fixture.root.join("origin-project");
    support::private_dir(&origin).unwrap();
    for args in [
        vec!["init", "--quiet"],
        vec!["config", "--local", "remote.origin.url", REPOSITORY],
    ] {
        let output = Command::new("/usr/bin/git")
            .args(args)
            .current_dir(&origin)
            .env_clear()
            .env("HOME", f.fixture.root.join("provider"))
            .env("PATH", "/usr/bin:/bin")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "Git origine162: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let path = f.fixture.root.join("state/agents.json");
    let mut registry: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    registry["machines"]["cartae162"]["mappings"]["project162"]["source_host"] = json!(host);
    registry["machines"]["cartae162"]["mappings"]["project162"]["source_root"] = json!(origin);
    support::private_write(&path, serde_json::to_vec(&registry).unwrap()).unwrap();
    f.start_master_with_one_slot();
    if case == "late-ready" {
        f.front = f.fixture.root.join("state/worker.sock");
        // Le permis réel expire après 5 s ; 6 s suffisent, sans attendre
        // l'expiration RPC de 30 s ni modifier une horloge de production.
        f.relay = Some(Relay::with_ready_delay(
            &f.front,
            socket.clone(),
            Duration::from_secs(6),
        ));
    } else if case == "startup-failed" {
        support::private_write(&f.fixture.root.join("fail-init162"), b"fail").unwrap();
    }
    f.spawn_worker();

    let mut parent = support::Client::connect(&socket);
    parent.send(WrapperToDaemon::Register {
        agent_type: "fake-codex".into(),
        identity_version: 2,
        agent_id: PARENT.into(),
        host: Some(host),
        transport: Some("codex_app_server".into()),
        channel: None.into(),
        mode: Some(PresenceMode::Acp),
        location: None,
        os: Some("test".into()),
        instance_id: Some(INSTANCE.into()),
        domain: None,
        journal_available: None,
        turn_in_progress: false,
        turn_attest: false,
        queue_fallback: false,
    });
    let DaemonToWrapper::Registered {
        agent_id,
        credential: Some(credential),
    } = parent.receive()
    else {
        panic!("parent réel162 non enregistré");
    };
    assert_eq!(agent_id, PARENT);
    support::save_fixture_credential(&socket, PARENT, INSTANCE, credential);
    let observation = uuid::Uuid::new_v4().to_string();
    parent.send(WrapperToDaemon::NativePermissionFact {
        fact: Some(ProviderPermissions { version: 1, source: "native_wrapper".into(), run_id: "parent-turn162".into(),
            provider_session_id: "parent-session162".into(), provider_instance_id: INSTANCE.into(), revision: 1,
            driver: "codex_app_server".into(), cwd: origin.to_string_lossy().into_owned(),
            runtime_mode: "full-access".into(), interaction_mode: "default".into(),
            provider_policy: json!({"kind":"codex","approval_policy":"never","approvals_reviewer":"user","sandbox_policy":{"type":"readOnly","networkAccess":false}}) }),
        request_id: None, observation_id: Some(observation.clone()), unavailable_code: None,
    });
    assert!(
        matches!(f.fixture.receive(&mut parent), DaemonToWrapper::NativePermissionAcknowledged { observation_id, accepted: true } if observation_id == observation)
    );
    let mut request = delegate(&origin, "nominal162-unique", "NATIVE162_NOMINAL");
    let NativeDelegationRequest::Delegate {
        machine_id,
        project_id,
        agent_type,
        model,
        ..
    } = &mut request
    else {
        unreachable!()
    };
    *machine_id = Some(identity().machine_id);
    *project_id = Some("project162".into());
    *agent_type = "fake-codex".into();
    *model = "gpt-5.6-terra".into();
    let deadline = Instant::now() + Duration::from_secs(12);
    let mut created = loop {
        let result = f.fixture.request(&mut parent, request.clone());
        if result["task_id"].is_string()
            || (case == "late-ready"
                && result["code"]
                    .as_str()
                    .is_some_and(|code| code != "remote_prepare:WorkerUnavailable"))
        {
            break result;
        }
        assert_eq!(
            result["code"],
            "remote_prepare:WorkerUnavailable",
            "admission162: {result}; {}",
            f.fixture.root.display()
        );
        assert!(
            Instant::now() < deadline,
            "worker réel non connecté: {result}"
        );
        assert!(
            f.worker.as_mut().unwrap().0.try_wait().unwrap().is_none(),
            "worker162 terminé"
        );
        thread::sleep(Duration::from_millis(25));
    };
    let mut failed_attempt = None;
    if case != "nominal" {
        let files: Vec<_> = fs::read_dir(f.worker_state.join("store/commands"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        assert_eq!(files.len(), 1, "un seul démarrage tenté : {created}");
        let receipt: WorkerCommandReceipt =
            serde_json::from_slice(&fs::read(&files[0]).unwrap()).unwrap();
        let failed_command = receipt.identity.clone();
        let marker = receipt
            .marker
            .clone()
            .expect("Ready réel a précédé la panne");
        f.fixture.native_groups.push(marker.clone());
        if case == "startup-failed" {
            f.await_started(&failed_command.instance_id);
            assert_eq!(process_birth(marker.pgid).unwrap(), marker.birth);
            assert!(group_exists(marker.pgid).unwrap());
            support::private_write(&f.fixture.root.join("release-failure162"), b"go").unwrap();
        } else {
            assert!(
                f.relay
                    .as_ref()
                    .unwrap()
                    .ready_delayed
                    .load(Ordering::Acquire)
            );
            assert_eq!(f.starts(&failed_command.instance_id), 0);
        }
        f.fixture
            .status_until(&mut parent, &failed_command.task_id, "failed");
        let failed = native_task152(&f.fixture, &failed_command.task_id);
        assert_eq!(failed["error"], "remote_start_failed", "{failed}");
        assert!(failed["mission_issued_at"].is_null());
        let terminal = f.receipt(&failed_command);
        assert_eq!(terminal.phase, WorkerCommandPhase::Terminal);
        assert!(!terminal.native_ready);
        if case == "startup-failed" {
            // Un exit déjà récolté peut précéder la lecture de StartupFailed.
            // L'arrêt prouvé, sans NativeReady, reste l'autorité terminale.
            assert!(terminal.permit.is_some());
            assert!(
                f.fixture
                    .evidence()
                    .iter()
                    .any(|event| event["event"] == "failed-init162"
                        && event["instance_id"] == failed_command.instance_id
                        && event["exit_code"] == 23)
            );
        } else {
            assert!(terminal.permit.is_none());
        }
        assert!(
            wait_group_gone(
                marker.pgid,
                Duration::from_secs(6),
                Duration::from_millis(20)
            )
            .unwrap()
        );
        let desired = desired_store152(&f.fixture).load().unwrap();
        let entry = &desired.equipiers[failed["child"].as_str().unwrap()];
        assert_eq!(
            entry.lifecycle_state,
            bridget_daemon::desired_state::DesiredLifecycleState::Stopped
        );
        assert_eq!(entry.command_id, failed_command.command_id);
        assert_eq!(entry.generation, failed_command.generation);
        assert_eq!(
            f.starts(&failed_command.instance_id),
            usize::from(case == "startup-failed")
        );
        assert!(
            !f.fixture
                .evidence()
                .iter()
                .any(|event| event["event"] == "turn162"
                    && event["instance_id"] == failed_command.instance_id)
        );
        if case == "startup-failed" {
            fs::remove_file(f.fixture.root.join("fail-init162")).unwrap();
        }
        failed_attempt = Some(failed_command.instance_id);
        // Avec un quota réel de 1, la mission suivante ne peut démarrer que
        // si le maître a rendu la réservation. Aucun invalidate manuel.
        let NativeDelegationRequest::Delegate { request_id, .. } = &mut request else {
            unreachable!()
        };
        *request_id = format!("after-{case}-162");
        created = f.fixture.request(&mut parent, request.clone());
        assert!(
            created["task_id"].is_string(),
            "capacité non rendue : {created}"
        );
    }
    let task_id = created["task_id"].as_str().unwrap().to_string();
    let mission = created["message_id"].as_str().unwrap().to_string();
    let deadline = Instant::now() + Duration::from_secs(20);
    let (task, turn) = loop {
        let task = native_task152(&f.fixture, &task_id);
        if let Some(instance) = task["child_instance"].as_str()
            && let Some(turn) = f
                .fixture
                .evidence()
                .into_iter()
                .find(|event| event["event"] == "turn162" && event["instance_id"] == instance)
        {
            break (task, turn);
        }
        let status = f.fixture.request(
            &mut parent,
            NativeDelegationRequest::Status {
                task_id: task_id.clone(),
            },
        );
        assert_ne!(
            status["status"],
            "failed",
            "mission162: {status}; {}",
            f.fixture.root.display()
        );
        assert!(
            Instant::now() < deadline,
            "aucun tour distant162: {status}; {}",
            f.fixture.root.display()
        );
        thread::sleep(Duration::from_millis(25));
    };
    let slot: WorkerResultSlot =
        serde_json::from_value(task["remote_result_slots"][0].clone()).unwrap();
    assert_eq!(slot.identity.round, 1);
    assert_eq!(slot.identity.mission_id, mission);
    let command = slot.identity.command;
    let receipt = f.receipt(&command);
    let marker = receipt.marker.unwrap();
    f.fixture.native_groups.push(marker.clone());
    assert!(receipt.permit.is_some());
    assert_eq!(process_birth(marker.pgid).unwrap(), marker.birth);
    // Le transport Codex crée son propre groupe : le wrapper attesté est
    // le parent du fournisseur, pas le chef de son groupe de processus.
    let provider_pid = u32::try_from(turn["pid"].as_u64().unwrap()).unwrap();
    assert_eq!(turn["ppid"], marker.pgid);
    assert_eq!(turn["pgid"], provider_pid);
    let observed = Command::new("/bin/ps")
        .args(["-p", &provider_pid.to_string(), "-o", "pid=,ppid=,pgid="])
        .output()
        .unwrap();
    assert!(observed.status.success());
    let actual: Vec<u32> = String::from_utf8(observed.stdout)
        .unwrap()
        .split_whitespace()
        .map(|number| number.parse().unwrap())
        .collect();
    assert_eq!(actual, vec![provider_pid, marker.pgid, provider_pid]);
    assert_eq!(turn["cwd"], json!(f.fixture.root.join("project")));
    assert_ne!(turn["cwd"], json!(origin));
    assert_eq!(turn["model"], "gpt-5.6-terra");
    assert_eq!(task["remote_placement"]["machine_id"], "cartae162");
    assert_eq!(task["remote_placement"]["origin_cwd"], json!(origin));
    assert_eq!(task["definition"]["protocol"], "codex_app_server");
    assert_eq!(f.starts(&command.instance_id), 1);
    // Un rejeu de la vraie admission garde la même mission et ne redélivre pas.
    let replay = f.fixture.request(&mut parent, request);
    assert_eq!(replay["task_id"], task_id);
    assert_eq!(replay["message_id"], mission);
    support::private_write(&f.fixture.root.join("project/release162"), b"go").unwrap();
    f.fixture
        .status_until(&mut parent, &task_id, "result_available");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let status = f.fixture.request(
            &mut parent,
            NativeDelegationRequest::Status {
                task_id: task_id.clone(),
            },
        );
        if status["mailbox"]["state"] == "delivered" {
            break;
        }
        assert!(Instant::now() < deadline, "remise parent162: {status}");
        thread::sleep(Duration::from_millis(25));
    }
    let replies: Vec<_> = f
        .fixture
        .replies
        .iter()
        .filter(|message| message.in_reply_to.as_deref() == Some(mission.as_str()))
        .collect();
    assert_eq!(replies.len(), 1);
    assert_eq!(replies[0].body, "fixture162-remote-answer");
    let saved = native_task152(&f.fixture, &task_id);
    assert_eq!(saved["remote_result_receipts"].as_array().unwrap().len(), 1);
    assert_eq!(f.starts(&command.instance_id), 1);
    assert_eq!(
        f.fixture
            .evidence()
            .iter()
            .filter(
                |event| event["event"] == "turn162" && event["instance_id"] == command.instance_id
            )
            .count(),
        1
    );
    if let Some(instance) = failed_attempt {
        assert_eq!(
            f.starts(&instance),
            usize::from(case == "startup-failed"),
            "le démarrage échoué ne doit jamais être rejoué"
        );
    }
    parent.send(WrapperToDaemon::Unregister);
}
