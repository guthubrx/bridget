# Recherche T3 — session149

Statut : proposition d'architecture. Aucun code modifié. Aucun test ni revue exécuté par cet owner. Le principal détient les décisions finales et Git. GLM 5.3 Flash détient tous les tests et toutes les relectures.

## Décision proposée

Réutiliser les fils enfants, les sous-agents et Lineage de T3. Chaque tâche native Bridget devient un fil de présentation durable. Bridget conserve toute l'exécution. T3 ne crée aucun tour de fournisseur, aucune session de fournisseur et aucun effet de lancement pour cet enfant.

Le fil enfant possède `lineage.relationshipToParent = subagent`. Son parent est le fil T3 attesté pour une tâche racine. Pour une tâche imbriquée, son parent est le fil projeté de `parent_task_id`. Son `rootThreadId` reste le fil racine T3. Les identifiants sont déterministes : `thread:bridget-task:<task_id>` et `node:bridget-task:<task_id>`.

Ajouter une origine `bridget_native` aux sous-agents et un champ facultatif `bridgetTaskRef` aux fils et résumés T3. Ce champ contient les références natives, la version du snapshot et l'état observé. Il distingue une présentation Bridget d'une exécution T3. Le fournisseur de présentation utilise un espace de noms réservé, par exemple `bridget:<agent_type>`. Il ne prétend pas être une instance T3 configurée.

Ne jamais appeler `delegated_task.request` pour cette projection. Cette commande lance une seconde exécution. Ne pas utiliser `app_owned`, qui active le retour de résultat et les reprises T3. Ne pas présenter Bridget comme un sous-agent interne du fournisseur.

## Faits du code existant

Racine T3 : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`.

- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/packages/contracts/src/orchestrationV2.ts:108` définit déjà le parent, la racine et les relations `fork/subagent`.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/packages/contracts/src/orchestrationV2.ts:657` définit les sous-agents, les états, le résultat et le fil enfant. Les origines actuelles sont `provider_native/app_owned`.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/web/src/components/Sidebar.logic.ts:844` masque déjà tous les fils `subagent` dans la colonne de gauche.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/packages/client-runtime/src/state/threadRelationships.ts:56` construit Lineage depuis les résumés de fils et les sous-agents. Les relations imbriquées existent déjà.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/web/src/components/chat/ThreadRelationshipsControl.tsx` ouvre les fils depuis Lineage. L'arrêt actuel ne cible que les tâches `app_owned`.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/web/src/components/ChatView.tsx:4263` remplace le composeur par une barre en lecture seule pour les enfants internes du fournisseur. Cette garde doit couvrir les enfants Bridget avec une présentation adaptée.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/bridget/BridgetReader.ts` réutilise le binaire Bridget et le contexte fil/projet. Son protocole147 possède un flux d'invalidation `generation/seq` et des lectures bornées. Le même schéma de service convient au connecteur149.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/EventSink.ts:122` permet une écriture atomique des événements, des projections et du reçu. Une commande interne de synchronisation peut utiliser ce chemin avec zéro effet.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/ProjectionStore.ts:1469` déduit actuellement le statut du dernier tour. Un enfant sans tour affiche `idle`. La projection149 doit utiliser l'état Bridget pour son résumé.

## Contrat de lecture native proposé

CLI ou contrôle natif Bridget, limité au binding humain147 du fil racine et au projet canonique. Le client transmet un fil T3 et le projet. Il ne transmet aucun AgentId parent autodéclaré.

Snapshot version1 : `generation`, `seq`, `root_owner_agent_id`, `tasks`, `next_cursor`. Chaque page est bornée à100. Les pages d'un même snapshot conservent le même `seq`. Le service T3 prépare toutes les pages avant publication atomique. La limite totale reste celle du magasin natif.

Chaque tâche expose au minimum :

- `task_id`, `parent_task_id|null`, `parent_agent_id`, `child_agent_id`, `child_instance_id|null` ;
- `created_at`, `updated_at`, `started_at|null`, `completed_at|null` ;
- `agent_type`, `execution_protocol`, `model`, `effort|null`, `cwd`, `posture` ;
- `title` ou une consigne bornée pour le nom de ligne ;
- `status`, `result|null`, `error|null` ;
- une référence de journal contrôlée par Bridget et un curseur de lecture.

Le flux n'expose que `version/generation/seq/status`. Un changement invalide le snapshot. Une perte de génération impose une nouvelle lecture. Aucune lecture ni souscription ne lance ou ne progresse une mission. T3 ne déduit jamais un résultat de la seule fin du processus. Il reprend le résultat corrélé et l'état final natifs.

Le journal se lit par `rootT3ThreadId + taskId + cursor`. Aucun chemin libre ne vient du RPC humain. Bridget vérifie l'appartenance et choisit le journal natif réel. Le journal doit rester accessible après nettoyage du processus et lors d'une reprise. Il peut être affiché sur le fil enfant sans créer de session fournisseur.

L'annulation humaine se fait par le même contexte `rootT3ThreadId + taskId`. Elle appelle le moteur natif, puis invalide la projection. Bridget décide de la descendance et de la fin réelle. Une demande d'arrêt n'est pas un état terminal observé.

## Attestation des permissions

L'outil148 `bridget_session` accepte un objet vide. Il rend uniquement `version1/environmentId/threadId/providerSessionId/providerInstanceId`. Il n'atteste aucun droit aujourd'hui.

Sources exactes :

- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/mcp/OrchestratorMcpService.ts:1777` réalise l'introspection. `assertLiveCaller` vérifie un tour actif et le fournisseur propriétaire. Le registre MCP vérifie le credential éphémère.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/RuntimePolicy.ts` résout le mode supporté, le dossier du tour et les éventuelles surcharges de politique. Le seul mode affiché dans le fil ne suffit pas à prouver la politique réellement transmise.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/ProviderTurnStartService.ts:525` sélectionne la politique de ce tour, puis la transmet à la session et à l'adaptateur.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts:6220` calcule les paramètres réels de `turn/start`. `buildCodexTurnStartParams` applique les valeurs par défaut puis les surcharges `approvalPolicy/sandboxPolicy`.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.ts:7291` calcule la politique de la requête. Les options finales sont construites vers7374. Le mode est ensuite mutable dans le contexte de requête et mis à jour depuis les messages `system/init` et `system/status` vers7457.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/packages/provider-core/src/server/mcpSession.ts` possède déjà le registre privé des configurations MCP par fil. Il constitue le point simple pour lier un fait de politique publié par l'adaptateur au credential et au tour.

Étendre l'introspection en version2. Aucun argument nouveau. Réponse `permissions.version1`, liée à `runId`, `providerSessionId`, `providerInstanceId`, `driver`, `cwd`, `runtimeMode`, `interactionMode`, avec `source=provider_turn`.

Politique Codex : `approval_policy`, `approvals_reviewer`, `sandbox_policy` issus des paramètres réels validés. Conserver les racines autorisées et l'accès réseau qui figurent dans ce sandbox. Ne pas inventer une valeur absente.

Politique Claude/GLM : `permission_mode` réellement courant, `tools`, `allowed_tools`, `disallowed_tools`, `sandbox_settings` seulement s'ils existent dans les options réelles, et présence de la médiation des permissions. Ne pas promettre une isolation du système d'exploitation que le fournisseur ne met pas en place.

Les adaptateurs publient ce fait privé juste avant de transmettre le travail au fournisseur. Ils le lient au credential courant et au `runId`. Ils le retirent si le lancement échoue. Le mode Claude mis à jour remplace le fait courant. L'introspection refuse une donnée absente, périmée ou liée à un autre propriétaire. Une politique du tour précédent ne remplace jamais celle du tour actuel.

Transport au moteur : le daemon Bridget reçoit seulement la preuve éphémère du endpoint local réel et le credential opaque. Son adaptateur T3 les revalide hors verrou, puis lie la politique obtenue au binding vivant agent/instance/fil/projet. Aucun JSON de permission fourni par l'agent ne fait autorité. Aucun token ne doit être persisté, journalisé ou transmis à l'enfant. Les descendants hors T3 héritent du snapshot natif de leur parent tâche.

Bridget applique les droits attestés sans ajouter un grant humain. Le fournisseur conserve sa propre politique. Le refus de permission devient un échec explicite. Les droits ne viennent ni du prompt, ni d'un nom de modèle, ni d'un paramètre MCP du parent.

## Garde contre la boucle de pont

`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/t3code.rs:1285` monte les fils T3 en agents Bridget. Un fil virtuel149 entrerait dans cet inventaire.

Le contrat du résumé T3 Rust doit donc reconnaître le champ `bridgetTaskRef` et exclure ces fils du montage. Aucun binding ni agent supplémentaire ne doit être créé pour une projection. Le marqueur explicite fait autorité. L'espace de noms des IDs fournit une garde additionnelle. Les anciens fils ordinaires restent inchangés.

## Intégration T3 minimale

1. Contrats communs : attestation de politique, origine native, référence de tâche, lectures/journal/annulation149. Ajouter des champs facultatifs pour les données historiques.
2. Adaptateurs Codex et Claude : publier la politique réelle avec le credential courant. Aucun changement de permission fournisseur par T3.
3. Service `BridgetLineage` dans le domaine Bridget : réutiliser la résolution du binaire et les bornes du service147. Snapshot plus invalidation. Réconciliation au premier accès et après reconnexion. Une panne garde l'historique vérifié et indique que l'état courant est indisponible.
4. Commande interne de synchronisation : créer ou mettre à jour les fils virtuels et les sous-agents en une transaction. Zéro outbox fournisseur. Les reçus et les IDs déterministes rendent le rejeu sans doublon.
5. Projection des résumés : rendre l'état Bridget, les horaires, le résultat et l'erreur. Garder les champs de tour actifs nuls, car Bridget n'est pas un tour T3.
6. Lineage web/client partagé : garder la navigation et les relations existantes. Ajouter l'arrêt natif pour `bridget_native`. Le clic ouvre le journal de l'enfant. La barre du fil rend le modèle et le statut natifs. Les surfaces mobiles doivent recevoir les mêmes données et la même garde de lancement.
7. Gardes serveur : refuser `message.dispatch`, lancement, reprise fournisseur, changement de fournisseur, fork et rollback sur un fil virtuel. Arrêter un enfant dirige vers Bridget. Arrêter le parent doit annuler ses descendants natifs sans les transformer en tâches T3.
8. Réconciliation : ne projeter que des tâches natives explicitement reliées au root attesté. Ne pas convertir les anciens agents ordinaires. Contrôler toute identité native déjà importée pour éviter une seconde ligne du même enfant.

## Estimation et risques

Charge T3 estimée : 8 à12 heures d'implémentation, puis2 à4 heures de validation GLM. Environ15 à22 fichiers selon le chemin retenu pour le journal et les surfaces mobiles. Pas de nouvelle dépendance ni framework.

Risques principaux : politique Claude mutable ; credential réutilisé entre tours ; pagination de snapshot incohérente ; enfant virtuel pris pour une exécution T3 ; réimport par le pont ; arrêt parent incomplet ; résultats publiés deux fois ; journal disparu après nettoyage. Les contrats ci-dessus portent une garde explicite pour chaque risque.

Complexité attendue : O(n) pour la réconciliation des tâches bornées, avec Maps par `task_id` et `parent_task_id`. Pas de requête par ligne de Lineage. Souscriptions et lectures de journal seulement pour le contexte affiché. Les limites natives149 restent l'autorité.

Les tests et relectures restent à GLM 5.3 Flash. Priorités : attestation absente/fausse/périmée, droits Codex et GLM, une exécution native unique, rejeu et reconnexion, parent imbriqué, annulation de descendance, lecture de journal, zéro fil dans la colonne de gauche, anciens agents inchangés, zéro boucle de pont.
