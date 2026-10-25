# Tâches — SPEC-167 Coordination CLI autonome

**Statut :** Implemented
**Progression :** 28/28
**Pipeline :** Livré

Chaque tâche reste ouverte jusqu’à sa preuve. Le moteur natif reste la seule source de vérité.

## Phase 1 — Cadrage et réutilisation

- [x] T001 Figer la spécification et la checklist dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/167-coordination-cli-autonome/spec.md`
- [x] T002 Résoudre les choix d’architecture et l’identité du parent dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/167-coordination-cli-autonome/plan.md`
- [x] T003 Auditer les moteurs, adaptateurs et commandes réutilisables dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/167-coordination-cli-autonome/reuse-audit.md`
- [x] T004 Consigner la recherche sur l’orchestration simple et contrôlable dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/167-coordination-cli-autonome/research.md`
- [x] T005 Définir le contrat CLI fermé dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/167-coordination-cli-autonome/contracts/cli.md`
- [x] T006 Tenter la contre-revue adverse du plan et traiter ses objections dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/167-coordination-cli-autonome/adversarial-review-plan.md`

## Phase 2 — Fondations testables

- [x] T007 [US1] Ajouter les oracles RED du parser `delegate` nominal, des options et des refus dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs`
- [x] T008 [US3] Ajouter les oracles RED de chaque sous-commande `task` et de sa charge native dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs`
- [x] T009 [US4] Ajouter les oracles RED de sortie humaine, sortie JSON unique et refus sans identité dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs`
- [x] T010 Conserver les résultats RED dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/167-coordination-cli-autonome/validation/red.md`

## Phase 3 — US1 Confier une mission en une commande

**But :** un coordinateur authentifié démarre une mission avec son seul texte.
**Test indépendant :** `delegate` produit la charge `bridget_auto_delegate` avec mission, dossier courant et clé sûre.

- [x] T011 [US1] Ajouter le routage et l’aide de `bridget delegate` dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs`
- [x] T012 [US1] Parser la mission positionnelle, le dossier courant et la clé générée sans injecter d’autre valeur implicite dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs`
- [x] T013 [US1] Appeler `delegation_mcp::execute` avec l’identité vivante et rendre le reçu durable dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs`
- [x] T014 [US1] Rendre le refus sans identité actionnable et sans effet métier dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs`

## Phase 4 — US2 Garder les choix avancés

**But :** les choix explicites traversent la façade sans substitution.
**Test indépendant :** chaque option produit le même champ que `bridget_auto_delegate` ou un refus local précis.

- [x] T015 [US2] Exposer les sélecteurs type, modèle, effort, posture, machine et projet dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs`
- [x] T016 [US2] Exposer délai, garde, demandes enfant, urgence, livrable, workspace et brief dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs`
- [x] T017 [US2] Vérifier les doublons, bornes, JSON fermé et conservation d’une clé explicite dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs`

## Phase 5 — US3 Piloter le cycle de vie

**But :** le coordinateur pilote ses tâches depuis le terminal.
**Test indépendant :** chaque sous-commande appelle l’outil natif existant avec la charge attendue.

- [x] T018 [US3] Ajouter `task status`, `wait` et `ack` dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs`
- [x] T019 [US3] Ajouter `task continue`, `respond` et `cancel` dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs`
- [x] T020 [US3] Ajouter `task list` comme alias documenté de la projection humaine globale existante dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs`
- [x] T021 [US3] Mutualiser l’exécution, la traduction des erreurs et l’affichage des résultats dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs`

## Phase 6 — US4 Automatiser par le parcours natif Bridget

**But :** les scripts utilisent une sortie stable et le même moteur dans un environnement natif isolé.
**Test indépendant :** `--json` produit un objet unique et la recette passe avec dans un environnement natif isolé.

- [x] T022 [US4] Implémenter la sortie JSON pure et la vue humaine concise dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs`
- [x] T023 [US4] Prouver le rejeu par clé explicite et la compatibilité de `auto-delegate` dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs`
- [x] T024 [US4] Ajouter une recette isolée par le parcours natif Bridget et consigner son résultat dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/167-coordination-cli-autonome/validation/tests.md`

## Phase 7 — Documentation, qualité et livraison

- [x] T025 Documenter l’invariant d’identité et le parcours par le parcours natif Bridget dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/docs/decisions/167-coordination-cli-autonome.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/docs/reference-communication.md`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/README.md` et `/Users/moi/Nextcloud/10.Scripts/64.bridget/CHANGELOG.md`
- [x] T026 Exécuter format, tests ciblés, régressions et suite proportionnée ; consigner les codes retour dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/167-coordination-cli-autonome/validation/tests.md`
- [x] T027 Faire converger l’implémentation, tenter la contre-revue inter-fournisseur et produire l’audit final dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/167-coordination-cli-autonome/validation/`
- [x] T028 Intégrer, pousser, construire, signer, sauvegarder, installer, vérifier le runtime et nettoyer sans force ; finaliser `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/167-coordination-cli-autonome/validation/livraison167.md`

## Dépendances

`T001–T006 → T007–T010 → T011–T014 → T015–T017 → T018–T021 → T022–T024 → T025–T028`

## Possibilités de travail parallèle

Le code et ses tests vivent dans le même fichier. Ils restent séquentiels. La documentation peut commencer après la stabilisation du contrat. La livraison attend tous les contrôles.

## Stratégie d’implémentation

La première tranche testable est `delegate`. Le groupe `task` vient ensuite. La sortie machine clôt le parcours. Aucun minimum livrable ne permet d’omettre une histoire utilisateur : la session doit livrer le cycle complet validé.
