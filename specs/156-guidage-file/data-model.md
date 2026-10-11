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
- Purge à 24 h dans `run_maintenance`. 24 h couvre la fenêtre de rythme (10 min) ; l'horizon de rejeu (7 jours) vit dans `idempotency_records`.

## Faits de la garde (`guidance.rs`, valeurs sans I/O)

```text
GuidanceFacts {
  action: Steer | Interrupt | MidTurnDelivery,   // MidTurnDelivery : guidage implicite T3
  path: SendIdempotent | Send | Control | Cancel,
  actor: AttachedHuman | BareClient | Agent{ id } | InternalPause,
  identity_attested: bool,
  child_proof: bool,            // preuve enfant ADR 044
  actor_is_target: bool,
  actor_is_parent: bool,        // agent_link ouvert ou tâche native propriétaire
  actor_is_child: bool,         // seulement pour MidTurnDelivery
  cross_project: bool,
  paused: bool,
  target_has_active_native_task: bool,
  recent_interrupts_pair: u32,  // 30 s
  recent_interrupts_target: u32 // 10 min
}
GuidanceRefusal =
  IntentNotPermitted{ reason: not_parent | cross_project | self_target | identity_unattested }
  | ReferentPaused | MissionActive | InterruptRateLimited{ retry_after_secs }
```

La garde est pure. `InternalPause` est admis sans compter. `BareClient` est
refusé `identity_unattested` pour `Steer`, `Interrupt` et `MidTurnDelivery`.
`MidTurnDelivery` est admis pour `AttachedHuman`, parent et enfant de la cible ;
elle n'est ni limitée en rythme ni comptée, mais refusée pendant la pause. L'origine
(`origin=Human`) est normalisée avant la garde par une fonction distincte du
daemon (FR-009) : un client non humain voit son origine retirée.

## Trames (protocole)

| Sens | Trame | Champs |
|---|---|---|
| client vers daemon | `QueueList` | `target` |
| daemon vers client | `QueueListResult` | `entries[]`, `queue_bound` |

Capacité négociée : `QueueListV1`. Ajout à `ExecutionControlRefusal` :
`not_permitted`. Champs ajoutés (optionnels, anciens lecteurs inchangés) :
`DeliverIdempotent.mid_turn_ok` et `CancelDelivery.interrupt_active` (défaut faux).
Aucune trame daemon vers wrapper n'est créée.

## États

Aucun nouvel état de tâche, d'exécution ou de projection Lineage.
