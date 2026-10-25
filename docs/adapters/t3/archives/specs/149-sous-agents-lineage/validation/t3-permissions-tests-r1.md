# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/t3-permissions-tests-r1.md](../../../../../../../specs/149-sous-agents-lineage/validation/t3-permissions-tests-r1.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Date : 2026-10-10
Worktree : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage` (branche `session-149-sous-agents-lineage`)
Périmètre : T004–T008 permissions T3. Sources production **read-only** (aucune modification). Quatre nouveaux fichiers de tests + ce rapport.

<a id="extrait-2"></a>

## Extrait 2

```
cd /Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage

# Fichier 4 seul (dernier run)
./node_modules/.bin/vp test run apps/server/src/orchestration-v2/Adapters/BridgetPermissions149.test.ts
→ Test Files 1 passed (1) | Tests 12 passed (12) | Duration ~2s

# Consolidation
./node_modules/.bin/vp test run \
  packages/contracts/src/bridgetPermissions149.test.ts \
  packages/provider-core/src/server/mcpSession149.test.ts \
  apps/server/src/mcp/OrchestratorMcpService.bridgetPermissions149.test.ts \
  apps/server/src/orchestration-v2/Adapters/BridgetPermissions149.test.ts
→ Test Files 1 failed | 3 passed (4) | Tests 7 failed | 58 passed (65) | Duration ~4.3s

# Typechecks
cd packages/contracts && ../../node_modules/.bin/tsc --noEmit
→ 2 erreurs production attendues (bridgetPermissions.ts:87/100, TS2322). 0 erreur dans mes fichiers.
cd packages/provider-core && ../../node_modules/.bin/tsc --noEmit
→ 0 erreur propre du package (les 2 TS2322 contracts remontent via le symlink deps).
cd apps/server && ../../node_modules/.bin/tsc --noEmit
→ 366 erreurs au total. 0 dans mes 2 fichiers après corrections de mes seuls harnais.
   Les 366 viennent du lot BridgetLineage/BridgetReader (+ leurs tests 149) d'un autre agent — hors ownership, non audités.

# Régression (fichiers existants)
./node_modules/.bin/vp test run packages/contracts/src/bridget.test.ts \
  packages/provider-core/src/server/mcpSession.test.ts \
  apps/server/src/mcp/BridgetSession.test.ts \
  apps/server/src/orchestration-v2/Adapters/ClaudeMcpPreparation.test.ts
→ Test Files 2 failed | 2 passed (4) | Tests 3 failed | 55 passed (58)
```

<a id="extrait-3"></a>

## Extrait 3

1. `initialize` (id 1) → résultat → `initialized`. NB : `replay.ts:196-210` masque `clientInfo.version` — l'attendu n'a pas besoin de `packageJson`.
2. `config/read` (id 2, `{cwd, includeLayers: false}`) — déclenché par le credential monté via `prepareCodexBridgetMcp` (`CodexAdapterV2.ts:1818-1826`). Résultat attendu : `{config: {mcp_servers: {}}, origins: {}}` (schéma `CodexMcp.ts:6-12`). Un `mcp_servers.bridget` utilisateur court-circuite (jamais remplacé).
3. `thread/start` (id 3) : la config gagne la clé **plate** `"mcp_servers.t3-code"` `{url, http_headers.Authorization}` construite depuis le credential (`CodexAdapterV2.ts:1341-1356`). Le normalizer (`replay.ts:262-279`) ne strippé que la clé exacte `mcp_servers` — les clés à points restent dans la comparaison. `cwd`/`model` sont strippés.
4. `turn/start` (id 4) : avec le mount t3-code, l'adapter ajoute `additionalContext` (`t3_code_orchestration`, `t3_code_runtime` ; `t3_code_tools` seulement si des tools sont disponibles) et `collaborationMode` (`default` + `developer_instructions`). Mes attendus appellent les exports de production (`buildCodexAdditionalContext`, `buildCodexDeveloperInstructions`) — aucune copie figée de prose.

<a id="extrait-4"></a>

## Extrait 4

Environnement fixture Codex `{}` → `bridgetMcpServer` retourne `namespace_unavailable` → pas de mount stdio bridget (un seul mount t3-code HTTP).
