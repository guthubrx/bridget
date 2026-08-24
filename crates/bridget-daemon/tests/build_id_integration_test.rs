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

fn start_daemon(binary: &Path, home: &Path) -> Child {
    let child = Command::new(binary)
        .arg("daemon")
        .env_clear()
        .env("HOME", home)
        .env("PATH", env::var("PATH").unwrap_or_default())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("lancer le daemon réel");

    let socket = home.join(".cache/bridget/bridget.sock");
    let deadline = Instant::now() + Duration::from_secs(5);
    while UnixStream::connect(&socket).is_err() {
        assert!(
            Instant::now() < deadline,
            "daemon réel non joignable: {socket:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
    child
}

fn status(binary: &Path, home: &Path) -> std::process::Output {
    Command::new(binary)
        .arg("status")
        .env_clear()
        .env("HOME", home)
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

    let mut daemon = start_daemon(&daemon_binary, &home);
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
    let expected = format!(
        "daemon périmé (daemon-build-test vs client-build-avance) : launchctl kickstart -k gui/{}/com.bridget.daemon\n",
        unsafe { libc::getuid() }
    );
    assert_eq!(String::from_utf8_lossy(&different_build.stderr), expected);

    daemon.kill().expect("arrêter le daemon de test");
    daemon.wait().expect("attendre le daemon de test");
    fs::remove_dir_all(root).expect("nettoyer le test de couture");
}
