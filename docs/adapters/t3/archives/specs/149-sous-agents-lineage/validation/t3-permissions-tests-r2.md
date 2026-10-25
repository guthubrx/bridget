# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/t3-permissions-tests-r2.md](../../../../../../../specs/149-sous-agents-lineage/validation/t3-permissions-tests-r2.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-10
Worktree : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage` (branche `session-149-sous-agents-lineage`, HEAD `33f6d04e11` + changements non commités de Sol)
Baseline propre : checkout principal `/Users/moi/11.Repositories/t3code-local` à `33f6d04e11`, statut git propre (aucun changement non commité).
Périmètre r2 : reprise après fixes Sol (union `permission_attestation_unavailable`, wrappers `decodeConstructor`, durcissement C01/C02). Sources production **read-only**. Preuves baseline par exécution réelle sur le checkout propre — pas sur l'argument « tests non importés ».

<a id="extrait-2"></a>

## Extrait 2

- `prepares Bridget before publishing a V2 query and preserves T3 MCP` → **ROUGE à la baseline**, `AssertionError: expected 1 to equal 2` à `ClaudeMcpPreparation.test.ts:74:12` — signature et ligne **identiques** au worktree.
- `closes a failed MCP candidate without publishing it` → **ROUGE à la baseline**, même dump d'objet session qu'au worktree.

<a id="extrait-3"></a>

## Extrait 3

Artefact complet : `validation/t3-permissions-types-r2.log` (97 `error TS` + 157 `suggestion TS`, apps/server).

<a id="extrait-4"></a>

## Extrait 4

```
cd /Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage
./node_modules/.bin/vp test run apps/server/src/orchestration-v2/Adapters/BridgetPermissions149.test.ts
→ Test Files 1 passed (1) | Tests 14 passed (14)
./node_modules/.bin/vp test run \
  packages/contracts/src/bridgetPermissions149.test.ts \
  packages/provider-core/src/server/mcpSession149.test.ts \
  apps/server/src/mcp/OrchestratorMcpService.bridgetPermissions149.test.ts \
  apps/server/src/orchestration-v2/Adapters/BridgetPermissions149.test.ts
→ Test Files 4 passed (4) | Tests 67 passed (67)
./node_modules/.bin/vp test run packages/contracts/src/bridget.test.ts \
  packages/provider-core/src/server/mcpSession.test.ts \
  apps/server/src/mcp/BridgetSession.test.ts \
  apps/server/src/orchestration-v2/Adapters/ClaudeMcpPreparation.test.ts
→ Test Files 2 failed | 2 passed (4) | Tests 3 failed | 55 passed (58)
cd packages/contracts && ../../node_modules/.bin/tsc --noEmit   → EXIT=0
cd packages/provider-core && ../../node_modules/.bin/tsc --noEmit → EXIT=0
cd apps/server && ../../node_modules/.bin/tsc --noEmit          → EXIT=1 (97, cf. artefact)

cd /Users/moi/11.Repositories/t3code-local   # baseline propre 33f6d04
./node_modules/.bin/vp test run apps/server/src/orchestration-v2/Adapters/ClaudeMcpPreparation.test.ts \
  apps/server/src/mcp/BridgetSession.test.ts --exclude "**/.worktrees/**"
→ Test Files 1 failed | 1 passed (2) | Tests 2 failed | 7 passed (9)
  (ClaudeMcpPreparation ×2 rouges, BridgetSession 5/5 verts)
cd apps/server && ../../node_modules/.bin/tsc --noEmit → EXIT=1 (16, 0 dans scripts/lib)
```
