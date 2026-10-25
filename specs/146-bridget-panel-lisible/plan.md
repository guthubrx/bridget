# Plan SPEC146 — Panneau Bridget plus lisible

Date : 2026-10-08. Statut : conception réalisée, validation et clôture en cours ; aucune installation146. Estimation validée : 45–75 minutes.

## Objectif

Présenter les fils selon leur activité récente et leurs messages récents en premier. Rendre la lecture plus compacte, sans toucher au travail des agents. Conserver le socle145 et prouver les nouvelles garanties sur des données isolées.

## Contexte et périmètre

Le socle145 est installé mais non committé. Deux worktrees146 reçoivent une copie identifiée des seuls fichiers145 utiles. Un manifeste d'import permet de distinguer ce socle des changements146. Les racines principales,145, la base réelle et les processus actifs restent intacts.

Les nouvelles opérations humaines `list_recent` et `history_recent` sont distinctes de `list`, `show` et `history`. Les méthodes145 et les méthodes agents gardent leur sémantique. Une capacité négociée `HumanThreadViewRecentV1` empêche d'accepter une réponse ancienne comme une réponse146. L'absence de cette capacité produit une incompatibilité explicite ; aucun repli vers une première page ancienne n'est admis.

## Architecture prévue

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/plan.md#extrait-1).

Une petite extraction de composant de message est permise si elle réunit ce comportement et améliore sa lisibilité. Aucun framework, dépendance, réglage global ou sélecteur d'ordre nouveau n'est requis.

## Garanties de pagination

La liste est globalement triée à chaque lecture. Elle est stable sans trou ni doublon lorsque les données ne changent pas. Elle n'est pas un instantané complet : une publication concurrente peut déplacer un fil. La vue ne l'affiche pas deux fois ; le rafraîchissement repart de la première page.

L'historique est un instantané borné. Les messages sont ordonnés par séquence décroissante. La position humaine ne modifie pas un curseur agent. La séquence du dernier message émis sert à calculer la suite, y compris lorsque la borne de taille limite une page avant sa borne de nombre. La projection des corrections emploie toujours `snapshot_seq`, jamais la borne inférieure de page : une correction visible dans l'instantané doit rester visible sur une page plus ancienne.

## Réutilisation et exclusions

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/plan.md#extrait-2).

Aucune modification des boucles agent, ACK, missions ou notifications. Aucun compositeur. Aucun polling. Aucun accès à une base de conversation active pendant les tests. Aucun commit, fusion, push, installation ou redémarrage autorisé.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/plan.md#extrait-3).

## Contrôles prévus

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/plan.md#extrait-4).

## Gates

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/plan.md#extrait-5).

Avant clôture : preuves fonctionnelles, non-régressions, recette isolée et revue du diff. Aucun verdict inter-fournisseurs n'est revendiqué si l'annuaire du même projet ne fournit aucun autre fournisseur joignable. Ne pas solliciter d'autres projets pour remplir artificiellement cette exigence.

Statut final : Implemented, 10/10 tâches. Cette clôture n'autorise aucune opération Git ni installation ou redémarrage.
