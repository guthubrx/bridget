# Archives techniques de l’adaptateur T3

Document d’origine : [specs/149-sous-agents-lineage/validation/prelivraison149.md](../../../../../../../specs/149-sous-agents-lineage/validation/prelivraison149.md).

Ces extraits conservent les faits, contrats, limites et références techniques
datés de l’intégration. Ils ne définissent pas les parcours du produit.
Les liens de navigation sont adaptés au dossier d’archive. Les chemins des
exemples techniques gardent leur valeur d’origine. Les comptes rendus ne prouvent pas le runtime actuel.

<a id="extrait-1"></a>

## Extrait 1

- **Binaire candidat :** release `abc850858975`, sha256 `abc850858975fb5c7d733eb052be4fff6e3cb24537ce8b7d4475019b79abf6d8`.
- **Production revue :** 111 fichiers, empreinte `b2b87458cf3cec7989debeb91352b838c417c3eb48329d57099bec4c9a0d5f29`. Revue des sources : SOURCE_ONLY_APPROVE (`validation/native-restart-final-source-review-sonnet-r2.md`).
- **Recettes réelles sans nouveau grant :** GLM `glm-5.3-flash` et Codex `gpt-6.1-sol` (effort high) (`validation/native-real-recipes-sonnet-r5.md`). Smoke r6 : `validation/native-real-smoke-sonnet-r6.md`. Redémarrage avec parent Codex vivant : `validation/native-real-restart-sonnet-r7.md`, 29 oracles OK, 0 échec.
- **Réseau et interop :** `validation/native-network-proofs-r3.md` (T036 fermé). `validation/native-network-proofs-r4.md` : O4.6, E2 et F3 fermés, 16 contrôles OK, 0 FAIL.
- **Interface :** `validation/ui-native-recipe149-sonnet-r1.md`, avec daemon réel et fournisseurs fictifs explicites.
- **Frontière T3 :** `validation/t3-final-boundary-tests-sonnet-r2.md` (G3 et G5 fermés).

<a id="extrait-2"></a>

## Extrait 2

| Contrôle | Résultat |
|---|---|
| Rust natif (r9) | 1840 PASS = 1507 daemon + 333 transport, 0 FAIL final. 19 tests nouveaux (17 + 2). |
| Rust, charge | 3 échecs initiaux sous charge, puis 3 PASS isolés. |
| SC005 (bench ancien) | Non concluant. Ce bench ne mesure pas l'opt-out SPEC149. |
| Recompte r8 | 1822, et non 1825. |
| `tsc` T3 | Serveur 16 erreurs, web 10 erreurs, 0 nouvelle. |
| Lint T3 | 0 erreur, 908 avertissements dont 7 nouveaux non bloquants. |
| Format T3 | 48 PASS. |
| Interface | 25 PASS sur 4 fichiers (décision du principal). La revue T041 comptait 24 sur les mêmes fichiers. Écart non tranché. |
| Frontière T3 (r2) | 262 tests, 16 fichiers. |

<a id="extrait-3"></a>

## Extrait 3

Ces chiffres ne prouvent pas un GREEN global exhaustif. La suite T3 complète n'est pas promise.
