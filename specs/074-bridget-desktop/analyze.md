# Analyse de cohérence - SPEC-074 Bridget Desktop

Date: 2026-08-30
Mode: fallback manuel, car le runtime SpecKit officiel et ses scripts `.specify/` ne sont pas installés sur cartae.app.

## Specification Analysis Report

| ID | Catégorie | Sévérité | Emplacement(s) | Résumé | Recommandation |
|---|---|---|---|---|---|
| C1 | Couverture | MEDIUM | `spec.md:108`, `tasks.md:T010` | FR-008 exige la confirmation avant retrait persistant, qui n'était pas explicite dans la tâche de coque. | Appliqué: T010 impose la confirmation observable. |
| C2 | Couverture | MEDIUM | `spec.md:108`, `tasks.md:T022` | FR-008 exige reconnecter un profil; les états étaient testés, pas l'action UI explicite. | Appliqué: T022 couvre l'action de reconnexion. |
| C3 | Couverture | MEDIUM | `spec.md:115`, `spec.md:137`, `tasks.md:T036` | L'accessibilité clavier et le critère d'ajout en moins de trois minutes n'étaient pas des preuves opérateur explicites. | Appliqué: T036 mesure les deux. |
| R1 | Risque maîtrisé | LOW | `plan.md:14`, `research.md` | La double webview Tauri dépend du feature `unstable`. | Conserver l'isolement par labels et la preuve macOS; ne pas traiter cette dette comme un protocole Bridget. |

## Resume de couverture

| Exigence | Tâches | Note |
|---|---|---|
| FR-001 profils SSH et local | T006-T010, T029-T031 | Couverte |
| FR-002 canal SSH Mac -> serveur | T011-T019 | Couverte |
| FR-003 relais serveur loopback | T003, T016, T019 | Couverte |
| FR-004 contenu distant non privilégié | T024-T026 | Couverte |
| FR-005 profils non secrets | T006-T009, T033 | Couverte |
| FR-006 origine persistante | T018, T024-T028 | Couverte |
| FR-007 local sans SSH | T029-T031 | Couverte |
| FR-008 fermer, reconnecter, retirer | T010, T022-T023 | Couverte après correction |
| FR-009 endpoint minimal non journalisé | T001-T003, T015-T016 | Couverte |
| FR-010 échecs catégorisés | T020-T023, T029 | Couverte |
| Critères sécurité, fiabilité, clavier, performance et paquet macOS | T019, T023, T026, T028, T033-T037 | Couverte |

## Alignement constitutionnel

Aucun écart critique observé. Le plan réutilise le relais et le transport SSH existants, isole le code client dans `apps/bridget-desktop`, évite un tunnel générique, et assigne une preuve observable à chaque élément à risque.

## Article XIX/XX - minimalisme et responsabilité future

- `apps/bridget-desktop` est justifié car aucun client n'existe dans le workspace.
- Les modules `profile`, `ssh`, `host_identity`, `connection` et `panels` portent chacun une règle métier, de sûreté ou de cycle de vie et sont testés.
- Le feature Tauri `unstable` est la seule exception de complexité; il est limité aux deux panneaux et documenté dans la recherche et l'ADR.

## Métriques

- Exigences fonctionnelles: 10
- Tâches: 38
- Couverture: 100 % après corrections C1-C3
- Ambiguïtés bloquantes: 0
- Duplications: 0
- Issues critiques: 0
- Issues Article XIX/XX non justifiées: 0

## Issue

Analyse relue après corrections: prête pour l'implémentation.
