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
devices_manage seul et toute portée partielle ne donnent pas ce droit.
La permission actuelle specialists_manage conserve ses règles propres.

Le serveur ne réécrit ni appareil, clé ni portée persistante. Il vérifie toujours
l'état courant, l'expiration et la révocation. Le changement de mandat exige la
réponse humaine native à la question exacte et une révision inchangée. Aucun
accord d'agent, champ JSON libre ou copie d'une autre décision n'est accepté.

Le refus d'un appareil réellement limité conserve la décision, sa clé et
l'opération ; un nouveau clic ne remplace pas une autorisation manquante.
Aucun redémarrage d'une application cliente n'est requis pour cette résolution.
