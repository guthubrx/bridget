# Administration mobile historique complète

Décision du 7 novembre 2026, correction demandée par François après approbation
Android enregistrée et refus specialists_admin_scope_required.

Les anciennes associations --all accordaient tous les projets, toutes les
machines et exactement read, message, task_read, task_continue, task_cancel,
task_respond, devices_manage. L'administration des spécialistes a été ajoutée
ensuite ; son opération explicite n'existait pas dans cette association globale.

Cette portée historique complète reste une administration globale. La reconnaître
pour les spécialistes exige les deux indicateurs globaux, des listes de machines
et projets vides et exactement les sept opérations, sans manque ni doublon.
L'association doit aussi avoir été créée avant l'introduction des spécialistes
(1 novembre 2026, Unix 1793532131, commit dadca5d0). Une nouvelle association
personnalisée qui omet specialists_manage ne reçoit donc jamais ce droit.
devices_manage seul et toute portée partielle ne donnent pas ce droit.
La permission actuelle specialists_manage conserve ses règles propres.

Il s'agit d'une extension de compatibilité explicite à tous les appareils qui
respectent ces conditions, pas d'un simple changement d'affichage. L'inventaire
réel relève deux associations Android actives de cette forme, créées le
24 octobre, dont celle qui a enregistré la décision de François. Aucun autre
appareil n'est concerné. Les métadonnées exactes restent dans le dossier privé
/Users/moi/.cache/bridget239-maintenance-transversale.

La liste des opérations stockées reste historique ; le commentaire du contrôle
et ce document indiquent le droit effectif ajouté. Une nouvelle association
avec l'opération explicite sort de cette compatibilité. Révoquer ou laisser
expirer l'ancienne association retire toujours son autorisation ; un renouvellement
conserve sa date initiale et ne transforme pas une association récente en ancienne.

Le serveur ne réécrit ni appareil, clé ni portée persistante. Il vérifie toujours
l'état courant, l'expiration et la révocation. Le changement de mandat exige la
réponse humaine native à la question exacte et une révision inchangée. Aucun
accord d'agent, champ JSON libre ou copie d'une autre décision n'est accepté.

Le refus d'un appareil réellement limité conserve la décision, sa clé et
l'opération ; un nouveau clic ne remplace pas une autorisation manquante.
Aucun redémarrage d'une application cliente n'est requis pour cette résolution.

Les décisions de mandat gardent leur projection par projets quand l'autorisation
spécialistes est explicite. Les mutations directes mobiles conservent leur garde
préexistante de tous les projets. Le correctif n'élargit pas cette seconde voie
aux appareils administrateurs limités à certains projets.

La date de création repose sur l'horloge du maître. Un administrateur de cet
hôte peut la modifier ; cet accès dépasse le modèle de menace d'un appareil
mobile. L'inventaire des appareils éligibles est relu avant activation.
Lors d'un remplacement ou d'une réassociation choisie par l'utilisateur,
utiliser l'opération explicite actuelle. Aucun nouvel appairage n'est imposé
pour chaque approbation ou pour terminer la décision déjà enregistrée.
