# Livraison SPEC-163 — 58/58, Livré avec conservations justifiées

**Statut courant au 14 octobre 2026 :** Implemented, 58/58 (100 %) ; **Pipeline : Livré** avec conservations justifiées. G8/G9 et documents de livraison sont acceptés ROOT ; T045 clôturée sur le commit documentaire c57cfd35efa1338cda5256a02c57d71258d8b8d4 poussé et vérifié. Source active 3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5 inchangée ; les marqueurs de clôture 58/58 sont intégrés et poussés dans af8998a7d7494913d4420e2dca7354a73bd8a4f5, github/main égal HEAD propre vérifié par ROOT. La source active reste 3efa, distincte de ce commit documentaire.

## Checkpoint installation et provenance — 53/57, 14 octobre 2026

ROOT accepte T038/T040/T041/T042 sur leurs preuves réelles. Bridget source 5105378bf4f03376e9bc93e52065cb022f81dee5 installé : Mac SHA-256 6c8f89203c85c76e7add2a4d06bb86eedd6cdb3f74470eac11920cc0f6068063, Linux SHA-256 6e5d9717d6054511326f84c684d65c3f3bc1a5086cb514b2d6b64a03ad9b97e0. Maître 44553 né 04:55:03 CEST et pont 44667 né 04:55:05 utilisent /Users/moi/Nextcloud/10.Scripts/64.bridget/target/release/bridget ; worker 1411115, naissance 146166371 utilise /home/moi/.local/lib/bridget-communication/163-5105378bf4f0/bin/bridget.

T3 /Applications/T3 Code (Local).app remplacée atomiquement en 0.0.45-local.164/source 511190beb307e0952066fe1a4b511cb9dd7adb18, signature stricte/deep RC0 et entitlements préservés. PID 704/969 et naissances 17:31:27/33 du 13 octobre restent inchangés : aucun redémarrage, ancien code encore chargé. Aucun réveil différé programmé.

Sauvegarde retry : phase_complete/activate, projection173→174 de même génération et 33 autres tables exactes, schéma/count inchangés ; halt humain complet préservé. Migration additive reçusauto, quick_check ok/FK0, schémas 7/12/Fleet5. Sauvegarde jointe vérifiée ; après activation, toute rétrogradation exige une nouvelle sauvegarde jointe sans perte. La sauvegarde préactivation ne couvre pas les nouveaux reçus. L’incident171→172 et les anciens PID 86807/86830/1403968 restent uniquement historiques.

Le reçu d’installation conserve handshake_and_cancel_recipe_not_proven=true : poignée de main et annulation opérationnelles restent à vérifier pour G8.

T039 : /Users/moi/.cache/bridget-core/agents.json est appliquée au SHA 5dead7198c0220477d88229d1b0ab0f4b3fe5858a0391bde1cfb687c3265c93b ; fixed/fallback local-codex-pro, agents/machines inchangés. auto_engine=false, auto_placement=false, service_enabled=false. L’application off est réelle, mais l’activation des axes/service et les preuves G8 restent ouvertes : T039 non cochée, comme T043/T044/T045.

Ordre réel : sources committées/intégrées/poussées avant laboratoire T040, pour obtenir un checkpoint déterministe compilé identique sur Mac/Linux ; validation T040 exécutée avant activation. Ce commit de source préalable n’est pas une clôture de livraison ni un hash futur inventé. ROOT accepte T040 sur observations R4 et supplément corrigé 54056f…8d49 ; RC1/course F historique conservé, aucun nouveau RC0 ou POST ajouté. P3 NONCONVERGED historique/exception utilisateur phase5 conservés, aucun P4 formel.

Reçus :

- /Users/moi/.cache/bridget-delivery163-20261013/joint-backup-20261014-163164-retry-projection/progress.json ; SHA-256 b6da23d355fbb396b50fab7c946dd5693dd6ced27aafe5ae3ac2aaffaf7cef16
- /Users/moi/.cache/bridget-delivery163-20261013/t3-backup163164/T3-install-receipt.json ; SHA-256 b73ddc5019a644fc9964372816089f95cfffdd276b319ad0bbef5196429fa2f0
- /Users/moi/.cache/bridget-delivery163-20261013/g8-registry-off-applied.json ; SHA-256 ed358ea42dc927adf19d4c4e7cca2ecdd0de76e2aa2a213116f6b15049b26aae

## Annexe préparatoire historique conservée

T041 : Bridget 5105378bf4f03376e9bc93e52065cb022f81dee5 intégrée/poussée vers github/main, push vérifié. Cartae main a reçu ce commit par avance rapide et était propre au reçu du 14 octobre 02:09:54Z. T3 511190beb307e0952066fe1a4b511cb9dd7adb18 poussée vers fork, refs/heads/local/main-20261009. La précédente intégration T3 8dadcb… non poussée reste historique. Les builds source 5105378 et le paquet T3 version 164/source 511190 prouvent la préparation des fichiers, pas leur installation ou activation actuelle.

T039 : quatre profils, préférences et routes retenus ; profil fixe/repli local-codex-pro. Candidats privés off puis on préparés. Ils ne constituent pas une configuration effective ni une disponibilité de quota attestée en production.

Incident de sauvegarde historique : premier gel-backup refusé avant substitution, install_started=false, garde new_work_or_unexplained_stop_delta. ROOT a identifié la projection seq171→172 sur déconnexion, génération stable, sans nouveau travail métier. Pilote privé corrigé pour qualifier cette transition bornée ; archive avant patch et contrôles exacts après gel conservés. Cette qualification ne dispense pas des contrôles des tables métier.

La reprise avant installation a redémarré seulement les anciens services initialement actifs, source 1dc0c6022e8d0712fbc4c65a8c20be7b2ea8a437 : daemon 86807 né 04:48:21 CEST, pont 86830 né 04:48:23 CEST, worker 1403968, naissance 146126086. Aucun fichier ou DB restauré. Ces identités restent historiques, sans observation actuelle ni activation 163 revendiquée. T3 et tunnel étaient protégés dans ce reçu.

Le supplément T040 corrigé retire la conclusion production_unchanged finale : les captures anciennes ne prouvent pas l’absence de changements pendant les opérations ROOT suivantes. Ce brouillon ne recontrôle aucun état runtime.

Au mandat préparatoire antérieur, T042–T045 n’étaient pas faits. T042 est maintenant acceptée par le checkpoint installation ci-dessus. Fichiers installés, activation/configuration effective, recettes opérationnelles, reçus finaux et nettoyage restent à attester par ROOT. Aucun G8/G9 final, recette RC0, tâche cochée ou clôture globale déduit. P3 NONCONVERGED historique et exception de poursuite corrective conservés, aucun P4 formel inventé. Rapprocher ces annexes des observations finales avant intégration.

Reçus lus, avec empreintes :

- /Users/moi/.cache/bridget-delivery163-20261013/git-bridget-source163164-pushed.json ; SHA-256 863cef70702815e6205bb1b2dfc0c617bf5759231a7e501d12810fb026e249e2
- /Users/moi/.cache/bridget-delivery163-20261013/git-cartae-main-source163164.json ; SHA-256 a053317356cf11f356b5088caac487d988b2fd326be080d5d9270456738030f9
- /Users/moi/.cache/bridget-delivery163-20261013/joint-backup-20261014-163164-020006Z/progress.json ; SHA-256 5905efff9912d6b78e76b5d505a668cec4aee2c7a01ffad47e771d5999096dd6
- /Users/moi/.cache/bridget-delivery163-20261013/preparation-upgrade163-retry-projection.json ; SHA-256 ad2286bca15d135516ce3e661c55ecce95804f4835a250a32b1e4b09904dce66
- /Users/moi/.cache/bridget-delivery163-20261013/lab163-preparation/t040-r4-cleanup-conclusions-supplement.json ; SHA-256 54056f844075544f45ae83a8072f7a6c1421ea16bf391d32458a05726d4c8d49
- /Users/moi/.cache/bridget-delivery164-20261013/git-t3-source164-pushed.json ; SHA-256 e7685a463e56580b511caa1afb76eb1fa6d7c53612025122506088be677c0259

## Correctif G8 — façade MCP T058, 14 octobre 2026

Le MCP propre à la nouvelle commande annonce bridget_auto_delegate dans tools/list, mais tools/call renvoie JSON-RPC -32602, message outil inconnu, request_id g8-spec163-a73f9432-53d5-46b5-94b9-e8f0ee3cfe7e. Les attestations officielles immédiatement avant appel portent version2 et permissions_present=true ; provider codex-pro déclaré disponible et capacité maître contrôlée. Aucun reçu auto/Task sous cette clé selon la vérification durable ROOT. La cause confirmée ROOT est le dispatcher de /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/mcp.rs:753–760 : routage auto omis malgré l’annonce. Scope/hook/wrapper doivent être raccordés à la voie native existante. Ce refus ne démontre pas un problème de quota ou d’authentification et ne nécessite pas de redémarrage T3.

T058 reste dans la session163 autorisée : correctif minimal sur Cartae, oracles MCP existants affectés, contre-revue/delta d’audit ciblés, build et livraison du correctif avant reprise G8. Aucun nouveau framework, banc de confort ou FULL. Le prochain commit source sera différent de5105378bf4f03376e9bc93e52065cb022f81dee5 ; aucun hash futur fixé. T038/T042 demeurent cochées pour les builds/installations réellement prouvés du510, pas pour le futur correctif. T3 source511190beb307e0952066fe1a4b511cb9dd7adb18/version164 est déjà installée et inchangée. Aucun résultat antérieur effacé, aucun livré final ou G8/G9 déduit.

Compteur courant53/58 : T039/T043/T044/T045/T058 seules ouvertes. La preuve brute est conservée : /Users/moi/.cache/bridget-delivery163-20261013/g8-mcp-off-error-observation.log ; SHA-256 03e823bb6a993a7f1b8b546edc53f6aeded6e163bcd9ed507b22a68dc28f2f47. P3 NONCONVERGED historique et poursuite corrective sous mandat utilisateur restent explicites ; aucun P4 formel inventé.


## Checkpoint source T058 — tests et contre-revue, 14 octobre 2026

Les douze oracles ciblés réussissent : onze tests de bibliothèque et un test d’intégration exact. Le reçu /Users/moi/.cache/bridget-delivery163-20261013/auto-mcp163-g8-validation-final.json porte SHA-256 bcbe4d808ea71c47c11690ef9570c282ed8ba62e324c186de1176e41cf60aa78. Les six empreintes source sont identiques avant et après la campagne. Le filtre initial qui avait exécuté zéro test reste archivé et ne compte pas parmi les PASS. Son nom a été corrigé ; les six premiers succès n’ont pas été rejoués.

La contre-revue indépendante R5 reste APPROVE_WITH_CHANGES. Elle valide les cinq empreintes du premier périmètre par lecture, sans exécuter de test. F1 a été traité ensuite : une seule ligne ajoute bridget_auto_delegate à l’attendu du test d’intégration existant. spec094_claude_recoit_la_liste_fermee_des_outils_bridget_autorises donne 1 PASS/0 FAIL, RC0, 9,584249 s de commande. Les cinq sources déjà revues sont inchangées. Reçu R5 : /Users/moi/.cache/bridget-delivery163-20261013/t058-review-r5.json ; SHA-256 51c128704ecad1610eae22c50038e9c35f7a6deff8a7e8943279d9b22cc36c4d. Aucun nouveau verdict APPROVE final n’est inventé.

L’audit T058 readonly conclut A/100 sur le delta lu, avec 0C/0H/0M/0L actif. Sa couverture est partielle : 1 561/23 127 lignes physiques des six fichiers, soit 6,749687 %. Le validateur canonique donne RC0, zéro erreur et zéro avertissement. L’audit163 précédent conserve ses 3 MEDIUM et 2 LOW ; aucun finding historique n’est effacé ou réputé corrigé. Artefacts : /Users/moi/Nextcloud/10.Scripts/64.bridget/audits/2026-10-14/session-2026-10-14-spec-163-t058-01. read_manifest SHA-256 58538c71eeab71f5fcebd97b4fc19bb68f43b137c6e02025359ec6791c17014a.

Limite conservée : le refus du hook AutoDelegate sans request_id est vérifié par lecture de la garde, sans oracle négatif auto spécifique. L’oracle public tools/call prouve le raccord du wire sur un daemon simulé ; il ne prouve pas une Task durable ni un processus fournisseur réel.

T058 reste ouverte. Reconstruction, réinstallation du correctif et recette G8 réelle restent à faire. Compteurs inchangés : SPEC163 53/58, SPEC164 19/22. La source5105378bf4f03376e9bc93e52065cb022f81dee5 reste chargée au dernier checkpoint de runtime ; aucun nouveau commit, binaire ou succès G8/G9 n’est anticipé. T3 source511190beb307e0952066fe1a4b511cb9dd7adb18 reste inchangée. P3 NONCONVERGED historique et poursuite corrective autorisée sont conservés.


## Livraison opérationnelle T058 et recette G8 — 14 octobre 2026

La source Bridget 3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5 est intégrée et poussée sur github/main. Le build Mac donne RC0 en 148,075920 s ; signature ad hoc vérifiée strict RC0, binaire SHA-256 cfb1ff4f91e1b59fbbfd403607d4f6dd08f24229724822f3d8eee9b6eb1a381c. Le build Linux donne RC0 en 73,638388 s, ELF x86-64 SHA-256 dfc9c3c78f7c8e16a394bc57f77d06488acde6495be45093674025be4209af97. Les deux builds portent le même commit de source. Le maître Mac 31298 et le pont 31420 sont nés à 05:32:24/26 CEST ; le worker Cartae 1454489 a pour naissance 146390445 et utilise /home/moi/.local/lib/bridget-communication/163-3efa3ef5a88a/bin/bridget. Ces identités remplacent les anciens checkpoints510.

G8 est prouvée par une admission native réelle, avec les automatismes désactivés. Le fournisseur codex-pro/model gpt-6.1-sol retourne exactement RECETTE163_G8_OK pour Task 8aaba836-5451-4497-bfcc-a90d9614ac59. Le résultat est disponible et transmis (result_sent=true), cleanup_done=true. Le workspace Shared/discovery n’a acquis aucune garde : guarded=false et guard_released=false ne sont pas présentés comme une libération de garde.

La configuration effective /Users/moi/.cache/bridget-core/agents.json porte SHA-256 36c7d13c571f97e49f4caaa59c272597c87735fbfd44667068da77f883399cf4 : auto_engine=true, auto_placement=true et serveur de décision activé, protocole laya_systemone_v1/model typed-decisions. L’endpoint est http://127.0.0.1:8792/v1/systemone ; machine de décision laya-lan, santé/reprise 30 s. Les quatre profils restent configurés ; fixed/fallback local-codex-pro. Seul codex-pro est nouvellement prouvé disponible par G8 ; aucune nouvelle validation de quota/auth des autres profils n’est déduite.

Le rejeu de la même clé après activation ON conserve le même Task, le résultat et le reçu auto entier, avec un seul reçu. Cet état durable prouve l’absence de nouvelle décision et de nouvelle Task sur ce rejeu. Aucune capture réseau n’a été effectuée. La recette HTTP réelle reste T040 R4, avec ses qualifications et son RC1 historique conservés : deux POST réels au total dans R3/R4, aucun nouvel appel de décision ajouté pour G8.

T3 /Applications/T3 Code (Local).app est installée en 0.0.45-local.164/source 511190beb307e0952066fe1a4b511cb9dd7adb18. Ses processus 704/969 restent en version 162, sans redémarrage. Le catalogue d’outils de l’hôte conserve sa version précédente jusqu’à rechargement autorisé ; G8 utilise le MCP propre à la commande. Aucune nouvelle recette runtime T3 n’est revendiquée.

ROOT accepte T039/T043/T044/T058 ; progression 57/58 In Progress, T045 seule ouverte pour la clôture documentaire/Git. Le commit source poussé ne prouve pas encore le push de ces métadonnées finales. P3 NONCONVERGED historique et la poursuite corrective autorisée par l’utilisateur en phase 5 restent tracés ; aucun P4 formel inventé.

Reçu G8 : /Users/moi/.cache/bridget-delivery163-20261013/g8-t058-operational-final.json ; SHA-256 5b1c00923f2f307136b5831cc8d3328227b7284deacf44c793d71542e3ceb52c. Archive opérationnelle : /Users/moi/.cache/bridget-delivery163-20261013/g8-t058-operational-evidence.tgz ; SHA-256 9c1e6c33f936fab753f2fbfaa7fe6196a504c87b44a59d9f77e938cea5521eb8.


## Nettoyage G9 accepté ROOT — 14 octobre 2026

Les onze ressources éligibles retirées comprennent trois worktrees Mac, leurs trois branches fusionnées, deux stages de build Mac et six namespaces TMP du lot. Aucun retrait forcé, gain physique de disque ni mutation des bases de production n’est revendiqué. Les premiers refus de nettoyage restent historiques.

Le dernier scan du worktree T3 163 a été complet : 282 139 fichiers dans la fenêtre 180 s. L’inventaire a ensuite changé après l’archive, donc la garde de stabilité a échoué. Le worktree /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission et sa branche sont conservés. Cette cause remplace le seul constat historique du scan 60 s incomplet ; aucune cause d’inode ou mutation de contenu n’est inventée. Reçu : /Users/moi/.cache/bridget-delivery163-20261013/g9-t3wt163-complete-retirement.json ; SHA-256 7e2388b90e22f0b33bb3f6140851c7dedc1a398a5b5d10957a490153d4e13d29.

Sur Cartae, sudo lsof retourne RC0 mais signale fuse.portal et une visibilité incomplète. Le worktree /home/moi/10.Scripts/64.bridget/.worktrees/163-decision-admission, sa branche, son .connector164, le cache Cargo partagé et les TMP de tests164 restent conservés. Les sources suivies du worktree/main sont alignées sur 3efa et propres. Le refus de la garde d’usage n’est pas contourné.

Les stages de paquet /Users/moi/.cache/t3-final162.2nosz6zd et /Users/moi/.cache/t3-final163.zsb7fmfu contiennent chacun une base persistante de cache SQLite de 20 480 octets ; ils restent protégés, sans prétexte de futur build163. L’ancienne application /Applications/.T3 Code (Local).app.new-163 reste chargée par les processus T3 protégés. Sauvegardes, preuves, retours arrière, bases et caches partagés sont conservés.

G9 est accepté sur ces retraits et conservations nommées. Reçu final : /Users/moi/.cache/bridget-delivery163-20261013/g9-t058-cleanup-final.json ; SHA-256 08091fc94791644308b258f2eb912ee375145d66004ef9f9c5e0d822a34fb5c3. Les preuves d’exécution e397 et de retrait T3 164 b066 restent rattachées au reçu final. Aucun fichier de production ou service n’est touché par cet ajout documentaire.


## Limites conservées à la clôture

L’audit 163 reste A/99,047619 partiel : 0C/0H/3M/2L, couverture 4,655325 % du snapshot et 0,355837 % du contexte global. Le delta T058 reste A/100 partiel 6,749687 %, 0 finding actif, validateur RC0/0 erreur/0 avertissement. Le delta T3 R6 conserve 0 finding actif sur son seul périmètre. Ces audits ne deviennent pas des audits globaux du produit.

Les suites FULL antérieures avec échecs restent historiques et ne sont pas réécrites GREEN. Les campagnes ciblées réussies restent rattachées à leurs sources. Aucun FULL répété, benchmark nouveau, nouveau test du hook auto sans request_id ou nouveau processus fournisseur issu du test de dispatch n’est inventé. Les douze exacts T058 restent 11 lib + 1 intégration ; le zéro-filtre initial reste exclu.

L’ETA initiale du plan 300–540 min et celle après Tasks 360–600 min sont historiques. La dernière fenêtre recalibrée 05:30–06:00 CEST n’est pas une mesure CPU. ROOT situe approximativement le début du planning au 13 octobre 2026 à 19:52:55 CEST. Le reçu G8 du 14 octobre vers 05:38:41 CEST intervient près de 9 h 46 plus tard ; ce temps écoulé approximatif inclut attente, incidents, revues, tests, builds et maintenance, sans attribution de temps processeur.


## Clôture T045 — 58/58, 14 octobre 2026

ROOT a intégré et poussé les documents de livraison G8/G9 au commit c57cfd35efa1338cda5256a02c57d71258d8b8d4 sur github/main. Le reçu vérifie HEAD égal à la référence distante et dépôt propre à ce checkpoint. T045 est acceptée sur cette preuve : 58/58, Implemented ; Pipeline : Livré avec conservations justifiées. Le checkpoint 57/58 précédent, intégré dans c57, reste historique.

Preuve : /Users/moi/.cache/bridget-delivery163-20261013/git-final-doc-checkpoint-c57cfd35.json ; SHA-256 56316fdcb4202865fbb57e7979a11d302bb968e2170453a9523070a490a30390. Le build actif reste 3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5. Le commit documentaire c57 n’est pas un nouveau binaire. Les marqueurs Implemented/58/58 sont intégrés et poussés dans af8998a7d7494913d4420e2dca7354a73bd8a4f5. ROOT a vérifié github/main égal HEAD propre ; les sources suivies de Cartae main/worktree sont alignées sur ce commit documentaire, sans changement du build3efa. Preuve d’alignement Cartae : /Users/moi/.cache/bridget-delivery163-20261013/git-final-cartae-af8998a7-command.json ; SHA-256 a8566f602017d52ea0ca9ab3706871038275aa04477b67077fb572b74d638620.

Les conservations G9 sont acceptées et nommées, sans suppression forcée. Anciennes FULL non GREEN, audits partiels et leurs findings, P3 NONCONVERGED historique et poursuite corrective autorisée par l’utilisateur en phase 5 sont maintenus. Aucun nouveau test, audit global, service ou P4 formel n’est déduit. L’ETA initiale 300–540 min et l’ETA après Tasks 360–600 min restent historiques ; dernière fenêtre 05:30–06:00 CEST. Aucun temps écoulé exact de développement ou CPU n’est ajouté.
