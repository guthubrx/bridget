# Plan d'implémentation: Environnement Docker partagé par projet

**Branche**: `session-066-environnement-partage-projet`
**Spec**: `specs/066-environnement-partage-projet/spec.md`
**Date**: 2026-08-29
**Statut du plan**: proposé, non implémenté

## Résumé

Bridget reçoit un second backend de lancement. Le backend `host` conserve le
chemin actuel. Le backend `docker` gère un conteneur persistant par projet et y
lance plusieurs wrappers au moyen d'exécutions distinctes. Le code, les
worktrees et l'état durable restent montés depuis l'hôte. Un point d'entrée Unix
dédié évite de monter tout le cache privé Bridget.

La tranche utilise Docker CLI via `std::process::Command`, car le projet ne
possède pas de client Docker et une dépendance API supplémentaire augmenterait
la surface sans besoin prouvé. Les commandes sont construites en arguments,
jamais par shell.

## Contexte technique

| Élément | Choix |
|---|---|
| Hôte | Linux, Bridget et Maicie hors conteneur |
| Moteur | Docker Engine local via CLI |
| Granularité | un conteneur par project_id |
| Processus agent | un `docker exec` suivi par génération |
| Source | bind mount rw au même chemin absolu |
| État projet | répertoire hôte privé monté rw |
| Root filesystem | read-only, tmpfs explicites |
| Réseau | bridge sortant, aucun port publié, pas de host network |
| Secrets | aucun dans cette tranche |
| Image | référence par digest |
| Source de politique | document hôte Bridget v1 fourni par chemin absolu |
| Utilisateur | UID/GID numériques non-root, vérifiés au préflight |
| HOME conteneur | `/var/lib/bridget-project/home`, sous le state root projet |
| Socket wrapper | `/run/bridget/runtime/bridget.sock`, fourni explicitement |
| Compatibilité | enum host/docker, host inchangé |

## Architecture cible

```text
HÔTE
  Maicie CLI + SQLite
  Bridget daemon + stores + registre projet
       |
       +-- backend host --> ManagedLaunch existant
       |
       +-- backend docker --> ProjectEnvironment
              |
              +-- conteneur unique project-A
              |      +-- agent Codex, cwd worktree-1
              |      +-- agent Claude, cwd worktree-2
              |      +-- agent Cursor, cwd project root
              |
              +-- conteneur unique project-B

  RuntimeIngress Unix dédié <---- wrappers dans les conteneurs
```

## Réutilisation de l'existant

- `ProjectBinding` de 065 porte le backend et la génération.
- `ManagedLaunch` conserve cwd, env, identité et canaux de supervision.
- `SpawnOrder`, `SpawnLease`, `DesiredFleet` et la récupération restent les
  autorités des agents.
- `build_environment` reste la baseline; aucun secret n'est ajouté.
- Les wrappers et adaptateurs providers restent identiques à l'intérieur.
- Les mécanismes de reconnexion socket, interruption et steering sont réutilisés.
- SPEC-075 fournit le contrat stabilisé de cycle de vie avant toute extension
  Docker de stop, relance, décommissionnement ou admission concurrente.
- Les incidents runtime délégués SPEC-068 conservent leur store, curseur,
  remise et accusé; le backend Docker ne crée pas un second canal d'incident.
- Le store Bridget conserve l'état environnement avec les autres faits hôte.

Un module `project_runtime.rs` est justifié: il porte une machine d'état et est
consommé par daemon, lifecycle, CLI/UI, récupération et tests. Aucune extraction
supplémentaire n'est admise sans trois usages réels.

## ProjectRuntimePolicy v1

La politique minimale contient:

- `policy_id`, `policy_version` et digest canonique;
- référence d'image immuable typée `registry_digest` ou `local_image_id`;
- limites CPU, mémoire et PID;
- UID/GID numériques non-root;
- racine projet canonique;
- state root projet privé;
- chemin du runtime ingress privé par project_id et binding_generation;
- environment epoch monotone;
- tmpfs autorisés;
- réseau `bridge` sans port publié.

La définition provient exclusivement du document fermé
`contracts/project-runtime-policy-config-v1.md`, chargé une fois depuis le
chemin absolu fourni au daemon. Une demande projet ne transporte que
`policy_id` et `policy_version`. L'absence ou l'invalidité du document ferme le
backend Docker, sans changer le backend host.

## ABI du conteneur v1

- Le state root hôte propre au projet est l'unique montage durable
  supplémentaire et apparaît dans le conteneur sous
  `/var/lib/bridget-project`.
- `HOME`, `XDG_CONFIG_HOME`, `XDG_CACHE_HOME`, `XDG_DATA_HOME` et
  `XDG_STATE_HOME` pointent sous cette racine. Aucun chemin ne reprend le HOME
  hôte.
- Le runtime ingress apparaît à `/run/bridget/runtime/bridget.sock`. Son chemin
  est un champ explicite de la configuration de lancement du wrapper, pas une
  dérivation de HOME.
- Le préflight exécute une probe avec l'UID/GID configuré et vérifie accès à la
  racine, écriture state, socket et lecture d'une fixture privée `0600`.

Tous les autres montages, devices, capabilities, environnements globaux et
options Docker sont refusés. La politique n'est pas une surface libre de
passage d'arguments Docker.

## Découpage technique

### Lot 1 - Contrat backend et préflight

- Étendre ProjectBinding avec host/docker et état d'environnement.
- Ajouter un préflight Docker borné: version, daemon, image digest et capacités
  minimales, utilisateur numérique, HOME/XDG, state root et ingress.
- Ajouter la machine d'état ProjectEnvironment.

Gate: la suite fixture distingue erreur Docker, image et politique avant spawn.

### Lot 2 - Création et attestation

- Construire `docker create` sans shell avec l'ensemble fermé d'options.
- Inspecter labels, image, user, montages, réseau, capabilities et limites.
- Pour une image locale, utiliser l'identifiant `sha256` produit par
  `docker build --iidfile`; pour un registre, exiger `repository@sha256` et
  attester l'identifiant résolu. Aucun tag n'est autoritatif.
- Démarrer seulement après attestation complète.

Gate: un seul mutant de contrainte suffit à classer l'environnement
`recreate_required`.

### Lot 3 - Runtime ingress et agents partagés

- Ajouter un endpoint Unix distinct par project_id/binding_generation dans un
  répertoire privé ne contenant aucun autre état.
- Monter ce seul endpoint en lecture seule dans son conteneur et jamais dans
  celui d'un autre projet.
- Attester au premier handshake project_id, binding_generation, container_id,
  environment_epoch et génération Bridget avant tout effet.
- Lancer chaque wrapper par `docker exec` avec son identité et cwd.
- Corréler l'exec à la génération Bridget et réconcilier après redémarrage.

Gate: deux agents du même projet partagent le container id, tandis qu'un
wrapper A ne peut joindre ni usurper l'ingress B.

### Lot 4 - Exploitation et rollback

- Ajouter prepare/status/stop/remove/recreate/switch-backend.
- Sérialiser réservation de spawn et passage à stopping/switching dans Bridget.
- Porter un environment_epoch sur la réservation et le revérifier juste avant
  docker exec.
- Publier `runtime_policy_changed` et invalider les admissions lors de tout
  changement de backend/version/digest/image/UID/GID afin que SPEC-067 rende
  ses profils stale.
- Interdire stop/remove/switch si une réservation ou un agent est actif.
- Revenir au backend host sans conversion de données.

Gate: parcours US3 complet, interleaving spawn/lifecycle déterministe et
non-destruction prouvée.

## Stratégie de chemin et worktrees

La racine canonique du projet est montée au même chemin absolu. Les worktrees
Git qui vivent en dehors de cette racine nécessitent un montage supplémentaire
calculé depuis `git worktree list --porcelain`, mais uniquement s'ils sont
rattachés au même common dir. Chaque montage est spécifique au projet et
attesté. Aucun scan global des worktrees n'est effectué.

Si la topologie ne peut pas être représentée sans élargir un montage, la
préparation refuse avec `unsupported_worktree_layout`.

## Cycle de vie et récupération

```text
absent -> creating -> ready -> running
   ^          |         |        |
   |          v         v        v
   +------ degraded <- stopped <- stopping
                         |
                  recreate_required
```

Au redémarrage, Bridget rapproche store, Docker inspect et agents enregistrés.
Il ne supprime rien pendant ce premier passage. Toute divergence devient un
état observable. La réparation exige une action explicite.

## Sécurité

- Image par digest, user non-root, `--cap-drop=ALL`,
  `--security-opt=no-new-privileges`, rootfs read-only.
- UID/GID numériques et droits des montages prouvés par une exécution fixture;
  le daemon Docker cible n'est pas supposé rootless.
- Aucun device, privileged, host PID/IPC/network ou Docker socket.
- Aucun montage de home global ni du cache Bridget complet.
- State root privé par projet, permissions vérifiées avant create.
- Pas de secrets en 066; les variables interdites restent interdites.
- Egress via bridge non filtré finement: limite documentée, pas promesse fausse.

## Observabilité

- Logs structurés par opération, issue, backend, phase et durée.
- Métriques bornées: environnements par état, latences create/start/exec/stop,
  erreurs par raison, ressources et nombre d'agents.
- Traces corrélées par project_id dans les logs/traces, jamais en label métrique.
- UI: digest court, état, agents actifs, limites, dérive et prochaine action.

## Stratégie de test

1. Tests unitaires des arguments Docker et de la machine d'état.
2. Faux binaire Docker pour les refus, timeouts et sorties hostiles.
3. Tests d'intégration réels sur Docker avec agent fixture sans réseau ni secret.
4. Inspection adversariale des montages, labels, droits et réseau.
5. Crash injecté entre create, attest, start, exec et persistance.
6. Redémarrage Bridget avec conteneur et agent actifs.
7. Parité steering/interruption/stop avec backend host.
8. Test de deux projets et plusieurs worktrees.
9. Empreinte non-destruction avant/après remove.
10. Gherkin `tests/features/066-environnement-partage-projet.feature`.
11. Test ABI de l'utilisateur, HOME/XDG, socket explicite et fixture privée
    `0600`, y compris échec si le wrapper tente une dérivation depuis HOME.
12. Parité SPEC-068: incident warning/failed depuis un enfant Docker, rejeu
    ordonné après reconnexion et accusé idempotent avec ProjectReference.

## Déploiement et réversibilité

- Livrer le code avec docker backend désactivé.
- Activer une fixture sans secrets.
- Activer un seul projet pilote après preuve complète.
- Ne jamais migrer les agents actifs.
- Conserver host pour chaque projet et refuser tout fallback implicite.
- Rollback: arrêter les agents, choisir host, arrêter le conteneur; données
  nouvelles conservées pour diagnostic.

## Constitution Check

| Règle | Verdict | Décision |
|---|---|---|
| Progressivité | PASS | 066 dépend de 065 et reste opt-in. |
| Minimalisme | PASS | Docker CLI, pas de SDK, Compose ou K8s. |
| Réutilisation | PASS | Spawn, fleet, lifecycle, env et wrapper existants. |
| Sécurité | PASS | Politique fermée et fail-closed, limites annoncées. |
| Observabilité | PASS | États et raisons séparés dès la conception. |
| Réversibilité | PASS | Host conservé, aucun déplacement de données. |
| Responsabilité future | PASS | Machine d'état unique, inspection et suppression compréhensibles. |
| Infrastructure déclarative | PASS | Image et politique sont versionnées; aucune manipulation production manuelle n'est le plan. |

## Gate avant implémentation

- Créer un nouveau worktree depuis `74234641fe04293c007717b8f1e3879823f5d382` ou un descendant propre contenant
  SPEC-068, après livraison et preuve de SPEC-065; ne pas réutiliser le
  worktree documentaire courant.
- L'absence de `.specify` ou de l'outil `specify` ne bloque pas T001 et ne doit
  déclencher ni installation ni mise à jour. Les documents versionnés sous
  `specs/066-environnement-partage-projet/` sont la source de vérité.
- SPEC-065 est implémentée, testée et activée sur une fixture.
- Docker Engine et ses permissions sont observés sur l'hôte cible.
- Le layout réel des worktrees est mesuré.
- L'UID/GID cible, la racine des state roots et la politique d'images fixtures
  sont choisis dans une configuration de test, sans activer un projet réel.
- `reuse-audit.md` reste PASS contre la tête d'implémentation.
- Aucun finding Analyze CRITICAL.
- L'utilisateur approuve explicitement l'activation de la tranche 066.
- Aucune implémentation ne commence tant que les gates précédentes ne sont pas levées.
