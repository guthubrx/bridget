# Correctif durable de l’identité — complément 149

Date : 2026-10-11. Statut : validation et livraison en cours.

Cause observée : Codex et Claude excluaient toute configuration Bridget existante. Le MCP Codex de ce fil ne recevait donc aucune preuve privée de session.
Le montage officiel reçoit maintenant une preuve fraîche à chaque création/reprise. Les configurations désactivées, étrangères et les identités natives explicites restent protégées.
Le chemin réel du binaire et son environnement canonique sont vérifiés. Claude exige un catalogue connecté de source `dynamic`.
Cause distincte : le pont rejetait les `%` présents dans les identifiants opaques des sous-agents T3. Chaque identifiant est maintenant encodé en un segment HTTP, sans prédécodage.

La revue Sonnet R1 a relevé deux problèmes de montage, corrigés. La revue R2 approuve le code final sans finding bloquant. Les derniers changements sont uniquement de format.
Les résultats des tests et leurs limites sont consignés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/identite-durable-tests149.md.
Le paquet privé et ses preuves sont préparés dans /Users/moi/.cache/t3-identite149-final.OBXQrQ.

Activation : le serveur T3 chargé ne recharge pas son JavaScript après remplacement des fichiers. Une relance T3 reste nécessaire.
La commande native `provider-session.detach` ne suffit pas à actualiser le code du serveur.
Aucune preuve de réussite depuis le MCP de ce fil n’est déclarée avant activation et délégation native réelle.
