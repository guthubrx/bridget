# Conservation avant retrait des anciens worktrees

François a demandé le 10 novembre 2026 de ne conserver que main, d'y réunir tous les travaux et de libérer les builds reproductibles, sans perdre les sources ni les preuves.

Le commit 2316a179 de la session 260 est fusionné dans main par 8366988f. Les trois décisions D1, D9 et D10 sont ainsi conservées dans leur fichier actif. Aucun code exécutable n'a été ajouté par cette fusion.

Trois anciens checkouts de compilation SYS contenaient encore des différences non commitées des sessions 258 et 259. Elles sont conservées ici intégralement par git diff HEAD --binary, compressées sans perte, avec leur commit de base et SHA-256 dans manifest.json. Leurs versions finales figurent déjà dans main ; les anciennes variantes ne doivent pas remplacer le code actif. Le couple base Git + patch décompressé permet de reconstruire chaque état de travail retiré.

- sys-258-composer-partiel.patch.gz : état partiel du worktree enfant da08d884, base e41b75fe6371085c34e73da36c6ff1a1b4d05a45.
- sys-258-construction.patch.gz : checkout de compilation 258-release-build, même base.
- sys-259-construction.patch.gz : checkout de compilation 259-guidage-preflight, base 065f54f05d26224b0e31198a4db48ca208232da7.

Ces archives ne constituent pas de nouveaux correctifs à appliquer. Les sources courantes restent celles de main. La conservation précède toute restauration puis suppression des anciens checkouts. Seuls les fichiers de build reproductibles et les doubles de fichiers déjà conservés sont supprimables.

Le journal ignoré du worktree Mac 260 est conservé dans le checkout principal : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/260-assainissement-code/validation/base-tests.txt. Taille 454273 octets ; SHA-256 b8672273c6e87256913bafcec549c604c31f685a9e66a1cbc8795bb4b160761c. Son contenu brut n'est pas ajouté au dépôt public.
