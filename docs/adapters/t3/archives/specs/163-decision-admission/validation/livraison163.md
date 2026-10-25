# Archives techniques de l’adaptateur T3

Document d’origine : [specs/163-decision-admission/validation/livraison163.md](../../../../../../../specs/163-decision-admission/validation/livraison163.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

T3 /Applications/T3 Code (Local).app remplacée atomiquement en 0.0.45-local.164/source 511190beb307e0952066fe1a4b511cb9dd7adb18, signature stricte/deep RC0 et entitlements préservés. PID 704/969 et naissances 17:31:27/33 du 13 octobre restent inchangés : aucun redémarrage, ancien code encore chargé. Aucun réveil différé programmé.

<a id="extrait-2"></a>

## Extrait 2

- /Users/moi/.cache/bridget-delivery163-20261013/joint-backup-20261014-163164-retry-projection/progress.json ; SHA-256 b6da23d355fbb396b50fab7c946dd5693dd6ced27aafe5ae3ac2aaffaf7cef16
- /Users/moi/.cache/bridget-delivery163-20261013/t3-backup163164/T3-install-receipt.json ; SHA-256 b73ddc5019a644fc9964372816089f95cfffdd276b319ad0bbef5196429fa2f0
- /Users/moi/.cache/bridget-delivery163-20261013/g8-registry-off-applied.json ; SHA-256 ed358ea42dc927adf19d4c4e7cca2ecdd0de76e2aa2a213116f6b15049b26aae

<a id="extrait-3"></a>

## Extrait 3

T041 : Bridget 5105378bf4f03376e9bc93e52065cb022f81dee5 intégrée/poussée vers github/main, push vérifié. Cartae main a reçu ce commit par avance rapide et était propre au reçu du 14 octobre 02:09:54Z. T3 511190beb307e0952066fe1a4b511cb9dd7adb18 poussée vers fork, refs/heads/local/main-20261009. La précédente intégration T3 8dadcb… non poussée reste historique. Les builds source 5105378 et le paquet T3 version 164/source 511190 prouvent la préparation des fichiers, pas leur installation ou activation actuelle.

<a id="extrait-4"></a>

## Extrait 4

La reprise avant installation a redémarré seulement les anciens services initialement actifs, source 1dc0c6022e8d0712fbc4c65a8c20be7b2ea8a437 : daemon 86807 né 04:48:21 CEST, pont 86830 né 04:48:23 CEST, worker 1403968, naissance 146126086. Aucun fichier ou DB restauré. Ces identités restent historiques, sans observation actuelle ni activation 163 revendiquée. T3 et tunnel étaient protégés dans ce reçu.

<a id="extrait-5"></a>

## Extrait 5

- /Users/moi/.cache/bridget-delivery163-20261013/git-bridget-source163164-pushed.json ; SHA-256 863cef70702815e6205bb1b2dfc0c617bf5759231a7e501d12810fb026e249e2
- /Users/moi/.cache/bridget-delivery163-20261013/git-cartae-main-source163164.json ; SHA-256 a053317356cf11f356b5088caac487d988b2fd326be080d5d9270456738030f9
- /Users/moi/.cache/bridget-delivery163-20261013/joint-backup-20261014-163164-020006Z/progress.json ; SHA-256 5905efff9912d6b78e76b5d505a668cec4aee2c7a01ffad47e771d5999096dd6
- /Users/moi/.cache/bridget-delivery163-20261013/preparation-upgrade163-retry-projection.json ; SHA-256 ad2286bca15d135516ce3e661c55ecce95804f4835a250a32b1e4b09904dce66
- /Users/moi/.cache/bridget-delivery163-20261013/lab163-preparation/t040-r4-cleanup-conclusions-supplement.json ; SHA-256 54056f844075544f45ae83a8072f7a6c1421ea16bf391d32458a05726d4c8d49
- /Users/moi/.cache/bridget-delivery164-20261013/git-t3-source164-pushed.json ; SHA-256 e7685a463e56580b511caa1afb76eb1fa6d7c53612025122506088be677c0259

<a id="extrait-6"></a>

## Extrait 6

Le MCP propre à la nouvelle commande annonce bridget_auto_delegate dans tools/list, mais tools/call renvoie JSON-RPC -32602, message outil inconnu, request_id g8-spec163-a73f9432-53d5-46b5-94b9-e8f0ee3cfe7e. Les attestations officielles immédiatement avant appel portent version2 et permissions_present=true ; provider codex-pro déclaré disponible et capacité maître contrôlée. Aucun reçu auto/Task sous cette clé selon la vérification durable ROOT. La cause confirmée ROOT est le dispatcher de /Users/moi/Nextcloud/10.Scripts/64.bridget/crates/bridget-daemon/src/mcp.rs:753–760 : routage auto omis malgré l’annonce. Scope/hook/wrapper doivent être raccordés à la voie native existante. Ce refus ne démontre pas un problème de quota ou d’authentification et ne nécessite pas de redémarrage T3.

<a id="extrait-7"></a>

## Extrait 7

T058 reste dans la session163 autorisée : correctif minimal sur Cartae, oracles MCP existants affectés, contre-revue/delta d’audit ciblés, build et livraison du correctif avant reprise G8. Aucun nouveau framework, banc de confort ou FULL. Le prochain commit source sera différent de5105378bf4f03376e9bc93e52065cb022f81dee5 ; aucun hash futur fixé. T038/T042 demeurent cochées pour les builds/installations réellement prouvés du510, pas pour le futur correctif. T3 source511190beb307e0952066fe1a4b511cb9dd7adb18/version164 est déjà installée et inchangée. Aucun résultat antérieur effacé, aucun livré final ou G8/G9 déduit.

<a id="extrait-8"></a>

## Extrait 8

T058 reste ouverte. Reconstruction, réinstallation du correctif et recette G8 réelle restent à faire. Compteurs inchangés : SPEC163 53/58, SPEC164 19/22. La source5105378bf4f03376e9bc93e52065cb022f81dee5 reste chargée au dernier checkpoint de runtime ; aucun nouveau commit, binaire ou succès G8/G9 n’est anticipé. T3 source511190beb307e0952066fe1a4b511cb9dd7adb18 reste inchangée. P3 NONCONVERGED historique et poursuite corrective autorisée sont conservés.

<a id="extrait-9"></a>

## Extrait 9

T3 /Applications/T3 Code (Local).app est installée en 0.0.45-local.164/source 511190beb307e0952066fe1a4b511cb9dd7adb18. Ses processus 704/969 restent en version 162, sans redémarrage. Le catalogue d’outils de l’hôte conserve sa version précédente jusqu’à rechargement autorisé ; G8 utilise le MCP propre à la commande. Aucune nouvelle recette runtime T3 n’est revendiquée.

<a id="extrait-10"></a>

## Extrait 10

Le dernier scan du worktree T3 163 a été complet : 282 139 fichiers dans la fenêtre 180 s. L’inventaire a ensuite changé après l’archive, donc la garde de stabilité a échoué. Le worktree /Users/moi/11.Repositories/t3code-local/.worktrees/163-decision-admission et sa branche sont conservés. Cette cause remplace le seul constat historique du scan 60 s incomplet ; aucune cause d’inode ou mutation de contenu n’est inventée. Reçu : /Users/moi/.cache/bridget-delivery163-20261013/g9-t3wt163-complete-retirement.json ; SHA-256 7e2388b90e22f0b33bb3f6140851c7dedc1a398a5b5d10957a490153d4e13d29.

<a id="extrait-11"></a>

## Extrait 11

Les stages de paquet /Users/moi/.cache/t3-final162.2nosz6zd et /Users/moi/.cache/t3-final163.zsb7fmfu contiennent chacun une base persistante de cache SQLite de 20 480 octets ; ils restent protégés, sans prétexte de futur build163. L’ancienne application /Applications/.T3 Code (Local).app.new-163 reste chargée par les processus T3 protégés. Sauvegardes, preuves, retours arrière, bases et caches partagés sont conservés.

<a id="extrait-12"></a>

## Extrait 12

G9 est accepté sur ces retraits et conservations nommées. Reçu final : /Users/moi/.cache/bridget-delivery163-20261013/g9-t058-cleanup-final.json ; SHA-256 08091fc94791644308b258f2eb912ee375145d66004ef9f9c5e0d822a34fb5c3. Les preuves d’exécution e397 et de retrait T3 164 b066 restent rattachées au reçu final. Aucun fichier de production ou service n’est touché par cet ajout documentaire.

<a id="extrait-13"></a>

## Extrait 13

L’audit 163 reste A/99,047619 partiel : 0C/0H/3M/2L, couverture 4,655325 % du snapshot et 0,355837 % du contexte global. Le delta T058 reste A/100 partiel 6,749687 %, 0 finding actif, validateur RC0/0 erreur/0 avertissement. Le delta T3 R6 conserve 0 finding actif sur son seul périmètre. Ces audits ne deviennent pas des audits globaux du produit.
