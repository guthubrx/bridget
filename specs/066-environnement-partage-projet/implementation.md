### T021, T023 et T024 - Admission et corrélation Docker

- Statut: terminées.
- Preuve: le wrapper reçoit le socket fixe `/run/bridget/runtime/bridget.sock` et présente un handshake complet avant Register. Après admission, le daemon persiste l'identité de l'exec, de l'instance, de la génération, du fournisseur, du conteneur et de l'epoch avant `docker exec`; la reprise relit cette corrélation et ne crée pas de second exec.
- Vérification: `cargo test -p bridget-daemon --test project_runtime_integration_test`: 2 passed; `cargo test -p bridget-daemon spec_066_ --lib`: 13 passed.

### T017 - Ingress privé et reprise

- Statut: terminée.
- Preuve: `crates/bridget-daemon/tests/project_runtime_ingress_test.rs` vérifie deux sockets distinctes, la reconstruction au même chemin après reprise, le refus A vers B et les falsifications d'instance, génération et epoch. Les oracles daemon existants vérifient en plus la consommation unique à l'admission et la reconnexion durable sans second exec; les oracles SPEC-068 vérifient persistance, rejeu ordonné et accusé.
- Vérification: `cargo test -p bridget-daemon --test project_runtime_ingress_test`: 1 passed; `cargo test -p bridget-daemon spec_066_ --lib`: 13 passed.

### T019 - Agents isolés dans un conteneur partagé

- Statut: terminée.
- Preuve: `crates/bridget-daemon/tests/project_runtime_agents_test.rs` crée deux environnements Docker réels, lance deux agents dans le conteneur du projet A et vérifie que l'arrêt attesté de l'agent A ne termine pas l'agent B. Le projet B a son propre conteneur et son propre ingress.
- Vérification: `cargo test -p bridget-daemon --test project_runtime_agents_test`: 1 passed; `cargo test -p bridget-daemon --test project_runtime_integration_test`: 2 passed.

### Correction rebind Docker en cours

- La mutation de rebind fait maintenant passer le runtime Docker en `recreate_required` et incrémente son epoch, sans effacer le `container_id`. Une exécution existante n'est donc pas arrêtée implicitement, tandis qu'une nouvelle admission est fermée jusqu'à recréation.
- Vérification ciblée: `cargo test -p bridget-daemon spec_066_rebind_docker --lib`: 1 passed.

# Journal d'implémentation: SPEC-066

**Statut**: En cours
**Dernière mise à jour documentaire**: 2026-08-30
**Tête observée**: main à 74234641fe04293c007717b8f1e3879823f5d3823a72c5b599cb2f8f2e5723cdaea684bf2.

L'implémentation se déroule dans le worktree neuf de la branche
session-066-environnement-partage-projet, créé depuis main à
74234641fe04293c007717b8f1e3879823f5d382. Le worktree documentaire source
reste inchangé. Aucun service de production, projet réel ou cycle de vie Docker
hors tests ne sont activés dans cette tranche en cours.

## Journal des tâches

Aucune entrée.

### T001 - Scénarios Gherkin

- Statut: terminé.
- Preuve: tests/features/066-environnement-partage-projet.feature contient cinq scénarios couvrant préparation, refus sans fallback, partage, isolation et rollback.
- Vérification: git diff --check sans erreur.

### Self-review Article XIX/XX
- Pourquoi cette solution est nécessaire : expliciter les oracles métier avant le runtime Docker.
- Pourquoi elle est plus simple ou plus maintenable que les alternatives raisonnables : un seul fichier Gherkin, aligné sur les parcours de la spec.
- Hypothèses prises : ces scénarios servent de contrat vivant; les tests Rust viennent dans les tâches dédiées.
- Vérifications réalisées : lecture des features 064 et 065, contrôle de format Git.
- Non vérifié : aucun moteur Gherkin automatisé nest câblé dans le workspace actuel.
- Code supprimé ou évité : aucun nouveau runner ni dépendance.
- Complexité ajoutée et justification : documentation exécutable minimale, requise par T001.

### T002 - Préflight de la cible

- Statut: terminé.
- Preuve: specs/066-environnement-partage-projet/evidence/preflight-host.md consigne Docker 29.6.2, uid/gid 1002, groupe docker, cgroup v2, overlayfs, permissions et topologie Git.
- Vérification: commandes Docker read-only, stat, findmnt et git worktree list; aucune opération de cycle de vie Docker.

### Self-review Article XIX/XX
- Pourquoi cette solution est nécessaire : les contraintes machine doivent être prouvées avant le premier runtime fixture.
- Pourquoi elle est plus simple ou plus maintenable que les alternatives raisonnables : une preuve versionnée évite de déduire des droits depuis la seule présence de Docker.
- Hypothèses prises : le parent de state roots sera créé par le préflight fixture avec uid/gid configurés.
- Vérifications réalisées : version, sécurité, socket, permissions, espace disque et common dir Git observés.
- Non vérifié : image fixture, ingress et droits réels dans le conteneur, réservés aux tests US1.
- Code supprimé ou évité : aucune configuration de production ni service auxiliaire.
- Complexité ajoutée et justification : un document de preuve requis par T002.

### T003 - Audit de réutilisation rejoué

- Statut: terminé.
- Preuve: reuse-audit.md est PASS sur 74234641fe04293c007717b8f1e3879823f5d382; les extensions de SPEC-075, les contrats 065 et les incidents 068 sont reliés aux points de réutilisation.
- Vérification: recherche par symbole et responsabilité sur protocole, store, lifecycle, processus géré, wrapper, daemon et dépendances.

### Self-review Article XIX/XX
- Pourquoi cette solution est nécessaire : empêcher un second cycle de vie, store ou socket par accident.
- Pourquoi elle est plus simple ou plus maintenable que les alternatives raisonnables : le rapport lie chaque création au seam existant et à une ligne vérifiable.
- Hypothèses prises : Docker reste absent du code productif sur cette tête.
- Vérifications réalisées : rg ciblé, lecture des types et schémas, contrôle de SPEC-075 intégré.
- Non vérifié : intégration runtime, couverte par les tests puis les tâches US1 à US3.
- Code supprimé ou évité : SDK Docker, second superviseur et cache Bridget monté.
- Complexité ajoutée et justification : aucune abstraction productive.

### T004 et T005 - Machine d état et politique runtime

- Statut: terminées.
- Fichiers: crates/bridget-daemon/src/project_runtime.rs et crates/bridget-daemon/src/lib.rs.
- Preuve: sept tests unitaires passent avec cargo test -p bridget-daemon project_runtime --lib.
- Vérification: transitions fermées, réservation capturant epoch, invalidation policy, images immuables, permissions de policy, ABI HOME/XDG, arguments sans shell et refus de montages interdits.

### Self-review Article XIX/XX
- Pourquoi cette solution est nécessaire : une autorité unique doit porter état, policy et arguments avant intégration daemon.
- Pourquoi elle est plus simple ou plus maintenable que les alternatives raisonnables : un module limité remplace des validations éparses dans daemon et lifecycle.
- Hypothèses prises : le module reste sans accès Docker effectif et sans identité fournisseur.
- Vérifications réalisées : compilation rouge sans module, puis sept tests unitaires verts.
- Non vérifié : create, inspect et exec Docker réels, réservés aux tâches US1 et US2.
- Code supprimé ou évité : aucun SDK Docker, shell, daemon secondaire ou dépendance externe.
- Complexité ajoutée et justification : une machine d état consommée par store, daemon, lifecycle, CLI/UI et tests.

### T006 et T009 - Contrats additifs host et Docker

- Statut: terminées.
- Preuve: ProjectBackend accepte docker; les requêtes peuvent porter policy_id et policy_version sans les rendre obligatoires; les projections et issues exposent une référence de politique uniquement quand elle existe.
- Vérification: cargo test -p bridget-transport passe, dont project_runtime_contract_test pour la relecture des liaisons host historiques et le round-trip Docker.

### T007 - Module runtime exporté

- Statut: terminée.
- Preuve: crates/bridget-daemon/src/project_runtime.rs est exporté par crates/bridget-daemon/src/lib.rs.
- Vérification: cargo test -p bridget-daemon project_runtime --lib: 7 passed, 0 failed.

### T008 - Stockage runtime Docker additif

- Statut: terminée.
- Preuve: project_bindings migre les tables 065 vers le schéma host/docker et persiste état, politique, empreintes, UID/GID, epoch et container_id pour Docker; les lignes host historiques restent sans runtime.
- Vérification: cargo test -p bridget-daemon spec_066_store_migre_hote_et_persiste_runtime_docker --lib: 1 passed, 0 failed.

### Self-review Article XIX/XX - T006 a T009

- Pourquoi cette solution est nécessaire : le runtime Docker doit survivre au redémarrage sans rendre obligatoire une donnée Docker pour les projets host existants.
- Pourquoi elle est plus simple ou plus maintenable que les alternatives raisonnables : une extension additive de ProjectBinding garde le registre 065 comme seule autorité durable au lieu de créer un second store.
- Hypothèses prises : les commandes Docker et les admissions restent a raccorder dans les tâches US1 et US2.
- Vérifications réalisées : format Rust, test complet du transport, sept tests unitaires runtime et test de migration host vers le schéma étendu.
- Non vérifié : Docker réel, ingress privé et docker exec, réservés aux tâches suivantes.
- Code supprimé ou évité : aucun SDK Docker, shell, stockage de secret ou fallback host.
- Complexité ajoutée et justification : dix colonnes optionnelles et une migration unique pour préserver les données antérieures tout en attestant les environnements Docker.

### T010 et T013 - Préparation Docker bornée

- Statut: terminées.
- Preuve: le faux binaire Docker couvre timeout, JSON hostile, image absente et daemon indisponible. Le runtime construit create/start/inspect sans shell, avec image immuable, rootfs lecture seule, capabilities retirées, limites, HOME/XDG et montages fermés.
- Vérification: cargo test -p bridget-daemon docker_fixture_covers_timeout_hostile_json_image_and_daemon_failure --lib: 1 passed, 0 failed.

### T015 - Commande locale et projection UI

- Statut: terminée.
- Preuve: bridget project-runtime prepare, status ou recreate avec --project utilise une requête locale versionnée. Le daemon vérifie l UID pair et ne reçoit aucune option Docker libre. Le relais UI expose GET /v1/projects/runtime?project=<id>, protégé par son jeton, sans exposer de racine hôte ni identifiant complet de conteneur.
- Vérification: cargo test -p bridget-daemon spec_066_cli_runtime_projet --lib: 1 passed, 0 failed; cargo test -p bridget-daemon spec_066_daemon_runtime_status --lib: 1 passed, 0 failed; cargo test -p bridget-transport spec_066_runtime_local -q: 1 passed, 0 failed.

### T016 - Recréation non destructive

- Statut: terminée.
- Preuve: le test Docker réel prépare, atteste, supprime et recrée le conteneur fixture. Les empreintes de code et de state root restent identiques.
- Vérification: cargo test -p bridget-daemon --test project_runtime_integration_test -q: 1 passed, 0 failed. Aucun conteneur de test portant le label project-066 ne subsiste après le test.

### T018 - Worktrees du même projet

- Statut: terminée.
- Preuve: resolve_project_mounts ne consulte que git worktree list --porcelain du projet, exige le même common dir et refuse les chemins ambigus. Les montages gardent leur chemin absolu dans le conteneur.
- Vérification: cargo test -p bridget-daemon --test project_runtime_mounts_test -q: 1 passed, 0 failed.

### T011 - Intégration Docker réelle et T012 - Préflight fermé

- Statut: terminées.
- Preuve: l'image fixture est construite par `docker build --iidfile`, puis le
  test vérifie dans un vrai conteneur l'image ID immuable, les labels, UID/GID,
  HOME/XDG, les limites, le réseau bridge, les montages, l'écriture du state
  root, la lecture d'une fixture `0600` et le socket Unix explicite.
- Préflight: la politique est chargée depuis un chemin absolu configuré au
  démarrage; Docker version et image sont bornés, et l'UID/GID doivent égaler
  l'identité propriétaire du state root privé.
- Vérification: `cargo test -p bridget-daemon --test project_runtime_integration_test -q`: 1 passed, 0 failed; `cargo test -p bridget-daemon preflight_refuse_un_uid_ou_gid_incompatible_avec_le_state_root_prive -q`: 1 passed, 0 failed.

### T033 - Image fixture reproductible

- Statut: terminée.
- Fichiers: `infra/project-runtime/Dockerfile` et `infra/project-runtime/README.md`.
- Preuve: `docker build --iidfile /tmp/bridget-066-runtime-fixture.iid infra/project-runtime` a produit `sha256:e68018fad5c762960807ece7ff41142a9be97a7cb84b9dc261345bdda5852eb1`; le test consomme cet ID local, jamais un tag.

### T014 - Divergence attestée

- Statut: terminée.
- Preuve: une divergence d attestation fait passer l environnement en recreate_required, état qui est persisté par le pilote daemon.
- Vérification: cargo test -p bridget-daemon attestation_divergente_impose_recreate_required --lib -q: 1 passed, 0 failed.
### Self-review Article XIX/XX - état intermédiaire

- Pourquoi cette solution est nécessaire: elle établit un backend Docker fermé et observable avant de brancher le lancement partagé des agents.
- Ce qui reste explicitement hors périmètre livré: docker exec réel, corrélation flotte, réconciliation et rollback avec agents actifs.
- Code évité: SDK Docker, shell, Docker socket monté, privilèges et fallback host implicite.

### Avancement partiel US2 - ingress fermé

- Un socket Unix privé déterministe par projet/génération est créé sous le
  state root, avec répertoire `0700` et socket `0600`, et est monté seulement
  dans le conteneur concerné lors de `prepare`.
- Le contrat versionné de handshake est présent. Le conteneur reçoit explicitement
  la variable BRIDGET_RUNTIME_SOCKET; le wrapper la privilégie sans dérivation depuis HOME.
- Le daemon contrôle le frame reçu contre une réservation à usage unique, la
  génération, l epoch, le conteneur et l UID obtenu du noyau, puis répond avant
  tout Register. Sans réservation ou en cas de divergence, aucune inscription
  ne commence.
- T017 et T021 restent ouvertes: la réservation doit encore être créée et
  revérifiée par le futur docker exec et les scénarios indépendants de reprise
  et d incidents runtime doivent être exécutés.

### Contrat d exécutables internes - résolu avant T023

La politique runtime porte désormais un manifeste fermé de lanceur Bridget et de
commandes fournisseurs présentes dans l image. Ce mapping est validé, digesté
et résolu sans consulter les chemins de définitions de l hôte. Le contrat est
ainsi disponible avant le câblage de docker exec, qui doit encore réserver la
génération puis confirmer l admission sur l ingress privé.

### Correction de sécurité - absence de repli host (2026-08-30)

La policy runtime comporte maintenant une autorité optionnelle et fermée pour le
lanceur Bridget interne et les commandes fournisseurs internes. Les chemins du
registre hôte restent inutilisables dans Docker. Tant que docker exec avec
réservation et handshake reste absent, SpawnOrder, relaunch et recovery refusent
explicitement les projets Docker avec docker_runtime_unavailable. Aucun processus
agent ne démarre sur hôte.

Preuves ciblées: le contrat de transport sérialise ce refus, la résolution de
policy refuse un type non déclaré, et le test daemon vérifie que Docker prend ce
refus tandis que host ne le prend pas.

### Limite constatée avant T023

L image fixture de SPEC-066 atteste l isolation de base mais ne contient ni le
lanceur Bridget interne ni un exécutable fournisseur. De plus, le superviseur
host actuel dépend de deux canaux de bootstrap privés qui ne peuvent pas être
hérités par docker exec. Un lancement Docker réel exige donc une voie de
supervision dédiée: créer la réservation, lancer docker exec, attendre le
handshake, suivre le processus Docker et raccorder stop, reprise et incidents.
Cette voie n est pas simulée ni remplacée par un lancement host.

## Clôture de SPEC-066 - 2026-08-30

Statut final : terminé. Les 36 tâches sont cochées après preuve.

- Le backend Docker est activable projet par projet, reste fail-closed et ne bascule jamais implicitement vers host.
- Les agents du même projet réutilisent l'environnement attesté; l'ingress privé vérifie projet, générations, conteneur et epoch avant Register.
- Stop, suppression et bascule explicite vers host refusent les agents actifs; la bascule invalide les réservations Docker sans toucher au dépôt ni au state root.
- Les sorties runtime inattendues conservent une raison fermée; l'interface locale expose cette dernière raison sans révéler les détails Docker bruts.
- Les scénarios OOM, limite PID, exec perdu et redémarrage Docker sont classés par raisons stables et sont couverts par test.

Validation finale : format, Clippy workspace et tests workspace sont verts. Les preuves détaillées sont dans `evidence/final-validation.md`, `evidence/us2-shared-agents.md` et `evidence/us3-rollback.md`.

Aucun projet réel, provider réel, secret ni service de production n'a été activé durant l'implémentation.
