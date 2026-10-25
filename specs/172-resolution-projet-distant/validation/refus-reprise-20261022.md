# Reprise distante : distinguer l’état de l’exécution et les droits

22 octobre 2026. Session 172. Livré et actif : build 172-222941e6. Quatre tests ciblés PASS, contre-revue r2 APPROVE et refus MCP réel vérifié.

## Faits observés

La continuation Notre-Dame d7ddb20c-ac72-4f88-aad4-040acf30625e, clé monuments-notre-dame-regroupement-20261022-02, a été refusée à 07:45:42 UTC. Le worker avait enregistré la commande `terminal` dès 07:36:56 UTC. Cette heure correspond à la coupure de l’ancien wrapper lors du redémarrage maître de 09:36 CEST.

Les traces Codex des tours 67 et 70 portent le même dossier et les mêmes réglages de permissions. La garde est désormais libérée et le résultat reste conservé. Aucune seconde ronde n’a été admise. Le reçu historique ne conserve pas le détail de la branche qui a produit le refus ; le défaut de classification est reproduit sur le contrôle de preuve distant.

L’ancien contrôle `remote_dispatch_proof` exigeait simultanément `Released`, `native_ready` et le même condensat des droits. Tous les autres cas devenaient `permission_not_inherited`, y compris une commande déjà terminée.

## Correction

Fichier : /home/moi/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon/native_delegation.rs.

- `Terminal` ou `Refused` produit `remote_execution_ended`.
- Une phase non libérée, un agent non prêt, un condensat absent ou une réponse inattendue produit `remote_execution_unconfirmed`.
- Un agent prêt dont le condensat attesté est différent reste refusé par `permission_not_inherited`.

Le message du premier cas dit que la commande est terminée ou son lancement refusé. Il ne prétend pas qu’un processus a existé dans le cas `Refused`. Le message du cas incertain demande de vérifier la tâche sans créer de doublon.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/refus-reprise-20261022.md#extrait-1).

La classification n’écrit pas l’enregistrement de tâche. Le gestionnaire entrant réel conserve son comportement antérieur : une observation terminale invalide le bail de flotte et marque l’instance comme arrêtée. Le correctif ne change ni droits, ni reprise automatique, ni protocole worker. Il ne crée aucune ronde ni aucune nouvelle mission.

## Choix de conception Bridget

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/refus-reprise-20261022.md#extrait-2).

- `failure`, vers 213, conserve le code et le message explicatif.
- `interruptThread`, vers 2740–2780, distingue `no_active_run` et `already_terminal` des contrôles préalables de modes et permissions.
- `delegateTask`, vers 2090–2118, prend le checkout actuel du parent et son commit ; les fils partagés reprennent aussi `parent.thread.worktreePath`, vers 2440.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/refus-reprise-20261022.md#extrait-3).

Le refus `cwd_outside_parent_project` de l’intégrateur opus2D est séparé : il demandait la racine Git depuis un parent attesté dans un worktree. Le checkout et le SHA choisis pour une prochaine revue doivent correspondre. Aucun rejeu de cette demande n’a été effectué.

## Validation ciblée

Quatre tests distincts PASS sur Cartae, trois commandes, aucune suite complète :

1. `spec172_remote_continuation_distinguishes_execution_state_from_permissions` : requête `Reconcile` réellement écrite sur une socket Unix ; réponse worker simulée injectée dans le canal interne attendu par le maître. Huit cas : terminé avec le drapeau réel `native_ready=false`, drapeau incohérent `true`, refusé, inconnu, non prêt, condensat absent, droits différents et preuve prête conforme. Le snapshot compare la tâche persistée avant et après.
2. `spec172_continuation_parent_turn_renewal_preserves_permission_ceiling`.
3. `spec172_continuation_renewed_turn_reaches_worker_proof_without_mutating_task`.
4. `native157_quota_detail_visible_cote_mcp` : échanges MCP réels sur une socket Unix avec démon simulé, inscription auxiliaire et décodage de la réponse. Deux cas 172 vérifient la transmission des conseils dans `detail.message` ; les trois cas historiques restent couverts.

Le premier test ne couvre pas le décodage entrant du worker ni son invalidation du bail. Le test MCP couvre le client MCP, pas le fournisseur. Ces tests ne lancent aucun fournisseur.

Preuves privées : /home/moi/.cache/bridget-reprise-diagnostic172/green-r2.json et journaux associés. La source est identifiée par SHA-256 pour les deux fichiers Rust. /home/moi/.cache/bridget-reprise-diagnostic172/red-r2.json lie le RED à la fixture finale identique ; seule la classification est rétablie à l’état antérieur. Ce RED prouve le premier cas seulement, car le test s’arrête au premier échec. Les premiers essais avaient aussi révélé une préparation de test incohérente ; le snapshot final est pris après la normalisation du store.

Le correctif de reconnexion 172-50b0673f reste actif sur le worker Cartae pour les nouveaux wrappers. La vérification MCP réelle est terminée ; elle utilise la tâche propre déjà clôturée décrite ci-dessous.


## Livraison active

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/172-resolution-projet-distant/validation/refus-reprise-20261022.md#extrait-4).

La recette officielle bridget_task_continue a utilisé UNE fois la tâche déjà terminée 508c947f-038a-42fe-836f-363d1fad6fda, clé recette172-refus-reprise-terminee-20261022-r1. Le client MCP déjà chargé a reçu remote_execution_ended et detail.message en français : « La commande distante est terminée ou son lancement a été refusé. Elle ne peut plus recevoir de travail. » Le refus est attendu. Le SHA-256 du payload entier de la tâche reste identique ; ronde 2, résultat et instance enfant sont conservés. Aucun nouveau calcul ni nouvelle tâche.

La contre-revue r2 est APPROVE, sans défaut bloquant. Les remarques documentaires ont été traitées ; la limite sur le bail entrant est déclarée, sans prétendre à une nouvelle couverture. fixture-link-r2.json vérifie que le fichier actuel porte le condensat du GREEN et du fichier restauré en RED, puis que son extrait de test porte le condensat RED r2 f293e563…. Le condensat r1 bcf399c5… diffère volontairement : r2 ajoute deux cas et l’assertion du conseil transmis.

Reçu public : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/172-resolution-projet-distant/validation/refus-reprise-20261022.json. Preuves privées : /Users/moi/.cache/bridget-reprise-diagnostic172 et /home/moi/.cache/bridget-reprise-diagnostic172. Le binaire précédent reste disponible pour retour arrière. Le correctif ne change aucun droit ni la limite du checkout parent.
