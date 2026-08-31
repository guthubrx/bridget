# Tâches - Pilotage des rondes par projet dans l'interface

**Spec**: `081-pilotage-rondes-projet-ui`
**Branche**: `session-081-pilotage-rondes-projet-ui`

## Phase 1 - Tests et contrats initiaux

- [x] T001 Créer le scénario d'acceptation métier dans `tests/features/081-pilotage-rondes-projet-ui.feature` couvrant lecture, activation, désactivation, rebind et absence d'effet sur les travaux.
- [x] T002 [P] Ajouter les tests de sérialisation et compatibilité des faits de dernier passage dans `crates/bridget-transport/src/protocol.rs` avant d'étendre le contrat.
- [x] T003 [P] Ajouter les tests de migration, projection absente, écriture monotone et isolation de génération dans `crates/bridget-daemon/src/store.rs` avant le code productif.
- [x] T004 [P] Ajouter les tests du relais pour lecture jointe, capacité absente, mutation confirmée, génération divergente et corps inconnu dans `crates/bridget-daemon/src/ui.rs`.
- [x] T005 [P] Ajouter les tests Node de projection, libellés, accessibilité et construction de mutation dans `crates/bridget-daemon/assets/ui/app.js` avant le rendu.

## Phase 2 - Fondations bloquantes

- [x] T006 Étendre `ProjectRoundProjection` avec l'état fermé et les faits optionnels du dernier passage dans `crates/bridget-transport/src/protocol.rs`, sans branche fournisseur ni rupture des données historiques.
- [x] T007 Ajouter la migration additive et projeter les nouveaux champs dans `crates/bridget-daemon/src/store.rs`; une base existante doit rester lisible sans réécriture.
- [x] T008 Ajouter dans `crates/bridget-daemon/src/store.rs` l'enregistrement monotone O(1) d'un résultat de dispatch sur la génération exacte.

## Phase 3 - User Story 1: piloter la ronde d'un projet actif

**But**: une activation ou désactivation demandée depuis l'interface utilise l'autorité existante et n'est affichée qu'après confirmation.

**Test indépendant**: muter un projet actif, rejouer la commande, puis tenter la même action sur une génération obsolète; seule la première enveloppe valide change l'état.

- [x] T009 [US1] Classer puis persister l'issue de chaque dispatch admis par la politique dans `crates/bridget-daemon/src/daemon.rs`, sans modifier le scheduler, le message ou l'exécution des contrôles.
- [x] T010 [US1] Ajouter la négociation locale de `ProjectRoundPolicyV1` et joindre les politiques à `GET /v1/projects` dans `crates/bridget-daemon/src/ui.rs` avec une table de hachage O(p).
- [x] T011 [US1] Ajouter `POST /v1/projects/round` et sa validation fermée dans `crates/bridget-daemon/src/ui.rs`; traduire exactement les refus du daemon et retourner la projection confirmée.
- [x] T012 [US1] Faire passer les tests ciblés transport, store, daemon et relais pour activation, désactivation, idempotence et refus; consigner les commandes dans `specs/081-pilotage-rondes-projet-ui/implementation.md`.

## Phase 4 - User Story 2: comprendre l'état et le prochain passage

**But**: la ligne et le menu expliquent l'état confirmé, le dernier passage et le délai maximal du prochain cycle.

**Test indépendant**: rendre successivement un projet non configuré, activé, désactivé, inactif et rebindé; aucun état ni horaire ne doit être inventé.

- [x] T013 [US2] Ajouter les projections pures de présentation et de mutation dans `crates/bridget-daemon/assets/ui/app.js`; elles doivent traiter les champs absents et conserver la dernière valeur confirmée.
- [x] T014 [US2] Étendre le menu contextuel dans `crates/bridget-daemon/assets/ui/app.js` avec un `menuitemcheckbox`, un état occupé, le dernier passage et le prochain cycle maximal; réutiliser les trois points, le clic droit et `Maj + F10`.
- [x] T015 [US2] Ajouter le suffixe discret `ronde activée` à la ligne projet et les styles compacts du menu dans `crates/bridget-daemon/assets/ui/theme.css`, sans nouvel overlay ni fond global.
- [x] T016 [US2] Faire passer le programme Node complet et vérifier que l'échec d'une mutation ne modifie ni `projects` ni la ligne rendue; consigner le résultat dans `specs/081-pilotage-rondes-projet-ui/implementation.md`.

## Phase 5 - User Story 3: conserver une commande sûre et générique

**But**: la fonctionnalité reste indépendante du fournisseur, bornée à la génération et sans donnée sensible.

**Test indépendant**: utiliser les mêmes contrats sur des fixtures de transports différents et vérifier qu'aucun champ provider, secret, message ou chemin supplémentaire n'apparaît.

- [x] T017 [US3] Vérifier dans `crates/bridget-transport/src/protocol.rs`, `crates/bridget-daemon/src/daemon.rs` et `crates/bridget-daemon/src/ui.rs` qu'aucune condition fournisseur, commande libre ou donnée sensible n'a été ajoutée; supprimer toute branche constante détectée.
- [x] T018 [US3] Étendre les preuves de non-divulgation et de capacité absente dans les tests Rust et Node concernés, avec résultat observable dans `specs/081-pilotage-rondes-projet-ui/implementation.md`.

## Phase 6 - Validation, convergence et responsabilité future

- [x] T019 Exécuter `cargo fmt --check`, les tests ciblés transport et daemon, `ui_relay_test`, le programme Node, `cargo clippy --workspace --all-targets -- -D warnings` et `git diff --check`; documenter tout échec préexistant factuel dans `specs/081-pilotage-rondes-projet-ui/implementation.md`.
- [x] T020 Comparer chaque FR-081 et SC-081 avec le diff et les tests, corriger les écarts en trois boucles maximum et écrire `specs/081-pilotage-rondes-projet-ui/convergence.md`.
- [x] T021 Réaliser la revue hostile Minimalisme, complexité, frontière UI, génération, confidentialité et responsabilité LLM; écrire `specs/081-pilotage-rondes-projet-ui/adversarial-review-implementation.md` et corriger les findings confirmés.
- [x] T022 Exécuter l'audit final selon le protocole v14 disponible ou son repli manuel, écrire `specs/081-pilotage-rondes-projet-ui/audit.md`, puis synchroniser le statut et les compteurs de `specs/081-pilotage-rondes-projet-ui/spec.md`.

## Dépendances

```text
T001-T005 -> T006-T008 -> T009-T012 -> T013-T016 -> T017-T018 -> T019-T022
```

- US1 dépend des fondations de contrat et de store.
- US2 dépend de la projection confirmée de US1 mais peut être testée avec des objets purs.
- US3 est une vérification transversale après US1 et US2.
- Convergence et audit exigent toutes les stories terminées.

## Opportunités parallèles

- T002, T003, T004 et T005 portent sur des fichiers distincts et peuvent être préparées en parallèle, mais elles seront appliquées séquentiellement dans ce worktree pour éviter les conflits avec d'autres travaux du dépôt.
- Les validations Node et Rust sont indépendantes après T015.

## Stratégie d'implémentation

1. Faire échouer les preuves ciblées sur le contrat manquant.
2. Livrer l'autorité complète de US1 avant le rendu.
3. Ajouter US2 sans cache ni état secondaire.
4. Vérifier US3 et les frontières de confidentialité.
5. Ne conclure qu'après toutes les tâches, la convergence et l'audit.

## Validation Article XIX/XX

- La feature modifie six fichiers productifs existants et n'ajoute aucune dépendance.
- La migration est additive et la seule nouvelle donnée durable répond à l'ambiguïté du dernier passage.
- Aucun helper n'est créé pour un seul appel sauf s'il porte une négociation, une validation ou une migration de sécurité.
- Les tâches T017, T020 et T021 imposent la suppression du code constant, des branches fournisseur et des abstractions prématurées.
