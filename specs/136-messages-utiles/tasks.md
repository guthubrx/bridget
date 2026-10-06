# Tâches136 — Périmètre complet

## Préparation

- [x] T001 Spécifier, planifier, auditer la réutilisation et analyser les12 exigences dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/136-messages-utiles/specs/136-messages-utiles ; checklist complète et aucun doublon ouvert.
- [x] T002 Écrire Gherkin et tests RED dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/136-messages-utiles/tests/features/136-messages-utiles.feature et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/136-messages-utiles/crates/bridget-daemon/tests/spec102_threads_test.rs (FR01–12).

## US1 — Historique silencieux

- [x] T003 [US1] Étendre contrat et validations dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/136-messages-utiles/crates/bridget-transport/src/protocol.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/136-messages-utiles/crates/bridget-daemon/src/threads.rs ; classes fermées, UTF-8, silence et canon legacy (FR01/02/04/09/10).

## US2 — Actualité des consignes

- [x] T004 [US2] Étendre migration et dépôt transactionnel dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/136-messages-utiles/crates/bridget-daemon/src/store/threads.rs ; auteur/audience/tête contrôlés avant ACK, refus atomiques (FR03/05/06/12).
- [x] T005 [US2] Étendre read_range dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/136-messages-utiles/crates/bridget-daemon/src/store/threads.rs ; corps actuels et références compactes, historique exact (FR03/07).

## US3 — Reprise et accès commun

- [x] T006 [US3] Vérifier snapshot, reçu, ancienne base, pagination/restart dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/136-messages-utiles/crates/bridget-daemon/tests/spec102_threads_test.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/136-messages-utiles/crates/bridget-daemon/src/store/threads.rs (FR08/09/10/12).
- [x] T007 [US3] Étendre CLI/MCP et leurs tests dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/136-messages-utiles/crates/bridget-daemon/src/cli.rs et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/136-messages-utiles/crates/bridget-daemon/src/mcp.rs ; mêmes champs et refus (FR11).

## Validation et livraison

- [x] T008 Documenter la règle dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/136-messages-utiles/docs/reference-communication.md et l'enveloppe T3 ; ordre relu par history non exécuté, limites legacy explicites (FR07/08/09/11).
- [x] T009 Lancer tests ciblés et workspace, fmt/clippy/build dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/136-messages-utiles ; logs et résultats conservés dans specs136 (SC01–04).
- [x] T010 Contre-revue, Converge et audit v14 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/136-messages-utiles/specs/136-messages-utiles ; chaque FR relié au code/test, tâches byte-identiques (SC03/04).
- [x] T011 Livrer le seul périmètre136 depuis /Users/moi/Nextcloud/10.Scripts/64.bridget : sauvegarde, commit/fusion/push, déploiement vérifié, règle envoyée aux quatre responsables (deux accepted, deux en file pendant leurs tours actifs), nettoyage sûr. Préserver135 et anciennes files (FR09/11/12). L'adoption des règles par les agents n'est pas présumée.

## Dépendances et exécution

T001→T002→T003→T004→T005→T006→T007→T008→T009→T010→T011.
Pas de travail parallèle sur les mêmes fichiers. Opportunité sans conflit:
revue adverse en lecture pendant tests. Chaque histoire doit rester vérifiable.
Tous les scénarios sont livrés ; pas d'arrêt à US1 seule.
