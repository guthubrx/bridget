# Contrat de consultation récente — SPEC146

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/contracts/recent-human-view.md#extrait-1).

## Compatibilité

La voie humaine négocie le singleton `HumanThreadViewRecentV1`, sérialisé `human_thread_view_recent_v1`. Les nouvelles actions sont `list_recent` et `history_recent`. Le singleton145 distinct et les actions145 `list`, `show`, `history` restent inchangés. Une capacité récente absente produit un refus de version explicite. Ne pas charger `list` ou `history` comme repli prétendument récent.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/contracts/recent-human-view.md#extrait-2).

## Liste récente

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/contracts/recent-human-view.md#extrait-3).

Réponse : variante `listed_recent`, `threads` reprenant les résumés145 avec `last_activity_at` requis, et `next_after` curseur ou null. Chaque activité est un entier sûr non négatif.

Le tri de tous les fils autorisés précède la limite. Il suit l'activité DESC puis UUID ASC pour les égalités. L'activité vient du message de dernière séquence, pas du maximum de ses dates, ou de la création pour un fil vide. La position suivante est exactement celle du dernier fil émis.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/contracts/recent-human-view.md#extrait-4).

## Historique récent

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/contracts/recent-human-view.md#extrait-5).

Réponse : variante `history_recent`, `thread_id`, `entries`, `snapshot_seq`, `through_seq`, `has_more`, `next_before_seq`. Aucun `before_seq` de sortie n'est ajouté.

La première page fixe l'instantané à la dernière séquence du fil. Elle contient les vrais messages les plus récents autorisés. `before_seq` est inclusif ; la borne effective vaut `through_seq = min(before_seq, snapshot_seq)`, ou `snapshot_seq` sans position explicite. Chaque page conserve l'instantané et fixe sa prochaine borne à la dernière séquence réellement émise moins1. Les publications ultérieures sont exclues. Seul le rafraîchissement renouvelle la borne.

Les séquences sont strictement descendantes, sans doublon, au plus `through_seq ≤ snapshot_seq`. `has_more` signifie qu'une entrée de séquence plus petite que le minimum réellement émis existe encore dans l'instantané. Il équivaut à `next_before_seq` non null, lequel vaut alors `min(séquences émises) − 1`. Une page vide vaut false/null et une fin à séquence1 vaut false/null sans curseur0. Les séquences actuelles sont contiguës, attribuées atomiquement par thread_post vers1034 ; cela rend `min > 1` équivalent à l'existence, sans remplacer sa garantie. Une page écourtée par la borne existante de48 KiB d'entrées calcule sa suite depuis le dernier message renvoyé. La projection de remplacement utilise `snapshot_seq`, non la borne de page. Les limites invalides et les champs inconnus sont refusés. La projection de sortie garde son budget existant de128 KiB.

## Arguments CLI

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/contracts/recent-human-view.md#extrait-6).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/contracts/recent-human-view.md#extrait-7).

Ces exemples décrivent l'interface attendue, pas une commande de production à exécuter pendant la recette. L'option `--thread` reprend le sélecteur inspect existant confirmé dans cli.rs994/1045/1076.

## Projection et présentation

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/contracts/recent-human-view.md#extrait-8).

Un corps long possède un aperçu visuel d'au plus quatre lignes. L'original entier reste la source de copie et de recherche locale. Les détails techniques regroupent séquence, type français et relations de remplacement. L'état de dépliage n'est pas une mutation de message.

## Refus et non-mutation

Conserver les refus145 pour appartenance, liaison absente ou ambiguë, version, délai et daemon absent. Une révocation ou un changement de contexte efface les données concernées. Une ancienne réponse ne les rétablit pas.

Les lectures, leurs refus et les contrôles de présentation ne provoquent aucun ACK, aucune émission, aucune avance de curseur agent, aucun réveil, aucun appel de modèle ni changement de mission. La voie CLI récente reste froide : aucune initialisation ou autostart pour une preuve de lecture.
