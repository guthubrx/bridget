# Livraison164 — intégration, paquet et recette acceptés

**Statut : In Progress,19/22.** La source est intégrée et poussée vers le fork T3. Le paquet signé est installé sur disque. La recette web isolée S164-12 est acceptée par ROOT. T018/G8, T019/G9 et T020 restent ouverts : ce reçu ne clôt pas la livraison.

## Preuves durables

Source T3 intégrée/poussée : `511190beb307e0952066fe1a4b511cb9dd7adb18`, commit source164 `966f296ac70c3134bc6e91a722cfe86e726f50e3`, fork `refs/heads/local/main-20261009`. Reçu : `/Users/moi/.cache/bridget-delivery164-20261013/git-t3-source164-pushed.json`, SHA-256 `e7685a463e56580b511caa1afb76eb1fa6d7c53612025122506088be677c0259`.

Paquet commun163/164 version0.0.45-local.164 signé, une compilation native. Reçu : `/Users/moi/.cache/bridget-delivery164-20261013/t3-package164-result.json`, SHA-256 `2fb1b8f95140f09676d107f2429be07322c7ff77ff094993f1a82f95dff18984`. R1 conserve2574PASS/2FAIL des oracles ZIP ; R2 rejoue seulement les deux oracles affectés,2PASS/0FAIL. Aucun nouveau succès global2576 annoncé.

Recette S164-12 : `/Users/moi/.cache/bridget-delivery164-20261013/ui-recipe/t017164-final.json`, SHA-256 `fcf03a0e7f86ca30b13497df82a0fd9b512232e5db516eb0bd11f22fb9d47f6f`. Web réel issu du paquet final, données synthétiques, sept captures. Archive66membres vérifiés : `/Users/moi/.cache/bridget-delivery164-20261013/ui-recipe/ui-recipe164-evidence.tar.gz`, SHA-256 `ef4fa8ed527f20ccbc1ba89d467bf02096b67112eae258c1ccd1173786c6cd99`. Viewports1280×900 et390×844 ; le second reste un rendu web mobile, pas une recette React Native. Aucun redémarrage Electron/T3, modèle ou agent réel utilisé dans la recette.

Installation signée sur disque : `/Users/moi/.cache/bridget-delivery163-20261013/t3-backup163164/T3-install-receipt.json`, SHA-256 `b73ddc5019a644fc9964372816089f95cfffdd276b319ad0bbef5196429fa2f0`. Application `/Applications/T3 Code (Local).app` en0.0.45-local.164 ; app.asar SHA-256 `6eb2fbdfed44a603d0a66d81d92fadd6d3486ff1ff43a880fe788c7d39fc923f`. PID704/969 et dates de naissance inchangés. Le code162 reste chargé selon le checkpoint ROOT. Pas d'activation différée ni de faux handshake G8.

## Paquet et rollback

Archive signée : `/Users/moi/.cache/t3-final163.zsb7fmfu/artifacts/T3-Code-0.0.45-local.164-arm64-signed.zip`, SHA-256 `8e28b1e186a7bc5a35a9ed3c007f3b8cfd682056df8590a462e413dd8732b3b1`. Signature strict/deep RC0, entitlements conservés. Le ZIP non signé V1 et les deux échecs initiaux restent historiques. Build natif unique176,676s ; réarchivage signé11,943s sans nouvelle compilation. Vérification des deux oracles ZIP corrigés1,68s. Aucun total global GREEN recomposé.

Rollback conservé : `/Users/moi/.cache/bridget-delivery163-20261013/t3-backup163164/T3-before.app`. Chemin d'échange conservé : `/Applications/.T3 Code (Local).app.new-163`. Ces chemins ne sont pas des cibles de suppression autorisées par ce reçu.

## Limites et reste à faire

La recette prouve le client web construit, avec données synthétiques, en taille desktop et mobile. Elle ne prouve ni React Native sur appareil ni Electron actif. Les processus T3 existants continuent leur ancien code162 sans redémarrage. Bridget reste indépendant de T3 ; aucune nouvelle autorité machine ou décision de modèle n'est déduite de la fixture.

ROOT fournira la preuve G8 indépendante avant T018. G9/nettoyage n'est pas effectué ; T019 reste ouverte. Métadonnées et reçu finalT020 attendent ces gates. Aucun Git, service, test, cache ou installation exécuté par l'owner documentaire.
