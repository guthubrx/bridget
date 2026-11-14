# Restrictions de guidage

Le signalement mcp-92249-6af5f3fb-33 relève deux restrictions absentes de la description : steer refuse reply=true avant l’envoi métier ; un agent ordinaire ne peut pas guider un pair avec steer ou interrupt. La garde conserve les cas humains attachés et boucles autorisées. Le texte distingue ces cas et conseille queue/reply=true pour une demande à un pair, sans promettre une remise pendant son tour.

Changement de descriptions seulement : aucun schéma de type, canon, code d’erreur, droit ou parcours modifié. Le refus est attendu ; le défaut confirmé concerne l’aide. Le catalogue déjà chargé garde son texte jusqu’au rechargement. L’activation est volontairement non annoncée : le parcours ponctuel de redémarrage du connecteur reste suspendu après l’incident a716ff89. Aucun nouveau redémarrage, aucune mission de test et aucun message métier rejoué.

Qualification exécutée sur le runtime cumulatif Linux par scripts/build.py : tests existants guidance156_mcp_ (description, canon et refus local) et guidance156_matrix (autorité parent, pairs et exceptions). Aucun test ajouté pour recopier les nouvelles chaînes. Revue directe du delta de trois descriptions ; aucune modification du refus ni de son traitement.

Résultat : 6 tests MCP et 3 matrices de permissions réussis. Le filtre initial guidance_tests sélectionnait zéro test ; il a été remplacé par guidance156_matrix et ne compte pas comme preuve. Compilation de tests Linux par scripts/build.py réussie. Aucun binaire release construit ou activé pour ce lot de texte. Preuves : /Users/moi/.cache/bridget289-aide-guidage.

Le 14 novembre, le signalement mcp-85484-6af8236d-1a confirme le même refus pour un chef d'orchestre vers un rôle persistant pair. Lecture de guidance.rs : le rôle déclaré ne remplace pas actor_is_parent ; les exceptions de contrôle restent explicites. La référence française précise le parcours queue ou fil partagé, le délai incluant la file et le recours humain si une intervention pendant le tour est indispensable. Documentation seule, sans nouvelle émission métier ni changement de droits ; aucune activation du lot 289 n'est déduite de cette mise à jour.
