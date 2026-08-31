# Contrat project-round-ui-v1

## Principes

Le relais HTTP loopback traduit une intention d'opérateur vers `ProjectRoundPolicyV1`. Il n'est pas une nouvelle autorité et n'accepte aucune cadence, commande libre, chemin ou fournisseur.

## Lecture

`GET /v1/projects?token=<jeton>` conserve la version 1 et enrichit chaque projet.

```json
{
  "version": 1,
  "projects": [
    {
      "project_id": "project-opaque",
      "display_name": "bridget",
      "canonical_path": "/racine/autorisee/bridget",
      "state": "active",
      "binding_generation": 3,
      "round": {
        "configured": true,
        "enabled": true,
        "revision": 2,
        "updated_at": 1788160000,
        "interval_secs": 420,
        "last_occurrence_at": 1788159780,
        "last_dispatch_state": "deposited",
        "last_dispatch_observed_at": 1788159792
      }
    }
  ]
}
```

Les trois champs `last_*` sont absents ensemble lorsqu'aucun passage n'est connu.

## Mutation

`POST /v1/projects/round?token=<jeton>`

```json
{
  "version": 1,
  "command_id": "ui-project-round-uuid",
  "project_id": "project-opaque",
  "binding_generation": 3,
  "enabled": false
}
```

Réponse 200 uniquement après confirmation du daemon :

```json
{
  "version": 1,
  "command_id": "ui-project-round-uuid",
  "project_id": "project-opaque",
  "binding_generation": 3,
  "round": {
    "configured": true,
    "enabled": false,
    "revision": 3,
    "updated_at": 1788160042,
    "interval_secs": 420,
    "last_occurrence_at": 1788159780,
    "last_dispatch_state": "deposited",
    "last_dispatch_observed_at": 1788159792
  }
}
```

## Validation

- corps JSON strict, aucun champ inconnu;
- `version == 1`;
- `command_id` non vide, au plus 160 octets et limité aux octets de requête déjà admis;
- `project_id` non vide, au plus 128 octets, alphanumérique, tiret ou souligné;
- `binding_generation > 0`;
- `enabled` booléen obligatoire.

## Erreurs

| HTTP | Code | Sens |
|---:|---|---|
| 400 | `invalid_request` | corps, version ou identifiant invalide |
| 404 | `project_not_found` | projet absent |
| 409 | `project_inactive` | liaison inactive |
| 409 | `binding_generation_mismatch` | menu ouvert sur une génération obsolète |
| 409 | `round_command_conflict` | même commande avec une enveloppe différente |
| 503 | `round_service_unavailable` | daemon, capacité ou store indisponible |

Toutes les erreurs utilisent l'enveloppe existante :

```json
{
  "version": 1,
  "code": "binding_generation_mismatch",
  "message": "Le projet a été reconnecté. Actualisez son état avant de modifier la ronde."
}
```

## Accessibilité et rendu

- L'action est un `menuitemcheckbox` avec `aria-checked` égal à l'état confirmé.
- Un projet inactif rend l'action désactivée avec une explication textuelle.
- Pendant la requête, le contrôle porte `aria-busy=true`; le projet en mémoire reste inchangé.
- Après succès, la liste complète est relue avant affichage.
- La ligne affiche `actif · ronde activée` seulement lorsque `enabled=true`.
- Le prochain passage est décrit par un délai maximal de 420 secondes, jamais par une heure calculée.

## Confidentialité

La réponse ne doit ajouter ni racine hors de la projection administrative déjà autorisée, ni secret, ni profil, ni fournisseur, ni contenu de message.
