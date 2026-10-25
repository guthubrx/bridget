# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/159-file-retenue-et-edition.md](../../../../../decisions/159-file-retenue-et-edition.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

La remise durable099 et sa saga existent dans le daemon. Retenir un message
dans une file de porteur rend les retraits et changements d'ordre répartis
entre processus. Le lot A place la rétention dans send_deliveries, avant toute
poussée. Le lot B d'édition et les fils T3 restent distincts et différés.

<a id="extrait-2"></a>

## Extrait 2

La table `execution_queue` d’ExecutionStore est écartée : elle porte des
exécutions, sans échéance de remise, instance ni lien à la saga d’envoi. Sa
connexion distincte ne devient pas un second writer sous cette transaction.
La conception antérieure d’édition dans les files mémoire des porteurs
présentait onze défauts liés au partage d’état entre processus (research R2). Le découpage A/B conserve l'envoi fermé d'A et
limite B aux trois actions. `edit`, les fils T3 et 159c restent différés.

<a id="extrait-3"></a>

## Extrait 3

Les droits restent frais avant toute lecture du reçu. Si l’auteur ne peut
plus être vérifié parce que l’envoi d’origine a été purgé ou que message_bytes
est absent, son seul statut d’auteur ne permet plus cancel : refus
`intent_not_permitted` avant le reçu. `receipt.actor` est une trace, pas un
grant persistant. Les autres droits exigent toujours leur propre preuve fraîche.
Le réglage held reste fermé par défaut. Les preuves ciblées des mutations,
transports et façades sont consignées dans :
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/validation/tests-159.md
Les coûts B non mesurés ne deviennent pas une latence promise. Le FULL, les
replays, la revue et l’audit ont des périmètres distincts ; leurs reçus conservent
les limites. La livraison B ne se déduit pas d’un PASS ciblé.
La recette SC010 est privée : ses marqueurs enfant149 et PID/naissance ne
prouvent pas une interopérabilité T3 réelle. La reprise E2E cancel emploie une
pause explicite ; SC007 commence au commit Applied, sans preuve de retry
gagné contre une libération non pausée.
Choix des trois questions : destinataire sans droit d’édition ; held fermé
par défaut ; modification du corps différée. Aucun choix supplémentaire exigé.
