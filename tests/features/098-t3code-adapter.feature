# language: fr
Fonctionnalité: Joindre les fils t3code par Bridget
  Chaque fil ouvert dans t3code est un agent Bridget visible, destinataire de
  messages qui apparaissent dans son fil, et dont la réponse revient à l'émetteur ;
  t3code n'est jamais modifié.

  Scénario: Installer et retirer sans toucher t3code
    Étant donné t3code démarré et la commande "t3" disponible
    Quand l'humain lance "bridget t3 install" puis "bridget t3 uninstall"
    Alors un jeton dédié est obtenu par la commande officielle puis révoqué
    Et aucun fichier de t3code n'a été modifié

  Scénario: Un fil t3code est un agent de l'annuaire
    Étant donné un fil ouvert dans t3code
    Quand l'humain consulte "bridget who"
    Alors le fil apparaît avec le canal "t3code"
    Et il disparaît quand le fil est archivé

  Scénario: Un fil porte le titre de t3code comme nom
    Étant donné un fil t3code intitulé "Alpha"
    Quand le pont l'expose
    Alors "bridget who" le montre sous le nom "Alpha"
    Et un titre déjà pris laisse le nom attribué par le daemon

  Scénario: Un message arrive dans le fil et la réponse revient
    Étant donné un fil t3code sans tour actif
    Quand un agent Bridget lui envoie une demande suivie
    Alors un tour démarre dans le fil avec le message
    Et la réponse du tour de même rang que ce message clôt la demande

  Scénario: Un humain intercalé ne détourne pas la réponse
    Étant donné un fil t3code dont l'humain vient de démarrer un tour
    Quand un agent Bridget lui envoie une demande suivie
    Alors la remise attend la fin du tour humain
    Et la réponse liée est celle du tour de Bridget, pas celle de l'humain

  Scénario: Un tour sans réponse ne fait pas attribuer celle du voisin
    Étant donné un fil où le tour de Bridget s'est interrompu sans texte
    Et un tour humain qui a produit une réponse
    Quand le pont relit le fil au repos
    Alors la demande de Bridget reste sans réponse et l'ambiguïté est journalisée
    Et la réponse de l'humain ne lui est jamais attribuée

  Scénario: Le pont ne change pas la politique du fil
    Étant donné un fil réglé sur un mode d'exécution donné
    Quand ce mode change entre deux sondages puis qu'un message est remis
    Alors le tour démarre avec le mode enregistré au moment de la remise

  Scénario: Un 401 isolé est renouvelé une seule fois
    Étant donné une session t3 refusée une fois par le serveur
    Quand le pont rejoue l'appel
    Alors une nouvelle session est émise et l'ancienne révoquée
    Et une session refusée deux fois de suite met le pont en échec explicite

  Scénario: Un tour actif fait attendre puis échoue nommément
    Étant donné un fil t3code avec un tour en cours
    Quand une demande lui est envoyée et que la borne d'attente expire
    Alors la remise est indéterminée et nommée
    Et un rejeu aux mêmes paramètres ne crée pas de second tour

  Scénario: Journal sans rejeu ni doublon
    Étant donné un fil créé et achevé pendant un arrêt du pont
    Quand le pont redémarre
    Alors les tours postérieurs à l'installation sont journalisés une seule fois, dans l'ordre
