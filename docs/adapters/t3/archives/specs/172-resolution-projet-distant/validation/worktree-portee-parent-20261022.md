# Archives techniques de l’adaptateur T3

Document d’origine : [specs/172-resolution-projet-distant/validation/worktree-portee-parent-20261022.md](../../../../../../../specs/172-resolution-projet-distant/validation/worktree-portee-parent-20261022.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

/Users/moi/11.Repositories/t3code/apps/server/src/vcs/GitVcsDriverCore.ts : createWorktree crée un checkout depuis input.cwd sans remplacer l’identité du projet.

<a id="extrait-2"></a>

## Extrait 2

/Users/moi/11.Repositories/t3code/apps/server/src/ws.ts : la résolution du dossier choisit thread.worktreePath ou project.workspaceRoot. Bridget reprend cette séparation entre projet autorisé et dossier d’exécution. La projection entre machines et ses contrôles de permissions restent propres à Bridget.

<a id="extrait-3"></a>

## Extrait 3

Le seul worker Cartae a été relancé par SIGTERM individuel, avec KillMode=process. Aucun wrapper géré n’était actif au préflight. T3 et le maître Mac n’ont pas été relancés. L’admission c67ba253 n’est ni rejouée ni réparée par mutation de base. Son reçu et son worktree restent conservés. La reprise métier appartient à son responsable après clôture officielle de l’ancienne admission, avec une nouvelle demande à jour.

<a id="extrait-4"></a>

## Extrait 4

Seul le worker a reçu un SIGTERM individuel. Aucun wrapper géré ni reçu de commande actif au contrôle précédant l’arrêt. T3 et le maître Mac sont conservés. Aucun test supplémentaire ni mission métier n’est lancé par l’intégrateur.
