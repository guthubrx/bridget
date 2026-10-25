# Archives techniques de l’adaptateur T3

Document d’origine : [specs/179-mises-a-jour-automatiques/validation/livraison.md](../../../../../../../specs/179-mises-a-jour-automatiques/validation/livraison.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Composant | Observation réelle |
|---|---|
| Core Cartae | 0.6.0, PID 1864895, service actif |
| Worker Cartae | 0.6.0, PID 1867403, service actif |
| Mobile Cartae | 0.6.0, PID 1865084, service actif |
| Connecteur Mac | 0.6.0, PID 45047, service actif |
| Surveillants de mise à jour | Cartae 1869234, Mac 50305, actifs |
| Pixel | Android 17, Bridget 0.6.0, versionCode 6000, installée à 06:51:06 CET |
| T3 | PID 650/975 conservés, aucune relance |

<a id="extrait-2"></a>

## Extrait 2

Une barrière privée bornée à 180 secondes a été posée pendant que l’autorité était arrêtée. Les liens stables et services ont été installés, puis les nouveaux exécutables chargés et vérifiés. La barrière a été libérée après la santé Mac/Cartae. Aucun état de tâche n’a été écrit. Le worker a effectué cinq reprises de connexion pendant la fenêtre, puis est resté stable après sa libération. Aucun fournisseur ni T3 n’a été arrêté.

<a id="extrait-3"></a>

## Extrait 3

- La tâche historique e5959986-e426-422d-956f-e035dbcf89be reste cancelling, avec arrêt non confirmé. Son payload est identique avant/après livraison. Un essai réel de préparation d’activation renvoie update_missions_active_or_unconfirmed : une prochaine activation attendra sa réconciliation. Aucune fausse clôture n’a été inscrite.
- Les clients MCP déjà ouverts gardent leur ancien code. Le montage de ce tour répond encore identity_not_found. Le chemin de lancement stable est configuré ; pas de copie d’identité ni de redémarrage de T3 pour forcer le rechargement.
- Mac : quatre anciens exécutables non ouverts et sans référence de service retirés, quatre copies de validation retirées, worktree de compilation 179 retiré. Total 225 324 587 octets logiques pour les fichiers retirés ; pas de gain physique APFS annoncé.
- Version active 0.6.0 et retour 178-c49d2766 conservés. Les MCP encore ouverts sur 178 restent protégés.
- Cartae : lsof privilégié signale fuse.portal et une observation incomplète. Worktree 179, branche et caches conservés, sans forcer le retrait. Les rapports de tests et fichiers mémoire ignorés sont archivés : 26 fichiers, archive 85 666 octets, SHA-256 0bd92c51c1dc27a4f19c989478dfac6e45c3ab0c2ebd9fe7e3115b038867bd11.
- Les worktrees 177, 178 et la contre-revue associée appartiennent aux autres travaux ; ils sont conservés. Aucun processus d’un autre travail terminé pour le ménage.
- Commande installed exécutée sur les deux machines après installation vérifiée. Cache Mac environ 1,40 Gio ; cache Cartae conservé sur observation incomplète. Aucun target copié par test.
