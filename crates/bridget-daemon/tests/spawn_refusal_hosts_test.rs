//! Le refus de lancement doit nommer LES DEUX machines, par la vraie couture.
//!
//! Un premier oracle injectait `SpawnHosts` directement dans `submit_spawn` et
//! prouvait donc uniquement que la fonction recopie ce qu'on lui donne. Il
//! CONTOURNAIT la couture : le CLI s'enregistrait avec `host: None`, le daemon
//! lisait `requested_from` dans `conn_hosts`, et n'y trouvait rien. Le refus
//! réel ne pouvait pas nommer le demandeur.
//!
//! Ce banc emprunte le chemin complet — vrai daemon, vrai CLI, deux `HOSTNAME`
//! distincts imposés — et asserte les VALEURS attendues, pas la présence des
//! champs.

use bridget_transport::protocol::{PresenceMode, decode, encode};
use bridget_transport::{DaemonToWrapper, SpawnRefusal, WrapperToDaemon};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Deux machines DIFFÉRENTES : c'est la seule configuration où l'oracle peut
/// distinguer « où l'on a cherché » de « qui a demandé ».
const HOTE_DAEMON: &str = "machine-executante";
const HOTE_CLIENT: &str = "machine-demandeuse";

fn racine_temporaire() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("horloge système")
        .as_nanos();
    PathBuf::from(format!("/tmp/bgsh-{}-{nanos}", std::process::id()))
}

/// Registre minimal : un type lançable dont la commande existe, pour que la
/// garde du `cwd` soit ATTEINTE. Sans lui le refus serait `CommandMissing` et
/// l'oracle passerait à côté de ce qu'il éprouve.
fn ecrire_registre(home: &Path) {
    bridget_daemon::environment::Namespace::resolve(
        Some(home.join("state")),
        None,
        Some(home.to_path_buf()),
    )
    .unwrap()
    .prepare()
    .unwrap();
    // La garde CwdGone est située après la posture. Précondition privée,
    // écrite avant le daemon ; aucune autorisation de la flotte n'est changée.
    let database = home.join("state/bridget.db");
    drop(bridget_daemon::store::Store::open(&database).unwrap());
    let connection = rusqlite::Connection::open(&database).unwrap();
    let initial = bridget_daemon::referent_control::read(&connection).unwrap();
    bridget_daemon::referent_control::set(
        &connection,
        bridget_daemon::referent_control::ControlMutation {
            command_id: "host-fixture-complete",
            expected_generation: initial.generation,
            paused: None,
            auto_objectives_cap: None,
            reason: None,
            actor: "test",
            now: 1,
            agent_posture: Some(bridget_transport::protocol::AgentPosture::Complete),
            auto_reassignment: None,
        },
    )
    .unwrap()
    .unwrap();
    let dossier = home.join("state");
    fs::create_dir_all(&dossier).expect("dossier de configuration");
    fs::write(
        dossier.join("agents.json"),
        r#"{"agents":{"fixture":{
            "command":"/bin/sh","args":[],"protocol":"acp",
            "forbidden_env":[],"pass_env":[],"permissions":"allow",
            "queue_capacity":1,"notify_timeout_secs":2,
            "capabilities":{"execution_paths":["acp"],"models":{}}
        }}}"#,
    )
    .expect("registre d'agents");
    // Le daemon refuse un registre plus permissif que 0600.
    fs::set_permissions(
        dossier.join("agents.json"),
        fs::Permissions::from_mode(0o600),
    )
    .expect("registre privé");
}

struct DaemonProcess(Child);

impl Drop for DaemonProcess {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            unsafe {
                libc::kill(self.0.id() as i32, libc::SIGTERM);
            }
            let deadline = Instant::now() + Duration::from_secs(5);
            while self.0.try_wait().ok().flatten().is_none() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

fn demarrer_daemon(home: &Path) -> DaemonProcess {
    let enfant = Command::new(env!("CARGO_BIN_EXE_bridget"))
        .arg("daemon")
        .env_clear()
        .env("HOME", home)
        .env("BRIDGET_HOME", home.join("state"))
        .env("BRIDGET_SOCKET", home.join("state/bridget.sock"))
        .env("HOSTNAME", HOTE_DAEMON)
        .env("PATH", "/usr/bin:/bin")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("daemon réel démarré");
    let enfant = DaemonProcess(enfant);
    let socket = home.join("state/bridget.sock");
    let echeance = Instant::now() + Duration::from_secs(10);
    while UnixStream::connect(&socket).is_err() {
        assert!(
            Instant::now() < echeance,
            "daemon réel non joignable: {socket:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
    enfant
}

fn arreter(mut daemon: DaemonProcess) {
    let _ = unsafe { libc::kill(daemon.0.id() as i32, libc::SIGTERM) };
    let echeance = Instant::now() + Duration::from_secs(15);
    while Instant::now() < echeance {
        if matches!(daemon.0.try_wait(), Ok(Some(_))) {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("daemon privé non arrêté après SIGTERM");
}

/// Se connecte, s'enregistre en se déclarant sur une AUTRE machine que le
/// daemon — exactement ce que fait désormais le CLI — puis demande un
/// lancement dont le `cwd` n'existe pas.
///
/// Pourquoi pas `bridget spawn` : le CLI vérifie le `cwd` sur SON PROPRE
/// système de fichiers avant d'émettre l'ordre, et rend « cwd absent ou non
/// répertoire » sans jamais joindre le daemon. Le refus du daemon n'est donc
/// atteignable, en fédération, que lorsque le répertoire existe chez le
/// demandeur et pas chez l'exécutant — situation qu'une machine unique ne peut
/// pas produire. Ce banc emprunte la voie protocolaire, qui est celle que le
/// CLI emprunte une fois sa vérification locale passée.
fn refus_de_lancement(socket: &Path, cwd: &Path) -> SpawnRefusal {
    let stream = UnixStream::connect(socket).expect("connexion");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("délai de lecture");
    let mut reader = BufReader::new(stream.try_clone().expect("clone"));
    let mut writer = BufWriter::new(stream);

    let register = WrapperToDaemon::Register {
        agent_type: "cli".to_string(),
        identity_version: 2,
        agent_id: "89000000-0000-4000-8000-000000000901".to_string(),
        // La valeur que le CLI transmet désormais. Avant le correctif : `None`.
        host: Some(HOTE_CLIENT.to_string()),
        transport: None,
        channel: bridget_transport::ChannelReport::Unknown,
        mode: Some(PresenceMode::Cli),
        location: None,
        os: None,
        instance_id: None,
        domain: None,
        turn_in_progress: false,
        journal_available: None,
    };
    writeln!(writer, "{}", encode(&register).expect("encodage")).expect("écriture");
    writer.flush().expect("flush");
    let mut ligne = String::new();
    reader.read_line(&mut ligne).expect("lecture");
    assert!(
        matches!(
            decode::<DaemonToWrapper>(ligne.trim_end()).expect("décodage"),
            DaemonToWrapper::Registered { .. }
        ),
        "enregistrement refusé: {ligne}"
    );

    let maintenant = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("horloge")
        .as_secs() as i64;
    let ordre = WrapperToDaemon::SpawnOrder {
        posture: None,
        agent_type: "fixture".to_string(),
        project: None,
        agent_id: Some("89000000-0000-4000-8000-000000000902".to_string()),
        cwd: cwd.display().to_string(),
        persistent: false,
        command_id: "temoin-hotes-1".to_string(),
        issued_at: maintenant,
        deadline_at: maintenant + 10,
        ownership: None,
    };
    writeln!(writer, "{}", encode(&ordre).expect("encodage")).expect("écriture");
    writer.flush().expect("flush");
    ligne.clear();
    reader.read_line(&mut ligne).expect("lecture");
    match decode::<DaemonToWrapper>(ligne.trim_end()).expect("décodage") {
        DaemonToWrapper::SpawnRejected { reason, .. } => reason,
        autre => panic!("refus de lancement attendu, obtenu: {autre:?}"),
    }
}

/// Mutant qui tue ce test : remettre `host: None` dans le `Register` du CLI
/// (`send_control_to_daemon_at`) rend `conn_hosts` vide — et ici, remettre le
/// `host: None` ci-dessus produit `requested_from` = machine non attestée,
/// l'assertion meurt en affichant la valeur reçue.
#[test]
fn le_refus_de_cwd_nomme_les_deux_machines_par_la_vraie_couture() {
    let racine = racine_temporaire();
    let home = racine.join("home");
    ecrire_registre(&home);
    let daemon = demarrer_daemon(&home);
    let socket = home.join("state/bridget.sock");

    let refus = refus_de_lancement(&socket, &racine.join("repertoire-qui-n-existe-pas"));

    arreter(daemon);
    let _ = fs::remove_dir_all(&racine);

    match refus {
        SpawnRefusal::CwdGone {
            searched_on,
            requested_from,
        } => {
            // Les DEUX valeurs, chacune à sa place. Un oracle qui vérifierait la
            // seule présence d'un hôte passerait avec la même des deux côtés.
            assert_eq!(
                searched_on, HOTE_DAEMON,
                "la machine où l'on a cherché est celle du daemon"
            );
            assert_eq!(
                requested_from, HOTE_CLIENT,
                "la machine demandeuse est celle attestée à l'enregistrement"
            );
            assert_ne!(searched_on, requested_from);
        }
        autre => panic!("refus CwdGone attendu, obtenu: {autre:?}"),
    }
}
