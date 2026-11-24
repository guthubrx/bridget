//! Observation et projection unifiée des quotas restants par abonnement (`bridget usage`).
//!
//! Principes :
//! - Aucun jeton LLM consommé :
//!   - **Codex (OpenAI)** : sonde JSON-RPC officielle `codex app-server` → `account/rateLimits/read`
//!     (~0,6 s, 0 jeton, incluant `accountId`, fenêtres 5h/7j et `rateLimitResetCredits`).
//!   - **Antigravity (Google)** : sonde CLI officielle `agy -p /usage --output-format json`
//!     (~1,7 s, `num_turns: 0`, `total_tokens: 0`, pools `Gemini Models` et `Claude and GPT models`
//!     sur 5h et 7j).
//!   - **Claude (Anthropic)** : capture passive via StatusLine officiel (`bridget hook claude-statusline`,
//!     y compris sur sessions interactives) et `rate_limit_event` (`stream-json`).
//!   - **Cursor** : signalé honnêtement `not_exposed_by_provider` tant que la CLI ne publie pas de jauge libre.
//! - Déduplication automatique des comptes partagés entre serveurs (par `accountId` Codex et
//!   empreinte de pools Antigravity), cache privé `usage-quotas.json` (`0600`, TTL 5 min),
//!   historique léger `usage-history.jsonl` (`0600`, borné 48h / 512 points) pour la tendance,
//!   agrégation de capacité par fournisseur (style CPAMC) et alertes seuil bas (<= 15 %).

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
const AGY_PROBE_TIMEOUT_SECS: u64 = 12;
const PASSIVE_FLUSH_MIN_INTERVAL_SECS: i64 = 10;
const HISTORY_MIN_INTERVAL_SECS: i64 = 600;
const HISTORY_MAX_AGE_SECS: i64 = 48 * 3600;
const HISTORY_MAX_LINES: usize = 512;
const LOW_QUOTA_ALERT_THRESHOLD_PCT: u8 = 15;

static CACHE_LOCK: Mutex<i64> = Mutex::new(0);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct QuotaWindow {
    pub window: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pool: Option<String>,
    pub used_percent: Option<u8>,
    pub remaining_percent: Option<u8>,
    pub resets_at: Option<i64>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta_1h: Option<i16>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manual_resets_available: Option<u32>,
    pub observed_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ProviderSegment {
    pub label: String,
    pub remaining_percent: Option<u8>,
    pub tone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ProviderTotal {
    pub key: String,
    pub engine: String,
    pub label: String,
    pub credentials_count: usize,
    pub window_label: String,
    pub remaining_sum: u32,
    pub capacity_max: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_reset_at: Option<i64>,
    pub segments: Vec<ProviderSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct QuotaAlert {
    pub subscription_id: String,
    pub account_label: String,
    pub engine: String,
    pub window_label: String,
    pub remaining_percent: u8,
    pub resets_at: Option<i64>,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct UsageSummary {
    pub total_subscriptions: usize,
    pub with_live_quota: usize,
    pub not_exposed: usize,
    pub exhausted: usize,
    #[serde(default)]
    pub low_quota_alerts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct UsageReport {
    pub generated_at: i64,
    pub cache_ttl_secs: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_filter: Option<String>,
    #[serde(default)]
    pub provider_totals: Vec<ProviderTotal>,
    #[serde(default)]
    pub alerts: Vec<QuotaAlert>,
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
    #[serde(default)]
    manual_resets_available: Option<u32>,
    observed_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedWindow {
    window: String,
    #[serde(default)]
    pool: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HistoryPoint {
    ts: i64,
    sub_id: String,
    window: String,
    remaining_percent: u8,
}

struct CodexProbeResult {
    account_id: Option<String>,
    plan_type: Option<String>,
    windows: Vec<CachedWindow>,
    credits_balance: Option<String>,
    credits_unlimited: bool,
    manual_resets_available: Option<u32>,
}

struct AgyProbeResult {
    account_fingerprint: Option<String>,
    windows: Vec<CachedWindow>,
}

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn state_root_dir() -> Result<PathBuf, String> {
    if let Ok(home) = std::env::var("BRIDGET_HOME")
        && !home.trim().is_empty()
    {
        return Ok(PathBuf::from(home));
    }
    let home = std::env::var("HOME").map_err(|_| "variable HOME absente".to_string())?;
    Ok(PathBuf::from(home).join(".cache").join("bridget-core"))
}

fn cache_path() -> Result<PathBuf, String> {
    Ok(state_root_dir()?.join("usage-quotas.json"))
}

fn history_path() -> Result<PathBuf, String> {
    Ok(state_root_dir()?.join("usage-history.jsonl"))
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
    if lower == "gemini-5h" {
        return "Gemini · 5h".to_string();
    }
    if lower == "gemini-weekly" {
        return "Gemini · 7j".to_string();
    }
    if lower == "3p-5h" {
        return "Claude/GPT · 5h".to_string();
    }
    if lower == "3p-weekly" {
        return "Claude/GPT · 7j".to_string();
    }
    if lower == "five_hour" || lower.contains("five_hour") || lower == "primary/300m" || lower == "secondary/300m" {
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

/// Convertit un horodatage RFC3339/ISO-8601 UTC (`2026-11-30T20:03:03Z` ou avec fractions)
/// en epoch Unix (secondes). Pur Rust, sans dépendance externe.
pub(crate) fn parse_iso8601_utc(raw: &str) -> Option<i64> {
    let s = raw.trim();
    let core = s.strip_suffix('Z').or_else(|| s.strip_suffix('z'))?;
    let (date_part, time_part_full) = core.split_once('T')?;
    let time_part = time_part_full.split('.').next()?;
    let mut d_it = date_part.split('-');
    let year: i64 = d_it.next()?.parse().ok()?;
    let month: i64 = d_it.next()?.parse().ok()?;
    let day: i64 = d_it.next()?.parse().ok()?;
    if d_it.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let mut t_it = time_part.split(':');
    let hour: i64 = t_it.next()?.parse().ok()?;
    let min: i64 = t_it.next()?.parse().ok()?;
    let sec: i64 = t_it.next()?.parse().ok()?;
    if t_it.next().is_some() || hour > 23 || min > 59 || sec > 60 {
        return None;
    }
    // Algorithme civil-from-days / days-from-civil de Howard Hinnant (UTC exact).
    let y = year - if month <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let m_adj = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * m_adj + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some(days * 86_400 + hour * 3600 + min * 60 + sec)
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

fn load_history(path: &Path, now: i64) -> Vec<HistoryPoint> {
    let Ok(raw) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let cutoff = now.saturating_sub(HISTORY_MAX_AGE_SECS);
    raw.lines()
        .filter_map(|line| serde_json::from_str::<HistoryPoint>(line).ok())
        .filter(|pt| pt.ts >= cutoff && pt.ts <= now + 60)
        .collect()
}

fn update_history(path: &Path, subs: &[SubscriptionQuota], now: i64) -> Vec<HistoryPoint> {
    let mut pts = load_history(path, now);
    let mut changed = false;
    for sub in subs {
        if !matches!(sub.source_state.as_str(), "live_probe" | "observed") {
            continue;
        }
        for w in &sub.windows {
            let Some(rem) = w.remaining_percent else {
                continue;
            };
            let latest = pts
                .iter()
                .rev()
                .find(|p| p.sub_id == sub.id && p.window == w.window);
            let should_record = match latest {
                None => true,
                Some(prev) => {
                    prev.remaining_percent != rem
                        || now.saturating_sub(prev.ts) >= HISTORY_MIN_INTERVAL_SECS
                }
            };
            if should_record {
                pts.push(HistoryPoint {
                    ts: now,
                    sub_id: sub.id.clone(),
                    window: w.window.clone(),
                    remaining_percent: rem,
                });
                changed = true;
            }
        }
    }
    if pts.len() > HISTORY_MAX_LINES {
        let drop_n = pts.len() - HISTORY_MAX_LINES;
        pts.drain(0..drop_n);
        changed = true;
    }
    if changed
        && let Some(parent) = path.parent()
        && crate::environment::ensure_private_directory(parent).is_ok()
    {
        let tmp = parent.join(format!(
            ".usage-history-{}.tmp",
            uuid::Uuid::new_v4().simple()
        ));
        let mut buf = Vec::new();
        for p in &pts {
            if let Ok(line) = serde_json::to_string(p) {
                buf.extend_from_slice(line.as_bytes());
                buf.push(b'\n');
            }
        }
        if let Ok(mut f) = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&tmp)
            && f.write_all(&buf).is_ok()
            && f.sync_all().is_ok()
        {
            drop(f);
            if fs::rename(&tmp, path).is_err() {
                let _ = fs::remove_file(&tmp);
            }
        } else {
            let _ = fs::remove_file(&tmp);
        }
    }
    pts
}

fn compute_delta_1h(pts: &[HistoryPoint], sub_id: &str, window: &str, cur_rem: u8, now: i64) -> Option<i16> {
    let target_ts = now.saturating_sub(3600);
    let baseline = pts
        .iter()
        .filter(|p| p.sub_id == sub_id && p.window == window && p.ts <= now.saturating_sub(300))
        .min_by_key(|p| (p.ts - target_ts).abs())?;
    let delta = cur_rem as i16 - baseline.remaining_percent as i16;
    if delta == 0 { None } else { Some(delta) }
}

/// Enregistre passivement un fait `RateLimitFact` reçu par le daemon ou StatusLine dans
/// le cache privé `usage-quotas.json` (0 jeton, throttlé).
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
        manual_resets_available: None,
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
        // Ne pas écraser un pourcentage connu récent par un `None` issu d'un stream-json sans %
        let effective_used = fact.used_percent.or_else(|| {
            prev.and_then(|p| {
                if p.resets_at == fact.resets_at && p.resets_at.is_none_or(|at| at > now) {
                    p.used_percent
                } else {
                    None
                }
            })
        });
        if prev.is_none_or(|p| {
            p.used_percent != effective_used
                || p.status != fact.status
                || p.resets_at != fact.resets_at
        }) {
            meaningful_change = true;
        }
        entry.windows.insert(
            fact.window.clone(),
            CachedWindow {
                window: fact.window.clone(),
                pool: None,
                used_percent: effective_used,
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
    if matches!(engine, "codex" | "antigravity") {
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
    let manual_resets_available = result
        .get("rateLimitResetCredits")
        .and_then(|c| c.get("availableCount"))
        .and_then(Value::as_u64)
        .map(|n| n as u32);
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
            pool: None,
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
        manual_resets_available,
    })
}

/// Parse la sortie officielle `agy -p /usage --output-format json` (`num_turns: 0`, `total_tokens: 0`).
fn parse_antigravity_usage_response(value: &Value, now: i64) -> Option<AgyProbeResult> {
    let groups = value
        .get("command")
        .and_then(|c| c.get("data"))
        .and_then(|d| d.get("groups"))
        .and_then(Value::as_array)?;
    let mut windows = Vec::new();
    let mut fp_parts: Vec<String> = Vec::new();

    for group in groups {
        let group_name = group
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("Antigravity")
            .trim();
        let short_pool = if group_name.to_ascii_lowercase().contains("gemini") {
            "Gemini".to_string()
        } else if group_name.to_ascii_lowercase().contains("claude")
            || group_name.to_ascii_lowercase().contains("gpt")
        {
            "Claude/GPT".to_string()
        } else {
            group_name.to_string()
        };
        let Some(buckets) = group.get("buckets").and_then(Value::as_array) else {
            continue;
        };
        for bucket in buckets {
            let id = bucket
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .trim();
            if id.is_empty() {
                continue;
            }
            let frac = bucket.get("remaining_fraction").and_then(Value::as_f64);
            let rem_pct = frac.map(|f| (f * 100.0).round().clamp(0.0, 100.0) as u8);
            let used_pct = rem_pct.map(|r| 100u8.saturating_sub(r));
            let resets_at = bucket
                .get("reset_time")
                .and_then(Value::as_str)
                .and_then(parse_iso8601_utc);
            let status = if rem_pct == Some(0) {
                "rejected".to_string()
            } else {
                "available".to_string()
            };
            if id.contains("weekly")
                && let Some(rst) = resets_at
            {
                fp_parts.push(format!("{id}:{rst}"));
            }
            windows.push(CachedWindow {
                window: id.to_string(),
                pool: Some(short_pool.clone()),
                used_percent: used_pct,
                resets_at,
                status,
                observed_at: now,
            });
        }
    }
    if windows.is_empty() {
        return None;
    }
    fp_parts.sort();
    let account_fingerprint = if fp_parts.is_empty() {
        None
    } else {
        Some(format!("agy:{}", fp_parts.join("|")))
    };
    Some(AgyProbeResult {
        account_fingerprint,
        windows,
    })
}

fn probe_antigravity_usage(
    command: &str,
    profile_dir: Option<&str>,
    is_local: bool,
    machine_id: &str,
    now: i64,
) -> Result<AgyProbeResult, String> {
    let mut child = if is_local {
        let mut cmd = Command::new(command);
        cmd.args(["-p", "/usage", "--output-format", "json"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        if let Some(dir) = profile_dir.filter(|s| !s.trim().is_empty()) {
            cmd.env("ANTIGRAVITY_HOME", dir);
        }
        cmd.spawn()
            .map_err(|e| format!("lancement agy /usage local impossible : {e}"))?
    } else {
        let env_prefix = match profile_dir.filter(|s| !s.trim().is_empty()) {
            Some(dir) => format!("ANTIGRAVITY_HOME={} ", shell_single_quote(dir)),
            None => String::new(),
        };
        let remote_cmd = format!(
            "PATH=\"$HOME/.local/bin:/opt/homebrew/bin:/usr/local/bin:$PATH\" {env_prefix}{} -p /usage --output-format json",
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
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("sonde SSH agy sur {machine_id} impossible : {e}"))?
    };

    let Some(stdout) = child.stdout.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return Err("stdout agy indisponible".to_string());
    };
    let (tx, rx) = std::sync::mpsc::channel::<Result<String, String>>();
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        let mut out = String::new();
        match std::io::Read::read_to_string(&mut reader, &mut out) {
            Ok(_) => {
                let _ = tx.send(Ok(out));
            }
            Err(e) => {
                let _ = tx.send(Err(format!("lecture agy /usage : {e}")));
            }
        }
    });
    let raw = match rx.recv_timeout(Duration::from_secs(AGY_PROBE_TIMEOUT_SECS)) {
        Ok(Ok(text)) => text,
        Ok(Err(e)) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(e);
        }
        Err(_) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("délai sonde agy dépassé ({AGY_PROBE_TIMEOUT_SECS}s)"));
        }
    };
    let _ = child.wait();
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("machine hors ligne ou agy /usage vide".to_string());
    }
    // Tolérer une éventuelle ligne de log avant l'objet JSON.
    let json_slice = trimmed
        .find('{')
        .map(|idx| &trimmed[idx..])
        .unwrap_or(trimmed);
    let val: Value = serde_json::from_str(json_slice)
        .map_err(|_| "sortie agy /usage non JSON".to_string())?;
    parse_antigravity_usage_response(&val, now)
        .ok_or_else(|| "réponse agy /usage sans groupes de quota".to_string())
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
                pool: None,
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
        if w.window == "unknown" && w.status != "rejected" && w.used_percent.is_none() {
            continue;
        }
        let expired = w.resets_at.is_some_and(|at| at <= now);
        if expired {
            out.push(QuotaWindow {
                window: w.window.clone(),
                label: format_window_label(&w.window),
                pool: w.pool.clone(),
                used_percent: None,
                remaining_percent: None,
                resets_at: w.resets_at,
                status: "reset_elapsed".to_string(),
                delta_1h: None,
            });
        } else {
            any_live = true;
            let remaining = w.used_percent.map(|u| 100u8.saturating_sub(u.min(100)));
            out.push(QuotaWindow {
                window: w.window.clone(),
                label: format_window_label(&w.window),
                pool: w.pool.clone(),
                used_percent: w.used_percent,
                remaining_percent: remaining,
                resets_at: w.resets_at,
                status: w.status.clone(),
                delta_1h: None,
            });
        }
    }
    out.sort_by(|a, b| a.label.cmp(&b.label).then_with(|| a.window.cmp(&b.window)));
    (out, any_live)
}

/// Fusionne les abonnements partageant le même `account_id` (ex. même compte Codex Pro
/// ou même compte Antigravity connecté sur plusieurs machines).
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
                if sub.manual_resets_available.is_some() {
                    existing.manual_resets_available = sub.manual_resets_available;
                }
                existing.credits_unlimited |= sub.credits_unlimited;
            }
            if existing.machines.len() > 1 {
                let plan_title = existing
                    .plan_type
                    .as_deref()
                    .map(|p| {
                        let mut chars = p.chars();
                        match chars.next() {
                            Some(first) => format!(
                                " {}",
                                first.to_uppercase().collect::<String>() + chars.as_str()
                            ),
                            None => String::new(),
                        }
                    })
                    .unwrap_or_default();
                let base = match existing.engine.as_str() {
                    "codex" => format!("Codex{plan_title}"),
                    "antigravity" => "Antigravity".to_string(),
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

fn tone_for_remaining(rem: u8) -> String {
    if rem <= 10 {
        "danger".to_string()
    } else if rem <= 30 {
        "warn".to_string()
    } else {
        "ok".to_string()
    }
}

/// Construit le bandeau de capacité agrégée par fournisseur (façon CPAMC).
fn build_provider_totals(subs: &[SubscriptionQuota]) -> Vec<ProviderTotal> {
    let mut totals = Vec::new();

    // 1. Codex : somme des fenêtres hebdomadaires (ou principales) par compte.
    let codex_live: Vec<_> = subs
        .iter()
        .filter(|s| s.engine == "codex" && s.source_state == "live_probe" && !s.windows.is_empty())
        .collect();
    if !codex_live.is_empty() {
        let mut sum = 0u32;
        let mut next_reset: Option<i64> = None;
        let mut segments = Vec::new();
        for s in &codex_live {
            let primary = s
                .windows
                .iter()
                .find(|w| w.label == "7j")
                .or_else(|| s.windows.first());
            let rem = primary.and_then(|w| w.remaining_percent).unwrap_or(0);
            sum += rem as u32;
            if let Some(rst) = primary.and_then(|w| w.resets_at) {
                next_reset = Some(next_reset.map_or(rst, |cur| cur.min(rst)));
            }
            segments.push(ProviderSegment {
                label: s.account_label.clone(),
                remaining_percent: Some(rem),
                tone: tone_for_remaining(rem),
            });
        }
        totals.push(ProviderTotal {
            key: "codex".to_string(),
            engine: "codex".to_string(),
            label: "Codex".to_string(),
            credentials_count: codex_live.len(),
            window_label: "Limite 7j".to_string(),
            remaining_sum: sum,
            capacity_max: (codex_live.len() as u32) * 100,
            next_reset_at: next_reset,
            segments,
        });
    }

    // 2. Antigravity : deux pools distincts (Gemini et Claude/GPT) en 7j.
    let agy_live: Vec<_> = subs
        .iter()
        .filter(|s| {
            s.engine == "antigravity" && s.source_state == "live_probe" && !s.windows.is_empty()
        })
        .collect();
    if !agy_live.is_empty() {
        for (pool_key, pool_title, win_id) in [
            ("antigravity-gemini", "Antigravity · Gemini", "gemini-weekly"),
            ("antigravity-3p", "Antigravity · Claude/GPT", "3p-weekly"),
        ] {
            let mut sum = 0u32;
            let mut count = 0usize;
            let mut next_reset: Option<i64> = None;
            let mut segments = Vec::new();
            for s in &agy_live {
                if let Some(w) = s.windows.iter().find(|w| w.window == win_id)
                    && let Some(rem) = w.remaining_percent
                {
                    count += 1;
                    sum += rem as u32;
                    if let Some(rst) = w.resets_at {
                        next_reset = Some(next_reset.map_or(rst, |cur| cur.min(rst)));
                    }
                    segments.push(ProviderSegment {
                        label: s.account_label.clone(),
                        remaining_percent: Some(rem),
                        tone: tone_for_remaining(rem),
                    });
                }
            }
            if count > 0 {
                totals.push(ProviderTotal {
                    key: pool_key.to_string(),
                    engine: "antigravity".to_string(),
                    label: pool_title.to_string(),
                    credentials_count: count,
                    window_label: "Limite 7j".to_string(),
                    remaining_sum: sum,
                    capacity_max: (count as u32) * 100,
                    next_reset_at: next_reset,
                    segments,
                });
            }
        }
    }

    // 3. Claude : si au moins un abonnement a un pourcentage réel (via StatusLine / utilization).
    let claude_pct: Vec<_> = subs
        .iter()
        .filter(|s| {
            s.engine == "claude"
                && s.source_state == "observed"
                && s.windows.iter().any(|w| w.remaining_percent.is_some())
        })
        .collect();
    if !claude_pct.is_empty() {
        let mut sum = 0u32;
        let mut next_reset: Option<i64> = None;
        let mut segments = Vec::new();
        for s in &claude_pct {
            let primary = s
                .windows
                .iter()
                .find(|w| w.label == "7j" && w.remaining_percent.is_some())
                .or_else(|| s.windows.iter().find(|w| w.remaining_percent.is_some()));
            let rem = primary.and_then(|w| w.remaining_percent).unwrap_or(0);
            sum += rem as u32;
            if let Some(rst) = primary.and_then(|w| w.resets_at) {
                next_reset = Some(next_reset.map_or(rst, |cur| cur.min(rst)));
            }
            segments.push(ProviderSegment {
                label: s.account_label.clone(),
                remaining_percent: Some(rem),
                tone: tone_for_remaining(rem),
            });
        }
        totals.push(ProviderTotal {
            key: "claude".to_string(),
            engine: "claude".to_string(),
            label: "Claude".to_string(),
            credentials_count: claude_pct.len(),
            window_label: "Limite 7j".to_string(),
            remaining_sum: sum,
            capacity_max: (claude_pct.len() as u32) * 100,
            next_reset_at: next_reset,
            segments,
        });
    }

    totals
}

fn build_low_quota_alerts(subs: &[SubscriptionQuota]) -> Vec<QuotaAlert> {
    let mut alerts = Vec::new();
    for s in subs {
        if !matches!(s.source_state.as_str(), "live_probe" | "observed") {
            continue;
        }
        for w in &s.windows {
            if let Some(rem) = w.remaining_percent
                && rem <= LOW_QUOTA_ALERT_THRESHOLD_PCT
            {
                alerts.push(QuotaAlert {
                    subscription_id: s.id.clone(),
                    account_label: s.account_label.clone(),
                    engine: s.engine.clone(),
                    window_label: w.label.clone(),
                    remaining_percent: rem,
                    resets_at: w.resets_at,
                    severity: if rem == 0 {
                        "exhausted".to_string()
                    } else {
                        "low".to_string()
                    },
                });
            }
        }
    }
    alerts
}

/// Collecte les quotas restants sur la flotte sans consommer aucun jeton LLM.
pub(crate) fn collect_usage(
    provider_filter_raw: Option<&str>,
    force_refresh: bool,
    registry_override: Option<&Path>,
) -> Result<UsageReport, String> {
    collect_usage_targeted(provider_filter_raw, None, force_refresh, registry_override)
}

pub(crate) fn collect_usage_targeted(
    provider_filter_raw: Option<&str>,
    target_subscription_id: Option<&str>,
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

    let live_daemon = collect_live_daemon_rate_limits(now);

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

    // Sondes gratuites en parallèle : Codex (account/rateLimits/read) et Antigravity (agy -p /usage).
    let mut cache_dirty = false;
    let mut codex_jobs: Vec<(String, String, Option<String>, bool, String)> = Vec::new();
    let mut agy_jobs: Vec<(String, String, Option<String>, bool, String)> = Vec::new();

    for group in groups.values() {
        let ckey = cache_entry_key(
            &group.engine,
            &group.machine_id,
            group.profile_dir.as_deref(),
            group.connection_ids.first().map(String::as_str),
            group.agent_types.first().map(String::as_str).unwrap_or(&group.engine),
        );
        let cached_acc_id = cache.entries.get(&ckey).and_then(|e| e.account_id.as_deref());
        let targeted = match target_subscription_id {
            None => force_refresh,
            Some(tid) => {
                force_refresh
                    && (tid == group.id
                        || cached_acc_id.is_some_and(|a| tid == format!("{}:account:{a}", group.engine)))
            }
        };

        if group.engine == "codex" {
            let fresh = cache.entries.get(&ckey).is_some_and(|e| {
                !targeted
                    && e.source == "codex_rpc_probe"
                    && e.account_id.is_some()
                    && e.observed_at > 0
                    && now.saturating_sub(e.observed_at) <= USAGE_CACHE_TTL_SECS as i64
            });
            if !fresh {
                codex_jobs.push((
                    group.id.clone(),
                    group.command.clone(),
                    group.profile_dir.clone(),
                    group.is_local,
                    group.machine_id.clone(),
                ));
            }
        } else if group.engine == "antigravity" {
            let fresh = cache.entries.get(&ckey).is_some_and(|e| {
                !targeted
                    && e.source == "antigravity_cli_usage"
                    && !e.windows.is_empty()
                    && e.observed_at > 0
                    && now.saturating_sub(e.observed_at) <= USAGE_CACHE_TTL_SECS as i64
            });
            if !fresh {
                agy_jobs.push((
                    group.id.clone(),
                    group.command.clone(),
                    group.profile_dir.clone(),
                    group.is_local,
                    group.machine_id.clone(),
                ));
            }
        }
    }

    let codex_handles: Vec<_> = codex_jobs
        .into_iter()
        .map(|(gid, cmd, prof, is_local, mid)| {
            std::thread::spawn(move || {
                let res = probe_codex_app_server(&cmd, prof.as_deref(), is_local, &mid, now);
                (gid, res)
            })
        })
        .collect();
    let agy_handles: Vec<_> = agy_jobs
        .into_iter()
        .map(|(gid, cmd, prof, is_local, mid)| {
            std::thread::spawn(move || {
                let res = probe_antigravity_usage(&cmd, prof.as_deref(), is_local, &mid, now);
                (gid, res)
            })
        })
        .collect();

    let mut codex_results: BTreeMap<String, Result<CodexProbeResult, String>> = BTreeMap::new();
    for h in codex_handles {
        if let Ok((gid, res)) = h.join() {
            codex_results.insert(gid, res);
        }
    }
    let mut agy_results: BTreeMap<String, Result<AgyProbeResult, String>> = BTreeMap::new();
    for h in agy_handles {
        if let Ok((gid, res)) = h.join() {
            agy_results.insert(gid, res);
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
                if let Some(res) = codex_results.remove(&group.id) {
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
                                    manual_resets_available: pr.manual_resets_available,
                                    observed_at: now,
                                },
                            );
                            cache_dirty = true;
                        }
                        Err(e) => probe_err = Some(e),
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
                    manual_resets_available: cached.and_then(|e| e.manual_resets_available),
                    observed_at: cached.map(|e| e.observed_at),
                    detail,
                });
            }
            "antigravity" => {
                let mut probe_err: Option<String> = None;
                if let Some(res) = agy_results.remove(&group.id) {
                    match res {
                        Ok(pr) => {
                            let mut map = BTreeMap::new();
                            for w in pr.windows {
                                map.insert(w.window.clone(), w);
                            }
                            cache.entries.insert(
                                ckey.clone(),
                                CachedQuotaEntry {
                                    engine: "antigravity".to_string(),
                                    machine_id: group.machine_id.clone(),
                                    agent_type: group
                                        .agent_types
                                        .first()
                                        .cloned()
                                        .unwrap_or_default(),
                                    connection_id: group.connection_ids.first().cloned(),
                                    profile_dir: group.profile_dir.clone(),
                                    account_id: pr.account_fingerprint,
                                    plan_type: None,
                                    source: "antigravity_cli_usage".to_string(),
                                    windows: map,
                                    credits_balance: None,
                                    credits_unlimited: false,
                                    manual_resets_available: None,
                                    observed_at: now,
                                },
                            );
                            cache_dirty = true;
                        }
                        Err(e) => probe_err = Some(e),
                    }
                }
                let cached = cache.entries.get(&ckey);
                let (windows, any_live) = cached
                    .map(|e| materialize_windows(&e.windows, now))
                    .unwrap_or_else(|| (Vec::new(), false));
                let (source_state, source_label, detail) = if any_live {
                    (
                        "live_probe".to_string(),
                        "Sonde officielle agy /usage (0 jeton)".to_string(),
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
                            "Aucune donnée de quota reçue via agy -p /usage.".to_string()
                        })),
                    )
                };
                subscriptions.push(SubscriptionQuota {
                    id: group.id,
                    engine: group.engine,
                    provider_label: provider_display_name("antigravity").to_string(),
                    account_label: group.account_label,
                    machine_id: group.machine_id.clone(),
                    machines: vec![group.machine_id],
                    account_id: cached.and_then(|e| e.account_id.clone()),
                    connection_ids: group.connection_ids,
                    plan_type: None,
                    source_state,
                    source_label,
                    windows,
                    credits_balance: None,
                    credits_unlimited: false,
                    manual_resets_available: None,
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
                            let prefer_new = match merged_windows.get(wk) {
                                None => true,
                                Some(cur) => {
                                    // Privilégier une fenêtre avec pourcentage réel si même reset ou plus récente.
                                    (wv.used_percent.is_some() && cur.used_percent.is_none())
                                        || (cur.observed_at < wv.observed_at
                                            && !(cur.used_percent.is_some()
                                                && wv.used_percent.is_none()
                                                && cur.resets_at == wv.resets_at))
                                }
                            };
                            if prefer_new {
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
                            "Claude publie ses jauges 5h/7j pendant ses sessions actives (StatusLine / stream-json). Aucun jeton n'est dépensé."
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
                    manual_resets_available: None,
                    observed_at: latest_obs,
                    detail,
                });
            }
            "cursor" => {
                subscriptions.push(SubscriptionQuota {
                    id: group.id,
                    engine: "cursor".to_string(),
                    provider_label: provider_display_name("cursor").to_string(),
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
                    manual_resets_available: None,
                    observed_at: None,
                    detail: Some(
                        "Cursor ne publie pas de compteur de quota restant via sa CLI sans consommer de requête."
                            .to_string(),
                    ),
                });
            }
            _ => {}
        }
    }

    if cache_dirty && let Some(path) = cpath.as_deref() {
        cache.version = 1;
        let _ = save_cache_to(path, &cache);
    }

    subscriptions = deduplicate_shared_accounts(subscriptions);

    // Enregistrer et annoter l'historique léger (tendance sur 1h) à 0 jeton.
    if let Ok(hpath) = history_path() {
        let pts = update_history(&hpath, &subscriptions, now);
        for s in &mut subscriptions {
            for w in &mut s.windows {
                if let Some(rem) = w.remaining_percent {
                    w.delta_1h = compute_delta_1h(&pts, &s.id, &w.window, rem, now);
                }
            }
        }
    }

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

    let provider_totals = build_provider_totals(&subscriptions);
    let alerts = build_low_quota_alerts(&subscriptions);

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
    let low_quota_alerts = alerts.len();

    Ok(UsageReport {
        generated_at: now,
        cache_ttl_secs: USAGE_CACHE_TTL_SECS,
        provider_filter: filter.map(str::to_string),
        provider_totals,
        alerts,
        subscriptions,
        summary: UsageSummary {
            total_subscriptions,
            with_live_quota,
            not_exposed,
            exhausted,
            low_quota_alerts,
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
                    "Jauges 5h/7j mises à jour passivement dès la prochaine session Claude active (StatusLine / stream-json, 0 jeton)."
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
            (BTreeSet::new(), 0, s.provider_label.clone(), reason)
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
        .map(
            |((engine, source_state), (machines, count, provider_label, reason))| {
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
            },
        )
        .collect();

    (active, compact)
}

/// Rendu terminal propre avec capacité agrégée et jauges Unicode de quota restant.
pub(crate) fn render_terminal(report: &UsageReport) -> String {
    let mut out = String::new();
    let title = match &report.provider_filter {
        Some(p) => format!("Quotas restants Bridget · {}", provider_display_name(p)),
        None => "Quotas restants Bridget · Abonnements".to_string(),
    };
    out.push_str(&format!("{title}\n"));
    out.push_str(&format!(
        "{} abonnement(s) actif(s) · 0 jeton LLM consommé\n",
        report.summary.with_live_quota,
    ));
    if !report.provider_totals.is_empty() {
        let agg = report
            .provider_totals
            .iter()
            .map(|t| format!("{} {}%/{}%", t.label, t.remaining_sum, t.capacity_max))
            .collect::<Vec<_>>()
            .join(" · ");
        out.push_str(&format!("Capacité globale : {agg}\n"));
    }
    out.push('\n');

    let (active, compact) = split_active_and_compact(&report.subscriptions);
    if active.is_empty() && compact.is_empty() {
        out.push_str("Aucun abonnement trouvé pour ce filtre.\n");
        return out;
    }
    for sub in &active {
        let resets_note = sub
            .manual_resets_available
            .map(|n| format!(" · resets manuels : {n}"))
            .unwrap_or_default();
        out.push_str(&format!(
            "• {} — {} ({}){}\n",
            sub.account_label,
            sub.source_label,
            format_age_fr(sub.observed_at, report.generated_at),
            resets_note
        ));
        for w in &sub.windows {
            if w.status == "reset_elapsed" {
                out.push_str(&format!(
                    "    {:>15}  [────────────] fenêtre réinitialisée\n",
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
                let delta_part = w
                    .delta_1h
                    .map(|d| format!(" ({:+}%/1h)", d))
                    .unwrap_or_default();
                out.push_str(&format!(
                    "    {:>15}  [{}] {:>3}% restant ({}% utilisé){}{}\n",
                    w.label,
                    render_unicode_bar(rem),
                    rem,
                    used,
                    delta_part,
                    rst_part
                ));
            } else if w.status == "rejected" {
                let rst = format_reset_delta_fr(w.resets_at, report.generated_at);
                out.push_str(&format!(
                    "    {:>15}  [░░░░░░░░░░░░] épuisé · {rst}\n",
                    w.label
                ));
            } else {
                let rst = format_reset_delta_fr(w.resets_at, report.generated_at);
                let rst_part = if rst.is_empty() {
                    String::new()
                } else {
                    format!(" · {rst}")
                };
                out.push_str(&format!(
                    "    {:>15}  disponible (% non publié par ce flux){rst_part}\n",
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

/// Rendu HTML autonome avec bandeau agrégé (façon CPAMC) et jauges propres.
pub(crate) fn render_html(report: &UsageReport) -> String {
    let (active, compact) = split_active_and_compact(&report.subscriptions);

    // 1. Bandeau de capacité agrégée par fournisseur.
    let totals_section = if report.provider_totals.is_empty() {
        String::new()
    } else {
        let mut items = String::new();
        for t in &report.provider_totals {
            let rst = format_reset_delta_fr(t.next_reset_at, report.generated_at);
            let segs = t
                .segments
                .iter()
                .map(|seg| {
                    let pct = seg.remaining_percent.unwrap_or(0);
                    format!(
                        r#"<div class="seg-track" title="{}: {}%"><div class="seg-fill {}" style="width:{}%"></div></div>"#,
                        html_esc(&seg.label),
                        pct,
                        html_esc(&seg.tone),
                        pct
                    )
                })
                .collect::<Vec<_>>()
                .join("");
            items.push_str(&format!(
                r#"<div class="tot-card"><div class="tot-top"><span class="tot-name">{}</span><span class="tot-cred">{} compte{}</span></div><div class="tot-val"><strong>{}%</strong> <span class="muted">sur {}%</span></div><div class="seg-row">{}</div><div class="tot-foot"><span>{}</span><span>{}</span></div></div>"#,
                html_esc(&t.label),
                t.credentials_count,
                if t.credentials_count > 1 { "s" } else { "" },
                t.remaining_sum,
                t.capacity_max,
                segs,
                html_esc(&t.window_label),
                html_esc(&rst)
            ));
        }
        format!(r#"<div class="tot-grid">{items}</div>"#)
    };

    // 2. Alerte seuil bas / saturation.
    let alert_section = if report.alerts.is_empty() {
        String::new()
    } else {
        let txt = report
            .alerts
            .iter()
            .map(|a| {
                let rst = format_reset_delta_fr(a.resets_at, report.generated_at);
                let suffix = if rst.is_empty() {
                    String::new()
                } else {
                    format!(" ({rst})")
                };
                format!(
                    "{} · {} : {}% restant{}",
                    a.account_label, a.window_label, a.remaining_percent, suffix
                )
            })
            .collect::<Vec<_>>()
            .join(" — ");
        format!(
            r#"<div class="alert-strip"><strong>▲ Seuil bas</strong><span>{}</span></div>"#,
            html_esc(&txt)
        )
    };

    // 3. Cartes détaillées des abonnements actifs.
    let mut cards = String::new();
    for sub in &active {
        let plan_badge = sub
            .plan_type
            .as_deref()
            .filter(|p| {
                !sub.account_label
                    .to_ascii_lowercase()
                    .contains(&p.to_ascii_lowercase())
            })
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
                    r#"<div class="win"><div class="win-head"><span class="win-label">{}</span><span class="win-val muted">Réinitialisée</span></div><div class="track"><div class="fill muted" style="width:0%"></div></div></div>"#,
                    html_esc(&w.label)
                ));
                continue;
            }
            let rst = format_reset_delta_fr(w.resets_at, report.generated_at);
            let rst_html = if rst.is_empty() {
                String::new()
            } else {
                format!(r#"<span class="win-rst">{}</span>"#, html_esc(&rst))
            };
            if w.used_percent.is_none() && w.status != "rejected" {
                body.push_str(&format!(
                    r#"<div class="win"><div class="win-head"><span class="win-label">{}</span><span class="win-val ok"><strong>Disponible</strong> <span class="muted">(% non publié)</span></span></div><div class="win-foot">{}</div></div>"#,
                    html_esc(&w.label),
                    rst_html
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
            let delta_html = w
                .delta_1h
                .map(|d| format!(r#" <span class="delta">({:+}%/1h)</span>"#, d))
                .unwrap_or_default();
            body.push_str(&format!(
                r#"<div class="win"><div class="win-head"><span class="win-label">{}</span><span class="win-val {}"><strong>{}%</strong> restant <span class="muted">({}% utilisé)</span>{}</span></div><div class="track"><div class="fill {}" style="width:{}%"></div></div><div class="win-foot">{}</div></div>"#,
                html_esc(&w.label),
                tone,
                rem,
                used,
                delta_html,
                tone,
                rem,
                rst_html
            ));
        }
        let resets_html = sub
            .manual_resets_available
            .map(|n| {
                format!(
                    r#"<div class="card-extra">Resets manuels Codex : <strong>{}</strong> disponible{}</div>"#,
                    n,
                    if n > 1 { "s" } else { "" }
                )
            })
            .unwrap_or_default();
        cards.push_str(&format!(
            r#"<section class="card"><div class="card-head"><div class="card-meta"><div class="card-title"><span>{}</span>{}</div><div class="card-sub">{} · {} · {}</div></div><span class="badge {}">{}</span></div>{}{}</section>"#,
            html_esc(&sub.account_label),
            plan_badge,
            html_esc(&sub.provider_label),
            html_esc(&machines_txt),
            html_esc(&age),
            short_status.0,
            short_status.1,
            body,
            resets_html
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
        format!("{totals_section}{alert_section}{active_section}{compact_section}")
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
.tot-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(195px, 1fr)); gap: 10px; }}
.tot-card {{ background: var(--card, rgba(127,127,127,0.08)); border: 1px solid var(--border, rgba(127,127,127,0.24)); border-radius: var(--radius, 10px); padding: 11px 13px; display: flex; flex-direction: column; gap: 7px; }}
.tot-top {{ display: flex; justify-content: space-between; align-items: baseline; gap: 6px; font-size: 12px; }}
.tot-name {{ font-weight: 600; }}
.tot-cred {{ font-size: 11px; color: var(--muted-foreground, var(--muted, #9ca3af)); }}
.tot-val {{ font-size: 18px; line-height: 1.1; }}
.tot-val .muted {{ font-size: 12px; font-weight: 400; }}
.seg-row {{ display: flex; gap: 4px; }}
.seg-track {{ flex: 1; height: 6px; border-radius: 999px; background: var(--secondary, rgba(127,127,127,0.2)); overflow: hidden; }}
.seg-fill {{ height: 100%; border-radius: 999px; }}
.seg-fill.ok {{ background: var(--success, #22c55e); }}
.seg-fill.warn {{ background: var(--warning, #f59e0b); }}
.seg-fill.danger {{ background: var(--destructive, #ef4444); }}
.tot-foot {{ display: flex; justify-content: space-between; gap: 6px; font-size: 11px; color: var(--muted-foreground, var(--muted, #9ca3af)); }}
.alert-strip {{ display: flex; gap: 8px; align-items: baseline; flex-wrap: wrap; padding: 8px 12px; border-radius: var(--radius, 8px); border: 1px solid color-mix(in srgb, var(--warning, #f59e0b) 40%, transparent); background: color-mix(in srgb, var(--warning, #f59e0b) 10%, transparent); font-size: 12px; }}
.alert-strip strong {{ color: var(--warning, #f59e0b); }}
.grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(290px, 1fr)); gap: 12px; }}
.card {{ background: var(--card, rgba(127,127,127,0.08)); border: 1px solid var(--border, rgba(127,127,127,0.25)); border-radius: var(--radius, 10px); padding: 14px 16px; display: flex; flex-direction: column; gap: 10px; }}
.card-head {{ display: flex; justify-content: space-between; align-items: flex-start; gap: 10px; }}
.card-meta {{ min-width: 0; flex: 1; }}
.card-title {{ font-weight: 600; font-size: 13.5px; display: flex; align-items: center; gap: 6px; flex-wrap: wrap; line-height: 1.3; }}
.card-sub {{ font-size: 11.5px; color: var(--muted-foreground, var(--muted, #9ca3af)); margin-top: 3px; }}
.card-extra {{ font-size: 11.5px; color: var(--muted-foreground, var(--muted, #9ca3af)); border-top: 1px solid var(--border, rgba(127,127,127,0.18)); padding-top: 7px; }}
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
.delta {{ font-size: 11px; color: var(--muted-foreground, var(--muted, #9ca3af)); }}
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
</style></head><body><div class="wrap"><div class="hdr"><h1>{}</h1><span class="hdr-meta">{} abonnement(s) actif(s) · 0 jeton LLM consommé</span></div>{}</div></body></html>"#,
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
                    "bridget usage [fournisseur] [--refresh] [--json] [--html]\n  fournisseur : claude/anthropic | codex/openai | antigravity/agy/google | cursor | all\n  --refresh   : relance les sondes gratuites (Codex + Antigravity, 0 jeton LLM) sans attendre le TTL de 5 min\n  --json      : sortie JSON structurée\n  --html      : sortie HTML autonome avec jauges de quotas restants"
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
    usage_api_payload_targeted(provider, None, refresh, registry_override)
}

pub(crate) fn usage_api_payload_targeted(
    provider: Option<&str>,
    subscription_id: Option<&str>,
    refresh: bool,
    registry_override: Option<&Path>,
) -> Result<Value, String> {
    let report = collect_usage_targeted(provider, subscription_id, refresh, registry_override)?;
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
    fn spec345_parse_antigravity_usage_json_and_iso8601_utc() {
        let now = parse_iso8601_utc("2026-11-24T14:48:25Z").unwrap();
        assert_eq!(parse_iso8601_utc("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_iso8601_utc("2026-11-30T20:03:03Z"), Some(1_796_068_983));

        let payload = json!({
            "status": "SUCCESS",
            "num_turns": 0,
            "usage": { "total_tokens": 0 },
            "command": {
                "name": "usage",
                "data": {
                    "groups": [
                        {
                            "name": "Gemini Models",
                            "buckets": [
                                {"id": "gemini-weekly", "window": "weekly", "remaining_fraction": 0.8434, "reset_time": "2026-11-30T20:03:03Z"},
                                {"id": "gemini-5h", "window": "5h", "remaining_fraction": 0.8302, "reset_time": "2026-11-24T16:03:03Z"}
                            ]
                        },
                        {
                            "name": "Claude and GPT models",
                            "buckets": [
                                {"id": "3p-weekly", "window": "weekly", "remaining_fraction": 0.4555, "reset_time": "2026-11-28T13:39:43Z"},
                                {"id": "3p-5h", "window": "5h", "remaining_fraction": 1.0, "reset_time": "2026-11-24T19:48:25Z"}
                            ]
                        }
                    ]
                }
            }
        });
        let parsed = parse_antigravity_usage_response(&payload, now).unwrap();
        assert!(parsed.account_fingerprint.is_some());
        assert_eq!(parsed.windows.len(), 4);

        let mut map = BTreeMap::new();
        for w in parsed.windows {
            map.insert(w.window.clone(), w);
        }
        let (mat, any_live) = materialize_windows(&map, now);
        assert!(any_live);
        assert_eq!(mat.len(), 4);
        let gem_w = mat.iter().find(|w| w.window == "gemini-weekly").unwrap();
        assert_eq!(gem_w.remaining_percent, Some(84));
        assert_eq!(gem_w.label, "Gemini · 7j");
        let tp_w = mat.iter().find(|w| w.window == "3p-weekly").unwrap();
        assert_eq!(tp_w.remaining_percent, Some(46));
        assert_eq!(tp_w.label, "Claude/GPT · 7j");
    }

    #[test]
    fn spec342_parse_codex_rate_limits_dedup_by_account_and_render_compact_html() {
        let now = 1_795_500_000;
        let payload = json!({
            "id": 2,
            "result": {
                "accountId": "82ae4828-611c-4a88-9020-dac5ee4a001c",
                "rateLimitResetCredits": { "availableCount": 2, "credits": [] },
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
        assert_eq!(pr.manual_resets_available, Some(2));
        assert_eq!(pr.plan_type.as_deref(), Some("pro"));
        assert_eq!(pr.windows.len(), 2);

        let mut map = BTreeMap::new();
        for w in pr.windows {
            map.insert(w.window.clone(), w);
        }
        let (mat, any_live) = materialize_windows(&map, now);
        assert!(any_live);

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
                manual_resets_available: pr.manual_resets_available,
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
                manual_resets_available: pr.manual_resets_available,
                observed_at: Some(now - 5),
                detail: None,
            },
        ]);

        assert_eq!(deduped.len(), 1);
        let totals = build_provider_totals(&deduped);
        assert_eq!(totals.len(), 1);
        assert_eq!(totals[0].remaining_sum, 56);
        assert_eq!(totals[0].capacity_max, 100);

        let report = UsageReport {
            generated_at: now,
            cache_ttl_secs: USAGE_CACHE_TTL_SECS,
            provider_filter: None,
            provider_totals: totals,
            alerts: build_low_quota_alerts(&deduped),
            subscriptions: deduped,
            summary: UsageSummary {
                total_subscriptions: 1,
                with_live_quota: 1,
                not_exposed: 0,
                exhausted: 0,
                low_quota_alerts: 0,
            },
        };
        let html = render_html(&report);
        assert!(html.contains("tot-grid"));
        assert!(html.contains("Resets manuels Codex"));
        assert!(html.contains("56%"));
    }

    #[test]
    fn spec342_expired_window_is_marked_reset_elapsed_never_stale_percent() {
        let now = 1_795_500_000;
        let mut map = BTreeMap::new();
        map.insert(
            "five_hour".to_string(),
            CachedWindow {
                window: "five_hour".to_string(),
                pool: None,
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

    #[test]
    fn spec344_allowed_without_percent_never_fakes_100_percent_and_drops_unknown() {
        let now = 1_795_500_000;
        let mut map = BTreeMap::new();
        map.insert(
            "seven_day".to_string(),
            CachedWindow {
                window: "seven_day".to_string(),
                pool: None,
                used_percent: None,
                resets_at: Some(now + 3600 * 24),
                status: "allowed".to_string(),
                observed_at: now - 120,
            },
        );
        map.insert(
            "unknown".to_string(),
            CachedWindow {
                window: "unknown".to_string(),
                pool: None,
                used_percent: None,
                resets_at: None,
                status: "allowed".to_string(),
                observed_at: now - 120,
            },
        );
        let (mat, any_live) = materialize_windows(&map, now);
        assert!(any_live);
        assert_eq!(mat.len(), 1);
        assert_eq!(mat[0].label, "7j");
        assert_eq!(mat[0].remaining_percent, None);

        let report = UsageReport {
            generated_at: now,
            cache_ttl_secs: USAGE_CACHE_TTL_SECS,
            provider_filter: None,
            provider_totals: vec![],
            alerts: vec![],
            subscriptions: vec![SubscriptionQuota {
                id: "claude:cartae:default".into(),
                engine: "claude".into(),
                provider_label: provider_display_name("claude").into(),
                account_label: "Cartae Claude natif".into(),
                machine_id: "cartae".into(),
                machines: vec!["cartae".into()],
                account_id: None,
                connection_ids: vec!["claude-cartae".into()],
                plan_type: None,
                source_state: "observed".into(),
                source_label: "Observé passivement".into(),
                windows: mat,
                credits_balance: None,
                credits_unlimited: false,
                manual_resets_available: None,
                observed_at: Some(now - 120),
                detail: None,
            }],
            summary: UsageSummary {
                total_subscriptions: 1,
                with_live_quota: 1,
                not_exposed: 0,
                exhausted: 0,
                low_quota_alerts: 0,
            },
        };
        let html = render_html(&report);
        assert!(html.contains("Disponible"));
        assert!(html.contains("% non publié"));
        assert!(!html.contains("100%</strong>"));
        assert!(!html.contains("0% utilisé"));
        assert!(!html.contains("unknown"));
    }
}
