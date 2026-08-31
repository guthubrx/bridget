# Feature Specification: Espaces projets et coordinateur initial

<!-- SPEC-FORMALISM:START -->
## Fiche synthèse

Spec: 076-interface-projets-coordinateur
Titre: Espaces projets et coordinateur initial
Statut: Draft
Priorité: P1
Tâches: 0/42
Tests: non définis avant planification

Résumé:
- Contexte: SPEC-065 rend l'identité projet durable mais l'interface ne permet pas encore de créer, importer, ouvrir, retirer ni piloter un projet comme espace de travail.
- Objectif: faire d'un dossier serveur autorisé le projet visible dans Bridget, le rattacher à un coordinateur durable et ouvrir directement sa conversation.
- Limite: cette SPEC prépare et utilise les garanties de SPEC-066 et SPEC-067 sans redéfinir l'environnement, les secrets, les profils ou leur approbation.
- Risque principal: créer des doublons, laisser l'interface explorer le serveur librement ou remplacer silencieusement un coordinateur.
- Mitigation: racines serveur explicites, identité SPEC-065, prévols fail-closed, un coordinateur courant et aucune mutation silencieuse.
- Validation: les parcours créer, importer, reconnecter, retirer et revenir sont prouvés sans suppression de dossier ni création automatique d'agents secondaires.
<!-- SPEC-FORMALISM:END -->

**Feature Branch**: `session-076-interface-projets-coordinateur`
**Created**: 2026-08-30
**Status**: Draft
**Priority**: P1
**Dependencies**: SPEC-065, SPEC-066, SPEC-067, SPEC-072, SPEC-074, SPEC-075

## Contexte et problème

SPEC-065 donne à Bridget et Maicie une identité projet durable et une liaison
technique vers une racine hôte. Cette capacité est encore principalement
accessible par les surfaces d'exploitation. Une personne ne peut pas, depuis
l'interface, choisir un dossier serveur autorisé pour en faire un projet,
créer proprement un nouveau dossier projet, retrouver un projet déjà connu ou
entrer immédiatement dans l'échange avec son coordinateur.

Le produit ne doit pas inventer un second objet visible, comparable à un
espace séparé du système de fichiers. Pour l'utilisateur, un projet est un
dossier réel du serveur. Bridget ajoute l'identité durable de SPEC-065, le
contexte de coordination et l'historique, mais ne déplace pas le code et ne
cache pas où il se trouve.

Un coordinateur initial est nécessaire pour démarrer l'usage réel de Bridget.
Il doit être configuré avant son lancement, comprendre le projet de façon
approfondie, puis rendre un premier compte rendu lisible. Cette analyse ne doit
ni créer de fichier de mémoire Bridget, ni modifier le dépôt, ni créer d'autres
agents de son propre chef.

## Objectifs

1. Permettre à l'administrateur initial de gérer des racines de projets
   explicitement autorisées sur le serveur.
2. Permettre de créer un dossier projet nouveau ou d'importer un dossier
   existant, Git ou non, sans déplacer ni écrire dans le contenu existant sauf
   initialisation Git explicitement demandée.
3. Présenter chaque projet par le nom de son dossier et son chemin serveur,
   sans surnom obligatoire ni registre visible concurrent.
4. Configurer et lancer un unique coordinateur durable par projet à partir
   d'une configuration d'exécution réellement disponible et compatible.
5. Ouvrir la conversation durable du coordinateur lors de la sélection d'un
   projet et y rendre visible le déroulement du démarrage.
6. Faire produire au coordinateur une première analyse strictement en lecture
   seule, contrôlée par des créneaux de temps et suivie de propositions écrites.
7. Permettre le retrait, la réactivation et la reconnexion explicite d'un
   projet sans supprimer son dossier, son dépôt, son historique ou sa
   conversation.
8. Préserver une distinction nette entre le choix d'un outil agent, le
   fournisseur ou upstream, le modèle, le niveau d'effort et les permissions.

## Hors périmètre

- Créer, modifier, approuver, faire tourner ou révoquer un ProjectProfile, une
  extension ou un secret. Ces autorités restent celles de SPEC-067.
- Créer, démarrer, arrêter ou gérer le conteneur projet. Cela relève de
  SPEC-066.
- Remplacer, migrer ou transmettre la session d'un coordinateur vers un autre
  outil, fournisseur ou modèle. Une SPEC dédiée devra traiter cette bascule.
- Créer automatiquement des agents complémentaires, des modèles d'équipe ou
  des boutons de création d'agents à partir des propositions du coordinateur.
- Ajouter des comptes, rôles ou une administration multi-utilisateur dans la
  première version.
- Remplacer la connexion, les profils de serveur ou l'application de bureau de
  SPEC-074.
- Écrire un fichier de mémoire, un document Bridget, un commit Git, un
  .gitignore ou un premier contenu de projet sans confirmation explicite.
- Exposer une opération d'approbation locale à travers l'interface, MCP ou un
  agent.

## Principes directeurs

- **Le dossier est le projet visible**: le nom affiché vient du dossier et le
  chemin serveur complet reste consultable.
- **Le serveur n'est pas librement navigable**: seuls les sous-arbres autorisés
  par l'administrateur sont proposés, après résolution du chemin réel.
- **Une configuration complète**: outil agent, fournisseur ou upstream, modèle,
  effort et permissions sont distincts. Les choix proposés reflètent seulement
  des combinaisons attestées et compatibles.
- **Un coordinateur courant**: revenir dans le projet reprend sa conversation
  durable; un second coordinateur n'est jamais créé silencieusement.
- **Des états véridiques**: dossier prêt, coordinateur démarré, analyse en
  cours, compte rendu disponible et erreurs correspondent à des faits
  observables, jamais à une progression décorative.
- **Décision humaine aux frontières**: création Git, retrait, réactivation,
  poursuite d'analyse, changement de racine et remplacement futur demandent
  une action explicite.
- **Découverte sans écriture**: la première analyse respecte les limites de
  sécurité de SPEC-067 et n'exécute aucune opération qui modifie le projet.

## Récits utilisateur et critères d'acceptation

### US1 - Définir le cadre sûr et les valeurs par défaut (Priority: P1)

En tant qu'administrateur initial, je veux gérer les racines serveur où Bridget
peut proposer des projets et les valeurs préremplies d'un coordinateur futur,
afin de ne pas laisser l'interface choisir des accès ou une configuration à ma
place.

**Independent Test**: déclarer deux racines sûres, tenter de sélectionner un
dossier hors racine ou un lien symbolique sortant, modifier les valeurs par
défaut, puis vérifier que seuls les nouveaux projets les reçoivent.

**Acceptance Scenarios**:

1. **Given** aucune racine autorisée, **When** l'administrateur ouvre les
   réglages, **Then** il peut en déclarer une après confirmation et aucun
   parcours projet ne peut choisir une autre zone du serveur.
2. **Given** une racine autorisée, **When** l'explorateur parcourt son contenu,
   **Then** seuls des dossiers situés sous sa destination réelle sont proposés.
3. **Given** un lien symbolique, **When** sa destination réelle sort d'une
   racine autorisée, **Then** il est refusé avant toute lecture projet ou
   écriture durable.
4. **Given** une racine qui contient un projet actif, **When** l'administrateur
   veut retirer cette racine, **Then** Bridget bloque l'opération et nomme les
   projets à reconnecter ou retirer auparavant.
5. **Given** plusieurs configurations attestées, **When** les réglages sont
   consultés, **Then** ils distinguent outil agent, fournisseur ou upstream,
   modèle, effort et permissions sans supposer qu'un nom de produit désigne un
   fournisseur.
6. **Given** les valeurs par défaut changent, **When** un projet existant est
   rouvert, **Then** son coordinateur conserve sa configuration durable.

### US2 - Créer un nouveau projet et son coordinateur (Priority: P1)

En tant qu'utilisateur, je veux créer un dossier sous une racine autorisée,
choisir la configuration de son coordinateur et confirmer un récapitulatif,
afin d'arriver directement dans une vraie conversation de projet.

**Independent Test**: choisir une racine, créer un dossier, confirmer
l'initialisation Git proposée, démarrer un coordinateur compatible et vérifier
les étapes de démarrage dans sa conversation.

**Acceptance Scenarios**:

1. **Given** une racine autorisée et une configuration compatible disponible,
   **When** l'utilisateur choisit un nom de dossier inexistant, **Then**
   Bridget affiche le chemin final et ne crée le dossier qu'après confirmation.
2. **Given** le chemin final existe déjà, **When** l'utilisateur demande une
   création, **Then** Bridget refuse toute fusion ou écrasement et propose de
   basculer vers l'import.
3. **Given** la création d'un projet nouveau, **When** le récapitulatif est
   affiché, **Then** il montre dossier, option Git, configuration complète du
   coordinateur et durée d'analyse avant confirmation finale.
4. **Given** l'option Git est conservée par défaut, **When** la création est
   confirmée, **Then** seul un dépôt Git vide est initialisé et aucun commit,
   fichier ou règle Git additionnelle n'est créé.
5. **Given** aucune configuration compatible n'est disponible, **When** la
   création est demandée, **Then** Bridget renvoie vers les réglages sans créer
   ni enregistrer de projet.
6. **Given** le dossier et l'identité projet sont prêts, **When** le lancement
   du coordinateur échoue de façon imprévue, **Then** le projet reste visible
   avec l'état « coordinateur non démarré » et une action de correction ou de
   relance, sans supprimer le dossier.

### US3 - Importer, retrouver et reconnecter un dossier existant (Priority: P1)

En tant qu'utilisateur, je veux faire entrer un dossier de travail antérieur
dans Bridget sans le transformer ni perdre son historique existant.

**Independent Test**: importer un dépôt Git propre, un dépôt Git modifié et un
dossier non Git, puis déplacer ce dernier et le reconnecter explicitement.

**Acceptance Scenarios**:

1. **Given** un dossier existant inconnu sous une racine autorisée, **When**
   l'utilisateur l'importe, **Then** Bridget affiche son chemin, son état Git
   et sa situation de projet avant confirmation.
2. **Given** un dossier Git comportant des modifications locales, **When** il
   est importé, **Then** Bridget affiche un avertissement sans empêcher
   l'import ni modifier les fichiers.
3. **Given** un dossier non Git, **When** il est importé, **Then**
   l'initialisation Git est proposée, cochée par défaut mais modifiable et
   confirmée dans le récapitulatif final.
4. **Given** un dossier déjà associé à un projet actif, **When** il est choisi,
   **Then** Bridget ouvre directement ce projet sans créer une seconde identité
   ni un second coordinateur.
5. **Given** un dossier associé à un projet retiré, **When** il est choisi,
   **Then** Bridget propose explicitement de réactiver ce projet et son
   historique, sans en créer un nouveau.
6. **Given** la racine d'un projet a été déplacée, **When** son ancien chemin
   manque, **Then** Bridget indique cet état et ne cherche, ne déplace ni ne
   reconnecte automatiquement le dossier.
7. **Given** le nouveau chemin est confirmé sous une racine autorisée, **When**
   la reconnexion aboutit, **Then** l'identité et l'historique existants sont
   conservés.
8. **Given** des agents antérieurs non enregistrés travaillent dans le dossier,
   **When** le dossier est importé, **Then** ils restent historiques et ne sont
   pas reclassés silencieusement; seul le nouveau coordinateur porte le lien
   projet à son lancement.

### US4 - Entrer dans la conversation du coordinateur (Priority: P1)

En tant qu'utilisateur, je veux retrouver un coordinateur durable qui comprend
le projet avant de proposer la suite, afin de pouvoir commencer à travailler
naturellement dans sa conversation.

**Independent Test**: créer ou importer un projet, suivre l'analyse initiale,
quitter puis rouvrir le projet, et vérifier la conservation de la conversation,
de la configuration et des limites de découverte.

**Acceptance Scenarios**:

1. **Given** un projet sélectionné, **When** son espace s'ouvre, **Then**
   l'interface affiche directement la conversation durable du coordinateur,
   plutôt qu'un tableau de bord intermédiaire.
2. **Given** la conversation du coordinateur, **When** le démarrage progresse,
   **Then** les étapes système sont affichées de manière visuellement atténuée
   et distincte de ses messages.
3. **Given** une configuration encore compatible, **When** l'utilisateur
   revient dans le projet, **Then** la même conversation et le même
   coordinateur courant sont retrouvés.
4. **Given** une incompatibilité de configuration ou de version, **When** le
   projet est rouvert, **Then** Bridget conserve le projet et son coordinateur,
   explique la cause et ne substitue ni outil, ni fournisseur, ni modèle.
5. **Given** une découverte initiale, **When** elle s'exécute, **Then** le
   coordinateur se limite aux lectures autorisées, ne crée aucun fichier, ne
   modifie pas Git, ne crée aucun agent et ne lance aucune commande modifiante.
6. **Given** le premier créneau d'analyse, **When** il commence, **Then** sa
   durée préremplie est de dix minutes par défaut et l'utilisateur peut choisir
   trente minutes, une heure ou une durée personnalisée plafonnée à deux heures.
7. **Given** la fin d'un créneau, **When** un travail reste utile, **Then** le
   coordinateur rend un état intermédiaire et attend une confirmation explicite
   avant de poursuivre.
8. **Given** le compte rendu initial, **When** il est remis, **Then** il
   distingue faits observés, état et risques, questions à l'utilisateur et
   propositions éventuelles d'agents complémentaires, sans action automatique.

### US5 - Naviguer, retirer et réactiver sans perte (Priority: P2)

En tant qu'utilisateur, je veux naviguer entre la flotte et mes projets, puis
retirer un projet de l'interface sans toucher à son dossier ni à son historique.

**Independent Test**: naviguer entre plusieurs projets et « Toute la flotte »,
retirer un projet dont le coordinateur est actif, puis le réactiver depuis le
même dossier.

**Acceptance Scenarios**:

1. **Given** plusieurs projets, **When** l'interface est ouverte, **Then** une
   barre de projets repliable est présente à gauche de la liste d'agents avec
   une entrée permanente « Toute la flotte ».
2. **Given** un projet sélectionné, **When** la liste d'agents est affichée,
   **Then** elle montre le coordinateur et les agents rattachés à ce projet;
   « Toute la flotte » garde accès aux agents historiques ou non enregistrés.
3. **Given** un projet actif, **When** l'utilisateur demande son retrait,
   **Then** Bridget propose d'abord l'arrêt propre du coordinateur si nécessaire
   et explique que dossier, dépôt, messages et historique seront conservés.
4. **Given** le retrait confirmé, **When** l'opération aboutit, **Then** le
   projet disparaît de la barre active mais reste consultable comme retiré dans
   « Toute la flotte ».
5. **Given** un projet retiré, **When** il est réactivé explicitement, **Then**
   il retrouve sa place dans la barre de projets sans dupliquer son identité,
   sa conversation ni son historique.

## Exigences fonctionnelles

- **FR-001**: Bridget DOIT présenter un projet visible comme un dossier serveur
  dont le chemin canonique et l'identité SPEC-065 sont associés.
- **FR-002**: Bridget NE DOIT PAS afficher de nom de projet indépendant du nom
  du dossier dans cette version.
- **FR-003**: Bridget DOIT permettre de gérer des racines serveur autorisées
  depuis les réglages de l'application, sous contrôle de l'administrateur
  initial.
- **FR-004**: toute sélection, création, importation ou reconnexion de dossier
  DOIT être refusée si le chemin réel sort des racines autorisées.
- **FR-005**: Bridget DOIT refuser un lien symbolique dont la destination réelle
  sort des racines autorisées.
- **FR-006**: Bridget DOIT empêcher le retrait d'une racine qui rendrait un
  projet actif inaccessible, en listant les projets concernés.
- **FR-007**: l'explorateur de projets DOIT se limiter aux dossiers autorisés et
  ne DOIT PAS afficher le contenu de leurs fichiers.
- **FR-008**: une création DOIT demander une racine, un nom de dossier et une
  confirmation du chemin final.
- **FR-009**: une création NE DOIT jamais fusionner, écraser ou modifier un
  dossier déjà existant.
- **FR-010**: l'initialisation Git d'un nouveau projet DOIT être proposée par
  défaut, rester révocable et ne créer aucun commit ni fichier supplémentaire.
- **FR-011**: l'import d'un dossier non Git DOIT proposer l'initialisation Git
  dans les mêmes conditions, sans rendre Git obligatoire.
- **FR-012**: l'import d'un dépôt Git modifié DOIT avertir sans modifier son
  contenu ni bloquer l'import.
- **FR-013**: Bridget DOIT reconnaître une racine déjà active et ouvrir son
  projet existant au lieu de créer une identité ou un coordinateur en double.
- **FR-014**: Bridget DOIT proposer la réactivation explicite d'une racine
  précédemment retirée.
- **FR-015**: un déplacement de dossier DOIT rester en état manquant jusqu'à
  une reconnexion explicitement confirmée.
- **FR-016**: les valeurs par défaut d'un coordinateur DOIVENT distinguer outil
  agent, fournisseur ou upstream, modèle, effort et permissions.
- **FR-017**: une configuration sélectionnable DOIT correspondre à une
  combinaison attestée, disponible et compatible; Bridget NE DOIT PAS
  présenter une option future comme immédiatement utilisable.
- **FR-018**: une modification des valeurs par défaut NE DOIT PAS modifier la
  configuration durable d'un coordinateur existant.
- **FR-019**: en l'absence de configuration compatible, Bridget DOIT empêcher
  la création ou l'import avant toute écriture projet et guider vers les
  réglages.
- **FR-020**: un projet nouvellement créé ou importé DOIT avoir au plus un
  coordinateur courant et une conversation durable associée.
- **FR-021**: le retour dans un projet DOIT reprendre ce coordinateur et cette
  conversation, ou signaler honnêtement son indisponibilité.
- **FR-022**: une incompatibilité de version, d'outil ou de modèle DOIT être
  signalée sans substitution, relance ou remplacement automatique.
- **FR-023**: Bridget DOIT afficher un déroulement fiable des états de création
  et d'analyse dans la conversation projet.
- **FR-024**: la découverte initiale DOIT être en lecture seule et rester dans
  les autorisations valides du projet.
- **FR-025**: la découverte initiale NE DOIT créer ni modifier fichier, dépôt,
  agent, profil, secret, extension ou configuration.
- **FR-026**: la durée initiale de découverte DOIT être paramétrable et bornée;
  aucun mode illimité ne peut être proposé.
- **FR-027**: toute poursuite après échéance DOIT nécessiter une confirmation
  humaine explicite.
- **FR-028**: le premier compte rendu DOIT séparer constats, risques, questions
  et propositions, sans créer d'agent complémentaire.
- **FR-029**: la barre projet DOIT pouvoir être repliée et conserver l'accès à
  « Toute la flotte ».
- **FR-030**: sélectionner un projet DOIT ouvrir directement la conversation
  de son coordinateur.
- **FR-031**: retirer un projet DOIT être précédé d'une confirmation et d'une
  proposition d'arrêt propre de son coordinateur encore actif.
- **FR-032**: retirer un projet NE DOIT supprimer ni dossier, ni dépôt, ni
  conversation, ni historique ni identité durable.
- **FR-033**: un projet retiré DOIT rester consultable comme historique et
  pouvoir être réactivé explicitement depuis le même dossier.
- **FR-034**: Bridget DOIT préserver la distinction entre les agents
  historiquement non enregistrés et les agents rattachés au projet.
- **FR-035**: cette interface NE DOIT exposer aucune approbation ou rotation de
  profil, extension ou secret interdite par SPEC-067.
- **FR-036**: cette SPEC remplace la restriction de tranche de SPEC-065 qui
  réservait les mutations projet à la seule CLI, uniquement pour l'interface
  locale authentifiée; MCP, agents et API générique restent exclus.
- **FR-037**: « interface locale authentifiée » DOIT désigner un relais UI lié
  à la boucle locale du serveur. Bridget Desktop ne peut l'atteindre qu'au
  travers d'un tunnel SSH attesté par SPEC-074 et ne doit jamais l'exposer sur
  le réseau.
- **FR-038**: la réactivation DOIT être une mutation typée et idempotente qui
  restaure la même ProjectIdentity et une ProjectBinding active seulement après
  validation de la racine, avec un événement d'audit. Elle ne crée pas une
  seconde identité.
- **FR-039**: la valeur métier `display_name` requise par Maicie DOIT être
  dérivée du nom du dossier au moment de la création ou de l'import, sans champ
  ni surnom Bridget distinct dans cette version.

## Exigences non fonctionnelles

- **Sécurité**: aucune entrée utilisateur ne peut étendre les racines
  accessibles; les diagnostics et aperçus ne révèlent ni secret, ni contenu de
  fichier non nécessaire.
- **Fiabilité**: les parcours de création, import, retrait et reconnexion sont
  idempotents ou échouent avec un état de reprise explicite.
- **Compatibilité**: une évolution d'application ou d'exécution qui rend le
  coordinateur inutilisable est visible et réversible, jamais silencieusement
  remplacée.
- **Compréhension**: l'interface explique la différence entre création,
  importation, retrait, réactivation et reconnexion sans vocabulaire technique
  obligatoire.
- **Accessibilité**: la barre projets, l'explorateur, les confirmations et la
  conversation sont accessibles au clavier avec un focus déterministe.
- **Observabilité**: toute transition utile conserve une raison lisible et des
  éléments d'audit non sensibles.
- **Charge cognitive**: le premier écran d'un projet est la conversation utile,
  pas un tableau de bord imposé.

## Entités clés

- **Racine autorisée**: dossier serveur administré sous lequel les projets
  peuvent être créés, importés ou reconnectés.
- **Projet visible**: présentation d'une ProjectIdentity et de sa liaison
  SPEC-065 par le nom et le chemin de son dossier réel.
- **Configuration de coordinateur**: combinaison durable d'outil agent,
  fournisseur ou upstream, modèle, effort et permissions, soumise aux
  autorisations des spécifications dépendantes.
- **Coordinateur courant**: agent logique unique chargé de la conversation et
  du démarrage d'un projet.
- **Découverte initiale**: période de lecture bornée qui produit le premier
  compte rendu sans modifier le projet.
- **État d'onboarding**: état véridique du dossier, de l'enregistrement, du
  coordinateur et de l'analyse, visible dans la conversation.
- **Compatibilité**: verdict qui explique si la configuration durable peut
  toujours être employée sans changement silencieux.

## Hypothèses et dépendances

- SPEC-065 reste l'autorité de l'identité, de la liaison, du retrait et du
  rebind des projets.
- SPEC-076 remplace explicitement la limite de surface de SPEC-065 selon
  laquelle l'UI ne peut pas muter un projet. Cette ouverture reste bornée à
  l'interface locale authentifiée et ne délègue aucune autorité à MCP, à un
  agent ou à une API générique.
- Le relais UI local est lié à la boucle locale du serveur. Son accès depuis
  Bridget Desktop passe uniquement par un tunnel SSH attesté par SPEC-074, sans
  écoute réseau additionnelle.
- SPEC-066 fournira le cycle d'environnement projet; cette SPEC ne le remplace
  pas et ne peut être implémentée complètement avant sa livraison.
- SPEC-067 fournira les profils, permissions, secrets et approbations bornés;
  cette SPEC ne peut pas lancer un coordinateur réel hors de ce contrat.
- SPEC-072 fournit la provenance explicite des fournisseurs et empêche
  d'assimiler un outil agent à un fournisseur ou modèle.
- SPEC-074 portera ultérieurement la même interface par origine serveur dans
  Bridget Desktop, sans transférer à SPEC-076 la responsabilité SSH.
- SPEC-075 fournit les opérations génériques d'arrêt et de relance utilisées
  lors du retrait ou de l'indisponibilité du coordinateur.
- La première version concerne un administrateur initial unique. Une politique
  multi-utilisateur nécessite une spécification distincte.

## Critères de succès

- Un utilisateur peut créer ou importer un projet prêt à converser en moins de
  trois minutes lorsqu'une racine et une configuration compatible existent.
- Dans 100 % des essais contrôlés, un chemin ou lien symbolique hors racine
  autorisée est refusé avant enregistrement ou création.
- Dans 100 % des essais de dossier déjà connu, Bridget ouvre ou réactive le
  projet existant sans créer de seconde identité ni second coordinateur.
- Dans 100 % des essais de retrait, le dossier, le dépôt Git, les messages et
  l'historique restent présents après l'opération.
- Dans 100 % des essais de découverte initiale, aucune écriture projet, création
  d'agent secondaire ou mutation Git n'a lieu avant une instruction humaine
  ultérieure.
- Un utilisateur qui revient sur un projet retrouve la conversation de son
  coordinateur ou une raison explicite d'indisponibilité, jamais une
  substitution silencieuse.
- Au terme de chaque créneau d'analyse, le coordinateur remet un état lisible
  et attend une confirmation avant de poursuivre.
