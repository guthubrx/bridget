# Modèle de données - Session152

## Tâche native (`delegation.rs`, struct `Task`, payload JSON)

Tous les champs sont optionnels à la relecture (`#[serde(default)]`). Les
payloads 148 à 151 se relisent sans migration.

| Champ | Type | Défaut | Rôle |
|---|---|---|---|
| `launch_issued_at` | `Option<i64>` | `None` | Date d'émission de l'ordre de lancement, posée et sauvegardée avant la première soumission. `created_at` si absent |
| `mission_issued_at` | `Option<i64>` | `None` | Date d'émission de la mission, posée avant le premier envoi. `created_at` si absent |
| `restart_resumes` | `u8` | 0 | Redémarrages du daemon survécus par la mission. Incrémenté à la décision de reprise |
| `cancel_stopped` | `Vec<String>` | vide | Noms déjà arrêtés par une annulation en cascade (tâche racine) |

Valeurs dérivées, jamais stockées :

- **suspendue** : pause active et état `queued`, `starting`, `mission_pending`
  ou `working`.
- **échéance effective** : `mission_deadline_at + pause_overlap(mission_deadline_at - reply_timeout, now)`.
- **temps de pause du suivi** : `pause_overlap(request.created_at, now)`, où
  `request.created_at` est la date de la ligne durable `tracked_requests`.
- **commande de flotte courante** : commande de l'entrée de flotte du nom de
  l'enfant dont `agent_link.delegation_id == task_id` et `role == native_delegate`,
  sinon `task_id`.

Aucun nouvel état.

### Invariants

- I1 : aucune écriture n'est faite pour suspendre ou lever une suspension.
- I2 : `mission_deadline_at` n'est jamais réécrite après sa pose.
- I3 : `launch_issued_at` est sauvegardé avant le premier `submit_spawn` et ne
  change plus. `mission_issued_at` est sauvegardé avant le premier envoi.
- I4 : `restart_resumes` ne change que dans la décision de reprise de
  `prepare_restart`, avant toute reprise. Un crash ne peut que le sur-compter.
- I5 : `cancel_stopped` ne contient que des noms dont l'arrêt est confirmé.
- I6 : une tâche dont l'identité ne se réconcilie pas passe `failed`
  (`native_restart_identity_mismatch`) ; elle ne bloque jamais le démarrage.

## Journal des pauses

`pause_overlap(conn, from, to)` (dans `referent_control.rs`) additionne les
secondes de pause entre `from` et `to`. Sources : `control_events` (`pause_on`,
`pause_off`, colonne `at`) et `control_state` (`paused`, `paused_since`) pour
l'intervalle encore ouvert. L'état au début de la fenêtre vient du dernier
événement avant `from`. Complexité : O(P), P = événements dans la fenêtre.
Appelée seulement quand le résultat sert : décision d'expiration, suivi d'une
demande native, décision de reprise.

## Machine d'état (résumé)

```text
working --pause--> working (suspendue, dérivé ; aucune écriture)
working (suspendue) --levée + interrupted--> working (nouvelle exécution de la chaîne)
working --redémarrage, identité, échéance effective, restart_resumes < 2--> working (restart_resumes + 1, sauvegardé avant la reprise)
working --redémarrage, restart_resumes = 2--> failed (native_resume_exhausted)
working --redémarrage, échéance effective passée--> failed (mission_reply_timeout)
working --redémarrage, identité irréconciliable--> failed (native_restart_identity_mismatch)
starting | mission_pending --redémarrage--> même règle, sans exécution de mission
interrupting + ligne de pause --redémarrage--> interrupted
```

## Chaîne d'exécutions

Racine : `execution-{mission}`. Reprise : `execution-recovery-<uuid>`, reliée par
`execution_continuations(parent_execution_id)`. `latest_in_chain(racine)` rend la
dernière exécution créée de la chaîne.

```sql
WITH RECURSIVE chain(id) AS (
  SELECT ?1
  UNION
  SELECT c.execution_id FROM execution_continuations c JOIN chain p ON c.parent_execution_id = p.id)
SELECT e.execution_id FROM executions e JOIN chain ON e.execution_id = chain.id
WHERE NOT EXISTS (SELECT 1 FROM execution_continuations k WHERE k.parent_execution_id = e.execution_id)
ORDER BY e.generation DESC LIMIT 1;
```

La dernière exécution est la feuille de plus grande `generation`. Chaque
reconstruction crée la génération `parent + 1` (`execution_store.rs:1402`). Le tri
ne dépend ni de la date en secondes ni de l'identifiant aléatoire.

Complexité : O(c), c = nombre de pauses et de reprises de la mission. Non borné
par une constante, petit en pratique.

## Ligne de pause (existante)

`control_pause_interruptions(execution_id, target_agent, control_generation,
requested_at, resumed_at, resume_execution_id)`. Ajout d'une fonction
`abandon_pause_interruption(execution_id)` : pose `resumed_at`, laisse
`resume_execution_id` nul. Pas de changement de schéma.

## Suivi de demande (mémoire du daemon)

`PendingReply` gagne `frozen_secs: u64`. Affecté (jamais incrémenté) à chaque
tick et dans `restore_pending_for_agent` : `pause_overlap(created_at_durable, now)`.
Temps écoulé : `now - created_at - frozen_secs`. Une demande de mission native
est ignorée par `collect_reminder_actions` tant que la pause est active.
`tracked_requests` ne change pas.

## Reçu de refus (nouvelle table)

```sql
CREATE TABLE IF NOT EXISTS native_delegation_refusals(
  owner TEXT NOT NULL,
  request_id TEXT NOT NULL,
  canonical BLOB NOT NULL,
  code TEXT NOT NULL,
  refused_at INTEGER NOT NULL,
  PRIMARY KEY(owner, request_id)
);
```

`owner` et `request_id` reprennent la clé de rejeu de `by_agent_request`.
Écriture seulement pour un code de la liste fermée du contrat. Purge au-delà de
30 jours dans l'entretien. Un rejeu avec le même canonique rend `code`. Un autre
canonique rend `envelope_mismatch`. Si une tâche existe déjà pour la clé, elle
prime (aucun reçu n'est alors écrit).

## Classification d'arrêt (`classify_stop`, mémoire)

`StopKind` : `Supervised`, `Marker`, `AlreadyStopped`, `NotFound`, `Unmanaged`.
Pure, sans effet. `Unmanaged` empêche tout arrêt (FR-019).

## Garde (`referent_control.rs`)

`AutonomousEffect::NativeMissionDispatch` : lancement d'une tâche native et
remise de sa mission. Pause : `Deferred { motif: "pause" }`. Sinon `Admitted`.
