# Archives techniques de l’adaptateur T3

Document d’origine : [specs/100-observation-partage/contracts/observation.md](../../../../../../../specs/100-observation-partage/contracts/observation.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Couverture : fins de tour corrélées ACP/Claude stream-json/Codex app-server ;
permissions effectivement journalisées (ACP/Codex), éventuellement déjà traitées.
Pas de fin de tour déduite d'un état idle natif/T3, du silence ou d'une déconnexion.
Écritures confirmées des outils reconnus Claude/ACP et items Codex fileChange
completed ; aucun shell arbitraire, lecture seule ou écriture échouée.
Les tours liés aux messages `bridget-observation:` ne produisent pas de faits.
Cache de collisions : 4096 chemins, normalisation lexicale sans suivre les
symlinks, même hôte et deux auteurs dans 30 s. Pas de fédération d'événements.
