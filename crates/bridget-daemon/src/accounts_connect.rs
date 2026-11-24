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
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const PROBE_TIMEOUT_SECS: u64 = 8;
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
    #[serde(default)]
    pub registry_path: String,
    pub is_local: bool,
    pub auth_state: String,
    pub detail: String,
    #[serde(default)]
    pub account_label: String,
    #[serde(default)]
    pub covered_connections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FleetAuthStatus {
    pub local_machine: String,
    pub checked_at: i64,
    pub machines: Vec<String>,
    #[serde(default)]
    pub known_machines: Vec<String>,
    pub connections: Vec<ConnectionStatus>,
}

pub(crate) fn run(args: &[String]) {
    let sub = args.first().map(String::as_str).unwrap_or("");
    let rest = if args.is_empty() { &[][..] } else { &args[1..] };
    let result = match sub {
        "status" => cmd_status(rest),
        "login" => cmd_login(rest),
        "ui" => crate::console_ui::cmd_ui(rest),
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

pub(crate) fn local_machine_name() -> String {
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

fn read_remote_home_file(machine: &str, rel_path: &str) -> Option<(PathBuf, String)> {
    if !valid_machine_id(machine) {
        return None;
    }
    let rel = rel_path.trim_start_matches('/');
    let cmd = format!(
        "p=\"$HOME/\"{}; [ -r \"$p\" ] && printf '%s\\n' \"$p\" && cat \"$p\" 2>/dev/null",
        sh_quote(rel)
    );
    let output = Command::new("ssh")
        .args([
            "-o",
            "BatchMode=yes",
            "-o",
            &format!("ConnectTimeout={DEFAULT_SSH_CONNECT_TIMEOUT}"),
            machine,
            &cmd,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() || output.stdout.is_empty() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let (first_line, rest) = text.split_once('\n')?;
    if first_line.is_empty() || rest.is_empty() {
        return None;
    }
    Some((PathBuf::from(first_line.trim()), rest.to_string()))
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
        if !matches!(
            conn.engine.as_str(),
            "claude" | "codex" | "antigravity" | "cursor"
        ) {
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
/// joignables de la fédération (en fusionnant le registre maître et les registres worker),
/// ainsi que la liste complète des machines connues (y compris sans connexion ou hors ligne).
pub(crate) fn load_fleet_connections_with_machines(
    registry_override: Option<&Path>,
) -> Result<(String, Vec<String>, Vec<ConnectionTarget>), String> {
    let local_machine = local_machine_name();
    let local_path = match registry_override {
        Some(p) => p.to_path_buf(),
        None => crate::registry::config_path()?,
    };
    let local_raw = crate::registry::read_private_registry(&local_path)
        .map_err(|e| format!("registre local illisible ({}): {e}", local_path.display()))?;

    let mut known_machines: BTreeSet<String> = BTreeSet::new();
    known_machines.insert(local_machine.clone());
    if let Ok(reg) = AgentRegistry::from_json(&local_raw, &local_path) {
        for name in reg.machines().keys() {
            if name != "local" && valid_machine_id(name) {
                known_machines.insert(name.clone());
            }
        }
    }

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
        known_machines.insert(target.machine_id.clone());
        by_key.insert(
            (target.machine_id.clone(), target.connection_id.clone()),
            target,
        );
    }

    // Si un registre explicite est fourni (--registry), ne pas interroger d'autres hôtes en SSH.
    if registry_override.is_some() {
        return Ok((
            local_machine,
            known_machines.into_iter().collect(),
            by_key.into_values().collect(),
        ));
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
        known_machines.insert("cartae".to_string());
    }

    // Si on n'est pas sur cartae, lire la liste complète des machines depuis cartae.
    let mut master_connections = Vec::new();
    if local_machine != "cartae"
        && let Some((cartae_path, cartae_raw)) =
            read_remote_home_file("cartae", ".cache/bridget-core/agents.json")
    {
        if let Ok(reg) = AgentRegistry::from_json(&cartae_raw, &cartae_path) {
            for name in reg.machines().keys() {
                if name != "local" && valid_machine_id(name) {
                    known_machines.insert(name.clone());
                }
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
        known_machines.insert(target.machine_id.clone());
        by_key
            .entry((target.machine_id.clone(), target.connection_id.clone()))
            .or_insert(target);
    }

    // Lire en parallèle le registre worker privé de chaque machine distante (Linux ou macOS)
    // pour récupérer les chemins de binaires locaux réels et les comptes isolés préparés.
    let handles: Vec<_> = remote_machines
        .into_iter()
        .map(|machine| {
            let local_m = local_machine.clone();
            std::thread::spawn(move || {
                let candidates = if machine == "cartae" {
                    vec![
                        ".cache/bridget-core/agents.json",
                        ".config/bridget/agents.json",
                    ]
                } else {
                    vec![
                        ".config/bridget/agents.json",
                        ".cache/bridget-core/agents.json",
                    ]
                };
                for rel_path in candidates {
                    if let Some((p, raw)) = read_remote_home_file(&machine, rel_path) {
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
                known_machines.insert(target.machine_id.clone());
                by_key.insert(
                    (target.machine_id.clone(), target.connection_id.clone()),
                    target,
                );
            }
        }
    }

    Ok((
        local_machine,
        known_machines.into_iter().collect(),
        by_key.into_values().collect(),
    ))
}

/// Charge toutes les connexions connues sur la machine locale et sur les serveurs
/// joignables de la fédération (en fusionnant le registre maître et les registres worker).
pub(crate) fn load_fleet_connections(
    registry_override: Option<&Path>,
) -> Result<(String, Vec<ConnectionTarget>), String> {
    let (local_machine, _, targets) = load_fleet_connections_with_machines(registry_override)?;
    Ok((local_machine, targets))
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
            "cursor" => {
                script.push_str(&format!(
                    "if {missing}; then printf '%s\\tbinary_missing\\tcursor-agent absent\\n' {id}; else\n\
                     out=$({env_prefix} {cmd} status 2>&1 || true)\n\
                     if printf '%s' \"$out\" | grep -qi 'logged in'; then\n\
                       line=$(printf '%s' \"$out\" | grep -i 'logged in' | head -n 1 | tr '\\t\\r\\n' '   ')\n\
                       printf '%s\\tauthenticated\\t%s\\n' {id} \"$line\"\n\
                     else\n\
                       printf '%s\\tunauthenticated\\tnon connecte\\n' {id}\n\
                     fi; fi\n"
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

fn account_group_key(t: &ConnectionTarget) -> (String, String, String) {
    let base = Path::new(&t.command)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&t.command)
        .to_string();
    let profile = t.profile_dir.clone().unwrap_or_default();
    (t.engine.clone(), base, profile)
}

fn friendly_account_label(t: &ConnectionTarget) -> String {
    if t.managed {
        let name = t
            .connection_id
            .strip_prefix("account-")
            .unwrap_or(&t.connection_id);
        let eng = match t.engine.as_str() {
            "claude" => "Claude",
            "codex" => "Codex",
            "antigravity" => "Antigravity",
            "cursor" => "Cursor",
            other => other,
        };
        return format!("{eng} ({name})");
    }
    let base = Path::new(&t.command)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&t.command);
    match (t.engine.as_str(), base) {
        ("claude", "gclaude") => "GLM (gclaude)".to_string(),
        ("claude", _) => "Claude".to_string(),
        ("codex", "codex-pro") => "Codex Pro".to_string(),
        ("codex", _) => "Codex".to_string(),
        ("antigravity", _) => "Antigravity".to_string(),
        ("cursor", _) => "Cursor".to_string(),
        (other, _) => other.to_string(),
    }
}

fn primary_connection_rank(t: &ConnectionTarget) -> (u8, usize, String) {
    let penalty = if t.connection_id.ends_with("-haiku213")
        || t.connection_id.ends_with("-terra213")
        || t.connection_id.ends_with("-models62")
    {
        2
    } else if !t.enabled {
        3
    } else if t.connection_id.ends_with("-native") {
        1
    } else {
        0
    };
    (penalty, t.connection_id.len(), t.connection_id.clone())
}

/// Regroupe les connexions de routage d'une machine par vrai compte fournisseur
/// `(engine, binaire, dossier_profil)` afin de n'afficher et ne sonder chaque compte qu'une fois.
pub(crate) fn group_machine_accounts(
    targets: Vec<ConnectionTarget>,
) -> Vec<(ConnectionTarget, Vec<String>, bool)> {
    let mut by_account: BTreeMap<(String, String, String), Vec<ConnectionTarget>> = BTreeMap::new();
    for t in targets {
        by_account.entry(account_group_key(&t)).or_default().push(t);
    }
    let mut grouped = Vec::with_capacity(by_account.len());
    for mut items in by_account.into_values() {
        items.sort_by_key(primary_connection_rank);
        let any_enabled = items.iter().any(|t| t.enabled);
        let mut covered: Vec<String> = items.iter().map(|t| t.connection_id.clone()).collect();
        covered.sort();
        let primary = items.into_iter().next().unwrap();
        grouped.push((primary, covered, any_enabled));
    }
    grouped
}

fn probe_machine(machine: String, targets: Vec<ConnectionTarget>) -> Vec<ConnectionStatus> {
    let grouped = group_machine_accounts(targets);
    let primaries: Vec<ConnectionTarget> = grouped.iter().map(|(t, _, _)| t.clone()).collect();
    let is_local = primaries.first().is_some_and(|t| t.is_local);
    let script = build_probe_script(&primaries);
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

    grouped
        .into_iter()
        .map(|(t, covered, any_enabled)| {
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
            let account_label = friendly_account_label(&t);
            ConnectionStatus {
                connection_id: t.connection_id,
                machine_id: t.machine_id,
                engine: t.engine,
                agent_type: t.agent_type,
                enabled: any_enabled,
                display_name: t.display_name,
                managed: t.managed,
                profile_dir: t.profile_dir,
                registry_path: t.registry_path,
                is_local: t.is_local,
                auth_state,
                detail,
                account_label,
                covered_connections: covered,
            }
        })
        .collect()
}

pub(crate) fn collect_fleet_status(
    registry_override: Option<&Path>,
) -> Result<FleetAuthStatus, String> {
    let (local_machine, known_machines, targets) =
        load_fleet_connections_with_machines(registry_override)?;
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
        known_machines,
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
        let label = if row.account_label.is_empty() {
            row.connection_id.as_str()
        } else {
            row.account_label.as_str()
        };
        let extra = if row.covered_connections.len() > 1 {
            format!(" [couvre: {}]", row.covered_connections.join(", "))
        } else {
            String::new()
        };
        println!(
            "  [{:<11}] {:<14} {:<18} ({:<24}) : {}{}",
            badge, row.machine_id, label, row.connection_id, row.detail, extra
        );
    }
    Ok(())
}

/// Construit la commande shell de connexion (et optionnellement déconnexion préalable)
/// pour un moteur et un profil donné.
pub(crate) fn build_login_shell_command(target: &ConnectionTarget, logout_first: bool) -> String {
    let path_setup =
        "export PATH=\"$HOME/.local/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:$PATH\"";
    let cmd = sh_quote(&target.command);
    let profile = target.profile_dir.as_deref().unwrap_or("");
    let env_prefix = env_prefix_for(&target.engine, profile);
    let body = match target.engine.as_str() {
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
        "cursor" => {
            if logout_first {
                format!(
                    "{env_prefix} {cmd} logout >/dev/null 2>&1 || true; {env_prefix} {cmd} login"
                )
            } else {
                format!("{env_prefix} {cmd} login")
            }
        }
        _ => format!(
            "printf 'Moteur non pris en charge : %s\\n' {} && exit 1",
            sh_quote(&target.engine)
        ),
    };
    format!("{path_setup}; {body}")
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

pub(crate) fn build_ssh_login_args(target: &ConnectionTarget, login_cmd: String) -> Vec<String> {
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

pub(crate) fn open_browser(url: &str) {
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

pub(crate) fn find_target(
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

/// Reconnaît strictement un code ou une URL de retour OAuth pour Antigravity ou Claude.
#[cfg_attr(not(any(target_os = "macos", test)), allow(dead_code))]
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

/// Extrait l'identifiant du compte affiché par le CLI après connexion réussie.
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

    #[test]
    fn spec309_group_machine_accounts_deduplicates_shared_profiles() {
        let mk = |cid: &str, engine: &str, cmd: &str, profile: Option<&str>| ConnectionTarget {
            connection_id: cid.into(),
            machine_id: "sysle2".into(),
            engine: engine.into(),
            agent_type: cid.into(),
            enabled: true,
            display_name: None,
            managed: cid.starts_with("account-"),
            command: cmd.into(),
            profile_dir: profile.map(str::to_string),
            registry_path: "/home/moi/.config/bridget/agents.json".into(),
            is_local: false,
        };
        let grouped = group_machine_accounts(vec![
            mk(
                "claude-sysle2-haiku213",
                "claude",
                "/home/moi/.local/bin/claude",
                None,
            ),
            mk(
                "claude-sysle2-native",
                "claude",
                "/home/moi/.local/bin/claude",
                None,
            ),
            mk(
                "codex-sysle2-terra213",
                "codex",
                "/home/moi/.local/bin/codex",
                None,
            ),
            mk("codex-sysle2", "codex", "/home/moi/.local/bin/codex", None),
            mk(
                "codex-pro",
                "codex",
                "/Users/moi/.local/bin/codex-pro",
                None,
            ),
            mk(
                "antigravity-sysle2",
                "antigravity",
                "/home/moi/.local/bin/agy",
                None,
            ),
        ]);
        assert_eq!(grouped.len(), 4);
        let claude = grouped
            .iter()
            .find(|(t, _, _)| t.engine == "claude")
            .unwrap();
        assert_eq!(claude.0.connection_id, "claude-sysle2-native");
        assert_eq!(
            claude.1,
            vec!["claude-sysle2-haiku213", "claude-sysle2-native"]
        );
        let codex = grouped
            .iter()
            .find(|(t, _, _)| t.connection_id == "codex-sysle2")
            .unwrap();
        assert_eq!(codex.1, vec!["codex-sysle2", "codex-sysle2-terra213"]);
    }
}
