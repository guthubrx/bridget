# Tâches - SPEC-081 Flotte globale et sources Desktop

**Branche** : `session-081-flotte-globale-sources-ui`  
**Feature** : `/home/moi/bridget-referent/.worktrees/session-081-flotte-globale-sources-ui/specs/081-flotte-globale-sources-ui`  
**Approche** : test-first, aucune livraison automatique.

## Dépendances

```text
Fondations -> US1 flotte globale -> US2 Sources -> US3 organisation -> US4 épingles -> validation complète
                         |-------------------------------> US3 présentation pure peut démarrer après fondations
```

## Phase 1 - Préparation et témoins

- [x] T001 Consigner la base et exécuter les témoins existants dans `specs/081-flotte-globale-sources-ui/evidence/baseline.md` avec `cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml` et `node --test crates/bridget-daemon/assets/ui/app.js`.
- [x] T002 Écrire les témoins de projection sans secret dans `apps/bridget-desktop/src-tauri/tests/spec_081_fleet_projection.rs` : homonymes, suppression de `canonical_path`, source en erreur, source locale dynamique et absence de verrou durant I/O simulée.
- [x] T003 Écrire les témoins purs de présentation dans `apps/bridget-desktop/ui/fleet-presentation.test.mjs` : filtre indépendant, clé composée, ordre stable, critères inversés et coordinateur explicite.
- [x] T004 Écrire les témoins de découverte locale et d'URL de panneau compact dans `apps/bridget-desktop/src-tauri/src/connection.rs` : commande constante, endpoint versionné, boucle locale, jeton encodé, `desktop_shell=1` et agent encodé.

## Phase 2 - Fondations de projection et de sécurité

- [x] T005 Implémenter dans `apps/bridget-desktop/src-tauri/src/connection.rs` la découverte dynamique et non persistée de « Cet ordinateur », puis les DTO privés et publics dans `apps/bridget-desktop/src-tauri/src/fleet.rs`, avec réduction explicite de `canonical_path` et clé `source_id:agent_name`.
- [x] T006 Déclarer le module `fleet` dans `apps/bridget-desktop/src-tauri/src/lib.rs` et étendre les chemins authentifiés dans `apps/bridget-desktop/src-tauri/src/connection.rs` sans faire passer de secret par IPC.
- [x] T007 Étendre `apps/bridget-desktop/src-tauri/src/preferences_store.rs` avec épingles, exclusions de coordinateur, tris et groupes repliés versionnés, en gardant la lecture compatible des préférences existantes.
- [x] T008 Ajouter la commande Tauri `fleet_snapshot` dans `apps/bridget-desktop/src-tauri/src/lib.rs` en copiant les sessions avant `request_relay_json`, en joignant « Cet ordinateur » disponible et en isolant chaque échec de source.
- [x] T009 Étendre les tests de contrat dans `apps/bridget-desktop/src-tauri/tests/desktop_commands.rs` pour prouver que `fleet_snapshot` ne contient jamais jeton, URL, détail SSH ou chemin de projet.
- [x] T010 [P] Ajouter les tests d'intégration de projection dans `apps/bridget-desktop/src-tauri/tests/spec_081_fleet_projection.rs` pour les réponses relais illisibles, le projet sans agent et deux homonymes.

## Phase 3 - US1 Retrouver toute la flotte - P1

**But** : voir les agents de toutes les sources connectées et ouvrir exactement la conversation de leur origine.  
**Test indépendant** : deux sources, deux homonymes, perte d'une source, ouverture vérifiée.

- [x] T011 [US1] Adapter `apps/bridget-desktop/src-tauri/src/lib.rs` afin que `panel_open` reçoive `source_id` et `agent_name`, résolve tunnel SSH ou « Cet ordinateur », construise une URL compacte et refuse une source non connectée.
- [x] T012 [US1] Modifier `apps/bridget-desktop/src-tauri/src/lib.rs` pour réserver la largeur de coque Desktop à gauche dans `arrange_panels`, sans augmenter `MAXIMUM_OPEN_PANELS`.
- [x] T013 [P] [US1] Ajouter le mode `desktop_shell=1` dans `crates/bridget-daemon/assets/ui/app.js`, en conservant la sélection par `agent` et les contrôles conversationnels.
- [x] T014 [P] [US1] Ajouter les règles de mode compact dans `crates/bridget-daemon/assets/ui/theme.css` afin de cacher seulement Projet, Agents et leur séparateur, sans masquer conversation ni dialogues de projet.
- [x] T015 [US1] Recomposer la structure persistante de flotte dans `apps/bridget-desktop/ui/index.html` : Sources, liste d'agents globale, région conversation et gestionnaire local.
- [x] T016 [US1] Implémenter dans `apps/bridget-desktop/ui/fleet-app.js` appelé par `apps/bridget-desktop/ui/app.js` le chargement de `fleet_snapshot`, les cartes source/projet explicites, l'état source indisponible et l'ouverture de l'agent avec son `source_id`.
- [x] T017 [US1] Mettre en forme la coque à trois zones dans `apps/bridget-desktop/ui/fleet-desktop.css` afin que le parent reste visible pendant le panneau enfant et que la liste reste lisible à 200 agents.
- [x] T018 [US1] Ajouter des témoins d'origine, de géométrie et de panneau unique dans `apps/bridget-desktop/src-tauri/tests/desktop_commands.rs` et `apps/bridget-desktop/src-tauri/tests/two_panels.rs`.
- [x] T019 [US1] Ajouter les tests Node du mode compact et de l'agent ciblé dans `crates/bridget-daemon/assets/ui/app.js`, puis exécuter `node --test crates/bridget-daemon/assets/ui/app.js`.

## Phase 4 - US2 Filtrer par sources et projets - P1

**But** : utiliser Sources comme un filtre explicite et garder les serveurs accessibles.  
**Test indépendant** : source, projet puis retrait indépendant des deux chips.

- [x] T020 [US2] Implémenter la colonne Sources dans `apps/bridget-desktop/ui/fleet-app.js` avec Toute la flotte, sources, projets et état de connexion issus exclusivement de `fleet_snapshot`, en affichant exactement « Cet ordinateur » pour la source locale.
- [x] T021 [US2] Implémenter les chips indépendants de filtre source et projet dans `apps/bridget-desktop/ui/fleet-presentation.js` et `apps/bridget-desktop/ui/fleet-app.js`, avec retrait clavier et aucun effet sur les tris.
- [x] T022 [US2] Ajouter dans `apps/bridget-desktop/ui/index.html` et `apps/bridget-desktop/ui/fleet-app.js` le sélecteur explicite de source cible pour Créer et Importer quand aucun filtre source ne s'applique.
- [x] T023 [US2] Étendre `apps/bridget-desktop/src-tauri/src/connection.rs` et `apps/bridget-desktop/src-tauri/src/lib.rs` pour transmettre seulement à la source choisie l'intention `desktop_action=create_project|import_project`.
- [x] T024 [US2] Consommer l'intention de projet dans `crates/bridget-daemon/assets/ui/app.js` en réutilisant `beginProject("create"|"import")`, y compris lorsque `desktop_shell=1` cache la colonne Projet.
- [x] T025 [US2] Rendre « Gérer les serveurs » atteignable depuis la coque dans `apps/bridget-desktop/ui/index.html` et `apps/bridget-desktop/ui/fleet-app.js`, sans fermer tunnel ou panneau par défaut.
- [x] T026 [US2] Ajouter les tests clavier, source-cible et apparition conditionnelle de la source locale dans `apps/bridget-desktop/ui/fleet-presentation.test.mjs` et `apps/bridget-desktop/src-tauri/tests/desktop_commands.rs`.

## Phase 5 - US3 Organiser la flotte - P1

**But** : composer des tris visibles, réordonnables et repliables sans modifier les filtres.  
**Test indépendant** : trois critères, déplacement, inversion, retrait et repli d'un groupe.

- [x] T027 [US3] Implémenter la projection pure filtres, ordre stable et regroupements dans `apps/bridget-desktop/ui/fleet-presentation.js` avec une complexité O(a log a) maximale.
- [x] T028 [US3] Implémenter les critères Source, État, Projet, Activité récente et Nom, leurs directions et leur ordre sémantique « À traiter d'abord » dans `apps/bridget-desktop/ui/fleet-presentation.js`.
- [x] T029 [US3] Ajouter les chips de tri réordonnables au pointeur et au clavier, inversables et supprimables dans `apps/bridget-desktop/ui/index.html`, `apps/bridget-desktop/ui/fleet-app.js` et `apps/bridget-desktop/ui/fleet-desktop.css`.
- [x] T030 [US3] Ajouter les groupes repliables et persistés localement dans `apps/bridget-desktop/ui/fleet-app.js` et `apps/bridget-desktop/src-tauri/src/preferences_store.rs`, puis vérifier qu'ils ne changent ni filtres ni agents.
- [x] T031 [US3] Étendre `apps/bridget-desktop/ui/fleet-presentation.test.mjs` avec trois critères, sens inverses, stabilité, état sémantique, groupe replié et 200 agents sous 150 ms.

## Phase 6 - US4 Épingler les coordinateurs - P2

**But** : mettre un coordinateur explicitement déclaré en tête tout en respectant le choix local.  
**Test indépendant** : pré-épinglage, désépinglage, rafraîchissement, réépinglage et regroupement.

- [x] T032 [US4] Ajouter la détection stricte `agent_link.role === "coordinator"` et la règle de pré-épinglage dans `apps/bridget-desktop/src-tauri/src/fleet.rs` et `apps/bridget-desktop/ui/fleet-presentation.js`, sans heuristique sur le nom.
- [x] T033 [US4] Ajouter l'action d'épingle locale et son affichage dans `apps/bridget-desktop/ui/fleet-app.js`, `apps/bridget-desktop/ui/index.html` et `apps/bridget-desktop/ui/fleet-desktop.css`.
- [x] T034 [US4] Étendre `apps/bridget-desktop/src-tauri/src/preferences_store.rs` et ses tests pour persister l'épinglage et le désépinglage d'un coordinateur entre deux chargements.
- [x] T035 [US4] Étendre `apps/bridget-desktop/ui/fleet-presentation.test.mjs` pour le coordinateur épinglé par défaut, le choix utilisateur durable et la première place dans un groupe.

## Phase 7 - Validation, convergence et documentation

- [x] T036 Exécuter et corriger les tests ciblés : `cargo test --manifest-path apps/bridget-desktop/src-tauri/Cargo.toml`, `node --test apps/bridget-desktop/ui/fleet-presentation.test.mjs`, `node --test crates/bridget-daemon/assets/ui/app.js`, puis consigner les résultats dans `specs/081-flotte-globale-sources-ui/evidence/validation.md`.
- [x] T037 Exécuter et corriger `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` et `cargo test --workspace` depuis `/home/moi/bridget-referent/.worktrees/session-081-flotte-globale-sources-ui`, puis consigner chaque résultat dans `specs/081-flotte-globale-sources-ui/evidence/validation.md`.
- [x] T038 Vérifier le parcours manuel de `specs/081-flotte-globale-sources-ui/quickstart.md` et consigner les limites macOS ou multi-serveur dans `specs/081-flotte-globale-sources-ui/evidence/manual-validation.md`.
- [x] T039 Rejouer `specs/081-flotte-globale-sources-ui/reuse-audit.md`, mettre à jour le rapport d'analyse dans `specs/081-flotte-globale-sources-ui/analysis-report.md` et ajouter toute tâche manquante sous `## Convergence` si nécessaire.
- [x] T040 Consigner la revue adverse ou son indisponibilité dans `specs/081-flotte-globale-sources-ui/adversarial-review.md`, effectuer l'auto-revue de diff et produire `specs/081-flotte-globale-sources-ui/audit.md`.
- [x] T041 Mettre à jour preuves, état des tâches et `specs/081-flotte-globale-sources-ui/implementation.md`, sans commit, merge, push, déploiement ni redémarrage.

## Opportunités de parallélisme

- Après T008 : T013-T014 et T015-T017 sont séparables, mais les changements de `app.js` doivent être séquencés par un seul propriétaire.
- Après T021 : T027-T028 peuvent être développés avant la couche de rendu T029.
- Les tests Rust T010 et Node T019 peuvent être lancés en parallèle après leur implémentation.

## Stratégie

Le premier incrément utilisable est US1 : flotte globale, origine lisible et ouverture correcte. US2 rend la navigation Sources réellement utile. US3 et US4 enrichissent l'organisation sans toucher aux autorités métier.

**Validation Article XX** : `fleet.rs` réduit la frontière IPC et `fleet-presentation.js` isole les règles pures testées hors WebView. Ces deux modules sont justifiés par des responsabilités distinctes, sans transport, store ni framework nouveau. T009, T018, T031, T036-T040 réduisent la charge du mainteneur en vérifiant explicitement les frontières de sécurité, l'origine et les comportements visibles.
