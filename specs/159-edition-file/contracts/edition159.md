# Contrat - File retenue et édition 159

Compatible avec les contrats 148, 149, 150 et 156. Réglage `held_queue` fermé : un
appel et sa remise produisent les mêmes octets et le même comportement qu'avant.

## Réglage `held_queue` (lot A)

Champ booléen de la définition d'un type d'agent dans le registre (défaut faux).
Ouvert, il retient en base du daemon les messages ordinaires destinés à un agent de
ce type tant qu'il est occupé (règle complète : `research.md` R4). Les fils T3 ne
sont jamais concernés.

La rétention n'est active que pour une instance qui a déclaré un signal de tour
véridique (`turn_attest` à l'enregistrement). Sinon, chemin actuel.

Ce que voit l'expéditeur : rien ne change. Son envoi est accepté. Sa remise reste
« en cours » jusqu'à l'injection. Il peut recevoir un refus net `queue_full` si la
file retenue atteint `queue_capacity` du type.

## Messages jamais retenus

Origine humaine ou système, `intent` `steer` ou `interrupt`, alertes de fil,
observations, incidents délégués, missions natives, fils T3, types sans réglage.

## `bridget_queue list` (156, étendu)

| Champ d'entrée | Valeurs |
|---|---|
| `state` | `held` (retenu, éditable) ou `released` (libéré au porteur, non éditable) |
| `position` | rang dans l'ordre de libération pour `held` |

Lecture de la base, sans aller-retour vers le porteur. Les autres champs
(`message_id`, `from`, `intent`, `reply`, `age_secs`, `preview`) sont ceux de 156.
`queue_bound` vaut `queue_capacity` du type quand le réglage est ouvert, `null`
sinon.

## `bridget_queue` - actions d'écriture (lot B)

Arguments communs : `target` (nom, UUID ou préfixe, comme `bridget_send`), `action`,
`request_id` (obligatoire, texte non vide).

| `action` | Arguments | Résultats |
|---|---|---|
| `cancel` | `message_id` | `applied`, `not_held` |
| `reorder` | `message_id`, `before_message_id` (texte ou `null` = fin) | `applied`, `not_held` |
| `promote` | `message_id` | `applied`, `not_held`, `no_active_turn`, `steer_unavailable` |

Le résultat est toujours final : jamais d'état intermédiaire ni `outcome_unknown`.
`promote` ajoute `effect: "unconfirmed"`.

### `cancel`

Une transaction : phase `cancelled`, l'envoi d'origine devient
`Rejected(cancelled_in_queue)`, notice à l'auteur, suivi de réponse clos, reçu. Le
message n'est plus jamais livré.

### `reorder`

Place `message_id` juste avant `before_message_id`, ou à la fin. Les deux messages
doivent être retenus, pour le même destinataire. L'ordre de libération suit.

### `promote`

Libère le message tout de suite en guidage, dans le tour en cours (demande encore
ouverte et échéance non passée, vérifiées dans la transaction). Le transport qui
ne peut pas guider l'enfile en file ordinaire. **Garanties réduites** : après la
libération, file ordinaire pleine = rejet explicite (remise indéterminée) ; erreur
ambiguë de Codex après guidage appliqué = doublon possible. Refus `no_active_turn` si le
destinataire n'est pas occupé, `steer_unavailable` si son porteur n'a pas déclaré
`queue_fallback`. Dans ces deux cas le message reste retenu.

## Rejeu et reçu

Reçu `queue_edits`, 7 jours, clé `(issuer scope, request_id)`.

| Rejeu | Résultat |
|---|---|
| mêmes paramètres | même résultat |
| autres paramètres | `envelope_mismatch` |
| refus déjà rendu | même refus ; pour réessayer, nouvelle clé |

## Refus

| Code | Quand |
|---|---|
| `intent_not_permitted` (motifs de 156) | appelant sans droit (matrice de `research.md` R12) |
| `referent_paused` | `promote` par un agent pendant la pause |
| `not_held` | message libéré, accusé, orphelin, indéterminé, déjà retiré, ou jamais retenu |
| `no_active_turn` | `promote` quand le destinataire n'est pas occupé |
| `steer_unavailable` | `promote` : porteur sans `queue_fallback` |
| `queue_full` | envoi vers une file retenue pleine |
| `envelope_mismatch` | clé rejouée avec d'autres paramètres |

Ordre : identité attestée, droits, reçu, `not_held`, pause, `no_active_turn`,
`steer_unavailable`, effet.

## Annonce de capacités

`bridget_capabilities` ajoute :

| Clé | Valeur |
|---|---|
| `queue.actions` | `["list","cancel","reorder","promote"]` |
| `queue.held` | `true` si le type de l'appelant ou de la cible a le réglage ouvert (par cible) |

## Protocole

- Capacité client `QueueEditV1` : sans elle, `CapabilityNotNegotiated` pour les écritures.
- `QueueEditRequest` / `QueueEditResult` : client et daemon.
- `Register` : champs optionnels `turn_attest` (lot A) et `queue_fallback` (lot B), défaut faux, absents des octets si faux.
- `TurnState` : le wrapper envoie l'état réel de sa session (après un refus de remise aussi) et le réattèste au plus toutes les 30 s.
- Trame de remise d'une promotion : référence `steer-or-queue` et `intent = SteerCurrent` dans la copie envoyée ; l'enveloppe stockée ne change pas.
- Les trames sont classées dans la matrice exhaustive de `daemon.rs` (aucun `_`).
- Aucune trame daemon vers porteur n'est ajoutée.

## Base de données

Migration v7 (voir `data-model.md`) : phases `held` et `cancelled`, colonne
`queue_rank`, table `queue_edits` (lot B). Elle se rejoue sans effet. **Sens
unique** : un binaire antérieur refuse la base. Sauvegarde obligatoire avant la
première ouverture ; la restauration avec l'ancien binaire est répétée en
isolation avant la livraison du lot A.

## Lineage T3 et pont

Aucun changement de forme de la projection Lineage. Le pont T3 n'est pas modifié.
