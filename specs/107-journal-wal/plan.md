# Plan 107 — Journal WAL

## Base

Worktree `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/107-journal-wal`, branche
`session-107-journal-wal`, base main `b44cbde8` (102, 103, 104 intégrées).

## Décision

Dans `Store::open` (`crates/bridget-daemon/src/store.rs`), après la validation de schéma (lecture seule) et
avant `init_schema` : relire `PRAGMA journal_mode`, poser `PRAGMA journal_mode=WAL` si nécessaire avec un
réessai borné (2 s) sur `SQLITE_BUSY` — SQLite n'appelle pas le gestionnaire d'attente sur cette
transition — et vérifier la valeur rendue ; si elle n'est ni `wal` ni `memory` (base en mémoire), retourner
`StoreError::Invariant("journal WAL refusé")`. (Amendé après la recette : voir analysis.md A4/A5.) Aucune autre modification de
réglage (`synchronous` reste `FULL`).

Pourquoi ici : `Store::open` est le premier ouvreur au démarrage du daemon (`daemon.rs:2919`) et le mode
est persistant dans le fichier ; toutes les connexions suivantes (idempotence, exécution, profils,
artefacts, lecteurs 104, préflight) l'héritent. Un seul point de vérité, une ligne de contrat.

## Tests

- `store/tests.rs` (unitaires) : mode `wal` sur base neuve et sur base `delete` existante avec données,
  idempotence à la réouverture, `:memory:` accepté, droits 0600 de `-wal`/`-shm`, lecteur en lecture seule
  non bloqué pendant une transaction d'écriture (écriture non validée puis validation, 100 lectures,
  0 `SQLITE_BUSY`), écrivain-écrivain toujours borné.
- `tests/search_104_test.rs` : S14 provoque `SQLITE_BUSY` par `locking_mode=EXCLUSIVE` ; S25 rapporte
  `rejeux = 0`.
- Recette : tests unitaires store, `search_104_test`, puis recette release complète.

## Documentation

`docs/demarrage-a-froid.md` (fichiers auxiliaires, règle de copie), `README.md` (une phrase),
`scripts/deploy-remote.sh` (exclure `*.db-*`), ADR 041.

## Fichiers prévus

- `crates/bridget-daemon/src/store.rs` (+ tests dans `crates/bridget-daemon/src/store/tests.rs`)
- `crates/bridget-daemon/tests/search_104_test.rs` (S14, S25)
- `docs/demarrage-a-froid.md`, `README.md`, `scripts/deploy-remote.sh`
- `docs/decisions/041-journal-wal-base-daemon.md`

## Complexité et minimalisme

Aucune boucle, aucune structure : un pragma et une vérification. Aucun helper : un seul point d'ouverture
a besoin de la règle. Charge cognitive future : une ligne à connaître, documentée dans l'ADR.
