# language: fr
Fonctionnalité: Observer un agent depuis une conversation T3 sans configuration manuelle

  Scénario: Attribuer l'appel au bon fil malgré un dossier commun et un sous-agent
    Étant donné deux fils T3 dans le même dossier et un sous-agent Codex plus récent
    Quand la conversation appelle Bridget
    Alors seule l'identité du fil attesté possède l'abonnement

  Scénario: Refuser une association ambiguë
    Étant donné plusieurs fils correspondant au même processus fournisseur
    Quand Bridget tente de rattacher son identité
    Alors aucune identité n'est devinée ni marqueur ancien conservé

  Scénario: Recevoir une seule fin de tour future
    Étant donné un abonnement ponctuel à Atelier-3D
    Quand une fin de tour attestée est reçue puis répétée
    Alors le destinataire reçoit une seule notification sans déclaration de succès métier

  Scénario: Ne pas notifier l'historique ou une notification
    Étant donné un pont découvrant un historique puis un tour provoqué par une notification
    Quand ces éléments sont journalisés
    Alors aucune boucle de notifications n'est produite

  Scénario: Distinguer une vraie écriture d'une lecture ou d'un échec
    Étant donné des activités structurées de permission, écriture réussie, lecture et écriture échouée
    Quand le pont les reçoit
    Alors seuls la permission et le fait d'écriture réussi deviennent observables

  Scénario: Refuser une surveillance impossible
    Étant donné une source inexistante ou incompatible
    Quand une conversation tente de s'abonner à ses événements
    Alors Bridget refuse avec une cause explicite

  Scénario: Signaler la perte de surveillance
    Étant donné un abonnement actif
    Quand la source disparaît ou que le daemon redémarre
    Alors l'état devient indisponible ou interrompu sans continuité fictive
    Et le propriétaire est averti lorsque sa disponibilité permet la remise

  Scénario: Conserver les fonctions autonomes de Bridget
    Étant donné des sources compatibles sans T3
    Quand un extrait est partagé et deux agents écrivent le même fichier
    Alors le destinataire reçoit l'extrait sourcé et un avertissement de collision sans verrou
