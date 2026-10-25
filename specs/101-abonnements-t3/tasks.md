# Tâches101 — Implemented

Base des chemins : racine du worktree101. Gate reuse-audit PASS lu avant création.
Toutes les tâches sont nécessaires ; pas d'arrêt après une tranche MVP.

## Préparation

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/101-abonnements-t3/tasks.md#extrait-1).

## US1 — Identité sûre

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/101-abonnements-t3/tasks.md#extrait-2).

## US2 — Faits adaptateur de bureau

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/101-abonnements-t3/tasks.md#extrait-3).

## US3 — Capacité et interruptions

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/101-abonnements-t3/tasks.md#extrait-4).

## US4 — Parcours quotidiens et livraison vérifiable

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/101-abonnements-t3/tasks.md#extrait-5).

## Dépendances et parallélisme

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/101-abonnements-t3/tasks.md#extrait-6).

## Validation et limites de recette

Commandes prévues : cargo fmt --all -- --check ; cargo clippy --workspace
--all-targets -- -D warnings ; cargo test -p bridget-daemon --lib spec101 ;
cargo test -p bridget-daemon --test spec101_observation_test ; régressions
spec100, spec099, mcp::tests, attach::tests, communication::client::security_tests,
journal::tests et protocole. HOME/socket privés pour recettes ; variables du
daemon réel retirées des tests utilisant leurs propres fixtures.
Pas de cargo test --workspace aveugle : plusieurs anciens harnais SIGKILL sont
incompatibles avec la règle processus utilisateur. Déclarer cette exclusion.
T011 exige une preuve réelle et l'autorisation d'adoption101 ; sans celle-ci,
laisser cette tâche bloquée, pas de statut Implemented.

## Convergence

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/101-abonnements-t3/tasks.md#extrait-7).

- [x] T013 [US3] Rendre les lacunes de la file de faits consultables et notifiées via protocole primaire attesté et relais existant (wrapper.rs, protocol.rs, daemon.rs, observation.rs), sans fausse observation de fin. Réduire l'attente de verrou SQLite des seules sauvegardes d'abonnements (store.rs) en conservant refus/rollback et comportement des autres opérations. Tester saturation amont réelle, refus auxiliaire et contention réelle à deux connexions (FR005,FR007–008).
