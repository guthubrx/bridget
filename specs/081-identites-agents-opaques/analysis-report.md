# Analyse de cohérence - SPEC-081

Statut : PASS avec ajustement

## Vérifications

- Spec : modèle limité à agent_id/display_name pour les agents.
- Plan : réutilise AgentProfileStore au lieu d'un nouveau registre.
- Audit : confirme le réemploi de Router, AgentInfo, stores et migration Maicie.
- Tâches : couvrent contrat, runtime, UI, Maicie, migration, tests et convergence.

## Ajustement appliqué

La formulation initiale « transaction atomique Bridget et Maicie » était techniquement impossible car la migration touche plusieurs SQLite et fleet.json. Le plan et l'audit précisent désormais la garantie réelle : préflight complet, sauvegarde préalable, transaction par ressource, journal de progression et refus de redémarrage incomplet.

## Risques résiduels

- Le changement de vocabulaire est transversal et le compilateur Rust devra guider les appels restants.
- Les données déjà supprimées sans correspondance d'identité ne peuvent pas être reconstituées honnêtement : elles doivent passer requires_retarget.
- Il faut vérifier l'existence d'un agent non-Codex joignable pour la contre-revue adverse.
