# Tâches — Session 063

## Phase 1 — Cadrage

- [x] T001 Créer `specs/063-interruption-pilotage-tour-humain/spec.md` et borner le chantier à l’interruption/pilotage humain.
- [x] T002 Cartographier sans écriture les transports dans `crates/bridget-transport/src/` et le chemin de remise daemon.
- [x] T003 Écrire `specs/063-interruption-pilotage-tour-humain/plan.md` avec des couloirs exclusifs.

## Phase 2 — Fondations

- [x] T004 Vérifier dans `crates/bridget-transport/src/claude_stream_json.rs` que la trame existante reste réutilisée, sans la réimplémenter.
- [x] T005 Définir dans `crates/bridget-transport/src/codex_app_server.rs` le passage de l’acceptation de steering à la confirmation de remise.

## Phase 3 — Interrompre proprement [US1]

- [x] T006 [P] [US1] Modifier `crates/bridget-transport/src/acp.rs` pour interrompre seulement un tour actif lors de l’arrivée d’un message humain, en conservant la FIFO et le règlement des permissions.
- [x] T007 [P] [US1] Modifier `crates/bridget-transport/src/claude_stream_json.rs` pour déclencher la trame existante seulement après une remise humaine admissible, sans perdre ce message.

## Phase 4 — Piloter et acquitter Codex [US2]

- [x] T008 [US2] Modifier `crates/bridget-transport/src/codex_app_server.rs` afin que le steering humain conserve l’ordre après rejet/interruption et n’acquitte qu’après consommation attestée.

## Phase 5 — Intégration [US3]

- [x] T009 [US3] Vérifier `crates/bridget-daemon/src/wrapper.rs` : le pont existant relie déjà `PromptDispatched` à `DeliverAcked` et classe un reçu `Seen` non attesté en `DeliveryIndeterminate` sans seconde injection.
- [x] T010 Mettre à jour `specs/063-interruption-pilotage-tour-humain/implementation.md` avec les preuves, limites et SHAs livrés.
- [ ] T011 Compiler/formater les couloirs touchés, intégrer les lots et produire l’horodatage minimal de route réelle après mise en service.

## Dépendances

`T006` et `T007` sont parallèles. `T008` est isolée dans son fichier. `T009`
et `T011` attendent les trois lots. Aucune tâche ne relance les campagnes
archivées.
