# Spécification 121 - Pont adaptateur de bureau : fils Claude observables, journal sobre, réponses incertaines transmises

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/121-pont-t3-observable/spec.md#extrait-1).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/121-pont-t3-observable/spec.md#extrait-2).

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
