# Archives techniques de l’adaptateur T3

Document d’origine : [specs/141-messages-groupes/implementation.md](../../../../../../specs/141-messages-groupes/implementation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-07. Début : 07:30 CEST. Estimation totale : 20–35 minutes.
Statut : Implemented, installé, activation différée. Code gelé à 07:50:41 CEST. Dix tâches validées sur preuves. Installation adjacente vérifiée sans relance ni activation de T3.

<a id="extrait-2"></a>

## Extrait 2

Deux essais preview_resize ont expiré ; le viewport natif est resté à 2074 pixels. La largeur réelle de 320 pixels a donc été testée dans une iframe du T3 natif, avec son CSS réel. En sombre, un nom long de 300 caractères Unicode garde largeur et débordement mesurés à 320 ; carte à 259,2656, libellé réparti sur 300 pixels de hauteur, overflow-wrap:anywhere. Une iframe fraîche en clair avec le même nom garde largeur et débordement à 320, classe de thème vide, fond de corps relevé à .992 et aucune source. Le premier corps long reste complet sur dix paragraphes à 320 pixels, sans source secondaire ni exécution de contenu hostile.

<a id="extrait-3"></a>

## Extrait 3

Capture sombre : /Users/moi/.t3/userdata/browser-artifacts/browser-screenshot-127-0-0-1-muxp14lx-25fd3477.png
Capture claire : /Users/moi/.t3/userdata/browser-artifacts/browser-screenshot-127-0-0-1-muxp1vid-2e2033d4.png

<a id="extrait-4"></a>

## Extrait 4

Capture finale du renderer inspectée par le principal : /Users/moi/.t3/userdata/browser-artifacts/browser-screenshot-127-0-0-1-muxp5nb3-4bb46026.png.

<a id="extrait-5"></a>

## Extrait 5

Temps de réalisation transmis par le principal : environ trente minutes depuis 07:30 CEST. Le bilan inclut 294 tests PASS, contrôles web, recette native, copie exacte interceptée/restaurée, Converge et audit. Aucun essai fournisseur de bout en bout ni installation de production. La copie native de fixture t3-context reste couverte uniquement par les tests du composant.

<a id="extrait-6"></a>

## Extrait 6

Le 2026-10-07, l'utilisateur a autorisé commit, fusion, push et installation de la session141. Il a interdit de relancer T3. Le principal réalise les opérations Git et l'installation. Les processus et conversations T3 en cours doivent être conservés. La nouvelle présentation reste non activée jusqu'à une preuve distincte.

<a id="extrait-7"></a>

## Extrait 7

Preuves transmises par le principal : commit T3 5724eb7f12e4556327efa14f51f43b9caf404e25 ; avance rapide vers local/v0.0.45 ; pushes sur fork confirmés pour local/v0.0.45 et session141. Commit portable cf0bd9903181327a793be06ccc23b2d402090abd poussé et présence distante vérifiée.

<a id="extrait-8"></a>

## Extrait 8

Le paquet adjacent /Applications/T3 Code (Local SPEC141).app est en préparation dans /Users/moi/.cache/t3-spec141-package.OaEnOF. L'application active /Applications/T3 Code (Local).app reste en place. Les PID 85017 et 85080 ont les mêmes dates de démarrage à 07:21 ; son archive a pour SHA256 dab141939a4171c3b4e7169fc68346b498ce179c414f9e2e8ee6acdb42f89ba1 et son contrôle codesign strict passe. La sauvegarde privée /Users/moi/.cache/t3-adoptions/spec141-20261007.llKRu1/state-before-install.sqlite a suivi VACUUM depuis la source en lecture seule puis quick_check=ok, sortie 0. Aucune installation, activation ou relance n'est déclarée terminée.

<a id="extrait-9"></a>

## Extrait 9

Artefacts préparatoires Bridget : commit 7f4d65fb04261c09f420a2c907b6eeeecd1e9eaf, avance rapide vers main, pushes github main et session141 confirmés, valeurs distantes ls-remote identiques. Code T3 : 5724eb7f12e4556327efa14f51f43b9caf404e25 ; commit portable : cf0bd9903181327a793be06ccc23b2d402090abd, déjà poussés. Tests fraîchement rejoués : 294/294 PASS.

<a id="extrait-10"></a>

## Extrait 10

Packaging normal --target zip : sortie 0, environ 89 secondes, source 5724eb7f12e4, version 0.0.45-local.141. L'application /Applications/T3 Code (Local SPEC141).app est installée à côté. Signature ad hoc appliquée par le principal après packaging. Vérification codesign stricte et profonde : sortie 0 sur l'application de staging et sur l'application installée. SHA256 ASAR installé : f4fd81f645b81c65bd8a0471022931c9e2e789df837d1b42c9ffa693151c90da.

<a id="extrait-11"></a>

## Extrait 11

Le principal a extrait le chunk _chat-DqlgvoJO.js de l'application installée et comparé son SHA256 53aa48551495f867385170d4e5a490aac82004ce5932bff1a291295533b32505. Comparaison et présence des marqueurs data-bridget-member-toggle / Messages groupés : PASS. L'archive zip non signée reste conservée dans /Users/moi/.cache/t3-spec141-package.OaEnOF/T3-Code-0.0.45-local.141-arm64.zip, SHA256 f38ae9b8322061d6d762e6b3fbe6dfd2b4d67dc330a167b230befa00beb53c9c. La signature porte sur l'application après packaging ; elle n'est pas revendiquée pour le zip.

<a id="extrait-12"></a>

## Extrait 12

L'application active /Applications/T3 Code (Local).app reste intacte, archive SHA256 dab141939a4171c3b4e7169fc68346b498ce179c414f9e2e8ee6acdb42f89ba1 et signature stricte PASS. PID 85017/85080 et dates de démarrage à 07:21 inchangés. L'endpoint public de l'environnement dont l'identifiant commence par 3b1ba3d1 répond en HTTP après installation. La sauvegarde d'état a un quick_check=ok. Aucun quit, open ou restart. Aucun changement du daemon ni d'Agent Loop.

<a id="extrait-13"></a>

## Extrait 13

La présentation141 n'est pas activée. Quand l'utilisateur choisira de quitter l'ancienne application, il devra ouvrir /Applications/T3 Code (Local SPEC141).app. Relancer /Applications/T3 Code (Local).app ne charge pas141. L'audit et les rapports précédents restent les preuves historiques de la source validée ; la livraison est prouvée séparément ici.
