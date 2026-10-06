# Livraison locale : lookup ROOT psychologie après SPEC138

Le stderr historique n'était pas une nouvelle panne : sa dernière modification
date du 2026-10-05 à 17:45:35 +0200. Le tick récent du wrapper est silencieux.
La lecture a cependant prouvé une incompatibilité : son callback role appelle
bridget_agents() sans contexte, alors que le nouveau canon retourne [] dans ce
cas. Il ne peut plus vérifier son ROOT_UUID déjà déclaré.

Fix d'une seule ligne dans
/Users/moi/Documents/opus2D/orchestration/psychologie-heartbeat/heartbeat.py:341 :
bridget_agents(global_scope=True, project_root=str(run.get('project_root') or '')).
Le callback résout toujours le seul ROOT_UUID configuré. Aucune suggestion ni
recrutement n'est ajouté. domain_context.project_root n'est pas promu en racine
de communication. Le canon et la règle locale ne sont pas modifiés.

Backup privé avant patch :
/Users/moi/.cache/bridget-adoptions/spec138-20261006.qJc9jg/psychologie-heartbeat.before.py

Recette isolée (callback réel, moteur métier et notifications simulés ; aucun
heartbeat complet, aucun appel Bridget et aucun envoi) :

```text
python3 /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/138-priorite-projet/specs/138-priorite-projet/evidence/legacy-psychologie-lookup-test.py
```

RED avant patch : trois échecs, appel attendu avec global_scope/root, appel réel
vide. GREEN après patch : trois tests PASS en 0,004 s. Cas : ROOT déclaré seul,
ROOT absent malgré un agent étranger présent, racine explicite transmise sans
promotion du contexte métier. Les deux rôles consultent le même ROOT_UUID et
l'annuaire n'est consulté qu'une fois. Chaque fixture compare tous les fichiers
octet par octet avant/après dry-run ; write_json et send ne sont jamais appelés.
Les fichiers temporaires sont privés et aucune tâche de production n'est lue.

Le RED peut être reproduit en fixant PSYCHOLOGIE_LOOKUP_WRAPPER au backup privé.
Patch portable versionné : legacy-psychologie-lookup.patch dans ce dossier.
Après fusion, la recette reste accessible sans worktree :
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/138-priorite-projet/evidence/legacy-psychologie-lookup-test.py

SHA256 avant : 6a270f1fc324ee3cbee9e9031d54ee86400e272e9fc6773002be02c4ccf01dd3
SHA256 après : f464239bdf3f861068c45c0985fcb422c93f1ef7f5f326f61b1658fa2dab986d
SHA256 canon inchangé : bd561d1c46f1c70a5517cb9852610a97897d94e9fd4eb900b861bc9cb8b5347f

Aucun commit, merge, push ou redémarrage. Le wrapper est rechargé au prochain
tick du LaunchAgent existant. La recette ne prouve pas une livraison réseau.
