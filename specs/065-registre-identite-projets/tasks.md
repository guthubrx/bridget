# Tâches: Registre et identité des projets

**Statut**: terminée, avec validation workspace non verte sur une baseline hors SPEC-065
**Gate**: nouveau worktree créé depuis `d589b24` ou un descendant propre
contenant SPEC-068, `reuse-audit.md` rejoué et `implementation.md` mis à jour
après chaque tâche prouvée. L'absence de `.specify` ou de l'outil `specify` ne
doit déclencher ni installation ni mise à jour. Le présent worktree documentaire
initial ne porte aucune implémentation.

## Phase 1 - Préparation

- [x] T001 Créer le scénario Gherkin dans `tests/features/065-registre-identite-projets.feature` avec les parcours US1-US3 et l'oracle de non-destruction.
- [x] T002 Figer les exemples valides et invalides dans `specs/065-registre-identite-projets/contracts/project-registry-v1.md` et `specs/065-registre-identite-projets/contracts/project-root-policy-v1.md`, puis vérifier leur cohérence avec FR-001 à FR-030.
- [x] T003 Depuis un nouveau worktree créé sur `d589b24` ou un descendant propre, rejouer `specs/065-registre-identite-projets/reuse-audit.md`, inventorier les types SPEC-064 et SPEC-068 réellement présents et consigner tout nouvel équivalent avant création de table, type ou commande.

## Phase 2 - Fondations bloquantes

- [x] T004 [P] Ajouter les tests de contrat fermés pour les variantes dédiées ProjectRegistryRequest/ProjectRegistryOutcome dans `crates/bridget-transport/src/protocol.rs`, y compris direction Maicie vers Bridget, rôle local, UID pair, négociation `project_registry_v1`, version inconnue, collision avec existing_project_id et génération, avant les types productifs.
- [x] T005 [P] Ajouter les tests de transitions ProjectIdentity dans `plugins/maicie/src/domain.rs`, y compris pending_binding, registration_conflict, activation gagnante, désactivation et refus de référence avant activation.
- [x] T006 [P] Ajouter les tests de transitions ProjectBinding dans `crates/bridget-daemon/src/store.rs`, y compris path_missing, rebind et génération.
- [x] T007 Ajouter les variantes publiques dédiées et les raisons structurées dans `crates/bridget-transport/src/protocol.rs`, sans réutiliser `ServiceRequest`, avec compatibilité de décodage des fixtures historiques.
- [x] T008 Ajouter ProjectIdentity et ProjectReference dans `plugins/maicie/src/domain.rs`, sans chemin hôte ni état runtime privé.
- [x] T009 Ajouter ProjectBinding, ProjectAuditEvent et leurs index dans `crates/bridget-daemon/src/store.rs`, avec migration additive, unicité déterministe de l'audit et permissions privées inchangées.

## Phase 3 - User Story 1: enregistrer un projet (P1)

**Test indépendant**: une action, un project_id, une liaison host, puis rejeu après crash.

- [x] T010 [P] [US1] Ajouter les tests SQLite d'identité, unicité, commande idempotente et deux intentions concurrentes dans `plugins/maicie/src/store.rs` avant la migration productive.
- [x] T011 [P] [US1] Ajouter les tests Bridget de politique absente, vide, invalide, propriétaire/mode interdits, canonicalisation, préfixes autorisés, liens symboliques et racines trop larges dans `crates/bridget-daemon/src/store.rs` et la surface de chargement de configuration.
- [x] T012 [US1] Persister ProjectIdentity pending_binding, ProjectRegistrationCommand et l'outbox dans une transaction de `plugins/maicie/src/store.rs`, puis promouvoir uniquement resolved_project_id après issue Bridget avec résultat observable par command_id.
- [x] T013 [US1] Implémenter la commande locale d'enregistrement et sa reprise dans `plugins/maicie/src/main.rs` et `plugins/maicie/src/app.rs`, sans seconde action utilisateur.
- [x] T014 [US1] Charger la politique de racines depuis le chemin absolu explicite du daemon, authentifier rôle/capability/UID, puis recevoir, canonicaliser et sérialiser ProjectBindRequest dans `crates/bridget-daemon/src/daemon.rs` et `crates/bridget-daemon/src/store.rs`, backend fixé à host, avec issue de collision portant existing_project_id et génération et exactement un ProjectAuditEvent transactionnel pour la liaison créée.
- [x] T015 [US1] Étendre `plugins/maicie/src/bridget_client.rs` pour négocier `project_registry_v1`, envoyer la variante dédiée et relire l'issue exacte de liaison sans lire la base Bridget.
- [x] T016 [US1] Injecter un crash aux frontières Maicie/outbox/Bridget/accusé et deux commandes concurrentes par alias/symlink dans `plugins/maicie/tests/project_registration_integration.rs`, puis prouver une seule identité active et une issue durable pour la perdante.
- [x] T017 [US1] Consigner le parcours US1, la course canonique, commandes, versions, identités actives et comptes de lignes dans `specs/065-registre-identite-projets/evidence/us1-register-replay.md`.

## Phase 4 - User Story 2: administrer sans détruire (P1)

**Test indépendant**: path_missing, rebind, disable et empreinte du dépôt inchangée.

- [x] T018 [P] [US2] Ajouter les tests de list, status, rebind, disable et `review-project reconcile --dry-run/confirm` idempotent dans `crates/bridget-daemon/tests/project_registration_e2e.rs` et `plugins/maicie/src/main.rs`, y compris absence de création au démarrage et pendant les migrations. Le test vit côté daemon car Maicie ne peut pas dépendre du daemon en test sans cycle de dépendances.
- [x] T019 [P] [US2] Ajouter les tests d'historique de liaison, non-suppression et ProjectAuditEvent pour register/rebind/disable/reconcile, y compris rejeu produisant exactement un événement, ainsi que rebind avec agent actif conservé sur son ancienne génération dans `crates/bridget-daemon/tests/project_binding_integration_test.rs`.
- [x] T020 [US2] Implémenter list, status, rebind, disable et le rapprochement local explicite de `review_project` dans `plugins/maicie/src/main.rs` et `plugins/maicie/src/app.rs`, avec prévisualisation, confirmation et prochaine action structurée.
- [x] T021 [US2] Implémenter rebind et disable idempotents dans `crates/bridget-daemon/src/daemon.rs` et `crates/bridget-daemon/src/store.rs`, avec ProjectAuditEvent écrit dans la transaction de chaque mutation effective et sans arrêt implicite; conserver les exécutions actives sur leur ancienne génération.
- [x] T022 [US2] Ajouter la projection de fraîcheur et d'indisponibilité dans `plugins/maicie/src/runtime.rs` et `plugins/maicie/src/ui_projection.rs`.
- [x] T023 [US2] Prouver l'empreinte identique du dépôt avant et après disable/rebind, ainsi que l'unicité et l'absence de contenu sensible des ProjectAuditEvent, puis consigner le résultat dans `specs/065-registre-identite-projets/evidence/us2-safety.md`.

## Phase 5 - User Story 3: corréler agents et missions (P2)

**Test indépendant**: projet A, projet B et agent historique restent distingués.

- [x] T024 [P] [US3] Ajouter les tests de compatibilité sans project_id, refus projet/cwd divergent, snapshot/restart, reprise par curseur, liens parent-enfant et incidents runtime délégués SPEC-068 avec ProjectReference inchangée dans les modules stabilisés par SPEC-064 et SPEC-068.
- [x] T025 [P] [US3] Ajouter les tests de divergence délégation/exécution dans `plugins/maicie/src/app.rs` sans transition métier automatique.
- [x] T026 [US3] Propager et persister ProjectReference optionnelle dans `crates/bridget-transport/src/protocol.rs`, `crates/bridget-daemon/src/fleet.rs`, `crates/bridget-daemon/src/desired_state.rs`, `crates/bridget-daemon/src/execution_store.rs`, `crates/bridget-daemon/src/idempotency.rs`, les ordres Maicie persistés, `ExecutionReference`, `ExecutionProjection`, snapshots, curseurs, `AgentLinkRecord`, `AgentLinkEvent`, `DelegatedRuntimeEventRecord` et `DelegatedRuntimeEventFrame`; l'historique sans projet reste `None`.
- [x] T027 [US3] Valider l'appartenance du cwd à la liaison ou à un worktree rattaché dans `crates/bridget-daemon/src/lifecycle.rs`, avant le lancement provider.
- [x] T028 [US3] Afficher project_id, domain et état unregistered séparément dans `crates/bridget-daemon/src/ui.rs` et `crates/bridget-daemon/assets/ui/app.js`.

## Phase 6 - Validation transversale

- [x] T029 Exécuter `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` et `cargo test --workspace`, puis consigner versions, comptes et verdicts dans `specs/065-registre-identite-projets/evidence/final-validation.md` et achever le journal `specs/065-registre-identite-projets/implementation.md`.
- [x] T030 Rejouer Analyze, la revue hostile chemins/symlinks/crash/non-destruction, archiver la session selon le workflow SpecKit disponible et mettre à jour `specs/065-registre-identite-projets/analysis-report.md` sans passer le statut à Implemented tant qu'une tâche reste ouverte.

## Dépendances

```text
T001-T003
   |
T004-T009
   |
US1 T010-T017
   |
US2 T018-T023
   |
US3 T024-T028
   |
T029-T030
```

- T004, T005 et T006 sont parallèles car ils touchent des frontières distinctes.
- US2 dépend de la saga US1.
- US3 dépend de la liaison stable et des contrats SPEC-064.
- Le MVP utile est US1 + US2; US3 complète la corrélation nécessaire à 066.

## Article XIX et XX

- Aucun nouveau crate, daemon ou dépendance externe.
- Les nouveaux concepts ont trois consommateurs réels: Maicie, Bridget et les
  projections/exécutions.
- `domain`, `review_project`, outboxes, stores et SpawnOrder sont réutilisés ou
  explicitement distingués.
- Chaque tâche non triviale possède un oracle, une preuve ou un chemin de test.
