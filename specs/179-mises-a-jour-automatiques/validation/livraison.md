# Livraison 179 — 25 octobre 2026

Statut : livré, automatisme activé, conservations justifiées.

## Version publiée et active

Release stable : https://github.com/guthubrx/bridget/releases/tag/v0.6.0

Source du binaire : 533f9ec60fe6f6d94868ac18974507a35c7f4c07, intégrée et poussée sur main. Le tag v0.6.0 vise ce commit. Le commit documentaire qui contient ce reçu est distinct de la source compilée. Base commune d6a2984f ; développements mobiles 177 et sécurité 178 conservés.

| Composant | Observation réelle |
|---|---|
| Core Cartae | 0.6.0, PID 1864895, service actif |
| Worker Cartae | 0.6.0, PID 1867403, service actif |
| Mobile Cartae | 0.6.0, PID 1865084, service actif |
| Connecteur Mac | 0.6.0, PID 45047, service actif |
| Surveillants de mise à jour | Cartae 1869234, Mac 50305, actifs |
| Pixel | Android 17, Bridget 0.6.0, versionCode 6000, installée à 06:51:06 CET |
| T3 | PID 650/975 conservés, aucune relance |

Choix réel du daemon : automatic=true. Paramètres Android : interrupteur ON. Le passage OFF a donné automatic=false sur le daemon ; le choix OFF a survécu à la fermeture de l’application. Le choix a ensuite été remis ON et conservé après installation.

Le téléphone courant a reçu le droit de gérer les mises à jour. Les permissions mobiles historiques n’ont pas changé. Ce droit séparé conserve la lisibilité de l’association par la version de retour.

## Publication et intégrité

Six actifs publiés puis retéléchargés : les deux archives natives, l’APK et leurs trois fichiers SHA-256. Les tailles et empreintes correspondent à l’API GitHub.

| Actif | SHA-256 |
|---|---|
| macOS ARM64 | 40078e76c65bd0b026b5ae24369bce33d40d2be5edde282f22c24b5c856d34bd |
| Linux x86_64 | a431d52e8c264514e318d37488ebb120f9b925a70d8a2a1327ed7a18e5897f26 |
| APK | 787864d11501d1ee98987da34bf839a0a7c2b46ae0d29bfa6fe6a7a2aeb13880 |

Exécutables installés : Mac cf14e5bee6855863f80f13a8e4c6a12d8762137960e86c4321755a89bad4849c ; Linux 3fd1a97589e286cc4064d3b29e43c21e82d556babec758edea32ca88d7c9cf68. Signature Mac vérifiée. APK installé relu depuis le Pixel et identique à l’actif GitHub ; installateur enregistré : app.cartae.bridget. Aucun adb install utilisé pour cette installation finale.

## Activation initiale

Les anciens services 178 ne savaient pas traiter l’API de mise à jour. Cette première activation a donc utilisé une maintenance contrôlée. Core et mobile arrêtés proprement pour fermer les nouvelles admissions. Cgroups relus : aucun enfant de mission ; worker seul et 250 commandes terminales, une refusée ; aucune admission pending/chosen. Le worker a été arrêté après ce contrôle.

Une barrière privée bornée à 180 secondes a été posée pendant que l’autorité était arrêtée. Les liens stables et services ont été installés, puis les nouveaux exécutables chargés et vérifiés. La barrière a été libérée après la santé Mac/Cartae. Aucun état de tâche n’a été écrit. Le worker a effectué cinq reprises de connexion pendant la fenêtre, puis est resté stable après sa libération. Aucun fournisseur ni T3 n’a été arrêté.

Les activations suivantes passent par la demande de fenêtre du daemon, avec exclusion de l’activité, revalidation, santé et retour contrôlé. Ce premier déploiement n’est pas présenté comme une activation automatique depuis un ancien daemon incompatible.

## Recette Android

Réglages réels : défaut ON, OFF, fermeture/réouverture OFF, retour ON ; préférence des hôtes observée. Autorisation Android d’installer depuis Bridget accordée par son écran système.

La recherche a trouvé 0.6.0 sur GitHub. Le job Android planifié a été déclenché pour la recette, sans attendre le regroupement du système. La première tentative automatique a ouvert Play Protect. La réouverture de Bridget pendant cette fenêtre l’a interrompue ; Android a refusé l’installation. L’application a conservé 0.5.0 et n’a pas bouclé. La reprise depuis son bouton et la confirmation « Mettre à jour » ont installé 0.6.0. L’association et les favoris restent présents.

Installation avec confirmation prouvée ; installation entièrement silencieuse non revendiquée. Les protections Android restent actives. Le job périodique persistant est présent ; les six heures réelles n’ont pas été attendues.

## Contrôles

Python 13 PASS ; Rust updates 7 PASS, build_info 9 PASS, mobile 69 PASS / 1 ignored ; Android JVM 18 PASS, lint/constructions réussis ; Pixel 3 tests instrumentés PASS. Compilation finale Mac 1 min 48 s et Linux 1 min 38 s. Relecture directe du delta ; pas de contre-revue indépendante, car la délégation était interdite dans ce tour. Aucune FULL ni score global.

Preuves visibles : diaporama vérifié, dix étapes, navigation testée. Captures originales, journaux et manifestes hors Git. Le HTML versionné ne contient pas de credential.

## Limites et entretien

- La tâche historique e5959986-e426-422d-956f-e035dbcf89be reste cancelling, avec arrêt non confirmé. Son payload est identique avant/après livraison. Un essai réel de préparation d’activation renvoie update_missions_active_or_unconfirmed : une prochaine activation attendra sa réconciliation. Aucune fausse clôture n’a été inscrite.
- Les clients MCP déjà ouverts gardent leur ancien code. Le montage de ce tour répond encore identity_not_found. Le chemin de lancement stable est configuré ; pas de copie d’identité ni de redémarrage de T3 pour forcer le rechargement.
- Mac : quatre anciens exécutables non ouverts et sans référence de service retirés, quatre copies de validation retirées, worktree de compilation 179 retiré. Total 225 324 587 octets logiques pour les fichiers retirés ; pas de gain physique APFS annoncé.
- Version active 0.6.0 et retour 178-c49d2766 conservés. Les MCP encore ouverts sur 178 restent protégés.
- Cartae : lsof privilégié signale fuse.portal et une observation incomplète. Worktree 179, branche et caches conservés, sans forcer le retrait. Les rapports de tests et fichiers mémoire ignorés sont archivés : 26 fichiers, archive 85 666 octets, SHA-256 0bd92c51c1dc27a4f19c989478dfac6e45c3ab0c2ebd9fe7e3115b038867bd11.
- Les worktrees 177, 178 et la contre-revue associée appartiennent aux autres travaux ; ils sont conservés. Aucun processus d’un autre travail terminé pour le ménage.
- Commande installed exécutée sur les deux machines après installation vérifiée. Cache Mac environ 1,40 Gio ; cache Cartae conservé sur observation incomplète. Aucun target copié par test.

Les sources et documents sont intégrés à main et poussés. Le worktree 179 conservé reste propre et sur le même commit documentaire. Les reçus Git finaux et de nettoyage sont conservés hors Git pour éviter une référence circulaire au commit du présent document.

## Contrôle complémentaire d’entretien — 25 octobre 2026, 07:02 UTC

Les deux `main` sont propres et alignés sur GitHub au commit documentaire 504e1a34 avant ce complément. Toutes les branches de travail restantes sur Cartae sont intégrées à `main`. Après actualisation des références, GitHub ne contient aucune branche de travail distante à retirer.

Le dernier checkout temporaire Mac, `/Users/moi/.cache/bridget-security178/source-mac`, a été retiré sans force. Sa source c49d2766 est intégrée, son état est propre, aucun fichier ignoré n’y reste et le contrôle lsof ciblé ne signale aucun usage ni avertissement. Retrait supplémentaire : 98 769 613 octets logiques, distincts des 225 324 587 octets précédemment annoncés. Mac ne conserve qu’un checkout et la branche `main`.

Mac conserve les installations 0.6.0 active et 178-c49d2766 pour retour ; cette dernière reste aussi chargée par un client MCP. Quatre anciennes sauvegardes Git (stash, sessions 100, 142 et 168) restent présentes : leur intégration complète n’est pas qualifiée et elles ne sont pas effacées.

La commande d’entretien a été exécutée sur les deux machines. Aucun cache supplémentaire n’a été retiré : 1,40 Gio sur Mac et 9,91 Gio sur Cartae, sous le budget de 10 Gio. Les quatre worktrees Cartae et les binaires 174-mcp-installed-d4965982 / 177-r10e-e631e47c restent conservés : lsof privilégié signale toujours fuse.portal et une observation incomplète, y compris lors du contrôle ciblé. Les tâches des autres travaux et leurs fichiers ignorés ne sont pas clôturés ou supprimés pour contourner ce refus.

Aucun code, binaire actif ou service n’a changé pendant ce contrôle. La règle permanente de fin de livraison reste dans AGENTS.md : intégration, push, vérification et nettoyage à chaque livraison, avec conservation explicite des usages et travaux non qualifiés. Le reçu détaillé Mac est conservé hors Git dans `/Users/moi/.cache/bridget-updates179/cleanup-followup.json`.
