# Archives techniques de l’adaptateur T3

Document d’origine : [specs/146-bridget-panel-lisible/research.md](../../../../../../specs/146-bridget-panel-lisible/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Alternative rejetée : trier seulement les pages déjà chargées dans T3. Cela laisserait le vrai fil le plus actif hors de la première page et montrerait les vieux messages avant les derniers.

<a id="extrait-2"></a>

## Extrait 2

Décision : conserver typographie, teintes neutres, icône monochrome et primitives du panneau T3.

<a id="extrait-3"></a>

## Extrait 3

Les contrats T3145 utilisent des unions strictes pour `list`, `show`, `history` et leurs réponses. Source : `/Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/packages/contracts/src/bridget.ts:10`, résultats vers90. Le lecteur existant construit les arguments CLI séparément et conserve les budgets6 secondes,256 KiB de processus et128 KiB stdout. Source : `/Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/server/src/bridget/BridgetReader.ts:111`. Les chemins ont été contrôlés dans l'inventaire146.

<a id="extrait-4"></a>

## Extrait 4

Le comportement de dépliage suit les principes de la [W3C WAI, motif Disclosure](https://www.w3.org/WAI/ARIA/apg/patterns/disclosure/) : contrôle activable au clavier et état ouvert ou fermé accessible. Ce motif ne dicte ni la hauteur de l'aperçu ni son style T3.
