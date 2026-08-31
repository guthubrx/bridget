# Tâches: Environnement Docker partagé par projet

**Statut**: terminé, 36 tâches prouvées
**Gate**: nouveau worktree créé depuis 74234641fe04293c007717b8f1e3879823f5d382 ou un descendant propre
contenant SPEC-068 et SPEC-075 après SPEC-065, reuse-audit.md rejoué et
implementation.md mis à jour après chaque tâche prouvée. L'absence de
`.specify` ou de l'outil `specify` ne doit déclencher ni installation ni mise à
jour. Le présent worktree porte l implémentation isolée; le worktree documentaire source reste inchangé.

## Phase 1 - Préparation

- [x] T001 Créer `tests/features/066-environnement-partage-projet.feature` avec prepare, partage, isolation, panne et rollback.
- [x] T002 Observer Docker Engine, mode rootless déclaré ou non, UID/GID du compte, layout des worktrees, droits des racines/state roots et permissions sur l'hôte cible, puis consigner les faits dans `specs/066-environnement-partage-projet/evidence/preflight-host.md` sans modifier la production.
- [x] T003 Rejouer `specs/066-environnement-partage-projet/reuse-audit.md` contre la tête d'implémentation contenant SPEC-068 avant toute création de module ou dépendance.

## Phase 2 - Fondations bloquantes

- [x] T004 [P] Ajouter les tests de machine d'état ProjectEnvironment, environment_epoch, `runtime_policy_changed` et exclusion spawn/lifecycle dans `crates/bridget-daemon/src/project_runtime.rs` avant l'implémentation productive.
- [x] T005 [P] Ajouter les tests de construction d'arguments Docker sans shell, politique fermée absente/invalide, image `registry_digest`/`local_image_id`, UID/GID, HOME/XDG et socket explicite dans `crates/bridget-daemon/src/project_runtime.rs`.
- [x] T006 [P] Ajouter les tests de contrat backend host/docker, `policy_id`, `policy_version`, policy digest et compatibilité des anciennes liaisons dans `crates/bridget-transport/src/protocol.rs`.
- [x] T007 Créer `crates/bridget-daemon/src/project_runtime.rs` pour la machine d'état, la politique et les issues, puis l'exporter depuis `crates/bridget-daemon/src/lib.rs`.
- [x] T008 Étendre ProjectBinding et le store dans `crates/bridget-daemon/src/store.rs` avec backend, état environnement, policy_id/version, digests, UID/GID et migration additive.
- [x] T009 Étendre les contrats de `crates/bridget-transport/src/protocol.rs` sans rendre les champs Docker obligatoires pour host.

## Phase 3 - User Story 1: préparer et diagnostiquer (P1)

**Test indépendant**: prepare, attest, remove, recreate sans agent ni secret.

- [x] T010 [P] [US1] Ajouter un faux binaire Docker couvrant timeout, JSON hostile, image absente et daemon indisponible dans `crates/bridget-daemon/tests/fixtures/docker/`.
- [x] T011 [P] [US1] Ajouter les tests d'intégration Docker réels pour image immuable, labels, UID/GID, HOME/XDG, écriture state root, lecture fixture `0600`, socket explicite, mounts, réseau et limites dans `crates/bridget-daemon/tests/project_runtime_integration_test.rs`.
- [x] T012 [US1] Charger au démarrage le document hôte `project-runtime-policy-config-v1` depuis un chemin absolu configuré dans `crates/bridget-daemon/src/daemon.rs` et `crates/bridget-daemon/src/cli.rs`, puis implémenter le préflight version/daemon/image/UID/GID/droits dans `crates/bridget-daemon/src/project_runtime.rs` avec timeouts et raisons fermées.
- [x] T013 [US1] Implémenter create/start/inspect par arguments dans `crates/bridget-daemon/src/project_runtime.rs`, sans shell, tag mutable ni option libre, avec state root monté sous `/var/lib/bridget-project` et ABI HOME/XDG fixe.
- [x] T014 [US1] Refuser tout environnement divergent et persister `recreate_required` dans `crates/bridget-daemon/src/store.rs`.
- [x] T015 [US1] Ajouter prepare/status/recreate à la CLI locale dans `crates/bridget-daemon/src/cli.rs` et la projection dans `crates/bridget-daemon/src/ui.rs`.
- [x] T016 [US1] Prouver destruction/recréation et empreintes inchangées dans `specs/066-environnement-partage-projet/evidence/us1-environment.md`.

## Phase 4 - User Story 2: partager entre agents (P1)

**Test indépendant**: deux agents et deux worktrees, un container id, cycles distincts.

- [x] T017 [P] [US2] Ajouter les tests du runtime ingress privé par projet, reconnexion après restart, refus A vers B, refus identité/génération forgée et rejeu/accusé des incidents runtime délégués SPEC-068 avant effet dans `crates/bridget-daemon/tests/project_runtime_ingress_test.rs`.
- [x] T018 [P] [US2] Ajouter les tests de montage de worktrees du même common dir et de refus des layouts étrangers dans `crates/bridget-daemon/tests/project_runtime_mounts_test.rs`.
- [x] T019 [P] [US2] Ajouter les tests de deux agents dans un conteneur et deux projets séparés dans `crates/bridget-daemon/tests/project_runtime_agents_test.rs`.
- [x] T020 [US2] Ajouter un endpoint Unix distinct par project_id/binding_generation dans `crates/bridget-daemon/src/daemon.rs`, avec répertoire privé monté dans ce seul conteneur.
- [x] T021 [US2] Propager explicitement `/run/bridget/runtime/bridget.sock` sans dérivation depuis HOME et imposer le handshake project_id/binding_generation/container_id/environment_epoch/génération dans `crates/bridget-transport/src/protocol.rs`, `crates/bridget-daemon/src/lifecycle.rs` et `crates/bridget-daemon/src/wrapper.rs` avant inscription ou remise.
- [x] T022 [US2] Résoudre et attester les montages racine/worktrees dans `crates/bridget-daemon/src/project_runtime.rs`, sans scan d'autres projets.
- [x] T023 [US2] Lancer chaque agent via docker exec après résolution fermée du lanceur et de la commande interne à l image dans `crates/bridget-daemon/src/daemon.rs`, `crates/bridget-daemon/src/managed_process.rs` et `crates/bridget-daemon/src/lifecycle.rs`, avec cwd, identité et environment_epoch réservés puis revérifiés juste avant exec.
- [x] T024 [US2] Corréler exec, génération, provider et environnement dans `crates/bridget-daemon/src/fleet.rs` et `crates/bridget-daemon/src/desired_state.rs`.
- [x] T025 [US2] Réconcilier les agents et conteneurs sans suppression au démarrage dans `crates/bridget-daemon/src/managed_supervisor.rs`, y compris rebind qui conserve les agents actifs mais bloque toute nouvelle admission.
- [x] T026 [US2] Exécuter la parité steering/interruption/communication et incidents runtime délégués SPEC-068, y compris warning, failed, rejeu ordonné, accusé idempotent et ProjectReference, puis consigner les preuves dans `specs/066-environnement-partage-projet/evidence/us2-shared-agents.md`.

## Phase 5 - User Story 3: exploitation et rollback (P1)

**Test indépendant**: refus avec agents actifs, stop/remove sûr, switch host et absence de fallback.

- [x] T027 [P] [US3] Ajouter les tests stop/remove/switch avec agents actifs/inactifs, invalidation sur changement backend/policy/image/UID/GID et interleaving déterministe admission/exec/lifecycle dans `crates/bridget-daemon/tests/project_runtime_lifecycle_test.rs`.
- [x] T028 [P] [US3] Ajouter les tests OOM, PID limit, exec lost et Docker restart dans `crates/bridget-daemon/tests/project_runtime_failures_test.rs`.
- [x] T029 [US3] Implémenter stop/remove/recreate/switch-backend, l'émission `runtime_policy_changed` et la réservation transactionnelle de lifecycle dans `crates/bridget-daemon/src/project_runtime.rs`, `crates/bridget-daemon/src/store.rs` et `crates/bridget-daemon/src/cli.rs`, mutations locales uniquement.
- [x] T030 [US3] Interdire tout fallback host implicite dans `crates/bridget-daemon/src/lifecycle.rs` et rendre la cause Docker structurée.
- [x] T031 [US3] Ajouter la réconciliation de ressource et les raisons OOM/PID/exec lost dans `crates/bridget-daemon/src/managed_supervisor.rs` et `crates/bridget-daemon/src/ui.rs`.
- [x] T032 [US3] Prouver le rollback host et la non-destruction dans `specs/066-environnement-partage-projet/evidence/us3-rollback.md`.

## Phase 6 - Validation transversale

- [x] T033 Ajouter l'image fixture et sa construction reproductible par `docker build --iidfile` dans `infra/project-runtime/Dockerfile` et `infra/project-runtime/README.md`, sans credential ni tag utilisé comme autorité.
- [x] T034 Exécuter la revue hostile des montages, UID/GID, HOME/XDG, socket explicite, capabilities, réseau, ports, Docker socket et state roots, puis consigner dans `specs/066-environnement-partage-projet/evidence/security-review.md`.
- [x] T035 Exécuter `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` et `cargo test --workspace`, puis rejouer les non-régressions SPEC-063, 064, 065, 068 et 075, consigner dans `specs/066-environnement-partage-projet/evidence/final-validation.md` et achever `specs/066-environnement-partage-projet/implementation.md`.
- [x] T036 Rejouer Analyze, archiver la session selon le workflow SpecKit disponible et mettre à jour `specs/066-environnement-partage-projet/analysis-report.md`, sans activer un projet réel ni passer le statut à Implemented tant qu'une tâche reste ouverte.

## Dépendances

```text
T001-T003
   |
T004-T009
   |
US1 T010-T016
   |
US2 T017-T026
   |
US3 T027-T032
   |
T033-T036
```

- Les tests T004-T006 sont parallèles.
- US2 dépend d'un environnement attesté US1.
- US3 dépend du suivi d'agents US2.
- Le MVP technique est US1 avec fixture; aucun fournisseur réel n'est activé
  avant US2 et la future SPEC-067.

## Article XIX et XX

- Un seul nouveau module productif, justifié par une machine d'état et plus de
  trois consommateurs.
- Aucun SDK Docker, Compose, K8s ou service auxiliaire.
- Les abstractions provider ne changent pas; le backend agit sous lifecycle.
- Chaque lot expose une inspection, un oracle de panne et un rollback.
