# Modèle de données - SPEC-086

## ProjectRole

```text
standard
bridget_system
```

Le rôle est attribué uniquement par une commande d'installation/admin locale. Une contrainte garantit au plus un `bridget_system` par store.

## BridgetSystemProject

| Champ | Type | Invariant |
|---|---|---|
| `project_id` | identifiant opaque | référence une liaison existante |
| `canonical_source_root` | chemin | absolu, attesté, exact |
| `attestation_digest` | digest | signature structurelle sans contenu secret |
| `location_id` | identifiant | emplacement system_only de SPEC-084 |
| `configured_at` | instant | audit |

## BridgetDogfoodingPolicy

| Champ | Type | Invariant |
|---|---|---|
| `generation` | entier positif | verrou optimiste |
| `mode` | enum | disabled ou enabled |
| `project_generation` | entier | projet système exact |
| `environment_epoch` | entier | augmente à chaque transition appliquée |
| `updated_at` | instant | audit |

## SystemMountTopology

| Champ | Type | Sens |
|---|---|---|
| `source_root` | chemin | checkout principal |
| `git_common_dir` | chemin | métadonnées partagées |
| `worktrees` | liste de chemins | linked worktrees attestés |
| `writable_worktree` | chemin optionnel | absent si disabled, worktree attribué si enabled |
| `digest` | digest | déclenche recreate si divergent |

## WorktreeLease

| Champ | Type | Sens |
|---|---|---|
| `canonical_worktree` | chemin | clé d'exclusion locale |
| `project_id` | identifiant | doit être le projet système |
| `work_id` | identifiant | travail propriétaire |
| `agent_id` | identifiant | observation |
| `state` | enum | active ou released |

## Transitions

```text
disabled -> transitioning -> enabled
enabled -> transitioning -> disabled
transitioning -> previous_mode en cas d'échec compensé
```

Toute transition requiert zéro agent actif et un backend Docker prêt.

Le checkout principal reste read-only dans tous les modes. Le git common dir doit être writable pour committer en mode enabled; cette capacité place tous les agents du projet système dans un même domaine de confiance coopératif.
