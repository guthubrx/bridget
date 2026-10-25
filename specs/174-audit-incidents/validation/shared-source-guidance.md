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
de l’utilisateur ; elle reste conservée. Les quatre tests ciblés passent à nouveau après ces précisions.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/174-audit-incidents/validation/shared-source-guidance.md#extrait-1).

Circuit réel : appel MCP shared avec cwd volontairement absent et destination
distante. Le refus contient le message et le conseil complets. Les lectures
ciblées des deux tables durables avant et après donnent zéro reçu d’admission
et zéro tâche pour cette seule clé. Aucun modèle ni mission métier lancé par
ce contrôle. Le conseil couvre aussi le cas général d’une admission déjà en
cours, sans déduire sa clôture de l’absence d’identifiant.

Résultat envoyé au demandeur par Bridget ; remise en vol, lecture non confirmée.
La sauvegarde du binaire précédent est conservée. Le wrapper a effectué
l’entretien du cache, puis la commande installed a confirmé cet entretien.
Aucun build worker nécessaire : cette projection est produite par le maître
et les clients existants rendent déjà detail. Aucun protocole ne change.
