# Guidage du contrôle source shared — 24 octobre 2026

Défaut : `workspace_base_preflight_failed`, produit par le contrôle source shared
ou implicite, échappait au guidage des erreurs Git. Un chemin distant envoyé
par un parent local renvoyait un code brut. L’incident est contextuel ; le conseil
manquant est un défaut de Bridget.

Correction : projection constante dans `detail.message` et `detail.recovery`.
Le code reste identique. Les suffixes système/Git ne sont pas exposés. Le conseil
sépare parent/destination, HEAD shared/commit demandé, mode imposé/worktree
autorisé, rejeu strict/demande modifiée. Les contrôles, droits, reçus et tâches
ne changent pas. Aucun fetch ni mission métier automatique.

Référence T3 lue : `GitVcsDriverCore.ts`, `resolveCommit` et
`fetchPullRequestHeadCommit`. Résolution dans le cwd fourni, récupération explicite
avant résolution ; distinction conservée. Bridget doit en plus expliquer l’hôte
source et l’identité durable des demandes distantes. Aucun déplacement forcé
de branche n’est repris.

Validation ciblée : trois tests `spec174_workspace_` réussis et un test socket
MCP réussi. Le nouveau cas passe par le vrai contrôle d’un cwd absent, en shared
et sans mode explicite, puis vérifie la projection du refus. Les suffixes privés
et les codes voisins sont couverts. Le socket vérifie le transport du conseil,
avec une réponse simulée. Documentation : 27 pages, 422 liens et 11 paires EN/FR.
Aucune suite complète ni mission applicative lancée.

Premier lancement non concluant : syntaxe de fixture corrigée ; le cache partagé
contenait aussi des métadonnées d’une autre branche. Les sources du composant
transport ont été réhorodatées pour le reconstruire dans le même cache, sans
modification de contenu. Les quatre tests ci-dessus passent ensuite.

La règle permanente « défaut de guidage confirmé = correction et livraison »
est conservée dans AGENTS.md. Revue indépendante lue et acquittée : deux précisions reprises avant activation.
Le conseil fetch a été retiré de ce code d’accès source. Une admission déjà
choisie peut rester en attente sans TaskID : la nouvelle clé exige donc de
confirmer aussi l’absence d’admission en cours. Le test vérifie que ce code
reste non terminal. La proposition de la revue fondée sur la seule absence de
tâche n’a pas été reprise. La règle AGENTS correspond à l’autorisation explicite
de l’utilisateur ; elle reste conservée. Livraison à compléter.
