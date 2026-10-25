# Archives techniques de l’adaptateur T3

Document d’origine : [specs/174-audit-incidents/validation/collect-index.md](../../../../../../../specs/174-audit-incidents/validation/collect-index.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- Copie lue : `/home/moi/.cache/bridget-maintenance174/T3-ProviderSessionReaper.ts`.
- T3 parcourt une liste de sessions avec un seuil d'inactivité. Il fait une requête par session.
- Il ne croise pas deux collections et ne supprime aucune preuve.
- T3 ne possède pas le store distant de Bridget. Il ne sert donc pas de référence de correction pour `collect`.
