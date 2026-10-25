# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/162-execution-distante-sous-autorite-unique.md](../../../../../decisions/162-execution-distante-sous-autorite-unique.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Exécuter une tâche sur une machine explicitement choisie, sans déplacer l’autorité métier ni perdre les droits du parent. Suivre la tâche depuis Bridget et le Lineage T3. Une coupure ne doit produire ni doublon ni résultat perdu après sa persistance technique.

<a id="extrait-2"></a>

## Extrait 2

Lineage conserve v1 par défaut. Une projection v2 est demandée explicitement sans modifier l’authentification humaine. T3 reste un lecteur ; ses fichiers sont livrés sans relancer le runtime actuel. Le worker ne dépend pas de T3.
