# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r3.md](../../../../../../../specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r3.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Couche | État |
|---|---|
| Serveur T3 du WT (`apps/server/src/bin.ts`, Node 24.13.1, PID 74491, `127.0.0.1:14774`), auth bearer scopée, WebSocket RPC, `BridgetLineage`, `Orchestrator`, projection SQL privée | RÉEL |
| UI web du WT (Vite+ dev `localhost:15734`), navigateur preview T3, onglet à moi (fermé en fin de recette) | RÉEL |
| Daemon Bridget : `bridget_fixture.mjs` sur un magasin JSON privé | **SIMULÉ** (aucun modèle, aucun binaire natif) |
| Données | Synthétiques `[recette149]` |

<a id="extrait-2"></a>

## Extrait 2

Isolation : cache neuf `/Users/moi/.cache/bridget149-ui.r3` (0700), base T3 privée neuve, ports 14774/15734. Les preuves r2 (`/Users/moi/.cache/bridget149-ui`, rapports r2, `ui149-screenshots/`) sont intactes.

<a id="extrait-3"></a>

## Extrait 3

| Replay | Résultat | Preuve mesurée |
|---|---|---|
| Journal 1/4 (fait modifié) | PASS | 3 événements : 1 `subagent.updated`, 2 `thread.metadata-updated` (la tâche + la racine). Les 3 autres tâches : 0 événement |
| Tick journal seul 1/4 | PASS | 1 événement : racine seule |
| Journal 1/134 | PASS | 3 événements (pas environ 270) |
| Tick journal seul 1/134 | PASS | 1 événement : racine seule |
| Création initiale de 130 tâches | PASS | 130 `thread.created`, 130 `subagent.updated`, 1 racine |
| 100 `list` + 100 `show` identiques | PASS | 0 événement |
| Nouvelle génération, faits inchangés | PASS | 135 `thread.metadata-updated` (134 fils + racine), 0 `subagent.updated`, 0 `thread.created`, 134 marqueurs réécrits |
| Panne globale (magasin illisible), puis reprise complète au même seq | PASS | `invalid_output`, racine et 134 fils `available:false` ; reprise : tous `true`, seq 14/14, 0 création, 0 subagent, 134 fils sans doublon |
| Base à 130 historiques (retrait de 130 tâches, même génération) | PASS | 130 fils `available:false` (observé seq 15, marqueur seq 14), 4 `true`, 134 fils conservés, aucun prune |
| `journal_unavailable` (T4, lecture + flux) | PASS | 0 événement, racine et T1/T2/T3 inchangés |
| `result_offset_invalid` | PASS | 0 événement, racine inchangée |
| `task_unavailable` d'un enfant connu de ce root | PASS | ce fil seul `false`, T1/T2/T4 et racine `true` ; reprise au snapshot suivant |
| `task_unavailable` d'une tâche liée à un AUTRE root, et d'un UUID inconnu | PASS | 0 événement, racine 1 et racine 2 intactes |
| Snapshot partiel / mutation entre pages | PASS | `snapshot_changed` après 3 tentatives, nombre de fils inchangé (134), 0 `thread.created`, 0 `subagent.updated` ; test unitaire : snapshot à curseur ouvert refusé sans écriture |
| Snapshot complet identique après indisponibilité | PASS | `available:true` rétabli au même seq, aucun doublon ; un second snapshot identique écrit 0 événement (test unitaire) |

<a id="extrait-4"></a>

## Extrait 4

MCP privé authentifié, rotation/révocation, identité étrangère, S149-19 (la fixture ignore `--t3-thread`), résultat retenu natif, reprise sans relance native, double lancement natif, SC001, T037. `orchestration.launchThread` sur fil virtuel non lancé (risque de vrai fournisseur ; garde couverte par les tests serveur). Aucune case T036/T039/T040 ne peut être cochée sur cette seule preuve.

<a id="extrait-5"></a>

## Extrait 5

| Fichier | sha256 |
|---|---|
| `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/bridget/BridgetLineage149.test.ts` | `961e14b4acae4ee8` |
| `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage/apps/server/src/orchestration-v2/Orchestrator.bridget149.test.ts` | `ba3210d373f8885a` |
| `ui-recipes/replay_r3.mjs` (nouveau) | `602630a9e446094c` |
| `ui-recipes/ev_counts.sh` (nouveau) | `7029efe1d499ff61` |
| `ui-recipes/scenario_step.mjs` (étapes `tick`, `remove-task`, `foreign-root`, `title`) | `e6c2740977309a26` |
| `ui-recipes/rpc149.ts` (cache/port par variables, mode `setup2`) | `4bda99c188b3fd06` |
| `ui-recipes/matrix149.mjs`, `run_recipe_server.sh`, `db_counts.sh` (cache/port par variables) | `fbba255c451be2e1`, `ce71c09a78d47dfb`, `3efb5a5cd70b46f2` |

<a id="extrait-6"></a>

## Extrait 6

```
export RECIPE149_CACHE=/Users/moi/.cache/bridget149-ui.r3 RECIPE149_T3_PORT=14774 RECIPE149_WEB_PORT=15734
PATH=/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin:$PATH
# tests (depuis la racine du WT)
./node_modules/.bin/vp test run apps/server/src/bridget/BridgetLineage149.test.ts apps/server/src/bridget/BridgetReader149.test.ts apps/server/src/orchestration-v2/Orchestrator.bridget149.test.ts apps/server/src/orchestration-v2/ProjectionStore.bridget149.test.ts apps/server/src/orchestration-v2/ThreadLaunchService.bridget149.test.ts
# recette (depuis ui-recipes/)
node fixture_store_init.mjs $RECIPE149_CACHE/store.json $RECIPE149_CACHE/proj
zsh run_recipe_server.sh start ; vp dev (apps/web, PORT=15734 T3CODE_PORT=14774)
node --no-warnings matrix149.mjs all ; node --no-warnings replay_r3.mjs all
```
