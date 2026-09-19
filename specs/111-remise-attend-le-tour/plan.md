# Plan 111

## Décision

Dans `crates/bridget-daemon/src/t3code.rs`, retirer la condition `received.elapsed() >= turn_wait`
des deux points de filtrage de la file. Les autres motifs d'écartement, tous fondés sur une échéance
réellement portée par le message, restent inchangés.

Le délai servait de garde-fou mémoire. Il est remplacé par une borne explicite en nombre :
`QUEUE_BOUND`, appliquée à l'insertion. Un dépassement écarte la plus ancienne remise avec un
avertissement distinct, pour ne pas confondre saturation et péremption.

`turn_wait` reste employé par le cache des annulations, dont la sémantique est différente.

## Pourquoi ne pas simplement allonger le délai

Allonger `turn_wait` déplace le seuil sans le supprimer : un tour plus long que le nouveau seuil
perdrait encore ses messages, et le symptôme serait plus rare donc plus difficile à diagnostiquer.
Un message sans échéance n'a aucune raison de périmer : c'est à l'expéditeur de déclarer une
échéance s'il en veut une, et le protocole le permet déjà.

## Tests

Unitaires dans `t3code.rs` : une remise sans échéance survit à une attente supérieure à l'ancien
délai ; une remise avec échéance dépassée est écartée ; la borne de file écarte la plus ancienne.

## Livraison

Recette complète, reconstruction, relance du daemon et du pont, puis contrôle des trois remises en
attente et du journal.
