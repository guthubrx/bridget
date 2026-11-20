# Mesures et résultats d'implémentation : Spécification 306

**Date** : 2026-11-20
**Branche** : `main`
**Spécification** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/306-plan-amelioration-audit-complet/spec.md`

## Mesures clés

- **OSV-Scanner** :
  - `Cargo.lock` : 1 -> 0 vulnérabilité (`pulldown-cmark` 0.13.4 -> 0.13.6).
  - `android/buildscript-gradle.lockfile` : 1 -> 0 vulnérabilité (`com.google.guava:guava` contraint à `33.7.2-jre`).
  - `deploy/caddy-security/go.mod` : 11 -> 3 avis transitifs `go-jose/v3` sans correctif amont sous Caddy `v2.11.9` (`0 vulnerabilities can be fixed`).
- **Sûreté `unsafe`** : 174 -> 0 occurrence de la phrase de repli `invariant non établi par lecture locale` dans `crates/`, garde ajoutée en CI.
- **Maintenabilité Rust** :
  - `launch_session_with_status` (`crates/bridget-daemon/src/wrapper.rs`) : 1 520 -> 986 lignes (< 1 000 lignes, SC-006 atteint).
  - 4 sous-modules extraits : `daemon/metrics.rs`, `daemon/native_routing_types.rs`, `remote_worker_types.rs`, `crates/bridget-transport/src/project_registry_protocol.rs`.
  - Passerelle mobile (`crates/bridget-daemon/src/mobile/http.rs`) migrée vers `axum::Router` + nouvelles routes `http_opt_in.rs`.
- **Tests et build** :
  - `cargo fmt --all -- --check` : PASS (0 écart).
  - `cargo clippy --workspace --all-targets --locked --features bridget-daemon/test-support -- -D warnings` : PASS (0 avertissement).
  - `bridget-core` : 56/56 PASS.
  - `bridget-transport --lib` : 423/423 PASS.
  - `bridget-daemon --lib` (`spec306`, `mobile::*`, `sentinel::*`, 5 tests `base-echecs.txt` réconciliés) : 47/47 PASS.
  - 6 suites Python (`scripts/tests/`) : 179/179 OK (3 skipped).
  - `python3 scripts/check-docs.py` : PASS (32 pages, 524 liens, 13 paires EN/FR, 0 IPv4 publique).
  - `ruff check scripts` + `bash -n scripts/*.sh` : PASS.
  - Build release `target/release/bridget` : PASS (compilé et vérifié en exécution locale).

## Écarts et limites documentées

1. **Android SDK absent sur ce poste macOS** : `android/app/gradle.lockfile` et `./gradlew testDebugUnitTest` ne peuvent pas tourner localement ; couverts par le nouveau job CI `android` dans `.github/workflows/rust.yml`.
2. **Caddy `go-jose/v3`** : 3 avis transitifs sans correctif amont tant que `smallstep/certificates` dépend de `go-jose/v3`.
3. **Activation runtime effectuée et vérifiée le 20 novembre 2026** :
   - **Binaire macOS ARM64** (`BRIDGET_BUILD_ID=306-audit-complet-r1`, SHA-256 `c0da38551c693671cbefbaec9dbb08fa5686b2921297622ae525c89283af91cc`) installé dans `/Users/moi/.local/lib/bridget-communication/306-audit-complet-r1/bin/bridget`, lié atomiquement depuis `/Users/moi/.local/bin/bridget`. Services `com.bridget.worker.monordinateur` et `com.bridget.updates` redémarrés ; `com.bridget.t3` (PID `7450`) et `/Users/moi/.cache/bridget-core/worker-monordinateur-agents.json` préservés sans interruption.
   - **Binaire Linux commun Debian 12 / glibc 2.36** (`BRIDGET_BUILD_ID=306-audit-complet-r1`, SHA-256 `6b96fc884dc89d67f2435057ee921f92a152c82b85236dc9d8c9b5b54be22768`, qualifié `{"status": "compatible", "required_glibc": "2.34", "maximum_glibc": "2.36"}` par `scripts/check-linux-abi.py`) compilé sur `sysle2` puis distribué et activé sur `cartae`, `sysle2` et `ksstore`.
   - **Compatibilité stockage bidirectionnelle** (`formats report` / `formats validate-report`) vérifiée `{"valid": true}` avant bascule sur les 4 machines (nouvelle image et image de retour).
   - **Pointeurs `rollback`** dans `update-host-state.json` mis à jour atomiquement sous verrou natif `update-host.lock` avec sauvegarde `.bak-306` : `288-preparation-guidage-r1` sur `monordinateur`, `cartae` et `ksstore`, `287-role-specialistes-r1` sur `sysle2`.
   - **Ménage post-livraison** : `/tmp/bridget-build-306` supprimé sur `sysle2`, 34 anciennes copies de binaires non utilisées retirées dans `~/.local/lib/bridget-communication/` (13 sur `cartae`, 7 sur `sysle2`, 2 sur `ksstore`, 12 sur `monordinateur`), version active + 1 rollback vérifié conservés partout. Caches `/Users/moi/.cache/bridget-build-*` conservés sur Mac par garde `scripts/build.py` (observation incomplète du processus PID `92290`). `macbook-m1` (`192.168.1.51`) hors ligne au moment du déploiement.
