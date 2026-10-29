# Validation204

## Cause réelle
La tâche4fb1a401 a été remise le29octobre02:19:02UTC et a échoué à02:26:43UTC. Session fournisseur01a18573-f1a5-7832-a17a-d284d1a80cff, instance22bd7691-ad09-43a1-907a-cc22c68a9607/g1329. Le task_complete fournisseur atteste server_overloaded : Selected model is at capacity. Le schéma généré par le programme installé0.164.0 confirme le champ de fil codexErrorInfo et sa valeur serverOverloaded. Le pilote Bridget ne gardait que turn.status=failed. Stderr vide ; reçu worker terminal et nettoyage maître terminé, dossier dirty conservé. Aucun échec de placement, aucune nouvelle exécution ni mutation de cette tâche.

## Correction et limites
Projection d’un seul code fermé, seulement pour un terminal failed avec code attesté. Texte libre fournisseur et additionalDetails exclus. Succès, interruption et causes inconnues gardent leur contrat. Le statut précise la saturation du modèle, la conservation du travail et les préconditions de suite ; aucune reprise de failed ni substitution automatique. Les anciens échecs restent inconnus sans trace corrélée. Ce correctif améliore le diagnostic ; il ne supprime pas la saturation fournisseur.

## Contrôles
Trois tests204 PASS : code et statut attestés, champs libres exclus, vrai parser transport et journal jusqu’au terminal, statut sans mutation et attente du nettoyage. Commande via scripts/build.py avec cache commun et -j2. Compilation et revue indépendantes encore à réaliser. Aucun binaire204 chargé à ce stade. Propriétaire informé de la cause exacte via message mcp-50239-6ae2b8a9-1, accepté in_flight ; lecture non déduite.
