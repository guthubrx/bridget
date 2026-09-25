//! CLI — point d'entrée unifié pour toutes les sous-commandes bridget.

use crate::communication::client::DaemonConnection;
use crate::daemon::{self, DaemonConfig};
use bridget_core::{
    BridgetMessage,
    router::{validate_agent_id, validate_technical_label},
};
use bridget_transport::protocol::{
    AdoptStoppedOutcome, AgentInfo, AttachWindow, CLIENT_CONTRACT_VERSION, ClientCapability,
    ConnectionRole, DecommissionOutcome, GuichetDurationClass, IdempotencyIssue, LedgerMessage,
    LedgerScope, PresenceMode, RelaunchOutcome, RequestInfo, ReviewTarget, ReviewVerdict,
    ReviewVerdictEvidence, RuntimeSource, ServiceRequestOperation, ServiceRequestPayload,
    ServiceSuiteDeclaration, decode, encode, is_canonical_git_sha,
};
use bridget_transport::{DaemonToWrapper, SpawnRefusal, StopOutcome, WrapperToDaemon};
use std::fmt::Write as _;
use std::io::Write;
#[cfg(test)]
use std::io::{BufRead, BufReader, BufWriter};
#[cfg(test)]
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

fn unknown_argument(command: &str, argument: &str) -> String {
    format!("{command}: argument inconnu: {argument}")
}

/// Les commandes sans option doivent refuser le premier surplus avant tout
/// effet de bord. Les wrappers et les corps libres de `send`/`reply` ne font
/// volontairement pas partie de cette grammaire fermée.
fn validate_zero_arity_command(command: &str, args: &[String]) -> Result<(), String> {
    if matches!(
        command,
        "mcp" | "discover" | "status" | "version" | "--version" | "-v" | "help" | "--help" | "-h"
    ) && let Some(argument) = args.first()
    {
        return Err(unknown_argument(command, argument));
    }
    Ok(())
}

fn exit_argument_error(error: &str) -> ! {
    eprintln!("bridget {error}");
    std::process::exit(2);
}

fn validate_communication_entry(command: &str, args: &[String]) -> Result<(), String> {
    if command == "cleanup" {
        return Err("cleanup : inventaire des worktrees retiré du noyau de communication".into());
    }
    if matches!(
        command,
        "managed-runtime-wrapper" | "managed-runtime-stop" | "project-runtime" | "project-round"
    ) {
        return Err(format!(
            "{command} : runtime de projet retiré du noyau de communication ; aucun repli sur l’hôte"
        ));
    }
    if command == "daemon"
        && let Some(argument) = args.first()
    {
        return Err(unknown_argument(command, argument));
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

    // Avant namespace, résolution du registre et lancement : une invocation
    // conteneur historique ne devient jamais une commande hôte.
    if let Err(error) = validate_communication_entry(cmd, &args[2..]) {
        exit_argument_error(&error);
    }

    // Refus avant même l'ouverture du namespace : une ancienne commande UI
    // ne doit ni lancer un programme homonyme ni relire un endpoint historique.
    if cmd == "ui" {
        exit_argument_error(
            "ui : interface retirée du noyau de communication ; utiliser who, ledger et attach",
        );
    }

    if !matches!(
        cmd.as_str(),
        "federate" | "version" | "--version" | "-v" | "help" | "--help" | "-h"
    ) && let Err(error) = crate::environment::initialize_process()
    {
        exit_argument_error(&error);
    }

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

    if let Err(error) = validate_zero_arity_command(cmd, &args[2..]) {
        exit_argument_error(&error);
    }

    // --- Sous-commandes daemon / client ---
    match cmd.as_str() {
        "daemon" => cmd_daemon(&args[2..]),
        "federate" => crate::federate::run(&args[2..]),
        "t3" => crate::t3code::run(&args[2..]),
        "managed-bootstrap" => cmd_managed_bootstrap(&args[2..]),
        "managed-wrapper" => cmd_managed_wrapper(&args[2..]),
        "mcp" => cmd_mcp(),
        "attach" => cmd_attach(&args[2..]),
        "journal" => cmd_journal(&args[2..]),
        "events" => cmd_events(&args[2..]),
        "thread" => cmd_thread(&args[2..]),
        "handoff" => cmd_handoff(&args[2..]),
        "artifact" => cmd_artifact(&args[2..]),
        "spawn" => cmd_spawn(&args[2..]),
        "stop" => cmd_stop(&args[2..]),
        "relaunch" => cmd_relaunch(&args[2..]),
        "decommission" => cmd_decommission(&args[2..]),
        "adopt-stopped" => cmd_adopt_stopped(&args[2..]),
        "send" => cmd_send(&args[2..]),
        "guichet" => cmd_guichet(&args[2..]),
        "cancel" => cmd_cancel(&args[2..]),
        "requests" => cmd_requests(&args[2..]),
        "rename" => cmd_rename(&args[2..]),
        "runtime" => cmd_runtime(&args[2..]),
        "identity" => cmd_identity(&args[2..]),
        "domain" => cmd_domain(&args[2..]),
        "control" => cmd_control(&args[2..]),
        "inbox" => cmd_inbox(&args[2..]),
        "dnd" => cmd_dnd(&args[2..]),
        "hook" => cmd_hook(&args[2..]),
        "install-hooks" => cmd_install_hooks(&args[2..]),
        "reply" => cmd_reply(&args[2..]),
        "who" => cmd_who(&args[2..]),
        "agents" => cmd_agents(&args[2..]),
        "discover" => cmd_discover(),
        "status" => cmd_status(),
        "ledger" => cmd_ledger(&args[2..]),
        "reprise" => cmd_reprise(&args[2..]),
        "reaper" => cmd_reaper(&args[2..]),
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

fn parse_artifact_read(
    args: &[String],
) -> Result<bridget_transport::protocol::ArtifactReadRequest, String> {
    use bridget_transport::protocol::{
        ARTIFACT_READ_VERSION, ArtifactReadKind, ArtifactReadRequest, MAX_ARTIFACT_READ_BYTES,
    };
    if args.first().map(String::as_str) != Some("read") {
        return Err("artifact : attendu read --artifact-ref REF --version-ref REF [--blob SHA256] [--offset N] [--limit N]".into());
    }
    let mut values = std::collections::BTreeMap::new();
    let mut arguments = args[1..].iter();
    while let Some(option) = arguments.next() {
        if ![
            "--artifact-ref",
            "--version-ref",
            "--blob",
            "--offset",
            "--limit",
        ]
        .contains(&option.as_str())
        {
            return Err(unknown_argument("artifact read", option));
        }
        let value = arguments
            .next()
            .filter(|value| !value.starts_with("--"))
            .ok_or_else(|| format!("artifact read : valeur manquante pour {option}"))?;
        if values.insert(option.as_str(), value.as_str()).is_some() {
            return Err(format!("artifact read : option répétée {option}"));
        }
    }
    let required = |key: &str| {
        values
            .get(key)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
            .ok_or_else(|| format!("artifact read : {key} requis"))
    };
    let offset = values
        .get("--offset")
        .unwrap_or(&"0")
        .parse::<u64>()
        .map_err(|_| "artifact read : offset invalide".to_string())?;
    let limit = values
        .get("--limit")
        .map(|value| value.parse::<u32>())
        .transpose()
        .map_err(|_| "artifact read : limit invalide".to_string())?
        .unwrap_or(MAX_ARTIFACT_READ_BYTES);
    if limit == 0 || limit > MAX_ARTIFACT_READ_BYTES {
        return Err("artifact read : limit doit être comprise entre 1 et 16384".into());
    }
    Ok(ArtifactReadRequest {
        version: ARTIFACT_READ_VERSION,
        artifact_ref: required("--artifact-ref")?,
        version_ref: required("--version-ref")?,
        kind: values
            .get("--blob")
            .map(|digest| ArtifactReadKind::Blob {
                digest: digest.to_string(),
            })
            .unwrap_or(ArtifactReadKind::Manifest),
        offset,
        limit,
    })
}

fn cmd_artifact(args: &[String]) {
    let request = parse_artifact_read(args).unwrap_or_else(|error| exit_argument_error(&error));
    let identity = crate::mcp_identity::resolve_current_identity().unwrap_or_else(|error| {
        eprintln!(
            "bridget artifact : {} : {}",
            error.code(),
            error.remediation()
        );
        std::process::exit(1);
    });
    let response = crate::communication::client::read_artifact(
        &identity.name,
        &identity.instance_id,
        &DaemonConfig::default().socket_path,
        request,
    )
    .unwrap_or_else(|error| {
        match error {
            crate::communication::client::ClientError::InvalidParams(message) => {
                exit_argument_error(&message)
            }
            crate::communication::client::ClientError::Technical { code, message } => {
                eprintln!("bridget artifact : {code} : {message}")
            }
        }
        std::process::exit(1);
    });
    println!(
        "{}",
        serde_json::to_string(&response).expect("DTO de lecture sérialisable")
    );
    if matches!(
        response,
        DaemonToWrapper::ArtifactReadResult {
            outcome: bridget_transport::protocol::ArtifactReadOutcome::Rejected { .. },
            ..
        }
    ) {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod artifact_read_tests {
    use super::parse_artifact_read;
    use bridget_transport::protocol::{ArtifactReadKind, MAX_ARTIFACT_READ_BYTES};

    #[test]
    fn lecture_cli_construit_le_dto_ferme_sans_identite_ou_chemin() {
        let args = |values: &[&str]| {
            values
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
        };
        let base = ["read", "--artifact-ref", "a", "--version-ref", "v"];
        let request = parse_artifact_read(&args(&base)).unwrap();
        assert_eq!(request.version, 1);
        assert_eq!(request.kind, ArtifactReadKind::Manifest);
        assert_eq!(request.offset, 0);
        assert_eq!(request.limit, MAX_ARTIFACT_READ_BYTES);
        let blob = parse_artifact_read(&args(&[
            "read",
            "--artifact-ref",
            "a",
            "--version-ref",
            "v",
            "--blob",
            "digest",
            "--offset",
            "32",
            "--limit",
            "7",
        ]))
        .unwrap();
        assert_eq!(
            blob.kind,
            ArtifactReadKind::Blob {
                digest: "digest".into()
            }
        );
        assert_eq!((blob.offset, blob.limit), (32, 7));
        for extra in [
            vec!["--project", "autre"],
            vec!["--agent", "autre"],
            vec!["--path", "/etc/passwd"],
            vec!["--limit", "0"],
            vec!["--limit", "16385"],
            vec!["--offset", "-1"],
            vec!["--limit"],
            vec!["--artifact-ref", "b"],
        ] {
            let mut invalid = args(&base);
            invalid.extend(args(&extra));
            assert!(parse_artifact_read(&invalid).is_err(), "{invalid:?}");
        }
        assert!(parse_artifact_read(&args(&["read", "--artifact-ref", "a"])).is_err());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum IdentityCommand {
    Migrate {
        apply: bool,
        db_path: PathBuf,
        fleet_path: Option<PathBuf>,
    },
}

fn parse_identity_command(args: &[String]) -> Result<IdentityCommand, String> {
    if args.first().map(String::as_str) != Some("migrate") {
        return Err(
            "usage: bridget identity migrate --dry-run|--apply [--db <chemin>] [--fleet <chemin>]"
                .to_string(),
        );
    }
    let mut apply = None;
    let mut db_path = None;
    let mut fleet_path = None;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--dry-run" => {
                if apply.replace(false).is_some() {
                    return Err("bridget identity migrate: mode dupliqué".to_string());
                }
            }
            "--apply" => {
                if apply.replace(true).is_some() {
                    return Err("bridget identity migrate: mode dupliqué".to_string());
                }
            }
            "--db" => {
                index += 1;
                db_path = args.get(index).map(PathBuf::from);
                if db_path.is_none() {
                    return Err(
                        "bridget identity migrate: --db requiert un chemin absolu".to_string()
                    );
                }
            }
            "--fleet" => {
                index += 1;
                fleet_path = args.get(index).map(PathBuf::from);
                if fleet_path.is_none() {
                    return Err(
                        "bridget identity migrate: --fleet requiert un chemin absolu".to_string(),
                    );
                }
            }
            option => return Err(unknown_argument("identity migrate", option)),
        }
        index += 1;
    }
    let db_path = db_path.unwrap_or_else(|| DaemonConfig::default().db_path);
    // La migration explicite n'est pas une échappatoire au namespace :
    // valider avant toute lecture de plan et avant tout --apply.
    for path in std::iter::once(&db_path).chain(fleet_path.iter()) {
        crate::environment::validate_path(path)?;
    }
    for path in [&db_path, fleet_path.as_ref().unwrap_or(&PathBuf::new())] {
        if !path.as_os_str().is_empty() && !path.is_absolute() {
            return Err("bridget identity migrate: les chemins doivent être absolus".to_string());
        }
    }
    Ok(IdentityCommand::Migrate {
        apply: apply.ok_or_else(|| {
            "bridget identity migrate: --dry-run ou --apply est obligatoire".to_string()
        })?,
        db_path,
        fleet_path,
    })
}

fn cmd_identity(args: &[String]) {
    let command = parse_identity_command(args).unwrap_or_else(|error| exit_argument_error(&error));
    let IdentityCommand::Migrate {
        apply,
        db_path,
        fleet_path,
    } = command;
    let mut paths = crate::identity_migration::IdentityMigrationPaths::for_bridget_db(db_path);
    if let Some(fleet_path) = fleet_path {
        paths.fleet_path = fleet_path;
    }
    if apply && socket_path().exists() {
        eprintln!(
            "bridget identity migrate: daemon actif ou socket encore présent; arrêtez Bridget avant --apply"
        );
        std::process::exit(1);
    }
    let plan = crate::identity_migration::plan(paths).unwrap_or_else(|error| {
        eprintln!("bridget identity migrate: {error}");
        std::process::exit(1);
    });
    if !apply {
        let output = serde_json::json!({
            "mode": "dry_run",
            "agents_a_migrer": plan.mapping.len(),
            "references_legacy": plan.mapping.keys().collect::<Vec<_>>(),
            "bridget_db": plan.paths.bridget_db,
            "fleet_path": plan.paths.fleet_path,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&output).expect("rapport sérialisable")
        );
        return;
    }
    let outcome = crate::identity_migration::apply(&plan).unwrap_or_else(|error| {
        eprintln!("bridget identity migrate: {error}");
        std::process::exit(1);
    });
    let output = serde_json::json!({
        "mode": "applied",
        "agents_migres": outcome.migrated_agents,
        "sauvegardes": outcome.backup_paths,
        "journal": outcome.journal_path,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).expect("rapport sérialisable")
    );
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

fn extract_wrapper_args(args: &[String]) -> (Option<String>, Vec<String>) {
    let mut agent_id = None;
    let mut rest = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--agent-id" && i + 1 < args.len() {
            agent_id = Some(args[i + 1].clone());
            i += 2;
        } else {
            rest.push(args[i].clone());
            i += 1;
        }
    }
    (agent_id, rest)
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
        "bridget {} — protocole de communication inter-agents\n\n\
         Usage: bridget <COMMANDE> [OPTIONS]\n\n\
         Agents CLI (wrappers) :\n  \
           codex [ARGS...]        Lance Codex + connexion daemon\n  \
           claude [ARGS...]       Lance Claude + connexion daemon\n  \
           gemini [ARGS...]       Lance Gemini + connexion daemon\n  \
           gclaude [ARGS...]      Lance gclaude + connexion daemon\n  \
           -- <CMD> [ARGS...]     Agent personnalisé\n\n\
         Daemon & client :\n  \
           daemon                 Lance le daemon\n  \
           federate <OP>          Gère les liaisons SSH persistantes (--help)\n  \
           t3 <OP>                Expose les fils t3code comme agents (install|status|uninstall|serve)\n  \
           mcp                    Lance le serveur MCP sur stdio\n  \
           attach <UUID>          Observe et écrit à un équipier [--from-seq N | --date AAAA-MM-JJ]\n  \
           journal <UUID>         Extrait [--tail N | --from-seq N] [--to UUID] [--reply]\n  \
           events <OP>            types | sub EVENEMENT [--agent UUID] [--file MOTIF] [--once] [--ttl S] | list | unsub ID\n  \
           thread <OP>            Fils partagés : create | list | show | post | read | ack | history | close (thread --help)\n  \
           handoff <OP>           Dossier de passation : preview | send, objet JSON sur stdin (--json-stdin [--json])\n  \
           spawn <TYPE>           Lance un équipier géré (--persistent | --no-persistent) [--agent-id UUID] [--posture discovery|development]\n  \
           stop <N>               Arrête un équipier géré\n  \
           relaunch <N>           Relance un équipier géré arrêté\n  \
           decommission <N>       Retire un équipier de la flotte visible\n  \
           adopt-stopped <N>...   Importe explicitement d'anciens agents arrêtés\n  \
           send --to <N> [--] <MSG> Envoie un message\n  \
           reply [--] <MSG>       Répond au dernier expéditeur\n  \
           cancel <ID>            Annule une demande suivie [--reason <T>]\n  \
           requests [--all]       Mes demandes (défaut) ou toutes les ouvertes\n  \
           rename <N>             Renomme l'agent courant\n  \
           control <OP>           Référent : status [--history] | pause [--reason <T>] | resume | budget <N> | posture discovery|complete\n  \
           inbox <OP>             Référent : list [--all] | resolve <ID> <CHOIX>\n  \
           runtime --model <M>    Déclare le modèle courant [--effort <E>]\n  \
           domain <N> | --reset   Change le domaine de l'agent courant\n  \
           dnd [off]              Ne pas déranger [--duration 30m]\n  \
           install-hooks          Installe la détection auto du modèle (Claude)\n  \
           who [--domain <D>]     Agents connectés\n  \
           agents [--json]        Idem, format machine [--domain <D>]\n  \
           status                 Santé du daemon\n  \
           ledger [--limit N]     Historique des messages (défaut : maximum lisible)\n  \
           ledger search --query Q  Cherche dans ses échanges ou un fil [--cursor HEX] [--json]\n  \
           ledger read --id I --target T  Relit un message exact par fragments [--json]\n  \
           artifact read          Lit un contenu exact par références, sans exécution\n  \
           reprise [--write P]    Carte de reprise du référent\n  \
           reaper report          Observateur J2 (ne tue jamais)\n  \
           version                Version\n  \
           help                   Cette aide\n\n\
         Options de send :\n  \
           identity migrate --dry-run|--apply [--db <P>] [--fleet <P>]\n  \
           --to <nom>             Destinataire (requis)\n  \
           --from <nom>           Se nommer ; le nom doit être adressable en\n  \
           \x20                      retour, sinon l'envoi est refusé\n  \
           --in-reply-to <id>     Lie la réponse à une demande suivie\n  \
           --reply                Réponse attendue\n  \
           --timeout <S>          Délai avant échec (défaut: 60)\n  \
           --hops <N>             Sauts restants (défaut: 4)\n  \
           --                      Fin des options ; le reste est le message\n\n\
           --id <clé>             Clé de rejeu (avec --issued-at)\n  \
           --issued-at <unix>     Instant d'émission du rejeu\n  \
           --issuer-scope <portée> Portée requise pour un envoi ordinaire idempotent\n\n\
         Usage interne :\n  \
           hook claude-runtime    Appelé par le hook Claude Code, lit stdin\n  \
           hook claude-statusline Limites de forfait, lit le payload StatusLine\n    \
                                  sur stdin. N'affiche RIEN : à appeler en plus\n    \
                                  de votre statusLine, pas à sa place —\n    \
                                  printf '%s' \"$input\" | bridget hook claude-statusline &",
        env!("CARGO_PKG_VERSION")
    );
}

fn socket_path() -> std::path::PathBuf {
    DaemonConfig::default().socket_path
}

fn cmd_journal(args: &[String]) {
    let result = (|| -> Result<crate::attach::JournalRequest, String> {
        let agent = args
            .first()
            .filter(|s| !s.starts_with('-'))
            .ok_or("journal : agent requis")?;
        let mut request = serde_json::json!({"agent":agent});
        let mut i = 1;
        while i < args.len() {
            let key = match args[i].as_str() {
                "--tail" => "tail",
                "--from-seq" => "from_seq",
                "--to" => "to",
                "--reply" => "reply",
                other => return Err(unknown_argument("journal", other)),
            };
            if request.get(key).is_some() {
                return Err(format!("journal : option répétée {key}"));
            }
            if key == "reply" {
                request[key] = serde_json::json!(true);
            } else {
                i += 1;
                let value = args.get(i).ok_or("journal : valeur manquante")?;
                request[key] = if key == "to" {
                    serde_json::json!(value)
                } else {
                    serde_json::json!(
                        value
                            .parse::<u64>()
                            .map_err(|_| "journal : entier requis")?
                    )
                };
            }
            i += 1;
        }
        serde_json::from_value(request).map_err(|e| e.to_string())
    })()
    .unwrap_or_else(|error| exit_argument_error(&error));
    let excerpt = result.read(&socket_path()).unwrap_or_else(|error| {
        eprintln!("bridget journal : {error}");
        std::process::exit(1);
    });
    if let Some(to) = result.to {
        let mut send = vec!["--to".into(), to];
        if result.reply {
            send.push("--reply".into());
        }
        send.extend(["--".into(), excerpt.shared_body()]);
        cmd_send(&send);
    } else {
        println!("{}", serde_json::to_string(&excerpt).expect("extrait JSON"));
    }
}

fn parse_events_args(
    args: &[String],
) -> Result<bridget_transport::protocol::ObservationRequest, String> {
    let action = args
        .first()
        .ok_or("events : types, sub, list ou unsub requis")?;
    let mut value = serde_json::json!({"action":action});
    let mut i = 1;
    if action == "sub" || action == "unsub" {
        let argument = args
            .get(i)
            .filter(|s| !s.starts_with('-'))
            .ok_or("events : événement ou id manquant")?;
        value[if action == "sub" { "event" } else { "id" }] = serde_json::json!(argument);
        i += 1;
    }
    while i < args.len() {
        if action != "sub" {
            return Err("events : paramètres superflus".into());
        }
        let key = match args[i].as_str() {
            "--agent" => "agent",
            "--file" => "file",
            "--once" => "once",
            "--ttl" => "ttl_secs",
            other => return Err(unknown_argument("events", other)),
        };
        if value.get(key).is_some() {
            return Err(format!("events : option répétée {key}"));
        }
        if key == "once" {
            value[key] = serde_json::json!(true);
        } else {
            i += 1;
            let argument = args.get(i).ok_or("events : valeur manquante")?;
            value[key] = if key == "ttl_secs" {
                serde_json::json!(
                    argument
                        .parse::<u64>()
                        .map_err(|_| "events : ttl entier requis")?
                )
            } else {
                serde_json::json!(argument)
            };
        }
        i += 1;
    }
    serde_json::from_value(value).map_err(|e| format!("events : {e}"))
}

#[cfg(test)]
#[test]
fn spec100_cli_events_contract() {
    use bridget_transport::protocol::{ObservationKind, ObservationRequest};
    let args = [
        "sub",
        "file_collision",
        "--agent",
        "a",
        "--file",
        "*.rs",
        "--ttl",
        "30",
        "--once",
    ]
    .map(String::from);
    assert!(matches!(
        parse_events_args(&args).unwrap(),
        ObservationRequest::Sub {
            event: ObservationKind::FileCollision,
            once: true,
            ttl_secs: Some(30),
            agent: Some(_),
            file: Some(_)
        }
    ));
    assert!(matches!(
        parse_events_args(&["list".into()]).unwrap(),
        ObservationRequest::List {}
    ));
    assert!(matches!(
        parse_events_args(&["types".into()]).unwrap(),
        ObservationRequest::Types {}
    ));
    assert!(matches!(
        parse_events_args(&["unsub".into(), "id".into()]).unwrap(),
        ObservationRequest::Unsub { .. }
    ));
}

fn cmd_events(args: &[String]) {
    let request = parse_events_args(args).unwrap_or_else(|error| exit_argument_error(&error));
    let identity = crate::mcp_identity::resolve_current_identity().unwrap_or_else(|error| {
        eprintln!(
            "bridget events : {} : {}",
            error.code(),
            error.remediation()
        );
        std::process::exit(1);
    });
    let result = crate::communication::client::observation_request(
        &identity.name,
        &identity.instance_id,
        &socket_path(),
        request,
    )
    .unwrap_or_else(|error| {
        eprintln!("bridget events : {error}");
        std::process::exit(1);
    });
    println!("{result}");
    if result["status"] == "rejected" {
        std::process::exit(1);
    }
}

/// Contexte de réponse laissé par une alerte de fil (format JSON discriminé) ;
/// les anciennes lignes « expéditeur[TAB]id » restent lues telles quelles.
fn thread_notice_marker(previous: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(previous.trim()).ok()?;
    (value["kind"] == "thread_notice")
        .then(|| value["thread_id"].as_str().map(str::to_owned))
        .flatten()
}

const THREAD_USAGE: &str = "usage :\n  \
  bridget thread create --title TITRE --member UUID|NOM [--member …] --id UUID\n  \
  bridget thread list [--limit N] [--after UUID]\n  \
  bridget thread show THREAD\n  \
  bridget thread post THREAD (--silent | --notify UUID|NOM [--notify …] | --all) --id UUID [--reply-to N] [--ack RECU] -- TEXTE\n  \
  bridget thread read THREAD [--limit N]\n  \
  bridget thread ack THREAD RECU\n  \
  bridget thread history THREAD [--from-seq N] [--to-seq N] [--limit N]\n  \
  bridget thread close THREAD --id UUID\n\n\
Un dépôt --silent ne réveille personne ; --notify vise des membres, --all tous les autres membres.\n\
Recevoir une alerte n'est pas lire ; lire (read) n'est pas confirmer (ack) : confirmer chaque page reçue.\n\
--id est une clé de rejeu à préparer avant l'appel (uuidgen) et à réutiliser à l'identique après une coupure.\n\
Sorties JSON ; code de sortie 0 succès, 2 validation ou refus, 1 panne technique.";

/// Session 102 : famille `bridget thread`, même contrat que l'outil MCP.
fn cmd_thread(args: &[String]) {
    if matches!(
        args.first().map(String::as_str),
        None | Some("--help") | Some("-h") | Some("help")
    ) {
        println!("{THREAD_USAGE}");
        std::process::exit(if args.is_empty() { 2 } else { 0 });
    }
    let action = parse_thread_args(args, &resolve_thread_name).unwrap_or_else(|error| {
        eprintln!("{THREAD_USAGE}");
        eprintln!("erreur: {error}");
        std::process::exit(2);
    });
    let identity = crate::mcp_identity::resolve_current_identity().unwrap_or_else(|error| {
        eprintln!(
            "bridget thread : {} : {}",
            error.code(),
            error.remediation()
        );
        std::process::exit(1);
    });
    let result = crate::communication::client::thread_request(
        &identity.name,
        &identity.instance_id,
        &socket_path(),
        bridget_transport::protocol::ThreadRequest {
            version: bridget_transport::protocol::THREAD_CONTRACT_VERSION,
            request: action,
        },
    )
    .unwrap_or_else(|error| {
        eprintln!("bridget thread : {error}");
        std::process::exit(1);
    });
    println!("{}", result.result);
    if result.result["status"] == "error" {
        std::process::exit(2);
    }
}

/// Résout un nom d'affichage saisi explicitement via l'annuaire ; un nom
/// inconnu ou porté par plusieurs agents est refusé, jamais choisi.
fn resolve_thread_name(name: &str) -> Result<String, String> {
    let agents = match send_control_to_daemon(WrapperToDaemon::ListAgents) {
        Ok(DaemonToWrapper::AgentList { agents }) => agents,
        Ok(_) => return Err("annuaire indisponible : donner l'UUID".to_string()),
        Err(error) => return Err(format!("annuaire indisponible ({error}) : donner l'UUID")),
    };
    resolve_name_in_directory(
        name,
        agents
            .iter()
            .map(|agent| (agent.display_name.as_str(), agent.agent_id.as_str())),
    )
}

fn resolve_name_in_directory<'a>(
    name: &str,
    directory: impl Iterator<Item = (&'a str, &'a str)>,
) -> Result<String, String> {
    let mut matches: Vec<&str> = directory
        .filter(|(display_name, _)| *display_name == name)
        .map(|(_, agent_id)| agent_id)
        .collect();
    matches.sort_unstable();
    matches.dedup();
    match matches.as_slice() {
        [] => Err(format!(
            "membre inconnu « {name} » : donner l'UUID de l'agent"
        )),
        [single] => Ok((*single).to_string()),
        several => Err(format!(
            "nom ambigu « {name} » ({} agents) : donner l'UUID de l'agent",
            several.len()
        )),
    }
}

#[derive(Default)]
struct ThreadArgs {
    positionals: Vec<String>,
    single: std::collections::BTreeMap<&'static str, String>,
    multi: std::collections::BTreeMap<&'static str, Vec<String>>,
    flags: std::collections::BTreeSet<&'static str>,
    text: Option<String>,
}

const THREAD_SINGLE_OPTIONS: [&str; 8] = [
    "--title",
    "--id",
    "--limit",
    "--after",
    "--reply-to",
    "--ack",
    "--from-seq",
    "--to-seq",
];
const THREAD_MULTI_OPTIONS: [&str; 2] = ["--member", "--notify"];
const THREAD_FLAGS: [&str; 2] = ["--silent", "--all"];

fn collect_thread_args(args: &[String]) -> Result<ThreadArgs, String> {
    let mut parsed = ThreadArgs::default();
    let mut index = 0;
    while index < args.len() {
        let argument = args[index].as_str();
        if argument == "--" {
            let rest = &args[index + 1..];
            if rest.is_empty() {
                return Err("thread : texte manquant après --".to_string());
            }
            parsed.text = Some(rest.join(" "));
            break;
        }
        if let Some(option) = THREAD_SINGLE_OPTIONS.iter().find(|o| **o == argument) {
            if parsed.single.contains_key(option) {
                return Err(format!("thread : option répétée {option}"));
            }
            let value = option_value(args, &mut index, option)?;
            parsed.single.insert(option, value);
        } else if let Some(option) = THREAD_MULTI_OPTIONS.iter().find(|o| **o == argument) {
            let value = option_value(args, &mut index, option)?;
            parsed.multi.entry(option).or_default().push(value);
        } else if let Some(flag) = THREAD_FLAGS.iter().find(|f| **f == argument) {
            if !parsed.flags.insert(flag) {
                return Err(format!("thread : option répétée {flag}"));
            }
        } else if argument.starts_with('-') && argument.len() > 1 {
            return Err(unknown_argument("thread", argument));
        } else {
            parsed.positionals.push(argument.to_string());
        }
        index += 1;
    }
    Ok(parsed)
}

fn thread_positive(parsed: &ThreadArgs, option: &str) -> Result<Option<u64>, String> {
    match parsed.single.get(option) {
        None => Ok(None),
        Some(value) => value
            .parse::<u64>()
            .ok()
            .filter(|value| *value > 0)
            .map(Some)
            .ok_or_else(|| format!("thread : {option} attend un entier strictement positif")),
    }
}

fn thread_members(
    parsed: &ThreadArgs,
    option: &'static str,
    resolver: &dyn Fn(&str) -> Result<String, String>,
) -> Result<Vec<String>, String> {
    let mut members = Vec::new();
    for value in parsed.multi.get(option).into_iter().flatten() {
        if crate::threads::canonical_uuid(value).is_some() {
            members.push(value.clone());
        } else {
            members.push(resolver(value)?);
        }
    }
    Ok(members)
}

/// Analyse stricte : sous-commande, options nommées, texte après `--`.
/// Les noms explicitement saisis sont résolus par `resolver` ; un UUID
/// canonique est transmis tel quel. Aucun --from, aucun chemin, aucun script.
fn parse_thread_args(
    args: &[String],
    resolver: &dyn Fn(&str) -> Result<String, String>,
) -> Result<bridget_transport::protocol::ThreadAction, String> {
    use bridget_transport::protocol::{ThreadAction, ThreadNotify, ThreadNotifyAll};
    let (subcommand, rest) = args.split_first().ok_or("thread : sous-commande requise")?;
    let parsed = collect_thread_args(rest)?;
    let allowed: &[&str] = match subcommand.as_str() {
        "create" => &["--title", "--member", "--id"],
        "list" => &["--limit", "--after"],
        "show" | "ack" => &[],
        "post" => &[
            "--silent",
            "--notify",
            "--all",
            "--id",
            "--reply-to",
            "--ack",
        ],
        "read" => &["--limit"],
        "history" => &["--from-seq", "--to-seq", "--limit"],
        "close" => &["--id"],
        other => return Err(format!("thread : sous-commande inconnue « {other} »")),
    };
    for used in parsed
        .single
        .keys()
        .chain(parsed.multi.keys())
        .chain(parsed.flags.iter())
    {
        if !allowed.contains(used) {
            return Err(format!("thread {subcommand} : option {used} non admise"));
        }
    }
    if parsed.text.is_some() && subcommand != "post" {
        return Err(format!("thread {subcommand} : texte après -- non admis"));
    }
    let positional = |index: usize, name: &str| -> Result<String, String> {
        parsed
            .positionals
            .get(index)
            .cloned()
            .ok_or_else(|| format!("thread {subcommand} : {name} requis"))
    };
    let expected_positionals = match subcommand.as_str() {
        "create" | "list" => 0,
        "ack" => 2,
        _ => 1,
    };
    if parsed.positionals.len() > expected_positionals {
        return Err(format!(
            "thread {subcommand} : argument superflu « {} »",
            parsed.positionals[expected_positionals]
        ));
    }
    let id = || -> Result<String, String> {
        parsed.single.get("--id").cloned().ok_or_else(|| {
            format!(
                "thread {subcommand} : --id UUID requis (clé de rejeu à préparer avant l'appel)"
            )
        })
    };
    let limit =
        thread_positive(&parsed, "--limit")?.map(|value| value.min(u64::from(u32::MAX)) as u32);
    Ok(match subcommand.as_str() {
        "create" => ThreadAction::Create {
            title: parsed
                .single
                .get("--title")
                .cloned()
                .ok_or("thread create : --title requis")?,
            members: thread_members(&parsed, "--member", resolver)?,
            operation_id: id()?,
        },
        "list" => ThreadAction::List {
            limit,
            after_thread_id: parsed.single.get("--after").cloned(),
        },
        "show" => ThreadAction::Show {
            thread_id: positional(0, "THREAD")?,
        },
        "post" => {
            let thread_id = positional(0, "THREAD")?;
            let silent = parsed.flags.contains("--silent");
            let all = parsed.flags.contains("--all");
            let targets = thread_members(&parsed, "--notify", resolver)?;
            let choices = usize::from(silent) + usize::from(all) + usize::from(!targets.is_empty());
            if choices != 1 {
                return Err(
                    "thread post : choisir exactement un mode parmi --silent, --notify, --all"
                        .to_string(),
                );
            }
            let body = parsed
                .text
                .clone()
                .ok_or("thread post : texte requis après --")?;
            ThreadAction::Post {
                thread_id,
                body,
                notify: if all {
                    ThreadNotify::All(ThreadNotifyAll::All)
                } else {
                    ThreadNotify::Targets(targets)
                },
                operation_id: id()?,
                reply_to_seq: thread_positive(&parsed, "--reply-to")?,
                ack_receipt: parsed.single.get("--ack").cloned(),
            }
        }
        "read" => ThreadAction::Read {
            thread_id: positional(0, "THREAD")?,
            limit,
        },
        "ack" => ThreadAction::Ack {
            thread_id: positional(0, "THREAD")?,
            receipt: positional(1, "RECU")?,
        },
        "history" => ThreadAction::History {
            thread_id: positional(0, "THREAD")?,
            from_seq: thread_positive(&parsed, "--from-seq")?,
            to_seq: thread_positive(&parsed, "--to-seq")?,
            limit,
        },
        "close" => ThreadAction::Close {
            thread_id: positional(0, "THREAD")?,
            operation_id: id()?,
        },
        _ => unreachable!("sous-commande validée plus haut"),
    })
}

#[cfg(test)]
mod spec102_cli_thread_tests {
    use super::*;
    use bridget_transport::protocol::{ThreadAction, ThreadNotify, ThreadNotifyAll};

    const A: &str = "10200000-0000-4000-8000-00000000000a";
    const B: &str = "10200000-0000-4000-8000-00000000000b";
    const T: &str = "33333333-3333-4333-8333-333333333333";
    const OP: &str = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    fn resolver(name: &str) -> Result<String, String> {
        resolve_name_in_directory(
            name,
            [
                ("Agent-Relecture", B),
                ("Daily", A),
                ("Daily", B),
                ("Daily", B),
            ]
            .into_iter(),
        )
    }

    #[test]
    fn spec102_v29_marqueur_thread_notice_reconnu_et_ancien_format_conserve() {
        assert_eq!(
            thread_notice_marker(
                r#"{"kind":"thread_notice","thread_id":"33333333-3333-4333-8333-333333333333"}"#
            )
            .as_deref(),
            Some("33333333-3333-4333-8333-333333333333")
        );
        assert!(thread_notice_marker("10200000-0000-4000-8000-00000000000a\tmsg-1").is_none());
        assert!(thread_notice_marker("10200000-0000-4000-8000-00000000000a").is_none());
        assert!(thread_notice_marker(r#"{"kind":"autre","thread_id":"x"}"#).is_none());
    }

    #[test]
    fn spec102_v07_cli_noms_ambigus_refuses() {
        let unique = parse_thread_args(
            &args(&[
                "create",
                "--title",
                "T",
                "--member",
                "Agent-Relecture",
                "--id",
                OP,
            ]),
            &resolver,
        )
        .unwrap();
        assert!(
            matches!(unique, ThreadAction::Create { members, .. } if members == vec![B.to_string()])
        );
        let ambiguous = parse_thread_args(
            &args(&["post", T, "--notify", "Daily", "--id", OP, "--", "x"]),
            &resolver,
        )
        .unwrap_err();
        assert!(
            ambiguous.contains("ambigu") && ambiguous.contains("2 agents"),
            "{ambiguous}"
        );
        let unknown = parse_thread_args(
            &args(&["post", T, "--notify", "Inconnu", "--id", OP, "--", "x"]),
            &resolver,
        )
        .unwrap_err();
        assert!(unknown.contains("inconnu"), "{unknown}");
        // Un UUID canonique est une adresse stable : jamais résolu ni renommé.
        let direct = parse_thread_args(
            &args(&["post", T, "--notify", A, "--id", OP, "--", "x"]),
            &|_| Err("annuaire interdit".into()),
        )
        .unwrap();
        assert!(
            matches!(direct, ThreadAction::Post { notify: ThreadNotify::Targets(t), .. } if t == vec![A.to_string()])
        );
    }

    #[test]
    fn spec102_v33_cli_thread_flags_stricts_et_formes_exactes() {
        let silent = parse_thread_args(
            &args(&["post", T, "--silent", "--id", OP, "--", "un", "texte"]),
            &resolver,
        )
        .unwrap();
        assert!(
            matches!(silent, ThreadAction::Post { ref body, notify: ThreadNotify::Targets(ref t), .. } if body == "un texte" && t.is_empty())
        );
        let all = parse_thread_args(
            &args(&[
                "post",
                T,
                "--all",
                "--id",
                OP,
                "--reply-to",
                "3",
                "--ack",
                OP,
                "--",
                "x",
            ]),
            &resolver,
        )
        .unwrap();
        assert!(matches!(
            all,
            ThreadAction::Post {
                notify: ThreadNotify::All(ThreadNotifyAll::All),
                reply_to_seq: Some(3),
                ack_receipt: Some(_),
                ..
            }
        ));
        assert!(matches!(
            parse_thread_args(&args(&["list", "--limit", "5", "--after", T]), &resolver).unwrap(),
            ThreadAction::List {
                limit: Some(5),
                after_thread_id: Some(_)
            }
        ));
        assert!(matches!(
            parse_thread_args(&args(&["show", T]), &resolver).unwrap(),
            ThreadAction::Show { .. }
        ));
        assert!(matches!(
            parse_thread_args(&args(&["read", T, "--limit", "200"]), &resolver).unwrap(),
            ThreadAction::Read {
                limit: Some(200),
                ..
            }
        ));
        assert!(matches!(
            parse_thread_args(&args(&["ack", T, OP]), &resolver).unwrap(),
            ThreadAction::Ack { .. }
        ));
        assert!(matches!(
            parse_thread_args(
                &args(&["history", T, "--from-seq", "2", "--to-seq", "9"]),
                &resolver
            )
            .unwrap(),
            ThreadAction::History {
                from_seq: Some(2),
                to_seq: Some(9),
                ..
            }
        ));
        assert!(matches!(
            parse_thread_args(&args(&["close", T, "--id", OP]), &resolver).unwrap(),
            ThreadAction::Close { .. }
        ));
        for invalid in [
            vec!["post", T, "--id", OP, "--", "x"],
            vec!["post", T, "--silent", "--all", "--id", OP, "--", "x"],
            vec!["post", T, "--silent", "--", "x"],
            vec!["post", T, "--silent", "--id", OP],
            vec!["post", T, "--silent", "--id", OP, "--id", OP, "--", "x"],
            vec!["post", T, "--silent", "--id", OP, "--from", A, "--", "x"],
            vec![
                "post",
                T,
                "--silent",
                "--id",
                OP,
                "--reply-to",
                "0",
                "--",
                "x",
            ],
            vec!["create", "--title", "T", "--member", B],
            vec!["create", "--member", B, "--id", OP],
            vec!["show"],
            vec!["show", T, "extra"],
            vec!["ack", T],
            vec!["read", T, "--limit", "abc"],
            vec!["history", T, "--", "texte"],
            vec!["close", T],
            vec!["summary", T],
            vec!["list", "--script", "/tmp/x.sh"],
        ] {
            assert!(
                parse_thread_args(&args(&invalid), &resolver).is_err(),
                "{invalid:?} aurait dû être refusé"
            );
        }
    }
}

const HANDOFF_USAGE: &str = "usage :\n  \
  bridget handoff preview --json-stdin [--json]\n  \
  bridget handoff send --json-stdin [--json]\n\n\
stdin : le même objet JSON que l'outil MCP bridget_handoff (action, draft ; pour send : to UUID,\n\
puis id et issued_at ensemble pour rejouer, reply, reply_timeout, in_reply_to), au plus 65 536 octets.\n\
Aucun fichier n'est lu, aucune référence n'est ouverte, aucune source n'est certifiée.\n\
Sortie : --json rend l'objet MCP ; sinon aperçu du corps ou reçu d'envoi lisible.\n\
Codes de sortie : 0 preview_valid ou accepted ; 2 paramètres invalides ; 1 panne, refus ou issue\n\
non confirmée (in_flight, outcome_unknown) : rejouer la même clé, jamais une nouvelle.";

const HANDOFF_STDIN_MAX_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HandoffArgs {
    action: crate::handoff::HandoffAction,
    json: bool,
}

/// Analyse stricte : sous-commande, `--json-stdin` obligatoire, `--json` facultatif.
fn parse_handoff_args(args: &[String]) -> Result<HandoffArgs, String> {
    let (subcommand, rest) = args
        .split_first()
        .ok_or("handoff : sous-commande preview ou send requise")?;
    let action = match subcommand.as_str() {
        "preview" => crate::handoff::HandoffAction::Preview,
        "send" => crate::handoff::HandoffAction::Send,
        other => return Err(format!("handoff : sous-commande inconnue « {other} »")),
    };
    let mut json_stdin = false;
    let mut json = false;
    for argument in rest {
        match argument.as_str() {
            "--json-stdin" if !json_stdin => json_stdin = true,
            "--json" if !json => json = true,
            "--json-stdin" | "--json" => {
                return Err(format!("handoff : option répétée {argument}"));
            }
            other => return Err(unknown_argument("handoff", other)),
        }
    }
    if !json_stdin {
        return Err("handoff : --json-stdin est requis (aucun fichier n'est lu)".to_string());
    }
    Ok(HandoffArgs { action, json })
}

/// Lit au plus 65 537 octets : le 65 537e est refusé avant tout parsing.
fn read_bounded_stdin<R: std::io::Read>(reader: R) -> Result<Vec<u8>, String> {
    use std::io::Read as _;
    let mut bytes = Vec::with_capacity(4096);
    reader
        .take(HANDOFF_STDIN_MAX_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("handoff : lecture de stdin : {error}"))?;
    if bytes.len() > HANDOFF_STDIN_MAX_BYTES {
        return Err(format!(
            "handoff : entrée de plus de {HANDOFF_STDIN_MAX_BYTES} octets refusée avant analyse"
        ));
    }
    Ok(bytes)
}

fn handoff_request_from_stdin(
    action: crate::handoff::HandoffAction,
    bytes: &[u8],
) -> Result<crate::handoff::HandoffRequest, String> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| format!("handoff : JSON de stdin invalide : {error}"))?;
    let request =
        crate::handoff::parse_request(&value).map_err(|error| format!("handoff : {error}"))?;
    if request.action != action {
        return Err(format!(
            "handoff : action « {} » dans l'entrée, sous-commande « {} »",
            request.action.name(),
            action.name()
        ));
    }
    Ok(request)
}

/// Session 103 : dossier de passation, même validation et même transport
/// idempotent que l'outil MCP. Aucun fichier lu, aucune source ouverte.
fn cmd_handoff(args: &[String]) {
    if matches!(
        args.first().map(String::as_str),
        None | Some("--help") | Some("-h") | Some("help")
    ) {
        println!("{HANDOFF_USAGE}");
        std::process::exit(if args.is_empty() { 2 } else { 0 });
    }
    let parsed = parse_handoff_args(args).unwrap_or_else(|error| {
        eprintln!("{HANDOFF_USAGE}");
        eprintln!("erreur: {error}");
        std::process::exit(2);
    });
    let bytes = read_bounded_stdin(std::io::stdin().lock()).unwrap_or_else(|error| {
        eprintln!("erreur: {error}");
        std::process::exit(2);
    });
    let request = handoff_request_from_stdin(parsed.action, &bytes).unwrap_or_else(|error| {
        eprintln!("erreur: {error}");
        std::process::exit(2);
    });
    let Some(transport) = request.transport else {
        let result = crate::handoff::preview_result(&request.rendered);
        if parsed.json {
            println!("{result}");
        } else {
            println!("{}", request.rendered.body);
            println!();
            println!(
                "aperçu valide : {} octets ; rien n'a été envoyé",
                request.rendered.bytes
            );
            for detail in crate::handoff::WARNING_DETAILS {
                println!("- {detail}");
            }
        }
        std::process::exit(0);
    };
    let identity = crate::mcp_identity::resolve_current_identity().unwrap_or_else(|error| {
        eprintln!(
            "bridget handoff send : {} : {}",
            error.code(),
            error.remediation()
        );
        std::process::exit(1);
    });
    let mut message = BridgetMessage::new(&identity.name, &transport.to, &request.rendered.body);
    let id = transport.id.clone().unwrap_or_else(|| message.id.clone());
    let issued_at = transport.issued_at.unwrap_or_else(unix_timestamp);
    message.id = id.clone();
    message.reply = transport.reply;
    message.reply_timeout = transport.reply_timeout;
    message.in_reply_to = transport.in_reply_to.clone();
    let options = IdempotentSendOptions {
        id: id.clone(),
        issued_at,
        issuer_scope: crate::communication::issuer_scope(&identity.instance_id),
    };
    let receipt = match send_idempotent_to_daemon(&message, &options) {
        Ok(DaemonToWrapper::IdempotencyResult { issue, .. }) => {
            crate::mcp::send_issue_result(&id, issued_at, issue)
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
    };
    let result = crate::handoff::decorate_send_result(receipt, &request.rendered);
    let accepted = result["status"] == "accepted";
    if parsed.json {
        println!("{result}");
    } else {
        println!(
            "{} : id={} issued_at={} ({} octets)",
            result["status"].as_str().unwrap_or("?"),
            result["id"].as_str().unwrap_or(&id),
            result["issued_at"],
            request.rendered.bytes
        );
        if let Some(reason) = result["reason"].as_str() {
            println!("  {reason}");
        }
        println!(
            "  {} ; {}",
            crate::handoff::WARNING_DETAILS[1],
            crate::handoff::WARNING_DETAILS[2]
        );
    }
    std::process::exit(if accepted { 0 } else { 1 });
}

#[cfg(test)]
mod spec103_cli_handoff_tests {
    use super::*;
    use crate::handoff::HandoffAction;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn spec103_s18_arguments_stricts_et_action_coherente() {
        assert_eq!(
            parse_handoff_args(&args(&["preview", "--json-stdin"])).unwrap(),
            HandoffArgs {
                action: HandoffAction::Preview,
                json: false
            }
        );
        assert_eq!(
            parse_handoff_args(&args(&["send", "--json-stdin", "--json"])).unwrap(),
            HandoffArgs {
                action: HandoffAction::Send,
                json: true
            }
        );
        for invalid in [
            vec!["preview"],
            vec!["send", "--json"],
            vec!["send", "--json-stdin", "--file", "/tmp/x"],
            vec!["send", "--json-stdin", "--json-stdin"],
            vec!["render", "--json-stdin"],
            vec!["send", "--json-stdin", "--from", "x"],
        ] {
            assert!(parse_handoff_args(&args(&invalid)).is_err(), "{invalid:?}");
        }
        let preview = br#"{"action":"preview","draft":{"objective":"o","summary":"s"}}"#;
        assert!(handoff_request_from_stdin(HandoffAction::Preview, preview).is_ok());
        let mismatch = handoff_request_from_stdin(HandoffAction::Send, preview).unwrap_err();
        assert!(mismatch.contains("sous-commande"), "{mismatch}");
        let invalid =
            handoff_request_from_stdin(HandoffAction::Preview, b"{pas du json").unwrap_err();
        assert!(invalid.contains("JSON"), "{invalid}");
        let empty = handoff_request_from_stdin(HandoffAction::Preview, b"").unwrap_err();
        assert!(empty.contains("JSON"), "{empty}");
    }

    #[test]
    fn spec103_s19_stdin_borne_a_64_kio_avant_analyse() {
        let exact = vec![b' '; HANDOFF_STDIN_MAX_BYTES];
        assert_eq!(
            read_bounded_stdin(&exact[..]).unwrap().len(),
            HANDOFF_STDIN_MAX_BYTES
        );
        let over = vec![b' '; HANDOFF_STDIN_MAX_BYTES + 1];
        let error = read_bounded_stdin(&over[..]).unwrap_err();
        assert!(error.contains("65536"), "{error}");
        // Le contenu au-delà de la borne n'est jamais analysé : même un JSON
        // valide mais trop long est refusé avant parsing.
        let mut long = br#"{"action":"preview","draft":{"objective":"o","summary":""#.to_vec();
        long.extend(std::iter::repeat_n(b's', HANDOFF_STDIN_MAX_BYTES));
        long.extend_from_slice(b"\"}}");
        assert!(read_bounded_stdin(&long[..]).is_err());
    }
}

fn cmd_attach(args: &[String]) {
    let (agent, window) = match parse_attach_args(args) {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("usage: bridget attach <UUID> [--from-seq N | --date AAAA-MM-JJ]");
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

/// Refus opposé à un spawn qui n'a pas décidé du sort de son équipier.
///
/// Un défaut silencieux se subit ; un choix obligatoire se décide. L'ancien
/// avertissement était affiché puis ignoré cinq fois de suite le 28/08, et
/// l'omission est irrattrapable : le `cwd` d'un équipier non persistant
/// n'existe plus nulle part une fois qu'il est connecté, donc rien ne permet
/// de le promouvoir après coup. Le seul moment où ce choix peut encore être
/// fait est celui-ci.
const SPAWN_SURVIE_NON_CHOISIE: &str = "choix de survie obligatoire : ajouter --persistent \
     (l'équipier est repris au redémarrage du service) ou --no-persistent (il est retiré de \
     la flotte au redémarrage et son contexte est perdu). Ce choix ne peut pas être omis : \
     il est irrattrapable une fois l'équipier lancé";

/// Les deux drapeaux ensemble ne sont pas un choix : c'est une intention
/// contradictoire, et la trancher par « le dernier gagne » rétablirait le
/// défaut silencieux que la garde supprime.
const SPAWN_SURVIE_CONTRADICTOIRE: &str =
    "--persistent et --no-persistent sont contradictoires : n'en garder qu'un";

#[derive(Debug)]
struct ParsedSpawnArgs {
    posture: Option<bridget_transport::protocol::SpawnPosture>,
    agent_type: String,
    agent_id: Option<String>,
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
            "usage: bridget spawn <type> (--persistent | --no-persistent) [--agent-id UUID] \
             [--cwd CHEMIN] [--timeout S] [--command-id ID] [--posture discovery|development]"
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
    if let Some(agent_id) = parsed.agent_id.as_deref()
        && !parsed.persistent
    {
        eprintln!(
            "avertissement: {}",
            crate::recovery_trace::non_persistent_spawn_warning(agent_id)
        );
    }
    let command_id = match &order {
        WrapperToDaemon::SpawnOrder { command_id, .. } => command_id.clone(),
        _ => unreachable!("resolve_spawn_order ne produit qu'un SpawnOrder"),
    };
    let development = matches!(
        order,
        WrapperToDaemon::SpawnOrder {
            posture: Some(bridget_transport::protocol::SpawnPosture::Development),
            ..
        }
    );
    if development && !require_interactive_terminal("spawn --posture development") {
        std::process::exit(1);
    }
    println!("command_id: {command_id}");
    let response = if matches!(
        order,
        WrapperToDaemon::SpawnOrder {
            posture: Some(_),
            ..
        }
    ) {
        send_control_request(order)
    } else {
        send_control_to_daemon(order)
    };
    match response {
        Ok(DaemonToWrapper::SpawnAccepted { .. }) => {
            println!("Équipier connecté.");
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
        agent_id: name,
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

fn cmd_relaunch(args: &[String]) {
    let (name, command_id) = parse_lifecycle_args(args, "relaunch").unwrap_or_else(|error| {
        eprintln!("usage: bridget relaunch <nom> [--command-id ID]");
        eprintln!("erreur: {error}");
        std::process::exit(2);
    });
    println!("command_id: {command_id}");
    match send_control_to_daemon(WrapperToDaemon::RelaunchOrder {
        agent_id: name,
        command_id: command_id.clone(),
    }) {
        Ok(DaemonToWrapper::RelaunchResult { outcome, .. }) => match outcome {
            RelaunchOutcome::Started { generation, .. } => {
                println!("Équipier relancé (génération {generation}).")
            }
            RelaunchOutcome::AlreadyRunning => {
                eprintln!("RELANCE REFUSÉE: l'équipier est déjà actif");
                std::process::exit(1);
            }
            RelaunchOutcome::NotManaged => {
                eprintln!("RELANCE REFUSÉE: l'agent n'est pas géré par le daemon");
                std::process::exit(1);
            }
            RelaunchOutcome::NotFound => {
                eprintln!("RELANCE REFUSÉE: équipier introuvable");
                std::process::exit(1);
            }
            RelaunchOutcome::NotRelaunchable { reason } => {
                eprintln!("RELANCE REFUSÉE: {reason}");
                std::process::exit(1);
            }
            RelaunchOutcome::Rejected { reason } => {
                eprintln!("RELANCE REFUSÉE: {}", display_spawn_refusal(&reason));
                std::process::exit(1);
            }
            RelaunchOutcome::Timeout { state } => {
                eprintln!("RELANCE INCOMPLÈTE: délai dépassé dans l'état {state}");
                std::process::exit(1);
            }
        },
        Ok(other) => {
            eprintln!("réponse relaunch inattendue du daemon: {other:?}");
            std::process::exit(1);
        }
        Err(error) => {
            eprintln!("daemon inaccessible: {error}");
            std::process::exit(1);
        }
    }
}

fn cmd_decommission(args: &[String]) {
    let (name, command_id) = parse_lifecycle_args(args, "decommission").unwrap_or_else(|error| {
        eprintln!("usage: bridget decommission <nom> [--command-id ID]");
        eprintln!("erreur: {error}");
        std::process::exit(2);
    });
    println!("command_id: {command_id}");
    match send_control_to_daemon(WrapperToDaemon::DecommissionOrder {
        agent_id: name,
        command_id: command_id.clone(),
    }) {
        Ok(DaemonToWrapper::DecommissionResult { outcome, .. }) => match outcome {
            DecommissionOutcome::Decommissioned => {
                println!("Équipier décommissionné. Historique conservé.")
            }
            DecommissionOutcome::DecommissionedForced { survivors_killed } => println!(
                "Équipier décommissionné après arrêt forcé ({survivors_killed} processus survivants terminés). Historique conservé."
            ),
            DecommissionOutcome::AlreadyDecommissioned => {
                println!("Équipier déjà décommissionné.")
            }
            DecommissionOutcome::NotManaged => {
                eprintln!("DÉCOMMISSIONNEMENT REFUSÉ: l'agent n'est pas géré par le daemon");
                std::process::exit(1);
            }
            DecommissionOutcome::NotFound => {
                eprintln!("DÉCOMMISSIONNEMENT REFUSÉ: équipier introuvable");
                std::process::exit(1);
            }
            DecommissionOutcome::Timeout { state } => {
                eprintln!("DÉCOMMISSIONNEMENT INCOMPLET: {state}");
                std::process::exit(1);
            }
        },
        Ok(other) => {
            eprintln!("réponse decommission inattendue du daemon: {other:?}");
            std::process::exit(1);
        }
        Err(error) => {
            eprintln!("daemon inaccessible: {error}");
            std::process::exit(1);
        }
    }
}

fn cmd_adopt_stopped(args: &[String]) {
    if args.is_empty() {
        eprintln!("usage: bridget adopt-stopped <nom>...");
        std::process::exit(2);
    }
    for name in args {
        if let Err(error) = validate_agent_id(name) {
            eprintln!("adopt-stopped: agent_id invalide {name}: {error}");
            std::process::exit(2);
        }
    }
    let mut refused = false;
    for name in args {
        let command_id = uuid::Uuid::new_v4().to_string();
        match send_control_to_daemon(WrapperToDaemon::AdoptStoppedOrder {
            agent_id: name.clone(),
            command_id,
        }) {
            Ok(DaemonToWrapper::AdoptStoppedResult { outcome, .. }) => match outcome {
                AdoptStoppedOutcome::Adopted { generation } => {
                    println!("{name}: adopté en état arrêté (génération {generation})")
                }
                AdoptStoppedOutcome::AlreadyManaged => {
                    println!("{name}: déjà géré par le registre durable")
                }
                AdoptStoppedOutcome::NotStopped => {
                    eprintln!("{name}: refusé, l'agent est encore actif ou injoignable");
                    refused = true;
                }
                AdoptStoppedOutcome::NoManagedHistory => {
                    eprintln!("{name}: refusé, aucune génération gérée connectée n'est prouvée");
                    refused = true;
                }
                AdoptStoppedOutcome::IncompleteHistory { reason } => {
                    eprintln!("{name}: refusé, historique géré incomplet: {reason}");
                    refused = true;
                }
            },
            Ok(other) => {
                eprintln!("{name}: réponse d'adoption inattendue du daemon: {other:?}");
                refused = true;
            }
            Err(error) => {
                eprintln!("{name}: daemon inaccessible: {error}");
                refused = true;
            }
        }
    }
    if refused {
        std::process::exit(1);
    }
}

fn parse_spawn_args(args: &[String]) -> Result<ParsedSpawnArgs, String> {
    let agent_type = args
        .first()
        .filter(|value| !value.starts_with('-'))
        .cloned()
        .ok_or_else(|| "type d'agent manquant".to_string())?;
    validate_technical_label(&agent_type)?;
    let mut parsed = ParsedSpawnArgs {
        posture: None,
        agent_type,
        agent_id: None,
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
                if parsed.persistent_was_set && !parsed.persistent {
                    return Err(SPAWN_SURVIE_CONTRADICTOIRE.to_string());
                }
                parsed.persistent = true;
                parsed.persistent_was_set = true;
                index += 1;
            }
            "--no-persistent" => {
                if parsed.persistent_was_set && parsed.persistent {
                    return Err(SPAWN_SURVIE_CONTRADICTOIRE.to_string());
                }
                parsed.persistent = false;
                parsed.persistent_was_set = true;
                index += 1;
            }
            "--agent-id" | "--cwd" | "--command-id" | "--timeout" | "--posture" => {
                let option = args[index].as_str();
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| format!("valeur manquante pour {option}"))?;
                match option {
                    "--posture" => {
                        if parsed.posture.is_some() {
                            return Err("--posture doit être choisi une seule fois".to_string());
                        }
                        parsed.posture = Some(match value.as_str() {
                            "discovery" => bridget_transport::protocol::SpawnPosture::Discovery,
                            "development" => bridget_transport::protocol::SpawnPosture::Development,
                            _ => {
                                return Err("--posture attend discovery ou development".to_string());
                            }
                        });
                    }
                    "--agent-id" => {
                        validate_agent_id(value)?;
                        parsed.agent_id = Some(value.clone());
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
    parse_lifecycle_args(args, "stop")
}

fn parse_lifecycle_args(args: &[String], action: &str) -> Result<(String, String), String> {
    let name = args
        .first()
        .cloned()
        .ok_or_else(|| "nom manquant".to_string())?;
    validate_agent_id(&name)?;
    let command_id = match args.get(1).map(String::as_str) {
        None => uuid::Uuid::new_v4().to_string(),
        Some("--command-id") if args.len() == 3 => {
            validate_command_id(&args[2])?;
            args[2].clone()
        }
        Some(_) => return Err(format!("options {action} invalides")),
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
    crate::environment::validate_state_file(&path, false)?;
    if path.exists() {
        let content = std::fs::read_to_string(&path)
            .map_err(|error| format!("ordre mémorisé illisible {}: {error}", path.display()))?;
        let stored: WrapperToDaemon = decode(content.trim())
            .map_err(|error| format!("ordre mémorisé invalide {}: {error}", path.display()))?;
        validate_retry_options(parsed, &stored)?;
        return Ok(stored);
    }
    // Ordre NEUF seulement : au-dessus, un `--command-id` déjà mémorisé rend
    // l'enveloppe figée telle quelle, et le choix de survie y est déjà inscrit.
    // Le redemander au rejeu bloquerait un rappel légitime et ouvrirait une
    // divergence entre ce qui est retapé et ce qui est mémorisé.
    if !parsed.persistent_was_set {
        return Err(SPAWN_SURVIE_NON_CHOISIE.to_string());
    }
    let cwd = parsed.cwd.as_deref().unwrap_or(current_dir);
    if !cwd.is_absolute() {
        return Err("--cwd doit être absolu".to_string());
    }
    let order = WrapperToDaemon::SpawnOrder {
        posture: parsed.posture,
        agent_type: parsed.agent_type.clone(),
        project: None,
        agent_id: Some(
            parsed
                .agent_id
                .clone()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        ),
        cwd: cwd.to_string_lossy().into_owned(),
        persistent: parsed.persistent,
        command_id,
        issued_at: now,
        deadline_at: now.saturating_add(parsed.timeout_secs),
        ownership: None,
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

#[cfg(test)]
mod spawn_executor_tests {
    use super::*;
    use std::os::unix::net::UnixListener;
    use std::path::{Path, PathBuf};
    use std::thread;

    /// Double de l'exécuteur : la CLI emprunte son vrai protocole Register /
    /// SpawnOrder, puis reçoit l'issue que seul le daemon est autorisé à rendre.
    fn daemon_repond_a_un_spawn(
        socket: &Path,
        response: DaemonToWrapper,
    ) -> thread::JoinHandle<WrapperToDaemon> {
        let listener = UnixListener::bind(socket).expect("socket d'exécuteur");
        thread::spawn(move || {
            let (stream, _) = listener.accept().expect("connexion CLI");
            let mut reader = BufReader::new(stream.try_clone().expect("clone lecture"));
            let mut writer = BufWriter::new(stream);
            let mut line = String::new();

            reader.read_line(&mut line).expect("Register CLI");
            assert!(matches!(
                decode::<WrapperToDaemon>(line.trim()).expect("Register décodable"),
                WrapperToDaemon::Register { .. }
            ));
            writeln!(
                writer,
                "{}",
                encode(&DaemonToWrapper::Registered {
                    credential: None,
                    agent_id: "cli-cwd-oracle".to_string(),
                })
                .expect("Registered encodable")
            )
            .expect("Registered écrit");
            writer.flush().expect("Registered envoyé");

            line.clear();
            reader.read_line(&mut line).expect("SpawnOrder CLI");
            let order = decode(line.trim()).expect("SpawnOrder décodable");
            writeln!(writer, "{}", encode(&response).expect("réponse encodable"))
                .expect("réponse écrite");
            writer.flush().expect("réponse envoyée");
            order
        })
    }

    fn temporary_socket(label: &str) -> PathBuf {
        PathBuf::from(format!(
            "/tmp/bg-cwd-{label}-{}-{}.sock",
            std::process::id(),
            uuid::Uuid::new_v4().simple()
        ))
    }

    #[test]
    fn spawn_cli_laisse_l_executant_refuser_le_cwd_absent_et_garde_le_cas_valide() {
        let root = PathBuf::from("/tmp").join(format!("bgcwd-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&root).expect("racine de test");
        crate::environment::ensure_private_directory(
            &crate::environment::root_for_home(&root).unwrap(),
        )
        .unwrap();
        let cwd_absent = root.join("absent-sur-le-client");
        assert!(!cwd_absent.exists(), "précondition : cwd client absent");
        // Le registre d'ordres appartient au namespace, pas au cwd. Deux
        // passes du banc ne doivent pas réutiliser une clé pour deux cwd.
        let absent_id = format!("cwd-absent-{}", uuid::Uuid::new_v4());
        let valid_id = format!("cwd-valide-{}", uuid::Uuid::new_v4());
        let args_absents = vec![
            "fixture".to_string(),
            "--cwd".to_string(),
            cwd_absent.to_string_lossy().into_owned(),
            "--command-id".to_string(),
            absent_id.clone(),
            "--no-persistent".to_string(),
        ];

        // Le mutant qui réintroduit `cwd.is_dir()` tombe ICI, avant toute
        // socket : la CLI doit transmettre le chemin à la machine exécutante.
        let ordre_absent = resolve_spawn_order(
            &parse_spawn_args(&args_absents).expect("arguments valides"),
            100,
            &root,
            &root,
        )
        .expect("la CLI ne tranche pas l'existence d'un cwd exécutant");
        let socket_absent = temporary_socket("absent");
        let daemon_absent = daemon_repond_a_un_spawn(
            &socket_absent,
            DaemonToWrapper::SpawnRejected {
                command_id: absent_id.clone(),
                // DEUX machines ATTESTÉES et DIFFÉRENTES, imposées au banc :
                // c'est la seule configuration où le rendu peut prouver qu'il
                // distingue « où l'on a cherché » de « qui a demandé ». Des
                // valeurs recalculées par le code de production ne prouveraient
                // que la recopie.
                reason: SpawnRefusal::CwdGone {
                    searched_on: "machine-executante".to_string(),
                    requested_from: "machine-demandeuse".to_string(),
                },
            },
        );
        let refusal = send_control_to_daemon_at(&socket_absent, ordre_absent)
            .expect("la CLI reçoit le refus de l'exécuteur");
        let reason = match refusal {
            DaemonToWrapper::SpawnRejected { command_id, reason } => {
                assert_eq!(command_id, absent_id);
                reason
            }
            other => panic!("SpawnRejected attendu, reçu {other:?}"),
        };
        // Le texte HISTORIQUE appartenait aux deux issues : il ne prouvait plus
        // rien depuis que le refus nomme les machines. On exige donc le rendu
        // EXACT du nouveau contrat — les deux noms, chacun à sa place.
        assert_eq!(
            format!("SPAWN REFUSÉ: {}", display_spawn_refusal(&reason)),
            "SPAWN REFUSÉ: répertoire de travail introuvable sur machine-executante, \
             demandé depuis machine-demandeuse",
            "le texte doit venir de CwdGone et nommer les DEUX machines, pas du pré-contrôle client"
        );
        assert!(matches!(
            daemon_absent.join().expect("daemon absent termine"),
            WrapperToDaemon::SpawnOrder { cwd, .. } if cwd == cwd_absent.to_string_lossy()
        ));
        let _ = std::fs::remove_file(&socket_absent);

        let cwd_valide = root.join("present-sur-le-client");
        std::fs::create_dir_all(&cwd_valide).expect("cwd valide");
        let args_valides = vec![
            "fixture".to_string(),
            "--cwd".to_string(),
            cwd_valide.to_string_lossy().into_owned(),
            "--command-id".to_string(),
            valid_id.clone(),
            "--no-persistent".to_string(),
        ];
        let ordre_valide = resolve_spawn_order(
            &parse_spawn_args(&args_valides).expect("arguments valides"),
            100,
            &root,
            &root,
        )
        .expect("un cwd valide construit toujours un ordre");
        let socket_valide = temporary_socket("valide");
        let daemon_valide = daemon_repond_a_un_spawn(
            &socket_valide,
            DaemonToWrapper::SpawnAccepted {
                command_id: valid_id.clone(),
                agent_id: "fixture-cwd".to_string(),
                definition: None,
            },
        );
        assert!(matches!(
            send_control_to_daemon_at(&socket_valide, ordre_valide),
            Ok(DaemonToWrapper::SpawnAccepted { command_id, agent_id, .. })
                if command_id == valid_id && agent_id == "fixture-cwd"
        ));
        assert!(matches!(
            daemon_valide.join().expect("daemon valide termine"),
            WrapperToDaemon::SpawnOrder { cwd, .. } if cwd == cwd_valide.to_string_lossy()
        ));
        let _ = std::fs::remove_file(&socket_valide);
        std::fs::remove_file(spawn_order_path(&root, &absent_id)).unwrap();
        std::fs::remove_file(spawn_order_path(&root, &valid_id)).unwrap();
        let _ = std::fs::remove_dir_all(root);
    }
}

fn validate_retry_options(
    parsed: &ParsedSpawnArgs,
    stored: &WrapperToDaemon,
) -> Result<(), String> {
    let WrapperToDaemon::SpawnOrder {
        posture,
        agent_type,
        agent_id: name,
        cwd,
        persistent,
        ..
    } = stored
    else {
        return Err("le command_id mémorisé n'est pas un ordre spawn".to_string());
    };
    if &parsed.agent_type != agent_type
        || parsed.posture.is_some_and(|value| Some(value) != *posture)
        || parsed
            .agent_id
            .as_ref()
            .is_some_and(|value| Some(value) != name.as_ref())
        || parsed
            .cwd
            .as_ref()
            .is_some_and(|value| value.to_string_lossy() != cwd.as_str())
        // Le choix de survie étant désormais obligatoire, `persistent_was_set`
        // est toujours vrai : la comparaison doit porter sur la valeur choisie
        // dans les DEUX sens. Ne tester que `!persistent` laisserait rejouer
        // un ordre persistant sous --no-persistent sans détecter la divergence.
        || (parsed.persistent_was_set && parsed.persistent != *persistent)
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
    crate::environment::root_for_home(home)
        .expect("namespace validé avant la lecture des ordres")
        .join("spawn-orders")
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

/// Une machine vide vient d'un daemon antérieur au champ : on le DIT, on ne
/// suppose pas la machine locale.
fn machine_ou_non_attestee(host: &str) -> &str {
    // Rendu UNIQUE : un champ vide et un champ portant la valeur de repli
    // disent le meme fait et doivent se lire pareil.
    if host.trim().is_empty() {
        crate::build_info::MACHINE_NON_ATTESTEE
    } else {
        crate::build_info::describe_host(Some(host))
    }
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
        // Le refus dit OÙ il a cherché et QUI a demandé : sans cela, un
        // opérateur fédéré cherche le répertoire du mauvais côté du tunnel.
        SpawnRefusal::CwdGone {
            searched_on,
            requested_from,
        } => {
            let searched_on = machine_ou_non_attestee(searched_on);
            let requested_from = machine_ou_non_attestee(requested_from);
            format!(
                "répertoire de travail introuvable sur {searched_on}, demandé depuis {requested_from}"
            )
        }
        SpawnRefusal::ProjectCwdMismatch { project_id } => {
            format!("répertoire de travail hors de la liaison du projet '{project_id}'")
        }
        SpawnRefusal::DockerRuntimeUnavailable { project_id } => {
            format!(
                "lancement Docker indisponible pour le projet {project_id}; aucun repli hote n a ete execute"
            )
        }
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
        return Err("agent_id d'équipier manquant".to_string());
    };
    validate_agent_id(agent)?;
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

    if args[0].chars().any(char::is_control)
        || args[0].chars().count() > crate::agent_profile::MAX_DISPLAY_NAME_CHARS
    {
        exit_argument_error("nom affiché invalide (80 caractères maximum, sans contrôle)");
    }
    let identity = resolve_command_identity("rename");
    match crate::communication::client::rename_display_name(
        &identity.name,
        &identity.instance_id,
        &socket_path(),
        &args[0],
    ) {
        Ok(DaemonToWrapper::DisplayNameResult {
            outcome:
                bridget_transport::protocol::DisplayNameOutcome::Applied {
                    agent_id,
                    display_name,
                    revision,
                },
        }) => {
            println!(
                "{}",
                serde_json::json!({"agent_id":agent_id,"display_name":display_name,"revision":revision})
            );
        }
        Ok(response) => {
            eprintln!(
                "bridget rename : {}",
                encode(&response).expect("réponse sérialisable")
            );
            std::process::exit(1);
        }
        Err(error) => {
            eprintln!("bridget rename : résultat non confirmé : {error:?}");
            std::process::exit(1);
        }
    }
}

fn resolve_command_identity(command: &str) -> crate::mcp_identity::ResolvedIdentity {
    crate::mcp_identity::resolve_current_identity().unwrap_or_else(|error| {
        eprintln!(
            "bridget {command} : {} : {}",
            error.code(),
            error.remediation()
        );
        std::process::exit(1);
    })
}

fn current_agent_id() -> String {
    let file_agent_id = std::env::var("BRIDGET_AGENT_ID_FILE")
        .ok()
        .and_then(|path| std::fs::read_to_string(path).ok());
    resolve_cli_agent_id(
        file_agent_id.as_deref(),
        std::env::var("BRIDGET_AGENT_ID").ok().as_deref(),
        || {
            crate::mcp_identity::resolve_current_identity()
                .ok()
                .map(|identity| identity.name)
        },
    )
}

/// Identité qui signe un message CLI, par ordre de preuve : fichier puis
/// variable posés par un lanceur Bridget, puis filiation de processus attestée.
/// Sans aucune des trois, on n'invente pas d'identité d'équipier — le daemon
/// conserve l'UUID du CLI temporaire.
///
/// Session 115 : un fil T3 n'est pas lancé par un lanceur Bridget et n'a donc
/// ni le fichier ni la variable. Sa connexion au daemon était pourtant attestée
/// par filiation (`send_control_to_daemon_at`) ; seul le message signait
/// « human ». Le daemon voyait alors une connexion attestée prétendre parler au
/// nom d'un humain, et refusait l'usurpation. Signer par la même preuve que la
/// connexion rend les deux cohérents. La filiation n'est consultée qu'en
/// dernier recours : un agent lancé par Bridget garde sa preuve habituelle.
fn resolve_cli_agent_id(
    file_agent_id: Option<&str>,
    env_agent_id: Option<&str>,
    lineage_agent_id: impl FnOnce() -> Option<String>,
) -> String {
    if let Some(agent_id) = file_agent_id
        .map(str::trim)
        .filter(|agent_id| validate_agent_id(agent_id).is_ok())
    {
        return agent_id.to_string();
    }
    if let Some(agent_id) = env_agent_id
        .map(str::trim)
        .filter(|agent_id| validate_agent_id(agent_id).is_ok())
    {
        return agent_id.to_string();
    }
    if let Some(agent_id) = lineage_agent_id()
        .map(|agent_id| agent_id.trim().to_string())
        .filter(|agent_id| validate_agent_id(agent_id).is_ok())
    {
        return agent_id;
    }
    "human".to_string()
}

fn cmd_daemon(args: &[String]) {
    if let Some(argument) = args.first() {
        exit_argument_error(&unknown_argument("daemon", argument));
    }
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
            "--to" => match option_value(args, &mut i, "--to") {
                Ok(value) => to = Some(value),
                Err(error) => send_usage_error(&error),
            },
            "--from" => match option_value(args, &mut i, "--from") {
                Ok(value) => from = Some(value),
                Err(error) => send_usage_error(&error),
            },
            "--reply" => {
                reply = true;
            }
            "--timeout" => match positive_integer_option(args, &mut i, "--timeout") {
                Ok(value) => timeout_secs = Some(value),
                Err(error) => exit_argument_error(&error),
            },
            "--hops" => match positive_integer_option(args, &mut i, "--hops") {
                Ok(value) => hops = value,
                Err(error) => exit_argument_error(&error),
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
            "--in-reply-to" => match option_value(args, &mut i, "--in-reply-to") {
                Ok(value) => in_reply_to = Some(value),
                Err(error) => send_usage_error(&error),
            },
            "--" => {
                body_parts.extend(args[i + 1..].iter().cloned());
                break;
            }
            unknown if unknown.starts_with('-') => {
                exit_argument_error(&format!("argument non reconnu: {unknown}"));
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
            eprintln!(
                "usage: bridget send --to <agent_id> [--from <agent_id>] [--in-reply-to ID] [--reply] [--hops N] [--] <message>"
            );
            std::process::exit(2);
        }
    };

    // Le transport ne connaît que les principaux opaques : aucun alias
    // historique ne peut être accepté par la CLI.
    if let Err(e) = validate_agent_id(&to) {
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

    // Se nommer est un acte explicite : le daemon doit pouvoir le distinguer
    // d'un nom de repli, sans quoi il ne sait pas s'il a le droit de le
    // remplacer par l'identité éphémère de la connexion.
    let from_declared = from.is_some();
    let sender = from.unwrap_or_else(current_agent_id);
    if sender != "human"
        && let Err(error) = validate_agent_id(&sender)
    {
        exit_argument_error(&error);
    }
    if let Err(error) = validate_reply_options(&sender, reply, timeout_secs) {
        exit_argument_error(&error);
    }
    let mut msg = BridgetMessage::new(&sender, &to, &body);
    msg.from_declared = from_declared;
    msg.in_reply_to = in_reply_to;
    msg.reply = reply;
    msg.hops = hops;
    if reply {
        msg.reply_timeout = Some(timeout_secs.unwrap_or(60));
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
                let reply_str = if reply { " [réponse attendue]" } else { "" };
                println!(
                    "OK: envoyé à « {} » (id={}, hops={}){}",
                    to, id, hops, reply_str
                );
                println!("    ↳ Vérifie : « {} » est bien le destinataire voulu.", to);
            }
            DaemonToWrapper::Nack { id: _, reason } => {
                eprintln!("REJET: {}", reason);
                if let Some(hint) = unattested_hint(
                    &msg.from,
                    &reason,
                    crate::mcp_identity::resolve_current_identity().map(|_| ()),
                ) {
                    eprintln!("{hint}");
                }
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

fn positive_integer_option<T>(args: &[String], index: &mut usize, option: &str) -> Result<T, String>
where
    T: std::str::FromStr + PartialOrd + From<u8>,
{
    let value = option_value(args, index, option)?;
    let invalid =
        || format!("{option}: valeur « {value} » invalide (entier strictement positif attendu)");
    let parsed = value.parse::<T>().map_err(|_| invalid())?;
    if parsed <= T::from(0) {
        return Err(invalid());
    }
    Ok(parsed)
}

/// Session 116 : un refus faute d'identité ne reste plus muet. Il dit d'où la
/// preuve aurait dû venir et pourquoi la filiation n'a pas abouti — c'est ce
/// silence qui a rendu la panne du 24/09 si longue à diagnostiquer.
fn unattested_hint(
    sender: &str,
    reason: &str,
    lineage: Result<(), crate::mcp_identity::IdentityError>,
) -> Option<String> {
    if sender != "human" || !reason.contains("attest") {
        return None;
    }
    let cause = match lineage {
        Ok(()) => "la filiation aboutit maintenant : relancer la commande".to_string(),
        Err(error) => format!("{} : {}", error.code(), error.remediation()),
    };
    Some(format!(
        "    ↳ Envoyé sans identité d'agent : ni variable d'un lanceur Bridget, ni filiation attestée ({cause})."
    ))
}

fn validate_reply_options(
    sender: &str,
    reply_requested: bool,
    timeout_secs: Option<u64>,
) -> Result<(), String> {
    if sender == "human" && reply_requested {
        return Err(
            "--reply ne peut pas être utilisé avec l’expéditeur « human » : aucune réponse ne peut lui être livrée"
                .to_string(),
        );
    }
    if timeout_secs.is_some() && !reply_requested {
        return Err("--timeout requiert --reply".to_string());
    }
    Ok(())
}

fn cmd_guichet(args: &[String]) {
    let request = match parse_guichet_deposit(args) {
        Ok(request) => request,
        Err(error) => {
            eprintln!("erreur: {error}");
            eprintln!(
                "usage: bridget guichet deposer <delivery-report|mission-status|deadline-question|delegate|registre-add|objective-close> [options]"
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
    let mut goal = None;
    let mut explicit_target = None;
    let mut required_tags = Vec::new();
    let mut duration = None;
    let mut suite_objective_id = None;
    let mut depends_on = Vec::new();
    let mut references = Vec::new();
    let mut line = None;
    let mut reason = None;
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
            "--goal" => goal = Some(value),
            "--target" => explicit_target = Some(value),
            "--tag" => required_tags.push(value),
            "--duration" => duration = Some(value),
            "--suite-objective" => suite_objective_id = Some(value),
            "--depends-on" => depends_on.push(value),
            "--reference" => references.push(value),
            "--line" => line = Some(value),
            "--reason" => reason = Some(value),
            _ => return Err(format!("option guichet inconnue: {option}")),
        }
        index += 1;
    }

    let from = from.unwrap_or_else(current_agent_id);
    if from == "human" {
        return Err("--from est requis hors wrapper Bridget".to_string());
    }
    validate_agent_id(&from)?;
    let retry = idempotent_options(id, issued_at, issuer_scope)?;
    let scope_identity =
        std::env::var("BRIDGET_AGENT_INSTANCE_ID").unwrap_or_else(|_| from.clone());
    let (request_id, issued_at, issuer_scope) = retry
        .map(|retry| (retry.id, retry.issued_at, retry.issuer_scope))
        .unwrap_or_else(|| {
            (
                uuid::Uuid::new_v4().to_string(),
                unix_timestamp(),
                crate::communication::issuer_scope(&scope_identity),
            )
        });
    if kind != "delivery-report" && verdict.is_some() {
        return Err("--verdict est réservé à delivery-report".to_string());
    }
    if !matches!(kind, "delivery-report" | "delegate")
        && (review_ref.is_some() || expected_head.is_some())
    {
        return Err(
            "--review-ref et --expected-head sont réservés à delivery-report ou delegate"
                .to_string(),
        );
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
        "delegate" => {
            let review_target = parse_review_target(review_ref, expected_head)?;
            let duration = match duration.as_deref().unwrap_or("normale") {
                "courte" => GuichetDurationClass::Courte,
                "normale" => GuichetDurationClass::Normale,
                "longue" => GuichetDurationClass::Longue,
                _ => return Err("--duration doit valoir courte, normale ou longue".to_string()),
            };
            let suite = suite_objective_id
                .map_or(ServiceSuiteDeclaration::Aucune, |objective_id| {
                    ServiceSuiteDeclaration::Objectif { objective_id }
                });
            (
                ServiceRequestOperation::Delegate,
                ServiceRequestPayload::Delegate {
                    goal: goal.ok_or_else(|| "--goal est requis".to_string())?,
                    review_target,
                    explicit_target,
                    required_tags,
                    duration,
                    suite,
                    depends_on,
                    references,
                    origin: None,
                    focus: None,
                },
            )
        }
        "registre-add" => (
            ServiceRequestOperation::RegistreAdd,
            ServiceRequestPayload::RegistreAdd {
                line: line.ok_or_else(|| "--line est requis".to_string())?,
            },
        ),
        "objective-close" => (
            ServiceRequestOperation::ObjectiveClose,
            ServiceRequestPayload::ObjectiveClose {
                objective_id: objective_id.ok_or_else(|| "--objective est requis".to_string())?,
                reason: reason.ok_or_else(|| "--reason est requis".to_string())?,
            },
        ),
        _ => return Err(format!("type de dépôt fermé inconnu: {kind}")),
    };
    let version = payload.required_contract_version();
    Ok(WrapperToDaemon::ServiceRequest {
        version,
        issuer_scope,
        request_id,
        issued_at,
        from,
        to: "guichet".to_string(),
        operation,
        payload,
    })
}

fn observe_review_verdict(
    verdict: Option<String>,
    review_ref: Option<String>,
    expected_head: Option<String>,
) -> Result<Option<ReviewVerdictEvidence>, String> {
    let review_target = parse_review_target(review_ref, expected_head)?;
    let (verdict, target) = match (verdict, review_target) {
        (None, None) => return Ok(None),
        (Some(verdict), Some(target)) => (verdict, target),
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
    let (remote, branch) = target.remote_and_branch().expect("cible déjà validée");

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

fn parse_review_target(
    review_ref: Option<String>,
    expected_head: Option<String>,
) -> Result<Option<ReviewTarget>, String> {
    let target = match (review_ref, expected_head) {
        (None, None) => return Ok(None),
        (Some(target_ref), Some(expected_head)) => ReviewTarget {
            target_ref,
            expected_head,
        },
        _ => {
            return Err(
                "--review-ref et --expected-head doivent être fournis ensemble".to_string(),
            );
        }
    };
    if target.remote_and_branch().is_none() {
        return Err("--review-ref attend <remote>/<branche> valide".to_string());
    }
    if !is_canonical_git_sha(&target.expected_head) {
        return Err("--expected-head attend exactement 40 hexadécimaux minuscules".to_string());
    }
    Ok(Some(target))
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
        issuer_scope: crate::communication::issuer_scope(&instance_id),
    }))
}

fn send_usage_error(error: &str) -> ! {
    eprintln!("erreur: {error}");
    eprintln!(
        "usage: bridget send --to <nom> [--in-reply-to ID] [--id <clé> --issued-at <unix> [--issuer-scope <portée>]] [--] <message>"
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
        IdempotencyIssue::Orphaned {
            delivery_id,
            reason,
            ..
        } => eprintln!(
            "ISSUE: {} id={} issued_at={} delivery_id={delivery_id} — {} ; {} ({reason})",
            crate::mcp::STATUT_ORPHELIN,
            options.id,
            options.issued_at,
            crate::mcp::DIAGNOSTIC_ORPHELIN,
            crate::mcp::CONDUITE_ORPHELIN,
        ),
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
    let identity = match crate::mcp_identity::resolve_current_identity() {
        Ok(identity) => Some(identity),
        // Un humain au clavier n'a pas d'identité d'agent à prouver : il parle
        // sous l'étiquette humaine, que le daemon n'assimile jamais à un agent.
        Err(_) if message.from == "human" => None,
        Err(_) => {
            return Err(
                "identité auxiliaire indisponible : lancez la commande depuis le wrapper"
                    .to_string(),
            );
        }
    };
    send_idempotent_to_daemon_at(&socket_path(), message, options, identity.as_ref())
}

fn send_idempotent_to_daemon_at(
    path: &std::path::Path,
    message: &BridgetMessage,
    options: &IdempotentSendOptions,
    identity: Option<&crate::mcp_identity::ResolvedIdentity>,
) -> Result<DaemonToWrapper, String> {
    let mut connection = DaemonConnection::connect(path).map_err(|error| error.to_string())?;
    match connection
        .exchange(&WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Client,
        })
        .map_err(|error| error.to_string())?
    {
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client,
        } => {}
        response => return Err(format!("handshake client refusé: {response:?}")),
    }

    if let Some(identity) = identity {
        if identity.name != message.from {
            return Err("identité expéditeur différente du wrapper appelant".into());
        }
        crate::communication::client::authenticate_auxiliary(
            &mut connection,
            &identity.name,
            &identity.instance_id,
            path,
        )
        .map_err(|error| error.to_string())?;
    }

    match connection
        .exchange(&WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: options.issuer_scope.clone(),
            capabilities: vec![ClientCapability::SendIdempotent],
        })
        .map_err(|error| error.to_string())?
    {
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

    connection
        .send_then_wait(&WrapperToDaemon::SendIdempotent {
            message: message.clone(),
            message_id: options.id.clone(),
            issued_at: options.issued_at,
        })
        .map_err(|error| error.to_string())
}

#[cfg(test)]
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

#[cfg(test)]
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

/// Enregistrement d'une connexion CLI — source UNIQUE des trois usages.
///
/// Le champ `host` était `None` sur les trois : le daemon lisait ensuite
/// `requested_from` dans `conn_hosts` et n'y trouvait rien, si bien qu'un refus
/// de lancement ne pouvait pas nommer la machine demandeuse. Trois copies du
/// même bloc, c'est trois occasions d'oublier la même chose ; il n'y en a plus
/// qu'une.
fn cli_register(_usage: &str) -> WrapperToDaemon {
    WrapperToDaemon::Register {
        agent_type: "cli".to_string(),
        identity_version: 2,
        agent_id: uuid::Uuid::new_v4().to_string(),
        host: Some(crate::build_info::local_host()),
        transport: None,
        channel: bridget_transport::ChannelReport::Unknown,
        mode: Some(PresenceMode::Cli),
        location: None,
        os: None,
        instance_id: None,
        domain: None,
        turn_in_progress: false,
        journal_available: None,
    }
}

fn send_control_to_daemon_at(
    socket: &std::path::Path,
    command: WrapperToDaemon,
) -> Result<DaemonToWrapper, String> {
    let mut connection = DaemonConnection::connect(socket).map_err(|e| e.to_string())?;
    let reg = match crate::mcp_identity::resolve_current_identity() {
        Ok(identity) => crate::mcp_identity::auxiliary_registration(
            &identity.name,
            &identity.instance_id,
            socket,
        )?,
        Err(_) => cli_register("send"),
    };
    if !matches!(
        connection.exchange(&reg).map_err(|e| e.to_string())?,
        DaemonToWrapper::Registered { .. }
    ) {
        return Err("enregistrement CLI refusé ; aucune commande envoyée".into());
    }
    connection
        .send_then_wait(&command)
        .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CancelArgs {
    id: String,
    reason: Option<String>,
}

fn parse_cancel_args(args: &[String]) -> Result<CancelArgs, String> {
    let id = match args.first() {
        Some(argument) if argument.starts_with('-') => {
            return Err(unknown_argument("cancel", argument));
        }
        Some(id) => id.clone(),
        None => return Err("cancel: identifiant manquant".to_string()),
    };
    let reason = match args.get(1).map(String::as_str) {
        None => None,
        Some("--reason") => Some(
            args.get(2)
                .cloned()
                .ok_or_else(|| "cancel: --reason requiert une valeur".to_string())?,
        ),
        Some(argument) => return Err(unknown_argument("cancel", argument)),
    };
    if let Some(argument) = args.get(if reason.is_some() { 3 } else { 1 }) {
        return Err(unknown_argument("cancel", argument));
    }
    Ok(CancelArgs { id, reason })
}

fn cmd_cancel(args: &[String]) {
    let parsed = parse_cancel_args(args).unwrap_or_else(|error| {
        eprintln!("bridget {error}");
        eprintln!("usage: bridget cancel <id> [--reason <texte>]");
        std::process::exit(2);
    });
    let identity = crate::mcp_identity::resolve_current_identity().unwrap_or_else(|error| {
        eprintln!(
            "bridget cancel : {} : {}",
            error.code(),
            error.remediation()
        );
        std::process::exit(1);
    });
    match crate::communication::client::cancel_request(
        &identity.name,
        &identity.instance_id,
        &socket_path(),
        &parsed.id,
        parsed.reason,
    ) {
        Ok(DaemonToWrapper::RequestCancelled { state, .. }) => {
            println!("Demande #{} : {}", parsed.id, state)
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
            sender: current_agent_id(),
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
    // Sans délai borné, un daemon qui ne répond pas — par exemple un daemon
    // d'une version antérieure qui ignore ce message — bloquerait le hook, donc
    // la fin de tour de l'agent observé. Constaté en test réel.
    let mut connection = DaemonConnection::connect_until(
        &socket_path(),
        std::time::Instant::now() + Duration::from_secs(RUNTIME_REPLY_TIMEOUT_SECS),
    )
    .map_err(|e| e.to_string())?;
    let register = cli_register("runtime");
    if !matches!(
        connection.exchange(&register).map_err(|e| e.to_string())?,
        DaemonToWrapper::Registered { .. }
    ) {
        return Err("enregistrement runtime refusé".into());
    }

    let runtime = WrapperToDaemon::Runtime {
        agent: agent.to_string(),
        model: model.to_string(),
        effort: effort.map(str::to_owned),
        source,
    };
    connection.exchange(&runtime).map_err(|e| e.to_string())
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

    let identity = resolve_command_identity("runtime");
    match crate::communication::client::declare_runtime(
        &identity.name,
        &identity.instance_id,
        &socket_path(),
        model.clone(),
        effort.clone(),
    ) {
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum HookCommand {
    ClaudeRuntime,
    ClaudeStatusline,
    Unknown(String),
}

fn parse_hook_args(args: &[String]) -> Result<HookCommand, String> {
    let Some(name) = args.first() else {
        return Err("hook: nom manquant".to_string());
    };
    match name.as_str() {
        "claude-runtime" | "claude-statusline" if args.len() > 1 => {
            Err(unknown_argument("hook", &args[1]))
        }
        "claude-runtime" => Ok(HookCommand::ClaudeRuntime),
        "claude-statusline" => Ok(HookCommand::ClaudeStatusline),
        _ => Ok(HookCommand::Unknown(name.clone())),
    }
}

/// Commande appelée par un hook d'agent, jamais par un humain.
///
/// Exception délibérée à la règle générale de refus : un NOM de hook inconnu
/// reste fail-soft, avec stdout vide et code 0, car faire échouer un hook
/// perturberait la session observée (FR-013). Les deux noms connus gardent en
/// revanche une arité stricte afin de ne jamais ignorer un argument.
fn cmd_hook(args: &[String]) {
    match parse_hook_args(args).unwrap_or_else(|error| {
        eprintln!("bridget {error}");
        eprintln!("usage: bridget hook <claude-runtime|claude-statusline>");
        std::process::exit(2);
    }) {
        HookCommand::ClaudeRuntime => hook_claude_runtime(),
        HookCommand::ClaudeStatusline => hook_claude_statusline(),
        HookCommand::Unknown(name) => {
            log::debug!("hook inconnu: {}", name);
        }
    }
}

fn hook_claude_runtime() {
    // Hors d'un agent Bridget, le hook est inerte : les sessions Claude
    // ordinaires de l'utilisateur ne doivent subir aucun effet.
    let agent = current_agent_id();
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
    let agent = current_agent_id();
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
    // Même garde que la sonde de runtime : un daemon d'une version antérieure
    // ignore ce message, et sans délai borné le hook bloquerait le
    // rafraîchissement de la ligne d'état de l'agent observé.
    let mut connection = DaemonConnection::connect_until(
        socket,
        std::time::Instant::now() + Duration::from_secs(RUNTIME_REPLY_TIMEOUT_SECS),
    )
    .map_err(|e| e.to_string())?;

    for fact in facts {
        let message = WrapperToDaemon::RateLimit {
            agent: agent.to_string(),
            window: fact.window.clone(),
            status: STATUSLINE_LIMIT_STATUS.to_string(),
            resets_at: fact.resets_at,
            used_percent: fact.used_percent,
            source: bridget_transport::protocol::RateLimitSource::ClaudeStatusLine,
        };
        match connection.exchange(&message).map_err(|e| e.to_string())? {
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

fn parse_install_hooks_args(args: &[String]) -> Result<bool, String> {
    match args {
        [] => Ok(false),
        [option] if option == "--remove" => Ok(true),
        [option, argument, ..] if option == "--remove" => {
            Err(unknown_argument("install-hooks", argument))
        }
        [argument, ..] => Err(unknown_argument("install-hooks", argument)),
    }
}

fn cmd_install_hooks(args: &[String]) {
    let remove = parse_install_hooks_args(args).unwrap_or_else(|error| {
        eprintln!("bridget {error}");
        eprintln!("usage: bridget install-hooks [--remove]");
        std::process::exit(2);
    });
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

fn parse_domain_args(args: &[String]) -> Result<Option<String>, String> {
    match args {
        [option] if option == "--reset" => Ok(None),
        [domain] if !domain.starts_with('-') => {
            validate_technical_label(domain).map_err(|reason| format!("domain: {reason}"))?;
            Ok(Some(domain.clone()))
        }
        [] => Err("domain: nom ou --reset requis".to_string()),
        [first, argument, ..] if first == "--reset" || !first.starts_with('-') => {
            Err(unknown_argument("domain", argument))
        }
        [argument, ..] => Err(unknown_argument("domain", argument)),
    }
}

fn cmd_domain(args: &[String]) {
    let domain = parse_domain_args(args).unwrap_or_else(|error| {
        eprintln!("bridget {error}");
        eprintln!("usage: bridget domain <nom> | bridget domain --reset");
        std::process::exit(2);
    });

    let identity = resolve_command_identity("domain");
    match crate::communication::client::set_domain(
        &identity.name,
        &identity.instance_id,
        &socket_path(),
        domain.clone(),
    ) {
        Ok(DaemonToWrapper::Ack { .. }) => match &domain {
            Some(domain) => {
                println!("Domaine de « {} » : {}", identity.name, domain);
            }
            None => {
                println!(
                    "Domaine de « {} » réinitialisé sur le dépôt courant.",
                    identity.name
                );
            }
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

/// Durée de sécurité appliquée à un « ne pas déranger » sans échéance précisée.
const DND_DEFAULT_MINUTES: u64 = crate::communication::client::DND_DEFAULT_SECS / 60;

/// Interprète une durée de la forme `90s`, `30m` ou `2h`.
fn parse_duration(value: &str) -> Result<Duration, String> {
    crate::communication::client::parse_dnd_duration_secs(value)
        .map(Duration::from_secs)
        .map_err(|error| match error {
            crate::communication::client::ClientError::InvalidParams(reason) => reason,
            crate::communication::client::ClientError::Technical { message, .. } => message,
        })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DndArgs {
    Enable(Option<Duration>),
    Disable,
}

fn parse_dnd_args(args: &[String]) -> Result<DndArgs, String> {
    match args {
        [] => Ok(DndArgs::Enable(None)),
        [value] if value == "off" => Ok(DndArgs::Disable),
        [option, value] if option == "--duration" => parse_duration(value)
            .map(|duration| DndArgs::Enable(Some(duration)))
            .map_err(|reason| format!("dnd: {reason}")),
        [option] if option == "--duration" => {
            Err("dnd: --duration requiert une valeur".to_string())
        }
        [option, value, argument, ..] if option == "--duration" => {
            parse_duration(value).map_err(|reason| format!("dnd: {reason}"))?;
            Err(unknown_argument("dnd", argument))
        }
        [first, argument, ..] if first == "off" => Err(unknown_argument("dnd", argument)),
        [argument, ..] => Err(unknown_argument("dnd", argument)),
    }
}

fn cmd_dnd(args: &[String]) {
    let parsed = parse_dnd_args(args).unwrap_or_else(|error| {
        eprintln!("bridget {error}");
        eprintln!("usage: bridget dnd [off] [--duration 30m]");
        std::process::exit(2);
    });
    let (lift, duration) = match parsed {
        DndArgs::Enable(duration) => (false, duration),
        DndArgs::Disable => (true, None),
    };

    let duration_secs = if lift {
        None
    } else {
        Some(
            duration
                .unwrap_or(Duration::from_secs(
                    crate::communication::client::DND_DEFAULT_SECS,
                ))
                .as_secs(),
        )
    };

    let identity = resolve_command_identity("dnd");
    match crate::communication::client::set_dnd(
        &identity.name,
        &identity.instance_id,
        &socket_path(),
        duration_secs,
    ) {
        Ok(DaemonToWrapper::Ack { .. }) => match duration_secs {
            Some(_) => {
                let minutes = duration
                    .map(|d| d.as_secs().div_ceil(60))
                    .unwrap_or(DND_DEFAULT_MINUTES);
                println!(
                    "« {} » ne sera pas dérangé pendant {} min. Levée : bridget dnd off",
                    identity.name, minutes
                );
            }
            None => println!("« {} » est à nouveau joignable.", identity.name),
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
            "--timeout" => match positive_integer_option(args, &mut i, "--timeout") {
                Ok(value) => timeout_secs = Some(value),
                Err(error) => exit_argument_error(&error),
            },
            "--hops" => match positive_integer_option(args, &mut i, "--hops") {
                Ok(value) => hops = value,
                Err(error) => exit_argument_error(&error),
            },
            "--in-reply-to" => match option_value(args, &mut i, "--in-reply-to") {
                Ok(value) => explicit_in_reply_to = Some(value),
                Err(error) => {
                    eprintln!("reply: {error}");
                    eprintln!(
                        "usage: bridget reply [--in-reply-to ID] [--reply] [--hops N] [--] <message>"
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
            "--" => {
                body_parts.extend(args[i + 1..].iter().cloned());
                break;
            }
            unknown if unknown.starts_with('-') => {
                exit_argument_error(&format!("argument non reconnu: {unknown}"));
            }
            _ => {
                body_parts.push(args[i].clone());
            }
        }
        i += 1;
    }

    let agent_id = current_agent_id();
    if let Err(error) = validate_reply_options(&agent_id, reply_flag, timeout_secs) {
        exit_argument_error(&error);
    }
    let reply_file = socket_path()
        .parent()
        .unwrap()
        .join(format!("last-sender-{agent_id}"));
    let previous = match std::fs::read_to_string(&reply_file) {
        Ok(content) => content.trim().to_string(),
        Err(_) => {
            eprintln!("reply: aucun expediteur precedent trouve.");
            eprintln!("  (utilise 'bridget send --to <nom> \"message\"')");
            std::process::exit(1);
        }
    };
    // Session 102 : après une alerte de fil, le dernier contexte est un
    // marqueur typé ; répondre en direct viserait l'alerte ou un ancien DM.
    if let Some(thread_id) = thread_notice_marker(&previous) {
        println!(
            "{}",
            serde_json::json!({
                "status": "error",
                "code": "thread_notice_not_replyable",
                "detail": format!("La dernière remise est une alerte du fil {thread_id} : lis-la avec « bridget thread read {thread_id} » et publie avec « bridget thread post » ; un message direct reste possible avec « bridget send --to <UUID> »."),
                "retryable": false,
            })
        );
        std::process::exit(2);
    }
    let mut previous_parts = previous.splitn(2, '\t');
    let to = previous_parts.next().unwrap_or_default().to_string();
    let implicit_in_reply_to = previous_parts.next().map(str::to_string);
    if to.is_empty() {
        eprintln!("reply: expediteur precedent vide.");
        std::process::exit(1);
    }

    let body = body_parts.join(" ");
    if body.is_empty() {
        eprintln!("usage: bridget reply [--in-reply-to ID] [--reply] [--hops N] [--] <message>");
        std::process::exit(2);
    }

    // Validation du corps du message (H-001)
    if let Err(e) = validate_message_body(&body) {
        eprintln!("erreur: {}", e);
        std::process::exit(2);
    }

    let sender = agent_id.clone();

    let mut msg = BridgetMessage::new(&sender, &to, &body);
    msg.in_reply_to = explicit_in_reply_to.or(implicit_in_reply_to);
    msg.reply = reply_flag;
    msg.hops = hops;
    if reply_flag {
        msg.reply_timeout = Some(timeout_secs.unwrap_or(60));
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
                if let Some(hint) = unattested_hint(
                    &msg.from,
                    &reason,
                    crate::mcp_identity::resolve_current_identity().map(|_| ()),
                ) {
                    eprintln!("{hint}");
                }
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct DirectoryArgs {
    json: bool,
    domain: Option<String>,
}

fn parse_directory_args(
    command: &str,
    args: &[String],
    allow_json: bool,
) -> Result<DirectoryArgs, String> {
    let mut parsed = DirectoryArgs {
        json: false,
        domain: None,
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" if allow_json && !parsed.json => parsed.json = true,
            "--json" if allow_json => {
                return Err(format!("{command}: option dupliquée: --json"));
            }
            "--domain" if parsed.domain.is_none() => {
                index += 1;
                let domain = args
                    .get(index)
                    .filter(|value| !value.starts_with('-'))
                    .cloned()
                    .ok_or_else(|| format!("{command}: --domain requiert une valeur"))?;
                parsed.domain = Some(domain);
            }
            "--domain" => return Err(format!("{command}: option dupliquée: --domain")),
            argument => return Err(unknown_argument(command, argument)),
        }
        index += 1;
    }
    Ok(parsed)
}

fn daemon_status_or_exit(command: &str, config: &DaemonConfig) -> daemon::DaemonStatus {
    let status = daemon::get_status(config).unwrap_or_else(|error| {
        eprintln!("bridget {command}: {error}");
        std::process::exit(1);
    });
    if !status.agents_inventory_available {
        eprintln!(
            "bridget {command}: daemon présent, inventaire indisponible — aucune liste vide n'est attestée"
        );
        std::process::exit(1);
    }
    status
}

fn cmd_agents(args: &[String]) {
    let parsed = parse_directory_args("agents", args, true).unwrap_or_else(|error| {
        eprintln!("bridget {error}");
        eprintln!("usage: bridget agents [--json] [--domain <nom>]");
        std::process::exit(2);
    });

    let config = DaemonConfig::default();
    let mut status = daemon_status_or_exit("agents", &config);
    if let Some(domain) = &parsed.domain {
        status
            .agents
            .retain(|agent| agent.domain.as_deref() == Some(domain.as_str()));
    }
    if !status.running {
        if parsed.json {
            println!("[]");
        } else {
            eprintln!("daemon non demarre (socket absente)");
        }
        std::process::exit(1);
    }

    if parsed.json {
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
                "  {} ({}) [{}] — {} / {} via {} (canal {}) — {} / {} [{}] — persiste {}",
                agent.display_name,
                agent.agent_type,
                cell(agent.domain.as_deref()),
                agent.host,
                agent.os,
                agent.transport,
                cell(agent.channel.as_deref()),
                format_model(agent),
                cell(agent.effort.as_deref()),
                agent.state,
                format_persistent(agent)
            );
        }
    }
}

// ---------------------------------------------------------------------------
// SPEC-087 : `bridget control` et `bridget inbox`. Les mutations exigent un
// terminal interactif, même borne que l'approbation d'activation le service compagnon.
// ---------------------------------------------------------------------------

fn send_control_request(request: WrapperToDaemon) -> Result<DaemonToWrapper, String> {
    let mut connection = DaemonConnection::connect(&socket_path()).map_err(|e| e.to_string())?;
    match connection
        .exchange(&WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Client,
        })
        .map_err(|e| e.to_string())?
    {
        DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client,
        } => {}
        response => return Err(format!("handshake control refusé: {response:?}")),
    }
    match connection
        .exchange(&WrapperToDaemon::ClientHello {
            contract_version: CLIENT_CONTRACT_VERSION,
            issuer_scope: crate::communication::issuer_scope("bridget-control-cli"),
            capabilities: vec![ClientCapability::ControlStateV1],
        })
        .map_err(|e| e.to_string())?
    {
        DaemonToWrapper::ClientWelcome { capabilities, .. }
            if capabilities.contains(&ClientCapability::ControlStateV1) => {}
        DaemonToWrapper::ClientRejected { reason } => {
            return Err(format!("négociation control refusée: {reason:?}"));
        }
        response => return Err(format!("négociation control refusée: {response:?}")),
    }
    connection
        .send_then_wait(&request)
        .map_err(|e| e.to_string())
}

fn require_interactive_terminal(command: &str) -> bool {
    use std::io::IsTerminal;
    if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
        return true;
    }
    eprintln!("bridget {command}: contrôle du référent = terminal interactif uniquement");
    false
}

fn fetch_control_state() -> Result<(bridget_transport::protocol::ControlStateFrame, u32), String> {
    match send_control_request(WrapperToDaemon::ControlStateRead {
        version: bridget_transport::protocol::CONTROL_STATE_CONTRACT_VERSION,
    })? {
        DaemonToWrapper::ControlState {
            state,
            inbox_open_count,
        } => Ok((state, inbox_open_count)),
        DaemonToWrapper::ControlStateRejected { reason } => Err(format!("{reason:?}")),
        response => Err(format!("réponse inattendue: {response:?}")),
    }
}

fn control_footer_line(
    state: &bridget_transport::protocol::ControlStateFrame,
    inbox_open_count: u32,
    now: i64,
) -> String {
    let mut line = crate::referent_control::summary_line(state, now);
    if inbox_open_count > 0 {
        line.push_str(&format!(
            " · {inbox_open_count} décision{} en attente",
            if inbox_open_count > 1 { "s" } else { "" }
        ));
    }
    line
}

/// Pied de `bridget who` : silencieux si le daemon ne sait pas encore répondre,
/// pour ne jamais casser l'annuaire sur un daemon antérieur à SPEC-087.
fn emit_control_footer() {
    if let Ok((state, inbox_open_count)) = fetch_control_state() {
        println!(
            "{}",
            control_footer_line(&state, inbox_open_count, unix_now_secs_cli())
        );
    }
}

fn unix_now_secs_cli() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs().min(i64::MAX as u64) as i64)
        .unwrap_or(0)
}

fn print_control_status(
    state: &bridget_transport::protocol::ControlStateFrame,
    inbox_open_count: u32,
) {
    println!(
        "{}",
        control_footer_line(state, inbox_open_count, unix_now_secs_cli())
    );
    println!("génération : {}", state.generation);
    println!(
        "posture : {}",
        state
            .agent_posture
            .map(|posture| posture.as_sql())
            .unwrap_or("inconnue")
    );
    if state.paused {
        println!(
            "pause posée par {} · motif : {}",
            state.paused_by.as_deref().unwrap_or("inconnu"),
            state.pause_reason.as_deref().unwrap_or("aucun")
        );
    }
}

fn apply_control_mutation(
    command: &str,
    paused: Option<bool>,
    auto_objectives_cap: Option<u32>,
    reason: Option<String>,
    agent_posture: Option<bridget_transport::protocol::AgentPosture>,
) {
    if !require_interactive_terminal(command) {
        std::process::exit(2);
    }
    let (current, _) = fetch_control_state().unwrap_or_else(|error| {
        eprintln!("bridget control: {error}");
        std::process::exit(1);
    });
    let response = send_control_request(WrapperToDaemon::ControlStateSet {
        version: bridget_transport::protocol::CONTROL_STATE_CONTRACT_VERSION,
        command_id: format!("control-cli-{}", uuid::Uuid::new_v4()),
        expected_generation: current.generation,
        paused,
        auto_objectives_cap,
        reason,
        agent_posture,
        auto_reassignment: None,
    });
    match response {
        Ok(DaemonToWrapper::ControlState {
            state,
            inbox_open_count,
        }) => print_control_status(&state, inbox_open_count),
        Ok(DaemonToWrapper::ControlStateRejected { reason }) => {
            eprintln!("bridget control: refusé: {reason:?}");
            std::process::exit(1);
        }
        Ok(response) => {
            eprintln!("bridget control: réponse inattendue: {response:?}");
            std::process::exit(1);
        }
        Err(error) => {
            eprintln!("bridget control: {error}");
            std::process::exit(1);
        }
    }
}

fn parse_control_posture(
    args: &[String],
) -> Result<bridget_transport::protocol::AgentPosture, String> {
    match args {
        [value] => bridget_transport::protocol::AgentPosture::from_sql(value).ok_or_else(|| {
            "control posture : attendu discovery ou complete, sans substitution".to_string()
        }),
        _ => Err("usage: bridget control posture discovery|complete".to_string()),
    }
}

fn cmd_control(args: &[String]) {
    let usage = "usage: bridget control status [--history] | pause [--reason <T>] | resume | budget <N> | posture discovery|complete";
    match args.first().map(String::as_str) {
        Some("status") => {
            let (state, inbox_open_count) = fetch_control_state().unwrap_or_else(|error| {
                eprintln!("bridget control: {error}");
                std::process::exit(1);
            });
            print_control_status(&state, inbox_open_count);
            if args.iter().any(|argument| argument == "--history") {
                match send_control_request(WrapperToDaemon::ControlHistory {
                    version: bridget_transport::protocol::CONTROL_STATE_CONTRACT_VERSION,
                    limit: 50,
                }) {
                    Ok(DaemonToWrapper::ControlHistory { events }) => {
                        if events.is_empty() {
                            println!("historique : aucune mutation");
                        }
                        for event in events {
                            println!(
                                "  {} · {} · {} · génération {}{}",
                                event.at,
                                event.actor,
                                event.kind,
                                event.generation_after,
                                event
                                    .reason
                                    .map(|reason| format!(" · {reason}"))
                                    .unwrap_or_default()
                            );
                        }
                    }
                    Ok(response) => {
                        eprintln!("bridget control: historique indisponible: {response:?}")
                    }
                    Err(error) => eprintln!("bridget control: {error}"),
                }
            }
        }
        Some("pause") => {
            let reason = args
                .iter()
                .position(|argument| argument == "--reason")
                .and_then(|index| args.get(index + 1))
                .cloned();
            apply_control_mutation("control pause", Some(true), None, reason, None);
        }
        Some("resume") => apply_control_mutation("control resume", Some(false), None, None, None),
        Some("budget") => {
            let cap = args.get(1).and_then(|value| value.parse::<u32>().ok());
            let Some(cap) = cap else {
                eprintln!("{usage}");
                std::process::exit(2);
            };
            apply_control_mutation("control budget", None, Some(cap), None, None);
        }
        Some("posture") => {
            let posture = parse_control_posture(&args[1..])
                .unwrap_or_else(|error| exit_argument_error(&error));
            apply_control_mutation("control posture", None, None, None, Some(posture));
        }
        _ => {
            eprintln!("{usage}");
            std::process::exit(2);
        }
    }
}

fn cmd_inbox(args: &[String]) {
    use bridget_transport::protocol::{HUMAN_INBOX_CONTRACT_VERSION, HumanInboxListFilter};
    let usage = "usage: bridget inbox list [--all] | resolve <ID> <CHOIX>";
    match args.first().map(String::as_str) {
        Some("list") => {
            let filter = if args.iter().any(|argument| argument == "--all") {
                HumanInboxListFilter::All
            } else {
                HumanInboxListFilter::Open
            };
            match send_control_request(WrapperToDaemon::HumanInboxList {
                version: HUMAN_INBOX_CONTRACT_VERSION,
                state: filter,
                limit: 100,
            }) {
                Ok(DaemonToWrapper::HumanInbox { items, open_count }) => {
                    println!("{open_count} décision(s) en attente");
                    for item in items {
                        println!(
                            "  {} · {} · {:?} · ×{} · options : {}",
                            item.id,
                            item.kind.as_sql(),
                            item.state,
                            item.occurrences,
                            item.options.join(", ")
                        );
                        if let Ok(context) =
                            serde_json::from_str::<serde_json::Value>(&item.context)
                            && let Some(summary) = context.get("summary").and_then(|v| v.as_str())
                        {
                            println!("      {summary}");
                        }
                    }
                }
                Ok(DaemonToWrapper::HumanInboxRejected { reason }) => {
                    eprintln!("bridget inbox: refusé: {reason:?}");
                    std::process::exit(1);
                }
                Ok(response) => {
                    eprintln!("bridget inbox: réponse inattendue: {response:?}");
                    std::process::exit(1);
                }
                Err(error) => {
                    eprintln!("bridget inbox: {error}");
                    std::process::exit(1);
                }
            }
        }
        Some("resolve") => {
            let (Some(item_id), Some(choice)) = (args.get(1), args.get(2)) else {
                eprintln!("{usage}");
                std::process::exit(2);
            };
            if !require_interactive_terminal("inbox resolve") {
                std::process::exit(2);
            }
            match send_control_request(WrapperToDaemon::HumanInboxResolve {
                version: HUMAN_INBOX_CONTRACT_VERSION,
                command_id: format!("inbox-cli-{}", uuid::Uuid::new_v4()),
                item_id: item_id.clone(),
                choice: choice.clone(),
            }) {
                Ok(DaemonToWrapper::HumanInbox { items, open_count }) => {
                    for item in items {
                        println!(
                            "{} tranché « {} » · {open_count} décision(s) restante(s)",
                            item.id,
                            item.decision.map(|d| d.choice).unwrap_or_default()
                        );
                    }
                }
                Ok(DaemonToWrapper::HumanInboxRejected { reason }) => {
                    eprintln!("bridget inbox: refusé: {reason:?}");
                    std::process::exit(1);
                }
                Ok(response) => {
                    eprintln!("bridget inbox: réponse inattendue: {response:?}");
                    std::process::exit(1);
                }
                Err(error) => {
                    eprintln!("bridget inbox: {error}");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("{usage}");
            std::process::exit(2);
        }
    }
}

fn cmd_who(args: &[String]) {
    let parsed = parse_directory_args("who", args, false).unwrap_or_else(|error| {
        eprintln!("bridget {error}");
        eprintln!("usage: bridget who [--domain <nom>]");
        std::process::exit(2);
    });
    let config = DaemonConfig::default();
    let status = daemon_status_or_exit("who", &config);
    if !status.running {
        eprintln!("daemon non démarré (socket absente)");
        std::process::exit(1);
    }

    let build_id = status.build_id.as_deref().unwrap_or("inconnu");
    let agents: Vec<_> = match &parsed.domain {
        Some(domain) => status
            .agents
            .into_iter()
            .filter(|agent| agent.domain.as_deref() == Some(domain.as_str()))
            .collect(),
        None => status.agents,
    };

    print!("{}", render_who(&agents, parsed.domain.as_deref()));
    println!("Daemon build-id: {build_id}");
    emit_control_footer();
    emit_stale_daemon_warning(status.build_id.as_deref(), status.daemon_host.as_deref());
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
    let name_w = column("NOM", &|a: &AgentInfo| a.display_name.clone());
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
    let disk_w = column("DISQUE", &|a: &AgentInfo| format_disk_space(a));
    let persistent_w = column("PERSIST", &|a: &AgentInfo| format_persistent(a).to_string());

    let mut output = String::new();
    match filter {
        Some(domain) => writeln!(output, "Agents du domaine « {} » :", domain).unwrap(),
        None => writeln!(output, "Agents connectés :").unwrap(),
    }
    writeln!(
        output,
        "  {:<name_w$}  {:<type_w$}  {:<host_w$}  {:<os_w$}  {:<transport_w$}  {:<channel_w$}  {:<mode_w$}  {:<location_w$}  {:<domain_w$}  {:<model_w$}  {:<effort_w$}  {:<rate_limit_w$}  {:<disk_w$}  {:<persistent_w$}  ÉTAT",
        "NOM", "TYPE", "HÔTE", "OS", "TRANSPORT", "CANAL", "MODE", "LOCALISATION", "DOMAINE", "MODÈLE", "EFFORT", "LIMITE", "DISQUE", "PERSIST"
    )
    .unwrap();
    for agent in agents {
        writeln!(
            output,
            "  {:<name_w$}  {:<type_w$}  {:<host_w$}  {:<os_w$}  {:<transport_w$}  {:<channel_w$}  {:<mode_w$}  {:<location_w$}  {:<domain_w$}  {:<model_w$}  {:<effort_w$}  {:<rate_limit_w$}  {:<disk_w$}  {:<persistent_w$}  {}",
            agent.display_name,
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
            format_disk_space(agent),
            format_persistent(agent),
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

/// Survie au redémarrage du service, telle qu'attestée par la flotte.
///
/// `—` n'est pas un « non » : il dit qu'aucune entrée de flotte ne couvre cet
/// agent, donc que rien ne le drainera au redémarrage. Confondre les deux
/// ferait croire à une disparition programmée et pousserait à relancer un
/// agent sain, c'est-à-dire à détruire son contexte.
fn format_persistent(agent: &AgentInfo) -> &'static str {
    match agent.persistent {
        Some(true) => "oui",
        Some(false) => "non",
        None => "—",
    }
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

/// Affiche la dernière photographie d'espace libre, sans en déduire une
/// pression disque ni une action. L'absence reste explicitement inconnue.
fn format_disk_space(agent: &AgentInfo) -> String {
    agent
        .disk_space
        .as_ref()
        .map(|fact| {
            format!(
                "{:.1} Gio libres",
                fact.free_bytes as f64 / 1024_f64.powi(3)
            )
        })
        .unwrap_or_else(|| "—".to_string())
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
    let snapshot =
        crate::reprise::collect_snapshot(&config, &repo, pin_ref, std::time::SystemTime::now());
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
            "usage: bridget reaper report --tmp DIR [--json] [--state-dir DIR] [--min-age-secs N]"
        );
        eprintln!("phase observer uniquement — aucune action destructive n'existe");
        std::process::exit(2);
    }
    let mut json_output = false;
    let mut state_dir = crate::reaper::default_state_dir();
    let mut tmp_dir = None;
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
                Ok(value) => tmp_dir = Some(PathBuf::from(value)),
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

    let tmp_dir = require_explicit_reaper_tmp(tmp_dir).unwrap_or_else(|error| {
        eprintln!("bridget reaper: {error}");
        eprintln!(
            "usage: bridget reaper report --tmp DIR [--json] [--state-dir DIR] [--min-age-secs N]"
        );
        std::process::exit(2);
    });

    crate::environment::validate_existing_tree(&state_dir)
        .unwrap_or_else(|error| exit_argument_error(&error));
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

/// Une racine explicite évite de regarder silencieusement `TMPDIR` de la
/// machine courante, qui peut être un sous-répertoire sans rapport avec les
/// arbres de travail à examiner.
fn require_explicit_reaper_tmp(tmp_dir: Option<PathBuf>) -> Result<PathBuf, String> {
    tmp_dir.ok_or_else(|| "--tmp DIR est obligatoire : aucune racine TMPDIR implicite".to_string())
}

fn cmd_status() {
    let config = DaemonConfig::default();
    let status = daemon_status_or_exit("status", &config);
    println!(
        "Daemon: {}",
        if status.running {
            "en ligne"
        } else {
            "hors ligne"
        }
    );
    let machine = crate::build_info::describe_host(status.daemon_host.as_deref());
    println!("Machine du daemon: {machine}");
    println!("Socket: {}", config.socket_path.display());
    // La base affichée est celle que le DAEMON atteste, jamais le chemin que ce
    // client calculerait pour lui-même : les deux divergent dès que le daemon
    // est au bout d'un tunnel.
    println!(
        "Base de données du daemon: {}",
        status
            .daemon_db_path
            .as_deref()
            .unwrap_or(crate::build_info::MACHINE_NON_ATTESTEE)
    );
    println!("Agents connectés: {}", status.agents.len());
    match status.message_count {
        Some(count) => println!("Messages en base: {count}"),
        None => println!("Messages en base: total non publié par le daemon — consulter ledger"),
    }
    println!(
        "Build-id daemon: {}",
        status.build_id.as_deref().unwrap_or("inconnu")
    );
    emit_stale_daemon_warning(status.build_id.as_deref(), status.daemon_host.as_deref());
    emit_disk_warning();
}

fn stale_daemon_warning_for_status(
    build_id: Option<&str>,
    daemon_host: Option<&str>,
) -> Option<String> {
    crate::build_info::stale_daemon_warning_at(build_id.unwrap_or("unknown"), daemon_host)
}

fn emit_stale_daemon_warning(build_id: Option<&str>, daemon_host: Option<&str>) {
    if let Some(warning) = stale_daemon_warning_for_status(build_id, daemon_host) {
        eprintln!("{warning}");
    }
}

fn emit_disk_warning() {
    if let Some(warning) = crate::disk_hygiene::disk_warning_for_display(Path::new("/")) {
        eprintln!("{warning}");
    }
}

/// Borne d'affichage par défaut du ledger : le maximum que la projection
/// accepte de lire.
///
/// Le CLI figeait 20 alors que le protocole porte déjà un champ `limit` et que
/// la projection lit jusqu'à `MAX_LEDGER_PROJECTION`. Cette borne n'était
/// écrite nulle part dans la sortie : à 94 messages par heure, elle réduisait
/// la vue à treize minutes sans le dire. Une borne subie est pire qu'une borne
/// étroite — celle-ci est désormais la plus large possible, et surtout elle
/// s'annonce.
const DEFAULT_LEDGER_LIMIT: usize = crate::ledger::MAX_LEDGER_PROJECTION;

#[cfg(test)]
mod ledger_borne_tests {
    use super::*;

    fn message(index: usize) -> LedgerMessage {
        LedgerMessage {
            id: format!("m{index}"),
            // Décroissant : la projection rend `ORDER BY ts DESC`.
            ts: 1_000 - index as i64,
            sender: "bridget".into(),
            target: "jc1-flux".into(),
            body: format!("message {index}"),
            delivery_status: None,
        }
    }

    /// ORACLE — une vue bornée DIT qu'elle est bornée.
    ///
    /// Le CLI figeait 20 sans l'écrire nulle part : à 94 messages par heure,
    /// il ne montrait pas les vingt derniers messages, il montrait le dernier
    /// quart d'heure. Un message humain du 27/08 est ainsi sorti de la vue du
    /// référent en 17 min 52 s et a attendu huit heures. Une borne silencieuse
    /// se lit comme « il n'y a rien d'autre ».
    #[test]
    fn vue_bornee_declare_ce_qu_elle_ne_montre_pas() {
        // Une entrée de plus que la borne : c'est ainsi que le surplus se
        // détecte, sans rien demander de plus au daemon.
        let entrees: Vec<_> = (0..4).map(message).collect();
        let rendu = render_ledger_borne(&entrees, 3);

        assert!(
            rendu.contains("message 0") && rendu.contains("message 2"),
            "les trois plus récents doivent être rendus : {rendu}"
        );
        assert!(
            !rendu.contains("message 3"),
            "le surplus ne doit pas être affiché : {rendu}"
        );
        assert!(
            rendu.contains("vue bornée à 3"),
            "la troncature doit être déclarée, pas subie : {rendu}"
        );
        assert!(
            rendu.contains("--limit"),
            "le refus doit dire comment élargir : {rendu}"
        );
    }

    /// ORACLE — sans surplus, aucune mention parasite : une vue complète ne
    /// doit pas faire croire qu'il manque quelque chose.
    #[test]
    fn vue_complete_n_annonce_aucune_troncature() {
        let entrees: Vec<_> = (0..3).map(message).collect();
        let rendu = render_ledger_borne(&entrees, 20);
        assert!(rendu.contains("message 2"));
        assert!(
            !rendu.contains("vue bornée"),
            "aucune troncature à déclarer ici : {rendu}"
        );
        assert!(
            !rendu.contains("maximum lisible"),
            "le maximum n'est pas atteint : {rendu}"
        );
    }

    /// ORACLE — au maximum lisible, l'ignorance est déclarée plutôt que tue.
    /// La projection borne à `MAX_LEDGER_PROJECTION` : au-delà, le CLI ne peut
    /// pas savoir s'il existe des messages plus anciens. Se taire ferait passer
    /// cette ignorance pour une absence.
    #[test]
    fn maximum_lisible_declare_l_ignorance_au_lieu_de_la_taire() {
        let entrees: Vec<_> = (0..DEFAULT_LEDGER_LIMIT).map(message).collect();
        let rendu = render_ledger_borne(&entrees, DEFAULT_LEDGER_LIMIT);
        assert!(
            rendu.contains("maximum lisible"),
            "atteindre le plafond doit se dire : {rendu}"
        );
        assert!(
            rendu.contains("n'est pas observable"),
            "l'ignorance doit être nommée, pas déguisée en absence : {rendu}"
        );
    }

    /// ORACLE — la borne est réglable, et le défaut n'est plus 20.
    #[test]
    fn borne_reglable_et_defaut_au_maximum_lisible() {
        assert_eq!(parse_ledger_args(&[]).unwrap(), DEFAULT_LEDGER_LIMIT);
        assert_eq!(DEFAULT_LEDGER_LIMIT, crate::ledger::MAX_LEDGER_PROJECTION);
        assert_ne!(
            DEFAULT_LEDGER_LIMIT, 20,
            "le défaut figé à 20 était le défaut mesuré"
        );
        assert_eq!(
            parse_ledger_args(&["--limit".into(), "5".into()]).unwrap(),
            5
        );

        // Un dépassement est refusé et NOMME le plafond, au lieu d'être raboté
        // en silence par la projection.
        let refus = parse_ledger_args(&["--limit".into(), "10000".into()])
            .expect_err("au-delà du maximum lisible doit être refusé");
        assert!(refus.contains(&DEFAULT_LEDGER_LIMIT.to_string()), "{refus}");
        assert!(parse_ledger_args(&["--limit".into(), "0".into()]).is_err());
        assert!(parse_ledger_args(&["--limit".into()]).is_err());
        assert!(parse_ledger_args(&["--inconnu".into()]).is_err());
    }
}

fn parse_ledger_args(args: &[String]) -> Result<usize, String> {
    let mut limite = DEFAULT_LEDGER_LIMIT;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--limit" => {
                let valeur = args
                    .get(index + 1)
                    .ok_or_else(|| "valeur manquante pour --limit".to_string())?;
                let demande = valeur
                    .parse::<usize>()
                    .ok()
                    .filter(|nombre| *nombre >= 1)
                    .ok_or_else(|| {
                        "--limit doit être un entier supérieur ou égal à 1".to_string()
                    })?;
                // Refus explicite plutôt que rabotage silencieux : la projection
                // ramènerait la valeur à son maximum sans le dire, et l'appelant
                // croirait avoir demandé plus qu'il ne recevra.
                if demande > DEFAULT_LEDGER_LIMIT {
                    return Err(format!(
                        "--limit {demande} dépasse le maximum lisible ({DEFAULT_LEDGER_LIMIT})"
                    ));
                }
                limite = demande;
                index += 2;
            }
            option => return Err(format!("option ledger inconnue: {option}")),
        }
    }
    Ok(limite)
}

/// Rend la vue en disant ce qu'elle ne montre pas.
///
/// L'appelant demande une entrée de plus que la borne : si elle revient, des
/// messages plus anciens existent et la vue le déclare. Aucune modification du
/// protocole ni du daemon n'est nécessaire pour cela.
fn render_ledger_borne(entries: &[LedgerMessage], limite: usize) -> String {
    // Les entrées arrivent du plus récent au plus ancien (`ORDER BY ts DESC`) :
    // le surplus à écarter est en queue, pas en tête.
    let visibles = &entries[..entries.len().min(limite)];
    let mut rendu = render_ledger(visibles);
    if entries.len() > limite {
        rendu.push_str(&format!(
            "… vue bornée à {limite} : des messages plus anciens existent et ne sont pas montrés (élargir avec --limit N, maximum {DEFAULT_LEDGER_LIMIT}).\n"
        ));
    } else if limite >= DEFAULT_LEDGER_LIMIT && entries.len() >= DEFAULT_LEDGER_LIMIT {
        rendu.push_str(&format!(
            "… maximum lisible atteint ({DEFAULT_LEDGER_LIMIT}) : l'existence de messages plus anciens n'est pas observable par cette commande.\n"
        ));
    }
    rendu
}

const LEDGER_SEARCH_USAGE: &str = "usage: bridget ledger search --query \"termes\" [--source messages|thread] [--thread-id UUID]\n\
       [--peer UUID] [--author UUID] [--since UNIX] [--until UNIX] [--limit N] [--cursor HEX] [--json]\n\
       bridget ledger read --id ID --target UUID [--offset N --digest HEX] [--json]\n\
Recherche partielle et reprenable : recopier next_cursor avec la même query et les mêmes filtres.\n\
Codes : 0 succès (même sans résultat), 2 paramètres invalides, 1 refus ou erreur.";

/// Session 104 : options fermées de `ledger search`, sans état implicite.
fn parse_ledger_search_args(
    args: &[String],
) -> Result<(bridget_transport::protocol::LedgerSearchRequest, bool), String> {
    use bridget_transport::protocol::{LedgerSearchRequest, LedgerSearchSource};
    let mut request = LedgerSearchRequest {
        source: LedgerSearchSource::Messages,
        query: String::new(),
        author: None,
        peer: None,
        since: None,
        until: None,
        limit: None,
        cursor: None,
        thread_id: None,
    };
    let mut json = false;
    let mut query = None;
    let mut index = 0;
    let value = |index: &mut usize, name: &str| -> Result<String, String> {
        *index += 1;
        args.get(*index)
            .cloned()
            .ok_or_else(|| format!("{name} attend une valeur"))
    };
    while index < args.len() {
        match args[index].as_str() {
            "--json" => json = true,
            "--query" => query = Some(value(&mut index, "--query")?),
            "--source" => {
                request.source = match value(&mut index, "--source")?.as_str() {
                    "messages" => LedgerSearchSource::Messages,
                    "thread" => LedgerSearchSource::Thread,
                    other => return Err(format!("--source inconnu : {other} (messages|thread)")),
                }
            }
            "--thread-id" => request.thread_id = Some(value(&mut index, "--thread-id")?),
            "--peer" => request.peer = Some(value(&mut index, "--peer")?),
            "--author" => request.author = Some(value(&mut index, "--author")?),
            "--cursor" => request.cursor = Some(value(&mut index, "--cursor")?),
            "--since" | "--until" | "--limit" => {
                let name = args[index].clone();
                let raw = value(&mut index, &name)?;
                let parsed: i64 = raw
                    .parse()
                    .map_err(|_| format!("{name} attend un entier ≥ 0, reçu {raw}"))?;
                if parsed < 0 {
                    return Err(format!("{name} attend un entier ≥ 0"));
                }
                match name.as_str() {
                    "--since" => request.since = Some(parsed),
                    "--until" => request.until = Some(parsed),
                    _ => {
                        request.limit = Some(
                            u16::try_from(parsed).map_err(|_| "--limit trop grand".to_string())?,
                        )
                    }
                }
            }
            other => return Err(format!("option ledger search inconnue : {other}")),
        }
        index += 1;
    }
    request.query = query.ok_or_else(|| "--query est obligatoire".to_string())?;
    Ok((request, json))
}

fn parse_ledger_read_args(
    args: &[String],
) -> Result<(bridget_transport::protocol::LedgerReadRequest, bool), String> {
    let (mut id, mut target, mut offset, mut digest, mut json) = (None, None, 0u64, None, false);
    let mut index = 0;
    while index < args.len() {
        let name = args[index].as_str();
        if name == "--json" {
            json = true;
            index += 1;
            continue;
        }
        let raw = args
            .get(index + 1)
            .ok_or_else(|| format!("{name} attend une valeur"))?
            .clone();
        match name {
            "--id" => id = Some(raw),
            "--target" => target = Some(raw),
            "--digest" => digest = Some(raw),
            "--offset" => {
                offset = raw
                    .parse()
                    .map_err(|_| format!("--offset attend un entier ≥ 0, reçu {raw}"))?
            }
            other => return Err(format!("option ledger read inconnue : {other}")),
        }
        index += 2;
    }
    Ok((
        bridget_transport::protocol::LedgerReadRequest {
            id: id.ok_or_else(|| "--id est obligatoire".to_string())?,
            target: target.ok_or_else(|| "--target est obligatoire".to_string())?,
            offset,
            digest,
        },
        json,
    ))
}

/// Neutralise les caractères de contrôle (ANSI, retours) dans un rendu
/// terminal ; la donnée JSON n'est jamais modifiée par l'affichage.
pub(crate) fn inert_text(text: &str) -> String {
    text.chars()
        .map(|ch| if ch.is_control() { '\u{FFFD}' } else { ch })
        .collect()
}

pub(crate) fn render_ledger_search(page: &bridget_transport::protocol::LedgerSearchPage) -> String {
    use bridget_transport::protocol::LedgerSearchHit;
    let mut out = format!(
        "{} résultat(s) — source {} — {} candidat(s) parcourus, {} octets examinés{}\n",
        page.hits.len(),
        page.source.name(),
        page.scanned_count,
        page.scanned_bytes,
        if page.skipped_oversized > 0 {
            format!(", {} ignoré(s) > 16 Mio", page.skipped_oversized)
        } else {
            String::new()
        }
    );
    for hit in &page.hits {
        match hit {
            LedgerSearchHit::Message {
                id,
                target,
                sender,
                ts,
                excerpt,
                match_offset,
                body_digest,
                ..
            } => out.push_str(&format!(
                "  [{ts}] {} → {}  id={} offset={match_offset} digest={}\n      {}\n",
                inert_text(sender),
                inert_text(target),
                inert_text(id),
                &body_digest[..16],
                inert_text(excerpt)
            )),
            LedgerSearchHit::ThreadEntry {
                thread_id,
                seq,
                author_id,
                ts,
                excerpt,
                match_offset,
                ..
            } => out.push_str(&format!(
                "  [{ts}] fil {thread_id} seq={seq} auteur={} offset={match_offset}\n      {}\n",
                inert_text(author_id),
                inert_text(excerpt)
            )),
        }
    }
    match &page.next_cursor {
        Some(cursor) => out.push_str(&format!(
            "suite disponible ({}) : répéter la commande avec --cursor {cursor}\n",
            page.stop_reason
        )),
        None => out.push_str("fin de la partie conservée.\n"),
    }
    out
}

fn ledger_error_exit(code: &str, reason: &str, json: bool) -> ! {
    if json {
        println!(
            "{}",
            serde_json::json!({"status":"error","code":code,"reason":reason})
        );
    } else {
        eprintln!("bridget ledger : {code} : {}", inert_text(reason));
    }
    std::process::exit(if matches!(code, "invalid_params" | "invalid_cursor") {
        2
    } else {
        1
    });
}

fn ledger_identity() -> crate::mcp_identity::ResolvedIdentity {
    crate::mcp_identity::resolve_current_identity().unwrap_or_else(|error| {
        eprintln!(
            "bridget ledger : {} : {}",
            error.code(),
            error.remediation()
        );
        std::process::exit(1);
    })
}

fn cmd_ledger_search(args: &[String]) {
    let (request, json) = parse_ledger_search_args(args).unwrap_or_else(|error| {
        eprintln!("{LEDGER_SEARCH_USAGE}");
        eprintln!("erreur: {error}");
        std::process::exit(2);
    });
    let identity = ledger_identity();
    let outcome = crate::communication::client::ledger_search(
        &identity.name,
        &identity.instance_id,
        &socket_path(),
        request,
    )
    .unwrap_or_else(|error| match error {
        crate::communication::client::ClientError::InvalidParams(message) => {
            ledger_error_exit("invalid_params", &message, json)
        }
        crate::communication::client::ClientError::Technical { code, message } => {
            ledger_error_exit(code, &message, json)
        }
    });
    match outcome {
        bridget_transport::protocol::LedgerSearchOutcomeV1::Ok(page) => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string(&bridget_transport::protocol::LedgerSearchOutcomeV1::Ok(
                        page
                    ))
                    .expect("page sérialisable")
                );
            } else {
                print!("{}", render_ledger_search(&page));
            }
        }
        bridget_transport::protocol::LedgerSearchOutcomeV1::Error { code, reason } => {
            ledger_error_exit(&code, &reason, json)
        }
    }
}

fn cmd_ledger_read(args: &[String]) {
    let (request, json) = parse_ledger_read_args(args).unwrap_or_else(|error| {
        eprintln!("{LEDGER_SEARCH_USAGE}");
        eprintln!("erreur: {error}");
        std::process::exit(2);
    });
    let identity = ledger_identity();
    let outcome = crate::communication::client::ledger_read(
        &identity.name,
        &identity.instance_id,
        &socket_path(),
        request,
    )
    .unwrap_or_else(|error| match error {
        crate::communication::client::ClientError::InvalidParams(message) => {
            ledger_error_exit("invalid_params", &message, json)
        }
        crate::communication::client::ClientError::Technical { code, message } => {
            ledger_error_exit(code, &message, json)
        }
    });
    match outcome {
        bridget_transport::protocol::LedgerReadOutcomeV1::Ok(fragment) => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string(&bridget_transport::protocol::LedgerReadOutcomeV1::Ok(
                        fragment
                    ))
                    .expect("fragment sérialisable")
                );
            } else {
                println!(
                    "[{}] {} → {}  id={}  {} octets, digest={}\n{}\n{}",
                    fragment.ts,
                    inert_text(&fragment.sender),
                    inert_text(&fragment.target),
                    inert_text(&fragment.id),
                    fragment.body_bytes,
                    fragment.digest,
                    inert_text(&fragment.fragment),
                    match fragment.next_offset {
                        Some(next) =>
                            format!("suite : --offset {next} --digest {}", fragment.digest),
                        None => "fin du message.".to_string(),
                    }
                );
            }
        }
        bridget_transport::protocol::LedgerReadOutcomeV1::Error { code, reason } => {
            ledger_error_exit(&code, &reason, json)
        }
    }
}

fn cmd_ledger(args: &[String]) {
    match args.first().map(String::as_str) {
        Some("search") => return cmd_ledger_search(&args[1..]),
        Some("read") => return cmd_ledger_read(&args[1..]),
        _ => {}
    }
    let limite = parse_ledger_args(args).unwrap_or_else(|error| {
        eprintln!("bridget ledger: {error}");
        eprintln!("usage: bridget ledger [--limit N] | ledger search … | ledger read …");
        std::process::exit(2);
    });
    // Une de plus que la borne, pour savoir s'il y en a plus — jamais au-delà
    // de ce que la projection accepte.
    let demande = limite.saturating_add(1).min(DEFAULT_LEDGER_LIMIT);
    let messages = match send_control_to_daemon(WrapperToDaemon::LedgerProjection {
        scope: LedgerScope::Messages,
        // `demande` est borné par `DEFAULT_LEDGER_LIMIT` : la conversion ne
        // peut pas déborder, le repli n'existe que pour ne jamais paniquer.
        limit: u16::try_from(demande).unwrap_or(u16::MAX),
    }) {
        Ok(DaemonToWrapper::LedgerProjection { messages, .. }) => Ok(messages),
        Ok(DaemonToWrapper::Nack { reason, .. }) => Err(format!("erreur lecture ledger: {reason}")),
        Ok(_) => Err("réponse inattendue du daemon".to_string()),
        // Le maître est l'unique source, y compris au travers de SSH. Créer
        // une base cliente après une coupure fabriquerait un historique vide.
        Err(error) => Err(format!(
            "ledger indisponible : daemon inaccessible ({error})"
        )),
    };
    match messages {
        Ok(messages) => print!("{}", render_ledger_borne(&messages, limite)),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
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

    fn argv(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn reaper_exige_une_racine_explicitement_choisie() {
        assert!(
            require_explicit_reaper_tmp(None)
                .unwrap_err()
                .contains("--tmp DIR est obligatoire")
        );
        assert_eq!(
            require_explicit_reaper_tmp(Some(PathBuf::from("/tmp"))).unwrap(),
            PathBuf::from("/tmp")
        );
    }

    #[test]
    fn commandes_sans_arguments_refusent_le_surplus_qu_elles_recevront_en_production() {
        for command in [
            "mcp",
            "discover",
            "status",
            "version",
            "--version",
            "-v",
            "help",
            "--help",
            "-h",
        ] {
            let error = validate_zero_arity_command(command, &argv(&["SURPLUS"]))
                .expect_err("le surplus doit être refusé");
            assert!(error.contains(command), "commande absente de {error}");
            assert!(error.contains("SURPLUS"), "argument absent de {error}");
            assert!(validate_zero_arity_command(command, &[]).is_ok());
        }
        assert!(
            validate_zero_arity_command("daemon", &argv(&["--project-root-policy", "/tmp/policy"]))
                .is_ok()
        );
        // `ledger` a quitté cette grammaire en recevant `--limit` : il ne doit
        // plus être refusé en amont, sinon l'option n'atteindrait jamais son
        // parseur. Le refus du surplus lui reste dû, mais par `parse_ledger_args`.
        assert!(validate_zero_arity_command("ledger", &argv(&["--limit", "5"])).is_ok());
        assert!(parse_ledger_args(&argv(&["SURPLUS"])).is_err());
    }

    #[test]
    fn parseurs_structures_refusent_tous_le_premier_surplus() {
        let errors = [
            parse_cancel_args(&argv(&["--SURPLUS"])).unwrap_err(),
            parse_cancel_args(&argv(&["demande", "SURPLUS"])).unwrap_err(),
            parse_hook_args(&argv(&["claude-runtime", "SURPLUS"])).unwrap_err(),
            parse_install_hooks_args(&argv(&["--remove", "SURPLUS"])).unwrap_err(),
            parse_domain_args(&argv(&["revue", "SURPLUS"])).unwrap_err(),
            parse_dnd_args(&argv(&["off", "SURPLUS"])).unwrap_err(),
            parse_directory_args("agents", &argv(&["--json", "SURPLUS"]), true).unwrap_err(),
            parse_directory_args("who", &argv(&["--domain", "revue", "SURPLUS"]), false)
                .unwrap_err(),
        ];
        for error in errors {
            assert!(error.contains("SURPLUS"), "argument non nommé dans {error}");
        }
    }

    #[test]
    fn parseurs_structures_conservent_les_formes_documentees_et_le_hook_fail_soft() {
        assert_eq!(
            parse_cancel_args(&argv(&["demande", "--reason", "plus utile"])).unwrap(),
            CancelArgs {
                id: "demande".to_string(),
                reason: Some("plus utile".to_string()),
            }
        );
        assert_eq!(
            parse_hook_args(&argv(&["claude-runtime"])).unwrap(),
            HookCommand::ClaudeRuntime
        );
        assert_eq!(
            parse_hook_args(&argv(&["extension-future", "payload-opaque"])).unwrap(),
            HookCommand::Unknown("extension-future".to_string())
        );
        assert!(!parse_install_hooks_args(&[]).unwrap());
        assert!(parse_install_hooks_args(&argv(&["--remove"])).unwrap());
        assert_eq!(
            parse_domain_args(&argv(&["revue-croisee"])).unwrap(),
            Some("revue-croisee".to_string())
        );
        assert_eq!(parse_domain_args(&argv(&["--reset"])).unwrap(), None);
        assert_eq!(parse_dnd_args(&[]).unwrap(), DndArgs::Enable(None));
        assert_eq!(parse_dnd_args(&argv(&["off"])).unwrap(), DndArgs::Disable);
        assert_eq!(
            parse_dnd_args(&argv(&["--duration", "15m"])).unwrap(),
            DndArgs::Enable(Some(Duration::from_secs(15 * 60)))
        );
        assert_eq!(
            parse_directory_args("agents", &argv(&["--domain", "revue", "--json"]), true).unwrap(),
            DirectoryArgs {
                json: true,
                domain: Some("revue".to_string()),
            }
        );
    }

    #[test]
    fn spec116_refus_sans_identite_dit_pourquoi() {
        use crate::mcp_identity::IdentityError;
        let raison =
            "identité expéditeur non attestée : --from ne permet pas d'emprunter une identité";
        let indice = unattested_hint("human", raison, Err(IdentityError::IdentityNotFound))
            .expect("un refus d'identité est expliqué");
        assert!(indice.contains("identity_not_found"));
        assert!(indice.contains("agent Bridget enregistré"));
        assert!(
            unattested_hint(
                "human",
                "agent introuvable: x",
                Err(IdentityError::IdentityNotFound)
            )
            .is_none(),
            "un autre refus n'est pas maquillé en problème d'identité"
        );
        assert!(
            unattested_hint(
                "127bccff-8490-453a-8182-884b749ec41e",
                raison,
                Err(IdentityError::IdentityNotFound)
            )
            .is_none(),
            "un envoi signé n'a pas besoin de cet indice"
        );
    }

    #[test]
    fn repli_cli_n_accepte_que_l_identifiant_uuid_dans_l_environnement() {
        let first = "550e8400-e29b-41d4-a716-446655440000";
        let second = "550e8400-e29b-41d4-a716-446655440001";
        assert_eq!(resolve_cli_agent_id(None, Some(first), || None), first);
        assert_eq!(
            resolve_cli_agent_id(Some("  "), Some(second), || None),
            second
        );
        assert_eq!(
            resolve_cli_agent_id(Some(first), Some(second), || None),
            first
        );
        assert_eq!(resolve_cli_agent_id(None, None, || None), "human");
        assert_eq!(
            resolve_cli_agent_id(None, Some("legacy-name"), || None),
            "human"
        );
    }

    /// Session 115 : un fil T3 n'a ni fichier ni variable d'identité, mais sa
    /// filiation est attestée par le pont. Il signe désormais sous son identité
    /// au lieu de « human », que sa propre connexion attestée ne peut usurper.
    #[test]
    fn spec115_fil_t3_signe_par_sa_filiation_attestee() {
        let fil = "127bccff-8490-453a-8182-884b749ec41e";
        let lanceur = "550e8400-e29b-41d4-a716-446655440000";
        assert_eq!(
            resolve_cli_agent_id(None, None, || Some(fil.to_string())),
            fil,
            "la filiation attestée signe le message"
        );
        assert_eq!(
            resolve_cli_agent_id(None, None, || Some(format!("  {fil}\n"))),
            fil,
            "le contenu du fichier de nom est normalisé"
        );
        assert_eq!(
            resolve_cli_agent_id(None, None, || Some("nom-libre".to_string())),
            "human",
            "une filiation qui ne livre pas un UUID n'invente rien"
        );
        // Un agent lancé par Bridget garde sa preuve habituelle, et la
        // filiation n'est même pas consultée.
        assert_eq!(
            resolve_cli_agent_id(None, Some(lanceur), || panic!("filiation consultée")),
            lanceur
        );
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
    fn rendu_ledger_compte_une_ligne_par_message_apres_refus_d_un_nom_avec_lf() {
        let mut router = bridget_core::Router::new();
        router
            .register(
                "550e8400-e29b-41d4-a716-446655440000",
                &bridget_core::AgentType::Codex,
                "conn-1",
            )
            .unwrap();
        let _ = router.register(
            "invalid-agent-id",
            &bridget_core::AgentType::Codex,
            "conn-2",
        );
        let entries = router
            .list_agents()
            .into_iter()
            .map(|agent| LedgerMessage {
                id: format!("message-{}", agent.connection_id),
                ts: 42,
                sender: agent.agent_id.clone(),
                target: "victime".to_string(),
                body: "corps".to_string(),
                delivery_status: None,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            render_ledger(&entries).lines().count(),
            2,
            "l'en-tête et l'unique message légitime doivent former exactement deux lignes"
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
                sender: "guichet".to_string(),
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
        assert!(!participant.contains("guichet"));
        assert!(!participant.contains("prospective"));

        let global = render_requests(&requests, true);
        let global_header = global.lines().next().unwrap();
        assert!(global_header.split_whitespace().any(|cell| cell == "DE"));
        assert!(global_header.split_whitespace().any(|cell| cell == "VERS"));
        assert!(!global_header.contains("DESTINATAIRE"));
        assert!(global.contains("guichet") && global.contains("coderBridget"));
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
            parse_attach_args(&["89000000-0000-4000-8000-000000000011".to_string()]).unwrap(),
            (
                "89000000-0000-4000-8000-000000000011".to_string(),
                AttachWindow::Today
            )
        );
        assert_eq!(
            parse_attach_args(&[
                "89000000-0000-4000-8000-000000000011".to_string(),
                "--from-seq".to_string(),
                "42".to_string(),
            ])
            .unwrap(),
            (
                "89000000-0000-4000-8000-000000000011".to_string(),
                AttachWindow::Seq(42)
            )
        );
        assert_eq!(
            parse_attach_args(&[
                "89000000-0000-4000-8000-000000000011".to_string(),
                "--date".to_string(),
                "2026-08-22".to_string(),
            ])
            .unwrap(),
            (
                "89000000-0000-4000-8000-000000000011".to_string(),
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
                "89000000-0000-4000-8000-000000000011".to_string(),
                "--from-seq".to_string(),
                "pas-un-entier".to_string(),
            ])
            .is_err()
        );
        assert!(
            parse_attach_args(&[
                "89000000-0000-4000-8000-000000000011".to_string(),
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
            "--agent-id".to_string(),
            "550e8400-e29b-41d4-a716-446655440002".to_string(),
            "--no-persistent".to_string(),
        ])
        .unwrap();
        assert!(!parsed.persistent);
        assert_eq!(
            parsed.agent_id.as_deref(),
            Some("550e8400-e29b-41d4-a716-446655440002")
        );
        let warning = crate::recovery_trace::non_persistent_spawn_warning("cursor3");
        assert!(warning.contains("cursor3"));
        assert!(warning.contains("ne survivra pas au redémarrage"));
        assert!(warning.contains("sans --persistent"));
    }

    /// ORACLE — un spawn neuf REFUSE de partir tant que le sort de l'équipier
    /// n'est pas décidé, et le refus nomme les deux options.
    ///
    /// L'avertissement précédent était affiché puis ignoré cinq fois de suite
    /// le 28/08 : un avertissement qu'on peut ignorer n'est pas une garde.
    /// L'omission est irrattrapable, le `cwd` d'un équipier non persistant
    /// n'existant plus nulle part une fois connecté.
    #[test]
    fn spawn_neuf_refuse_de_partir_sans_choix_de_survie() {
        let root = PathBuf::from("/tmp").join(format!("bgsurv-{}", uuid::Uuid::new_v4().simple()));
        let cwd = root.join("work");
        std::fs::create_dir_all(&cwd).unwrap();
        crate::environment::ensure_private_directory(
            &crate::environment::root_for_home(&root).unwrap(),
        )
        .unwrap();
        let neuf = |options: &[&str]| {
            let mut args = vec!["codex".to_string()];
            args.extend(options.iter().map(|option| option.to_string()));
            resolve_spawn_order(
                &parse_spawn_args(&args).expect("arguments analysables"),
                100,
                &cwd,
                &root,
            )
        };

        let refus = neuf(&["--agent-id", "89000000-0000-4000-8000-000000000012"])
            .expect_err("un spawn muet doit être refusé");
        assert!(
            refus.contains("--persistent") && refus.contains("--no-persistent"),
            "le refus doit nommer les deux options : {refus}"
        );
        assert!(
            refus.contains("irrattrapable"),
            "le refus doit dire pourquoi le choix ne peut pas être différé : {refus}"
        );

        match neuf(&[
            "--agent-id",
            "89000000-0000-4000-8000-000000000013",
            "--persistent",
        ])
        .expect("choix explicite accepté")
        {
            WrapperToDaemon::SpawnOrder { persistent, .. } => assert!(persistent),
            other => panic!("ordre inattendu: {other:?}"),
        }
        match neuf(&[
            "--agent-id",
            "89000000-0000-4000-8000-000000000014",
            "--no-persistent",
        ])
        .expect("choix explicite accepté")
        {
            WrapperToDaemon::SpawnOrder { persistent, .. } => assert!(!persistent),
            other => panic!("ordre inattendu: {other:?}"),
        }

        // Deux drapeaux opposés ne valent pas décision : les départager par
        // « le dernier gagne » restaurerait le défaut silencieux supprimé.
        let contradiction = parse_spawn_args(&[
            "codex".to_string(),
            "--persistent".to_string(),
            "--no-persistent".to_string(),
        ])
        .expect_err("intention contradictoire refusée");
        assert!(contradiction.contains("contradictoires"), "{contradiction}");

        let _ = std::fs::remove_dir_all(root);
    }

    /// ORACLE — le rejeu d'un `--command-id` mémorisé reste possible SANS
    /// redéclarer le choix : l'enveloppe figée le porte déjà. Une garde qui
    /// bloquerait ce rappel bloquerait un lancement légitime.
    #[test]
    fn rejeu_d_un_ordre_memorise_n_exige_pas_de_redeclarer_la_survie() {
        let root = PathBuf::from("/tmp").join(format!("bgretry-{}", uuid::Uuid::new_v4().simple()));
        let cwd = root.join("work");
        std::fs::create_dir_all(&cwd).unwrap();
        crate::environment::ensure_private_directory(
            &crate::environment::root_for_home(&root).unwrap(),
        )
        .unwrap();
        let premier = resolve_spawn_order(
            &parse_spawn_args(&[
                "codex".to_string(),
                "--agent-id".to_string(),
                "89000000-0000-4000-8000-000000000015".to_string(),
                "--persistent".to_string(),
                "--command-id".to_string(),
                "garde-survie-rejeu".to_string(),
            ])
            .unwrap(),
            100,
            &cwd,
            &root,
        )
        .expect("premier envoi accepté");

        let rejeu = resolve_spawn_order(
            &parse_spawn_args(&[
                "codex".to_string(),
                "--command-id".to_string(),
                "garde-survie-rejeu".to_string(),
            ])
            .unwrap(),
            999,
            &cwd,
            &root,
        )
        .expect("le rejeu ne doit pas exiger de redéclarer le choix");
        assert_eq!(encode(&premier).unwrap(), encode(&rejeu).unwrap());

        // En revanche, rejouer en RENVERSANT le choix reste une divergence.
        let divergent = resolve_spawn_order(
            &parse_spawn_args(&[
                "codex".to_string(),
                "--no-persistent".to_string(),
                "--command-id".to_string(),
                "garde-survie-rejeu".to_string(),
            ])
            .unwrap(),
            999,
            &cwd,
            &root,
        )
        .expect_err("un choix renversé au rejeu doit être refusé");
        assert!(divergent.contains("options divergentes"), "{divergent}");

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn spawn_cli_rejoue_l_enveloppe_memorisee_octet_pour_octet() {
        let root = PathBuf::from("/tmp").join(format!("bgcanon-{}", uuid::Uuid::new_v4().simple()));
        let cwd = root.join("work");
        std::fs::create_dir_all(&cwd).unwrap();
        crate::environment::ensure_private_directory(
            &crate::environment::root_for_home(&root).unwrap(),
        )
        .unwrap();
        let args = vec![
            "codex".to_string(),
            "--agent-id".to_string(),
            "89000000-0000-4000-8000-000000000016".to_string(),
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
    fn spec_091_posture_replay_is_exact_and_divergence_refused() {
        let root = PathBuf::from("/tmp").join(format!("bg091-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&root).unwrap();
        let private_root = crate::environment::root_for_home(&root).unwrap();
        std::fs::create_dir_all(&private_root).unwrap();
        std::fs::set_permissions(&private_root, std::fs::Permissions::from_mode(0o700)).unwrap();
        let args = [
            "codex",
            "--persistent",
            "--command-id",
            "scoped-091",
            "--posture",
            "development",
        ]
        .map(str::to_string);
        let first =
            resolve_spawn_order(&parse_spawn_args(&args).unwrap(), 100, &root, &root).unwrap();
        let retry = ["codex", "--command-id", "scoped-091"].map(str::to_string);
        let replay = resolve_spawn_order(
            &parse_spawn_args(&retry).unwrap(),
            200,
            Path::new("/different"),
            &root,
        )
        .unwrap();
        assert_eq!(encode(&first).unwrap(), encode(&replay).unwrap());
        let divergent = [
            "codex",
            "--command-id",
            "scoped-091",
            "--posture",
            "discovery",
        ]
        .map(str::to_string);
        assert!(
            resolve_spawn_order(&parse_spawn_args(&divergent).unwrap(), 200, &root, &root)
                .unwrap_err()
                .contains("divergentes")
        );
        for value in ["complete", "allow", "", "workspace-write"] {
            assert!(
                parse_spawn_args(&[
                    "codex".to_string(),
                    "--posture".to_string(),
                    value.to_string()
                ])
                .is_err()
            );
        }
        std::fs::remove_dir_all(root).unwrap();
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
                        credential: None,
                        agent_id: "cli-test".to_string()
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
                        agent_id: "codex-managed".to_string(),
                        definition: Some(bridget_transport::ResolvedAgentDefinition {
                            command: "npx".to_string(),
                            args: vec!["fixture-acp".to_string()],
                            protocol: "acp".to_string(),
                            forbidden_env: vec!["OPENAI_API_KEY".to_string()],
                            pass_env: Vec::new(),
                            claude_config_dir: None,
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
            posture: None,
            agent_type: "codex".to_string(),
            agent_id: Some("codex-managed".to_string()),
            cwd: "/tmp".to_string(),
            persistent: false,
            command_id: "command-lost".to_string(),
            issued_at: 100,
            deadline_at: 110,
            project: None,
            ownership: None,
        };
        for _ in 0..2 {
            assert!(matches!(
                send_control_to_daemon_at(&socket, order.clone()).unwrap(),
                DaemonToWrapper::SpawnAccepted { ref command_id, ref agent_id, .. }
                    if command_id == "command-lost" && agent_id == "codex-managed"
            ));
        }
        let observed = server.join().unwrap();
        assert_eq!(observed[0], observed[1], "le retry doit être canonique");
        let _ = std::fs::remove_file(socket);
    }

    #[test]
    fn stop_cli_genere_ou_reutilise_un_command_id() {
        let generated =
            parse_stop_args(&["89000000-0000-4000-8000-000000000011".to_string()]).unwrap();
        assert_eq!(generated.0, "89000000-0000-4000-8000-000000000011");
        assert!(!generated.1.is_empty());
        assert_eq!(
            parse_stop_args(&[
                "89000000-0000-4000-8000-000000000011".to_string(),
                "--command-id".to_string(),
                "stop-retry".to_string(),
            ])
            .unwrap(),
            (
                "89000000-0000-4000-8000-000000000011".to_string(),
                "stop-retry".to_string()
            )
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
            agent_id: uuid::Uuid::new_v4().to_string(),
            display_name: "bridget".to_string(),
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
            disk_space: None,
            provider: None,
            persistent: None,
            execution: None,
            agent_link: None,
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
        send_rate_limits_to_daemon(&socket, "89000000-0000-4000-8000-000000000105", &facts)
            .unwrap();
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
                (
                    "89000000-0000-4000-8000-000000000105".to_string(),
                    "five_hour".to_string()
                ),
                (
                    "89000000-0000-4000-8000-000000000105".to_string(),
                    "seven_day".to_string()
                ),
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
        let root = PathBuf::from("/tmp").join(format!("bgsl-{}", uuid::Uuid::new_v4().simple()));
        crate::environment::ensure_private_directory(&root).unwrap();
        let socket = root.join("bridget.sock");
        let db_path = root.join("bridget.db");
        let config = crate::daemon::DaemonConfig {
            socket_path: socket.clone(),
            db_path: db_path.clone(),
            log_path: root.join("daemon.log"),
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
            identity_version: 2,
            agent_id: "89000000-0000-4000-8000-000000000105".to_string(),
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
        send_rate_limits_to_daemon(&socket, "89000000-0000-4000-8000-000000000105", &facts)
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
            .find(|agent| agent.agent_id == "89000000-0000-4000-8000-000000000105")
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
    use bridget_transport::protocol::{REVIEW_DELEGATE_CONTRACT_VERSION, SERVICE_CONTRACT_VERSION};
    use rusqlite::params;
    use std::os::unix::net::UnixListener;

    fn temporary_socket_path() -> std::path::PathBuf {
        crate::mcp_identity::mock_socket("cli-canon")
    }

    fn caller_identity() -> crate::mcp_identity::ResolvedIdentity {
        crate::mcp_identity::ResolvedIdentity {
            name: "89000000-0000-4000-8000-000000000200".into(),
            instance_id: "test-cli-instance".into(),
        }
    }

    fn owner_connection(path: &Path) -> DaemonConnection {
        let identity = caller_identity();
        let mut owner = DaemonConnection::connect(path).unwrap();
        let mut registration = cli_register("fixture");
        if let WrapperToDaemon::Register {
            agent_id,
            agent_type,
            instance_id,
            ..
        } = &mut registration
        {
            *agent_id = identity.name.clone();
            *agent_type = "fixture".into();
            *instance_id = Some(identity.instance_id.clone());
        }
        let DaemonToWrapper::Registered {
            credential: Some(credential),
            ..
        } = owner.exchange(&registration).unwrap()
        else {
            panic!("credential propriétaire requis");
        };
        crate::mcp_identity::save_credential(
            path.parent().unwrap(),
            &identity.name,
            &identity.instance_id,
            credential,
        )
        .unwrap();
        owner
    }

    fn start_real_daemon() -> (std::path::PathBuf, std::path::PathBuf) {
        // Le namespace du daemon est privé : ne jamais valider/réutiliser
        // l'ensemble du répertoire temporaire partagé comme son état.
        let root = PathBuf::from("/tmp").join(format!("bgcanon-{}", uuid::Uuid::new_v4().simple()));
        crate::environment::ensure_private_directory(&root).unwrap();
        let socket_path = root.join("bridget.sock");
        let db_path = root.join("bridget.db");
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
                let WrapperToDaemon::RegisterAuxiliary {
                    agent_id,
                    instance_id,
                    credential,
                } = decode(line.trim()).unwrap()
                else {
                    panic!("preuve auxiliaire attendue");
                };
                let expected =
                    crate::mcp_identity::auxiliary_registration(&agent_id, &instance_id, &path)
                        .unwrap();
                assert!(
                    matches!(expected, WrapperToDaemon::RegisterAuxiliary { credential: expected, .. } if expected == credential)
                );
                write_response(
                    &mut writer,
                    DaemonToWrapper::Registered {
                        agent_id,
                        credential: None,
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
            &crate::mcp_identity::auxiliary_registration(
                &message.from,
                &caller_identity().instance_id,
                path,
            )
            .unwrap(),
        )
        .unwrap();
        assert!(matches!(
            read_control_message(&mut reader).unwrap(),
            DaemonToWrapper::Registered { .. }
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
            "89000000-0000-4000-8000-000000000011",
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
    fn depot_guichet_parse_les_trois_mutations_sans_chemin_ni_principal_annexe() {
        let cases = [
            (
                vec![
                    "deposer",
                    "delegate",
                    "--from",
                    "89000000-0000-4000-8000-000000000017",
                    "--goal",
                    "lot central",
                    "--target",
                    "89000000-0000-4000-8000-000000000018",
                    "--duration",
                    "courte",
                ],
                ServiceRequestOperation::Delegate,
            ),
            (
                vec![
                    "deposer",
                    "registre-add",
                    "--from",
                    "89000000-0000-4000-8000-000000000017",
                    "--line",
                    "kind=add id=constat-1",
                ],
                ServiceRequestOperation::RegistreAdd,
            ),
            (
                vec![
                    "deposer",
                    "objective-close",
                    "--from",
                    "89000000-0000-4000-8000-000000000017",
                    "--objective",
                    "objective-1",
                    "--reason",
                    "objectif atteint",
                ],
                ServiceRequestOperation::ObjectiveClose,
            ),
        ];
        for (args, expected_operation) in cases {
            let args = args.into_iter().map(str::to_string).collect::<Vec<_>>();
            assert!(matches!(
                parse_guichet_deposit(&args).unwrap(),
                WrapperToDaemon::ServiceRequest { operation, .. }
                    if operation == expected_operation
            ));
        }
        for forbidden in ["--database", "--journal", "--principal"] {
            let args = [
                "deposer",
                "registre-add",
                "--from",
                "89000000-0000-4000-8000-000000000017",
                "--line",
                "kind=add id=constat-1",
                forbidden,
                "/tmp/forged",
            ]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
            assert!(parse_guichet_deposit(&args).is_err());
        }
    }

    #[test]
    fn depot_delegate_versionne_atomiquement_sa_cible_de_revue() {
        let ordinary = [
            "deposer",
            "delegate",
            "--from",
            "89000000-0000-4000-8000-000000000017",
            "--goal",
            "relire le lot",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        assert!(matches!(
            parse_guichet_deposit(&ordinary).unwrap(),
            WrapperToDaemon::ServiceRequest {
                version: SERVICE_CONTRACT_VERSION,
                payload: ServiceRequestPayload::Delegate {
                    review_target: None,
                    ..
                },
                ..
            }
        ));

        let targeted = [
            "deposer",
            "delegate",
            "--from",
            "89000000-0000-4000-8000-000000000017",
            "--goal",
            "relire le lot",
            "--review-ref",
            "origin/session-047-verdict-tete-reecrite",
            "--expected-head",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        assert!(matches!(
            parse_guichet_deposit(&targeted).unwrap(),
            WrapperToDaemon::ServiceRequest {
                version: REVIEW_DELEGATE_CONTRACT_VERSION,
                payload: ServiceRequestPayload::Delegate {
                    review_target: Some(ReviewTarget { target_ref, expected_head }),
                    ..
                },
                ..
            } if target_ref == "origin/session-047-verdict-tete-reecrite"
                && expected_head == "a".repeat(40)
        ));

        for missing in ["--review-ref", "--expected-head"] {
            let mut incomplete = targeted.clone();
            let position = incomplete
                .iter()
                .position(|value| value == missing)
                .unwrap();
            incomplete.drain(position..=position + 1);
            assert!(
                parse_guichet_deposit(&incomplete).is_err(),
                "option isolée acceptée : {missing}"
            );
        }
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
        let identity = caller_identity();
        crate::mcp_identity::mock_private_identity(&path, &identity.name, &identity.instance_id);
        let mut message = BridgetMessage::new(&identity.name, "codex-1", "bonjour");
        message.id = options.id.clone();

        let reference = reference_client_send(&path, &message, &options);
        let cli = send_idempotent_to_daemon_at(&path, &message, &options, Some(&caller_identity()))
            .unwrap();
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
            send_idempotent_to_daemon_at(&path, &divergent, &options, Some(&caller_identity()))
                .unwrap(),
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
        let _owner = owner_connection(&socket_path);
        let issued_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let options = IdempotentSendOptions {
            id: "message-t1208-reel".to_string(),
            issued_at,
            issuer_scope: "012_scope_aaaaaaaaaaaa".to_string(),
        };
        let mut message =
            BridgetMessage::new(&caller_identity().name, "destinataire-absent", "bonjour");
        message.id = options.id.clone();
        message.reply = true;
        message.reply_timeout = Some(12);

        let reference = idempotency_issue(reference_client_send(&socket_path, &message, &options));
        let cli = idempotency_issue(
            send_idempotent_to_daemon_at(
                &socket_path,
                &message,
                &options,
                Some(&caller_identity()),
            )
            .unwrap(),
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
                    send_idempotent_to_daemon_at(
                        &socket_path,
                        &divergent,
                        &options,
                        Some(&caller_identity())
                    )
                    .unwrap(),
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

    /// ORACLE — l'annuaire distingue trois réponses à « survit au redémarrage ? » :
    /// oui, non, et « nul ne l'atteste ».
    ///
    /// Le troisième cas n'est pas un non. Le 28/08, lire un `0` là où la
    /// question ne se posait pas a failli faire relancer cinq agents sains,
    /// donc détruire leur contexte. La ligne rendue doit porter la réponse,
    /// pas seulement l'en-tête.
    #[test]
    fn who_distingue_persistant_ephemere_et_non_atteste() {
        let agent = |name: &str, persistent: Option<bool>| AgentInfo {
            agent_id: uuid::Uuid::new_v4().to_string(),
            display_name: name.to_string(),
            agent_type: "fixture".to_string(),
            connection_id: format!("conn-{name}"),
            host: "local".to_string(),
            transport: "claude_stream_json".to_string(),
            channel: Some("ssh-unix".to_string()),
            mode: Some(PresenceMode::Cli),
            location: None,
            os: "Linux".to_string(),
            state: "connected".to_string(),
            last_seen_secs: 0,
            reconnect_count: 0,
            domain: Some("bridget".to_string()),
            model: None,
            effort: None,
            rate_limits: vec![],
            model_mismatch: None,
            disk_space: None,
            provider: None,
            persistent,
            agent_link: None,
            execution: None,
        };
        let rendered = render_who(
            &[
                agent("survivant", Some(true)),
                agent("ephemere", Some(false)),
                agent("hors-flotte", None),
            ],
            None,
        );

        assert!(rendered.contains("PERSIST"), "colonne absente de l'en-tête");
        let ligne = |nom: &str| {
            rendered
                .lines()
                .find(|line| line.contains(nom))
                .unwrap_or_else(|| panic!("ligne {nom} absente"))
                .to_string()
        };
        assert!(
            ligne("survivant").contains("  oui  "),
            "un agent persistant doit se lire « oui » : {}",
            ligne("survivant")
        );
        assert!(
            ligne("ephemere").contains("  non  "),
            "un agent qui sera drainé doit se lire « non » : {}",
            ligne("ephemere")
        );
        assert!(
            !ligne("hors-flotte").contains("non"),
            "une persistance non attestée ne doit jamais se lire « non » : {}",
            ligne("hors-flotte")
        );
        assert!(
            ligne("hors-flotte").contains('—'),
            "une persistance non attestée doit se lire « — » : {}",
            ligne("hors-flotte")
        );
    }

    /// ORACLE — la sortie machine publie toujours la clé, `null` compris.
    /// Une ronde doit pouvoir distinguer « indéterminable » d'un daemon trop
    /// ancien pour connaître le champ ; un champ escamoté rendrait les deux
    /// cas identiques et ramènerait le détour par SQL.
    #[test]
    fn json_publie_la_persistance_meme_indeterminee() {
        let agent = |persistent: Option<bool>| AgentInfo {
            agent_id: uuid::Uuid::new_v4().to_string(),
            display_name: "jc1-flux".to_string(),
            agent_type: "claude".to_string(),
            connection_id: "conn-182".to_string(),
            host: "poste-beta".to_string(),
            transport: "claude_stream_json".to_string(),
            channel: Some("ssh-unix".to_string()),
            mode: Some(PresenceMode::Cli),
            location: None,
            os: "Linux".to_string(),
            state: "connected".to_string(),
            last_seen_secs: 0,
            reconnect_count: 1,
            domain: Some("bridget".to_string()),
            model: None,
            effort: None,
            rate_limits: vec![],
            model_mismatch: None,
            disk_space: None,
            provider: None,
            agent_link: None,
            persistent,
            execution: None,
        };

        let atteste = serde_json::to_value(agent(Some(true))).unwrap();
        assert_eq!(
            atteste.get("persistent"),
            Some(&serde_json::Value::Bool(true))
        );
        let inconnu = serde_json::to_value(agent(None)).unwrap();
        assert_eq!(
            inconnu.get("persistent"),
            Some(&serde_json::Value::Null),
            "la clé doit rester présente et valoir null quand nul ne l'atteste"
        );

        // DTO d'identité courant sans champ optionnel persistent : on ne
        // fabrique pas ce fait. Ce n'est pas une compatibilité nom-seul v0.
        let ancien: AgentInfo = serde_json::from_str(
            r#"{"agent_id":"89000000-0000-4000-8000-000000000011","display_name":"vieux","agent_type":"codex","connection_id":"c","host":"h",
                "transport":"tmux","os":"Linux","state":"connected",
                "last_seen_secs":0,"reconnect_count":0}"#,
        )
        .unwrap();
        assert_eq!(ancien.persistent, None);
        let mut sans_identite = serde_json::to_value(&ancien).unwrap();
        sans_identite.as_object_mut().unwrap().remove("agent_id");
        assert!(serde_json::from_value::<AgentInfo>(sans_identite).is_err());
    }

    #[test]
    fn who_affiche_mode_et_localisation_sans_dependre_d_un_tty() {
        let agent = |name: &str, mode: Option<PresenceMode>, location: Option<&str>| AgentInfo {
            agent_id: uuid::Uuid::new_v4().to_string(),
            display_name: name.to_string(),
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
            agent_link: None,
            disk_space: None,
            provider: None,
            persistent: None,
            execution: None,
        };
        let mut acp = agent("acp-gere", Some(PresenceMode::Acp), None);
        acp.disk_space = Some(bridget_transport::protocol::DiskSpaceFact {
            volume: "/".to_string(),
            free_bytes: 47_300_000_000,
            observed_at_unix: 1_788_000_000,
        });
        let rendered = render_who(
            &[
                acp,
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
        assert!(rendered.contains("DISQUE"));
        assert!(rendered.contains("44.1 Gio libres"));
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
            agent_id: uuid::Uuid::new_v4().to_string(),
            display_name: "claude-1".to_string(),
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
            agent_link: None,
            model_mismatch: None,
            disk_space: None,
            provider: None,
            persistent: None,
            execution: None,
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
            agent_id: uuid::Uuid::new_v4().to_string(),
            display_name: "claude-1".to_string(),
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
            agent_link: None,
            rate_limits: vec![],
            model_mismatch: None,
            disk_space: None,
            provider: None,
            persistent: None,
            execution: None,
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
        // Observe les identifiants et l'ATTRIBUTION — pas un libellé humain
        // (le gate fondateur a déjà payé ce piège cette nuit).
        let ici = crate::build_info::local_host();
        assert!(
            stale_daemon_warning_for_status(Some(crate::build_info::BUILD_ID), Some(&ici))
                .is_none()
        );
        let warning =
            stale_daemon_warning_for_status(Some("daemon-ancien"), Some("poste-alpha")).unwrap();
        assert!(warning.contains("daemon-ancien"));
        assert!(warning.contains(crate::build_info::BUILD_ID));
        // La VALEUR portée est la machine du daemon, pas la mienne : un oracle
        // qui vérifierait seulement la présence d'un hôte passerait aussi avec
        // le mauvais.
        assert!(
            warning.contains("daemon périmé sur poste-alpha"),
            "{warning}"
        );
        // Machine non attestée : le message le dit, il ne suppose pas la mienne.
        let sans_hote = stale_daemon_warning_for_status(Some("daemon-ancien"), None).unwrap();
        assert!(
            sans_hote.contains(crate::build_info::MACHINE_NON_ATTESTEE),
            "{sans_hote}"
        );
        assert!(
            !sans_hote.contains(&format!("daemon périmé sur {ici}")),
            "{sans_hote}"
        );
    }

    /// Daemon injoignable : aucune attestation, donc aucun chiffre.
    ///
    /// CE QUE CE TEST NE PROUVE PAS, et je le dis parce que je l'ai cru :
    /// il ne franchit PAS la garde de comptage. `get_status` sort dès que la
    /// socket est absente, bien avant elle. Le mutant qui retire la garde
    /// laisse ce test VERT. La garde elle-même est éprouvée par
    /// `daemon::attribution_tests::compter_n_est_permis_que_sur_la_base_attestee_par_le_daemon`.
    #[test]
    fn status_ne_compte_pas_les_messages_d_une_base_qui_n_est_pas_celle_du_daemon() {
        let config = DaemonConfig {
            socket_path: std::env::temp_dir()
                .join(format!("b89-status-{}.sock", uuid::Uuid::new_v4())),
            ..DaemonConfig::default()
        };
        // Daemon injoignable → aucune attestation, donc aucun compte.
        let status = daemon::get_status(&config).expect("statut hors ligne observable");
        assert!(
            status.message_count.is_none(),
            "sans attestation du daemon, aucun compte ne doit être rendu"
        );
        assert!(status.daemon_db_path.is_none());
        assert!(status.daemon_host.is_none());
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

    /// M2 — le `Register` du CLI doit porter SA machine.
    ///
    /// Il envoyait `host: None` sur ses trois usages ; le daemon lisait ensuite
    /// `requested_from` dans `conn_hosts` et n'y trouvait rien. La couture
    /// bout-en-bout est éprouvée par `spawn_refusal_hosts_test` ; celui-ci garde
    /// la VALEUR émise, à la source.
    ///
    /// Mutant qui tue ce test : remettre `host: None` dans `cli_register` → la
    /// première assertion meurt en affichant `None`.
    #[test]
    fn le_register_du_cli_porte_la_machine_locale() {
        let attendu = crate::build_info::local_host();
        for usage in ["send", "rename", "runtime"] {
            match cli_register(usage) {
                WrapperToDaemon::Register { host, agent_id, .. } => {
                    assert_eq!(
                        host.as_deref(),
                        Some(attendu.as_str()),
                        "usage {usage} : le CLI doit attester sa machine"
                    );
                    assert!(
                        uuid::Uuid::parse_str(&agent_id).is_ok(),
                        "usage {usage} : l'identifiant doit être opaque"
                    );
                }
                autre => panic!("Register attendu, obtenu {autre:?}"),
            }
        }
    }

    #[test]
    fn commandes_projet_retirees_refusees_par_la_garde_avant_namespace() {
        for command in [
            "managed-runtime-wrapper",
            "managed-runtime-stop",
            "project-runtime",
            "project-round",
        ] {
            for arguments in [
                vec![],
                vec!["--help".to_string()],
                vec!["/bin/sh".to_string()],
            ] {
                let error = validate_communication_entry(command, &arguments).unwrap_err();
                assert!(error.contains(command));
                assert!(error.contains("aucun repli"));
            }
        }
        for argument in [
            "--project-root-policy",
            "--project-runtime-policy",
            "--project-resource-catalog",
        ] {
            assert!(
                validate_communication_entry(
                    "daemon",
                    &[argument.to_string(), "/unread/policy".to_string()]
                )
                .is_err()
            );
        }
        // Une observation fournisseur n'est pas un moteur de projet.
        for command in ["runtime", "spawn", "stop", "attach", "daemon"] {
            assert!(validate_communication_entry(command, &[]).is_ok());
        }
    }

    #[test]
    fn posture_cli_est_une_projection_fermee_sans_defaut_permissif() {
        use bridget_transport::protocol::AgentPosture;
        assert_eq!(
            parse_control_posture(&["discovery".to_string()]).unwrap(),
            AgentPosture::Discovery
        );
        assert_eq!(
            parse_control_posture(&["complete".to_string()]).unwrap(),
            AgentPosture::Complete
        );
        for args in [
            vec![],
            vec![""],
            vec!["Complete"],
            vec!["allow"],
            vec![" complete"],
            vec!["discovery", "complete"],
            vec!["complete", "--confirm"],
        ] {
            let args = args.into_iter().map(str::to_string).collect::<Vec<_>>();
            assert!(parse_control_posture(&args).is_err(), "{args:?}");
        }
    }
}
