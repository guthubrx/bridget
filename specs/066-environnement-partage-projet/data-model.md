# Modèle de données: Environnement Docker partagé par projet

## ProjectEnvironment

| Champ | Règle |
|---|---|
| `project_id` | référence à ProjectBinding, clé primaire |
| `binding_generation` | doit égaler la liaison 065 |
| `backend` | `docker` pour cette entité |
| `state` | machine d'état fermée |
| `container_id` | optionnel avant création, observé et opaque |
| `container_name` | dérivé d'un hash borné de project_id |
| `image_reference_kind` | `registry_digest` ou `local_image_id` |
| `image_reference` | référence immuable configurée pour la génération |
| `resolved_image_id` | identifiant Docker `sha256` observé et attesté |
| `policy_id`, `policy_version` | définition hôte exacte utilisée |
| `policy_digest` | hash de la politique canonique |
| `environment_epoch` | entier monotone, changé avant lifecycle ou recreate |
| `runtime_contract_version` | version de l'attestation |
| `created_at`, `updated_at` | instants UTC |
| `last_reason` | raison structurée optionnelle |

## ProjectRuntimePolicy

| Champ | Règle |
|---|---|
| `image_reference_kind` | forme immuable explicitement attestée |
| `image_reference` | `repository@sha256` ou image ID `sha256`, jamais tag |
| `policy_id`, `policy_version` | clé de la définition hôte fermée |
| `cpu_limit` | borne positive configurée |
| `memory_limit_bytes` | borne positive configurée |
| `pids_limit` | borne positive configurée |
| `project_root` | racine canonique 065 |
| `worktree_mounts` | uniquement même common dir Git |
| `state_root` | privé, spécifique project_id |
| `run_as_uid`, `run_as_gid` | entiers non nuls, prouvés sur les montages |
| `container_state_root` | `/var/lib/bridget-project`, fixe |
| `container_home` | `/var/lib/bridget-project/home`, fixe |
| `container_xdg_roots` | sous `/var/lib/bridget-project`, ensemble fermé |
| `runtime_ingress_dir` | privé par project_id/binding_generation, lecture seule dans ce seul conteneur |
| `runtime_ingress_socket` | endpoint unique attesté, jamais partagé entre projets |
| `container_ingress_socket` | `/run/bridget/runtime/bridget.sock`, explicite |
| `tmpfs` | ensemble fermé |
| `network_mode` | `bridge` uniquement en v1 |

## ContainerAttestation

Observation non autoritative tant qu'elle n'a pas été validée contre la
politique: container id, image id/digest, labels, mounts, user, read-only root,
security opts, capabilities, network, ports et limites.

## ContainerAgentExecution

| Champ | Règle |
|---|---|
| `agent_instance_id` | identité Bridget |
| `generation` | génération Bridget |
| `project_id` | égal à l'environnement |
| `binding_generation` | égale à la liaison attestée |
| `environment_epoch` | égal à la réservation et revérifié avant exec |
| `container_id` | attestation courante |
| `exec_id` | opaque, distinct de l'agent |
| `cwd` | racine ou worktree autorisé |
| `state` | starting, running, terminal, lost |
| `provider_identity` | issue de SPEC-064 |

## ProjectSpawnReservation

| Champ | Règle |
|---|---|
| `reservation_id` | opaque et unique |
| `project_id` | environnement visé |
| `binding_generation` | génération 065 attendue |
| `environment_epoch` | epoch observé à l'admission |
| `agent_generation` | génération Bridget réservée |
| `state` | reserved, executing, released, rejected |

La réservation et le passage de l'environnement vers `stopping` ou
`switching` utilisent la même autorité transactionnelle Bridget. Une
réservation active fait refuser le lifecycle; un lifecycle engagé interdit une
nouvelle réservation.

## Transitions

```text
absent -> creating -> ready -> running
              |         |        |
              v         v        v
           degraded <- stopping <-+
              |          |
              |       switching -> stopped
              +-> recreate_required -> absent après remove explicite
```

Invariants:

- un seul container id attesté par project_id;
- aucun remove si une ContainerAgentExecution non terminale existe;
- chaque ingress est privé à un project_id et une binding_generation;
- la première connexion divergente est refusée avant inscription;
- lifecycle et spawn sont exclus mutuellement par réservation et epoch;
- aucune réutilisation si binding_generation, image ou policy divergent;
- aucune déduction du socket, des fichiers de suivi ou de l'état Bridget depuis
  HOME; seul le state root projet fournit HOME/XDG au conteneur;
- tout changement backend, policy_version/digest, image ou UID/GID incrémente
  environment_epoch et produit `runtime_policy_changed`;
- aucune transition Docker ne change ProjectIdentity Maicie;
- `switch_backend` exige zéro exécution active.

## Index et bornes

- Index unique sur project_id et container id.
- Index sur état et binding_generation.
- Liste des worktrees bornée par le dépôt observé.
- Réconciliation O(p + a), p environnements et a agents, avec maps indexées;
  aucune double boucle par défaut.
