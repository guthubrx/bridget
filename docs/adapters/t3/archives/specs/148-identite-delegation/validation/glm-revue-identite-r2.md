# Archives techniques de l’adaptateur T3

Document d’origine : [specs/148-identite-delegation/validation/glm-revue-identite-r2.md](../../../../../../../specs/148-identite-delegation/validation/glm-revue-identite-r2.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- `t3code_mcp.rs:135-146` : le client capture `mcp-session-id` sur la réponse `initialize`. La casse est correcte : minreq 2.14.1 force les noms d'en-têtes en minuscules (`make_ascii_lowercase`, `response.rs:592`). L'absence d'en-tête ferme l'appel (ligne 136). Le session-id est validé : non vide, ≤ 256, alphanumérique ASCII + `-`/`_` (lignes 137-144). L'injection d'en-tête par un serveur corrompu est impossible.
- `t3code_mcp.rs:128-130` : le session-id est renvoyé sur `notifications/initialized` et `tools/call`.
- Test TCP strict `http_keeps_sessions_distinct_and_checks_revocation_on_next_call` (`t3code_mcp.rs:345-445`) : le serveur factice applique exactement la règle d'admission du finding R1. Il exige `Mcp-Session-Id` sur notify, call et DELETE. Il rejette sinon. Deux cycles complets + un 401. C'est le test négatif exigé par R1.
- Le test interop Node (`BridgetRustInterop.test.ts`) monte le **vrai** `McpHttpServer.layerMcpTransport` avec `McpProtocol.v2025_06_18` stateful. L'admission réelle s'applique. Le finding ne peut pas être masqué par un mock de transport.

<a id="extrait-2"></a>

## Extrait 2

- `t3code_mcp.rs:85-111` : un DELETE part après chaque attestation. Même si la preuve est invalide. Si la preuve est valide, un DELETE en échec fait échouer l'appel (`cleanup?`, ligne 107) — comportement strict conforme au brief.
- Le DELETE passe par le middleware d'auth du montage (`McpHttpServer.ts:846-851` : `McpServer.layerHttp(...).pipe(Layer.provide(layerMcpAuthMiddleware))`). Le routeur du patch (`patches/effect@4.0.1.patch`, route DELETE : 400 sans session, 204 si terminée, 404 sinon) est enveloppé par ce middleware. Un DELETE sans Bearer résolvable reçoit 401. Pas de relâchement.
- Le test interop compte : 2 DELETE, avec les sessions exactement égales aux sessions émises (lignes 140-145). Puis 404 sur un POST réutilisant une session supprimée (lignes 148-166).

<a id="extrait-3"></a>

## Extrait 3

**PID ambigu non assoupli** : `t3_session_identity_env.rs` prouve le refus pour 4 env dégradées (endpoint seul, authorization seule, paire invalide, paire vide) **avec une identité PID native valide posée au préalable** (lignes 63-86). Le refus passe par le vrai point d'entrée `mcp::serve`. La paire vide est bien prise : `var_os` retourne `Some("")`, le chemin T3 est sélectionné puis échoue.

<a id="extrait-4"></a>

## Extrait 4

**Pas de fuite de token** :
- Rust : toutes les erreurs sont réduites à `T3SessionUnavailable`, sans détail (t3code_mcp.rs).
- T3 : registry au hash SHA-256 seul (`McpSessionRegistry.ts:113-116, 153`) ; log de rejet sans token, seulement un booléen `presentedToken` (`McpHttpServer.ts:166-173`).
- Testkit : l'observateur HTTP n'enregistre jamais l'en-tête Authorization (`BridgetRustInterop.test.ts:50-61`).
- `test307` (`t3code_mcp.rs:301-342`) : après le 307, la destination n'a reçu aucune connexion. Le credential ne part jamais vers une autre origine.

<a id="extrait-5"></a>

## Extrait 5

**Indépendance du moteur natif** : `t3code_mcp` est un connecteur pur (HTTP + socket Unix). Aucun appel d'orchestration T3. FR025/FR026 respectées.

<a id="extrait-6"></a>

## Extrait 6

**Compatibilités d'interop vérifiées par lecture** :
- Token registry : base64url, 43 caractères (`McpSessionRegistry.ts:89, 131`). Il passe la validation Rust (32..=256, charset).
- Endpoint registry : `http://127.0.0.1:{port}/mcp`, `http://localhost:…` ou `http://[::1]:…` — dans l'allowlist Rust.
- Anti-SSRF : le port vient du runtime validé par PID vivant `kill(pid,0)` (`t3code_contract.rs:176`), et l'URL d'appel est reconstruite depuis `base_url()` (toujours `127.0.0.1`, lignes 120-122), jamais depuis l'env.
