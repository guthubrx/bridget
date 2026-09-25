# Journal 122 - Abonnements repris après redémarrage

- **Base** : main `9a9a2b89` (après 121) - **Date** : 2026-09-25 - **Statut** : Implemented, livré 08:13

## Livraison
- 06:13:27Z : relance du daemon (build `2595241421c1`), puis du pont ; 31 agents avant et après ;
  27 sources, aucune sans événement (les workers Claude sont observables grâce à la 121).
- `sol_city_ai` : quatre avis « abonnement repris automatiquement », aucun réabonnement. Un
  abonnement a reçu « source indisponible » puis « active » : sa source a mis plus de 30 s à revenir
  car le pont a été relancé juste après le daemon ; comportement voulu (119).
- L'instantané écrit par l'ancien daemon a été repris tel quel (format inchangé).
