# Rapport d'analyse - Session156 (spec, plan, tâches)

Date : 2026-10-11, révisé après les contre-revues rondes 1 et 2
(`adversarial-review-codex.md`). Analyse non destructive de
`specs/156-guidage-file/`. Le code du worktree 150 n'a pas été lu (consigne) ;
les acquis 150 sont supposés livrés.

## Verdict

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
- 152 doit lever `mission_active` (T020).
- 155 pourra ajouter un autre type de remise : `bridget_queue` ne suppose pas que toute entrée est un message d'agent.
- La future session d'édition de file reprend les prérequis de `research.md` R8.

## Questions utilisateur (défauts retenus)

1. Un pair du même projet peut-il guider ? Défaut : non.
2. Reporter `cancel`, `reorder`, `promote` de la file à une session dédiée ? Défaut : oui.
3. Accepter deux changements visibles : les pairs attendent la fin du tour (guidage implicite T3) ; la CLI nue ne peut plus guider ? Défaut : oui.
