# Contrat - Contrôle des délégations 158a

Compatible avec les contrats 148 à 150. `bridget_delegate` ne change pas de
schéma. Un appel sans nouveau champ produit les mêmes octets et le même
comportement qu'en 150.

## Refus ajoutés

| Code | Quand | Écriture |
|---|---|---|
| `delegations_halted` | arrêt global posé, nouvelle clé (renvoyé dans `native_delegation_refused`) | aucune ; la clé reste libre |
| `capability_denied` (MCP) | appel hors plafond de l'appelant | aucune |

Ordre des refus de `bridget_delegate` (150 + 158a) : `identity_unavailable`,
`invalid_request`, `reply_timeout_out_of_range`, rejeu (`envelope_mismatch`,
`request_expired`), décision commune d'admission (`delegations_halted`,
`referent_paused`), gardes de droits 149, `task_limit`, `retention_saturated`,
lignée parente, `reply_timeout_exceeds_parent`.

Un rejeu d'une clé existante pendant l'arrêt rend la tâche existante.

## `bridget_capabilities` - clé ajoutée

`delegations_halted` : booléen, état courant de l'arrêt.

## Protocole de contrôle (socket Unix)

Toutes les requêtes exigent la négociation `ControlStateV1` ET un acteur humain
attesté (`control_client_actor`). `ControlStateV1` seule ne suffit pas : elle
est accordée à tout client qui la demande.

| Requête | Réponse |
|---|---|
| `DelegationOverviewRead { version, state: active\|all, limit 1..100 }` | `DelegationOverview { halted, items[], total_active }` |
| `DelegationHaltSet { version, command_id, halted: bool, reason?, cancel_active: bool }` | `DelegationHaltState { halted, since, by, reason, generation, cancelled: n }` |
| `DelegationCancel { version, task_id, request_id }` | `DelegationOverview` (ligne mise à jour) |

Refus fermés (`DelegationControlRejected { reason }`) : `unsupported_version`,
`invalid_request`, `negotiation_required`, `capability_not_negotiated`,
`human_actor_required`, `task_unavailable`, `store_unavailable`.

`cancel_active` n'est admis que si `halted = true`. `command_id` connu : la
réponse enregistrée est rendue (`replayed: true`), sans réécriture et sans aucun
parcours d'annulation. L'annulation ne s'exécute qu'à la première application
et vise les arbres actifs à cet instant. Elle est au plus une fois.

## CLI

```text
bridget control delegations [list] [--state active|all] [--limit N] [--json]
bridget control delegations cancel <task_id>
bridget control halt-delegations [--reason <texte>] [--cancel-active]
bridget control resume-delegations
bridget control status           # ajoute la ligne d'arrêt s'il est posé
```

`halt-delegations`, `resume-delegations` et `delegations cancel` exigent un
terminal interactif. Codes de sortie : 0 succès, 1 refus ou erreur daemon, 2 usage.

## Portées MCP (`mcp_scopes.rs`)

| Outil | Base | Lecture | État propre |
|---|---|---|---|
| `bridget_capabilities`, `bridget_who`, `bridget_status`, `bridget_control_status`, `bridget_task_status`, `bridget_read_artifact`, `bridget_ledger`, `guichet_request_status` | `reads` | toutes | - |
| `bridget_rename`, `bridget_dnd`, `bridget_domain`, `bridget_runtime` | `writes_own_state` | - | toutes |
| `bridget_events` | `writes_own_state` | `types`, `list` | `sub`, `unsub` |
| `bridget_thread` | `writes_threads` | `list`, `show`, `history` | `read`, `ack` |
| `bridget_journal` | `sends` (abaissé à `reads` sans `to`) | sans `to` | - |
| `bridget_handoff` | `sends` | `preview` | - |
| `bridget_send`, `bridget_cancel` | `sends` | - | - |
| `bridget_publish_artifact` | `publishes` | - | - |
| `bridget_delegate`, `bridget_task_cancel`, `guichet_delegate`, `guichet_registre_add`, `guichet_objective_close` | `delegates` | - | - |
| outils ajoutés par 151 à 157 | à classer en T015 | | |

Cette table est indicative. Le test exhaustif fait foi. Aucune ligne n'a de
portée « contrôle humain ».

`readOnlyHint` = vrai si et seulement si la portée de base est `reads`. Pour
`bridget_journal`, la base est `sends` : `readOnlyHint` est faux.

Plafond `read_only` (enfant `discovery`) : seuls `reads` et `writes_own_state`.
Ruptures voulues (politique FR-029) : refusés `bridget_thread` `post`, `create`,
`add_members`, `close` ; `bridget_journal` avec `to` ; `bridget_publish_artifact` ;
`bridget_send`, `bridget_cancel`, `bridget_handoff` en envoi ; toute délégation
(`capability_denied`, « outil hors du plafond de cette tâche, posture
discovery »). Restent permis : `read` puis `ack` de fil (pas de rejeu de page),
`bridget_read_artifact`, et la réponse finale automatique au parent, qui ne
passe pas par un outil MCP.

## Compatibilité

- Un appelant sans tâche `discovery` garde tous ses outils.
- La vue Lineage T3 ne change pas.
