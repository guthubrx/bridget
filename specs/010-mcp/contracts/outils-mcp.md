# Contrat : outils MCP Bridget

Référence de conformité des schémas et de la taxonomie d'erreurs. Sous-ensemble
MCP servi : `initialize` (version pinnée, capacité `tools` seule),
`notifications/initialized`, `tools/list` (idempotente, rappelable),
`tools/call`, `ping`. Pureté stdout absolue (logs sur stderr uniquement).

## Taxonomie des issues (FR-011)

| Niveau | Quand | Forme |
|---|---|---|
| Erreur JSON-RPC | paramètres invalides, méthode inconnue, `tools/call` avant `initialized` | erreur protocolaire standard (`-32602`, `-32601`, …) |
| `tools/call` avec `isError: true` | échec **technique** : daemon injoignable, timeout connexion/accusé (budget 10 s), `busy` (limite d'appels simultanés), `legacy_marker`, identité introuvable | `content` texte explicatif en français + code stable |
| Résultat **métier** (`isError` absent) | accusé ou refus Bridget | objet structuré : `status` à catégorie fermée + champs utiles |

Catégories métier fermées (extensibles — un client ignore une catégorie
inconnue sans casser) : `accepted`, `dnd`, `circuit_breaker`, `duplicate`,
`hops_exhausted`, `unknown_recipient`, `queue_full`, `reply_requires_agent`,
`outcome_unknown`.

## `bridget_send`

Entrée :

```json
{
  "to":            { "type": "string", "minLength": 1 },
  "body":          { "type": "string", "minLength": 1 },
  "reply":         { "type": "boolean", "default": false },
  "reply_timeout": { "type": "integer", "minimum": 1, "description": "secondes ; uniquement avec reply=true, sinon invalid_params" }
}
```

Résultat métier :

```json
{ "status": "accepted", "id": "…", "hops": 4 }
{ "status": "dnd", "reason": "« sol » ne souhaite pas être dérangé (encore 12 min)", "minutes_left": 12 }
{ "status": "outcome_unknown", "id": "…", "reason": "accusé perdu après transmission — retry possible avec le même id" }
```

Règles : corps transmis octet pour octet ; id métier généré avant la connexion
daemon ; retour dès accusé/refus (jamais d'attente de la réponse du
destinataire) ; `retry` explicite avec le même id → déduplication daemon.

## `bridget_who`

Entrée : `{ "domain": { "type": "string" } }` (optionnel).
Résultat : liste d'agents `{ name, type, host, os, transport, domain, model,
effort, state }` — mêmes champs que l'annuaire du binaire.

## `bridget_ledger`

Entrée :

```json
{
  "view":  { "enum": ["messages", "requests", "both"] },
  "limit": { "type": "integer", "minimum": 1, "maximum": 200, "default": 20 }
}
```

Résultat : `requests` = demandes suivies `{ id, from, to, state, deadline,
created }` (source `ListRequests`) ; `messages` = messages récents `{ id,
from, to, body, ts }` (couche de lecture partagée avec `bridget ledger`) —
schémas séparés, jamais mélangés dans une même liste.

## Identité (rappel du contrat FR-004)

Résolue à chaque appel : fichier de nom courant (`BRIDGET_AGENT_NAME_FILE`) →
filiation `agent-pids/` validée (naissance + `instance_id`) en remontant les
ancêtres avec borne → erreur `identity_not_found` (`isError`). Jamais fournie
par l'agent, jamais issue d'un nom d'environnement figé.
