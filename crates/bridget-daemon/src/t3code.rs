//! Adaptateur t3code (session 098) : `bridget t3 install|status|uninstall|serve`.
//!
//! Le pont ne modifie jamais t3code. Il lit son serveur local par HTTP en
//! boucle locale avec une session émise par le CLI officiel `t3`, présente
//! chaque fil vivant au daemon Bridget comme un agent (`t3code | cli`), remet
//! les messages par `thread.turn.start` et renvoie à l'expéditeur la réponse
//! du tour corrélé par rang FIFO. Tout l'état est sous `<BRIDGET_HOME>/t3code`.

use std::collections::{BTreeMap, HashSet};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use bridget_transport::journal::{JournalLiveFeed, JournalWriter};
use bridget_transport::protocol::{DaemonToWrapper, PresenceMode, WrapperToDaemon, decode};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};

use crate::t3code_contract::{
    self as contract, Client, ContractError, ServerRuntime, T3Cli, ThreadDetail, ThreadSummary,
};
use crate::wrapper::{
    AttachRelayWorker, IdempotentDeliveryTracker, connect_and_register_at,
    deliver_idempotent_to_interactive, record_interactive_turn, send_wrapper_message,
};

const PROTOCOL: &str = "t3code";
const TOKEN_TTL: &str = "30d";
const DEFAULT_POLL: Duration = Duration::from_secs(3);
const DEFAULT_TURN_WAIT: Duration = Duration::from_secs(120);
const RECONNECT_BACKOFF: Duration = Duration::from_secs(5);
const AUTH_FAILED_PAUSE: Duration = Duration::from_secs(60);
const DETAIL_PAGE: u32 = 20;
const DETAIL_PAGE_MAX: u32 = 320;
const SEEN_BOUND: usize = 1000;
/// Lectures successives d'un fil au repos avant de renoncer à corréler. Au
/// sondage par défaut (3 s) cela laisse une quinzaine de secondes au tour pour
/// démarrer : t3code en met moins d'une, mais renoncer trop tôt perdrait une
/// réponse réelle.
const SETTLE_ATTEMPTS: u32 = 5;
const NAMESPACE_098: uuid::Uuid = uuid::Uuid::from_bytes([
    0x09, 0x8b, 0x71, 0xd3, 0xc0, 0xde, 0x4a, 0x11, 0x9b, 0x1d, 0x73, 0x63, 0x6f, 0x64, 0x65, 0x01,
]);

// ---------------------------------------------------------------------------
// Entrée CLI
// ---------------------------------------------------------------------------

const USAGE: &str = "Usage : bridget t3 <install [--no-service] | status | uninstall | serve>

  install    Émet une session t3 dédiée (`t3 auth session issue`), enregistre
             le service de pont, n'écrit rien dans t3code. Idempotent.
  status     Serveur t3code, session, service et fils exposés.
  uninstall  Retire le service, révoque la session, efface l'état du pont.
  serve      Lance le pont au premier plan (utilisé par le service).

Prérequis : t3code démarré et le CLI `t3` installé (npm i -g t3).
";

pub fn run(arguments: &[String]) -> ! {
    match invoke(arguments) {
        Ok(code) => std::process::exit(code),
        Err(detail) => {
            eprintln!("bridget t3 : {detail}");
            std::process::exit(1);
        }
    }
}

fn invoke(arguments: &[String]) -> Result<i32, String> {
    let Some(operation) = arguments.first() else {
        eprint!("{USAGE}");
        return Ok(2);
    };
    let rest = &arguments[1..];
    match operation.as_str() {
        "install" => {
            let mut service = true;
            for flag in rest {
                match flag.as_str() {
                    "--no-service" => service = false,
                    other => return Err(format!("option inconnue : {other}")),
                }
            }
            install(&Paths::from_environment()?, service)?;
            Ok(0)
        }
        "status" => status(&Paths::from_environment()?),
        "uninstall" => {
            uninstall(&Paths::from_environment()?)?;
            Ok(0)
        }
        "serve" => {
            serve(&Paths::from_environment()?)?;
            Ok(0)
        }
        "--help" | "-h" | "help" => {
            print!("{USAGE}");
            Ok(0)
        }
        other => Err(format!("opération inconnue : {other}\n{USAGE}")),
    }
}

// ---------------------------------------------------------------------------
// État privé du pont
// ---------------------------------------------------------------------------

/// Chemins de l'état du pont : tout sous `<racine Bridget>/t3code`, 0700/0600.
pub(crate) struct Paths {
    pub root: PathBuf,
    pub socket: PathBuf,
    pub dir: PathBuf,
}

impl Paths {
    pub(crate) fn from_environment() -> Result<Self, String> {
        let namespace = crate::environment::Namespace::from_environment()?;
        Ok(Self::at(namespace.root, namespace.socket))
    }

    pub(crate) fn at(root: PathBuf, socket: PathBuf) -> Self {
        let dir = root.join("t3code");
        Self { root, socket, dir }
    }

    fn token(&self) -> PathBuf {
        self.dir.join("token.json")
    }
    fn manifest(&self) -> PathBuf {
        self.dir.join("manifest.json")
    }
    fn status(&self) -> PathBuf {
        self.dir.join("status.json")
    }
    fn threads(&self) -> PathBuf {
        self.dir.join("threads")
    }
    fn thread_state(&self, thread_id: &str) -> PathBuf {
        self.threads().join(format!("{thread_id}.json"))
    }
    fn ensure(&self) -> Result<(), String> {
        crate::environment::ensure_private_directory(&self.dir)?;
        crate::environment::ensure_private_directory(&self.threads())
    }
}

/// Session t3 détenue par le pont. Le fichier est le seul endroit où vit le jeton.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct TokenFile {
    installation_id: String,
    label: String,
    session_id: String,
    token: String,
    expires_at: String,
    issued_at: String,
}

/// Repère d'installation : ce qui a été posé, donc ce que `uninstall` retire.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Manifest {
    installation_id: String,
    label: String,
    installed_at: String,
    service_path: Option<String>,
}

/// Dernier état publié par `serve`, lu par `status`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct StatusFile {
    state: String,
    detail: String,
    updated_at: String,
    threads: usize,
}

/// Remise en attente de sa réponse liée. Survit au redémarrage du pont.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct Pending {
    pub request_id: String,
    pub from: String,
    pub message_id: String,
    pub anchor_turn_id: Option<String>,
    pub dispatched_at: String,
    #[serde(default)]
    pub attempts: u32,
}

/// État durable d'un fil : curseur du journal et remises en attente.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub(crate) struct ThreadState {
    #[serde(default)]
    pub seen: Vec<String>,
    #[serde(default)]
    pub ended_turns: Vec<String>,
    #[serde(default)]
    pub pending: Vec<Pending>,
    #[serde(default)]
    pub seeded: bool,
}

impl ThreadState {
    fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    fn save(&self, path: &Path) -> Result<(), String> {
        let text = serde_json::to_string(self).map_err(|e| e.to_string())?;
        private_write(path, &text)
    }

    fn remember(&mut self, message_id: &str) {
        self.seen.push(message_id.to_string());
        if self.seen.len() > SEEN_BOUND {
            let excess = self.seen.len() - SEEN_BOUND;
            self.seen.drain(..excess);
        }
    }
}

/// Écriture atomique 0600 : fichier temporaire voisin puis renommage.
fn private_write(path: &Path, text: &str) -> Result<(), String> {
    use std::os::unix::fs::OpenOptionsExt;
    let temporary = path.with_extension("tmp");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&temporary)
        .map_err(|e| format!("{} : {e}", temporary.display()))?;
    file.write_all(text.as_bytes())
        .and_then(|_| file.sync_all())
        .map_err(|e| format!("{} : {e}", temporary.display()))?;
    std::fs::rename(&temporary, path).map_err(|e| format!("{} : {e}", path.display()))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Option<T>, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text)
            .map(Some)
            .map_err(|e| format!("{} illisible : {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("{} : {e}", path.display())),
    }
}

/// Sessions émises que le pont n'a pas pu révoquer : `uninstall` les reprend.
fn orphans_path(token_path: &Path) -> PathBuf {
    token_path.with_file_name("orphan-sessions.json")
}

fn note_orphan(token_path: &Path, session_id: &str) {
    let path = orphans_path(token_path);
    let mut known: Vec<String> = read_json(&path).ok().flatten().unwrap_or_default();
    if !known.iter().any(|k| k == session_id) {
        known.push(session_id.to_string());
        let _ = private_write(&path, &serde_json::to_string(&known).unwrap());
    }
}

/// UUID stable dérivé d'un nom (v5) puis présenté comme v4 : le daemon exige
/// un v4 canonique et l'identité d'un fil doit survivre aux redémarrages.
pub(crate) fn stable_uuid(name: &str) -> String {
    let bytes = *uuid::Uuid::new_v5(&NAMESPACE_098, name.as_bytes()).as_bytes();
    uuid::Builder::from_random_bytes(bytes)
        .into_uuid()
        .hyphenated()
        .to_string()
}

/// Type d'agent affiché dans `who` : l'instance de fournisseur t3code
/// (`claudeAgent`, `codex`, `antigravity`) ramenée au vocabulaire Bridget.
/// Un fil jamais démarré n'a pas de session : seule `modelSelection` le dit.
pub(crate) fn agent_type_for(provider: &str) -> String {
    let lower = provider.to_ascii_lowercase();
    if lower.contains("antigravity") || lower.contains("gemini") {
        return "gemini".to_string();
    }
    for known in ["claude", "codex"] {
        if lower.contains(known) {
            return known.to_string();
        }
    }
    lower
}

/// Fournisseur du fil : la session quand elle existe, sinon le modèle choisi.
pub(crate) fn thread_provider(summary: &ThreadSummary) -> Option<String> {
    summary
        .session
        .as_ref()
        .map(|session| session.provider_name.clone())
        .or_else(|| summary.provider_instance_id.clone())
        .map(|provider| agent_type_for(&provider))
}

// ---------------------------------------------------------------------------
// install / status / uninstall
// ---------------------------------------------------------------------------

fn install(paths: &Paths, with_service: bool) -> Result<(), String> {
    paths.ensure()?;
    if let Some(manifest) = read_json::<Manifest>(&paths.manifest())?
        && read_json::<TokenFile>(&paths.token())?.is_some()
    {
        println!(
            "déjà installé (installation {}, session « {} »)",
            manifest.installation_id, manifest.label
        );
        if with_service && manifest.service_path.is_none() {
            let service_path = service::install()?;
            let manifest = Manifest {
                service_path: Some(service_path),
                ..manifest
            };
            private_write(
                &paths.manifest(),
                &serde_json::to_string(&manifest).unwrap(),
            )?;
            println!("service enregistré");
        }
        return Ok(());
    }

    match contract::read_runtime(&contract::base_dir().map_err(|e| e.to_string())?) {
        Ok(runtime) => println!(
            "serveur t3code : {} (pid {})",
            runtime.base_url(),
            runtime.pid
        ),
        Err(e) => println!("serveur t3code : {e} (le pont attendra son démarrage)"),
    }

    let installation_id = uuid::Uuid::new_v4().simple().to_string()[..12].to_string();
    let label = format!("bridget-{installation_id}");
    let cli = T3Cli::from_env();
    let issued = match cli.issue_session(&label, TOKEN_TTL) {
        Ok(issued) => issued,
        Err(e) => {
            // La commande a pu créer la session sans que sa sortie soit lisible.
            let _ = revoke_by_label(&cli, &label);
            return Err(format!("émission de la session t3 : {e}"));
        }
    };
    let token = TokenFile {
        installation_id: installation_id.clone(),
        label: label.clone(),
        session_id: issued.session_id,
        token: issued.token,
        expires_at: issued.expires_at,
        issued_at: contract::iso_now(),
    };
    if let Err(e) = private_write(&paths.token(), &serde_json::to_string(&token).unwrap()) {
        let _ = cli.revoke_session(&token.session_id);
        return Err(e);
    }
    let mut manifest = Manifest {
        installation_id,
        label: label.clone(),
        installed_at: contract::iso_now(),
        service_path: None,
    };
    if with_service {
        match service::install() {
            Ok(path) => manifest.service_path = Some(path),
            Err(e) => {
                // Retour arrière complet : rien ne reste d'une installation ratée.
                let _ = cli.revoke_session(&token.session_id);
                let _ = std::fs::remove_file(paths.token());
                return Err(format!("service : {e}"));
            }
        }
    }
    if let Err(detail) = private_write(
        &paths.manifest(),
        &serde_json::to_string(&manifest).unwrap(),
    ) {
        // Sans manifeste, `uninstall` ne saurait ni quel service retirer ni
        // quelle session révoquer : on défait tout de suite.
        if let Some(path) = &manifest.service_path {
            let _ = service::uninstall(Path::new(path));
        }
        let _ = cli.revoke_session(&token.session_id);
        let _ = std::fs::remove_file(paths.token());
        return Err(detail);
    }
    println!(
        "installé : session « {label} » (expire {}), service {}",
        token.expires_at,
        if with_service {
            "enregistré"
        } else {
            "non demandé"
        }
    );
    Ok(())
}

/// Révoque toutes les sessions portant ce libellé ; rend celles qui résistent.
fn revoke_by_label(cli: &T3Cli, label: &str) -> Vec<String> {
    let Ok(sessions) = cli.list_sessions() else {
        return vec![format!("liste des sessions illisible (libellé {label})")];
    };
    let mut failed = Vec::new();
    for (session_id, session_label) in sessions {
        if session_label.as_deref() == Some(label)
            && let Err(e) = cli.revoke_session(&session_id)
        {
            failed.push(format!("{session_id} ({e})"));
        }
    }
    failed
}

fn uninstall(paths: &Paths) -> Result<(), String> {
    let manifest = read_json::<Manifest>(&paths.manifest())?;
    let token = read_json::<TokenFile>(&paths.token())?;
    if manifest.is_none() && token.is_none() && !paths.dir.exists() {
        println!("rien à retirer");
        return Ok(());
    }
    if let Some(path) = manifest.as_ref().and_then(|m| m.service_path.as_deref()) {
        service::uninstall(Path::new(path))?;
        println!("service retiré");
    }
    let cli = T3Cli::from_env();
    let label = token
        .as_ref()
        .map(|t| t.label.clone())
        .or_else(|| manifest.as_ref().map(|m| m.label.clone()));
    let mut to_revoke: Vec<String> =
        read_json::<Vec<String>>(&orphans_path(&paths.token()))?.unwrap_or_default();
    if let Some(token) = &token {
        to_revoke.push(token.session_id.clone());
    }
    let mut failed = Vec::new();
    for session_id in &to_revoke {
        match cli.revoke_session(session_id) {
            Ok(()) => println!("session {session_id} révoquée"),
            Err(e) => failed.push(format!("{session_id} ({e})")),
        }
    }
    // Filet par libellé : une session dont l'identifiant est perdu reste
    // trouvable par son libellé, qui n'appartient qu'à cette installation.
    if let Some(label) = &label {
        failed.extend(revoke_by_label(&cli, label));
    }
    if !failed.is_empty() {
        // Ne rien effacer : sans l'état local, plus personne ne saurait quelle
        // session administrative traîne encore dans t3code.
        return Err(format!(
            "sessions non révoquées : {} — état conservé ; vérifier `t3 auth session list` puis relancer `bridget t3 uninstall`",
            failed.join(", ")
        ));
    }
    if paths.dir.exists() {
        std::fs::remove_dir_all(&paths.dir)
            .map_err(|e| format!("{} : {e}", paths.dir.display()))?;
    }
    println!("état du pont effacé ; t3code n'a jamais été modifié");
    Ok(())
}

fn status(paths: &Paths) -> Result<i32, String> {
    let manifest = read_json::<Manifest>(&paths.manifest())?;
    let token = read_json::<TokenFile>(&paths.token())?;
    let Some(manifest) = manifest else {
        println!("non installé (bridget t3 install)");
        return Ok(1);
    };
    println!(
        "installation : {} ({})",
        manifest.installation_id, manifest.installed_at
    );
    match &token {
        Some(token) => println!(
            "session t3    : « {} », expire {}",
            token.label, token.expires_at
        ),
        None => println!("session t3    : absente (réinstaller)"),
    }
    println!(
        "service       : {}",
        manifest.service_path.as_deref().unwrap_or("non enregistré")
    );
    let runtime = contract::base_dir()
        .map_err(|e| e.to_string())
        .and_then(|base| contract::read_runtime(&base).map_err(|e| e.to_string()));
    match &runtime {
        Ok(runtime) => println!(
            "serveur t3code : {} (pid {})",
            runtime.base_url(),
            runtime.pid
        ),
        Err(e) => println!("serveur t3code : {e}"),
    }
    if let Some(bridge) = read_json::<StatusFile>(&paths.status())? {
        println!(
            "pont          : {} — {} ({} fil(s), {})",
            bridge.state, bridge.detail, bridge.threads, bridge.updated_at
        );
    } else {
        println!("pont          : jamais démarré");
    }
    if let (Ok(runtime), Some(token)) = (runtime, token) {
        match Client::new(&runtime, &token.token).snapshot() {
            Ok(snapshot) => {
                for thread in snapshot.threads.iter().filter(|t| t.is_live()) {
                    println!(
                        "  fil {} « {} » → agent {} ({})",
                        &thread.id[..thread.id.len().min(8)],
                        thread.title,
                        stable_uuid(&thread.id),
                        thread_provider(thread)
                            .unwrap_or_else(|| "fournisseur inconnu".to_string())
                    );
                }
            }
            Err(ContractError::Unauthorized) => println!("  session refusée par t3code (401)"),
            Err(e) => println!("  fils illisibles : {e}"),
        }
    }
    Ok(0)
}

// ---------------------------------------------------------------------------
// Service de pont (launchd / systemd --user)
// ---------------------------------------------------------------------------

mod service {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    const LABEL: &str = "com.bridget.t3";

    fn env_pairs() -> Vec<(String, String)> {
        [
            "HOME",
            "PATH",
            "USER",
            "BRIDGET_HOME",
            "BRIDGET_SOCKET",
            "T3CODE_HOME",
            "BRIDGET_T3_BIN",
        ]
        .iter()
        .filter_map(|key| {
            std::env::var(key)
                .ok()
                .map(|value| (key.to_string(), value))
        })
        .collect()
    }

    fn run(program: &str, args: &[&str]) -> Result<(), String> {
        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(|e| format!("{program} : {e}"))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "{program} {} : {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    }

    /// Journal du service, volontairement HORS du namespace Bridget :
    /// launchd crée ses fichiers de sortie en 0644, et le daemon refuse de
    /// démarrer si un état non privé traîne dans son namespace.
    pub(super) fn log_path(home: &str) -> PathBuf {
        PathBuf::from(home)
            .join("Library/Logs")
            .join(format!("{LABEL}.log"))
    }

    pub(super) fn install() -> Result<String, String> {
        let bridget = std::env::current_exe().map_err(|e| e.to_string())?;
        let home = std::env::var("HOME").map_err(|_| "HOME absent".to_string())?;
        let log = log_path(&home);
        if let Some(parent) = log.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{} : {e}", parent.display()))?;
        }
        if cfg!(target_os = "macos") {
            let path = PathBuf::from(&home)
                .join("Library/LaunchAgents")
                .join(format!("{LABEL}.plist"));
            let env = env_pairs()
                .into_iter()
                .map(|(k, v)| format!("      <key>{k}</key><string>{}</string>\n", xml(&v)))
                .collect::<String>();
            let plist = format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\"><dict>\n\
  <key>Label</key><string>{LABEL}</string>\n\
  <key>ProgramArguments</key><array>\n\
    <string>{}</string><string>t3</string><string>serve</string>\n\
  </array>\n\
  <key>EnvironmentVariables</key><dict>\n{env}  </dict>\n\
  <key>RunAtLoad</key><true/>\n  <key>KeepAlive</key><true/>\n\
  <key>StandardOutPath</key><string>{}</string>\n\
  <key>StandardErrorPath</key><string>{}</string>\n\
</dict></plist>\n",
                xml(&bridget.display().to_string()),
                xml(&log.display().to_string()),
                xml(&log.display().to_string()),
            );
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(&path, plist).map_err(|e| format!("{} : {e}", path.display()))?;
            let domain = format!("gui/{}", unsafe { libc::getuid() });
            let _ = run("launchctl", &["bootout", &format!("{domain}/{LABEL}")]);
            run(
                "launchctl",
                &["bootstrap", &domain, &path.display().to_string()],
            )?;
            Ok(path.display().to_string())
        } else {
            let path = PathBuf::from(&home)
                .join(".config/systemd/user")
                .join(format!("{LABEL}.service"));
            let env = env_pairs()
                .into_iter()
                .map(|(k, v)| format!("Environment=\"{k}={v}\"\n"))
                .collect::<String>();
            let unit = format!(
                "[Unit]\nDescription=Pont Bridget vers t3code\n\n[Service]\n\
ExecStart={} t3 serve\nRestart=always\nRestartSec=5\n{env}\n[Install]\nWantedBy=default.target\n",
                bridget.display()
            );
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(&path, unit).map_err(|e| format!("{} : {e}", path.display()))?;
            run("systemctl", &["--user", "daemon-reload"])?;
            run(
                "systemctl",
                &["--user", "enable", "--now", &format!("{LABEL}.service")],
            )?;
            Ok(path.display().to_string())
        }
    }

    pub(super) fn uninstall(path: &Path) -> Result<(), String> {
        if cfg!(target_os = "macos") {
            let domain = format!("gui/{}", unsafe { libc::getuid() });
            let _ = run("launchctl", &["bootout", &format!("{domain}/{LABEL}")]);
        } else {
            let _ = run(
                "systemctl",
                &["--user", "disable", "--now", &format!("{LABEL}.service")],
            );
        }
        match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(format!("{} : {e}", path.display())),
        }
    }

    fn xml(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }
}

// ---------------------------------------------------------------------------
// serve : boucle principale
// ---------------------------------------------------------------------------

/// Session t3 partagée par la boucle principale et tous les liens : un 401
/// déclenche UN renouvellement (le premier appelant le fait, les autres le
/// voient par la génération), puis l'appel est rejoué une fois.
struct Session {
    token_path: PathBuf,
    inner: Mutex<SessionInner>,
}

struct SessionInner {
    runtime: ServerRuntime,
    token: TokenFile,
    client: Client,
    generation: u64,
    /// Posé quand une session fraîchement émise est refusée à son tour :
    /// sans lui, chaque incident rouvrirait le droit de renouveler et le pont
    /// émettrait une session administrative par minute.
    auth_failed: bool,
}

type SharedSession = Arc<Session>;

impl Session {
    fn new(paths: &Paths, runtime: ServerRuntime, token: TokenFile) -> Self {
        let client = Client::new(&runtime, &token.token);
        Self {
            token_path: paths.token(),
            inner: Mutex::new(SessionInner {
                runtime,
                token,
                client,
                generation: 0,
                auth_failed: false,
            }),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, SessionInner> {
        self.inner.lock().unwrap_or_else(|p| p.into_inner())
    }

    fn base_url(&self) -> String {
        self.lock().runtime.base_url()
    }

    fn label(&self) -> String {
        self.lock().token.label.clone()
    }

    fn reset_runtime(&self, runtime: ServerRuntime) {
        let mut inner = self.lock();
        inner.client = Client::new(&runtime, &inner.token.token);
        inner.runtime = runtime;
    }

    /// Exécute `call` ; sur 401, renouvelle une fois puis rejoue. Un second
    /// 401 remonte tel quel : c'est l'échec explicite attendu par l'appelant.
    fn call<T>(
        &self,
        call: impl Fn(&Client) -> Result<T, ContractError>,
    ) -> Result<T, ContractError> {
        let (generation, first) = {
            let inner = self.lock();
            (inner.generation, call(&inner.client))
        };
        match first {
            Err(ContractError::Unauthorized) => {
                self.renew(generation)?;
                let retried = {
                    let inner = self.lock();
                    call(&inner.client)
                };
                if matches!(retried, Err(ContractError::Unauthorized)) {
                    // Une session neuve refusée : plus aucun renouvellement
                    // jusqu'à réinstallation, sinon le pont boucle.
                    self.lock().auth_failed = true;
                }
                retried
            }
            other => other,
        }
    }

    fn renew(&self, seen_generation: u64) -> Result<(), ContractError> {
        let mut inner = self.lock();
        if inner.auth_failed {
            return Err(ContractError::Unauthorized);
        }
        if inner.generation != seen_generation {
            return Ok(()); // un autre appelant vient de renouveler
        }
        let cli = T3Cli::from_env();
        let issued = cli.issue_session(&inner.token.label, TOKEN_TTL)?;
        let token = TokenFile {
            session_id: issued.session_id,
            token: issued.token,
            expires_at: issued.expires_at,
            issued_at: contract::iso_now(),
            ..inner.token.clone()
        };
        if let Err(detail) =
            private_write(&self.token_path, &serde_json::to_string(&token).unwrap())
        {
            // Session émise mais non conservée : la révoquer tout de suite,
            // sinon elle survit sans que rien ne sache la retirer.
            let _ = cli.revoke_session(&token.session_id);
            return Err(ContractError::Cli(detail));
        }
        if let Err(detail) = cli.revoke_session(&inner.token.session_id) {
            // L'ancienne session survit : sa trace est conservée pour que
            // `uninstall` la retire, plutôt qu'oubliée dans un log.
            warn!("ancienne session non révoquée ({detail}) ; consignée pour le retrait");
            note_orphan(&self.token_path, &inner.token.session_id);
        }
        info!("session t3 renouvelée (expire {})", token.expires_at);
        inner.client = Client::new(&inner.runtime, &token.token);
        inner.token = token;
        inner.generation += 1;
        Ok(())
    }
}

fn poll_interval() -> Duration {
    std::env::var("BRIDGET_T3_POLL_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(Duration::from_millis)
        .unwrap_or(DEFAULT_POLL)
}

fn turn_wait() -> Duration {
    std::env::var("BRIDGET_T3_TURN_WAIT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(Duration::from_secs)
        .unwrap_or(DEFAULT_TURN_WAIT)
}

fn publish_status(paths: &Paths, state: &str, detail: &str, threads: usize) {
    let status = StatusFile {
        state: state.to_string(),
        detail: detail.to_string(),
        updated_at: contract::iso_now(),
        threads,
    };
    if let Err(e) = private_write(&paths.status(), &serde_json::to_string(&status).unwrap()) {
        warn!("statut du pont non publié : {e}");
    }
}

/// Attend le serveur t3code : fichier d'état valide, PID vivant, boucle locale.
fn wait_runtime(paths: &Paths, poll: Duration) -> ServerRuntime {
    let base = loop {
        match contract::base_dir() {
            Ok(base) => break base,
            Err(e) => {
                publish_status(paths, "waiting", &e.to_string(), 0);
                thread::sleep(poll.max(Duration::from_secs(1)));
            }
        }
    };
    let mut announced = String::new();
    loop {
        match contract::read_runtime(&base) {
            Ok(runtime) => return runtime,
            Err(e) => {
                let detail = e.to_string();
                if detail != announced {
                    info!("serveur t3code indisponible : {detail}");
                    publish_status(paths, "waiting", &detail, 0);
                    announced = detail;
                }
                thread::sleep(poll.max(Duration::from_secs(1)));
            }
        }
    }
}

fn serve(paths: &Paths) -> Result<(), String> {
    paths.ensure()?;
    let Some(token) = read_json::<TokenFile>(&paths.token())? else {
        return Err("non installé : lancer `bridget t3 install` d'abord".to_string());
    };
    let poll = poll_interval();
    let runtime = wait_runtime(paths, poll);
    let session: SharedSession = Arc::new(Session::new(paths, runtime, token));
    info!(
        "pont t3code actif sur {} (session « {} »)",
        session.base_url(),
        session.label()
    );

    let mut links: BTreeMap<String, Link> = BTreeMap::new();
    let mut retry_after: BTreeMap<String, Instant> = BTreeMap::new();
    loop {
        let snapshot = match session.call(|client| client.snapshot()) {
            Ok(snapshot) => snapshot,
            Err(ContractError::Unauthorized) => {
                let detail = "session refusée deux fois par t3code ; réinstaller (bridget t3 uninstall && bridget t3 install)";
                error!("{detail}");
                publish_status(paths, "auth_failed", detail, links.len());
                thread::sleep(AUTH_FAILED_PAUSE);
                continue;
            }
            Err(ContractError::Cli(detail)) => {
                error!("renouvellement impossible : {detail}");
                publish_status(paths, "auth_failed", &detail, links.len());
                thread::sleep(AUTH_FAILED_PAUSE);
                continue;
            }
            Err(ContractError::Transport(detail)) => {
                // Serveur parti : on ferme les liens et on attend son retour.
                warn!("t3code injoignable ({detail}) ; liens fermés");
                for (_, link) in std::mem::take(&mut links) {
                    link.close();
                }
                publish_status(paths, "waiting", &detail, 0);
                session.reset_runtime(wait_runtime(paths, poll));
                continue;
            }
            Err(e) => {
                warn!("snapshot t3code : {e}");
                publish_status(paths, "degraded", &e.to_string(), links.len());
                thread::sleep(poll);
                continue;
            }
        };

        // Liens morts (daemon parti, panique) : retirés, recréés après un délai.
        links.retain(|thread_id, link| {
            if link.is_alive() {
                true
            } else {
                warn!("lien du fil {thread_id} terminé ; nouvelle tentative dans {RECONNECT_BACKOFF:?}");
                retry_after.insert(thread_id.clone(), Instant::now() + RECONNECT_BACKOFF);
                false
            }
        });

        // O(fils) par tour de boucle : un fil = un lien, retiré à l'archivage.
        // Un fil neuf n'a pas encore de session fournisseur (elle naît au
        // premier tour) : l'exiger le rendrait injoignable pour son premier
        // message. Seul un fournisseur identifiable est requis.
        let live: HashSet<&str> = snapshot
            .threads
            .iter()
            .filter(|t| t.is_live() && thread_provider(t).is_some())
            .map(|t| t.id.as_str())
            .collect();
        for thread_id in links.keys().cloned().collect::<Vec<_>>() {
            if !live.contains(thread_id.as_str())
                && let Some(link) = links.remove(&thread_id)
            {
                info!("fil {thread_id} archivé ou sans fournisseur : agent retiré");
                link.close();
            }
        }
        for summary in snapshot
            .threads
            .iter()
            .filter(|t| live.contains(t.id.as_str()))
        {
            if !links.contains_key(&summary.id) {
                if retry_after
                    .get(&summary.id)
                    .is_some_and(|until| Instant::now() < *until)
                {
                    continue;
                }
                match Link::open(paths, session.clone(), summary) {
                    Ok(link) => {
                        links.insert(summary.id.clone(), link);
                    }
                    Err(e) => {
                        warn!("fil {} non exposé : {e}", summary.id);
                        retry_after.insert(summary.id.clone(), Instant::now() + RECONNECT_BACKOFF);
                        continue;
                    }
                }
            }
            if let Some(link) = links.get(&summary.id) {
                link.tick(summary.clone());
            }
        }
        publish_status(paths, "running", &session.base_url(), links.len());
        thread::sleep(poll);
    }
}

// ---------------------------------------------------------------------------
// Lien : un fil t3code = une connexion au daemon
// ---------------------------------------------------------------------------

enum LinkEvent {
    Frame(Box<DaemonToWrapper>),
    Tick(Box<ThreadSummary>),
    DaemonGone,
    Close,
}

struct Link {
    events: Sender<LinkEvent>,
    worker: Option<thread::JoinHandle<()>>,
}

impl Link {
    fn open(
        paths: &Paths,
        session: SharedSession,
        summary: &ThreadSummary,
    ) -> Result<Self, String> {
        let agent_type = thread_provider(summary).ok_or("fil sans fournisseur")?;
        let agent_id = stable_uuid(&summary.id);
        let instance_id = stable_uuid(&format!("instance:{}", summary.id));
        let domain = summary
            .worktree_path
            .as_deref()
            .and_then(|p| Path::new(p).file_name())
            .map(|n| n.to_string_lossy().to_string());
        // La localisation n'est retenue par le daemon que si tmux l'atteste ;
        // le titre du fil reste visible par `bridget t3 status`.
        let (reader, writer, name) = connect_and_register_at(
            &paths.socket,
            &agent_type,
            Some(&agent_id),
            &crate::wrapper::host_name(),
            PROTOCOL,
            None,
            PresenceMode::Cli,
            None,
            &crate::wrapper::operating_system(),
            &instance_id,
            domain.as_deref(),
            summary
                .session
                .as_ref()
                .is_some_and(|session| session.active_turn_id.is_some()),
        )?;
        info!(
            "fil « {} » exposé comme « {name} » ({agent_type}, {agent_id})",
            summary.title
        );
        // La connexion naît non bloquante avec un délai de lecture d'une
        // seconde (poll côté wrapper) ; le lien lit dans un fil dédié et
        // préfère des lectures bloquantes sans délai.
        let stream = reader.get_ref();
        stream
            .set_nonblocking(false)
            .and_then(|_| stream.set_read_timeout(None))
            .map_err(|e| format!("socket du lien : {e}"))?;
        let stream = reader.get_ref().try_clone().map_err(|e| e.to_string())?;
        let (events, inbox) = mpsc::channel();
        spawn_reader(reader, events.clone());
        let worker = LinkWorker::new(
            paths,
            session,
            summary,
            agent_id,
            instance_id,
            writer,
            stream,
        )?;
        let handle = thread::Builder::new()
            .name(format!("t3-{}", &summary.id[..summary.id.len().min(8)]))
            .spawn(move || worker.run(inbox))
            .map_err(|e| e.to_string())?;
        Ok(Self {
            events,
            worker: Some(handle),
        })
    }

    fn is_alive(&self) -> bool {
        self.worker.as_ref().is_some_and(|h| !h.is_finished())
    }

    fn tick(&self, summary: ThreadSummary) {
        let _ = self.events.send(LinkEvent::Tick(Box::new(summary)));
    }

    fn close(mut self) {
        let _ = self.events.send(LinkEvent::Close);
        if let Some(handle) = self.worker.take() {
            let _ = handle.join();
        }
    }
}

fn spawn_reader(mut reader: BufReader<UnixStream>, events: Sender<LinkEvent>) {
    thread::spawn(move || {
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    match decode::<DaemonToWrapper>(trimmed) {
                        Ok(frame) => {
                            if events.send(LinkEvent::Frame(Box::new(frame))).is_err() {
                                break;
                            }
                        }
                        Err(e) => warn!("trame daemon illisible : {e}"),
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
        let _ = events.send(LinkEvent::DaemonGone);
    });
}

struct LinkWorker {
    thread_id: String,
    agent_id: String,
    session: SharedSession,
    writer: Arc<Mutex<Option<BufWriter<UnixStream>>>>,
    stream: UnixStream,
    tracker: Option<IdempotentDeliveryTracker>,
    journal: Arc<JournalWriter>,
    relay: AttachRelayWorker,
    state_path: PathBuf,
    state: ThreadState,
    /// Dernier titre publié comme nom humain. t3code régénère les titres et
    /// l'humain les change : un nom figé à la connexion vieillirait aussitôt.
    title: String,
    last_key: String,
    turn_wait: Duration,
}

impl LinkWorker {
    #[allow(clippy::too_many_arguments)]
    fn new(
        paths: &Paths,
        session: SharedSession,
        summary: &ThreadSummary,
        agent_id: String,
        instance_id: String,
        writer: BufWriter<UnixStream>,
        stream: UnixStream,
    ) -> Result<Self, String> {
        let tracker = IdempotentDeliveryTracker::open_at(&paths.root.join("state"), &instance_id)?;
        let journal_root = paths.root.join("sessions");
        crate::environment::ensure_private_directory(&journal_root)?;
        let live_feed = JournalLiveFeed::default();
        let journal = Arc::new(
            JournalWriter::start_with_live_feed_and_failure(
                &journal_root,
                &agent_id,
                &instance_id,
                Arc::new(|detail| warn!("journal t3code en échec : {detail}")),
                Some(live_feed.clone()),
            )
            .map_err(|e| format!("journal du fil : {e}"))?,
        );
        let writer = Arc::new(Mutex::new(Some(writer)));
        send_wrapper_message(&writer, WrapperToDaemon::JournalReady);
        // Nom humain = titre du fil, pour que `who` et `send --to` parlent
        // la langue de t3code ; un refus (doublon) garde le nom attribué.
        let title = display_title(&summary.title);
        publish_title(&writer, &title);
        let relay_writer = writer.clone();
        let relay = AttachRelayWorker::start(
            journal_root.join(&agent_id),
            live_feed,
            Arc::new(move |message| send_wrapper_message(&relay_writer, message)),
        );
        let state_path = paths.thread_state(&summary.id);
        let state = ThreadState::load(&state_path);
        Ok(Self {
            thread_id: summary.id.clone(),
            agent_id,
            session,
            writer,
            stream,
            tracker: Some(tracker),
            journal,
            relay,
            state_path,
            state,
            title,
            last_key: String::new(),
            turn_wait: turn_wait(),
        })
    }

    fn run(mut self, inbox: Receiver<LinkEvent>) {
        loop {
            match inbox.recv_timeout(Duration::from_secs(30)) {
                Ok(LinkEvent::Tick(summary)) => {
                    send_wrapper_message(&self.writer, WrapperToDaemon::Heartbeat);
                    let title = display_title(&summary.title);
                    if title != self.title {
                        info!(
                            "fil {} renommé « {} » → « {title} »",
                            self.thread_id, self.title
                        );
                        publish_title(&self.writer, &title);
                        self.title = title;
                    }
                    let key = summary.change_key();
                    if key != self.last_key || !self.state.pending.is_empty() {
                        self.last_key = key;
                        self.refresh(&summary);
                    }
                }
                Ok(LinkEvent::Frame(frame)) => self.handle_frame(*frame),
                Ok(LinkEvent::DaemonGone) => {
                    warn!("daemon parti pour le fil {}", self.thread_id);
                    break;
                }
                Ok(LinkEvent::Close) => {
                    send_wrapper_message(&self.writer, WrapperToDaemon::Unregister);
                    break;
                }
                Err(RecvTimeoutError::Timeout) => {
                    send_wrapper_message(&self.writer, WrapperToDaemon::Heartbeat);
                }
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
        self.relay.shutdown();
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }

    fn handle_frame(&mut self, frame: DaemonToWrapper) {
        match frame {
            DaemonToWrapper::Deliver(message) => {
                record_interactive_turn(self.journal.as_ref(), &message);
                if let Err(e) = self.dispatch(&message) {
                    error!("remise {} au fil {} : {e}", message.id, self.thread_id);
                }
            }
            DaemonToWrapper::DeliverIdempotent {
                delivery_id,
                recipient_instance_id,
                delivery_generation,
                expires_at,
                message,
                ..
            } => {
                record_interactive_turn(self.journal.as_ref(), &message);
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or_default();
                // Le tracker déduplique les rejeux ; l'ack ne part qu'après
                // acceptation de la commande par t3code (dispatch 200).
                let mut outcome = Ok(());
                let mut tracker = self.tracker.take().expect("tracker présent");
                let reports = deliver_idempotent_to_interactive(
                    &mut tracker,
                    delivery_id,
                    recipient_instance_id,
                    delivery_generation,
                    expires_at,
                    message,
                    now,
                    |message| {
                        outcome = self.dispatch_with_id(message, Some(&message.id));
                        outcome.clone()
                    },
                );
                self.tracker = Some(tracker);
                if let Err(e) = outcome {
                    error!("remise idempotente au fil {} : {e}", self.thread_id);
                }
                for report in reports {
                    send_wrapper_message(&self.writer, report);
                }
            }
            DaemonToWrapper::Subscribe {
                subscription_id,
                window,
                ..
            } => {
                if let Err(reason) = self.relay.subscribe(subscription_id.clone(), window) {
                    send_wrapper_message(
                        &self.writer,
                        WrapperToDaemon::AttachRejected {
                            subscription_id: Some(subscription_id),
                            reason,
                        },
                    );
                }
            }
            DaemonToWrapper::Unsubscribe { subscription_id } => {
                self.relay.unsubscribe(subscription_id);
            }
            DaemonToWrapper::DisplayNameResult { outcome } => {
                use bridget_transport::protocol::DisplayNameOutcome;
                match outcome {
                    DisplayNameOutcome::Applied { display_name, .. } => {
                        info!("fil {} nommé « {display_name} »", self.thread_id)
                    }
                    DisplayNameOutcome::Rejected { reason } => {
                        warn!("nom du fil {} refusé : {reason:?}", self.thread_id)
                    }
                }
            }
            DaemonToWrapper::Disconnect => {
                warn!("daemon a demandé la déconnexion du fil {}", self.thread_id);
            }
            _ => {}
        }
    }

    fn dispatch(&mut self, message: &bridget_core::BridgetMessage) -> Result<(), String> {
        self.dispatch_with_id(message, None)
    }

    /// Attend un fil sans tour actif (borné), puis `thread.turn.start`.
    /// `commandId` = identifiant du message Bridget (t3code déduplique dessus).
    fn dispatch_with_id(
        &mut self,
        message: &bridget_core::BridgetMessage,
        command_id: Option<&str>,
    ) -> Result<(), String> {
        let (anchor, runtime_mode, interaction_mode) = self.wait_idle()?;
        let command_id = command_id.unwrap_or(&message.id).to_string();
        // Identifiant de message déterministe : un rejeu après panne reconstruit
        // exactement la même commande, que t3code déduplique par `commandId`.
        let message_id = stable_uuid(&format!("t3code-message:{command_id}"));
        let command = contract::turn_start_command(
            &self.thread_id,
            &command_id,
            &message_id,
            &envelope(message),
            &runtime_mode,
            &interaction_mode,
        );
        // L'attente est écrite AVANT l'appel : si le pont meurt entre le 200 de
        // t3code et cette ligne, la corrélation serait perdue et la réponse ne
        // reviendrait jamais. Écrite avant, elle est au pire inutile — et alors
        // retirée faute de trouver son message dans le fil.
        if !self
            .state
            .pending
            .iter()
            .any(|p| p.request_id == message.id)
        {
            self.state.pending.push(Pending {
                request_id: message.id.clone(),
                from: message.from.clone(),
                message_id,
                anchor_turn_id: anchor,
                dispatched_at: contract::iso_now(),
                attempts: 0,
            });
            self.state.save(&self.state_path)?;
        }
        match self.session.call(|client| client.dispatch(&command)) {
            Ok(result) => {
                info!(
                    "message {} remis au fil {} (séquence {})",
                    message.id, self.thread_id, result.sequence
                );
                Ok(())
            }
            // Refus de forme : le tour n'existera jamais, l'attente est retirée.
            // Une panne réseau, elle, laisse l'attente : le tour a peut-être
            // démarré et sa réponse doit encore pouvoir revenir.
            Err(error @ (ContractError::Http { .. } | ContractError::InvalidShape { .. })) => {
                self.state.pending.retain(|p| p.request_id != message.id);
                let _ = self.state.save(&self.state_path);
                Err(error.to_string())
            }
            Err(error) => Err(error.to_string()),
        }
    }

    /// Attend un fil libre et rend, lus au même instant : l'ancre (dernier
    /// tour clos) et la politique du fil. La politique n'est pas mise en cache :
    /// entre deux sondages l'humain peut la changer, et la valeur périmée
    /// serait refusée par t3code — ou, pire, figerait des droits.
    fn wait_idle(&self) -> Result<(Option<String>, String, String), String> {
        let deadline = Instant::now() + self.turn_wait;
        loop {
            let snapshot = self
                .session
                .call(|client| client.snapshot())
                .map_err(|e| e.to_string())?;
            let Some(thread) = snapshot.threads.iter().find(|t| t.id == self.thread_id) else {
                return Err("fil disparu du snapshot".to_string());
            };
            if !thread.is_live() {
                return Err("fil archivé".to_string());
            }
            let busy = thread
                .session
                .as_ref()
                .is_some_and(|s| s.active_turn_id.is_some());
            if !busy {
                return Ok((
                    thread.latest_turn.as_ref().map(|t| t.turn_id.clone()),
                    thread.runtime_mode.clone(),
                    thread.interaction_mode.clone(),
                ));
            }
            if Instant::now() >= deadline {
                return Err(format!(
                    "fil occupé depuis plus de {:?} ; remise refusée",
                    self.turn_wait
                ));
            }
            thread::sleep(Duration::from_millis(500));
        }
    }

    /// Relit le fil (page bornée, élargie si l'ancre manque), projette le
    /// journal et clôt les remises dont la réponse est complète.
    fn refresh(&mut self, summary: &ThreadSummary) {
        let mut limit = DETAIL_PAGE;
        let detail = loop {
            let fetched = self
                .session
                .call(|client| client.thread_detail(&self.thread_id, limit));
            let detail = match fetched {
                Ok(detail) => detail,
                Err(e) => {
                    warn!("détail du fil {} : {e}", self.thread_id);
                    return;
                }
            };
            let anchors_missing = self.state.pending.iter().any(|p| {
                p.anchor_turn_id.as_ref().is_some_and(|anchor| {
                    !detail
                        .messages
                        .iter()
                        .any(|m| m.turn_id.as_deref() == Some(anchor))
                })
            });
            let seeded = self.state.seeded;
            if (anchors_missing || !seeded) && limit < DETAIL_PAGE_MAX {
                limit *= 2;
                continue;
            }
            break detail;
        };
        self.project_journal(&detail, summary);
        self.settle_pending(&detail, summary);
        if let Err(e) = self.state.save(&self.state_path) {
            warn!("état du fil {} : {e}", self.thread_id);
        }
    }

    /// Repère d'installation : au premier regard, l'historique antérieur est
    /// mémorisé sans être rejoué. Ensuite, chaque message nouveau est projeté.
    fn project_journal(&mut self, detail: &ThreadDetail, summary: &ThreadSummary) {
        let seen: HashSet<&str> = self.state.seen.iter().map(String::as_str).collect();
        let mut new_seen = Vec::new();
        let mut ended = Vec::new();
        if !self.state.seeded {
            new_seen.extend(detail.messages.iter().map(|m| m.id.clone()));
            ended.extend(detail.messages.iter().filter_map(|m| m.turn_id.clone()));
            self.state.seeded = true;
        } else {
            let final_turn = |turn_id: &str| {
                summary
                    .latest_turn
                    .as_ref()
                    .is_none_or(|latest| latest.turn_id != turn_id || turn_is_final(&latest.state))
            };
            let mine: HashSet<&str> = self
                .state
                .pending
                .iter()
                .map(|p| p.message_id.as_str())
                .collect();
            let ended_turns: HashSet<&str> =
                self.state.ended_turns.iter().map(String::as_str).collect();
            for message in &detail.messages {
                if seen.contains(message.id.as_str()) {
                    continue;
                }
                let text = message.text.chars().take(4096).collect::<String>();
                let write = match message.role.as_str() {
                    "user" => {
                        // Nos propres remises sont déjà journalisées par record_interactive_turn.
                        if mine.contains(message.id.as_str()) {
                            new_seen.push(message.id.clone());
                            continue;
                        }
                        self.journal.enqueue(
                            "turn_start",
                            Some(&message.id),
                            serde_json::json!({"from": "human", "body": text}),
                        )
                    }
                    _ => {
                        if message.streaming {
                            continue;
                        }
                        self.journal.enqueue(
                            "update",
                            Some(&message.id),
                            serde_json::json!({"kind": "text", "text": text}),
                        )
                    }
                };
                if let Err(e) = write {
                    warn!("journal du fil {} : {e}", self.thread_id);
                }
                new_seen.push(message.id.clone());
                if let Some(turn_id) = message.turn_id.as_deref()
                    && !ended_turns.contains(turn_id)
                    && !ended.iter().any(|t| t == turn_id)
                    && final_turn(turn_id)
                {
                    let _ =
                        self.journal
                            .enqueue("turn_end", Some(&message.id), serde_json::json!({}));
                    ended.push(turn_id.to_string());
                }
            }
        }
        for id in new_seen {
            self.state.remember(&id);
        }
        self.state.ended_turns.extend(ended);
        if self.state.ended_turns.len() > SEEN_BOUND {
            let excess = self.state.ended_turns.len() - SEEN_BOUND;
            self.state.ended_turns.drain(..excess);
        }
    }

    fn settle_pending(&mut self, detail: &ThreadDetail, summary: &ThreadSummary) {
        for mut pending in std::mem::take(&mut self.state.pending) {
            let verdict = correlate(detail, summary, &pending);
            let keep = match verdict {
                Correlation::Answered(text) => {
                    let mut reply = bridget_core::BridgetMessage::new(
                        self.agent_id.clone(),
                        pending.from.clone(),
                        text,
                    );
                    // Identifiant stable : si le pont meurt entre l'envoi et la
                    // sauvegarde, le rejeu porte le même identifiant et ne crée
                    // pas une seconde réponse.
                    reply.id = reply_id(&pending.request_id);
                    reply.in_reply_to = Some(pending.request_id.clone());
                    info!(
                        "réponse du fil {} renvoyée à {} (demande {})",
                        self.thread_id, pending.from, pending.request_id
                    );
                    send_wrapper_message(&self.writer, WrapperToDaemon::Send(reply));
                    false
                }
                Correlation::Waiting => true,
                Correlation::Ambiguous | Correlation::Missing => {
                    pending.attempts += 1;
                    if pending.attempts >= SETTLE_ATTEMPTS {
                        let reason = match verdict {
                            Correlation::Ambiguous => format!(
                                "fil {} au repos sans appariement certain entre messages et tours après {} lectures ; demande laissée sans réponse plutôt qu'attribuée au tour d'autrui",
                                self.thread_id, pending.attempts
                            ),
                            _ => format!(
                                "message {} introuvable dans le fil t3code après {} lectures ; réponse abandonnée",
                                pending.message_id, pending.attempts
                            ),
                        };
                        warn!("{reason}");
                        let _ = self.journal.enqueue(
                            "error",
                            Some(&pending.request_id),
                            serde_json::json!({"reason": reason}),
                        );
                        false
                    } else {
                        true
                    }
                }
            };
            if keep {
                self.state.pending.push(pending);
            }
            // Sauvegarde après CHAQUE décision : une panne ne peut pas rejouer
            // une réponse déjà émise ni perdre celles qui restent.
            if let Err(e) = self.state.save(&self.state_path) {
                warn!("état du fil {} : {e}", self.thread_id);
            }
        }
    }
}

/// Titre présentable : espaces normalisés et longueur bornée, le daemon
/// refusant les noms de contrôle ou démesurés.
pub(crate) fn display_title(title: &str) -> String {
    title
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(48)
        .collect()
}

/// Publie le nom humain du fil ; un titre vide laisse le nom attribué par le
/// daemon, un doublon est refusé par lui sans conséquence pour la remise.
fn publish_title(writer: &Arc<Mutex<Option<BufWriter<UnixStream>>>>, title: &str) {
    if title.is_empty() {
        return;
    }
    send_wrapper_message(
        writer,
        WrapperToDaemon::DisplayNameSet {
            request: bridget_transport::protocol::DisplayNameRequest {
                version: 1,
                display_name: title.to_string(),
            },
        },
    );
}

/// Identifiant déterministe de la réponse à une demande : deux envois issus
/// du même `request_id` portent le même identifiant, jamais deux réponses.
pub(crate) fn reply_id(request_id: &str) -> String {
    format!("t3-{}", stable_uuid(&format!("t3code-reply:{request_id}")))
}

/// Texte remis au fil : l'agent t3code répond normalement, le pont relaie.
pub(crate) fn envelope(message: &bridget_core::BridgetMessage) -> String {
    format!(
        "💬 Message Bridget de {} (id {}) :\n\n{}\n\n— Réponds normalement dans ce tour : Bridget transmettra ta réponse à {}.",
        message.from, message.id, message.body, message.from
    )
}

pub(crate) fn turn_is_final(state: &str) -> bool {
    !matches!(
        state,
        "running" | "queued" | "pending" | "streaming" | "in_progress" | "starting"
    )
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Correlation {
    /// Le tour de notre message est clos : voici son texte.
    Answered(String),
    /// L'appariement n'est pas encore décidable ; le fil travaille encore.
    Waiting,
    /// L'appariement ne sera plus décidable : le fil est au repos et les
    /// messages utilisateur et les tours ne se correspondent plus un pour un.
    Ambiguous,
    /// Notre message n'est pas (ou plus) dans la page lue.
    Missing,
}

/// Corrélation par rang, **et seulement quand le rang est prouvé**.
///
/// t3code ne lie pas un message utilisateur à un tour : les messages
/// utilisateur portent `turnId: null`. L'ordre observé (k-ième message
/// utilisateur ↔ k-ième tour assistant) ne vaut donc que si, après l'ancre,
/// il y a exactement autant de tours assistants que de messages utilisateur.
/// Dès que ce n'est plus le cas — un tour interrompu sans texte, un tour de
/// sous-agent sans message utilisateur — le rang ordinal désignerait le tour
/// d'un autre : on préfère rendre la demande ambiguë plutôt que d'attribuer
/// la réponse d'autrui. O(n) sur la page lue.
pub(crate) fn correlate(
    detail: &ThreadDetail,
    summary: &ThreadSummary,
    pending: &Pending,
) -> Correlation {
    let messages = &detail.messages;
    let start = match pending.anchor_turn_id.as_deref() {
        Some(anchor) => match messages
            .iter()
            .rposition(|m| m.turn_id.as_deref() == Some(anchor))
        {
            Some(index) => index + 1,
            None => 0,
        },
        None => 0,
    };
    let window = &messages[start.min(messages.len())..];
    let mut rank = None;
    let mut users = 0usize;
    for message in window {
        if message.role == "user" {
            users += 1;
            if message.id == pending.message_id {
                rank = Some(users);
            }
        }
    }
    let Some(rank) = rank else {
        return Correlation::Missing;
    };
    let mut turns: Vec<&str> = Vec::new();
    for message in window {
        if let Some(turn_id) = message.turn_id.as_deref()
            && message.role != "user"
            && !turns.contains(&turn_id)
        {
            turns.push(turn_id);
        }
    }
    // Le fil travaille-t-il encore ? Un tour ouvert peut encore rétablir
    // l'égalité ; un fil au repos ne le fera plus.
    let busy = summary
        .session
        .as_ref()
        .is_some_and(|session| session.active_turn_id.is_some())
        || summary
            .latest_turn
            .as_ref()
            .is_some_and(|latest| !turn_is_final(&latest.state))
        || detail
            .latest_turn
            .as_ref()
            .is_some_and(|latest| !turn_is_final(&latest.state));
    if users != turns.len() {
        return if busy {
            Correlation::Waiting
        } else {
            Correlation::Ambiguous
        };
    }
    let Some(turn_id) = turns.get(rank - 1).copied() else {
        return Correlation::Waiting;
    };
    let parts: Vec<&contract::Message> = window
        .iter()
        .filter(|m| m.role != "user" && m.turn_id.as_deref() == Some(turn_id))
        .collect();
    if parts.iter().any(|m| m.streaming) {
        return Correlation::Waiting;
    }
    let open = |latest: Option<&contract::LatestTurn>| {
        latest.is_some_and(|latest| latest.turn_id == turn_id && !turn_is_final(&latest.state))
    };
    if open(summary.latest_turn.as_ref()) || open(detail.latest_turn.as_ref()) {
        return Correlation::Waiting;
    }
    let text = parts
        .iter()
        .map(|m| m.text.trim())
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    if text.is_empty() {
        return if busy {
            Correlation::Waiting
        } else {
            Correlation::Ambiguous
        };
    }
    Correlation::Answered(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::t3code_contract::{LatestTurn, Message};

    fn message(id: &str, role: &str, text: &str, turn: Option<&str>, streaming: bool) -> Message {
        Message {
            id: id.to_string(),
            role: role.to_string(),
            text: text.to_string(),
            streaming,
            turn_id: turn.map(str::to_string),
        }
    }

    fn summary(latest: Option<(&str, &str)>) -> ThreadSummary {
        ThreadSummary {
            id: "thread-1".to_string(),
            project_id: "p".to_string(),
            title: "Fil".to_string(),
            provider_instance_id: None,
            runtime_mode: "approval-required".to_string(),
            interaction_mode: "default".to_string(),
            worktree_path: None,
            archived_at: None,
            deleted_at: None,
            updated_at: "2026-09-14T00:00:00Z".to_string(),
            latest_turn: latest.map(|(id, state)| LatestTurn {
                turn_id: id.to_string(),
                state: state.to_string(),
                assistant_message_id: None,
            }),
            session: None,
        }
    }

    fn pending(message_id: &str, anchor: Option<&str>) -> Pending {
        Pending {
            request_id: "req".to_string(),
            from: "alice".to_string(),
            message_id: message_id.to_string(),
            anchor_turn_id: anchor.map(str::to_string),
            dispatched_at: String::new(),
            attempts: 0,
        }
    }

    #[test]
    fn spec098_correlation_par_rang_fifo_avec_message_humain_intercale() {
        // Ordre observé en sonde : user A (nous), user B (humain), assistant A, assistant B.
        let detail = ThreadDetail {
            messages: vec![
                message("u0", "user", "avant", None, false),
                message("a0", "assistant", "réponse avant", Some("t0"), false),
                message("uA", "user", "nous", None, false),
                message("uB", "user", "humain", None, false),
                message("aA", "assistant", "pour nous", Some("tA"), false),
                message("aB", "assistant", "pour l'humain", Some("tB"), false),
            ],
            latest_turn: Some(LatestTurn {
                turn_id: "tB".to_string(),
                state: "completed".to_string(),
                assistant_message_id: None,
            }),
        };
        let s = summary(Some(("tB", "completed")));
        assert_eq!(
            correlate(&detail, &s, &pending("uA", Some("t0"))),
            Correlation::Answered("pour nous".to_string())
        );
        assert_eq!(
            correlate(&detail, &s, &pending("uB", Some("t0"))),
            Correlation::Answered("pour l'humain".to_string())
        );
    }

    #[test]
    fn spec098_correlation_attend_la_fin_du_tour() {
        let detail = ThreadDetail {
            messages: vec![
                message("uA", "user", "nous", None, false),
                message("aA", "assistant", "partiel", Some("tA"), true),
            ],
            latest_turn: Some(LatestTurn {
                turn_id: "tA".to_string(),
                state: "running".to_string(),
                assistant_message_id: None,
            }),
        };
        let s = summary(Some(("tA", "running")));
        assert_eq!(
            correlate(&detail, &s, &pending("uA", None)),
            Correlation::Waiting
        );
        let done = ThreadDetail {
            messages: vec![
                message("uA", "user", "nous", None, false),
                message("aA", "assistant", "complet", Some("tA"), false),
            ],
            latest_turn: Some(LatestTurn {
                turn_id: "tA".to_string(),
                state: "completed".to_string(),
                assistant_message_id: None,
            }),
        };
        assert_eq!(
            correlate(
                &done,
                &summary(Some(("tA", "completed"))),
                &pending("uA", None)
            ),
            Correlation::Answered("complet".to_string())
        );
    }

    #[test]
    fn spec098_correlation_sans_message_ni_tour() {
        let detail = ThreadDetail {
            messages: vec![message("uA", "user", "nous", None, false)],
            latest_turn: None,
        };
        // Fil au travail : le tour peut encore naître, on patiente.
        let mut occupe = summary(None);
        occupe.session = Some(crate::t3code_contract::SessionSummary {
            provider_name: "claudeAgent".to_string(),
            provider_instance_id: None,
            status: "running".to_string(),
            active_turn_id: Some("tA".to_string()),
        });
        assert_eq!(
            correlate(&detail, &occupe, &pending("uA", None)),
            Correlation::Waiting
        );
        // Fil au repos sans aucun tour : plus rien ne viendra.
        assert_eq!(
            correlate(&detail, &summary(None), &pending("uA", None)),
            Correlation::Ambiguous
        );
        assert_eq!(
            correlate(&detail, &summary(None), &pending("uZ", None)),
            Correlation::Missing
        );
    }

    #[test]
    fn spec098_tour_sans_reponse_n_attribue_pas_le_tour_du_voisin() {
        // Objection de contre-revue : notre tour est interrompu sans produire
        // de message assistant, puis l'humain obtient le sien. Un appariement
        // ordinal donnerait la réponse de l'humain à Bridget.
        let detail = ThreadDetail {
            messages: vec![
                message("uB", "user", "nous", None, false),
                message("uH", "user", "humain", None, false),
                message("aH", "assistant", "réponse à l'humain", Some("tH"), false),
            ],
            latest_turn: Some(LatestTurn {
                turn_id: "tH".to_string(),
                state: "completed".to_string(),
                assistant_message_id: None,
            }),
        };
        let s = summary(Some(("tH", "completed")));
        assert_eq!(
            correlate(&detail, &s, &pending("uB", None)),
            Correlation::Ambiguous,
            "la réponse de l'humain ne doit jamais être attribuée à Bridget"
        );
        // L'humain, lui, garde un appariement faux aussi : deux utilisateurs
        // pour un seul tour, aucun rang n'est prouvé.
        assert_eq!(
            correlate(&detail, &s, &pending("uH", None)),
            Correlation::Ambiguous
        );
    }

    #[test]
    fn spec098_tour_supplementaire_sans_message_utilisateur_est_ambigu() {
        // Fan-out : t3code peut ouvrir un tour sans message utilisateur associé.
        let detail = ThreadDetail {
            messages: vec![
                message("uA", "user", "nous", None, false),
                message("aX", "assistant", "sous-agent", Some("tX"), false),
                message("aA", "assistant", "réponse", Some("tA"), false),
            ],
            latest_turn: Some(LatestTurn {
                turn_id: "tA".to_string(),
                state: "completed".to_string(),
                assistant_message_id: None,
            }),
        };
        assert_eq!(
            correlate(
                &detail,
                &summary(Some(("tA", "completed"))),
                &pending("uA", None)
            ),
            Correlation::Ambiguous
        );
    }

    #[test]
    fn spec098_journal_du_service_reste_hors_du_namespace() {
        // launchd crée ses fichiers de sortie en 0644 ; un seul fichier non
        // privé dans le namespace empêche le daemon Bridget de démarrer.
        let home = "/Users/essai";
        let log = service::log_path(home);
        let namespace = crate::environment::root_for_home(std::path::Path::new(home))
            .expect("racine du namespace");
        assert!(
            !log.starts_with(&namespace),
            "journal {} sous le namespace {}",
            log.display(),
            namespace.display()
        );
    }

    #[test]
    fn spec098_identifiant_de_reponse_est_stable_par_demande() {
        // Un rejeu après panne ne doit pas produire une seconde réponse.
        assert_eq!(reply_id("demande-1"), reply_id("demande-1"));
        assert_ne!(reply_id("demande-1"), reply_id("demande-2"));
        assert!(reply_id("demande-1").starts_with("t3-"));
    }

    #[test]
    fn spec098_identite_stable_et_v4_canonique() {
        let a = stable_uuid("thread-1");
        assert_eq!(a, stable_uuid("thread-1"));
        assert_ne!(a, stable_uuid("thread-2"));
        assert!(bridget_core::router::validate_agent_id(&a).is_ok(), "{a}");
    }

    #[test]
    fn spec098_type_d_agent_depuis_le_fournisseur() {
        assert_eq!(agent_type_for("claudeAgent"), "claude");
        assert_eq!(agent_type_for("codex"), "codex");
        assert_eq!(agent_type_for("antigravity"), "gemini");
        assert_eq!(agent_type_for("Gemini"), "gemini");
        assert_eq!(agent_type_for("autre"), "autre");
    }

    #[test]
    fn spec098_fil_neuf_sans_session_reste_joignable_par_son_modele() {
        // Forme réelle d'un fil créé et jamais démarré (t3code 0.0.41) :
        // `session` est nul, seul `modelSelection.instanceId` nomme le fournisseur.
        let mut neuf = summary(None);
        neuf.provider_instance_id = Some("claudeAgent".to_string());
        assert_eq!(thread_provider(&neuf).as_deref(), Some("claude"));
        // La session, quand elle existe, prime : elle atteste ce qui tourne.
        let mut demarre = summary(None);
        demarre.provider_instance_id = Some("claudeAgent".to_string());
        demarre.session = Some(crate::t3code_contract::SessionSummary {
            provider_name: "codex".to_string(),
            provider_instance_id: Some("codex".to_string()),
            status: "stopped".to_string(),
            active_turn_id: None,
        });
        assert_eq!(thread_provider(&demarre).as_deref(), Some("codex"));
        // Sans l'un ni l'autre, le fil n'est pas exposé plutôt que mal typé.
        assert_eq!(thread_provider(&summary(None)), None);
    }

    #[test]
    fn spec098_etat_de_fil_borne_et_relu() {
        let dir = std::env::temp_dir().join(format!("t3code-098-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("répertoire de test");
        let path = dir.join("t.json");
        let mut state = ThreadState::default();
        for i in 0..(SEEN_BOUND + 10) {
            state.remember(&format!("m{i}"));
        }
        state.pending.push(pending("x", Some("t0")));
        state.save(&path).expect("save");
        let reloaded = ThreadState::load(&path);
        assert_eq!(reloaded.seen.len(), SEEN_BOUND);
        assert_eq!(reloaded.pending, state.pending);
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}
