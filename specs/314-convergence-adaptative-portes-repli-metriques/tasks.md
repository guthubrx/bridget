# Tâches — Spec 314 (Convergence adaptative, portes déterministes, repli DAG et métriques Time-Machine)

**Statut** : Implémentation, tests, Converge, contre-revue adverse Claude (2 passes) et Audit validés (`12 / 12` — livraison flotte en cours)
**Gate `reuse-audit.md`** : `READY`
**Contre-revue adverse plan (Claude)** : 18 objections traitées et intégrées dans `spec.md` / `data-model.md` / `adversarial-review-claude.md`.
**Contre-revue adverse code (Claude)** : 0 P0, 5 P1 + 6 P2 corrigés et re-vérifiés (`32/32` tests `bridget-daemon` + `433/433` tests `bridget-transport` verts).
**Gate Phase 6 (`Converge`)** : `CONVERGED` (100 % des exigences Spec 314 couvertes par code + tests).
**Gate Phase 7 (`Audit`)** : `PASS` (sécurité porte sans shell + clean worktree avant/après + contrôle principal humain sur `graphs metrics*` + cap MCP compact `<= 550 B` prouvé).

---

## Lot 1 — Protocole partagé et porte déterministe isolée

- [x] **T01** — Étendre `crates/bridget-transport/src/protocol.rs` avec `GraphDeterministicGate`, `GraphConvergenceEscalation`, `GraphStepCondition` (`when` + `when_outcomes`), les extensions additives de `GraphConvergenceLoop` / `DelegateGraphStep`, `WorkerRequestOperation::RunDeterministicGate`, `WorkerResponseOperation::DeterministicGateResult`, et `NativeDelegationRequest::{GraphMetrics, GraphMetricsPolicy}` (+ types `ConvergenceMetricsPolicy` / `ConvergenceMetricsPolicyUpdate`). Vérification : `cargo check -p bridget-transport` et tests serde de rétrocompatibilité Spec 313.
- [x] **T02** — Créer `crates/bridget-daemon/src/deterministic_gate.rs` : validation stricte `validate_gate_spec`, vérification Git `HEAD == expected_commit` et arbre propre avant/après, exécution directe `argv` sans shell avec `env_clear()` + PATH minimal (zéro `BRIDGET_*`), `stdin(null)`, groupe de processus Unix (`setpgid(0,0)` + `kill(-pgid, SIGKILL)` au timeout), lecture continue bornée UTF-8 ≤ 8 Kio, empreinte normalisée `gate:{exit_code}:...`. Tests unitaires réels sur dossier Git temporaire (succès, échec exit!=0, arbre sale refusé, timeout avec processus enfant, troncature UTF-8).
- [x] **T03** — Brancher `WorkerRequestOperation::RunDeterministicGate` dans `crates/bridget-daemon/src/remote_worker.rs` (exécution asynchrone non bloquante `defer_deterministic_gate` / `poll_deterministic_gates` calquée sur `defer_view_file`, résolution worktree attesté via `WorkerResultIdentity`, capacité max 16 portes concurrentes). Vérification : tests unitaires worker sur `RunDeterministicGate`.

## Lot 2 — Machine à états convergence adaptative & branches DAG conditionnelles

- [x] **T04** — Étendre `NativeGraphConvergenceState` et `NativeGraphConvergenceIteration` dans `crates/bridget-daemon/src/delegation.rs` (`tier`, `tier_escalated`, `total_max_iterations`, `prior_producer_task_ids`, `seen_commits`, `seen_blocking_fingerprints`, champs porte et compteurs d'économies, `metrics_recorded`) avec `#[serde(default)]` pour 100 % rétrocompatibilité SQLite.
- [x] **T05** — Implémenter dans `crates/bridget-daemon/src/daemon/native_graphs.rs` :
  - validation complète (`gate` exige posture parent `development`, `when` + `when_outcomes`, rejet `cancel_graph` + `when != all_succeeded`, cohérence `stop_on_stagnation` / `escalation`, plafond global `<= 8`) ;
  - exécution non bloquante de la porte déterministe via `GraphLaunchJob::ConvergenceGate` (local ou worker distant avec conversion fail-closed `gate_unavailable` sur ancien worker) + court-circuit vérificateur LLM à 0 token si `passed == false` ;
  - détection anti-oscillation (`A → B → A`) sur `seen_commits` et `seen_blocking_fingerprints` ;
  - cascade d'escalade palier 0 → palier 1 (session neuve `AutoDelegateV2` clé déterministe `:t1:i{k}:produce`, ancrage commit, verdict `escalate` court-circuitant toujours l'escalade de palier, annulation des `prior_producer_task_ids`) ;
  - sélection de vérificateur orthogonal (`different_provider: true`) avec repli propre traçant `verifier_orthogonal`.
- [x] **T06** — Implémenter dans `crates/bridget-daemon/src/daemon/native_graphs.rs` la propagation et l'ordonnancement des branches conditionnelles DAG (`all_succeeded`, `any_failed`, `on_outcome` avec `when_outcomes`, état terminal `skipped_condition_not_met` propagé sans interblocage, `cancelled` exclu des replis, `fallback_recovered` et `counts.fallback_steps_completed` dans `graph_view`). Tests unitaires couvrant tous les chemins nominaux, erreurs, oscillations, escalades, portes et replis DAG.

## Lot 3 — Métriques durables SQLite & rollup/purge Time-Machine + CLI

- [x] **T07** — Créer `crates/bridget-daemon/src/convergence_metrics.rs` : schéma SQLite (`convergence_metrics_policy`, `convergence_metrics_raw`, `convergence_metrics_daily`, `convergence_metrics_monthly`, clés `TEXT NOT NULL DEFAULT ''`), validation des seuils (`daily >= raw`, `monthly * 31 > daily`, `graph >= 7`), écriture atomique couplée à `save_graph` (`metrics_recorded = true`, zéro double comptage), rollup transactionnel par table temporaire de clés (`raw → daily → monthly → purge` + `max_raw_rows`), purge des graphes terminaux (`notice_sent = 1`) + `native_delegation_graph_tasks`, et lecture agrégée multi-paliers. Tests SQLite complets prouvant la conservation exacte des sommes/MIN/MAX et l'absence de double comptage.
- [x] **T08** — Brancher `convergence_metrics` dans `DelegationStore`, `native_graphs.rs` (enregistrement atomique à la clôture d'une étape convergée + `metrics_summary` dans `graph_view`), `native_delegation::maintain_retention` (entretien horaire) et les handlers daemon `GraphMetrics` / `GraphMetricsPolicy`.
- [x] **T09** — Exposer les sous-commandes CLI `bridget graphs metrics [--project] [--window] [--json]` et `bridget graphs metrics-policy [--raw-days] [--daily-days] [--monthly-months] [--graph-days] [--max-raw-rows] [--run-now] [--json]` dans `crates/bridget-daemon/src/cli.rs`. Tests CLI de parsing et de rendu.

## Lot 4 — MCP (pré-validation & compact ≤ 550 B), skills et documentation

- [x] **T10** — Étendre la pré-validation locale MCP et le schéma complet dans `crates/bridget-daemon/src/delegation_mcp.rs` (0 consommation `request_id` sur erreur locale) et mettre à jour le schéma compact `bridget_delegate_graph` dans `crates/bridget-daemon/src/mcp.rs` en prouvant par test que TOUS les 20 outils restent `<= 550` octets.
- [x] **T11** — Mettre à jour `/Users/moi/.agents/skills/bridget/SKILL.md`, `/Users/moi/.agents/skills/graph_it/SKILL.md`, `docs/delegation-native.fr.md` et `docs/delegation-native.md` avec les nouvelles options (`gate`, `escalation`, `different_provider`, `when`, `bridget graphs metrics*`).

## Lot 5 — Validation finale, Converge, Audit et Livraison

- [x] **T12** — Exécuter les suites ciblées + `clippy` / `fmt-check`, mener Phase 6 (`Converge`) + contre-revue adverse finale (Claude) + Phase 7 (`Audit`), puis livrer sur `main` (commit, push `github` + `origin`, builds `linux-x86_64` `sysle2` + `macos-arm64` local, `deploy-fleet.py` sans toucher à `com.bridget.t3`, vérification live `bridget graphs metrics-policy` et ménage `menage.py`).
