# Analyse SPEC-162 — deux passes documentaires

**Date :** 2026-10-13. **Portée :** spec, plan, tasks, modèle, contrats, quickstart, ADR et constitution. Aucune implémentation ni réussite de test n’est déduite.

Skills Tasks/Analyze lues. setup-tasks.sh tenté : RC127, absent ; template/extensions absents. ROOT a tenté check-prerequisites.sh --json --require-tasks --include-tasks : RC127, absent. Fallback documentaire autorisé par le pipeline. Analyse en lecture, puis corrections ciblées autorisées dans le pipeline my-specify-all, puis seconde lecture. Aucun outil installé.

## Passe 1

| ID | Catégorie | Importance | Constat | Correction appliquée |
|---|---|---|---|---|
| C1 | Propriété | Moyenne | T019 et T029 traversaient des fichiers attribués à deux owners sans moment de passation précis. | Passations explicites de wrapper.rs avant T022 et remote_worker.rs après T027 puis retour avant T031. Aucun écrivain concurrent. |
| C2 | Couverture de fichier | Moyenne | T029 décrivait aussi le relais vivant maître, mais n’incluait pas daemon.rs. | Surface et propriétaire MASTER ajoutés ; dépendance de fin worker T027 explicitée. |
| C3 | Lisibilité | Faible | Plusieurs descriptions reprenaient une sténographie peu lisible. | Espaces et termes explicites corrigés sans ajout de périmètre. |
| C4 | Vérification opérationnelle | Faible | T040 ne rappelait pas le budget de rétention fraîchement décidé. | Cache utile 162/163 conservé avec budget temporaire40Gio, preuve existante citée ; pas nouvelle purge ni test. |

Ces constats étaient documentaires. Aucun constat de code, aucune nouvelle architecture et aucune nouvelle suite de tests.

## Passe 2 — résultat

42 tâches séquentiellement identifiées, 0 cochée. Dépendances acycliques vérifiées. Chaque tâche a owner, fichiers absolus, résultat observable, dépendances et mapping FR/SC. Les seules cibles encore absentes sont le module worker justifié et les futurs reçus/journaux documentaires. Les fichiers sources existants cités ont été vérifiés par existence. Les cinq US ont leur oracle indépendant et un groupe de tests prévu.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/analysis.md#extrait-1).

## Couverture

Les mappings ci-dessous excluent T042, qui relit transversalement mais ne remplace pas une tâche d’implémentation. T040 est un contrôle final, pas l’unique preuve d’une exigence. G8/G9 apportent les preuves de production que les tâches préparatoires ne peuvent pas fournir.

| Exigence | Tâches correspondantes | Gate réelle si nécessaire |
|---|---|---|
| FR-001 | T002, T007, T009, T011, T014, T040 | G8 |
| FR-002 | T002, T003, T006, T007, T014, T040 | Tests isolés ; G8 pour route réelle |
| FR-003 | T002, T003, T008, T014, T017, T040 | Tests isolés ; G8 pour route réelle |
| FR-004 | T004, T006, T007, T011, T012, T014, T023, T024, T030, T040 | Tests isolés ; G8 pour route réelle |
| FR-005 | T003, T007, T009, T011, T013, T014, T016, T040 | Tests isolés ; G8 pour route réelle |
| FR-006 | T003, T006, T009, T015, T016, T018, T019, T020, T040 | Tests isolés ; G8 pour route réelle |
| FR-007 | T009, T015, T016, T017, T018, T019, T020, T040 | Tests isolés ; G8 pour route réelle |
| FR-008 | T010, T015, T016, T017, T018, T019, T020, T040 | Tests isolés ; G8 pour route réelle |
| FR-009 | T004, T005, T010, T011, T014, T021, T022, T028, T040 | Tests isolés ; G8 pour route réelle |
| FR-010 | T002, T004, T005, T010, T014, T021, T022, T023, T024, T025, T027, T028, T040 | Tests isolés ; G8 pour route réelle |
| FR-011 | T010, T011, T014, T019, T028, T040 | Tests isolés ; G8 pour route réelle |
| FR-012 | T005, T008, T010, T012, T022, T023, T025, T026, T027, T028, T031, T040 | Tests isolés ; G8 pour route réelle |
| FR-013 | T002, T005, T012, T014, T021, T022, T027, T028, T040 | Tests isolés ; G8 pour route réelle |
| FR-014 | T002, T008, T010, T011, T014, T023, T024, T025, T026, T028, T040 | Tests isolés ; G8 pour route réelle |
| FR-015 | T019, T031, T032, T040 | Tests isolés ; G8 pour route réelle |
| FR-016 | T004, T012, T021, T026, T028, T040 | Tests isolés ; G8 pour route réelle |
| FR-017 | T004, T021, T026, T028, T040 | Tests isolés ; G8 pour route réelle |
| FR-018 | T029, T030, T031, T032, T040 | Tests isolés ; G8 pour route réelle |
| FR-019 | T006, T029, T030, T031, T032, T036, T037, T039, T040 | Tests isolés ; G8 pour route réelle |
| FR-020 | T033, T034, T035, T036, T037, T038, T039, T040 | Tests isolés ; G8 pour route réelle |
| FR-021 | T008, T034, T035, T037, T038, T039, T040 | Tests isolés ; G8 pour route réelle |
| FR-022 | T002, T033, T034, T035, T036, T038, T039, T040 | Tests isolés ; G8 pour route réelle |
| FR-023 | T029, T032, T033, T034, T036, T037, T038, T039, T040 | Tests isolés ; G8 pour route réelle |
| FR-024 | T001, T003, T015, T018, T020, T040, T041 | Tests isolés ; G8 pour route réelle |
| FR-025 | T002, T003, T006, T008, T014, T017, T020, T040, T041 | G8 |
| FR-026 | T001, T004, T041 | G8 |
| FR-027 | T001, T038, T039, T041 | G8 |
| FR-028 | T001, T027, T041 | G9 |
| FR-029 | T005, T021, T022, T023, T024, T025, T026, T027, T028, T030, T032, T040 | Tests isolés ; G8 pour route réelle |
| FR-030 | T009, T013, T014, T040 | Tests isolés ; G8 pour route réelle |
| SC-001 | T007, T009, T010, T011, T013, T014, T040, T041 | G8 |
| SC-002 | T002, T003, T006, T007, T008, T009, T012, T013, T014, T040 | Tests isolés ; G8 pour route réelle |
| SC-003 | T003, T009, T015, T016, T017, T018, T019, T020, T040 | Tests isolés ; G8 pour route réelle |
| SC-004 | T002, T004, T005, T008, T010, T011, T012, T014, T017, T019, T020, T021, T022, T023, T024, T025, T026, T027, T028, T040 | Tests isolés ; G8 pour route réelle |
| SC-005 | T019, T031, T032, T040, T041 | Tests isolés ; G8 pour route réelle |
| SC-006 | T004, T012, T021, T026, T028, T040 | Tests isolés ; G8 pour route réelle |
| SC-007 | T005, T006, T021, T022, T023, T024, T025, T026, T027, T028, T029, T030, T031, T032, T040, T041 | Tests isolés ; G8 pour route réelle |
| SC-008 | T008, T033, T034, T035, T036, T037, T038, T039, T040 | Tests isolés ; G8 pour route réelle |
| SC-009 | T002, T033, T034, T035, T036, T037, T038, T039, T040 | Tests isolés ; G8 pour route réelle |
| SC-010 | T002, T003, T005, T012, T014, T021, T022, T027, T028, T040 | Tests isolés ; G8 pour route réelle |
| SC-011 | T001, T004, T038, T039, T041 | G8 |
| SC-012 | T001, T013, T027, T041 | G9 |

## Constitution, minimalisme et responsabilité

ADR présent, worktrees dédiés et propriétés explicites. Réemploi PASS47/47 lu par ROOT, contre-revue R2 corrigée. Aucun store métier worker, seconde Fleet, dépendance, miroir de journal, moteur Git ou framework d’outbox. Le nouveau module, le store technique et les opérations supplémentaires portent une frontière de sécurité attestée. Les tests comptent des effets durables, pas seulement des mocks. Les passations de fichier limitent les conflits.

La nouvelle frontière distante augmente la charge de maintenance ; le plan la rend explicable par une autorité maître unique, un écrivain par fichier, une voie finale de résultat et des dispositions fermées. T042 impose une relecture humaine responsable et les axes complexité/minimalisme. Pas de lignes supprimables estimables sans code produit : aucune estimation fictive. Les gates G8/G9 rendent commit, intégration, push, installation, activation et nettoyage obligatoires sans circularité avec les cases de développement.

## Mesures

| Mesure | Valeur |
|---|---:|
| Exigences fonctionnelles | 30 |
| Critères de succès | 12 |
| Histoires utilisateur | 5 |
| Tâches de développement/préparation | 42 |
| Tâches cochées | 0 |
| Exigences/critères avec tâche | 42/42, 100 % |
| Tâches sans besoin associé | 0 |
| Dépendances circulaires | 0 |
| Constats actifs après passe2 | 0 |
| Gates opérationnelles encore ouvertes | G8, G9 |

Répartition : préparation1, fondations5, US1=8, US2=6, US3=8, US4=4, US5=7, final3. Quatre tâches marquées parallélisables : T002, T003, T015 et T035 sont des possibilités selon leurs dépendances. Elles n’autorisent ni compilation concurrente ni propriétaire concurrent d’un fichier.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/162-execution-distante/analysis.md#extrait-2).

Les projets réels restent en attente d’autorisation et de paramètres. Cela ne bloque pas le code complet ni les recettes isolées ; cela bloque uniquement la route réelle concernée en G8. Aucun choix par défaut inventé.

## Suite autorisée

Implémenter les fondations T001–T006 puis suivre les dépendances. Aucun blocage documentaire restant. ROOT conserve le gate de lancement des owners, Git et production.

SHA-256 de tasks.md analysé : 56a8a2f15e85493a9e99dc5f707681528b8f43e457c2ec6eae158d0bd5025431

## Ajustement de dépendance à l’implémentation

Lecture ROOT de cli_lineage.rs, lineage_client.rs et lineage_protocol.rs : T033 ne lit ni Task ni Fleet. Sa dépendance T004 était superflue et est retirée ; le contrat Lineage T002 figé suffit. Les validations regroupées restent obligatoires et les cases restent ouvertes jusque preuve. Une interface de code figée peut débloquer son consommateur avant ces validations groupées, sans prétendre que la tâche est validée. Aucun changement fonctionnel ou ajout de périmètre.

## Contrôle de cohérence T044 — checkpoint après R4

Analyse documentaire ciblée, sans nouveau cycle de test. La correction T044 reste rattachée aux FR-009/012/014/015 et SC-004/005/007 ; aucun nouveau besoin ni nouvelle surface métier n’est ajouté. Les preuves de primitives avant Ready et les scénarios après RELEASE restent distingués. La durée de six secondes exerce le permis cinq secondes et ne démontre pas une expiration 30 secondes.

Le compteur reste 43/44, In Progress, car le parcours répété de l’inventaire worker trouvé après R4 attend sa correction et son replay. Les gates G8/G9, le caractère historique de Converge2 et l’absence de Converge3 sont cohérents. Les 77 PASS R4 ne remplacent pas la FULL historique en échec. Aucun gain de latence ni verdict de convergence supplémentaire n’est déduit. La vérification finale de cohérence et le gel de tasks.md attendent la preuve du correctif groupant la lecture.

## Analyse manuelle finale après T044 — deux lectures, fallback conservé

Le runtime SpecKit absent reste celui constaté plus haut ; aucun outil n’a été installé. Première lecture ciblée : le dernier checkpoint devait distinguer le reliquat N+1 après R4 de sa résolution R5 et conserver le statut de livraison ouvert. Autofix documentaire autorisé : ajout des preuves R5, mise à jour des en-têtes 44/44 Implemented et conservation des checkpoints antérieurs comme historiques. Aucun changement de périmètre ni nouvelle tâche.

Seconde lecture : T044 reste liée aux FR-009/012/014/015 et SC-004/005/007. Réconciliation maître hors mutex, retained Child jusqu’à arrêt attesté, slots vides clos, Deferred conservé et inventaire worker groupé sont décrits avec leurs preuves R4/R5. Les deux scénarios ajoutés enrichissent quatre fonctions E2E existantes ; aucun compte gonflé. Le retard six secondes ne revendique pas un essai 30 secondes. Le mapping 30 FR/12 SC/5 US reste inchangé. Les tâches totalisent 44 cochées et zéro ouverte ; G8/G9, revue et Converge3 ne sont pas déduits de ce compteur. Zéro incohérence active trouvée dans cette relecture ciblée ; aucune nouvelle campagne de tests.

SHA-256 final gelé de tasks.md : 746adf1b544e476ffe21e79131cb9ec2ad0b89c6bd90c0870475ecd242e800ea. Ne plus modifier les tâches pendant Converge3.


## Checkpoint ciblé T045/T046 — 13 octobre 2026

Le fallback manuel précédent reste applicable. ROOT a relu le diff produit final, les 30 FR, les 12 SC et les artefacts. Aucun besoin nouveau ni extension de périmètre n’a été identifié. La revue T045, deux tests de bibliothèque et cinq E2E Linux permettent de valider le correctif source sans nouvel écart retenu. Le contrôle UID divergent reste une preuve de lecture, pas un test runtime réussi.

Le compteur 45/46 est cohérent : T045 est cochée ; T046 reste ouverte pour la recette réelle après installation. Cette séparation évite d’exiger la recette du binaire installé avant son commit et son installation. Le statut reste In Progress. G8 et G9 restent ouverts ; aucun nouveau passage global de convergence ou de tests n’est revendiqué. Le fichier tasks.md est figé après cochage, SHA-256 c35f2d559f151575d953e31a348e4a1739b8fd65773c88e93b8089699a976196.


## Checkpoint ciblé après recette T046

La preuve réelle d’annulation complète les sept essais Linux ciblés. T046 et G8 sont validés ; 46/46 tâches cochées, statut Implemented. ROOT a relu les correspondances code, documents et tests sans nouvel écart. Le contrôle UID divergent reste une preuve de lecture, sans prétention de test runtime. Le nettoyage G9 reste ouvert. Aucun nouvel audit global ou passage FULL n’est déduit.

Le fichier tasks.md est figé après cochage de T046 : SHA-256 44d552e8cf891ef1789b26ab3c85f8a670ea88e617220222aae7849d609adb06.
