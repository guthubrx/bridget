# Tâches — SPEC-165 Cycle de vie des liens adaptateur de bureau

**Statut :** Implemented
**Progression :** 18/18
**Pipeline :** Livré.

Chaque tâche reste ouverte jusqu'à sa preuve. Les tests qui définissent le comportement précèdent le code correspondant.

## Phase 1 — Préparation

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/tasks.md#extrait-1).

## Phase 2 — Fondations

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/tasks.md#extrait-2).

## Phase 3 — US1 Libérer les sous-agents terminés

**But :** le nombre de liens ne croît plus avec l'historique enfant.  
**Test indépendant :** 1 000 sous-agents terminés ne produisent pas 1 000 liens.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/tasks.md#extrait-3).

## Phase 4 — US2 Garder le travail actif joignable

**But :** un enfant actif reste exposé et un enfant repris retrouve son lien.  
**Test indépendant :** la même fixture passe de terminal à actif et redevient éligible au rafraîchissement suivant.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/tasks.md#extrait-4).

## Phase 5 — US3 Préserver les conversations principales

**But :** les racines, forks et anciens snapshots restent joignables.  
**Test indépendant :** une racine terminale et un fil V1 restent exposés tandis qu'un enfant terminal ne l'est plus.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/tasks.md#extrait-5).

## Phase 6 — Qualité et livraison

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/tasks.md#extrait-6).

## Dépendances

`T001 → T002 → (T003 || T004) → T005 → T006 → T007 → T008 → T009 → T010 → T011 → T012 → T013 → T014 → T015 → T016 → T017 → T018`

## Charge de maintenance

- T005 à T012 étendent trois fichiers existants. Aucun service, format de stockage ou dépendance n'est ajouté.
- T008 ferme une ressource déjà possédée. Elle réduit le risque de fuite future.
- T010 réutilise l'état durable existant. Le scan est unique et évite une lecture disque par cycle.
- T013 rend l'invariant visible pour éviter une nouvelle hausse de limite comme correction principale.
