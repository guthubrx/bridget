//! Gestion `automanaged` (active par défaut) :
//! - Auto-référencement additif des nouveaux modèles annoncés par les fournisseurs
//!   (Codex et Antigravity) dans les registres `agents.json` (verrouillés et validés).
//! - Mise à jour périodique des CLI fournisseurs (`claude`, `codex`, `agy`) via leurs
//!   commandes officielles, sans interrompre les missions en cours.
//!
//! Configuration privée : `~/.cache/bridget-core/automanaged.json` (absent => actif par défaut).
//! État privé           : `~/.cache/bridget-core/automanaged-state.json`.

use crate::cli_table::table_println as println;
use crate::registry::AgentRegistry;
use bridget_transport::provider_catalogue::catalogue_token;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

pub(crate) const DEFAULT_INTERVAL_SECS: u64 = 21600; // 6 heures
const CLI_UPDATE_TIMEOUT_SECS: u64 = 180;
const MODEL_PROBE_TIMEOUT_SECS: u64 = 12;

fn default_true() -> bool {
    true
}
fn default_interval() -> u64 {
    DEFAULT_INTERVAL_SECS
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AutomanagedConfig {
    #[serde(default = "default_true")]
    pub cli_updates: bool,
    #[serde(default = "default_true")]
    pub model_sync: bool,
    #[serde(default = "default_interval")]
    pub interval_secs: u64,
}

impl Default for AutomanagedConfig {
    fn default() -> Self {
        Self {
            cli_updates: true,
            model_sync: true,
            interval_secs: DEFAULT_INTERVAL_SECS,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CliUpdateOutcome {
    pub engine: String,
    pub command: String,
    pub method: String,
    pub version_before: Option<String>,
    pub version_after: Option<String>,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RegistryModelSyncOutcome {
    pub registry_path: String,
    pub added: BTreeMap<String, Vec<String>>,
    pub total_added: usize,
    pub status: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct AutomanagedState {
    pub last_run_at: Option<i64>,
    pub last_cli_check_at: Option<i64>,
    pub last_model_sync_at: Option<i64>,
    #[serde(default)]
    pub cli_results: Vec<CliUpdateOutcome>,
    #[serde(default)]
    pub model_results: Vec<RegistryModelSyncOutcome>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct DiscoveredModel {
    pub model: String,
    pub supported_efforts: Vec<String>,
    pub default_effort: Option<String>,
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn state_dir() -> Result<PathBuf, String> {
    if let Ok(home) = std::env::var("BRIDGET_HOME")
        && !home.trim().is_empty()
    {
        return Ok(PathBuf::from(home));
    }
    let home = std::env::var("HOME").map_err(|_| "variable HOME absente".to_string())?;
    Ok(PathBuf::from(home).join(".cache").join("bridget-core"))
}

pub(crate) fn config_path() -> Result<PathBuf, String> {
    Ok(state_dir()?.join("automanaged.json"))
}

pub(crate) fn state_path() -> Result<PathBuf, String> {
    Ok(state_dir()?.join("automanaged-state.json"))
}

pub(crate) fn load_config() -> AutomanagedConfig {
    let Ok(path) = config_path() else {
        return AutomanagedConfig::default();
    };
    let Ok(raw) = fs::read_to_string(&path) else {
        return AutomanagedConfig::default();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

pub(crate) fn save_config(cfg: &AutomanagedConfig) -> Result<(), String> {
    let dir = state_dir()?;
    crate::environment::ensure_private_directory(&dir)?;
    let path = dir.join("automanaged.json");
    let raw = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    write_atomic_private(&path, format!("{raw}\n").as_bytes())
}

pub(crate) fn load_state() -> AutomanagedState {
    let Ok(path) = state_path() else {
        return AutomanagedState::default();
    };
    let Ok(raw) = fs::read_to_string(&path) else {
        return AutomanagedState::default();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

fn save_state(st: &AutomanagedState) -> Result<(), String> {
    let dir = state_dir()?;
    crate::environment::ensure_private_directory(&dir)?;
    let path = dir.join("automanaged-state.json");
    let raw = serde_json::to_string_pretty(st).map_err(|e| e.to_string())?;
    write_atomic_private(&path, format!("{raw}\n").as_bytes())
}

fn write_atomic_private(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or("dossier parent manquant")?;
    let tmp = parent.join(format!(
        ".{}.tmp-{}",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("file"),
        uuid::Uuid::new_v4().simple()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&tmp)
        .map_err(|e| e.to_string())?;
    file.write_all(bytes).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        e.to_string()
    })?;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    Ok(())
}

fn wait_bounded(mut child: Child, timeout_secs: u64) -> Result<std::process::Output, String> {
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    loop {
        if child.try_wait().map_err(|e| e.to_string())?.is_some() {
            return child.wait_with_output().map_err(|e| e.to_string());
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err("delai depasse".into());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn is_legacy_alias_connection(connection_id: &str) -> bool {
    connection_id.ends_with("-haiku213")
        || connection_id.ends_with("-terra213")
        || connection_id.ends_with("-models62")
}

/// Fusionne de manière strictement additive les nouveaux modèles annoncés dans un objet
/// JSON `agents.json` (sans jamais retirer ni modifier un modèle déjà déclaré).
/// Retourne les modèles ajoutés par `connection_id`.
pub(crate) fn merge_discovered_models_into_registry_value(
    value: &mut Value,
    discovered_by_connection: &BTreeMap<String, Vec<DiscoveredModel>>,
) -> BTreeMap<String, Vec<String>> {
    let mut added_by_conn: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let Some(connections) = value
        .pointer("/routing_policy/connections")
        .and_then(Value::as_array)
        .cloned()
    else {
        return added_by_conn;
    };

    let mut any_change = false;
    for conn in &connections {
        let Some(cid) = conn.get("connection_id").and_then(Value::as_str) else {
            continue;
        };
        let Some(engine) = conn.get("engine").and_then(Value::as_str) else {
            continue;
        };
        let Some(agent_type) = conn.get("agent_type").and_then(Value::as_str) else {
            continue;
        };
        if !conn
            .get("enabled")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            continue;
        }
        if is_legacy_alias_connection(cid) {
            continue;
        }
        let Some(discovered) = discovered_by_connection.get(cid) else {
            continue;
        };
        if discovered.is_empty() {
            continue;
        }

        // Vérifier que l'agent existe dans `agents`.
        let has_agent = value
            .get("agents")
            .and_then(Value::as_object)
            .is_some_and(|m| m.contains_key(agent_type));
        if !has_agent {
            continue;
        }

        let mut added_here = Vec::new();
        for item in discovered {
            if !catalogue_token(&item.model) {
                continue;
            }
            if !item.supported_efforts.iter().all(|e| catalogue_token(e)) {
                continue;
            }
            let valid_default = item
                .default_effort
                .as_deref()
                .filter(|d| item.supported_efforts.iter().any(|e| e == d))
                .map(str::to_string);

            // 1. Ajouter dans `agents.<agent_type>.capabilities.models` si absent.
            let mut touched = false;
            if let Some(cap_models) = value
                .pointer_mut(&format!("/agents/{agent_type}/capabilities/models"))
                .and_then(Value::as_object_mut)
                && !cap_models.contains_key(&item.model)
            {
                cap_models.insert(
                    item.model.clone(),
                    json!({ "efforts": item.supported_efforts }),
                );
                touched = true;
            }

            // 2. Ajouter dans `routing_policy.model_catalog.<cid>` si absent.
            if let Some(catalog) = value
                .pointer_mut(&format!("/routing_policy/model_catalog/{cid}"))
                .and_then(Value::as_array_mut)
            {
                let already = catalog
                    .iter()
                    .any(|m| m.get("model").and_then(Value::as_str) == Some(item.model.as_str()));
                if !already {
                    let provenance = if engine == "codex" {
                        "codex_model_list"
                    } else {
                        "declared"
                    };
                    catalog.push(json!({
                        "model": item.model,
                        "supported_efforts": item.supported_efforts,
                        "default_effort": valid_default,
                        "provenance": provenance,
                        "observed_at": now_secs()
                    }));
                    touched = true;
                }
            }

            if touched {
                added_here.push(item.model.clone());
                any_change = true;
            }
        }
        if !added_here.is_empty() {
            added_by_conn.insert(cid.to_string(), added_here);
        }
    }

    if any_change
        && let Some(rev) = value
            .pointer("/routing_policy/revision")
            .and_then(Value::as_u64)
        && let Some(next_rev) = rev.checked_add(1)
    {
        value["routing_policy"]["revision"] = json!(next_rev);
    }

    added_by_conn
}

/// Collecte les modèles actuellement annoncés par le daemon (`delegate --catalogue --observed --json`)
/// et par les CLI Antigravity locaux en repli si le daemon n'a pas encore publié la connexion.
fn collect_discovered_models(
    local_registries: &[PathBuf],
) -> BTreeMap<String, Vec<DiscoveredModel>> {
    let mut by_conn: BTreeMap<String, Vec<DiscoveredModel>> = BTreeMap::new();

    // 1. Source principale : catalogues live déjà observés par les workers (Codex + Antigravity).
    if let Ok(exe) = std::env::current_exe()
        && let Ok(child) = Command::new(exe)
            .args(["delegate", "--catalogue", "--observed", "--json"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        && let Ok(out) = wait_bounded(child, MODEL_PROBE_TIMEOUT_SECS)
        && out.status.success()
        && let Ok(parsed) = serde_json::from_slice::<Value>(&out.stdout)
    {
        let cats = parsed
            .get("connection_catalogues")
            .or_else(|| parsed.pointer("/fleet/connection_catalogues"))
            .and_then(Value::as_array);
        if let Some(cats) = cats {
            for entry in cats {
                if entry.get("status").and_then(Value::as_str) != Some("reported_models") {
                    continue;
                }
                let Some(cid) = entry.get("connection_id").and_then(Value::as_str) else {
                    continue;
                };
                let Some(models) = entry.get("models").and_then(Value::as_array) else {
                    continue;
                };
                let mut list = Vec::new();
                for m in models {
                    let Some(name) = m.get("model").and_then(Value::as_str) else {
                        continue;
                    };
                    let efforts: Vec<String> = m
                        .get("supported_efforts")
                        .and_then(Value::as_array)
                        .map(|arr| {
                            arr.iter()
                                .filter_map(Value::as_str)
                                .map(str::to_string)
                                .collect()
                        })
                        .unwrap_or_default();
                    let default_effort = m
                        .get("default_effort")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    list.push(DiscoveredModel {
                        model: name.to_string(),
                        supported_efforts: efforts,
                        default_effort,
                    });
                }
                if !list.is_empty() {
                    by_conn.insert(cid.to_string(), list);
                }
            }
        }
    }

    // 2. Complément direct local pour Antigravity (`agy models`) et Cursor (`cursor-agent --list-models`)
    // si non encore présents dans le cache.
    let mut probed_agy_cmds: BTreeMap<String, Vec<DiscoveredModel>> = BTreeMap::new();
    let mut probed_cursor_cmds: BTreeMap<String, Vec<DiscoveredModel>> = BTreeMap::new();
    for reg_path in local_registries {
        let Ok(raw) = crate::registry::read_private_registry(reg_path) else {
            continue;
        };
        let Ok(registry) = AgentRegistry::from_json(&raw, reg_path) else {
            continue;
        };
        let Some(policy) = registry.routing_policy() else {
            continue;
        };
        for conn in &policy.connections {
            if !conn.enabled || by_conn.contains_key(&conn.connection_id) {
                continue;
            }
            let Ok(def) = registry.get(&conn.agent_type) else {
                continue;
            };
            if conn.engine == "antigravity" {
                if def.antigravity_home.is_some() {
                    continue;
                }
                let models = probed_agy_cmds
                    .entry(def.command.clone())
                    .or_insert_with(|| probe_local_agy_models(&def.command));
                if !models.is_empty() {
                    by_conn.insert(conn.connection_id.clone(), models.clone());
                }
            } else if conn.engine == "cursor" {
                let models = probed_cursor_cmds
                    .entry(def.command.clone())
                    .or_insert_with(|| probe_local_cursor_models(&def.command));
                if !models.is_empty() {
                    by_conn.insert(conn.connection_id.clone(), models.clone());
                }
            }
        }
    }

    by_conn
}

fn probe_local_cursor_models(command: &str) -> Vec<DiscoveredModel> {
    let mut cmd = Command::new(command);
    cmd.arg("--list-models")
        .env_remove("CURSOR_API_KEY")
        .env_remove("OPENAI_API_KEY")
        .env_remove("ANTHROPIC_API_KEY")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let Ok(child) = cmd.spawn() else {
        return Vec::new();
    };
    let Ok(out) = wait_bounded(child, MODEL_PROBE_TIMEOUT_SECS) else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(&out.stdout);
    crate::provider_catalogue::parse_cursor_models(&text)
        .into_iter()
        .map(|m| DiscoveredModel {
            model: m.model,
            supported_efforts: m.supported_efforts,
            default_effort: m.default_effort,
        })
        .collect()
}

fn probe_local_agy_models(command: &str) -> Vec<DiscoveredModel> {
    let mut cmd = Command::new(command);
    cmd.arg("models")
        .env_remove("GEMINI_API_KEY")
        .env_remove("GOOGLE_API_KEY")
        .env_remove("AGY_ADC_AUTH")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let Ok(child) = cmd.spawn() else {
        return Vec::new();
    };
    let Ok(out) = wait_bounded(child, MODEL_PROBE_TIMEOUT_SECS) else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(&out.stdout);
    crate::provider_catalogue::parse_antigravity_models(&text)
        .into_iter()
        .map(|m| DiscoveredModel {
            model: m.model,
            supported_efforts: m.supported_efforts,
            default_effort: m.default_effort,
        })
        .collect()
}

fn discover_local_registries(explicit: Option<&Path>) -> Vec<PathBuf> {
    if let Some(p) = explicit {
        return vec![p.to_path_buf()];
    }
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    let mut add = |p: PathBuf| {
        if p.exists() && seen.insert(p.clone()) {
            out.push(p);
        }
    };
    if let Ok(primary) = crate::registry::config_path() {
        add(primary);
    }
    if let Ok(home) = std::env::var("HOME") {
        let h = PathBuf::from(home);
        add(h.join(".cache").join("bridget-core").join("agents.json"));
        add(h
            .join(".cache")
            .join("bridget-core")
            .join("worker-monordinateur-agents.json"));
        add(h.join(".config").join("bridget").join("agents.json"));
    }
    out
}

fn sync_one_registry(
    reg_path: &Path,
    discovered: &BTreeMap<String, Vec<DiscoveredModel>>,
    dry_run: bool,
) -> RegistryModelSyncOutcome {
    let path_str = reg_path.to_string_lossy().into_owned();
    let Some(parent) = reg_path.parent() else {
        return RegistryModelSyncOutcome {
            registry_path: path_str,
            added: BTreeMap::new(),
            total_added: 0,
            status: "parent_missing".into(),
        };
    };
    let lock_path = parent.join("project-preparation.lock");
    let lock_file = match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(&lock_path)
    {
        Ok(f) => f,
        Err(e) => {
            return RegistryModelSyncOutcome {
                registry_path: path_str,
                added: BTreeMap::new(),
                total_added: 0,
                status: format!("lock_error: {e}"),
            };
        }
    };
    // SAFETY: descripteur ouvert détenu pendant toute la mise à jour atomique.
    if unsafe { libc::flock(lock_file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        return RegistryModelSyncOutcome {
            registry_path: path_str,
            added: BTreeMap::new(),
            total_added: 0,
            status: "registry_busy".into(),
        };
    }

    let raw = match crate::registry::read_private_registry(reg_path) {
        Ok(r) => r,
        Err(e) => {
            return RegistryModelSyncOutcome {
                registry_path: path_str,
                added: BTreeMap::new(),
                total_added: 0,
                status: format!("read_error: {e}"),
            };
        }
    };
    let mut value: Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            return RegistryModelSyncOutcome {
                registry_path: path_str,
                added: BTreeMap::new(),
                total_added: 0,
                status: format!("json_error: {e}"),
            };
        }
    };
    let added = merge_discovered_models_into_registry_value(&mut value, discovered);
    let total_added: usize = added.values().map(Vec::len).sum();
    if total_added == 0 {
        return RegistryModelSyncOutcome {
            registry_path: path_str,
            added,
            total_added: 0,
            status: "up_to_date".into(),
        };
    }

    let serialized = match serde_json::to_string_pretty(&value) {
        Ok(s) => format!("{s}\n"),
        Err(e) => {
            return RegistryModelSyncOutcome {
                registry_path: path_str,
                added: BTreeMap::new(),
                total_added: 0,
                status: format!("serialize_error: {e}"),
            };
        }
    };

    // Valider strictement avant toute écriture disque.
    if let Err(e) = AgentRegistry::from_json(&serialized, reg_path) {
        return RegistryModelSyncOutcome {
            registry_path: path_str,
            added: BTreeMap::new(),
            total_added: 0,
            status: format!("validation_refused: {e}"),
        };
    }

    if dry_run {
        return RegistryModelSyncOutcome {
            registry_path: path_str,
            added,
            total_added,
            status: "dry_run".into(),
        };
    }

    let backup = parent.join(format!(
        "{}.bak-automanaged",
        reg_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("agents.json")
    ));
    let _ = write_atomic_private(&backup, raw.as_bytes());
    match write_atomic_private(reg_path, serialized.as_bytes()) {
        Ok(()) => RegistryModelSyncOutcome {
            registry_path: path_str,
            added,
            total_added,
            status: "updated".into(),
        },
        Err(e) => RegistryModelSyncOutcome {
            registry_path: path_str,
            added: BTreeMap::new(),
            total_added: 0,
            status: format!("write_error: {e}"),
        },
    }
}

fn read_cli_version(cmd: &str) -> Option<String> {
    let child = Command::new(cmd)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;
    let out = wait_bounded(child, 8).ok()?;
    let text = format!(
        "{} {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    text.split_whitespace()
        .find(|tok| {
            let clean = tok.trim_start_matches('v');
            clean.split('.').count() >= 3
                && clean
                    .split('.')
                    .take(3)
                    .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
        })
        .map(|t| t.trim_start_matches('v').to_string())
}

fn is_homebrew_path(cmd: &str) -> bool {
    let p = Path::new(cmd);
    let resolved = fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let s = resolved.to_string_lossy();
    s.contains("/opt/homebrew/") || s.contains("/usr/local/Caskroom/")
}

/// Met à jour les CLI fournisseurs locaux (`claude`, `codex`, `agy`) via leurs
/// commandes officielles sans interrompre les missions en cours.
fn run_local_cli_updates(registries: &[PathBuf], dry_run: bool) -> Vec<CliUpdateOutcome> {
    // Repérer un binaire officiel par moteur parmi les registres locaux.
    let mut targets: BTreeMap<String, String> = BTreeMap::new();
    for reg_path in registries {
        let Ok(raw) = crate::registry::read_private_registry(reg_path) else {
            continue;
        };
        let Ok(reg) = AgentRegistry::from_json(&raw, reg_path) else {
            continue;
        };
        let Some(policy) = reg.routing_policy() else {
            continue;
        };
        for conn in &policy.connections {
            if !conn.enabled
                || !matches!(
                    conn.engine.as_str(),
                    "claude" | "codex" | "antigravity" | "cursor"
                )
            {
                continue;
            }
            let Ok(def) = reg.get(&conn.agent_type) else {
                continue;
            };
            // Ignorer les wrappers (gclaude, codex-pro) : la mise à jour du binaire principal suffit.
            let base = Path::new(&def.command)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if matches!(base, "gclaude" | "codex-pro") {
                continue;
            }
            if Path::new(&def.command).exists() {
                targets
                    .entry(conn.engine.clone())
                    .or_insert_with(|| def.command.clone());
            }
        }
    }

    let mut outcomes = Vec::new();
    for (engine, command) in targets {
        let before = read_cli_version(&command);
        let (method, prog, args): (&str, String, Vec<String>) = match engine.as_str() {
            "claude" => ("claude update", command.clone(), vec!["update".into()]),
            "codex" => {
                if cfg!(target_os = "macos") && is_homebrew_path(&command) {
                    (
                        "brew upgrade --cask codex",
                        "/opt/homebrew/bin/brew".into(),
                        vec!["upgrade".into(), "--cask".into(), "codex".into()],
                    )
                } else {
                    ("codex update", command.clone(), vec!["update".into()])
                }
            }
            "antigravity" => ("agy update", command.clone(), vec!["update".into()]),
            "cursor" => (
                "cursor-agent update",
                command.clone(),
                vec!["update".into()],
            ),
            _ => continue,
        };
        if dry_run {
            outcomes.push(CliUpdateOutcome {
                engine,
                command,
                method: method.into(),
                version_before: before.clone(),
                version_after: before,
                status: "dry_run".into(),
                detail: "simulation uniquement".into(),
            });
            continue;
        }
        let result = Command::new(&prog)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())
            .and_then(|c| wait_bounded(c, CLI_UPDATE_TIMEOUT_SECS));
        let after = read_cli_version(&command);
        match result {
            Ok(out) if out.status.success() => {
                let status = if before != after && after.is_some() {
                    "updated"
                } else {
                    "up_to_date"
                };
                outcomes.push(CliUpdateOutcome {
                    engine,
                    command,
                    method: method.into(),
                    version_before: before,
                    version_after: after,
                    status: status.into(),
                    detail: "commande officielle terminee".into(),
                });
            }
            Ok(out) => {
                let msg = String::from_utf8_lossy(&out.stderr)
                    .lines()
                    .next()
                    .unwrap_or("echec")
                    .trim()
                    .chars()
                    .take(160)
                    .collect();
                outcomes.push(CliUpdateOutcome {
                    engine,
                    command,
                    method: method.into(),
                    version_before: before,
                    version_after: after,
                    status: "error".into(),
                    detail: msg,
                });
            }
            Err(e) => {
                outcomes.push(CliUpdateOutcome {
                    engine,
                    command,
                    method: method.into(),
                    version_before: before,
                    version_after: after,
                    status: "error".into(),
                    detail: e,
                });
            }
        }
    }
    outcomes
}

/// Exécute la synchronisation `automanaged` locale (appelable depuis CLI, UI ou `update.py`).
pub(crate) fn execute_sync(
    run_cli: bool,
    run_models: bool,
    explicit_registry: Option<&Path>,
    dry_run: bool,
) -> Result<AutomanagedState, String> {
    let registries = discover_local_registries(explicit_registry);
    let mut state = load_state();
    let now = now_secs();
    state.last_run_at = Some(now);
    state.last_error = None;

    if run_cli {
        state.cli_results = run_local_cli_updates(&registries, dry_run);
        state.last_cli_check_at = Some(now);
    }
    if run_models {
        let discovered = collect_discovered_models(&registries);
        state.model_results = registries
            .iter()
            .map(|p| sync_one_registry(p, &discovered, dry_run))
            .collect();
        state.last_model_sync_at = Some(now);
    }
    if !dry_run {
        save_state(&state)?;
    }
    Ok(state)
}

pub(crate) fn status_payload() -> Value {
    let cfg = load_config();
    let st = load_state();
    json!({
        "config": cfg,
        "state": st,
        "config_path": config_path().ok().map(|p| p.to_string_lossy().into_owned()),
        "state_path": state_path().ok().map(|p| p.to_string_lossy().into_owned()),
    })
}

pub(crate) fn run(args: &[String]) {
    let sub = args.first().map(String::as_str).unwrap_or("status");
    let rest = if args.is_empty() { &[][..] } else { &args[1..] };
    let res = match sub {
        "status" => cmd_status(rest),
        "on" => cmd_toggle(rest, true),
        "off" => cmd_toggle(rest, false),
        "sync" => cmd_sync(rest),
        _ => Err(
            "Usage : bridget automanaged <status|on|off|sync> [--cli] [--models] [--dry-run] [--registry PATH] [--quiet] [--json]"
                .into(),
        ),
    };
    if let Err(e) = res {
        eprintln!("bridget automanaged : {e}");
        crate::cli_table::exit(1);
    }
}

fn cmd_status(args: &[String]) -> Result<(), String> {
    let json_mode = args.iter().any(|a| a == "--json");
    let payload = status_payload();
    if json_mode {
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
        return Ok(());
    }
    let cfg = load_config();
    let st = load_state();
    println!(
        "Mode automanaged : CLI={} | Modeles={} | Intervalle={}h",
        if cfg.cli_updates {
            "ACTIF"
        } else {
            "DESACTIVE"
        },
        if cfg.model_sync { "ACTIF" } else { "DESACTIVE" },
        cfg.interval_secs / 3600
    );
    if let Some(ts) = st.last_run_at {
        println!("Dernier passage : unix {ts}");
    } else {
        println!("Dernier passage : jamais execute");
    }
    for cli in &st.cli_results {
        println!(
            "  CLI {:12} {:10} {} -> {} ({})",
            cli.engine,
            cli.status,
            cli.version_before.as_deref().unwrap_or("?"),
            cli.version_after.as_deref().unwrap_or("?"),
            cli.method
        );
    }
    for reg in &st.model_results {
        println!(
            "  Modeles {:10} +{} ({})",
            reg.status, reg.total_added, reg.registry_path
        );
    }
    Ok(())
}

fn cmd_toggle(args: &[String], enable: bool) -> Result<(), String> {
    let mut cli_flag = false;
    let mut models_flag = false;
    let mut json_mode = false;
    for a in args {
        match a.as_str() {
            "--cli" => cli_flag = true,
            "--models" => models_flag = true,
            "--json" => json_mode = true,
            other => return Err(format!("option inconnue : {other}")),
        }
    }
    let mut cfg = load_config();
    if !cli_flag && !models_flag {
        cfg.cli_updates = enable;
        cfg.model_sync = enable;
    } else {
        if cli_flag {
            cfg.cli_updates = enable;
        }
        if models_flag {
            cfg.model_sync = enable;
        }
    }
    save_config(&cfg)?;
    if json_mode {
        println!(
            "{}",
            serde_json::to_string_pretty(&status_payload()).unwrap_or_default()
        );
    } else {
        println!(
            "Automanaged mis a jour : CLI={} | Modeles={}",
            if cfg.cli_updates {
                "ACTIF"
            } else {
                "DESACTIVE"
            },
            if cfg.model_sync { "ACTIF" } else { "DESACTIVE" }
        );
    }
    Ok(())
}

fn cmd_sync(args: &[String]) -> Result<(), String> {
    let mut cli_flag = false;
    let mut models_flag = false;
    let mut dry_run = false;
    let mut quiet = false;
    let mut json_mode = false;
    let mut respect_config = false;
    let mut registry: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--cli" => {
                cli_flag = true;
                i += 1;
            }
            "--models" => {
                models_flag = true;
                i += 1;
            }
            "--dry-run" => {
                dry_run = true;
                i += 1;
            }
            "--quiet" => {
                quiet = true;
                i += 1;
            }
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--if-due" => {
                respect_config = true;
                i += 1;
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
    let cfg = load_config();
    let (run_cli, run_models) = if !cli_flag && !models_flag {
        if respect_config {
            let st = load_state();
            let now = now_secs();
            let due_cli = cfg.cli_updates
                && now.saturating_sub(st.last_cli_check_at.unwrap_or(0))
                    >= cfg.interval_secs as i64;
            let due_models = cfg.model_sync
                && now.saturating_sub(st.last_model_sync_at.unwrap_or(0))
                    >= cfg.interval_secs as i64;
            (due_cli, due_models)
        } else {
            (cfg.cli_updates, cfg.model_sync)
        }
    } else {
        (cli_flag, models_flag)
    };

    if !run_cli && !run_models {
        if json_mode {
            println!(
                "{}",
                json!({"status": "skipped", "reason": "not_due_or_disabled"})
            );
        } else if !quiet {
            println!("Automanaged : aucune synchronisation requise.");
        }
        return Ok(());
    }

    let state = execute_sync(run_cli, run_models, registry.as_deref(), dry_run)?;
    if json_mode {
        println!(
            "{}",
            serde_json::to_string_pretty(&state).unwrap_or_default()
        );
    } else if !quiet {
        for cli in &state.cli_results {
            println!(
                "  CLI {:12} {:10} {} -> {} ({})",
                cli.engine,
                cli.status,
                cli.version_before.as_deref().unwrap_or("?"),
                cli.version_after.as_deref().unwrap_or("?"),
                cli.detail
            );
        }
        for reg in &state.model_results {
            println!(
                "  Modeles {:10} +{} ({})",
                reg.status, reg.total_added, reg.registry_path
            );
            for (cid, mods) in &reg.added {
                println!("    {} : +{}", cid, mods.join(", "));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec309_automanaged_defaults_enabled_and_additive_model_merge_preserves_existing() {
        let cfg = AutomanagedConfig::default();
        assert!(cfg.cli_updates);
        assert!(cfg.model_sync);
        assert_eq!(cfg.interval_secs, DEFAULT_INTERVAL_SECS);

        let mut reg = json!({
            "agents": {
                "codex": {
                    "command": "/usr/local/bin/codex",
                    "args": ["app-server"],
                    "protocol": "codex_app_server",
                    "capabilities": {
                        "execution_paths": ["codex_app_server"],
                        "models": {
                            "gpt-6.1-sol": {"efforts": ["low", "high"]}
                        }
                    }
                },
                "codex-legacy": {
                    "command": "/usr/local/bin/codex",
                    "args": ["app-server"],
                    "protocol": "codex_app_server",
                    "capabilities": {
                        "execution_paths": ["codex_app_server"],
                        "models": {
                            "gpt-6.1-terra": {"efforts": ["high"]}
                        }
                    }
                }
            },
            "routing_policy": {
                "version": 2,
                "revision": 4,
                "enabled": true,
                "connections": [
                    {"connection_id": "codex-sysle2", "machine_id": "sysle2", "engine": "codex", "agent_type": "codex", "enabled": true},
                    {"connection_id": "codex-sysle2-terra213", "machine_id": "sysle2", "engine": "codex", "agent_type": "codex-legacy", "enabled": true}
                ],
                "model_catalog": {
                    "codex-sysle2": [
                        {"model": "gpt-6.1-sol", "supported_efforts": ["low", "high"], "default_effort": "low", "provenance": "codex_model_list", "observed_at": 100}
                    ],
                    "codex-sysle2-terra213": [
                        {"model": "gpt-6.1-terra", "supported_efforts": ["high"], "default_effort": "high", "provenance": "declared", "observed_at": null}
                    ]
                }
            }
        });

        let mut discovered = BTreeMap::new();
        discovered.insert(
            "codex-sysle2".to_string(),
            vec![
                DiscoveredModel {
                    model: "gpt-6.1-sol".into(),
                    supported_efforts: vec!["low".into(), "high".into()],
                    default_effort: Some("low".into()),
                },
                DiscoveredModel {
                    model: "gpt-6.3-sol".into(),
                    supported_efforts: vec!["medium".into(), "high".into(), "xhigh".into()],
                    default_effort: Some("high".into()),
                },
            ],
        );
        discovered.insert(
            "codex-sysle2-terra213".to_string(),
            vec![DiscoveredModel {
                model: "gpt-6.3-sol".into(),
                supported_efforts: vec!["high".into()],
                default_effort: Some("high".into()),
            }],
        );

        let added = merge_discovered_models_into_registry_value(&mut reg, &discovered);
        assert_eq!(
            added.get("codex-sysle2"),
            Some(&vec!["gpt-6.3-sol".to_string()])
        );
        // Les alias historiques (-terra213) ne sont pas pollués.
        assert!(!added.contains_key("codex-sysle2-terra213"));
        assert_eq!(reg["routing_policy"]["revision"], 5);
        assert!(reg["agents"]["codex"]["capabilities"]["models"]["gpt-6.1-sol"].is_object());
        assert!(reg["agents"]["codex"]["capabilities"]["models"]["gpt-6.3-sol"].is_object());
        let cat = reg["routing_policy"]["model_catalog"]["codex-sysle2"]
            .as_array()
            .unwrap();
        assert_eq!(cat.len(), 2);

        // Second passage idempotent : aucun doublon ni incrément de révision.
        let added_again = merge_discovered_models_into_registry_value(&mut reg, &discovered);
        assert!(added_again.is_empty());
        assert_eq!(reg["routing_policy"]["revision"], 5);
    }
}
