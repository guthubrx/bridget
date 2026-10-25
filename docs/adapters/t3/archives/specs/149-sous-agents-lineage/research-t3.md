# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/research-t3.md](../../../../../../specs/149-sous-agents-lineage/research-t3.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Réutiliser les fils enfants, les sous-agents et Lineage de T3. Chaque tâche native Bridget devient un fil de présentation durable. Bridget conserve toute l'exécution. T3 ne crée aucun tour de fournisseur, aucune session de fournisseur et aucun effet de lancement pour cet enfant.

<a id="extrait-2"></a>

## Extrait 2

Le fil enfant possède `lineage.relationshipToParent = subagent`. Son parent est le fil T3 attesté pour une tâche racine. Pour une tâche imbriquée, son parent est le fil projeté de `parent_task_id`. Son `rootThreadId` reste le fil racine T3. Les identifiants sont déterministes : `thread:bridget-task:<task_id>` et `node:bridget-task:<task_id>`.

<a id="extrait-3"></a>

## Extrait 3

Ajouter une origine `bridget_native` aux sous-agents et un champ facultatif `bridgetTaskRef` aux fils et résumés T3. Ce champ contient les références natives, la version du snapshot et l'état observé. Il distingue une présentation Bridget d'une exécution T3. Le fournisseur de présentation utilise un espace de noms réservé, par exemple `bridget:<agent_type>`. Il ne prétend pas être une instance T3 configurée.

<a id="extrait-4"></a>

## Extrait 4

Ne jamais appeler `delegated_task.request` pour cette projection. Cette commande lance une seconde exécution. Ne pas utiliser `app_owned`, qui active le retour de résultat et les reprises T3. Ne pas présenter Bridget comme un sous-agent interne du fournisseur.

<a id="extrait-5"></a>

## Extrait 5

Racine T3 : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`.

<a id="extrait-6"></a>

## Extrait 6

- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/packages/contracts/src/orchestrationV2.ts:108` définit déjà le parent, la racine et les relations `fork/subagent`.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/packages/contracts/src/orchestrationV2.ts:657` définit les sous-agents, les états, le résultat et le fil enfant. Les origines actuelles sont `provider_native/app_owned`.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/web/src/components/Sidebar.logic.ts:844` masque déjà tous les fils `subagent` dans la colonne de gauche.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/packages/client-runtime/src/state/threadRelationships.ts:56` construit Lineage depuis les résumés de fils et les sous-agents. Les relations imbriquées existent déjà.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/web/src/components/chat/ThreadRelationshipsControl.tsx` ouvre les fils depuis Lineage. L'arrêt actuel ne cible que les tâches `app_owned`.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/web/src/components/ChatView.tsx:4263` remplace le composeur par une barre en lecture seule pour les enfants internes du fournisseur. Cette garde doit couvrir les enfants Bridget avec une présentation adaptée.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/bridget/BridgetReader.ts` réutilise le binaire Bridget et le contexte fil/projet. Son protocole147 possède un flux d'invalidation `generation/seq` et des lectures bornées. Le même schéma de service convient au connecteur149.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/EventSink.ts:122` permet une écriture atomique des événements, des projections et du reçu. Une commande interne de synchronisation peut utiliser ce chemin avec zéro effet.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/ProjectionStore.ts:1469` déduit actuellement le statut du dernier tour. Un enfant sans tour affiche `idle`. La projection149 doit utiliser l'état Bridget pour son résumé.

<a id="extrait-7"></a>

## Extrait 7

CLI ou contrôle natif Bridget, limité au binding humain147 du fil racine et au projet canonique. Le client transmet un fil T3 et le projet. Il ne transmet aucun AgentId parent autodéclaré.

<a id="extrait-8"></a>

## Extrait 8

Snapshot version1 : `generation`, `seq`, `root_owner_agent_id`, `tasks`, `next_cursor`. Chaque page est bornée à100. Les pages d'un même snapshot conservent le même `seq`. Le service T3 prépare toutes les pages avant publication atomique. La limite totale reste celle du magasin natif.

<a id="extrait-9"></a>

## Extrait 9

Le flux n'expose que `version/generation/seq/status`. Un changement invalide le snapshot. Une perte de génération impose une nouvelle lecture. Aucune lecture ni souscription ne lance ou ne progresse une mission. T3 ne déduit jamais un résultat de la seule fin du processus. Il reprend le résultat corrélé et l'état final natifs.

<a id="extrait-10"></a>

## Extrait 10

- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/mcp/OrchestratorMcpService.ts:1777` réalise l'introspection. `assertLiveCaller` vérifie un tour actif et le fournisseur propriétaire. Le registre MCP vérifie le credential éphémère.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/RuntimePolicy.ts` résout le mode supporté, le dossier du tour et les éventuelles surcharges de politique. Le seul mode affiché dans le fil ne suffit pas à prouver la politique réellement transmise.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/ProviderTurnStartService.ts:525` sélectionne la politique de ce tour, puis la transmet à la session et à l'adaptateur.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/Adapters/CodexAdapterV2.ts:6220` calcule les paramètres réels de `turn/start`. `buildCodexTurnStartParams` applique les valeurs par défaut puis les surcharges `approvalPolicy/sandboxPolicy`.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/Adapters/ClaudeAdapterV2.ts:7291` calcule la politique de la requête. Les options finales sont construites vers7374. Le mode est ensuite mutable dans le contexte de requête et mis à jour depuis les messages `system/init` et `system/status` vers7457.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/packages/provider-core/src/server/mcpSession.ts` possède déjà le registre privé des configurations MCP par fil. Il constitue le point simple pour lier un fait de politique publié par l'adaptateur au credential et au tour.

<a id="extrait-11"></a>

## Extrait 11

Transport au moteur : le daemon Bridget reçoit seulement la preuve éphémère du endpoint local réel et le credential opaque. Son adaptateur T3 les revalide hors verrou, puis lie la politique obtenue au binding vivant agent/instance/fil/projet. Aucun JSON de permission fourni par l'agent ne fait autorité. Aucun token ne doit être persisté, journalisé ou transmis à l'enfant. Les descendants dans un terminal héritent du snapshot natif de leur parent tâche.

<a id="extrait-12"></a>

## Extrait 12

`/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/crates/bridget-daemon/src/t3code.rs:1285` monte les fils T3 en agents Bridget. Un fil virtuel149 entrerait dans cet inventaire.

<a id="extrait-13"></a>

## Extrait 13

Le contrat du résumé T3 Rust doit donc reconnaître le champ `bridgetTaskRef` et exclure ces fils du montage. Aucun binding ni agent supplémentaire ne doit être créé pour une projection. Le marqueur explicite fait autorité. L'espace de noms des IDs fournit une garde additionnelle. Les anciens fils ordinaires restent inchangés.

<a id="extrait-14"></a>

## Extrait 14

1. Contrats communs : attestation de politique, origine native, référence de tâche, lectures/journal/annulation149. Ajouter des champs facultatifs pour les données historiques.
2. Adaptateurs Codex et Claude : publier la politique réelle avec le credential courant. Aucun changement de permission fournisseur par T3.
3. Service `BridgetLineage` dans le domaine Bridget : réutiliser la résolution du binaire et les bornes du service147. Snapshot plus invalidation. Réconciliation au premier accès et après reconnexion. Une panne garde l'historique vérifié et indique que l'état courant est indisponible.
4. Commande interne de synchronisation : créer ou mettre à jour les fils virtuels et les sous-agents en une transaction. Zéro outbox fournisseur. Les reçus et les IDs déterministes rendent le rejeu sans doublon.
5. Projection des résumés : rendre l'état Bridget, les horaires, le résultat et l'erreur. Garder les champs de tour actifs nuls, car Bridget n'est pas un tour T3.
6. Lineage web/client partagé : garder la navigation et les relations existantes. Ajouter l'arrêt natif pour `bridget_native`. Le clic ouvre le journal de l'enfant. La barre du fil rend le modèle et le statut natifs. Les surfaces mobiles doivent recevoir les mêmes données et la même garde de lancement.
7. Gardes serveur : refuser `message.dispatch`, lancement, reprise fournisseur, changement de fournisseur, fork et rollback sur un fil virtuel. Arrêter un enfant dirige vers Bridget. Arrêter le parent doit annuler ses descendants natifs sans les transformer en tâches T3.
8. Réconciliation : ne projeter que des tâches natives explicitement reliées au root attesté. Ne pas convertir les anciens agents ordinaires. Contrôler toute identité native déjà importée pour éviter une seconde ligne du même enfant.

<a id="extrait-15"></a>

## Extrait 15

Charge T3 estimée : 8 à12 heures d'implémentation, puis2 à4 heures de validation GLM. Environ15 à22 fichiers selon le chemin retenu pour le journal et les surfaces mobiles. Pas de nouvelle dépendance ni framework.

<a id="extrait-16"></a>

## Extrait 16

Risques principaux : politique Claude mutable ; credential réutilisé entre tours ; pagination de snapshot incohérente ; enfant virtuel pris pour une exécution T3 ; réimport par le pont ; arrêt parent incomplet ; résultats publiés deux fois ; journal disparu après nettoyage. Les contrats ci-dessus portent une garde explicite pour chaque risque.
