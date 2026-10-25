# Archives techniques de l’adaptateur T3

Document d’origine : [specs/176-observation-bornee/validation/automatic-build-maintenance.md](../../../../../../../specs/176-observation-bornee/validation/automatic-build-maintenance.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Read T3's package.json clean command and stageResourceMonitor in
scripts/build-desktop-artifact.ts at ef86c2bbc3388be8fc593db23a323e3d80b58509.
T3 reuses its monitor target and provides an explicit broad clean command.
Those paths do not implement Bridget's per-repository intermediate-cache budget.
Reuse of the existing target is applicable; broad deletion is not reused because
Bridget also has running binaries and independent remote work.
