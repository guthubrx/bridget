//! Couture binaire du détecteur de daemon périmé.
//!
//! Les tests unitaires couvrent la comparaison et le protocole séparément.
//! Celui-ci lance le vrai daemon puis le vrai CLI : il interdit de réintroduire
//! un `ClientWelcome` qui perdrait le build-id sur le chemin de production.

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Racine strictement possédée par un banc : même une panique libère ses
/// artefacts, dont le `target/` privé peut peser plus d'un Gio.
struct FixtureRoot(PathBuf);

impl FixtureRoot {
    fn new(label: &str) -> Self {
        // Les sockets Unix comptent chaque octet du chemin : le dossier
        // temporaire macOS peut déjà dépasser SUN_LEN avant bridget.sock.
        let root = PathBuf::from(format!("/tmp/bgbi-{label}-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).expect("créer la racine de fixture");
        Self(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
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
            "--offline",
            "--locked",
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
                .env("BRIDGET_HOME", home.join("state"))
                .env("BRIDGET_SOCKET", home.join("state/bridget.sock"))
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

        let socket = home.join("state/bridget.sock");
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
        .env("BRIDGET_HOME", home.join("state"))
        .env("BRIDGET_SOCKET", home.join("state/bridget.sock"))
        .env("HOSTNAME", BANC_HOST)
        .env("PATH", env::var("PATH").unwrap_or_default())
        .output()
        .expect("interroger le daemon via le CLI réel")
}

#[test]
fn daemon_et_cli_reels_transmettent_et_comparent_le_build_id() {
    let root = FixtureRoot::new("seam");
    let home = root.path().join("home");
    let target = root.path().join("target");
    fs::create_dir_all(&home).expect("home isolé");

    let old_build_id = "daemon-build-test";
    let new_build_id = "client-build-avance";
    let compiled = build_cli(&repository_root(), &target, old_build_id);
    let daemon_binary = root.path().join("bridget-daemon");
    copy_binary(&compiled, &daemon_binary);

    let daemon = DaemonGuard::start(&daemon_binary, &home);
    let same_build = status(&daemon_binary, &home);
    assert!(same_build.status.success());
    let rendu = String::from_utf8_lossy(&same_build.stdout).to_string();
    assert!(rendu.contains("Build-id daemon: daemon-build-test"));
    let stderr = String::from_utf8_lossy(&same_build.stderr);
    // L'identité égale ne doit produire aucun avertissement de version. En
    // revanche l'hôte du banc peut légitimement publier son avertissement
    // global de capacité disque : il ne dépend ni du daemon isolé ni du
    // build-id que cette couture vérifie.
    assert!(
        stderr.lines().all(|line| line.starts_with("WARN disque:")),
        "égalité avec un avertissement étranger au disque: {stderr:?}"
    );

    // La sonde d'identité doit rendre les DEUX valeurs que le daemon atteste,
    // et ce sont les VALEURS qu'on éprouve — pas la présence des champs.
    // La machine est imposée au banc, la base est celle du HOME isolé : les deux
    // sont donc connues d'avance et nommées ici, sans être recalculées par le
    // code de production.
    assert!(
        rendu.contains(&format!("Machine du daemon: {BANC_HOST}")),
        "la machine attestée doit être celle du banc: {rendu}"
    );
    let base_attendue = home.join("state/bridget.db");
    assert!(
        rendu.contains(&format!(
            "Base de données du daemon: {}",
            base_attendue.display()
        )),
        "la base attestée doit être celle du daemon: {rendu}"
    );
    // Contrôle de sens : sans attestation, ces deux lignes diraient « machine
    // non attestée ». Leur absence prouve que la sonde a bien abouti.
    assert!(
        !rendu.contains("machine non attestée"),
        "la sonde d'identité n'a pas abouti: {rendu}"
    );

    // Une reconstruction ultérieure porte une identité différente, comme après
    // l'avancée de HEAD déjà vérifiée par build_identity.rs. Cette couture doit
    // alors alerter le CLI sans toucher au daemon encore en cours.
    let rebuilt = build_cli(&repository_root(), &target, new_build_id);
    let client_binary = root.path().join("bridget-client-avance");
    copy_binary(&rebuilt, &client_binary);
    let different_build = status(&client_binary, &home);
    assert!(different_build.status.success());
    assert!(
        String::from_utf8_lossy(&different_build.stdout)
            .contains("Build-id daemon: daemon-build-test")
    );
    // Le daemon de ce banc est local mais indépendant du service historique.
    // L'oracle fixe le texte sans rappeler son producteur.
    let ici = BANC_HOST;
    let remediation = "relancer manuellement le daemon Bridget communication du namespace BRIDGET_HOME/BRIDGET_SOCKET vérifié — ne pas relancer le service historique";
    let expected = format!(
        "daemon périmé sur {ici} (daemon-build-test) — client client-build-avance sur {ici} : {remediation}\n"
    );
    let stderr = String::from_utf8_lossy(&different_build.stderr);
    let disk_suffix = stderr
        .strip_prefix(&expected)
        .expect("le verdict de build-id doit rester le premier avertissement");
    assert!(
        disk_suffix.is_empty()
            || disk_suffix
                .lines()
                .all(|line| line.starts_with("WARN disque:")),
        "l'avertissement supplémentaire doit être limité au disque: {stderr:?}"
    );

    daemon.stop();
}

#[test]
fn racines_de_fixture_sont_uniques_au_dela_du_pid() {
    let first = FixtureRoot::new("unicite");
    let second = FixtureRoot::new("unicite");
    assert_ne!(
        first.path(),
        second.path(),
        "deux fixtures ne peuvent pas partager une racine même sous le même PID"
    );
}

#[test]
fn racine_de_fixture_est_liberee_en_sortie_normale() {
    let path = {
        let fixture = FixtureRoot::new("normal");
        fixture.path().to_path_buf()
    };
    assert!(
        !path.exists(),
        "la racine doit disparaître à la sortie normale: {}",
        path.display()
    );
}

#[test]
fn racine_de_fixture_est_liberee_apres_panique() {
    let mut observed = None;
    let result = catch_unwind(AssertUnwindSafe(|| {
        let fixture = FixtureRoot::new("panique");
        observed = Some(fixture.path().to_path_buf());
        panic!("panique volontaire du témoin de nettoyage");
    }));
    assert!(result.is_err(), "la panique de contrôle doit être capturée");
    let path = observed.expect("racine créée avant la panique");
    assert!(
        !path.exists(),
        "la racine doit disparaître après panique: {}",
        path.display()
    );
}
