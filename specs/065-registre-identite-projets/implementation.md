# Journal d'implémentation: SPEC-065

**Statut**: en cours
**Dernière mise à jour**: 2026-08-30

Worktree d'implémentation: `session-065-registre-identite-projets`, créé depuis
`main` à `ae2b71a` puis complété par les artefacts documentaires de la spec.
Les gates SPEC-063, SPEC-064, SPEC-068, concurrence et tête de branche ont été
vérifiées avant le premier changement. L'absence de `.specify` est explicitement
non bloquante et n'a déclenché aucune installation ni mise à jour.

## Journal des tâches

### T001 - Scénarios Gherkin US1-US3

- **Statut**: complétée
- **Commit**: `test(065): Ajouter les scénarios Gherkin`
- **Fichiers**: `tests/features/065-registre-identite-projets.feature`,
  `specs/065-registre-identite-projets/tasks.md`,
  `specs/065-registre-identite-projets/implementation.md`
- **Preuve**: les dix scénarios couvrent US1-US3, SC-001 à SC-011, le rejeu,
  les racines refusées, la non-destruction, le rebind, la désactivation, la
  corrélation durable et la compatibilité historique. Leur syntaxe suit les
  fichiers Gherkin versionnés par SPEC-023 et SPEC-064, sans nouveau runner ou
  dépendance.
- **Self-review Article XIX/XX**:
  - Pourquoi cette solution est nécessaire : elle fixe les oracles métier avant les types Rust.
  - Pourquoi elle est simple : un fichier Gherkin autonome, conforme aux précédents du dépôt.
  - Hypothèses : les scénarios restent des documents exécutables par lecture jusqu'à l'ajout futur d'un runner explicitement voulu.
  - Vérifications réalisées : lecture des précédents 023 et 064, couverture des trois user stories et des onze critères de succès.
  - Non vérifié : aucun runner Gherkin n'existe actuellement dans le workspace.
  - Code supprimé ou évité : aucun framework, dépendance ou couche de test ajoutés.
  - Complexité ajoutée et justification : O(1) par scénario documentaire, sans chemin runtime.

### T002 - Exemples de contrats et cohérence des exigences

- **Statut**: complétée
- **Commit**: `docs(065): Figer les exemples de contrat`
- **Complément de preuve**: `docs(065): Completer les exemples de contrat`
- **Fichiers**: `contracts/project-registry-v1.md`,
  `contracts/project-root-policy-v1.md`,
  `tests/features/065-registre-identite-projets.feature`, `tasks.md`,
  `implementation.md`
- **Preuve**: les dix cas du registre couvrent admission, rejeu, chemin,
  collision et politique; les sept cas de politique couvrent schéma, bornes et
  permissions. Les raisons du Gherkin correspondent désormais au contrat fermé.
  FR-001 à FR-030 ont été relus contre les principes, direction, outcomes,
  audit, compatibilité et politique; FR-027 a révélé les trois raisons de
  politique manquantes, ajoutées avant le premier test Rust.
- **Self-review Article XIX/XX**:
  - Pourquoi cette solution est nécessaire : les tests doivent dépendre de raisons stables et exhaustives.
  - Pourquoi elle est simple : les exemples sont dans les deux contrats propriétaires, sans nouvelle abstraction.
  - Hypothèses : les chemins d'exemple sont des fixtures non créées et non des valeurs de production.
  - Vérifications réalisées : comparaison Gherkin, ensemble fermé des raisons et FR-001 à FR-030.
  - Non vérifié : aucune politique hôte réelle n'est lue à ce stade.
  - Code supprimé ou évité : aucun parser ou dépendance de configuration ajouté.
  - Complexité ajoutée et justification : aucune complexité runtime.

### T003 - Rejeu de l'audit de réutilisation

- **Statut**: complétée
- **Commit**: `docs(065): Rejouer l'audit de réutilisation`
- **Fichiers**: `specs/065-registre-identite-projets/reuse-audit.md`,
  `specs/065-registre-identite-projets/tasks.md`,
  `specs/065-registre-identite-projets/implementation.md`
- **Preuve**: worktree rebased sur `main` à `a63cf97`; le delta après sa création
  est uniquement documentaire/UI. Les porteurs SPEC-064/068 sont présents et
  aucun registre de projet, audit ou contrat `project_registry_v1` n'existe déjà.
- **Self-review Article XIX/XX**:
  - Pourquoi cette solution est nécessaire : elle évite de créer un type ou un transport déjà introduit par une livraison récente.
  - Pourquoi elle est simple : audit par symboles et chemins stables, sans outil ni dépendance supplémentaire.
  - Hypothèses : toute nouvelle avancée de `main` imposera un nouveau rejeu avant une abstraction supplémentaire.
  - Vérifications réalisées : rebase, delta `main`, inventaire SPEC-064/068 et recherche d'équivalents 065.
  - Non vérifié : les comportements restent à prouver par les tests T004-T006.
  - Code supprimé ou évité : aucun crate, daemon, transport parallèle ou registre concurrent.
  - Complexité ajoutée et justification : aucune complexité runtime.

### T004 - Tests de contrat du registre local

- **Statut**: complétée
- **Commit**: `feat(065): Ajouter le contrat registre projet`
- **Fichiers**: `crates/bridget-transport/src/protocol.rs`,
  `crates/bridget-daemon/src/daemon.rs`, `specs/065-registre-identite-projets/tasks.md`,
  `specs/065-registre-identite-projets/contracts/project-registry-v1.md`,
  `specs/065-registre-identite-projets/implementation.md`
- **Preuve**: le test de protocole couvre la direction Maicie vers Bridget,
  la négociation `service` + `maicie` + `project_registry_v1`, le refus
  fermé d'une version inconnue, la forme filaire du refus `peer_uid_mismatch`,
  et l'outcome de collision avec identité et génération gagnantes. Le champ
  UID n'est pas porté par le JSON : la preuve socket correspondante sera
  ajoutée avec T014.
- **Vérifications réalisées**: `/home/moi/.cargo/bin/cargo fmt --check`,
  `/home/moi/.cargo/bin/cargo test -p bridget-transport` - 219 passants,
  1 ignoré connu, et `/home/moi/.cargo/bin/cargo check --workspace` vert.

### T007 - Variantes publiques et raisons fermées

- **Statut**: complétée
- **Commit**: `feat(065): Ajouter le contrat registre projet`
- **Preuve**: `ProjectRegistryRequest` et `ProjectRegistryOutcome` sont deux
  variantes dédiées, distinctes de `ServiceRequest`. `ProjectBindRequest` et
  `ProjectBindOutcome` refusent les champs inconnus; les 22 raisons filaires
  sont un enum fermé. Les fixtures transport existantes restent lisibles par
  l'exécution complète des tests du crate.
- **Self-review Article XIX/XX**:
  - Pourquoi cette solution est nécessaire : séparer le registre local du guichet à sens inverse.
  - Pourquoi elle est simple : une capability du rôle Service déjà existant, sans socket, crate ou daemon parallèle.
  - Hypothèses : la persistance et SO_PEERCRED restent exclusivement T014.
  - Vérifications réalisées : roundtrip request/outcome, fermeture Serde, compatibilité transport et compilation workspace.
  - Code supprimé ou évité : aucun `ServiceRequest` détourné, aucune capacité implicite.
  - Complexité ajoutée et justification : une enum de capability et deux enveloppes versionnées, nécessaires aux trois consommateurs futurs.

### T005 - Tests de transitions ProjectIdentity

- **Statut**: complétée
- **Commit**: `feat(065): Ajouter identité projet`
- **Preuve**: le test construit une identité `pending_binding`, vérifie le
  refus de référence, fixe un `registration_conflict`, active l'identité
  gagnante avec la génération 4, puis vérifie qu'une désactivation interdit
  toute nouvelle référence ou réactivation implicite.
- **Vérifications réalisées**: `/home/moi/.cargo/bin/cargo test -p maicie --lib`
  - 73 passants, 1 ignoré connu - et `/home/moi/.cargo/bin/cargo check --workspace` vert.

### T008 - ProjectIdentity et ProjectReference

- **Statut**: complétée
- **Commit**: `feat(065): Ajouter identité projet`
- **Fichiers**: `plugins/maicie/src/domain.rs`,
  `specs/065-registre-identite-projets/data-model.md`,
  `specs/065-registre-identite-projets/tasks.md`,
  `specs/065-registre-identite-projets/implementation.md`
- **Preuve**: `ProjectIdentity` ne porte que l'identifiant, le nom métier,
  l'état, les dates et la commande initiale. `ProjectReference` porte seulement
  `project_id` et `binding_generation`; aucune racine, aucun backend et aucun
  état runtime privé n'entrent dans Maicie.
- **Self-review Article XIX/XX**:
  - Pourquoi cette solution est nécessaire : l'activation métier doit être distincte de la liaison hôte.
  - Pourquoi elle est simple : deux types Serde fermés et trois transitions explicites.
  - Hypothèses : la génération provient toujours d'une issue Bridget attestée, jamais d'une déduction Maicie.
  - Vérifications réalisées : test de toutes les transitions demandées, tests unitaires Maicie et compilation workspace.
  - Code supprimé ou évité : aucun chemin hôte, parser de politique ou copie de store Bridget.
  - Complexité ajoutée et justification : O(1) par transition, nécessaire pour interdire les références prématurées.

### T006 - Tests de transitions ProjectBinding

- **Statut**: complétée
- **Commit**: `feat(065): Ajouter les liaisons projet`
- **Preuve**: le test crée une liaison active de génération 1, constate
  `path_missing` sans perte d'identité, puis effectue un rebind explicite vers
  une autre racine avec génération 2. Il vérifie également le rejeu d'un audit
  déterministe, sans chemin en clair.
- **Vérifications réalisées**: `/home/moi/.cargo/bin/cargo test -p bridget-daemon --lib`
  - 652 passants, 7 ignorés connus - et `/home/moi/.cargo/bin/cargo check --workspace` vert.

### T009 - Persistance des liaisons et audit

- **Statut**: complétée
- **Commit**: `feat(065): Ajouter les liaisons projet`
- **Fichiers**: `crates/bridget-daemon/src/store.rs`,
  `specs/065-registre-identite-projets/tasks.md`,
  `specs/065-registre-identite-projets/implementation.md`
- **Preuve**: la migration est additive (`CREATE TABLE IF NOT EXISTS`) et
  ajoute l'index unique des racines non désactivées, l'index chronologique des
  audits et l'unicité `(command_id, operation, binding_generation)`. L'identifiant
  d'audit et la référence de racine sont des SHA-256 déterministes; la référence
  ne contient pas le chemin. La stratégie de permissions du store existant est
  inchangée.
- **Self-review Article XIX/XX**:
  - Pourquoi cette solution est nécessaire : Bridget doit posséder la racine et l'historique technique sans les confier à Maicie.
  - Pourquoi elle est simple : deux tables additives dans le store SQLite existant, sans base ni service parallèle.
  - Hypothèses : la canonicalisation, la politique hôte et l'écriture auditée dans la même transaction de commande arrivent avec T014 et T021.
  - Vérifications réalisées : transition persistée, génération, audit idempotent, absence de chemin auditée, tests daemon complets et compilation workspace.
  - Non vérifié : Clippy global est bloqué par deux avertissements préexistants de `bridget-transport` hors SPEC-065.
  - Code supprimé ou évité : aucun chemin brut dans l'audit, aucune migration destructive, aucune modification de droits existants.
  - Complexité ajoutée et justification : index O(log n) et empreintes SHA-256, nécessaires à l'unicité et à la non-divulgation.

### T010 - Tests SQLite de préparation d'enregistrement

- **Statut**: complétée
- **Commit**: `feat(065): Préparer les enregistrements projet`
- **Fichiers**: `plugins/maicie/src/store.rs`,
  `plugins/maicie/tests/project_registration_integration.rs`,
  `specs/065-registre-identite-projets/tasks.md`,
  `specs/065-registre-identite-projets/implementation.md`
- **Preuve**: le test crée une identité `pending_binding`, sa commande et son
  outbox dans la même transaction. Le rejeu identique retourne la même issue,
  les octets divergents retournent `EnvelopeMismatch`, et deux threads créent
  deux intentions concurrentes pour une même racine demandée sans activer
  aucune identité.
- **Vérifications réalisées**: `/home/moi/.cargo/bin/cargo test -p maicie --test project_registration_integration`
  - 1 passant - et `/home/moi/.cargo/bin/cargo test -p maicie --lib -- --test-threads=1`
  - 73 passants, 1 ignoré connu. La variante parallèle de la suite lib est
  instable dans deux tests `install_publish` préexistants (`Text file busy`),
  alors que chacun passe isolément et en exécution séquentielle.
- **État de T012**: entamée seulement par la migration v22 et la préparation
  atomique. La promotion après issue Bridget, l'issue observable par
  `command_id` et la reprise productive restent ouvertes.

### T011 - Tests et chargement de la politique de racines

- **Statut**: complétée
- **Fichiers**: `crates/bridget-daemon/src/project_policy.rs`,
  `crates/bridget-daemon/src/lib.rs`,
  `specs/065-registre-identite-projets/tasks.md`,
  `specs/065-registre-identite-projets/implementation.md`
- **Preuve**: cinq tests ciblés couvrent l'absence du document, les schémas
  vide, inconnu et invalide, les frontières `/`, `/home`, `/Users` et le home
  du daemon, le propriétaire et les permissions modifiables par groupe, la
  canonicalisation, les préfixes permis et refusés, les doublons canoniques,
  ainsi que les liens symboliques du document et des racines candidates. Le
  chargeur n'accepte ni chemin implicite ni valeur par défaut.
- **Vérifications réalisées**: `/home/moi/.cargo/bin/cargo fmt --check`,
  `/home/moi/.cargo/bin/cargo test -p bridget-daemon project_policy --lib`
  - 5 passants - et `/home/moi/.cargo/bin/cargo check -p bridget-daemon` vert.
- **Self-review Article XIX/XX**:
  - Pourquoi cette solution est nécessaire : la liaison à une racine hôte doit être refusée avant toute écriture si la frontière de confiance est douteuse.
  - Pourquoi elle est simple : un seul chargeur fermé, appelé explicitement, et une liste de chemins canoniques conservée en mémoire.
  - Hypothèses : l'UID du daemon est l'autorité locale; le chargement unique au démarrage sera raccordé à `DaemonConfig` par T014.
  - Vérifications réalisées : lecture sans suivre de lien, vérification UID/mode, canonicalisation des deux côtés et cinq tests unitaires ciblés.
  - Non vérifié : l'injection du chemin explicite dans le daemon et l'usage avant mutation relèvent de T014.
  - Code supprimé ou évité : aucune valeur de repli, lecture de `HOME`, dépendance ou parseur de politique runtime ajouté.
  - Complexité ajoutée et justification : une liste triée de racines et des vérifications O(n), nécessaires seulement aux mutations d'enregistrement.

### T012 - Saga durable de résolution Bridget

- **Statut**: complétée
- **Fichiers**: `plugins/maicie/src/store.rs`,
  `plugins/maicie/tests/project_registration_integration.rs`,
  `specs/065-registre-identite-projets/tasks.md`,
  `specs/065-registre-identite-projets/implementation.md`
- **Preuve**: l'intention, l'identité `pending_binding` et l'outbox restent
  atomiques. L'issue Bridget est désormais sérialisée avec son horodatage et
  relisible par `command_id` après réouverture SQLite. Une issue `active`
  active l'identité et fixe son propre `resolved_project_id`; une collision
  fige l'identité proposée en `registration_conflict` et conserve l'identifiant
  opaque gagnant, même s'il n'est pas présent dans le store Maicie; un refus de
  liaison laisse l'identité non active. Un rejeu octet pour octet relit le même
  résultat et une issue divergente est refusée.
- **Vérifications réalisées**: `/home/moi/.cargo/bin/cargo fmt --check`,
  `/home/moi/.cargo/bin/cargo test -p maicie --test project_registration_integration`
  - 2 passants -, `/home/moi/.cargo/bin/cargo test -p maicie --lib -- --test-threads=1`
  - 73 passants, 1 ignoré connu - et `/home/moi/.cargo/bin/cargo check -p maicie` vert.
- **Self-review Article XIX/XX**:
  - Pourquoi cette solution est nécessaire : aucune présence d'outbox ne doit être confondue avec une liaison réussie; seule une issue Bridget attestée peut activer une identité.
  - Pourquoi elle est simple : les octets de l'issue et son horodatage sont ajoutés à la commande durable existante, sans coordinateur, cache ou base partagée.
  - Hypothèses : une issue `binding_failed` est terminale pour cette commande; la reprise de transport avant issue relève de T013 et T016.
  - Vérifications réalisées : test de succès, collision externe, refus, rejeu identique, issue divergente et lecture après réouverture SQLite.
  - Non vérifié : l'envoi réel vers Bridget et les crashs entre transport et accusé seront couverts par T013, T015 et T016.
  - Code supprimé ou évité : aucun accès Maicie au store Bridget, aucune identité fictive pour le projet gagnant externe, aucun mécanisme de promotion déduit d'une outbox.
  - Complexité ajoutée et justification : deux colonnes d'issue et une transition transactionnelle O(1), indispensables pour la saga rejouable.

### T013 - Commande locale et reprise

- **Statut**: complétée
- **Commit**: `feat(065): Finaliser l'enregistrement projet`
- **Fichiers**: `plugins/maicie/src/main.rs`, `plugins/maicie/src/app.rs`,
  `plugins/maicie/src/store.rs`, `plugins/maicie/tests/project_registration_integration.rs`
- **Preuve**: `maicie project register` prépare l'intention et la requête
  versionnée avant toute I/O, puis appelle la variante dédiée. `maicie project
  resume --command-id` relit exclusivement les octets d'outbox déjà persistés.
  Une issue terminale est seulement relue, jamais redemandée. Le test
  d'intégration vérifie les mêmes octets, `backend=host` et l'activation après
  une issue active.
- **Vérifications réalisées**: `/home/moi/.cargo/bin/cargo test -p maicie --test project_registration_integration`
  - 3 passants - et `/home/moi/.cargo/bin/cargo test -p maicie --bin maicie`
  - 24 passants.

### T014 - Admission Bridget, UID et transaction de liaison

- **Statut**: complétée
- **Commit**: `feat(065): Finaliser l'enregistrement projet`
- **Fichiers**: `crates/bridget-daemon/src/cli.rs`,
  `crates/bridget-daemon/src/daemon.rs`, `crates/bridget-daemon/src/store.rs`,
  `crates/bridget-daemon/src/project_policy.rs`,
  `crates/bridget-daemon/src/managed_supervisor.rs`
- **Preuve**: le chemin `--project-root-policy` doit être absolu et est chargé
  une fois par le daemon. L'admission vérifie rôle Service, capability dédiée,
  UID pair reçu par la socket Unix, version, échéance et racine canonique avant
  une transaction SQLite unique qui conserve l'issue par `command_id` et écrit
  exactement un audit pour la liaison créée. Une collision retourne l'identité
  et la génération déjà liées sans exposer la racine.
- **Vérifications réalisées**: `/home/moi/.cargo/bin/cargo test -p bridget-daemon --lib spec_065_`
  - 10 passants. La lecture réelle de l'UID par `SO_PEERCRED` est couverte par
  une paire de sockets Unix locale.

### T015 - Client Maicie du registre dédié

- **Statut**: complétée
- **Commit**: `feat(065): Finaliser l'enregistrement projet`
- **Fichiers**: `plugins/maicie/src/bridget_client.rs`
- **Preuve**: `ProjectRegistryClient` négocie uniquement
  `project_registry_v1` sur le rôle Service, injecte sans réécriture les octets
  du `ProjectBindRequest` durable dans `project_registry_request`, puis corrèle
  l'issue retournée au `command_id`. Il ne possède aucun accès au store Bridget.
- **Vérifications réalisées**: le test socket du client contrôle la négociation,
  la trame exacte et l'issue `active`; il passe dans les 74 tests unitaires
  Maicie (1 ignoré connu).

### T016 - Reprise croisée et collision canonique

- **Statut**: complétée
- **Commit**: `test(065): Prouver la reprise du registre projet`
- **Fichiers**: `crates/bridget-daemon/tests/project_registration_e2e.rs`,
  `crates/bridget-daemon/tests/integration_test.rs`,
  `crates/bridget-daemon/tests/sc005_attach_budget.rs`
- **Preuve**: le test de bout en bout est placé côté daemon, qui possède déjà
  Maicie en dépendance de test - l'inverse créerait un cycle de crates. Il
  prépare une intention Maicie, simule un crash après l'outbox puis un autre
  après la persistance Bridget avant résolution Maicie, redémarre le store et
  rejoue les octets exacts. Une seconde commande passant par un lien symbolique
  reçoit la collision durable; une seule identité Maicie est active.
- **Vérifications réalisées**: `/home/moi/.cargo/bin/cargo test -p bridget-daemon --test project_registration_e2e`
  - 1 passant. Les deux initialiseurs de configuration d'intégration conservés
  par les suites historiques fournissent explicitement une politique absente,
  donc restent compatibles avec les lancements non enregistrés.

### T017 - Preuve US1

- **Statut**: complétée
- **Fichier**: `specs/065-registre-identite-projets/evidence/us1-register-replay.md`
- **Preuve**: le parcours complet, les deux commandes, le contrat v1, la
  capability, les identités gagnante/perdante et les comptes de lignes y sont
  consignés à partir des tests exécutés.

### T018 et T020 - Administration locale et rapprochement explicite

- **Statut**: complétées
- **Fichiers**: `crates/bridget-daemon/tests/project_registration_e2e.rs`,
  `plugins/maicie/src/main.rs`, `plugins/maicie/src/app.rs`,
  `plugins/maicie/src/bridget_client.rs`, `plugins/maicie/src/store.rs`
- **Preuve**: le test de bout en bout appelle `list`, `status`, `rebind`,
  `disable` et le rapprochement explicite à travers la socket locale. La
  commande `review-project reconcile` exige exactement `--dry-run` ou
  `--confirm`; la prévisualisation est pure et la confirmation utilise une
  requête versionnée dédiée. La sortie JSON porte une `next_action` structurée.
  Le test est côté daemon parce qu'il détient déjà Maicie comme dépendance de
  test: le déplacer dans Maicie créerait un cycle de crates.
- **Vérifications réalisées**: `/home/moi/.cargo/bin/cargo test -p bridget-daemon --test project_registration_e2e`
  - 1 passant - et `/home/moi/.cargo/bin/cargo test -p maicie --bin maicie -- --test-threads=1`
  - 25 passants.

### T021 - Mutations Bridget

- **Statut**: complétée
- **Fichiers**: `crates/bridget-daemon/src/daemon.rs`,
  `crates/bridget-daemon/src/store.rs`, `crates/bridget-transport/src/protocol.rs`
- **Preuve obtenue**: rebind, disable et rapprochement sont transactionnels,
  idempotents par `command_id`, précédés de la validation de policy et UID,
  et n'invoquent aucun arrêt de processus. Les audits sont écrits avec la
  mutation effective ou la confirmation de rapprochement.
- **Preuve finale**: une exécution active rattachée à `ProjectReference`
  conserve sa génération précédente pendant un rebind. Elle est fournie par
  `project_binding_integration_test` après la propagation US3.

### T022 - Projection de fraîcheur

- **Statut**: complétée
- **Fichiers**: `plugins/maicie/src/runtime.rs`,
  `plugins/maicie/src/ui_projection.rs`
- **Preuve**: une observation Bridget versionnée rend séparément l'état métier
  Maicie, l'état technique Bridget, la fraîcheur et l'action suivante. Sans
  lecture Bridget, la projection affiche explicitement `unavailable`, sans
  racine hôte et sans déduire `active`.
- **Vérifications réalisées**: `/home/moi/.cargo/bin/cargo test -p maicie --lib -- --test-threads=1`
  - 76 passants, 1 ignoré connu.

### T023 - Non-destruction et confidentialité de l'audit

- **Statut**: complétée
- **Fichier**: `specs/065-registre-identite-projets/evidence/us2-safety.md`
- **Preuve**: la sentinelle du dépôt temporaire est strictement identique avant
  et après `rebind` et `disable`. Les quatre audits attendus sont uniques et
  ne contiennent aucune racine canonique. Le résultat exact est consigné dans
  la preuve US2.

### Self-review Article XIX/XX - lot US2 intermédiaire

- Pourquoi cette solution est nécessaire : l'opérateur doit administrer une
  liaison sans accéder au store Bridget ni modifier un dépôt.
- Pourquoi elle est simple : une variante versionnée sur le canal Service
  existant, une table d'idempotence et une projection publique réduite.
- Hypothèses : le lien entre une exécution active et sa génération sera porté
  par `ProjectReference` en US3; aucune inférence de projet depuis un chemin
  n'est faite avant cette tâche.
- Vérifications réalisées : e2e socket local, rejeux exacts, audit unique,
  sentinelle inchangée, tests CLI et tests de projection.
- Non vérifié : maintien d'une exécution active sur son ancienne génération,
  explicitement laissé ouvert en T021.
- Code supprimé ou évité : aucune API de base Bridget, aucun socket ou daemon
  supplémentaire, aucun accès direct au contenu du dépôt.
- Complexité ajoutée et justification : recherche de liaison indexée O(log n),
  liste O(n) pour un affichage explicite, et une écriture transactionnelle O(1)
  par mutation.

### T019 et T021 - Audit et générations actives

- **Statut**: complétées
- **Commit**: `feat(065): Administrer les liaisons projet`
- **Fichiers**: `crates/bridget-daemon/src/store.rs`,
  `crates/bridget-daemon/src/daemon.rs`,
  `crates/bridget-daemon/tests/project_binding_integration_test.rs`
- **Preuve**: le test de liaison crée un projet, démarre une réservation de
  flotte en génération 1, effectue un rebind vers la génération 2 et vérifie
  que la réservation reste nommée par la génération 1. Le rejeu exact du
  rebind ne crée aucun audit supplémentaire. Les deux mutations effectives
  produisent donc exactement les audits `register` puis `rebind`.
- **Vérification réalisée**:
  `/home/moi/.cargo/bin/cargo test -p bridget-daemon --test project_binding_integration_test`
  - 1 passant.

### T024 à T028 - Corrélation durable des projets

- **Statut**: complétées
- **Commit**: `feat(065): Corréler les exécutions aux projets`
- **Fichiers**: `crates/bridget-transport/src/protocol.rs`,
  `crates/bridget-daemon/src/{fleet.rs,desired_state.rs,execution_store.rs,idempotency.rs,lifecycle.rs,daemon.rs,ui.rs}`,
  `crates/bridget-daemon/assets/ui/app.js`,
  `plugins/maicie/src/{domain.rs,app.rs,store.rs,bridget_client.rs,ui_projection.rs}`.
- **Preuve**: `ProjectReference` est optionnelle et porte un couple immuable
  `project_id` et `binding_generation`. Les anciens enregistrements se
  désérialisent à `None`. Les migrations SQLite ajoutent les colonnes sans
  écraser l'historique, y compris une migration idempotence v6 qui ne collisionne
  pas avec la migration v5 déjà utilisée par un autre lot. Les snapshots,
  curseurs, événements de liens et faits runtime délégués gardent la même
  référence.
- **Cwd**: le lancement lié valide la racine canonique ou un worktree Git qui
  partage son répertoire Git commun. Un voisin de la racine est refusé avant
  toute réservation provider.
- **Projection**: la vue affiche `project_id`, `domain` et l'état
  `registered` ou `unregistered` comme trois données distinctes. Aucun contrôle
  de sécurité ne dépend de `domain`.
- **Vérifications réalisées**:
  - `cargo test -p bridget-daemon lifecycle::tests::cwd_projet_accepte_racine_descendante_et_worktree_lie_mais_refuse_un_voisin --lib` - 1 passant.
  - `cargo test -p bridget-daemon --test execution_store_test reference_projet_du_snapshot_survit_au_redemarrage_sans_reduction` - 1 passant.
  - `cargo test -p bridget-daemon migration_v6_ajoute_les_references_projet_apres_une_base_deja_en_v5 --lib` - 1 passant.
  - `cargo test -p bridget-daemon spec_068_faits_runtime_delegues_restent_ordonnes_et_accuses --lib` - 1 passant.
  - `cargo test -p bridget-daemon fleet::tests::reprise_expose_les_generations_en_vol_dans_l_ordre_des_noms --lib` - 1 passant.
  - `cargo test -p maicie --test contract execution_projection -- --test-threads=1` - 8 passants.
  - `cargo test -p maicie project_correlation_tests --lib -- --test-threads=1` - 2 passants.

### T029 - Validation finale

- **Statut**: exécutée, baseline globale non verte documentée.
- **Preuve**: `cargo fmt --check` réussit. Les tests ciblés de SPEC-065
  réussissent et `cargo test --workspace --no-run` réussit. La validation
  complète, lancée avec un seul thread par binaire pour limiter la contention,
  s'arrête sur trois tests existants de `managed_parity_test` liés à MCP, hors
  fichiers et comportements SPEC-065. Clippy échoue aussi sur deux diagnostics
  préexistants dans `bridget-transport`.
- **Détail**: `specs/065-registre-identite-projets/evidence/final-validation.md`.

### T030 - Relecture finale

- **Statut**: complétée
- **Preuve**: les chemins et symlinks ont un refus fail-closed, le rebind ne
  modifie pas les exécutions actives, les migrations sont additives et les
  références historiques restent absentes plutôt que déduites. Aucun document
  de SPEC-066 ou SPEC-067 n'a été modifié. L'absence de `.specify` a été
  respectée sans installation ni synchronisation.

### Self-review Article XIX/XX - clôture SPEC-065

- Pourquoi cette solution est nécessaire : une exécution ne doit pas changer
  rétroactivement de projet quand sa liaison est rebindée.
- Pourquoi elle est simple : une référence optionnelle unique traverse les
  contrats existants; elle ne crée ni registre parallèle ni nouveau daemon.
- Hypothèses : l'alimentation automatique de `ProjectReference` par des
  environnements isolés reste explicitement le travail de SPEC-066.
- Vérifié : persistance, redémarrage, curseur, événements délégués, worktree
  Git lié, refus du voisin, projection UI et compatibilité historique.
- Risque résiduel : la suite globale possède trois échecs MCP hors lot et deux
  diagnostics Clippy préexistants, détaillés dans la preuve finale.
