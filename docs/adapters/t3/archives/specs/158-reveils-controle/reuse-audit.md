# Archives techniques de l’adaptateur T3

Document d’origine : [specs/158-reveils-controle/reuse-audit.md](../../../../../../specs/158-reveils-controle/reuse-audit.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| 087 | plan de contrôle, `ControlStateV1` | base de la CLI |
| 148, 149, 150 | contrat de délégation, `human_cancel`, ordre des refus | `delegations_halted` dans l'ordre |
| 151 à 157 | nouveaux outils MCP, rondes (153), posture | classés par le test exhaustif ; plafond suit la vie de l'enfant |
| 145-147 | vue Lineage T3 | aucun changement de forme |
