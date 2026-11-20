# Audit de reutilisation de l'existant - 306-plan-amelioration-audit-complet

## Decision

Statut: PASS
Date: 2026-11-20
Feature dir: /Users/moi/Nextcloud/10.Scripts/64.bridget/specs/306-plan-amelioration-audit-complet

Conclusion courte: Tous les items du plan 306 etendent les composants, bibliotheques et contrats deja presents dans Bridget (`axum`, `tokio`, `rusqlite`, `daemon::Metrics`, `store_schema.rs`, `reaper.rs`, `scripts/check-docs.py`, `scripts/install.sh`, `scripts/sentinel.py`). Aucune duplication evidente non arbitree n'a ete detectee ; les rares nouveaux modules sont des extractions de monolithes ou des capacites sans equivalent existant.

## Synthese

| Metrique | Valeur |
|---|---:|
| Items extraits du plan | 24 |
| Items audites | 24 |
| Reutilisations deja prevues | 19 |
| Existants potentiellement pertinents | 2 |
| Duplications evidentes | 0 |
| Regles/memoires applicables | 7 |
| Specs existantes applicables | 3 |

## Reutilisations correctement identifiees

| Item du plan | Existant reutilise | Preuve | Commentaire |
|---|---|---|---|
| Mise a jour Caddy Go (FR-001) | `deploy/caddy-security/go.mod` et `main.go` | `deploy/caddy-security/go.mod:1` | Etend le module Go existant sans creer de second proxy |
| Assainissement lockfiles (FR-002) | `Cargo.lock` et `android/app/build.gradle.kts` | `crates/bridget-daemon/Cargo.toml:43` | Utilise `resolutionStrategy` deja present dans Gradle |
| Masquage IP + garde docs (FR-003) | `scripts/check-docs.py` et `specs/233-placement-opus2d-mac/validation.md` | `specs/233-placement-opus2d-mac/validation.md:17` | Ajoute la regle dans le validateur existant |
| Plafond connexions Unix (FR-004) | Boucle `accept` de `daemon.rs` | `crates/bridget-daemon/src/daemon.rs:7208` | Etend la boucle existante sans second serveur |
| Delimiteur anti-injection (FR-005a) | `crates/bridget-core/src/envelope.rs` | `crates/bridget-core/src/envelope.rs:29` | Modifie le formatage existant |
| Remplacement `exec()` sentinelle (FR-005b) | `crates/bridget-daemon/src/sentinel.rs` et `scripts/sentinel.py` | `scripts/sentinel.py:174` | Remplace la source injectee par une sous-commande CLI du binaire unique |
| Filtre preventif secrets (FR-005c) | Masquage existant dans `bridget-core` / `bridget-daemon` | `crates/bridget-core/src/lib.rs:1` | Nouveau module pur dans `bridget-core` branche sur le point d'entree persistance |
| Signature detachee `ssh-keygen -Y` (FR-006) | `scripts/install.sh` et `scripts/tests/test_install.py` | `scripts/install.sh:413` | Etend la verification d'integrite existante sans nouvel outil externe |
| `poll(2)` + pipe shutdown (FR-007) | Boucle `accept` de `daemon.rs` + `libc` | `crates/bridget-daemon/src/daemon.rs:7226` | Reutilise `libc` deja verrouille |
| Cache `AgentRegistry` (FR-008) | `crates/bridget-daemon/src/registry.rs` | `crates/bridget-daemon/src/registry.rs:532` | Cache transparent dans `AgentRegistry::load*` |
| Gardes O(1) bornes `bridget-core` (FR-009) | `envelope.rs`, `dedup.rs`, `circuit_breaker.rs` | `crates/bridget-core/src/envelope.rs:82` | Modification en place, 0 dependance |
| SQL groupees `native_delegation::tick` (FR-010) | `daemon/native_delegation.rs` et `delegation_store.rs` | `crates/bridget-daemon/src/daemon/native_delegation.rs:15434` | Etend les requetes du magasin existant |
| Optimisations passerelle mobile (FR-011) | `mobile/store.rs`, `mobile/notifications.rs`, `mobile/http.rs`, `MobileUpdates.kt` | `crates/bridget-daemon/src/mobile/http.rs:1122` | Reutilise `tokio::sync::watch` deja verrouille |
| Decoupage verrou `DaemonState` (FR-012) | `crates/bridget-daemon/src/daemon.rs` | `crates/bridget-daemon/src/daemon.rs:1035` | Remplace un verrou unique par `RwLock` cibles |
| Wrappers POSIX et audit `unsafe` (FR-013) | `bridget-transport` et `bridget-daemon` | `crates/bridget-daemon/src/environment.rs:132` | Regroupe les appels POSIX repetes deja presents |
| Tests et CI bloquante (FR-014) | `.github/workflows/rust.yml`, `rust-audit.yml`, `scripts/tests/` | `.github/workflows/rust.yml:26` | Modifie les workflows existants |
| Decoupage monolithes + Axum (FR-015) | `daemon.rs`, `native_delegation.rs`, `remote_worker.rs`, `protocol.rs`, `wrapper.rs`, `mobile/http.rs` | `crates/bridget-daemon/Cargo.toml:49` | `axum = "=0.8.9"` deja present en dependance |
| Unification SQLite (FR-016) | `store_schema.rs`, `store/mod.rs`, `mobile/store.rs`, `idempotency.rs` | `crates/bridget-daemon/src/store_schema.rs:33` | Applique l'ADR `260-stockage-versions-schema.md` |
| Typage erreurs, `Cargo.toml`, nommage (FR-017) | `Cargo.toml`, `attach.rs`, symboles internes | `Cargo.toml:26` | Modification en place, 0 dependance (`thiserror` evite) |
| Modularisation et evolutions Android (FR-018/FR-019) | `MobileViewModel.kt`, `MainActivity.kt`, `mobile/http.rs` | `android/app/src/main/java/com/bridget/mobile/MobileViewModel.kt:1` | Extraction de fichiers dans le package existant `com.bridget.mobile` |
| Observabilite `status --metrics` / `doctor` / logs JSON (FR-020) | `daemon::Metrics`, `main.rs`, `env_logger` | `crates/bridget-daemon/src/daemon.rs:117` | Reutilise `Metrics` deja calcule et `env_logger` deja present |
| `backup` / `restore --verify` (FR-021) | `rusqlite` bundled (`VACUUM INTO`) et `sha2` | `Cargo.toml:18` | Aucune dependance nouvelle |
| `background_liveness` + `reaper` Phase 1 (FR-022) | `crates/bridget-daemon/src/reaper.rs` | `crates/bridget-daemon/src/reaper.rs:21` | Etend l'observateur Phase 0 existant |
| `--dry-run`, `completions`, schemas MCP (FR-023/FR-024) | `main.rs`, `daemon/native_delegation.rs`, `mcp.rs`, `delegation_mcp.rs` | `crates/bridget-daemon/src/delegation_mcp.rs:195` | Etend le parseur CLI et l'inventaire MCP existants |

## Existant potentiellement pertinent non mentionne

| Item du plan | Existant proche | Preuve | Decision attendue |
|---|---|---|---|
| `bridget doctor` (FR-020) | `bridget status` + `storage_compatibility.rs` + `updates.rs` | `crates/bridget-daemon/src/storage_compatibility.rs:122` | Reutiliser les fonctions de controle de compatibilite et de statut dans `doctor` sans dupliquer la logique |
| Signature detachee (FR-006) | `scripts/install.sh` verification SHA-256 + `MobileRelease.kt` | `scripts/install.sh:413` | Completer la verification SHA-256 existante par la verification `ssh-keygen -Y` optionnelle/configurable |

## Duplications evidentes

Aucune duplication evidente detectee.

## Memoires et regles applicables

| Source | Regle | Impact sur le plan |
|---|---|---|
| `AGENTS.md` | Branche unique `main` (10 nov. 2026) | Pas de branche ni worktree auxiliaire |
| `AGENTS.md` | `scripts/build.py cargo ...` obligatoire | Tous les builds/tests Cargo passent par ce script |
| `AGENTS.md` | Confirmation obligatoire avant tout arret T3 | T3 jamais touche |
| `AGENTS.md` | Anonymat IA absolu dans Git | Format strict `type(scope): Description` |
| `AGENTS.md` | Livraison complete | Build, verification et push verifies ; distinguer sources et runtime |
| `.specify/memory/constitution.md` | Articles XV, XVII, XVIII, XIX, XX | Tests avant suite, menage `menage.py`, 0 N+1/O(n), 0 dependance inutile |
| `docs/decisions/260-stockage-versions-schema.md` | Convergence SQLite | Option C d'abord puis ouverture `WAL` commune |

## Specs livrees applicables

| Spec | Pattern deja etabli | Impact |
|---|---|---|
| `specs/260-assainissement-code` | Structure par lots, portes D1-D16, ADR SQLite, `base-echecs.txt` | Le plan 306 reprend et solde les portes ouvertes sans perdre les acquis |
| `specs/295-diagnostic-activation` / `296-diagnostic-demarrage` | Formats de diagnostic structures et actionnables | Reutilises pour `bridget doctor` |
| `specs/298-questions-mobile` / `299-conversations-mobile` | Contrats HTTP mobiles bornes et tests Android | Etendus de facon retrocompatible en Vague 5 |

## Journal de recherche

| Requete | Portee | Resultat |
|---|---|---|
| `rg -n "fn doctor\|backup\|restore\|completions\|dry_run\|BRIDGET_LOG_FORMAT\|background_liveness\|federation"` | `crates/`, `scripts/` | Aucune commande `doctor`, `backup`, `restore`, `completions`, `dry-run` existante ; `scripts/federate-ssh.sh` trouve |
| `rg -n "axum\|clap\|rusqlite\|serde\|ed25519\|schemars"` | `Cargo.toml`, `crates/*/Cargo.toml` | `axum 0.8.9`, `tokio 1.54.1`, `rusqlite 0.41`, `ring 0.17.14` presents ; pas de `clap` ni `schemars` |
| `rg -n "Connection::open\|journal_mode\|user_version\|schema_version"` | `crates/` | 4 points d'ouverture SQLite confirmes (`store`, `mobile/store`, `idempotency`, `agent_profile`) |
| `rg -n "BRIDGET_SENTINEL_PREPARE_SOURCE\|exec\("` | `crates/`, `scripts/sentinel.py` | 2 sites confirmes (`sentinel.rs:162`, `sentinel.py:174`) |
| `rg -n "fn execute\|Router::new\|/v1/"` | `crates/bridget-daemon/src/mobile/http.rs` | Routage manuel de 509 lignes confirme dans `execute()` |

## Arbitrages

| Sujet | Decision | Justification | Date |
|---|---|---|---|
| `bridget doctor` vs `status` | reutiliser | `doctor` appelle les controleurs existants (`status`, `store_schema`, `Metrics`) et les agrege en un rapport unique | 2026-11-20 |
| Signature detachee | creer nouveau script minimal | `scripts/sign-release.sh` + extension `scripts/install.sh` via `ssh-keygen -Y` natif (0 outil tiers) | 2026-11-20 |
| Filtre de secrets `bridget-core` | creer nouveau module pur | Aucun filtre preventif de persistance n'existe dans `bridget-core` (le masquage actuel est dans les journaux) | 2026-11-20 |

## Gate avant tasks

- [x] Aucune duplication evidente non arbitree
- [x] Chaque item extrait du plan a une ligne d'audit
- [x] Les regles projet applicables ont ete lues
- [x] Les specs existantes proches ont ete verifiees
- [x] Le plan.md a ete refactore ou les divergences sont justifiees
