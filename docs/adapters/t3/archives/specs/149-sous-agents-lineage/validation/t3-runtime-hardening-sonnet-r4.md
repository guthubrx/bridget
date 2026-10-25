# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r4.md](../../../../../../../specs/149-sous-agents-lineage/validation/t3-runtime-hardening-sonnet-r4.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

Testeur : Sonnet 5.5 normal, effort high. Date : 2026-10-10.
Base T3 : `33f6d04e116430bf7f0011902d6af3868163f2d1`. WT : `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`.

<a id="extrait-2"></a>

## Extrait 2

| Couche | Statut |
|---|---|
| Serveur T3 du WT (`apps/server/src/bin.ts`, Node 24.13.1, `127.0.0.1:14775`), auth bearer scopée, RPC WebSocket, `BridgetLineage`, `Orchestrator`, projection SQL privée | RÉEL |
| UI web du WT (Vite+ `localhost:15735`), navigateur preview T3 (onglet à moi, profil incognito, fermé en fin de recette) | RÉEL |
| Daemon Bridget (CLI `bridget_fixture.mjs` sur `store.json` privé) | SIMULÉ, nommé |
| Données de tâches, journaux, résultats | SYNTHÉTIQUES |

<a id="extrait-3"></a>

## Extrait 3

| Volet | PASS | FAIL | SKIP / note |
|---|---|---|---|
| Tests serveur 149 (5 fichiers : Lineage, Reader, Orchestrator, ProjectionStore, ThreadLaunch) | 120 | 0 | 115 avant, +5 nouveaux tests F1 |
| Tests UI/client ciblés (journal, ThreadRelationshipsControl, client-runtime) | 22 | 0 | journal 7 -> 9 (+2 tests F2) |
| `BridgetLineage149.test.ts` seul | 23 | 0 | 7 échecs AVANT adaptation du harnais (voir plus bas) |
| Matrice réseau `matrix149.mjs all` (serveur r4 neuf) | 38 | 0 | |
| Rejeux r3 `replay_r3.mjs all` (22 + Z1) | 23 | 0 | Z1 était un FINDING en r3 ; il passe |
| Rejeux F1 `replay_r4.mjs all` (nouveau) | 22 | 0 | ZD-1 est une observation, limite fixture |
| Mesures navigateur F2 (preview T3) | 9 | 0 | 9 captures |
| `tsc --noEmit` serveur | - | - | 16 erreurs, toutes baseline, 0 dans les fichiers 149 |
| `tsc --noEmit` web | - | - | 10 erreurs, toutes baseline (`MessagesTimeline.logic.test.ts`), 0 dans les fichiers Bridget |
| Compteurs SQL en fin de recette | runs 0, run_attempts 0, provider_turns 0, provider_sessions 0, provider_threads 0, bindings 0, runtime_requests 0, workflows 0, messages 0 | | `effect_outbox` = 4, voir ci-dessous |

<a id="extrait-4"></a>

## Extrait 4

| Largeur x hauteur | Hauteur barre | `paddingBottom` | Écart dernière ligne / barre | Composeur | « Arrêter » (elementFromPoint) |
|---|---|---|---|---|---|
| 1280 x 800, carte « Thread details » ouverte | 60 px | 72 px | 12 px | aucun | atteignable |
| 480 x 800 | 54 px | 66 px | 12 px | aucun | atteignable |
| 375 x 800 | 54 px | 66 px | 12 px | aucun | atteignable |
| 1280 x 800 après redimensionnement 375 -> 1280 sans rechargement | 60 px | 72 px | 12 px | aucun | - |
| Après clic « Reconnecter » (démontage/remontage du flux) | 60 px | 72 px (pas de doublement) | 12 px | aucun | - |
| T3 (résultat disponible, 2 + 20 lignes de journal, résultat lu) | 60 px | 72 px | 12 px | aucun | - |

<a id="extrait-5"></a>

## Extrait 5

Sorties brutes (même dossier de validation) : `network149-matrix-run-r4.txt`, `t3-runtime-r4-replay-r3.txt`, `t3-runtime-r4-replay-r4.txt`.

<a id="extrait-6"></a>

## Extrait 6

1. Un `snapshot_changed` épuisé dégrade les 135 fils d'un coup (`thread.metadata-updated` x135), comme en r3.
2. Après panne globale, la bannière d'indisponibilité du flux watch exige un clic « Reconnecter Bridget » alors que la projection est déjà rétablie (design, test existant).
3. Limites de ma fixture : le CLI ignore `--t3-thread` (S149-19 non prouvé), refuse le namespace `mcp`, et un second fil hôte du même projet ne peut pas être lié (ZD-1).

<a id="extrait-7"></a>

## Extrait 7

```
export C=/Users/moi/.cache/bridget149-ui.r4 RECIPE149_CACHE=$C RECIPE149_T3_PORT=14775 RECIPE149_WEB_PORT=15735
PATH=/Users/moi/.cache/t3-toolchains/148/node-v24.13.1-darwin-arm64/bin:$PATH
# tests (racine du WT)
./node_modules/.bin/vp test run apps/server/src/bridget/BridgetLineage149.test.ts apps/server/src/bridget/BridgetReader149.test.ts apps/server/src/orchestration-v2/Orchestrator.bridget149.test.ts apps/server/src/orchestration-v2/ProjectionStore.bridget149.test.ts apps/server/src/orchestration-v2/ThreadLaunchService.bridget149.test.ts
./node_modules/.bin/vp test run apps/web/src/components/BridgetTaskJournal149.test.tsx apps/web/src/components/chat/ThreadRelationshipsControl.bridget149.test.tsx packages/client-runtime/src/state/orchestration.bridget149.test.ts
# recette (depuis ui-recipes/)
node fixture_store_init.mjs $C/store.json $C/proj ; zsh run_recipe_server.sh start
T3CODE_HOME=$C/t3home node apps/server/src/bin.ts auth session issue --ttl 3h --token-only --scope ...   # jetons read, operate, settings:write
node --no-warnings rpc149.ts setup|setup2 $C/tok-operate.txt
node --no-warnings matrix149.mjs all ; node --no-warnings replay_r3.mjs all ; node --no-warnings replay_r4.mjs all
vp dev (apps/web, PORT=15735 T3CODE_PORT=14775) ; T3CODE_HOME=$C/t3home node apps/server/src/bin.ts auth pairing create --ttl 10m --base-url http://localhost:15735
```

<a id="extrait-8"></a>

## Extrait 8

Onglet preview fermé. Listener Vite 72503 arrêté en SIGTERM (cwd WT vérifié) ; shell Vite 72488 déjà terminé ; serveur T3 56889 arrêté par `run_recipe_server.sh stop` (SIGTERM, PID et cwd vérifiés, pas Firefox). Ports 14775 et 15735 libres, 0 processus fixture. Jetons bearer, jeton de recette et pidfiles supprimés ; jeton d'appairage consommé. Aucun job différé. Conservés : `/Users/moi/.cache/bridget149-ui.r4` (DB privée `t3home`, `store.json` et copies `store.json.r4*`, `logs/`).
