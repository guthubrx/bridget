# Archives techniques de l’adaptateur T3

Document d’origine : [specs/138-priorite-projet/evidence/loop.md](../../../../../../../specs/138-priorite-projet/evidence/loop.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le transport passe les options structurées depuis le lot figé et retire toute
identité T3 héritée. Seules les lignes AVERTISSEMENT PROJET avec JSON de code
cross_project/project_unknown sont rendues au caller. Le stderr libre et les
champs body supplémentaires restent absents du rendu. Le corps livré est exact.
