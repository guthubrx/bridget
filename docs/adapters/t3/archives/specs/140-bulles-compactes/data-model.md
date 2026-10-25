# Archives techniques de l’adaptateur T3

Document d’origine : [specs/140-bulles-compactes/data-model.md](../../../../../../specs/140-bulles-compactes/data-model.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Le texte, l'identifiant, le fil T3, les pièces jointes et les actions existants restent les données de référence. Aucun changement SQL ou de stockage T3. La copie conserve le texte source, pas le résumé.

<a id="extrait-2"></a>

## Extrait 2

État local ouvert/replié, par identité composée fil T3 + message. Valeur initiale : replié. Les détails techniques peuvent être ouverts seulement depuis la bulle. Changer de fil ou recycler une ligne ne transmet pas l'état à un autre message. L'état n'est pas une prise en charge ou un ACK Bridget.
