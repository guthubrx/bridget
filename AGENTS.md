## Incidents : corriger et livrer le guidage

Pour chaque incident remonté, vérifier la cause réelle. Tout défaut confirmé de
Bridget, y compris un message ou un conseil insuffisant, doit être corrigé puis
livré dans le binaire actif, avec les contrôles ciblés utiles et les sources
poussées. Une explication ponctuelle au demandeur ne suffit pas : le prochain
agent doit pouvoir agir avec le message reçu. Informer ensuite le demandeur du
correctif livré. Cette consigne permanente a été donnée le 24 octobre 2026.

## Builds : entretien automatique obligatoire

Pour toute compilation ou test Cargo de Bridget, utiliser `make build`,
`make release`, `make test` ou `python3 scripts/build.py cargo ...`, y compris
les tests filtrés. Ne pas lancer Cargo directement : cela contourne le nettoyage.
Le passage contrôle les caches avant et après Cargo, même en cas d'échec.
Réutiliser le cache de ce checkout ; ne pas copier un target par test ou par revue.
Si une isolation est nécessaire, utiliser un dossier `.cache/bridget-build-*`
du compte courant, via `CARGO_TARGET_DIR` ou `--target-dir`, pour son enregistrement.
Le budget commun des intermédiaires reconnus reste 10 Gio et 7 jours.
Après installation vérifiée hors staging, appeler la commande `installed` du
même script. Les exécutables installés et sauvegardes restent hors de ce budget.
Conserver la version active et une version de retour ; traiter les autres copies
identifiées à la livraison, sans supprimer de travaux non intégrés ni de preuves.

## Fin de livraison : Git et nettoyage obligatoires

À chaque livraison autorisée, sans nouvelle demande utilisateur :

- committer le travail validé avec l’identité Git existante, l’intégrer à `main`
  et pousser les sources ; vérifier que `main` est propre et aligné au distant
  sur les machines concernées ;
- retirer les worktrees et branches de travail devenus inutiles après vérification
  de l’intégration, de la clôture des tâches, des fichiers ignorés et des usages
  réels ; supprimer aussi les références distantes obsolètes vérifiées ;
- nettoyer les intermédiaires de compilation et les copies de binaires devenues
  inutiles ; conserver la version active et une version de retour vérifiée ;
- conserver tout travail non intégré, preuve, tâche active ou binaire encore
  utilisé ; annoncer le motif précis et reprendre son nettoyage à la livraison
  suivante ; ne pas terminer les processus d’un autre travail pour faire le ménage ;
- donner le bilan réel : commit poussé, éléments retirés et conservations.

Le nettoyage avant et après Cargo reste automatique avec le script existant.
Le contrôle de fin de livraison complète ce script : il ne transforme pas une
suppression de données ou de travaux en cours en entretien autorisé. Consigne
permanente confirmée par l’utilisateur le 24 octobre 2026.

## T3 comme référence de mise en œuvre

Pour un parcours déjà couvert par T3 (sessions, fournisseurs, Git/worktrees,
préparation et lancement), lire d'abord son implémentation dans
`/Users/moi/11.Repositories/t3code`. Réutiliser son expérience et ses mécanismes
éprouvés dans Bridget. Justifier les différences nécessaires au travail distant
ou aux permissions ; ne pas reconstruire par défaut un parcours plus complexe.
Vérifier le parcours utilisateur complet, pas seulement ses étapes isolées.

Pour CHAQUE défaut concret à corriger, rechercher le cas équivalent dans T3
avant de choisir la correction. Lire le code concerné, puis noter brièvement
le mécanisme réutilisable et les différences nécessaires dans Bridget. Si T3
ne couvre pas ce cas, le dire explicitement. Une référence générale à T3 ne
remplace pas cette lecture par défaut. Cette étape doit rester courte et utile
au correctif, sans nouvelle campagne de validation.

<!-- SPECKIT START -->
Session179 livrée le25octobre2026 :12/12, Implemented, avec conservations justifiées. Releasev0.6.0 publiée depuis533f9ec60fe6f6d94868ac18974507a35c7f4c07, six actifs retéléchargés conformes. Mac/Cartae/Pixel actifs0.6.0 ; auto-updateON, désactivable dans les paramètres. Pixel installé depuis GitHub via PackageInstaller avec confirmation, association conservée. Core1864895/worker1867403/mobile1865084/connecteur45047 ; T3650/975 inchangés. Python13PASS, Rust7/9/69PASS+1ignored, JVM18PASS, Pixel3PASS ; pas de FULL ni de contre-revue indépendante. Preuves : diaporama vérifié. Quatre anciens binaires Mac et quatre copies de validation retirés,225324587octets logiques ; worktree Mac179 retiré. Cartae179/branche/caches conservés car lsof fuse.portal incomplet. Tâche historiquee5959986 inchangée : update_missions_active_or_unconfirmed bloque une prochaine activation jusqu’à réconciliation. MCP anciens encore chargés, aucun rechargement ni identité réparée revendiqué. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/179-mises-a-jour-automatiques/validation/livraison.md. Les checkpoints suivants sont historiques.

Session170 livrée le16octobre2026 :8/8, Implemented, pipeline Livré avec conservations justifiées. Correctif des faux timeouts après résultat distant accepté. Source57f2fff31b1c23d015a483a5a97754130b1e44bc intégrée/poussée, releasev0.4.1 publiée avec quatre actifs retéléchargés conformes. Version0.4.1 active Mac/Cartae ; maître55610/pont55688/worker560830. T3 4717/4792 et tunnel60206 préservés sans relance. Tests ciblés46+3PASS, dernier oracleSQL sélectif rejoué seul1PASS ; aucuneFULL. Recette Cartae, rejeu mêmeTask et zéro rappel après échéance prouvés, delivery_ackedfalse conservé. Schémas/configurations conservés ; reprise native de l’ancien choix Revue sansTask vers refus permission_not_inherited. CacheMac170 retiré ; worktreesR1/R2 conservés pour deltaR1 noncommitté et garde lsofCartae incomplète. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/170-delais-reponses-distantes/validation/livraison170.md. SHA documentaire final : /Users/moi/.cache/bridget-delivery170/final170.json. Les checkpoints suivants restent historiques.

Session169 livrée le15octobre2026 :14/14, Implemented, pipeline Livré avec conservations justifiées. Priorité Bridget ON pour les nouvelles délégations depuis T3, tâches existantes gardées sur leur moteur. Source T3 58ddcf3a16205a24f940164cba500089f49ec6c0 intégrée/poussée fork/local/main-20261009 et alignée Mac/Cartae ; T3 0.0.45-local.169 signé installé/actif, ASAR a23d41088ed6aa3671a129778efc42736089f68b433d7f4dd946bc25888fb52c. Une compilation/une activation, main4717/serveur4792 ; Bridget71927/72468 et tunnel60206 conservés, moteur0.4.0 inchangé. Reprise48, contexte prefer et HTTP complet7508698B prouvés. Recette Cartae Luna,2RPC initial+rejeu mais1Task/1lancement, résultat final natif exact et cleanup_done=true ; zéro délégation T3 pour cette recette. P2 CONVERGED, audit source pass,536cas affectés finalement PASS ;75blocs types hérités/RC1, Pi réel et reprise Claude existante positive non qualifiés, aucun GREEN global. Nettoyage borné, deux worktrees Cartae conservés car lsof fuse.portal incomplet ; DB/preuves/rollback protégés. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/169-priorite-bridget/validation/livraison169.md. Commit documentaire final et détails du ménage : /Users/moi/.cache/bridget-delivery169/final169.json. Les checkpoints suivants restent historiques.

Session 168 livrée le 15 octobre 2026 : 56/56, Implemented, pipeline Livré avec conservations justifiées. Source binaire B431b78f72bdd73eac4772af17bf36879d7a629d2 active Mac/Cartae, tag v0.4.0 et release GitHub publiés depuis Cartae ; quatre actifs retéléchargés conformes. Trois voies réelles Laya2/Haiku21/score21 et replays vérifiés. C1 db46c7816777199da7c41bb9bc10063cd361dae7 intégré/poussé/récupéré ; dernier SHA documentaire contrôlé dans les reçus Git privés. Nettoyage : deux anciens worktrees et branche Mac retirés, caches propres traités ; worktree168/branche Cartae conservés propres et fusionnés car garde lsof incomplète,18 anciens candidats conservés avec motifs. DB/preuves/rollback protégés. B en ligne, catalogue27, pont32fils, DB quickOK/FK0 versions7/12, worker3767260actif. Aucun redémarrage T3 par ce lot ; app65544 et tunnel60206 préservés, serveur97328 observé après remplacement extérieur. FULL unique2820PASS/32FAIL/71ignorés,29reprisesPASS,3annulationsLinuxnonvalidables ; auditA98compteurpartiel et2LOW/1INFO conservés, aucun GREENglobal. Reçu : /home/moi/10.Scripts/64.bridget/specs/168-routage-flotte/validation/livraison168.md. Les checkpoints suivants restent historiques.

Session 168 In Progress — 15/10/2026 à 17:13 CEST : 53/56 tâches cochées. T045/T046 closes : source B 431b78f72bdd73eac4772af17bf36879d7a629d2 construite et active sur Mac/Cartae, trois recettes réelles avec rejeux validés, release v0.4.0 publiée et quatre actifs retéléchargés conformes. Tag annoté 43202c9de66ce466b76ba05bd3077699ac88ca3d cible B. T047 commit documentaire C1 en cours ; T048/T049 ouvertes, nettoyage final source par ROOT. Statut In Progress conservé. Santé B en ligne, catalogue 27, pont 32 fils vers 97328, DB readonly quickOK/FK0 versions 7/12, worker B 3767260 actif. Application T3 65544 et tunnel 60206 préservés ; serveur 97328 remplace extérieurement 65608. FULL unique 2820 PASS/32 FAIL/71 ignorés,29 reprises PASS,3 annulations Linux non validables ; aucun GREEN global ni nouvel audit/FULL/P4. Les anciens états168 ci-dessous sont historiques. Reçu : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/livraison168.md. Release : https://github.com/guthubrx/bridget/releases/tag/v0.4.0 .

Session 168 In Progress le 15 octobre 2026 : 45/56 tâches cochées après T036. Code réalisé, Converge P3 CONVERGED, contre-revue T037 et corrections closes, audit T038 pass (A, compteur de pénalités 98 ; axes qualité non notés). FULL unique 2 820 PASS/32 FAIL/71 ignorés ; 29 des 32 échecs initiaux PASS après reprise, trois annulations 159 non validables sur Linux ; cause initiale T3 inconnue malgré replay PASS. T039 en publication, T040 et livraison à suivre. Aucun paquet 0.4.0 construit, aucun commit/push du lot ni activation 168 attestés à ce checkpoint ; production 0.3.0 conservée. Reçu : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/validation/livraison168.md. Plan : /home/moi/10.Scripts/64.bridget/.worktrees/168-routage-flotte/specs/168-routage-flotte/plan.md. SPECIFY_FEATURE=168-routage-flotte ; branche session-168-routage-flotte. Les checkpoints suivants restent historiques.

Session 167 livrée le 15 octobre 2026 : 28/28, Implemented, pipeline Livré. `bridget delegate` et `bridget task` exposent la coordination native depuis la CLI sans exiger T3. Source binaire `24a6507a99520bf2245caf67fa69cbac40bd300f`, tag `v0.3.0` et release GitHub publiés. Client macOS 0.3.0 signé actif sur disque, SHA-256 `4783e8cb4e3389d71c40d6ecb5e0d89bb98f2d7af14e2275c7e48f6b92ed98cd`. Archive Linux SHA-256 `3f4455dc380c855002178df49a4f0b6447bd7323d00f2acc090ca83eeb0296d8`. Tests ciblés 87 PASS/2 ignored ; audit A sans finding. Daemon et pont non relancés pour préserver 58 agents ; PID 68982/68977 inchangés, pont running avec 32 fils et T3 PID 65544/65608 inchangés. Rollback `/Users/moi/.cache/bridget-install167.eT471KDV`. Reçu : `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/167-coordination-cli-autonome/validation/livraison167.md`.

Session 166 livrée le 15 octobre 2026 : 14/14, Implemented, pipeline Livré. Correctif T3 `76780a20bb22b9390eab2389eaf0db87e1349e7e` intégré et poussé sur `fork/local/main-20261009`. T3 `0.0.45-local.166` signé et actif, ASAR SHA-256 `b397aaa905d5c023392afee8459c6d6150a576fc7ef5d18987c0dafb822fb433`. Le fil ancien « Relancer le processus » qui renvoyait `t3_session_unavailable` a répondu `RECETTE166_OK` sans réinstallation Bridget. Tests ciblés 233 PASS et 8 PASS ; typecheck global encore non GREEN pour erreurs héritées hors diff. Incident de livraison : un `launchctl submit` temporaire a relancé T3 71 fois ; service retiré et script neutralisé. T3 stable sur PID 65544, serveur PID 65608, pont `running` avec 32 fils. Rollback conservé dans `/Users/moi/.cache/t3-install166.VqBHdpl2/T3-before.app`. Reçu : `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/166-rattachement-bridget-reprise/validation/livraison166.md`.

Session 165 livrée le 14 octobre 2026 : 18/18, Implemented, pipeline Livré. Source 3b4f101e0732 intégrée et poussée sur github/main. Binaire signé SHA-256 45bc4836c874273012af9c3fffa52d5f676a9147809737548d59120c256dc1a4, pont PID 42156 actif depuis 20:59:39 CEST. T3 PID 30234 inchangé. Liens 1130→32, fichiers ouverts 4539→167, limite souple revenue à 4096 et override 16384 retiré. Aucun auth_failed, HTTP401, EMFILE ou Too many open files après activation. Tests ciblés 13/102/22 PASS avec 1 ignored ; suite complète 1944 PASS/91 FAIL hérités/16 ignored, aucun GREEN global déduit. Contre-revue finale approuvée. Worktree et branche165 retirés sans force. Rollback conservé dans /Users/moi/.cache/bridget-install165.UOCBU3. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/165-cycle-vie-liens-t3/validation/livraison165.md.

Complément visuel164 livré le14octobre2026 : T3 source3ea39f11095debbba6ca6a73f99f44fbe308cd60 intégrée/poussée fork/local/main-20261009. Symbole Bridget avant fournisseur, bleu local connu/monochrome distant/neutre inconnu. App164 signée installée, ASARdc5623d70f45c85909703274ba59d216295b9e84b1e50ab8685b75aabe73f942. T3 processus courants conservés sans relance ; ancien code chargé. Reçu /Users/moi/.cache/t3-lineage-icons-164-backup/T3-install-receipt.json. Développement et contrôles ciblés Cartae ; 7PASS Bridget et typecheck/lint conformes, échec de libellé de compte fournisseur attesté sur baseline. Les checkpoints ci-dessous sont historiques.

Sessions 163/164 livrées et nettoyées le 14 octobre 2026 avec conservations justifiées : 163 58/58 et 164 22/22, Implemented ; Pipeline : Livré. G8/G9 validés ; T045 163 clôturée sur documents de livraison c57 poussés.
Source active 3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5 intégrée et poussée github/main ; Mac SHA-256 cfb1ff4f91e1b59fbbfd403607d4f6dd08f24229724822f3d8eee9b6eb1a381c, Linux SHA-256 dfc9c3c78f7c8e16a394bc57f77d06488acde6495be45093674025be4209af97.
Maître 31298/pont 31420 nés 05:32:24/26 CEST ; worker Cartae 1454489, naissance 146390445. Configuration ON réelle SHA 36c7d13c571f97e49f4caaa59c272597c87735fbfd44667068da77f883399cf4.
G8 : admission OFF native codex-pro/gpt-6.1-sol, résultat exact RECETTE163_G8_OK transmis et cleanup_done=true. Workspace Shared sans garde acquise. Rejeu ON même clé : Task/résultat/reçu entier inchangés, aucune nouvelle décision durable, pas de capture réseau. T040 R4 qualifiée/RC1 historique conservés, aucun POST supplémentaire.
T3 /Applications/T3 Code (Local).app installée en 0.0.45-local.164, source 511190beb307e0952066fe1a4b511cb9dd7adb18 ; processus 704/969 restent en version162 sans redémarrage. Le runtime162 est exclu des preuves de nouveau fonctionnement T3 de164. Trois worktrees et branches Mac, deux stages et six TMP retirés sans force. Worktree T3 163/branche conservés : scan 180 s complet 282139 fichiers, inventaire changé après archive. Cartae WT/branche/connecteur/caches/TMP conservés : sudo lsof RC0 mais fuse.portal incomplet. Cache SQLite persistant/ancienne app T3 chargée/rollback/preuves/DB protégés, aucun contournement.
Audit 163 historique A99,047619 partiel 3M/2L conservé ; delta T058 A100 partiel 6,749687 % 0 finding/validateur RC0 et delta T3 R6 sans finding. Anciennes FULL non GREEN conservées ; P3 NONCONVERGED historique et poursuite corrective utilisateur phase 5, pas de P4 formel.
Reçu canonique : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/163-decision-admission/validation/livraison163.md.
Documents G8/G9 intégrés et poussés dans c57cfd35efa1338cda5256a02c57d71258d8b8d4, github/main égal HEAD propre au reçu /Users/moi/.cache/bridget-delivery163-20261013/git-final-doc-checkpoint-c57cfd35.json. Build/source active 3efa distincte de la source documentaire c57. Marqueurs finaux 58/58 et 22/22 intégrés et poussés dans af8998a7d7494913d4420e2dca7354a73bd8a4f5, github/main égal HEAD propre vérifié par ROOT ; Cartae main/worktree alignés sans delta de code ni changement du build3efa. Preuve d’alignement Cartae : /Users/moi/.cache/bridget-delivery163-20261013/git-final-cartae-af8998a7-command.json ; SHA-256 a8566f602017d52ea0ca9ab3706871038275aa04477b67077fb572b74d638620. Reçu 164 : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/164-conversation-t3-distante/validation/livraison164.md. Les checkpoints suivants sont historiques, pas une preuve du runtime courant.


Session 162 livrée et nettoyée le 13 octobre 2026 : 46/46, G8/G9 validés avec conservations justifiées.
Source active 1dc0c6022e8d0712fbc4c65a8c20be7b2ea8a437 intégrée et poussée ; documents 4cf50b68688b2d28872f67ae32e129f989d22449 intégrés et poussés.
Daemon Mac 71469 et pont 71567 actifs depuis 17:53:53/55 CEST ; worker Cartae 471321, naissance 142199350. T3 704/969 et tunnel 1384 inchangés depuis le préflight après redémarrage externe.
Trois routes Bridget/Roadie/Opus2D validées ; reprise et résultat durable attestés. Annulation corrigée : huit PID absents avant fin naturelle du sleep, preuve .drained et slot fermé. DB quick_check ok, FK0, schémas 7/12, Fleet 5.
Correctif : deux tests de bibliothèque et cinq E2E Linux PASS, contre-revue R2 APPROVE, annexe audit 0 critique/0 élevé/1 faible. Aucun GREEN global ni nouvel audit global déduit.
Worktrees et branches du lot retirés sans force, 45 preuves ignorées copiées byte-identiques. Stage Mac réduit de 0,31 Gio logiques ; debug 44,96 Gio conservé pour 163. Stages distants conservés car garde lsof incomplète, sans contournement ; prérequis, DB, rollback et preuves conservés.
Client /home/moi/.local/bin/bridget pointe vers /home/moi/.local/lib/bridget-communication/162-1dc0c6022e8d/bin/bridget.
Reçu durable : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/162-execution-distante/validation/livraison162.md.
Ce checkpoint final doit encore être committé et poussé par ROOT. Les checkpoints suivants sont historiques.


Session161 client Cartae livré opérationnellement le13octobre2026 à08:59:38CEST :14/14, ConvergeP1/P2, auditA100 ciblé7fichiers/1169lignes. Runner10f1e11cf04deb38b3220ce3ec1a5afae403fcef intégré/poussé github/main ; sourcecompilée160e6812e3e7b2d051d630192857cc3f440180d51fa. ClientLinuxSHAf008936c92086b4c440c226ac430d019dfa8ddf337dfbd11c3629bdba016e938 actif via /home/moi/.local/bin/bridget. Maître/pont/T3/tunnel inchangés ; aucundaemonCartae. DeuxWT etbranchesession retirés, fixturepackage retirée ; target348420473octets logiques conservé carlsofincomplet, garde noncontournée. Rollback096 etpreuves conservés.
Plan durable : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/161-client-cartae/plan.md.
Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/161-client-cartae/validation/livraison161.md.
Métadonnées finales àcommitter/pousser parROOT ; git-final161.json attendu, aucune preuve future présumée. Lescheckpointssuivants sont historiques.
Sessions152–160 livrées et nettoyées le 2026-10-13 :317/317 tâches du périmètre prévu.
Dernière source160 e6812e3e7b2d051d630192857cc3f440180d51fa intégrée/poussée github/main ; release signée installée et active, daemon20937/pont21097 depuis00:25:51/54CEST. Base idempotentev7/exécutionsv12 quickOK/FK0 ; T3 PID60116/60228 inchangés.
160 :37/37, Converge3, audit A99,65 partiel6,22%,0C/0H/1M/1L nonbloquants, validateurRC0. FULL unique2683PASS/6FAIL/76ignored historique conservée ; six oracles testseuls corrigés, replays exacts1+5PASS. Aucun GREEN global ni quota réel attesté.
WT/branche160/stage retirés sansforce ;16namespaces privés TMP retirés,2383932535octets logiques, archive196membres SHA vérifiés. Rollback159B privé, preuves et caches utiles conservés. US8/HTTP158 reportés restent horspérimètre.
Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/160-reprise-quota/validation/livraison160.md.
Les checkpoints ci-dessous sont historiques, pas une preuve du runtime actuel.

Lot159A livré le12octobre2026 :14/14 tâches ; lot159B14ouvert, spec159InProgress.
Source53863bd09a4ff364dc2b412519b343c77e163685 intégrée/poussée github/main ; release signé actif daemon77955/pont78064 depuis19:42:37/40CEST. Basev7 quickOK/FK0 ; held_queuefalse par défaut après SC004. T3 non relancé.
ConvergeP3+annexeR2 ; neuf FAIL historiques FULL rejouésPASS exacts, pas GREEN global. Branche/worktree/stage159A retirés, archive244membres vérifiée, sauvegarde158Bv6 conservée. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/159-edition-file/validation/livraison159A.md.

Session158B livrée et nettoyée le 2026-10-12 : 23/23 tâches B, 45/45 A+B. Sources e52a96240b6168bf4466712e1bda8852b4ecb43f intégrées et poussées github/main. Binaire signé build e52a96240b61 installé et actif : daemon20530/pont20607 depuis15:21:38/41CEST. T3 PID60116/60228 inchangés. FULL2456PASS/12FAIL/74ignored historique conservée ; derniers replays affectés4PASS ; aucun GREEN global. Audit A99,95 partiel7,9%,0C/0H/0M/1LOW ; validateurRC0. P3 CONVERGED et complément maintenu. WT/branche158B retirés, preuves466membres SHA vérifiées, rollback158A conservé. US8/HTTP reportés. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/validation/livraison158B.md.

158A livrée et activée le 2026-10-12 : 22/22, sources80b434079361 intégrées et poussées github/main. Binaire signé installé, daemon53200/pont53329 actifs ; T3 inchangé. WT/branche158A retirés. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/158-reveils-controle/validation/livraison158A.md. 158B puis159/160 restent à traiter ; les checkpoints ci-dessous sont historiques.

Session157 livrée et nettoyée le 2026-10-12 : 26/26 tâches, CONVERGED passage2.
Sources8f21dfd7b553e822fcabe0a0e7eabc6741a8e45b intégrées et poussées github/main.
Binaire signé SHA256 f45b261a57ebaa6c4cd53e9195ceec66709de54b4fa394fa947e1a758ee1697d installé et actif.
Daemon5528 et pont5638 activés à10:33:52/55 CEST ; T3 60116/60228 inchangé.
FULL unique2396PASS/9FAIL/74ignored, oracle154 corrigé1PASS et quota13PASS séparés ; huit échecs hérités conservés.
Audit A100 sur couverture source4,4095%, aucun succès global déduit.
Worktree et branche157 retirés ; 317243392 octets logiques du stage retirés, preuves et sauvegarde156 conservées.
Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/157-catalogue-quotas/validation/livraison157.md.

Session156 livrée et active le2026-10-12 :32/32, ConvergeP2, audit0C/0H/1M/4L, validateur conforme.
Sources cf498755dc87 fusionnées/poussées github/main ; binaire signé SHA2152455f09cd, daemon55530/pont55643, T3 inchangé.
Worktree/branche156 retirés, archives/rollback conservés. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/156-guidage-file/validation/livraison156.md.
Les checkpoints précédents sont historiques.

Session 155 livrée et activée le 2026-10-12 : 42/42 ; trois Converge puis contrôle cwd ciblé, pas passage 4.
Source 59da3968c7d14dbd7acd1370875dd9d135ba7dcb intégrée/poussée github/main ; build 133,80 s RC0.
Binaire signé installé/actif SHA 4d3ff43d6a82de02b4ed115f1af422703ca584d1ce7f4843dcda41179ac748a7.
Daemon 69512/pont 69652 nés 05:47:08/11 CEST ; T3 PID 60116/60228 inchangés.
71 PASS post-cwd ; T040 quatre cas réels PASS validés par l’utilisateur avant cwd, provenances conservées.
FULL unique antérieur 2259 PASS/15 FAIL/72 ignored, 180 baseline PASS ; aucun GREEN global ni seconde suite.
Audit final 0C/0H/0M/2LOW, couverture 27,46 %, validateur 0/0 ; limites conservées.
WT/branche 155 retirés ; 81 preuves ignorées archivées/vérifiées, DB/rollback/cache utile 156–160 préservés.
Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/155-demandes-enfant/validation/livraison155.md.
La clôture documentaire conserve la distinction entre source du binaire et métadonnées de livraison.
Les checkpoints ci-dessous restent historiques, pas une preuve du runtime courant.
Correctif T000155 livré seul le 2026-10-12 : sources c8f6f8a0e09a fusionnées et poussées github/main.
Binaire signé installé et actif ; daemon81535/pont81746 à01:48CEST. T3 inchangé, aucune base supprimée.
Trois tests ciblés PASS, ConvergeP2, audit A99.95 sans blocage. Les 40 autres tâches155 restent ouvertes.
Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/155-demandes-enfant/validation/livraison-t000155.md.
Session 154 livrée et activée le 2026-10-12 : 28/28, CONVERGED passage 2 ; Claude R2 APPROVE.
Code 806d0984fe45eea9fcc885770285bc00a85d005e intégré/poussé github/main ; build 58,36 s RC0.
Binaire signé installé/actif SHA 39834b7d36bd184842ea8cf99994a08e92af583178a97b0dba537643e52ffaa8.
Daemon 67985/pont 68081 nés 01:25:36/39 CEST ; T3 PID 60116/60228 et naissances inchangés.
Audit A 99,9, 0C/0H/0M/1L ; couverture 84/122, 68,85 % ; validateur RC0.
Suite workspace UNIQUE antérieure 2194 PASS/9 FAIL/72 ignorés, 499,95 s ; huit échecs historiques et fixture UTC corrigée 1 PASS, puis ciblés 5/3/2 PASS. Aucun GREEN global ni seconde suite.
WT/branche 154 retirés 01:26 ; 0,29 Gio logiques d’intermédiaires retirés, debug ≈18,30 Gio conservé pour 155–160, budget 20 Gio dans les commandes suivantes.
Fixtures : 18 TMPDIR attestés, 7 racines vides et 74 937 515 octets hors DB retirés ; DB/sidecars et sauvegarde 153 conservés, aucun gain physique revendiqué.
Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/154-isolation-worktree/validation/livraison154.md
Les checkpoints ci-dessous restent historiques, pas une preuve du runtime courant.
Session153 livrée et activée le2026-10-11 à23:45 CEST ;29/29, CONVERGED passage1.
Code 7c848850ba6067cf1d0d94b8edf0dd11896a1b24 intégré/poussé github/main ;57 fichiers byte-identiques.
Build7c848850ba60 RC0, signé installé/actif :daemon67026/pont67170 nés23:45:25/27.
T3 PID60116/60228 et naissances01:13:35/36 inchangés ; aucune relance T3.
41 tests natifs finaux et179 compatlib PASS ; workspace R1 antérieur2145/10/72,
huit échecs baseline conservés et corrections affectées prouvées ; aucun GREEN global.
Audit A98,35,0C/0H/4M/11L, validateur RC0 ; mémoire réelle et mesures SC001/002/006 limitées.
WT/branche153 retirés ;0,29 Gio logiques nettoyés, cache debug13,50 Gio utile154–160 conservé.
Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/153-rondes-contexte/validation/livraison153.md.
Les checkpoints ci-dessous restent historiques, pas une preuve du runtime courant.
Sessions 150/151 livrées sur Git et sur disque le 2026-10-11 : 150 39/39, 151 27/27. Feuille de route 152–160 préparée seulement. Sources b39367a633cd puis e1af5c58a0e8 intégrées et poussées github/main. Binaire e1af5c58a0e8 signé installé atomiquement ; runtime inchangé, aucune relance. Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/151-attente-fin-de-tache/validation/livraison150-151.md.
Complément149 identité durable en cours le 2026-10-11, dans la session déjà autorisée.
Sources Bridget9f2ccbf23195 et T3d833387203 intégrées et poussées vers leurs forks.
MCP officiel existant réattesté par fil ; opt-out/configurations étrangères protégés. Routes de fils opaques encodées.
Tests ciblés T3124PASS/1SKIP et Rust25PASS/1ignored ; revue finale approuvée, format final validé.
Binaire Bridget9f2ccbf23195 signé installé/actif : daemon30562, pont30974 depuis02:17:41/48 le11octobre. Serveur T360228 encore ancien.
Paquet T3 corrigé signé prêt dans /Users/moi/.cache/t3-identite149-final.OBXQrQ/stage/T3 Code (Local).app.
Défaut runtime supplémentaire : longs IDs comme noms de fichiers d’état. Correctif compact/reprise legacy9f2ccbf2 intégré/poussé, 85testsPASS et revue approuvée ; 121états compacts observés, aucun ENAMETOOLONG dans les nouveaux journaux.
L’utilisateur a demandé d’attendre la fin des autres fils avant de relancer T3 ; aucune relance programmée.
Reprise après activation : /Users/moi/.cache/bridget-identite149-rollback.af6k7sa_/RESUME.md.
Reçu courant : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/identite-durable149.md.
Les checkpoints ci-dessous restent historiques, pas une preuve du runtime actuel.
Correctif149 de limite de fichiers activé le 2026-10-11 après demande « corrige ».
Code Bridget0d5107afcf186bacc57aefd1925cd98f076806b8, intégré et poussé github/main.
Binaire signé build0d5107afcf18 ; daemon10624 et pont14501 relancés individuellement.
Limite souple4096 sur les deux services, plafond dur inchangé ; 4testsPASS, 1fixture ignorée.
Pont322fils contre63 avant, daemon983descripteurs et pont1295–1296 ; zéroEMFILE après démarrage corrigé.
T3 reste en0.0.45-local.149 ; PID60116/60228 et dates de naissance inchangés par ce correctif.
MCP hérité du principal : identity_not_found, variables d’attestation absentes dans son processus73669.
Warnings thread_detail/threadId antérieurs conservés ; aucune validation globale déduite.
Reçu : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/limite-fichiers149-apres.json.
Les checkpoints de livraison ci-dessous décrivent les opérations historiques du 10 octobre.
Session149 livrée sur disque le 2026-10-10 : 45/45 tâches validées et cochées.
Sous-agents natifs autonomes hors T3 ; Lineage T3 avec statut, journal et résultat.
Droits hérités du parent sans grant humain neuf, sans gain de droits.
Sources Bridget : 072161262a7d4d6ea2e1305d546f115c634506d4, intégrées et poussées github/main.
Sources T3 : f6f3d1998c686b20e316f304aff7f5df6a14592c, intégrées et poussées fork/local/main-20261009.
Binaire /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget signé, build072161262a7d.
Application /Applications/T3 Code (Local).app signée en0.0.45-local.149.
PID58394/58396/58468 et leurs dates de naissance inchangés ; ancien code chargé.
Aucun redémarrage ni activation différée ; prochaines relances utilisateur utilisent149.
Rust1840PASS ; paquet2576PASS ; contrôle final Haiku21 vérificationsPASS.
Ancien banc p95 non conclu et erreurs T3 préexistantes documentés ; aucun succès global inventé.
Worktrees/branches149 propres retirés ; environ13,6Gio de fichiers temporaires retirés.
Sauvegarde148 /Users/moi/.cache/bridget-install149.8atzwz conservée, aucune DB touchée.
Reçu canonique : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/149-sous-agents-lineage/validation/final.md.
Les chemins de WT/clones/stages dans les preuves antérieures sont historiques.
Les checkpoints148 ci-dessous restent historiques.
Plan session148 approuvée et livrée : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/148-identite-delegation/plan.md.
Codex high, identité par session T3 et délégation Bridget en un appel. Aucun redémarrage réel.
Session148 livrée sur disque le 2026-10-10 : code Bridget347d788510b4, T333f6d04e1164.
Recette GLM 5.3 réelle validée sans T3 ; parent Codex synthétique, résultat corrélé unique.
Application /Applications/T3 Code (Local).app en0.0.45-local.148 ; binaire Bridget signé remplacé.
Les PID58468/57109/58394/58396 restent chargés sans relance ; aucune activation différée.
Sources poussées vers github/main Bridget et fork/local/main-20261009 T3, jamais upstream.
Reçu canonique : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/148-identite-delegation/validation/final.md.
Worktrees/branches propres148 et reliquat147 retirés ; preuves et sauvegardes conservées.
Les checkpoints147 ci-dessous restent historiques.
Session147 approuvée le 2026-10-08 : abonnement humain du panneau Bridget et
sélection retenue par environnement/projet/conversation T3.
Socles142/145/146 committés et fusionnés dans la livraison147 ; autres projets intacts.
Contrat147 : human_thread_watch_v1, bridget.watch, flux version/generation/seq/status.
ready seq0 premier non coalescible ; changed/resync sans corps ni UUID de fil.
Garde humaine commune, mutations réellement committées seulement, aucun signal
sur replay/ACK/read/refus. Aucun réveil, notification agent, modèle ou mission.
Reprise par lectures146, jamais par journal durable ou polling de contenu.
Fil sélectionné : UUID seul dans store natif, clé environment/project/conversation.
Un refus thread_unavailable confirmé efface le choix. Panne/binding garde le UUID
mais masque le contenu non vérifié. Refus métier fermé : pas retry infini.
Snapshot nouveau S commun au segment déjà chargé, staging puis publication atomique.
Préserver dépliage, pages, focus, texte sélectionné et copie des lignes inchangées.
Flux scoped annulé à fermeture/page masquée ; idleTtlMs0, ressources bornées.
Session147 Implemented, Validé en environnement isolé, après ajouts US6–US9 approuvés le 2026-10-09.
Le premier lot US1–US5 reste validé en isolé :30 tâches prouvées et cochées.
Extension US6 : UUID des actions de fils partagés CLI/MCP normalisés avant idempotence/SQL et vrai montage MCP
Claude/GLM avec identité attestée, opt-out et permissions conservés.
Plan UUID validé ; T031–T034 prouvées et cochées sur GO après RED/GREEN et régressions.
US7 : SDK294PASS/1SKIP final, MCP OS3PASS/refresh1PASS distincts ; T049/T050 validées après revues finales APPROVE.
US8 : plan/gate5/5, tests et interop natifs validés ; T042–T048 cochées, capacité ThreadMembersV1/thread_members_v1.
Créateur initial/fil ouvert/max16, --member répétable, historique complet/cursor0 et zéro alerte à l'adhésion.
Union projet recheck transactionnelle ; ancien wakepending non dispatché ; pas composeur UI.
US9 native validée : choix en haut/ligne épinglée unique autorisée hors page/filtre, sans header inférieur ; détails frères avec titre et membres complets. T051–T054 cochées sur122PASS et recette native finale. Pas nouvelle API/store/helper/rebrand.
Cible regional-wrkr-1/claude_glm/.claude-glm vérifiée : MCP effectif sans Bridget.
Avant correctif, T3 injectait t3-code seulement. Cause locale prouvée, montage MCP corrigé et validé dans les fixtures privées. Aucun succès de modèle actif déduit.
Aucun modèle, restart ou configuration de production dans cette phase.
Deux passes finales du principal :31FR/20SC/9US mappées, zéro finding actif,54/54 tâches après GO final. Tâches byte-identiques durant Converge ; cochage seulement après son approbation.
Régressions Rust103PASS/2ignored, transport10PASS ; RPC US7294PASS/1SKIP et Web122PASS. Audits frais Rust A/T3 A99.86, validateurs RC0/0erreur/0warning, périmètres diff limités.
Recette native avec données synthétiques distinguée de l’interop native réelle.
US5 affiche les seuls destinataires sollicités dans la ligne auteur/date existante.
Développement validé,54/54. Livraison finale reconstruite : Bridget1163038fc25f, T3eace00356d91, sources intégrées et poussées.
Fichiers installés : /Applications/T3 Code (Local).app en0.0.45-local.147 et /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget.
Signatures valides ; paquet2413 contrôles réussis. Aucun redémarrage : T349975/50035 et Bridget93353/93355 conservent leur code chargé.
Reçu actuel : /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/147-panneau-bridget-vivant/validation/lecture-mcp-correctif.md, dernier checkpoint.
L'ancien reçu d'activation aborted_before_replacement reste historique. Aucun job d'activation différé n'est programmé.
Push Bridget vers github/guthubrx/bridget et T3 vers fork/guthubrx/t3code, sans PR T3 ni push upstream.
Sauvegardes ciblées146 et DB Bridget arrêtée/quickcheckok : /Users/moi/.cache/bridget-install147.VTg5jQ. Application146 rollback-ready vérifiée, pas DB T3.
Cleanup final : reliquats142/145/146 committés et intégrés ; leurs worktrees propres et branches de session retirés. Les intégrations147 des skills sont également retirées.
Les snapshots de build, caches Cargo et anciens paquets/stages remplacés sont nettoyés. Paquet final et reçus : /Users/moi/.cache/t3-final147.zrm6Zc.
Conserver la sauvegarde146, les preuves et tous les travaux étrangers. Aucun moteur Agent Loop, fichier de conversation ou base T3 supprimé.
Les chemins WT dans les preuves sont historiques. Audits archivés dans /Users/moi/Nextcloud/10.Scripts/64.bridget/audits/2026-10-09, copie diff-qrRC0 ; archive complète /Users/moi/.cache/bridget-install147.VTg5jQ/audits147.tgz.
Le principal exécute Git et livraison ; l'owner documentaire ne touche aucun code/service/production.
Sélection SpecKit : SPECIFY_FEATURE=147-panneau-bridget-vivant ; livraison finale sur main.
Le complément CLI/MCP147 est intégré. Reconstruire depuis les arbres fusionnés propres.
Installer uniquement les fichiers ; ne relancer ni T3 ni les services Bridget.
Les anciens checkpoints ci-dessus restent historiques, pas une preuve du runtime actuel.
<!-- SPECKIT END -->

<!-- SPECKIT-USER START -->
# Couche utilisateur SpecKit

- Source de verite utilisateur: `~/.speckit/`.
- Avant une feature/refactor structurel: executer ou appliquer `/speckit.sync`.
- Charger `.specify/memory/constitution.md` pour les gates projet.
- Charger `.specify/memory/standards.md` pour retrouver les refs et baselines research.
- Ne pas modifier directement `.agents/skills/speckit-*`, `.specify/templates/*`, `.specify/scripts/*` ou `.specify/workflows/*`.
- Les commandes utilisateur canoniques vivent dans `~/.speckit/commands/` et sont publiees vers Codex/Claude/Gemini par `~/.speckit/scripts/sync-agent-adapters.py`.
<!-- SPECKIT-USER END -->
