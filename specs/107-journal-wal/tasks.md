# Tâches 107

Statut: Implemented — 9/9 (2026-09-18). Recette release complète 1513/0, fmt/clippy OK, Analyze/Converge dans analysis.md.

## Socle

- [x] T001 Écrire les tests unitaires `spec107_*` dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/107-journal-wal/crates/bridget-daemon/src/store/tests.rs ; attendu : mode `wal` sur base neuve, conversion d'une base `delete` de 10 000 lignes sans perte et idempotente, `:memory:` accepté, droits 0600 de `-wal`/`-shm`, lecteur lecture seule sans `SQLITE_BUSY` pendant écriture et validation (100 lectures), écrivain-écrivain toujours borné ; échecs attendus avant T002. Couverture : FR-001…FR-008, SC-001, SC-002, SC-005.
- [x] T002 Poser et vérifier `PRAGMA journal_mode=WAL` dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/107-journal-wal/crates/bridget-daemon/src/store.rs (`Store::open`) ; attendu : erreur explicite si la valeur rendue n'est ni `wal` ni `memory` ; T001 vert. Couverture : FR-001, FR-002, FR-003.

## Régressions

- [x] T003 Adapter S14 dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/107-journal-wal/crates/bridget-daemon/tests/search_104_test.rs ; attendu : `SQLITE_BUSY` réel provoqué par `locking_mode=EXCLUSIVE`, assertion `storage_unavailable` conservée, et un cas supplémentaire : écrivain en transaction → lecteur 104 répond `ok`. Couverture : FR-004, FR-009.
- [x] T004 Exécuter `search_104_test` (dont S25 en release, `--nocapture`) et consigner `rejeux storage_unavailable` dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/107-journal-wal/specs/107-journal-wal/implementation.md ; attendu : 0 rejeu. Couverture : SC-003.
- [x] T005 Exécuter les tests unitaires `store`, `idempotency`, `execution_store`, `agent_profile`, `artifact_store` et les tests de barrière (`daemon.rs` lecture seule) ; attendu : tous verts sur base WAL. Couverture : FR-006, FR-008.

## Exploitation et documentation

- [x] T006 Étendre l'exclusion `*.db-*` dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/107-journal-wal/scripts/deploy-remote.sh ; attendu : `-wal`/`-shm` jamais synchronisés. Couverture : FR-010.
- [x] T007 Documenter fichiers auxiliaires et règle de copie dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/107-journal-wal/docs/demarrage-a-froid.md et /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/107-journal-wal/README.md ; attendu : un opérateur sait qu'une copie à chaud de `bridget.db` seul est incomplète. Couverture : FR-010.
- [x] T008 Rédiger l'ADR /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/107-journal-wal/docs/decisions/041-journal-wal-base-daemon.md ; attendu : contexte, décision, conséquences, hors périmètre `synchronous`. Couverture : FR-005.

## Validation finale

- [x] T009 fmt, clippy, recette release complète du workspace, puis Analyze/Converge dans /Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/107-journal-wal/specs/107-journal-wal/analysis.md ; attendu : 0 échec, correspondance exigences → code → tests. Couverture : SC-004.

Exécution T001→T009. Pas de commit automatique.
