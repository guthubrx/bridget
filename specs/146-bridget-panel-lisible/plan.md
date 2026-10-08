# Plan SPEC146 — Panneau Bridget plus lisible

Date : 2026-10-08. Statut : conception réalisée, validation et clôture en cours ; aucune installation146. Estimation validée : 45–75 minutes.

## Objectif

Présenter les fils selon leur activité récente et leurs messages récents en premier. Rendre la lecture plus compacte, sans toucher au travail des agents. Conserver le socle145 et prouver les nouvelles garanties sur des données isolées.

## Contexte et périmètre

Le socle145 est installé mais non committé. Deux worktrees146 reçoivent une copie identifiée des seuls fichiers145 utiles. Un manifeste d'import permet de distinguer ce socle des changements146. Les racines principales,145, la base réelle et les processus actifs restent intacts.

Les nouvelles opérations humaines `list_recent` et `history_recent` sont distinctes de `list`, `show` et `history`. Les méthodes145 et les méthodes agents gardent leur sémantique. Une capacité négociée `HumanThreadViewRecentV1` empêche d'accepter une réponse ancienne comme une réponse146. L'absence de cette capacité produit une incompatibilité explicite ; aucun repli vers une première page ancienne n'est admis.

## Architecture prévue

1. **Données Bridget.** Calculer l'activité sur le message de dernière séquence, avec création en repli pour un fil vide ; ne pas prendre le maximum des horodatages. Trier par activité DESC puis UUID ASC avant la limite. Le curseur `after` est le couple canonique activité/UUID du dernier fil émis. Conserver l'autorisation par appartenance.
2. **Historique récent.** Lire les messages de séquence décroissante dans un `snapshot_seq` fixé à l'ouverture. `before_seq` est inclusif ; la page suivante utilise `next_before_seq = dernière séquence émise − 1`. La borne de page vaut `through_seq = min(before_seq, snapshot_seq)`. Les publications ultérieures ne changent pas cet instantané. Le rafraîchissement en crée un autre.
3. **Transport humain.** Étendre le contrat fermé par deux variantes et la capacité récente. Conserver des singletons distincts pour Recent et145 dans la négociation et les deux gardes qui évitent la maintenance lors des lectures humaines. Préserver le refus avant initialisation ou démarrage. Les lecteurs145 restent inchangés ; aucun rôle agent n'est accordé.
4. **Frontière T3.** Étendre les schémas fermés, le lecteur existant et ses arguments CLI séparés. Conserver résolution serveur de la conversation et du projet, nettoyage des refus, budgets de sortie et annulation. Réutiliser la RPC `bridget.read` ; ne pas ajouter un second transport.
5. **État du panneau.** Employer les réponses récentes pour l'ordre initial et les pages. Conserver invalidation de contexte, génération de requête et rejet A → B → A. Fusionner par UUID en remplaçant le résumé déjà chargé par sa nouvelle observation, date comprise ; retrier l'ensemble chargé par activité DESC puis UUID ASC. Un simple append qui ignore le doublon conserverait une date périmée. Charger les pages seulement sur demande.
6. **Présentation.** Réutiliser le panneau Bridget, les composants et les jetons graphiques natifs. Une entrée montre titre, participants et date courte. Un message montre auteur, date et corps. Une séparation légère remplace l'effet de bloc dense. Les corps longs sont limités à quatre lignes visuelles, avec dépliage exact. Le contrôle de copie lit toujours le corps original entier. Les détails techniques montrent séquence, type français et remplacement disponibles.
7. **Accessibilité.** Donner un nom clair aux commandes, exposer leur état ouvert et fermé, conserver le focus visible et les commandes clavier. Tester panneau étroit, texte long et recherche sur portion repliée.

Une petite extraction de composant de message est permise si elle réunit ce comportement et améliore sa lisibilité. Aucun framework, dépendance, réglage global ou sélecteur d'ordre nouveau n'est requis.

## Garanties de pagination

La liste est globalement triée à chaque lecture. Elle est stable sans trou ni doublon lorsque les données ne changent pas. Elle n'est pas un instantané complet : une publication concurrente peut déplacer un fil. La vue ne l'affiche pas deux fois ; le rafraîchissement repart de la première page.

L'historique est un instantané borné. Les messages sont ordonnés par séquence décroissante. La position humaine ne modifie pas un curseur agent. La séquence du dernier message émis sert à calculer la suite, y compris lorsque la borne de taille limite une page avant sa borne de nombre. La projection des corrections emploie toujours `snapshot_seq`, jamais la borne inférieure de page : une correction visible dans l'instantané doit rester visible sur une page plus ancienne.

## Réutilisation et exclusions

Réutiliser les projections de stockage et contrôles d'appartenance Bridget. Réutiliser le lecteur T3, la RPC, le panneau, les composants Button/Input/ScrollArea et la copie native. Conserver les rôles humains distincts et les refus de la session145.

Aucune modification des boucles agent, ACK, missions ou notifications. Aucun compositeur. Aucun polling. Aucun accès à une base de conversation active pendant les tests. Aucun commit, fusion, push, installation ou redémarrage autorisé.

Les conventions Cartae de Next.js, Axios et traduction bilingue ne s'appliquent pas à ce panneau T3. Les libellés français suivent le contexte145 et les primitives T3 existantes. La session n'introduit pas un système de traduction parallèle.

## Contrôles prévus

- Tests RED sur le socle : ancien fil récemment actif hors première page ; vrais derniers messages d'un historique multipage ; ordre et dates cohérents.
- Tests Rust : appartenance, fils vides, égalités, curseur invalide, limites, historique descendant, fin à séquence1 sans curseur0, instantané, refus des versions anciennes, absence de mutation et voie CLI froide.
- Tests de frontière T3 : unions fermées, types entiers sûrs, curseurs bornés, argv sans shell, capability manquante et réponses incompatibles.
- Tests de panneau : quatre lignes, Déplier/Replier, détails français, copie entière, recherche dans le corps replié, auteurs, pagination et fusion d'un doublon avec activité mise à jour puis retri.
- Régressions145 : révocation, daemon absent, liaison absente ou ambiguë, timeout, stale A → B → A, fermeture, accès clavier, zéro wake/ACK/modèle.
- Contrôles applicables : tests ciblés puis type, lint et build. Consigner tous les warnings et les limites. Un résultat historique145 ne prouve pas une nouvelle vérification146.
- Aperçu isolé : composants réels sur données de test. Vérifier rendu normal et étroit, clavier, dépliage et copie. Ne pas ouvrir une nouvelle application de production.

## Gates

La spécification, la checklist, l'inventaire et l'audit de réutilisation sont rédigés. Les champs et bornes du contrat sont alignés avec les responsables Rust et T3. Le principal a validé la conception, Analyze, la réalisation et la clôture des dix tâches. Converge 1 est CONVERGED. Les deux audits ciblés ont passé leur validation stricte : code retour 0, zéro erreur et zéro warning de validation.

Avant clôture : preuves fonctionnelles, non-régressions, recette isolée et revue du diff. Aucun verdict inter-fournisseurs n'est revendiqué si l'annuaire du même projet ne fournit aucun autre fournisseur joignable. Ne pas solliciter d'autres projets pour remplir artificiellement cette exigence.

Statut final : Implemented, 10/10 tâches. Cette clôture n'autorise aucune opération Git ni installation ou redémarrage.
