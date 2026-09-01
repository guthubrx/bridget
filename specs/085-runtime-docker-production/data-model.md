# Modèle de données - SPEC-085

## ServerExecutionSettingsV1

| Champ | Type | Invariant |
|---|---|---|
| `generation` | entier positif | mutation optimiste atomique |
| `default_backend` | enum | `host` ou `docker` |
| `default_policy_id` | option chaîne | obligatoire si Docker par défaut |
| `default_policy_version` | option entier | version résolue |

## RuntimeCapability

| Champ | Type | Sens |
|---|---|---|
| `docker_available` | booléen | daemon Docker joignable par Bridget |
| `policy_available` | booléen | politique valide chargée |
| `resource_catalog_available` | booléen | catalogue valide chargé |
| `image_attested` | booléen | digest attendu résolu |
| `reason` | enum optionnelle | indisponibilité fermée |

## ProjectRuntimeTransition

| Champ | Type | Sens |
|---|---|---|
| `command_id` | identifiant | idempotence |
| `project_id` | identifiant | cible locale |
| `from_backend` | enum | état source confirmé |
| `to_backend` | enum | état demandé |
| `policy_reference` | option | politique Docker résolue |
| `phase` | enum | validating, preparing, attesting, committing, compensating, completed, failed |
| `previous_generation` | entier | liaison à préserver |
| `observed_at` | instant | diagnostic |

## ProjectRuntimeBinding

Le modèle existant est conservé: backend, génération, runtime policy, epoch, état et raison. Un champ de topologie de montage peut porter un digest non secret des chemins approuvés afin de détecter un recreate nécessaire.

## États

```text
Host -> activating -> Docker(absent|preparing|ready|stopped|degraded)
Docker -> switching_to_host -> Host
Docker -> recreating -> Docker(ready|degraded)
```

Une transition échouée revient à la liaison publiée précédente. Aucun état hybride n'est présenté comme actif.

