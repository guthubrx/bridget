# Audit de l'existant (Reuse Audit) — Spec 314

**Date** : 2026-11-27
**Statut** : `READY` (aucune duplication bloquante, aucune dépendance externe nouvelle)

## 1. Inventaire de l'existant vérifié

| Élément proposé par `plan.md` | Recherche existant (`file:line`) | Décision | Justification |
|---|---|---|---|
| Protocole graphe & convergence | `crates/bridget-transport/src/protocol.rs:2056-2112` (`GraphConvergenceLoop`, `DelegateGraphStep`) | **REUTILISER / ÉTENDRE** | Extension strictement additive (`#[serde(default)]`), zéro rupture V1/Spec 313. |
| Protocole worker distant | `crates/bridget-transport/src/protocol.rs:3917-3960` (`WorkerRequestOperation::ReadCommitViewFile`) et `crates/bridget-daemon/src/remote_worker.rs:6586-6675` | **REUTILISER / ÉTENDRE** | Réutilise la même enveloppe `WorkerRequest`/`WorkerResponse`, la même preuve `WorkerResultIdentity` et le même motif asynchrone non bloquant (`defer_*`/`poll_*`). |
| Exécution porte déterministe `argv` sans shell | Aucune primitive générique de commande arbitraire bornée sur worktree attesté (`rg` confirmé) | **CRÉER (`deterministic_gate.rs`)** | Module pur ~140 lignes partagé entre exécution locale et `remote_worker.rs`, isole les invariants sécurité (sans shell, timeout dur, troncature UTF-8). |
| Machine à états convergence & DAG | `crates/bridget-daemon/src/daemon/native_graphs.rs` (2867 lignes) et `delegation.rs:3246-3325` | **REUTILISER / ÉTENDRE** | Étend `validate_and_order_graph`, `propagate_graph_state`, `ready_waiting_steps`, `apply_producer_iteration_result`, `apply_verifier_verdict`, `tick()`. |
| Entretien horaire SQLite | `crates/bridget-daemon/src/daemon/maintenance.rs:157` + `native_delegation.rs:18479` (`maintain_retention`) | **REUTILISER / ÉTENDRE** | Branche le rollup Time-Machine et la purge des graphes dans le passage horaire existant sans nouveau thread ni timer. |
| Stockage métriques & rollup Time-Machine | `crates/bridget-daemon/src/delegation.rs:1439-1455` (`native_delegation_graphs`) ; aucune table de métriques de convergence ni rollup multi-paliers existant | **CRÉER (`convergence_metrics.rs`)** | Isole le schéma SQLite (`convergence_metrics_{policy,raw,daily,monthly}`), le rollup atomique transactionnel et l'agrégation multi-paliers, branché sur la même connexion `DelegationStore`. |
| Pré-validation MCP & schéma compact ≤ 550 B | `crates/bridget-daemon/src/delegation_mcp.rs:570-745` et `crates/bridget-daemon/src/mcp.rs:2947-2960` | **REUTILISER / ÉTENDRE** | Aucun nouvel outil MCP (reste à 20 outils) ; maintien strict ≤ 550 octets. |
| CLI métriques & politique | `crates/bridget-daemon/src/cli.rs` | **REUTILISER / ÉTENDRE** | Ajoute la sous-commande `bridget graphs <metrics|metrics-policy>` au routeur CLI existant. |
| Dépendances Cargo | `Cargo.toml` (`rusqlite`, `serde`, `sha2`, `serde_json` déjà présents) | **REUTILISER** | Zéro nouvelle dépendance externe. |

## 2. Gate avant tasks

- [x] Aucun doublon de service, table, endpoint ou dépendance détecté.
- [x] Tous les nouveaux fichiers (`deterministic_gate.rs`, `convergence_metrics.rs`) sont justifiés par une responsabilité cohésive distincte (Article XIX).
- [x] Aucun arbitrage utilisateur bloquant restant (politique de porte et rétention Time-Machine déjà validées).
- [x] Statut final : **`READY`** — passage autorisé vers la contre-revue adverse du plan et la génération de `tasks.md`.

## 3. Arbitrages en cours d'implémentation

*(Section réservée au gate anti-doublon de la Phase 5 si un arbitrage supplémentaire apparaît.)*
