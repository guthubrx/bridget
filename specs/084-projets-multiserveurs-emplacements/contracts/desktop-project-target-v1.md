# Contrat Desktop - Ciblage d'une opération projet

## Autorité

Bridget Desktop choisit une `source_id`, ouvre ou réutilise son tunnel, puis adresse exclusivement le daemon de cette source. Le daemon ne connaît pas la flotte.

## État du dialogue

```text
source_id
source_status
source_capabilities
location_id
catalog_generation
operation
preview
```

## Invariants

- La source est toujours rendue à l'écran.
- Changer de source purge emplacement et prévisualisation incompatibles.
- Une source déconnectée peut être choisie pour consultation mais pas confirmée.
- La clé d'un projet agrégé est `(source_id, project_id)`.
- Aucun jeton de tunnel, secret SSH ou chemin non projeté par le daemon n'entre dans l'état rendu.

