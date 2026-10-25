# Archives techniques de l’adaptateur T3

Document d’origine : [specs/159-edition-file/checklists/requirements.md](../../../../../../../specs/159-edition-file/checklists/requirements.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Les noms d'outils et d'actions (`bridget_queue`, `cancel`, `promote`, `reorder`),
  d'états (`held`, `released`, `cancelled`) et de refus (`not_held`,
  `no_active_turn`, `steer_unavailable`, `queue_full`, `envelope_mismatch`,
  `referent_paused`) sont le contrat public du produit.
- Les exigences d'entrée E1 à E6 viennent de 156 ; chacune a une exigence, une
  tâche et un critère de succès (voir `analyze-report.md`).
- Conception refondue après contre-revue : la file est tenue par le daemon ; les
  11 points ont chacun un sort dans `adversarial-review-codex.md`.
- Cas limites couverts : arrêt du daemon ou du porteur à chaque étape, signal de
  fin de tour perdu, réaffectation d'instance, arrivée concurrente pendant un
  déplacement, demande close pendant la rétention, file pleine, message de
  mission native, fils T3, réglage fermé, porteur ancien, migration rejouée,
  retour à un binaire antérieur, pause, preuve enfant, client nu.
- Découpage : lot A (159a) livrable seul, réglage fermé par défaut ; lot B (159b)
  dépend de A. `edit` et les fils T3 sont hors périmètre, justifiés.
- Trois questions ouvertes, défauts prudents. Aucune ne bloque la planification.
- Cadre de menace repris de 156 (ADR 011).
