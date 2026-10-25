# Archives techniques de l’adaptateur T3

Document d’origine : [specs/177-installateur/validation.md](../../../../../../specs/177-installateur/validation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Analyse de cohérence : spec, plan, tâches, code et documents comparés ; les deux
canaux et la version figée utilisent le même chemin de validation/remplacement.
Complexité linéaire dans la taille de l'archive/binaire, nombre de profils borné.
Minimalisme : aucun framework ni dépendance Python pour la release ; T3 ne
fournit pas d'installateur Rust réutilisable. Charge future limitée à un script
et ses tests. Le script est public et la redirection du site vise cette source
unique. Potentiel minimalisme retenu : 0 ligne supprimable sans perdre un
contrôle d'installation ou de conservation demandé.
