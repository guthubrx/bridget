# Limite de l’annuaire MCP

Signalement mcp-86090-6aff4989-3. Appel réel bridget_who, scope same_project,
limit 200. Le client retourne isError true, aucune structuredContent et
l'erreur JSON-RPC -32602. Le texte chargé commence par tool call error.
Le plafond 100 est exposé dans inputSchema.maximum mais pas dans la
description visible du catalogue MCP en session. La pagination par octets
reste indépendante de la limite de nombre de lignes.

Hypothèse vérifiable. Le client perd le résultat structuré parce que la
validation de limit retourne InvalidParams à la couche JSON-RPC. Un refus
d'outil status error doit utiliser le résultat MCP normal avec isError true,
le code stable, les deux bornes et un texte JSON équivalent.

Correction. Les bornes sont partagées entre le parseur et le schéma. La
description de l'outil nomme la borne et le code who_limit_invalid. Une
valeur invalide rend isError true, structuredContent avec status error,
code, field, minimum, maximum et retryable false. Le texte contient le même
objet JSON. La validation n'envoie pas de requête annuaire. Les autres
erreurs d'arguments gardent leur réponse JSON-RPC. Les pages restent
limitées à 100 présences demandées et 24 Kio.

Le test du refus passe pour 200, 0, une chaîne et null. Le premier montage
de test s'arrêtait au contrôle de portée faute de socket de fixture. Le
test final exerce l'exécuteur d'annuaire et le répartiteur JSON-RPC sans
connexion à un service. Un appel réel limit 100 rend une page structurée.

Les 11 tests ciblés de l'annuaire et Clippy strict passent. La suite MCP
élargie donne 178 succès, un ignoré et un échec dans
native153_mcp_continue_catalogue_et_champs_fermes. Son assertion compare
le texte exact champ inconnu pour delegate et continue. Le diagnostic
détaillé a été introduit par 642d8f4b, hors du chemin who corrigé ici.

Revue native en lecture seule. Clé bridget-304-annuaire-limite-revue-20261119-r1
et tâche 4123d4d8-8372-49dc-b2e0-2b21d77c9a86. Aucun défaut trouvé dans le
périmètre, résultat lu. La revue n'a pas exécuté de tests.

Activation toujours refusée par le maître avec
update_missions_active_or_unconfirmed. Aucun bail ni arrêt de service.
Le test du contrat MCP est disponible sur les sources. Le client actif
garde la réponse historique jusqu'au chargement d'une version corrigée.
Cache de build à 9,61 Gio, sous le plafond de 10 Gio. Aucun worktree créé.
