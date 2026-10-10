//! Session 149 - O4 côté pilote Codex : `stop_native_mission`.
//!
//! Faux app-server Codex (script Python en stdio, aucun modèle). Il garde le
//! tour ouvert, note chaque trame reçue dans une trace, répond à
//! `turn/interrupt`, puis note la fin de son entrée standard. Les oracles
//! portent sur ce que le fournisseur reçoit réellement et sur son processus.
//!
//! Limite : le mode interactif (socket WebSocket, TUI) n'est pas simulé ici.
//! Il partage le même `stop_native_mission` ; son drainage gracieux reste couvert
//! par la recette réelle, pas par ce fichier.

use bridget_core::BridgetMessage;
use bridget_transport::codex_app_server::{CodexAppServerOptions, CodexAppServerTransport};
use bridget_transport::managed_session::ManagedSession;
use bridget_transport::transport::Transport;
use serde_json::Value;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

const SCRIPT: &str = r#"#!/usr/bin/python3
import json
import os
import signal
import sys
import threading
import time

trace = os.environ["B149_TRACE"]
with open(os.environ["B149_PID"], "w") as f:
    f.write(str(os.getpid()))

def note(value):
    with open(trace, "a") as f:
        f.write(json.dumps(value) + "\n")

verrou = threading.Lock()

def send(value):
    with verrou:
        sys.stdout.write(json.dumps(value) + "\n")
        sys.stdout.flush()

def arret(numero, cadre):
    note({"arret": "sigterm"})
    os._exit(0)

signal.signal(signal.SIGTERM, arret)
turns = 0
for line in sys.stdin:
    frame = json.loads(line)
    note(frame)
    method = frame.get("method")
    ident = frame.get("id")
    if method == "initialize":
        send({"id": ident, "result": {"userAgent": "fake", "codexHome": "/tmp", "platformFamily": "unix", "platformOs": "macos"}})
    elif method == "thread/start":
        send({"id": ident, "result": {"thread": {"id": "thread-native"}, "model": "gpt-5.6-terra", "reasoningEffort": "high"}})
    elif method == "account/rateLimits/read":
        send({"id": ident, "result": {"rateLimits": {"primary": {"usedPercent": 1, "windowDurationMins": 300, "resetsAt": 1787572200}, "rateLimitReachedType": None}}})
    elif method == "turn/start":
        turns += 1
        send({"id": ident, "result": {"turn": {"id": "turn-%d" % turns}}})
    elif method == "turn/interrupt":
        # Le terminal du tour précède la réponse RPC : un worker dont la file
        # resterait ouverte démarrerait le message suivant dans cet intervalle.
        send({"method": "turn/completed", "params": {"threadId": "thread-native", "turn": {"id": frame["params"]["turnId"], "status": "interrupted", "items": []}}})
        # La réponse part dans un fil : la boucle d'entrée continue de noter
        # ce que le pilote écrit pendant l'intervalle.
        threading.Timer(float(os.environ.get("B149_INTERRUPT_DELAY", "0")), send, [{"id": ident, "result": {}}]).start()
note({"arret": "fin_entree"})
"#;

struct Racine(PathBuf);

impl Drop for Racine {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Fournisseur {
    racine: Racine,
    trace: PathBuf,
    pid_file: PathBuf,
}

fn racine(label: &str) -> Fournisseur {
    let chemin = PathBuf::from(format!(
        "/tmp/b149ns-{label}-{}-{}",
        std::process::id(),
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    ));
    fs::create_dir_all(&chemin).unwrap();
    fs::set_permissions(&chemin, fs::Permissions::from_mode(0o700)).unwrap();
    let script = chemin.join("faux-codex-native-149.py");
    fs::write(&script, SCRIPT).unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();
    Fournisseur {
        trace: chemin.join("trace.jsonl"),
        pid_file: chemin.join("fournisseur.pid"),
        racine: Racine(chemin),
    }
}

fn demarrer(f: &Fournisseur) -> CodexAppServerTransport {
    let options = CodexAppServerOptions {
        command: "/usr/bin/python3".to_string(),
        args: vec![
            f.racine
                .0
                .join("faux-codex-native-149.py")
                .to_string_lossy()
                .into_owned(),
        ],
        queue_capacity: 4,
        notify_timeout_secs: 3,
        model: Some("gpt-5.6-terra".to_string()),
        permissions: "allow".to_string(),
        provider_observation: None,
        thread_bootstrap: Default::default(),
        dynamic_tool_handler: None,
    };
    let environment = vec![
        ("B149_INTERRUPT_DELAY".to_string(), "0.6".to_string()),
        (
            "B149_TRACE".to_string(),
            f.trace.to_string_lossy().into_owned(),
        ),
        (
            "B149_PID".to_string(),
            f.pid_file.to_string_lossy().into_owned(),
        ),
    ];
    CodexAppServerTransport::spawn_with_environment(options, &environment, false)
        .expect("faux app-server lancé")
}

fn mission(id: &str) -> BridgetMessage {
    let mut message = BridgetMessage::new("bridget", "codex-native", format!("mission {id}"));
    message.id = id.to_string();
    message.reply = true;
    message
}

fn trames(trace: &Path) -> Vec<Value> {
    fs::read_to_string(trace)
        .unwrap_or_default()
        .lines()
        .filter_map(|ligne| serde_json::from_str(ligne).ok())
        .collect()
}

fn methode(trace: &Path, nom: &str) -> Vec<Value> {
    trames(trace)
        .into_iter()
        .filter(|trame| trame["method"] == nom)
        .collect()
}

fn attendre(description: &str, limite: Duration, mut condition: impl FnMut() -> bool) {
    let fin = Instant::now() + limite;
    while !condition() {
        assert!(Instant::now() < fin, "délai dépassé : {description}");
        thread::sleep(Duration::from_millis(20));
    }
}

fn pid(f: &Fournisseur) -> i32 {
    fs::read_to_string(&f.pid_file)
        .unwrap()
        .trim()
        .parse()
        .unwrap()
}

fn vivant(pid: i32) -> bool {
    unsafe { libc::kill(pid, 0) == 0 }
}

/// Mission active : tour ouvert côté fournisseur et identifiant de tour connu du pilote.
fn mission_active(f: &Fournisseur, transport: &mut CodexAppServerTransport, id: &str) {
    transport
        .deliver(&mission(id))
        .expect("remise de la mission");
    attendre(
        "turn/start reçu par le fournisseur",
        Duration::from_secs(10),
        || !methode(&f.trace, "turn/start").is_empty(),
    );
    attendre("pilote occupé", Duration::from_secs(5), || {
        transport.is_busy()
    });
    // La réponse `turn/start` publie l'identifiant du tour avant de rendre la main.
    thread::sleep(Duration::from_millis(600));
}

/// Rang de la première trame de la méthode `nom`, ou de la première marque d'arrêt si `nom == "arret"`.
fn ordre(f: &Fournisseur, nom: &str) -> Option<usize> {
    trames(&f.trace)
        .iter()
        .position(|trame| trame["method"] == nom || (nom == "arret" && trame["arret"].is_string()))
}

/// Tour exact interrompu AVANT la fermeture du canal, puis fournisseur arrêté.
#[test]
fn native149_stop_native_interrompt_le_tour_exact_puis_arrete_le_fournisseur() {
    let f = racine("exact");
    let mut transport = demarrer(&f);
    let fournisseur = pid(&f);
    mission_active(&f, &mut transport, "mission-a");

    transport.stop_native_mission("mission-a");

    let interruptions = methode(&f.trace, "turn/interrupt");
    assert_eq!(
        interruptions.len(),
        1,
        "une seule interruption: {interruptions:?}"
    );
    assert_eq!(interruptions[0]["params"]["threadId"], "thread-native");
    assert_eq!(
        interruptions[0]["params"]["turnId"], "turn-1",
        "le tour actif de la mission"
    );
    attendre("fournisseur arrêté", Duration::from_secs(10), || {
        !vivant(fournisseur)
    });
    attendre(
        "arrêt du fournisseur noté",
        Duration::from_secs(5),
        || ordre(&f, "arret").is_some(),
    );
    assert!(
        ordre(&f, "turn/interrupt").unwrap() < ordre(&f, "arret").unwrap(),
        "l'interruption précède la fermeture du canal et le signal d'arrêt"
    );
    assert!(!transport.is_alive(), "le pilote n'accepte plus de travail");
}

/// Idempotence : répéter l'arrêt, ou l'arrêt ordinaire ensuite, ne renvoie rien au fournisseur.
#[test]
fn native149_stop_native_est_idempotent() {
    let f = racine("idempotent");
    let mut transport = demarrer(&f);
    let fournisseur = pid(&f);
    mission_active(&f, &mut transport, "mission-i");

    transport.stop_native_mission("mission-i");
    let apres_premier = trames(&f.trace).len();
    let debut = Instant::now();
    transport.stop_native_mission("mission-i");
    transport.stop_native_mission("mission-i");
    transport.stop();

    assert!(
        debut.elapsed() < Duration::from_secs(5),
        "les rappels ne bloquent pas"
    );
    attendre("fournisseur arrêté", Duration::from_secs(10), || {
        !vivant(fournisseur)
    });
    let apres = trames(&f.trace);
    assert_eq!(
        apres.len(),
        apres_premier,
        "aucune trame de plus après le premier arrêt"
    );
    assert_eq!(
        methode(&f.trace, "turn/interrupt").len(),
        1,
        "une seule interruption au total"
    );
}

/// Drainage : la file n'avance pas après l'arrêt, même quand le fournisseur
/// termine le tour interrompu, et plus rien n'est admis.
#[test]
fn native149_stop_native_ferme_l_admission_et_ne_demarre_aucun_message_en_file() {
    let f = racine("drainage");
    let mut transport = demarrer(&f);
    let fournisseur = pid(&f);
    mission_active(&f, &mut transport, "mission-d");
    transport
        .deliver(&mission("suivante"))
        .expect("message mis en file derrière la mission");

    transport.stop_native_mission("mission-d");

    attendre("fournisseur arrêté", Duration::from_secs(10), || {
        !vivant(fournisseur)
    });
    thread::sleep(Duration::from_millis(500));
    assert_eq!(
        methode(&f.trace, "turn/start").len(),
        1,
        "le message en file n'est jamais démarré"
    );
    assert!(
        transport.deliver(&mission("tardive")).is_err(),
        "plus d'admission après l'arrêt natif"
    );
}

/// Un identifiant de mission étranger n'interrompt pas le tour d'une autre mission.
#[test]
fn native149_stop_native_d_une_autre_mission_n_interrompt_pas_le_tour_actif() {
    let f = racine("etranger");
    let mut transport = demarrer(&f);
    let fournisseur = pid(&f);
    mission_active(&f, &mut transport, "mission-e");

    transport.stop_native_mission("mission-etrangere");

    attendre("fournisseur arrêté", Duration::from_secs(10), || {
        !vivant(fournisseur)
    });
    assert!(
        methode(&f.trace, "turn/interrupt").is_empty(),
        "aucune interruption du tour d'une autre mission"
    );
}

/// Témoin ordinaire : `stop()` seul garde sa sémantique (pas d'interruption en mode stdio).
#[test]
fn native149_temoin_stop_ordinaire_n_envoie_aucune_interruption() {
    let f = racine("ordinaire");
    let mut transport = demarrer(&f);
    let fournisseur = pid(&f);
    mission_active(&f, &mut transport, "mission-o");

    transport.stop();

    attendre("fournisseur arrêté", Duration::from_secs(10), || {
        !vivant(fournisseur)
    });
    assert!(
        methode(&f.trace, "turn/interrupt").is_empty(),
        "arrêt ordinaire inchangé"
    );
    assert_eq!(methode(&f.trace, "turn/start").len(), 1);
}
