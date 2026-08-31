# Tâches - SPEC-079 Continuité des travaux après redémarrage

## Règles

- Worktree exclusif: `/home/moi/bridget-referent/.worktrees/session-079-continuite-travaux-redemarrage`.
- Aucun commit, merge, push, déploiement ou redémarrage production automatique.
- Tests avant branche productive pour chaque frontière de crash.
- Aucune reprise depuis une ronde ou une synthèse.
- Aucune politique globale comme substitut à la politique projet.

## Phase 1 - Baseline et contrats

- [x] T001 Consigner commit, statut et tests de base ciblés dans `evidence/baseline.md`.
- [x] T002 Ajouter les tests protocole du contexte optionnel de `DeliverIdempotent`.
- [x] T003 Ajouter les tests du lien atomique remise-exécution dans IdempotencyStore.
- [x] T004 Ajouter les tests de reconstruction transactionnelle dans ExecutionStore.
- [x] T005 Ajouter les tests du contrat project-round-policy-v1.

## Phase 2 - Admission et binding du travail humain

- [x] T006 Poser `origin=human` et `intent=trigger_turn` dans le relais UI.
- [x] T007 Admettre l'enveloppe exacte et son exécution dans le chemin SendIdempotent.
- [x] T008 Écrire remise et DeliveryExecutionLink dans une transaction.
- [x] T009 Projeter le contexte d'exécution dans DeliverIdempotent.
- [x] T010 Créer le binding wrapper avant injection et conserver la suppression des doublons.
- [x] T011 Prouver transitions provider et UI `turn_state` sur une remise idempotente liée.

## Phase 3 - Reprise après redémarrage

- [x] T012 Implémenter la sélection bornée d'une exécution active par agent.
- [x] T013 Implémenter parent terminal, enfant reconstructed et projet conservé en une transaction.
- [x] T014 Créer la remise interne idempotente et son lien causal.
- [x] T015 Raccorder la reprise au Register après les remises déjà dispatching.
- [x] T016 Refuser la reconstruction si `turn_in_progress=true`.
- [x] T017 Rendre visibles enveloppe absente, corruption et actifs concurrents sans prompt inventé.
- [x] T018 Ajouter le test bout-en-bout arrêt après ack puis reprise unique.

## Phase 4 - Politique de ronde par projet

- [x] T019 Ajouter tables, migration et projections ProjectRoundPolicy.
- [x] T020 Implémenter commandes list/status/enable/disable idempotentes par command_id.
- [x] T021 Valider ProjectBinding actif et binding_generation avant mutation et sélection.
- [x] T022 Exposer le contrat client local et la commande CLI sans accès provider direct.
- [x] T023 Prouver absence=disabled, rebind=stale et désactivation sans effet sur exécutions.

## Phase 5 - Scheduler unique

- [x] T024 Ajouter le dispatcher source-controlled et sa grammaire fermée.
- [x] T025 Calculer une occurrence stable et une clé par projet/génération/occurrence.
- [x] T026 Émettre origin=routine, intent=trigger_turn et ProjectReference exacts.
- [x] T027 Tester double tick, zéro projet, projet désactivé et génération divergente.
- [x] T028 Documenter l'installation sans modifier le timer productif pendant la skill.

## Phase 6 - Convergence

- [x] T029 Exécuter fmt, clippy, tests ciblés et workspace; consigner dans `evidence/final-validation.md`.
- [x] T030 Rejouer Analyze, auditer les frontières de crash et écrire `analysis-report.md` et `audit.md`.
- [x] T031 Mettre à jour `implementation.md`, les compteurs et toutes les cases selon les preuves réelles.

## Dépendances

```text
T001 -> T002-T005 -> T006-T011 -> T012-T018
                    T019-T023 -> T024-T028
T018 + T028 -> T029 -> T030 -> T031
```
