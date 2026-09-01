# Tâches - SPEC-084 Projets multi-serveurs et catalogue d'emplacements

## Dépendances

`Contrat v2 -> Workspace daemon -> Administration -> Desktop multi-source -> Migration -> Validation`

La SPEC-085 peut commencer après stabilisation des contrats de création/import. La SPEC-086 attend la classification `exact_project + system_only`.

## Phase 1 - Préparation et contrats

- [x] T001 Créer `tests/features/084-projets-multiserveurs-emplacements.feature` avec choix de source, workspace, exact project, migration v1, déconnexion et conflit de génération.
- [x] T002 Créer `specs/084-projets-multiserveurs-emplacements/implementation.md` et y consigner les commandes, preuves et décisions effectives sans statut prématuré.
- [x] T003 [P] Écrire dans `crates/bridget-daemon/src/project_policy.rs` les tests de sérialisation v2, champs inconnus, identifiants dupliqués, défaut multiple et imbrications contradictoires.
- [x] T004 Implémenter `ProjectLocationCatalogV2`, `ProjectLocation` et `ProjectLocationKind` dans `crates/bridget-daemon/src/project_policy.rs` en conservant le décodage v1.
- [x] T005 Séparer et tester `validate_creation_parent` et `validate_import_root` dans `crates/bridget-daemon/src/project_policy.rs`; prouver qu'un exact project et un system only refusent la création.
- [x] T006 Ajouter les tests de migration v1 restrictive et de conservation des liaisons existantes dans `crates/bridget-daemon/src/project_policy.rs` et `crates/bridget-daemon/src/store.rs`.

## Phase 2 - Création et import côté daemon

- [x] T007 [P] Ajouter les charges v2 et refus fermés du catalogue/placement dans `crates/bridget-transport/src/protocol.rs` avec tests de compatibilité.
- [x] T008 Écrire les tests de prévisualisation create/import par `location_id`, collision, symlink, traversal et génération dans `crates/bridget-daemon/src/project_workspace.rs`.
- [x] T009 Adapter `crates/bridget-daemon/src/project_workspace.rs` pour calculer le chemin final côté daemon et revalider la capacité au moment de l'effet.
- [x] T010 Ajouter les routes GET catalog et POST preview/apply v2 dans `crates/bridget-daemon/src/ui.rs`, avec limites de corps, version et token existants.
- [x] T011 Conserver et tester les routes v1 en compatibilité bornée dans `crates/bridget-daemon/src/ui.rs`; toute création non liée à un workspace v2 doit être refusée.

## Phase 3 - Parcours Desktop multi-serveurs

- [x] T012 [P] Écrire les tests JavaScript de source toujours visible, changement de source, purge de preview et source indisponible dans `apps/bridget-desktop/ui/fleet-app.js`.
- [x] T013 Étendre l'état du dialogue create/import dans `apps/bridget-desktop/ui/fleet-app.js` avec `source_id`, capacité, emplacement et génération, sans jeton de tunnel.
- [x] T014 Adapter `apps/bridget-desktop/src-tauri/src/lib.rs` pour ouvrir le relais de la source confirmée et interdire toute mutation croisée depuis un panneau serveur.
- [x] T015 Étendre le sélecteur de source SPEC-081 et ajouter le sélecteur d'emplacement accessible dans `apps/bridget-desktop/ui/index.html`, `fleet-app.js` et `fleet-desktop.css`.
- [x] T016 Remplacer les clés de projet agrégé par `(source_id, project_id)` dans `apps/bridget-desktop/ui/fleet-app.js` et couvrir deux identifiants locaux identiques.
- [x] T017 Rendre les capacités absentes, sources déconnectées et raisons de refus sans état optimiste dans `apps/bridget-desktop/ui/fleet-app.js`.

## Phase 4 - Administration du catalogue

- [x] T018 [P] Écrire les tests de preview/apply du catalogue v2, génération concurrente, rejeu et rollback dans `crates/bridget-daemon/src/control_settings.rs`.
- [x] T019 Remplacer le descripteur plat des racines par un descripteur structuré v2 dans `crates/bridget-daemon/src/control_settings.rs` sans créer un second stockage.
- [x] T020 Implémenter la prévisualisation et l'application atomique du catalogue dans `crates/bridget-daemon/src/control_settings.rs` et `crates/bridget-daemon/src/project_policy.rs`.
- [x] T021 Ajouter à la prévisualisation l'inventaire des projets déjà liés sous chaque racine v1 dans `crates/bridget-daemon/src/store.rs`, sans mutation de liaison.
- [x] T022 Ajouter l'éditeur d'emplacements, les types et le delta de migration dans `crates/bridget-daemon/assets/ui/app.js`, `index.html` et `theme.css`.
- [x] T023 Ajouter les reçus et erreurs localisées du catalogue dans `crates/bridget-daemon/src/ui.rs` et les tests Node correspondants.

## Phase 5 - Validation transversale

- [x] T024 [P] Compléter les tests Rust de canonicalisation, liens symboliques, répertoire manquant, collision et enfant direct dans `crates/bridget-daemon/src/project_policy.rs` et `project_workspace.rs`.
- [x] T025 [P] Tester les courses source/catalogue entre preview et apply dans `crates/bridget-daemon/src/ui.rs` et les stores concernés.
- [x] T026 Étendre les tests Tauri de routage local/distant dans `apps/bridget-desktop/src-tauri/tests/desktop_commands.rs`.
- [x] T027 Exécuter et compléter les tests Node du dialogue et du centre de contrôle dans `apps/bridget-desktop/ui/fleet-app.js` et `crates/bridget-daemon/assets/ui/app.js`.
- [x] T028 Ajouter un test d'intégration avec deux sources possédant le même `project_id` local et prouver le routage vers une seule source.
- [x] T029 Exécuter les scénarios `tests/features/084-projets-multiserveurs-emplacements.feature` et consigner chaque preuve dans `implementation.md`.
- [x] T030 Exécuter une migration sans effet sur une copie de `/home/moi/.config/bridget/project-root-policy.json` et vérifier l'inventaire des liaisons historiques.
- [x] T031 Mettre à jour la documentation opérateur et les exemples de politique sans coder de chemin de workspace par défaut.
- [x] T032 Exécuter `cargo fmt --check`, les tests ciblés daemon/transport/Desktop, les tests Node et `git diff --check`.
- [x] T033 Relire le diff final contre `reuse-audit.md` et `adversarial-review.md`; retirer toute base globale, chemin libre ou duplication découverte.
- [ ] T034 Effectuer le parcours de `quickstart.md` sur deux sources, obtenir la validation opérateur et consigner le résultat sans implémenter SPEC-085/086.

## Parallélisme sûr

- T003 et T012 peuvent être écrites en parallèle car elles touchent daemon et Desktop.
- T007 attend les types de T004; T008 peut ensuite avancer avec T012.
- T018 peut commencer après le schéma T004, mais T019-T023 attendent les routes v2.
- Les validations T024-T028 peuvent être réparties par surface après convergence des contrats.

## Premier incrément livrable

T001 à T011 livrent le catalogue v2 et une API locale sûre. Le produit n'est complet qu'après le choix explicite de source, l'administration et la migration vérifiée.
