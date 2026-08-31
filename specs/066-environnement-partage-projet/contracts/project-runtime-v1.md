# Contrat public v1: runtime projet

## Surface opérateur

Opérations locales:

- `prepare`
- `status`
- `stop`
- `remove`
- `recreate`
- `switch_backend`

Les opérations distantes, UI et MCP sont consultatives dans cette version.

## ProjectRuntimeStatus

```json
{
  "contract_version": 1,
  "project_id": "opaque-project-id",
  "binding_generation": 1,
  "backend": "docker",
  "state": "ready",
  "container_id": "opaque-container-id",
  "policy_id": "default-local",
  "policy_version": 1,
  "image_reference_kind": "local_image_id",
  "image_reference": "sha256:...",
  "resolved_image_id": "sha256:...",
  "policy_digest": "sha256:...",
  "environment_epoch": 7,
  "run_as_uid": 1002,
  "run_as_gid": 1002,
  "active_agents": 0,
  "observed_at": 1787997600,
  "next_action": "launch_agent"
}
```

## Ensemble fermé de raisons

- `docker_unavailable`
- `docker_timeout`
- `image_missing`
- `image_not_pinned`
- `policy_invalid`
- `runtime_policy_config_unavailable`
- `runtime_policy_changed`
- `runtime_user_incompatible`
- `container_abi_mismatch`
- `project_binding_stale`
- `container_missing`
- `container_labels_mismatch`
- `container_image_mismatch`
- `container_policy_mismatch`
- `forbidden_mount`
- `forbidden_privilege`
- `unsupported_worktree_layout`
- `runtime_ingress_unavailable`
- `runtime_ingress_identity_mismatch`
- `runtime_ingress_generation_mismatch`
- `active_agents_present`
- `spawn_reservation_active`
- `lifecycle_in_progress`
- `environment_epoch_stale`
- `exec_lost`
- `resource_limit_reached`
- `recreate_required`

## Attestation obligatoire

Un environnement ne devient `ready` qu'après inspection et comparaison de:

- labels;
- digest image;
- montages source/destination/mode;
- user non-root;
- rootfs read-only;
- capabilities et no-new-privileges;
- réseau et ports;
- limites CPU, mémoire, PID.
- UID/GID numériques, HOME/XDG sous `/var/lib/bridget-project` et socket
  explicite `/run/bridget/runtime/bridget.sock`.

Une sortie Docker non JSON, incomplète ou inconnue rend `degraded` ou
`recreate_required`, jamais `ready`.

## Handshake du runtime ingress

Chaque environnement reçoit un endpoint Unix distinct sous un répertoire hôte
privé par `project_id` et `binding_generation`. Seul ce répertoire est monté
dans son conteneur. Avant toute inscription, le wrapper présente
`project_id`, `binding_generation`, `container_id`, `environment_epoch` et sa
génération Bridget. Le daemon compare ces valeurs à ProjectBinding,
ProjectEnvironment et à la réservation de spawn attendue. Toute divergence
ferme la connexion sans créer agent, session, remise ou événement métier.

Un redémarrage Bridget recrée le même endpoint privé pour la génération
attestée. La reconnexion rejoue le handshake complet; la possession du chemin
du socket seule ne vaut jamais identité.

Le chemin conteneur du socket est fourni explicitement à la configuration du
wrapper. Le wrapper ne le calcule jamais depuis HOME. Les fichiers de nom,
suivi de PID ou reprise du wrapper utilisent le state root projet ou des champs
explicites, jamais le cache Bridget hôte.

## Référence d'image

`registry_digest` exige `repository@sha256:...`. `local_image_id` exige
l'identifiant `sha256:...` observé localement, notamment celui écrit par
`docker build --iidfile` pour la fixture. Dans les deux cas, Bridget résout puis
atteste l'identifiant réel avant create et au démarrage. Un tag ou un nom seul
n'est jamais comparé comme autorité.

## Changement de politique

Un changement de backend, policy_id/version, policy_digest, image ou UID/GID
incrémente `environment_epoch` et produit `runtime_policy_changed`. Toute
réservation antérieure est refusée. SPEC-067 consomme ce fait pour rendre un
profil stale avant résolution de ressource.

## Exclusion lifecycle et spawn

Une réservation de spawn capture `environment_epoch`. Le passage atomique à
`stopping` ou `switching` échoue si une réservation ou exécution est active et
bloque toute nouvelle réservation. Juste avant `docker exec`, Bridget compare
à nouveau l'epoch et l'état; une divergence refuse l'exec.

## Parité des incidents runtime délégués

Le backend Docker réutilise le canal durable SPEC-068. Un incident `warning` ou
`failed` produit par un agent délégué conserve le même `ProjectReference` que
l'exécution source dans `DelegatedRuntimeEventFrame`, le store, la notification,
le rejeu et l'acquittement. Un redémarrage avant acquittement ne perd pas
l'incident et ne le réordonne pas.

Aucun second canal propre à Docker n'est créé. Le changement de backend ne
modifie ni les codes fermés, ni les règles de curseur, ni l'idempotence, ni la
sémantique d'acquittement définis par SPEC-068.

## Compatibilité

- `backend=host` ignore ProjectEnvironment et suit ManagedLaunch existant.
- Le fallback docker vers host n'est jamais automatique.
- Les champs environnement sont optionnels dans les projections historiques.
