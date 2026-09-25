# language: fr
Fonctionnalité: Administrer une fédération depuis le binaire Bridget
  Le binaire distribué embarque l'unique gestionnaire de fédération afin que
  l'humain n'ait aucun chemin de dépôt ou de script à connaître.

  Scénario: Réutiliser sans mutation une liaison connue par son nom DNS
    Étant donné la liaison attestée "poste-beta-core" enregistrée avec l'adresse IP de Poste-beta
    Quand l'humain lance "bridget federate ssh://exemple.test -p 2222"
    Alors la liaison existante est reconnue sans second service ni réinstallation

  Scénario: Refuser une destination ambiguë
    Étant donné deux liaisons attestées correspondant à la même destination
    Quand aucun label n'est précisé
    Alors Bridget refuse de choisir arbitrairement et demande "--label"

  Scénario: Diagnostiquer une nouvelle liaison non interactive
    Étant donné une destination qui ne correspond à aucune liaison
    Quand l'entrée ou la sortie n'est pas un terminal et que des paramètres manquent
    Alors Bridget liste les options manquantes sans lancer SSH ni gestionnaire natif

  Scénario: Installer depuis le seul binaire
    Étant donné une nouvelle destination et tous les paramètres de liaison
    Quand l'humain lance la commande federate
    Alors le script embarqué installe un runner autonome avec les gardes de la session 095

  Scénario: Observer toutes les liaisons sans réseau
    Étant donné plusieurs installations attestées
    Quand l'humain lance "bridget federate status"
    Alors leur état natif est affiché sans résolution DNS ni commande SSH

  Scénario: Retirer seulement une destination non ambiguë
    Étant donné une installation attestée correspondant à la destination demandée
    Quand l'humain lance "bridget federate remove" avec cette destination
    Alors le retrait 095 est exécuté et toute erreur native reste un échec

  Scénario: Exécuter le binaire loin du dépôt source
    Étant donné une copie isolée du binaire sans script adjacent
    Quand l'humain demande l'aide federate
    Alors l'aide est rendue et la copie privée temporaire du script est nettoyée
