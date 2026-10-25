# Archives techniques de l’adaptateur T3

Document d’origine : [specs/145-bridget-thread-viewer/spec.md](../../../../../../specs/145-bridget-thread-viewer/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

L'utilisateur veut lire les échanges partagés entre agents dans la surface native droite de T3. L'onglet « Bridget » montre les fils auxquels appartient l'agent T3 sélectionné. Il permet de lire plusieurs fils, leurs membres et leur historique exact. Une consultation humaine ne doit pas lancer un tour de modèle ni modifier la conversation des agents.

<a id="extrait-2"></a>

## Extrait 2

Le panneau appartient à T3. Il suit ses règles d'ouverture, de fermeture, de taille et de navigation. Son icône `b` utilise le blanc ou la teinte neutre des autres onglets. Elle n'utilise pas de bleu spécifique. La liste montre un titre humain quand il existe. Elle affiche les membres, les auteurs, les dates et les types réels de message. Un identifiant attesté remplace un nom absent. Aucun résumé généré ne remplace le contenu.

<a id="extrait-3"></a>

## Extrait 3

Documents : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/`.
Code Bridget : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/`, base `3bb89e0d`.
Code T3 : `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/`, base `a1a4f2ef12`.

<a id="extrait-4"></a>

## Extrait 4

Comme utilisateur, je veux ouvrir Bridget dans le panneau droit de T3 afin de voir les fils partagés de l'agent actuellement sélectionné.

<a id="extrait-5"></a>

## Extrait 5

Test indépendant : préparer deux agents membres de fils différents, puis changer de conversation T3. La liste reflète seulement l'appartenance attestée de chaque agent. Une réponse tardive du premier contexte ne s'affiche pas dans le second.

<a id="extrait-6"></a>

## Extrait 6

1. Étant donné une liaison T3 attestée et plusieurs fils accessibles, quand j'ouvre Bridget, alors leurs titres et membres apparaissent dans le panneau natif.
2. Étant donné un agent sans fil, quand la lecture se termine, alors un état vide explique qu'aucun fil accessible n'a été trouvé.
3. Étant donné une identité absente ou ambiguë, quand j'ouvre le panneau, alors une erreur explicite remplace les données ; aucun accès élargi ne sert de repli.
4. Étant donné un changement de conversation, quand une ancienne lecture se termine, alors ses résultats sont ignorés et la nouvelle identité est vérifiée côté serveur.

<a id="extrait-7"></a>

## Extrait 7

Comme utilisateur, je veux retrouver les commandes et états habituels de T3 afin de consulter Bridget à la souris ou au clavier.

<a id="extrait-8"></a>

## Extrait 8

- FR145-01 : ajouter « Bridget » comme surface native du panneau droit T3. Réutiliser la sélection et les interactions de ses onglets ; respecter leurs états ouvert, fermé et inactif.
- FR145-02 : utiliser une icône `b` neutre, blanche ou atténuée selon les styles T3. Ne pas introduire une couleur bleue propre à Bridget.
- FR145-03 : limiter la liste aux fils dont l'agent associé à la conversation T3 sélectionnée est membre. Vérifier cette appartenance pour chaque liste, détail et page d'historique.
- FR145-04 : résoudre cette identité côté serveur à partir de la liaison T3 attestée. Une identité envoyée par le navigateur, un nom libre ou une conversation non liée ne suffit jamais à autoriser une lecture.
- FR145-05 : proposer une voie humaine distincte de lecture. Son vocabulaire fermé comprend seulement liste, détail et historique. Elle conserve les règles d'appartenance du stockage existant.
- FR145-06 : ne provoquer aucune émission, aucun accusé de réception, aucune avance de curseur agent, aucun réveil, aucune modification de mission et aucun jeton de modèle lors d'une ouverture, fermeture, navigation, recherche, pagination ou actualisation humaine.
- FR145-07 : n'exposer au navigateur aucun secret, credential, enveloppe de transport ni état de curseur agent. Projeter seulement les champs nécessaires à la consultation.
- FR145-08 : afficher les titres humains et les membres réellement disponibles. Utiliser un identifiant stable attesté si un nom manque. Ne pas déduire de nom, d'intention ni de livraison.
- FR145-09 : montrer l'auteur, la date et le type réel de chaque message, avec le corps complet. Préserver espaces, Unicode, lignes et ponctuation ; la copie rend le corps original exact.
- FR145-10 : permettre de sélectionner plusieurs fils successivement. Isoler leur historique et leur chargement par contexte et par identifiant de fil.
- FR145-11 : borner chaque lecture de liste et d'historique. Un contrôle explicite charge la page suivante. Rejeter les limites invalides ; documenter taille, ordre et position de page dans le contrat.
- FR145-12 : paginer de manière stable sur les identifiants du stockage. Sur des données stables, les pages contiguës n'ont ni trou ni doublon. La position de page humaine ne modifie aucun curseur agent.
- FR145-13 : effectuer la recherche dans les seules données chargées. Indiquer cette portée. Ne pas lancer de recherche distante ni d'appel modèle à chaque frappe.
- FR145-14 : offrir un rafraîchissement manuel. Aucune boucle de lecture périodique n'est ajoutée. Les requêtes en cours devenues obsolètes sont annulées quand possible et leurs résultats sont ignorés dans tous les cas.
- FR145-15 : vider les données sensibles du contexte quitté ou refusé. Une réponse tardive, un changement de conversation, une fermeture ou une révocation ne doit jamais restaurer des données d'un contexte ancien.
- FR145-16 : couvrir les états chargement, liste vide, fil vide, recherche vide, erreur, daemon absent, liaison absente ou ambiguë, refus d'accès, délai dépassé et version incompatible. Une erreur conserve un moyen clair de réessayer.
- FR145-17 : authentifier les opérations dans la voie RPC T3 existante. Exécuter la CLI Bridget par arguments séparés, sans interpréteur de commandes ni interpolation de contenu utilisateur.
- FR145-18 : ne pas détourner les requêtes réservées aux agents ni leur rôle IPC. Ne pas ressusciter l'ancien `ui.rs` supprimé. Le code T3 ne possède actuellement aucune API Bridget réutilisable.
- FR145-19 : réutiliser les primitives natives T3, son service Effect, son contrat RPC, ses contrôles et son rendu approprié. Aucune obligation de Next.js, Axios ou dépendance nouvelle n'est introduite.
- FR145-20 : rendre les onglets, la liste, la pagination, la recherche, le rafraîchissement et la fermeture utilisables au clavier. Fournir des noms accessibles, une sélection perceptible, un focus visible et des états de chargement annoncés sans déplacement forcé du focus.
- FR145-21 : prouver la non-mutation avec des tests qui comparent les données métier avant et après les lectures et refus. Tester aussi les contextes obsolètes, l'appartenance, la pagination et les interactions réelles.
- FR145-22 : conserver les autres panneaux T3, messages et fonctions agent. Ne modifier ni le daemon actif, ni les LaunchAgents, ni les missions pendant le développement isolé.

<a id="extrait-9"></a>

## Extrait 9

- SC145-01 : les fixtures multiagents montrent uniquement les fils autorisés ; des identifiants de fil ou de conversation forgés sont refusés côté serveur.
- SC145-02 : trois pages ou plus d'un historique stable restituent chaque message une fois, dans l'ordre documenté. Les limites hors bornes sont refusées.
- SC145-03 : l'affichage et la copie reproduisent les corps tests exacts, y compris lignes blanches, Unicode, blocs de code et texte non Markdown.
- SC145-04 : un changement rapide A → B, une fermeture et une révocation invalident les réponses A tardives. Le panneau n'affiche aucune donnée du contexte quitté.
- SC145-05 : les lectures humaines et leurs refus laissent identiques les curseurs agents, les missions et les files d'émission. Les spies de génération, réveil et ACK reçoivent zéro appel.
- SC145-06 : la recherche ne produit aucun appel distant. Le rafraîchissement produit seulement les lectures autorisées et ne déclenche pas de boucle.
- SC145-07 : les états indisponible, incompatible, vide et refusé sont testés. Le clavier peut ouvrir, sélectionner, paginer, rafraîchir et fermer la surface native.
- SC145-08 : tests ciblés Bridget et T3, contrôles de type, lint et build applicables passent sur les worktrees isolés. Aucun résultat d'installation ou d'exécution active n'est revendiqué sans preuve séparée.

<a id="extrait-10"></a>

## Extrait 10

L'appartenance à un fil est la règle d'autorisation. Le rattachement réel T3 est une preuve, pas un libellé d'affichage. La consultation humaine relève du rôle client distinct. Les méthodes agent existantes refusent ce rôle ; ce refus ne doit pas être supprimé globalement.
