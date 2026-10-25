# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/t3-lint-format-haiku-r1.md](../../../../../../../specs/149-sous-agents-lineage/validation/t3-lint-format-haiku-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Le lint échoue : 14 erreurs, dont 13 nouvelles introduites par 149. Une erreur existe déjà dans le checkout racine.
- Le lint échoue aussi sur 924 avertissements, contre 901 en racine (+23 liés à 149).
- Le format échoue : 37 fichiers sur 48 modifiés ou nouveaux par 149 ne respectent pas `vp fmt`. Deux échecs existent déjà en racine.
- Cause du blocage initial : le plugin de lint `oxlint-plugin-t3code` ne trouvait ni `@oxlint/plugins` ni `effect` dans le worktree. J'ai ajouté deux liens ciblés. Aucune dépendance n'a été installée ni copiée.
- Aucun fichier source, test, manifeste ou lock n'a été modifié. Aucun format ni fix automatique n'a été lancé.
- Aucune commande Git, Cargo, modèle réel, install ou redémarrage.

<a id="extrait-2"></a>

## Extrait 2

- Worktree : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`
- Racine (baseline) : `/Users/moi/11.Repositories/t3code-local`
- Toolchain Node : `/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin` (Node v24.13.1, vérifié par `verification.json`)
- Commande : `export PATH=/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin:$PATH`
- `vp` n'est pas dans le PATH. J'ai utilisé `./node_modules/.bin/vp` (lien vers la racine).
- Dossier des logs (hors dépôt) : `/Users/moi/.cache/t3-lint149-PaLNaU`

<a id="extrait-3"></a>

## Extrait 3

| Lien | Cible |
|---|---|
| `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/oxlint-plugin-t3code/node_modules/@oxlint/plugins` | `/Users/moi/11.Repositories/t3code-local/node_modules/.pnpm/@oxlint+plugins@1.68.0/node_modules/@oxlint/plugins` |
| `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/oxlint-plugin-t3code/node_modules/effect` | `/Users/moi/11.Repositories/t3code-local/node_modules/.pnpm/effect@4.0.1_patch_hash=a33cba07c41f32374c2aaa86ea4a84d3ab9a872a78c94b65e929b19f8361856f/node_modules/effect` |

<a id="extrait-4"></a>

## Extrait 4

Justification : la racine a les mêmes liens dans `oxlint-plugin-t3code/node_modules`. Le `package.json` du plugin demande `@oxlint/plugins` `^1.63.0`, et 1.68.0 le satisfait. Les règles du plugin importent seulement `@oxlint/plugins` et `effect`, vérifié par lecture des fichiers hors tests.

<a id="extrait-5"></a>

## Extrait 5

Liste des 48 fichiers 149 : `/Users/moi/.cache/t3-lint149-PaLNaU/files149.txt`. Elle reprend le `git status` lu au début (fichiers modifiés et non suivis, `apps/` et `packages/`).

<a id="extrait-6"></a>

## Extrait 6

Aucun test unitaire relancé. Les tsc baseline (16 server et 10 web) ne sont pas refaits. Les 104 PASS du rapport `t3-final-boundary-tests-sonnet-r1.md` ne sont pas revérifiés ici.
