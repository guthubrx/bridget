# Plan 111

## Décision

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/111-remise-attend-le-tour/plan.md#extrait-1).

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

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/111-remise-attend-le-tour/plan.md#extrait-2).

## Livraison

Recette complète, reconstruction, relance du daemon et du pont, puis contrôle des trois remises en
attente et du journal.
