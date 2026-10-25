# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/t3-final-boundary-tests-sonnet-r1.md](../../../../../../../specs/149-sous-agents-lineage/validation/t3-final-boundary-tests-sonnet-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/Adapters/BridgetPermissions149.test.ts` (G3)
- `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/Orchestrator.bridget149.test.ts` (G5)
- Ce rapport.

<a id="extrait-2"></a>

## Extrait 2

- `vp lint` n'a pas pu tourner : le plugin oxlint `@oxlint/plugins` manque dans ce worktree. Je n'ai rien installé.
- `vp fmt --check` signale les deux fichiers. Je n'ai pas formaté, pour ne pas réécrire le code d'autres propriétaires. Le formatage précédent des fichiers n'était pas vérifié non plus.
- L'ingestor n'est pas dans le harnais : provider_native est semé par événements de même forme, pas par un vrai appel d'adaptateur.
- La cellule start de provider_native vérifie le refus normal, pas un démarrage (un fil provider-native ne prend aucun message).
- Pas de mutation sur le code de production (écriture interdite). Les mutations G3 passent par le seam `mcpSession`.
- G1, G2, G4, G6, G7, G8 : hors périmètre, non traités. Pas de test global T3, pas de 1796 natif, pas de reconstruction.
- Observation, pas un bug : en recovery, la protection d'un enregistrement bridget_native tient au `runId` nul que `bridget.lineage.sync` écrit toujours. Un enregistrement non canonique avec un `runId` serait traité comme du travail fournisseur. Le producteur réel ne le produit pas.
