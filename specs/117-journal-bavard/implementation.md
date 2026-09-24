# Journal 117 — Un fil non observable dit pourquoi

- **Base** : main `f72c82c1` — **Date** : 2026-09-24 — **Statut** : Implemented, livré 21:49

## Incident
`opus-city-glmwrk1` (3b09f990) sans aucun événement d'observation depuis 21:01 (disque plein : quatre
échecs d'enregistrement de l'état du fil). Messages toujours remis. Coordinateur `sol_city_ai` sans
suivi de ses fins de tour.

## Diagnostic (données réelles)
Écartés : ligne tronquée dans le journal (aucune), taille de lignes (8 238 octets, comparable à un fil
sain), structure et continuité des séquences (identiques), volume d'activités (390 contre 416 pour un
fil sain). Relance du pont de 21:37 : lien neuf, toujours aucune capacité à +85 s.

Lecture du code : un fil n'est déclaré observable qu'une fois sa projection de journal terminée ; cinq
sorties l'interrompaient sans trace (rattrapage en retard sur le fichier, trois écritures refusées,
journal en échec).

## Correction
`note_journal_block` signale une interruption qui persiste plus de `JOURNAL_BLOCK_GRACE` (30 s), une
fois par motif, avec son détail ; `clear_journal_block` annonce le rétablissement s'il avait été
signalé. Pas de message pour un blocage bref, normal pendant une écriture.

## Livraison
Relance du pont à 21:49 : `opus-city-glmwrk1` annonce de nouveau `turn_ended` et
`permission_required`, 24 sources sur 24 ; aucun blocage signalé. `sol_city_ai` prévenu (contournement
puis rétablissement).

## Vérifications
Test `spec117_*` ; `t3code` 83/83 ; clippy OK. Recette complète : 1544 réussis, 3 échecs dans
`search_104_test` sous une charge de 72, 23/23 relancé seul deux fois. 0 fixture restante.
