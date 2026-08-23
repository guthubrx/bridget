use maicie::store::MaicieStore;
use serde_json::{Value, json};
use std::fs;
use std::io::{Read, Write};
use std::os::fd::{FromRawFd, RawFd};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};
use uuid::Uuid;

#[test]
fn profile_propose_puis_approve_expose_le_consentement_local_et_l_outbox() {
    let fixture = Fixture::new();
    let objective_id = Uuid::new_v4().to_string();

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
        "/tmp/maicie-profile-cli",
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
    assert_eq!(proposed["screen"]["command"], "claude-code-acp");
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

    let scripted = fixture.run(&[
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
        fs::write(
            &config,
            serde_json::to_vec(&json!({
                "version": 1,
                "bridget_socket": root.join("bridget.sock"),
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
                "command": "claude-code-acp",
                "args": ["--model", "claude-fable-5"],
                "protocol": "acp",
                "forbidden_env": ["ANTHROPIC_API_KEY"],
                "pass_env": ["HOME"],
                "permissions": "allow",
                "queue_capacity": 32,
                "notify_timeout_secs": 60,
                "mcp": {"interactive": "none", "acp_session": true},
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
        }
    }

    fn run(&self, tail: &[&str]) -> std::process::Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_maicie"));
        command.args(tail);
        command.args(["--config", self.config.to_str().unwrap()]);
        command.output().unwrap()
    }

    fn approve_in_pseudo_tty(&self, approval_id: &str, confirmation: &str) -> (ExitStatus, String) {
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
        (
            status,
            String::from_utf8_lossy(&pseudo_tty.read_available()).into_owned(),
        )
    }
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
