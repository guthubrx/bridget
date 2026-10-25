# Archives techniques de l’adaptateur T3

Document d’origine : [specs/148-identite-delegation/validation/glm-tests-natifs-r2.md](../../../../../../../specs/148-identite-delegation/validation/glm-tests-natifs-r2.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

| Étape | Résultat | RC |
|---|---|---|
| 1. e2e natif 148 | **PASS** — 1/1 (5,51 s) | 0 |
| 2. identité T3 | **PASS** — 1 passed, 1 ignored (helper volontaire) | 0 |
| 3. workspace `--lib` mono-thread | **PASS — 1496 / 0 / 15** sous ENV complet | 0 |
| 4a. `cargo fmt --check` | **FAIL** — 36 diffs | 101 |
| 4b. `clippy -D warnings` | **FAIL** — 4 erreurs | 1 |
