# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/t3-permissions-fixtures-final-r3.md](../../../../../../../specs/149-sous-agents-lineage/validation/t3-permissions-fixtures-final-r3.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-10
Worktree : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage` (branche `session-149-sous-agents-lineage`, HEAD `33f6d04e11` + changements non commités)
Base de comparaison : checkout principal `/Users/moi/11.Repositories/t3code-local` à `33f6d04e11` (voir r2 pour les preuves baseline).
Périmètre r3 : clôture des 2 fixtures rouges (BridgetSession, ProviderTurnControlService), vérification du fix Sol `ClaudeAdapterV2.ts:901`, montage des deps manquantes du worktree. Sources production **read-only** — aucun fichier production touché.

<a id="extrait-2"></a>

## Extrait 2

```
cd /Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage
./node_modules/.bin/vp test run apps/server/src/mcp/BridgetSession.test.ts
→ Test Files 1 passed (1) | Tests 6 passed (6)
./node_modules/.bin/vp test run apps/server/src/orchestration-v2/ProviderTurnControlService.test.ts
→ Test Files 1 passed (1) | Tests 1 passed (1)
./node_modules/.bin/vp test run \
  packages/contracts/src/bridgetPermissions149.test.ts \
  packages/provider-core/src/server/mcpSession149.test.ts \
  apps/server/src/mcp/OrchestratorMcpService.bridgetPermissions149.test.ts \
  apps/server/src/orchestration-v2/Adapters/BridgetPermissions149.test.ts \
  apps/server/src/mcp/BridgetSession.test.ts \
  apps/server/src/orchestration-v2/ProviderTurnControlService.test.ts
→ Test Files 6 passed (6) | Tests 74 passed (74)
cd packages/contracts    && ../../node_modules/.bin/tsc --noEmit → EXIT=0
cd packages/provider-core && ../../node_modules/.bin/tsc --noEmit → EXIT=0
cd apps/server           && ../../node_modules/.bin/tsc --noEmit → EXIT=1 (16, artefact t3-permissions-types-r3.log)
```
