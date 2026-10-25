# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/archive-r4-recettes/recipes/README.md](../../../../../../../../../specs/149-sous-agents-lineage/validation/archive-r4-recettes/recipes/README.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Oracle | Où il s'observe |
|---|---|
| Politique figée + révisions | `recipe_env.sh` (expected.json, sources-snapshot.json) ; le daemon revalide au spawn |
| Corrélation unique request_id | `verify_oracles.py` (1 ligne par request_id) |
| Refus fournisseur G-P-01 | payload `failed` + `provider_permission_denied` |
| Observer PTY G-P-08 | branche (c) : blocage nommé sans ACK ; artefacts `np-*.sock`/`no-*.json` supprimés en sortie |
| Composition lanceur+profil | hash gclaude == constante + `CLAUDE_CONFIG_DIR` exporté (capturé de l'environnement source) |
| Cleanup lifecycle | aucun processus lié au fixture (`ps`), état `cancelled` persistant |
| Absence T3 | variables `BRIDGET_T3_MCP_*` absentes, voie `NativeDelegation` seule |
