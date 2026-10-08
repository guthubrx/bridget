# Modèle de lecture — SPEC146

Date : 2026-10-08. Statut : modèle réalisé et vérifié sur sources146 ; clôture en cours, aucune installation146.

## Fil récent

Un fil récent conserve l'identifiant, le titre, les membres et les champs de lecture145. Il ajoute une date d'activité vérifiable : date du dernier message, ou date de création si aucun message n'existe.

L'activité vient du message de dernière séquence, pas du maximum des horodatages. L'ordre est activité DESC puis UUID ASC. `after` et `next_after` encodent la dernière position émise sous la forme canonique `<last_activity_at>:<uuidlowercase>`, sur 128 caractères au plus. Le timestamp est un entier décimal de 0 à MAX_SAFE sans zéro initial sauf `0`. La vue traite ce curseur comme opaque. La date courte correspond à cette activité ; le cas vide est présenté comme création.

La réponse `listed_recent` comprend `threads`, dont chaque résumé145 ajoute `last_activity_at` requis, et `next_after` curseur ou null. La limite est de100 fils au plus. Les dates sont des entiers sûrs non négatifs. Aucun curseur agent, credential ou enveloppe de transport n'est exposé.

## Page d'historique récent

Un message conserve sa séquence stable, auteur attesté, date, type, corps original et les relations de remplacement disponibles.

La requête emploie `before_seq` et `to_seq` facultatifs, entiers sûrs non négatifs. `before_seq` est inclusif. `to_seq` fixe l'instantané ; sans lui, la dernière séquence du fil le fixe à l'ouverture. La réponse `history_recent` comprend `thread_id`, `entries`, `snapshot_seq`, `through_seq`, `has_more` et `next_before_seq`. La borne de page vaut `through_seq = min(before_seq, snapshot_seq)` lorsque `before_seq` existe, sinon `snapshot_seq`. L'ordre est strictement décroissant sans doublon. Chaque séquence est au plus `through_seq`, lui-même au plus `snapshot_seq`. La limite est de200 messages au plus.

`has_more` signifie qu'au moins une entrée de séquence inférieure à la plus petite séquence réellement émise reste dans l'instantané. Le curseur suivant vaut cette plus petite séquence moins1 seulement dans ce cas, sinon null. Une page vide n'a pas de suite ; la fin à séquence1 donne false/null, jamais un curseur0. Le stockage actuel attribue des séquences contiguës atomiquement ; `min > 1` y traduit cette existence, mais le contrat ne promet pas une page suivante vide si des trous apparaissent. Ce calcul conserve les limites de taille sans sauter de messages. Le résultat ne possède pas un champ de sortie `before_seq`.

La projection d'un remplacement conserve la borne globale `snapshot_seq`, même sur une page de borne `through_seq` plus basse. Une correction publiée au-delà de cette dernière borne mais dans l'instantané reste visible. Une correction au-delà de l'instantané reste exclue.

## État de présentation

L'état de contexte garde la conversation T3, le projet et les protections145 contre les réponses devenues obsolètes. La sélection du fil reste distincte de celle de la conversation.

Le corps original sert de source unique à l'affichage, la recherche et la copie. L'état « développé » d'un message et l'état « détails ouverts » sont des choix visuels ; ils ne changent pas le corps, son type ou son verdict. Le changement de contexte ne réutilise pas ces choix pour une autre identité.

La liste fusionne par UUID lorsque l'activité évolue entre deux pages. La nouvelle observation remplace le résumé précédent, y compris sa date. L'ensemble chargé est ensuite retrié par activité DESC puis UUID ASC. Cette déduplication ne prétend pas recréer une liste figée. Le rafraîchissement remet les positions humaines au début.

## Invariants

1. L'appartenance et la liaison attestée autorisent chaque lecture.
2. Une absence de capacité récente ne devient jamais un ordre ancien présenté comme récent.
3. Les opérations145 et les opérations agents gardent leur sémantique.
4. Les pages d'un historique figé n'ont ni trou ni doublon.
5. Les nouvelles publications ne changent pas cet historique avant rafraîchissement.
6. La liste est globalement triée à chaque page ; elle ne promet pas un instantané complet concurrent.
7. Corps développé et copie correspondent exactement à l'original.
8. Aucune ouverture, copie, pagination ou recherche ne modifie l'état métier des agents.
9. Les réponses obsolètes ou non autorisées ne restaurent aucune ancienne donnée.
