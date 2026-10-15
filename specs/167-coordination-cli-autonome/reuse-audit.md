# Audit de réutilisation — SPEC-167

## Verdict

La fonctionnalité doit réutiliser le moteur existant. Aucun nouveau moteur, protocole ou stockage n’est justifié.

## Cartographie

| Besoin | Existant réutilisé | Écart restant |
|---|---|---|
| Choisir un profil et lancer un agent | `bridget_auto_delegate` et `AutoDelegateV1` | Syntaxe CLI trop technique. |
| Propriété de tâche | `ResolvedIdentity` et connexion enregistrée | Message de rattachement à rendre clair. |
| État | `bridget_task_status` | Pas de sous-commande CLI. |
| Attente | `bridget_task_wait` | Pas de sous-commande CLI. |
| Nouvelle ronde | `bridget_task_continue` | Pas de sous-commande CLI. |
| Réponse enfant | `bridget_task_respond` | Pas de sous-commande CLI. |
| Annulation | `bridget_task_cancel` | Pas de sous-commande CLI parent. |
| Accusé | `bridget_task_ack` | Pas de sous-commande CLI. |
| Liste humaine | `control delegations` | Ajouter un alias cohérent. |
| Rejeu | Reçus natifs par `request_id` | Générer et afficher une clé si absente. |

## Fichiers inspectés

- `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/cli.rs`
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/delegation_mcp.rs`
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/mcp_identity.rs`
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/daemon/native_delegation.rs`
- `/Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-transport/src/protocol.rs`

## Décision de périmètre

La première livraison modifie la façade CLI et sa documentation. Elle ne modifie le protocole que si un test démontre qu’un contrat natif manque réellement. L’absence actuelle concerne surtout l’exposition des opérations existantes.
