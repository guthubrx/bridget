# Tâches - SPEC-085 Runtime Docker de production

## Dépendances

`Image/catalogues -> Configuration serveur -> Activation atomique -> Montages Git -> UI -> Installation et preuves`

La SPEC-086 attend l'activation Docker et la politique de montages de cette session.

## Phase 1 - Préparation et image

- [x] T001 Créer `tests/features/085-runtime-docker-production.feature` avec défaut serveur, activation Host, rollback, agent actif, secrets, worktrees et redémarrage.
- [x] T002 Créer `specs/085-runtime-docker-production/implementation.md` et y consigner builds, digests, tests et preuves sans déployer implicitement.
- [x] T003 [P] Ajouter les contrôles statiques de Dockerfile, versions, digest de base et absence de secret dans `infra/project-runtime/production/tests/`.
- [x] T004 Créer `infra/project-runtime/production/Dockerfile` avec base Linux amd64 par digest, utilisateur non root et socle de développement neutre.
- [x] T005 Ajouter `infra/project-runtime/production/toolchain.lock`, inventaire de provenance et génération de SBOM ou inventaire équivalent sans credential.
- [x] T006 Créer le test de fumée `infra/project-runtime/production/smoke.sh` pour shell, Git, rg, jq, curl, Rust, Node/pnpm et Python.
- [x] T007 Créer les exemples validés de runtime policy et resource catalog dans `infra/project-runtime/production/config/`, avec clients fournisseurs versionnés par politique.

## Phase 2 - Configuration serveur

- [x] T008 [P] Écrire les tests de `execution.default_backend`, migration Host et capacité incomplète dans `crates/bridget-daemon/src/control_settings.rs`.
- [x] T009 Implémenter le réglage fermé et sa génération dans `crates/bridget-daemon/src/control_settings.rs`; ne modifier aucun binding existant.
- [x] T010 Raccorder les trois fichiers `project-root-policy`, `project-runtime-policy` et `project-resource-catalog` dans les unités d'installation sous `packaging/systemd/`.
- [x] T011 Ajouter la projection `RuntimeCapability` dans `crates/bridget-daemon/src/daemon.rs` et `ui.rs` avec raisons fermées.
- [x] T012 Ajouter un preflight d'accès Docker du compte de service qui refuse sans modifier groupe, socket ou permissions dans `crates/bridget-daemon/src/project_runtime.rs`.
- [x] T013 Refuser explicitement les architectures non Linux amd64 dans le loader de politique et ses tests.

## Phase 3 - Activation atomique Host vers Docker

- [x] T014 [P] Écrire les tests de sérialisation, refus et rejeu d'`ActivateDocker` dans `crates/bridget-transport/src/protocol.rs`.
- [x] T015 Étendre `ProjectRuntimeOperation` et les charges v2 avec `activate_docker`, génération attendue et référence de politique fermée.
- [x] T016 [P] Écrire les tests store de transition préparatoire, publication tardive, même commande et conflit de génération dans `crates/bridget-daemon/src/store.rs`.
- [x] T017 Implémenter `ProjectRuntimeTransition` et la publication atomique de binding dans `crates/bridget-daemon/src/store.rs`.
- [x] T018 Orchestrer résolution, préparation, attestation et commit dans `crates/bridget-daemon/src/daemon.rs` sans publier un binding Docker partiel.
- [x] T019 Ajouter des injections de panne et prouver la compensation vers Host après chaque phase dans les tests daemon/runtime.
- [x] T020 Réconcilier au démarrage transitions incomplètes, binding, conteneur réel, image et epoch dans `crates/bridget-daemon/src/daemon.rs`.

## Phase 4 - Montages, état et sécurité

- [x] T021 [P] Créer une fixture Git avec checkout principal et deux linked worktrees dans les tests de `crates/bridget-daemon/src/project_runtime.rs`.
- [x] T022 Étendre `resolve_project_mounts` pour inclure checkout, git common dir et worktrees aux chemins absolus identiques.
- [x] T023 Calculer un digest de topologie et imposer recreate lorsqu'un worktree ou une mutabilité change.
- [x] T024 Persister HOME, caches et états sous la racine hôte de projet par bind mounts et refuser les volumes Docker nommés comme source de vérité.
- [x] T025 Tester rootfs RO, UID/GID, cap-drop, no-new-privileges, CPU, mémoire, PIDs, tmpfs, réseau sans port entrant et absence de socket Docker.

## Phase 5 - Interface et opérations projet

- [x] T026 [P] Écrire les tests de routes runtime v2, token, corps, état et refus dans `crates/bridget-daemon/src/ui.rs`.
- [x] T027 Ajouter GET capability/status et POST actions typées dans `crates/bridget-daemon/src/ui.rs`, sans option Docker libre.
- [x] T028 [P] Écrire les tests Node des états Host/Docker, actions autorisées, confirmation et absence d'optimisme dans `crates/bridget-daemon/assets/ui/app.js`.
- [x] T029 Ajouter le choix backend/politique à la création/import et respecter la valeur présélectionnée du serveur dans l'UI projet.
- [x] T030 Ajouter status, activate, stop, remove, recreate et switch to Host au menu projet avec reçus et erreurs actionnables.

## Phase 6 - Intégration et preuves

- [x] T031 Construire l'image de production, enregistrer son digest et exécuter le smoke test sur le serveur cible.
- [x] T032 Inspecter image, historique, environnement, mounts, logs et projections avec secrets factices pour prouver l'absence de fuite.
- [x] T033 Exécuter un test Docker Host -> activate -> ready avec deux agents successifs partageant le même conteneur projet.
- [x] T034 Tester que stop, remove, recreate et switch to Host sont refusés avec un agent actif et restent idempotents après rejeu.
- [x] T035 Redémarrer le daemon dans la fixture d'intégration et prouver la réconciliation sans création d'un second conteneur.
- [x] T036 Revenir à Host et vérifier bit à bit que dépôt, branches et worktrees n'ont pas été supprimés ou déplacés.
- [x] T037 Exécuter `cargo fmt --check`, tests transport/daemon/Desktop, tests Node, tests Docker ciblés et `git diff --check`.
- [ ] T038 Effectuer le parcours `quickstart.md` avec validation opérateur avant tout changement du défaut serveur en Docker.
- [x] T039 Réaliser la revue sécurité et minimalisme finale contre `reuse-audit.md` et `adversarial-review.md`, puis documenter les limites Linux amd64 et domaine de confiance projet.

## Parallélisme sûr

- T003 et T008 peuvent avancer en parallèle.
- T014 et T016 sont parallèles après accord sur le contrat v2.
- T021 et T026 peuvent avancer pendant la saga store/daemon.
- T028 peut être écrite avec les fixtures de contrat avant T027.

## Premier incrément livrable

T001 à T020 fournissent une activation atomique testable par CLI. La fonctionnalité produit exige ensuite montages Git, UI, image attestée et installation prouvée.
