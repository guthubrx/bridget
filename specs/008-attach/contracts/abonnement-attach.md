# Contrat : abonnement de vue (attach)

Extension applicative sur la socket daemon existante (locale et fédérée) —
aucun canal nouveau. Noms de variantes définitifs fixés au reuse-audit (style
007 : variantes typées, D-209). Révisé après contre-revue du plan (round 3) :
identité d'abonnement, sélecteur typé, marqueur de rattrapage, envoi humain sur
la connexion attach.

## Identité d'abonnement

Chaque abonnement porte un **`subscription_id` opaque**, attribué par le daemon
à la souscription et présent sur **tous** les messages du plan de contrôle
(événements, rattrapage, lacunes, fin). Sans lui, deux vues aux historiques
différents seraient indiscernables et une fin tardive d'un ancien abonnement
pourrait fermer le nouveau. Daemon et client ignorent tout message d'une
génération obsolète.

## Séquence nominale

1. **Subscribe** (client → daemon) : `{ agent, window }` où `window` est un
   sélecteur **typé** : `Today` | `Seq(n)` (inclusif : le premier événement
   relayé porte `seq == n`) | `Date(aaaa-mm-jj)`. La résolution du sélecteur en
   position concrète appartient au **wrapper** (seul lecteur du journal — le
   client ne peut convertir ni date ni « aujourd'hui »). **Référentiel
   temporel** : `Today` et `Date` s'entendent dans le fuseau et à la frontière
   de jour de **l'hôte du wrapper** (celui qui écrit le journal) — jamais ceux
   du client ni du daemon. Refus typés : date invalide, date future, date hors
   rétention (aucun journal correspondant). Le daemon attribue le
   `subscription_id`, le **propage au wrapper** avec la demande, et n'émet
   `Subscribed { subscription_id }` au client **qu'après acceptation du
   wrapper** — sinon refus motivé (agent inconnu, non-ACP, wrapper absent →
   message d'indisponibilité de la spec FR-007, ou canal de commandes du
   wrapper saturé → refus typé, jamais de perte silencieuse).
2. **Rejeu** (wrapper → daemon → client) : événements dans l'ordre de `seq`,
   lignes JSONL v1 telles quelles, portant `subscription_id`.
3. **Rattrapage** : `SnapshotCaughtUp { subscription_id, through_seq }` marque
   la fin du rejeu — l'état « à jour » devient observable et testable.
   `through_seq` est **optionnel** : absent quand la fenêtre ne contient aucun
   événement (journal vide ou pas encore créé) — le client affiche « en attente
   du premier événement ».
4. **Suivi** : mêmes événements, au fil de l'eau.
5. **Fin** (daemon → client) : `End { subscription_id, reason }` — wrapper
   déconnecté, équipier arrêté, désabonnement. Resynchronisation :
   `Subscribe { window: Seq(last_seq + 1) }` (nouvel id) **si au moins un
   événement a été reçu** ; sinon (`SnapshotCaughtUp` sans `through_seq`,
   aucun événement), le client se réabonne avec son **sélecteur initial
   mémorisé** — test imposé : snapshot vide → `End` → réabonnement → premier
   événement reçu exactement une fois.

## Fragmentation filaire (événements volumineux)

Contradiction fermée au round 4 : la ligne v1 est relayée « telle quelle » mais
peut être arbitrairement grosse, alors que la frame de transport est bornée.
Règle : un événement est découpé en **fragments** `{ subscription_id, seq,
offset, final }` de 256 Kio maximum chacun — la borne porte sur le fragment,
pas sur l'événement. `offset` est en **octets**, les fragments sont
**contigus depuis 0**. Le client assemble jusqu'à `final`, dans la limite
d'une **borne de réassemblage distincte** (défaut 4 Mio) : un `seq` qui la
dépasse est abandonné en `Gap { reason: event_too_large }`, les fragments
restants étant **consommés jusqu'à `final` sans boucle**. Un `Gap` sur un
`seq` invalide tout assemblage partiel de ce `seq`. Si **un** fragment est
lâché par le tampon, **tout le `seq` est abandonné** et comptabilisé au `Gap`
— jamais de fragment orphelin, jamais de rejeu qui retombe en boucle sur le
même événement. Fixtures imposées : événement > 256 Kio mais sous la borne de
réassemblage (rejeu **et** live, sans boucle) ; événement dépassant la borne
de réassemblage → `Gap(event_too_large)` propre.

## Lacunes d'affichage (tampon)

`Gap { subscription_id, from_seq, to_seq }` est un **compteur coalescé tenu
hors de la file** de la vue : il est émis avant le prochain événement conservé
et ne peut par construction être lâché lui-même. Jamais de trou silencieux.

## Envoi humain depuis la vue

L'envoi emprunte la **connexion attach persistante** (pas une connexion
`cli-send` éphémère : elle se ferme après l'accusé et ne peut pas recevoir un
rejet différé — `daemon.rs:1203-1207`) : `Send` ordinaire avec `message_id`
conservé par le client, puis issue corrélée retournée **sur cette même
connexion** : accusé, refus immédiat (`Nack` DND/arrêté/file pleine) ou rejet
différé (`DeliveryRejected`). Chemin unique, testé y compris le cas « accusé
puis rejet tardif ». Le message reste un message Bridget ordinaire (garde-fous
intacts).

## Règles

- L'ordre par `seq` est garanti par abonnement ; ligne illisible → événement
  d'erreur de lecture (numéro de ligne + offset, `seq` inconnu).
- Le wrapper cesse toute lecture de journal quand plus aucun abonnement n'est
  actif.
- Les événements de flux ne passent pas par le routage de messages (pas de
  dedup/disjoncteur/hops) — plan de contrôle, comme les variantes d'état 007.
- Cas de test imposés : deux vues aux fenêtres différentes sur le même agent
  (le rejeu de B n'atteint jamais A), réabonnement pendant qu'une fin de
  l'ancienne génération est en vol, rotation pendant le rejeu.
