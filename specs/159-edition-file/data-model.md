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
| `queue_rank` | absente | `INTEGER PRIMARY KEY AUTOINCREMENT` ; rang global monotone durable après purge |
| `delivery_id` | clé primaire historique | `UNIQUE NOT NULL` ; cible des clés étrangères existantes conservée |
| `recipient_agent` | absente | `TEXT NOT NULL` ; identifiant d'agent résolu à l'admission ; rempli à la migration par `json_extract(message_bytes,'$.to')` |
| `turn_gate` | absente | `INTEGER NOT NULL DEFAULT 0 CHECK (turn_gate IN (0,1))` ; réservation de tour |
| `acked_at` | absente | `INTEGER` nullable ; date du premier ACK, écrite avec sa phase pour lever la réservation après 30 s, même après redémarrage |

- Lignes existantes : `queue_rank` = ancien `rowid`.
- Nouvelle ligne : INSERT sans queue_rank dans la transaction Immediate de begin_send_delivery_with_guidance ; SQLite attribue le rang via AUTOINCREMENT. La séquence durable survit à la purge ; aucun nouveau compteur ni table métier. delivery_id reste UNIQUE NOT NULL et cible des FK.
- `begin_send_delivery_with_guidance` prend un mode : `Dispatching` (existant) ou `Held` (nouvelle phase d'insertion). En mode `Held` : pas de lien d'exécution, enregistrement d'idempotence en `dispatching`, ledger et suivi de réponse écrits comme aujourd'hui.
- Réaffectation et orphelinage : s'appliquent à `dispatching` et `held`. Le rang ne change pas.
- `dispatching_deliveries_for_instance` : inchangé de contrat (ne rend que `dispatching`), trié par `queue_rank` au lieu de `delivery_id`.
- Index complémentaire : `idx_send_deliveries_agent_phase ON (recipient_agent, phase, queue_rank)` pour le rattachement par agent et la réservation de tour.
- Index : `idx_send_deliveries_operation` (unique), `idx_send_deliveries_kind_key`, index existant non unique `idx_send_deliveries_recipient_phase`, étendu à `(recipient_instance_id, phase, queue_rank)` ; aucun second index recipient redondant.
- `send_delivery_execution_links` (`ON DELETE CASCADE`) : copiée avant le `DROP`, réinsérée après le renommage. `PRAGMA foreign_key_check` à zéro.
- Migration enregistrée : `idempotency_schema_migrations(version) = 7`. `MAX_IDEMPOTENCY_SCHEMA_VERSION` passe de 6 à 7 (`store_schema.rs:4`).

Phase `cancelled` : terminale, absorbante. Jamais relivrée, jamais listée comme
`dispatching`, purgée avec son enregistrement d'envoi (purge existante).

## Réservation de tour et rattachement (lot A)

- `turn_gate = 1` : posé par la libération, sur la remise libérée. Levé par : `TurnState true` observé pour l'agent après la libération ; sortie de `dispatching` sans accusé ; accusé depuis plus de 30 s avec l'agent non occupé. Une libération exige qu'aucune remise de `recipient_agent` n'ait `turn_gate = 1`.
- `adopt_held_for_agent(agent_id, instance_id, recover_dispatching)` : appelé après établissement de l’identité finale de `Register`. Rattache les lignes `held` du même agent, même si l’ancienne présence a disparu.
  Si `recover_dispatching` est vrai, rattache aussi les lignes `dispatching` non expirées d’une autre instance du même agent et pose `turn_gate = 1`. Le caller active ce mode seulement pour une instance `turn_attest`, un type `held_queue` ouvert et un transport hors T3.
  Les trois scopes internes de réveil et les alertes de fil restent exclus de ce raccord `dispatching`. ID, génération, octets, phase et rang sont conservés ; une réinscription de la même instance ne réarme pas ses réservations courantes. Le chemin `held` reste inchangé.

### Présence (mémoire du daemon)

| Champ | Rôle |
|---|---|
| `busy_period_start` | début de la période occupée ; non remis à zéro par un nouveau `true` |
| `last_attested_at` | date de la dernière attestation de l'état (vrai ou faux) |
| `turn_attest` | l'instance a déclaré un signal de tour véridique et réattesté |

`busy_since` garde son comportement legacy. Pour toute instance turn_attest, la sortie forcée utilise busy_period_start, même si held_queue est fermé. Les instances anciennes non attestées gardent le calcul legacy sur busy_since. La borne par défaut de 2730 s vaut notify_timeout_secs = 2700 s + grâce de 30 s ; notify_timeout_secs reste configurable.

La minuterie Instant du heartbeat existant est vérifiée à chaque passage de la boucle, même sous trafic continu. Elle déclenche une réattestation vraie périodique (30 s au plus), sans nouveau scheduler.

### Suivi mémoire

`held_ids` : ensemble en mémoire des identifiants de messages retenus,
reconstruit depuis la base au démarrage, tenu à jour à la rétention, la
libération et tout retrait. Les paliers 1 et 2 des relances ignorent les demandes
dont le message est dans `held_ids`. Après un commit de retrait, l'entrée de
`pending_replies` du message est retirée.

Garde de libération (décision ROOT) : conserver le mutex State de la lecture du repos jusqu’au commit de Held vers Dispatching. La même transaction Immediate d’IdempotencyStore lit control_state et control_pause_interruptions dans config.db_path, partagé par les magasins existants. Préserver l’ordre de verrous existant ; aucun writer socket ni autre connexion SQL dans cette transaction. La libération est linéarisée au commit. Le push différé se fait hors verrou ; une pause validée après ce commit ne reprend pas une remise déjà libérée. Pas de table miroir. Les oracles concurrents de T007 déjà prévus couvrent l’ordre pause/TurnState true ; aucun résultat runtime nouveau n’est annoncé.

## Opérations de la phase `held`

| Opération | Transaction | Effets |
|---|---|---|
| Rétention | `begin_send_delivery_with_guidance` mode `Held` | ligne `held`, rang, ledger, suivi de réponse |
| Libération | `release_next_held(agent, now)` | vérifie réservation, saga, demande ouverte et échéance ; plus petit rang `held` passe à `dispatching` avec `turn_gate = 1` ; rend `SendDelivery` |
| Retrait automatique | `settle_held(delivery_id, reason)` | `cancelled`, `Rejected(<raison>)`, notice d'auteur |
| Retrait demandé (lot B) | `apply_cancel` | idem + reçu `applied` |
| Déplacement (lot B) | `apply_reorder` | permutation des valeurs de rang existantes du groupe ; rangs provisoires négatifs dans la même transaction pour éviter une collision de clé primaire, jamais publiés ; reçu |
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

Le réordonnancement ne fait aucun nouvel INSERT de rang. Aucun CHECK positif ne bloque ses valeurs provisoires négatives ; la transaction ne publie que la permutation des rangs existants. AUTOINCREMENT ajoute un coût SQLite/CPU/E/S, justifié par FR008 qui interdit le recyclage après purge. Références : [SQLite AUTOINCREMENT](https://www.sqlite.org/autoinc.html), [SQLite Foreign Keys](https://www.sqlite.org/foreignkeys.html).

Les lectures control_state et control_pause_interruptions sont exécutées après BEGIN IMMEDIATE, dans cette même transaction. En cas de SQLITE_BUSY, la remise reste held : aucun refus, aucune libération ni push ; le prochain déclencheur réessaie. L’attente SQL peut durer busy_timeout (2 s) sous State ; ce risque de latence sera mesuré, sans nouvelle garantie. Les chemins TurnState et pause prennent ce même mutex ; les cas concurrents T007 prévus vérifient l’ordre.
