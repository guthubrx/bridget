# Recherche et décisions — Session 175
## T3 lu avant choix
- apps/server/src/vcs/GitVcsDriverCore.ts:2870 : création Git directe dans worktreesDir, sans plafond applicatif trouvé.
- apps/web/src/hooks/useThreadActions.ts:350-445 : lors de la suppression, proposition de retrait du worktree uniquement si non partagé, puis force:true après confirmation humaine.
- apps/web/src/worktreeCleanup.ts : détection du partage par chemin parmi les conversations connues.
- apps/server/src/orchestration/Layers/ProviderCommandReactor.ts:487-530 : réparation d'un dossier disparu à partir de la branche et prune des inscriptions périmées.
Lecture dans la copie T3 locale le 2026-10-23. Le schéma conversation → espace inspire la restitution de clôture. Le retrait forcé n'est pas repris. La recréation depuis une branche n'est pas compatible par défaut avec une mission distante à base exacte.
## Bridget existant
task_view dispose déjà des faits utiles ; les worktrees sont conservés à la fin et les rapports sont des instantanés. WorkspaceRecord est deny_unknown_fields, donc ajouter des champs persistés ferait intervenir la compatibilité interversions : rejeté.
Le nettoyeur connaît déjà propreté, fichiers ignorés, ascendance, ancienneté et processus. Son absence d'explications est la friction traitée. Le noyau CLI refuse intentionnellement cleanup : l'inventaire demeure dans l'outil d'entretien.
## Sources premières
https://git-scm.com/docs/git-worktree : list --porcelain -z, remove, lock et prune. Prune supprime des inscriptions périmées et ne remplace pas le retrait d'un espace existant. Référence consultée le 2026-10-23.
Baselines utilisateur 03-cognitive-load-productivity et 08-testing-quality consultées : retours compréhensibles et tests représentatifs. Aucune métrique de productivité externe ni pourcentage de fiabilité déduit.
## Arbitrages
- Réutiliser la projection et le nettoyeur ; aucune nouvelle table, dépendance ou commande du noyau.
- Conserver la capacité 128 et le seuil préventif 96 déjà documenté.
- Pas de partage automatique : une posture de lecture ne suffit pas à autoriser une base différente ou un espace partagé.
- Pas de migration de rangement : chemins/permissions/tâches en cours préservés.
- Inventaire inconnu ou incomplet = observation bloquée, jamais candidat sûr.
- Les primitives SpecKit sont lues comme skills ; les scripts .specify/scripts/bash sont absents sur Mac et Cartae malgré sync. Exécution manuelle de leurs protocoles, artefacts complets et vérification explicite des prérequis.
