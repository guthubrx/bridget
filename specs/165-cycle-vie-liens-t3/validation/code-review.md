# Contre-revue du code — SPEC-165

Date : 2026-10-14  
Mode : lecture seule

## Verdict final R4

**APPROVE.** Aucun défaut bloquant ne reste.

Tâche : `node:delegated-task:command%3Amcp%3Ae3126f2b-7428-43cd-ad5d-737df2712318%3Adelegate-task%3Aspec165-code-review-r4-20261014`

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/validation/code-review.md#extrait-1).

## Objections closes

| Ronde | Objection | Réponse vérifiée |
|---|---|---|
| R2 | Remise perdue entre le lecteur et la file | Compteur `inbound_deliveries` et invalidation immédiate de `drained_key`. |
| R2 | Liens absents du snapshot laissés ouverts | Fermeture et nettoyage explicites. |
| R2 | Tentative consommée avant une fermeture attestée | Consommation au drain attesté ou à la borne dure. |
| R2 | Reprises en délai comptées dans le plafond | Exclusion avant la sélection des 32 reprises. |
| R2 | `lineage: null` V2 refusé | Valeur acceptée et testée. |
| R3 | Publication périmée possible après une remise | Calcul et écriture sous le même mutex ; test déterministe. |
| R3 | Remises restantes non rapportées à la fermeture | Rapport `LinkClosing` avant désinscription et après perte du daemon. |
| R3 | Couverture insuffisante des transitions | Décisions pures et chemins réels de fermeture couverts. |
| R3 | Compteurs documentaires incohérents | 13 tests SPEC, 102 tests module et 23 tests contrat consignés. |

## Observations non bloquantes

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/165-cycle-vie-liens-t3/validation/code-review.md#extrait-2).

## Conclusion

Le diff peut être intégré. La recette runtime doit encore prouver la baisse des
liens et des fichiers ouverts sous la limite normale de 4 096.
