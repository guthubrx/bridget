# Spécification - SPEC-081 Conversation structurée et rendu technique sûr

<!-- SPEC-FORMALISM:START -->
## Fiche synthèse

Spec: 081-conversation-renderer
Titre: Conversation structurée, Markdown technique et contenus enrichis sûrs
Statut: In Progress
Priorité: P1

Résumé:
- Faire du fil Bridget une succession de tours lisibles : demande humaine, trace de travail, réponse et résultat.
- Conserver la preuve d'activité déjà disponible sans noyer la conversation dans des bulles ou des cartes.
- Présenter textes, tableaux et code de manière éditoriale, lisible et copiable.
- Donner à l'opérateur le contrôle local et explicite de l'ouverture des liens, fichiers et images.
<!-- SPEC-FORMALISM:END -->

**Branche**: `081-conversation-renderer`
**Créée**: 2026-08-31
**Statut**: In Progress
**Priorité**: P1
**Dépendances**: SPEC-069, SPEC-074, SPEC-076, SPEC-080

## Contexte et problème

Bridget sait déjà reconstituer les faits d'un échange : demande humaine,
remise, activités fournisseur, validations, raisonnement disponible, réponse,
durée et échec éventuel. Ces faits sont toutefois présentés comme une suite de
bulles et de zones d'activité distinctes. Sur une conversation longue,
l'opérateur doit reconstruire mentalement quelle action appartient à quelle
demande et où finit une interaction.

Le texte technique est actuellement sûr mais minimal. Les extraits de code,
les tableaux et les instructions ne disposent pas d'une présentation ni de
contrôles adaptés à leur consultation. Bridget interdit également par défaut
les liens et les images issus des agents. Cette protection est saine, mais
l'opérateur doit pouvoir l'assouplir localement et explicitement pour son
propre environnement de confiance, sans créer de capacité d'exécution ou
d'accès implicite.

L'objectif est de reprendre les qualités de lisibilité observées dans T3 Code,
dont le code est disponible sous licence MIT, sans recopier aveuglément son
architecture ni élargir les pouvoirs d'un contenu non fiable.

La première validation manuelle a toutefois invalidé l'acceptation visuelle de
US1 : le rendu Markdown était bien chargé, mais un texte simple produit une
réponse agent trop peu délimitée dans un fil large. L'opérateur voit des
phrases isolées et des vides, au lieu de reconnaître immédiatement le tour,
son activité et son résultat. Cette reprise complète la composition du tour,
sans remettre en cause le renderer Markdown ni les frontières de sécurité.

## Objectifs

1. Rendre chaque tour de conversation immédiatement identifiable, de la
   demande humaine à son résultat.
2. Distinguer visuellement la parole humaine, la réponse de l'agent et les
   traces d'exécution, sans perdre aucune preuve disponible.
3. Rendre les contenus Markdown et les blocs techniques confortables à lire,
   copier et parcourir dans les thèmes clair et sombre.
4. Permettre à l'opérateur de choisir localement si les liens, fichiers et
   images peuvent être rendus ou ouverts depuis une conversation.
5. Maintenir une frontière stricte : aucun texte, lien, image ou bloc de code
   provenant d'un agent ne peut déclencher une exécution, une commande, une
   modification de fichier ou un accès local sans action et autorisation
   explicites de l'opérateur.
6. Préserver l'honnêteté de remise, de réponse et d'activité établie par les
   SPEC-069 et SPEC-080.

## Hors périmètre

- Exécuter une commande depuis un bloc de code ou un lien de conversation.
- Ajouter un terminal intégré, un éditeur de code complet ou un navigateur
  distant.
- Modifier le protocole agent, le mécanisme de remise, les autorisations des
  agents ou le modèle de sécurité SSH.
- Ajouter des pièces jointes binaires, une gestion de fichiers générale ou un
  système de partage d'images.
- Afficher un lien, une image ou un fichier sans que la préférence locale
  correspondante l'autorise.
- Reproduire intégralement l'interface ou l'architecture de T3 Code.

## Récits utilisateur et critères d'acceptation

### US1 - Lire un tour complet sans reconstruire mentalement le fil (Priorité : P1)

En tant qu'opérateur, je veux distinguer immédiatement ma demande, les actes
de l'agent et sa réponse afin de pouvoir relire une conversation longue sans
confondre les interactions.

**Pourquoi cette priorité** : c'est le parcours central de Bridget. Une preuve
d'exécution qui existe mais ne peut pas être reliée à une demande n'aide pas à
piloter un agent.

**Test indépendant** : ouvrir un fil comportant plusieurs demandes, réponses,
activités, erreurs et rondes, puis identifier le résultat de cinq demandes
successives sans ouvrir les détails techniques inutiles.

**Scénarios d'acceptation** :

1. **Étant donné** une demande humaine suivie d'activités et d'une réponse,
   **quand** l'opérateur la consulte, **alors** la demande est affichée à
   droite, la réponse à gauche sans bulle envahissante, et les activités sont
   visiblement rattachées au même tour.
2. **Étant donné** plusieurs actes techniques pendant un tour, **quand**
   l'opérateur ne souhaite voir que le résultat, **alors** il voit un résumé
   compact et peut développer le détail sans perdre sa position de lecture.
3. **Étant donné** un tour interrompu, échoué ou sans réponse finale,
   **quand** il est consulté, **alors** son état réel est visible sans être
   confondu avec une réponse disponible.
4. **Étant donné** une ronde de vigilance ou un échange inter-agents,
   **quand** il est rendu, **alors** il conserve sa nature distincte et ne se
   fait pas passer pour une réponse humaine ou une réponse d'agent.
5. **Étant donné** une réponse agent en texte simple dans une conversation
   large, **quand** l'opérateur la parcourt, **alors** elle est identifiable
   comme réponse et rattachée visuellement à sa demande sans devenir une
   bulle identique au message humain ni laisser croire à des messages isolés.

---

### US2 - Lire et réutiliser un contenu technique (Priorité : P1)

En tant qu'opérateur, je veux que les explications, listes, tableaux et blocs
de code soient lisibles et puissent être copiés sans perte inutile afin de
réutiliser correctement les consignes et résultats des agents.

**Pourquoi cette priorité** : Bridget orchestre des agents techniques. Une
commande ou un extrait de configuration mal lisible devient une source
d'erreur, même lorsqu'il est correct.

**Test indépendant** : consulter un message contenant une liste, un tableau,
un appelout, un détail repliable et des blocs de code de plusieurs langages,
puis copier un bloc et un tableau.

**Scénarios d'acceptation** :

1. **Étant donné** un bloc de code identifié, **quand** il est affiché,
   **alors** son langage ou son nom de fichier est lisible, le code est
   coloré de manière compatible avec le thème et un contrôle permet de le
   copier.
2. **Étant donné** une ligne technique trop longue, **quand** l'opérateur
   active ou désactive son retour à la ligne, **alors** le changement reste
   borné au bloc concerné et le texte reste sélectionnable.
3. **Étant donné** un tableau, **quand** l'opérateur le consulte, **alors**
   il peut le lire sans que la colonne de conversation déborde et le copier
   dans un format réutilisable.
4. **Étant donné** une sélection dans une réponse riche, **quand** elle est
   copiée, **alors** le texte conserve autant que possible sa structure de
   lecture plutôt que d'être réduit à une suite de caractères sans forme.

---

### US3 - Maîtriser l'affichage des liens, fichiers et images (Priorité : P1)

En tant qu'opérateur, je veux décider localement si une conversation peut
afficher ou proposer d'ouvrir des liens, fichiers et images, afin d'adapter
Bridget au niveau de confiance de mon environnement sans donner de privilège
implicite aux agents.

**Pourquoi cette priorité** : les contenus d'agent sont non fiables par
défaut, mais un opérateur sur son propre environnement doit pouvoir activer
les capacités de lecture dont il a besoin.

**Test indépendant** : recevoir une réponse contenant un lien externe, un
lien de fichier et une image avec les préférences désactivées, puis les
activer localement et vérifier les comportements autorisés et refusés.

**Scénarios d'acceptation** :

1. **Étant donné** une installation nouvelle, réinitialisée ou inconnue,
   **quand** un agent fournit un lien, un fichier ou une image, **alors** aucun
   contenu actif n'est chargé ni ouvert automatiquement.
2. **Étant donné** les préférences de contenu enrichi désactivées,
   **quand** l'opérateur consulte un message, **alors** Bridget explique de
   façon concise que le contenu est bloqué par un réglage local, sans le
   remplacer par une erreur trompeuse.
3. **Étant donné** l'opérateur courant, **quand** il active explicitement les
   préférences de liens, de fichiers et d'images dans les réglages de
   l'application, **alors** son choix est conservé localement et s'applique à
   ses conversations sans modifier aucun serveur ni agent.
4. **Étant donné** une préférence activée, **quand** un lien, fichier ou
   image est rendu, **alors** aucune exécution ni écriture ne peut être
   déclenchée par le seul rendu ou par un clic non confirmé.
5. **Étant donné** la livraison de cette fonctionnalité pour l'opérateur
   actuel, **quand** il ouvre Bridget, **alors** ses trois préférences de
   consultation peuvent être actives localement, tandis que les valeurs par
   défaut restent désactivées pour tout nouveau profil ou toute réinitialisation.

---

### US4 - Retrouver une interaction ancienne sans perturber la lecture (Priorité : P2)

En tant qu'opérateur, je veux retrouver rapidement une demande antérieure et
son résultat sans que l'arrivée de nouvelles sorties me ramène de force en bas
du fil.

**Pourquoi cette priorité** : les échanges opérateur-agent sont longs et les
activités en direct ne doivent pas rendre l'historique inutilisable.

**Test indépendant** : consulter une conversation contenant au moins vingt
tours, remonter vers une demande antérieure pendant qu'un agent produit de
nouvelles activités, puis revenir au direct volontairement.

**Scénarios d'acceptation** :

1. **Étant donné** un fil suffisamment long, **quand** l'opérateur cherche une
   demande précédente, **alors** les repères disponibles permettent de
   rejoindre son tour sans balayage imprécis de toutes les sorties.
2. **Étant donné** l'opérateur hors du bas du fil, **quand** de nouvelles
   sorties arrivent, **alors** sa position est préservée et un indicateur lui
   permet de revenir au direct volontairement.
3. **Étant donné** un détail d'activité développé, **quand** son état ou sa
   hauteur change, **alors** le contenu que l'opérateur lisait reste visible.

## Cas limites

- Un même message humain est observé à la fois dans le journal et dans le flux
  de conversation : il apparaît une seule fois dans son tour.
- Un agent produit du texte avant, entre ou après plusieurs activités : chaque
  élément conserve son ordre réel et son rattachement au tour attesté.
- Un langage de code est absent, inconnu ou rendu impossible : le contenu reste
  lisible et copiable sans coloration trompeuse.
- Une préférence locale est corrompue, inconnue ou indisponible : Bridget
  revient à l'état sûr désactivé et l'indique sans interrompre le fil.
- Un lien ou une image ne respecte pas les règles de sécurité applicables :
  Bridget le bloque, n'exécute rien et explique le refus.
- Le fil contient un très grand nombre d'entrées : la consultation reste
  fluide et l'arrivée d'une entrée ne duplique aucun message ni n'efface le
  brouillon de l'opérateur.

## Exigences

### Exigences fonctionnelles

- **FR-001** : Bridget DOIT présenter une demande humaine, les traces de son
  tour et la réponse associée comme une interaction identifiable.
- **FR-002** : Bridget DOIT afficher la demande humaine à droite et la réponse
  d'agent à gauche avec une hiérarchie visuelle différente.
- **FR-003** : Bridget DOIT conserver les états de remise, d'activité,
  d'interruption, d'échec et de durée lorsqu'ils sont attestés.
- **FR-004** : Bridget DOIT permettre de développer et replier les détails
  d'activité d'un tour sans masquer son résumé ni modifier silencieusement la
  position de lecture.
- **FR-005** : Bridget DOIT rendre les structures Markdown prises en charge,
  les tableaux et les blocs techniques de façon lisible dans les deux thèmes.
- **FR-006** : Bridget DOIT permettre de copier un bloc de code et un tableau
  avec un retour visuel de réussite ou d'échec.
- **FR-007** : Bridget DOIT proposer une présentation sûre de contenus riches,
  dont liens, fichiers et images, contrôlée par des préférences locales
  distinctes.
- **FR-008** : Les préférences de contenu enrichi DOIVENT être désactivées par
  défaut pour une nouvelle installation, un profil inconnu et une valeur
  invalide.
- **FR-009** : L'opérateur actuel DOIT pouvoir activer localement les trois
  préférences de consultation demandées, sans requête de mutation vers un
  serveur Bridget.
- **FR-010** : Aucun contenu conversationnel ne DOIT déclencher seul une
  exécution, une écriture, une navigation automatique ou une élévation de
  privilège.
- **FR-011** : Bridget DOIT préserver la politique de défilement honnête de
  SPEC-069 et éviter toute duplication d'une demande humaine durable.
- **FR-012** : Toute réutilisation directe d'éléments T3 Code DOIT conserver
  les avis de licence MIT applicables et être traçable dans les artefacts de la
  feature.

### Exigences non fonctionnelles

- **Sécurité** : les préférences sont locales, versionnées et validées ; aucun
  secret, chemin sensible ou contenu de conversation n'est ajouté à un journal
  de préférence.
- **Accessibilité** : les détails, boutons de copie, actions de navigation et
  contenus bloqués sont utilisables au clavier, avec un nom accessible et un
  retour de statut non exclusivement visuel.
- **Performance** : l'ouverture d'un détail, la copie et l'arrivée d'une
  nouvelle entrée ne doivent pas figer le fil perceptiblement ; l'historique
  long ne doit pas imposer le rendu visible de toutes ses entrées en même temps.
- **Compatibilité** : le fil reste utilisable lorsque la coloration, le presse-
  papier, les préférences persistées ou un type de contenu enrichi ne sont pas
  disponibles.
- **Conformité de licence** : la provenance et les avis requis sont conservés
  pour tout code directement adapté depuis T3 Code.

## Entités clés

- **Tour de conversation** : regroupement attesté d'une demande humaine, de
  ses activités, de ses réponses, de son état et de son résultat.
- **Entrée de tour** : message, activité, résumé de travail, plan, échec ou
  échange spécial, conservant son ordre et son origine.
- **Préférence de contenu enrichi** : choix local et distinct autorisant ou
  bloquant la consultation de liens, fichiers ou images.
- **Bloc technique** : portion de message dont la lecture, la copie, le langage
  éventuel et le retour à la ligne sont gérés comme une unité.
- **Repère de conversation** : point de navigation qui permet de retrouver une
  demande et le résultat de son tour.

## Hypothèses et dépendances

- SPEC-069 reste la source de vérité pour les preuves de remise et la règle de
  défilement en direct.
- SPEC-074 fournit le contexte de l'application de bureau et ses limites de
  privilèges locaux.
- SPEC-076 et SPEC-080 conservent leurs responsabilités de projets et de
  réglages ; cette SPEC n'ajoute pas de réglage serveur.
- T3 Code est une source d'inspiration et de réutilisation MIT possible, mais
  Bridget conserve son architecture et ses frontières de sécurité propres.
- L'opérateur actuel demande que ses préférences locales de consultation soient
  activées à la livraison ; leur valeur par défaut de création demeure sûre et
  désactivée.

## Critères de succès

- **SC-001** : lors d'un test de lecture de cinq tours successifs, l'opérateur
  identifie la demande, les actes et la réponse de chacun sans ouvrir plus d'un
  détail par tour.
- **SC-002** : 100 % des messages humains portant le même identifiant durable
  sont affichés une seule fois dans les scénarios de reprise et de flux vivant.
- **SC-003** : 100 % des blocs de code testés restent copiables et lisibles
  lorsque leur langage est connu, inconnu ou indisponible.
- **SC-004** : sur une installation ou un profil réinitialisé, 100 % des liens,
  fichiers et images d'agent restent bloqués jusqu'à une activation explicite.
- **SC-005** : avec les préférences activées par l'opérateur, aucun test de
  rendu ou de clic de contenu enrichi ne déclenche une exécution ou une écriture
  sans confirmation distincte.
- **SC-006** : pendant la consultation d'un historique, 100 % des nouvelles
  sorties testées préservent la position de lecture jusqu'à l'action volontaire
  de retour au direct.
- **SC-007** : les contrôles ajoutés sont accessibles au clavier et les tests
  automatisés couvrent les refus de sécurité, les dégradations et les parcours
  de copie.
