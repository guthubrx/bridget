# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/t3-lint-format-haiku-r2.md](../../../../../../../specs/149-sous-agents-lineage/validation/t3-lint-format-haiku-r2.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Worktree T3 : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`
- Toolchain : Node v24.13.1 (`/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin`)
- Aucun code source, test, manifeste ou lock modifié.
- Aucun Git, Cargo, modèle réel, installation, redémarrage ou correction automatique (`vp fmt --write` non lancé).
- Aucun test unitaire ni `tsc` relancé.
- Aucun processus lancé par moi. Rien à nettoyer.

<a id="extrait-2"></a>

## Extrait 2

Logs privés :
- `/Users/moi/.cache/t3-lint149-PaLNaU/r2/lint-global-r2final.log` (et `.rc`)
- `/Users/moi/.cache/t3-lint149-PaLNaU/r2/fmt-check-149-r2final.log` (et `.rc`)
- `/Users/moi/.cache/t3-lint149-PaLNaU/r2/wt-diag-r2final.txt` et `root-diag-r2final.txt` (listes triées pour le delta)

<a id="extrait-3"></a>

## Extrait 3

- Liste utilisée : `/Users/moi/.cache/t3-lint149-PaLNaU/files149.txt` (48 fichiers). Elle n'a pas été modifiée.
- Comparaison avec `git status` : 49 chemins. Le seul écart est `apps/server/.vitest/json/output.json`.
- Ce fichier est une sortie de test Vitest. Ce n'est pas un fichier source 149. Il n'est donc pas ajouté à la liste.
