# Archives techniques de l’adaptateur T3

Document d’origine : [specs/148-identite-delegation/validation/glm-tests-identite-r3.md](../../../../../../../specs/148-identite-delegation/validation/glm-tests-identite-r3.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Commande | RC | Résultat |
|---|---|---|
| `cargo build -p bridget-daemon --bin bridget` (WT Bridget) | 0 | Binaire frais 06:15 |
| Interop `BridgetRustInterop.test.ts` (réseau réel, T3 apps/server) | **1** | **1 failed / 1** — ligne 164, `initialized.length` = 0 |
| Régressions T3 (7 fichiers identité) | 0 | **389 passed, 1 skipped** |
| `cargo test -p bridget-daemon t3code_mcp --lib` (WT Bridget) | 0 | **6 passed / 6** |
| `cargo test -p bridget-daemon mcp_identity --lib` (WT Bridget) | 0 | **15 passed / 15** |

<a id="extrait-2"></a>

## Extrait 2

| Élément | Avant (06:15) | Après (06:40) |
|---|---|---|
| `BridgetRustInterop.test.ts` | `da9dab7d…4a28` | identique |
| `BridgetRustInterop.testkit.ts` | `192dfcad…0a91` | identique |
| `t3code_mcp.rs` / `mcp_identity.rs` / `mcp.rs` / `daemon.rs` | `793fd54a…` / `a1e3add7…` / `bd7919cb…` / `5d90983f…` | identiques |
| Binaire `target/debug/bridget` | `e7aeff86…0013` (mon build 06:15) | `a60f4e36…8b1e` (recompilé par l'autre agent à 06:24) |

<a id="extrait-3"></a>

## Extrait 3

- Régressions T3 : **PASS** (389/389 exécutés).
- Rust ciblé : **PASS** (21/21).
- Interop : **ÉCHEC sur un défaut de mesure précis et localisé**. La correction R1 est fonctionnelle sur le réseau (header émis, renvoyé, DELETE 204). La partie aval (404/401/B valide) reste **sans preuve d'exécution**.
