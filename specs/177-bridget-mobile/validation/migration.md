# Migration 177 — preuves et limites

Statut : In Progress. Aucune migration ou relance de production effectuée.

## Origine isolée sur deux machines, 24 octobre

Le coordinateur de fixture tourne sur Cartae. Le worker d'origine tourne sur le Mac. La fixture HTTP du fournisseur et le dépôt Git sont privés et n'existent que sur le Mac. Aucun fournisseur de modèle ni aucune Task n'est créé.

Le test natif mobile177_origin_live_two_hosts_no_task_or_provider_launch passe : attestation version 2, identité de fil attendue, politique validée, base Git réelle et refus d'un autre fil. Les trois réponses traversent la connexion du worker négociée et corrélée à son époque. Le magasin des Tasks reste vide.

Le transport SSH direct-streamlocal a d'abord été refusé par la règle PermitOpen existante. Ce n'était pas un refus du protocole. Un écho brut reproduit le refus. La fixture corrigée utilise une socket Unix privée locale et les flux stdin/stdout d'une commande SSH authentifiée vers la socket Unix privée distante. Aucun port TCP n'est exposé ; la configuration SSH reste inchangée. Cette fixture ne vaut pas installation du raccordement permanent.

Contrôles ciblés actuels : mobile177, 53 PASS et 1 recette isolée explicitement ignorée par défaut ; cette recette est ensuite jouée explicitement, 1 PASS en 0,78 s. Régressions worker : 60 PASS avant l'ajout des dernières lectures Git et signature locale. Pas de suite globale ni de bascule déduite.

## Préconditions encore ouvertes

- La préparation d'un dépôt doit être exécutée sur sa machine d'origine avec le helper existant et ses reçus. Le coordinateur doit relire le registre publié avant la sonde.
- Un candidat installé sur le coordinateur peut utiliser le worker local lorsque les sources appartiennent à une autre machine. La résidence du maître ne doit pas faire lire un chemin Mac sur Linux.
- Préparer la migration hors ligne des données, le retour arrière, les services et les raccordements avant toute activation.
- Prouver le fonctionnement du téléphone et des agents serveur lorsque le Mac est hors ligne.

## Préparation réelle et contre-revue

La recette isolée r7 ajoute la préparation réelle du dépôt source Mac vers le dossier géré Cartae et son rejeu strict : 1 PASS en 2,02 s. Le HEAD et le fichier témoin restent identiques ; aucun fichier non suivi, aucune Task et aucun lancement fournisseur. SSH, Git et le worker sont réels ; les métadonnées d’installation du lien appartiennent à la fixture et ne prouvent pas un service permanent.

La contre-revue indépendante Claude (Task ff277e40-e1f3-4d2b-be46-61ead4abc71f, résultat lu et acquitté) accepte avec corrections. Les quatre remarques sont appliquées : absence de lien refusée, marge de 5 s sous le plafond worker, échéance de 595 s pour tout le passage et relecture finale de la machine d’origine. Lot mobile177 après corrections : 54 PASS, 1 recette physique ignorée par défaut, 0 échec, 5,82 s. La recette physique reste celle de r7 avant ces gardes supplémentaires ; pas de nouvelle activation déduite.

La configuration Android publique Firebase du projet bridget-mobile est créée et conservée hors Git. Build Android réussi avec cette configuration ; stockage et gardes sur le Pixel : 13 PASS. Aucun compte serveur FCM installé et aucune notification réelle encore démontrée.

## Raccordement permanent — première recette réelle

Le runner natif est étendu par une direction to-master. Une liaison launchd isolée sur le Mac joint une socket privée sur Cartae par SSH réel, sans nouveau port ni changement SSHD. Aller-retour de 40 octets exact, SHA-256 identique côté Mac et Cartae. Socket cliente 0600. Le service de fixture est ensuite retiré par la commande native ; zéro processus runner/SSH de cette fixture restant. Ce test ne déplace ni le maître ni une mission.

Régressions initiales : scripts095 23 PASS sans skip ; scripts096 12 PASS ; scripts089 31 PASS. Contre-revue transport reçue et lue : quoting et autorité sans défaut bloquant, deux remarques moyennes. La sonde est corrigée : stream-run attend le premier octet avant SSH, donc une connexion de contrôle vide ne contacte pas le maître. La garde locale précède désormais le préflight distant. Une direction absente hérite intentionnellement de la liaison déjà installée ; elle est maintenant affichée. Une direction explicite divergente reste refusée. Cela conserve la reprise sans réinstallation, au lieu de remplacer silencieusement le sens d’un lien existant.

La borne de 1024 canaux évite l’absence de limite par défaut de socat. Elle porte sur les connexions natives, pas sur les worktrees ou les missions. Chaque canal a sa propre session SSH et ses buffers de flux bornés par le système ; le coût à grande échelle n’est pas mesuré. Aucun partage de session SSH ni changement de droits n’est ajouté. Le dimensionnement à forte concurrence reste une limite déclarée, pas une preuve de charge.

## Seconde bascule réelle — r6, 24 octobre 2026

Le maître optimisé r6 est actif sur Cartae. Le maître Mac est arrêté et désactivé ; T3 reste chargé. La copie hors ligne récente contient 360 Tasks et 431 reçus. Toutes les tables applicatives ont le même nombre de lignes et le même SHA-256 de lignes avant et après copie, avant démarrage. quick_check est ok et foreign_key_check ne donne aucune erreur. Les tâches distantes conservent leurs placements ; les processus métier du worker n’ont pas été tués. Les anciens fichiers de cible sont conservés dans une sauvegarde privée distincte. Aucun maître n’a été actif simultanément.

Les services utilisateur core, mobile et worker Cartae sont actifs et persistants, linger=yes. Le worker d’origine Mac et le relais to-master sont actifs. Le statut MCP réel de ce fil retourne cartae, build r6. Le catalogue réel retourne dix profils, dont huit autorisés, avec verdict_context.machine_id=monordinateur et cwd_root Mac exact. Cela vérifie la lecture d’origine et la réattestation du parent après déplacement du coordinateur ; ce n’est pas un lancement de mission. Le premier appel pendant la reconnexion a donné primary_absent ; après reconnexion native du pont, les mêmes lectures passent.

L’archive privée de transfert a le SHA-256 9eee369f96bff0c3ce79ce9e653c75f811dc81e9b42644c1c37ec7ae3eb0f5ec. Les clés et fichiers de machine restent sur leur origine. La paire de binaires r6 n’est pas encore une release publiée. La recette téléphone et l’indépendance physique du Mac restent ouvertes.
