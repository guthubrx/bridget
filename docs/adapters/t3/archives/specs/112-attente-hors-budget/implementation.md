# Archives techniques de l’adaptateur T3

Document d’origine : [specs/112-attente-hors-budget/implementation.md](../../../../../../specs/112-attente-hors-budget/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

## Correction (`t3code.rs`)
- `deadline_after_wait(deadline, received)` crédite l'échéance du temps passé en file.
- Les deux contrôles avant dispatch n'écartent une remise pour échéance de tour que si elle était
  déjà passée à la réception (le report ne ressuscite rien).
- Au dispatch, `deadline_at` est reporté sur le message injecté : le budget d'exécution court à
  partir du dispatch effectif, pour l'enveloppe et les contrôles ultérieurs.
- `reply_timeout`, expiration de saga et demande suivie : inchangés. Aucun changement côté daemon.
