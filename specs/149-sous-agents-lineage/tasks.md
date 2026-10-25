# Tâches — session149

**Spec**: 149-sous-agents-lineage
**Date**: 2026-10-10
**Progression**: 45/45 ; T001–T045 validées ; livraison sur disque, sans redémarrage
**Gates**: GO principal après SOURCE_ONLY_APPROVE Sonnet r2 et recettes finales réseau r4 / Codex réel r7. Héritage sans grant neuf, interop privée, journal réel, UI et reprise validés. Limites acceptées et niveaux réels/simulés consignés dans validation/prelivraison149.md. Aucun redémarrage de production.

Les cases T001–T042 sont cochées par le principal après lecture des preuves liées au code exact. Les cases de livraison restent vides jusqu'à leurs résultats effectifs. Sol réalise le code complexe. Claude écrit les tests, fixtures et relectures : Sonnet 5.5 high pour permissions, reprise et tests complexes ; Haiku 5.5 medium pour docs et tests simples ; Haiku 5.5 high pour relectures ciblées. GLM 5.3 Flash reste le fournisseur produit des recettes réelles. Le principal réalise Git et la livraison. Les commandes, recettes et bases sont isolées. Aucun restart réel n'est autorisé.

Le plan commun est `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/plan.md`. Les chemins ci-dessous sont absolus. Le partage strict des owners du plan s'applique à chaque tâche.

## Phase0 — Contrats et gate documentaire

- [x] T001 [US1–US7] Principal et owner natif : clôturer la spec formelle, les contrats permission/Lineage, le modèle de données et l'ADR dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/` ; corriger FR018, exclusions de phase et opt-out sans nouveau toggle. Preuve : artefacts cohérents lus par Haiku 5.5 high.
- [x] T002 [US1–US7] Sonnet 5.5 high : écrire la stratégie de tests et relire spec/plan/tasks/contrats dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/test-strategy.md` et `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/` ; produire deux décisions distinctes G-L Lineage et G-P permissions. APPROVE G-L puis GO principal autorisent T016–T035 indépendantes ; T003–T015 attendent G-P. La tâche entière reste non cochée tant que les deux gates ne sont pas prouvées.
- [x] T003 [US3,US4,US5] Owner natif et Sonnet 5.5 high : fermer les sources de politique des premiers parents standalone Codex et Claude/GLM ainsi que le mappage inter-fournisseurs dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/contracts/permissions.md` ; prouver la provenance réelle, sans prompt/grant supplémentaire/discovery imposé ; consigner le registre source réel de la recette — chemin absolu, entrée `glm` exacte (`gclaude` + profil `/Users/moi/.claude-glm`), digests du launcher et du CLI résolu — et noter que le registre vivant de production ne reçoit aucune entrée nouvelle sans décision humaine.

## Phase1 — Attestation effective et héritage natif

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/tasks.md#extrait-1).

## Phase2 — État natif, journal et suivi

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/tasks.md#extrait-2).

## Phase3 — Projection et surfaces adaptateur de bureau

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/tasks.md#extrait-3).

## Phase4 — Recettes complètes et revue finale

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/tasks.md#extrait-4).

## Phase5 — Sources et livraison sans activation

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/tasks.md#extrait-5).

## Dépendances et parallélisme

- T001 est validée par volet. G-L de T002 puis GO principal précèdent T016–T035. T003 et G-P de T002 puis GO principal précèdent T004–T015. Une gate partielle ne coche pas T002. Aucun changement de droits n'est admis sous la seule G-L.
- T004→T005→T006/T007→T008. T006 et T007 peuvent avancer en parallèle seulement si les fichiers restent séparés. T009 dépend du contrat et de T008 pour son interop.
- T010→T011/T012→T013/T014→T015. T009 et T010 se coordonnent sur l'admission ; owner natif unique.
- T016→T017→T018/T019/T020/T021. T022 dépend du contrat T024 mais peut être réalisé par owner natif après verrouillage de celui-ci. T023 attend les fichiers libérés.
- T024→T025/T026→T027/T028/T029→T030→T031/T032/T033→T034→T035. Le service T025 attend les contrats natifs, puis les commandes réelles de phase2.
- T036–T040 attendent les preuves ciblées T015/T023/T035 et les trois lots intégrés. Sonnet 5.5 high peut paralléliser des recettes isolées indépendantes, sans lancer deux recettes sur la même DB ou le même fichier.
- T041→T042→T043→T044→T045. Une nouvelle correction rouvre les preuves affectées ; aucun cochage de livraison anticipé.

## Mapping des exigences

| Exigence | Tâches de réalisation et preuves |
|---|---|
| US1 / FR001 / SC001 | T016,T022,T024–T027,T030–T031,T035,T037,T040 |
| US2 / FR002 / SC002 | T018,T020,T024–T025,T031–T032,T035,T039–T040 |
| FR003 | T017–T020,T023,T025–T028,T035,T039–T040 |
| FR004 | T022,T024,T026,T028,T031,T035,T040 |
| FR005 | T010,T016,T023,T026,T035,T039 |
| FR006 | T009–T010,T025,T037–T038 |
| US3 / FR007 / SC003 | T003–T015,T028,T036–T038 |
| FR008 | T011–T015,T037–T038 |
| US4 / FR009 / SC004 | T003,T010–T015,T038–T039 |
| FR010 | T014,T025–T029,T035,T039 |
| US5 / FR011 / SC005 | T004–T009,T015,T018–T020,T022–T025,T034–T036,T040 |
| FR012 | T001,T005–T009,T013,T015,T034–T036 |
| US6 / FR013 / SC006 | T010,T016–T023,T026,T035,T039 |
| FR014 | T001–T002,T039,T042 |
| FR015 | T016–T020,T023,T027,T030–T032,T039–T040 |
| FR016 | T004–T009,T013,T015,T018–T020,T023–T025,T034–T035 |
| US7 / FR017 / SC007 | T041–T045 |
| FR018 | T001,T003–T015,T036–T038 |
| FR019 | T007,T010,T013,T015,T023,T027–T028,T038,T039 |

FR018 est mappée selon sa formulation actuelle : préserver les permissions effectives du parent et retirer seulement la garde du grant humain supplémentaire dans l'héritage autorisé, sans dépasser les droits du parent. FR019 fige les droits, le modèle et la définition avant premier effet, sans élargissement au rejeu et sans effet implicite d'un changement de mode UI. Le refus corrélé en cours de mission (S149-29), la divergence de digest (S149-30), la source nécessaire opaque (S149-31) et l'union d'introspection v1/v2 (S149-32) portent aussi SC003 ; l'observer PTY (S149-33) porte SC003 et SC004. La spec reste l'autorité.

## Preuves minimales par critère

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/tasks.md#extrait-6).

La stratégie S149-01 àS149-33 est un inventaire initial à affiner par Sonnet 5.5 high avant chaque gate. Les fils virtuels durables sont permis ; seuls top-level, provider sessions/turns et outbox de lancement enfant sont interdits. Full-access ne doit pas être testé comme une sandbox interdisant toute écriture hors cwd. Un refus de cwd de mission et une limite imposée par le fournisseur sont des preuves distinctes. Un fournisseur factice qui refuse lui-même un fichier ne prouve aucune limite du fournisseur réel. Les nouveaux grants humains sont exclus du chemin positif d'héritage149.
