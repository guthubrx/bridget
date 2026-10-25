# Archives techniques de l’adaptateur T3

Document d’origine : [specs/162-execution-distante/tasks.md](../../../../../../specs/162-execution-distante/tasks.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Les owners sont des responsabilités, affectées à une personne/agent par ROOT avant édition. Aucun fichier ne reçoit deux écrivains simultanés. MASTER garde daemon.rs et native_delegation.rs ; WORKER garde remote_worker.rs et wrapper.rs ; PERMISSIONS garde native_permissions.rs ; TRANSPORT garde les contrats wire ; REGISTRY garde registry.rs ; T3 garde son dépôt ; ROOT garde Git, production, reçus et vérifications. Une tâche touchant une interface partagée exige une passation explicite de fichier, pas une édition concurrente. Ne jamais révoquer le travail étranger.

<a id="extrait-2"></a>

## Extrait 2

- [x] T035 [P] [US5] Owner T3 — Étendre schémas T3 avec v2 et métadonnées optionnelles distinctes de v1 ; garder ancien décodeur strict et fixture v1 gelée. Aucun nouveau store ou fil fournisseur. Fichiers : /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/packages/contracts/src/bridgetLineage.ts. Dépendances : T002. Couverture : FR-020, FR-021, FR-022 ; SC-008, SC-009.

<a id="extrait-3"></a>

## Extrait 3

- [x] T036 [US5] Owner T3 — Négocier v2 puis une lecture v1 seulement sur invalid_request/unsupported_version. Abandonner tout le staging avant curseur nul ; timeout/auth/malformation restent erreurs. Mêmes RPC et gardes, aucun repli de mutation. Fichiers : /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/apps/server/src/bridget/BridgetReader.ts ; /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/apps/server/src/bridget/BridgetLineage.ts. Dépendances : T033, T035. Couverture : FR-019, FR-020, FR-022, FR-023 ; SC-008, SC-009.

<a id="extrait-4"></a>

## Extrait 4

- [x] T037 [US5] Owner T3 — Propager placement/contact dans la projection thread:bridget-task existante, activeProviderThreadId nul et pas de colonne conversation. Préserver root/projet, profondeur 8, historique et atomicité des pages ; reprise de vue sans lancement fournisseur. Fichiers : /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/apps/server/src/orchestration-v2/Orchestrator.ts. Dépendances : T034, T036. Couverture : FR-019, FR-020, FR-021, FR-023 ; SC-008, SC-009.

<a id="extrait-5"></a>

## Extrait 5

- [x] T038 [US5] Owner T3 — Afficher machine/moteur/date du contact observée par page et fraîcheur dans le Lineage et les détails existants. Distinguer maître accessible/worker perdu et métadonnées v1 indisponibles ; journal indisponible explicite, pas d’âge vivant inventé. Fichiers : /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/apps/web/src/components/chat/ThreadRelationshipsControl.tsx ; /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/apps/web/src/components/chat/SubagentTooltipContent.tsx ; /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant/apps/web/src/components/BridgetTaskJournal.tsx. Dépendances : T035, T037. Couverture : FR-020, FR-021, FR-022, FR-023, FR-027 ; SC-008, SC-009, SC-011.

<a id="extrait-6"></a>

## Extrait 6

- [x] T039 [US5] Owner ROOT — Exécuter les suites ciblées 149 des contrats/reader/orchestrateur/composants étendues : v1 stricte, v2 avec version erronée, erreurs v1, repli unique, page 2 refusée, transition de contact, droits, ouverture/fermeture sans lancement. Pas suite T3 globale, navigateur ou redémarrage. Fichiers : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/validation/tests162.md. Dépendances : T032, T034, T038. Couverture : FR-019, FR-020, FR-021, FR-022, FR-023, FR-027 ; SC-008, SC-009, SC-011.

<a id="extrait-7"></a>

## Extrait 7

- [x] T041 Owner ROOT — Préparer procédure exécutable et reçu NON EXÉCUTÉ : sauvegarde jointe et rollback strict, unité utilisateur bridget-worker-cartae.service avec SIGTERM/KillModeprocess/SendSIGKILLno, credential en 0600 / état en 0700, route désactivée puis activation autorisée. Lier préflight des missions et recette enfant synthétique survivantredémarrage worker, sans relance T3. Inventorier TMP/WT/branches/rollback. Fichiers : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/quickstart.md ; /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/validation/livraison162.md. Dépendances : T032, T039. Couverture : FR-024, FR-025, FR-026, FR-027, FR-028 ; SC-001, SC-005, SC-007, SC-011, SC-012.

<a id="extrait-8"></a>

## Extrait 8

1. Exécuter Converge sur le développement, sans modifier tasks.md durant ses passes. Réaliser contre-revue de code et audit requis ; traiter leurs blocages. Les corrections testées ne nécessitent pas une seconde FULL sans raison nouvelle.
2. Commits avec identité utilisateur, hooks normaux ; intégrer Bridget vers github/main et T3 vers fork/local/main-20261009, vérifier les références distantes. Aucun push upstream ni PR T3. Un commit de préparation ne vaut pas livraison opérationnelle.
3. Depuis les sources intégrées, construire, signer et installer les binaires/paquets avec provenance. T3 est livré sur disque uniquement ; PID/naissances restent inchangés.
4. Avant activation Bridget : fermer nouvelles admissions si nécessaire, attester missions actives et sauvegarde cohérente DB/Fleet/registre/worker. Aucun arrêt forcé inconnu ni downgrade des stores 162 sur ancien binaire. Conserver preuves post-162 ; rollback joint seulement sans perte nouvelle, sinon rester en 162 route désactivée.
5. Installer l’unité utilisateur worker, credential privé et état sans secret exposé ; tunnel 095 existant conservé. Tester avec enfant synthétique que l’arrêt du worker ne vaut pas annulation des missions et que la reprise réconcilie sans doublon.
6. Activer uniquement les projets autorisés après réponse utilisateur et preuve fournisseur réelle. Exécuter recette Linux bornée pour chaque route autorisée : machine/cwd/moteur, résultat unique, parent retrouvé, journal et annulation attestée. Aucun choix de projet par défaut déduit d’une question utilisateur. Si route non autorisée, la nommer non activée ; ne pas déclarer SC-001 réel satisfait pour elle.

<a id="extrait-9"></a>

## Extrait 9

1. Archiver et vérifier les empreintes des preuves, reçu compact, sauvegardes utiles et état post-162 avant tout retrait.
2. Ne jamais collecter une exécution inconnue, un résultat différé, un blob dont le producteur vit, une branche non récupérée ou un worker injoignable. Aucun effacement de base. Suivre les règles de collecte du contrat.
3. Retirer sans force les deux worktrees 162 propres et intégrés ainsi que leurs branches devenues inutiles. Les17worktreesT3préexistants restent hors périmètre. Nettoyer seulement TMP/stages/caches propres attestés avec les gardes du projet ; justifier les conservations sans faux gain physique.
4. Commiter/pousser les métadonnées finales depuis le checkout durable, vérifier état Git et références distantes. Nommer séparément source binaire, commit de clôture, runtime actif et T3 ancien chargé. Clôturer l’objectif seulement après G8 et G9 prouvés.

<a id="extrait-10"></a>

## Extrait 10

- US1 : T002 et T003 sont indépendantes après T001 ; aucun autre chevauchement implicite des fichiers Rust.
- US2 : T015 peut préparer les fixtures pendant US1 après T002/T003, puis propriétaire PERMISSIONS jusqu’à passation T019.
- US3 : construction du blob et admission maître ont des fichiers distincts mais une frontière de contrat commune ; ROOT peut les alterner après T021, sans faire écrire deux agents sur remote_worker.rs ou native_delegation.rs.
- US4 : journal worker et notification parent ont des fichiers distincts après US3 ; sérialiser annulation et journal si même owner/fichier worker.
- US5 : T035 contrats T3 est indépendante du code Rust après T002 ; T036 peut suivre pendant US2/US3. T037 attend T034, puis T038/T039.

<a id="extrait-11"></a>

## Extrait 11

Réemplois réduisant la charge future : lifecycle/managed_process, mêmes stores métier, mêmes notifications/wait, journal vivant existant, lecteur Lineage et vues T3. Complexité nouvelle justifiée par ADR : frontière worker, store technique avec un écrivain par fichier, parent_proof, résultat durable et réserve technique. Aucun moteur Git, miroir de journal, seconde Fleet, nouvelle dépendance ou outil de maintenance générique.

<a id="extrait-12"></a>

## Extrait 12

Le quota Fleet global peut être réduit par un unknown distant. Les modes0600/0700 ne séparent pas des fournisseurs du même UID ; les gardes de chemins évitent l’exposition explicite, pas un accès global hostile. Le credential reste hors environnement enfant. Les blobs vivants non clôturés peuvent saturer le budget. T3 n’est pas relancé. Ces limites doivent rester dans le reçu, sans masquer une absence de recette réelle.
