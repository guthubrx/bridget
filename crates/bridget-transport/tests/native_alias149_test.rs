//! Session 149 - alias de socket du serveur Codex interactif.
//!
//! Le wrapper donne au transport un chemin dans un répertoire privé créé sous
//! `/tmp`, hors de `BRIDGET_HOME`. Le transport reste le garde : il refuse
//! tout chemin non privé, tout symlink et tout alias déjà occupé AVANT de
//! lancer le fournisseur, puis il retire l'alias à l'arrêt confirmé.
//!
//! Faux app-server Codex en Python (WebSocket sur socket Unix, aucun modèle).
//! Les oracles portent sur le système de fichiers et sur le processus.
//! Le dossier privé que le wrapper possède après l'arrêt est couvert par la
//! recette réelle `codex_interactive_090.py` (`assert_aliases_cleaned`).

use bridget_transport::codex_app_server::{CodexAppServerOptions, CodexAppServerTransport};
use bridget_transport::managed_session::ManagedSession;
use std::fs;
use std::os::unix::fs::{FileTypeExt, PermissionsExt, symlink};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

const SCRIPT: &str = r#"#!/usr/bin/python3
import base64
import hashlib
import json
import os
import signal
import socket
import struct
import sys

args = sys.argv[1:]
chemin = None
for i, a in enumerate(args):
    if a == "--listen":
        chemin = args[i + 1][len("unix://"):]
with open(os.environ["B149_PID"], "w") as f:
    f.write(str(os.getpid()))
serveur = socket.socket(socket.AF_UNIX)
serveur.bind(chemin)
serveur.listen(1)

def arret(numero, cadre):
    os._exit(0)

signal.signal(signal.SIGTERM, arret)
conn, _ = serveur.accept()
entete = b""
while b"\r\n\r\n" not in entete:
    morceau = conn.recv(4096)
    if not morceau:
        sys.exit(1)
    entete += morceau
cle = [l.split(b":", 1)[1].strip() for l in entete.split(b"\r\n") if l.lower().startswith(b"sec-websocket-key")][0]
accept = base64.b64encode(hashlib.sha1(cle + b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11").digest())
conn.sendall(b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: " + accept + b"\r\n\r\n")

def lire(n):
    donnees = b""
    while len(donnees) < n:
        morceau = conn.recv(n - len(donnees))
        if not morceau:
            os._exit(0)
        donnees += morceau
    return donnees

def trame():
    h = lire(2)
    opcode = h[0] & 0x0F
    longueur = h[1] & 0x7F
    if longueur == 126:
        longueur = struct.unpack(">H", lire(2))[0]
    elif longueur == 127:
        longueur = struct.unpack(">Q", lire(8))[0]
    masque = lire(4) if h[1] & 0x80 else b"\0\0\0\0"
    charge = bytearray(lire(longueur))
    for i in range(len(charge)):
        charge[i] ^= masque[i % 4]
    return opcode, bytes(charge)

def envoyer(valeur):
    charge = json.dumps(valeur).encode()
    if len(charge) < 126:
        entete = struct.pack(">BB", 0x81, len(charge))
    else:
        entete = struct.pack(">BBH", 0x81, 126, len(charge))
    conn.sendall(entete + charge)

while True:
    opcode, charge = trame()
    if opcode == 8:
        os._exit(0)
    if opcode != 1:
        continue
    cadre = json.loads(charge)
    methode = cadre.get("method")
    ident = cadre.get("id")
    if ident is None:
        continue
    if methode == "initialize":
        envoyer({"id": ident, "result": {"userAgent": "fake", "codexHome": "/tmp", "platformFamily": "unix", "platformOs": "macos"}})
    elif methode == "thread/start":
        envoyer({"id": ident, "result": {"thread": {"id": "thread-alias", "historyMode": "legacy"}, "model": "gpt-5.6-terra", "reasoningEffort": "high"}})
    else:
        envoyer({"id": ident, "result": {}})
"#;

struct Racine(PathBuf);

impl Drop for Racine {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Banc {
    racine: Racine,
    script: PathBuf,
    pid_file: PathBuf,
}

fn banc(label: &str) -> Banc {
    let chemin = PathBuf::from(format!(
        "/tmp/b149al-{label}-{}-{}",
        std::process::id(),
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    ));
    fs::create_dir_all(&chemin).unwrap();
    fs::set_permissions(&chemin, fs::Permissions::from_mode(0o700)).unwrap();
    let script = chemin.join("faux-codex-alias-149.py");
    fs::write(&script, SCRIPT).unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();
    Banc {
        pid_file: chemin.join("fournisseur.pid"),
        script,
        racine: Racine(chemin),
    }
}

/// Répertoire privé de l'alias, comme celui que crée le wrapper.
fn dossier_alias(banc: &Banc, nom: &str, mode: u32) -> PathBuf {
    let dossier = banc.racine.0.join(nom);
    fs::create_dir(&dossier).unwrap();
    fs::set_permissions(&dossier, fs::Permissions::from_mode(mode)).unwrap();
    dossier
}

fn lancer(banc: &Banc, socket: &Path) -> Result<CodexAppServerTransport, String> {
    let options = CodexAppServerOptions {
        command: "/usr/bin/python3".to_string(),
        args: vec![banc.script.to_string_lossy().into_owned()],
        queue_capacity: 4,
        notify_timeout_secs: 3,
        model: Some("gpt-5.6-terra".to_string()),
        permissions: "interactive".to_string(),
        provider_observation: None,
        thread_bootstrap: Default::default(),
        dynamic_tool_handler: None,
    };
    let environment = vec![(
        "B149_PID".to_string(),
        banc.pid_file.to_string_lossy().into_owned(),
    )];
    CodexAppServerTransport::spawn_interactive(options, &environment, socket)
        .map_err(|error| error.to_string())
}

fn attendre(description: &str, limite: Duration, mut condition: impl FnMut() -> bool) {
    let fin = Instant::now() + limite;
    while !condition() {
        assert!(Instant::now() < fin, "délai dépassé : {description}");
        thread::sleep(Duration::from_millis(20));
    }
}

fn vivant(pid: i32) -> bool {
    unsafe { libc::kill(pid, 0) == 0 }
}

#[test]
fn native149_alias_dans_un_dossier_prive_hors_home_demarre_et_l_arret_retire_le_socket() {
    let banc = banc("accepte");
    let dossier = dossier_alias(&banc, "bridget-codex-test", 0o700);
    let socket = dossier.join("s.sock");

    let transport = lancer(&banc, &socket).expect("alias privé accepté");

    assert_eq!(transport.thread_id(), "thread-alias");
    let type_du_chemin = fs::symlink_metadata(&socket).unwrap().file_type();
    assert!(
        type_du_chemin.is_socket(),
        "le fournisseur écoute sur l'alias"
    );
    let pid: i32 = fs::read_to_string(&banc.pid_file)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert!(vivant(pid));

    transport.stop();

    attendre("fournisseur arrêté", Duration::from_secs(10), || {
        !vivant(pid)
    });
    assert!(
        fs::symlink_metadata(&socket).is_err(),
        "l'alias est retiré après l'arrêt confirmé"
    );
    assert!(
        dossier.is_dir(),
        "le dossier appartient au wrapper : le transport ne le supprime pas"
    );
}

/// Chaque cas prépare un alias dangereux. Le transport doit refuser avant
/// de lancer le fournisseur, et laisser l'objet existant intact.
#[test]
fn native149_alias_dangereux_est_refuse_avant_tout_lancement_et_laisse_l_existant_intact() {
    // 1. Dossier lisible par d'autres (0755) : espace de noms partagé.
    {
        let banc = banc("ouvert");
        let dossier = dossier_alias(&banc, "partage", 0o755);
        let erreur = lancer(&banc, &dossier.join("s.sock")).err().expect("refus");
        assert!(erreur.contains("non privé"), "{erreur}");
        assert!(!banc.pid_file.exists(), "fournisseur jamais lancé");
        assert!(!dossier.join("s.sock").exists());
    }
    // 2. Dossier parent qui est un lien symbolique vers un dossier privé.
    {
        let banc = banc("parent-lien");
        let reel = dossier_alias(&banc, "reel", 0o700);
        let lien = banc.racine.0.join("lien");
        symlink(&reel, &lien).unwrap();
        let erreur = lancer(&banc, &lien.join("s.sock")).err().expect("refus");
        assert!(erreur.contains("non privé"), "{erreur}");
        assert!(!banc.pid_file.exists(), "fournisseur jamais lancé");
        assert!(!reel.join("s.sock").exists(), "rien créé via le lien");
    }
    // 3. L'alias est un symlink vers un fichier étranger.
    {
        let banc = banc("alias-lien");
        let dossier = dossier_alias(&banc, "prive", 0o700);
        let etranger = banc.racine.0.join("etranger.txt");
        fs::write(&etranger, b"ETRANGER").unwrap();
        symlink(&etranger, dossier.join("s.sock")).unwrap();
        let erreur = lancer(&banc, &dossier.join("s.sock")).err().expect("refus");
        assert!(erreur.contains("déjà occupé"), "{erreur}");
        assert!(!banc.pid_file.exists(), "fournisseur jamais lancé");
        assert_eq!(fs::read(&etranger).unwrap(), b"ETRANGER", "cible intacte");
        assert!(
            fs::symlink_metadata(dossier.join("s.sock"))
                .unwrap()
                .file_type()
                .is_symlink(),
            "le lien existant n'est ni suivi ni retiré"
        );
    }
    // 4. L'alias est un fichier ordinaire.
    {
        let banc = banc("alias-fichier");
        let dossier = dossier_alias(&banc, "prive", 0o700);
        fs::write(dossier.join("s.sock"), b"EXISTANT").unwrap();
        let erreur = lancer(&banc, &dossier.join("s.sock")).err().expect("refus");
        assert!(erreur.contains("déjà occupé"), "{erreur}");
        assert!(!banc.pid_file.exists());
        assert_eq!(fs::read(dossier.join("s.sock")).unwrap(), b"EXISTANT");
    }
    // 5. Un ancien alias encore vivant (socket en écoute) reste fermé.
    {
        let banc = banc("alias-vivant");
        let dossier = dossier_alias(&banc, "prive", 0o700);
        let ancien = UnixListener::bind(dossier.join("s.sock")).unwrap();
        let erreur = lancer(&banc, &dossier.join("s.sock")).err().expect("refus");
        assert!(erreur.contains("déjà occupé"), "{erreur}");
        assert!(!banc.pid_file.exists());
        drop(ancien);
        assert!(
            fs::symlink_metadata(dossier.join("s.sock"))
                .unwrap()
                .file_type()
                .is_socket(),
            "l'ancien socket n'est pas supprimé par le refus"
        );
    }
    // 6. Chemin trop long pour sockaddr_un : refus avant tout lancement.
    {
        let banc = banc("trop-long");
        let dossier = dossier_alias(&banc, "prive", 0o700);
        let long = dossier.join("s".repeat(200));
        let erreur = lancer(&banc, &long).err().expect("refus");
        assert!(erreur.contains("trop long"), "{erreur}");
        assert!(!banc.pid_file.exists());
    }
}
