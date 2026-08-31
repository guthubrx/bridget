# Tâches: Profils, extensions et secrets bornés par projet

**Statut**: completees avec echec de regression preexistant documente
**Gate**: nouveau worktree créé depuis `dda4ec2` (`main` après SPEC-066),
`reuse-audit.md` rejoué et
`implementation.md` mis à jour après chaque tâche prouvée. L'absence de
`.specify` ou de l'outil `specify` ne doit déclencher ni installation ni mise à
jour. Ce worktree isolé est le seul périmètre d'implémentation de SPEC-067.

## Phase 1 - Préparation

- [x] T001 Créer `tests/features/067-profils-extensions-secrets-projet.feature` avec approbation, extensions, secrets, rotation et trois fournisseurs.
- [x] T002 Créer uniquement des extensions et secrets synthétiques sous `crates/bridget-daemon/tests/fixtures/project-profile/`, avec sentinelles de fuite sans credential réel.
- [x] T003 Rejouer `specs/067-profils-extensions-secrets-projet/reuse-audit.md` contre la tête d'implémentation contenant SPEC-068 avant tout nouveau type, montage ou dépendance.

## Phase 2 - Fondations bloquantes

- [x] T004 [P] Ajouter les tests ProjectProfile et ProjectProfileApproval dans `plugins/maicie/src/project_profile.rs`, y compris binding_generation, runtime_policy_version, policy_digest et transition stale sur rebind, switch backend ou `runtime_policy_changed`, avant les types productifs.
- [x] T005 [P] Ajouter les tests de contrat ProjectProfileProposal et ResolvedProjectProfile dans `crates/bridget-transport/src/project_profile_protocol.rs`, avec générations, runtime_policy_version et politique obligatoires, source_revision absente de la proposition puis obligatoire dans la résolution et l'approbation.
- [x] T006 [P] Ajouter les tests ExtensionRef/SecretRef, catalogue absent/invalide, projet non autorisé, collisions, bornes, digests et SecretSourceStamp sans contenu dans `crates/bridget-daemon/src/project_runtime.rs`.
- [x] T007 Étendre `plugins/maicie/src/config.rs` et `plugins/maicie/src/profiles.rs` avec ProjectProfile qui référence les AgentProfiles existants.
- [x] T008 Étendre `crates/bridget-transport/src/protocol.rs` avec les références, générations, attestations et raisons sans valeur secrète.
- [x] T009 Étendre ProjectRuntimePolicy dans `crates/bridget-daemon/src/project_runtime.rs` pour digester runtime_policy_version, extensions, SecretRefs, source_revision et stamps sans stocker de valeur.

## Phase 3 - User Story 1: définir et approuver (P1)

**Test indépendant**: vue exhaustive, digest exact, approbation locale seulement.

- [x] T010 [P] [US1] Ajouter les tests de vue exhaustive et absence de valeurs dans `plugins/maicie/src/profiles.rs`.
- [x] T011 [P] [US1] Ajouter les tests qui prouvent l'absence de routes approve/rotate/revoke dans `crates/bridget-daemon/src/mcp.rs` et `crates/bridget-daemon/src/ui.rs`.
- [x] T012 [US1] Étendre proposition et approbation locale dans `plugins/maicie/src/app.rs` et `plugins/maicie/src/store.rs`, avec profil, projet, binding_generation, runtime_policy_version, policy_digest, source revisions, génération et digest.
- [x] T013 [US1] Résoudre AgentProfiles contre AgentRegistry dans `plugins/maicie/src/bridget_client.rs` et `crates/bridget-daemon/src/registry.rs`, sans heuristique fournisseur.
- [x] T014 [US1] Afficher la vue locale complète et l'avertissement de confiance projet dans `plugins/maicie/src/main.rs`, sans valeur ni approbation distante.
- [x] T015 [US1] Consigner le parcours d'approbation et les scans négatifs dans `specs/067-profils-extensions-secrets-projet/evidence/us1-approval.md`.

## Phase 4 - User Story 2: extensions approuvées (P1)

**Test indépendant**: deux extensions read-only, deux projets, aucune visibilité croisée.

- [x] T016 [P] [US2] Ajouter les tests catalogue source/destination/projet autorisé/owner/mode/digest/symlink dans `crates/bridget-daemon/tests/project_extensions_test.rs`.
- [x] T017 [P] [US2] Ajouter les tests de montages read-only et isolation entre projets dans `crates/bridget-daemon/tests/project_extensions_integration_test.rs`.
- [x] T018 [US2] Charger `project-resource-catalog-v1` puis implémenter la résolution et l'attestation ExtensionRef dans `crates/bridget-daemon/src/project_runtime.rs`, avec projet autorisé, destinations fermées et collisions refusées.
- [x] T019 [US2] Ajouter les montages d'extensions à create/inspect dans `crates/bridget-daemon/src/project_runtime.rs`, sans téléchargement ni option libre.
- [x] T020 [US2] Détecter une divergence avant spawn dans `crates/bridget-daemon/src/lifecycle.rs` et persister `recreate_required` dans `crates/bridget-daemon/src/store.rs`.
- [x] T021 [US2] Prouver read-only, absence croisée et digest divergent dans `specs/067-profils-extensions-secrets-projet/evidence/us2-extensions.md`.

## Phase 5 - User Story 3: secrets et rotation (P1)

**Test indépendant**: file/dir/process-env synthétiques, zéro fuite, rotation et ancienne génération absente.

- [x] T022 [P] [US3] Ajouter les tests de sources secrètes, catalogue/projet autorisé, UID/GID runtime, permissions, propriétaires, symlinks, dépôt interdit et mutations fichier/répertoire détectées par SecretSourceStamp dans `crates/bridget-daemon/tests/project_secrets_test.rs`.
- [x] T023 [P] [US3] Ajouter les tests process-env qui émettent après démarrage une sentinelle entière puis découpée au milieu, au retour ligne et sur chaque canal, et prouvent que JournalWriter ne reçoit jamais les octets bruts, puis scanner args, inspect, ps, stores, logs, UI, DelegatedRuntimeEventFrame et le store d'incidents SPEC-068 dans `crates/bridget-daemon/tests/project_secret_leak_test.rs`.
- [x] T024 [P] [US3] Ajouter les tests de rotation/recreate/refus ancienne génération, mutation hors rotation, rebind, switch backend et runtime_policy_version divergente avec refus avant ressource dans `plugins/maicie/tests/project_secret_rotation_integration.rs`.
- [x] T025 [US3] Implémenter SecretRef, génération et décisions de rotation/révocation dans `plugins/maicie/src/domain.rs`, `plugins/maicie/src/app.rs` et `plugins/maicie/src/store.rs`.
- [x] T026 [US3] Résoudre exclusivement par le catalogue hôte, calculer/attester source_revision et SecretSourceStamp sans valeur dans `crates/bridget-daemon/src/project_runtime.rs`, puis revérifier le stamp avant tout montage et spawn.
- [x] T027 [US3] Monter file/dir read-only et les sources process-env réservées dans `crates/bridget-daemon/src/project_runtime.rs`.
- [x] T028 [US3] Faire lire process-env par `crates/bridget-daemon/src/wrapper.rs` après admission capabilities et créer OutputRedactionLease avant spawn provider, sans argument Docker ni persistance.
- [x] T029 [US3] Imposer la redaction binaire à état conservé par canal avant JournalWriter et avant toute construction d'incident runtime délégué dans `crates/bridget-daemon/src/wrapper.rs` et `crates/bridget-daemon/src/daemon.rs`, n'autoriser dans SPEC-068 que code fermé, référence pseudonymisée et ProjectReference, garder la lease jusqu'à fermeture complète et ajouter un mutant qui réinitialise ou transmet avant comparaison.
- [x] T030 [US3] Imposer arrêt des agents et recreate lors de rotation/révocation, puis stale et nouvelle approbation lors de rebind, switch backend, runtime_policy_changed ou divergence de stamp dans `crates/bridget-daemon/src/lifecycle.rs`, `crates/bridget-daemon/src/project_runtime.rs` et `plugins/maicie/src/app.rs`, sans arrêter les agents déjà actifs sur l'ancienne génération.
- [x] T031 [US3] Prouver zéro fuite, disparition de l'ancienne génération et refus après rebind, switch backend, politique ou mutation hors rotation avant lecture/montage dans `specs/067-profils-extensions-secrets-projet/evidence/us3-secrets.md`.

## Phase 6 - User Story 4: portabilité fournisseurs (P2)

**Test indépendant**: même profil et mêmes refus sur Codex, Claude et Cursor ACP.

- [x] T032 [P] [US4] Ajouter un contrat commun profils/capabilities dans `crates/bridget-transport/tests/project_profile_provider_contract.rs` pour Codex, Claude, Cursor ACP et un faux provider.
- [x] T033 [P] [US4] Ajouter les tests backend host qui refusent les profils à montages Docker dans `crates/bridget-daemon/tests/project_profile_host_compat_test.rs`.
- [x] T034 [US4] Brancher la validation fournisseur-neutre dans `crates/bridget-daemon/src/lifecycle.rs` et les ManagedSession existantes, sans branche Cursor hors ACP.
- [x] T035 [US4] Consigner la matrice providers/capabilities/refus dans `specs/067-profils-extensions-secrets-projet/evidence/us4-providers.md`.

## Phase 7 - Validation transversale

- [x] T036 Exécuter une revue sécurité locale sur approbations, sources, montages, redaction, générations et limite intra-projet, puis écrire `specs/067-profils-extensions-secrets-projet/evidence/security-review.md` sans aucune valeur réelle.
- [x] T037 Exécuter `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` et `cargo test --workspace`, puis consigner dans `specs/067-profils-extensions-secrets-projet/evidence/final-validation.md` et achever `specs/067-profils-extensions-secrets-projet/implementation.md`.
- [x] T038 Rejouer Analyze, archiver la session selon le workflow SpecKit disponible et mettre à jour `specs/067-profils-extensions-secrets-projet/analysis-report.md`, sans activer de credential réel ni passer le statut à Implemented tant qu'une tâche reste ouverte.

## Dépendances

```text
T001-T003
   |
T004-T009
   |
US1 T010-T015
   |
US2 T016-T021
   |
US3 T022-T031
   |
US4 T032-T035
   |
T036-T038
```

- Les tests de fondation T004-T006 sont parallèles.
- Les extensions précèdent les secrets afin de prouver les montages sans valeur.
- US4 dépend de l'admission et des secrets mais reste testable avec fixtures.
- Le MVP utile est US1-US3 sur un provider fixture; les vrais credentials ne
  sont jamais nécessaires à la première preuve.

## Article XIX et XX

- Aucun secret service, broker, Vault, mémoire globale ou plugin manager.
- Les profils existants restent l'autorité; ProjectProfile est une composition.
- Recreate remplace un mécanisme de hot reload complexe.
- Chaque tâche sensible inclut un scanner négatif et une preuve observable.
- La limite intra-projet est un invariant documenté, pas une dette cachée.

## Cloture de preuve 2026-08-31

T006 a T036 sont couverts par les contrats, catalogues, admissions runtime, redaction et tests cibles consignes dans evidence/. T037 a execute fmt, Clippy et la batterie workspace; quatre echecs managed_parity_test sont reproduits a l identique sur la base dda4ec2198b941cc38915a00f847df435d80934d et sont documentes dans evidence/final-validation.md. T038 a rejoue l analyse et ne cree ni ne met a jour aucun SpecKit global, conformement a la decision utilisateur.
