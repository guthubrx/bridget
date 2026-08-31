# Contrat relay - Server Control v1

Toutes les routes sont accessibles seulement via le relais local déjà authentifié. Elles utilisent `version: 1`, refusent les champs inconnus et ne retournent aucun secret.

## `GET /v1/control/overview`

Retourne l'identité bornée du serveur, les catégories disponibles, la génération de contrôle, la dernière synchronisation et les diagnostics non sensibles.

## `GET /v1/control/settings`

Retourne `ServerControlSnapshotV1`. Un serveur qui ne possède aucune clé mutable retourne tout de même ses catégories et leurs raisons de lecture seule.

## `POST /v1/control/settings/preview`

Entrée: `version`, `command_id`, `expected_generation`, `changes`.

Effet: aucun.

Sortie succès: `ControlChangePreview` avec delta normalisé et avertissements.

Refus: `400 invalid_request`, `403 local_operator_required`, `409 control_generation_stale`, `409 setting_refused`.

## `POST /v1/control/settings/apply`

Entrée: copie exacte et confirmée de la prévisualisation, y compris `command_id` et `expected_generation`.

Effet: transaction atomique de toutes les clés. Une confirmation se produit exclusivement dans le client local avant cet appel.

Sortie succès: `201 ControlChangeReceipt`.

Refus: mêmes codes que preview. Une répétition exacte de `command_id` retourne le reçu existant. Un même id avec un autre corps est refusé.

## Invariants

- `key` est une énumération fermée; aucune clé JSON arbitraire, commande, chemin ou variable d'environnement n'est admise.
- L'absence de capacité retourne une projection lecture seule ou un refus explicite, jamais une valeur supposée.
- Les routes `/v1/projects/settings` et `/v1/projects/preview` existantes restent compatibles pendant la migration et délèguent vers le même validateur lorsque pertinent.
