//! Observation et projection unifiée des quotas restants par abonnement (`bridget usage`).
//!
//! Principes :
//! - Aucun jeton LLM consommé : utilise uniquement les observations passives du daemon
//!   (Claude `rate_limit_event` / StatusLine, Codex `account/rateLimits/read`), le cache
//!   persistant `usage-quotas.json` (`0600`), et la sonde JSON-RPC gratuite Codex
//!   (`codex app-server` → `account/rateLimits/read`, ~0,6 s, 0 jeton) dédupliquée par
//!   compte (`accountId`) et profil avec TTL de 5 minutes.
//! - Honnêteté stricte : jamais de pourcentage fabriqué. Les fournisseurs sans compteur
//!   exposé en lecture libre (Antigravity, Cursor) sont annoncés explicitement comme
//!   `not_exposed_by_provider`. Une fenêtre dont `resets_at <= now` est marquée
//!   `reset_elapsed` plutôt que d'afficher un vieux pourcentage périmé.

use crate::accounts_connect::{ConnectionTarget, load_fleet_connections_with_machines};
use crate::cli_table::table_println as println;
use bridget_transport::protocol::{
    CommunicationDirectoryScope, DaemonToWrapper, RateLimitFact, RateLimitSource,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(crate) const USAGE_CACHE_TTL_SECS: u64 = 300;
const CODEX_PROBE_TIMEOUT_SECS: u64 = 6;
const PASSIVE_FLUSH_MIN_INTERVAL_SECS: i64 = 10;

static CACHE_LOCK: Mutex<i64> = Mutex::new(0);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct QuotaWindow {
    pub window: String,
    pub label: String,
    pub used_percent: Option<u8>,
    pub remaining_percent: Option<u8>,
    pub resets_at: Option<i64>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SubscriptionQuota {
    pub id: String,
    pub engine: String,
    pub provider_label: String,
    pub account_label: String,
    pub machine_id: String,
    #[serde(default)]
    pub machines: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    pub connection_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_type: Option<String>,
    pub source_state: String,
    pub source_label: String,
    pub windows: Vec<QuotaWindow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credits_balance: Option<String>,
    #[serde(default)]
    pub credits_unlimited: bool,
    pub observed_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct UsageSummary {
    pub total_subscriptions: usize,
    pub with_live_quota: usize,
    pub not_exposed: usize,
    pub exhausted: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct UsageReport {
    pub generated_at: i64,
    pub cache_ttl_secs: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_filter: Option<String>,
    pub subscriptions: Vec<SubscriptionQuota>,
    pub summary: UsageSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CachedQuotaEntry {
    engine: String,
    machine_id: String,
    #[serde(default)]
    agent_type: String,
    #[serde(default)]
    connection_id: Option<String>,
    #[serde(default)]
    profile_dir: Option<String>,
    #[serde(default)]
    account_id: Option<String>,
    #[serde(default)]
    plan_type: Option<String>,
    #[serde(default)]
    source: String,
    #[serde(default)]
    windows: BTreeMap<String, CachedWindow>,
    #[serde(default)]
    credits_balance: Option<String>,
    #[serde(default)]
    credits_unlimited: bool,
    observed_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedWindow {
    window: String,
    used_percent: Option<u8>,
    resets_at: Option<i64>,
    status: String,
    observed_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct UsageCacheFile {
    version: u32,
    #[serde(default)]
    entries: BTreeMap<String, CachedQuotaEntry>,
}

struct CodexProbeResult {
    account_id: Option<String>,
    plan_type: Option<String>,
    windows: Vec<CachedWindow>,
    credits_balance: Option<String>,
    credits_unlimited: bool,
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn cache_path() -> Result<PathBuf, String> {
    if let Ok(home) = std::env::var("BRIDGET_HOME")
        && !home.trim().is_empty()
    {
        return Ok(PathBuf::from(home).join("usage-quotas.json"));
    }
    let home = std::env::var("HOME").map_err(|_| "variable HOME absente".to_string())?;
    Ok(PathBuf::from(home)
        .join(".cache")
        .join("bridget-core")
        .join("usage-quotas.json"))
}

/// Normalise un alias fournisseur vers l'identifiant canonique (`claude`, `codex`,
/// `antigravity`, `cursor`), ou `Ok(None)` pour « tous ».
pub(crate) fn normalize_provider_alias(raw: &str) -> Result<Option<&'static str>, String> {
    let trimmed = raw.trim().to_ascii_lowercase();
    if trimmed.is_empty() || matches!(trimmed.as_str(), "all" | "tous" | "global" | "fleet") {
        return Ok(None);
    }
    match trimmed.as_str() {
        "claude" | "anthropic" | "gclaude" | "claude-code" => Ok(Some("claude")),
        "codex" | "openai" | "chatgpt" | "codex-pro" => Ok(Some("codex")),
        "antigravity" | "agy" | "google" | "gemini" => Ok(Some("antigravity")),
        "cursor" | "cursor-agent" => Ok(Some("cursor")),
        _ => Err(format!(
            "fournisseur inconnu '{}' (attendus : claude/anthropic, codex/openai, antigravity/agy/google, cursor, ou all)",
            raw.trim()
        )),
    }
}

pub(crate) fn provider_display_name(engine: &str) -> &'static str {
    match engine {
        "claude" => "Claude (Anthropic)",
        "codex" => "Codex (OpenAI)",
        "antigravity" => "Antigravity (Google)",
        "cursor" => "Cursor",
        _ => "Fournisseur",
    }
}

pub(crate) fn format_window_label(window: &str) -> String {
    let lower = window.to_ascii_lowercase();
    if lower == "five_hour" || lower.contains("five_hour") || lower == "primary/300m" {
        return "5h".to_string();
    }
    if lower == "seven_day"
        || lower.contains("seven_day")
        || lower == "weekly"
        || lower == "primary/10080m"
        || lower == "secondary/10080m"
    {
        return "7j".to_string();
    }
    if let Some(tail) = window.rsplit('/').next()
        && let Some(mins_str) = tail.strip_suffix('m').or_else(|| tail.strip_suffix('M'))
        && let Ok(mins) = mins_str.parse::<i64>()
    {
        if mins % (24 * 60) == 0 {
            return format!("{}j", mins / (24 * 60));
        }
        if mins % 60 == 0 {
            return format!("{}h", mins / 60);
        }
        return format!("{mins}m");
    }
    window.to_string()
}

fn load_cache_from(path: &Path) -> UsageCacheFile {
    let Ok(raw) = fs::read_to_string(path) else {
        return UsageCacheFile {
            version: 1,
            entries: BTreeMap::new(),
        };
    };
    serde_json::from_str::<UsageCacheFile>(&raw).unwrap_or(UsageCacheFile {
        version: 1,
        entries: BTreeMap::new(),
    })
}

fn save_cache_to(path: &Path, cache: &UsageCacheFile) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "dossier parent usage-quotas manquant".to_string())?;
    crate::environment::ensure_private_directory(parent)?;
    let tmp = parent.join(format!(
        ".usage-quotas-{}.tmp",
        uuid::Uuid::new_v4().simple()
    ));
    let mut bytes = serde_json::to_vec_pretty(cache)
        .map_err(|e| format!("sérialisation usage-quotas impossible : {e}"))?;
    bytes.push(b'\n');
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&tmp)
        .map_err(|e| format!("écriture temporaire usage-quotas impossible : {e}"))?;
    f.write_all(&bytes)
        .and_then(|_| f.sync_all())
        .map_err(|e| format!("synchronisation usage-quotas impossible : {e}"))?;
    drop(f);
    fs::rename(&tmp, path).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        format!("remplacement atomique usage-quotas impossible : {e}")
    })?;
    if let Ok(dir) = File::open(parent) {
        let _ = dir.sync_all();
    }
    Ok(())
}

/// Enregistre passivement un fait `RateLimitFact` reçu par le daemon dans le cache privé
/// `usage-quotas.json` (0 jeton, throttlé pour éviter toute tempête I/O).
#[allow(clippy::too_many_arguments)]
pub(crate) fn record_passive_observation(
    engine: &str,
    machine_id: &str,
    agent_type: &str,
    connection_id: Option<&str>,
    profile_dir: Option<&str>,
    fact: &RateLimitFact,
    source: RateLimitSource,
    now: i64,
) {
    if fact.window == "account/credits" && fact.credits.is_none() {
        return;
    }
    let Ok(path) = cache_path() else {
        return;
    };
    let Ok(mut last_flush) = CACHE_LOCK.lock() else {
        return;
    };
    let mut cache = load_cache_from(&path);
    let key = cache_entry_key(engine, machine_id, profile_dir, connection_id, agent_type);
    let entry = cache.entries.entry(key).or_insert_with(|| CachedQuotaEntry {
        engine: engine.to_string(),
        machine_id: machine_id.to_string(),
        agent_type: agent_type.to_string(),
        connection_id: connection_id.map(str::to_string),
        profile_dir: profile_dir.map(str::to_string),
        account_id: None,
        plan_type: None,
        source: format!("{source}"),
        windows: BTreeMap::new(),
        credits_balance: None,
        credits_unlimited: false,
        observed_at: now,
    });
    entry.observed_at = entry.observed_at.max(now);
    entry.source = format!("{source}");
    if let Some(conn) = connection_id {
        entry.connection_id = Some(conn.to_string());
    }
    if let Some(credits) = &fact.credits {
        entry.credits_unlimited = credits.unlimited.unwrap_or(false);
    }
    let mut meaningful_change = false;
    if fact.window != "account/credits" {
        let prev = entry.windows.get(&fact.window);
        if prev.is_none_or(|p| {
            p.used_percent != fact.used_percent
                || p.status != fact.status
                || p.resets_at != fact.resets_at
        }) {
            meaningful_change = true;
        }
        entry.windows.insert(
            fact.window.clone(),
            CachedWindow {
                window: fact.window.clone(),
                used_percent: fact.used_percent,
                resets_at: fact.resets_at,
                status: fact.status.clone(),
                observed_at: now,
            },
        );
    }
    if meaningful_change || now.saturating_sub(*last_flush) >= PASSIVE_FLUSH_MIN_INTERVAL_SECS {
        cache.version = 1;
        if save_cache_to(&path, &cache).is_ok() {
            *last_flush = now;
        }
    }
}

fn cache_entry_key(
    engine: &str,
    machine_id: &str,
    profile_dir: Option<&str>,
    connection_id: Option<&str>,
    agent_type: &str,
) -> String {
    let profile = profile_dir
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("default");
    if profile != "default" {
        return format!("{engine}:{machine_id}:profile:{profile}");
    }
    if let Some(conn) = connection_id.filter(|s| !s.is_empty())
        && engine == "claude"
    {
        return format!("{engine}:{machine_id}:default:{conn}");
    }
    if engine == "codex" {
        return format!("{engine}:{machine_id}:default");
    }
    format!("{engine}:{machine_id}:{agent_type}")
}

fn infer_engine_from_agent_type(agent_type: &str) -> &'static str {
    let lower = agent_type.to_ascii_lowercase();
    if lower.contains("codex") {
        "codex"
    } else if lower.contains("cursor") {
        "cursor"
    } else if lower.contains("antigravity") || lower.contains("agy") || lower.contains("gemini") {
        "antigravity"
    } else {
        "claude"
    }
}

/// Parse la réponse JSON-RPC `account/rateLimits/read` de `codex app-server`.
fn parse_codex_rate_limits_response(value: &Value, now: i64) -> Option<CodexProbeResult> {
    let result = value.get("result")?;
    let account_id = result
        .get("accountId")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let rl = result.get("rateLimits").or_else(|| {
        result
            .get("rateLimitsByLimitId")
            .and_then(|m| m.get("codex"))
    })?;
    let plan_type = rl
        .get("planType")
        .and_then(Value::as_str)
        .map(str::to_string);
    let credits = rl.get("credits");
    let credits_balance = credits
        .and_then(|c| c.get("balance"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let credits_unlimited = credits
        .and_then(|c| c.get("unlimited"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let spend_control = rl
        .get("spendControlReached")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let mut windows = Vec::new();
    for (slot, name) in [("primary", "primary"), ("secondary", "secondary")] {
        let Some(obj) = rl.get(slot).filter(|v| v.is_object()) else {
            continue;
        };
        let used = obj
            .get("usedPercent")
            .and_then(Value::as_u64)
            .map(|u| u.min(100) as u8);
        let mins = obj
            .get("windowDurationMins")
            .and_then(Value::as_i64)
            .unwrap_or(0);
        let resets_at = obj.get("resetsAt").and_then(Value::as_i64);
        let win_name = if mins > 0 {
            format!("{name}/{mins}m")
        } else {
            name.to_string()
        };
        let status = if spend_control || used.is_some_and(|p| p >= 100) {
            "rejected".to_string()
        } else {
            "available".to_string()
        };
        windows.push(CachedWindow {
            window: win_name,
            used_percent: used,
            resets_at,
            status,
            observed_at: now,
        });
    }
    Some(CodexProbeResult {
        account_id,
        plan_type,
        windows,
        credits_balance,
        credits_unlimited,
    })
}

fn probe_codex_app_server(
    command: &str,
    profile_dir: Option<&str>,
    is_local: bool,
    machine_id: &str,
    now: i64,
) -> Result<CodexProbeResult, String> {
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"clientInfo":{"name":"bridget","version":"0.8.53"},"capabilities":{"experimentalApi":true}}}"#;
    let inited = r#"{"jsonrpc":"2.0","method":"initialized"}"#;
    let read_req = r#"{"jsonrpc":"2.0","id":2,"method":"account/rateLimits/read","params":{}}"#;

    let mut child = if is_local {
        let mut cmd = Command::new(command);
        cmd.arg("app-server")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        if let Some(dir) = profile_dir.filter(|s| !s.trim().is_empty()) {
            cmd.env("CODEX_HOME", dir);
        }
        cmd.spawn()
            .map_err(|e| format!("lancement codex app-server local impossible : {e}"))?
    } else {
        let env_prefix = match profile_dir.filter(|s| !s.trim().is_empty()) {
            Some(dir) => format!("CODEX_HOME={} ", shell_single_quote(dir)),
            None => String::new(),
        };
        let remote_cmd = format!(
            "PATH=\"$HOME/.local/bin:/opt/homebrew/bin:/usr/local/bin:$PATH\" {env_prefix}{} app-server",
            shell_single_quote(command)
        );
        Command::new("ssh")
            .args([
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=4",
                "-o",
                "StrictHostKeyChecking=accept-new",
                machine_id,
                &remote_cmd,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("sonde SSH codex sur {machine_id} impossible : {e}"))?
    };

    let Some(mut stdin) = child.stdin.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return Err("flux stdin codex indisponible".to_string());
    };
    let Some(stdout) = child.stdout.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return Err("flux stdout codex indisponible".to_string());
    };

    let (tx, rx) = std::sync::mpsc::channel::<Result<Value, String>>();
    std::thread::spawn(move || {
        let write_ok = writeln!(stdin, "{init}")
            .and_then(|_| stdin.flush())
            .and_then(|_| writeln!(stdin, "{inited}"))
            .and_then(|_| writeln!(stdin, "{read_req}"))
            .and_then(|_| stdin.flush());
        if let Err(e) = write_ok {
            let _ = tx.send(Err(format!("écriture JSON-RPC codex : {e}")));
            return;
        }
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        for _ in 0..32 {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Ok(val) = serde_json::from_str::<Value>(trimmed)
                        && val.get("id").and_then(Value::as_i64) == Some(2)
                    {
                        let _ = tx.send(Ok(val));
                        return;
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(format!("lecture JSON-RPC codex : {e}")));
                    return;
                }
            }
        }
        let _ = tx.send(Err("machine hors ligne ou sonde indisponible".to_string()));
    });

    let outcome = rx
        .recv_timeout(Duration::from_secs(CODEX_PROBE_TIMEOUT_SECS))
        .map_err(|_| format!("délai sonde codex dépassé ({CODEX_PROBE_TIMEOUT_SECS}s)"))?;
    let _ = child.kill();
    let _ = child.wait();
    let val = outcome?;
    if let Some(err) = val.get("error") {
        let msg = err
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("erreur fournisseur");
        return Err(format!("codex account/rateLimits/read refusé : {msg}"));
    }
    parse_codex_rate_limits_response(&val, now)
        .ok_or_else(|| "réponse codex sans bloc rateLimits".to_string())
}

fn shell_single_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn fetch_remote_cache_if_useful(local_machine: &str, machines: &[String]) -> Option<UsageCacheFile> {
    if local_machine == "cartae" || !machines.iter().any(|m| m == "cartae") {
        return None;
    }
    let output = Command::new("ssh")
        .args([
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=3",
            "cartae",
            "cat ~/.cache/bridget-core/usage-quotas.json 2>/dev/null || true",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() || output.stdout.is_empty() {
        return None;
    }
    serde_json::from_slice::<UsageCacheFile>(&output.stdout).ok()
}

fn collect_live_daemon_rate_limits(now: i64) -> Vec<(String, String, Vec<CachedWindow>)> {
    let Ok(ns) = crate::environment::Namespace::from_environment() else {
        return Vec::new();
    };
    let Ok(DaemonToWrapper::AgentList { agents }) = crate::communication::client::directory_request(
        None,
        &ns.socket,
        CommunicationDirectoryScope::Global,
        None,
    ) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for agent in agents {
        if agent.rate_limits.is_empty() {
            continue;
        }
        let engine = infer_engine_from_agent_type(&agent.agent_type).to_string();
        let windows = agent
            .rate_limits
            .into_iter()
            .filter(|f| f.window != "account/credits")
            .map(|f| CachedWindow {
                window: f.window,
                used_percent: f.used_percent,
                resets_at: f.resets_at,
                status: f.status,
                observed_at: now,
            })
            .collect::<Vec<_>>();
        if !windows.is_empty() {
            out.push((engine, agent.agent_type, windows));
        }
    }
    out
}

fn materialize_windows(
    windows: &BTreeMap<String, CachedWindow>,
    now: i64,
) -> (Vec<QuotaWindow>, bool) {
    let mut out = Vec::new();
    let mut any_live = false;
    for w in windows.values() {
        let expired = w.resets_at.is_some_and(|at| at <= now);
        if expired {
            out.push(QuotaWindow {
                window: w.window.clone(),
                label: format_window_label(&w.window),
                used_percent: None,
                remaining_percent: None,
                resets_at: w.resets_at,
                status: "reset_elapsed".to_string(),
            });
        } else {
            any_live = true;
            let remaining = w.used_percent.map(|u| 100u8.saturating_sub(u.min(100)));
            out.push(QuotaWindow {
                window: w.window.clone(),
                label: format_window_label(&w.window),
                used_percent: w.used_percent,
                remaining_percent: remaining,
                resets_at: w.resets_at,
                status: w.status.clone(),
            });
        }
    }
    out.sort_by(|a, b| a.label.cmp(&b.label).then_with(|| a.window.cmp(&b.window)));
    (out, any_live)
}

/// Fusionne les abonnements Codex partageant le même `account_id` (même abonnement
/// OpenAI connecté sur plusieurs serveurs, ex. `cartae` + `monordinateur`).
fn deduplicate_shared_accounts(subs: Vec<SubscriptionQuota>) -> Vec<SubscriptionQuota> {
    let mut by_account: BTreeMap<(String, String), SubscriptionQuota> = BTreeMap::new();
    let mut passthrough: Vec<SubscriptionQuota> = Vec::new();

    for sub in subs {
        let Some(acc_id) = sub.account_id.clone().filter(|s| !s.is_empty()) else {
            passthrough.push(sub);
            continue;
        };
        let key = (sub.engine.clone(), acc_id.clone());
        if let Some(existing) = by_account.get_mut(&key) {
            for m in &sub.machines {
                if !existing.machines.contains(m) {
                    existing.machines.push(m.clone());
                }
            }
            existing.machines.sort();
            existing.machine_id = existing.machines.join(", ");
            for c in &sub.connection_ids {
                if !existing.connection_ids.contains(c) {
                    existing.connection_ids.push(c.clone());
                }
            }
            if sub.observed_at.unwrap_or(0) > existing.observed_at.unwrap_or(0) {
                existing.observed_at = sub.observed_at;
                if !sub.windows.is_empty() {
                    existing.windows = sub.windows;
                    existing.source_state = sub.source_state;
                    existing.source_label = sub.source_label;
                }
                if sub.plan_type.is_some() {
                    existing.plan_type = sub.plan_type;
                }
                if sub.credits_balance.is_some() {
                    existing.credits_balance = sub.credits_balance;
                }
                existing.credits_unlimited |= sub.credits_unlimited;
            }
            // Titre propre quand plusieurs machines partagent le même abonnement.
            if existing.machines.len() > 1 {
                let plan_title = existing
                    .plan_type
                    .as_deref()
                    .map(|p| {
                        let mut chars = p.chars();
                        match chars.next() {
                            Some(first) => format!(" {}", first.to_uppercase().collect::<String>() + chars.as_str()),
                            None => String::new(),
                        }
                    })
                    .unwrap_or_default();
                let base = match existing.engine.as_str() {
                    "codex" => format!("Codex{plan_title}"),
                    "claude" => format!("Claude{plan_title}"),
                    other => other.to_string(),
                };
                existing.account_label = format!("{base} ({})", existing.machines.join(" · "));
            }
        } else {
            let mut first = sub;
            first.id = format!("{}:account:{}", first.engine, acc_id);
            by_account.insert(key, first);
        }
    }

    passthrough.extend(by_account.into_values());
    passthrough
}

/// Collecte les quotas restants sur la flotte sans consommer aucun jeton LLM.
pub(crate) fn collect_usage(
    provider_filter_raw: Option<&str>,
    force_refresh: bool,
    registry_override: Option<&Path>,
) -> Result<UsageReport, String> {
    let filter = match provider_filter_raw {
        Some(raw) => normalize_provider_alias(raw)?,
        None => None,
    };
    let now = now_secs();
    let cpath = cache_path().ok();
    let mut cache = cpath
        .as_deref()
        .map(load_cache_from)
        .unwrap_or(UsageCacheFile {
            version: 1,
            entries: BTreeMap::new(),
        });

    let (local_machine, machines, targets) =
        load_fleet_connections_with_machines(registry_override).unwrap_or_else(|_| {
            (
                crate::accounts_connect::local_machine_name(),
                Vec::new(),
                Vec::new(),
            )
        });

    // Fusionner les observations passives Claude du maître `cartae` (utile depuis le Mac).
    // Pour Codex, la sonde gratuite par profil reste prioritaire afin de ne jamais mélanger
    // deux machines ou deux comptes.
    if let Some(remote_cache) = fetch_remote_cache_if_useful(&local_machine, &machines) {
        for (k, entry) in remote_cache.entries {
            if entry.engine == "codex" && entry.source != "codex_rpc_probe" {
                continue;
            }
            let replace = cache
                .entries
                .get(&k)
                .is_none_or(|cur| cur.observed_at < entry.observed_at);
            if replace {
                cache.entries.insert(k, entry);
            }
        }
    }

    // Intégrer les rate_limits des présences actuellement connectées au daemon.
    let live_daemon = collect_live_daemon_rate_limits(now);

    // Dédupliquer les connexions en profils distincts : (engine, machine_id, profile_dir).
    #[derive(Clone)]
    struct Group {
        id: String,
        engine: String,
        machine_id: String,
        profile_dir: Option<String>,
        command: String,
        is_local: bool,
        account_label: String,
        connection_ids: Vec<String>,
        agent_types: Vec<String>,
    }

    let mut groups: BTreeMap<String, Group> = BTreeMap::new();
    for target in &targets {
        if let Some(f) = filter
            && target.engine != f
        {
            continue;
        }
        let prof = target
            .profile_dir
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());
        let gid = format!(
            "{}:{}:{}",
            target.engine,
            target.machine_id,
            prof.unwrap_or("default")
        );
        let entry = groups.entry(gid.clone()).or_insert_with(|| Group {
            id: gid,
            engine: target.engine.clone(),
            machine_id: target.machine_id.clone(),
            profile_dir: prof.map(str::to_string),
            command: target.command.clone(),
            is_local: target.is_local,
            account_label: friendly_subscription_label(target),
            connection_ids: Vec::new(),
            agent_types: Vec::new(),
        });
        if !entry.connection_ids.contains(&target.connection_id) {
            entry.connection_ids.push(target.connection_id.clone());
        }
        if !entry.agent_types.contains(&target.agent_type) {
            entry.agent_types.push(target.agent_type.clone());
        }
    }

    // Sonde Codex gratuite en parallèle pour les groupes périmés, sans account_id, ou sur refresh.
    let mut cache_dirty = false;
    let mut probe_jobs: Vec<(String, String, Option<String>, bool, String)> = Vec::new();
    for group in groups.values() {
        if group.engine != "codex" {
            continue;
        }
        let ckey = cache_entry_key(
            "codex",
            &group.machine_id,
            group.profile_dir.as_deref(),
            group.connection_ids.first().map(String::as_str),
            group.agent_types.first().map(String::as_str).unwrap_or("codex"),
        );
        let fresh = cache.entries.get(&ckey).is_some_and(|e| {
            !force_refresh
                && e.source == "codex_rpc_probe"
                && e.account_id.is_some()
                && e.observed_at > 0
                && now.saturating_sub(e.observed_at) <= USAGE_CACHE_TTL_SECS as i64
        });
        if !fresh {
            probe_jobs.push((
                group.id.clone(),
                group.command.clone(),
                group.profile_dir.clone(),
                group.is_local,
                group.machine_id.clone(),
            ));
        }
    }

    let mut probe_results: BTreeMap<String, Result<CodexProbeResult, String>> = BTreeMap::new();
    if !probe_jobs.is_empty() {
        let handles: Vec<_> = probe_jobs
            .into_iter()
            .map(|(gid, cmd, prof, is_local, mid)| {
                std::thread::spawn(move || {
                    let res =
                        probe_codex_app_server(&cmd, prof.as_deref(), is_local, &mid, now);
                    (gid, res)
                })
            })
            .collect();
        for h in handles {
            if let Ok((gid, res)) = h.join() {
                probe_results.insert(gid, res);
            }
        }
    }

    let mut subscriptions = Vec::new();
    for group in groups.into_values() {
        let ckey = cache_entry_key(
            &group.engine,
            &group.machine_id,
            group.profile_dir.as_deref(),
            group.connection_ids.first().map(String::as_str),
            group.agent_types.first().map(String::as_str).unwrap_or(&group.engine),
        );

        match group.engine.as_str() {
            "codex" => {
                let mut probe_err: Option<String> = None;
                if let Some(res) = probe_results.remove(&group.id) {
                    match res {
                        Ok(pr) => {
                            let mut map = BTreeMap::new();
                            for w in pr.windows {
                                map.insert(w.window.clone(), w);
                            }
                            cache.entries.insert(
                                ckey.clone(),
                                CachedQuotaEntry {
                                    engine: "codex".to_string(),
                                    machine_id: group.machine_id.clone(),
                                    agent_type: group
                                        .agent_types
                                        .first()
                                        .cloned()
                                        .unwrap_or_default(),
                                    connection_id: group.connection_ids.first().cloned(),
                                    profile_dir: group.profile_dir.clone(),
                                    account_id: pr.account_id,
                                    plan_type: pr.plan_type,
                                    source: "codex_rpc_probe".to_string(),
                                    windows: map,
                                    credits_balance: pr.credits_balance,
                                    credits_unlimited: pr.credits_unlimited,
                                    observed_at: now,
                                },
                            );
                            cache_dirty = true;
                        }
                        Err(e) => {
                            probe_err = Some(e);
                        }
                    }
                }
                let cached = cache.entries.get(&ckey);
                let (windows, any_live) = cached
                    .map(|e| materialize_windows(&e.windows, now))
                    .unwrap_or_else(|| (Vec::new(), false));
                let (source_state, source_label, detail) = if any_live {
                    (
                        "live_probe".to_string(),
                        "Sonde gratuite Codex (0 jeton)".to_string(),
                        probe_err.map(|e| format!("Cache conservé ({e})")),
                    )
                } else if !windows.is_empty() {
                    (
                        "stale_reset".to_string(),
                        "Fenêtre réinitialisée".to_string(),
                        probe_err,
                    )
                } else {
                    (
                        "unavailable".to_string(),
                        "Hors ligne ou indisponible".to_string(),
                        Some(probe_err.unwrap_or_else(|| {
                            "Aucune donnée de quota reçue pour ce profil Codex.".to_string()
                        })),
                    )
                };
                subscriptions.push(SubscriptionQuota {
                    id: group.id,
                    engine: group.engine,
                    provider_label: provider_display_name("codex").to_string(),
                    account_label: group.account_label,
                    machine_id: group.machine_id.clone(),
                    machines: vec![group.machine_id],
                    account_id: cached.and_then(|e| e.account_id.clone()),
                    connection_ids: group.connection_ids,
                    plan_type: cached.and_then(|e| e.plan_type.clone()),
                    source_state,
                    source_label,
                    windows,
                    credits_balance: cached.and_then(|e| e.credits_balance.clone()),
                    credits_unlimited: cached.is_some_and(|e| e.credits_unlimited),
                    observed_at: cached.map(|e| e.observed_at),
                    detail,
                });
            }
            "claude" => {
                let mut merged_windows: BTreeMap<String, CachedWindow> = BTreeMap::new();
                let mut latest_obs: Option<i64> = None;

                for entry in cache.entries.values() {
                    if entry.engine != "claude" {
                        continue;
                    }
                    let same_machine = entry.machine_id == group.machine_id;
                    let same_conn = entry
                        .connection_id
                        .as_deref()
                        .is_some_and(|cid| group.connection_ids.iter().any(|c| c == cid));
                    let same_type = !entry.agent_type.is_empty()
                        && group.agent_types.iter().any(|t| t == &entry.agent_type);
                    if same_conn
                        || (same_machine && (same_type || entry.profile_dir == group.profile_dir))
                    {
                        latest_obs = latest_obs.max(Some(entry.observed_at));
                        for (wk, wv) in &entry.windows {
                            if merged_windows
                                .get(wk)
                                .is_none_or(|cur| cur.observed_at < wv.observed_at)
                            {
                                merged_windows.insert(wk.clone(), wv.clone());
                            }
                        }
                    }
                }
                for (eng, atype, wins) in &live_daemon {
                    if eng == "claude" && group.agent_types.iter().any(|t| t == atype) {
                        for w in wins {
                            latest_obs = latest_obs.max(Some(w.observed_at));
                            if merged_windows
                                .get(&w.window)
                                .is_none_or(|cur| cur.observed_at <= w.observed_at)
                            {
                                merged_windows.insert(w.window.clone(), w.clone());
                            }
                        }
                    }
                }

                let (windows, any_live) = materialize_windows(&merged_windows, now);
                let (source_state, source_label, detail) = if any_live {
                    (
                        "observed".to_string(),
                        "Observé passivement (sessions Claude, 0 jeton)".to_string(),
                        None,
                    )
                } else if !windows.is_empty() {
                    (
                        "stale_reset".to_string(),
                        "Fenêtre réinitialisée — en attente d'une session Claude".to_string(),
                        Some(
                            "Le compteur précédent est arrivé à échéance ; la prochaine session Claude actualisera la jauge sans coût."
                                .to_string(),
                        ),
                    )
                } else {
                    (
                        "unavailable".to_string(),
                        "En attente d'une session Claude (0 jeton)".to_string(),
                        Some(
                            "Claude publie ses jauges 5h/7j pendant ses sessions actives (rate_limit_event / StatusLine). Aucun jeton n'est dépensé pour sonder."
                                .to_string(),
                        ),
                    )
                };
                subscriptions.push(SubscriptionQuota {
                    id: group.id,
                    engine: group.engine,
                    provider_label: provider_display_name("claude").to_string(),
                    account_label: group.account_label,
                    machine_id: group.machine_id.clone(),
                    machines: vec![group.machine_id],
                    account_id: None,
                    connection_ids: group.connection_ids,
                    plan_type: None,
                    source_state,
                    source_label,
                    windows,
                    credits_balance: None,
                    credits_unlimited: false,
                    observed_at: latest_obs,
                    detail,
                });
            }
            "antigravity" | "cursor" => {
                let engine_name = group.engine.clone();
                subscriptions.push(SubscriptionQuota {
                    id: group.id,
                    engine: engine_name.clone(),
                    provider_label: provider_display_name(&engine_name).to_string(),
                    account_label: group.account_label,
                    machine_id: group.machine_id.clone(),
                    machines: vec![group.machine_id],
                    account_id: None,
                    connection_ids: group.connection_ids,
                    plan_type: None,
                    source_state: "not_exposed_by_provider".to_string(),
                    source_label: "Non exposé en lecture libre par le fournisseur".to_string(),
                    windows: Vec::new(),
                    credits_balance: None,
                    credits_unlimited: false,
                    observed_at: None,
                    detail: Some(format!(
                        "{} ne publie pas de compteur de quota restant via sa CLI sans consommer de requête.",
                        provider_display_name(&engine_name)
                    )),
                });
            }
            _ => {}
        }
    }

    if cache_dirty && let Some(path) = cpath.as_deref() {
        cache.version = 1;
        let _ = save_cache_to(path, &cache);
    }

    // Dédupliquer les comptes partagés entre machines (ex. même abonnement Codex Pro sur 2 serveurs).
    subscriptions = deduplicate_shared_accounts(subscriptions);

    // Trier : jauges actives d'abord, puis par moteur et libellé.
    subscriptions.sort_by(|a, b| {
        let rank = |s: &str| match s {
            "live_probe" | "observed" => 0,
            "stale_reset" => 1,
            "unavailable" => 2,
            _ => 3,
        };
        rank(&a.source_state)
            .cmp(&rank(&b.source_state))
            .then_with(|| a.engine.cmp(&b.engine))
            .then_with(|| a.account_label.cmp(&b.account_label))
    });

    let total_subscriptions = subscriptions.len();
    let with_live_quota = subscriptions
        .iter()
        .filter(|s| matches!(s.source_state.as_str(), "live_probe" | "observed"))
        .count();
    let not_exposed = subscriptions
        .iter()
        .filter(|s| s.source_state == "not_exposed_by_provider")
        .count();
    let exhausted = subscriptions
        .iter()
        .filter(|s| {
            s.windows
                .iter()
                .any(|w| w.status == "rejected" || w.used_percent.is_some_and(|p| p >= 100))
        })
        .count();

    Ok(UsageReport {
        generated_at: now,
        cache_ttl_secs: USAGE_CACHE_TTL_SECS,
        provider_filter: filter.map(str::to_string),
        subscriptions,
        summary: UsageSummary {
            total_subscriptions,
            with_live_quota,
            not_exposed,
            exhausted,
        },
    })
}

fn friendly_subscription_label(target: &ConnectionTarget) -> String {
    if let Some(name) = target
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        return name.to_string();
    }
    let base = match target.engine.as_str() {
        "claude" => "Claude",
        "codex" => {
            if target.connection_id.contains("pro") {
                "Codex Pro"
            } else {
                "Codex"
            }
        }
        "antigravity" => "Antigravity",
        "cursor" => "Cursor",
        other => other,
    };
    format!("{base} · {}", target.machine_id)
}

fn format_age_fr(observed_at: Option<i64>, now: i64) -> String {
    let Some(at) = observed_at else {
        return "jamais observé".to_string();
    };
    let delta = now.saturating_sub(at).max(0);
    if delta < 10 {
        "à l'instant".to_string()
    } else if delta < 60 {
        format!("il y a {delta}s")
    } else if delta < 3600 {
        format!("il y a {} min", delta / 60)
    } else {
        format!("il y a {}h", delta / 3600)
    }
}

fn format_reset_delta_fr(resets_at: Option<i64>, now: i64) -> String {
    let Some(at) = resets_at else {
        return String::new();
    };
    let diff = at - now;
    if diff <= 0 {
        return "réinitialisé".to_string();
    }
    let hours = diff / 3600;
    let mins = (diff % 3600) / 60;
    if hours >= 24 {
        format!("reset dans {}j {}h", hours / 24, hours % 24)
    } else if hours > 0 {
        format!("reset dans {hours}h {mins:02}m")
    } else {
        format!("reset dans {mins} min")
    }
}

fn render_unicode_bar(remaining_pct: u8) -> String {
    let filled = ((remaining_pct as usize * 12) + 50) / 100;
    let filled = filled.min(12);
    format!("{}{}", "█".repeat(filled), "░".repeat(12 - filled))
}

struct CompactProviderGroup {
    engine: String,
    provider_label: String,
    source_state: String,
    short_badge: String,
    machines: Vec<String>,
    connections_count: usize,
    reason: String,
}

fn split_active_and_compact<'a>(
    subs: &'a [SubscriptionQuota],
) -> (Vec<&'a SubscriptionQuota>, Vec<CompactProviderGroup>) {
    let mut active = Vec::new();
    let mut grouped: BTreeMap<(String, String), (BTreeSet<String>, usize, String, String)> =
        BTreeMap::new();

    for s in subs {
        if matches!(
            s.source_state.as_str(),
            "live_probe" | "observed" | "stale_reset"
        ) && !s.windows.is_empty()
        {
            active.push(s);
            continue;
        }
        let key = (s.engine.clone(), s.source_state.clone());
        let entry = grouped.entry(key).or_insert_with(|| {
            let reason = match (s.engine.as_str(), s.source_state.as_str()) {
                ("claude", _) => {
                    "Jauges 5h/7j mises à jour passivement dès la prochaine session Claude active (0 jeton)."
                        .to_string()
                }
                (_, "not_exposed_by_provider") => {
                    "Compteur de quota restant non exposé en lecture libre par le fournisseur."
                        .to_string()
                }
                _ => s
                    .detail
                    .clone()
                    .unwrap_or_else(|| "Machine hors ligne ou sonde indisponible.".to_string()),
            };
            (
                BTreeSet::new(),
                0,
                s.provider_label.clone(),
                reason,
            )
        });
        if s.machines.is_empty() {
            entry.0.insert(s.machine_id.clone());
        } else {
            for m in &s.machines {
                entry.0.insert(m.clone());
            }
        }
        entry.1 += s.connection_ids.len().max(1);
    }

    let compact = grouped
        .into_iter()
        .map(|((engine, source_state), (machines, count, provider_label, reason))| {
            let short_badge = match source_state.as_str() {
                "not_exposed_by_provider" => "Non exposé".to_string(),
                _ if engine == "claude" => "En attente".to_string(),
                _ => "Indisponible".to_string(),
            };
            CompactProviderGroup {
                engine,
                provider_label,
                source_state,
                short_badge,
                machines: machines.into_iter().collect(),
                connections_count: count,
                reason,
            }
        })
        .collect();

    (active, compact)
}

/// Rendu terminal propre avec jauges Unicode de quota restant.
pub(crate) fn render_terminal(report: &UsageReport) -> String {
    let mut out = String::new();
    let title = match &report.provider_filter {
        Some(p) => format!("Quotas restants Bridget · {}", provider_display_name(p)),
        None => "Quotas restants Bridget · Abonnements".to_string(),
    };
    out.push_str(&format!("{title}\n"));
    out.push_str(&format!(
        "{} jauge(s) active(s) · 0 jeton LLM consommé\n\n",
        report.summary.with_live_quota,
    ));
    let (active, compact) = split_active_and_compact(&report.subscriptions);
    if active.is_empty() && compact.is_empty() {
        out.push_str("Aucun abonnement trouvé pour ce filtre.\n");
        return out;
    }
    for sub in &active {
        let plan = sub
            .plan_type
            .as_deref()
            .map(|p| format!(" [{}]", p.to_ascii_uppercase()))
            .unwrap_or_default();
        out.push_str(&format!(
            "• {}{} — {} ({})\n",
            sub.account_label,
            plan,
            sub.source_label,
            format_age_fr(sub.observed_at, report.generated_at)
        ));
        for w in &sub.windows {
            if w.status == "reset_elapsed" {
                out.push_str(&format!(
                    "    {:>4}  [────────────] fenêtre réinitialisée\n",
                    w.label
                ));
            } else if let Some(rem) = w.remaining_percent {
                let used = w.used_percent.unwrap_or(100u8.saturating_sub(rem));
                let rst = format_reset_delta_fr(w.resets_at, report.generated_at);
                let rst_part = if rst.is_empty() {
                    String::new()
                } else {
                    format!(" · {rst}")
                };
                out.push_str(&format!(
                    "    {:>4}  [{}] {:>3}% restant ({}% utilisé){}\n",
                    w.label,
                    render_unicode_bar(rem),
                    rem,
                    used,
                    rst_part
                ));
            } else if w.status == "rejected" {
                let rst = format_reset_delta_fr(w.resets_at, report.generated_at);
                out.push_str(&format!(
                    "    {:>4}  [░░░░░░░░░░░░] épuisé · {rst}\n",
                    w.label
                ));
            }
        }
    }
    if !compact.is_empty() {
        if !active.is_empty() {
            out.push('\n');
        }
        out.push_str("Autres fournisseurs :\n");
        for g in compact {
            out.push_str(&format!(
                "  • {} [{}] — {} ({} connexion(s)) : {}\n",
                g.provider_label,
                g.short_badge,
                g.machines.join(", "),
                g.connections_count,
                g.reason
            ));
        }
    }
    out
}

fn html_esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Rendu HTML autonome avec jauges propres (compatible `html_render` T3 et navigateur).
pub(crate) fn render_html(report: &UsageReport) -> String {
    let (active, compact) = split_active_and_compact(&report.subscriptions);
    let mut cards = String::new();

    for sub in &active {
        let plan_badge = sub
            .plan_type
            .as_deref()
            .map(|p| format!(r#"<span class="pill plan">{}</span>"#, html_esc(p)))
            .unwrap_or_default();
        let short_status = match sub.source_state.as_str() {
            "live_probe" => ("ok", "● Live"),
            "observed" => ("ok", "● Passif"),
            "stale_reset" => ("warn", "◌ Réinitialisé"),
            _ => ("muted", "○ Inconnu"),
        };
        let age = format_age_fr(sub.observed_at, report.generated_at);
        let machines_txt = if sub.machines.is_empty() {
            sub.machine_id.clone()
        } else {
            sub.machines.join(" · ")
        };
        let mut body = String::new();
        for w in &sub.windows {
            if w.status == "reset_elapsed" {
                body.push_str(&format!(
                    r#"<div class="win"><div class="win-head"><span class="win-label">Fenêtre {}</span><span class="win-val muted">Réinitialisée</span></div><div class="track"><div class="fill muted" style="width:0%"></div></div></div>"#,
                    html_esc(&w.label)
                ));
                continue;
            }
            let used = w
                .used_percent
                .unwrap_or(if w.status == "rejected" { 100 } else { 0 });
            let rem = w.remaining_percent.unwrap_or(100u8.saturating_sub(used));
            let tone = if w.status == "rejected" || used >= 90 {
                "danger"
            } else if used >= 70 {
                "warn"
            } else {
                "ok"
            };
            let rst = format_reset_delta_fr(w.resets_at, report.generated_at);
            let rst_html = if rst.is_empty() {
                String::new()
            } else {
                format!(r#"<span class="win-rst">{}</span>"#, html_esc(&rst))
            };
            body.push_str(&format!(
                r#"<div class="win"><div class="win-head"><span class="win-label">Fenêtre {}</span><span class="win-val {}"><strong>{}%</strong> restant <span class="muted">({}% utilisé)</span></span></div><div class="track"><div class="fill {}" style="width:{}%"></div></div><div class="win-foot">{}</div></div>"#,
                html_esc(&w.label),
                tone,
                rem,
                used,
                tone,
                rem,
                rst_html
            ));
        }
        cards.push_str(&format!(
            r#"<section class="card"><div class="card-head"><div class="card-meta"><div class="card-title"><span>{}</span>{}</div><div class="card-sub">{} · {} · {}</div></div><span class="badge {}">{}</span></div>{}</section>"#,
            html_esc(&sub.account_label),
            plan_badge,
            html_esc(&sub.provider_label),
            html_esc(&machines_txt),
            html_esc(&age),
            short_status.0,
            short_status.1,
            body
        ));
    }

    let active_section = if cards.is_empty() {
        String::new()
    } else {
        format!(r#"<div class="grid">{cards}</div>"#)
    };

    let compact_section = if compact.is_empty() {
        String::new()
    } else {
        let mut rows = String::new();
        for g in &compact {
            let tone = match g.source_state.as_str() {
                "not_exposed_by_provider" => "muted",
                _ if g.engine == "claude" => "warn",
                _ => "muted",
            };
            let plural = if g.connections_count > 1 { "s" } else { "" };
            rows.push_str(&format!(
                r#"<div class="other-row"><div class="other-main"><div class="other-head"><strong>{}</strong><span class="badge {}">{}</span><span class="other-hosts">{} ({} connexion{})</span></div><div class="other-reason">{}</div></div></div>"#,
                html_esc(&g.provider_label),
                tone,
                html_esc(&g.short_badge),
                html_esc(&g.machines.join(", ")),
                g.connections_count,
                plural,
                html_esc(&g.reason)
            ));
        }
        format!(r#"<section class="other-box">{rows}</section>"#)
    };

    let content = if active_section.is_empty() && compact_section.is_empty() {
        r#"<div class="empty-note">Aucun abonnement détecté pour ce filtre.</div>"#.to_string()
    } else {
        format!("{active_section}{compact_section}")
    };

    let heading = match &report.provider_filter {
        Some(p) => format!("Quotas restants · {}", provider_display_name(p)),
        None => "Quotas restants · Abonnements Bridget".to_string(),
    };

    format!(
        r#"<!doctype html><html lang="fr"><head><meta charset="utf-8" /><style>
:root {{ color-scheme: light dark; }}
* {{ box-sizing: border-box; }}
body {{ margin: 0; font-family: var(--font-sans, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif); color: var(--foreground, #e5e7eb); font-size: 13px; }}
.wrap {{ display: flex; flex-direction: column; gap: 12px; }}
.hdr {{ display: flex; justify-content: space-between; align-items: baseline; gap: 12px; flex-wrap: wrap; }}
.hdr h1 {{ margin: 0; font-size: 15px; font-weight: 600; }}
.hdr-meta {{ font-size: 12px; color: var(--muted-foreground, var(--muted, #9ca3af)); }}
.grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(290px, 1fr)); gap: 12px; }}
.card {{ background: var(--card, rgba(127,127,127,0.08)); border: 1px solid var(--border, rgba(127,127,127,0.25)); border-radius: var(--radius, 10px); padding: 14px 16px; display: flex; flex-direction: column; gap: 11px; }}
.card-head {{ display: flex; justify-content: space-between; align-items: flex-start; gap: 10px; }}
.card-meta {{ min-width: 0; flex: 1; }}
.card-title {{ font-weight: 600; font-size: 13.5px; display: flex; align-items: center; gap: 6px; flex-wrap: wrap; line-height: 1.3; }}
.card-sub {{ font-size: 11.5px; color: var(--muted-foreground, var(--muted, #9ca3af)); margin-top: 3px; }}
.pill {{ font-size: 10.5px; padding: 1px 7px; border-radius: 999px; border: 1px solid var(--border, rgba(127,127,127,0.28)); color: var(--muted-foreground, var(--muted, #9ca3af)); white-space: nowrap; }}
.pill.plan {{ text-transform: uppercase; font-weight: 600; color: var(--foreground, #e5e7eb); }}
.badge {{ font-size: 11px; padding: 2px 8px; border-radius: 999px; border: 1px solid var(--border, rgba(127,127,127,0.28)); white-space: nowrap; flex-shrink: 0; font-weight: 500; }}
.badge.ok {{ color: var(--success, #22c55e); border-color: color-mix(in srgb, var(--success, #22c55e) 35%, transparent); }}
.badge.warn {{ color: var(--warning, #f59e0b); border-color: color-mix(in srgb, var(--warning, #f59e0b) 35%, transparent); }}
.badge.muted {{ color: var(--muted-foreground, var(--muted, #9ca3af)); }}
.win {{ display: flex; flex-direction: column; gap: 5px; }}
.win-head {{ display: flex; justify-content: space-between; align-items: baseline; gap: 8px; font-size: 12.5px; }}
.win-label {{ font-weight: 500; }}
.win-val.ok strong {{ color: var(--success, #22c55e); }}
.win-val.warn strong {{ color: var(--warning, #f59e0b); }}
.win-val.danger strong {{ color: var(--destructive, #ef4444); }}
.muted {{ color: var(--muted-foreground, var(--muted, #9ca3af)); }}
.track {{ height: 8px; border-radius: 999px; background: var(--secondary, rgba(127,127,127,0.18)); overflow: hidden; }}
.fill {{ height: 100%; border-radius: 999px; transition: width 0.25s ease; }}
.fill.ok {{ background: var(--success, #22c55e); }}
.fill.warn {{ background: var(--warning, #f59e0b); }}
.fill.danger {{ background: var(--destructive, #ef4444); }}
.fill.muted {{ background: var(--muted, #6b7280); }}
.win-foot {{ font-size: 11px; color: var(--muted-foreground, var(--muted, #9ca3af)); display: flex; justify-content: flex-end; }}
.other-box {{ background: var(--card, rgba(127,127,127,0.06)); border: 1px solid var(--border, rgba(127,127,127,0.22)); border-radius: var(--radius, 10px); padding: 4px 14px; display: flex; flex-direction: column; }}
.other-row {{ padding: 9px 0; border-bottom: 1px solid var(--border, rgba(127,127,127,0.18)); }}
.other-row:last-child {{ border-bottom: 0; }}
.other-head {{ display: flex; align-items: center; gap: 8px; flex-wrap: wrap; font-size: 12.5px; }}
.other-hosts {{ font-size: 11.5px; color: var(--muted-foreground, var(--muted, #9ca3af)); }}
.other-reason {{ font-size: 11.5px; color: var(--muted-foreground, var(--muted, #9ca3af)); margin-top: 3px; line-height: 1.4; }}
.empty-note {{ font-size: 12px; color: var(--muted-foreground, var(--muted, #9ca3af)); line-height: 1.45; }}
</style></head><body><div class="wrap"><div class="hdr"><h1>{}</h1><span class="hdr-meta">{} jauge(s) active(s) · 0 jeton LLM consommé</span></div>{}</div></body></html>"#,
        html_esc(&heading),
        report.summary.with_live_quota,
        content
    )
}

/// Point d'entrée CLI `bridget usage [provider|alias] [--refresh] [--json] [--html]`.
pub(crate) fn run_cli(args: &[String]) {
    let mut provider: Option<String> = None;
    let mut refresh = false;
    let mut as_json = false;
    let mut as_html = false;

    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => {
                println!(
                    "bridget usage [fournisseur] [--refresh] [--json] [--html]\n  fournisseur : claude/anthropic | codex/openai | antigravity/agy/google | cursor | all\n  --refresh   : relance les sondes gratuites (0 jeton LLM) sans attendre le TTL de 5 min\n  --json      : sortie JSON structurée\n  --html      : sortie HTML autonome avec jauges de quotas restants"
                );
                return;
            }
            "--refresh" => refresh = true,
            "--json" => as_json = true,
            "--html" => as_html = true,
            other if other.starts_with('-') => {
                eprintln!("option inconnue pour bridget usage : {other}");
                crate::cli_table::exit(2);
            }
            other => {
                if provider.is_some() {
                    eprintln!("un seul filtre fournisseur attendu (reçu '{other}')");
                    crate::cli_table::exit(2);
                }
                provider = Some(other.to_string());
            }
        }
    }

    match collect_usage(provider.as_deref(), refresh, None) {
        Ok(report) => {
            if as_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).unwrap_or_default()
                );
            } else if as_html {
                println!("{}", render_html(&report));
            } else {
                println!("{}", render_terminal(&report));
            }
        }
        Err(e) => {
            eprintln!("erreur bridget usage : {e}");
            crate::cli_table::exit(1);
        }
    }
}

/// Point d'entrée JSON pour la console web (`/api/usage` et `/api/usage/refresh`).
pub(crate) fn usage_api_payload(
    provider: Option<&str>,
    refresh: bool,
    registry_override: Option<&Path>,
) -> Result<Value, String> {
    let report = collect_usage(provider, refresh, registry_override)?;
    serde_json::to_value(report).map_err(|e| format!("sérialisation usage : {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn spec342_provider_aliases_normalize_cleanly() {
        assert_eq!(normalize_provider_alias("anthropic").unwrap(), Some("claude"));
        assert_eq!(normalize_provider_alias("Claude").unwrap(), Some("claude"));
        assert_eq!(normalize_provider_alias("openai").unwrap(), Some("codex"));
        assert_eq!(normalize_provider_alias("CODEX").unwrap(), Some("codex"));
        assert_eq!(normalize_provider_alias("agy").unwrap(), Some("antigravity"));
        assert_eq!(normalize_provider_alias("google").unwrap(), Some("antigravity"));
        assert_eq!(normalize_provider_alias("cursor-agent").unwrap(), Some("cursor"));
        assert_eq!(normalize_provider_alias("all").unwrap(), None);
        assert!(normalize_provider_alias("inexistant").is_err());
    }

    #[test]
    fn spec342_parse_codex_rate_limits_dedup_by_account_and_render_compact_html() {
        let now = 1_795_500_000;
        let payload = json!({
            "id": 2,
            "result": {
                "accountId": "82ae4828-611c-4a88-9020-dac5ee4a001c",
                "rateLimits": {
                    "planType": "pro",
                    "primary": {
                        "usedPercent": 44,
                        "windowDurationMins": 10080,
                        "resetsAt": now + 3600 * 48
                    },
                    "secondary": {
                        "usedPercent": 15,
                        "windowDurationMins": 300,
                        "resetsAt": now + 7200
                    },
                    "credits": {
                        "hasCredits": false,
                        "unlimited": false,
                        "balance": "0"
                    },
                    "spendControlReached": false
                }
            }
        });
        let pr = parse_codex_rate_limits_response(&payload, now).unwrap();
        assert_eq!(
            pr.account_id.as_deref(),
            Some("82ae4828-611c-4a88-9020-dac5ee4a001c")
        );
        assert_eq!(pr.plan_type.as_deref(), Some("pro"));
        assert_eq!(pr.windows.len(), 2);

        let mut map = BTreeMap::new();
        for w in pr.windows {
            map.insert(w.window.clone(), w);
        }
        let (mat, any_live) = materialize_windows(&map, now);
        assert!(any_live);
        assert_eq!(mat[0].label, "5h");
        assert_eq!(mat[0].remaining_percent, Some(85));
        assert_eq!(mat[1].label, "7j");
        assert_eq!(mat[1].remaining_percent, Some(56));

        // Deux serveurs partageant le même accountId Codex doivent fusionner en 1 seul abonnement.
        let deduped = deduplicate_shared_accounts(vec![
            SubscriptionQuota {
                id: "codex:cartae:default".into(),
                engine: "codex".into(),
                provider_label: provider_display_name("codex").into(),
                account_label: "Cartae Codex".into(),
                machine_id: "cartae".into(),
                machines: vec!["cartae".into()],
                account_id: pr.account_id.clone(),
                connection_ids: vec!["codex-cartae-normal".into()],
                plan_type: pr.plan_type.clone(),
                source_state: "live_probe".into(),
                source_label: "Sonde gratuite Codex (0 jeton)".into(),
                windows: mat.clone(),
                credits_balance: pr.credits_balance.clone(),
                credits_unlimited: pr.credits_unlimited,
                observed_at: Some(now - 20),
                detail: None,
            },
            SubscriptionQuota {
                id: "codex:monordinateur:default".into(),
                engine: "codex".into(),
                provider_label: provider_display_name("codex").into(),
                account_label: "Mac Codex".into(),
                machine_id: "monordinateur".into(),
                machines: vec!["monordinateur".into()],
                account_id: pr.account_id,
                connection_ids: vec!["codex-mac-normal".into()],
                plan_type: pr.plan_type,
                source_state: "live_probe".into(),
                source_label: "Sonde gratuite Codex (0 jeton)".into(),
                windows: mat,
                credits_balance: pr.credits_balance,
                credits_unlimited: pr.credits_unlimited,
                observed_at: Some(now - 5),
                detail: None,
            },
            SubscriptionQuota {
                id: "antigravity:cartae:default".into(),
                engine: "antigravity".into(),
                provider_label: provider_display_name("antigravity").into(),
                account_label: "Antigravity Cartae".into(),
                machine_id: "cartae".into(),
                machines: vec!["cartae".into()],
                account_id: None,
                connection_ids: vec!["antigravity-cartae".into()],
                plan_type: None,
                source_state: "not_exposed_by_provider".into(),
                source_label: "Non exposé".into(),
                windows: vec![],
                credits_balance: None,
                credits_unlimited: false,
                observed_at: None,
                detail: Some("Non exposé".into()),
            },
            SubscriptionQuota {
                id: "antigravity:monordinateur:default".into(),
                engine: "antigravity".into(),
                provider_label: provider_display_name("antigravity").into(),
                account_label: "Antigravity Mac".into(),
                machine_id: "monordinateur".into(),
                machines: vec!["monordinateur".into()],
                account_id: None,
                connection_ids: vec!["antigravity-mac".into()],
                plan_type: None,
                source_state: "not_exposed_by_provider".into(),
                source_label: "Non exposé".into(),
                windows: vec![],
                credits_balance: None,
                credits_unlimited: false,
                observed_at: None,
                detail: Some("Non exposé".into()),
            },
        ]);

        // 2 Codex fusionnés en 1 + 2 Antigravity = 3 entrées dans `subscriptions`,
        // et dans le rendu HTML les 2 Antigravity sont regroupés en 1 seule ligne compacte.
        assert_eq!(deduped.len(), 3);
        let codex_sub = deduped.iter().find(|s| s.engine == "codex").unwrap();
        assert_eq!(codex_sub.machines, vec!["cartae", "monordinateur"]);
        assert_eq!(codex_sub.connection_ids.len(), 2);

        let report = UsageReport {
            generated_at: now,
            cache_ttl_secs: USAGE_CACHE_TTL_SECS,
            provider_filter: None,
            subscriptions: deduped,
            summary: UsageSummary {
                total_subscriptions: 3,
                with_live_quota: 1,
                not_exposed: 2,
                exhausted: 0,
            },
        };

        let term = render_terminal(&report);
        assert!(term.contains("85% restant"));
        assert!(term.contains("56% restant"));
        assert!(term.contains("cartae, monordinateur"));

        let html = render_html(&report);
        assert!(html.contains("85%"));
        assert!(html.contains("56%"));
        assert!(html.contains("0 jeton LLM consommé"));
        assert!(html.contains("other-box"));
        assert!(html.contains("2 connexions"));
    }

    #[test]
    fn spec342_expired_window_is_marked_reset_elapsed_never_stale_percent() {
        let now = 1_795_500_000;
        let mut map = BTreeMap::new();
        map.insert(
            "five_hour".to_string(),
            CachedWindow {
                window: "five_hour".to_string(),
                used_percent: Some(92),
                resets_at: Some(now - 60),
                status: "allowed".to_string(),
                observed_at: now - 3600,
            },
        );
        let (mat, any_live) = materialize_windows(&map, now);
        assert!(!any_live);
        assert_eq!(mat.len(), 1);
        assert_eq!(mat[0].status, "reset_elapsed");
        assert_eq!(mat[0].remaining_percent, None);
    }
}
