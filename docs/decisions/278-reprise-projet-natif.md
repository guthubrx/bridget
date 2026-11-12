# 278 : reprendre la preuve de projet au retour du worker

Statut : activé et vérifié en fonctionnement sur la flotte, le 12 novembre 2026.

## Contexte

Une annonce de projet reçue pendant une coupure du worker restait sans projet
après son retour. Quatre connexions natives vivantes ont montré ce défaut après
le passage 277. Le wrapper ne réannonçait pas son contexte après ce seul refus.

## Décision

Conserver en mémoire un fait par connexion native authentifiée lorsque la preuve
échoue avec origin_worker_unavailable ou origin_proof_busy. Un heartbeat du worker
actuel déclenche un seul lot asynchrone de lectures. Le canal de réponses et le
verrou global restent libres pendant ces lectures. Une capacité encore occupée
arrête le lot jusqu'au prochain heartbeat.

Chaque annonce porte une révision monotone. La reprise vérifie avant et après
la preuve la connexion, le propriétaire, l'instance, la commande et cette révision.
Une nouvelle annonce, une annulation ou une déconnexion invalide le fait ancien.
Le retrait d'une connexion purge le fait, y compris par Unregister. Les textes
retenus sont bornés à 4096 octets de racine et 255 octets d'hôte.

## Conséquences

Aucun stockage durable ni changement du protocole. Aucun prompt, lancement de
modèle ou message métier n'est rejoué. Les droits restent ceux de la commande
native déjà admise. Les annonces sans autorité ou contradictoires ne sont pas
reprises. Après un redémarrage du daemon, les wrappers réannoncent normalement
leur contexte. Le tri par ancienneté coûte O(n log n), avec n connexions en attente ;
un seul lot par worker limite les lectures concurrentes.

Le test de régression simule capacité occupée, heartbeat d'une autre machine,
réannonce identique pendant la preuve, annulation et retrait public de connexion.
Il vérifie que les octets de la tâche restent identiques lors de la récupération.
