# Tâches 104

Statut: Implemented — 28/28 (2026-09-18). Recette release complète verte (1509 réussis, 0 échec), fmt/clippy OK,
Analyze/Converge consignés dans analysis.md, dossier de livraison dans quickstart.md.
Gate reuse-audit.md PASS relu ; contre-revue bdget reçue et corrections intégrées avant cette liste.

Chaque tâche a un résultat observable. Les tâches de tests avant code établissent l'oracle
et documentent les échecs attendus ; ne pas appeler ces tests « verts » avant le comportement.
Les gates de story exigent ensuite tous les tests concernés verts avant livraison de l'incrément.

## Préparation

- [x] T001 Vérifier les bases et la disponibilité du code 102 puis consigner l'ordre d'intégration dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/specs/104-recherche-echanges/implementation.md ; attendu : messages indépendants, fils non livrables avant102 validée ; ne pas copier un worktree non commité. Couverture : FR-003, FR-012.

- [x] T002 Préparer corpus oracle, accès attestés099 et harnais isolé dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-daemon/tests/search_104_test.rs ; attendu : home/socket/base uniques, aucun fournisseur réel, arrêt propre sans SIGKILL ; fixture de grande taille entièrement synthétique. Couverture : FR-008, FR-015, SC-001.

## Socle commun

- [x] T003 Écrire puis définir les requêtes/réponses strictes104 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-transport/src/protocol.rs ; attendu : LedgerSearch/LedgerRead, erreurs et schémas par action, aucune identité appelante ni chemin de base dans les paramètres, tests spec104 de sérialisation et compatibilité ancien client. Couverture : FR-008, FR-010, FR-012. Scénarios : S05, S12, S27.

- [x] T004 Ajouter idempotemment les deux index d'accès participant dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-daemon/src/store.rs ; attendu : réouverture d’une ancienne base deux fois sans perte de données ; EXPLAIN des plages par index, conserver les index existants. Couverture : FR-011. Scénarios : S23.

## US1 — Retrouver mes échanges

- [x] T005 [US1] Écrire les tests du corpus, termes, filtres et autorisation dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-daemon/tests/search_104_test.rs ; attendu : oracle déterministe, tiers invisibles, paramètres malformés, accents précomposés/décomposés ; échecs attendus explicités. Couverture : FR-001, FR-002, FR-008, SC-001. Scénarios : S01–S06.

- [x] T006 [US1] Unifier le repli et la localisation de l'occurrence dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-daemon/src/store/ledger_requests.rs ; attendu : une seule règle de repli par caractère, corps replié une fois, match_offset brutUTF8 correct sans tableau géant d’offsets ; SHA256 des résultats et extrait ciblé de 512 octets. Couverture : FR-001, FR-004, SC-003. Scénarios : S02, S03, S10, S30.

- [x] T007 [US1] Remplacer la primitive search_messages dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-daemon/src/store/ledger_requests.rs ; attendu : deux plages de 129 clés maximum, fusion de 258 clés maximum, chargement groupé et lecture ligne par ligne, bornes128/1Mio/16Mio ; filtres avant repli et erreurs propagées. Couverture : FR-001, FR-002, FR-004, FR-005, FR-010, FR-011. Scénarios : S01, S04, S06, S09.

- [x] T008 [US1] Installer l'exécution read-only bornée côté daemon dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-daemon/src/daemon.rs ; attendu : identité vérifiée avant/après,2 permis libérés automatiquement, pas de file d’attente ni verrou global pendant SQL ; ouverture READ_ONLY liée au chemin configuré, attente SQLite limitée à 100 ms. Couverture : FR-008, FR-010, FR-011, FR-015. Scénarios : S06, S14, S22, S24.

- [x] T009 [US1] Brancher action search dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-daemon/src/mcp.rs ; attendu : registered_connection099, résultats typés, aucune source cliente ni repli vers le ledger global ; ancien contrat ledger préservé. Couverture : FR-001, FR-010, FR-012. Scénarios : S01, S05, S27.

- [x] T010 [US1] Brancher ledger search et ses options fermées dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-daemon/src/cli.rs ; attendu : mêmes filtres/résultats que MCP, --json et rendu des caractères de contrôle inerte, aide claire ; ledger --limit inchangé. Couverture : FR-002, FR-004, FR-012, FR-015. Scénarios : S04, S27, S29.

## US2 — Continuer sans omission

- [x] T011 [US2] Écrire les tests curseurs, budgets et corpus vivant dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-daemon/tests/search_104_test.rs ; attendu : page vide avec suite, ID dupliqué vers deux cibles, purge, ID trop grand, erreur SQL, dernière ligne traitée différente de la dernière occurrence. Couverture : FR-005, FR-006, FR-008, FR-010, SC-002, SC-003. Scénarios : S07–S15.

- [x] T012 [US2] Implémenter validation/empreinte/encodage du curseur dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-daemon/src/ledger.rs ; attendu : query brute, filtres et acteur liés, upper inclusive et before exclusive, taille maximale de 16 384 octets, aucun pouvoir d’accès porté par le curseur ; un curseur forgé ne révèle aucun échange tiers. Couverture : FR-005, FR-006, FR-008, SC-002. Scénarios : S07, S08, S11, S12.

- [x] T013 [US2] Finaliser pagination et réponse bornée dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-daemon/src/store/ledger_requests.rs ; attendu : 129e clé témoin, curseur sur la dernière clé effectivement consommée, has_more ne garantit pas une occurrence future, résultat dépassant la sortie de 60 Kio non consommé, erreurs jamais remplacées par des résultats vides. Couverture : FR-004, FR-005, FR-006, FR-010, SC-002, SC-003. Scénarios : S07–S15.

- [x] T014 [US2] Exécuter l'oracle complet et les courses par pages dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-daemon/tests/search_104_test.rs ; attendu : toutes les occurrences du corpus fixe exactement une fois ; caractère vivant du corpus annoncé, requête textuellement différente refusée avec l’ancien curseur. Couverture : FR-001, FR-005, FR-006, SC-001, SC-002. Scénarios : S01, S07–S13.

## US3 — Lire la source précise et les fils

- [x] T015 [US3] Écrire les tests de relecture exacte dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-daemon/tests/search_104_test.rs ; attendu : id+target, offsets UTF-8, digest, changement/purge/tiers, corps à la borne de 16 Mio, zéro voisin implicite. Couverture : FR-007, FR-008, SC-003. Scénarios : S16–S18, S30.

- [x] T016 [US3] Implémenter lecture par clé primaire et fragments/digest dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-daemon/src/store/ledger_requests.rs ; attendu : vérifier les droits avant corps/hash, limite de 16 Mio, fragment de 16 Kio, content_changed sans morceau ; aucune confusion entre ID et cible. Couverture : FR-007, FR-008, FR-010, SC-003. Scénarios : S16–S18.

- [x] T017 [US3] Exposer LedgerRead au daemon et actions read CLI/MCP dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-daemon/src/mcp.rs ; attendu : mettre à jour daemon.rs et cli.rs ; mêmes contrôles d’identité et permis de lecture seule, erreurs définies, read depuis match_offset+digest fonctionne. Couverture : FR-007, FR-008, FR-012. Scénarios : S16–S18, S27, S30.

- [x] T018 [US3] Écrire les tests recherche de fils 102 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-daemon/tests/search_104_test.rs ; attendu : membre, non-membre et fil absent, borne 200 avec entrée 201 concurrente, références history, aucune lecture confirmée. Couverture : FR-003, FR-006, FR-009, FR-014, SC-004. Scénarios : S19–S21.

- [x] T019 [US3] Intégrer source thread aux accès Store 102 depuis /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-daemon/src/ledger.rs ; attendu : réutiliser l’appartenance dans la même transaction de lecture, PK(thread_id, seq), filtres/bornes, aucun read/ack ; si 102 absente : capability_unavailable, état intermédiaire non livrable. Couverture : FR-003, FR-005, FR-006, FR-008, FR-014. Scénarios : S19, S20, S31.

- [x] T020 [US3] Prouver les non-mutations et révocations dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-daemon/tests/search_104_test.rs ; attendu : tables demandes, lectures 102, opérations et sollicitations inchangées, aucun message envoyé, identité révoquée pendant SQL : aucun résultat émis. Couverture : FR-008, FR-009, FR-014, SC-004. Scénarios : S21, S22.

## US4 — Réactivité et parité

- [x] T021 [US4] Achever la matrice de compatibilité dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-daemon/tests/search_104_test.rs ; attendu : ancien ledger inchangé, nouveau daemon et client alignés, ancien serveur : erreur honnête sans repli, pas de nouvel outil MCP. Couverture : FR-010, FR-012, SC-007. Scénarios : S27, S31.

- [x] T022 [US4] Retirer les anciennes variantes mortes de recherche dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-daemon/src/store/ledger_requests.rs ; attendu : rg des appels avant retrait, un seul repli et un seul moteur ; aucune suppression d’un helper encore utilisé ailleurs. Couverture : FR-001, FR-011, FR-012. Scénarios : S02, S23.

- [x] T023 [US4] Créer les mesures index, latence, mémoire, concurrence dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/crates/bridget-daemon/tests/search_104_test.rs ; attendu : 100 000 lignes de 1 Kio,200 pages et 200 messages directs, cas 16 Mio et 2 appels simultanés, troisième busy, mesure mémoire séparée du générateur de données. Couverture : FR-011, SC-005, SC-006. Scénarios : S23–S26.

- [x] T024 [US4] Exécuter et analyser ces mesures puis consigner les résultats dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/specs/104-recherche-echanges/implementation.md ; attendu : p95, mémoire maximale, build, hôte et EXPLAIN réels ; corriger les écarts dans les accès existants sans réduire le corpus ni promettre des résultats non mesurés. Couverture : FR-011, SC-005, SC-006. Scénarios : S23–S26.

- [x] T025 [US4] Documenter chercher/continuer/lire/citer dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/skills/bridget/references/commandes.md ; attendu : mettre à jour SKILL.md et README.md ; limites de rétention, visibilité, Unicode et instantané, curseur exact et recherche partielle, pas de parcours complet automatique. Couverture : FR-013, FR-014, FR-015. Scénarios : S28, S29.

## Validation finale

- [x] T026 Exécuter fmt/tests/clippy et régressions après audit du harnais dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/specs/104-recherche-echanges/implementation.md ; attendu : rapport des commandes et succès/échecs, aucune commande de tests n’exécutant zéro test, nouveaux protocoles et 102, aucune erreur ignorée. Couverture : FR-012, SC-001, SC-002, SC-003, SC-004, SC-005, SC-006, SC-007.

- [x] T027 Analyser puis confronter les exigences au code livré dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/specs/104-recherche-echanges/analysis.md ; attendu : Analyze et Converge réels, dépendance 102 satisfaite, corriger les manques de tests et de métadonnées avant le statut livré. Couverture : FR-003, FR-011, FR-012.

- [x] T028 Préparer le dossier de livraison et reprise dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges/specs/104-recherche-echanges/quickstart.md ; attendu : régression 103 si intégrée, skill/catalogues, état Git, aucune installation ou commit implicite ; joindre les mesures et les limites héritées. Couverture : FR-012, FR-013.

## Dépendances et stratégie

Exécution T001→T028. Le premier incrément utile est US1,
mais la livraison de la spec exige TOUTES les stories et gates. Ce n'est pas une autorisation
à s'arrêter après un sous-ensemble. Aucun statut Implemented si une tâche reste ouverte.
La102 doit être intégrée avant T018/T019 ; si absente, poursuivre les tâches indépendantes
et conserver ces tâches explicitement bloquées, sans déclarer104 complète.

Pas de marqueur[P] : les tâches de chaque story se partagent les mêmes fichiers. Une revue
lecture seule peut se faire en parallèle ; l'écriture des fixtures/doc peut être déléguée
après accords explicites et propriété séparée. Ne pas lancer deux agents sur mcp.rs/cli.rs.

## Responsabilité et simplicité

Chaque création doit rester reliée au reuse-audit. Aucun nouveau service/table/dépendance
; seuls deux index, opérations de protocole et extensions de modules prévus.
La documentation doit permettre une reprise sans accéder à cette conversation.
Après réalisation : revue du diff, tests négatifs, Analyze, Converge, puis audit de code.
Pas de commit automatique. Installation et agents réels : autorisation nouvelle requise.
