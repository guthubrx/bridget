# ADR 043 - Avis d'observation amortis, abonnements repris après redémarrage

- **Date** : 2026-09-25
- **Statut** : Accepté
- **Sessions** : 119, 122 (amende le contrat d'observation de l'ADR 037)

## Contexte

Chaque avis d'observation est remis comme un message et démarre un tour payant chez son
destinataire. Deux règles de la session 101 ont coûté cher le 25/09 :

- le daemon avertissait l'abonné à chaque changement du nombre de sources : une source qui
  clignotait a produit 222 avis et 197 tours en trois heures chez un coordinateur ;
- après un redémarrage du daemon, les abonnements passaient `interrupted` et exigeaient un nouvel
  abonnement explicite : à chaque relance, le coordinateur dépensait des tours à se réabonner, et
  restait aveugle tant qu'il ne l'avait pas fait.

## Décision

1. L'état d'une source change aussitôt dans `events list`, mais l'avis n'est remis qu'après 30 s de
   stabilité ; un aller-retour vers l'état déjà annoncé est muet ; une source qui ne se stabilise
   pas donne au plus un avis `source instable` par 5 minutes, avec le nombre de bascules.
2. Au démarrage, les abonnements non expirés de l'instantané sont repris : `source_unavailable`
   jusqu'au retour de leur source, puis `active`. Le propriétaire reçoit un seul avertissement à
   son retour ; les faits de la coupure sont perdus et le disent.

## Conséquences

- Positives : plus de réveils en rafale ; plus de réabonnement après chaque livraison du daemon.
- Négatives : une interruption de moins de 30 s n'est jamais annoncée (elle reste visible dans
  `events list` et dans les compteurs de lacunes) ; un abonné n'a plus à confirmer la reprise, il
  doit donc lire l'avertissement pour savoir qu'une période n'a pas été observée.
