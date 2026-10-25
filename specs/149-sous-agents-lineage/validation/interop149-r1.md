# T036 - Interop réseau native 149 (adaptateur de bureau réel <-> daemon Bridget 149 réel)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent), ronde natif/réseau r1.
Aucun code de production modifié. Aucun commit. Aucune case cochée dans `tasks.md`.

## Verdict

**APPROVE pour ce qui a été exécuté, sous réserve du tableau des limites. T036 reste à cocher par le principal.**

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r1.md#extrait-1).

## Couches : réel ou simulé

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r1.md#extrait-2).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r1.md#extrait-3).

## Commandes exactes

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r1.md#extrait-4).

Résultats bruts : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results/interop149.json`, `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/native-network-recipes/results/server149.json`, sorties `*.stdout.txt` du même dossier.
Fixtures : `/Users/moi/.cache/bridget149-native-interop.*` (0700, APFS, supprimées après la ronde) et projet git privé sous `/tmp/b149n.*`.

## Partie 1 - hôte MCP adaptateur de bureau réel et daemon natif réel

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r1.md#extrait-5).

### Compteurs observés (dernier passage, partie 1)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r1.md#extrait-6).

## Partie 2 - serveur adaptateur de bureau complet réel et daemon natif réel

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r1.md#extrait-7).

### Compteurs observés (dernier passage, partie 2)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r1.md#extrait-8).

## Constats

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r1.md#extrait-9).

## Ce qui n'est PAS prouvé ici

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r1.md#extrait-10).

## Empreintes

Binaire : sha256 `823e8a5fab8abe38fcb439d8fda98d929622dae3751ff8f629d306744379328b` (égal au reçu `native149-debug-receipt.json` : True).

Sources Rust : empreinte actuelle `f54a66ee8bcdc305…`, empreinte du reçu `e6f45086b2397460…`, égales : **False**.
Sources de production plus récentes que le binaire : `crates/bridget-daemon/src/daemon.rs`, `crates/bridget-daemon/src/daemon/native_delegation.rs`, `crates/bridget-daemon/src/wrapper.rs`.
Conséquence : les résultats valent pour le binaire `823e8a5f`, pas pour l'état actuel de ces trois fichiers.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/interop149-r1.md#extrait-11).

| Fichier | sha256 |
|---|---|
| `packages/provider-core/src/server/mcpSession.ts` | `2e95140365bef3f8…` |
| `apps/server/src/mcp/OrchestratorMcpService.ts` | `e4d6a2db680f9720…` |
| `apps/server/src/mcp/McpSessionRegistry.ts` | `6e1282d6fcde6846…` |
| `apps/server/src/mcp/McpHttpServer.ts` | `0bf639fbb86bb23f…` |
| `apps/server/src/bridget/BridgetReader.ts` | `99acc8a7c38c2477…` |
| `apps/server/src/bridget/BridgetLineage.ts` | `ddb6be37c21e1582…` |
| `apps/server/src/ws.ts` | `c9cce8ceef8fcb5a…` |
| `apps/server/src/server.ts` | `04bcd1aeae7c70f3…` |
| `packages/contracts/src/bridgetLineage.ts` | `cd216feb52261c28…` |
| `packages/contracts/src/bridgetPermissions.ts` | `df03fd3fb71dc3e2…` |

Outils de recette :

| Fichier | sha256 |
|---|---|
| `codex149.py` | `7de49a1b3edc6a73…` |
| `common149.ts` | `e273956be66a6e1d…` |
| `fingerprints.py` | `c5dec1c463b8f651…` |
| `fx.mjs` | `6f2e9f3c2467ce6a…` |
| `interop149.ts` | `f1f7cfaa59963617…` |
| `provider149.py` | `cdcefa2b742738fd…` |
| `recovery149.ts` | `12c3a16109eb5b49…` |
| `server149.ts` | `c120df6f1d291336…` |
| `t3host.ts` | `10b96a42ed8a9750…` |
| `t3server.ts` | `6c8ffc8b5ddc7d24…` |
