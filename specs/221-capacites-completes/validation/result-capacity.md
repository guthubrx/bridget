# Incident — réservation du résultat
Tâche ba4ae9e3-375f-44d8-866d-0655f0efe53e, clé
opus201-dix-paralleles-03-20261031-01, SYS.
État source : correction en cours, aucune activation revendiquée.

## Fait établi
MAX_WORKER_STORE_BYTES=16Mio ; chaque retour actif réserve2Mio + métadonnées.
Le test worker162 attendait explicitement un refus à la huitième réservation.
Dix missions ne peuvent donc pas réserver simultanément dix retours sur ce budget.
Lecture SYS postérieure :366commandes toutes terminales,428slots tous fermés,
7499546octets de fichiers. Le pic de réservation exact à l'instant du refus
n'a pas été conservé ; ne pas déduire sa valeur depuis cette lecture ultérieure.
Le disque SYS avait31Gio libres, /mnt/storage6,4Tio ; ne pas attribuer ce code
à une saturation du volume ou à une limite de compte fournisseur.

## Correction
Budget worker partagé256Mio, enveloppe2Mio et corps256Kio inchangés.
Test ciblé64commandes+64slots puis quota par orphelin. Historiques conservés,
réservations inchangées, pas de Start ou de retry ajouté.
Le diagnostic indique budget/borne et occupation non observée, portée tâche,
geste utile et décision humaine précise. Aucun chiffre de concurrence garanti :
la capacité dépend aussi des reçus, vues, entrées, disque et exécution.
Un retour compatible doit comprendre worker.store.capacity.v2 avant activation.
La tâche échouée n'est ni rejouée ni remplacée ; autres missions préservées.

## Vérification ciblée
Cinq contrôles PASS en10,15s :64commandes/64slots, historique120résultats, budget partagé des vues, diagnostic/rejeu sans nouvelle tâche et refus du retour16Mio. Aucune mission fournisseur ou Opus2D lancée par ces contrôles. Log /home/moi/.cache/bridget221-provider-lot/capacity-targeted-r1.log.

## Revue indépendante ciblée
6edffb00-8366-4d97-9390-81155a9c273a : pas de P1/P2 confirmé sur le budget.
Le doute sur la commande de slot65 est réfuté par le helper reserve initial
qui réserve command() : le contrôle passe bien sur Quota, confirmé par le test.
L'ancien plafond sept est lui aussi fixé par l'oracle162 avant correction.
Le champ de budget reste explicitement celui du correctif, pas une mesure du
worker ou du pic historique. L'occupation actuelle n'est pas inventée.
L'objection sur resume concerne une décision de produit : commande explicite
d'activation de la liaison enregistrée, aucune invocation automatique ; le geste explicite autorise ce démarrage.
Voir contrat machine-preparation. Aucun nouveau test ou audit global.
