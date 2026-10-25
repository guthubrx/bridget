# Archives techniques de l’adaptateur T3

Document d’origine : [specs/153-rondes-contexte/quickstart.md](../../../../../../specs/153-rondes-contexte/quickstart.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

La livraison finale est distincte de cette recette privée. Le principal vérifie
Git, disque et runtime, active de façon coordonnée sans interrompre les autres
fils T3, puis nettoie le worktree153. Aucun succès actif n'est déduit du seul build.
