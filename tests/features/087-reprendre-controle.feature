Fonctionnalité: Reprendre le contrôle du coordinateur

  Scénario: La pause différe une routine et une ronde
    Étant donné une routine dont l'occurrence est due et une ronde active sur un projet
    Quand le référent active la pause
    Alors l'occurrence est consignée différée avec le motif pause
    Et la ronde n'émet aucun réveil et son refus est consigné

  Scénario: La pause interrompt un tour commencé
    Étant donné un agent en plein tour
    Quand la pause est activée
    Alors le tour reçoit une interruption et son exécution est mémorisée

  Scénario: La levée de la pause reprend le tour interrompu
    Étant donné un tour interrompu par la pause et un agent connecté libre
    Quand le référent lève la pause
    Alors l'agent reçoit la reprise de la même exécution

  Scénario: La pause survit au redémarrage
    Étant donné la pause active
    Quand le service Bridget redémarre
    Alors la pause est toujours active et affichée

  Scénario: La pause ne bride pas le référent
    Étant donné la pause active
    Quand le référent ouvre un focus
    Alors le focus est accepté

  Scénario: Un agent ne peut pas lever la pause
    Étant donné la pause active
    Quand un agent tente de la lever par ses outils déclarés
    Alors la demande est refusée avec un motif explicite et le refus est consigné

  Scénario: Une phrase ouvre un focus d'origine humaine
    Étant donné un projet actif avec deux agents disponibles
    Quand le référent soumet « Travaille sur X » pour ce projet
    Alors un objectif est ouvert avec une origine humaine attestée liée au message
    Et il est marqué focus
    Et une délégation est remise à un agent du projet avec sa base gelée calculée par le système
    Et l'instruction contient les identifiants de dépôt sans saisie du référent

  Scénario: Le focus différe le travail automatique
    Étant donné un focus ouvert
    Quand une routine veut ouvrir du travail auto-généré
    Alors ce travail est différé avec le motif focus
    Et il repart quand le focus se ferme

  Scénario: Un seul focus à la fois
    Étant donné un focus ouvert
    Quand un second focus est demandé sans conduite de conflit
    Alors le système demande de choisir entre remplacer et mettre en file

  Scénario: Une origine humaine déclarée par un agent est refusée
    Étant donné un agent agissant par ses outils déclarés
    Quand il dépose une délégation avec une origine humaine ou un focus
    Alors le dépôt est refusé
    Et aucun objectif n'est ouvert

  Scénario: Une reprise de l'identité humaine est signalée
    Étant donné l'interface connectée sous l'identité humaine
    Quand une autre connexion s'enregistre sous cette identité
    Alors un item de boîte human_route_replaced est déposé
    Et il est poussé sur le canal externe

  Scénario: Une chaîne épuisée arrive dans la boîte
    Étant donné une délégation dont la chaîne de réassignation est épuisée
    Quand le coordinateur l'enregistre
    Alors un item intervention requise apparaît dans la boîte avec l'objectif, la délégation, l'historique et les décisions possibles

  Scénario: Le canal externe injoignable ne perd rien
    Étant donné un canal externe injoignable
    Quand un item est créé
    Alors l'item existe dans la boîte
    Et l'échec de notification est consigné

  Scénario: Seul le référent tranche
    Étant donné un item ouvert
    Quand un agent tente de le traiter
    Alors la tentative est refusée

  Scénario: Un même événement ne produit qu'un item
    Étant donné un item ouvert pour une clé
    Quand le même événement est déposé une seconde fois
    Alors la boîte ne contient qu'un item et sa seconde occurrence lui est rattachée

  Scénario: Une décision relue jusqu'à confirmation
    Étant donné une décision prise par le référent
    Quand le coordinateur plante entre la relève et l'application
    Alors la décision est relue à la relève suivante
    Et elle est appliquée une seule fois puis acquittée

  Scénario: Le plafond différe une ouverture automatique
    Étant donné un plafond de un et un objectif auto-généré ouvert
    Quand une routine veut en ouvrir un autre
    Alors l'ouverture est refusée
    Et l'occurrence est différée avec le motif budget
    Et un item budget atteint est déposé une seule fois

  Scénario: Le plafond ne compte pas le travail humain
    Étant donné le plafond atteint
    Quand le référent ouvre un focus
    Alors le focus est accepté

  Scénario: Les quatre faits sont visibles
    Étant donné la pause active, un focus ouvert et un item ouvert
    Quand le référent ouvre l'interface ou lance l'annuaire
    Alors la pause avec sa durée, le focus, le nombre d'items et le budget consommé sont affichés
