//! Adaptateur t3code (session 098) : `bridget t3 install|status|uninstall|serve`.
//!
//! Le pont ne modifie jamais t3code. Il lit son serveur local par HTTP en
//! boucle locale avec une session émise par le CLI officiel `t3`, présente
//! chaque fil vivant au daemon Bridget comme un agent (`t3code | cli`), remet
//! les messages par le protocole natif V1/V2 et renvoie à l'expéditeur la
//! réponse du tour attesté. Tout l'état est sous `<BRIDGET_HOME>/t3code`.

#[path = "t3code_agent_tracking230.rs"]
mod agent_tracking230;
use bridget_transport::poison::RecoverPoison;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::io::{BufRead, BufReader, BufWriter};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use bridget_transport::journal::{
    IncrementalJournalReader, JournalLiveFeed, JournalReadItem, JournalWriter,
};
use bridget_transport::protocol::{
    CommunicationProjectSource, DaemonToWrapper, PresenceMode, RequestInfo, WrapperToDaemon, decode,
};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::t3code_contract::{
    self as contract, Client, ContractError, ServerRuntime, T3Cli, ThreadDetail, ThreadSummary,
};
use crate::wrapper::{
    AttachRelayControl, AttachRelayWorker, BatchedDelivery, ConnectAndRegisterAtArgs,
    IdempotentDeliveryTracker, connect_and_register_at, deliver_batch_to_interactive,
    send_wrapper_message,
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
/// Session 117 : durée au-delà de laquelle un blocage du journal est signalé.
const JOURNAL_BLOCK_GRACE: Duration = Duration::from_secs(30);
/// Session 114 : bornes du regroupement des messages sans réponse attendue en
/// un seul tour. Un lot doit rester lisible par son destinataire et tenir dans
/// le budget d'un tour ; au-delà, les messages suivants attendent le tour d'après.
const BATCH_BOUND: usize = 8;
const BATCH_BODY_BOUND: usize = 32 * 1024;
const RECONNECT_BACKOFF: Duration = Duration::from_secs(5);
const AUTH_FAILED_PAUSE: Duration = Duration::from_secs(60);
const DETAIL_PAGE: u32 = 20;
const DETAIL_PAGE_MAX: u32 = 320;
const SEEN_BOUND: usize = 1000;
/// Une reprise durable ne doit jamais recréer le pic de descripteurs qu'elle
/// répare. Les obligations suivantes restent planifiées pour un cycle futur.
const DRAIN_REOPEN_BOUND: usize = 32;
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
        // Le nom borné ne remplace jamais l'identifiant T3 dans le protocole.
        self.threads()
            .join("thread-states-v1")
            .join(format!("{}.json", stable_uuid(thread_id)))
    }
    fn load_thread_state(&self, thread_id: &str) -> ThreadState {
        let path = self.thread_state(thread_id);
        // Seule l'absence autorise l'ancien état ; un nouveau fichier invalide
        // ou inaccessible ne doit pas faire ressurgir un ancien curseur.
        if !matches!(std::fs::symlink_metadata(&path), Err(error) if error.kind() == std::io::ErrorKind::NotFound)
        {
            return ThreadState::load(&path);
        }
        if !thread_id.is_empty()
            && thread_id.len() <= 250 // Extension `.json` comprise : 255 octets.
            && !thread_id.contains(['/', '\\'])
            && !thread_id.chars().any(bridget_core::is_disallowed_control)
        {
            return ThreadState::load(&self.threads().join(format!("{thread_id}.json")));
        }
        ThreadState::default()
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
    #[serde(default)]
    pub tool_progress_seeded: bool,
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

    fn has_drain_obligation(&self, summary: &ThreadSummary) -> bool {
        if !self.pending.is_empty() {
            return true;
        }
        summary.latest_turn.as_ref().is_some_and(|turn| {
            turn_is_final(&turn.state)
                && self.turn_origins.contains_key(&turn.turn_id)
                && !self.ended_turns.contains(&turn.turn_id)
        })
    }

    fn remember(&mut self, message_id: &str) {
        self.seen.push(message_id.to_string());
    }

    /// Au-delà de SEEN_BOUND, oublie seulement ce que T3 ne montre plus : cela
    /// seul ne peut pas revenir. Session 118 : une éviction des plus anciens,
    /// sur un fil de 1 363 messages, les faisait recopier à chaque lecture
    /// (jusqu'à 131 fois chacun). O(S + M + A).
    fn forget_invisible(&mut self, detail: &ThreadDetail) {
        if self.seen.len() <= SEEN_BOUND {
            return;
        }
        let visible: HashSet<&str> = detail
            .messages
            .iter()
            .map(|m| m.id.as_str())
            .chain(detail.activities.iter().map(|a| a.id.as_str()))
            .collect();
        self.seen.retain(|id| {
            visible.contains(id.as_str())
                || id
                    .strip_prefix("t3-activity:")
                    .or_else(|| id.strip_prefix("bridget-observation:"))
                    .and_then(|rest| rest.rsplit_once(':'))
                    .is_some_and(|(activity, _)| visible.contains(activity))
        });
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
    let bridge = read_json::<StatusFile>(&paths.status())?;
    if let Some(bridge) = &bridge {
        println!(
            "pont          : {} — {} ({} fil(s), {})",
            bridge.state, bridge.detail, bridge.threads, bridge.updated_at
        );
    } else {
        println!("pont          : jamais démarré");
    }
    // Session 114 : diagnostic explicite du partage de dossier T3. Sans lui, la
    // divergence entre ces deux lignes se lit à l'œil — et c'est par là que
    // toutes les identités MCP peuvent tomber.
    if let Some(bridge) = bridge.filter(|bridge| bridge.state == "running") {
        match &runtime {
            Ok(runtime) if !bridge.detail.is_empty() && bridge.detail != runtime.base_url() => {
                println!(
                    "ATTENTION     : le pont sert {} mais le fichier d'état désigne {} ; \
                     deux applications T3 partagent le même dossier. N'en garder qu'une.",
                    bridge.detail,
                    runtime.base_url()
                );
            }
            Err(_) => println!(
                "ATTENTION     : aucun serveur t3code déclaré alors que le pont sert {} ; \
                 une autre application T3 a effacé le fichier d'état en se fermant. \
                 Le pont continue, mais son prochain démarrage ne trouvera rien.",
                bridge.detail
            ),
            _ => {}
        }
    }
    if let (Ok(runtime), Some(token)) = (runtime, token) {
        match Client::new(&runtime, &token.token).snapshot() {
            Ok(snapshot) => {
                for thread in snapshot
                    .threads
                    .iter()
                    .filter(|thread| thread_requires_link(thread, false))
                {
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
            // SAFETY: getuid ne déréférence aucun pointeur ; ses arguments sont des entiers.
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
            // SAFETY: getuid ne déréférence aucun pointeur ; ses arguments sont des entiers.
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
        self.inner.lock().recover_poison()
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

    /// La remise longue ne garde pas le mutex des lectures. Chaque fil a
    /// déjà sa file bornée et n'exécute qu'une remise à la fois.
    fn call_detached<T>(
        &self,
        call: impl Fn(&Client) -> Result<T, ContractError>,
    ) -> Result<T, ContractError> {
        let (generation, runtime, client) = {
            let inner = self.lock();
            (
                inner.generation,
                inner.runtime.clone(),
                inner.client.clone(),
            )
        };
        let birth = crate::managed_process::process_birth(runtime.pid)
            .map_err(|_| ContractError::Transport("rpc_runtime_identity_unavailable".into()))?;
        let unchanged = || {
            self.lock().runtime == runtime
                && crate::managed_process::process_birth(runtime.pid).ok() == Some(birth)
        };
        let first = call(&client);
        if !unchanged() {
            return Err(ContractError::Transport(
                "rpc_runtime_changed_effect_unknown".into(),
            ));
        }
        if !matches!(first, Err(ContractError::Unauthorized)) {
            return first;
        }
        // Seul 401 conserve le renouvellement unique déjà prévu. Aucune
        // coupure ni expiration ne provoque une nouvelle remise ici.
        self.renew(generation)?;
        let (generation, client) = {
            let inner = self.lock();
            if inner.runtime != runtime {
                return Err(ContractError::Transport(
                    "rpc_runtime_changed_effect_unknown".into(),
                ));
            }
            (inner.generation, inner.client.clone())
        };
        let retried = call(&client);
        if !unchanged() {
            return Err(ContractError::Transport(
                "rpc_runtime_changed_effect_unknown".into(),
            ));
        }
        if matches!(retried, Err(ContractError::Unauthorized)) {
            let mut inner = self.lock();
            if inner.generation == generation {
                inner.auth_failed = true;
            }
        }
        retried
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

/// Session 114 : signale une seconde application T3 sur le même dossier
/// utilisateur, une fois par changement.
///
/// Les deux applications partagent `server-runtime.json` et `state.sqlite` : la
/// dernière démarrée s'y déclare. Nos attestations restent valides — elles ne
/// dépendent que de NOTRE serveur — mais le partage de base doit être dit, et
/// l'autre application efface ce fichier en se fermant, ce qui empêchera un
/// prochain démarrage du pont de retrouver le serveur.
fn report_foreign_runtime(
    base: &Path,
    ours: &ServerRuntime,
    declaration: Option<&str>,
    last: &mut Option<u32>,
) {
    if let Some(declaration) = declaration
        && restore_declaration(base, declaration, || server_alive(ours))
    {
        info!(
            "fichier d'état T3 restauré pour notre serveur {} (pid {}) : une autre \
             application l'avait effacé en se fermant",
            ours.base_url(),
            ours.pid
        );
    }
    let foreign = match contract::read_runtime(base) {
        Ok(other) if other != *ours => Some(other),
        _ => None,
    };
    match foreign {
        Some(other) if *last != Some(other.pid) => {
            let app = app_bundle(other.pid).unwrap_or_else(|| format!("pid {}", other.pid));
            notify_user(&format!(
                "Une deuxième application T3 est ouverte : {app}. Elle partage vos fils \
                 et casse les tours en cours. Quittez-la (Cmd+Q) et gardez T3 Code (Local)."
            ));
            warn!(
                "une autre application T3 (pid {}, {}) s'est déclarée dans {} ; \
                 la nôtre reste {} (pid {}). Les deux partagent state.sqlite : \
                 n'en laisser tourner qu'une.",
                other.pid,
                other.base_url(),
                contract::runtime_file(base).display(),
                ours.base_url(),
                ours.pid
            );
            *last = Some(other.pid);
        }
        None if last.is_some() => {
            info!(
                "plus qu'une application T3 déclarée ; la nôtre reste {}",
                ours.base_url()
            );
            *last = None;
        }
        _ => {}
    }
}

/// Session 120 : l'autre application efface `server-runtime.json` en se
/// fermant ; le 25/09, un agent a dû le réécrire à la main. Rétablit notre
/// déclaration, telle que lue au démarrage, si le fichier manque ou désigne un
/// serveur mort, et seulement si notre serveur répond encore.
fn restore_declaration(base: &Path, declaration: &str, ours_alive: impl FnOnce() -> bool) -> bool {
    if contract::read_runtime(base).is_ok() || !ours_alive() {
        return false;
    }
    match private_write(&contract::runtime_file(base), declaration) {
        Ok(()) => true,
        Err(e) => {
            warn!("fichier d'état T3 non restauré : {e}");
            false
        }
    }
}

fn server_alive(ours: &ServerRuntime) -> bool {
    // SAFETY: kill(pid, 0) est l'idiome POSIX de sonde d'existence ; aucun signal n'est envoyé.
    let alive = unsafe { libc::kill(ours.pid as i32, 0) == 0 };
    alive
        && std::net::TcpStream::connect_timeout(
            &std::net::SocketAddr::from(([127, 0, 0, 1], ours.port)),
            Duration::from_secs(1),
        )
        .is_ok()
}

/// Nom du paquet `.app` d'un processus : les deux applications T3 portent le
/// même nom de processus, seul le chemin permet de dire laquelle quitter.
#[cfg(target_os = "macos")]
fn app_bundle(pid: u32) -> Option<String> {
    let mut buffer = vec![0u8; libc::PROC_PIDPATHINFO_MAXSIZE as usize];
    let length =
        // SAFETY: invariant non établi par lecture locale ; revue demandée (programme 260).
        unsafe { libc::proc_pidpath(pid as i32, buffer.as_mut_ptr().cast(), buffer.len() as u32) };
    if length <= 0 {
        return None;
    }
    buffer.truncate(length as usize);
    let path = String::from_utf8(buffer).ok()?;
    Path::new(&path)
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .find(|c| c.ends_with(".app"))
        .map(str::to_string)
}

#[cfg(not(target_os = "macos"))]
fn app_bundle(_pid: u32) -> Option<String> {
    None
}

/// Alerte visible hors des journaux : le dommage (tours marqués en échec)
/// survient dans les secondes qui suivent l'ouverture du doublon, seul un
/// humain peut le fermer. Jamais pendant les tests.
fn notify_user(text: &str) {
    if cfg!(test) || std::env::var_os("BRIDGET_T3_NO_NOTIFY").is_some() {
        return;
    }
    let script = format!(
        "display notification \"{}\" with title \"Bridget\" sound name \"Basso\"",
        text.replace(['\\', '"'], "'")
    );
    thread::spawn(move || {
        let _ = std::process::Command::new("/usr/bin/osascript")
            .args(["-e", &script])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    });
}

const SNAPSHOT_TIMEOUT_GRACE: Duration = Duration::from_secs(90);

/// Une lecture lente ne prouve pas la disparition du serveur. La grâce ne
/// conserve que les liens, jamais une observation ni un droit mis en cache.
fn retain_links_after_timeout(
    detail: &str,
    last_success: Option<(Instant, ServerRuntime, u64)>,
    observed: Option<(ServerRuntime, u64)>,
    now: Instant,
) -> bool {
    matches!(detail, "http_timeout" | "rpc_response_timeout")
        && last_success
            .zip(observed)
            .is_some_and(|((at, runtime, birth), current)| {
                now.saturating_duration_since(at) < SNAPSHOT_TIMEOUT_GRACE
                    && current == (runtime, birth)
            })
}

fn runtime_incarnation(base: &Path) -> Option<(ServerRuntime, u64)> {
    let runtime = contract::read_runtime(base).ok()?;
    let birth = crate::managed_process::process_birth(runtime.pid).ok()?;
    Some((runtime, birth))
}

fn serve(paths: &Paths) -> Result<(), String> {
    crate::service_limits::ensure_open_file_limit()?;
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
    let mut obligations: HashSet<String> = HashSet::new();
    let mut attempted_obligations: HashSet<String> = HashSet::new();
    let mut drain_reopens: HashSet<String> = HashSet::new();
    let mut obligation_scan_required = true;
    let mut last_relation_counts: Option<RelationCounts> = None;
    let t3_base = contract::base_dir().map_err(|e| e.to_string())?;
    let mut identities = crate::t3code_identity::IdentityBindings::new(&paths.root);
    let mut identity_error: Option<String> = None;
    let mut foreign_runtime: Option<u32> = None;
    // Notre déclaration telle que T3 l'a écrite, pour la rétablir si besoin.
    // Liée au serveur qu'elle décrit : sans effet si T3 redémarre ailleurs.
    let declaration = std::fs::read_to_string(contract::runtime_file(&t3_base))
        .ok()
        .zip(contract::read_runtime(&t3_base).ok());
    let mut last_snapshot_success = None;
    let mut tracking_issue = None;
    loop {
        let expected_runtime = session.lock().runtime.clone();
        let snapshot_incarnation =
            runtime_incarnation(&t3_base).filter(|(runtime, _)| *runtime == expected_runtime);
        let snapshot_started = Instant::now();
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
                if retain_links_after_timeout(
                    &detail,
                    last_snapshot_success.clone(),
                    runtime_incarnation(&t3_base),
                    Instant::now(),
                ) {
                    for link in links.values() {
                        let _ = link.events.send(LinkEvent::SnapshotTimeout);
                    }
                    warn!(
                        "lecture T3 expirée ({detail}) ; liens conservés, observations suspendues"
                    );
                    publish_status(
                        paths,
                        "degraded",
                        &format!(
                            "{detail} : lecture expirée ; liens conservés au plus 90 s depuis la dernière lecture réussie, opérations soumises à une preuve fraîche"
                        ),
                        links.len(),
                    );
                    thread::sleep(poll);
                    continue;
                }
                // Un autre transport, un nouveau serveur ou une grâce expirée
                // ferme toujours les liens. Aucune remise n'est rejouée.
                last_snapshot_success = None;
                warn!("t3code injoignable ({detail}) ; liens fermés");
                for (_, link) in std::mem::take(&mut links) {
                    link.close();
                }
                drain_reopens.clear();
                obligation_scan_required = true;
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

        last_snapshot_success = snapshot_incarnation
            .filter(|incarnation| runtime_incarnation(&t3_base).as_ref() == Some(incarnation))
            .map(|(runtime, birth)| (snapshot_started, runtime, birth));
        let counts = relation_counts(&snapshot.threads);
        match agent_tracking230::sync(paths, &session, &snapshot) {
            Err(error) => {
                if tracking_issue.as_ref() != Some(&error) {
                    warn!("synchronisation des agents dormants en attente : {error}");
                }
                tracking_issue = Some(error);
            }
            Ok(()) => tracking_issue = None,
        }
        if last_relation_counts != Some(counts) {
            info!(
                "inventaire T3 : {} racine(s), {} fork(s), {} sous-agent(s)",
                counts.roots, counts.forks, counts.subagents
            );
            last_relation_counts = Some(counts);
        }

        // Le nom compact d'un état n'est pas réversible : le premier snapshot
        // réussi fournit les identifiants qui autorisent une lecture ciblée.
        if obligation_scan_required {
            for summary in snapshot.threads.iter().filter(|summary| {
                summary.relationship_to_parent.as_deref() == Some("subagent")
                    && summary.is_live()
                    && !thread_requires_link(summary, false)
                    && !attempted_obligations.contains(&summary.id)
            }) {
                if paths
                    .load_thread_state(&summary.id)
                    .has_drain_obligation(summary)
                {
                    obligations.insert(summary.id.clone());
                }
            }
            info!(
                "reprise T3 : {} obligation(s) persistée(s), plafond {} drain(s)",
                obligations.len(),
                DRAIN_REOPEN_BOUND
            );
            obligation_scan_required = false;
        }

        let summaries: HashMap<&str, &ThreadSummary> = snapshot
            .threads
            .iter()
            .map(|summary| (summary.id.as_str(), summary))
            .collect();
        for thread_id in links_missing_from_snapshot(links.keys(), &summaries) {
            if let Some(link) = links.remove(&thread_id) {
                info!("fil {thread_id} absent du snapshot : agent retiré");
                link.close();
            }
            obligations.remove(&thread_id);
            attempted_obligations.remove(&thread_id);
            drain_reopens.remove(&thread_id);
            retry_after.remove(&thread_id);
        }
        obligations.retain(|thread_id| summaries.contains_key(thread_id.as_str()));
        attempted_obligations.retain(|thread_id| summaries.contains_key(thread_id.as_str()));
        drain_reopens.retain(|thread_id| summaries.contains_key(thread_id.as_str()));
        retry_after.retain(|thread_id, _| summaries.contains_key(thread_id.as_str()));

        // Un worker mort ne doit jamais abandonner son lecteur. Une obligation
        // créée après le scan initial obtient une seule reprise par processus.
        for thread_id in links
            .iter()
            .filter(|(_, link)| !link.is_alive())
            .map(|(thread_id, _)| thread_id.clone())
            .collect::<Vec<_>>()
        {
            let retry_obligation = summaries.get(thread_id.as_str()).is_some_and(|summary| {
                summary.relationship_to_parent.as_deref() == Some("subagent")
                    && summary.is_live()
                    && !thread_requires_link(summary, false)
                    && !attempted_obligations.contains(&thread_id)
                    && paths
                        .load_thread_state(&thread_id)
                        .has_drain_obligation(summary)
            });
            if retry_obligation {
                obligations.insert(thread_id.clone());
            }
            let retry_active = summaries
                .get(thread_id.as_str())
                .is_some_and(|summary| thread_requires_link(summary, false));
            if let Some(link) = links.remove(&thread_id) {
                link.finish_dead();
            }
            drain_reopens.remove(&thread_id);
            if retry_active || retry_obligation {
                warn!(
                    "lien du fil {thread_id} terminé ; nouvelle tentative dans {RECONNECT_BACKOFF:?}"
                );
                retry_after.insert(thread_id, Instant::now() + RECONNECT_BACKOFF);
            }
        }

        let linked: HashSet<String> = links.keys().cloned().collect();
        let reopen_cooldowns: HashSet<String> = retry_after
            .iter()
            .filter(|(_, until)| Instant::now() < **until)
            .map(|(thread_id, _)| thread_id.clone())
            .collect();
        let selected_reopens: HashSet<String> = select_drain_reopens(
            &snapshot.threads,
            &obligations,
            &attempted_obligations,
            &linked,
            &reopen_cooldowns,
            drain_reopens.len(),
        )
        .into_iter()
        .map(str::to_string)
        .collect();

        // O(fils) par tour : la décision pure sépare présence T3, exposition et
        // drain. Une racine ou un fork neuf reste ouvrable dès qu'un
        // fournisseur est connu. Un sous-agent attend sa première activité.
        for summary in &snapshot.threads {
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

            if thread_provider(&summary).is_none() {
                if let Some(link) = links.remove(&summary.id) {
                    info!("fil {} sans fournisseur : agent retiré", summary.id);
                    link.close();
                }
                drain_reopens.remove(&summary.id);
                continue;
            }

            let now = Instant::now();
            let requires_link = thread_requires_link(&summary, false);
            if requires_link {
                // Un nouveau tour rend possible une future obligation distincte.
                // L'essai unique du tour précédent ne doit pas la neutraliser.
                attempted_obligations.remove(&summary.id);
                obligations.remove(&summary.id);
            }
            let has_link = links.contains_key(&summary.id);
            let (drained_key, drain_since, wait) = links
                .get(&summary.id)
                .map(|link| (link.drained_key(), link.drain_since, link.turn_wait))
                .unwrap_or((None, None, turn_wait()));
            let action = link_action(
                &summary,
                has_link,
                drained_key.as_deref(),
                drain_since,
                obligations.contains(&summary.id) && !attempted_obligations.contains(&summary.id),
                now,
                wait,
            );

            if action == LinkAction::Close {
                let terminal_child = summary.relationship_to_parent.as_deref() == Some("subagent")
                    && summary.is_live()
                    && !thread_requires_link(&summary, false);
                if let Some(link) = links.remove(&summary.id) {
                    info!("fil {} drainé ou retiré : agent fermé", summary.id);
                    link.close();
                }
                if terminal_child {
                    record_terminal_child_closed(
                        &summary.id,
                        &mut obligations,
                        &mut attempted_obligations,
                    );
                }
                drain_reopens.remove(&summary.id);
                continue;
            }

            if action == LinkAction::Open {
                let persisted_reopen = !thread_requires_link(&summary, false);
                if persisted_reopen && !selected_reopens.contains(&summary.id) {
                    continue;
                }
                if retry_after
                    .get(&summary.id)
                    .is_some_and(|until| Instant::now() < *until)
                {
                    continue;
                }
                match Link::open(paths, session.clone(), &summary) {
                    Ok(mut link) => {
                        if persisted_reopen {
                            link.begin_drain(now);
                            record_reopen_started(
                                &summary.id,
                                &mut obligations,
                                &mut drain_reopens,
                            );
                        }
                        links.insert(summary.id.clone(), link);
                    }
                    Err(e) => {
                        warn!("fil {} non exposé : {e}", summary.id);
                        retry_after.insert(summary.id.clone(), Instant::now() + RECONNECT_BACKOFF);
                        continue;
                    }
                }
            }

            if let Some(link) = links.get_mut(&summary.id) {
                if requires_link {
                    link.cancel_drain();
                    drain_reopens.remove(&summary.id);
                } else {
                    link.begin_drain(now);
                }
                link.tick(summary);
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
        let (runtime, uses_v2) = {
            let state = session.lock();
            (state.runtime.clone(), state.client.uses_v2())
        };
        // Session 114 : ces deux faits se répètent à chaque cycle de trois
        // secondes. Ne les dire qu'au changement : sinon un seul incident noie
        // le journal (456 lignes identiques en une journée) et cache le reste.
        match identities.refresh(&t3_base, &runtime, &bindings, uses_v2) {
            Ok(_) => {
                if identity_error.take().is_some() {
                    info!("rattachement MCP T3 rétabli");
                }
            }
            Err(error) => {
                if identity_error.as_deref() != Some(error.as_str()) {
                    warn!("rattachement MCP T3 indisponible : {error}");
                    identity_error = Some(error);
                }
            }
        }
        report_foreign_runtime(
            &t3_base,
            &runtime,
            declaration
                .as_ref()
                .filter(|(_, declared)| *declared == runtime)
                .map(|(text, _)| text.as_str()),
            &mut foreign_runtime,
        );
        publish_status(paths, "running", &session.base_url(), links.len());
        thread::sleep(poll);
    }
}

// ---------------------------------------------------------------------------
// Lien : un fil t3code = une connexion au daemon
// ---------------------------------------------------------------------------

enum LinkEvent {
    /// Envoyé seulement dans la grâce bornée du snapshot.
    SnapshotTimeout,
    Frame(Box<DaemonToWrapper>),
    Tick(Box<ThreadSummary>),
    DaemonGone,
    Close,
    Unavailable,
    JournalRejected {
        subscription_id: String,
        reason: bridget_transport::protocol::AttachRefusal,
    },
}

struct Link {
    events: Sender<LinkEvent>,
    worker: Option<thread::JoinHandle<()>>,
    reader: Option<thread::JoinHandle<()>>,
    shutdown: Option<UnixStream>,
    drained_key: Arc<Mutex<Option<String>>>,
    inbound_deliveries: Arc<AtomicUsize>,
    drain_since: Option<Instant>,
    turn_wait: Duration,
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
        let (reader, writer, name) = connect_and_register_at(ConnectAndRegisterAtArgs {
            socket: &paths.socket,
            agent_type: &agent_type,
            agent_id: Some(&agent_id),
            host: &crate::wrapper::host_name(),
            protocol: PROTOCOL,
            channel: None,
            mode: PresenceMode::Cli,
            location: None,
            os: &crate::wrapper::operating_system(),
            instance_id: &instance_id,
            domain: domain.as_deref(),
            turn_in_progress: summary
                .session
                .as_ref()
                .is_some_and(|session| session.active_turn_id.is_some()),
            turn_attest: true,
        })?;
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
        let shutdown = reader.get_ref().try_clone().map_err(|e| e.to_string())?;
        let (events, inbox) = mpsc::channel();
        let drained_key = Arc::new(Mutex::new(None));
        let inbound_deliveries = Arc::new(AtomicUsize::new(0));
        let worker = LinkWorker::new_with_drain(
            paths,
            session,
            summary,
            agent_id,
            instance_id,
            writer,
            stream,
            drained_key.clone(),
            inbound_deliveries.clone(),
        )?;
        let daemon_alive = worker.daemon_alive.clone();
        let cancelled = worker.cancelled.clone();
        let journal_control = worker.relay.subscription_control();
        let worker_handle = thread::Builder::new()
            .name(format!("t3-{}", &summary.id[..summary.id.len().min(8)]))
            .spawn(move || worker.run(inbox))
            .map_err(|e| e.to_string())?;
        let reader_handle = match spawn_reader(
            reader,
            events.clone(),
            daemon_alive,
            cancelled,
            drained_key.clone(),
            inbound_deliveries.clone(),
            &summary.id,
            Some(journal_control),
        ) {
            Ok(handle) => handle,
            Err(error) => {
                let _ = events.send(LinkEvent::Close);
                let _ = worker_handle.join();
                let _ = shutdown.shutdown(std::net::Shutdown::Both);
                return Err(error);
            }
        };
        Ok(Self::from_parts(
            events,
            worker_handle,
            reader_handle,
            shutdown,
            drained_key,
            inbound_deliveries,
            turn_wait(),
        ))
    }

    fn from_parts(
        events: Sender<LinkEvent>,
        worker: thread::JoinHandle<()>,
        reader: thread::JoinHandle<()>,
        shutdown: UnixStream,
        drained_key: Arc<Mutex<Option<String>>>,
        inbound_deliveries: Arc<AtomicUsize>,
        turn_wait: Duration,
    ) -> Self {
        Self {
            events,
            worker: Some(worker),
            reader: Some(reader),
            shutdown: Some(shutdown),
            drained_key,
            inbound_deliveries,
            drain_since: None,
            turn_wait,
        }
    }

    fn is_alive(&self) -> bool {
        self.worker.as_ref().is_some_and(|h| !h.is_finished())
    }

    fn tick(&self, summary: ThreadSummary) {
        let _ = self.events.send(LinkEvent::Tick(Box::new(summary)));
    }

    fn drained_key(&self) -> Option<String> {
        if self.inbound_deliveries.load(Ordering::SeqCst) != 0 {
            return None;
        }
        let value = self.drained_key.lock().recover_poison().clone();
        if self.inbound_deliveries.load(Ordering::SeqCst) != 0 {
            None
        } else {
            value
        }
    }

    fn begin_drain(&mut self, now: Instant) {
        self.drain_since.get_or_insert(now);
    }

    fn cancel_drain(&mut self) {
        self.drain_since = None;
    }

    fn finish(mut self, unregister: bool) {
        if unregister {
            let _ = self.events.send(LinkEvent::Close);
        }
        if let Some(handle) = self.worker.take() {
            let _ = handle.join();
        }
        if let Some(stream) = self.shutdown.take() {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
        if let Some(handle) = self.reader.take() {
            let _ = handle.join();
        }
    }

    fn close(self) {
        self.finish(true);
    }

    fn finish_dead(self) {
        self.finish(false);
    }
}

impl Drop for Link {
    fn drop(&mut self) {
        if let Some(stream) = self.shutdown.take() {
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }
    }
}

// Les identités et contrôles du lecteur restent explicitement séparés.
#[allow(clippy::too_many_arguments)]
fn spawn_reader(
    mut reader: BufReader<UnixStream>,
    events: Sender<LinkEvent>,
    alive: Arc<AtomicBool>,
    cancelled: Arc<Mutex<BTreeMap<String, Instant>>>,
    drained_key: Arc<Mutex<Option<String>>>,
    inbound_deliveries: Arc<AtomicUsize>,
    thread_id: &str,
    journal_control: Option<AttachRelayControl>,
) -> Result<thread::JoinHandle<()>, String> {
    thread::Builder::new()
        .name(format!(
            "t3-reader-{}",
            &thread_id[..thread_id.len().min(8)]
        ))
        .spawn(move || {
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
                                // Le journal déjà écrit ne dépend pas de HTTP ni du tour.
                                // Réutiliser le relais borné ; ne pas doubler le traitement
                                // dans la file de LinkWorker.
                                if let Some(relay) = &journal_control {
                                    match &frame {
                                        DaemonToWrapper::Subscribe {
                                            subscription_id,
                                            window,
                                            ..
                                        } => {
                                            if let Err(reason) = relay
                                                .subscribe(subscription_id.clone(), window.clone())
                                            {
                                                // Le lecteur ne prend jamais le verrou d'écriture.
                                                if events
                                                    .send(LinkEvent::JournalRejected {
                                                        subscription_id: subscription_id.clone(),
                                                        reason,
                                                    })
                                                    .is_err()
                                                {
                                                    break;
                                                }
                                            }
                                            continue;
                                        }
                                        DaemonToWrapper::Unsubscribe { subscription_id } => {
                                            relay.unsubscribe(subscription_id.clone());
                                            continue;
                                        }
                                        _ => {}
                                    }
                                }
                                let delivery = matches!(
                                    &frame,
                                    DaemonToWrapper::Deliver { .. }
                                        | DaemonToWrapper::DeliverIdempotent { .. }
                                );
                                match &frame {
                                    DaemonToWrapper::Deliver { .. }
                                    | DaemonToWrapper::DeliverIdempotent { .. } => {
                                        inbound_deliveries.fetch_add(1, Ordering::SeqCst);
                                        *drained_key.lock().recover_poison() = None;
                                    }
                                    DaemonToWrapper::CancelDelivery { id, .. } => {
                                        cancelled
                                            .lock()
                                            .recover_poison()
                                            .insert(id.clone(), Instant::now());
                                    }
                                    DaemonToWrapper::Disconnect => {
                                        alive.store(false, Ordering::SeqCst)
                                    }
                                    _ => {}
                                }
                                if events.send(LinkEvent::Frame(Box::new(frame))).is_err() {
                                    if delivery {
                                        inbound_deliveries.fetch_sub(1, Ordering::SeqCst);
                                    }
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
        })
        .map_err(|error| error.to_string())
}

fn t3_user_input_answers(answers: &Value) -> Option<Value> {
    let mut out = serde_json::Map::new();
    for (id, a) in answers.as_object()? {
        let mut values = a["selected"].as_array()?.clone();
        if let Some(text) = a["text"].as_str().filter(|t| !t.is_empty()) {
            values.push(json!(text));
        }
        out.insert(id.clone(), Value::Array(values));
    }
    Some(Value::Object(out))
}

struct LinkWorker {
    thread_id: String,
    agent_id: String,
    user_inputs: BTreeMap<String, Value>,
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
    /// Fait déjà annoncé sur cette connexion propriétaire. Un changement
    /// est envoyé au daemon, qui décide seul de sa cohérence.
    project_context: Option<(Option<String>, Option<String>)>,
    last_key: String,
    /// Clé observée juste avant un dispatch qui a pu démarrer un nouveau tour.
    /// Tant que T3 la republie, le lien ne peut pas se déclarer drainé.
    awaiting_snapshot_change: Option<String>,
    /// Plus grande page lue avec succès après un échec d'élargissement. Le
    /// fil complet T3 v2 n'est pas paginé : au-delà de 16 Mio il échoue
    /// toujours. Ne pas le redemander à chaque passage (fil qui ne fait que croître).
    detail_page_cap: Option<u32>,
    turn_wait: Duration,
    queue: VecDeque<(DaemonToWrapper, Instant)>,
    overflow: crate::delivery_spool244::DeliverySpool,
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
    /// Session 117 : blocage en cours de la projection du journal (motif,
    /// depuis quand, déjà signalé).
    journal_block: Option<(&'static str, Instant, bool)>,
    observation_events: Vec<bridget_transport::protocol::ObservationKind>,
    observation_schema: Vec<bridget_transport::protocol::ObservationKind>,
    drained_key: Arc<Mutex<Option<String>>>,
    /// Remises déjà lues sur le socket, mais pas encore placées dans la file
    /// du worker. Ce compteur ferme la fenêtre lecteur → canal → worker.
    inbound_deliveries: Arc<AtomicUsize>,
}

impl LinkWorker {
    #[cfg(test)]
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
        Self::new_with_drain(
            paths,
            session,
            summary,
            agent_id,
            instance_id,
            writer,
            stream,
            Arc::new(Mutex::new(None)),
            Arc::new(AtomicUsize::new(0)),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_with_drain(
        paths: &Paths,
        session: SharedSession,
        summary: &ThreadSummary,
        agent_id: String,
        instance_id: String,
        writer: BufWriter<UnixStream>,
        stream: UnixStream,
        drained_key: Arc<Mutex<Option<String>>>,
        inbound_deliveries: Arc<AtomicUsize>,
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
        send_wrapper_message(
            &writer,
            WrapperToDaemon::JournalReady {
                receipt_probe: true,
            },
        );
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
        let overflow = crate::delivery_spool244::DeliverySpool::open(
            &state_path.with_extension(format!("inbox-{instance_id}.sqlite")),
            &instance_id,
        )?;
        let state = paths.load_thread_state(&summary.id);
        let mut worker = Self {
            thread_id: summary.id.clone(),
            user_inputs: BTreeMap::new(),
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
            project_context: None,
            last_key: String::new(),
            awaiting_snapshot_change: None,
            detail_page_cap: None,
            turn_wait: turn_wait(),
            queue: VecDeque::new(),
            overflow,
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
            journal_block: None,
            observation_events: Vec::new(),
            observation_schema: Vec::new(),
            drained_key,
            inbound_deliveries,
        };
        worker.report_project_context(summary);
        Ok(worker)
    }

    fn report_project_context(&mut self, summary: &ThreadSummary) {
        let context = (
            summary.workspace_root.clone(),
            summary.worktree_path.clone(),
        );
        if self.project_context.as_ref() == Some(&context) {
            return;
        }
        send_wrapper_message(
            &self.writer,
            WrapperToDaemon::CommunicationProjectFact {
                root: context.0.clone().unwrap_or_default(),
                source: CommunicationProjectSource::T3,
                host: crate::wrapper::host_name(),
                worktree_root: context.1.clone(),
            },
        );
        self.project_context = Some(context);
        send_wrapper_message(
            &self.writer,
            WrapperToDaemon::T3ThreadBindingFact {
                version: bridget_transport::protocol::HUMAN_THREAD_VIEW_VERSION,
                t3_thread_id: summary.id.clone(),
            },
        );
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
                self.publish_drain_state(false);
                break;
            }
            self.refill_deliveries();
            if !self.queue.is_empty() && Instant::now() >= self.next_dispatch_check {
                self.next_dispatch_check = Instant::now() + Duration::from_millis(500);
                self.drive_queue(&inbox);
            }
            self.publish_drain_state(true);
        }
        self.publish_drain_state(false);
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
                self.report_project_context(&summary);
                send_wrapper_message(&self.writer, WrapperToDaemon::Heartbeat);
                send_wrapper_message(
                    &self.writer,
                    WrapperToDaemon::TurnState {
                        in_progress: summary
                            .session
                            .as_ref()
                            .is_some_and(|s| s.active_turn_id.is_some()),
                    },
                );
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
                self.observe_snapshot_change(&key);
                if (key != self.last_key
                    || !self.user_inputs.is_empty()
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
            LinkEvent::Frame(frame) => {
                if self.daemon_alive.load(Ordering::SeqCst) {
                    self.handle_frame(*frame);
                } else if matches!(
                    frame.as_ref(),
                    DaemonToWrapper::Deliver { .. } | DaemonToWrapper::DeliverIdempotent { .. }
                ) {
                    self.report_discard(frame.as_ref(), Discard::LinkClosing);
                }
            }
            LinkEvent::JournalRejected {
                subscription_id,
                reason,
            } => {
                send_wrapper_message(
                    &self.writer,
                    WrapperToDaemon::AttachRejected {
                        subscription_id: Some(subscription_id),
                        reason,
                    },
                );
            }
            LinkEvent::SnapshotTimeout => {
                // Seulement pendant la grâce : aucun tour ni droit attesté.
                send_wrapper_message(&self.writer, WrapperToDaemon::Heartbeat);
                self.handle_event(LinkEvent::Unavailable);
            }
            LinkEvent::Unavailable => {
                self.set_observation_ready(false);
                self.last_key.clear();
                self.publish_drain_state(false);
            }
            LinkEvent::DaemonGone => {
                warn!("daemon parti pour le fil {}", self.thread_id);
                self.daemon_alive.store(false, Ordering::SeqCst);
            }
            LinkEvent::Close => {
                while let Some((frame, _)) = self.queue.pop_front() {
                    self.report_discard(&frame, Discard::LinkClosing);
                }
                send_wrapper_message(&self.writer, WrapperToDaemon::Unregister);
                self.daemon_alive.store(false, Ordering::SeqCst);
            }
        }
    }

    fn publish_drain_state(&self, available: bool) {
        let mut published = self.drained_key.lock().recover_poison();
        let drained = available
            && !self.last_key.is_empty()
            && self.awaiting_snapshot_change.is_none()
            && self.queue.is_empty()
            && !self.overflow.has_pending()
            && self.journal_inflight.is_empty()
            && self.journal_caught_up
            && !self.journal_dirty
            && !self.journal_failed.load(Ordering::SeqCst)
            && self.inbound_deliveries.load(Ordering::SeqCst) == 0
            && self.state.pending.is_empty()
            && self.response_sent.is_empty();
        *published = drained.then(|| self.last_key.clone());
    }

    fn observe_snapshot_change(&mut self, key: &str) {
        if self
            .awaiting_snapshot_change
            .as_ref()
            .is_some_and(|before| before != key)
        {
            self.awaiting_snapshot_change = None;
        }
    }

    fn refresh_user_inputs(&mut self, detail: &ThreadDetail) {
        use bridget_transport::protocol::{
            ChildProviderRequest, ChildProviderRequestId, ChildRequestClosedState,
            ChildRequestDecision, ChildRequestKind,
        };
        for activity in detail
            .activities
            .iter()
            .filter(|a| a.kind == "user_input.request")
        {
            let source = &activity.payload["request"];
            let Some(source_id) = source["id"].as_str() else {
                continue;
            };
            let existing = self
                .user_inputs
                .iter()
                .find(|(_, r)| r["callback"]["provider_request_id"] == source_id)
                .map(|(id, row)| (id.clone(), row.clone()));
            if source["status"] == "pending"
                && matches!(
                    source["responseCapability"]["type"].as_str(),
                    Some("live" | "message")
                )
            {
                let questions = &activity.payload["questions"];
                if let Some((id, row)) = &existing
                    && row["callback"]["details"]["questions"] != *questions
                {
                    send_wrapper_message(
                        &self.writer,
                        WrapperToDaemon::ChildRequestClosed {
                            request_id: id.clone(),
                            state: ChildRequestClosedState::ClosedBeforeApply,
                        },
                    );
                    self.user_inputs.remove(id);
                }
                let request = ChildProviderRequest {
                    version: 1,
                    provider_session: self.thread_id.clone(),
                    turn_id: activity
                        .turn_id
                        .clone()
                        .unwrap_or_else(|| self.thread_id.clone()),
                    provider_request_id: ChildProviderRequestId::String(source_id.into()),
                    kind: ChildRequestKind::UserInput,
                    details: json!({"method":"t3.runtime-request.respond","questions":questions,"secret":false}),
                    offered: vec![ChildRequestDecision::Accept],
                    question_ids: questions
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|q| q["id"].as_str().map(str::to_owned))
                        .collect(),
                    truncated: false,
                };
                send_wrapper_message(&self.writer, WrapperToDaemon::ChildRequest { request });
            } else if let Some((id, row)) = existing {
                let expected = t3_user_input_answers(&row["answers"]);
                let applied = row["state"] == "submitted"
                    && source["status"] == "resolved"
                    && expected.as_ref() == Some(&source["answers"]);
                send_wrapper_message(
                    &self.writer,
                    WrapperToDaemon::ChildRequestClosed {
                        request_id: id.clone(),
                        state: if applied {
                            ChildRequestClosedState::AlreadyApplied
                        } else {
                            ChildRequestClosedState::ClosedBeforeApply
                        },
                    },
                );
                self.user_inputs.remove(&id);
            }
        }
    }
    fn answer_user_input(&mut self, answer: bridget_transport::protocol::ChildRequestAnswer) {
        use bridget_transport::protocol::{ChildProviderRequestId, ChildRequestDecision};
        if answer.provider_session != self.thread_id
            || answer.decision != ChildRequestDecision::Accept
        {
            return;
        }
        let ChildProviderRequestId::String(source_id) = &answer.provider_request_id else {
            return;
        };
        let Some(row) = self.user_inputs.get_mut(&answer.request_id) else {
            return;
        };
        if row["callback"]["provider_request_id"] != *source_id
            || row["callback"]["turn_id"] != answer.turn_id
        {
            return;
        }
        // La commande durable appartient à Bridget. Une erreur réseau reprend le même commandId.
        if uuid::Uuid::parse_str(&answer.outcome).is_err() {
            return;
        }
        let Some(answers) = answer.answer.as_ref() else {
            return;
        };
        let expected_questions = row["callback"]["details"]["questions"].clone();
        row["state"] = json!("submitted");
        // Canon commun utilisé pour comparer la résolution relue depuis T3.
        let mut canonical = serde_json::Map::new();
        for (id, values) in answers.as_object().into_iter().flatten() {
            canonical.insert(id.clone(), json!({"selected":values,"text":""}));
        }
        row["answers"] = Value::Object(canonical);
        let detail = match self
            .session
            .call(|c| c.thread_detail(&self.thread_id, DETAIL_PAGE))
        {
            Ok(v) => v,
            Err(_) => return,
        };
        let pending = detail.activities.iter().any(|a| {
            a.kind == "user_input.request"
                && a.payload["request"]["id"] == *source_id
                && a.payload["request"]["status"] == "pending"
                && a.payload["questions"] == expected_questions
                && matches!(
                    a.payload["request"]["responseCapability"]["type"].as_str(),
                    Some("live" | "message")
                )
        });
        if pending {
            let _ = self.session.call(|c| {
                c.answer_user_input(&self.thread_id, source_id, &answer.outcome, answers)
            });
        }
        self.refresh_user_inputs(&detail);
        self.last_key.clear();
    }

    fn handle_frame(&mut self, frame: DaemonToWrapper) {
        match frame {
            DaemonToWrapper::DeliveryReceiptQuery {
                delivery_id,
                recipient_instance_id,
                delivery_generation,
            } => {
                if let Some(report) = self.tracker.as_ref().and_then(|tracker| {
                    tracker.receipt_report(
                        delivery_id,
                        &recipient_instance_id,
                        delivery_generation,
                        unix_now() as i64,
                    )
                }) {
                    send_wrapper_message(&self.writer, report);
                }
            }
            // Verdict de l'annonce propriétaire, pas une livraison ni une
            // réponse de mission. Aucun texte n'est injecté dans le fil.
            DaemonToWrapper::ProjectContextResult { .. } => {}
            DaemonToWrapper::UserInputResult { result } => {
                let item = &result["item"];
                if item["origin"] == "t3"
                    && item["callback"]["provider_session"] == self.thread_id
                    && let Some(id) = item["request_id"].as_str()
                {
                    if matches!(
                        item["state"].as_str(),
                        Some("pending" | "submitted" | "unknown")
                    ) && (self.user_inputs.contains_key(id) || self.user_inputs.len() < 32)
                    {
                        if !(item["state"] == "pending"
                            && self
                                .user_inputs
                                .get(id)
                                .is_some_and(|r| r["state"] == "submitted"))
                        {
                            self.user_inputs.insert(id.into(), item.clone());
                        }
                    } else {
                        self.user_inputs.remove(id);
                    }
                }
            }
            DaemonToWrapper::ChildRequestAnswer { answer } => self.answer_user_input(answer),

            delivery @ (DaemonToWrapper::Deliver { .. }
            | DaemonToWrapper::DeliverIdempotent { .. }) => {
                self.enqueue_delivery(delivery);
                *self.drained_key.lock().recover_poison() = None;
                let _ = self.inbound_deliveries.fetch_update(
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                    |count| count.checked_sub(1),
                );
                self.request_status();
            }
            DaemonToWrapper::CancelDelivery { id, reason, .. } => {
                if let Err(error) = self.overflow.cancel(&id) {
                    error!(
                        "annulation en file durable impossible : {error} ; aucune nouvelle injection"
                    );
                    self.daemon_alive.store(false, Ordering::SeqCst);
                }
                self.cancelled
                    .lock()
                    .recover_poison()
                    .insert(id.clone(), Instant::now());
                self.discard_by_id(
                    &id,
                    if reason == "échéance de la demande dépassée" {
                        Discard::Expired("de demande")
                    } else {
                        Discard::RequestClosed("annulée")
                    },
                );
                info!("demande {id} retirée avant démarrage : {reason}");
            }
            DaemonToWrapper::Ack { id, .. } => {
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

    fn enqueue_delivery(&mut self, delivery: DaemonToWrapper) {
        if let DaemonToWrapper::DeliverIdempotent { delivery_id, .. } = &delivery {
            if let Some((queued, _)) = self.queue.iter_mut().find(|(frame,_)|
                matches!(frame,DaemonToWrapper::DeliverIdempotent{delivery_id:id,..} if id==delivery_id)) {
                match crate::delivery_spool244::reconcile_delivery_replay(queued,&delivery) {
                    Ok(frozen)=>*queued=frozen,
                    Err(_)=>{
                        error!("remise {delivery_id} divergente : lien fermé, copie maître conservée");
                        self.daemon_alive.store(false, Ordering::SeqCst);
                    }
                }
                return;
            }
            if self.queue.len() >= QUEUE_BOUND || self.overflow.has_pending() {
                if let Err(error) = self.overflow.defer(&delivery) {
                    error!(
                        "file durable indisponible : {error} ; copie maître conservée, reprise au raccordement"
                    );
                    self.daemon_alive.store(false, Ordering::SeqCst);
                }
                return;
            }
        }
        if self.queue.len() >= QUEUE_BOUND {
            // Une trame historique n'a pas de copie durable. Elle ne peut
            // évincer une remise idempotente conservée pour le destinataire.
            if matches!(
                self.queue.front(),
                Some((DaemonToWrapper::DeliverIdempotent { .. }, _))
            ) {
                self.report_discard(&delivery, Discard::Saturated);
                return;
            }
            if let Some((oldest, _)) = self.queue.pop_front() {
                self.report_discard(&oldest, Discard::Saturated);
            }
        }
        self.queue.push_back((delivery, Instant::now()));
    }

    fn refill_deliveries(&mut self) {
        if !self.queue.is_empty() {
            return;
        }
        while self.queue.len() < QUEUE_BOUND {
            match self.overflow.next() {
                Ok(Some(frame)) => self.queue.push_back((frame, Instant::now())),
                Ok(None) => break,
                Err(error) => {
                    error!(
                        "relecture de file impossible : {error} ; aucune injection, copie maître conservée"
                    );
                    self.daemon_alive.store(false, Ordering::SeqCst);
                    break;
                }
            }
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
        // La file précède le tracker et tout dispatch fournisseur. À la
        // fermeture, ces remises idempotentes restent dans le magasin maître ;
        // elles ne sont ni rejetées ni déclarées injectées. Le Register suivant
        // reprend exactement leurs IDs, jamais une mission sous une autre clé.
        if matches!(why, Discard::LinkClosing)
            && matches!(frame, DaemonToWrapper::DeliverIdempotent { .. })
        {
            info!(
                "remise {} conservée au maître pendant la coupure",
                message.id
            );
            return;
        }
        let reason = match why {
            Discard::Saturated => {
                format!("file du fil saturée à {QUEUE_BOUND}, remise écartée")
            }
            Discard::Expired(which) => format!("échéance {which} dépassée avant démarrage"),
            Discard::RequestClosed(state) => format!("demande {state} avant démarrage"),
            Discard::GuidanceUnavailable(reason) => reason.into(),
            Discard::LinkClosing => "lien fermé avant démarrage de la remise".into(),
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
        if (message.reply || matches!(why, Discard::GuidanceUnavailable(_)))
            && !matches!(why, Discard::RequestClosed(_))
        {
            send_wrapper_message(
                &self.writer,
                WrapperToDaemon::DeliveryRejected {
                    id: message.id.clone(),
                    reason,
                    execution_id: None,
                    generation: None,
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

    /// O(n), n<=64 : un guidage explicite peut dépasser une tête bloquée.
    fn next_delivery_index(&mut self, summary: &ThreadSummary) -> Option<usize> {
        let provider = thread_provider(summary);
        let v2 = self.session.inner.lock().recover_poison().client.uses_v2();
        if !v2 || !explicit_steering_supported(provider.as_deref()) {
            for (frame, _) in &mut self.queue {
                let message = match frame {
                    DaemonToWrapper::Deliver { message, .. }
                    | DaemonToWrapper::DeliverIdempotent { message, .. } => message,
                    _ => unreachable!(),
                };
                if message.intent == Some(bridget_core::MessageIntent::SteerCurrent)
                    && message
                        .references
                        .iter()
                        .any(|reference| reference == "steer-or-queue")
                {
                    message.intent = None;
                }
            }
        }
        let guidance = self.queue.iter().position(|(frame, _)| {
            matches!(
                delivery_message(frame).intent.as_ref(),
                Some(
                    bridget_core::MessageIntent::SteerCurrent
                        | bridget_core::MessageIntent::InterruptAndStart
                )
            )
        });
        let index = guidance.unwrap_or(0);
        let (frame, _) = self.queue.get(index)?;
        let message = delivery_message(frame);

        let unavailable = match message.intent.as_ref() {
            Some(bridget_core::MessageIntent::SteerCurrent)
                if !v2 || !explicit_steering_supported(provider.as_deref()) =>
            {
                Some("steer_unavailable")
            }
            Some(bridget_core::MessageIntent::InterruptAndStart) if !v2 => {
                Some("interruption indisponible : protocole T3 historique sans deliveryIntent")
            }
            _ => None,
        };
        if let Some(reason) = unavailable {
            let (frame, _) = self.queue.remove(index).expect("remise présente");
            self.report_discard(&frame, Discard::GuidanceUnavailable(reason));
            return None;
        }
        let busy = summary
            .session
            .as_ref()
            .is_some_and(|session| session.active_turn_id.is_some());
        if guidance.is_none()
            && busy
            && (!v2
                || !deliverable_while_busy(
                    provider.as_deref(),
                    groupable(message) && delivery_mid_turn_ok(frame),
                ))
        {
            return None;
        }
        Some(index)
    }

    fn drive_queue(&mut self, inbox: &Receiver<LinkEvent>) {
        self.refill_deliveries();
        let now = unix_now();
        self.cancelled
            .lock()
            .recover_poison()
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
        let busy = summary
            .session
            .as_ref()
            .is_some_and(|s| s.active_turn_id.is_some());
        let Some(index) = self.next_delivery_index(&summary) else {
            return;
        };
        // Le temps de la requête HTTP a pu faire passer une échéance.
        let stale = self
            .queue
            .get(index)
            .and_then(|(frame, received)| self.discard_cause(frame, *received, unix_now()));
        if let Some(why) = stale {
            let (frame, _) = self.queue.remove(index).expect("remise présente");
            self.report_discard(&frame, why);
            return;
        }
        let Some((frame, _)) = self.queue.get(index) else {
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
        let delivery_intent = match message.intent.as_ref() {
            Some(bridget_core::MessageIntent::SteerCurrent) => Some("steer"),
            Some(bridget_core::MessageIntent::InterruptAndStart) => Some("restart"),
            _ if busy => Some("steer"),
            _ => None,
        };
        let guidance_frame = message
            .intent
            .as_ref()
            .is_some_and(|intent| {
                matches!(
                    intent,
                    bridget_core::MessageIntent::SteerCurrent
                        | bridget_core::MessageIntent::InterruptAndStart
                )
            })
            .then(|| frame.clone());
        let length = if guidance_frame.is_some() {
            1
        } else if busy {
            self.queue
                .iter()
                .take(self.batchable_prefix())
                .take_while(|(frame, _)| delivery_mid_turn_ok(frame))
                .count()
                .max(1)
        } else {
            self.batchable_prefix()
        };
        // Session 114 : les messages sans réponse attendue en tête de file
        // partagent un seul tour. Sans cela, vingt-trois comptes rendus
        // d'avancement ouvraient vingt-trois tours chez le coordinateur : budget
        // consommé pour rien, et une corrélation faussée pour les demandes
        // voisines dès qu'un de ces tours ne produisait pas de texte.
        let mut batch = Vec::new();
        for _ in 0..length {
            let (mut frame, received) = self.queue.remove(index).expect("remise présente");
            // Session 112 : le budget d'exécution repart du dispatch réel.
            match &mut frame {
                DaemonToWrapper::Deliver { message, .. }
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
            batch.push(match frame {
                DaemonToWrapper::Deliver { message, .. } => BatchedDelivery::Historic(message),
                DaemonToWrapper::DeliverIdempotent {
                    delivery_id,
                    recipient_instance_id,
                    delivery_generation,
                    expires_at,
                    message,
                    ..
                } => BatchedDelivery::Idempotent {
                    delivery_id,
                    recipient_instance_id,
                    delivery_generation,
                    expires_at,
                    message,
                },
                _ => unreachable!("la file ne contient que des remises"),
            });
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or_default();
        // Le tracker déduplique les rejeux ; l'accusé ne part qu'après
        // acceptation de la commande par t3code (dispatch 200).
        let mut outcome = Ok(());
        let mut tracker = self.tracker.take().expect("tracker présent");
        let reports = deliver_batch_to_interactive(&mut tracker, batch, now, |messages| {
            outcome = self.dispatch_batch(messages, &summary, delivery_intent);
            outcome.clone()
        });
        self.tracker = Some(tracker);
        if let Err(e) = outcome {
            error!("remise au fil {} : {e}", self.thread_id);
            if let Some(frame) = guidance_frame {
                let reason = if e == "steer_unavailable" {
                    Some("steer_unavailable")
                } else if e.contains("protocole T3 historique sans deliveryIntent") {
                    Some("interruption indisponible : protocole T3 historique sans deliveryIntent")
                } else {
                    None
                };
                if let Some(reason) = reason {
                    self.report_discard(&frame, Discard::GuidanceUnavailable(reason));
                }
            }
        }
        for report in reports {
            send_wrapper_message(&self.writer, report);
        }
    }

    /// Session 114 : longueur du lot, en tête de file, qui peut partager un
    /// tour. Toujours au moins un — la tête a déjà passé tous les contrôles.
    ///
    /// Au-delà du premier, seuls des messages groupables et encore valides
    /// entrent dans le lot : un message écarté doit garder son propre sort,
    /// rapporté au daemon par le cycle suivant (session 113).
    /// Complexité : O(n) sur la file, bornée par `QUEUE_BOUND`.
    fn batchable_prefix(&self) -> usize {
        let now = unix_now();
        let mut bytes = 0;
        let mut length = 0;
        let family = self
            .queue
            .front()
            .and_then(|(frame, _)| batch_family(delivery_message(frame)));
        for (frame, received) in &self.queue {
            let message = delivery_message(frame);
            if family.is_none()
                || batch_family(message) != family
                || length >= BATCH_BOUND
                || bytes + message.body.len() > BATCH_BODY_BOUND
            {
                break;
            }
            if length > 0 && self.discard_cause(frame, *received, now).is_some() {
                break;
            }
            bytes += message.body.len();
            length += 1;
        }
        length.max(1)
    }

    /// Message que le pont ne doit plus injecter : sa propre échéance, ou celle
    /// de sa demande suivie, est passée.
    fn is_stale(&self, message: &bridget_core::BridgetMessage) -> bool {
        message
            .deadline_at
            .is_some_and(|deadline| deadline <= unix_now())
            || self
                .requests
                .get(&message.id)
                .is_some_and(|request| request.deadline_at <= unix_now() as i64)
    }

    /// Remise d'un seul message. La production passe toujours par un lot ;
    /// ce raccourci sert aux tests qui n'ont rien à grouper.
    #[cfg(test)]
    fn dispatch_with_id(
        &mut self,
        message: &bridget_core::BridgetMessage,
        summary: &ThreadSummary,
    ) -> Result<(), String> {
        let intent = match message.intent.as_ref() {
            Some(bridget_core::MessageIntent::SteerCurrent) => Some("steer"),
            Some(bridget_core::MessageIntent::InterruptAndStart) => Some("restart"),
            _ => None,
        };
        self.dispatch_batch(std::slice::from_ref(message), summary, intent)
    }

    /// Démarre le fil attesté libre par drive_queue, après contrôle d'autorité.
    /// `commandId` = identifiant du premier message du lot (t3code déduplique
    /// dessus). Session 114 : le lot ne dépasse un message que s'ils sont tous
    /// ordinaires et sans réponse attendue, donc sans attente à corréler.
    fn dispatch_batch(
        &mut self,
        messages: &[bridget_core::BridgetMessage],
        summary: &ThreadSummary,
        delivery_intent: Option<&str>,
    ) -> Result<(), String> {
        let Some(first) = messages.first() else {
            return Ok(());
        };
        if !self.daemon_alive.load(Ordering::SeqCst) {
            return Err("daemon déconnecté".into());
        }
        let anchor = summary.latest_turn.as_ref().map(|t| t.turn_id.clone());
        let command_id = first.id.clone();
        // Identifiant de message déterministe : un rejeu après panne reconstruit
        // exactement la même commande, que t3code déduplique par `commandId`.
        let message_id = stable_uuid(&format!("t3code-message:{command_id}"));
        let command = contract::turn_start_command(
            &self.thread_id,
            &command_id,
            &message_id,
            &batch_envelope(messages),
            &summary.runtime_mode,
            &summary.interaction_mode,
        );
        // L'attente est écrite AVANT l'appel : si le pont meurt entre le 200 de
        // t3code et cette ligne, la corrélation serait perdue et la réponse ne
        // reviendrait jamais. Écrite avant, elle est au pire inutile — et reste
        // conservée jusqu'à un refus certain ou une clôture attestée.
        for message in messages {
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
                    message_id: message_id.clone(),
                    anchor_turn_id: anchor.clone(),
                    dispatched_at: contract::iso_now(),
                    attempts: 0,
                    response: None,
                    reply_requested: Some(true),
                });
                self.state.save(&self.state_path)?;
            }
        }
        // La persistance peut elle-même prendre du temps. Le lecteur socket
        // publie la perte d'autorité sans attendre le traitement de sa file.
        if !self.daemon_alive.load(Ordering::SeqCst)
            || messages.iter().any(|message| self.is_stale(message))
        {
            return Err("autorité absente ou demande périmée avant dispatch".into());
        }
        // Session.call peut attendre le verrou partagé de renouvellement.
        // Vérifier aussi dans sa closure, à la dernière frontière HTTP.
        self.awaiting_snapshot_change = Some(summary.change_key());
        self.publish_drain_state(false);
        match self.session.call_detached(|client| {
            if !self.daemon_alive.load(Ordering::SeqCst)
                || messages.iter().any(|message| {
                    self.cancelled
                        .lock()
                        .recover_poison()
                        .contains_key(&message.id)
                        || self.is_stale(message)
                })
            {
                return Err(ContractError::Transport(
                    "remise annulée, périmée ou autorité déconnectée avant dispatch".into(),
                ));
            }
            client.dispatch(&command, delivery_intent)
        }) {
            Ok(result) => {
                for message in messages {
                    info!(
                        "message {} remis au fil {} (séquence {})",
                        message.id, self.thread_id, result.sequence
                    );
                }
                Ok(())
            }
            Err(ContractError::Transport(reason))
                if reason == "steer_unavailable"
                    || reason.contains("protocole T3 historique sans deliveryIntent") =>
            {
                self.awaiting_snapshot_change = None;
                self.state.pending.retain(|pending| {
                    !messages
                        .iter()
                        .any(|message| pending.request_id == message.id)
                });
                let _ = self.state.save(&self.state_path);
                Err(reason)
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
                self.awaiting_snapshot_change = None;
                self.state
                    .pending
                    .retain(|p| !messages.iter().any(|message| p.request_id == message.id));
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
        let mut previous = None;
        let mut detail = loop {
            let fetched = self
                .session
                .call(|client| client.thread_detail(&self.thread_id, limit));
            let detail = match fetched {
                Ok(detail) => detail,
                // Même principe que T3 (fenêtre bornée récente) : l'élargissement
                // est une aide, pas une condition. La page bornée déjà lue suffit
                // à amorcer et à corréler les tours récents.
                Err(e) if previous.is_some() => {
                    info!(
                        "détail du fil {} : élargissement à {limit} tours indisponible ({e}) ; page de {} tours conservée",
                        self.thread_id,
                        limit / 2
                    );
                    self.detail_page_cap = Some(limit / 2);
                    break previous.take().expect("page précédente");
                }
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
                    && !spontaneous_turn(&detail, turn)
            });
            if (anchors_missing || !seeded || origin_missing)
                && limit < self.detail_page_cap.unwrap_or(DETAIL_PAGE_MAX)
            {
                limit *= 2;
                previous = Some(detail);
                continue;
            }
            break detail;
        };
        self.refresh_user_inputs(&detail);
        let seen_activities: HashSet<_> = self
            .state
            .seen_activities
            .iter()
            .map(String::as_str)
            .collect();
        // Un détail omis relu au plus une fois par passage ; aucun accès depuis Android à la source.
        let mut fetched = false;
        if self.state.tool_progress_seeded {
            detail.activities.retain_mut(|activity| {
                if activity.kind != "tool.progress"
                    || activity.payload["output_omitted"] != true
                    || seen_activities.contains(activity.id.as_str())
                {
                    return true;
                }
                if fetched {
                    return false;
                }
                fetched = true;
                let id = activity.payload["tool_id"]
                    .as_str()
                    .unwrap_or("")
                    .to_owned();
                let node = activity.payload["source_node_id"].clone();
                if let Some(run) = activity.turn_id.as_deref() {
                    match self
                        .session
                        .call(|client| client.tool_detail(&self.thread_id, &id, run, &node))
                    {
                        Ok(fields) => activity.payload = fields,
                        Err(_) => activity.payload["details_unavailable"] = serde_json::json!(true),
                    }
                }
                true
            });
        }
        // Le fait de capacité précède les faits du journal sur le même writer,
        // notamment lors du rétablissement après une indisponibilité HTTP.
        use bridget_transport::protocol::ObservationKind as Kind;
        self.observation_schema.clear();
        let origin_known = detail.latest_turn.as_ref().is_none_or(|turn| {
            self.state.turn_origins.contains_key(&turn.turn_id)
                || observation_origin(&detail, turn).is_some()
                || spontaneous_turn(&detail, turn)
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
                        // Session 125 : origine relevée, sa fin sera annoncée.
                        && !self.state.turn_origins.contains_key(id)
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
        if gap {
            // Le daemon ignore la lacune d'une source non déclarée. Hors lacune,
            // ne rien annoncer avant le journal : sinon un journal en retard
            // faisait alterner prête/indisponible à chaque lecture (118).
            self.set_observation_ready(!self.journal_failed.load(Ordering::SeqCst));
            // Lacune connue, quantité inconnue : ne pas inventer un nombre de
            // faits perdus ni deux notices de fausse déconnexion/reconnexion.
            send_wrapper_message(&self.writer, WrapperToDaemon::ObservationGap { dropped: 0 });
        }
        self.record_turn_origin(&detail, summary);
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
            WrapperToDaemon::ObservationCapabilities {
                events,
                runtime_catalogue: false,
            },
        );
    }

    /// Repère d'installation : au premier regard, l'historique antérieur est
    /// mémorisé sans être rejoué. Ensuite, chaque message nouveau est projeté.
    fn project_journal(&mut self, detail: &ThreadDetail, summary: &ThreadSummary) {
        self.confirm_journal();
        self.state.forget_invisible(detail);
        self.journal_dirty = true;
        let v2_states: HashMap<&str, &str> = detail
            .activities
            .iter()
            .filter(|a| a.kind == "t3v2.run")
            .filter_map(|a| Some((a.turn_id.as_deref()?, a.payload["status"].as_str()?)))
            .collect();
        if !self.journal_caught_up {
            return;
        }
        if !self.state.tool_progress_seeded {
            if self.state.seeded {
                self.state.seen_activities.extend(
                    detail
                        .activities
                        .iter()
                        .filter(|activity| activity.kind == "tool.progress")
                        .map(|activity| activity.id.clone()),
                );
            }
            self.state.tool_progress_seeded = true;
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
                            if !v2_states.is_empty() {
                                return v2_states
                                    .get(id.as_str())
                                    .is_some_and(|state| turn_is_final(state));
                            }
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
            self.clear_journal_block();
            return;
        }
        if self.journal_failed.load(Ordering::SeqCst) {
            self.note_journal_block("journal en échec", || {
                "l'écrivain s'est arrêté ; le lien sera rouvert".to_string()
            });
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
            if !v2_states.is_empty() {
                return v2_states
                    .get(turn_id)
                    .is_some_and(|state| turn_is_final(state));
            }
            summary
                .latest_turn
                .as_ref()
                .is_none_or(|latest| latest.turn_id != turn_id || turn_is_final(&latest.state))
        };
        // O(S + M) : sans index, O(S × M) à chaque lecture d'un grand fil.
        let seen: HashSet<&str> = self.state.seen.iter().map(String::as_str).collect();
        for message in &detail.messages {
            if message.streaming {
                continue;
            }
            if !seen.contains(message.id.as_str()) && !self.journal_inflight.contains(&message.id) {
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
                        // Session 121 : répété à chaque lecture tant que la
                        // file est pleine ; signalé une fois après le délai.
                        self.note_journal_block("message refusé par le journal", || e);
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
                // Session 125 : un tour suivi de près par un autre n'est jamais vu
                // clos comme dernier tour ; si son origine a été relevée pendant
                // qu'il tournait, sa fin est un fait observable, pas une lacune.
                // L'état exact n'est plus exposé par T3 : « ended ».
                let origin = self.state.turn_origins.get(turn_id);
                let notification =
                    origin.is_some_and(|id| self.state.notification_messages.contains(id));
                let (id, payload) = match origin {
                    Some(_) if notification => (
                        format!("bridget-observation:{turn_id}"),
                        serde_json::json!({"t3_turn_id": turn_id, "stop_reason": "ended"}),
                    ),
                    Some(origin) => (
                        origin.clone(),
                        serde_json::json!({"t3_turn_id": turn_id, "stop_reason": "ended"}),
                    ),
                    None => (
                        message.id.clone(),
                        serde_json::json!({"t3_turn_id": turn_id}),
                    ),
                };
                match self.journal.enqueue("turn_end", Some(&id), payload) {
                    Ok(()) => {
                        self.journal_inflight.insert(format!("end:{turn_id}"));
                    }
                    Err(e) => {
                        self.note_journal_block("fin de tour refusée par le journal", || e);
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
            if let Err(error) = self.journal.enqueue("turn_end", Some(&id), payload) {
                self.note_journal_block("fin de tour refusée par le journal", || {
                    error.to_string()
                });
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
            // Détails des seuls outils autorisés du fil principal ; les autres activités sont exclues.
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
                if let Err(error) = self.journal.enqueue(event, Some(&id), payload) {
                    self.note_journal_block("activité refusée par le journal", || {
                        error.to_string()
                    });
                    return;
                }
                self.journal_inflight.insert(id);
            }
            // Repère séparé : ne pas confirmer une activité multi-fichiers sur le
            // premier fichier seulement ; le marqueur suit tous ses faits.
            if let Err(error) = self.journal.enqueue(
                "t3_activity_seen",
                Some(&key),
                serde_json::json!({"t3_activity_id":activity.id}),
            ) {
                self.note_journal_block("repère d'activité refusé par le journal", || {
                    error.to_string()
                });
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
        self.prune_turn_origins(detail, latest);
        self.journal_dirty = false;
        self.clear_journal_block();
    }

    /// Session 117 : signale qu'un blocage de la projection du journal persiste.
    ///
    /// Ces sorties étaient muettes : le 24/09, un fil est resté non observable
    /// des heures sans aucune trace. Un blocage bref est normal (écriture en
    /// cours) ; seul un blocage qui dure JOURNAL_BLOCK_GRACE est signalé, une
    /// fois par motif, et son rétablissement est annoncé.
    fn note_journal_block(&mut self, kind: &'static str, detail: impl FnOnce() -> String) {
        let now = Instant::now();
        match &mut self.journal_block {
            Some((current, since, reported)) if *current == kind => {
                if !*reported && now.duration_since(*since) >= JOURNAL_BLOCK_GRACE {
                    warn!(
                        "observation du fil {} suspendue depuis {} s : {kind} ({})",
                        self.thread_id,
                        now.duration_since(*since).as_secs(),
                        detail()
                    );
                    *reported = true;
                }
            }
            _ => self.journal_block = Some((kind, now, false)),
        }
    }

    /// Session 117 : fin d'un blocage ; annoncée seulement s'il avait été signalé.
    fn clear_journal_block(&mut self) {
        if let Some((kind, since, true)) = self.journal_block.take() {
            info!(
                "observation du fil {} rétablie après {} s ({kind})",
                self.thread_id,
                since.elapsed().as_secs()
            );
        }
    }

    /// Session 116 : relève l'origine du dernier tour à chaque lecture, que le
    /// journal d'observation soit à jour ou non : l'appariement des réponses en
    /// dépend désormais, pas seulement l'observation.
    fn record_turn_origin(&mut self, detail: &ThreadDetail, summary: &ThreadSummary) {
        let users: HashSet<&str> = detail
            .messages
            .iter()
            .filter(|m| m.role == "user")
            .map(|m| m.id.as_str())
            .collect();
        for activity in detail.activities.iter().filter(|a| a.kind == "t3v2.run") {
            if let (Some(turn), Some(user)) = (
                activity.turn_id.as_ref(),
                activity.payload["userMessageId"].as_str(),
            ) && users.contains(user)
            {
                self.state.turn_origins.insert(turn.clone(), user.into());
            }
        }
        let latest = detail.latest_turn.as_ref().or(summary.latest_turn.as_ref());
        if let Some(turn) = latest {
            if let Some(origin) = observation_origin(detail, turn) {
                self.state
                    .turn_origins
                    .insert(turn.turn_id.clone(), origin.to_string());
            } else if spontaneous_turn(detail, turn) {
                self.state
                    .turn_origins
                    .insert(turn.turn_id.clone(), spontaneous_origin(&turn.turn_id));
            }
        }
        self.prune_turn_origins(detail, latest);
    }

    /// Borne la table à SEEN_BOUND. Un tour silencieux n'a aucun message
    /// visible : son origine est gardée tant qu'une demande l'attend, sinon la
    /// preuve disparaîtrait avant l'appariement. O(origines + messages).
    fn prune_turn_origins(&mut self, detail: &ThreadDetail, latest: Option<&contract::LatestTurn>) {
        if self.state.turn_origins.len() <= SEEN_BOUND {
            return;
        }
        let visible: HashSet<&str> = detail
            .messages
            .iter()
            .filter_map(|m| m.turn_id.as_deref())
            .collect();
        let awaited: HashSet<&str> = self
            .state
            .pending
            .iter()
            .map(|p| p.message_id.as_str())
            .collect();
        self.state.turn_origins.retain(|id, origin| {
            visible.contains(id.as_str())
                || awaited.contains(origin.as_str())
                || latest.is_some_and(|t| t.turn_id == *id)
        });
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
            let offset = reader.next_offset();
            let size = std::fs::metadata(&path).map(|m| m.len()).ok();
            if size.is_none_or(|size| offset < size) {
                self.journal_caught_up = false;
                self.note_journal_block("journal non rattrapé", || {
                    format!(
                        "{} lu jusqu'à {offset} octets sur {}",
                        path.display(),
                        size.map_or("taille inconnue".to_string(), |size| format!("{size}"))
                    )
                });
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
        let origins = &self.state.turn_origins;
        for pending in &mut self.state.pending {
            if !pending.needs_correlation() {
                continue;
            }
            let verdict = correlate(detail, summary, pending, origins);
            match verdict {
                Correlation::Answered(text) => {
                    pending.response = Some(text);
                }
                // Session 116 : le tour prouvé de la demande est clos et n'a
                // rien écrit. L'expéditeur l'apprend au lieu d'attendre son
                // échéance ; le texte est celui de Bridget, jamais un propos
                // prêté au destinataire. T3 projette ses événements de façon
                // asynchrone : un tour peut se dire clos juste avant que son
                // texte n'apparaisse. Le constat n'est donc engagé qu'après
                // SETTLE_ATTEMPTS lectures ; une lecture intermédiaire qui voit
                // le texte le relaie normalement.
                Correlation::Silent => {
                    pending.attempts = pending.attempts.saturating_add(1);
                    if pending.attempts >= SETTLE_ATTEMPTS {
                        pending.response = Some(SILENT_TURN_NOTICE.to_string());
                    }
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
                        // Session 121 : plutôt que rien, le texte écrit après
                        // la demande, marqué comme hypothèse.
                        if matches!(verdict, Correlation::Ambiguous) {
                            pending.response = uncertain_answer(detail, &pending.message_id);
                        }
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
    /// Capacité T3 absente : aucune injection ni interruption tentée.
    GuidanceUnavailable(&'static str),
    /// Le propriétaire ferme le lien avant que la remise ait démarré.
    LinkClosing,
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
        DaemonToWrapper::Deliver { message, .. }
        | DaemonToWrapper::DeliverIdempotent { message, .. } => message,
        _ => unreachable!("seules les remises sont mises en attente"),
    }
}

/// Le droit est porté par le daemon sur chaque remise, jamais par origin/from.
fn delivery_mid_turn_ok(frame: &DaemonToWrapper) -> bool {
    matches!(
        frame,
        DaemonToWrapper::Deliver {
            mid_turn_ok: Some(true),
            ..
        } | DaemonToWrapper::DeliverIdempotent {
            mid_turn_ok: Some(true),
            ..
        }
    )
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
/// Session 114 : enveloppe d'un lot partageant un seul tour. Un message seul
/// garde exactement son enveloppe historique ; le format de lot n'apparaît qu'à
/// partir de deux, et ne concerne que des messages sans réponse attendue —
/// c'est ce qui autorise une enveloppe commune, puisqu'aucune réponse du tour
/// n'aura à être attribuée à l'un plutôt qu'à l'autre.
pub(crate) fn batch_envelope(messages: &[bridget_core::BridgetMessage]) -> String {
    let [message] = messages else {
        let total = messages.len();
        if messages
            .iter()
            .all(|m| batch_family(m) == Some(BatchFamily::Notification))
        {
            let mut rendered =
                format!("🔔 {total} notifications Bridget groupées dans ce tour :\n");
            for (index, message) in messages.iter().enumerate() {
                rendered.push_str(&format!(
                    "\n── {}/{total} (id {}) ──\n{}\n",
                    index + 1,
                    message.id,
                    message.body
                ));
            }
            rendered.push_str("\nPas d'accusé de réception à envoyer. Agis seulement si ces faits changent ce que tu dois faire ; informe l'utilisateur si utile.");
            return rendered;
        }
        let mut rendered =
            format!("📥 {total} messages Bridget groupés dans ce tour (reply=no) :\n");
        for (index, message) in messages.iter().enumerate() {
            rendered.push_str(&format!(
                "\n── {}/{total} — de {} (id {}) ──\n{}\n",
                index + 1,
                message.sender_label(),
                message.id,
                message.body
            ));
        }
        if messages
            .iter()
            .any(bridget_core::envelope::is_human_instruction)
        {
            rendered.push('\n');
            rendered.push_str(bridget_core::envelope::HUMAN_REPLY_NOTICE);
            rendered.push_str(" Les résultats destinés aux autres agents se transmettent à ces agents par Bridget.");
        } else {
            rendered.push('\n');
            rendered.push_str(bridget_core::envelope::NO_REPLY_NOTICE);
        }
        return rendered;
    };
    envelope(message)
}

/// Session 129 : le 01/10, un coordinateur resté 8 h 45 dans un seul tour a
/// reçu d'un coup treize messages qui attendaient sa fin. L'attente du repos
/// sert à relier une réponse à sa demande ; un message sans réponse attendue
/// n'a rien à relier. Chez les fournisseurs dont T3 pilote le tour en cours
/// (Claude, donc GLM, et Codex enV2), il est remis aussitôt si mid_turn_ok vaut true.
/// Les demandes suivies, sollicitations de fil et notifications non groupables
/// attendent la fin du tour. Cursor149 sait seulement relancer.
// Le guidage explicite V2 est distinct du regroupement automatique des notifications.
fn explicit_steering_supported(provider: Option<&str>) -> bool {
    matches!(provider, Some("claude" | "codex"))
}

fn deliverable_while_busy(provider: Option<&str>, groupable_front: bool) -> bool {
    groupable_front && matches!(provider, Some("claude" | "codex"))
}

/// Famille de regroupement : seuls des membres d'une même famille partagent un
/// tour, chacune ayant sa consigne. Une demande suivie attend LA réponse du
/// tour et une sollicitation de fil sa propre lecture : jamais groupées.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BatchFamily {
    /// Session 114 : messages sans réponse attendue.
    Message,
    /// Session 130 : notifications d'observation. Une rafale de fins de tour
    /// réveillait l'abonné une fois par notification (471 en dix jours).
    Notification,
}

fn batch_family(message: &bridget_core::BridgetMessage) -> Option<BatchFamily> {
    if message.reply
        || message.thread_notice.is_some()
        || matches!(
            message.intent.as_ref(),
            Some(
                bridget_core::MessageIntent::SteerCurrent
                    | bridget_core::MessageIntent::InterruptAndStart
            )
        )
    {
        return None;
    }
    Some(if message.id.starts_with("bridget-observation:") {
        BatchFamily::Notification
    } else {
        BatchFamily::Message
    })
}

/// Message sans réponse attendue, seul autorisé à piloter un tour en cours (129).
fn groupable(message: &bridget_core::BridgetMessage) -> bool {
    batch_family(message) == Some(BatchFamily::Message)
}

pub(crate) fn envelope(message: &bridget_core::BridgetMessage) -> String {
    let sender = message.sender_label();
    if let Some(notice) = &message.thread_notice {
        // Session 102 : sollicitation de fil. Aucun relais de la réponse finale,
        // aucun accusé : le destinataire lit et publie par bridget_thread.
        // Présentation seulement : O(n) au plus sur le titre source, mémoire
        // et sortie bornées à 200 scalaires ; le corps reste byte-identique.
        let mut title = String::new();
        let mut title_chars = 0;
        let mut separator = false;
        for character in message
            .thread_display_title
            .as_deref()
            .unwrap_or("")
            .chars()
        {
            if character.is_whitespace() || bridget_core::is_disallowed_control(character) {
                separator = !title.is_empty();
                continue;
            }
            if title_chars == 200 {
                break;
            }
            if separator {
                title.push(' ');
                title_chars += 1;
                separator = false;
            }
            if title_chars == 200 {
                break;
            }
            title.push(character);
            title_chars += 1;
        }
        let title = title.trim_end();
        let title_line = if title.is_empty() {
            String::new()
        } else {
            format!(
                "Titre du fil Bridget : {}\n\n",
                serde_json::to_string(title).expect("une chaîne JSON est sérialisable")
            )
        };
        return format!(
            "🧵 Sollicitation Bridget dans le fil {} (nouveautés jusqu'à {}, id {}) :\n\n{}\n\nConsulte ce fil avec l'outil bridget_thread (read, puis ack) ; publie dans le fil si utile (post). Ne fais pas de réponse directe à cette alerte : le pont ne relaie pas ta réponse finale.",
            notice.thread_id,
            notice.through_seq,
            message.id,
            format_args!("{title_line}{}", message.body)
        );
    }
    if message.id.starts_with("bridget-observation:") {
        return format!(
            "🔔 Notification Bridget (id {}) :\n\n{}\n\nPas d'accusé de réception à envoyer. Agis seulement si ce fait change ce que tu dois faire ; informe l'utilisateur si utile.",
            message.id, message.body
        );
    }
    if !message.reply && bridget_core::envelope::is_human_instruction(message) {
        return bridget_core::envelope::wrap_envelope(message);
    }
    if !message.reply {
        return format!(
            "💬 Message Bridget de {} (id {}, reply=no) :\n\n{}\n\n{}",
            sender,
            message.id,
            message.body,
            bridget_core::envelope::NO_REPLY_NOTICE
        );
    }
    format!(
        "💬 Message Bridget de {} (id {}) :\n\n{}\n\n— Réponds dans ce tour : Bridget transmettra ta réponse finale à {}. Elle reste aussi affichée à l'utilisateur dans ce fil : commence-la par « ↪ Réponse à {} (relayée par Bridget) : » pour qu'il sache qu'elle ne lui est pas adressée.",
        sender, message.id, message.body, message.from, message.from
    )
}

pub(crate) fn turn_is_final(state: &str) -> bool {
    matches!(state, "completed" | "error" | "interrupted")
}

/// Distingue l'existence du fil T3 du besoin de garder un agent Bridget.
/// L'absence de filiation conserve le comportement V1 compatible.
fn thread_requires_link(summary: &ThreadSummary, has_obligation: bool) -> bool {
    if !summary.is_live() {
        return false;
    }
    if summary.relationship_to_parent.as_deref() != Some("subagent") {
        return true;
    }
    has_obligation
        || summary
            .session
            .as_ref()
            .is_some_and(|session| session.active_turn_id.is_some())
        || summary
            .latest_turn
            .as_ref()
            .is_some_and(|turn| !turn_is_final(&turn.state))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LinkAction {
    Open,
    Tick,
    Drain,
    Close,
    Ignore,
}

/// Décision sans I/O. `drain_since` appartient au lien et ne dépend jamais de
/// `change_key()`, afin qu'un `updated_at` mouvant ne repousse pas la borne.
fn link_action(
    summary: &ThreadSummary,
    has_link: bool,
    drained_key: Option<&str>,
    drain_since: Option<Instant>,
    has_reopen_obligation: bool,
    now: Instant,
    wait: Duration,
) -> LinkAction {
    if !summary.is_live() {
        return if has_link {
            LinkAction::Close
        } else {
            LinkAction::Ignore
        };
    }
    if thread_requires_link(summary, has_reopen_obligation) {
        return if has_link {
            LinkAction::Tick
        } else {
            LinkAction::Open
        };
    }
    if has_link {
        let drained = drained_key.is_some_and(|key| key == summary.change_key());
        let expired = drain_since.is_some_and(|since| now.duration_since(since) >= wait);
        return if drained || expired {
            LinkAction::Close
        } else {
            LinkAction::Drain
        };
    }
    LinkAction::Ignore
}

fn select_drain_reopens<'a>(
    threads: &'a [ThreadSummary],
    obligations: &HashSet<String>,
    attempted: &HashSet<String>,
    linked: &HashSet<String>,
    cooldowns: &HashSet<String>,
    already_open: usize,
) -> Vec<&'a str> {
    let available = DRAIN_REOPEN_BOUND.saturating_sub(already_open);
    threads
        .iter()
        .filter(|thread| {
            thread.relationship_to_parent.as_deref() == Some("subagent")
                && thread.is_live()
                && !thread_requires_link(thread, false)
                && obligations.contains(&thread.id)
                && !attempted.contains(&thread.id)
                && !linked.contains(&thread.id)
                && !cooldowns.contains(&thread.id)
                && thread_provider(thread).is_some()
        })
        .take(available)
        .map(|thread| thread.id.as_str())
        .collect()
}

/// Identifie les propriétaires qui n'existent plus dans la projection T3.
/// La fermeture reste dans `serve`, car elle doit joindre les deux threads.
fn links_missing_from_snapshot<'a>(
    linked: impl Iterator<Item = &'a String>,
    summaries: &HashMap<&str, &ThreadSummary>,
) -> Vec<String> {
    linked
        .filter(|thread_id| !summaries.contains_key(thread_id.as_str()))
        .cloned()
        .collect()
}

/// Une ouverture ne consomme pas la tentative. Une mort imprévue du worker
/// peut ainsi relire l'obligation et reprendre après le délai de reconnexion.
fn record_reopen_started(
    thread_id: &str,
    obligations: &mut HashSet<String>,
    drain_reopens: &mut HashSet<String>,
) {
    obligations.remove(thread_id);
    drain_reopens.insert(thread_id.to_string());
}

/// Seule une fermeture attestée par drain ou par borne dure consomme l'essai
/// du processus courant.
fn record_terminal_child_closed(
    thread_id: &str,
    obligations: &mut HashSet<String>,
    attempted: &mut HashSet<String>,
) {
    attempted.insert(thread_id.to_string());
    obligations.remove(thread_id);
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct RelationCounts {
    roots: usize,
    forks: usize,
    subagents: usize,
}

fn relation_counts(threads: &[ThreadSummary]) -> RelationCounts {
    let mut counts = RelationCounts::default();
    for thread in threads {
        match thread.relationship_to_parent.as_deref() {
            Some("subagent") => counts.subagents += 1,
            Some("fork") => counts.forks += 1,
            _ => counts.roots += 1,
        }
    }
    counts
}

/// O(messages), page bornée. T3 projette requestedAt et createdAt depuis la
/// même commande. Égalité unique, jamais ordre temporel ni texte de l'utilisateur.
fn observation_origin<'a>(
    detail: &'a ThreadDetail,
    turn: &contract::LatestTurn,
) -> Option<&'a str> {
    if let Some(run) = detail
        .activities
        .iter()
        .find(|a| a.kind == "t3v2.run" && a.turn_id.as_deref() == Some(&turn.turn_id))
    {
        let user_id = run.payload["userMessageId"].as_str()?;
        return detail
            .messages
            .iter()
            .find(|m| m.role == "user" && m.id == user_id)
            .map(|m| m.id.as_str());
    }
    let requested = turn.requested_at.as_ref()?;
    let mut candidates = detail
        .messages
        .iter()
        .filter(|m| m.role == "user" && m.created_at.as_ref() == Some(requested));
    let origin = candidates.next()?;
    candidates.next().is_none().then_some(origin.id.as_str())
}

/// Session 121 : un tour sans message déclencheur (réveil d'arrière-plan à la
/// fin d'un sous-agent) est spontané, prouvé quand la page lue couvre sa
/// demande sans qu'aucun message utilisateur n'en porte l'horodatage. Nos
/// notifications arrivent toujours comme messages : un tour spontané ne peut
/// pas en provenir, sa fin est donc observable sans risque de boucle. Avant,
/// un tel tour rendait tout le fil Claude inobservable. O(n).
fn spontaneous_turn(detail: &ThreadDetail, turn: &contract::LatestTurn) -> bool {
    if detail.activities.iter().any(|a| a.kind == "t3v2.run") {
        return false;
    }
    let Some(requested) = turn.requested_at.as_deref() else {
        return false;
    };
    let covered = detail
        .messages
        .iter()
        .filter_map(|m| m.created_at.as_deref())
        .any(|at| at < requested);
    covered
        && !detail
            .messages
            .iter()
            .any(|m| m.role == "user" && m.created_at.as_deref() == Some(requested))
}

/// Origine d'un tour spontané : jamais l'identifiant d'un message, donc jamais
/// appariée à une demande.
fn spontaneous_origin(turn_id: &str) -> String {
    format!("t3-turn:{turn_id}")
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
    if activity.kind == "context.compaction" {
        let valid = payload["kind"] == "tool"
            && payload["tool_type"] == "context_compaction"
            && matches!(
                payload["status"].as_str(),
                Some("running" | "completed" | "failed" | "cancelled")
            )
            && payload["turn_id"]
                .as_str()
                .is_some_and(|turn| Some(turn) == activity.turn_id.as_deref())
            && payload["item_id"].as_str().is_some_and(|id| {
                !id.is_empty() && id.len() <= 256 && !id.chars().any(char::is_control)
            });
        return if valid { vec![payload.clone()] } else { vec![] };
    }
    if activity.kind == "tool.progress" {
        let mut event = serde_json::json!({"kind":"tool","tool_id":payload["tool_id"],"text":payload["tool"].as_str().unwrap_or_else(||payload["itemType"].as_str().unwrap_or("tool")),"tool_type":payload["itemType"],"status":payload["status"],"tool":payload["tool"]});
        for field in [
            "input",
            "output",
            "exit_code",
            "output_omitted",
            "details_truncated",
            "details_unavailable",
        ] {
            if let Some(value) = payload.get(field) {
                event[field] = value.clone();
            }
        }
        return vec![event];
    }
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

/// Session 116 : texte du tour dont l'origine est prouvée.
///
/// Les messages de ce tour suivent notre message, présent dans la page : ils y
/// sont donc tous. Un tour qui n'est plus le dernier est clos, T3 n'en menant
/// qu'un à la fois par fil. Seuls ses messages comptent : les réveils
/// d'arrière-plan qui suivent ont leurs propres tours. O(messages de la page).
fn proven_turn_answer(
    detail: &ThreadDetail,
    summary: &ThreadSummary,
    turn_id: &str,
) -> Correlation {
    if detail.activities.iter().any(|a| {
        a.kind == "t3v2.run"
            && a.turn_id.as_deref() == Some(turn_id)
            && a.payload["status"]
                .as_str()
                .is_none_or(|status| !turn_is_final(status))
    }) {
        return Correlation::Waiting;
    }
    let parts: Vec<&contract::Message> = detail
        .messages
        .iter()
        .filter(|m| m.role != "user" && m.turn_id.as_deref() == Some(turn_id))
        .collect();
    let open = |latest: Option<&contract::LatestTurn>| {
        latest.is_some_and(|latest| latest.turn_id == turn_id && !turn_is_final(&latest.state))
    };
    let active = summary
        .session
        .as_ref()
        .is_some_and(|session| session.active_turn_id.as_deref() == Some(turn_id));
    if active
        || parts.iter().any(|m| m.streaming)
        || open(summary.latest_turn.as_ref())
        || open(detail.latest_turn.as_ref())
    {
        return Correlation::Waiting;
    }
    let text = parts
        .iter()
        .map(|m| m.text.trim())
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    if text.is_empty() {
        Correlation::Silent
    } else {
        Correlation::Answered(text)
    }
}

/// Session 116 : réponse relayée quand le tour prouvé d'une demande s'est
/// terminé sans rien écrire.
const SILENT_TURN_NOTICE: &str =
    "(Bridget) Le destinataire a terminé le tour de cette demande sans réponse écrite.";

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Correlation {
    /// Le tour de notre message est clos : voici son texte.
    Answered(String),
    /// Session 116 : le tour prouvé de notre message est clos sans texte.
    Silent,
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
    origins: &BTreeMap<String, String>,
) -> Correlation {
    // V2 lie explicitement chaque message à son run. Aucun appariement ordinal
    // ni hypothèse « un tour plus récent clôt le précédent » n'est nécessaire.
    if detail.activities.iter().any(|a| a.kind == "t3v2.run") {
        let Some(message) = detail
            .messages
            .iter()
            .find(|m| m.id == pending.message_id && m.role == "user")
        else {
            return Correlation::Missing;
        };
        let Some(run_id) = message.turn_id.as_deref() else {
            return Correlation::Waiting;
        };
        if !detail
            .activities
            .iter()
            .any(|a| a.kind == "t3v2.run" && a.turn_id.as_deref() == Some(run_id))
        {
            return Correlation::Waiting;
        }
        return proven_turn_answer(detail, summary, run_id);
    }
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
    // Session 116 : la preuve directe d'abord. Le pont relève quel tour notre
    // message a déclenché, par égalité stricte entre l'horodatage de demande du
    // tour et celui du message (`observation_origin`). Le rang ci-dessous ne
    // vaut que si chaque message a exactement un tour écrit ; un tour qui
    // n'écrit rien, un réveil d'agent en arrière-plan ou deux messages
    // partageant un tour le faussent, et la demande était abandonnée. La preuve
    // directe ne dépend d'aucun décompte : la réponse est le texte de CE tour.
    // O(k), k ≤ SEEN_BOUND origines retenues.
    if let Some(turn_id) = origins
        .iter()
        .find_map(|(turn, origin)| (origin == &pending.message_id).then_some(turn.as_str()))
    {
        return proven_turn_answer(detail, summary, turn_id);
    }
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

const UNCERTAIN_ANSWER_NOTICE: &str = "[Bridget : appariement incertain. Texte écrit par le \
destinataire après ta demande, jusqu'au message suivant ; il peut appartenir à un autre tour.]";

/// Session 121 : le 25/09, deux réponses sont restées dans leur fil faute
/// d'appariement certain (tours faussés par une seconde application T3).
/// Texte non utilisateur qui suit la demande jusqu'au message utilisateur
/// suivant, préfixé d'un avertissement ; `None` s'il n'y en a pas. O(n).
fn uncertain_answer(detail: &ThreadDetail, message_id: &str) -> Option<String> {
    let start = detail.messages.iter().position(|m| m.id == message_id)? + 1;
    let text = detail.messages[start..]
        .iter()
        .take_while(|m| m.role != "user")
        .filter(|m| !m.streaming)
        .map(|m| m.text.trim())
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    (!text.is_empty()).then(|| format!("{UNCERTAIN_ANSWER_NOTICE}\n\n{text}"))
}

#[cfg(test)]
#[path = "t3code_tests.rs"]
mod tests;

#[cfg(test)]
mod mobile183_compaction_projection_tests {
    use super::*;
    #[test]
    fn mobile183_compaction_projection_preserves_only_correlated_structural_facts() {
        let mut activity = contract::Activity {
            id: "comp".into(),
            kind: "context.compaction".into(),
            turn_id: Some("r".into()),
            payload: json!({"kind":"tool","tool_type":"context_compaction","status":"running","turn_id":"r","item_id":"i"}),
        };
        let facts = activity_payloads(&activity, "codex", Path::new("/tmp"));
        assert_eq!(facts.len(), 1);
        assert!(bridget_transport::act_kind::validate_journal_write("update", &facts[0]).is_ok());
        activity.payload["turn_id"] = json!("foreign");
        assert!(activity_payloads(&activity, "codex", Path::new("/tmp")).is_empty());
    }
}
