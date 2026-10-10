Validation terminée (~20 min). Voici le rapport complet.

---

# Rapport R2 — Validation indépendante, session 148 identité/délégation

## 1. Résultats par commande

| Commande | RC | Résultat |
|---|---|---|
| Interop `BridgetRustInterop.test.ts` (réseau réel) | **1** | **1 failed / 1** — échec ligne 138 |
| Régressions T3 (7 fichiers) | 0 | **389 passed, 1 skipped** |
| `cargo test t3code_mcp --lib` | 0 | **6 passed / 6** |
| `cargo test mcp_identity --lib` | 0 | **15 passed / 15** |

Le skip T3 est explicite et attendu : `ClaudeMcp.test.ts > SPEC147 … metadata proof without a model` est un `it.live.skip` conditionné à `BRIDGET_147_CLAUDE_METADATA_COMMAND` absente. Ce n'est pas un masquage.

## 2. Échec interop — erreur brute, cause prouvée, reproduction

**Erreur brute** : `AssertionError: expected +0 to equal 2` — `BridgetRustInterop.test.ts:138` (`initialized.length` = 0).

**Ce qui a réellement marché sur le fil** (assertions lignes 129-136 passées) :
- 2 POST initialize sans session-id.
- 4 POST avec `mcp-session-id` renvoyé par le client Rust.
- 2 `tools/call` `bridget_who` réussis. Chaque réponse contient l'agentId du daemon réel. Donc **les deux attestations ont réussi**.
- Le daemon Rust réel a émis la credential. L'enregistrement du fil a été accepté.

**Cause racine prouvée — le défaut est dans l'instrumentation du test, pas dans la correction** :
1. Le serveur émet le header. Preuve : `mcpRuntime.ts:524-536` (effect 4.0.1) — `crypto.randomUUID()` + `setHeaders` via pre-response handler.
2. Effect écrit toutes les réponses Node via `writeHead`. Preuve : `NodeHttpServer.js:342`.
3. Micro-test Node (`/tmp/i148_probe.js`, cas B) : après `writeHead` incluant un header, le client `fetch` le reçoit sur le fil, mais `response.getHeader()` retourne `null` dans le hook `finish`. C'est un comportement Node confirmé (v26.9.0).
4. Le hook du test (`BridgetRustInterop.test.ts:57`) lit `issued` via `response.getHeader()`. Il est donc structurellement aveugle aux headers émis par effect. `initialized` vaut toujours 0.

**Preuve que le header voyage** : le client Rust renvoie un session-id que le serveur reconnaît (sessions enregistrées via `crypto.randomUUID()` côté serveur). Sans le header dans la réponse initialize, les appels auraient échoué en 404 (`mcpRuntime.ts:448-451`). Les appels ont réussi.

**Conséquence** : le test meurt ligne 138. **DELETE session, réemploi 404, token révoqué 401 et erreur Rust restent NON VALIDÉS.** La correction R1 (émission/capture/renvoi du header) est prouvée fonctionnelle sur le réseau. La partie fermeture de session n'a pas de preuve d'exécution. Le correctif nécessaire est dans le test : observer les headers côté client (fetch), pas via `getHeader` côté Node.

**Reproduction** : relancer la commande du brief telle quelle, ou `node /tmp/i148_probe.js`.

## 3. Réseau réel vs synthétique

- **Interop** : vrai transport réseau (NodeHTTP/Effect sur 127.0.0.1, port aléatoire). Vrai daemon Rust (socket unix + credential). Vrai client `bridget mcp`. Fixtures limitées aux données stockées (projection de fils, registres vides). La boucle HTTP complète est réelle.
- **`t3code_mcp` (Rust)** : vrai TCP local (`TcpListener` embarqué), mais serveur factice contrôlé — pas le vrai serveur T3. Identité daemon mockée (`mock_socket`, `mock_private_identity`).
- **`mcp_identity` (Rust)** : tests unitaires de résolution de filiation. Pas de transport réseau.
- **Non couvert par moi** (périmètre de l'autre agent) : bibliothèques Rust complètes, E2E native, test env subprocess.

## 4. Mouvements détectés pendant la validation

L'environnement n'était **pas figé**. Un autre agent travaillait dans les mêmes worktrees.

- Exécutable : début `09113df6…cc18` (conforme au brief, vérifié avant le run) → fin `a2dab634…1f95b` (recompilé 06:02:53). Mon run interop a bien utilisé `09113df6`.
- Sources T3 : digest `11bc755a…` → `8c491263…`. Modifiés après mon run interop : `ClaudeMcp.ts/.test.ts`, `ClaudeAdapterV2.ts/.test.ts` (05:58-06:00). Mon run de régressions (06:00:09) a exécuté ces versions récentes.
- Sources Rust : digest `509cf2d9…` → `bac3d562…`. Modifiés : `wrapper.rs`, `daemon.rs`, `delegation.rs`, `daemon/native_delegation.rs`. Les modules que j'ai testés (`t3code_mcp.rs` 05:44, `mcp_identity.rs` 05:31) n'ont pas bougé depuis avant l'interop.
- Le test interop lui-même (05:46:11) n'a pas été retouché après mon run. Ma lecture correspond à la version exécutée.

## 5. Verdict

- Régressions T3 : **PASS** (389/389 exécutés).
- Rust ciblé : **PASS** (21/21).
- Recette interop réseau réel : **ÉCHEC — mais sur un défaut de mesure du test, pas sur le comportement**. La moitié aval de la recette (DELETE, 404, 401, erreur Rust) reste à valider après correction du hook d'observation. Je recommande de faire corriger `issued` (lecture côté client) puis de relancer avant de conclure sur la fermeture de session.

Cleanup : aucun processus résiduel (`i148`/bridget), `/tmp/i148-*` supprimés par le testkit. Seul artefact déposé : `/tmp/i148_probe.js` (reproduction).
