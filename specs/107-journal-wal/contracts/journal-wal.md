# Contrat 107 — Ouverture de la base

`Store::open(path)` :
1. `Connection::open(path)` ; `busy_timeout` 2 s (inchangé).
2. Validation de schéma (lecture seule, inchangée) : une base d'une version future est refusée sans mutation.
3. `PRAGMA journal_mode` relu ; si ni `wal` ni `memory` : `PRAGMA journal_mode=WAL`, réessayé sur
   `SQLITE_BUSY`/`SQLITE_LOCKED` toutes les 10 ms pendant 2 s au plus ; sinon `Err(StoreError::Invariant(...))`.
4. `init_schema` (inchangé).

Garanties : aucune autre connexion n'est requise ; une base `delete` existante est convertie sans
réécriture des lignes ; une base déjà WAL rend `wal` immédiatement. Lecteurs en lecture seule : pas de
`SQLITE_BUSY` dû à une validation d'écriture. Écrivains : sérialisés, attente `busy_timeout` puis erreur,
comme avant.
