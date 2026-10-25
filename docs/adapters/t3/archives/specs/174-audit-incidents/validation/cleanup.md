# Archives techniques de l’adaptateur T3

Document d’origine : [specs/174-audit-incidents/validation/cleanup.md](../../../../../../../specs/174-audit-incidents/validation/cleanup.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

T3 reference: GitVcsDriverCore.removeWorktree runs an explicit removal, without force by default, and reports Git failure. Bridget reuses Git removal but needs additional guards because maintenance is automatic. Git alone does not preserve ignored output.
