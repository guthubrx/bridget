use maicie::store::MaicieStore;
use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::os::fd::{FromRawFd, RawFd};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::thread;
use uuid::Uuid;

/// Chemin absolu synthétique partagé par l'assertion et la fixture écrite.
const FIXTURE_CLAUDE_COMMAND: &str = "/opt/bridget-fixtures/bin/claude";

#[test]
fn profile_propose_puis_approve_expose_le_consentement_local_et_l_outbox() {
    let fixture = Fixture::new();
    let objective_id = Uuid::new_v4().to_string();

    // Mutation : un cwd partagé (/tmp/maicie-profile-cli) ferait coller deux
    // exécutions parallèles sur le même répertoire — l'unicité sous fixture.root
    // isole l'effet de bord.
    let cwd = fixture.root.join("agent-cwd");
    fs::create_dir_all(&cwd).unwrap();
    let proposed = fixture.run(&[
        "profile",
        "propose",
        &objective_id,
        "review-hostile",
        "--definition",
        fixture.definition.to_str().unwrap(),
        "--context-scope",
        "objective:review",
        "--cwd",
        cwd.to_str().unwrap(),
        "--persistent",
        "--reason",
        "profil absent compatible",
        "--json",
    ]);
    assert!(
        proposed.status.success(),
        "{}",
        String::from_utf8_lossy(&proposed.stderr)
    );
    let proposed: Value = serde_json::from_slice(&proposed.stdout).unwrap();
    assert_eq!(proposed["kind"], "proposed");
    assert_eq!(proposed["screen"]["command"], FIXTURE_CLAUDE_COMMAND);
    assert_eq!(
        proposed["screen"]["args"],
        json!(["--model", "claude-fable-5"])
    );
    let approval_id = proposed["approval_id"].as_str().unwrap().to_string();
    assert!(
        MaicieStore::open(&fixture.database)
            .unwrap()
            .pending_activation_outboxes()
            .unwrap()
            .is_empty()
    );

    let refused = fixture.run(&[
        "profile",
        "approve",
        &approval_id,
        "--definition",
        fixture.definition.to_str().unwrap(),
    ]);
    assert!(!refused.status.success());
    assert!(
        String::from_utf8_lossy(&refused.stderr)
            .contains("approbation = terminal interactif uniquement")
    );

    let scripted = fixture.run_without_daemon(&[
        "profile",
        "approve",
        &approval_id,
        "--definition",
        fixture.definition.to_str().unwrap(),
        "--confirm",
    ]);
    assert!(!scripted.status.success());
    assert!(String::from_utf8_lossy(&scripted.stderr).contains("option profile approve inconnue"));

    let (status, rendered) = fixture.approve_in_pseudo_tty(&approval_id, "oui\n");
    assert!(status.success(), "{rendered}");
    assert!(rendered.contains("Approbation locale du profil"));
    assert!(rendered.contains("args=[\"--model\",\"claude-fable-5\"]"));
    assert!(rendered.contains("Tapez oui pour approuver"));
    assert!(rendered.contains("actor=local_human"));
    let pending = MaicieStore::open(&fixture.database)
        .unwrap()
        .pending_activation_outboxes()
        .unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].approval.id.to_string(), approval_id);
}

struct Fixture {
    root: PathBuf,
    database: PathBuf,
    config: PathBuf,
    definition: PathBuf,
    socket: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        // Le socket de Bridget reste sous la borne Unix de 103 octets.
        let root = PathBuf::from("/tmp").join(format!("mcp-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        let database = root.join("maicie.sqlite3");
        let config = root.join("maicie.json");
        let definition = root.join("resolved-definition.json");
        let socket = root.join("bridget.sock");
        fs::write(
            &config,
            serde_json::to_vec(&json!({
                "version": 1,
                "bridget_socket": socket,
                "database_path": database,
                "durations": {"short_secs": 30, "normal_secs": 60, "long_secs": 90},
                "profiles": [{
                    "id": "review-hostile",
                    "agent_name": "cxbridget",
                    "display_name": "Revue hostile",
                    "agent_type": "claude",
                    "model": "claude-fable-5",
                    "effort": "raisonnement-renforce",
                    "tags": ["review", "security"],
                    "personality_ref": "profiles/cxbridget.md",
                    "tools": ["bridget_send"],
                    "spawn_order_ref": "agents/claude-review"
                }]
            }))
            .unwrap(),
        )
        .unwrap();
        fs::write(
            &definition,
            serde_json::to_vec(&json!({
                "command": FIXTURE_CLAUDE_COMMAND,
                "args": ["--model", "claude-fable-5"],
                "protocol": "claude_stream_json",
                "forbidden_env": ["ANTHROPIC_API_KEY"],
                "pass_env": ["HOME"],
                "permissions": "allow",
                "queue_capacity": 32,
                "notify_timeout_secs": 60,
                "mcp": {"interactive": "none", "acp_session": false},
                "digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            }))
            .unwrap(),
        )
        .unwrap();
        Self {
            root,
            database,
            config,
            definition,
            socket,
        }
    }

    fn run(&self, tail: &[&str]) -> std::process::Output {
        let server = start_local_daemon(&self.socket);
        let mut command = Command::new(env!("CARGO_BIN_EXE_maicie"));
        command.args(tail);
        command.args(["--config", self.config.to_str().unwrap()]);
        let output = command.output().unwrap();
        server.join().unwrap();
        output
    }

    fn run_without_daemon(&self, tail: &[&str]) -> std::process::Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_maicie"));
        command.args(tail);
        command.args(["--config", self.config.to_str().unwrap()]);
        command.output().unwrap()
    }

    fn approve_in_pseudo_tty(&self, approval_id: &str, confirmation: &str) -> (ExitStatus, String) {
        let server = start_local_daemon(&self.socket);
        let pseudo_tty = PseudoTerminal::open();
        let mut command = Command::new(env!("CARGO_BIN_EXE_maicie"));
        command.args([
            "profile",
            "approve",
            approval_id,
            "--definition",
            self.definition.to_str().unwrap(),
            "--config",
            self.config.to_str().unwrap(),
        ]);
        command
            .stdin(Stdio::from(pseudo_tty.slave_file()))
            .stdout(Stdio::from(pseudo_tty.slave_file()))
            .stderr(Stdio::from(pseudo_tty.slave_file()));
        let mut child = command.spawn().unwrap();
        pseudo_tty.write_all(confirmation.as_bytes());
        let status = child.wait().unwrap();
        server.join().unwrap();
        (
            status,
            String::from_utf8_lossy(&pseudo_tty.read_available()).into_owned(),
        )
    }
}

fn start_local_daemon(socket: &std::path::Path) -> thread::JoinHandle<()> {
    let socket = socket.to_owned();
    let (ready_tx, ready_rx) = mpsc::channel();
    let server = thread::spawn(move || {
        let _ = fs::remove_file(&socket);
        let listener = UnixListener::bind(&socket).unwrap();
        ready_tx.send(()).unwrap();
        accept_daemon_identity(&listener);
        accept_empty_guichet(&listener);
        accept_empty_coordination(&listener);
    });
    ready_rx.recv().unwrap();
    server
}

fn accept_daemon_identity(listener: &UnixListener) {
    let (stream, _) = listener.accept().unwrap();
    let (mut reader, mut writer) = split(stream);
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"RoleHandshake","role":"client"})
    );
    write_json(&mut writer, json!({"type":"RoleAccepted","role":"client"}));
    assert_eq!(read_json(&mut reader)["type"], "ClientHello");
    write_json(
        &mut writer,
        json!({
            "type":"ClientWelcome",
            "version":1,
            "horizon_secs":3600,
            "issued_at_tolerance_secs":30,
            "capabilities":["send_idempotent","lookup"]
        }),
    );
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"DaemonIdentityRequest"})
    );
    write_json(
        &mut writer,
        json!({
            "type":"DaemonIdentityReport",
            "host":bridget_core::local_host(),
            "db_path":"/var/lib/bridget/bridget.db"
        }),
    );
}

fn accept_empty_guichet(listener: &UnixListener) {
    let (stream, _) = listener.accept().unwrap();
    let (mut reader, mut writer) = split(stream);
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"RoleHandshake","role":"service"})
    );
    write_json(&mut writer, json!({"type":"RoleAccepted","role":"service"}));
    let hello = read_json(&mut reader);
    assert_eq!(hello["type"], "ServiceHello");
    assert_eq!(hello["service"], "maicie");
    assert_eq!(hello["capabilities"], json!(["maicie_guichet"]));
    write_json(
        &mut writer,
        json!({
            "type":"ServiceWelcome",
            "version":1,
            "horizon_secs":3600,
            "issued_at_tolerance_secs":30,
            "capabilities":["maicie_guichet"]
        }),
    );
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"guichet_claim_next","v":1})
    );
    write_json(&mut writer, json!({"type":"guichet_empty","v":1}));
}

fn accept_empty_coordination(listener: &UnixListener) {
    let (stream, _) = listener.accept().unwrap();
    let (mut reader, mut writer) = split(stream);
    assert_eq!(
        read_json(&mut reader),
        json!({"type":"RoleHandshake","role":"service"})
    );
    write_json(&mut writer, json!({"type":"RoleAccepted","role":"service"}));
    let hello = read_json(&mut reader);
    assert_eq!(
        hello["capabilities"],
        json!(["maicie_guichet", "coordination_events_v2"])
    );
    write_json(
        &mut writer,
        json!({
            "type":"ServiceWelcome",
            "version":1,
            "horizon_secs":3600,
            "issued_at_tolerance_secs":30,
            "capabilities":["maicie_guichet","coordination_events_v2"]
        }),
    );
    assert_eq!(read_json(&mut reader)["type"], "coordination_subscribe");
    write_json(
        &mut writer,
        json!({"type":"coordination_snapshot_caught_up","v":2}),
    );
}

fn split(stream: UnixStream) -> (BufReader<UnixStream>, BufWriter<UnixStream>) {
    (
        BufReader::new(stream.try_clone().unwrap()),
        BufWriter::new(stream),
    )
}

fn read_json(reader: &mut BufReader<UnixStream>) -> Value {
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    serde_json::from_str(&line).unwrap()
}

fn write_json(writer: &mut BufWriter<UnixStream>, value: Value) {
    serde_json::to_writer(&mut *writer, &value).unwrap();
    writer.write_all(b"\n").unwrap();
    writer.flush().unwrap();
}

/// Réutilise le motif `openpty` des tests attach du chantier : le test traverse
/// le binaire avec stdin et stdout réellement TTY, sans simuler la branche de
/// sécurité dans le processus parent.
struct PseudoTerminal {
    master: RawFd,
    slave: RawFd,
}

impl PseudoTerminal {
    fn open() -> Self {
        let mut master = -1;
        let mut slave = -1;
        assert_eq!(
            unsafe {
                libc::openpty(
                    &mut master,
                    &mut slave,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            },
            0,
            "openpty: {}",
            std::io::Error::last_os_error()
        );
        Self { master, slave }
    }

    fn slave_file(&self) -> fs::File {
        let fd = unsafe { libc::dup(self.slave) };
        assert!(fd >= 0, "dup slave: {}", std::io::Error::last_os_error());
        unsafe { fs::File::from_raw_fd(fd) }
    }

    fn write_all(&self, bytes: &[u8]) {
        let fd = unsafe { libc::dup(self.master) };
        assert!(fd >= 0, "dup master: {}", std::io::Error::last_os_error());
        let mut file = unsafe { fs::File::from_raw_fd(fd) };
        file.write_all(bytes).unwrap();
        file.flush().unwrap();
    }

    fn read_available(&self) -> Vec<u8> {
        let fd = unsafe { libc::dup(self.master) };
        assert!(fd >= 0, "dup master: {}", std::io::Error::last_os_error());
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        assert!(
            flags >= 0,
            "fcntl F_GETFL: {}",
            std::io::Error::last_os_error()
        );
        assert_eq!(
            unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) },
            0,
            "fcntl F_SETFL: {}",
            std::io::Error::last_os_error()
        );
        let mut file = unsafe { fs::File::from_raw_fd(fd) };
        let mut output = Vec::new();
        loop {
            let mut chunk = [0_u8; 4096];
            match file.read(&mut chunk) {
                Ok(0) => break,
                Ok(count) => output.extend_from_slice(&chunk[..count]),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(error) => panic!("lecture pseudo-TTY: {error}"),
            }
        }
        output
    }
}

impl Drop for PseudoTerminal {
    fn drop(&mut self) {
        for fd in [self.master, self.slave] {
            if fd >= 0 {
                assert_eq!(unsafe { libc::close(fd) }, 0);
            }
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
