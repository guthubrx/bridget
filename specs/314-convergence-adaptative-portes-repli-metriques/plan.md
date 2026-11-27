# Plan d'implémentation — Spec 314 (Convergence adaptative, portes déterministes, repli DAG et métriques Time-Machine)

**Branche** : `main` (décision branche unique `AGENTS.md`)
**Stratégie** : Extension additive de l'architecture Spec 313 en 5 lots incrémentaux, sans nouvelle dépendance externe, sans casser la rétrocompatibilité SQLite/protocole/MCP compact.

---

## Architecture par couches (réutilisation maximale de l'existant)

### Lot 1 — Protocole partagé (`crates/bridget-transport/src/protocol.rs`)
- Ajouter les types additifs (`#[serde(default, deny_unknown_fields)]`) :
  - `GraphDeterministicGate { argv: Vec<String>, timeout_secs: u64 }` (constantes `MIN_GATE_TIMEOUT_SECS = 5`, `DEFAULT_GATE_TIMEOUT_SECS = 120`, `MAX_GATE_TIMEOUT_SECS = 300`, `MAX_GATE_ARGV = 32`, `MAX_GATE_OUTPUT_BYTES = 8192`).
  - `GraphConvergenceEscalation { engine, model, effort, machine_id, extra_iterations (1..=3, défaut 2), on_stagnation (true), on_max_iterations (true) }` + `MAX_TOTAL_CONVERGENCE_ITERATIONS = 8`.
  - `GraphStepCondition` (`AllDependenciesSucceeded` défaut, `AnyDependencyFailed`, `OnDependencyOutcome { outcomes: Vec<String> }`).
  - Étendre `GraphConvergenceLoop` (`gate`, `escalation`, `different_provider`) et `DelegateGraphStep` (`when`).
- Ajouter les opérations worker :
  - `WorkerRequestOperation::RunDeterministicGate { execution: WorkerResultIdentity, argv: Vec<String>, timeout_secs: u64 }`.
  - `WorkerResponseOperation::DeterministicGateResult { execution, passed, exit_code, timed_out, duration_ms, output_excerpt, reason }`.
- Ajouter les opérations daemon pour consultation/réglage des métriques :
  - `NativeDelegationRequest::GraphMetrics { project: Option<String>, window_days: Option<u32> }`
  - `NativeDelegationRequest::GraphMetricsPolicy { update: Option<ConvergenceMetricsPolicyUpdate>, run_now: bool }`

### Lot 2 — Exécution isolée des portes déterministes (`crates/bridget-daemon/src/deterministic_gate.rs` + `remote_worker.rs`)
- Un seul petit module pur `deterministic_gate.rs` (~140 lignes) partagé par l'exécution locale et par `remote_worker.rs` :
  - `validate_gate_spec(&GraphDeterministicGate)` (utilisé par `delegation_mcp.rs`, `native_graphs.rs` et `remote_worker.rs`).
  - `execute_gate_in_dir(dir: &Path, argv: &[String], timeout_secs: u64) -> GateExecOutput` : `Command::new(&argv[0]).args(&argv[1..]).current_dir(dir)`, pas de shell, capture stdout+stderr bornée à `8 192` octets UTF-8, attente bornée avec `kill()` garanti en cas de timeout.
- Dans `remote_worker.rs` : réutiliser exactement le motif asynchrone non bloquant de `defer_view_file` / `poll_view_files` (`defer_deterministic_gate` / `poll_deterministic_gates`, capacité bornée à 16 portes concurrentes, résolution du worktree via la même preuve `WorkerResultIdentity` que `begin_view_file`).

### Lot 3 — Machine à états de convergence adaptative & DAG conditionnel (`crates/bridget-daemon/src/delegation.rs` + `daemon/native_graphs.rs`)
- **Porte déterministe (`pending_gate` / `running_gate`)** :
  - Après remise producteur : si `conv.gate.is_some()` → phase `pending_gate`, lancement hors verrou dans `tick()` (via `remote_worker_roundtrip_with_send_deadline` si `task.remote_placement.is_some()`, sinon exécution locale dans le worktree/cwd attesté).
  - Si `passed == true` → `pending_verify` (preuve de porte verte incluse dans le préambule du vérificateur).
  - Si `passed == false` → **0 appel vérificateur**, incrément `gate_saved_verifier_calls`, empreinte `gate_fingerprint`, passage par détection stagnation/oscillation + escalade + plafond, puis `pending_produce` avec l'extrait déterministe.
- **Anti-oscillation & escalade de modèle** :
  - `seen_commits` et `seen_blocking_fingerprints` sur le palier courant → détecte `A → A` et `A → B → A`.
  - Helper pur `try_trigger_escalation(step, trigger_reason, now) -> bool` : si `escalation_tier == 0`, `conv.escalation` présent et déclencheur autorisé (`on_stagnation` / `on_max_iterations`), incrémente `escalation_tier = 1`, remet les `seen_*` du palier à vide, ajoute `extra_iterations` au budget de tours effectif (`<= 8`), force une nouvelle admission `AutoDelegateV2` avec les surcharges `(engine, model, effort, machine_id)` et un brief compact des échecs du palier 1.
- **Vérificateur orthogonal (`different_provider: true`)** :
  - Dans `tick()` lors de la préparation de `ConvergenceVerify`, si `conv.different_provider && verify_del.engine.is_none()`, inspecter l'`engine` effectif du producteur et sélectionner un moteur alternatif disponible dans le catalogue observé (`observe_routing_candidates_for_project` / `providers`), sinon conserver le routage auto et marquer `verifier_orthogonal = false`.
- **Branches de repli conditionnelles (`when`)** :
  - Mettre à jour `validate_and_order_graph` (interdire `when != AllDependenciesSucceeded` sur une étape racine sans `depends_on`, valider `outcomes` non vide et valeurs connues).
  - Mettre à jour `propagate_graph_state` et `ready_waiting_steps` :
    - Pour `AllDependenciesSucceeded` : inchangé.
    - Pour `AnyDependencyFailed` / `OnDependencyOutcome` : une étape attend que toutes ses `depends_on` soient terminales ; si la condition est satisfaite → prête à démarrer (`ready_waiting_steps`, enrichie des erreurs/commit/blocking_points des dépendances échouées) ; sinon → `skipped_condition_not_met`.
    - Statut final du graphe : `skipped_condition_not_met` ne déclenche pas `partial_failure` si toutes les étapes exécutées ont réussi (`completed`). Exposer `skipped_condition_not_met` et `fallback_steps_completed` dans `counts`.

### Lot 4 — Métriques SQLite + Rollup/Purge multi-paliers Time-Machine + CLI (`delegation.rs` + `convergence_metrics.rs` + `daemon/native_delegation.rs` + `cli.rs`)
- Nouveau sous-module ciblé `crates/bridget-daemon/src/convergence_metrics.rs` (responsabilité unique : schéma SQLite, enregistrement idempotent d'une étape convergée, politique `ConvergenceMetricsPolicy`, rollup transactionnel `raw → daily → monthly → purge`, purge `native_delegation_graphs`, agrégation de requête multi-paliers).
- Brancher `rollup_and_purge_convergence_metrics` dans `native_delegation::maintain_retention` (`native_delegation.rs:18479`, appelé toutes les heures par `maintenance.rs`).
- Enregistrer la métrique automatiquement dans `native_graphs.rs` dès qu'une étape avec `convergence` atteint un état terminal (idempotent par `(graph_id, step_id)` via `INSERT OR IGNORE`).
- Exposer côté CLI dans `cli.rs` :
  - `bridget graphs metrics [--project <p>] [--window <days>] [--json]`
  - `bridget graphs metrics-policy [--raw-days N] [--daily-days N] [--monthly-months N] [--graph-days N] [--max-raw-rows N] [--run-now] [--json]`

### Lot 5 — MCP compact (≤ 550 B), pré-validation locale, skills et docs
- `delegation_mcp.rs` : pré-validation locale de `when`, `convergence.gate`, `convergence.escalation`, `convergence.different_provider` sans consommer `request_id` ; mise à jour du schéma complet.
- `mcp.rs` : mise à jour ultra-compacte de `bridget_delegate_graph` (`<= 550 B` prouvé par test).
- Mise à jour de `/Users/moi/.agents/skills/bridget/SKILL.md`, `/Users/moi/.agents/skills/graph_it/SKILL.md`, `docs/delegation-native.fr.md`, `docs/delegation-native.md`.

---

## Charge cognitive (Article XX)
- Aucune nouvelle abstraction générique ou dépendance crate ajoutée.
- Les 2 nouveaux fichiers (`deterministic_gate.rs` et `convergence_metrics.rs`) isolent chacun une responsabilité cohésive (exécution bornée sans shell + rollup Time-Machine SQLite) pour éviter d'alourdir `native_graphs.rs` et `delegation.rs` déjà longs.
- Tout champ nouveau est optionnel avec défaut rétrocompatible : les appels Spec 313 existants restent 100 % valides.
