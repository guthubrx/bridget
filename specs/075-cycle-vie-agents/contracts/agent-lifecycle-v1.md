# Contrat agent lifecycle v1 - SPEC-075

## Protocole daemon

### Arrêt

```json
{"type":"StopOrder","name":"jc6","command_id":"stop-ui-<uuid>"}
```

Réponse existante `StopResult`, avec la sémantique corrigée: un succès laisse
une entrée durable `stopped`.

### Relance

```json
{"type":"RelaunchOrder","name":"jc6","command_id":"relaunch-ui-<uuid>"}
```

```json
{
  "type":"RelaunchResult",
  "command_id":"relaunch-ui-<uuid>",
  "outcome":{"kind":"started","name":"jc6","generation":42}
}
```

### Décommissionnement

```json
{"type":"DecommissionOrder","name":"jc6","command_id":"decommission-ui-<uuid>"}
```

```json
{
  "type":"DecommissionResult",
  "command_id":"decommission-ui-<uuid>",
  "outcome":{"kind":"decommissioned"}
}
```

Les noms Rust exacts et leur sérialisation JSON sont couverts par les tests de
protocole. Le résultat de relance n'est émis qu'après connexion réelle, comme
`SpawnAccepted`.

## HTTP local

Routes:

- `POST /v1/agents/stop?token=<session>`
- `POST /v1/agents/relaunch?token=<session>`
- `POST /v1/agents/decommission?token=<session>`

Corps commun:

```json
{"version":1,"name":"jc6","command_id":"<action>-ui-<uuid>"}
```

Réponse de succès:

```json
{
  "version":1,
  "name":"jc6",
  "command_id":"<action>-ui-<uuid>",
  "outcome":"stopped|stopped_forced|started|decommissioned|decommissioned_forced",
  "generation":42,
  "survivors_killed":0
}
```

Les champs non pertinents à une issue sont omis.

Erreurs HTTP:

| Statut | Code | Sens |
|---|---|---|
| 400 | `invalid_request` | Version, nom ou corrélation invalide |
| 403 | `invalid_token` | Secret de session absent ou faux |
| 404 | `agent_not_found` | Identité absente |
| 409 | `agent_not_managed` | Cycle de vie externe |
| 409 | `agent_already_stopped` | Arrêt incohérent |
| 409 | `agent_already_running` | Relance incohérente |
| 409 | `agent_not_relaunchable` | Définition durable incomplète |
| 409 | `agent_already_decommissioned` | Tombstone déjà présente |
| 409 | `agent_name_reserved` | Nouveau spawn sur un nom décommissionné |
| 409 | `lifecycle_conflict` | Mutation concurrente |
| 503 | `daemon_unavailable` | Socket ou protocole indisponible |
| 504 | `lifecycle_timeout` | Verdict non reçu dans le délai |

## CLI

```text
bridget stop <nom> [--command-id <id>]
bridget relaunch <nom> [--command-id <id>]
bridget decommission <nom> [--command-id <id>]
```

La CLI affiche le verdict typé et retourne un code non nul pour tout refus ou
timeout. Elle n'émet jamais de signal système direct.

## UI

| État attesté | Arrêter | Relancer | Décommissionner |
|---|---:|---:|---:|
| Géré actif | Oui | Non | Oui |
| Géré occupé | Oui, avec avertissement | Non | Oui, avec avertissement |
| Géré en reprise | Non | Non | Non |
| Géré arrêté | Non | Oui | Oui |
| Externe ou TMUX | Non | Non | Non |
| Mutation en cours | Non | Non | Non |

Chaque action possède sa propre confirmation. La fiche reste ouverte en cas
d'erreur et attend un nouvel instantané avant de refléter un succès.
