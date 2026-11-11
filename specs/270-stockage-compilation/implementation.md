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
032ad58b examine le raccord final ; aucune nouvelle mission métier créée.

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

Activation du réglage et compilation réelle à compléter après publication des
scripts. Les exécutables Bridget et T3 ne nécessitent aucun redémarrage.
