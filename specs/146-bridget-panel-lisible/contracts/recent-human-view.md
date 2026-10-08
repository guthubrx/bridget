# Contrat de consultation récente — SPEC146

Date : 2026-10-08. Statut : contrat réalisé, interopRust/T3 vérifiée ; clôture en cours, aucune installation146.

## Compatibilité

La voie humaine négocie le singleton `HumanThreadViewRecentV1`, sérialisé `human_thread_view_recent_v1`. Les nouvelles actions sont `list_recent` et `history_recent`. Le singleton145 distinct et les actions145 `list`, `show`, `history` restent inchangés. Une capacité récente absente produit un refus de version explicite. Ne pas charger `list` ou `history` comme repli prétendument récent.

T3 emploie sa RPC `bridget.read`, ses schémas fermés et son lecteur CLI existants. Le serveur résout conversation et projet ; le navigateur ne fournit pas une identité libre autorisante. La commande est exécutée par arguments séparés, jamais par shell.

## Liste récente

Entrée : contexte T3 attesté, `list_recent`, `limit` facultatif de1 à100 et `after` facultatif sur128 caractères au plus.

Réponse : variante `listed_recent`, `threads` reprenant les résumés145 avec `last_activity_at` requis, et `next_after` curseur ou null. Chaque activité est un entier sûr non négatif.

Le tri de tous les fils autorisés précède la limite. Il suit l'activité DESC puis UUID ASC pour les égalités. L'activité vient du message de dernière séquence, pas du maximum de ses dates, ou de la création pour un fil vide. La position suivante est exactement celle du dernier fil émis.

Le curseur canonique vaut `<last_activity_at>:<uuidlowercase>`. Le timestamp est décimal de0 à MAX_SAFE, sans zéro initial sauf `0`. L'UUID est canonique en minuscules. T3 garde ce curseur opaque. Les schémas refusent doublons, ordre incohérent ou curseur qui ne désigne pas le dernier résumé reçu. Le curseur ne crée aucune autorisation. Sans changement de données, les pages sont sans trou ni doublon. Une publication concurrente peut déplacer un fil : T3 déduplique les fils chargés et le rafraîchissement repart du début. Aucun instantané complet de liste n'est annoncé.

## Historique récent

Entrée : contexte T3 attesté, `history_recent`, identifiant de fil, `limit` facultatif de1 à200, `before_seq` et `to_seq` facultatifs, entiers sûrs non négatifs. Les champs T3 correspondants sont `sharedThreadId`, `beforeSeq` et `toSeq` ; le lecteur traduit les arguments CLI séparément.

Réponse : variante `history_recent`, `thread_id`, `entries`, `snapshot_seq`, `through_seq`, `has_more`, `next_before_seq`. Aucun `before_seq` de sortie n'est ajouté.

La première page fixe l'instantané à la dernière séquence du fil. Elle contient les vrais messages les plus récents autorisés. `before_seq` est inclusif ; la borne effective vaut `through_seq = min(before_seq, snapshot_seq)`, ou `snapshot_seq` sans position explicite. Chaque page conserve l'instantané et fixe sa prochaine borne à la dernière séquence réellement émise moins1. Les publications ultérieures sont exclues. Seul le rafraîchissement renouvelle la borne.

Les séquences sont strictement descendantes, sans doublon, au plus `through_seq ≤ snapshot_seq`. `has_more` signifie qu'une entrée de séquence plus petite que le minimum réellement émis existe encore dans l'instantané. Il équivaut à `next_before_seq` non null, lequel vaut alors `min(séquences émises) − 1`. Une page vide vaut false/null et une fin à séquence1 vaut false/null sans curseur0. Les séquences actuelles sont contiguës, attribuées atomiquement par thread_post vers1034 ; cela rend `min > 1` équivalent à l'existence, sans remplacer sa garantie. Une page écourtée par la borne existante de48 KiB d'entrées calcule sa suite depuis le dernier message renvoyé. La projection de remplacement utilise `snapshot_seq`, non la borne de page. Les limites invalides et les champs inconnus sont refusés. La projection de sortie garde son budget existant de128 KiB.

## Arguments CLI

`thread inspect --t3-thread CONVERSATION --project-root RACINE --action list_recent --after CURSEUR --limit N --json`.

`thread inspect --t3-thread CONVERSATION --project-root RACINE --action history_recent --thread FIL --before-seq S --to-seq SNAPSHOT --limit N --json`.

Ces exemples décrivent l'interface attendue, pas une commande de production à exécuter pendant la recette. L'option `--thread` reprend le sélecteur inspect existant confirmé dans cli.rs994/1045/1076.

## Projection et présentation

Le contrat rend seulement les données utiles déjà autorisées. Il n'expose ni secrets, transport, credentials ni curseur agent. Les corps restent exacts, sans résumé ni transformation. T3 ne traite pas ce texte comme HTML exécutable.

Un corps long possède un aperçu visuel d'au plus quatre lignes. L'original entier reste la source de copie et de recherche locale. Les détails techniques regroupent séquence, type français et relations de remplacement. L'état de dépliage n'est pas une mutation de message.

## Refus et non-mutation

Conserver les refus145 pour appartenance, liaison absente ou ambiguë, version, délai et daemon absent. Une révocation ou un changement de contexte efface les données concernées. Une ancienne réponse ne les rétablit pas.

Les lectures, leurs refus et les contrôles de présentation ne provoquent aucun ACK, aucune émission, aucune avance de curseur agent, aucun réveil, aucun appel de modèle ni changement de mission. La voie CLI récente reste froide : aucune initialisation ou autostart pour une preuve de lecture.
