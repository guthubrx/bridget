# Contrat de lecture humaine des fils — SPEC145 V1

Date : 2026-10-07. Décision principale figée pour les tâches. RPC T3 : `bridget.read`, constante `WS_METHODS.bridgetRead`.

## Transport Bridget

Capacité distincte : `HumanThreadViewV1`, sérialisation `human_thread_view_v1`. Entrée : `WrapperToDaemon::HumanThreadViewV1`. Sortie : `DaemonToWrapper::HumanThreadViewResult`. Rôle : Client, connexion Unix locale. Ancien daemon ou capacité non négociée : refus explicite de compatibilité. Aucune utilisation de `ThreadRequest` agent, `read`, ACK, poste, clôture ou mission comme repli.

Requête humaine :

```json
{
  "version": 1,
  "t3_thread_id": "UUID",
  "project_root": "/chemin/absolu/autoritatif",
  "request": {"action": "list", "limit": 20, "after_thread_id": "UUID optionnel"}
}
```

L'enum `request` est fermé :

| Action | Champs | Règles |
| --- | --- | --- |
| `list` | `limit?`, `after_thread_id?` | Défaut20, maximum100 ; UUID ASC, position exclusive. |
| `show` | `thread_id` | UUID canonique ; appartenance vérifiée. |
| `history` | `thread_id`, `from_seq?`, `to_seq?`, `limit?` | Défaut50, maximum200 ; from positif, ASC, snapshot borné. |

Le daemon ne fait pas confiance à un `agent_id` fourni par le client. Il n'existe aucun tel champ dans ce contrat. Il résout `T3ThreadBindingFact{version:1,t3_thread_id}` détenu par la connexion primaire. Vérifications : route.conn égale cette connexion ; identité vivante ; agent égale `stable_uuid(thread)` ; instance égale `stable_uuid(instance:thread)` ; projet actif `communication_projects.0` de source T3. Canoniser le projet avec `resolve_communication_project(workspaceRoot,host,T3,None)` ; racine Git et workspace ne sont pas toujours identiques. `last_known` n'est pas une autorité. Deux liaisons admissibles produisent un refus ambigu. À déconnexion, retirer le fait et sa preuve source T3 en mémoire.

La confiance locale reste celle du compte système existant. Ce fait ne prétend ni credential nouveau ni attestation cryptographique entre processus du même compte. La consultation humaine fonctionne quand le modèle est dormant, tant que la liaison primaire attestée reste présente.

Réponse : `{version:1,subject:{agent_id,name:null|string}|null,result:{status:listed|shown|history|error,...}}`. Réutiliser list/show/history. Noms par `Store::agent_display_name` seulement. Retirer `own_acked_seq`, `own_wake`, credentials, transport, états de connexion et capacités de notification. Une position de page humaine n'est pas un curseur agent. Charger membres et noms de la liste par SQL groupé, avec au maximum16 membres par fil.

Le handler humain retourne avant `collect_closed_attach_views`. Aucune maintenance mutative annexe ne précède lecture ou refus. Erreurs métier fermées : `unsupported_version`, `binding_unavailable`, `project_mismatch`, `thread_unavailable`, `invalid_request`, `storage_unavailable`, `response_too_large`. Une erreur sans sujet attesté peut avoir subject null. Les erreurs opérationnelles CLI/T3, dont daemon absent et délai, restent distinctes.

Page historique source bornée à60 Kio ; projection humaine enrichie bornée à128 Kio. Aucun résultat partiel n'est présenté comme complet. Les corps restent exacts. Un type legacy absent reçoit « Message ». History, Action, Blocker et Decision gardent leurs types réels.

Interopérabilité I004 vérifiée sur la sortie stockage existante : `notify` historique est un objet fermé `{mode:"none"|"targets"|"all",targets:UUID[]}`, au maximum16 cibles. Ce n'est pas le `ThreadNotify` d'une requête Post, qui accepte `"all"` ou une liste d'UUID. Réutiliser la sortie réelle, sans nouveau type Post ni conversion en requête de mutation. Le stockage sérialise cet objet à la ligne1035, puis le restitue dans entry_json à la ligne624 de `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/store/threads.rs`.

Les dates `created_at` et `closed_at` sont des nombres Unix en secondes, de type Rust i64. `closed_at` peut être null. Elles ne sont pas des chaînes ISO. Le rendu humain peut formater ces secondes, sans changer la valeur source. `ThreadRow` les déclare i64/Option<i64> ; l'entrée reçoit created_at:i64. La fonction unix_now_secs utilise as_secs dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/daemon.rs`.

## CLI

Commande : `bridget thread inspect --t3-thread UUID --project-root chemin --action list|show|history ... --json`. Les options de pagination correspondent strictement à l'action. Elle utilise le rôle Client et négocie la capacité avant la demande.

Le service T3 lance des arguments séparés. Il n'utilise aucun shell. Il fixe un délai de six secondes et un plafond de sortie256 Kio. Échec de commande, daemon absent, délai, dépassement, UTF-8 invalide, JSON invalide et version incompatible deviennent des erreurs distinctes utiles. Les diagnostics ne journalisent ni corps ni credentials.

La CLI intercepte inspect dans `run()` avant `initialize_process` et `resolve_current_identity`. Utiliser `Namespace::from_environment` sans prepare et `DaemonConnection::connect` direct. Un daemon absent ne déclenche ni autostart ni création de namespace.

Résolution d'exécutable : `T3CODE_BRIDGET_EXECUTABLE` via `HostProcessEnvironment`, puis binaire utilisateur `/Users/moi/.local/bin/bridget` sur cette machine ou `resolveCommandPath("bridget")`. Construire le chemin générique depuis HOME. Refuser `.cmd` et `.bat` Windows. Décoder un JSON métier valide sur exit2 ; ne pas exposer stderr brut.

Accord opérationnel : exit3 signifie daemon absent ou namespace indisponible. Le service T3 le projette en état `unavailable`, sans parser stderr. Exit2 conserve le JSON métier existant. Aucun nouveau code d'erreur métier ni repli d'identité n'est ajouté.

## RPC T3

Une méthode native `bridget.read` porte les trois actions fermées. Elle transmet la conversation T3 sélectionnée et la pagination/désignation du fil Bridget. Le groupe RPC applique `AuthOrchestrationReadScope`. La connexion authentifiée route l'environnement. Le serveur résout conversation et projet via `ProjectionSnapshotQuery`, puis choisit `project.workspaceRoot`.

Le navigateur ne choisit pas la racine, le sujet Bridget ni une connexion primaire. Un fil Bridget forgé doit être refusé même si la liste n'a pas été appelée avant. Aucune RPC de génération, d'émission, d'ACK ou d'action mission ne fait partie de la consultation.

## Invalidation UI

Toute réponse est liée à environnement, conversation T3, fil Bridget et génération. Changer ou fermer invalide la génération. Un refus vide les données non autorisées. La recherche agit localement sur les pages chargées. Rafraîchir relit seulement liste/détail/historique de la sélection courante et repart avec un nouveau snapshot. Il n'y a ni abonnement ni polling.

## Preuves attendues

Tester rôle incorrect, capacité absente, version incorrecte, liaison absente/ambiguë/révoquée, fil forgé, autre projet, contexte dormant, limites0/hors plage, pagination stable et bornes de sortie. Comparer les tables métier avant/après lecture et refus. Vérifier zéro appel de `read`, ACK, réveil, dispatch et génération de modèle. Les erreurs doivent garder la distinction indisponible, incompatible et interdit.
