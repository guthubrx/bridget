# Rapport d'analyse - Session156 (spec, plan, tâches)

État de reprise : les écarts U1/C1/I1/M1 et les contradictions Claude/Cursor
sont corrigés, ainsi que les limites REST/V1 et de livraison. Six axes T002
levés statiquement et reuse frais24/24/gate5/5. Baseline T001774 PASS/0 FAIL/7
ignored par root ;31 IDs/oracles conservés. L’unique Analyze indépendant après
autofix est terminé : PASS documentaire, 0 CRITICAL/0 HIGH/0 MEDIUM/0 LOW.
Sa provenance figure dans le reçu final ci-dessous. Les PASS historiques
ci-dessous gardent leurs dates.
Date : 2026-10-11, révisé après les contre-revues rondes 1 et 2
(`adversarial-review-codex.md`). Analyse non destructive de
`specs/156-guidage-file/`. Le code du worktree 150 n'a pas été lu (consigne) ;
les acquis 150 sont supposés livrés.

## Verdict historique du2026-10-11

PASS avec 6 vérifications ouvertes, toutes affectées à T002. Ronde 1 : 11
objections traitées (6 par correction, 5 par réduction de périmètre). Ronde 2 :
4 points intégrés (4 dans le modèle, avec limites hors modèle documentées).

## Cadre de menace

Dans le modèle : agent qui utilise MCP, CLI, pont T3, wrapper. Hors modèle :
processus hostile du même compte qui forge des trames brutes (ADR 011, ADR
027). Chaque limite est documentée avec sa preuve (spec « Cadre de menace »,
`research.md` R0). Les tests de trames brutes sont de la défense en profondeur.

## Couverture exigences vers tâches

| Exigence | Tâches |
|---|---|
| FR-001, FR-003 | T007, T008 |
| FR-002 | T009 |
| FR-004, FR-005 | T010, T011 |
| FR-006, FR-007 | T012 |
| FR-008 | T005, T014, T015, T016 |
| FR-009 | T014, T015, T017 |
| FR-010 | T014, T017 |
| FR-011 | T017, T023 |
| FR-012 | T006, T018 |
| FR-013 | T020 |
| FR-014 | T004, T019, T023 |
| FR-015 | T014, T015 |
| FR-016 | T006, T015 |
| FR-017, FR-018 | T024, T025, T026 |
| FR-019 | T008 |
| FR-020 | T003 |
| FR-021 | T027, T028 |
| FR-022 | T003, T021 |
| FR-023 | T003, T022 |
| FR-024 | T027, T028 |

24 exigences sur 24 couvertes.

| Critère | Tâches de preuve |
|---|---|
| SC-001, SC-002 | T013 |
| SC-003 | T005, T014 |
| SC-004 | T018 |
| SC-005 | T019, T023 |
| SC-006 | T012 |
| SC-007 | T010, T029 |
| SC-008 | T017 |
| SC-009 | T024 |
| SC-010 | T017 |
| SC-011 | T021 |
| SC-012 | T022 |
| SC-013 | T016 |

13 critères sur 13 couverts.

## Couverture des contre-revues

| Point | Où |
|---|---|
| R1-1 | T017 (garde et exception interne de pause), T023 |
| R1-2 | T014, T015 (origine fabriquée), cadre de menace |
| R1-3 | T010, T011 |
| R1-4 | FR-005, contrat |
| R1-5, 6, 7, 10 | hors périmètre (spec, ADR T028) |
| R1-8 | data-model, FR-015, FR-016 |
| R1-9 | T012 |
| R1-11 | T023 |
| R2-1 | T016 (client nu, reconnexion d'agent), T002(e) |
| R2-2 | T017 |
| R2-3 | T021 |
| R2-4 | T022 |

## Constats restants

| # | Sévérité | Constat | Traitement |
|---|---|---|---|
| A1 | moyenne | Ce que reçoit l'émetteur quand un transport rejette un `steer` n'est pas relu | T002(a) ; sans avis, la doc le dit (`effect: unconfirmed`) |
| A2 | moyenne | Producteurs légitimes de `origin=Human` (attach, relais UI) hors des crates lues | T002(b) avant T015 |
| A3 | moyenne | Liste blanche T3 (Claude, Cursor) déduite de la session 129 | T002(c) |
| A4 | basse | Champs de la tâche native non relus | T002(d) |
| A5 | basse | Guidage vise 4 et non 5 | question 2 ; ADR |
| A6 | moyenne | Un `RoleHandshake{Attach}` est-il accepté sur une connexion qui a déjà une identité d'agent ? Les citations de lignes de la ronde 2 (`daemon.rs:9060-9075`, `:18125-18175`) sont approximatives | T002(e) ; l'implémenteur relit les lignes exactes |
| A7 | moyenne | Changement visible : les pairs perdent la remise en cours de tour, la CLI nue perd le contrôle | question 3 ; tests de la session 129 réécrits (T021, T029) |
| A8 | basse | Un ancien wrapper garde l'ancien `cancel_delivery` jusqu'à sa relance | noté dans la doc (T022, T027) |

## Risques de conflit avec les autres sessions

- 150, 152, 155, 157 modifient `daemon.rs`, `mcp.rs`. Exécution séquentielle.
- 152 reprend pause/panne ; interruption utilisateur terminale. T020 conserve mission_active pour un agent ; humain explicite exempt.
- Base155 livrée à revalider par T002 ; QueueList couvre seulement send_deliveries dispatching.
- La future session d'édition de file reprend les prérequis de `research.md` R8.

## Questions utilisateur (défauts retenus)

1. Un pair du même projet peut-il guider ? Défaut : non.
2. Reporter `cancel`, `reorder`, `promote` de la file à une session dédiée ? Défaut : oui.
3. Accepter deux changements visibles : les pairs attendent la fin du tour (guidage implicite T3) ; la CLI nue ne peut plus guider ? Défaut : oui.

## Préparation R1 du2026-10-12 — aucune nouvelle Analyze exécutée

Réponses aux neuf findings : /Users/moi/.cache/bridget-plan156-20261012-r1/response.md.
F1/F2/F3/F5 arbitrés ; F4/F6/F7/F8/F9 clarifiés. Les31 tâches, leurs IDs, cases
non cochées et noms des tests planifiés sont conservés. Extensions de scénarios
existants seulement. La couverture historique ci-dessus n'est pas une nouvelle
Analyze ni preuve runtime156. Application par root après livraison155, puis T002
sur base intégrée et Analyze fraîche du pipeline156.

## Reçu final de l’Analyze indépendant après autofix

Le 2026-10-12 à 04:19:25 UTC, l’unique Analyze après autofix conclut
**PASS documentaire : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 0 LOW**. Les écarts
U1/C1/I1/M1 sont résolus. La couverture est de 24 FR, 13 SC et 4 US pour
31 tâches, sans exigence orpheline. La réutilisation fraîche couvre 24/24
éléments et satisfait les cinq conditions du gate.

Preuve indépendante immuable :
/Users/moi/.cache/bridget-plan156-20261012-r1/base155-fresh/analyze-after-fix.md

SHA-256 : `e3176427d5fee611d625a9f36df587fae33412cd91d9836c6f1ec23b9a81e38a`.

Le gel analysé contenait T001 cochée et T002 ouverte. Le principal a ensuite
coché T002 après lecture de ses six axes. Ce changement de case ne déclenche
pas une nouvelle Analyze et ne modifie pas la preuve indépendante. Les
31 identifiants de tâches et les oracles restent conservés.

Le verdict porte sur les documents et leur cohérence. Il ne prouve aucune
implémentation156, aucun test156 et aucun GREEN global. Les lectures ciblées
de sources155 et du paquet149 gardent les limites décrites dans la preuve ;
aucune lecture intégrale fraîche du dépôt n’est revendiquée.
