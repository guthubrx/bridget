# ADR-024 - Catalogue d'emplacements et composition multi-serveurs

## Statut

Accepté pour implémentation par SPEC-084.

## Contexte

Bridget Desktop compose déjà plusieurs serveurs, tandis que chaque daemon possède son registre local. La politique de racines v1 ne distingue pas un checkout exact d'un espace autorisant la création de descendants.

## Décision

- Conserver les registres de projets locaux à chaque daemon.
- Composer l'identité dans le Desktop par `(source_id, project_id)`.
- Rendre la source cible visible et modifiable avant confirmation.
- Remplacer la liste v1 par un catalogue v2 d'emplacements `workspace` ou `exact_project`.
- Traiter les entrées v1 comme import exact jusqu'à migration explicite.
- Réserver `system_only` aux parcours experts.

## Conséquences

- Aucun service central ni synchronisation distribuée supplémentaire.
- Les anciennes installations ne gagnent aucun droit de création implicite.
- L'opérateur doit déclarer au moins un workspace pour créer des projets.
- Le checkout Bridget peut être protégé sans cesser d'être un projet enregistré.

## Alternatives rejetées

- Une base globale de projets.
- Un répertoire de création codé en dur.
- Des chemins libres fournis par le client.
- La promotion automatique de toutes les racines historiques en workspaces.

