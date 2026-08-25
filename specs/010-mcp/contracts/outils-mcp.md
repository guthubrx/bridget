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
{ "status": "outcome_unknown", "id": "…", "issued_at": 1700000000, "delivery_id": "…", "reason": "remise en vol — le destinataire n'a pas encore accusé ; rejouer à l'identique — même id, même issued_at, même corps — lit le sort réel sans jamais dupliquer" }
{ "status": "outcome_unknown", "id": "…", "issued_at": 1700000000, "reason": "sort indéterminé ; rejouer à l'identique — même id, même issued_at, même corps — lit le sort réel sans jamais dupliquer" }
{ "status": "outcome_unknown", "id": "…", "issued_at": 1700000000, "reason": "accusé perdu après transmission — rejouer à l'identique … (détail technique)" }
```

Règles : corps transmis octet pour octet ; id métier généré avant la connexion
daemon ; retour dès accusé/refus (jamais d'attente de la réponse du
destinataire) ; le premier résultat renvoie `id` et `issued_at`, qui doivent
être rejoués ensemble pour un retry → déduplication daemon.

`outcome_unknown` est le retour **nominal** d'un premier envoi : le daemon
répond avant que le destinataire ait accusé. La présence de `delivery_id`
distingue les deux cas — avec lui, la remise est en vol et le dépôt a réussi ;
sans lui, le sort est réellement indéterminé. Dans les deux cas le rejeu à
l'identique (**même `id`, même `issued_at`, même corps**) est une
**consultation** sûre, jamais une seconde émission : il rend `accepted` une fois
l'accusé aval consolidé.

`outcome_unknown` a donc **trois** formes, pas deux. Les deux premières viennent
d'une issue rendue par le daemon (avec ou sans `delivery_id`). La troisième naît
côté client, sans issue du tout : la connexion tombe **après** l'écriture de la
commande, si bien que l'outil ne lit jamais la réponse. Le message a pu partir
ou non ; c'est le seul cas où l'outil l'ignore vraiment. Le rejeu à l'identique
est là aussi le geste correct, et le seul.

Un `delivery_id` n'atteste un dépôt que tant que la remise est **en vol**. Une
remise mise en quarantaine — échec de reprise, `DeliveryIndeterminate`, ou
migration écartant une enveloppe absente — cesse d'être annoncée comme telle et
retombe sur la forme « sort indéterminé » : cet état est absorbant, plus rien ne
l'accusera, et l'annoncer comme un dépôt réussi serait un mensonge tenu jusqu'à
l'expiration de l'horizon.

`envelope_mismatch` survient lorsqu'un rejeu réutilise un `id` déjà connu avec
un contenu différent — corps modifié, destinataire changé, `issued_at` distinct.
C'est la garde qui rend le rejeu sûr : elle refuse qu'une clé déjà engagée serve
à faire passer un autre message. Un rejeu qui la déclenche n'est pas à
contourner par une nouvelle clé sans avoir d'abord lu le sort du premier envoi ;
tout ajout part dans un message séparé.

Le code de sortie du binaire n'est pas défini ici : il est domicilié dans
`specs/003-cycle-vie-demandes/contracts/cli.md`.

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
from, to, body, ts, delivery_status? }` (couche de lecture partagée avec
`bridget ledger`) — schémas séparés, jamais mélangés dans une même liste.

`ts` est l'horodatage d'**émission** (gravure au ledger au début de la remise),
pas celui de l'accusé. Il fait foi pour la chronologie des émissions ; ce n'est
pas un horodatage de réception — ne pas en déduire un délai de livraison.

`delivery_status` (snake_case, optionnel) expose la phase de remise idempotente
quand une saga `send_deliveries` existe pour le même `id` (plage P31 :
`protocol.rs:LedgerMessage`). Valeurs fermées :

| Valeur JSON | Phase SQL | Sens pour le lecteur |
|---|---|---|
| `en_vol` | `dispatching` | émis, accusé pas encore reçu — ne pas conclure à une perte |
| `recu` | `acked` | destinataire a accusé |
| `indetermine` | `indeterminate` | quarantaine absorbante — ne sera plus accusé |
| *(absent)* | pas de saga / Store-only | statut inconnu — **ne pas inventer** `recu` |

Vocabulaires équivalents, une seule réalité : `en_vol` (projection ledger /
MCP) = phase SQL `dispatching` = dépôt « en vol » / `in_flight` côté issue
CLI (`outcome_unknown` avec `delivery_id`). Ce ne sont pas trois états
concurrentiels.

Le rendu CLI miroir suffixe `[en vol]` / `[reçu]` / `[indéterminé]`. Visibilité
ledger ≠ accusé : un message peut être listé `en_vol` sans que le destinataire
l'ait encore vu.

## Identité (rappel du contrat FR-004)

Résolue à chaque appel : fichier de nom courant (`BRIDGET_AGENT_NAME_FILE`) →
filiation `agent-pids/` validée (naissance + `instance_id`) en remontant les
ancêtres avec borne → erreur `identity_not_found` (`isError`). Jamais fournie
par l'agent, jamais issue d'un nom d'environnement figé.
