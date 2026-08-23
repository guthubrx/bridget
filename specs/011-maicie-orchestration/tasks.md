# Tasks : Maicie v3, coordinatrice d'orchestration légère

**Input** : `spec.md`, `plan.md`, `research.md`, `data-model.md`,
`contracts/maicie-cli-et-frontiere-bridget.md`, `reuse-audit.md`  
**Statut** : plan d'exécution uniquement. Aucune tâche n'est commencée ici.

**Règle** : tous les fichiers Maicie sont sous `plugins/maicie/`. Chaque lot
passe `cargo test --workspace` et `cargo clippy --all-targets -- -D warnings`.
Maicie n'importe aucun module interne Bridget, ne lit pas `bridget.db`, ne crée
aucun processus OS et ne possède aucun timer actif.

## Phase 1 : Fondation et contrat Bridget

- [x] T001 Ajouter le membre `plugins/maicie/` au workspace avec `plugins/maicie/Cargo.toml` et `plugins/maicie/src/main.rs` ; le binaire compagnon s'enregistre sous `maicie`, démarre et s'arrête sans charger de code dans le démon Bridget.
- [x] T002 Définir les types, invariants et transitions d'`ObjectifCoordonné`, `Délégation`, `OutboxDélégation`, `SnapshotTransport`, `ProfilÉquipe`, `ApprobationActivation` et `ActivationOutbox` dans `plugins/maicie/src/domain.rs` ; tests unitaires de transitions interdites dans `plugins/maicie/tests/contract/domain.rs`.
- [x] T003 [P] Définir la configuration validée dans `plugins/maicie/src/config.rs` : socket Bridget, SQLite privée, classes de durée, profils par tags et références SpawnOrder ; tests `plugins/maicie/tests/contract/config.rs` sans secret ni politique sémantique implicite.
- [x] T004 Définir `BridgetClient` dans `plugins/maicie/src/bridget_client.rs` : négociation version/capacités, annuaire, envoi suivi idempotent par `message_id` client, lecture d'issue par id, annulation, Subscribe session 008 et SpawnOrder session 009 ; aucune I/O directe hors cet adaptateur.
- [x] T005 Ajouter les fixtures producteur↔client dans `plugins/maicie/tests/contract/bridget_client.rs` : compatibilité de version, capability absente, `message_id` dédupliqué, corps divergent refusé sous même id, `Gap`/`End` et refus de toute tentative de chemin `bridget.db`.

**Checkpoint** : contrat public vérifiable avant toute persistance ou logique de
coordination.

**Gate non contournable avant T006** : vérifier par contrat que Bridget accepte
un `message_id` client, permet lookup par id et conserve son tombstone au moins
jusqu'à `retry_until`. Si une capacité manque, arrêter le lot Maicie et ouvrir
une session Bridget distincte ; ne jamais remplacer ce gate par un retry neuf.

## Phase 2 : Durabilité et reprise après crash

- [x] T006 Implémenter migrations idempotentes et l'outbox transactionnelle dans `plugins/maicie/src/store.rs` et `plugins/maicie/src/outbox.rs` : prepared contient message_id, target, body bytes, reply, timeout/deadline, hash et horizon ; index sur objectif ouvert/message_id ; tests `plugins/maicie/tests/integration/store_outbox.rs`.
- [x] T007 Implémenter décisions, approbations et `ActivationOutbox(command_id, spawn_order_bytes)` atomiques dans `plugins/maicie/src/store.rs` et `plugins/maicie/src/outbox.rs` ; test `plugins/maicie/tests/integration/approval_atomicity.rs` couvrant expiration, consommation après issue, mismatch hash et TOCTOU.
- [x] T008 Implémenter la réconciliation dans `plugins/maicie/src/reconcile.rs` : toute ligne non terminale, prepared comprise, fait lookup puis replay des bytes exacts ; tests à barrières dans `plugins/maicie/tests/integration/ack_lost_recovery.rs` avant socket, après write/avant Ack et après Ack/avant commit, sans double message ni demande.
- [x] T009 Ajouter la journalisation corrélée dans `plugins/maicie/src/telemetry.rs` ; test `plugins/maicie/tests/contract/telemetry.rs` imposant objective_id/message_id/source/fraîcheur et excluant les corps de messages par défaut.

**Checkpoint** : le crash entre acceptation et retour Bridget ne peut pas créer
de délégation doublon.

## Phase 3 : User Story 1 — Délégation déterministe et conversation libre (P1) 🎯 MVP

**Goal** : créer volontairement un objectif, déléguer avec une règle explicable
et garder tout message direct hors de l'état Maicie.

- [x] T010 [US1] Implémenter le cas d'usage dans `plugins/maicie/src/app.rs` : cible explicite, ou unique agent dont les tags sont égaux ; sinon liste de candidats sans choix ; contrat `plugins/maicie/tests/contract/delegate.rs` avec DND, absence et ambiguïté.
- [x] T011 [US1] Exposer `maicie delegate` et son JSON dans `plugins/maicie/src/main.rs`, conforme à `specs/011-maicie-orchestration/contracts/maicie-cli-et-frontiere-bridget.md` ; test `plugins/maicie/tests/integration/cli_delegate.rs` avec message_id présent avant I/O. **[Amendement R7 : option --idempotency-key K — même K = même objectif/délégation rejoués, jamais de doublon ; cf. conditions-agent-conversationnel.md]**
- [x] T012 [US1] Exposer `status`, `add-participant`, `remove-participant`, `summarize` et `close` dans `plugins/maicie/src/main.rs` et `plugins/maicie/src/app.rs` ; `summarize` agrège les réponses corrélées sans modèle ; tests `plugins/maicie/tests/integration/cli_objective.rs`.
- [x] T013 [US1] Ajouter le garde-fou dans `plugins/maicie/src/app.rs` : aucun message direct Bridget non adressé à Maicie ne crée ou modifie un objectif ; test négatif `plugins/maicie/tests/integration/direct_message_isolation.rs`.
- [x] T014 [US1] Traiter tout message libre adressé à Maicie dans `plugins/maicie/src/app.rs` comme enregistrement/affichage immuable et réponse d'aide CLI structurée ; test `plugins/maicie/tests/integration/maicie_confirmation.rs` : aucune détection d'intention ni mutation, seul `maicie delegate`/confirmation CLI crée un objectif.
- [x] T015 [US1] Réconcilier réponse, annulation, timeout et échec Bridget dans `plugins/maicie/src/reconcile.rs` ; une issue passe à `à_évaluer`, jamais à `clos`, et une synthèse reste factuelle ; test `plugins/maicie/tests/integration/delegation_outcomes.rs`.
- [x] T016 [US1] Exécuter le gate MVP dans `plugins/maicie/tests/mvp_gate.rs` : daemon Bridget réel, équipier ACP géré, délégation idempotente, remise/ACK durable, statut local distinct du snapshot transport et clôture ; corrélation de réponse explicitement reportée à T015b/T017-T018.

**Checkpoint** : délégation réellement utile, sans scheduler, interpréteur LLM,
profil réveillé ni dépendance ACP live.

## Phase 4 : User Stories 2 et 3 — Abonnement ACP et délais passifs (P2)

- [x] T017 [US2] Implémenter la consommation Subscribe session 008 dans `plugins/maicie/src/runtime.rs` avec subscription_id, seq, reprise, Gap et End ; tests `plugins/maicie/tests/contract/runtime_subscription.rs` interdisant la lecture de journal/socket interne.
- [x] T018 [US2] Afficher `transport_snapshot`, runtime, fraîcheur, flux incomplet et permission auto-décidée dans `plugins/maicie/src/app.rs` et `plugins/maicie/src/main.rs` ; test `plugins/maicie/tests/integration/status_sources.rs` interdisant `bloqué` et toute « permission humaine en attente » fictive.
- [x] T019 [US3] Traduire les classes configurées vers le timeout Bridget et les afficher passivement dans `plugins/maicie/src/app.rs` ; test `plugins/maicie/tests/contract/duration_timeout.rs` prouvant trois valeurs, aucune relance/timer local et aucune transition sans événement Bridget ou consultation.
- [x] T020 [US2] Ajouter le benchmark reproductible de `maicie status` sur 100 objectifs dans `plugins/maicie/tests/integration/status_benchmark.rs` ; l'observable consigné respecte SC-008 p95 < 250 ms.

**Gate** : T017–T020 ne commencent qu'après disponibilité et compatibilité
testée de session 008. La session 007 seule ne suffit pas.

## Phase 5 : User Story 4 — Profils approuvés et SpawnOrder (P3)

- [x] T021 [US4] Charger et valider les profils dans `plugins/maicie/src/profiles.rs` ; fixtures sous `plugins/maicie/tests/fixtures/profiles/` et tests `plugins/maicie/tests/contract/profiles.rs` d'égalité de tags et référence SpawnOrder obligatoire.
- [x] T022 [US4] Créer la proposition, l'approbation locale et ActivationOutbox dans `plugins/maicie/src/app.rs` : objective_id/profile_id, hashes, scope, paramètres, local_human, expiration et command_id ; test `plugins/maicie/tests/integration/profile_approval.rs`.
- [x] T023 [US4] Réconcilier/rejouer SpawnOrder 009 depuis `plugins/maicie/src/profiles.rs` et `plugins/maicie/src/reconcile.rs` : command_id et bytes exacts, lookup préalable, consommation après issue ; test `plugins/maicie/tests/integration/spawn_order.rs` aux trois frontières crash et zéro `Child`, `Command::spawn`, contexte ou ordre après refus/expiration/TOCTOU.

**Gate** : T021–T023 ne commencent qu'après contrat public stabilisé de session
009 ; Maicie ne devient jamais un superviseur de processus.

## Phase 6 : Finition

- [x] T024 [P] Mettre à jour `plugins/maicie/README.md`, `README.md` et `docs/decisions/003-maicie-compagnon-orchestration.md` avec démarrage, limites, outbox, arrêt et frontière d'état ; vérifier `quickstart.md`.
- [x] T025 Exécuter les scénarios de `specs/011-maicie-orchestration/quickstart.md`, `cargo test --workspace` et `cargo clippy --all-targets -- -D warnings` ; consigner les preuves dans `specs/011-maicie-orchestration/implementation.md` seulement lors de l'implémentation réelle.
- [x] T026 Effectuer une revue hostile finale des fichiers `plugins/maicie/`, couvrant outbox/ack perdu, TOCTOU d'approbation, dépendances 008/009, fuite de contexte, permission ACP et minimalisme ; corriger avant clôture.

## Dépendances et stratégie

- T001–T005 → T006–T009 → T010–T016 : chemin MVP obligatoire.
- T017–T020 dépend de session 008 ; T021–T023 dépend de session 009.
- T024–T026 clôturent seulement les lots effectivement choisis.
- Arrêter après T016 pour valider le MVP ; GUI/TUI, MCP sémantique et toute
  intelligence de langage doivent faire l'objet de specs séparées.
