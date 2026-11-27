# Convergence et Audit — Spec 313 (Graphes de convergence bornée et découvrabilité MCP)

**Date** : 2026-11-27
**Verdict Phase 6 (Converge)** : `CONVERGED` (100 % des exigences FR-01..FR-09 et NFR-01..NFR-03 vérifiées dans le code et les tests, zéro tâche manquante).
**Verdict Phase 7 (Audit)** : `PASS` (sécurité, permissions, minimalisme, complexité $O(V+E)$ / $O(L)$, rétrocompatibilité SQLite/protocole et budget MCP compact vérifiés).

## Matrice de convergence (Exigence -> Preuve)

| Exigence | Implémentation | Preuve / Test |
|---|---|---|
| **FR-01** (`GraphConvergenceLoop`, `2..=5`, défaut `3`, `stop_on_stagnation: true`, `deny_unknown_fields`) | `crates/bridget-transport/src/protocol.rs` (`GraphConvergenceLoop`, `DelegateGraphStep.convergence`) | `bridget-transport --lib protocol::` (140/140 ok) + `convergence_validation_bounds_and_verdict_parser` |
| **FR-02** (Validation stricte client + daemon : `1..=16`, `2..=5`, Kahn, `AutoDelegateV2`, `brief`, `workspace`) | `crates/bridget-daemon/src/delegation_mcp.rs` (`parse_auto_delegate_object`, `parse_convergence_loop`, `validate_dag_topology_locally`) + `crates/bridget-daemon/src/daemon/native_graphs.rs` (`validate_and_order_graph`) | `spec313_delegate_graph_prevalidates_convergence_and_dag_locally` + `convergence_validation_bounds_and_verdict_parser` |
| **FR-03** (Préambule vérificateur : faits + contrat JSON fermé) | `crates/bridget-daemon/src/daemon/native_graphs.rs` (`build_verifier_task`) | `daemon::native_graphs::tests` |
| **FR-04** (Parseur verdict depuis la fin, tolérance bloc ```` ```json ````, fail-closed) | `crates/bridget-daemon/src/daemon/native_graphs.rs` (`parse_convergence_verdict`) | `convergence_validation_bounds_and_verdict_parser` + `convergence_state_machine_...` (`verdict_unparseable`) |
| **FR-05** (Continuité producteur `keep_alive_secs >= 900`, `Continue` + fallback `AutoDelegateV2`, propagation commit) | `crates/bridget-daemon/src/daemon/native_graphs.rs` (`ensure_convergence_producer_keep_alive`, `apply_commit_to_delegation`, `tick`, `GraphLaunchJob::ConvergenceNextProduce`) | `daemon::native_graphs::tests` + durcissement reprise après redémarrage et annulation concurrente |
| **FR-06** (Clés déterministes <= 128 octets + persistance SQLite `convergence_state`) | `crates/bridget-daemon/src/delegation.rs` (`NativeGraphConvergenceState`, `NativeGraphConvergenceIteration`) + `native_graphs.rs` (`convergence_step_request_id`) | `convergence_validation_bounds_and_verdict_parser` + `sqlite_graph_store_roundtrip_and_notice_suppression` |
| **FR-07** (Stagnation #1 commit identique + #2 `blocking_fingerprint` SHA-256) | `crates/bridget-daemon/src/daemon/native_graphs.rs` (`apply_producer_iteration_result`, `apply_verifier_verdict`, `blocking_fingerprint`) | `convergence_state_machine_revise_approve_stagnation_escalate_and_max_rounds` (cas 2 et 8) |
| **FR-08** (Mode MCP compact Antigravity sans `properties` imbriquées + budget ACP) | `crates/bridget-daemon/src/mcp.rs` (`compact_tool_description`, `compact_field_hint`) | `spec318_schemas_compacts_gardent_items_de_chaque_tableau` + `spec309_compact_discovery_tools_fits_acp_budget_and_keeps_core_delegation_fields` |
| **FR-09** (Skills + documentation FR/EN) | `skills/bridget/SKILL.md` (`~/.agents/skills/bridget/SKILL.md`), `~/.agents/skills/graph_it/SKILL.md`, `docs/delegation-native.fr.md`, `docs/delegation-native.md` | Fichiers mis à jour et vérifiés |
| **NFR-01** (Zéro élévation de droits) | `native_graphs.rs` réutilise `graph.attested_permissions` et `prepare_exec_conn` | Revue code |
| **NFR-02** (Complexité bornée $O(V+E)$, $O(L)$, $O(G \cdot S)$) | Kahn $V \le 16$, parseur en une passe, `tick()` court-circuité si `active_graphs().is_empty()` | Revue code |
| **NFR-03** (Rétrocompatibilité SQLite sans migration destructive) | `#[serde(default)]` sur `convergence` et `convergence_state` dans `NativeGraphStepRecord` | `sqlite_graph_store_roundtrip_and_notice_suppression` |

## Audit sécurité, minimalisme et fiabilité (Phase 7)

- **Sécurité & permissions** : aucune élévation de droits, aucune fuite de valeur JSON libre dans les refus locaux (`parse_workspace`, `parse_brief`, `parse_convergence_loop`), `request_id` jamais consommé sur erreur de forme locale.
- **Fiabilité concurrence & reprise** :
  - Reprise après redémarrage daemon sécurisée sur `launching_verify` / `launching_produce` via clés idempotentes.
  - Annulation concurrente pendant admission nettoyée hors verrou (zéro tâche orpheline).
  - `CancelGraph` et `bridget_graph_cancel` annulent à la fois `producer_task_id` et `verifier_task_id`.
- **Minimalisme** : zéro nouvelle table SQLite, zéro dépendance Cargo ajoutée, réutilisation intégrale des primitives natives (`AutoDelegateV2`, `Continue`, `Cancel`, `parse_workspace`, `parse_brief`, `DelegationStore`).
