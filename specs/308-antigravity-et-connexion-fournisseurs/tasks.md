# Tâches - Spec 308 : Google Antigravity et connexion facilitée multi-fournisseurs

## Phase 1 : Prérequis serveurs (US5)
- [x] T001 [US5] Installer `agy` (`https://antigravity.google/cli/install.sh`) sur `cartae`, `sysle2` et `ksstore` et vérifier `agy --version` sur les 4 machines.

## Phase 2 : Transport ACP et moteur `antigravity` (US1, US2)
- [x] T002 [US1] Étendre `AcpOptions` et `AcpTransport` dans `crates/bridget-transport/src/acp.rs` (`client_info`, `auth_method_id`, `session_mode`, `model`, `effort`) avec tests unitaires dans `crates/bridget-transport/src/acp_tests.rs`.
- [x] T003 [US2] Ajouter `antigravity_home: Option<String>` dans `ResolvedAgentDefinition` (`crates/bridget-transport/src/protocol.rs`) et `AgentDefinition` (`crates/bridget-daemon/src/registry.rs`), autoriser `("antigravity", "acp")` et `native_antigravity_definition()`, préserver les digests existants, avec tests dans `registry_tests.rs`.
- [x] T004 [US2] Raccorder `antigravity` dans `crates/bridget-daemon/src/lifecycle.rs`, `wrapper.rs`, `client_compatibility.rs`, `provider_accounts.rs` (`prepare` pour `antigravity`/`acp`) et `cli.rs` (`bridget agy`), avec tests unitaires.

## Phase 3 : Connexion centralisée CLI et fenêtre graphique locale (US3, US4)
- [x] T005 [US3] Créer `crates/bridget-daemon/src/accounts_connect.rs` : `bridget accounts status [--json]` multi-machines/multi-fournisseurs (Claude, Codex, Antigravity) et `bridget accounts login` (local + SSH interactif, tunnel OAuth Codex, ouverture auto URL sur Mac, `--logout-first`, création+connexion+activation distante `--from ... --prepare-id ...`).
- [x] T006 [US4] Ajouter `bridget accounts ui` dans `crates/bridget-daemon/src/accounts_connect.rs` (serveur `axum` strictement local `127.0.0.1`, jeton de session aléatoire, page HTML/CSS/JS embarquée avec statut en direct, boutons « Se connecter / Changer de compte », ouverture auto OAuth + champ code, formulaire « + Nouveau compte isolé ») + tests unitaires.
- [x] T007 [US4] Créer le lanceur macOS `/Users/moi/Applications/Bridget Comptes.app` ouvrant `bridget accounts ui`.

## Phase 4 : Vérification, déploiement et ménage (US5)
- [x] T008 Exécuter `make fmt`, `make lint` et tous les tests ciblés (`bridget-transport` ACP, `registry`, `provider_accounts`, `client_compatibility`, `accounts_connect`, `wrapper`).
- [x] T009 Committer et pousser sur `main`, compiler `311-antigravity-comptes-r1` (macOS ARM64 + Linux Debian 12 sur `sysle2` avec `check-linux-abi.py`), déployer et activer sur les 4 machines, ajouter les connexions `antigravity` dans les `agents.json` (avec sauvegardes), vérifier `bridget accounts status` et `bridget accounts ui`, et exécuter le ménage.
