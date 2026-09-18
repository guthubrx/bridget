# Journal d'implémentation — 107 Journal WAL

## Métadonnées
- **Spec** : 107-journal-wal — **Branche** : session-107-journal-wal (worktree `.worktrees/107-journal-wal`)
- **Base** : main `b44cbde8` — **Démarré** : 2026-09-18 05:55 — **Statut** : Implemented — 9/9 tâches, recette release verte (1513/0), diff non commité

## Progression

### T001 — Tests unitaires spec107 ✅
- `crates/bridget-daemon/src/store/tests.rs` : `spec107_base_neuve_en_wal_et_base_memoire_acceptee` (l. 1975),
  `spec107_conversion_base_delete_sans_perte_et_idempotente` (l. 1985, 10 000 lignes, somme/max identiques,
  deux réouvertures, connexion tierce en `wal`), `spec107_fichiers_auxiliaires_prives_comme_la_base` (l. 2029,
  0600), `spec107_lecteur_lecture_seule_jamais_bloque_par_ecriture_ni_validation` (l. 2053, écriture ouverte
  puis 50 validations de 64 Kio ; lecteur `busy_timeout` nul).
- Rouge avant T002, avec la mesure du problème : mode `delete` → **96 747 `SQLITE_BUSY` sur 100 001 lectures**.
- Écrivain-écrivain : déjà couvert par `spec101_contended_snapshot_fails_fast_and_restores_timeout` (inchangé).

### T002 — Pragma vérifié ✅
- `crates/bridget-daemon/src/store.rs` l. 80 : `PRAGMA journal_mode=WAL` dans `Store::open`, valeur rendue
  contrôlée (`wal` ou `memory`), sinon `StoreError::Invariant`. Aucun autre réglage.
- T001 vert : 4/4 ; lecteur : **0 `SQLITE_BUSY`** sur le même banc.

### T003 — S14 en WAL ✅
- `crates/bridget-daemon/tests/search_104_test.rs` l. 1014 : écrivain en transaction ouverte → recherche `ok`
  (état validé précédent), validation visible ensuite ; refus réel via `locking_mode=EXCLUSIVE` sur une
  base sans autre connexion ouverte (une connexion WAL ouverte garde un marqueur partagé sur `-shm`, ce qui
  interdit le verrou exclusif : découverte consignée) → `storage_unavailable` < 2 s, jamais `hits=[]`.
- `search_104_test` : 23 réussis, 0 échec (debug).

### T004 — Banc S25 en release ✅
- `MESURE104 s25 archive 100000×1Kio : 200 pages, 264 résultats, p95 page=3,3 ms, max=3,7 ms,
  rejeux storage_unavailable=0 ; 200 DM témoins p95=0,53 ms ; build=release` (contre p95 6,8 ms et
  4,3 ms avant WAL, avec un rejeu observé sous charge).

### T005 — Régressions unitaires ✅
- `cargo test -p bridget-daemon --lib` (store, idempotency, execution_store, agent_profile, artifact_store,
  barrières lecture seule de daemon.rs) : **935 réussis, 0 échec, 10 ignorés** sur base WAL.

### T006 — Déploiement ✅
- `scripts/deploy-remote.sh` : `--exclude='*.db-*'` ajouté à côté de `'*.db'`.

### T007 — Documentation ✅
- `docs/demarrage-a-froid.md` (chemins vivants : fichiers auxiliaires, règle de copie, `VACUUM INTO`),
  `README.md` (paragraphe racine d'état).

### T008 — ADR ✅
- `docs/decisions/041-journal-wal-base-daemon.md` (Accepté ; hors périmètre `synchronous`).

### T009 — Validation finale ✅
- `cargo fmt --all -- --check` : OK. `cargo clippy --workspace --all-targets -- -D warnings` : OK.
- Recette release complète n° 1 : 1510 réussis, **3 échecs** : (a) `core_089_concurrency_test` — huit
  `Store::open` simultanés sur une base neuve, `DatabaseBusy` sur `PRAGMA journal_mode=WAL` (SQLite n'appelle
  pas le gestionnaire d'attente sur cette transition) ; (b) `core_089_migration_test` — le pragma précédait le
  contrôle de schéma et modifiait l'en-tête d'une base future avant refus ; (c)
  `codex_app_server::tests::journal_codex_atteste…` (transport, sans lien). Corrections : `Store::ensure_wal`
  (relecture du mode, réessai borné 2 s) placé après `store_schema::validate`. Rejoués en release ×3 : (a) et
  (b) verts, (c) vert isolé.
- Recette release complète n° 2 (après corrections) : **1513 réussis, 0 échec, 52 ignorés** (`/tmp/b102/r107-release2.log`),
  fmt et clippy OK dans la même passe.
