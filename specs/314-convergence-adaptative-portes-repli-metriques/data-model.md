# Modèle de données — Spec 314

## 1. Extensions protocole (`crates/bridget-transport/src/protocol.rs`)

### `GraphDeterministicGate`
| Champ | Type | Contrainte / Défaut |
|---|---|---|
| `argv` | `Vec<String>` | `1..=32` éléments, `argv[0]` non vide, max 1024 octets/arg, aucun caractère de contrôle/NUL |
| `timeout_secs` | `u64` | `5..=300`, défaut `120` |

### `GraphConvergenceEscalation`
| Champ | Type | Contrainte / Défaut |
|---|---|---|
| `engine` | `Option<String>` | Optionnel ; au moins un parmi `engine`, `model`, `effort` requis |
| `model` | `Option<String>` | Optionnel |
| `effort` | `Option<String>` | Optionnel |
| `machine_id` | `Option<String>` | Optionnel |
| `extra_iterations` | `u8` | `1..=3`, défaut `2` ; `max_iterations + extra_iterations <= 8` |
| `on_stagnation` | `bool` | Défaut `true` |
| `on_max_iterations` | `bool` | Défaut `true` |

### `GraphStepCondition` (champs plats `when` + `when_outcomes` sur `DelegateGraphStep`)
- `when`: `"all_succeeded"` (défaut) | `"any_failed"` | `"on_outcome"`
- `when_outcomes`: `Vec<String>` — obligatoire et non vide uniquement si `when == "on_outcome"`, doit être vide sinon. Valeurs autorisées : `failed`, `stagnated`, `max_iterations_reached`, `escalated`, `gate_unavailable`, `verdict_unparseable` (jamais `cancelled`).

## 2. Extensions état durable (`NativeGraphConvergenceState` / `NativeGraphConvergenceIteration`)

- Sur `NativeGraphConvergenceIteration` (`#[serde(default)]`) :
  - `tier: u8` (`0` = initial, `1` = escaladé)
  - `producer_engine: Option<String>`, `producer_model: Option<String>`
  - `verifier_engine: Option<String>`, `verifier_model: Option<String>`
  - `verifier_orthogonal: Option<bool>`
  - `gate_passed: Option<bool>`, `gate_exit_code: Option<i32>`, `gate_timed_out: bool`, `gate_duration_ms: Option<u64>`, `gate_output_excerpt: Option<String>`
- Sur `NativeGraphConvergenceState` (`#[serde(default)]`) :
  - `escalation_tier: u8` (`0` ou `1`)
  - `tier_escalated: bool` (distinct de `outcome == "escalated"` qui désigne le verdict humain du vérificateur)
  - `total_max_iterations: u8` (initialement `max_iterations`, augmenté de `extra_iterations` à l'escalade, `<= 8`)
  - `escalated_from_reason: Option<String>`
  - `prior_producer_task_ids: Vec<String>` (suivi pour annulation complète)
  - `seen_commits: Vec<String>` (réinitialisé lors du passage palier 0 → palier 1)
  - `seen_blocking_fingerprints: Vec<String>` (réinitialisé lors du passage palier 0 → palier 1)
  - `gate_runs: u32`, `gate_failures: u32`, `gate_saved_verifier_calls: u32`, `stagnation_saved_iterations: u32`
  - `metrics_recorded: bool` (écrit atomiquement dans la même transaction SQLite que `convergence_metrics_raw`)

## 3. Tables SQLite métriques & rétention Time-Machine (`bridget.db`)

### `convergence_metrics_policy` (singleton `id = 1`)
| Colonne | Type | Défaut | Bornes / Invariants |
|---|---|---|---|
| `id` | `INTEGER PRIMARY KEY CHECK(id = 1)` | `1` | fixe |
| `raw_retention_days` | `INTEGER NOT NULL` | `30` | `1..=365` |
| `daily_retention_days` | `INTEGER NOT NULL` | `180` | `7..=1825`, `>= raw_retention_days` |
| `monthly_retention_months` | `INTEGER NOT NULL` | `24` | `1..=120`, `monthly_retention_months * 31 > daily_retention_days` |
| `graph_retention_days` | `INTEGER NOT NULL` | `30` | `7..=365` (purge uniquement terminaux avec `notice_sent = 1`) |
| `max_raw_rows` | `INTEGER NOT NULL` | `10000` | `500..=100000` |
| `updated_at` | `INTEGER NOT NULL` | horodatage Unix | — |

### `convergence_metrics_raw`
- Clé primaire `(graph_id, step_id)` (enregistrement atomique avec `save_graph`, `metrics_recorded = true`).
- Colonnes `TEXT NOT NULL DEFAULT ''` pour `project`, `producer_engine`, `producer_model`, `verifier_engine`, `verifier_model`, `outcome` (aucun `NULL` dans les dimensions d'agrégation).
- Colonnes numériques : `verifier_orthogonal`, `iterations_used`, `tier1_iterations`, `tier2_iterations`, `tier_escalated`, `gate_configured`, `gate_runs`, `gate_failures`, `gate_saved_verifier_calls`, `stagnation_saved_iterations`, `duration_secs`, `finished_at`.
- Index : `idx_conv_metrics_raw_finished ON convergence_metrics_raw(finished_at)`.

### `convergence_metrics_daily`
- Clé primaire composite (`TEXT NOT NULL DEFAULT ''` partout) : `(day_utc, project, producer_engine, producer_model, verifier_engine, verifier_model, outcome)`.
- Rollup via `TEMP TABLE` de clés `(graph_id, step_id)` + `INSERT ... ON CONFLICT DO UPDATE` (sommes, `MIN`, `MAX`) + `DELETE` exact dans la même transaction.

### `convergence_metrics_monthly`
- Clé primaire composite (`TEXT NOT NULL DEFAULT ''` partout) : `(month_utc, project, producer_engine, producer_model, verifier_engine, verifier_model, outcome)` (`month_utc = 'YYYY-MM'`).
- Rollup identique depuis `convergence_metrics_daily`, puis purge des `month_utc` expirés.
