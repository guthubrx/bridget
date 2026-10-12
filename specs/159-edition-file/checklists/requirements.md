# Liste de contrôle qualité de la spécification : Édition de la file de remise

**Objet** : Vérifier la complétude et la qualité de la spécification avant la planification
**Créée le** : 2026-10-11
**Fonctionnalité** : [spec.md](../spec.md)

## Qualité du contenu

- [x] Aucun détail d’implémentation (langages, frameworks, API)
- [x] Centrée sur la valeur pour l’utilisateur et les besoins métier
- [x] Rédigée pour des lecteurs non spécialistes
- [x] Toutes les sections obligatoires sont complètes

## Complétude des exigences

- [x] Aucun marqueur [NEEDS CLARIFICATION] ne subsiste
- [x] Les exigences sont testables et sans ambiguïté
- [x] Les critères de succès sont mesurables
- [x] Les critères de succès ne dépendent pas d’une technologie (aucun détail d’implémentation)
- [x] Tous les scénarios d’acceptation sont définis
- [x] Les cas limites sont identifiés
- [x] Le périmètre est clairement borné
- [x] Les dépendances et les hypothèses sont identifiées

## Préparation de la fonctionnalité

- [x] Toutes les exigences fonctionnelles ont des critères d’acceptation clairs
- [x] Les scénarios utilisateur couvrent les parcours principaux
- [x] La fonctionnalité satisfait les résultats mesurables définis dans les critères de succès
- [x] Aucun détail d’implémentation ne fuit dans la spécification

## Notes

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
