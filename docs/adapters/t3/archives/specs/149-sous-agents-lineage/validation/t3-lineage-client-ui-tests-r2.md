# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/t3-lineage-client-ui-tests-r2.md](../../../../../../../specs/149-sous-agents-lineage/validation/t3-lineage-client-ui-tests-r2.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Environnement identique au r1 : Node 24 `/Users/moi/.cache/t3-toolchains/148/`,
deps du WT via symlinks préexistants, aucun install, aucune duplication
`node_modules`. Runs de validation exécutée ; grande revue T040 reste future.

<a id="extrait-2"></a>

## Extrait 2

```
cd /Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage
./node_modules/.bin/vp test run \
  packages/client-runtime/src/state/orchestration.bridget149.test.ts \
  apps/web/src/components/BridgetTaskJournal149.test.tsx \
  apps/web/src/components/chat/ThreadRelationshipsControl.bridget149.test.tsx \
  apps/mobile/src/features/threads/BridgetTaskJournal149.test.tsx
```
