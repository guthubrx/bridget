# Recherche - SPEC-085

## Sources internes examinées

- `crates/bridget-daemon/src/project_runtime.rs`: politiques, image, UID/GID, ressources, montages, rootfs RO et cycle Docker existent.
- `crates/bridget-transport/src/protocol.rs`: opérations runtime typées sans options Docker libres.
- `crates/bridget-daemon/src/daemon.rs`: les projets Host ne peuvent actuellement pas être préparés en Docker via le cycle runtime.
- `crates/bridget-daemon/src/cli.rs`: les chemins des trois politiques sont déjà acceptés.
- `infra/project-runtime/README.md` et `Dockerfile`: fixture de test, pas image de production.
- SPEC-066, SPEC-067 et SPEC-080.

## Sources externes primaires

- Docker, bind mounts: les montages donnent un accès direct au filesystem hôte et peuvent être read-only. https://docs.docker.com/engine/storage/bind-mounts/
- Docker, resource constraints: les conteneurs n'ont pas de limites par défaut. https://docs.docker.com/engine/containers/resource_constraints/
- Docker, build best practices: épingler une image de base par digest améliore la reproductibilité. https://docs.docker.com/build/building/best-practices/
- Docker, sécurité du daemon: contrôler Docker est une capacité hôte privilégiée et doit rester réservée aux utilisateurs de confiance. https://docs.docker.com/engine/security/
- Docker, rootless mode: durcissement possible, mais avec prérequis hôte spécifiques. https://docs.docker.com/engine/security/rootless/
- Git, linked worktrees: les worktrees partagent un répertoire Git commun avec des HEAD/index séparés. https://git-scm.com/docs/git-worktree
- OpenAI, exécution sûre de Codex: sandbox bornée, réseau géré, approbations et télémétrie sont des contrôles complémentaires. https://openai.com/index/running-codex-safely-at-openai/
- Anthropic, sécurité Claude Code: permissions projet et devcontainers sont recommandés pour une isolation supplémentaire. https://docs.anthropic.com/en/docs/claude-code/security

## Conclusions

1. La frontière de sécurité utile est le projet, pas l'agent individuel. Cela conserve la collaboration et borne les fichiers/secrets.
2. Le socket Docker ne doit jamais être monté dans le conteneur. Seul le daemon hôte traduit des commandes typées en appels Docker.
3. Les limites CPU, mémoire et PIDs sont obligatoires, car Docker n'en impose pas par défaut.
4. Une image par digest protège la reproductibilité, mais nécessite un processus volontaire de rebuild pour les correctifs.
5. Les chemins absolus identiques évitent de rendre invalides les pointeurs Git de worktrees liés.
6. Rootless Docker peut devenir un durcissement ultérieur, mais ne doit pas bloquer ce lot simple.

## Alternatives rejetées

| Alternative | Rejet |
|---|---|
| Un conteneur par agent | Complexifie collaboration, caches et worktrees sans besoin actuel. |
| Docker Compose | Ajoute une deuxième orchestration au-dessus du daemon Bridget. |
| Kubernetes | Disproportionné pour un serveur et des projets locaux. |
| GUI/VNC | Augmente image, surface d'attaque et état persistant sans besoin défini. |
| Monter `/var/run/docker.sock` dans les projets | Donne un contrôle quasi hôte aux agents. |
| Fallback automatique vers Host | Contredit l'intention d'isolation de l'opérateur. |

