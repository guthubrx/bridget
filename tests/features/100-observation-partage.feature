# language: fr
Fonctionnalité: Observation et partage inter-agents sans blocage
  Scénario: Partager un extrait sourcé
    Étant donné un journal Unicode fragmenté de 60 entrées dans une instance isolée
    Quand un agent partage les 50 dernières entrées avec un autre agent
    Alors le destinataire reçoit un extrait ordonné et sourcé
    Et aucune instruction n'est exécutée par le lecteur de journal

  Scénario: Limite et lacune visibles
    Étant donné un journal dépassant la borne ou contenant une lacune
    Quand un extrait est demandé
    Alors il signale son incomplétude et son point de reprise

  Scénario: Une seule notification de fin de tour
    Étant donné un abonnement ponctuel à la fin de tour d'Alpha
    Quand Alpha termine un tour puis en termine un autre
    Alors une seule notification arrive sans prétendre que la mission a réussi

  Scénario: Propriété et expiration
    Étant donné un abonnement appartenant à Alpha
    Quand Bêta tente de le supprimer
    Alors la suppression est refusée
    Et son expiration empêche ensuite toute notification

  Scénario: Risque de modification concurrente
    Étant donné deux agents écrivant le même chemin structuré sur le même hôte
    Quand leurs écritures sont observées à moins de 30 secondes d'écart
    Alors un abonné reçoit une alerte nommant les deux agents et le fichier
    Et aucun agent n'est bloqué

  Plan du scénario: Ne pas inventer une collision
    Étant donné deux observations avec <différence>
    Quand les observations sont traitées
    Alors aucune collision n'est signalée
    Exemples:
      | différence             |
      | le même auteur         |
      | deux chemins distincts |
      | deux hôtes distincts   |
      | plus de 30 secondes    |
      | une lecture simple     |
