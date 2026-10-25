# Archives techniques de l’adaptateur T3

Document d’origine : [specs/102-fils-inter-agents/research.md](../../../../../../specs/102-fils-inter-agents/research.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Décision : extension facultative thread_notice, traitée par wrappers/T3 avant
le chemin de réponse automatique. Alternative rejetée : préfixe texte ou DM normal.
Impact : tests adaptateurs nécessaires mais impossibilité de forger une alerte
en citant son texte ; pas de réponses involontaires hors fil.
