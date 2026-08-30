# Preuve US3 - rollback Docker vers hôte

Date : 2026-08-30

## Oracles exécutés

- `cargo test -p bridget-daemon --test project_runtime_integration_test` : 2 réussites.
- `cargo test -p bridget-daemon spec_066_switch_vers_host --lib` : 1 réussite.
- `cargo test -p bridget-daemon spec_066_lifecycle_refuse --lib` : 1 réussite.

## Faits prouvés

- Le test Docker réel distingue stop, remove puis recréation et vérifie que le dépôt et le state root restent inchangés.
- La bascule Docker vers host efface exclusivement le runtime, augmente la génération de liaison et rend toute réservation Docker obsolète.
- Une action destructive est refusée tant qu'un agent du projet est actif.
- Aucun conteneur de production ni aucune donnée projet réelle ne participe à ces tests.
