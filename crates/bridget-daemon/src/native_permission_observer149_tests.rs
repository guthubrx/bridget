//! Session 149 — volet G-P : observer temporaire du CLI Claude détenu
//! (S149-33 branches a/b/c, oracle G-P-08). Le hook CLI réel reste la recette
//! T038 ; ici le client est un enfant Python qui parle au socket Unix de
//! l'observer comme le ferait le hook. Fixture privée, aucun modèle.

use super::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

const PYTHON: &str = "/usr/bin/python3";

static CLAUDE_PATH: std::sync::Once = std::sync::Once::new();

/// `Observer::start` lit le PATH du processus (`source_environment`). Fixture
/// unitaire, jamais une preuve modèle : un vrai binaire Mach-O nommé `claude`
/// est placé en tête du PATH, une seule fois, hors de toute racine de test.
fn ensure_claude_on_path() {
    CLAUDE_PATH.call_once(|| {
        let bin = std::env::temp_dir().join(format!("bridget-np149o-bin-{}", std::process::id()));
        fs::create_dir_all(&bin).unwrap();
        fs::copy("/bin/echo", bin.join("claude")).unwrap();
        fs::set_permissions(bin.join("claude"), fs::Permissions::from_mode(0o755)).unwrap();
        let mut paths = vec![bin];
        paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()));
        // SAFETY : appelé une seule fois, avant tout Observer::start du module.
        unsafe { std::env::set_var("PATH", std::env::join_paths(paths).unwrap()) };
    });
}

fn fixture_root(label: &str) -> PathBuf {
    ensure_claude_on_path();
    // Nom court : le socket Unix (104 octets sous macOS) vit sous cette racine.
    let root = std::env::temp_dir().join(format!(
        "n149o-{}-{}",
        label,
        &uuid::Uuid::new_v4().simple().to_string()[..8]
    ));
    fs::create_dir_all(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    root
}

fn collector() -> (Arc<dyn Fn(WrapperToDaemon) + Send + Sync>, mpsc::Receiver<WrapperToDaemon>) {
    let (tx, rx) = mpsc::channel();
    (Arc::new(move |event| {
        let _ = tx.send(event);
    }), rx)
}

fn definition(root: &Path) -> AgentDefinition {
    AgentDefinition {
        command: "claude".into(),
        args: vec![],
        protocol: "claude_stream_json".into(),
        forbidden_env: vec![],
        pass_env: vec![],
        claude_config_dir: Some(root.to_string_lossy().into_owned()),
        permissions: "allow".into(),
        queue_capacity: 4,
        notify_timeout_secs: 4,
        mcp: Default::default(),
        capabilities: Default::default(),
    }
}

fn write_client_script(root: &Path) -> PathBuf {
    let script = root.join("client-149.py");
    fs::write(
        &script,
        "import socket,sys,json,time\n\
         payload=json.load(open(sys.argv[1]))\n\
         sock=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM)\n\
         sock.settimeout(6)\n\
         time.sleep(0.4)\n\
         sock.connect(sys.argv[2])\n\
         sock.sendall((json.dumps(payload)+\"\\n\").encode())\n\
         buf=b\"\"\n\
         try:\n\
         \x20   while not buf.endswith(b\"\\n\"):\n\
         \x20       chunk=sock.recv(4096)\n\
         \x20       if not chunk:\n\
         \x20           break\n\
         \x20       buf+=chunk\n\
         except (socket.timeout,OSError):\n\
         \x20   pass\n\
         data=buf.decode().strip() or \"SILENT\"\n\
         open(sys.argv[3],\"w\").write(data)\n",
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o600)).unwrap();
    script
}

/// Connecte un enfant Python réel au socket de l'observer. Le PID de cet
/// enfant est celui que `bind_provider` atteste : le pair est donc bien un
/// descendant du processus fournisseur lié (l'enfant est sa propre racine).
fn connect_payload(observer: &Observer, root: &Path, payload: Value, out: &Path) -> std::process::Child {
    let script = write_client_script(root);
    let payload_path = root.join(format!("payload-{}.json", uuid::Uuid::new_v4().simple()));
    fs::write(&payload_path, serde_json::to_vec(&payload).unwrap()).unwrap();
    Command::new(PYTHON)
        .args([script.to_str().unwrap(), payload_path.to_str().unwrap(), socket_argument(observer).as_str(), out.to_str().unwrap()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
}

/// Le socket et le nonce ne sont publiés que dans la composition d'overlay :
/// c'est exactement ce que lit le vrai hook.
fn overlay_identity(observer: &Observer) -> (String, String) {
    let settings: Value = serde_json::from_slice(&fs::read(observer.overlay()).unwrap()).unwrap();
    let command = settings["hooks"]["PreToolUse"][0]["hooks"][0]["command"].as_str().unwrap().to_string();
    let parts: Vec<String> = command.split(' ').map(|part| part.trim_matches('\'').to_string()).collect();
    assert_eq!(parts[1], "__native-permission-observer");
    (parts[2].clone(), parts[3].clone())
}

fn socket_argument(observer: &Observer) -> String {
    overlay_identity(observer).0
}

fn valid_payload(observer: &Observer, session: &str, tool_input: Value) -> Value {
    json!({
        "version":1,
        "nonce":overlay_identity(observer).1,
        "permission_mode":"default",
        "session_id":session,
        "prompt_id":"prompt-149",
        "cwd":std::env::current_dir().unwrap().to_string_lossy(),
        "tool_name":"mcp__bridget__bridget_delegate",
        "tool_input":tool_input,
        "hook_event_name":"PreToolUse",
    })
}

fn wait_file(path: &Path) -> String {
    let deadline = std::time::Instant::now() + Duration::from_secs(9);
    loop {
        // Le client crée puis écrit le fichier : un contenu vide est une écriture en cours.
        if let Ok(content) = fs::read_to_string(path)
            && !content.is_empty()
        {
            return content;
        }
        if std::time::Instant::now() > deadline {
            panic!("le client {} n'a jamais répondu", path.display());
        }
        std::thread::sleep(Duration::from_millis(40));
    }
}

fn wait_fact(rx: &mpsc::Receiver<WrapperToDaemon>) -> (Option<ProviderPermissions>, Option<String>, Option<String>, Option<String>) {
    match rx.recv_timeout(Duration::from_secs(6)).unwrap() {
        WrapperToDaemon::NativePermissionFact { fact, request_id, observation_id, unavailable_code } => (fact, request_id, observation_id, unavailable_code),
        autre => panic!("événement inattendu : {autre:?}"),
    }
}

#[test]
fn native149_lancement_refuse_un_settings_preexistant_sans_trace() {
    let root = fixture_root("settings-preexistant");
    for arg in ["--settings", "--settings={\"permissions\":{}}"] {
        let (emit, _rx) = collector();
        let Err(erreur) = Observer::start(&root, &definition(&root), &[arg.to_string()], "instance-149", emit) else {
            panic!("le lancement doit refuser un settings préexistant pour {arg}")
        };
        assert_eq!(erreur, "permission_source_unavailable", "arg {arg}");
        assert!(fs::read_dir(&root).unwrap().next().is_none(), "aucun fichier résiduel pour {arg}");
    }
}

#[test]
fn native149_cycle_de_vie_fichiers_prives_et_nettoyage_raii() {
    let root = fixture_root("lifecycle");
    let (emit, _rx) = collector();
    let observer = Observer::start(&root, &definition(&root), &[], "instance-149", emit).unwrap();
    let mut vus = Vec::new();
    for entry in fs::read_dir(&root).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().to_string_lossy().into_owned();
        vus.push(name.clone());
        let mode = entry.metadata().unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "{name} doit être privé");
    }
    vus.sort();
    assert_eq!(vus.len(), 2, "exactement socket + overlay : {vus:?}");
    assert!(vus.iter().any(|name| name.starts_with("np-") && name.ends_with(".sock")));
    assert!(vus.iter().any(|name| name.starts_with("no-") && name.ends_with(".json")));
    // La composition d'overlay vise les deux outils Bridget, et rien d'autre.
    let settings: Value = serde_json::from_slice(&fs::read(observer.overlay()).unwrap()).unwrap();
    let matcher = settings["hooks"]["PreToolUse"][0]["matcher"].as_str().unwrap();
    assert_eq!(matcher, "mcp__bridget__bridget_delegate|mcp__bridget__bridget_capabilities");
    // Aucun droit de la session ne traverse l'overlay.
    let overlay = fs::read_to_string(observer.overlay()).unwrap();
    assert!(!overlay.contains("session_id") && !overlay.contains("prompt_id") && !overlay.contains("nonce"));
    drop(observer);
    assert!(fs::read_dir(&root).unwrap().next().is_none(), "RAII : socket et overlay supprimés");
}

#[test]
fn native149_fait_publie_acquitte_le_hook_avant_sa_reprise() {
    let root = fixture_root("ack");
    let (emit, rx) = collector();
    let observer = Observer::start(&root, &definition(&root), &[], "instance-149", emit).unwrap();
    let out = root.join("out-ack.json");
    let client = connect_payload(&observer, &root, valid_payload(&observer, "sess-149", json!({"request_id":"req-149"})), &out);
    observer.bind_provider(client.id()).unwrap();
    let (fact, request_id, observation_id, unavailable) = wait_fact(&rx);
    assert!(unavailable.is_none());
    let fact = fact.expect("le fait validé est publié");
    assert_eq!(fact.version, 1);
    assert_eq!(fact.source, "native_wrapper");
    assert_eq!(fact.run_id, "prompt-149");
    assert_eq!(fact.provider_session_id, "sess-149");
    assert_eq!(fact.provider_instance_id, "instance-149");
    assert_eq!(fact.driver, "claude_stream_json");
    assert_eq!(fact.cwd, std::env::current_dir().unwrap().to_string_lossy());
    assert_eq!(fact.runtime_mode, "approval-required");
    assert_eq!(fact.interaction_mode, "default");
    assert_eq!(fact.provider_policy["permission_mode"], json!("default"));
    assert_eq!(fact.provider_policy["permission_callback"]["kind"], json!("native_wrapper"));
    assert!(fact.provider_policy["launch_context"].is_object());
    assert_eq!(bridget_transport::protocol::validate_permissions(&fact, false), Ok(()));
    assert_eq!(request_id.as_deref(), Some("req-149"), "la demande délégataire est corrélée");
    let observation_id = observation_id.expect("observation corrélée");
    // Ni le nonce, ni le socket, ni l'overlay ne quittent l'observer.
    let (_, nonce) = overlay_identity(&observer);
    let sérialisé = serde_json::to_string(&fact).unwrap();
    assert!(!sérialisé.contains(&nonce) && !sérialisé.contains("np-") && !sérialisé.contains("no-"));
    // L'ACK précède la reprise du hook : sans lui, rien ne repart.
    Observer::acknowledge(&observer.acknowledgement_sink(), observation_id, true);
    assert_eq!(wait_file(&out), "{\"accepted\":true,\"code\":\"ok\"}");
}

#[test]
fn native149_acquittement_negatif_bloque_le_hook_par_nom() {
    let root = fixture_root("ack-false");
    let (emit, rx) = collector();
    let observer = Observer::start(&root, &definition(&root), &[], "instance-149", emit).unwrap();
    let out = root.join("out-ack-false.json");
    let client = connect_payload(&observer, &root, valid_payload(&observer, "sess-149", json!({"request_id":"req-149"})), &out);
    observer.bind_provider(client.id()).unwrap();
    let (_, _, observation_id, _) = wait_fact(&rx);
    Observer::acknowledge(&observer.acknowledgement_sink(), observation_id.unwrap(), false);
    assert_eq!(wait_file(&out), "{\"accepted\":false,\"code\":\"permission_attestation_unavailable\"}");
}

#[test]
fn native149_overlay_modifie_est_refuse_par_un_refus_nomme() {
    let root = fixture_root("tamper");
    let (emit, rx) = collector();
    let observer = Observer::start(&root, &definition(&root), &[], "instance-149", emit).unwrap();
    let mut contenu = fs::read(observer.overlay()).unwrap();
    contenu.push(b' ');
    fs::write(observer.overlay(), &contenu).unwrap();
    let out = root.join("out-tamper.json");
    let client = connect_payload(&observer, &root, valid_payload(&observer, "sess-149", json!({"request_id":"req-149"})), &out);
    observer.bind_provider(client.id()).unwrap();
    let (fact, _, observation_id, unavailable) = wait_fact(&rx);
    // L'overlay n'est pas une source de settings : sa divergence tombe sous
    // le refus d'attestation nommé, jamais sous un succès silencieux.
    assert_eq!(unavailable.as_deref(), Some("permission_attestation_unavailable"));
    assert!(fact.is_none(), "aucun fait n'est publié sur un overlay mué");
    assert!(observation_id.is_none());
    assert_eq!(wait_file(&out), "{\"accepted\":false,\"code\":\"permission_attestation_unavailable\"}");
}

#[test]
fn native149_delegation_sans_request_id_est_refusee_sans_fait() {
    let root = fixture_root("unbound");
    let (emit, rx) = collector();
    let observer = Observer::start(&root, &definition(&root), &[], "instance-149", emit).unwrap();
    let out = root.join("out-unbound.json");
    let client = connect_payload(&observer, &root, valid_payload(&observer, "sess-149", json!({})), &out);
    observer.bind_provider(client.id()).unwrap();
    assert_eq!(wait_file(&out), "{\"accepted\":false,\"code\":\"permission_attestation_unavailable\"}");
    assert!(rx.recv_timeout(Duration::from_millis(300)).is_err(), "aucun fait émis sans request_id");
}

#[test]
fn native149_nonce_inconnu_et_pair_etranger_restant_silencieux() {
    let root = fixture_root("silence");
    let (emit, rx) = collector();
    let observer = Observer::start(&root, &definition(&root), &[], "instance-149", emit).unwrap();
    // Nonce faux : aucun accusé, aucun fait.
    let mut faux = valid_payload(&observer, "sess-149", json!({"request_id":"req-149"}));
    faux["nonce"] = json!("0".repeat(64));
    let out = root.join("out-nonce.json");
    let client = connect_payload(&observer, &root, faux, &out);
    observer.bind_provider(client.id()).unwrap();
    assert_eq!(wait_file(&out), "SILENT");
    // Une fois ce pair mort, une reconnexion même bien signée reste silencieuse :
    // la naissance liée n'existe plus, aucune récupération n'est tentée.
    let out = root.join("out-unbound-pair.json");
    let mut client = connect_payload(&observer, &root, valid_payload(&observer, "sess-149", json!({"request_id":"req-149"})), &out);
    for _ in 0..40 {
        if client.try_wait().unwrap().is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(40));
    }
    let _ = client.wait();
    assert_eq!(wait_file(&out), "SILENT");
    assert!(rx.recv_timeout(Duration::from_millis(300)).is_err(), "aucun fait pour un pair étranger ou mort");
}

#[test]
fn native149_liaison_d_un_pid_mort_est_refusee_et_ne_recupere_jamais() {
    let root = fixture_root("pid-mort");
    let (emit, _rx) = collector();
    let observer = Observer::start(&root, &definition(&root), &[], "instance-149", emit).unwrap();
    let mut ephemère = Command::new(PYTHON).arg("-c").arg("pass").spawn().unwrap();
    let pid = ephemère.id();
    ephemère.wait().unwrap();
    let erreur = observer.bind_provider(pid).unwrap_err();
    assert_eq!(erreur, "permission_source_unavailable");
    // Un fournisseur vivant lié puis mort : un client qui se connecte ensuite
    // n'est jamais récupéré (naissance absente → silence).
    let mut fournisseur = Command::new(PYTHON).arg("-c").arg("import time;time.sleep(30)").spawn().unwrap();
    observer.bind_provider(fournisseur.id()).unwrap();
    fournisseur.kill().unwrap();
    fournisseur.wait().unwrap();
    let out = root.join("out-mort.json");
    let mut client = connect_payload(&observer, &root, valid_payload(&observer, "sess-149", json!({"request_id":"req-149"})), &out);
    for _ in 0..40 {
        if client.try_wait().unwrap().is_some() {
            break;
        }
        std::thread::sleep(Duration::from_millis(40));
    }
    let _ = client.wait();
    assert_eq!(wait_file(&out), "SILENT", "reconnexion sans récupération");
}

#[test]
fn native149_session_attendue_filtre_les_demandes_hors_session() {
    let root = fixture_root("session-attendue");
    let (emit, rx) = collector();
    let observer = Observer::start(&root, &definition(&root), &["--session-id".to_string(), "attendue-149".to_string()], "instance-149", emit).unwrap();
    let out = root.join("out-session.json");
    let client = connect_payload(&observer, &root, valid_payload(&observer, "sess-149", json!({"request_id":"req-149"})), &out);
    observer.bind_provider(client.id()).unwrap();
    let (fact, _, observation_id, unavailable) = wait_fact(&rx);
    assert_eq!(unavailable.as_deref(), Some("permission_attestation_unavailable"));
    assert!(fact.is_none() && observation_id.is_none());
    assert_eq!(wait_file(&out), "{\"accepted\":false,\"code\":\"permission_attestation_unavailable\"}");
}

#[test]
fn native149_fait_capacities_est_informationnel_et_corrige() {
    let root = fixture_root("capacities");
    let (emit, rx) = collector();
    let observer = Observer::start(&root, &definition(&root), &[], "instance-149", emit).unwrap();
    let out = root.join("out-capacities.json");
    let payload = valid_payload(&observer, "sess-149", json!({}));
    // L'outil capabilities ne porte pas de request_id : le fait reste publié,
    // corrélé au prompt et à la session, jamais un droit d'une autre demande.
    let mut payload = payload;
    payload["tool_name"] = json!("mcp__bridget__bridget_capabilities");
    let client = connect_payload(&observer, &root, payload, &out);
    observer.bind_provider(client.id()).unwrap();
    let (fact, request_id, observation_id, unavailable) = wait_fact(&rx);
    assert!(unavailable.is_none());
    let fact = fact.expect("le fait capabilities est publié");
    assert_eq!(request_id, None);
    assert!(observation_id.is_some());
    assert_eq!(fact.provider_session_id, "sess-149");
    Observer::acknowledge(&observer.acknowledgement_sink(), observation_id.unwrap(), true);
    assert_eq!(wait_file(&out), "{\"accepted\":true,\"code\":\"ok\"}");
}
