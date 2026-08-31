# Spécification - SPEC-081 Flotte globale et sources

<!-- SPEC-FORMALISM:START -->
## Fiche synthèse

Spec: 081-flotte-globale-sources-ui
Titre: Flotte globale, sources et organisation des agents
Statut: Implémentée - prête pour revue, non livrée
Priorité: P1
Résumé: Bridget Desktop réunit les agents de plusieurs serveurs dans une flotte lisible. La colonne Sources applique des filtres explicites à cette flotte, sans changer de contexte applicatif ni confondre l'origine d'une action.
Périmètre: chaque projet reste rattaché à une seule source. La SPEC ne crée ni projet multi-serveur ni migration de projet.
<!-- SPEC-FORMALISM:END -->

**Branche**: `session-081-flotte-globale-sources-ui`
**Créée**: 2026-08-31
**Statut**: Implémentée - prête pour revue, non livrée
**Priorité**: P1
**Dépendances**: SPEC-065, SPEC-074, SPEC-076, SPEC-080

## Contexte

Bridget Desktop sait enregistrer plusieurs serveurs, mais la vue distante actuelle
recouvre l'application et ne montre qu'une seule source. L'opérateur ne peut ni
revenir simplement à ses serveurs ni voir dans une même liste les agents de son
ordinateur et de ses serveurs distants. La colonne Projets semble aussi changer
d'espace alors qu'elle devrait seulement restreindre les agents affichés.

L'opérateur veut arriver sur toute sa flotte, comprendre sans ambiguïté où vit
chaque agent, puis filtrer et organiser cette même liste selon le besoin.

## Objectifs

1. Afficher une flotte initiale unique de toutes les sources disponibles.
2. Faire de Sources une colonne de filtres explicites, pas une navigation entre
   applications.
3. Rendre la gestion et l'ajout de serveurs accessibles même lorsqu'une
   conversation distante est visible.
4. Associer toute carte et toute action d'agent à son serveur et projet d'origine.
5. Rendre les filtres multiples visibles, indépendants et supprimables.
6. Rendre les tris successifs réordonnables, inversables et supprimables.
7. Épingler les coordinateurs par défaut, avec un choix local de désépinglage.
8. Préserver le principe un projet = une source.

## Hors périmètre

- Projet simultanément actif sur plusieurs serveurs, synchronisation de dossier
  ou migration de projet.
- Changement du coordinateur, de sa session, de son fournisseur, de son modèle
  ou de ses permissions.
- Installation d'un service Bridget local absent.
- Modification des secrets, fournisseurs, conteneurs projet, permissions,
  rondes ou protocoles métier existants.
- Affichage de messages, chemins sensibles, jetons ou secrets dans la flotte.

## Principes directeurs

- **Flotte globale par défaut** : aucun filtre serveur ou projet n'est appliqué
  à l'ouverture.
- **Origine vérifiable** : le nom d'agent seul ne suffit jamais à cibler une
  conversation ou une action.
- **Projet sous source** : choisir un projet ajoute toujours sa source et son
  projet comme deux filtres distincts.
- **Aucune priorité opaque** : « À traiter d'abord » provient seulement d'une
  réponse attendue, d'un blocage ou d'une alerte observable.
- **Préférences locales** : filtres, tri, groupes repliés et épinglage restent
  sur le poste, sans modifier le serveur ni les agents.
- **État honnête** : une source indisponible ne devient jamais une source vivante
  sur la base d'une donnée périmée.

## Récits utilisateur et critères d'acceptation

### US1 - Retrouver toute la flotte - P1

Comme opérateur, je veux ouvrir Bridget sur la liste de tous les agents de mes
sources reliées, afin de retrouver un travail sans ouvrir chaque serveur.

**Test indépendant** : connecter deux sources, créer des agents homonymes dans
des projets différents et ouvrir la flotte.

1. **Étant donné** deux sources connectées, **quand** Bridget s'ouvre,
   **alors** une liste continue contient les agents des deux sources, sans
   regroupement serveur ou projet imposé.
2. **Étant donné** deux agents homonymes, **quand** ils sont affichés,
   **alors** leur serveur et leur projet restent lisibles et chacun ouvre la
   bonne conversation.
3. **Étant donné** une source perdue, **quand** la flotte se rafraîchit,
   **alors** les autres sources restent utilisables et la source perdue ne
   propose plus d'action.
4. **Étant donné** un agent choisi dans la flotte, **quand** l'opérateur ouvre
   ou contacte cet agent, **alors** Bridget utilise uniquement son origine.

### US2 - Filtrer par sources et projets - P1

Comme opérateur, je veux sélectionner une source ou un projet dans Sources, afin
de filtrer la flotte sans perdre la vue de ce qui est effectivement limité.

**Test indépendant** : sélectionner successivement un serveur, un projet puis
retirer chacune des chips de filtre.

1. **Étant donné** Toute la flotte, **quand** l'opérateur clique un serveur,
   **alors** la liste est filtrée et une chip serveur supprimable apparaît.
2. **Étant donné** un projet sous un serveur, **quand** l'opérateur le clique,
   **alors** deux chips indépendantes apparaissent, une pour le serveur et une
   pour le projet.
3. **Étant donné** plusieurs chips, **quand** une chip est retirée,
   **alors** seul son filtre disparaît.
4. **Étant donné** aucune source sélectionnée, **quand** l'opérateur crée ou
   importe un projet, **alors** Bridget lui demande sa source cible.
5. **Étant donné** une source locale, **quand** elle est affichée,
   **alors** son libellé est « Cet ordinateur », sans dépendre de la plate-forme.
6. **Étant donné** une conversation distante, **quand** l'opérateur choisit
   Gérer les serveurs, **alors** il y accède sans quitter l'application ni
   fermer involontairement son tunnel.

### US3 - Organiser la flotte - P1

Comme opérateur, je veux composer plusieurs critères de tri visibles, afin de
lire la même flotte par serveur, état, projet, activité ou nom.

**Test indépendant** : avec plusieurs serveurs, états et projets, ajouter trois
critères, les déplacer, inverser un sens et en retirer un.

1. **Étant donné** une flotte, **quand** l'opérateur ajoute Serveur, État et
   Projet, **alors** les groupes suivent l'ordre de leurs chips.
2. **Étant donné** plusieurs chips de tri, **quand** l'opérateur les réordonne,
   **alors** la hiérarchie suit ce nouvel ordre sans modifier les filtres.
3. **Étant donné** un tri alphabétique ou temporel, **quand** son sens est
   inversé, **alors** son ordre visible est réellement inversé.
4. **Étant donné** le tri État, **quand** l'opérateur choisit « À traiter
   d'abord », **alors** les réponses attendues, blocages et alertes passent
   devant les autres états.
5. **Étant donné** aucun tri ajouté, **quand** la flotte est visible, **alors**
   elle reste continue, stable et classée par activité récente.
6. **Étant donné** un regroupement, **quand** un groupe est replié,
   **alors** les filtres et les états d'agents restent intacts.

### US4 - Épingler les coordinateurs - P2

Comme opérateur, je veux voir chaque coordinateur au début de son contexte, tout
en pouvant le désépingler, afin de retrouver l'entrée principale d'un projet.

**Test indépendant** : découvrir un coordinateur, le désépingler, rafraîchir la
flotte, puis le réépingler.

1. Les coordinateurs sont épinglés par défaut.
2. L'épinglage est visible et peut être modifié sans modifier l'agent.
3. Dans une vue regroupée, un coordinateur épinglé reste au début de son groupe.
4. Le choix local persiste au rafraîchissement.

## Exigences fonctionnelles

- FR-8101: Toute la flotte est l'état initial, sans filtre source ni projet.
- FR-8102: La flotte réunit les projections des sources connectées avec une
  origine stable `source + agent`.
- FR-8103: Chaque carte affiche nom, état, serveur et projet ou absence de projet.
- FR-8104: Toute conversation, tout envoi et toute action sont routés vers
  l'origine attestée de l'agent.
- FR-8105: Sources contient Toute la flotte, les sources, leurs projets,
  Ajouter un serveur et Gérer les serveurs.
- FR-8106: Une sélection Sources produit des chips de filtre visibles et
  supprimables; un projet ajoute séparément source et projet.
- FR-8107: Les chips de filtre ne modifient jamais les chips de tri.
- FR-8108: Les critères disponibles sont au moins Serveur, État, Projet,
  Activité récente et Nom.
- FR-8109: Chaque chip de tri est réordonnable, supprimable et, lorsque le
  critère le permet, inversable.
- FR-8110: Les coordinateurs sont épinglés par défaut et l'opérateur peut
  modifier l'épinglage localement.
- FR-8111: Une source indisponible est explicite, n'empêche pas les autres
  sources et refuse les actions dirigées vers elle.
- FR-8112: Gérer les serveurs est atteignable depuis une conversation distante
  sans détruire par défaut ses connexions.
- FR-8113: Créer et Importer demandent une source cible sans filtre source actif.
- FR-8114: La source locale est nommée « Cet ordinateur ».
- FR-8115: Les préférences de présentation et d'organisation restent locales.

## Exigences non fonctionnelles

- NFR-8101: Pour 200 agents, filtre, retrait de chip et réorganisation locale
  affichent leur résultat en moins de 150 ms.
- NFR-8102: L'agrégation est linéaire avant le tri ou regroupement demandé, sans
  appel distant par carte.
- NFR-8103: Les chips sont accessibles au clavier et exposent libellé, ordre,
  sens et retrait.
- NFR-8104: Aucune nouvelle projection ne contient secret, jeton, clé privée,
  message complet ou chemin sensible.
- NFR-8105: Le contenu distant ne reçoit aucun privilège local par l'agrégation.

## Cas limites

- Aucune source, aucune source connectée ou Cet ordinateur indisponible.
- Agents ou projets homonymes sur deux sources.
- Source perdue pendant rafraîchissement ou ouverture d'agent.
- Agent disparu après affichage.
- Projet désactivé ou sans agent.
- Flotte vide après filtre.
- Critère de tri redondant avec un filtre.
- Tous les critères retirés.
- Coordinateur désépinglé puis redécouvert.

## Données et confidentialité

La projection globale se limite à l'origine de source, à l'identité d'agent, à
l'état, à l'identité de projet et aux métadonnées d'activité nécessaires à
l'affichage. Les préférences sont locales. Aucun jeton de relais, secret, clé
SSH, message, prompt, sortie d'agent ou chemin projet n'est ajouté.

## Hypothèses

- SPEC-074 fournit profils, tunnels et états de connexion des serveurs.
- SPEC-076 fournit les projets enregistrés et leurs actions par serveur.
- Cet ordinateur apparaît comme disponible seulement si Bridget y est joignable;
  cette SPEC ne l'installe pas.
- L'agrégation est une lecture côté Desktop, pas un transport entre serveurs.

## Critères de succès

- SC-8101: Avec trois sources et vingt agents, un agent homonyme est retrouvé
  et ouvert en trois interactions ou moins.
- SC-8102: 100 % des cartes globales montrent l'origine serveur et le projet
  lorsqu'il existe.
- SC-8103: Les tests prouvent la sélection source/projet et le retrait
  indépendant de chaque chip.
- SC-8104: Trois critères ordonnés donnent les groupes et sens demandés avec
  ordre stable dans chaque feuille.
- SC-8105: La perte d'une source laisse les autres consultables et aucune
  action ne part vers l'origine perdue.
- SC-8106: Gérer les serveurs est atteignable d'une conversation distante sans
  fermeture involontaire du tunnel.
- SC-8107: Les tests couvrent flotte vide, homonymes, source perdue, filtres,
  tris, épinglage, routage et fuite sensible.
