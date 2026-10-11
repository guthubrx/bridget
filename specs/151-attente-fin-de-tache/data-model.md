# Modèle de données - Session151

## Tâche native (`delegation.rs`, struct `Task`, payload JSON)

Un seul champ, ajouté en dernière priorité (US7, P2), optionnel pour garder les
payloads 148 à 150 lisibles :

| Champ | Type | Défaut si absent | Rôle |
|---|---|---|---|
| `acknowledged_at` | `Option<i64>` | `None` | Instant de l'accusé de lecture explicite du parent (`bridget_task_ack`). Écrit une seule fois. Informatif. |

Champs 150 utilisés sans changement : `delivery_acked`, `reply_timeout`,
`result_sent`, `failure_sent`. La session 151 (attente, statut) n'écrit AUCUN de
ces champs. Seuls `tick` et l'accusé de remise les écrivent, comme en 150.

`visible_changed` (`delegation_lineage.rs`) ignore `acknowledged_at` : aucun
signal Lineage, aucune nouvelle génération. Aucun nouvel état de tâche.

Sans US7, la session 151 n'ajoute aucun champ à `Task`.

## Boîte de fin de tâche (vue calculée)

| `mailbox.state` | Condition | Source |
|---|---|---|
| `pending` | `result_available` avec `result_sent = false`, ou `failed` avec `failure_sent = false` | tâche |
| `claimed` | envoi fait, phase de remise `dispatching` ou `indeterminate`, `delivery_acked = false` | tâche + lecture de phase (150-T005) |
| `delivered` | phase `acked`, ou `delivery_acked = true` | `send_deliveries` / tâche |
| `acknowledged` (P2) | `acknowledged_at` présent | tâche |

Priorité : `acknowledged` > `delivered` > `claimed` > `pending`.
Une tâche non terminale (`queued` à `waiting_for_children`, `cancelling`) n'a pas
de boîte : `mailbox` vaut `null`. Une tâche `cancelled` non plus.

Une tâche purgée par la rétention 150 n'est plus observable. Son identifiant rend
`task_unavailable`, sans entrée de boîte. La purge n'est pas un état de l'API.

La remise par message écrit les faits lus par les trois premiers états.
L'attente et le statut ne les écrivent pas :

```
pending -> claimed -> delivered
```

L'accusé de lecture explicite US7 donne `acknowledged`, même si la remise n'est
pas encore prouvée. La purge 150 garde ses conditions de remise et de nettoyage.
L'accusé de lecture ne les remplace pas.

## `work_state` (dérivé de `status`)

| `status` | `work_state` |
|---|---|
| `queued`, `starting`, `mission_pending`, `working`, `cancelling` | `working` |
| `waiting_for_children` | `waiting_for_children` |
| `result_available` | `result_available` |
| `failed` | `failed` |
| `cancelled` | `cancelled` |

"Terminée" pour l'attente : `result_available`, `failed`, `cancelled`.

## Résumé

`summary` = octets 0 à 2048 de `result`, reculés jusqu'à une frontière de caractère
UTF-8. Présent seulement pour `result_available`. `summary_truncated` = vrai si
`result` dépasse la coupe. `result_bytes` = longueur de `result` en octets.
`detail` = `{"tool":"bridget_task_status","task_id":"…"}` pour `result_available`
et `failed`, sinon `null`. Les entrées d'attente n'ont pas `result`.

## Curseur d'événements (mémoire du daemon, non durable)

`WaitEpoch = Arc<(Mutex<u64>, Condvar)>`, créé par `DelegationStore::open`, exposé
par un accesseur. `bump()` à chaque `save_projected` et `insert_projected`, et aux
sites de demande suivie (réponse accusée, chemin hérité, annulation).

Ordre des verrous : verrou d'état, puis verrou du compteur (pour `bump`). Le
compteur n'est jamais tenu pendant l'accès à l'état. La boucle d'attente lit
`seen = get()`, lit l'état, puis `wait_changed(seen, 5 s)` : `seen` est relu avant
chaque lecture.

Borne : 64 attentes actives au total (`wait_capacity`), compteur mémoire du daemon
décrémenté par une garde RAII (résultat, expiration, fermeture, arrêt, panique).

## Demande suivie (`tracked_requests`)

Colonne ajoutée :

| Colonne | Type | Défaut | Rôle |
|---|---|---|---|
| `reply_summary` | `TEXT` | `NULL` | JSON `{"id","from","summary","summary_truncated"}` : début de la réponse (1024 octets au plus, frontière UTF-8), écrit dans le même `UPDATE` que `state = 'answered'`. |

`NULL` pour une réponse via guichet, une enveloppe manquante ou une ligne antérieure
à 151. Alors l'attente rend `reply: null` et `reply_reason: "reply_unavailable"`.

Migration : `ALTER TABLE tracked_requests ADD COLUMN reply_summary TEXT`, gardée par
`pragma table_info`, dans les deux définitions de la table (`idempotency.rs`,
`store.rs`) pour que les tests isolés du socle idempotent gardent leur schéma.

## Annonce de fonctions du daemon

`DaemonIdentityReport` (`bridget-transport`) gagne `features: Vec<String>`
(`#[serde(default)]`). Un daemon 151 renvoie `["wait_v1"]`. Un ancien daemon n'a
pas le champ : le client lit une liste vide.

## Façade MCP (mémoire du processus MCP)

`MAX_WAITING_TOOL_CALLS = 4`, en plus de `MAX_IN_FLIGHT_TOOL_CALLS = 8`.
Un registre unique sous verrou porte `appel -> {waiting, cancelled, socket}`.
Il compte les créneaux et ferme le socket de sonde ou d'attente sur annulation.

## Aucun changement

États de tâche, projection Lineage, table `send_deliveries`, table `ledger`,
tombes de 150, `tick`, enveloppe de `bridget_send`.
