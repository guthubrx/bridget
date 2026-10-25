# Archives techniques de l’adaptateur T3

Document d’origine : [specs/140-bulles-compactes/contracts/presentation.md](../../../../../../../specs/140-bulles-compactes/contracts/presentation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

La valeur après le séparateur est une chaîne JSON, pas une interpolation brute. Le pont normalise contrôles et espaces, puis limite à200 caractères. Cette ligne est facultative. T3 ne la consomme que dans la position attendue et avec une valeur JSON valide ; une ligne invalide reste du texte, sans titre prétendu. Le brut conserve la ligne même après projection.
