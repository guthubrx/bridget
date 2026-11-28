# Tâches 317 : Map-Reduce borné et Best-of-N sur escalade (Lot B)

- [x] T001 : Étendre `crates/bridget-transport/src/protocol.rs` (`GraphMapSpec`, `DelegateGraphStep.map`, `GraphConvergenceEscalation.candidates`).
- [x] T002 : Étendre `crates/bridget-daemon/src/delegation.rs` (`NativeGraphMapState`, `NativeGraphEscalationCandidate`) et `crates/bridget-daemon/src/daemon/native_delegation.rs` (`pick_diverse_healthy_engine_pub`).
- [x] T003 : Implémenter dans `crates/bridget-daemon/src/daemon/native_graphs.rs` la machine d'état Map-Reduce sur workers chauds, l'escalade `Best-of-N` avec arrêt anticipé et annulation des perdants, le branchement `tick`/`handle_cancel`/`collect_step_usage_agents`, et les tests unitaires complets.
- [x] T004 : Mettre à jour `crates/bridget-daemon/src/delegation_mcp.rs`, `crates/bridget-daemon/src/mcp.rs` (`< 600` octets) et `skills/bridget/SKILL.md`.
- [x] T005 : Formater (`rustfmt`), exécuter les tests unitaires + `spec309_*` + `compact318_tests`, et lancer la revue adverse Claude (`bridget_auto_delegate`).
- [ ] T006 : Corriger tout constat fondé, committer et pousser Lot B sur `main`, compiler macOS + Linux, déployer `370-graphes-frugalite-mapreduce-316-317` sur la flotte (sans toucher T3), vérifier en live, et nettoyer.
