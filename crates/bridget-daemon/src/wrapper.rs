//! Module wrapper — lance un agent CLI et le connecte au daemon bridget.
//!
//! Appelé par le CLI quand l'utilisateur tape : bridget codex, bridget claude, etc.

use bridget_transport::protocol::{decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon, TmuxTransport, Transport};
use log::{error, info, warn};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

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

fn get_current_pane_id() -> Result<String, String> {
    let output = Command::new("tmux")
        .args(["display-message", "-p", "#{pane_id}"])
        .output()
        .map_err(|e| format!("tmux exec: {}", e))?;
    if !output.status.success() {
        return Err(format!("tmux: {}", String::from_utf8_lossy(&output.stderr)));
    }
    let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if id.is_empty() { Err("pane vide".into()) } else { Ok(id) }
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
fn load_persistent_name(agent_type: &str, agent_args: &[String]) -> Option<String> {
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
fn save_persistent_name(agent_type: &str, agent_args: &[String], name: &str) {
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
    socket_path().parent().unwrap().join("agent-names").join(hash)
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
    // 1. Connexion au daemon
    let stream = UnixStream::connect(socket_path())?;
    // FD_CLOEXEC : fermer la socket dans les processus fils (fork/exec)
    // Sans ça, quand Codex exécute "bridget send", le processus fils hérite
    // de la socket et la ferme à sa mort → le wrapper perd sa connexion.
    set_cloexec(&stream);
    let read_stream = stream.try_clone()?;
    set_cloexec(&read_stream);
    let write_stream = stream.try_clone()?;
    set_cloexec(&write_stream);
    let writer = Arc::new(Mutex::new(BufWriter::new(write_stream)));

    // 2. Enregistrement — avec persistance du nom
    // Si l'utilisateur a passé --name, on l'utilise.
    // Sinon, si on fait un resume, on essaie de retrouver le nom précédent.
    let effective_name = if let Some(n) = explicit_name {
        Some(n.to_string())
    } else {
        load_persistent_name(agent_type, agent_args)
    };

    let reg = WrapperToDaemon::Register {
        agent_type: agent_type.to_string(),
        name: effective_name.clone(),
    };
    {
        let mut w = writer.lock().unwrap();
        writeln!(w, "{}", encode(&reg)?)?;
        w.flush()?;
    }

    // Lire la réponse d'enregistrement
    let mut reader = BufReader::new(read_stream.try_clone()?);
    let mut reg_line = String::new();
    reader.read_line(&mut reg_line)?;
    let reg_resp: DaemonToWrapper = decode(reg_line.trim())?;

    let my_name = match reg_resp {
        DaemonToWrapper::Registered { name } => name,
        DaemonToWrapper::Nack { reason, .. } => {
            eprintln!("bridget: enregistrement refusé: {}", reason);
            std::process::exit(1);
        }
        other => {
            eprintln!("bridget: réponse inattendue: {:?}", other);
            std::process::exit(1);
        }
    };

    eprintln!("[bridget] Enregistré en tant que « {} »", my_name);
    info!("enregistré: {}", my_name);

    // Sauvegarder le nom pour les futurs resume
    save_persistent_name(agent_type, agent_args, &my_name);

    // 3. Détection du pane tmux
    let pane_id = get_current_pane_id().unwrap_or_else(|e| {
        warn!("pas de pane tmux: {}", e);
        String::new()
    });

    // 4. Lancer l'agent CLI
    // Pour Codex : ajouter automatiquement --dangerously-bypass-approvals-and-sandbox
    // (= --yolo) sinon le sandbox bloque la connexion socket vers le daemon.
    // + injecter un prompt initial qui dit à l'agent de répondre via bridget.
    let mut final_args: Vec<String> = Vec::new();
    if agent_type == "codex" {
        // Vérifier si l'utilisateur n'a pas déjà passé --yolo ou le bypass
        let already_bypassed = agent_args.iter().any(|a| {
            a == "--yolo" || a == "--dangerously-bypass-approvals-and-sandbox"
        });
        if !already_bypassed {
            final_args.push("--dangerously-bypass-approvals-and-sandbox".to_string());
        }

        // Ne PAS utiliser --cd : ça change le working directory de l'agent.
        // Les instructions bridget viennent du prompt initial ci-dessous.
        // Le prompt initial est persistant dans le transcript et survive
        // le compaction de contexte (contrairement à un message système).
    }

    // Si l'utilisateur n'a pas passé de prompt initial (un argument libre
    // qui n'est pas un flag --xxx), injecter le prompt bridget.
    let has_prompt = agent_args.iter().any(|a| !a.starts_with("--"));
    if !has_prompt && agent_type == "codex" {
        let bridget_prompt = format!(
            "Tu es l'agent \"{}\" dans une session Bridget. Tu peux recevoir des messages d'autres agents IA. Quand tu vois une ligne qui commence par 💬 dans ton terminal, c'est un message d'un autre agent IA, pas de l'humain. Le format est : 💬 <expéditeur> → <toi> (reply=yes/no, ...)\n<message>\n\nRègles ABSOLUES :\n1. Réponds TOUJOURS avec: bridget send --to <expéditeur> \"ta réponse\"\n2. N'accuse JAMAIS réception (pas de \"bien reçu\", \"OK\", \"compris\")\n3. Ne fais PAS bridget who, bridget ledger ou bridget --help sans y être explicitement invité\n4. reply=no = notification, ne réponds que si utile. reply=yes = réponds avec du contenu.\n5. Ne réponds JAMAIS uniquement dans ton terminal — l'expéditeur ne te verrait pas.\n\nTu es maintenant en attente. Dis \"Bridget ready\" puis attends les messages.",
            my_name
        );
        final_args.push(bridget_prompt);
    }

    final_args.extend(agent_args.iter().cloned());

    eprintln!("[bridget] Lancement: {} {}", agent_binary, final_args.join(" "));

    

    let mut child = Command::new(agent_binary)
        .args(&final_args)
        .env("BRIDGET_AGENT_NAME", &my_name)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("impossible de lancer '{}': {}", agent_binary, e))?;

    let agent_pid = child.id();

    // 5. Thread d'écoute
    let listener_stream = read_stream.try_clone()?;
    let writer_clone = writer.clone();
    let pane_for_thread = pane_id.clone();
    let name_for_thread = my_name.clone();

    let listener_handle = thread::spawn(move || {
        let mut listener = BufReader::new(listener_stream);
        let mut transport = if !pane_for_thread.is_empty() {
            Some(TmuxTransport::new(pane_for_thread.clone(), agent_pid))
        } else {
            None
        };

        loop {
            let mut line = String::new();
            match listener.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {}
                Err(_) => break,
            }
            let line = line.trim();
            if line.is_empty() { continue; }

            let msg: DaemonToWrapper = match decode(line) {
                Ok(m) => m,
                Err(e) => { warn!("message illisible: {}", e); continue; }
            };

            match msg {
                DaemonToWrapper::Deliver(bm) => {
                    info!("reçu de « {} »: {}", bm.from,
                        bm.body.chars().take(60).collect::<String>());
                    // Stocker le dernier expéditeur pour la commande reply
                    let name_for_reply = name_for_thread.clone();
                    if !name_for_reply.is_empty() {
                        let reply_file = socket_path()
                            .parent()
                            .unwrap()
                            .join(format!("last-sender-{}", name_for_reply));
                        let _ = std::fs::write(&reply_file, &bm.from);
                    }
                    if let Some(ref mut t) = transport {
                        if let Err(e) = t.deliver(&bm) {
                            error!("injection tmux: {}", e);
                        }
                    } else {
                        eprintln!("\n[bridget] ← « {} »: {}\n", bm.from, bm.body);
                    }
                }
                DaemonToWrapper::Disconnect => { info!("daemon déconnecté"); break; }
                _ => {}
            }
        }
    });

    // 6. Attendre la fin de l'agent
    let status = child.wait()?;

    // 7. Désenregistrement
    {
        if let Ok(json) = encode(&WrapperToDaemon::Unregister) {
            if let Ok(mut w) = writer_clone.lock() {
                let _ = writeln!(w, "{}", json);
                let _ = w.flush();
            }
        }
    }

    drop(writer);
    drop(stream);
    let _ = listener_handle.join();

    if let Some(code) = status.code() {
        std::process::exit(code);
    }
    std::process::exit(0);
}

