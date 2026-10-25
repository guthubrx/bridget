Validation terminée (~20 min). Voici le rapport complet.

---

# Rapport R2 — Validation indépendante, session 148 identité/délégation

## 1. Résultats par commande

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-tests-identite-r2.md#extrait-1).

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-tests-identite-r2.md#extrait-2).

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

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-tests-identite-r2.md#extrait-3).

## 4. Mouvements détectés pendant la validation

L'environnement n'était **pas figé**. Un autre agent travaillait dans les mêmes worktrees.

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-tests-identite-r2.md#extrait-4).

## 5. Verdict

[Note technique de l’adaptateur](../../../docs/adapters/t3/archives/specs/148-identite-delegation/validation/glm-tests-identite-r2.md#extrait-5).

Cleanup : aucun processus résiduel (`i148`/bridget), `/tmp/i148-*` supprimés par le testkit. Seul artefact déposé : `/tmp/i148_probe.js` (reproduction).
