# ADR-026 - Projet système Bridget et dogfooding expert

## Statut

Accepté pour implémentation par SPEC-086.

## Contexte

Bridget doit pouvoir être développée par ses propres agents sans permettre à tous les projets de modifier le plan de contrôle. Le modèle par projet et conteneur partagé rend une permission par agent inutilement complexe.

## Décision

- Autoriser au plus un projet de rôle `bridget_system` par installation.
- Exiger un checkout explicitement configuré et attesté.
- Le classer `exact_project + system_only`.
- Fournir un réglage expert disabled/enabled commun à tous les agents du projet.
- Garder le checkout principal read-only, rendre seulement le worktree attribué writable par les mounts Docker et recréer sur transition.
- Conserver Git/worktrees pour les développements concurrents.
- Exclure merge main, push, install, restart et deploy automatiques.

## Conséquences

- Le dogfooding exige le runtime Docker de production.
- Les agents externes peuvent continuer sur d'autres worktrees.
- Une transition attend l'absence d'agents actifs.
- L'opérateur assume explicitement le risque d'édition, mais la livraison reste séparée.
- Les agents du projet système partagent les métadonnées Git et appartiennent au même domaine de confiance coopératif.

## Alternatives rejetées

- Droits pour tous les projets.
- RBAC par agent ou mission.
- Permission temporisée.
- Copie du dépôt dans Docker.
- Auto-déploiement du daemon.
