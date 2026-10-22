# Préparation des worktrees liés — 22 octobre 2026

## Cause observée

La demande opus158-156-parcs-publications-candidates-cartae-20261022-01
reçoit routing_candidate_unavailable avec dix refus mapping_unavailable,
avant toute Task. Aucun reçu pour cette clé dans le store maître.

Source : /Users/moi/Nextcloud/10.Scripts/69.opus2D/.worktrees/158-integration-continuite256
Destination attendue : /home/moi/10.Scripts/69.opus2D/.worktrees/158-integration-continuite256
Commit demandé : 37485568b9d94e22ab20db52a62650eeede4d9d8.

Les correspondances Opus2D des deux hôtes sont identiques. Le commit est déjà
présent sur Cartae. Le dossier distant attendu est absent. Le helper importe
les objets Git du dépôt principal, puis la sonde exige le dossier inexistant.

## Référence T3 et correction

Code relu : /Users/moi/11.Repositories/t3code-local/apps/server/src/vcs/GitVcsDriverCore.ts,
fonction resolveRepositoryPathsUncached, lignes 1290–1367.
T3 distingue le répertoire Git commun, résolu en chemin réel, du dossier
de travail. Bridget reprend cette preuve pour reconnaître un checkout lié.

Différence distante : le helper prépare ce checkout sur Cartae, via la
fédération SSH existante. Son .git doit être un fichier et son répertoire
Git commun doit être celui de la source déclarée. Seul le chemin relatif
vérifié est transmis. La destination crée un checkout en tête détachée au
commit demandé. Les hooks restent désactivés. Aucun nouveau mapping,
protocole worker, fournisseur ou droit.

Le checkout principal, ses fichiers locaux et ses branches restent inchangés.
Un checkout existant doit partager le même dépôt Git. Le helper ne le
réinitialise pas. Les contrôles worker de commit, droits, identité,
disponibilité et rafraîchissement partagé restent ceux déjà en place.

Un dossier étranger, un lien symbolique et un dépôt imbriqué indépendant
sont refusés. Les fichiers non committés ne sont pas transférés.

## Vérification ciblée

15 tests Python PASS sur de vrais dépôts Git. Le test Rust existant des gardes
mapping absent, ambigu, explicite et origine passe aussi (1 PASS). Le transport SSH est simulé par
un exécutable qui lance réellement le helper distant dans une seconde
arborescence. Les cinq nouveaux cas couvrent : création et rejeu, dossier
étranger, lien symbolique, dépôt indépendant et hooks de destination avec
contrôle positif.

Le RED utilise la fixture finale et le seul script de production rétabli à
la base 4c23fcc07ffa3cbbd4c0849913714663cdf58b75. La préparation réussit à
tort, puis la lecture du dossier absent échoue. La même fixture passe avec
le correctif.

Preuves privées :
/home/moi/.cache/bridget-worktrees172/red-final.json
/home/moi/.cache/bridget-worktrees172/red-final.log
/home/moi/.cache/bridget-worktrees172/tests.json
/home/moi/.cache/bridget-worktrees172/green-final.log

Ce checkpoint ne qualifie pas encore l'activation ni le lancement de la
mission Opus2D. Ils doivent être consignés séparément. Aucun parcours
d'interface modifié ; pas de diaporama requis.
