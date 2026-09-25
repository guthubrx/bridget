//! Le daemon **réel** doit mourir quand on le lui demande.
//!
//! Pourquoi un banc d'intégration et pas un test unitaire : `daemon.rs` rend
//! `install_daemon_signal_handlers` no-op sous `cfg(test)`, précisément pour
//! qu'un binaire de test n'hérite pas des dispositions du service. Le chemin
//! d'arrêt du service est donc **inatteignable** depuis un test unitaire du
//! même crate. Seul un enfant compilé sans `cfg(test)` l'emprunte — d'où le
//! lancement du vrai binaire ici.
//!
//! Défaut d'origine : SIGTERM était reçu, le drapeau lu, l'arrêt propre exécuté
//! en entier — puis le processus bloquait à la dernière ligne dans un `join()`
//! sans échéance, et survivait indéfiniment. Mesuré sur un daemon nu, sans
//! aucune connexion : toujours vivant six secondes après `kill -TERM`.

use std::fs;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Marge très large au-dessus du délai nominal mesuré (une centaine de
/// millisecondes) : on éprouve que l'arrêt ABOUTIT, pas qu'il est rapide.
const BUDGET_ARRET: Duration = Duration::from_secs(15);

/// Borne de l'arrêt PRÉCOCE, à distinguer de l'arrêt par expiration.
///
/// Le garde du superviseur abandonne son join au bout de trois secondes : un
/// daemon dont les fils de fond seraient redevenus insensibles à l'ordre
/// d'arrêt finirait donc quand même, mais SEULEMENT à cette échéance. Rester
/// sous cette borne prouve que l'arrêt vient du chemin nominal — les fils
/// lâchent l'état, le canal se ferme, le join réussit — et non du filet.
/// Mesuré à une centaine de millisecondes sur le chemin sain ; la borne laisse
/// un ordre de grandeur de marge tout en restant nettement sous les trois
/// secondes du filet.
const BORNE_ARRET_PRECOCE: Duration = Duration::from_secs(2);

fn racine_temporaire(etiquette: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("horloge système")
        .as_nanos();
    // Les sockets Unix comptent chaque octet du chemin : rester court.
    PathBuf::from(format!(
        "/tmp/bgsd-{etiquette}-{}-{nanos}",
        std::process::id()
    ))
}

fn demarrer_daemon(home: &Path) -> Child {
    bridget_daemon::environment::Namespace::resolve(
        Some(home.join("state")),
        None,
        Some(home.to_path_buf()),
    )
    .unwrap()
    .prepare()
    .unwrap();
    let mut enfant = Command::new(env!("CARGO_BIN_EXE_bridget"))
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
    let socket = home.join("state/bridget.sock");
    let echeance = Instant::now() + Duration::from_secs(10);
    while UnixStream::connect(&socket).is_err() {
        if let Some(status) = enfant.try_wait().unwrap() {
            panic!("daemon arrêté avant disponibilité : {status}");
        }
        if Instant::now() >= echeance {
            unsafe {
                libc::kill(enfant.id() as i32, libc::SIGTERM);
            }
            let _ = attendre_la_fin(&mut enfant, BUDGET_ARRET);
            panic!("daemon réel non joignable : {socket:?}");
        }
        thread::sleep(Duration::from_millis(10));
    }
    enfant
}

/// Attend la fin sans jamais bloquer sans borne : rend le délai observé, ou
/// `None` si le budget est épuisé. Un oracle qui appellerait `wait()` se
/// pendrait sur le défaut au lieu de le signaler — c'est exactement ce qui
/// arrivait au banc `attach_journal_attestation_test`.
fn attendre_la_fin(enfant: &mut Child, budget: Duration) -> Option<Duration> {
    let debut = Instant::now();
    while debut.elapsed() < budget {
        match enfant.try_wait() {
            Ok(Some(_)) => return Some(debut.elapsed()),
            Ok(None) => thread::sleep(Duration::from_millis(20)),
            Err(_) => return None,
        }
    }
    None
}

/// SIGTERM sur un daemon **nu** : aucune connexion, aucune charge, aucun agent.
///
/// Mutant qui tue ce test : rétablir `let _ = join.join();` après l'échéance
/// dans `ManagedSupervisorGuard::join_supervisor`, ou rendre les threads de fond
/// à nouveau immortels dans `daemon::run` → le processus survit au budget et
/// l'assertion meurt en affichant le budget dépassé, pas un booléen.
#[test]
fn un_daemon_nu_meurt_sur_sigterm_sans_signal_non_capturable() {
    let racine = racine_temporaire("nu");
    let home = racine.join("home");
    let mut daemon = demarrer_daemon(&home);

    let envoye = unsafe { libc::kill(daemon.id() as i32, libc::SIGTERM) };
    assert_eq!(envoye, 0, "SIGTERM doit être délivré au daemon réel");

    let delai = attendre_la_fin(&mut daemon, BUDGET_ARRET);

    // Nettoyer AVANT d'assertir : un échec ne doit pas ajouter un orphelin de
    // plus à ceux que ce test existe pour empêcher.
    if delai.is_none() {
        let _ = daemon.kill();
        let _ = daemon.wait();
    }
    let _ = fs::remove_dir_all(&racine);

    let delai = delai.unwrap_or_else(|| {
        panic!(
            "le daemon a survécu à SIGTERM au-delà de {} s — il n'aurait cédé qu'à \
             un signal non capturable, c'est-à-dire à un orphelin de plus",
            BUDGET_ARRET.as_secs()
        )
    });
    // ARRÊT PRÉCOCE, et non arrêt par expiration du filet. Sans cette borne,
    // un daemon dont les fils de fond ignorent l'ordre d'arrêt passerait quand
    // même — il finirait à l'échéance de trois secondes du garde, et le banc ne
    // saurait pas distinguer le chemin nominal de son filet.
    assert!(
        delai < BORNE_ARRET_PRECOCE,
        "arrêt trop tardif : {} ms, soit au-delà de la borne d'arrêt précoce \
         de {} ms — le daemon n'est sorti que par le filet du superviseur, \
         pas par le chemin nominal",
        delai.as_millis(),
        BORNE_ARRET_PRECOCE.as_millis()
    );
}

/// Le fichier PID et la socket doivent avoir disparu : c'est la preuve que
/// l'arrêt PROPRE s'est déroulé, et pas seulement que le processus est mort.
///
/// Mutant qui tue ce test : sortir du processus avant le nettoyage (par exemple
/// un `_exit` posé dans le gestionnaire de signal) → les deux fichiers restent
/// et les assertions meurent en nommant celui qui subsiste.
#[test]
fn l_arret_reste_propre_et_ne_laisse_ni_socket_ni_fichier_pid() {
    let racine = racine_temporaire("propre");
    let home = racine.join("home");
    let mut daemon = demarrer_daemon(&home);
    let socket = home.join("state/bridget.sock");
    let fichier_pid = socket.with_extension("pid");

    // Contrôle positif : les deux existent AVANT l'arrêt. Sans lui, un test qui
    // vérifie leur absence passerait aussi si le daemon ne les créait jamais.
    assert!(socket.exists(), "socket absente avant l'arrêt: {socket:?}");
    assert!(
        fichier_pid.exists(),
        "fichier PID absent avant l'arrêt: {fichier_pid:?}"
    );

    assert_eq!(unsafe { libc::kill(daemon.id() as i32, libc::SIGTERM) }, 0);
    let delai = attendre_la_fin(&mut daemon, BUDGET_ARRET);
    if delai.is_none() {
        let _ = daemon.kill();
        let _ = daemon.wait();
    }

    let socket_restante = socket.exists();
    let pid_restant = fichier_pid.exists();
    let _ = fs::remove_dir_all(&racine);

    assert!(delai.is_some(), "le daemon n'a pas terminé dans le budget");
    assert!(!socket_restante, "socket laissée derrière: {socket:?}");
    assert!(!pid_restant, "fichier PID laissé derrière: {fichier_pid:?}");
}
