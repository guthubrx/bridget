# Modèle de données - Session150

## Tâche native (`delegation.rs`, struct `Task`, payload JSON)

Champ ajouté, optionnel pour garder les payloads 148-149 lisibles :

| Champ | Type | Défaut si absent | Rôle |
|---|---|---|---|
| `reply_timeout` | `Option<i64>` | 3600 | Délai de réponse choisi à la délégation |
| `delivery_acked` | `bool` | `false` | Preuve durable que la remise du résultat ou de l'avis d'échec a été accusée |

Aucun nouvel état. États terminaux inchangés : `result_available`, `failed`,
`cancelled`. La vue Lineage T3 ne change pas.

## Arbre purgeable

Arbre = une tâche sans `parent_task_id` et tous ses descendants.

Purgeable si chaque tâche de l'arbre :

- est terminale et a `cleanup_done = true` ;
- si elle est `result_available` ou `failed` : a `delivery_acked = true` ;
- n'a pas de sous-agent vivant : ni présence connectée, ni lancement en cours
  (`managed_spawns`, sagas de lancement), ni entrée de flotte désirée
  `Running`.

Avant suppression, l'entrée de flotte de chaque enfant est décommissionnée.

Âge de l'arbre : maximum de `completed_at` (ou `updated_at` si absent).

Purge normale : âge plus vieux que 7 jours. Purge de borne : total ≥ 4096,
arbres purgeables du plus ancien au plus récent. Au plus 128 arbres par
passage.

## Pierre tombale (nouvelle table)

```sql
CREATE TABLE IF NOT EXISTS native_delegation_tombstones(
  owner TEXT NOT NULL,
  request_id TEXT NOT NULL,
  purged_at INTEGER NOT NULL,
  PRIMARY KEY(owner, request_id)
);
```

`owner` reprend la clé de rejeu existante (`by_agent_request`,
`delegation.rs:114-128`). Écrite dans la même transaction que la suppression.
Purgée après 30 jours.

## Suppression atomique d'un arbre

Dans une transaction :

1. insérer les pierres tombales des tâches de l'arbre ;
2. supprimer leurs reçus `native_delegation_cancel_receipts` ;
3. supprimer les lignes `native_delegations` ;
4. changer la génération de `native_delegation_projection_meta`.

Après le commit : signaler chaque propriétaire racine touché via
`committed()`.

## Enum de contrôle (`referent_control.rs`)

`AutonomousEffect::NativeDelegation` : admission d'une délégation native.
