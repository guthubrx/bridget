# Audit de réutilisation de l'existant (`reuse-audit.md`) — 313

**Date** : 2026-11-27
**Statut** : `READY_FOR_TASKS`

## 1. Inventaire des composants proposés et recherche d'existant

| Élément proposé dans `plan.md` | Recherche existant (`grep` / lecture) | Décision | Justification / Emplacement |
|---|---|---|---|
| Structure de requête de graphe | `DelegateGraphRequest` / `DelegateGraphStep` dans `crates/bridget-transport/src/protocol.rs:2059-2086` | `REUTILISER` | Ajouter `convergence: Option<GraphConvergenceLoop>` sur `DelegateGraphStep` (rétrocompatible). |
| Persistance graphe SQLite | `NativeGraphRecord` / `NativeGraphStepRecord` + `DelegationStore::save_graph` / `get_graph` / `active_graphs` dans `crates/bridget-daemon/src/delegation.rs:3220-3269` | `REUTILISER` | Étendre `NativeGraphStepRecord` avec `convergence` + `convergence_state` (`#[serde(default)]`). Aucune nouvelle table SQLite. |
| Orchestration et `tick()` des graphes | `crates/bridget-daemon/src/daemon/native_graphs.rs` (`validate_and_order_graph`, `handle_submit`, `sync_steps_from_tasks`, `tick`, `graph_view`, `handle_cancel`) | `REUTILISER` | Étendre la machine d'état existante pour gérer les phases `producing` / `verifying` et la reprise via `NativeDelegationRequest::Continue`. |
| Reprise sur même worktree producteur | `NativeDelegationRequest::Continue` dans `crates/bridget-daemon/src/daemon/native_delegation.rs:13060+` | `REUTILISER` | Utiliser la commande native `Continue` existante (aucun nouveau mécanisme de session enfant). |
| Validation `AutoDelegateV2` / `workspace` / `brief` MCP | `parse_workspace`, `parse_brief`, `parse_delegate_graph` dans `crates/bridget-daemon/src/delegation_mcp.rs:19-468` | `REUTILISER` | Factoriser `parse_auto_delegate_object` pour valider `step.delegation` et `step.convergence.verify` sans duplication. |
| Schémas MCP compacts Antigravity | `compact_discovery_tools`, `compact_tool_description`, `compact_field_hint` dans `crates/bridget-daemon/src/mcp.rs:2907-3079` | `REUTILISER` | Ajouter les hints `bridget_delegate_graph` (`steps`) dans `compact_field_hint` / `compact_tool_description` sans créer un second projectionneur. |
| Documentation skills & guides | `/Users/moi/.agents/skills/bridget/SKILL.md`, `/Users/moi/.agents/skills/graph_it/SKILL.md`, `docs/delegation-native.fr.md`, `docs/delegation-native.md` | `REUTILISER` | Compléter les fichiers canoniques existants ; ne créer aucune nouvelle skill. |
| Parseur de verdict fermé + empreinte stagnation | Aucun parseur de verdict de graphe existant (recherche `parse_.*verdict\|blocking_fingerprint` négative dans `native_graphs.rs`) | `CREER` | Helper local dans `native_graphs.rs` réutilisant `sha2::Sha256` déjà présent dans `Cargo.toml`. |
| Dépendances externes Cargo | `Cargo.toml` (`serde`, `serde_json`, `sha2`, `uuid`, `rusqlite`) | `REUTILISER` | 0 nouvelle dépendance externe. |

## 2. Arbitrages
Aucun doublon détecté, aucun nouvel outil MCP, aucune nouvelle table, aucune nouvelle dépendance.

## 3. Gate avant tasks
- [x] Chaque élément du plan a été confronté à l'existant dans le dépôt.
- [x] Aucune duplication évidente non arbitrée.
- [x] Aucun refactor risqué hors périmètre.
- [x] Compatibilité ascendante SQLite, protocole et Antigravity compact vérifiée.
