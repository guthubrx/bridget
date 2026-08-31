# Plan - SPEC-069

Etendre `crates/bridget-daemon/assets/ui/app.js`, qui possède déjà l'état d'envoi, la projection du journal et la politique de défilement. Aucun changement de transport ou de daemon n'est requis : la remise est déjà attestée par le fil durable.

## Lots réversibles

1. Formaliser les états affichés de livraison et conserver une trace locale bornée des envois UI.
2. Coalescer un rendu de rattrapage uniquement pour un fil auquel l'utilisateur vient d'écrire, afin de ne pas revenir au rendu coûteux par fragment pour toute l'historique.
3. Préserver la puce de nouveaux messages lors d'un rendu sans nouvel incrément.
4. Exécuter les tests Node de l'asset et vérifier l'interface avant livraison.

## Garde-fous
