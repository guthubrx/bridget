# SPEC145 — Consultation native des fils Bridget dans T3

Date : 2026-10-07. Statut : Implemented ; 20/20 tâches vérifiées. Non installé, non activé. Branche : `session-145-bridget-thread-viewer`.

## Besoin et périmètre

L'utilisateur veut lire les échanges partagés entre agents dans la surface native droite de T3. L'onglet « Bridget » montre les fils auxquels appartient l'agent T3 sélectionné. Il permet de lire plusieurs fils, leurs membres et leur historique exact. Une consultation humaine ne doit pas lancer un tour de modèle ni modifier la conversation des agents.

Le panneau appartient à T3. Il suit ses règles d'ouverture, de fermeture, de taille et de navigation. Son icône `b` utilise le blanc ou la teinte neutre des autres onglets. Elle n'utilise pas de bleu spécifique. La liste montre un titre humain quand il existe. Elle affiche les membres, les auteurs, les dates et les types réels de message. Un identifiant attesté remplace un nom absent. Aucun résumé généré ne remplace le contenu.

L'utilisateur a autorisé la session145 et le travail isolé. La dernière session livrée est143. Les répertoires144 ne prouvent aucun développement. Les travaux142 et les processus actifs restent hors périmètre. Aucun commit, fusion, push, installation, déploiement ou redémarrage n'est autorisé par cette spécification.

Documents : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/specs/145-bridget-thread-viewer/`.
Code Bridget : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/145-bridget-thread-viewer/`, base `3bb89e0d`.
Code T3 : `/Users/moi/11.Repositories/t3code-local/.worktrees/145-bridget-thread-viewer/`, base `a1a4f2ef12`.

## Histoires utilisateur

### US1 — Trouver les fils de l'agent sélectionné (P1)

Comme utilisateur, je veux ouvrir Bridget dans le panneau droit de T3 afin de voir les fils partagés de l'agent actuellement sélectionné.

Test indépendant : préparer deux agents membres de fils différents, puis changer de conversation T3. La liste reflète seulement l'appartenance attestée de chaque agent. Une réponse tardive du premier contexte ne s'affiche pas dans le second.

1. Étant donné une liaison T3 attestée et plusieurs fils accessibles, quand j'ouvre Bridget, alors leurs titres et membres apparaissent dans le panneau natif.
2. Étant donné un agent sans fil, quand la lecture se termine, alors un état vide explique qu'aucun fil accessible n'a été trouvé.
3. Étant donné une identité absente ou ambiguë, quand j'ouvre le panneau, alors une erreur explicite remplace les données ; aucun accès élargi ne sert de repli.
4. Étant donné un changement de conversation, quand une ancienne lecture se termine, alors ses résultats sont ignorés et la nouvelle identité est vérifiée côté serveur.

### US2 — Lire le contenu exact de plusieurs fils (P1)

Comme utilisateur, je veux sélectionner un fil et parcourir ses messages afin de comprendre l'échange réel.

Test indépendant : ouvrir deux fils avec des auteurs distincts, des types différents, du texte multiligne et un historique supérieur à une page. Chaque page conserve son ordre. Le texte et la copie correspondent à la source. Le changement de fil n'affiche pas l'historique du précédent.

1. Étant donné un fil accessible, quand je le sélectionne, alors son titre, ses membres et une première page bornée de messages apparaissent.
2. Étant donné un historique plus long, quand je demande la page suivante, alors une page bornée s'ajoute sans doublon ni trou dans un jeu de données stable.
3. Étant donné un message, quand je le lis ou le copie, alors son auteur, sa date, son type et son corps exact sont disponibles. Les noms absents ne sont pas inventés.
4. Étant donné une révocation d'appartenance, quand une nouvelle lecture a lieu, alors l'accès est refusé et les données désormais non autorisées disparaissent du panneau.

### US3 — Chercher et rafraîchir sans agir sur les agents (P2)

Comme utilisateur, je veux filtrer les données chargées et les rafraîchir manuellement afin de retrouver un échange sans réveiller un agent.

Test indépendant : chercher un mot présent dans une page chargée, vider la recherche, puis rafraîchir. Observer les appels et l'état Bridget avant et après. Seules les lectures humaines prévues ont lieu. Les curseurs agents et les missions restent identiques.

1. Étant donné des données chargées, quand je saisis une recherche, alors seuls ces titres, membres, auteurs et corps chargés sont filtrés localement ; la portée est indiquée.
2. Étant donné une recherche sans résultat, quand elle est appliquée, alors un état sans résultat permet de l'effacer.
3. Étant donné le bouton Rafraîchir, quand je l'utilise, alors la liste et le fil courant sont relus selon l'identité attestée actuelle, sans doublon ni émission.
4. Étant donné le panneau fermé, quand j'attends ou change de conversation, alors aucune lecture périodique ni génération de modèle n'est déclenchée par ce panneau.

### US4 — Utiliser un panneau natif fiable et accessible (P2)

Comme utilisateur, je veux retrouver les commandes et états habituels de T3 afin de consulter Bridget à la souris ou au clavier.

Test indépendant : ouvrir l'onglet, parcourir les fils, charger une page, rafraîchir et fermer au clavier. Simuler un daemon absent, une réponse incompatible et un délai dépassé. Chaque situation affiche un état utile sans modifier les agents.

## Exigences fonctionnelles

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

## Critères de succès

- SC145-01 : les fixtures multiagents montrent uniquement les fils autorisés ; des identifiants de fil ou de conversation forgés sont refusés côté serveur.
- SC145-02 : trois pages ou plus d'un historique stable restituent chaque message une fois, dans l'ordre documenté. Les limites hors bornes sont refusées.
- SC145-03 : l'affichage et la copie reproduisent les corps tests exacts, y compris lignes blanches, Unicode, blocs de code et texte non Markdown.
- SC145-04 : un changement rapide A → B, une fermeture et une révocation invalident les réponses A tardives. Le panneau n'affiche aucune donnée du contexte quitté.
- SC145-05 : les lectures humaines et leurs refus laissent identiques les curseurs agents, les missions et les files d'émission. Les spies de génération, réveil et ACK reçoivent zéro appel.
- SC145-06 : la recherche ne produit aucun appel distant. Le rafraîchissement produit seulement les lectures autorisées et ne déclenche pas de boucle.
- SC145-07 : les états indisponible, incompatible, vide et refusé sont testés. Le clavier peut ouvrir, sélectionner, paginer, rafraîchir et fermer la surface native.
- SC145-08 : tests ciblés Bridget et T3, contrôles de type, lint et build applicables passent sur les worktrees isolés. Aucun résultat d'installation ou d'exécution active n'est revendiqué sans preuve séparée.

## Limites et hypothèses

L'appartenance à un fil est la règle d'autorisation. Le rattachement réel T3 est une preuve, pas un libellé d'affichage. La consultation humaine relève du rôle client distinct. Les méthodes agent existantes refusent ce rôle ; ce refus ne doit pas être supprimé globalement.

La recherche porte sur les pages chargées. La session n'ajoute ni composition de message, ni ACK humain, ni abonnement, ni notification, ni action de mission. La vue ne revendique pas une lecture humaine de l'ensemble d'un fil tant que toutes ses pages ne sont pas chargées.

Les assistants et modèles ne reçoivent aucun nouveau contexte par l'ouverture de cette surface. L'absence de consommation de jetons vise les appels de modèle provoqués par la consultation ; elle ne prétend pas mesurer le coût de processus indépendants déjà actifs.

Les helpers et templates SpecKit attendus sont absents du worktree et du runtime utilisateur inspectés. Les artefacts sont produits par application documentaire du protocole lu par le principal. Aucun helper exécuté ni synchronisation installée n'est prétendu.
