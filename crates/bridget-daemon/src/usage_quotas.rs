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
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(crate) const USAGE_CACHE_TTL_SECS: u64 = 300;
const ROUTING_POSITIVE_CACHE_MAX_AGE_SECS: i64 = 1800;
const ROUTING_UNKNOWN_EXHAUSTION_TTL_SECS: i64 = 300;
const CODEX_PROBE_TIMEOUT_SECS: u64 = 6;
const AGY_PROBE_TIMEOUT_SECS: u64 = 12;
const CLAUDE_AUTH_PROBE_TIMEOUT_SECS: u64 = 5;
const CURSOR_PROBE_TIMEOUT_SECS: u64 = 6;
const PASSIVE_FLUSH_MIN_INTERVAL_SECS: i64 = 10;
const HISTORY_MIN_INTERVAL_SECS: i64 = 600;
const HISTORY_MAX_AGE_SECS: i64 = 48 * 3600;
const HISTORY_MAX_LINES: usize = 512;
const LOW_QUOTA_ALERT_THRESHOLD_PCT: u8 = 15;

static CACHE_LOCK: Mutex<i64> = Mutex::new(0);
static BG_REFRESH_IN_FLIGHT: AtomicBool = AtomicBool::new(false);
static BG_REFRESH_LAST_SECS: AtomicI64 = AtomicI64::new(0);

#[derive(Clone)]
struct RoutingCacheSnapshot {
    path: PathBuf,
    mtime: Option<SystemTime>,
    len: u64,
    cache: UsageCacheFile,
}

static ROUTING_CACHE_SNAPSHOT: OnceLock<Mutex<Option<RoutingCacheSnapshot>>> = OnceLock::new();

fn load_routing_cache_cached(path: &Path) -> Option<UsageCacheFile> {
    let meta = fs::metadata(path).ok()?;
    let mtime = meta.modified().ok();
    let len = meta.len();
    let slot = ROUTING_CACHE_SNAPSHOT.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = slot.lock()
        && let Some(snap) = guard.as_ref()
        && snap.path == path
        && snap.mtime == mtime
        && snap.len == len
    {
        return Some(snap.cache.clone());
    }
    let cache = load_cache_from(path);
    if let Ok(mut guard) = slot.lock() {
        *guard = Some(RoutingCacheSnapshot {
            path: path.to_path_buf(),
            mtime,
            len,
            cache: cache.clone(),
        });
    }
    Some(cache)
}

fn entry_credits_usable(entry: &CachedQuotaEntry) -> bool {
    entry.credits_unlimited
        || entry
            .credits_balance
            .as_deref()
            .and_then(|raw| raw.trim().parse::<f64>().ok())
            .is_some_and(|val| val > 0.0)
}

fn normalized_profile_key(profile_dir: Option<&str>) -> Option<&str> {
    profile_dir
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "default")
}

/// Projette le cache 0-jeton `usage-quotas.json` vers un fait `Quota` exploitable
/// par le routage (`observe_routing_candidates_for_project`) :
/// - bloque immédiatement toute connexion dont une fenêtre active est à 100 % / `rejected` /
///   `rate_limit_reached` (sans crédits Codex utilisables), y compris pour les profils frères
///   partageant le même `account_id` attesté ;
/// - fournit le `used_percent` réel (si observé depuis <= 30 min) afin que les connexions saines
///   reçoivent leur véritable score `quota_remaining`.
pub(crate) fn routing_quota_from_cache_at(
    path: &Path,
    connection_id: &str,
    engine: &str,
    machine_id: &str,
    profile_dir: Option<&str>,
    now: i64,
) -> Option<crate::provider_limits::Quota> {
    let cache = load_routing_cache_cached(path)?;
    let target_engine = engine.to_ascii_lowercase();
    let target_profile = normalized_profile_key(profile_dir);

    let mut matched: Vec<&CachedQuotaEntry> = cache
        .entries
        .values()
        .filter(|entry| {
            if entry.engine.to_ascii_lowercase() != target_engine {
                return false;
            }
            if entry.connection_id.as_deref() == Some(connection_id) {
                return true;
            }
            entry.machine_id == machine_id
                && normalized_profile_key(entry.profile_dir.as_deref()) == target_profile
        })
        .collect();
    if matched.is_empty() {
        return None;
    }

    // Si une entrée directe porte un `account_id` attesté, inclure les entrées
    // partageant exactement ce même `(engine, account_id)` (même abonnement,
    // sans jamais transposer entre deux comptes distincts).
    let account_ids: BTreeSet<&str> = matched
        .iter()
        .filter_map(|e| {
            e.account_id
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
        })
        .collect();
    if !account_ids.is_empty() {
        for entry in cache.entries.values() {
            if entry.engine.to_ascii_lowercase() == target_engine
                && entry
                    .account_id
                    .as_deref()
                    .map(str::trim)
                    .is_some_and(|acc| account_ids.contains(acc))
                && !matched
                    .iter()
                    .any(|existing| std::ptr::eq(*existing, entry))
            {
                matched.push(entry);
            }
        }
    }

    if matched.iter().all(|entry| entry.logged_in == Some(false)) {
        return None;
    }

    // Pour chaque fenêtre, conserver l'observation la plus récente non expirée.
    let mut best_windows: BTreeMap<&str, (&CachedWindow, &CachedQuotaEntry)> = BTreeMap::new();
    for entry in &matched {
        for (wkey, win) in &entry.windows {
            if win.observed_at <= 0 || win.observed_at > now + 60 {
                continue;
            }
            if win.resets_at.is_some_and(|at| at <= now) {
                continue;
            }
            match best_windows.get(wkey.as_str()) {
                Some((prev, _)) if prev.observed_at >= win.observed_at => {}
                _ => {
                    best_windows.insert(wkey.as_str(), (win, entry));
                }
            }
        }
    }
    if best_windows.is_empty() {
        return None;
    }

    let mut windows = Vec::new();
    let mut exhausted = false;
    let mut launch_blocked = false;
    let mut limit_reached = false;
    let mut resets_at: Option<i64> = None;
    let mut observed_at: Option<i64> = None;
    let mut codex_credits: Option<bridget_transport::protocol::CodexCredits> = None;

    for (win, entry) in best_windows.values() {
        let is_exhausted = matches!(win.status.as_str(), "rejected" | "rate_limit_reached")
            || win.used_percent.is_some_and(|p| p >= 100);
        let fresh_enough = if is_exhausted {
            win.resets_at.map_or_else(
                || now.saturating_sub(win.observed_at) <= ROUTING_UNKNOWN_EXHAUSTION_TTL_SECS,
                |at| at > now,
            )
        } else {
            now.saturating_sub(win.observed_at) <= ROUTING_POSITIVE_CACHE_MAX_AGE_SECS
        };
        if !fresh_enough {
            continue;
        }
        let credits_usable = target_engine == "codex" && entry_credits_usable(entry);
        if target_engine == "codex" && (entry.credits_unlimited || entry.credits_balance.is_some())
        {
            codex_credits = Some(bridget_transport::protocol::CodexCredits {
                has_credits: Some(credits_usable),
                unlimited: Some(entry.credits_unlimited),
                spend_control_reached: false,
            });
        }
        exhausted |= is_exhausted;
        launch_blocked |= is_exhausted && !credits_usable;
        limit_reached |= is_exhausted;
        if is_exhausted {
            resets_at = resets_at.max(win.resets_at);
        } else if resets_at.is_none() {
            resets_at = win.resets_at;
        }
        observed_at = observed_at.max(Some(win.observed_at));
        windows.push(serde_json::json!({
            "window": win.window,
            "used_percent": win.used_percent,
            "resets_at": win.resets_at,
            "exhausted": is_exhausted,
        }));
    }

    let observed_at = observed_at?;
    Some(crate::provider_limits::Quota {
        windows,
        exhausted,
        launch_blocked,
        credits: codex_credits,
        limit_reached,
        resets_at,
        observed_at,
    })
}

/// Rafraîchit `usage-quotas.json` en arrière-plan (0 jeton LLM, cadence `USAGE_CACHE_TTL_SECS`)
/// afin que le maître dispose en continu des quotas réels de la flotte pour le routage.
pub(crate) fn maybe_spawn_background_refresh(registry_path: PathBuf) {
    if cfg!(test)
        || std::env::var("BRIDGET_DISABLE_USAGE_BG_PROBE")
            .is_ok_and(|v| matches!(v.trim(), "1" | "true" | "TRUE"))
    {
        return;
    }
    let now = now_secs();
    let last = BG_REFRESH_LAST_SECS.load(Ordering::Relaxed);
    if last > 0 && now.saturating_sub(last) < USAGE_CACHE_TTL_SECS as i64 {
        return;
    }
    if BG_REFRESH_IN_FLIGHT
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
        .is_err()
    {
        return;
    }
    BG_REFRESH_LAST_SECS.store(now, Ordering::Relaxed);
    if std::thread::Builder::new()
        .name("bridget-usage-probe".into())
        .spawn(move || {
            let _ = collect_usage(None, false, Some(&registry_path));
            BG_REFRESH_LAST_SECS.store(now_secs(), Ordering::Relaxed);
            BG_REFRESH_IN_FLIGHT.store(false, Ordering::Release);
        })
        .is_err()
    {
        BG_REFRESH_IN_FLIGHT.store(false, Ordering::Release);
    }
}

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logged_in: Option<bool>,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub manual_reset_expirations: Vec<i64>,
    pub observed_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MachineCell {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<String>,
    pub tone: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MachineAccountRow {
    pub machine_id: String,
    pub online: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex: Option<MachineCell>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claude: Option<MachineCell>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub antigravity: Option<MachineCell>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<MachineCell>,
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
    #[serde(default)]
    pub machines: Vec<MachineAccountRow>,
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
    account_email: Option<String>,
    #[serde(default)]
    logged_in: Option<bool>,
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
    #[serde(default)]
    manual_reset_expirations: Vec<i64>,
    #[serde(default)]
    auth_checked_at: Option<i64>,
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
    account_email: Option<String>,
    plan_type: Option<String>,
    windows: Vec<CachedWindow>,
    credits_balance: Option<String>,
    credits_unlimited: bool,
    manual_resets_available: Option<u32>,
    manual_reset_expirations: Vec<i64>,
}

struct AgyProbeResult {
    account_fingerprint: Option<String>,
    account_email: Option<String>,
    windows: Vec<CachedWindow>,
}

struct ClaudeAuthProbeResult {
    logged_in: bool,
    account_email: Option<String>,
    plan_type: Option<String>,
}

struct ClaudeOAuthUsageResult {
    windows: Vec<CachedWindow>,
    manual_resets_available: Option<u32>,
    manual_reset_expirations: Vec<i64>,
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
/// `antigravity`, `cursor`, `glm`), ou `Ok(None)` pour « tous ».
pub(crate) fn normalize_provider_alias(raw: &str) -> Result<Option<&'static str>, String> {
    let trimmed = raw.trim().to_ascii_lowercase();
    if trimmed.is_empty() || matches!(trimmed.as_str(), "all" | "tous" | "global" | "fleet") {
        return Ok(None);
    }
    match trimmed.as_str() {
        "claude" | "anthropic" | "claude-code" => Ok(Some("claude")),
        "glm" | "gclaude" => Ok(Some("glm")),
        "codex" | "openai" | "chatgpt" | "codex-pro" => Ok(Some("codex")),
        "antigravity" | "agy" | "google" | "gemini" => Ok(Some("antigravity")),
        "cursor" | "cursor-agent" => Ok(Some("cursor")),
        _ => Err(format!(
            "fournisseur inconnu '{}' (attendus : claude/anthropic, codex/openai, antigravity/agy/google, cursor, glm/gclaude, ou all)",
            raw.trim()
        )),
    }
}

pub(crate) fn provider_display_name(engine: &str) -> &'static str {
    match engine {
        "claude" => "Claude",
        "codex" => "Codex",
        "antigravity" => "Antigravity",
        "cursor" => "Cursor",
        "glm" => "GLM (gclaude)",
        _ => "Fournisseur",
    }
}

fn is_glm_target(target: &ConnectionTarget) -> bool {
    let cid = target.connection_id.to_ascii_lowercase();
    let atype = target.agent_type.to_ascii_lowercase();
    let cmd_base = Path::new(&target.command)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&target.command)
        .to_ascii_lowercase();
    let prof = target
        .profile_dir
        .as_deref()
        .unwrap_or("")
        .to_ascii_lowercase();
    cid.contains("glm")
        || atype.contains("glm")
        || cmd_base == "gclaude"
        || prof.contains("claude-glm")
}

fn format_plan_type(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    match trimmed.to_ascii_lowercase().as_str() {
        "pro" => "Pro".to_string(),
        "plus" => "Plus".to_string(),
        "team" => "Team".to_string(),
        "enterprise" => "Enterprise".to_string(),
        "max" => "Max".to_string(),
        "max_5x" | "max5x" => "Max 5x".to_string(),
        "max_20x" | "max20x" => "Max 20x".to_string(),
        other => other
            .split(['_', '-'])
            .filter(|p| !p.is_empty())
            .enumerate()
            .map(|(idx, part)| {
                if idx == 0 {
                    let mut ch = part.chars();
                    match ch.next() {
                        Some(first) => first.to_uppercase().collect::<String>() + ch.as_str(),
                        None => String::new(),
                    }
                } else {
                    part.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
    }
}

/// Formate un epoch Unix UTC en date courte française (`30 nov.`, `12 déc.`).
fn format_short_date_fr(epoch_secs: i64) -> String {
    let days = epoch_secs.div_euclid(86_400);
    // Howard Hinnant civil_from_days (UTC exact).
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let m_label = match month {
        1 => "janv.",
        2 => "févr.",
        3 => "mars",
        4 => "avr.",
        5 => "mai",
        6 => "juin",
        7 => "juil.",
        8 => "août",
        9 => "sept.",
        10 => "oct.",
        11 => "nov.",
        12 => "déc.",
        _ => "",
    };
    format!("{day} {m_label}")
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
    if lower == "five_hour"
        || lower.contains("five_hour")
        || lower == "primary/300m"
        || lower == "secondary/300m"
    {
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

/// Convertit un horodatage RFC3339/ISO-8601 UTC (`2026-11-30T20:03:03Z`, `+00:00` ou avec fractions)
/// en epoch Unix (secondes). Pur Rust, sans dépendance externe.
pub(crate) fn parse_iso8601_utc(raw: &str) -> Option<i64> {
    let s = raw.trim();
    let core = s
        .strip_suffix('Z')
        .or_else(|| s.strip_suffix('z'))
        .or_else(|| s.strip_suffix("+00:00"))
        .or_else(|| s.strip_suffix("-00:00"))?;
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

fn compute_delta_1h(
    pts: &[HistoryPoint],
    sub_id: &str,
    window: &str,
    cur_rem: u8,
    now: i64,
) -> Option<i16> {
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
    let entry = cache
        .entries
        .entry(key)
        .or_insert_with(|| CachedQuotaEntry {
            engine: engine.to_string(),
            machine_id: machine_id.to_string(),
            agent_type: agent_type.to_string(),
            connection_id: connection_id.map(str::to_string),
            profile_dir: profile_dir.map(str::to_string),
            account_id: None,
            account_email: None,
            logged_in: None,
            plan_type: None,
            source: format!("{source}"),
            windows: BTreeMap::new(),
            credits_balance: None,
            credits_unlimited: false,
            manual_resets_available: None,
            manual_reset_expirations: Vec::new(),
            auth_checked_at: None,
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

fn parse_timestamp_value(v: &Value) -> Option<i64> {
    if let Some(n) = v.as_i64() {
        if n > 1_000_000_000_000 {
            return Some(n / 1000);
        }
        return Some(n);
    }
    if let Some(s) = v.as_str() {
        if let Ok(n) = s.trim().parse::<i64>() {
            return Some(if n > 1_000_000_000_000 { n / 1000 } else { n });
        }
        return parse_iso8601_utc(s);
    }
    None
}

fn parse_codex_reset_credit_expirations(result: &Value) -> Vec<i64> {
    let Some(credits) = result
        .get("rateLimitResetCredits")
        .and_then(|c| c.get("credits"))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for item in credits {
        let ts = item
            .get("expiresAt")
            .or_else(|| item.get("expires_at"))
            .or_else(|| item.get("expiration"))
            .or_else(|| item.get("validUntil"))
            .or_else(|| item.get("valid_until"))
            .and_then(parse_timestamp_value);
        if let Some(t) = ts {
            out.push(t);
        }
    }
    out.sort_unstable();
    out
}

/// Parse la réponse JSON-RPC `account/read` de `codex app-server`.
fn parse_codex_account_read_response(value: &Value) -> (Option<String>, Option<String>) {
    let Some(acc) = value.get("result").and_then(|r| r.get("account")) else {
        return (None, None);
    };
    let email = acc
        .get("email")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let plan = acc
        .get("planType")
        .and_then(Value::as_str)
        .map(format_plan_type)
        .filter(|s| !s.is_empty());
    (email, plan)
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
    let manual_reset_expirations = parse_codex_reset_credit_expirations(result);
    let rl = result.get("rateLimits").or_else(|| {
        result
            .get("rateLimitsByLimitId")
            .and_then(|m| m.get("codex"))
    })?;
    let plan_type = rl
        .get("planType")
        .and_then(Value::as_str)
        .map(format_plan_type)
        .filter(|s| !s.is_empty());
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
        account_email: None,
        plan_type,
        windows,
        credits_balance,
        credits_unlimited,
        manual_resets_available,
        manual_reset_expirations,
    })
}

fn parse_antigravity_google_account_email(raw: &str) -> Option<String> {
    let val: Value = serde_json::from_str(raw.trim()).ok()?;
    val.get("active")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn read_local_antigravity_email(profile_dir: Option<&str>) -> Option<String> {
    if let Some(dir) = profile_dir.filter(|s| !s.trim().is_empty()) {
        let p = Path::new(dir).join("google_accounts.json");
        if let Ok(raw) = fs::read_to_string(p)
            && let Some(email) = parse_antigravity_google_account_email(&raw)
        {
            return Some(email);
        }
    }
    let home = std::env::var("HOME").ok()?;
    let p = PathBuf::from(home)
        .join(".gemini")
        .join("google_accounts.json");
    let raw = fs::read_to_string(p).ok()?;
    parse_antigravity_google_account_email(&raw)
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
        account_email: None,
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
    let local_email = if is_local {
        read_local_antigravity_email(profile_dir)
    } else {
        None
    };
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
        let acc_check = match profile_dir.filter(|s| !s.trim().is_empty()) {
            Some(dir) => format!(
                "f={}/google_accounts.json; [ -r \"$f\" ] || f=\"$HOME/.gemini/google_accounts.json\";",
                shell_single_quote(dir)
            ),
            None => "f=\"$HOME/.gemini/google_accounts.json\";".to_string(),
        };
        let remote_cmd = format!(
            "{acc_check} [ -r \"$f\" ] && printf '__BRIDGET_AGY_ACC_START__\\n' && cat \"$f\" && printf '\\n__BRIDGET_AGY_ACC_END__\\n'; PATH=\"$HOME/.local/bin:/opt/homebrew/bin:/usr/local/bin:$PATH\" {env_prefix}{} -p /usage --output-format json",
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
            return Err(format!(
                "délai sonde agy dépassé ({AGY_PROBE_TIMEOUT_SECS}s)"
            ));
        }
    };
    let _ = child.wait();
    let mut remote_email: Option<String> = None;
    let body_after_acc = if let Some(start) = raw.find("__BRIDGET_AGY_ACC_START__")
        && let Some(end) = raw[start..].find("__BRIDGET_AGY_ACC_END__")
    {
        let block = &raw[start + "__BRIDGET_AGY_ACC_START__".len()..start + end];
        remote_email = parse_antigravity_google_account_email(block);
        &raw[start + end + "__BRIDGET_AGY_ACC_END__".len()..]
    } else {
        raw.as_str()
    };
    let trimmed = body_after_acc.trim();
    if trimmed.is_empty() {
        return Err("machine hors ligne ou agy /usage vide".to_string());
    }
    // Tolérer une éventuelle ligne de log avant l'objet JSON.
    let json_slice = trimmed
        .find('{')
        .map(|idx| &trimmed[idx..])
        .unwrap_or(trimmed);
    let val: Value =
        serde_json::from_str(json_slice).map_err(|_| "sortie agy /usage non JSON".to_string())?;
    let mut parsed = parse_antigravity_usage_response(&val, now)
        .ok_or_else(|| "réponse agy /usage sans groupes de quota".to_string())?;
    parsed.account_email = local_email.or(remote_email);
    Ok(parsed)
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
    let acc_req =
        r#"{"jsonrpc":"2.0","id":3,"method":"account/read","params":{"refreshToken":false}}"#;

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

    let (tx, rx) = std::sync::mpsc::channel::<Result<(Value, Option<Value>), String>>();
    std::thread::spawn(move || {
        let write_ok = writeln!(stdin, "{init}")
            .and_then(|_| stdin.flush())
            .and_then(|_| writeln!(stdin, "{inited}"))
            .and_then(|_| writeln!(stdin, "{read_req}"))
            .and_then(|_| writeln!(stdin, "{acc_req}"))
            .and_then(|_| stdin.flush());
        if let Err(e) = write_ok {
            let _ = tx.send(Err(format!("écriture JSON-RPC codex : {e}")));
            return;
        }
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        let mut rl_val: Option<Value> = None;
        let mut acc_val: Option<Value> = None;
        for _ in 0..64 {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Ok(val) = serde_json::from_str::<Value>(trimmed) {
                        match val.get("id").and_then(Value::as_i64) {
                            Some(2) => rl_val = Some(val),
                            Some(3) => acc_val = Some(val),
                            _ => {}
                        }
                        if rl_val.is_some() && acc_val.is_some() {
                            break;
                        }
                    }
                }
                Err(e) => {
                    if let Some(rl) = rl_val {
                        let _ = tx.send(Ok((rl, acc_val)));
                    } else {
                        let _ = tx.send(Err(format!("lecture JSON-RPC codex : {e}")));
                    }
                    return;
                }
            }
        }
        if let Some(rl) = rl_val {
            let _ = tx.send(Ok((rl, acc_val)));
        } else {
            let _ = tx.send(Err("machine hors ligne ou sonde indisponible".to_string()));
        }
    });

    let outcome = rx
        .recv_timeout(Duration::from_secs(CODEX_PROBE_TIMEOUT_SECS))
        .map_err(|_| format!("délai sonde codex dépassé ({CODEX_PROBE_TIMEOUT_SECS}s)"))?;
    let _ = child.kill();
    let _ = child.wait();
    let (val, acc_val) = outcome?;
    if let Some(err) = val.get("error") {
        let msg = err
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("erreur fournisseur");
        return Err(format!("codex account/rateLimits/read refusé : {msg}"));
    }
    let mut parsed = parse_codex_rate_limits_response(&val, now)
        .ok_or_else(|| "réponse codex sans bloc rateLimits".to_string())?;
    if let Some(av) = acc_val {
        let (email, plan) = parse_codex_account_read_response(&av);
        if email.is_some() {
            parsed.account_email = email;
        }
        if parsed.plan_type.is_none() {
            parsed.plan_type = plan;
        }
    }
    Ok(parsed)
}

fn parse_claude_auth_status_response(val: &Value) -> Option<ClaudeAuthProbeResult> {
    let logged_in = val.get("loggedIn").and_then(Value::as_bool)?;
    let account_email = val
        .get("email")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let plan_type = val
        .get("planTier")
        .and_then(Value::as_str)
        .or_else(|| val.get("subscriptionType").and_then(Value::as_str))
        .map(format_plan_type)
        .filter(|s| !s.is_empty());
    Some(ClaudeAuthProbeResult {
        logged_in,
        account_email,
        plan_type,
    })
}

fn run_command_with_timeout(
    mut child: std::process::Child,
    timeout_secs: u64,
) -> Result<String, String> {
    let Some(stdout) = child.stdout.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return Err("stdout indisponible".to_string());
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
                let _ = tx.send(Err(format!("lecture sortie : {e}")));
            }
        }
    });
    let raw = match rx.recv_timeout(Duration::from_secs(timeout_secs)) {
        Ok(Ok(text)) => text,
        Ok(Err(e)) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(e);
        }
        Err(_) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("délai dépassé ({timeout_secs}s)"));
        }
    };
    let _ = child.wait();
    Ok(raw)
}

fn probe_claude_auth(
    command: &str,
    profile_dir: Option<&str>,
    is_local: bool,
    machine_id: &str,
) -> Result<ClaudeAuthProbeResult, String> {
    let child = if is_local {
        let mut cmd = Command::new(command);
        cmd.args(["auth", "status"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        if let Some(dir) = profile_dir.filter(|s| !s.trim().is_empty()) {
            cmd.env("CLAUDE_CONFIG_DIR", dir);
        }
        cmd.spawn()
            .map_err(|e| format!("claude auth status local impossible : {e}"))?
    } else {
        let env_prefix = match profile_dir.filter(|s| !s.trim().is_empty()) {
            Some(dir) => format!("CLAUDE_CONFIG_DIR={} ", shell_single_quote(dir)),
            None => String::new(),
        };
        let remote_cmd = format!(
            "PATH=\"$HOME/.local/bin:/opt/homebrew/bin:/usr/local/bin:$PATH\" {env_prefix}{} auth status 2>/dev/null || true",
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
            .map_err(|e| format!("sonde SSH claude sur {machine_id} impossible : {e}"))?
    };
    let raw = run_command_with_timeout(child, CLAUDE_AUTH_PROBE_TIMEOUT_SECS)?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("machine hors ligne ou claude auth status vide".to_string());
    }
    let json_slice = trimmed
        .find('{')
        .map(|idx| &trimmed[idx..])
        .unwrap_or(trimmed);
    let val: Value = serde_json::from_str(json_slice)
        .map_err(|_| "sortie claude auth status non JSON".to_string())?;
    parse_claude_auth_status_response(&val)
        .ok_or_else(|| "réponse claude auth status sans champ loggedIn".to_string())
}

/// Parse la réponse officielle OAuth Anthropic `/api/oauth/usage?cedar_ember=1&skip_spend=1`
/// (0 jeton LLM : fenêtres `five_hour` / `seven_day` et crédits de reset `cedar_ember.grants`).
fn parse_claude_oauth_usage_response(val: &Value, now: i64) -> Option<ClaudeOAuthUsageResult> {
    let mut windows = Vec::new();

    let parse_win = |obj: &Value, name: &str| -> Option<CachedWindow> {
        let util = obj
            .get("utilization")
            .and_then(Value::as_f64)
            .or_else(|| obj.get("percent").and_then(Value::as_f64))?;
        let used = util.round().clamp(0.0, 100.0) as u8;
        let resets_at = obj
            .get("resets_at")
            .and_then(Value::as_str)
            .and_then(parse_iso8601_utc);
        let status = if used >= 100 {
            "rejected".to_string()
        } else {
            "available".to_string()
        };
        Some(CachedWindow {
            window: name.to_string(),
            pool: None,
            used_percent: Some(used),
            resets_at,
            status,
            observed_at: now,
        })
    };

    let seven_day = val
        .get("seven_day")
        .filter(|v| v.is_object())
        .and_then(|v| parse_win(v, "seven_day"))
        .or_else(|| {
            val.get("limits")
                .and_then(Value::as_array)
                .and_then(|arr| {
                    arr.iter()
                        .find(|l| l.get("kind").and_then(Value::as_str) == Some("weekly_all"))
                })
                .and_then(|v| parse_win(v, "seven_day"))
        });

    let five_hour = val
        .get("five_hour")
        .filter(|v| v.is_object())
        .and_then(|v| parse_win(v, "five_hour"));

    if let Some(w5) = five_hour {
        windows.push(w5);
    } else if seven_day.is_some() {
        // Fenêtre 5h non démarrée (0 % utilisé / 100 % disponible).
        windows.push(CachedWindow {
            window: "five_hour".to_string(),
            pool: None,
            used_percent: Some(0),
            resets_at: None,
            status: "idle".to_string(),
            observed_at: now,
        });
    }
    if let Some(w7) = seven_day {
        windows.push(w7);
    }

    let mut manual_resets_available: Option<u32> = None;
    let mut manual_reset_expirations: Vec<i64> = Vec::new();
    if let Some(grants) = val
        .get("cedar_ember")
        .and_then(|ce| ce.get("grants"))
        .and_then(Value::as_array)
    {
        let mut total_left = 0u32;
        for g in grants {
            let paused = g.get("paused").and_then(Value::as_bool).unwrap_or(false);
            if paused {
                continue;
            }
            let left = g.get("resets_left").and_then(Value::as_u64).unwrap_or(0) as u32;
            if left > 0 {
                total_left = total_left.saturating_add(left);
                if let Some(exp) = g
                    .get("ends_at")
                    .and_then(Value::as_str)
                    .and_then(parse_iso8601_utc)
                {
                    manual_reset_expirations.push(exp);
                }
            }
        }
        manual_reset_expirations.sort_unstable();
        manual_resets_available = Some(total_left);
    }

    if windows.is_empty() && manual_resets_available.is_none() {
        return None;
    }
    Some(ClaudeOAuthUsageResult {
        windows,
        manual_resets_available,
        manual_reset_expirations,
    })
}

fn read_local_claude_oauth_token(profile_dir: Option<&str>) -> Option<String> {
    let extract = |raw: &str| -> Option<String> {
        let val: Value = serde_json::from_str(raw.trim()).ok()?;
        val.get("claudeAiOauth")
            .and_then(|o| o.get("accessToken"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    };
    if let Some(dir) = profile_dir.filter(|s| !s.trim().is_empty()) {
        let p = Path::new(dir).join(".credentials.json");
        if let Ok(raw) = fs::read_to_string(p)
            && let Some(tok) = extract(&raw)
        {
            return Some(tok);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home)
            .join(".claude")
            .join(".credentials.json");
        if let Ok(raw) = fs::read_to_string(p)
            && let Some(tok) = extract(&raw)
        {
            return Some(tok);
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(out) = Command::new("security")
            .args([
                "find-generic-password",
                "-s",
                "Claude Code-credentials",
                "-w",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            && out.status.success()
            && let Ok(text) = String::from_utf8(out.stdout)
            && let Some(tok) = extract(&text)
        {
            return Some(tok);
        }
    }
    None
}

fn probe_claude_oauth_usage(
    profile_dir: Option<&str>,
    is_local: bool,
    machine_id: &str,
    now: i64,
) -> Result<ClaudeOAuthUsageResult, String> {
    let body = if is_local {
        let token = read_local_claude_oauth_token(profile_dir)
            .ok_or_else(|| "jeton OAuth Claude introuvable localement".to_string())?;
        let resp =
            minreq::get("https://api.anthropic.com/api/oauth/usage?cedar_ember=1&skip_spend=1")
                .with_header("Authorization", format!("Bearer {token}"))
                .with_header("anthropic-beta", "oauth-2025-04-20")
                .with_header("User-Agent", "claude-cli/2.1.332 (external, cli)")
                .with_header("Content-Type", "application/json")
                .with_timeout(8)
                .send()
                .map_err(|e| format!("requête OAuth usage Claude impossible : {e}"))?;
        if resp.status_code != 200 {
            return Err(format!("OAuth usage Claude HTTP {}", resp.status_code));
        }
        resp.as_str()
            .map(str::to_string)
            .map_err(|e| format!("lecture réponse OAuth usage Claude : {e}"))?
    } else {
        let dir_expr = match profile_dir.filter(|s| !s.trim().is_empty()) {
            Some(dir) => format!("Path({})", serde_json::to_string(dir).unwrap_or_default()),
            None => "Path.home() / '.claude'".to_string(),
        };
        let py = format!(
            "import json,urllib.request,urllib.error\n\
from pathlib import Path\n\
p={dir_expr}/'.credentials.json'\n\
tok=json.loads(p.read_text()).get('claudeAiOauth',{{}}).get('accessToken')\n\
assert tok\n\
req=urllib.request.Request('https://api.anthropic.com/api/oauth/usage?cedar_ember=1&skip_spend=1',headers={{'Authorization':'Bearer '+tok,'anthropic-beta':'oauth-2025-04-20','User-Agent':'claude-cli/2.1.332 (external, cli)','Content-Type':'application/json'}})\n\
with urllib.request.urlopen(req,timeout=8) as r: print(r.read().decode())\n"
        );
        let remote_cmd = format!("python3 -c {}", shell_single_quote(&py));
        let child = Command::new("ssh")
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
            .map_err(|e| {
                format!("sonde SSH OAuth usage Claude sur {machine_id} impossible : {e}")
            })?;
        run_command_with_timeout(child, 10)?
    };
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err("réponse OAuth usage Claude vide".to_string());
    }
    let val: Value = serde_json::from_str(trimmed)
        .map_err(|_| "réponse OAuth usage Claude non JSON".to_string())?;
    parse_claude_oauth_usage_response(&val, now)
        .ok_or_else(|| "réponse OAuth usage Claude sans fenêtre ni reset".to_string())
}

fn parse_cursor_list_models_logged_in(output: &str) -> bool {
    let lower = output.to_ascii_lowercase();
    if lower.contains("authentication required")
        || lower.contains("not logged in")
        || lower.contains("agent login")
        || lower.contains("please log in")
    {
        return false;
    }
    output.lines().any(|line| {
        let t = line.trim();
        t == "auto"
            || (t.contains('-')
                && t.bytes()
                    .next()
                    .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit()))
    })
}

fn probe_cursor_auth(
    command: &str,
    profile_dir: Option<&str>,
    is_local: bool,
    machine_id: &str,
) -> Result<bool, String> {
    let child = if is_local {
        let mut cmd = Command::new(command);
        cmd.arg("--list-models")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(dir) = profile_dir.filter(|s| !s.trim().is_empty()) {
            cmd.env("CURSOR_CONFIG_DIR", dir);
        }
        cmd.spawn()
            .map_err(|e| format!("cursor-agent local impossible : {e}"))?
    } else {
        let env_prefix = match profile_dir.filter(|s| !s.trim().is_empty()) {
            Some(dir) => format!("CURSOR_CONFIG_DIR={} ", shell_single_quote(dir)),
            None => String::new(),
        };
        let remote_cmd = format!(
            "PATH=\"$HOME/.local/bin:/opt/homebrew/bin:/usr/local/bin:$PATH\" {env_prefix}{} --list-models 2>&1 || true",
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
            .map_err(|e| format!("sonde SSH cursor sur {machine_id} impossible : {e}"))?
    };
    let raw = run_command_with_timeout(child, CURSOR_PROBE_TIMEOUT_SECS)?;
    if !is_local && raw.trim().is_empty() {
        return Err("machine hors ligne ou cursor-agent vide".to_string());
    }
    Ok(parse_cursor_list_models_logged_in(&raw))
}

fn probe_glm_auth(
    profile_dir: Option<&str>,
    is_local: bool,
    machine_id: &str,
) -> Result<bool, String> {
    if is_local {
        if let Some(dir) = profile_dir.filter(|s| !s.trim().is_empty()) {
            return Ok(Path::new(dir).exists());
        }
        if let Ok(home) = std::env::var("HOME") {
            return Ok(PathBuf::from(home).join(".claude-glm").exists());
        }
        return Ok(false);
    }
    let dir_check = match profile_dir.filter(|s| !s.trim().is_empty()) {
        Some(dir) => format!("[ -d {} ] && echo OK || echo NO", shell_single_quote(dir)),
        None => "[ -d \"$HOME/.claude-glm\" ] && echo OK || echo NO".to_string(),
    };
    let child = Command::new("ssh")
        .args([
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=4",
            "-o",
            "StrictHostKeyChecking=accept-new",
            machine_id,
            &dir_check,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("sonde SSH glm sur {machine_id} impossible : {e}"))?;
    let raw = run_command_with_timeout(child, CLAUDE_AUTH_PROBE_TIMEOUT_SECS)?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("machine hors ligne".to_string());
    }
    Ok(trimmed == "OK")
}

fn shell_single_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn fetch_remote_cache_if_useful(
    local_machine: &str,
    machines: &[String],
) -> Option<UsageCacheFile> {
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

fn sort_machines_local_first(machines: &mut [String], local_machine: &str) {
    machines.sort_by(|a, b| {
        let a_loc = a == local_machine;
        let b_loc = b == local_machine;
        b_loc.cmp(&a_loc).then_with(|| a.cmp(b))
    });
}

fn sort_windows_for_engine(engine: &str, windows: &mut [QuotaWindow]) {
    if engine == "antigravity" {
        let rank = |w: &str| match w {
            "gemini-5h" => 0,
            "gemini-weekly" => 1,
            "3p-5h" => 2,
            "3p-weekly" => 3,
            _ => 4,
        };
        windows.sort_by(|a, b| {
            rank(&a.window)
                .cmp(&rank(&b.window))
                .then_with(|| a.label.cmp(&b.label))
        });
    } else {
        let rank = |lbl: &str| match lbl {
            "5h" => 0,
            "7j" => 1,
            _ => 2,
        };
        windows.sort_by(|a, b| {
            rank(&a.label)
                .cmp(&rank(&b.label))
                .then_with(|| a.window.cmp(&b.window))
        });
    }
}

fn merge_quota_windows(target: &mut Vec<QuotaWindow>, incoming: Vec<QuotaWindow>, engine: &str) {
    for w in incoming {
        if let Some(existing) = target
            .iter_mut()
            .find(|e| e.window == w.window || e.label == w.label)
        {
            let prefer_new = (w.remaining_percent.is_some()
                && existing.remaining_percent.is_none())
                || (w.status != "reset_elapsed" && existing.status == "reset_elapsed")
                || (w.remaining_percent.is_some()
                    && existing.remaining_percent.is_some()
                    && w.resets_at.unwrap_or(0) >= existing.resets_at.unwrap_or(0));
            if prefer_new {
                *existing = w;
            }
        } else {
            target.push(w);
        }
    }
    sort_windows_for_engine(engine, target);
}

fn propagate_known_emails_by_account_id(subs: &mut [SubscriptionQuota]) {
    let mut known: BTreeMap<(String, String), String> = BTreeMap::new();
    for s in subs.iter() {
        if let (Some(acc_id), Some(email)) = (
            s.account_id.as_deref().filter(|v| !v.is_empty()),
            s.account_email.as_deref().filter(|v| !v.is_empty()),
        ) {
            known
                .entry((s.engine.clone(), acc_id.to_string()))
                .or_insert_with(|| email.to_string());
        }
    }
    for s in subs.iter_mut() {
        if s.account_email.as_deref().is_none_or(str::is_empty)
            && let Some(acc_id) = s.account_id.as_deref().filter(|v| !v.is_empty())
            && let Some(email) = known.get(&(s.engine.clone(), acc_id.to_string()))
        {
            s.account_email = Some(email.clone());
            s.account_label = email.clone();
        }
    }
}

/// Fusionne les abonnements partageant le même compte réel (Codex par `account_id`/`email`,
/// Antigravity par `empreinte`/`email`, Claude connecté par `email`, GLM et Cursor regroupés).
fn deduplicate_shared_accounts(
    subs: Vec<SubscriptionQuota>,
    local_machine: &str,
) -> Vec<SubscriptionQuota> {
    let mut by_account: BTreeMap<(String, String), SubscriptionQuota> = BTreeMap::new();
    let mut passthrough: Vec<SubscriptionQuota> = Vec::new();

    for mut sub in subs {
        sort_windows_for_engine(&sub.engine, &mut sub.windows);
        let dedup_key: Option<String> = match sub.engine.as_str() {
            "codex" => sub
                .account_id
                .clone()
                .filter(|s| !s.is_empty())
                .or_else(|| sub.account_email.clone().filter(|s| !s.is_empty())),
            "antigravity" => sub
                .account_id
                .clone()
                .filter(|s| !s.is_empty())
                .or_else(|| sub.account_email.clone().filter(|s| !s.is_empty())),
            "claude" => {
                if sub.logged_in == Some(true) {
                    sub.account_email.clone().filter(|s| !s.is_empty())
                } else {
                    None
                }
            }
            "glm" => Some("glm".to_string()),
            "cursor" => {
                if sub.source_state != "unavailable" {
                    Some("cursor".to_string())
                } else {
                    None
                }
            }
            _ => None,
        };

        let Some(acc_key) = dedup_key else {
            passthrough.push(sub);
            continue;
        };

        let key = (sub.engine.clone(), acc_key.clone());
        if let Some(existing) = by_account.get_mut(&key) {
            for m in &sub.machines {
                if !existing.machines.contains(m) {
                    existing.machines.push(m.clone());
                }
            }
            sort_machines_local_first(&mut existing.machines, local_machine);
            existing.machine_id = existing.machines.join(", ");
            for c in &sub.connection_ids {
                if !existing.connection_ids.contains(c) {
                    existing.connection_ids.push(c.clone());
                }
            }
            if existing.account_email.is_none() && sub.account_email.is_some() {
                existing.account_email = sub.account_email.clone();
            }
            if existing.account_id.is_none() && sub.account_id.is_some() {
                existing.account_id = sub.account_id.clone();
            }
            if existing.plan_type.is_none() && sub.plan_type.is_some() {
                existing.plan_type = sub.plan_type.clone();
            }
            if sub.logged_in == Some(true) {
                existing.logged_in = Some(true);
            }
            if sub.engine == "claude" {
                merge_quota_windows(&mut existing.windows, sub.windows, "claude");
                existing.observed_at = existing.observed_at.max(sub.observed_at);
                if sub.manual_resets_available.is_some() {
                    existing.manual_resets_available = sub.manual_resets_available;
                }
                if !sub.manual_reset_expirations.is_empty() {
                    existing.manual_reset_expirations = sub.manual_reset_expirations;
                }
                if sub.source_state == "live_probe" || existing.source_state == "live_probe" {
                    existing.source_state = "live_probe".to_string();
                    existing.source_label = "Sonde OAuth Claude (0 jeton)".to_string();
                } else if sub.source_state == "observed" || existing.source_state == "observed" {
                    existing.source_state = "observed".to_string();
                    existing.source_label = "Observé passivement".to_string();
                } else {
                    existing.source_state = "connected_no_quota".to_string();
                    existing.source_label = "Connecté (via StatusLine)".to_string();
                }
            } else if sub.engine == "cursor" {
                if existing.logged_in == Some(true) {
                    existing.source_state = "connected_no_quota".to_string();
                    existing.source_label = "Connecté".to_string();
                }
            } else if sub.engine == "glm" {
                if existing.logged_in == Some(true) {
                    existing.source_state = "connected_no_quota".to_string();
                    existing.source_label = "Connecté".to_string();
                }
            } else if sub.observed_at.unwrap_or(0) >= existing.observed_at.unwrap_or(0) {
                existing.observed_at = sub.observed_at.max(existing.observed_at);
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
                if !sub.manual_reset_expirations.is_empty() {
                    existing.manual_reset_expirations = sub.manual_reset_expirations;
                }
                existing.credits_unlimited |= sub.credits_unlimited;
            }
        } else {
            sort_machines_local_first(&mut sub.machines, local_machine);
            sub.machine_id = sub.machines.join(", ");
            sub.id = format!("{}:account:{}", sub.engine, acc_key);
            by_account.insert(key, sub);
        }
    }

    for sub in by_account.values_mut() {
        sub.account_label = match sub.engine.as_str() {
            "glm" => "GLM (gclaude)".to_string(),
            "cursor" => "Cursor".to_string(),
            _ => sub
                .account_email
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| match sub.engine.as_str() {
                    "codex" => "Codex".to_string(),
                    "antigravity" => "Compte Google".to_string(),
                    "claude" => "Claude".to_string(),
                    other => other.to_string(),
                }),
        };
        if let Some(plan) = sub.plan_type.as_deref() {
            let formatted = format_plan_type(plan);
            if !formatted.is_empty() {
                sub.plan_type = Some(formatted);
            }
        }
    }

    passthrough.extend(by_account.into_values());
    passthrough
}

fn build_machine_matrix(
    local_machine: &str,
    fleet_machines: &[String],
    raw_subs: &[SubscriptionQuota],
    machine_online: &BTreeMap<String, bool>,
) -> Vec<MachineAccountRow> {
    let mut all_machines: BTreeSet<String> = fleet_machines.iter().cloned().collect();
    for s in raw_subs {
        all_machines.insert(s.machine_id.clone());
    }
    let mut rows = Vec::new();
    for mid in all_machines {
        let online = machine_online
            .get(&mid)
            .copied()
            .unwrap_or(mid == local_machine);
        let for_m = |eng: &str| {
            raw_subs
                .iter()
                .find(|s| s.machine_id == mid && s.engine == eng)
        };

        let codex = for_m("codex").map(|s| {
            let cid = s.connection_ids.first().cloned();
            if s.source_state == "live_probe" {
                let primary = s
                    .windows
                    .iter()
                    .find(|w| w.label == "7j")
                    .or_else(|| s.windows.first());
                let rem = primary.and_then(|w| w.remaining_percent);
                let tone = rem
                    .map(tone_for_remaining)
                    .unwrap_or_else(|| "ok".to_string());
                MachineCell {
                    status: "connected".to_string(),
                    account: s
                        .account_email
                        .clone()
                        .or_else(|| Some(s.account_label.clone())),
                    summary: rem.map(|r| format!("{r} %")),
                    extra: None,
                    tone,
                    connection_id: cid,
                }
            } else if !online || s.source_state == "unavailable" {
                MachineCell {
                    status: "offline".to_string(),
                    account: None,
                    summary: Some("Hors ligne".to_string()),
                    extra: None,
                    tone: "muted".to_string(),
                    connection_id: cid,
                }
            } else {
                MachineCell {
                    status: "disconnected".to_string(),
                    account: None,
                    summary: Some("Non connecté".to_string()),
                    extra: None,
                    tone: "warn".to_string(),
                    connection_id: cid,
                }
            }
        });

        let glm_present = for_m("glm").is_some_and(|g| g.logged_in != Some(false));
        let claude = match for_m("claude") {
            Some(s) => {
                let cid = s.connection_ids.first().cloned();
                let extra = if glm_present {
                    Some("+ GLM".to_string())
                } else {
                    None
                };
                let cell = if s.logged_in == Some(true) {
                    let rem7 = s
                        .windows
                        .iter()
                        .find(|w| w.label == "7j")
                        .or_else(|| s.windows.first())
                        .and_then(|w| w.remaining_percent);
                    let plan_lbl = s
                        .plan_type
                        .clone()
                        .unwrap_or_else(|| "Connecté".to_string());
                    let summary = match rem7 {
                        Some(r) => Some(format!("{plan_lbl} · {r} %")),
                        None => Some(plan_lbl),
                    };
                    let tone = rem7
                        .map(tone_for_remaining)
                        .unwrap_or_else(|| "ok".to_string());
                    MachineCell {
                        status: "connected".to_string(),
                        account: s
                            .account_email
                            .clone()
                            .or_else(|| Some("Claude".to_string())),
                        summary,
                        extra,
                        tone,
                        connection_id: cid,
                    }
                } else if s.logged_in == Some(false) {
                    MachineCell {
                        status: "disconnected".to_string(),
                        account: None,
                        summary: Some("Non connecté".to_string()),
                        extra,
                        tone: "warn".to_string(),
                        connection_id: cid,
                    }
                } else if !online {
                    MachineCell {
                        status: "offline".to_string(),
                        account: None,
                        summary: Some("Hors ligne".to_string()),
                        extra,
                        tone: "muted".to_string(),
                        connection_id: cid,
                    }
                } else {
                    MachineCell {
                        status: "disconnected".to_string(),
                        account: None,
                        summary: Some("En attente".to_string()),
                        extra,
                        tone: "muted".to_string(),
                        connection_id: cid,
                    }
                };
                Some(cell)
            }
            None if glm_present => for_m("glm").map(|g| MachineCell {
                status: "connected".to_string(),
                account: Some("GLM (gclaude)".to_string()),
                summary: Some("Connecté".to_string()),
                extra: None,
                tone: "ok".to_string(),
                connection_id: g.connection_ids.first().cloned(),
            }),
            None => None,
        };

        let antigravity = for_m("antigravity").map(|s| {
            let cid = s.connection_ids.first().cloned();
            if s.source_state == "live_probe" {
                let gem = s
                    .windows
                    .iter()
                    .find(|w| w.window == "gemini-weekly")
                    .and_then(|w| w.remaining_percent);
                let tp = s
                    .windows
                    .iter()
                    .find(|w| w.window == "3p-weekly")
                    .and_then(|w| w.remaining_percent);
                let summary = match (gem, tp) {
                    (Some(g), Some(t)) => Some(format!("{g} % / {t} %")),
                    (Some(g), None) => Some(format!("{g} %")),
                    (None, Some(t)) => Some(format!("{t} %")),
                    (None, None) => Some("Connecté".to_string()),
                };
                let min_rem = match (gem, tp) {
                    (Some(g), Some(t)) => Some(g.min(t)),
                    (Some(g), None) => Some(g),
                    (None, Some(t)) => Some(t),
                    (None, None) => None,
                };
                let tone = min_rem
                    .map(tone_for_remaining)
                    .unwrap_or_else(|| "ok".to_string());
                MachineCell {
                    status: "connected".to_string(),
                    account: s
                        .account_email
                        .clone()
                        .or_else(|| Some("Compte Google".to_string())),
                    summary,
                    extra: None,
                    tone,
                    connection_id: cid,
                }
            } else if !online {
                MachineCell {
                    status: "offline".to_string(),
                    account: None,
                    summary: Some("Hors ligne".to_string()),
                    extra: None,
                    tone: "muted".to_string(),
                    connection_id: cid,
                }
            } else {
                MachineCell {
                    status: "disconnected".to_string(),
                    account: None,
                    summary: Some("Non connecté".to_string()),
                    extra: None,
                    tone: "warn".to_string(),
                    connection_id: cid,
                }
            }
        });

        let cursor = for_m("cursor").map(|s| {
            let cid = s.connection_ids.first().cloned();
            if s.logged_in == Some(true) {
                MachineCell {
                    status: "connected".to_string(),
                    account: Some("Connecté".to_string()),
                    summary: None,
                    extra: None,
                    tone: "ok".to_string(),
                    connection_id: cid,
                }
            } else if !online || s.source_state == "unavailable" {
                MachineCell {
                    status: "offline".to_string(),
                    account: None,
                    summary: Some("Hors ligne".to_string()),
                    extra: None,
                    tone: "muted".to_string(),
                    connection_id: cid,
                }
            } else {
                MachineCell {
                    status: "disconnected".to_string(),
                    account: None,
                    summary: Some("Non connecté".to_string()),
                    extra: None,
                    tone: "warn".to_string(),
                    connection_id: cid,
                }
            }
        });

        if codex.is_none() && claude.is_none() && antigravity.is_none() && cursor.is_none() {
            continue;
        }
        rows.push(MachineAccountRow {
            machine_id: mid,
            online,
            codex,
            claude,
            antigravity,
            cursor,
        });
    }

    rows.sort_by(|a, b| {
        let a_loc = a.machine_id == local_machine;
        let b_loc = b.machine_id == local_machine;
        b_loc
            .cmp(&a_loc)
            .then_with(|| b.online.cmp(&a.online))
            .then_with(|| a.machine_id.cmp(&b.machine_id))
    });
    rows
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
            (
                "antigravity-gemini",
                "Antigravity · Gemini",
                "gemini-weekly",
            ),
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

    // 3. Claude : si au moins un abonnement a un pourcentage réel (sonde OAuth ou StatusLine).
    let claude_pct: Vec<_> = subs
        .iter()
        .filter(|s| {
            s.engine == "claude"
                && matches!(s.source_state.as_str(), "live_probe" | "observed")
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
        let effective_engine = if target.engine == "claude" && is_glm_target(target) {
            "glm"
        } else {
            target.engine.as_str()
        };
        if let Some(f) = filter
            && effective_engine != f
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
            effective_engine,
            target.machine_id,
            prof.unwrap_or("default")
        );
        let entry = groups.entry(gid.clone()).or_insert_with(|| Group {
            id: gid,
            engine: effective_engine.to_string(),
            machine_id: target.machine_id.clone(),
            profile_dir: prof.map(str::to_string),
            command: target.command.clone(),
            is_local: target.is_local,
            account_label: if effective_engine == "glm" {
                "GLM (gclaude)".to_string()
            } else {
                friendly_subscription_label(target)
            },
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

    // Sondes gratuites en parallèle (0 jeton LLM) :
    // Codex (account/rateLimits/read + account/read), Antigravity (agy -p /usage + google_accounts.json),
    // Claude (claude auth status), Cursor (cursor-agent --list-models), GLM (dossier profil).
    let mut cache_dirty = false;
    let mut codex_jobs: Vec<(String, String, Option<String>, bool, String)> = Vec::new();
    let mut agy_jobs: Vec<(String, String, Option<String>, bool, String)> = Vec::new();
    let mut claude_auth_jobs: Vec<(String, String, Option<String>, bool, String)> = Vec::new();
    let mut cursor_jobs: Vec<(String, String, Option<String>, bool, String)> = Vec::new();
    let mut glm_jobs: Vec<(String, Option<String>, bool, String)> = Vec::new();

    for group in groups.values() {
        let ckey = cache_entry_key(
            &group.engine,
            &group.machine_id,
            group.profile_dir.as_deref(),
            group.connection_ids.first().map(String::as_str),
            group
                .agent_types
                .first()
                .map(String::as_str)
                .unwrap_or(&group.engine),
        );
        let cached_entry = cache.entries.get(&ckey);
        let cached_acc_id = cached_entry.and_then(|e| e.account_id.as_deref());
        let cached_email = cached_entry.and_then(|e| e.account_email.as_deref());
        let targeted = match target_subscription_id {
            None => force_refresh,
            Some(tid) => {
                force_refresh
                    && (tid == group.id
                        || tid == format!("{}:account:{}", group.engine, group.engine)
                        || cached_acc_id
                            .is_some_and(|a| tid == format!("{}:account:{a}", group.engine))
                        || cached_email
                            .is_some_and(|em| tid == format!("{}:account:{em}", group.engine)))
            }
        };

        if group.engine == "codex" {
            let fresh = cached_entry.is_some_and(|e| {
                !targeted
                    && e.source == "codex_rpc_probe"
                    && e.account_id.is_some()
                    && e.account_email.is_some()
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
            let fresh = cached_entry.is_some_and(|e| {
                !targeted
                    && e.source == "antigravity_cli_usage"
                    && !e.windows.is_empty()
                    && e.account_email.is_some()
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
        } else if group.engine == "claude" {
            let fresh = cached_entry.is_some_and(|e| {
                !targeted
                    && e.logged_in.is_some()
                    && e.auth_checked_at
                        .is_some_and(|ts| now.saturating_sub(ts) <= USAGE_CACHE_TTL_SECS as i64)
            });
            if !fresh {
                claude_auth_jobs.push((
                    group.id.clone(),
                    group.command.clone(),
                    group.profile_dir.clone(),
                    group.is_local,
                    group.machine_id.clone(),
                ));
            }
        } else if group.engine == "cursor" {
            let fresh = cached_entry.is_some_and(|e| {
                !targeted
                    && e.logged_in.is_some()
                    && e.auth_checked_at
                        .is_some_and(|ts| now.saturating_sub(ts) <= USAGE_CACHE_TTL_SECS as i64)
            });
            if !fresh {
                cursor_jobs.push((
                    group.id.clone(),
                    group.command.clone(),
                    group.profile_dir.clone(),
                    group.is_local,
                    group.machine_id.clone(),
                ));
            }
        } else if group.engine == "glm" {
            let fresh = cached_entry.is_some_and(|e| {
                !targeted
                    && e.logged_in.is_some()
                    && e.auth_checked_at
                        .is_some_and(|ts| now.saturating_sub(ts) <= USAGE_CACHE_TTL_SECS as i64)
            });
            if !fresh {
                glm_jobs.push((
                    group.id.clone(),
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
                (gid, mid, res)
            })
        })
        .collect();
    let agy_handles: Vec<_> = agy_jobs
        .into_iter()
        .map(|(gid, cmd, prof, is_local, mid)| {
            std::thread::spawn(move || {
                let res = probe_antigravity_usage(&cmd, prof.as_deref(), is_local, &mid, now);
                (gid, mid, res)
            })
        })
        .collect();
    let claude_handles: Vec<_> = claude_auth_jobs
        .into_iter()
        .map(|(gid, cmd, prof, is_local, mid)| {
            std::thread::spawn(move || {
                let res = probe_claude_auth(&cmd, prof.as_deref(), is_local, &mid);
                (gid, mid, res)
            })
        })
        .collect();
    let cursor_handles: Vec<_> = cursor_jobs
        .into_iter()
        .map(|(gid, cmd, prof, is_local, mid)| {
            std::thread::spawn(move || {
                let res = probe_cursor_auth(&cmd, prof.as_deref(), is_local, &mid);
                (gid, mid, res)
            })
        })
        .collect();
    let glm_handles: Vec<_> = glm_jobs
        .into_iter()
        .map(|(gid, prof, is_local, mid)| {
            std::thread::spawn(move || {
                let res = probe_glm_auth(prof.as_deref(), is_local, &mid);
                (gid, mid, res)
            })
        })
        .collect();

    let mut machine_online: BTreeMap<String, bool> = BTreeMap::new();
    machine_online.insert(local_machine.clone(), true);

    let mut codex_results: BTreeMap<String, Result<CodexProbeResult, String>> = BTreeMap::new();
    for h in codex_handles {
        if let Ok((gid, mid, res)) = h.join() {
            if res.is_ok() {
                machine_online.insert(mid, true);
            } else {
                machine_online.entry(mid).or_insert(false);
            }
            codex_results.insert(gid, res);
        }
    }
    let mut agy_results: BTreeMap<String, Result<AgyProbeResult, String>> = BTreeMap::new();
    for h in agy_handles {
        if let Ok((gid, mid, res)) = h.join() {
            if res.is_ok() {
                machine_online.insert(mid, true);
            } else {
                machine_online.entry(mid).or_insert(false);
            }
            agy_results.insert(gid, res);
        }
    }
    let mut claude_results: BTreeMap<String, Result<ClaudeAuthProbeResult, String>> =
        BTreeMap::new();
    for h in claude_handles {
        if let Ok((gid, mid, res)) = h.join() {
            if res.is_ok() {
                machine_online.insert(mid, true);
            } else {
                machine_online.entry(mid).or_insert(false);
            }
            claude_results.insert(gid, res);
        }
    }

    // Mettre à jour les métadonnées d'authentification Claude avant la sonde OAuth usage
    // (1 seule requête HTTPS /api/oauth/usage par email Claude unique pour éviter tout HTTP 429).
    let mut claude_oauth_jobs: BTreeMap<String, (Option<String>, bool, String)> = BTreeMap::new();
    for group in groups.values().filter(|g| g.engine == "claude") {
        let ckey = cache_entry_key(
            &group.engine,
            &group.machine_id,
            group.profile_dir.as_deref(),
            group.connection_ids.first().map(String::as_str),
            group
                .agent_types
                .first()
                .map(String::as_str)
                .unwrap_or(&group.engine),
        );
        if let Some(Ok(auth)) = claude_results.get(&group.id) {
            let entry = cache
                .entries
                .entry(ckey.clone())
                .or_insert_with(|| CachedQuotaEntry {
                    engine: "claude".to_string(),
                    machine_id: group.machine_id.clone(),
                    agent_type: group.agent_types.first().cloned().unwrap_or_default(),
                    connection_id: group.connection_ids.first().cloned(),
                    profile_dir: group.profile_dir.clone(),
                    source: "claude_auth_probe".to_string(),
                    observed_at: 0,
                    ..CachedQuotaEntry::default()
                });
            entry.logged_in = Some(auth.logged_in);
            entry.account_email = if auth.logged_in {
                auth.account_email.clone()
            } else {
                None
            };
            entry.plan_type = if auth.logged_in {
                auth.plan_type.clone()
            } else {
                None
            };
            entry.auth_checked_at = Some(now);
            cache_dirty = true;
        }
        let cached_entry = cache.entries.get(&ckey);
        let is_logged = cached_entry.and_then(|e| e.logged_in) == Some(true);
        let email = cached_entry
            .and_then(|e| e.account_email.clone())
            .filter(|s| !s.is_empty());
        if !is_logged {
            continue;
        }
        let acc_key = email.clone().unwrap_or_else(|| group.id.clone());
        let targeted = match target_subscription_id {
            None => force_refresh,
            Some(tid) => {
                force_refresh
                    && (tid == group.id
                        || email
                            .as_deref()
                            .is_some_and(|em| tid == format!("claude:account:{em}")))
            }
        };
        let fresh = cached_entry.is_some_and(|e| {
            !targeted
                && e.source == "claude_oauth_usage"
                && !e.windows.is_empty()
                && e.observed_at > 0
                && now.saturating_sub(e.observed_at) <= USAGE_CACHE_TTL_SECS as i64
        });
        if !fresh && !claude_oauth_jobs.contains_key(&acc_key) {
            claude_oauth_jobs.insert(
                acc_key,
                (
                    group.profile_dir.clone(),
                    group.is_local,
                    group.machine_id.clone(),
                ),
            );
        }
    }

    let claude_oauth_handles: Vec<_> = claude_oauth_jobs
        .into_iter()
        .map(|(acc_key, (prof, is_local, mid))| {
            std::thread::spawn(move || {
                let res = probe_claude_oauth_usage(prof.as_deref(), is_local, &mid, now);
                (acc_key, res)
            })
        })
        .collect();
    let mut claude_oauth_results: BTreeMap<String, Result<ClaudeOAuthUsageResult, String>> =
        BTreeMap::new();
    for h in claude_oauth_handles {
        if let Ok((acc_key, res)) = h.join() {
            claude_oauth_results.insert(acc_key, res);
        }
    }
    for group in groups.values().filter(|g| g.engine == "claude") {
        let ckey = cache_entry_key(
            &group.engine,
            &group.machine_id,
            group.profile_dir.as_deref(),
            group.connection_ids.first().map(String::as_str),
            group
                .agent_types
                .first()
                .map(String::as_str)
                .unwrap_or(&group.engine),
        );
        let Some(entry) = cache.entries.get_mut(&ckey) else {
            continue;
        };
        if entry.logged_in != Some(true) {
            continue;
        }
        let acc_key = entry
            .account_email
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| group.id.clone());
        if let Some(Ok(usage)) = claude_oauth_results.get(&acc_key) {
            for w in &usage.windows {
                entry.windows.insert(w.window.clone(), w.clone());
            }
            entry.manual_resets_available = usage.manual_resets_available;
            entry.manual_reset_expirations = usage.manual_reset_expirations.clone();
            entry.source = "claude_oauth_usage".to_string();
            entry.observed_at = now;
            cache_dirty = true;
        }
    }
    let mut cursor_results: BTreeMap<String, Result<bool, String>> = BTreeMap::new();
    for h in cursor_handles {
        if let Ok((gid, mid, res)) = h.join() {
            if res.is_ok() {
                machine_online.insert(mid, true);
            } else {
                machine_online.entry(mid).or_insert(false);
            }
            cursor_results.insert(gid, res);
        }
    }
    let mut glm_results: BTreeMap<String, Result<bool, String>> = BTreeMap::new();
    for h in glm_handles {
        if let Ok((gid, mid, res)) = h.join() {
            if res.is_ok() {
                machine_online.insert(mid, true);
            } else {
                machine_online.entry(mid).or_insert(false);
            }
            glm_results.insert(gid, res);
        }
    }

    let mut raw_subscriptions = Vec::new();
    for group in groups.into_values() {
        let ckey = cache_entry_key(
            &group.engine,
            &group.machine_id,
            group.profile_dir.as_deref(),
            group.connection_ids.first().map(String::as_str),
            group
                .agent_types
                .first()
                .map(String::as_str)
                .unwrap_or(&group.engine),
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
                            let prev_email = cache
                                .entries
                                .get(&ckey)
                                .and_then(|e| e.account_email.clone());
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
                                    account_email: pr.account_email.or(prev_email),
                                    logged_in: Some(true),
                                    plan_type: pr.plan_type,
                                    source: "codex_rpc_probe".to_string(),
                                    windows: map,
                                    credits_balance: pr.credits_balance,
                                    credits_unlimited: pr.credits_unlimited,
                                    manual_resets_available: pr.manual_resets_available,
                                    manual_reset_expirations: pr.manual_reset_expirations,
                                    auth_checked_at: Some(now),
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
                if any_live {
                    machine_online
                        .entry(group.machine_id.clone())
                        .or_insert(true);
                }
                let (source_state, source_label, detail) = if any_live {
                    (
                        "live_probe".to_string(),
                        "Sonde Codex (0 jeton)".to_string(),
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
                            "Machine hors ligne ou sonde indisponible.".to_string()
                        })),
                    )
                };
                let email = cached.and_then(|e| e.account_email.clone());
                raw_subscriptions.push(SubscriptionQuota {
                    id: group.id,
                    engine: group.engine,
                    provider_label: provider_display_name("codex").to_string(),
                    account_label: email.clone().unwrap_or(group.account_label),
                    account_email: email,
                    logged_in: if any_live {
                        Some(true)
                    } else {
                        cached.and_then(|e| e.logged_in)
                    },
                    machine_id: group.machine_id.clone(),
                    machines: vec![group.machine_id],
                    account_id: cached.and_then(|e| e.account_id.clone()),
                    connection_ids: group.connection_ids,
                    plan_type: cached
                        .and_then(|e| e.plan_type.as_deref())
                        .map(format_plan_type)
                        .filter(|s| !s.is_empty()),
                    source_state,
                    source_label,
                    windows,
                    credits_balance: cached.and_then(|e| e.credits_balance.clone()),
                    credits_unlimited: cached.is_some_and(|e| e.credits_unlimited),
                    manual_resets_available: cached.and_then(|e| e.manual_resets_available),
                    manual_reset_expirations: cached
                        .map(|e| e.manual_reset_expirations.clone())
                        .unwrap_or_default(),
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
                            let prev_email = cache
                                .entries
                                .get(&ckey)
                                .and_then(|e| e.account_email.clone());
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
                                    account_email: pr.account_email.or(prev_email),
                                    logged_in: Some(true),
                                    plan_type: None,
                                    source: "antigravity_cli_usage".to_string(),
                                    windows: map,
                                    credits_balance: None,
                                    credits_unlimited: false,
                                    manual_resets_available: None,
                                    manual_reset_expirations: Vec::new(),
                                    auth_checked_at: Some(now),
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
                if any_live {
                    machine_online
                        .entry(group.machine_id.clone())
                        .or_insert(true);
                }
                let (source_state, source_label, detail) = if any_live {
                    (
                        "live_probe".to_string(),
                        "Sonde agy /usage (0 jeton)".to_string(),
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
                let email = cached.and_then(|e| e.account_email.clone());
                raw_subscriptions.push(SubscriptionQuota {
                    id: group.id,
                    engine: group.engine,
                    provider_label: provider_display_name("antigravity").to_string(),
                    account_label: email.clone().unwrap_or(group.account_label),
                    account_email: email,
                    logged_in: if any_live {
                        Some(true)
                    } else {
                        cached.and_then(|e| e.logged_in)
                    },
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
                    manual_reset_expirations: Vec::new(),
                    observed_at: cached.map(|e| e.observed_at),
                    detail,
                });
            }
            "claude" => {
                let probe_err: Option<String> =
                    claude_results.remove(&group.id).and_then(|r| r.err());
                let cached_meta = cache.entries.get(&ckey).cloned();
                let logged_in = cached_meta.as_ref().and_then(|e| e.logged_in);
                let account_email = cached_meta.as_ref().and_then(|e| e.account_email.clone());
                let plan_type = cached_meta
                    .as_ref()
                    .and_then(|e| e.plan_type.as_deref())
                    .map(format_plan_type)
                    .filter(|s| !s.is_empty());
                let is_oauth_live = cached_meta
                    .as_ref()
                    .is_some_and(|e| e.source == "claude_oauth_usage");
                let manual_resets_available =
                    cached_meta.as_ref().and_then(|e| e.manual_resets_available);
                let manual_reset_expirations = cached_meta
                    .as_ref()
                    .map(|e| e.manual_reset_expirations.clone())
                    .unwrap_or_default();

                let mut merged_windows: BTreeMap<String, CachedWindow> = BTreeMap::new();
                let mut latest_obs: Option<i64> = None;

                if logged_in != Some(false) {
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
                            || (same_machine
                                && (same_type || entry.profile_dir == group.profile_dir))
                        {
                            if entry.observed_at > 0 {
                                latest_obs = latest_obs.max(Some(entry.observed_at));
                            }
                            for (wk, wv) in &entry.windows {
                                let prefer_new = match merged_windows.get(wk) {
                                    None => true,
                                    Some(cur) => {
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
                }

                let (windows, any_live) = materialize_windows(&merged_windows, now);
                let (source_state, source_label, detail) = if logged_in == Some(false) {
                    ("disconnected".to_string(), "Non connecté".to_string(), None)
                } else if any_live && is_oauth_live {
                    (
                        "live_probe".to_string(),
                        "Sonde OAuth Claude (0 jeton)".to_string(),
                        None,
                    )
                } else if any_live {
                    (
                        "observed".to_string(),
                        "Observé passivement".to_string(),
                        None,
                    )
                } else if logged_in == Some(true) {
                    (
                        "connected_no_quota".to_string(),
                        "Connecté (via StatusLine)".to_string(),
                        None,
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
                        "Hors ligne ou en attente".to_string(),
                        probe_err,
                    )
                };
                raw_subscriptions.push(SubscriptionQuota {
                    id: group.id,
                    engine: group.engine,
                    provider_label: provider_display_name("claude").to_string(),
                    account_label: account_email.clone().unwrap_or(group.account_label),
                    account_email,
                    logged_in,
                    machine_id: group.machine_id.clone(),
                    machines: vec![group.machine_id],
                    account_id: None,
                    connection_ids: group.connection_ids,
                    plan_type,
                    source_state,
                    source_label,
                    windows,
                    credits_balance: None,
                    credits_unlimited: false,
                    manual_resets_available,
                    manual_reset_expirations,
                    observed_at: latest_obs,
                    detail,
                });
            }
            "glm" => {
                let mut probe_err: Option<String> = None;
                if let Some(res) = glm_results.remove(&group.id) {
                    match res {
                        Ok(ok) => {
                            let entry = cache.entries.entry(ckey.clone()).or_insert_with(|| {
                                CachedQuotaEntry {
                                    engine: "glm".to_string(),
                                    machine_id: group.machine_id.clone(),
                                    agent_type: group
                                        .agent_types
                                        .first()
                                        .cloned()
                                        .unwrap_or_default(),
                                    connection_id: group.connection_ids.first().cloned(),
                                    profile_dir: group.profile_dir.clone(),
                                    source: "glm_auth_probe".to_string(),
                                    observed_at: now,
                                    ..CachedQuotaEntry::default()
                                }
                            });
                            entry.logged_in = Some(ok);
                            entry.auth_checked_at = Some(now);
                            entry.observed_at = now;
                            cache_dirty = true;
                        }
                        Err(e) => probe_err = Some(e),
                    }
                }
                let logged_in = cache.entries.get(&ckey).and_then(|e| e.logged_in);
                let (source_state, source_label) = match logged_in {
                    Some(true) => ("connected_no_quota".to_string(), "Connecté".to_string()),
                    Some(false) => ("disconnected".to_string(), "Non connecté".to_string()),
                    None => ("unavailable".to_string(), "Hors ligne".to_string()),
                };
                raw_subscriptions.push(SubscriptionQuota {
                    id: group.id,
                    engine: "glm".to_string(),
                    provider_label: provider_display_name("glm").to_string(),
                    account_label: "GLM (gclaude)".to_string(),
                    account_email: None,
                    logged_in,
                    machine_id: group.machine_id.clone(),
                    machines: vec![group.machine_id],
                    account_id: None,
                    connection_ids: group.connection_ids,
                    plan_type: None,
                    source_state,
                    source_label,
                    windows: Vec::new(),
                    credits_balance: None,
                    credits_unlimited: false,
                    manual_resets_available: None,
                    manual_reset_expirations: Vec::new(),
                    observed_at: cache.entries.get(&ckey).map(|e| e.observed_at),
                    detail: probe_err,
                });
            }
            "cursor" => {
                let mut probe_err: Option<String> = None;
                if let Some(res) = cursor_results.remove(&group.id) {
                    match res {
                        Ok(ok) => {
                            let entry = cache.entries.entry(ckey.clone()).or_insert_with(|| {
                                CachedQuotaEntry {
                                    engine: "cursor".to_string(),
                                    machine_id: group.machine_id.clone(),
                                    agent_type: group
                                        .agent_types
                                        .first()
                                        .cloned()
                                        .unwrap_or_default(),
                                    connection_id: group.connection_ids.first().cloned(),
                                    profile_dir: group.profile_dir.clone(),
                                    source: "cursor_auth_probe".to_string(),
                                    observed_at: now,
                                    ..CachedQuotaEntry::default()
                                }
                            });
                            entry.logged_in = Some(ok);
                            entry.auth_checked_at = Some(now);
                            entry.observed_at = now;
                            cache_dirty = true;
                        }
                        Err(e) => probe_err = Some(e),
                    }
                }
                let logged_in = cache.entries.get(&ckey).and_then(|e| e.logged_in);
                let (source_state, source_label) = match logged_in {
                    Some(true) => ("connected_no_quota".to_string(), "Connecté".to_string()),
                    Some(false) => (
                        "not_exposed_by_provider".to_string(),
                        "Non connecté".to_string(),
                    ),
                    None => ("unavailable".to_string(), "Hors ligne".to_string()),
                };
                raw_subscriptions.push(SubscriptionQuota {
                    id: group.id,
                    engine: "cursor".to_string(),
                    provider_label: provider_display_name("cursor").to_string(),
                    account_label: "Cursor".to_string(),
                    account_email: None,
                    logged_in,
                    machine_id: group.machine_id.clone(),
                    machines: vec![group.machine_id],
                    account_id: None,
                    connection_ids: group.connection_ids,
                    plan_type: None,
                    source_state,
                    source_label,
                    windows: Vec::new(),
                    credits_balance: None,
                    credits_unlimited: false,
                    manual_resets_available: None,
                    manual_reset_expirations: Vec::new(),
                    observed_at: cache.entries.get(&ckey).map(|e| e.observed_at),
                    detail: probe_err,
                });
            }
            _ => {}
        }
    }

    if cache_dirty && let Some(path) = cpath.as_deref() {
        cache.version = 1;
        let _ = save_cache_to(path, &cache);
    }

    propagate_known_emails_by_account_id(&mut raw_subscriptions);

    let machine_rows = if filter.is_none() {
        build_machine_matrix(
            &local_machine,
            &machines,
            &raw_subscriptions,
            &machine_online,
        )
    } else {
        Vec::new()
    };

    let mut subscriptions = deduplicate_shared_accounts(raw_subscriptions, &local_machine);

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
        let eng_rank = |e: &str| match e {
            "codex" => 0,
            "antigravity" => 1,
            "claude" => 2,
            "glm" => 3,
            "cursor" => 4,
            _ => 5,
        };
        let state_rank = |s: &str| match s {
            "live_probe" | "observed" => 0,
            "connected_no_quota" => 1,
            "stale_reset" => 2,
            "disconnected" => 3,
            "not_exposed_by_provider" => 4,
            _ => 5,
        };
        let rem_desc = |sub: &SubscriptionQuota| {
            sub.windows
                .iter()
                .find(|w| w.label == "7j")
                .or_else(|| sub.windows.first())
                .and_then(|w| w.remaining_percent)
                .map(|r| 100u8.saturating_sub(r))
                .unwrap_or(101)
        };
        eng_rank(&a.engine)
            .cmp(&eng_rank(&b.engine))
            .then_with(|| state_rank(&a.source_state).cmp(&state_rank(&b.source_state)))
            .then_with(|| rem_desc(a).cmp(&rem_desc(b)))
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
        machines: machine_rows,
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

fn format_reset_short_fr(resets_at: Option<i64>, now: i64, with_prefix: bool) -> String {
    let Some(at) = resets_at else {
        return String::new();
    };
    let diff = at - now;
    if diff <= 0 {
        return "réinitialisé".to_string();
    }
    let hours = diff / 3600;
    let mins = (diff % 3600) / 60;
    let dur = if hours >= 24 {
        format!("{}j {}h", hours / 24, hours % 24)
    } else if hours > 0 {
        format!("{hours}h {mins:02}m")
    } else {
        format!("{mins} min")
    };
    if with_prefix {
        format!("reset {dur}")
    } else {
        dur
    }
}

fn render_unicode_bar(remaining_pct: u8) -> String {
    let filled = ((remaining_pct as usize * 12) + 50) / 100;
    let filled = filled.min(12);
    format!("{}{}", "█".repeat(filled), "░".repeat(12 - filled))
}

fn format_resets_terminal(sub: &SubscriptionQuota, now: i64) -> String {
    match sub.engine.as_str() {
        "codex" | "claude" => match sub.manual_resets_available {
            Some(0) => " · 0 reset".to_string(),
            Some(n) => {
                let plural = if n > 1 { "s" } else { "" };
                let exp = sub
                    .manual_reset_expirations
                    .iter()
                    .copied()
                    .find(|t| *t > now)
                    .or_else(|| sub.manual_reset_expirations.first().copied())
                    .map(|t| format!(" (exp. {})", format_short_date_fr(t)))
                    .unwrap_or_default();
                format!(" · {n} reset{plural}{exp}")
            }
            None => String::new(),
        },
        _ => String::new(),
    }
}

fn vendor_visible_subs<'a>(
    subs: &'a [SubscriptionQuota],
    engine: &str,
) -> Vec<&'a SubscriptionQuota> {
    let all: Vec<_> = subs.iter().filter(|s| s.engine == engine).collect();
    let active: Vec<_> = all
        .iter()
        .copied()
        .filter(|s| {
            matches!(
                s.source_state.as_str(),
                "live_probe" | "observed" | "connected_no_quota" | "stale_reset"
            )
        })
        .collect();
    if !active.is_empty() { active } else { all }
}

/// Rendu terminal compact regroupé par éditeur, comptes réels, banked resets et machines.
pub(crate) fn render_terminal(report: &UsageReport) -> String {
    let mut out = String::new();
    let codex_subs = vendor_visible_subs(&report.subscriptions, "codex");
    let agy_subs = vendor_visible_subs(&report.subscriptions, "antigravity");
    let claude_subs = vendor_visible_subs(&report.subscriptions, "claude");
    let other_subs: Vec<_> = report
        .subscriptions
        .iter()
        .filter(|s| matches!(s.engine.as_str(), "glm" | "cursor"))
        .collect();

    if codex_subs.is_empty()
        && agy_subs.is_empty()
        && claude_subs.is_empty()
        && other_subs.is_empty()
    {
        out.push_str("Aucun abonnement trouvé pour ce filtre.\n");
        return out;
    }

    let render_vendor =
        |out: &mut String, title: &str, meta: String, list: &[&SubscriptionQuota]| {
            if list.is_empty() {
                return;
            }
            if meta.is_empty() {
                out.push_str(&format!("{title}\n"));
            } else {
                out.push_str(&format!("{title}  ({meta})\n"));
            }
            for sub in list {
                let plan = sub
                    .plan_type
                    .as_deref()
                    .map(|p| format!(" [{p}]"))
                    .unwrap_or_default();
                let hosts = if sub.machines.is_empty() {
                    sub.machine_id.clone()
                } else {
                    sub.machines.join(", ")
                };
                let resets = format_resets_terminal(sub, report.generated_at);
                out.push_str(&format!(
                    "  {} {plan} — {hosts}{resets}\n",
                    sub.account_label
                ));
                if sub.windows.is_empty() {
                    let st = if sub.logged_in == Some(true) {
                        "Connecté"
                    } else if sub.logged_in == Some(false) {
                        "Non connecté"
                    } else {
                        sub.source_label.as_str()
                    };
                    out.push_str(&format!("    {st}\n"));
                    continue;
                }
                for w in &sub.windows {
                    if w.status == "reset_elapsed" {
                        out.push_str(&format!(
                            "    {:>15}  [────────────] réinitialisé\n",
                            w.label
                        ));
                    } else if let Some(rem) = w.remaining_percent {
                        let rst = format_reset_short_fr(w.resets_at, report.generated_at, true);
                        let rst_part = if rst.is_empty() {
                            String::new()
                        } else {
                            format!(" · {rst}")
                        };
                        let delta_part = w
                            .delta_1h
                            .map(|d| format!(" ({:+} %/1h)", d))
                            .unwrap_or_default();
                        out.push_str(&format!(
                            "    {:>15}  [{}] {:>3} %{delta_part}{rst_part}\n",
                            w.label,
                            render_unicode_bar(rem),
                            rem
                        ));
                    } else if w.status == "rejected" {
                        let rst = format_reset_short_fr(w.resets_at, report.generated_at, true);
                        out.push_str(&format!(
                            "    {:>15}  [░░░░░░░░░░░░] 0 % · {rst}\n",
                            w.label
                        ));
                    } else {
                        let rst = format_reset_short_fr(w.resets_at, report.generated_at, true);
                        let rst_part = if rst.is_empty() {
                            String::new()
                        } else {
                            format!(" · {rst}")
                        };
                        out.push_str(&format!("    {:>15}  Disponible{rst_part}\n", w.label));
                    }
                }
            }
            out.push('\n');
        };

    let codex_meta = report
        .provider_totals
        .iter()
        .find(|t| t.engine == "codex")
        .map(|t| {
            format!(
                "{} % / {} % · {} compte{}",
                t.remaining_sum,
                t.capacity_max,
                t.credentials_count,
                if t.credentials_count > 1 { "s" } else { "" }
            )
        })
        .unwrap_or_else(|| {
            format!(
                "{} compte{}",
                codex_subs.len(),
                if codex_subs.len() > 1 { "s" } else { "" }
            )
        });
    render_vendor(&mut out, "Codex", codex_meta, &codex_subs);
    render_vendor(
        &mut out,
        "Antigravity",
        format!(
            "{} compte{}",
            agy_subs.len(),
            if agy_subs.len() > 1 { "s" } else { "" }
        ),
        &agy_subs,
    );
    render_vendor(
        &mut out,
        "Claude",
        format!(
            "{} compte{}",
            claude_subs.len(),
            if claude_subs.len() > 1 { "s" } else { "" }
        ),
        &claude_subs,
    );
    render_vendor(&mut out, "Autres", String::new(), &other_subs);

    if !report.machines.is_empty() {
        out.push_str("Machines\n");
        let cell_txt = |c: Option<&MachineCell>| -> String {
            let Some(c) = c else {
                return "—".to_string();
            };
            let mut parts = Vec::new();
            if let Some(a) = &c.account {
                parts.push(a.clone());
            }
            if let Some(s) = &c.summary {
                parts.push(s.clone());
            }
            if let Some(e) = &c.extra {
                parts.push(e.clone());
            }
            if parts.is_empty() {
                "—".to_string()
            } else {
                parts.join(" · ")
            }
        };
        for m in &report.machines {
            out.push_str(&format!(
                "  {:14} Codex: {} | Claude: {} | Antigravity: {} | Cursor: {}\n",
                m.machine_id,
                cell_txt(m.codex.as_ref()),
                cell_txt(m.claude.as_ref()),
                cell_txt(m.antigravity.as_ref()),
                cell_txt(m.cursor.as_ref()),
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

fn render_html_chips(sub: &SubscriptionQuota) -> String {
    let hosts: Vec<&str> = if sub.machines.is_empty() {
        vec![sub.machine_id.as_str()]
    } else {
        sub.machines.iter().map(String::as_str).collect()
    };
    hosts
        .into_iter()
        .map(|m| format!(r#"<span class="chip">{}</span>"#, html_esc(m)))
        .collect::<Vec<_>>()
        .join("")
}

fn render_html_resets(sub: &SubscriptionQuota, now: i64) -> String {
    match sub.engine.as_str() {
        "codex" | "claude" => match sub.manual_resets_available {
            Some(0) => r#"<div class="resets"><div><strong>0</strong> reset</div><div class="r-sub">—</div></div>"#.to_string(),
            Some(n) => {
                let plural = if n > 1 { "s" } else { "" };
                let exp_dates: Vec<String> = sub
                    .manual_reset_expirations
                    .iter()
                    .copied()
                    .filter(|t| *t > now)
                    .take(2)
                    .map(format_short_date_fr)
                    .collect();
                let sub_txt = if exp_dates.is_empty() {
                    "—".to_string()
                } else {
                    format!("exp. {}", exp_dates.join(", "))
                };
                format!(
                    r#"<div class="resets"><div><strong>{n}</strong> reset{plural}</div><div class="r-sub">{}</div></div>"#,
                    html_esc(&sub_txt)
                )
            }
            None => r#"<div class="resets"><div class="muted">—</div></div>"#.to_string(),
        },
        _ => r#"<div class="resets"><div class="muted">—</div></div>"#.to_string(),
    }
}

fn render_html_machine_cell(cell: Option<&MachineCell>) -> String {
    let Some(c) = cell else {
        return r#"<td class="muted">—</td>"#.to_string();
    };
    if c.status == "offline" {
        return r#"<td class="muted">Hors ligne</td>"#.to_string();
    }
    let dot_cls = match c.tone.as_str() {
        "ok" => "ok",
        "warn" => "warn",
        "danger" => "danger",
        _ => "muted",
    };
    let line_txt = match &c.account {
        Some(acc) => html_esc(acc),
        None => format!(
            r#"<span class="muted">{}</span>"#,
            html_esc(c.summary.as_deref().unwrap_or("Non connecté"))
        ),
    };
    let sub_html = if c.account.is_some() {
        let mut sub_parts = Vec::new();
        if let Some(s) = &c.summary {
            let colored = if s.ends_with('%') && !s.contains('/') {
                format!(r#"<span class="{dot_cls}">{}</span>"#, html_esc(s))
            } else {
                html_esc(s)
            };
            sub_parts.push(colored);
        }
        if let Some(ex) = &c.extra {
            sub_parts.push(html_esc(ex));
        }
        if sub_parts.is_empty() {
            String::new()
        } else {
            format!(r#"<div class="c-sub">{}</div>"#, sub_parts.join(" · "))
        }
    } else if let Some(ex) = &c.extra {
        format!(r#"<div class="c-sub">{}</div>"#, html_esc(ex))
    } else {
        String::new()
    };
    format!(
        r#"<td><div class="cell"><div class="c-line"><span class="dot {dot_cls}"></span>{line_txt}</div>{sub_html}</div></td>"#
    )
}

/// Rendu HTML autonome v4 regroupé par éditeur, comptes réels, banked resets et matrice machines.
pub(crate) fn render_html(report: &UsageReport) -> String {
    let now = report.generated_at;
    let mut sections = String::new();

    // 1. CODEX
    let codex_subs = vendor_visible_subs(&report.subscriptions, "codex");
    if !codex_subs.is_empty() {
        let meta = report
            .provider_totals
            .iter()
            .find(|t| t.engine == "codex")
            .map(|t| {
                let pct_cls = if t.remaining_sum == 0 {
                    "danger"
                } else if t.capacity_max > 0 && (t.remaining_sum * 100 / t.capacity_max) <= 20 {
                    "warn"
                } else {
                    "ok"
                };
                format!(
                    r#"<span class="{pct_cls}">{} %</span> / {} % · {} compte{}"#,
                    t.remaining_sum,
                    t.capacity_max,
                    t.credentials_count,
                    if t.credentials_count > 1 { "s" } else { "" }
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "{} compte{}",
                    codex_subs.len(),
                    if codex_subs.len() > 1 { "s" } else { "" }
                )
            });

        let mut rows = String::new();
        for sub in &codex_subs {
            let plan = sub
                .plan_type
                .as_deref()
                .map(|p| format!(r#"<span class="plan">{}</span>"#, html_esc(p)))
                .unwrap_or_default();
            let chips = render_html_chips(sub);
            let gauge = if let Some(w) = sub
                .windows
                .iter()
                .find(|w| w.label == "7j")
                .or_else(|| sub.windows.first())
            {
                let rem = w.remaining_percent.unwrap_or(0);
                let tone = if w.status == "rejected" || rem <= 10 {
                    "danger"
                } else if rem <= 30 {
                    "warn"
                } else {
                    "ok"
                };
                let left_sub = if rem == 0 || w.status == "rejected" {
                    "épuisé".to_string()
                } else if let Some(d) = w.delta_1h {
                    format!("{d:+} %/1h")
                } else {
                    format_age_fr(sub.observed_at, now)
                };
                let right_sub = format_reset_short_fr(w.resets_at, now, true);
                format!(
                    r#"<div class="gauge"><div class="g-top"><span class="muted">{}</span><span class="{tone} nw">{rem} %</span></div><div class="track"><div class="fill {tone}" style="width:{rem}%"></div></div><div class="g-sub"><span>{}</span><span>{}</span></div></div>"#,
                    html_esc(&w.label),
                    html_esc(&left_sub),
                    html_esc(&right_sub)
                )
            } else {
                format!(
                    r#"<div class="muted">{}</div>"#,
                    html_esc(&sub.source_label)
                )
            };
            rows.push_str(&format!(
                r#"<div class="row"><div class="who"><div><span class="email">{}</span>{plan}</div><div class="chips">{chips}</div></div>{gauge}{}</div>"#,
                html_esc(&sub.account_label),
                render_html_resets(sub, now)
            ));
        }
        sections.push_str(&format!(
            r#"<div class="vendor"><div class="v-head"><span class="v-name">Codex</span><span class="v-meta">{meta}</span></div>{rows}</div>"#
        ));
    }

    // 2. ANTIGRAVITY
    let agy_subs = vendor_visible_subs(&report.subscriptions, "antigravity");
    if !agy_subs.is_empty() {
        let meta = format!(
            "{} compte{}",
            agy_subs.len(),
            if agy_subs.len() > 1 { "s" } else { "" }
        );
        let mut rows = String::new();
        for sub in &agy_subs {
            let chips = render_html_chips(sub);
            let gauges_html = if sub.windows.is_empty() {
                format!(
                    r#"<div class="muted">{}</div>"#,
                    html_esc(&sub.source_label)
                )
            } else {
                let mut g = String::new();
                for w in &sub.windows {
                    let rem = w.remaining_percent.unwrap_or(0);
                    let tone = if w.status == "rejected" || rem <= 10 {
                        "danger"
                    } else if rem <= 30 {
                        "warn"
                    } else {
                        "ok"
                    };
                    let rst = format_reset_short_fr(w.resets_at, now, false);
                    g.push_str(&format!(
                        r#"<div class="gauge"><div class="g-top"><span class="muted">{}</span><span class="{tone} nw">{rem} %</span></div><div class="track"><div class="fill {tone}" style="width:{rem}%"></div></div><div class="g-sub"><span>{}</span></div></div>"#,
                        html_esc(&w.label),
                        html_esc(&rst)
                    ));
                }
                format!(r#"<div class="gauges-2">{g}</div>"#)
            };
            rows.push_str(&format!(
                r#"<div class="row"><div class="who"><div><span class="email">{}</span></div><div class="chips">{chips}</div></div>{gauges_html}{}</div>"#,
                html_esc(&sub.account_label),
                render_html_resets(sub, now)
            ));
        }
        sections.push_str(&format!(
            r#"<div class="vendor"><div class="v-head"><span class="v-name">Antigravity</span><span class="v-meta">{meta}</span></div>{rows}</div>"#
        ));
    }

    // 3. CLAUDE
    let claude_subs = vendor_visible_subs(&report.subscriptions, "claude");
    if !claude_subs.is_empty() {
        let meta = report
            .provider_totals
            .iter()
            .find(|t| t.engine == "claude")
            .map(|t| {
                let pct_cls = if t.remaining_sum == 0 {
                    "danger"
                } else if t.capacity_max > 0 && (t.remaining_sum * 100 / t.capacity_max) <= 20 {
                    "warn"
                } else {
                    "ok"
                };
                format!(
                    r#"<span class="{pct_cls}">{} %</span> / {} % · {} compte{}"#,
                    t.remaining_sum,
                    t.capacity_max,
                    t.credentials_count,
                    if t.credentials_count > 1 { "s" } else { "" }
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "{} compte{}",
                    claude_subs.len(),
                    if claude_subs.len() > 1 { "s" } else { "" }
                )
            });
        let mut rows = String::new();
        for sub in &claude_subs {
            let plan = sub
                .plan_type
                .as_deref()
                .map(|p| format!(r#"<span class="plan">{}</span>"#, html_esc(p)))
                .unwrap_or_default();
            let chips = render_html_chips(sub);
            let win_5h = sub.windows.iter().find(|w| w.label == "5h");
            let win_7j = sub.windows.iter().find(|w| w.label == "7j");
            let render_claude_win = |lbl: &str, opt: Option<&QuotaWindow>| -> String {
                match opt {
                    Some(w) if w.status != "reset_elapsed" => {
                        let rst = format_reset_short_fr(w.resets_at, now, true);
                        let sub_txt = if w.status == "idle" {
                            "inactif".to_string()
                        } else if rst.is_empty() {
                            "via StatusLine".to_string()
                        } else {
                            rst
                        };
                        if let Some(rem) = w.remaining_percent {
                            let tone = if w.status == "rejected" || rem <= 10 {
                                "danger"
                            } else if rem <= 30 {
                                "warn"
                            } else {
                                "ok"
                            };
                            format!(
                                r#"<div class="gauge"><div class="g-top"><span class="muted">{lbl}</span><span class="{tone} nw">{rem} %</span></div><div class="track"><div class="fill {tone}" style="width:{rem}%"></div></div><div class="g-sub"><span>{}</span></div></div>"#,
                                html_esc(&sub_txt)
                            )
                        } else if w.status == "rejected" {
                            format!(
                                r#"<div class="gauge"><div class="g-top"><span class="muted">{lbl}</span><span class="danger nw">0 %</span></div><div class="track"><div class="fill danger" style="width:0%"></div></div><div class="g-sub"><span>{}</span></div></div>"#,
                                html_esc(&sub_txt)
                            )
                        } else {
                            format!(
                                r#"<div class="gauge"><div class="g-top"><span class="muted">{lbl}</span><span class="ok nw">Disponible</span></div><div class="track"><div class="fill neutral" style="width:100%"></div></div><div class="g-sub"><span>{}</span></div></div>"#,
                                html_esc(&sub_txt)
                            )
                        }
                    }
                    _ => format!(
                        r#"<div class="gauge"><div class="g-top"><span class="muted">{lbl}</span><span class="muted nw">—</span></div><div class="track"></div><div class="g-sub"><span>via StatusLine</span></div></div>"#
                    ),
                }
            };
            let gauges_html = format!(
                r#"<div class="gauges-2">{}{}</div>"#,
                render_claude_win("5h", win_5h),
                render_claude_win("7j", win_7j)
            );
            rows.push_str(&format!(
                r#"<div class="row"><div class="who"><div><span class="email">{}</span>{plan}</div><div class="chips">{chips}</div></div>{gauges_html}{}</div>"#,
                html_esc(&sub.account_label),
                render_html_resets(sub, now)
            ));
        }
        sections.push_str(&format!(
            r#"<div class="vendor"><div class="v-head"><span class="v-name">Claude</span><span class="v-meta">{meta}</span></div>{rows}</div>"#
        ));
    }

    // 4. AUTRES (GLM & Cursor)
    let other_subs: Vec<_> = report
        .subscriptions
        .iter()
        .filter(|s| matches!(s.engine.as_str(), "glm" | "cursor"))
        .collect();
    if !other_subs.is_empty() {
        let mut rows = String::new();
        for sub in &other_subs {
            let chips = render_html_chips(sub);
            let (st_cls, st_txt) = if sub.logged_in == Some(true) {
                ("ok", "Connecté")
            } else if sub.logged_in == Some(false) {
                ("warn", "Non connecté")
            } else {
                ("muted", sub.source_label.as_str())
            };
            rows.push_str(&format!(
                r#"<div class="row"><div class="who"><div><span class="email">{}</span></div><div class="chips">{chips}</div></div><div class="{st_cls}">{}</div><div class="resets muted">—</div></div>"#,
                html_esc(&sub.account_label),
                html_esc(st_txt)
            ));
        }
        sections.push_str(&format!(
            r#"<div class="vendor"><div class="v-head"><span class="v-name">Autres</span></div>{rows}</div>"#
        ));
    }

    // 5. MACHINES
    if !report.machines.is_empty() {
        let mut tbody = String::new();
        for m in &report.machines {
            let row_cls = if m.online { "" } else { r#" class="off""# };
            tbody.push_str(&format!(
                r#"<tr{row_cls}><td class="m">{}</td>{}{}{}{}</tr>"#,
                html_esc(&m.machine_id),
                render_html_machine_cell(m.codex.as_ref()),
                render_html_machine_cell(m.claude.as_ref()),
                render_html_machine_cell(m.antigravity.as_ref()),
                render_html_machine_cell(m.cursor.as_ref()),
            ));
        }
        sections.push_str(&format!(
            r#"<div class="vendor"><div class="v-head"><span class="v-name">Machines</span></div><table><thead><tr><th>Machine</th><th>Codex</th><th>Claude</th><th>Antigravity</th><th>Cursor</th></tr></thead><tbody>{tbody}</tbody></table></div>"#
        ));
    }

    if sections.is_empty() {
        sections = r#"<div class="vendor"><div class="muted">Aucun abonnement trouvé pour ce filtre.</div></div>"#.to_string();
    }

    format!(
        r#"<!doctype html>
<html lang="fr">
<head>
<meta charset="utf-8" />
<style>
:root {{ color-scheme: light dark; }}
* {{ box-sizing: border-box; }}
body {{
  margin: 0;
  font-family: var(--font-sans, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif);
  color: var(--foreground, #e5e7eb);
  font-size: 12.5px;
  line-height: 1.35;
}}
.wrap {{ display: flex; flex-direction: column; gap: 10px; }}
.vendor {{
  background: var(--card, rgba(127,127,127,0.07));
  border: 1px solid var(--border, rgba(127,127,127,0.22));
  border-radius: var(--radius, 10px);
  padding: 10px 14px;
}}
.vendor:first-child .v-head {{ padding-right: 28px; }}
.v-head {{
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  gap: 8px;
  padding-bottom: 7px;
  border-bottom: 1px solid var(--border, rgba(127,127,127,0.18));
}}
.v-name {{ font-size: 13px; font-weight: 600; }}
.v-meta {{ font-size: 11.5px; color: var(--muted-foreground, var(--muted, #9ca3af)); }}
.row {{
  display: grid;
  grid-template-columns: minmax(175px, 1.05fr) minmax(240px, 1.65fr) minmax(95px, 0.55fr);
  gap: 12px;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid var(--border, rgba(127,127,127,0.12));
}}
.row:last-child {{ border-bottom: none; padding-bottom: 2px; }}
@media (max-width: 600px) {{
  .row {{ grid-template-columns: 1fr; gap: 6px; }}
}}
.who {{ display: flex; flex-direction: column; gap: 3px; min-width: 0; }}
.email {{ font-weight: 600; font-size: 12.5px; word-break: break-word; }}
.plan {{
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  padding: 0 5px;
  border-radius: 4px;
  background: var(--secondary, rgba(127,127,127,0.2));
  margin-left: 5px;
}}
.chips {{ display: flex; gap: 4px; flex-wrap: wrap; }}
.chip {{
  font-family: var(--font-mono, ui-monospace, SFMono-Regular, Menlo, monospace);
  font-size: 10px;
  padding: 1px 5px;
  border-radius: 4px;
  background: var(--secondary, rgba(127,127,127,0.15));
  color: var(--muted-foreground, var(--muted, #9ca3af));
}}
.gauges-2 {{ display: grid; grid-template-columns: 1fr 1fr; gap: 8px 12px; }}
.gauge {{ display: flex; flex-direction: column; gap: 4px; }}
.g-top {{ display: flex; justify-content: space-between; align-items: baseline; gap: 6px; font-size: 11.5px; }}
.g-sub {{ display: flex; justify-content: space-between; font-size: 10.5px; color: var(--muted-foreground, var(--muted, #9ca3af)); }}
.track {{ height: 5px; border-radius: 999px; background: var(--secondary, rgba(127,127,127,0.22)); overflow: hidden; }}
.fill {{ height: 100%; border-radius: 999px; }}
.fill.ok {{ background: var(--success, #22c55e); }}
.fill.warn {{ background: var(--warning, #f59e0b); }}
.fill.danger {{ background: var(--destructive, #ef4444); }}
.fill.neutral {{ background: color-mix(in srgb, var(--info, #3b82f6) 55%, transparent); }}
.ok {{ color: var(--success, #22c55e); font-weight: 600; }}
.warn {{ color: var(--warning, #f59e0b); font-weight: 600; }}
.danger {{ color: var(--destructive, #ef4444); font-weight: 600; }}
.muted {{ color: var(--muted-foreground, var(--muted, #9ca3af)); }}
.nw {{ white-space: nowrap; }}
.resets {{ font-size: 11.5px; text-align: right; }}
.resets .r-sub {{ font-size: 10px; color: var(--muted-foreground, var(--muted, #9ca3af)); }}
table {{ width: 100%; border-collapse: collapse; font-size: 11.5px; }}
th {{
  text-align: left;
  font-size: 10.5px;
  font-weight: 600;
  color: var(--muted-foreground, var(--muted, #9ca3af));
  padding: 4px 6px 6px;
  border-bottom: 1px solid var(--border, rgba(127,127,127,0.2));
}}
td {{
  padding: 6px 6px;
  border-bottom: 1px solid var(--border, rgba(127,127,127,0.1));
  vertical-align: top;
}}
tr:last-child td {{ border-bottom: none; }}
.m {{ font-family: var(--font-mono, monospace); font-weight: 600; white-space: nowrap; }}
.cell {{ display: flex; flex-direction: column; gap: 2px; }}
.c-line {{ display: flex; align-items: center; gap: 5px; }}
.dot {{ width: 6px; height: 6px; border-radius: 999px; flex-shrink: 0; }}
.dot.ok {{ background: var(--success, #22c55e); }}
.dot.warn {{ background: var(--warning, #f59e0b); }}
.dot.danger {{ background: var(--destructive, #ef4444); }}
.dot.muted {{ background: var(--muted-foreground, #6b7280); }}
.c-sub {{ padding-left: 11px; font-size: 10.5px; color: var(--muted-foreground, var(--muted, #9ca3af)); }}
.off {{ opacity: 0.55; }}
</style>
</head>
<body>
<div class="wrap">{sections}</div>
</body>
</html>"#
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
        assert_eq!(
            normalize_provider_alias("anthropic").unwrap(),
            Some("claude")
        );
        assert_eq!(normalize_provider_alias("Claude").unwrap(), Some("claude"));
        assert_eq!(normalize_provider_alias("openai").unwrap(), Some("codex"));
        assert_eq!(normalize_provider_alias("CODEX").unwrap(), Some("codex"));
        assert_eq!(
            normalize_provider_alias("agy").unwrap(),
            Some("antigravity")
        );
        assert_eq!(
            normalize_provider_alias("google").unwrap(),
            Some("antigravity")
        );
        assert_eq!(
            normalize_provider_alias("cursor-agent").unwrap(),
            Some("cursor")
        );
        assert_eq!(normalize_provider_alias("all").unwrap(), None);
        assert!(normalize_provider_alias("inexistant").is_err());
    }

    #[test]
    fn spec345_parse_antigravity_usage_json_and_iso8601_utc() {
        let now = parse_iso8601_utc("2026-11-24T14:48:25Z").unwrap();
        assert_eq!(parse_iso8601_utc("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(
            parse_iso8601_utc("2026-11-30T20:03:03Z"),
            Some(1_796_068_983)
        );

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
                "rateLimitResetCredits": {
                    "availableCount": 2,
                    "credits": [
                        { "expiresAt": "2026-11-30T20:03:03Z" },
                        { "expiresAt": 1_796_500_000 }
                    ]
                },
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
        let acc_payload = json!({
            "id": 3,
            "result": {
                "account": {
                    "type": "chatgpt",
                    "email": "moi@cartae.app",
                    "planType": "pro"
                }
            }
        });
        let (acc_email, acc_plan) = parse_codex_account_read_response(&acc_payload);
        assert_eq!(acc_email.as_deref(), Some("moi@cartae.app"));
        assert_eq!(acc_plan.as_deref(), Some("Pro"));

        let pr = parse_codex_rate_limits_response(&payload, now).unwrap();
        assert_eq!(
            pr.account_id.as_deref(),
            Some("82ae4828-611c-4a88-9020-dac5ee4a001c")
        );
        assert_eq!(pr.manual_resets_available, Some(2));
        assert_eq!(pr.manual_reset_expirations.len(), 2);
        assert_eq!(pr.plan_type.as_deref(), Some("Pro"));
        assert_eq!(pr.windows.len(), 2);

        let mut map = BTreeMap::new();
        for w in pr.windows {
            map.insert(w.window.clone(), w);
        }
        let (mat, any_live) = materialize_windows(&map, now);
        assert!(any_live);

        let raw_subs = vec![
            SubscriptionQuota {
                id: "codex:cartae:default".into(),
                engine: "codex".into(),
                provider_label: provider_display_name("codex").into(),
                account_label: "moi@cartae.app".into(),
                account_email: acc_email.clone(),
                logged_in: Some(true),
                machine_id: "cartae".into(),
                machines: vec!["cartae".into()],
                account_id: pr.account_id.clone(),
                connection_ids: vec!["codex-cartae-normal".into()],
                plan_type: pr.plan_type.clone(),
                source_state: "live_probe".into(),
                source_label: "Sonde Codex (0 jeton)".into(),
                windows: mat.clone(),
                credits_balance: pr.credits_balance.clone(),
                credits_unlimited: pr.credits_unlimited,
                manual_resets_available: pr.manual_resets_available,
                manual_reset_expirations: pr.manual_reset_expirations.clone(),
                observed_at: Some(now - 20),
                detail: None,
            },
            SubscriptionQuota {
                id: "codex:monordinateur:default".into(),
                engine: "codex".into(),
                provider_label: provider_display_name("codex").into(),
                account_label: "moi@cartae.app".into(),
                account_email: acc_email,
                logged_in: Some(true),
                machine_id: "monordinateur".into(),
                machines: vec!["monordinateur".into()],
                account_id: pr.account_id,
                connection_ids: vec!["codex-mac-normal".into()],
                plan_type: pr.plan_type,
                source_state: "live_probe".into(),
                source_label: "Sonde Codex (0 jeton)".into(),
                windows: mat,
                credits_balance: pr.credits_balance,
                credits_unlimited: pr.credits_unlimited,
                manual_resets_available: pr.manual_resets_available,
                manual_reset_expirations: pr.manual_reset_expirations,
                observed_at: Some(now - 5),
                detail: None,
            },
        ];

        let mut online = BTreeMap::new();
        online.insert("monordinateur".to_string(), true);
        online.insert("cartae".to_string(), true);
        let machine_rows = build_machine_matrix(
            "monordinateur",
            &["monordinateur".into(), "cartae".into()],
            &raw_subs,
            &online,
        );
        assert_eq!(machine_rows.len(), 2);
        assert_eq!(machine_rows[0].machine_id, "monordinateur");

        let deduped = deduplicate_shared_accounts(raw_subs, "monordinateur");
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].account_label, "moi@cartae.app");
        assert_eq!(deduped[0].machines, vec!["monordinateur", "cartae"]);

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
            machines: machine_rows,
            summary: UsageSummary {
                total_subscriptions: 1,
                with_live_quota: 1,
                not_exposed: 0,
                exhausted: 0,
                low_quota_alerts: 0,
            },
        };
        let html = render_html(&report);
        assert!(html.contains("moi@cartae.app"));
        assert!(html.contains("<strong>2</strong> resets"));
        assert!(html.contains("exp. 30 nov."));
        assert!(html.contains("56 %"));
        assert!(html.contains("padding-right: 28px"));
        assert!(html.contains("Machines"));
        assert!(!html.contains("0 jeton LLM"));
    }

    #[test]
    fn spec349_claude_auth_dedup_glm_separation_and_agy_email() {
        let auth_ok = parse_claude_auth_status_response(&json!({
            "loggedIn": true,
            "email": "fullflare@runbox.com",
            "planTier": "max_20x",
            "subscriptionType": "max"
        }))
        .unwrap();
        assert!(auth_ok.logged_in);
        assert_eq!(
            auth_ok.account_email.as_deref(),
            Some("fullflare@runbox.com")
        );
        assert_eq!(auth_ok.plan_type.as_deref(), Some("Max 20x"));

        let auth_no = parse_claude_auth_status_response(&json!({ "loggedIn": false })).unwrap();
        assert!(!auth_no.logged_in);
        assert_eq!(
            parse_antigravity_google_account_email(r#"{"active":"fullflare@gmail.com","old":[]}"#)
                .as_deref(),
            Some("fullflare@gmail.com")
        );
        assert!(parse_cursor_list_models_logged_in(
            "auto\nclaude-4-sonnet\n"
        ));
        assert!(!parse_cursor_list_models_logged_in(
            "Authentication required. Please run agent login."
        ));

        let now = 1_795_500_000;
        let raw = vec![
            SubscriptionQuota {
                id: "claude:cartae:default".into(),
                engine: "claude".into(),
                provider_label: "Claude".into(),
                account_label: "fullflare@runbox.com".into(),
                account_email: Some("fullflare@runbox.com".into()),
                logged_in: Some(true),
                machine_id: "cartae".into(),
                machines: vec!["cartae".into()],
                account_id: None,
                connection_ids: vec!["claude-cartae".into()],
                plan_type: Some("Max 20x".into()),
                source_state: "observed".into(),
                source_label: "Observé passivement".into(),
                windows: vec![QuotaWindow {
                    window: "seven_day".into(),
                    label: "7j".into(),
                    pool: None,
                    used_percent: None,
                    remaining_percent: None,
                    resets_at: Some(now + 3600 * 100),
                    status: "allowed".into(),
                    delta_1h: None,
                }],
                credits_balance: None,
                credits_unlimited: false,
                manual_resets_available: None,
                manual_reset_expirations: Vec::new(),
                observed_at: Some(now - 60),
                detail: None,
            },
            SubscriptionQuota {
                id: "claude:ksstore:default".into(),
                engine: "claude".into(),
                provider_label: "Claude".into(),
                account_label: "fullflare@runbox.com".into(),
                account_email: Some("fullflare@runbox.com".into()),
                logged_in: Some(true),
                machine_id: "ksstore".into(),
                machines: vec!["ksstore".into()],
                account_id: None,
                connection_ids: vec!["claude-ksstore".into()],
                plan_type: Some("Max 20x".into()),
                source_state: "connected_no_quota".into(),
                source_label: "Connecté".into(),
                windows: vec![],
                credits_balance: None,
                credits_unlimited: false,
                manual_resets_available: None,
                manual_reset_expirations: Vec::new(),
                observed_at: None,
                detail: None,
            },
            SubscriptionQuota {
                id: "claude:monordinateur:default".into(),
                engine: "claude".into(),
                provider_label: "Claude".into(),
                account_label: "Claude · monordinateur".into(),
                account_email: None,
                logged_in: Some(false),
                machine_id: "monordinateur".into(),
                machines: vec!["monordinateur".into()],
                account_id: None,
                connection_ids: vec!["claude-mac".into()],
                plan_type: None,
                source_state: "disconnected".into(),
                source_label: "Non connecté".into(),
                windows: vec![],
                credits_balance: None,
                credits_unlimited: false,
                manual_resets_available: None,
                manual_reset_expirations: Vec::new(),
                observed_at: None,
                detail: None,
            },
            SubscriptionQuota {
                id: "glm:monordinateur:~/.claude-glm".into(),
                engine: "glm".into(),
                provider_label: "GLM (gclaude)".into(),
                account_label: "GLM (gclaude)".into(),
                account_email: None,
                logged_in: Some(true),
                machine_id: "monordinateur".into(),
                machines: vec!["monordinateur".into()],
                account_id: None,
                connection_ids: vec!["glm-mac".into()],
                plan_type: None,
                source_state: "connected_no_quota".into(),
                source_label: "Connecté".into(),
                windows: vec![],
                credits_balance: None,
                credits_unlimited: false,
                manual_resets_available: None,
                manual_reset_expirations: Vec::new(),
                observed_at: Some(now),
                detail: None,
            },
        ];
        let mut online = BTreeMap::new();
        online.insert("monordinateur".to_string(), true);
        online.insert("cartae".to_string(), true);
        online.insert("ksstore".to_string(), true);
        let matrix = build_machine_matrix("monordinateur", &[], &raw, &online);
        let mac_row = matrix
            .iter()
            .find(|r| r.machine_id == "monordinateur")
            .unwrap();
        let mac_claude = mac_row.claude.as_ref().unwrap();
        assert_eq!(mac_claude.status, "disconnected");
        assert_eq!(mac_claude.extra.as_deref(), Some("+ GLM"));

        let deduped = deduplicate_shared_accounts(raw, "monordinateur");
        let active_claude = vendor_visible_subs(&deduped, "claude");
        assert_eq!(active_claude.len(), 1);
        assert_eq!(active_claude[0].account_label, "fullflare@runbox.com");
        assert_eq!(active_claude[0].machines, vec!["cartae", "ksstore"]);
        assert_eq!(active_claude[0].windows.len(), 1);

        // Antigravity : propagation de l'email connu via l'empreinte hebdomadaire commune
        // même si certains serveurs n'ont pas ~/.gemini/google_accounts.json.
        let mut agy_raw = vec![
            SubscriptionQuota {
                id: "antigravity:monordinateur:default".into(),
                engine: "antigravity".into(),
                provider_label: "Antigravity".into(),
                account_label: "fullflare@gmail.com".into(),
                account_email: Some("fullflare@gmail.com".into()),
                logged_in: Some(true),
                machine_id: "monordinateur".into(),
                machines: vec!["monordinateur".into()],
                account_id: Some("agy:3p-weekly:100|gemini-weekly:200".into()),
                connection_ids: vec!["agy-mac".into()],
                plan_type: None,
                source_state: "live_probe".into(),
                source_label: "Sonde agy /usage".into(),
                windows: vec![],
                credits_balance: None,
                credits_unlimited: false,
                manual_resets_available: None,
                manual_reset_expirations: Vec::new(),
                observed_at: Some(now),
                detail: None,
            },
            SubscriptionQuota {
                id: "antigravity:ksstore:default".into(),
                engine: "antigravity".into(),
                provider_label: "Antigravity".into(),
                account_label: "Antigravity · ksstore".into(),
                account_email: None,
                logged_in: Some(true),
                machine_id: "ksstore".into(),
                machines: vec!["ksstore".into()],
                account_id: Some("agy:3p-weekly:100|gemini-weekly:200".into()),
                connection_ids: vec!["agy-ksstore".into()],
                plan_type: None,
                source_state: "live_probe".into(),
                source_label: "Sonde agy /usage".into(),
                windows: vec![],
                credits_balance: None,
                credits_unlimited: false,
                manual_resets_available: None,
                manual_reset_expirations: Vec::new(),
                observed_at: Some(now),
                detail: None,
            },
        ];
        propagate_known_emails_by_account_id(&mut agy_raw);
        let agy_matrix = build_machine_matrix("monordinateur", &[], &agy_raw, &online);
        let ks_agy = agy_matrix
            .iter()
            .find(|r| r.machine_id == "ksstore")
            .unwrap()
            .antigravity
            .as_ref()
            .unwrap();
        assert_eq!(ks_agy.account.as_deref(), Some("fullflare@gmail.com"));
        let agy_dedup = deduplicate_shared_accounts(agy_raw, "monordinateur");
        assert_eq!(agy_dedup.len(), 1);
        assert_eq!(agy_dedup[0].account_label, "fullflare@gmail.com");
        assert_eq!(agy_dedup[0].machines, vec!["monordinateur", "ksstore"]);
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
                account_label: "fullflare@runbox.com".into(),
                account_email: Some("fullflare@runbox.com".into()),
                logged_in: Some(true),
                machine_id: "cartae".into(),
                machines: vec!["cartae".into()],
                account_id: None,
                connection_ids: vec!["claude-cartae".into()],
                plan_type: Some("Max 20x".into()),
                source_state: "observed".into(),
                source_label: "Observé passivement".into(),
                windows: mat,
                credits_balance: None,
                credits_unlimited: false,
                manual_resets_available: None,
                manual_reset_expirations: Vec::new(),
                observed_at: Some(now - 120),
                detail: None,
            }],
            machines: vec![],
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
        assert!(!html.contains("100 %</span>"));
        assert!(!html.contains("unknown"));
    }

    #[test]
    fn spec351_parse_claude_oauth_usage_and_cedar_ember_resets() {
        let now = parse_iso8601_utc("2026-11-25T07:14:00+00:00").unwrap();
        let payload = json!({
            "five_hour": null,
            "seven_day": {
                "utilization": 4.0,
                "resets_at": "2026-11-30T05:59:59.787228+00:00"
            },
            "cedar_ember": {
                "eligible": true,
                "grants": [
                    {
                        "id": "full-reset-2611",
                        "resets_total": 1,
                        "resets_left": 0,
                        "ends_at": "2026-12-10T22:00:00+00:00",
                        "paused": false
                    },
                    {
                        "id": "launch-2610-fable-6mo",
                        "resets_total": 1,
                        "resets_left": 1,
                        "ends_at": "2027-03-09T07:59:00+00:00",
                        "paused": false
                    }
                ]
            }
        });
        let parsed = parse_claude_oauth_usage_response(&payload, now).unwrap();
        assert_eq!(parsed.manual_resets_available, Some(1));
        assert_eq!(parsed.manual_reset_expirations.len(), 1);
        assert_eq!(
            format_short_date_fr(parsed.manual_reset_expirations[0]),
            "9 mars"
        );

        let mut map = BTreeMap::new();
        for w in parsed.windows {
            map.insert(w.window.clone(), w);
        }
        let (mat, any_live) = materialize_windows(&map, now);
        assert!(any_live);
        let w5 = mat.iter().find(|w| w.label == "5h").unwrap();
        assert_eq!(w5.remaining_percent, Some(100));
        assert_eq!(w5.status, "idle");
        let w7 = mat.iter().find(|w| w.label == "7j").unwrap();
        assert_eq!(w7.remaining_percent, Some(96));

        let subs = vec![SubscriptionQuota {
            id: "claude:account:fullflare@runbox.com".into(),
            engine: "claude".into(),
            provider_label: "Claude".into(),
            account_label: "fullflare@runbox.com".into(),
            account_email: Some("fullflare@runbox.com".into()),
            logged_in: Some(true),
            machine_id: "cartae, ksstore, sysle2".into(),
            machines: vec!["cartae".into(), "ksstore".into(), "sysle2".into()],
            account_id: None,
            connection_ids: vec!["claude-cartae".into()],
            plan_type: Some("Max 20x".into()),
            source_state: "live_probe".into(),
            source_label: "Sonde OAuth Claude (0 jeton)".into(),
            windows: mat,
            credits_balance: None,
            credits_unlimited: false,
            manual_resets_available: parsed.manual_resets_available,
            manual_reset_expirations: parsed.manual_reset_expirations,
            observed_at: Some(now),
            detail: None,
        }];
        let totals = build_provider_totals(&subs);
        assert_eq!(totals.len(), 1);
        assert_eq!(totals[0].remaining_sum, 96);
        assert_eq!(totals[0].capacity_max, 100);

        let report = UsageReport {
            generated_at: now,
            cache_ttl_secs: USAGE_CACHE_TTL_SECS,
            provider_filter: None,
            provider_totals: totals,
            alerts: vec![],
            subscriptions: subs,
            machines: vec![],
            summary: UsageSummary {
                total_subscriptions: 1,
                with_live_quota: 1,
                not_exposed: 0,
                exhausted: 0,
                low_quota_alerts: 0,
            },
        };
        let html = render_html(&report);
        assert!(html.contains("96 %"));
        assert!(html.contains("100 %"));
        assert!(html.contains("inactif"));
        assert!(html.contains("<strong>1</strong> reset"));
        assert!(html.contains("exp. 9 mars"));
        assert!(!html.contains("non exposé"));
    }

    #[test]
    fn incident_routing_quota_from_cache_blocks_exhausted_account_and_scores_healthy() {
        let dir = std::env::temp_dir().join(format!(
            "bridget-usage-routing-{}",
            uuid::Uuid::new_v4().simple()
        ));
        crate::environment::ensure_private_directory(&dir).unwrap();
        let path = dir.join("usage-quotas.json");
        let now = 1_795_769_900_i64;
        let mut cache = UsageCacheFile {
            version: 1,
            entries: BTreeMap::new(),
        };
        cache.entries.insert(
            "codex:monordinateur:default".into(),
            CachedQuotaEntry {
                engine: "codex".into(),
                machine_id: "monordinateur".into(),
                agent_type: "codex-mac-normal".into(),
                connection_id: Some("codex-mac-normal".into()),
                profile_dir: None,
                account_id: Some("82ae4828-611c-4a88-9020-dac5ee4a001c".into()),
                logged_in: Some(true),
                credits_balance: Some("0".into()),
                credits_unlimited: false,
                observed_at: now - 120,
                windows: BTreeMap::from([(
                    "primary/10080m".into(),
                    CachedWindow {
                        window: "primary/10080m".into(),
                        pool: None,
                        used_percent: Some(100),
                        resets_at: Some(now + 180_000),
                        status: "rejected".into(),
                        observed_at: now - 120,
                    },
                )]),
                ..Default::default()
            },
        );
        // Profil frère partageant le même account_id sur monordinateur (codex-mac-pro).
        cache.entries.insert(
            "codex:monordinateur:/Volumes/8TB2/01-workflow/cx-pro".into(),
            CachedQuotaEntry {
                engine: "codex".into(),
                machine_id: "monordinateur".into(),
                agent_type: "codex-pro".into(),
                connection_id: Some("codex-mac-pro".into()),
                profile_dir: Some("/Volumes/8TB2/01-workflow/cx-pro".into()),
                account_id: Some("82ae4828-611c-4a88-9020-dac5ee4a001c".into()),
                logged_in: Some(true),
                credits_balance: Some("0".into()),
                credits_unlimited: false,
                observed_at: now - 200,
                windows: BTreeMap::new(),
                ..Default::default()
            },
        );
        // Compte Codex distinct sur cartae (19 % restants -> 81 % utilisés).
        cache.entries.insert(
            "codex:cartae:default".into(),
            CachedQuotaEntry {
                engine: "codex".into(),
                machine_id: "cartae".into(),
                agent_type: "codex".into(),
                connection_id: Some("codex-cartae-normal".into()),
                profile_dir: None,
                account_id: Some("bfdfeb93-1d22-482e-9230-34e239451d25".into()),
                logged_in: Some(true),
                credits_balance: Some("0".into()),
                credits_unlimited: false,
                observed_at: now - 60,
                windows: BTreeMap::from([(
                    "primary/10080m".into(),
                    CachedWindow {
                        window: "primary/10080m".into(),
                        pool: None,
                        used_percent: Some(81),
                        resets_at: Some(now + 400_000),
                        status: "available".into(),
                        observed_at: now - 60,
                    },
                )]),
                ..Default::default()
            },
        );
        // Claude sain (21 % utilisés -> 79 % restants).
        cache.entries.insert(
            "claude:monordinateur:default:claude-mac".into(),
            CachedQuotaEntry {
                engine: "claude".into(),
                machine_id: "monordinateur".into(),
                agent_type: "claude-mac".into(),
                connection_id: Some("claude-mac".into()),
                logged_in: Some(true),
                observed_at: now - 30,
                windows: BTreeMap::from([(
                    "seven_day".into(),
                    CachedWindow {
                        window: "seven_day".into(),
                        pool: None,
                        used_percent: Some(21),
                        resets_at: Some(now + 240_000),
                        status: "available".into(),
                        observed_at: now - 30,
                    },
                )]),
                ..Default::default()
            },
        );
        save_cache_to(&path, &cache).unwrap();

        let mac_normal = routing_quota_from_cache_at(
            &path,
            "codex-mac-normal",
            "codex",
            "monordinateur",
            None,
            now,
        )
        .expect("codex-mac-normal observé");
        assert!(mac_normal.exhausted && mac_normal.launch_blocked);

        let mac_pro = routing_quota_from_cache_at(
            &path,
            "codex-mac-pro",
            "codex",
            "monordinateur",
            Some("/Volumes/8TB2/01-workflow/cx-pro"),
            now,
        )
        .expect("codex-mac-pro partage le même account_id");
        assert!(mac_pro.launch_blocked);

        let cartae_codex =
            routing_quota_from_cache_at(&path, "codex-cartae-normal", "codex", "cartae", None, now)
                .expect("codex-cartae-normal distinct");
        assert!(!cartae_codex.launch_blocked);
        assert_eq!(cartae_codex.windows[0]["used_percent"], 81);

        let claude =
            routing_quota_from_cache_at(&path, "claude-mac", "claude", "monordinateur", None, now)
                .expect("claude-mac sain");
        assert!(!claude.launch_blocked);
        assert_eq!(claude.windows[0]["used_percent"], 21);

        // Après resets_at, le blocage expire automatiquement.
        assert!(
            routing_quota_from_cache_at(
                &path,
                "codex-mac-normal",
                "codex",
                "monordinateur",
                None,
                now + 180_001,
            )
            .is_none()
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
