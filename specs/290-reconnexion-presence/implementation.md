# Reconnexion après expiration de présence

Incident : mcp-85319-6af7541a-11, le 13 novembre 2026. Trois liaisons de rôle Parix sont absentes du routeur après une longue veille, alors que les fils restent ouverts. Le suivi est actif. Une autre liaison du même projet répond normalement. Le maître chargé est le build 288-preparation-guidage-r1.

## Cause reproduite

`agent_infos` retire les présences dont le dernier battement dépasse 300 secondes. `release_router_for_dangling_instances` retire ensuite l'identité et ses droits, mais conserve la socket. Un battement sur cette socket ne recrée pas l'identité. Le connecteur peut donc garder son lecteur vivant sans recevoir d'ordre de réenregistrement.

Ce défaut est reproduit par une vraie paire de sockets Unix. Avant correction, après purge puis battement, la socket continue de répondre à la sonde d'identité du démon. L'événement exact de purge des trois liaisons de production n'est pas journalisé : la correspondance historique reste une inférence, distincte de la reproduction.

## Correction limitée

Le lecteur de connexion mémorise si un Register primaire réussi a identifié une liaison T3. Si son prochain battement arrive après retrait de son entrée dans `conn_instances`, il ferme cette seule socket, hors du verrou d'état. Le connecteur reprend alors son parcours existant de réenregistrement. Une présence simplement en retard, encore enregistrée, reste ouverte : son battement la rafraîchit. Ce booléen ne porte aucun droit et ne restaure aucune présence.

Le traitement ne touche ni les battements des workers, ni ceux des clients de service, ni les wrappers natifs. Il n'envoie pas Disconnect aux fournisseurs. Aucun état durable de tâche, aucune permission, aucun identifiant de demande n'est réécrit.

## Qualification et activation

Le test ciblé échoue avant le correctif et réussit après sur une copie de qualification de main, sous Linux. Sa version finale est également exécutée après les corrections de revue et réussit. Elle couvre six cas sur socket réelle : présence purgée, retard sans purge et lien sain, pour T3 et pour un wrapper natif. Il vérifie aussi le rafraîchissement après retard sans purge et réutilise la même identité lors des nouveaux enregistrements.

La base du runtime cumulatif diffère de main. Les trois insertions de production sont portées séparément sur une copie de ce runtime, avec le même test. Dix tests ciblés réussissent sur cette copie Linux après revue : le nouveau test socket, cinq tests de battement, trois tests de reprise de nom et un test des contrôles T3. Clippy strict sur tous les targets du paquet bridget-daemon réussit également après les corrections de revue. Aucun build n'est lancé dans le checkout principal ; les copies de qualification réutilisent le cache prévu via scripts/build.py.

La revue indépendante b733a3c3-01c0-4739-be43-0d8b01bc7b83 a trouvé deux défauts dans la première proposition. Le critère initial fermait aussi une présence en retard non purgée : il est remplacé par l'absence de `conn_instances`, avec un cas de test dédié. L'écriture fragmentée du test pouvait échouer pendant la fermeture : les trames sont maintenant sérialisées avant l'envoi et seule la lecture prouve la fermeture. Le calcul d'horloge utilise `checked_sub` et le délai de lecture dispose d'un message explicite. La garde des connexions auxiliaires est vérifiée par lecture, sans nouveau test socket dédié. La revue est acquittée et sa garde fermée ; sa vue reste `kept/review_required` et n'est pas retirée manuellement.

Le contrôle rustfmt des deux fichiers entiers échoue sur des blocs préexistants. Le même échec est confirmé sur HEAD avant ce correctif. Les ajouts du lot ne figurent pas dans les différences de format ; aucun reformatage général n’est inclus.

L'activation n'est pas effectuée. Le parcours natif de mise à jour refuse les missions actives ou non confirmées. La lecture du maître à observed_at 1794595818 relève notamment deux tâches working, quatre starting et une queued. Ce constat, rapproché de la garde de maintenance, bloque l'activation ; aucun appel Prepare n'a été émis et aucun refus d'exécution n'est donc prétendu. Le parcours ponctuel de redémarrage du connecteur reste suspendu depuis l'incident a716ff89. Aucun arrêt T3, aucun redémarrage de service et aucune mission de remplacement n'ont été exécutés. Les trois liaisons restent unobserved sans connexion lors de la dernière lecture native ; le maître exécute toujours 288-preparation-guidage-r1. La livraison est incomplète tant que cette activation et le retour réel des liens ne sont pas vérifiés.

Preuves locales : /Users/moi/.cache/bridget290-reconnexion.
Qualification Linux : /mnt/data/moi/bridget290-reconnexion.

Ménage du dépôt : le rapport du helper commun ne trouve aucun worktree ni branche à retirer. Les preuves locales occupent 1,7 Mio. La copie de qualification de main sur SYS est retirée après fin du test et contrôle des usages dans /proc : 127 Mio. La copie du runtime cumulatif (126 Mio) reste nécessaire à la préparation de l'activation. La vue native de revue reste sous conservation Bridget ; son reçu ne permet pas de la supprimer manuellement.
