# Préflight hôte - SPEC-066

Date : 2026-08-30
Branche : session-066-environnement-partage-projet
Tête : 74234641fe04293c007717b8f1e3879823f5d382

## Docker

- Client Docker : 29.6.2.
- Serveur Docker : 29.6.2.
- Mode rootless : non déclaré. Les options de sécurité observées sont AppArmor,
  seccomp builtin et cgroup namespace.
- Cgroup : v2. Driver de stockage : overlayfs.
- Contexte Docker : default.
- Compte opérateur : uid=1002, gid=1002, membre du groupe docker gid=988.
- Socket Docker : /var/run/docker.sock, mode 0660, propriétaire root:docker.
- Des conteneurs applicatifs existent déjà. SPEC-066 ne les inspecte pas, ne les
  modifie pas et ne les réutilise pas.

## Chemins et permissions

- Racine du compte : /home/moi, mode 0755, uid/gid 1002.
- Etat local : /home/moi/.local/state/bridget, mode 0770, uid/gid 1002.
- Parent de state roots proposé par le contrat :
  /home/moi/.local/state/bridget/projects. Il est absent à ce stade et sera créé
  uniquement par le préflight fixture sous propriété uid/gid 1002.
- Référent Bridget : /home/moi/bridget-referent/bridget, mode 0770.
- Répertoire de worktrees : /home/moi/bridget-referent/.worktrees, mode 0770.
- Système de fichiers : ext4 rw sur /dev/md3. Espace disponible : 191 Gio.

## Worktrees

Tous les worktrees observés de Bridget, y compris le worktree 066, pointent vers
le common dir /home/moi/bridget-referent/bridget/.git. Ce fait autorise la
vérification d'un worktree rattaché sans scan global. Tout autre common dir ou
tout chemin qui imposerait un montage plus large reste refusé.

## Décision de préflight

La cible satisfait les préconditions minimales de test de Docker CLI : moteur
accessible, compte non-root dans le groupe docker, cgroup v2 et espace disque
suffisant. Elle ne prouve pas encore la politique fixture, l'image immuable, le
state root, les droits UID/GID dans le conteneur ni le runtime ingress : ces
preuves sont réservées aux tâches T011 à T016.

Aucun service n'a été redémarré, aucun conteneur existant n'a été modifié et
aucun projet réel n'a été activé.
