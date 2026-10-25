# Archives techniques de l’adaptateur T3

Document d’origine : [specs/146-bridget-panel-lisible/plan.md](../../../../../../specs/146-bridget-panel-lisible/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

1. **Données Bridget.** Calculer l'activité sur le message de dernière séquence, avec création en repli pour un fil vide ; ne pas prendre le maximum des horodatages. Trier par activité DESC puis UUID ASC avant la limite. Le curseur `after` est le couple canonique activité/UUID du dernier fil émis. Conserver l'autorisation par appartenance.
2. **Historique récent.** Lire les messages de séquence décroissante dans un `snapshot_seq` fixé à l'ouverture. `before_seq` est inclusif ; la page suivante utilise `next_before_seq = dernière séquence émise − 1`. La borne de page vaut `through_seq = min(before_seq, snapshot_seq)`. Les publications ultérieures ne changent pas cet instantané. Le rafraîchissement en crée un autre.
3. **Transport humain.** Étendre le contrat fermé par deux variantes et la capacité récente. Conserver des singletons distincts pour Recent et145 dans la négociation et les deux gardes qui évitent la maintenance lors des lectures humaines. Préserver le refus avant initialisation ou démarrage. Les lecteurs145 restent inchangés ; aucun rôle agent n'est accordé.
4. **Frontière T3.** Étendre les schémas fermés, le lecteur existant et ses arguments CLI séparés. Conserver résolution serveur de la conversation et du projet, nettoyage des refus, budgets de sortie et annulation. Réutiliser la RPC `bridget.read` ; ne pas ajouter un second transport.
5. **État du panneau.** Employer les réponses récentes pour l'ordre initial et les pages. Conserver invalidation de contexte, génération de requête et rejet A → B → A. Fusionner par UUID en remplaçant le résumé déjà chargé par sa nouvelle observation, date comprise ; retrier l'ensemble chargé par activité DESC puis UUID ASC. Un simple append qui ignore le doublon conserverait une date périmée. Charger les pages seulement sur demande.
6. **Présentation.** Réutiliser le panneau Bridget, les composants et les jetons graphiques natifs. Une entrée montre titre, participants et date courte. Un message montre auteur, date et corps. Une séparation légère remplace l'effet de bloc dense. Les corps longs sont limités à quatre lignes visuelles, avec dépliage exact. Le contrôle de copie lit toujours le corps original entier. Les détails techniques montrent séquence, type français et remplacement disponibles.
7. **Accessibilité.** Donner un nom clair aux commandes, exposer leur état ouvert et fermé, conserver le focus visible et les commandes clavier. Tester panneau étroit, texte long et recherche sur portion repliée.

<a id="extrait-2"></a>

## Extrait 2

Réutiliser les projections de stockage et contrôles d'appartenance Bridget. Réutiliser le lecteur T3, la RPC, le panneau, les composants Button/Input/ScrollArea et la copie native. Conserver les rôles humains distincts et les refus de la session145.

<a id="extrait-3"></a>

## Extrait 3

Les conventions Cartae de Next.js, Axios et traduction bilingue ne s'appliquent pas à ce panneau T3. Les libellés français suivent le contexte145 et les primitives T3 existantes. La session n'introduit pas un système de traduction parallèle.

<a id="extrait-4"></a>

## Extrait 4

- Tests RED sur le socle : ancien fil récemment actif hors première page ; vrais derniers messages d'un historique multipage ; ordre et dates cohérents.
- Tests Rust : appartenance, fils vides, égalités, curseur invalide, limites, historique descendant, fin à séquence1 sans curseur0, instantané, refus des versions anciennes, absence de mutation et voie CLI froide.
- Tests de frontière T3 : unions fermées, types entiers sûrs, curseurs bornés, argv sans shell, capability manquante et réponses incompatibles.
- Tests de panneau : quatre lignes, Déplier/Replier, détails français, copie entière, recherche dans le corps replié, auteurs, pagination et fusion d'un doublon avec activité mise à jour puis retri.
- Régressions145 : révocation, daemon absent, liaison absente ou ambiguë, timeout, stale A → B → A, fermeture, accès clavier, zéro wake/ACK/modèle.
- Contrôles applicables : tests ciblés puis type, lint et build. Consigner tous les warnings et les limites. Un résultat historique145 ne prouve pas une nouvelle vérification146.
- Aperçu isolé : composants réels sur données de test. Vérifier rendu normal et étroit, clavier, dépliage et copie. Ne pas ouvrir une nouvelle application de production.

<a id="extrait-5"></a>

## Extrait 5

La spécification, la checklist, l'inventaire et l'audit de réutilisation sont rédigés. Les champs et bornes du contrat sont alignés avec les responsables Rust et T3. Le principal a validé la conception, Analyze, la réalisation et la clôture des dix tâches. Converge 1 est CONVERGED. Les deux audits ciblés ont passé leur validation stricte : code retour 0, zéro erreur et zéro warning de validation.
