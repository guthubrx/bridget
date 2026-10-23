# Tâches — Session 175
Statut : Implemented
## Préparation
- [x] T001 Vérifier les mécanismes T3 et Bridget, le périmètre et les gates dans specs/175-cycle-worktrees/reuse-audit.md.
## US1 — Conservation compréhensible
- [x] T002 [US1] Ajouter les tests de statut via le handler natif réel dans crates/bridget-daemon/src/daemon/native_workspace154_tests.rs : activité, continuation, rapport inconnu, changements, commits et lecture sans mutation.
- [x] T003 [US1] Étendre task_view dans crates/bridget-daemon/src/daemon/native_delegation.rs avec motif/action dérivés, sans E/S ni changement de WorkspaceRecord.
- [x] T004 [US1] Compléter append_workspace_result dans crates/bridget-daemon/src/daemon/native_delegation.rs par une consigne de conservation claire, dans le budget UTF-8 existant.
## US2 — Inventaire et capacité
- [x] T005 [P] [US2] Étendre les fixtures Git de scripts/tests/test_build_cleanup.py pour les motifs, seuils et erreurs sans mutation.
- [x] T006 [P] [US2] Réutiliser les observations du nettoyeur dans scripts/build.py ; expliquer les espaces conservés et candidats sans certifier une absence de tâche.
- [x] T007 [P] [US2] Ajouter la commande worktrees --repo --base-ref --json à scripts/build.py avec capacité scoped/total, 96/128 et état inconnu explicite.
- [x] T008 [P] [US2] Vérifier dans scripts/tests/test_build_cleanup.py que les gardes de retrait ignorés/processus/locked/nested sont conservées ; traiter les régressions du périmètre sans assouplir lsof.
## US3 — Choix et clôture
- [x] T009 [US3] Aligner docs/delegation-native.md et docs/delegation-native.fr.md : shared explicite, clôture, inspection, intégration/préservation puis retrait sans force ; limites d'observation.
- [x] T010 [US3] Exécuter l'inventaire réel en lecture seule sur Cartae et consigner commande/résultat/limites dans specs/175-cycle-worktrees/validation/verification.md.
## Dépendances
T001 précède tout. T002→T003→T004. T005→T006→T007→T008 indépendant du lot Rust ; T009 et T010 après les deux lots. ROOT possède Rust/docs ; un responsable distinct peut posséder les deux fichiers Python.
## Vérifications et livraison
Tests ciblés Rust native175 + rapports154 affectés ; Python WorktreePruneTests et tests de la nouvelle commande. Converge après code final. Contre-revue adverse, audit V15 ciblé, intégration/push/runtime et nettoyage documentés séparément comme gates de pipeline ; aucune certification globale.

## Convergence
- [x] T011 Aligner specs/175-cycle-worktrees/data-model.md sur les noms exacts du JSON produit (motifs, recovery, git_total, state, processus), sans changer le code.
- [x] T012 Traiter la revue finale : éviter lsof sans worktree, distinguer préfixe système lié des liens dans le périmètre, aligner candidat_a_examiner dans les guides et tester ces retouches.
