# Plan d'implémentation : Programme complet d'amélioration post-audit V15 (306)

**Branche** : `main` (décision branche unique du 10 novembre 2026) | **Date** : 2026-11-20
**Spécification** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/306-plan-amelioration-audit-complet/spec.md`

## Contexte technique

- **Langages et chaînes** : Rust édition 2024 (workspace 3 crates : `bridget-core`, `bridget-transport`, `bridget-daemon`), Go 1.27.3 (`deploy/caddy-security`), Python 3 (`scripts/`), Kotlin / Jetpack Compose (`android/`).
- **Stockage** : SQLite (`rusqlite 0.41` bundled) pour `bridget.db`, `mobile-devices.db`, délégation, profils, idempotence ; fichiers JSON atomiques (`agents.json`, `mobile.json`).
- **Tests et contrôles** : `python3 scripts/build.py cargo test/clippy`, `cargo fmt --all -- --check`, `python3 -m unittest`, `ruff check scripts`, `python3 scripts/check-docs.py`, `osv-scanner`, `./gradlew testDebugUnitTest`.
- **Contraintes projet** : jamais appeler `cargo` directement (toujours via `python3 scripts/build.py cargo ...`), jamais arrêter/redémarrer T3 sans confirmation explicite, aucune trace IA dans Git, travail direct sur `main`.

## Contrôle de constitution et Articles XIX / XX

- **Article XV (Tester avant de poursuivre)** : chaque vague est compilée et testée avant de passer à la suivante ; chaque vague donne un commit cohérent vérifié sur `main`.
- **Article XVIII (Complexité)** : la Vague 2 élimine les 5 violations confirmées (polling 50 ms, N+1 SQL, relectures disque `AgentRegistry`, scans O(n) `bridget-core`, ouverture SQLite par appareil).
- **Article XIX (Minimalisme)** : 0 nouvelle dépendance Cargo/Android ; réutilisation d'`axum`, `tokio`, `rusqlite`, `ssh-keygen -Y`, et des modules existants. Potentiel de suppression estimé ~1 800 à 3 200 lignes en Vague 4.
- **Article XX (Responsabilité future)** : audit réel des 174 blocs `unsafe`, découpage des 5 monolithes Rust et des 2 monolithes Android, documentation des contrats dans `contracts/`.

## Architecture en 6 vagues ordonnées

### Vague 1 - Sécurité, hygiène immédiate et `cargo fmt` (US1, FR-001 à FR-006)
- `deploy/caddy-security/go.mod`, `go.sum`, `main.go` : mise à jour Go/Caddy, modules minimaux, vérification `go build` et `osv-scanner`.
- `Cargo.lock` (`pulldown-cmark`) et `android/app/build.gradle.kts` (`guava` `resolutionStrategy`).
- `specs/233-placement-opus2d-mac/validation.md` (masquage IP RFC 5737) + `scripts/check-docs.py` (détection IPv4 publiques hors RFC 5737).
- `crates/bridget-daemon/src/daemon.rs` : plafond de connexions Unix (256 générales + 32 réservées workers/admin).
- `crates/bridget-core/src/envelope.rs` : délimiteur unique par message anti-usurpation.
- `crates/bridget-daemon/src/main.rs` + `crates/bridget-daemon/src/sentinel.rs` + `scripts/sentinel.py` : sous-commande `bridget federation links --json` remplaçant `exec()`.
- `crates/bridget-core/src/secret_filter.rs` (intégré à la persistance des briefs/messages) : filtre préventif configurable de secrets.
- `scripts/install.sh` + `scripts/sign-release.sh` + `scripts/tests/test_install.py` : vérification et signature détachée `ssh-keygen -Y`.
- `cargo fmt --all` sur les 18 fichiers en dérive.

### Vague 2 - Performance, complexité et contention (US2, FR-007 à FR-012)
- `crates/bridget-daemon/src/daemon.rs` : remplacement de `sleep(50 ms)` par `poll(2)` + pipe de shutdown.
- `crates/bridget-daemon/src/registry.rs` : cache mémoire `(path, mtime, size, inode)` pour `AgentRegistry::load*`.
- `crates/bridget-core/src/envelope.rs`, `dedup.rs`, `circuit_breaker.rs` : `VecDeque` O(1) amorti, plafonds de capacité, `checked_sub`, clé `(content_key, target)` dans `Deduplicator`, purge des clés vides dans `CircuitBreaker`.
- `crates/bridget-daemon/src/daemon/native_delegation.rs` + `crates/bridget-daemon/src/delegation_store.rs` : requêtes SQL groupées dans `tick()` (fin N+1).
- `crates/bridget-daemon/src/mobile/store.rs` (`WAL`), `mobile/notifications.rs` (`Store` partagé par rafale), `mobile/http.rs` (invalidation `tokio::sync::watch` dans `follow()`), `android/.../MobileUpdates.kt` (`LocalStore` partagé).
- `crates/bridget-daemon/src/daemon.rs` : extraction des structures majoritairement lues (`RwLock` pour catalogue/annuaire/présences) hors du `Mutex<DaemonState>` monolithique.

### Vague 3 - Sûreté des blocs `unsafe`, correction des tests et CI bloquante (US3, FR-013 à FR-014)
- `crates/bridget-transport/src/posix.rs` (extension ou création d'un module dédié aux wrappers POSIX sûrs : `fcntl`, `poll`, `cloexec`, `flock`) + audit et justification réelle de l'ensemble des 174 blocs `unsafe` ; retrait complet de la phrase de repli.
- Correction des 3 échecs de `scripts/tests/test_build_cleanup.py` et des 16 échecs macOS de `scripts/tests/test_updates.py` (`/private/var`).
- Diagnostic et correction des échecs Rust préexistants (`base-echecs.txt`).
- `.github/workflows/rust.yml` et `.github/workflows/rust-audit.yml` : retrait de `continue-on-error: true`, ajout du contrôle anti-phrase-repli `unsafe`, ajout des jobs CI Android et Go Caddy (`osv-scanner` + `go build`).

### Vague 4 - Maintenabilité Rust, unification SQLite, typage et minimalisme (US4, FR-015 à FR-017)
- Découpage en sous-modules cohérents de `daemon.rs`, `daemon/native_delegation.rs`, `remote_worker.rs`, `protocol.rs` et `wrapper.rs` (avec `launch_session_with_status` < 1 000 lignes).
- Migration de `mobile/http.rs::execute()` vers des routes déclaratives `axum::Router`.
- Unification SQLite (ADR 260) : ouverture privée commune `WAL` + test de version lisible sur tous les magasins.
- Typage des erreurs comparées par texte (lot 8 / D13), `Box<...>` des variantes volumineuses du protocole et retrait des `allow` résorbés dans `Cargo.toml`, arbitrage documenté/nettoyage T806b dans `attach.rs`, renommage des symboles internes suffixés par numéro de spec.

### Vague 5 - Modularisation et évolutions Android (US5, FR-018 à FR-019)
- Découpage de `MobileViewModel.kt` en contrôleurs/dépôts + classes de données typées (`MobileModels.kt`), et découpage de `MainActivity.kt` par écrans (`DirectoryScreen.kt`, `ConversationScreen.kt`, `SettingsScreen.kt`, `Dialogs.kt`).
- Extension opt-in serveur mobile (`crates/bridget-daemon/src/mobile/`) et client Android : relance encadrée d'un agent arrêté, lancement d'un profil spécialiste approuvé, recherche plein texte mobile et filtrage des notifications par projet/urgence.

### Vague 6 - Nouvelles fonctionnalités d'exploitation (US6, FR-020 à FR-024)
- `bridget status --metrics` et `bridget doctor` (exposant `daemon::Metrics`, quantiles de tick, état des files/workers/mobile) + `BRIDGET_LOG_FORMAT=json`.
- `bridget backup <archive>` et `bridget restore --verify <archive>` (copie cohérente à chaud SQLite + manifeste SHA-256 + contrôle isolé).
- Signal `background_liveness` + `reaper` Phase 1 opt-in.
- Mode `--dry-run` sur `bridget delegate` et `bridget auto-delegate`.
- `bridget completions bash|zsh|fish` et source unique testée des schémas MCP.

## Réutilisation de l'existant

Tous les éléments existants réutilisés (`axum`, `tokio::sync::watch`, `rusqlite`, `daemon::Metrics`, `store_schema.rs`, `reaper.rs`, `scripts/build.py`, `scripts/check-docs.py`, `scripts/install.sh`) sont vérifiés item par item dans `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/306-plan-amelioration-audit-complet/reuse-audit.md`.
