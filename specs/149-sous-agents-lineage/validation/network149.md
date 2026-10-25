# Recette réseau 149 - RPC réels sur WebSocket, scopes et refus (T036 partiel)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent), r2. Aucun edit de production, aucun commit, aucune case cochée.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/network149.md#extrait-1).

## Compteurs

| Groupe | PASS | FAIL | SKIP |
|---|---|---|---|
| Matrice `matrix149.mjs all` (scopes S1-S10, entrées I1-I13, roots R1-R12, cancel C1-C3) | 38 | 0 | 0 |
| 200 lectures répétées (100 `show` + 100 `list`, jeton read) | 200 | 0 | 0 |
| Pagination 134 tâches (P1 sans mutation, P2 une mutation entre pages, P3 mutations épuisant 3 tentatives) | 3 | 0 | 0 |
| Commandes mutantes sur fil Bridget virtuel (10 refus, `thread.visit` accepté) | 11 | 0 | 0 |
| MCP privé authentifié, scopes read/operate MCP, rotation/révocation, identité étrangère | 0 | 0 | 1 (bloqué) |
| `orchestration.launchThread` sur fil virtuel | 0 | 0 | 1 (non lancé volontairement) |

Sortie brute de la matrice : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/network149-matrix-run.txt`.

## Couches réelles et simulées

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/network149.md#extrait-2).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/network149.md#extrait-3).

## Commandes exactes

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/network149.md#extrait-4).

Répertoire des outils : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui-recipes/`. Le lien `ui-recipes/node_modules` pointe vers `apps/server/node_modules` du WT (symlink, aucune copie).

## Constats prouvés

**Scopes (S1-S10).** Le jeton `orchestration:read` lit `list`, `show`, `journal`, `watch` (ready seq 0) et le flux `journal`. Il est refusé sur `bridget.lineage.cancel` (`EnvironmentAuthorizationError`, scope manquant). Le jeton `settings:write` seul est refusé sur `read` et `cancel`. Sans jeton ou avec un jeton invalide, le ticket WS rend HTTP 401 (`missing_credential` / `invalid_credential`). Le jeton operate annule une tâche déjà terminale sans inventer `cancelled` : l'état courant revient (`result_available`, `failed`).

**Autorité inconnue ou en trop (I1-I13).** Treize charges non conformes, envoyées en trames brutes, sont refusées par le serveur : `projectRoot`, `workspaceRoot`, `rootOwnerAgentId` (extra), `force` (extra sur cancel), `afterSeq` sur watch (extra), action inconnue, `taskId` non UUID ou UUID en majuscules, `limit` 16385 et 101, `afterSeq` négatif, `requestId` absent ou non UUID. Exemple : `Expected no excess property at ["projectRoot"]`.

**Mauvais root, tâche, contexte (R1-R12).** `projectId` inexistant : `project_mismatch`. Fil inexistant : `binding_unavailable`. Fil de proj1 avec `projectId` de proj2 : `project_mismatch`. Fil réel de proj2 (root différent du magasin) : `project_mismatch` natif sur `list`, `show` et `cancel` (aucune existence de tâche étrangère révélée). Tâche inconnue : `task_unavailable` (show, journal, cancel). Journal absent : `journal_unavailable`. Offset hors résultat : `result_offset_invalid`.

**Cancel (C1-C3).** Rejeu avec le même `request_id` et la même enveloppe : même reçu. Même `request_id` avec une autre tâche : `envelope_mismatch`.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/network149.md#extrait-5).

**Fils virtuels en lecture seule.** `message.dispatch` (modes `start_immediately`, `queue_after_active`, `defer_start`), `provider.switch`, `thread.model-selection.set`, `thread.runtime-mode.set`, `thread.metadata.update`, `thread.fork`, `checkpoint.rollback`, `thread.delete` sont refusés sur `thread:bridget-task:<uuid>` (`OrchestratorSubagentThreadReadOnlyError`). `thread.visit` est accepté. Après tout cela : 0 run, 0 tentative, 0 tour fournisseur, 0 session fournisseur, 0 effet en attente, 0 message.

## Limites et obstacles

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/network149.md#extrait-6).

## Défauts de mes outils corrigés pendant la recette (aucun code de production touché)

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/network149.md#extrait-7).

## Empreintes

| Fichier | sha256 (16 premiers) |
|---|---|
| `bridget_fixture.mjs` | `e1b3f510e618e4b9` |
| `scenario_step.mjs` | `16f29610fccbec04` |
| `rpc149.ts` | `aaccea6dd2a9336d` |
| `matrix149.mjs` | `ae4f96cce7afebf2` |
| `db_counts.sh` | `244eb028ac9de785` |
| `fixture_store_init.mjs` | `eaf326e07019147d` |

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/network149.md#extrait-8).

## État final et nettoyage

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/149-sous-agents-lineage/validation/network149.md#extrait-9).
