# Archives techniques de l’adaptateur T3

Document d’origine : [specs/125-fin-de-tour-rapide/spec.md](../../../../../../specs/125-fin-de-tour-rapide/spec.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

## Problème observé
Le 25/09 à 07:49:05.9Z, GLM4 a clos un tour ; 0,6 s plus tard un message du coordinateur en a
lancé un autre. T3 n'expose que le dernier tour et le pont lit toutes les ~3 s : il n'a jamais vu
le premier clos comme dernier. Il l'a signalé « continuité non garantie » (lacune) au lieu de
« turn_ended », alors qu'il avait relevé son origine pendant qu'il tournait.
