# Tâches - Spec 307 : Hygiène, réactivité et déblocage des automatismes Bridget

## Phase 1 : Extraction modulaire et socle (US7)

- [x] T001 [US7] Extraire le bloc d'entretien périodique (`MAINTENANCE_FIRST_DELAY`, `MAINTENANCE_INTERVAL`, `MEMORY_TRIM_INTERVAL`, `SEND_PURGE_GRACE_SECS`, `IDENTITY_FILES_MIN_AGE`, `sweep_held`, `maintain_stores`, `run_maintenance`, `spawn_maintenance_thread`) de `crates/bridget-daemon/src/daemon.rs` vers `crates/bridget-daemon/src/daemon/maintenance.rs` et relier le module dans `daemon.rs` (vérifiable par `make clippy`).

## Phase 2 : Déblocage des gardes et automatismes (US1, US2)

- [x] T002 [US1] Ajouter `drop_foreign_dead_cwd_records` dans `scripts/build.py` (utilisé dans `busy_paths()` uniquement avec `scopes` non vide, fail-closed sur les processus Cargo/Rust/Bridget/compilateurs et sur commande illisible) et ajouter les tests unitaires correspondants dans `scripts/tests/test_build_cleanup.py`.
- [x] T003 [US2] Ajuster `Updater.step()` dans `scripts/update.py` pour rapporter `up_to_date` lorsqu'aucun candidat plus récent n'est à activer malgré un ancien `client_probe_block_generation`, tout en conservant le blocage sur candidat actif, et ajouter un test dans `scripts/tests/test_updates.py`.
- [x] T004 [US2] Ajouter `purge_expired_auto_admission_abandoned` dans `crates/bridget-daemon/src/delegation.rs` (purger les reçus `pending`/`decision_started`/`chosen` sans `task_id` vieux de plus de 24 h et absents de `native_delegations`), l'appeler dans `maintain_retention` (`crates/bridget-daemon/src/daemon/native_delegation.rs`), et ajouter un test unitaire dans `crates/bridget-daemon/src/delegation_tests.rs`.

## Phase 3 : Hygiène SQLite (US3)

- [x] T005 [US3] Ajouter `purge_expired_auto_admission_admitted` dans `crates/bridget-daemon/src/delegation.rs` (lots <= 512, `state='admitted'`, > 14 jours, absents de `native_delegations`), l'appeler dans `maintain_retention` (`crates/bridget-daemon/src/daemon/native_delegation.rs`), et ajouter un test unitaire dans `crates/bridget-daemon/src/delegation_tests.rs`.
- [x] T006 [US3] Ajouter un entretien SQLite non bloquant (`PRAGMA wal_checkpoint(PASSIVE)`, `PRAGMA incremental_vacuum(256)`, `PRAGMA optimize`) dans `Store::maintain_storage` (`crates/bridget-daemon/src/store.rs`) appelé par `maintain_stores` dans `crates/bridget-daemon/src/daemon/maintenance.rs`, avec test unitaire.

## Phase 4 : Réactivité et lisibilité de `bridget who` (US4)

- [x] T007 [US4] Ajouter `Store::agent_display_names` dans `crates/bridget-daemon/src/store/ledger_requests.rs` (requête unique sur `agent_profiles`) et l'utiliser dans `list_agents()` (`crates/bridget-daemon/src/daemon.rs`) pour supprimer le N+1 SQLite, avec test unitaire.
- [x] T008 [US4] Ajouter l'option `--all` dans `parse_directory_args` et `cmd_who` (`crates/bridget-daemon/src/cli.rs`) pour masquer par défaut les sous-agents natifs `stopped` (UUID) avec compteur discret en pied de liste et affichage complet sous `--all`, avec tests unitaires.

## Phase 5 : Reprise rapide du pont `com.bridget.t3` (US5)

- [x] T009 [US5] Ajouter `IncrementalJournalReader::from_offset` dans `crates/bridget-transport/src/journal.rs` (avec repli automatique à 0 en cas de troncature/rotation) et ses tests unitaires.
- [x] T010 [US5] Persister `journal_offsets: BTreeMap<String, u64>` dans `ThreadState` et remplacer les recherches O(N²) par des `HashSet` dans `confirm_journal` (`crates/bridget-daemon/src/t3code.rs`), avec tests unitaires dans `crates/bridget-daemon/src/t3code_tests.rs`.

## Phase 6 : Relais SSH rapide Mac vers Cartae (US6)

- [x] T011 [US6] Ajouter le support optionnel `mux_slots` (1..16, `--mux-slots`) pour `direction=to-master` dans `scripts/federate-ssh.sh` (pool de sockets `ControlMaster=auto` dans `federation_stream_run`, `federation_connection` inchangé à `ControlMaster=no`) et couvrir par tests dans `scripts/tests/federation_095_test.sh`.

## Phase 7 : Vérification, convergence, audit, livraison et nettoyage

- [x] T012 Exécuter tous les tests ciblés (Python, shell, Rust `bridget-transport` et `bridget-daemon`), `make lint`, `ruff check scripts`, `bash -n scripts/*.sh`, puis exécuter Converge et Audit.
- [x] T013 Committer et pousser sur `main`, compiler `309-hygiene-reactivite-r1` (macOS ARM64 + Linux Debian 12 sur `sysle2` avec `check-linux-abi.py`), déployer et activer sur `monordinateur`, `cartae`, `sysle2` et `ksstore`, basculer `cartae-master177` vers `mux_slots=16` avec reçu réattesté, vérifier les mesures réelles (latence `who`, nettoyage `build.py`, purge `bridget.db`, statut `updates`), et exécuter le ménage Phase 9 (`menage.py` + `scripts/build.py`).
