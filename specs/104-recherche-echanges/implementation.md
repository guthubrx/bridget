# Journal d'implémentation — 104 Recherche dans les échanges

## Métadonnées
- **Spec** : 104-recherche-echanges
- **Branche** : session-104-recherche-echanges (worktree `/Users/moi/Nextcloud/10.Scripts/64.bridget/.worktrees/104-recherche-echanges`)
- **Base** : `2720a0c1 merge(104)` sur main `218c5cc1` (fusion 102 + 103 recettée en release : 1477 réussis, 0 échec)
- **Démarré** : 2026-09-18
- **Statut** : Implemented — 28/28 tâches, recette release verte (1509/0), Analyze/Converge consignés ; contre-revue adverse : voir « Validation finale »

## T001 — Bases et ordre d'intégration ✅
- 102 (fils) et 103 (passation) sont commitées et fusionnées dans main (`218c5cc1`), donc présentes ici :
  `crates/bridget-daemon/src/store/threads.rs` (tables `discussion_*`, `schema_ready`, `load_thread_for_member`).
- Ordre suivi : socle protocole/index → messages (US1–US2) → relecture et fils (US3) → parité, mesures, docs (US4).
- Aucun worktree non commité copié.

## T002 — Harnais isolé ✅
- `crates/bridget-daemon/tests/search_104_test.rs` : racine privée sous `/tmp` (`spec104_root`), base fixture au schéma du daemon
  (`fixture_db`), daemon isolé (`Trio::start`, `spawn_daemon` / `spawn_performance_daemon`), arrêt SIGTERM attendu
  (`stop_cooperatively`), aucun fournisseur réel ; fixtures entièrement synthétiques (`oracle_corpus`, `slow_corpus_at`).
- Test `spec104_harness_isolated_daemon_and_fixture`.

## T003 — Protocole ✅
- `crates/bridget-transport/src/protocol.rs` : `LedgerSearchSource`, `LedgerSearchRequest` (`deny_unknown_fields`, aucun champ
  d'identité), `LedgerSearchHit` (`kind=message|thread_entry`), `LedgerSearchPage`, `LedgerSearchOutcomeV1` (`status=ok|error`),
  `LedgerReadRequest`, `LedgerReadFragment`, `LedgerReadOutcomeV1` ; trames `WrapperToDaemon::LedgerSearch/LedgerRead`
  (`ledger_search`/`ledger_read`) et `DaemonToWrapper::LedgerSearchResult/LedgerReadResult`.
- Tests `spec104_search_contract_tests` (3) : strict, sans identité, ancien contrat `LedgerProjection` inchangé.

## T004 — Index ✅
- `crates/bridget-daemon/src/store.rs` : `idx_ledger_sender_page(sender, ts DESC, id DESC, target DESC)` et
  `idx_ledger_target_page(target, ts DESC, id DESC)` (`CREATE INDEX IF NOT EXISTS`, index existants conservés) ;
  `Store::open_read_only` (READ_ONLY | NO_MUTEX, `busy_timeout` 100 ms, sans `init_schema`).
- Écart au plan : la colonne `target` finale du second index était redondante (colonne de tête). EXPLAIN vérifié par
  `spec104_plages_indexees_et_migration_idempotente` (index utilisés, ni `SCAN ledger` ni `TEMP B-TREE`, réouverture ×2).

## T005 → T010 — US1 ✅
- Tests : S01/S08 oracle (1000 + 1000 lignes, 6 requêtes, ordre `(ts,id,target)` DESC, aucun doublon), S02/S03 (accents, casse,
  jokers littéraux, emoji, NFD documentée), S04 (filtres inclusifs, message à soi-même, rejets comptés), S05 (13 refus
  `invalid_params` sur connexion sans table = aucun SQL), S06 (tiers invisibles, compteurs muets, identité absente).
- `store/ledger_requests.rs` : `fold_char` (règle unique, `Ÿ→y` ajouté), `fold_for_search`, `folded_len`, module `search`
  (`MessageKey::page_cmp`, `message_upper_bound`, `message_candidates` : deux plages ≤129 + fusion ≤258, métadonnées >256 o
  détectées en SQL avant matérialisation, `octet_length(body)` sans lecture du corps ; `message_bodies` : une requête
  `WITH keys … JOIN` par clé primaire, participation revérifiée, ≤16 Mio ; `locate_terms` : repli unique + reconversion
  d'offset par un parcours ; `excerpt` 512 o à frontière UTF-8).
- `ledger.rs` module `search` : validation (256 o, 1–8 termes, limit 1–50, dates, UUID canoniques, exclusions source),
  sélection bornée (`select_prefix` : 128 candidats, 1 Mio + une ligne entière, >16 Mio ignoré), `PageBuilder::step`
  (limite avant traitement, budget de réponse 61 440 o avec premier résultat garanti), notices fixes.
- `daemon.rs` : bras `LedgerSearch`/`LedgerRead`, `ledger_read_context` (identité + permis sous verrou bref, chemin de base
  configuré), `ledger_identity_still_live` (revérification avant publication), `ReadPermits` (2, RAII) ; matrices de rôles
  alignées sur `ThreadRequest` (MCP auxiliaire = rôle wrapper implicite ; rôle Client négocié : hors rôle).
- `mcp.rs` : `bridget_ledger` `action=recent|search|read` (`execute_ledger_search/read`), schéma fermé, ancien contrat
  intact (les paramètres de recherche sont refusés sans action).
- `cli.rs` : `bridget ledger search|read` (`parse_ledger_search_args`, `parse_ledger_read_args`, `render_ledger_search`,
  `inert_text`, codes 0/2/1), `ledger --limit` inchangé, aide mise à jour.
- `communication/client.rs` : `ledger_search`, `ledger_read` (Nack → `daemon_protocol`, jamais de repli) ;
  test `spec104_ancien_daemon_erreur_honnete_sans_repli`.

## T011 → T014 — US2 ✅
- Tests : S07 (128 non-correspondances → `hits=[]` + curseur ; `exhausted` prime sur la dernière ligne ; `result_limit`
  n'acquitte pas la ligne suivante), S09 (budget 1 Mio avec une ligne entière au-delà ; >16 Mio ignoré sans chargement ;
  16 Mio exact traité), S10 (50 extraits de 512 o ≤ 60 Kio ; `response_budget` sans consommer le résultat refusé ; aucune
  perte sur 128 lignes), S11/S12 (curseur d'autrui, requête différente, filtres changés, forgé sur clé tierce, 12 malformés),
  S13 (insertion récente hors borne, purge et remplacement entre pages, relecture honnête), S14 (`BEGIN EXCLUSIVE` →
  `storage_unavailable` <2 s ; ligne BLOB non UTF-8 propagée ; table absente), S15 (identifiant hérité 257 o → refus sans
  valeur, dates restreintes → succès).
- Curseur : hex minuscule d'un JSON strict `{v:1, actor, fingerprint, source, upper, before}` ≤16 384 o ; empreinte SHA-256
  d'une structure à ordre fixe (source, query brute, author, peer, since, until, thread_id) ; `limit` libre.

## T015 → T020 — US3 ✅
- Tests : S16/S17/S18 (même `id` vers deux cibles, fragments 16 Kio avec emoji sur la limite, reconstruction exacte, digest,
  `offset=body_bytes`, milieu d'UTF-8 refusé, `content_changed` sans morceau, `source_too_large`), S30 (préfixe `ŸÉ`
  changeant la longueur du repli : `match_offset` original exact, `read` ciblé), S19/S20 (membre : 13 résultats sur 130,
  instantané `last_seq` malgré l'entrée 131 concurrente, filtres, non-membre et fil absent : même refus et même texte,
  curseur de fil chez un non-membre refusé), S21 (dump de 7 tables identique avant/après, aucune remise, lecture 102 non
  avancée), S22 (owner coupé pendant le repli d'un corps de 16 Mio via connexion auxiliaire : `identity_unavailable`,
  permis rendu), S31 (tables 102 supprimées : `capability_unavailable`, messages toujours cherchables).
- `ledger.rs::search_thread` : `schema_ready` puis `load_thread_for_member` dans la transaction de lecture ; `upper =
  last_seq` en première page ; `thread_candidates`/`thread_bodies` par `(thread_id, seq)` ; aucune écriture.

## T021 → T025 — US4 ✅
- S27/S29 : catalogue MCP inchangé (20 outils), schéma `action` fermé, parité `hits` CLI `--json` ↔ MCP, JSON brut (ANSI
  conservé), rendu humain sans `ESC`, injection SQL inerte (table présente), 7 cas de code 2, refus code 1, zéro résultat
  code 0, `ledger --limit 5` inchangé.
- S24 : deux recherches de 16 Mio concurrentes, troisième `busy` immédiat, annuaire réactif pendant le travail, permis rendus.
- T022 : `search_messages`, `folded_body_sql`, `escape_like_needle`, `LedgerSearchOutcome`, `MAX_LEDGER_SEARCH` retirés
  après `rg` (aucun appelant hors du module) ; `fold_for_search` conservée (utilisée par `agent_profile.rs`).
- T025 : `skills/bridget/references/commandes.md` (section « Recherche dans les échanges (104) », tableau 094, catalogue),
  `skills/bridget/SKILL.md` (« Chercher, continuer, relire, citer », sans nouvel exemple JSON pour ne pas casser le test
  089), `README.md`.
- Mesures (debug, poste de développement, 2026-09-18, `MESURE104`) : archive 100 000 × 1 Kio → 200 pages, p95 = 49 ms,
  max = 82 ms ; 200 messages directs témoins simultanés p95 = 43 ms ; trois corps de 16 Mio → 3 pages, max = 1,5 s,
  RSS daemon 12 → 51 Mio (+38 Mio).
- Mesures **release** (`cargo test --release --test search_104_test spec104_s2 -- --nocapture --test-threads=1`,
  macOS, Apple Silicon, poste de développement, 2026-09-18, bornes SC-005/SC-006 assertées et vertes) :
  - archive 100 000 lignes × 1 Kio (fichier ≈ 110 Mo) : 200 pages de recherche (`limit 50`, 264 résultats),
    **p95 = 6,8 ms**, max = 90 ms ; 200 messages directs témoins simultanés **p95 = 4,3 ms** ;
  - trois corps de 16 Mio : 3 pages (un corps hors budget par page), **max = 135 ms** ; RSS du daemon 8 → 45 Mio
    (**+36 Mio** < 128 Mio), mesure séparée du générateur (le corpus est écrit avant le démarrage du daemon) ;
  - deux recherches de 16 Mio concurrentes : 148 ms au total, troisième `busy` immédiat, annuaire réactif.
  - EXPLAIN QUERY PLAN réel (sqlite3 3.51, même schéma) : `SEARCH ledger USING INDEX idx_ledger_sender_page
    (sender=? AND ts>? AND ts<?)` ; `SEARCH ledger USING INDEX idx_ledger_target_page (target=? AND ts>? AND ts<?)` ;
    chargement des corps : `SCAN keys` (CTE de ≤128 lignes) puis `SEARCH l USING INDEX sqlite_autoindex_ledger_1
    (id=? AND target=?)`. Aucun `SCAN ledger`, aucun `TEMP B-TREE` (assertion unitaire).

## Validation finale (T026–T028)
- Commandes exécutées (worktree 104, 2026-09-18) :
  - `cargo fmt --all -- --check` : OK.
  - `cargo clippy --workspace --all-targets -- -D warnings` : OK (0 avertissement).
  - `cargo test -p bridget-transport -p bridget-daemon --lib --test search_104_test --test spec102_threads_test
    --test handoff_103_test --test core_089_ledger_test --test core_089_skill_test --test claude_native_permissions_test`
    (debug, `BRIDGET_HOME` vide, `umask 077`) : 1293 réussis, 0 échec (avant les retouches de documentation).
  - `cargo test --test search_104_test` : 23 réussis (24 avec le harnais), 0 échec, 1 ignoré (worker de performance privé).
  - Recette release complète n° 1 (`cargo test --workspace --release --no-fail-fast`, `umask 077`, `BRIDGET_HOME` vide) :
    1507 réussis, **1 échec**, 52 ignorés. L'échec : `spec104_s25_s26_performance…` — pendant le banc à 200 messages
    directs, une page de recherche a reçu `storage_unavailable` (attente SQLite de 100 ms dépassée pendant une validation
    d'écriture du daemon, journal rollback + fsync, sous la charge de la recette complète). C'est le comportement prescrit
    par le contrat (« SQL échouée → storage_unavailable, réessayer ») ; seul le banc traitait ce refus comme fatal. Le
    banc rejoue désormais la même page (borne 50 rejeux, latence mesurée rejeux compris, nombre rapporté). Aucun autre
    échec ; aucune modification du daemon.
  - Recette release complète n° 2 (même commande, après cette seule correction de test) : **1509 réussis, 0 échec,
    52 ignorés** (`/tmp/b102/r104-release2.log`).
  - Tests unitaires 104 + inventaire 094 + exemples skill 089 sur build séparé (debug) : 7 + 1 réussis, 0 échec.
- Contre-revue adverse : demandée à cursor-listen (Cursor, `04c521fe-…`) le 2026-09-18 05:17 (borne 25 min), puis en
  repli à horizon-cursor (Cursor, `7e9dec19-…`) à 05:29 (borne 20 min) ; traçabilité et issue dans
  `adversarial-review-cursor-listen.md`.

## Observation à retenir (fonctionnement)
Sans mode WAL, une validation d'écriture du daemon bloque brièvement les lecteurs ; avec l'attente de 100 ms, une
recherche concurrente peut recevoir `storage_unavailable` sous forte charge d'E/S. Le client rejoue la même page (même
curseur) : aucune perte ni doublon. Activer WAL n'est pas dans le périmètre 104 (le socle ne le configure pas).
