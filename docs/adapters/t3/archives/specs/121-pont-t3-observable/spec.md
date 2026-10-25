# Archives techniques de l’adaptateur T3

Document d’origine : [specs/121-pont-t3-observable/spec.md](../../../../../../specs/121-pont-t3-observable/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

## Fiche synthèse
Spec: 121-pont-t3-observable | Statut: Implemented | Priorité: P1 | Date: 2026-09-25
Branche: session-121-pont-t3-observable | Points 1, 3 et 6 du plan d'amélioration.

<a id="extrait-2"></a>

## Extrait 2

## Problèmes observés
1. Les workers Claude de `sol_city_ai` (glmwrk1, glmwrk2, glmkrkr4) déclaraient `events: []` : le
   dernier tour, un réveil d'arrière-plan sans message déclencheur, n'avait pas d'origine prouvée ;
   tout le fil devenait inobservable et le coordinateur devait réclamer des bilans.
3. « journal du fil … : journal ACP saturé » répété 94 fois : ce chemin écrivait à chaque lecture
   au lieu du signalement unique de la 117.
6. Le 25/09, deux réponses (GLM3, GLM4) sont restées dans leur fil : tours faussés par une seconde
   application T3, appariement impossible, rien n'était transmis.
