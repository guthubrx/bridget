# Revue adverse code251
R1 : task44290ee8-56ad-48ec-83d1-2fdda9184adc, Claude Haiku SYS, sourceexécutabledbbcecd0. Lecture statique, aucun test/build/runtime/Gitdiff revendiqué. Verdict APPROVE_WITH_CHANGES, aucunP1, deuxP2 vérifiés/retenus. Résultat8942octets lu/ACK.
La demande originale avait d'abord échoué primary_absent puis binding_transport_unavailable. Clé et paramètres conservés, lookup idempotency_expired jamais interprété comme absence. Après activation/lien natif revenu, le rejeu strict a admis l'unique tâche44290 ; aucun remplaçant.
| Objection | Vérification et disposition |
|---|---|
| P2-1 gardes de remise sur information | Confirmée : CircuitBreaker::check lit l'historique normal saturable, même si information ne le remplit pas. T017 : ignorer ces gardes sans supprimer identité/projet/routage ; test sous saturation/dédup/quarantaine et zéro saut, mode normal toujours limité. |
| P2-2 collision et réessai inutile | Confirmée par le test de collision qui ne distinguait pas l'erreur. T018 : SELECT de collision dans Immediate, InformationCollision puis refus durable information_id_conflict ; corps antérieur conservé, même clé stable, guidage nouvelle demande après refus. |
| P3 portée parent/enfant native | Pas de droit nouveau : même relation prouvée existante que les messages normaux, contrôle native_task_exchange conservé ; aucun bypass ajouté par StoreOnly. |
| Ancien daemon enum inconnu | Négociation avant corps ; erreur de négociation ne prouve aucun Send. Aucune promesse de lecture. R1 du nouveau writer et lecteur250 réels ont conservé ledger/reçu sans remise. FormatR2 inchangé. |
| Résumés dormant non lus | Garde de disponibilité relue dans observer_return_tick ; garde FrozenWake existante conservée, pas de voie de StoreOnly vers le résumé. |
Les limites absence de destinataire et anciennes façades restent documentées, sans contournement. R2 sera relue sur le deltaT017/T018 ; pas de nouvelle campagne générale.
