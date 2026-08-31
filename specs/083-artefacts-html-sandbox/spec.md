# Spécification - SPEC-083 Artefacts HTML sandboxés et navigateur latéral

<!-- SPEC-FORMALISM:START -->
## Fiche synthèse

Spec: 083-artefacts-html-sandbox
Titre: Artefacts HTML/JavaScript isolés et navigateur Bridget latéral
Statut: Prête à implémenter
Priorité: P1

Résumé:
- Ajouter un type d'artefact HTML/JavaScript interactif, rendu inline dans une sandbox stricte.
- Fournir un navigateur latéral utilisable par l'opérateur pour les artefacts, contenus locaux publiés et pages Internet.
- Conserver l'autorité de données, de réseau et de sécurité dans Bridget, jamais dans l'artefact ni dans le navigateur de conversation.
<!-- SPEC-FORMALISM:END -->

**Branche**: 082-artifact-publication
**Créée**: 2026-08-31
**Statut**: Prête à implémenter
**Priorité**: P1
**Dépendances**: SPEC-074, SPEC-080, SPEC-081, SPEC-082

## Contexte et problème

Certaines réponses méritent davantage qu'un graphique ou un tableau natif : une
simulation, un tableau de bord interactif, une page d'explication ou une
visualisation composée. Le HTML accompagné de JavaScript peut créer cette
expérience, mais il est dangereux lorsqu'il peut appeler le réseau, accéder aux
fichiers, aux secrets, à la fenêtre parente ou aux privilèges locaux.

L'opérateur veut aussi un vrai espace de droite. Il doit pouvoir y consulter un
artefact développé, retrouver fichiers et liens publiés, et naviguer lui-même
vers une page Internet classique. Cette capacité ne doit pas transformer le
navigateur en extension privilégiée d'un agent ni donner aux artefacts exécutés
le droit de sortir de leur isolation.

La solution doit reproduire l'ergonomie naturelle observée dans les harnesses
modernes : contenu inline utile, action explicite pour agrandir, navigateur
latéral discret et historique lisible. Elle ne doit pas recopier leur code ni
présenter une page distante ou une image en échec comme un résultat silencieux.

## Objectifs

1. Permettre la publication d'un artefact HTML/JavaScript interactif à travers
   le même cycle Bridget que les artefacts natifs.
2. Exécuter cet artefact dans une sandbox dont les données, le réseau, les
   fichiers, les secrets, les privilèges locaux et la navigation sont isolés.
3. Afficher l'artefact dans le fil jusqu'à 1 200 px de haut, puis proposer une
   ouverture explicite dans le panneau de droite.
4. Proposer à l'opérateur un navigateur latéral normal, distinct de ses
   navigateurs personnels et sans privilège agent implicite.
5. Faire passer toute collecte, ouverture de lien d'artefact, image externe ou
   restauration par Bridget et conserver la provenance associée.
6. Organiser l'espace droit autour de Browser, Artefacts, Fichiers, Liens et
   Activité, sans ajouter de palette de commandes dans ce lot.

## Hors périmètre

- Accorder à un agent le contrôle direct du navigateur ou l'accès à ses cookies,
  mots de passe, historique ou profil.
- Ajouter maintenant la sélection de nœuds DOM, l'automatisation de navigation
  ou l'édition d'une page Internet par un agent, même avec consentement.
- Autoriser l'artefact HTML à appeler Internet, le système de fichiers, une
  commande, une base locale ou une API native.
- Migrer les anciens fichiers HTML ou conversations vers des artefacts.
- Lancer une navigation, un téléchargement, une impression ou une exportation
  sans geste explicite de l'opérateur.
- Réinventer un renderer de graphiques, tableaux, Markdown ou coloration quand
  une dépendance locale sûre répond au besoin.

## Récits utilisateur et critères d'acceptation

### US1 - Utiliser une visualisation HTML interactive sans élargir ses privilèges (Priorité : P1)

En tant qu'opérateur, je veux consulter dans le fil une visualisation HTML ou
JavaScript interactive, afin d'explorer un résultat riche sans que ce contenu
accède à mon Mac, à mon réseau ou à mes secrets.

**Pourquoi cette priorité** : l'interactivité est le bénéfice du format HTML,
mais elle n'est acceptable que si ses droits restent plus petits que ceux de
l'application qui l'héberge.

**Test indépendant** : publier une visualisation qui filtre localement ses
données, tente des accès interdits et propose un lien source, puis vérifier que
seul le filtre fonctionne et que le lien suit le parcours Bridget explicite.

**Scénarios d'acceptation** :

1. **Étant donné** un artefact HTML valide, **quand** la réponse est affichée,
   **alors** il est rendu inline dans une zone isolée et ne dépasse pas 1 200 px
   de hauteur sans action de l'opérateur.
2. **Étant donné** un script d'artefact, **quand** il s'exécute, **alors** il
   peut manipuler ses données autorisées et son propre rendu, mais ne peut pas
   joindre Internet, lire un fichier, accéder à un secret, invoquer Bridget ou
   naviguer la conversation.
3. **Étant donné** une tentative de ressource externe ou de sortie de sandbox,
   **quand** elle est bloquée, **alors** l'opérateur voit un état explicite et
   peut consulter le manifeste ou demander l'action adaptée à Bridget.
4. **Étant donné** un artefact plus haut que sa zone inline, **quand**
   l'opérateur choisit de l'agrandir, **alors** il s'ouvre dans le panneau droit
   sans réécrire ni déplacer silencieusement le message original.

### US2 - Explorer et sauvegarder une interaction sans réécrire l'historique (Priorité : P1)

En tant qu'opérateur, je veux ajuster temporairement un filtre ou un curseur dans
un artefact HTML, puis sauvegarder volontairement le résultat, afin de tester
une lecture sans perdre ni falsifier la version publiée par l'agent.

**Pourquoi cette priorité** : l'interactivité a une valeur immédiate, mais une
conversation ne doit jamais changer parce qu'un contrôle a bougé localement.

**Test indépendant** : changer un filtre, fermer puis rouvrir l'artefact,
ensuite demander une sauvegarde et vérifier qu'une version explicitement nommée
apparaît à côté de l'original.

**Scénarios d'acceptation** :

1. **Étant donné** une interaction dans un artefact, **quand** l'opérateur ne
   la sauvegarde pas, **alors** elle reste temporaire et la version originale
   demeure inchangée.
2. **Étant donné** un état interactif utile, **quand** l'opérateur choisit
   Enregistrer comme nouvelle version, **alors** Bridget crée une version liée,
   explicitement marquée et dotée de sa propre provenance.
3. **Étant donné** une exportation ou une copie demandée, **quand** elle est
   produite, **alors** elle correspond à la version ou à l'état explicitement
   sélectionné et n'altère aucun historique.

### US3 - Utiliser l'espace droit comme un navigateur d'opérateur (Priorité : P1)

En tant qu'opérateur, je veux ouvrir et fermer un navigateur latéral où je peux
voir un artefact, une page locale publiée ou une page Internet classique, afin
de travailler sans quitter Bridget ni confondre navigation personnelle et
exécution d'agent.

**Pourquoi cette priorité** : l'artefact développé et la navigation sont deux
activités de consultation continues qui ne doivent pas étouffer la conversation.

**Test indépendant** : ouvrir un artefact, une URL HTTPS choisie et une page
locale publiée dans le panneau Browser, masquer puis réafficher le panneau et
vérifier que la conversation conserve son état.

**Scénarios d'acceptation** :

1. **Étant donné** un lien ou l'action Ouvrir, **quand** l'opérateur la
   déclenche, **alors** Bridget ouvre la destination dans l'onglet Browser du
   panneau droit après validation de l'action autorisée.
2. **Étant donné** le navigateur droit, **quand** l'opérateur navigue lui-même
   vers une page HTTPS, **alors** il utilise une session de navigation isolée de
   ses navigateurs personnels et des agents.
3. **Étant donné** l'usage du panneau droit, **quand** l'opérateur le masque, le
   réaffiche ou agrandit l'espace de travail, **alors** la conversation et l'état
   de navigation ne sont pas perdus inutilement.
4. **Étant donné** une session de navigation persistée, **quand** l'opérateur
   demande l'effacement de ses données de navigation, **alors** Bridget efface
   cette session sans toucher aux artefacts canoniques ni aux conversations.

### US4 - Retrouver les sources et contenus publiés sans devenir un explorateur de disque (Priorité : P2)

En tant qu'opérateur, je veux retrouver dans l'espace droit les artefacts, les
fichiers publiés et les liens connus de la conversation ou du projet, afin de
reprendre un travail sans ouvrir un accès général au système de fichiers.

**Pourquoi cette priorité** : le contexte lié à une conversation est utile,
mais Bridget ne doit pas se changer en gestionnaire de fichiers privilégié.

**Test indépendant** : publier un fichier, une image, un lien source et un
artefact, puis vérifier leur présence dans les onglets correspondants et
l'absence de tout fichier local non publié.

**Scénarios d'acceptation** :

1. **Étant donné** un projet actif, **quand** l'opérateur ouvre le panneau,
   **alors** les onglets Artefacts, Fichiers et Liens affichent les éléments
   publiés ou référencés dans la portée sélectionnée.
2. **Étant donné** un fichier ou une image non disponible, **quand**
   l'opérateur le consulte, **alors** Bridget indique l'erreur réelle et une
   action de restauration adaptée, jamais un placeholder final muet.
3. **Étant donné** un lien détecté dans du Markdown, **quand** il n'est pas
   explicitement ouvert, **alors** Bridget n'effectue aucune prévisualisation
   réseau automatique.
4. **Étant donné** une recherche globale d'artefacts ou de liens, **quand** elle
   est activée par l'opérateur, **alors** elle reste une capacité de son
   interface et n'élargit pas la visibilité d'un agent.

### US5 - Préserver une frontière claire entre navigateur, artefact et agent (Priorité : P2)

En tant qu'opérateur, je veux savoir quel composant a accès à quelle donnée,
afin d'utiliser le navigateur et les artefacts riches avec confiance.

**Pourquoi cette priorité** : HTML actif et navigation sont utiles mais leur
confusion créerait une élévation de privilège difficile à détecter.

**Test indépendant** : consulter les détails de sécurité d'un artefact et du
navigateur, puis tenter depuis un agent ou l'artefact d'atteindre une donnée de
session navigateur.

**Scénarios d'acceptation** :

1. **Étant donné** un artefact HTML, **quand** l'opérateur consulte ses détails,
   **alors** il voit ses capacités actives, ses données injectées et ses accès
   explicitement refusés.
2. **Étant donné** un navigateur ayant conservé une session Internet, **quand**
   un agent ou un artefact est actif, **alors** ni l'un ni l'autre ne peut lire
   ses cookies, son contenu privé ou ses identifiants.
3. **Étant donné** une action demandant une collecte ou une ouverture depuis un
   artefact, **quand** elle exige du réseau ou une donnée externe, **alors**
   Bridget en devient l'intermédiaire traçable et applique les règles globales
   de sécurité et de provenance.

## Cas limites

- Un artefact HTML inclut un script, un formulaire, une navigation ou une
  ressource distante : seule l'interactivité locale autorisée fonctionne ; les
  autres tentatives sont refusées explicitement.
- Le navigateur est hors ligne : les artefacts déjà mis en cache restent
  consultables selon leur état ; les pages distantes indiquent l'absence de
  connexion sans lancer de récupération cachée.
- Une page Internet redirige : Bridget applique sa politique de navigation et
  ne laisse pas une redirection contourner la validation de destination.
- L'opérateur ouvre une version ancienne : elle reste la version exacte,
  accompagnée d'un indicateur non intrusif lorsqu'une version plus récente existe.
- Un artefact HTML est illisible ou trop grand : Bridget n'exécute pas un rendu
  dégradé silencieux et propose ses détails, son export ou son erreur.
- Les contrôles de visualisation n'ont pas besoin du réseau : ils continuent à
  fonctionner sur les données déjà injectées, sans inventer de nouvelles données.

## Exigences

### Exigences fonctionnelles

- **FR-083-001** : L'artefact HTML DOIT être un type de publication Bridget et
  utiliser le modèle de version, provenance, cache et projet de SPEC-082.
- **FR-083-002** : Bridget DOIT exécuter un artefact HTML dans une isolation
  stricte qui interdit par défaut réseau, fichiers, secrets, API locales,
  privilèges de l'application, stockage de navigation, sortie de cadre et
  navigation automatique.
- **FR-083-003** : Un artefact HTML DOIT recevoir uniquement les données
  déclarées dans son manifeste et ne peut obtenir d'information additionnelle
  qu'à travers une action Bridget explicitement autorisée.
- **FR-083-004** : Bridget DOIT afficher un artefact HTML inline jusqu'à 1 200
  px de hauteur et proposer une action explicite pour l'ouvrir développé dans
  l'espace droit.
- **FR-083-005** : Une interaction dans un artefact HTML DOIT rester temporaire
  jusqu'à l'action explicite de création d'une nouvelle version.
- **FR-083-006** : Bridget DOIT conserver le code HTML, ses données, paramètres
  autorisés, provenance et manifeste sous forme consultable, copiable et
  exportable, sans l'exécuter hors de son isolation.
- **FR-083-007** : Bridget Desktop DOIT proposer un panneau latéral droit
  affichable, masquable et redimensionnable, sans palette de commandes dans ce
  lot.
- **FR-083-008** : Le panneau droit DOIT proposer initialement les onglets
  Browser, Artefacts, Fichiers, Liens et Activité. Ces onglets ne deviennent pas
  un accès général aux fichiers ni aux données d'agent.
- **FR-083-009** : L'onglet Browser DOIT permettre à l'opérateur de naviguer
  volontairement vers des contenus publiés, des pages locales autorisées et des
  pages Internet HTTPS, dans un profil isolé de ses navigateurs personnels.
- **FR-083-010** : Le navigateur droit DOIT conserver son propre état selon la
  politique locale, permettre son effacement explicite et ne jamais exposer ses
  cookies, identifiants, historique ou contenu privé aux agents ou artefacts.
- **FR-083-011** : Tout lien issu du Markdown, d'un artefact ou d'une source
  DOIT rester inactif jusqu'au geste de l'opérateur et s'ouvrir via Bridget dans
  le Browser, pas par navigation directe de la conversation ou de la sandbox.
- **FR-083-012** : Toute collecte d'image, de donnée ou de page distante DOIT
  être exécutée et journalisée par Bridget. La sandbox et le renderer de
  conversation n'ont aucun accès réseau direct.
- **FR-083-013** : Les résultats d'une collecte externe DOIVENT conserver les
  octets ou le manifeste de provenance nécessaires à leur restitution, selon les
  règles de conservation de SPEC-082.
- **FR-083-014** : Un contenu indisponible, refusé ou non restituable DOIT
  présenter sa cause et ses actions de récupération plutôt qu'un placeholder muet.
- **FR-083-015** : Les réglages de sécurité et de conservation applicables
  doivent rester dans les paramètres globaux de l'application avec une portée
  visible Ce Mac, et ne doivent pas être transmis comme réglages aux agents ou
  projets. La récupération et l'affichage automatique de contenu externe sont
  désactivés par défaut ; après activation explicite, ils passent toujours par
  Bridget et jamais par le renderer de conversation, l'iframe ou un navigateur.
- **FR-083-016** : Les actions d'automatisation de navigateur, de sélection de
  DOM, de modification de page et de soumission de formulaire par agent sont
  explicitement exclues de cette version.

### Exigences non fonctionnelles

- **Sécurité** : l'isolement doit appliquer le moindre privilège, une politique
  de contenu restrictive et une séparation nette entre application, navigateur,
  panneau relayé et artefact non fiable.
- **Accessibilité** : l'ouverture, le masquage, les onglets, l'agrandissement,
  l'export et les erreurs sont utilisables au clavier, annoncent leur état et
  respectent les alternatives fournies par SPEC-082.
- **Performance** : l'artefact inline ne bloque pas le défilement du fil ; le
  contenu développé et le navigateur sont chargés à la demande ; le panneau peut
  être masqué sans détruire son état nécessaire.
- **Confidentialité** : le profil du navigateur est séparé ; la navigation ne
  devient jamais un canal implicite de collecte agent ou de transfert de secret.
- **Conformité de licence** : toute dépendance locale de rendu ou de table doit
  être identifiée, auditée et accompagnée de sa licence avant livraison.

## Entités clés

- **Artefact HTML sandboxé** : version d'artefact contenant code, données et
  manifeste, exécutée sous des capacités minimales.
- **Contexte de sandbox** : environnement temporaire isolé qui expose seulement
  les données et interactions locales déclarées.
- **Browser Bridget** : espace de navigation contrôlé par l'opérateur, doté de
  son profil séparé et sans privilège ou lecture agent.
- **Destination validée** : contenu publié, page locale autorisée ou URL HTTPS
  passée à Bridget après un geste explicite.
- **État interactif temporaire** : variation locale d'un artefact non inscrite
  dans l'historique avant la création d'une nouvelle version.

## Critères de succès

- **SC-083-001** : 100 % des recettes d'isolement vérifient qu'un artefact ne
  peut ni joindre le réseau, ni accéder à un fichier, ni appeler une capacité
  locale, ni naviguer hors de son cadre.
- **SC-083-002** : dans 100 % des essais d'interaction, aucune modification
  temporaire ne change une version publiée sans l'action explicite de création
  d'une nouvelle version.
- **SC-083-003** : 100 % des artefacts HTML dépassant 1 200 px restent lisibles
  inline et s'ouvrent développé seulement après action de l'opérateur.
- **SC-083-004** : 100 % des liens de recette restent sans accès réseau avant le
  geste d'ouverture et passent ensuite par le Browser Bridget validé.
- **SC-083-005** : les onglets du panneau droit sont utilisables au clavier,
  conservant le contexte de la conversation et sans révéler de données de
  navigateur aux agents dans 100 % des scénarios de test.
- **SC-083-006** : l'opérateur peut effacer le profil de navigation sans perdre
  un artefact canonique, sa provenance ou son historique de versions dans 100 %
  des essais contrôlés.

## Hypothèses et dépendances

- SPEC-082 définit l'identité, le manifeste, la version, la provenance, le
  stockage canonique, la rétention et la recherche d'un artefact.
- SPEC-074 établit déjà l'isolation entre coque native et panneaux relayés. Le
  navigateur et les artefacts ne doivent pas affaiblir cette frontière.
- SPEC-080 porte les réglages locaux de l'application. Les nouvelles options y
  sont ajoutées avec une portée explicite et sans synchronisation vers serveur.
- SPEC-081 reste l'autorité du Markdown et interdit qu'une URL Markdown crée
  seule une navigation ou une collecte.
- L'opérateur valide la présence d'un navigateur latéral normal pour son usage
  direct, mais l'automatisation de navigateur par agent est hors scope.
- L'affichage inline maximal validé pour HTML est de 1 200 px ; les artefacts
  natifs conservent une hauteur naturelle selon SPEC-082.
