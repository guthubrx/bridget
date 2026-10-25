# SPEC146 — Panneau Bridget plus lisible

Date : 2026-10-08. Statut : Implemented ; 10/10 tâches, contrôles et audits ciblés PASS. Branche : `session-146-bridget-panel-lisible`.

## Besoin et périmètre

Le panneau Bridget fonctionne. Son utilisateur trouve sa présentation trop dense. Il veut retrouver d'abord les conversations récemment actives et les derniers messages. Il veut lire les échanges sans subir tous leurs détails techniques.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/spec.md#extrait-1).

La session145 est installée. Ses sources ne sont pas encore committées. La session146 part de ces sources validées dans des espaces isolés. Un manifeste d'import doit identifier ce socle ; la présence du logiciel installé ne remplace pas cette preuve.

Le développement et la recette utilisent des données de test isolées. Aucun accès à la base des conversations actives n'est nécessaire pour la recette visuelle. Le « go » ne vaut pas autorisation de commit, fusion, push, installation, déploiement ou redémarrage.

## Histoires utilisateur

### US1 — Retrouver les échanges récents (P1)

Comme utilisateur, je veux voir les conversations récemment actives en premier et ouvrir directement leurs derniers messages.

Test indépendant : préparer plus d'une page de fils et plus de deux pages de messages. Placer le fil créé le plus tôt en tête de l'activité récente. Vérifier l'ordre dès la première page, puis parcourir les pages suivantes.

1. Étant donné des fils créés à des dates différentes, quand un ancien fil reçoit le dernier échange, alors il apparaît avant les fils moins récemment actifs, même s'ils seraient sur une autre page selon l'ancien ordre.
2. Étant donné un fil sans message, quand la liste est triée, alors sa date de création sert de date d'activité et apparaît comme telle.
3. Étant donné un fil contenant plusieurs pages de messages, quand je l'ouvre, alors sa première page contient les messages réellement les plus récents, du plus récent au plus ancien.
4. Étant donné une première page chargée, quand je demande les messages plus anciens, alors ils s'ajoutent après les messages récents, sans trou ni doublon dans l'instantané consulté.
5. Étant donné de nouveaux messages publiés après cet instantané, quand je poursuis ses pages, alors ces nouveaux messages ne déplacent pas son historique ; le rafraîchissement manuel permet de consulter le nouvel état.

### US2 — Lire un échange sans encombrement (P1)

Comme utilisateur, je veux distinguer rapidement l'auteur, le texte et les détails secondaires de chaque échange.

Test indépendant : ouvrir un fil avec plusieurs auteurs et des corps courts, longs, multilignes et Unicode. Déplier un long message et ses détails. Copier son corps et comparer les caractères à la source originale.

1. Étant donné la liste des fils, quand elle apparaît, alors chaque entrée présente un titre, des participants et une date courte, avec une sélection clairement visible.
2. Étant donné les messages d'un fil, quand ils apparaissent, alors chaque auteur reste visible et une séparation discrète distingue les messages.
3. Étant donné un corps de plus de quatre lignes visuelles, quand il apparaît, alors un aperçu de quatre lignes au plus et le bouton « Déplier » remplacent son affichage intégral par défaut.
4. Étant donné un message déplié, quand je le lis, alors son corps original complet est accessible ; « Replier » retrouve l'aperçu. Les lignes, espaces et caractères ne sont pas réécrits.
5. Étant donné un message replié ou déplié, quand je le copie, alors la copie contient toujours son corps original entier, jamais seulement l'aperçu.
6. Étant donné un message, quand j'ouvre ses détails techniques, alors je retrouve sa séquence, son type et ses informations de remplacement disponibles. Ils restent discrets par défaut.
7. Étant donné un mot situé dans une portion repliée mais chargée, quand je le recherche, alors le message reste trouvable sans appel distant.

### US3 — Conserver une lecture sûre et native (P1)

Comme utilisateur, je veux cette présentation améliorée sans changer les agents, leurs messages ou leurs missions.

Test indépendant : comparer les données métier avant et après ouverture, recherche, copie, dépliage, pagination et rafraîchissement. Changer rapidement de conversation et révoquer une appartenance. Tester les contrôles au clavier et dans un panneau étroit.

1. Étant donné la consultation humaine, quand j'utilise ses contrôles, alors aucun message, ACK, réveil, appel de modèle ou changement de mission n'est provoqué.
2. Étant donné une réponse devenue obsolète, une fermeture ou une révocation, quand la lecture se termine, alors aucune donnée du contexte quitté ou refusé ne réapparaît.
3. Étant donné le daemon absent, une liaison absente ou une version incompatible, quand je consulte le panneau, alors un état explicite et une possibilité de réessayer remplacent les données.
4. Étant donné le clavier ou un panneau étroit, quand je consulte les fils, alors sélection, dépliage, copie, détails, pagination et rafraîchissement restent accessibles sans chevauchement de contenu.

## Exigences fonctionnelles

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/spec.md#extrait-2).

## Critères de succès

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/spec.md#extrait-3).

## Hypothèses et exclusions

« Récent d'abord » s'applique par défaut aux fils et à leurs messages. Un fil plus ancien peut donc monter en tête après un nouvel échange. Aucun sélecteur d'ordre supplémentaire n'est requis.

L'ordre global de la liste est calculé à chaque page. Les pages restent sans trou ni doublon sur des données stables. Une publication concurrente peut déplacer un fil entre deux pages : la vue élimine les doublons éventuels et le rafraîchissement manuel repart du début. Aucun instantané complet de la liste n'est promis. L'historique d'un fil conserve, lui, son instantané paginé.

L'aperçu visuel n'altère jamais la source. Les corps ne sont ni résumés, ni reformulés, ni exécutés. La recherche porte uniquement sur les pages chargées. Le rafraîchissement est volontaire, pas automatique.

Aucune composition, édition, acceptation de mission ou notification nouvelle n'est ajoutée. Les méthodes agent, leurs boucles de relance et leurs règles de communication restent inchangées.

Les choix d'implémentation et les limites de pages seront fixés dans le plan et les contrats après examen de l'existant. Cette spécification ne prescrit ni bibliothèque nouvelle ni changement de stockage.

## Espaces de travail isolés

Documents : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/specs/146-bridget-panel-lisible/`.

Code Bridget : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/146-bridget-panel-lisible/`.

[Note technique de l’adaptateur](../../docs/adapters/t3/archives/specs/146-bridget-panel-lisible/spec.md#extrait-4).
