# Modèle de données — 313

## 1. Protocole (`crates/bridget-transport/src/protocol.rs`)

### `GraphConvergenceLoop`
- `verify: AutoDelegateV2` — délégation du vérificateur indépendant exécuté après chaque itération du producteur.
- `max_iterations: u8` — borné `2..=5`, défaut `3`.
- `stop_on_stagnation: bool` — défaut `true`.

### `DelegateGraphStep` (extension additive)
- Champs existants inchangés : `step_id`, `depends_on`, `pass_dependency_results`, `delegation`.
- Nouveau champ optionnel : `convergence: Option<GraphConvergenceLoop>` (`#[serde(default, skip_serializing_if = "Option::is_none")]`).

## 2. Persistance durable (`crates/bridget-daemon/src/delegation.rs`)

### `NativeGraphConvergenceIteration`
- `iteration: u8` (`1..=5`)
- `producer_task_id: Option<String>`
- `producer_round: u32`
- `producer_commit: Option<String>`
- `verifier_task_id: Option<String>`
- `verdict: Option<String>` (`approve` | `revise` | `escalate` | `unparseable`)
- `reason: Option<String>`
- `blocking_points: Vec<String>`
- `blocking_fingerprint: Option<String>` (SHA-256 hex normalisé)
- `started_at: Option<i64>`
- `finished_at: Option<i64>`

### `NativeGraphConvergenceState`
- `current_iteration: u8`
- `max_iterations: u8`
- `stop_on_stagnation: bool`
- `phase: String` (`waiting` | `producing` | `verifying` | `done`)
- `producer_task_id: Option<String>`
- `producer_expected_round: u32`
- `verifier_task_id: Option<String>`
- `last_producer_commit: Option<String>`
- `last_blocking_fingerprint: Option<String>`
- `outcome: Option<String>` (`approved` | `escalated` | `stagnated` | `max_iterations_reached` | `verdict_unparseable` | `failed`)
- `iterations: Vec<NativeGraphConvergenceIteration>`

### `NativeGraphStepRecord` (extension additive)
- Nouveaux champs `#[serde(default, skip_serializing_if = "Option::is_none")]` :
  - `convergence: Option<GraphConvergenceLoop>`
  - `convergence_state: Option<NativeGraphConvergenceState>`
- Aucune altération du schéma SQLite : `native_delegation_graphs.payload` désérialise les anciens graphes avec `None`.
