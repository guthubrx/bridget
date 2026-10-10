//! Session 149 - O4 : arrêt du wrapper d'une mission native.
//!
//! Vrai binaire `bridget managed-wrapper`, faux daemon (un seul `Register`,
//! pas de reconnexion servie), faux fournisseur ACP lancé dans son PROPRE
//! groupe de processus (`setsid`). Ce dernier détail reproduit O4 : un
//! SIGTERM reçu par le seul wrapper laissait le fournisseur orphelin.
//!
//! - Natif + SIGTERM : sortie coopérative (pas tué par le signal), fournisseur
//!   arrêté, aucune reconnexion.
//! - Natif + EOF du daemon : fournisseur arrêté, aucune reconnexion.
//! - Témoin ordinaire + EOF : le wrapper se reconnecte (comportement inchangé).
//! - Témoin ordinaire + SIGTERM : disposition par défaut du signal (inchangée).
//!
//! Aucun modèle. Chaque PID arrêté par le test est un PID de fixture, vérifié
//! par sa ligne de commande, arrêté un à un par SIGTERM.

use bridget_core::BridgetMessage;
use bridget_daemon::managed_process::{
    ManagedIdentity, ManagedLaunch, ManagedMarkerStore, RunningManagedChild,
    spawn_managed_bootstrap_with_stderr,
};
use bridget_daemon::registry::AgentRegistry;
use bridget_transport::protocol::{decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixListener;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

const AGENT_ID: &str = "89000000-0000-4000-8000-000000000a49";
const BOOTSTRAP_ENV: &str = "BRIDGET_NATIVE_MISSION_BOOTSTRAP";
const INSTANCE: &str = "instance-native-stop-149";
static SERIE: Mutex<()> = Mutex::new(());

struct Racine(PathBuf);

impl Drop for Racine {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn racine(label: &str) -> Racine {
    let chemin = PathBuf::from(format!(
        "/tmp/b149ws-{label}-{}-{}",
        std::process::id(),
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    ));
    bridget_daemon::environment::Namespace::resolve(
        Some(chemin.join("state")),
        None,
        Some(chemin.clone()),
    )
    .unwrap()
    .prepare()
    .unwrap();
    Racine(chemin)
}

fn processus_vivant(pid: i32) -> bool {
    unsafe { libc::kill(pid, 0) == 0 }
}

fn attendre(description: &str, limite: Duration, mut condition: impl FnMut() -> bool) {
    let fin = Instant::now() + limite;
    while !condition() {
        assert!(Instant::now() < fin, "délai dépassé : {description}");
        thread::sleep(Duration::from_millis(25));
    }
}

/// Fournisseur ACP : groupe indépendant, PID noté, tour `session/prompt` jamais terminé.
fn ecrire_registre(racine: &Path) -> (String, PathBuf) {
    let pid_file = racine.join("fournisseur.pid");
    let adapter = racine.join("faux-acp-native-149.py");
    fs::write(
        &adapter,
        format!(
            r#"#!/usr/bin/python3
import json
import os
import sys
import time

os.setsid()
with open("{}", "w") as f:
    f.write(str(os.getpid()))

for line in sys.stdin:
    request = json.loads(line)
    method = request.get("method")
    if method == "initialize":
        print(json.dumps({{"jsonrpc":"2.0","id":request["id"],"result":{{"protocolVersion":1}}}}), flush=True)
    elif method == "session/new":
        print(json.dumps({{"jsonrpc":"2.0","id":request["id"],"result":{{"sessionId":"ns-session"}}}}), flush=True)
    elif method == "session/prompt":
        while True:
            time.sleep(1)
"#,
            pid_file.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&adapter, fs::Permissions::from_mode(0o700)).unwrap();
    let json = serde_json::json!({"agents": {"faux-natif": {
        "command": adapter,
        "protocol": "acp",
        "permissions": "allow",
        "queue_capacity": 4,
        "notify_timeout_secs": 600
    }}})
    .to_string();
    let chemin = racine.join("state/agents.json");
    fs::write(&chemin, &json).unwrap();
    fs::set_permissions(&chemin, fs::Permissions::from_mode(0o600)).unwrap();
    (json, pid_file)
}

#[derive(Clone, Copy, PartialEq)]
enum FinDaemon {
    /// Le faux daemon reste connecté, sans rien envoyer de plus.
    Silence,
    /// Le faux daemon ferme la connexion après la remise du tour (EOF).
    Ferme,
}

struct Scenario {
    _racine: Racine,
    journal_wrapper: PathBuf,
    wrapper: RunningManagedChild,
    pid_fournisseur: i32,
    /// Nombre de connexions `Register` reçues par le faux daemon.
    connexions: mpsc::Receiver<()>,
    // Garde le faux daemon vivant tant que le scénario existe.
    _daemon: thread::JoinHandle<()>,
    _liberer: mpsc::Sender<()>,
}

/// Lance le vrai wrapper géré sous un faux daemon. `natif` pose le bootstrap
/// de mission pour l'instance du wrapper.
fn lancer(label: &str, natif: bool, persistant: bool, fin: FinDaemon) -> Scenario {
    let racine = racine(label);
    let (registre_json, pid_file) = ecrire_registre(&racine.0);
    let socket = racine.0.join("state/bridget.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    let (connexions_tx, connexions) = mpsc::channel();
    let (liberer_tx, liberer_rx) = mpsc::channel::<()>();
    let (pret_tx, pret_rx) = mpsc::channel();
    let daemon = thread::spawn(move || {
        // Chaque connexion reçoit Registered ; seule la première reçoit le tour.
        let mut premiere = true;
        let mut flux_gardes = Vec::new();
        listener.set_nonblocking(true).unwrap();
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    stream.set_nonblocking(false).unwrap();
                    stream
                        .set_read_timeout(Some(Duration::from_secs(5)))
                        .unwrap();
                    stream
                        .set_write_timeout(Some(Duration::from_secs(5)))
                        .unwrap();
                    let mut reader = BufReader::new(stream.try_clone().unwrap());
                    let mut writer = BufWriter::new(stream.try_clone().unwrap());
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        continue;
                    }
                    if !matches!(
                        decode::<WrapperToDaemon>(line.trim_end()),
                        Ok(WrapperToDaemon::Register { .. })
                    ) {
                        continue;
                    }
                    let _ = connexions_tx.send(());
                    writeln!(
                        writer,
                        "{}",
                        encode(&DaemonToWrapper::Registered {
                            credential: None,
                            agent_id: AGENT_ID.to_string()
                        })
                        .unwrap()
                    )
                    .unwrap();
                    if premiere {
                        premiere = false;
                        writeln!(
                            writer,
                            "{}",
                            encode(&DaemonToWrapper::Deliver(BridgetMessage::new(
                                "humain",
                                AGENT_ID,
                                "tour natif 149"
                            )))
                            .unwrap()
                        )
                        .unwrap();
                        writer.flush().unwrap();
                        let _ = pret_tx.send(());
                        if fin == FinDaemon::Ferme {
                            // Laisser le wrapper recevoir le tour, puis EOF.
                            thread::sleep(Duration::from_millis(600));
                            let _ = stream.shutdown(std::net::Shutdown::Both);
                            continue;
                        }
                    } else {
                        writer.flush().unwrap();
                    }
                    // Les flux restent ouverts : pas de EOF parasite pour le wrapper.
                    flux_gardes.push((reader, writer));
                }
                Err(erreur) if erreur.kind() == std::io::ErrorKind::WouldBlock => {
                    if liberer_rx.recv_timeout(Duration::from_millis(20)).is_ok()
                        || matches!(liberer_rx.try_recv(), Err(mpsc::TryRecvError::Disconnected))
                    {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let identity = ManagedIdentity {
        instance_id: INSTANCE.to_string(),
        command_id: format!("command-{label}"),
        generation: 1,
    };
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_bridget"));
    let definition = AgentRegistry::from_json(&registre_json, &racine.0.join("state/agents.json"))
        .unwrap()
        .resolved_definition("faux-natif")
        .unwrap();
    let mut env = BTreeMap::from([
        ("HOME".to_string(), racine.0.as_os_str().to_owned()),
        (
            "BRIDGET_HOME".to_string(),
            racine.0.join("state").into_os_string(),
        ),
        ("BRIDGET_SOCKET".to_string(), socket.into_os_string()),
        (
            "PATH".to_string(),
            OsString::from("/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin"),
        ),
        ("USER".to_string(), OsString::from("tester")),
        ("LANG".to_string(), OsString::from("C")),
        ("TMPDIR".to_string(), OsString::from("/tmp")),
        ("BRIDGET_CHANNEL".to_string(), OsString::from("unix")),
    ]);
    if natif {
        let bootstrap = serde_json::json!({"instance_id": INSTANCE, "mission_id": uuid::Uuid::new_v4().to_string()});
        env.insert(
            BOOTSTRAP_ENV.to_string(),
            OsString::from(bootstrap.to_string()),
        );
    }
    if persistant {
        env.insert(
            "BRIDGET_MANAGED_PERSISTENT".to_string(),
            OsString::from("1"),
        );
    }
    let launch = ManagedLaunch {
        bootstrap_executable: binary.clone(),
        identity,
        wrapper_executable: binary,
        wrapper_args: vec![
            "managed-wrapper".to_string(),
            "faux-natif".to_string(),
            AGENT_ID.to_string(),
            serde_json::to_string(&definition).unwrap(),
        ],
        cwd: racine.0.clone(),
        env,
    };
    let marker_store = ManagedMarkerStore::at_directory(racine.0.join("managed"));
    let journal_wrapper = racine.0.join("wrapper.stderr");
    let mut wrapper = spawn_managed_bootstrap_with_stderr(
        &launch,
        std::process::Stdio::from(fs::File::create(&journal_wrapper).unwrap()),
    )
    .unwrap()
    .wait_ready()
    .unwrap()
    .persist_marker(&marker_store, AGENT_ID)
    .unwrap()
    .release()
    .unwrap();
    pret_rx
        .recv_timeout(Duration::from_secs(10))
        .expect("le faux daemon a remis le tour");
    attendre("fournisseur démarré", Duration::from_secs(10), || {
        fs::read_to_string(&pid_file).is_ok_and(|texte| texte.trim().parse::<i32>().is_ok())
    });
    let pid_fournisseur: i32 = fs::read_to_string(&pid_file)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert!(
        processus_vivant(pid_fournisseur),
        "précondition : fournisseur vivant"
    );
    // Précondition O4 : le fournisseur n'est PAS dans le groupe du wrapper.
    let pgid_fournisseur = unsafe { libc::getpgid(pid_fournisseur) };
    let pgid_wrapper = unsafe { libc::getpgid(wrapper.child_mut().id() as i32) };
    assert_ne!(
        pgid_fournisseur, pgid_wrapper,
        "précondition : groupe indépendant"
    );
    Scenario {
        journal_wrapper,
        _racine: racine,
        wrapper,
        pid_fournisseur,
        connexions,
        _daemon: daemon,
        _liberer: liberer_tx,
    }
}

/// Arrête un PID de fixture après avoir vérifié sa ligne de commande.
fn arreter_pid_de_fixture(pid: i32) {
    if !processus_vivant(pid) {
        return;
    }
    let commande = Command::new("ps")
        .args(["-o", "command=", "-p", &pid.to_string()])
        .output()
        .unwrap();
    let commande = String::from_utf8_lossy(&commande.stdout).to_string();
    assert!(
        commande.contains("faux-acp-native-149"),
        "PID {pid} n'est pas le fournisseur de fixture: {commande}"
    );
    assert!(!commande.to_lowercase().contains("firefox"));
    unsafe { libc::kill(pid, libc::SIGTERM) };
    attendre(
        "fournisseur de fixture arrêté",
        Duration::from_secs(5),
        || !processus_vivant(pid),
    );
}

impl Drop for Scenario {
    fn drop(&mut self) {
        // Filet : jamais de fournisseur ni de wrapper résiduel après un échec de test.
        let wrapper = self.wrapper.child_mut().id() as i32;
        if self.wrapper.child_mut().try_wait().ok().flatten().is_none() && processus_vivant(wrapper)
        {
            unsafe { libc::kill(wrapper, libc::SIGTERM) };
            let fin = Instant::now() + Duration::from_secs(5);
            while self.wrapper.child_mut().try_wait().ok().flatten().is_none()
                && Instant::now() < fin
            {
                thread::sleep(Duration::from_millis(25));
            }
        }
        if processus_vivant(self.pid_fournisseur) {
            let commande = Command::new("ps")
                .args(["-o", "command=", "-p", &self.pid_fournisseur.to_string()])
                .output()
                .unwrap();
            if String::from_utf8_lossy(&commande.stdout).contains("faux-acp-native-149") {
                unsafe { libc::kill(self.pid_fournisseur, libc::SIGTERM) };
            }
        }
    }
}

fn attendre_sortie(scenario: &mut Scenario, limite: Duration) -> std::process::ExitStatus {
    let fin = Instant::now() + limite;
    loop {
        if let Some(statut) = scenario.wrapper.child_mut().try_wait().unwrap() {
            return statut;
        }
        assert!(
            Instant::now() < fin,
            "le wrapper ne s'est pas arrêté ; stderr du wrapper :\n{}",
            fs::read_to_string(&scenario.journal_wrapper).unwrap_or_default()
        );
        thread::sleep(Duration::from_millis(25));
    }
}

fn connexions_recues(scenario: &Scenario) -> usize {
    let mut total = 0;
    while scenario.connexions.try_recv().is_ok() {
        total += 1;
    }
    total
}

/// O4 : SIGTERM sur le seul wrapper natif. Le fournisseur (autre groupe) ne reste pas orphelin.
#[test]
fn native149_sigterm_du_wrapper_natif_arrete_le_fournisseur_sans_reconnexion() {
    let _serie = SERIE.lock().unwrap_or_else(|poison| poison.into_inner());
    let mut scenario = lancer("sigterm-natif", true, true, FinDaemon::Silence);
    assert_eq!(
        connexions_recues(&scenario),
        1,
        "une seule inscription avant le signal"
    );
    // Le tour reste ouvert (notify_timeout_secs: 600) : sans le signal, le
    // wrapper ne sortirait pas dans la fenêtre d'observation.
    let wrapper = scenario.wrapper.child_mut().id() as i32;
    let depart = Instant::now();
    assert_eq!(unsafe { libc::kill(wrapper, libc::SIGTERM) }, 0);

    let statut = attendre_sortie(&mut scenario, Duration::from_secs(15));
    let sortie = depart.elapsed();
    assert!(
        sortie < Duration::from_secs(8),
        "sortie coopérative et rapide ({sortie:?}) : le tour reste ouvert côté fournisseur"
    );
    assert_eq!(
        statut.signal(),
        None,
        "sortie coopérative : le wrapper n'est pas tué par le signal ({statut:?})"
    );
    attendre(
        "fournisseur d'un autre groupe arrêté par le wrapper",
        Duration::from_secs(10),
        || !processus_vivant(scenario.pid_fournisseur),
    );
    assert!(
        depart.elapsed() < Duration::from_secs(8),
        "fournisseur arrêté sans orphelin durable ({:?})",
        depart.elapsed()
    );
    thread::sleep(Duration::from_secs(2));
    assert_eq!(
        connexions_recues(&scenario),
        0,
        "aucune reconnexion après l'arrêt natif"
    );
}

/// Même garantie sur coupure du daemon : le natif ne se reconnecte jamais et rend son fournisseur.
#[test]
fn native149_eof_du_daemon_arrete_le_wrapper_natif_et_son_fournisseur() {
    let _serie = SERIE.lock().unwrap_or_else(|poison| poison.into_inner());
    let mut scenario = lancer("eof-natif", true, true, FinDaemon::Ferme);
    let _ = connexions_recues(&scenario);

    let statut = attendre_sortie(&mut scenario, Duration::from_secs(15));
    assert_eq!(statut.signal(), None, "sortie normale ({statut:?})");
    attendre(
        "fournisseur arrêté après EOF",
        Duration::from_secs(10),
        || !processus_vivant(scenario.pid_fournisseur),
    );
    thread::sleep(Duration::from_secs(2));
    assert_eq!(
        connexions_recues(&scenario),
        0,
        "le natif ne se reconnecte pas"
    );
}

/// Témoin : le même EOF, sans bootstrap natif, fait reconnecter le wrapper ordinaire.
#[test]
fn native149_temoin_ordinaire_eof_se_reconnecte_et_garde_son_fournisseur() {
    let _serie = SERIE.lock().unwrap_or_else(|poison| poison.into_inner());
    let mut scenario = lancer("eof-ordinaire", false, true, FinDaemon::Ferme);
    let mut total = connexions_recues(&scenario);
    let fin = Instant::now() + Duration::from_secs(20);
    while total < 2 && Instant::now() < fin {
        thread::sleep(Duration::from_millis(100));
        total += connexions_recues(&scenario);
    }
    assert!(
        total >= 2,
        "le wrapper ordinaire se reconnecte ({total} inscription(s))"
    );
    assert!(
        scenario.wrapper.child_mut().try_wait().unwrap().is_none(),
        "wrapper ordinaire toujours vivant"
    );
    assert!(
        processus_vivant(scenario.pid_fournisseur),
        "fournisseur ordinaire conservé"
    );
    arreter_pid_de_fixture(scenario.pid_fournisseur);
}

/// Témoin : sans bootstrap, aucun gestionnaire de signal n'est installé. Le
/// wrapper ordinaire garde la disposition par défaut de SIGTERM.
#[test]
fn native149_temoin_ordinaire_sigterm_garde_la_disposition_par_defaut() {
    let _serie = SERIE.lock().unwrap_or_else(|poison| poison.into_inner());
    let mut scenario = lancer("sigterm-ordinaire", false, true, FinDaemon::Silence);
    let wrapper = scenario.wrapper.child_mut().id() as i32;
    assert_eq!(unsafe { libc::kill(wrapper, libc::SIGTERM) }, 0);

    let statut = attendre_sortie(&mut scenario, Duration::from_secs(15));
    assert_eq!(
        statut.signal(),
        Some(libc::SIGTERM),
        "ordinaire inchangé : tué par le signal ({statut:?})"
    );
    // Le fournisseur d'un autre groupe survit à un wrapper tué : c'est précisément
    // le défaut O4, propre au natif seulement corrigé. On le nettoie.
    arreter_pid_de_fixture(scenario.pid_fournisseur);
}
