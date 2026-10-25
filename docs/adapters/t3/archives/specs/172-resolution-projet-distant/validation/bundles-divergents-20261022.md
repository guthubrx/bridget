# Archives techniques de l’adaptateur T3

Document d’origine : [specs/172-resolution-projet-distant/validation/bundles-divergents-20261022.md](../../../../../../../specs/172-resolution-projet-distant/validation/bundles-divergents-20261022.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le worker Cartae reste au build 172-76b682d6, PID 2219043. Le maître transmet le helper à chaque appel SSH ; aucun remplacement du worker n'est requis. Pont PID 898 et T3 PID 28528/28591 préservés, dates de naissance et ASAR inchangés. L'activation a contrôlé l'absence de commande Bridget distante non terminale et de wrapper actif. Elle n'affirme rien sur les calculs T3 ou SSH indépendants.
