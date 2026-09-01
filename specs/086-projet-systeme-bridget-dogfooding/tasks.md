# Tâches - SPEC-086 Projet système Bridget et dogfooding expert

## Dépendances

`Rôle système -> Attestation -> Politique de mounts -> Transition expert -> Leases worktree -> UI -> Preuves`

Cette session commence seulement après mise à disposition des contrats SPEC-084 et du runtime SPEC-085.

## Phase 1 - Préparation et identité système

- [x] T001 Créer `tests/features/086-projet-systeme-bridget-dogfooding.feature` avec unicité, disabled, enabled, main read-only, worktree, conflit et livraison exclue.
- [x] T002 Créer `specs/086-projet-systeme-bridget-dogfooding/implementation.md` et y consigner transitions et preuves sans installer ni redémarrer Bridget.
- [x] T003 [P] Écrire les tests de `ProjectRole`, unicité et promotion standard interdite dans `crates/bridget-daemon/src/store.rs` et `crates/bridget-transport/src/protocol.rs`.
- [x] T004 Implémenter `ProjectRole::{Standard, BridgetSystem}` et la contrainte d'unicité dans le registre et ses migrations.
- [x] T005 [P] Écrire les tests d'attestation sans exécution pour checkout absent, faux checkout, worktree valide et git common dir incohérent dans `crates/bridget-daemon/src/daemon.rs`.
- [x] T006 Implémenter l'attestation structurelle versionnée du checkout Bridget sans exécuter de fichier du dépôt.
- [x] T007 Ajouter le contrat admin local de déclaration/réconciliation dans `crates/bridget-transport/src/protocol.rs` et refuser son usage par les routes standard.
- [x] T008 Réconcilier un emplacement `exact_project + system_only` SPEC-084 avec la génération attendue, sans le rendre créable.
- [x] T009 Ajouter les tests négatifs prouvant qu'un projet standard ne peut ni devenir système ni obtenir son emplacement ou ses mounts.

## Phase 2 - Politique de montage système

- [x] T010 [P] Écrire les tests de résolution des mounts pour standard, système disabled et système enabled dans `crates/bridget-daemon/src/project_runtime.rs`.
- [x] T011 Étendre le résolveur avec rôle, mode et worktree attribué; interdire centralement tout mount Bridget aux projets standard.
- [x] T012 Garder le checkout principal read-only dans les deux modes et rendre seulement le worktree non-main attribué writable en mode enabled.
- [x] T013 Borner l'écriture du git common dir nécessaire aux commits et exposer explicitement le domaine de confiance partagé dans la projection.
- [x] T014 Calculer le digest de topologie/mutabilité et imposer un nouvel epoch/recreate en cas de divergence.

## Phase 3 - Réglage expert et transition

- [x] T015 [P] Écrire les tests de `dogfooding.bridget` pour défaut disabled, génération, agent actif, backend Host, rejeu et rollback dans `control_settings.rs`.
- [x] T016 Ajouter le descripteur expert fermé, son avertissement et sa projection dans `crates/bridget-daemon/src/control_settings.rs`.
- [x] T017 Implémenter preview/apply lié aux générations du réglage et du projet système, sans mutation optimiste.
- [x] T018 Orchestrer stop contrôlé, recreate, attestation des mounts et publication tardive du mode dans `crates/bridget-daemon/src/daemon.rs`.
- [x] T019 Injecter les pannes de recreate et prouver la compensation vers le mode/mounts précédents sans fallback Host.

## Phase 4 - Attribution des worktrees

- [x] T020 [P] Écrire les tests de lease pour worktree principal, branche `main`, même worktree concurrent, worktrees distincts et libération dans `crates/bridget-daemon/src/store.rs`.
- [x] T021 Implémenter `WorktreeLease` canonique et sa persistance idempotente dans `crates/bridget-daemon/src/store.rs`.
- [x] T022 Raccorder la lease au lancement/arrêt des agents du projet système dans `crates/bridget-daemon/src/daemon.rs` et refuser main/checkout principal.
- [x] T023 Réconcilier les worktrees créés par des agents externes et demander recreate avant de les attribuer dans le conteneur.

## Phase 5 - Interface expert

- [x] T024 [P] Écrire les tests relay et Node des états indisponible/disabled/transition/enabled/degraded, avertissement et confirmations.
- [x] T025 Exposer la projection et les routes expert bornées dans `crates/bridget-daemon/src/ui.rs`, sans surface projet standard équivalente.
- [x] T026 Ajouter la carte Projet système et le toggle expert dans `crates/bridget-daemon/assets/ui/app.js`, `index.html` et `theme.css`.
- [x] T027 Afficher explicitement checkout principal read-only, worktree requis, confiance partagée et exclusion merge/push/install/restart/deploy.

## Phase 6 - Validation et preuves

- [x] T028 Exécuter les tests Docker prouvant écriture refusée en disabled, checkout principal refusé en enabled et worktree attribué writable.
- [x] T029 Lancer deux agents dans deux worktrees distincts, puis prouver le refus d'une double attribution du même worktree.
- [x] T030 Faire coexister un agent Bridget et un agent hôte externe sur deux worktrees, vérifier branches/index séparés et absence de copie.
- [x] T031 Exécuter `cargo fmt --check`, tests transport/daemon/UI, tests Node, tests Docker ciblés et `git diff --check`.
- [ ] T032 Effectuer le parcours `quickstart.md` et vérifier qu'aucune action automatique de merge, push, install, restart ou deploy n'existe.
- [ ] T033 Relire le diff contre `reuse-audit.md` et `adversarial-review.md`, documenter la confiance coopérative et obtenir la validation opérateur.

## Parallélisme sûr

- T003 et T005 peuvent avancer en parallèle.
- T010 peut être préparée dès que le rôle T004 est stabilisé.
- T015 et T020 peuvent être écrites en parallèle après validation du modèle.
- T024 peut utiliser des fixtures de projection avant les routes T025.

## Premier incrément livrable

T001 à T019 rendent le rôle et le mode applicables. La fonctionnalité reste incomplète tant que les leases, l'UI et les preuves Docker/multi-worktree ne sont pas terminées.
