# Tâches 134 — Noms humains dans les messages Bridget

## Phase 1 — Préparation

- [x] T001 Vérifier le worktree, la base `main`, le contrat et les tests ciblés existants ; consigner le résultat dans `specs/134-noms-humains-messages/implementation.md`

## Phase 2 — Libellé commun

Objectif : rendre le nom humain et l’UUID sans changer l’identité routable.

Test indépendant : les cas nommé, absent, vide, blanc, égal à l’UUID et délégué
produisent exactement le libellé du contrat.

- [x] T002 [US1] [US3] Ajouter les tests rouges `spec134` puis implémenter le libellé `nom (UUID)` avec repli UUID et provenance conservée dans `crates/bridget-core/src/message.rs`

## Phase 3 — Lots adaptateur de bureau

Objectif : appliquer le libellé commun à chaque message groupé.

Test indépendant : un lot de deux expéditeurs nommés associe chaque nom au bon
UUID complet.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/134-noms-humains-messages/tasks.md#extrait-1).

## Phase 4 — Réparation du profil

Objectif : garantir l’invariant identité, profil et état d’application après
chaque enregistrement.

Test indépendant : conserver une identité, retirer son profil, rappeler deux fois
`ensure_agent_ids()` puis vérifier une seule identité, un seul profil et un seul
état d’application.

- [x] T004 [P] [US4] Ajouter le test rouge `spec134` puis rendre `ensure_agent_ids()` idempotent sur les trois lignes dans `crates/bridget-daemon/src/agent_profile.rs`

## Phase 5 — Validation et livraison

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/134-noms-humains-messages/tasks.md#extrait-2).

## Correctif de saturation — reprise du 2026-10-05

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/134-noms-humains-messages/tasks.md#extrait-3).

## Dépendances initiales

1. T001 précède toute modification.
2. T002 précède T003, car le lot réutilise le libellé central.
3. T004 peut avancer indépendamment de T002 et T003.
4. T002, T003 et T004 doivent être terminées avant T005.
5. T005 précède T006. T006 précède T007.

## Exécution parallèle possible

- T004 touche uniquement le registre de profils et peut être traité en parallèle
  de T002.
- T003 attend T002 pour éviter deux définitions du format visible.

## Stratégie d’implémentation

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/134-noms-humains-messages/tasks.md#extrait-4).

## Validation Article XIX/XX

- T002 centralise une règle déjà existante et ajoute zéro abstraction.
- T003 supprime un accès direct divergent.
- T004 renforce le contrat d’un `ensure_*` existant dans sa transaction actuelle.
- T005 et T006 rendent les hypothèses et les limites observables pour le prochain
  mainteneur.
