# Nettoyage Git et compilations — 22 octobre 2026

Demande : committer les travaux prêts, les fusionner puis nettoyer les branches,
les worktrees et les fichiers de compilation inutilisés de Bridget.

## Intégration

Le test retrouvé est commité dans `ba3cea393c3227b5bcfa7107b41c88cb74407b3e`,
puis fusionné dans `6b0bae2300aa3965e4599ab47300a7d7fcb29f9b` et poussé sur main.
Il vérifie qu'une préparation complète ne déclenche pas les hooks du dépôt source.
Un vrai commit témoin prouve que le hook post-commit peut s'exécuter.
Les 10 tests de préparation passent, avant puis après la fusion.
La revue indépendante Bridget `69e564de-5ac7-4d16-99b2-71f2146df8fd` approuve.
Son résultat a été lu, acquitté ; cleanup_done=true a été contrôlé avant retrait.
Limite : ce test ne prouve pas, à lui seul, la protection core.hooksPath de la destination.
Aucun code de production modifié, aucune compilation ni suite complète nécessaire.

## Nettoyage réalisé avant le commit de ce reçu

- 4 worktrees retirés sur Mac et 15 sur Cartae ; leurs HEAD sont intégrés dans main.
- 16 branches locales correspondantes retirées par git branch -d, sans forcer.
- Caches régénérables purgés sous verrou Cargo : 1 502 492 809 octets logiques sur Mac,
  3 540 434 317 sur Cartae. Exécutables, verrous, journaux et preuves conservés.
- 54 copies de sources de livraison compactées : 5 313 193 442 octets logiques
  deviennent 1 999 221 518 octets d'archives gzip, hors manifestes de preuve.
  Chaque fichier, mode et lien a été comparé à l'archive avant retrait de la copie.
  Les empreintes SHA-256 et les originaux non commités sont conservés dans les archives.
- Aucun worktree modifié retiré. Le témoin de recette du worktree 42ba7b03 a été
  copié et comparé avant son déplacement dans les preuves privées.

Le contrôle des fichiers ouverts a précédé les retraits. Sur Cartae, il porte sur
le volume ext4 des candidats. Le montage FUSE séparé /run/user/1002/doc a été exclu
explicitement du scan ; la seconde observation ne comporte aucune erreur.
Les profils contenant un exécutable actif restent protégés.
Les tailles logiques ne constituent pas une mesure du gain physique APFS.

## Travaux conservés

Ces dossiers contiennent encore des changements non commités et restent en place :

- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/171-corrections-usage-2
- /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission
- /home/moi/10.Scripts/64.bridget/.worktrees/8a8f5934
- /home/moi/10.Scripts/64.bridget/.worktrees/c933dd39

Leurs modifications ont été comparées à l'inventaire initial et sont inchangées.
La modification personnelle de /Users/moi/Nextcloud/10.Scripts/64.bridget/AGENTS.md
reste hors de ce commit, avec la même empreinte f0eb89de95fbd0bccb2759b51b2612d3b97e73ace41769d6e6a2f3b826a037a1.

## Services et preuves

Le maître Bridget 3688, le pont 898 et T3 1372/1435 gardent leurs dates de naissance.
Le worker Cartae reste actif sur 589872, NRestarts=0.
Build actif inchangé : 172-e5195b7d. Aucun service ni mission métier relancé.
Binaires actifs conservés : Mac 20a44e11b74ebbe92e118fee4152cec6874270542aa5d6a895bd31e01860511c ;
Cartae 2b3f36fc577a08fa41ac55cbb6c69b6e4f925463d688077f6188cb486dc4574c.
Bases, données métier, sauvegardes de retour arrière et historiques conservés.

Index durable :
/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/172-resolution-projet-distant/validation/nettoyage-20261022.json

Archives et manifestes complets :
/Users/moi/.cache/bridget-cleanup-20261022/snapshots

Le worktree /home/moi/10.Scripts/64.bridget/.worktrees/172-resolution-projet-distant
sert uniquement à publier ce reçu. Sa fermeture, le retrait de la branche locale
et distante de session, les SHA Git finaux et l'espace libre observé après le lot
sont consignés séparément dans :
/Users/moi/.cache/bridget-cleanup-20261022/final.json

Restauration d'une copie : lire son champ source et son champ archive dans l'index,
puis extraire l'archive dans le dossier parent de source. Aucune extraction automatique.
