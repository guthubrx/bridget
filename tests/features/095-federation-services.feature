# language: fr
Fonctionnalité: Fédération SSH permanente autonome
  Le script distribué administre un tunnel SSH persistant sans dépendre du
  dépôt historique, sans créer un second daemon Bridget et sans interpréter sa
  configuration comme du code.

  Scénario: Installer puis retirer un service launchd possédé
    Étant donné une socket maître privée et des fichiers SSH privés
    Quand l'humain installe la liaison sur macOS
    Alors le runner autonome et sa configuration privée sont copiés sous un chemin stable
    Et le LaunchAgent utilise ce runner sans référencer le worktree
    Et la cadence de relance est bornée
    Et status rapporte l'état launchd sans modifier la socket
    Et remove arrête le service avant de supprimer uniquement les fichiers attestés

  Scénario: Installer puis retirer un service systemd utilisateur
    Étant donné une socket maître privée et des fichiers SSH privés
    Quand l'humain installe la liaison sur Linux
    Alors l'unité systemd utilisateur lance le runner autonome
    Et RestartSec et StartLimit bornent la cadence de relance
    Et le statut et le retrait passent par systemctl --user
    Et la persistance après déconnexion est annoncée comme dépendante du linger

  Scénario: Refuser une installation en collision
    Étant donné un fichier de service étranger ou un chemin symbolique
    Quand l'humain demande install
    Alors l'installation échoue sans écraser le fichier
    Et aucun service n'est activé

  Scénario: Refuser le retrait d'une installation altérée
    Étant donné une installation dont le runner, la configuration ou l'unité ne correspond plus au reçu privé
    Quand l'humain demande remove
    Alors le retrait échoue avant tout arrêt ou effacement
    Et les clés SSH et les données Bridget restent intactes

  Scénario: Récupérer une socket distante périmée attestée
    Étant donné un service installé et une racine distante privée
    Et une socket distante privée possédée par le compte
    Et une connexion Unix refusée avec ECONNREFUSED
    Quand le runner supervisé reprend le tunnel
    Alors il revalide type propriétaire mode périphérique et inode
    Et il supprime seulement cette socket périmée
    Et le bind SSH reste protégé par StreamLocalBindUnlink=no et ExitOnForwardFailure=yes

  Scénario: Refuser une socket vivante ou ambiguë
    Étant donné une socket distante vivante ou une sonde qui expire
    Quand le runner supervisé tente une reprise
    Alors la socket reste intacte
    Et le tunnel n'est pas lancé
    Et status ne supprime jamais de socket

  Scénario: Préserver le comportement run de la session 089
    Étant donné une socket distante périmée
    Quand run est appelé directement hors installation supervisée
    Alors il refuse la socket sans l'effacer

