# Spécification 121 - Pont T3 : fils Claude observables, journal sobre, réponses incertaines transmises

## Fiche synthèse
Spec: 121-pont-t3-observable | Statut: Implemented | Priorité: P1 | Date: 2026-09-25
Branche: session-121-pont-t3-observable | Points 1, 3 et 6 du plan d'amélioration.

## Problèmes observés
1. Les workers Claude de `sol_city_ai` (glmwrk1, glmwrk2, glmkrkr4) déclaraient `events: []` : le
   dernier tour, un réveil d'arrière-plan sans message déclencheur, n'avait pas d'origine prouvée ;
   tout le fil devenait inobservable et le coordinateur devait réclamer des bilans.
3. « journal du fil … : journal ACP saturé » répété 94 fois : ce chemin écrivait à chaque lecture
   au lieu du signalement unique de la 117.
6. Le 25/09, deux réponses (GLM3, GLM4) sont restées dans leur fil : tours faussés par une seconde
   application T3, appariement impossible, rien n'était transmis.

## Exigences
- **FR-001** : un tour dont la page lue couvre la demande sans message utilisateur à son
  horodatage est spontané ; il garde le fil observable et sa fin est un fait observable. Un tour
  provoqué par une notification reste exclu.
- **FR-002** : un refus d'écriture répété dans le journal est signalé une fois après le délai de la
  117, comme les autres blocages.
- **FR-003** : une demande restée sans appariement certain, fil au repos, reçoit le texte écrit
  après elle jusqu'au message utilisateur suivant, préfixé d'un avertissement d'incertitude ;
  rien s'il n'y a pas de texte.

## Critères de succès
- **SC-001** : un fil Claude avec réveil d'arrière-plan déclare TurnEnded et journalise la fin avec
  `stop_reason` ; le test échoue sans la détection.
- **SC-002** : la réponse incertaine s'arrête au message utilisateur suivant.
- **SC-003** : recette complète verte hors instabilité connue de `search_104_test`.
