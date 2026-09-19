//! Adaptateur t3code (session 098) : `bridget t3 install|status|uninstall|serve`.
//!
//! Le pont ne modifie jamais t3code. Il lit son serveur local par HTTP en
//! boucle locale avec une session émise par le CLI officiel `t3`, présente
//! chaque fil vivant au daemon Bridget comme un agent (`t3code | cli`), remet
//! les messages par `thread.turn.start` et renvoie à l'expéditeur la réponse
//! du tour corrélé par rang FIFO. Tout l'état est sous `<BRIDGET_HOME>/t3code`.

use std::collections::{BTreeMap, HashSet, VecDeque};
use std::io::{BufRead, BufReader, BufWriter};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use bridget_transport::journal::{
    IncrementalJournalReader, JournalLiveFeed, JournalReadItem, JournalWriter,
};
use bridget_transport::protocol::{
    DaemonToWrapper, PresenceMode, RequestInfo, WrapperToDaemon, decode,
};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};

use crate::t3code_contract::{
    self as contract, Client, ContractError, ServerRuntime, T3Cli, ThreadDetail, ThreadSummary,
};
use crate::wrapper::{
    AttachRelayWorker, IdempotentDeliveryTracker, connect_and_register_at,
    deliver_idempotent_to_interactive, send_wrapper_message,
};

const PROTOCOL: &str = "t3code";
const TOKEN_TTL: &str = "30d";
const DEFAULT_POLL: Duration = Duration::from_secs(3);
const DEFAULT_TURN_WAIT: Duration = Duration::from_secs(120);
/// Session 111 : borne mémoire de la file d'un fil. Elle remplace l'ancien
/// garde-fou temporel, qui jetait une remise valide au seul motif que le
/// destinataire travaillait depuis plus de deux minutes. Une saturation est un
/// fait distinct d'une péremption : elle a son propre avertissement.
const QUEUE_BOUND: usize = 64;
const RECONNECT_BACKOFF: Duration = Duration::from_secs(5);
const AUTH_FAILED_PAUSE: Duration = Duration::from_secs(60);
const DETAIL_PAGE: u32 = 20;
const DETAIL_PAGE_MAX: u32 = 320;
const SEEN_BOUND: usize = 1000;
/// Lectures successives avant de signaler une corrélation incertaine. L'attente
/// reste conservée : ce seuil ne prouve ni l'absence du tour ni sa clôture.
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
    /// Réponse préparée : conservée avant envoi, supprimée seulement sur preuve.
    #[serde(default)]
    pub response: Option<String>,
    /// None : ancien état sans contrat ; attendre la preuve du daemon.
    #[serde(default)]
    pub reply_requested: Option<bool>,
}

impl Pending {
    fn needs_correlation(&self) -> bool {
        self.reply_requested == Some(true) && self.response.is_none()
    }
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
    #[serde(default)]
    pub observations_seeded: bool,
    #[serde(default)]
    pub seen_activities: Vec<String>,
    /// IDs natifs issus des notifications, pas de recherche dans leur texte.
    #[serde(default)]
    pub notification_messages: Vec<String>,
    #[serde(default)]
    pub turn_origins: BTreeMap<String, String>,
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
    bridget_transport::fsutil::write_private_file_atomic(path, text.as_bytes())
        .map_err(|e| format!("{} : {e}", path.display()))
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
        // Sans niveau de journal, le service tourne muet et une panne du pont
        // ne laisse aucune trace exploitable.
        let mut pairs = vec![(
            "RUST_LOG".to_string(),
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        )];
        pairs.extend(
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
            }),
        );
        pairs
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

/// Délai entre deux essais d'un nom refusé (nom déjà pris). Le daemon libère
/// les présences au bout de cinq minutes ; on retente un peu au-delà.
fn rename_retry() -> Duration {
    std::env::var("BRIDGET_T3_RENAME_RETRY_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(330))
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
    let t3_base = contract::base_dir().map_err(|e| e.to_string())?;
    let mut identities = crate::t3code_identity::IdentityBindings::new(&paths.root);
    loop {
        let snapshot = match session.call(|client| client.snapshot()) {
            Ok(snapshot) => snapshot,
            Err(ContractError::Unauthorized) => {
                identities.clear();
                for link in links.values() {
                    let _ = link.events.send(LinkEvent::Unavailable);
                }
                let detail = "session refusée deux fois par t3code ; réinstaller (bridget t3 uninstall && bridget t3 install)";
                error!("{detail}");
                publish_status(paths, "auth_failed", detail, links.len());
                thread::sleep(AUTH_FAILED_PAUSE);
                continue;
            }
            Err(ContractError::Cli(detail)) => {
                identities.clear();
                for link in links.values() {
                    let _ = link.events.send(LinkEvent::Unavailable);
                }
                error!("renouvellement impossible : {detail}");
                publish_status(paths, "auth_failed", &detail, links.len());
                thread::sleep(AUTH_FAILED_PAUSE);
                continue;
            }
            Err(ContractError::Transport(detail)) => {
                identities.clear();
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
                identities.clear();
                for link in links.values() {
                    let _ = link.events.send(LinkEvent::Unavailable);
                }
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
            // Un fil sans worktree travaille dans la racine publiée du projet.
            // Nécessaire aux chemins relatifs des écritures, jamais à l'identité.
            let mut summary = summary.clone();
            if summary.worktree_path.is_none() {
                summary.worktree_path = snapshot
                    .projects
                    .iter()
                    .find(|project| project.id == summary.project_id)
                    .map(|project| project.workspace_root.clone());
            }
            if !links.contains_key(&summary.id) {
                if retry_after
                    .get(&summary.id)
                    .is_some_and(|until| Instant::now() < *until)
                {
                    continue;
                }
                match Link::open(paths, session.clone(), &summary) {
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
        let bindings: Vec<_> = links
            .iter()
            .filter(|(_, link)| link.is_alive())
            .map(|(id, _)| {
                (
                    id.clone(),
                    stable_uuid(id),
                    stable_uuid(&format!("instance:{id}")),
                )
            })
            .collect();
        let runtime = session.lock().runtime.clone();
        if let Err(error) = identities.refresh(&t3_base, &runtime, &bindings) {
            warn!("rattachement MCP T3 indisponible : {error}");
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
    Unavailable,
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
        let worker = LinkWorker::new(
            paths,
            session,
            summary,
            agent_id,
            instance_id,
            writer,
            stream,
        )?;
        spawn_reader(
            reader,
            events.clone(),
            worker.daemon_alive.clone(),
            worker.cancelled.clone(),
        );
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

fn spawn_reader(
    mut reader: BufReader<UnixStream>,
    events: Sender<LinkEvent>,
    alive: Arc<AtomicBool>,
    cancelled: Arc<Mutex<BTreeMap<String, Instant>>>,
) {
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
                            match &frame {
                                DaemonToWrapper::CancelDelivery { id, .. } => {
                                    cancelled
                                        .lock()
                                        .unwrap_or_else(|e| e.into_inner())
                                        .insert(id.clone(), Instant::now());
                                }
                                DaemonToWrapper::Disconnect => alive.store(false, Ordering::SeqCst),
                                _ => {}
                            }
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
        alive.store(false, Ordering::SeqCst);
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
    /// Titre refusé par le daemon (nom déjà pris) et date du refus : on
    /// retente à intervalle borné, car le nom peut se libérer plus tard.
    title_refused_at: Option<Instant>,
    last_key: String,
    turn_wait: Duration,
    queue: VecDeque<(DaemonToWrapper, Instant)>,
    requests: BTreeMap<String, RequestInfo>,
    daemon_alive: Arc<AtomicBool>,
    cancelled: Arc<Mutex<BTreeMap<String, Instant>>>,
    next_dispatch_check: Instant,
    journal_failed: Arc<AtomicBool>,
    journal_dir: PathBuf,
    journal_readers: BTreeMap<PathBuf, IncrementalJournalReader>,
    journal_inflight: HashSet<String>,
    journal_caught_up: bool,
    /// Une admission refusée doit être reprise même après vidage des inflight
    /// et sans changement du snapshot fournisseur.
    journal_dirty: bool,
    response_sent: BTreeMap<String, Instant>,
    observation_events: Vec<bridget_transport::protocol::ObservationKind>,
    observation_schema: Vec<bridget_transport::protocol::ObservationKind>,
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
        let journal_failed = Arc::new(AtomicBool::new(false));
        let failure_flag = journal_failed.clone();
        let journal = Arc::new(
            JournalWriter::start_with_live_feed_and_failure(
                &journal_root,
                &agent_id,
                &instance_id,
                Arc::new(move |detail| {
                    failure_flag.store(true, Ordering::SeqCst);
                    warn!("journal t3code en échec : {detail}");
                }),
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
            journal_dir: journal_root.join(&agent_id),
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
            title_refused_at: None,
            last_key: String::new(),
            turn_wait: turn_wait(),
            queue: VecDeque::new(),
            requests: BTreeMap::new(),
            daemon_alive: Arc::new(AtomicBool::new(true)),
            cancelled: Arc::new(Mutex::new(BTreeMap::new())),
            next_dispatch_check: Instant::now(),
            journal_failed,
            journal_readers: BTreeMap::new(),
            journal_inflight: HashSet::new(),
            journal_caught_up: false,
            journal_dirty: false,
            response_sent: BTreeMap::new(),
            observation_events: Vec::new(),
            observation_schema: Vec::new(),
        })
    }

    fn run(mut self, inbox: Receiver<LinkEvent>) {
        while self.daemon_alive.load(Ordering::SeqCst) {
            match inbox.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => self.handle_event(event),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
            self.drain_controls(&inbox);
            if !self.daemon_alive.load(Ordering::SeqCst) {
                break;
            }
            if !self.journal_caught_up || !self.journal_inflight.is_empty() {
                self.confirm_journal();
            }
            if self.journal_failed.load(Ordering::SeqCst) {
                // Le writer est arrêté après échec : rouvrir le lien recrée le
                // journal. Les messages non confirmés restent à reprendre.
                break;
            }
            if !self.queue.is_empty() && Instant::now() >= self.next_dispatch_check {
                self.next_dispatch_check = Instant::now() + Duration::from_millis(500);
                self.drive_queue(&inbox);
            }
        }
        self.relay.shutdown();
        self.journal.stop();
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }

    fn drain_controls(&mut self, inbox: &Receiver<LinkEvent>) {
        while let Ok(event) = inbox.try_recv() {
            self.handle_event(event);
        }
    }

    fn handle_event(&mut self, event: LinkEvent) {
        match event {
            LinkEvent::Tick(summary) => {
                send_wrapper_message(&self.writer, WrapperToDaemon::Heartbeat);
                let title = display_title(&summary.title);
                let retry = self
                    .title_refused_at
                    .is_some_and(|since| since.elapsed() >= rename_retry());
                if title != self.title || retry {
                    if title != self.title {
                        info!(
                            "fil {} renommé « {} » → « {title} »",
                            self.thread_id, self.title
                        );
                    }
                    publish_title(&self.writer, &title);
                    self.title = title;
                    // En attente du verdict : Applied efface, Rejected redate.
                    self.title_refused_at = Some(Instant::now());
                }
                let key = summary.change_key();
                if (key != self.last_key
                    || self.state.pending.iter().any(Pending::needs_correlation)
                    || !self.journal_inflight.is_empty()
                    || self.journal_dirty)
                    && self.refresh(&summary)
                {
                    self.last_key = key;
                }
                if !self.state.pending.is_empty() || !self.queue.is_empty() {
                    self.request_status();
                    self.send_ready_responses();
                }
            }
            LinkEvent::Frame(frame) => self.handle_frame(*frame),
            LinkEvent::Unavailable => {
                self.set_observation_ready(false);
                self.last_key.clear();
            }
            LinkEvent::DaemonGone => {
                warn!("daemon parti pour le fil {}", self.thread_id);
                self.daemon_alive.store(false, Ordering::SeqCst);
            }
            LinkEvent::Close => {
                send_wrapper_message(&self.writer, WrapperToDaemon::Unregister);
                self.daemon_alive.store(false, Ordering::SeqCst);
            }
        }
    }

    fn handle_frame(&mut self, frame: DaemonToWrapper) {
        match frame {
            delivery
            @ (DaemonToWrapper::Deliver(_) | DaemonToWrapper::DeliverIdempotent { .. }) => {
                if self.queue.len() >= QUEUE_BOUND
                    && let Some((oldest, _)) = self.queue.pop_front()
                {
                    self.report_discard(&oldest, Discard::Saturated);
                }
                self.queue.push_back((delivery, Instant::now()));
                self.request_status();
            }
            DaemonToWrapper::CancelDelivery { id, reason } => {
                self.cancelled
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .insert(id.clone(), Instant::now());
                self.discard_by_id(&id, Discard::RequestClosed("annulée"));
                info!("demande {id} retirée avant démarrage : {reason}");
            }
            DaemonToWrapper::Ack { id } => {
                self.response_sent
                    .retain(|request, _| reply_id(request) != id);
                self.state
                    .pending
                    .retain(|p| p.response.is_none() || reply_id(&p.request_id) != id);
                if let Err(e) = self.state.save(&self.state_path) {
                    warn!("confirmation réponse : {e}");
                }
            }
            DaemonToWrapper::Nack { id, reason } => {
                if self
                    .state
                    .pending
                    .iter()
                    .any(|p| reply_id(&p.request_id) == id)
                {
                    warn!("réponse {id} non remise, conservée : {reason}");
                }
            }
            DaemonToWrapper::RequestList { requests } => {
                // O(demandes + attentes) pour la migration ; une absence dans
                // cette liste bornée n'est jamais une preuve de clôture.
                let open: HashSet<(&str, &str)> = requests
                    .iter()
                    .filter(|r| {
                        r.target == self.agent_id
                            && r.state == "open"
                            && r.deadline_at > unix_now() as i64
                    })
                    .map(|r| (r.id.as_str(), r.sender.as_str()))
                    .collect();
                for pending in &mut self.state.pending {
                    if pending.reply_requested.is_none()
                        && open.contains(&(pending.request_id.as_str(), pending.from.as_str()))
                    {
                        pending.reply_requested = Some(true);
                    }
                }
                for request in requests {
                    if request.target != self.agent_id {
                        continue;
                    }
                    if matches!(
                        request.state.as_str(),
                        "answered" | "cancelled" | "timed_out"
                    ) {
                        self.state
                            .pending
                            .retain(|p| p.request_id != request.id || p.from != request.sender);
                        self.discard_by_id(
                            &request.id,
                            Discard::RequestClosed(request_state_label(&request.state)),
                        );
                    }
                    self.requests.insert(request.id.clone(), request);
                }
                let queued_ids: HashSet<&str> = self
                    .queue
                    .iter()
                    .map(|(frame, _)| delivery_message(frame).id.as_str())
                    .collect();
                self.requests
                    .retain(|id, _| queued_ids.contains(id.as_str()));
                let pending_ids: HashSet<&str> = self
                    .state
                    .pending
                    .iter()
                    .map(|pending| pending.request_id.as_str())
                    .collect();
                self.response_sent
                    .retain(|id, _| pending_ids.contains(id.as_str()));
                if let Err(e) = self.state.save(&self.state_path) {
                    warn!("réconciliation réponse : {e}");
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
                        info!("fil {} nommé « {display_name} »", self.thread_id);
                        self.title_refused_at = None;
                    }
                    DisplayNameOutcome::Rejected { reason } => {
                        warn!(
                            "nom du fil {} refusé : {reason:?} ; nouvel essai dans {:?}",
                            self.thread_id,
                            rename_retry()
                        );
                        self.title_refused_at = Some(Instant::now());
                    }
                }
            }
            DaemonToWrapper::Disconnect => {
                self.daemon_alive.store(false, Ordering::SeqCst);
                warn!("daemon a demandé la déconnexion du fil {}", self.thread_id);
            }
            _ => {}
        }
    }

    fn request_status(&self) {
        send_wrapper_message(
            &self.writer,
            WrapperToDaemon::ListRequests {
                sender: self.agent_id.clone(),
                limit: 200,
            },
        );
    }

    /// Session 113 : pourquoi une remise en file ne partira pas, ou `None` si
    /// elle attend légitimement la fin du tour.
    fn discard_cause(
        &self,
        frame: &DaemonToWrapper,
        received: Instant,
        now: u64,
    ) -> Option<Discard> {
        let message = delivery_message(frame);
        // Session 112 : `deadline_at` est le budget d'exécution du tour posé
        // par le daemon au moment de la poussée, pas une date de fraîcheur.
        // L'attente en file (destinataire occupé) ne le consomme pas : la
        // remise n'est écartée que si elle était déjà périmée à sa réception.
        // Le budget restant est reporté au dispatch.
        if message
            .deadline_at
            .is_some_and(|deadline| deadline_after_wait(deadline, received) <= now)
        {
            return Some(Discard::Expired("de tour"));
        }
        // Session 111 : l'attente seule ne périme plus. Un message sans
        // échéance attend la fin du tour ; c'est à l'expéditeur de déclarer
        // une échéance s'il en veut une.
        if message
            .reply_timeout
            .is_some_and(|timeout| received.elapsed() >= Duration::from_secs(timeout))
        {
            return Some(Discard::Expired("de réponse"));
        }
        if matches!(frame, DaemonToWrapper::DeliverIdempotent { expires_at, .. } if *expires_at <= now as i64)
        {
            return Some(Discard::Expired("de saga"));
        }
        match self.requests.get(&message.id) {
            Some(request) if request.state != "open" => {
                Some(Discard::RequestClosed(request_state_label(&request.state)))
            }
            Some(request) if request.deadline_at <= now as i64 => {
                Some(Discard::Expired("de demande"))
            }
            _ => None,
        }
    }

    /// Session 113 : une remise que le pont jette n'est jamais jetée en
    /// silence. Le daemon la croirait « en vol » jusqu'à expiration de la saga,
    /// et l'expéditeur d'une demande suivie attendrait pour rien.
    fn report_discard(&self, frame: &DaemonToWrapper, why: Discard) {
        let message = delivery_message(frame);
        let reason = match why {
            Discard::Saturated => {
                format!("file du fil saturée à {QUEUE_BOUND}, remise écartée")
            }
            Discard::Expired(which) => format!("échéance {which} dépassée avant démarrage"),
            Discard::RequestClosed(state) => format!("demande {state} avant démarrage"),
        };
        warn!(
            "remise {} écartée du fil {} : {reason}",
            message.id, self.thread_id
        );
        // Sort connu (jamais injectée) : rapporté comme le wrapper ACP le fait
        // pour une injection ratée, afin que la saga devienne terminale.
        if let DaemonToWrapper::DeliverIdempotent {
            delivery_id,
            delivery_generation,
            ..
        } = frame
        {
            send_wrapper_message(
                &self.writer,
                WrapperToDaemon::DeliveryIndeterminate {
                    delivery_id: delivery_id.clone(),
                    delivery_generation: *delivery_generation,
                },
            );
        }
        // Demande suivie encore ouverte : le daemon prévient l'expéditeur et,
        // sur un motif d'échéance, clôt la demande. Une demande déjà close a
        // déjà eu son issue : ne pas en fabriquer une seconde.
        if message.reply && !matches!(why, Discard::RequestClosed(_)) {
            send_wrapper_message(
                &self.writer,
                WrapperToDaemon::DeliveryRejected {
                    id: message.id.clone(),
                    reason,
                },
            );
        }
    }

    /// Retire de la file toute remise du message `id` et le dit au daemon.
    /// O(n), n ≤ QUEUE_BOUND.
    fn discard_by_id(&mut self, id: &str, why: Discard) {
        let queue = std::mem::take(&mut self.queue);
        for (frame, received) in queue {
            if delivery_message(&frame).id == id {
                self.report_discard(&frame, why);
            } else {
                self.queue.push_back((frame, received));
            }
        }
    }

    fn drive_queue(&mut self, inbox: &Receiver<LinkEvent>) {
        let now = unix_now();
        self.cancelled
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .retain(|_, received| received.elapsed() <= self.turn_wait);
        // O(n), n ≤ QUEUE_BOUND : chaque remise écartée est rapportée au daemon.
        let queue = std::mem::take(&mut self.queue);
        for (frame, received) in queue {
            match self.discard_cause(&frame, received, now) {
                Some(why) => self.report_discard(&frame, why),
                None => self.queue.push_back((frame, received)),
            }
        }
        if self.queue.is_empty() || !self.daemon_alive.load(Ordering::SeqCst) {
            return;
        }
        let snapshot = match self.session.call(|client| client.snapshot()) {
            Ok(snapshot) => snapshot,
            Err(e) => {
                warn!("attente du fil {} : {e}", self.thread_id);
                return;
            }
        };
        // Une requête HTTP peut prendre du temps. Appliquer tous les contrôles
        // reçus entre son départ et son retour AVANT toute commande fournisseur.
        self.drain_controls(inbox);
        if !self.daemon_alive.load(Ordering::SeqCst) {
            return;
        }
        let Some(summary) = snapshot
            .threads
            .into_iter()
            .find(|t| t.id == self.thread_id && t.is_live())
        else {
            return;
        };
        if summary
            .session
            .as_ref()
            .is_some_and(|s| s.active_turn_id.is_some())
        {
            return;
        }
        // Le temps de la requête HTTP a pu faire passer une échéance.
        let stale = self
            .queue
            .front()
            .and_then(|(frame, received)| self.discard_cause(frame, *received, unix_now()));
        if let Some(why) = stale {
            let (frame, _) = self.queue.pop_front().expect("remise présente");
            self.report_discard(&frame, why);
            return;
        }
        let Some((frame, _)) = self.queue.front() else {
            return;
        };
        let message = delivery_message(frame);
        // Pour une demande suivie, le daemon atteste l'échéance véritable (qui
        // peut être plus courte que celle du tour). Une absence dans une page
        // bornée n'est ni une autorisation de démarrage ni une clôture.
        if message.reply
            && !self.requests.get(&message.id).is_some_and(|r| {
                r.sender == message.from && r.state == "open" && r.deadline_at > unix_now() as i64
            })
        {
            self.request_status();
            return;
        }
        let (mut frame, received) = self.queue.pop_front().expect("remise présente");
        // Session 112 : le budget d'exécution repart du dispatch réel.
        match &mut frame {
            DaemonToWrapper::Deliver(message)
            | DaemonToWrapper::DeliverIdempotent { message, .. } => {
                if let Some(deadline) = message.deadline_at {
                    let shifted = deadline_after_wait(deadline, received);
                    if shifted != deadline {
                        info!(
                            "remise {} : {} s d'attente en file reportés sur l'échéance de tour",
                            message.id,
                            received.elapsed().as_secs()
                        );
                    }
                    message.deadline_at = Some(shifted);
                }
            }
            _ => {}
        }
        match frame {
            DaemonToWrapper::Deliver(message) => {
                if let Err(e) = self.dispatch_with_id(&message, None, &summary) {
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
                        outcome = self.dispatch_with_id(message, Some(&message.id), &summary);
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
            _ => unreachable!("la file ne contient que des remises"),
        }
    }

    /// Démarre le fil attesté libre par drive_queue, après contrôle d'autorité.
    /// `commandId` = identifiant du message Bridget (t3code déduplique dessus).
    fn dispatch_with_id(
        &mut self,
        message: &bridget_core::BridgetMessage,
        command_id: Option<&str>,
        summary: &ThreadSummary,
    ) -> Result<(), String> {
        if !self.daemon_alive.load(Ordering::SeqCst) {
            return Err("daemon déconnecté".into());
        }
        let anchor = summary.latest_turn.as_ref().map(|t| t.turn_id.clone());
        let command_id = command_id.unwrap_or(&message.id).to_string();
        // Identifiant de message déterministe : un rejeu après panne reconstruit
        // exactement la même commande, que t3code déduplique par `commandId`.
        let message_id = stable_uuid(&format!("t3code-message:{command_id}"));
        let command = contract::turn_start_command(
            &self.thread_id,
            &command_id,
            &message_id,
            &envelope(message),
            &summary.runtime_mode,
            &summary.interaction_mode,
        );
        // L'attente est écrite AVANT l'appel : si le pont meurt entre le 200 de
        // t3code et cette ligne, la corrélation serait perdue et la réponse ne
        // reviendrait jamais. Écrite avant, elle est au pire inutile — et reste
        // conservée jusqu'à un refus certain ou une clôture attestée.
        if message.id.starts_with("bridget-observation:") {
            if !self.state.notification_messages.contains(&message_id) {
                self.state.notification_messages.push(message_id.clone());
                if self.state.notification_messages.len() > SEEN_BOUND {
                    self.state.notification_messages.remove(0);
                }
                self.state.save(&self.state_path)?;
            }
        } else if message.reply
            && !self
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
                response: None,
                reply_requested: Some(true),
            });
            self.state.save(&self.state_path)?;
        }
        // La persistance peut elle-même prendre du temps. Le lecteur socket
        // publie la perte d'autorité sans attendre le traitement de sa file.
        if !self.daemon_alive.load(Ordering::SeqCst)
            || message
                .deadline_at
                .is_some_and(|deadline| deadline <= unix_now())
            || self
                .requests
                .get(&message.id)
                .is_some_and(|request| request.deadline_at <= unix_now() as i64)
        {
            return Err("autorité absente ou demande périmée avant dispatch".into());
        }
        // Session.call peut attendre le verrou partagé de renouvellement.
        // Vérifier aussi dans sa closure, à la dernière frontière HTTP.
        match self.session.call(|client| {
            if !self.daemon_alive.load(Ordering::SeqCst)
                || self
                    .cancelled
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .contains_key(&message.id)
                || message
                    .deadline_at
                    .is_some_and(|deadline| deadline <= unix_now())
                || self
                    .requests
                    .get(&message.id)
                    .is_some_and(|request| request.deadline_at <= unix_now() as i64)
            {
                return Err(ContractError::Transport(
                    "remise annulée, périmée ou autorité déconnectée avant dispatch".into(),
                ));
            }
            client.dispatch(&command)
        }) {
            Ok(result) => {
                info!(
                    "message {} remis au fil {} (séquence {})",
                    message.id, self.thread_id, result.sequence
                );
                Ok(())
            }
            // Seuls ces refus explicites attestent une commande rejetée.
            // JSON/sequence invalides APRÈS 2xx, 5xx ou coupure : le tour a pu
            // démarrer, sa corrélation doit survivre sans réexécution.
            Err(
                error @ ContractError::Http {
                    status: 400 | 404 | 405 | 413 | 415 | 422,
                    ..
                },
            ) => {
                self.state.pending.retain(|p| p.request_id != message.id);
                let _ = self.state.save(&self.state_path);
                Err(error.to_string())
            }
            Err(error) => Err(error.to_string()),
        }
    }

    /// Relit le fil (page bornée, élargie si l'ancre manque), projette le
    /// journal et clôt les remises dont la réponse est complète.
    fn refresh(&mut self, summary: &ThreadSummary) -> bool {
        let mut limit = DETAIL_PAGE;
        let detail = loop {
            let fetched = self
                .session
                .call(|client| client.thread_detail(&self.thread_id, limit));
            let detail = match fetched {
                Ok(detail) => detail,
                Err(e) => {
                    warn!("détail du fil {} : {e}", self.thread_id);
                    self.set_observation_ready(false);
                    return false;
                }
            };
            let anchors_missing = self
                .state
                .pending
                .iter()
                .filter(|p| p.needs_correlation())
                .any(|p| {
                    p.anchor_turn_id.as_ref().is_some_and(|anchor| {
                        !detail
                            .messages
                            .iter()
                            .any(|m| m.turn_id.as_deref() == Some(anchor))
                    })
                });
            let seeded = self.state.seeded;
            let origin_missing = detail.latest_turn.as_ref().is_some_and(|turn| {
                !self.state.turn_origins.contains_key(&turn.turn_id)
                    && observation_origin(&detail, turn).is_none()
            });
            if (anchors_missing || !seeded || origin_missing) && limit < DETAIL_PAGE_MAX {
                limit *= 2;
                continue;
            }
            break detail;
        };
        // Le fait de capacité précède les faits du journal sur le même writer,
        // notamment lors du rétablissement après une indisponibilité HTTP.
        use bridget_transport::protocol::ObservationKind as Kind;
        self.observation_schema.clear();
        let origin_known = detail.latest_turn.as_ref().is_none_or(|turn| {
            self.state.turn_origins.contains_key(&turn.turn_id)
                || observation_origin(&detail, turn).is_some()
        });
        if origin_known {
            self.observation_schema.push(Kind::TurnEnded);
        }
        if detail.activities_available && origin_known {
            self.observation_schema.push(Kind::PermissionRequired);
            // T3 ne conserve pas actuellement input.file_path de Claude.
            // Ne pas annoncer une couverture d'écritures qu'il ne fournit pas.
            if thread_provider(summary).as_deref() == Some("codex") {
                self.observation_schema.push(Kind::FileWritten);
            }
        }
        // La fenêtre T3 est bornée : absence de recouvrement à la reprise
        // signifie une lacune possible, jamais une continuité garantie.
        // O(A + S), A<=4096, S<=8192 : pas de recherche linéaire du curseur
        // pour chacune des activités à chaque lecture du fil.
        let seen_activities: HashSet<_> = self
            .state
            .seen_activities
            .iter()
            .map(String::as_str)
            .collect();
        let activity_gap = self.state.observations_seeded
            && detail.activities.iter().any(|a| {
                if seen_activities.contains(a.id.as_str()) {
                    return false;
                }
                let useful = a.kind == "approval.requested"
                    || (a.kind == "tool.completed"
                        && a.payload["itemType"] == "file_change"
                        && a.payload["status"] == "completed");
                let origin_known = a.turn_id.as_ref().is_some_and(|id| {
                    self.state.turn_origins.contains_key(id)
                        || detail.latest_turn.as_ref().is_some_and(|turn| {
                            turn.turn_id == *id && observation_origin(&detail, turn).is_some()
                        })
                });
                let paths_truncated = a.payload["data"]["files"]
                    .as_array()
                    .is_some_and(|files| files.len() >= 12);
                useful && (!origin_known || paths_truncated)
            });
        let turn_gap = self.state.observations_seeded
            && detail.messages.iter().any(|message| {
                message.turn_id.as_ref().is_some_and(|id| {
                    detail
                        .latest_turn
                        .as_ref()
                        .is_some_and(|latest| latest.turn_id != *id)
                        && !self.state.ended_turns.contains(id)
                        && !self.journal_inflight.contains(&format!("end:{id}"))
                })
            });
        let gap = activity_gap
            || turn_gap
            || (self.state.observations_seeded
                && !self.state.seen_activities.is_empty()
                && !detail.activities.is_empty()
                && !detail
                    .activities
                    .iter()
                    .any(|a| seen_activities.contains(a.id.as_str())));
        self.set_observation_ready(!self.journal_failed.load(Ordering::SeqCst));
        if gap {
            // Lacune connue, quantité inconnue : ne pas inventer un nombre de
            // faits perdus ni deux notices de fausse déconnexion/reconnexion.
            send_wrapper_message(&self.writer, WrapperToDaemon::ObservationGap { dropped: 0 });
        }
        self.project_journal(&detail, summary);
        self.settle_pending(&detail, summary);
        if let Err(e) = self.state.save(&self.state_path) {
            warn!("état du fil {} : {e}", self.thread_id);
            self.set_observation_ready(false);
            return false;
        }
        let ready = self.journal_caught_up
            && !self.journal_dirty
            && !self.journal_failed.load(Ordering::SeqCst);
        self.set_observation_ready(ready);
        ready
    }

    /// Annonce un fait de source, jamais une capacité déduite du titre du fil.
    fn set_observation_ready(&mut self, ready: bool) {
        let events = if ready {
            self.observation_schema.clone()
        } else {
            vec![]
        };
        if events == self.observation_events {
            return;
        }
        self.observation_events = events.clone();
        send_wrapper_message(
            &self.writer,
            WrapperToDaemon::ObservationCapabilities { events },
        );
    }

    /// Repère d'installation : au premier regard, l'historique antérieur est
    /// mémorisé sans être rejoué. Ensuite, chaque message nouveau est projeté.
    fn project_journal(&mut self, detail: &ThreadDetail, summary: &ThreadSummary) {
        self.confirm_journal();
        self.journal_dirty = true;
        if !self.journal_caught_up {
            return;
        }
        if !self.state.seeded {
            for message in &detail.messages {
                self.state.remember(&message.id);
            }
            let latest = detail.latest_turn.as_ref().or(summary.latest_turn.as_ref());
            self.state
                .ended_turns
                .extend(detail.messages.iter().filter_map(|m| {
                    m.turn_id
                        .as_ref()
                        .filter(|id| {
                            latest.is_none_or(|turn| {
                                &turn.turn_id != *id || turn_is_final(&turn.state)
                            })
                        })
                        .cloned()
                }));
            // Une fin sans texte assistant fait aussi partie de l'historique.
            if let Some(turn) = latest.filter(|turn| turn_is_final(&turn.state))
                && !self.state.ended_turns.contains(&turn.turn_id)
            {
                self.state.ended_turns.push(turn.turn_id.clone());
            }
            if let Some(turn) = latest
                && let Some(origin) = observation_origin(detail, turn)
            {
                self.state
                    .turn_origins
                    .insert(turn.turn_id.clone(), origin.into());
            }
            self.state
                .seen_activities
                .extend(detail.activities.iter().map(|a| a.id.clone()));
            self.state.observations_seeded = true;
            self.state.seeded = true;
            self.journal_dirty = false;
            return;
        }
        if self.journal_failed.load(Ordering::SeqCst) {
            return;
        }
        let latest = detail.latest_turn.as_ref().or(summary.latest_turn.as_ref());
        // Migration100 : les activités déjà présentes ne sont pas des faits futurs.
        if !self.state.observations_seeded {
            self.state
                .seen_activities
                .extend(detail.activities.iter().map(|a| a.id.clone()));
            if let Some(turn) = latest.filter(|turn| turn_is_final(&turn.state)) {
                self.state.ended_turns.push(turn.turn_id.clone());
            }
            self.state.notification_messages.extend(
                self.state
                    .pending
                    .iter()
                    .filter(|p| p.request_id.starts_with("bridget-observation:"))
                    .map(|p| p.message_id.clone()),
            );
            self.state
                .pending
                .retain(|p| !p.request_id.starts_with("bridget-observation:"));
            self.state.observations_seeded = true;
        }
        if let Some(turn) = latest
            && let Some(origin) = observation_origin(detail, turn)
        {
            self.state
                .turn_origins
                .insert(turn.turn_id.clone(), origin.to_string());
        }
        let final_turn = |turn_id: &str| {
            summary
                .latest_turn
                .as_ref()
                .is_none_or(|latest| latest.turn_id != turn_id || turn_is_final(&latest.state))
        };
        for message in &detail.messages {
            if message.streaming {
                continue;
            }
            if !self.state.seen.contains(&message.id)
                && !self.journal_inflight.contains(&message.id)
            {
                let (mut event, mut payload) = if message.role == "user" {
                    let from = self
                        .state
                        .pending
                        .iter()
                        .find(|p| p.message_id == message.id)
                        .map(|p| p.from.as_str())
                        .unwrap_or("human");
                    (
                        "turn_start",
                        serde_json::json!({"from": from, "body": message.text}),
                    )
                } else {
                    (
                        "update",
                        serde_json::json!({"kind": "text", "text": message.text}),
                    )
                };
                // La borne concerne le JSON échappé, pas les caractères UTF-8.
                // Réserver l'enveloppe sous la limite lecteur existante de 4 Mio.
                if serde_json::to_vec(&payload).map_or(true, |v| v.len() > 4 * 1024 * 1024 - 4096) {
                    event = "error";
                    payload = serde_json::json!({"gap": true, "reason": "sortie t3code au-delà de la borne de journalisation (4 Mio)", "source_bytes": message.text.len()});
                }
                match self.journal.enqueue(event, Some(&message.id), payload) {
                    Ok(()) => {
                        self.journal_inflight.insert(message.id.clone());
                    }
                    Err(e) => {
                        warn!("journal du fil {} : {e}", self.thread_id);
                        // Ne pas dépasser un trou : sinon B peut être écrit
                        // avant A lorsque la file se libère pendant ce lot.
                        return;
                    }
                }
            }
            if let Some(turn_id) = message.turn_id.as_deref()
                && !self.state.ended_turns.iter().any(|id| id == turn_id)
                && !self.journal_inflight.contains(&format!("end:{turn_id}"))
                && final_turn(turn_id)
                && latest.is_none_or(|turn| turn.turn_id != turn_id)
            {
                match self.journal.enqueue(
                    "turn_end",
                    Some(&message.id),
                    serde_json::json!({"t3_turn_id": turn_id}),
                ) {
                    Ok(()) => {
                        self.journal_inflight.insert(format!("end:{turn_id}"));
                    }
                    Err(e) => {
                        warn!("fin de tour non journalisée : {e}");
                        return;
                    }
                }
            }
        }
        // L'état terminal vient du contrat T3 fermé, jamais du silence ou d'idle.
        if let Some(turn) = latest.filter(|turn| turn_is_final(&turn.state))
            && !self.state.ended_turns.contains(&turn.turn_id)
            && !self
                .journal_inflight
                .contains(&format!("end:{}", turn.turn_id))
        {
            let origin = self.state.turn_origins.get(&turn.turn_id);
            let notification =
                origin.is_some_and(|id| self.state.notification_messages.contains(id));
            let id = if notification {
                format!("bridget-observation:{}", turn.turn_id)
            } else {
                origin.cloned().unwrap_or_else(|| turn.turn_id.clone())
            };
            let payload = if origin.is_some() {
                serde_json::json!({"t3_turn_id":turn.turn_id,"stop_reason":turn.state})
            } else {
                serde_json::json!({"t3_turn_id":turn.turn_id,"gap":true,"reason":"origine du tour non attestée, notification de fin supprimée"})
            };
            if self
                .journal
                .enqueue("turn_end", Some(&id), payload)
                .is_err()
            {
                return;
            }
            self.journal_inflight
                .insert(format!("end:{}", turn.turn_id));
        }
        let provider = thread_provider(summary).unwrap_or_default();
        let cwd = Path::new(summary.worktree_path.as_deref().unwrap_or(""));
        let seen: HashSet<_> = self
            .state
            .seen_activities
            .iter()
            .map(String::as_str)
            .collect();
        let mut ignored = Vec::new();
        for activity in &detail.activities {
            if seen.contains(activity.id.as_str()) {
                continue;
            }
            let key = format!("activity:{}", activity.id);
            if self.journal_inflight.contains(&key) {
                continue;
            }
            let origin = activity
                .turn_id
                .as_ref()
                .and_then(|turn| self.state.turn_origins.get(turn));
            let notification =
                origin.is_some_and(|id| self.state.notification_messages.contains(id));
            let payloads = activity_payloads(activity, &provider, cwd);
            if payloads.is_empty() {
                ignored.push(activity.id.clone());
                continue;
            }
            // Métadonnées uniquement ; les autres activités ne sont pas recopiées.
            for (index, mut payload) in payloads.into_iter().enumerate() {
                payload["t3_activity_id"] = serde_json::json!(activity.id);
                let id = if notification {
                    format!("bridget-observation:{}:{index}", activity.id)
                } else {
                    format!("t3-activity:{}:{index}", activity.id)
                };
                // Une admission partielle n'autorise pas à rejouer les fichiers
                // déjà confirmés ou encore dans la file d'écriture.
                if self.state.seen.contains(&id) || self.journal_inflight.contains(&id) {
                    continue;
                }
                // Sans origine prouvée, écrire une lacune et ne pas nourrir le catalogue.
                let event = if origin.is_some() {
                    "update"
                } else {
                    payload = serde_json::json!({"t3_activity_id":activity.id,"gap":true,"reason":"origine activité non attestée"});
                    "error"
                };
                if self.journal.enqueue(event, Some(&id), payload).is_err() {
                    return;
                }
                self.journal_inflight.insert(id);
            }
            // Repère séparé : ne pas confirmer une activité multi-fichiers sur le
            // premier fichier seulement ; le marqueur suit tous ses faits.
            if self
                .journal
                .enqueue(
                    "t3_activity_seen",
                    Some(&key),
                    serde_json::json!({"t3_activity_id":activity.id}),
                )
                .is_err()
            {
                return;
            }
            self.journal_inflight.insert(key);
        }
        self.state.seen_activities.extend(ignored);
        if self.state.seen_activities.len() > 8192 {
            self.state
                .seen_activities
                .drain(..self.state.seen_activities.len() - 8192);
        }
        if self.state.turn_origins.len() > SEEN_BOUND {
            let visible: HashSet<_> = detail
                .messages
                .iter()
                .filter_map(|m| m.turn_id.as_ref())
                .collect();
            self.state
                .turn_origins
                .retain(|id, _| visible.contains(id) || latest.is_some_and(|t| t.turn_id == *id));
        }
        self.journal_dirty = false;
    }

    /// Le flux live est consommé par AttachRelayWorker. Réutiliser le lecteur
    /// incrémental du journal pour attester l'append, y compris après un crash
    /// entre flush et sauvegarde du curseur, sans voler les événements d'attach.
    fn confirm_journal(&mut self) {
        let Ok(entries) = std::fs::read_dir(&self.journal_dir) else {
            return;
        };
        self.journal_caught_up = true;
        let mut paths: Vec<_> = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
            .collect();
        paths.sort();
        let mut changed = false;
        for path in paths {
            let reader = self
                .journal_readers
                .entry(path.clone())
                .or_insert_with(|| IncrementalJournalReader::new(path.clone()));
            // Budget par tour de boucle ; les lecteurs gardent leur position.
            let items = match reader.read_chunk(4 * 1024 * 1024) {
                Ok(items) => items,
                Err(e) => {
                    self.journal_caught_up = false;
                    warn!("confirmation journal : {e}");
                    break;
                }
            };
            for item in items {
                let JournalReadItem::Event(entry) = item else {
                    continue;
                };
                let Ok(value) = serde_json::from_slice::<serde_json::Value>(&entry.bytes) else {
                    continue;
                };
                let Some(id) = value["message_id"].as_str() else {
                    continue;
                };
                match value["event"].as_str() {
                    Some("t3_activity_seen") => {
                        if let Some(activity) = value["payload"]["t3_activity_id"].as_str() {
                            if !self.state.seen_activities.iter().any(|id| id == activity) {
                                self.state.seen_activities.push(activity.into());
                                changed = true;
                            }
                            self.journal_inflight
                                .remove(&format!("activity:{activity}"));
                        }
                    }
                    Some("turn_start" | "update") => {
                        if !self.state.seen.iter().any(|seen| seen == id) {
                            self.state.remember(id);
                            changed = true;
                        }
                        self.journal_inflight.remove(id);
                    }
                    Some("error") if value["payload"]["gap"] == true => {
                        if !self.state.seen.iter().any(|seen| seen == id) {
                            self.state.remember(id);
                            changed = true;
                        }
                        self.journal_inflight.remove(id);
                    }
                    Some("turn_end") => {
                        if let Some(turn) = value["payload"]["t3_turn_id"].as_str() {
                            if !self.state.ended_turns.iter().any(|ended| ended == turn) {
                                self.state.ended_turns.push(turn.to_string());
                                changed = true;
                            }
                            self.journal_inflight.remove(&format!("end:{turn}"));
                        }
                    }
                    _ => {}
                }
            }
            if std::fs::metadata(&path).map_or(true, |m| reader.next_offset() < m.len()) {
                self.journal_caught_up = false;
                break;
            }
        }
        if self.state.ended_turns.len() > SEEN_BOUND {
            self.state
                .ended_turns
                .drain(..self.state.ended_turns.len() - SEEN_BOUND);
        }
        if self.state.seen_activities.len() > 8192 {
            self.state
                .seen_activities
                .drain(..self.state.seen_activities.len() - 8192);
        }
        if changed && let Err(e) = self.state.save(&self.state_path) {
            warn!("curseur journal non sauvegardé : {e}");
        }
    }

    fn settle_pending(&mut self, detail: &ThreadDetail, summary: &ThreadSummary) {
        // La collection reste entière pendant chaque remplacement atomique :
        // une panne ne peut pas laisser sur disque un simple préfixe traité.
        for pending in &mut self.state.pending {
            if !pending.needs_correlation() {
                continue;
            }
            let verdict = correlate(detail, summary, pending);
            match verdict {
                Correlation::Answered(text) => {
                    pending.response = Some(text);
                }
                Correlation::Waiting => {}
                Correlation::Ambiguous | Correlation::Missing => {
                    pending.attempts = pending.attempts.saturating_add(1);
                    if pending.attempts == SETTLE_ATTEMPTS {
                        let reason = match verdict {
                            Correlation::Ambiguous => format!(
                                "fil {} au repos sans appariement certain entre messages et tours après {} lectures ; demande laissée sans réponse plutôt qu'attribuée au tour d'autrui",
                                self.thread_id, pending.attempts
                            ),
                            _ => format!(
                                "message {} introuvable dans le fil t3code après {} lectures ; attente conservée sans réponse inventée",
                                pending.message_id, pending.attempts
                            ),
                        };
                        warn!("{reason}");
                        let _ = self.journal.enqueue(
                            "error",
                            Some(&pending.request_id),
                            serde_json::json!({"reason": reason}),
                        );
                    }
                }
            }
        }
        // Une réponse n'est jamais expédiée avant sa conservation durable.
        if let Err(e) = self.state.save(&self.state_path) {
            warn!("état du fil {} : {e}", self.thread_id);
            return;
        }
        self.send_ready_responses();
    }

    fn send_ready_responses(&mut self) {
        if !self.daemon_alive.load(Ordering::SeqCst) {
            return;
        }
        // Également requis à la reprise si une sauvegarde précédente a échoué.
        if let Err(e) = self.state.save(&self.state_path) {
            warn!("réponses non persistées, envoi différé : {e}");
            return;
        }
        for pending in &self.state.pending {
            if pending.reply_requested != Some(true) {
                continue;
            }
            let Some(text) = pending.response.as_ref() else {
                continue;
            };
            if self
                .response_sent
                .get(&pending.request_id)
                .is_some_and(|sent| sent.elapsed() < RECONNECT_BACKOFF)
            {
                continue;
            }
            let mut reply = bridget_core::BridgetMessage::new(
                self.agent_id.clone(),
                pending.from.clone(),
                text.clone(),
            );
            reply.id = reply_id(&pending.request_id);
            reply.in_reply_to = Some(pending.request_id.clone());
            send_wrapper_message(&self.writer, WrapperToDaemon::Send(reply));
            self.response_sent
                .insert(pending.request_id.clone(), Instant::now());
        }
    }
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Session 112 : échéance de tour créditée du temps passé en file. Le daemon
/// pose `deadline_at = poussée + budget` ; le budget doit courir à partir du
/// dispatch effectif, pas de la poussée. Une échéance déjà passée à la
/// réception reste passée : le report ne ressuscite rien.
fn deadline_after_wait(deadline: u64, received: Instant) -> u64 {
    deadline.saturating_add(received.elapsed().as_secs())
}

/// Session 113 : motif pour lequel le pont renonce à une remise avant tout
/// démarrage. Chaque motif est rapporté au daemon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Discard {
    /// File bornée : un message plus récent a pris la place.
    Saturated,
    /// Échéance de tour, de réponse, de saga ou de demande déjà passée.
    Expired(&'static str),
    /// Demande suivie déjà close côté daemon : l'expéditeur le sait, seule
    /// la saga d'envoi reste à clore.
    RequestClosed(&'static str),
}

fn request_state_label(state: &str) -> &'static str {
    match state {
        "answered" => "répondue",
        "cancelled" => "annulée",
        "timed_out" => "expirée",
        _ => "close",
    }
}

fn delivery_message(frame: &DaemonToWrapper) -> &bridget_core::BridgetMessage {
    match frame {
        DaemonToWrapper::Deliver(message) | DaemonToWrapper::DeliverIdempotent { message, .. } => {
            message
        }
        _ => unreachable!("seules les remises sont mises en attente"),
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

/// Le pont ne relaie la réponse finale que si elle a été demandée.
pub(crate) fn envelope(message: &bridget_core::BridgetMessage) -> String {
    if let Some(notice) = &message.thread_notice {
        // Session 102 : sollicitation de fil. Aucun relais de la réponse finale,
        // aucun accusé : le destinataire lit et publie par bridget_thread.
        return format!(
            "🧵 Sollicitation Bridget dans le fil {} (nouveautés jusqu'à {}, id {}) :\n\n{}\n\nConsulte ce fil avec l'outil bridget_thread (read, puis ack) ; publie dans le fil si utile (post). Ne fais pas de réponse directe à cette alerte : le pont ne relaie pas ta réponse finale.",
            notice.thread_id, notice.through_seq, message.id, message.body
        );
    }
    if message.id.starts_with("bridget-observation:") {
        return format!(
            "🔔 Notification Bridget (id {}) :\n\n{}\n\nAucune réponse inter-agent requise ; informe l'utilisateur si utile.",
            message.id, message.body
        );
    }
    if !message.reply {
        return format!(
            "💬 Message Bridget de {} (id {}, reply=no) :\n\n{}\n\nAucune réponse inter-agent attendue. N'envoie pas d'accusé de réception, même pour dire que tu ne répondras pas. Le pont ne relaie pas ta réponse finale pour ce message.",
            message.from, message.id, message.body
        );
    }
    format!(
        "💬 Message Bridget de {} (id {}) :\n\n{}\n\n— Réponds normalement dans ce tour : Bridget transmettra ta réponse à {}.",
        message.from, message.id, message.body, message.from
    )
}

pub(crate) fn turn_is_final(state: &str) -> bool {
    matches!(state, "completed" | "error" | "interrupted")
}

/// O(messages), page bornée. T3 projette requestedAt et createdAt depuis la
/// même commande. Égalité unique, jamais ordre temporel ni texte de l'utilisateur.
fn observation_origin<'a>(
    detail: &'a ThreadDetail,
    turn: &contract::LatestTurn,
) -> Option<&'a str> {
    let requested = turn.requested_at.as_ref()?;
    let mut candidates = detail
        .messages
        .iter()
        .filter(|m| m.role == "user" && m.created_at.as_ref() == Some(requested));
    let origin = candidates.next()?;
    candidates.next().is_none().then_some(origin.id.as_str())
}

/// O(fichiers), au plus256 chemins. Type fermé ET succès ; aucun parsing shell.
fn activity_payloads(
    activity: &contract::Activity,
    provider: &str,
    cwd: &Path,
) -> Vec<serde_json::Value> {
    use bridget_transport::journal::confirmed_write_payload;
    if activity.kind == "approval.requested" {
        return vec![serde_json::json!({"kind":"approval","text":"permission demandée dans T3"})];
    }
    let payload = &activity.payload;
    if activity.kind != "tool.completed"
        || payload["itemType"] != "file_change"
        || payload["status"] != "completed"
    {
        return vec![];
    }
    // Claude : T3 perd input.file_path et peut synthétiser completed à la fin
    // du tour sans résultat d'outil attesté. Aucun fait d'écriture dans ce cas.
    if provider != "codex" {
        return vec![];
    }
    let Some(files) = payload["data"]["files"]
        .as_array()
        .filter(|files| files.len() <= 256)
    else {
        return vec![];
    };
    let mut seen = HashSet::new();
    files
        .iter()
        .filter_map(|file| file["path"].as_str())
        .filter(|path| seen.insert(*path))
        .filter_map(|path| confirmed_write_payload(path, cwd))
        .collect()
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

    fn replies105(peer: UnixStream) -> Vec<bridget_core::BridgetMessage> {
        peer.set_nonblocking(true).unwrap();
        let mut reader = BufReader::new(peer);
        let mut replies = Vec::new();
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    if let WrapperToDaemon::Send(message) = serde_json::from_str(&line).unwrap() {
                        replies.push(message);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => panic!("lecture des trames : {e}"),
            }
        }
        replies
    }

    #[test]
    fn spec105_notification_et_reponse_ne_demandent_pas_de_relais() {
        for parent in [None, Some("question")] {
            let mut msg = bridget_core::BridgetMessage::new("alice", "bob", "Bien reçu");
            msg.in_reply_to = parent.map(str::to_string);
            assert!(!envelope(&msg).contains("transmettra ta réponse"));
            assert!(envelope(&msg).contains("Aucune réponse"));
            msg.reply = true;
            assert!(envelope(&msg).contains("transmettra ta réponse"));
        }
    }

    #[test]
    fn spec102_v28_alerte_de_fil_injectee_sans_attente_ni_relais() {
        let (mut worker, peer) = worker099();
        let mut msg = bridget_core::BridgetMessage::new(
            "bridget",
            &worker.agent_id,
            "Sollicitation dans le fil t, nouveautés jusqu'à 3.",
        );
        msg.id = "thread-notice:t:agent:1".into();
        msg.origin = Some(bridget_core::MessageOrigin::System);
        msg.intent = Some(bridget_core::MessageIntent::TriggerTurn);
        msg.thread_notice = Some(bridget_core::ThreadNotice {
            version: 1,
            thread_id: "33333333-3333-4333-8333-333333333333".into(),
            through_seq: 3,
            generation: 1,
        });
        let text = envelope(&msg);
        assert!(
            text.contains("bridget_thread")
                && text.contains("33333333-3333-4333-8333-333333333333")
        );
        assert!(text.contains("jusqu'à 3"));
        assert!(
            !text.contains("transmettra ta réponse"),
            "aucun relais de réponse finale"
        );
        assert!(
            !text.starts_with("🔔"),
            "une sollicitation n'est pas une observation"
        );
        let server = http_once099(&worker, 200, r#"{"sequence":1}"#.into());
        let result = worker.dispatch_with_id(&msg, Some(&msg.id), &summary(None));
        let request = server.join().unwrap();
        worker.journal.stop();
        worker.relay.shutdown();
        assert!(result.is_ok(), "{result:?}");
        assert!(
            request.contains("bridget_thread"),
            "l'enveloppe typée est injectée"
        );
        assert!(
            worker.state.pending.is_empty(),
            "aucune attente de message direct"
        );
        assert!(ThreadState::load(&worker.state_path).pending.is_empty());
        assert!(replies105(peer).is_empty(), "aucune réponse automatique");
    }

    #[test]
    fn spec105_dispatch_sans_demande_ne_cree_aucune_attente() {
        for parent in [None, Some("question")] {
            let (mut worker, _peer) = worker099();
            let mut msg = bridget_core::BridgetMessage::new("alice", &worker.agent_id, ".");
            msg.in_reply_to = parent.map(str::to_string);
            let server = http_once099(&worker, 200, r#"{"sequence":1}"#.into());
            let result = worker.dispatch_with_id(&msg, None, &summary(None));
            let request = server.join().unwrap();
            worker.journal.stop();
            worker.relay.shutdown();
            assert!(request.starts_with("POST /api/orchestration/dispatch"));
            assert!(result.is_ok(), "{result:?}");
            assert!(worker.state.pending.is_empty());
            assert!(ThreadState::load(&worker.state_path).pending.is_empty());
        }
    }

    #[test]
    fn spec105_ancienne_reponse_prete_ne_part_pas_sans_preuve() {
        let (mut worker, peer) = worker099();
        worker.state = serde_json::from_value(serde_json::json!({"pending":[{
            "request_id":"legacy", "from":"alice", "message_id":"u",
            "anchor_turn_id":null, "dispatched_at":"", "response":"Noté."
        }]}))
        .unwrap();
        worker.state.save(&worker.state_path).unwrap();
        worker.state = ThreadState::load(&worker.state_path);
        worker.send_ready_responses();
        worker.journal.stop();
        worker.relay.shutdown();
        assert!(worker.response_sent.is_empty());
        assert!(replies105(peer).is_empty());
        assert_eq!(worker.state.pending.len(), 1, "préserver sans relayer");
    }

    #[test]
    fn spec105_reprise_exige_demande_ouverte_et_identites_exactes() {
        for case in [
            "absente",
            "expediteur",
            "cible",
            "expiree",
            "annulee",
            "ouverte",
        ] {
            let (mut worker, peer) = worker099();
            let mut old = pending("u", None);
            old.reply_requested = None;
            old.response = Some("OK".into());
            worker.state.pending.push(old);
            let mut request = RequestInfo {
                id: "req".into(),
                sender: "alice".into(),
                target: worker.agent_id.clone(),
                state: "open".into(),
                created_at: 0,
                deadline_at: i64::MAX,
                cancel_reason: None,
                deferred_reminder_level: None,
                deferred_reminder_at: None,
            };
            match case {
                "expediteur" => request.sender = "autre".into(),
                "cible" => request.target = "autre".into(),
                "expiree" => request.deadline_at = 0,
                "annulee" => request.state = "cancelled".into(),
                _ => {}
            }
            worker.handle_frame(DaemonToWrapper::RequestList {
                requests: if case == "absente" {
                    vec![]
                } else {
                    vec![request]
                },
            });
            worker.state = ThreadState::load(&worker.state_path);
            worker.send_ready_responses();
            worker.send_ready_responses();
            worker.journal.stop();
            worker.relay.shutdown();
            assert_eq!(
                worker.response_sent.len(),
                usize::from(case == "ouverte"),
                "{case}"
            );
            let replies = replies105(peer);
            assert_eq!(replies.len(), usize::from(case == "ouverte"), "{case}");
            if case == "ouverte" {
                let r = &replies[0];
                assert!(
                    !r.reply
                        && r.body == "OK"
                        && r.to == "alice"
                        && r.in_reply_to.as_deref() == Some("req")
                );
            } else if case == "annulee" {
                assert!(worker.state.pending.is_empty());
            } else {
                assert_eq!(worker.state.pending[0].reply_requested, None);
            }
        }
    }

    #[test]
    fn spec105_question_suivie_relayee_une_fois_puis_arret() {
        let (mut worker, peer) = worker099();
        let mut question = bridget_core::BridgetMessage::new("alice", &worker.agent_id, "Valide ?");
        question.reply = true;
        question.in_reply_to = Some("question-precedente".into());
        let server = http_once099(&worker, 200, r#"{"sequence":1}"#.into());
        worker
            .dispatch_with_id(&question, None, &summary(None))
            .unwrap();
        server.join().unwrap();
        worker.state = ThreadState::load(&worker.state_path);
        let detail = ThreadDetail {
            activities: vec![],
            activities_available: false,
            latest_turn: None,
            messages: vec![
                message(
                    &worker.state.pending[0].message_id,
                    "user",
                    "Valide ?",
                    None,
                    false,
                ),
                message("answer", "assistant", "OK", Some("t"), false),
            ],
        };
        worker.settle_pending(&detail, &summary(Some(("t", "completed"))));
        let replies = replies105(peer);
        assert_eq!(replies.len(), 1);
        let response = &replies[0];
        assert_eq!(response.in_reply_to.as_deref(), Some(question.id.as_str()));
        assert!(!response.reply);
        assert_eq!(response.body, "OK");
        let (mut recipient, _peer) = worker099();
        let server = http_once099(&recipient, 200, r#"{"sequence":1}"#.into());
        recipient
            .dispatch_with_id(response, None, &summary(None))
            .unwrap();
        server.join().unwrap();
        assert!(
            recipient.state.pending.is_empty(),
            "pas de réponse à la réponse"
        );
        for w in [&mut worker, &mut recipient] {
            w.journal.stop();
            w.relay.shutdown();
        }
    }

    #[test]
    fn spec105_attente_non_attestee_ne_correlle_ni_ne_force_la_lecture() {
        for flag in [None, Some(false)] {
            let (mut worker, _peer) = worker099();
            let mut old = pending("u", Some("ancre-ancienne"));
            old.reply_requested = flag;
            worker.state.pending.push(old);
            let detail = ThreadDetail {
                activities: vec![],
                activities_available: false,
                latest_turn: None,
                messages: vec![],
            };
            worker.settle_pending(&detail, &summary(None));
            assert_eq!(worker.state.pending[0].attempts, 0);
            assert!(!worker.state.pending[0].needs_correlation());
            let summary = summary(None);
            worker.last_key = summary.change_key();
            let server = http_once099(&worker, 200, r#"{"thread":{"messages":[]}}"#.into());
            worker.handle_event(LinkEvent::Tick(Box::new(summary)));
            assert!(
                server.join().unwrap().is_empty(),
                "pas de HTTP pour une attente sans preuve"
            );
            worker.journal.stop();
            worker.relay.shutdown();
        }
    }

    #[test]
    fn spec101_t3_terminal_state_is_closed() {
        assert!(!turn_is_final("future_unknown_state"));
        assert!(turn_is_final("completed"));
        assert!(turn_is_final("error"));
        assert!(turn_is_final("interrupted"));
    }

    #[test]
    fn spec101_t3_observation_notification_ne_demande_pas_de_reponse() {
        let mut msg = bridget_core::BridgetMessage::new("bridget", "target", "fait observé");
        msg.id = "bridget-observation:test".into();
        msg.origin = Some(bridget_core::MessageOrigin::System);
        let text = envelope(&msg);
        assert!(!text.contains("transmettra ta réponse"));
        assert!(text.contains("Aucune réponse"));
    }

    #[test]
    fn spec101_t3_fin_sans_texte_et_origine_non_ambigue() {
        let detail = contract::parse_thread_detail(r#"{"thread":{
            "messages":[{"id":"u","role":"user","text":"travaille","streaming":false,"turnId":null,"createdAt":"2026-09-16T12:00:00Z"}],
            "latestTurn":{"turnId":"t","state":"interrupted","requestedAt":"2026-09-16T12:00:00Z"},"activities":[]}}"#).unwrap();
        let latest = detail.latest_turn.as_ref().unwrap();
        assert_eq!(observation_origin(&detail, latest), Some("u"));
        let mut ambiguous = detail.clone();
        ambiguous.messages.push(ambiguous.messages[0].clone());
        assert_eq!(observation_origin(&ambiguous, latest), None);
    }

    #[test]
    fn spec101_t3_lectures_et_echecs_ne_sont_pas_ecritures() {
        let mut activity = contract::Activity {
            id: "a".into(),
            kind: "tool.completed".into(),
            turn_id: Some("t".into()),
            payload: serde_json::json!({"itemType":"file_change","status":"completed",
                "data":{"toolName":"ReadFile","files":[{"path":"/tmp/fichier"}]}}),
        };
        assert!(activity_payloads(&activity, "claude", Path::new("/tmp")).is_empty());
        activity.payload["data"]["toolName"] = serde_json::json!("Write");
        assert!(activity_payloads(&activity, "claude", Path::new("/tmp")).is_empty());
        assert_eq!(
            activity_payloads(&activity, "codex", Path::new("/tmp")).len(),
            1
        );
        activity.payload["status"] = serde_json::json!("failed");
        assert!(activity_payloads(&activity, "claude", Path::new("/tmp")).is_empty());
        activity.kind = "approval.requested".into();
        assert_eq!(
            activity_payloads(&activity, "claude", Path::new("/tmp"))[0]["kind"],
            "approval"
        );
    }

    #[test]
    fn spec101_t3_journal_confirme_fin_permission_et_fichiers_sans_doublon() {
        let (mut worker, _peer) = worker099();
        worker.state.seeded = true;
        worker.state.observations_seeded = true;
        let detail = contract::parse_thread_detail(r#"{"thread":{
            "messages":[{"id":"u","role":"user","streaming":false,"createdAt":"date"}],
            "latestTurn":{"turnId":"t","state":"completed","requestedAt":"date"},
            "activities":[
              {"id":"permission","kind":"approval.requested","turnId":"t","payload":{}},
              {"id":"write","kind":"tool.completed","turnId":"t","payload":{"itemType":"file_change","status":"completed","data":{"files":[{"path":"/tmp/a"},{"path":"/tmp/b"}]}}}
            ]}}"#).unwrap();
        let mut summary = summary(Some(("t", "completed")));
        summary.provider_instance_id = Some("codex".into());
        worker.project_journal(&detail, &summary);
        worker.journal.stop();
        let entries = std::fs::read_dir(&worker.journal_dir)
            .unwrap()
            .flat_map(|path| bridget_transport::journal::valid_events(&path.unwrap().path()))
            .collect::<Vec<_>>();
        assert_eq!(
            entries
                .iter()
                .filter(|v| v["payload"]["stop_reason"] == "completed")
                .count(),
            1
        );
        assert_eq!(
            entries
                .iter()
                .filter(|v| v["payload"]["write_confirmed"] == true)
                .count(),
            2
        );
        assert_eq!(
            entries
                .iter()
                .filter(|v| v["payload"]["kind"] == "approval")
                .count(),
            1
        );
        worker.journal_inflight.clear();
        worker.journal_readers.clear();
        worker.project_journal(&detail, &summary);
        assert!(
            worker.journal_inflight.is_empty(),
            "aucun fait n'est réémis après flush sans curseur"
        );
        worker.relay.shutdown();
    }

    #[test]
    fn spec101_t3_tour_notification_porte_origine_exclue_du_collecteur() {
        let (mut worker, _peer) = worker099();
        worker.state.seeded = true;
        worker.state.observations_seeded = true;
        worker.state.notification_messages.push("u".into());
        let detail = contract::parse_thread_detail(
            r#"{"thread":{
            "messages":[{"id":"u","role":"user","streaming":false,"createdAt":"date"}],
            "latestTurn":{"turnId":"t","state":"error","requestedAt":"date"},"activities":[]}}"#,
        )
        .unwrap();
        worker.project_journal(&detail, &summary(Some(("t", "error"))));
        worker.journal.stop();
        let entries = std::fs::read_dir(&worker.journal_dir)
            .unwrap()
            .flat_map(|path| bridget_transport::journal::valid_events(&path.unwrap().path()))
            .collect::<Vec<_>>();
        let end = entries.iter().find(|v| v["event"] == "turn_end").unwrap();
        assert!(
            end["message_id"]
                .as_str()
                .unwrap()
                .starts_with("bridget-observation:")
        );
        assert!(worker.state.pending.is_empty());
        worker.relay.shutdown();
    }

    #[test]
    fn spec101_t3_seed_ne_consomme_pas_la_fin_future_du_tour_actif() {
        let (mut worker, _peer) = worker099();
        let mut detail = contract::parse_thread_detail(
            r#"{"thread":{
            "messages":[{"id":"u","role":"user","streaming":false,"createdAt":"date"},
                {"id":"a","role":"assistant","streaming":true,"turnId":"t"}],
            "latestTurn":{"turnId":"t","state":"running","requestedAt":"date"},"activities":[]}}"#,
        )
        .unwrap();
        worker.project_journal(&detail, &summary(Some(("t", "running"))));
        assert!(!worker.state.ended_turns.contains(&"t".to_string()));
        detail.latest_turn.as_mut().unwrap().state = "completed".into();
        worker.project_journal(&detail, &summary(Some(("t", "completed"))));
        worker.journal.stop();
        let entries = std::fs::read_dir(&worker.journal_dir)
            .unwrap()
            .flat_map(|path| bridget_transport::journal::valid_events(&path.unwrap().path()))
            .collect::<Vec<_>>();
        assert_eq!(
            entries
                .iter()
                .filter(|v| v["payload"]["stop_reason"] == "completed")
                .count(),
            1
        );
        worker.relay.shutdown();
    }

    // Réutilise LinkWorker, Session et socketpair : aucun daemon ou compte réel.
    #[test]
    fn spec101_t3_seed_ignore_fin_historique_sans_texte() {
        let (mut worker, _peer) = worker099();
        let detail = contract::parse_thread_detail(
            r#"{"thread":{
            "messages":[{"id":"u","role":"user","streaming":false,"createdAt":"date"}],
            "latestTurn":{"turnId":"t","state":"error","requestedAt":"date"},"activities":[]}}"#,
        )
        .unwrap();
        let summary = summary(Some(("t", "error")));
        worker.project_journal(&detail, &summary);
        worker.project_journal(&detail, &summary);
        worker.journal.stop();
        let entries = std::fs::read_dir(&worker.journal_dir)
            .unwrap()
            .flat_map(|path| bridget_transport::journal::valid_events(&path.unwrap().path()))
            .collect::<Vec<_>>();
        assert!(!entries.iter().any(|v| v["event"] == "turn_end"));
        worker.relay.shutdown();
    }

    fn worker099() -> (LinkWorker, UnixStream) {
        let root = std::env::temp_dir().join(format!("t3-spec099-{}", uuid::Uuid::new_v4()));
        let paths = Paths::at(root.clone(), root.join("bridget.sock"));
        paths.ensure().unwrap();
        let runtime = ServerRuntime {
            port: 1,
            pid: std::process::id(),
        };
        let session = Arc::new(Session::new(
            &paths,
            runtime,
            TokenFile {
                installation_id: "test".into(),
                label: "test".into(),
                session_id: "test".into(),
                token: "synthetic".into(),
                expires_at: String::new(),
                issued_at: String::new(),
            },
        ));
        let (stream, peer) = UnixStream::pair().unwrap();
        let worker = LinkWorker::new(
            &paths,
            session,
            &summary(None),
            stable_uuid("test"),
            stable_uuid("instance:test"),
            BufWriter::new(stream.try_clone().unwrap()),
            stream,
        )
        .unwrap();
        (worker, peer)
    }

    #[test]
    fn spec101_http_capacites_honnetes_et_lacune_signalee() {
        use bridget_transport::protocol::ObservationKind as Kind;
        for provider in ["codex", "claudeAgent"] {
            let (mut worker, peer) = worker099();
            worker.state.seeded = true;
            worker.state.observations_seeded = true;
            worker
                .state
                .seen_activities
                .push("sortie-de-la-fenetre".into());
            worker.observation_events = vec![Kind::TurnEnded];
            let server = http_once099(&worker, 200, serde_json::json!({"thread":{
                "messages":[{"id":"u","role":"user","streaming":false,"createdAt":"date"}],
                "latestTurn":{"turnId":"t","state":"running","requestedAt":"date"},
                "activities":[{"id":"nouvelle","kind":"approval.requested","turnId":"t","payload":{}}]
            }}).to_string());
            let mut summary = summary(Some(("t", "running")));
            summary.provider_instance_id = Some(provider.into());
            assert!(worker.refresh(&summary));
            assert!(server.join().unwrap().starts_with("GET "));
            assert!(worker.observation_events.contains(&Kind::TurnEnded));
            assert!(
                worker
                    .observation_events
                    .contains(&Kind::PermissionRequired)
            );
            assert_eq!(
                worker.observation_events.contains(&Kind::FileWritten),
                provider == "codex"
            );
            peer.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
            let mut reader = BufReader::new(peer);
            let mut saw_capabilities = false;
            loop {
                let mut line = String::new();
                assert!(reader.read_line(&mut line).unwrap() > 0);
                match decode::<WrapperToDaemon>(line.trim()).unwrap() {
                    WrapperToDaemon::ObservationCapabilities { events } => {
                        assert!(!events.is_empty(), "pas de fausse déconnexion");
                        saw_capabilities = true;
                    }
                    WrapperToDaemon::ObservationGap { dropped } => {
                        assert_eq!(dropped, 0, "ne pas quantifier une perte inconnue");
                        assert!(saw_capabilities);
                        break;
                    }
                    _ => {}
                }
            }
            worker.journal.stop();
            worker.relay.shutdown();
        }
    }

    #[test]
    fn spec099_reponse_non_acquittee_et_autres_attentes_survivent() {
        let (mut worker, _peer) = worker099();
        let first = pending("uA", None);
        let mut second = pending("uB", None);
        second.request_id = "other".into();
        worker.state.pending = vec![first, second];
        let detail = ThreadDetail {
            activities: Vec::new(),
            activities_available: false,
            messages: vec![
                message("uA", "user", "nous", None, false),
                message("aA", "assistant", "réponse durable", Some("tA"), false),
            ],
            latest_turn: None,
        };
        worker.settle_pending(&detail, &summary(Some(("tA", "completed"))));
        let saved = ThreadState::load(&worker.state_path);
        assert_eq!(
            saved.pending.len(),
            2,
            "aucune attente ne disparaît avant confirmation"
        );
        assert!(
            serde_json::to_string(&saved)
                .unwrap()
                .contains("réponse durable")
        );
        worker.handle_frame(DaemonToWrapper::Nack {
            id: reply_id("req"),
            reason: "absent".into(),
        });
        assert_eq!(worker.state.pending.len(), 2);
        worker.handle_frame(DaemonToWrapper::RequestList { requests: vec![] });
        assert_eq!(
            worker.state.pending.len(),
            2,
            "une liste bornée vide ne prouve rien"
        );
        worker.handle_frame(DaemonToWrapper::Ack {
            id: reply_id("req"),
        });
        assert_eq!(ThreadState::load(&worker.state_path).pending.len(), 1);
        worker.journal.stop();
        worker.relay.shutdown();
    }

    #[test]
    fn spec099_journal_conserve_dix_mille_caracteres_unicode() {
        let (mut worker, _peer) = worker099();
        worker.state.seeded = true;
        let text = "é🦀漢".repeat(4000);
        let detail = ThreadDetail {
            activities: Vec::new(),
            activities_available: false,
            messages: vec![message("long", "assistant", &text, Some("t"), false)],
            latest_turn: None,
        };
        worker.project_journal(&detail, &summary(None));
        worker.journal.stop();
        let root = worker
            .state_path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let folder = root.join("sessions").join(&worker.agent_id);
        let events: Vec<_> = std::fs::read_dir(folder)
            .unwrap()
            .flat_map(|p| bridget_transport::journal::valid_events(&p.unwrap().path()))
            .collect();
        assert!(
            events.iter().any(|v| v["payload"]["text"] == text),
            "sortie intégrale"
        );
        worker.relay.shutdown();
    }

    #[test]
    fn spec099_journal_refuse_n_avance_pas_le_repere() {
        let (mut worker, _peer) = worker099();
        worker.state.seeded = true;
        worker.journal.stop();
        let detail = ThreadDetail {
            activities: Vec::new(),
            activities_available: false,
            messages: vec![message("lost", "assistant", "texte", Some("t"), false)],
            latest_turn: None,
        };
        worker.project_journal(&detail, &summary(None));
        assert!(!worker.state.seen.iter().any(|id| id == "lost"));
        assert!(!worker.state.ended_turns.iter().any(|id| id == "t"));
        worker.relay.shutdown();
    }

    /// Session 112 : une échéance de tour posée par le daemon est un budget
    /// d'exécution ; une longue attente en file ne l'entame pas.
    #[test]
    fn spec112_attente_en_file_ne_consomme_pas_le_budget_de_tour() {
        let (mut worker, _peer) = worker099();
        let mut message = bridget_core::BridgetMessage::new("alice", "bob", "budget");
        message.id = "budget-45min".into();
        message.deadline_at = Some(unix_now() + 2_700);
        worker.handle_frame(DaemonToWrapper::Deliver(message));
        // Antidater la réception au-delà du budget : sans report, la remise
        // serait périmée ; avec report, elle attend toujours.
        if let Some(entry) = worker.queue.front_mut() {
            entry.1 = Instant::now() - Duration::from_secs(3_600);
        }
        let (_tx, inbox) = mpsc::channel();
        worker.drive_queue(&inbox);
        assert_eq!(
            worker.queue.len(),
            1,
            "le budget court à partir du dispatch"
        );
        let received = worker.queue[0].1;
        let reporte = deadline_after_wait(unix_now() + 2_700 - 3_600, received);
        assert!(
            reporte >= unix_now() + 2_700 - 2,
            "report de l'attente sur l'échéance : {reporte}"
        );
    }

    /// Une remise déjà périmée à sa réception n'est pas ressuscitée par le report.
    #[test]
    fn spec112_echeance_deja_passee_a_la_reception_reste_ecartee() {
        let (mut worker, _peer) = worker099();
        let mut message = bridget_core::BridgetMessage::new("alice", "bob", "stale");
        message.id = "stale".into();
        message.deadline_at = Some(unix_now() - 10);
        worker.handle_frame(DaemonToWrapper::Deliver(message));
        let (_tx, inbox) = mpsc::channel();
        worker.drive_queue(&inbox);
        assert!(worker.queue.is_empty());
    }

    /// Session 111 : un message sans échéance attend la fin du tour, même
    /// bien après l'ancien délai de deux minutes qui le jetait.
    #[test]
    fn spec111_remise_sans_echeance_survit_a_une_longue_attente() {
        let (mut worker, _peer) = worker099();
        let mut message = bridget_core::BridgetMessage::new("alice", "bob", "libération");
        message.id = "attente-longue".into();
        worker.handle_frame(DaemonToWrapper::Deliver(message));
        // Antidater la réception bien au-delà de l'ancien seuil.
        let ancien_seuil = worker.turn_wait * 10;
        if let Some(entry) = worker.queue.front_mut() {
            entry.1 = Instant::now() - ancien_seuil;
        }
        let (_tx, inbox) = mpsc::channel();
        worker.drive_queue(&inbox);
        assert_eq!(
            worker.queue.len(),
            1,
            "une remise sans échéance ne périme pas sur la seule attente"
        );
        assert_eq!(delivery_message(&worker.queue[0].0).id, "attente-longue");
    }

    /// Une échéance réellement portée par le message reste respectée.
    #[test]
    fn spec111_remise_avec_echeance_depassee_est_toujours_ecartee() {
        let (mut worker, _peer) = worker099();
        let mut message = bridget_core::BridgetMessage::new("alice", "bob", "périmée");
        message.id = "echeance-depassee".into();
        message.deadline_at = Some(unix_now() - 1);
        worker.handle_frame(DaemonToWrapper::Deliver(message));
        let (_tx, inbox) = mpsc::channel();
        worker.drive_queue(&inbox);
        assert!(
            worker.queue.is_empty(),
            "une échéance dépassée écarte toujours la remise"
        );
    }

    /// La borne de file remplace l'ancien garde-fou temporel : au-delà, la plus
    /// ancienne est écartée, et la file cesse de grandir.
    #[test]
    fn spec111_file_saturee_ecarte_la_plus_ancienne() {
        let (mut worker, _peer) = worker099();
        for index in 0..(QUEUE_BOUND + 5) {
            let mut message = bridget_core::BridgetMessage::new("alice", "bob", "lot");
            message.id = format!("m-{index}");
            worker.handle_frame(DaemonToWrapper::Deliver(message));
        }
        assert_eq!(worker.queue.len(), QUEUE_BOUND, "file bornée");
        assert_eq!(
            delivery_message(&worker.queue[0].0).id,
            "m-5",
            "les plus anciennes sont écartées, les récentes conservées"
        );
        assert_eq!(
            delivery_message(&worker.queue[QUEUE_BOUND - 1].0).id,
            format!("m-{}", QUEUE_BOUND + 4)
        );
    }

    /// Session 113 : toutes les trames émises vers le daemon, sans filtrage.
    fn frames113(peer: UnixStream) -> Vec<WrapperToDaemon> {
        peer.set_nonblocking(true).unwrap();
        let mut reader = BufReader::new(peer);
        let mut frames = Vec::new();
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => frames.push(serde_json::from_str(&line).unwrap()),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => panic!("lecture des trames : {e}"),
            }
        }
        frames
    }

    fn idempotent113(id: &str) -> DaemonToWrapper {
        let mut message = bridget_core::BridgetMessage::new("alice", "bob", "suivie");
        message.id = id.into();
        message.reply = true;
        DaemonToWrapper::DeliverIdempotent {
            delivery_id: format!("d-{id}"),
            recipient_instance_id: stable_uuid("instance:test"),
            delivery_generation: 1,
            expires_at: unix_now() as i64 + 3600,
            message,
            execution: None,
        }
    }

    fn saga_closed113(frames: &[WrapperToDaemon], delivery_id: &str) -> bool {
        frames.iter().any(|frame| {
            matches!(
                frame,
                WrapperToDaemon::DeliveryIndeterminate { delivery_id: id, delivery_generation: 1 }
                    if id == delivery_id
            )
        })
    }

    fn rejection113<'a>(frames: &'a [WrapperToDaemon], message_id: &str) -> Option<&'a str> {
        frames.iter().find_map(|frame| match frame {
            WrapperToDaemon::DeliveryRejected { id, reason } if id == message_id => {
                Some(reason.as_str())
            }
            _ => None,
        })
    }

    /// Une remise poussée hors de la file par saturation n'est plus « en vol » :
    /// la saga est close et l'expéditeur d'une demande suivie est prévenu.
    #[test]
    fn spec113_saturation_signale_la_remise_ecartee_au_daemon() {
        let (mut worker, peer) = worker099();
        worker.handle_frame(idempotent113("premiere"));
        for index in 0..QUEUE_BOUND {
            let mut message = bridget_core::BridgetMessage::new("alice", "bob", "lot");
            message.id = format!("m-{index}");
            worker.handle_frame(DaemonToWrapper::Deliver(message));
        }
        assert_eq!(worker.queue.len(), QUEUE_BOUND);
        let frames = frames113(peer);
        assert!(saga_closed113(&frames, "d-premiere"), "saga close");
        assert!(
            rejection113(&frames, "premiere").is_some_and(|reason| reason.contains("saturée")),
            "expéditeur prévenu avec le motif"
        );
    }

    /// Une échéance passée avant démarrage vaut refus explicite : le motif
    /// porte « échéance », ce sur quoi le daemon clôt la demande suivie.
    #[test]
    fn spec113_echeance_passee_signale_refus_et_saga_close() {
        let (mut worker, peer) = worker099();
        let mut frame = idempotent113("perimee");
        if let DaemonToWrapper::DeliverIdempotent { message, .. } = &mut frame {
            message.deadline_at = Some(unix_now() - 1);
        }
        worker.handle_frame(frame);
        let (_tx, inbox) = mpsc::channel();
        worker.drive_queue(&inbox);
        assert!(worker.queue.is_empty());
        let frames = frames113(peer);
        assert!(saga_closed113(&frames, "d-perimee"), "saga close");
        assert!(
            rejection113(&frames, "perimee").is_some_and(|reason| reason.contains("échéance")),
            "refus porteur du motif d'échéance"
        );
    }

    /// Une demande déjà close côté daemon (répondue, annulée, expirée) a déjà
    /// son issue : la saga est close, aucun second échec n'est fabriqué.
    #[test]
    fn spec113_demande_close_clot_la_saga_sans_second_echec() {
        let (mut worker, peer) = worker099();
        worker.handle_frame(idempotent113("close"));
        worker.handle_frame(idempotent113("annulee"));
        worker.handle_frame(DaemonToWrapper::RequestList {
            requests: vec![RequestInfo {
                id: "close".into(),
                sender: "alice".into(),
                target: worker.agent_id.clone(),
                state: "answered".into(),
                created_at: 0,
                deadline_at: i64::MAX,
                cancel_reason: None,
                deferred_reminder_level: None,
                deferred_reminder_at: None,
            }],
        });
        worker.handle_frame(DaemonToWrapper::CancelDelivery {
            id: "annulee".into(),
            reason: "test".into(),
        });
        assert!(worker.queue.is_empty());
        let frames = frames113(peer);
        assert!(
            saga_closed113(&frames, "d-close"),
            "saga de la demande répondue close"
        );
        assert!(
            saga_closed113(&frames, "d-annulee"),
            "saga de la demande annulée close"
        );
        assert!(rejection113(&frames, "close").is_none());
        assert!(rejection113(&frames, "annulee").is_none());
    }

    #[test]
    fn spec099_controles_retirent_seulement_la_demande_annulee() {
        let (mut worker, _peer) = worker099();
        let mut cancelled = bridget_core::BridgetMessage::new("alice", "bob", "annulée");
        cancelled.id = "cancelled".into();
        let mut next = cancelled.clone();
        next.id = "next".into();
        let started = Instant::now();
        worker.handle_frame(DaemonToWrapper::Deliver(cancelled));
        worker.handle_frame(DaemonToWrapper::Deliver(next));
        worker.handle_frame(DaemonToWrapper::CancelDelivery {
            id: "cancelled".into(),
            reason: "test".into(),
        });
        assert!(
            started.elapsed() < Duration::from_millis(100),
            "aucune attente fournisseur dans le traitement des contrôles"
        );
        assert_eq!(worker.queue.len(), 1);
        assert_eq!(delivery_message(&worker.queue[0].0).id, "next");
        worker.handle_frame(DaemonToWrapper::Disconnect);
        let (_tx, inbox) = mpsc::channel();
        worker.drive_queue(&inbox);
        assert!(!worker.daemon_alive.load(Ordering::SeqCst));
        assert!(
            worker.state.pending.is_empty(),
            "aucune acceptation après disparition de l'autorité"
        );
        worker.journal.stop();
        worker.relay.shutdown();
    }

    #[test]
    fn spec099_peremption_demande_et_tour_avant_tout_http() {
        let (mut worker, _peer) = worker099();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        worker.session.reset_runtime(ServerRuntime {
            port: listener.local_addr().unwrap().port(),
            pid: std::process::id(),
        });
        let mut message = bridget_core::BridgetMessage::new("alice", "bob", "périmée");
        message.deadline_at = Some(unix_now().saturating_sub(1));
        worker.handle_frame(DaemonToWrapper::Deliver(message.clone()));
        let (_tx, inbox) = mpsc::channel();
        worker.drive_queue(&inbox);
        assert!(worker.queue.is_empty());
        message.deadline_at = Some(unix_now() + 1000);
        message.reply_timeout = Some(0);
        worker.handle_frame(DaemonToWrapper::Deliver(message.clone()));
        worker.drive_queue(&inbox);
        assert!(
            worker.queue.is_empty(),
            "l'échéance demande peut précéder le tour"
        );
        message.reply_timeout = None;
        worker.handle_frame(DaemonToWrapper::Deliver(message.clone()));
        worker.requests.insert(
            message.id.clone(),
            RequestInfo {
                id: message.id,
                sender: "alice".into(),
                target: worker.agent_id.clone(),
                state: "open".into(),
                created_at: unix_now() as i64 - 100,
                deadline_at: unix_now() as i64 - 1,
                cancel_reason: None,
                deferred_reminder_level: None,
                deferred_reminder_at: None,
            },
        );
        worker.drive_queue(&inbox);
        assert!(worker.queue.is_empty());
        assert!(
            matches!(listener.accept(), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock),
            "aucun appel fournisseur pour un travail périmé"
        );
        worker.journal.stop();
        worker.relay.shutdown();
    }

    #[test]
    fn spec099_journal_echec_apres_enqueue_ne_confirme_rien() {
        let (mut worker, _peer) = worker099();
        worker.state.seeded = true;
        let date = &contract::iso_now()[..10];
        // SessionJournal ouvre son fichier lors du premier append : rendre
        // ce chemin inouvrable provoque un échec réel APRÈS admission en file.
        std::fs::create_dir(worker.journal_dir.join(format!("{date}.jsonl"))).unwrap();
        let detail = ThreadDetail {
            activities: Vec::new(),
            activities_available: false,
            messages: vec![message("failed", "assistant", "texte", Some("t"), false)],
            latest_turn: None,
        };
        worker
            .journal
            .enqueue(
                "update",
                Some("failed"),
                serde_json::json!({"kind": "text", "text": "texte"}),
            )
            .unwrap();
        worker.journal.stop();
        assert!(worker.journal_failed.load(Ordering::SeqCst));
        worker.project_journal(&detail, &summary(None));
        worker.confirm_journal();
        assert!(!worker.state.seen.contains(&"failed".to_string()));
        assert!(
            !ThreadState::load(&worker.state_path)
                .seen
                .contains(&"failed".to_string())
        );
        worker.relay.shutdown();
    }

    #[test]
    fn spec099_reprise_journal_apres_flush_sans_curseur_ne_duplique_pas() {
        let (mut worker, _peer) = worker099();
        worker.state.seeded = true;
        let detail = ThreadDetail {
            activities: Vec::new(),
            activities_available: false,
            messages: vec![message(
                "once",
                "assistant",
                "é🦀".repeat(6000).as_str(),
                Some("t"),
                false,
            )],
            latest_turn: None,
        };
        worker.project_journal(&detail, &summary(None));
        worker.journal.stop();
        assert!(
            !worker.state.seen.contains(&"once".to_string()),
            "enqueue seul ne confirme pas"
        );
        // Simuler le redémarrage entre append et conservation du curseur.
        worker.journal_inflight.clear();
        worker.journal_readers.clear();
        worker.project_journal(&detail, &summary(None));
        assert!(worker.state.seen.contains(&"once".to_string()));
        assert!(worker.journal_inflight.is_empty(), "rien à réémettre");
        let events: Vec<_> = std::fs::read_dir(&worker.journal_dir)
            .unwrap()
            .flat_map(|p| bridget_transport::journal::valid_events(&p.unwrap().path()))
            .collect();
        assert_eq!(
            events
                .iter()
                .filter(|v| v["event"] == "update" && v["message_id"] == "once")
                .count(),
            1
        );
        worker.relay.shutdown();
    }

    #[test]
    fn spec099_reponse_rechargee_sans_reexecution_et_ack_perdu_reconcilie() {
        let (mut worker, _peer) = worker099();
        let mut ready = pending("uA", None);
        ready.response = Some("déjà produite".into());
        worker.state.pending.push(ready);
        worker.state.save(&worker.state_path).unwrap();
        worker.state = ThreadState::load(&worker.state_path);
        worker.send_ready_responses();
        assert_eq!(
            worker.state.pending.len(),
            1,
            "l'envoi ne vaut pas acquittement"
        );
        worker.handle_frame(DaemonToWrapper::RequestList {
            requests: vec![RequestInfo {
                id: "req".into(),
                sender: "alice".into(),
                target: worker.agent_id.clone(),
                state: "answered".into(),
                created_at: 0,
                deadline_at: i64::MAX,
                cancel_reason: None,
                deferred_reminder_level: None,
                deferred_reminder_at: None,
            }],
        });
        assert!(ThreadState::load(&worker.state_path).pending.is_empty());
        assert!(
            worker.queue.is_empty(),
            "la reprise de réponse ne crée aucune commande fournisseur"
        );
        worker.journal.stop();
        worker.relay.shutdown();
    }

    #[test]
    fn spec099_ancien_etat_pending_reste_lisible() {
        let state: ThreadState = serde_json::from_value(serde_json::json!({"pending": [{
            "request_id": "legacy", "from": "alice", "message_id": "u", "anchor_turn_id": null,
            "dispatched_at": "2026-09-16T00:00:00Z", "attempts": 0
        }]}))
        .unwrap();
        assert!(state.pending[0].response.is_none());
    }

    #[test]
    fn spec099_annulation_signalee_par_lecteur_interdit_la_frontiere_http() {
        let (mut worker, _peer) = worker099();
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        worker.session.reset_runtime(ServerRuntime {
            port: listener.local_addr().unwrap().port(),
            pid: std::process::id(),
        });
        let message =
            bridget_core::BridgetMessage::new("alice", &worker.agent_id, "ne démarre pas");
        // Le lecteur a reçu CancelDelivery mais sa trame n'a pas encore été
        // traitée par le worker occupé à persister une remise.
        worker
            .cancelled
            .lock()
            .unwrap()
            .insert(message.id.clone(), Instant::now());
        assert!(
            worker
                .dispatch_with_id(&message, None, &summary(None))
                .is_err()
        );
        assert!(matches!(listener.accept(), Err(e) if e.kind() == std::io::ErrorKind::WouldBlock));
        worker.journal.stop();
        worker.relay.shutdown();
    }

    #[test]
    fn spec099_journal_hors_borne_rend_une_lacune_explicite() {
        let (mut worker, _peer) = worker099();
        worker.state.seeded = true;
        // Les contrôles s'échappent sur six octets en JSON : la limite doit
        // porter sur la ligne sérialisée, pas la longueur du texte source.
        let detail = ThreadDetail {
            activities: Vec::new(),
            activities_available: false,
            messages: vec![message(
                "oversized",
                "assistant",
                &"\u{1}".repeat(800_000),
                None,
                false,
            )],
            latest_turn: None,
        };
        worker.project_journal(&detail, &summary(None));
        worker.journal.stop();
        worker.confirm_journal();
        let events: Vec<_> = std::fs::read_dir(&worker.journal_dir)
            .unwrap()
            .flat_map(|p| bridget_transport::journal::valid_events(&p.unwrap().path()))
            .collect();
        assert!(
            events
                .iter()
                .any(|v| v["event"] == "error" && v["payload"]["gap"] == true)
        );
        assert!(!events.iter().any(|v| v["event"] == "update"));
        assert!(
            worker.state.seen.contains(&"oversized".to_string()),
            "la lacune n'est confirmée qu'après append"
        );
        worker.relay.shutdown();
    }

    // Aucun équivalent HTTP dans ce module : un seul échange sur boucle
    // locale, délai d'attente borné, aucun sous-processus ni fournisseur réel.
    fn http_once099(worker: &LinkWorker, status: u16, body: String) -> thread::JoinHandle<String> {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        worker.session.reset_runtime(ServerRuntime {
            port: listener.local_addr().unwrap().port(),
            pid: std::process::id(),
        });
        thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(1);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(e)
                        if e.kind() == std::io::ErrorKind::WouldBlock
                            && Instant::now() < deadline =>
                    {
                        thread::sleep(Duration::from_millis(5))
                    }
                    _ => return String::new(),
                }
            };
            stream
                .set_read_timeout(Some(Duration::from_secs(1)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut headers = String::new();
            let mut length = 0;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if let Some((name, value)) = line.split_once(':')
                    && name.eq_ignore_ascii_case("content-length")
                {
                    length = value.trim().parse().unwrap();
                }
                headers.push_str(&line);
                if line == "\r\n" || line.is_empty() {
                    break;
                }
            }
            let mut content = vec![0; length];
            reader.read_exact(&mut content).unwrap();
            write!(
                stream,
                "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
            headers + &String::from_utf8(content).unwrap()
        })
    }

    #[test]
    fn spec099_refus_publication_est_repris_sur_tick_inchange() {
        let (mut worker, _peer) = worker099();
        worker.state.seeded = true;
        let summary = summary(None);
        worker.last_key = summary.change_key();
        let detail = ThreadDetail {
            activities: Vec::new(),
            activities_available: false,
            messages: vec![message("retry", "assistant", "conservé", Some("t"), false)],
            latest_turn: None,
        };
        // Même branche Err que queueFull, sans failure sink asynchrone : cet
        // oracle n'affirme pas provoquer une saturation réelle de la file.
        worker.journal.stop();
        worker.project_journal(&detail, &summary);
        worker.confirm_journal();
        assert!(!worker.journal_failed.load(Ordering::SeqCst));
        assert!(worker.journal_inflight.is_empty());
        worker.journal = Arc::new(
            JournalWriter::start_with_live_feed_and_failure(
                worker.journal_dir.parent().unwrap(),
                &worker.agent_id,
                "retry-instance",
                Arc::new(|_| {}),
                None,
            )
            .unwrap(),
        );
        let server = http_once099(&worker, 200, serde_json::json!({"thread": {"messages": [{"id": "retry", "role": "assistant", "text": "conservé", "streaming": false, "turnId": "t"}]}}).to_string());
        worker.handle_event(LinkEvent::Tick(Box::new(summary)));
        worker.journal.stop();
        let request = server.join().unwrap();
        worker.confirm_journal();
        worker.relay.shutdown();
        assert!(
            request.starts_with("GET /api/orchestration/threads/"),
            "un refus ne fige pas la clé du fil"
        );
        assert!(worker.state.seen.contains(&"retry".to_string()));
        assert!(worker.state.ended_turns.contains(&"t".to_string()));
    }

    #[test]
    fn spec099_post_accepte_issue_illisible_conserve_la_correlation() {
        for (status, response) in [(200, "{}"), (500, "erreur après acceptation")] {
            let (mut worker, _peer) = worker099();
            let mut request =
                bridget_core::BridgetMessage::new("alice", &worker.agent_id, "travail accepté");
            request.reply = true;
            let server = http_once099(&worker, status, response.into());
            assert!(
                worker
                    .dispatch_with_id(&request, None, &summary(None))
                    .is_err()
            );
            let accepted = server.join().unwrap();
            assert!(accepted.starts_with("POST /api/orchestration/dispatch"));
            let saved = ThreadState::load(&worker.state_path);
            assert_eq!(
                saved.pending.len(),
                1,
                "une issue HTTP inconnue ne prouve pas le refus du tour"
            );
            let detail = ThreadDetail {
                activities: Vec::new(),
                activities_available: false,
                messages: vec![
                    message(&saved.pending[0].message_id, "user", "accepté", None, false),
                    message("answer", "assistant", "réponse récupérée", Some("t"), false),
                ],
                latest_turn: None,
            };
            worker.settle_pending(&detail, &summary(Some(("t", "completed"))));
            assert_eq!(
                worker.state.pending[0].response.as_deref(),
                Some("réponse récupérée")
            );
            worker.journal.stop();
            worker.relay.shutdown();
        }
    }

    fn message(id: &str, role: &str, text: &str, turn: Option<&str>, streaming: bool) -> Message {
        Message {
            created_at: None,
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
            settled_override: None,
            updated_at: "2026-09-14T00:00:00Z".to_string(),
            latest_turn: latest.map(|(id, state)| LatestTurn {
                requested_at: None,
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
            response: None,
            reply_requested: Some(true),
        }
    }

    #[test]
    fn spec098_correlation_par_rang_fifo_avec_message_humain_intercale() {
        // Ordre observé en sonde : user A (nous), user B (humain), assistant A, assistant B.
        let detail = ThreadDetail {
            activities: Vec::new(),
            activities_available: false,
            messages: vec![
                message("u0", "user", "avant", None, false),
                message("a0", "assistant", "réponse avant", Some("t0"), false),
                message("uA", "user", "nous", None, false),
                message("uB", "user", "humain", None, false),
                message("aA", "assistant", "pour nous", Some("tA"), false),
                message("aB", "assistant", "pour l'humain", Some("tB"), false),
            ],
            latest_turn: Some(LatestTurn {
                requested_at: None,
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
            activities: Vec::new(),
            activities_available: false,
            messages: vec![
                message("uA", "user", "nous", None, false),
                message("aA", "assistant", "partiel", Some("tA"), true),
            ],
            latest_turn: Some(LatestTurn {
                requested_at: None,
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
            activities: Vec::new(),
            activities_available: false,
            messages: vec![
                message("uA", "user", "nous", None, false),
                message("aA", "assistant", "complet", Some("tA"), false),
            ],
            latest_turn: Some(LatestTurn {
                requested_at: None,
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
            activities: Vec::new(),
            activities_available: false,
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
            activities: Vec::new(),
            activities_available: false,
            messages: vec![
                message("uB", "user", "nous", None, false),
                message("uH", "user", "humain", None, false),
                message("aH", "assistant", "réponse à l'humain", Some("tH"), false),
            ],
            latest_turn: Some(LatestTurn {
                requested_at: None,
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
            activities: Vec::new(),
            activities_available: false,
            messages: vec![
                message("uA", "user", "nous", None, false),
                message("aX", "assistant", "sous-agent", Some("tX"), false),
                message("aA", "assistant", "réponse", Some("tA"), false),
            ],
            latest_turn: Some(LatestTurn {
                requested_at: None,
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
    fn spec098_fil_range_n_est_pas_un_agent() {
        // t3code range les fils issus d'un import d'historique : les exposer
        // noierait l'annuaire sous des centaines de conversations passées.
        let mut range = summary(None);
        range.settled_override = Some("settled".to_string());
        assert!(!range.is_live());
        let mut repris = summary(None);
        repris.settled_override = Some("active".to_string());
        assert!(repris.is_live());
        assert!(summary(None).is_live());
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
