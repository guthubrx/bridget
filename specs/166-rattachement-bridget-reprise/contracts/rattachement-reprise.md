# Contrat interne — Rattachement Bridget à la reprise

## Précondition

Un tour Codex va démarrer pour un fil T3 qui possède une preuve privée Bridget courante.

## Comportement

- Si la preuve courante est déjà montée, démarrer le tour sans reprise supplémentaire.
- Si elle n’est pas montée, reprendre le même fil natif avec la configuration courante.
- Attendre la fin de la reprise avant de publier les droits et de démarrer le tour.
- Si la reprise échoue, ne pas démarrer le tour.

## Garanties

- Le `ThreadId` T3 ne change pas.
- L’identifiant natif Codex ne change pas.
- L’historique n’est ni remplacé ni dupliqué.
- Une preuve n’est jamais partagée entre deux fils.
- Une preuve ancienne ne reçoit pas les droits du nouveau run.
- Aucun repli global n’est autorisé.
