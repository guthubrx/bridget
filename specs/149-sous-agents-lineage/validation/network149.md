# Recette réseau 149 - RPC réels sur WebSocket, scopes et refus (T036 partiel)

Date : 2026-10-10. Testeur : Claude Sonnet 5.5 (claudeAgent), r2. Aucun edit de production, aucun commit, aucune case cochée.

**Statut : PARTIEL.** La couche T3 est prouvée (serveur réel, auth réelle, RPC réels). La couche native (MCP privé authentifié, deux sessions, rotation/révocation, identité étrangère) n'est PAS prouvée : le reçu `native149-debug-receipt.json` n'existe pas. T036 ne peut pas être coché.

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

| Couche | État |
|---|---|
| Serveur T3 du WT (`apps/server/src/bin.ts`, Node 24.13.1, PID 74293, `127.0.0.1:14773`) | RÉEL |
| Auth : jetons bearer émis par `bin.ts auth session issue`, ticket WS `/api/auth/websocket-ticket`, scopes vérifiés par `RpcAuthorization` | RÉEL |
| Transport : WebSocket + Effect RPC JSON (client `rpc149.ts` avec `WsRpcGroup`, et trames brutes) | RÉEL |
| Lecteur `BridgetReader`, service `BridgetLineage`, projection SQL `statev2.sqlite` privée | RÉEL |
| Daemon Bridget | SIMULÉ : `bridget_fixture.mjs` (CLI Node qui lit un magasin JSON privé). Il ne lance aucun modèle, aucune mission. |
| Données | Synthétiques et nommées `[recette149]`. Elles ne valident ni SC001 de bout en bout, ni T037. |

Isolation : cache `/Users/moi/.cache/bridget149-ui` (0700), T3 home privé, ports loopback 14773 et 15733, aucune base, config, secret ni service de production, aucune app T3 Desktop.

## Commandes exactes

```
# jetons (CLI d'auth du WT, base privée T3CODE_HOME=$C/t3home, fichiers 0600 sous $C)
node apps/server/src/bin.ts auth session issue --ttl 2h --token-only --scope orchestration:read --scope orchestration:operate   # operate
node apps/server/src/bin.ts auth session issue --ttl 2h --token-only --scope orchestration:read                                  # read
node apps/server/src/bin.ts auth session issue --ttl 2h --token-only --scope settings:write                                      # noorch
node rpc149.ts setup <tok-operate>                 # project.create (HTTP) + thread.create (RPC dispatchCommand), zéro tour
node rpc149.ts call <tok> <méthode> '<json>' [--take N] [--wait-ms M] [--repeat N]   # client Effect typé
node rpc149.ts raw  <tok> <méthode> '<json>'                                          # trame brute, le serveur reçoit la charge telle quelle
node matrix149.mjs all
```

Répertoire des outils : `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/149-sous-agents-lineage/specs/149-sous-agents-lineage/validation/ui-recipes/`. Le lien `ui-recipes/node_modules` pointe vers `apps/server/node_modules` du WT (symlink, aucune copie).

## Constats prouvés

**Scopes (S1-S10).** Le jeton `orchestration:read` lit `list`, `show`, `journal`, `watch` (ready seq 0) et le flux `journal`. Il est refusé sur `bridget.lineage.cancel` (`EnvironmentAuthorizationError`, scope manquant). Le jeton `settings:write` seul est refusé sur `read` et `cancel`. Sans jeton ou avec un jeton invalide, le ticket WS rend HTTP 401 (`missing_credential` / `invalid_credential`). Le jeton operate annule une tâche déjà terminale sans inventer `cancelled` : l'état courant revient (`result_available`, `failed`).

**Autorité inconnue ou en trop (I1-I13).** Treize charges non conformes, envoyées en trames brutes, sont refusées par le serveur : `projectRoot`, `workspaceRoot`, `rootOwnerAgentId` (extra), `force` (extra sur cancel), `afterSeq` sur watch (extra), action inconnue, `taskId` non UUID ou UUID en majuscules, `limit` 16385 et 101, `afterSeq` négatif, `requestId` absent ou non UUID. Exemple : `Expected no excess property at ["projectRoot"]`.

**Mauvais root, tâche, contexte (R1-R12).** `projectId` inexistant : `project_mismatch`. Fil inexistant : `binding_unavailable`. Fil de proj1 avec `projectId` de proj2 : `project_mismatch`. Fil réel de proj2 (root différent du magasin) : `project_mismatch` natif sur `list`, `show` et `cancel` (aucune existence de tâche étrangère révélée). Tâche inconnue : `task_unavailable` (show, journal, cancel). Journal absent : `journal_unavailable`. Offset hors résultat : `result_offset_invalid`.

**Cancel (C1-C3).** Rejeu avec le même `request_id` et la même enveloppe : même reçu. Même `request_id` avec une autre tâche : `envelope_mismatch`.

**Curseur et génération (pagination).** Avec 134 tâches (2 pages), le serveur fusionne les pages et projette 134 fils virtuels. Une mutation entre deux pages rend `snapshot_changed` côté CLI ; T3 repart de la page 1 et réussit (P2). Quand les trois tentatives échouent, T3 rend `snapshot_changed` et le nombre de fils virtuels reste 134 (P3). Limite : le watch du navigateur tournait en parallèle et a pu consommer des mutations ; la preuve « aucun snapshot partiel » repose sur le décompte stable des fils, pas sur une exclusion stricte.

**Fils virtuels en lecture seule.** `message.dispatch` (modes `start_immediately`, `queue_after_active`, `defer_start`), `provider.switch`, `thread.model-selection.set`, `thread.runtime-mode.set`, `thread.metadata.update`, `thread.fork`, `checkpoint.rollback`, `thread.delete` sont refusés sur `thread:bridget-task:<uuid>` (`OrchestratorSubagentThreadReadOnlyError`). `thread.visit` est accepté. Après tout cela : 0 run, 0 tentative, 0 tour fournisseur, 0 session fournisseur, 0 effet en attente, 0 message.

## Limites et obstacles

- **MCP privé (bloquant pour T036 complet).** La fixture refuse le namespace `mcp` (`unavailable`). Les scopes MCP read/operate, les deux sessions authentifiées, la rotation/révocation et le refus d'identité étrangère exigent le binaire natif 149. Reprise : quand `native149-debug-receipt.json` existe et que son hash correspond à `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-target/debug/bridget`, relancer le serveur avec `T3CODE_BRIDGET_EXECUTABLE` pointant ce binaire et un registre privé de recette.
- S149-19 (conversation B du même projet ne voit pas la lineage de A) : non prouvé. La fixture ignore `--t3-thread` ; seul le daemon natif sélectionne par conversation racine.
- `orchestration.launchThread` sur fil virtuel : non exécuté. Un échec de la garde (`ThreadLaunchService.ts:728`) aurait pu démarrer un vrai fournisseur avec les identifiants de l'utilisateur. La garde est couverte par les tests serveur r2.
- Les refus de charge invalide sortent en `Die` (défaut) avec le message de schéma, pas en erreur typée. Le refus est fermé ; je le signale pour information.
- Le message « This subagent is run by its provider and cannot take messages » s'affiche aussi pour `thread.delete`, `fork`, `rollback`. Cosmétique.
- Les comptages d'invocations CLI individuelles n'existent pas : la fixture ne journalise pas ses appels. « Aucun double lancement » est prouvé au niveau T3 (compteurs de runs, tours, sessions, effets à 0), pas au niveau du daemon natif.

## Défauts de mes outils corrigés pendant la recette (aucun code de production touché)

1. `bridget_fixture.mjs` n'avait pas le bit exécutable (644) : `unavailable` sur le tout premier appel. Corrigé par `chmod 755`.
2. `process.exit` final tuait `lineage watch` après `ready` (31 ms) : `command_failed`. Corrigé.
3. Le suivi `--follow` ré-émettait seq 1-2 : le lecteur T3 le refuse à raison (`invalid_output`). Corrigé (seulement les événements après le dernier `next_seq`).
4. Le suivi `--follow` s'arrêtait seul à 60 s, ce qui coupait le flux. Plafond remonté à 20 min.
5. `rpc149.ts` tronquait stdout au-delà de 64 Ko (`process.exit` sur pipe). Corrigé (vidage avant sortie).

## Empreintes

| Fichier | sha256 (16 premiers) |
|---|---|
| `bridget_fixture.mjs` | `e1b3f510e618e4b9` |
| `scenario_step.mjs` | `16f29610fccbec04` |
| `rpc149.ts` | `aaccea6dd2a9336d` |
| `matrix149.mjs` | `ae4f96cce7afebf2` |
| `db_counts.sh` | `244eb028ac9de785` |
| `fixture_store_init.mjs` | `eaf326e07019147d` |

Source T3 : WT `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`, base `33f6d04e116430bf7f0011902d6af3868163f2d1`, `git diff HEAD` sha `c938b430c79493f0`, 48 fichiers modifiés.

## État final et nettoyage

Les trois processus owned ont été arrêtés un par un (SIGTERM, sans `-9`) : serveur T3 74293, shell Vite 76196, listener Vite 76205 (orphelin, cwd vérifié). Ports 14773 et 15733 libres, 0 processus fixture. Jetons bearer et pairing supprimés. Conservés pour examen : `/Users/moi/.cache/bridget149-ui/t3home` (DB privée), `store.json`, `store.json.bak149`, `ids.json`, `ids2.json`, `logs/`. Aucun job différé, aucune activation planifiée.
