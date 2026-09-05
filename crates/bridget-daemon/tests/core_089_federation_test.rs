//! SSH local RÉEL, namespace privé, aucun sshd/config/authorized_keys système modifié.
//! Recette opt-in : nécessite OpenSSH et un compte local autorisé à s'authentifier.
#[path = "support/idempotent.rs"]
pub mod fixture;
use bridget_core::BridgetMessage;
use bridget_transport::WrapperToDaemon;
use fixture::*;
use std::fs;
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::{FileTypeExt, OpenOptionsExt, PermissionsExt};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

struct SshChild {
    child: Child,
    marker: PathBuf,
}
impl SshChild {
    fn start(command: &mut Command, root: &Path, label: &str) -> Self {
        let log = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(root.join(format!("{label}.log")))
            .unwrap();
        let child = command
            .stdin(Stdio::null())
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log))
            .process_group(0)
            .spawn()
            .expect("enfant SSH isolé");
        Self {
            child,
            marker: root.to_path_buf(),
        }
    }
    fn stop(mut self) {
        self.terminate();
        assert!(
            self.child.try_wait().unwrap().is_some(),
            "processus SSH survivant"
        );
    }

    fn terminate(&mut self) {
        if matches!(self.child.try_wait(), Ok(Some(_))) {
            return;
        }
        let pid = self.child.id();
        let observed = Command::new("/bin/ps")
            .args(["-p", &pid.to_string(), "-o", "ppid=", "-o", "command="])
            .output();
        if let Ok(observed) = observed {
            let observed = String::from_utf8_lossy(&observed.stdout);
            if observed
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<u32>().ok())
                == Some(std::process::id())
                && observed.contains(self.marker.to_str().unwrap())
                && !observed.to_lowercase().contains("firefox")
            {
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }
            }
        }
        let deadline = Instant::now() + Duration::from_secs(3);
        while !matches!(self.child.try_wait(), Ok(Some(_))) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        if !matches!(self.child.try_wait(), Ok(Some(_))) {
            eprintln!("nettoyage SSH non attesté pour l'enfant {pid}");
        }
    }
}

impl Drop for SshChild {
    fn drop(&mut self) {
        self.terminate();
    }
}

fn ready(child: &mut SshChild, log: &Path, predicate: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        assert!(
            child.child.try_wait().unwrap().is_none(),
            "enfant arrêté : {}",
            fs::read_to_string(log).unwrap()
        );
        if predicate() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "borne de démarrage : {}",
            fs::read_to_string(log).unwrap()
        );
        std::thread::sleep(Duration::from_millis(5)); // observation de disponibilité, pas délai métier
    }
}

#[test]
#[ignore = "recette réelle opt-in : BRIDGET_SSH_LOCAL_GATE=1, sshd local requis"]
fn ssh_unix_reel_garde_annuaire_ledger_permissions_et_coupure_honnete() {
    assert_eq!(std::env::var("BRIDGET_SSH_LOCAL_GATE").as_deref(), Ok("1"));
    let root = fs::canonicalize(test_root("089-ssh")).unwrap();
    // StrictModes parcourt les parents d'AuthorizedKeysFile : /tmp, partagé,
    // est refusé par sshd même si son sous-répertoire est privé. Clés de TEST
    // sous target, parents possédés ; jamais ~/.ssh/authorized_keys.
    let keys = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target")
        .join(format!("ssh-089-{}", uuid::Uuid::new_v4()));
    private_dir(&keys).unwrap();
    let keys = fs::canonicalize(keys).unwrap();
    for key in ["host", "client"] {
        let status = Command::new("/usr/bin/ssh-keygen")
            .args(["-q", "-t", "ed25519", "-N", "", "-f"])
            .arg(keys.join(key))
            .status()
            .unwrap();
        assert!(status.success());
    }
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let user = Command::new("/usr/bin/id").arg("-un").output().unwrap();
    let user = String::from_utf8(user.stdout).unwrap().trim().to_owned();
    assert!(user.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_'));
    // OpenSSH 10 session.c::do_authenticated désactive aussi l'ACL Unix si
    // AllowTcpForwarding=no. Autoriser la direction remote, mais borner le TCP
    // à loopback:1 (non utilisé, non accessible à ce sshd sans privilèges).
    // Source : openssh-portable V_10_0_P2/session.c, lignes 329–344.
    let config = format!(
        "ListenAddress 127.0.0.1\nPort {port}\nHostKey {host}\nPidFile {pid}\nAuthorizedKeysFile {client}\nPasswordAuthentication no\nKbdInteractiveAuthentication no\nPubkeyAuthentication yes\nUsePAM no\nStrictModes yes\nAllowUsers {user}\nPermitRootLogin no\nAllowTcpForwarding remote\nPermitListen 127.0.0.1:1\nPermitOpen none\nAllowStreamLocalForwarding yes\nStreamLocalBindMask 0177\nStreamLocalBindUnlink no\nX11Forwarding no\nPermitUserEnvironment no\nPermitUserRC no\nLogLevel VERBOSE\n",
        host = keys.join("host").display(),
        pid = root.join("sshd.pid").display(),
        client = keys.join("client.pub").display()
    );
    private_write(&root.join("sshd_config"), config).unwrap();
    let public_key = fs::read_to_string(keys.join("host.pub")).unwrap();
    private_write(
        &keys.join("known_hosts"),
        format!("[127.0.0.1]:{port} {public_key}"),
    )
    .unwrap();
    let mut server = SshChild::start(
        Command::new("/usr/sbin/sshd")
            .args(["-D", "-e", "-f"])
            .arg(root.join("sshd_config")),
        &root,
        "sshd",
    );
    ready(&mut server, &root.join("sshd.log"), || {
        TcpStream::connect(("127.0.0.1", port)).is_ok()
    });

    let store = fixture_store(&root.join("state/bridget.db")).unwrap();
    let mut message = BridgetMessage::new(ACTOR, RECIPIENT, "SSH maître : été $VAR\nligne intacte");
    message.id = "089-ssh-ledger".into();
    store.record_message(&message, &message.id).unwrap();
    drop(store);
    let daemon = spawn_daemon(&root, None);
    let actor = register_agent_as(&socket(&root), ACTOR, "089-ssh-actor");
    let remote_root = root.join("remote");
    let remote_socket = remote_root.join("peer.sock");
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/federate-ssh.sh");
    let mut command = Command::new("/bin/bash");
    command
        .arg(script)
        .args([
            "run",
            "--label",
            "089-local",
            "--host",
            "127.0.0.1",
            "--user",
            &user,
            "--port",
            &port.to_string(),
            "--identity",
        ])
        .arg(keys.join("client"))
        .arg("--known-hosts")
        .arg(keys.join("known_hosts"))
        .arg("--root")
        .arg(root.join("state"))
        .arg("--socket")
        .arg(socket(&root))
        .arg("--remote-root")
        .arg(&remote_root)
        .arg("--remote-socket")
        .arg(&remote_socket);
    let mut tunnel = SshChild::start(&mut command, &root, "tunnel");
    ready(&mut tunnel, &root.join("tunnel.log"), || {
        fs::metadata(&remote_socket).is_ok()
    });
    let metadata = fs::symlink_metadata(&remote_socket).unwrap();
    assert!(metadata.file_type().is_socket());
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
    assert_eq!(
        fs::metadata(&remote_root).unwrap().permissions().mode() & 0o777,
        0o700
    );
    let mut direct = Client::connect(&socket(&root));
    let mut forwarded = Client::connect(&remote_socket);
    direct.send(WrapperToDaemon::ListAgents);
    forwarded.send(WrapperToDaemon::ListAgents);
    assert_eq!(
        serde_json::to_value(direct.receive()).unwrap(),
        serde_json::to_value(forwarded.receive()).unwrap()
    );
    let local = run_isolated(&root, &["ledger", "--limit", "20"], false);
    let mut distant_command = isolated_command(&root);
    distant_command
        .env("BRIDGET_HOME", &remote_root)
        .env("BRIDGET_SOCKET", &remote_socket)
        .env("BRIDGET_CHANNEL", "ssh-unix")
        .args(["ledger", "--limit", "20"]);
    let distant = run_command(distant_command);
    assert!(
        local.status.success() && distant.status.success(),
        "{}",
        output_text(&distant)
    );
    assert_eq!(
        local.stdout, distant.stdout,
        "même source et mêmes octets à travers SSH"
    );
    assert!(!remote_root.join("bridget.db").exists());
    drop(forwarded);
    tunnel.stop(); // Seul le tunnel de TEST est arrêté, pas le daemon ni l'agent.
    let mut offline_command = isolated_command(&root);
    offline_command
        .env("BRIDGET_HOME", &remote_root)
        .env("BRIDGET_SOCKET", &remote_socket)
        .args(["ledger"]);
    let offline = run_command(offline_command);
    assert!(
        !offline.status.success(),
        "une coupure ne devient pas un ledger vide"
    );
    assert!(offline.stdout.is_empty());
    assert!(!remote_root.join("bridget.db").exists());
    direct.send(WrapperToDaemon::ListAgents);
    assert!(
        serde_json::to_string(&direct.receive())
            .unwrap()
            .contains(ACTOR)
    );
    // Si OpenSSH conserve une socket stale, elle n'autorise jamais unlink
    // automatique : la recette de reconnexion T027 vérifie cette frontière.
    eprintln!(
        "socket après arrêt tunnel : existe={}",
        remote_socket.exists()
    );
    drop(actor);
    drop(direct);
    daemon.stop();
    server.stop();
    fs::remove_dir_all(&root).unwrap();
    fs::remove_dir_all(&keys).unwrap();
}
