# Contrat : boîte de réception humaine v1

Constante : `HUMAN_INBOX_CONTRACT_VERSION = 1`. Capacité service : `ServiceCapability::HumanInboxV1` (Maicie). Capacité client : `ControlStateV1` (lecture et résolution par le principal humain).

## Dépôt (Maicie ou daemon interne)

### `WrapperToDaemon::HumanInboxDeposit`
```json
{ "version": 1, "dedup_key": "chain-exhausted:<delegation_id>", "kind": "chain_exhausted",
  "subject_ref": { "objective_id": "…", "delegation_id": "…" },
  "context": { "summary": "…", "attempts": [ … ] },
  "options": [ "reassign:<agent_id>", "cancel", "ack" ] }
```
Rôle : `Service`, capacité `HumanInboxV1`. Réponse : `DaemonToWrapper::HumanInboxDeposited { item_id, created: true|false, occurrences }`. Idempotent par `dedup_key` parmi les items ouverts.

## Lecture et résolution (principal humain)

### `WrapperToDaemon::HumanInboxList { version: 1, state: "open" | "all", limit: 50 }`
Réponse : `DaemonToWrapper::HumanInbox { items: [ HumanInboxItem… ], open_count }`.

### `WrapperToDaemon::HumanInboxResolve { version: 1, item_id, choice, command_id }`
Rôle : `Client`, capacité `ControlStateV1`, principal humain requis. `choice` doit appartenir à `options`. Réponse : `HumanInbox` avec l'item résolu, ou `HumanInboxRejected { reason ∈ { HumanPrincipalRequired, UnknownItem, AlreadyResolved, ChoiceNotOffered } }`.

## Relève des décisions (Maicie, pull)

### `WrapperToDaemon::HumanInboxDecisions { version: 1, producer: "maicie", limit: 20 }`
Réponse : `DaemonToWrapper::HumanInboxDecisionsBatch { decisions: [ { decision_id, item_id, dedup_key, kind, subject_ref, choice, at } ] }`. La relève rend toutes les décisions **non acquittées** et ne modifie rien.

### `WrapperToDaemon::HumanInboxAck { version: 1, decision_id }`
Rôle : `Service`, capacité `HumanInboxV1`. Envoyé par Maicie **après** le commit de la transaction qui applique la décision et écrit `human_decisions_applied`. Idempotent : un second acquittement rend la même réponse. Réponse : `DaemonToWrapper::HumanInboxAcked { decision_id, acked_at }`. Un plantage entre relève et application relit la décision à la relève suivante ; Maicie l'ignore si `human_decisions_applied` la contient déjà, et renvoie l'acquittement manquant.

### Alerte de reprise de route humaine
Quand `replace_stale_ui_human_route` remplace une route dont l'écrivain est encore joignable et la présence fraîche, le daemon dépose lui-même un item `human_route_replaced` (dedup `human-route:<conn_id>`) avec l'instance et le canal de la nouvelle connexion, et le pousse sur le canal externe.

## Canal externe

Fichier `~/.config/bridget/human-channel.json` (0600) :
```json
{ "command": ["/Users/moi/.local/bin/founder-telegram-push", "--stdin"], "reminder_after_secs": 3600 }
```
Le daemon lance la commande à chaque dépôt créé et à chaque rappel, avec sur stdin :
```json
{ "item_id": "…", "kind": "…", "summary": "≤ 400 caractères", "options": [ … ], "open_count": 3 }
```
Échec ou absence de fichier : consigné dans `notify_attempts_json`, jamais bloquant.

## Vocabulaire fermé des `kind`

`intervention_required`, `chain_exhausted`, `review_verdict_pending`, `activation_approval`, `budget_reached`, `reply_debt`, `focus_waiting_agent`, `object_vanished`, `human_route_replaced`.
