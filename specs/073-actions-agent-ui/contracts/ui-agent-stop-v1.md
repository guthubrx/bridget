# Contrat local UI - Décommissionnement d'un agent v1

## Requête

```http
POST /v1/agents/stop?token=<secret-session>
Content-Type: application/json
```

```json
{
  "version": 1,
  "name": "agent-jetable",
  "command_id": "stop-ui-<identifiant>"
}
```

Les champs inconnus sont refusés. Le secret reste transporté par le mécanisme
local existant et ne figure jamais dans la réponse ni les logs fonctionnels.

## Succès arrêt propre

```http
HTTP/1.1 200 OK
Content-Type: application/json
```

```json
{
  "version": 1,
  "name": "agent-jetable",
  "command_id": "stop-ui-<identifiant>",
  "outcome": "stopped"
}
```

## Succès avec terminaison forcée

```json
{
  "version": 1,
  "name": "agent-jetable",
  "command_id": "stop-ui-<identifiant>",
  "outcome": "stopped_forced",
  "survivors_killed": 1
}
```

## Erreurs

| HTTP | Code | Sens |
|---:|---|---|
| 400 | `invalid_request` | JSON, version, nom ou identifiant invalide |
| 404 | `agent_not_found` | agent absent au moment de la demande |
| 409 | `agent_not_managed` | cycle de vie non géré par Bridget |
| 409 | `agent_stopped` | agent déjà arrêté |
| 503 | `daemon_unavailable` | socket ou protocole indisponible |
| 504 | `stop_timeout` | disparition réelle non confirmée dans le délai |

```json
{
  "version": 1,
  "code": "agent_not_managed",
  "message": "Cet agent n'est pas géré par Bridget."
}
```

## Invariants

- Le relais ne signale jamais un succès avant `StopResult`.
- Le relais ne transforme jamais une erreur en état `stopped`.
- La route ne lance aucun processus et n'envoie aucun signal système.
- La route n'efface aucun historique.
- Le daemon reste l'unique autorité sur le verdict terminal.
- `command_id` sert à la corrélation de la demande et de la réponse ; ce contrat
  ne promet pas une déduplication durable des ordres d'arrêt.
- Le navigateur bloque une seconde émission en vol et n'effectue aucun retry
  automatique.
