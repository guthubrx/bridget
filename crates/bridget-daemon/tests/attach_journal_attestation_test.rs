//! Oracles de couture : attestation véridique du journal pour attach.
//!
//! Côté positif — un wrapper interactif réel ouvre son journal et devient
//! attachable. Côté négatif — un pilote qui annonce `false` sans JournalReady
//! reste refusé. Aucune inversion en dur du booléen.

use bridget_core::BridgetMessage;
use bridget_transport::protocol::{ConnectionRole, PresenceMode, decode, encode};
use bridget_transport::{AttachRefusal, AttachWindow, DaemonToWrapper, WrapperToDaemon};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

struct DaemonProcess(Child);

impl DaemonProcess {
    fn start(home: &Path) -> Self {
        let child = Command::new(env!("CARGO_BIN_EXE_bridget"))
            .arg("daemon")
            .env_clear()
            .env("HOME", home)
            .env("BRIDGET_HOME", home.join("state"))
            .env("BRIDGET_SOCKET", home.join("state/bridget.sock"))
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("daemon réel démarré");
        let daemon = Self(child);
        let socket = home.join("state/bridget.sock");
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if UnixStream::connect(&socket).is_ok() {
                return daemon;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("daemon non prêt");
    }
}

impl Drop for DaemonProcess {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            assert_eq!(unsafe { libc::kill(self.0.id() as i32, libc::SIGTERM) }, 0);
            let _ = self.0.wait();
        }
    }
}

struct InteractiveWrapper {
    child: Child,
    process_group_id: i32,
}

impl InteractiveWrapper {
    fn start(root: &Path, agent: &Path, session: &str) -> (Self, PathBuf) {
        let marker = root.join("agent-name-file-path");
        let mut command = Command::new(env!("CARGO_BIN_EXE_bridget"));
        command
            .args(["--", agent.to_str().expect("agent UTF-8"), session])
            .env_clear()
            .env("HOME", root)
            .env("BRIDGET_HOME", root.join("state"))
            .env("BRIDGET_SOCKET", root.join("state/bridget.sock"))
            // Session 097 : un type resté tmux exige un pane attesté par tmux
            // lui-même ; la fixture fournit un tmux factice qui atteste un pane
            // et accepte le collage, pour rester un stand-in interactif.
            .env(
                "PATH",
                format!("{}:/usr/bin:/bin", root.join("bin").display()),
            )
            .env("BRIDGET_TEST_NAME_FILE_PATH", &marker)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        unsafe {
            command.pre_exec(|| {
                if libc::setpgid(0, 0) == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let child = command.spawn().expect("wrapper interactif démarré");
        let process = Self {
            process_group_id: child.id() as i32,
            child,
        };
        let name_state = wait_until("wrapper enregistré", || {
            std::fs::read_to_string(&marker)
                .ok()
                .map(|path| PathBuf::from(path.trim()))
                .filter(|path| path.exists())
        });
        (process, name_state)
    }

    fn terminate(&mut self) {
        if self.child.try_wait().ok().flatten().is_some() {
            return;
        }
        let _ = unsafe { libc::kill(-self.process_group_id, libc::SIGTERM) };
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if self.child.try_wait().ok().flatten().is_some() {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("wrapper de test encore vivant après SIGTERM");
    }
}

impl Drop for InteractiveWrapper {
    fn drop(&mut self) {
        self.terminate();
    }
}

fn wait_until<T>(label: &str, mut condition: impl FnMut() -> Option<T>) -> T {
    let deadline = Instant::now() + Duration::from_secs(8);
    while Instant::now() < deadline {
        if let Some(value) = condition() {
            return value;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("timeout: {label}");
}

fn write_frame(writer: &mut BufWriter<UnixStream>, message: &WrapperToDaemon) {
    writeln!(writer, "{}", encode(message).expect("encodage")).expect("écriture");
    writer.flush().expect("flush");
}

fn read_frame(reader: &mut BufReader<UnixStream>) -> DaemonToWrapper {
    let mut line = String::new();
    reader.read_line(&mut line).expect("lecture");
    decode(line.trim_end()).expect("décodage")
}

fn attach_role(socket: &Path) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
    let stream = UnixStream::connect(socket).expect("connexion attach");
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .expect("timeout");
    let mut reader = BufReader::new(stream.try_clone().expect("clone"));
    let mut writer = BufWriter::new(stream);
    write_frame(
        &mut writer,
        &WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Attach,
        },
    );
    assert!(matches!(
        read_frame(&mut reader),
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Attach
        }
    ));
    (reader, writer)
}

fn prepare_interactive_fixture(root: &Path) -> PathBuf {
    let config = root.join("state");
    std::fs::create_dir_all(&config).expect("config");
    let agent = root.join("fixture-agent");
    std::fs::write(
        &agent,
        "#!/bin/sh\nprintf '%s' \"$BRIDGET_AGENT_ID_FILE\" > \"$BRIDGET_TEST_NAME_FILE_PATH\"\nwhile :; do /bin/sleep 1; done\n",
    )
    .expect("agent");
    std::fs::set_permissions(&agent, std::fs::Permissions::from_mode(0o700)).expect("chmod");
    let bin = root.join("bin");
    std::fs::create_dir_all(&bin).expect("bin");
    let tmux = bin.join("tmux");
    std::fs::write(
        &tmux,
        "#!/bin/sh\ncase \"$1\" in\n  display-message) printf '%%1\\tfixture:0.0\\n' ;;\n  load-buffer) cat >/dev/null ;;\n  show-buffer) exit 1 ;;\n  *) : ;;\nesac\nexit 0\n",
    )
    .expect("tmux factice");
    std::fs::set_permissions(&tmux, std::fs::Permissions::from_mode(0o700)).expect("chmod tmux");
    let registry = serde_json::json!({
        "agents": {
            "fixture": {
                "command": agent,
                "protocol": "tmux",
                "permissions": "allow",
                "mcp": {"interactive": "none"}
            }
        }
    });
    let registry_path = config.join("agents.json");
    std::fs::write(&registry_path, registry.to_string()).expect("registre");
    std::fs::set_permissions(&registry_path, std::fs::Permissions::from_mode(0o600))
        .expect("registre privé");
    agent
}

fn test_root(label: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("horloge")
        .as_nanos();
    let root = PathBuf::from(format!(
        "/tmp/br-attach-journal-{label}-{}-{nanos}",
        std::process::id()
    ));
    bridget_daemon::environment::Namespace::resolve(
        Some(root.join("state")),
        None,
        Some(root.clone()),
    )
    .unwrap()
    .prepare()
    .unwrap();
    root
}

#[test]
fn wrapper_interactif_avec_journal_actif_est_attachable() {
    let root = test_root("avec-journal");
    let agent = prepare_interactive_fixture(&root);
    let _daemon = DaemonProcess::start(&root);
    let socket = root.join("state/bridget.sock");
    let (mut wrapper, name_state) =
        InteractiveWrapper::start(&root, &agent, "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa");
    let agent_name = std::fs::read_to_string(&name_state)
        .expect("nom")
        .trim()
        .to_string();

    // Preuve empirique : le répertoire de journal existe après le spawn réel
    // (le fichier jsonl n'apparaît qu'à la première écriture).
    wait_until("répertoire journal interactif créé", || {
        let dir = root.join("state/sessions").join(&agent_name);
        dir.is_dir().then_some(())
    });

    let (mut attach_reader, mut attach_writer) = attach_role(&socket);
    let subscription_id = wait_until("abonnement attach accepté", || {
        write_frame(
            &mut attach_writer,
            &WrapperToDaemon::Subscribe {
                agent: agent_name.clone(),
                window: AttachWindow::Seq(0),
            },
        );
        match read_frame(&mut attach_reader) {
            DaemonToWrapper::Subscribed { subscription_id } => Some(subscription_id),
            DaemonToWrapper::AttachRejected { .. } => None,
            other => panic!("réponse attach inattendue: {other:?}"),
        }
    });

    // Livraison réelle → entrée journal → fragment attach.
    let sender_stream = UnixStream::connect(&socket).expect("sender");
    sender_stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let mut sender_reader = BufReader::new(sender_stream.try_clone().expect("clone sender"));
    let mut sender_writer = BufWriter::new(sender_stream);
    write_frame(
        &mut sender_writer,
        &WrapperToDaemon::Register {
            agent_type: "cli".into(),
            identity_version: 2,
            agent_id: "89000000-0000-4000-8000-000000000501".into(),
            host: Some("test".into()),
            transport: Some("unix".into()),
            channel: None.into(),
            mode: Some(PresenceMode::Cli),
            location: None,
            os: Some("test".into()),
            instance_id: Some("sender-journal-instance".into()),
            domain: None,
            turn_in_progress: false,
            journal_available: Some(false),
        },
    );
    assert!(matches!(
        read_frame(&mut sender_reader),
        DaemonToWrapper::Registered { .. }
    ));
    let mut mission = BridgetMessage::new(
        "89000000-0000-4000-8000-000000000501",
        &agent_name,
        "mission journalisée",
    );
    mission.id = "mission-interactive-journal".into();
    write_frame(&mut sender_writer, &WrapperToDaemon::Send(mission));
    assert!(matches!(
        read_frame(&mut sender_reader),
        DaemonToWrapper::Ack { .. }
    ));

    let deadline = Instant::now() + Duration::from_secs(3);
    let mut saw_journal = false;
    while Instant::now() < deadline {
        match read_frame(&mut attach_reader) {
            DaemonToWrapper::JournalFragment {
                subscription_id: received,
                ..
            } if received == subscription_id => {
                saw_journal = true;
                break;
            }
            DaemonToWrapper::SnapshotCaughtUp { .. } => {}
            other => panic!("flux attach inattendu: {other:?}"),
        }
    }
    assert!(
        saw_journal,
        "attach n'a reçu aucun fragment du journal interactif"
    );

    wrapper.terminate();
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn pilote_sans_journal_reste_refuse_par_le_gate() {
    let root = test_root("sans-journal");
    let _daemon = DaemonProcess::start(&root);
    let socket = root.join("state/bridget.sock");

    let stream = UnixStream::connect(&socket).expect("connexion");
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let mut reader = BufReader::new(stream.try_clone().expect("clone"));
    let mut writer = BufWriter::new(stream);
    write_frame(
        &mut writer,
        &WrapperToDaemon::Register {
            agent_type: "fixture".into(),
            identity_version: 2,
            agent_id: "89000000-0000-4000-8000-000000000502".into(),
            host: Some("test".into()),
            transport: Some("unix".into()),
            channel: None.into(),
            mode: Some(PresenceMode::Cli),
            location: None,
            os: Some("test".into()),
            instance_id: Some("sans-journal-instance".into()),
            domain: None,
            turn_in_progress: false,
            // Attestation explicite : pas de journal, pas de JournalReady.
            journal_available: Some(false),
        },
    );
    assert!(matches!(
        read_frame(&mut reader),
        DaemonToWrapper::Registered { .. }
    ));

    let (mut attach_reader, mut attach_writer) = attach_role(&socket);
    write_frame(
        &mut attach_writer,
        &WrapperToDaemon::Subscribe {
            agent: "89000000-0000-4000-8000-000000000502".into(),
            window: AttachWindow::Seq(0),
        },
    );
    match read_frame(&mut attach_reader) {
        DaemonToWrapper::AttachRejected {
            reason: AttachRefusal::JournalUnavailable,
            ..
        } => {}
        other => panic!("refus journal attendu, obtenu: {other:?}"),
    }

    std::fs::remove_dir_all(root).ok();
}
