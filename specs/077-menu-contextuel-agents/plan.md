# Plan d'implémentation - SPEC-077 Menu contextuel des agents

## Résumé

Transformer la fiche globale d'identité existante en menu contextuel unique. Le bouton à trois points, le clic droit et Maj+F10 ouvrent le même nœud DOM et le même modèle d'actions. Les commandes de cycle de vie réutilisent strictement SPEC-075. Les préférences d'épinglage, de masquage et de lecture sont locales, versionnées et validées.

## Résultat

Implémentation terminée dans les trois assets prévus. La validation ciblée
compte 146 tests passants sur 146, sans endpoint ni dépendance nouvelle.
L’audit v14 attribue la note A sans blocage de livraison.

## Contexte technique

- Projet: workspace Rust Bridget avec interface embarquée HTML, CSS et JavaScript sans framework.
- Interface productive: crates/bridget-daemon/assets/ui/index.html, app.js et theme.css.
- Tests UI: tests Node intégrés à app.js avec node:test.
- Backend: routes existantes dans crates/bridget-daemon/src/ui.rs.
- Persistance locale existante: localStorage pour largeur de colonne et apparence des agents.
- Dépendances nouvelles: aucune.
- Endpoint nouveau: aucun.
- Migration serveur: aucune.

## Constitution Check

| Gate | Décision |
|---|---|
| Langue française | Artefacts et libellés en français |
| Processus SpecKit complet | SPEC-077, plan, audit, tâches, Analyze, implémentation, Converge et audit |
| Worktree isolé | session-077-menu-contextuel-agents |
| Réutilisation avant création | Transformation de la fiche existante et réemploi des actions SPEC-075 |
| Test before next | Tests purs du modèle puis câblage DOM et CSS |
| Complexité | O(n log n) au plus pour la projection de la flotte, n étant le nombre d'agents |
| Minimalisme | Trois assets productifs modifiés, aucune dépendance ni endpoint |
| Responsabilité future | Un modèle d'actions unique, inspectable et testé, sans contexte caché |

Aucune violation constitutionnelle n'est requise.

## Architecture actuelle réutilisée

1. renderAgentButton construit la ligne, le bouton principal et les trois points.
2. identityCardData construit l'identité fournisseur et mode.
3. identityCardPosition borne la fiche à la fenêtre.
4. renderIdentityCard rend actuellement l'identité et les actions de cycle de vie.
5. agentLifecycleEligibility, openStopConfirmation et submitAgentStop portent déjà les règles et les verdicts SPEC-075.
6. readThrough et markSelectedReadIfEligible gèrent déjà la lecture dans la session.
7. renderAgents remplace les lignes sans perdre le défilement et rouvre la fiche si nécessaire.

## Conception cible

### 1. Modèle d'actions unique

Ajouter une fonction pure agentContextMenuItems qui reçoit l'agent et ses préférences locales. Elle retourne dans un ordre stable:

- ouvrir la conversation;
- épingler ou désépingler;
- marquer comme lu;
- masquer ou afficher dans la barre;
- arrêter;
- relancer;
- décommissionner.

Chaque entrée contient une clé, un libellé, un groupe, un état disponible, une raison éventuelle et un caractère destructif. Cette fonction concentre la matrice UX et évite toute divergence entre les déclencheurs.

### 2. Un seul menu global

Conserver le nœud global aujourd'hui nommé identityCard afin de ne pas créer une seconde couche. Le rendre comme menu d'application compact:

- role menu;
- en-tête non interactif avec nom, présence, logo, fournisseur et mode;
- groupes d'actions séparés visuellement;
- boutons role menuitem;
- aria-haspopup menu et aria-expanded sur les trois points.

Le menu reste attaché au body pour éviter le clipping horizontal de la colonne.

### 3. Trois voies d'ouverture

- Trois points: ancrage sur le bouton existant.
- Clic droit: événement contextmenu sur la ligne, prévention du menu natif, ancrage ponctuel aux coordonnées du pointeur.
- Clavier: ContextMenu ou Maj+F10 depuis le bouton de ligne, ancrage sur la ligne.

Toutes les voies appellent openIdentityCard avec le même agent et le même rendu.

### 4. Navigation clavier

À l'ouverture, le premier menuitem disponible reçoit le focus. Flèche bas et haut parcourent tous les éléments, y compris les commandes indisponibles afin que leur raison soit annoncée. Début et Fin vont aux extrémités. Échap ferme et restaure le déclencheur. Tab ferme puis poursuit la séquence normale. Les commandes indisponibles restent visibles avec aria-disabled, leur raison et une activation bloquée.

### 5. Préférences locales

Ajouter une clé bridget.ui.agent-sidebar-preferences.v1 contenant:

- version;
- pinned: noms épinglés;
- hidden: noms masqués;
- readThrough: dernier horodatage lu par nom.

La lecture valide le type, déduplique les noms, borne le volume et ignore toute valeur invalide. Toute erreur localStorage dégrade en préférences vides. Aucune donnée de message, jeton ou secret n'est persistée.

### 6. Projection de la barre

Une fonction pure agentSidebarProjection répartit les agents en actifs visibles, arrêtés visibles et masqués. Les épinglés passent avant les autres sans altérer l'ordre relatif déjà fourni par normalizeAgents. La nouvelle section Agents masqués réutilise le pattern HTML et CSS de Agents arrêtés.

La signature de rendu de la barre inclut les préférences normalisées afin qu'un épinglage, un masquage ou une restauration invalide immédiatement le rendu sans attendre un snapshot du daemon.

Le compteur de flotte conserve le nombre total d'agents actifs, y compris les agents volontairement masqués, afin de ne pas présenter une flotte fausse.

### 7. Lecture

L'action Marquer comme lu et la lecture automatique existante mettent à jour le même readThrough local. applyReadThrough continue de neutraliser un compteur serveur dont l'activité est antérieure ou égale au curseur local. Un seul répartiteur d'actions exécute les commandes produites par agentContextMenuItems, quelle que soit la voie d'ouverture.

### 8. Cycle de vie

Aucun changement de contrat. Les trois actions utilisent:

- agentLifecycleEligibility;
- buildAgentLifecycleUrl;
- openStopConfirmation;
- submitAgentStop;
- agentLifecycleFeedback.

Après succès, le snapshot réel décide de l'état affiché.

## Ordre d'implémentation

1. Ajouter les tests purs de préférences, projection et matrice d'actions.
2. Implémenter les fonctions pures dans app.js.
3. Ajouter la section Agents masqués dans index.html et les identifiants DOM.
4. Transformer le rendu de la fiche en menu compact.
5. Câbler clic droit, clavier, actions locales et persistance.
6. Adapter le CSS existant sans créer une seconde famille de composants.
7. Exécuter tests Node, tests Rust UI ciblés et diff check.
8. Converger exigence par exigence.

## Fichiers prévus

| Fichier | Modification |
|---|---|
| crates/bridget-daemon/assets/ui/app.js | préférences, projection, menu, événements et tests |
| crates/bridget-daemon/assets/ui/index.html | section Agents masqués |
| crates/bridget-daemon/assets/ui/theme.css | présentation compacte, groupes, éléments et section masquée |
| specs/077-menu-contextuel-agents/* | preuves et traçabilité |

## Tests prévus

- node --test crates/bridget-daemon/assets/ui/app.js
- cargo test -p bridget-daemon ui --lib -- --test-threads=1
- git diff --check
- contrôle source des rôles ARIA, contextmenu et absence d'endpoint nouveau.

## Risques et atténuations

| Risque | Atténuation |
|---|---|
| Deux menus divergents | Un seul nœud et une seule fonction agentContextMenuItems |
| Préférences corrompues | normalisation fermée et fallback vide |
| Agent masqué irrécupérable | section Agents masqués toujours disponible |
| Menu coupé | nœud global fixé au body et position bornée existante |
| Focus perdu après rafraîchissement | réouverture existante adaptée et restauration explicite |
| Action lifecycle mensongère | réemploi de l'éligibilité et des verdicts existants |
| Régression de défilement | renderAgents conserve le scrollTop actuel |
| Charge future | aucune dépendance, aucun endpoint, trois assets seulement |

## Déploiement

La skill n'effectue ni commit ni déploiement. Après intégration explicite, le relais UI doit être reconstruit puis redémarré pour charger les assets embarqués. Le daemon ne nécessite pas de changement de protocole.
