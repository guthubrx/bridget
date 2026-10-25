# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/036-observation-non-bloquante.md](../../../../../decisions/036-observation-non-bloquante.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Les identifiants internes de notifications portent `bridget-observation:`.
Le journal ignore les observations issues de ces tours corrélés, pour empêcher
les boucles entre abonnés. Une fin de tour non corrélée (idle natif/T3 seul)
n'est pas un substitut fiable. Cette couverture limitée est explicite, pas
une prétention d'observation universelle. Les notifications ne clôturent jamais
les demandes de réponse et n'imposent aucune action aux destinataires.

<a id="extrait-2"></a>

## Extrait 2

Pas de migration DB, dépendance externe, verrou de fichier, formulaire, mandat,
moteur de règles général ni couche le service compagnon/T3 obligatoire.
