# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/t3-lineage-server-tests-r2.md](../../../../../../../specs/149-sous-agents-lineage/validation/t3-lineage-server-tests-r2.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Session : 149-sous-agents-lineage. Lot : L (serveur), reprise r2.
Auteur : sous-agent test du lot serveur. Périmètre : 5 fichiers test + ce rapport + le log types.
Date : 2026-10-10. Worktree : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`.

<a id="extrait-2"></a>

## Extrait 2

Runner : `./node_modules/.bin/vp test run <fichiers>` depuis la racine du worktree.
Node v24.13.1 (toolchain `/Users/moi/.cache/t3-toolchains/148`, PATH en tête).
Sortie vérifiée par compteurs stdout (`Test Files` / `Tests`), pas par code de sortie.

<a id="extrait-3"></a>

## Extrait 3

Note de traçabilité : le log `t3-lineage-server-types-r2.log` a été réécrit à 12:43 par une
session concurrente (permissions r2 fait aussi un typecheck serveur). Contenu repris : même
compte de 79, mêmes familles. La liste compacte ci-dessus fait foi.
