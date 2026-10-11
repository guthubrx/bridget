# Modèle de données - Session158a

Aucun champ n'est ajouté à la tâche native (le budget de tours est retiré).

## État d'arrêt (nouvelle table, base de délégation)

```sql
CREATE TABLE IF NOT EXISTS native_delegation_halt(
  id INTEGER PRIMARY KEY CHECK (id = 1),
  halted INTEGER NOT NULL CHECK (halted IN (0, 1)),
  since INTEGER,
  by_actor TEXT,
  reason TEXT,
  generation INTEGER NOT NULL
);
INSERT OR IGNORE INTO native_delegation_halt(id, halted, generation) VALUES (1, 0, 0);

CREATE TABLE IF NOT EXISTS native_delegation_halt_commands(
  command_id TEXT PRIMARY KEY,
  applied_at INTEGER NOT NULL,
  response_json TEXT NOT NULL
);
```

- Une seule ligne d'état. `reason` borné à 512 caractères (`MAX_REASON_CHARS`, `referent_control.rs:20`). `generation` augmente à chaque vrai changement.
- Reçus de commandes : 256 lignes au plus, la plus ancienne est purgée à l'insertion. Ils sont distincts de l'état et ne servent pas d'audit.
- Règle : un `command_id` présent dans les reçus rend la réponse ENREGISTRÉE (état au moment de la commande, nombre d'annulées, `replayed: true`), sans réécriture et sans parcours d'annulation. Le reçu et le changement d'état sont écrits dans la même transaction. L'annulation en cascade suit après le commit, une seule fois : un crash entre les deux perd la fin de l'annulation, jamais ne la rejoue. Séquence A, B, C (nouvelle tâche), rejeu de A : C reste active. Séquence A (arrêt), B (reprise), rejeu de A : l'état reste « repris ». Un `command_id` plus vieux que 256 commandes serait ré-appliqué : la CLI génère un identifiant neuf à chaque geste, donc seul un client qui rejoue un geste très ancien est concerné. Limite documentée.
- Persistant : les reçus survivent au redémarrage du daemon.
- Aucune relation avec `control_state` ni `control_events` (pas de migration de `CHECK`).

## Décision d'admission commune (`referent_control.rs`)

`admit_native_delegation(control: &ControlStateFrame, halt: &HaltState) -> Admission`

| Pause | Arrêt | Résultat |
|---|---|---|
| non | non | `Admitted` |
| oui | non | `Deferred { motif: "pause" }` |
| non | oui | `Deferred { motif: "delegations_halted" }` |
| oui | oui | `Deferred { motif: "delegations_halted" }` (l'arrêt est prioritaire) |

Le rejeu d'une clé admise précède cette décision (ordre de 150).

## Ligne de la vue globale (projection, rien de stocké)

| Champ | Source |
|---|---|
| `task_id` | `Task.task_id` |
| `owner` | `Task.owner` |
| `child` | `Task.child` |
| `state` | `Task.state` |
| `age_secs` | now moins `created_at` |
| `deadline_in_secs` | `mission_deadline_at` moins now, nul si absent |
| `parent_task_id` | `Task.parent_task_id` |

Jamais présents : corps de la mission, résultat, chemins de fichiers.

## Table des portées (`mcp_scopes.rs`, constante)

```text
Scope = Reads | WritesOwnState | Sends | WritesThreads | Delegates | Publishes
ToolScope { tool, base: Scope, read_actions: &[&str], own_state_actions: &[&str] }
```

- `scope_of(tool, arguments)` : si `action` est dans `read_actions` → `Reads` ; dans `own_state_actions` → `WritesOwnState` ; sinon `base`. Cas particulier lu dans les arguments : `bridget_journal` a pour base `Sends` et passe à `Reads` sans `to`.
- `readOnlyHint` dérive de la base seule (jamais de l'abaissement) : faux pour `bridget_journal`.
- Ordre de privilège pour le plafond : `Reads` < `WritesOwnState` < `Sends` = `WritesThreads` = `Publishes` < `Delegates`.
- « Contrôle humain » n'est pas une valeur de `Scope`.
- Plafond `read_only` : autorise `Reads` et `WritesOwnState`. Plafond `standard` : tout.

## Plafond d'appelant (calculé à chaque appel MCP, non stocké)

`ceiling(identity)` = `read_only` s'il existe une tâche native T telle que `T.child == identity.name`, `T.cleanup_done == false` et `T.effective_posture.or(T.request.posture) == Some(discovery)`. Sinon `standard`. Une tâche des versions 148 et 149 sans posture donne `standard` (compatibilité).
