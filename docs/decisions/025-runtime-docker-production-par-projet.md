# ADR-025 - Runtime Docker de production par projet

## Statut

Accepté pour implémentation par SPEC-085.

## Contexte

SPEC-066 et SPEC-067 fournissent le moteur d'environnement partagé et les ressources approuvées, mais l'installation normale ne les rend pas activables. Une fixture ne constitue pas une image de développement de production.

## Décision

- Conserver un conteneur partagé par projet.
- Publier une image headless Linux amd64 épinglée par digest.
- Garder le contrôle Docker exclusivement dans le daemon hôte.
- Ajouter un défaut serveur Host/Docker pour les nouveaux projets.
- Ajouter une activation Host -> Docker atomique et un retour Host explicite.
- Monter le dépôt et les worktrees aux mêmes chemins absolus.
- Réutiliser le catalogue SPEC-067 pour extensions et secrets.

## Conséquences

- Les projets existants restent Host après migration.
- Un échec Docker ne conduit jamais à une exécution Host implicite.
- Les outils de l'image doivent être entretenus et reconstruits volontairement.
- Le daemon reste une autorité hôte sensible et ne transmet jamais le socket Docker.

## Alternatives rejetées

- Conteneur par agent.
- Docker Compose ou Kubernetes.
- Interface graphique ou VNC.
- Docker socket dans le conteneur.
- Images ou arguments libres depuis l'UI.

