//! Module wrapper — lance un agent CLI et le connecte au daemon bridget.
//!
//! Appelé par le CLI quand l'utilisateur tape : bridget codex, bridget claude, etc.

use bridget_transport::protocol::{decode, encode};
use bridget_transport::{DaemonToWrapper, TmuxTransport, Transport, WrapperToDaemon};
use log::{debug, error, info, warn};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// Constantes de reconnexion optimisées pour auto-reconnect transparent
const RECONNECT_INITIAL_DELAY: Duration = Duration::from_secs(1);
const RECONNECT_MAX_DELAY: Duration = Duration::from_secs(30);
const RECONNECT_STABLE_RESET: Duration = Duration::from_secs(60);

// HEARTBEAT amélioré : plus fréquent pour une détection rapide (3s au lieu de 15s)
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(3);

// SOCKET_CHECK_INTERVAL : vérification proactive de disponibilité du socket
const SOCKET_CHECK_INTERVAL: Duration = Duration::from_secs(5);

// Sonde de runtime : le fichier de session n'est relu que si sa date de
// modification a changé, et le chemin n'est re-résolu que rarement — `lsof`
// coûte environ 130 ms, contre quelques microsecondes pour un `stat`.
const RUNTIME_PROBE_INTERVAL: Duration = Duration::from_secs(20);
// Un `codex resume` ouvre un second rollout sans fermer le premier : garder le
// chemin trop longtemps rendrait la nouvelle session invisible d'autant.
// 60 s aligne cette latence sur l'exigence de fraîcheur de la spec, pour un
// surcoût de 134 ms par minute.
const RUNTIME_PATH_REFRESH: Duration = Duration::from_secs(60);

fn socket_path() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join(".cache")
            .join("bridget")
            .join("bridget.sock")
    } else {
        PathBuf::from("/tmp").join("bridget.sock")
    }
}

fn host_name() -> String {
    if let Ok(host) = std::env::var("HOSTNAME") {
        if !host.trim().is_empty() {
            return host;
        }
    }
    Command::new("hostname")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|host| host.trim().to_string())
        .filter(|host| !host.is_empty())
        .unwrap_or_else(|| "inconnu".to_string())
}

fn transport_name() -> String {
    if let Ok(transport) = std::env::var("BRIDGET_TRANSPORT") {
        if !transport.trim().is_empty() {
            return transport;
        }
    }
    let config_path = std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
        .join(".config/bridget/federation.env");
    std::fs::read_to_string(config_path)
        .ok()
        .and_then(|config| {
            config
                .lines()
                .find_map(|line| line.strip_prefix("transport=").map(str::to_owned))
        })
        .filter(|transport| !transport.trim().is_empty())
        .unwrap_or_else(|| "unix".to_string())
}

/// Domaine de travail dérivé du répertoire courant.
///
/// La racine du dépôt git donne le regroupement le plus naturel : deux agents
/// lancés n'importe où dans le même projet partagent un domaine. Hors dépôt, le
/// nom du répertoire courant fait office de domaine.
///
/// Le nom est rendu brut, sans embellissement : un répertoire
/// `projet-b` donne le domaine `projet-b`. Une règle
/// de nettoyage implicite serait indevinable pour l'utilisateur, qui peut de
/// toute façon surcharger avec `bridget domain`.
fn derive_domain() -> Option<String> {
    let git_root = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|path| path.trim().to_string())
        .filter(|path| !path.is_empty())
        .map(PathBuf::from);

    let base = match git_root {
        Some(root) => root,
        None => std::env::current_dir().ok()?,
    };
    base.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
}

/// Chemin du domaine surchargé d'un agent, en miroir de `agent-names/`.
fn domain_state_path(agent: &str) -> PathBuf {
    socket_path()
        .parent()
        .unwrap()
        .join("agent-domains")
        .join(agent)
}

/// Domaine effectif d'un agent : la surcharge si elle existe, le dérivé sinon.
///
/// Relu à chaque enregistrement, y compris après une reconnexion, pour la même
/// raison que le nom : seule la trace sur disque connaît l'intention de
/// l'utilisateur.
fn effective_domain(agent: &str) -> Option<String> {
    std::fs::read_to_string(domain_state_path(agent))
        .map(|domain| domain.trim().to_string())
        .ok()
        .filter(|domain| !domain.is_empty())
        .or_else(derive_domain)
}

/// Nom d'OS stable et lisible pour l'annuaire Bridget.
fn operating_system() -> String {
    match std::env::consts::OS {
        "macos" => "macOS".to_string(),
        "linux" => "Linux".to_string(),
        os => os.to_string(),
    }
}

fn get_current_pane_id() -> Result<String, String> {
    let output = Command::new("tmux")
        .args(["display-message", "-p", "#{pane_id}"])
        .output()
        .map_err(|e| format!("tmux exec: {}", e))?;
    if !output.status.success() {
        return Err(format!("tmux: {}", String::from_utf8_lossy(&output.stderr)));
    }
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if id.is_empty() {
        Err("pane vide".into())
    } else {
        Ok(id)
    }
}

/// Calcule un hash des args pour identifier une session (resume, etc.).
fn session_hash(agent_args: &[String]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    let mut found_session = false;
    for arg in agent_args {
        // Un vrai session-id de Codex ressemble à un UUID :
        // ex: 019ff375-1f42-7552-b87c-481a5ac14faa (36 chars, 4 tirets)
        if arg.len() == 36 && arg.matches('-').count() == 4 {
            arg.hash(&mut hasher);
            found_session = true;
        }
    }
    if found_session {
        format!("{:016x}", hasher.finish())
    } else {
        // Pas de session-id → pas de persistance → auto-incrément normal
        String::new()
    }
}

/// Charge le nom persistant pour cette session.
fn load_persistent_name(_agent_type: &str, agent_args: &[String]) -> Option<String> {
    let hash = session_hash(agent_args);
    if hash.is_empty() {
        return None; // Pas de session-id → auto-incrément normal
    }
    let name_file = persistent_name_path(&hash);
    if name_file.exists() {
        let name = std::fs::read_to_string(&name_file).ok()?;
        let name = name.trim().to_string();
        if !name.is_empty() {
            eprintln!("[bridget] Nom retrouvé: « {} »", name);
            return Some(name);
        }
    }
    None
}

/// Sauvegarde le nom pour les futurs resume.
fn save_persistent_name(_agent_type: &str, agent_args: &[String], name: &str) {
    let hash = session_hash(agent_args);
    if hash.is_empty() {
        return; // Pas de session-id, rien à sauver
    }
    let name_file = persistent_name_path(&hash);
    if let Some(parent) = name_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&name_file, name);
}

fn persistent_name_path(hash: &str) -> std::path::PathBuf {
    socket_path()
        .parent()
        .unwrap()
        .join("agent-names")
        .join(hash)
}

/// Met un file descriptor en mode close-on-exec (FD_CLOEXEC).
fn set_cloexec(stream: &UnixStream) {
    use std::os::unix::io::AsRawFd;
    let fd = stream.as_raw_fd();
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFD);
        if flags >= 0 {
            libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC);
        }
    }
}

/// Délai exponentiel (1, 2, 4, 8, 16, 30 s) avec un jitter de ±20 %.
/// Le plafond protège le daemon et le serveur SSH pendant une panne longue.
fn reconnect_delay(attempt: u32) -> Duration {
    let multiplier = 1_u64 << attempt.min(5);
    let base_ms = (RECONNECT_INITIAL_DELAY.as_millis() as u64)
        .saturating_mul(multiplier)
        .min(RECONNECT_MAX_DELAY.as_millis() as u64);
    let jitter_span = base_ms / 5;
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64
        ^ std::process::id() as u64
        ^ attempt as u64;
    let jitter = (seed % (jitter_span.saturating_mul(2) + 1)) as i64 - jitter_span as i64;
    Duration::from_millis((base_ms as i64 + jitter).max(1) as u64).min(RECONNECT_MAX_DELAY)
}

/// Nom courant de l'agent, tel que `bridget rename` l'a laissé sur disque.
///
/// Le wrapper ne peut pas se fier au nom qu'il a obtenu à son enregistrement :
/// l'agent a pu être renommé depuis, et seul ce fichier en garde la trace. S'y
/// référer à chaque reconnexion évite qu'un agent renommé ne réapparaisse sous
/// son nom d'origine après une coupure — ce qui se produit à chaque rupture de
/// tunnel dans une installation fédérée.
fn resolve_current_name(name_state_path: &std::path::Path, fallback: &str) -> String {
    std::fs::read_to_string(name_state_path)
        .map(|name| name.trim().to_string())
        .ok()
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

/// Sonde qui suit le modèle et l'effort courants d'un agent Codex en observant
/// son fichier de session.
///
/// Elle n'émet que sur changement effectif : un agent inactif ne produit aucun
/// trafic vers le daemon (FR-007).
struct RuntimeProbe {
    pid: u32,
    path: Option<PathBuf>,
    path_resolved_at: Instant,
    last_check: Instant,
    last_mtime: Option<SystemTime>,
    last_sent: Option<crate::runtime::RuntimeObservation>,
}

impl RuntimeProbe {
    fn new(pid: u32) -> Self {
        RuntimeProbe {
            pid,
            path: None,
            // Forcer une première résolution au tout premier tick.
            path_resolved_at: Instant::now() - RUNTIME_PATH_REFRESH,
            last_check: Instant::now() - RUNTIME_PROBE_INTERVAL,
            last_mtime: None,
            last_sent: None,
        }
    }

    /// Rend une observation à transmettre, ou `None` s'il n'y a rien de neuf.
    fn poll(&mut self) -> Option<crate::runtime::RuntimeObservation> {
        if self.last_check.elapsed() < RUNTIME_PROBE_INTERVAL {
            return None;
        }
        self.last_check = Instant::now();

        let stale_path = self
            .path
            .as_ref()
            .map(|path| !path.exists())
            .unwrap_or(true);
        if stale_path || self.path_resolved_at.elapsed() >= RUNTIME_PATH_REFRESH {
            let resolved = crate::runtime::open_session_file(self.pid);
            // Un changement de fichier invalide la date de modification
            // mémorisée : sans cela, un nouveau rollout dont la mtime coïncide
            // avec celle de l'ancien ne serait jamais lu. Défaut soulevé par
            // la contre-revue « agent-1 ».
            if resolved != self.path {
                debug!("sonde runtime : fichier de session {:?}", resolved);
                self.last_mtime = None;
            }
            self.path = resolved;
            self.path_resolved_at = Instant::now();
            if self.path.is_none() {
                debug!("sonde runtime : aucun fichier de session pour le pid {}", self.pid);
            }
        }

        let path = self.path.as_ref()?;
        let mtime = std::fs::metadata(path).ok()?.modified().ok()?;
        if self.last_mtime == Some(mtime) {
            return None;
        }
        self.last_mtime = Some(mtime);

        let observed = crate::runtime::parse_codex_rollout(path)?;
        if self.last_sent.as_ref() == Some(&observed) {
            return None;
        }
        self.last_sent = Some(observed.clone());
        Some(observed)
    }
}

/// Ouvre une connexion vers le daemon et enregistre le wrapper.
///
/// `name = None` laisse le daemon attribuer le nom initial. Après une
/// reconnexion, le wrapper passe son nom établi afin de reprendre son identité.
fn connect_and_register(
    agent_type: &str,
    name: Option<&str>,
    host: &str,
    transport: &str,
    os: &str,
    instance_id: &str,
    domain: Option<&str>,
) -> Result<(BufReader<UnixStream>, BufWriter<UnixStream>, String), String> {
    let stream = UnixStream::connect(socket_path()).map_err(|e| e.to_string())?;
    set_cloexec(&stream);
    let read_stream = stream.try_clone().map_err(|e| e.to_string())?;
    set_cloexec(&read_stream);
    read_stream
        .set_read_timeout(Some(Duration::from_secs(1)))
        .map_err(|e| e.to_string())?;
    let write_stream = stream.try_clone().map_err(|e| e.to_string())?;
    set_cloexec(&write_stream);
    let mut writer = BufWriter::new(write_stream);
    let mut reader = BufReader::new(read_stream);

    let register = WrapperToDaemon::Register {
        agent_type: agent_type.to_string(),
        name: name.map(str::to_owned),
        host: Some(host.to_string()),
        transport: Some(transport.to_string()),
        os: Some(os.to_string()),
        instance_id: Some(instance_id.to_string()),
        domain: domain.map(str::to_owned),
    };
    writeln!(writer, "{}", encode(&register).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;

    let mut line = String::new();
    reader.read_line(&mut line).map_err(|e| e.to_string())?;
    match decode(line.trim()).map_err(|e| e.to_string())? {
        DaemonToWrapper::Registered { name } => Ok((reader, writer, name)),
        DaemonToWrapper::Nack { reason, .. } => Err(format!("enregistrement refusé: {}", reason)),
        other => Err(format!("réponse inattendue: {:?}", other)),
    }
}

/// Lance un agent CLI wrapper.
/// `agent_binary` = nom de la commande à lancer ("codex", "claude", etc.)
/// `agent_type` = type pour le daemon ("codex", "claude", "custom")
/// `agent_args` = arguments à passer à l'agent CLI
/// `explicit_name` = nom optionnel (--name)
pub fn launch(
    agent_binary: &str,
    agent_type: &str,
    agent_args: &[String],
    explicit_name: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Enregistrement initial — avec persistance du nom.
    // Si l'utilisateur a passé --name, on l'utilise.
    // Sinon, si on fait un resume, on essaie de retrouver le nom précédent.
    let effective_name = if let Some(n) = explicit_name {
        Some(n.to_string())
    } else {
        load_persistent_name(agent_type, agent_args)
    };

    let host = host_name();
    let transport = transport_name();
    let os = operating_system();
    let instance_id = uuid::Uuid::new_v4().to_string();
    // Au premier enregistrement, le nom définitif n'est pas encore connu : si
    // l'utilisateur en a demandé un, sa surcharge de domaine est déjà lisible,
    // sinon on part du domaine dérivé.
    let initial_domain = match effective_name.as_deref() {
        Some(name) => effective_domain(name),
        None => derive_domain(),
    };
    let (reader, initial_writer, my_name) = connect_and_register(
        agent_type,
        effective_name.as_deref(),
        &host,
        &transport,
        &os,
        &instance_id,
        initial_domain.as_deref(),
    )?;
    let writer = Arc::new(Mutex::new(Some(initial_writer)));

    eprintln!("[bridget] Enregistré en tant que « {} »", my_name);
    info!("enregistré: {}", my_name);

    // Sauvegarder le nom pour les futurs resume
    save_persistent_name(agent_type, agent_args, &my_name);
    let name_state_path = {
        let hash = session_hash(agent_args);
        if hash.is_empty() {
            socket_path()
                .parent()
                .unwrap()
                .join("agent-names")
                .join(format!("active-{}", my_name))
        } else {
            persistent_name_path(&hash)
        }
    };
    if let Some(parent) = name_state_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&name_state_path, &my_name)?;

    // 3. Détection du pane tmux
    let pane_id = get_current_pane_id().unwrap_or_else(|e| {
        warn!("pas de pane tmux: {}", e);
        String::new()
    });

    // 4. Lancer l'agent CLI
    // Pour Codex : ajouter automatiquement --dangerously-bypass-approvals-and-sandbox
    // (= --yolo) sinon le sandbox bloque la connexion socket vers le daemon.
    // Pour Claude Code : ajouter --dangerously-skip-permissions --permission-mode bypassPermissions
    // + injecter un prompt initial qui dit à l'agent de répondre via bridget.
    let mut final_args: Vec<String> = Vec::new();
    if agent_type == "codex" {
        // Vérifier si l'utilisateur n'a pas déjà passé --yolo ou le bypass
        let already_bypassed = agent_args
            .iter()
            .any(|a| a == "--yolo" || a == "--dangerously-bypass-approvals-and-sandbox");
        if !already_bypassed {
            final_args.push("--dangerously-bypass-approvals-and-sandbox".to_string());
        }

        // Ne PAS utiliser --cd : ça change le working directory de l'agent.
        // Les instructions bridget viennent du prompt initial ci-dessous.
        // Le prompt initial est persistant dans le transcript et survive
        // le compaction de contexte (contrairement à un message système).
    } else if agent_type == "claude" {
        // Pour Claude Code (claude et gclaude)
        let already_bypassed = agent_args
            .iter()
            .any(|a| a.contains("dangerously-skip-permissions") || a.contains("bypassPermissions"));
        if !already_bypassed {
            final_args.push("--dangerously-skip-permissions".to_string());
            final_args.push("--permission-mode".to_string());
            final_args.push("bypassPermissions".to_string());
        }
    }

    // Si l'utilisateur n'a pas passé de prompt initial (un argument libre
    // qui n'est pas un flag --xxx), injecter le prompt bridget.
    let has_prompt = agent_args.iter().any(|a| !a.starts_with("--"));
    if !has_prompt && (agent_type == "codex" || agent_type == "claude") {
        let bridget_prompt = format!(
            "Tu es l'agent \"{}\" dans une session Bridget. Tu peux recevoir des messages d'autres agents IA. Quand tu vois une ligne qui commence par 💬 dans ton terminal, c'est un message d'un autre agent IA, pas de l'humain. Le format est : 💬 <expéditeur> → <toi> (reply=yes/no, ...)\n<message>\n\nRègles ABSOLUES :\n1. Réponds TOUJOURS avec: bridget send --to <expéditeur> \"ta réponse\"\n2. N'accuse JAMAIS réception (pas de \"bien reçu\", \"OK\", \"compris\")\n3. Ne fais PAS bridget who, bridget ledger ou bridget --help sans y être explicitement invité\n4. reply=no = notification, ne réponds que si utile. reply=yes = réponds avec du contenu.\n5. Ne réponds JAMAIS uniquement dans ton terminal — l'expéditeur ne te verrait pas.\n\nTu es maintenant en attente. Dis \"Bridget ready\" puis attends les messages.",
            my_name
        );
        final_args.push(bridget_prompt);
    }

    final_args.extend(agent_args.iter().cloned());

    // VALIDATION DE SÉCURITÉ : whitelist stricte des binaires autorisés
    let allowed_binaries = ["codex", "claude", "gemini", "gclaude", "claude-son"];
    let binary_name = agent_binary.split('/').last().unwrap_or(agent_binary);
    if !allowed_binaries.contains(&binary_name) {
        return Err(format!(
            "Binaire non autorisé '{}'. Binaires permis: {}",
            agent_binary,
            allowed_binaries.join(", ")
        ).into());
    }

    // Validation des arguments pour prévenir injection
    for arg in &final_args {
        // Rejeter les tentatives d'injection de commandes
        if arg.contains(';') || arg.contains('&') || arg.contains('|') || arg.contains('$') {
            return Err(format!(
                "Argument non autorisé contient des caractères shell dangereux: '{}'",
                arg
            ).into());
        }
    }

    eprintln!(
        "[bridget] Lancement: {} {}",
        agent_binary,
        final_args.join(" ")
    );

    let mut child = Command::new(agent_binary)
        .args(&final_args)
        .env("BRIDGET_AGENT_NAME", &my_name)
        .env("BRIDGET_AGENT_NAME_FILE", &name_state_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("impossible de lancer '{}': {}", agent_binary, e))?;

    let agent_pid = child.id();

    // 5. Thread d'écoute
    let writer_clone = writer.clone();
    let writer_for_listener = writer_clone.clone();
    let pane_for_thread = pane_id.clone();
    let name_state_for_thread = name_state_path.clone();
    let agent_type_for_thread = agent_type.to_string();
    let mut my_name_for_thread = my_name.clone();
    let host_for_thread = host.clone();
    let transport_for_thread = transport.clone();
    let os_for_thread = os.clone();
    let instance_id_for_thread = instance_id.clone();
    let stopping = Arc::new(AtomicBool::new(false));
    let stopping_for_thread = stopping.clone();

    let listener_handle = thread::spawn(move || {
        let mut listener = reader;
        let mut transport = if !pane_for_thread.is_empty() {
            Some(TmuxTransport::new(pane_for_thread.clone(), agent_pid))
        } else {
            None
        };
        let mut connected_since = Instant::now();
        let mut failed_attempts = 0_u32;
        let mut last_heartbeat = Instant::now();
        // Seul Codex tient son fichier de session ouvert ; pour Claude, c'est
        // le hook `Stop` qui rapporte le runtime (research.md D-002).
        let mut runtime_probe = (agent_type_for_thread == "codex")
            .then(|| RuntimeProbe::new(agent_pid));

        'connection: while !stopping_for_thread.load(Ordering::SeqCst) {
            let mut line = String::new();
            match listener.read_line(&mut line) {
                Ok(0) => {
                    // Connexion fermée par le daemon
                    warn!("connexion fermée par le daemon");
                }
                Ok(_) => {
                    // Données reçues, traiter plus bas
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    // Timeout de lecture - vérifier heartbeat et socket
                    if last_heartbeat.elapsed() >= HEARTBEAT_INTERVAL {
                        let heartbeat = encode(&WrapperToDaemon::Heartbeat).unwrap_or_default();
                        if let Some(writer) = writer_for_listener.lock().unwrap().as_mut() {
                            match writeln!(writer, "{}", heartbeat).and_then(|_| writer.flush()) {
                                Ok(_) => {
                                    last_heartbeat = Instant::now();
                                    debug!("heartbeat envoyé avec succès");
                                }
                                Err(e) => {
                                    error!("heartbeat échoué: {}", e);
                                    // Heartbeat échoué = probablement déconnecté
                                    warn!("Détection de déconnexion via heartbeat échoué");
                                }
                            }
                        } else {
                            error!("Impossible d'obtenir le writer pour heartbeat");
                        }
                    }

                    // Sonde de runtime : greffée sur le même réveil que le
                    // heartbeat, elle ne coûte qu'un `stat` la plupart du temps.
                    if let Some(observed) = runtime_probe.as_mut().and_then(RuntimeProbe::poll) {
                        // Le nom est relu à chaque émission : `bridget rename`
                        // met à jour ce fichier, pas la variable capturée au
                        // démarrage. S'adresser au nom initial vaudrait un
                        // « agent introuvable » sur tout agent renommé.
                        let current_name = std::fs::read_to_string(&name_state_for_thread)
                            .map(|name| name.trim().to_string())
                            .ok()
                            .filter(|name| !name.is_empty())
                            .unwrap_or_else(|| my_name_for_thread.clone());
                        let message = WrapperToDaemon::Runtime {
                            agent: current_name,
                            model: observed.model.clone(),
                            effort: observed.effort.clone(),
                            source: bridget_transport::protocol::RuntimeSource::CodexRollout,
                        };
                        match encode(&message) {
                            Ok(json) => {
                                if let Some(writer) = writer_for_listener.lock().unwrap().as_mut() {
                                    if let Err(e) =
                                        writeln!(writer, "{}", json).and_then(|_| writer.flush())
                                    {
                                        debug!("sonde runtime : envoi impossible ({})", e);
                                    } else {
                                        debug!(
                                            "sonde runtime : modèle={} effort={:?}",
                                            observed.model, observed.effort
                                        );
                                    }
                                }
                            }
                            Err(e) => debug!("sonde runtime : encodage impossible ({})", e),
                        }
                    }

                    // Vérification proactive du socket (auto-reconnect)
                    // Vérifier toutes les X secondes si le socket existe toujours
                    if Instant::now().duration_since(last_heartbeat).as_secs() % SOCKET_CHECK_INTERVAL.as_secs() == 0 {
                        if socket_path().exists() {
                            debug!("Socket Bridget détecté - daemon probablement disponible");
                        } else {
                            warn!("⚠️ Socket Bridget absent - daemon probablement arrêté");
                        }
                    }

                    continue;
                }
                Err(e) => {
                    error!("Erreur de lecture: {}", e);
                    // Erreur de lecture = probablement déconnecté
                }
            }

            if line.is_empty() {
                // Détection de déconnexion
                if stopping_for_thread.load(Ordering::SeqCst) {
                    info!("Arrêt demandé, déconnexion propre");
                    break;
                }

                warn!(
                    "🔌 Connexion Bridget perdue pour « {} » - reconnexion automatique...",
                    my_name_for_thread
                );

                // Reset du compteur si la connexion était stable
                if connected_since.elapsed() >= RECONNECT_STABLE_RESET {
                    failed_attempts = 0;
                    info!("Connexion était stable - reset du compteur de tentatives");
                }

                // Boucle de reconnexion avec backoff exponentiel
                loop {
                    if stopping_for_thread.load(Ordering::SeqCst) {
                        info!("Arrêt demandé pendant reconnexion");
                        break 'connection;
                    }

                    let delay = reconnect_delay(failed_attempts);
                    failed_attempts = failed_attempts.saturating_add(1);

                    // Le nom choisi par l'utilisateur prime sur celui obtenu au
                    // démarrage : sans cela, un agent renommé revient sous son
                    // nom d'origine à chaque coupure.
                    let wanted_name =
                        resolve_current_name(&name_state_for_thread, &my_name_for_thread);

                    info!(
                        "🔄 Tentative de reconnexion {} pour « {} » (délai: {:.1}s)",
                        failed_attempts, wanted_name, delay.as_secs_f64()
                    );

                    thread::sleep(delay);
                    match connect_and_register(
                        &agent_type_for_thread,
                        Some(&wanted_name),
                        &host_for_thread,
                        &transport_for_thread,
                        &os_for_thread,
                        &instance_id_for_thread,
                        effective_domain(&wanted_name).as_deref(),
                    ) {
                        Ok((new_reader, new_writer, registered_name)) => {
                            if registered_name != wanted_name {
                                warn!(
                                    "⚠️ Reconnexion refusée : nom inattendu « {} » (attendu: « {} »)",
                                    registered_name, wanted_name
                                );
                                continue;
                            }

                            // Reconnexion réussie ! 🎉
                            *writer_for_listener.lock().unwrap() = Some(new_writer);
                            listener = new_reader;
                            connected_since = Instant::now();
                            last_heartbeat = Instant::now();
                            failed_attempts = 0; // Reset du compteur
                            my_name_for_thread = registered_name;

                            info!(
                                "✅ Agent « {} » reconnecté au daemon avec succès !",
                                my_name_for_thread
                            );

                            // Notification visuelle à l'utilisateur (si tmux)
                            if let Some(ref mut t) = transport {
                                let notif = format!("🔄 Bridget: reconnecté au daemon");
                                if let Err(e) = t.deliver(&bridget_core::BridgetMessage::new(
                                    "bridget", &my_name_for_thread, &notif
                                )) {
                                    error!("Impossible d'afficher la notification de reconnexion: {}", e);
                                }
                            }

                            continue 'connection;
                        }
                        Err(error) => {
                            warn!(
                                "❌ Reconnexion échouée pour « {} » : {} (tentative {})",
                                wanted_name, error, failed_attempts
                            );
                        }
                    }
                }
            }
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let msg: DaemonToWrapper = match decode(line) {
                Ok(m) => m,
                Err(e) => {
                    warn!("message illisible: {}", e);
                    continue;
                }
            };

            match msg {
                DaemonToWrapper::Deliver(bm) => {
                    info!(
                        "reçu de « {} »: {}",
                        bm.from,
                        bm.body.chars().take(60).collect::<String>()
                    );
                    // Stocker le dernier expéditeur pour la commande reply
                    let name_for_reply = std::fs::read_to_string(&name_state_for_thread)
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    if bm.reply && !name_for_reply.is_empty() {
                        let reply_file = socket_path()
                            .parent()
                            .unwrap()
                            .join(format!("last-sender-{}", name_for_reply));
                        let value = if bm.reply {
                            format!("{}\t{}", bm.from, bm.id)
                        } else {
                            bm.from.clone()
                        };
                        let _ = std::fs::write(&reply_file, value);
                    }
                    if let Some(ref mut t) = transport {
                        if let Err(e) = t.deliver(&bm) {
                            error!("injection tmux: {}", e);
                        }
                    } else {
                        eprintln!("\n[bridget] ← « {} »: {}\n", bm.from, bm.body);
                    }
                }
                DaemonToWrapper::Disconnect => {
                    info!("daemon déconnecté");
                    if connected_since.elapsed() >= RECONNECT_STABLE_RESET {
                        failed_attempts = 0;
                    }
                    loop {
                        if stopping_for_thread.load(Ordering::SeqCst) {
                            break 'connection;
                        }
                        let delay = reconnect_delay(failed_attempts);
                        failed_attempts = failed_attempts.saturating_add(1);
                        thread::sleep(delay);
                        let wanted_name =
                            resolve_current_name(&name_state_for_thread, &my_name_for_thread);
                        match connect_and_register(
                            &agent_type_for_thread,
                            Some(&wanted_name),
                            &host_for_thread,
                            &transport_for_thread,
                            &os_for_thread,
                            &instance_id_for_thread,
                            effective_domain(&wanted_name).as_deref(),
                        ) {
                            Ok((new_reader, new_writer, registered_name))
                                if registered_name == wanted_name =>
                            {
                                *writer_for_listener.lock().unwrap() = Some(new_writer);
                                listener = new_reader;
                                connected_since = Instant::now();
                                last_heartbeat = Instant::now();
                                my_name_for_thread = registered_name;
                                continue 'connection;
                            }
                            Ok((_, _, registered_name)) => warn!(
                                "reconnexion refusée : nom inattendu « {} »",
                                registered_name
                            ),
                            Err(error) => warn!(
                                "reconnexion Bridget de « {} » impossible : {}",
                                wanted_name, error
                            ),
                        }
                    }
                }
                _ => {}
            }
        }
    });

    // 6. Attendre la fin de l'agent
    let status = child.wait()?;

    // 7. Désenregistrement
    stopping.store(true, Ordering::SeqCst);
    {
        if let Ok(json) = encode(&WrapperToDaemon::Unregister) {
            if let Ok(mut writer) = writer_clone.lock() {
                if let Some(w) = writer.as_mut() {
                    let _ = writeln!(w, "{}", json);
                    let _ = w.flush();
                }
            }
        }
    }

    *writer.lock().unwrap() = None;
    let _ = listener_handle.join();

    if let Some(code) = status.code() {
        std::process::exit(code);
    }
    std::process::exit(0);
}

#[cfg(test)]
mod reconnect_tests {
    use super::*;

    #[test]
    fn le_domaine_derive_nomme_le_depot_courant() {
        // Le test s'exécute dans le dépôt Bridget : la racine git doit donner
        // son nom, sans configuration ni embellissement.
        assert_eq!(derive_domain().as_deref(), Some("bridget"));
    }

    #[test]
    fn le_domaine_surcharge_prime_sur_le_derive() {
        let path = domain_state_path("agent-de-test-domaine");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, "revue-croisee\n").unwrap();
        assert_eq!(
            effective_domain("agent-de-test-domaine").as_deref(),
            Some("revue-croisee")
        );

        // Sans trace disque, on retombe sur le domaine dérivé.
        std::fs::remove_file(&path).unwrap();
        assert_eq!(
            effective_domain("agent-de-test-domaine"),
            derive_domain()
        );
    }

    #[test]
    fn le_nom_choisi_par_l_utilisateur_survit_a_la_reconnexion() {
        // `bridget rename` n'écrit que dans ce fichier ; le wrapper doit s'y
        // référer, sinon un agent renommé revient sous son nom d'origine à
        // chaque coupure — y compris une rupture de tunnel en fédération SSH.
        let path = std::env::temp_dir().join(format!(
            "bridget-nom-{}-{}",
            std::process::id(),
            "renomme"
        ));
        std::fs::write(&path, "agent-1\n").unwrap();
        assert_eq!(resolve_current_name(&path, "codex-8"), "agent-1");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn nom_illisible_ou_vide_retombe_sur_le_nom_d_enregistrement() {
        let absent = std::env::temp_dir().join("bridget-nom-inexistant-xyz");
        let _ = std::fs::remove_file(&absent);
        assert_eq!(resolve_current_name(&absent, "codex-8"), "codex-8");

        let vide = std::env::temp_dir().join(format!("bridget-nom-vide-{}", std::process::id()));
        std::fs::write(&vide, "   \n").unwrap();
        assert_eq!(resolve_current_name(&vide, "codex-8"), "codex-8");
        let _ = std::fs::remove_file(&vide);
    }

    #[test]
    fn reconnect_delay_grows_then_stays_capped() {
        let first = reconnect_delay(0);
        let second = reconnect_delay(1);
        let sixth = reconnect_delay(5);
        let later = reconnect_delay(20);

        assert!(first <= Duration::from_millis(1200));
        assert!(second >= Duration::from_millis(1600));
        assert!(sixth <= RECONNECT_MAX_DELAY);
        assert!(later <= RECONNECT_MAX_DELAY);
    }
}
