//! T011 : daemon + wrapper + ACP réels, fournisseur déterministe sans compte.
//! Aucun Docker, aucun HOME utilisateur, aucune requête réseau fournisseur.
use bridget_core::BridgetMessage;
use bridget_daemon::managed_process::{ManagedMarkerStore, group_exists};
use bridget_transport::protocol::{AttachWindow, ConnectionRole, ProjectReference, decode, encode};
use bridget_transport::{DaemonToWrapper, SpawnRefusal, StopOutcome, WrapperToDaemon};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::fd::FromRawFd;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const AGENT: &str = "89000000-0000-4000-8000-000000000081";
const CALLER: &str = "89000000-0000-4000-8000-000000000082";
// Même échange initialize/session/new/session/prompt que managed_parity_test.
const ADAPTER: &str = r#"import json, os, sys
with open(sys.argv[1], 'x') as f:
    f.write(str(os.getpid()))
for line in sys.stdin:
    req = json.loads(line)
    method = req.get('method')
    if method == 'initialize':
        result = {'protocolVersion': 1}
    elif method == 'session/new':
        result = {'sessionId': 'core-089-session'}
    elif method == 'session/prompt':
        body = req['params']['prompt'][0]['text']
        if 'core-089-message' in body:
            print(json.dumps({'jsonrpc':'2.0','method':'session/update','params':{
                'sessionId':'core-089-session','update':{'sessionUpdate':'agent_message_chunk',
                'content':{'type':'text','text':'core-089-answer'}}}}), flush=True)
        result = {'stopReason':'end_turn'}
    else:
        continue
    print(json.dumps({'jsonrpc':'2.0','id':req['id'],'result':result}), flush=True)
"#;

fn private_file(path: &Path, bytes: &[u8]) -> fs::File {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .unwrap();
    file.write_all(bytes).unwrap();
    file
}

struct Fixture {
    root: PathBuf,
    daemon: Option<Child>,
    control_cli: Option<Child>,
    deadline: Instant,
}
impl Fixture {
    fn new() -> Self {
        let root = fs::canonicalize("/tmp").unwrap().join(format!(
            "b89h-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..12]
        ));
        for path in [
            &root,
            &root.join("state"),
            &root.join("home"),
            &root.join("tmp"),
        ] {
            fs::DirBuilder::new().mode(0o700).create(path).unwrap();
        }
        private_file(&root.join("adapter.py"), ADAPTER.as_bytes());
        let registry = serde_json::json!({"agents":{"fixture":{
            "command":"/usr/bin/python3", "args":[root.join("adapter.py"), root.join("started")],
            "protocol":"acp", "permissions":"allow", "queue_capacity":4,
            "forbidden_env":["OPENAI_API_KEY","ANTHROPIC_API_KEY"], "pass_env":[]
        }}});
        private_file(
            &root.join("state/agents.json"),
            &serde_json::to_vec(&registry).unwrap(),
        );
        let mut f = Self {
            root,
            daemon: None,
            control_cli: None,
            deadline: Instant::now() + Duration::from_secs(40),
        };
        let log = private_file(&f.root.join("daemon.log"), &[]);
        f.daemon = Some(
            Command::new(env!("CARGO_BIN_EXE_bridget"))
                .arg("daemon")
                .env_clear()
                .env("HOME", f.root.join("home"))
                .env("BRIDGET_HOME", f.root.join("state"))
                .env("BRIDGET_SOCKET", f.socket())
                .env("TMPDIR", f.root.join("tmp"))
                .env("PATH", "/usr/bin:/bin")
                .env("HOSTNAME", "core-089-fixture")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(log)
                .spawn()
                .unwrap(),
        );
        let ready = Instant::now() + Duration::from_secs(10);
        while UnixStream::connect(f.socket()).is_err() {
            assert!(Instant::now() < ready, "readiness: {}", f.log());
            assert!(
                f.daemon.as_mut().unwrap().try_wait().unwrap().is_none(),
                "{}",
                f.log()
            );
            std::thread::sleep(Duration::from_millis(5)); // watchdog, pas oracle métier
        }
        f
    }
    fn socket(&self) -> PathBuf {
        self.root.join("state/s")
    }
    fn log(&self) -> String {
        fs::read_to_string(self.root.join("daemon.log")).unwrap_or_default()
    }
    fn peer(&self) -> Peer {
        Peer::new(&self.socket(), self.deadline)
    }
    fn markers(&self) -> ManagedMarkerStore {
        ManagedMarkerStore::at_directory(self.root.join("state/managed"))
    }
    fn set_complete_via_terminal(&mut self) {
        let mut master_fd = -1;
        let mut slave_fd = -1;
        assert_eq!(
            unsafe {
                libc::openpty(
                    &mut master_fd,
                    &mut slave_fd,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            },
            0
        );
        let _master = unsafe { fs::File::from_raw_fd(master_fd) };
        let slave = unsafe { fs::File::from_raw_fd(slave_fd) };
        let error = private_file(&self.root.join("control.log"), &[]);
        self.control_cli = Some(
            Command::new(env!("CARGO_BIN_EXE_bridget"))
                .args(["control", "posture", "complete"])
                .env_clear()
                .env("HOME", self.root.join("home"))
                .env("BRIDGET_HOME", self.root.join("state"))
                .env("BRIDGET_SOCKET", self.socket())
                .env("TMPDIR", self.root.join("tmp"))
                .env("PATH", "/usr/bin:/bin")
                .stdin(slave.try_clone().unwrap())
                .stdout(slave)
                .stderr(error)
                .spawn()
                .unwrap(),
        );
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = self.control_cli.as_mut().unwrap().try_wait().unwrap() {
                assert!(
                    status.success(),
                    "contrôle réel refusé : {}",
                    fs::read_to_string(self.root.join("control.log")).unwrap()
                );
                self.control_cli.take();
                break;
            }
            assert!(Instant::now() < deadline, "contrôle interactif hors délai");
            std::thread::sleep(Duration::from_millis(5)); // watchdog processus
        }
    }
    fn assert_no_reservation(&self, command: &str) {
        let db = rusqlite::Connection::open_with_flags(
            self.root.join("state/bridget.db"),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .unwrap();
        let count: i64 = db
            .query_row(
                "SELECT count(*) FROM spawn_commands WHERE command_id=?1",
                [command],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0, "refus après réservation interdit");
        assert!(
            !self.root.join("started").exists(),
            "un fournisseur est né après refus"
        );
        assert!(
            self.markers().load(AGENT).is_err(),
            "un processus supervisé est né après refus"
        );
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        // Ne consulter que LE marqueur de notre UUID, dans notre répertoire
        // neuf. Le helper vérifie la naissance avant tout signal de groupe.
        let _ = self.markers().stop_current_group(
            AGENT,
            Duration::from_secs(3),
            Duration::from_millis(10),
        );
        for mut child in self
            .control_cli
            .take()
            .into_iter()
            .chain(self.daemon.take())
        {
            if child.try_wait().ok().flatten().is_none() {
                let observed = Command::new("/bin/ps")
                    .args(["-p", &child.id().to_string(), "-o", "command="])
                    .output();
                if observed.is_ok_and(|p| {
                    let name = String::from_utf8_lossy(&p.stdout);
                    name.contains("bridget") && !name.to_ascii_lowercase().contains("firefox")
                }) {
                    unsafe {
                        libc::kill(child.id() as i32, libc::SIGTERM);
                    }
                    let deadline = Instant::now() + Duration::from_secs(3);
                    while child.try_wait().ok().flatten().is_none() && Instant::now() < deadline {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    if child.try_wait().ok().flatten().is_none() {
                        let _ = child.kill(); // enfant exact isolé, jamais la flotte
                    }
                    let _ = child.wait();
                }
            }
        }
        if !std::thread::panicking() {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

struct Peer {
    reader: BufReader<UnixStream>,
    writer: UnixStream,
    deadline: Instant,
}
impl Peer {
    fn new(path: &Path, deadline: Instant) -> Self {
        let writer = UnixStream::connect(path).unwrap();
        Self {
            reader: BufReader::new(writer.try_clone().unwrap()),
            writer,
            deadline,
        }
    }
    fn remaining(&self) -> Duration {
        self.deadline
            .checked_duration_since(Instant::now())
            .expect("budget global du scénario épuisé")
    }
    fn send(&mut self, message: &WrapperToDaemon) {
        self.writer
            .set_write_timeout(Some(self.remaining()))
            .unwrap();
        writeln!(self.writer, "{}", encode(message).unwrap()).unwrap();
    }
    fn receive(&mut self) -> DaemonToWrapper {
        self.reader
            .get_ref()
            .set_read_timeout(Some(self.remaining()))
            .unwrap();
        let mut line = String::new();
        assert_ne!(
            self.reader.read_line(&mut line).unwrap(),
            0,
            "EOF inattendu"
        );
        decode(line.trim_end()).unwrap()
    }
    fn register(&mut self) {
        self.send(&WrapperToDaemon::Register {
            agent_type: "core-fixture".into(),
            identity_version: 2,
            agent_id: CALLER.into(),
            host: None,
            transport: None,
            channel: None.into(),
            mode: None,
            location: None,
            os: None,
            instance_id: None,
            domain: None,
            turn_in_progress: false,
            journal_available: None,
        });
        assert!(matches!(self.receive(), DaemonToWrapper::Registered { .. }));
    }
}

#[test]
fn projet_refuse_et_session_hote_spawn_send_attach_stop_sans_docker() {
    // PATH limité aux outils système : l'ancien moteur ne peut être requis.
    assert!(!Path::new("/usr/bin/docker").exists() && !Path::new("/bin/docker").exists());
    let mut f = Fixture::new();
    let mut control = f.peer();
    control.register();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let mut order = WrapperToDaemon::SpawnOrder {
        posture: None,
        agent_type: "fixture".into(),
        project: Some(ProjectReference {
            project_id: "missing-binding".into(),
            binding_generation: 1,
        }),
        agent_id: Some(AGENT.into()),
        cwd: f.root.to_string_lossy().into_owned(),
        persistent: false,
        command_id: "core-089-host-session".into(),
        issued_at: now,
        deadline_at: now + 30,
        ownership: None,
    };
    control.send(&order);
    assert!(matches!(control.receive(), DaemonToWrapper::SpawnRejected {
        reason:SpawnRefusal::DockerRuntimeUnavailable { ref project_id }, .. } if project_id == "missing-binding"));
    f.assert_no_reservation("core-089-host-session");

    // Correction EXPLICITE de l'ordre et même command_id, pas de fallback.
    if let WrapperToDaemon::SpawnOrder { project, .. } = &mut order {
        *project = None;
    }
    control.send(&order);
    assert!(matches!(control.receive(), DaemonToWrapper::SpawnRejected {
        reason: SpawnRefusal::UnsupportedCapability { ref capability, .. }, ..
    } if capability == "posture_decouverte"));
    f.assert_no_reservation("core-089-host-session");
    // Le vrai CLI en pseudo-TTY joint le vrai daemon : aucune écriture DB
    // du harnais ne se substitue à l'autorité de configuration conservée.
    f.set_complete_via_terminal();
    control.send(&order);
    let response = control.receive();
    assert!(
        matches!(response, DaemonToWrapper::SpawnAccepted { .. }),
        "{response:?}\n{}",
        f.log()
    );
    let marker = f.markers().load(AGENT).unwrap();
    assert!(group_exists(marker.pgid).unwrap());
    assert!(f.root.join("started").exists());

    let mut view = f.peer();
    view.send(&WrapperToDaemon::RoleHandshake {
        role: ConnectionRole::Attach,
    });
    assert!(matches!(
        view.receive(),
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        }
    ));
    view.send(&WrapperToDaemon::Subscribe {
        agent: AGENT.into(),
        window: AttachWindow::Today,
    });
    assert!(matches!(view.receive(), DaemonToWrapper::Subscribed { .. }));
    loop {
        if matches!(view.receive(), DaemonToWrapper::SnapshotCaughtUp { .. }) {
            break;
        }
    }
    let message = BridgetMessage::new(CALLER, AGENT, "core-089-message");
    control.send(&WrapperToDaemon::Send(message));
    let mut journal = Vec::new();
    loop {
        match view.receive() {
            DaemonToWrapper::JournalFragment { bytes, .. } => {
                journal.extend(bytes);
                if String::from_utf8_lossy(&journal).contains("core-089-answer") {
                    break;
                }
            }
            DaemonToWrapper::Gap { .. }
            | DaemonToWrapper::End { .. }
            | DaemonToWrapper::AttachRejected { .. }
            | DaemonToWrapper::JournalReadError { .. } => panic!("journal perdu"),
            _ => {}
        }
    }
    // L'octet réponse vient du vrai journal du wrapper, pas de sa sortie CLI.
    control.send(&WrapperToDaemon::StopOrder {
        agent_id: AGENT.into(),
        command_id: "core-089-stop".into(),
    });
    loop {
        if let DaemonToWrapper::StopResult {
            command_id,
            outcome,
        } = control.receive()
        {
            assert_eq!(command_id, "core-089-stop");
            assert!(
                matches!(
                    outcome,
                    StopOutcome::Stopped | StopOutcome::StoppedForced { .. }
                ),
                "{outcome:?}"
            );
            break;
        }
    }
    assert!(
        !group_exists(marker.pgid).unwrap(),
        "le groupe fournisseur doit avoir disparu"
    );
}
