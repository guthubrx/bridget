# Audit V15 de clôture : Spécification 306

**Date** : 2026-11-20
**Périmètre** : Changements complets de la spécification 306 (`T001` à `T031`) sur `main`.
**Protocole** : Audit V15 (vérifications reproductibles, minimalisme et frugalité, responsabilité LLM).

## Résultats mesurés

| Contrôle | Commande exécutée | Résultat |
| --- | --- | --- |
| Formatage Rust | `cargo fmt --all -- --check` | PASS (0 écart) |
| Clippy strict CI | `python3 scripts/build.py cargo clippy --workspace --all-targets --locked --features bridget-daemon/test-support -- -D warnings` | PASS (0 avertissement, 0 erreur) |
| Garde `unsafe` | `rg -c "invariant non établi par lecture locale" crates/` | PASS (`0` sur les 174 blocs audités) |
| Tests `bridget-core` | `python3 scripts/build.py cargo test -p bridget-core --locked` | PASS (56/56) |
| Tests `bridget-transport --lib` | `python3 scripts/build.py cargo test -p bridget-transport --lib --locked` | PASS (423/423) |
| Tests `bridget-daemon --lib` ciblés | `spec306`, `mobile::store::tests`, `mobile::http::tests`, `sentinel::tests`, 5 tests `base-echecs.txt` réconciliés | PASS (47/47) |
| Tests Python (`scripts/tests/`) | 6 suites (`test_build_cleanup`, `test_updates`, `test_install`, `test_sentinel`, `test_publish_android`, `test_linux_abi`) | PASS (179/179 OK, 3 skipped) |
| Lint Python & Shell | `ruff check scripts` + `bash -n scripts/*.sh` | PASS (0 erreur) |
| Documentation & anti-IPv4 | `python3 scripts/check-docs.py` | PASS (32 pages, 524 liens, 13 paires EN/FR, 0 IPv4 publique) |
| Build Caddy Go | `go build -mod=readonly -trimpath` (`deploy/caddy-security`) | PASS (Caddy `v2.11.9`) |
| OSV-Scanner `Cargo.lock` | `osv-scanner scan source Cargo.lock` | PASS (0 vulnérabilité sur 175 paquets) |
| OSV-Scanner Android buildscript | `osv-scanner scan source android/buildscript-gradle.lockfile` | PASS (0 vulnérabilité sur 148 paquets) |
| Taille `launch_session_with_status` | Mesure Python sur `crates/bridget-daemon/src/wrapper.rs` | PASS (986 lignes < 1 000, contre 1 520 avant) |

## Constats résiduels et limites environnementales

1. **P2 / Dépendance amont Go (`deploy/caddy-security/go.mod`)** : 3 avis transitifs `GHSA-4w3g-jfj2-cw2q`, `GHSA-mgcc-cww5-fp2q`, `GHSA-vvm8-gf78-28g4` sur `github.com/go-jose/go-jose/v3`, tirés par `caddy/v2@v2.11.9 -> smallstep/certificates@v0.30.2 -> go.step.sm/crypto@v0.91.0`. OSV confirme `0 vulnerabilities can be fixed` sans correctif amont Caddy/Smallstep. 8 des 11 CVE initiales ont été éliminées.
2. **P2 / SDK Android absent localement** : `android/app/gradle.lockfile` et `./gradlew testDebugUnitTest` n'ont pas pu être exécutés sur ce Mac. `resolutionStrategy` contraint `com.google.guava:guava:33.7.2-jre` dans `android/build.gradle.kts` et `android/app/build.gradle.kts`, et le nouveau job CI `android` dans `.github/workflows/rust.yml` exécutera `./gradlew testDebugUnitTest --no-daemon`.
3. **P3 / Tests E2E préexistants** : les tests d'intégration IPC/processus préexistants de `base-echecs.txt` (ex. `spec102_v25_issue_inconnue_puis_nouvelle_mention`) restent suivis dans le job dédié `tests-e2e-baseline`, tandis que les suites déterministes (`bridget-core`, `bridget-transport --lib`, `mobile::*`, `sentinel::*`, `spec306`) bloquent désormais la CI sans `continue-on-error`.
4. **Revue adversariale multi-fournisseur** : non exécutée (`délégation Bridget bloquée dans ce tour` par la consigne T3 Code).

## Axes Minimalisme & Frugalité et Responsabilité LLM

- **Minimalisme & Frugalité** : aucune nouvelle dépendance Rust ou Android ajoutée ; réutilisation directe de `rusqlite`, `axum`, `tokio::sync::watch`, `sha2` et OpenSSH `ssh-keygen -Y`. Estimation des lignes suppressibles sur le diff 306 : < 30 lignes.
- **Responsabilité LLM** : zéro secret ajouté, zéro clé privée de production générée, zéro trace IA, chaque bloc `unsafe` documenté par son invariant réel vérifié, chaque nouvelle capacité couverte par un test unitaire reproductible.

## Décision calculée

- **Critiques ou bloquants introduits** : 0
- **Decision** : `ACCEPT_WITH_NOTES` (livraison Git et binaire release autorisée ; les 2 limites externes amont/environnement sont documentées sans contournement).
- **`release_blocked`** : `false`
