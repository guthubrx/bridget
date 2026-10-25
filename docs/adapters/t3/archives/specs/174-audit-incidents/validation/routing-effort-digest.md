# Archives techniques de l’adaptateur T3

Document d’origine : [specs/174-audit-incidents/validation/routing-effort-digest.md](../../../../../../../specs/174-audit-incidents/validation/routing-effort-digest.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Dans T3, `ProviderCommandReactor::ensureSessionForThread` vérifie l'identité
de l'instance. `CodexAdapter::sendTurn` transmet séparément l'option
`reasoningEffort` pour cette instance. Ce mécanisme confirme la distinction
entre identité du fournisseur et option du tour. T3 n'a pas la sonde distante
de Bridget ; il ne justifie aucune suppression de nos preuves de droits.
