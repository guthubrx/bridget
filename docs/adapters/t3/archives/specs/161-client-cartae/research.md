# Archives techniques de l’adaptateur T3

Document d’origine : [specs/161-client-cartae/research.md](../../../../../../specs/161-client-cartae/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Cache offline complet, temps réel de compilation, résultat des recettes et disponibilité du préfixe neuf sont à constater pendant l'implémentation. Les conduites d'échec sont spécifiées ; aucune décision architecturale suspendue. Les paramètres des serveurs de décision par machine et du Lineage T3 appartiennent à162/163 et ne bloquent pas161.

<a id="extrait-2"></a>

## Extrait 2

Lecture distante fraîche ROOT : status RC0, maître e6812e3e7b2d ; le vieux client096 émet un avertissement trompeur qui accuse le daemon. Ne pas suivre sa suggestion de relance. PID maître20937, pont21097, T3 60116/60228 inchangés au relevé. Python3.12.3 et /usr/bin/lsof disponibles sur Cartae. L'audit de réemploi séparé recherche la réutilisation de SOURCE-MANIFEST.sha256 de package089 et du nettoyage existant build.py. Ces constats guident la future implémentation ; ce document ne remplace pas reuse-audit.md ni son gate.
