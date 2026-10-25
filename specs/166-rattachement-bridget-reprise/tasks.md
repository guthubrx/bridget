# Tâches — SPEC-166 Rattachement Bridget après reprise adaptateur de bureau

**Statut :** Implemented
**Progression :** 14/14
**Pipeline :** Livré.

Chaque tâche reste ouverte jusqu’à sa preuve. Le test de rotation précède le correctif.

## Phase 1 — Préparation

- [x] T001 Figer le diagnostic expurgé et les identifiants de version dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/166-rattachement-bridget-reprise/specs/166-rattachement-bridget-reprise/validation/runtime-before.md`
- [x] T002 Vérifier l’absence de duplication et les règles applicables dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/166-rattachement-bridget-reprise/specs/166-rattachement-bridget-reprise/reuse-audit.md`

## Phase 2 — Fondations

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/tasks.md#extrait-1).

## Phase 3 — US1 Reprendre un fil sans réparer Bridget

**But :** le premier tour d’un fil ancien recharge sa preuve privée courante.  
**Test indépendant :** A est montée, B devient courante, puis la transcription observe la reprise avec B avant le tour.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/tasks.md#extrait-2).

## Phase 4 — US2 Préserver les refus de sécurité

**But :** la réparation ne prête jamais l’identité d’un autre fil.  
**Test indépendant :** B reçoit les droits du run ; A et la preuve d’un fil voisin ne les reçoivent pas.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/tasks.md#extrait-3).

## Phase 5 — US3 Conserver la continuité des conversations

**But :** la réparation garde le même fil natif et son historique.  
**Test indépendant :** la reprise utilise le même identifiant avec `excludeTurns: true`, puis le tour démarre une fois.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/tasks.md#extrait-4).

## Phase 6 — Qualité et livraison

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/166-rattachement-bridget-reprise/tasks.md#extrait-5).

## Dépendances

`T001 → T002 → T003 → T004 → T005 → T006 → T007 → T008 → T009 → T010 → T011 → T012 → T013 → T014`

## Possibilités de travail parallèle

Le changement de code et son rejeu utilisent les mêmes fichiers. Ils restent séquentiels pour éviter une course. La documentation T011 peut être relue pendant T012, mais sa validation finale dépend du diff complet.

## Stratégie d’implémentation

Le minimum livrable est US1 avec l’oracle RED puis GREEN. US2 et US3 verrouillent ensuite la sécurité et la continuité avant toute installation. T005 centralise un comportement déjà présent. Cette tâche réduit la maintenance future. Aucune tâche n’ajoute une couche ou une dépendance.
