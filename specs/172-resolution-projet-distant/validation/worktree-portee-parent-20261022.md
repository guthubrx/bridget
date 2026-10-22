# Portée du parent et worktree imbriqué — 22 octobre 2026

## Fait observé

La tâche c67ba253-18a7-4c46-958a-ab1f4b60a242 a créé son worktree sur Cartae, puis reçu remote_workspace_prepare:PermissionsChanged. Aucun child_instance, lancement ou remise métier n’est enregistré. Le parent est attesté à la racine Opus2D, tandis que la demande vise son worktree lié 158-monuments-realistes.

## Cause et correction

Après création dans un worktree déjà lié, remap_prepared_worktree réduisait source_root au checkout parent et déplaçait remote_root vers le nouveau checkout. Le deuxième contrôle refusait alors le cwd attesté à la racine, devenu extérieur à cette correspondance réduite.

Un worktree contenu conserve maintenant la correspondance administrative d’origine. Seul son dossier d’exécution change. Le contrôle de confinement dans remote_root, les chemins protégés, l’identité Git et les contrôles de permissions restent actifs. Le cas où la correspondance commence dans un sous-dossier du checkout conserve son traitement existant.

## Référence T3 lue

/Users/moi/11.Repositories/t3code/apps/server/src/vcs/GitVcsDriverCore.ts : createWorktree crée un checkout depuis input.cwd sans remplacer l’identité du projet.

/Users/moi/11.Repositories/t3code/apps/server/src/ws.ts : la résolution du dossier choisit thread.worktreePath ou project.workspaceRoot. Bridget reprend cette séparation entre projet autorisé et dossier d’exécution. La projection entre machines et ses contrôles de permissions restent propres à Bridget.

## Validation ciblée

- RED : le scénario parent à la racine + demande depuis le checkout lié échoue sur PermissionsChanged avec l’ancienne production.
- GREEN final : spec172_linked_checkout_prepare_preserves_main, un test avec sept combinaisons. Il couvre notamment un parent à la racine vers Codex, puis vers Claude natif simulé, ainsi qu’un parent extérieur refusé avant création. Le recheck passe et le checkout principal reste inchangé.
- Régression : worker162_prepare_worktree_rejoue_et_rapport_preserve_modifications, un test PASS.
- Total final : deux tests distincts PASS. Les relances intermédiaires ne sont pas additionnées. Aucun fournisseur, tour métier ou lot Opus2D n’est lancé par ces tests. Aucune suite complète.

Preuves privées : /home/moi/.cache/bridget-worktree-scope172/red.json et /home/moi/.cache/bridget-worktree-scope172/tests-final.json.

## État de livraison à ce checkpoint

Correctif source 39a9df21e925ad54b904a2b3c2078eba9f20ef17 intégré et poussé sur GitHub/main. Worker Cartae actif : PID 2828136, build 172-39a9df21, SHA-256 6e7e4fa39117702594fc6b14303ccd0661a5fcd1cf6304793a1644db717cffcb.

La compilation optimisée a échoué sur ENOSPC. Livraison immédiate demandée par l’utilisateur : binaire de dépannage non optimisé, sans symboles de débogage (profil dev-debug0). La revue indépendante e5959986-e426-422d-956f-e035dbcf89be n’a pas fourni de verdict, son démarrage ayant échoué ; son arrêt a été demandé. Aucun APPROVE n’est revendiqué.

Le seul worker Cartae a été relancé par SIGTERM individuel, avec KillMode=process. Aucun wrapper géré n’était actif au préflight. T3 et le maître Mac n’ont pas été relancés. L’admission c67ba253 n’est ni rejouée ni réparée par mutation de base. Son reçu et son worktree restent conservés. La reprise métier appartient à son responsable après clôture officielle de l’ancienne admission, avec une nouvelle demande à jour.

Reçu privé : /home/moi/.cache/bridget-worktree-scope172/activation.json.
