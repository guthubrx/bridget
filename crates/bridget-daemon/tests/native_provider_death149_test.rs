//! Session 149 - mort du fournisseur d'un enfant natif (recette R9.2).
//!
//! Vrai binaire `bridget`, vrai daemon, vrai `managed-wrapper`, faux fournisseur
//! ACP (script Python) : aucun modèle. Le daemon natif place
//! `BRIDGET_NATIVE_MISSION_BOOTSTRAP` dans l'environnement de l'enfant ; ici le
//! daemon de test le reçoit dans son propre environnement et le registre le
//! transmet par `pass_env`. La valeur est une instruction interne sans droit :
//! l'instance ne correspond à aucun bail, seule sa présence compte pour la
//! garde de relance.
//!
//! - Témoin : agent persistant ordinaire, fournisseur tué, relancé (2 démarrages).
//! - Natif : même scénario, aucune relance (1 démarrage), wrapper terminé.

use bridget_daemon::managed_process::{ManagedMarkerStore, group_exists};
use bridget_transport::protocol::{decode, encode};
use bridget_transport::{DaemonToWrapper, StopOutcome, WrapperToDaemon};
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const FROZEN_PATH: &str = "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin";
const TIMEOUT: Duration = Duration::from_secs(10);
const BOOTSTRAP_ENV: &str = "BRIDGET_NATIVE_MISSION_BOOTSTRAP";
/// Première relance ordinaire après 1 s de repli ; marge pour le démarrage.
const FENETRE_SANS_RELANCE: Duration = Duration::from_secs(5);
static SERIE: Mutex<()> = Mutex::new(());

fn maintenant() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}

fn agent_id_for(label: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    label.hash(&mut hasher);
    let value = hasher.finish();
    format!("{:08x}-0000-4000-8000-{:012x}", value as u32, value & 0x0000_0fff_ffff_ffff)
}

struct Racine(PathBuf);

impl Drop for Racine {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn racine(label: &str) -> Racine {
    let chemin = PathBuf::from(format!(
        "/tmp/b149pd-{label}-{}-{}",
        std::process::id(),
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    ));
    bridget_daemon::environment::Namespace::resolve(Some(chemin.join("state")), None, Some(chemin.clone()))
        .unwrap()
        .prepare()
        .unwrap();
    Racine(chemin)
}

/// Fournisseur ACP minimal ; chaque démarrage ajoute son PID au compteur.
fn ecrire_fixture(racine: &Path) -> PathBuf {
    let compteur = racine.join("demarrages.txt");
    let adapter = racine.join("faux-acp.py");
    fs::write(
        &adapter,
        format!(
            r#"#!/usr/bin/python3
import json
import os
import sys

with open("{}", "a") as f:
    f.write(str(os.getpid()) + "\n")

for line in sys.stdin:
    request = json.loads(line)
    method = request.get("method")
    if method == "initialize":
        print(json.dumps({{"jsonrpc":"2.0","id":request["id"],"result":{{"protocolVersion":1}}}}), flush=True)
    elif method == "session/new":
        print(json.dumps({{"jsonrpc":"2.0","id":request["id"],"result":{{"sessionId":"pd-session"}}}}), flush=True)
    elif method == "session/prompt":
        print(json.dumps({{"jsonrpc":"2.0","id":request["id"],"result":{{"stopReason":"end_turn"}}}}), flush=True)
"#,
            compteur.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&adapter, fs::Permissions::from_mode(0o700)).unwrap();
    let registre = serde_json::json!({"agents": {"faux": {
        "command": adapter,
        "protocol": "acp",
        "permissions": "allow",
        "queue_capacity": 8,
        "notify_timeout_secs": 6,
        "forbidden_env": ["OPENAI_API_KEY", "CODEX_API_KEY"],
        "pass_env": [BOOTSTRAP_ENV]
    }}});
    let chemin = racine.join("state/agents.json");
    fs::write(&chemin, serde_json::to_vec_pretty(&registre).unwrap()).unwrap();
    fs::set_permissions(&chemin, fs::Permissions::from_mode(0o600)).unwrap();
    // Précondition privée : ce fournisseur synthétique n'offre pas de mode découverte.
    let base = racine.join("state/bridget.db");
    drop(bridget_daemon::store::Store::open(&base).unwrap());
    let connexion = rusqlite::Connection::open(&base).unwrap();
    let initial = bridget_daemon::referent_control::read(&connexion).unwrap();
    bridget_daemon::referent_control::set(
        &connexion,
        bridget_daemon::referent_control::ControlMutation {
            command_id: "pd149-fixture-complete",
            expected_generation: initial.generation,
            paused: None,
            auto_objectives_cap: None,
            reason: None,
            actor: "test",
            now: maintenant(),
            agent_posture: Some(bridget_transport::protocol::AgentPosture::Complete),
            auto_reassignment: None,
        },
    )
    .unwrap()
    .unwrap();
    compteur
}

struct Daemon {
    enfant: Option<Child>,
    socket: PathBuf,
}

impl Daemon {
    fn demarrer(racine: &Path, natif: bool) -> Self {
        let mut daemon = Self { enfant: None, socket: racine.join("state/bridget.sock") };
        let mut commande = Command::new(env!("CARGO_BIN_EXE_bridget"));
        commande
            .arg("daemon")
            .env_clear()
            .env("HOME", racine)
            .env("BRIDGET_HOME", racine.join("state"))
            .env("BRIDGET_SOCKET", racine.join("state/bridget.sock"))
            .env("PATH", FROZEN_PATH)
            .env("USER", "pd149-test")
            .env("LANG", "C")
            .env("TMPDIR", "/tmp")
            .env("BRIDGET_PROVIDER_RELAUNCH_MAX", "5")
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if natif {
            let bootstrap = serde_json::json!({
                "instance_id": "instance-sans-bail-149",
                "mission_id": uuid::Uuid::new_v4().to_string(),
            });
            commande.env(BOOTSTRAP_ENV, bootstrap.to_string());
        }
        daemon.enfant = Some(commande.spawn().unwrap());
        let limite = Instant::now() + Duration::from_secs(5);
        while Instant::now() < limite && UnixStream::connect(&daemon.socket).is_err() {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(UnixStream::connect(&daemon.socket).is_ok(), "le daemon n'accepte pas les connexions");
        daemon
    }
}

impl Drop for Daemon {
    fn drop(&mut self) {
        let Some(enfant) = self.enfant.as_mut() else { return };
        let _ = unsafe { libc::kill(enfant.id() as i32, libc::SIGTERM) };
        let limite = Instant::now() + Duration::from_secs(5);
        while Instant::now() < limite {
            if matches!(enfant.try_wait(), Ok(Some(_))) {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let _ = enfant.kill();
        let _ = enfant.wait();
    }
}

struct Pair {
    lecteur: BufReader<UnixStream>,
    ecrivain: BufWriter<UnixStream>,
}

impl Pair {
    fn enregistrer(socket: &Path, nom: &str) -> Self {
        let flux = UnixStream::connect(socket).unwrap();
        flux.set_read_timeout(Some(TIMEOUT)).unwrap();
        let mut pair = Self {
            lecteur: BufReader::new(flux.try_clone().unwrap()),
            ecrivain: BufWriter::new(flux),
        };
        pair.envoyer(&WrapperToDaemon::Register {
            identity_version: 2,
            agent_type: "pd149-client".to_string(),
            agent_id: agent_id_for(nom),
            host: Some("fixture-host".to_string()),
            transport: Some("unix".to_string()),
            channel: None.into(),
            mode: Some(bridget_transport::protocol::PresenceMode::Acp),
            location: None,
            os: Some("fixture-os".to_string()),
            instance_id: Some(format!("instance-{nom}")),
            domain: None,
            turn_in_progress: false,
            journal_available: None,
        });
        assert!(matches!(pair.recevoir(), DaemonToWrapper::Registered { .. }));
        pair
    }

    fn envoyer(&mut self, message: &WrapperToDaemon) {
        writeln!(self.ecrivain, "{}", encode(message).unwrap()).unwrap();
        self.ecrivain.flush().unwrap();
    }

    fn recevoir(&mut self) -> DaemonToWrapper {
        let mut ligne = String::new();
        self.lecteur.read_line(&mut ligne).unwrap();
        assert!(!ligne.is_empty(), "EOF inattendu depuis le daemon");
        decode(ligne.trim_end()).unwrap()
    }
}

/// Joignable = connecté ou occupé (le tour de la carte de reprise le rend `busy`).
fn agent_connecte(socket: &Path, nom: &str) -> bool {
    let mut observateur = Pair::enregistrer(socket, "pd149-observateur");
    observateur.envoyer(&WrapperToDaemon::ListAgents);
    match observateur.recevoir() {
        DaemonToWrapper::AgentList { agents } => agents
            .iter()
            .any(|agent| agent.agent_id == agent_id_for(nom) && matches!(agent.state.as_str(), "connected" | "busy")),
        autre => panic!("annuaire inattendu: {autre:?}"),
    }
}

fn attendre(description: &str, limite: Duration, mut condition: impl FnMut() -> bool) {
    let fin = Instant::now() + limite;
    while !condition() {
        assert!(Instant::now() < fin, "délai dépassé : {description}");
        thread::sleep(Duration::from_millis(25));
    }
}

fn demarrages(compteur: &Path) -> Vec<u32> {
    fs::read_to_string(compteur)
        .unwrap_or_default()
        .lines()
        .filter_map(|ligne| ligne.trim().parse().ok())
        .collect()
}

/// Lance un agent persistant, tue son fournisseur (SIGTERM) et rend les PID
/// de démarrage observés après la fenêtre d'observation, avec la présence.
fn tuer_le_fournisseur(label: &str, natif: bool) -> (Vec<u32>, bool, u32) {
    let racine = racine(label);
    let compteur = ecrire_fixture(&racine.0);
    let daemon = Daemon::demarrer(&racine.0, natif);
    let nom = format!("pd149-{label}");
    let mut controle = Pair::enregistrer(&daemon.socket, "pd149-ordonnateur");
    let maintenant = maintenant();
    controle.envoyer(&WrapperToDaemon::SpawnOrder {
        posture: None,
        agent_type: "faux".to_string(),
        agent_id: Some(agent_id_for(&nom)),
        cwd: racine.0.to_string_lossy().into_owned(),
        persistent: true,
        command_id: format!("pd149-{label}-1"),
        issued_at: maintenant,
        deadline_at: maintenant + 10,
        project: None,
        ownership: None,
    });
    assert!(matches!(controle.recevoir(), DaemonToWrapper::SpawnAccepted { .. }));
    attendre("agent connecté avant la mort du fournisseur", Duration::from_secs(15), || {
        agent_connecte(&daemon.socket, &nom)
    });
    attendre("premier démarrage du fournisseur", Duration::from_secs(5), || demarrages(&compteur).len() == 1);
    let premier = demarrages(&compteur)[0];
    let pgid = ManagedMarkerStore::at_directory(racine.0.join("state/managed"))
        .load(&agent_id_for(&nom))
        .unwrap()
        .pgid;
    assert_eq!(unsafe { libc::kill(premier as i32, libc::SIGTERM) }, 0, "SIGTERM fournisseur {premier}");

    if natif {
        // Aucune relance pendant toute la fenêtre, puis le wrapper se termine.
        let fin = Instant::now() + FENETRE_SANS_RELANCE;
        while Instant::now() < fin {
            assert_eq!(demarrages(&compteur).len(), 1, "un fournisseur natif ne doit pas être relancé");
            thread::sleep(Duration::from_millis(100));
        }
        attendre("wrapper natif terminé après la mort du fournisseur", TIMEOUT, || !group_exists(pgid).unwrap_or(false));
    } else {
        attendre("relance du fournisseur ordinaire", Duration::from_secs(20), || demarrages(&compteur).len() >= 2);
        attendre("agent ordinaire de nouveau connecté", Duration::from_secs(15), || agent_connecte(&daemon.socket, &nom));
    }
    let relances = demarrages(&compteur);
    let connecte = agent_connecte(&daemon.socket, &nom);
    if !natif {
        controle.envoyer(&WrapperToDaemon::StopOrder {
            agent_id: agent_id_for(&nom),
            command_id: format!("pd149-{label}-stop"),
        });
        assert!(matches!(
            controle.recevoir(),
            DaemonToWrapper::StopResult { outcome: StopOutcome::Stopped | StopOutcome::StoppedForced { .. }, .. }
        ));
    }
    (relances, connecte, premier)
}

/// Témoin : sans bootstrap natif, le fournisseur persistant est relancé.
#[test]
fn native149_temoin_fournisseur_ordinaire_tue_est_relance() {
    let _serie = SERIE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (demarrages, connecte, premier) = tuer_le_fournisseur("ordinaire", false);
    assert!(demarrages.len() >= 2, "le témoin doit relancer : {demarrages:?}");
    assert_ne!(demarrages[1], premier, "nouveau processus fournisseur");
    assert!(connecte, "l'agent ordinaire est de nouveau joignable");
}

/// R9.2 côté wrapper : un enfant natif dont le fournisseur meurt n'est jamais
/// relancé, ni par le wrapper ni par le daemon ; il n'y a qu'un fournisseur.
#[test]
fn native149_fournisseur_natif_tue_n_est_jamais_relance() {
    let _serie = SERIE.lock().unwrap_or_else(|poison| poison.into_inner());
    let (demarrages, connecte, premier) = tuer_le_fournisseur("natif", true);
    assert_eq!(demarrages, vec![premier], "un seul démarrage de fournisseur pour la mission");
    assert!(!connecte, "l'enfant natif n'est plus joignable : échec explicite côté saga");
}
