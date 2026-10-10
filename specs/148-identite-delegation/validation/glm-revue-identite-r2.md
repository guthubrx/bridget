La ronde est complète. Voici le rapport R2.

---

# Revue R2 — identité Bridget par session T3 (lecture seule)

**Verdict : APPROVÉ pour le code.** Un manque de preuve runtime reste ouvert. Il est déjà planifié pour la validation finale. Aucun défaut de code nouveau trouvé.

---

## Réponses aux findings R1 — vérifiées

### Finding critique R1 (Mcp-Session-Id ignoré) — résolu en code

- `t3code_mcp.rs:135-146` : le client capture `mcp-session-id` sur la réponse `initialize`. La casse est correcte : minreq 2.14.1 force les noms d'en-têtes en minuscules (`make_ascii_lowercase`, `response.rs:592`). L'absence d'en-tête ferme l'appel (ligne 136). Le session-id est validé : non vide, ≤ 256, alphanumérique ASCII + `-`/`_` (lignes 137-144). L'injection d'en-tête par un serveur corrompu est impossible.
- `t3code_mcp.rs:128-130` : le session-id est renvoyé sur `notifications/initialized` et `tools/call`.
- Test TCP strict `http_keeps_sessions_distinct_and_checks_revocation_on_next_call` (`t3code_mcp.rs:345-445`) : le serveur factice applique exactement la règle d'admission du finding R1. Il exige `Mcp-Session-Id` sur notify, call et DELETE. Il rejette sinon. Deux cycles complets + un 401. C'est le test négatif exigé par R1.
- Le test interop Node (`BridgetRustInterop.test.ts`) monte le **vrai** `McpHttpServer.layerMcpTransport` avec `McpProtocol.v2025_06_18` stateful. L'admission réelle s'applique. Le finding ne peut pas être masqué par un mock de transport.

### Finding majeur R1 (sessions accumulées) — résolu

- `t3code_mcp.rs:85-111` : un DELETE part après chaque attestation. Même si la preuve est invalide. Si la preuve est valide, un DELETE en échec fait échouer l'appel (`cleanup?`, ligne 107) — comportement strict conforme au brief.
- Le DELETE passe par le middleware d'auth du montage (`McpHttpServer.ts:846-851` : `McpServer.layerHttp(...).pipe(Layer.provide(layerMcpAuthMiddleware))`). Le routeur du patch (`patches/effect@4.0.1.patch`, route DELETE : 400 sans session, 204 si terminée, 404 sinon) est enveloppé par ce middleware. Un DELETE sans Bearer résolvable reçoit 401. Pas de relâchement.
- Le test interop compte : 2 DELETE, avec les sessions exactement égales aux sessions émises (lignes 140-145). Puis 404 sur un POST réutilisant une session supprimée (lignes 148-166).

### Mineur R1 (test deletedAt) — résolu

`BridgetSession.test.ts:63-69` couvre inactif, archivé et supprimé (5 tests au total). Le handler refuse `deletedAt` (`OrchestratorMcpService.ts:1780`) puis `assertLiveCaller` refuse archivé / sans run actif / provider étranger (`OrchestratorMcpService.ts:1028-1040`).

### E0502 — corrigé selon le principal. `--no-run` vert annoncé. Non réexécuté ici (lecture seule). Aucun problème de compilation visible dans les fichiers lus.

---

## Points demandés par le brief — vérifiés

**Point de branchement `mcp_identity`** (`mcp_identity.rs:279-345`) :
- Une preuve refusée ferme l'appel **avant** toute lecture PID (`transpose()?`, ligne 281).
- Identités native et session divergentes → refus (`T3SessionUnavailable`, lignes 331-334).
- Native valide → `delegated_origin` native conservée (lignes 337-340). La preuve de session ne promeut jamais un enfant interne en principal.
- Marqueur enfant présent, même invalide ou cassé, interdit le repli sur la session seule : `has_delegated_marker` utilise `symlink_metadata`, et toute erreur autre que NotFound retourne vrai (lignes 308-322). Couvert par `session_proof_preserves_native_child_limits_and_rejects_conflicting_identity` et les tests spec133.

**PID ambigu non assoupli** : `t3_session_identity_env.rs` prouve le refus pour 4 env dégradées (endpoint seul, authorization seule, paire invalide, paire vide) **avec une identité PID native valide posée au préalable** (lignes 63-86). Le refus passe par le vrai point d'entrée `mcp::serve`. La paire vide est bien prise : `var_os` retourne `Some("")`, le chemin T3 est sélectionné puis échoue.

**Pas d'identité en argument** : `BridgetMcp.ts:43-51` — args `["mcp"]` seuls, identité par l'env du sous-processus. Refus des identités Bridget héritées (lignes 12-17).

**Pas de fuite de token** :
- Rust : toutes les erreurs sont réduites à `T3SessionUnavailable`, sans détail (t3code_mcp.rs).
- T3 : registry au hash SHA-256 seul (`McpSessionRegistry.ts:113-116, 153`) ; log de rejet sans token, seulement un booléen `presentedToken` (`McpHttpServer.ts:166-173`).
- Testkit : l'observateur HTTP n'enregistre jamais l'en-tête Authorization (`BridgetRustInterop.test.ts:50-61`).
- `test307` (`t3code_mcp.rs:301-342`) : après le 307, la destination n'a reçu aucune connexion. Le credential ne part jamais vers une autre origine.

**Pas de cache** : chaque appel d'outil refait l'attestation complète. Aucun état partagé, aucun credential conservé.

**Indépendance du moteur natif** : `t3code_mcp` est un connecteur pur (HTTP + socket Unix). Aucun appel d'orchestration T3. FR025/FR026 respectées.

**Mocks résiduels dans l'interop** : seuls la projection de stockage (`ThreadManagement`), `ServerEnvironment` et des dépendances vides sont simulés. Le transport Effect (admission stateful), la registry, le toolkit, le handler, le client Rust et le daemon Bridget sont réels. Le credential du thread est réellement émis par le daemon (`registerThread`, testkit lignes 133-172), pas forgé. `bridgetThreadUuid` doit rester synchrone de `stable_uuid` : toute divergence fait échouer le test, donc rien n'est masqué.

**Compatibilités d'interop vérifiées par lecture** :
- Token registry : base64url, 43 caractères (`McpSessionRegistry.ts:89, 131`). Il passe la validation Rust (32..=256, charset).
- Endpoint registry : `http://127.0.0.1:{port}/mcp`, `http://localhost:…` ou `http://[::1]:…` — dans l'allowlist Rust.
- Anti-SSRF : le port vient du runtime validé par PID vivant `kill(pid,0)` (`t3code_contract.rs:176`), et l'URL d'appel est reconstruite depuis `base_url()` (toujours `127.0.0.1`, lignes 120-122), jamais depuis l'env.

---

## Findings

1. **MINEUR — manque de preuve runtime, pas un défaut de code.** Le test interop est opt-in : sans `BRIDGET_148_RUST_EXECUTABLE`, il est skippé (`BridgetRustInterop.test.ts:32-33`). Il n'a pas encore tourné. L'exécution des tests Rust compilés (`--no-run` vert) n'est pas non plus observée. Ces preuves doivent venir de la passe de validation finale déjà planifiée. Tout l'édifice code est vérifié par lecture croisée des deux sources.

2. **OBSERVATION — coût par appel, pas un défaut.** Chaque appel d'outil coûte 4 allers-retours HTTP (initialize, initialized, call, DELETE) plus l'échange Unix. La réponse choisie élimine l'accumulation par sessions éphémères. La recommandation R1 d'une session réutilisée n'a pas été suivie, mais le problème de sécurité est clos. Choix défendable : aucun état partagé, aucune fenêtre de vol élargie.

---

## Verdict

**APPROVE** — pour le code de cette ronde. Les trois findings R1 reçoivent une réponse correcte et complète. Le branchement `mcp_identity` est fail-closed sur tous les chemins examinés. L'approbation reste conditionnée à la validation runtime finale : exécution du test interop réel avec le binaire isolé, et exécution de la suite Rust (TCP strict, test307, subprocess env). Aucun de ces points ne demande de changement de code sur la base de la lecture.
