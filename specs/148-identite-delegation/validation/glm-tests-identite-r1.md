# Rapport de test — session 148 identité/délégation

## Résultats d'exécution

| Commande | Retour | Résultat |
|---|---|---|
| `cargo test -p bridget-daemon t3code_mcp --lib` (Bridget WT) | 0 | **5 passed / 0 failed / 0 ignored** (1145 filtrés) |
| `cargo test -p bridget-daemon mcp_identity --lib` (Bridget WT) | 0 | **15 passed / 0 failed / 0 ignored** (1135 filtrés) |
| `pnpm exec vp test run src/mcp/BridgetSession.test.ts src/mcp/McpSessionRegistry.test.ts` (T3 WT `apps/server`) | 0 | **2 fichiers / 11 tests passed** |

Chemins : Bridget WT `…/64.bridget/.worktrees/session-148-identite-delegation/crates/bridget-daemon/src/{t3code_mcp,mcp_identity}.rs` ; T3 WT `…/t3code-local/.worktrees/session-148-identite-delegation/apps/server/src/mcp/`.

## Mouvement du code (worktrees édités en parallèle pendant les tests)

- Rust début → fin : `t3code_mcp.rs` a5656e3e…→**f7c23242…**, `mcp_identity.rs` f0d17bdf…→**a1e3add7…**, `lib.rs` be1f0993…→**6c24f906…** (`t3code_identity.rs` 88d4c7a4 inchangé).
- T3 début → fin : `BridgetSession.test.ts` cb69b13c…→**e5ecfb04…** (10→11 tests), `BridgetMcp.ts` ca5a2bcc…→**701e425e…** (les deux autres inchangés).
- Événements transitoires observés : 2 casses de compilation (variant `NativeDelegation` non câblé ; `delegation_mcp`/`tempfile` manquants) et 1 échec de `local_binding_requires_current_daemon_admission` (errno 22 dans le thread serveur). Tout cela concernait des états intermédiaires. Les verts ci-dessus sont obtenus sur les hashes finaux.

## Couverture vs contrat — ce qui est prouvé

- **Mauvais endpoint** : allowlist stricte (port du runtime réel lu sur disque). Refus du mauvais port, host externe, query, slash final, userinfo `@evil.test`, CRLF dans le token.
- **Révocation à chaque opération** : vrai client HTTP (minreq) sur TcpListener loopback. Chaque appel refait initialize + `bridget_session`. Un 401 en cours de série → `T3SessionUnavailable`. Deux fils (thread:a/thread:b) restent distincts.
- **Refus fermés** : `isError:true`, erreur JSON-RPC, threadId vide ou avec caractère de contrôle, `structuredContent` absent.
- **Pas de repli sur identifiants bruts** : `combine_session_identity` — identité native en conflit avec la preuve session → refus ; échec de résolution enfant → la preuve session tient seul. Marqueurs enfant invalide/symlink → repli principal interdit (spec133).
- **Rattachement vivant** : socket vivante sans preuve privée → refus ; `Nack "instance revoked"` → refus ; `Registered` → succès (UnixListener réel, protocole forgé).
- **T3 côté serveur** : réponse conforme (version/environmentId/threadId/providerSessionId/providerInstanceId) ; deux sessions distinctes à instance partagée ; client externe, capability manquante, autre provider, fil inactif/archivé/**supprimé** → refus ; registre ne stocke qu'un hash, `revokeThread` invalide, expiration après fenêtre de vie, `touch` par tour maintient la session longue, `touch` d'un autre fil n'entretient pas le mien.

## Trous — preuves nécessaires, par priorité

**P1 — interop réelle (aucune preuve actuelle)**
1. **Bout-en-bout réel** : tous les serveurs sont factices. Côté Rust : HTTP réel sur loopback mais serveur forgé. Côté T3 : `HttpServer` factice + `ThreadManagement` mocké, zéro réseau. Rien ne prouve que le vrai runtime T3 émet un endpoint/token accepté par `validate_endpoint` (port réel, headers, `MCP-Protocol-Version 2025-06-18`, SSE réel). Preuve : un test qui démarre le vrai serveur T3 et exécute `resolve_current` contre lui.
2. **Paire env partielle** : le code refuse une seule variable posée (aucun repli PID). Clause explicite du contrat. **Aucun test** ne pose `BRIDGET_T3_MCP_ENDPOINT` sans `AUTHORIZATION` (ni l'inverse) pour vérifier le refus. Aucun test du tout n'installe ces variables.
3. **Non-repli PID au point de branchement** : le `if let Some(...) { return }` de `resolve_current_mcp_identity` n'est testé qu'indirectement (via `combine_session_identity` en valeurs injectées). Preuve : test env posée + preuve révoquée → erreur, sans résolution PID.

**P2**
4. `lifecycle.rs:463` exclut la paire de `pass_env` (credential jamais hérité par un enfant). Non testé.
5. 401 d'un vrai serveur après `revokeThread` : la révocation est testée au registre (`resolve` → undefined), le 401 est simulé par le serveur factice.
6. Réponse 3xx explicite : `with_max_redirects(0)` est dans le code, pas dans les tests.
7. Credential auxiliaire réel : `Registered` avec `credential: None` suffit au test ; le chemin avec credential fourni et vérifié n'est pas prouvé.

**P3** : cross-use de token (structurellement impossible, non testé) ; rejet d'un `environmentId` étranger côté T3 ; sessions concurrentes (testées séquentiellement).

## Limites

Tests exécutés sur des états mouvants. Les verts sont valables pour les hashes finaux listés ci-dessus. Ils peuvent être invalidés par la prochaine édition des agents Codex. Faux HTTP partout côté serveur : la conformité protocole et les refus sont prouvés en isolation. L'accord réel des deux bords reste à démontrer (P1-1 à P1-3).
