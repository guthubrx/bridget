# Rapport de recette R4 — Identité/délégation 148

**Verdict : SUCCÈS. RC=0. 6 tests PASS sur 2 fichiers. Aucun fix appliqué.**

## 1. Empreintes vérifiées avant exécution (lecture seule)

| Élément | SHA256 | Constat |
|---|---|---|
| Binaire Rust (complet) | `a60f4e36…3378b1e` | Conforme au préfixe figé `a60f4…` |
| Binaire — head 1 Mio | `ba574f7c…` | Capturé pour la traçabilité |
| Binaire — tail 1 Mio | `08b226b1…` | Capturé pour la traçabilité |
| `BridgetRustInterop.test.ts` | `da9dab7d…d54ee4a28` | Byte-identique à R3 — réseau inchangé |
| `BridgetRustInterop.testkit.ts` | `72808e22…f330f43d` | Conforme au correctif annoncé |
| `BridgetRustInteropObserver.test.ts` | `51a4b806…9a2a87a3` | Conforme |

Le binaire n'a pas été recompilé. Les 389 tests T3 déjà PASS en R2/R3 n'ont pas été rejoués.

## 2. Résultat d'exécution

Commande exacte du brief, depuis T3 `apps/server`, `BRIDGET_148_RUST_EXECUTABLE` positionné. Deux exécutions (normale puis verbose) : RC=0 chacune, 5,6 s puis 5,5 s.

- `BridgetRustInteropObserver.test.ts` — 5 PASS : surcharges `writeHead` dont `undefined` en 3e argument, headers bruts alternés, conservation du header préexistant.
- `BridgetRustInterop.test.ts > native148 Rust uses real HTTP session headers, closes sessions, and refuses revoked credentials` — **1 PASS en 587 ms**. Le test a réellement tourné (portail `it.effect` actif grâce à la variable d'environnement, pas de skip).

## 3. Preuve assertion par assertion (source `BridgetRustInterop.test.ts`)

Transport réel : serveur HTTP Node/Effect vivant (l. 49-68), `McpHttpServer.layerMcpTransport` réel (l. 111), toolkit et handlers réels (l. 107-109). Seuls les dépôts de données sont des fixtures (l. 69-92). Le binaire Rust est passé au fixture (l. 102-103).

| Checkpoint du brief | Preuve (toutes assertions PASS) |
|---|---|
| 2 conversations distinctes, même `providerInstanceId` codex | l. 41, 125-128 : même endpoint, même instance, `providerSessionId` et headers d'autorisation différents par thread ; agents A ≠ B (l. 133) ; chaque réponse outils contient seulement l'identité de son propre agent (l. 150-152) |
| Sessions / idempotence | l. 156-158 : exactement 2 POST sans `Mcp-Session-Id` (initialize) ; l. 159-162 : exactement 4 POST avec session ; l. 163-165 : exactement 2 sessions émises, distinctes |
| DELETE | l. 166-171 : exactement 2 DELETE ; l'ensemble des sessions supprimées égale l'ensemble des sessions émises ; l. 172 : tous les statuts observés sont 2xx |
| Périmée 404 | l. 174-192 : POST avec l'ancienne session A et le credential A encore vivant → statut exact `404` |
| Revoke A → 401 + `t3_session_unavailable` | l. 194-198 : après `revokeThread(threadA)`, l'appel A renvoie `isError: true` et `code: "t3_session_unavailable"` ; dernier statut observé côté serveur = `401` ; l. 199 : toujours 2 DELETE (pas de suppression implicite) |
| B reste valide | l. 200-203 : appel B suivant sans erreur ; `attestedThreads = [A, B, B]` — le service d'identité réel a attesté B une seconde fois après la révocation de A |
| 3e DELETE | l. 204-210 : dernière requête = DELETE en 2xx ; total DELETE = 3 ; total de sessions émises distinctes = 3 (B a ré-initialisé puis re-fermé proprement) |

Le modèle d'échange observé est cohérent de bout en bout : par appel, initialize + notification + appel + DELETE ; ré-initialisation propre de B après la clôture ; refus sec d'A après révocation.

## 4. Conformité au périmètre

- Lecture seule : aucun edit, aucune config, aucun Git, aucun restart, aucune recompilation.
- Aucun secret ni corps de requête enregistré dans ce rapport (observation passive des statuts et headers de session uniquement).
- Processus modèle non lancés : conforme au brief, le partage app-server est prouvé séparément dans les tests adapters.

La correction testkit (sélection `arguments.length >= 3` + `Reflect.apply`) est validée par les 5 tests Observer et par la réussite complète du scénario réseau qui échouait en R3. La recette R4 est atteinte sur tous les points.
