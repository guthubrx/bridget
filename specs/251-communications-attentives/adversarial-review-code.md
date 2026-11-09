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

## Contre-revue R2
Tâche d8c6d4e3-4977-4adc-9e05-feca0f403ad2, source6334f58b, Claude Haiku SYS. Résultat7039octets lu/ACK ; APPROVE_WITH_CHANGES, aucunP1/P2 confirmé. Lecture statique uniquement, aucun test/build/Gitdiff/runtime revendiqué. Les deuxP2 R1 sont vérifiés corrigés. Les contrôles Mac10 et Linux9 sont des preuves auteur distinctes, pas des exécutions du relecteur.
P3 : DispatchUnavailable d'un enregistrement nonPrepared reçoit le guidage générique information_store_unavailable. Dans le chemin public, la réservation idempotente et le mutex précèdent la transaction ; un reçu terminal est relu avant l'appel. Le message ne prétend jamais un succès ni une révocation et demande de relire la même clé. Limite de précision conservée, sans modifier les binaires qualifiés pour un chemin d'état non démontré atteignable.
Destinataire absent : limite explicitement conservée ; aucune création d'identité ou relance automatique. Lecture volontaire par un modèle : elle ajoute les fragments au contexte ; le dépôt lui-même ne le fait pas.
