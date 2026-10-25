# Archives techniques de l’adaptateur T3

Document d’origine : [specs/164-conversation-t3-distante/validation/livraison164.md](../../../../../../../specs/164-conversation-t3-distante/validation/livraison164.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

**Statut : Implemented,22/22. Pipeline : Livré.** Source T3 intégrée et poussée, paquet signé installé, recette web isolée acceptée. G8/G9 et conservations justifiées acceptés par ROOT. T020 clôturée sur le push documentaire réelc57. Le commit des cases ferméesaf899 est intégré, poussé et vérifié.

<a id="extrait-2"></a>

## Extrait 2

Source T3 intégrée/poussée : `511190beb307e0952066fe1a4b511cb9dd7adb18`, commit source164 `966f296ac70c3134bc6e91a722cfe86e726f50e3`, fork `refs/heads/local/main-20261009`. Reçu : `/Users/moi/.cache/bridget-delivery164-20261013/git-t3-source164-pushed.json`, SHA-256 `e7685a463e56580b511caa1afb76eb1fa6d7c53612025122506088be677c0259`.

<a id="extrait-3"></a>

## Extrait 3

Paquet commun163/164 version0.0.45-local.164 signé, une compilation native. Reçu : `/Users/moi/.cache/bridget-delivery164-20261013/t3-package164-result.json`, SHA-256 `2fb1b8f95140f09676d107f2429be07322c7ff77ff094993f1a82f95dff18984`. R1 conserve2574PASS/2FAIL des oracles ZIP ; R2 rejoue seulement les deux oracles affectés,2PASS/0FAIL. Aucun nouveau succès global2576 annoncé.

<a id="extrait-4"></a>

## Extrait 4

Recette S164-12 : `/Users/moi/.cache/bridget-delivery164-20261013/ui-recipe/t017164-final.json`, SHA-256 `fcf03a0e7f86ca30b13497df82a0fd9b512232e5db516eb0bd11f22fb9d47f6f`. Web réel issu du paquet final, données synthétiques, sept captures. Archive66membres vérifiés : `/Users/moi/.cache/bridget-delivery164-20261013/ui-recipe/ui-recipe164-evidence.tar.gz`, SHA-256 `ef4fa8ed527f20ccbc1ba89d467bf02096b67112eae258c1ccd1173786c6cd99`. Viewports1280×900 et390×844 ; le second reste un rendu web mobile, pas une recette React Native. Aucun redémarrage Electron/T3, modèle ou agent réel utilisé dans la recette.

<a id="extrait-5"></a>

## Extrait 5

Installation signée sur disque : `/Users/moi/.cache/bridget-delivery163-20261013/t3-backup163164/T3-install-receipt.json`, SHA-256 `b73ddc5019a644fc9964372816089f95cfffdd276b319ad0bbef5196429fa2f0`. Application `/Applications/T3 Code (Local).app` en0.0.45-local.164 ; app.asar SHA-256 `6eb2fbdfed44a603d0a66d81d92fadd6d3486ff1ff43a880fe788c7d39fc923f`. PID704/969 et dates de naissance inchangés. Le code162 reste chargé selon le checkpoint ROOT. Pas d'activation différée ni de faux handshake G8.

<a id="extrait-6"></a>

## Extrait 6

Archive signée : `/Users/moi/.cache/t3-final163.zsb7fmfu/artifacts/T3-Code-0.0.45-local.164-arm64-signed.zip`, SHA-256 `8e28b1e186a7bc5a35a9ed3c007f3b8cfd682056df8590a462e413dd8732b3b1`. Signature strict/deep RC0, entitlements conservés. Le ZIP non signé V1 et les deux échecs initiaux restent historiques. Build natif unique176,676s ; réarchivage signé11,943s sans nouvelle compilation. Vérification des deux oracles ZIP corrigés1,68s. Aucun total global GREEN recomposé.

<a id="extrait-7"></a>

## Extrait 7

Rollback conservé : `/Users/moi/.cache/bridget-delivery163-20261013/t3-backup163164/T3-before.app`. Chemin d'échange conservé : `/Applications/.T3 Code (Local).app.new-163`. Ces chemins ne sont pas des cibles de suppression autorisées par ce reçu.

<a id="extrait-8"></a>

## Extrait 8

La recette prouve le client web construit, avec données synthétiques, en taille desktop et mobile. Elle ne prouve ni React Native sur appareil ni Electron actif. Les processus T3 existants continuent leur ancien code162 sans redémarrage. Bridget est autonome ; aucune nouvelle autorité machine ou décision de modèle n'est déduite de la fixture.

<a id="extrait-9"></a>

## Extrait 9

T3 : source `511190beb307e0952066fe1a4b511cb9dd7adb18` poussée vers le fork/local/main-20261009. Application signée `/Applications/T3 Code (Local).app` installée en0.0.45-local.164 ; PID704/969 et naissances inchangés. Code162 encore chargé, aucun redémarrage ni activation différée. Bridget demeure natif Bridget.

<a id="extrait-10"></a>

## Extrait 10

Nettoyage T3 WT164 et branche attesté : `/Users/moi/.cache/bridget-delivery163-20261013/g9-t3wt164-fresh-stable-retirement.json`, SHA-256 `b0661c19c7243969d90035dfd8a9de062e2a823e5ae2183c78ac8fa6553f6b1b` ; retrait sans force après source intégrée/poussée et garde lsof complète,321991571octets logiques. Les anciens chemins WT164 sont historiques. G9 final : `/Users/moi/.cache/bridget-delivery163-20261013/g9-t058-cleanup-final.json`, SHA-256 `08091fc94791644308b258f2eb912ee375145d66004ef9f9c5e0d822a34fb5c3`. Onze ressources retirées : trois worktrees, deux stages Mac et six namespaces TMP ; trois branches retirées. Le WT T3 de163 et sa branche sont conservés : inventaire complet282139fichiers réussi, puis inventaire exact changé après archive. La garde refuse le retrait ; aucune cause de changement inventée, aucun contournement.

<a id="extrait-11"></a>

## Extrait 11

Tests sources :28PASSR6 distincts des10réducteurR4 et8MarkdownR1 conservés, soit46preuves de tests qualifiées par provenance, pas une nouvelle campagne46 ni un secondFULL. L'échec historique de rejeu/réalignement reste historique ; sa correction est couverte par l'oracleR6, aucun replay supplémentaire déduit de ce texte. Paquet :2574PASS/2ZIPFAILR1 conservés puis deux seuls oracles ZIP PASSR2 après réarchivage ; pas de nouveauGREEN2576. P1/P2/P3, contre-revues et audits ciblés restent inchangés ; aucun succès global/runtime de T3 inventé.

<a id="extrait-12"></a>

## Extrait 12

Les caches `/Users/moi/.cache/t3-final162.2nosz6zd` et `/Users/moi/.cache/t3-final163.zsb7fmfu` sont conservés pour leurs bases SQLite persistantes réelles,20480octets chacune, aux chemins `/Users/moi/.cache/t3-final162.2nosz6zd/source/node_modules/.vite/task-cache/v18/cache.db` et `/Users/moi/.cache/t3-final163.zsb7fmfu/source/node_modules/.vite/task-cache/v18/cache.db`. Ces dépendances ne sont pas conservées sous prétexte d'un futur build163. L'ancien échange `/Applications/.T3 Code (Local).app.new-163` reste utilisé par les processus T3 protégés ; le rollback et les preuves restent conservés. Cache Cargo partagé `/Users/moi/Nextcloud/10.Scripts/64.bridget/target/debug` conservé, permis lourd rendu ; aucun nouveau FULL ou build163 en attente prétendu. Les trois namespaces SQLite `/tmp/b163.uqd3443p`, `/tmp/b163.dxmfq_la` et `/tmp/b163.07di472q` sont protégés par leurs attestations historiques ; leur existence actuelle n'est pas revendiquée. Aucun gain physique ni modification DB de production revendiqué.

<a id="extrait-13"></a>

## Extrait 13

Le build actif Bridget est toujours source `3efa3ef5a88a9a00b104f53c5183ba68dbb11fd5` ; le commit de métadonnéesc57 est distinct et ne prétend pas une nouvelle compilation. Source T3 `511190beb307e0952066fe1a4b511cb9dd7adb18` poussée au fork/local/main-20261009, application164 signée sur disque. T3PID704/969 reste chargé en162, sans redémarrage ni activation différée.

<a id="extrait-14"></a>

## Extrait 14

Source T3 `3ea39f11095debbba6ca6a73f99f44fbe308cd60` intégrée et poussée vers fork/local/main-20261009. Application `/Applications/T3 Code (Local).app` installée et signée, version `0.0.45-local.164`, ASAR SHA-256 `dc5623d70f45c85909703274ba59d216295b9e84b1e50ab8685b75aabe73f942`. Aucun redémarrage : processus courants conservés. Reçu `/Users/moi/.cache/t3-lineage-icons-164-backup/T3-install-receipt.json` ; preuves `/Users/moi/.cache/t3-lineage-icons-164-proofs`.
