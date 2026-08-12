//! CLI — point d'entrée unifié pour toutes les sous-commandes bridget.

use crate::daemon::{self, DaemonConfig};
use bridget_core::BridgetMessage;
use bridget_transport::protocol::{decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixStream;

pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(2);
    }

    let cmd = &args[1];

    // --- Sous-commandes wrapper (lancement d'agents CLI) ---
    match cmd.as_str() {
        "codex" => {
            // Extraire --name et les args restants
            let (name, rest) = extract_wrapper_args(&args[2..]);
            if let Err(e) = crate::wrapper::launch("codex", "codex", &rest, name.as_deref()) {
                eprintln!("bridget: {}", e);
                std::process::exit(1);
            }
            return;
        }
        "claude" => {
            let (name, rest) = extract_wrapper_args(&args[2..]);
            if let Err(e) = crate::wrapper::launch("claude", "claude", &rest, name.as_deref()) {
                eprintln!("bridget: {}", e);
                std::process::exit(1);
            }
            return;
        }
        "gemini" => {
            let (name, rest) = extract_wrapper_args(&args[2..]);
            if let Err(e) = crate::wrapper::launch("gemini", "gemini", &rest, name.as_deref()) {
                eprintln!("bridget: {}", e);
                std::process::exit(1);
            }
            return;
        }
        "--" => {
            // Wrapper générique : bridget -- /path/to/command args
            if args.len() < 3 {
                eprintln!("bridget: commande manquante après --");
                std::process::exit(2);
            }
            let cmd = &args[2];
            let rest = &args[3..];
            if let Err(e) = crate::wrapper::launch(cmd, "custom", rest, None) {
                eprintln!("bridget: {}", e);
                std::process::exit(1);
            }
            return;
        }
        _ => {}
    }

    // --- Sous-commandes daemon / client ---
    match cmd.as_str() {
        "daemon" => cmd_daemon(),
        "send" => cmd_send(&args[2..]),
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
                let (name, rest) = extract_wrapper_args(&args[1..]);
                if let Err(e) = crate::wrapper::launch(cmd, "custom", &rest, name.as_deref()) {
                    eprintln!("bridget: {}", e);
                    std::process::exit(1);
                }
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
            if full.exists() && std::fs::metadata(&full).map(|m| m.is_file()).unwrap_or(false) {
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
           -- <CMD> [ARGS...]     Agent personnalisé\n\n\
         Daemon & client :\n  \
           daemon                 Lance le daemon\n  \
           send --to <N> <MSG>    Envoie un message\n  \
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

    let body = body_parts.join(" ");
    if body.is_empty() {
        eprintln!("erreur: message vide");
        std::process::exit(2);
    }

    let sender = from.unwrap_or_else(|| {
        std::env::var("BRIDGET_AGENT_NAME").unwrap_or_else(|_| "human".to_string())
    });
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
                let reply_str = if effective_reply { " [réponse attendue]" } else { "" };
                println!("OK: envoyé à « {} » (id={}, hops={}){}", to, id, hops, reply_str);
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
    let stream = UnixStream::connect(socket_path()).map_err(|e| e.to_string())?;
    let mut writer = BufWriter::new(stream);

    let reg = WrapperToDaemon::Register {
        agent_type: "cli".to_string(),
        name: Some(format!("cli-send-{}", std::process::id())),
    };
    let reg_json = encode(&reg).map_err(|e| e.to_string())?;
    writeln!(writer, "{}", reg_json).map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;

    let read_stream = writer.get_ref().try_clone().map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(read_stream);
    let mut reg_line = String::new();
    reader.read_line(&mut reg_line).map_err(|e| e.to_string())?;
    let _reg_resp: DaemonToWrapper = decode(&reg_line).map_err(|e| e.to_string())?;

    let send_msg = WrapperToDaemon::Send(msg.clone());
    let send_json = encode(&send_msg).map_err(|e| e.to_string())?;
    writeln!(writer, "{}", send_json).map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;

    let mut resp_line = String::new();
    reader.read_line(&mut resp_line).map_err(|e| e.to_string())?;
    let resp: DaemonToWrapper = decode(&resp_line).map_err(|e| e.to_string())?;

    Ok(resp)
}

fn cmd_reply(args: &[String]) {
    let agent_name = std::env::var("BRIDGET_AGENT_NAME")
        .unwrap_or_else(|_| "human".to_string());

    let reply_file = socket_path()
        .parent()
        .unwrap()
        .join(format!("last-sender-{}", agent_name));

    let to = match std::fs::read_to_string(&reply_file) {
        Ok(content) => content.trim().to_string(),
        Err(_) => {
            eprintln!("reply: aucun expediteur precedent trouve.");
            eprintln!("  (utilise 'bridget send --to <nom> \"message\"')");
            std::process::exit(1);
        }
    };

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
            "--reply" => { reply_flag = true; }
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
            _ => { body_parts.push(args[i].clone()); }
        }
        i += 1;
    }

    let body = body_parts.join(" ");
    if body.is_empty() {
        eprintln!("usage: bridget reply [--reply] [--hops N] <message>");
        std::process::exit(2);
    }

    let sender = agent_name.clone();
    let effective_reply = if sender == "human" { false } else { reply_flag };

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
        let agents_json: Vec<String> = status.agents.iter().map(|a| {
            if let Some(paren_pos) = a.rfind(" (") {
                let name = &a[..paren_pos];
                let agent_type = &a[paren_pos + 2..a.len() - 1];
                format!("{{\"name\":\"{}\",\"type\":\"{}\"}}", name, agent_type)
            } else {
                format!("{{\"name\":\"{}\",\"type\":\"unknown\"}}", a)
            }
        }).collect();
        println!("[{}]", agents_json.join(","));
    } else {
        if status.agents.is_empty() {
            println!("Aucun agent connecte.");
        } else {
            println!("Agents connectes :");
            for name in &status.agents {
                println!("  {}", name);
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
        for name in &status.agents {
            println!("  {}", name);
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
        if status.running { "en ligne" } else { "hors ligne" }
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
