# Archives techniques de l’adaptateur T3

Document d’origine : [specs/141-messages-groupes/livraison.md](../../../../../../specs/141-messages-groupes/livraison.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-07. État : implemented_installed_activation_deferred. Commits et pushes confirmés. Application adjacente installée et vérifiée. Présentation141 non activée dans T3 en cours.

<a id="extrait-2"></a>

## Extrait 2

L'utilisateur autorise commit, fusion, push et installation de la session141. Il interdit de relancer T3. Le principal réalise les opérations Git et l'installation. L'agent documentaire ne committe pas et n'installe pas.

<a id="extrait-3"></a>

## Extrait 3

T3 n'a été ni quitté, ni ouvert, ni redémarré. Ses processus et conversations restent en cours. L'installation sur disque est vérifiée ; l'activation dans l'application ouverte reste différée. Aucune activation141 n'est revendiquée.

<a id="extrait-4"></a>

## Extrait 4

| Fait | État actuel | Preuve consignée |
|---|---|---|
| Commit T3 | Confirmé par le principal | 5724eb7f12e4556327efa14f51f43b9caf404e25, périmètre des quatre fichiers frontend validés. |
| Fusion T3 | Avance rapide confirmée | Branche local/v0.0.45 vers le commit 5724eb7f12e4556327efa14f51f43b9caf404e25. |
| Push T3 | Confirmé par le principal | Dépôt distant fork, branches local/v0.0.45 et session141 poussées. |
| Livraison du commit portable | Confirmée par le principal | Cherry-pick cf0bd9903181327a793be06ccc23b2d402090abd poussé ; présence distante vérifiée avec ls-remote. |
| Commit préparatoire Bridget | Fusion et pushes confirmés | 7f4d65fb04261c09f420a2c907b6eeeecd1e9eaf ; avance rapide vers main ; pushes github main et session141 confirmés, ls-remote identique. |
| Sauvegarde de l'état | Copie et contrôle réussis | /Users/moi/.cache/t3-adoptions/spec141-20261007.llKRu1/state-before-install.sqlite, VACUUM depuis la source en lecture seule puis quick_check=ok, sortie 0. |
| Packaging | Réussi | --target zip, sortie 0, environ 89 secondes ; source 5724eb7f12e4 ; version 0.0.45-local.141. Archive conservée ci-dessous. |
| Installation sur disque | Installée à côté, vérifiée | /Applications/T3 Code (Local SPEC141).app ; SHA256 ASAR f4fd81f645b81c65bd8a0471022931c9e2e789df837d1b42c9ffa693151c90da. |
| Signature de la nouvelle application | Ad hoc après packaging ; vérification réussie | codesign --force --deep --sign - par le principal ; vérification stricte et profonde sortie 0 sur staging et application installée. |
| Contenu frontend installé | Comparaison et marqueurs PASS | Chunk _chat-DqlgvoJO.js extrait de l'application installée ; SHA256 53aa48551495f867385170d4e5a490aac82004ce5932bff1a291295533b32505 identique ; marqueurs data-bridget-member-toggle et Messages groupés présents. |
| T3 en cours | Relance interdite ; application active conservée | PID 85017 et 85080 inchangés, dates de démarrage identiques à 07:21. Archive active SHA256 dab141939a4171c3b4e7169fc68346b498ce179c414f9e2e8ee6acdb42f89ba1 ; contrôle codesign strict PASS. Aucun lancement de la nouvelle application ni redémarrage de l'application active revendiqué. |
| Santé de l'application en cours | Réponse HTTP valide après installation | Endpoint public de l'environnement dont l'identifiant commence par 3b1ba3d1 ; aucune relance effectuée. |
| Tests fraîchement rejoués | PASS | 294/294 sur la source livrée. |
| Activation de la présentation | Différée | L'ancienne application reste active ; la nouvelle n'a pas été ouverte. |
| Journal post-installation | Preuves regroupées dans ce document | Son commit de livraison est consultable dans l'historique Git de ce fichier ; aucun identifiant autoréférentiel n'est inscrit. |

<a id="extrait-5"></a>

## Extrait 5

Archive conservée : /Users/moi/.cache/t3-spec141-package.OaEnOF/T3-Code-0.0.45-local.141-arm64.zip
SHA256 : f38ae9b8322061d6d762e6b3fbe6dfd2b4d67dc330a167b230befa00beb53c9c

<a id="extrait-6"></a>

## Extrait 6

Quand l'utilisateur choisira de quitter /Applications/T3 Code (Local).app, il pourra ouvrir /Applications/T3 Code (Local SPEC141).app. Cette nouvelle application charge les fichiers141 installés. Une simple relance de /Applications/T3 Code (Local).app garde l'ancien paquet et ne charge pas141.
