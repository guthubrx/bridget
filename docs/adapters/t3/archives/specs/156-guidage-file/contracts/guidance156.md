# Archives techniques de l’adaptateur T3

Document d’origine : [specs/156-guidage-file/contracts/guidance156.md](../../../../../../../specs/156-guidage-file/contracts/guidance156.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Code | Quand | Mémorisé |
|---|---|---|
| `invalid_intent` | valeur inconnue ou steer+reply=true | non (aucune clé réservée) |
| `intent_not_permitted` (motif : `not_parent`, `cross_project`, `self_target`, `identity_unattested`) | appelant sans droit | oui, par `reject_idempotent_send` |
| `referent_paused` | `steer`/`interrupt` d'un agent pendant la pause | oui |
| `mission_active` | interrupt agent vers enfant en mission ; humain explicite exempt | oui |
| `interrupt_rate_limited` (`retry_after_secs`) | 1 par 30 s par couple, 3 par 10 min par cible | oui |
| `steer_unavailable` | fil T3 dont le fournisseur n'a pas de guidage natif prouvé | rapporté par le pont comme remise rejetée |

<a id="extrait-2"></a>

## Extrait 2

`bridget_send` (MCP), `bridget send` (CLI), `SendIdempotent`, `Send`
historique (y compris `intent` bruts `SteerCurrent`/`InterruptAndStart` et
`origin=Human`, en défense en profondeur), `ControlExecution` (`SteerCurrent`,
`Interrupt`, avec normalisation de l'origine du message), guidage implicite du
pont T3, `bridget_cancel`. Exception interne : l'interruption de pause (sans
connexion cliente).

<a id="extrait-3"></a>

## Extrait 3

| Champ | Sens |
|---|---|
| `entries[]` | au plus 200 ; `message_id`, `from`, `intent`, `reply`, `age_secs`, `preview` (120 caractères), `position`, `state: "pending_or_injecting"` |
| `queue_bound` | 64 pour un fil T3, `null` sinon |
