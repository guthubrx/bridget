# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/native-real-recipes-sonnet-r5.md](../../../../../../../specs/149-sous-agents-lineage/validation/native-real-recipes-sonnet-r5.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Exigence | Preuve r5 |
|---|---|
| G-P-01 refus fournisseur corrélé, pas de faux succès | R1 : `failed` + `provider_permission_denied`, task unique |
| Héritage sans grant | table grants = 0 partout ; R1, R2a, R2c |
| Même famille, règles préexistantes révisionnées | R3 (17 sources, project settings `df145aa2…`) |
| Confinement non représentable → refus nommé, pas de repli | R2b `provider_confinement_unavailable` |
| Politique Codex héritée exactement | R2c `workspaceWrite` / `never` ; enfant `gpt-6.1-sol` high |
| Observer : branche négative | R4a `permission_source_unavailable`, R4b `permission_attestation_unavailable` |
| Annulation native, descendants stoppés | R5 |
| dans un terminal | aucune variable T3, aucun processus T3 |
