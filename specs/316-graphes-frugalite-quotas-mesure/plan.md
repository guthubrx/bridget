# Plan technique 316 : Graphes frugaux en quotas et mesure comparative (Lot A)

## 1. Modifications par crate / module

### 1. `crates/bridget-transport/src/protocol.rs`
- Ajouter `GraphBudget` (`max_total_rounds`, `max_frontier_rounds`, `max_duration_secs`) avec constantes de bornes (`1..=128`, `0..=64`, `30..=86_400`).
- Ajouter `GraphModelTier` (`Light`, `Standard`, `Frontier`).
- Étendre `DelegateGraphRequest` : `#[serde(default, skip_serializing_if = "Option::is_none")] pub budget: Option<GraphBudget>`.
- Étendre `DelegateGraphStep` :
  - `#[serde(default, skip_serializing_if = "Option::is_none")] pub model_tier: Option<GraphModelTier>`
  - `#[serde(default, skip_serializing_if = "Vec::is_empty")] pub blackboard_reads: Vec<String>`
  - `#[serde(default, skip_serializing_if = "Vec::is_empty")] pub blackboard_writes: Vec<String>`
- Étendre `NativeDelegationRequest::GraphMetrics` : `#[serde(default, skip_serializing_if = "Option::is_none")] pub compare: Option<(String, String)>` (rétrocompatible grâce à `#[serde(default)]`).

### 2. `crates/bridget-daemon/src/delegation.rs`
- Ajouter `NativeGraphBudgetUsage` (`total_rounds_used`, `frontier_rounds_used`, `exhausted_reason`) et `NativeGraphBlackboardEntry` (`value`, `updated_by_step`, `updated_at`).
- Étendre `NativeGraphRecord` avec `#[serde(default)]` sur : `bridget_build_id`, `budget`, `budget_usage`, `blackboard`.
- Étendre `NativeGraphStepRecord` avec `#[serde(default)]` sur : `model_tier`, `blackboard_reads`, `blackboard_writes`, `blackboard_reads_injected`, `blackboard_writes_applied`, `metrics_recorded`.
- Garder 100 % la rétrocompatibilité de désérialisation des graphes existants en SQLite.

### 3. `crates/bridget-daemon/src/store/ledger_requests.rs`
- Ajouter `Store::sum_execution_usage_tokens(&self, execution_ids: &[String]) -> Result<UsageTokens, StoreError>` qui somme `execution_usage_totals` sur les `execution_id` fournis (0 si vide ou absent).

### 4. `crates/bridget-daemon/src/convergence_metrics.rs`
- Étendre `ConvergenceMetricRawRow` avec : `bridget_build_id`, `model_tier`, `input_tokens`, `output_tokens`, `cache_creation_input_tokens`, `cache_read_input_tokens`, `blackboard_reads`, `blackboard_writes`.
- Dans `ensure_schema` :
  - Ajout idempotent des colonnes manquantes sur `convergence_metrics_raw` (via `PRAGMA table_info` + `ALTER TABLE ADD COLUMN`).
  - Création de la table durable `graph_build_metrics` (PK `(bridget_build_id, project, model_tier)`) stockant les compteurs fonctionnels + tokens + cache + budget + tableau noir par version Bridget, mise à jour dans `insert_raw_in_tx` (même transaction atomique, protégée par `INSERT OR IGNORE` vérifiant `changes() > 0` afin de garantir zéro double comptage !).
- Étendre `query_metrics_report` avec `compare: Option<(&str, &str)>` :
  - Ajoute `by_build`, `by_tier`, et `comparison` (si `--compare A..B` est demandé) dans le rapport JSON.

### 5. `crates/bridget-daemon/src/daemon/native_delegation.rs`
- Ajouter `resolve_model_tier_pub(st: &DaemonState, delegation: &mut AutoDelegateV2, tier: GraphModelTier)` et `is_frontier_model_name(model: &str) -> bool`.
- Classification déterministe des modèles disponibles dans `st.registry` + `st.remote_workers` (en écartant les moteurs épuisés/bloqués dans `st.provider_limits`), sans jamais écraser un `model` ou `effort` explicitement imposé.

### 6. `crates/bridget-daemon/src/daemon/native_graphs.rs`
- **Préfixe stable (FR-001)** : réordonner `build_verifier_task`, `build_enriched_step_task` et `build_producer_revision_task` (invariant en tête, variable en pied).
- **Budgets durs (FR-002)** : ajouter `"budget_exhausted"` à `ALLOWED_WHEN_OUTCOMES`, valider `GraphBudget`, contrôler `check_graph_budget` avant chaque lancement (étapes racines, étapes aval, `ConvergenceVerify`, `ConvergenceNextProduce`), incrémenter `budget_usage`, exposer dans `graph_view` et `format_graph_summary_message`.
- **Tableau noir (FR-003)** : parser `extract_blackboard_updates(text)` + `apply_blackboard_updates(...)`, injecter `blackboard_reads` dans `build_enriched_step_task`, exposer `blackboard` dans `graph_view`.
- **Palier de coût (FR-004)** : appliquer `resolve_model_tier_pub` à l'admission des étapes (racines et aval).
- **Mesure complète (FR-005)** : dans `persist_graph_with_pending_metrics`, collecter les `execution_id` de toutes les tâches de l'étape, lire leurs tokens attestés depuis `st.store.sum_execution_usage_tokens(...)`, et enregistrer la métrique pour toutes les étapes terminales du graphe (convergées ou simples) avec `bridget_build_id`.

### 7. `crates/bridget-daemon/src/delegation_mcp.rs`, `mcp.rs`, `cli.rs`, `skills/bridget/SKILL.md`
- Parser et pré-valider `budget`, `model_tier`, `blackboard_reads`, `blackboard_writes` localement dans `parse_delegate_graph` (sans consommer `request_id`).
- Mettre à jour le schéma complet + schéma compact (`< 600` octets garanti !).
- Ajouter `--compare <buildA>..<buildB>` dans `bridget graphs metrics` (`cli.rs`) et afficher les sections `Par version Bridget (by_build)`, `Par palier (by_tier)` et `Comparaison`.
- Mettre à jour `skills/bridget/SKILL.md` (sans élision).

## 2. Plan de vérification (TDD)
- Tests unitaires dans `convergence_metrics.rs` (migration colonnes, `graph_build_metrics`, zéro double comptage, `by_build`, `by_tier`, `compare` deltas exacts).
- Tests unitaires dans `native_graphs.rs` (préfixe stable `build_verifier_task` / `build_enriched_step_task` / `build_producer_revision_task`, budgets durs `max_total_rounds` / `max_frontier_rounds` / `max_duration_secs` + repli `light` sur `budget_exhausted`, extraction/validation/injection `blackboard`, classification `model_tier`).
- Tests de pré-validation MCP dans `delegation_mcp.rs` + plafond compact `< 600` octets dans `mcp.rs` (`spec309_*`, `compact318_tests`).
- Tests CLI dans `cli.rs` pour `bridget graphs metrics --compare A..B`.
- Revue adverse indépendante via Claude (`bridget_auto_delegate`, posture `discovery`).
