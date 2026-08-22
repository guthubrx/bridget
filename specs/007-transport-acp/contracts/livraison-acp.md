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
   `sessionUpdate == "agent_message_chunk"` et `content.type == "text"`
   (et dont le `sessionId`, quand présent, correspond à la session), dans
   l'ordre — jamais le texte porté par un appel d'outil ni la progression.
2. La réponse est routée par le canal wrapper→daemon avec l'id du message
   d'origine — c'est cet id qui clôt la demande suivie (cycle de vie 003).
3. Un seul tour actif par équipier ; les messages reçus pendant un tour sont
   livrés FIFO aux tours suivants (FR-007).

## Demandes de permission pendant un tour

Une requête `session/request_permission` porte `params.options[]`. La réponse
ACP v1 sélectionne un `optionId` réellement proposé : `outcome: Selected` pour
une option autorisante/rejetante suivant la politique ; `outcome: Cancelled`
si aucun rejet n'est proposé ou si le tour est annulé pendant la permission.

`session/cancel` est une notification JSON-RPC sans `id`. Le timeout court ne
s'applique qu'à `initialize` et `session/new` : un prompt attend son résultat
ou l'annulation de l'autorité daemon. Les notifications inconnues sont
journalisées ; une requête inconnue reçoit aussi `-32601`.
