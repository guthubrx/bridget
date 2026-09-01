Fonctionnalité: Runtime Docker de production par projet

  Scénario: Le défaut serveur reste Host sans réglage explicite
    Étant donné une installation Bridget existante sans réglage runtime
    Quand un projet est créé ou importé
    Alors son backend reste Host et aucune image Docker n'est créée

  Scénario: L'activation Docker est atomique
    Étant donné un projet Host et une politique Docker attestée
    Quand l'opérateur confirme activate_docker avec la génération attendue
    Alors le binding Docker est publié seulement après préparation et attestation du conteneur

  Scénario: Une activation échouée conserve Host
    Étant donné un projet Host et un preflight Docker en échec
    Quand l'opérateur demande activate_docker
    Alors le binding Host initial reste inchangé

  Scénario: Une opération destructive refuse un agent actif
    Étant donné un projet Docker avec un agent actif
    Quand l'opérateur demande stop, remove, recreate ou switch_to_host
    Alors le daemon refuse sans modifier le conteneur ni le binding

  Scénario: Les secrets ne deviennent jamais des options Docker libres
    Étant donné un profil projet contenant des références de secrets attestées
    Quand le daemon prépare le conteneur
    Alors les valeurs ne figurent ni dans la commande, ni dans les logs, ni dans le contrat UI

  Scénario: Les linked worktrees gardent leurs chemins absolus
    Étant donné un dépôt avec son checkout principal et deux linked worktrees
    Quand le runtime Docker est préparé
    Alors chaque checkout est monté au même chemin absolu et aucun volume nommé n'est créé

  Scénario: Un redémarrage réconcilie sans dupliquer
    Étant donné une transition Docker interrompue avant publication
    Quand le daemon redémarre
    Alors il réconcilie le binding et le conteneur attesté sans en créer un second
