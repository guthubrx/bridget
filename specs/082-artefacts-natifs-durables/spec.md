# Spécification - SPEC-082 Artefacts natifs durables et vérifiables

<!-- SPEC-FORMALISM:START -->
## Fiche synthèse

Spec: 082-artefacts-natifs-durables
Titre: Publication Bridget d'artefacts natifs, durables et sourcés
Statut: Prête à implémenter
Priorité: P1

Résumé:
- Donner aux agents un moyen unique de publier des résultats structurés dans une conversation Bridget.
- Afficher naturellement graphiques, indicateurs, tableaux, chronologies, images et fichiers, sans demander deux fois les mêmes informations au moteur.
- Conserver un artefact, ses sources, ses données et ses versions comme une preuve durable et restaurable.
<!-- SPEC-FORMALISM:END -->

**Branche**: 082-artifact-publication
**Créée**: 2026-08-31
**Statut**: Prête à implémenter
**Priorité**: P1
**Dépendances**: SPEC-063, SPEC-069, SPEC-074, SPEC-080, SPEC-081
**Dépendant aval**: SPEC-083

## Contexte et problème

Un message Markdown explique correctement un résultat textuel, mais ne suffit pas à
transporter un graphique, une série de données, une table interactive, une image
issue d'une source ou un fichier utile. Demander au moteur de produire un texte,
puis une image ou une seconde version textuelle, duplique les informations et
augmente le risque de divergence.

Bridget doit introduire un résultat durable attaché à un tour de conversation,
projeté nativement dans le fil et publié par un seul mécanisme. L'artefact doit
rester compréhensible, vérifiable, exportable et restaurable, même lorsque les
octets de son cache ont été purgés. L'agent ne devient pas une autorité de
stockage ni un navigateur : Bridget conserve l'autorité de collecte, validation,
persistance et restitution.

La provenance est une obligation absolue. Un résultat externe ne doit jamais être
présenté comme un fait vérifié sans afficher la source, la date de collecte, les
unités et les transformations connues. Un résultat calculé ou synthétisé doit
indiquer honnêtement son origine.

## Objectifs

1. Permettre à un agent de publier un seul artefact structuré pour enrichir une
   réponse, sans balise Markdown magique ni seconde génération de contenu.
2. Rendre directement dans le fil les artefacts natifs de type graphique,
   indicateur, table, chronologie, image ou fichier.
3. Dériver les vues complémentaires d'un même contenu : rendu, résumé lisible,
   données tabulaires et exports proviennent du même artefact.
4. Conserver chaque version, sa provenance, sa recette de rendu et ses données
   afin qu'une conversation reste vérifiable dans le temps.
5. Gérer cache, rétention, épinglage, suppression, restauration et actualisation
   sans modifier silencieusement l'historique.
6. Maintenir une séparation stricte entre les artefacts d'un projet et les agents
   qui n'y ont pas explicitement accès.

## Hors périmètre

- Exécuter du HTML ou JavaScript fourni par un agent, traité par SPEC-083.
- Créer un navigateur Internet, une automatisation de navigateur ou un éditeur
  complet de contenu.
- Migrer ou convertir les anciens messages et conversations existants.
- Interpréter des balises Markdown cachées comme une demande de publication.
- Exposer les artefacts d'un projet à un autre agent sans mention ou partage
  explicite.
- Modifier silencieusement une version existante après une actualisation.

## Récits utilisateur et critères d'acceptation

### US1 - Publier une réponse enrichie une seule fois (Priorité : P1)

En tant qu'opérateur, je veux qu'un agent publie un résultat structuré dans sa
réponse lorsqu'il apporte réellement de la valeur, afin de consulter un
graphique, un tableau ou une synthèse sans recevoir plusieurs représentations
incohérentes.

**Pourquoi cette priorité** : c'est le contrat central. Sans publication unique,
un artefact devient une image décorative ou une duplication de Markdown.

**Test indépendant** : demander une visualisation fondée sur un jeu de données
connu, puis vérifier qu'un seul contenu source permet d'afficher le graphique,
le tableau et le résumé correspondant.

**Scénarios d'acceptation** :

1. **Étant donné** une demande explicite de visualisation, **quand** l'agent sait
   produire un résultat pertinent, **alors** il publie un artefact Bridget
   unique, attaché au tour qui contient sa réponse.
2. **Étant donné** un résultat compact qui améliore clairement la compréhension,
   **quand** l'agent le publie sans demande explicite, **alors** il se limite à
   un artefact natif pertinent et ne publie pas une page complète inattendue.
3. **Étant donné** un fournisseur ne sachant pas publier d'artefact, **quand**
   une visualisation lui est demandée, **alors** il répond honnêtement à cette
   limite au lieu d'émettre une pseudo-commande Markdown.
4. **Étant donné** un artefact invalide, **quand** Bridget le reçoit, **alors**
   Bridget refuse sa publication et renvoie une cause exploitable à l'agent,
   sans inscrire de résultat partiel trompeur.

### US2 - Lire un résultat et vérifier ses données (Priorité : P1)

En tant qu'opérateur, je veux consulter un artefact naturellement dans le fil,
comprendre son message essentiel et accéder à ses données et ses sources, afin
de ne pas avoir à choisir entre une belle visualisation et un résultat
vérifiable.

**Pourquoi cette priorité** : une visualisation ne doit pas masquer la nature,
la qualité ou les limites de l'information affichée.

**Test indépendant** : ouvrir un graphique avec plusieurs séries et une donnée
manquante, puis accéder à son résumé, ses valeurs, sa table, sa provenance et
son export sans réinterroger le moteur.

**Scénarios d'acceptation** :

1. **Étant donné** un artefact natif dans une réponse, **quand** l'opérateur
   ouvre la conversation, **alors** l'artefact apparaît inline à une hauteur
   naturelle et reste clairement rattaché au tour d'origine.
2. **Étant donné** un graphique, **quand** l'opérateur le consulte, **alors** il
   dispose d'un résumé textuel, des valeurs importantes et d'une vue tabulaire
   issue des mêmes données.
3. **Étant donné** une donnée incomplète, estimée ou manquante, **quand** elle
   est affichée, **alors** son état est visible sans empêcher la lecture des
   informations utiles restantes.
4. **Étant donné** un artefact sourcé, **quand** l'opérateur ouvre Données et
   source, **alors** il peut consulter et exporter sa provenance, ses données,
   ses hypothèses et sa recette de rendu.

### US3 - Conserver, actualiser et restaurer un résultat (Priorité : P1)

En tant qu'opérateur, je veux qu'un artefact puisse être retrouvé après une purge
de cache et actualisé explicitement, afin de garder un historique fiable sans
transformer mon stockage en archive illimitée.

**Pourquoi cette priorité** : l'artefact est utile lorsqu'il reste relié à la
conversation, même si son contenu lourd a été évacué du cache.

**Test indépendant** : publier une image distante, expirer son cache, retrouver
le message d'origine, demander sa restauration, puis vérifier le comportement
quand la source fournit les mêmes ou de nouveaux octets.

**Scénarios d'acceptation** :

1. **Étant donné** une actualisation ou une modification sauvegardée, **quand**
   l'opérateur la confirme, **alors** Bridget crée une nouvelle version reliée à
   l'original et ne réécrit jamais l'ancienne.
2. **Étant donné** une source actualisée, **quand** l'opérateur demande un
   rafraîchissement, **alors** Bridget conserve l'original, crée une nouvelle
   version si les octets changent et indique explicitement l'opération.
3. **Étant donné** un cache expiré mais un manifeste conservé, **quand**
   l'opérateur demande la restauration depuis le message d'origine, **alors**
   Bridget tente la récupération et affiche le résultat ou l'erreur réelle.
4. **Étant donné** un artefact épinglé, **quand** les politiques automatiques de
   cache s'exécutent, **alors** il n'est pas purgé sans une action manuelle de
   l'opérateur.

### US4 - Retrouver et partager un artefact au bon endroit (Priorité : P2)

En tant qu'opérateur, je veux retrouver les artefacts du projet courant dans le
panneau latéral et les référencer depuis une autre conversation, afin de
réutiliser un résultat sans exposer automatiquement mon contexte aux agents.

**Pourquoi cette priorité** : un artefact durable devient réellement utile
lorsqu'il est retrouvable sans devenir une fuite implicite de données.

**Test indépendant** : publier des artefacts dans deux projets, ouvrir le panneau
d'un projet, puis rechercher volontairement dans tous les projets et partager
une référence avec un agent choisi.

**Scénarios d'acceptation** :

1. **Étant donné** un projet actif, **quand** l'opérateur ouvre l'onglet
   Artefacts, **alors** il voit d'abord ceux de ce projet, triés et filtrables.
2. **Étant donné** un besoin de recherche historique, **quand** l'opérateur
   active explicitement la portée globale, **alors** il peut rechercher dans ses
   autres projets sans que cette portée devienne celle des agents.
3. **Étant donné** une référence à un artefact dans une réponse ultérieure,
   **quand** l'opérateur l'ouvre, **alors** Bridget rejoint ou ouvre la version
   exacte d'origine et indique s'il existe une version plus récente.
4. **Étant donné** un autre agent du même projet, **quand** aucune référence ne
   lui est partagée, **alors** il ne peut pas consulter l'artefact par simple
   appartenance au projet.

### US5 - Comprendre les échecs sans résultat muet (Priorité : P2)

En tant qu'opérateur, je veux qu'un artefact en échec, interrompu ou indisponible
reste visible avec des actions claires, afin de savoir ce qui manque et de
pouvoir reprendre sans perdre le contexte.

**Pourquoi cette priorité** : un placeholder silencieux masque un problème et
donne l'impression qu'un agent n'a rien fait.

**Test indépendant** : interrompre une publication, rendre une source
injoignable, puis remettre Bridget en ligne et vérifier les états et les actions
proposées.

**Scénarios d'acceptation** :

1. **Étant donné** une publication interrompue par l'opérateur, **quand** le
   tour reste affiché, **alors** il indique interrompu par vous et ne finalise
   ni ne présente de résultat comme publié.
2. **Étant donné** Bridget indisponible et un cache local absent, **quand**
   l'opérateur consulte le tour, **alors** une carte compacte indique
   l'indisponibilité, le titre, la version, la date et les sources connues.
3. **Étant donné** un échec de collecte, de rendu ou de restauration, **quand**
   l'opérateur ouvre l'artefact, **alors** il peut voir la cause, récupérer le
   manifeste ou les données disponibles et demander une reprise via Bridget.

## Cas limites

- La structure est valide mais le jeu de données est partiel : l'artefact est
  publiable avec une signalisation explicite, pas bloqué pour une complétude que
  la source ne permet pas.
- Les mêmes octets sont restaurés : Bridget rétablit la version existante au
  lieu de créer une fausse nouvelle version.
- La source distante change ou disparaît : l'ancienne version et sa provenance
  restent intactes ; la nouvelle tentative devient un fait distinct.
- Le cache local est purgé en mode serveur : le manifeste reste consultable et
  la récupération ne passe jamais directement par le navigateur de conversation.
- L'opérateur supprime une conversation : les artefacts non épinglés et non
  référencés ailleurs sont supprimés avec elle ; les autres conservent leurs
  références restantes.
- Un agent reçoit reprends : ce texte lui est transmis normalement, sans
  interprétation spéciale par l'interface ; il peut s'appuyer sur les états et
  versions déjà attestés pour éviter les doublons.

## Exigences

### Exigences fonctionnelles

- **FR-082-001** : Bridget DOIT fournir un unique mécanisme de publication
  d'artefact, quel que soit son type natif.
- **FR-082-002** : Une publication DOIT être rattachée à un tour, à une
  conversation, à un projet et à une version d'artefact identifiables.
- **FR-082-003** : Bridget DOIT accepter au minimum les types graphique,
  indicateur, table, chronologie, image et fichier.
- **FR-082-004** : La demande explicite d'une visualisation, d'un tableau
  graphique ou d'un indicateur engage l'agent capable à publier un artefact
  approprié ou à expliquer clairement pourquoi il ne le peut pas.
- **FR-082-005** : Bridget DOIT dériver rendu, résumé, table accessible et
  export d'un même contenu source, sans demander une seconde production au
  moteur.
- **FR-082-006** : Tout graphique DOIT proposer un résumé textuel, les valeurs
  significatives et une alternative tabulaire accessible.
- **FR-082-007** : Les artefacts DOIVENT afficher la provenance complète :
  origine, sources, date de collecte, unités, transformations, hypothèses,
  données manquantes et niveau d'affirmation applicable.
- **FR-082-008** : Une synthèse ou un calcul sans source externe DOIT indiquer
  explicitement qu'il provient des données fournies ou du raisonnement agent.
- **FR-082-009** : Bridget DOIT valider la structure, le type, les tailles et
  les références avant persistance, sans confondre validité structurelle et
  complétude des données.
- **FR-082-010** : Une donnée partielle DOIT pouvoir être publiée lorsqu'elle
  est honnêtement signalée et que l'artefact reste lisible.
- **FR-082-011** : Une version publiée est immuable. Toute actualisation,
  restauration modifiée ou sauvegarde d'une interaction crée une nouvelle
  version explicitement reliée à l'original.
- **FR-082-012** : Bridget DOIT permettre à l'opérateur de consulter, copier et
  exporter les données, le manifeste et les sources d'un artefact.
- **FR-082-013** : Les exports DOIVENT être adaptés au type de résultat et ne
  pas introduire de données qui n'appartiennent pas à la version exportée.
- **FR-082-014** : L'opérateur DOIT pouvoir épingler, désépingler, actualiser,
  restaurer et supprimer un artefact par une action explicite.
- **FR-082-015** : Les politiques globales de l'application DOIVENT prévoir un
  cache par défaut de 1 Gio et 30 jours, un avertissement à 8 Gio de contenus
  publiés et un blocage de nouvelle publication à 10 Gio jusqu'à libération
  d'espace ou modification explicite du réglage.
- **FR-082-016** : Les limites de cache et de rétention DOIVENT être
  configurables dans les réglages globaux de l'application, jamais par projet.
- **FR-082-017** : Une purge automatique ne DOIT pas supprimer un artefact
  épinglé ni un artefact publié avant sa politique explicite de conservation.
- **FR-082-018** : En mode local, Bridget DOIT conserver l'autorité canonique
  des artefacts localement ; en mode serveur, elle DOIT la conserver sur le
  serveur Bridget qui les a produits. Le Mac ne détient qu'un cache borné.
- **FR-082-019** : Toute collecte, restauration, actualisation ou récupération
  de contenu externe DOIT passer par Bridget, jamais directement par le rendu de
  conversation.
- **FR-082-020** : Le panneau Artefacts DOIT ouvrir par défaut la portée du
  projet courant et ne proposer la recherche globale qu'après choix explicite de
  l'opérateur.
- **FR-082-021** : Le partage avec un agent DOIT être explicite et créer une
  référence attestée ; l'appartenance au projet seule ne suffit pas.
- **FR-082-022** : Bridget DOIT afficher un état et des actions de reprise
  exploitables pour tout résultat interrompu, indisponible ou en erreur. Aucun
  résultat final ne DOIT se terminer par un placeholder muet.
- **FR-082-023** : Les anciens messages et conversations DOIVENT rester
  inchangés. Aucun mécanisme de migration ou d'inférence rétroactive n'est
  autorisé dans cette fonctionnalité.

### Exigences non fonctionnelles

- **Sécurité** : les sources, données et exports sont validés, bornés et liés à
  leur projet ; les agents ne reçoivent aucune capacité implicite de lecture
  globale, de stockage arbitraire ou de navigation externe.
- **Accessibilité** : chaque rendu visuel dispose d'un équivalent textuel ou
  tabulaire issu de la même version ; les commandes et états sont utilisables au
  clavier et ne reposent pas seulement sur la couleur.
- **Performance** : le fil charge en priorité ses textes et métadonnées ; les
  rendus lourds sont différés et ne doivent pas empêcher la lecture du tour.
- **Durabilité** : un manifeste minimal reste disponible après éviction du cache
  et décrit suffisamment une restauration ou un échec historique.
- **Confidentialité** : aucune collecte ne transmet de cookie de navigateur,
  secret local ou donnée hors de la demande attestée.

## Entités clés

- **Artefact** : résultat structuré durable, identifié, rattaché à un projet et
  à son message de publication.
- **Version d'artefact** : état immuable d'un artefact, relié à sa version
  précédente lorsqu'une évolution explicite est créée.
- **Manifeste de provenance** : description durable de l'origine, des sources,
  données, transformations, limites, recette de rendu et empreintes.
- **Contenu canonique** : octets et données autoritaires conservés par
  l'instance Bridget productrice.
- **Entrée de cache** : copie locale bornée, évictible et restaurable du contenu
  canonique.
- **Référence partagée** : lien explicite d'un artefact vers une conversation ou
  un agent, distinct de sa simple appartenance à un projet.

## Critères de succès

- **SC-082-001** : dans 100 % des scénarios de publication testés, un seul
  contenu source produit le rendu principal et ses vues textuelles, tabulaires
  et exportables.
- **SC-082-002** : 100 % des graphiques de recette de validation proposent un
  résumé, des valeurs importantes et une table accessible de la même version.
- **SC-082-003** : 100 % des artefacts externes de recette exposent source, date,
  unité et transformations avant d'être présentés comme résultats.
- **SC-082-004** : dans 100 % des essais d'actualisation, l'ancienne version
  demeure consultable et aucune donnée n'est écrasée silencieusement.
- **SC-082-005** : après éviction contrôlée du cache, 100 % des artefacts de
  recette conservent leur manifeste et proposent une restauration ou une cause
  d'échec explicite.
- **SC-082-006** : l'ouverture de l'onglet Artefacts affiche le projet actif sans
  révéler le contenu d'autres projets aux agents non destinataires.

## Hypothèses et dépendances

- SPEC-081 reste responsable du Markdown ordinaire, de la composition du tour,
  de la copie des blocs de code et des préférences de contenu existantes.
- SPEC-063 reste responsable du contrat d'interruption et de continuité des
  tours ; cette SPEC ne crée pas un interpréteur propriétaire de reprends.
- SPEC-074 fournit l'isolation de Bridget Desktop et des panneaux relayés.
- SPEC-080 fournit le centre de contrôle où les réglages globaux de cache et de
  conservation seront présentés.
- Les valeurs validées sont 1 Gio et 30 jours pour le cache, avec avertissement
  à 8 Gio et blocage à 10 Gio pour les contenus publiés.
- La version HTML interactive et le navigateur latéral sont traités par
  SPEC-083, qui réutilise ce modèle d'artefact.
