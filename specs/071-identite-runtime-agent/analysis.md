# Analyse croisée et auto-correction - SPEC-071

**Passage 1** : 2026-08-30
**Résultat après correction** : PASS

## Matrice de cohérence

| Axe | Vérification | Résultat |
|---|---|---|
| Besoin vers exigences | les trois user stories couvrent identité, marque et interaction | cohérent |
| Exigences vers plan | FR-7101 à FR-7114 ont chacune un lot ou un garde-fou | cohérent |
| Plan vers tâches | projection, catalogue, assets, fiche, accessibilité et validation sont ordonnés | cohérent |
| Réutilisation | chaque création est justifiée dans `reuse-audit.md` | cohérent |
| Compatibilité | champs UI additionnels et optionnels, protocole `AgentInfo` inchangé | cohérent |
| Tests | témoins avant implémentation sur données, routes et interaction | cohérent |
| Charge cognitive | aucun service, dépendance, framework ou stockage nouveau | cohérent |

## Findings et corrections

| ID | Sévérité | Finding | Correction appliquée |
|---|---|---|---|
| A071-01 | HIGH | `renderAgents` remplace les boutons ; une fiche globale pourrait conserver une référence DOM détachée. | T008 et T009 couvrent désormais explicitement le nettoyage au nouveau rendu, au scroll et au redimensionnement. |
| A071-02 | MEDIUM | Le mot « fournisseur » mélangeait éditeur du runtime et fournisseur du modèle, non attesté. | `spec.md`, `plan.md`, le modèle et le contrat parlent désormais d'éditeur du produit ; le fournisseur du modèle reste hors affirmation. |
| A071-03 | MEDIUM | Le tooltip imbriqué proposé initialement serait coupé par `overflow-x: hidden`. | Le plan impose une fiche globale unique positionnée dans la fenêtre. |
| A071-04 | LOW | Une variante monochrome recolorée en CSS pourrait violer les règles de marque. | Le plan et T005 imposent une variante officielle inchangée et une provenance SHA-256. |

## Vérification après auto-fix

- Aucun finding CRITICAL ne reste.
- Aucun marqueur de clarification ne reste.
- Les 12 tâches sont ordonnées et vérifiables.
- Les tests risqués précèdent l'implémentation correspondante.
- Aucun périmètre fonctionnel n'a été réduit pour simplifier l'exécution.
- La checklist de spécification est entièrement fermée.

## Gate d'implémentation

**PASS** : l'implémentation peut commencer à T001.
