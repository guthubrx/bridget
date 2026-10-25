# Modèle de données — SPEC145

Date : 2026-10-07. Contrat de conception ; aucune migration.

## Données existantes réutilisées

Les fils, membres et entrées restent dans les tables Bridget existantes. `thread_list` filtre par membre. `thread_show` et `thread_history` appellent la règle `load_thread_for_member`. Les reçus, ACK et réveils restent des données agent ; ils ne sont ni créés ni déplacés par le lecteur humain.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/data-model.md#extrait-1).

## Fait adaptateur de bureau de liaison en mémoire

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/data-model.md#extrait-2).

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/data-model.md#extrait-3).

## Projection humaine

Enveloppe version1, sujet `{agent_id,name:null|string}|null` et résultat borné sous result. Un refus sans sujet attesté peut avoir subject null. Le nom vient de `Store::agent_display_name` ; un nom indisponible reste null, avec ID disponible. Aucun nom n'est généré.

Fil : identifiant, titre source, créateur attesté, état ouvert/fermé et métadonnées utiles. Membres : ID et nom attesté lorsqu'il existe. Message : identifiant/séquence stable, auteur ID et nom disponible, date, type réel et corps source. Les autres champs ne doivent pas être exposés par simple propagation de JSON brut.

Dates source : created_at:i64 et closed_at:Option<i64>, secondes depuis l'époque Unix. Ne pas les déclarer en chaînes ISO. Notification d'entrée historique : objet fermé mode none/targets/all et targets UUID[] borné à16. La sortie historique diffère de ThreadNotify en entrée Post, all|UUID[]. Réutiliser cette sortie stockée ; la consultation ne crée aucune notification.

Pagination liste : `next_after` indique l'UUID exclusif suivant. Pagination historique : `from_seq`, `through_seq`, `snapshot_seq`, `has_more`, `next_from_seq` décrivent la page. `snapshot_seq` est une borne de lecture, pas un curseur d'ACK. Les pages suivantes réutilisent la borne sous `to_seq`.

## État du panneau

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/145-bridget-thread-viewer/data-model.md#extrait-4).

États : fermé ; ouvert inactif ; chargement liste ; prêt vide ; prêt avec fils ; chargement fil ; prêt historique ; chargement page ; rafraîchissement ; recherche sans résultat ; erreur récupérable ; incompatible ; identité indisponible ; interdit.

À changement de contexte, fermeture ou refus, invalider les générations et retirer les données qui ne sont plus autorisées. Une ancienne réponse ne peut pas restaurer un sujet quitté. La recherche locale ne change aucune donnée Bridget.

## Invariants

- Aucune transition UI ne compose de message ni de commande agent.
- Aucune lecture humaine ne crée de reçu, ACK, réveil ou mission.
- Chaque page est autorisée selon la liaison actuelle et l'appartenance.
- Les limites de page et de sortie sont vérifiées avant publication.
- Les corps et leurs copies restent identiques à la chaîne source.
- Les noms, auteurs, dates et types affichés proviennent des données attestées.
