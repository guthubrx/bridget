//! CLI — point d'entrée unifié pour toutes les sous-commandes bridget.

use crate::daemon::{self, DaemonConfig};
use bridget_core::BridgetMessage;
use bridget_transport::protocol::{
    AgentInfo, AttachWindow, LedgerMessage, LedgerScope, RuntimeSource, decode, encode,
};
use bridget_transport::{DaemonToWrapper, SpawnRefusal, StopOutcome, WrapperToDaemon};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

/// Fonction générique pour lancer un agent wrapper (M-001)
fn launch_agent_wrapper(binary: &str, agent_type: &str, args: &[String]) -> ! {
    let (name, rest) = extract_wrapper_args(args);
    if let Err(e) = crate::wrapper::launch(binary, agent_type, &rest, name.as_deref()) {
        eprintln!("bridget: {}", e);
        std::process::exit(1);
    }
    std::process::exit(0);
}

// Constantes de validation (H-001)
const MAX_MESSAGE_LENGTH: usize = 10000;
const MAX_AGENT_NAME_LENGTH: usize = 100;

/// Délai maximal d'attente d'une réponse du daemon pour une observation de
/// runtime. Court volontairement : l'appelant est un hook exécuté dans la
/// boucle de l'agent, il ne doit jamais le faire patienter.
const RUNTIME_REPLY_TIMEOUT_SECS: u64 = 2;

/// Valide un nom d'agent Bridget (H-001)
fn validate_agent_name(name: &str) -> Result<(), String> {
    if name.len() > MAX_AGENT_NAME_LENGTH {
        return Err(format!("nom d'agent trop long (max {} caractères)", MAX_AGENT_NAME_LENGTH));
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err("nom d'agent contient des caractères invalides (alphanumériques, -, _ uniquement)".to_string());
    }
    Ok(())
}

/// Valide le corps d'un message (H-001)
fn validate_message_body(body: &str) -> Result<(), String> {
    if body.len() > MAX_MESSAGE_LENGTH {
        return Err(format!("message trop long (max {} caractères)", MAX_MESSAGE_LENGTH));
    }
    // Vérifier les caractères de contrôle potentiellement dangereux
    if body.contains('\x00') || body.contains('\x1b') {
        return Err("message contient des caractères de contrôle invalides".to_string());
    }
    Ok(())
}

pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(2);
    }

    let cmd = &args[1];

    // Les lanceurs historiques restent interactifs ; leur type et leur
    // autorisation viennent désormais du registre, pas d'une liste CLI.
    if let Some(agent_type) = crate::registry::AgentRegistry::interactive_alias(cmd) {
        launch_agent_wrapper(cmd, agent_type, &args[2..]);
    }

    // --- Sous-commande wrapper générique ---
    if cmd == "--" {
        if args.len() < 3 {
            eprintln!("bridget: commande manquante après --");
            std::process::exit(2);
        }
        let cmd = &args[2];
        let rest = &args[3..];
        let registry = crate::registry::AgentRegistry::load().unwrap_or_else(|error| {
            eprintln!("bridget: {error}");
            std::process::exit(1);
        });
        let agent_type = registry.type_for_command(cmd).unwrap_or_else(|error| {
            eprintln!("bridget: {error}");
            std::process::exit(1);
        });
        launch_agent_wrapper(cmd, &agent_type, rest);
    }

    // --- Sous-commandes daemon / client ---
    match cmd.as_str() {
        "daemon" => cmd_daemon(),
        "managed-bootstrap" => cmd_managed_bootstrap(&args[2..]),
        "managed-wrapper" => cmd_managed_wrapper(&args[2..]),
        "mcp" => cmd_mcp(),
        "attach" => cmd_attach(&args[2..]),
        "spawn" => cmd_spawn(&args[2..]),
        "stop" => cmd_stop(&args[2..]),
        "send" => cmd_send(&args[2..]),
        "cancel" => cmd_cancel(&args[2..]),
        "requests" => cmd_requests(&args[2..]),
        "rename" => cmd_rename(&args[2..]),
        "runtime" => cmd_runtime(&args[2..]),
        "domain" => cmd_domain(&args[2..]),
        "dnd" => cmd_dnd(&args[2..]),
        "hook" => cmd_hook(&args[2..]),
        "install-hooks" => cmd_install_hooks(&args[2..]),
        "reply" => cmd_reply(&args[2..]),
        "who" => cmd_who(&args[2..]),
        "agents" => cmd_agents(&args[2..]),
        "discover" => cmd_discover(),
        "status" => cmd_status(),
        "ledger" => cmd_ledger(),
        "version" | "--version" | "-v" => {
            println!("bridget {}", env!("CARGO_PKG_VERSION"));
        }
        "help" | "--help" | "-h" => print_usage(),
        _ => {
            // Si c'est une commande inconnue mais qu'elle existe dans le PATH,
            // la traiter comme un agent personnalisé
            if which(cmd) {
                let registry = crate::registry::AgentRegistry::load().unwrap_or_else(|error| {
                    eprintln!("bridget: {error}");
                    std::process::exit(1);
                });
                let agent_type = registry.type_for_command(cmd).unwrap_or_else(|error| {
                    eprintln!("bridget: {error}");
                    std::process::exit(1);
                });
                launch_agent_wrapper(cmd, &agent_type, &args[2..]);
            } else {
                eprintln!("sous-commande inconnue: {}", cmd);
                print_usage();
                std::process::exit(2);
            }
        }
    }
}

fn cmd_mcp() {
    if let Err(error) = crate::mcp::run_stdio() {
        eprintln!("bridget mcp: {error}");
        std::process::exit(1);
    }
}

fn cmd_managed_bootstrap(args: &[String]) {
    if let Err(error) = crate::managed_process::run_managed_bootstrap(args) {
        eprintln!("bridget managed-bootstrap: {error}");
        std::process::exit(1);
    }
}

fn cmd_managed_wrapper(args: &[String]) {
    if args.len() != 2 {
        eprintln!("bridget managed-wrapper: type et nom requis");
        std::process::exit(2);
    }
    if let Err(error) = crate::wrapper::launch_managed_acp(&args[0], &args[1]) {
        eprintln!("bridget managed-wrapper: {error}");
        std::process::exit(1);
    }
}

/// Extrait --name des arguments du wrapper et retourne (name_option, args_restants).
fn extract_wrapper_args(args: &[String]) -> (Option<String>, Vec<String>) {
    let mut name = None;
    let mut rest = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--name" && i + 1 < args.len() {
            name = Some(args[i + 1].clone());
            i += 2;
        } else {
            rest.push(args[i].clone());
            i += 1;
        }
    }
    (name, rest)
}

fn which(cmd: &str) -> bool {
    if let Ok(path) = std::env::var("PATH") {
        for dir in path.split(':') {
            let full = std::path::Path::new(dir).join(cmd);
            if full.exists()
                && std::fs::metadata(&full)
                    .map(|m| m.is_file())
                    .unwrap_or(false)
            {
                return true;
            }
        }
    }
    false
}

fn print_usage() {
    eprintln!(
        "bridget 0.1.0 — protocole de communication inter-agents\n\n\
         Usage: bridget <COMMANDE> [OPTIONS]\n\n\
         Agents CLI (wrappers) :\n  \
           codex [ARGS...]        Lance Codex + connexion daemon\n  \
           claude [ARGS...]       Lance Claude + connexion daemon\n  \
           gemini [ARGS...]       Lance Gemini + connexion daemon\n  \
           gclaude [ARGS...]      Lance gclaude + connexion daemon\n  \
           -- <CMD> [ARGS...]     Agent personnalisé\n\n\
         Daemon & client :\n  \
           daemon                 Lance le daemon\n  \
           mcp                    Lance le serveur MCP sur stdio\n  \
           attach <N>             Suit un équipier [--from-seq N | --date AAAA-MM-JJ]\n  \
           spawn <TYPE>           Lance un équipier géré [--name N] [--persistent]\n  \
           stop <N>               Arrête un équipier géré\n  \
           send --to <N> <MSG>    Envoie un message\n  \
           reply <MSG>            Répond au dernier expéditeur\n  \
           cancel <ID>            Annule une demande suivie [--reason <T>]\n  \
           requests               Liste mes demandes suivies\n  \
           rename <N>             Renomme l'agent courant\n  \
           runtime --model <M>    Déclare le modèle courant [--effort <E>]\n  \
           domain <N> | --reset   Change le domaine de l'agent courant\n  \
           dnd [off]              Ne pas déranger [--duration 30m]\n  \
           install-hooks          Installe la détection auto du modèle (Claude)\n  \
           who [--domain <D>]     Agents connectés\n  \
           agents [--json]        Idem, format machine [--domain <D>]\n  \
           status                 Santé du daemon\n  \
           ledger                 Historique des messages\n  \
           version                Version\n  \
           help                   Cette aide\n\n\
         Options de send :\n  \
           --to <nom>             Destinataire (requis)\n  \
           --reply                Réponse attendue\n  \
           --timeout <S>          Délai avant échec (défaut: 60)\n  \
           --hops <N>             Sauts restants (défaut: 4)\n\n\
         Usage interne :\n  \
           hook claude-runtime    Appelé par le hook Claude Code, lit stdin"
    );
}

fn socket_path() -> std::path::PathBuf {
    DaemonConfig::default().socket_path
}

fn cmd_attach(args: &[String]) {
    let (agent, window) = match parse_attach_args(args) {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("usage: bridget attach <nom> [--from-seq N | --date AAAA-MM-JJ]");
            eprintln!("erreur: {error}");
            std::process::exit(2);
        }
    };
    if let Err(error) = crate::attach::run(&agent, window, &socket_path()) {
        eprintln!("bridget attach: {error}");
        std::process::exit(1);
    }
}

const DEFAULT_SPAWN_TIMEOUT_SECS: i64 = 10;

#[derive(Debug)]
struct ParsedSpawnArgs {
    agent_type: String,
    name: Option<String>,
    cwd: Option<std::path::PathBuf>,
    persistent: bool,
    persistent_was_set: bool,
    command_id: Option<String>,
    timeout_secs: i64,
    timeout_was_set: bool,
}

fn cmd_spawn(args: &[String]) {
    let parsed = parse_spawn_args(args).unwrap_or_else(|error| {
        eprintln!(
            "usage: bridget spawn <type> [--name N] [--cwd CHEMIN] [--persistent] \
             [--timeout S] [--command-id ID]"
        );
        eprintln!("erreur: {error}");
        std::process::exit(2);
    });
    let now = unix_timestamp();
    let current_dir = std::env::current_dir().unwrap_or_else(|error| {
        eprintln!("bridget spawn: cwd inaccessible: {error}");
        std::process::exit(1);
    });
    let home = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            eprintln!("bridget spawn: HOME absent");
            std::process::exit(1);
        });
    let order = resolve_spawn_order(&parsed, now, &current_dir, &home).unwrap_or_else(|error| {
        eprintln!("bridget spawn: {error}");
        std::process::exit(1);
    });
    let command_id = match &order {
        WrapperToDaemon::SpawnOrder { command_id, .. } => command_id.clone(),
        _ => unreachable!("resolve_spawn_order ne produit qu'un SpawnOrder"),
    };
    println!("command_id: {command_id}");
    match send_control_to_daemon(order) {
        Ok(DaemonToWrapper::SpawnAccepted { name, .. }) => {
            println!("Équipier connecté : {name}");
        }
        Ok(DaemonToWrapper::SpawnRejected { reason, .. }) => {
            eprintln!("SPAWN REFUSÉ: {}", display_spawn_refusal(&reason));
            std::process::exit(1);
        }
        Ok(DaemonToWrapper::IdempotencyResult {
            issue: bridget_transport::protocol::IdempotencyIssue::EnvelopeMismatch,
            ..
        }) => {
            eprintln!("SPAWN REFUSÉ: command_id déjà associé à une autre enveloppe");
            std::process::exit(1);
        }
        Ok(other) => {
            eprintln!("réponse spawn inattendue du daemon: {other:?}");
            std::process::exit(1);
        }
        Err(error) => {
            eprintln!("daemon inaccessible: {error}");
            std::process::exit(1);
        }
    }
}

fn cmd_stop(args: &[String]) {
    let (name, command_id) = parse_stop_args(args).unwrap_or_else(|error| {
        eprintln!("usage: bridget stop <nom> [--command-id ID]");
        eprintln!("erreur: {error}");
        std::process::exit(2);
    });
    println!("command_id: {command_id}");
    match send_control_to_daemon(WrapperToDaemon::StopOrder {
        name,
        command_id: command_id.clone(),
    }) {
        Ok(DaemonToWrapper::StopResult { outcome, .. }) => match outcome {
            StopOutcome::Stopped => println!("Équipier arrêté proprement."),
            StopOutcome::StoppedForced { survivors_killed } => println!(
                "Équipier arrêté de force ({survivors_killed} processus survivants terminés)."
            ),
            StopOutcome::NotManaged => {
                eprintln!("STOP REFUSÉ: l'agent n'est pas géré par le daemon");
                std::process::exit(1);
            }
            StopOutcome::NotFound => {
                eprintln!("STOP REFUSÉ: équipier introuvable");
                std::process::exit(1);
            }
            StopOutcome::Timeout { state } => {
                eprintln!("STOP INCOMPLET: délai dépassé dans l'état {state}");
                std::process::exit(1);
            }
        },
        Ok(other) => {
            eprintln!("réponse stop inattendue du daemon: {other:?}");
            std::process::exit(1);
        }
        Err(error) => {
            eprintln!("daemon inaccessible: {error}");
            std::process::exit(1);
        }
    }
}

fn parse_spawn_args(args: &[String]) -> Result<ParsedSpawnArgs, String> {
    let agent_type = args
        .first()
        .filter(|value| !value.starts_with('-'))
        .cloned()
        .ok_or_else(|| "type d'agent manquant".to_string())?;
    validate_agent_name(&agent_type)?;
    let mut parsed = ParsedSpawnArgs {
        agent_type,
        name: None,
        cwd: None,
        persistent: false,
        persistent_was_set: false,
        command_id: None,
        timeout_secs: DEFAULT_SPAWN_TIMEOUT_SECS,
        timeout_was_set: false,
    };
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--persistent" => {
                parsed.persistent = true;
                parsed.persistent_was_set = true;
                index += 1;
            }
            "--name" | "--cwd" | "--command-id" | "--timeout" => {
                let option = args[index].as_str();
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| format!("valeur manquante pour {option}"))?;
                match option {
                    "--name" => {
                        validate_agent_name(value)?;
                        parsed.name = Some(value.clone());
                    }
                    "--cwd" => parsed.cwd = Some(std::path::PathBuf::from(value)),
                    "--command-id" => {
                        validate_command_id(value)?;
                        parsed.command_id = Some(value.clone());
                    }
                    "--timeout" => {
                        parsed.timeout_secs = value
                            .parse::<i64>()
                            .ok()
                            .filter(|seconds| *seconds > 0 && *seconds <= 600)
                            .ok_or_else(|| "--timeout doit être compris entre 1 et 600".to_string())?;
                        parsed.timeout_was_set = true;
                    }
                    _ => unreachable!(),
                }
                index += 2;
            }
            option => return Err(format!("option spawn inconnue: {option}")),
        }
    }
    Ok(parsed)
}

fn parse_stop_args(args: &[String]) -> Result<(String, String), String> {
    let name = args.first().cloned().ok_or_else(|| "nom manquant".to_string())?;
    validate_agent_name(&name)?;
    let command_id = match args.get(1).map(String::as_str) {
        None => uuid::Uuid::new_v4().to_string(),
        Some("--command-id") if args.len() == 3 => {
            validate_command_id(&args[2])?;
            args[2].clone()
        }
        Some(_) => return Err("options stop invalides".to_string()),
    };
    Ok((name, command_id))
}

fn resolve_spawn_order(
    parsed: &ParsedSpawnArgs,
    now: i64,
    current_dir: &std::path::Path,
    home: &std::path::Path,
) -> Result<WrapperToDaemon, String> {
    let command_id = parsed
        .command_id
        .clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let path = spawn_order_path(home, &command_id);
    if path.exists() {
        let content = std::fs::read_to_string(&path)
            .map_err(|error| format!("ordre mémorisé illisible {}: {error}", path.display()))?;
        let stored: WrapperToDaemon = decode(content.trim())
            .map_err(|error| format!("ordre mémorisé invalide {}: {error}", path.display()))?;
        validate_retry_options(parsed, &stored)?;
        return Ok(stored);
    }
    let cwd = parsed.cwd.as_deref().unwrap_or(current_dir);
    if !cwd.is_absolute() {
        return Err("--cwd doit être absolu".to_string());
    }
    if !cwd.is_dir() {
        return Err(format!("cwd absent ou non répertoire: {}", cwd.display()));
    }
    let order = WrapperToDaemon::SpawnOrder {
        agent_type: parsed.agent_type.clone(),
        name: parsed.name.clone(),
        cwd: cwd.to_string_lossy().into_owned(),
        persistent: parsed.persistent,
        command_id,
        issued_at: now,
        deadline_at: now.saturating_add(parsed.timeout_secs),
    };
    let bytes = format!("{}\n", encode(&order).map_err(|error| error.to_string())?).into_bytes();
    bridget_transport::fsutil::write_private_file_atomic(&path, &bytes)
        .map_err(|error| format!("mémorisation de l'ordre impossible {}: {error}", path.display()))?;
    Ok(order)
}

fn validate_retry_options(parsed: &ParsedSpawnArgs, stored: &WrapperToDaemon) -> Result<(), String> {
    let WrapperToDaemon::SpawnOrder {
        agent_type,
        name,
        cwd,
        persistent,
        ..
    } = stored
    else {
        return Err("le command_id mémorisé n'est pas un ordre spawn".to_string());
    };
    if &parsed.agent_type != agent_type
        || parsed.name.as_ref().is_some_and(|value| Some(value) != name.as_ref())
        || parsed
            .cwd
            .as_ref()
            .is_some_and(|value| value.to_string_lossy() != cwd.as_str())
        || (parsed.persistent_was_set && !persistent)
        || (parsed.timeout_was_set
            && stored_spawn_timeout(stored).is_some_and(|value| value != parsed.timeout_secs))
    {
        return Err("--command-id rejoué avec des options divergentes".to_string());
    }
    Ok(())
}

fn stored_spawn_timeout(stored: &WrapperToDaemon) -> Option<i64> {
    match stored {
        WrapperToDaemon::SpawnOrder {
            issued_at,
            deadline_at,
            ..
        } => deadline_at.checked_sub(*issued_at),
        _ => None,
    }
}

fn spawn_order_path(home: &std::path::Path, command_id: &str) -> std::path::PathBuf {
    home.join(".local/state/bridget/spawn-orders")
        .join(format!("{command_id}.json"))
}

fn validate_command_id(command_id: &str) -> Result<(), String> {
    if command_id.is_empty()
        || command_id.len() > 128
        || !command_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err("command_id invalide (1..128 caractères alphanumériques, - ou _)".to_string());
    }
    Ok(())
}

fn display_spawn_refusal(reason: &SpawnRefusal) -> String {
    match reason {
        SpawnRefusal::UnknownType => "type d'agent inconnu".to_string(),
        SpawnRefusal::CommandMissing { command, registry } => {
            format!("commande '{command}' introuvable (registre {registry})")
        }
        SpawnRefusal::BillingGuard { variable } => {
            format!("variable de facturation interdite présente: {variable}")
        }
        SpawnRefusal::NameActive => "nom déjà actif".to_string(),
        SpawnRefusal::EnvUnfit { detail } => format!("environnement inapte: {detail}"),
        SpawnRefusal::CwdGone => "répertoire de travail disparu".to_string(),
        SpawnRefusal::NegotiationFailed { detail } => format!("négociation échouée: {detail}"),
        SpawnRefusal::SpawnTimeout => "délai de lancement dépassé".to_string(),
        SpawnRefusal::QuotaExceeded { limit } => format!("quota de flotte atteint ({limit})"),
        SpawnRefusal::DaemonRecovering => "daemon en réconciliation".to_string(),
        SpawnRefusal::IdempotencyExpired => "command_id expiré".to_string(),
    }
}

fn unix_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn parse_attach_args(args: &[String]) -> Result<(String, AttachWindow), String> {
    let Some(agent) = args.first() else {
        return Err("nom d'équipier manquant".to_string());
    };
    validate_agent_name(agent)?;
    let window = match args.get(1).map(String::as_str) {
        None => AttachWindow::Today,
        Some("--from-seq") if args.len() == 3 => {
            let seq = args[2]
                .parse::<u64>()
                .map_err(|_| "--from-seq exige un entier positif ou nul".to_string())?;
            AttachWindow::Seq(seq)
        }
        Some("--date") if args.len() == 3 => AttachWindow::Date(args[2].clone()),
        Some(_) => return Err("options attach invalides ou incompatibles".to_string()),
    };
    Ok((agent.clone(), window))
}

fn cmd_rename(args: &[String]) {
    if args.len() != 1 || args[0].trim().is_empty() {
        eprintln!("usage: bridget rename <nouveau-nom>");
        std::process::exit(2);
    }

    // Validation du nouveau nom (H-001)
    if let Err(e) = validate_agent_name(&args[0]) {
        eprintln!("erreur: {}", e);
        std::process::exit(2);
    }

    let current_name = current_agent_name();
    if current_name == "human" {
        eprintln!("rename indisponible hors d'un agent Bridget");
        std::process::exit(1);
    }
    match send_rename_to_daemon(&current_name, &args[0]) {
        Ok(DaemonToWrapper::Renamed { old_name, name }) => {
            if let Ok(path) = std::env::var("BRIDGET_AGENT_NAME_FILE") {
                let _ = std::fs::write(path, &name);
            }
            let parent = socket_path().parent().unwrap().to_path_buf();
            let _ = std::fs::rename(
                parent.join(format!("last-sender-{}", old_name)),
                parent.join(format!("last-sender-{}", name)),
            );
            println!("Renommé : « {} » → « {} »", old_name, name);
        }
        Ok(DaemonToWrapper::Nack { reason, .. }) => {
            eprintln!("REJET: {}", reason);
            std::process::exit(1);
        }
        Ok(_) => {
            eprintln!("réponse inattendue du daemon");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("daemon inaccessible: {}", e);
            std::process::exit(1);
        }
    }
}

fn current_agent_name() -> String {
    if let Ok(path) = std::env::var("BRIDGET_AGENT_NAME_FILE")
        && let Ok(name) = std::fs::read_to_string(path) {
            let name = name.trim();
            if !name.is_empty() {
                return name.to_string();
            }
        }
    std::env::var("BRIDGET_AGENT_NAME").unwrap_or_else(|_| "human".to_string())
}

fn cmd_daemon() {
    let config = DaemonConfig::default();
    match daemon::run(config) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("daemon error: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_send(args: &[String]) {
    let mut to: Option<String> = None;
    let mut from: Option<String> = None;
    let mut reply = false;
    let mut hops: i32 = 4;
    let mut timeout_secs: Option<u64> = None;
    let mut body_parts: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--to" => {
                i += 1;
                if i < args.len() {
                    to = Some(args[i].clone());
                }
            }
            "--from" => {
                i += 1;
                if i < args.len() {
                    from = Some(args[i].clone());
                }
            }
            "--reply" => {
                reply = true;
            }
            "--timeout" => {
                i += 1;
                if i < args.len() {
                    timeout_secs = args[i].parse().ok();
                }
            }
            "--hops" => {
                i += 1;
                if i < args.len() {
                    hops = args[i].parse().unwrap_or(4);
                }
            }
            _ => {
                body_parts.push(args[i].clone());
            }
        }
        i += 1;
    }

    let to = match to {
        Some(t) => t,
        None => {
            eprintln!("usage: bridget send --to <nom> [--reply] [--hops N] <message>");
            std::process::exit(2);
        }
    };

    // Validation du destinataire (H-001)
    if let Err(e) = validate_agent_name(&to) {
        eprintln!("erreur: {}", e);
        std::process::exit(2);
    }

    let body = body_parts.join(" ");
    if body.is_empty() {
        eprintln!("erreur: message vide");
        std::process::exit(2);
    }

    // Validation du corps du message (H-001)
    if let Err(e) = validate_message_body(&body) {
        eprintln!("erreur: {}", e);
        std::process::exit(2);
    }

    let sender = from.unwrap_or_else(current_agent_name);
    let effective_reply = if sender == "human" {
        // L'humain n'est pas un agent connecté — pas de reply possible
        false
    } else {
        reply
    };
    let mut msg = BridgetMessage::new(&sender, &to, &body);
    msg.reply = effective_reply;
    msg.hops = hops;
    if let Some(t) = timeout_secs {
        msg.reply_timeout = Some(t);
    } else if effective_reply {
        msg.reply_timeout = Some(60);
    }

    match send_to_daemon(&msg) {
        Ok(response) => match response {
            DaemonToWrapper::Ack { id } => {
                // Écho du destinataire résolu : l'expéditeur vérifie immédiatement
                // qu'il a visé la bonne cible (anti aiguillage).
                let reply_str = if effective_reply {
                    " [réponse attendue]"
                } else {
                    ""
                };
                println!(
                    "OK: envoyé à « {} » (id={}, hops={}){}",
                    to, id, hops, reply_str
                );
                println!("    ↳ Vérifie : « {} » est bien le destinataire voulu.", to);
            }
            DaemonToWrapper::Nack { id: _, reason } => {
                eprintln!("REJET: {}", reason);
                std::process::exit(1);
            }
            _ => {
                eprintln!("réponse inattendue du daemon");
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("daemon inaccessible: {}", e);
            eprintln!("  (le daemon tourne-t-il ? lancez 'bridget daemon')");
            std::process::exit(1);
        }
    }
}

fn send_to_daemon(msg: &BridgetMessage) -> Result<DaemonToWrapper, String> {
    send_control_to_daemon(WrapperToDaemon::Send(msg.clone()))
}

fn send_control_to_daemon(command: WrapperToDaemon) -> Result<DaemonToWrapper, String> {
    send_control_to_daemon_at(&socket_path(), command)
}

fn send_control_to_daemon_at(
    socket: &std::path::Path,
    command: WrapperToDaemon,
) -> Result<DaemonToWrapper, String> {
    let stream = UnixStream::connect(socket).map_err(|e| e.to_string())?;
    let mut writer = BufWriter::new(stream);

    let reg = WrapperToDaemon::Register {
        agent_type: "cli".to_string(),
        name: Some(format!("cli-send-{}", std::process::id())),
        host: None,
        transport: None,
        os: None,
        instance_id: None,
        domain: None,
        turn_in_progress: false,
    };
    let reg_json = encode(&reg).map_err(|e| e.to_string())?;
    writeln!(writer, "{}", reg_json).map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;

    let read_stream = writer.get_ref().try_clone().map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(read_stream);
    let mut reg_line = String::new();
    reader.read_line(&mut reg_line).map_err(|e| e.to_string())?;
    let _reg_resp: DaemonToWrapper = decode(&reg_line).map_err(|e| e.to_string())?;

    let send_json = encode(&command).map_err(|e| e.to_string())?;
    writeln!(writer, "{}", send_json).map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;

    let mut resp_line = String::new();
    reader
        .read_line(&mut resp_line)
        .map_err(|e| e.to_string())?;
    let resp: DaemonToWrapper = decode(&resp_line).map_err(|e| e.to_string())?;

    Ok(resp)
}

fn cmd_cancel(args: &[String]) {
    if args.is_empty() {
        eprintln!("usage: bridget cancel <id> [--reason <texte>]");
        std::process::exit(2);
    }
    let reason = args
        .windows(2)
        .find(|pair| pair[0] == "--reason")
        .map(|pair| pair[1].clone());
    let id = args[0].clone();
    match send_control_to_daemon(WrapperToDaemon::CancelRequest {
        id: id.clone(),
        sender: current_agent_name(),
        reason,
    }) {
        Ok(DaemonToWrapper::RequestCancelled { state, .. }) => {
            println!("Demande #{} : {}", id, state)
        }
        Ok(DaemonToWrapper::Nack { reason, .. }) => {
            eprintln!("REJET: {}", reason);
            std::process::exit(1);
        }
        Ok(_) => {
            eprintln!("réponse inattendue du daemon");
            std::process::exit(1);
        }
        Err(error) => {
            eprintln!("daemon inaccessible: {}", error);
            std::process::exit(1);
        }
    }
}

fn cmd_requests(args: &[String]) {
    let json_output = args.iter().any(|arg| arg == "--json");
    match send_control_to_daemon(WrapperToDaemon::ListRequests {
        sender: current_agent_name(),
    }) {
        Ok(DaemonToWrapper::RequestList { requests }) if json_output => println!(
            "{}",
            serde_json::to_string(&requests).unwrap_or_else(|_| "[]".to_string())
        ),
        Ok(DaemonToWrapper::RequestList { requests }) => {
            if requests.is_empty() {
                println!("Aucune demande suivie.");
                return;
            }
            let id_width = requests
                .iter()
                .map(|request| request.id.len())
                .max()
                .unwrap_or(2)
                .max(2);
            let target_width = requests
                .iter()
                .map(|request| request.target.len())
                .max()
                .unwrap_or(11)
                .max(11);
            let state_width = requests
                .iter()
                .map(|request| request.state.len())
                .max()
                .unwrap_or(4)
                .max(4);
            println!(
                "{:<id_width$}  {:<target_width$}  {:<state_width$}  ÉCHÉANCE  REPORT",
                "ID", "DESTINATAIRE", "ÉTAT"
            );
            for request in requests {
                println!(
                    "{:<id_width$}  {:<target_width$}  {:<state_width$}  {}  {}",
                    request.id, request.target, request.state, request.deadline_at,
                    request.deferred_reminder_level.map(|level| format!("palier {level} @ {}", request.deferred_reminder_at.unwrap_or_default())).unwrap_or_else(|| "—".to_string())
                );
            }
        }
        Ok(DaemonToWrapper::Nack { reason, .. }) => {
            eprintln!("REJET: {}", reason);
            std::process::exit(1);
        }
        Ok(_) => {
            eprintln!("réponse inattendue du daemon");
            std::process::exit(1);
        }
        Err(error) => {
            eprintln!("daemon inaccessible: {}", error);
            std::process::exit(1);
        }
    }
}

fn send_rename_to_daemon(current_name: &str, name: &str) -> Result<DaemonToWrapper, String> {
    let stream = UnixStream::connect(socket_path()).map_err(|e| e.to_string())?;
    let read_stream = stream.try_clone().map_err(|e| e.to_string())?;
    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(read_stream);
    let register = WrapperToDaemon::Register {
        agent_type: "cli".to_string(),
        name: Some(format!("cli-rename-{}", std::process::id())),
        host: None,
        transport: None,
        os: None,
        instance_id: None,
        domain: None,
        turn_in_progress: false,
    };
    writeln!(writer, "{}", encode(&register).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;
    let mut line = String::new();
    reader.read_line(&mut line).map_err(|e| e.to_string())?;
    let _: DaemonToWrapper = decode(line.trim()).map_err(|e| e.to_string())?;
    let rename = WrapperToDaemon::Rename {
        current_name: current_name.to_string(),
        name: name.to_string(),
    };
    writeln!(writer, "{}", encode(&rename).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;
    line.clear();
    reader.read_line(&mut line).map_err(|e| e.to_string())?;
    decode(line.trim()).map_err(|e| e.to_string())
}

/// Transmet une observation de runtime au daemon depuis le client CLI.
///
/// Le client s'enregistre sous une identité éphémère : c'est le champ `agent`
/// du message, et non cette connexion, qui désigne l'agent observé.
fn send_runtime_to_daemon(
    agent: &str,
    model: &str,
    effort: Option<&str>,
    source: RuntimeSource,
) -> Result<DaemonToWrapper, String> {
    let stream = UnixStream::connect(socket_path()).map_err(|e| e.to_string())?;
    let read_stream = stream.try_clone().map_err(|e| e.to_string())?;
    // Sans délai borné, un daemon qui ne répond pas — par exemple un daemon
    // d'une version antérieure qui ignore ce message — bloquerait le hook, donc
    // la fin de tour de l'agent observé. Constaté en test réel.
    read_stream
        .set_read_timeout(Some(std::time::Duration::from_secs(RUNTIME_REPLY_TIMEOUT_SECS)))
        .map_err(|e| e.to_string())?;
    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(read_stream);
    let register = WrapperToDaemon::Register {
        agent_type: "cli".to_string(),
        name: Some(format!("cli-runtime-{}", std::process::id())),
        host: None,
        transport: None,
        os: None,
        instance_id: None,
        domain: None,
        turn_in_progress: false,
    };
    writeln!(writer, "{}", encode(&register).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;
    let mut line = String::new();
    reader.read_line(&mut line).map_err(|e| e.to_string())?;
    let _: DaemonToWrapper = decode(line.trim()).map_err(|e| e.to_string())?;

    let runtime = WrapperToDaemon::Runtime {
        agent: agent.to_string(),
        model: model.to_string(),
        effort: effort.map(str::to_owned),
        source,
    };
    writeln!(writer, "{}", encode(&runtime).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;
    line.clear();
    reader.read_line(&mut line).map_err(|e| e.to_string())?;
    decode(line.trim()).map_err(|e| e.to_string())
}

fn cmd_runtime(args: &[String]) {
    let mut model: Option<String> = None;
    let mut effort: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--model" if i + 1 < args.len() => {
                model = Some(args[i + 1].clone());
                i += 2;
            }
            "--effort" if i + 1 < args.len() => {
                effort = Some(args[i + 1].clone());
                i += 2;
            }
            other => {
                eprintln!("argument inconnu: {}", other);
                eprintln!("usage: bridget runtime --model <modèle> [--effort <niveau>]");
                std::process::exit(2);
            }
        }
    }

    let Some(model) = model.filter(|value| !value.trim().is_empty()) else {
        eprintln!("usage: bridget runtime --model <modèle> [--effort <niveau>]");
        std::process::exit(2);
    };

    let agent = current_agent_name();
    if agent == "human" {
        eprintln!("runtime indisponible hors d'un agent Bridget");
        std::process::exit(1);
    }

    match send_runtime_to_daemon(&agent, &model, effort.as_deref(), RuntimeSource::Declared) {
        Ok(DaemonToWrapper::Ack { .. }) => match effort {
            Some(effort) => println!("Runtime déclaré : {} (effort: {})", model, effort),
            None => println!("Runtime déclaré : {} (effort: —)", model),
        },
        Ok(DaemonToWrapper::Nack { reason, .. }) => {
            eprintln!("REJET: {}", reason);
            std::process::exit(1);
        }
        Ok(_) => {
            eprintln!("réponse inattendue du daemon");
            std::process::exit(1);
        }
        Err(error) => {
            eprintln!("daemon inaccessible: {}", error);
            std::process::exit(1);
        }
    }
}

/// Commande appelée par un hook d'agent, jamais par un humain.
///
/// Contrat : sortie standard vide, code de retour toujours 0. Un hook qui
/// écrit ou qui échoue perturberait la session de l'agent observé (FR-013).
fn cmd_hook(args: &[String]) {
    match args.first().map(String::as_str) {
        Some("claude-runtime") => hook_claude_runtime(),
        Some(other) => {
            log::debug!("hook inconnu: {}", other);
        }
        None => {
            eprintln!("usage: bridget hook claude-runtime");
            std::process::exit(2);
        }
    }
}

fn hook_claude_runtime() {
    // Hors d'un agent Bridget, le hook est inerte : les sessions Claude
    // ordinaires de l'utilisateur ne doivent subir aucun effet.
    let agent = current_agent_name();
    if agent == "human" {
        return;
    }

    let mut payload = String::new();
    if std::io::Read::read_to_string(&mut std::io::stdin(), &mut payload).is_err() {
        log::debug!("hook claude-runtime : payload illisible");
        return;
    }
    let Ok(payload) = serde_json::from_str::<serde_json::Value>(&payload) else {
        log::debug!("hook claude-runtime : payload non JSON");
        return;
    };
    let Some(transcript) = payload.get("transcript_path").and_then(|v| v.as_str()) else {
        log::debug!("hook claude-runtime : pas de transcript_path");
        return;
    };
    // Journalisé pour rendre diagnosticable le cas d'une session Claude
    // imbriquée qui hériterait du nom de l'agent parent (research.md D-002).
    log::debug!(
        "hook claude-runtime : agent={} session={:?}",
        agent,
        payload.get("session_id").and_then(|v| v.as_str())
    );

    let Some(observed) = crate::runtime::parse_claude_transcript(std::path::Path::new(transcript))
    else {
        log::debug!("hook claude-runtime : aucun modèle dans {}", transcript);
        return;
    };

    match send_runtime_to_daemon(
        &agent,
        &observed.model,
        observed.effort.as_deref(),
        RuntimeSource::ClaudeHook,
    ) {
        Ok(DaemonToWrapper::Ack { .. }) => {}
        Ok(other) => log::debug!("hook claude-runtime : réponse inattendue {:?}", other),
        Err(error) => log::debug!("hook claude-runtime : daemon inaccessible: {}", error),
    }
}

fn claude_settings_path() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()))
        .join(".claude")
        .join("settings.json")
}

/// Commande du hook telle qu'inscrite dans la configuration de Claude Code.
const HOOK_COMMAND: &str = "bridget hook claude-runtime";

fn cmd_install_hooks(args: &[String]) {
    let remove = args.iter().any(|arg| arg == "--remove");
    let path = claude_settings_path();

    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) => {
            eprintln!("{} illisible: {}", path.display(), error);
            std::process::exit(1);
        }
    };
    let mut settings: serde_json::Value = match serde_json::from_str(&content) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("{} n'est pas un JSON valide: {}", path.display(), error);
            std::process::exit(1);
        }
    };

    let changed = if remove {
        remove_bridget_hook(&mut settings)
    } else {
        insert_bridget_hook(&mut settings)
    };

    if !changed {
        println!(
            "Aucun changement : le hook Bridget est déjà {}.",
            if remove { "absent" } else { "installé" }
        );
        return;
    }

    // Sauvegarde AVANT écriture : l'utilisateur doit pouvoir revenir en arrière
    // sur un fichier qui ne nous appartient pas (FR-012).
    let backup = path.with_extension(format!("json.bak-{}", timestamp()));
    if let Err(error) = std::fs::copy(&path, &backup) {
        eprintln!("sauvegarde impossible ({}) : rien n'a été modifié", error);
        std::process::exit(1);
    }

    let serialized = match serde_json::to_string_pretty(&settings) {
        Ok(text) => format!("{}\n", text),
        Err(error) => {
            eprintln!("sérialisation impossible: {}", error);
            std::process::exit(1);
        }
    };
    if let Err(error) = write_atomically(&path, &serialized) {
        eprintln!("écriture impossible: {}", error);
        std::process::exit(1);
    }

    println!("Sauvegarde : {}", backup.display());
    if remove {
        println!("Hook Bridget retiré de {}", path.display());
    } else {
        println!("Hook Bridget installé dans {}", path.display());
        println!("Les sessions Claude déjà ouvertes ne sont pas affectées.");
    }
}

/// Écrit un fichier de configuration sans jamais le laisser tronqué.
///
/// Un `write` direct expose à un fichier à moitié écrit si le processus meurt
/// ou si le disque est plein. Le fichier temporaire vit dans le même
/// répertoire pour que le `rename` soit atomique — il le serait pas entre
/// systèmes de fichiers différents.
fn write_atomically(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    let directory = path.parent().unwrap_or(std::path::Path::new("."));
    let temporary = directory.join(format!(
        ".{}.bridget-{}",
        path.file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "settings.json".to_string()),
        std::process::id()
    ));
    let mut file = std::fs::File::create(&temporary)?;
    file.write_all(content.as_bytes())?;
    file.sync_all()?;
    drop(file);
    // Conserver les permissions d'origine : le fichier de configuration de
    // l'utilisateur ne doit pas devenir plus permissif à cause de nous.
    if let Ok(metadata) = std::fs::metadata(path) {
        let _ = std::fs::set_permissions(&temporary, metadata.permissions());
    }
    match std::fs::rename(&temporary, path) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = std::fs::remove_file(&temporary);
            Err(error)
        }
    }
}

/// Ajoute l'entrée Bridget au tableau `hooks.Stop` sans toucher aux entrées
/// existantes de l'utilisateur. Rend `false` si elle y était déjà, ou si la
/// structure du fichier n'est pas celle attendue — auquel cas on préfère ne
/// rien faire plutôt que d'écraser une configuration qu'on ne comprend pas.
fn insert_bridget_hook(settings: &mut serde_json::Value) -> bool {
    if hook_is_present(settings) {
        return false;
    }
    let entry = serde_json::json!({
        "hooks": [{
            "type": "command",
            "command": HOOK_COMMAND,
            "timeout": 5
        }]
    });
    let Some(root) = settings.as_object_mut() else {
        eprintln!("le fichier de configuration n'est pas un objet JSON");
        return false;
    };
    let hooks = root
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));
    let Some(hooks) = hooks.as_object_mut() else {
        eprintln!("la section « hooks » n'est pas un objet JSON");
        return false;
    };
    let stop = hooks.entry("Stop").or_insert_with(|| serde_json::json!([]));
    match stop.as_array_mut() {
        Some(array) => {
            array.push(entry);
            true
        }
        None => {
            eprintln!("la section « hooks.Stop » n'est pas une liste JSON");
            false
        }
    }
}

/// Retire la seule entrée dont la commande est celle de Bridget.
fn remove_bridget_hook(settings: &mut serde_json::Value) -> bool {
    let Some(stop) = settings
        .get_mut("hooks")
        .and_then(|hooks| hooks.get_mut("Stop"))
        .and_then(|stop| stop.as_array_mut())
    else {
        return false;
    };
    let before = stop.len();
    stop.retain(|entry| !entry_is_bridget(entry));
    before != stop.len()
}

fn hook_is_present(settings: &serde_json::Value) -> bool {
    settings
        .get("hooks")
        .and_then(|hooks| hooks.get("Stop"))
        .and_then(|stop| stop.as_array())
        .map(|entries| entries.iter().any(entry_is_bridget))
        .unwrap_or(false)
}

fn entry_is_bridget(entry: &serde_json::Value) -> bool {
    entry
        .get("hooks")
        .and_then(|hooks| hooks.as_array())
        .map(|hooks| {
            hooks.iter().any(|hook| {
                hook.get("command").and_then(|c| c.as_str()) == Some(HOOK_COMMAND)
            })
        })
        .unwrap_or(false)
}

/// Horodatage `AAAAMMJJ-HHMMSS` en temps local, pour nommer une sauvegarde.
fn timestamp() -> String {
    let output = std::process::Command::new("date")
        .arg("+%Y%m%d-%H%M%S")
        .output();
    match output {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string(),
        _ => std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "inconnu".to_string()),
    }
}

/// Chemin du domaine surchargé d'un agent, en miroir de `agent-names/`.
fn domain_state_path(agent: &str) -> std::path::PathBuf {
    socket_path()
        .parent()
        .unwrap()
        .join("agent-domains")
        .join(agent)
}

fn cmd_domain(args: &[String]) {
    let reset = args.iter().any(|arg| arg == "--reset");
    let requested = args.iter().find(|arg| !arg.starts_with("--")).cloned();

    if !reset && requested.is_none() {
        eprintln!("usage: bridget domain <nom> | bridget domain --reset");
        std::process::exit(2);
    }
    if let Some(name) = &requested
        && let Err(reason) = validate_agent_name(name) {
            eprintln!("erreur: {}", reason);
            std::process::exit(2);
        }

    let agent = current_agent_name();
    if agent == "human" {
        eprintln!("domain indisponible hors d'un agent Bridget");
        std::process::exit(1);
    }

    let domain = if reset { None } else { requested };
    let message = WrapperToDaemon::Domain {
        agent: agent.clone(),
        domain: domain.clone(),
    };
    match send_control_to_daemon(message) {
        Ok(DaemonToWrapper::Ack { .. }) => {
            // La trace disque porte l'intention : elle survit au redémarrage du
            // daemon et est relue par le wrapper à chaque reconnexion.
            let path = domain_state_path(&agent);
            match &domain {
                Some(domain) => {
                    if let Some(parent) = path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    let _ = std::fs::write(&path, domain);
                    println!("Domaine de « {} » : {}", agent, domain);
                }
                None => {
                    let _ = std::fs::remove_file(&path);
                    println!("Domaine de « {} » réinitialisé sur le dépôt courant.", agent);
                }
            }
        }
        Ok(DaemonToWrapper::Nack { reason, .. }) => {
            eprintln!("REJET: {}", reason);
            std::process::exit(1);
        }
        Ok(_) => {
            eprintln!("réponse inattendue du daemon");
            std::process::exit(1);
        }
        Err(error) => {
            eprintln!("daemon inaccessible: {}", error);
            std::process::exit(1);
        }
    }
}

/// Durée de sécurité appliquée à un « ne pas déranger » sans échéance précisée.
const DND_DEFAULT_MINUTES: u64 = 60;

/// Interprète une durée de la forme `90s`, `30m` ou `2h`.
fn parse_duration(value: &str) -> Result<Duration, String> {
    let value = value.trim();
    let (digits, multiplier) = match value.chars().last() {
        Some('s') => (&value[..value.len() - 1], 1),
        Some('m') => (&value[..value.len() - 1], 60),
        Some('h') => (&value[..value.len() - 1], 3600),
        Some(last) if last.is_ascii_digit() => (value, 60), // sans unité : minutes
        _ => return Err("durée attendue sous la forme 90s, 30m ou 2h".to_string()),
    };
    let amount: u64 = digits
        .parse()
        .map_err(|_| "durée attendue sous la forme 90s, 30m ou 2h".to_string())?;
    if amount == 0 {
        return Err("durée nulle".to_string());
    }
    Ok(Duration::from_secs(amount * multiplier))
}

fn cmd_dnd(args: &[String]) {
    let lift = args.iter().any(|arg| arg == "off");
    let duration = match args.iter().position(|arg| arg == "--duration") {
        Some(index) => match args.get(index + 1) {
            Some(value) => match parse_duration(value) {
                Ok(duration) => Some(duration),
                Err(reason) => {
                    eprintln!("erreur: {}", reason);
                    std::process::exit(2);
                }
            },
            None => {
                eprintln!("usage: bridget dnd [off] [--duration 30m]");
                std::process::exit(2);
            }
        },
        None => None,
    };

    let agent = current_agent_name();
    if agent == "human" {
        eprintln!("dnd indisponible hors d'un agent Bridget");
        std::process::exit(1);
    }

    let until_secs = if lift {
        None
    } else {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|since| since.as_secs())
            .unwrap_or(0);
        let window = duration.unwrap_or(Duration::from_secs(DND_DEFAULT_MINUTES * 60));
        Some(now + window.as_secs())
    };

    let message = WrapperToDaemon::Availability {
        agent: agent.clone(),
        until_secs,
    };
    match send_control_to_daemon(message) {
        Ok(DaemonToWrapper::Ack { .. }) => match until_secs {
            Some(_) => {
                let minutes = duration
                    .map(|d| d.as_secs().div_ceil(60))
                    .unwrap_or(DND_DEFAULT_MINUTES);
                println!(
                    "« {} » ne sera pas dérangé pendant {} min. Levée : bridget dnd off",
                    agent, minutes
                );
            }
            None => println!("« {} » est à nouveau joignable.", agent),
        },
        Ok(DaemonToWrapper::Nack { reason, .. }) => {
            eprintln!("REJET: {}", reason);
            std::process::exit(1);
        }
        Ok(_) => {
            eprintln!("réponse inattendue du daemon");
            std::process::exit(1);
        }
        Err(error) => {
            eprintln!("daemon inaccessible: {}", error);
            std::process::exit(1);
        }
    }
}

fn cmd_reply(args: &[String]) {
    let agent_name = current_agent_name();

    let reply_file = socket_path()
        .parent()
        .unwrap()
        .join(format!("last-sender-{}", agent_name));

    let previous = match std::fs::read_to_string(&reply_file) {
        Ok(content) => content.trim().to_string(),
        Err(_) => {
            eprintln!("reply: aucun expediteur precedent trouve.");
            eprintln!("  (utilise 'bridget send --to <nom> \"message\"')");
            std::process::exit(1);
        }
    };

    let mut previous_parts = previous.splitn(2, '\t');
    let to = previous_parts.next().unwrap_or_default().to_string();
    let in_reply_to = previous_parts.next().map(str::to_string);
    if to.is_empty() {
        eprintln!("reply: expediteur precedent vide.");
        std::process::exit(1);
    }

    let mut reply_flag = false;
    let mut hops: i32 = 4;
    let mut timeout_secs: Option<u64> = None;
    let mut body_parts: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--reply" => {
                reply_flag = true;
            }
            "--timeout" => {
                if i + 1 < args.len() {
                    timeout_secs = args[i + 1].parse().ok();
                    i += 1;
                }
            }
            "--hops" => {
                if i + 1 < args.len() {
                    hops = args[i + 1].parse().unwrap_or(4);
                    i += 1;
                }
            }
            _ => {
                body_parts.push(args[i].clone());
            }
        }
        i += 1;
    }

    let body = body_parts.join(" ");
    if body.is_empty() {
        eprintln!("usage: bridget reply [--reply] [--hops N] <message>");
        std::process::exit(2);
    }

    // Validation du corps du message (H-001)
    if let Err(e) = validate_message_body(&body) {
        eprintln!("erreur: {}", e);
        std::process::exit(2);
    }

    let sender = agent_name.clone();
    let effective_reply = if sender == "human" { false } else { reply_flag };

    let mut msg = BridgetMessage::new(&sender, &to, &body);
    msg.in_reply_to = in_reply_to;
    msg.reply = effective_reply;
    msg.hops = hops;
    if let Some(t) = timeout_secs {
        msg.reply_timeout = Some(t);
    } else if effective_reply {
        msg.reply_timeout = Some(60);
    }

    match send_to_daemon(&msg) {
        Ok(response) => match response {
            DaemonToWrapper::Ack { id } => {
                println!("OK: reply a {} (id={}, hops={})", to, id, hops);
            }
            DaemonToWrapper::Nack { id: _, reason } => {
                eprintln!("REJET: {}", reason);
                std::process::exit(1);
            }
            _ => {
                eprintln!("reponse inattendue du daemon");
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("daemon inaccessible: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_agents(args: &[String]) {
    let json_output = args.iter().any(|a| a == "--json");
    let filter = extract_domain_filter(args);

    let config = DaemonConfig::default();
    let mut status = daemon::get_status(&config);
    if let Some(domain) = &filter {
        status
            .agents
            .retain(|agent| agent.domain.as_deref() == Some(domain.as_str()));
    }
    if !status.running {
        if json_output {
            println!("[]");
        } else {
            eprintln!("daemon non demarre (socket absente)");
        }
        std::process::exit(1);
    }

    if json_output {
        println!(
            "{}",
            serde_json::to_string(&status.agents).unwrap_or_else(|_| "[]".to_string())
        );
    } else if status.agents.is_empty() {
        println!("Aucun agent connecte.");
    } else {
        println!("Agents connectes :");
        for agent in &status.agents {
            println!(
                "  {} ({}) [{}] — {} / {} via {} — {} / {} [{}]",
                agent.name,
                agent.agent_type,
                cell(agent.domain.as_deref()),
                agent.host,
                agent.os,
                agent.transport,
                cell(agent.model.as_deref()),
                cell(agent.effort.as_deref()),
                agent.state
            );
        }
    }
}

/// Extrait la valeur de `--domain <nom>` des arguments d'une commande d'annuaire.
fn extract_domain_filter(args: &[String]) -> Option<String> {
    args.iter()
        .position(|arg| arg == "--domain")
        .and_then(|index| args.get(index + 1))
        .cloned()
}

fn cmd_who(args: &[String]) {
    let config = DaemonConfig::default();
    let status = daemon::get_status(&config);
    if !status.running {
        eprintln!("daemon non démarré (socket absente)");
        std::process::exit(1);
    }

    let filter = extract_domain_filter(args);
    let agents: Vec<_> = match &filter {
        Some(domain) => status
            .agents
            .into_iter()
            .filter(|agent| agent.domain.as_deref() == Some(domain.as_str()))
            .collect(),
        None => status.agents,
    };

    if agents.is_empty() {
        match &filter {
            Some(domain) => println!("Aucun agent dans le domaine « {} ».", domain),
            None => println!("Aucun agent connecté."),
        }
        return;
    }

    // Chaque colonne s'aligne sur sa valeur la plus longue, en-tête comprise.
    let column = |header: &str, values: &dyn Fn(&AgentInfo) -> String| {
        agents
            .iter()
            .map(|agent| values(agent).chars().count())
            .max()
            .unwrap_or(0)
            .max(header.chars().count())
    };
    let name_w = column("NOM", &|a: &AgentInfo| a.name.clone());
    let type_w = column("TYPE", &|a: &AgentInfo| a.agent_type.clone());
    let host_w = column("HÔTE", &|a: &AgentInfo| a.host.clone());
    let os_w = column("OS", &|a: &AgentInfo| a.os.clone());
    let transport_w = column("TRANSPORT", &|a: &AgentInfo| a.transport.clone());
    let domain_w = column("DOMAINE", &|a: &AgentInfo| cell(a.domain.as_deref()).to_string());
    let model_w = column("MODÈLE", &|a: &AgentInfo| cell(a.model.as_deref()).to_string());
    let effort_w = column("EFFORT", &|a: &AgentInfo| cell(a.effort.as_deref()).to_string());

    match &filter {
        Some(domain) => println!("Agents du domaine « {} » :", domain),
        None => println!("Agents connectés :"),
    }
    println!(
        "  {:<name_w$}  {:<type_w$}  {:<host_w$}  {:<os_w$}  {:<transport_w$}  {:<domain_w$}  {:<model_w$}  {:<effort_w$}  ÉTAT",
        "NOM", "TYPE", "HÔTE", "OS", "TRANSPORT", "DOMAINE", "MODÈLE", "EFFORT"
    );
    for agent in &agents {
        println!(
            "  {:<name_w$}  {:<type_w$}  {:<host_w$}  {:<os_w$}  {:<transport_w$}  {:<domain_w$}  {:<model_w$}  {:<effort_w$}  {}",
            agent.name,
            agent.agent_type,
            agent.host,
            agent.os,
            agent.transport,
            cell(agent.domain.as_deref()),
            cell(agent.model.as_deref()),
            cell(agent.effort.as_deref()),
            agent.state
        );
    }
}

/// Rend une valeur d'annuaire affichable : un tiret cadratin marque une valeur
/// inconnue, ce qui la distingue d'une valeur vide qui casserait la lecture des
/// colonnes.
fn cell(value: Option<&str>) -> &str {
    value.unwrap_or("—")
}

fn cmd_discover() {
    cmd_who(&[]);
}

fn cmd_status() {
    let config = DaemonConfig::default();
    let status = daemon::get_status(&config);
    println!(
        "Daemon: {}",
        if status.running {
            "en ligne"
        } else {
            "hors ligne"
        }
    );
    println!("Socket: {}", config.socket_path.display());
    println!("Base de données: {}", config.db_path.display());
    println!("Agents connectés: {}", status.agents.len());
    println!("Messages en base: {}", status.message_count);
}

fn cmd_ledger() {
    let messages = match send_control_to_daemon(WrapperToDaemon::LedgerProjection {
        scope: LedgerScope::Messages,
        limit: 20,
    }) {
        Ok(DaemonToWrapper::LedgerProjection { messages, .. }) => Ok(messages),
        Ok(DaemonToWrapper::Nack { reason, .. }) => Err(format!("erreur lecture ledger: {reason}")),
        Ok(_) => Err("réponse inattendue du daemon".to_string()),
        Err(_) => {
            let config = DaemonConfig::default();
            crate::store::Store::open(&config.db_path)
                .map_err(|error| format!("base inaccessible: {error}"))
                .and_then(|store| {
                    crate::ledger::read_projection(&store, LedgerScope::Messages, 20)
                        .map(|projection| projection.messages)
                        .map_err(|error| format!("erreur lecture ledger: {error}"))
                })
        }
    };
    match messages {
        Ok(messages) => print_ledger(&messages),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

fn print_ledger(entries: &[LedgerMessage]) {
    print!("{}", render_ledger(entries));
}

fn render_ledger(entries: &[LedgerMessage]) -> String {
    if entries.is_empty() {
        return "Ledger vide.\n".to_string();
    }
    let mut rendered = format!("Derniers {} messages :\n", entries.len());
    for entry in entries.iter().rev() {
        rendered.push_str(&format!(
            "  [{}] {} → {}: {}\n",
            entry.ts,
            entry.sender,
            entry.target,
            entry.body.chars().take(60).collect::<String>()
        ));
    }
    rendered
}

#[cfg(test)]
mod hook_tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::UnixListener;
    use std::path::{Path, PathBuf};
    use std::thread;

    /// Configuration réaliste : quatre hooks utilisateur déjà en place, dont
    /// un sur `Stop`. L'insertion doit être additive, jamais destructive.
    fn settings_utilisateur() -> serde_json::Value {
        serde_json::json!({
            "model": "opus",
            "hooks": {
                "UserPromptSubmit": [{"hooks": [{"type": "command", "command": "attention.sh working"}]}],
                "Stop": [{"hooks": [{"type": "command", "command": "attention.sh mark"}]}],
                "PostToolUse": [{"matcher": "Edit|Write", "hooks": [{"type": "command", "command": "auto-commit.sh"}]}],
                "SessionEnd": [{"hooks": [{"type": "command", "command": "session-sync.sh"}]}]
            }
        })
    }

    #[test]
    fn installation_additive_puis_retrait_restaure_l_original() {
        let original = settings_utilisateur();
        let mut settings = original.clone();

        assert!(insert_bridget_hook(&mut settings));
        let stop = settings["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop.len(), 2, "le hook utilisateur doit être préservé");
        assert_eq!(
            stop[0]["hooks"][0]["command"].as_str(),
            Some("attention.sh mark")
        );
        // Les autres événements sont intacts.
        assert_eq!(settings["hooks"]["SessionEnd"], original["hooks"]["SessionEnd"]);
        assert_eq!(settings["model"], original["model"]);

        assert!(remove_bridget_hook(&mut settings));
        assert_eq!(settings, original, "le retrait doit rendre le fichier d'origine");
    }

    #[test]
    fn installation_est_idempotente() {
        let mut settings = settings_utilisateur();
        assert!(insert_bridget_hook(&mut settings));
        assert!(
            !insert_bridget_hook(&mut settings),
            "une seconde installation ne doit rien changer"
        );
        assert_eq!(settings["hooks"]["Stop"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn retrait_sans_installation_prealable_ne_change_rien() {
        let mut settings = settings_utilisateur();
        assert!(!remove_bridget_hook(&mut settings));
        assert_eq!(settings, settings_utilisateur());
    }

    #[test]
    fn installation_cree_la_section_hooks_absente() {
        let mut settings = serde_json::json!({"model": "opus"});
        assert!(insert_bridget_hook(&mut settings));
        assert!(hook_is_present(&settings));
        assert_eq!(settings["hooks"]["Stop"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn structure_inattendue_ne_declenche_aucune_modification() {
        // Ni panique, ni écrasement : on refuse de toucher un fichier dont la
        // structure n'est pas celle attendue.
        let mut racine_non_objet = serde_json::json!(["pas", "un", "objet"]);
        assert!(!insert_bridget_hook(&mut racine_non_objet));

        let mut hooks_non_objet = serde_json::json!({"hooks": "une chaîne"});
        assert!(!insert_bridget_hook(&mut hooks_non_objet));
        assert_eq!(hooks_non_objet["hooks"], serde_json::json!("une chaîne"));

        let mut stop_non_liste = serde_json::json!({"hooks": {"Stop": 42}});
        assert!(!insert_bridget_hook(&mut stop_non_liste));
        assert_eq!(stop_non_liste["hooks"]["Stop"], serde_json::json!(42));
    }

    #[test]
    fn ecriture_atomique_preserve_le_contenu() {
        let path = std::env::temp_dir().join(format!("bridget-atomic-{}.json", std::process::id()));
        std::fs::write(&path, "{\"origine\":true}\n").unwrap();
        write_atomically(&path, "{\"nouveau\":true}\n").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{\"nouveau\":true}\n");
        // Aucun fichier temporaire ne subsiste dans le répertoire.
        let restes = std::fs::read_dir(std::env::temp_dir())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .contains(".bridget-")
            })
            .count();
        assert_eq!(restes, 0);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn cellule_runtime_marque_une_valeur_absente() {
        assert_eq!(cell(Some("claude-opus-5")), "claude-opus-5");
        assert_eq!(cell(None), "—");
    }

    #[test]
    fn arguments_attach_resolvent_les_trois_fenetres() {
        assert_eq!(
            parse_attach_args(&["codex-1".to_string()]).unwrap(),
            ("codex-1".to_string(), AttachWindow::Today)
        );
        assert_eq!(
            parse_attach_args(&[
                "codex-1".to_string(),
                "--from-seq".to_string(),
                "42".to_string(),
            ])
            .unwrap(),
            ("codex-1".to_string(), AttachWindow::Seq(42))
        );
        assert_eq!(
            parse_attach_args(&[
                "codex-1".to_string(),
                "--date".to_string(),
                "2026-08-22".to_string(),
            ])
            .unwrap(),
            (
                "codex-1".to_string(),
                AttachWindow::Date("2026-08-22".to_string())
            )
        );
    }

    #[test]
    fn arguments_attach_refusent_les_formes_ambigues() {
        assert!(parse_attach_args(&[]).is_err());
        assert!(
            parse_attach_args(&["nom invalide".to_string()]).is_err(),
            "la validation des noms existante reste appliquée"
        );
        assert!(
            parse_attach_args(&[
                "codex-1".to_string(),
                "--from-seq".to_string(),
                "pas-un-entier".to_string(),
            ])
            .is_err()
        );
        assert!(
            parse_attach_args(&[
                "codex-1".to_string(),
                "--date".to_string(),
                "2026-08-22".to_string(),
                "--from-seq".to_string(),
                "1".to_string(),
            ])
            .is_err()
        );
    }

    #[test]
    fn spawn_cli_rejoue_l_enveloppe_memorisee_octet_pour_octet() {
        let root = std::env::temp_dir().join(format!(
            "bridget-cli-t905-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let cwd = root.join("work");
        std::fs::create_dir_all(&cwd).unwrap();
        let args = vec![
            "codex".to_string(),
            "--name".to_string(),
            "codex-managed".to_string(),
            "--persistent".to_string(),
            "--command-id".to_string(),
            "command-retry".to_string(),
        ];
        let first = resolve_spawn_order(&parse_spawn_args(&args).unwrap(), 100, &cwd, &root)
            .unwrap();
        let retry_args = vec![
            "codex".to_string(),
            "--command-id".to_string(),
            "command-retry".to_string(),
        ];
        let retry = resolve_spawn_order(
            &parse_spawn_args(&retry_args).unwrap(),
            999,
            Path::new("/autre/cwd"),
            &root,
        )
        .unwrap();
        assert_eq!(encode(&first).unwrap(), encode(&retry).unwrap());
        let state_path = spawn_order_path(&root, "command-retry");
        assert_eq!(
            std::fs::metadata(&state_path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert!(resolve_spawn_order(
            &parse_spawn_args(&[
                "claude".to_string(),
                "--command-id".to_string(),
                "command-retry".to_string(),
            ])
            .unwrap(),
            999,
            &cwd,
            &root,
        )
        .unwrap_err()
        .contains("options divergentes"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn spawn_roundtrip_rejoue_la_meme_issue_apres_reponse_perdue() {
        let socket = PathBuf::from(format!(
            "/tmp/bg-t905-{}-{}.sock",
            std::process::id(),
            uuid::Uuid::new_v4().simple()
        ));
        let listener = UnixListener::bind(&socket).unwrap();
        let server = thread::spawn(move || {
            let mut observed = Vec::new();
            for _ in 0..2 {
                let (stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut writer = BufWriter::new(stream);
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                assert!(matches!(
                    decode::<WrapperToDaemon>(line.trim()).unwrap(),
                    WrapperToDaemon::Register { .. }
                ));
                writeln!(
                    writer,
                    "{}",
                    encode(&DaemonToWrapper::Registered {
                        name: "cli-test".to_string()
                    })
                    .unwrap()
                )
                .unwrap();
                writer.flush().unwrap();
                line.clear();
                reader.read_line(&mut line).unwrap();
                observed.push(line.trim().to_string());
                writeln!(
                    writer,
                    "{}",
                    encode(&DaemonToWrapper::SpawnAccepted {
                        command_id: "command-lost".to_string(),
                        name: "codex-managed".to_string(),
                    })
                    .unwrap()
                )
                .unwrap();
                writer.flush().unwrap();
            }
            observed
        });
        let order = WrapperToDaemon::SpawnOrder {
            agent_type: "codex".to_string(),
            name: Some("codex-managed".to_string()),
            cwd: "/tmp".to_string(),
            persistent: false,
            command_id: "command-lost".to_string(),
            issued_at: 100,
            deadline_at: 110,
        };
        for _ in 0..2 {
            assert!(matches!(
                send_control_to_daemon_at(&socket, order.clone()).unwrap(),
                DaemonToWrapper::SpawnAccepted { ref command_id, ref name }
                    if command_id == "command-lost" && name == "codex-managed"
            ));
        }
        let observed = server.join().unwrap();
        assert_eq!(observed[0], observed[1], "le retry doit être canonique");
        let _ = std::fs::remove_file(socket);
    }

    #[test]
    fn stop_cli_genere_ou_reutilise_un_command_id() {
        let generated = parse_stop_args(&["codex-1".to_string()]).unwrap();
        assert_eq!(generated.0, "codex-1");
        assert!(!generated.1.is_empty());
        assert_eq!(
            parse_stop_args(&[
                "codex-1".to_string(),
                "--command-id".to_string(),
                "stop-retry".to_string(),
            ])
            .unwrap(),
            ("codex-1".to_string(), "stop-retry".to_string())
        );
    }
}
