//! Couture binaire du détecteur de daemon périmé.
//!
//! Les tests unitaires couvrent la comparaison et le protocole séparément.
//! Celui-ci lance le vrai daemon puis le vrai CLI : il interdit de réintroduire
//! un `ClientWelcome` qui perdrait le build-id sur le chemin de production.

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

fn temporary_root(label: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("horloge système")
        .as_nanos();
    // Les sockets Unix comptent chaque octet du chemin : le dossier temporaire
    // macOS peut déjà dépasser SUN_LEN avant même d'ajouter bridget.sock.
    PathBuf::from(format!("/tmp/bgbi-{label}-{}-{nanos}", std::process::id()))
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("racine du dépôt")
}

fn build_cli(root: &Path, target: &Path, build_id: &str) -> PathBuf {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let status = Command::new(cargo)
        .current_dir(root)
        .args([
            "build",
            "-p",
            "bridget-daemon",
            "--bin",
            "bridget",
            "--quiet",
        ])
        .env("CARGO_TARGET_DIR", target)
        .env("BRIDGET_BUILD_ID", build_id)
        .status()
        .expect("compiler le binaire réel");
    assert!(status.success(), "compilation du binaire {build_id}");
    target.join("debug/bridget")
}

fn copy_binary(source: &Path, destination: &Path) {
    fs::copy(source, destination).expect("copier le binaire compilé");
    let mut permissions = fs::metadata(destination)
        .expect("métadonnées du binaire")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(destination, permissions).expect("rendre le binaire exécutable");
}

/// Possède le daemon de couture. Créée avant le spawn : un échec d'attente
/// socket ne laisse pas l'enfant sous PID 1. `Drop` ne panique jamais.
struct DaemonGuard {
    child: Option<Child>,
}

impl DaemonGuard {
    fn start(binary: &Path, home: &Path) -> Self {
        let mut guard = Self { child: None };
        guard.child = Some(
            Command::new(binary)
                .arg("daemon")
                .env_clear()
                .env("HOME", home)
                // Machine du banc, imposée : l'oracle peut alors nommer la
                // VALEUR attendue au lieu de la recalculer.
                .env("HOSTNAME", BANC_HOST)
                .env("PATH", env::var("PATH").unwrap_or_default())
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect("lancer le daemon réel"),
        );

        let socket = home.join(".cache/bridget/bridget.sock");
        let deadline = Instant::now() + Duration::from_secs(5);
        while UnixStream::connect(&socket).is_err() {
            assert!(
                Instant::now() < deadline,
                "daemon réel non joignable: {socket:?}"
            );
            thread::sleep(Duration::from_millis(10));
        }
        guard
    }

    fn stop(mut self) {
        let stopped = stop_daemon_child_best_effort(self.child.as_mut());
        if stopped {
            let _ = self.child.take();
            return;
        }
        panic!("le daemon de couture n'a pas terminé après SIGTERM");
    }
}

impl Drop for DaemonGuard {
    fn drop(&mut self) {
        let _ = stop_daemon_child_best_effort(self.child.as_mut());
    }
}

fn stop_daemon_child_best_effort(child: Option<&mut Child>) -> bool {
    let Some(child) = child else {
        return true;
    };
    match child.try_wait() {
        Ok(Some(_)) => return true,
        Ok(None) => {}
        Err(_) => {}
    }
    let _ = unsafe { libc::kill(child.id() as i32, libc::SIGTERM) };
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        match child.try_wait() {
            Ok(Some(_)) => return true,
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(_) => break,
        }
    }
    let _ = child.kill();
    child.wait().is_ok()
}

/// Nom de machine imposé au daemon ET au client de ce banc.
const BANC_HOST: &str = "banc-attribution";

fn status(binary: &Path, home: &Path) -> std::process::Output {
    Command::new(binary)
        .arg("status")
        .env_clear()
        .env("HOME", home)
        .env("HOSTNAME", BANC_HOST)
        .env("PATH", env::var("PATH").unwrap_or_default())
        .output()
        .expect("interroger le daemon via le CLI réel")
}

#[test]
fn daemon_et_cli_reels_transmettent_et_comparent_le_build_id() {
    let root = temporary_root("seam");
    let home = root.join("home");
    let target = root.join("target");
    fs::create_dir_all(&home).expect("home isolé");

    let old_build_id = "daemon-build-test";
    let new_build_id = "client-build-avance";
    let compiled = build_cli(&repository_root(), &target, old_build_id);
    let daemon_binary = root.join("bridget-daemon");
    copy_binary(&compiled, &daemon_binary);

    let daemon = DaemonGuard::start(&daemon_binary, &home);
    let same_build = status(&daemon_binary, &home);
    assert!(same_build.status.success());
    assert!(
        String::from_utf8_lossy(&same_build.stdout).contains("Build-id daemon: daemon-build-test")
    );
    assert!(
        same_build.stderr.is_empty(),
        "égalité silencieuse: {:?}",
        same_build.stderr
    );

    // Une reconstruction ultérieure porte une identité différente, comme après
    // l'avancée de HEAD déjà vérifiée par build_identity.rs. Cette couture doit
    // alors alerter le CLI sans toucher au daemon encore en cours.
    let rebuilt = build_cli(&repository_root(), &target, new_build_id);
    let client_binary = root.join("bridget-client-avance");
    copy_binary(&rebuilt, &client_binary);
    let different_build = status(&client_binary, &home);
    assert!(different_build.status.success());
    assert!(
        String::from_utf8_lossy(&different_build.stdout)
            .contains("Build-id daemon: daemon-build-test")
    );
    // Le daemon de ce banc tourne SUR CETTE MACHINE : le verdict doit donc la
    // nommer, et la remédiation doit être la commande de CETTE plateforme.
    // Deux littéraux, un par plateforme — l'oracle nomme la valeur au lieu de
    // la recalculer avec le code de production.
    let ici = BANC_HOST;
    let remediation = if cfg!(target_os = "macos") {
        format!("launchctl kickstart -k gui/{}/com.bridget.daemon", unsafe {
            libc::getuid()
        })
    } else {
        "systemctl --user restart bridget-daemon".to_string()
    };
    let expected = format!(
        "daemon périmé sur {ici} (daemon-build-test) — client client-build-avance sur {ici} : {remediation}\n"
    );
    assert_eq!(String::from_utf8_lossy(&different_build.stderr), expected);

    daemon.stop();
    fs::remove_dir_all(root).expect("nettoyer le test de couture");
}
