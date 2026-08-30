# Preuve US2 - agents partagés et incidents délégués

Date : 2026-08-30

## Oracles exécutés

- `cargo test -p bridget-daemon --test project_runtime_agents_test` : 1 réussite.
- `cargo test -p bridget-daemon --test project_runtime_ingress_test` : 1 réussite.
- `cargo test -p bridget-daemon spec_068 --lib` : 7 réussites.

## Faits prouvés

- Deux agents d'un même projet partagent un conteneur tout en conservant leurs identités Bridget distinctes.
- Deux projets ne partagent ni conteneur ni ingress; une identité ou une génération forgée est refusée avant Register.
- Les incidents délégués warning et failed sont persistés, rejoués dans l'ordre puis acquittés de façon idempotente avec leur ProjectReference.
