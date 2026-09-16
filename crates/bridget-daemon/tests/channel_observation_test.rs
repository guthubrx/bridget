//! Oracles SPEC-024 déplacés du relais : Register → ListAgents, sans HTTP/UI.

use bridget_transport::protocol::{PresenceMode, decode, encode};
use bridget_transport::{ChannelReport, DaemonToWrapper, WrapperToDaemon};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const IO_TIMEOUT: Duration = Duration::from_secs(3);
const WATCHDOG_TIMEOUT: Duration = Duration::from_secs(30);
const AGENT_ID: &str = "b70b5e4f-a234-471a-b167-bc37fc855260";
const INSTANCE_ID: &str = "9bb24696-c0dd-4e15-a44c-0144a609c48e";

fn owned_daemon(pid: u32) -> bool {
    let Ok(observed) = Command::new("/bin/ps")
        .args(["-p", &pid.to_string(), "-o", "ppid=", "-o", "command="])
        .output()
    else {
        return false;
    };
    let command = String::from_utf8_lossy(&observed.stdout);
    command
        .split_whitespace()
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        == Some(std::process::id())
        && command.contains(env!("CARGO_BIN_EXE_bridget"))
        && !command.to_ascii_lowercase().contains("firefox")
        && unsafe { libc::getpgid(pid as i32) } == pid as i32
}

fn stop_owned_daemon(pid: u32) {
    // Jamais de PID lu dans un état de flotte : uniquement notre Child direct,
    // vérifié par parent, exécutable et groupe dédié avant chaque signal.
    if owned_daemon(pid) {
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }
    }
}

struct DaemonProcess {
    child: Child,
    cancel_watchdog: mpsc::Sender<()>,
    watchdog: Option<thread::JoinHandle<()>>,
}

impl DaemonProcess {
    fn start(root: &Path, socket: &Path) -> Self {
        let stderr = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(root.join("daemon.stderr"))
            .unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_bridget"));
        command
            .arg("daemon")
            .env_clear()
            .env("HOME", root.join("provider"))
            .env("BRIDGET_HOME", root.join("state"))
            .env("BRIDGET_SOCKET", socket)
            .env("TMPDIR", root.join("tmp"))
            .env("XDG_CACHE_HOME", root.join("xdg"))
            .env("XDG_CONFIG_HOME", root.join("xdg"))
            .env("XDG_DATA_HOME", root.join("xdg"))
            .env("XDG_STATE_HOME", root.join("xdg"))
            .env("HOSTNAME", "channel-observation-isolated")
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .current_dir(root)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(stderr);
        unsafe {
            command.pre_exec(|| {
                if libc::setpgid(0, 0) == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let child = command.spawn().expect("daemon isolé");
        let pid = child.id();
        let (cancel_watchdog, cancelled) = mpsc::channel();
        let watchdog = thread::spawn(move || {
            if matches!(
                cancelled.recv_timeout(WATCHDOG_TIMEOUT),
                Err(mpsc::RecvTimeoutError::Timeout)
            ) {
                stop_owned_daemon(pid);
            }
        });
        let mut daemon = Self {
            child,
            cancel_watchdog,
            watchdog: Some(watchdog),
        };
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            assert!(
                daemon.child.try_wait().unwrap().is_none(),
                "daemon sorti avant disponibilité : {}",
                fs::read_to_string(root.join("daemon.stderr")).unwrap_or_default()
            );
            if UnixStream::connect(socket).is_ok() {
                return daemon;
            }
            assert!(Instant::now() < deadline, "socket privée non disponible");
            thread::sleep(Duration::from_millis(5));
        }
    }

    fn stop(&mut self) -> bool {
        if matches!(self.child.try_wait(), Ok(Some(_))) {
            return true;
        }
        stop_owned_daemon(self.child.id());
        let deadline = Instant::now() + IO_TIMEOUT;
        while Instant::now() < deadline {
            if matches!(self.child.try_wait(), Ok(Some(_))) {
                return true;
            }
            thread::sleep(Duration::from_millis(5));
        }
        false
    }
}

impl Drop for DaemonProcess {
    fn drop(&mut self) {
        let stopped = self.stop();
        let _ = self.cancel_watchdog.send(());
        if let Some(watchdog) = self.watchdog.take() {
            let _ = watchdog.join();
        }
        if !stopped {
            eprintln!("arrêt de l’enfant privé {} non attesté", self.child.id());
        }
    }
}

struct Fixture {
    root: PathBuf,
    daemon: Option<DaemonProcess>,
}

impl Fixture {
    fn new() -> Self {
        let root = fs::canonicalize("/tmp").unwrap().join(format!(
            "b89-ch-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..12]
        ));
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        let mut fixture = Self { root, daemon: None };
        for directory in ["provider", "state", "tmp", "xdg"] {
            fs::DirBuilder::new()
                .mode(0o700)
                .create(fixture.root.join(directory))
                .unwrap();
        }
        assert!(fixture.socket().as_os_str().len() < 104);
        fixture.daemon = Some(DaemonProcess::start(&fixture.root, &fixture.socket()));
        fixture
    }

    fn socket(&self) -> PathBuf {
        self.root.join("state/bridget.sock")
    }

    fn finish(mut self) {
        assert!(self.daemon.as_mut().unwrap().stop(), "daemon non récolté");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        drop(self.daemon.take());
        let _ = fs::remove_dir_all(&self.root);
    }
}

struct LiveAgent {
    writer: BufWriter<UnixStream>,
    reader: BufReader<UnixStream>,
}

impl LiveAgent {
    fn connect(socket: &Path, transport: &str, channel: ChannelReport) -> Self {
        let stream = UnixStream::connect(socket).unwrap();
        stream.set_read_timeout(Some(IO_TIMEOUT)).unwrap();
        stream.set_write_timeout(Some(IO_TIMEOUT)).unwrap();
        let mut agent = Self {
            writer: BufWriter::new(stream.try_clone().unwrap()),
            reader: BufReader::new(stream),
        };
        agent.send(WrapperToDaemon::Register {
            agent_type: "channel-test".to_string(),
            identity_version: 2,
            agent_id: AGENT_ID.to_string(),
            host: Some("isolated-channel-test".to_string()),
            transport: Some(transport.to_string()),
            channel,
            mode: Some(PresenceMode::Cli),
            location: None,
            os: Some("test".to_string()),
            instance_id: Some(INSTANCE_ID.to_string()),
            domain: None,
            turn_in_progress: false,
            journal_available: Some(false),
        });
        assert!(
            matches!(agent.read(), DaemonToWrapper::Registered { agent_id, .. } if agent_id == AGENT_ID)
        );
        agent
    }

    fn unregister_with_barrier(&mut self) {
        self.send(WrapperToDaemon::Unregister);
        self.send(WrapperToDaemon::ListAgents);
        assert!(matches!(self.read(), DaemonToWrapper::AgentList { .. }));
    }

    fn listed_channel(&mut self) -> Option<String> {
        self.send(WrapperToDaemon::ListAgents);
        let DaemonToWrapper::AgentList { agents } = self.read() else {
            panic!("AgentList attendu");
        };
        agents
            .iter()
            .find(|agent| agent.agent_id == AGENT_ID)
            .expect("présence réellement enregistrée")
            .channel
            .clone()
    }

    fn send(&mut self, message: WrapperToDaemon) {
        writeln!(self.writer, "{}", encode(&message).unwrap()).unwrap();
        self.writer.flush().unwrap();
    }

    fn read(&mut self) -> DaemonToWrapper {
        let mut line = String::new();
        assert!(self.reader.read_line(&mut line).unwrap() > 0, "EOF daemon");
        decode(line.trim()).unwrap()
    }
}

fn observe_reconnection_channel(report: ChannelReport) -> Option<String> {
    let fixture = Fixture::new();
    let mut initial = LiveAgent::connect(
        &fixture.socket(),
        "cli",
        ChannelReport::Known("unix".to_string()),
    );
    assert_eq!(initial.listed_channel().as_deref(), Some("unix"));
    initial.unregister_with_barrier();
    drop(initial);
    let mut reconnected = LiveAgent::connect(&fixture.socket(), "cli", report);
    let channel = reconnected.listed_channel();
    drop(reconnected);
    fixture.finish();
    channel
}

fn observe_fresh_registration_channel(transport: &str, report: ChannelReport) -> Option<String> {
    let fixture = Fixture::new();
    let mut agent = LiveAgent::connect(&fixture.socket(), transport, report);
    let channel = agent.listed_channel();
    drop(agent);
    fixture.finish();
    channel
}

#[test]
fn spec_024_reconnexion_inconnue_explicite_efface_le_canal_precedent() {
    // Mutant : traiter Unknown comme Omitted conserverait à tort « unix ».
    assert_eq!(observe_reconnection_channel(ChannelReport::Unknown), None);
}

#[test]
fn spec_024_reconnexion_historique_omise_conserve_le_canal_precedent() {
    // Mutant inverse : une omission historique n'efface pas un fait attesté.
    assert_eq!(
        observe_reconnection_channel(ChannelReport::Omitted).as_deref(),
        Some("unix")
    );
}

#[test]
fn spec_024_inconnu_explicite_interdit_repli_transport_historique() {
    for transport in ["unix", "ssh-unix"] {
        assert_eq!(
            observe_fresh_registration_channel(transport, ChannelReport::Unknown),
            None,
            "le transport historique {transport} ne doit pas devenir un canal moderne",
        );
    }
}

#[test]
fn spec_024_omission_historique_conserve_repli_transport() {
    for transport in ["unix", "ssh-unix"] {
        assert_eq!(
            observe_fresh_registration_channel(transport, ChannelReport::Omitted).as_deref(),
            Some(transport),
            "l’omission historique doit conserver le canal {transport}",
        );
    }
}
