# Analyse 093 — 2026-09-06

Analyse manuelle selon speckit-analyze, scripts check-prerequisites absents
constatés par ls. Spec, plan, recherche, contrats, tâches et réemploi lus.

| Exigence | Tâches | Preuve prévue |
|---|---|---|
| FR-001/002 | T003/T004 | Fixture complète/fragmentée |
| FR-003 | T005 | Label attesté/fallback, UUID stable |
| FR-004/005 | T006 | Mix de terminaux et erreurs |
| FR-006 | T003/T008 | ANSI hostile, NO_COLOR/non-TTY |
| FR-007 | T007 | Resize PTY sans frappe |
| FR-008 | T006/T007/T008 | Gates historiques attach/092 |
| FR-009 | T010/T011 | Release installée et recette |

9 exigences, 11 tâches, couverture 100 %, zéro CRITICAL/HIGH non résolu.
Pas de tâche sans exigence, pas d'extension du transport.

Clarifications intégrées avant code : ne pas confondre absence de raisonnement
et erreur ; ne pas masquer terminaux inconnus ; ancien scrollback hors zone gérée ;
pas de promesse de coloration lexicale tous langages. Styles locaux seulement.

XIX/XX : renderer unique conservé, module pur portant sécurité/layout plutôt que
wrapper vide. Nouvelle dépendance justifiée contre parseur artisanal. Noms via
requête existante sans N+1. Coût O(n) par rendu borné, à contrôler au code.
Seconde lecture : aucun conflit FR/SC/plan/tasks identifié. Statut PASS avant code.
