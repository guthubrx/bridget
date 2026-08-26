//! CLI — point d'entrée unifié pour toutes les sous-commandes bridget.

use crate::daemon::{self, DaemonConfig};
use bridget_core::BridgetMessage;
use bridget_transport::protocol::{
    AgentInfo, AttachWindow, CLIENT_CONTRACT_VERSION, ClientCapability, ConnectionRole,
    IdempotencyIssue, LedgerMessage, LedgerScope, PresenceMode, RequestInfo, ReviewTarget,
    ReviewVerdict, ReviewVerdictEvidence, RuntimeSource, SERVICE_CONTRACT_VERSION,
    ServiceRequestOperation, ServiceRequestPayload, decode, encode, is_canonical_git_sha,
};
use bridget_transport::{DaemonToWrapper, SpawnRefusal, StopOutcome, WrapperToDaemon};
use std::fmt::Write as _;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct IdempotentSendOptions {
    id: String,
    issued_at: i64,
    issuer_scope: String,
}

/// Délai maximal d'attente d'une réponse du daemon pour une observation de
/// runtime. Court volontairement : l'appelant est un hook exécuté dans la
/// boucle de l'agent, il ne doit jamais le faire patienter.
const RUNTIME_REPLY_TIMEOUT_SECS: u64 = 2;

/// Valide un nom d'agent Bridget (H-001)
fn validate_agent_name(name: &str) -> Result<(), String> {
    if name.len() > MAX_AGENT_NAME_LENGTH {
        return Err(format!(
            "nom d'agent trop long (max {} caractères)",
            MAX_AGENT_NAME_LENGTH
        ));
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(
            "nom d'agent contient des caractères invalides (alphanumériques, -, _ uniquement)"
                .to_string(),
        );
    }
    Ok(())
}

/// Valide le corps d'un message (H-001)
fn validate_message_body(body: &str) -> Result<(), String> {
    if body.len() > MAX_MESSAGE_LENGTH {
        return Err(format!(
            "message trop long (max {} caractères)",
            MAX_MESSAGE_LENGTH
        ));
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
        "ui" => cmd_ui(&args[2..]),
        "attach" => cmd_attach(&args[2..]),
        "spawn" => cmd_spawn(&args[2..]),
        "stop" => cmd_stop(&args[2..]),
        "send" => cmd_send(&args[2..]),
        "guichet" => cmd_guichet(&args[2..]),
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
        "reprise" => cmd_reprise(&args[2..]),
        "reaper" => cmd_reaper(&args[2..]),
        "cleanup" => cmd_cleanup(&args[2..]),
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

fn cmd_ui(args: &[String]) {
    let mut maicie_config = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--maicie-config" => {
                index += 1;
                maicie_config = args.get(index).map(PathBuf::from);
                if maicie_config.is_none() {
                    eprintln!("bridget ui: --maicie-config requiert un chemin absolu");
                    std::process::exit(2);
                }
            }
            option => {
                eprintln!("bridget ui: option inconnue {option}");
                std::process::exit(2);
            }
        }
        index += 1;
    }
    let maicie_config = maicie_config.unwrap_or_else(|| {
        eprintln!("bridget ui: --maicie-config <chemin-absolu> est obligatoire");
        std::process::exit(2);
    });
    if !maicie_config.is_absolute() {
        eprintln!("bridget ui: le chemin --maicie-config doit être absolu");
        std::process::exit(2);
    }
    if let Err(error) = crate::ui::run(socket_path(), maicie_config) {
        eprintln!("bridget ui: {error}");
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
    if args.len() != 3 {
        eprintln!("bridget managed-wrapper: type, nom et définition figée requis");
        std::process::exit(2);
    }
    let definition = match serde_json::from_str(&args[2]) {
        Ok(definition) => definition,
        Err(error) => {
            eprintln!("bridget managed-wrapper: définition figée invalide: {error}");
            std::process::exit(2);
        }
    };
    if let Err(error) = crate::wrapper::launch_managed_acp(&args[0], &args[1], &definition) {
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
           ui --maicie-config <P> Lance le relais UI (port+jeton stables)\n  \
           attach <N>             Suit un équipier [--from-seq N | --date AAAA-MM-JJ]\n  \
           spawn <TYPE>           Lance un équipier géré [--name N] [--persistent]\n  \
           stop <N>               Arrête un équipier géré\n  \
           send --to <N> <MSG>    Envoie un message\n  \
           reply <MSG>            Répond au dernier expéditeur\n  \
           cancel <ID>            Annule une demande suivie [--reason <T>]\n  \
           requests [--all]       Mes demandes (défaut) ou toutes les ouvertes\n  \
           rename <N>             Renomme l'agent courant\n  \
           runtime --model <M>    Déclare le modèle courant [--effort <E>]\n  \
           domain <N> | --reset   Change le domaine de l'agent courant\n  \
           dnd [off]              Ne pas déranger [--duration 30m]\n  \
           install-hooks          Installe la détection auto du modèle (Claude)\n  \
           who [--domain <D>]     Agents connectés\n  \
           agents [--json]        Idem, format machine [--domain <D>]\n  \
           status                 Santé du daemon\n  \
           ledger                 Historique des messages\n  \
           reprise [--write P]    Carte de reprise du référent\n  \
           reaper report          Observateur J2 (ne tue jamais)\n  \
           cleanup --dry-run      Liste target/ des worktrees mergés\n  \
           version                Version\n  \
           help                   Cette aide\n\n\
         Options de send :\n  \
           --to <nom>             Destinataire (requis)\n  \
           --in-reply-to <id>     Lie la réponse à une demande suivie\n  \
           --reply                Réponse attendue\n  \
           --timeout <S>          Délai avant échec (défaut: 60)\n  \
           --hops <N>             Sauts restants (défaut: 4)\n\n\
           --id <clé>             Clé de rejeu (avec --issued-at)\n  \
           --issued-at <unix>     Instant d'émission du rejeu\n  \
           --issuer-scope <portée> Portée requise pour un envoi ordinaire idempotent\n\n\
         Usage interne :\n  \
           hook claude-runtime    Appelé par le hook Claude Code, lit stdin\n  \
           hook claude-statusline Limites de forfait, lit le payload StatusLine\n    \
                                  sur stdin. N'affiche RIEN : à appeler en plus\n    \
                                  de votre statusLine, pas à sa place —\n    \
                                  printf '%s' \"$input\" | bridget hook claude-statusline &"
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
    if let Some(name) = parsed.name.as_deref()
        && !parsed.persistent
    {
        eprintln!(
            "avertissement: {}",
            crate::recovery_trace::non_persistent_spawn_warning(name)
        );
    }
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
                            .ok_or_else(|| {
                                "--timeout doit être compris entre 1 et 600".to_string()
                            })?;
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
    let name = args
        .first()
        .cloned()
        .ok_or_else(|| "nom manquant".to_string())?;
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
    bridget_transport::fsutil::write_private_file_atomic(&path, &bytes).map_err(|error| {
        format!(
            "mémorisation de l'ordre impossible {}: {error}",
            path.display()
        )
    })?;
    Ok(order)
}

fn validate_retry_options(
    parsed: &ParsedSpawnArgs,
    stored: &WrapperToDaemon,
) -> Result<(), String> {
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
        || parsed
            .name
            .as_ref()
            .is_some_and(|value| Some(value) != name.as_ref())
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
        SpawnRefusal::UnknownType {
            requested_type,
            known_types,
            registry,
        } => {
            let requested = (!requested_type.is_empty())
                .then_some(format!(" '{requested_type}'"))
                .unwrap_or_default();
            let types = if known_types.is_empty() {
                "indisponibles (issue historique)".to_string()
            } else {
                known_types.join(", ")
            };
            let source = if registry.is_empty() {
                "des agents"
            } else {
                registry.as_str()
            };
            format!(
                "type d'agent inconnu{requested}. Types connus du daemon : {types}. \
                 Le registre {source} est lu au démarrage du daemon ; après modification, relancez-le."
            )
        }
        SpawnRefusal::CommandMissing { command, registry } => {
            format!("commande '{command}' introuvable (registre {registry})")
        }
        SpawnRefusal::UnsupportedCapability {
            agent_type,
            model,
            capability,
        } => format!(
            "lancement refusé pour le type '{agent_type}', modèle '{model}': capacité manquante {capability}"
        ),
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
    let file_name = std::env::var("BRIDGET_AGENT_NAME_FILE")
        .ok()
        .and_then(|path| std::fs::read_to_string(path).ok());
    resolve_cli_agent_name(
        file_name.as_deref(),
        std::env::var("BRIDGET_AGENT_NAME").ok().as_deref(),
    )
}

/// Repli binaire : le nom vient du fichier puis de l'env. Sans les deux, on
/// n'invente pas d'identité d'équipier — le daemon conserve `cli-send-<pid>`.
fn resolve_cli_agent_name(file_name: Option<&str>, env_name: Option<&str>) -> String {
    if let Some(name) = file_name.map(str::trim).filter(|name| !name.is_empty()) {
        return name.to_string();
    }
    if let Some(name) = env_name.map(str::trim).filter(|name| !name.is_empty()) {
        return name.to_string();
    }
    "human".to_string()
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
    let mut id: Option<String> = None;
    let mut issued_at: Option<String> = None;
    let mut issuer_scope: Option<String> = None;
    let mut in_reply_to: Option<String> = None;
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
            "--id" => match option_value(args, &mut i, "--id") {
                Ok(value) => id = Some(value),
                Err(error) => send_usage_error(&error),
            },
            "--issued-at" => match option_value(args, &mut i, "--issued-at") {
                Ok(value) => issued_at = Some(value),
                Err(error) => send_usage_error(&error),
            },
            "--issuer-scope" => match option_value(args, &mut i, "--issuer-scope") {
                Ok(value) => issuer_scope = Some(value),
                Err(error) => send_usage_error(&error),
            },
            "--in-reply-to" => match option_value(args, &mut i, "--in-reply-to") {
                Ok(value) => in_reply_to = Some(value),
                Err(error) => send_usage_error(&error),
            },
            _ => {
                body_parts.push(args[i].clone());
            }
        }
        i += 1;
    }

    let to = match to {
        Some(t) => t,
        None => {
            eprintln!(
                "usage: bridget send --to <nom> [--in-reply-to ID] [--reply] [--hops N] <message>"
            );
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
    msg.in_reply_to = in_reply_to;
    msg.reply = effective_reply;
    msg.hops = hops;
    if let Some(t) = timeout_secs {
        msg.reply_timeout = Some(t);
    } else if effective_reply {
        msg.reply_timeout = Some(60);
    }

    let idempotent = match resolved_idempotent_options(
        id,
        issued_at,
        issuer_scope,
        msg.in_reply_to.is_some(),
        &msg.id,
    ) {
        Ok(options) => options,
        Err(error) => send_usage_error(&error),
    };

    if send_idempotent_if_requested(&mut msg, idempotent) {
        return;
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

fn option_value(args: &[String], index: &mut usize, option: &str) -> Result<String, String> {
    *index += 1;
    args.get(*index)
        .cloned()
        .filter(|value| !value.starts_with("--"))
        .ok_or_else(|| format!("{option} requiert une valeur"))
}

fn cmd_guichet(args: &[String]) {
    let request = match parse_guichet_deposit(args) {
        Ok(request) => request,
        Err(error) => {
            eprintln!("erreur: {error}");
            eprintln!(
                "usage: bridget guichet deposer <delivery-report|mission-status|deadline-question> [options]"
            );
            std::process::exit(2);
        }
    };
    let (issuer_scope, request_id, issued_at) = match &request {
        WrapperToDaemon::ServiceRequest {
            issuer_scope,
            request_id,
            issued_at,
            ..
        } => (issuer_scope.clone(), request_id.clone(), *issued_at),
        _ => unreachable!("le parseur ne construit que des dépôts guichet"),
    };
    match send_control_to_daemon(request) {
        Ok(DaemonToWrapper::GuichetResult {
            issue, expires_at, ..
        }) => {
            println!(
                "DÉPÔT: {issue} (id={request_id}, issued_at={issued_at}, issuer_scope={issuer_scope}, expire={expires_at})"
            );
            // DETTE CONNUE (voie guichet, hors périmètre de ce lot) : le jeton
            // `outcome_unknown` reste ici le nom d'un dépôt NOMINAL réussi,
            // imprimé sur stdout et suivi d'une sortie 0. C'est le défaut que
            // le lot corrige sur la voie send, non transposé : cette voie a
            // ses propres consommateurs (`scripts/install-k1.sh` filtre ce
            // jeton, `guichet_integration_test.rs` l'atteste), et les changer
            // demande son propre mandat.
            if issue != "queued" && issue != "outcome_unknown" {
                std::process::exit(1);
            }
        }
        Ok(DaemonToWrapper::ServiceRejected { reason }) => {
            eprintln!("REJET: {reason:?}");
            std::process::exit(1);
        }
        Ok(response) => {
            eprintln!("réponse inattendue du daemon: {response:?}");
            std::process::exit(1);
        }
        Err(error) => {
            eprintln!("daemon inaccessible: {error}");
            std::process::exit(1);
        }
    }
}

fn parse_guichet_deposit(args: &[String]) -> Result<WrapperToDaemon, String> {
    if args.first().map(String::as_str) != Some("deposer") {
        return Err("la seule opération guichet disponible est « deposer »".to_string());
    }
    let kind = args
        .get(1)
        .map(String::as_str)
        .ok_or_else(|| "type de dépôt manquant".to_string())?;
    let mut objective_id = None;
    let mut delegation_id = None;
    let mut delivery_hash = None;
    let mut in_reply_to = None;
    let mut from = None;
    let mut id = None;
    let mut issued_at = None;
    let mut issuer_scope = None;
    let mut verdict = None;
    let mut review_ref = None;
    let mut expected_head = None;
    let mut index = 2;
    while index < args.len() {
        let option = args[index].as_str();
        let value = option_value(args, &mut index, option)?;
        match option {
            "--objective" => objective_id = Some(value),
            "--delegation" => delegation_id = Some(value),
            "--hash" => delivery_hash = Some(value),
            "--in-reply-to" => in_reply_to = Some(value),
            "--from" => from = Some(value),
            "--id" => id = Some(value),
            "--issued-at" => issued_at = Some(value),
            "--issuer-scope" => issuer_scope = Some(value),
            "--verdict" => verdict = Some(value),
            "--review-ref" => review_ref = Some(value),
            "--expected-head" => expected_head = Some(value),
            _ => return Err(format!("option guichet inconnue: {option}")),
        }
        index += 1;
    }

    let from = from.unwrap_or_else(current_agent_name);
    if from == "human" {
        return Err("--from est requis hors wrapper Bridget".to_string());
    }
    validate_agent_name(&from)?;
    let retry = idempotent_options(id, issued_at, issuer_scope)?;
    let scope_identity =
        std::env::var("BRIDGET_AGENT_INSTANCE_ID").unwrap_or_else(|_| from.clone());
    let (request_id, issued_at, issuer_scope) = retry
        .map(|retry| (retry.id, retry.issued_at, retry.issuer_scope))
        .unwrap_or_else(|| {
            (
                uuid::Uuid::new_v4().to_string(),
                unix_timestamp(),
                crate::mcp::issuer_scope(&scope_identity),
            )
        });
    if kind != "delivery-report"
        && (verdict.is_some() || review_ref.is_some() || expected_head.is_some())
    {
        return Err("les options de verdict sont réservées à delivery-report".to_string());
    }
    let (operation, payload) = match kind {
        "delivery-report" => {
            let objective_id = objective_id.ok_or_else(|| "--objective est requis".to_string())?;
            let delegation_id =
                delegation_id.ok_or_else(|| "--delegation est requis".to_string())?;
            let delivery_hash = delivery_hash.ok_or_else(|| "--hash est requis".to_string())?;
            let in_reply_to = in_reply_to
                .ok_or_else(|| "--in-reply-to est requis pour delivery-report".to_string())?;
            let review_verdict = observe_review_verdict(verdict, review_ref, expected_head)?;
            (
                ServiceRequestOperation::DeliveryReport,
                ServiceRequestPayload::DeliveryReport {
                    objective_id,
                    delegation_id,
                    delivery_hash,
                    in_reply_to,
                    review_verdict,
                },
            )
        }
        "mission-status" => (
            ServiceRequestOperation::MissionStatus,
            ServiceRequestPayload::Delegation {
                delegation_id: delegation_id
                    .ok_or_else(|| "--delegation est requis".to_string())?,
            },
        ),
        "deadline-question" => (
            ServiceRequestOperation::DeadlineQuestion,
            ServiceRequestPayload::Delegation {
                delegation_id: delegation_id
                    .ok_or_else(|| "--delegation est requis".to_string())?,
            },
        ),
        _ => return Err(format!("type de dépôt fermé inconnu: {kind}")),
    };
    Ok(WrapperToDaemon::ServiceRequest {
        version: SERVICE_CONTRACT_VERSION,
        issuer_scope,
        request_id,
        issued_at,
        from,
        to: "maicie".to_string(),
        operation,
        payload,
    })
}

fn observe_review_verdict(
    verdict: Option<String>,
    review_ref: Option<String>,
    expected_head: Option<String>,
) -> Result<Option<ReviewVerdictEvidence>, String> {
    let (verdict, target_ref, expected_head) = match (verdict, review_ref, expected_head) {
        (None, None, None) => return Ok(None),
        (Some(verdict), Some(target_ref), Some(expected_head)) => {
            (verdict, target_ref, expected_head)
        }
        _ => {
            return Err(
                "--verdict, --review-ref et --expected-head doivent être fournis ensemble"
                    .to_string(),
            );
        }
    };
    let verdict = match verdict.as_str() {
        "approve" => ReviewVerdict::Approve,
        "approve_with_changes" => ReviewVerdict::ApproveWithChanges,
        "amender" => ReviewVerdict::Amender,
        "stop" => ReviewVerdict::Stop,
        _ => {
            return Err("--verdict attend approve|approve_with_changes|amender|stop".to_string());
        }
    };
    let target = ReviewTarget {
        target_ref,
        expected_head,
    };
    let (remote, branch) = target
        .remote_and_branch()
        .ok_or_else(|| "--review-ref attend <remote>/<branche> valide".to_string())?;
    if !is_canonical_git_sha(&target.expected_head) {
        return Err("--expected-head attend exactement 40 hexadécimaux minuscules".to_string());
    }

    let _ = git_stdout(&["remote", "get-url", remote], "remote de revue")?;
    let measured_head = git_stdout(&["rev-parse", "--verify", "HEAD^{commit}"], "HEAD")?;
    if !is_canonical_git_sha(&measured_head) {
        return Err("git rev-parse n'a pas rendu un SHA-1 canonique".to_string());
    }
    let remote_ref = format!("refs/heads/{branch}");
    let remote_output = git_stdout(
        &["ls-remote", "--exit-code", "--refs", remote, &remote_ref],
        "référence distante",
    )?;
    let mut lines = remote_output.lines();
    let line = lines
        .next()
        .ok_or_else(|| "git ls-remote n'a rendu aucune tête".to_string())?;
    if lines.next().is_some() {
        return Err("git ls-remote a rendu une tête ambiguë".to_string());
    }
    let mut fields = line.split_whitespace();
    let observed_target_head = fields
        .next()
        .ok_or_else(|| "git ls-remote n'a rendu aucun SHA".to_string())?
        .to_string();
    let observed_ref = fields
        .next()
        .ok_or_else(|| "git ls-remote n'a rendu aucune référence".to_string())?;
    if fields.next().is_some()
        || observed_ref != remote_ref
        || !is_canonical_git_sha(&observed_target_head)
    {
        return Err("git ls-remote a rendu une observation non canonique".to_string());
    }

    Ok(Some(ReviewVerdictEvidence {
        verdict,
        target_ref: target.target_ref,
        expected_head: target.expected_head,
        measured_head,
        observed_target_head,
    }))
}

fn git_stdout(arguments: &[&str], observation: &str) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(arguments)
        .output()
        .map_err(|error| format!("git indisponible pour mesurer {observation}: {error}"))?;
    if !output.status.success() {
        return Err(format!("git n'a pas pu mesurer {observation}"));
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_string())
        .map_err(|_| format!("git a rendu {observation} hors UTF-8"))
}

fn idempotent_options(
    id: Option<String>,
    issued_at: Option<String>,
    issuer_scope: Option<String>,
) -> Result<Option<IdempotentSendOptions>, String> {
    let provided = [id.is_some(), issued_at.is_some(), issuer_scope.is_some()];
    if !provided.iter().any(|provided| *provided) {
        return Ok(None);
    }
    if !provided.iter().all(|provided| *provided) {
        return Err("--id, --issued-at et --issuer-scope sont obligatoires ensemble".to_string());
    }
    let issued_at = issued_at
        .expect("présence vérifiée")
        .parse::<i64>()
        .map_err(|_| "--issued-at doit être un instant Unix entier".to_string())?;
    Ok(Some(IdempotentSendOptions {
        id: id.expect("présence vérifiée"),
        issued_at,
        issuer_scope: issuer_scope.expect("présence vérifiée"),
    }))
}

fn resolved_idempotent_options(
    id: Option<String>,
    issued_at: Option<String>,
    issuer_scope: Option<String>,
    linked_reply: bool,
    generated_id: &str,
) -> Result<Option<IdempotentSendOptions>, String> {
    if !linked_reply || issuer_scope.is_some() {
        return idempotent_options(id, issued_at, issuer_scope);
    }
    if id.is_some() != issued_at.is_some() {
        return Err(
            "--id et --issued-at sont obligatoires ensemble pour rejouer une réponse liée"
                .to_string(),
        );
    }
    let instance_id = std::env::var("BRIDGET_AGENT_INSTANCE_ID").map_err(|_| {
        "BRIDGET_AGENT_INSTANCE_ID absent : impossible de garantir un rejeu idempotent lié"
            .to_string()
    })?;
    if instance_id.is_empty() {
        return Err(
            "BRIDGET_AGENT_INSTANCE_ID vide : impossible de garantir un rejeu idempotent lié"
                .to_string(),
        );
    }
    let issued_at = match issued_at {
        Some(value) => value
            .parse::<i64>()
            .ok()
            .filter(|value| *value > 0)
            .ok_or_else(|| "--issued-at doit être un instant Unix positif".to_string())?,
        None => unix_timestamp(),
    };
    Ok(Some(IdempotentSendOptions {
        id: id.unwrap_or_else(|| generated_id.to_string()),
        issued_at,
        issuer_scope: crate::mcp::issuer_scope(&instance_id),
    }))
}

fn send_usage_error(error: &str) -> ! {
    eprintln!("erreur: {error}");
    eprintln!(
        "usage: bridget send --to <nom> [--in-reply-to ID] [--id <clé> --issued-at <unix> [--issuer-scope <portée>]] <message>"
    );
    std::process::exit(2);
}

fn print_idempotency_issue(issue: &IdempotencyIssue, options: &IdempotentSendOptions) {
    match issue {
        IdempotencyIssue::Accepted { expires_at } => {
            println!(
                "OK: accepted id={} issued_at={} expires_at={expires_at}",
                options.id, options.issued_at
            );
        }
        IdempotencyIssue::Rejected {
            category, reason, ..
        } => eprintln!(
            "REJET: {} id={} issued_at={}: {reason}",
            crate::mcp::public_refusal_category(category),
            options.id,
            options.issued_at
        ),
        // Le binaire nomme le MÊME statut que le retour MCP, via le même point
        // de vérité : un agent qui lit les deux surfaces n'a aucune traduction
        // à faire, et elles ne peuvent pas diverger.
        //
        // Le discriminant passe par `attestation_de_depot`, pas par un `Some`
        // nu : un `Some("")` imprimait « en vol » sur la sortie standard alors
        // que le code de sortie le refusait déjà en échec — la ligne disait
        // dépôt, le `rc` disait panne.
        IdempotencyIssue::OutcomeUnknown { delivery_id, .. } => {
            match crate::mcp::attestation_de_depot(delivery_id.as_deref()) {
                // Une remise en vol n'est pas une panne : elle passe par la
                // sortie standard, comme le succès dont elle est le premier
                // temps.
                Some(delivery_id) => println!(
                    "DÉPÔT: {} (remise en vol) id={} issued_at={} delivery_id={delivery_id} — {}",
                    crate::mcp::STATUT_IN_FLIGHT,
                    options.id,
                    options.issued_at,
                    crate::mcp::REJEU_A_L_IDENTIQUE
                ),
                None => eprintln!(
                    "ISSUE: {} id={} issued_at={} — sort indéterminé ; {}",
                    crate::mcp::STATUT_OUTCOME_UNKNOWN,
                    options.id,
                    options.issued_at,
                    crate::mcp::REJEU_A_L_IDENTIQUE
                ),
            }
        }
        IdempotencyIssue::EnvelopeMismatch => eprintln!(
            "REJET: envelope_mismatch id={} issued_at={}",
            options.id, options.issued_at
        ),
        IdempotencyIssue::IdempotencyExpired => eprintln!(
            "REJET: idempotency_expired id={} issued_at={}",
            options.id, options.issued_at
        ),
        IdempotencyIssue::InvalidIssuedAt => eprintln!(
            "REJET: invalid_issued_at id={} issued_at={}",
            options.id, options.issued_at
        ),
    }
}

/// Le code de sortie suit le DÉPÔT, pas la consolidation de l'accusé aval.
///
/// Le daemon répond avant que le destinataire ait accusé : sur un premier envoi
/// nominal l'issue est `OutcomeUnknown`, et un `delivery_id` atteste qu'il a
/// pris la remise. Traiter ce cas en échec faisait sortir en `rc=1` tout envoi
/// réussi. Sans `delivery_id`, le sort est réellement indéterminé : l'échec est
/// alors honnête.
pub(crate) fn send_deposited(issue: &IdempotencyIssue) -> bool {
    match issue {
        IdempotencyIssue::Accepted { .. } => true,
        // Même point de vérité que la ligne imprimée et que le `status` MCP :
        // le code de sortie ne peut donc pas contredire ce qui est affiché.
        // Un identifiant vide n'atteste rien — le daemon n'en produit jamais,
        // et le prendre pour une preuve ferait sortir en succès sur une valeur
        // que lui-même refuserait.
        IdempotencyIssue::OutcomeUnknown { delivery_id, .. } => {
            crate::mcp::attestation_de_depot(delivery_id.as_deref()).is_some()
        }
        _ => false,
    }
}

fn send_idempotent_if_requested(
    message: &mut BridgetMessage,
    options: Option<IdempotentSendOptions>,
) -> bool {
    let Some(options) = options else {
        return false;
    };
    message.id = options.id.clone();
    match send_idempotent_to_daemon(message, &options) {
        Ok(DaemonToWrapper::IdempotencyResult { issue, .. }) => {
            print_idempotency_issue(&issue, &options);
            if !send_deposited(&issue) {
                std::process::exit(1);
            }
        }
        Ok(DaemonToWrapper::ClientRejected { reason }) => {
            eprintln!("REJET: {reason:?}");
            std::process::exit(1);
        }
        Ok(_) => {
            eprintln!("réponse inattendue du daemon");
            std::process::exit(1);
        }
        Err(error) => {
            eprintln!("daemon inaccessible: {error}");
            std::process::exit(1);
        }
    }
    true
}

fn send_to_daemon(msg: &BridgetMessage) -> Result<DaemonToWrapper, String> {
    send_control_to_daemon(WrapperToDaemon::Send(msg.clone()))
}

fn send_idempotent_to_daemon(
    message: &BridgetMessage,
    options: &IdempotentSendOptions,
) -> Result<DaemonToWrapper, String> {
    send_idempotent_to_daemon_at(&socket_path(), message, options)
}

fn send_idempotent_to_daemon_at(
    path: &std::path::Path,
    message: &BridgetMessage,
    options: &IdempotentSendOptions,
) -> Result<DaemonToWrapper, String> {
    let stream = UnixStream::connect(path).map_err(|error| error.to_string())?;
    let read_stream = stream.try_clone().map_err(|error| error.to_string())?;
    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(read_stream);

    write_control_message(
        &mut writer,
        &WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Client,
        },
    )?;
    match read_control_message(&mut reader)? {
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client,
        } => {}
        response => return Err(format!("handshake client refusé: {response:?}")),
    }

    write_control_message(
        &mut writer,
        &WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: options.issuer_scope.clone(),
            capabilities: vec![ClientCapability::SendIdempotent],
        },
    )?;
    match read_control_message(&mut reader)? {
        DaemonToWrapper::ClientWelcome {
            capabilities,
            build_id,
            ..
        } if capabilities.contains(&ClientCapability::SendIdempotent) => {
            if let Some(warning) = crate::build_info::stale_daemon_warning(&build_id) {
                eprintln!("{warning}");
            }
        }
        DaemonToWrapper::ClientRejected { reason } => {
            return Ok(DaemonToWrapper::ClientRejected { reason });
        }
        response => return Err(format!("négociation client refusée: {response:?}")),
    }

    write_control_message(
        &mut writer,
        &WrapperToDaemon::SendIdempotent {
            message: message.clone(),
            message_id: options.id.clone(),
            issued_at: options.issued_at,
        },
    )?;
    read_control_message(&mut reader)
}

fn write_control_message(
    writer: &mut BufWriter<UnixStream>,
    message: &WrapperToDaemon,
) -> Result<(), String> {
    writeln!(
        writer,
        "{}",
        encode(message).map_err(|error| error.to_string())?
    )
    .map_err(|error| error.to_string())?;
    writer.flush().map_err(|error| error.to_string())
}

fn read_control_message(reader: &mut BufReader<UnixStream>) -> Result<DaemonToWrapper, String> {
    let mut line = String::new();
    let bytes = reader
        .read_line(&mut line)
        .map_err(|error| error.to_string())?;
    if bytes == 0 {
        return Err("daemon a fermé la connexion".to_string());
    }
    decode(line.trim_end()).map_err(|error| error.to_string())
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
        channel: bridget_transport::ChannelReport::Unknown,
        mode: Some(PresenceMode::Cli),
        location: None,
        os: None,
        instance_id: None,
        domain: None,
        turn_in_progress: false,
        journal_available: None,
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
    let options = match parse_requests_args(args) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("usage: bridget requests [--all] [--json]");
            std::process::exit(2);
        }
    };
    let response = if options.all {
        // Vue globale : primitive ledger/open_requests, pas ListRequests.
        send_control_to_daemon(WrapperToDaemon::LedgerProjection {
            scope: LedgerScope::Requests,
            limit: 200,
        })
        .and_then(|response| match response {
            DaemonToWrapper::LedgerProjection { requests, .. } => {
                Ok(DaemonToWrapper::RequestList { requests })
            }
            DaemonToWrapper::Nack { reason, .. } => Err(format!("erreur ledger: {reason}")),
            _ => Err("réponse inattendue du daemon".to_string()),
        })
    } else {
        send_control_to_daemon(WrapperToDaemon::ListRequests {
            sender: current_agent_name(),
            limit: 200,
        })
    };
    match response {
        Ok(DaemonToWrapper::RequestList { requests }) if options.json => println!(
            "{}",
            serde_json::to_string(&requests).unwrap_or_else(|_| "[]".to_string())
        ),
        Ok(DaemonToWrapper::RequestList { requests }) => {
            print!("{}", render_requests(&requests, options.all));
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RequestsOptions {
    all: bool,
    json: bool,
}

fn parse_requests_args(args: &[String]) -> Result<RequestsOptions, String> {
    let mut options = RequestsOptions {
        all: false,
        json: false,
    };
    for arg in args {
        match arg.as_str() {
            "--all" => options.all = true,
            "--json" => options.json = true,
            other => return Err(format!("option requests inconnue: {other}")),
        }
    }
    Ok(options)
}

/// Rend la table des demandes. `global` ajoute la colonne émetteur (de → vers).
pub(crate) fn render_requests(requests: &[RequestInfo], global: bool) -> String {
    if requests.is_empty() {
        return if global {
            "Aucune demande ouverte.\n".to_string()
        } else {
            "Aucune demande suivie.\n".to_string()
        };
    }
    let id_width = requests
        .iter()
        .map(|request| request.id.len())
        .max()
        .unwrap_or(2)
        .max(2);
    let state_width = requests
        .iter()
        .map(|request| request.state.len())
        .max()
        .unwrap_or(4)
        .max(4);
    let mut output = String::new();
    if global {
        let from_width = requests
            .iter()
            .map(|request| request.sender.len())
            .max()
            .unwrap_or(2)
            .max(2);
        let to_width = requests
            .iter()
            .map(|request| request.target.len())
            .max()
            .unwrap_or(4)
            .max(4);
        let _ = writeln!(
            output,
            "{:<id_width$}  {:<from_width$}  {:<to_width$}  {:<state_width$}  ÉCHÉANCE  REPORT",
            "ID", "DE", "VERS", "ÉTAT"
        );
        for request in requests {
            let _ = writeln!(
                output,
                "{:<id_width$}  {:<from_width$}  {:<to_width$}  {:<state_width$}  {}  {}",
                request.id,
                request.sender,
                request.target,
                request.state,
                request.deadline_at,
                request_report(request)
            );
        }
    } else {
        let target_width = requests
            .iter()
            .map(|request| request.target.len())
            .max()
            .unwrap_or(11)
            .max(11);
        let _ = writeln!(
            output,
            "{:<id_width$}  {:<target_width$}  {:<state_width$}  ÉCHÉANCE  REPORT",
            "ID", "DESTINATAIRE", "ÉTAT"
        );
        for request in requests {
            let _ = writeln!(
                output,
                "{:<id_width$}  {:<target_width$}  {:<state_width$}  {}  {}",
                request.id,
                request.target,
                request.state,
                request.deadline_at,
                request_report(request)
            );
        }
    }
    output
}

fn request_report(request: &RequestInfo) -> String {
    request
        .deferred_reminder_level
        .map(|level| {
            format!(
                "palier {level} @ {}",
                request.deferred_reminder_at.unwrap_or_default()
            )
        })
        .unwrap_or_else(|| "—".to_string())
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
        channel: bridget_transport::ChannelReport::Unknown,
        mode: Some(PresenceMode::Cli),
        location: None,
        os: None,
        instance_id: None,
        domain: None,
        turn_in_progress: false,
        journal_available: None,
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
        .set_read_timeout(Some(std::time::Duration::from_secs(
            RUNTIME_REPLY_TIMEOUT_SECS,
        )))
        .map_err(|e| e.to_string())?;
    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(read_stream);
    let register = WrapperToDaemon::Register {
        agent_type: "cli".to_string(),
        name: Some(format!("cli-runtime-{}", std::process::id())),
        host: None,
        transport: None,
        channel: bridget_transport::ChannelReport::Unknown,
        mode: Some(PresenceMode::Cli),
        location: None,
        os: None,
        instance_id: None,
        domain: None,
        turn_in_progress: false,
        journal_available: None,
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
        Some("claude-statusline") => hook_claude_statusline(),
        Some(other) => {
            log::debug!("hook inconnu: {}", other);
        }
        None => {
            eprintln!("usage: bridget hook <claude-runtime|claude-statusline>");
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
    // Volet 3 (format LIMITE) : le payload Stop n'atteste PAS de limites.
    // Les champs stables sont session_id, transcript_path, cwd, hook_event_name,
    // stop_hook_active, last_assistant_message. Les rate_limits (five_hour /
    // seven_day + used_percentage) vivent dans le payload StatusLine, pas Stop.
    // On ne pousse donc aucune limite depuis ce hook : la ligne référent reste
    // honnêtement vide tant qu'aucun flux natif (stream-json) n'a observé.
    let Some(transcript) = payload.get("transcript_path").and_then(|v| v.as_str()) else {
        log::debug!("hook claude-runtime : pas de transcript_path");
        return;
    };
    let transcript_path = std::path::Path::new(transcript);
    let session_id = payload.get("session_id").and_then(|v| v.as_str());
    // Claude Code nomme le fichier `{session_id}.jsonl`. Si le payload porte
    // les deux, exiger l'accord : un chemin voisin contaminerait la présence.
    if !transcript_matches_hook_session(transcript_path, session_id) {
        log::debug!(
            "hook claude-runtime : transcript {:?} ≠ session_id {:?}",
            transcript_path.file_name(),
            session_id
        );
        return;
    }
    // Journalisé pour rendre diagnosticable le cas d'une session Claude
    // imbriquée qui hériterait du nom de l'agent parent (research.md D-002).
    log::debug!(
        "hook claude-runtime : agent={} session={:?}",
        agent,
        session_id
    );

    let Some(observed) = crate::runtime::parse_claude_transcript(transcript_path) else {
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

/// Statut posé sur un fait de limite venu du StatusLine.
///
/// Le payload atteste une consommation et un instant de retour, jamais un
/// verdict du fournisseur : écrire « allowed » inventerait une décision que
/// Claude Code n'a pas rendue. « unknown » n'affirme rien, et n'active pas le
/// rendu « épuisée » de `format_one_rate_limit`, réservé à « rejected ».
const STATUSLINE_LIMIT_STATUS: &str = "unknown";

/// Fenêtres de forfait portées par le payload StatusLine, dans l'ordre de
/// poussée. Fermée à dessein : une clé inconnue du bloc `rate_limits` n'est
/// pas remontée, faute de savoir ce qu'elle mesure.
const STATUSLINE_WINDOWS: [&str; 2] = ["five_hour", "seven_day"];

/// Fait de limite relevé dans un payload StatusLine, avant envoi.
#[derive(Debug, PartialEq, Eq)]
struct StatusLineLimit {
    window: String,
    used_percent: Option<u8>,
    resets_at: Option<i64>,
}

/// Hook StatusLine : pousse les limites de forfait attestées par Claude Code.
///
/// Seule source de limite pour un Claude interactif — le flux `stream-json`,
/// qui porte les `rate_limit_event`, n'existe que pour les agents gérés.
///
/// Contrat identique au hook de fin de tour : sortie standard VIDE, code de
/// retour 0. Le réglage `statusLine` de Claude Code n'accepte qu'un seul
/// programme, et c'est celui de l'utilisateur qui doit afficher la ligne :
/// ce hook s'appelle EN PLUS, en lui repassant le payload —
/// `printf '%s' "$input" | bridget hook claude-statusline &`.
///
/// Aucune limitation de débit ici, alors que le StatusLine s'exécute souvent
/// (déclenchement par événement, débounce 300 ms). C'est délibéré : le daemon
/// fait un upsert par fenêtre, et republier à chaque tour est précisément ce
/// qui répare la présence après un redémarrage du daemon. Un cache « ne
/// renvoyer que si la valeur change » recréerait le trou d'affichage déjà
/// constaté sur la sonde de runtime.
fn hook_claude_statusline() {
    // Hors d'un agent Bridget, le hook est inerte : les sessions Claude
    // ordinaires de l'utilisateur ne doivent subir aucun effet.
    let agent = current_agent_name();
    if agent == "human" {
        return;
    }

    let mut payload = String::new();
    if std::io::Read::read_to_string(&mut std::io::stdin(), &mut payload).is_err() {
        log::debug!("hook claude-statusline : payload illisible");
        return;
    }
    let Ok(payload) = serde_json::from_str::<serde_json::Value>(&payload) else {
        log::debug!("hook claude-statusline : payload non JSON");
        return;
    };

    let facts = rate_limit_facts_from_statusline(&payload);
    if facts.is_empty() {
        // Cas nominal, pas une panne : le bloc `rate_limits` n'existe que pour
        // un abonné Claude.ai, et seulement après une première réponse de
        // l'API. Tant qu'il manque, il n'y a rien à attester.
        log::debug!("hook claude-statusline : aucune limite attestée");
        return;
    }
    if let Err(error) = send_rate_limits_to_daemon(&socket_path(), &agent, &facts) {
        log::debug!("hook claude-statusline : {}", error);
    }
}

/// Relève les limites d'un payload StatusLine. Atteste ou rien.
///
/// Le bloc `rate_limits` est optionnel, et chacune de ses fenêtres l'est aussi
/// (schéma Claude Code : « Only present for subscribers after first API
/// response »). Une fenêtre sans aucune valeur exploitable ne produit pas de
/// fait : elle occuperait la colonne sans rien y dire.
fn rate_limit_facts_from_statusline(payload: &serde_json::Value) -> Vec<StatusLineLimit> {
    let Some(limits) = payload
        .get("rate_limits")
        .and_then(serde_json::Value::as_object)
    else {
        return Vec::new();
    };
    STATUSLINE_WINDOWS
        .iter()
        .filter_map(|window| {
            let block = limits.get(*window)?;
            let used_percent = block.get("used_percentage").and_then(statusline_percent);
            let resets_at = block.get("resets_at").and_then(statusline_epoch);
            if used_percent.is_none() && resets_at.is_none() {
                return None;
            }
            Some(StatusLineLimit {
                window: (*window).to_string(),
                used_percent,
                resets_at,
            })
        })
        .collect()
}

/// `used_percentage` est un nombre de 0 à 100, parfois fractionnaire.
///
/// Tronqué vers le bas, jamais rabattu dans les bornes : une valeur hors
/// domaine n'est pas une valeur à corriger, c'est une valeur qu'on n'a pas
/// comprise. La ramener à 100 afficherait une saturation que le fournisseur
/// n'a pas annoncée — et le daemon refuse déjà tout pourcentage > 100.
fn statusline_percent(value: &serde_json::Value) -> Option<u8> {
    let raw = value.as_f64()?;
    if !raw.is_finite() || !(0.0..=100.0).contains(&raw) {
        return None;
    }
    Some(raw.trunc() as u8)
}

/// `resets_at` est un instant Unix en secondes. Le daemon refuse `<= 0`.
fn statusline_epoch(value: &serde_json::Value) -> Option<i64> {
    value.as_i64().filter(|seconds| *seconds > 0)
}

/// Envoie les faits relevés sur une seule connexion : deux fenêtres ne valent
/// pas deux allers-retours.
///
/// La connexion ne se DÉCLARE PAS : aucun `Register` n'est émis, donc aucune
/// présence n'est inscrite à l'annuaire. C'est volontaire et c'est le cœur du
/// correctif. Le StatusLine s'exécute à chaque tour (débounce 300 ms) ; un
/// `Register` par poussée peuplait l'annuaire de `cli-statusline-<pid>`
/// éphémères, comptés comme agents LIBRES par les rondes.
///
/// Cette suppression ne coûte rien à l'attribution : le daemon route un
/// `RateLimit` sur le champ `agent` du message — `handle_rate_limit` résout la
/// cible par `presence_of_agent(state, agent)` et ne lit jamais l'identité de
/// la connexion émettrice. Il n'y a pas non plus de poignée de main
/// obligatoire : la boucle de connexion décode chaque ligne et la dispatche
/// telle quelle, et l'accusé repart par le writer de la connexion, pas par le
/// registre des présences. Le nom de repli n'attestait donc rien — il ne
/// faisait que du bruit.
fn send_rate_limits_to_daemon(
    socket: &std::path::Path,
    agent: &str,
    facts: &[StatusLineLimit],
) -> Result<(), String> {
    let stream = UnixStream::connect(socket).map_err(|e| e.to_string())?;
    let read_stream = stream.try_clone().map_err(|e| e.to_string())?;
    // Même garde que la sonde de runtime : un daemon d'une version antérieure
    // ignore ce message, et sans délai borné le hook bloquerait le
    // rafraîchissement de la ligne d'état de l'agent observé.
    read_stream
        .set_read_timeout(Some(std::time::Duration::from_secs(
            RUNTIME_REPLY_TIMEOUT_SECS,
        )))
        .map_err(|e| e.to_string())?;
    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(read_stream);
    let mut line = String::new();

    for fact in facts {
        let message = WrapperToDaemon::RateLimit {
            agent: agent.to_string(),
            window: fact.window.clone(),
            status: STATUSLINE_LIMIT_STATUS.to_string(),
            resets_at: fact.resets_at,
            used_percent: fact.used_percent,
            source: bridget_transport::protocol::RateLimitSource::ClaudeStatusLine,
        };
        writeln!(writer, "{}", encode(&message).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;
        line.clear();
        reader.read_line(&mut line).map_err(|e| e.to_string())?;
        match decode(line.trim()).map_err(|e| e.to_string())? {
            DaemonToWrapper::Ack { .. } => {}
            DaemonToWrapper::Nack { reason, .. } => {
                return Err(format!("limite {} refusée: {}", fact.window, reason));
            }
            other => return Err(format!("réponse inattendue {:?}", other)),
        }
    }
    Ok(())
}

/// Vérifie que le chemin de transcript appartient bien à la session du hook.
///
/// Sans `session_id`, on ne peut pas trancher : le chemin fourni par Claude
/// Code reste l'unique source. Avec les deux, le stem du fichier doit être
/// l'identifiant — sinon c'est un voisin du même projet.
fn transcript_matches_hook_session(transcript: &std::path::Path, session_id: Option<&str>) -> bool {
    let Some(session_id) = session_id.filter(|value| !value.is_empty()) else {
        return true;
    };
    transcript
        .file_stem()
        .and_then(|value| value.to_str())
        .is_some_and(|stem| stem == session_id)
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
    let hooks = root.entry("hooks").or_insert_with(|| serde_json::json!({}));
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
            hooks
                .iter()
                .any(|hook| hook.get("command").and_then(|c| c.as_str()) == Some(HOOK_COMMAND))
        })
        .unwrap_or(false)
}

/// Horodatage `AAAAMMJJ-HHMMSS` en temps local, pour nommer une sauvegarde.
fn timestamp() -> String {
    let output = std::process::Command::new("date")
        .arg("+%Y%m%d-%H%M%S")
        .output();
    match output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
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
        && let Err(reason) = validate_agent_name(name)
    {
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
                    println!(
                        "Domaine de « {} » réinitialisé sur le dépôt courant.",
                        agent
                    );
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
    let implicit_in_reply_to = previous_parts.next().map(str::to_string);
    if to.is_empty() {
        eprintln!("reply: expediteur precedent vide.");
        std::process::exit(1);
    }

    let mut reply_flag = false;
    let mut hops: i32 = 4;
    let mut timeout_secs: Option<u64> = None;
    let mut explicit_in_reply_to: Option<String> = None;
    let mut id: Option<String> = None;
    let mut issued_at: Option<String> = None;
    let mut issuer_scope: Option<String> = None;
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
            "--in-reply-to" => match option_value(args, &mut i, "--in-reply-to") {
                Ok(value) => explicit_in_reply_to = Some(value),
                Err(error) => {
                    eprintln!("reply: {error}");
                    eprintln!(
                        "usage: bridget reply [--in-reply-to ID] [--reply] [--hops N] <message>"
                    );
                    std::process::exit(2);
                }
            },
            "--id" => match option_value(args, &mut i, "--id") {
                Ok(value) => id = Some(value),
                Err(error) => send_usage_error(&error),
            },
            "--issued-at" => match option_value(args, &mut i, "--issued-at") {
                Ok(value) => issued_at = Some(value),
                Err(error) => send_usage_error(&error),
            },
            "--issuer-scope" => match option_value(args, &mut i, "--issuer-scope") {
                Ok(value) => issuer_scope = Some(value),
                Err(error) => send_usage_error(&error),
            },
            _ => {
                body_parts.push(args[i].clone());
            }
        }
        i += 1;
    }

    let body = body_parts.join(" ");
    if body.is_empty() {
        eprintln!("usage: bridget reply [--in-reply-to ID] [--reply] [--hops N] <message>");
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
    msg.in_reply_to = explicit_in_reply_to.or(implicit_in_reply_to);
    msg.reply = effective_reply;
    msg.hops = hops;
    if let Some(t) = timeout_secs {
        msg.reply_timeout = Some(t);
    } else if effective_reply {
        msg.reply_timeout = Some(60);
    }

    let idempotent = match resolved_idempotent_options(
        id,
        issued_at,
        issuer_scope,
        msg.in_reply_to.is_some(),
        &msg.id,
    ) {
        Ok(options) => options,
        Err(error) => send_usage_error(&error),
    };
    if send_idempotent_if_requested(&mut msg, idempotent) {
        return;
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
                "  {} ({}) [{}] — {} / {} via {} (canal {}) — {} / {} [{}]",
                agent.name,
                agent.agent_type,
                cell(agent.domain.as_deref()),
                agent.host,
                agent.os,
                agent.transport,
                cell(agent.channel.as_deref()),
                format_model(agent),
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
    let build_id = status.build_id.as_deref().unwrap_or("inconnu");
    let agents: Vec<_> = match &filter {
        Some(domain) => status
            .agents
            .into_iter()
            .filter(|agent| agent.domain.as_deref() == Some(domain.as_str()))
            .collect(),
        None => status.agents,
    };

    print!("{}", render_who(&agents, filter.as_deref()));
    println!("Daemon build-id: {build_id}");
    emit_stale_daemon_warning(status.build_id.as_deref());
    emit_disk_warning();
}

/// Rend l'annuaire sans dépendre d'un terminal : les appels non-TTY reçoivent
/// exactement la même projection que la sous-commande who.
fn render_who(agents: &[AgentInfo], filter: Option<&str>) -> String {
    if agents.is_empty() {
        return match filter {
            Some(domain) => format!("Aucun agent dans le domaine « {} ».\n", domain),
            None => "Aucun agent connecté.\n".to_string(),
        };
    }

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
    let channel_w = column("CANAL", &|a: &AgentInfo| {
        cell(a.channel.as_deref()).to_string()
    });
    let mode_w = column("MODE", &|a: &AgentInfo| {
        cell(a.mode.map(PresenceMode::as_str)).to_string()
    });
    let location_w = column("LOCALISATION", &|a: &AgentInfo| {
        cell(a.location.as_deref()).to_string()
    });
    let domain_w = column("DOMAINE", &|a: &AgentInfo| {
        cell(a.domain.as_deref()).to_string()
    });
    let model_w = column("MODÈLE", &|a: &AgentInfo| format_model(a));
    let effort_w = column("EFFORT", &|a: &AgentInfo| {
        cell(a.effort.as_deref()).to_string()
    });
    let rate_limit_w = column("LIMITE", &|a: &AgentInfo| format_rate_limit(a));

    let mut output = String::new();
    match filter {
        Some(domain) => writeln!(output, "Agents du domaine « {} » :", domain).unwrap(),
        None => writeln!(output, "Agents connectés :").unwrap(),
    }
    writeln!(
        output,
        "  {:<name_w$}  {:<type_w$}  {:<host_w$}  {:<os_w$}  {:<transport_w$}  {:<channel_w$}  {:<mode_w$}  {:<location_w$}  {:<domain_w$}  {:<model_w$}  {:<effort_w$}  {:<rate_limit_w$}  ÉTAT",
        "NOM", "TYPE", "HÔTE", "OS", "TRANSPORT", "CANAL", "MODE", "LOCALISATION", "DOMAINE", "MODÈLE", "EFFORT", "LIMITE"
    )
    .unwrap();
    for agent in agents {
        writeln!(
            output,
            "  {:<name_w$}  {:<type_w$}  {:<host_w$}  {:<os_w$}  {:<transport_w$}  {:<channel_w$}  {:<mode_w$}  {:<location_w$}  {:<domain_w$}  {:<model_w$}  {:<effort_w$}  {:<rate_limit_w$}  {}",
            agent.name,
            agent.agent_type,
            agent.host,
            agent.os,
            agent.transport,
            cell(agent.channel.as_deref()),
            cell(agent.mode.map(PresenceMode::as_str)),
            cell(agent.location.as_deref()),
            cell(agent.domain.as_deref()),
            format_model(agent),
            cell(agent.effort.as_deref()),
            format_rate_limit(agent),
            agent.state
        )
        .unwrap();
    }
    output
}

/// Rend une valeur d'annuaire affichable : un tiret cadratin marque une valeur
/// inconnue, ce qui la distingue d'une valeur vide qui casserait la lecture des
/// colonnes.
fn cell(value: Option<&str>) -> &str {
    value.unwrap_or("—")
}

/// Marqueur d'écart : le modèle servi précède l'épinglé. Sans signal de flux,
/// on conserve le tiret ou le modèle épinglé, sans inventer de verdict.
fn format_model(agent: &AgentInfo) -> String {
    match &agent.model_mismatch {
        Some(gap) => format!("{} ≠ {}", gap.served, gap.pinned),
        None => cell(agent.model.as_deref()).to_string(),
    }
}

/// Affiche les fenêtres attestées en format compact, côte à côte.
/// Absent = « — » ; une fenêtre non observée n'apparaît pas (pas de « 5h — »).
fn format_rate_limit(agent: &AgentInfo) -> String {
    if agent.rate_limits.is_empty() {
        return "—".to_string();
    }
    agent
        .rate_limits
        .iter()
        .map(format_one_rate_limit)
        .collect::<Vec<_>>()
        .join(" · ")
}

fn format_one_rate_limit(limit: &bridget_transport::protocol::RateLimitFact) -> String {
    let mut parts = vec![abbreviate_window(&limit.window)];
    if let Some(pct) = limit.used_percent {
        parts.push(format!("{pct}%"));
    } else if limit.status == "rejected" {
        parts.push("épuisée".to_string());
    }
    if let Some(reset) = limit.resets_at.and_then(format_local_reset) {
        parts.push(format!("rst {reset}"));
    }
    parts.join(" ")
}

/// Abrège un nom de fenêtre attesté sans jeter les inconnues.
/// `five_hour`→`5h`, `seven_day`/`weekly`→`7d`, `primary/300m`→`5h` via
/// la durée ; sinon raccourci du nom brut.
///
/// Les motifs connus ne matchent qu'en égalité exacte ou comme jeton entier
/// (frontière hors `[A-Za-z0-9_]`). Un faux-ami du type `not_five_hour_custom`
/// n'est donc pas abrégé en `5h`.
fn abbreviate_window(name: &str) -> String {
    let lower = name.to_ascii_lowercase();
    if window_token_matches(&lower, "five_hour") {
        return "5h".to_string();
    }
    if window_token_matches(&lower, "seven_day") || window_token_matches(&lower, "weekly") {
        return "7d".to_string();
    }
    if let Some(mins) = duration_mins_from_window(name) {
        return abbreviate_minutes(mins);
    }
    shorten_raw_window(name)
}

/// Vrai si `needle` est le nom entier ou un jeton délimité (pas un sous-mot).
fn window_token_matches(haystack: &str, needle: &str) -> bool {
    if haystack == needle {
        return true;
    }
    haystack
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .any(|part| part == needle)
}

fn duration_mins_from_window(name: &str) -> Option<i64> {
    let tail = name.rsplit('/').next().unwrap_or(name);
    let digits = tail.trim_end_matches('m').trim_end_matches('M');
    if digits.chars().all(|c| c.is_ascii_digit()) && !digits.is_empty() {
        digits.parse().ok()
    } else {
        None
    }
}

fn abbreviate_minutes(mins: i64) -> String {
    if mins <= 0 {
        return format!("{mins}m");
    }
    if mins % (60 * 24) == 0 {
        return format!("{}d", mins / (60 * 24));
    }
    if mins % 60 == 0 {
        return format!("{}h", mins / 60);
    }
    format!("{mins}m")
}

fn shorten_raw_window(name: &str) -> String {
    let compact: String = name.chars().filter(|c| *c != '_').take(12).collect();
    if compact.is_empty() {
        "?".to_string()
    } else {
        compact
    }
}

/// Heure locale ; date incluse si la réinitialisation est à plus d'un jour.
fn format_local_reset(timestamp: i64) -> Option<String> {
    let seconds: libc::time_t = timestamp;
    let mut local: libc::tm = unsafe { std::mem::zeroed() };
    if unsafe { libc::localtime_r(&seconds, &mut local) }.is_null() {
        return None;
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs() as i64;
    let day_secs = 24 * 60 * 60;
    let time = format!("{:02}:{:02}", local.tm_hour, local.tm_min);
    if (timestamp - now).abs() >= day_secs {
        // tm_mon est 0-indexé ; tm_year depuis 1900.
        Some(format!(
            "{:02}/{:02} {time}",
            local.tm_mday,
            local.tm_mon + 1
        ))
    } else {
        Some(time)
    }
}

fn cmd_discover() {
    cmd_who(&[]);
}

fn cmd_reprise(args: &[String]) {
    let mut write_path: Option<PathBuf> = None;
    let mut pin_path: Option<PathBuf> = None;
    let mut repo_path: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--write" => {
                i += 1;
                let Some(path) = args.get(i) else {
                    eprintln!(
                        "usage: bridget reprise [--write <chemin>] [--pin <chemin>] [--repo <chemin>]"
                    );
                    std::process::exit(2);
                };
                write_path = Some(PathBuf::from(path));
            }
            "--pin" => {
                i += 1;
                let Some(path) = args.get(i) else {
                    eprintln!(
                        "usage: bridget reprise [--write <chemin>] [--pin <chemin>] [--repo <chemin>]"
                    );
                    std::process::exit(2);
                };
                pin_path = Some(PathBuf::from(path));
            }
            "--repo" => {
                i += 1;
                let Some(path) = args.get(i) else {
                    eprintln!(
                        "usage: bridget reprise [--write <chemin>] [--pin <chemin>] [--repo <chemin>]"
                    );
                    std::process::exit(2);
                };
                repo_path = Some(PathBuf::from(path));
            }
            other => {
                eprintln!("option reprise inconnue: {other}");
                eprintln!(
                    "usage: bridget reprise [--write <chemin>] [--pin <chemin>] [--repo <chemin>]"
                );
                std::process::exit(2);
            }
        }
        i += 1;
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let repo = repo_path
        .or_else(|| crate::reprise::discover_repo(&cwd))
        .unwrap_or_else(|| cwd.clone());
    let pin = pin_path.unwrap_or_else(|| repo.join(crate::reprise::DEFAULT_PIN_REL));
    let pin_ref = pin.exists().then_some(pin.as_path());

    let config = DaemonConfig::default();
    let snapshot = crate::reprise::collect_snapshot(
        &config,
        &repo,
        pin_ref,
        None,
        std::time::SystemTime::now(),
    );
    let card = crate::reprise::render_card(&snapshot);

    match write_path {
        Some(path) => {
            let path = if path.as_os_str().is_empty() {
                repo.join(crate::reprise::DEFAULT_GENERATED_REL)
            } else {
                path
            };
            if let Err(error) = crate::reprise::write_card(&path, &card) {
                eprintln!("bridget reprise: {error}");
                std::process::exit(1);
            }
            println!("carte écrite: {}", path.display());
        }
        None => print!("{card}"),
    }
}

fn cmd_reaper(args: &[String]) {
    let sub = args.first().map(String::as_str).unwrap_or("");
    if sub != "report" {
        eprintln!(
            "usage: bridget reaper report [--json] [--state-dir DIR] [--tmp DIR] [--min-age-secs N]"
        );
        eprintln!("phase observer uniquement — aucune action destructive n'existe");
        std::process::exit(2);
    }
    let mut json_output = false;
    let mut state_dir = crate::reaper::default_state_dir();
    let mut tmp_dir = std::env::temp_dir();
    let mut min_age_secs = crate::reaper::DEFAULT_MIN_AGE_SECS;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_output = true;
            }
            "--state-dir" => match option_value(args, &mut i, "--state-dir") {
                Ok(value) => state_dir = PathBuf::from(value),
                Err(error) => {
                    eprintln!("bridget reaper: {error}");
                    std::process::exit(2);
                }
            },
            "--tmp" => match option_value(args, &mut i, "--tmp") {
                Ok(value) => tmp_dir = PathBuf::from(value),
                Err(error) => {
                    eprintln!("bridget reaper: {error}");
                    std::process::exit(2);
                }
            },
            "--min-age-secs" => match option_value(args, &mut i, "--min-age-secs") {
                Ok(value) => match value.parse::<u64>() {
                    Ok(secs) => min_age_secs = secs,
                    Err(_) => {
                        eprintln!("bridget reaper: --min-age-secs attend un entier");
                        std::process::exit(2);
                    }
                },
                Err(error) => {
                    eprintln!("bridget reaper: {error}");
                    std::process::exit(2);
                }
            },
            other => {
                eprintln!("bridget reaper: option inconnue: {other}");
                std::process::exit(2);
            }
        }
        i += 1;
    }

    // Relève disque à chaque observation (fait attesté, pas de panique auto).
    crate::disk_hygiene::warn_if_disk_low(Path::new("/"));

    match crate::reaper::observe_live(&state_dir, &tmp_dir, min_age_secs) {
        Ok(report) => {
            if json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
                );
            } else {
                print!("{}", crate::reaper::render_human(&report));
                if let Some(warning) = crate::disk_hygiene::disk_warning_for_display(Path::new("/"))
                {
                    println!("{warning}");
                }
            }
        }
        Err(error) => {
            eprintln!("bridget reaper report: {error}");
            std::process::exit(1);
        }
    }
}

fn cmd_cleanup(args: &[String]) {
    if args.first().map(String::as_str) != Some("--dry-run") {
        eprintln!("usage: bridget cleanup --dry-run");
        eprintln!("Liste les target/ des worktrees déjà mergés — aucune suppression.");
        std::process::exit(2);
    }
    let repo = std::env::current_dir().unwrap_or_else(|error| {
        eprintln!("bridget cleanup: répertoire courant indisponible: {error}");
        std::process::exit(1);
    });
    match crate::disk_hygiene::list_merged_worktree_targets(&repo) {
        Ok(targets) => print!("{}", crate::disk_hygiene::render_cleanup_dry_run(&targets)),
        Err(error) => {
            eprintln!("bridget cleanup: {error}");
            std::process::exit(1);
        }
    }
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
    println!(
        "Build-id daemon: {}",
        status.build_id.as_deref().unwrap_or("inconnu")
    );
    emit_stale_daemon_warning(status.build_id.as_deref());
    emit_disk_warning();
}

fn stale_daemon_warning_for_status(build_id: Option<&str>) -> Option<String> {
    crate::build_info::stale_daemon_warning(build_id.unwrap_or("unknown"))
}

fn emit_stale_daemon_warning(build_id: Option<&str>) {
    if let Some(warning) = stale_daemon_warning_for_status(build_id) {
        eprintln!("{warning}");
    }
}

fn emit_disk_warning() {
    if let Some(warning) = crate::disk_hygiene::disk_warning_for_display(Path::new("/")) {
        eprintln!("{warning}");
    }
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

pub(crate) fn render_ledger(entries: &[LedgerMessage]) -> String {
    if entries.is_empty() {
        return "Ledger vide.\n".to_string();
    }
    let mut rendered = format!("Derniers {} messages :\n", entries.len());
    for entry in entries.iter().rev() {
        let status = entry
            .delivery_status
            .map(|status| format!(" [{}]", status.label_fr()))
            .unwrap_or_default();
        rendered.push_str(&format!(
            "  [{}] {} → {}{}: {}\n",
            entry.ts,
            entry.sender,
            entry.target,
            status,
            entry.body.chars().take(60).collect::<String>()
        ));
    }
    rendered
}

#[cfg(test)]
mod hook_tests {
    use super::*;
    use bridget_transport::protocol::LedgerDeliveryStatus;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::UnixListener;
    use std::path::{Path, PathBuf};
    use std::thread;

    #[test]
    fn repli_cli_prend_le_nom_dans_l_environnement() {
        assert_eq!(
            resolve_cli_agent_name(None, Some("fable-reviewer")),
            "fable-reviewer"
        );
        assert_eq!(
            resolve_cli_agent_name(Some("  "), Some("codex-1")),
            "codex-1"
        );
        assert_eq!(
            resolve_cli_agent_name(Some("renamed"), Some("old")),
            "renamed"
        );
        assert_eq!(resolve_cli_agent_name(None, None), "human");
        assert_eq!(resolve_cli_agent_name(None, Some("")), "human");
    }

    #[test]
    fn refus_type_inconnu_explique_l_instantane_du_daemon() {
        let rendered = display_spawn_refusal(&SpawnRefusal::UnknownType {
            requested_type: "cursor".to_string(),
            known_types: vec!["claude".to_string(), "codex".to_string()],
            registry: "/Users/test/.config/bridget/agents.json".to_string(),
        });

        assert_eq!(
            rendered,
            "type d'agent inconnu 'cursor'. Types connus du daemon : claude, codex. \
             Le registre /Users/test/.config/bridget/agents.json est lu au démarrage du daemon ; après modification, relancez-le."
        );
    }

    #[test]
    fn refus_capacite_nomme_type_modele_et_capacite_absente() {
        assert_eq!(
            display_spawn_refusal(&SpawnRefusal::UnsupportedCapability {
                agent_type: "codex".to_string(),
                model: "gpt-5.6-terra".to_string(),
                capability: "modèle pris en charge par l'adaptateur".to_string(),
            }),
            "lancement refusé pour le type 'codex', modèle 'gpt-5.6-terra': capacité manquante modèle pris en charge par l'adaptateur"
        );
    }

    #[test]
    fn rendu_ledger_cli_reste_octet_pour_octet_stable() {
        let entries = vec![
            LedgerMessage {
                id: "ancien".to_string(),
                ts: 1,
                sender: "alice".to_string(),
                target: "bob".to_string(),
                body: "premier".to_string(),
                delivery_status: None,
            },
            LedgerMessage {
                id: "recent".to_string(),
                ts: 2,
                sender: "bob".to_string(),
                target: "alice".to_string(),
                body: "corps riche $VAR\nintact".to_string(),
                delivery_status: Some(LedgerDeliveryStatus::Recu),
            },
        ];
        assert_eq!(
            render_ledger(&entries),
            "Derniers 2 messages :\n  [2] bob → alice [reçu]: corps riche $VAR\nintact\n  [1] alice → bob: premier\n"
        );
    }

    #[test]
    fn rendu_ledger_distingue_en_vol_et_recu() {
        let en_vol = LedgerMessage {
            id: "a".into(),
            ts: 10,
            sender: "peer-a".into(),
            target: "peer-b".into(),
            body: "collège".into(),
            delivery_status: Some(LedgerDeliveryStatus::EnVol),
        };
        let recu = LedgerMessage {
            id: "b".into(),
            ts: 11,
            sender: "peer-a".into(),
            target: "peer-b".into(),
            body: "collège".into(),
            delivery_status: Some(LedgerDeliveryStatus::Recu),
        };
        let rendered_vol = render_ledger(&[en_vol]);
        let rendered_recu = render_ledger(&[recu]);
        assert_ne!(
            rendered_vol, rendered_recu,
            "dispatching et acked ne doivent pas se rendre pareil"
        );
        assert!(rendered_vol.contains("[en vol]"));
        assert!(rendered_recu.contains("[reçu]"));
        assert!(!rendered_vol.contains("[reçu]"));
        assert!(!rendered_recu.contains("[en vol]"));
    }

    #[test]
    fn requests_defaut_reste_participant_et_all_expose_de_vers() {
        assert_eq!(
            parse_requests_args(&[]).unwrap(),
            RequestsOptions {
                all: false,
                json: false
            }
        );
        assert_eq!(
            parse_requests_args(&["--all".into(), "--json".into()]).unwrap(),
            RequestsOptions {
                all: true,
                json: true
            }
        );
        assert!(parse_requests_args(&["--global".into()]).is_err());

        let requests = vec![
            RequestInfo {
                id: "req-a".to_string(),
                sender: "maicie".to_string(),
                target: "coderBridget".to_string(),
                state: "open".to_string(),
                created_at: 10,
                deadline_at: 100,
                cancel_reason: None,
                deferred_reminder_level: None,
                deferred_reminder_at: None,
            },
            RequestInfo {
                id: "req-b".to_string(),
                sender: "prospective".to_string(),
                target: "reviewer2".to_string(),
                state: "open".to_string(),
                created_at: 11,
                deadline_at: 200,
                cancel_reason: None,
                deferred_reminder_level: Some(1),
                deferred_reminder_at: Some(50),
            },
        ];

        let participant = render_requests(&requests, false);
        let participant_header = participant.lines().next().unwrap();
        assert!(participant_header.contains("DESTINATAIRE"), "{participant}");
        assert!(
            !participant_header
                .split_whitespace()
                .any(|cell| cell == "DE")
        );
        assert!(
            !participant_header
                .split_whitespace()
                .any(|cell| cell == "VERS")
        );
        assert!(participant.contains("coderBridget"));
        assert!(!participant.contains("maicie"));
        assert!(!participant.contains("prospective"));

        let global = render_requests(&requests, true);
        let global_header = global.lines().next().unwrap();
        assert!(global_header.split_whitespace().any(|cell| cell == "DE"));
        assert!(global_header.split_whitespace().any(|cell| cell == "VERS"));
        assert!(!global_header.contains("DESTINATAIRE"));
        assert!(global.contains("maicie") && global.contains("coderBridget"));
        assert!(global.contains("prospective") && global.contains("reviewer2"));
        assert!(global.contains("100") && global.contains("open"));
    }

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
        assert_eq!(
            settings["hooks"]["SessionEnd"],
            original["hooks"]["SessionEnd"]
        );
        assert_eq!(settings["model"], original["model"]);

        assert!(remove_bridget_hook(&mut settings));
        assert_eq!(
            settings, original,
            "le retrait doit rendre le fichier d'origine"
        );
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
    fn hook_exige_l_accord_session_id_et_transcript() {
        use std::path::Path;
        assert!(transcript_matches_hook_session(
            Path::new("/tmp/proj/aaaa-session.jsonl"),
            Some("aaaa-session")
        ));
        assert!(
            !transcript_matches_hook_session(
                Path::new("/tmp/proj/bbbb-voisin.jsonl"),
                Some("aaaa-session")
            ),
            "un transcript voisin ne doit pas passer pour la session du hook"
        );
        // Sans session_id, le chemin fourni par Claude Code reste accepté.
        assert!(transcript_matches_hook_session(
            Path::new("/tmp/proj/bbbb-voisin.jsonl"),
            None
        ));
        assert!(transcript_matches_hook_session(
            Path::new("/tmp/proj/bbbb-voisin.jsonl"),
            Some("")
        ));
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
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "{\"nouveau\":true}\n"
        );
        // Aucun fichier temporaire ne subsiste dans le répertoire.
        let restes = std::fs::read_dir(std::env::temp_dir())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains(".bridget-"))
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
    fn spawn_nomme_sans_persistent_avertit_qu_il_ne_survivra_pas() {
        let parsed = parse_spawn_args(&[
            "cursor".to_string(),
            "--name".to_string(),
            "cursor3".to_string(),
        ])
        .unwrap();
        assert!(!parsed.persistent);
        assert_eq!(parsed.name.as_deref(), Some("cursor3"));
        let warning = crate::recovery_trace::non_persistent_spawn_warning("cursor3");
        assert!(warning.contains("cursor3"));
        assert!(warning.contains("ne survivra pas au redémarrage"));
        assert!(warning.contains("sans --persistent"));
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
        let first =
            resolve_spawn_order(&parse_spawn_args(&args).unwrap(), 100, &cwd, &root).unwrap();
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
        assert!(
            resolve_spawn_order(
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
            .contains("options divergentes")
        );
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
                        definition: Some(bridget_transport::ResolvedAgentDefinition {
                            command: "npx".to_string(),
                            args: vec!["fixture-acp".to_string()],
                            protocol: "acp".to_string(),
                            forbidden_env: vec!["OPENAI_API_KEY".to_string()],
                            pass_env: Vec::new(),
                            permissions: "allow".to_string(),
                            queue_capacity: 32,
                            notify_timeout_secs: 600,
                            mcp: bridget_transport::ResolvedMcpDefinition {
                                interactive: "none".to_string(),
                                acp_session: false,
                            },
                            capabilities: bridget_transport::AdapterCapabilities::default(),
                            digest: "fixture-digest".to_string(),
                        }),
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
                DaemonToWrapper::SpawnAccepted { ref command_id, ref name, .. }
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

    /// Payload StatusLine conforme au schéma publié par Claude Code, avec les
    /// deux fenêtres présentes. Le reste du payload est là pour vérifier qu'on
    /// ne confond pas la barre de contexte avec une limite de forfait.
    fn statusline_payload_complet() -> serde_json::Value {
        serde_json::json!({
            "session_id": "abc",
            "model": {"display_name": "Opus"},
            "context_window": {"used_percentage": 42, "remaining_percentage": 58},
            "rate_limits": {
                "five_hour": {"used_percentage": 19, "resets_at": 1787590200_i64},
                "seven_day": {"used_percentage": 61, "resets_at": 1788136905_i64}
            }
        })
    }

    #[test]
    fn statusline_releve_les_deux_fenetres_attestees() {
        let facts = rate_limit_facts_from_statusline(&statusline_payload_complet());
        assert_eq!(
            facts,
            vec![
                StatusLineLimit {
                    window: "five_hour".to_string(),
                    used_percent: Some(19),
                    resets_at: Some(1787590200),
                },
                StatusLineLimit {
                    window: "seven_day".to_string(),
                    used_percent: Some(61),
                    resets_at: Some(1788136905),
                },
            ]
        );
    }

    /// Le cas exigé : sans bloc `rate_limits`, on ne pousse RIEN. Le schéma
    /// Claude Code le donne absent hors abonnement et avant la première
    /// réponse d'API — la colonne du référent doit rester vide, pas se
    /// remplir d'un zéro inventé.
    #[test]
    fn statusline_sans_bloc_limites_ne_pousse_rien() {
        let sans_limites = serde_json::json!({
            "session_id": "abc",
            "context_window": {"used_percentage": 42}
        });
        assert!(rate_limit_facts_from_statusline(&sans_limites).is_empty());
    }

    /// La barre de contexte porte AUSSI un `used_percentage`, à un autre
    /// endroit et pour une autre grandeur. Le confondre avec la consommation
    /// de forfait afficherait le remplissage du contexte comme un quota.
    #[test]
    fn statusline_ne_prend_pas_le_contexte_pour_une_limite() {
        let contexte_seul = serde_json::json!({
            "context_window": {"used_percentage": 97, "remaining_percentage": 3}
        });
        assert!(rate_limit_facts_from_statusline(&contexte_seul).is_empty());
    }

    #[test]
    fn statusline_garde_la_fenetre_presente_et_ignore_l_absente() {
        let une_seule = serde_json::json!({
            "rate_limits": {"five_hour": {"used_percentage": 5, "resets_at": 1787590200_i64}}
        });
        let facts = rate_limit_facts_from_statusline(&une_seule);
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].window, "five_hour");
    }

    #[test]
    fn statusline_tronque_le_pourcentage_fractionnaire() {
        let fractionnaire = serde_json::json!({
            "rate_limits": {"five_hour": {"used_percentage": 19.87, "resets_at": 1787590200_i64}}
        });
        assert_eq!(
            rate_limit_facts_from_statusline(&fractionnaire)[0].used_percent,
            Some(19)
        );
    }

    /// Hors domaine ou non numérique : on n'invente pas, et on ne rabat pas
    /// dans les bornes. La fenêtre survit par son `resets_at` attesté, sans
    /// pourcentage — le daemon aurait refusé un 150 %.
    #[test]
    fn statusline_ecarte_un_pourcentage_hors_domaine_sans_le_rabattre() {
        let aberrant = serde_json::json!({
            "rate_limits": {
                "five_hour": {"used_percentage": 150, "resets_at": 1787590200_i64},
                "seven_day": {"used_percentage": "beaucoup", "resets_at": 1788136905_i64}
            }
        });
        let facts = rate_limit_facts_from_statusline(&aberrant);
        assert_eq!(facts.len(), 2);
        assert!(facts.iter().all(|fact| fact.used_percent.is_none()));
        assert!(facts.iter().all(|fact| fact.resets_at.is_some()));
    }

    /// Un instant de retour nul ou négatif serait refusé par le daemon : le
    /// hook ne le fabrique pas en `None` silencieux d'une fenêtre par ailleurs
    /// vide — la fenêtre disparaît entièrement.
    #[test]
    fn statusline_ne_pousse_pas_une_fenetre_sans_aucune_valeur() {
        let vides = serde_json::json!({
            "rate_limits": {
                "five_hour": {},
                "seven_day": {"resets_at": 0}
            }
        });
        assert!(rate_limit_facts_from_statusline(&vides).is_empty());
    }

    /// Le statut n'est pas attesté par ce payload : il ne doit jamais valoir
    /// « allowed » (verdict inventé) ni « rejected » (qui ferait afficher
    /// « épuisée » sur une fenêtre saine).
    #[test]
    fn statusline_n_invente_aucun_verdict_de_statut() {
        assert_eq!(STATUSLINE_LIMIT_STATUS, "unknown");
        assert_ne!(STATUSLINE_LIMIT_STATUS, "allowed");
        assert_ne!(STATUSLINE_LIMIT_STATUS, "rejected");
    }

    /// Jonction : ce que le hook relève doit remplir la colonne du référent.
    ///
    /// Relie l'extraction au rendu réel de `who`, avec le statut non attesté
    /// tel qu'il sera posé. Sans cet oracle, l'extraction pourrait être juste
    /// et la colonne rester vide — c'est la colonne qui est la mission.
    #[test]
    fn statusline_remplit_la_colonne_du_referent() {
        let facts = rate_limit_facts_from_statusline(&statusline_payload_complet());
        let agent = AgentInfo {
            name: "bridget".to_string(),
            agent_type: "claude".to_string(),
            connection_id: "conn-referent".to_string(),
            host: "local".to_string(),
            transport: "unix".to_string(),
            channel: None,
            mode: Some(PresenceMode::Tmux),
            location: None,
            os: "macOS".to_string(),
            state: "connected".to_string(),
            last_seen_secs: 0,
            reconnect_count: 0,
            domain: None,
            model: Some("claude-fable-5".to_string()),
            effort: None,
            rate_limits: facts
                .iter()
                .map(|fact| bridget_transport::protocol::RateLimitFact {
                    window: fact.window.clone(),
                    status: STATUSLINE_LIMIT_STATUS.to_string(),
                    resets_at: fact.resets_at,
                    used_percent: fact.used_percent,
                })
                .collect(),
            model_mismatch: None,
        };
        let rendered = format_rate_limit(&agent);
        assert!(rendered.contains("5h 19% rst "), "{rendered}");
        assert!(rendered.contains("7d 61% rst "), "{rendered}");
        // Le statut non attesté ne doit pas fuir à l'écran.
        assert!(!rendered.contains("unknown"), "{rendered}");
        assert!(!rendered.contains("épuisée"), "{rendered}");
        assert_ne!(
            rendered, "—",
            "la colonne du référent ne doit plus être vide"
        );
    }

    /// ORACLE PRINCIPAL DU CORRECTIF : le hook n'émet AUCUN `Register`.
    ///
    /// La propriété se mesure à LA SOURCE — les trames émises — et non par une
    /// inspection de l'annuaire après la poussée. Ce choix n'est pas un confort
    /// de test, c'est une nécessité démontrée : à la fermeture d'une connexion
    /// le daemon appelle `router.unregister_by_conn`, si bien qu'une présence
    /// transitoire a TOUJOURS disparu au moment où on interroge `ListAgents`.
    /// Un oracle tardif passe donc au vert même avec le `Register` remis en
    /// place — vérifié : le mutant survivait. C'est exactement le piège qui a
    /// rendu le constat d'origine difficile à saisir (« observée comme agent
    /// LIBRE, disparue avant inspection »).
    ///
    /// `Register` étant le seul chemin vers `router.register` hors tests, ne
    /// pas l'émettre suffit à ne jamais peupler l'annuaire.
    #[test]
    fn statusline_n_emet_aucune_declaration_de_presence() {
        let unique = format!("{}-{}", std::process::id(), line!());
        let socket = std::env::temp_dir().join(format!("bridget-statusline-{unique}.sock"));
        std::fs::remove_file(&socket).ok();
        let listener = UnixListener::bind(&socket).unwrap();

        // Faux daemon : il accuse tout, et retient ce qu'on lui a dit.
        let recu = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let journal = std::sync::Arc::clone(&recu);
        let serveur = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut writer = BufWriter::new(stream.try_clone().unwrap());
            let reader = BufReader::new(stream);
            for line in reader.lines() {
                let Ok(line) = line else { break };
                journal.lock().unwrap().push(line);
                let ack = DaemonToWrapper::Ack {
                    id: "rate-limit".to_string(),
                };
                if writeln!(writer, "{}", encode(&ack).unwrap()).is_err() || writer.flush().is_err()
                {
                    break;
                }
            }
        });

        let facts = rate_limit_facts_from_statusline(&statusline_payload_complet());
        send_rate_limits_to_daemon(&socket, "referent-oracle", &facts).unwrap();
        serveur.join().unwrap();
        std::fs::remove_file(&socket).ok();

        let trames = recu.lock().unwrap().clone();
        let declarations: Vec<_> = trames
            .iter()
            .filter(|trame| {
                matches!(
                    decode::<WrapperToDaemon>(trame.trim()),
                    Ok(WrapperToDaemon::Register { .. })
                )
            })
            .collect();
        assert!(
            declarations.is_empty(),
            "le hook s'est déclaré à l'annuaire : {declarations:?}"
        );

        // Et il n'a rien émis d'autre que ses deux faits, attribués au référent.
        let fenetres: Vec<_> = trames
            .iter()
            .map(|trame| decode::<WrapperToDaemon>(trame.trim()).unwrap())
            .map(|message| match message {
                WrapperToDaemon::RateLimit { agent, window, .. } => (agent, window),
                other => panic!("trame inattendue émise par le hook : {other:?}"),
            })
            .collect();
        assert_eq!(
            fenetres,
            vec![
                ("referent-oracle".to_string(), "five_hour".to_string()),
                ("referent-oracle".to_string(), "seven_day".to_string()),
            ],
            "le hook doit émettre ses deux faits, et rien de plus"
        );
    }

    /// ORACLE D'ATTRIBUTION, contre un daemon jetable : la poussée sans
    /// déclaration remplit bel et bien la colonne du référent.
    ///
    /// Complément indispensable du précédent : supprimer le `Register` sans
    /// vérifier l'attribution échangerait un annuaire bruyant contre une
    /// colonne muette. On observe par la projection publique `ListAgents`,
    /// celle-là même que lisent les rondes.
    ///
    /// Le daemon est jetable : socket et base sous `TMPDIR`, uniques, détruits
    /// en fin de test. Jamais la production.
    #[test]
    fn statusline_attribue_ses_faits_sans_se_declarer() {
        let unique = format!("{}-{}", std::process::id(), line!());
        let socket = std::env::temp_dir().join(format!("bridget-statusline-{unique}.sock"));
        let db_path = std::env::temp_dir().join(format!("bridget-statusline-{unique}.db"));
        let config = crate::daemon::DaemonConfig {
            socket_path: socket.clone(),
            db_path: db_path.clone(),
            log_path: std::env::temp_dir().join(format!("bridget-statusline-{unique}.log")),
            ..Default::default()
        };
        let daemon_socket = socket.clone();
        thread::spawn(move || {
            let _ = crate::daemon::run(config);
        });
        for _ in 0..100 {
            if daemon_socket.exists() {
                break;
            }
            thread::sleep(std::time::Duration::from_millis(50));
        }
        assert!(socket.exists(), "daemon jetable non prêt");

        // Le référent, lui, est un vrai agent : il se déclare. C'est la cible
        // des faits, et le seul nom qui a le droit d'être à l'annuaire.
        let referent = UnixStream::connect(&socket).unwrap();
        let mut referent_writer = BufWriter::new(referent.try_clone().unwrap());
        let mut referent_reader = BufReader::new(referent);
        let register = WrapperToDaemon::Register {
            agent_type: "claude".to_string(),
            name: Some("referent-oracle".to_string()),
            host: None,
            transport: None,
            channel: None.into(),
            mode: Some(PresenceMode::Tmux),
            location: None,
            os: None,
            // Indispensable, et pas un détail de fixture : une présence est
            // indexée par instance, et `presence_of_agent` remonte
            // agent → connexion → instance. Sans `instance_id`, le référent
            // serait à l'annuaire mais sans présence — donc sans colonne à
            // remplir, et l'oracle mesurerait autre chose que la mission.
            instance_id: Some("instance-referent-oracle".to_string()),
            domain: None,
            turn_in_progress: false,
            journal_available: None,
        };
        writeln!(referent_writer, "{}", encode(&register).unwrap()).unwrap();
        referent_writer.flush().unwrap();
        let mut line = String::new();
        referent_reader.read_line(&mut line).unwrap();
        assert!(
            matches!(
                decode(line.trim()).unwrap(),
                DaemonToWrapper::Registered { .. }
            ),
            "le référent doit être enregistré : {line}"
        );

        // LA POUSSÉE, par le chemin de production exact.
        let facts = rate_limit_facts_from_statusline(&statusline_payload_complet());
        assert_eq!(facts.len(), 2, "le payload d'essai porte deux fenêtres");
        send_rate_limits_to_daemon(&socket, "referent-oracle", &facts)
            .expect("la poussée doit aboutir sans se déclarer");

        let agents = match ask_agent_list(&socket) {
            DaemonToWrapper::AgentList { agents } => agents,
            other => panic!("AgentList attendu, reçu {other:?}"),
        };

        // On n'assert PAS ici l'absence de `cli-statusline-*` : la connexion
        // du hook est déjà refermée, donc déjà désenregistrée. L'assertion
        // serait vraie quoi qu'il arrive, y compris avec le défaut. Cette
        // propriété-là se prouve à la source, dans l'oracle précédent.
        let referent_info = agents
            .iter()
            .find(|agent| agent.name == "referent-oracle")
            .expect("le référent doit rester à l'annuaire");
        let mut windows: Vec<_> = referent_info
            .rate_limits
            .iter()
            .map(|fact| (fact.window.as_str(), fact.used_percent))
            .collect();
        windows.sort_unstable();
        assert_eq!(
            windows,
            vec![("five_hour", Some(19)), ("seven_day", Some(61))],
            "les faits poussés doivent être attribués au référent"
        );
        assert!(
            referent_info
                .rate_limits
                .iter()
                .all(|fact| fact.status == STATUSLINE_LIMIT_STATUS),
            "le statut non attesté doit être conservé tel quel"
        );

        std::fs::remove_file(&socket).ok();
        std::fs::remove_file(&db_path).ok();
    }

    /// Interroge la projection publique de l'annuaire, sans se déclarer non
    /// plus : l'observateur ne doit pas fausser ce qu'il mesure.
    fn ask_agent_list(socket: &Path) -> DaemonToWrapper {
        let stream = UnixStream::connect(socket).unwrap();
        let mut writer = BufWriter::new(stream.try_clone().unwrap());
        let mut reader = BufReader::new(stream);
        writeln!(writer, "{}", encode(&WrapperToDaemon::ListAgents).unwrap()).unwrap();
        writer.flush().unwrap();
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        decode(line.trim()).unwrap()
    }
}

#[cfg(test)]
mod idempotency_projection_tests {
    use super::*;
    use rusqlite::params;
    use std::os::unix::net::UnixListener;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SOCKET_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temporary_socket_path() -> std::path::PathBuf {
        let counter = SOCKET_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "bridget-t1208-{}-{counter}.sock",
            std::process::id()
        ))
    }

    fn temporary_database_path() -> std::path::PathBuf {
        let counter = SOCKET_COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("bridget-t1208-{}-{counter}.db", std::process::id()))
    }

    fn start_real_daemon() -> (std::path::PathBuf, std::path::PathBuf) {
        let socket_path = temporary_socket_path();
        let db_path = temporary_database_path();
        let _ = std::fs::remove_file(&socket_path);
        let _ = std::fs::remove_file(&db_path);
        let config = DaemonConfig {
            socket_path: socket_path.clone(),
            db_path: db_path.clone(),
            log_path: db_path.with_extension("log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        };
        std::thread::spawn(move || {
            let _ = daemon::run(config);
        });
        for _ in 0..100 {
            if socket_path.exists() {
                return (socket_path, db_path);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("daemon réel T1208 non démarré");
    }

    fn stored_canonical_bytes(
        db_path: &std::path::Path,
        options: &IdempotentSendOptions,
    ) -> Vec<u8> {
        let connection = rusqlite::Connection::open(db_path).unwrap();
        connection
            .query_row(
                "SELECT canonical_bytes FROM idempotency_records
                 WHERE issuer_scope = ?1 AND operation_kind = 'send' AND idempotency_key = ?2",
                params![options.issuer_scope, options.id],
                |row| row.get(0),
            )
            .unwrap()
    }

    fn idempotency_issue(response: DaemonToWrapper) -> IdempotencyIssue {
        match response {
            DaemonToWrapper::IdempotencyResult { issue, .. } => issue,
            unexpected => panic!("issue idempotente attendue : {unexpected:?}"),
        }
    }

    fn write_response(writer: &mut BufWriter<UnixStream>, response: DaemonToWrapper) {
        writeln!(writer, "{}", encode(&response).unwrap()).unwrap();
        writer.flush().unwrap();
    }

    fn start_reference_server(
        path: std::path::PathBuf,
        expected_connections: usize,
    ) -> std::thread::JoinHandle<Vec<String>> {
        let listener = UnixListener::bind(&path).unwrap();
        std::thread::spawn(move || {
            let mut sends = Vec::new();
            let mut first_send: Option<String> = None;
            for _ in 0..expected_connections {
                let (stream, _) = listener.accept().unwrap();
                let read_stream = stream.try_clone().unwrap();
                let mut reader = BufReader::new(read_stream);
                let mut writer = BufWriter::new(stream);

                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                assert!(matches!(
                    decode::<WrapperToDaemon>(line.trim_end()).unwrap(),
                    WrapperToDaemon::RoleHandshake {
                        role: ConnectionRole::Client
                    }
                ));
                write_response(
                    &mut writer,
                    DaemonToWrapper::RoleAccepted {
                        role: ConnectionRole::Client,
                    },
                );

                line.clear();
                reader.read_line(&mut line).unwrap();
                assert!(matches!(
                    decode::<WrapperToDaemon>(line.trim_end()).unwrap(),
                    WrapperToDaemon::ClientHello {
                        contract_version: CLIENT_CONTRACT_VERSION,
                        capabilities,
                        ..
                    } if capabilities == vec![ClientCapability::SendIdempotent]
                ));
                write_response(
                    &mut writer,
                    DaemonToWrapper::ClientWelcome {
                        version: CLIENT_CONTRACT_VERSION,
                        build_id: "test-build".to_string(),
                        horizon_secs: 300,
                        issued_at_tolerance_secs: 5,
                        capabilities: vec![ClientCapability::SendIdempotent],
                    },
                );

                line.clear();
                reader.read_line(&mut line).unwrap();
                let command: WrapperToDaemon = decode(line.trim_end()).unwrap();
                let serialized = encode(&command).unwrap();
                let issue = if first_send
                    .as_ref()
                    .is_some_and(|first| first != &serialized)
                {
                    IdempotencyIssue::EnvelopeMismatch
                } else {
                    first_send.get_or_insert_with(|| serialized.clone());
                    IdempotencyIssue::Accepted {
                        expires_at: 123_456,
                    }
                };
                sends.push(serialized);
                write_response(
                    &mut writer,
                    DaemonToWrapper::IdempotencyResult {
                        operation_kind: "send".to_string(),
                        idempotency_key: "message-t1208".to_string(),
                        issue,
                    },
                );
            }
            std::fs::remove_file(path).unwrap();
            sends
        })
    }

    /// Client de référence volontairement indépendant de la projection CLI :
    /// il déroule les trois étapes publiées du contrat sur le socket.
    fn reference_client_send(
        path: &std::path::Path,
        message: &BridgetMessage,
        options: &IdempotentSendOptions,
    ) -> DaemonToWrapper {
        let stream = UnixStream::connect(path).unwrap();
        let read_stream = stream.try_clone().unwrap();
        let mut writer = BufWriter::new(stream);
        let mut reader = BufReader::new(read_stream);

        write_control_message(
            &mut writer,
            &WrapperToDaemon::RoleHandshake {
                role: ConnectionRole::Client,
            },
        )
        .unwrap();
        assert!(matches!(
            read_control_message(&mut reader).unwrap(),
            DaemonToWrapper::RoleAccepted {
                role: ConnectionRole::Client
            }
        ));
        write_control_message(
            &mut writer,
            &WrapperToDaemon::ClientHello {
                contract_version: CLIENT_CONTRACT_VERSION,
                issuer_scope: options.issuer_scope.clone(),
                capabilities: vec![ClientCapability::SendIdempotent],
            },
        )
        .unwrap();
        assert!(matches!(
            read_control_message(&mut reader).unwrap(),
            DaemonToWrapper::ClientWelcome { .. }
        ));
        write_control_message(
            &mut writer,
            &WrapperToDaemon::SendIdempotent {
                message: message.clone(),
                message_id: options.id.clone(),
                issued_at: options.issued_at,
            },
        )
        .unwrap();
        read_control_message(&mut reader).unwrap()
    }

    fn example_options() -> IdempotentSendOptions {
        IdempotentSendOptions {
            id: "message-t1208".to_string(),
            issued_at: 123_000,
            issuer_scope: "012_scope_aaaaaaaaaaaa".to_string(),
        }
    }

    #[test]
    fn options_idempotentes_sont_obligatoires_ensemble() {
        assert_eq!(idempotent_options(None, None, None).unwrap(), None);
        for (id, issued_at, issuer_scope) in [
            (Some("id".to_string()), None, None),
            (None, Some("123".to_string()), None),
            (None, None, Some("scope".to_string())),
            (Some("id".to_string()), Some("123".to_string()), None),
            (Some("id".to_string()), None, Some("scope".to_string())),
            (None, Some("123".to_string()), Some("scope".to_string())),
        ] {
            assert!(idempotent_options(id, issued_at, issuer_scope).is_err());
        }
        assert!(
            idempotent_options(
                Some("id".to_string()),
                Some("pas-un-instant".to_string()),
                Some("scope".to_string())
            )
            .is_err()
        );
    }

    #[test]
    fn depot_guichet_rejoue_le_canon_ferme_avec_les_trois_cles() {
        let args = vec![
            "deposer",
            "delivery-report",
            "--from",
            "codex-1",
            "--objective",
            "objective-1",
            "--delegation",
            "delegation-1",
            "--hash",
            "aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899",
            "--in-reply-to",
            "message-1",
            "--id",
            "deposit-1",
            "--issued-at",
            "1787500000",
            "--issuer-scope",
            "015_scope_0123456789abcdef0123456789abcdef",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        let first = parse_guichet_deposit(&args).unwrap();
        let second = parse_guichet_deposit(&args).unwrap();
        assert_eq!(encode(&first).unwrap(), encode(&second).unwrap());
        assert!(matches!(
            first,
            WrapperToDaemon::ServiceRequest {
                version: SERVICE_CONTRACT_VERSION,
                request_id,
                issued_at: 1_787_500_000,
                operation: ServiceRequestOperation::DeliveryReport,
                payload: ServiceRequestPayload::DeliveryReport { in_reply_to, .. },
                ..
            } if request_id == "deposit-1" && in_reply_to == "message-1"
        ));
    }

    #[test]
    fn depot_guichet_refuse_les_formes_ouvertes_ou_incompletes() {
        let hash = "0".repeat(64);
        let missing_link = vec![
            "deposer",
            "delivery-report",
            "--from",
            "codex-1",
            "--objective",
            "objective-1",
            "--delegation",
            "delegation-1",
            "--hash",
            hash.as_str(),
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        assert!(parse_guichet_deposit(&missing_link).is_err());
        let free_form = vec!["deposer", "message-libre", "--from", "codex-1"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        assert!(parse_guichet_deposit(&free_form).is_err());
    }

    #[test]
    fn projection_cli_et_client_reference_produisent_le_meme_envoi_et_le_meme_rejet() {
        let path = temporary_socket_path();
        let server = start_reference_server(path.clone(), 3);
        let options = example_options();
        let mut message = BridgetMessage::new("human", "codex-1", "bonjour");
        message.id = options.id.clone();

        let reference = reference_client_send(&path, &message, &options);
        let cli = send_idempotent_to_daemon_at(&path, &message, &options).unwrap();
        assert!(matches!(
            (reference, cli),
            (
                DaemonToWrapper::IdempotencyResult {
                    issue: IdempotencyIssue::Accepted { .. },
                    ..
                },
                DaemonToWrapper::IdempotencyResult {
                    issue: IdempotencyIssue::Accepted { .. },
                    ..
                }
            )
        ));

        let mut divergent = message.clone();
        divergent.body = "message différent".to_string();
        assert!(matches!(
            send_idempotent_to_daemon_at(&path, &divergent, &options).unwrap(),
            DaemonToWrapper::IdempotencyResult {
                issue: IdempotencyIssue::EnvelopeMismatch,
                ..
            }
        ));

        let sends = server.join().unwrap();
        assert_eq!(sends[0], sends[1], "le canon publié est identique");
        assert_ne!(sends[0], sends[2]);
        assert!(sends[0].contains("bonjour"));
        assert!(sends[2].contains("message différent"));
    }

    #[test]
    fn projection_cli_et_reference_partagent_le_canon_du_daemon_reel() {
        let (socket_path, db_path) = start_real_daemon();
        let issued_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let options = IdempotentSendOptions {
            id: "message-t1208-reel".to_string(),
            issued_at,
            issuer_scope: "012_scope_aaaaaaaaaaaa".to_string(),
        };
        let mut message = BridgetMessage::new("human", "destinataire-absent", "bonjour");
        message.id = options.id.clone();
        message.reply = true;
        message.reply_timeout = Some(12);

        let reference = idempotency_issue(reference_client_send(&socket_path, &message, &options));
        let cli = idempotency_issue(
            send_idempotent_to_daemon_at(&socket_path, &message, &options).unwrap(),
        );
        assert_eq!(reference, cli, "référence et CLI rejouent la même issue");
        let canonical = stored_canonical_bytes(&db_path, &options);

        let mut divergences = Vec::new();
        let mut body = message.clone();
        body.body = "bonjour divergent".to_string();
        divergences.push(body);
        let mut target = message.clone();
        target.to = "autre-destinataire".to_string();
        divergences.push(target);
        let mut reply = message.clone();
        reply.reply = false;
        divergences.push(reply);
        let mut deadline = message.clone();
        deadline.deadline_at = Some((issued_at + 13) as u64);
        divergences.push(deadline);

        for divergent in divergences {
            assert_eq!(
                idempotency_issue(
                    send_idempotent_to_daemon_at(&socket_path, &divergent, &options).unwrap(),
                ),
                IdempotencyIssue::EnvelopeMismatch,
            );
            assert_eq!(
                stored_canonical_bytes(&db_path, &options),
                canonical,
                "EnvelopeMismatch ne doit jamais modifier le record initial",
            );
        }
    }

    #[test]
    fn who_affiche_mode_et_localisation_sans_dependre_d_un_tty() {
        let agent = |name: &str, mode: Option<PresenceMode>, location: Option<&str>| AgentInfo {
            name: name.to_string(),
            agent_type: "fixture".to_string(),
            connection_id: format!("conn-{name}"),
            host: "local".to_string(),
            transport: "tmux".to_string(),
            channel: Some("ssh-unix".to_string()),
            mode,
            location: location.map(str::to_string),
            os: "macOS".to_string(),
            state: "connected".to_string(),
            last_seen_secs: 0,
            reconnect_count: 0,
            domain: None,
            model: None,
            effort: None,
            rate_limits: vec![],
            model_mismatch: None,
        };
        let rendered = render_who(
            &[
                agent("acp-gere", Some(PresenceMode::Acp), None),
                agent(
                    "tmux-interactif",
                    Some(PresenceMode::Tmux),
                    Some("bridget:4.2"),
                ),
                agent("cli-ephemere", Some(PresenceMode::Cli), None),
            ],
            None,
        );

        assert!(rendered.starts_with("Agents connectés :\n"));
        assert!(rendered.contains("MODE"));
        assert!(rendered.contains("CANAL"));
        assert!(rendered.contains("LOCALISATION"));
        assert!(rendered.contains("LIMITE"));
        assert!(rendered.contains("acp-gere"));
        assert!(rendered.contains("tmux-interactif"));
        assert!(rendered.contains("cli-ephemere"));
        assert!(rendered.contains("bridget:4.2"));
        assert!(rendered.contains("acp"));
        assert!(rendered.contains("tmux"));
        assert!(rendered.contains("ssh-unix"));
        assert!(rendered.contains("cli"));
        assert!(!rendered.contains('\u{1b}'));
    }

    #[test]
    fn who_rend_une_limite_compacte_par_fenetre_et_garde_l_absence() {
        let mut agent = AgentInfo {
            name: "claude-1".to_string(),
            agent_type: "claude".to_string(),
            connection_id: "conn-claude".to_string(),
            host: "local".to_string(),
            transport: "stdio".to_string(),
            channel: None,
            mode: Some(PresenceMode::Cli),
            location: None,
            os: "macOS".to_string(),
            state: "connected".to_string(),
            last_seen_secs: 0,
            reconnect_count: 0,
            domain: None,
            model: Some("claude-opus-5".to_string()),
            effort: None,
            rate_limits: vec![bridget_transport::protocol::RateLimitFact {
                window: "five_hour".to_string(),
                status: "allowed".to_string(),
                resets_at: Some(1_787_572_200),
                used_percent: Some(19),
            }],
            model_mismatch: None,
        };
        let rendered = format_rate_limit(&agent);
        assert!(rendered.starts_with("5h 19% rst "), "{rendered}");
        assert!(!rendered.contains("seven_day"), "fenêtre absente = absente");

        agent
            .rate_limits
            .push(bridget_transport::protocol::RateLimitFact {
                window: "seven_day".to_string(),
                status: "allowed".to_string(),
                resets_at: Some(1_787_700_000),
                used_percent: Some(61),
            });
        let both = format_rate_limit(&agent);
        assert!(both.contains("5h 19% rst "), "{both}");
        assert!(both.contains(" · "), "{both}");
        assert!(both.contains("7d 61% rst "), "{both}");

        agent.rate_limits = vec![bridget_transport::protocol::RateLimitFact {
            window: "five_hour".to_string(),
            status: "rejected".to_string(),
            resets_at: None,
            used_percent: None,
        }];
        assert_eq!(format_rate_limit(&agent), "5h épuisée");

        agent.rate_limits.clear();
        assert_eq!(format_rate_limit(&agent), "—");

        assert_eq!(abbreviate_window("primary/300m"), "5h");
        assert_eq!(abbreviate_window("secondary/10080m"), "7d");
        assert_eq!(abbreviate_window("exotic_quota_xyz"), "exoticquotax");
        // Faux-ami : contains("five_hour") aurait menti ; frontière de jeton non.
        assert_ne!(abbreviate_window("not_five_hour_custom"), "5h");
        assert_eq!(
            abbreviate_window("not_five_hour_custom"),
            shorten_raw_window("not_five_hour_custom")
        );
        assert_eq!(abbreviate_window("five_hour"), "5h");
        assert_eq!(abbreviate_window("x/five_hour"), "5h");
    }

    #[test]
    fn who_marque_l_ecart_de_modele_et_reste_muet_sans_signal() {
        let mut agent = AgentInfo {
            name: "claude-1".to_string(),
            agent_type: "claude".to_string(),
            connection_id: "conn-claude".to_string(),
            host: "local".to_string(),
            transport: "stdio".to_string(),
            channel: None,
            mode: Some(PresenceMode::Cli),
            location: None,
            os: "macOS".to_string(),
            state: "connected".to_string(),
            last_seen_secs: 0,
            reconnect_count: 0,
            domain: None,
            model: Some("claude-opus-5".to_string()),
            effort: None,
            rate_limits: vec![],
            model_mismatch: None,
        };
        assert_eq!(format_model(&agent), "claude-opus-5");
        agent.model_mismatch = Some(bridget_transport::protocol::ModelMismatchFact {
            pinned: "claude-opus-5".to_string(),
            served: "claude-opus-4-6".to_string(),
        });
        let rendered = render_who(std::slice::from_ref(&agent), None);
        assert!(rendered.contains("claude-opus-4-6 ≠ claude-opus-5"));
        agent.model = None;
        agent.model_mismatch = None;
        assert_eq!(format_model(&agent), "—");
    }

    #[test]
    fn who_et_status_signalent_exactement_un_daemon_perime() {
        // Observe les identifiants et la remédiation structurelle — pas un
        // libellé humain (le gate fondateur a déjà payé ce piège cette nuit).
        assert!(stale_daemon_warning_for_status(Some(crate::build_info::BUILD_ID)).is_none());
        let warning = stale_daemon_warning_for_status(Some("daemon-ancien")).unwrap();
        assert!(warning.contains("daemon-ancien"));
        assert!(warning.contains(crate::build_info::BUILD_ID));
        let remediation = format!("gui/{}/com.bridget.daemon", unsafe { libc::getuid() });
        assert!(warning.contains(&remediation));
        assert!(
            stale_daemon_warning_for_status(None)
                .unwrap()
                .contains(&remediation)
        );
    }
}

#[cfg(test)]
mod depot_tests {
    use super::*;

    /// Le cas nominal : le daemon a pris la remise mais le destinataire n'a pas
    /// encore accusé. C'est un succès de dépôt, pas une panne — c'est ce que le
    /// `rc=1` d'origine niait sur CHAQUE premier envoi.
    #[test]
    fn une_remise_en_vol_est_un_depot_reussi() {
        assert!(send_deposited(&IdempotencyIssue::OutcomeUnknown {
            expires_at: 1_700_000_060,
            delivery_id: Some("livraison-1".to_string()),
        }));
        assert!(send_deposited(&IdempotencyIssue::Accepted {
            expires_at: 1_700_000_060,
        }));
    }

    /// La contre-épreuve : sans `delivery_id`, le sort est réellement inconnu.
    /// Si cette assertion tombe, le correctif est allé trop loin et masque un
    /// échec véritable derrière un code de sortie nul.
    #[test]
    fn un_sort_indetermine_n_est_pas_un_depot() {
        assert!(!send_deposited(&IdempotencyIssue::OutcomeUnknown {
            expires_at: 1_700_000_060,
            delivery_id: None,
        }));
    }

    /// C5 — un identifiant de remise vide n'est pas une preuve de dépôt.
    ///
    /// Le daemon n'en produit jamais et refuserait celui-ci ; l'accepter
    /// ferait sortir en succès sur une valeur qu'il rejette lui-même. La garde
    /// porte sur le contenu, pas sur la seule présence du `Some`.
    #[test]
    fn un_identifiant_de_remise_vide_n_atteste_pas_un_depot() {
        for vide in ["", " ", "\t", "\n"] {
            assert!(
                !send_deposited(&IdempotencyIssue::OutcomeUnknown {
                    expires_at: 1_700_000_060,
                    delivery_id: Some(vide.to_string()),
                }),
                "un delivery_id {vide:?} ne prouve aucune remise"
            );
        }
        // Contre-épreuve : un identifiant réel reste un dépôt, sinon la garde
        // aurait simplement tout refusé.
        assert!(send_deposited(&IdempotencyIssue::OutcomeUnknown {
            expires_at: 1_700_000_060,
            delivery_id: Some("livraison-2".to_string()),
        }));
    }

    /// Les refus restent des échecs : le correctif ne touche qu'à l'issue qui
    /// décrivait un succès comme une perte.
    #[test]
    fn les_refus_restent_des_echecs() {
        assert!(!send_deposited(&IdempotencyIssue::Rejected {
            category: "dnd".to_string(),
            reason: "destinataire en ne-pas-déranger".to_string(),
            expires_at: 1_700_000_060,
        }));
        assert!(!send_deposited(&IdempotencyIssue::EnvelopeMismatch));
        assert!(!send_deposited(&IdempotencyIssue::IdempotencyExpired));
        assert!(!send_deposited(&IdempotencyIssue::InvalidIssuedAt));
    }
}
