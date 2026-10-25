# Tâches SPEC146 — Panneau Bridget plus lisible

Date : 2026-10-08. Statut : Implemented ; 10/10 terminées. Gate conception, réutilisation et Analyze : PASS. GO code donné aux trois responsables.

Estimation globale validée : 45–75 minutes. Découpage résiduel depuis le GO code : 35–55 minutes pour implémentation et contrôles, hors incident nouveau de build ou de recette. Les durées de lignes ne s'additionnent pas : trois responsabilités distinctes peuvent avancer en parallèle.

Les helpers SpecKit attendus sont absents, constat déjà documenté. Les artefacts suivent le protocole appliqué manuellement par le principal ; aucun helper exécuté n'est revendiqué.

## Règles de travail

Les étapes RED doivent être exécutées et consignées avant leur GREEN. Un échec pour import ou compilation absent n'est pas une preuve fonctionnelle du défaut : distinguer cette étape et conserver au moins une preuve observable sur le socle.

Les tâches `[P]` ont des fichiers de production distincts. Les responsables ne sont pas seuls dans les worktrees. Ils préservent les changements des autres et n'écrivent pas dans leurs fichiers. Les contrats figés sont la frontière commune. Les tests restent des fixtures isolées, jamais les bases ou les agents actifs.

Les dix tâches sont des unités vérifiables, pas une autorisation de production. Aucun commit, fusion, push, installation, déploiement, redémarrage ou cleanup n'est inclus.

## US1 — Fils et messages récents

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/tasks.md#extrait-1).

## US2 — Présentation plus lisible

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/tasks.md#extrait-2).

## US3 — Vérification et clôture

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/tasks.md#extrait-3).

## Correspondance des exigences

| Exigence | Tâches principales |
| --- | --- |
| FR146-01/02/03 — ordre et pagination | T001–T004, T007, T009 |
| FR146-04/05/06/07 — présentation, copie et recherche | T005/T006, T007/T008, T009 |
| FR146-08/09 — autorisation, stale et non-mutation | T001–T007, T009 |
| FR146-10 — accès clavier, largeur et erreurs | T002–T008, T009 |
| SC146-01/02 | T001–T004, T007 |
| SC146-03/04/05 | T005/T006, T007/T008 |
| SC146-06 | T001–T007, T009 |
| SC146-07 | T008 |
| SC146-08 | T001/T003/T005 RED, T002/T004/T006 GREEN, T007/T009 |

Ordre de sortie : trois pistes RED → GREEN indépendantes, puis intégration → aperçu → Converge → audit et scoring → Converge après éventuelle correction de code → clôture. Le principal a autorisé le code après Analyze ; ce document ne prouve aucun succès d'implémentation.

Preuve T003/T004 relue par le principal : `/Users/moi/.cache/bridget-data146.2WukXz/validation.md`. RED initial4 FAIL/61 PASS puis durcissement2 FAIL/22 PASS ; GREEN71 PASS/0 FAIL en3 suites (45 contrats,24 Reader,2 runtime). Types contrats/serveur exit0, huit suggestions Effect héritées consignées. Fmt, lint et diff exit0. Cette preuve ne couvre ni Rust, ni navigateur, ni intégration wire complète T007.
