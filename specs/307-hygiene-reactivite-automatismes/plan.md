# Plan d'implémentation - Spec 307

## Architecture retenue

Tous les changements étendent des modules existants avec une seule nouvelle unité extraite (`daemon/maintenance.rs`), aucune nouvelle dépendance externe, aucune migration de schéma SQL, et compatibilité bidirectionnelle totale des formats de stockage.

| Lot | Fichiers touchés | Complexité / Invariants |
|---|---|---|
| Lot 1 (build.py) | `scripts/build.py`, `scripts/tests/test_build_cleanup.py` | O(P) sur les lignes `lsof` ; fail-closed sur tout processus Cargo/Rust/Bridget ou nom illisible |
| Lot 2 (updates) | `scripts/update.py`, `scripts/tests/test_updates.py`, `crates/bridget-daemon/src/delegation.rs`, `crates/bridget-daemon/src/daemon/native_delegation.rs` | Purge bornée (<= 128) des admissions abandonnées > 24 h sans tâche ; garde `test209` préservée |
| Lot 3 (SQLite) | `crates/bridget-daemon/src/delegation.rs`, `crates/bridget-daemon/src/store.rs`, `crates/bridget-daemon/src/daemon/maintenance.rs`, `crates/bridget-daemon/src/delegation_tests.rs` | Purge bornée (<= 512) des reçus `admitted` > 14 j sans tâche ; `PRAGMA wal_checkpoint(PASSIVE)` + `incremental_vacuum(256)` + `optimize` |
| Lot 4 (who) | `crates/bridget-daemon/src/store/ledger_requests.rs`, `crates/bridget-daemon/src/daemon.rs`, `crates/bridget-daemon/src/cli.rs` | 1 requête SQL au lieu de N+1 ; filtrage CLI par défaut des UUID `stopped` + `--all` |
| Lot 5 (t3code) | `crates/bridget-transport/src/journal.rs`, `crates/bridget-daemon/src/t3code.rs`, `crates/bridget-daemon/src/t3code_tests.rs` | Reprise O(delta) via `journal_offsets` + recherche O(1) par `HashSet` au lieu de O(N²) |
| Lot 6 (SSH mux) | `scripts/federate-ssh.sh`, `scripts/tests/federation_095_test.sh` (ou test dédié) | `ControlMaster=no` préservé dans `federation_connection` ; pool privé `m0..m{N-1}.sock` opt-in sur `to-master` |
| Lot 7 (maintenance.rs) | `crates/bridget-daemon/src/daemon.rs`, `crates/bridget-daemon/src/daemon/maintenance.rs` | Extraction pure sans changement d'interface ni de verrouillage |

## Vérifications et livraison

1. Tests unitaires Python (`test_build_cleanup.py`, `test_updates.py`) + `ruff check scripts` + `bash -n scripts/*.sh`.
2. Tests shell fédération (`federation_089_test.sh`, `federation_095_test.sh` + nouveau test `mux_slots`).
3. `make lint` (`clippy -D warnings` + `fmt-check`) et tests Rust ciblés (`bridget-transport` `journal::`, `bridget-daemon` `delegation`, `native_delegation`, `t3code`, `cli`, `store`).
4. Commit et push sur `main`, compilations release `309-hygiene-reactivite-r1` (macOS ARM64 + Linux Debian 12 sur `sysle2` avec `check-linux-abi.py`), vérification compatibilité stockage bidirectionnelle, déploiement et activation sur `monordinateur`, `cartae`, `sysle2`, `ksstore`, bascule `cartae-master177` vers `mux_slots=8` avec mesure de latence réelle avant/après, puis ménage Phase 9.
