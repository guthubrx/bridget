//! Daemon bridget — écoute sur socket locale Unix, route les messages
//! entre les wrappers connectés, persiste l'état en SQLite.

use bridget_core::{
    CircuitBreaker, Deduplicator, EnvelopeGuard, Router, RouterAction,
};
use bridget_transport::protocol::{decode, encode};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use log::{error, info, warn};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::store::Store;
use std::sync::atomic::{AtomicBool, Ordering};

static SHUTDOWN_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Configuration du daemon.
#[derive(Clone)]
pub struct DaemonConfig {
    pub socket_path: PathBuf,
    pub db_path: PathBuf,
    pub log_path: PathBuf,
    pub circuit_breaker_window: u64,
    pub circuit_breaker_limit: usize,
    pub dedup_window: u64,
    pub quarantine_window: u64,
    pub retention_days: u32,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        let cache_dir = dirs_cache();
        DaemonConfig {
            socket_path: cache_dir.join("bridget.sock"),
            db_path: cache_dir.join("bridget.db"),
            log_path: cache_dir.join("daemon.log"),
            circuit_breaker_window: 180,
            circuit_breaker_limit: 8,
            dedup_window: 180,
            quarantine_window: 3600,
            retention_days: 7,
        }
    }
}

fn dirs_cache() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".cache").join("bridget")
    } else {
        PathBuf::from("/tmp").join("bridget")
    }
}

/// État partagé du daemon.
struct DaemonState {
    router: Router,
    circuit_breaker: CircuitBreaker,
    deduplicator: Deduplicator,
    envelope_guard: EnvelopeGuard,
    store: Store,
    connections: HashMap<String, Arc<Mutex<BufWriter<UnixStream>>>>,
    conn_names: HashMap<String, String>,
    conn_counter: u64,
    /// Messages --reply en attente de réponse : (msg_id, from, to, expire_at, target_conn)
    pending_replies: Vec<PendingReply>,
}

struct PendingReply {
    msg_id: String,
    from: String,
    from_conn: String,
    to: String,
    target_conn: String,
    /// Timeout total en secondes (configurable par l'émetteur)
    timeout_secs: u64,
    created_at: std::time::Instant,
    /// Palier d'escalade atteint : 0 = rien, 1 = rappel discret,
    /// 2 = rappel ferme, 3 = notification échec à l'émetteur
    escalation_level: u8,
}

enum ReminderAction {
    Gentle { to: String, from: String, msg_id: String, target_conn: String },
    Firm { to: String, from: String, msg_id: String, target_conn: String },
    Timeout { to: String, from: String, msg_id: String, from_conn: String, timeout_secs: u64 },
}

fn deliver_to_agent(
    writer: &Arc<Mutex<BufWriter<UnixStream>>>,
    target_name: &str,
    body: &str,
) {
    let msg = bridget_core::BridgetMessage::new("bridget", target_name, body);
    let dtw = DaemonToWrapper::Deliver(msg);
    let json = encode(&dtw).unwrap_or_default();
    if let Ok(mut w) = writer.lock() {
        let _ = writeln!(w, "{}", json);
        let _ = w.flush();
    }
}

impl DaemonState {
    fn new(config: &DaemonConfig) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(config.socket_path.parent().unwrap())?;
        let store = Store::open(&config.db_path)?;
        Ok(DaemonState {
            router: Router::new(),
            circuit_breaker: CircuitBreaker::new(
                config.circuit_breaker_window,
                config.circuit_breaker_limit,
            ),
            deduplicator: Deduplicator::new(config.dedup_window),
            envelope_guard: EnvelopeGuard::new(Duration::from_secs(config.quarantine_window)),
            store,
            connections: HashMap::new(),
            conn_names: HashMap::new(),
            conn_counter: 0,
            pending_replies: Vec::new(),
        })
    }

    fn next_conn_id(&mut self) -> String {
        self.conn_counter += 1;
        format!("conn-{}", self.conn_counter)
    }
}

/// Lance le daemon.
pub fn run(config: DaemonConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Verrouillage par PID file — empêche deux daemons de démarrer en même temps
    let pid_file = config.socket_path.with_extension("pid");
    if pid_file.exists() {
        let pid_str = std::fs::read_to_string(&pid_file).unwrap_or_default();
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            // Vérifier si ce process existe encore
            unsafe {
                if libc::kill(pid, 0) == 0 {
                    eprintln!("bridget: un daemon tourne déjà (PID {})", pid);
                    std::process::exit(0);
                }
            }
        }
        // Le PID est mort — on peut prendre la relance
        eprintln!("bridget: nettoyage PID file stale (daemon précédent mort)");
        let _ = std::fs::remove_file(&pid_file);
    }
    std::fs::write(&pid_file, std::process::id().to_string())?;
    eprintln!("[BRIDGET] PID file écrit: {} (PID {})", pid_file.display(), std::process::id());

    if config.socket_path.exists() {
        std::fs::remove_file(&config.socket_path)?;
    }
    std::fs::create_dir_all(config.socket_path.parent().unwrap())?;

    let listener = UnixListener::bind(&config.socket_path)?;
    info!("bridget daemon écoute sur {}", config.socket_path.display());

    let state = Arc::new(Mutex::new(DaemonState::new(&config)?));

    // Purge au démarrage
    {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        if let Ok(n) = st.store.purge_older_than_days(config.retention_days) {
            if n > 0 {
                info!("purge: {} messages supprimés (> {} jours)", n, config.retention_days);
            }
        }
    }

    // Thread de surveillance des --reply sans réponse (escalade progressive)
    // Palier 1 (T/3) : rappel discret au destinataire
    // Palier 2 (2T/3) : rappel ferme au destinataire
    // Palier 3 (T) : notification d'échec à l'émetteur
    // Après T + 30s : abandon (retiré de la liste)
    let st_reminder = state.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(3));
        let now = std::time::Instant::now();

        // Collecter les actions à faire
        let actions: Vec<ReminderAction> = {
            let mut st = st_reminder.lock().unwrap_or_else(|e| e.into_inner());
            let mut actions = Vec::new();

            for p in st.pending_replies.iter_mut() {
                let elapsed = now.duration_since(p.created_at).as_secs();
                let t = p.timeout_secs;

                if p.escalation_level == 0 && elapsed >= t / 3 {
                    p.escalation_level = 1;
                    actions.push(ReminderAction::Gentle {
                        to: p.to.clone(),
                        from: p.from.clone(),
                        msg_id: p.msg_id.clone(),
                        target_conn: p.target_conn.clone(),
                    });
                } else if p.escalation_level == 1 && elapsed >= (t * 2) / 3 {
                    p.escalation_level = 2;
                    actions.push(ReminderAction::Firm {
                        to: p.to.clone(),
                        from: p.from.clone(),
                        msg_id: p.msg_id.clone(),
                        target_conn: p.target_conn.clone(),
                    });
                } else if p.escalation_level == 2 && elapsed >= t {
                    p.escalation_level = 3;
                    actions.push(ReminderAction::Timeout {
                        to: p.to.clone(),
                        from: p.from.clone(),
                        msg_id: p.msg_id.clone(),
                        from_conn: p.from_conn.clone(),
                        timeout_secs: t,
                    });
                }
            }

            // Retirer les entries au palier 3 depuis plus de 30s (abandon)
            st.pending_replies.retain(|p| {
                if p.escalation_level >= 3 {
                    let elapsed = now.duration_since(p.created_at).as_secs();
                    elapsed < p.timeout_secs + 30
                } else {
                    true
                }
            });

            actions
        };

        // Exécuter les actions hors lock
        for action in actions {
            match action {
                ReminderAction::Gentle { to, from, msg_id, target_conn } => {
                    let st = st_reminder.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(target_writer) = st.connections.get(&target_conn) {
                        let body = format!(
                            "Rappel : {} attend ta reponse au message #{}.\nReponds avec: bridget reply \"ta reponse\"",
                            from, &msg_id[..msg_id.len().min(8)]
                        );
                        deliver_to_agent(target_writer, &to, &body);
                        info!("palier 1 (rappel doux) → {} pour {}", to, from);
                    }
                }
                ReminderAction::Firm { to, from, msg_id, target_conn } => {
                    let st = st_reminder.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(target_writer) = st.connections.get(&target_conn) {
                        let body = format!(
                            "URGENT : {} attend toujours ta reponse au message #{}.\nTu DOIS repondre maintenant avec: bridget reply \"ta reponse\"\nSi tu ne peux pas repondre, notifie-le : bridget reply \"impossible de repondre : <raison>\"",
                            from, &msg_id[..msg_id.len().min(8)]
                        );
                        deliver_to_agent(target_writer, &to, &body);
                        info!("palier 2 (rappel ferme) → {} pour {}", to, from);
                    }
                }
                ReminderAction::Timeout { to, from, msg_id, from_conn, timeout_secs } => {
                    let st = st_reminder.lock().unwrap_or_else(|e| e.into_inner());
                    // Notifier l'émetteur que le destinataire n'a pas répondu
                    if let Some(sender_writer) = st.connections.get(&from_conn) {
                        let body = format!(
                            "{} n'a pas repondu en {}s au message #{}.\nTu peux reessayer, changer de destinataire ou abandonner.",
                            to, timeout_secs, &msg_id[..msg_id.len().min(8)]
                        );
                        deliver_to_agent(sender_writer, &from, &body);
                        info!("palier 3 (timeout notifié à {} : {} n'a pas répondu)", from, to);
                    }
                }
            }
        }
    });

    // Thread de purge périodique
    let st_purge = state.clone();
    let retention = config.retention_days;
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(3600));
        let st = st_purge.lock().unwrap_or_else(|e| e.into_inner());
        if let Ok(n) = st.store.purge_older_than_days(retention) {
            if n > 0 {
                info!("purge périodique: {} messages supprimés", n);
            }
        }
    });

    // Setup signal handler — flag atomique global (pas de Mutex dans le handler)
    // On utilise un flag atomique simple. Le shutdown propre (notification
    // des wrappers) est fait dans la boucle principale quand elle détecte le flag.
    SHUTDOWN_REQUESTED.store(false, Ordering::SeqCst);

    // Seul SIGTERM déclenche le shutdown propre (c'est ce que launchd/systemd envoie).
    // SIGINT (Ctrl+C) est ignoré en mode daemon — l'utilisateur doit utiliser
    // launchctl stop ou kill -TERM pour arrêter le daemon.
    unsafe {
        let _ = signal_hook::low_level::register(signal_hook::consts::SIGTERM, || {
            eprintln!("[BRIDGET] *** SIGTERM REÇU *** à {}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs());
            SHUTDOWN_REQUESTED.store(true, Ordering::SeqCst);
        });
        let _ = signal_hook::low_level::register(signal_hook::consts::SIGINT, || {
            eprintln!("[BRIDGET] *** SIGINT REÇU (ignoré) *** à {}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs());
        });
        // Ignorer SIGHUP (envoyé quand le terminal se ferme)
        let _ = signal_hook::low_level::register(signal_hook::consts::SIGHUP, || {
            eprintln!("[BRIDGET] *** SIGHUP REÇU (ignoré) ***");
        });
    }

    // Boucle d'acceptation avec timeout pour vérifier shutdown
    listener.set_nonblocking(true)?;
    loop {
        // Vérifier si shutdown demandé
        if SHUTDOWN_REQUESTED.load(Ordering::SeqCst) {
            info!("shutdown demandé — notification des wrappers...");
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            for (_, writer) in &st.connections {
                let msg = DaemonToWrapper::Disconnect;
                if let Ok(json) = encode(&msg) {
                    if let Ok(mut w) = writer.lock() {
                        let _ = writeln!(w, "{}", json);
                        let _ = w.flush();
                    }
                }
            }
            drop(st);
            let _ = std::fs::remove_file(&config.socket_path);
            let _ = std::fs::remove_file(&config.socket_path.with_extension("pid"));
            info!("daemon arrêté proprement");
            return Ok(());
        }

        match listener.accept() {
            Ok((stream, _)) => {
                if let Err(e) = stream.set_nonblocking(false) {
                    error!("set_nonblocking failed: {} — connexion ignorée", e);
                    continue;
                }
                let st = state.clone();
                thread::spawn(move || {
                    if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        handle_connection(stream, st)
                    })) {
                        error!("PANIC dans thread connexion: {:?}", e);
                    }
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                error!("accept: {}", e);
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

/// Gère une connexion wrapper.
fn handle_connection(
    stream: UnixStream,
    state: Arc<Mutex<DaemonState>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn_id = state.lock().unwrap_or_else(|e| e.into_inner()).next_conn_id();
    log::debug!("handle_connection: nouvelle connexion {}", conn_id);
    let reader_stream = stream.try_clone()?;
    let reader = BufReader::new(reader_stream);

    // Enregistrer le writer dans la map pour les push
    let push_stream = stream.try_clone()?;
    {
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        st.connections
            .insert(conn_id.clone(), Arc::new(Mutex::new(BufWriter::new(push_stream))));
    }
    info!("connexion {} établie", conn_id);

    let mut my_writer = BufWriter::new(stream);

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.is_empty() {
            continue;
        }

        let msg: WrapperToDaemon = match decode(&line) {
            Ok(m) => m,
            Err(e) => {
                warn!("message illisible de {}: {}", conn_id, e);
                continue;
            }
        };

        let response = handle_wrapper_message(&conn_id, msg, &state);
        if let Some(dtw) = response {
            let json = encode(&dtw)?;
            writeln!(my_writer, "{}", json)?;
            my_writer.flush()?;
        }
    }

    // Connexion fermée : désenregistrer
    let removed = {
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        st.connections.remove(&conn_id);
        let removed = st.router.unregister_by_conn(&conn_id);
        st.conn_names.remove(&conn_id);
        removed
    };
    if let Some(agent) = removed {
        info!("agent '{}' déconnecté ({})", agent.name, conn_id);
    } else {
        info!("connexion {} fermée (non enregistrée)", conn_id);
    }
    log::debug!("handle_connection {} terminée", conn_id);

    Ok(())
}

/// Traite un message wrapper et retourne une réponse optionnelle.
fn handle_wrapper_message(
    conn_id: &str,
    msg: WrapperToDaemon,
    state: &Arc<Mutex<DaemonState>>,
) -> Option<DaemonToWrapper> {
    match msg {
        WrapperToDaemon::Register { agent_type, name } => {
            log::debug!("Register reçu de {}: type={}, name={:?}", conn_id, agent_type, name);
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            let parsed_type = agent_type
                .parse()
                .unwrap_or(bridget_core::AgentType::Custom(agent_type));
            match st.router.register(name.as_deref(), &parsed_type, conn_id) {
                Ok(final_name) => {
                    st.conn_names
                        .insert(conn_id.to_string(), final_name.clone());
                    info!("agent '{}' enregistré ({})", final_name, conn_id);
                    Some(DaemonToWrapper::Registered { name: final_name })
                }
                Err(e) => Some(DaemonToWrapper::Nack {
                    id: "register".to_string(),
                    reason: e.to_string(),
                }),
            }
        }

        WrapperToDaemon::Unregister => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            st.router.unregister_by_conn(conn_id);
            st.conn_names.remove(conn_id);
            None
        }

        WrapperToDaemon::Rename { current_name, name } => {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            let target_conn = match st.router.get_agent(&current_name) {
                Some(agent) => agent.connection_id.clone(),
                None => return Some(DaemonToWrapper::Nack {
                    id: "rename".to_string(),
                    reason: format!("agent introuvable: {}", current_name),
                }),
            };
            match st.router.rename(&target_conn, &name) {
                Ok((old_name, new_name)) => {
                    st.conn_names.insert(target_conn, new_name.clone());
                    info!("agent '{}' renommé en '{}'", old_name, new_name);
                    Some(DaemonToWrapper::Renamed { old_name, name: new_name })
                }
                Err(error) => Some(DaemonToWrapper::Nack {
                    id: "rename".to_string(),
                    reason: error.to_string(),
                }),
            }
        }

        WrapperToDaemon::Send(mut bridge_msg) => {
            eprintln!("[BRIDGET] Send de {}: to={}, body={}", conn_id, bridge_msg.to, bridge_msg.body.chars().take(40).collect::<String>());
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            let sender_name = st.conn_names.get(conn_id).cloned().unwrap_or_default();
            // Résolution de l'expéditeur :
            // - Si la connexion est un wrapper (agent enregistré sous son vrai nom),
            //   utiliser ce nom.
            // - Si la connexion est un CLI temporaire (cli-send-XXXXX), vérifier si
            //   le from du message correspond à un agent enregistré (ex: codex-1).
            //   Si oui, utiliser ce from (le CLI a été lancé depuis l'intérieur du wrapper).
            //   Si non, garder le from tel quel (envoi depuis terminal externe).
            // Vérifier si ce message répond à un --reply en attente
            {
                let resolved_from = if !sender_name.is_empty() && !sender_name.starts_with("cli-send-") {
                    sender_name.clone()
                } else {
                    bridge_msg.from.clone()
                };
                let before = st.pending_replies.len();
                st.pending_replies.retain(|p| {
                    if p.to == resolved_from && p.from == bridge_msg.to {
                        info!("réponse reçue: {} a répondu à {}", resolved_from, p.from);
                        false // Retirer : la réponse est arrivée
                    } else {
                        true
                    }
                });
            }

            if !sender_name.is_empty() && !sender_name.starts_with("cli-send-") {
                // Wrapper : utiliser le nom enregistré
                bridge_msg.from = sender_name.clone();
            } else if st.router.get_agent(&bridge_msg.from).is_some() {
                // CLI temporaire mais le from correspond à un agent enregistré
                // → confiance accordée (le CLI tourne dans le contexte du wrapper)
                // On garde bridge_msg.from tel quel
            } else {
                // CLI temporaire avec from inconnu
                // → utiliser le nom de connexion (cli-send-XXXXX ou human)
                if !sender_name.is_empty() {
                    bridge_msg.from = sender_name.clone();
                }
            }

            // 1. Disjoncteur
            if !st.circuit_breaker.check(&bridge_msg.from, &bridge_msg.to) {
                warn!(
                    "DISJONCTEUR: {} ↔ {} — {} échanges dans la fenêtre",
                    bridge_msg.from,
                    bridge_msg.to,
                    st.circuit_breaker.count(&bridge_msg.from, &bridge_msg.to)
                );
                return Some(DaemonToWrapper::Nack {
                    id: bridge_msg.id.clone(),
                    reason: format!(
                        "disjoncteur: limite {} échanges / {}s",
                        st.circuit_breaker.limit(),
                        st.circuit_breaker.window_secs()
                    ),
                });
            }

            // 2. Déduplication par contenu
            let content_key = bridge_msg.content_key();
            if st.deduplicator.is_duplicate(&content_key, &bridge_msg.to) {
                warn!("DEDUP: doublon vers « {} » (clé {})", bridge_msg.to, content_key);
                return Some(DaemonToWrapper::Nack {
                    id: bridge_msg.id.clone(),
                    reason: "doublon de contenu".to_string(),
                });
            }

            // 3. Quarantaine par ID
            if st
                .envelope_guard
                .is_quarantined(&bridge_msg.id, &bridge_msg.to)
            {
                warn!(
                    "QUARANTAINE: id {} déjà relayé vers « {} »",
                    bridge_msg.id, bridge_msg.to
                );
                return Some(DaemonToWrapper::Nack {
                    id: bridge_msg.id.clone(),
                    reason: "message déjà relayé (quarantaine)".to_string(),
                });
            }

            // 4. Décrémenter les hops
            if !bridge_msg.decrement_hops() {
                return Some(DaemonToWrapper::Nack {
                    id: bridge_msg.id.clone(),
                    reason: "budget de sauts épuisé".to_string(),
                });
            }

            // 5. Router
            let action = st.router.resolve(
                &bridge_msg.from,
                &bridge_msg.to,
                bridge_msg.hops,
                conn_id,
            );

            match action {
                RouterAction::Deliver { target_conn } => {
                    let conv_key = format!("{}|{}", bridge_msg.from, bridge_msg.to);

                    if let Err(e) = st.store.record_message(&bridge_msg, &conv_key) {
                        error!("store: {}", e);
                    }
                    st.circuit_breaker.record(&bridge_msg.from, &bridge_msg.to);
                    st.deduplicator.mark_sent(&content_key, &bridge_msg.to);
                    st.envelope_guard
                        .mark_relayed(&bridge_msg.id, &bridge_msg.to);

                    // Push vers le destinataire
                    let dtw = DaemonToWrapper::Deliver(bridge_msg.clone());
                    let json = encode(&dtw).unwrap_or_default();
                    eprintln!("[BRIDGET] Push vers {}: {} octets", target_conn, json.len());

                    if let Some(target_writer) = st.connections.get(&target_conn) {
                        log::debug!("push vers {}: écriture sur writer", target_conn);
                        if let Ok(mut w) = target_writer.lock() {
                            eprintln!("[BRIDGET] Writer locked for {}, écriture...", target_conn);
                            match writeln!(w, "{}", json) {
                                Ok(_) => {
                                    let _ = w.flush();
                                    info!(
                                        "livré: {} → « {} » (hops={}, reply={})",
                                        bridge_msg.id, bridge_msg.to, bridge_msg.hops, bridge_msg.reply
                                    );
                                }
                                Err(e) => error!("push {}: {}", target_conn, e),
                            }
                        }
                    } else {
                        warn!("cible {} disparue", target_conn);
                    }

                    // Si reply=yes, enregistrer dans pending_replies pour escalade
                    if bridge_msg.reply {
                        let timeout = bridge_msg.reply_timeout.unwrap_or(60);
                        st.pending_replies.push(PendingReply {
                            msg_id: bridge_msg.id.clone(),
                            from: bridge_msg.from.clone(),
                            from_conn: conn_id.to_string(),
                            to: bridge_msg.to.clone(),
                            target_conn: target_conn.clone(),
                            timeout_secs: timeout,
                            created_at: std::time::Instant::now(),
                            escalation_level: 0,
                        });
                        info!(
                            "reply attendu: {} → {} (timeout={}s, escalade à {}/{}s)",
                            bridge_msg.id, bridge_msg.to, timeout, timeout / 3, timeout * 2 / 3
                        );
                    }

                    Some(DaemonToWrapper::Ack {
                        id: bridge_msg.id.clone(),
                    })
                }
                RouterAction::Reject(err) => {
                    warn!("rejet: {}", err);
                    Some(DaemonToWrapper::Nack {
                        id: bridge_msg.id.clone(),
                        reason: err.to_string(),
                    })
                }
            }
        }

        WrapperToDaemon::Heartbeat => None,

        WrapperToDaemon::ListAgents => {
            let st = state.lock().unwrap_or_else(|e| e.into_inner());
            let agents: Vec<bridget_transport::protocol::AgentInfo> = st
                .router
                .list_agents()
                .iter()
                .map(|a| bridget_transport::protocol::AgentInfo {
                    name: a.name.clone(),
                    agent_type: a.agent_type.to_string(),
                    connection_id: a.connection_id.clone(),
                })
                .collect();
            Some(DaemonToWrapper::AgentList { agents })
        }
    }
}

/// Statut du daemon — interroge le daemon via la socket locale.
pub fn get_status(config: &DaemonConfig) -> DaemonStatus {
    use std::os::unix::net::UnixStream;
    use std::io::{BufRead, BufReader, BufWriter, Write};

    if !config.socket_path.exists() {
        return DaemonStatus::default();
    }

    let stream = match UnixStream::connect(&config.socket_path) {
        Ok(s) => s,
        Err(_) => return DaemonStatus::default(),
    };

    let read_stream = match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return DaemonStatus::default(),
    };

    let mut writer = BufWriter::new(stream);
    let mut reader = BufReader::new(read_stream);

    // Register
    let reg = WrapperToDaemon::Register {
        agent_type: "status-probe".to_string(),
        name: Some(format!("status-{}", std::process::id())),
    };
    let reg_json = match encode(&reg) { Ok(j) => j, Err(_) => return DaemonStatus::default() };
    if writeln!(writer, "{}", reg_json).is_err() { return DaemonStatus::default(); }
    if writer.flush().is_err() { return DaemonStatus::default(); }

    // Lire Registered
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() { return DaemonStatus::default(); }

    // Demander la liste des agents
    let list_req = WrapperToDaemon::ListAgents;
    let list_json = match encode(&list_req) { Ok(j) => j, Err(_) => return DaemonStatus::default() };
    if writeln!(writer, "{}", list_json).is_err() { return DaemonStatus::default(); }
    if writer.flush().is_err() { return DaemonStatus::default(); }

    // Lire AgentList
    let mut resp_line = String::new();
    if reader.read_line(&mut resp_line).is_err() { return DaemonStatus::default(); }
    let agents = match decode::<DaemonToWrapper>(resp_line.trim()) {
        Ok(DaemonToWrapper::AgentList { agents }) => {
            agents.iter().map(|a| format!("{} ({})", a.name, a.agent_type)).collect()
        }
        _ => vec![],
    };

    // Compter les messages en base
    let message_count = match Store::open(&config.db_path) {
        Ok(store) => store.recent_messages(1000).map(|v| v.len()).unwrap_or(0),
        Err(_) => 0,
    };

    DaemonStatus {
        running: true,
        agents,
        message_count,
    }
}

impl Default for DaemonStatus {
    fn default() -> Self {
        DaemonStatus { running: false, agents: vec![], message_count: 0 }
    }
}

pub struct DaemonStatus {
    pub running: bool,
    pub agents: Vec<String>,
    pub message_count: usize,
}
