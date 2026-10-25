# Archives techniques de l’adaptateur T3

Document d’origine : [docs/decisions/003-service-compagnon-orchestration.md](../../../../../decisions/003-service-compagnon-orchestration.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- **Réintégrer le service compagnon dans le daemon Bridget** : couplage fort, cycle de
  publication commun et transport alourdi.
- **Reprendre le superviseur historique** : pipeline rigide, cycle de
  worktrees et fournisseur imposés.
- **Partager `bridget.db`** : double autorité et migrations couplées.
- **Adopter DSH ou T3 Code comme interface canonique** : modèle de sessions et
  source d'état supplémentaires.
