# Contrat CLI : demandes suivies

## Créer une demande suivie

```text
bridget send --to <agent> --reply [--timeout <secondes>] <message>
```

Retour : l'identifiant de demande est affiché dans l'acquittement.

## Annuler

```text
bridget cancel <id> [--reason <texte>]
```

Résultats : `cancelled` si la demande ouverte appartient à l'agent courant; `cancelled` à nouveau si elle était déjà annulée par ce même agent; erreur actionnable sinon.

## Consulter

```text
bridget requests
bridget requests --json
```

Affiche uniquement les demandes de l'agent courant, avec identifiant, destinataire, état et délai restant ou date terminale.

## Répondre

```text
bridget reply <message>
bridget reply --in-reply-to <id> <message>
bridget send --to <agent> --in-reply-to <id> <message>
```

La CLI associe `bridget reply` à la dernière demande suivie reçue par l'agent.
Le flag `--in-reply-to` sélectionne explicitement une demande et prend le pas
sur cet identifiant implicite. Après livraison, le daemon clôt la demande par
la même transition transactionnelle que les réponses MCP ; ses rappels cessent.

## Code de sortie d'un envoi idempotent

Le code de sortie du binaire suit le **dépôt**, jamais la consolidation de
l'accusé aval. Le daemon répond avant que le destinataire ait accusé : exiger
l'accusé pour sortir en `0` ferait échouer tout envoi nominal.

| Issue | Sortie | Code |
|---|---|---|
| `accepted` | stdout | `0` |
| remise en vol — `delivery_id` probant | stdout, `DÉPÔT: in_flight (remise en vol)` | `0` |
| sort indéterminé — pas de `delivery_id` probant | stderr, `ISSUE: outcome_unknown` | `1` |
| `rejected` (dnd, circuit ouvert, destinataire inconnu, …) | stderr | `1` |
| `envelope_mismatch`, `idempotency_expired`, `invalid_issued_at` | stderr | `1` |

Le binaire nomme le **même statut** que le retour MCP — `in_flight` et
`outcome_unknown` — parce que les deux surfaces partagent un point de vérité
unique. Un agent qui lit les deux n'a aucune traduction à faire, et elles ne
peuvent pas diverger.

Un `delivery_id` est **probant** s'il est présent et non vide : le daemon n'en
produit jamais de vide, et l'accepter annoncerait un dépôt sur une valeur qu'il
refuse lui-même. La ligne imprimée et le code de sortie appliquent la même règle,
donc l'affichage ne peut pas contredire le `rc`.

Un `delivery_id` n'atteste un dépôt que tant que la remise est **en vol** : une
remise en quarantaine (échec de reprise, `DeliveryIndeterminate`, migration
écartant une enveloppe absente) n'en porte plus et sort donc en `1`. L'état est
absorbant — l'annoncer comme un dépôt réussi serait un faux succès tenu jusqu'à
l'expiration de l'horizon. `in_flight` atteste le DÉPÔT, jamais la remise au
destinataire.

Dans tous les cas non terminaux, le geste correct est de **rejouer à
l'identique** — même `id`, même `issued_at`, même corps : c'est une
consultation, jamais une seconde émission. Voir
`specs/010-mcp/contracts/outils-mcp.md` pour la forme MCP des mêmes issues.
