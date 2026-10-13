# Analyze SPEC-161 — analyse et auto-correction

**Date :** 2026-10-13. **Périmètre :** spec, plan, tasks, data-model, contrat, quickstart, checklist et reuse-audit161. **Issue :** aucun défaut bloquant documentaire restant après une correction ciblée puis une relecture. Cette analyse ne valide aucun code, build ou déploiement.

## Primitive et méthode

Skill speckit-analyze lue, après génération Tasks. Tentative réelle de .specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks : RC127, fichier absent. Fallback manuel prescrit par my-specify-all : comparaison des artefacts, inventaire des exigences/tâches, corrections non ambiguës autorisées par le pipeline, seconde lecture. Aucun toolkit installé. L'analyse seule est en lecture ; les corrections qui suivent relèvent de l'auto-fix explicitement invoqué dans my-specify-all.

## Constats, corrections et seconde passe

| ID | Catégorie | Constat initial | Correction ciblée | Seconde passe |
|---|---|---|---|---|
| A1 | Cohérence | FR009 interdisait toute modificationDB alors que roundtrip095 inscrit agents/messages | Aucune éditiondirecte/migration, écrituresAPI synthétiques autorisées et identifiées ; clientstatus/who distinct du transportPython | Résolu dans spec/plan/contrat/quickstart/tasks |
| A2 | Couverture | Source exacte non imposée par l'outil existant | T002/T003 --expect-commit optionnelle, obligatoire161, HEAD/propreté/index avantSSH ; legacy conservé | Exigence couverte, implémentation à réaliser |
| A3 | Sous-spécification | Manifest n'explicitait pas ensemble exact et modesGit | T004/T005 listefiltrée unique/SHAattendu/extra/no-symlink ; noms et exclusions précisés | Couvert sans nouveau composant |
| A4 | Vérifiabilité | Bascule/rollback décrits mais commande non figée | Extraitcontractuel os.replace et T008/T009 fixture qui l'exécute exactement ; limiteTOCTOU | Couvert, aucune preuve de test prématurée |
| A5 | Ordre | Tout cocher avantcommit pourrait conduire à attendre un déploiement qui exigecommit |14tâches développement/préparation avantcommit, gatesG8/G9 obligatoires séparés et mapping14FR/6SC ; aucun succèsruntime prématuré | Ordre cohérent, aucune exigence retranchée |
| A6 | Paramètres | target-dir, nice réel, reprisepréfixe et nettoyage distant imprécis | T006/T007 et contrat, suffixer2, build.py distant avec ses gardes | Couvert ; preuve réelle encore requise |
| A7 | Décision opérationnelle | CacheCartae incomplet et approvisionnement initialement à résoudre | DécisionROOT bornée : archivesMac SHA==lock, copieadditive, indexbackup/CAS/refusdivergence, aucun téléchargementtiers | Résolu pour cette copie seulement, succèsbuild non présumé |

## Couverture

| Exigence | Tâches | Gate réel |
|---|---|---|
| FR001 | T002/T003/T005/T011 | G8b |
| FR002 | T002/T003/T004/T005 | G8b |
| FR003 | T001/T006/T007 | G8b |
| FR004 | T006/T007 | G8b |
| FR005 | T004/T005/T006/T007 | G8b |
| FR006 | T010/T011 | G8b/G8c |
| FR007 | T008/T009/T011 | G8c |
| FR008 | T008/T009/T010/T011 | G8c |
| FR009 | T001/T010 | G8c |
| FR010 | T010 | G8c |
| FR011 | T012/T014 | G8a/G8d |
| FR012 | T001/T011/T012/T014 | G8d |
| FR013 | T001/T012/T014 | G9 |
| FR014 | T001/T002/T003/T004/T005/T009/T014 | G8b/G8c |
| SC001 | T002–T007 | G8b |
| SC002 | T010 | G8b/G8c |
| SC003 | T001/T010 | G8c |
| SC004 | T006/T008/T009/T010 | G8c |
| SC005 | T012/T014 | G8a/G8d/G9 |
| SC006 | T013 | Résultats finaux ciblés |

## Constitution et minimalisme

Pas de violation non justifiée repérée. Worktree, identitéGit et processus protégés conservés. L'optionstrict est justifiée parFR001/002 tout en préservant legacy. Aucun service, helper permanent, dépendance ou table ajouté. Les vérifications répètent seulement les frontières nécessaires ; pas de FULL Rust. Le mainteneur peut lire la commande de bascule, son test, le manifest et les reçus sans contexte caché. Potentiel de suppression documentaire : redondances légères de rappels, sans code supplémentaire ; pas de nouvelle automatisation proposée.

## Mesures

14exigencesFR +6critèresSC,14tâches,100% de couverture prévue,0tâche sans rattachement,0case cochée. Vérification mécanique des IDsT001–T014 ordonnés et uniques, aucun placeholder. Aucun finding bloquant documentaire restant ; implémentation, Converge, audit et gates8/9 non exécutés. Le mapping des SC à un gate n'est pas une preuve qu'ils sont atteints.

Prochaine action : ROOT relit les artefacts corrigés puis exécute T001. RecalibrageTasks :120–240minutes,14tâches, hypothèsescache/build/revue. Aucun succès de livraison annoncé.
