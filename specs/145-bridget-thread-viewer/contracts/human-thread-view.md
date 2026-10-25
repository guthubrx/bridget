# Contrat de lecture humaine des fils — SPEC145 V1

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/contracts/human-thread-view.md#extrait-1).

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

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/contracts/human-thread-view.md#extrait-2).

La confiance locale reste celle du compte système existant. Ce fait ne prétend ni credential nouveau ni attestation cryptographique entre processus du même compte. La consultation humaine fonctionne quand le modèle est dormant, tant que la liaison primaire attestée reste présente.

Réponse : `{version:1,subject:{agent_id,name:null|string}|null,result:{status:listed|shown|history|error,...}}`. Réutiliser list/show/history. Noms par `Store::agent_display_name` seulement. Retirer `own_acked_seq`, `own_wake`, credentials, transport, états de connexion et capacités de notification. Une position de page humaine n'est pas un curseur agent. Charger membres et noms de la liste par SQL groupé, avec au maximum16 membres par fil.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/contracts/human-thread-view.md#extrait-3).

Page historique source bornée à60 Kio ; projection humaine enrichie bornée à128 Kio. Aucun résultat partiel n'est présenté comme complet. Les corps restent exacts. Un type legacy absent reçoit « Message ». History, Action, Blocker et Decision gardent leurs types réels.

Interopérabilité I004 vérifiée sur la sortie stockage existante : `notify` historique est un objet fermé `{mode:"none"|"targets"|"all",targets:UUID[]}`, au maximum16 cibles. Ce n'est pas le `ThreadNotify` d'une requête Post, qui accepte `"all"` ou une liste d'UUID. Réutiliser la sortie réelle, sans nouveau type Post ni conversion en requête de mutation. Le stockage sérialise cet objet à la ligne1035, puis le restitue dans entry_json à la ligne624 de `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/store/threads.rs`.

Les dates `created_at` et `closed_at` sont des nombres Unix en secondes, de type Rust i64. `closed_at` peut être null. Elles ne sont pas des chaînes ISO. Le rendu humain peut formater ces secondes, sans changer la valeur source. `ThreadRow` les déclare i64/Option<i64> ; l'entrée reçoit created_at:i64. La fonction unix_now_secs utilise as_secs dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/crates/bridget-daemon/src/daemon.rs`.

## CLI

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/contracts/human-thread-view.md#extrait-4).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/contracts/human-thread-view.md#extrait-5).

La CLI intercepte inspect dans `run()` avant `initialize_process` et `resolve_current_identity`. Utiliser `Namespace::from_environment` sans prepare et `DaemonConnection::connect` direct. Un daemon absent ne déclenche ni autostart ni création de namespace.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/contracts/human-thread-view.md#extrait-6).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/contracts/human-thread-view.md#extrait-7).

## RPC adaptateur de bureau

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/contracts/human-thread-view.md#extrait-8).

Le navigateur ne choisit pas la racine, le sujet Bridget ni une connexion primaire. Un fil Bridget forgé doit être refusé même si la liste n'a pas été appelée avant. Aucune RPC de génération, d'émission, d'ACK ou d'action mission ne fait partie de la consultation.

## Invalidation UI

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/contracts/human-thread-view.md#extrait-9).

## Preuves attendues

Tester rôle incorrect, capacité absente, version incorrecte, liaison absente/ambiguë/révoquée, fil forgé, autre projet, contexte dormant, limites0/hors plage, pagination stable et bornes de sortie. Comparer les tables métier avant/après lecture et refus. Vérifier zéro appel de `read`, ACK, réveil, dispatch et génération de modèle. Les erreurs doivent garder la distinction indisponible, incompatible et interdit.
