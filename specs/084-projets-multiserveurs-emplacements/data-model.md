# Modèle de données - SPEC-084

## ProjectLocationCatalogV2

| Champ | Type | Invariant |
|---|---|---|
| `contract_version` | entier | vaut 2 |
| `generation` | entier positif | incrément atomique |
| `locations` | liste | identifiants et chemins uniques |

## ProjectLocation

| Champ | Type | Invariant |
|---|---|---|
| `location_id` | chaîne opaque | stable, non vide, jamais un chemin |
| `label` | chaîne | affichable, bornée, non secrète |
| `canonical_path` | chemin absolu | canonicalisé côté serveur |
| `kind` | enum | `workspace` ou `exact_project` |
| `system_only` | booléen | exclut les parcours standard |
| `default_creation` | booléen | autorisé seulement sur `workspace` non système |

## ProjectPlacementPreviewV2

| Champ | Type | Sens |
|---|---|---|
| `command_id` | identifiant | idempotence |
| `catalog_generation` | entier | verrou optimiste |
| `location_id` | identifiant | autorité sélectionnée |
| `operation` | enum | `create` ou `import` |
| `requested_name` | chaîne optionnelle | création uniquement |
| `canonical_target` | chemin | résultat calculé par le daemon |
| `allowed` | booléen | décision sans effet |
| `reason` | enum optionnelle | refus fermé |

## DesktopProjectKey

| Champ | Type | Sens |
|---|---|---|
| `source_id` | identifiant Desktop | profil/source locale ou distante |
| `project_id` | identifiant daemon | identité locale au serveur |

Le couple n'est pas envoyé au daemon comme nouvelle identité. Il sert uniquement aux collections, sélections et clés de rendu du Desktop.

## Règles de transition

- v1 chargée -> projection compatible `legacy_exact`, aucune création.
- entrée v1 -> `workspace`: mutation explicite avec génération et confirmation.
- retrait d'emplacement: le catalogue change, les liaisons existantes restent.
- changement de source dans le dialogue: toute prévisualisation précédente devient invalide.

