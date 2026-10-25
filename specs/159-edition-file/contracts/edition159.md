# Contrat - File retenue et édition 159

Compatible avec les contrats 148, 149, 150 et 156. Réglage `held_queue` fermé : un
appel et sa remise produisent les mêmes octets et le même comportement qu'avant.

## Réglage `held_queue` (lot A)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/159-edition-file/contracts/edition159.md#extrait-1).

La rétention n'est active que pour une instance qui a déclaré un signal de tour
véridique (`turn_attest` à l'enregistrement). Sinon, chemin actuel.

Hors refus d’admission `queue_full`, le résultat d’admission existant reste
conservé. La remise reste « en cours » jusqu’à son issue. Si la file retenue
atteint `queue_capacity` du type, l’admission rend un refus net `queue_full`.
Ce refus est distinct de la saturation de la FIFO après une promotion libérée.

## Messages jamais retenus

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/159-edition-file/contracts/edition159.md#extrait-2).

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

Libère le message tout de suite en guidage dans un tour actif recontrôlé. Ce tour
remplace le repos exigé pour la libération ordinaire. La transaction vérifie
l’identité attestée, les droits et gardes de `Steer` de 156, la phase `held`,
l’absence de réservation, la saga non expirée et, si `reply=true`, la demande
ouverte et son échéance. Elle pose la réservation de tour. Un agent est refusé
pendant la pause. L’humain attaché attesté garde l’exception de guidage de 156 ;
son identité vient de la connexion validée. State protège les faits du tour
jusqu’au commit. Après `BEGIN IMMEDIATE`, la même connexion relit les tables
`control_state` et `control_pause_interruptions`, la réservation, la saga et la
demande. Le commit précède le relâchement de State et le push par le writer ;
aucun writer socket ni autre connexion SQL sous cette transaction. La pause et la priorité de reprise
continuent à différer la libération automatique `HeldRelease`. Elles ne
suppriment pas cette commande humaine explicite ni ses autres gardes. Le transport qui
ne peut pas guider l'enfile en file ordinaire. **Garanties réduites** : après la
libération, file ordinaire pleine = rejet explicite (remise indéterminée) ; erreur
ambiguë de Codex après guidage appliqué = doublon possible. Refus `no_active_turn` si le
destinataire n'est pas occupé, `steer_unavailable` si son porteur n'a pas déclaré
`queue_fallback`. Dans ces deux cas le message reste retenu. Un tour perdu
après le commit déclenche le repli ; il ne rend pas admissible une promotion
initialement hors tour. `applied` atteste le commit de libération ;
`effect: "unconfirmed"` ne prouve pas une consommation par le modèle. Le daemon
ne refait pas l’effet sur rejeu et ne fabrique ni ACK ni reçu d’injection.

Le transport reconnaît le marqueur avant ses refus de `SteerCurrent`. Une copie
marquée qui ne peut guider, y compris en interface interactive ou avec
`steering_open` fermé, rejoint la FIFO ordinaire. Une copie non marquée garde le
comportement actuel. Pour le repli de la copie marquée, `steering_requested`
et `interrupt_requested` deviennent tous deux faux avant les refus de contrôle.
Un message humain en tour actif ne déclenche donc pas un interrupt implicite.
La saturation teste `queue.messages`. La FIFO pleine garde son rejet explicite.

`applied` n’est pas démenti par l’issue de transport ultérieure : son objet est
le commit d’édition. Rejet ou `indeterminate` n’avertissent pas universellement
l’auteur d’un envoi ordinaire `reply=false`. Les avis existants restent ceux
des chemins attach et demande suivie. Aucune nouvelle notification ni lecture
d’état n’est promise. Le retrait demandé conserve sa notice durable existante.

## Rejeu et reçu

Reçu `queue_edits`, 7 jours, clé `(issuer scope, request_id)`.

| Rejeu | Résultat |
|---|---|
| mêmes paramètres, droits frais admis | même résultat |
| autres paramètres, droits frais admis | `envelope_mismatch` |
| refus déjà rendu, droits frais admis | même refus ; pour réessayer, nouvelle clé |

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

Les droits restent frais avant toute lecture du reçu. Si l’auteur ne peut
plus être vérifié parce que l’envoi d’origine a été purgé ou que message_bytes
est absent, son seul statut d’auteur ne permet plus cancel : refus
`intent_not_permitted` avant le reçu. `receipt.actor` est une trace, pas un
grant persistant. Les autres droits exigent toujours leur propre preuve fraîche.
Le reçu ne contourne donc jamais l’ordre identité → droits → reçu.

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

La migration v7 du lotA est déjà livrée : phases `held` et `cancelled`, rang,
rattachement et réservation (voir `data-model.md`). Ses preuves de sauvegarde
et restauration restent celles de159A. Le lotB crée `queue_edits` et ses index
par `CREATE IF NOT EXISTS` dans `IdempotencyStore::init_schema`, exécuté à chaque
ouverture même si la base est déjàv7. Le DDL reste hors branche de migrationv7.
Les cinq oracles T014 existants partent aussi d’une basev7 sans table, la
réouvrent puis appliquent la première édition. Pas de v8, reconstruction
ou nouvelle sauvegarde de migration. Le précontrôle livré accepte7 et refuse8 ;
un binaire antérieur à159A continue à refuser la basev7.

## Lineage adaptateur de bureau et pont

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/159-edition-file/contracts/edition159.md#extrait-3).


### Résolution déterministe du message public — arbitrage ROOT B

Le message_id public n'est pas unique entre plusieurs issuer_scopes pour une
même cible : la clé durable reste (issuer_scope, send, message_id). Chaque
mutation résout uniquement une remise held de la cible, avec au plus deux
correspondances lues (LIMIT 2). Une seule correspondance permet l'édition.
Zéro correspondance ou ambiguïté : reçu refused/not_held, delivery_id=NULL ;
aucun UUID interne fabriqué. Les arguments publics restent dans le canon.
Cet arbitrage ne crée ni index, identifiant, scénario ou autorité supplémentaire.
Les fondations validées ne prouvent pas encore les trois chemins de mutation.


### Promotion et reprise de pause — précision T021

Une réservation de tour déjà prise, une saga expirée ou une demande close
rendent `not_held`. Une ligne de `control_pause_interruptions` pour la cible
avec `resumed_at IS NULL` bloque la promotion par un agent avec
`referent_paused`, même lorsque `control_state.paused` est faux. L'humain
Attach attesté conserve son exception explicite. Cette commande humaine
n'est pas une libération autonome `HeldRelease`. Les deux tables sont relues
dans la même transaction ; aucun droit ni identifiant de test supplémentaire.
