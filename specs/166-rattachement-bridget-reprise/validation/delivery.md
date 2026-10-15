# Préparation de livraison — SPEC-166

Date : 2026-10-15.

## Source

- Dépôt T3 : `/Users/moi/11.Repositories/t3code-local`
- Branche intégrée et poussée : `local/main-20261009`
- Commit source : `76780a20bb22b9390eab2389eaf0db87e1349e7e`
- Message : `fix(codex): Remonter l'identité Bridget avant un tour`

## Paquet

- Version : `0.0.45-local.166`
- Stage privé : `/Users/moi/.cache/t3-final166.K49J0ZDU`
- Archive : `/Users/moi/.cache/t3-final166.K49J0ZDU/artifacts/T3-Code-0.0.45-local.166-arm64.zip`
- SHA-256 archive : `8fbf8c9a7043bb0e263939fb70cb77643805f790ca71c9222abcd19b38b3328c`
- SHA-256 ASAR : `b397aaa905d5c023392afee8459c6d6150a576fc7ef5d18987c0dafb822fb433`
- Signature : 38 objets signés, droits préservés, vérification stricte profonde conforme.

Le premier contrôle post-construction a échoué pendant la recherche du stage natif. La profondeur maximale du `find` était trop faible. La construction et le paquet étaient déjà terminés. Le contrôle corrigé a trouvé un seul stage, puis la signature a réussi.

## Installation

- Application installée : `/Applications/T3 Code (Local).app`
- Version sur disque : `0.0.45-local.166`
- Rollback : `/Users/moi/.cache/t3-install166.VqBHdpl2/T3-before.app`
- Version du rollback : `0.0.45-local.164`
- Données utilisateur : inchangées.

La version 166 a ensuite été activée. Une erreur de livraison a provoqué une boucle de relance : `launchctl submit` a traité le script temporaire comme une tâche persistante. Le script a été exécuté 71 fois avant le retrait du service. Le diagnostic a montré des arrêts propres, pas des crashs du code T3.

Le service `com.bridget.t3-activate166` a été retiré. Le script `/Users/moi/.cache/t3-activate166.sh` a été rendu inerte. T3 est ensuite resté sur le même processus. La recette réelle est consignée dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/166-rattachement-bridget-reprise/validation/livraison166.md`.
