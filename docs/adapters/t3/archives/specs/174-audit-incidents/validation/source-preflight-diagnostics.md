# Archives techniques de l’adaptateur T3

Document d’origine : [specs/174-audit-incidents/validation/source-preflight-diagnostics.md](../../../../../../../specs/174-audit-incidents/validation/source-preflight-diagnostics.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Read `GitVcsDriverCore.ts`: `gitCommandContext`, `isMissingGitCwdError`, command spawning with `cwd`, and `branchExists`. T3 distinguishes a missing working directory from an existing branch. Bridget additionally needs to explain source versus destination hosts. No automatic path remapping was added.

<a id="extrait-2"></a>

## Extrait 2

Only the master restarted. The bridge, T3 and remote worker were preserved. The worker remains on its previous optimized build because no worker production code changed. Private activation and test receipts are retained outside Git. The project owner was notified; message submission does not prove that the recipient has read it.
