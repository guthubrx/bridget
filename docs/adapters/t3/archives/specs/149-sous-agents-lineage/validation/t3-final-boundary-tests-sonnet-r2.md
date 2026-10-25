# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/t3-final-boundary-tests-sonnet-r2.md](../../../../../../../specs/149-sous-agents-lineage/validation/t3-final-boundary-tests-sonnet-r2.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Worktree : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`
- Liste source : `/Users/moi/.cache/t3-lint149-PaLNaU/files149.txt` (48 fichiers). Seuls les 16 fichiers `.test.ts` et `.test.tsx` sont à moi.
- Liste exacte des 16 : `/Users/moi/.cache/t3-lint149-PaLNaU/r2/tests149-list.txt`
- Node 24.13.1 : `/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin`
- Logs : `/Users/moi/.cache/t3-lint149-PaLNaU/r2/`
- Aucun Cargo, Git, install, redémarrage, modèle réel ni script de configuration.

<a id="extrait-2"></a>

## Extrait 2

- Aucun fichier de production modifié par moi.
- Hash de la liste production (32 fichiers non test de files149, SHA-256 de la liste des SHA-256) : avant mes corrections et après tous les runs : `4409ef3ff15ec1fa46e966e05eb83654c145603d77339e3b8c17059b5697ae06`. **Identique**, donc aucun effet de la production de Sol ni de Solt3149 pendant ce round.
- Détail : `/Users/moi/.cache/t3-lint149-PaLNaU/r2/prod-hash-before.txt` et `prod-hash-after.txt`.
- Limite : les hash prouvent l'absence de changement pendant ma fenêtre. Si Sol modifie la production après, le principal doit relancer G3/G5 (adaptateur Codex et Orchestrator).

<a id="extrait-3"></a>

## Extrait 3

- Les 3 erreurs de lint de production listées par Haiku (ClaudeAdapterV2.ts, BridgetTaskJournal.tsx) et les avertissements de production : propriété de Sol / Solt3149.
- Lint et format des fichiers de production ne sont pas évalués ici.
- Les 16 erreurs tsc serveur et 10 web : base connue.
- Les 2 liens `@oxlint/plugins` et `effect` dans `oxlint-plugin-t3code/node_modules` du worktree restent en place (posés par Haiku).

<a id="extrait-4"></a>

## Extrait 4

- Pas de lint global ni de test global T3.
- `tsc` web non relancé (seuls des imports inutilisés y ont été retirés dans 1 fichier ; ce fichier passe en test et en lint).
