# Tâches : cycle de vie des demandes Bridget

## Phase 1 — Fondations persistantes

- [x] T001 [P1] Ajouter le modèle `TrackedRequest` et les opérations SQLite de création, lecture, transition atomique, listing et purge dans `crates/bridget-daemon/src/store.rs`; couvrir les transitions terminales par tests unitaires.
- [x] T002 [P1] Ajouter `in_reply_to` au message Bridget et les commandes protocolaires de création, annulation et consultation dans `crates/bridget-core/src/message.rs` et `crates/bridget-transport/src/protocol.rs`; couvrir la sérialisation.

## Phase 2 — Cycle de vie daemon

- [x] T003 [US1] Faire créer une demande suivie pour chaque envoi `--reply`, vérifier l'émetteur à l'annulation, arrêter immédiatement les relances et notifier le destinataire dans `crates/bridget-daemon/src/daemon.rs`.
- [x] T004 [US2] Recharger les demandes ouvertes au démarrage, calculer les rappels depuis les dates persistées et préserver les états terminaux dans `crates/bridget-daemon/src/daemon.rs`.
- [x] T005 [US2] Corréler une réponse à `in_reply_to`, faire la transition `open → answered` et empêcher toute transition après un état terminal dans `crates/bridget-daemon/src/daemon.rs`.

## Phase 3 — CLI et enveloppes

- [x] T006 [US1] Ajouter `bridget cancel <id> [--reason]` et `bridget requests [--json]` dans `crates/bridget-daemon/src/cli.rs`, avec erreurs actionnables et affichage aligné.
- [x] T007 [US3] Propager l'identifiant de demande dans l'enveloppe et l'état de dernière demande reçue afin que `bridget reply` le réutilise automatiquement dans `crates/bridget-core/src/envelope.rs`, `crates/bridget-daemon/src/wrapper.rs` et `crates/bridget-daemon/src/cli.rs`.

## Phase 4 — Validation

- [x] T008 [P1] Ajouter des tests d'intégration socket Unix pour annulation autorisée, annulation interdite, idempotence, absence de rappel après annulation, réponse corrélée et reprise après redémarrage dans `crates/bridget-daemon/tests/integration_test.rs`.
- [x] T009 [P2] Documenter les commandes, le périmètre coopératif et le scénario de validation dans `README.md` et `specs/003-cycle-vie-demandes/quickstart.md`.
- [x] T010 [P1] Exécuter `cargo test -p bridget-daemon`, `cargo test --workspace`, `cargo build --release -p bridget-daemon` et `git diff --check`; consigner les résultats dans `specs/003-cycle-vie-demandes/implementation.md`.

## Phase 5 — Complément d'annuaire (session réouverte)

- [x] T011 [P1] Publier l'OS détecté par le wrapper dans `Register`, le conserver dans la présence et l'exposer dans `AgentInfo`, `bridget who`, `bridget agents` et la documentation ; couvrir la sérialisation et la présence durable.
