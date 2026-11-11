# 270 : compilation et rangement hors du disque système

Les cinq refus remontés par Cerveau sont antérieurs à 269 et corrélés à
quota_exceeded, plafond 24. Ils sont annulés et cleanup_done=true, sans instance,
Start ni remise de mission. Les worktrees restent conservés. Les quatre steer
Claude rejoignent le défaut livré en 269 ; les reçus historiques ne sont pas
rejoués. La revue 8db01550 reçue à nouveau est un doublon déjà traité.

SYS dispose initialement d’environ 29,6 Gio sur `/`. Le volume storage est monté
sur un autre système de fichiers et offre plus de 6 To disponibles ; data offre
plus de 7 To. Aucun processus cargo/rustc/rustup n’est actif. Les toolchains
1.92 et 1.98 sont conservées. La toolchain par défaut est 1.98, le dépôt épingle
1.92. Le changement de stockage ne change pas ces choix.

Le lot ajoute la configuration portable décrite par l’ADR 270. 84 contrôles
du nettoyage et 44 de l’installateur passent sur Mac et Debian, avec les ignorés
de plateforme existants pour le nettoyeur (3 sur Mac, 2 sur Debian).
La première exécution Linux du paquet de test isolé manquait du Makefile ; le
contexte du test est complété, sans patch produit. La revue 04e70286 relève le
FIFO et le blocage global du ménage si le volume manque. Les deux cas sont
corrigés et couverts. La revue est lue, ACK et garde propriétaire close.
Les points supplémentaires sur l’ancre résiduelle et le changement de volume
avant/après Cargo sont également couverts. Le cache partagé est un choix explicite,
compatible avec la pratique existante et avec des enfants séparés si nécessaire.

Le lecteur administrateur natif a d’abord refusé un descripteur transitoire du
worker SYS. La lecture réelle a ensuite constaté ce descripteur absent. Une nouvelle
observation complète réussit, sans retirer de garde ni arrêter le worker. Le
rangement conserve ses rapports et les empreintes de copie dans les preuves
privées, sans contenu de secret dans les arguments ou les journaux.

La contre-revue ba63377c trouve le Makefile qui exportait encore target local.
Le nouveau résolveur readonly est raccordé à Make, RELEASE_BIN, installed et
l’installateur source. Un test par le vrai Makefile vérifie installation depuis
un chemin externe avec espaces et nettoyage local après perte du volume.
L’installateur couvre aussi le repli des anciens paquets sans configuration et
le refus avant Cargo quand une configuration existe mais n’est pas résolue.
Les deux revues sont lues, ACK et gardes propriétaires closes. Une revue ciblée
032ad58b ne trouve aucun défaut nouveau dans le raccord final. Revue statique,
sans tests exécutés par le relecteur ; résultat lu, ACK et garde propriétaire
close. Aucun nouvel agent métier créé. L’avertissement Make hors compilation
reste volontaire et non bloquant, couvert par le test clean-builds.

Le déplacement termine 291 éléments : 243 archives fermées, les deux dossiers
Rust et 46 fichiers de familles SQLite temporaires mortes. 7 630 166 455 octets
logiques sont copiés et comparés. Les anciens chemins sont des liens vers les
copies vérifiées ; les 291 anciennes copies ont été retirées après une nouvelle
observation native complète. Rustup et Cargo fonctionnent depuis data : 1.98
par défaut, 1.92 dans le dépôt. Trois anciennes images inutilisées sont retirées
après observation administrateur complète ; 268 et 269 sont conservées.
Le disque système passe de 31 772 737 536 à 40 673 009 664 octets disponibles,
soit environ 8,3 Gio gagnés lors du rangement (variations des autres travaux
possibles). Les données actives et l’outil de mise à jour ne sont pas déplacés.

Les scripts sont poussés en 6639b901b102d645c7f28814c67a856449b46c62 et chargés
depuis les checkouts propres et alignés des quatre machines. Le réglage privé SYS
est créé avec une racine sur storage, modes 0700/0600 et ancre concordante.
Une compilation Rust réelle d’un petit programme autonome passe par le vrai
Makefile release et le vrai script, sans dépendance ni réseau. Seul le manifeste
du programme de recette est fourni à Cargo par BRIDGET_CARGO, crochet existant.
Le binaire de recette porte un nom distinct de bridget et ne peut pas remplacer
son résultat de compilation. Il s’exécute avec le marqueur attendu. L’ABI exige
glibc 2.34, sous le plafond 2.36. La cible et le disque système ont des identités
de systèmes de fichiers distinctes (2065 et 2052). Le target local reste inchangé.
Cette preuve qualifie le stockage, pas une nouvelle compilation du produit.

Le rangement final retire aussi 17 anciens fichiers de compilation, soit
1 111 188 019 octets logiques, dans les trois staging enregistrés 227/264/269.
Les verrous de profils sont acquis sans attente, les lecteurs observés par le
lecteur natif administrateur, les fichiers vérifiés et leurs empreintes gardées.
Les petits marqueurs, verrous et dossiers vides restent en place ; aucun verrou
Cargo n’est supprimé. La copie de sources de test SYS est retirée après contrôle
d’usage. Le cache externe récent de recette et ses sources sont conservés comme
preuve, moins de 1 Mio, hors disque système.

Mesure finale : 41 830 858 752 octets disponibles sur `/`, environ 39 Gio, soit
9,4 Gio de plus qu’au début de ce passage. Les variations des autres travaux
peuvent modifier cette valeur après mesure. Aucun processus métier n’est arrêté.
Les exécutables Bridget restent en 269 ; T3 conserve son PID et son heure de
lancement. Aucun redémarrage nécessaire pour les scripts Python et Make.

Preuves privées : inventaires, journal des 291 copies et comparaisons, empreintes
des fichiers retirés, trois revues, journaux de tests et compilation. Le dossier
de preuves local occupe moins de 1 Mio. La présente correction est technique,
sans changement d’interface : aucun diaporama requis.

Information propriétaire tentée une seule fois en intent=information. Reçu
mcp-86189-6af4b210-2f : unknown_recipient, aucune route active pour le responsable
du rangement à cet instant. La liste des fils partagés est vide. Bilan et reçu
conservés, sans repli vers queue ni déduction de lecture. Les vues des revues
restent des preuves conservées ; leurs gardes sont closes, sans retrait forcé.
