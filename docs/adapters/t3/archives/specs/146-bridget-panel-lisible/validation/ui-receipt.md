# Archives techniques de l’adaptateur T3

Document d’origine : [specs/146-bridget-panel-lisible/validation/ui-receipt.md](../../../../../../../specs/146-bridget-panel-lisible/validation/ui-receipt.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Racine des commandes : `/Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web`.

<a id="extrait-2"></a>

## Extrait 2

RED à 06:25:45 :18 PASS/2 FAIL sur20. L'UI héritée145 demandait `list` plutôt que `list_recent` et n'affichait pas les éléments de date courte et détails attendus. Preuve : `/Users/moi/.cache/t3-spec146-preview.650MHJ/ui-red-captured-transcript.txt`, transcription verbatim de la sortie PTY32976/chunkfbb74b, explicitement pas une nouvelle exécution.

<a id="extrait-3"></a>

## Extrait 3

- `/Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.tsx` : `b77cb990970b7f521084a444642bec4179013c38b1409a243d0bbf384cbb92d8`.
- `/Users/moi/11.Repositories/t3code-local/.worktrees/146-bridget-panel-lisible/apps/web/src/components/BridgetPanel.test.tsx` : `8c6a3cde1eed7d8a5477d35fc9de8385036065e7e0b81ad69dc4e4846ab06f98`.

<a id="extrait-4"></a>

## Extrait 4

Le composant local BridgetMessage réunit le corps exact, le dépliage, la mesure du débordement par ResizeObserver, la copie native et les détails HTML. Aucun framework, module générique ou dépendance n'est créé. Button, Tooltip, ScrollArea, Input et la copie restent natifs T3. Le corps original unique sert à l'affichage, la recherche et la copie.
