# Archives techniques de l’adaptateur T3

Document d’origine : [specs/161-client-cartae/plan.md](../../../../../../specs/161-client-cartae/plan.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Étendre l'outil client-only existant pour borner les ressources et contrôler les octets transférés avant build. Exécuter cet outil 161 depuis le worktree, avec une source Git isolée et propre exactement au commit 160. Installer dans un préfixe neuf, vérifier le client par le tunnel existant, puis basculer la commande courante de façon atomique. Le maître et T3 restent actifs sans intervention.
