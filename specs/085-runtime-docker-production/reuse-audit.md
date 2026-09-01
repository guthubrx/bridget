# Audit de réutilisation - SPEC-085

## Verdict

EXTEND + COMPLETE - Le moteur Docker existe déjà. Il faut compléter son activation produit, son image et son installation, pas le remplacer.

## Inventaire audité

| Besoin | Composant existant | Décision |
|---|---|---|
| Politique runtime | `ProjectRuntimePolicyConfig` dans `project_runtime.rs` | Réutiliser et fournir une configuration de production. |
| Cycle Docker | `ProjectRuntimeOperation` et moteur Docker CLI | Étendre avec `ActivateDocker`; conserver les autres opérations. |
| Liaison initiale Docker | `ProjectBindRequest` et `bind_project_docker_registration` | Réutiliser pour les nouveaux projets. |
| Retour Host | `SwitchBackend` | Renommer/projeter clairement `switch_to_host`; conserver la logique et les gardes. |
| Limites/mounts | `create_project_container` et `resolve_project_mounts` | Conserver, compléter la topologie Git et les tests. |
| Profils/secrets | SPEC-067 | Réutiliser exclusivement. |
| CLI config | options `--project-*-policy` | Raccorder au service installé. |
| UI runtime | GET `/v1/projects/runtime` | Étendre en contrat v2 et actions POST. |

## Preuves de lacune

- Le unit systemd actif passe uniquement `--project-root-policy`.
- Aucune politique runtime ou ressource n'est chargée en production.
- `infra/project-runtime/README.md` qualifie l'image de fixture.
- Pour une liaison Host, seules `Status` et `SwitchBackend` retournent un état; `Prepare` est refusé `ProjectNotDocker`.
- L'UI ne possède qu'un GET de statut, sans cycle d'activation.

## Éléments à ne pas dupliquer

- Store de liaisons et générations.
- Lanceur Docker CLI.
- Contrats d'idempotence et de refus.
- Catalogue de ressources et profils résolus.
- Centre de contrôle et menu projet.
- Mécanismes de round/agent lifecycle non concernés.

## Nouveaux éléments justifiés

- Image de production distincte de la fixture.
- Réglage serveur `execution.default_backend`.
- Transition atomique Host -> Docker.
- Digest de topologie Git/mounts.
- Routes UI d'action typées.

Chaque élément possède au moins trois consommateurs ou invariants: configuration, daemon, UI/tests pour le réglage; protocole, store, runtime/UI pour la transition; build, politique et attestation pour l'image.

## Conclusion

La SPEC-085 est une finition verticale de SPEC-066/067. Créer un second orchestrateur, une image par projet ou une API Docker générique serait une duplication et une régression de sécurité.

