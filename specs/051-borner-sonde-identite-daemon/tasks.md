# Tâches — Session 051

- [x] T001 [US1] Ajouter le banc du vrai binaire et mesurer le pair silencieux dans `crates/bridget-daemon/tests/identity_probe_timeout_test.rs`.
- [ ] T002 [US1] Borner et typer la sonde d’identité dans `crates/bridget-daemon/src/daemon.rs`.
- [ ] T003 [US2] [US3] Propager l’indisponibilité dans `crates/bridget-daemon/src/cli.rs`, `crates/bridget-daemon/src/reprise.rs` et `crates/bridget-daemon/src/reaper.rs` sans casser les états voisins.
- [ ] T004 Rejouer les contrôles, le mutant causal, les validations et consigner les preuves dans `specs/051-borner-sonde-identite-daemon/implementation.md`.

## Dépendances

- T001 précède T002 : le défaut doit être observé avant le correctif.
- T002 précède T003 : les consommateurs dépendent du résultat typé.
- T004 dépend de T001 à T003.

## Critères indépendants

- **US1** : un pair accepte et capture la requête ; le client termine avec une
  erreur explicite dans la borne, sans seconde connexion.
- **US2** : socket absente et daemon ancien répondant restent deux contrôles
  valides et distincts.
- **US3** : aucun consommateur de statut ne convertit l’erreur en collection
  vide ou en succès.
