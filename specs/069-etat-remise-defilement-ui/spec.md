# SPEC-069 - Etat de remise honnête et défilement de conversation

**Statut** : Terminé

## Problème constaté

Le 30 août 2026, un message humain a été accepté à 08:54:06 CEST puis la première sortie de l'agent a été observée à 08:55:19 CEST. L'interface est restée sur « envoi accepté · remise en cours », alors que la remise était acquittée : le message avait rejoint un tour Codex déjà actif, dont l'identifiant de journal diffère de celui de la livraison humaine.

Pendant la reprise du journal, les fragments du tour actif peuvent aussi être conservés hors écran jusqu'à la fin du rattrapage. Un rechargement rend alors la réponse visible, mais pas l'interface déjà ouverte.

## Objectif et exigences

- Après acceptation HTTP : « envoi accepté · confirmation en attente », jamais une remise en cours non prouvée.
- Quand le fil durable contient le `delivery_id` : « remis à l'agent ».
- Les sorties ultérieures du tour actif restent visibles pendant la reprise du journal, même si le tour a un autre identifiant racine.
- Au bas du fil, les nouvelles sorties font suivre le défilement. Hors du bas, elles ne déplacent pas la lecture et la puce de nouveaux messages reste affichée jusqu'à son ouverture.
- La correction reste confinée au client UI et ne modifie ni le protocole de remise ni le daemon.

## Hors périmètre
