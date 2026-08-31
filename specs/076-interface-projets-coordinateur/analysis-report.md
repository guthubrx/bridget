# Analyse inter-artefacts - SPEC-076

**Date**: 2026-08-30
**Méthode**: analyse manuelle documentée.

## Contexte d'exécution

La primitive SpecKit Analyze n'est pas disponible dans ce dépôt car le dossier
.specify est volontairement absent. La synchronisation qui le créerait a été
explicitement interdite par l'utilisateur. L'analyse compare donc directement
spec.md, plan.md, tasks.md, les artefacts de conception et le code de la tête
de référence.

## Passage 1 - Findings et corrections appliquées

| ID | Sévérité | Constat vérifié | Correction appliquée |
|---|---|---|---|
| A-01 | HIGH | FR-036 limitait les mutations à une interface locale authentifiée sans préciser le lien avec Bridget Desktop ni l'interdiction d'écoute réseau. | FR-037, plan et contrat imposent loopback serveur et tunnel SSH attesté par SPEC-074 uniquement. |
| A-02 | HIGH | La réactivation est demandée par US3 et US5, mais ProjectIdentity::activate refuse actuellement l'état Disabled et Bridget refuse une liaison désactivée. | FR-038, modèle, contrat, quickstart, T030 et T031 imposent une mutation typée, idempotente, auditée et sans seconde identité. |
| A-03 | MEDIUM | Maicie exige display_name, alors que la SPEC refuse un nom projet concurrent. | FR-039 et T021 imposent la dérivation unique depuis le nom de dossier, sans champ éditable ni alias UI. |
| A-04 | MEDIUM | FR-035 exclut les approbations SPEC-067, sans oracle de test nommé. | T013 inclut le refus ou l'absence démontrée de ces opérations. |

## Passage 2 - Résultat après corrections

Aucun conflit restant entre le périmètre fonctionnel, les frontières techniques
et les tâches. Les concepts nouveaux sont justifiés par une absence vérifiée
dans la tête de référence :

- projection UI de projets : extension de UiSnapshotV1, pas de second store;
- intentions UI Maicie : extension de la saga ProjectRegistry, sans accès direct
  au store Maicie;
- réactivation : extension explicite des transitions SPEC-065, pas une
  réinterprétation silencieuse de disable;
- configuration coordinateur : projection des autorités SPEC-066, SPEC-067,
  SPEC-072 et SPEC-075, sans les dupliquer.

## Couverture

| Groupe d'exigences | Tâches de preuve ou d'implémentation |
|---|---|
| FR-001 à FR-007 - dossier et racines | T010 à T015, T031 |
| FR-008 à FR-015 - création, import, reconnexion | T020 à T025, T030 à T035 |
| FR-016 à FR-019 - configuration attestée | T014, T015, T040, T041 |
| FR-020 à FR-028 - coordinateur et découverte | T040 à T046 |
| FR-029 à FR-034 - navigation et retrait | T050 à T053 |
| FR-035 - absence d'approbation | T013 |
| FR-036 à FR-037 - surface UI locale bornée | T003, T013, T051, T052 |
| FR-038 - réactivation idempotente | T030, T031, T033, T053 |
| FR-039 - nom métier issu du dossier | T020, T021, T024 |

## Métriques

- Exigences fonctionnelles : 39
- Tâches : 42
- Exigences couvertes : 39/39 - 100 %
- Findings CRITICAL restants : 0
- Ambiguïtés matérielles restantes : 0
- Tests exécutés : 0 - aucune implémentation n'est autorisée tant que les gates
  de dépendances ne sont pas levées.

## Verdict

**PASS_DOCUMENTATION_ONLY**. Les artefacts sont cohérents pour une future
implémentation. Ce verdict ne vaut ni autorisation de coder ni preuve runtime.
La première tâche non cochée reste T001.
