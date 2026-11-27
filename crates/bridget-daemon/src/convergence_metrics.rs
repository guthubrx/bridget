//! Métriques durables de convergence et politique de rétention/rollup multi-paliers
//! façon Time Machine (`raw` -> `daily` -> `monthly` -> purge finale, Spec 314).
//!
//! Garanties SQLite :
//! - Toutes les dimensions des clés primaires d'agrégation sont `TEXT NOT NULL DEFAULT ''`
//!   (aucun `NULL` ne casse `ON CONFLICT DO UPDATE`).
//! - L'enregistrement d'une métrique et la persistance de `metrics_recorded = true` sur le
//!   graphe se font dans une seule transaction atomique : zéro double comptage même si les
//!   lignes brutes sont agrégées immédiatement.
//! - Le rollup utilise une sélection exacte par table temporaire de clés dans la transaction
//!   afin que l'ensemble agrégé et l'ensemble supprimé soient strictement identiques.

use bridget_transport::protocol::{ConvergenceMetricsPolicy, ConvergenceMetricsPolicyUpdate};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const MIN_RAW_RETENTION_DAYS: u32 = 1;
pub const MAX_RAW_RETENTION_DAYS: u32 = 365;
pub const MIN_DAILY_RETENTION_DAYS: u32 = 7;
pub const MAX_DAILY_RETENTION_DAYS: u32 = 1825;
pub const MIN_MONTHLY_RETENTION_MONTHS: u32 = 1;
pub const MAX_MONTHLY_RETENTION_MONTHS: u32 = 120;
pub const MIN_GRAPH_RETENTION_DAYS: u32 = 7;
pub const MAX_GRAPH_RETENTION_DAYS: u32 = 365;
pub const MIN_MAX_RAW_ROWS: u32 = 500;
pub const MAX_MAX_RAW_ROWS: u32 = 100_000;
const ROLLUP_BATCH_LIMIT: usize = 2000;
const GRAPH_PURGE_BATCH_LIMIT: usize = 256;

/// Ligne brute enregistrée à la clôture d'une étape convergée.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvergenceMetricRawRow {
    pub graph_id: String,
    pub step_id: String,
    pub project: String,
    pub producer_engine: String,
    pub producer_model: String,
    pub verifier_engine: String,
    pub verifier_model: String,
    pub verifier_orthogonal: bool,
    pub outcome: String,
    pub iterations_used: u32,
    pub tier1_iterations: u32,
    pub tier2_iterations: u32,
    pub tier_escalated: bool,
    pub gate_configured: bool,
    pub gate_runs: u32,
    pub gate_failures: u32,
    pub gate_saved_verifier_calls: u32,
    pub stagnation_saved_iterations: u32,
    pub duration_secs: i64,
    pub finished_at: i64,
}

/// Bilan d'un passage de rollup et purge multi-paliers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConvergenceMaintenanceReport {
    pub raw_rolled_to_daily: usize,
    pub daily_rolled_to_monthly: usize,
    pub monthly_purged: usize,
    pub graphs_purged: usize,
}

pub fn ensure_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS convergence_metrics_policy (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            raw_retention_days INTEGER NOT NULL,
            daily_retention_days INTEGER NOT NULL,
            monthly_retention_months INTEGER NOT NULL,
            graph_retention_days INTEGER NOT NULL,
            max_raw_rows INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS convergence_metrics_raw (
            graph_id TEXT NOT NULL,
            step_id TEXT NOT NULL,
            project TEXT NOT NULL DEFAULT '',
            producer_engine TEXT NOT NULL DEFAULT '',
            producer_model TEXT NOT NULL DEFAULT '',
            verifier_engine TEXT NOT NULL DEFAULT '',
            verifier_model TEXT NOT NULL DEFAULT '',
            verifier_orthogonal INTEGER NOT NULL DEFAULT 0,
            outcome TEXT NOT NULL DEFAULT '',
            iterations_used INTEGER NOT NULL DEFAULT 0,
            tier1_iterations INTEGER NOT NULL DEFAULT 0,
            tier2_iterations INTEGER NOT NULL DEFAULT 0,
            tier_escalated INTEGER NOT NULL DEFAULT 0,
            gate_configured INTEGER NOT NULL DEFAULT 0,
            gate_runs INTEGER NOT NULL DEFAULT 0,
            gate_failures INTEGER NOT NULL DEFAULT 0,
            gate_saved_verifier_calls INTEGER NOT NULL DEFAULT 0,
            stagnation_saved_iterations INTEGER NOT NULL DEFAULT 0,
            duration_secs INTEGER NOT NULL DEFAULT 0,
            finished_at INTEGER NOT NULL,
            PRIMARY KEY (graph_id, step_id)
        );
        CREATE INDEX IF NOT EXISTS idx_conv_metrics_raw_finished
            ON convergence_metrics_raw(finished_at);
        CREATE TABLE IF NOT EXISTS convergence_metrics_daily (
            day_utc TEXT NOT NULL,
            project TEXT NOT NULL DEFAULT '',
            producer_engine TEXT NOT NULL DEFAULT '',
            producer_model TEXT NOT NULL DEFAULT '',
            verifier_engine TEXT NOT NULL DEFAULT '',
            verifier_model TEXT NOT NULL DEFAULT '',
            outcome TEXT NOT NULL DEFAULT '',
            steps_count INTEGER NOT NULL DEFAULT 0,
            approved_count INTEGER NOT NULL DEFAULT 0,
            tier_escalated_count INTEGER NOT NULL DEFAULT 0,
            orthogonal_count INTEGER NOT NULL DEFAULT 0,
            total_iterations INTEGER NOT NULL DEFAULT 0,
            total_tier1_iterations INTEGER NOT NULL DEFAULT 0,
            total_tier2_iterations INTEGER NOT NULL DEFAULT 0,
            total_gate_runs INTEGER NOT NULL DEFAULT 0,
            total_gate_failures INTEGER NOT NULL DEFAULT 0,
            total_gate_saved_verifier_calls INTEGER NOT NULL DEFAULT 0,
            total_stagnation_saved_iterations INTEGER NOT NULL DEFAULT 0,
            total_duration_secs INTEGER NOT NULL DEFAULT 0,
            min_duration_secs INTEGER NOT NULL DEFAULT 0,
            max_duration_secs INTEGER NOT NULL DEFAULT 0,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY (day_utc, project, producer_engine, producer_model, verifier_engine, verifier_model, outcome)
        );
        CREATE INDEX IF NOT EXISTS idx_conv_metrics_daily_day
            ON convergence_metrics_daily(day_utc);
        CREATE TABLE IF NOT EXISTS convergence_metrics_monthly (
            month_utc TEXT NOT NULL,
            project TEXT NOT NULL DEFAULT '',
            producer_engine TEXT NOT NULL DEFAULT '',
            producer_model TEXT NOT NULL DEFAULT '',
            verifier_engine TEXT NOT NULL DEFAULT '',
            verifier_model TEXT NOT NULL DEFAULT '',
            outcome TEXT NOT NULL DEFAULT '',
            steps_count INTEGER NOT NULL DEFAULT 0,
            approved_count INTEGER NOT NULL DEFAULT 0,
            tier_escalated_count INTEGER NOT NULL DEFAULT 0,
            orthogonal_count INTEGER NOT NULL DEFAULT 0,
            total_iterations INTEGER NOT NULL DEFAULT 0,
            total_tier1_iterations INTEGER NOT NULL DEFAULT 0,
            total_tier2_iterations INTEGER NOT NULL DEFAULT 0,
            total_gate_runs INTEGER NOT NULL DEFAULT 0,
            total_gate_failures INTEGER NOT NULL DEFAULT 0,
            total_gate_saved_verifier_calls INTEGER NOT NULL DEFAULT 0,
            total_stagnation_saved_iterations INTEGER NOT NULL DEFAULT 0,
            total_duration_secs INTEGER NOT NULL DEFAULT 0,
            min_duration_secs INTEGER NOT NULL DEFAULT 0,
            max_duration_secs INTEGER NOT NULL DEFAULT 0,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY (month_utc, project, producer_engine, producer_model, verifier_engine, verifier_model, outcome)
        );
        CREATE INDEX IF NOT EXISTS idx_conv_metrics_monthly_month
            ON convergence_metrics_monthly(month_utc);",
    )
}

pub fn validate_policy(policy: &ConvergenceMetricsPolicy) -> Result<(), String> {
    if !(MIN_RAW_RETENTION_DAYS..=MAX_RAW_RETENTION_DAYS).contains(&policy.raw_retention_days) {
        return Err(format!(
            "raw_retention_days doit être compris entre {MIN_RAW_RETENTION_DAYS} et {MAX_RAW_RETENTION_DAYS}."
        ));
    }
    if !(MIN_DAILY_RETENTION_DAYS..=MAX_DAILY_RETENTION_DAYS).contains(&policy.daily_retention_days)
    {
        return Err(format!(
            "daily_retention_days doit être compris entre {MIN_DAILY_RETENTION_DAYS} et {MAX_DAILY_RETENTION_DAYS}."
        ));
    }
    if !(MIN_MONTHLY_RETENTION_MONTHS..=MAX_MONTHLY_RETENTION_MONTHS)
        .contains(&policy.monthly_retention_months)
    {
        return Err(format!(
            "monthly_retention_months doit être compris entre {MIN_MONTHLY_RETENTION_MONTHS} et {MAX_MONTHLY_RETENTION_MONTHS}."
        ));
    }
    if !(MIN_GRAPH_RETENTION_DAYS..=MAX_GRAPH_RETENTION_DAYS).contains(&policy.graph_retention_days)
    {
        return Err(format!(
            "graph_retention_days doit être compris entre {MIN_GRAPH_RETENTION_DAYS} et {MAX_GRAPH_RETENTION_DAYS}."
        ));
    }
    if !(MIN_MAX_RAW_ROWS..=MAX_MAX_RAW_ROWS).contains(&policy.max_raw_rows) {
        return Err(format!(
            "max_raw_rows doit être compris entre {MIN_MAX_RAW_ROWS} et {MAX_MAX_RAW_ROWS}."
        ));
    }
    if policy.daily_retention_days < policy.raw_retention_days {
        return Err(format!(
            "Invariant violé : daily_retention_days ({}) doit être >= raw_retention_days ({}).",
            policy.daily_retention_days, policy.raw_retention_days
        ));
    }
    if policy.monthly_retention_months.saturating_mul(31) <= policy.daily_retention_days {
        return Err(format!(
            "Invariant violé : monthly_retention_months * 31 ({}) doit être strictement supérieur à daily_retention_days ({}).",
            policy.monthly_retention_months.saturating_mul(31),
            policy.daily_retention_days
        ));
    }
    Ok(())
}

pub fn load_policy(conn: &Connection) -> Result<ConvergenceMetricsPolicy, String> {
    let row = conn
        .query_row(
            "SELECT raw_retention_days, daily_retention_days, monthly_retention_months, graph_retention_days, max_raw_rows, updated_at
             FROM convergence_metrics_policy WHERE id = 1",
            [],
            |r| {
                Ok(ConvergenceMetricsPolicy {
                    raw_retention_days: r.get(0)?,
                    daily_retention_days: r.get(1)?,
                    monthly_retention_months: r.get(2)?,
                    graph_retention_days: r.get(3)?,
                    max_raw_rows: r.get(4)?,
                    updated_at: r.get(5)?,
                })
            },
        )
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(row.unwrap_or_default())
}

pub fn update_policy(
    conn: &Connection,
    update: &ConvergenceMetricsPolicyUpdate,
    now: i64,
) -> Result<ConvergenceMetricsPolicy, String> {
    let mut policy = load_policy(conn)?;
    if let Some(v) = update.raw_retention_days {
        policy.raw_retention_days = v;
    }
    if let Some(v) = update.daily_retention_days {
        policy.daily_retention_days = v;
    }
    if let Some(v) = update.monthly_retention_months {
        policy.monthly_retention_months = v;
    }
    if let Some(v) = update.graph_retention_days {
        policy.graph_retention_days = v;
    }
    if let Some(v) = update.max_raw_rows {
        policy.max_raw_rows = v;
    }
    policy.updated_at = now;
    validate_policy(&policy)?;
    conn.execute(
        "INSERT INTO convergence_metrics_policy
            (id, raw_retention_days, daily_retention_days, monthly_retention_months, graph_retention_days, max_raw_rows, updated_at)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET
            raw_retention_days = excluded.raw_retention_days,
            daily_retention_days = excluded.daily_retention_days,
            monthly_retention_months = excluded.monthly_retention_months,
            graph_retention_days = excluded.graph_retention_days,
            max_raw_rows = excluded.max_raw_rows,
            updated_at = excluded.updated_at",
        params![
            policy.raw_retention_days,
            policy.daily_retention_days,
            policy.monthly_retention_months,
            policy.graph_retention_days,
            policy.max_raw_rows,
            policy.updated_at,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(policy)
}

/// Insère une ligne brute dans la transaction courante (appelé atomiquement avec `save_graph`).
pub fn insert_raw_in_tx(conn: &Connection, row: &ConvergenceMetricRawRow) -> Result<(), String> {
    let duration = row.duration_secs.max(0);
    conn.execute(
        "INSERT OR IGNORE INTO convergence_metrics_raw (
            graph_id, step_id, project, producer_engine, producer_model,
            verifier_engine, verifier_model, verifier_orthogonal, outcome,
            iterations_used, tier1_iterations, tier2_iterations, tier_escalated,
            gate_configured, gate_runs, gate_failures, gate_saved_verifier_calls,
            stagnation_saved_iterations, duration_secs, finished_at
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5,
            ?6, ?7, ?8, ?9,
            ?10, ?11, ?12, ?13,
            ?14, ?15, ?16, ?17,
            ?18, ?19, ?20
         )",
        params![
            row.graph_id,
            row.step_id,
            row.project,
            row.producer_engine,
            row.producer_model,
            row.verifier_engine,
            row.verifier_model,
            i32::from(row.verifier_orthogonal),
            row.outcome,
            row.iterations_used,
            row.tier1_iterations,
            row.tier2_iterations,
            i32::from(row.tier_escalated),
            i32::from(row.gate_configured),
            row.gate_runs,
            row.gate_failures,
            row.gate_saved_verifier_calls,
            row.stagnation_saved_iterations,
            duration,
            row.finished_at,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Convertit un horodatage Unix (secondes UTC) en `(YYYY-MM-DD, YYYY-MM)` sans dépendance externe.
pub fn unix_to_utc_day_month(ts: i64) -> (String, String) {
    let days = ts.div_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (format!("{y:04}-{m:02}-{d:02}"), format!("{y:04}-{m:02}"))
}

fn subtract_months_utc(ts: i64, months: u32) -> String {
    let (_, ym) = unix_to_utc_day_month(ts);
    let mut parts = ym.split('-');
    let year: i64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(1970);
    let month: i64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    let total = year * 12 + (month - 1) - i64::from(months);
    let cut_y = total.div_euclid(12);
    let cut_m = total.rem_euclid(12) + 1;
    format!("{cut_y:04}-{cut_m:02}")
}

/// Exécute en une transaction SQLite atomique le rollup multi-paliers façon Time Machine :
/// 1) `raw -> daily` (âge > `raw_retention_days` ou surplus au-delà de `max_raw_rows`),
/// 2) `daily -> monthly` (âge > `daily_retention_days`),
/// 3) purge `monthly` (âge > `monthly_retention_months`),
/// 4) purge des graphes terminaux anciens (`notice_sent = 1`, âge > `graph_retention_days`) + leurs liaisons.
pub fn run_rollup_and_purge(
    conn: &mut Connection,
    now: i64,
) -> Result<ConvergenceMaintenanceReport, String> {
    let policy = load_policy(conn)?;
    let raw_cutoff = now.saturating_sub(i64::from(policy.raw_retention_days) * 86_400);
    let (daily_cutoff_day, _) =
        unix_to_utc_day_month(now.saturating_sub(i64::from(policy.daily_retention_days) * 86_400));
    let monthly_cutoff = subtract_months_utc(now, policy.monthly_retention_months);
    let graph_cutoff = now.saturating_sub(i64::from(policy.graph_retention_days) * 86_400);

    let tx = conn.transaction().map_err(|e| e.to_string())?;
    let mut report = ConvergenceMaintenanceReport::default();

    // 1) Sélection exacte des lignes brutes à agréger dans une table temporaire de rowids.
    tx.execute_batch(
        "CREATE TEMP TABLE IF NOT EXISTS _conv_raw_batch(rid INTEGER PRIMARY KEY);
         DELETE FROM _conv_raw_batch;",
    )
    .map_err(|e| e.to_string())?;

    // Par âge
    tx.execute(
        "INSERT OR IGNORE INTO _conv_raw_batch(rid)
         SELECT rowid FROM convergence_metrics_raw
         WHERE finished_at < ?1
         ORDER BY finished_at ASC
         LIMIT ?2",
        params![raw_cutoff, ROLLUP_BATCH_LIMIT as i64],
    )
    .map_err(|e| e.to_string())?;

    // Par volume (`max_raw_rows`) sur les lignes restantes
    let remaining_raw: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM convergence_metrics_raw
             WHERE rowid NOT IN (SELECT rid FROM _conv_raw_batch)",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if remaining_raw > i64::from(policy.max_raw_rows) {
        let excess =
            (remaining_raw - i64::from(policy.max_raw_rows)).min(ROLLUP_BATCH_LIMIT as i64);
        tx.execute(
            "INSERT OR IGNORE INTO _conv_raw_batch(rid)
             SELECT rowid FROM convergence_metrics_raw
             WHERE rowid NOT IN (SELECT rid FROM _conv_raw_batch)
             ORDER BY finished_at ASC
             LIMIT ?1",
            params![excess],
        )
        .map_err(|e| e.to_string())?;
    }

    let raw_count: i64 = tx
        .query_row("SELECT COUNT(*) FROM _conv_raw_batch", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if raw_count > 0 {
        tx.execute(
            "INSERT INTO convergence_metrics_daily (
                day_utc, project, producer_engine, producer_model, verifier_engine, verifier_model, outcome,
                steps_count, approved_count, tier_escalated_count, orthogonal_count,
                total_iterations, total_tier1_iterations, total_tier2_iterations,
                total_gate_runs, total_gate_failures, total_gate_saved_verifier_calls,
                total_stagnation_saved_iterations, total_duration_secs, min_duration_secs, max_duration_secs, updated_at
             )
             SELECT
                strftime('%Y-%m-%d', finished_at, 'unixepoch') AS day_utc,
                project, producer_engine, producer_model, verifier_engine, verifier_model, outcome,
                COUNT(*),
                SUM(CASE WHEN outcome = 'approved' THEN 1 ELSE 0 END),
                SUM(tier_escalated),
                SUM(verifier_orthogonal),
                SUM(iterations_used),
                SUM(tier1_iterations),
                SUM(tier2_iterations),
                SUM(gate_runs),
                SUM(gate_failures),
                SUM(gate_saved_verifier_calls),
                SUM(stagnation_saved_iterations),
                SUM(duration_secs),
                MIN(duration_secs),
                MAX(duration_secs),
                ?1
             FROM convergence_metrics_raw
             WHERE rowid IN (SELECT rid FROM _conv_raw_batch)
             GROUP BY day_utc, project, producer_engine, producer_model, verifier_engine, verifier_model, outcome
             ON CONFLICT(day_utc, project, producer_engine, producer_model, verifier_engine, verifier_model, outcome)
             DO UPDATE SET
                steps_count = steps_count + excluded.steps_count,
                approved_count = approved_count + excluded.approved_count,
                tier_escalated_count = tier_escalated_count + excluded.tier_escalated_count,
                orthogonal_count = orthogonal_count + excluded.orthogonal_count,
                total_iterations = total_iterations + excluded.total_iterations,
                total_tier1_iterations = total_tier1_iterations + excluded.total_tier1_iterations,
                total_tier2_iterations = total_tier2_iterations + excluded.total_tier2_iterations,
                total_gate_runs = total_gate_runs + excluded.total_gate_runs,
                total_gate_failures = total_gate_failures + excluded.total_gate_failures,
                total_gate_saved_verifier_calls = total_gate_saved_verifier_calls + excluded.total_gate_saved_verifier_calls,
                total_stagnation_saved_iterations = total_stagnation_saved_iterations + excluded.total_stagnation_saved_iterations,
                total_duration_secs = total_duration_secs + excluded.total_duration_secs,
                min_duration_secs = MIN(min_duration_secs, excluded.min_duration_secs),
                max_duration_secs = MAX(max_duration_secs, excluded.max_duration_secs),
                updated_at = excluded.updated_at",
            params![now],
        )
        .map_err(|e| e.to_string())?;
        let deleted = tx
            .execute(
                "DELETE FROM convergence_metrics_raw WHERE rowid IN (SELECT rid FROM _conv_raw_batch)",
                [],
            )
            .map_err(|e| e.to_string())?;
        report.raw_rolled_to_daily = deleted;
    }

    // 2) Rollup exact `daily -> monthly` via table temporaire de rowids.
    tx.execute_batch(
        "CREATE TEMP TABLE IF NOT EXISTS _conv_daily_batch(rid INTEGER PRIMARY KEY);
         DELETE FROM _conv_daily_batch;",
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO _conv_daily_batch(rid)
         SELECT rowid FROM convergence_metrics_daily
         WHERE day_utc < ?1
         ORDER BY day_utc ASC
         LIMIT ?2",
        params![daily_cutoff_day, ROLLUP_BATCH_LIMIT as i64],
    )
    .map_err(|e| e.to_string())?;
    let daily_count: i64 = tx
        .query_row("SELECT COUNT(*) FROM _conv_daily_batch", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    if daily_count > 0 {
        tx.execute(
            "INSERT INTO convergence_metrics_monthly (
                month_utc, project, producer_engine, producer_model, verifier_engine, verifier_model, outcome,
                steps_count, approved_count, tier_escalated_count, orthogonal_count,
                total_iterations, total_tier1_iterations, total_tier2_iterations,
                total_gate_runs, total_gate_failures, total_gate_saved_verifier_calls,
                total_stagnation_saved_iterations, total_duration_secs, min_duration_secs, max_duration_secs, updated_at
             )
             SELECT
                substr(day_utc, 1, 7) AS month_utc,
                project, producer_engine, producer_model, verifier_engine, verifier_model, outcome,
                SUM(steps_count),
                SUM(approved_count),
                SUM(tier_escalated_count),
                SUM(orthogonal_count),
                SUM(total_iterations),
                SUM(total_tier1_iterations),
                SUM(total_tier2_iterations),
                SUM(total_gate_runs),
                SUM(total_gate_failures),
                SUM(total_gate_saved_verifier_calls),
                SUM(total_stagnation_saved_iterations),
                SUM(total_duration_secs),
                MIN(min_duration_secs),
                MAX(max_duration_secs),
                ?1
             FROM convergence_metrics_daily
             WHERE rowid IN (SELECT rid FROM _conv_daily_batch)
             GROUP BY month_utc, project, producer_engine, producer_model, verifier_engine, verifier_model, outcome
             ON CONFLICT(month_utc, project, producer_engine, producer_model, verifier_engine, verifier_model, outcome)
             DO UPDATE SET
                steps_count = steps_count + excluded.steps_count,
                approved_count = approved_count + excluded.approved_count,
                tier_escalated_count = tier_escalated_count + excluded.tier_escalated_count,
                orthogonal_count = orthogonal_count + excluded.orthogonal_count,
                total_iterations = total_iterations + excluded.total_iterations,
                total_tier1_iterations = total_tier1_iterations + excluded.total_tier1_iterations,
                total_tier2_iterations = total_tier2_iterations + excluded.total_tier2_iterations,
                total_gate_runs = total_gate_runs + excluded.total_gate_runs,
                total_gate_failures = total_gate_failures + excluded.total_gate_failures,
                total_gate_saved_verifier_calls = total_gate_saved_verifier_calls + excluded.total_gate_saved_verifier_calls,
                total_stagnation_saved_iterations = total_stagnation_saved_iterations + excluded.total_stagnation_saved_iterations,
                total_duration_secs = total_duration_secs + excluded.total_duration_secs,
                min_duration_secs = MIN(min_duration_secs, excluded.min_duration_secs),
                max_duration_secs = MAX(max_duration_secs, excluded.max_duration_secs),
                updated_at = excluded.updated_at",
            params![now],
        )
        .map_err(|e| e.to_string())?;
        let deleted = tx
            .execute(
                "DELETE FROM convergence_metrics_daily WHERE rowid IN (SELECT rid FROM _conv_daily_batch)",
                [],
            )
            .map_err(|e| e.to_string())?;
        report.daily_rolled_to_monthly = deleted;
    }

    // 3) Purge des agrégats mensuels expirés.
    report.monthly_purged = tx
        .execute(
            "DELETE FROM convergence_metrics_monthly WHERE month_utc < ?1",
            params![monthly_cutoff],
        )
        .map_err(|e| e.to_string())?;

    // 4) Purge des graphes terminaux anciens dont l'avis parent est remis (`notice_sent = 1`)
    //    et de leurs liaisons `native_delegation_graph_tasks` dans la même transaction.
    tx.execute_batch(
        "CREATE TEMP TABLE IF NOT EXISTS _conv_graph_purge(graph_id TEXT PRIMARY KEY);
         DELETE FROM _conv_graph_purge;",
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "INSERT INTO _conv_graph_purge(graph_id)
         SELECT graph_id FROM native_delegation_graphs
         WHERE state != 'running'
           AND updated_at < ?1
           AND json_extract(payload, '$.notice_sent') = 1
         ORDER BY updated_at ASC
         LIMIT ?2",
        params![graph_cutoff, GRAPH_PURGE_BATCH_LIMIT as i64],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        "DELETE FROM native_delegation_graph_tasks
         WHERE graph_id IN (SELECT graph_id FROM _conv_graph_purge)",
        [],
    )
    .map_err(|e| e.to_string())?;
    report.graphs_purged = tx
        .execute(
            "DELETE FROM native_delegation_graphs
             WHERE graph_id IN (SELECT graph_id FROM _conv_graph_purge)",
            [],
        )
        .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(report)
}

/// Agrège `raw` + `daily` + `monthly` sur la fenêtre demandée (`window_days`, défaut 90 j).
pub fn query_metrics_report(
    conn: &Connection,
    project: Option<&str>,
    window_days: Option<u32>,
    now: i64,
) -> Result<Value, String> {
    let policy = load_policy(conn)?;
    let window = window_days.unwrap_or(90).clamp(1, 3650);
    let since_ts = now.saturating_sub(i64::from(window) * 86_400);
    let (since_day, since_month) = unix_to_utc_day_month(since_ts);
    let proj_filter = project
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("");

    let sql = "
        WITH unified AS (
            SELECT
                project, producer_engine, producer_model, verifier_engine, verifier_model, outcome,
                1 AS steps_count,
                CASE WHEN outcome = 'approved' THEN 1 ELSE 0 END AS approved_count,
                tier_escalated AS tier_escalated_count,
                verifier_orthogonal AS orthogonal_count,
                iterations_used AS total_iterations,
                tier1_iterations AS total_tier1_iterations,
                tier2_iterations AS total_tier2_iterations,
                gate_runs AS total_gate_runs,
                gate_failures AS total_gate_failures,
                gate_saved_verifier_calls AS total_gate_saved_verifier_calls,
                stagnation_saved_iterations AS total_stagnation_saved_iterations,
                duration_secs AS total_duration_secs
            FROM convergence_metrics_raw
            WHERE finished_at >= ?1 AND (?4 = '' OR project = ?4)
            UNION ALL
            SELECT
                project, producer_engine, producer_model, verifier_engine, verifier_model, outcome,
                steps_count, approved_count, tier_escalated_count, orthogonal_count,
                total_iterations, total_tier1_iterations, total_tier2_iterations,
                total_gate_runs, total_gate_failures, total_gate_saved_verifier_calls,
                total_stagnation_saved_iterations, total_duration_secs
            FROM convergence_metrics_daily
            WHERE day_utc >= ?2 AND (?4 = '' OR project = ?4)
            UNION ALL
            SELECT
                project, producer_engine, producer_model, verifier_engine, verifier_model, outcome,
                steps_count, approved_count, tier_escalated_count, orthogonal_count,
                total_iterations, total_tier1_iterations, total_tier2_iterations,
                total_gate_runs, total_gate_failures, total_gate_saved_verifier_calls,
                total_stagnation_saved_iterations, total_duration_secs
            FROM convergence_metrics_monthly
            WHERE month_utc >= ?3 AND (?4 = '' OR project = ?4)
        )
        SELECT
            COALESCE(SUM(steps_count), 0),
            COALESCE(SUM(approved_count), 0),
            COALESCE(SUM(tier_escalated_count), 0),
            COALESCE(SUM(orthogonal_count), 0),
            COALESCE(SUM(total_iterations), 0),
            COALESCE(SUM(total_gate_runs), 0),
            COALESCE(SUM(total_gate_failures), 0),
            COALESCE(SUM(total_gate_saved_verifier_calls), 0),
            COALESCE(SUM(total_stagnation_saved_iterations), 0),
            COALESCE(SUM(total_duration_secs), 0)
        FROM unified";

    let (
        steps_count,
        approved_count,
        tier_escalated_count,
        orthogonal_count,
        total_iterations,
        total_gate_runs,
        total_gate_failures,
        total_gate_saved_verifier_calls,
        total_stagnation_saved_iterations,
        total_duration_secs,
    ): (i64, i64, i64, i64, i64, i64, i64, i64, i64, i64) = conn
        .query_row(
            sql,
            params![since_ts, since_day, since_month, proj_filter],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                    r.get(7)?,
                    r.get(8)?,
                    r.get(9)?,
                ))
            },
        )
        .map_err(|e| e.to_string())?;

    let mut by_pair_stmt = conn
        .prepare(
            "WITH unified AS (
                SELECT producer_engine, producer_model, verifier_engine, verifier_model,
                       1 AS steps_count,
                       CASE WHEN outcome = 'approved' THEN 1 ELSE 0 END AS approved_count,
                       tier_escalated AS tier_escalated_count,
                       iterations_used AS total_iterations,
                       gate_saved_verifier_calls AS total_gate_saved,
                       stagnation_saved_iterations AS total_stag_saved
                FROM convergence_metrics_raw
                WHERE finished_at >= ?1 AND (?4 = '' OR project = ?4)
                UNION ALL
                SELECT producer_engine, producer_model, verifier_engine, verifier_model,
                       steps_count, approved_count, tier_escalated_count, total_iterations,
                       total_gate_saved_verifier_calls, total_stagnation_saved_iterations
                FROM convergence_metrics_daily
                WHERE day_utc >= ?2 AND (?4 = '' OR project = ?4)
                UNION ALL
                SELECT producer_engine, producer_model, verifier_engine, verifier_model,
                       steps_count, approved_count, tier_escalated_count, total_iterations,
                       total_gate_saved_verifier_calls, total_stagnation_saved_iterations
                FROM convergence_metrics_monthly
                WHERE month_utc >= ?3 AND (?4 = '' OR project = ?4)
            )
            SELECT producer_engine, producer_model, verifier_engine, verifier_model,
                   SUM(steps_count) AS n,
                   SUM(approved_count) AS ok,
                   SUM(tier_escalated_count) AS esc,
                   SUM(total_iterations) AS iters,
                   SUM(total_gate_saved) AS gate_saved,
                   SUM(total_stag_saved) AS stag_saved
            FROM unified
            GROUP BY producer_engine, producer_model, verifier_engine, verifier_model
            ORDER BY n DESC
            LIMIT 32",
        )
        .map_err(|e| e.to_string())?;
    let pairs: Vec<Value> = by_pair_stmt
        .query_map(
            params![since_ts, since_day, since_month, proj_filter],
            |r| {
                let n: i64 = r.get(4)?;
                let ok: i64 = r.get(5)?;
                let iters: i64 = r.get(7)?;
                Ok(json!({
                    "producer_engine": r.get::<_, String>(0)?,
                    "producer_model": r.get::<_, String>(1)?,
                    "verifier_engine": r.get::<_, String>(2)?,
                    "verifier_model": r.get::<_, String>(3)?,
                    "steps": n,
                    "approved": ok,
                    "approval_rate_pct": if n > 0 { (ok * 100) / n } else { 0 },
                    "tier_escalated": r.get::<_, i64>(6)?,
                    "avg_iterations_x10": if n > 0 { (iters * 10) / n } else { 0 },
                    "gate_saved_verifier_calls": r.get::<_, i64>(8)?,
                    "stagnation_saved_iterations": r.get::<_, i64>(9)?,
                }))
            },
        )
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut by_outcome_stmt = conn
        .prepare(
            "WITH unified AS (
                SELECT outcome, 1 AS steps_count FROM convergence_metrics_raw
                WHERE finished_at >= ?1 AND (?4 = '' OR project = ?4)
                UNION ALL
                SELECT outcome, steps_count FROM convergence_metrics_daily
                WHERE day_utc >= ?2 AND (?4 = '' OR project = ?4)
                UNION ALL
                SELECT outcome, steps_count FROM convergence_metrics_monthly
                WHERE month_utc >= ?3 AND (?4 = '' OR project = ?4)
            )
            SELECT outcome, SUM(steps_count) AS n FROM unified GROUP BY outcome ORDER BY n DESC",
        )
        .map_err(|e| e.to_string())?;
    let mut outcomes = serde_json::Map::new();
    for row in by_outcome_stmt
        .query_map(
            params![since_ts, since_day, since_month, proj_filter],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)),
        )
        .map_err(|e| e.to_string())?
    {
        let (k, v) = row.map_err(|e| e.to_string())?;
        outcomes.insert(k, json!(v));
    }

    let raw_rows: i64 = conn
        .query_row("SELECT COUNT(*) FROM convergence_metrics_raw", [], |r| {
            r.get(0)
        })
        .unwrap_or(0);
    let daily_rows: i64 = conn
        .query_row("SELECT COUNT(*) FROM convergence_metrics_daily", [], |r| {
            r.get(0)
        })
        .unwrap_or(0);
    let monthly_rows: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM convergence_metrics_monthly",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    Ok(json!({
        "version": 1,
        "status": "ok",
        "window_days": window,
        "project": if proj_filter.is_empty() { Value::Null } else { json!(proj_filter) },
        "totals": {
            "steps": steps_count,
            "approved": approved_count,
            "approval_rate_pct": if steps_count > 0 { (approved_count * 100) / steps_count } else { 0 },
            "tier_escalated": tier_escalated_count,
            "verifier_orthogonal": orthogonal_count,
            "total_iterations": total_iterations,
            "gate_runs": total_gate_runs,
            "gate_failures": total_gate_failures,
            "gate_saved_verifier_calls": total_gate_saved_verifier_calls,
            "stagnation_saved_iterations": total_stagnation_saved_iterations,
            "total_duration_secs": total_duration_secs,
        },
        "outcomes": outcomes,
        "by_model_pair": pairs,
        "storage": {
            "raw_rows": raw_rows,
            "daily_rows": daily_rows,
            "monthly_rows": monthly_rows,
        },
        "policy": policy,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE native_delegation_graphs (
                graph_id TEXT PRIMARY KEY,
                owner TEXT NOT NULL,
                request_id TEXT NOT NULL,
                state TEXT NOT NULL,
                updated_at INTEGER NOT NULL,
                payload TEXT NOT NULL
            );
            CREATE TABLE native_delegation_graph_tasks (
                task_id TEXT PRIMARY KEY,
                graph_id TEXT NOT NULL,
                step_id TEXT NOT NULL,
                suppress_step_notices INTEGER NOT NULL DEFAULT 1
            );",
        )
        .unwrap();
        ensure_schema(&conn).unwrap();
        conn
    }

    fn sample_raw(
        graph_id: &str,
        step_id: &str,
        finished_at: i64,
        duration: i64,
    ) -> ConvergenceMetricRawRow {
        ConvergenceMetricRawRow {
            graph_id: graph_id.into(),
            step_id: step_id.into(),
            project: "64.bridget".into(),
            producer_engine: "codex".into(),
            producer_model: "gpt-5.4".into(),
            verifier_engine: "".into(),
            verifier_model: "".into(),
            verifier_orthogonal: false,
            outcome: "approved".into(),
            iterations_used: 2,
            tier1_iterations: 2,
            tier2_iterations: 0,
            tier_escalated: false,
            gate_configured: true,
            gate_runs: 2,
            gate_failures: 1,
            gate_saved_verifier_calls: 1,
            stagnation_saved_iterations: 0,
            duration_secs: duration,
            finished_at,
        }
    }

    #[test]
    fn spec314_policy_validation_enforces_time_machine_invariants() {
        let mut p = ConvergenceMetricsPolicy::default();
        assert!(validate_policy(&p).is_ok());

        p.daily_retention_days = 10;
        p.raw_retention_days = 30;
        assert!(validate_policy(&p).is_err());

        p.raw_retention_days = 10;
        p.daily_retention_days = 180;
        p.monthly_retention_months = 2; // 62 <= 180
        assert!(validate_policy(&p).is_err());

        p.monthly_retention_months = 12;
        p.graph_retention_days = 3; // < 7
        assert!(validate_policy(&p).is_err());
    }

    #[test]
    fn spec314_time_machine_rollup_conserves_sums_min_max_and_purges_graphs() {
        let mut conn = mem_conn();
        let now = 1_795_800_000_i64; // ~2026-11-27
        let old_raw = now - 45 * 86_400; // > 30 j
        let very_old_raw = now - 220 * 86_400; // > 180 j
        let ancient_raw = now - 800 * 86_400; // > 24 mois

        // Deux lignes le même jour avec verifier vide (teste NOT NULL DEFAULT '' + MIN/MAX)
        insert_raw_in_tx(&conn, &sample_raw("g1", "s1", old_raw, 40)).unwrap();
        insert_raw_in_tx(&conn, &sample_raw("g2", "s1", old_raw + 60, 120)).unwrap();
        insert_raw_in_tx(&conn, &sample_raw("g3", "s1", very_old_raw, 90)).unwrap();
        insert_raw_in_tx(&conn, &sample_raw("g4", "s1", ancient_raw, 50)).unwrap();
        // Une ligne récente qui doit rester en raw
        insert_raw_in_tx(&conn, &sample_raw("g5", "s1", now - 3600, 30)).unwrap();

        // Deux graphes anciens : un avec notice_sent=1 (purgé), un avec notice_sent=0 (conservé)
        conn.execute(
            "INSERT INTO native_delegation_graphs VALUES ('g-old-done','o','r1','completed',?1,'{\"notice_sent\":true}')",
            params![now - 40 * 86_400],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO native_delegation_graph_tasks VALUES ('t-old','g-old-done','s1',1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO native_delegation_graphs VALUES ('g-old-unnotified','o','r2','completed',?1,'{\"notice_sent\":false}')",
            params![now - 40 * 86_400],
        )
        .unwrap();

        let rep = run_rollup_and_purge(&mut conn, now).unwrap();
        assert_eq!(rep.raw_rolled_to_daily, 4);
        assert_eq!(rep.daily_rolled_to_monthly, 2); // very_old + ancient
        assert_eq!(rep.monthly_purged, 1); // ancient > 24 mois
        assert_eq!(rep.graphs_purged, 1);

        // Vérifier conservation exacte sur la journée old_raw (2 étapes, min=40, max=120, sum=160)
        let (steps, min_d, max_d, sum_d): (i64, i64, i64, i64) = conn
            .query_row(
                "SELECT steps_count, min_duration_secs, max_duration_secs, total_duration_secs
                 FROM convergence_metrics_daily",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        assert_eq!((steps, min_d, max_d, sum_d), (2, 40, 120, 160));

        // Vérifier que les tâches du graphe purgé ont bien été retirées et que g-old-unnotified reste
        let task_left: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM native_delegation_graph_tasks",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(task_left, 0);
        let graphs_left: i64 = conn
            .query_row("SELECT COUNT(*) FROM native_delegation_graphs", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(graphs_left, 1);

        // Vérifier fusion multi-paliers sur 365 jours (1 raw + 2 daily + 1 monthly = 4 étapes)
        let view = query_metrics_report(&conn, Some("64.bridget"), Some(365), now).unwrap();
        assert_eq!(view["totals"]["steps"], 4);
        assert_eq!(view["totals"]["gate_saved_verifier_calls"], 4);
        assert_eq!(view["storage"]["raw_rows"], 1);
        assert_eq!(view["storage"]["daily_rows"], 1);
        assert_eq!(view["storage"]["monthly_rows"], 1);

        // Idempotence : un second passage immédiat ne double ni ne purge rien de plus
        let rep2 = run_rollup_and_purge(&mut conn, now).unwrap();
        assert_eq!(rep2, ConvergenceMaintenanceReport::default());
    }
}
