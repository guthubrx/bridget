# Contrat — Session 175
## Lecture parent
bridget_task_status conserve son enveloppe et ajoute workspace.retention pour une tâche avec worktree. Même contrôle d'identité, aucune mutation. Les vieux échanges worker et enregistrements restent inchangés.
## Inventaire opérateur
python3 scripts/build.py worktrees --repo <chemin absolu> [--base-ref main] [--json]
Commande strictement en lecture seule. Aucun build, cleanup, prune Git, branche ou agent fournisseur démarré. JSON stable version=1, observations inconnues explicites. Une erreur d'inventaire global sort en échec, sans stderr privé dans JSON.
La capacité vise le même cwd/.worktrees que le contrôle d'admission ; total du registre séparé. L'outil ne contacte pas les fournisseurs et ne présume pas qu'un fil arrêté n'est plus lié.

Précision capacité : observation des inscriptions Git dans le périmètre choisi ; les réservations en cours et dossiers non enregistrés restent contrôlés par l’admission, sans équivalence de total promise.

JSON d’inventaire : git_total inclut le principal ; scoped_count ne compte que les inscriptions sous le cwd/.worktrees choisi. Chaque entrée expose motifs et recovery. Le dernier n’est jamais une permission de retrait.
