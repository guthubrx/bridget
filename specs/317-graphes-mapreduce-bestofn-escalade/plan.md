# Plan technique 317 : Map-Reduce borné et Best-of-N sur escalade (Lot B)

## 1. Modifications par fichier

1. **`crates/bridget-transport/src/protocol.rs`** :
   - Ajouter `GraphMapSpec` + constantes (`MIN_GRAPH_MAP_ITEMS = 2`, `MAX_GRAPH_MAP_ITEMS = 16`, `MAX_GRAPH_MAP_CONCURRENCY = 4`, `MAX_GRAPH_MAP_ITEM_BYTES = 512`).
   - Ajouter `#[serde(default, skip_serializing_if = "Option::is_none")] pub map: Option<GraphMapSpec>` sur `DelegateGraphStep`.
   - Ajouter `#[serde(default = "default_escalation_candidates")] pub candidates: u8` sur `GraphConvergenceEscalation`.

2. **`crates/bridget-daemon/src/delegation.rs`** :
   - Ajouter `NativeGraphMapItemRecord`, `NativeGraphMapWorkerSlot`, `NativeGraphMapState`.
   - Ajouter `NativeGraphEscalationCandidate`.
   - Étendre `NativeGraphStepRecord` (`map`, `map_state`) et `NativeGraphConvergenceState` (`escalation_candidates`, `winning_candidate_id`, `candidates_cancelled`) avec `#[serde(default)]` pour 100 % de rétrocompatibilité SQLite.

3. **`crates/bridget-daemon/src/daemon/native_delegation.rs`** :
   - Ajouter `pick_diverse_healthy_engine_pub(st: &DaemonState, exclude_families: &[&str]) -> Option<String>` pour assigner des moteurs orthogonaux aux candidats `Best-of-N` d'escalade.

4. **`crates/bridget-daemon/src/daemon/native_graphs.rs`** :
   - Validation : `map` (`2..=16` items, `max_concurrency` `1..=4`, mutuellement exclusif avec `convergence`), `escalation.candidates` (`1..=3`, `<= 2` sans `gate`).
   - Machine d'état Map-Reduce : `init_map_state`, `build_map_item_task`, `sync_map_step_from_tasks`, `finalize_map_step_if_done`, `GraphLaunchJob::MapItemDispatch` (tente `Continue` sur worker slot chaud si `reuse_workers`, sinon nouvelle admission `keep_alive >= 900` avec préfixe stable en tête).
   - Machine d'état Escalade `Best-of-N` : dans `try_trigger_escalation`, si `esc.candidates > 1`, initialise `escalation_candidates` et `phase = "escalating_candidates"` ; `sync_escalation_candidates_from_tasks`, `apply_escalation_candidate_gate`, `apply_escalation_candidate_verdict` (arrêt anticipé au premier `approve` + retour des `task_id` perdants à annuler), jobs `EscalationCandidateProduce` / `EscalationCandidateGate` / `EscalationCandidateVerify`.
   - Intégration `handle_submit`, `tick`, `propagate_graph_state`, `handle_cancel`, `collect_step_usage_agents`, `graph_view` et `format_graph_summary_message`.
   - Tests unitaires complets pour Map-Reduce (slots, réutilisation worker chaud, `fail_fast`, agrégation bilan, budget) et `Best-of-N` sur escalade (diversité moteurs, rejet par `gate` à 0 jeton, victoire au premier `approve` + annulation des autres candidats, plafonds validation).

5. **`crates/bridget-daemon/src/delegation_mcp.rs`, `mcp.rs`, `skills/bridget/SKILL.md`** :
   - Parsing et pré-validation locale de `map` et `convergence.escalation.candidates`.
   - Mise à jour du schéma complet + maintien `< 600` octets sur tous les outils compacts (`spec309_*`, `compact318_tests`).
   - Mise à jour concise de `skills/bridget/SKILL.md` (sans élision).
