**Verdict : APPROVE avec réserves. Je ne trouve aucun défaut P1 ou P2.** Les trois défauts décrits sont corrigés dans le code que j'ai lu. J'ai relevé 4 points P3 (mineurs, pas bloquants).

**Limites de la lecture.**
- Lecture statique de la vue du commit `aa3cbeb1`. Je n'ai rien compilé, aucun test lancé, aucun service ni runtime consulté.
- Je n'ai pas relu le corps des tests, ni `protocol.rs`, ni `workspace_commit_view_job.rs`.
- Je n'ai lu que `specs/234-inventaire-git/spec.md`, pas `plan.md` ni `tasks.md`.
- Dans `workspace_compact.rs`, j'ai lu le plan d'inventaire (l. 90-280) et `audit`, pas le reste.
- Je n'ai relu que `native_workspace_preparation210.rs` l. 55-137 (messages de refus), pas le reste du fichier.
- Pour `cli.rs`, `storage_compatibility.rs` et `delegation.rs`, seuls les passages cités sont lus.
- Les chiffres de l'incident (11 s, 69 commandes, 465 fichiers) viennent de votre message, je ne les ai pas revérifiés.

**Ce que j'ai vérifié**

1. **Budget d'inventaire** (`workspace_git.rs:30-33`, `387-397`, `452`, `workspace_compact.rs:114-125`, `workspace_commit_view.rs:267`).
   - `capture` applique `timeout.min(plafond)`, donc 60 s n'est qu'un plafond pour `ls-tree` et `ls-files`.
   - Les deux appelants d'inventaire passent `INVENTORY_TIMEOUT` explicitement. Les autres lectures gardent 10 s, `fetch` 20 s, `read-tree` et `worktree add` 300 s.
   - La liste blanche `allowed` n'accepte que `ls-tree --full-tree -r -l -z <sha>` et `ls-files -v -z`.
   - Le contrôle `--version` imbriqué consomme le même délai global.
   - Le plafond de 64 Mio sur stdout ne vaut que pour ces deux commandes.
2. **Lecture équitable** (`workspace_git.rs:436-484`). Chaque flux est lu par blocs de 8 Kio et le délai est vérifié à chaque tour. La pause de 1 ms n'a lieu que sans progression.
3. **Journal de délai fermé** (`:513-555`). Les libellés sont fermés, sans argv, chemin, SHA ni stderr. `stop` est ramené à `confirmed`, `not_spawned` ou `unconfirmed`. `GitTask234` est un garde RAII qui restaure l'ancienne valeur (`:37-43`).
4. **Constante partagée de réservation** (`workspace_commit_view.rs:1443`, `remote_worker.rs:554`). La borne du validateur est désormais `MAX_VIEW_RESERVATION_BYTES`, la même que celle de l'écriture.
5. **Récupération froide** (`remote_worker_capacity234.rs`).
   - Elle prend le verrou existant, exige l'inscription en quarantaine et la même identité, puis refait un inventaire complet.
   - Elle n'accepte que des commandes `Terminal` ou `Refused` et exige l'arrêt de tous les marqueurs, y compris les orphelins.
   - Elle exige un témoin légitime : réservation égale au maximum et supérieure à l'ancienne borne, non adoptée, sans tentative source ni marqueur.
   - Elle sauvegarde l'inscription, ne change que le drapeau de quarantaine, puis revérifie la santé. En cas d'échec, elle remet en quarantaine.
   - Un arrêt entre la sauvegarde et l'écriture laisse le magasin en quarantaine et la commande rejouable.
6. **Requête `pending`** (`delegation.rs:2684`). Elle inclut `cancelled` avec `workspace` ou `commit_view_prepare` non nul tant que `cleanup_done=0`. Le test `spec234_pending…` (l. 5562) couvre l'inclusion, la sortie après `cleanup_done` et le comportement ordinaire conservé.
7. **Carte publique** (`workspace_commit_view.rs:1429-1441`, `1743-1766`).
   - `MAX_NOTICE` est de 256 Mio et la lecture est limitée par `take(MAX_NOTICE+1)`.
   - Seul le manifeste est désérialisé. Les racines viennent de la preuve privée.
   - L'identité du dossier est contrôlée avant et après la lecture, et `proof_digest` lie le manifeste à l'intention.
   - `NoticeBytes` a son message de refus (`native_workspace_preparation210.rs:59`).

**Points P3**

- **P3-1, `workspace_compact.rs:234-235`.** `audit` lit `ls-files -v -z` avec `READ_TIMEOUT` (10 s) alors que `capture` accepte jusqu'à 64 Mio et que le journal le nomme `index_inventory`. C'est la même classe de dépassement que l'incident, sur l'index du worktree et non sur l'arbre du commit. Hypothèse à vérifier : mesurer `ls-files` sur le worktree SYS de 22 519 fichiers. Si la marge est faible, passer `INVENTORY_TIMEOUT` ici aussi. Sinon, documenter ce choix.
- **P3-2, `remote_worker_capacity234.rs:68-74`.** Le magasin n'enregistre qu'un booléen de quarantaine, pas sa cause. Le témoin prouve que le défaut a pu la provoquer, pas qu'il l'a fait. L'inventaire complet et l'absence de producteurs vivants limitent le risque, donc c'est acceptable. À documenter dans le reçu : l'outil ne lève la quarantaine que si aucun autre défaut ne subsiste.
- **P3-3, `:98-102`.** Chaque exécution laisse une sauvegarde `.before-view-capacity-<uuid>.json` qui n'est jamais retirée. L'effet est négligeable, mais le nettoyage de fin de livraison doit la conserver comme preuve.
- **P3-4, `workspace_git.rs:34-43`.** `task_id` du journal est un `thread_local`. Si Git est appelé depuis un autre thread que celui qui a posé `GitTask234`, la valeur est `null`. Hypothèse à vérifier : le chemin de l'incident passe-t-il par `spawn_blocking`, donc par un autre thread ? Si oui, le contexte tâche ne servira pas pour l'exploitation. C'est sans risque, l'impact est seulement diagnostique.

**Minimalisme et responsabilité future.** La correction est étroite : une constante partagée, un plafond par commande et une commande administrative à portée unique `this_store`. Elle ne crée ni nouvelle autorité ni nouveau protocole. Je n'ai vu ni chemin de reprise qui réécrit des reçus ni suppression de marqueurs. Le formatage est dense et les fonctions longues (`capture_with_objects`, `recover_legacy_view_capacity`), ce qui augmente le coût de relecture sans être un défaut.

**Non vérifié.** Ni l'activation réelle des 7 images `build234-inventory-r3`, ni le retour `234-r0`, ni les façades client encore à recharger, ni le comportement de `compact_plan` sur SYS : je m'en remets à vos preuves.

Je n'ai lancé aucune mission ni aucun service, et n'ai modifié aucun fichier.

Réponses aux réserves et preuves runtime : validation.md. Résultat intégral et reçu ACK conservés dans le cache privé de livraison.
