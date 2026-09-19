Fonctionnalité: Centre de contrôle Bridget

  Scénario: Chaque panneau relié ouvre les réglages de son propre serveur
    Étant donné un panneau Bridget relié à un serveur enregistré
    Quand l'opérateur active l'engrenage en bas de la barre des agents
    Alors le centre affiche les catégories déclarées par ce serveur uniquement
    Et aucune configuration d'un autre serveur n'est présentée comme locale

  Scénario: Une préférence du Mac ne traverse pas le tunnel
    Étant donné que l'opérateur modifie son thème, son fuseau ou sa police
    Quand il enregistre les préférences Bridget Desktop
    Alors le document local est mis à jour atomiquement
    Et aucune requête de réglage serveur n'est produite

  Scénario: La première écriture serveur est contrôlée
    Étant donné une politique de racines de projets à la génération 1
    Quand l'opérateur prévisualise une liste de racines valide puis la confirme localement
    Alors le serveur écrit la génération 2 et retourne son identifiant de commande
    Et un rejeu immédiat du même identifiant ne crée pas une génération 3

  Scénario: Les coûts inconnus ne sont pas inventés
    Étant donné des échantillons d'usage sans grille tarifaire datée
    Quand l'opérateur ouvre l'onglet Usage
    Alors Bridget présente les jetons attestés par fournisseur et modèle quand ils existent
    Et affiche que l'estimation API est indisponible plutôt qu'un montant nul

  Scénario: La maintenance reste informative
    Étant donné qu'aucune source de mise à jour vérifiée n'est configurée
    Quand Bridget affiche l'état de version
    Alors l'état est non configuré
    Et aucune installation, aucun redémarrage et aucune commande d'hôte ne sont déclenchés
