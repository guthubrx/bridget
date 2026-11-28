# Tâches 316 : Graphes frugaux en quotas et mesure comparative (Lot A)

- [x] T001 : Étendre `crates/bridget-transport/src/protocol.rs` (`GraphBudget`, `GraphModelTier`, champs sur `DelegateGraphRequest`/`DelegateGraphStep`, `compare` sur `GraphMetrics`).
- [x] T002 : Étendre `crates/bridget-daemon/src/delegation.rs` (`NativeGraphBudgetUsage`, `NativeGraphBlackboardEntry`, champs rétrocompatibles sur `NativeGraphRecord`/`NativeGraphStepRecord`) et ajouter `Store::sum_task_usage_tokens` dans `crates/bridget-daemon/src/store/ledger_requests.rs`.
- [x] T003 : Étendre `crates/bridget-daemon/src/convergence_metrics.rs` (colonnes tokens + `bridget_build_id` + `model_tier`, table durable `graph_build_metrics`, vues `by_build`/`by_tier`/`comparison` + tests unitaires).
- [x] T004 : Ajouter le routage par palier `resolve_model_tier_pub` et `is_frontier_model_name` dans `crates/bridget-daemon/src/daemon/native_delegation.rs`.
- [x] T005 : Implémenter dans `crates/bridget-daemon/src/daemon/native_graphs.rs` : préfixe stable (Prompt Caching), budgets durs (`budget_exhausted`), tableau noir structuré (`blackboard`), appel `resolve_model_tier_pub`, et enregistrement complet des métriques + tokens réels par version, avec tests unitaires.
- [x] T006 : Étendre `crates/bridget-daemon/src/delegation_mcp.rs`, `crates/bridget-daemon/src/mcp.rs` (en gardant `< 600` octets sur tous les outils compacts), `crates/bridget-daemon/src/cli.rs` (`--compare` et affichage `by_build`/`by_tier`), et `skills/bridget/SKILL.md`.
- [x] T007 : Formater (`rustfmt`), exécuter les tests unitaires ciblés + `spec309_*` + `compact318_tests`, et lancer la revue adverse Claude (`bridget_auto_delegate`) + intégrer les correctifs P1/P2.
- [ ] T008 : Committer et pousser sur `main`, compiler macOS + Linux, déployer `370-graphes-frugalite-316` sur la flotte (sans toucher T3), vérifier en live, et nettoyer.
