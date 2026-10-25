# Archives techniques de l’adaptateur T3

Document d’origine : [specs/148-identite-delegation/validation/glm-tests-identite-r2.md](../../../../../../../specs/148-identite-delegation/validation/glm-tests-identite-r2.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Commande | RC | Résultat |
|---|---|---|
| Interop `BridgetRustInterop.test.ts` (réseau réel) | **1** | **1 failed / 1** — échec ligne 138 |
| Régressions T3 (7 fichiers) | 0 | **389 passed, 1 skipped** |
| `cargo test t3code_mcp --lib` | 0 | **6 passed / 6** |
| `cargo test mcp_identity --lib` | 0 | **15 passed / 15** |

<a id="extrait-2"></a>

## Extrait 2

Le skip T3 est explicite et attendu : `ClaudeMcp.test.ts > SPEC147 … metadata proof without a model` est un `it.live.skip` conditionné à `BRIDGET_147_CLAUDE_METADATA_COMMAND` absente. Ce n'est pas un masquage.

<a id="extrait-3"></a>

## Extrait 3

- **Interop** : vrai transport réseau (NodeHTTP/Effect sur 127.0.0.1, port aléatoire). Vrai daemon Rust (socket unix + credential). Vrai client `bridget mcp`. Fixtures limitées aux données stockées (projection de fils, registres vides). La boucle HTTP complète est réelle.
- **`t3code_mcp` (Rust)** : vrai TCP local (`TcpListener` embarqué), mais serveur factice contrôlé — pas le vrai serveur T3. Identité daemon mockée (`mock_socket`, `mock_private_identity`).
- **`mcp_identity` (Rust)** : tests unitaires de résolution de filiation. Pas de transport réseau.
- **Non couvert par moi** (périmètre de l'autre agent) : bibliothèques Rust complètes, E2E native, test env subprocess.

<a id="extrait-4"></a>

## Extrait 4

- Exécutable : début `09113df6…cc18` (conforme au brief, vérifié avant le run) → fin `a2dab634…1f95b` (recompilé 06:02:53). Mon run interop a bien utilisé `09113df6`.
- Sources T3 : digest `11bc755a…` → `8c491263…`. Modifiés après mon run interop : `ClaudeMcp.ts/.test.ts`, `ClaudeAdapterV2.ts/.test.ts` (05:58-06:00). Mon run de régressions (06:00:09) a exécuté ces versions récentes.
- Sources Rust : digest `509cf2d9…` → `bac3d562…`. Modifiés : `wrapper.rs`, `daemon.rs`, `delegation.rs`, `daemon/native_delegation.rs`. Les modules que j'ai testés (`t3code_mcp.rs` 05:44, `mcp_identity.rs` 05:31) n'ont pas bougé depuis avant l'interop.
- Le test interop lui-même (05:46:11) n'a pas été retouché après mon run. Ma lecture correspond à la version exécutée.

<a id="extrait-5"></a>

## Extrait 5

- Régressions T3 : **PASS** (389/389 exécutés).
- Rust ciblé : **PASS** (21/21).
- Recette interop réseau réel : **ÉCHEC — mais sur un défaut de mesure du test, pas sur le comportement**. La moitié aval de la recette (DELETE, 404, 401, erreur Rust) reste à valider après correction du hook d'observation. Je recommande de faire corriger `issued` (lecture côté client) puis de relancer avant de conclure sur la fermeture de session.
