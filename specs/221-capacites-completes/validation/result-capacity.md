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

## Chargement vérifié —31octobre2026
Source exécutable5eaeae07d25d7d4107d47905d58c163724ba9a77
(compte/options618888c1 inclus). Build221-capacity-accounts-r1.
LinuxSHA256632c4f2d389cdb5792baf354812c73917803f898bde5ea1dc5eebd4e500362e3 ;
MacSHA2562c19c5c0292eb8b040373d10815dd7db86cd1ca278bd6bac9175f24d083999ca.
MaîtreCartae2861459/mobile2861514/worker2862763 ; SYS1064810 ; KS1463106 ;
workerMac99292/pont99547. Images chargées vérifiées, coupuremaître2,20s.
SuperviseurCartae2500061/birth270585460 conservé ; aucun service Opus2D touché.
Retour : image distincte de récupération au même code nouveau, formats contrôlés
dans les deux sens. Ce n'est pas un retour vers l'ancien16Mio, incompatible.
Pointeurs rollbackCartae/Mac écrits atomiquement sous verrou natif ; outils
de mise à jour repris. Les anciens exécutables encore chargés restent conservés.

Une attestation réussie n'a pas suffi au premier appel : primary_absent pendant
le retour du pont. La lecture suivante du catalogue a réussi45options/gel=false ;
aucun nouveau restart ou admission pour rétablir le lien. MCP de ce filPID146
utilise la nouvelle image ; d'autres MCP encore anciens sont conservés sous usage.
Lecture comptes en production réussieCartae/SYS/KS avec le registre worker explicite ;
auth reste non vérifiée. Le défaut de registreCLI implicite surSYS/KS ne concerne
pas leurs workers configurés par --registry-file ; --registry absolu est requis.
Aucun compte réel créé/connecté/modifié.

Signalement monuments : message de résultat bridget221-capacite-retours-resultat-20261031-01,
delivery4ebe43d3-816a-46db-998e-5703ebbe9e2a, in_flight sans preuve de lecture.
La tâcheba4ae9e3 et sa clé restent conservées, aucun rejeu ni remplacement.
Les deux autres propriétaires ont reçu l'avis de reprise, aussi in_flight.

installedMac/SYS a refusé le propriétaire du staging enregistré ; caches
conservés, aucune réécriture de marqueur ni purge forcée. Budget/observabilité
ne sont pas contournés pour le rangement. Programme52/182InProgress ;
pas de clôtureC19 ou de parité globale. Pas de campagne de tests générale.
