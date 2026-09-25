# language: fr
Fonctionnalité: Piloter la ronde de vigilance par projet
  Afin de contrôler les réveils périodiques sans agir sur les travaux
  En tant qu'opérateur Bridget
  Je veux activer ou désactiver la ronde depuis le menu du projet

  Scénario: Activer puis désactiver une ronde confirmée
    Étant donné un projet actif dont la ronde est désactivée
    Quand l'opérateur active la ronde depuis le menu contextuel
    Alors la ligne indique la ronde activée seulement après confirmation de Bridget
    Et le prochain passage est annoncé dans un délai maximal de sept minutes
    Quand l'opérateur désactive ensuite la ronde
    Alors aucun agent ni travail accepté n'est arrêté ou annulé

  Scénario: Refuser une ancienne génération après reconnexion
    Étant donné un menu ouvert sur une génération de projet
    Et le projet reconnecté sur une nouvelle génération
    Quand l'opérateur demande de modifier la ronde depuis l'ancien menu
    Alors Bridget refuse la génération obsolète
    Et la dernière valeur confirmée reste affichée

  Scénario: Expliquer un projet inactif et le dernier passage
    Étant donné un projet retiré avec un dernier passage connu
    Quand l'opérateur ouvre son menu contextuel
    Alors l'action de ronde est indisponible avec une cause lisible
    Et le dernier passage conserve son résultat fermé
