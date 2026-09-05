//! Les anciennes entrées moteur refusent avant namespace et fournisseur.

use std::fs;
use std::io::Read;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

struct PrivateRoot(PathBuf);

impl PrivateRoot {
    fn new() -> Self {
        let path = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "b089-retired-{}",
            &uuid::Uuid::new_v4().simple().to_string()[..12]
        ));
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        Self(path)
    }

    fn directory(&self, name: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        path
    }
}

impl Drop for PrivateRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct OwnedCli(Option<Child>);

impl Drop for OwnedCli {
    fn drop(&mut self) {
        let Some(child) = self.0.as_mut() else { return };
        if matches!(child.try_wait(), Ok(Some(_))) {
            return;
        }
        // On ne termine que le fils créé ici, vérifié avant tout signal.
        let command = Command::new("/bin/ps")
            .args(["-p", &child.id().to_string(), "-o", "command="])
            .output()
            .expect("identification du fils CLI");
        let command = String::from_utf8_lossy(&command.stdout);
        if command.is_empty() {
            let _ = child.wait();
            return;
        }
        assert!(!command.to_ascii_lowercase().contains("firefox"));
        assert!(command.contains(env!("CARGO_BIN_EXE_bridget")));
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn execute(
    binary: &Path,
    args: &[&str],
    root: &PrivateRoot,
    deadline: Instant,
) -> (i32, Vec<u8>, String) {
    let home = root.0.join("home");
    let namespace = root.0.join("state");
    let child = Command::new(binary)
        .args(args)
        .env_clear()
        .env("HOME", home)
        .env("BRIDGET_HOME", &namespace)
        .env("BRIDGET_SOCKET", namespace.join("bridget.sock"))
        .env("TMPDIR", root.0.join("tmp"))
        .env("PATH", root.0.join("bin"))
        .env("TEST_PROVIDER_SENTINEL", root.0.join("sentinel"))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut child = OwnedCli(Some(child));
    let status = loop {
        if let Some(status) = child.0.as_mut().unwrap().try_wait().unwrap() {
            break status;
        }
        assert!(Instant::now() < deadline, "commande bloquée : {args:?}");
        std::thread::sleep(Duration::from_millis(10));
    };
    let mut stdout = Vec::new();
    let mut stderr = String::new();
    child
        .0
        .as_mut()
        .unwrap()
        .stdout
        .take()
        .unwrap()
        .read_to_end(&mut stdout)
        .unwrap();
    child
        .0
        .as_mut()
        .unwrap()
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut stderr)
        .unwrap();
    (status.code().unwrap(), stdout, stderr)
}

#[test]
fn anciennes_entrees_runtime_refusees_avant_namespace_et_fournisseur() {
    let root = PrivateRoot::new();
    root.directory("home");
    root.directory("tmp");
    let bin = root.directory("bin");
    let sentinel = root.0.join("sentinel");
    fs::write(&sentinel, b"intact\n").unwrap();
    fs::set_permissions(&sentinel, fs::Permissions::from_mode(0o600)).unwrap();
    for provider in ["docker", "codex", "claude", "npx"] {
        let path = bin.join(provider);
        fs::write(
            &path,
            b"#!/bin/sh\nprintf 'provider-called\\n' > \"$TEST_PROVIDER_SENTINEL\"\nexit 89\n",
        )
        .unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }
    let deadline = Instant::now() + Duration::from_secs(30);
    let binary = Path::new(env!("CARGO_BIN_EXE_bridget"));
    for arguments in [
        vec![
            "managed-runtime-wrapper",
            "codex",
            "obsolete",
            "/bin/sh",
            "{}",
        ],
        vec!["managed-runtime-stop", "obsolete"],
        vec!["project-runtime", "prepare", "--project", "obsolete"],
        vec!["project-round", "dispatch"],
        vec!["cleanup", "--dry-run"],
        vec!["daemon", "--project-root-policy", "/never-read"],
        vec!["daemon", "--project-runtime-policy", "/never-read"],
        vec!["daemon", "--project-resource-catalog", "/never-read"],
    ] {
        let (code, stdout, stderr) = execute(binary, &arguments, &root, deadline);
        assert_eq!(code, 2, "{arguments:?} : {stderr}");
        assert!(stdout.is_empty(), "{arguments:?} : {stdout:?}");
        assert!(stderr.contains(arguments[0]), "{stderr}");
        if arguments[0] == "daemon" {
            assert!(stderr.contains(arguments[1]), "{stderr}");
        } else if arguments[0] == "cleanup" {
            assert!(
                stderr.contains("inventaire des worktrees retiré"),
                "{stderr}"
            );
        } else {
            assert!(stderr.contains("runtime de projet retiré"), "{stderr}");
        }
        // Mutant : réintroduire le dispatch ou déplacer le refus après
        // initialize_process crée state ou atteint le faux fournisseur.
        assert!(
            !root.0.join("state").exists(),
            "namespace créé par {arguments:?}"
        );
        assert_eq!(fs::read(&sentinel).unwrap(), b"intact\n", "{arguments:?}");
    }
}

#[test]
fn posture_humaine_refusee_sans_tty_avant_toute_connexion_de_controle() {
    let root = PrivateRoot::new();
    root.directory("home");
    root.directory("tmp");
    root.directory("bin");
    let state = root.directory("state");
    let socket = state.join("bridget.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    fs::set_permissions(&socket, fs::Permissions::from_mode(0o600)).unwrap();
    listener.set_nonblocking(true).unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    for posture in ["discovery", "complete"] {
        let (code, stdout, stderr) = execute(
            Path::new(env!("CARGO_BIN_EXE_bridget")),
            &["control", "posture", posture],
            &root,
            deadline,
        );
        assert_eq!(code, 2, "{stderr}");
        assert!(stdout.is_empty());
        assert!(
            stderr.contains("contrôle du référent = terminal interactif uniquement"),
            "{stderr}"
        );
        // Mutant : retirer la garde TTY atteint cette socket et ne peut plus
        // produire le refus avant toute négociation/lecture de génération.
        assert!(
            matches!(listener.accept(), Err(error) if error.kind() == std::io::ErrorKind::WouldBlock)
        );
        assert!(!state.join("bridget.db").exists());
    }
}
