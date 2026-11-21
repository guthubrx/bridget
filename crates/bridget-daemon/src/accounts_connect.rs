//! Connexion et changement de compte facilités multi-machines et multi-fournisseurs.
//!
//! Fournit :
//! - `bridget accounts status [--json]` : diagnostic parallèle sur les machines connues
//! - `bridget accounts login [--connection ID] [--machine M] [--from C --prepare-id N] [--logout-first]`
//! - `bridget accounts ui` : fenêtre web strictement locale (`127.0.0.1`, jeton aléatoire)
//!
//! Aucun jeton ni secret n'est stocké par Bridget : chaque fournisseur conserve
//! ses propres identifiants dans son dossier privé ou son trousseau système.

use crate::cli_table::table_println as println;
use crate::registry::AgentRegistry;
use axum::Router;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Json};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const PROBE_TIMEOUT_SECS: u64 = 8;
const MAX_SESSION_LOG_BYTES: usize = 256 * 1024;
const DEFAULT_SSH_CONNECT_TIMEOUT: &str = "4";

const CLEAR_AUTH_ENV: &[&str] = &[
    "OPENAI_API_KEY",
    "CODEX_API_KEY",
    "OPENAI_ACCESS_TOKEN",
    "ANTHROPIC_API_KEY",
    "ANTHROPIC_AUTH_TOKEN",
    "CLAUDE_CODE_OAUTH_TOKEN",
    "GEMINI_API_KEY",
    "GOOGLE_API_KEY",
    "AGY_ADC_AUTH",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ConnectionTarget {
    pub connection_id: String,
    pub machine_id: String,
    pub engine: String,
    pub agent_type: String,
    pub enabled: bool,
    pub display_name: Option<String>,
    pub managed: bool,
    pub command: String,
    pub profile_dir: Option<String>,
    pub registry_path: String,
    pub is_local: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ConnectionStatus {
    pub connection_id: String,
    pub machine_id: String,
    pub engine: String,
    pub agent_type: String,
    pub enabled: bool,
    pub display_name: Option<String>,
    pub managed: bool,
    pub profile_dir: Option<String>,
    pub is_local: bool,
    pub auth_state: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FleetAuthStatus {
    pub local_machine: String,
    pub checked_at: i64,
    pub machines: Vec<String>,
    pub connections: Vec<ConnectionStatus>,
}

pub(crate) fn run(args: &[String]) {
    let sub = args.first().map(String::as_str).unwrap_or("");
    let rest = if args.is_empty() { &[][..] } else { &args[1..] };
    let result = match sub {
        "status" => cmd_status(rest),
        "login" => cmd_login(rest),
        "ui" => cmd_ui(rest),
        _ => Err(format!("sous-commande accounts inconnue : {sub}")),
    };
    if let Err(error) = result {
        eprintln!("bridget accounts {sub} : {error}");
        crate::cli_table::exit(1);
    }
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn local_machine_name() -> String {
    let host = crate::build_info::local_host();
    if host.is_empty() {
        "local".to_string()
    } else {
        host
    }
}

/// Echappe une valeur en argument POSIX sûr entre quotes simples.
pub(crate) fn sh_quote(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('\'');
    for ch in value.chars() {
        if ch == '\'' {
            out.push_str("'\"'\"'");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

/// Valide un identifiant de machine utilisé comme hôte SSH (lettres, chiffres, `.`, `_`, `-`).
pub(crate) fn valid_machine_id(machine: &str) -> bool {
    !machine.is_empty()
        && machine.len() <= 64
        && !machine.starts_with('-')
        && machine
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}

/// Indique si une URL HTTPS correspond à un flux de connexion OAuth / authentification
/// (et non à un lien de documentation ou d'aide imprimé par le CLI).
fn is_oauth_url(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.contains("accounts.google.com")
        || lower.contains("oauth")
        || lower.contains("authorize")
        || lower.contains("auth.openai.com")
        || lower.contains("claude.ai/")
        || lower.contains("console.anthropic.com")
        || lower.contains("login")
        || lower.contains("device")
}

/// Extrait la première URL HTTPS d'authentification présente dans une ligne texte.
pub(crate) fn extract_auth_url(text: &str) -> Option<String> {
    let cleaned = strip_ansi(text);
    for token in cleaned.split_whitespace() {
        let candidate = token
            .trim_matches(|c: char| matches!(c, '(' | ')' | '<' | '>' | '"' | '\'' | ',' | ';'));
        if candidate.starts_with("https://") && candidate.len() > 16 && is_oauth_url(candidate) {
            return Some(candidate.to_string());
        }
    }
    None
}

/// Retire les séquences ANSI d'échappement pour un affichage lisible.
pub(crate) fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            if matches!(chars.peek(), Some('[')) {
                let _ = chars.next();
                for c in chars.by_ref() {
                    if ('@'..='~').contains(&c) {
                        break;
                    }
                }
            } else if matches!(chars.peek(), Some(']')) {
                let _ = chars.next();
                while let Some(c) = chars.next() {
                    if c == '\x07' || (c == '\x1b' && matches!(chars.peek(), Some('\\'))) {
                        if c == '\x1b' {
                            let _ = chars.next();
                        }
                        break;
                    }
                }
            }
            continue;
        }
        if ch == '\r' {
            continue;
        }
        out.push(ch);
    }
    out
}

fn read_remote_file(machine: &str, remote_path: &str) -> Option<String> {
    if !valid_machine_id(machine) {
        return None;
    }
    let output = Command::new("ssh")
        .args([
            "-o",
            "BatchMode=yes",
            "-o",
            &format!("ConnectTimeout={DEFAULT_SSH_CONNECT_TIMEOUT}"),
            machine,
            &format!("cat {} 2>/dev/null", sh_quote(remote_path)),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() || output.stdout.is_empty() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

fn extract_connections_from_registry(
    raw: &str,
    path: &Path,
    default_machine: &str,
    local_machine: &str,
    only_machine: Option<&str>,
) -> Vec<ConnectionTarget> {
    let Ok(registry) = AgentRegistry::from_json(raw, path) else {
        return Vec::new();
    };
    let Some(policy) = registry.routing_policy() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for conn in &policy.connections {
        if !matches!(conn.engine.as_str(), "claude" | "codex" | "antigravity") {
            continue;
        }
        let machine_id = if conn.machine_id == "local" {
            default_machine.to_string()
        } else {
            conn.machine_id.clone()
        };
        if let Some(filter) = only_machine
            && machine_id != filter
        {
            continue;
        }
        let Ok(def) = registry.get(&conn.agent_type) else {
            continue;
        };
        let profile_dir = match conn.engine.as_str() {
            "claude" => def.claude_config_dir.clone(),
            "codex" => def.codex_home.clone(),
            "antigravity" => def.antigravity_home.clone(),
            _ => None,
        };
        out.push(ConnectionTarget {
            connection_id: conn.connection_id.clone(),
            machine_id: machine_id.clone(),
            engine: conn.engine.clone(),
            agent_type: conn.agent_type.clone(),
            enabled: conn.enabled,
            display_name: conn.display_name.clone(),
            managed: conn.connection_id.starts_with("account-"),
            command: def.command.clone(),
            profile_dir,
            registry_path: path.to_string_lossy().into_owned(),
            is_local: machine_id == local_machine,
        });
    }
    out
}

/// Charge toutes les connexions connues sur la machine locale et sur les serveurs
/// joignables de la fédération (en fusionnant le registre maître et les registres worker).
pub(crate) fn load_fleet_connections(
    registry_override: Option<&Path>,
) -> Result<(String, Vec<ConnectionTarget>), String> {
    let local_machine = local_machine_name();
    let local_path = match registry_override {
        Some(p) => p.to_path_buf(),
        None => crate::registry::config_path()?,
    };
    let local_raw = crate::registry::read_private_registry(&local_path)
        .map_err(|e| format!("registre local illisible ({}): {e}", local_path.display()))?;

    let mut by_key: BTreeMap<(String, String), ConnectionTarget> = BTreeMap::new();
    let local_only_filter = if registry_override.is_some() {
        None
    } else {
        Some(local_machine.as_str())
    };
    for target in extract_connections_from_registry(
        &local_raw,
        &local_path,
        &local_machine,
        &local_machine,
        local_only_filter,
    ) {
        by_key.insert(
            (target.machine_id.clone(), target.connection_id.clone()),
            target,
        );
    }

    // Si un registre explicite est fourni (--registry), ne pas interroger d'autres hôtes en SSH.
    if registry_override.is_some() {
        return Ok((local_machine, by_key.into_values().collect()));
    }

    // Découvrir les machines de la fédération depuis le registre local et/ou cartae.
    let mut remote_machines: Vec<String> = Vec::new();
    if let Ok(reg) = AgentRegistry::from_json(&local_raw, &local_path) {
        for name in reg.machines().keys() {
            if name != &local_machine && name != "local" && valid_machine_id(name) {
                remote_machines.push(name.clone());
            }
        }
    }
    if local_machine != "cartae" && !remote_machines.iter().any(|m| m == "cartae") {
        remote_machines.push("cartae".to_string());
    }

    // Si on n'est pas sur cartae, lire la liste complète des machines depuis cartae.
    let mut master_connections = Vec::new();
    if local_machine != "cartae"
        && let Some(cartae_raw) =
            read_remote_file("cartae", "/home/moi/.cache/bridget-core/agents.json")
    {
        let cartae_path = PathBuf::from("/home/moi/.cache/bridget-core/agents.json");
        if let Ok(reg) = AgentRegistry::from_json(&cartae_raw, &cartae_path) {
            for name in reg.machines().keys() {
                if name != &local_machine
                    && name != "local"
                    && valid_machine_id(name)
                    && !remote_machines.iter().any(|m| m == name)
                {
                    remote_machines.push(name.clone());
                }
            }
        }
        master_connections = extract_connections_from_registry(
            &cartae_raw,
            &cartae_path,
            "cartae",
            &local_machine,
            None,
        );
    }
    for target in master_connections {
        by_key
            .entry((target.machine_id.clone(), target.connection_id.clone()))
            .or_insert(target);
    }

    // Lire en parallèle le registre worker privé de chaque machine distante pour
    // récupérer les chemins de binaires locaux réels et les comptes isolés préparés.
    let handles: Vec<_> = remote_machines
        .into_iter()
        .map(|machine| {
            let local_m = local_machine.clone();
            std::thread::spawn(move || {
                let candidates = if machine == "cartae" {
                    vec![
                        "/home/moi/.cache/bridget-core/agents.json",
                        "/home/moi/.config/bridget/agents.json",
                    ]
                } else {
                    vec![
                        "/home/moi/.config/bridget/agents.json",
                        "/home/moi/.cache/bridget-core/agents.json",
                    ]
                };
                for path_str in candidates {
                    if let Some(raw) = read_remote_file(&machine, path_str) {
                        let p = PathBuf::from(path_str);
                        let found = extract_connections_from_registry(
                            &raw,
                            &p,
                            &machine,
                            &local_m,
                            Some(&machine),
                        );
                        if !found.is_empty() {
                            return found;
                        }
                    }
                }
                Vec::new()
            })
        })
        .collect();

    for handle in handles {
        if let Ok(items) = handle.join() {
            for target in items {
                by_key.insert(
                    (target.machine_id.clone(), target.connection_id.clone()),
                    target,
                );
            }
        }
    }

    Ok((local_machine, by_key.into_values().collect()))
}

fn unset_auth_env_args() -> String {
    CLEAR_AUTH_ENV
        .iter()
        .map(|k| format!("-u {k}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn env_prefix_for(engine: &str, profile: &str) -> String {
    let unset_env = unset_auth_env_args();
    if profile.is_empty() {
        return format!("env {unset_env}");
    }
    match engine {
        "claude" => format!(
            "env {unset_env} CLAUDE_CONFIG_DIR={} ANTHROPIC_CONFIG_DIR={}",
            sh_quote(profile),
            sh_quote(&format!("{profile}/anthropic"))
        ),
        "codex" => format!("env {unset_env} CODEX_HOME={}", sh_quote(profile)),
        _ => format!("env {unset_env}"),
    }
}

fn cmd_missing_condition(raw_cmd: &str) -> String {
    let quoted = sh_quote(raw_cmd);
    if raw_cmd.contains('/') {
        format!("[ ! -x {quoted} ]")
    } else {
        format!("! command -v {quoted} >/dev/null 2>&1")
    }
}

/// Construit le script shell de diagnostic léger pour toutes les connexions d'une machine.
/// Aucun secret n'est affiché : uniquement `id<TAB>state<TAB>detail`.
fn build_probe_script(targets: &[ConnectionTarget]) -> String {
    let mut script = String::from(
        "PATH=\"$HOME/.local/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:$PATH\"\n",
    );
    for t in targets {
        let id = sh_quote(&t.connection_id);
        let cmd = sh_quote(&t.command);
        let missing = cmd_missing_condition(&t.command);
        let profile = t.profile_dir.as_deref().unwrap_or("");
        let env_prefix = env_prefix_for(&t.engine, profile);
        match t.engine.as_str() {
            "claude" => {
                script.push_str(&format!(
                    "if {missing}; then printf '%s\\tbinary_missing\\tbinaire absent\\n' {id}; else\n\
                     out=$({env_prefix} {cmd} auth status 2>&1 || true)\n\
                     if printf '%s' \"$out\" | grep -qiE '\"loggedIn\"[[:space:]]*:[[:space:]]*true|logged in|authenticated'; then\n\
                       email=$(printf '%s' \"$out\" | sed -n 's/.*\"email\"[[:space:]]*:[[:space:]]*\"\\([^\"]*\\)\".*/\\1/p' | head -n 1)\n\
                       printf '%s\\tauthenticated\\t%s\\n' {id} \"${{email:-connecte}}\"\n\
                     else\n\
                       printf '%s\\tunauthenticated\\tnon connecte\\n' {id}\n\
                     fi; fi\n"
                ));
            }
            "codex" => {
                script.push_str(&format!(
                    "if {missing}; then printf '%s\\tbinary_missing\\tbinaire absent\\n' {id}; else\n\
                     out=$({env_prefix} {cmd} login status 2>&1 || true)\n\
                     if printf '%s' \"$out\" | grep -qi 'logged in'; then\n\
                       line=$(printf '%s' \"$out\" | head -n 1 | tr '\\t\\r\\n' '   ')\n\
                       printf '%s\\tauthenticated\\t%s\\n' {id} \"$line\"\n\
                     else\n\
                       printf '%s\\tunauthenticated\\tnon connecte\\n' {id}\n\
                     fi; fi\n"
                ));
            }
            "antigravity" => {
                // Vérifie d'abord la présence du binaire puis une sonde légère de session
                // en nettoyant les variables interdites (CLEAR_AUTH_ENV).
                let dir_check = if profile.is_empty() {
                    format!(
                        "out=$({env_prefix} python3 -c 'import subprocess,sys\ntry:\n r=subprocess.run([sys.argv[1],\"models\"],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=4,text=True)\n print(r.returncode)\n print(r.stdout[:400])\nexcept Exception as e:\n print(99)\n print(e)' {cmd} 2>&1)\n\
                         code=$(printf '%s\\n' \"$out\" | head -n 1)\n\
                         if [ \"$code\" = \"0\" ] && printf '%s' \"$out\" | grep -q 'gemini-'; then\n\
                           cnt=$(printf '%s\\n' \"$out\" | grep -c 'gemini-' || true)\n\
                           printf '%s\\tauthenticated\\t%s modeles visibles\\n' {id} \"$cnt\"\n\
                         else\n\
                           printf '%s\\tunauthenticated\\tconnexion Google requise\\n' {id}\n\
                         fi"
                    )
                } else {
                    let p = sh_quote(profile);
                    format!(
                        "if [ -d {p} ] && [ -n \"$(find {p} -maxdepth 3 -type f \\( -name 'oauth_creds.json' -o -name 'google_accounts.json' -o -name 'jetski_state.pbtxt' -o -name 'credentials.json' \\) -size +0c 2>/dev/null | head -n 1)\" ]; then\n\
                           printf '%s\\tauthenticated\\tprofil isole present\\n' {id}\n\
                         else\n\
                           printf '%s\\tunauthenticated\\tprofil isole vide (connexion Google requise)\\n' {id}\n\
                         fi"
                    )
                };
                script.push_str(&format!(
                    "if {missing}; then printf '%s\\tbinary_missing\\tagy absent\\n' {id}; else\n{dir_check}\nfi\n"
                ));
            }
            _ => {
                script.push_str(&format!(
                    "printf '%s\\tunsupported\\tmoteur non gere\\n' {id}\n"
                ));
            }
        }
    }
    script
}

fn wait_child_output_bounded(
    mut child: Child,
    stdin_payload: Option<&[u8]>,
    timeout_secs: u64,
) -> std::io::Result<std::process::Output> {
    if let Some(payload) = stdin_payload
        && let Some(mut stdin) = child.stdin.take()
    {
        let _ = stdin.write_all(payload);
    }
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    loop {
        if child.try_wait()?.is_some() {
            return child.wait_with_output();
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "délai dépassé",
            ));
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn probe_machine(machine: String, targets: Vec<ConnectionTarget>) -> Vec<ConnectionStatus> {
    let is_local = targets.first().is_some_and(|t| t.is_local);
    let script = build_probe_script(&targets);
    let output = if is_local {
        Command::new("bash")
            .arg("-lc")
            .arg(&script)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .and_then(|child| wait_child_output_bounded(child, None, PROBE_TIMEOUT_SECS))
    } else if valid_machine_id(&machine) {
        Command::new("ssh")
            .args([
                "-o",
                "BatchMode=yes",
                "-o",
                &format!("ConnectTimeout={DEFAULT_SSH_CONNECT_TIMEOUT}"),
                &machine,
                "bash -s",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .and_then(|child| {
                wait_child_output_bounded(child, Some(script.as_bytes()), PROBE_TIMEOUT_SECS)
            })
    } else {
        Err(std::io::Error::other("identifiant machine invalide"))
    };

    let mut parsed: HashMap<String, (String, String)> = HashMap::new();
    let reachable = match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                let mut parts = line.splitn(3, '\t');
                if let (Some(id), Some(state), Some(detail)) =
                    (parts.next(), parts.next(), parts.next())
                {
                    parsed.insert(
                        id.to_string(),
                        (state.to_string(), detail.trim().to_string()),
                    );
                }
            }
            true
        }
        _ => false,
    };

    targets
        .into_iter()
        .map(|t| {
            let (auth_state, detail) = if !reachable {
                (
                    "machine_unreachable".to_string(),
                    "machine injoignable en SSH".to_string(),
                )
            } else if let Some((s, d)) = parsed.remove(&t.connection_id) {
                (s, d)
            } else {
                ("unknown".to_string(), "statut non retourne".to_string())
            };
            ConnectionStatus {
                connection_id: t.connection_id,
                machine_id: t.machine_id,
                engine: t.engine,
                agent_type: t.agent_type,
                enabled: t.enabled,
                display_name: t.display_name,
                managed: t.managed,
                profile_dir: t.profile_dir,
                is_local: t.is_local,
                auth_state,
                detail,
            }
        })
        .collect()
}

pub(crate) fn collect_fleet_status(
    registry_override: Option<&Path>,
) -> Result<FleetAuthStatus, String> {
    let (local_machine, targets) = load_fleet_connections(registry_override)?;
    let mut grouped: BTreeMap<String, Vec<ConnectionTarget>> = BTreeMap::new();
    for target in targets {
        grouped
            .entry(target.machine_id.clone())
            .or_default()
            .push(target);
    }
    let machines: Vec<String> = grouped.keys().cloned().collect();
    let handles: Vec<_> = grouped
        .into_iter()
        .map(|(machine, items)| std::thread::spawn(move || probe_machine(machine, items)))
        .collect();
    let mut connections = Vec::new();
    for handle in handles {
        if let Ok(mut rows) = handle.join() {
            connections.append(&mut rows);
        }
    }
    connections.sort_by(|a, b| {
        a.machine_id
            .cmp(&b.machine_id)
            .then_with(|| a.engine.cmp(&b.engine))
            .then_with(|| a.connection_id.cmp(&b.connection_id))
    });
    Ok(FleetAuthStatus {
        local_machine,
        checked_at: now_secs(),
        machines,
        connections,
    })
}

fn cmd_status(args: &[String]) -> Result<(), String> {
    let mut json_mode = false;
    let mut registry: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--registry" => {
                let value = args.get(i + 1).ok_or("--registry exige un chemin")?;
                registry = Some(PathBuf::from(value));
                i += 2;
            }
            other => return Err(format!("option inconnue : {other}")),
        }
    }
    let status = collect_fleet_status(registry.as_deref())?;
    if json_mode {
        println!(
            "{}",
            serde_json::to_string_pretty(&status).map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    println!(
        "Statut d'authentification des fournisseurs (depuis {}) :",
        status.local_machine
    );
    for row in &status.connections {
        let badge = match row.auth_state.as_str() {
            "authenticated" => "OK",
            "unauthenticated" => "A CONNECTER",
            "binary_missing" => "ABSENT",
            "machine_unreachable" => "HORS LIGNE",
            _ => "INCONNU",
        };
        let active = if row.enabled { "actif" } else { "desactive" };
        println!(
            "  [{:<11}] {:<14} {:<26} ({:<11}, {:<9}) : {}",
            badge, row.machine_id, row.connection_id, row.engine, active, row.detail
        );
    }
    Ok(())
}

/// Construit la commande shell de connexion (et optionnellement déconnexion préalable)
/// pour un moteur et un profil donné.
pub(crate) fn build_login_shell_command(target: &ConnectionTarget, logout_first: bool) -> String {
    let cmd = sh_quote(&target.command);
    let profile = target.profile_dir.as_deref().unwrap_or("");
    let env_prefix = env_prefix_for(&target.engine, profile);
    match target.engine.as_str() {
        "claude" => {
            if logout_first {
                format!(
                    "{env_prefix} {cmd} auth logout >/dev/null 2>&1 || true; {env_prefix} {cmd} auth login"
                )
            } else {
                format!("{env_prefix} {cmd} auth login")
            }
        }
        "codex" => {
            if logout_first {
                format!(
                    "{env_prefix} {cmd} logout >/dev/null 2>&1 || true; {env_prefix} {cmd} login"
                )
            } else {
                format!("{env_prefix} {cmd} login")
            }
        }
        "antigravity" => {
            let mut flags = String::from("acp");
            if !profile.is_empty() {
                flags.push_str(&format!(
                    " --gemini-dir {} --app-data-dir antigravity-bridget",
                    sh_quote(profile)
                ));
            }
            flags.push_str(" --login");
            format!("{env_prefix} {cmd} {flags}")
        }
        _ => format!(
            "printf 'Moteur non pris en charge : %s\\n' {} && exit 1",
            sh_quote(&target.engine)
        ),
    }
}

fn bridget_bin_quoted(is_local: bool) -> String {
    if is_local {
        let exe = std::env::current_exe()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "bridget".to_string());
        sh_quote(&exe)
    } else {
        "\"$HOME/.local/bin/bridget\"".to_string()
    }
}

fn parse_json_payload(raw: &str) -> Result<Value, String> {
    if let Ok(v) = serde_json::from_str::<Value>(raw.trim()) {
        return Ok(v);
    }
    // Tolère une éventuelle bannière de shell avant la ligne JSON.
    for line in raw.lines().rev() {
        let trimmed = line.trim();
        if trimmed.starts_with('{')
            && let Ok(v) = serde_json::from_str::<Value>(trimmed)
        {
            return Ok(v);
        }
    }
    Err(format!("sortie JSON illisible : {}", raw.trim()))
}

fn build_ssh_login_args(target: &ConnectionTarget, login_cmd: String) -> Vec<String> {
    let mut ssh_args = vec![
        "-tt".to_string(),
        "-o".to_string(),
        "ServerAliveInterval=15".to_string(),
        "-o".to_string(),
        format!("ConnectTimeout={DEFAULT_SSH_CONNECT_TIMEOUT}"),
    ];
    if target.engine == "codex" {
        // Redirige le callback OAuth local de Codex (port 1455) vers le serveur distant.
        ssh_args.push("-L".to_string());
        ssh_args.push("1455:127.0.0.1:1455".to_string());
    }
    ssh_args.push(target.machine_id.clone());
    ssh_args.push(login_cmd);
    ssh_args
}

/// Prépare (preview + apply) un nouveau compte isolé sur la machine locale ou distante.
pub(crate) fn prepare_isolated_account(
    source: &ConnectionTarget,
    new_id: &str,
) -> Result<String, String> {
    if new_id.is_empty()
        || new_id.len() > 48
        || !new_id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_-".contains(&b))
        || !new_id.as_bytes()[0].is_ascii_alphanumeric()
    {
        return Err(
            "identifiant de compte invalide (minuscules, chiffres, - ou _, 1..48 caracteres)"
                .into(),
        );
    }
    let reg = sh_quote(&source.registry_path);
    let from = sh_quote(&source.connection_id);
    let id = sh_quote(new_id);
    let bin = bridget_bin_quoted(source.is_local);
    let preview_cmd = format!("{bin} accounts prepare --registry {reg} --from {from} --id {id}");
    let preview_out = run_shell_capture(source, &preview_cmd)?;
    let parsed =
        parse_json_payload(&preview_out).map_err(|e| format!("aperçu prepare illisible : {e}"))?;
    if parsed.get("status").and_then(Value::as_str) != Some("preview") {
        let msg = parsed
            .get("next_action")
            .or_else(|| parsed.get("code"))
            .and_then(Value::as_str)
            .unwrap_or(&preview_out);
        return Err(format!("préparation refusée : {msg}"));
    }
    let expected = parsed
        .get("expected")
        .and_then(Value::as_str)
        .ok_or("SHA attendu absent de l'aperçu")?;
    let apply_cmd = format!(
        "{bin} accounts prepare --registry {reg} --from {from} --id {id} --expected {} --apply",
        sh_quote(expected)
    );
    let (ok, apply_out) = run_shell_pty_capture(source, &apply_cmd)?;
    if !ok || !apply_out.contains("applied_local") {
        return Err(format!("application refusée : {apply_out}"));
    }
    Ok(format!("account-{new_id}"))
}

/// Active un compte isolé existant (`accounts enable`) sur sa machine cible.
pub(crate) fn enable_isolated_account(target: &ConnectionTarget) -> Result<(), String> {
    let raw_id = target
        .connection_id
        .strip_prefix("account-")
        .ok_or("seuls les comptes isolés account-* s'activent via accounts enable")?;
    let reg = sh_quote(&target.registry_path);
    let id = sh_quote(raw_id);
    let bin = bridget_bin_quoted(target.is_local);
    let preview_cmd = format!("{bin} accounts enable --registry {reg} --id {id}");
    let preview_out = run_shell_capture(target, &preview_cmd)?;
    let parsed =
        parse_json_payload(&preview_out).map_err(|e| format!("aperçu enable illisible : {e}"))?;
    let expected = parsed
        .get("expected")
        .and_then(Value::as_str)
        .ok_or("SHA attendu absent pour enable")?;
    let apply_cmd = format!(
        "{bin} accounts enable --registry {reg} --id {id} --expected {} --apply",
        sh_quote(expected)
    );
    let (ok, apply_out) = run_shell_pty_capture(target, &apply_cmd)?;
    if !ok || !apply_out.contains("applied_local") {
        return Err(format!("activation refusée : {apply_out}"));
    }
    Ok(())
}

fn run_shell_capture(target: &ConnectionTarget, shell_cmd: &str) -> Result<String, String> {
    let output = if target.is_local {
        Command::new("bash")
            .args(["-lc", shell_cmd])
            .stdin(Stdio::null())
            .output()
            .map_err(|e| e.to_string())?
    } else {
        if !valid_machine_id(&target.machine_id) {
            return Err("machine cible invalide".into());
        }
        Command::new("ssh")
            .args([
                "-o",
                "BatchMode=yes",
                "-o",
                &format!("ConnectTimeout={DEFAULT_SSH_CONNECT_TIMEOUT}"),
                &target.machine_id,
                shell_cmd,
            ])
            .stdin(Stdio::null())
            .output()
            .map_err(|e| e.to_string())?
    };
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if !output.status.success() && stdout.trim().is_empty() {
        return Err(format!("échec commande : {stderr}"));
    }
    Ok(stdout)
}

fn run_shell_pty_capture(
    target: &ConnectionTarget,
    shell_cmd: &str,
) -> Result<(bool, String), String> {
    let output = if target.is_local {
        #[cfg(target_os = "macos")]
        {
            Command::new("/usr/bin/script")
                .args(["-q", "/dev/null", "bash", "-lc", shell_cmd])
                .stdin(Stdio::null())
                .output()
                .map_err(|e| e.to_string())?
        }
        #[cfg(not(target_os = "macos"))]
        {
            Command::new("/usr/bin/script")
                .args(["-q", "-e", "-c", shell_cmd, "/dev/null"])
                .stdin(Stdio::null())
                .output()
                .map_err(|e| e.to_string())?
        }
    } else {
        if !valid_machine_id(&target.machine_id) {
            return Err("machine cible invalide".into());
        }
        Command::new("ssh")
            .args([
                "-tt",
                "-o",
                "BatchMode=yes",
                "-o",
                &format!("ConnectTimeout={DEFAULT_SSH_CONNECT_TIMEOUT}"),
                &target.machine_id,
                shell_cmd,
            ])
            .stdin(Stdio::null())
            .output()
            .map_err(|e| e.to_string())?
    };
    Ok((
        output.status.success(),
        strip_ansi(&String::from_utf8_lossy(&output.stdout)),
    ))
}

fn open_browser(url: &str) {
    #[cfg(target_os = "macos")]
    let _ = Command::new("open").arg(url).spawn();
    #[cfg(not(target_os = "macos"))]
    let _ = Command::new("xdg-open").arg(url).spawn();
}

fn cmd_login(args: &[String]) -> Result<(), String> {
    let mut connection: Option<String> = None;
    let mut machine: Option<String> = None;
    let mut from: Option<String> = None;
    let mut prepare_id: Option<String> = None;
    let mut logout_first = false;
    let mut registry: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--connection" | "--id" => {
                connection = Some(
                    args.get(i + 1)
                        .ok_or("--connection exige une valeur")?
                        .clone(),
                );
                i += 2;
            }
            "--machine" => {
                machine = Some(args.get(i + 1).ok_or("--machine exige une valeur")?.clone());
                i += 2;
            }
            "--from" => {
                from = Some(args.get(i + 1).ok_or("--from exige une valeur")?.clone());
                i += 2;
            }
            "--prepare-id" => {
                prepare_id = Some(
                    args.get(i + 1)
                        .ok_or("--prepare-id exige une valeur")?
                        .clone(),
                );
                i += 2;
            }
            "--registry" => {
                registry = Some(PathBuf::from(
                    args.get(i + 1).ok_or("--registry exige un chemin")?,
                ));
                i += 2;
            }
            "--logout-first" => {
                logout_first = true;
                i += 1;
            }
            other => return Err(format!("option inconnue : {other}")),
        }
    }

    let (_, mut targets) = load_fleet_connections(registry.as_deref())?;
    let mut created_account = false;
    let target = if let (Some(from_conn), Some(new_id)) = (from.as_deref(), prepare_id.as_deref()) {
        let source = find_target(&targets, from_conn, machine.as_deref())?;
        println!(
            "Préparation du compte isolé '{}' sur {} depuis {}...",
            new_id, source.machine_id, source.connection_id
        );
        let created_id = prepare_isolated_account(&source, new_id)?;
        created_account = true;
        let (_, refreshed) = load_fleet_connections(registry.as_deref())?;
        targets = refreshed;
        find_target(&targets, &created_id, Some(&source.machine_id))?
    } else if let Some(conn_id) = connection.as_deref() {
        find_target(&targets, conn_id, machine.as_deref())?
    } else {
        return Err(
            "préciser --connection <ID> [--machine <M>] ou --from <ID> --prepare-id <NOM> (ou utiliser bridget accounts ui)"
                .into(),
        );
    };

    let login_cmd = build_login_shell_command(&target, logout_first);
    println!(
        "Connexion {} ({}) sur {}...",
        target.connection_id, target.engine, target.machine_id
    );
    let status = if target.is_local {
        Command::new("bash")
            .args(["-lc", &login_cmd])
            .status()
            .map_err(|e| e.to_string())?
    } else {
        if !valid_machine_id(&target.machine_id) {
            return Err("machine cible invalide".into());
        }
        let ssh_args = build_ssh_login_args(&target, login_cmd);
        Command::new("ssh")
            .args(&ssh_args)
            .status()
            .map_err(|e| e.to_string())?
    };

    if !status.success() {
        return Err(format!("la connexion a échoué (code {:?})", status.code()));
    }
    if created_account {
        println!(
            "Activation du nouveau compte isolé {}...",
            target.connection_id
        );
        enable_isolated_account(&target)?;
    }
    println!(
        "Connexion terminée pour {} sur {}.",
        target.connection_id, target.machine_id
    );
    Ok(())
}

fn find_target(
    targets: &[ConnectionTarget],
    connection_id: &str,
    machine: Option<&str>,
) -> Result<ConnectionTarget, String> {
    let matches: Vec<_> = targets
        .iter()
        .filter(|t| t.connection_id == connection_id && machine.is_none_or(|m| t.machine_id == m))
        .cloned()
        .collect();
    match matches.len() {
        0 => Err(format!("connexion introuvable : {connection_id}")),
        1 => Ok(matches.into_iter().next().unwrap()),
        _ => Err(format!(
            "connexion '{connection_id}' présente sur plusieurs machines ; préciser --machine"
        )),
    }
}

// ---------------------------------------------------------------------------
// Interface graphique locale (`bridget accounts ui`)
// ---------------------------------------------------------------------------

struct LoginSession {
    engine: String,
    log: String,
    auth_url: Option<String>,
    browser_opened: bool,
    open_in_browser: bool,
    code_sent: bool,
    code_auto_sent: bool,
    logged_in_as: Option<String>,
    running: bool,
    exit_code: Option<i32>,
    stdin: Option<ChildStdin>,
    child: Arc<Mutex<Option<Child>>>,
    enable_on_success: Option<ConnectionTarget>,
    auto_enabled: bool,
}

/// Reconnaît strictement un code ou une URL de retour OAuth pour Antigravity ou Claude.
/// Évite tout faux positif sur du texte ordinaire présent dans le presse-papiers.
pub(crate) fn looks_like_oauth_clipboard_code(engine: &str, raw: &str) -> bool {
    let trimmed = raw.trim();
    if trimmed.len() < 24 || trimmed.len() > 2048 || trimmed.chars().any(char::is_whitespace) {
        return false;
    }
    match engine {
        "antigravity" => {
            (trimmed.starts_with("4/0A") || trimmed.starts_with("4/")) && trimmed.len() >= 30
                || (trimmed.starts_with("https://antigravity.google/oauth-callback?")
                    && trimmed.contains("code="))
        }
        "claude" => {
            if trimmed.starts_with("https://platform.claude.com/oauth/code/callback?")
                && trimmed.contains("code=")
            {
                return true;
            }
            if let Some((left, right)) = trimmed.split_once('#') {
                left.len() >= 20
                    && right.len() >= 20
                    && left
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
                    && right
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Extrait l'identifiant du compte affiché par le CLI après connexion réussie
/// (ex. `Logged in as fullflare@gmail.com.`).
pub(crate) fn extract_logged_in_account(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    let idx = lower.find("logged in as ")?;
    let after = &line[idx + "logged in as ".len()..];
    let candidate = after
        .split_whitespace()
        .next()?
        .trim_end_matches(['.', ',', ';', ')', '"']);
    if candidate.contains('@') && candidate.len() >= 5 && candidate.len() <= 160 {
        Some(candidate.to_string())
    } else {
        None
    }
}

#[cfg(target_os = "macos")]
fn read_local_clipboard() -> Option<String> {
    let output = Command::new("/usr/bin/pbpaste")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

#[cfg(target_os = "macos")]
fn clipboard_fingerprint(text: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

/// Sur macOS, surveille le presse-papiers uniquement pendant une session de connexion
/// active (Antigravity/Claude) après ouverture du navigateur : dès que l'utilisateur
/// clique sur « Copier » sur la page Google/Claude, le code OAuth est transmis
/// automatiquement au processus de connexion, sans rien journaliser.
#[cfg(target_os = "macos")]
fn start_clipboard_watcher(session: Arc<Mutex<LoginSession>>, initial_fp: Option<String>) {
    std::thread::spawn(move || {
        let mut last_fp = initial_fp;
        for _ in 0..500 {
            std::thread::sleep(Duration::from_millis(600));
            let (running, code_sent, engine) = {
                let guard = session.lock().unwrap_or_else(|e| e.into_inner());
                (guard.running, guard.code_sent, guard.engine.clone())
            };
            if !running || code_sent {
                break;
            }
            let Some(clip) = read_local_clipboard() else {
                continue;
            };
            let fp = clipboard_fingerprint(&clip);
            if last_fp.as_deref() == Some(fp.as_str()) {
                continue;
            }
            last_fp = Some(fp);
            if looks_like_oauth_clipboard_code(&engine, &clip) {
                let payload = format!("{}\n", clip.trim());
                let mut guard = session.lock().unwrap_or_else(|e| e.into_inner());
                if !guard.running || guard.code_sent {
                    break;
                }
                if let Some(stdin) = guard.stdin.as_mut()
                    && stdin
                        .write_all(payload.as_bytes())
                        .and_then(|_| stdin.flush())
                        .is_ok()
                {
                    guard.code_sent = true;
                    guard.code_auto_sent = true;
                    guard
                        .log
                        .push_str("\n[Bridget] Code OAuth detecte dans le presse-papiers et envoye automatiquement.\n");
                    break;
                }
            }
        }
    });
}

#[cfg(not(target_os = "macos"))]
fn start_clipboard_watcher(_session: Arc<Mutex<LoginSession>>, _initial_fp: Option<String>) {}

#[derive(Clone)]
struct UiState {
    token: String,
    registry: Option<PathBuf>,
    sessions: Arc<Mutex<HashMap<String, Arc<Mutex<LoginSession>>>>>,
}

#[derive(Deserialize)]
struct TokenQuery {
    token: Option<String>,
    session: Option<String>,
    offset: Option<usize>,
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize)]
struct StartLoginRequest {
    connection_id: String,
    machine_id: String,
    #[serde(default)]
    logout_first: bool,
    #[serde(default)]
    enable_on_success: bool,
    #[serde(default = "default_true")]
    open_in_browser: bool,
}

#[derive(Deserialize)]
struct InputLoginRequest {
    session_id: String,
    text: String,
}

#[derive(Deserialize)]
struct CancelLoginRequest {
    session_id: String,
}

#[derive(Deserialize)]
struct PrepareRequest {
    from_connection_id: String,
    machine_id: String,
    new_id: String,
}

fn check_token(state: &UiState, headers: &HeaderMap, query: &TokenQuery) -> bool {
    let header_ok = headers
        .get("x-bridget-token")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v == state.token);
    let query_ok = query.token.as_deref().is_some_and(|v| v == state.token);
    let cookie_ok = headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|raw| {
            raw.split(';').any(|part| {
                part.trim()
                    .strip_prefix("bridget_ui_token=")
                    .is_some_and(|v| v == state.token)
            })
        });
    header_ok || query_ok || cookie_ok
}

fn spawn_pty_login(target: &ConnectionTarget, logout_first: bool) -> Result<Child, String> {
    let login_cmd = build_login_shell_command(target, logout_first);
    let mut command = if target.is_local {
        #[cfg(target_os = "macos")]
        {
            let mut c = Command::new("/usr/bin/script");
            c.args(["-q", "/dev/null", "bash", "-lc", &login_cmd]);
            c
        }
        #[cfg(not(target_os = "macos"))]
        {
            let mut c = Command::new("/usr/bin/script");
            c.args(["-q", "-e", "-c", &login_cmd, "/dev/null"]);
            c
        }
    } else {
        if !valid_machine_id(&target.machine_id) {
            return Err("machine cible invalide".into());
        }
        let ssh_args = build_ssh_login_args(target, login_cmd);
        #[cfg(target_os = "macos")]
        {
            let mut c = Command::new("/usr/bin/script");
            c.args(["-q", "/dev/null", "ssh"]);
            c.args(&ssh_args);
            c
        }
        #[cfg(not(target_os = "macos"))]
        {
            let mut c = Command::new("ssh");
            c.args(&ssh_args);
            c
        }
    };
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("impossible de lancer la session de connexion : {e}"))
}

async fn ui_index(
    State(state): State<UiState>,
    headers: HeaderMap,
    Query(query): Query<TokenQuery>,
) -> impl IntoResponse {
    if !check_token(&state, &headers, &query) {
        return (
            StatusCode::UNAUTHORIZED,
            HeaderMap::new(),
            Html("Acces refuse : jeton de session requis.".to_string()),
        );
    }
    let mut resp_headers = HeaderMap::new();
    if let Ok(cookie) = axum::http::HeaderValue::from_str(&format!(
        "bridget_ui_token={}; HttpOnly; SameSite=Strict; Path=/",
        state.token
    )) {
        resp_headers.insert(axum::http::header::SET_COOKIE, cookie);
    }
    resp_headers.insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::HeaderValue::from_static("no-store"),
    );
    let page = UI_HTML.replace("__BRIDGET_TOKEN__", &state.token);
    (StatusCode::OK, resp_headers, Html(page))
}

async fn api_status(
    State(state): State<UiState>,
    headers: HeaderMap,
    Query(query): Query<TokenQuery>,
) -> impl IntoResponse {
    if !check_token(&state, &headers, &query) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized"})),
        );
    }
    let registry = state.registry.clone();
    let res = tokio::task::spawn_blocking(move || collect_fleet_status(registry.as_deref())).await;
    match res {
        Ok(Ok(status)) => (
            StatusCode::OK,
            Json(serde_json::to_value(status).unwrap_or_default()),
        ),
        Ok(Err(e)) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn api_prepare(
    State(state): State<UiState>,
    headers: HeaderMap,
    Query(query): Query<TokenQuery>,
    Json(req): Json<PrepareRequest>,
) -> impl IntoResponse {
    if !check_token(&state, &headers, &query) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized"})),
        );
    }
    let registry = state.registry.clone();
    let res = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let (_, targets) = load_fleet_connections(registry.as_deref())?;
        let source = find_target(&targets, &req.from_connection_id, Some(&req.machine_id))?;
        let new_conn_id = prepare_isolated_account(&source, &req.new_id)?;
        Ok(json!({
            "status": "prepared",
            "connection_id": new_conn_id,
            "machine_id": source.machine_id,
            "engine": source.engine
        }))
    })
    .await;
    match res {
        Ok(Ok(val)) => (StatusCode::OK, Json(val)),
        Ok(Err(e)) => (StatusCode::BAD_REQUEST, Json(json!({"error": e}))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        ),
    }
}

async fn api_login_start(
    State(state): State<UiState>,
    headers: HeaderMap,
    Query(query): Query<TokenQuery>,
    Json(req): Json<StartLoginRequest>,
) -> impl IntoResponse {
    if !check_token(&state, &headers, &query) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized"})),
        );
    }
    let registry = state.registry.clone();
    let target_res = tokio::task::spawn_blocking(move || {
        let (_, targets) = load_fleet_connections(registry.as_deref())?;
        find_target(&targets, &req.connection_id, Some(&req.machine_id))
    })
    .await;
    let target = match target_res {
        Ok(Ok(t)) => t,
        Ok(Err(e)) => return (StatusCode::BAD_REQUEST, Json(json!({"error": e}))),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            );
        }
    };

    let mut child = match spawn_pty_login(&target, req.logout_first) {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e}))),
    };
    let stdin = child.stdin.take();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let child_handle = Arc::new(Mutex::new(Some(child)));

    let session_id = uuid::Uuid::new_v4().to_string();
    let session = Arc::new(Mutex::new(LoginSession {
        engine: target.engine.clone(),
        log: format!(
            "Session ouverte pour {} ({}) sur {}...\n",
            target.connection_id, target.engine, target.machine_id
        ),
        auth_url: None,
        browser_opened: false,
        open_in_browser: req.open_in_browser,
        code_sent: false,
        code_auto_sent: false,
        logged_in_as: None,
        running: true,
        exit_code: None,
        stdin,
        child: child_handle.clone(),
        enable_on_success: req.enable_on_success.then_some(target.clone()),
        auto_enabled: false,
    }));
    {
        let mut map = state.sessions.lock().unwrap_or_else(|e| e.into_inner());
        if map.len() >= 16 {
            map.retain(|_, s| s.lock().unwrap_or_else(|e| e.into_inner()).running);
        }
        map.insert(session_id.clone(), session.clone());
    }

    if let Some(out) = stdout {
        let s = session.clone();
        std::thread::spawn(move || pump_stream(out, s));
    }
    if let Some(err) = stderr {
        let s = session.clone();
        std::thread::spawn(move || pump_stream(err, s));
    }
    {
        let s = session.clone();
        std::thread::spawn(move || {
            // Attend la fin du processus enfant sans bloquer le verrou pendant le sommeil,
            // ce qui garantit que `api_login_cancel` ne peut jamais cibler un PID déjà récolté.
            let exit = loop {
                let maybe_status = {
                    let mut guard = child_handle.lock().unwrap_or_else(|e| e.into_inner());
                    match guard.as_mut() {
                        Some(c) => match c.try_wait() {
                            Ok(Some(st)) => {
                                *guard = None;
                                Some(st.code().unwrap_or(-1))
                            }
                            Ok(None) => None,
                            Err(_) => {
                                *guard = None;
                                Some(-1)
                            }
                        },
                        None => Some(-1),
                    }
                };
                if let Some(code) = maybe_status {
                    break code;
                }
                std::thread::sleep(Duration::from_millis(80));
            };
            let enable_target = {
                let mut guard = s.lock().unwrap_or_else(|e| e.into_inner());
                guard.running = false;
                guard.exit_code = Some(exit);
                guard.stdin = None;
                if exit == 0 {
                    guard.log.push_str("\nConnexion terminee avec succes.\n");
                    guard.enable_on_success.clone()
                } else {
                    guard
                        .log
                        .push_str(&format!("\nSession terminee (code {exit}).\n"));
                    None
                }
            };
            if let Some(t) = enable_target {
                match enable_isolated_account(&t) {
                    Ok(()) => {
                        let mut guard = s.lock().unwrap_or_else(|e| e.into_inner());
                        guard.auto_enabled = true;
                        guard
                            .log
                            .push_str("Compte isole active dans le registre.\n");
                    }
                    Err(e) => {
                        let mut guard = s.lock().unwrap_or_else(|e| e.into_inner());
                        guard
                            .log
                            .push_str(&format!("Activation automatique echouee : {e}\n"));
                    }
                }
            }
        });
    }

    (
        StatusCode::OK,
        Json(json!({
            "session_id": session_id,
            "connection_id": target.connection_id,
            "machine_id": target.machine_id,
            "engine": target.engine
        })),
    )
}

fn pump_stream<R: std::io::Read>(reader: R, session: Arc<Mutex<LoginSession>>) {
    let mut buf = BufReader::new(reader);
    let mut line = Vec::new();
    loop {
        line.clear();
        match buf.read_until(b'\n', &mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let text = strip_ansi(&String::from_utf8_lossy(&line));
                let detected_url = extract_auth_url(&text);
                let detected_account = extract_logged_in_account(&text);
                let mut url_to_open = None;
                let mut open_via_os = false;
                let mut watch_engine = false;
                {
                    let mut guard = session.lock().unwrap_or_else(|e| e.into_inner());
                    if guard.log.len() < MAX_SESSION_LOG_BYTES {
                        guard.log.push_str(&text);
                    }
                    if let Some(acc) = detected_account {
                        guard.logged_in_as = Some(acc);
                    }
                    if let Some(url) = detected_url {
                        guard.auth_url = Some(url.clone());
                        if !guard.browser_opened {
                            guard.browser_opened = true;
                            url_to_open = Some(url);
                            open_via_os = guard.open_in_browser;
                            watch_engine =
                                matches!(guard.engine.as_str(), "antigravity" | "claude");
                        }
                    }
                }
                if let Some(url) = url_to_open {
                    if watch_engine {
                        #[cfg(target_os = "macos")]
                        let initial_fp = read_local_clipboard().map(|c| clipboard_fingerprint(&c));
                        #[cfg(not(target_os = "macos"))]
                        let initial_fp = None;
                        start_clipboard_watcher(session.clone(), initial_fp);
                    }
                    if open_via_os {
                        open_browser(&url);
                    }
                }
            }
        }
    }
}

async fn api_login_poll(
    State(state): State<UiState>,
    headers: HeaderMap,
    Query(query): Query<TokenQuery>,
) -> impl IntoResponse {
    if !check_token(&state, &headers, &query) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized"})),
        );
    }
    let Some(session_id) = query.session.as_deref() else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "session requis"})),
        );
    };
    let session = {
        state
            .sessions
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(session_id)
            .cloned()
    };
    let Some(session) = session else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "session inconnue"})),
        );
    };
    let guard = session.lock().unwrap_or_else(|e| e.into_inner());
    let offset = query.offset.unwrap_or(0).min(guard.log.len());
    // Assurer une frontière UTF-8 valide.
    let mut safe_offset = offset;
    while safe_offset > 0 && !guard.log.is_char_boundary(safe_offset) {
        safe_offset -= 1;
    }
    let chunk = &guard.log[safe_offset..];
    (
        StatusCode::OK,
        Json(json!({
            "running": guard.running,
            "exit_code": guard.exit_code,
            "auth_url": guard.auth_url,
            "engine": guard.engine,
            "code_sent": guard.code_sent,
            "code_auto_sent": guard.code_auto_sent,
            "logged_in_as": guard.logged_in_as,
            "auto_enabled": guard.auto_enabled,
            "chunk": chunk,
            "next_offset": guard.log.len()
        })),
    )
}

async fn api_login_input(
    State(state): State<UiState>,
    headers: HeaderMap,
    Query(query): Query<TokenQuery>,
    Json(req): Json<InputLoginRequest>,
) -> impl IntoResponse {
    if !check_token(&state, &headers, &query) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized"})),
        );
    }
    let session = {
        state
            .sessions
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&req.session_id)
            .cloned()
    };
    let Some(session) = session else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "session inconnue"})),
        );
    };
    let mut guard = session.lock().unwrap_or_else(|e| e.into_inner());
    let Some(stdin) = guard.stdin.as_mut() else {
        return (
            StatusCode::CONFLICT,
            Json(json!({"error": "session fermee"})),
        );
    };
    let payload = format!("{}\n", req.text.trim_end_matches(&['\r', '\n'][..]));
    if let Err(e) = stdin
        .write_all(payload.as_bytes())
        .and_then(|_| stdin.flush())
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        );
    }
    guard.code_sent = true;
    (StatusCode::OK, Json(json!({"status": "sent"})))
}

async fn api_login_cancel(
    State(state): State<UiState>,
    headers: HeaderMap,
    Query(query): Query<TokenQuery>,
    Json(req): Json<CancelLoginRequest>,
) -> impl IntoResponse {
    if !check_token(&state, &headers, &query) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "unauthorized"})),
        );
    }
    let session = {
        state
            .sessions
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&req.session_id)
            .cloned()
    };
    let Some(session) = session else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "session inconnue"})),
        );
    };
    let child_handle = session
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .child
        .clone();
    {
        let mut child_guard = child_handle.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(child) = child_guard.as_mut() {
            // Envoie SIGTERM sous verrou du Child (non encore récolté -> zéro réutilisation PID).
            let pid = child.id();
            let _ = Command::new("kill")
                .args(["-TERM", &pid.to_string()])
                .status();
        }
    }
    (StatusCode::OK, Json(json!({"status": "cancelled"})))
}

fn build_ui_router(state: UiState) -> Router {
    Router::new()
        .route("/", get(ui_index))
        .route("/api/status", get(api_status))
        .route("/api/prepare", post(api_prepare))
        .route("/api/login/start", post(api_login_start))
        .route("/api/login/poll", get(api_login_poll))
        .route("/api/login/input", post(api_login_input))
        .route("/api/login/cancel", post(api_login_cancel))
        .with_state(state)
}

fn cmd_ui(args: &[String]) -> Result<(), String> {
    let mut no_open = false;
    let mut port: u16 = 0;
    let mut registry: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--no-open" => {
                no_open = true;
                i += 1;
            }
            "--port" => {
                port = args
                    .get(i + 1)
                    .ok_or("--port exige un numero")?
                    .parse()
                    .map_err(|_| "--port invalide")?;
                i += 2;
            }
            "--registry" => {
                registry = Some(PathBuf::from(
                    args.get(i + 1).ok_or("--registry exige un chemin")?,
                ));
                i += 2;
            }
            other => return Err(format!("option inconnue : {other}")),
        }
    }

    let token = uuid::Uuid::new_v4().simple().to_string();
    let state = UiState {
        token: token.clone(),
        registry,
        sessions: Arc::new(Mutex::new(HashMap::new())),
    };
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("runtime tokio indisponible : {e}"))?;

    rt.block_on(async move {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
            .await
            .map_err(|e| format!("ecoute locale impossible : {e}"))?;
        let addr = listener.local_addr().map_err(|e| e.to_string())?;
        let url = format!("http://127.0.0.1:{}/?token={token}", addr.port());
        println!("Interface Bridget Comptes prete sur : {url}");
        println!("Ctrl+C pour fermer cette fenetre de gestion.");
        if !no_open {
            let open_url = url.clone();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(150)).await;
                open_browser(&open_url);
            });
        }
        axum::serve(listener, build_ui_router(state))
            .await
            .map_err(|e| e.to_string())
    })
}

const UI_HTML: &str = r###"<!doctype html>
<html lang="fr">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width,initial-scale=1" />
<title>Bridget Comptes - Connexions fournisseurs</title>
<style>
  :root {
    --bg: #0f1117; --card: #181c25; --border: #2a3142; --text: #edf2f7;
    --muted: #9aa5b8; --accent: #4f8cff; --ok: #22c55e; --warn: #f59e0b; --err: #ef4444;
  }
  * { box-sizing: border-box; }
  body { margin: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; background: var(--bg); color: var(--text); }
  header { display: flex; align-items: center; justify-content: space-between; padding: 18px 24px; border-bottom: 1px solid var(--border); background: #131720; position: sticky; top: 0; z-index: 5; }
  header h1 { font-size: 18px; margin: 0; font-weight: 600; }
  header .sub { font-size: 13px; color: var(--muted); margin-top: 2px; }
  .toolbar { display: flex; gap: 10px; align-items: center; }
  button { cursor: pointer; border: 1px solid var(--border); background: #222838; color: var(--text); padding: 8px 14px; border-radius: 8px; font-size: 13px; font-weight: 500; }
  button:hover { background: #2c3448; }
  button.primary { background: var(--accent); border-color: var(--accent); color: #fff; }
  button.primary:hover { filter: brightness(1.08); }
  button.warn { background: #3b2a12; border-color: #6b4715; color: #fcd34d; }
  main { max-width: 1120px; margin: 0 auto; padding: 20px 24px 60px; }
  .filters { display: flex; gap: 8px; flex-wrap: wrap; margin-bottom: 18px; }
  .chip { padding: 6px 12px; border-radius: 999px; font-size: 12px; background: var(--card); border: 1px solid var(--border); color: var(--muted); }
  .chip.active { background: var(--accent); border-color: var(--accent); color: #fff; }
  .machine-block { margin-bottom: 24px; }
  .machine-title { font-size: 15px; font-weight: 600; margin: 0 0 10px; display: flex; align-items: center; justify-content: space-between; color: #cbd5e1; }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(310px, 1fr)); gap: 12px; }
  .card { background: var(--card); border: 1px solid var(--border); border-radius: 12px; padding: 14px 16px; display: flex; flex-direction: column; gap: 10px; }
  .row { display: flex; justify-content: space-between; align-items: center; gap: 8px; }
  .conn-name { font-weight: 600; font-size: 14px; word-break: break-word; }
  .meta { font-size: 12px; color: var(--muted); }
  .pill { font-size: 11px; font-weight: 600; padding: 3px 9px; border-radius: 999px; white-space: nowrap; }
  .pill.authenticated { background: rgba(34,197,94,.16); color: #4ade80; }
  .pill.unauthenticated { background: rgba(245,158,11,.18); color: #fbbf24; }
  .pill.binary_missing, .pill.machine_unreachable { background: rgba(239,68,68,.16); color: #f87171; }
  .actions { display: flex; gap: 8px; flex-wrap: wrap; margin-top: auto; }
  .modal-bg { position: fixed; inset: 0; background: rgba(0,0,0,.7); display: none; align-items: center; justify-content: center; padding: 20px; z-index: 20; }
  .modal-bg.open { display: flex; }
  .modal { background: #141822; border: 1px solid var(--border); border-radius: 14px; width: 100%; max-width: 580px; padding: 22px; display: flex; flex-direction: column; gap: 16px; max-height: 88vh; overflow-y: auto; }
  .banner { border-radius: 10px; padding: 14px 16px; display: flex; align-items: center; gap: 12px; font-size: 14px; line-height: 1.45; }
  .banner.info { background: rgba(79,140,255,.14); border: 1px solid rgba(79,140,255,.45); color: #dbeafe; }
  .banner.ok { background: rgba(34,197,94,.16); border: 1px solid rgba(34,197,94,.55); color: #dcfce7; }
  .banner.err { background: rgba(239,68,68,.16); border: 1px solid rgba(239,68,68,.55); color: #fee2e2; }
  .banner-icon { font-size: 20px; flex: 0 0 auto; }
  .banner-title { font-weight: 600; font-size: 15px; }
  .banner-sub { font-size: 13px; opacity: .9; margin-top: 2px; }
  .steps { display: flex; flex-direction: column; gap: 10px; }
  .step { background: #191f2d; border: 1px solid var(--border); border-radius: 10px; padding: 12px 14px; display: flex; align-items: center; justify-content: space-between; gap: 12px; }
  .step-num { width: 24px; height: 24px; border-radius: 999px; background: rgba(79,140,255,.22); color: #93c5fd; font-weight: 700; font-size: 12px; display: inline-flex; align-items: center; justify-content: center; flex: 0 0 auto; }
  .step-body { flex: 1; font-size: 13px; line-height: 1.4; }
  .step-body strong { display: block; margin-bottom: 2px; }
  .step-body span { color: var(--muted); font-size: 12px; }
  details.tech-log { border: 1px solid var(--border); border-radius: 8px; background: #0c0f16; }
  details.tech-log summary { cursor: pointer; padding: 8px 12px; font-size: 12px; color: var(--muted); user-select: none; }
  .log { background: #0a0c10; border-top: 1px solid var(--border); padding: 10px 12px; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 11px; line-height: 1.45; white-space: pre-wrap; word-break: break-word; overflow-y: auto; max-height: 220px; color: #cbd5e1; }
  button.success { background: #16a34a; border-color: #16a34a; color: #fff; font-weight: 600; }
  button.success:hover { filter: brightness(1.08); }
  .hidden { display: none !important; }
  input, select { background: #0c0f16; border: 1px solid var(--border); color: var(--text); padding: 9px 11px; border-radius: 8px; font-size: 13px; width: 100%; }
  .input-row { display: flex; gap: 8px; }
  label { font-size: 12px; color: var(--muted); display: block; margin-bottom: 4px; }
</style>
</head>
<body>
<header>
  <div>
    <h1>Bridget Comptes</h1>
    <div class="sub" id="subtitle">Chargement des connexions sur vos machines...</div>
  </div>
  <div class="toolbar">
    <button onclick="openPrepareModal()">+ Nouveau compte isole</button>
    <button class="primary" onclick="refreshStatus()">Actualiser</button>
  </div>
</header>
<main>
  <div class="filters" id="filters"></div>
  <div id="content">Chargement...</div>
</main>

<div class="modal-bg" id="loginModal">
  <div class="modal">
    <div class="row">
      <strong id="loginTitle">Connexion</strong>
      <button onclick="closeLoginModal()">Fermer</button>
    </div>

    <div class="banner info" id="loginBanner">
      <div class="banner-icon" id="bannerIcon">...</div>
      <div>
        <div class="banner-title" id="bannerTitle">Demarrage de la session...</div>
        <div class="banner-sub" id="bannerSub">Connexion securisee au serveur en cours.</div>
      </div>
    </div>

    <div class="steps hidden" id="loginSteps">
      <div class="step">
        <span class="step-num">1</span>
        <div class="step-body">
          <strong>Page de connexion ouverte dans votre navigateur</strong>
          <span>Choisissez votre compte puis validez l'autorisation.</span>
        </div>
        <button class="primary" onclick="openDetectedUrl()">Rouvrir la page</button>
      </div>
      <div class="step" id="stepCode">
        <span class="step-num">2</span>
        <div class="step-body">
          <strong>Cliquez sur « Copier » sur la page d'arrivee</strong>
          <span>Bridget detecte automatiquement le code copie et l'envoie tout seul.</span>
        </div>
      </div>
    </div>

    <div class="hidden" id="manualCodeBox">
      <label for="codeInput">Ou collez le code manuellement ici si besoin :</label>
      <div class="input-row">
        <input id="codeInput" placeholder="Coller le code puis Entree..." onkeydown="if(event.key==='Enter')sendCode()" />
        <button class="primary" onclick="sendCode()">Envoyer</button>
      </div>
    </div>

    <div class="row" style="justify-content:flex-end" id="loginFooter">
      <button id="cancelBtn" onclick="cancelSession()">Annuler</button>
      <button id="doneBtn" class="success hidden" onclick="closeLoginModal()">Terminer</button>
    </div>

    <details class="tech-log" id="logDetails">
      <summary>Journal technique</summary>
      <div class="log" id="loginLog"></div>
    </details>
  </div>
</div>

<div class="modal-bg" id="prepareModal">
  <div class="modal">
    <div class="row">
      <strong>Creer et connecter un nouveau compte isole</strong>
      <button onclick="closePrepareModal()">Fermer</button>
    </div>
    <div>
      <label>Connexion modele (machine + fournisseur)</label>
      <select id="prepSource"></select>
    </div>
    <div>
      <label>Nom du nouveau compte (minuscules, chiffres, tiret) : ex. perso2, pro-alt</label>
      <input id="prepName" placeholder="ex: google2" />
    </div>
    <div id="prepErr" class="meta" style="color:#f87171"></div>
    <div class="row" style="justify-content:flex-end">
      <button onclick="closePrepareModal()">Annuler</button>
      <button class="primary" onclick="submitPrepare()">Creer et lancer la connexion</button>
    </div>
  </div>
</div>

<script>
const TOKEN = "__BRIDGET_TOKEN__";
if (window.location.search.includes("token=")) {
  try { window.history.replaceState(null, "", "/"); } catch (e) {}
}
let fleet = null;
let selectedMachine = "ALL";
let currentSession = null;
let pollTimer = null;
let logOffset = 0;
let detectedUrl = null;
let authPopup = null;

function openManagedPopup(url) {
  try {
    const w = 560, h = 720;
    const left = Math.max(40, Math.round((window.screen.width - w) / 2));
    const top = Math.max(40, Math.round((window.screen.height - h) / 2));
    const feats = `popup=yes,width=${w},height=${h},left=${left},top=${top}`;
    if (authPopup && !authPopup.closed) {
      if (url) authPopup.location.href = url;
      authPopup.focus();
      return authPopup;
    }
    const pop = window.open(url || "about:blank", "bridget_oauth_popup", feats);
    if (pop && !url) {
      try {
        pop.document.title = "Connexion fournisseur - Bridget";
        pop.document.body.style.cssText = "margin:0;background:#0f1117;color:#edf2f7;font-family:-apple-system,sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;";
        pop.document.body.textContent = "Preparation de la page de connexion...";
      } catch (e) {}
    }
    authPopup = pop;
    return pop;
  } catch (e) {
    authPopup = null;
    return null;
  }
}

function closeAuthPopup() {
  try {
    if (authPopup && !authPopup.closed) authPopup.close();
  } catch (e) {}
  authPopup = null;
  try { window.focus(); } catch (e) {}
}

async function api(path, opts = {}) {
  const headers = Object.assign({ "X-Bridget-Token": TOKEN }, opts.headers || {});
  if (opts.body) headers["Content-Type"] = "application/json";
  const resp = await fetch(path, Object.assign({}, opts, { headers }));
  const data = await resp.json().catch(() => ({}));
  if (!resp.ok) throw new Error(data.error || ("HTTP " + resp.status));
  return data;
}

async function refreshStatus() {
  document.getElementById("subtitle").textContent = "Verification en cours sur les machines...";
  try {
    fleet = await api("/api/status");
    const okCount = fleet.connections.filter(c => c.auth_state === "authenticated").length;
    document.getElementById("subtitle").textContent =
      okCount + " / " + fleet.connections.length + " connexions authentifiees sur " + fleet.machines.length + " machines";
    renderFilters();
    renderCards();
  } catch (e) {
    document.getElementById("content").textContent = "Erreur : " + e.message;
  }
}

function renderFilters() {
  const el = document.getElementById("filters");
  const all = ["ALL", ...fleet.machines];
  el.innerHTML = all.map(m =>
    `<button class="chip ${selectedMachine===m?'active':''}" data-filter-machine="${esc(m)}">${m==='ALL'?'Toutes les machines':esc(m)}</button>`
  ).join("");
}

function stateLabel(s) {
  return { authenticated: "Connecte", unauthenticated: "A connecter", binary_missing: "Binaire absent", machine_unreachable: "Hors ligne" }[s] || s;
}

function renderCards() {
  const container = document.getElementById("content");
  const machines = selectedMachine === "ALL" ? fleet.machines : [selectedMachine];
  container.innerHTML = machines.map(m => {
    const rows = fleet.connections.filter(c => c.machine_id === m);
    return `<section class="machine-block">
      <h2 class="machine-title"><span>${esc(m)}</span><span class="meta">${rows.length} connexion(s)</span></h2>
      <div class="grid">${rows.map(c => `
        <div class="card">
          <div class="row">
            <span class="conn-name">${esc(c.connection_id)}</span>
            <span class="pill ${esc(c.auth_state)}">${esc(stateLabel(c.auth_state))}</span>
          </div>
          <div class="meta">Moteur : <b>${esc(c.engine)}</b> · ${c.enabled ? 'actif' : 'desactive'}${c.managed ? ' · compte isole' : ''}</div>
          <div class="meta">${esc(c.detail)}</div>
          <div class="actions">
            ${c.auth_state === 'authenticated'
              ? `<button class="warn" data-login-conn="${esc(c.connection_id)}" data-login-machine="${esc(c.machine_id)}" data-logout-first="1">Changer de compte</button>`
              : `<button class="primary" data-login-conn="${esc(c.connection_id)}" data-login-machine="${esc(c.machine_id)}" data-logout-first="0">Se connecter</button>`}
          </div>
        </div>`).join("")}
      </div>
    </section>`;
  }).join("");
}

document.getElementById("filters").addEventListener("click", (ev) => {
  const btn = ev.target.closest("button[data-filter-machine]");
  if (!btn) return;
  selectedMachine = btn.getAttribute("data-filter-machine") || "ALL";
  renderFilters();
  renderCards();
});

document.getElementById("content").addEventListener("click", (ev) => {
  const btn = ev.target.closest("button[data-login-conn]");
  if (!btn) return;
  const connId = btn.getAttribute("data-login-conn");
  const machineId = btn.getAttribute("data-login-machine");
  const logoutFirst = btn.getAttribute("data-logout-first") === "1";
  if (connId && machineId) startLogin(connId, machineId, logoutFirst, false);
});

function esc(s) {
  return String(s || "").replace(/[&<>"']/g, m => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[m]));
}

function setBanner(kind, icon, title, sub) {
  const b = document.getElementById("loginBanner");
  b.className = "banner " + kind;
  document.getElementById("bannerIcon").textContent = icon;
  document.getElementById("bannerTitle").textContent = title;
  document.getElementById("bannerSub").textContent = sub;
}

async function startLogin(connId, machineId, logoutFirst, enableOnSuccess) {
  if (pollTimer) clearInterval(pollTimer);
  closeAuthPopup();
  logOffset = 0; detectedUrl = null;
  const pop = openManagedPopup(null);
  const popupReady = !!(pop && !pop.closed);
  document.getElementById("loginTitle").textContent = (logoutFirst ? "Changer de compte : " : "Connexion : ") + connId + " (" + machineId + ")";
  document.getElementById("loginLog").textContent = "";
  document.getElementById("logDetails").open = false;
  document.getElementById("loginSteps").classList.add("hidden");
  document.getElementById("manualCodeBox").classList.add("hidden");
  document.getElementById("cancelBtn").classList.remove("hidden");
  document.getElementById("doneBtn").classList.add("hidden");
  document.getElementById("codeInput").value = "";
  setBanner("info", "⏳", "Demarrage sur " + machineId + "...", "Ouverture automatique de la page de connexion dans quelques secondes.");
  document.getElementById("loginModal").classList.add("open");
  try {
    const res = await api("/api/login/start", {
      method: "POST",
      body: JSON.stringify({
        connection_id: connId,
        machine_id: machineId,
        logout_first: logoutFirst,
        enable_on_success: enableOnSuccess,
        open_in_browser: !popupReady
      })
    });
    currentSession = res.session_id;
    pollTimer = setInterval(pollSession, 500);
    pollSession();
  } catch (e) {
    closeAuthPopup();
    setBanner("err", "✕", "Impossible de demarrer la connexion", e.message);
    document.getElementById("logDetails").open = true;
    document.getElementById("loginLog").textContent += "\nErreur : " + e.message;
    document.getElementById("cancelBtn").classList.add("hidden");
    document.getElementById("doneBtn").classList.remove("hidden");
  }
}

async function pollSession() {
  if (!currentSession) return;
  try {
    const res = await api(`/api/login/poll?session=${encodeURIComponent(currentSession)}&offset=${logOffset}`);
    if (res.chunk) {
      const logEl = document.getElementById("loginLog");
      logEl.textContent += res.chunk;
      logEl.scrollTop = logEl.scrollHeight;
      logOffset = res.next_offset;
    }
    if (res.auth_url && res.auth_url !== detectedUrl) {
      detectedUrl = res.auth_url;
      if (authPopup && !authPopup.closed) {
        try { authPopup.location.href = detectedUrl; authPopup.focus(); } catch (e) {}
      }
      document.getElementById("loginSteps").classList.remove("hidden");
      const needsCode = res.engine !== "codex";
      document.getElementById("stepCode").classList.toggle("hidden", !needsCode);
      document.getElementById("manualCodeBox").classList.toggle("hidden", !needsCode);
      if (needsCode) {
        setBanner("info", "🌐", "Validez dans la fenetre ouverte puis cliquez « Copier »", "Des que vous copiez le code, Bridget ferme la fenetre fournisseur et termine la connexion tout seul.");
      } else {
        setBanner("info", "🌐", "Validez la connexion dans la fenetre ouverte", "La redirection locale est automatique pour Codex et la fenetre se fermera toute seule.");
      }
    }
    if (res.running && res.code_auto_sent) {
      closeAuthPopup();
      setBanner("info", "⚡", "Code detecte et envoye automatiquement !", "Finalisation de la connexion sur le serveur...");
      document.getElementById("manualCodeBox").classList.add("hidden");
    } else if (res.running && res.code_sent) {
      closeAuthPopup();
      setBanner("info", "⏳", "Code envoye, validation en cours...", "Le serveur enregistre votre session.");
    }
    if (!res.running) {
      clearInterval(pollTimer);
      pollTimer = null;
      closeAuthPopup();
      document.getElementById("loginSteps").classList.add("hidden");
      document.getElementById("manualCodeBox").classList.add("hidden");
      document.getElementById("cancelBtn").classList.add("hidden");
      document.getElementById("doneBtn").classList.remove("hidden");
      if (res.exit_code === 0) {
        const who = res.logged_in_as ? ("Compte connecte : " + res.logged_in_as) : "Votre session est enregistree et prete a l'emploi.";
        const extra = res.auto_enabled ? " Le nouveau compte isole a ete active." : "";
        setBanner("ok", "✓", "Connexion reussie !", who + extra);
      } else {
        setBanner("err", "✕", "La connexion n'a pas abouti (code " + res.exit_code + ")", "Ouvrez le journal technique ci-dessous pour voir le detail.");
        document.getElementById("logDetails").open = true;
      }
      refreshStatus();
    }
  } catch (e) {}
}

function openDetectedUrl() { if (detectedUrl) openManagedPopup(detectedUrl); }

async function sendCode() {
  if (!currentSession) return;
  const input = document.getElementById("codeInput");
  const text = input.value;
  input.value = "";
  await api("/api/login/input", { method: "POST", body: JSON.stringify({ session_id: currentSession, text }) });
}

async function cancelSession() {
  closeAuthPopup();
  if (!currentSession) return;
  await api("/api/login/cancel", { method: "POST", body: JSON.stringify({ session_id: currentSession }) });
}

function closeLoginModal() {
  if (pollTimer) clearInterval(pollTimer);
  closeAuthPopup();
  document.getElementById("loginModal").classList.remove("open");
}

function openPrepareModal() {
  if (!fleet) return;
  const sel = document.getElementById("prepSource");
  const candidates = fleet.connections.filter(c => !c.managed && c.auth_state !== "machine_unreachable");
  sel.innerHTML = candidates.map(c =>
    `<option value="${esc(c.machine_id)}|${esc(c.connection_id)}">${esc(c.machine_id)} - ${esc(c.connection_id)} (${esc(c.engine)})</option>`
  ).join("");
  document.getElementById("prepName").value = "";
  document.getElementById("prepErr").textContent = "";
  document.getElementById("prepareModal").classList.add("open");
}

function closePrepareModal() { document.getElementById("prepareModal").classList.remove("open"); }

async function submitPrepare() {
  const val = document.getElementById("prepSource").value;
  const [machineId, fromConn] = val.split("|");
  const newId = document.getElementById("prepName").value.trim().toLowerCase();
  document.getElementById("prepErr").textContent = "Preparation du compte isole sur " + machineId + "...";
  try {
    const res = await api("/api/prepare", {
      method: "POST",
      body: JSON.stringify({ machine_id: machineId, from_connection_id: fromConn, new_id: newId })
    });
    closePrepareModal();
    await refreshStatus();
    startLogin(res.connection_id, res.machine_id, false, true);
  } catch (e) {
    document.getElementById("prepErr").textContent = e.message;
  }
}

refreshStatus();
</script>
</body>
</html>
"###;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec308_sh_quote_and_machine_validation() {
        assert_eq!(sh_quote("simple"), "'simple'");
        assert_eq!(sh_quote("a'b"), "'a'\"'\"'b'");
        assert!(valid_machine_id("cartae"));
        assert!(valid_machine_id("sysle2"));
        assert!(!valid_machine_id("-oProxyCommand=bad"));
        assert!(!valid_machine_id("host;rm -rf /"));
    }

    #[test]
    fn spec308_extract_auth_url_and_strip_ansi() {
        let raw = "\x1b[32mOpen this URL:\x1b[0m https://accounts.google.com/o/oauth2/v2/auth?client_id=123\r\n";
        assert_eq!(
            extract_auth_url(raw).as_deref(),
            Some("https://accounts.google.com/o/oauth2/v2/auth?client_id=123")
        );
        // Une URL de documentation générique ne doit pas être confondue avec un lien OAuth.
        assert_eq!(
            extract_auth_url("See docs at https://antigravity.google/docs/install.md"),
            None
        );
        assert_eq!(extract_auth_url("no link here"), None);
        assert_eq!(
            cmd_missing_condition("agy"),
            "! command -v 'agy' >/dev/null 2>&1"
        );
        assert_eq!(
            cmd_missing_condition("/usr/local/bin/agy"),
            "[ ! -x '/usr/local/bin/agy' ]"
        );
        assert!(looks_like_oauth_clipboard_code(
            "antigravity",
            "4/0AUn6Hd_0-XIlNMRKGGh-JY09C7sJGeSq0VEx0WH3C0Y5ET0V2uMCwRgQ2tQ5hRrap2puA"
        ));
        assert!(looks_like_oauth_clipboard_code(
            "antigravity",
            "https://antigravity.google/oauth-callback?state=abc&code=4/0AUn6Hd_0-1234567890"
        ));
        assert!(!looks_like_oauth_clipboard_code(
            "antigravity",
            "un texte ordinaire dans le presse-papiers"
        ));
        assert!(looks_like_oauth_clipboard_code(
            "claude",
            "abcdefghijklmnopqrstuvwx#0123456789abcdefghijkl"
        ));
        assert!(!looks_like_oauth_clipboard_code(
            "codex",
            "4/0AUn6Hd_0-123456789012345678901234"
        ));
        assert_eq!(
            extract_logged_in_account("Logged in as fullflare@gmail.com.\n").as_deref(),
            Some("fullflare@gmail.com")
        );
    }

    #[test]
    fn spec308_build_login_commands_isolate_profiles_and_clear_env() {
        let target = |engine: &str, cmd: &str, profile: Option<&str>| ConnectionTarget {
            connection_id: format!("{engine}-test"),
            machine_id: "cartae".into(),
            engine: engine.into(),
            agent_type: engine.into(),
            enabled: true,
            display_name: None,
            managed: profile.is_some(),
            command: cmd.into(),
            profile_dir: profile.map(str::to_string),
            registry_path: "/home/moi/.cache/bridget-core/agents.json".into(),
            is_local: false,
        };

        let agy_cmd = build_login_shell_command(
            &target(
                "antigravity",
                "/home/moi/.local/bin/agy",
                Some("/tmp/acc/antigravity"),
            ),
            false,
        );
        assert!(agy_cmd.contains("-u GEMINI_API_KEY"));
        assert!(agy_cmd.contains("--gemini-dir '/tmp/acc/antigravity'"));
        assert!(agy_cmd.contains("--app-data-dir antigravity-bridget --login"));

        let claude_cmd = build_login_shell_command(
            &target(
                "claude",
                "/home/moi/.local/bin/claude",
                Some("/tmp/acc/claude"),
            ),
            true,
        );
        assert!(claude_cmd.contains("CLAUDE_CONFIG_DIR='/tmp/acc/claude'"));
        assert!(claude_cmd.contains("ANTHROPIC_CONFIG_DIR='/tmp/acc/claude/anthropic'"));
        assert!(claude_cmd.contains("auth logout"));
        assert!(claude_cmd.contains("auth login"));

        let codex_cmd = build_login_shell_command(
            &target("codex", "/usr/local/bin/codex", Some("/tmp/acc/codex")),
            false,
        );
        assert!(codex_cmd.contains("CODEX_HOME='/tmp/acc/codex'"));
        assert!(codex_cmd.contains("-u OPENAI_API_KEY"));
        assert!(codex_cmd.contains("login"));
    }
}
