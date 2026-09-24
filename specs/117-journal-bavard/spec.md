# Spécification 117 — Un fil non observable dit pourquoi

## Fiche synthèse
Spec: 117-journal-bavard | Statut: Implemented (livré 2026-09-24) | Priorité: P1 | Date: 2026-09-24
Branche: session-117-journal-bavard | Suite de la 116.

## Problème observé
Depuis l'épisode de disque plein de 21:01 le 24/09, le fil `opus-city-glmwrk1` n'annonce plus aucun
événement d'observation : son coordinateur ne peut plus suivre ses fins de tour. Une relance du pont
ne l'a pas rétabli. Journal intact, structure et volume normaux : aucune cause visible, parce que la
projection du journal s'interrompt par plusieurs sorties muettes (rattrapage en retard sur le
fichier, trois écritures refusées, journal en échec).

## Exigences
- **FR-001** : chaque interruption persistante de la projection du journal est journalisée avec son
  motif et son détail, une fois, après 30 secondes.
- **FR-002** : le rétablissement est annoncé s'il suit un blocage signalé.
- **FR-003** : un blocage bref, normal pendant une écriture, ne produit aucun message.

## Critères de succès
- **SC-001** : la cause réelle du blocage de `opus-city-glmwrk1` apparaît dans le journal du pont.
