# Contrat : mapping message Bridget ↔ tour ACP

Ce contrat est la référence de conformité des tests à fixtures. Il doit rester
stable : tout changement est une décision, pas un détail.

## Sens entrant : message Bridget → `session/prompt`

Le prompt d'un tour est composé de deux blocs de texte, sans JSON ni emoji :

```text
[message Bridget de <expéditeur> — réponse attendue : oui|non]

<corps du message, brut, intact>
```

Règles :

1. L'en-tête tient sur une ligne, entre crochets, format fixe ci-dessus.
2. Le corps est transmis **octet pour octet** après une ligne vide — aucun
   échappement, aucune troncature, aucun reformatage (SC-002).
3. Une relance de demande suivie est un message ordinaire dont le corps est le
   texte de relance existant du daemon ; l'en-tête porte l'expéditeur d'origine.
4. Aucune autre instruction n'est ajoutée au prompt (FR-004). L'équipier n'a
   pas besoin de savoir « comment répondre » : sa réponse naturelle EST la
   réponse.

## Sens sortant : fin de tour → réponse Bridget

| `stopReason` du tour | `reply=yes` (demande suivie) | `reply=no` (notification) |
|---|---|---|
| fin normale avec texte | texte final routé vers l'émetteur, demande close | rien routé, texte au journal |
| fin normale sans texte | échec motivé « réponse vide » vers l'émetteur | rien |
| refus / erreur / annulation | échec motivé avec le `stopReason` vers l'émetteur | rien, erreur au journal |
| processus mort avant fin de tour | échec motivé « équipier arrêté » + état annuaire mis à jour | idem journal |

Règles :

1. Le « texte final » est la concaténation des blocs dont
   `sessionUpdate == "agent_message_chunk"` **et** `content.type == "text"`
   (et dont le `sessionId`, quand présent, correspond à la session), dans
   l'ordre — **jamais** le texte porté par `tool_call`/`tool_call_update` ni la
   progression (précision imposée par la pair review T704 : sans le
   discriminateur, du texte d'outil polluerait la réponse).
2. La réponse est routée par le canal wrapper→daemon avec l'id du message
   d'origine — c'est cet id qui clôt la demande suivie (cycle de vie 003).
3. Un seul tour actif par équipier ; les messages reçus pendant un tour sont
   livrés FIFO aux tours suivants (FR-007).

## Demandes de permission pendant un tour

Contrat ACP v1 exact (corrigé après pair review T704 — la forme
`result.outcome = "allow"|"deny"` est **hors contrat**) :

- la requête `session/request_permission` porte `params.options[]`, chaque
  option ayant un `optionId` et un `kind` officiel : `allow_once` /
  `allow_always` / `reject_once` / `reject_always` (pas de `deny_*` — valeur
  inexistante au schéma) ;
- la réponse DOIT suivre l'enveloppe **imbriquée et en minuscules** du schéma
  officiel : `result.outcome = { "outcome": "selected", "optionId": … }`, avec
  un `optionId` **choisi parmi les options reçues** — politique `allow` → kind
  `allow_*`, politique `deny` → kind `reject_*` ;
- si aucune option ne convient, la réponse est
  `result.outcome = { "outcome": "cancelled" }` — une requête serveur ne reste
  **jamais** sans réponse ;
- si le tour est annulé pendant qu'une permission est pendante, elle est close
  par `cancelled` ;
- les fixtures de test proviennent du schéma officiel et l'assertion porte sur
  le **JSON brut** émis.

Chaque demande et sa réponse automatique sont journalisées
(`event: permission`).

## Conformité protocolaire complémentaire

- `session/cancel` est une **notification** JSON-RPC : émise **sans `id`**,
  aucune réponse attendue (une forme requête créerait un waiter orphelin).
- Le timeout de requête court (poignée de main `initialize`/`session/new`) ne
  s'applique **jamais** à `session/prompt` : un tour attend jusqu'à son
  résultat ou jusqu'à l'annulation décidée par l'autorité daemon (D-206) ; les
  tours de notification (`reply=no`) relèvent du `notify_timeout_secs` du
  registre. **Tout timeout ou annulation d'un tour actif suit le même
  protocole** : retrait atomique du waiter, `session/cancel`, attente d'une
  grâce bornée de fin de tour ; si l'adaptateur l'ignore, arrêt forcé
  (kill+récolte), transport marqué mort, file drainée en échecs terminaux —
  jamais un simple abandon d'attente pendant que le tour continue.
- Toute méthode ou notification inconnue reçue est journalisée (événement
  diagnostic) ; une requête inconnue reçoit en plus l'erreur `-32601`.
