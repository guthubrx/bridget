# Contrat de configuration v1: politiques runtime projet

## Autorité et chargement

Bridget possède ce document hôte fermé. Le daemon le charge une fois depuis un
chemin absolu explicitement fourni au démarrage. Le fichier appartient à l'UID
du daemon et n'est modifiable ni par le groupe ni par les autres comptes. Son
absence ou son invalidité ferme prepare et les admissions Docker, sans changer
le backend host ni provoquer de fallback.

Maicie et les commandes projet ne transmettent que `policy_id` et
`policy_version`. Elles ne transmettent jamais un chemin hôte, un UID/GID, une
option Docker ou une définition libre.

## Forme fermée

```json
{
  "contract_version": 1,
  "state_root_parent": "/home/moi/.local/state/bridget/projects",
  "policies": [
    {
      "policy_id": "fixture-local",
      "policy_version": 1,
      "image_reference_kind": "local_image_id",
      "image_reference": "sha256:...",
      "run_as_uid": 1002,
      "run_as_gid": 1002,
      "cpu_limit": 2.0,
      "memory_limit_bytes": 4294967296,
      "pids_limit": 512,
      "tmpfs": ["/tmp"],
      "network_mode": "bridge",
      "runtime_launcher": "/usr/local/bin/bridget",
      "runtime_executables": [
        {"agent_type": "fixture", "provider_command": "/usr/local/bin/fixture-agent"}
      ]
    }
  ]
}
```

Les champs inconnus, doublons `policy_id`/`policy_version`, UID ou GID nul,
tag mutable, valeur non bornée et chemin relatif sont refusés. Bridget dérive
le state root final depuis `state_root_parent` et un identifiant opaque borné;
le projet ne choisit jamais ce chemin.

## Références d'image

- `registry_digest`: la valeur est `repository@sha256:...`.
- `local_image_id`: la valeur est l'identifiant Docker `sha256:...`, produit
  par `docker build --iidfile` pour la fixture locale.

Dans les deux cas, l'identifiant résolu est inspecté et comparé avant création
et au démarrage. Un tag ou un nom seul n'est pas une autorité.

## ABI fixe

- state root conteneur: `/var/lib/bridget-project`;
- HOME: `/var/lib/bridget-project/home`;
- XDG: sous `/var/lib/bridget-project/xdg`;
- runtime ingress: `/run/bridget/runtime/bridget.sock`;
- utilisateur: UID/GID numériques configurés, non-root.

Ces chemins font partie de `policy_digest` et ne sont pas librement
configurables en v1.

## Exécutables internes fermés

Une politique qui déclare des agents Docker DOIT aussi déclarer les champs
runtime_launcher et runtime_executables. Chaque entrée associe exactement un
type agent à une commande absolue, sans argument, présente dans image attestée.
Les chemins définis dans le registre agent hôte ne sont jamais transmis au
conteneur. Un type absent, un chemin relatif, une traversée de répertoire, un
espace ou un doublon ferment le lancement Docker. Absence de cette paire ferme
le lancement Docker sans basculer vers backend host. Ces champs entrent dans le
calcul de policy_digest.
