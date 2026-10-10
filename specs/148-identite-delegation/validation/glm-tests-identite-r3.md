Toutes les données sont réunies. Voici le rapport final.

---

# Rapport R3 — Test indépendant GLM, session 148 identité/délégation

## 1. Résultats par commande

| Commande | RC | Résultat |
|---|---|---|
| `cargo build -p bridget-daemon --bin bridget` (WT Bridget) | 0 | Binaire frais 06:15 |
| Interop `BridgetRustInterop.test.ts` (réseau réel, T3 apps/server) | **1** | **1 failed / 1** — ligne 164, `initialized.length` = 0 |
| Régressions T3 (7 fichiers identité) | 0 | **389 passed, 1 skipped** |
| `cargo test -p bridget-daemon t3code_mcp --lib` (WT Bridget) | 0 | **6 passed / 6** |
| `cargo test -p bridget-daemon mcp_identity --lib` (WT Bridget) | 0 | **15 passed / 15** |

Le skip est explicite : `ClaudeMcp.test.ts:212`, `it.live.skip` gated sur `BRIDGET_147_CLAUDE_METADATA_COMMAND` absente. Identique à R2. Pas un masquage.

Note : le RC=0 affiché au premier run interop venait de mon pipe zsh (`PIPESTATUS`). Les runs suivants, redirigés vers fichier, donnent RC=1. Le RC réel est 1.

## 2. Cause racine de l'échec interop — prouvée par données runtime

R2 avait raison sur le défaut de mesure. R2 avait tort sur le mécanisme exact. Voici la chaîne prouvée :

1. Effect 4.0.1 appelle `nodeResponse.writeHead(response.status, response.statusText, headers)` (`NodeHttpServer.js:342`). `statusText` vaut `undefined` par défaut. L'appel réel est donc `writeHead(status, undefined, headers)`.
2. Sonde prototype + capture `_writeRaw` (`/tmp/i148_hook.cjs`, journal `/tmp/i148_wire.ndjson`) : 8 réponses servies. Le bloc d'en-têtes sérialisé contient les headers applicatifs — `mcp-protocol-version: 2025-06-18` vu sur le fil des deux 202. Aucun appel public `setHeader`/`appendHeader`.
3. Micro-test Node v26.9.0 : `res.writeHead(200, undefined, {...})` → le client `fetch` **reçoit bien** les headers du 3e argument.
4. Le parseur du testkit (`BridgetRustInterop.testkit.ts:25`) fait `typeof statusOrHeaders === "string" ? headers : statusOrHeaders`. Quand la raison est `undefined`, il sélectionne `statusOrHeaders` (= `undefined`) et **ignore le 3e argument**. `issued` reste indéfini. `initialized.length` = 0.

Le header va sur le fil. L'observateur du test est aveugle sur cette seule forme d'appel. Ma propre sonde a eu le même défaut de parseur avant correction — ce qui l'a révélé.

## 3. Preuves obtenues sur le fil (runs réels)

- 2 POST `initialize` sans session → 200.
- **4 POST avec `mcp-session-id`** (le client Rust a bien reçu le header) → 202 et 200. Chaque `bridget_who` répond avec l'agentId du daemon réel.
- **2 DELETE → 204** : le terminateSession du patch effect fonctionne. Sessions bien fermées.
- A/B distincts prouvés avant la ligne d'échec : `providerSessionId` ≠, `authorizationHeader` ≠, `agentA` ≠ `agentB`, même endpoint et même `providerInstanceId`. Le daemon a émis deux credentials séparés.
- **NON atteints** (le test meurt ligne 164) : réemploi de session périmée → 404 attendu ; révocation A → 401 + `t3_session_unavailable` côté Rust ; B valide après révocation de A ; 3e DELETE. Ces assertions **n'ont pas tourné**.

## 4. Empreintes et mouvements de l'environnement

| Élément | Avant (06:15) | Après (06:40) |
|---|---|---|
| `BridgetRustInterop.test.ts` | `da9dab7d…4a28` | identique |
| `BridgetRustInterop.testkit.ts` | `192dfcad…0a91` | identique |
| `t3code_mcp.rs` / `mcp_identity.rs` / `mcp.rs` / `daemon.rs` | `793fd54a…` / `a1e3add7…` / `bd7919cb…` / `5d90983f…` | identiques |
| Binaire `target/debug/bridget` | `e7aeff86…0013` (mon build 06:15) | `a60f4e36…8b1e` (recompilé par l'autre agent à 06:24) |

L'environnement a encore bougé : recompilation du binaire à 06:24, pendant mes runs. Runs 06:16–~06:24 sur mon binaire ; runs ~06:25–06:26 probablement sur le sien. Comportement identique (même échec, même fil). Aucun processus fixture résiduel ; `/tmp/i148-*` nettoyés par le testkit.

## 5. Verdict

- Régressions T3 : **PASS** (389/389 exécutés).
- Rust ciblé : **PASS** (21/21).
- Interop : **ÉCHEC sur un défaut de mesure précis et localisé**. La correction R1 est fonctionnelle sur le réseau (header émis, renvoyé, DELETE 204). La partie aval (404/401/B valide) reste **sans preuve d'exécution**.

## 6. Correctif recommandé (je ne fixe pas — lecture seule)

Dans `observeWrittenMcpSession`, traiter la forme `(status, undefined, headers)` :

```ts
const outgoing =
  typeof statusOrHeaders === "string" || statusOrHeaders === undefined
    ? headers
    : statusOrHeaders;
```

Relancer ensuite la recette pour valider l'aval.

## 7. Artéfacts conservés

`/tmp/i148_hook.cjs` (sonde, Authorization masquée), `/tmp/i148_wire.ndjson` (journal wire brut), `/tmp/i148_run2-7.log`, `/tmp/i148_regress.log`, `/tmp/i148_wt_*.log`. Aucun secret dans les sorties.

Conformément au brief : aucun processus fournisseur réel n'a été lancé. La preuve app-server partagé reste du ressort des tests adapters, couverts par les régressions. Durée : ~25 min.
