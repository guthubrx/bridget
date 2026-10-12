# Modèle de données - Session156

## Intention d'envoi (API) et intention posée (message)

| `intent` de l'appel | `message.intent` posé avant le canon |
|---|---|
| absent | aucune |
| `queue` | aucune |
| `steer` | `SteerCurrent` |
| `interrupt` | `InterruptAndStart` |

L'intention posée est celle qui entre dans le canon d'envoi
(`canonical_send`, `daemon.rs:23255-23270`). Elle ne change jamais après la
réservation. Aucune colonne de résultat d'effet n'est stockée.

## Reçu de rejeu

Inchangé : l'enregistrement d'idempotence existant (`idempotency_records`,
horizon de 7 jours, `daemon.rs:466`) avec son canon complet. Les refus de la
garde passent par `reject_idempotent_send`. Aucune nouvelle table n'est un reçu.

## Remise en file (table existante `send_deliveries`, inchangée)

Aucune colonne, aucune phase ajoutée. Le `CHECK` de `phase` reste
`dispatching | acked | indeterminate | orphaned` (`idempotency.rs:534`).

Lecture (`bridget_queue`) :

```sql
SELECT d.delivery_id, d.message_bytes, r.issued_at
FROM send_deliveries d
JOIN idempotency_records r
  ON r.issuer_scope = d.issuer_scope
 AND r.operation_kind = d.operation_kind
 AND r.idempotency_key = d.idempotency_key
WHERE d.recipient_instance_id = ?1 AND d.phase = 'dispatching' AND d.expires_at > ?2
ORDER BY d.rowid
LIMIT 200;
```

- Ordre : `rowid` (ordre d'insertion). La réaffectation met à jour en place (`send_delivery.rs:230`) : le `rowid` est conservé.
- `position` = rang dans ce résultat.
- `age_secs` = `now - issued_at`.
- Une ligne `dispatching` peut être en cours d'injection : d'où l'état `pending_or_injecting`.
- Send historique et files internes des transports sont exclus.
- Index ajouté (sûr, `IF NOT EXISTS`) : `idx_send_deliveries_recipient_phase ON send_deliveries(recipient_instance_id, phase)`. Sans lui, la recherche est un parcours complet (`send_delivery.rs:372-377`).

## Événement de guidage (nouvelle table d'audit et de rythme)

```sql
CREATE TABLE IF NOT EXISTS guidance_events(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  at INTEGER NOT NULL,
  actor TEXT NOT NULL,     -- identité attestée ("human" pour l'humain)
  target TEXT NOT NULL,    -- agent_id de la cible
  action TEXT NOT NULL CHECK (action IN ('steer','interrupt')),
  path TEXT NOT NULL CHECK (path IN ('send_idempotent','send','control','cancel')),
  message_id TEXT,         -- ou command_id
  outcome TEXT NOT NULL    -- 'admitted' ou 'refused:<code>'
);
CREATE INDEX IF NOT EXISTS idx_guidance_events_target
  ON guidance_events(target, action, at);
CREATE INDEX IF NOT EXISTS idx_guidance_events_pair
  ON guidance_events(actor, target, action, at);
```

- Audit et compteur seulement. Jamais lue pour rejouer un résultat.
- Comptage de rythme : `COUNT(*) WHERE target=? AND action='interrupt' AND outcome='admitted' AND at>=?`, pour les acteurs agents. L'humain et l'interruption de pause ne sont pas comptés (la pause n'écrit pas).
- Un rejeu d'une clé déjà admise n'écrit rien (la garde n'est pas rejouée).
- Projet avant admission guidance ; admitted après acceptation durable begin_send_delivery ou équivalent, jamais un refus tardif. Effet fournisseur toujours unconfirmed.
- Purge à 24 h dans `run_maintenance`. 24 h couvre la fenêtre de rythme (10 min) ; l'horizon de rejeu (7 jours) vit dans `idempotency_records`.

## Faits de la garde (`guidance.rs`, valeurs sans I/O)

```text
GuidanceFacts {
  action: Steer | Interrupt | MidTurnDelivery | QueueList,   // MidTurnDelivery : guidage implicite T3
  path: SendIdempotent | Send | Control | Cancel,
  actor: AttachedHuman | BareClient | Agent{ id } | InternalPause,
  identity_attested: bool,
  trusted_origin: Human | System | None, // MessageOrigin existant ; preuve/source interne, distincte du rôle, jamais payload seul
  child_proof: bool,            // preuve enfant ADR 044
  actor_is_target: bool,
  actor_is_parent: bool,        // agent_link ouvert ou tâche native propriétaire
  actor_is_child: bool,         // seulement pour MidTurnDelivery
  cross_project: bool,
  paused: bool,
  target_has_active_native_task: bool,
  now: i64,                    // même horloge injectée que les lectures
  recent_interrupts_pair: u32,  // COUNT inclusif sur 30 s
  last_interrupt_pair_at: Option<i64>, // LAST admis du couple acteur/cible
  recent_interrupts_target: u32, // COUNT inclusif sur 10 min
  third_latest_interrupt_target_at: Option<i64> // troisième événement cible le plus récent
}
GuidanceRefusal =
  IntentNotPermitted{ reason: not_parent | cross_project | self_target | identity_unattested }
  | ReferentPaused | MissionActive | InterruptRateLimited{ retry_after_secs }
```

Les deux lectures T006 déjà prévues renvoient `(count, timestamp)` depuis
`guidance_events`, avec `action='interrupt'`, `outcome='admitted'`, `actor!='human'`
et `at>=since`. Pour le couple : dernier événement admis. Pour la cible :
troisième événement le plus récent, pas MIN de toute la fenêtre si plus de
3 lignes existent. Aucun timestamp dérivé n’est stocké ailleurs.
Le caller transmet `now` et les deux dates ; la garde ne décide pas sur COUNT
seul. Les échéances sont `last_interrupt_pair_at+30` et
`third_latest_interrupt_target_at+600`. Une échéance `<=now` est admise, même
si COUNT inclut encore la ligne de frontière. Le délai exact est le maximum
des échéances encore futures moins now ; zéro signifie aucun blocage de rythme.
Les lectures de fenêtre coûtent O(log n + k), n événements dans la table,
k entrées candidates parcourues ; le filtre outcome n’est pas dans l’index.
Cette projection réutilise table, index et lectures T006 ; aucun service,
horloge globale, état persistant ou helper supplémentaire.

La garde est pure. `InternalPause` est admis sans compter. `BareClient` sans preuve est
refusé `identity_unattested` pour `Steer`, `Interrupt` et `MidTurnDelivery`.
Une UI auxiliaire attestée garde l'acteur BareClient et identity_attested=true ;
sa trusted_origin validée conserve MidTurnDelivery sans contrôle explicite.
MidTurnDelivery conserve les messages ordinaires déjà groupables attach, UI
attestée (identité réservée/instance primaire ui/credential validé), systèmes
internes et parent/enfant. Les pairs/clients nus ne gagnent rien par from=humain.
Le rôle Client UI est immuable ; trusted_origin est un fait local du type
MessageOrigin existant, pas une nouvelle preuve ni un champ client. Aucun
origin=Human fabriqué. Pendant la pause, AgentGuidance bloque les agents ;
humains validés et systèmes internes conservent leur voie existante, sans rythme.
Steer/Interrupt explicites gardent leur autorité FR008 ; la provenance UI ne
promet pas un nouveau droit de contrôle attach.

## Trames (protocole)

| Sens | Trame | Champs |
|---|---|---|
| client vers daemon | `QueueList` | `target` |
| daemon vers client | `QueueListResult` | `entries[]`, `queue_bound` |

Capacité négociée : `QueueListV1`. Ajout à `ExecutionControlRefusal` :
`not_permitted`. Champs ajoutés (optionnels, anciens lecteurs inchangés) :
`DeliverIdempotent.mid_turn_ok`, `Deliver.mid_turn_ok` et `CancelDelivery.interrupt_active` (défaut faux). Deliver sans ce champ garde sa forme historique.
Aucune trame daemon vers wrapper n'est créée.

## États

Aucun nouvel état de tâche, d'exécution ou de projection Lineage.

## Alignement de la garde de lecture QueueList

QueueList réutilise le même point pur pour refuser child_proof et le filtrage
R5 prévu par T024. Cette lecture n’écrit aucun guidance_event, ne consomme
aucun quota d’interruption et n’écrit aucun état de pause. InternalPause est
un acteur, pas une action. Aucune table, API ou fonction supplémentaire.

## Compatibilité des champs de remise

Deliver reste une trame historique aplatie : message porte serde(flatten),
mid_turn_ok est Option<bool> et None est omis pour conserver ses octets anciens.
DeliverIdempotent suit le même patron optionnel. CancelDelivery.interrupt_active
absent/None vaut false. QueueListResult réutilise entries:Vec<Value> et
queue_bound:Option<u32> (null lorsque la borne n’est pas connue), sans DTO neuf.

## Coût de la projection de file

L’index recipient/phase évite un scan global, mais s candidats dispatching,
expirés inclus, peuvent être examinés. Coût de parcours O(log n + s), jointures
indexées en plus ; LIMIT200 borne uniquement les lignes rendues. L’ordre rowid
et la borne200 ne sont pas une promesse de nombre de candidats visités.
