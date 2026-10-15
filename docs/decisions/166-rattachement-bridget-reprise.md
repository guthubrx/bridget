# ADR-166 — Remonter la preuve Bridget avant un tour repris

## Statut

Accepté le 15 octobre 2026.

## Contexte

Une session privée Bridget peut tourner pendant qu’un fil Codex natif reste chargé. Le code détectait la différence au début du tour. Il retirait les droits publiés, mais il ne rechargeait pas la nouvelle preuve dans le fil natif.

## Décision

Avant chaque tour, l’adaptateur compare la preuve privée courante avec la preuve montée. En cas de différence, il reprend le même fil natif avec sa configuration courante. Il attend cette reprise avant de publier les droits et de démarrer le tour.

## Conséquences

- Un fil ancien retrouve Bridget lors de son premier nouveau tour.
- Une preuve stable n’ajoute aucun appel natif.
- Un échec de remontage bloque le tour de façon sûre.
- L’historique et l’identité du fil restent inchangés.
- Aucun service, stockage ou mécanisme de secours n’est ajouté.
