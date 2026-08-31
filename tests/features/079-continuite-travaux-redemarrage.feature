# language: fr
Fonctionnalité: Continuité durable et ronde pilotée par projet

  Scénario: Reprendre une demande humaine après redémarrage
    Étant donné une demande humaine admise et acquittée par un agent géré
    Quand le daemon et le provider redémarrent avant la fin du tour
    Alors une continuation reconstruite reçoit exactement la demande une seule fois

  Scénario: Ne pas rejouer une simple reconnexion
    Étant donné un wrapper qui se réenregistre avec un tour en cours
    Quand Bridget réconcilie ses exécutions actives
    Alors aucune continuation et aucune nouvelle remise ne sont créées

  Scénario: Conserver la frontière projet
    Étant donné une exécution rattachée à un projet et une génération
    Quand Bridget reconstruit cette exécution
    Alors le descendant conserve exactement la même référence projet

  Scénario: Activer une ronde pour un seul projet
    Étant donné deux projets actifs dont seul le premier a une ronde activée
    Quand le scheduler exécute une occurrence deux fois
    Alors le premier projet reçoit un seul réveil et le second aucun

  Scénario: Désactiver sans interrompre
    Étant donné un projet avec une ronde activée et un travail en cours
    Quand l'opérateur désactive la ronde
    Alors le travail reste actif et les occurrences futures sont ignorées
