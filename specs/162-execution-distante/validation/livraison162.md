# Livraison SPEC-162 — livrée et nettoyée, conservations justifiées

**Checkpoint :** 13 octobre 2026, clôture opérationnelle. **Responsable :** ROOT. **Statut :** Implemented, 46/46. **Livraison :** G8 et G9 validés ; conservations justifiées ci-dessous.

## Clôture de livraison et de nettoyage — 13 octobre 2026

La session 162 est livrée : 46/46 tâches, G8 et G9 validés. Le code actif reste 1dc0c6022e8d0712fbc4c65a8c20be7b2ea8a437. Les documents précédents ont été intégrés et poussés au commit 4cf50b68688b2d28872f67ae32e129f989d22449. Le présent checkpoint documentaire doit encore être committé et poussé par ROOT ; aucun SHA futur n’est présumé.

Le worktree Bridget 162 et sa branche ont été retirés sans force. Le worktree et la branche T3 du lot avaient déjà été retirés. Les 45 fichiers ignorés inventoriés possèdent chacun une copie durable byte-identique. Preuve de retrait : /Users/moi/.cache/bridget-delivery162-20261013/cleanup162-cancel-worktree-result.json. Les chemins de worktrees dans la chronologie sont désormais historiques ; aucun worktree n’a été recréé.

Les intermédiaires du stage Mac ont été retirés par la commande existante build.py installed : 0,31 Gio logiques, binaire source conservé. Le retrait antérieur du stage T3 représente 840 737 986 octets logiques et son archive de 59 preuves reste vérifiée. Aucun gain physique de volume n’est déduit de ces tailles.

Conservations décidées après inventaire :

- Cache debug Mac de 44,96 Gio, utile pour 163, budget temporaire de rétention 46 Gio ; cache Cargo Linux utile également conservé.
- Stage Git distant d’environ 3,45 Go et préfixes de validation d’environ 276 Mo : garde d’usage lsof incomplète, y compris avec root, à cause du montage /run/user/1002/doc. Aucun retrait forcé ni masquage de cette limite.
- Git 2.56 installé, profil /etc/apparmor.d/bwrap-userns-restrict, sauvegardes de rollback, bases, preuves et cache source T3 contenant cache.db : nécessaires au fonctionnement, au retour arrière ou à 163.

L’inventaire et la qualification sont conservés dans /Users/moi/.cache/bridget-delivery162-20261013/cleanup162-cancel-inventory.md et /Users/moi/.cache/bridget-delivery162-20261013/cleanup162-cancel-priority-inventory.json. L’inventaire initial était en lecture seule ; les retraits réellement effectués sont distingués ici des conservations. Les 17 worktrees T3 étrangers restent hors périmètre.

Le lien client /home/moi/.local/bin/bridget a été basculé après comparaison de sa cible 611bfa vers /home/moi/.local/lib/bridget-communication/162-1dc0c6022e8d/bin/bridget. SHA effectif : 2131f9a1f2f1b33e39096d3618e012fb7bb2465966f8cef9b6fb7832c4e71352. Le parent Opus de recette est arrêté et décommissionné, historique conservé.

La lecture finale ciblée ROOT des preuves, du code et des correspondances documentaires ne retient aucun nouvel écart. tasks.md reste inchangé, SHA-256 44d552e8cf891ef1789b26ab3c85f8a670ea88e617220222aae7849d609adb06. Son en-tête G9 ouvert décrit le gel avant nettoyage ; ce reçu clôt le gate opérationnel sans modifier ce fichier figé. La FULL et l’audit global historiques conservent leurs limites ; aucun GREEN global ou nouvel audit global n’est annoncé.

## Chronologie des validations et opérations

Les gates ouverts et les chemins de worktrees ci-dessous décrivent les checkpoints antérieurs. La clôture ci-dessus fait foi.

## Checkpoint final T046 et G8 — 13 octobre 2026

ROOT et l’observateur ont validé l’annulation réelle après installation du correctif 1dc0c6022e8d0712fbc4c65a8c20be7b2ea8a437. La tâche e2a5d605-063b-4103-b88b-a9c6b247240a est cancelled, résultat null, cleanup_done=true et error=null. Les huit PID observés sont tous absents à 15:58:36,355475 UTC (17:58:36,355475 CEST), avant la fin naturelle prévue du sleep à 15:59:17 UTC. La commande unique avait commencé à 15:56:17 UTC ; la fermeture est intervenue à 15:57:45 UTC. Cette observation solde le défaut réel de descendants survivants.

Les preuves corrélées transmises par ROOT sont conservées avec leurs empreintes :

| Preuve | SHA-256 et résultat |
|---|---|
| .drained exact | f005b5dfae73a0d1da3bb9cc60cc3b75a8d440bc5a7f0266d12ff3fe41e06a99 ; preuve de fin du drainage corrélée à l’exécution. |
| Reçu terminal | c83d05cd282f9cdbd46422979ef10832d19e39011356c1cc4fdb440c8abbae40. |
| Slot résultat | 50f41beea096f645d3606095e9ba555f751080b4fcac2afb84c9ac0664960b5c ; producer_terminal=true, clôturé pour cancelled, disposition=null, finalisé à 15:57:46 UTC. |
| Journal | a7a69bb0b630d75d9a668a3cde0e158a230d5271efdad0aea80bfa67e509a142 ; 19 événements, marqueur de recette non émis. |

Le maître n’a plus de tâche active et les admissions sont ouvertes : /Users/moi/.cache/bridget-delivery162-20261013/cancel162-rerun-active-after.json porte total_active=0, items=[], halted=false. Le parent Opus de recette a été arrêté proprement par la commande e27086a3-3a46-45da-aaca-7862f4e61a66, puis décommissionné par 13c264d9-7bc1-428f-9476-896fcd037d3f ; son historique reste conservé.

L’activation est attestée par /Users/moi/.cache/bridget-delivery162-20261013/rollback611bfa-20261013-cancel-1dc0c602-r3/progress.json : phase activate réussie ; daemon Mac 71469 né à 17:53:53 CEST, pont 71567 né à 17:53:55 ; worker Linux 471321, naissance 142199350, exécutable du préfixe /home/moi/.local/lib/bridget-communication/162-1dc0c6022e8d. Les SHA des binaires actifs sont d9e09a78a0d0ff2a70b152f78e42ac994aea53094202f23f8ceb91f6b6bedc44 sur Mac et 2131f9a1f2f1b33e39096d3618e012fb7bb2465966f8cef9b6fb7832c4e71352 sur Linux. DB quick_check ok, zéro erreur de clé étrangère, schémas 7/12 et Fleet 5. T3 704/969 et le tunnel 1384 restent inchangés par rapport au préflight après le redémarrage externe.

T046 est cochée : 46/46, statut Implemented, G8 validé. Les deux tests de bibliothèque et cinq E2E Linux, la contre-revue R2 APPROVE et l’annexe d’audit sans constat critique ou élevé restent les preuves ciblées du correctif. ROOT a relu le rapprochement code/documents/tests sans nouvel écart retenu. Ce checkpoint ne remplace ni la FULL historique ni l’audit global ; aucun GREEN global n’est revendiqué. G9 reste ouvert jusqu’au nettoyage final exécuté par ROOT.


## Chronologie historique des opérations précédentes

Les états ci-dessous sont antérieurs au checkpoint final ci-dessus.

## État opérationnel après le préflight R3 — 13 octobre 2026

Le correctif source 1dc0c6022e8d0712fbc4c65a8c20be7b2ea8a437 est intégré et poussé. Les deux builds sont réussis. Le correctif n’est pas encore activé ; T046 et les gates G8/G9 restent ouverts. Le gel de sauvegarde a arrêté le worker, le daemon et le pont. Aucun redémarrage réussi après ce gel n’est encore attesté ici.

| Surface | Dernière preuve transmise par ROOT |
|---|---|
| Build Mac | RC0, 169,694 s ; binaire signé /Users/moi/.cache/bridget-build-162-cancel.8rq76kny/release/bridget ; SHA-256 d9e09a78a0d0ff2a70b152f78e42ac994aea53094202f23f8ceb91f6b6bedc44. |
| Build Linux | RC0, 70,866 s ; préfixe /home/moi/.local/lib/bridget-communication/162-1dc0c6022e8d ; SHA-256 2131f9a1f2f1b33e39096d3618e012fb7bb2465966f8cef9b6fb7832c4e71352. |
| Source précédemment active | 611bfa7557b9. Deux préflights avaient expiré pendant un ancien sweep tenant le mutex et un appel pread, dans un contexte de swap. L’arrêt du pont a rétabli la CLI ; admissions fermées à 17:13, génération 5, raison livraison162-correctif-annulation. |
| Redémarrage externe du Mac | Observé vers 17:29–17:31, avec uptime de 18 minutes à 17:47. T3 PID 704/969, nés à 17:31:27/33 ; daemon 1362 et pont 1374 nés à 17:31:50 ; tunnel 1384. Ces identités remplacent les anciennes observations. La procédure du correctif ne relance pas T3 et ne modifie pas le tunnel. |
| Gel et sauvegarde R3 | Préflight réussi ; worker, daemon et pont réellement arrêtés. Sauvegarde Linux réalisée, SHA-256 4f206849c7af476ac7257319365992de5d3c3c7d26d6e72953e126744e8a9876. Dossier de reprise : /Users/moi/.cache/bridget-delivery162-20261013/rollback611bfa-20261013-cancel-1dc0c602-r3. La sauvegarde jointe Mac n’est pas déclarée achevée. |
| Garde SQLite | La garde Mac a arrêté la procédure : le lecteur SQLite du contrôle de quiescence conservait sa propre connexion ouverte. Diagnostic reproduit : lsof trouve le PID lecteur avant collecte mémoire, puis aucun après. Correction de fermeture explicite dans le script privé et reprise en préparation ; aucune nouvelle modification produit nécessaire. |

Preuve du diagnostic : /Users/moi/.cache/bridget-delivery162-20261013/cancel162-sqlite-reader-diagnosis.json. Le diagnostic n’a pas modifié la production. Ces opérations n’ajoutent aucun test produit, aucune FULL et aucun rebuild aux builds déjà attestés. Les trois recettes nominales précédentes restent acquises ; elles ne soldent pas la recette d’annulation après installation.

## Chronologie antérieure — première installation et recettes

Les PID et les états actifs du tableau ci-dessous décrivent la première installation. Ils ne décrivent pas l’état après redémarrage externe puis gel de sauvegarde R3.

Les sources sont intégrées et poussées. Bridget 162 est actif sur le Mac et le worker Cartae est démarré. T3 162 est installé sur disque ; ses processus conservent leur code chargé précédent. La recette distante Bridget, la survie du fournisseur à l’arrêt du worker et la récupération du résultat par le parent revenu sont attestées. Les routes Roadie et Opus2D ont également réussi. La livraison complète reste ouverte : l’annulation a laissé des descendants Linux vivants malgré cancelled/cleanup_done. Le correctif T045 est validé sur Linux isolé. Son installation et la recette réelle T046 restent nécessaires ; le nettoyage final reste ouvert.

| Surface | État attesté |
|---|---|
| Git Bridget | 611bfa7557b97587614687f6c6379d5961101fa6 intégré sur main et poussé vers github ; référence distante identique, 61 fichiers vérifiés. |
| Git T3 | 174a9d04554758aacffb4aade588e61c810faf5a intégré sur local/main-20261009 et poussé vers fork ; référence distante identique, 10 fichiers vérifiés. |
| Bridget Mac | Binaire installé et signé, Build ID 611bfa7557b9. Daemon 90803 et pont 90815 nés le 13 octobre à 15:11:21 CEST. SHA binaire installé : 1b2de5e0a2fe7aecc4637585928fbc611cd9fd957b7d93540713e00cf06de531. |
| Sauvegarde et données | Sauvegarde jointe vérifiée dans /Users/moi/.cache/bridget-install162-20261013-1515. Schémas DB idempotence 7 et exécutions 12 ; quick_check ok, zéro erreur de clé étrangère. Fleet sur disque reste en schéma 4, byte-identique à la sauvegarde ; la prochaine persistance écrit le schéma 5. |
| Worker Cartae | Unité bridget-worker-cartae.service active, PID actuel 340014 après arrêt/reprise de recette, release 162-611bfa7557b9. Lien client basculé vers 162. Configurations des trois projets Bridget, Roadie et Opus2D enabled=true ; aucun projet Cartae activé. |
| T3 sur disque | /Applications/T3 Code (Local).app installé en 0.0.45-local.162 par échange atomique ; signature stricte/deep RC0 et entitlements inchangés. Sauvegarde /Users/moi/.cache/bridget-install162-20261013-1515/T3-before.app. |
| T3 chargé | PID 60116/60228 et naissances du 11 octobre à 01:13:35/36 inchangés. Aucun redémarrage ni activation programmée. L’installation sur disque ne prouve pas l’affichage 162 dans le processus actuel. |
| Recette G8 | Bridget : mission 26465d28-fc14-40b5-8ae4-07ca375ab948 en result_available, résultat Linux/cartae/bon cwd/HEAD611bfa ; reprise worker et récupération après retour du parent attestées. Roadie et Opus2D réussies également. Annulation en échec réel : descendants Linux survivants malgré Terminal/cleanup_done ; T045 validée ; T046 ouverte. Les premiers refus Git et sandbox restent historiques. |
| Nettoyage G9 | Worktree et branche T3 162 retirés après vérification ; stage /private/tmp/b162.24BX8rMN retiré, 840 737 986 octets logiques. Archive de 59 preuves vérifiée. Cache source T3 avec DB et cache debug conservés pour163. Le gate global reste ouvert. |

Le contrôle initial de fin d’activation exigeait à tort Fleet 5 immédiatement. ROOT a corrigé cette vérification après avoir constaté Fleet 4 byte-identique et la migration à l’écriture prévue par le code. Cette correction du contrôle n’a provoqué aucun redémarrage supplémentaire. La preuve d’activation conserve l’incident initial et sa résolution.

Preuves courantes relues :

- /Users/moi/.cache/bridget-delivery162-20261013/delivery162-git-sources.json
- /Users/moi/.cache/bridget-delivery162-20261013/delivery162-runtime-checkpoint.json
- /Users/moi/.cache/bridget-install162-20261013-1515/activation-progress.json
- /Users/moi/.cache/bridget-install162-20261013-1515/T3-install-receipt.json

Le premier checkpoint runtime conservait encore investigation_in_progress pour le refus Git. Les preuves de recette ci-dessous lui succèdent ; elles ne transforment pas les essais précédents en succès. Les états des admissions dans les preuves de bascule sont historiques ; aucun état courant supplémentaire n’est inventé ici. Audit final A 99,095238 sur 4,228866 % des lignes du périmètre, validateur conforme ; limites de tests et L1 conservées ci-dessous. Aucun GREEN global.

## Recette Bridget et reprise réelle — preuves courantes

La mission **26465d28-fc14-40b5-8ae4-07ca375ab948**, enfant 47426d54-b7e9-4438-b909-3cca5c4fef09, a exécuté une unique commande de recette en lecture seule. Résultat conservé : RECETTE162_BRIDGET_OK, Linux, cartae, /home/moi/10.Scripts/64.bridget, HEAD 611bfa7557b97587614687f6c6379d5961101fa6. Le fournisseur est Codex gpt-5.6-terra ; cela atteste cette recette, pas une disponibilité générale du quota.

L’arrêt du service worker a laissé le wrapper PID337093, naissance141424773, et sleep PID338338, naissance141426363, vivants et inchangés. Le worker a ensuite redémarré sous PID340014. Le parent a été arrêté pendant la recette. Le maître a conservé result_available et le résultat, avec native_result_owner_offline, result_sent=false et cleanup_done=false avant son retour.

Le même parent durable **51a2215b-f145-4e54-bf17-741a5f876192** est revenu : nouvelle instance c296fe45-11b2-476e-8a15-ea1ae06ec28a, ancienne instance d’origine caca3b8c-f25a-44c4-b656-ab2dc8196bf0 conservée. Après récupération, le résultat est identique, error=null, result_sent=true et cleanup_done=true. Le reçu final du résultat porte accepted. ROOT rapporte que la commande de relance avait rendu outcome_unknown à sa borne de lecture JSONL ; la présence réelle du même UUID puis les preuves de récupération attestent son issue. Ne pas convertir cet avertissement CLI en échec ou en seconde relance nécessaire.

Preuves :

- /Users/moi/.cache/bridget-delivery162-20261013/recipe162-bridget-recovery.json
- /Users/moi/.cache/bridget-delivery162-20261013/recipe162-worker-stop.json
- /Users/moi/.cache/bridget-delivery162-20261013/recipe162-worker-restart.json
- /Users/moi/.cache/bridget-delivery162-20261013/recipe162-parent-stopped-task.json
- /Users/moi/.cache/bridget-delivery162-20261013/recipe162-recovery-before-parent-return.json
- /Users/moi/.cache/bridget-delivery162-20261013/recipe162-parent-relaunch-agents.json
- /Users/moi/.cache/bridget-delivery162-20261013/recipe162-recovery-after-parent-return.json

Les essais antérieurs restent distincts : refus avant mission pour Git insuffisant, puis mission 258ff82c-90c7-4912-bf02-19806c1e7822 dont le résultat signalait un échec bwrap, sans sorties de commande. ROOT a corrigé le prérequis Ubuntu AppArmor avec le profil bwrap, sans désactiver la restriction globale. La sonde finale atteste lecture permise, écriture refusée (errno30) et réseau refusé (errno1), profil bwrap//&unpriv_bwrap en enforce. Aucun gain de droits de mission n’est déduit. Preuves : /Users/moi/.cache/bridget-delivery162-20261013/recipe162-bridget-sandbox-failure-final.json ; /Users/moi/.cache/bridget-delivery162-20261013/sandbox162-profile-installed.json ; /Users/moi/.cache/bridget-delivery162-20261013/sandbox162-probes-r3-result.json.

## Nettoyage partiel attesté — G9 encore ouvert

Le worktree /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant et la branche session-162-lineage-distant ont été retirés après contrôle de propreté, intégration/push et absence d’utilisation. Les worktrees étrangers restent hors périmètre. Preuve : /Users/moi/.cache/bridget-delivery162-20261013/cleanup162-t3-worktree.json.

L’archive /Users/moi/.cache/bridget-delivery162-20261013/cleanup162-t3-proofs.tgz contient **59 preuves vérifiées** ; SHA-256 **0e5e39b2614a0b286c979929ab5e4fdcbd2a69e49aaf1ab9dfeed87c7d2fc88c**. Le stage /private/tmp/b162.24BX8rMN a été retiré après contrôle de signature installée et de fichiers ouverts. Son inventaire compte **840 737 986 octets logiques**, sans revendication de gain physique APFS. Preuves : /Users/moi/.cache/bridget-delivery162-20261013/cleanup162-t3-proofs-verified.json ; /Users/moi/.cache/bridget-delivery162-20261013/cleanup162-t3-proofs-manifest.json ; /Users/moi/.cache/bridget-delivery162-20261013/cleanup162-t3-stage-result.json.

La source de cache /Users/moi/.cache/t3-final162.2nosz6zd/source et sa DB /Users/moi/.cache/t3-final162.2nosz6zd/source/node_modules/.vite/task-cache/v18/cache.db sont conservées pour163. Le retrait du stage n’a modifié ni cette DB, ni le cache debug, ni l’ancien bundle, et n’a relancé aucun processus. Le nettoyage release Mac rapporte séparément 0,31 Gio logiques retirés et 44,96 Gio de caches restants protégés, sans suppression forcée : /Users/moi/.cache/bridget-delivery162-20261013/cleanup162-mac-stage.log. Les sauvegardes et les ressources encore nécessaires à la recette restent conservées.

**G8/G9 restent ouverts.** Les succès des trois routes et ce nettoyage partiel ne résolvent pas le défaut réel d’annulation ni la clôture globale des ressources. Les 44 tâches validées sont conservées ; T045/T046 sont ajoutées ouvertes.

## Routes nominales supplémentaires attestées

Roadie : tâche d96b9131-0870-425a-90f0-35b7b56e1306, result_available, RECETTE162_ROADIE_OK, Linux/cartae, /home/moi/10.Scripts/39.roadie, HEAD acc10e13c41026ce444fe38933840b1ea9e2c976 ; result_sent=true et cleanup_done=true. Preuve : /Users/moi/.cache/bridget-delivery162-20261013/recipe162-roadie-task-current.json.

Opus2D : tâche 366ca210-2722-4de0-9ee1-01441d9f00b1, result_available, RECETTE162_OPUS2D_OK, Linux/cartae, /home/moi/10.Scripts/69.opus2D, HEAD 3e677ad669648f953bb0bd951b2dc72777b3a9fc ; result_sent=true et cleanup_done=true. Preuve : /Users/moi/.cache/bridget-delivery162-20261013/recipe162-opus2d-task-current.json. Les résultats contiennent des séparateurs de ligne échappés ; leur lecture n’ajoute aucune exécution. Ces deux recettes nominales ne prouvent pas l’arrêt de l’arbre de processus de la recette d’annulation.

## Réouverture après recette G8 — arrêt incomplet, 44/46

La recette d’annulation réelle a révélé un défaut sur la tâche **9fb66659-3ae0-481d-9b0a-335261a000f4** : cancelled/cleanup_done et reçu worker Terminal alors que codex PID355934 et sleep PID355935 restaient vivants, naissance141506025 et PGID355934. Le wrapper PID354553 avait disparu. Les descendants étaient encore observés après environ158 secondes puis ont fini naturellement ; aucun signal correctif n’a été envoyé par l’observateur. UID/SID/namespace de ces processus n’ont pas été capturés. Le statut métier n’atteste donc pas ici l’arrêt complet.

Preuve : /Users/moi/.cache/bridget-delivery162-20261013/recipe162-cancel-observer-failure.json, SHA d77ad9ca5de6111cb462cb019757dee720291e701bedcfb415d5378e4921edea. Cause retenue : le suivi du groupe du wrapper ne couvre pas les groupes distincts créés par la sandbox. Le correctif garde le wrapper Linux comme subreaper jusqu’à récolte de ses descendants et publication d’une preuve durable de drainage ; le worker exige cette preuve avant Terminal après RELEASE possible.

Le plan /Users/moi/.cache/bridget-delivery162-20261013/cancel162-fix-plan.md est relu, SHA c5f9e91687bf84713807523e984789ad63826a836502727a78de53aee91e9d3b. La revue /Users/moi/.cache/bridget-delivery162-20261013/cancel162-plan-review-r1.md porte APPROVE_WITH_CHANGES. Les quatre points ont une réponse dans le plan final : pas de récolte concurrente des Child ; inventaire de tous les TID et prérequis Linux ; garde des trois branches Terminal ; preuve corrélée au marqueur exact (bootstrap exec du wrapper). ROOT a arbitré un unique thread temporaire de signalisation sans récolte pendant stop/Drop, puis récolte après retour des propriétaires de Child. Ces réponses sont des décisions de plan autorisées, pas une preuve de correction ou de test réussi.

Prérequis réels : Linux6.8.0-142-generic, /usr/bin/bwrap0755 uid0 sans setuid/setgid, workerUID1002. Preuve /Users/moi/.cache/bridget-delivery162-20261013/cancel162-host-prerequisites.json, SHA ca34e067dd50a2d43c0fb935385d23ef62a65b7ae09f7ef1518c57e4fed45597. Le correctif doit refuser avant fournisseur si les primitives requises manquent.

T045 corrige ; T046 porte les oracles Linux, la recette réelle et la revue ciblée. **44 tâches validées conservées, deux tâches ouvertes, In Progress 44/46.** Aucun résultat de ces tâches anticipé. Si le wrapper meurt après RELEASE sans preuve, Unknown et la capacité peuvent rester retenus sans limite ; aucun outil force-clear ni arrêt prouvé inventé. Les trois routes nominales réussies restent acquises, mais **G8 demeure ouvert sur l’annulation, G9 reste ouvert**. Les convergences et audits antérieurs restent historiques sur leurs sources.

## Correctif G8 — revue R2 et limites opérationnelles actuelles

La revue ciblée /Users/moi/.cache/bridget-delivery162-20261013/cancel162-code-review-r2.md est **APPROVE**, sans exécution de test par le reviewer. R1 avait demandé le cas normal sensible à TERM, la poursuite des autres signaux après enfant douteux et l’ordre de lecture de preuve ; R2 constate ces corrections dans le code. Le runner Linux R1 a été refusé avant compilation ; R2 compile actuellement. Aucun test réussi ni recette d’annulation corrigée n’est attesté ici. T045/T046 restent non cochées,44/46.

L4 : si subreaper/pidfd ne peut être armé avant le fournisseur mais que RELEASE est déjà possible, le worker conserve **Unknown et le quota**. Ce n’est pas un refus propre. Si fsync du répertoire échoue après rename, un retry identique peut accepter la preuve sans refaire ce fsync ; la durabilité dans cette fenêtre de crash n’est pas garantie. Un Conflict de publication persistant garde le wrapper vivant et la capacité retenue. Une sortie contournant Drop laisse Unknown faute de preuve. Ces limites conservatrices sont acceptées comme non bloquantes par la revue ; aucune nouvelle procédure de force-clear ni garantie de confinement après mort du wrapper n’est introduite.

ROOT rapporte l’arrêt propre du parent Opus2D à16:45 par commande f1e09965-7238-47ce-a882-8b497604ca5c. Son enregistrement est conservé ; la relance pour recette reste à exécuter. Les trois succès nominaux précédents ne ferment toujours pas le gate d’annulation. **G8/G9 restent ouverts.**

## Procédure préparatoire et chronologie historiques

**Les sections suivantes conservent leurs états au moment de rédaction.** Les mentions NON EXÉCUTÉ, configurations désactivées, revues en attente ou absence d’installation ne décrivent plus l’état courant lorsqu’elles sont remplacées par le tableau ci-dessus. Elles servent de procédure et de trace ; elles ne doivent pas être rejouées automatiquement. Les opérations ont été exécutées par ROOT, jamais par le rédacteur de ce reçu.

**Date de préparation :** 13 octobre 2026. **Responsable des opérations :** ROOT. **G8 :** ouvert. **G9 :** ouvert.

Ce document décrit les opérations à faire et les preuves à recueillir. Il n’atteste aucun commit de livraison, build de release, installation, activation, mission distante ou nettoyage 162. Les résultats de développement restent dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/validation/tests162.md. Aucune commande de cette procédure n’a été lancée par son rédacteur.

## Décisions fermes et points encore ouverts

- Bridget conserve un seul maître. Le worker Linux n’est pas une seconde base métier. Le pipeline autorise l’activation contrôlée de Bridget 162 après ses gates ; il n’autorise aucune relance T3.
- L’utilisateur a autorisé les projets Bridget, Roadie et Opus2D. Cartae est exclu de cette activation. Les trois clones Linux et leurs identités Git sont attestés dans provision-projects162.final.json ; les mappings privés sont préparés et désactivés. Codex login status retourne RC0, sans preuve de quota ni appel réel de modèle. Les routes restent désactivées jusqu’aux contrôles G8. Préserver /home/moi/20.cartae et sa modification existante.
- Le CLI worker T006 est écrit, compilé et exécuté dans les quatre E2E R3 réussis. ROOT confirme également les quatre E2E PASS dans la FULL unique ; cela reste distinct de son verdict global RC101 et d’une activation de production. **ExecStart doit viser le préfixe Linux162 réellement livré, puis être vérifié avant installation de l’unité.** Les identités candidates sont cartae/worker-cartae/révision 1 ; la socket et les configurations installées restent à réattester en G8.
- Les commandes de fermeture des admissions et de lecture des missions sont précisées en section2. Les chemins DB/Fleet/registre du maître sont observés ci-dessous et doivent être réattestés au préflight final. Les PID historiques ne sont jamais des cibles d’arrêt. L’absence de ces éléments bloque l’activation, pas le développement.
- Fleet écrit le schéma 5 ; les requêtes stockées sont strictes. Un ancien binaire ne doit pas lire l’état 162 inchangé. La compatibilité de lecture Lineage v1 ne signifie pas compatibilité des stores avec un ancien binaire.

## Sources de procédure réemployées

- /Users/moi/.cache/bridget-delivery162-20261013/validation-commands.md : commandes préparatoires, explicitement non exécutées. Le budget cache de 20 Gio de cette ancienne note est remplacé par 45 Gio ci-dessous.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/validation/preflight162.md : propriété des ressources, prérequis et limites des observations historiques.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/plan.md : autorité, supervision et rollback strict.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/160-reprise-quota/validation/livraison160.md : staging signé, activation avec identité de processus et sauvegarde. Les valeurs160 sont historiques, pas à rejouer.
- /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/161-client-cartae/validation/livraison161.md : manifest Linux, provenance client, préfixe neuf, lien atomique et conservations justifiées.

## 1. Figer les sources et les conditions de départ — NON EXÉCUTÉ

Après développement vérifié, Converge, contre-revue de code et audit requis :

1. Enregistrer les commits de sources Bridget et T3, les branches et le statut des deux worktrees. Ne pas mélanger SHA du binaire, SHA du paquet et commit documentaire final.
2. Intégrer Bridget vers github/main et T3 vers fork/local/main-20261009 avec les hooks normaux et l’identité utilisateur. Vérifier les références distantes. Aucun push upstream ni PR T3.
3. Construire depuis les sources intégrées propres. Les worktrees de développement ne deviennent pas une preuve de provenance du binaire installé sans correspondance exacte.
4. Relever les PID et naissances actuels du maître, du pont, du tunnel et de T3. Relever leurs binaires et configurations effectives sans secrets. Photographier l’état via des preuves textuelles privées ; aucune capture d’écran requise.
5. Enregistrer le compteur et les identités de missions actives, préparées, en quota, en attente humaine, avec résultat en attente, inconnues et à reprendre. Une commande status réussie ne suffit pas à prouver la quiescence métier.

Preuves attendues : commits et références distantes, diff résiduel expliqué, inventaire des processus, état de missions et horodatage. Aucun emplacement futur de preuve n’est présenté comme fichier déjà créé.

## 2. Fermeture des admissions et sauvegarde jointe — NON EXÉCUTÉ

Cette étape porte sur un ensemble cohérent, pas sur des copies prises pendant des mutations concurrentes.

1. Capturer d’abord control status et conserver l’état initial des admissions. Fermer les nouvelles admissions avec halt-delegations --reason livraison162, sans --cancel-active. Garder les machines162 désactivées. Vérifier par lecture et essai synthétique borné qu’aucune nouvelle mission ne part. Ne pas remplacer cette garde par l’effacement d’une configuration ou d’une file.
2. Attendre ou réconcilier les missions vivantes selon les règles 152–160. Un `unknown`, une pause, un résultat non confirmé ou un bootstrap non libéré n’est pas une fin. Préserver les preuves et ne pas forcer un arrêt pour obtenir un compteur nul.
3. La condition normale de bascule est une quiescence attestée : aucune admission en vol, aucun writer métier actif pendant la copie, aucune mission dont une reprise pourrait être perdue. Si des missions doivent traverser la bascule, ROOT doit documenter la procédure déjà testée de reprise et leurs identités ; cette note ne l’invente pas. Sans preuve suffisante, différer l’activation en conservant les sources livrées.
4. Créer une racine de sauvegarde neuve et privée, enregistrer son chemin absolu, ses modes et son propriétaire. Capturer ensemble : base principale et bases annexes réellement ouvertes, état Fleet, registre privé et révisions, état worker existant ou preuve de son absence, unités/configurations actives et binaires de retour. Copier les secrets uniquement dans la sauvegarde privée, jamais dans les journaux ou le dépôt.
5. Pour SQLite, utiliser la sauvegarde cohérente déjà employée par le projet ou copier après fermeture attestée des writers et checkpoint contrôlé. Ne pas copier seulement le fichier principal d’une base en WAL encore active. Inclure les sidecars nécessaires selon la méthode retenue ; tracer cette méthode.
6. Pour le worker, un arrêt du superviseur ne prouve pas l’arrêt des producteurs de blobs. Attester leur fin ou différer la copie cohérente. Si l’état n’existe pas encore, conserver cette preuve d’absence au lieu de fabriquer un store vide.
7. Vérifier les SHA des fichiers copiés, les schémas, quick_check et foreign_key_check sur la copie cohérente. Vérifier également le manifeste de l’ensemble Fleet/registre/worker et sa période sans écriture. Aucun corps de mission ou credential dans le reçu public.

Commandes canoniques relues par ROOT, à exécuter lors du préflight réel seulement :

```bash
/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget control status
/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget control halt-delegations --reason livraison162
/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget control delegations list --state active --limit 100 --json
/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget control status
```

La liste est bornée à 100 ; elle ne prouve pas l’absence totale de missions. Compléter la quiescence par les snapshots ou requêtes SQL en lecture seule sur les stores actifs, dont les chemins restent à attester. Préserver toutes les catégories non soldées citées plus haut. Ne pas utiliser --cancel-active pour simplifier la bascule.

Après activation vérifiée seulement, réouvrir avec la commande ci-dessous si les admissions étaient ouvertes au préflight et si aucun arrêt humain postérieur n’a remplacé celui de livraison. Si un arrêt humain était déjà actif, le conserver ; ne pas exécuter cette reprise automatiquement.

```bash
/Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget control resume-delegations
```

Table à remplir au préflight réel :

| Élément | Chemin actif absolu vérifié | Copie privée absolue et SHA | Contrôle |
|---|---|---|---|
| Base(s) métier et sidecars applicables | /Users/moi/.cache/bridget-core/bridget.db observée ; annexes/sidecars à attester | Non créée | Non exécuté |
| Fleet et schéma | /Users/moi/.cache/bridget-core/bridget.fleet.json, 14 583 octets, mode 0600 observé | Non créée | Non exécuté |
| Registre maître privé | /Users/moi/.cache/bridget-core/agents.json, mode 0600 observé | Non créée | Non exécuté |
| État worker | /home/moi/.local/share/bridget-worker/cartae ou absence attestée | Non créée | Non exécuté |
| Credential worker, si déjà présent | /home/moi/.config/bridget/worker-cartae.credential | Copie privée seulement, aucune valeur exposée | Non exécuté |
| Binaire maître et configurations de services | À relever dans les processus réels | Non créée | Non exécuté |
| Application T3 actuelle | /Applications/T3 Code (Local).app | Non créée | Non exécuté |

## 3. Build, signature et installation Bridget — NON EXÉCUTÉ

Une seule compilation active, y compris le composant Rust du paquet T3. Cargo jobs 2, nice 10 et deux threads pour les validations. Le cache debug utile 162/163 est conservé sous budget temporaire de rétention de 45 Gio ; ce budget ne bloque pas mécaniquement la compilation. Ne pas purger tout le profil pour revenir arbitrairement à 20 Gio.

Créer un stage privé neuf avec le mécanisme déjà reconnu par /Users/moi/Nextcloud/10.Scripts/64.bridget/scripts/build.py. Enregistrer son chemin avant le build. La commande de release doit viser ce stage, pas écraser directement le binaire installé sous target/release.

Commande de build préparée ; BRIDGET_STAGE doit être le chemin réel du stage privé neuf :

```bash
env CARGO_TARGET_DIR="$BRIDGET_STAGE" CARGO_BUILD_JOBS=2 CARGO_NET_OFFLINE=true RUSTUP_AUTO_INSTALL=0 BRIDGET_BUILD_CACHE_GIB=45 BRIDGET_BUILD_CACHE_DAYS=7 nice -n 10 python3 /Users/moi/Nextcloud/10.Scripts/64.bridget/scripts/build.py cargo build --locked --offline --jobs 2 --release -p bridget-daemon --bin bridget
codesign --force --sign - "$BRIDGET_STAGE/release/bridget"
codesign --verify --strict "$BRIDGET_STAGE/release/bridget"
shasum -a 256 "$BRIDGET_STAGE/release/bridget"
```

Ne pas exécuter avec une variable vide. Relever commit source, sortie Cargo, durée, RC, BuildID réellement embarqué et SHA. Réemployer le remplacement atomique gardé de 160 seulement après sauvegarde et contrôle d’identité du fichier installé. Relire le SHA et la signature de /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget après échange. L’activation maître/pont se fait ensuite, avec les identités de processus fraîches et les contrôles de reprise. Aucun arrêt T3 ni tunnel déduit de cette installation.

## 4. Binaire Linux et lien client — NON EXÉCUTÉ

Réemployer le runner livré161, depuis un checkout propre du SHA intégré 162. Choisir un préfixe neuf ; ne pas écraser /home/moi/.local/lib/bridget-communication/161-e6812e3e7b2d. Les variables ci-dessous doivent recevoir leurs valeurs absolues et le SHA complet réellement vérifié :

```bash
bash /Users/moi/Nextcloud/10.Scripts/64.bridget/scripts/deploy-remote.sh --label cartae162 --host 192.0.2.10 --user moi --port 2222 --identity /Users/moi/.ssh/id_ed25519 --known-hosts /Users/moi/.ssh/known_hosts --source "$SOURCE162" --remote-prefix "$REMOTE_PREFIX162" --remote-cargo /home/moi/.rustup/toolchains/1.92.0-x86_64-unknown-linux-gnu/bin/cargo --expect-commit "$SHA162"
```

Le runner contrôle manifest, ensemble exact de sources, préfixe neuf, locked/offline, jobs2/nice10 et installe dans PREFIX/bin/bridget. Enregistrer manifest et SHA avant tout nettoyage du target. `--version` ne prouve pas le BuildID client ; `status` rapporte le maître. Réemployer la preuve Cargo/chaîne embarquée/SHA de 161.

Tester le candidat par son chemin absolu avant le lien global. Si le lien client doit être changé, réemployer le bloc atomique du contrat 161 et comparer la cible attendue juste avant l’échange ; un changement concurrent constaté entraîne un refus. Conserver le préfixe161 et le rollback096 utiles. Ne pas annoncer une transaction CAS contre un processus non coopérant.

## 5. Provisionnement et supervision worker — NON EXÉCUTÉ

Prérequis historiques : systemd --user joignable et Linger=yes ; unité absente lors du préflight. Rafraîchir ces observations juste avant installation. Le tunnel 095 reste maintenu sur le Mac ; aucun tunnel par mission ni relance de ce tunnel.

1. Contrôler propriétaire et absence de symlink/collision de /home/moi/.config/bridget/worker-cartae.credential et /home/moi/.local/share/bridget-worker/cartae. Provisionner credential en 0600 et état en 0700, sans changer les modes des répertoires partagés de l’utilisateur.
2. Le credential reste dans le fichier privé et la seule mémoire du worker. Ne pas l’injecter en argv, Environment, wrapper ou fournisseur. Le registre maître conserve son empreinte et sa révision. Aucun secret brut dans les reçus.
3. Préparer /home/moi/.config/systemd/user/bridget-worker-cartae.service. L’unité doit contenir les propriétés ci-dessous. Ce fragment est un contrat de procédure, pas un fichier de service créé ou installable sans ExecStart final :

```ini
[Service]
# ExecStart : à renseigner depuis T006 et le binaire Linux versionné réellement livré.
Restart=on-failure
KillSignal=SIGTERM
KillMode=process
SendSIGKILL=no
```

Le point d’entrée compilé est run_worker(WorkerRuntimeConfig). Les sept options CLI sont requises : machine-id, worker-id, provisioning-revision, credential-file, state-root, registry-file et master-socket. Le passage R1 est historique et avait échoué. Les quatre E2E R3 passent maintenant, dont le nominal avec maître réel ; ROOT confirme leur réussite dans la FULL. Aucune installation Linux ou activation de route n’en est déduite.

Préparer la ligne ExecStart depuis le commit source de release vérifié. Le préfixe doit être /home/moi/.local/lib/bridget-communication/162-SHA12, où SHA12 désigne les douze premiers caractères du SHA complet. Le shell de préparation développe les variables ; systemd doit recevoir une ligne absolue littérale, jamais une variable non développée. Les identités candidates privées sont figées : machine cartae, worker worker-cartae et révision 1. Elles sont distinctes des fixtures. Seul le SHA de release reste à fournir depuis la source intégrée :

```bash
[[ "${SHA162:?SHA de release requis}" =~ ^[0-9a-f]{40}$ ]] || exit 1
MACHINE162=cartae
[[ "$MACHINE162" =~ ^[a-zA-Z0-9_-]+$ ]] || exit 1
WORKER162=worker-cartae
[[ "$WORKER162" =~ ^[a-zA-Z0-9_-]+$ ]] || exit 1
REVISION162=1
[[ "$REVISION162" =~ ^[1-9][0-9]*$ ]] || exit 1
REMOTE_PREFIX162="/home/moi/.local/lib/bridget-communication/162-${SHA162:0:12}"
printf 'ExecStart=%s/bin/bridget worker --machine-id %s --worker-id %s --provisioning-revision %s --credential-file /home/moi/.config/bridget/worker-cartae.credential --state-root /home/moi/.local/share/bridget-worker/cartae --registry-file /home/moi/.config/bridget/agents.json --master-socket /home/moi/.cache/bridget-core/bridget.sock\n' "$REMOTE_PREFIX162" "$MACHINE162" "$WORKER162" "$REVISION162"
```

Ce bloc prépare le texte seulement ; il ne crée ni service ni credential. Vérifier sur Cartae le binaire, son SHA et son BuildID, le registre privé /home/moi/.config/bridget/agents.json et la socket fédérée /home/moi/.cache/bridget-core/bridget.sock avant dépôt de l’unité. Le chemin de socket est la cible prévue à réattester ; sa présence historique ne prouve pas le raccordement courant. Ne pas substituer le lien client mutable à ce binaire versionné. En cas de divergence de chemin ou d’identité, arrêter la préparation de l’unité jusqu’à correction documentée.

4. Réutiliser l’unité candidate et les valeurs existantes de supervision. Relever RestartSec et les valeurs StartLimit effectives en G8, y compris les valeurs héritées par défaut ; ne pas inventer de nouvelle borne pour ce lot. Vérifier l’unité et son état inactif avant de la démarrer.
5. Après dépôt et contrôle seulement, ROOT pourra exécuter sur Cartae :

```bash
systemctl --user daemon-reload
systemctl --user enable --now bridget-worker-cartae.service
systemctl --user show bridget-worker-cartae.service -p LoadState -p ActiveState -p SubState -p MainPID -p FragmentPath -p KillMode -p KillSignal -p SendSIGKILL -p Restart -p RestartUSec -p StartLimitIntervalUSec -p StartLimitBurst
```

6. Vérifier le handshake authentifié, la nouvelle époque et la réconciliation avant admission. Une connexion remplacée doit produire une trace sans secret. Un service active n’est pas une mission réussie.

KillMode=process est un choix exceptionnel : les wrappers survivent au superviseur. La recette isolée avec enfant synthétique doit prouver SIGTERM du worker, maintien de l’enfant, reprise du verrou/store et réconciliation sans doublon. Ne pas remplacer ce choix par control-group ou mixed, qui modifierait le contrat. Un bootstrap marqué mais non libéré doit sortir lorsque son canal RELEASE est fermé ; attester sa fin. Un arrêt worker n’annule pas ses missions.

Les modes 0600/0700 ne constituent pas une séparation entre processus du même UID. Les guards de chemins protègent credential/répertoire privé/état contre les autorisations explicites ; ils ne confinent pas dangerFullAccess. Cette limite reste inscrite dans le reçu final.

## 6. Activation des routes et recette réelle — NON EXÉCUTÉ

Avant tout enabled=true : appliquer les trois mappings candidats Bridget, Roadie et Opus2D seulement, puis réattester leurs racines/cwd/identitéGit/basecommit depuis le reçu final de provisionnement. L’authentification Codex est vérifiée par login status RC0 dans le reçu privé ; aucun quota ni appel de modèle réel n’est prouvé. Conserver le chemin de fournisseur sélectionné dans le candidat, sans le remplacer par une résolution PATH différente. Ne pas toucher à la modification existante dans /home/moi/20.cartae.

Sur chaque route réellement autorisée, exécuter une mission synthétique bornée avec machine et moteur explicites. Recueillir tâche/mission/ronde/instance/exécution/génération, lieu réel, cwd, source, résultat corrélé unique et disposition durable. Tester une annulation attestée et le retour du parent absent. Vérifier journal froid et suivi vivant, puis contact perdu signalé sans résultat inventé. La recette ne donne pas à elle seule une preuve de livraison Git des travaux de l’agent : dépôt, branche, commit et tests restent distincts.

Le choix de projets est reçu pour Bridget, Roadie et Opus2D. Laisser leurs routes désactivées jusqu’à validation des mappings et prérequis réels. Cartae reste exclu. Développement et recettes isolées peuvent être complets ; SC-001 réel et l’activation concernée restent non attestés tant que la recette n’a pas eu lieu.

## 7. Paquet T3, signature et installation sur disque — NON EXÉCUTÉ

Sources intégrées sur fork/local/main-20261009, paquet depuis une copie privée du commit. Réemployer /Users/moi/11.Repositories/t3code-local/scripts/build-desktop-artifact.ts et /Users/moi/.cache/bridget-delivery149-tools/sign-stage.py. Le script 149 de build complet contient une version et des opérations anciennes : ne pas le relancer tel quel.

Reprendre les commandes paramétrées de /Users/moi/.cache/bridget-delivery162-20261013/validation-commands.md avec version 162, source et répertoire de paquet réellement figés. Vérifier les caches natifs et la provenance du resource-monitor ; ne pas lancer un second Cargo en parallèle. Signer via le helper existant, puis vérifier signature et entitlements.

Sauvegarder /Applications/T3 Code (Local).app dans une destination privée neuve et contrôler la copie. Déposer le candidat sous /Applications/.T3 Code (Local).app.new-162 seulement si ce chemin n’existe pas déjà ; contrôler signature et contenu. Utiliser renamex_np avec RENAME_SWAP=2 après contrôle du candidat et du rollback. La sonde /Users/moi/.cache/bridget-delivery162-20261013/t3-atomic-swap-probe.json prouve la permutation puis le retour sur deux répertoires synthétiques du même volume sous /Applications ; aucun bundle réel n’a été touché. Ne pas prétendre que le script149 contenant cette primitive a été retrouvé. Comparer les identités des deux chemins juste avant l’échange et refuser un changement constaté. Ne pas inventer une commande mv non contrôlée. Relever SHA app.asar/Info.plist, version et signature après installation.

Comparer PID et naissances T3 avant/après : ils doivent rester inchangés. Aucun `open`, kill, restart, launchctl, activation différée ou écriture de base T3. Le nouveau rendu n’est attesté que par tests isolés tant que l’utilisateur n’a pas autorisé une relance. L’ancien reader doit continuer à recevoir v1.

## 8. Retour arrière strict — NON EXÉCUTÉ

Deux opérations distinctes : désactiver la route sous 162, ou restaurer un ensemble antérieur.

- **Route désactivée sous 162 :** fermer les nouveaux lancements, préserver les exécutions et résultats, réconcilier les identités connues. Arrêter le worker seul si nécessaire ne détruit ni blobs ni reçus. Ce repli est requis si une restauration perdrait des données nouvelles.
- **Retour à l’ancien ensemble :** seulement après fermeture des admissions, quiescence et missions réconciliées. Archiver d’abord DB/Fleet/registre/worker et preuves post-162. Démontrer qu’aucune activité ou résultat nouveau ne sera perdu. Restaurer ensemble sauvegarde métier, Fleet, registre, état worker applicable et binaires/configurations compatibles, jamais une seule base ou un seul binaire. Vérifier SHA, schémas et intégrité avant réouverture.

Si cette démonstration manque, rester sur162 avec route désactivée. Un unknown, un résultat différé ou un blob encore publiable n’est pas supprimé pour faciliter le rollback. Aucune commande de reset de preuve, purge de base ou downgrade sur stores 162 n’est proposée.

## 9. Nettoyage et clôture G9 — NON EXÉCUTÉ

| Ressource | Décision préparée | Condition avant action |
|---|---|---|
| /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante | Retrait sans force après intégration | Propre, référence poussée, aucun processus utilisateur, preuves copiées dans le checkout durable. |
| /Users/moi/11.Repositories/t3code-local/.worktrees/162-lineage-distant | Retrait sans force après intégration | Même contrôle, sans arrêter T3. |
| Branches session-162-execution-distante et session-162-lineage-distant | Retrait lorsqu’inutiles | Fusion et push attestés, aucun travail non récupéré. |
| Les 17 worktrees T3 préexistants listés dans le préflight | Conservation hors périmètre | Aucun mandat de nettoyage de ces travaux. |
| TMP et stages réellement créés pour 162 | Inventaire puis retrait ciblé ou conservation justifiée | Chemins exacts, propriétaire, absence d’usage, archives SHA vérifiées ; aucune suppression globale de TMP. |
| /Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug | Conservation utile 162/163 | Budget temporaire45Gio ; nettoyage par build.py seulement après dernière réutilisation et gardes satisfaites. |
| /home/moi/.cargo/registry | Conservation utile | Cache partagé ; aucune purge générale. |
| Préfixe161, rollback096 et sauvegardes160/161 utiles | Conservation | Ne pas détruire un retour ou une preuve encore nécessaires. |
| État et blobs worker 162 | Collecte contractuelle seulement | Disposition finale durable, producteur terminal, commande/slot irrévocablement clos ; reçu compact/digest/disposition 7 jours conservés. |
| /Users/moi/.cache/bridget-delivery162-20261013 | Archivage durable des preuves utiles | Relire SHA avant retrait de copies temporaires ; pas de faux gain physique APFS. |

Un échec lsof ou un verrou présent interdit le nettoyage concerné ; documenter la conservation, sans contourner la garde. En 161, le target distant a été conservé pour cette raison : cet historique n’autorise pas sa purge forcée en 162. Un worker injoignable, une branche non récupérée ou un résultat non confirmé restent conservés.

Avant retrait des worktrees, copier ce reçu et les preuves utiles vers /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/162-execution-distante/validation/livraison162.md et l’archive durable choisie par ROOT. Le chemin de worktree devient ensuite historique. Commiter/pousser les métadonnées finales et vérifier les refs distantes. Ne pas inventer le SHA du commit contenant ce même reçu.

## Tableau de reçu à compléter après opérations

| Preuve | État à cette rédaction |
|---|---|
| Source Bridget intégrée/poussée et référence distante | NON EXÉCUTÉ |
| Source T3 intégrée/poussée et référence distante | NON EXÉCUTÉ |
| Sauvegarde jointe cohérente et contrôles d’intégrité | NON EXÉCUTÉ |
| Stage Bridget signé, binaire installé et BuildID | NON EXÉCUTÉ |
| Build Linux, manifest et SHA candidat | NON EXÉCUTÉ |
| CLI worker exact et unité utilisateur validée | CLI compilé/exécuté en fixture ; unité de production NON EXÉCUTÉE |
| Activation Bridget/worker et réconciliation | NON EXÉCUTÉ |
| Projet autorisé et mission distante réelle | Bridget, Roadie et Opus2D autorisés ; mappings à attester, mission réelle NON EXÉCUTÉE ; Cartae exclu |
| Paquet T3 signé/installé, runtime T3 inchangé | NON EXÉCUTÉ |
| Nettoyage de chaque ressource ou conservation motivée | NON EXÉCUTÉ |
| Commit/push des métadonnées finales | NON EXÉCUTÉ |

Ce tableau est volontairement vide de SHA/PID 162 futurs. La rédaction de T041 prépare la livraison ; seule l’exécution attestée de G8/G9 permettra de la déclarer achevée.


## T041 — préparation validée, opérations G8 encore NON EXÉCUTÉES

ROOT a relu le dossier de préparation. T032 et T039 sont validées ; le quickstart est synchronisé. T041 peut être cochée comme préparation, sans cocher G8/G9 ni prétendre que l’unité finale est installée.

1. Les commandes d’admissions, chemins des stores, sauvegarde jointe et rollback strict sont décrits. G8 doit encore réattester les writers, prouver la quiescence et produire les copies ainsi que leurs empreintes.
2. Les identités cartae/worker-cartae/révision 1, les trois mappings et l’authentification Codex sont préparés et attestés dans les reçus privés. G8 résout le préfixe depuis le SHA intégré, réatteste registre/socket, valide l’unité et relève ses valeurs RestartSec/StartLimit effectives, sans inventer de limites nouvelles.
3. La survie de l’enfant pendant arrêt du worker, le blob durable, la reprise et l’ACK perdu sont prouvés par les E2E R3. Leurs quatre réussites sont confirmées dans la FULL, dont le verdict global reste RC101. La recette réelle sur les trois projets autorisés reste à exécuter après installation.
4. Le quickstart et le reçu décrivent les mêmes conditions. Bridget, Roadie et Opus2D sont autorisés ; Cartae est exclu. Les routes restent désactivées. Les gates de Converge, revue/audit, installation et activation restent obligatoires avant déclaration de livraison.

L’inventaire TMP/worktrees/branches, le rollback joint sans perte, les conservations utiles et l’interdiction de relancer T3 restent décrits ci-dessus. Le gel de sources et le formatage ne prouvent pas la livraison.


Checkpoint de ressources transmis par ROOT avant le groupe R2 : cache debug 37,90 Gio logiques et volume libre 30,88 Gio. La rétention temporaire passe de 40 à 45 Gio pour cette validation 162/163. build.py peut retirer un profil entier après build ; ce budget évite de purger le debug aussitôt reconstruit. Il s’agit d’un plafond de rétention temporaire, pas d’une cible de consommation ou d’un gain physique. Le nettoyage final reste soumis à l’inventaire et aux gardes, avec conservations motivées. Ce checkpoint de ressources est historique. Le reçu final de provisionnement atteste désormais les trois clones ; les mappings candidats sont préparés et aucune route n’est activée.


## Préflight observé avant activation — état courant, pas quiescence future

ROOT a exécuté status : build maître e6812e3e7b2d (160), socket /Users/moi/.cache/bridget-core/bridget.sock et DB /Users/moi/.cache/bridget-core/bridget.db. Le registre maître réel est /Users/moi/.cache/bridget-core/agents.json en0600, car config_path suit Namespace.root. Il est distinct du registre worker prévu /home/moi/.config/bridget/agents.json ; ne pas écrire le candidat maître dans un chemin .config supposé.

La lecture /Users/moi/.cache/bridget-delivery162-20261013/runtime162-delegations-preflight.json retourne halted=false, items=[] et total_active=0. Ces faits valent à cette observation seulement. Ils n’autorisent pas une copie incohérente ni une réouverture automatique après un arrêt humain postérieur. La sauvegarde jointe, ses annexes et la quiescence finale restent à établir au moment de bascule.

Les trois copies Linux autorisées sont préparées selon /Users/moi/.cache/bridget-delivery162-20261013/provision-projects162.final.json. État propre, uid1002 et mode 0700 ; origine et SHA attendus vérifiés :

| Projet | Racine Linux attestée | Commit observé | Origine attestée |
|---|---|---|---|
| Bridget | /home/moi/10.Scripts/64.bridget | 84cd24305837b70f866ae09bff86235b9743071e | https://github.com/guthubrx/bridget.git |
| Roadie | /home/moi/10.Scripts/39.roadie | acc10e13c41026ce444fe38933840b1ea9e2c976 | https://github.com/guthubrx/roadie.git |
| Opus2D | /home/moi/10.Scripts/69.opus2D | 3e677ad669648f953bb0bd951b2dc72777b3a9fc | /home/moi/.local/share/bridget/git/opus2d.git |

Le dépôt nu Opus2D est au même commit et son contrôle de connectivité retourne RC0. Le reçu porte services_or_routes_changed=false. Ces copies ne sont pas une activation des mappings ni une livraison162. L’authentification Codex est attestée séparément par son reçu privé, sans preuve de quota. Le candidat de registre privé reste à installer puis activer sous les gates du présent reçu.


## Préparation T041 — sauvegarde jointe et candidats privés actualisés

Les chemins maîtres observés sont /Users/moi/.cache/bridget-core/bridget.db (environ 72 Mo, mode 0600), /Users/moi/.cache/bridget-core/bridget.fleet.json (14 583 octets, mode 0600) et /Users/moi/.cache/bridget-core/agents.json (mode 0600). ExecutionStore ouvre config.db_path : l’état d’exécution est dans cette même DB, pas dans un fichier supposé sous .config. ROOT a relu desired_state.rs:141–151 et daemon.rs:4885. Le préflight halted=false/total_active=0 reste historique ; il ne prouve pas l’absence future de writer.

Procédure cohérente retenue, encore NON EXÉCUTÉE :

1. Créer une racine privée neuve sous /Users/moi/.cache avec le préfixe bridget-install162-, enregistrer son chemin exact et son mode 0700. Pour l’état Linux, créer une racine de sauvegarde privée neuve sous /home/moi/.cache. Ce document ne fournit pas le nom d’une sauvegarde déjà créée.
2. Capturer l’état des admissions, les fermer sans annuler les missions, puis établir l’inventaire complet et la quiescence. Identifier les PID/naissances et tous les writers à partir des processus réels. Arrêter proprement les writers Bridget autorisés une fois la reprise sécurisée. Vérifier leur arrêt avant de copier ; ne pas toucher à T3 ni au tunnel. Si un writer ou producteur de blob reste incertain, différer la sauvegarde cohérente.
3. Après fermeture attestée des writers, utiliser la sauvegarde SQLite en lecture de /Users/moi/.cache/bridget-core/bridget.db vers une destination neuve de la racine privée. Cette méthode intègre les données visibles dans le WAL ; ne pas copier seulement bridget.db pendant qu’un WAL actif contient encore des transactions. Conserver dans le reçu les sidecars observés et la méthode choisie. Copier Fleet et registre durant la même période sans écriture. Les éventuelles bases annexes effectivement ouvertes doivent être inventoriées et traitées de même, sans en inventer une liste.
4. Sauvegarder ensemble l’état worker /home/moi/.local/share/bridget-worker/cartae, le registre worker et le credential s’ils existent, après fin attestée de tous leurs écrivains. Un service worker arrêté ne prouve pas la fin des wrappers. Si l’état worker n’existe pas, conserver la preuve d’absence ; ne pas créer un store vide pour le sauvegarder. Les secrets restent uniquement dans la sauvegarde privée.
5. Vérifier sur les copies SQLite quick_check=ok et foreign_key_check sans ligne, puis vérifier les SHA, modes et propriétaires des copies Fleet/registre/worker et des binaires de retour. Conserver un manifeste joint horodaté et la preuve de la période sans writer. Réattester les sources avant remplacement. Un écart arrête la bascule ; aucune sauvegarde fragmentaire n’est présentée comme cohérente.
6. Le retour à une version antérieure restaure cet ensemble uniquement si aucun état post-162 n’est perdu. Sinon conserver 162 et désactiver les nouvelles routes, en préservant toutes les preuves et mutations postérieures. La réouverture des admissions reste conditionnée à l’état initial et à l’absence d’arrêt humain intervenu depuis.

Candidats de configuration préparés, pas installés : /Users/moi/.cache/bridget-delivery162-20261013/worker-config-candidate, racine 0700 et dix fichiers 0600 selon l’owner. Le reçu /Users/moi/.cache/bridget-delivery162-20261013/worker-config-candidate/candidate-receipt.json est relu : status=candidate_only_not_installed, routes_enabled=false ; machine cartae, worker worker-cartae, révision 1, allowed_agent_types=[codex]. SHA-256 du reçu : 14d23b586e92d19c9ff47343cc9130d62ed9430fbbacec27956907fee7c40370. Aucune valeur de credential ni contenu complet des registres n’est publié.

Le candidat maître dérive du registre réel /Users/moi/.cache/bridget-core/agents.json ; le candidat worker dérive de /home/moi/.config/bridget/agents.json. Les définitions existantes sont conservées. Les trois mappings autorisés Bridget/Roadie/Opus2D sont préparés ; aucun enabled=true n’est installé. Le credential reste dans le stage privé. L’unité candidate contient encore un préfixe de release à résoudre depuis le SHA intégré. L’owner atteste trois lectures de comparaison et la vérification post-copie des sources ; ROOT devra les refaire avant installation pour ne pas écraser un changement concurrent.

L’authentification Codex a donné login status RC0 selon /Users/moi/.cache/bridget-delivery162-20261013/worker-config-candidate/codex-auth-status.json. Cela ne prouve ni quota disponible ni appel de modèle. L’activation future doit rendre cohérents les deux registres puis démarrer le service validé ; la recette réelle reste un gate séparé. Cartae demeure exclu des projets de travail autorisés.

La FULL unique est terminée : RC101, 2 613 PASS/122 FAIL/68 ignorés sur 97 cibles selon le parser conservé ; aucun GREEN global. Les replays ciblés et les correctifs relus sont validés ; T040 et T042 sont soldées. Les 122 échecs de la FULL ont été rejoués PASS sans transformer le verdict global. Développement/préparation 42/42 ; Converge n’a pas encore été exécuté. La relecture ROOT du présent dossier valide T041 comme préparation seulement. La vérification et l’installation de l’unité finale restent G8.


## Clôture documentaire de préparation — 42/42, livraison encore NON EXÉCUTÉE

ROOT a rapproché la lecture finale et les replays : 83 PASS dans le dernier groupe, quatre défauts corrigés, 30 FR/12 SC mappés. T017/T019/T020/T040/T042 sont validées. Les 122 échecs de la FULL historique ont un rejeu PASS ; la FULL garde son RC101, sans succès global inventé. Les limites T3 (tests isolés, types web étrangers, aucun redémarrage) restent celles du registre de tests.

Cette clôture prépare Converge, la contre-revue/audit et les gates G8/G9. Elle ne produit aucun commit de livraison, binaire installé, configuration activée, mission distante réelle ou nettoyage final. Les candidats privés restent désactivés ; toute opération réelle suit la procédure et fournit ses propres reçus.

Checkpoint courant : Converge1 est clos non convergé sur une trace de remplacement worker manquante (T008). ROOT a ajouté T043, correctif et validation encore attendus. Le développement repasse In Progress, 42/43 ; le checkpoint 42/42 précédent est historique. Aucun changement aux gates G8/G9 ni activation. Attendre la preuve et le GO ROOT avant clôture 43/43.

État courant après GO ROOT T043 : trace worker corrigée et oracle ciblé réussi, développement/préparation Implemented, 43/43. Converge1 reste historiquement non convergé ; Converge2 ciblé doit encore statuer. Les gates G8/G9 restent NON EXÉCUTÉS ; aucune activation, installation ou livraison finale déduite de 43/43.

État courant : contre-revue code R1 BLOCKED, développement rouvert à 43/44 par T044 (réconciliation starting/refus durable et E2E sans état terminal injecté). Converge2 reste historique ; aucun passage 3 encore, audit pré-R1 suspendu. Attendre correction et validations avant reprise des gates. Aucun changement de l’autorisation des projets, aucune installation ni activation.

## Checkpoint de préparation après T044 — opérations toujours NON EXÉCUTÉES

Développement/préparation 44/44 après R4/R5 ; Converge3 et les gates de revue restent à clôturer. Ce compteur n’atteste ni installation ni activation ni nettoyage G8/G9.

Pour la première connexion du worker, une route neuve enabled=false ne permet pas WorkerHello sans commande déjà connue. L’ordre opérationnel requis est : fermer les admissions globales avec halt-delegations ; attester la quiescence et sauvegarder conjointement DB/Fleet/registre/worker ; installer les configurations enabled=true pour Bridget, Roadie et Opus2D pendant que les admissions restent fermées ; activer maître et worker puis vérifier leur handshake ; rouvrir uniquement si les admissions étaient ouvertes avant l’opération et qu’aucun arrêt humain plus récent ne s’y oppose. Ne pas modifier l’authentification pour contourner ce provisionnement. Ces actions ne sont pas déclarées exécutées.

Catalogue Codex Cartae attesté par ROOT dans /Users/moi/.cache/bridget-delivery162-20261013/codex-model-list162.json : API 0.150.1 et modèle gpt-5.6-terra. L’effort est omis, conformément au registre efforts[]. Cela ne prouve ni exécution d’une inférence réelle ni disponibilité de quota.

## Gate préalable G8 après contre-revue R2 — ouvert

R2 est APPROVE_WITH_CHANGES ; D1 doit être corrigé et réattesté avant G8. Une erreur d’écriture RELEASE ou de mise en mode non bloquant après libération doit conserver l’autorité sur le Child jusqu’à son traitement sûr. Le replay R6 ciblé est attendu, sans résultat déclaré. Les 44 tâches planifiées et Converge3 ne suffisent pas à ouvrir ce gate.

Limite L1 à conserver dans le reçu final : un crash worker entre Created et le marqueur peut maintenir Unknown et le quota associé. Un bootstrap fermé ne permet pas de redispatcher la mission. Aucun essai de zombie Linux n’a été exécuté ; ne pas transformer l’inférence de la revue en résultat opérationnel. Installation, activation et nettoyage G8/G9 restent non exécutés à ce checkpoint.

## Clôture du préalable D1 — G8/G9 restent ouverts

Le préalable D1 est levé après R6, le replay exact R7 (1 PASS, RC0) et la relecture indépendante ciblée. La conservation du handle et le traitement failed=true en ReleaseAuthorized avec marqueur sont attestés ; failed=false reste interdit, sans promotion de phase. Les preuves finales et leurs SHA figurent dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/validation/tests162.md.

R2 demeure APPROVE_WITH_CHANGES avec réponse ciblée D1 close. Le scoring et le validateur d’audit restent en finalisation. Aucun Converge4 ; tâches figées à 44/44. G8/G9 et la limite L1 restent ouverts/applicables : aucune installation, activation ou clôture opérationnelle n’est déclarée dans ce checkpoint.

## Gate d’audit final conforme — 13 octobre 2026

L’audit final après correction D1 reçoit **A, 99,095238/100**, avec **0 CRITICAL, 0 HIGH, 3 MEDIUM et 1 LOW**. Le validateur termine avec **RC0, zéro erreur et zéro warning**. D1 est corrigé et réattesté par R6/R7 ; les trois MEDIUM et le LOW du rapport restent ouverts.

La couverture de lecture est de **7 898 lignes uniques sur 186 764 lignes du périmètre, soit 4,228866 %**. Il s’agit d’un échantillon des sources Bridget et T3 : les lectures antérieures ne sont reprises que sur les lignes inchangées, avec lecture fraîche des deltas D1. Cette note ne certifie pas tout le produit, ne remplace pas les résultats de tests et ne constitue pas un GREEN global.

Reçu relu : /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/audits/2026-10-13/session-2026-10-13-spec-162-01/validation-final.json. SHA-256 : 85884560f21e2c4029d98ad6bfeab479df676ca843b9d7ec8432ae51e6f00170. Le reçu date de 2026-10-13T12:56:07.662381+00:00. Les empreintes de grade.json et coverage.json ont été vérifiées conformes à ce reçu : e68398193480b829418ab9349adb8192ec8ef7bc5875c50c9156afd87e851bad et df5fa5b02f4822f5ab449f2f4769004dccbf7ce17878cb487ed7273bcb570694.

**Le gate d’audit est clos ; G8/G9 restent ouverts.** Le reçu indique explicitement runtime_delivery_approved=false. Git, installation, activation et nettoyage relèvent encore de ROOT. Les 44/44 tâches planifiées restent figées ; aucune livraison n’est déduite du score ni du validateur.


## Préparation de livraison du correctif T045

Les deux tests de bibliothèque R2 et les cinq E2E Linux R3 sont réussis. Le détail et les empreintes sont consignés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/162-execution-distante/specs/162-execution-distante/validation/tests162.md. La revue R2 est APPROVE ; l’annexe d’audit n’a aucun constat critique ou élevé. T045 est cochée, T046 reste ouverte.

Ces validations utilisent un snapshot 611bfa7557b9-dirty ; elles ne prouvent pas une installation du correctif. ROOT a intégré par avance rapide et poussé le commit source 1dc0c6022e8d0712fbc4c65a8c20be7b2ea8a437 sur github/main, avec égalité distante vérifiée (compte rendu ROOT vers 17:01 CEST). Ce commit contient les quatre fichiers sources du correctif. Le build Mac est en cours ; installation et recette réelle avec observation des descendants restent attendues. Les trois routes nominales précédentes restent attestées. G8 reste ouvert sur cette recette ; G9 reste ouvert sur le nettoyage final. Aucun SHA de futur commit n’est inventé.
