//! CLI — point d'entrée unifié pour toutes les sous-commandes bridget.

use crate::daemon::{self, DaemonConfig};
use bridget_core::BridgetMessage;
use bridget_transport::protocol::{decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixStream;

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

    // --- Sous-commandes wrapper (lancement d'agents CLI) ---
    match cmd.as_str() {
        // Fonction générique pour lancer un agent (M-001)
        "codex" => launch_agent_wrapper("codex", "codex", &args[2..]),
        "claude" => launch_agent_wrapper("claude", "claude", &args[2..]),
        "gemini" => launch_agent_wrapper("gemini", "gemini", &args[2..]),
        "gclaude" => launch_agent_wrapper("gclaude", "claude", &args[2..]),
        "--" => {
            // Wrapper générique : bridget -- /path/to/command args
            if args.len() < 3 {
                eprintln!("bridget: commande manquante après --");
                std::process::exit(2);
            }
            let cmd = &args[2];
            let rest = &args[3..];
            launch_agent_wrapper(cmd, "custom", rest);
        }
        _ => {}
    }

    // --- Sous-commandes daemon / client ---
    match cmd.as_str() {
        "daemon" => cmd_daemon(),
        "send" => cmd_send(&args[2..]),
        "cancel" => cmd_cancel(&args[2..]),
        "requests" => cmd_requests(&args[2..]),
        "rename" => cmd_rename(&args[2..]),
        "reply" => cmd_reply(&args[2..]),
        "who" => cmd_who(),
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
                launch_agent_wrapper(cmd, "custom", &args[2..]);
            } else {
                eprintln!("sous-commande inconnue: {}", cmd);
                print_usage();
                std::process::exit(2);
            }
        }
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
           send --to <N> <MSG>    Envoie un message\n  \
           cancel <ID>             Annule une demande suivie\n  \
           requests                Liste mes demandes suivies\n  \
           rename <N>             Renomme l'agent courant\n  \
           who                    Agents connectés\n  \
           status                 Santé du daemon\n  \
           ledger                 Historique des messages\n  \
           version                Version\n  \
           help                   Cette aide\n\n\
         Options de send :\n  \
           --to <nom>             Destinataire (requis)\n  \
           --reply                Réponse attendue\n  \
           --hops <N>             Sauts restants (défaut: 4)"
    );
}

fn socket_path() -> std::path::PathBuf {
    DaemonConfig::default().socket_path
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
    if let Ok(path) = std::env::var("BRIDGET_AGENT_NAME_FILE") {
        if let Ok(name) = std::fs::read_to_string(path) {
            let name = name.trim();
            if !name.is_empty() {
                return name.to_string();
            }
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
    let stream = UnixStream::connect(socket_path()).map_err(|e| e.to_string())?;
    let mut writer = BufWriter::new(stream);

    let reg = WrapperToDaemon::Register {
        agent_type: "cli".to_string(),
        name: Some(format!("cli-send-{}", std::process::id())),
        host: None,
        transport: None,
        os: None,
        instance_id: None,
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
                "{:<id_width$}  {:<target_width$}  {:<state_width$}  ÉCHÉANCE",
                "ID", "DESTINATAIRE", "ÉTAT"
            );
            for request in requests {
                println!(
                    "{:<id_width$}  {:<target_width$}  {:<state_width$}  {}",
                    request.id, request.target, request.state, request.deadline_at
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

    let config = DaemonConfig::default();
    let status = daemon::get_status(&config);
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
    } else {
        if status.agents.is_empty() {
            println!("Aucun agent connecte.");
        } else {
            println!("Agents connectes :");
            for agent in &status.agents {
                println!(
                    "  {} ({}) — {} / {} via {} [{}]",
                    agent.name,
                    agent.agent_type,
                    agent.host,
                    agent.os,
                    agent.transport,
                    agent.state
                );
            }
        }
    }
}

fn cmd_who() {
    let config = DaemonConfig::default();
    let status = daemon::get_status(&config);
    if !status.running {
        eprintln!("daemon non démarré (socket absente)");
        std::process::exit(1);
    }
    if status.agents.is_empty() {
        println!("Aucun agent connecté.");
    } else {
        println!("Agents connectés :");
        let name_width = status
            .agents
            .iter()
            .map(|agent| agent.name.len())
            .max()
            .unwrap_or(3)
            .max(3);
        let type_width = status
            .agents
            .iter()
            .map(|agent| agent.agent_type.len())
            .max()
            .unwrap_or(4)
            .max(4);
        let host_width = status
            .agents
            .iter()
            .map(|agent| agent.host.len())
            .max()
            .unwrap_or(4)
            .max(4);
        let transport_width = status
            .agents
            .iter()
            .map(|agent| agent.transport.len())
            .max()
            .unwrap_or(9)
            .max(9);
        let os_width = status
            .agents
            .iter()
            .map(|agent| agent.os.len())
            .max()
            .unwrap_or(2)
            .max(2);
        println!(
            "  {:<name_width$}  {:<type_width$}  {:<host_width$}  {:<os_width$}  {:<transport_width$}  ÉTAT",
            "NOM", "TYPE", "HÔTE", "OS", "TRANSPORT"
        );
        for agent in &status.agents {
            println!(
                "  {:<name_width$}  {:<type_width$}  {:<host_width$}  {:<os_width$}  {:<transport_width$}  {}",
                agent.name, agent.agent_type, agent.host, agent.os, agent.transport, agent.state
            );
        }
    }
}

fn cmd_discover() {
    cmd_who();
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
    let config = DaemonConfig::default();
    match crate::store::Store::open(&config.db_path) {
        Ok(store) => match store.recent_messages(20) {
            Ok(entries) => {
                if entries.is_empty() {
                    println!("Ledger vide.");
                } else {
                    println!("Derniers {} messages :", entries.len());
                    for entry in entries.iter().rev() {
                        println!(
                            "  [{}] {} → {}: {}",
                            entry.ts,
                            entry.sender,
                            entry.target,
                            entry.body.chars().take(60).collect::<String>()
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("erreur lecture ledger: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("base inaccessible: {}", e);
            std::process::exit(1);
        }
    }
}
