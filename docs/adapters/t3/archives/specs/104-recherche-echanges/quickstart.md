# Archives techniques de l’adaptateur T3

Document d’origine : [specs/104-recherche-echanges/quickstart.md](../../../../../../specs/104-recherche-echanges/quickstart.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Depuis le worktree, `umask 077`, `BRIDGET_HOME` pointant sur un répertoire vide (sinon l'identité T3 du poste
est héritée par les tests CLI) :

<a id="extrait-2"></a>

## Extrait 2

Commits et fusion : selon l'autorisation en vigueur pour cette session (voir `implementation.md`).
Prochaine action technique après fusion : reconstruire le daemon installé (`t3-local-build` ou script 106) et
relancer `com.bridget.daemon` sur autorisation, puis exécuter la recette humaine ci-dessus.
