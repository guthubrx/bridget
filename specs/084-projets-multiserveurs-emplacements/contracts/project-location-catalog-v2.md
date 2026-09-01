# Contrat - Catalogue d'emplacements projet v2

## Lecture

`GET /v2/projects/locations`

Réponse bornée:

```json
{
  "contract_version": 2,
  "generation": 4,
  "locations": [
    {
      "location_id": "workspace-default",
      "label": "Projets",
      "canonical_path": "/srv/projects",
      "kind": "workspace",
      "system_only": false,
      "default_creation": true
    }
  ]
}
```

## Prévisualisation

`POST /v2/projects/placement/preview`

La requête contient version, `command_id`, génération attendue, `location_id`, opération et nom de dossier seulement pour `create`. Elle ne contient jamais un parent libre.

## Application

`POST /v2/projects/placement/apply`

L'application reprend exactement l'enveloppe prévisualisée. Tout changement de source, génération, emplacement, nom ou opération impose une nouvelle prévisualisation.

## Refus fermés

- `source_unavailable`
- `capability_unavailable`
- `catalog_generation_mismatch`
- `location_not_found`
- `location_not_creatable`
- `location_system_only`
- `target_outside_location`
- `target_exists`
- `invalid_name`
- `canonicalization_failed`

## Compatibilité

Les routes v1 restent disponibles pendant la migration. Elles projettent les anciennes racines comme import exact uniquement et refusent toute création nouvelle qui ne référence pas un workspace v2.

