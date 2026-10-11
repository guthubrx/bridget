# Modèle de données - Session159

## Réglage du registre (lot A)

`AgentDefinition` (`registry.rs:49-79`) gagne :

| Champ | Type | Défaut | Rôle |
|---|---|---|---|
| `held_queue` | `bool` | `false` | Le daemon retient les messages ordinaires destinés à un agent de ce type tant qu'il est occupé |

`#[serde(default)]` : un registre existant se lit sans changement. Le plafond de
la file retenue est `queue_capacity` du même type (`registry.rs:67`).

## Table `send_deliveries` après migration v7 (lot A)

Reconstruction (précédent v4, `idempotency.rs:709-806`).

| Colonne | Avant | Après |
|---|---|---|
| `phase` | `CHECK IN ('dispatching','acked','indeterminate','orphaned')` | `CHECK IN ('dispatching','acked','indeterminate','orphaned','held','cancelled')` |
| `queue_rank` | absente | `INTEGER NOT NULL` ; rang global monotone |
| `recipient_agent` | absente | `TEXT NOT NULL` ; identifiant d'agent résolu à l'admission ; rempli à la migration par `json_extract(message_bytes,'$.to')` |
| `turn_gate` | absente | `INTEGER NOT NULL DEFAULT 0 CHECK (turn_gate IN (0,1))` ; réservation de tour |

- Lignes existantes : `queue_rank` = ancien `rowid`.
- Nouvelle ligne : `queue_rank = (SELECT COALESCE(MAX(queue_rank),0)+1 FROM send_deliveries)` dans la transaction `Immediate` de `begin_send_delivery_inner`. L'unicité tient au verrou d'écriture unique de SQLite.
- `begin_send_delivery_inner` prend un mode : `Dispatching` (existant) ou `Held` (nouvelle phase d'insertion). En mode `Held` : pas de lien d'exécution, enregistrement d'idempotence en `dispatching`, ledger et suivi de réponse écrits comme aujourd'hui.
- Réaffectation et orphelinage : s'appliquent à `dispatching` et `held`. Le rang ne change pas.
- `dispatching_deliveries_for_instance` : inchangé de contrat (ne rend que `dispatching`), trié par `queue_rank` au lieu de `delivery_id`.
- Index complémentaire : `idx_send_deliveries_agent_phase ON (recipient_agent, phase, queue_rank)` pour le rattachement par agent et la réservation de tour.
- Index : `idx_send_deliveries_operation` (unique), `idx_send_deliveries_kind_key`, `idx_send_deliveries_recipient_rank ON (recipient_instance_id, phase, queue_rank)`.
- `send_delivery_execution_links` (`ON DELETE CASCADE`) : copiée avant le `DROP`, réinsérée après le renommage. `PRAGMA foreign_key_check` à zéro.
- Migration enregistrée : `idempotency_schema_migrations(version) = 7`. `MAX_IDEMPOTENCY_SCHEMA_VERSION` passe de 6 à 7 (`store_schema.rs:4`).

Phase `cancelled` : terminale, absorbante. Jamais relivrée, jamais listée comme
`dispatching`, purgée avec son enregistrement d'envoi (purge existante).

## Réservation de tour et rattachement (lot A)

- `turn_gate = 1` : posé par la libération, sur la remise libérée. Levé par : `TurnState true` observé pour l'agent après la libération ; sortie de `dispatching` sans accusé ; accusé depuis plus de 30 s avec l'agent non occupé. Une libération exige qu'aucune remise de `recipient_agent` n'ait `turn_gate = 1`.
- `adopt_held_for_agent(agent_id, instance_id)` : `UPDATE send_deliveries SET recipient_instance_id = ?2 WHERE recipient_agent = ?1 AND phase = 'held'`, appelé à chaque enregistrement d'agent. Ne dépend pas de `presences`.

### Présence (mémoire du daemon)

| Champ | Rôle |
|---|---|
| `busy_period_start` | début de la période occupée ; non remis à zéro par un nouveau `true` |
| `last_attested_at` | date de la dernière attestation de l'état (vrai ou faux) |
| `turn_attest` | l'instance a déclaré un signal de tour véridique et réattesté |

`busy_since` (existant) est inchangé. La sortie forcée de `busy` pour les
messages retenus se compte depuis `busy_period_start`.

### Suivi mémoire

`held_ids` : ensemble en mémoire des identifiants de messages retenus,
reconstruit depuis la base au démarrage, tenu à jour à la rétention, la
libération et tout retrait. Les paliers 1 et 2 des relances ignorent les demandes
dont le message est dans `held_ids`. Après un commit de retrait, l'entrée de
`pending_replies` du message est retirée.

## Opérations de la phase `held`

| Opération | Transaction | Effets |
|---|---|---|
| Rétention | `begin_send_delivery_inner` mode `Held` | ligne `held`, rang, ledger, suivi de réponse |
| Libération | `release_next_held(agent, now)` | vérifie réservation, saga, demande ouverte et échéance ; plus petit rang `held` passe à `dispatching` avec `turn_gate = 1` ; rend `SendDelivery` |
| Retrait automatique | `settle_held(delivery_id, reason)` | `cancelled`, `Rejected(<raison>)`, notice d'auteur |
| Retrait demandé (lot B) | `apply_cancel` | idem + reçu `applied` |
| Déplacement (lot B) | `apply_reorder` | ré-affectation des rangs du groupe + reçu |
| Promotion (lot B) | `apply_promote` | mêmes vérifications que la libération ; `held` vers `dispatching` (avec réservation) + reçu |

Raisons de `settle_held` : `expired_in_queue` (enregistrement d'envoi expiré),
`request_closed_in_queue` (demande suivie close ou expirée), `cancelled_in_queue`
(retrait demandé).

## Enregistrement d'envoi d'origine

À tout retrait : `idempotency_records` passe à `state = 'terminal'`,
`public_result_kind = 'rejected'`, `public_result_category = <raison>`,
`public_result_reason = "<texte>"`. Le rejeu de la clé d'envoi renvoie ce refus.
Aucun nouveau type `PublicResult`.

## Table `queue_edits` (lot B)

```sql
CREATE TABLE IF NOT EXISTS queue_edits(
  issuer_scope TEXT NOT NULL,
  request_id TEXT NOT NULL,
  canonical_bytes BLOB NOT NULL,      -- action, cible, message_id, before_message_id
  action TEXT NOT NULL CHECK (action IN ('cancel','reorder','promote')),
  actor TEXT NOT NULL,                -- identité attestée
  delivery_id TEXT NOT NULL,
  before_delivery_id TEXT,
  state TEXT NOT NULL CHECK (state IN ('applied','refused')),
  refusal TEXT,
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL,        -- horizon de rejeu : 7 jours
  PRIMARY KEY (issuer_scope, request_id)
);
CREATE INDEX IF NOT EXISTS idx_queue_edits_expires ON queue_edits(expires_at);
CREATE INDEX IF NOT EXISTS idx_queue_edits_delivery ON queue_edits(delivery_id);
```

- Reçu complet et trace d'audit à la fois. Écrit dans la **même transaction** que l'effet : un effet sans reçu ou un reçu sans effet n'existent pas.
- Deux états seulement. Une édition est toujours finale (`applied` ou `refused`). Pas de `reserved`, `dispatched`, `unknown`.
- Même clé et mêmes paramètres : même résultat. Autres paramètres : `envelope_mismatch`. Les refus sont mémorisés.
- Purge à l'entretien horaire, après `expires_at` seulement.
- Pas d'arriéré à plafonner (les éditions sont synchrones).
- Aucune clé étrangère : la ligne peut survivre à la purge de la remise (audit pendant 7 jours).

## Notice d'auteur

Réutilise `orphan_emitter_notices(delivery_id, sender, body, created_at, notified_at)`
(`idempotency.rs:835-841`), écrite dans la transaction du retrait et vidée par le
mécanisme existant. Texte : « RETIRÉ DE LA FILE : votre message <id> pour <cible>
a été retiré avant livraison (<raison>). Il ne sera pas livré. »

## Déclaration du porteur (lot B)

`Register` gagne deux champs optionnels, absents des octets quand ils sont faux :
`turn_attest: bool` (signal de tour véridique et réattesté, lot A) et
`queue_fallback: bool` (lot B). `queue_fallback` : le porteur sait enfiler en file ordinaire un message de guidage
qu'il ne peut pas appliquer. Le daemon garde les deux déclarations sur la
présence. La rétention exige `turn_attest` ; la promotion exige `queue_fallback`.

## Marqueur de promotion

Référence de message `steer-or-queue` (`BridgetMessage.references`) sur la trame de
remise d'une promotion. Absente de l'enveloppe stockée et du canon.

## Faits de la garde (extension de `guidance.rs`, 156)

```text
Actions ajoutées : QueueCancel | QueueReorder | QueuePromote
Faits ajoutés    : actor_is_author: bool, message_editable: bool,
                   recipient_turn_active: bool, carrier_queue_fallback: bool
Refus ajoutés    : NotEditable | NoActiveTurn
```

- `QueueCancel` : humain attaché, parent, auteur. Admis pendant la pause.
- `QueueReorder` : humain attaché, parent. Admis pendant la pause.
- `QueuePromote` : droits de `Steer` (156), refus de pause pour un agent, `NoActiveTurn` si le destinataire n'est pas `busy`, `steer_unavailable` si `carrier_queue_fallback` est faux.
- `NotEditable` = la remise n'est pas `held`. Le code de refus public est `not_held`.
- `BareClient`, preuve enfant, identité non attestée : refus, comme en 156.

## Trames (lot B)

| Sens | Trame | Champs |
|---|---|---|
| client vers daemon | `QueueEditRequest` | `request_id`, `target`, `action`, `message_id`, `before_message_id?` |
| daemon vers client | `QueueEditResult` | `request_id`, `state`, `refusal?`, `effect?` |

Capacité client : `QueueEditV1`. Aucune trame daemon vers porteur n'est ajoutée
(plus de `QueueEdit` ni de `QueueEditReported`). `QueueList` de 156 reste.

## États

Aucun nouvel état de tâche, d'exécution ou de projection Lineage.
