# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/network149.md](../../../../../../../specs/149-sous-agents-lineage/validation/network149.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

**Statut : PARTIEL.** La couche T3 est prouvée (serveur réel, auth réelle, RPC réels). La couche native (MCP privé authentifié, deux sessions, rotation/révocation, identité étrangère) n'est PAS prouvée : le reçu `native149-debug-receipt.json` n'existe pas. T036 ne peut pas être coché.

<a id="extrait-2"></a>

## Extrait 2

| Couche | État |
|---|---|
| Serveur T3 du WT (`apps/server/src/bin.ts`, Node 24.13.1, PID 74293, `127.0.0.1:14773`) | RÉEL |
| Auth : jetons bearer émis par `bin.ts auth session issue`, ticket WS `/api/auth/websocket-ticket`, scopes vérifiés par `RpcAuthorization` | RÉEL |
| Transport : WebSocket + Effect RPC JSON (client `rpc149.ts` avec `WsRpcGroup`, et trames brutes) | RÉEL |
| Lecteur `BridgetReader`, service `BridgetLineage`, projection SQL `statev2.sqlite` privée | RÉEL |
| Daemon Bridget | SIMULÉ : `bridget_fixture.mjs` (CLI Node qui lit un magasin JSON privé). Il ne lance aucun modèle, aucune mission. |
| Données | Synthétiques et nommées `[recette149]`. Elles ne valident ni SC001 de bout en bout, ni T037. |

<a id="extrait-3"></a>

## Extrait 3

Isolation : cache `/Users/moi/.cache/bridget149-ui` (0700), T3 home privé, ports loopback 14773 et 15733, aucune base, config, secret ni service de production, aucune app T3 Desktop.

<a id="extrait-4"></a>

## Extrait 4

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

<a id="extrait-5"></a>

## Extrait 5

**Curseur et génération (pagination).** Avec 134 tâches (2 pages), le serveur fusionne les pages et projette 134 fils virtuels. Une mutation entre deux pages rend `snapshot_changed` côté CLI ; T3 repart de la page 1 et réussit (P2). Quand les trois tentatives échouent, T3 rend `snapshot_changed` et le nombre de fils virtuels reste 134 (P3). Limite : le watch du navigateur tournait en parallèle et a pu consommer des mutations ; la preuve « aucun snapshot partiel » repose sur le décompte stable des fils, pas sur une exclusion stricte.

<a id="extrait-6"></a>

## Extrait 6

- **MCP privé (bloquant pour T036 complet).** La fixture refuse le namespace `mcp` (`unavailable`). Les scopes MCP read/operate, les deux sessions authentifiées, la rotation/révocation et le refus d'identité étrangère exigent le binaire natif 149. Reprise : quand `native149-debug-receipt.json` existe et que son hash correspond à `/Volumes/8TB2/50-repos-archives/validation-cache/bridget149-target/debug/bridget`, relancer le serveur avec `T3CODE_BRIDGET_EXECUTABLE` pointant ce binaire et un registre privé de recette.
- S149-19 (conversation B du même projet ne voit pas la lineage de A) : non prouvé. La fixture ignore `--t3-thread` ; seul le daemon natif sélectionne par conversation racine.
- `orchestration.launchThread` sur fil virtuel : non exécuté. Un échec de la garde (`ThreadLaunchService.ts:728`) aurait pu démarrer un vrai fournisseur avec les identifiants de l'utilisateur. La garde est couverte par les tests serveur r2.
- Les refus de charge invalide sortent en `Die` (défaut) avec le message de schéma, pas en erreur typée. Le refus est fermé ; je le signale pour information.
- Le message « This subagent is run by its provider and cannot take messages » s'affiche aussi pour `thread.delete`, `fork`, `rollback`. Cosmétique.
- Les comptages d'invocations CLI individuelles n'existent pas : la fixture ne journalise pas ses appels. « Aucun double lancement » est prouvé au niveau T3 (compteurs de runs, tours, sessions, effets à 0), pas au niveau du daemon natif.

<a id="extrait-7"></a>

## Extrait 7

1. `bridget_fixture.mjs` n'avait pas le bit exécutable (644) : `unavailable` sur le tout premier appel. Corrigé par `chmod 755`.
2. `process.exit` final tuait `lineage watch` après `ready` (31 ms) : `command_failed`. Corrigé.
3. Le suivi `--follow` ré-émettait seq 1-2 : le lecteur T3 le refuse à raison (`invalid_output`). Corrigé (seulement les événements après le dernier `next_seq`).
4. Le suivi `--follow` s'arrêtait seul à 60 s, ce qui coupait le flux. Plafond remonté à 20 min.
5. `rpc149.ts` tronquait stdout au-delà de 64 Ko (`process.exit` sur pipe). Corrigé (vidage avant sortie).

<a id="extrait-8"></a>

## Extrait 8

Source T3 : WT `/Users/moi/11.Repositories/t3code-local/.worktrees/149-sous-agents-lineage`, base `33f6d04e116430bf7f0011902d6af3868163f2d1`, `git diff HEAD` sha `c938b430c79493f0`, 48 fichiers modifiés.

<a id="extrait-9"></a>

## Extrait 9

Les trois processus owned ont été arrêtés un par un (SIGTERM, sans `-9`) : serveur T3 74293, shell Vite 76196, listener Vite 76205 (orphelin, cwd vérifié). Ports 14773 et 15733 libres, 0 processus fixture. Jetons bearer et pairing supprimés. Conservés pour examen : `/Users/moi/.cache/bridget149-ui/t3home` (DB privée), `store.json`, `store.json.bak149`, `ids.json`, `ids2.json`, `logs/`. Aucun job différé, aucune activation planifiée.
