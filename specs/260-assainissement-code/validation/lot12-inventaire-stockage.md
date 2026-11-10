# Inventaire du stockage SQLite - lot 12

Relevé fait le 10 novembre 2026 sur `main` (`50b9a119`), par `rg` hors fichiers de tests. Rien n'a été modifié. Une cellule « non lu » n'a pas été relevée.

## 1. Mécanismes de version de schéma

| Mécanisme | Fichiers | Ce qu'il compare | Base plus récente que le code |
|-----------|----------|------------------|-------------------------------|
| Table de migrations (`execution_schema_migrations`, `MAX(version)`) | `execution_store.rs` (16 lignes concernées) | une version par ligne insérée avec `INSERT OR IGNORE` | non lu |
| Préflight de version d'idempotence (`MAX_IDEMPOTENCY_SCHEMA_VERSION = 8`) | `store_schema.rs` (7 lignes), `idempotency.rs` (12) | une version maximale connue, avant la migration | refus `UnsupportedVersion { found, supported }`, « aucune migration » (message lu dans `store_schema.rs`) |
| `PRAGMA user_version` | `mobile/store.rs` (lignes 253 et 286) | `user_version` 1 puis 2 pour le magasin mobile | non lu |
| `PRAGMA table_info` / `pragma_table_info` (détection de colonnes) | `store/threads.rs` (11), `store/project_compat.rs`, `execution_store.rs`, `idempotency.rs`, `store.rs`, `agent_profile.rs` | présence d'une colonne, puis ajout si absente | non lu |
| Autres fichiers qui parlent de `schema_version` | `artifact_store.rs` (2), `agent_tracking.rs` (2), `agent_profile.rs` (1) | non lu | non lu |

Constat : au moins quatre façons de savoir « où en est la base » coexistent (table de migrations, constante maximale, `user_version`, détection de colonnes).

## 2. Ouvertures de bases

`Connection::open` : 111 occurrences hors fichiers de tests nommés `*_test.rs`, `*_tests.rs`, `tests.rs` (les fichiers de `crates/*/tests/` sont inclus dans ce compte).
Principaux fichiers : `daemon.rs` 20 ; `daemon/native_delegation.rs` 17 ; `delegation.rs` 10 ; `t3code_identity.rs` 9 ; `tests/native_delegation_e2e.rs` 8 ; `tests/held159_e2e.rs` 6 ; `tests/edit159_e2e.rs` 5 ; `referent_control.rs` 4 ; `store.rs` 3 ; `provider_limits.rs` 3.

## 3. Modes de journal et attentes

| Base ou fichier | Mode relevé | Raison écrite dans le code |
|-----------------|-------------|----------------------------|
| Base du démon (`store.rs`) | `PRAGMA journal_mode=WAL` (confirmé ou basculé à l'ouverture ; une base déjà en WAL est confirmée sans verrou) | commentaire « Session 107 : journal en écriture anticipée (WAL), propriété persistante » |
| Spool de remise (`delivery_spool244.rs`) | `journal_mode=DELETE; synchronous=FULL`, attente de 1 s | raison non écrite dans les lignes lues |
| Boîte de réponses (`reply_outbox244.rs`) | `journal_mode=DELETE; synchronous=FULL` | raison non écrite dans les lignes lues |
| Autres bases (`delegation.rs`, `execution_store.rs`, `artifact_store.rs`, `idempotency.rs`, `mobile/store.rs`, `lifecycle.rs`) | `busy_timeout` de 2 s à 5 s ; mode de journal non défini dans les lignes lues | non lu |

## 4. Schéma centralisé

`store_schema.rs` : préflight en lecture seule appelé avant les migrations des deux magasins d'idempotence. Version maximale connue : 8. Il vérifie le type et l'UID du chemin avant la socket. Il ne couvre pas les autres magasins (exécution, mobile, artefacts).

## 5. Écarts

- Les magasins n'ont pas le même mécanisme de version.
- Un seul préflight de version avant migration (idempotence). Les autres magasins : non lu.
- Deux jeux de réglages de journal : WAL pour la base du démon, DELETE+FULL pour deux petits magasins de remise.
- 111 ouvertures réparties dans de nombreux fichiers, sans point d'entrée unique relevé.
