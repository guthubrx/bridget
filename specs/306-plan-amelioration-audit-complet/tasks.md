# Tâches : Programme complet d'amélioration post-audit V15 (306)

**Spécification** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/306-plan-amelioration-audit-complet/spec.md`
**Plan** : `/Users/moi/Nextcloud/10.Scripts/64.bridget/specs/306-plan-amelioration-audit-complet/plan.md`
**Branche** : `main`

---

## Phase 1 : Vague 1 - Sécurité, hygiène immédiate et `cargo fmt` (US1)

Objectif : zéro CVE OSV corrigeable, IP publique masquée + garde CI, socket Unix bornée, sécurité IA renforcée, signature détachée `ssh-keygen -Y` testée, et 0 fichier hors `cargo fmt`.

- [x] T001 [P] [US1] Mettre à jour et alléger `deploy/caddy-security/go.mod`, `deploy/caddy-security/go.sum` et `deploy/caddy-security/main.go` (Caddy `v2.11.9`, 8/11 CVE résolues ; 3 avis transitifs `go-jose/v3` sans correctif amont documentés) et vérifier `go build -mod=readonly -trimpath` + `osv-scanner`.
- [x] T002 [P] [US1] Assainir `Cargo.lock` (`pulldown-cmark` 0.13.6, 0 CVE OSV) et ajouter la contrainte `resolutionStrategy` pour `com.google.guava:guava:33.7.2-jre` dans `android/build.gradle.kts` et `android/app/build.gradle.kts` (+ régénérer `android/buildscript-gradle.lockfile`, 0 CVE OSV ; `android/app/gradle.lockfile` reporté à la CI faute de SDK Android local).
- [x] T003 [P] [US1] Masquer l'adresse IPv4 publique dans `specs/233-placement-opus2d-mac/validation.md` par une plage RFC 5737 (`198.51.100.10:2222`) et ajouter un contrôle anti-IP-publique dans `scripts/check-docs.py`, vérifier `python3 scripts/check-docs.py`.
- [x] T004 [US1] Appliquer `cargo fmt --all` sur les fichiers Rust en dérive et vérifier `cargo fmt --all -- --check` à 0 écart.
- [x] T005 [P] [US1] Ajouter un plafond de connexions simultanées avec réserve workers/admin (`UnixConnectionBudget` 256 + 32) dans `crates/bridget-daemon/src/daemon/metrics.rs` et `daemon.rs` + test unitaire `spec306_unix_connection_budget_*`.
- [x] T006 [P] [US1] Ajouter des délimiteurs uniques par message dans `crates/bridget-core/src/envelope.rs` et un filtre préventif configurable de secrets dans `crates/bridget-core/src/secret_filter.rs` branché sur `store/ledger_requests.rs`, avec tests unitaires dédiés (`bridget-core` 56/56 PASS).
- [x] T007 [US1] Ajouter la sous-commande native `bridget federation links --json` dans `crates/bridget-daemon/src/cli.rs` et `crates/bridget-daemon/src/sentinel.rs`, et supprimer `exec(BRIDGET_SENTINEL_PREPARE_SOURCE)` dans `scripts/sentinel.py` ; vérifier `test_sentinel.py` et `spec306_run_federation_links_*`.
- [x] T008 [P] [US1] Implémenter `scripts/sign-release.sh` et la vérification de signature détachée `ssh-keygen -Y` dans `scripts/install.sh` (avec couverture valide/absente/corrompue/clé inconnue dans `scripts/tests/test_install.py`) ; vérifier `test_install.py`.

## Phase 2 : Vague 2 - Performance, complexité et contention (US2)

Objectif : supprimer les 5 violations Article XVIII confirmées et fractionner la contention de `DaemonState`.

- [x] T009 [US2] Remplacer `thread::sleep(50 ms)` sur `WouldBlock` par `poll(2)` (`bridget_transport::posix::poll_fds`) + pipe d'éveil de shutdown (`ShutdownWakePipe`) dans `crates/bridget-daemon/src/daemon/metrics.rs` et `daemon.rs`, vérifié par `spec306_shutdown_wake_pipe_wakes_poll_immediately`.
- [x] T010 [P] [US2] Mettre en cache `AgentRegistry::load*` par `(mtime, size, inode, mode, uid)` dans `crates/bridget-daemon/src/registry.rs`, vérifié par `spec306_registry_cache_hits_and_invalidates_on_change`.
- [x] T011 [P] [US2] Passer `EnvelopeGuard` (`envelope.rs`), `Deduplicator` (`dedup.rs`, clé `(content_key, target)`) et `CircuitBreaker` (`circuit_breaker.rs`, purge des clés vides) en O(1) amorti borné avec `checked_sub` et tests unitaires complets (`bridget-core` 56/56 PASS).
- [x] T012 [US2] Regrouper les requêtes SQL N+1 de `native_delegation::tick` (`failed_for_owners` et `for_missions` en `IN (...)`) dans `crates/bridget-daemon/src/delegation.rs` et `crates/bridget-daemon/src/daemon/native_delegation.rs`.
- [x] T013 [P] [US2] Activer `PRAGMA journal_mode=WAL` + `synchronous=NORMAL` dans `mobile/store.rs`, réutiliser un seul `Store` par rafale dans `mobile/notifications.rs`, remplacer le thread OS + polling 1 s de `mobile/http.rs::follow()` par une tâche Tokio + `watch` (`auth_rev`), et partager `LocalStore` dans `android/app/src/main/java/app/cartae/bridget/MobileUpdates.kt`.
- [x] T014 [US2] Introduire des verrous lecture/écriture (`RwLock`) sur la vue annuaire (`ReadMostlyDirectory`) et la fenêtre de latences `native_tick_latencies_ms` dans `crates/bridget-daemon/src/daemon/metrics.rs` et `daemon/native_delegation.rs`.

## Phase 3 : Vague 3 - Audit `unsafe`, correction des tests et CI bloquante (US3)

Objectif : 0 phrase `SAFETY` de repli, suites Python et Rust corrigées, CI bloquante sur Rust, Python, Android et Go Caddy.

- [x] T015 [US3] Encapsuler les appels POSIX récurrents (`fcntl`, `cloexec`, `poll`, `flock`, `pipe`) dans `crates/bridget-transport/src/posix.rs` et auditer les 174 blocs `unsafe` dans `crates/` afin de remplacer chaque phrase de repli par l'invariant vérifié (`rg` = 0 occurrence).
- [x] T016 [P] [US3] Corriger la fuite Git parente (`GIT_CEILING_DIRECTORIES`) dans `scripts/build.py` résolvant les 3 échecs de `test_build_cleanup.py`, et vérifier 100 % PASS sur les 6 suites Python (`179 tests, OK (skipped=3)`).
- [x] T017 [US3] Diagnostiquer la base d'échecs `specs/260-assainissement-code/validation/base-echecs.txt` et réconcilier 5 tests unitaires dérivés des commits `6807571d` et `45de7467` (`protocol.rs`, `native_permissions149_tests.rs`, `remote_worker_tests.rs`, `daemon_presence_tests.rs`, `native_delegation_tests.rs`) ; documenter les E2E préexistants dans un job CI dédié.
- [x] T018 [US3] Mettre à jour `.github/workflows/rust.yml` et `.github/workflows/rust-audit.yml` : rendre bloquants `tests` (core/transport/mobile/spec306), `scripts` et `rust-audit`, ajouter la garde anti-phrase-repli `unsafe`, et ajouter les jobs CI `caddy-go` et `android`.

## Phase 4 : Vague 4 - Maintenabilité Rust, unification SQLite, typage et minimalisme (US4)

Objectif : réduire la charge cognitive des 5 monolithes, adopter `Axum`, unifier SQLite, typer les erreurs prioritaires et nettoyer la dette de lints/nommage.

- [x] T019 [US4] Découper `launch_session_with_status` sous 1 000 lignes (de 1 520 à 986 lignes via 11 helpers extraits) dans `crates/bridget-daemon/src/wrapper.rs` sans régression.
- [x] T020 [US4] Extraire des sous-modules thématiques de `daemon.rs` (`daemon/metrics.rs`), `daemon/native_delegation.rs` (`daemon/native_routing_types.rs`), `remote_worker.rs` (`remote_worker_types.rs`) et `protocol.rs` (`project_registry_protocol.rs`) tout en préservant les exports publics.
- [x] T021 [US4] Refactoriser `crates/bridget-daemon/src/mobile/http.rs` vers des routes déclaratives `axum::Router` (`build_router`) en conservant l'intégralité des contrats HTTP et tests existants.
- [x] T022 [P] [US4] Appliquer l'ADR `docs/decisions/260-stockage-versions-schema.md` : créer `open_private_wal` et `inspect_store_schema` dans `crates/bridget-daemon/src/store_schema.rs`, brancher `open_private_wal` dans `mobile/store.rs`, et couvrir par `spec306_open_private_wal_and_inspect_schema_cover_all_stores`.
- [x] T023 [US4] Typer l'erreur inter-projets (`ProjectScopeError` / `project_scope_typed` dans `communication.rs` + `daemon.rs`), documenter T806b avec `#[expect(dead_code, reason = ... D12)]` dans `attach.rs`, et renommer les 5 symboles suffixés par numéro de spec.

## Phase 5 : Vague 5 - Modularisation et évolutions Android (US5)

Objectif : code Android découpé et typé + 4 évolutions utilisateur mobiles couvertes par tests unitaires.

- [x] T024 [US5] Créer les modèles Kotlin typés (`MobileModels.kt`), les contrôleurs de domaine (`MobileControllers.kt`), les composants d'écran (`MobileScreens.kt`) et les tests unitaires (`MobileModelsAndFiltersTest.kt`) dans `android/app/src/{main,test}/java/app/cartae/bridget/`, branchés dans `BridgetClient.kt` et `MobileViewModel.kt`.
- [x] T025 [US5] Ajouter côté passerelle mobile (`crates/bridget-daemon/src/mobile/http_opt_in.rs`, `http.rs`, `client.rs`) et Android les routes `POST /v1/agents/{id}/relaunch`, `POST /v1/specialists/{id}/launch` (opt-in explicite), `GET /v1/ledger/search` et le filtrage de `PUT /v1/notifications` par projet et urgence, avec tests Rust (`spec306_mobile_notification_filters_and_ledger_search_roundtrip`).

## Phase 6 : Vague 6 - Nouvelles fonctionnalités d'exploitation (US6)

Objectif : livrer les 6 nouvelles capacités opérateur/utilisateur avec couverture de tests et documentation alignée.

- [x] T026 [P] [US6] Exposer `daemon::Metrics` + quantiles `native_tick` dans `bridget status --metrics`, créer `bridget doctor [--json]` (`crates/bridget-daemon/src/doctor.rs`) et supporter `BRIDGET_LOG_FORMAT=json` dans `crates/bridget-daemon/src/main.rs`, vérifiés par `spec306_doctor_reports_private_wal_namespace_and_metrics`.
- [x] T027 [P] [US6] Implémenter `bridget backup` et `bridget restore --verify` (`crates/bridget-daemon/src/backup.rs`) avec copie SQLite cohérente à chaud (`VACUUM INTO`), manifeste SHA-256 et vérification en bac à sable, vérifiés par `spec306_hot_backup_and_sandbox_restore_verify_roundtrip_and_detect_corruption`.
- [x] T028 [P] [US6] Exposer le signal `background_liveness` et activer le mode nettoyage Phase 1 opt-in dans `crates/bridget-daemon/src/reaper.rs`, vérifiés par `spec306_reaper_background_liveness_protects_active_and_phase1_opt_in_cleans_only_eligible_tempdir`.
- [x] T029 [P] [US6] Implémenter l'option `--dry-run` sur `bridget delegate` et `bridget auto-delegate` (`crates/bridget-daemon/src/cli.rs`), vérifiée par `spec306_dry_run_plan_has_no_side_effects`.
- [x] T030 [P] [US6] Implémenter `bridget completions <bash|zsh|fish>` (`crates/bridget-daemon/src/completions.rs`) et unifier/vérifier les schémas MCP (`crates/bridget-daemon/src/mcp_scopes.rs`), vérifiés par `spec306_completions_*` et `spec306_mcp_tool_schemas_*`.

## Phase 7 : Vérification transversale, convergence, audit V15 et livraison

- [x] T031 Mettre à jour la documentation EN/FR (`reference-communication`, `federation-services`, `mobile`) et vérifier `check-docs.py`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked --features bridget-daemon/test-support -- -D warnings`, `ruff check scripts`, `bash -n scripts/*.sh` et `osv-scanner`.
- [x] T032 Exécuter Phase 6 (Converge), Phase 7 (Audit V15), Phase 8 (commits propres sur `main`, push vérifié sur `github/main`, build release vérifié) et Phase 9 (ménage `menage.py` + `scripts/build.py`).
