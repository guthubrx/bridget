# Reprise distante : distinguer un arrêt et un refus de droits

22 octobre 2026. Complément de correction de la session 172. Tests ciblés terminés ; revue et activation encore à réaliser à ce checkpoint.

## Faits observés

Le parent monuments a demandé la continuation de d7ddb20c-ac72-4f88-aad4-040acf30625e avec la clé monuments-notre-dame-regroupement-20261022-02 à 07:45:42 UTC.

Le reçu du worker situe déjà la commande en phase `terminal`, avec `native_ready=true`, depuis 07:36:56 UTC. Cette heure correspond à la coupure de l’ancien wrapper lors du redémarrage maître de 09:36 CEST. Les traces Codex des tours 67 et 70 portent le même dossier et les mêmes réglages de permissions. La garde de continuation est désormais libérée ; le résultat de première ronde reste conservé. Aucune seconde ronde n’a été admise.

Le contrôle `remote_dispatch_proof` exigeait `Released`, `native_ready` et le même condensat des droits. Tout autre état, y compris `Terminal`, devenait `permission_not_inherited`. Le code ne permettait donc pas de distinguer une exécution arrêtée d’une différence de permissions. Le reçu historique ne conserve pas le détail de la branche qui a produit le refus ; le défaut de classification est reproduit directement sur le canal maître-worker.

## Correction

Dans /home/moi/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon/native_delegation.rs :

- `Terminal` ou `Refused` produit `remote_execution_ended`.
- Une phase non libérée, un agent non prêt ou une réponse inattendue produit `remote_execution_unconfirmed`.
- Un agent prêt avec un condensat des droits différent reste refusé par `permission_not_inherited`.
- Les deux nouveaux codes portent un message constant. Le cas incertain demande de vérifier la tâche sans créer de doublon.

Aucune autorisation, reprise automatique, modification du protocole worker, nouvelle ronde ni réécriture de tâche ne sont ajoutées. Le résultat existant et son dossier restent sous le contrôle de leur propriétaire.

## Référence T3

Lecture de /Users/moi/11.Repositories/t3code-local/apps/server/src/mcp/OrchestratorMcpService.ts :

- `interruptThread`, vers 2740–2780, distingue `no_active_run` / `already_terminal` du contrôle préalable des modes et permissions.
- `delegateTask`, vers 2090–2118, prend le checkout actuel du parent et son commit ; la création de fils partagés reprend aussi `parent.thread.worktreePath`, vers 2440.

Mécanisme repris : distinguer l’état du processus et les permissions, puis garder une erreur sans effet métier. T3 ne couvre pas le reçu `WorkerObservation` d’un fournisseur distant conservé par Bridget ; cette phase fermée reste la preuve propre au worker Bridget.

Le refus `cwd_outside_parent_project` de l’intégrateur opus2D est séparé : la demande visait la racine Git depuis un parent attesté dans son worktree. Le prochain choix de revue doit utiliser le checkout voulu et son SHA exact. Aucune demande métier n’a été rejouée pour ce diagnostic.

## Validation ciblée

Trois tests distincts PASS sur Cartae, aucune suite complète :

1. `spec172_remote_continuation_distinguishes_execution_state_from_permissions` : vraie requête `Reconcile` sur socket Unix, six réponses worker, même tâche persistée avant/après. Cas terminé, refusé, inconnu, non prêt, droits différents et exécution prête autorisée.
2. `spec172_continuation_parent_turn_renewal_preserves_permission_ceiling`.
3. `spec172_continuation_renewed_turn_reaches_worker_proof_without_mutating_task`.

Les preuves privées sont dans /home/moi/.cache/bridget-reprise-diagnostic172 : green.json, les deux journaux de test et red-final.json. Le RED final conserve le test identique et retire seulement la classification corrigée. Les premiers essais GREEN ont aussi détecté une préparation de test incohérente ; la comparaison porte désormais sur la tâche réellement persistée, après normalisation des champs immuables et de la date de fin.

Ces tests utilisent un worker simulé à la frontière du canal. Ils ne lancent aucun fournisseur. Une vérification MCP du refus sur une tâche propre déjà terminée reste à effectuer après activation.

Le correctif de reconnexion 172-50b0673f reste actif sur le worker Cartae pour les nouveaux wrappers. Le maître ne sera relancé qu’après vérification des processus, sans relance T3.
