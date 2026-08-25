# Tâches — Activation gouvernée des outils de pilotage

## Phase 1 — Spécification et décision

- [x] T001 Documenter le contrat, la recherche, le plan et l'ADR dans `specs/018-activation-outils-pilotage/` et `docs/decisions/012-activation-outils-pilotage-par-release.md`

## Phase 2 — Oracles de la frontière

- [ ] T002 [US2] Créer les contrôles positifs des cinq refus et de l'absence d'effet dans `scripts/test-pilotage-install.sh`

## Phase 3 — Release admise et durable

- [ ] T003 [US1] Implémenter la politique commune d'admission et de matérialisation dans `scripts/lib/pilotage-release.sh`
- [ ] T004 [US1] Migrer l'installation de `bridget-idle` vers la release gouvernée dans `scripts/install-bridget-idle.sh`

## Phase 4 — Ronde honnête

- [ ] T005 [US3] Migrer `bridget-ronde` et arrêter avant toute unité en cas de refus dans `scripts/install-bridget-ronde.sh` et `scripts/test-bridget-ronde.sh`

## Phase 5 — Validation et livraison

- [ ] T006 [US1] Valider idempotence, corruption, origine lisible et survie sans dépôt via `scripts/test-pilotage-install.sh`
- [ ] T007 Exécuter harnais métier, mutants, formatage, compilation, suite et Clippy puis consigner les comptes dans `specs/018-activation-outils-pilotage/implementation.md`
- [ ] T008 Finaliser le REX, geler le SHA et livrer la branche `session-18-activation-outils-pilotage` via Bridget

## Dépendances

```text
T001 → T002 → T003 → T004 → T005 → T006 → T007 → T008
```

La politique et les deux installateurs touchent la même frontière ; aucune
tâche d'implémentation n'est marquée parallèle afin d'éviter deux versions du
contrat en circulation.

## Critères indépendants par story

- **US1** : installation depuis un `main` admis, origine lisible, commande exécutable après suppression du dépôt de test.
- **US2** : cinq refus discriminants sans modification de la cible active.
- **US3** : copie régulière de ronde sans `--force` = échec non nul, sentinelle intacte, aucune unité créée.

## MVP

Le MVP exige US1 + US2 + US3 ensemble : omettre la ronde ou un refus
recréerait une autre voie de canary clandestin.
