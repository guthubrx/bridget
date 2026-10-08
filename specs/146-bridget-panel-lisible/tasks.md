# Tâches SPEC146 — Panneau Bridget plus lisible

Date : 2026-10-08. Statut : Implemented ; 10/10 terminées. Gate conception, réutilisation et Analyze : PASS. GO code donné aux trois responsables.

Estimation globale validée : 45–75 minutes. Découpage résiduel depuis le GO code : 35–55 minutes pour implémentation et contrôles, hors incident nouveau de build ou de recette. Les durées de lignes ne s'additionnent pas : trois responsabilités distinctes peuvent avancer en parallèle.

Les helpers SpecKit attendus sont absents, constat déjà documenté. Les artefacts suivent le protocole appliqué manuellement par le principal ; aucun helper exécuté n'est revendiqué.

## Règles de travail

Les étapes RED doivent être exécutées et consignées avant leur GREEN. Un échec pour import ou compilation absent n'est pas une preuve fonctionnelle du défaut : distinguer cette étape et conserver au moins une preuve observable sur le socle.

Les tâches `[P]` ont des fichiers de production distincts. Les responsables ne sont pas seuls dans les worktrees. Ils préservent les changements des autres et n'écrivent pas dans leurs fichiers. Les contrats figés sont la frontière commune. Les tests restent des fixtures isolées, jamais les bases ou les agents actifs.

Les dix tâches sont des unités vérifiables, pas une autorisation de production. Aucun commit, fusion, push, installation, déploiement, redémarrage ou cleanup n'est inclus.

## US1 — Fils et messages récents

- [x] T001 [P] [US1] **Rust RED — 5–8 min.** Responsable Rust, fichiers de tests et modules Rust145 ciblés. Prouver l'ancienne première page UUID et ASC sur des fixtures multipages. Ajouter les cas activité de dernière séquence plutôt que maximum des dates, égalités UUID ASC, fil vide, fermé et non-membre, limites et curseur canonique invalide. Décrire les attentes DESC count/bytes, snapshot, correction visible au-delà de before mais dans snapshot, future correction exclue, fin de fil et non-mutation. Conserver sorties RED avant T002. Couvre FR146-01/02/03/08/09, SC146-01/02/06/08.
- [x] T002 [P] [US1] **Rust GREEN — 8–12 min, après T001.** Responsable Rust : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/crates/bridget-daemon/src/store/threads.rs`, threads.rs, daemon.rs, cli.rs, communication/client.rs et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/crates/bridget-transport/src/protocol.rs`. Implémenter list_recent/ListedRecent et history_recent/HistoryRecent, résumé distinct et singleton Recent. Étendre les deux gardes humaines sans maintenance et négociation froide. Garder145 inchangé. Calculer has_more par existence d'une entrée plus ancienne que min émis et next=min−1 seulement alors ; borne d'octets ne saute aucune entrée. Tests GREEN ciblés, CLI froide, ancien serveur refusé, nouveaux singletons et données métier inchangées. Couvre les mêmes exigences que T001 et incompatibilité FR146-10.
- [x] T003 [P] [US1] **Frontière T3 RED — 4–6 min.** Responsable contrats/reader, tests de `/Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/packages/contracts/src/bridget.ts` et `/Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/server/src/bridget/BridgetReader.test.ts`. Fixer fixtures wire Rust/T3, rejets champs inconnus, ordres incohérents, doublons, timestamps non sûrs, curseur non canonique ou non aligné au dernier fil, next_seq faux, page vide avec suite. Tester traduction des options, refus incompatibles et absence de shell. Consigner RED avant T004. Couvre FR146-01/02/03/08/09/10, SC146-01/02/06/08.
- [x] T004 [P] [US1] **Frontière T3 GREEN — 6–9 min, après T003.** Responsable contrats/reader uniquement : `/Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/packages/contracts/src/bridget.ts` et `/Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/server/src/bridget/BridgetReader.ts`, avec leurs tests. Ajouter unions strictes récentes et argv séparés, sans élargir Show145 ou changer sa sémantique. Conserver budgets, parsing, annulation et nettoyage des refus. Faire passer les tests avec les fixtures wire alignées sur Rust ; aucun changement du panneau dans cette tâche. Couvre les mêmes exigences que T003.

## US2 — Présentation plus lisible

- [x] T005 [P] [US2] **Panneau RED — 4–6 min.** Responsable UI, `/Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.test.tsx` et tests du petit composant de message si extrait. Prouver ordre récent initial et pagination ancienne, fusion d'un UUID mis à jour avec nouvelle date puis retri, aperçus longs, dépliage/repliage, original exact, copie entière repliée, recherche sur partie cachée, types français et détails de remplacement. Ajouter auteurs visibles, clavier et non-mutation des interactions. Consigner RED avant T006. Couvre FR146-04/05/06/07/08/09/10, SC146-03/04/05/06/08.
- [x] T006 [P] [US2] **Panneau GREEN — 8–12 min, après T005.** Responsable UI uniquement : `/Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.tsx` et éventuel petit composant de message natif associé. Employer actions récentes ; fusionner résumé/date par UUID puis trier date DESC et UUID ASC. Deux lignes compactes pour titre/membres/date, sélection perceptible et dates courtes françaises avec date complète consultable. Corps texte original avec aperçu quatre lignes, Déplier/Replier, copie native entière et détails accessibles. Séparations neutres et icône monochrome. Préserver contextes145, recherche locale et refresh manuel ; aucun polling ou réglage global. Couvre les mêmes exigences que T005.

## US3 — Vérification et clôture

- [x] T007 [US3] **Intégration ciblée — 6–10 min, après T002/T004/T006.** Principal et responsables des frontières, sans ownership supplémentaire de production. Lire de vraies sorties Rust sérialisées dans les schémas T3 et exécuter les tests ciblés unifiés. Rejouer non-régressions145 : appartenance/révocation, liaisons absente/ambiguë, incompatible, timeout, daemon absent, fermeture et A → B → A réel du runtime. Comparer données métier avant/après lectures et refus ; zéro ACK/wake/modèle/émission. Contrôler type, lint et builds applicables en cache privé ; relever warnings et limites. Couvre tous FR146 et SC146-01 à06/08.
- [x] T008 [US3] **Aperçu isolé — 5–8 min, après T007.** Principal, composants réels et données de test. Vérifier liste et historique récents, détails, deux états des corps, copie, recherche cachée, panneau étroit et clavier. Relever les données réellement rendues et limites de recette ; conserver capture si disponible. Aucun accès à la base réelle ni restart de l'application installée. Couvre FR146-04/05/06/07/10, SC146-03/04/05/07.
- [x] T009 [US3] **Converge et audit final — 5–8 min, après T007/T008.** Principal, lecture du diff et des preuves. Exécuter Converge en préservant les tâches existantes, avec seuls ajouts de manques constatés. Puis effectuer la revue adverse et l'audit disponibles, corriger les défauts prouvés dans la responsabilité correspondante, refaire leurs contrôles ; le scoring reste en lecture seule. Vérifier l'absence de dérive145, de mutation et de nouvelle dépendance. Si une correction touche le code, rejouer validations concernées et Converge. Consigner passages et limites sans prétendre à une revue inter-fournisseurs absente. Stabiliser tâches et sources, puis relecture finale. Couvre tous FR146 et SC146-01 à08.
- [x] T010 [US3] **Clôture documentaire — 2–4 min, après T009.** Responsable documentation, uniquement `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/specs/146-bridget-panel-lisible/`, sélection et bloc SPECKIT AGENTS146. Renseigner implementation.md et validation/results.json avec résultats réels, checks, limites, hashes du socle et changements146. Actualiser statut et compte tâches selon preuves. Handoff clair : implémenté ne signifie pas installé ; aucune action Git ou de production sans nouvelle autorisation. Couvre traçabilité de tous SC146.

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
