# Analyse 104 — Analyze et Converge (2026-09-18)

## Verdict

Analyze : aucun finding CRITICAL. Converge (passage 1) : `CONVERGED` — chaque exigence a une preuve
`fichier:ligne` et un test ; deux écarts au plan documentés (index, localisateur history), aucun manque
de spec. Recette release complète verte (1509/0) après une correction du seul banc de performance
(rejeu prescrit sur `storage_unavailable`, voir `implementation.md`). Statut : **Implemented**.

## Findings Analyze et corrections

| # | Finding | Sévérité | Correction |
|---|---|---|---|
| A1 | Plan : second index `(target, ts, id, target)` — colonne finale redondante | low | Index `(target, ts DESC, id DESC)` ; EXPLAIN vérifié (`ledger_requests.rs:1284`) ; consigné ADR 040 et reuse-audit |
| A2 | Plan « history locator » dans les résultats de fil vs data-model (champs sans locateur) | low | Data-model normatif suivi : `thread_id`+`seq` suffisent ; consigne `bridget_thread action=history` dans une notice fixe (`ledger.rs:696`) |
| A3 | `IN (VALUES …)` pour charger les corps laissait le planificateur SQLite préférer un MULTI-INDEX OR (linéaire dans le corpus de l'acteur) | medium | Réécrit en `WITH keys … JOIN ledger` → recherche par clé primaire (`ledger_requests.rs:970`), assertion EXPLAIN |
| A4 | `?N` explicite après des `?` anonymes : « Got 2, needed 3 » (fil) | high (trouvé par test S19) | `?1` d'abord puis anonymes (`ledger_requests.rs:1107`) ; test S19/S21 verts |
| A5 | Motif `content_changed` contenait « nouvelle version » (mots susceptibles d'apparaître dans un corps) | low | Reformulé « version courante » ; le test vérifie l'absence de contenu |
| A6 | Rôle Client négocié (099) : `LedgerSearch` hors rôle, comme `ThreadRequest` | info | Conforme : le MCP passe par `RegisterAuxiliary` (rôle wrapper implicite) ; tests alignés |

Constitution : XVIII (complexité annotée en tête du module `search`, deux plages indexées, fusion bornée, un
seul repli par corps, aucune boucle imbriquée sur le corpus) ; XIX (aucune nouvelle table/dépendance/outil,
ancien moteur retiré, helpers créés listés au reuse-audit avec justification) ; XX (self-review du diff,
tests négatifs, données runtime observées avant chaque correction — journaux `log::warn` sans contenu).

## Converge — exigences → code → test

| Exigence | Preuve code | Test |
|---|---|---|
| FR-001 recherche des corps, termes tous requis, repli | `ledger_requests.rs:12` `fold_char`, `:1151` `locate_terms` ; `ledger.rs:491` `validate` | S01, S02, S03 |
| FR-002 filtres auteur/correspondant/dates inclusives | `ledger.rs:704` `select_prefix`/`admissible` ; SQL `ts >= ?2 AND ts <= ?3` (`ledger_requests.rs:811`) | S04 |
| FR-003 fil : membres seulement | `ledger.rs:1018` `load_thread_for_member` dans la transaction | S19, S31 |
| FR-004 extraits bornés, références stables | `ledger_requests.rs:1172` `excerpt` ; hits `id,target` / `thread_id,seq` (`protocol.rs:3736`+) | S10, S30 |
| FR-005 pagination bornée, suite même sans résultat | `ledger.rs:765` `step` ; `select_prefix` ; curseur `ledger.rs:619/644` | S07, S09, S10 |
| FR-006 ordre déterministe, ni doublon ni perte | `MessageKey::page_cmp` ; row-values SQL ; `last_key` du dernier consommé (`ledger.rs:856`) | S01/S08, S13 |
| FR-007 relecture exacte paginée, changement signalé | `ledger.rs:1211` `read_fragment` ; `ledger_requests.rs:1020` `message_head` | S16, S17, S18 |
| FR-008 identité et droit à chaque appel, curseur sans pouvoir | `daemon.rs:7745` `ledger_read_context`, `:7764` `ledger_identity_still_live` ; `decode_cursor` (acteur, empreinte) ; SQL participant | S06, S11, S22 |
| FR-009 aucune mutation | lecture seule (`store.rs:84`), aucun `INSERT/UPDATE` dans le module | S21 |
| FR-010 erreurs signalées, jamais page vide | `SearchError::Storage` → `storage_unavailable` ; `MetadataTooLarge` | S14, S15 |
| FR-011 réactivité : hors verrou, permis, transaction courte, index | `daemon.rs:10751` (bras hors verrou), `ReadPermits` (`ledger.rs:380`), `drop(tx)` avant CPU, index `store.rs:118` | S23 (unitaire), S24, S25, S26 |
| FR-012 ledger historique et MCP préservés, actions nouvelles | `mcp.rs:988` dispatch `action`, `cli.rs:5867/5908` | S27, contrat transport |
| FR-013 documentation | `commandes.md` § « Recherche dans les échanges (104) », `SKILL.md`, `README.md` | S28 (recette documentée) |
| FR-014 fil non copié, pas d'ACK | résultats construits en mémoire ; aucune écriture | S21 |
| FR-015 extraits non fiables, rendu inerte | `cli.rs:5776` `inert_text` ; JSON brut | S27/S29 |
| SC-001 … SC-007 | voir tests S01/S08, S10/S17, S21, S25/S26, S27 ; bornes de performance assertées en release | `search_104_test.rs` |

Test-plan : 31 scénarios → 24 tests d'intégration (`search_104_test.rs`), 5 tests unitaires
(`spec104_text_tests`, `spec104_index_tests`, `spec104_permit_tests`), 3 tests de contrat
(`protocol.rs`), 1 test client (`client.rs`). S28 est une recette documentaire ; sa chaîne technique
(requête → extrait → `read` ciblé → citation) est exercée par S27.

## Vérifications réellement exécutées

Voir `implementation.md` § « Validation finale » (fmt, clippy, tests ciblés 1293/0, matrice 104 23/0,
recette release n° 1 1507/1 → correction du banc, recette n° 2 1509/0). Mesures debug et release consignées.

## Limites restantes (documentées, hors scope)

- Normalisation Unicode limitée aux accents précomposés du français ; formes décomposées non assimilées.
- Corpus des messages vivant : une purge entre deux pages n'est pas signalée ; recommencer pour une vue fraîche.
- Lecture intégrale d'un corps de 16 Mio par fragments : `B × ceil(B/16 Kio)` (compromis ADR 040).
- Un identifiant hérité > 256 octets dans la fenêtre bloque la page (`source_metadata_too_large`) : restreindre les dates ;
  aucune réécriture automatique.
