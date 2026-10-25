# Archives techniques de l’adaptateur T3

Document d’origine : [specs/166-rattachement-bridget-reprise/validation/delivery.md](../../../../../../../specs/166-rattachement-bridget-reprise/validation/delivery.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Dépôt T3 : `/Users/moi/11.Repositories/t3code-local`
- Branche intégrée et poussée : `local/main-20261009`
- Commit source : `76780a20bb22b9390eab2389eaf0db87e1349e7e`
- Message : `fix(codex): Remonter l'identité Bridget avant un tour`

<a id="extrait-2"></a>

## Extrait 2

- Version : `0.0.45-local.166`
- Stage privé : `/Users/moi/.cache/t3-final166.K49J0ZDU`
- Archive : `/Users/moi/.cache/t3-final166.K49J0ZDU/artifacts/T3-Code-0.0.45-local.166-arm64.zip`
- SHA-256 archive : `8fbf8c9a7043bb0e263939fb70cb77643805f790ca71c9222abcd19b38b3328c`
- SHA-256 ASAR : `b397aaa905d5c023392afee8459c6d6150a576fc7ef5d18987c0dafb822fb433`
- Signature : 38 objets signés, droits préservés, vérification stricte profonde conforme.

<a id="extrait-3"></a>

## Extrait 3

- Application installée : `/Applications/T3 Code (Local).app`
- Version sur disque : `0.0.45-local.166`
- Rollback : `/Users/moi/.cache/t3-install166.VqBHdpl2/T3-before.app`
- Version du rollback : `0.0.45-local.164`
- Données utilisateur : inchangées.

<a id="extrait-4"></a>

## Extrait 4

La version 166 a ensuite été activée. Une erreur de livraison a provoqué une boucle de relance : `launchctl submit` a traité le script temporaire comme une tâche persistante. Le script a été exécuté 71 fois avant le retrait du service. Le diagnostic a montré des arrêts propres, pas des crashs du code T3.

<a id="extrait-5"></a>

## Extrait 5

Le service `com.bridget.t3-activate166` a été retiré. Le script `/Users/moi/.cache/t3-activate166.sh` a été rendu inerte. T3 est ensuite resté sur le même processus. La recette réelle est consignée dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/166-rattachement-bridget-reprise/validation/livraison166.md`.
