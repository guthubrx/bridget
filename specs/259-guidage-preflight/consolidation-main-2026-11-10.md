# Consolidation sur main — 10 novembre 2026

François a demandé de ne conserver que main, de réunir et pousser les travaux, de retirer les branches et worktrees secondaires et de libérer les builds reproductibles, sans perdre les sources ni les preuves. Cette décision figure maintenant dans AGENTS.md et prévaut sur les anciennes obligations de worktree.

## Sources et historiques préservés

- Les décisions 260 du commit 2316a179 sont fusionnées par 8366988f ; la reprise des lignes de base 4c832f2a est fusionnée par bc94f2c9.
- Le brouillon Cartae bfe9f01a est conservé dans l'ascendance de main par 7a360e7d. Son état reste consultable par son commit.
- Les trois différences non commitées des anciens checkouts SYS sont conservées intégralement dans les archives compressées du commit d1c54b4e. Leur empreinte et leur application au commit de base sont vérifiées : trois reconstructions PASS. Taille compressée totale : 47720 octets.
- Sept anciennes sauvegardes stash, dont C05 f79fddc6, sont conservées avec leurs parents d'index et de fichiers non suivis dans l'ascendance de main. Leurs références sont dans stashes.json. Leurs entrées stash et les références transitoires d'import ont été retirées après push vérifié. Les fichiers courants gardent leur version active ; les anciens états restent reconstruisibles par leur SHA.

Les répertoires d'archives sont dans /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/259-guidage-preflight/archives-worktrees.

## Branches et worktrees

Les quatre dépôts principaux, Mac, SYS, Cartae et KS, ne gardent que main. Aucun worktree annexe ne reste inscrit. Huit anciens worktrees distants ont été retirés : quatre SYS, trois Cartae et un KS. Les deux incarnations successives du dernier worktree Mac 260 ont aussi été retirées, après intégration de leurs commits et conservation des fichiers supplémentaires.

Trente-trois branches SYS intégrées, la branche d'archive Cartae et les deux branches Mac 260b/260c ont été retirées. Les références distantes obsolètes ont été purgées. Aucun reset destructif ni push forcé n'a été employé. Les tags de versions et de compatibilité restent utiles à la publication et au retour arrière.

## Preuves locales et attente des tests

Le premier journal ignoré de 454273 octets a été déplacé dans le checkout principal, avec empreinte vérifiée : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/260-assainissement-code/validation/base-tests.txt.

Le fil 260 a recréé son worktree et lancé plusieurs passages de tests pendant le nettoyage. François a choisi deux fois d'attendre leur fin. Aucun de ces builds n'a été interrompu. Le contrôle de retrait a attendu la fermeture des fichiers. Sept fichiers supplémentaires sont conservés sous /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/260-assainissement-code/validation/reprise-nettoyage-20261110, avec noms, tailles et SHA-256 dans manifest.json. Les journaux bruts et les métadonnées Finder sont exclus du dépôt public par le fichier .gitignore local à ce dossier.

Les sources suivies du worktree étaient intégrées au moment de son retrait. Le fil 260 a ensuite poursuivi ses travaux directement sur main, conformément à la nouvelle règle. Ses nouvelles modifications en cours et ses compilations actives appartiennent à ce travail ; elles ne sont pas effacées par ce nettoyage. La consolidation ne constitue pas une déclaration d'achèvement de la spécification 260 ni de réussite générale de sa campagne de tests.

## Espace et contrôles

La purge native avec budget nul a retiré 9,06 Gio d'intermédiaires Mac et 5,54 Gio SYS, soit 14,60 Gio logiques. Vingt-quatre copies ou éléments de builds Mac inutilisés ont été retirés en plus : 189824405 octets. Le passage précédent avait déjà retiré quinze images Mac et une copie APK, soit 555768300 octets logiques. Les tailles logiques ne mesurent pas l'espace physique libéré sur APFS.

Le cache unique réutilisé par le travail 260 reste protégé par ses verrous lorsqu'une compilation est active ; son dernier relevé donne 3,16 Gio. Il ne s'agit pas d'un ancien worktree ni d'une copie créée pour une revue. Aucun cache actif n'a été forcé. Les exécutables installés 259, le retour 258 et les preuves utiles sont conservés.

Les sept services Bridget contrôlés restent chargés avec leurs empreintes 259. Le nettoyage et les fusions d'archives n'ont pas changé leur activation. La décision D1 de la session 260 demande des sources seules pour ses nouveaux lots. Aucun redémarrage de l'application T3, rejeu métier ou arrêt d'un agent n'a été effectué.

Deux index Git réécrits root pendant l'inspection administrative ont été rendus à leur propriétaire utilisateur, sans changement de contenu. Les inspections suivantes désactivent les verrous optionnels et perdent les droits administratifs avant les mutations Git. Sur SYS, cinq services système non dumpables ont été identifiés par nom et filiation ; aucun agent ou compilateur masqué n'a été accepté comme exemption.

Les contrôles propres à cette consolidation portent sur les empreintes, les reconstructions d'archives, l'ascendance Git, les références reçues à distance, l'inventaire réel des worktrees/branches/stashes, les processus et les verrous. Aucun nouveau test applicatif ou modèle n'a été lancé par le nettoyage.

Reçus détaillés : /Users/moi/.cache/bridget259-guidage-preflight. Repères : main-only-mac-retired.json, main-only-reconstruction.json, main-only-build-copies.json, main-only-second-log.json et les reçus distants main-only-retirement.json.
