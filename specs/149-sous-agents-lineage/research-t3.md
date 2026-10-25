# Recherche adaptateur de bureau — session149

Statut : proposition d'architecture. Aucun code modifié. Aucun test ni revue exécuté par cet owner. Le principal détient les décisions finales et Git. GLM 5.3 Flash détient tous les tests et toutes les relectures.

## Décision proposée

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-t3.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-t3.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-t3.md#extrait-3).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-t3.md#extrait-4).

## Faits du code existant

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-t3.md#extrait-5).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-t3.md#extrait-6).

## Contrat de lecture native proposé

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-t3.md#extrait-7).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-t3.md#extrait-8).

Chaque tâche expose au minimum :

- `task_id`, `parent_task_id|null`, `parent_agent_id`, `child_agent_id`, `child_instance_id|null` ;
- `created_at`, `updated_at`, `started_at|null`, `completed_at|null` ;
- `agent_type`, `execution_protocol`, `model`, `effort|null`, `cwd`, `posture` ;
- `title` ou une consigne bornée pour le nom de ligne ;
- `status`, `result|null`, `error|null` ;
- une référence de journal contrôlée par Bridget et un curseur de lecture.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-t3.md#extrait-9).

Le journal se lit par `rootT3ThreadId + taskId + cursor`. Aucun chemin libre ne vient du RPC humain. Bridget vérifie l'appartenance et choisit le journal natif réel. Le journal doit rester accessible après nettoyage du processus et lors d'une reprise. Il peut être affiché sur le fil enfant sans créer de session fournisseur.

L'annulation humaine se fait par le même contexte `rootT3ThreadId + taskId`. Elle appelle le moteur natif, puis invalide la projection. Bridget décide de la descendance et de la fin réelle. Une demande d'arrêt n'est pas un état terminal observé.

## Attestation des permissions

L'outil148 `bridget_session` accepte un objet vide. Il rend uniquement `version1/environmentId/threadId/providerSessionId/providerInstanceId`. Il n'atteste aucun droit aujourd'hui.

Sources exactes :

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-t3.md#extrait-10).

Étendre l'introspection en version2. Aucun argument nouveau. Réponse `permissions.version1`, liée à `runId`, `providerSessionId`, `providerInstanceId`, `driver`, `cwd`, `runtimeMode`, `interactionMode`, avec `source=provider_turn`.

Politique Codex : `approval_policy`, `approvals_reviewer`, `sandbox_policy` issus des paramètres réels validés. Conserver les racines autorisées et l'accès réseau qui figurent dans ce sandbox. Ne pas inventer une valeur absente.

Politique Claude/GLM : `permission_mode` réellement courant, `tools`, `allowed_tools`, `disallowed_tools`, `sandbox_settings` seulement s'ils existent dans les options réelles, et présence de la médiation des permissions. Ne pas promettre une isolation du système d'exploitation que le fournisseur ne met pas en place.

Les adaptateurs publient ce fait privé juste avant de transmettre le travail au fournisseur. Ils le lient au credential courant et au `runId`. Ils le retirent si le lancement échoue. Le mode Claude mis à jour remplace le fait courant. L'introspection refuse une donnée absente, périmée ou liée à un autre propriétaire. Une politique du tour précédent ne remplace jamais celle du tour actuel.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-t3.md#extrait-11).

Bridget applique les droits attestés sans ajouter un grant humain. Le fournisseur conserve sa propre politique. Le refus de permission devient un échec explicite. Les droits ne viennent ni du prompt, ni d'un nom de modèle, ni d'un paramètre MCP du parent.

## Garde contre la boucle de pont

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-t3.md#extrait-12).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-t3.md#extrait-13).

## Intégration adaptateur de bureau minimale

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-t3.md#extrait-14).

## Estimation et risques

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-t3.md#extrait-15).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/research-t3.md#extrait-16).

Complexité attendue : O(n) pour la réconciliation des tâches bornées, avec Maps par `task_id` et `parent_task_id`. Pas de requête par ligne de Lineage. Souscriptions et lectures de journal seulement pour le contexte affiché. Les limites natives149 restent l'autorité.

Les tests et relectures restent à GLM 5.3 Flash. Priorités : attestation absente/fausse/périmée, droits Codex et GLM, une exécution native unique, rejeu et reconnexion, parent imbriqué, annulation de descendance, lecture de journal, zéro fil dans la colonne de gauche, anciens agents inchangés, zéro boucle de pont.
