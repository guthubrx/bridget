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
`outcome_unknown`, `envelope_mismatch`, `idempotency_expired`,
`invalid_issued_at`.

## `bridget_send`

Entrée :

```json
{
  "to":            { "type": "string", "minLength": 1 },
  "body":          { "type": "string", "minLength": 1 },
  "reply":         { "type": "boolean", "default": false },
  "reply_timeout": { "type": "integer", "minimum": 1, "description": "secondes ; uniquement avec reply=true, sinon invalid_params" },
  "id":            { "type": "string", "minLength": 1, "description": "clé métier de retry" },
  "issued_at":     { "type": "integer", "minimum": 1, "description": "valeur renvoyée par le premier appel ; requise avec id au retry" }
}
```

Résultat métier :

```json
{ "status": "accepted", "id": "…", "issued_at": 1700000000, "hops": 4 }
{ "status": "dnd", "reason": "« sol » ne souhaite pas être dérangé (encore 12 min)", "minutes_left": 12 }
{ "status": "outcome_unknown", "id": "…", "issued_at": 1700000000, "delivery_id": "…", "reason": "remise en vol — le destinataire n'a pas encore accusé ; rejouer le même id et le même issued_at lit le sort réel sans jamais dupliquer" }
{ "status": "outcome_unknown", "id": "…", "issued_at": 1700000000, "reason": "sort indéterminé — rejouer le même id et le même issued_at lit le sort réel sans jamais dupliquer" }
```

Règles : corps transmis octet pour octet ; id métier généré avant la connexion
daemon ; retour dès accusé/refus (jamais d'attente de la réponse du
destinataire) ; le premier résultat renvoie `id` et `issued_at`, qui doivent
être rejoués ensemble pour un retry → déduplication daemon.

`outcome_unknown` est le retour **nominal** d'un premier envoi : le daemon
répond avant que le destinataire ait accusé. La présence de `delivery_id`
distingue les deux cas — avec lui, la remise est en vol et le dépôt a réussi ;
sans lui, le sort est réellement indéterminé. Dans les deux cas le rejeu de la
même clé (`id` + `issued_at`) est une **consultation** sûre, jamais une seconde
émission : il rend `accepted` une fois l'accusé aval consolidé. Côté CLI, le
code de sortie suit le dépôt : `0` quand `delivery_id` est présent.

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
